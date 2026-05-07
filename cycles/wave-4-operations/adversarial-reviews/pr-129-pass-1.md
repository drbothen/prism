---
document_type: adversarial-review-pass
pass_number: 1
pr_number: 129
story_id: S-3.02
branch_sha: a4316370
factory_artifacts_sha: 9a6be8c9
diff_base_sha: 2a7b83f5
verdict: BLOCKED
convergence_window: 0/3
reviewer: adversary
timestamp: 2026-05-06
producer: adversary
note: post-rebase first pass (PR #129 second cycle)
inputs:
  - .factory/stories/S-3.02-query-materialization.md
  - .factory/specs/behavioral-contracts/BC-2.11.005-ephemeral-materialization.md
  - .factory/specs/behavioral-contracts/BC-2.11.006-query-security-limits.md
  - .factory/specs/behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md
  - .factory/specs/behavioral-contracts/BC-2.11.011-cross-client-query-scoping.md
  - .factory/policies.yaml
input-hash: "[live-adv-review pass-1 post-rebase]"
traces_to: PR-129
---

# PR #129 Adversarial Pass-1 Post-Rebase — BLOCKED

## Verdict: BLOCKED — 0 CRIT / 0 HIGH / 3 MED / 1 LOW / 2 OBS / 3 KUDO

Convergence window: **0 / 3**.

Severity decay vs PR #129 first cycle (pre-rebase): 1 CRIT + 3 HIGH + 4 MED → 0 CRIT + 0 HIGH + 3 MED. Strong decay; remaining findings are spec/doc drift, NOT code defects. Pre-rebase fix bundle (8 fixes) and post-rebase fix bundle (6 fixes) are mostly clean — only SUG-018 missed a sibling line.

## Findings

### F-PR129-PR-MED-A — BC-2.11.005 v1.3 stale E-QUERY-005 for record cap
- Severity: MEDIUM
- Where: BC-2.11.005-ephemeral-materialization.md:69, :87
- What: BC still cites E-QUERY-005 for 10K record cap. BC-2.11.006 v1.12 reconciled SoT to E-QUERY-003 = records limit. Implementation correctly emits E-QUERY-003 at materialization.rs:186. Cross-BC drift.
- Fix: Bump BC-2.11.005 to v1.4 with E-QUERY-005 → E-QUERY-003 corrections.

### F-PR129-PR-MED-B — Story S-3.02 v1.10 has 3 stale scopeguard references
- Severity: MEDIUM
- Where: S-3.02 lines 86, 172, 342
- What: F-PR129-CR-002 removed scopeguard direct dep + replaced with plain Drop, but story body wasn't updated.
- Fix: Bump S-3.02 v1.10 → v1.11 with scopeguard references replaced by plain-Drop terminology.

### F-PR129-PR-MED-C — Cargo.toml line 62 retains S-3.2.08 typo (partial-fix regression of SUG-018)
- Severity: MEDIUM (blast radius = 1 file but same-commit sibling line missed)
- Where: crates/prism-query/Cargo.toml:62
- What: Commit a4316370 fixed line 31 but missed sibling typo at line 62.
- Fix: One-line edit at line 62.

### F-PR129-PR-LOW-A — AC-9 cold-start tests overstate verification
- Severity: LOW (stub-phase tolerated)
- Where: integration_tests.rs:341 (test name) + line 15 (module docstring)
- What: Test name claims "triggers live fetch and writes to buffer" but body only injects descriptor. Actual SensorAdapter call + EventBufferStore write + INFO log assertion all missing because run_materialization_pipeline is todo!().
- Fix: Either rename test to truthful name + #[ignore] stub for full AC-9, or rename + file TD entry.

## Observations

### F-PR129-PR-OBS-A [process-gap] — Cross-BC error-code reconciliation lacks propagation enforcement
- Severity: OBS [process-gap]
- What: BC-2.11.006 v1.12 SoT reconciliation propagated to BC-2.11.006 + S-3.02 v1.10 but not to sibling BC-2.11.005. Third recurrence of cross-BC propagation gap (PR-130 BC-INDEX, PR-127 perimeter sync, PR-129 BC-2.11.005).
- Recommendation: Add baseline policy `policies.yaml#11 e_query_code_canonical_mapping_propagation`. Add CI lint hook.

### F-PR129-PR-OBS-B — VP-031 fixture column names diverge from BC-2.11.007 example list
- Severity: OBS (informational; cosmetic)
- Where: proofs/vp031_pushdown.rs:43-55 vs BC-2.11.007:120-123
- Why OBS: BC explicitly states "illustrative... actual REQUIRED columns determined by TOML spec". VP-031 logic doesn't depend on per-sensor labeling.

## KUDOs

- K-1: HIGH-001 sanitization pattern parity with SEC-003 across three error sites (clean siblings audit)
- K-2: IMP-001 pub(crate) scoping is structurally enforced (no external bypass possible)
- K-3: CR-003 typed accessor encapsulation withstands test-suite usage audit

## Pre-Rebase Fix Bundle (8 fixes): 8/8 CLOSED

CR-001/002/003/004/005/007/008/009 + SEC-001/002/003 all survive post-rebase cleanly.

## Post-Rebase Fix Bundle (6 fixes): 5/6 CLEAN, 1 PARTIAL

| Commit | Fix | Status |
|--------|-----|--------|
| 762f3b60 | HIGH-001 sanitize map_datafusion_memory_error | CLOSED |
| e64cb688 | IMP-001 perimeter pub→pub(crate) | CLOSED |
| 74c721e0 | IMP-002 build_session_context redaction | CLOSED |
| 8ec7c6b5 | MED-002 translate_push_down_filter todo!() | CLOSED |
| 4ac0fcfa | SUG-017 as_any returns self | CLOSED |
| a4316370 | SUG-018 Cargo.toml typo | PARTIAL → P1-MED-C |

## Cross-PR Contamination from S-3.06: NONE

S-3.06's perimeter expansions (10 symbols incl. parse_sql_dml_with_limits), Visitor extensions (visit_dml_node + visit_write_node), case-insensitive internal table guard, and BC-2.11.003 v1.4 SQL denylist all intact post-rebase.

## 7-Lens Verification

| Lens | Status |
|------|--------|
| 1. Pre-rebase fix bundle survival | PASS |
| 2. Post-rebase fix bundle | PASS with caveats (SUG-018 partial) |
| 3. BC alignment | FAIL (BC-2.11.005 stale) |
| 4. Code correctness | PASS |
| 5. Story body↔code coherence | FAIL (scopeguard refs) |
| 6. Cross-PR contamination | PASS |
| 7. Process-gap | 1 [process-gap] (OBS-A) |

## Convergence Window State

- 0/3 (BLOCKED)
- After fix bundle: PO BC-2.11.005 v1.4 `74909d84` + story-writer S-3.02 v1.11 `c0ba6361` + implementer worktree `8727201b` + factory-artifacts `8a7123d5` (TD-S302-005 + TD-VSDD-063)
- Required for advance: pass-2 CLEAN
