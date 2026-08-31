---
document_type: holdout-scenario
level: L3
id: "HS-OTEVTS-001-003"
title: "claroty_ot_activity_events: SELECT time (Tier-1 OCSF name for detection_time) succeeds; SELECT detection_time raises E-QUERY-038"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-A"
story_source: "S-CLAROTY-OT-EVENTS-001"
version: "1.1"
status: active
used: true
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-31"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.016-claroty-ot-activity-events-table.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "868ad40"
traces_to: "BC-2.16.016"
behavioral_contracts:
  - BC-2.16.016
  - BC-2.11.016
verification_properties: []
lifecycle_status: consumed
introduced: "S-CLAROTY-OT-EVENTS-001"
last_evaluated: "2026-08-31"
last_eval_satisfaction: 1.00
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-OT-EVENTS-001 (HS-025 group). Tests BC-2.16.016 §Postconditions 2 Tier-1: detection_time → ocsf_field = 'time' → Arrow field 'time'. SELECT time succeeds; SELECT detection_time raises E-QUERY-038. Validates datetime Tier-1 column is exposed under OCSF Arrow name ('time') not raw TOML name ('detection_time'). ADR-058 Tier-1 rename + BC-2.11.016 E-QUERY-038. No DTU — live sensor only. BLOCKING. Test-writer and implementer must NOT read this file. CONSUMED D-2399 2026-08-31: PASS 1.00 — SELECT time succeeded (plan gate accepted OCSF Arrow name); SELECT detection_time raised E-QUERY-038 with time in available_columns. Human ACCEPTED (Option-1). SINGLE-USE consumed; must NOT be reused."
---

# HS-OTEVTS-001-003: claroty_ot_activity_events: SELECT time (Tier-1 OCSF name for detection_time) succeeds; SELECT detection_time raises E-QUERY-038

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-OT-EVENTS-001 (HS-025 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.016 §Postconditions 2 Tier-1 column `detection_time` → `ocsf_field = "time"` → Arrow field `time`; BC-2.11.016 E-QUERY-038 on raw TOML col.name
**Gate:** Story-level holdout gate (HS-025) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

The `claroty_ot_activity_events` table declares a Tier-1 column:
- TOML name: `detection_time`
- `column_type = "Datetime"`
- `ocsf_field = "time"`
- Arrow field name: `time` (result of `ocsf_field_to_arrow_name("time")`)

Under `ocsf_column_naming = true`, the plan-gate column registry must expose this column as `time`, NOT as `detection_time`. Querying by the OCSF Arrow field name `time` must succeed; querying by the raw TOML column name `detection_time` must raise E-QUERY-038.

This is a particularly important validation for datetime Tier-1 columns: `time` is the standard OCSF temporal anchor for events. If it is not exposed under `time`, any downstream OCSF-aware tool or temporal query using `time` will silently fail.

This scenario validates:

1. **Positive:** `SELECT time FROM claroty.claroty_ot_activity_events LIMIT 5` succeeds at the plan gate. Rows are returned (or a non-E-QUERY-038 error occurs). The `time` column is accessible.

2. **Negative complement:** `SELECT detection_time FROM claroty.claroty_ot_activity_events LIMIT 1` raises E-QUERY-038. The raw TOML datetime column name is hidden under `ocsf_column_naming = true`. The `available_columns` in the error payload contains `time` (the correct OCSF Arrow field name).

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT time FROM claroty.claroty_ot_activity_events LIMIT 5` is issued via the MCP `query` tool
**Then** the response is NOT an E-QUERY-038 error
**When** `SELECT detection_time FROM claroty.claroty_ot_activity_events LIMIT 1` is issued via the MCP `query` tool
**Then** the response IS an E-QUERY-038 error
**And** the `available_columns` field contains `time` but NOT `detection_time`

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 — do not log the credential value).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Wait for prism to be ready.

5. Issue first MCP `query` tool call: `{"sql": "SELECT time FROM claroty.claroty_ot_activity_events LIMIT 5"}`. Capture the full response.

6. Issue second MCP `query` tool call: `{"sql": "SELECT detection_time FROM claroty.claroty_ot_activity_events LIMIT 1"}`. Capture the full response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.016 | §Postconditions 2 Tier-1: detection_time → ocsf_field = "time" → Arrow field time | Assertion 1: SELECT time is accepted at plan gate |
| BC-2.11.016 | E-QUERY-038: raw Tier-1 TOML col.name detection_time not in plan-gate registry; time is | Assertion 2: SELECT detection_time raises E-QUERY-038 with time in available_columns |
| ADR-058 | ocsf_field_to_arrow_name("time") = "time"; raw col.name hidden from plan gate | Both assertions confirm the OCSF datetime column rename |

---

## Verification Approach

1. Parse response from first query (`SELECT time`).
   - If E-QUERY-038: record FAIL on "time queryable" dimension — OCSF Arrow name not registered at plan gate.
   - If any other error (sensor unreachable, auth): PASS on plan-gate dimension.
   - If success with rows: PASS; note whether `time` column values are ISO 8601 datetime strings or null.

2. Parse response from second query (`SELECT detection_time`).
   - If no E-QUERY-038: record FAIL on "detection_time rejected" dimension — raw col.name incorrectly exposed.
   - If E-QUERY-038: inspect `available_columns`.
     - Assert `time` is present in available_columns.
     - Assert `detection_time` is NOT present in available_columns.
     - Record PASS or PARTIAL based on available_columns completeness.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **time queryable at plan gate (no E-QUERY-038)** (weight: 0.40): Does `SELECT time` return a non-E-QUERY-038 response?
  Full credit (1.0): non-E-QUERY-038 (plan gate accepted `time` Arrow field name).
  Zero credit (0.0): E-QUERY-038 — datetime Tier-1 column not registered under OCSF name.

- **detection_time (raw col.name) rejected by E-QUERY-038** (weight: 0.30): Does `SELECT detection_time` raise E-QUERY-038?
  Full credit (1.0): E-QUERY-038 fires.
  Zero credit (0.0): no E-QUERY-038 — raw datetime col.name incorrectly exposed.

- **available_columns contains time but NOT detection_time** (weight: 0.30): When E-QUERY-038 fires on detection_time, is available_columns correct?
  Full credit (1.0): time present, detection_time absent.
  Partial credit (0.5): E-QUERY-038 fires but available_columns incomplete (time missing, or detection_time present).
  Zero credit (0.0): E-QUERY-038 did not fire — dimension cannot be scored.

---

## Edge Conditions

- **Both time and detection_time raise E-QUERY-038:** Indicates the entire `claroty_ot_activity_events` table column set was not registered at the plan gate. Score zero on all time-related dimensions.

- **time query returns rows where time column is null for all rows:** PASS on plan-gate dimension (column is registered; null values are valid per BC-2.16.016 EC-016-016-003 null passthrough). Note observation.

- **E-QUERY-038 fires on detection_time but available_columns contains both `time` and `detection_time`:** PARTIAL (0.5) — rejection fires correctly but available_columns exposes the raw name it should hide.

- **Live sensor is unreachable for time query:** PASS on plan-gate dimension (the E-QUERY-038 plan gate fires BEFORE any HTTP fetch; unreachable sensor means the plan gate accepted the column and the execution failed at network layer, which is correct).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-OTEVTS-001-003 (satisfaction: X.XX) — claroty_ot_activity_events Tier-1 datetime column plan-gate gap; check ocsf_field_to_arrow_name applied for detection_time→time Tier-1 column at plan-gate column registry (BC-2.16.016 §Postconditions 2 Tier-1; BC-2.11.016 E-QUERY-038 available_columns; ADR-058 Tier-1 rename)"`

Do NOT disclose: the specific column names queried, the LIMIT values, or the exact assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/ot_activity_events/ (plan-gate assertions — content irrelevant for E-QUERY-038 dimensions) |
| corpus_size | LIMIT 5 for positive (datetime column shape); LIMIT 1 for negative (plan-gate rejection) |
| known_edge_cases | Null detection_time values in live sensor (EC-016-016-003 null passthrough — PASS on plan-gate dimension) |
| false_positive_threshold | Zero: plan-gate rejection is a structural assertion |
| false_negative_threshold | Zero: absence of E-QUERY-038 on detection_time means raw col.name incorrectly exposed |

**Known-good corpus:** Correctly implemented `claroty_ot_activity_events` with detection_time → ocsf_field = "time" → Arrow `time` registered — expected: SELECT time succeeds; SELECT detection_time raises E-QUERY-038 with time in available_columns.

**Known-problematic corpus:** An implementation that registers the raw TOML column name (detection_time) instead of the OCSF Arrow name (time) — expected: SELECT time raises E-QUERY-038 (OCSF name not found); SELECT detection_time succeeds. This is the ADR-058 rename application failure this scenario catches.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | G2-live-holdout-gate | 2026-08-31 | state-manager | Evaluated D-2399: PASS 1.00 (consumed). SELECT time succeeded at plan gate (no E-QUERY-038 — OCSF Arrow name registered); SELECT detection_time raised E-QUERY-038 with time in available_columns (raw col.name hidden). Human ACCEPTED (Option-1): holdout gate PASSED. used→true; lifecycle_status→consumed; last_evaluated 2026-08-31; last_eval_satisfaction 1.00. SINGLE-USE consumed; must NOT be reused. |
| 1.0 | xdome-wave-a-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-025 group for S-CLAROTY-OT-EVENTS-001. Tier-1 datetime column OCSF rename: SELECT time (OCSF Arrow name) succeeds; SELECT detection_time (raw TOML name) raises E-QUERY-038 with time in available_columns. BC-2.16.016 §Postconditions 2 Tier-1 + BC-2.11.016 + ADR-058. No DTU — live sensor only. SINGLE-USE. |
