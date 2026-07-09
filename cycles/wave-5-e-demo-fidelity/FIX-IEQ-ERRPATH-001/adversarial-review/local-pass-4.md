---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [4]
feature_head_at_review: 2206f90a
fix_burst_head: ffcdc5fe
date: 2026-07-08
clean_strict: false
clean_pr_merge: true
finding_counts:
  MED: 2
  OBS: 1
  total: 3
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 4 — FIX-IEQ-ERRPATH-001

---

## Pass 4 (frozen 2206f90a; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES
**Findings:** 3 (2 MED + 1 OBS), zero code-behavior defects
**Code HEAD at review:** 2206f90a (frozen; post-pass-3 fix-burst HEAD; IIN plan-time arm added)
**Fix-burst HEAD:** ffcdc5fe (chain: 2206f90a → 6ef8eaf1 implementer doc-refresh → e54bfa77 test-writer RED EC-11-058 → ffcdc5fe implementer agg-arg walk)
**LOCAL 3-CLEAN(strict) streak after pass-4:** 0/3 (NOT CLEAN(strict); fix-burst dispatched; RESET by @ffcdc5fe push per DRIFT-ORCH-PRLEVEL-PUSH-001)

---

## Finding ADV-FIX-P4-MED-001 — Stale doc-comments claiming registry-not-plumbed (dangerous-stale class)

**Severity:** MED
**Classification:** doc-accuracy — dangerous-stale (misleads future implementers about production state; TD-VSDD-091 class)
**Affected files:** `crates/prism-query/src/` (inline doc comments in `check_query_column_availability` and `check_pipe_stage_columns`)
**BC reference:** TD-VSDD-091 (narrative spec content must cite behavior anchors NOT volatile line numbers; applies equally to production code doc-comments)

**Finding:** After the pass-3 fix-burst (implementer @2206f90a) wired the `infusion_registry` Option parameter through `check_query_column_availability` and `check_pipe_stage_columns`, several inline doc-comments in those functions still claimed the infusion registry was "not yet plumbed" or described the Enrich arm as using "always-suspend (registry not wired)". These descriptions were accurate at the start of the FIX-IEQ-ERRPATH-001 cascade but became false after the union-path closure (D-1612, implementer @ee72660d). A future implementer reading these comments would incorrectly believe the registry plumbing was absent and might duplicate or re-introduce the wiring.

This is the "dangerous-stale" subclass of TD-VSDD-091: not merely a stale line-number citation, but a comment asserting a false behavioral state ("not plumbed") about production code that IS plumbed. No code-behavior gap exists — the implementation is correct. The risk is maintainer confusion in future fix-bursts or feature extensions.

Additionally, some behavior-anchoring comments still cited BC version numbers inline (e.g., "per BC-2.11.016 v1.7") in a volatile-pin style, creating future staleness as the BC version advances. TD-VSDD-091 discipline requires citing BC IDs + behavioral anchors without version pins in production code comments.

**Closure:** CLOSED — implementer @6ef8eaf1: comprehensive doc-comment refresh across `check_query_column_availability` and `check_pipe_stage_columns` — (a) stale "not plumbed" / "always-suspend (registry not wired)" claims removed; (b) behavior comments updated to accurately describe the post-D-1612 wired state (registry Some → union-or-defensive-suspend; registry None → always-suspend); (c) BC references reformatted to cite BC IDs + behavioral anchors WITHOUT volatile version pins per TD-VSDD-091 (e.g., "per BC-2.11.016 DERIVED-COLUMN BINDING RULE" instead of "per BC-2.11.016 v1.7 §Preconditions.2"). No spec change, no code-behavior change.

---

## Finding ADV-FIX-P4-MED-002 — BC-2.11.004 and BC-2.11.020 pin BC-2.11.016 at v1.9 — multi-hop stale (spec-side)

**Severity:** MED
**Classification:** spec-side multi-hop stale pin (BC-2.11.016 is now at v1.11 post-pass-3; both companions still cite v1.9)
**Affected files:** BC-2.11.004 §Error Cases (E-QUERY-038 row BC anchor), BC-2.11.020 §Error Cases (E-QUERY-038 row BC anchor)
**BC reference:** POL-25 (multi-cite propagation — sibling BCs that reference the same canonical contract must update their citations when the canonical version advances)

**Finding:** BC-2.11.004 (Pipe Mode Parsing) and BC-2.11.020 (SqlPipe Composition) each contain an E-QUERY-038 §Error Cases row that cites BC-2.11.016 as the canonical E-QUERY-038 gate contract. After the pass-3 fix-burst, BC-2.11.016 advanced from v1.9 to v1.11 (v1.10 union-path closure + v1.11 payload dichotomy). The companion BCs still carry their v1.9 BC anchor reference, a multi-hop stale lag of 2 versions. The v1.9 anchor predates the DERIVED-COLUMN BINDING RULE (v1.8), the enrich-union path (v1.9→v1.10), and the payload dichotomy clarification (v1.11). A reader of BC-2.11.004 or BC-2.11.020 would follow a stale citation and miss the current authoritative semantics.

This is a spec-side issue only — the implementation is correct per BC-2.11.016's current content. The risk is misleading BC readers.

**Closure:** CLOSED — product-owner: BC-2.11.004 v1.17→v1.18 — E-QUERY-038 §Error Cases BC anchor updated v1.9→v1.12 (direct-to-current, skipping intermediate hops); position-11 description expanded to include aggregate function argument field paths (see ADV-FIX-P4-OBS-001 closure below; since both items are in the same BC section they were addressed in the same amendment). BC-2.11.020 v1.5→v1.6 — same E-QUERY-038 BC anchor update v1.9→v1.12 + position-11 expansion. error-taxonomy v2.25→v2.26: E-QUERY-038 gate-scope description and BC anchor updated to v1.12. BC-INDEX v7.63→v7.64 (entries prepended by PO).

---

## Finding ADV-FIX-P4-OBS-001 — Position-11 agg-arg field paths ungated; HAVING-parity gap (spec + implementation)

**Severity:** OBS
**Classification:** plan-time gate coverage gap (implementation symmetry)
**Affected files:** `crates/prism-query/src/` (agg-arg walk in `check_pipe_stage_columns`, position-11 handling); BC-2.11.016 §Preconditions.2 Gate-positions table
**BC reference:** BC-2.11.016 position-11 (`| stats ... by` grouping refs), EC-11-046 (HAVING gate mandate)

**Finding:** Position-11 in the E-QUERY-038 gate covers `| stats ... by` grouping field references. The existing implementation gated the `by` field keys (the group-by column references). However, within a `| stats` clause, aggregate function arguments (e.g., `| stats count(severity_id) by severity` — the `severity_id` argument to `count()`) also reference schema-bound column paths and were not checked by the gate. If a user writes `| stats count(nonexistent_field) by severity`, the gate would NOT produce E-QUERY-038 on `nonexistent_field`; instead, the query would fail with an opaque downstream error.

**PO adjudication (IN-SCOPE):** BC-2.11.016 v1.11 §Preconditions.2 already stated "for `| stats`: by-field grouping keys AND aggregate function arguments, where the argument is a direct field reference (FieldPath), are checked against the available binding set." The BC's own normative prose already mandated this behavior; the implementation had not fully honored it. This is therefore not a spec amendment — it is an implementation gap against existing spec. The HAVING gate (position 6, EC-11-046) established the HAVING parity principle: gate arms cover all positions where schema-bound field paths appear. Aggregate argument field paths are schema-bound at position 11. Both parity argument and BC normative prose require the fix.

**Closure:** CLOSED — BC-2.11.016 v1.11→v1.12: position-11 scope explicitly enumerated in Gate-positions table to cover both (a) `by` grouping field keys AND (b) aggregate function argument FieldPaths (CountField/Sum/Avg/Min/Max/DistinctCount/Percentile carriers); EC-11-058 (new edge case: agg-arg on non-existent field → E-QUERY-038 before stats binding-context replacement); pipe-stats-agg-arg test vector added to Canonical Test Vectors; HAVING parity rationale noted inline. test-writer @e54bfa77: RED EC-11-058 test for agg-arg on non-existent column (both CountField and Sum carriers; GREEN-lock confirms existing valid-agg-arg pass-through). implementer @ffcdc5fe: agg-arg walk added to `check_pipe_stage_columns` position-11 handling — iterates CountField/Sum/Avg/Min/Max/DistinctCount/Percentile argument FieldPaths and calls `check_column_against_available_set` for each; FP-001 wildcard passthrough (agg-arg `*` in `count(*)` style excluded); 26/26 module tests GREEN; just check 5345/5345 GREEN; non-exhaustive 89/89.

---

## Fix-burst Summary

**Chain:** 2206f90a (frozen pass-4 HEAD) → 6ef8eaf1 (implementer: doc-comment refresh; stale registry-not-plumbed claims removed; TD-VSDD-091 improvement) → e54bfa77 (test-writer: RED EC-11-058 agg-arg on non-existent column; GREEN-lock for valid agg-arg pass-through) → ffcdc5fe (implementer: agg-arg walk CountField/Sum/Avg/Min/Max/DistinctCount/Percentile; FP-001 wildcard; 26/26 module tests GREEN; just check 5345/5345 GREEN; non-exhaustive 89/89)

**Spec closures:**
- BC-2.11.016 v1.11→v1.12 (position-11 agg-arg expansion + EC-11-058; Gate-positions table clarified; pipe-stats-agg-arg test vector)
- BC-2.11.004 v1.17→v1.18 (E-QUERY-038 BC anchor v1.9→v1.12; position-11 description expanded; POL-25 companion)
- BC-2.11.020 v1.5→v1.6 (E-QUERY-038 BC anchor v1.9→v1.12; position-11 description expanded; POL-25 companion)
- error-taxonomy v2.25→v2.26 (E-QUERY-038 gate-scope + BC anchor updated to v1.12; POL-25 sweep)
- BC-INDEX v7.63→v7.64

**FULL-CORPUS RECONCILING pin round (story-writer — ALL versioned pins reconciled corpus-wide):**

Cascade carriers (BC-2.11.016 v1.12 + taxonomy v2.26 propagation):
- S-DEMO-FIDELITY-REMEDIATION-001 v2.29→v2.30: BC-2.11.016 v1.12 + taxonomy v2.26
- S-DEMO-PRISMQL-ONBOARDING-001-B v2.6→v2.7: BC-2.11.016 v1.12 + taxonomy v2.26
- S-PRISMQL-CASE-INSENSITIVE-001 v1.41→v1.42: BC-2.11.016 v1.12 + taxonomy v2.26
- S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.16→v1.17: BC-2.11.016 v1.12 + taxonomy v2.26
- S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 v1.11→v1.12: taxonomy v2.26 sites
- S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 v1.15→v1.16: taxonomy v2.26 sites

Corpus-wide multi-hop debt (OUTSIDE cascade perimeter; found by FULL-CORPUS sweep):
- S-DEMO-PRISMQL-ONBOARDING-001 (superseded) v1.1→v1.2: BC-2.11.016 v1.0 pin (~12 versions stale; superseded story retained for audit trail)
- S-PLUGIN-PREREQ-E v1.54→v1.55: taxonomy v1.38→v2.26 (~50 versions stale; outside any recent cascade)
- S-5.02 v1.13→v1.14: taxonomy v1.84→v2.26
- S-WATCHDOG-WIRING-001 v1.2→v1.3: taxonomy v1.70→v2.26
- S-DEMO-DTU-LIVE-SCENARIO-001-B v2.16→v2.17: taxonomy v1.78→v2.26
- S-DEMO-ENRICHMENT-PIVOT-002 v1.5→v1.6: taxonomy v1.88→v2.26

Zero stale live pins verified corpus-wide after FULL-CORPUS sweep.

**Lesson L20 addendum appended:** corpus-scale evidence — FULL-CORPUS RECONCILING mode found pins up to ~50 versions stale in 6 stories untouched by any recent cascade (S-PLUGIN-PREREQ-E at taxonomy v1.38 vs current v2.26); recommend wave-gate protocol adds a periodic corpus-wide pin-reconciliation step.

**LOCAL 3-CLEAN(strict) streak after pass-4:** 0/3 (NOT CLEAN(strict) per BC-5.39.001). **NEXT:** freeze ffcdc5fe → LOCAL pass 5. If passes 5/6/7 CLEAN(strict) → LOCAL CONVERGED → push → PR → PR-LEVEL cascade.
