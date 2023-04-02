#![feature(allocator_api)]
#![feature(unchecked_math)]
#![feature(maybe_uninit_slice)]
#![feature(slice_ptr_get)]
#![feature(ptr_as_uninit)]
//
// special lint
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
// rust compiler lints
#![deny(unused_must_use)]
#![warn(missing_debug_implementations)]

// Bare metal platforms usually have very small amounts of RAM
// (in the order of hundreds of KB)
/// RAM page size which is likely to be the same on most systems
#[rustfmt::skip]
pub const DEFAULT_PAGE_SIZE: usize = if cfg!(target_os = "espidf") { 512 } else { 8 * 1024 };

pub use {
    alloc::Alloc,
    raw_mem::{Error, RawMem, Result},
};
pub(crate) use {raw_place::RawPlace, utils::debug_mem};

mod alloc;
mod raw_mem;
mod raw_place;
mod utils;
