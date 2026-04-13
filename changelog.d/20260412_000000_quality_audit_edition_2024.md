---
bump: minor
---

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
