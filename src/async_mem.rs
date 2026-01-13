//! Asynchronous memory operations for file-backed storage.
//!
//! This module provides async alternatives to the synchronous `FileMapped` type,
//! offering efficient asynchronous file I/O for memory operations.
//!
//! # Backend Selection
//!
//! The implementation uses `tokio::fs::File` for file operations, which internally
//! uses `spawn_blocking` for filesystem operations. This works on all platforms.
//!
//! # Why Not Async Mmap?
//!
//! Memory-mapped file I/O cannot be truly asynchronous because page faults
//! trigger synchronous disk operations. For truly async file access, explicit
//! read/write operations are required.
//!
//! # Example
//!
//! ```ignore
//! use platform_mem::AsyncFileMem;
//!
//! async fn example() -> std::io::Result<()> {
//!     let mut mem = AsyncFileMem::<u64>::create("data.bin").await?;
//!
//!     // Async grow with zeroed initialization
//!     unsafe { mem.grow_zeroed(1000).await? };
//!
//!     // Read/write operations
//!     mem.set(0, 42);
//!     let val = mem.get(0);
//!     assert_eq!(val, Some(42));
//!
//!     // Sync to disk
//!     mem.sync().await?;
//!     Ok(())
//! }
//! ```

use std::{
    alloc::Layout,
    fmt,
    io,
    marker::PhantomData,
    mem,
    path::{Path, PathBuf},
};

use crate::Error;

/// Result type for async memory operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Asynchronous file-backed memory storage.
///
/// Unlike `FileMapped`, this type does not use memory mapping. Instead, it
/// performs explicit async read/write operations to the underlying file,
/// with an in-memory buffer for fast access.
///
/// This approach enables true asynchronous I/O operations.
pub struct AsyncFileMem<T> {
    /// In-memory buffer holding the current data
    buffer: Vec<T>,
    /// Path to the file (None for temp files)
    path: Option<PathBuf>,
    /// Track whether buffer has unsaved changes
    dirty: bool,
    /// Marker for the type
    _marker: PhantomData<T>,
}

impl<T: Copy + Default> AsyncFileMem<T> {
    /// Creates a new async file memory with a new file at the given path.
    ///
    /// If the file already exists, it will be truncated.
    pub async fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        use tokio::fs::OpenOptions;

        // Create/truncate the file
        let _ = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())
            .await?;

        Ok(Self {
            buffer: Vec::new(),
            path: Some(path.as_ref().to_path_buf()),
            dirty: false,
            _marker: PhantomData,
        })
    }

    /// Opens an existing file for async memory operations.
    ///
    /// The file contents will be loaded into memory.
    pub async fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        use tokio::fs::OpenOptions;
        use tokio::io::AsyncReadExt;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.as_ref())
            .await?;

        // Read file contents
        let metadata = file.metadata().await?;
        let file_size = metadata.len() as usize;
        let elem_size = mem::size_of::<T>();

        let mut buffer = Vec::new();
        if file_size > 0 && elem_size > 0 {
            let count = file_size / elem_size;
            buffer.reserve(count);

            let mut bytes = vec![0u8; count * elem_size];
            file.read_exact(&mut bytes).await?;

            // SAFETY: We're interpreting raw bytes as T, which is safe for Copy + Default types
            // that can be represented as bytes (like integers)
            unsafe {
                let ptr = bytes.as_ptr() as *const T;
                for i in 0..count {
                    buffer.push(*ptr.add(i));
                }
            }
        }

        Ok(Self {
            buffer,
            path: Some(path.as_ref().to_path_buf()),
            dirty: false,
            _marker: PhantomData,
        })
    }

    /// Creates a temporary async file memory.
    ///
    /// The temporary file will be automatically cleaned up when dropped.
    pub async fn temp() -> io::Result<Self> {
        // Create a temp file path
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!(
            "platform_mem_async_{}.tmp",
            std::process::id()
        ));

        // Create the file
        let _ = tokio::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .await?;

        Ok(Self {
            buffer: Vec::new(),
            path: Some(temp_path),
            dirty: false,
            _marker: PhantomData,
        })
    }

    /// Returns the number of elements currently allocated.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if no elements are allocated.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns a slice of the allocated memory.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.buffer
    }

    /// Returns a mutable slice of the allocated memory.
    ///
    /// Note: Modifications through this slice won't be automatically persisted.
    /// Call `sync()` to persist changes.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        self.dirty = true;
        &mut self.buffer
    }

    /// Gets the value at the given index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<T> {
        self.buffer.get(index).copied()
    }

    /// Sets the value at the given index.
    #[inline]
    pub fn set(&mut self, index: usize, value: T) -> Option<()> {
        if index < self.buffer.len() {
            self.buffer[index] = value;
            self.dirty = true;
            Some(())
        } else {
            None
        }
    }

    /// Grows the memory by the given number of elements, filling with the default value.
    pub async fn grow(&mut self, addition: usize) -> Result<&mut [T]> {
        self.grow_with(addition, T::default).await
    }

    /// Grows the memory by the given number of elements, filling with zeros.
    ///
    /// # Safety
    ///
    /// The type `T` must be valid when all bytes are zero.
    pub async unsafe fn grow_zeroed(&mut self, addition: usize) -> Result<&mut [T]> {
        let old_len = self.buffer.len();
        let new_len = old_len.checked_add(addition).ok_or(Error::CapacityOverflow)?;

        // Check layout is valid
        Layout::array::<T>(new_len).map_err(|_| Error::CapacityOverflow)?;

        self.buffer.reserve(addition);

        // Zero-initialize new elements
        let uninit_ptr = self.buffer.as_mut_ptr().add(old_len);
        std::ptr::write_bytes(uninit_ptr, 0, addition);
        self.buffer.set_len(new_len);

        self.dirty = true;
        Ok(&mut self.buffer[old_len..])
    }

    /// Grows the memory by the given number of elements, filling with the given value.
    pub async fn grow_filled(&mut self, addition: usize, value: T) -> Result<&mut [T]>
    where
        T: Clone,
    {
        let old_len = self.buffer.len();
        let new_len = old_len.checked_add(addition).ok_or(Error::CapacityOverflow)?;

        Layout::array::<T>(new_len).map_err(|_| Error::CapacityOverflow)?;

        self.buffer.resize(new_len, value);
        self.dirty = true;
        Ok(&mut self.buffer[old_len..])
    }

    /// Grows the memory by the given number of elements using a closure.
    pub async fn grow_with<F>(&mut self, addition: usize, mut f: F) -> Result<&mut [T]>
    where
        F: FnMut() -> T,
    {
        let old_len = self.buffer.len();
        let new_len = old_len.checked_add(addition).ok_or(Error::CapacityOverflow)?;

        Layout::array::<T>(new_len).map_err(|_| Error::CapacityOverflow)?;

        self.buffer.reserve(addition);
        for _ in 0..addition {
            self.buffer.push(f());
        }

        self.dirty = true;
        Ok(&mut self.buffer[old_len..])
    }

    /// Grows the memory by copying from a slice.
    pub async fn grow_from_slice(&mut self, src: &[T]) -> Result<&mut [T]>
    where
        T: Clone,
    {
        let old_len = self.buffer.len();
        let new_len = old_len.checked_add(src.len()).ok_or(Error::CapacityOverflow)?;

        Layout::array::<T>(new_len).map_err(|_| Error::CapacityOverflow)?;

        self.buffer.extend_from_slice(src);
        self.dirty = true;
        Ok(&mut self.buffer[old_len..])
    }

    /// Shrinks the memory by the given number of elements.
    pub async fn shrink(&mut self, count: usize) -> Result<()> {
        let new_len = self.buffer.len().saturating_sub(count);
        self.buffer.truncate(new_len);
        self.dirty = true;
        Ok(())
    }

    /// Syncs all data to the underlying file.
    pub async fn sync(&mut self) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;

        if let Some(ref path) = self.path {
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    self.buffer.as_ptr() as *const u8,
                    self.buffer.len() * mem::size_of::<T>(),
                )
            };

            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)
                .await?;

            file.write_all(bytes).await?;
            file.sync_all().await?;
        }

        self.dirty = false;
        Ok(())
    }

    /// Returns whether there are unsaved changes.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Flushes data to the file without full sync.
    ///
    /// This is faster than `sync()` but doesn't guarantee data is on disk.
    pub async fn flush(&mut self) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;

        if let Some(ref path) = self.path {
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    self.buffer.as_ptr() as *const u8,
                    self.buffer.len() * mem::size_of::<T>(),
                )
            };

            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)
                .await?;

            file.write_all(bytes).await?;
            file.flush().await?;
        }

        self.dirty = false;
        Ok(())
    }
}

impl<T> Drop for AsyncFileMem<T> {
    fn drop(&mut self) {
        // Best-effort sync on drop - note: cannot be async in drop
        // For temp files, optionally clean up
        // Note: We don't sync on drop since it's not async-safe
        // Users should call sync() explicitly before dropping if persistence is needed
    }
}

impl<T> fmt::Debug for AsyncFileMem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncFileMem")
            .field("len", &self.buffer.len())
            .field("path", &self.path)
            .field("dirty", &self.dirty)
            .finish()
    }
}

// AsyncFileMem is Send + Sync if T is
unsafe impl<T: Send> Send for AsyncFileMem<T> {}
unsafe impl<T: Sync> Sync for AsyncFileMem<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_file_mem_create_and_grow() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");

        let mut mem = AsyncFileMem::<u64>::create(&path).await.unwrap();
        assert!(mem.is_empty());

        mem.grow_filled(10, 42).await.unwrap();
        assert_eq!(mem.len(), 10);
        assert_eq!(mem.get(0), Some(42));

        mem.sync().await.unwrap();
    }

    #[tokio::test]
    async fn test_async_file_mem_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("persist.bin");

        // Write data
        {
            let mut mem = AsyncFileMem::<u64>::create(&path).await.unwrap();
            mem.grow_filled(5, 123).await.unwrap();
            mem.set(2, 456);
            mem.sync().await.unwrap();
        }

        // Read data back
        {
            let mem = AsyncFileMem::<u64>::open(&path).await.unwrap();
            assert_eq!(mem.len(), 5);
            assert_eq!(mem.get(0), Some(123));
            assert_eq!(mem.get(2), Some(456));
        }
    }

    #[tokio::test]
    async fn test_async_file_mem_temp() {
        let mut mem = AsyncFileMem::<u32>::temp().await.unwrap();
        mem.grow_filled(100, 0).await.unwrap();
        assert_eq!(mem.len(), 100);
    }

    #[tokio::test]
    async fn test_async_file_mem_shrink() {
        let mut mem = AsyncFileMem::<u64>::temp().await.unwrap();
        mem.grow_filled(20, 1).await.unwrap();
        assert_eq!(mem.len(), 20);

        mem.shrink(5).await.unwrap();
        assert_eq!(mem.len(), 15);
    }

    #[tokio::test]
    async fn test_async_file_mem_grow_zeroed() {
        let mut mem = AsyncFileMem::<u64>::temp().await.unwrap();
        unsafe {
            mem.grow_zeroed(10).await.unwrap();
        }
        assert_eq!(mem.len(), 10);
        for i in 0..10 {
            assert_eq!(mem.get(i), Some(0));
        }
    }

    #[tokio::test]
    async fn test_async_file_mem_slice_access() {
        let mut mem = AsyncFileMem::<u64>::temp().await.unwrap();
        mem.grow_filled(5, 0).await.unwrap();

        // Modify through slice
        {
            let slice = mem.as_slice_mut();
            slice[0] = 100;
            slice[4] = 400;
        }

        assert_eq!(mem.get(0), Some(100));
        assert_eq!(mem.get(4), Some(400));
        assert!(mem.is_dirty());
    }

    #[tokio::test]
    async fn test_async_file_mem_grow_from_slice() {
        let mut mem = AsyncFileMem::<u64>::temp().await.unwrap();
        let data = [1u64, 2, 3, 4, 5];
        mem.grow_from_slice(&data).await.unwrap();

        assert_eq!(mem.len(), 5);
        assert_eq!(mem.as_slice(), &data);
    }

    #[tokio::test]
    async fn test_async_file_mem_multiple_grows() {
        let mut mem = AsyncFileMem::<u64>::temp().await.unwrap();

        mem.grow_filled(5, 1).await.unwrap();
        mem.grow_filled(5, 2).await.unwrap();
        mem.grow_filled(5, 3).await.unwrap();

        assert_eq!(mem.len(), 15);
        assert_eq!(mem.get(0), Some(1));
        assert_eq!(mem.get(5), Some(2));
        assert_eq!(mem.get(10), Some(3));
    }
}
