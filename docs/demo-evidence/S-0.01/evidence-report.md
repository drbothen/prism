# Evidence Report — S-0.01: CI/CD Pipeline and Release Workflow

| Field | Value |
|-------|-------|
| Story ID | S-0.01 |
| Title | CI/CD Pipeline and Release Workflow |
| Date | 2026-04-21 |
| Impl SHAs | bd6a04b + a913aa1 + 49eb39b |
| Merge commit | 9de5e29 (merged to develop) |
| Branch | feature/S-0.01-ci-cd-pipeline |
| Product Type | CI/CD configuration — artifact-based evidence |
| Recording Tool | TAP test suite (`tests/ci-gate/run.sh`) + YAML validation |

## Green Gate Summary

```
bash tests/toolchain-gate/run.sh   # (ci-gate suite)

# S-0.01 Red Gate Summary
# Total:  72
# Passed: 72
# Failed: 0
# Skipped (tool not found): 0
```

Full output: [ci-gate-run.txt](ci-gate-run.txt)

## AC Coverage Matrix

| AC | Title | Test File | Tests | Status |
|----|-------|-----------|-------|--------|
| AC-1 | `cargo fmt --check` gate in CI | `test_AC-1_fmt-check.sh` | 3 | GREEN |
| AC-2 | `cargo clippy -D warnings` gate in CI | `test_AC-2_clippy-D-warnings.sh` | 3 | GREEN |
| AC-3 | 5-platform cross-compile matrix | `test_AC-3_matrix-5-platforms.sh` | 12 | GREEN |
| AC-4 | `cargo audit` gate in CI | `test_AC-4_cargo-audit.sh` | 3 | GREEN |
| AC-5 | Kani proofs job in post-merge | `test_AC-5_kani-proofs.sh` | 6 | GREEN |
| AC-6 | Release artifacts (archives + checksums) | `test_AC-6_release-artifacts.sh` | 9 | GREEN |
| AC-7 | Homebrew tap auto-update | `test_AC-7_homebrew-tap.sh` | 6 | GREEN |
| AC-8 | crates.io publish | `test_AC-8_crates-io-publish.sh` | 6 | GREEN |
| AC-9 | No hardcoded secrets in workflow files | `test_AC-9_no-hardcoded-secrets.sh` | 9 | GREEN |

**Total: 9 ACs / 57 assertions + 15 supporting assertions = 72 total — all GREEN**

## Evidence Files

| File | Purpose |
|------|---------|
| `evidence-report.md` | This aggregator (F-CV-001) |
| `AC-1-fmt-check.md` | AC-1: cargo fmt --check step presence |
| `AC-2-clippy-D-warnings.md` | AC-2: clippy -D warnings step presence |
| `AC-3-matrix-5-platforms.md` | AC-3: 5-target cross-compile matrix |
| `AC-4-cargo-audit.md` | AC-4: cargo audit step presence |
| `AC-5-kani-proofs.md` | AC-5: post-merge Kani job |
| `AC-6-release-artifacts.md` | AC-6: archive + checksum creation |
| `AC-7-homebrew-tap.md` | AC-7: Homebrew tap PR automation |
| `AC-8-crates-io-publish.md` | AC-8: crates.io publish job |
| `AC-9-no-hardcoded-secrets.md` | AC-9: secrets via GitHub Secrets only |
| `ci-gate-run.txt` | Full TAP output — 72/72 pass |
| `yaml-validation.txt` | YAML parse validation for all 3 workflow files |

## Known Limitations

1. **FIRST-PR CI gap (resolved):** The initial PR could not self-validate because CI
   runs on the same workflow being added. This is inherent to bootstrapping CI
   configuration and was noted at merge time. CI has been live since PR #1 merged
   and all subsequent PRs run against it.

2. **macOS BSD grep portability (resolved):** A follow-up commit (88d46bf) fixed
   `grep -P` usage (Perl-compatible regex) which is not available on macOS BSD grep.
   All CI scripts now use POSIX-compatible patterns.

3. **Kani + fuzz steps gated (Wave 0 fix):** Post-merge Kani and fuzz steps are now
   gated on `hashFiles()` guards so they skip cleanly until the respective targets
   exist. This was applied as part of the Wave 0 gate remediation (F-WV0-002).

4. **Release binary crate gated (Wave 0 fix):** Release build steps are gated on
   presence of `crates/prism-bin` and skip until that crate is added in a later wave.
   Applied as part of Wave 0 gate remediation (F-WV0-001).

## POL-010 Compliance

All evidence files are under `docs/demo-evidence/S-0.01/` only.
