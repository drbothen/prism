---
document_type: holdout-scenario
level: L3
id: "HS-SERVERS-001-001"
title: "claroty_servers SELECT * wire shape: class_uid=5001, device_name Tier-1 REQUIRED present, raw_extensions with server inventory fields"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-SERVERS-001 (HS-027 group). Tests BC-2.16.018 §Postconditions 1 (TOML table contract — ocsf_class = 'inventory_info' → class_uid 5001) and §Postconditions 2 Tier-1 (server_name → device_name REQUIRED, server_status → status_code) and Tier-2 (raw_extensions present with inventory fields). Runs against live monroe sensor — requires bearer_token credential configured. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-SERVERS-001-001: claroty_servers SELECT * wire shape: class_uid=5001, device_name Tier-1 REQUIRED present, raw_extensions with server inventory fields

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-SERVERS-001 (HS-027 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.018 §Postconditions 1 (TOML table contract — ocsf_class = "inventory_info" → class_uid 5001 from the existing class_selector arm) and §Postconditions 2 Tier-1 (`server_name → device.name` → Arrow `device_name` REQUIRED; `server_status → status_code`) and Tier-2 (`raw_extensions` object present with server inventory fields)
**Gate:** Story-level holdout gate (HS-027) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the `claroty_servers` table is registered and queryable via PrismQL
against the live Claroty xDome (monroe) sensor, and that the wire output matches BC-2.16.018
§Postconditions 1 and 2:

1. The returned JSON rows carry `class_uid = 5001` — the integer class_uid for `inventory_info`.
   If the class_selector resolved to a wrong class (e.g., 2002 vulnerability_finding) or if
   class_uid is absent from the response, this assertion fails.

2. The returned JSON rows carry a column named `device_name` — the Arrow field name for the
   Tier-1 mapping of `server_name` (source: `ocsf_field = "device.name"`, then
   `ocsf_field_to_arrow_name` → `device_name`). This is the REQUIRED Tier-1 column; its
   absence means the TOML spec was not parsed correctly or the ocsf_field_to_arrow_name
   transform was not applied.

3. The `device_name` value in at least one row is a non-null, non-empty string — evidence
   that real server data was retrieved from the live sensor.

4. The returned rows carry a `raw_extensions` column containing a JSON object with at least
   one server inventory key (e.g., `server_location`, `model`, `management_ip`, or
   `num_of_interfaces`) — evidence that Tier-2 columns were aggregated correctly.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT * FROM claroty.claroty_servers LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response wire JSON contains a row with a column `class_uid` equal to `5001`
**And** the response wire JSON contains a row with a column `device_name` that is a non-null, non-empty string
**And** the response wire JSON contains a row with a column `raw_extensions` that is a JSON object with at least one server inventory key

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor. Do NOT include
   the credential value in any output — reference the credential by the configuration key only (AD-017).

3. Start prism in MCP stdio mode with the claroty sensor spec included. Capture the full MCP stdio
   output and any stderr.

4. Wait for prism to be ready (startup completion log message or first JSON-RPC prompt).

5. Issue the MCP `query` tool call:
   `{"sql": "SELECT * FROM claroty.claroty_servers LIMIT 1"}`.

6. Capture the full raw wire-level JSON response from the MCP tool call.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.018 | §Postconditions 1: ocsf_class = "inventory_info" → class_uid 5001 | Assertion 1: class_uid = 5001 in wire output |
| BC-2.16.018 | §Postconditions 2 Tier-1: server_name → ocsf_field = "device.name" → Arrow field device_name REQUIRED | Assertion 2: device_name column present and non-null |
| BC-2.16.018 | §Postconditions 2 Tier-2: 15 scalar Tier-2 columns aggregate into raw_extensions | Assertion 3: raw_extensions JSON object with inventory keys present |
| BC-2.16.018 | §Postconditions 1: POST /api/v1/servers/, response_path = $.servers, offset_limit pagination | End-to-end: table successfully queries live sensor |

---

## Verification Approach

1. Parse the wire-level JSON response from the MCP `query` tool call.

2. Locate the `rows` (or equivalent row array) in the response payload. If the response is an error
   object (contains `error_code` or similar), record as FAIL with observation "query returned error."

3. Inspect the first row's column list. Find the column named `class_uid`. Assert its integer value
   equals `5001`. If the column is absent or the value differs, record FAIL on the "class_uid=5001"
   dimension.

4. Inspect the first row's column list for `device_name`. Assert its value is a non-null, non-empty
   string. If the column is absent, record FAIL on "device_name present" dimension.

5. Inspect the first row's column list for `raw_extensions`. Assert it is a JSON object (not null,
   not a string) containing at least one key that belongs to the Tier-2 column set. If `raw_extensions`
   is absent or null, record FAIL on "raw_extensions Tier-2 aggregation" dimension.

6. Do NOT assert specific device_name values or specific Tier-2 field values — the live sensor's
   content varies; structural assertions are sufficient.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query succeeds (no error)** (weight: 0.25): Does the MCP query return a non-error response
  with at least one row?
  Full credit (1.0): non-error response with ≥1 row.
  Zero credit (0.0): error response of any kind.

- **class_uid = 5001 in wire output** (weight: 0.35): Does at least one returned row carry
  `class_uid = 5001`?
  Full credit (1.0): class_uid column present, value is integer 5001.
  Partial credit (0.3): class_uid column present but value is wrong (e.g., 2002 or 3004).
  Zero credit (0.0): class_uid column absent or query errored.

- **device_name present and non-null** (weight: 0.25): Does at least one returned row carry
  a non-null `device_name` string?
  Full credit (1.0): device_name present, non-null, non-empty string.
  Partial credit (0.5): device_name present but null or empty.
  Zero credit (0.0): device_name column absent (ocsf_field_to_arrow_name transform not applied).

- **raw_extensions present with Tier-2 keys** (weight: 0.15): Does at least one returned row
  carry a `raw_extensions` JSON object with at least one server inventory key?
  Full credit (1.0): raw_extensions present, is a JSON object, contains at least one Tier-2 key.
  Partial credit (0.5): raw_extensions present but null or empty object.
  Zero credit (0.0): raw_extensions column absent.

---

## Edge Conditions

- **Live sensor returns empty result set (zero rows):** Record as SETUP-FAILURE (no servers
  provisioned in the xDome instance) — not a behavioral FAIL. Do not score row-content dimensions.

- **Sensor authentication failure (E-SENSOR-001 / 401):** Record as SETUP-FAILURE — not a
  behavioral FAIL.

- **`claroty_servers` table not registered (E-QUERY-038 or "table not found"):** This IS a
  behavioral FAIL — the TOML table block was not added or not parsed correctly.

- **`class_uid` present as string `"5001"` rather than integer:** Record as PARTIAL (0.5) on
  the class_uid dimension.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-SERVERS-001-001 (satisfaction: X.XX) — claroty_servers wire-shape gap; check TOML table block registration and OCSF class_uid=5001 mapping (BC-2.16.018 §Postconditions 1) and device_name Tier-1 column ocsf_field_to_arrow_name transform (BC-2.16.018 §Postconditions 2)"`

Do NOT disclose: the specific column values expected, the LIMIT value used, or the exact assertion
threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/servers/ |
| corpus_size | LIMIT 1 (single row sufficient for structural assertion) |
| known_edge_cases | Empty result set (no servers in xDome instance — SETUP-FAILURE, not behavioral FAIL) |
| false_positive_threshold | Zero: class_uid=5001 and device_name are structural wire-shape assertions |
| false_negative_threshold | Zero: if device_name is absent, the OCSF Tier-1 column mapping is broken |

**Known-good corpus:** monroe Claroty xDome with ≥1 collection server — expected: non-error
response, class_uid=5001, device_name non-null string, raw_extensions with inventory keys.

**Known-problematic corpus:** An environment where the `claroty_servers` TOML table block was
not added — expected: "table not found" error or E-QUERY-038. This is the failure mode
BC-2.16.018 §Postconditions 1 guards against.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-027 group for S-CLAROTY-SERVERS-001. Wire-shape assertion: class_uid=5001 and device_name Tier-1 column present in live monroe sensor output. BC-2.16.018 §Postconditions 1 and 2. SINGLE-USE. |
