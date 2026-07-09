---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [1]
feature_head_at_review: d56e933e
fix_burst_head: c2ece301
date: 2026-07-08
clean_strict: false
clean_pr_merge: false
finding_counts:
  HIGH: 2
  MED: 2
  OBS: 2
  total: 6
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 1 — FIX-IEQ-ERRPATH-001

---

## Pass 1 (frozen d56e933e; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO
**Findings:** 6 (2 HIGH + 2 MED + 2 OBS)
**Code HEAD at review:** d56e933e (frozen; initial fix-branch HEAD)
**Fix-burst HEAD:** c2ece301 (chain: d56e933e → 8c8b5628 test-writer 6 RED → 741e1506 implementer positions 9-12 + engine-layer E-QUERY-002 type-compat → c2ece301 suggested_column b1-parity via pub(crate) ocsf_suggested_string_column reuse)
**LOCAL 3-CLEAN(strict) streak after pass-1:** 0/3 (NOT CLEAN; fix-burst dispatched)

---

## Finding ADV-FIX-P1-HIGH-001 — BC-2.11.016 v1.5 six-position list reverse spec-drift; gate coverage did not match full AST

**Severity:** HIGH
**Classification:** spec-code drift / reverse spec-drift (spec lagging implementation)
**Affected files:** `.factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md`
**BC reference:** BC-2.11.016 v1.5

**Finding:** BC-2.11.016 v1.5 listed only six AST positions in the gated E-QUERY-038 ColumnNotFound trigger list. The full AST coverage required by the WARN-2 fix spans twelve positions (Filter/Pipe gate covers pipe-mode predicates, SqlPipe WHERE stages, and all remaining pipe-stage column references). The spec as written at v1.5 did not authorize coverage of Filter/Pipe gate positions, creating a reverse spec-drift: the implementation needed to be more comprehensive than the BC permitted. A strict reading of v1.5 would have incorrectly constrained the fix-burst to six positions only.

**Root cause:** The BC was authored before the full AST-position analysis was completed. The six-position list was an undercount derived from the initial T13 WARN-2 diagnosis (IEQ pre-flight path only) rather than a complete AST traversal survey.

**Closure:** CLOSED — BC-2.11.016 v1.5→v1.6: twelve-position list replacing six-position list; EC-11-047..EC-11-052 added for positions 7-12; §Invariants AST-completeness invariant added (ADV-FIX-P1-OBS-002 co-closure). v1.7 follow-up: |project→|fields grammar-verified keyword fix (correct PQL keyword, not SQL alias).

---

## Finding ADV-FIX-P1-HIGH-002 — SqlPipe |where stages bypass E-QUERY-038 gate (TD-VSDD-060 sibling site)

**Severity:** HIGH
**Classification:** correctness / TD-VSDD-060 sibling-site sweep gap
**Affected files:** `crates/prism-query/src/` (SqlPipe |where dispatch path)
**BC reference:** BC-2.11.016 v1.5 (gate scope)

**Finding:** The initial fix-burst addressed the IEQ/IIN/INE `check_ci_column_types` pre-flight path for pipe-mode. However, SqlPipe mode (SQL prefix + pipe stages) routes `|where` filter stages through a separate dispatch arm that was not connected to the E-QUERY-038 column existence gate. A TD-VSDD-060 sibling-site sweep of all column-reference-consuming AST arms revealed that `|where` in SqlPipe mode bypassed the gate. A user running `SELECT * FROM cyberint_alerts | where severity_id IEQ 'High'` would still get an opaque internal error instead of E-QUERY-038.

**Root cause:** The initial fix targeted only the pipe-mode IEQ/IIN/INE operator path. The SqlPipe |where path shares the same AST position class but routes through a different dispatch arm. TD-VSDD-060 sweep discipline was required but not applied to the SqlPipe arm at initial fix-burst authoring time.

**Closure:** CLOSED — implementer commit 741e1506: SqlPipe |where dispatch arm wired to E-QUERY-038 gate (positions 9-12); gate now covers all twelve AST positions per BC-2.11.016 v1.6.

---

## Finding ADV-FIX-P1-MED-001 — No E-QUERY-002 ordering test via public path

**Severity:** MED
**Classification:** test coverage gap / SID-1
**Affected files:** Test suite — no test via public query execution path for E-QUERY-002 ordering behavior
**BC reference:** BC-2.11.004 (pipe mode error ordering)

**Finding:** BC-2.11.004 specifies that when a non-existent column is referenced in a pipe-mode query, E-QUERY-038 (ColumnNotFound) must be returned ahead of E-QUERY-002 (QueryTypeMismatch) even if the column would also fail a type check. No test exercised this ordering invariant via the public query execution path (only internal-unit tests existed). An adversary could not verify the ordering without a public-path RED Gate test.

**Root cause:** The implementer supplied internal unit tests for the ordering behavior but did not add a public-path test that exercises the production dispatch chain end-to-end. SID-1 discipline requires a non-ignored test at the relevant boundary.

**Closure:** CLOSED — test-writer commit 8c8b5628 includes a public-path RED Gate test exercising E-QUERY-038 before E-QUERY-002 ordering (6 RED tests added in the drift module, including this ordering test).

---

## Finding ADV-FIX-P1-MED-002 — |sort, |stats, |fields pipe stages ungated for non-existent column

**Severity:** MED
**Classification:** correctness / scope gap
**Affected files:** `crates/prism-query/src/` (|sort, |stats, |fields dispatch arms)
**BC reference:** BC-2.11.016 v1.5 (gate scope)

**Finding:** The initial E-QUERY-038 gate implementation covered IEQ/IIN/INE operator pre-flight and SqlPipe |where stages. The |sort, |stats, and |fields (formerly |project) pipe stages also consume column references and were not connected to the E-QUERY-038 gate. A user running `SELECT * FROM cyberint_alerts | sort severity_id` with a non-existent column would still receive an opaque internal error.

**Adjudication:** PO review confirmed IN-SCOPE. The SQL GROUP BY/ORDER BY parity argument applies: all column-reference positions in pipe-mode that parallel SQL-mode column references must receive the same structured error. |sort (ORDER BY parity), |stats (GROUP BY parity), and |fields (SELECT parity) are column-reference-consuming positions and must be gated.

**Closure:** CLOSED — implementer commit 741e1506: |sort, |stats, |fields dispatch arms connected to E-QUERY-038 gate (completing twelve-position coverage per BC-2.11.016 v1.6); PO adjudication IN-SCOPE recorded.

---

## Finding ADV-FIX-P1-OBS-001 — T13 audit-script G7 soft-pass; ADR-052 temporal regression check may not cover new IEQ-on-datetime paths

**Severity:** OBS
**Classification:** process-gap / audit-script coverage
**Affected files:** `scripts/t13-preflight-audit.py` (G7 item)

**Finding:** T13 audit-script item G7 verifies that ADR-052 temporal typing has no regression (bare ISO date → E-QUERY-041). The G7 check did not cover IEQ-on-Datetime-column scenarios (e.g., `timestamp IEQ '2026-01-01'`) which could interact with the new E-QUERY-038 gate in unexpected ways. The G7 check passed but may be a soft pass for the expanded gate coverage.

**Adjudication:** Deferred to next audit-script revision. Registered as a coverage note. No behavioral regression was observed in the fix-burst test run. The IEQ operator validates column type via `check_ci_column_types` before reaching the Datetime/Temporal dispatch path — a non-existent column is caught by E-QUERY-038 before type-dispatch. The ordering is correct.

**Status:** DEFERRED — registered in next audit-script revision cycle. No blocker.

---

## Finding ADV-FIX-P1-OBS-002 [process-gap] — BC-2.11.016 lacked AST-completeness invariant; spec-undercount class

**Severity:** OBS (process-gap)
**Classification:** spec-gap / BC structural invariant missing
**Affected files:** `.factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md`
**BC reference:** BC-2.11.016 v1.5

**Finding:** BC-2.11.016 v1.5 enumerated specific AST positions in the gate list but did not include an AST-completeness invariant requiring that ALL AST positions that consume column references must be covered. Without this invariant, future additions to the AST (new pipe stages, new SqlPipe arms) could silently bypass the gate without triggering a spec-contract violation. The absence of the completeness invariant was the structural root cause that allowed ADV-FIX-P1-HIGH-001 and ADV-FIX-P1-HIGH-002 to occur.

**Process-gap captured:** BC authors specifying gated-list contracts must include an AST-completeness invariant (or a forward reference to the AST's authoritative position set) to prevent spec-undercount drift as AST evolves. Codified as Process-Gap Lesson 16 (lessons.md).

**Closure:** CLOSED — BC-2.11.016 v1.6: AST-completeness invariant added to §Invariants. Co-closed with ADV-FIX-P1-HIGH-001.

---

## Fix-Burst Chain Summary

| Commit | Author | Change |
|--------|--------|--------|
| d56e933e | (frozen pass-1 HEAD) | Initial fix-branch; IEQ/IIN/INE check_ci_column_types path wired to E-QUERY-038 (6 of 12 positions) |
| 8c8b5628 | test-writer | 6 RED Gate tests added (drift module): E-QUERY-038 gate on all 6 initial positions + E-QUERY-002 ordering test via public path |
| 741e1506 | implementer | Positions 9-12 connected (SqlPipe |where + |sort + |stats + |fields); E-QUERY-002 type-compat engine-layer fix |
| c2ece301 | implementer | suggested_column b1-parity: pub(crate) ocsf_suggested_string_column reused; b2 pre-empted-path regression on suggested_column fixed |

**Final fix-branch HEAD:** c2ece301 (LOCAL-ONLY; not yet pushed to origin)
**Test count after fix-burst:** 5329/5329 (just check GREEN; 10 tests added in drift module; non-exhaustive 89/89)
**LOCAL 3-CLEAN streak:** 0/3 — NEXT: freeze branch HEAD c2ece301 → LOCAL pass 2 (fresh adversary)

---

## Spec Versions at Fix-Burst Close

| Artifact | Before | After |
|----------|--------|-------|
| BC-2.11.016 | v1.5 | v1.7 (v1.6 twelve-position + AST-completeness invariant; v1.7 |project→|fields keyword fix) |
| BC-2.11.002 | v1.5 | v1.6 |
| BC-2.11.004 | v1.13 | v1.15 (v1.14 intermediate; v1.15 story-writer body-table cell fix) |
| BC-2.11.020 | v1.1 | v1.3 (v1.2 intermediate) |
| error-taxonomy | v2.20 | v2.22 (v2.21 intermediate) |
| BC-INDEX | v7.57 | v7.59 (v7.58 intermediate) |
| S-PRISMQL-CASE-INSENSITIVE-001 | v1.36 | v1.38 |
| S-DEMO-FIDELITY-REMEDIATION-001 | v2.23 | v2.25 |
| S-DEMO-PRISMQL-ONBOARDING-001-B | v2.0 | v2.2 |
| S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 | v1.12 | v1.14 |

---

## Orchestrator Verification Catches (pre-freeze)

Two issues caught by orchestrator verification before freezing c2ece301 as the post-burst HEAD:

1. **BC-2.11.004 body-table cell gap (story-writer):** Story-writer updated the BC-2.11.004 frontmatter pin but missed a body-table cell that still cited the pre-fix version. Caught by orchestrator cross-check; fixed @v1.15.

2. **E-QUERY-002 suggested_column regression on pre-empted path (engine-layer b2 vs b1):** The engine-layer fix at 741e1506 initially dropped `suggested_column` on the pre-empted path (b2 regression vs b1 parity). An IEQ query on a non-existent column that reaches the E-QUERY-002 type-mismatch path should surface `suggested_column` with the nearest matching column. The regression was caught by orchestrator b1/b2 parity verification; fixed @c2ece301 via pub(crate) `ocsf_suggested_string_column` reuse.
