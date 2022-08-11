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

#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(
    clippy::perf,
    clippy::single_match_else,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::semicolon_if_nothing_returned,
    clippy::pedantic,
    clippy::nursery
)]
// for `clippy::pedantic`
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::let_underscore_drop,
    clippy::non_send_fields_in_send_ty,
)]
#![deny(
    clippy::all,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::use_self,
    clippy::unnested_or_patterns,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_unwrap_or,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    // unreachable_pub,
    trivial_numeric_casts,
    // rustdoc,
    // missing_debug_implementations,
    // missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
)]
// must be fixed later
#![allow(clippy::needless_pass_by_value, clippy::comparison_chain)]

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
