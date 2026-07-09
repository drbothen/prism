---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [3]
feature_head_at_review: ee72660d
fix_burst_head: 2206f90a
date: 2026-07-08
clean_strict: false
clean_pr_merge: true
finding_counts:
  MED: 1
  LOW: 2
  total: 3
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 3 — FIX-IEQ-ERRPATH-001

---

## Pass 3 (frozen ee72660d; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES
**Findings:** 3 (1 MED + 2 LOW)
**Code HEAD at review:** ee72660d (frozen; post-pass-2 fix-burst HEAD; union-path closure implemented)
**Fix-burst HEAD:** 2206f90a (chain: ee72660d → f17e584e test-writer IIN plan-time RED → 2206f90a implementer IIN arm in collect_predicate_type_pairs_inner)
**LOCAL 3-CLEAN(strict) streak after pass-3:** 0/3 (NOT CLEAN(strict); fix-burst dispatched; RESET by @2206f90a push per DRIFT-ORCH-PRLEVEL-PUSH-001)

---

## FP-001 Hunt — CLEARED

**Result:** ZERO false-positive query shapes confirmed. The adversary exercised all adversarial traces: chained enrich queries, stats/enrich interleavings, suspension propagation, both tenancy modes (single-tenant + multi-tenant), and canonical BC-2.06.019 NVD/ThreatIntel pivot patterns. All traces produce correct results or fail-open per the DERIVED-COLUMN BINDING RULE. Design core verified. FP-001 invariant ("gate MUST NOT produce false E-QUERY-038 errors for valid column references") holds on ee72660d.

---

## Finding ADV-FIX-P3-MED-001 — BC-2.16.002 catalog row lists 2 emission sites for `column_not_found.rejected`; `check_column_against_available_set` is a third (SAP-1 / PG-LP11-001)

**Severity:** MED
**Classification:** tracing emission catalog completeness (SAP-1 standing adversary probe; PG-LP11-001)
**Affected files:** `crates/prism-query/src/` (`check_column_against_available_set`)
**BC reference:** BC-2.16.002 §Postconditions Canonical Structured Event Catalog (row `column_not_found.rejected`)

**Finding:** The `column_not_found.rejected` event type in BC-2.16.002's Canonical Structured Event Catalog lists two emission sites in its emission-site column (the two pre-cascade sites). The FIX-IEQ-ERRPATH-001 cascade added `check_column_against_available_set` as a third emission site for this event type — this new helper function emits `column_not_found.rejected` at positions 1-14 of the gate. BC-2.16.002 §Postconditions was not updated to enumerate this third emission site. Per SAP-1 (standing adversary probe: every `event_type =` site in `crates/` must have a catalog row with full field schema, audit role, and recurrence policy) and PG-LP11-001 (structured event catalog discipline), every emission site for a named event_type must appear in the catalog row. This is a P1 finding under SAP-1 / PG-LP11-001.

Additionally, the existing catalog row's field-schema description for `available_count` did not clarify that the count reflects the binding view (post-stage-mutation column set) rather than the raw source schema. After a stats stage, `available_count` reflects the stats output aliases count, not the source table column count. This semantic gap is addressable in the same catalog-row update.

**Closure:** CLOSED — BC-2.16.002 v2.06→v2.07: `column_not_found.rejected` catalog row updated to enumerate all three emission sites (`check_column_against_available_set` added as third site in addition to the two prior sites); binding-view `available_count` semantics documented in the field-schema (reflects the columns available AFTER stage-binding updates, not the raw source schema); no new event_type, catalog count unchanged 91; POL-30 Fork B (description extension only; no structural catalog change). story-writer propagated BC-2.16.002 v2.07 pin to S-PRISMQL-CASE-INSENSITIVE-001 v1.40→v1.41 (AC-018 live site). BC-INDEX v7.62→v7.63.

---

## Finding ADV-FIX-P3-LOW-001 — `IIN` operator absent from plan-time type-pair walker (`IEQ`/`INE` covered; silent failure path for invalid-column IIN queries)

**Severity:** LOW
**Classification:** plan-time gate coverage gap
**Affected files:** `crates/prism-query/src/` (`collect_predicate_type_pairs_inner`)
**BC reference:** BC-2.11.016 (plan-time gate coverage; all IEQ/IIN/INE operators should be symmetrically gated)

**Finding:** The plan-time type-pair walker (`collect_predicate_type_pairs_inner`) dispatched `IEQ` and `INE` predicates to the column existence check in `check_column_against_available_set`, but was missing an arm for `IIN` (in-list case-insensitive). An `IIN` predicate on a non-existent column would fall through the match without invoking the gate, producing a silent failure path instead of the pedagogical E-QUERY-038 error. This is an asymmetry in the IEQ/IIN/INE triad — IEQ and INE were gated, IIN was not. For a user who types e.g. `severity_id IIN ('critical', 'high')` against a table that has `severity` but not `severity_id`, the expected behavior is E-QUERY-038 with available_columns hint; the actual behavior was a silent failure (likely opaque downstream error).

Orchestrator adjudicated fix-in-scope (symmetry requirement for the IEQ/IIN/INE triad; no spec amendment needed, just closing the implementation gap).

**Closure:** CLOSED — test-writer @f17e584e: RED IIN plan-time vector added (shared-helper finding: one test covers both filter-mode and pipe-mode IIN on non-existent column; yields E-QUERY-038 with available_columns in both modes). implementer @2206f90a: IIN arm added to `collect_predicate_type_pairs_inner`; defensive negated-IIN posture noted (IIN with valid column = pass-through; only invalid column triggers gate); 24/24 module tests GREEN; just check 5343/5343 GREEN; non-exhaustive 89/89.

---

## Finding ADV-FIX-P3-LOW-002 — `available_columns` payload post-stats reflects binding-view; BC-2.11.016 wording implied raw-schema semantics (positions-1-7 vs 8-14 dichotomy undocumented)

**Severity:** LOW
**Classification:** spec/implementation alignment gap (wording update)
**Affected files:** BC-2.11.016 §Postconditions (wording of `available_columns` payload semantics)
**BC reference:** BC-2.11.016 v1.10

**Finding:** BC-2.11.016 v1.10 described the `available_columns` field in the E-QUERY-038 error payload as listing "the table's available columns". This description is accurate for positions 1-7 (SQL/filter mode, where the gate checks against the raw source schema). However, for positions 8-14 (pipe mode), the available set has been mutated by preceding stages: after a `| stats` stage, the available set is replaced by stats output aliases; after an `| enrich` stage (registry-wired), the set is expanded by enrich output columns. The E-QUERY-038 payload at positions 8-14 correctly reflects the current binding view (the columns actually available at that pipe position), but BC-2.11.016's "table's available columns" wording implied raw-schema semantics throughout. A future implementer reading BC-2.11.016 would be confused by the wording after a stats stage (where `available_columns` in the error would list only `{count, severity}` stats aliases, not the original 15 source columns).

This is not a code defect — the implementation is correct and BC-sanctioned per the DERIVED-COLUMN BINDING RULE added in v1.8. The gap is in the BC's postcondition wording.

**Closure:** CLOSED — BC-2.11.016 v1.10→v1.11: `available_columns` payload semantics clarified with a positions-1-7 vs positions-8-14 dichotomy: positions 1-7 (SQL/filter mode) = raw source schema columns available in the table; positions 8-14 (pipe mode) = current binding view after all preceding stage mutations (enrich: source ∪ enrich-output; stats: output aliases only; other stages: source schema). story-writer propagated BC-2.11.016 v1.11 pin to S-DEMO-FIDELITY-REMEDIATION-001 v2.28→v2.29 (6 sites) and S-DEMO-PRISMQL-ONBOARDING-001-B v2.5→v2.6 (2 sites).

---

## Fix-burst Summary

**Chain:** ee72660d (frozen pass-3 HEAD) → f17e584e (test-writer: RED IIN plan-time vector) → 2206f90a (implementer: IIN arm in `collect_predicate_type_pairs_inner`; 24/24 module tests GREEN; just check 5343/5343 GREEN; non-exhaustive 89/89)

**Spec closures:**
- BC-2.16.002 v2.06→v2.07 (three-site `column_not_found.rejected` catalog row + binding-view available_count semantics; no new event_type, catalog count unchanged 91; POL-30 Fork B)
- BC-2.11.016 v1.10→v1.11 (payload positions-1-7 vs 8-14 dichotomy documented; raw-schema vs binding-view disambiguation)
- error-taxonomy v2.24→v2.25 (POL-25 sweep incl. 2 opportunistic stale-claim fixes: gate-ordering "twelve"→"fourteen"; BC anchor v1.9→v1.11)
- BC-INDEX v7.62→v7.63

**RECONCILING pin round (story-writer — ALL versioned pins reconciled to current, not single-hop):**
- S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 v1.10→v1.11: 4 multi-hop stale taxonomy sites (taxonomy v2.14→v2.25; 11 versions stale — OUTSIDE prior cascade perimeter)
- S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 v1.14→v1.15: 5 multi-hop stale taxonomy sites (taxonomy v2.17→v2.25; 8 versions stale — OUTSIDE prior cascade perimeter)
- S-PRISMQL-CASE-INSENSITIVE-001 v1.40→v1.41: AC-018 BC-2.16.002 v2.07 + AC-022 taxonomy v2.25 (cascade carrier)
- S-DEMO-FIDELITY-REMEDIATION-001 v2.28→v2.29: 6 sites BC-2.11.016 v1.11 (cascade carrier)
- S-DEMO-PRISMQL-ONBOARDING-001-B v2.5→v2.6: 2 sites BC-2.11.016 v1.11 (cascade carrier)
- Zero stale live pins verified all classes after RECONCILING sweep.

**Lesson L20 appended:** single-hop pin sweeps (grep only the immediately-superseded version) systematically miss multi-hop stale pins in stories outside the active cascade; pin propagation must periodically run RECONCILING mode (grep all vN.NN pins per artifact, reconcile live sites to current). Evidence: 9 multi-hop sites up to 11 taxonomy versions stale found 2026-07-08.

**LOCAL 3-CLEAN(strict) streak after pass-3:** 0/3 (NOT CLEAN(strict) per BC-5.39.001). **NEXT:** freeze 2206f90a → LOCAL pass 4. If passes 4/5/6 CLEAN(strict) → LOCAL CONVERGED → push → PR → PR-LEVEL cascade.
