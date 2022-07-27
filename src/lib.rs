#![feature(const_nonnull_slice_from_raw_parts)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(allocator_api)]
#![feature(default_free_fn)]
#![feature(layout_for_ptr)]
#![feature(slice_ptr_get)]
#![feature(try_blocks)]
#![feature(slice_ptr_len)]
#![feature(io_error_other)]
#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

pub use alloc::Alloc;
pub use file_mapped::FileMapped;
pub use global::Global;
pub use prealloc::PreAlloc;
pub use temp_file::TempFile;
pub use traits::{Error, RawMem, Result, DEFAULT_PAGE_SIZE};

mod alloc;
mod base;
mod file_mapped;
mod global;
mod internal;
mod prealloc;
mod temp_file;
mod traits;

pub(crate) use base::Base;
pub(crate) use internal::IsTrue;
