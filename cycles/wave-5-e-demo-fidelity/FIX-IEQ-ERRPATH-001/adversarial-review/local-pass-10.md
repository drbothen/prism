---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [10]
feature_head_at_review: 656bc7a0
fix_burst_head: e5170899
date: 2026-07-09
clean_strict: false
clean_pr_merge: true
finding_counts:
  LOW: 1
  total: 1
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 10 — FIX-IEQ-ERRPATH-001

---

## Pass 10 (frozen 656bc7a0; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 1 (0 CRIT/HIGH/MED + 1 LOW — ADV-FIX-P10-OBS-001; closed same-burst)

**Code HEAD at review:** 656bc7a0 (frozen; D-1620 is_registered() disambiguation at both gate sites; EC-11-041 single-tenant zero-column; 5367/5367 GREEN; non-exhaustive 89/89)

**Fix-burst HEAD:** e5170899 (implementer: last-segment fallback seeding with DERIVED provenance in both branches (b) and (c) of compute_sqlpipe_head_binding; head-seeding only, pipe-stage handling unchanged; 5369/5369 GREEN; non-exhaustive 89/89)

**LOCAL 3-CLEAN(strict) streak after pass-10 fix-burst:** 0/3 (NOT CLEAN(strict); fix-burst dispatched; RESET by @e5170899 push per DRIFT-ORCH-PRLEVEL-PUSH-001)

---

## Finding ADV-FIX-P10-OBS-001 — Join-qualified un-aliased bare-Field head SELECT items not seeded into binding context (FP-001 false E-QUERY-038)

**Severity:** LOW (OBS-class; single well-defined gap in existing seeding logic; no CRIT/HIGH/MED surface)

**Classification:** FP-001 violation — compute_sqlpipe_head_binding branches (b) and (c) extracted join-qualified FieldPaths to None, causing the column to never be seeded into the initial-available binding context; subsequent `| where col` pipe-stage check fired ColumnNotFound with available:[] when `col` was a valid projected column

**Affected files:** `crates/prism-query/src/engine.rs` — `compute_sqlpipe_head_binding` branches (b)/(c); `crates/prism-spec-engine/src/` — 50/50 module (no signature changes)

**BC reference:** BC-2.11.016 v1.17 §LAST-SEGMENT OUTPUT-NAME RULE; EC-11-069; FP-001 invariant

**Finding:** At frozen 656bc7a0, `compute_sqlpipe_head_binding` handled three cases for each head SELECT item:

- Branch (a): `Field { name }` (unqualified bare name) — seeded as-is, correctly
- Branch (b): `Field { qualifier, name }` where qualifier matches the FROM table or FROM alias — previously seeded `name` as RAW; correct behavior
- Branch (c): `Field { qualifier, name }` where qualifier matches neither the FROM table nor any FROM alias (i.e., a join-alias-qualified reference like `j.col` in `SELECT j.col FROM t JOIN other j ON ...`) — extracted to None and returned without seeding

The adversary traced all 14 gate positions and the full EC-11-039..068 edge-case catalog. In a query of the form `SELECT j.col FROM t JOIN other j ON ... | where col`, branch (c) applied to `j.col`, extracting the entire FieldPath to None (neither the known table nor any alias). The result: `col` was never seeded into the initial-available binding context. The pipe-stage `| where col` check then fired `ColumnNotFound` with `available: []` — a false positive per FP-001 (the column IS projected by the head; the gate must not gate it out).

The orchestrator confirmed JOIN grammar support statically: `sql_parser.rs join_clause` is present per BC-2.11.003 (JOIN shapes parse correctly). The test-writer confirmed empirically at RED commit @37435264: both shapes (`SELECT j.col FROM t JOIN other j ON t.id = j.id | where col > 0` and the multi-table alias variant) fired ColumnNotFound with empty available — genuine behavioral deviation, not a test-harness artifact.

**Spec amendment:** Product-owner amended BC-2.11.016 v1.16→v1.17 (LAST-SEGMENT OUTPUT-NAME RULE): un-aliased bare-Field head SELECT items with a qualifier matching neither the FROM table nor FROM alias seed their last path segment as DERIVED into the binding context. This closes the FP-001 gap: downstream pipe-stage references to the projected column resolve via binding context and do not trigger a false E-QUERY-038.

**Fix at @e5170899:** Both branches (b) and (c) of `compute_sqlpipe_head_binding` now apply the LAST-SEGMENT rule when the qualifier is a join alias (matches neither FROM table nor FROM alias): they seed the last path segment with DERIVED provenance. Head-seeding only; pipe-stage handling is unchanged. The 50/50 module compiled and all tests passed GREEN on first run.

**Empirical confirmation by test-writer:** RED test authored at @37435264: two query shapes (`SELECT j.col FROM t JOIN other j ON ...` + variant) fired ColumnNotFound with empty available — confirmed genuine behavioral deviation. GREEN @e5170899: both shapes now resolve the downstream `| where col` reference via binding context (DERIVED provenance; no E-QUERY-038 false positive).

**Routed:** product-owner (BC-2.11.016 v1.17 LAST-SEGMENT OUTPUT-NAME RULE + EC-11-069 + sibling spec sync); test-writer (RED @37435264); implementer (GREEN @e5170899 last-segment fallback seeding); story-writer (4-story pin round, BC-INDEX v7.68→v7.69)

**Closure:** CLOSED — RED @37435264 (both join-qualified shapes fired ColumnNotFound with empty available); GREEN @e5170899 (last-segment seeding DERIVED provenance; 5369/5369 GREEN; non-exhaustive 89/89).

---

## Pass Notes

**All 14 positions + EC-11-039..068 traced:** PASS — adversary performed exhaustive trace of all 14 gate positions and the full EC-11-039..068 edge-case catalog at frozen 656bc7a0. ADV-FIX-P10-OBS-001 is the only deviation found across the entire catalog.

**SAP-1 (Structured Event Catalog):** PASS — no new `event_type =` sites introduced in this fix-burst; no new structured event catalog rows required. Existing `column_not_found.rejected` catalog entries in BC-2.16.002 v2.07 cover the emission path touched by this fix.

**POL-24 (byte-parity EC-body):** PASS — EC-11-069 added to BC-2.11.016 v1.17 with full field schema, audit role, and recurrence policy matching the EC-11-039..068 body format. Byte-parity with canonical template verified.

**bc_2_11_019_n1b fixture interaction verified:** PASS — the fix-burst HEAD e5170899 does not disturb the bc_2_11_019_n1b_test.rs fixture population added in D-1620 (real columns). The DERIVED provenance seeding introduced in compute_sqlpipe_head_binding is head-seeding only; the existing column-availability gate sites remain unchanged in their behavior for the n1b fixture scenarios.

**TableRegistry `#[non_exhaustive]` preserved:** PASS — no new public types introduced; existing `#[non_exhaustive]` annotations on `TableRegistry` and related structs remain intact. Non-exhaustive EXPECTED=89 gate unchanged.

**No `unwrap()`/`println!` introduced:** PASS — fix-burst diff reviewed; no unwrap() calls in non-test code; no println! calls added.

**Story pins:** BC-2.11.016 v1.17 propagated to all 4 carrier stories in same-burst story pin round. BC-INDEX v7.69. STORY-INDEX v2.641.

**Novelty:** LOW — LAST-SEGMENT OUTPUT-NAME RULE fills a natural gap in the existing three-branch seeding logic; join-alias-qualified un-aliased FieldPaths were simply unhandled. The fix is a single-branch extension following the established pattern for branches (a) and (b). No new BC gate architecture or provenance-state machine changes required. Convergence trajectory: 6→3→3→2→1→1 (per-pass total finding count; steady decay consistent with genuine convergence).
