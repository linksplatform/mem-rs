---
bump: patch
---
### Fixed
- Fixed GitHub Release creation failure caused by unsupported look-ahead regex in `create-github-release.rs` (Rust's `regex` crate does not support look-around assertions)
- Added crates.io and docs.rs badges to GitHub Release notes, matching the template repository best practices
