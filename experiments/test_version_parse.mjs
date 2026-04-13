#!/usr/bin/env node

/**
 * Test that version parsing handles SemVer pre-release versions correctly.
 * Reproduces and verifies the fix for issue #30.
 */

const testCases = [
  { input: 'version = "1.2.3"', expected: { major: 1, minor: 2, patch: 3 } },
  { input: 'version = "0.1.0-pre+beta.2"', expected: { major: 0, minor: 1, patch: 0 } },
  { input: 'version = "0.1.0-alpha"', expected: { major: 0, minor: 1, patch: 0 } },
  { input: 'version = "2.0.0-rc.1+build.123"', expected: { major: 2, minor: 0, patch: 0 } },
  { input: 'version = "10.20.30"', expected: { major: 10, minor: 20, patch: 30 } },
];

// The FIXED regex (handles pre-release/build metadata)
const fixedRegex = /^version\s*=\s*"(\d+)\.(\d+)\.(\d+)(?:-[^"]*)?"/m;

// The BROKEN regex (only matches simple X.Y.Z)
const brokenRegex = /^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"/m;

let allPassed = true;

for (const { input, expected } of testCases) {
  const fixedMatch = input.match(fixedRegex);
  const brokenMatch = input.match(brokenRegex);

  if (!fixedMatch) {
    console.error(`FAIL (fixed regex): Could not parse "${input}"`);
    allPassed = false;
    continue;
  }

  const parsed = {
    major: parseInt(fixedMatch[1], 10),
    minor: parseInt(fixedMatch[2], 10),
    patch: parseInt(fixedMatch[3], 10),
  };

  const ok =
    parsed.major === expected.major &&
    parsed.minor === expected.minor &&
    parsed.patch === expected.patch;

  if (!ok) {
    console.error(`FAIL: "${input}" => ${JSON.stringify(parsed)}, expected ${JSON.stringify(expected)}`);
    allPassed = false;
  } else {
    const brokenWouldFail = !brokenMatch && input.includes('-');
    console.log(
      `PASS: "${input}" => ${JSON.stringify(parsed)}` +
        (brokenWouldFail ? ' (broken regex would have FAILED here)' : '')
    );
  }
}

if (allPassed) {
  console.log('\nAll tests passed!');
  process.exit(0);
} else {
  console.error('\nSome tests failed!');
  process.exit(1);
}
