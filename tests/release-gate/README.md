# Release Gate Test Suite — S-REL-001

Structural assertion tests for the S-REL-001 release.yml repair acceptance criteria.
This suite is the **authoritative gate** for the release workflow (`release.yml`).

## What This Suite Tests

| Test file | AC | Assertion |
|-----------|-----|-----------|
| test_AC-001_binary-exists-removal.sh | AC-001 | binary-exists step + outputs block removed from build-release job (not merely stubbed) |
| test_AC-002_chocolatey-removal.sh | AC-002 | chocolatey-publish job is removed |
| test_AC-003_homebrew-removal.sh | AC-003 | homebrew-update job is removed |
| test_AC-004_crates-io-removal.sh | AC-004 | crates-io-publish job is removed |
| test_AC-005_prerelease-flag.sh | AC-005 | gh release create uses `--prerelease` flag; idempotent re-run guard (`gh release view`/`upload --clobber`) present; create path splices `"${PRERELEASE_ARGS[@]}"` (SID-2); CWE-78 regression guard: no `${{ github.ref* }}`, `${{ github.event* }}`, `${{ github.head_ref }}`, or `${{ env.* }}` inside any `run:` block (F-REL001-P1-001 / F-REL001-P18-001) |
| test_AC-006_matrix-targets.sh | AC-006 | All 5 platform targets present and correctly spelled; exactly 5 matrix entries |
| test_AC-007_checksums.sh | AC-007 | SHA-256 checksum step present and checksums.txt attached to release |
| test_AC-008_oidc-attestation.sh | AC-008 | OIDC attestation step present (actions/attest-build-provenance v4.1.1) |
| test_AC-009_demo-server-build.sh | AC-009 | demo-server build job present in the tag-triggered release matrix |
| test_AC-010_linux-setup.sh | AC-010 | Linux apt dependencies installed (musl-tools, libdbus-1-dev, etc.); protoc toolchain pinned (arduino/setup-protoc SHA+version comment, no if: gate — DEFECT-REL001-PROTOC-MISSING-001); §15 cargo-zigbuild design: full apt-get install composed line (9a) + clang absent from apt-get install line (9b, §15/F-REL001-P16-001 negative guard, comment-stripped scope — pass-16) + pip3 --require-hashes + cargo-zigbuild@0.23.0 + composed zigbuild line + requirements-musl-ci.txt (ziglang==0.16.0 pin + sha256 hash) + CXX_x86_64_unknown_linux_musl=clang++ ABSENT (DEFECT-REL001-MUSL-LIBSTDCXX-001, SID-2 composed assertions, 9 assertions); §14 Option B linux-gnu persistence invariant: target-cfg header present + keyring linux-native-sync-persistent in block (awk block-scoped) + NOT in default features (negative guard) — F-REL001-P14-001, crates/prism-credentials/Cargo.toml (3 assertions) |
| test_AC-011_actionlint.sh | AC-011 | actionlint exits 0 on release.yml (requires: brew install actionlint) |
| test_AC-012_install-scripts.sh | S-REL-003 AC-001..AC-010 | install.sh and install.ps1 exist with required structure (set -euo pipefail, dual-path SHA-256, all 5 platform targets, composite musl detection, /releases?per_page=1 not /releases/latest, PATH guidance, #Requires -Version 5.1, Get-FileHash, checksum-mismatch abort); ci.yml has shellcheck-install-scripts + psscriptanalyzer-install-ps1 jobs; release.yml publish-release uploads both scripts (ADJ-002) |

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
| actionlint | test_AC-011 | `brew install actionlint` — FAIL if absent — gate fails closed (AC-012/POL-34) |

No network access required. No files are modified.

## Red Gate Status

AC-001..AC-005, AC-008, AC-009, AC-010, AC-011 were expected to fail before
implementation. AC-006, AC-007, and partial AC-008 pass on the broken/dead jobs
in the original release.yml (the original workflow already had the correct matrix
shape for those checks).
See `.factory/cycles/v1.0.0-release-engineering/S-REL-001/implementation/red-gate-log.md` for details.

After implementation, all 123 assertions should pass. actionlint must be installed
(absent actionlint is a hard failure, not a skip — the gate fails closed).

## Floor Constants (F-REL001-P7-001)

`run.sh` enforces **exact** expected counts so that silently deleting a test file
or losing assertions causes a loud harness failure rather than a silent pass:

| Constant | Value | Meaning |
|----------|-------|---------|
| `EXPECTED_TEST_FILES` | 12 | Number of `test_AC-*.sh` files that must be executed |
| `EXPECTED_ASSERTIONS` | 123 | Total TAP assertions across all test files |

Exact equality is used (not `>=`), following the `scripts/check-non-exhaustive.sh
EXPECTED=92` precedent. An unexpected *increase* also requires a conscious constant
bump — coverage drift in either direction is flagged.

### When the suite grows (three places to update)

1. `run.sh` — bump `EXPECTED_TEST_FILES` and/or `EXPECTED_ASSERTIONS`
2. This README — update the table above
3. The test file table at the top of this README — add the new row

Failure to update all three will cause `run.sh` to exit non-zero with a message
like `HARNESS ERROR: expected 11 test files, executed 12`.
