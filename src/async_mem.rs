//! Asynchronous memory operations for file-backed storage.
//!
//! This module provides async access to memory-mapped file storage by using a
//! dedicated I/O thread with a producer-consumer command queue. The mmap is owned
//! by the I/O thread, so page faults never block async task threads.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────┐     commands      ┌──────────────────┐
//! │  async tasks  │ ───────────────► │  I/O thread       │
//! │  (tokio)      │ ◄─────────────── │  (owns FileMapped)│
//! └──────────────┘     responses     └──────────────────┘
//! ```
//!
//! Other threads submit operations (grow, shrink, get, set, etc.) through a
//! channel and await responses via oneshot channels, without being blocked
//! by mmap page faults.
//!
//! # Example
//!
//! ```ignore
//! use platform_mem::AsyncFileMem;
//!
//! async fn example() -> Result<(), platform_mem::Error> {
//!     let mut mem = AsyncFileMem::<u64>::from_path("data.bin").await?;
//!
//!     // Async grow with zero initialization
//!     mem.grow_zeroed(1000).await?;
//!
//!     // Read/write operations (dispatched to I/O thread)
//!     mem.set(0, 42).await?;
//!     let val = mem.get(0).await?;
//!     assert_eq!(val, Some(42));
//!
//!     Ok(())
//! }
//! ```

use std::{
    fmt,
    fs::File,
    io,
    marker::PhantomData,
    path::Path,
    sync::mpsc,
    thread,
};

use crate::{file_mapped::FileMapped, Error, RawMem};

/// A command sent to the I/O thread.
enum Command<T: Copy + Send + 'static> {
    /// Get the number of allocated elements.
    Len(tokio::sync::oneshot::Sender<usize>),

    /// Get a value at an index.
    Get(usize, tokio::sync::oneshot::Sender<Option<T>>),

    /// Set a value at an index. Returns true if index was valid.
    Set(usize, T, tokio::sync::oneshot::Sender<bool>),

    /// Read a range of values as a Vec.
    ReadSlice(usize, usize, tokio::sync::oneshot::Sender<Result<Vec<T>, Error>>),

    /// Write values starting at an offset.
    WriteSlice(usize, Vec<T>, tokio::sync::oneshot::Sender<Result<(), Error>>),

    /// Grow with fill value (grow_filled).
    GrowFilled(usize, T, tokio::sync::oneshot::Sender<Result<(), Error>>),

    /// Grow with zeros (grow_zeroed).
    GrowZeroed(usize, tokio::sync::oneshot::Sender<Result<(), Error>>),

    /// Grow assuming file data is already initialized (grow_assumed).
    GrowAssumed(usize, tokio::sync::oneshot::Sender<Result<(), Error>>),

    /// Shrink by count elements.
    Shrink(usize, tokio::sync::oneshot::Sender<Result<(), Error>>),

    /// Shutdown the I/O thread.
    Shutdown(tokio::sync::oneshot::Sender<()>),
}

/// Asynchronous file-backed memory storage using a dedicated I/O thread.
///
/// This type wraps `FileMapped<T>` and runs all mmap operations on a single
/// dedicated thread. Async callers submit commands through a channel and
/// receive results without being blocked by page faults.
///
/// No additional memory buffers are allocated — the mmap is the sole data store.
pub struct AsyncFileMem<T: Copy + Send + 'static> {
    /// Channel to send commands to the I/O thread.
    tx: Option<mpsc::Sender<Command<T>>>,
    /// Handle to the I/O thread (joined on drop).
    thread: Option<thread::JoinHandle<()>>,
    /// Marker for the element type.
    _marker: PhantomData<T>,
}

/// Start the I/O thread that owns the FileMapped and processes commands.
fn spawn_io_thread<T: Copy + Clone + Send + 'static>(
    mut mem: FileMapped<T>,
    rx: mpsc::Receiver<Command<T>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                Command::Len(reply) => {
                    let _ = reply.send(mem.allocated().len());
                }
                Command::Get(index, reply) => {
                    let val = mem.allocated().get(index).copied();
                    let _ = reply.send(val);
                }
                Command::Set(index, value, reply) => {
                    let slice = mem.allocated_mut();
                    let ok = if index < slice.len() {
                        slice[index] = value;
                        true
                    } else {
                        false
                    };
                    let _ = reply.send(ok);
                }
                Command::ReadSlice(offset, count, reply) => {
                    let slice = mem.allocated();
                    let result = if offset.saturating_add(count) <= slice.len() {
                        Ok(slice[offset..offset + count].to_vec())
                    } else {
                        Err(Error::CapacityOverflow)
                    };
                    let _ = reply.send(result);
                }
                Command::WriteSlice(offset, values, reply) => {
                    let slice = mem.allocated_mut();
                    let result = if offset.saturating_add(values.len()) <= slice.len() {
                        slice[offset..offset + values.len()].copy_from_slice(&values);
                        Ok(())
                    } else {
                        Err(Error::CapacityOverflow)
                    };
                    let _ = reply.send(result);
                }
                Command::GrowFilled(count, value, reply) => {
                    let result = mem.grow_filled(count, value).map(|_| ());
                    let _ = reply.send(result);
                }
                Command::GrowZeroed(count, reply) => {
                    let result = unsafe { mem.grow_zeroed(count).map(|_| ()) };
                    let _ = reply.send(result);
                }
                Command::GrowAssumed(count, reply) => {
                    let result = unsafe { mem.grow_assumed(count).map(|_| ()) };
                    let _ = reply.send(result);
                }
                Command::Shrink(count, reply) => {
                    let result = mem.shrink(count);
                    let _ = reply.send(result);
                }
                Command::Shutdown(reply) => {
                    // FileMapped syncs on drop
                    drop(mem);
                    let _ = reply.send(());
                    return;
                }
            }
        }
        // Channel closed without Shutdown — just drop FileMapped (syncs on drop)
    })
}

impl<T: Copy + Clone + Send + 'static> AsyncFileMem<T> {
    /// Creates a new async file memory from a file path.
    ///
    /// The file is opened (or created) and memory-mapped. A dedicated I/O
    /// thread is spawned to handle all mmap operations.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mem = FileMapped::<T>::from_path(path)?;
        Ok(Self::from_file_mapped(mem))
    }

    /// Creates a new async file memory backed by a temporary file.
    pub fn temp() -> Result<Self, Error> {
        let file = tempfile::tempfile()?;
        let mem = FileMapped::<T>::new(file)?;
        Ok(Self::from_file_mapped(mem))
    }

    /// Creates an AsyncFileMem from an existing File.
    pub fn from_file(file: File) -> Result<Self, Error> {
        let mem = FileMapped::<T>::new(file)?;
        Ok(Self::from_file_mapped(mem))
    }

    /// Internal: wrap a FileMapped in an async handle with a dedicated I/O thread.
    fn from_file_mapped(mem: FileMapped<T>) -> Self {
        let (tx, rx) = mpsc::channel();
        let thread = spawn_io_thread(mem, rx);

        Self {
            tx: Some(tx),
            thread: Some(thread),
            _marker: PhantomData,
        }
    }

    /// Helper to send a command and await the response.
    async fn send_command<R: Send + 'static>(
        &self,
        make_cmd: impl FnOnce(tokio::sync::oneshot::Sender<R>) -> Command<T>,
    ) -> Result<R, Error> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        let cmd = make_cmd(reply_tx);

        self.tx
            .as_ref()
            .expect("AsyncFileMem already shut down")
            .send(cmd)
            .map_err(|_| {
                io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread terminated unexpectedly")
            })?;

        reply_rx.await.map_err(|_| {
            Error::from(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "I/O thread dropped reply channel",
            ))
        })
    }

    /// Returns the number of elements currently allocated.
    pub async fn len(&self) -> Result<usize, Error> {
        self.send_command(Command::Len).await
    }

    /// Returns true if no elements are allocated.
    pub async fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.len().await? == 0)
    }

    /// Gets the value at the given index.
    pub async fn get(&self, index: usize) -> Result<Option<T>, Error> {
        self.send_command(|tx| Command::Get(index, tx)).await
    }

    /// Sets the value at the given index. Returns true if the index was valid.
    pub async fn set(&self, index: usize, value: T) -> Result<bool, Error> {
        self.send_command(|tx| Command::Set(index, value, tx)).await
    }

    /// Reads a range of values as a Vec.
    pub async fn read_slice(&self, offset: usize, count: usize) -> Result<Vec<T>, Error> {
        self.send_command(|tx| Command::ReadSlice(offset, count, tx))
            .await?
    }

    /// Writes values starting at the given offset.
    pub async fn write_slice(&self, offset: usize, values: Vec<T>) -> Result<(), Error> {
        self.send_command(|tx| Command::WriteSlice(offset, values, tx))
            .await?
    }

    /// Grows the memory by `count` elements, filling with the given value.
    pub async fn grow_filled(&self, count: usize, value: T) -> Result<(), Error> {
        self.send_command(|tx| Command::GrowFilled(count, value, tx))
            .await?
    }

    /// Grows the memory by `count` elements, zero-initialized.
    ///
    /// # Safety
    ///
    /// The type `T` must be valid when all bytes are zero.
    pub async unsafe fn grow_zeroed(&self, count: usize) -> Result<(), Error> {
        self.send_command(|tx| Command::GrowZeroed(count, tx))
            .await?
    }

    /// Grows the memory by `count` elements, assuming the file data is
    /// already initialized.
    ///
    /// # Safety
    ///
    /// The underlying file must contain valid initialized data for `count` elements.
    pub async unsafe fn grow_assumed(&self, count: usize) -> Result<(), Error> {
        self.send_command(|tx| Command::GrowAssumed(count, tx))
            .await?
    }

    /// Shrinks the memory by `count` elements.
    pub async fn shrink(&self, count: usize) -> Result<(), Error> {
        self.send_command(|tx| Command::Shrink(count, tx)).await?
    }

    /// Gracefully shuts down the I/O thread, syncing data to disk.
    ///
    /// This is called automatically on drop, but can be called explicitly
    /// to await completion.
    pub async fn shutdown(&mut self) -> Result<(), Error> {
        if let Some(tx) = self.tx.take() {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            let _ = tx.send(Command::Shutdown(reply_tx));
            let _ = reply_rx.await;
        }
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "I/O thread panicked"))?;
        }
        Ok(())
    }
}

impl<T: Copy + Send + 'static> Drop for AsyncFileMem<T> {
    fn drop(&mut self) {
        // Drop the sender to close the channel. The I/O thread's recv loop
        // will return Err and exit, running FileMapped's Drop (which syncs).
        drop(self.tx.take());
        // Wait for the I/O thread to finish (ensures sync completes).
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl<T: Copy + Send + 'static> fmt::Debug for AsyncFileMem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncFileMem")
            .field("active", &self.tx.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_file_mem_temp_and_grow() {
        let mem = AsyncFileMem::<u64>::temp().unwrap();
        assert!(mem.is_empty().await.unwrap());

        mem.grow_filled(10, 42).await.unwrap();
        assert_eq!(mem.len().await.unwrap(), 10);
        assert_eq!(mem.get(0).await.unwrap(), Some(42));
        assert_eq!(mem.get(9).await.unwrap(), Some(42));
        assert_eq!(mem.get(10).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_async_file_mem_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("persist.bin");

        // Write data
        {
            let mut mem = AsyncFileMem::<u64>::from_path(&path).unwrap();
            mem.grow_filled(5, 123).await.unwrap();
            mem.set(2, 456).await.unwrap();
            mem.shutdown().await.unwrap();
        }

        // Read data back — use grow_assumed to map existing file data
        {
            let mem = AsyncFileMem::<u64>::from_path(&path).unwrap();
            // The file has 5 elements from the previous session.
            // grow_assumed tells FileMapped to map the already-initialized data.
            unsafe { mem.grow_assumed(5).await.unwrap() };
            assert_eq!(mem.len().await.unwrap(), 5);
            assert_eq!(mem.get(0).await.unwrap(), Some(123));
            assert_eq!(mem.get(2).await.unwrap(), Some(456));
        }
    }

    #[tokio::test]
    async fn test_async_file_mem_shrink() {
        let mem = AsyncFileMem::<u64>::temp().unwrap();
        mem.grow_filled(20, 1).await.unwrap();
        assert_eq!(mem.len().await.unwrap(), 20);

        mem.shrink(5).await.unwrap();
        assert_eq!(mem.len().await.unwrap(), 15);
    }

    #[tokio::test]
    async fn test_async_file_mem_grow_zeroed() {
        let mem = AsyncFileMem::<u64>::temp().unwrap();
        unsafe {
            mem.grow_zeroed(10).await.unwrap();
        }
        assert_eq!(mem.len().await.unwrap(), 10);
        for i in 0..10 {
            assert_eq!(mem.get(i).await.unwrap(), Some(0));
        }
    }

    #[tokio::test]
    async fn test_async_file_mem_read_write_slice() {
        let mem = AsyncFileMem::<u64>::temp().unwrap();
        mem.grow_filled(10, 0).await.unwrap();

        // Write a slice
        mem.write_slice(3, vec![10, 20, 30]).await.unwrap();

        // Read it back
        let data = mem.read_slice(3, 3).await.unwrap();
        assert_eq!(data, vec![10, 20, 30]);

        // Verify surrounding values unchanged
        assert_eq!(mem.get(2).await.unwrap(), Some(0));
        assert_eq!(mem.get(6).await.unwrap(), Some(0));
    }

    #[tokio::test]
    async fn test_async_file_mem_set_get() {
        let mem = AsyncFileMem::<u64>::temp().unwrap();
        mem.grow_filled(5, 0).await.unwrap();

        assert!(mem.set(0, 100).await.unwrap());
        assert!(mem.set(4, 400).await.unwrap());
        assert!(!mem.set(5, 500).await.unwrap()); // out of bounds

        assert_eq!(mem.get(0).await.unwrap(), Some(100));
        assert_eq!(mem.get(4).await.unwrap(), Some(400));
    }

    #[tokio::test]
    async fn test_async_file_mem_multiple_grows() {
        let mem = AsyncFileMem::<u64>::temp().unwrap();

        mem.grow_filled(5, 1).await.unwrap();
        mem.grow_filled(5, 2).await.unwrap();
        mem.grow_filled(5, 3).await.unwrap();

        assert_eq!(mem.len().await.unwrap(), 15);
        assert_eq!(mem.get(0).await.unwrap(), Some(1));
        assert_eq!(mem.get(5).await.unwrap(), Some(2));
        assert_eq!(mem.get(10).await.unwrap(), Some(3));
    }

    #[tokio::test]
    async fn test_async_file_mem_sequential_operations() {
        let mem = AsyncFileMem::<u64>::temp().unwrap();
        mem.grow_filled(100, 0).await.unwrap();

        // Multiple set operations — all serialized through the I/O thread
        for i in 0..10u64 {
            mem.set(i as usize, i * 10).await.unwrap();
        }

        for i in 0..10u64 {
            assert_eq!(mem.get(i as usize).await.unwrap(), Some(i * 10));
        }
    }
}
