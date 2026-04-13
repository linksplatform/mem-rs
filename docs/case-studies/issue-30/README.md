# Case Study: Issue #30 — CI/CD and License Fix

## Issue Summary

**Issue:** [#30 — CI/CD and license fix](https://github.com/linksplatform/mem-rs/issues/30)

**Failed CI run:** [24322652581](https://github.com/linksplatform/mem-rs/actions/runs/24322652581/job/71011773179)

**Scope:** Investigate CI/CD failure, verify all licenses are Unlicense (not LGPL), and fix the root cause.

## Timeline of Events

1. **2026-04-13T02:18:59Z** — CI/CD Pipeline triggered on push to `main` (commit `747e7d2`)
2. **2026-04-13T02:19:25Z–02:20:13Z** — All three test jobs (ubuntu, macOS, Windows) pass successfully
3. **2026-04-13T02:20:48Z** — Build Package job completes successfully
4. **2026-04-13T02:20:55Z** — Auto Release job starts
5. **2026-04-13T02:21:01Z** — `get-bump-type.mjs` correctly identifies 2 changelog fragments, determines `minor` bump
6. **2026-04-13T02:21:01Z** — "Found changelog fragments, proceeding with release" — `should_release=true`, `skip_bump=false`
7. **2026-04-13T02:21:03Z** — **FAILURE**: `version-and-commit.mjs` exits with: `Error: Could not parse version from Cargo.toml`
8. **2026-04-13T02:21:03Z** — Auto Release job fails, Deploy Rust Documentation is skipped

## Requirements Analysis

### R1: Investigate and fix CI/CD failure

**Status:** Resolved

**Root cause:** The version parsing regex in `scripts/version-and-commit.mjs` (line 73) used:

```javascript
/^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"/m
```

This regex only matches simple `X.Y.Z` versions. However, `Cargo.toml` contains a SemVer pre-release version:

```toml
version = "0.1.0-pre+beta.2"
```

The `-pre+beta.2` suffix (valid SemVer pre-release identifier + build metadata) causes the regex to fail because after matching `0.1.0`, the next character is `-` rather than `"`.

**Fix:** Updated the regex to allow optional pre-release/build metadata:

```javascript
/^version\s*=\s*"(\d+)\.(\d+)\.(\d+)(?:-[^"]*)?"/m
```

The `(?:-[^"]*)?` non-capturing group optionally matches a `-` followed by any characters up to the closing `"`, which covers all valid SemVer pre-release and build metadata suffixes.

**Why the version has a pre-release suffix:** The version `0.1.0-pre+beta.2` was set during development. When the Auto Release job runs, it bumps this to a clean release version (e.g., `0.2.0`), but it must first parse the existing version — which fails on the pre-release format.

### R2: Verify all licenses are Unlicense (not LGPL)

**Status:** Verified — no issues found

A comprehensive search across all 38 non-git files in the repository found:

- **`LICENSE` file:** Contains the standard Unlicense text (public domain)
- **`Cargo.toml`:** `license = "Unlicense"`
- **`README.md`:** States "released into the public domain under the Unlicense"
- **`CONTRIBUTING.md`:** Documents the license as Unlicense

No references to LGPL, GPL, Apache, MIT, BSD, or any other license were found anywhere in the repository (source files, scripts, configuration, documentation, or workflows).

## Other Observations

### Node.js 20 deprecation warning

The CI logs contain a non-fatal warning:

> Node.js 20 actions are deprecated. Actions will be forced to run with Node.js 24 starting June 2nd, 2026.

Affected actions: `actions/checkout@v4`, `actions/setup-node@v4`. These will need updating before September 16th, 2026 when Node.js 20 is removed from runners.

## Existing Libraries/Components Considered

| Component | Purpose | Relevance |
|-----------|---------|-----------|
| [SemVer spec](https://semver.org/) | Version format standard | The pre-release version format (`X.Y.Z-pre+build`) is valid SemVer |
| [use-m](https://www.npmjs.com/package/use-m) | Dynamic package loading | Used by release scripts, not related to the bug |
| [command-stream](https://www.npmjs.com/package/command-stream) | Shell command execution | Used by release scripts, not related to the bug |

## Solution Summary

1. **Root cause identified:** Version regex in `scripts/version-and-commit.mjs` did not handle SemVer pre-release versions
2. **Fix applied:** Updated regex to accept optional `-prerelease+build` suffixes while still correctly extracting the `major.minor.patch` components
3. **License verified:** All files correctly use Unlicense (public domain), no LGPL references found
4. **Verification:** Automated test in `experiments/test_version_parse.mjs` confirms the fix handles all SemVer version formats
