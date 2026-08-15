---
document_type: holdout-scenario
level: L3
id: "HS-AUDITLOG-001-A-003"
title: "Claroty audit_logs explicit WHERE timestamp > 45d ago returns records from older window (push-down honors explicit filter)"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "defects-and-drift"
story_source: "S-CLAROTY-AUDITLOG-TIMEBOX-001"
version: "1.1"
status: active
producer: product-owner
timestamp: "2026-08-15T00:00:00Z"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
input-hash: null
traces_to: "BC-2.16.013"
behavioral_contracts:
  - BC-2.16.013
  - BC-2.01.013
verification_properties: []
lifecycle_status: active
introduced: DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-CLAROTY-AUDITLOG-TIMEBOX-001 — push-down honors explicit user time filter (older-than-default window returned, no silent truncation). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-AUDITLOG-001-A-003: Claroty audit_logs explicit WHERE timestamp > 45d ago returns records from older window (push-down honors explicit filter)

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-AUDITLOG-TIMEBOX-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.013 §Postconditions §1 Claroty `audit_logs` — Bounded push-down block: "Explicit `start_time` only: injects a `greater_or_equal` clause for the user-supplied start timestamp. Older-than-7-day windows are honored exactly — no silent truncation."; BC-2.01.013 §Claroty `audit_logs` push-down row
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the push-down implementation honors an explicit user time filter for a
window older than the 7-day default — i.e., there is NO silent truncation to the last 7 days.

When no time filter is provided, the adapter injects a `greater_or_equal now−604800s` default (7d).
When the user supplies an explicit `start_time` older than 7 days, the adapter MUST inject that
user-supplied bound instead of the default. The result set must contain records from the older window.

A silent-truncation bug would surface as: the query returns only the recent-group records and not the
middle-group records, even though the user's filter explicitly covers the older window.

**Behavioral assertions:**

1. The Claroty DTU clone is running with audit log fixtures in three temporal groups:
   - **Recent group** (last 7 days): 3 records — timestamp within `now − 7d`
   - **Middle group** (8–50 days ago): 3 records — timestamp within `now − 8d` to `now − 50d`
   - **Old group** (older than 50 days): 3 records — timestamp older than `now − 50d`
2. The evaluator sends a query with an explicit time filter for 45 days ago:
   `SELECT * FROM claroty.audit_logs WHERE timestamp > '<now_minus_45d_iso8601>' LIMIT 20`
3. The response contains records from BOTH the recent group AND the middle group (records between
   45 days ago and now).
4. The response does NOT contain records from the old group (older than 45 days).
5. The query completes within 5 seconds (bounded by the explicit filter, not full history).

**BDD supplement:**

**Given** the Claroty DTU has audit_log fixtures spanning recent (0–7d), middle (8–50d), and old (>50d) windows  
**And** prism is configured with the S-CLAROTY-AUDITLOG-TIMEBOX-001 TOML spec (dynamic `filter_by` push-down)  
**When** `SELECT * FROM claroty.audit_logs WHERE timestamp > '<now_minus_45d>'` is issued  
**Then** records from both the recent AND middle groups are returned  
**And** old group records (>50d) are NOT returned  
**And** the result is NOT silently bounded to the 7-day default window (recent group only)

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.013 | §Postconditions §1 Claroty `audit_logs` — explicit `start_time` honored without truncation | Core assertion: user's 45d filter is honored, not silently replaced by 7d default |
| BC-2.01.013 | §Claroty `audit_logs` (S-CLAROTY-AUDITLOG-TIMEBOX-001) push-down row | `filter_by` `greater_or_equal` injection for user-supplied lower time bound |

---

## Verification Approach

1. Start the Claroty DTU clone with three temporal groups of audit_log fixtures. The DTU handler for
   `POST /api/v1/audit_log/get` must support `GetAuditLogParameters.filter_by` with a `greater_or_equal`
   timestamp operation, filtering the returned records to those matching the filter.

   Compute `now_minus_45d` as ISO 8601 UTC: `<current_UTC_minus_45d>` (e.g.,
   `2026-07-01T00:00:00Z` if evaluating in August 2026). Seed:
   - Recent: 3 records with `timestamp` in the last 7 days (e.g., `2026-08-10T..Z`)
   - Middle: 3 records with `timestamp` between 8 and 44 days ago (e.g., `2026-07-05T..Z`)
   - Old: 3 records with `timestamp` more than 50 days ago (e.g., `2026-06-20T..Z`)

   Use distinguishable `audit_id` prefixes: `"recent-"`, `"middle-"`, `"old-"`.

2. Start prism MCP stdio with the S-CLAROTY-AUDITLOG-TIMEBOX-001 TOML configuration for Claroty.

3. Send `query` tool call with the SQL: `SELECT * FROM claroty.audit_logs WHERE timestamp > 'YYYY-MM-DDTHH:MM:SSZ' LIMIT 20`
   where the timestamp is `now_minus_45d`.

4. Capture the serialized JSON response.

5. Assert:
   - Response is valid JSON and not an error envelope.
   - Response contains records with `audit_id` starting with `"recent-"` (recent group present).
   - Response contains records with `audit_id` starting with `"middle-"` (middle group present —
     these would be ABSENT if the default 7-day window was incorrectly applied instead of the user's
     explicit filter, which is the key differentiator tested here).
   - Response does NOT contain records with `audit_id` starting with `"old-"` (old group excluded).
   - Total record count is 6 (recent + middle groups only).
   - Query completes in < 5 seconds.
   - Response does not contain `"E-QUERY-004"`.

6. If the response contains only 3 records (recent group only, no middle group), this means the
   7-day default window is overriding the explicit user filter — record as FAIL.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **Push-down correctness** (weight: 0.6): Does the result contain records from the middle group (8–44 days ago)?
  Full credit (1.0): both recent AND middle groups present; old group absent.
  Half credit (0.5): recent and middle groups present but old group also present (upper bound not enforced by DataFusion post-filter — not a push-down failure but an unexpected extra row).
  Zero credit (0.0): only recent group present (7-day default is overriding the explicit filter) OR no records at all.

- **Exclusion correctness** (weight: 0.25): Are old-group records (>50d) excluded?
  Full credit (1.0): zero old-group records. Zero credit (0.0): old-group records present.

- **Timing** (weight: 0.1): Did the query complete in under 5 seconds?
  Full credit (1.0): ≤ 5s. Zero credit (0.0): > 15s.

- **Error absence** (weight: 0.05): Clean response with no error codes.
  Full credit (1.0): no E-QUERY-004 or timeout. Zero credit (0.0): error present.

---

## Edge Conditions

- **7-day default override (regression):** If only the recent group appears, the push-down is not correctly
  substituting the user's explicit filter for the 7-day default. This is the primary failure mode tested.

- **DTU does not support `greater_or_equal` filter operation:** If the DTU returns HTTP 400 or all
  9 records (no filter applied), report SETUP-FAILURE and escalate — DTU must be updated to support
  the push-down `filter_by` operation semantics.

- **ASM-CLAROTY-AUDITLOG-001 (field name):** If the `timestamp` field name is wrong (e.g., `created_at`),
  xDome returns 4xx → E-SENSOR-001. This surfaces as an error response, not a truncated result set.
  Report as SETUP-FAILURE for the ASM validation step.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-AUDITLOG-001-A-003 (satisfaction: X.XX) — claroty.audit_logs query with an explicit older time filter did not return records from the requested historical window; the default bounded window appears to still be overriding the user's explicit filter; verify the filter_by push-down injection in spec_driven_adapter.rs and verify that ADR-033 Option T1 start_time extraction fires for this query pattern"`

Do NOT disclose: the specific fixture groups, the audit_id naming convention, the exact timestamp
threshold, or which group was/wasn't returned.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001-po-bc-amendments | 2026-08-15 | product-owner | Design rework: story reassigned from S-CLAROTY-AUDITLOG-TIMEBOX-002 to S-CLAROTY-AUDITLOG-TIMEBOX-001 (single-story consolidation). ID updated from HS-AUDITLOG-002-B-001 to HS-AUDITLOG-001-A-003. EC-016-013-010 framing removed — this scenario now tests the positive assertion (explicit filter honored) rather than a known-limitation resolution. Layer 1/2 language replaced with push-down correctness language. |
| 1.0 | DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001-po-bc-amendments | 2026-08-15 | product-owner | Initial authoring as Story B scenario. |
