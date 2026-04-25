# Review Findings — hotfix-fuzz-target (PR #48)

## Convergence Summary

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 | 0 | 0 | 0 | 0 → APPROVE |

**Result:** APPROVED in 1 cycle. Zero findings.

## Cycle 1 Review

**Reviewer verdict:** APPROVE
**Date:** 2026-04-25
**PR:** #48 — fix(ci): hotfix #4 — explicit --target x86_64-unknown-linux-gnu for cargo fuzz

### Diff reviewed

File: `.github/workflows/post-merge.yml`
Changes: 3 additions, 3 deletions (single-line flag addition per `cargo fuzz run` invocation)

### Adversarial checks performed

1. **Target choice correctness:** `--target x86_64-unknown-linux-gnu` vs. removing musl from `rust-toolchain.toml`
   - Verdict: Flag is correct. Musl removal would break the release pipeline cross-compilation.

2. **Flag placement before `--` separator:** Confirmed on all 3 invocations.
   - `cargo fuzz run <harness> --target x86_64-unknown-linux-gnu -- -max_total_time=1800`
   - `--target` is a cargo-fuzz flag (before `--`); `-max_total_time` is a libFuzzer flag (after `--`). Correct.

3. **Completeness:** All 3 harnesses updated — `normalize_fuzz`, `spec_parser`, `fuzz_injection_scanner`. No missed invocation.

4. **Security:** SHA-pinned actions, no new permissions, no new secrets, static string value.

### Findings

None.

## Gate Status at Merge

| Gate | Status |
|------|--------|
| Security review | CLEAN (0 findings) |
| Code review convergence | APPROVED (1 cycle, 0 findings) |
| CI checks | ALL PASS (Format, Clippy, Test ×5 platforms, Audit, Deny, Semver, Workflow) |
| Upstream dependencies | ALL MERGED (#44, #45, #46, #47) |
| Merge | Squash-merged at a4e0e0688f4634b01d8cd56f0b373b70f8ad6d6d |

## Post-Merge

Post-Merge Verification run 24930518890 on develop HEAD `a4e0e068` — COMPLETED with **failure**.

### Cascade closure status: NOT CLOSED — hotfix #5 required

### fuzz-corpus job: FAILED

Root cause: `fuzz/Cargo.toml` has `prism-ocsf` and `serde_json` declared **after** the `[workspace]` table (lines 23-25). In TOML, these lines are parsed as keys under `[workspace]`, not `[dependencies]`. Cargo never sees them as dependencies, so `normalize_fuzz.rs` fails to compile:

```
error[E0432]: unresolved import `prism_ocsf`       → fuzz_targets/normalize_fuzz.rs:45:5
error[E0433]: cannot find module or crate `serde_json` → fuzz_targets/normalize_fuzz.rs:61:23 (×3)
error: could not compile `prism-fuzz` (bin "normalize_fuzz") due to 4 previous errors
```

Fix required: Move `prism-ocsf = { path = "../crates/prism-ocsf" }` and `serde_json = "1"` into the `[dependencies]` block, before the `[workspace]` table.

### Kani job: in_progress at time of report (irrelevant — run already failed)

### Orchestrator decision required: do NOT auto-launch hotfix #5.
