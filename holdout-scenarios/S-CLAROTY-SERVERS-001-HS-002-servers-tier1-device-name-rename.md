---
document_type: holdout-scenario
level: L3
id: "HS-SERVERS-001-002"
title: "claroty_servers Tier-1 rename enforcement: SELECT server_name rejected E-QUERY-038, SELECT device_name and status_code accepted"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-C"
story_source: "S-CLAROTY-SERVERS-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-24"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.018-claroty-servers-table.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "9311257"
traces_to: "BC-2.16.018"
behavioral_contracts:
  - BC-2.16.018
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-SERVERS-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-SERVERS-001 (HS-027 group). Tests BC-2.16.018 §Postconditions 2 Tier-1 plan-gate: the raw col.name 'server_name' must be rejected with E-QUERY-038 because the Arrow field name is 'device_name' (ocsf_field_to_arrow_name transform). Selecting 'device_name' and 'status_code' must succeed. Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-SERVERS-001-002: claroty_servers Tier-1 rename enforcement: SELECT server_name rejected E-QUERY-038, SELECT device_name and status_code accepted

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-SERVERS-001 (HS-027 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.018 §Postconditions 2 Tier-1 plan-gate (the Tier-1 rename plan-gate must reject raw `col.name` references; the Arrow field name `device_name` from `ocsf_field_to_arrow_name("device.name")` must be accepted; `status_code` must also be accepted) and §Invariants (`ocsf_column_naming = true` — raw col.name for renamed Tier-1 fields is invalid at query time; available_columns in E-QUERY-038 response must list Arrow names)
**Gate:** Story-level holdout gate (HS-027) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the `claroty_servers` table enforces the Tier-1 OCSF column rename
at query time, per ADR-058 and BC-2.16.018 §Postconditions 2:

1. A query selecting `server_name` (the raw `col.name` from the TOML spec) must be rejected with
   error code E-QUERY-038 ("column not found; use OCSF Arrow name"). This is the plan-gate
   enforcement for `ocsf_column_naming = true`: once a column has `ocsf_field = "device.name"`,
   its Arrow field name is `device_name` (via `ocsf_field_to_arrow_name`), and the raw name
   `server_name` is not exposed at the query surface.

2. The E-QUERY-038 error response must include an `available_columns` list that contains
   `device_name` — evidence that the system knows the correct Arrow name for the renamed column.

3. A query selecting `device_name` (the Arrow field name) must succeed with at least one non-null
   value. This confirms the transform was applied and the column is accessible under its correct name.

4. A query selecting `status_code` (the second Tier-1 mapped column, from `server_status`) must
   also succeed with a non-null value. This confirms both Tier-1 mappings were applied.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured
**When** `SELECT server_name FROM claroty.claroty_servers LIMIT 1` is issued via the MCP `query` tool
**Then** the response is an E-QUERY-038 error
**And** the error response includes `available_columns` listing `device_name`
**When** `SELECT device_name FROM claroty.claroty_servers LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response contains a non-null `device_name` value
**When** `SELECT status_code FROM claroty.claroty_servers LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response contains a non-null `status_code` value

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 —
   reference the key by name, do not include the credential value in any output).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Issue three sequential MCP `query` tool calls as described in the Verification Approach below.
   Capture the full raw wire-level JSON response for each call.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.018 | §Postconditions 2 Tier-1 plan-gate: server_name rejected (not an Arrow field name) | Call 1: SELECT server_name → E-QUERY-038 |
| BC-2.16.018 | §Invariants: available_columns in E-QUERY-038 lists Arrow names, not raw col.names | Call 1: E-QUERY-038 response contains device_name in available_columns |
| BC-2.16.018 | §Postconditions 2 Tier-1: ocsf_field_to_arrow_name("device.name") → device_name REQUIRED | Call 2: SELECT device_name succeeds |
| BC-2.16.018 | §Postconditions 2 Tier-1: server_status → status_code | Call 3: SELECT status_code succeeds |

---

## Verification Approach

**Call 1:** Issue `SELECT server_name FROM claroty.claroty_servers LIMIT 1`.

- Parse the wire-level response. Assert it is an error response (not a rows response).
- Assert the error code is E-QUERY-038 (or equivalent column-not-found error). If the response
  is NOT an error — i.e., a row with a `server_name` column was returned — record FAIL on the
  "Tier-1 rename rejection" dimension.
- If it is an E-QUERY-038, inspect the error payload for an `available_columns` field. Assert
  it contains `device_name` as one of the listed available columns.

**Call 2:** Issue `SELECT device_name FROM claroty.claroty_servers LIMIT 1`.

- Parse the wire-level response. Assert it is a non-error rows response.
- Assert at least one row contains `device_name` with a non-null, non-empty string value.
- If the query errors, record FAIL on the "device_name query success" dimension.

**Call 3:** Issue `SELECT status_code FROM claroty.claroty_servers LIMIT 1`.

- Parse the wire-level response. Assert it is a non-error rows response.
- Assert at least one row contains `status_code` with a non-null value (string or integer).
- If the query errors, record FAIL on the "status_code query success" dimension.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **server_name rejected with E-QUERY-038** (weight: 0.40): Does the raw col.name SELECT fail?
  Full credit (1.0): query returns E-QUERY-038 or equivalent column-not-found error.
  Partial credit (0.3): query errors for a different reason (wrong error code but still rejected).
  Zero credit (0.0): query returns a row with a server_name column (rename not enforced).

- **available_columns lists device_name in E-QUERY-038 response** (weight: 0.20): Does the rejection
  suggest the correct Arrow name?
  Full credit (1.0): available_columns present in error payload and contains "device_name".
  Partial credit (0.5): available_columns present but does not contain "device_name" (wrong suggestion).
  Zero credit (0.0): available_columns absent from error payload OR server_name query did not error.

- **device_name SELECT succeeds** (weight: 0.25): Does the Arrow field name resolve?
  Full credit (1.0): non-error response with non-null device_name value.
  Partial credit (0.3): non-error response but device_name is null.
  Zero credit (0.0): query errors (ocsf_field_to_arrow_name transform not applied or column not registered).

- **status_code SELECT succeeds** (weight: 0.15): Does the second Tier-1 column resolve?
  Full credit (1.0): non-error response with non-null status_code value.
  Partial credit (0.3): non-error response but status_code is null.
  Zero credit (0.0): query errors.

---

## Edge Conditions

- **Live sensor returns no rows:** If `device_name` or `status_code` selects return empty result
  sets (zero rows), score those dimensions as SETUP-FAILURE rather than FAIL — the rejection
  dimension (server_name) is still scoreable from the error response alone.

- **Sensor authentication failure (E-SENSOR-001 / 401):** Record as SETUP-FAILURE on all
  non-rejection dimensions.

- **E-QUERY-038 error code variant:** If the implementation uses a different error code identifier
  for column-not-found (e.g., "COLUMN_NOT_FOUND" string rather than "E-QUERY-038"), check whether
  the error semantics match (column explicitly not found). Do not award zero credit solely due to
  code naming if the behavior is correct — note the discrepancy as an observation.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-SERVERS-001-002 (satisfaction: X.XX) — claroty_servers Tier-1 rename plan-gate gap; check ocsf_column_naming=true enforcement in plan-gate (BC-2.16.018 §Postconditions 2 Tier-1): raw col.name must resolve to Arrow name at query surface, raw name rejected with E-QUERY-038 and available_columns listing Arrow name"`

Do NOT disclose: which column name triggered the rejection, which SELECT succeeded, or the exact
assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/servers/ |
| corpus_size | Three LIMIT 1 queries (error path + two success paths) |
| known_edge_cases | Empty result set on success queries — scored as SETUP-FAILURE, not behavioral FAIL |
| false_positive_threshold | Zero: E-QUERY-038 on server_name and non-error on device_name are definitive behavioral assertions |
| false_negative_threshold | Zero: if server_name resolves to a row, the Tier-1 rename enforcement is not working |

**Known-good corpus:** A correctly-implemented claroty_servers table with `ocsf_field = "device.name"`
and `ocsf_column_naming = true` — expected: server_name rejected, device_name and status_code resolve.

**Known-problematic corpus:** An implementation that registered server_name without applying
ocsf_field_to_arrow_name — expected: server_name resolves (wrong), device_name is unregistered (wrong).
This is the exact failure BC-2.16.018 §Postconditions 2 Tier-1 guards against.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-027 group for S-CLAROTY-SERVERS-001. Tests ADR-058 Tier-1 rename enforcement at query surface. BC-2.16.018 §Postconditions 2 Tier-1 plan-gate. SINGLE-USE. |
