---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [4]
feature_head_at_review: e11cc8a7
date: 2026-07-14
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 1
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 4 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 4 (frozen e11cc8a7; fresh-context adversary; PR #222 MCP row-shape null serialization + H8b redundancy sweep + threatintel .prx staleness gate; PR-LEVEL cascade; streak candidate 4/3 — NOT ADVANCING — 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 1 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 1 OBS / 0 PROCESS-GAP)

**STREAK:** 0/3 — NOT CLEAN(strict); 1 OBS finding present. CLEAN(PR-merge)=YES (no CRIT/HIGH/MED). Finding closed fix-burst 17 (branch commits e11cc8a7→a2652c4c, PUSHED; PR #222 head confirmed a2652c4c; streak resets on push per DRIFT-ORCH-PRLEVEL-PUSH-001). NEW FROZEN HEAD a2652c4c.

**Code HEAD at review:** e11cc8a7 (frozen; fix-burst 16 MED-001 safety-arm addition + E-QUERY-005/010 GREEN locks; BC-2.10.007 v1.13→v1.14; error-taxonomy v2.46→v2.47; pushed to origin; PR #222 OPEN base develop; 470/470 prism-mcp GREEN; 5504/5504 workspace GREEN; non-exhaustive 91/91; CI green on e11cc8a7)

**CLEAN(strict):** NO — 1 OBS finding; strict criterion requires zero findings of any severity

**CLEAN(PR-merge):** YES — zero CRIT + HIGH + MED findings; OBS non-blocking

**SAP-1 result:** PASS — 12 event_type values scanned across crates/; all present in BC-2.16.002 Canonical Structured Event Catalog; no new unregistered emissions

---

## Policy Rubric Verification

Full VariantMeta categorization surface verified against BC-2.10.007 v1.14 (all 9 categories enumerated):

| Category | Variants | arm type | Notes |
|----------|----------|----------|-------|
| `query_error` | QueryPlanFailed, QueryExecutionFailed, QueryMaterializationLimitExceeded, QueryMemoryBudgetExceeded, QueryVirtualFieldFailed | nested arm | §LOW-002; all 6 variants locked by test |
| `safety` | SafetyContextContamination, SafetyDataExfiltration | dedicated arm | §MED-001; added fix-burst 16; 2 tests RED→GREEN |
| `watchdog` | QueryDenylisted | nested arm | §LOW-002; covered by pre-existing test |
| `sensor_error` | SensorFetchFailed, SensorAuthFailed, SensorConfigurationInvalid, SensorTimeout | dedicated arm | |
| `upstream_error` | catch-all (excludes safety per §MED-001 amendment) | catch-all | BC-2.10.007 v1.14 catch-all text explicitly excludes safety variants |
| `not_found` | TableNotFound | dedicated arm | |
| `invalid_query` | QueryParseError, QueryParseFailed | dedicated arm | |
| `permission_error` | AccessDenied, RateLimitExceeded | dedicated arm | |
| `internal_error` | all remaining variants | Rule 1 catch-all | |

BC-2.10.007 v1.14 ↔ error-taxonomy v2.47 ↔ error_mapping.rs: byte-verbatim on category strings, error codes, suggestion text, retryable flags. Safety-arm tests assert all 6 BC fields (category, code, message, suggestion, retryable, original_params_valid). Plugin surface and MCP gate surface consistent.

EC-11-079 chokepoint (the single `map_prism_error` function) verified with 5 dedicated-arm test probes + guard test. ADR-051 §D2 code-truth rule verified: BC-2.10.007 §MED-001 cites code location; guard test enforces that the catch-all cannot absorb safety variants.

---

## Findings

### F-MCPRS-PRL4-OBS-001 — Test symmetry: safety_data_exfiltration arm missing Rule-1 message invariance assertion

**Severity:** OBS
**Classification:** test-symmetry / td-vsdd-060-miss-at-test-authoring

**Finding:** The dedicated safety arm added in fix-burst 16 produced two tests:
1. `test_BC_2_10_007_safety_context_contamination_category_is_safety` — asserts category, code, message=="Internal error", suggestion, retryable, original_params_valid (6 fields; Rule-1 message invariance present).
2. `test_BC_2_10_007_safety_data_exfiltration_category_is_safety` — asserts category, code, suggestion, retryable, original_params_valid (5 fields; **Rule-1 message=="Internal error" assertion absent**).

The contamination sibling test had the message assertion; the exfiltration sibling did not. The asymmetry means the exfiltration arm's message contract (Rule 1: `map_prism_error` must not mutate the message field; it passes through from PrismError) was only implicitly covered via the contamination sibling, rather than being explicitly locked per-variant.

A TD-VSDD-060 sibling-sweep at test-authoring time (CLAUDE.md Self-Audit Checklist #8) would have caught this: both safety variants have the same 6-field assertion pattern per BC-2.10.007 §MED-001; the exfiltration test was missing one of the six.

This is an OBS-level finding because: (a) the safety arm itself is behaviorally correct (code produces the right message output for both variants); (b) Rule-1 message invariance is covered by the contamination test; (c) the only gap is that the exfiltration test does not independently lock the message field.

**Scope of TD-VSDD-060 sweep at fix-burst 17:** The 7 remaining dedicated-arm tests (E-QUERY-002, E-QUERY-005, E-QUERY-008, E-QUERY-010, E-QUERY-034, E-WATCHDOG-001 catch-all guard, + exfiltration primary) were swept for the same symmetry gap. All 6 internal-arm tests (`§LOW-002` arm: QueryPlanFailed/QueryExecutionFailed/QueryMaterializationLimitExceeded/QueryMemoryBudgetExceeded/QueryVirtualFieldFailed/QueryDenylisted) were also found to be missing the Rule-1 message assertion; all were added in fix-burst 17. Total: 7 tests updated (exfiltration + 6 internal-arm). The contamination test was already symmetric and required no change.

**Closure:** @a2652c4c: Rule-1 message-invariance assertion (`assert_eq!(result.message, "Internal error", "Rule 1: map_prism_error must not mutate the message field")`) added to 7 tests: exfiltration primary + QueryPlanFailed + QueryExecutionFailed + QueryMaterializationLimitExceeded + QueryMemoryBudgetExceeded + QueryVirtualFieldFailed + QueryDenylisted. All 8 dedicated-arm tests now symmetric (contamination sibling unchanged). prism-mcp 470/470; just check 5504/5504 GREEN.

**Status:** CLOSED @a2652c4c (7 tests updated; all 8 dedicated-arm tests symmetric on Rule-1 message assertion)

---

## Summary

| Finding | Severity | Status | Fix Commit |
|---------|----------|--------|------------|
| F-MCPRS-PRL4-OBS-001 | OBS | CLOSED | @a2652c4c |

**Fix-burst 17 summary (e11cc8a7→a2652c4c):**
- Test-only change: Rule-1 message-invariance assertion (`assert_eq!(result.message, "Internal error", "Rule 1: ...")`) added to 7 tests via TD-VSDD-060 sibling sweep (exfiltration primary + 6 internal-arm tests); contamination test unchanged (already symmetric)
- All 8 dedicated-arm tests now assert all 6 BC-2.10.007 §MED-001 fields
- prism-mcp 470/470; just check 5504/5504 GREEN; non-exhaustive 91/91
- CI note: e11cc8a7 main-suite run cancelled (superseded by a2652c4c push); crate-layout succeeded; full gate running on a2652c4c
- No BC/spec/index changes — test-only burst

**Finding-decay trend:** 6 → 5 → 2 → 1 (CLEAN(PR-merge) reached pass 4)

**Next:** PR-LEVEL pass 5 on frozen HEAD a2652c4c (streak 0/3; BC-5.39.001 3-CLEAN criterion)
