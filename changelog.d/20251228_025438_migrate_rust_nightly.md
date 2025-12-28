---
bump: minor
---

### Changed
- Migrated from Rust nightly-2022-08-22 to latest Rust nightly toolchain
- Updated `MaybeUninit` slice API to use new stabilized method names:
  - `MaybeUninit::slice_assume_init_mut(slice)` -> `slice.assume_init_mut()`
  - `MaybeUninit::write_slice_cloned(slice, src)` -> `slice.write_clone_of_slice(src)`
- Removed stabilized features from feature list:
  - `let_else` (stable since 1.65.0)
  - `inline_const` (stable since 1.79.0)
  - `nonnull_slice_from_raw_parts` (stable since 1.70.0)
  - `unchecked_math` (stable since 1.79.0)
  - `maybe_uninit_slice` (stable since 1.93.0)
  - `maybe_uninit_write_slice` (stable since 1.93.0)

### Updated
- Updated dependencies to latest versions:
  - `memmap2`: 0.7 -> 0.9
  - `tempfile`: 3.3 -> 3
  - `thiserror`: 1.0 -> 2
- Updated CI workflow to use `dtolnay/rust-toolchain@nightly` for latest nightly
