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

pub(crate) use raw_place::RawPlace;
pub use {
    alloc::Alloc,
    file_mapped::FileMapped,
    raw_mem::{Error, RawMem, Result},
};

mod alloc;
mod file_mapped;
mod raw_mem;
mod raw_place;
mod utils;

fn _assertion() {
    fn assert_sync_send<T: Sync + Send>() {}

    assert_sync_send::<FileMapped<()>>();
    assert_sync_send::<Alloc<(), std::alloc::Global>>();
}
