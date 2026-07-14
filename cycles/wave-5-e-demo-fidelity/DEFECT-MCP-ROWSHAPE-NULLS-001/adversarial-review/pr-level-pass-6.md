---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [6]
feature_head_at_review: a2652c4c
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
  out_of_scope_obs: 3
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 6 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 6 (frozen a2652c4c; fresh-context adversary; PR #222 MCP row-shape null serialization + H8b redundancy sweep + threatintel .prx staleness gate; PR-LEVEL cascade; streak RESET 1/3 → 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

---

## Findings

### F-MCPRS-PRL6-MED-001 [MED][spec-code-drift]

**BC:** BC-2.10.007 §Canonical Test Vectors — QueryDenylisted row
**Severity:** MED
**Classification:** spec-code-drift (BC narrative drifted from shipped code; shipped test was correct)

**Finding:** BC-2.10.007 v1.14 §Canonical Test Vectors row for `QueryDenylisted` cited a nonexistent field `query_hash: "abc123"` as the input struct. The actual `PrismError::QueryDenylisted` struct in `prism-core/src/error.rs` has three fields:

```rust
PrismError::QueryDenylisted { failure_count: u32, reason: String, expiry_ts: u64 }
```

The BC's §LOW-002 "Tests to add" entry for `test_BC_2_10_007_query_denylisted_category_is_internal` also cited `query_hash`.

**Impact:** The shipped test (`test_BC_2_10_007_query_denylisted_category_is_internal`) was CORRECT — it used the actual struct fields. The BC narrative was the drifted artifact. This is spec-code drift (spec wrong; code right). No runtime behavior defect.

**Root cause:** v1.12 authoring error — the QueryDenylisted vector row was written with a hypothetical `query_hash` field that never existed in `prism-core/src/error.rs`. The v1.13 and v1.14 amendments that added new rows and sections did not detect the existing query_hash drift (TD-VSDD-059 partial-coverage gap at BC authoring time).

**Resolution (fix-burst 18, spec-only):** BC-2.10.007 v1.15 — §Canonical Test Vectors QueryDenylisted row corrected to `{ failure_count: 3, reason: "timeout".to_string(), expiry_ts: 9_999_999 }`. §LOW-002 "Tests to add" entry corrected. POL-29 full sweep: all 10 vector rows verified against `prism-core/src/error.rs` (9 other rows confirmed correct). Branch HEAD UNTOUCHED at a2652c4c.

---

## Out-of-Scope Observations

The following observations are pre-existing drift not introduced by this PR branch. Per the frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001), these are dispositioned via spec adjudication (already committed to factory-artifacts in fix-burst 18) or deferred routing. Branch remains UNTOUCHED at a2652c4c.

### OOS-1: SensorHttpError 503 retryable spec-vs-code drift

**Observation:** §Canonical Test Vectors row for `PrismError::SensorHttpError { .. }` (sensor returns 503) specifies `retryable: true`. Shipped `error_mapping.rs` `SensorHttpError` arm sets `retryable: false` for ALL status codes unconditionally — the entire arm lacks status-code discrimination.

**Disposition:** **Spec is correct. Code fix required via separate fix-burst.** BC §Complete field specification defines `retryable: true` for "transient errors (rate limit, timeout, network)"; HTTP 503 Service Unavailable is an explicitly transient status. The transient set is `{408, 425, 429, 500, 502, 503, 504}`. Code change: `let retryable = matches!(status.as_u16(), 408 | 425 | 429 | 500 | 502 | 503 | 504)` in the `SensorHttpError` arm. Note: 400/404/422 and other 4xx codes are permanent client-side errors (not retryable); 401/403 are permanent credential failures (not transient).

**v1.15 adjudication added** to BC-2.10.007 as §RETRYABLE-503 under §Implementer Code Follow-Up. v1.16 precision-corrected the rule from overbroad `!matches!(status, 401|403)` to transient-only `matches!(status.as_u16(), 408|425|429|500|502|503|504)`. Fix dispatched as fix-burst 19 (code-only; implementer applying whitelist to `error_mapping.rs` SensorHttpError arm + tests; branch frozen at a2652c4c pending FB19 push).

### OOS-2: E-SENSOR-099 absence from error-taxonomy v2.48

**Observation:** `E-SENSOR-099` is emitted at runtime via two known paths but had no row in error-taxonomy v2.48:
1. `SensorError::Internal { detail: String }` in `prism-sensors/src/adapter.rs` — Display verbatim `"E-SENSOR-099: internal sensor error: {detail}"`
2. Direct format string in `prism-mcp/src/server.rs:3275` health probe path — `format!("E-SENSOR-099: health probe failed: {e}")`

Also referenced in `prism-credentials/src/resolution.rs` comment chain as the surface code for auth acquisition failures routed through `SensorError::Internal`.

**Disposition:** E-SENSOR-099 row added to error-taxonomy **v2.48→v2.49** in fix-burst 18 (spec-only; PO committed to factory-artifacts). Severity: `degraded`; category: `internal`; retryable: No. `{detail}` field must not carry credential values (AD-017). MCP surface: redacted to `-32000 INTERNAL_ERROR` / `"Internal error"` per BC-2.10.007 Rule 1 (E-SENSOR-* errors carry potential credential context).

### OOS-3: Threatintel manifest comment drift

**Observation:** Comment-level drift in the threatintel plugin manifest referencing a production API endpoint in scope for the pending S-MCP-THREATINTEL-PROD-ENDPOINT-001 story. The comment anticipated an endpoint configuration that belongs in the story's implementation scope rather than the DEFECT-MCP-ROWSHAPE-NULLS-001 PR.

**Disposition:** Comment drift anchored to S-MCP-THREATINTEL-PROD-ENDPOINT-001 v0.2 scope notes in fix-burst 18 (spec-only; committed to factory-artifacts). No branch change required.

---

## SAP-1 Verification

SAP-1 (tracing emission catalog completeness) PASS:
- `rg 'event_type\s*=' crates/ --type rust` executed: all emission sites pre-existing and catalogued in BC-2.16.002 §Postconditions Canonical Structured Event Catalog
- Zero new `event_type =` emissions introduced in this branch (fix/DEFECT-MCP-ROWSHAPE-NULLS-001 HEAD a2652c4c)
- Emission count: consistent with prior passes (no additions)

---

## Fix-Burst 18 Summary (spec-only; branch UNTOUCHED at a2652c4c)

**BC-2.10.007 v1.14→v1.15→v1.16:**
- v1.15: F-MCPRS-PRL6-MED-001 — QueryDenylisted vector corrected (`query_hash` → `{failure_count: 3, reason: "timeout".to_string(), expiry_ts: 9_999_999}`); §LOW-002 "Tests to add" corrected; POL-29 full 10-row vector sweep; §RETRYABLE-503 §Implementer Code Follow-Up section added (initial adjudication: spec correct, code fix required)
- v1.16: §RETRYABLE-503 rule precision fix — overbroad `!matches!(status, 401|403)` corrected to transient-only `matches!(status.as_u16(), 408|425|429|500|502|503|504)`; rationale text updated; "Tests to add/update" bullet updated to preserve permanent-status-code assertions

**error-taxonomy v2.48→v2.49:** E-SENSOR-099 row added (two emission paths documented; severity degraded; category internal; retryable No; AD-017 credential opacity; BC-2.10.007 Rule 1 MCP redaction)

**Threatintel manifest comment:** drift anchored to S-MCP-THREATINTEL-PROD-ENDPOINT-001 v0.2 scope

**POL-23 sweep:** S-MCP-E003-SERIALIZATION-MIGRATION-001 v0.4→v0.6 (9 pins ×2 rounds); S-TEST-WIRESHAPE-SWEEP-001 v0.13→v0.15 (11-12 pins ×2 rounds)

---

## Fix-Burst 19 Status (code-only; IN FLIGHT)

Implementer dispatched to apply §RETRYABLE-503 whitelist (`matches!(status.as_u16(), 408 | 425 | 429 | 500 | 502 | 503 | 504)`) to `error_mapping.rs` `SensorHttpError` arm + associated tests. Branch HEAD FROZEN at a2652c4c pending FB19 push.

---

## Finding-Decay Trajectory

6 → 5 → 2 → 1 → 0 → **1** (MED-001 spec-code-drift)

---

## Streak Status

**0/3** — RESET by F-MCPRS-PRL6-MED-001.

PR-LEVEL 3-CLEAN streak was 1/3 (from pass-5 CLEAN). F-MCPRS-PRL6-MED-001 resets streak to 0/3. Per frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001): branch remains frozen at a2652c4c until 3/3 achieved.

---

## Cascade Status

- Total passes: 26 (pre-PR-LEVEL LOCAL cascade + PR-LEVEL 1–6)
- Fix-bursts: 18 completed (+ FB19 IN FLIGHT)
- PR-LEVEL streak: 0/3 RESET
- HEAD: a2652c4c (FROZEN)
- PR #222: OPEN; merge HUMAN-GATED
- Next: FB19 push → PR-LEVEL pass 7 on new frozen HEAD
