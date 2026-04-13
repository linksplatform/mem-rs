# Case Study: Issue #32 - Rust Package Not Published by CI/CD

## Timeline of Events

1. **Initial state**: The `platform-mem` crate had version `0.1.0-pre+beta.2` in `Cargo.toml`, a pre-release version with build metadata.
2. **CI/CD pipeline**: The workflow (`release.yml`) used Node.js scripts (`.mjs`) for automation, which differed from the organization's standardized Rust-based CI/CD template.
3. **Publishing gap**: The auto-release job created GitHub Releases but **never published to crates.io** — the `cargo publish` step was entirely missing.
4. **Version format issue**: The pre-release version `0.1.0-pre+beta.2` would prevent stable publishing to crates.io even if the publish step existed, as crates.io treats pre-release versions differently.
5. **Issue #28** (referenced): Established requirements for CI/CD best practices, including using examples from the template repository.
6. **PR #31**: Fixed version regex parsing for SemVer pre-release versions but did not address the root causes.

## Requirements from the Issue

| # | Requirement | Status |
|---|-------------|--------|
| 1 | Support automatic Rust package publishing to crates.io | Fixed: Added `publish-crate.rs` script and `Publish to Crates.io` step in both auto-release and manual-release jobs |
| 2 | Support generation of GitHub Releases | Already working, retained |
| 3 | Drop incrementing beta versions, use minimum patch incrementing | Fixed: Version changed from `0.1.0-pre+beta.2` to `0.1.0` |
| 4 | Follow best practices from `trees-rs`, `Numbers`, and the template repo | Fixed: Adopted the full CI/CD pipeline from the template |
| 5 | Compile data to `docs/case-studies/issue-32/` folder | This document |
| 6 | Report issues to other repos if relevant | See "Cross-Repository Issues" section below |

## Root Cause Analysis

### Root Cause 1: Missing crates.io Publishing Step
The `release.yml` workflow created GitHub Releases but had no `cargo publish` step. The template repository (`rust-ai-driven-development-pipeline-template`) includes a dedicated `publish-crate.rs` script that handles:
- Token authentication (via `CARGO_REGISTRY_TOKEN` or `CARGO_TOKEN`)
- Already-published version detection
- Detailed error reporting for auth failures

### Root Cause 2: Pre-release Version Blocking Publishing
The version `0.1.0-pre+beta.2` uses SemVer pre-release identifiers (`-pre`) and build metadata (`+beta.2`). While crates.io accepts pre-release versions, they:
- Are not discoverable via `cargo search`
- Don't satisfy `^0.1` dependency requirements
- Signal instability to users

The issue explicitly requested dropping beta versioning in favor of standard patch incrementing.

### Root Cause 3: Divergence from Template CI/CD
The mem-rs repository used Node.js (`.mjs`) scripts while the template uses Rust scripts (`.rs`) via `rust-script`. This divergence meant:
- Missing features present in the template (lint checks, change detection, version modification guards, changelog PR support)
- Inconsistent tooling across the organization's Rust projects
- Harder maintenance since updates to the template weren't automatically applicable

### Root Cause 4: Missing Cargo.toml Metadata
The `Cargo.toml` was missing fields required or recommended for crates.io publishing:
- `readme` - displays README on crates.io
- `keywords` - enables search discovery
- `categories` - enables category browsing
- `rust-version` - communicates MSRV to users
- `[lib]` section - explicit library target
- `[lints]` section - consistent code quality checks
- `[profile.release]` - optimized release builds

## Solution Summary

### Changes Made

1. **Cargo.toml**:
   - Version: `0.1.0-pre+beta.2` -> `0.1.0`
   - Added: `readme`, `keywords`, `categories`, `rust-version`, `[lib]`, `[lints.rust]`, `[lints.clippy]`, `[profile.release]`

2. **Scripts** (replaced 6 Node.js scripts with 15 Rust scripts):
   - Removed: `*.mjs` (bump-version, check-file-size, collect-changelog, create-github-release, get-bump-type, version-and-commit)
   - Added: `*.rs` (all 15 scripts from the template, including publish-crate, detect-code-changes, check-changelog-fragment, check-version-modification, create-changelog-fragment, git-config, get-version, check-release-needed, rust-paths)

3. **Workflow** (`release.yml`):
   - Added: `detect-changes`, `lint`, `version-check`, `changelog-pr` jobs
   - Added: `Publish to Crates.io` step in auto-release and manual-release
   - Added: `RUSTFLAGS: -Dwarnings` and `CARGO_TOKEN` env vars
   - Updated: Action versions (checkout@v6, cache@v5, create-pull-request@v8)
   - Replaced: Node.js setup + `node scripts/*.mjs` with `cargo install rust-script` + `rust-script scripts/*.rs`

4. **Code fixes** (clippy compliance):
   - `benches/memory_benchmarks.rs`: Replaced deprecated `criterion::black_box` with `std::hint::black_box`
   - `tests/tests.rs`: Fixed ignored unit pattern lint
   - `experiments/async_example.rs`: Fixed unnecessary Debug formatting

## Cross-Repository Issues

The template repository (`link-foundation/rust-ai-driven-development-pipeline-template`) was found to be up-to-date with all necessary CI/CD features. No issues need to be reported there.

## References

- Issue: https://github.com/linksplatform/mem-rs/issues/32
- Template repo: https://github.com/link-foundation/rust-ai-driven-development-pipeline-template
- trees-rs CI/CD: https://github.com/linksplatform/trees-rs/blob/main/.github/workflows/ci.yml
- Numbers CI/CD: https://github.com/linksplatform/Numbers/blob/main/.github/workflows/rust.yml
- crates.io publishing docs: https://doc.rust-lang.org/cargo/reference/publishing.html
