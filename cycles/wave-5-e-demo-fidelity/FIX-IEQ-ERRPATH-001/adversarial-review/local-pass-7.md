---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [7]
feature_head_at_review: d983613b
fix_burst_head: eafe10c2
date: 2026-07-09
clean_strict: false
clean_pr_merge: true
finding_counts:
  MED: 1
  OBS: 2
  total: 3
code_behavior_defects: 3
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 7 — FIX-IEQ-ERRPATH-001

---

## Pass 7 (frozen d983613b; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 3 (1 MED + 2 OBS), 3 code-behavior defects

**Code HEAD at review:** d983613b (frozen; D-1617 implementer MIXED-STAR branch (c) Option A precise union; 34/34 module GREEN; just check 5353/5353 GREEN; non-exhaustive 89/89)

**Fix-burst HEAD:** eafe10c2 (implementer: provenance-aware binding context + table_alias threading + FIELDS transitions; 5360/5360 GREEN; non-exhaustive 89/89)

**LOCAL 3-CLEAN(strict) streak after pass-7:** 0/3 (NOT CLEAN(strict); fix-burst dispatched; RESET by @eafe10c2 push per DRIFT-ORCH-PRLEVEL-PUSH-001)

---

## Finding ADV-FIX-P7-MED-001 — False E-QUERY-002 on shadow aliases in PipeStage::Where type-compat check

**Severity:** MED (HIGH-confidence; orchestrator-CONFIRMED)

**Classification:** code-behavior defect — PipeStage::Where type-compat gate queried RAW schema while binding context used derived (shadow) column provenance

**Affected files:** `crates/prism-query/src/engine.rs` — `check_pipe_stage_columns` PipeStage::Where arm

**BC reference:** BC-2.11.016 v1.14 §Derived-Column Binding Rule (FP-001 invariant); BC-2.11.017 v1.3 §E-QUERY-002 interaction

**Finding:** When a SqlPipe query produces a derived column via `SELECT expr AS alias` in the SQL head (e.g., `SELECT count(*) AS severity FROM crowdstrike_alerts | where severity > 5`), the alias `severity` is seeded into the binding context by the HEAD-PROJECTION BINDING RULE (EC-11-065 class). However, the `PipeStage::Where` arm in `check_pipe_stage_columns` performed its E-QUERY-002 type-compat check by looking up the column type from the RAW source schema rather than from the binding-context entry. Since `severity` is a DERIVED column (it has no corresponding raw-schema entry with the type of the alias expression), the raw-schema lookup either fails or returns the wrong type (the original raw `severity` column's string type, not the aggregate integer result), causing a false E-QUERY-002 `QueryTypeMismatch` where none should fire.

The FP-001 invariant (EC-11-048: anonymous unaliased non-Field items → suspended fail-open) partially addressed this for anonymous expressions, but the per-name provenance distinction was not generalized. DERIVED names in the binding context (those seeded from aliased expressions) have an expression type unknown at plan-time and MUST be treated as fail-open. RAW names (bare fields, schema columns) retain their known type and SHOULD be type-checked.

This is a sibling-gate consistency gap: the binding-context provenance (RAW vs DERIVED) was computed by EC-11-059..061 and EC-11-062..064 but not threaded through to the type-compat check arm, which still performed a raw-schema lookup.

**Routed:** product-owner (BC-2.11.016 v1.15 provenance rule + BC-2.11.017 v1.4 cross-reference invariant clause) + test-writer (7 tests at @7f558b59: 5 RED + 2 green-locks) + implementer (GREEN @eafe10c2)

**Closure:** CLOSED — BC-2.11.016 v1.14→v1.15: SIBLING-GATE CONSISTENCY per-name RAW/DERIVED provenance rule; EC-11-065..068; FROM-ALIAS RESOLUTION; FIELDS TRANSITION rule; FP-001 generalized to ALL binding-context gates. BC-2.11.017 v1.3→v1.4: E-QUERY-002 cross-reference invariant clause. Implementer @eafe10c2: provenance-aware binding context threading; table_alias map wired into check_pipe_stage_columns; FIELDS transition updates binding context; 5360/5360 GREEN; non-exhaustive 89/89.

---

## Finding ADV-FIX-P7-OBS-001 — from_alias never threaded into check_pipe_stage_columns: aliased-qualifier refs skipped gate (fail-negative)

**Severity:** OBS (orchestrator-CONFIRMED)

**Classification:** code-behavior defect — fail-negative: aliased-qualifier FieldPaths (alias.column form) bypassed the column-not-found gate entirely

**Affected files:** `crates/prism-query/src/engine.rs` — `check_pipe_stage_columns` call sites

**BC reference:** BC-2.11.016 v1.14 §Gate Position table; FP-001 invariant

**Finding:** When a pipe-stage column reference uses the alias-qualified form `alias.column` (e.g., `FROM alerts AS a | where a.severity > 5`), the `from_alias` map (mapping alias→table name) was never threaded into `check_pipe_stage_columns`. As a result, the alias-resolution step before the binding-context lookup was absent, and the qualified reference silently bypassed the gate — the gate neither checked nor rejected `a.severity` even if `severity` was not in scope. This is a fail-negative: invalid aliased-qualifier references would not produce E-QUERY-038.

**Closure:** CLOSED — EC-11-065 closure includes from_alias resolution: aliased-qualifier FieldPaths resolved via table_alias map before binding-context lookup; implementer @eafe10c2 wired `table_alias` map (from `from_alias` source) into `check_pipe_stage_columns` at all call sites; gap confirmed closed via EC-11-068 test vector.

---

## Finding ADV-FIX-P7-OBS-002 — `| fields` did not transition binding context despite apply_fields genuinely restricting projection (fail-negative)

**Severity:** OBS (orchestrator-CONFIRMED)

**Classification:** code-behavior defect — fail-negative: post-`| fields` pipe stages saw the pre-fields binding context, missing columns removed by `| fields`

**Affected files:** `crates/prism-query/src/engine.rs` — `check_pipe_stage_columns` PipeStage::Fields arm; `crates/prism-query/src/pipe_sql_emitter.rs` — `apply_fields` emitter

**BC reference:** BC-2.11.016 v1.14 Gate Position 12; EC-11-052 (fields refs validated against current binding); BC-2.11.020 v1.8 §Pipe Stage Composition

**Finding:** The `apply_fields` emitter in `pipe_sql_emitter.rs` genuinely restricts the output projection: columns not selected by `| fields` are absent from the emitted SQL. However, `check_pipe_stage_columns` did not update the binding context after processing a `PipeStage::Fields` stage — subsequent pipe stages (e.g., a `| where` after `| fields`) still saw the full pre-fields binding context, meaning they would not fire E-QUERY-038 on columns removed by `| fields`. This is a fail-negative gap: a query like `FROM t | fields col_a | where col_b > 5` should fire E-QUERY-038 at the `| where col_b > 5` stage (since col_b was removed by `| fields`), but the gate would silently pass.

The FIELDS TRANSITION rule was missing: `| fields` must transition the binding context to the selected column subset so subsequent stages see only the surviving columns.

**Closure:** CLOSED — BC-2.11.016 v1.14→v1.15 FIELDS TRANSITION rule; EC-11-066/067/068 test vectors; BC-2.11.020 v1.8→v1.9 propagation. Implementer @eafe10c2: `PipeStage::Fields` arm now updates the binding context to the post-fields column subset before processing subsequent stages; 5360/5360 GREEN; non-exhaustive 89/89.

---

## Pass Notes

**SAP-1 (Structured Event Catalog):** PASS — 3 `column_not_found.rejected` emission sites verified cataloged in BC-2.16.002 v2.07 Canonical Structured Event Catalog. No new `event_type =` sites introduced in the fix-burst.

**POL-24 (byte-verbatim EC-body):** PASS — EC-11-065..068 test vectors verified against implementer commit eafe10c2; byte-verbatim match on error message templates.

**TD-VSDD-060 (sibling-site sweep on compute_sqlpipe_head_binding):** PASS — single caller confirmed; no additional call sites introduced.

**Forbidden patterns sweep:** PASS — no `unwrap()`/`expect()` in changed paths; no `println!`; no new pub types requiring `#[non_exhaustive]`.

**Story pins:** PASS — 4 carrier stories pinned to current BC versions; no stale pins remaining.

**Novelty:** MEDIUM — the sibling-gate RAW/DERIVED provenance class is genuinely novel versus passes P1–P6. Prior passes addressed missing gate positions, grammar fixes, union-path, agg-arg scope, and MIXED-STAR branch. This pass surfaces that the type-compat check (a different operation from column-existence) also needed provenance-awareness. The alias-threading and FIELDS-transition are companion gaps in the same provenance class.
