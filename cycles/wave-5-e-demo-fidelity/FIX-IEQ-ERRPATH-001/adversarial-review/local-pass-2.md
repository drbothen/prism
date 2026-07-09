---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [2]
feature_head_at_review: c2ece301
fix_burst_head: 21f20cb6
date: 2026-07-08
clean_strict: false
clean_pr_merge: false
finding_counts:
  CRIT: 2
  HIGH: 3
  OBS: 2
  total: 7
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 2 — FIX-IEQ-ERRPATH-001

---

## Pass 2 (frozen c2ece301; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO
**Findings:** 7 (2 CRIT + 3 HIGH + 2 OBS)
**Code HEAD at review:** c2ece301 (frozen; post-pass-1 fix-burst HEAD)
**Fix-burst HEAD:** 21f20cb6 (chain: c2ece301 → 4b5dc7f1 test-writer 10 RED → 21f20cb6 implementer binding-context walk + Stats REPLACE + enrich-input pos-13 + ALWAYS-SUSPEND enrich-output + dedup pos-14 + single-tenant table_registry fallback + OBS-001 comment scrub)
**LOCAL 3-CLEAN(strict) streak after pass-2:** 0/3 (NOT CLEAN; fix-burst dispatched; RESET by @21f20cb6 push)

---

## Finding ADV-FIX-P2-CRIT-001 — Enrich-derived columns false-positived by E-QUERY-038 gate at positions 8/10; broke canonical BC-2.06.019 NVD/ThreatIntel pivots

**Severity:** CRIT
**Classification:** false-positive gate / correctness regression
**Affected files:** `crates/prism-query/src/` (E-QUERY-038 gate at pipe-mode positions 8 and 10)
**BC reference:** BC-2.11.016 v1.7 (gate coverage), BC-2.06.019 (NVD/ThreatIntel pivot queries)

**Finding:** The twelve-position E-QUERY-038 gate added in pass-1's fix-burst (c2ece301) checked ALL column references at the listed positions against the source table's schema. Enrich pipe stages (`| enrich threat_score(iocs_value)`) output derived columns (e.g., `threat_score`, `cvss_base_score`) that are not present in the source table schema and thus do not exist in the pre-enrich column set. Downstream references to these derived columns in `| where` (position 8) and `| sort` (position 10) were incorrectly flagged as E-QUERY-038 ColumnNotFound because the gate's column existence check operated on the source-table schema only, without tracking which columns were made available by preceding enrich stages.

**Real-world impact:** The canonical BC-2.06.019 NVD/ThreatIntel pivot queries (e.g., `... | enrich threat_score(iocs_value) | where threat_score > 5 | sort threat_score`) were broken. These queries are the core demo-critical enrichment scenario. Any query with a downstream column reference to an enrich-output column would receive a false E-QUERY-038 error.

**Root cause:** BC-2.11.016 v1.7 specified the gate positions but did not define a derived-column binding context (a running set of columns made available by preceding stages). The implementation was correct to check column existence but lacked the mechanism to extend the "available" set as enrich stages produced output columns.

**Closure:** CLOSED — BC-2.11.016 v1.7→v1.8: DERIVED-COLUMN BINDING RULE added to §Preconditions (running `{available, suspended}` context tracking; Enrich union/fail-open semantics; Stats REPLACE semantics; FP-001 never-false-positive invariant). Implementer @21f20cb6: binding-context walk in `check_pipe_stage_columns` — each stage type updates the available-column set before checking downstream positions.

---

## Finding ADV-FIX-P2-CRIT-002 — Stats aggregate aliases false-positived by gate; broke count-and-sort pattern

**Severity:** CRIT
**Classification:** false-positive gate / correctness regression
**Affected files:** `crates/prism-query/src/` (E-QUERY-038 gate at stats + downstream positions)
**BC reference:** BC-2.11.016 v1.7 (gate coverage)

**Finding:** The stats pipe stage (`| stats count(*) as alert_count by severity`) produces aggregate alias columns (e.g., `alert_count`) that do not exist in the source table schema. Downstream references to these alias columns in `| sort` were incorrectly flagged as E-QUERY-038. The canonical count-and-sort pattern (`SELECT * FROM alerts | stats count(*) as alert_count by severity | sort alert_count`) was broken by the gate.

**Root cause:** Same root cause as CRIT-001: the gate checked column existence against source-table schema only, not against the enriched set of columns produced by preceding stages. Stats REPLACE semantics (the output replaces the input relation with `{aliases} ∪ {by_fields}`) were not modeled in the binding context.

**Closure:** CLOSED — BC-2.11.016 v1.8: Stats REPLACE semantics added to DERIVED-COLUMN BINDING RULE (output column set = aliases ∪ by_fields, replacing the input relation). Implementer @21f20cb6: Stats stage in binding-context walk replaces available-column set with `{aliases} ∪ {by_fields}`.

---

## Finding ADV-FIX-P2-HIGH-001 — Engine-layer E-QUERY-002 check_operator_type_compatibility failed-open in single-tenant path; no table_registry fallback; test fixture masked it with dual-tenant wiring

**Severity:** HIGH
**Classification:** silent failure / correctness gap
**Affected files:** `crates/prism-query/src/` (check_operator_type_compatibility single-tenant path)
**BC reference:** BC-2.11.004 (pipe mode error ordering — E-QUERY-038 before E-QUERY-002)

**Finding:** `check_operator_type_compatibility` (the engine-layer function that emits E-QUERY-002 for type mismatches) contained a code path for single-tenant configurations that short-circuited with a failed-open return (treating the query as valid) when `table_registry` was not provided. The test fixtures for this function used dual-tenant wiring (which always provides `table_registry`), masking the single-tenant path failure. In a single-tenant Prism deployment (the common demo configuration), IEQ on a non-existent column would bypass the type-compat check entirely — not even reaching E-QUERY-002, let alone E-QUERY-038.

**Root cause:** The single-tenant code path was not updated when the `table_registry` dependency was added for E-QUERY-002 type-checking. The test fixture always provided a dual-tenant context, so the gap was not caught during TDD.

**Closure:** CLOSED — Implementer @21f20cb6: single-tenant path in `check_operator_type_compatibility` now falls back to `table_registry` lookup (returns None/no-op if registry absent rather than failed-open).

---

## Finding ADV-FIX-P2-HIGH-002 — Enrich gate inverted: input column typo unguarded, legal output columns blocked

**Severity:** HIGH
**Classification:** gate-inversion / correctness
**Affected files:** `crates/prism-query/src/` (E-QUERY-038 gate position-13 enrich input)
**BC reference:** BC-2.11.016 v1.7 (position 13)

**Finding:** The position-13 gate (enrich INPUT column reference, e.g., `iocs_value` in `| enrich threat_score(iocs_value)`) was inverted. The implementation gated enrich OUTPUT columns (blocking them as non-existent) but not enrich INPUT columns (allowing typos to pass through silently). A query `| enrich threat_score(iocs_valu)` with a typo in the input column would not receive E-QUERY-038; it would silently produce an enrich with empty/null output. Conversely, valid enrich output columns in downstream positions were flagged as errors (CRIT-001 class).

**Root cause:** The binding-context walk had the enrich stage's check polarity reversed: it was checking downstream derived-output columns against the source schema (wrong) rather than checking the input column reference against the source schema (correct).

**Closure:** CLOSED — Implementer @21f20cb6: position-13 check corrected — input column reference to `check_pipe_stage_columns` checks the enrich INPUT column against the pre-enrich available set; enrich OUTPUT columns are added to the binding context (resolving CRIT-001 simultaneously).

---

## Finding ADV-FIX-P2-HIGH-003 — BC-2.11.016 v1.7 silent on derived-column bindings; CRIT-001/002 are spec-gap class findings

**Severity:** HIGH
**Classification:** spec-gap (BC structural omission)
**Affected files:** `.factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md`
**BC reference:** BC-2.11.016 v1.7

**Finding:** BC-2.11.016 v1.7 specified the fourteen-position gate (expanded from twelve by PO amendment) but was entirely silent on the derived-column binding context mechanism. There was no §Precondition, §Invariant, or §Postcondition addressing: (a) how the available-column set evolves as stages are traversed; (b) enrich stage REPLACE vs union vs fail-open semantics; (c) stats stage REPLACE semantics; (d) the never-false-positive invariant. Without these rules, any correct implementation would need to infer the correct semantics from BC-2.06.019 and the broader PQL semantics — the BC was under-specified for the implementation it governed.

**Root cause:** The PO authored BC-2.11.016 v1.7 focused on position enumeration (the pass-1 fix) without addressing derived-column propagation semantics. This is the same class of BC structural omission as ADV-FIX-P1-OBS-002 (AST-completeness invariant), which required a co-closure v1.6 spec amendment.

**Closure:** CLOSED — BC-2.11.016 v1.7→v1.8: DERIVED-COLUMN BINDING RULE added in §Preconditions with explicit available/suspended context, Stats REPLACE, Enrich union-or-suspend, FP-001 never-false-positive invariant, and EC-11-053..EC-11-056 (4 new test vectors).

---

## Finding ADV-FIX-P2-OBS-001 — Stale "PO fixing" comment in drift.rs

**Severity:** OBS
**Classification:** stale comment / housekeeping
**Affected files:** `crates/prism-query/src/` (drift.rs or check_pipe_stage_columns module)

**Finding:** A comment reading "PO fixing" or similar was found in the implementation code, a leftover from the active-development pass-1 fix cycle. Comments referring to in-progress work as if it is still in-progress are misleading in a committed codebase.

**Closure:** CLOSED — Implementer @21f20cb6: comment scrubbed in OBS-001 sweep.

---

## Finding ADV-FIX-P2-OBS-002 — Audit-script G4 permissive predicate; follow-up registered with DRIFT-AUDIT-SCRIPT-UNCOMMITTED-001

**Severity:** OBS
**Classification:** audit-script coverage / deferred
**Affected files:** `scripts/t13-preflight-audit.py` (G4 item predicate)

**Finding:** T13 audit-script item G4 uses a permissive predicate that would pass even with the gate inversion identified in HIGH-002. The G4 check is not sensitive enough to catch the output-vs-input gate polarity reversal. This is the same audit-script coverage gap class as ADV-FIX-P1-OBS-001 (G7 temporal predicate).

**Adjudication:** Registered as a follow-up note against DRIFT-AUDIT-SCRIPT-UNCOMMITTED-001 (the existing housekeeping drift item for the uncommitted t13-preflight-audit.py extension). No separate blocker.

**Status:** DEFERRED — registered in DRIFT-AUDIT-SCRIPT-UNCOMMITTED-001. No blocker for cascade progression.

---

## ALWAYS-SUSPEND Enrich-Output — FLAG FOR PASS-3 ADJUDICATION

The implementer chose an ALWAYS-SUSPEND strategy for enrich output columns in the binding context: any reference to an enrich output column in a downstream position produces a suspension (treated as unavailable) rather than a union (treated as available). This is a BC-sanctioned fail-open choice per the BC-2.11.016 v1.8 "union-or-suspend" clause but is the MORE CONSERVATIVE path. The union path (making enrich output columns available to downstream positions) is the correct semantically-rich behavior (it allows `| enrich threat_score(iocs_value) | where threat_score > 5`).

Pass-3 adversary MUST adjudicate: does ALWAYS-SUSPEND enrich-output cause a regression in canonical BC-2.06.019 demo scenarios? If the canonical pivot queries require downstream reference to enrich-output columns, ALWAYS-SUSPEND is a DEFECT and must be replaced with the union path.

---

## Fix-Burst Chain Summary

| Commit | Author | Change |
|--------|--------|--------|
| c2ece301 | (frozen pass-2 HEAD) | 12-position E-QUERY-038 gate; 5329 tests; pre-binding-context |
| 4b5dc7f1 | test-writer | 10 RED Gate tests: binding-context scenarios (enrich-output availability, stats-alias sort, enrich-input typo detection, single-tenant type-compat path) |
| 21f20cb6 | implementer | binding-context walk in check_pipe_stage_columns; Stats REPLACE {aliases ∪ by_fields}; anonymous-aggregate suspension; enrich input position-13 check; ALWAYS-SUSPEND enrich-output [FLAG FOR PASS-3]; dedup position-14 gate; single-tenant table_registry fallback in check_operator_type_compatibility; OBS-001 stale comment scrub |

**Final fix-branch HEAD:** 21f20cb6 (LOCAL-ONLY; not yet pushed to origin)
**Test count after fix-burst:** 5339/5339 (just check GREEN; 10 tests added; 20/20 module tests GREEN; non-exhaustive 89/89)
**LOCAL 3-CLEAN streak:** 0/3 — NEXT: freeze branch HEAD 21f20cb6 → LOCAL pass 3 (fresh adversary; must adjudicate ALWAYS-SUSPEND enrich-output choice)

---

## Spec Versions at Fix-Burst Close

| Artifact | Before | After |
|----------|--------|-------|
| BC-2.11.016 | v1.7 | v1.9 (v1.8: FOURTEEN-position gate + DERIVED-COLUMN BINDING RULE + FP-001 invariant + EC-11-053..056; v1.9: sort-grammar fix `\| sort by`→`\| sort` + every v1.8 vector parser-verified; Lesson 15 RECURRENCE) |
| BC-2.11.004 | v1.15 | v1.17 (v1.16: ADV-FIX-P2 propagation positions 13+14 + derived-binding note; v1.17: sort-grammar propagation) |
| BC-2.11.020 | v1.3 | v1.5 (v1.4: ADV-FIX-P2 propagation positions 13+14 + derived-binding note; v1.5: sort-grammar propagation) |
| error-taxonomy | v2.22 | v2.24 (v2.23: ADV-FIX-P2 closures; v2.24: sort-grammar round) |
| BC-INDEX | v7.59 | v7.61 (v7.60: ADV-FIX-P2 row syncs; v7.61: sort-grammar round) |
| S-PRISMQL-CASE-INSENSITIVE-001 | v1.38 | v1.40 |
| S-DEMO-FIDELITY-REMEDIATION-001 | v2.25 | v2.27 |
| S-DEMO-PRISMQL-ONBOARDING-001-B | v2.2 | v2.4 |
| S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 | v1.14 | v1.16 |

---

## Process-Gap Lessons Appended This Burst

- **L17 [process-gap]:** Lesson 15 RECURRED same-cascade (`| sort by` after `| project`) — PO grammar-keyword verification must be a mandatory pre-lock checklist step.
- **L18 [process-gap]:** Sibling-BC micro-bumps mid-cascade caused 4 story-pin propagation rounds — batch spec amendments before dispatching pin propagation.
- **L19:** Plan-time gate changes need integration coverage — canonical-pivot tests (bc_2_06_019_canonical_pivot_queries.rs) run only against DTU infra; module-level regression tests now lock the patterns.
