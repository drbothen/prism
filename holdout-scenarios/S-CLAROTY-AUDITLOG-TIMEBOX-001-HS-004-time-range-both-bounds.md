---
document_type: holdout-scenario
level: L3
id: "HS-AUDITLOG-001-A-004"
title: "Claroty audit_logs WHERE timestamp BETWEEN a AND b pushes both bounds — result scoped to [a, b]"
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
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-CLAROTY-AUDITLOG-TIMEBOX-001 — push-down: compound BETWEEN filter pushes both start_time and end_time as filter_by bounds. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-AUDITLOG-001-A-004: Claroty audit_logs WHERE timestamp BETWEEN a AND b pushes both bounds — result scoped to [a, b]

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-AUDITLOG-TIMEBOX-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.013 §Postconditions §1 Claroty `audit_logs` — "compound `and` filter: `greater_or_equal` for lower bound + `less_or_equal` for upper bound"; BC-2.01.013 §Claroty `audit_logs` (S-CLAROTY-AUDITLOG-TIMEBOX-001) row — `less_or_equal` compound clause when `end_time` present
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the compound `and` filter behavior: a `WHERE timestamp BETWEEN a AND b`
query (or equivalent `WHERE timestamp >= a AND timestamp <= b`) must push BOTH bounds into the `filter_by`
request body — a `greater_or_equal` clause for the lower bound and a compound `and` with `less_or_equal`
for the upper bound. Records outside the `[a, b]` window must not appear in the result.

This exercises the compound-filter path described in BC-2.01.013 §Claroty `audit_logs` (S-CLAROTY-AUDITLOG-TIMEBOX-001):
"compound `and` filter with `less_or_equal` when `end_time` present." Without the upper bound push-down,
a `WHERE timestamp BETWEEN a AND b` query would return all records from `a` to "now", not just the `[a, b]` window.

**Behavioral assertions:**

1. The Claroty DTU clone is running with audit log fixtures in three groups:
   - **Before window** (older than `a`): 3 records timestamped before the query window start.
   - **In window** (between `a` and `b`): 3 records timestamped within `[a, b]`.
   - **After window** (newer than `b` but older than "now"): 3 records timestamped after `b` but
     before DTU "now" (to test that the upper bound is enforced).
2. The evaluator sends a query:
   `SELECT * FROM claroty.audit_logs WHERE timestamp >= '<a_iso8601>' AND timestamp <= '<b_iso8601>' LIMIT 20`
   where `[a, b]` is a specific historical window that:
   - `a` is 30 days ago (well outside the 7-day Layer 1 default)
   - `b` is 20 days ago (also outside the 7-day Layer 1 default, but newer than `a`)
3. The response contains ONLY records from the "in window" group (3 records).
4. Records from "before window" and "after window" groups are NOT present.
5. The query completes within 5 seconds.

**BDD supplement:**

**Given** the Claroty DTU has audit_log fixtures spanning before, within, and after the `[30d, 20d]` query window  
**And** prism is configured with Layer 2 (compound `filter_by` push-down)  
**When** `SELECT * FROM claroty.audit_logs WHERE timestamp >= '<30d_ago>' AND timestamp <= '<20d_ago>'` is issued  
**Then** only records within the `[30d, 20d]` window are returned  
**And** records older than 30 days are excluded (lower bound enforced)  
**And** records newer than 20 days are excluded (upper bound enforced)  
**And** the query completes within 5 seconds

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.013 | §Postconditions §1 Claroty `audit_logs` — compound `and less_or_equal` clause | Upper bound push-down: records after `b` excluded |
| BC-2.16.013 | §Postconditions §1 Claroty `audit_logs` — `greater_or_equal` lower bound | Lower bound push-down: records before `a` excluded |
| BC-2.01.013 | §Claroty `audit_logs` (S-CLAROTY-AUDITLOG-TIMEBOX-001) row — "compound `and` filter with `less_or_equal` when `end_time` present" | The compound filter mechanism |

---

## Verification Approach

1. Compute:
   - `a` = ISO 8601 timestamp for DTU-simulated `now − 30d` (e.g., `2026-07-16T12:00:00Z`)
   - `b` = ISO 8601 timestamp for DTU-simulated `now − 20d` (e.g., `2026-07-26T12:00:00Z`)

2. Start the Claroty DTU clone with fixtures:
   - Before window: 3 records with `timestamp < a` (e.g., `2026-07-10T12:00:00Z`); `audit_id` prefix `"before-"`
   - In window: 3 records with `a ≤ timestamp ≤ b` (e.g., `2026-07-20T12:00:00Z`); `audit_id` prefix `"inwindow-"`
   - After window: 3 records with `b < timestamp ≤ now` (e.g., `2026-08-05T12:00:00Z`); `audit_id` prefix `"after-"`

   The DTU handler must support compound `filter_by` with `and` combining `greater_or_equal` and
   `less_or_equal` on the `timestamp` field.

3. Start prism MCP stdio with the S-CLAROTY-AUDITLOG-TIMEBOX-001 TOML configuration.

4. Send `query` tool call:
   `{"sql": "SELECT * FROM claroty.audit_logs WHERE timestamp >= '<a>' AND timestamp <= '<b>' LIMIT 20"}`

5. Capture the serialized JSON response.

6. Assert:
   - Response is valid JSON and not an error envelope.
   - Response contains exactly 3 records, all with `audit_id` starting with `"inwindow-"`.
   - Response does NOT contain any `audit_id` starting with `"before-"` (lower bound enforced).
   - Response does NOT contain any `audit_id` starting with `"after-"` (upper bound enforced).
   - Query completes in < 5 seconds.
   - Response does not contain `"E-QUERY-004"`.

7. Failure signatures:
   - **Only "inwindow" records BUT "after" records also present:** Upper bound (`less_or_equal`) not injected.
   - **Only recent records (Layer 1 truncation):** Neither bound was pushed down; fallback to static default.
   - **All 9 records returned:** DTU ignores `filter_by`; escalate as SETUP-FAILURE.
   - **"before" records also present:** Lower bound (`greater_or_equal`) not injected.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **Compound bounds correctness** (weight: 0.7): Are BOTH bounds enforced correctly?
  Full credit (1.0): only in-window records; before and after groups absent.
  Half credit (0.5): lower bound enforced but upper bound not (after-window records present).
  Quarter credit (0.25): upper bound enforced but lower bound not (before-window records present).
  Zero credit (0.0): only records matching the default 7-day window (bounds not pushed down) OR all 9 records (no filtering at all).

- **Timing** (weight: 0.2): Did the query complete in under 5 seconds?
  Full credit (1.0): ≤ 5s. Zero credit (0.0): > 15s.

- **Error absence** (weight: 0.1): Clean response with no error codes.
  Full credit (1.0): no E-QUERY-004 or timeout. Zero credit (0.0): error present.

---

## Edge Conditions

- **Layer 2 only injects lower bound (missing `end_time` extraction):** If ADR-033 Option T1 extracts
  `start_time` but not `end_time` from the WHERE clause, the upper bound is absent from the `filter_by`
  body. The after-window records appear in the result. Failure signal: after-group records present.

- **Both bounds extracted but only one injected:** If `spec_driven_adapter.rs` or `pipeline.rs`
  drops the `end_time` → `less_or_equal` compound clause, the upper bound is not enforced.

- **DTU does not support compound `and` filter:** If the DTU returns HTTP 400 for the compound filter
  body, record as SETUP-FAILURE and escalate — DTU must be extended to support compound `and` filters.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-AUDITLOG-001-A-004 (satisfaction: X.XX) — claroty.audit_logs BETWEEN query did not correctly scope the result to the requested window; check that both the lower and upper time bounds are being pushed down into the filter_by body in spec_driven_adapter.rs"`

Do NOT disclose: the specific fixture groups, the audit_id naming convention, the exact window
timestamps, or which failure signature was observed.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001-po-bc-amendments | 2026-08-15 | product-owner | Design rework: story reassigned from S-CLAROTY-AUDITLOG-TIMEBOX-002 to S-CLAROTY-AUDITLOG-TIMEBOX-001 (single-story consolidation). ID updated from HS-AUDITLOG-002-B-002 to HS-AUDITLOG-001-A-004. Layer 2 language replaced with push-down correctness language. Failure message ID updated. |
| 1.0 | DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001-po-bc-amendments | 2026-08-15 | product-owner | Initial authoring. Story-level holdout gate for S-CLAROTY-AUDITLOG-TIMEBOX-002 Layer 2 — compound BETWEEN bounds push-down. SINGLE-USE. |
