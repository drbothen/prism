---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [3]
feature_head_at_review: 6aab0f67
date: 2026-07-13
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 2
  crit: 0
  high: 0
  med: 1
  low: 1
  obs: 0
  process_gap: 0
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 3 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 3 (frozen 6aab0f67; fresh-context adversary; PR #222 MCP row-shape null serialization + H8b redundancy sweep + threatintel .prx staleness gate; PR-LEVEL cascade; streak candidate 3/3 — NOT ADVANCING — 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 2 total (0 CRIT / 0 HIGH / 1 MED / 1 LOW / 0 OBS / 0 PROCESS-GAP)

**STREAK:** 0/3 — NOT CLEAN(strict); 1 MED finding present (CLEAN(PR-merge) also fails — first merge-blocking pass this cascade). Both findings CLOSED fix-burst 16 (branch commits 6aab0f67→e11cc8a7, PUSHED; PR #222 head confirmed e11cc8a7; streak resets on push per DRIFT-ORCH-PRLEVEL-PUSH-001). NEW FROZEN HEAD e11cc8a7.

**Code HEAD at review:** 6aab0f67 (frozen; fix-burst 15 LOW-001+OBS-002 README two-vector residual risk + Provenance Anchors row; LOW-003 just@1.43.1 pinned 2 CI jobs; LOW-002 BC-2.10.007 v1.11→v1.13 + VariantMeta arm + 4 category tests; OBS-001 sweep test renamed+split; pushed to origin; PR #222 OPEN base develop; 5499/5499 GREEN; non-exhaustive 91/91; CI green on 91c8dc7f 21/21; new CI run pending on 6aab0f67)

**CLEAN(strict):** NO — 1 MED + 1 LOW finding; strict criterion requires zero findings of any severity

**CLEAN(PR-merge):** NO — 1 MED (code-behavior defect) present; merge-gate requires zero CRIT + HIGH + MED

**SAP-1 result:** PASS — no new event_type emissions without BC-2.16.002 catalog rows

---

## Findings

### F-MCPRS-PRL3-MED-001 — Spec-code drift: SafetyContextContamination + SafetyDataExfiltration miscategorized as upstream_error in catch-all

**Severity:** MED
**Classification:** spec-code-drift / LLM-agent-misdirection

**Finding:** `SafetyContextContamination` and `SafetyDataExfiltration` are `PrismError` variants for internal safety-boundary violations — specifically, cross-context data leakage attempts and exfiltration path detection. BC-2.10.007 §Category table explicitly enumerates a `"safety"` category for these variants, with the prescribed LLM-agent strategy "Do not retry; report to operator." and `retryable: false`.

However, in the shipped `map_prism_error` implementation at the time of pass-3 review (fix-burst-15 state @6aab0f67), both variants fall through to the **catch-all** arm, which emits `category: "upstream_error"` and error code `E-INT-001`. The `"upstream_error"` category carries the strategy "investigate sensor health and retry" — the exact opposite of what the safety boundary contract requires. An LLM-agent receiving either of these errors will attempt to retry the query, potentially triggering additional safety-violation events, when the correct response is immediate halting and operator escalation.

This is a sibling-sweep miss of the LOW-002 fix class from fix-burst-15: fix-burst-15 added a dedicated nested-match arm for internal PrismError variants (QueryPlanFailed, QueryExecutionFailed, etc.) but omitted the safety variants, which require a structurally similar dedicated arm per the same §MED-001 section of BC-2.10.007 that was being updated. The spec (BC-2.10.007 v1.13 §Category) stated the requirement; the code was not updated in scope.

POL-4 (no LLM-agent misdirection) and POL-29 (sibling-sweep completeness on fix-burst-15's LOW-002 fix class) both apply.

**Closure:** @e11cc8a7 (spec @dd2a1594 + code @e11cc8a7): BC-2.10.007 v1.13→v1.14 adds a dedicated `"safety"` arm in §MED-001 — verbatim `VariantMeta` for both `SafetyContextContamination` and `SafetyDataExfiltration`: `category: "safety"`, error codes E-SAFETY-001/E-SAFETY-002 via nested match, `suggestion: "Do not retry; report to operator."`, `retryable: false`, `original_params_valid: true`; Rule 1 invariance stated (map_prism_error not mutated). The §Category catch-all arm description text is amended to explicitly state "safety variants handled by dedicated arm above." Six new Canonical Test Vectors added (E-QUERY-002/005/010 existing internal category locks + 2 safety RED→GREEN + WritePartialFailure catch-all guard). Code: `match` arm for the two safety variants inserted verbatim per §MED-001 in `error_mapping.rs`; 2 tests RED→GREEN confirming correct `"safety"` category + E-SAFETY-001/002 codes; 1 catch-all-not-safety guard test added. `map_prism_error` internal logic untouched per Rule 1.

**Status:** CLOSED @e11cc8a7 (safety arm added; 2 safety tests RED→GREEN; BC-2.10.007 v1.14; error-taxonomy v2.47)

---

### F-MCPRS-PRL3-LOW-001 — Test coverage: E-QUERY-005/010 nested-match arms unlocked only by GREEN tests; BC test vectors cover only 3 of 6 internal variants

**Severity:** LOW
**Classification:** test-coverage / bc-test-vector-completeness

**Finding:** The `§LOW-002` arm added in fix-burst-15 contains nested-match sub-arms for six PrismError variants: `QueryPlanFailed`, `QueryExecutionFailed`, `QueryMaterializationLimitExceeded`, `QueryMemoryBudgetExceeded`, `QueryVirtualFieldFailed`, and `QueryDenylisted`. Fix-burst-15 tests covered `QueryPlanFailed` (E-QUERY-002), `QueryMaterializationLimitExceeded` (E-QUERY-034), and `QueryMemoryBudgetExceeded` (E-QUERY-008) via three test vectors. The arms for `QueryExecutionFailed` (E-QUERY-005), `QueryVirtualFieldFailed` (E-QUERY-010), and `QueryDenylisted` (E-WATCHDOG-001) existed in code but were not locked by any test — they were reachable dead code from a coverage perspective.

Separately, the BC-2.10.007 §LOW-002 Canonical Test Vectors listed only the 3 variants with tests, leaving the other 3 as undocumented. Under BC-5.39.001 convergence discipline, every implemented arm must be locked by a test before a cascade can advance.

**Closure:** @e11cc8a7: Two GREEN lock tests added for `QueryExecutionFailed` (E-QUERY-005) and `QueryVirtualFieldFailed` (E-QUERY-010); `QueryDenylisted`/E-WATCHDOG-001 was already covered by an existing test from the pre-fix-burst-15 watchdog suite (verified by TD-VSDD-060 sweep). BC-2.10.007 v1.14 §LOW-002 Canonical Test Vectors section updated to enumerate all 6 variants with test names.

**Status:** CLOSED @e11cc8a7 (E-QUERY-005/010 GREEN locks added; all 6 variants covered; BC test vectors complete)

---

## Summary

| Finding | Severity | Status | Fix Commit |
|---------|----------|--------|------------|
| F-MCPRS-PRL3-MED-001 | MED | CLOSED | @e11cc8a7 |
| F-MCPRS-PRL3-LOW-001 | LOW | CLOSED | @e11cc8a7 |

**Fix-burst 16 summary (6aab0f67→e11cc8a7):**
- Spec (PO @dd2a1594): BC-2.10.007 v1.13→v1.14 — dedicated safety arm, 6 new test vectors, catch-all excludes safety, Rule 1 invariance stated; error-taxonomy v2.46→v2.47 (E-SAFETY section retitled "Safety Boundary Violations", E-SAFETY-001 severity cosmetic→broken, message formats corrected, MCP mappings documented); POL-23 sweep: S-MCP-E003 v0.3→v0.4 (9 pins), S-TEST-WIRESHAPE v0.12→v0.13 (12 pins)
- Code (implementer @e11cc8a7): safety VariantMeta arm verbatim per §MED-001; 5 tests added (2 safety RED→GREEN + catch-all-not-safety guard + E-QUERY-005/010 GREEN locks — arms pre-existed from fix-burst-15); map_prism_error untouched (Rule 1); TD-VSDD-060 sweep zero stale assertions; prism-mcp 470/470; just check 5504/5504

**Next:** PR-LEVEL pass 4 on frozen HEAD e11cc8a7 (streak 0/3; BC-5.39.001 3-CLEAN criterion)
