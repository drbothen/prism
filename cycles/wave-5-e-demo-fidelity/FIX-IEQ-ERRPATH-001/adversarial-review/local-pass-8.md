---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [8]
feature_head_at_review: eafe10c2
fix_burst_head: 7a2a0f73
date: 2026-07-09
clean_strict: false
clean_pr_merge: true
finding_counts:
  OBS: 2
  total: 2
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 8 — FIX-IEQ-ERRPATH-001

---

## Pass 8 (frozen eafe10c2; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 2 (0 CRIT/HIGH/MED + 2 OBS) — first pass at PR-merge-CLEAN

**Code HEAD at review:** eafe10c2 (frozen; D-1618 implementer provenance-aware binding context + table_alias threading + FIELDS transitions; 5360/5360 GREEN; non-exhaustive 89/89)

**Fix-burst HEAD:** 7a2a0f73 (test-writer: 5 alias-qualified regression locks positions 10-14; BC-2.11.016 v1.16 narrative-only; 5365/5365 GREEN; non-exhaustive 89/89)

**LOCAL 3-CLEAN(strict) streak after pass-8:** 0/3 (NOT CLEAN(strict); fix-burst dispatched; RESET by @7a2a0f73 push per DRIFT-ORCH-PRLEVEL-PUSH-001)

---

## Finding ADV-FIX-P8-OBS-001 — Alias-qualified regression coverage existed only for position 9; positions 10-14 unlocked against future table_alias-threading regressions

**Severity:** OBS (coverage-only)

**Classification:** coverage gap — no code-behavior defect; implementation confirmed correct at eafe10c2; regression locks missing for alias-qualified form across positions 10-14

**Affected files:** `crates/prism-query/src/` — `engine.rs` `check_pipe_stage_columns` PipeStage::Sort/Stats/Fields/Enrich/Dedup arms (positions 10-14); test suite for alias-qualified variant

**BC reference:** BC-2.11.016 v1.15 §Gate Position table positions 10-14; EC-11-068 (from_alias threading invariant); FP-001 (fail-open discipline)

**Finding:** The fix-burst at eafe10c2 correctly implemented table_alias threading across all 14 gate positions. However, the regression test suite at eafe10c2 included alias-qualified form (`alias.column`) coverage only for position 9 (SqlPipe `| where` arm). Gate positions 10 (`| sort`), 11 (`| stats by` grouping refs + agg-arg paths), 12 (`| fields`), 13 (`| enrich` input), and 14 (`| dedup` field keys) lacked alias-qualified regression locks. A future refactor that breaks table_alias threading for positions 10-14 would not produce a RED test. The implementation at eafe10c2 was verified correct by manually tracing the code path — this is purely a coverage gap, not a behavioral defect.

**Additional observation — position-11 grammar deviation:** The Pipe form `| stats alias.column by ...` was used in the regression lock for position 11 rather than a SqlPipe form. This is intentional: the SqlPipe pipe-stage parser does not support `| stats` (SqlPipe composes a SQL SELECT head with Pipe tail stages; `| stats` is a Pipe-only construct). The regression lock correctly uses the Pipe form with the table-name qualifier. This grammar boundary is documented in BC-2.11.020 §Pipe Stage Composition and is not a defect.

**Routed:** test-writer

**Closure:** CLOSED — 5 alias-qualified regression locks added for positions 10-14 @7a2a0f73, all GREEN on first run. Implementation confirmed correct (GREEN first-run is load-bearing evidence). Position-11 grammar deviation (Pipe form with table-name qualifier; `| stats` absent from SqlPipe pipe-stage parser) documented in lock comment. 5365/5365 GREEN; non-exhaustive 89/89.

---

## Finding ADV-FIX-P8-OBS-002 — BC-2.11.016 §Preconditions Enrich bullet Note contradicted actual BC-2.11.019 gate ordering

**Severity:** OBS (narrative-only; no code-behavior defect)

**Classification:** spec commentary defect — BC prose claimed "E-QUERY-039 pre-gates BEFORE the binding-context walk / suspension unreachable-by-design" contradicting the actual and correct BC-2.11.019 gate ordering

**Affected files:** `.factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md` — §Preconditions Enrich bullet Note

**BC reference:** BC-2.11.016 v1.15 §Preconditions Enrich bullet; BC-2.11.019 (E-QUERY-039 gate — fires AFTER the resolver walk, not before)

**Finding:** The §Preconditions Enrich bullet in BC-2.11.016 v1.15 contained a Note stating that "E-QUERY-039 pre-gates BEFORE the binding-context walk, making resolver-miss suspension unreachable by design." This claim is false. BC-2.11.019 defines E-QUERY-039 as firing on registry-absent enrich clauses (unknown infusion name), which is checked AFTER the column-availability walk for the enrich input fields. The resolver-miss suspension path in E-QUERY-038 (suspending fail-open when the enrich registry lookup fails) is a live defensive fail-open path — not unreachable. The Note was likely written during an earlier draft based on an incorrect mental model of the gate sequence and survived multiple BC amendments undetected because it was in a Note sub-block rather than the normative Precondition text.

**Routed:** product-owner

**Closure:** CLOSED — BC-2.11.016 v1.15→v1.16 NARRATIVE-ONLY: §Preconditions Enrich bullet Note corrected to describe actual BC-2.11.019 gate ordering (E-QUERY-039 fires AFTER the walk; resolver-miss suspension is live defensive fail-open). No rule/EC/behavior change. Sweep grep evidence: "unreachable-by-design" removed from BC-2.11.016 body; no sibling BC amendments needed (the term does not appear in any other BC file). BC-INDEX v7.67→v7.68. 2 story pin syncs (S-DEMO-FIDELITY-REMEDIATION-001 v2.33→v2.34, S-DEMO-PRISMQL-ONBOARDING-001-B v2.10→v2.11; 6 and 2 pin sites respectively).

---

## Pass Notes

**SAP-1 (Structured Event Catalog):** PASS — 3 `column_not_found.rejected` emission sites verified cataloged in BC-2.16.002 v2.07 Canonical Structured Event Catalog. No new `event_type =` sites introduced in the fix-burst.

**POL-24 (byte-verbatim EC-body):** PASS — no new EC bodies added in this fix-burst; narrative-only BC amendment.

**POL-25 (sibling BC propagation):** PASS — BC-2.11.016 v1.16 is narrative-only; sibling BCs (BC-2.11.004, BC-2.11.020) have no Enrich-bullet Note to propagate; no sibling amendments required.

**POL-7 (BC title cell accuracy):** PASS — no BC titles changed.

**TD-VSDD-060 (sibling-site sweep on changed signatures):** PASS — both changed signatures in eafe10c2 (`check_pipe_stage_columns` new parameters: `initial_binding_override`, `table_alias`) swept; all call sites confirmed updated; no new callsites introduced in 7a2a0f73 (test-only commit).

**FP-001 invariant (fail-open discipline):** PASS — all 14 binding-context gates verified fail-open for anonymous unaliased non-Field items; resolver-miss suspension path confirmed live and reachable (ADV-FIX-P8-OBS-002 correction confirms this).

**AST-completeness invariant (OBS-002 from pass-6):** PASS — all 14 gate positions covered in test suite post-7a2a0f73; alias-qualified regression locks added for positions 10-14 (ADV-FIX-P8-OBS-001 closure).

**Forbidden patterns sweep:** PASS — no `unwrap()`/`expect()` in changed paths; no `println!`; no new pub types requiring `#[non_exhaustive]`.

**Story pins:** PASS — 2 carrier stories pinned to BC-2.11.016 v1.16 (narrative-only); S-PRISMQL-CASE-INSENSITIVE-001 and S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 confirmed zero BC-2.11.016 pins via grep — no updates needed.

**Novelty:** MODERATE and decaying — pass-6: 2 MED + 1 OBS; pass-7: 1 MED + 2 OBS; pass-8: 2 OBS (zero CRIT/HIGH/MED). First pass at PR-merge-CLEAN. The OBS-001 coverage class (missing regression locks for correct implementation) is lower-severity than prior structural/behavioral findings. The OBS-002 spec commentary defect is a doc-accuracy class without code impact. Decay trajectory indicates approaching CLEAN(strict).
