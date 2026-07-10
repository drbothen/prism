---
document_type: adversarial-review-cascade-summary
scope: LOCAL
defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
fix_branch: fix/csdevices-empty-pipeline
date_pass4: 2026-07-10
total_passes_to_date: 5
date_pass5: 2026-07-10
streak_at_pass5: 0
convergence: IN_PROGRESS
authored_by: state-manager
---

# LOCAL Adversary Cascade Summary — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

**Defect (two independent sub-defects):**

1. **Sub-defect 1 — Sensor TOML spec defect:** `crowdstrike.sensor.toml` `fetch_devices` step
   had no `${...}` interpolation reference in `path` or `body_template`; `find_fan_out_array()`
   returned `None`; GET `/devices/entities/devices/v2` issued with 0 `ids` params → empty DTU
   response. Fix: POST conversion (DTU `PostDeviceDetailsV2` route addition + TOML
   `method`/`body_template` change per architect Option 1 ratification D-1652).

2. **Sub-defect 2 — Product-code defect:** `materialization.rs` skipped DataFusion registration
   for 0-batch tables; mixed JOIN → `DataFusionError::Plan("table not found")` → `-32000`
   "Internal error". Fix: register schema-only empty `MemTable` from declared spec columns
   (BC-2.01.010 / BC-2.11.005 DEC-022).

**Root-cause artifact:** `.factory/research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md`

---

## 4-Pass Cascade Table

| Pass | Frozen HEAD | CLEAN(strict) | CLEAN(PR-merge) | Findings | Fix-burst HEAD | Streak |
|------|-------------|---------------|-----------------|----------|----------------|--------|
| 1 | (initial fix branch @22f429d0 area) | NO | NO | 2 HIGH (F-CSD-P1-001 contract-fidelity, F-CSD-P1-002) + 3 MED + LOWs | spec + code burst | 0/3 |
| 2 | (post-pass-1 fix HEAD) | YES | YES | 0 | — | 1/3 |
| 3 | (post-pass-2 HEAD) | NO | NO | 1 HIGH F-CSD-P3-001 (subquery walk — Expr::InSubquery projection-position execution) | architect adjudication + fix-burst | 0/3 |
| 4 | (frozen @22f429d0 + spec layer) | NO | NO | 1 HIGH F-CSD-P4-001 (unauthorized COUNT rewrite) + 2 MED (F-CSD-P4-002/003) + 1 LOW (F-CSD-P4-004) + 2 OBS incl. F-CSD-P4-005 [process-gap] | spec layer + S-HARDEN-PLAN-PINNING-001 draft | 0/3 |
| 5 | frozen @22f429d0 | NO | NO | 1 HIGH F-CSD-P5-001 (contains_insubquery FuncCall-args recursion gap) + 3 MED (F-CSD-P5-002 POL-24 hint drift / F-CSD-P5-003 stale test-vector row / F-CSD-P5-004 stale position-invariant claim) | — | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer RED @283bbc4b T18/T19+T17-tightened; implementer GREEN @30217403 FuncCall Scalar/Aggregate recursion arms + Window explicit false + byte-exact hint; PO BC-2.11.005 v1.8→v1.9; 1521/1521 prism-query; just check GREEN | 0/3 |

Streak reset at pass 3 and pass 4. Streak 0/3 after pass-5 fix-burst. LOCAL pass 6 NEXT on frozen HEAD `30217403`.

---

## Finding Summary

### Pass 1

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P1-001 | HIGH | Contract-fidelity: POST conversion spec not propagated to BC-2.06.019 | Spec propagation burst (D-1652 + D-1654) |
| F-CSD-P1-002 | HIGH | Sub-defect 2 empty-MemTable gate missing | Implementer TDD fix-burst |
| F-CSD-P1-003..005 | MED | Spec / test coverage gaps | Closed in fix-burst |
| F-CSD-P1-006 | (wording) | BC-2.06.019 merge-brittle sentence | BC-2.06.019 v1.16→v1.17 (D-1654) |

### Pass 2

Zero findings. CLEAN(strict)=YES. Streak 1/3.

### Pass 3

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P3-001 | HIGH | Subquery walk: `Expr::InSubquery` in projection position executed rather than plan-time rejected; misaligned with `Expr::InSubquery` WHERE-position path | Architect adjudication D-1656 — Option A: revert COUNT(*) rewrite; E-QUERY-043 plan-time rejection gate for Expr::InSubquery in non-WHERE positions; BC-2.11.003 v1.12→v1.13; code @22f429d0 (revert + gate + 6 position locks) |

Streak reset 1/3→0/3.

### Pass 4

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P4-001 | HIGH | Unauthorized COUNT(*) rewrite in non-WHERE subquery position introduced in fix-burst for F-CSD-P3-001; produced semantic alteration of count queries | Architect Option A adjudication D-1656: revert rewrite; E-QUERY-043 plan-time gate; code @22f429d0; BC-2.11.003 v1.13 |
| F-CSD-P4-002 | MED | Spec coverage gap in E-QUERY-043 error taxonomy row | error-taxonomy v2.37→v2.38 (E-QUERY-043 row added) |
| F-CSD-P4-003 | MED | DEC-022 position-invariant not fully covered by test vectors in BC-2.11.005 | BC-2.11.005 v1.7→v1.8 (11 test vectors; D-1656) |
| F-CSD-P4-004 | LOW | Pin / documentation finding | Closed in spec layer |
| F-CSD-P4-005 | OBS [process-gap] | `high002_plan_pinning_tests.rs` substring guards insufficient — should be structural SQL-shape assertions | S-HARDEN-PLAN-PINNING-001 draft v0.1 registered (D-1656); codified in lessons.md L25 |

Streak stays 0/3. LOCAL pass 5 NEXT on frozen `22f429d0`.

### Pass 5

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P5-001 | HIGH | `contains_insubquery` helper does not recurse into `Expr::ScalarFunction` / `Expr::AggregateFunction` args — FuncCall-args recursion gap; `Expr::WindowFunction` explicit-false missing | test-writer RED @283bbc4b T18/T19+T17-tightened; implementer GREEN @30217403 FuncCall Scalar/Aggregate recursion arms + Window explicit false |
| F-CSD-P5-002 | MED | POL-24 hint message for E-QUERY-043 does not match byte-exact spec — drift from BC-2.11.003 v1.13 hint wording | implementer GREEN @30217403 byte-exact hint string aligned |
| F-CSD-P5-003 | MED | Stale test-vector row in BC-2.11.005 cited wrong test name; E-QUERY-043 postcondition reference missing | BC-2.11.005 v1.8→v1.9 (PO fix-burst; D-1657) |
| F-CSD-P5-004 | MED | §Postconditions + §DEC-022 incorrectly listed "SELECT Expr::InSubquery" as a position covered by pre-registration claims | BC-2.11.005 v1.8→v1.9 (PO fix-burst; D-1657) |

Streak stays 0/3. LOCAL pass 6 NEXT on frozen `30217403`.

---

## Evidence at Pass 4 Fix HEAD (22f429d0)

- prism-query: **5431 + 6 = 1519** tests GREEN (defect suite 17/17)
- Full workspace `just check`: GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-4 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| error-taxonomy | v2.37→v2.38 | E-QUERY-043 row (Expr::InSubquery non-WHERE position plan-time rejection) |
| BC-2.11.003 | v1.12→v1.13 | Stale subquery invariant replaced; E-QUERY-001 subquery row superseded; E-QUERY-043 row + vectors |
| BC-2.11.005 | v1.7→v1.8 | DEC-022 position-invariant expansion + 11 test vectors |
| S-HARDEN-PLAN-PINNING-001 | NEW draft v0.1 | F-CSD-P4-005 [process-gap] closure story |

## Evidence at Pass 5 Fix HEAD (30217403)

- prism-query: **1521/1521** tests GREEN (defect suite 19/19 after T18+T19+T17-tightened)
- Full workspace `just check`: GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-5 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.11.005 | v1.8→v1.9 | F-CSD-P5-003: stale test-vector row corrected + E-QUERY-043 postcondition reference; F-CSD-P5-004: "SELECT Expr::InSubquery" removed from position-invariant pre-registration claims in §Postconditions + §DEC-022 |
| BC-INDEX | v7.81→v7.82 | BC-2.11.005 inline row updated (D-1657) |

## Architect Adjudication Record (F-CSD-P4-001 / F-CSD-P3-001)

**Option A selected (2026-07-10):** Revert COUNT(*) rewrite. Add E-QUERY-043 plan-time
rejection for `Expr::InSubquery` in non-WHERE projection position. WHERE-position preserved
(existing behavior correct). 6 position locks committed @22f429d0.

Source: `.factory/research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md`
§"Adjudication — Expr::InSubquery projection-position execution (F-CSD-P4-001) — 2026-07-10"

---

## Merge Record

_Pending: LOCAL cascade not yet converged. PR not yet created._

**Status: IN PROGRESS — LOCAL pass 6 NEXT on frozen `30217403` (streak 0/3). Pass-5 fix-burst COMPLETE (D-1657): FuncCall recursion arms + Window false + byte-exact hint @30217403 + BC-2.11.005 v1.9.**
