---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-07-27T00:00:00
phase: 3
origin: greenfield
subsystem: "SS-02"
capability: "CAP-003"
lifecycle_status: draft
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - "crates/prism-dtu-armis/src/types.rs"
  - ".factory/specs/architecture/decisions/ADR-057-armis-activity-per-device-push-down-grammar.md"
input-hash: ""
traces_to: ["CAP-003"]
extracted_from: ".factory/specs/prd.md"
scheduled_amendment_in: null
amendment_lifecycle: null
introduced: cycle-FB73
modified: "2026-07-27"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.014: Armis Device Activity Surface — Filter-Required Push-Down Fetch Contract

## Description

The `armis_device_activity` TOML table provides a single-device, filter-required query surface over the Armis DTU route `GET /api/v1/devices/{device_id}/activity`. Unlike the flat AQL-based `armis_devices` and `armis_alerts` tables, every query against this surface MUST include a `WHERE device_id = '...'` predicate. The `device_id` predicate is pushed down via `FetchContext.query_filters["device_id"]` (the `${query.filter.*}` namespace) into the step `path_template`. Fleet-wide activity queries (absent `device_id` filter) are explicitly out of scope for Wave-A — per-record fan-out over prior-step results is a confirmed capability gap (ADR-057 §D6). SAP-2 column parity: five `ActivityRecord` fields are verified against the DTU wire-emission site `routes::devices::get_device_activity` in `prism-dtu-armis/src/routes/devices.rs`.

## Related BCs

- BC-2.02.006: Armis Centrix Field Mapping to OCSF (7 Data Sources) — composes with (activity surface deferral tracked in EC-02-014; this BC owns the activity contract; devices/alerts field mapping in BC-2.02.006)
- BC-2.16.002: Multi-Step Fetch Pipeline Execution — depends on (pipeline interpolation mechanism used by `${query.filter.device_id}`)

## Architecture Anchors

- `.factory/specs/architecture/decisions/ADR-057-armis-activity-per-device-push-down-grammar.md` — authority for the `${query.filter.device_id}` grammar and the `${query.filter.*}` pre-seed mechanism (§D4, §D5)
- `.factory/specs/architecture/decisions/ADR-033-push-down-time-window-extraction-strategy-pre-fan-out-heuristic.md` — authority for datetime time-window extraction (`extract_time_window_from_ast §extract_time_window_from_ast` extracts `start_time`/`end_time` into `QueryParams`; T2: `classify_predicates §classify_predicates`, deferred); NOT the authority for equality push-down routing

## Story Anchor

`S-WAVE-A-ARMIS-ACTIVITY-001` — implementing story (currently UNBLOCKED per ADR-057 §C1; story-writer is updating to `status: ready`)

## VP Anchors

None assigned yet. To be added when verification properties are authored for this surface.

---

## Preconditions

- The `armis_device_activity` TOML table is declared in `crates/prism-sensors/specs/armis.sensor.toml` with a step using `path_template = "/api/v1/devices/${query.filter.device_id}/activity"` and `response_path = "$.data.activities"`
- The `device_id` column in the `armis_device_activity` table carries `options = ["INDEX"]`, declaring push-down eligibility per the BC-2.11.007 taxonomy (REQUIRED / INDEX / ADDITIONAL) for future T2 (`classify_predicates §classify_predicates`) integration; the current routing is annotation-agnostic: `predicate_tree_to_filter_map §predicate_tree_to_filter_map` collects all case-sensitive `field = 'string'` equality predicates regardless of column annotation into the `FilterMap`, which materializes as `FetchContext.query_filters`; `execute_impl §execute_impl` then pre-seeds `step_vars["query.filter.device_id"]` from that map (ADR-057 §D4)
- A `WHERE device_id = '...'` predicate is present in the PrismQL query against `armis_device_activity`

## Postconditions

- `PipelineExecutor.execute_impl` pre-seeds `step_vars["query.filter.device_id"]` from `FetchContext.query_filters["device_id"]` via the `${query.filter.*}` pre-seed loop (block comment `F-LP1-HIGH-004` in `pipeline.rs §execute_impl`: `for (k, v) in &context.query_filters { step_vars.insert(format!("query.filter.{k}"), ...) }`)
- The step URL is constructed as `<base_url>/api/v1/devices/<device_id_value>/activity` via interpolation of `${query.filter.device_id}` in the `path_template`
- The pipeline fetches from the DTU `routes::devices::get_device_activity` handler, extracts `$.data.activities` via `response_path`, and returns an array of activity records
- Each emitted record contains the five fields from `ActivityRecord` in `prism-dtu-armis/src/types.rs`, serialized via `serde_json::to_value(&body)` where `body` is `ActivityResponse { data: ActivityData { activities, total } }` (wire-emission site authority per SAP-2 §Rule 6):
  - `activity_id: String` → TOML column `activity_id`, `column_type = "string"`, `ocsf_field = "raw_extensions.activity_id"` (Armis-specific activity record identifier; no OCSF standard field)
  - `device_id: String` → TOML column `device_id`, `column_type = "string"`, `ocsf_field = "device.uid"`, `options = ["INDEX"]` (device identifier matching the filter value; enables push-down)
  - `activity_type: String` → TOML column `activity_type`, `column_type = "string"`, `ocsf_field = "activity_name"` (OCSF activity classification field)
  - `timestamp: String` → TOML column `timestamp`, `column_type = "string"`, `ocsf_field = "time"` (ISO-8601 activity timestamp; maps to OCSF `time` field)
  - `details: serde_json::Value` → TOML column `details`, `column_type = "json"`, `ocsf_field = "raw_extensions.details"` (JSON blob preserving Armis activity detail structure)
- Absent `device_id` filter → pipeline MUST return `SpecEngineError::HttpRequestFailed` or equivalent before issuing any HTTP request; pre-fix current-code behavior: `seed_missing_query_filter_vars §seed_missing_query_filter_vars` inserts empty-string seed → path `/api/v1/devices//activity` → DTU returns **HTTP 200 with `activities: []` and `total: 0`** (silent empty result, Standing Rule 3 §2 violation); anchor: AC-004 / RG-004 (`test_armis_device_activity_absent_device_id_filter_returns_hard_error`) per §TOML Contract required-filter note

## Invariants

- DI-005: OCSF schema validity — every record must produce a valid OCSF output with required fields populated
- Filter-required invariant: every successful query execution against `armis_device_activity` has exactly one `device_id` predicate resolved into `FetchContext.query_filters["device_id"]`; queries without this predicate are errors, not empty result sets

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| Hard error | `device_id` filter absent from query (no `WHERE device_id = '...'` predicate) | Pre-fix observable behavior: `seed_missing_query_filter_vars §seed_missing_query_filter_vars` inserts `query.filter.device_id = ""` → path `/api/v1/devices//activity` → DTU handler receives `device_id = ""` via axum path extraction → fixture filter matches nothing → **HTTP 200 with `activities: []` and `total: 0`** (silent empty result, Standing Rule 3 §2 violation). Required behavior: pipeline MUST return `SpecEngineError::HttpRequestFailed` or equivalent BEFORE issuing any HTTP request; zero records. Anchor: AC-004 / RG-004 (`test_armis_device_activity_absent_device_id_filter_returns_hard_error`) |
| Hard error | DTU returns non-200 status for the requested `device_id` path | `SpecEngineError::HttpRequestFailed` with DTU status code; not a silent empty result |
| Warning | `timestamp` value unparseable as ISO-8601 by OCSF normalization | Best-effort mapping; raw timestamp value preserved in `raw_extensions`; warning logged per DI-005 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-014-001 | Query without `WHERE device_id = '...'` predicate | Pre-fix: path `/api/v1/devices//activity` produced → DTU returns **HTTP 200 + `activities: []` + `total: 0`** (silent empty result). Required: hard error (`SpecEngineError::HttpRequestFailed`) returned before issuing any HTTP request; zero records; no silent empty result. Anchor: AC-004 / RG-004 (`test_armis_device_activity_absent_device_id_filter_returns_hard_error`) |
| EC-014-002 | `device_id` filter present but device has no activity records in DTU | DTU returns empty `activities` array (`total = 0`); pipeline returns zero records; no error |
| EC-014-003 | `device_id` filter present; device not found in DTU fixture | DTU may return empty activities or HTTP error; pipeline surfaces as `SpecEngineError::HttpRequestFailed` on non-200 or zero records on HTTP 200 + empty array |
| EC-014-004 | `details` is a complex nested JSON object | Emitted as-is in `raw_extensions.details` column (`column_type = "json"`); no truncation; agent receives full Armis activity detail structure |
| EC-014-005 | `details` is `null` in a record | `details` column emits null; not an error; nullable per `serde_json::Value` serialization |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.014-001 | Query `SELECT * FROM armis_device_activity WHERE device_id = 'dev-001'` against DTU with seeded activity fixture | Pipeline fetches `GET /api/v1/devices/dev-001/activity`; returns `ActivityRecord` array scoped to `dev-001`; all five columns present in each record |
| TV-BC-2.02.014-002 | Query `SELECT * FROM armis_device_activity` without `WHERE device_id` predicate | Hard error (`SpecEngineError::HttpRequestFailed` or equivalent); zero records; no silent empty result |
| TV-BC-2.02.014-003 | Query with `device_id = 'no-such-device'`; DTU has no matching activity records | Zero records returned (`activities: []`); no error (HTTP 200 with empty array) |
| TV-BC-2.02.014-004 | Activity record with `details = {"protocol": "ssh", "port": 22, "outcome": "success"}` | `details` column contains serialized JSON object; `raw_extensions.details` populated in OCSF output |

## TOML Contract

The `armis_device_activity` table MUST be declared in `crates/prism-sensors/specs/armis.sensor.toml` per ADR-057 §D5 (anchor: `S-WAVE-A-ARMIS-ACTIVITY-001` AC-001 / RG-001 `test_armis_toml_armis_device_activity_table_declared_with_correct_step_block`):

```toml
[[tables]]
name = "armis_device_activity"
sensor_name = "armis"

  [[tables.steps]]
  name = "fetch_device_activity"
  method = "GET"
  path_template = "/api/v1/devices/${query.filter.device_id}/activity"
  response_path = "$.data.activities"
  variables_produced = []

  # SAP-2 verified: activity_id — ActivityRecord.activity_id: String
  # emitted by routes::devices::get_device_activity via serde_json::to_value(&body)
  [[tables.columns]]
  name = "activity_id"
  column_type = "string"
  ocsf_field = "raw_extensions.activity_id"

  # SAP-2 verified: device_id — ActivityRecord.device_id: String
  # INDEX option enables push-down extraction: WHERE device_id = '...' → FetchContext.query_filters["device_id"]
  [[tables.columns]]
  name = "device_id"
  column_type = "string"
  ocsf_field = "device.uid"
  options = ["INDEX"]

  # SAP-2 verified: activity_type — ActivityRecord.activity_type: String
  [[tables.columns]]
  name = "activity_type"
  column_type = "string"
  ocsf_field = "activity_name"

  # SAP-2 verified: timestamp — ActivityRecord.timestamp: String (ISO-8601)
  [[tables.columns]]
  name = "timestamp"
  column_type = "string"
  ocsf_field = "time"

  # SAP-2 verified: details — ActivityRecord.details: serde_json::Value
  [[tables.columns]]
  name = "details"
  column_type = "json"
  ocsf_field = "raw_extensions.details"
```

All five columns correspond to `ActivityRecord` struct fields in `prism-dtu-armis/src/types.rs`, emitted by `routes::devices::get_device_activity` via `serde_json::to_value(&body)` where `body = ActivityResponse { data: ActivityData { activities, total } }`. SAP-2 §Rule 6 wire-emission-site authority: verified at the `get_device_activity` route handler in `prism-dtu-armis/src/routes/devices.rs §get_device_activity`.

**Required-filter implementation obligation (POL-29 9c):** The `path_template` uses `${query.filter.device_id}`. Confirmed current-code behavior when no `device_id` filter is present: (1) `FetchContext.query_filters` has no `device_id` key; (2) the `F-LP1-HIGH-004` pre-seed loop in `execute_impl §execute_impl` does NOT insert `query.filter.device_id`; (3) `seed_missing_query_filter_vars §seed_missing_query_filter_vars` inserts `query.filter.device_id = ""` (empty string); (4) interpolation yields path `/api/v1/devices//activity`; (5) the DTU handler receives `device_id = ""` via axum path extraction; (6) the fixture filter matches nothing; (7) **the handler returns HTTP 200 with an empty activities array and `total: 0`** — a silent empty result (Standing Rule 3 §2 / SOUL.md #4 violation). The implementing story MUST ensure queries against `armis_device_activity` without a `device_id` predicate produce a hard error (`SpecEngineError::HttpRequestFailed` or equivalent) BEFORE issuing any HTTP request. Required-filter anchor: `S-WAVE-A-ARMIS-ACTIVITY-001` **AC-004 / RG-004** (`test_armis_device_activity_absent_device_id_filter_returns_hard_error`). TOML-declaration anchor: `S-WAVE-A-ARMIS-ACTIVITY-001` **AC-001 / RG-001** (`test_armis_toml_armis_device_activity_table_declared_with_correct_step_block`).

## Verification Properties

None assigned yet. Verification properties will be added when `S-WAVE-A-ARMIS-ACTIVITY-001` is dispatched to test-writer.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| Capability Anchor Justification | CAP-003 ("OCSF Normalization") per capabilities.md §CAP-003 — this BC specifies the fetch pipeline contract and OCSF field mappings for the Armis device activity surface, which is exactly what CAP-003 ("OCSF Normalization") defines: normalize sensor records to OCSF v1.x via the DynamicMessage protobuf pattern, mapping vendor-specific fields to a common schema |
| L2 Invariants | DI-005 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | FB81 | 2026-07-28 | product-owner | F-WASE-P68-HIGH-003 — ADR-033 T1 mis-citation corrected (dimension 9b downstream-copy-target fix). §Architecture Anchors: (a) phantom path `ADR-033-push-down-filters.md` corrected to real file `ADR-033-push-down-time-window-extraction-strategy-pre-fan-out-heuristic.md`; (b) ADR-033 description updated — governs datetime time-window extraction via `extract_time_window_from_ast §extract_time_window_from_ast` only, NOT the authority for equality push-down routing; (c) ADR-057 description updated from "grammar and single-device scope decision" to "grammar and `${query.filter.*}` pre-seed mechanism (§D4, §D5)" for precision. §Preconditions: `device_id` column precondition rewritten — "via the push-down extraction path (ADR-033 T1)" replaced with annotation-agnostic routing description aligned to ADR-057 §D5 v0.5: `predicate_tree_to_filter_map §predicate_tree_to_filter_map` collects all equality predicates regardless of annotation → `FetchContext.query_filters` → `execute_impl §execute_impl` pre-seeds `step_vars["query.filter.device_id"]` per ADR-057 §D4. POL-29: 9a — no named twin (confirmed per v1.1 FB77 POL-29 9a record; BC-2.02.006 is parent, not split-event sibling); 9b — Sites 2 and 3 in this file are downstream copies of the corrected ADR-057 §D5; no further downstream copies of the T1 claim exist outside this BC and BC-2.02.006 (confirmed by `.factory/specs/` grep); 9c — no new unanchored MUSTs introduced. |
| 1.1 | FB77 | 2026-07-27 | product-owner | POL-29 9c mandate discharge: substituted real story AC/RG anchors for pending placeholders. TOML-declaration MUST anchored to `S-WAVE-A-ARMIS-ACTIVITY-001` AC-001 / RG-001 (`test_armis_toml_armis_device_activity_table_declared_with_correct_step_block`). Required-filter MUST anchored to `S-WAVE-A-ARMIS-ACTIVITY-001` AC-004 / RG-004 (`test_armis_device_activity_absent_device_id_filter_returns_hard_error`). Tightened failure-mode description per story v1.2 §Code-Reading Verdict: pre-fix behavior is HTTP 200 + `activities: []` + `total: 0` (silent empty result via path `/api/v1/devices//activity`), not a URL construction abort; `seed_missing_query_filter_vars §seed_missing_query_filter_vars` inserts empty-string seed. No `InterpolationError::FieldNotFound` claim present to remove (consistent with pre-seed guarantee). Updated §Postconditions absent-filter line, §Error Cases row 1, and EC-014-001 with observable pre-fix behavior. POL-29 9a: no named twin (BC-2.02.006 is parent, not split-event sibling). 9b: §TOML Contract TOML block is copy-source for story T-IMPL-01; anchor-text additions are prose-only and do not alter the TOML block, so no downstream copy drift. 9c: both mandate anchors now carry real AC + RGT IDs; no new unanchored MUST introduced. |
| 1.0 | FB73 | 2026-07-27 | product-owner | Initial authoring per ADR-057 §C2. Specifies `armis_device_activity` fetch pipeline contract: push-down grammar `${query.filter.device_id}` per ADR-057 §D5; five SAP-2-verified ActivityRecord columns (activity_id, device_id, activity_type, timestamp, details) verified against `ActivityRecord` in `prism-dtu-armis/src/types.rs` and wire-emission site `routes::devices::get_device_activity §get_device_activity`; OCSF mappings; single-device filter-required scope per ADR-057 §D4; absent-filter hard-error contract; required-filter implementation obligation delegated to `S-WAVE-A-ARMIS-ACTIVITY-001`. ITEM 3 decision: new BC justified over BC-2.02.006 amendment because (a) BC-2.02.006 is a field-mapping contract with preconditions that presuppose a record has already been fetched, while this contract governs fetch-request construction and filter-required preconditions — a distinct behavioral class; (b) BC-2.02.006 H1 ("Armis Centrix Field Mapping to OCSF (7 Data Sources)") does not encompass request-construction contracts; (c) ADR-057 §C2 explicitly says "author BCs", implying new contract creation. POL-29 9a: no named twin (this BC has no split-event sibling; BC-2.02.006 is the parent, not a twin). 9b: ADR-057 §D5 is the verbatim copy-source for the `path_template` text and TOML block — swept faithfully. 9c: required-filter MUST anchored to `S-WAVE-A-ARMIS-ACTIVITY-001` (real existing story ID per ADR-057 §C1); TOML column MUSTs anchored to same story; AC/RGT backfill pending story-writer update. |
