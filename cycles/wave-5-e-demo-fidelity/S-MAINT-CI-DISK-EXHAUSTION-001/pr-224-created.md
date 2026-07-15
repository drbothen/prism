---
document_type: stage-transition
story: S-MAINT-CI-DISK-EXHAUSTION-001
event: PR-OPEN
date: 2026-07-15
pr_number: 224
---
# S-MAINT-CI-DISK-EXHAUSTION-001 — PR #224 Created

**Stage transition: LOCAL 3-CLEAN CONVERGED → PUSHED + PR OPEN**

## Branch

- Branch: `maintenance/ci-disk-hardening`
- HEAD: `e48033e4a3b8face7124826914ca4ce35fdd6309` (e48033e4)
- Commits over develop@5f1b5771: 7
- Files changed: 1 (`.github/workflows/ci.yml`); 132 insertions / 10 deletions
- LOCAL-ONLY status: ENDED as of this push

## PR

- PR #224: https://github.com/drbothen/prism/pull/224
- Title: `ci(S-MAINT-CI-DISK-EXHAUSTION-001): disk-exhaustion hardening — reclaim + gates on Linux workspace-build jobs`
- Base: `develop`
- Head: `maintenance/ci-disk-hardening`

## LOCAL Convergence Summary

- 10 adversarial passes / 6 fix-bursts
- 3-CLEAN(strict) CONVERGED: passes 8/9/10 on frozen HEAD e48033e4
- DRIFT-ORCH-PRLEVEL-PUSH-001: satisfied (zero commits pushed since fix-burst-6)

## BC Governance

- `behavioral_contracts: []` CONFORMING — PO Option-B adjudication 2026-07-15
- Controlling precedent: W3-FIX-CI-001 (merged, PR #112)
- POL-14: no BC draft→active promotions ride this PR (no BCs authored)

## Scope

CI toolchain only — `.github/workflows/ci.yml`. No Rust crate source changes.

| What | Detail |
|------|--------|
| df preflight step | Both linux-test + test-no-default-features jobs; before checkout |
| disk-space-reclaimer | `insightsengineering/disk-space-reclaimer@dae9fabcb8febe09f6585471948acf9dc9a57489 # v1.1.2`; `swap-storage: false` (EC-008); 21–31 GB freed |
| ≥25 GB gate | df -P / 1K-block arithmetic + AVAIL_GB guard; early-fail exit if below 25 GB |
| failure annotation | `if: failure()` USED_PCT guard + ::warning:: emit in both Linux jobs |
| verify-workflow-structure | 4 new assertions (RG-1..RG-4) + 7 anchored-in-place; 13 total (11 reach + 2 config-invariant) |
| CARGO_PROFILE_DEV_DEBUG | NOT added (no-op; F-CIDISK-P4-HIGH-001 adjudication) |

## AC-005 Evidence Status

Three consecutive green CI runs on this PR required before merge. PENDING — this PR's own CI runs are the evidence. Run IDs to be recorded in PR description once available.

## Origin

D-1780 watch-note (3rd disk-exhaustion occurrence threshold):
- Run 29394488318: x86_64-unknown-linux-musl; `mold: failed to write to an output file. Disk full?`
- Run 29399778005: x86_64-unknown-linux-gnu; `couldn't create a temp dir: No space left on device`
- Run 29404746333: x86_64-unknown-linux-gnu; `mold: Disk full`
- 4th occurrence: f715b0a5; `No space left on device` (D-1780 triggered)

## pr-manager 9-Step Status

| Step | Status |
|------|--------|
| 1 (populate PR description) | DONE |
| 2 (demo evidence) | N/A (CI toolchain; AC-005 evidence = PR's own CI runs) |
| 3 (create PR) | DONE — PR #224 |
| 4–9 | DEFERRED → PR-LEVEL cascade (orchestrator-driven) |

## Next Step

PR-LEVEL adversarial cascade on frozen pushed HEAD `e48033e4`.
- Fresh streak: 0/3 (BC-5.39.001)
- DRIFT-ORCH-PRLEVEL-PUSH-001: no pushes mid-cascade
- AC-005: record 3 consecutive green CI run IDs in PR description once available
