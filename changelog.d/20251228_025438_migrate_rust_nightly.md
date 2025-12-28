---
bump: minor
---

### Changed
- **Migrated from Rust nightly to stable Rust** - The crate now works on stable Rust!
- Added `allocator-api2` dependency for allocator API functionality on stable Rust
- Replaced unstable `fn_traits`/`unboxed_closures` features with a custom `FillFn` trait
- Replaced unstable `slice_ptr_get`/`ptr_as_uninit` features with manual pointer arithmetic
- Replaced unstable `slice_range` with a manual bounds-checking implementation
- Implemented stable alternatives for `MaybeUninit` slice methods in `uninit` module:
  - Added `uninit::assume_init_mut()` as stable alternative to `MaybeUninit::slice_assume_init_mut()`
  - Added `uninit::write_clone_of_slice()` as stable alternative to `MaybeUninit::write_slice_cloned()`
- Updated CI workflow to use `dtolnay/rust-toolchain@stable`

### Updated
- Updated dependencies to latest versions:
  - `memmap2`: 0.7 -> 0.9
  - `tempfile`: 3.3 -> 3
  - `thiserror`: 1.0 -> 2
- Updated documentation to reflect stable Rust compatibility
