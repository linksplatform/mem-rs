---
bump: minor
---

### Changed
- Replaced Node.js (.mjs) CI/CD scripts with Rust (.rs) scripts using rust-script, matching the template at link-foundation/rust-ai-driven-development-pipeline-template
- Updated GitHub Actions workflow with lint checks (clippy, rustfmt), change detection, version modification guard, crates.io publishing, and changelog PR support
- Updated action versions to latest (actions/checkout@v6, actions/cache@v5, peter-evans/create-pull-request@v8)
- Added `RUSTFLAGS: -Dwarnings` to treat warnings as errors in CI

### Fixed
- Fixed version from `0.1.0-pre+beta.2` (pre-release/beta) to `0.1.0` (proper SemVer) to enable crates.io publishing
- Added missing Cargo.toml metadata required for crates.io: `readme`, `keywords`, `categories`, `rust-version`, `[lib]`, `[lints]`, `[profile.release]`
