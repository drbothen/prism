---
document_type: holdout-scenario
level: L3
id: "HS-ROUTING-001-A-001"
title: "Claroty alerts query returns OCSF-flattened Arrow field names — finding_info_uid not id"
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
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-ADR058-OCSF-ROUTING-001 — Core breaking-change wire-shape test: Claroty alerts query returns OCSF-flattened column names (finding_info_uid, finding_info_title) and SELECT id fails (no such column after Stage 2). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ROUTING-001-A-001: Claroty alerts query returns OCSF-flattened Arrow field names — finding_info_uid not id

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-ROUTING-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 §Column Routing postconditions — §Interpretation A: Arrow Field Naming; EC-016-013-017 (KF-03/KF-04 corrected ocsf_field → flattened Arrow names); ADR-058 §C2 Option 4 underscore-flattening convention
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the core breaking-change wire shape introduced by S-ADR058-OCSF-ROUTING-001: when `ocsf_column_naming = true` is active for the Claroty sensor, MCP query responses return OCSF-flattened Arrow field names instead of `col.name` values. The most visible change is that the `id` column (`col.name = "id"`, `ocsf_field = "finding_info.uid"`) is now named `"finding_info_uid"` in the Arrow schema — agents querying `SELECT id FROM claroty.alerts` receive a DataFusion error, while `SELECT finding_info_uid FROM claroty.alerts` succeeds.

The pre-Stage-2 behavior: `pipeline_result_to_record_batch` uses `col.name` unconditionally, producing Arrow field `"id"`. A query `SELECT id FROM claroty.alerts` returns a row with `"id"` column.

The post-Stage-2 behavior: `pipeline_result_to_record_batch` branches on `sensor_spec.ocsf_column_naming = true`, applies `ocsf_field_to_arrow_name("finding_info.uid")` = `"finding_info_uid"`, and produces Arrow field `"finding_info_uid"`. The `"id"` Arrow field no longer exists. A query `SELECT finding_info_uid FROM claroty.alerts` returns the alert ID; `SELECT id FROM claroty.alerts` returns a DataFusion column-not-found error.

This scenario exercises KF-03 (`alerts.id` ocsf_field corrected to `"finding_info.uid"`) and KF-04 (`alerts.alert_name` ocsf_field corrected to `"finding_info.title"`), verifying the full end-to-end path from mock Claroty API → `pipeline_result_to_record_batch` → Arrow schema → DataFusion query → serialized MCP JSON response at the wire level.

**Behavioral assertions:**

1. A minimal HTTP mock server handles `POST /api/v1/alerts` returning one alert where `id = "alert-777"` and `alert_name = "Modbus Violation"`. All other required alert fields are valid.
2. prism is started in MCP stdio mode with Claroty sensor TOML (with `ocsf_column_naming = true`) pointing at the mock.
3. A `query` MCP tool call issues `SELECT finding_info_uid, finding_info_title FROM claroty.alerts`.
4. The serialized JSON response contains at least one row where:
   - The key `"finding_info_uid"` is present with value `"alert-777"`
   - The key `"finding_info_title"` is present with value `"Modbus Violation"`
5. The serialized JSON row does NOT contain a top-level key named `"id"` (col.name is not an Arrow field name post-Stage-2).
6. The serialized JSON row does NOT contain a top-level key named `"alert_name"` (same — col.name is not the Arrow field name after Stage-2 for ocsf_field=Some columns).
7. A second `query` MCP tool call issues `SELECT id FROM claroty.alerts`.
8. This second query returns an MCP error response (DataFusion: column "id" not found) or an empty result set — NOT a row with `"id": "alert-777"`. An error response here is a PASS.

**BDD supplement:**

**Given** a minimal HTTP mock serves `POST /api/v1/alerts` returning one alert where `id = "alert-777"` and `alert_name = "Modbus Violation"`
**And** prism MCP stdio is configured with Claroty `ocsf_column_naming = true` pointing at the mock
**When** `SELECT finding_info_uid, finding_info_title FROM claroty.alerts` is issued via the MCP `query` tool
**Then** the serialized JSON response row contains `"finding_info_uid": "alert-777"` (OCSF-flattened name, not `"id"`)
**And** the serialized JSON response row contains `"finding_info_title": "Modbus Violation"` (OCSF-flattened name, not `"alert_name"`)
**And** the serialized JSON row does NOT contain a key named `"id"` or `"alert_name"` as a first-class column
**When** `SELECT id FROM claroty.alerts` is issued via the MCP `query` tool
**Then** the response is an MCP error or empty result (column "id" does not exist in the post-Stage-2 Arrow schema)

---

## Setup Instructions

1. Start a minimal HTTP mock server on `127.0.0.1:0` (ephemeral port). It must handle:
   - `POST /api/v1/alerts` → return HTTP 200 with body:
     ```json
     {
       "alerts": [{
         "id": "alert-777",
         "alert_name": "Modbus Violation",
         "alert_type_name": "Protocol Violation",
         "category": "OT Security",
         "status": "open",
         "detected_time": "2026-08-21T08:00:00Z",
         "updated_time": "2026-08-21T08:01:00Z",
         "devices_count": 2,
         "description": "Unauthorized Modbus function code detected on PLC segment",
         "alert_class": "OT",
         "ot_devices_count": 2
       }],
       "total": 1,
       "page": 1
     }
     ```
   - Subsequent pagination calls may return `{"alerts": [], "total": 1, "page": 2}`.

2. Confirm `crates/prism-sensors/specs/claroty.sensor.toml` has `ocsf_column_naming = true` (set by AC-005 of this story). If not set, record SETUP-FAILURE.

3. Configure a test Claroty sensor pointing the mock at `http://127.0.0.1:<PORT>` with bearer_token = any non-empty string.

4. Start prism in MCP stdio mode with `RUST_LOG=prism_bin=warn` or equivalent. Capture stderr.

5. First query: issue MCP `query` tool call with `{"sql": "SELECT finding_info_uid, finding_info_title FROM claroty.alerts"}`. Capture the full serialized JSON response.

6. Second query: issue MCP `query` tool call with `{"sql": "SELECT id FROM claroty.alerts"}`. Capture the response (expect MCP error or empty result).

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | §Column Routing postcondition: "columns with an ocsf_field value are mapped to the corresponding OCSF field" — Arrow field name = ocsf_field_to_arrow_name(ocsf_field) when ocsf_column_naming=true | Core assertion: finding_info_uid = "alert-777" in wire response |
| BC-2.16.003 | §Interpretation A: Arrow Field Naming — ocsf_field declarations produce queryable Arrow field identifiers | finding_info_uid and finding_info_title are the queryable field names |
| BC-2.16.003 | EC-016-013-017 (KF-03/KF-04): alerts.id ocsf_field "finding_info.uid" → Arrow "finding_info_uid"; alerts.alert_name ocsf_field "finding_info.title" → Arrow "finding_info_title" | Both corrected field names validated at wire level |
| ADR-058 §C2 | Option 4 underscore-flattening: dots replaced with underscores in Arrow field name | finding_info.uid → finding_info_uid (two dots each replaced) |
| ADR-058 §D1 | sensor_spec threaded from fetch() to pipeline_result_to_record_batch; ocsf_column_naming flag applied | End-to-end path from spec to Arrow schema observed at MCP wire level |
| BC-2.16.003 | Breaking-change surface: col.name no longer used for ocsf_field=Some columns when flag=true | SELECT id fails — no column named id after Stage 2 |

---

## Verification Approach

1. Build the prism binary (`cargo build --release -p prism-bin` or `just build`).
2. Start the mock HTTP server as specified in §Setup Instructions. Capture the bound port.
3. Configure prism with Claroty sensor pointing at the mock at `http://127.0.0.1:<PORT>`.
4. Launch prism in MCP stdio mode, capturing stderr.
5. Send first MCP `query` tool call: `{"sql": "SELECT finding_info_uid, finding_info_title FROM claroty.alerts"}`.
6. Receive the full MCP JSON response. Assert:
   - The response is valid JSON (parse without error).
   - The response contains at least one row.
   - In the row, key `"finding_info_uid"` is present with value `"alert-777"` (string).
   - In the row, key `"finding_info_title"` is present with value `"Modbus Violation"` (string).
   - The serialized row JSON bytes do NOT contain the literal string `"id":` as a top-level column key (substring search on the raw JSON row bytes).
   - The serialized row JSON bytes do NOT contain the literal string `"alert_name":` as a top-level column key.
7. Send second MCP `query` tool call: `{"sql": "SELECT id FROM claroty.alerts"}`.
8. Assert: the response is either an MCP error JSON (column not found) or an empty result set with zero rows. A row containing `"id": "alert-777"` is a FAIL — it means the pre-Stage-2 col.name path is still active.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **OCSF flattened field names present** (weight: 0.40): Does the first query response contain `"finding_info_uid": "alert-777"` AND `"finding_info_title": "Modbus Violation"` in the row?
  Full credit (1.0): both fields present with correct values.
  Partial credit (0.5): one of the two is correct; the other is missing or wrong.
  Zero credit (0.0): neither is present; or values are wrong; or row is missing.

- **col.name fields absent from wire response** (weight: 0.30): Does the first query row NOT contain keys `"id"` or `"alert_name"` at the top level?
  Full credit (1.0): neither `"id"` nor `"alert_name"` appears as a first-class column in the row.
  Zero credit (0.0): either `"id"` or `"alert_name"` appears as a first-class column — pre-Stage-2 behavior still active.

- **SELECT id fails (breaking-change confirmed)** (weight: 0.20): Does the second query (`SELECT id FROM claroty.alerts`) fail with an MCP error or return zero rows?
  Full credit (1.0): MCP error response or empty result — `id` is not a valid column name.
  Zero credit (0.0): response contains a row with `"id": "alert-777"` — pre-Stage-2 regression.

- **Record presence on first query** (weight: 0.10): Is at least one row returned for the first query?
  Full credit (1.0): row present.
  Zero credit (0.0): empty result on the first query despite mock returning data — pipeline failure.

---

## Edge Conditions

- **ocsf_column_naming flag not yet applied to Claroty TOML:** If the TOML shows `ocsf_column_naming` absent or false, the first query returns `"id"` instead of `"finding_info_uid"` — record as SETUP-FAILURE. Do NOT mark as behavioral FAIL.

- **KF-03 correction missing (ocsf_field still "finding.uid" not "finding_info.uid"):** The first query returns `"finding_uid"` instead of `"finding_info_uid"`. Record as FAIL with observation "KF-03 correction absent — produces finding_uid not finding_info_uid."

- **Mock returns HTTP 400 or connection error:** Record as SETUP-FAILURE. Do NOT mark as behavioral FAIL.

- **First query returns DataFusion error (finding_info_uid not found):** Check if TOML flag was applied. If applied, this is a FAIL — the OCSF-flattened field was not correctly added to the Arrow schema.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ROUTING-001-A-001 (satisfaction: X.XX) — Claroty alerts wire response does not contain OCSF-flattened column names; check pipeline_result_to_record_batch ocsf_column_naming branch and ocsf_field_to_arrow_name call in spec_driven_adapter.rs"`

Do NOT disclose: the specific alert ID or alert_name values used, the exact assertion threshold, or the fixture JSON structure.

---

## Category: real-world-corpus

This scenario is grounded in the BC-2.16.003 §Interpretation A postcondition: after enabling `ocsf_column_naming = true` for Claroty, LLM agents querying via MCP must use OCSF-flattened field names. This is the primary behavioral change agents observe and the primary breaking-change surface for existing queries. KF-03/KF-04 corrections (`id → finding_info_uid`, `alert_name → finding_info_title`) are the most user-visible changes.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome alerts API simulated via mock; grounded in BC-2.16.003 §Claroty Contracted OCSF Mappings (KF-03, KF-04) |
| corpus_size | Single alert record; two OCSF-mapped columns exercised (finding_info.uid, finding_info.title) |
| known_edge_cases | Single-segment ocsf_field values (e.g., status → status) remain unchanged; this test does not cover single-segment cases |
| false_positive_threshold | Zero: finding_info_uid in the wire output is an unambiguous Stage 2 behavioral postcondition |
| false_negative_threshold | Zero: "id" in the wire output after Stage 2 activation is a clear pre-Stage-2 regression |

**Known-good corpus:** Claroty TOML with `ocsf_column_naming = false` (Interpretation B / default) — expected result: `"id"` and `"alert_name"` appear as column names, NOT `finding_info_uid` / `finding_info_title`. Tests that the flag=false path produces the legacy behavior without regression (AC-004).

**Known-problematic corpus:** Claroty TOML with `ocsf_column_naming = true` as deployed by Stage 2 — expected result: `"finding_info_uid"` and `"finding_info_title"` appear; `"id"` and `"alert_name"` are absent from the wire row. Tests that Stage 2 breaking-change is correctly applied.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-ROUTING-001-holdout-authoring | 2026-08-21 | product-owner | Initial authoring. Story-level holdout gate for S-ADR058-OCSF-ROUTING-001 — Core breaking-change wire shape: finding_info_uid replaces id; finding_info_title replaces alert_name. KF-03/KF-04 end-to-end. SINGLE-USE. |
