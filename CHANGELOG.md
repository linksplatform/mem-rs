# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- changelog-insert-here -->

## [0.2.0] - 2026-04-13

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

### Changed
- Upgraded to Rust edition 2024 with proper unsafe block handling
- Updated allocator-api2 from 0.2 to 0.4
- Updated criterion from 0.5 to 0.8
- Updated rustfmt.toml to stable-only options

### Fixed
- Fixed 8 clippy warnings (missing Safety docs, suspicious_open_options, is_multiple_of, io_other_error)
- Fixed Error::OverAlloc bug in prealloc.rs (variant didn't exist, corrected to OverGrow)
- Fixed PreAlloc::grow() signature to match RawMem trait
- Fixed broken intra-doc links

### Added
- Comprehensive doc comments for all public types and methods
- Crate-level documentation with examples and feature table
- Automated documentation deployment to GitHub Pages via CI/CD
- Moved async_mem tests from src/ to tests/ directory
- Case study document for issue #28

### Fixed
- Fixed CI/CD Auto Release failure caused by version regex not handling SemVer pre-release versions (e.g., `0.1.0-pre+beta.2`)
- Verified all repository files use Unlicense (public domain) — no LGPL references found

### Changed
- Replaced Node.js (.mjs) CI/CD scripts with Rust (.rs) scripts using rust-script, matching the template at link-foundation/rust-ai-driven-development-pipeline-template
- Updated GitHub Actions workflow with lint checks (clippy, rustfmt), change detection, version modification guard, crates.io publishing, and changelog PR support
- Updated action versions to latest (actions/checkout@v6, actions/cache@v5, peter-evans/create-pull-request@v8)
- Added `RUSTFLAGS: -Dwarnings` to treat warnings as errors in CI

### Fixed
- Fixed version from `0.1.0-pre+beta.2` (pre-release/beta) to `0.1.0` (proper SemVer) to enable crates.io publishing
- Added missing Cargo.toml metadata required for crates.io: `readme`, `keywords`, `categories`, `rust-version`, `[lib]`, `[lints]`, `[profile.release]`
