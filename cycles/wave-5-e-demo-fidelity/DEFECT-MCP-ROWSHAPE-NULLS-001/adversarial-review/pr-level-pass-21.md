---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [21]
feature_head_at_review: 5d2624aa
date: 2026-07-14
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 1
  low: 0
  obs: 0
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 21 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 21 (frozen 5d2624aa; fresh-context adversary; PR #222 MCP row-shape null serialization cascade; streak 0/3 → RESET 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

Streak: **0/3** (RESET — 1 MED finding; BC-5.39.001: any finding resets streak regardless of fix-burst type)

---

## Findings

### F-MCPRS-PRL21-MED-001 [MED][BC-INDEX summary-row truth — 2nd instance of class] — CLOSED (D-1762 BC-INDEX v8.22 fix-burst)

**Severity:** MED
**Classification:** spec-drift/index-truth — BC-INDEX governance discovery surface (same class as F-MCPRS-PRL20-MED-001 BC-2.15.009; that instance CLOSED v8.20)

**Finding:** BC-INDEX.md BC-2.11.018 summary row describes v1.5 as containing "normalized_pql echo interaction with null-not-absent constraint" — this is FABRICATED. The actual BC-2.11.018 v1.5 changelog content records the F-MCPNULL-P3-MED-001 phantom-field sweep: four phantom envelope fields corrected in the §Postconditions structured response example (fields that appeared in the spec example but were never emitted by the implementation). The concept "normalized_pql echo interaction" and the phrase "null-not-absent constraint" do not appear anywhere in BC-2.11.018 — a grep of the BC body for both strings returns zero hits. Those semantics belong exclusively to BC-2.11.001 (the `query` MCP tool response contract). Additionally, the v1.4 entry in the same BC-INDEX row is mislabeled as "intermediate" — BC-2.11.018 v1.4 is the substantive `events→rows` sibling sweep under the same F-MCPNULL-P1-MED-001 cascade (a complete, standalone change; not a step toward a larger change).

**Impact:** BC-INDEX is the governance-critical discovery surface read at every session start and at every adversary review gate. An agent or reviewer reading BC-INDEX to understand BC-2.11.018's version history would conclude that v1.5 introduced null serialization constraint interaction into the MCP echo-mode contract — which is false. This creates incorrect dependency attribution between the DEFECT-MCP-ROWSHAPE-NULLS-001 cascade and BC-2.11.018, and misrepresents the scope of cascade impact on the echo-mode path.

**Fix plan (fix-burst next session, state-manager):** BC-INDEX v8.21→v8.22: rewrite BC-2.11.018 row — v1.5 clause to reflect phantom-field sweep ground truth (F-MCPNULL-P3-MED-001; four phantom envelope fields corrected); v1.4 clause to reflect events→rows sibling sweep (F-MCPNULL-P1-MED-001 cascade; complete standalone change, not "intermediate"). Following the rewrite: full audit of ALL cascade-touched BC-INDEX summary rows to confirm no further misdescription instances (BC-2.11.001 v1.22 / BC-2.10.007 v1.19 / BC-2.15.009 v1.7 confirmed accurate by this pass; BC-2.11.018 is the open instance).

**Fix was HELD at wrap-freeze** due to BC-INDEX staging race with D-1760 (BC-INDEX v8.21 committed in that burst; a same-burst second BC-INDEX mutation would have created a multi-commit chain violating TD-VSDD-053). Fix deferred to next session as the first state-manager dispatch.

**Status:** CLOSED — D-1762 fix-burst (state-manager; BC-INDEX v8.21→v8.22; BC-2.11.018 row rewritten to changelog ground truth; spec-only; PR HEAD 5d2624aa UNCHANGED). Pass-22 gates on same frozen 5d2624aa.

**Streak note:** Streak RESET 0/3 per BC-5.39.001 (1 MED finding present at pass-21 gate; DRIFT-ORCH-PRLEVEL-PUSH-001: no push to feature branch 5d2624aa).

---

## Positive Verifications

- **BC-2.11.001 v1.22 verified accurate:** §Postconditions correctly attributes EC-11-079 (null-not-absent invariant) to ADR-051 §D2 (source-returns-None precedent) and EC-11-081 (NaN/±Inf→null) to arrow-json 58.2.0 behavior boundary. No cross-citation confusion with BC-2.11.018 echo path.

- **BC-2.10.007 v1.19 verified accurate:** §RETRYABLE-503 §Implementer Code Follow-Up snippet correctly reflects the `.code()` accessor form used in production `server.rs` at 5d2624aa. BC-INDEX row description matches changelog ground truth.

- **BC-2.15.009 v1.7 verified accurate (pass-20 closure confirmed):** BC-INDEX v8.20 rewrite (F-MCPRS-PRL20-MED-001 closure) correctly reflects the 3-site cosmetic rows-terminology sweep. No regression from the v8.20 rewrite.

- **EC-11-081 locking test load-bearing confirmed:** `test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null` exercises the NaN/±Inf→null arrow-json conversion path at 5d2624aa. Test is green and load-bearing (not a doc-comment or assertion-only paper fix; TD-VSDD-059 satisfied).

- **H8b locks verified:** Explicit 28-arm VariantMeta dispatch (fix-burst-22) confirmed present at 5d2624aa. The 117-variant sentinel test fires on any unmapped variant addition.

- **SAP-1 PASS:** No `event_type =` emission changes in scope of 5d2624aa relative to the cascade baseline. All emission sites catalogued in BC-2.16.002 §Postconditions per standing probe SAP-1.

- **CI gates confirmed:** PR #222 remains at PUSHED HEAD 5d2624aa; CI state unchanged from prior passing runs; no new commit to the feature branch this pass.

---

## Summary

**CLEAN(strict): NO** (1 MED finding — BC-INDEX index-truth summary drift for BC-2.11.018 v1.5/v1.4)
**CLEAN(PR-merge): NO** (1 MED finding; CLOSED D-1762 BC-INDEX v8.22 fix-burst)

Streak: **0/3** (RESET — F-MCPRS-PRL21-MED-001 MED finding present at pass gate; BC-5.39.001 streak-reset rule; spec-only fix does not prevent reset)

Note: F-MCPRS-PRL21-MED-001 CLOSED — D-1762 fix-burst (BC-INDEX v8.21→v8.22; BC-2.11.018 row rewritten to changelog ground truth; spec-only; PR HEAD 5d2624aa UNCHANGED). Finding was HELD at wrap-freeze (TD-VSDD-053 single-commit discipline; D-1760 BC-INDEX v8.21 already committed that burst).

CASCADE TALLY: 41 passes / 27 fix-bursts. Frozen HEAD @5d2624aa UNCHANGED; streak 0/3 (RESET); next = PR-LEVEL pass 22 on same frozen 5d2624aa.
