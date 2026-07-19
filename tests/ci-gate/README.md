# CI Gate Test Suite — S-0.01

Structural assertion tests for the S-0.01 CI/CD pipeline acceptance criteria.
These tests are intentionally **failing** until the implementer fills in the workflow stubs.

## What This Suite Tests

| Test file | AC | Assertion |
|-----------|-----|-----------|
| test_AC-1_fmt-check.sh | AC-1 | `cargo fmt --check` is a real run step (not an echo stub) |
| test_AC-2_clippy-D-warnings.sh | AC-2 | `cargo clippy -- -D warnings` is a real run step (AD-008) |
| test_AC-3_matrix-5-platforms.sh | AC-3 | All 5 platform targets present; runners match; musl-tools installed |
| test_AC-4_cargo-audit.sh | AC-4 | `cargo audit` + `cargo deny check` are real steps; step order is fmt→clippy→test→deny→audit→semver |
| test_AC-5_kani-proofs.sh | AC-5 | Post-merge kani job: real invocation, --timeout 300, --mem-limit 8192, artifact upload, all 6 fuzz targets |
| test_AC-6_release-artifacts.sh | AC-6 | Release workflow: v* tag trigger, 5 targets, `--locked` build, sha256sum, gh release create |
| ~~test_AC-7_homebrew-tap.sh~~ | ~~AC-7~~ | **Superseded by S-REL-001**: homebrew-update and crates-io-publish jobs were removed from the release workflow. Tests deleted as part of S-REL-001 release.yml repair. |
| ~~test_AC-8_crates-io-publish.sh~~ | ~~AC-8~~ | **Superseded by S-REL-001**: see AC-7 note above. |
| test_AC-9_no-hardcoded-secrets.sh | AC-9 | All secrets referenced via `secrets.VARNAME`; no hardcoded values |

> **Note:** The authoritative test suite for the release workflow is `tests/release-gate/` (S-REL-001).
> That suite covers release.yml AC-001..AC-011 using a TAP harness compatible with BSD grep (macOS).
> `tests/ci-gate/` covers ci.yml, post-merge.yml, and release.yml (test_AC-6 asserts release.yml
> for AC-6, which predates the S-REL-001 suite split) — AC-1..AC-6, AC-9.

## How to Run

```bash
# From the worktree root:
bash tests/ci-gate/run.sh

# Or run a single AC test:
bash tests/ci-gate/test_AC-3_matrix-5-platforms.sh
```

## External Tool Requirements

| Tool | Used by | Install |
|------|---------|---------|
| bash 3.2+ | all tests | ships with macOS |
| grep | all tests | ships with macOS |

No network access required. No files are modified.

## Known Limitations

- **AC-9 runtime masking**: GitHub Actions automatically masks `secrets.VARNAME` values in
  logs at runtime. This cannot be verified locally — the test verifies structural compliance
  (references use `secrets.VARNAME` syntax and no 40+ char raw tokens appear outside `${{ }}`).

- **Merge gate enforcement (live GitHub config)**: ACs 1–4 ultimately require repository
  branch protection rules (required_status_checks) configured in GitHub. That configuration
  is outside the scope of these YAML-file assertions. The tests confirm the workflow structure
  that enables enforcement, not the enforcement settings themselves.

## Red Gate Status

All tests are expected to **FAIL** against the current stubs. That is the purpose of this suite.
When implementation (step 3) is complete, all tests should pass. If a test passes before
implementation, it is likely testing the wrong thing — investigate before proceeding.
