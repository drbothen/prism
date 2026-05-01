# PR Manifest — W3-FIX-CI-001

## Story
W3-FIX-CI-001: CI wall-clock optimization — cargo-nextest, per-platform PROPTEST_CASES, mold linker

## PR
- Number: #112
- URL: https://github.com/drbothen/prism/pull/112
- Base branch: develop
- Feature branch: fix/W3-FIX-CI-001 (remote deleted post-merge)

## Merge
- Merge SHA: a3bd5a0fd94d147b4052330dfd602cffbb0a8eff
- Merged at: 2026-05-01T17:26:18Z
- Strategy: squash

## Commit Chain (feature branch)
- f027503b — initial CI optimization implementation
- 8677faeb — refinements
- 05873847 — fix: Swatenim → Swatinem typo correction (rust-cache action reference)

## CI Runs
| Run ID | Trigger | Result | Notes |
|--------|---------|--------|-------|
| 25221873737 | PR trigger | SUCCESS | All 12 jobs passed |
| 25221871986 | Push trigger | SUCCESS | All 12 jobs passed |
| 25221871980 | Earlier push | Partial (workspace layout only) | Pre-typo-fix run |
| 25221873736 | Earlier PR | Partial (workspace layout only) | Pre-typo-fix run |

### CI Job Results (run 25221873737 — PR trigger, authoritative)
| Check | Result | Duration |
|-------|--------|----------|
| Format check | PASS | 34s |
| Verify workflow structure (AC-5..AC-8 reachability) | PASS | 16s |
| Workspace crate layout (ADR-012) | PASS | 18s |
| Clippy (AD-008) | PASS | 7m56s |
| Cargo audit (RustSec) | PASS | 37s |
| Cargo deny (license + advisory) | PASS | 58s |
| Semver compatibility | PASS | 2m56s |
| Test (no-default-features) | PASS | 56m4s |
| Test (aarch64-apple-darwin) | PASS | 19m42s |
| Test (x86_64-apple-darwin) | PASS | 16m41s |
| Test (x86_64-pc-windows-msvc) | PASS | 33m6s |
| Test (x86_64-unknown-linux-gnu) | PASS | 1h7m5s |
| Test (x86_64-unknown-linux-musl) | PASS | 23m8s |

## Notable: Windows Test Optimization Validated
- Windows test (x86_64-pc-windows-msvc): 22m53s (run 25221871986) and 33m6s (run 25221873737)
- Prior to this story's optimization: 70+ minutes
- Validates the cargo-nextest + per-platform PROPTEST_CASES + mold linker changes

## Prior Failure (Context)
- Test (no-default-features) previously failed: "Swatenim/rust-cache not found" — typo introduced by implementer
- Orchestrator fixed typo (Swatenim → Swatinem) in commit 05873847
- Post-fix: Test (no-default-features) = PASS on both runs

## Review Agents
- pr-reviewer: APPROVE in 1 cycle (dispatch a14ff4962e30efbe9)
- security-reviewer: CLEAN — 0 CRITICAL/HIGH/MEDIUM/LOW findings

## Dependency
- W3-FIX-LEFTHOOK-001 (PR #106): MERGED at 7418f26957ac59bdbf02914a7df000ae56bf1e1b (2026-05-01T02:30:59Z)

## Gate Summary
| Gate | Result |
|------|--------|
| Security review | CLEAN |
| PR review convergence | APPROVE in 1 cycle |
| CI all checks pass | PASS (both runs) |
| Dependency PR #106 merged | VERIFIED |
| Merge executed | a3bd5a0f on develop |
