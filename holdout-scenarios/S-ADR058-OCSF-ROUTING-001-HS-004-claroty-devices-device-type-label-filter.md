---
document_type: holdout-scenario
level: L3
id: "HS-ROUTING-001-A-004"
title: "Claroty devices query with WHERE device_type_label filter returns matching rows — KF-06 demo-critical end-to-end"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-OCSF-ROUTING"
story_source: "S-ADR058-OCSF-ROUTING-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-21T00:00:00Z"
modified: "2026-08-21"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "caaa833"
traces_to: "BC-2.16.003"
behavioral_contracts:
  - BC-2.16.003
  - BC-2.16.002
verification_properties:
  - VP-017
lifecycle_status: active
introduced: "S-ADR058-OCSF-ROUTING-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-ADR058-OCSF-ROUTING-001 — KF-06 demo-critical: WHERE device_type_label = 'PLC' filter on claroty.devices succeeds using OCSF field name, WHERE device_type = 'PLC' fails (breaking-change surface). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ROUTING-001-A-004: Claroty devices query with WHERE device_type_label filter returns matching rows — KF-06 demo-critical end-to-end

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-ROUTING-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 §Claroty Contracted OCSF Mappings devices table: device_type → ocsf_field = "device.type_label" (KF-06: was "device.type_name" pre-KF-06); BC-2.16.003 EC-016-013-014 (Claroty devices Arrow column "device_type_label" present; col.name "device_type" absent post-Stage-2); ADR-058 §H breaking-change surface; ADR-058 §I5 wire-shape obligation
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the KF-06 fix at the wire level and exercises the demo-critical OT filtering use case: an analyst queries Claroty devices to find all PLCs. The scenario simultaneously validates:

1. **KF-06 OCSF field name correction** (`device_type → ocsf_field = "device.type_label"`, Arrow column `"device_type_label"`) — the renamed column is queryable as `device_type_label`, NOT as the old col.name `device_type`.
2. **WHERE filter push-down on OCSF field names** — `WHERE device_type_label = 'PLC'` selects only PLC-type devices (the Claroty API's `device_type` field value that the analyst would see as the OT device category).
3. **Breaking-change surface** — `WHERE device_type = 'PLC'` fails because `device_type` is not a valid column name post-Stage-2 (the OCSF-flattened Arrow column is `device_type_label`).
4. **Serialized wire shape** — the row returned for the PLC device has `"device_type_label": "PLC"` in the JSON, not `"device_type": "PLC"`.

This is the demo-critical scenario because the typical OT analyst query pattern is "show me all PLCs" — exactly this filter. If `device_type_label` is the new query surface and the KF-06 TOML correction was applied, this query succeeds. If either the TOML was not corrected or the ocsf_field name to Arrow name mapping was wrong, the filter returns 0 rows or the query fails.

**Behavioral assertions:**

1. A minimal HTTP mock server handles the Claroty devices endpoint (verify the exact endpoint path from `crates/prism-sensors/specs/claroty.sensor.toml` devices table) returning two device records: one with `device_type = "PLC"` (IP: 192.168.1.10), one with `device_type = "Switch"` (IP: 192.168.1.20). Both records also have `name` and `ip_address` fields.

2. prism is started in MCP stdio mode with the Claroty TOML (`ocsf_column_naming = true`).

3. A `query` MCP tool call issues `SELECT device_type_label, name, ip_address FROM claroty.devices WHERE device_type_label = 'PLC'`.

4. The serialized JSON response contains exactly ONE row matching the PLC device:
   - The key `"device_type_label"` is present with value `"PLC"` (the OCSF-flattened Arrow column name for the vendor `device_type` field under `device.type_label` ocsf_field mapping with KF-06 correction).
   - The key `"ip_address"` is present with value `"192.168.1.10"` (the PLC device's IP — verifies the WHERE clause filtered correctly).
   - The Switch device (IP: 192.168.1.20) is NOT in the response.

5. A second `query` MCP tool call issues `SELECT device_type_label, name, ip_address FROM claroty.devices WHERE device_type = 'PLC'`. This query MUST fail or return an empty result set because `device_type` is no longer a valid Arrow column name (breaking-change surface from col.name to OCSF field name). If it succeeds or returns rows, that indicates the old col.name mapping is still active — a breaking-change regression.

6. A third `query` MCP tool call issues `SELECT device_type_label FROM claroty.devices`. The serialized JSON response must contain TWO rows total (one PLC, one Switch). The key `"device_type"` (the old col.name) must NOT appear at the top level of any row — only `"device_type_label"` (the OCSF Arrow field name) should be present.

**BDD supplement:**

**Given** a minimal HTTP mock serves the Claroty devices endpoint returning two records: `device_type = "PLC"` at 192.168.1.10 and `device_type = "Switch"` at 192.168.1.20
**And** prism MCP stdio is configured with Claroty `ocsf_column_naming = true` and `ocsf_field = "device.type_label"` for device_type (KF-06 TOML correction applied)
**When** `SELECT device_type_label, name, ip_address FROM claroty.devices WHERE device_type_label = 'PLC'` is issued via the MCP `query` tool
**Then** the serialized JSON response contains exactly one row with `"device_type_label": "PLC"` and `"ip_address": "192.168.1.10"`
**And** the Switch device (IP: 192.168.1.20) is not present in the response
**And** the key `"device_type"` (old col.name) does not appear in any row
**When** `SELECT device_type_label, name, ip_address FROM claroty.devices WHERE device_type = 'PLC'` is issued
**Then** the query fails or returns zero rows (breaking-change surface: device_type is not a valid column)

---

## Setup Instructions

1. Look up the devices table endpoint in `crates/prism-sensors/specs/claroty.sensor.toml`. Find the `[[tables]]` block where `table_name = "devices"` and identify the `path` or `url` field. The mock server must handle this exact path.

2. Look up the `device_type` column mapping in the devices table TOML block. Confirm `ocsf_field = "device.type_label"` (KF-06 correction). Also look up `ip_address` and `name` OCSF field mappings to determine the expected Arrow column names for those columns (needed for query construction).

3. Start a minimal HTTP mock server on `127.0.0.1:0` (ephemeral port). It must handle the devices endpoint returning:
   ```json
   {
     "assets": [
       {
         "id": "dev-001",
         "name": "PLC Controller A",
         "device_type": "PLC",
         "ip_address": "192.168.1.10",
         "mac_address": "00:11:22:33:44:55",
         "firmware_version": "3.2.1",
         "zone": "OT-Zone-A",
         "site": "Plant-1"
       },
       {
         "id": "dev-002",
         "name": "Network Switch B",
         "device_type": "Switch",
         "ip_address": "192.168.1.20",
         "mac_address": "00:AA:BB:CC:DD:EE",
         "firmware_version": "5.0.0",
         "zone": "OT-Zone-A",
         "site": "Plant-1"
       }
     ],
     "total": 2,
     "page": 1
   }
   ```
   Note: the exact JSON structure of the Claroty devices API response may vary — use whatever structure the Claroty TOML `source_path` expressions reference. The key field name is `device_type`; confirm against the TOML. The mock must return BOTH records (both PLC and Switch) so that the WHERE filter exercises DataFusion selection, not accidental mock filtering.

4. Confirm `crates/prism-sensors/specs/claroty.sensor.toml` has `ocsf_column_naming = true` (AC-005) and the device_type column has `ocsf_field = "device.type_label"` (KF-06 correction — Arrow column name will be `device_type_label`). If either is absent, record SETUP-FAILURE.

5. Configure a test Claroty sensor pointing the mock at `http://127.0.0.1:<PORT>` with bearer_token = any non-empty string.

6. Start prism in MCP stdio mode.

7. Issue the three MCP `query` tool calls as specified in §Behavioral Assertions.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | §Claroty Contracted OCSF Mappings devices: device_type → ocsf_field = "device.type_label" → Arrow "device_type_label" (KF-06) | device_type_label present in WHERE filter and row; device_type absent |
| BC-2.16.003 | EC-016-013-014: Claroty devices Arrow column "device_type_label" present; "device_type" absent at row level | device_type NOT in serialized row keys |
| ADR-058 §H | Breaking-change surface: SELECT device_type FROM claroty.devices fails post-Stage-2 | WHERE device_type = 'PLC' fails |
| ADR-058 §I5 | Wire-shape assertion: WHERE clause operates on OCSF Arrow names; PLC filter returns matching row | PLC row present, Switch row absent; wire key "device_type_label" present |
| BC-2.16.002 | Pipeline correctness: mock returns 2 records; WHERE filter on DataFusion columns correctly reduces to 1 | Exactly 1 row in response, not 0 (which would be a filter failure) |

---

## Verification Approach

1. Build the prism binary.
2. Start the mock HTTP server as specified in §Setup Instructions. Capture the bound port.
3. Configure prism with Claroty sensor pointing at the mock.
4. Launch prism in MCP stdio mode.

**Test A — WHERE device_type_label = 'PLC' (must succeed):**
5. Send MCP `query` tool call: `{"sql": "SELECT device_type_label, name, ip_address FROM claroty.devices WHERE device_type_label = 'PLC'"}`.
6. Receive the full MCP JSON response. Assert:
   - The response is valid JSON.
   - The response contains exactly ONE row (the PLC device).
   - In that row, key `"device_type_label"` is present with value `"PLC"`.
   - In that row, key `"ip_address"` is present with value `"192.168.1.10"`.
   - The key `"device_type"` is NOT a top-level key in the row (old col.name absent at wire level).

**Test B — WHERE device_type = 'PLC' (must fail or return 0 rows):**
7. Send MCP `query` tool call: `{"sql": "SELECT device_type_label, name, ip_address FROM claroty.devices WHERE device_type = 'PLC'"}`.
8. Receive the response. Assert:
   - The response is either: (a) an error response with message indicating `device_type` is not a recognized column, OR (b) a successful response with zero rows. Either outcome is acceptable for this dimension — the key is that `device_type` is not a queryable column name.
   - If the response contains any rows with `"device_type"` as a key, record FAIL with observation "device_type column still queryable by col.name — breaking-change surface not activated."

**Test C — Full SELECT without WHERE (must return 2 rows, both with device_type_label):**
9. Send MCP `query` tool call: `{"sql": "SELECT device_type_label FROM claroty.devices"}`.
10. Receive the response. Assert:
    - The response contains exactly TWO rows.
    - ALL rows have key `"device_type_label"` present (not null).
    - NO row has key `"device_type"` as a top-level field.
    - One row has `"device_type_label": "PLC"`, the other has `"device_type_label": "Switch"`.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **WHERE device_type_label filter returns exactly 1 PLC row (Test A)** (weight: 0.40): Is exactly 1 row returned by the PLC filter with `"device_type_label": "PLC"` and `"ip_address": "192.168.1.10"`?
  Full credit (1.0): Exactly 1 row, correct field values.
  Partial credit (0.4): Correct row returned but also Switch row included (filter not applied).
  Zero credit (0.0): 0 rows returned (filter applied but OCSF mapping broken) OR device_type_label not in row.

- **device_type (old col.name) absent at wire level (Tests A + C)** (weight: 0.25): Is the key `"device_type"` absent from ALL row responses?
  Full credit (1.0): `"device_type"` never appears as a top-level row key.
  Zero credit (0.0): `"device_type"` appears as a row key in any test response.

- **WHERE device_type = 'PLC' fails or returns 0 rows (Test B)** (weight: 0.25): Does querying by the old col.name fail (error) or return 0 rows?
  Full credit (1.0): Query returns 0 rows or an error.
  Zero credit (0.0): Query returns rows — old col.name still queryable, breaking-change surface not activated.

- **Full table returns 2 rows both with device_type_label (Test C)** (weight: 0.10): Does SELECT device_type_label return 2 rows, both with the field present?
  Full credit (1.0): 2 rows, both with device_type_label present.
  Partial credit (0.5): 2 rows but one has null device_type_label.
  Zero credit (0.0): 1 or 0 rows (data loss) or neither row has device_type_label.

---

## Edge Conditions

- **KF-06 TOML correction not applied (ocsf_field still "device.type_name"):** Arrow column will be `device_type_name` instead of `device_type_label`. Test A will fail (column `device_type_label` not found). Record as FAIL with observation "device_type_label column not found — KF-06 TOML correction not applied; ocsf_field is likely still 'device.type_name'."

- **`ocsf_column_naming = true` not in TOML:** Arrow columns use `col.name` instead of OCSF flattened names. Test A fails (`device_type_label` not a column). Test B succeeds (old col.name still active). Record as FAIL with observation "device_type is queryable and device_type_label is not — ocsf_column_naming = true not set in claroty.sensor.toml."

- **Both col.name and OCSF name present in Arrow schema (regression guard):** If somehow both `device_type` and `device_type_label` appear as Arrow columns (old code path not fully removed), Test C shows both keys in the row. Record FAIL on "device_type (old col.name) absent at wire level" dimension.

- **Mock returns HTTP error or wrong structure:** Record as SETUP-FAILURE. Check that the mock endpoint path matches what the TOML declares.

- **Query returns 0 rows (empty result set) for Test A:** Indicates the WHERE filter is applied but is NOT matching the field value — possible source_path extraction issue where `device_type = "PLC"` in the mock JSON is not being read correctly. Record as FAIL: "0 rows returned for PLC filter — possible source_path extraction issue in devices pipeline."

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ROUTING-001-A-004 (satisfaction: X.XX) — Claroty devices device_type_label filter behavioral failure; check KF-06 TOML ocsf_field correction (device.type_label) and ocsf_field_to_arrow_name Arrow column name in pipeline_result_to_record_batch"`

Do NOT disclose: the specific IP addresses used, the exact filter values, the two-device test structure, or the WHERE failure assertion.

---

## Category: real-world-corpus

This scenario is grounded in the demo-critical OT analyst query pattern: "show me all PLCs in my network." The KF-06 fix is directly traceable to OCSF v1.7.0 schema validation — `device.type_name` does not exist in the OCSF v1.7.0 schema; the correct attribute is `device.type_label`. This means pre-KF-06, a query like `WHERE device_type_name = 'PLC'` would reference a non-existent OCSF attribute. The KF-06 correction to `device.type_label` is the OCSF-correct attribute name, validated against the inventory_info (class 5001) OCSF schema for device objects.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome devices API simulated via mock; grounded in BC-2.16.003 §Claroty Contracted OCSF Mappings (devices) + KF-06 + ADR-058 §H breaking-change surface |
| corpus_size | Two device records (PLC + Switch); WHERE filter, full SELECT, and breaking-change queries exercised |
| known_edge_cases | WHERE on old col.name device_type returning rows is the primary demo-day risk; 0-row result is the secondary risk (source_path extraction failure) |
| false_positive_threshold | Zero: `"device_type_label": "PLC"` in the PLC row is an unambiguous KF-06 end-to-end success signal |
| false_negative_threshold | Low: WHERE filtering is DataFusion core; the risk is the OCSF column NAME being wrong, not the filter predicate itself |

**Known-good corpus:** Claroty devices with `device_type = "Workstation"` — Arrow column `device_type_label` should contain `"Workstation"`. Tests that the OCSF flattening is applied correctly regardless of the OT device category value.

**Known-problematic corpus:** Claroty TOML with `ocsf_column_naming = false` (or missing) and `device_type` col.name query — `device_type` IS queryable (old behavior). This tests that the ocsf_column_naming feature gate correctly switches the column naming mode and that the two modes don't accidentally overlap.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-ROUTING-001-holdout-authoring | 2026-08-21 | product-owner | Initial authoring. Story-level holdout gate for S-ADR058-OCSF-ROUTING-001 — KF-06 demo-critical: WHERE device_type_label = 'PLC' filter succeeds; WHERE device_type = 'PLC' fails (breaking-change); col.name absent at wire level. Covers AC-001 + BC-2.16.003 EC-016-013-014. SINGLE-USE. |
