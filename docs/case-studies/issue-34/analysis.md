# Case Study: Issue #34 - No GitHub Release Was Published with Badge in Rust CI/CD

## Timeline of Events

1. **2026-04-13 07:18**: PR #33 merged to main (from branch `issue-32-bb6017cd91ca`), which replaced Node.js CI/CD scripts with Rust scripts and added crates.io publishing support.
2. **2026-04-13 07:22**: CI/CD Pipeline triggered on push to main (SHA `ee9107a`).
3. **2026-04-13 07:27**: Auto Release job ran successfully through version bumping, changelog collection, and crates.io publishing (`platform-mem@0.2.0` published successfully).
4. **2026-04-13 07:27:44**: `create-github-release.rs` script panicked at line 48 with regex parse error: `look-around, including look-ahead and look-behind, is not supported`.
5. **Result**: Crate v0.2.0 was published to crates.io, but no GitHub Release was created. The existing release `v0.1.0-pre+beta.2` remained as the only GitHub Release.

## Requirements from the Issue

| # | Requirement | Status |
|---|-------------|--------|
| 1 | Support automatic GitHub Release publishing | Fixed: Replaced look-ahead regex with compatible pattern |
| 2 | Include badges in GitHub Releases | Fixed: Added crates.io and docs.rs badge generation |
| 3 | Follow best practices from `trees-rs`, `Numbers`, and the template repo | Fixed: Script now matches template's badge feature |
| 4 | Compile data to `docs/case-studies/issue-34/` folder | This document |
| 5 | Report issues to other repos if relevant | Reported to `link-foundation/rust-ai-driven-development-pipeline-template` |

## Root Cause Analysis

### Root Cause: Unsupported Look-Ahead Regex in Rust's `regex` Crate

**The failing code** (line 47-48 of `create-github-release.rs`):
```rust
let pattern = format!(r"(?s)## \[{}\].*?\n(.*?)(?=\n## \[|$)", escaped_version);
let re = Regex::new(&pattern).unwrap();
```

The pattern `(?=\n## \[|$)` uses a **positive look-ahead assertion**, which is not supported by Rust's `regex` crate (it uses a finite automaton engine that guarantees linear-time matching, which is incompatible with look-around assertions).

This bug was inherited from the template repository (`link-foundation/rust-ai-driven-development-pipeline-template`), where the same pattern exists in `scripts/create-github-release.rs`. The bug went undetected until a version with multiple changelog sections was processed (v0.2.0 had content before the `## [0.1.0]` section header, triggering the regex to attempt matching).

### Contributing Factor: Missing Badges Feature

The `create-github-release.rs` script in mem-rs was missing the automatic badge generation present in the template repository. The template includes crates.io and docs.rs badges in release notes, which were stripped during a previous simplification.

## Solution

### Fix 1: Replace Look-Ahead Regex with Compatible Pattern

Instead of using `(?=\n## \[|$)` to find the boundary between changelog sections, the fix:
1. Finds the version header using `(?m)^## \[{version}\]`
2. Skips to the first newline after the header
3. Uses a separate regex `(?m)^## \[` to find the start of the next section
4. Takes the content between the two positions

This approach achieves the same result without look-ahead, using only features supported by Rust's `regex` crate.

### Fix 2: Restore Badge Generation

Added back the badge generation from the template:
- `get_rust_root()` — detects Cargo.toml location
- `get_cargo_toml_path()` — constructs path to Cargo.toml
- `get_crate_name_from_toml()` — extracts crate name
- Badge format: `[![Crates.io](badge-url)](crate-url) [![Docs.rs](badge-url)](docs-url)`

## Cross-Repository Issues

The same regex bug exists in the template repository at `link-foundation/rust-ai-driven-development-pipeline-template`. An issue was filed there to fix the look-ahead regex in `create-github-release.rs`.

## CI Log Evidence

```
Auto Release  Create GitHub Release  2026-04-13T07:27:44.0504016Z
Auto Release  Create GitHub Release  2026-04-13T07:27:44.0505018Z thread 'main' (5683) panicked at scripts/create-github-release.rs:48:35:
Auto Release  Create GitHub Release  2026-04-13T07:27:44.0511843Z called `Result::unwrap()` on an `Err` value: Syntax(
Auto Release  Create GitHub Release  2026-04-13T07:27:44.0512240Z ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Auto Release  Create GitHub Release  2026-04-13T07:27:44.0512565Z regex parse error:
Auto Release  Create GitHub Release  2026-04-13T07:27:44.0512795Z     (?s)## \[0\.2\.0\].*?\n(.*?)(?=\n## \[|$)
Auto Release  Create GitHub Release  2026-04-13T07:27:44.0513040Z                                 ^^^
Auto Release  Create GitHub Release  2026-04-13T07:27:44.0513387Z error: look-around, including look-ahead and look-behind, is not supported
```

## Verification

The fix was verified with an experiment script (`experiments/test_changelog_regex.rs`) that tests:
1. Extracting a version section with content after it (multi-section changelog)
2. Extracting the last version section (no following section)
3. Handling non-existent versions (fallback message)
4. Single-section changelog edge case
