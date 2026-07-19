# Release Gate Test Suite — S-REL-001

Structural assertion tests for the S-REL-001 release.yml repair acceptance criteria.
This suite is the **authoritative gate** for the release workflow (`release.yml`).

## What This Suite Tests

| Test file | AC | Assertion |
|-----------|-----|-----------|
| test_AC-001_binary-exists-removal.sh | AC-001 | binary-exists job is removed (not merely stubbed) |
| test_AC-002_chocolatey-removal.sh | AC-002 | chocolatey-publish job is removed |
| test_AC-003_homebrew-removal.sh | AC-003 | homebrew-update job is removed |
| test_AC-004_crates-io-removal.sh | AC-004 | crates-io-publish job is removed |
| test_AC-005_prerelease-flag.sh | AC-005 | gh release create uses `--prerelease` flag |
| test_AC-006_matrix-targets.sh | AC-006 | All 5 platform targets present and correctly spelled; exactly 5 matrix entries |
| test_AC-007_checksums.sh | AC-007 | SHA-256 checksum step present and checksums.txt attached to release |
| test_AC-008_oidc-attestation.sh | AC-008 | OIDC attestation step present (actions/attest-build-provenance v1.4.1+) |
| test_AC-009_demo-server-build.sh | AC-009 | demo-server build job present for PR previews |
| test_AC-010_linux-setup.sh | AC-010 | Linux apt dependencies installed (musl-tools, libdbus-1-dev, etc.) |
| test_AC-011_actionlint.sh | AC-011 | actionlint exits 0 on release.yml (requires: brew install actionlint) |

## How to Run

```bash
# From the worktree root:
bash tests/release-gate/run.sh

# Or run a single AC test:
bash tests/release-gate/test_AC-006_matrix-targets.sh
```

## TAP Harness

The suite produces [TAP (Test Anything Protocol)](https://testanything.org/) output.
`run.sh` aggregates all `test_AC-*.sh` files and exits 1 if any test fails.

## BSD-grep Compatibility

All scripts use POSIX character classes (`[[:space:]]`) rather than GNU extensions
(`\s`, `\d`, `\b`). They run correctly on macOS `/usr/bin/grep` (BSD grep 2.6.0)
without requiring Homebrew GNU grep.

## External Tool Requirements

| Tool | Used by | Install |
|------|---------|---------|
| bash 3.2+ | all tests | ships with macOS |
| grep (BSD or GNU) | all tests | ships with macOS |
| actionlint | test_AC-011 | `brew install actionlint` (SKIP if absent) |

No network access required. No files are modified.

## Red Gate Status

AC-001..AC-005, AC-008, AC-009, AC-010, AC-011 were expected to fail before
implementation. AC-006, AC-007, and partial AC-008 pass on the unimplemented
release.yml (the unimplemented stubs already had the correct matrix shape).
See `.factory/stories/S-REL-001.md` red-gate-log for details.

After implementation, all 58 assertions should pass (1 SKIP for actionlint if
not installed).
