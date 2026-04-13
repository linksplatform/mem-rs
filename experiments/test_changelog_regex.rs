#!/usr/bin/env rust-script
//! Test that the changelog version extraction regex works correctly
//! without using look-ahead (which Rust's regex crate doesn't support).
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```

use regex::Regex;

fn get_changelog_for_version(content: &str, version: &str) -> String {
    let escaped_version = regex::escape(version);
    let header_pattern = format!(r"(?m)^## \[{}\]", escaped_version);
    let header_re = Regex::new(&header_pattern).unwrap();

    if let Some(m) = header_re.find(content) {
        let after_header = &content[m.end()..];
        let body_start = after_header.find('\n').map_or(after_header.len(), |i| i + 1);
        let body = &after_header[body_start..];

        let next_section_re = Regex::new(r"(?m)^## \[").unwrap();
        let section_body = if let Some(next) = next_section_re.find(body) {
            &body[..next.start()]
        } else {
            body
        };

        let trimmed = section_body.trim();
        if trimmed.is_empty() {
            format!("Release v{}", version)
        } else {
            trimmed.to_string()
        }
    } else {
        format!("Release v{}", version)
    }
}

fn main() {
    let changelog = r#"# Changelog

## [0.2.0] - 2026-04-13

### Changed
- Migrated from Rust nightly to stable Rust

### Fixed
- Fixed clippy warnings

## [0.1.0] - 2026-04-01

### Added
- Initial release
"#;

    // Test extracting v0.2.0
    let result = get_changelog_for_version(changelog, "0.2.0");
    assert!(result.contains("Migrated from Rust nightly"), "Should contain v0.2.0 content, got: {}", result);
    assert!(result.contains("Fixed clippy warnings"), "Should contain v0.2.0 fixes, got: {}", result);
    assert!(!result.contains("Initial release"), "Should NOT contain v0.1.0 content, got: {}", result);
    println!("PASS: v0.2.0 extraction correct");

    // Test extracting v0.1.0
    let result = get_changelog_for_version(changelog, "0.1.0");
    assert!(result.contains("Initial release"), "Should contain v0.1.0 content, got: {}", result);
    assert!(!result.contains("Migrated"), "Should NOT contain v0.2.0 content, got: {}", result);
    println!("PASS: v0.1.0 extraction correct");

    // Test non-existent version
    let result = get_changelog_for_version(changelog, "9.9.9");
    assert_eq!(result, "Release v9.9.9");
    println!("PASS: missing version returns default");

    // Test single-section changelog (no next section)
    let single = r#"# Changelog

## [1.0.0] - 2026-01-01

### Added
- The only feature
"#;
    let result = get_changelog_for_version(single, "1.0.0");
    assert!(result.contains("The only feature"), "Should extract from single section, got: {}", result);
    println!("PASS: single section extraction correct");

    println!("\nAll tests passed!");
}
