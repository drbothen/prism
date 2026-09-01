---
document_type: holdout-scenario
level: L3
id: "HS-SERVERS-001-003"
title: "claroty_server_interfaces: separate endpoint queryable, class_uid=5001, interface_status rejected E-QUERY-038 with status_code in available_columns, raw_extensions contains interface_name and interface_type"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-C"
story_source: "S-CLAROTY-SERVERS-001"
version: "1.1"
status: active
used: true
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-09-01"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.019-claroty-server-interfaces-table.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "700d52d"
traces_to: "BC-2.16.019"
behavioral_contracts:
  - BC-2.16.019
verification_properties: []
lifecycle_status: consumed
introduced: "S-CLAROTY-SERVERS-001"
last_evaluated: 2026-09-01
last_eval_satisfaction: 1.00
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-SERVERS-001 (HS-027 group). Tests BC-2.16.019: server_interfaces is a SEPARATE endpoint (/api/v1/server_interfaces/), table is registered and queryable, class_uid=5001 in wire output, interface_status (raw col.name) rejected with E-QUERY-038 + status_code in available_columns, SELECT raw_extensions shows interface_name and interface_type as Tier-2 JSON keys. Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-SERVERS-001-003: claroty_server_interfaces: separate endpoint queryable, class_uid=5001, interface_status rejected E-QUERY-038 with status_code in available_columns, raw_extensions contains interface_name and interface_type

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-SERVERS-001 (HS-027 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.019 §Postconditions 1 (separate endpoint POST `/api/v1/server_interfaces/`
registered as `claroty_server_interfaces`; ocsf_class = "inventory_info" → class_uid 5001),
§Postconditions 2 Tier-1 plan-gate (interface_status raw col.name rejected; status_code Arrow name
accepted), §Postconditions 2 Tier-2 (raw_extensions JSON string with interface-specific keys), and
§Postconditions 3 Composite PK (server_name + interface_name uniquely identify rows)
**Gate:** Story-level holdout gate (HS-027) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the `claroty_server_interfaces` table, covering four behavioral dimensions:

1. The table is independently queryable via `claroty.claroty_server_interfaces` — confirming
   that the separate `/api/v1/server_interfaces/` endpoint (distinct from `/api/v1/servers/`)
   was correctly registered in the TOML spec as a separate `[[tables]]` block.

2. The wire output carries `class_uid = 5001` — confirming the `inventory_info` class selector
   arm is used (same class as `claroty_servers`, appropriate for network interface inventory).

3. The raw col.name `interface_status` is rejected at query time with E-QUERY-038, and the error
   response's `available_columns` includes `status_code` — confirming the Tier-1 rename
   `interface_status → status_code` is enforced at the plan gate.

4. A `SELECT raw_extensions FROM claroty_server_interfaces` query returns a JSON-serialized
   STRING (Arrow Utf8; per ADR-058 §I2, raw_extensions is physically stored as an Arrow Utf8
   column and emitted on the wire as a serialized JSON string). When parsed via
   serde_json::from_str, the resulting object contains at least one of the expected Tier-2
   interface keys (`interface_name`, `interface_type`, `interface_connection_type`) — confirming
   the 8 Tier-2 columns are aggregated correctly. Note: D-2381's native-JSON rule applies only
   to values INSIDE the raw_extensions object, NOT to the raw_extensions container itself.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured
**When** `SELECT * FROM claroty.claroty_server_interfaces LIMIT 1` is issued
**Then** the response is not an error
**And** the response wire JSON contains a row with `class_uid` equal to `5001`
**When** `SELECT interface_status FROM claroty.claroty_server_interfaces LIMIT 1` is issued
**Then** the response is an E-QUERY-038 error
**And** the error response `available_columns` includes `status_code`
**When** `SELECT raw_extensions FROM claroty.claroty_server_interfaces LIMIT 1` is issued
**Then** the response is not an error
**And** the `raw_extensions` value is a non-null JSON-serialized string (Arrow Utf8) whose parsed object contains at least one interface Tier-2 key (ADR-058 §I2)

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 —
   reference by key name only, not credential value).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Issue three sequential MCP `query` tool calls as described in the Verification Approach below.
   Capture the full raw wire-level JSON response for each call.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.019 | §Postconditions 1: POST /api/v1/server_interfaces/, table registered as claroty_server_interfaces | Call 1: table queryable at all (separate endpoint registration) |
| BC-2.16.019 | §Postconditions 1: ocsf_class = "inventory_info" → class_uid 5001 | Call 1: class_uid = 5001 in wire output |
| BC-2.16.019 | §Postconditions 2 Tier-1: interface_status → ocsf_field = "status_code" → Arrow status_code; raw col.name rejected | Call 2: SELECT interface_status → E-QUERY-038 with status_code in available_columns |
| BC-2.16.019 | §Postconditions 2 Tier-2: 8 Tier-2 columns aggregate into raw_extensions JSON string (Arrow Utf8) | Call 3: SELECT raw_extensions returns a JSON-serialized string whose parsed object contains interface Tier-2 keys (ADR-058 §I2) |
| BC-2.16.019 | §Postconditions 3: Composite PK (server_name → device_name, interface_name) | Validated structurally by Tier-2 raw_extensions containing interface_name key |

---

## Verification Approach

**Call 1:** Issue `SELECT * FROM claroty.claroty_server_interfaces LIMIT 1`.

- Parse the wire-level response. Assert it is a non-error rows response with at least one row.
  If the response is "table not found" or E-QUERY-038 on the table itself, record FAIL on the
  "separate endpoint registered" dimension — the TOML table block for server_interfaces was not
  added or used the wrong endpoint path.
- Inspect the first row for a `class_uid` column. Assert its value equals integer `5001`.
  If class_uid is absent or has a wrong value, record FAIL on the "class_uid=5001" dimension.

**Call 2:** Issue `SELECT interface_status FROM claroty.claroty_server_interfaces LIMIT 1`.

- Parse the wire-level response. Assert it is an error response (E-QUERY-038 or equivalent).
  If a row with `interface_status` column is returned, record FAIL on the "Tier-1 rename rejection"
  dimension — the rename enforcement is missing.
- If the error is E-QUERY-038, inspect the error payload's `available_columns`. Assert it
  contains `status_code`. If `available_columns` is absent or does not contain `status_code`,
  record PARTIAL on the "available_columns suggests correct name" dimension.

**Call 3:** Issue `SELECT raw_extensions FROM claroty_server_interfaces LIMIT 1`.

- Parse the wire-level response. Assert it is a non-error rows response.
- Inspect the first row's `raw_extensions` column. Assert it is a non-null JSON-serialized STRING
  (Arrow Utf8). Per ADR-058 §I2, raw_extensions is physically stored as an Arrow Utf8 column and
  emitted on the wire as a serialized JSON string — a string value here is CORRECT, not a defect.
  D-2381's native-JSON rule governs only values inside the raw_extensions object, not the container
  itself. Parse the string value via serde_json::from_str and assert the parsed object contains at
  least one of: `interface_name`, `interface_type`, `interface_connection_type`, `site_id`,
  `avg_traffic_past_month_mbps`, `avg_traffic_past_week_mbps`, `avg_traffic_past_hour_mbps`,
  or `notes`.
- If `raw_extensions` is absent or null, record FAIL on the "Tier-2 aggregation into raw_extensions"
  dimension. If raw_extensions is present but is a native JSON object (not a string), record
  PARTIAL (0.5) — structurally wrong per ADR-058 §I2 but the data is present.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Separate endpoint registered (table queryable)** (weight: 0.25): Does `claroty_server_interfaces`
  exist as a distinct queryable table?
  Full credit (1.0): non-error SELECT * response with ≥1 row.
  Partial credit (0.3): response is an error for a non-table-registration reason (auth / empty).
  Zero credit (0.0): "table not found" — the TOML block for server_interfaces was not added.

- **class_uid = 5001 in wire output** (weight: 0.25): Does the table emit inventory_info class rows?
  Full credit (1.0): class_uid column present, value is integer 5001.
  Partial credit (0.3): class_uid present but wrong value.
  Zero credit (0.0): class_uid absent or table not registered.

- **interface_status rejected with E-QUERY-038** (weight: 0.30): Is the Tier-1 rename enforced?
  Full credit (1.0): query returns E-QUERY-038 or equivalent column-not-found error.
  Partial credit (0.3): error returned but wrong code or wrong reason.
  Zero credit (0.0): query returns a row with interface_status column (rename not enforced).

- **available_columns suggests status_code** (weight: 0.10): Does the error guide the user to the
  correct column name?
  Full credit (1.0): available_columns present in E-QUERY-038 payload and contains "status_code".
  Partial credit (0.5): available_columns present but does not include "status_code".
  Zero credit (0.0): available_columns absent, or interface_status did not trigger E-QUERY-038.

- **raw_extensions Tier-2 aggregation** (weight: 0.10): Are Tier-2 columns aggregated correctly?
  Per ADR-058 §I2, raw_extensions is an Arrow Utf8 column emitted as a JSON-serialized string on
  the wire (not a native JSON object).
  Full credit (1.0): raw_extensions is a non-null string; parsed object contains at least one
  interface Tier-2 key.
  Partial credit (0.5): raw_extensions present but is an empty string or empty parsed object.
  Zero credit (0.0): raw_extensions absent or null.

---

## Edge Conditions

- **Live sensor returns empty result set (zero rows on Call 1):** Score "separate endpoint registered"
  as PARTIAL (0.3) if the query returns a non-error empty result — the table is registered but
  there are no interfaces in the xDome instance. This is a SETUP condition, not a behavioral FAIL.

- **Sensor authentication failure (E-SENSOR-001 / 401):** Record as SETUP-FAILURE on all dimensions
  except the E-QUERY-038 plan-gate dimension (which can still be tested without live sensor data
  returning rows — if the table is registered, the column rename plan-gate fires before the sensor
  is called).

- **`interface_status` rejected for a different reason than column-not-found (e.g., syntax error):**
  Award partial credit on rejection dimension only if the actual column-not-found semantics are present
  even if the error code differs. Note the code discrepancy as an observation.

- **Multiple rows returned on Calls 2-3 with LIMIT 1 override from sensor:** Inspect the first row
  only. Do not fail on extra rows from the sensor returning all results despite LIMIT.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-SERVERS-001-003 (satisfaction: X.XX) — claroty_server_interfaces gap; check [dimension(s) failed]: table registration using separate /api/v1/server_interfaces/ endpoint (BC-2.16.019 §Postconditions 1), OCSF class_uid=5001 mapping, Tier-1 interface_status → status_code plan-gate enforcement (BC-2.16.019 §Postconditions 2), or Tier-2 raw_extensions aggregation (BC-2.16.019 §Postconditions 2)"`

Do NOT disclose: which specific column name was selected in Call 2, which Tier-2 keys were searched
for in Call 3, or the exact assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/server_interfaces/ |
| corpus_size | Three LIMIT 1 queries: SELECT *, SELECT interface_status (error path), SELECT raw_extensions |
| known_edge_cases | Empty result set (no server interfaces in xDome instance) — Call 1 scores as PARTIAL not FAIL |
| false_positive_threshold | Low: class_uid=5001 and E-QUERY-038 on interface_status are structural/behavioral assertions not data-dependent |
| false_negative_threshold | Zero: if interface_status resolves to a row, Tier-1 rename enforcement is broken |

**Known-good corpus:** A correctly-implemented claroty_server_interfaces table using the separate
`/api/v1/server_interfaces/` endpoint with `ocsf_field = "status_code"` on `interface_status` —
expected: class_uid=5001, interface_status rejected, status_code and raw_extensions resolve as a
JSON-serialized string (Arrow Utf8) whose parsed object contains interface Tier-2 keys.

**Known-problematic corpus A (endpoint confusion):** An implementation that registered server_interfaces
under the wrong endpoint (e.g., routing through `/api/v1/servers/`) — expected: zero rows or
wrong data shape in Call 1 output.

**Known-problematic corpus B (rename not applied):** An implementation that registered `interface_status`
without the `ocsf_field = "status_code"` or without `ocsf_column_naming = true` — expected:
Call 2 returns a row with `interface_status` column (failure mode BC-2.16.019 §Postconditions 2 guards against).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | G4-holdout-raw-extensions-correction | 2026-08-31 | product-owner | Corrected raw_extensions wire-encoding expectation in Call 3 verification (architect verdict-A, mirrors G3 HS-002 / D-2403 precedent). Per ADR-058 §I2, raw_extensions is an Arrow Utf8 column emitted as a JSON-serialized string on the wire — not a native JSON object. Removed "not a bare string" assertion from Scenario §4 and Verification Approach Call 3. Updated BDD supplement, Behavioral Contract Linkage, and Evaluation Rubric to expect a non-null JSON-serialized string whose parsed object contains interface Tier-2 keys. D-2381 native-JSON rule governs only values inside raw_extensions, not the container itself. Used bare table name in Call 3 SQL example. |
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-027 group for S-CLAROTY-SERVERS-001. Tests BC-2.16.019: separate /api/v1/server_interfaces/ endpoint registration, class_uid=5001, interface_status → status_code Tier-1 plan-gate, raw_extensions Tier-2 aggregation. SINGLE-USE. |
