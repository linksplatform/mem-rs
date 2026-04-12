//! Low-level memory management with pluggable backends.
//!
//! `platform-mem` provides a unified [`RawMem`] trait that abstracts over
//! different memory storage strategies: heap allocators, memory-mapped files,
//! and temporary files. Code written against [`RawMem`] works with any backend.
//!
//! # Backends
//!
//! | Type | Storage | Use case |
//! |------|---------|----------|
//! | [`Global<T>`] | Rust global allocator | General-purpose in-memory storage |
//! | [`System<T>`] | System allocator | When you need the OS allocator specifically |
//! | [`Alloc<T, A>`] | Any [`Allocator`](allocator_api2::alloc::Allocator) | Custom allocator strategies |
//! | [`FileMapped<T>`] | Memory-mapped file | Persistent storage, large datasets |
//! | [`TempFile<T>`] | Temporary mmap file | Anonymous storage cleaned on drop |
//! | [`AsyncFileMem<T>`](async_mem::AsyncFileMem) | Async mmap via I/O thread | Non-blocking file-backed storage (requires `async` feature) |
//!
//! # Quick start
//!
//! ```
//! use platform_mem::{Global, RawMem};
//!
//! let mut mem = Global::<u64>::new();
//! mem.grow_filled(3, 42).unwrap();
//! assert_eq!(mem.allocated(), &[42, 42, 42]);
//!
//! mem.grow_from_slice(&[1, 2, 3]).unwrap();
//! assert_eq!(mem.allocated(), &[42, 42, 42, 1, 2, 3]);
//!
//! mem.shrink(2).unwrap();
//! assert_eq!(mem.allocated(), &[42, 42, 42, 1]);
//! ```
//!
//! # Type erasure
//!
//! Use [`ErasedMem`] to work with heterogeneous memory backends via dynamic dispatch:
//!
//! ```
//! use platform_mem::{Global, ErasedMem, RawMem};
//!
//! fn use_any_mem(mem: &mut Box<dyn ErasedMem<Item = u64>>) {
//!     mem.grow_filled(5, 0).unwrap();
//! }
//!
//! let mut mem: Box<dyn ErasedMem<Item = u64>> = Box::new(Global::<u64>::new());
//! use_any_mem(&mut mem);
//! assert_eq!(mem.allocated().len(), 5);
//! ```
//!
//! # Features
//!
//! - **`async`** — enables [`AsyncFileMem`] for non-blocking
//!   file-backed memory via a dedicated I/O thread (requires tokio).

// special lint
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
// rust compiler lints
#![deny(unused_must_use)]
#![warn(missing_docs, missing_debug_implementations)]

mod alloc;
mod file_mapped;
/// Core trait definitions, error types, and `MaybeUninit` helpers.
pub mod raw_mem;
mod raw_place;
mod utils;

/// Async memory module providing [`AsyncFileMem`].
///
/// Requires the `async` feature flag.
#[cfg(feature = "async")]
pub mod async_mem;

pub(crate) use raw_place::RawPlace;
pub use {
    alloc::Alloc,
    file_mapped::FileMapped,
    raw_mem::{ErasedMem, Error, RawMem, Result},
};

#[cfg(feature = "async")]
pub use async_mem::AsyncFileMem;

fn _assertion() {
    fn assert_sync_send<T: Sync + Send>() {}

    assert_sync_send::<FileMapped<()>>();
    assert_sync_send::<Alloc<(), allocator_api2::alloc::Global>>();
}

macro_rules! delegate_memory {
    ($($(#[$attr:meta])* $me:ident<$param:ident>($inner:ty) { $($body:tt)* } )*) => {$(
        $(#[$attr])*
        pub struct $me<$param>($inner);

        impl<$param> $me<$param> {
            $($body)*
        }

        const _: () = {
            use std::{
                mem::MaybeUninit,
                fmt::{self, Formatter},
            };

            impl<$param> RawMem for $me<$param> {
                type Item = $param;

                fn allocated(&self) -> &[Self::Item] {
                    self.0.allocated()
                }

                fn allocated_mut(&mut self) -> &mut [Self::Item] {
                    self.0.allocated_mut()
                }

                unsafe fn grow(
                    &mut self,
                    addition: usize,
                    fill: impl FnOnce(usize, (&mut [Self::Item], &mut [MaybeUninit<Self::Item>])),
                ) -> Result<&mut [Self::Item]> { unsafe {
                    self.0.grow(addition, fill)
                }}

                fn shrink(&mut self, cap: usize) -> Result<()> {
                    self.0.shrink(cap)
                }

                fn size_hint(&self) -> Option<usize> {
                    self.0.size_hint()
                }
            }

            impl<T> fmt::Debug for $me<$param> {
                fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                    f.debug_tuple(stringify!($me)).field(&self.0).finish()
                }
            }

        };
    )*};
}

use allocator_api2::alloc::Global as GlobalAlloc;
use std::{alloc::System as SystemAlloc, fs::File, io, path::Path};

delegate_memory! {
    /// Memory backed by the Rust global allocator.
    ///
    /// This is the most common backend for in-memory storage.
    ///
    /// ```
    /// use platform_mem::{Global, RawMem};
    ///
    /// let mut mem = Global::<u64>::new();
    /// mem.grow_filled(10, 0).unwrap();
    /// assert_eq!(mem.allocated().len(), 10);
    /// ```
    #[doc(alias = "GlobalAlloc")]
    Global<T>(Alloc<T, GlobalAlloc>) {
        /// Creates a new empty `Global` memory.
        pub const fn new() -> Self {
            Self(Alloc::new(GlobalAlloc))
        }
    }

    /// Memory backed by the system allocator.
    #[doc(alias = "SystemAlloc")]
    System<T>(Alloc<T, SystemAlloc>) {
        /// Creates a new empty `System` memory.
        pub const fn new() -> Self {
            Self(Alloc::new(SystemAlloc))
        }
    }

    /// Temporary file-backed memory-mapped storage.
    ///
    /// Data is stored in an anonymous temporary file that is automatically
    /// cleaned up when the `TempFile` is dropped.
    TempFile<T>(FileMapped<T>) {
        /// Creates a new temporary file-backed memory.
        pub fn new() -> io::Result<Self> {
            Self::from_temp(tempfile::tempfile())
        }

        /// Creates a new temporary file-backed memory in the specified directory.
        pub fn new_in<P: AsRef<Path>>(path: P) -> io::Result<Self> {
            Self::from_temp(tempfile::tempfile_in(path))
        }

        fn from_temp(file: io::Result<File>) -> io::Result<Self> {
            file.and_then(FileMapped::new).map(Self)
        }
    }
}

impl<T> Default for Global<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Default for System<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn _is_raw_mem() {
    fn check<T: RawMem>() {}

    check::<Box<dyn ErasedMem<Item = ()>>>();
    check::<Box<dyn ErasedMem<Item = ()> + Sync>>();
    check::<Box<dyn ErasedMem<Item = ()> + Sync + Send>>();

    fn elie() -> Box<Global<()>> {
        todo!()
    }

    let _: Box<dyn ErasedMem<Item = ()>> = elie();
    let _: Box<dyn ErasedMem<Item = ()> + Sync> = elie();
    let _: Box<dyn ErasedMem<Item = ()> + Sync + Send> = elie();
}
