# Red Gate Log — S-REL-001

**Story:** S-REL-001 — devops: release.yml repair (DEF-REL-001..004, prerelease, demo-server, Linux setup)
**Wave:** F-A  **Cycle:** v1.0.0-release-engineering
**Test suite:** `tests/release-gate/run.sh`
**Entrypoint:** `bash tests/release-gate/run.sh`
**Commit:** 76e68898 (branch feature/S-REL-001)
**Verified against:** `.github/workflows/release.yml` @ e116a587 (develop HEAD at story start)
**Date:** 2026-07-19

---

## Red Gate Summary

```
Total assertions: 58
Passed:           26
Failed:           32
Skipped:           0
Suite exit code:   1 (RED GATE ACTIVE)
```

**RED GATE IS ACTIVE.** Implementation required before any assertion can be declared green.

---

## Per-AC Red Gate State

| AC | File | Status | FAIL count | PASS count | Notes |
|----|------|--------|-----------|-----------|-------|
| AC-001 | test_AC-001_binary-exists-removal.sh | RED | 4 | 1 | binary_exists, check_binary, composed condition, needs.build-release.outputs — all present in file |
| AC-002 | test_AC-002_chocolatey-removal.sh | RED | 4 | 1 | chocolatey-publish job, choco pack, choco push, nuspec — all in functional lines |
| AC-003 | test_AC-003_homebrew-removal.sh | RED | 4 | 1 | homebrew-update job present; 1898co/homebrew-tap present; Formula/prism.rb present; S-REL-008 absent |
| AC-004 | test_AC-004_crates-io-removal.sh | RED | 3 | 1 | crates-io-publish job, cargo publish, CARGO_REGISTRY_TOKEN — all in functional lines |
| AC-005 | test_AC-005_prerelease-flag.sh | RED | 3 | 2 | Array form and dash-detection absent; quality gate for forbidden pattern passes at Red |
| AC-006 | test_AC-006_matrix-targets.sh | GREEN | 0 | 9 | All 5 targets correctly spelled; typo x86_x64 absent; count=5; fail-fast: false present |
| AC-007 | test_AC-007_checksums.sh | GREEN | 0 | 4 | sha256sum + checksums.txt present; merge step present |
| AC-008 | test_AC-008_oidc-attestation.sh | RED | 2 | 4 | id-token:write and SHA-pin pass; v4.1.1 comment absent (has v4.1.0) |
| AC-009 | test_AC-009_demo-server-build.sh | RED | 6 | 1 | prism-dtu-demo-server completely absent from file |
| AC-010 | test_AC-010_linux-setup.sh | RED | 5 | 1 | No apt-get step; libdbus-1-dev, musl-tools, pkg-config, ADR-034/BC-2.06.003 comment all absent |
| AC-011 | test_AC-011_actionlint.sh | RED | 1 | 1 | actionlint exits 1 (SC2086 shellcheck on lines 45 and 71) |

---

## Pre-existing Pass Observations

The following assertions pass BEFORE implementation. The implementer must not break them:

- **AC-006 (all 9 assertions):** All 5 matrix targets are correctly spelled in the current file.
  The musl typo (U1: x86_x64 → x86_64) was already corrected. Matrix count is 5. fail-fast: false is present.
- **AC-007 (all 4 assertions):** sha256sum + shasum -a 256 conditional present. checksums.txt
  referenced. Multi-platform merge step present.
- **AC-008 partial (4/6 assertions):** id-token: write present. attest-build-provenance step present.
  SHA pin (40-char hex) present. Only the v4.1.1 version comment is wrong (currently v4.1.0).

---

## Actionlint Detail (AC-011 failure)

```
.github/workflows/release.yml:45:9: shellcheck SC2086:info:2:32: Double quote to prevent globbing and word splitting
.github/workflows/release.yml:45:9: shellcheck SC2086:info:4:33: Double quote to prevent globbing and word splitting
.github/workflows/release.yml:71:9: shellcheck SC2086:info:9:30: Double quote to prevent globbing and word splitting
exit=1
```

These SC2086 findings are in the `check_binary` step (line 45) and `Create archive` step (line 71).
After implementation, the `check_binary` step is removed (AC-001) and the archive step uses
quoted variables. The repaired file must exit 0.

---

## Conflict Note for Implementer

After S-REL-001 implementation:
- `tests/ci-gate/test_AC-7_homebrew-tap.sh` will FAIL (expects homebrew-update job to exist).
- `tests/ci-gate/test_AC-8_crates-io-publish.sh` will FAIL (expects crates-io-publish job to exist).

These ci-gate tests are from S-0.01 and have OPPOSITE expectations from S-REL-001.
The implementer must remove or replace these two ci-gate tests as part of the S-REL-001
implementation. The release-gate suite (`tests/release-gate/`) supersedes them for
release.yml assertions.

---

## Test File Paths

```
tests/release-gate/tap_lib.sh
tests/release-gate/run.sh
tests/release-gate/test_AC-001_binary-exists-removal.sh
tests/release-gate/test_AC-002_chocolatey-removal.sh
tests/release-gate/test_AC-003_homebrew-removal.sh
tests/release-gate/test_AC-004_crates-io-removal.sh
tests/release-gate/test_AC-005_prerelease-flag.sh
tests/release-gate/test_AC-006_matrix-targets.sh
tests/release-gate/test_AC-007_checksums.sh
tests/release-gate/test_AC-008_oidc-attestation.sh
tests/release-gate/test_AC-009_demo-server-build.sh
tests/release-gate/test_AC-010_linux-setup.sh
tests/release-gate/test_AC-011_actionlint.sh
```
