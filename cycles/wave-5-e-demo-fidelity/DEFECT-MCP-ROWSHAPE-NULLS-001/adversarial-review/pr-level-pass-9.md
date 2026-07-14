---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [9]
feature_head_at_review: 448158f8
date: 2026-07-14
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 0
  low: 1
  obs: 0
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 9 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 9 (frozen 448158f8; fresh-context adversary; PR #222 MCP row-shape null serialization + H8b redundancy sweep + threatintel .prx staleness gate + 905-pin version-agnostic sweep; PR-LEVEL cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

---

## Findings

### F-MCPRS-PRL9-LOW-001 [LOW][test-coverage] — CLOSED (fix-burst-21 @fa0e4d70)

**Severity:** LOW
**Classification:** test-coverage — retryable status code whitelist partial coverage
**BC:** BC-2.10.007 (§RETRYABLE-503 retryable error whitelist)

**Finding:** The retryable status code whitelist in `error_mapping.rs` covers the full set: 408 Request Timeout, 425 Too Early, 429 Too Many Requests, 500 Internal Server Error, 502 Bad Gateway, 503 Service Unavailable, 504 Gateway Timeout. The existing test exercises only a subset of these codes with an explicit positive assertion (408, 429, 503 verified individually in `test_retryable_classification`). Codes 425, 500, 502, and 504 are covered by an implicit bulk loop but the loop asserts only `is_retryable == true` without asserting the error code and classification fields in the structured `PrismError` output.

The BC-2.10.007 §RETRYABLE-503 postconditions require that each listed code produces the correct `retry_after_hint`, `attempt_number`, and `max_attempts` fields. The bulk-loop assertion does not verify these structured output fields for the four under-asserted codes.

**Impact:** Zero runtime impact — the retryable logic itself is correct. The gap is test-assertion completeness: four codes in the whitelist had weaker positive assertions than BC-2.10.007 §Postconditions requires. Severity: LOW (production behavior correct; test rigour gap only).

**Resolution (fix-burst-21 code @fa0e4d70):** Implementer expanded `test_retryable_classification` to add individual structured-field assertions for codes 425, 500, 502, and 504, verifying `retry_after_hint`, `attempt_number`, and `max_attempts` per BC-2.10.007 §Postconditions for each. prism-mcp test suite fully green. pre-push `just check` GREEN 5511/5511 (prism-mcp 474/474; non-exhaustive 91/91). Branch pushed: 448158f8→fa0e4d70. NEW MCP FROZEN HEAD: fa0e4d70. PR-LEVEL streak 0/3 RESET by push (DRIFT-ORCH-PRLEVEL-PUSH-001). PR-LEVEL pass 10 dispatched on frozen fa0e4d70.

---

## SAP-1 Emission Catalog Probe

**PASS.** No new `event_type =` emissions introduced by the branch relative to develop@5f1b5771. All emissions present in the branch were catalogued in BC-2.16.002 §Postconditions in prior bursts.

---

## Summary

**CLEAN(strict): NO** (1 LOW finding — F-MCPRS-PRL9-LOW-001 retryable-coverage partial assertion)
**CLEAN(PR-merge): YES** (zero CRIT/HIGH/MED findings)

Streak: **0/3** (stays 0/3; LOW finding prevents strict-clean advancement per BC-5.39.001).

In-scope LOW finding closed by fix-burst-21 @fa0e4d70. NEW FROZEN HEAD: fa0e4d70. Cascade tally: 29 passes / 21 fix-bursts.
