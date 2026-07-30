---
document_type: story
story_id: S-WAVE-A-ARMIS-ACTIVITY-001
title: "Armis Device Activity TOML Surface — Add armis_device_activity Table to Spec"
version: "1.8"
modified: "2026-07-30"
status: ready
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P2
points: 8
tdd_mode: strict
target_module: prism-sensors
subsystems: ["SS-06 (SensorSpec)", "SS-07 (SpecEngine)", "SS-12 (DTU-Armis)"]
crates_touched:
  - prism-sensors        # armis.sensor.toml: add armis_device_activity table block
  - prism-spec-engine    # required-filter hard error fix; pipeline integration tests
  - prism-dtu-armis      # wire-shape tests + generator activity builder + seeded-state population
depends_on: []
blocks: []
behavioral_contracts: [BC-2.02.006, BC-2.02.014]
verification_properties: []
assumption_validations: []
risk_mitigations: []
estimated_days: 3
---

# S-WAVE-A-ARMIS-ACTIVITY-001: Armis Device Activity TOML Surface

## Authority

**ADR-057** (accepted 2026-07-27) is the authoritative design decision governing the
`${query.filter.*}` filter-push-down grammar for parameterized sensor path templates.
ADR-057 adjudicated the single-device activity lookup path for Armis: confirmed
`path_template = "/api/v1/devices/${query.filter.device_id}/activity"` with
filter-required semantics, and documented per-record fan-out as a capability gap
requiring a separate cross-sensor story (ADR-057 §D6).

Read ADR-057 in full before implementing:
`.factory/specs/architecture/decisions/ADR-057-armis-activity-per-device-push-down-grammar.md`

This story is the resolution target for BC-2.02.006 EC-02-014 (product-owner, FB68d),
and the anchor for ADR-057's `armis_device_activity` surface adjudication.

---

## Narrative

As a Prism maintainer, I want `armis.sensor.toml` to declare an `armis_device_activity`
table so that the `get_device_activity` DTU route (which already exists in
`prism-dtu-armis::routes::devices`) becomes reachable from spec-driven queries, enabling
per-device activity timeline queries for the Armis sensor surface.

---

## Architect Adjudication — Grammar Confirmed (2026-07-27)

**Status: UNBLOCKED** — ADR-057 (architect, 2026-07-27) adjudicated the filter-push-down
grammar. BC-2.02.014 (product-owner, FB73) completed the behavioral contract. Story is
fully unblocked and is now `status: ready`.

### Adjudication Outcome

The confirmed `path_template` for single-device activity queries is:

```
path_template = "/api/v1/devices/${query.filter.device_id}/activity"
```

The `${query.filter.*}` namespace is pre-seeded from `FetchContext.query_filters` before
the steps loop begins. When a query carries `WHERE device_id = 'X'` (a filter push-down),
`query.filter.device_id` interpolates the device ID into the path_template at request time.
The `device_id` column must declare `options = ["INDEX"]` to enable filter extraction.

Per-record fan-out (iterating over all devices from a prior step) is documented as a
genuine capability gap in **ADR-057 §D6** — it requires a separate cross-sensor story
and is NOT in scope for this story. This story covers the single-device
filter-required lookup path only.

This deferral is documented in **BC-2.02.006 EC-02-014** (product-owner, FB68d).
The `${query.filter.device_id}` adjudication is the resolution of that deferral per
Canonical Principle Rule 3.

---

## Code-Reading Verdict: seed_missing_query_filter_vars

**CONFIRMED from direct code reading before authoring ACs.**

`seed_missing_query_filter_vars` (in `crates/prism-spec-engine/src/pipeline.rs
§seed_missing_query_filter_vars`) pre-seeds absent `${query.filter.*}` slots with
**empty string** (`serde_json::Value::String(String::new())`). Relevant excerpt:

```rust
step_vars
    .entry(full_key)
    .or_insert(serde_json::Value::String(String::new()));
```

This means: if a query against `armis_device_activity` lacks a `WHERE device_id = '...'`
predicate, then `FetchContext.query_filters["device_id"]` is absent, the initial
pre-seed loop in `execute_impl §F-LP1-HIGH-004` does NOT insert it, and then
`seed_missing_query_filter_vars` inserts `query.filter.device_id = ""` (empty string).

The interpolated URL becomes `/api/v1/devices//activity` — an empty path segment.
The DTU's `get_device_activity` handler receives `device_id = ""` via axum path
extraction, filters `activity_fixture` by `device_id == ""`, finds no matches, and
returns `ActivityResponse { data: ActivityData { activities: [], total: 0 } }` with
HTTP 200. **This is a silent empty result — a violation of BC-2.02.014 §Error Cases.**

**Summary of current behavior vs. required behavior:**

| Query | Current behavior | Required behavior (BC-2.02.014) |
|-------|-----------------|--------------------------------|
| No `device_id` filter | Silent empty result (empty-string seed → malformed URL → HTTP 200 + `[]`) | Hard error (`SpecEngineError::HttpRequestFailed`); zero records |
| `WHERE device_id = 'X'` | Correct — interpolates to `/api/v1/devices/X/activity` | Same (no change) |

**Note on ADR-057 §D4:** ADR-057 §D4 documents `seed_missing_query_filter_vars
§seed_missing_query_filter_vars` pre-seeding absent `${query.filter.*}` slots with
empty string — confirming that `FieldNotFound` does NOT fire for `${query.filter.*}`
slots, and the pre-seeding behavior IS the current ground truth. The earlier text
in retired v0.1/v0.2 of ADR-057 (which stated `Interpolator::interpolate` would abort
on a missing key) has been superseded; ADR-057 §D4 is the current authority.
Confirmed outcome: HTTP 200 with `activities: [], total: 0` (not 404). AC-004/RG-004
red-gate premise remains valid: the hard-error requirement is specified in BC-2.02.014,
which the engine must be brought into alignment with via T-IMPL-02.

---

## Ground-Truth DTU State (confirmed from code)

| Item | Source | State |
|------|--------|-------|
| `ActivityRecord` struct | `crates/prism-dtu-armis/src/types.rs §ActivityRecord` | Present: `activity_id: String`, `device_id: String`, `activity_type: String`, `timestamp: String`, `details: serde_json::Value` |
| `ActivityData` struct | `crates/prism-dtu-armis/src/types.rs §ActivityData` | Present: `activities: Vec<ActivityRecord>`, `total: u32` |
| `ActivityResponse` struct | `crates/prism-dtu-armis/src/types.rs §ActivityResponse` | Present: `data: ActivityData` |
| `get_device_activity` handler | `crates/prism-dtu-armis/src/routes/devices.rs §get_device_activity` | Fully implemented, auth-checked, filters by `device_id`, returns `ActivityResponse` |
| Wire-emission site | `crates/prism-dtu-armis/src/routes/devices.rs §get_device_activity` — `(StatusCode::OK, Json(body))` where `body = ActivityResponse { data: ActivityData { activities, total } }` | **One path only** (static fixture, no generated-records path). All five `ActivityRecord` fields serialized via axum `Json` wrapper |
| Route registration | `crates/prism-dtu-armis/src/clone.rs §build_router` | Registered at `GET /api/v1/devices/:device_id/activity` |
| `armis.sensor.toml` activity surface | `crates/prism-sensors/specs/armis.sensor.toml` | No `armis_device_activity` table declared — surface unreachable from spec-driven queries |

**SAP-2 parity result (wire-emission site `§get_device_activity`):** All five
`ActivityRecord` fields (`activity_id`, `device_id`, `activity_type`, `timestamp`,
`details`) are emitted via `Json(body)` serialization at the single wire-emission site.
No generated-records path exists for this handler — the absence of a generated-records
branch is the defect addressed by MED-005, not its clearance (see AC-008/RG-008
`test_armis_device_activity_seeded_mode_returns_activity_records_for_generated_device_ids`).
SAP-2 parity for the static-fixture path: **PASS** — no P1 CRITICAL at the
wire-emission site. The seeded-mode data-reachability gap (MED-005) is a separate
structural defect contracted by AC-008/RG-008.

---

## Acceptance Criteria

### AC-001: `armis_device_activity` table declared with correct step block
(traces to BC-2.02.014 precondition 1 AND BC-2.02.006 EC-02-014 deferral closure)

`armis.sensor.toml` declares an `armis_device_activity` table with a step block carrying
`name = "fetch_device_activity"`, `method = "GET"`,
`path_template = "/api/v1/devices/${query.filter.device_id}/activity"`, and
`response_path = "$.data.activities"`.

This AC also closes the BC-2.02.006 EC-02-014 deferral sentinel — the
`armis_device_activity` surface is no longer deliberately excluded from the spec.

Anchor: RG-001 (`test_armis_toml_armis_device_activity_table_declared_with_correct_step_block`).

### AC-002: `device_id` column carries `options = ["INDEX"]`
(traces to BC-2.02.014 precondition 2)

The `device_id` column in the `armis_device_activity` table MUST declare
`options = ["INDEX"]`, declaring push-down eligibility per the BC-2.11.007 taxonomy
(REQUIRED / INDEX / ADDITIONAL) for future T2 (`classify_predicates §classify_predicates`)
integration. The current routing is annotation-agnostic: `predicate_tree_to_filter_map
§predicate_tree_to_filter_map` collects all case-sensitive `field = 'string'` equality
predicates regardless of column annotation into the `FilterMap`, which materializes as
`FetchContext.query_filters`; `execute_impl §execute_impl` then pre-seeds
`step_vars["query.filter.device_id"]` from that map (ADR-057 §D4). ADR-033 T1 is NOT
the authority here — ADR-033 T1 governs datetime time-window extraction only
(authority: `extract_time_window_from_ast §extract_time_window_from_ast`).

Anchor: RG-002 (`test_armis_toml_armis_device_activity_device_id_column_has_index_option`).

### AC-003: All five `ActivityRecord` columns declared with correct types (SAP-2)
(traces to BC-2.02.014 postcondition 4 — emitted record schema)

The `armis_device_activity` table declares all five `ActivityRecord` columns with the
correct `column_type` and `ocsf_field` values per BC-2.02.014 §TOML Contract:

| TOML column | `column_type` | `ocsf_field` | `ActivityRecord` field |
|-------------|---------------|--------------|------------------------|
| `activity_id` | `string` | `raw_extensions.activity_id` | `activity_id: String` |
| `device_id` | `string` | `device.uid` | `device_id: String` |
| `activity_type` | `string` | `activity_name` | `activity_type: String` |
| `timestamp` | `string` | `time` | `timestamp: String` |
| `details` | `json` | `raw_extensions.details` | `details: serde_json::Value` |

Anchor: RG-003 (`test_armis_toml_armis_device_activity_has_all_five_activity_record_columns`).

### AC-004: Absent `device_id` filter returns hard error, not silent empty result
(traces to BC-2.02.014 §Error Cases row 1 / invariant filter-required / EC-014-001)

**This AC FAILS against the current engine implementation — correct Red Gate behavior.**

A query `SELECT * FROM armis_device_activity` without a `WHERE device_id = '...'`
predicate MUST produce a hard error (`SpecEngineError::HttpRequestFailed` with
`status_code = 0`, E-SPEC-029). Zero records are returned. The pipeline MUST NOT
silently produce an empty result set via a malformed URL path.

Current engine behavior (confirmed from code): `seed_missing_query_filter_vars
§seed_missing_query_filter_vars` in `crates/prism-spec-engine/src/pipeline.rs` pre-seeds
absent `query.filter.device_id` with empty string, producing path
`/api/v1/devices//activity`. DTU returns HTTP 200 + empty activities array — a silent
empty result. The spec wins per CLAUDE.md Source-of-Truth Precedence; the engine must
be fixed (see T-IMPL-02).

Anchor: RG-004 (`test_armis_device_activity_absent_device_id_filter_returns_hard_error`).

### AC-005: Successful single-device query fetches correct URL and returns records
(traces to BC-2.02.014 postconditions 1, 2, 3)

A query `SELECT * FROM armis_device_activity WHERE device_id = 'd-001'` against a
DTU seeded with activity fixture for `d-001` MUST:
1. Pre-seed `step_vars["query.filter.device_id"] = "dev-001"` via the
   `FetchContext.query_filters["device_id"]` loop (block comment `F-LP1-HIGH-004`
   in `pipeline.rs §execute_impl`)
2. Construct URL `<base_url>/api/v1/devices/d-001/activity` via interpolation
3. Fetch from `routes::devices::get_device_activity` and extract `$.data.activities`
4. Return an array of activity records scoped to `d-001`; all five columns present
   in each record

Anchor: RG-005 (`test_armis_device_activity_with_device_id_filter_fetches_correct_url_and_returns_records`).

### AC-006: Wire-level JSON response shape includes all five ActivityRecord fields
(traces to BC-2.02.014 postcondition 4 / 2026-07-13 wire-shape assertion discipline)

At least one test MUST assert on the SERIALIZED JSON bytes returned by
`GET /api/v1/devices/{device_id}/activity` from the DTU — not only on
pre-serialization Rust structures. The serialized response envelope MUST contain:

```json
{
  "data": {
    "activities": [
      {
        "activity_id": "<string>",
        "device_id": "<string>",
        "activity_type": "<string>",
        "timestamp": "<string>",
        "details": <value>
      }
    ],
    "total": <number>
  }
}
```

All five keys (`activity_id`, `device_id`, `activity_type`, `timestamp`, `details`)
MUST be present in each serialized activity record (null-not-absent discipline:
no key may be silently omitted per BC-2.11.001 EC-11-079).

Anchor: RG-006 (`test_armis_device_activity_dtu_response_json_shape_has_all_five_fields`).

### AC-007: Device with no activity records returns empty result set, not an error
(traces to BC-2.02.014 §Edge Cases EC-014-002)

A query `SELECT * FROM armis_device_activity WHERE device_id = 'no-such-device'`
against a DTU with no activity fixture entries for that device ID MUST return zero
records and no error (HTTP 200 with empty `activities` array → empty result set). This
is the normal case for a device that exists but has no recorded activity.

Anchor: RG-007 (`test_armis_device_activity_device_with_no_activities_returns_empty_result_set`).

### AC-008: Seeded-mode queries against generated device IDs return activity records, not empty results
(traces to BC-2.02.014 §Edge Cases EC-014-006 / §TOML Contract MED-005 generator obligation)

A query `SELECT * FROM armis_device_activity WHERE device_id = 'dev-<org_slug>-<seed>-0'`
in `fixture_gen_seeded=true` mode (after T-IMPL-04 is implemented) MUST return at least one
activity record. Root cause (MED-005): `routes::devices §get_device_activity` filters
`state.activity_fixture` only; both `new_with_seed §new_with_seed` and
`new_with_scenario §new_with_scenario` populate `state.activity_fixture` exclusively from
`fixtures/device-activity.json` (fixture device IDs: `d-001`, `d-002`, `d-005`, `d-013`,
`d-015`, `d-020`, `d-023`, `d-024`); generated device IDs (`dev-{org_slug}-{seed}-{i}`
from `generator.rs §build_asset`) are disjoint — zero overlap; surface inert in seeded mode.
Required fix (T-IMPL-04): (a) add an activity-record builder in `generator.rs` following the
`build_asset §build_asset` / `build_alert §build_alert` pattern, producing `ActivityRecord`
entries keyed to generated device IDs; (b) populate `state.activity_fixture` from BOTH
static fixture data AND generator-produced activity records in BOTH `new_with_seed
§new_with_seed` and `new_with_scenario §new_with_scenario` construction paths. Wire-shape
assertion: verify the serialized activity record contains all five fields at the wire level
(per AC-006 discipline).

Anchor: RG-008 (`test_armis_device_activity_seeded_mode_returns_activity_records_for_generated_device_ids`).

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Architecture Section |
|-----------|--------|---------------|----------------------|
| `armis.sensor.toml` (activity table) | `crates/prism-sensors/specs/` | Pure (config data) | `architecture/module-decomposition.md §SS-06 SensorSpec` |
| `seed_missing_query_filter_vars` fix | `crates/prism-spec-engine/src/pipeline.rs §seed_missing_query_filter_vars` | Pure (query transformation) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `get_device_activity` handler | `crates/prism-dtu-armis/src/routes/devices.rs §get_device_activity` | Effectful (HTTP handler) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |
| `ActivityRecord` / `ActivityData` / `ActivityResponse` | `crates/prism-dtu-armis/src/types.rs` | Pure (data types) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |

---

## Behavioral Contracts

| BC | Version | Relevance |
|----|---------|-----------|
| BC-2.02.006 | | Armis Centrix Field Mapping to OCSF (7 Data Sources) — EC-02-014 records the `armis_device_activity` deferral and its resolution to this story; AC-001 closes the deferral sentinel |
| BC-2.02.014 | | Armis Device Activity Surface — Filter-Required Push-Down Fetch Contract — specifies the full behavioral contract: push-down grammar, 5-column schema, required-filter hard error (`status_code = 0`, E-SPEC-029), edge cases EC-014-001..EC-014-006 including MED-005 seeded-mode data-reachability gap (AC-008/RG-008 generator obligation) |

---

## UX / Operator Surfaces

None — this story produces no user-facing UI changes. The surface changes are:
(1) addition of a new queryable table in `armis.sensor.toml`, and
(2) an engine-level required-filter guard in `crates/prism-spec-engine`.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `device_id` filter not provided in query | Hard error (`SpecEngineError::HttpRequestFailed` with `status_code = 0`, E-SPEC-029); zero records; no silent empty result; no malformed URL path segment — per BC-2.02.014 §Error Cases row 1 and EC-014-001 |
| EC-002 | Device has no activity records in DTU | Query returns zero records — not an error; per BC-2.02.014 EC-014-002 |
| EC-003 | `details` field contains nested JSON object | Serialized as `json` type per `column_type = "json"`; structure preserved in full; per BC-2.02.014 EC-014-004 |
| EC-004 | `details` is null in a record | `details` column emits null; not an error; nullable per `serde_json::Value` serialization; per BC-2.02.014 EC-014-005 |

---

## Tasks

### Red Gate tests (test-writer MUST author all RG tests BEFORE any implementation begins)

- [ ] **RG-001**: `test_armis_toml_armis_device_activity_table_declared_with_correct_step_block` — AC-001
  _(Parses `crates/prism-sensors/specs/armis.sensor.toml`; asserts `armis_device_activity`
  table exists; asserts step named `"fetch_device_activity"` has `method = "GET"`,
  `path_template` containing `${query.filter.device_id}`, and
  `response_path = "$.data.activities"`. Test fails before T-IMPL-01 adds the table.
  Mirrors existing column-assertion patterns in `crates/prism-spec-engine/tests/` or
  `crates/prism-sensors/tests/`.)_

- [ ] **RG-002**: `test_armis_toml_armis_device_activity_device_id_column_has_index_option` — AC-002
  _(Parses `armis.sensor.toml`; asserts the `armis_device_activity` table has a `device_id`
  column that declares `options = ["INDEX"]`; confirms push-down eligibility per BC-2.11.007
  taxonomy and ADR-057 §D4. Test fails before T-IMPL-01.)_

- [ ] **RG-003**: `test_armis_toml_armis_device_activity_has_all_five_activity_record_columns` — AC-003
  _(Parses `armis.sensor.toml`; asserts `armis_device_activity` table has exactly five
  columns with names `activity_id`, `device_id`, `activity_type`, `timestamp`, `details`
  and correct `column_type` values (`string`, `string`, `string`, `string`, `json`);
  SAP-2 parity check against `ActivityRecord §ActivityRecord` in
  `crates/prism-dtu-armis/src/types.rs`. Test fails before T-IMPL-01.)_

- [ ] **RG-004**: `test_armis_device_activity_absent_device_id_filter_returns_hard_error` — AC-004
  _(Pipeline integration test: issues a query against `armis_device_activity` with NO
  `WHERE device_id` predicate against a running DTU; asserts
  `SpecEngineError::HttpRequestFailed` with `status_code = 0` (E-SPEC-029) is returned;
  asserts zero records. Test fails on the current engine because `seed_missing_query_filter_vars
  §seed_missing_query_filter_vars` inserts an empty-string seed causing a silent empty
  result. Will remain failing until T-IMPL-02 provides the fix.)_

- [ ] **RG-005**: `test_armis_device_activity_with_device_id_filter_fetches_correct_url_and_returns_records` — AC-005
  _(Integration test with DTU seeded with activity fixture for `"d-001"` (fixture ID from
  `fixtures/device-activity.json`); query
  `SELECT * FROM armis_device_activity WHERE device_id = 'd-001'`; asserts pipeline
  constructs URL containing `/api/v1/devices/d-001/activity`; asserts returned records
  include all five column fields with non-empty values. Test fails before T-IMPL-01
  makes the surface reachable.)_

- [ ] **RG-006**: `test_armis_device_activity_dtu_response_json_shape_has_all_five_fields` — AC-006
  _(HTTP test against the DTU at `GET /api/v1/devices/d-001/activity` with DTU seeded
  with at least one `ActivityRecord` for `"d-001"` (fixture ID from
  `fixtures/device-activity.json`); deserializes the response body as
  `serde_json::Value`; asserts all five keys — `"activity_id"`, `"device_id"`,
  `"activity_type"`, `"timestamp"`, `"details"` — are present in
  `response["data"]["activities"][0]`. Wire-shape assertion per 2026-07-13 discipline.
  This test exercises only the DTU wire surface and passes on the current engine because
  the route is already implemented; it verifies the contract is upheld going forward.
  Preferred location: `crates/prism-dtu-armis/tests/`.)_

- [ ] **RG-007**: `test_armis_device_activity_device_with_no_activities_returns_empty_result_set` — AC-007
  _(Integration test; query `SELECT * FROM armis_device_activity WHERE device_id = 'no-such-device'`
  against DTU with no activity fixture for `"no-such-device"` (TV-BC-2.02.014-003 literal);
  asserts empty result set returned (zero rows); asserts no error condition. Test fails
  before T-IMPL-01 makes the table reachable from PrismQL.)_

- [ ] **RG-008**: `test_armis_device_activity_seeded_mode_returns_activity_records_for_generated_device_ids` — AC-008
  _(In `fixture_gen_seeded=true` mode, uses `new_with_seed §new_with_seed` DTU construction;
  queries `SELECT * FROM armis_device_activity WHERE device_id = 'dev-<org_slug>-<seed>-0'`
  (a generated device ID, distinct from fixture IDs `d-001` etc.); asserts the result set
  contains at least one activity record; asserts `activity_id`, `device_id`, `activity_type`,
  `timestamp`, `details` present in each record. Wire-shape assertion: verify serialized JSON
  activity record shape per AC-006 discipline. Test FAILS before T-IMPL-04 implements
  the generator activity builder and populates `state.activity_fixture` with generator-produced
  records. Preferred location: `crates/prism-dtu-armis/tests/`.)_

**Red Gate density check** (BC-5.38.001): **8 failing tests** required before
implementation begins. RG-001..RG-008 cover all 8 ACs one-to-one.
RED_RATIO is computed by the orchestrator at Step 3.5 per per-story-delivery.md from actual
Red Gate results; BC-5.38.002 and BC-5.38.003 define the exempt test classes (green-by-design
and wiring-exempt) that reduce the denominator.

---

### Implementation tasks (to be executed ONLY AFTER all RG tests are authored and failing)

### T-IMPL-01: Author `armis_device_activity` table in `armis.sensor.toml`
**Files:** `crates/prism-sensors/specs/armis.sensor.toml` (MODIFY)

Add `[[tables]]` block for `armis_device_activity`. Use the exact TOML verbatim from
BC-2.02.014 §TOML Contract (copy-source: ADR-057 §D5; authoritative):

**CRITICAL — registered surface vs declared `table_name`:**
`register_sensor §register_sensor` in `prism-query::table_registry §table_registry`
composes the registered SQL surface name as
`format!("{}_{}", spec.sensor_id, table.table_name)` =
`format!("{}_{}", "armis", "device_activity")` = `"armis_device_activity"`.
Declaring `table_name = "device_activity"` (not `"armis_device_activity"`) is required —
declaring `"armis_device_activity"` double-prefixes to `"armis_armis_device_activity"`
and breaks all consumers that query `FROM armis_device_activity`.

`TableSpec §TableSpec` requires both `pub table_name: String` and `pub ocsf_class: String`
with no `#[serde(default)]`; both are mandatory at deserialization time. The keys `name`
and `sensor_name` are NOT `TableSpec §TableSpec` fields and MUST NOT appear in the header.
Queries in this story that reference `FROM armis_device_activity` use the registered surface
name; that name is correct and unchanged.

```toml
[[tables]]
table_name = "device_activity"
ocsf_class = "network_activity"

  [[tables.steps]]
  name = "fetch_device_activity"
  method = "GET"
  path_template = "/api/v1/devices/${query.filter.device_id}/activity"
  response_path = "$.data.activities"
  variables_produced = []
  required_filters = ["device_id"]

  # SAP-2 verified: activity_id — ActivityRecord.activity_id: String
  # emitted by routes::devices::get_device_activity via (StatusCode::OK, Json(body)).into_response()
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

Consult existing table blocks in `armis.sensor.toml` for indentation and comment
formatting conventions. This task makes RG-001, RG-002, RG-003, RG-007 green
(the filter-required behavior still requires T-IMPL-02).

### T-IMPL-02: Implement required-filter hard error for `armis_device_activity`
**Files:** `crates/prism-spec-engine/src/pipeline.rs §execute_impl` (MODIFY), `crates/prism-spec-engine/src/spec_parser.rs §FetchStep` (MODIFY), `crates/prism-sensors/specs/armis.sensor.toml` (READ — `required_filters = ["device_id"]` declared in T-IMPL-01)

**Problem (confirmed from code):** `seed_missing_query_filter_vars §seed_missing_query_filter_vars` unconditionally pre-seeds absent `${query.filter.*}` slots with empty string. For `armis_device_activity`, an absent `device_id` filter produces the path `/api/v1/devices//activity` → DTU returns
HTTP 200 + empty array → silent empty result, violating BC-2.02.014 §Error Cases row 1.

**Required behavior:** When a query against `armis_device_activity` lacks a
`WHERE device_id = '...'` predicate, the pipeline MUST return
`SpecEngineError::HttpRequestFailed` (`status_code = 0`, E-SPEC-029) before issuing any
HTTP request. Anchor: AC-004 / RG-004.

**Ratified mechanism (ADR-057 §D7, ratified 2026-07-27):** Add
`#[serde(default)] pub required_filters: Vec<String>` to `FetchStep §FetchStep` in
`crates/prism-spec-engine/src/spec_parser.rs`. In `execute_impl §execute_impl`,
add a gate BEFORE `seed_missing_query_filter_vars §seed_missing_query_filter_vars` runs:
for each key in `step.required_filters`, if `FetchContext.query_filters` has no matching
entry, return `SpecEngineError::HttpRequestFailed` (`status_code = 0`, E-SPEC-029)
immediately, before any HTTP request is issued. The `armis_device_activity` step block
already declares `required_filters = ["device_id"]` per T-IMPL-01.

**`#[non_exhaustive]` obligation (ADR-057 §D7):** `FetchStep §FetchStep` already carries
`#[non_exhaustive]` in `crates/prism-spec-engine/src/spec_parser.rs` and is already
registered as `"FetchStep"` in `EXPECTED_SYMBOLS` in
`scripts/check-non-exhaustive-per-symbol.py`. Adding `pub required_filters: Vec<String>`
to an existing `#[non_exhaustive]` type does NOT require a new `EXPECTED_SYMBOLS` entry —
`EXPECTED_SYMBOLS` is a type-level check, not a field-level check. Do not add a new entry.

**The mechanism MUST NOT break** existing optional filter behavior:
- CrowdStrike `${query.filter._fql}` — absent is valid (no time predicate is normal)
- Armis `${query.filter.aql}` — absent is valid (table-level default AQL used)

Steps that omit `required_filters` (which is `#[serde(default)]`, defaults to `[]`)
continue to behave as before — the gate is a no-op when `required_filters` is empty,
and `seed_missing_query_filter_vars §seed_missing_query_filter_vars` pre-seeds with
empty string as currently.

**No empty-string defeat via post-fetch workaround:** the fix MUST be structural at the
pre-flight stage (Canonical Principle Rule 1).

This task makes RG-004 and RG-005 fully green (filter hard error + normal query path).

### T-IMPL-03: SAP-2 column parity verification
After T-IMPL-01, confirm every column in `armis_device_activity` corresponds to an
`ActivityRecord` field emitted at the wire-emission site `§get_device_activity`. The
five fields confirmed in code-reading:
`activity_id`, `device_id`, `activity_type`, `timestamp`, `details` — all present
on the struct and emitted via `Json(ActivityResponse { ... })`. No P1 CRITICAL.

### T-IMPL-04: Add activity-record builder to `generator.rs` and populate seeded state
**Files:** `crates/prism-dtu-armis/src/generator.rs` (MODIFY), `crates/prism-dtu-armis/src/clone.rs` (MODIFY)

**Problem (MED-005, confirmed from code):** `routes::devices §get_device_activity` filters
`state.activity_fixture` only. All three construction paths in
`crates/prism-dtu-armis/src/clone.rs` — `new_with_seed §new_with_seed` (a 4-line
`#[cfg(feature = "fixture-gen")]` delegation to `new_with_seed_anchored §new_with_seed_anchored`),
`new_with_seed_anchored §new_with_seed_anchored`, and `new_with_scenario §new_with_scenario`
— load `activity_fixture` exclusively from `fixtures/device-activity.json` via
`prism_dtu_common::load_fixture_as(crate_dir, "device-activity")` (fixture device IDs:
`d-001`, `d-002`, `d-005`, `d-013`, `d-015`, `d-020`, `d-023`, `d-024`).
`ArmisState::with_admin_token §ArmisState::with_admin_token` in `state.rs` merely assigns
the parameter to `activity_fixture` without referencing the file path. Generated device IDs
(`dev-{org_slug}-{seed}-{i}` from `generator.rs §build_asset`) have zero overlap with
fixture IDs. The `armis_device_activity` surface is inert in `fixture_gen_seeded=true` mode.

**Required changes:**

**(a) Generator activity builder:** Add an activity-record builder function in `generator.rs`
following the `build_asset §build_asset` / `build_alert §build_alert` pattern. The builder
produces `ActivityRecord` entries keyed to generated device IDs
(`dev-{org_slug}-{seed}-{i}` format matching what `build_asset §build_asset` generates).
Each record MUST populate all five `ActivityRecord` fields: `activity_id`, `device_id`,
`activity_type`, `timestamp`, `details`.

**(b) Seeded-state population:** In ALL THREE construction paths in `clone.rs` —
`new_with_seed §new_with_seed` (4-line delegation to `new_with_seed_anchored §new_with_seed_anchored`),
`new_with_seed_anchored §new_with_seed_anchored`, and `new_with_scenario §new_with_scenario`
— populate `state.activity_fixture` from BOTH: (1) static fixture records from
`fixtures/device-activity.json` (unchanged); AND (2) generator-produced activity records
from the new builder, so that `routes::devices §get_device_activity` can match queries
against generated device IDs.

This task makes RG-008 green (seeded-mode queries against generated device IDs return records).
Anchor: AC-008 / RG-008 (`test_armis_device_activity_seeded_mode_returns_activity_records_for_generated_device_ids`).

---

## Forbidden Dependencies

The `seed_missing_query_filter_vars` fix in T-IMPL-02 MUST NOT introduce a dependency
from `prism-spec-engine` onto `prism-dtu-armis`. The required-filter mechanism MUST
be expressed entirely within `prism-spec-engine` via TOML-level declarations or
engine-internal state. If the build gains `prism-spec-engine → prism-dtu-armis` as
a new dependency, the build MUST fail.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~4,300 |
| `armis.sensor.toml` (current state, reference) | ~2,000 |
| `crates/prism-dtu-armis/src/types.rs §ActivityRecord` area | ~500 |
| `crates/prism-dtu-armis/src/generator.rs` (activity builder context) | ~600 |
| `crates/prism-dtu-armis/src/clone.rs` (`new_with_seed_anchored §new_with_seed_anchored`, `new_with_scenario §new_with_scenario` construction paths) | ~400 |
| `crates/prism-spec-engine/src/pipeline.rs §execute_impl` and `§seed_missing_query_filter_vars` context | ~1,500 |
| `crates/prism-spec-engine/src/spec_parser.rs §FetchStep` (T-IMPL-02 `required_filters` field addition) | ~300 |
| ADR-057 (filter-push-down grammar adjudication, §D7 ratified mechanism) | ~1,000 |
| BC-2.02.014 (full contract) | ~1,500 |
| BC-2.02.006 §EC-02-014 (deferral sentinel) | ~500 |
| Running test output (nextest per-crate) | ~1,000 |
| **Total estimate** | **~13,600** |

13,600 tokens is within 20% context-window limit for most agent models. No story split required.

---

## Previous Story Intelligence

**From F-SAP2-MED-006 probe (wave-a-sap2-probe-pass-65):**
- `ActivityRecord`, `ActivityResponse`, `ActivityData` confirmed present in
  `prism-dtu-armis::types`; `get_device_activity` handler confirmed fully implemented
  and registered at `GET /api/v1/devices/:device_id/activity` in `§build_router`
- The DTU is ready; only the TOML spec and required-filter engine fix are missing

**From S-WAVE-A-ARMIS-REMEDIATION-001 (predecessor wave-a story):**
- Armis uses bearer static auth; `check_bearer_auth` pattern already established
- Activity route is auth-checked; no additional auth work needed in this story

**From ADR-057 adjudication (2026-07-27) and code-reading in FB75:**
- `${query.filter.device_id}` filter-push-down grammar confirmed — single-device lookup only
- Per-record fan-out is a capability gap (ADR-057 §D6) — NOT in scope for this story
- `seed_missing_query_filter_vars §seed_missing_query_filter_vars` pre-seeds absent
  filter slots with empty string — ADR-057 §D4 documents this behavior and confirms
  that `FieldNotFound` does NOT fire for `${query.filter.*}` slots; this is the current
  ground truth (supersedes retired v0.1/v0.2 abort-on-missing text). Confirmed outcome:
  HTTP 200 with `activities: [], total: 0` (not 404). AC-004/RG-004 red-gate premise
  remains valid: BC-2.02.014 governs; engine must be brought into alignment via T-IMPL-02.
  ADR-057 §D7 ratifies the `required_filters = ["device_id"]` TOML field as the
  implementation mechanism.

**From S-WAVE-A-ARMIS-SPEC-001 sibling story (POL-29 9a sweep):**
- That story modifies `armis.sensor.toml` devices table only; this story adds a new
  `armis_device_activity` table. Both stories have `depends_on: []` and modify the
  same file in non-overlapping sections.
- Implementer note: if both stories are implemented in the same wave, rebase before
  final merge to avoid TOML append-conflict in `armis.sensor.toml`.
- No content asymmetry detected in how the two stories describe `armis.sensor.toml`.

---

## Architecture Compliance Rules

1. **SAP-2 mandatory:** every TOML column in `armis_device_activity` MUST have a
   corresponding field in `ActivityRecord §ActivityRecord` (confirmed in
   `crates/prism-dtu-armis/src/types.rs`). Column in TOML with no DTU struct field
   = P1 CRITICAL per SAP-2 protocol. Anchor: AC-003 / RG-003.

2. **Wire-shape assertion discipline (CLAUDE.md):** any test covering the DTU HTTP
   surface must assert on serialized JSON output. At least one test must assert on
   the serialized `ActivityResponse` shape from the DTU at the wire level.
   Anchor: AC-006 / RG-006.

3. **ADR-028 §D1 — DTU-grounded spec authoring:** `path_template` must match the DTU
   route exactly. The registered route is `GET /api/v1/devices/:device_id/activity`
   per `§build_router`. Anchor: AC-001 / RG-001.

4. **Variable injection grammar — confirmed:** Use `${query.filter.device_id}` syntax
   (filter-push-down namespace, ADR-057 §D4). The `device_id` column MUST declare
   `options = ["INDEX"]` to enable filter extraction. Do NOT use the superseded
   `${variable.*}` syntax (phantom — zero workspace occurrences).
   Anchor: AC-002 / RG-002.

5. **Required-filter hard error:** the engine MUST detect and reject queries against
   `armis_device_activity` that lack a `device_id` filter, returning
   `SpecEngineError::HttpRequestFailed` (`status_code = 0`, E-SPEC-029). The fix MUST NOT
   break existing optional filter behavior (CrowdStrike `_fql`, Armis `aql`).
   Ratified mechanism: ADR-057 §D7 (`required_filters` TOML field + `execute_impl §execute_impl`
   pre-flight gate). Anchor: AC-004 / RG-004.

6. **No empty-string defeat via post-fetch workaround:** the fix must be structural
   at the pre-flight or pre-seed stage. A workaround that adds a post-fetch empty-
   response check is a defer-pattern anti-pattern (Canonical Principle Rule 1).

---

## Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|----------------|
| `axum` | pinned in workspace `Cargo.toml` | `architecture/dependency-graph.md §External Dependencies` |
| `serde` / `serde_json` | pinned in workspace `Cargo.toml` | same |
| `regex` | pinned in workspace `Cargo.toml` (used by `seed_missing_query_filter_vars §seed_missing_query_filter_vars`) | same |

No new external dependencies are introduced by this story.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-sensors/specs/armis.sensor.toml` | MODIFY | T-IMPL-01: add `[[tables]]` block for `armis_device_activity` per BC-2.02.014 §TOML Contract; `table_name = "device_activity"`, `ocsf_class = "network_activity"` (NOT `name`/`sensor_name`); declares `required_filters = ["device_id"]` on the step block; T-IMPL-02 engine gate reads this field (READ) |
| `crates/prism-spec-engine/src/pipeline.rs §execute_impl` | MODIFY | T-IMPL-02: add required-filter pre-flight gate BEFORE `seed_missing_query_filter_vars §seed_missing_query_filter_vars`; returns `SpecEngineError::HttpRequestFailed` (`status_code = 0`, E-SPEC-029) when a `required_filters` key is absent from `FetchContext.query_filters` |
| `crates/prism-spec-engine/src/spec_parser.rs §FetchStep` | MODIFY | T-IMPL-02: add `#[serde(default)] pub required_filters: Vec<String>` to `FetchStep §FetchStep` (already in EXPECTED_SYMBOLS; no new EXPECTED_SYMBOLS entry required) |
| `crates/prism-dtu-armis/src/generator.rs` | MODIFY | T-IMPL-04(a): add activity-record builder function following `build_asset §build_asset` / `build_alert §build_alert` pattern; produces `ActivityRecord` entries keyed to generated device IDs (`dev-{org_slug}-{seed}-{i}`) |
| `crates/prism-dtu-armis/src/clone.rs` | MODIFY | T-IMPL-04(b): in all three construction paths (`new_with_seed §new_with_seed`, `new_with_seed_anchored §new_with_seed_anchored`, `new_with_scenario §new_with_scenario`), populate `state.activity_fixture` from both `fixtures/device-activity.json` AND generator-produced activity records (MED-005 seeded-mode gap fix) |
| `crates/prism-spec-engine/tests/` (new or existing test file) | CREATE/MODIFY | RG-001, RG-002, RG-003 (TOML spec assertions); RG-004, RG-005, RG-007 (pipeline integration tests) |
| `crates/prism-dtu-armis/tests/` (new or existing test file) | CREATE/MODIFY | RG-006 (wire-shape assertion against `§get_device_activity`); RG-008 (seeded-mode generated-device-ID reachability assertion) |

---

## Verification Properties

None assigned yet. To be added when product-owner authors VP entries for the
`armis_device_activity` surface per BC-2.02.014 §VP Anchors.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.8 | 2026-07-30 | story-writer | FB96 story leg — close F-WASE-P70-HIGH-002 (T-IMPL-04 Files `state.rs`→`clone.rs`; Problem paragraph now cites `clone.rs` + all three construction paths `new_with_seed §new_with_seed`/`new_with_seed_anchored §new_with_seed_anchored`/`new_with_scenario §new_with_scenario`; T-IMPL-04(b) updated to ALL THREE paths; §FSR `state.rs` row replaced by `clone.rs` row; §Token Budget `state.rs §ArmisState` replaced by `clone.rs` row), F-WASE-P70-HIGH-005 (T-IMPL-02 replaced: "candidate approaches (non-exhaustive)" removed; ADR-057 §D7 ratified mechanism with `#[serde(default)] pub required_filters: Vec<String>` on `FetchStep §FetchStep`, pre-flight gate in `execute_impl §execute_impl` before `seed_missing_query_filter_vars`; `required_filters = ["device_id"]` added to T-IMPL-01 TOML step block; `armis.sensor.toml` + `spec_parser.rs` added to T-IMPL-02 Files; `spec_parser.rs` row added to §FSR and §Token Budget; EXPECTED_SYMBOLS no-new-entry note), F-WASE-P70-MED-003 (AC-005 `dev-001`→`d-001`; RG-005 `"dev-001"`→`"d-001"`; RG-006 `"dev-001"`→`"d-001"`, 3 sites), F-WASE-P70-MED-010 (AC-004 and EC-001 bind to `status_code = 0` + E-SPEC-029; remove "or equivalent"; RG-004 updated; §Architecture Compliance rule 5 updated), F-WASE-P70-LOW-001 (AC-007 `'no-activity-device'`→`'no-such-device'`; RG-007 `'empty-device'`→`'no-such-device'`, cite TV-BC-2.02.014-003). De-pinning (POL-39 early adoption): BC-2.02.006 `v1.15` and BC-2.02.014 `v1.6` removed from §Behavioral Contracts table version column; `ADR-057 v0.7 §D4` de-pinned at 2 live body sites (§Code-Reading Verdict, §Previous Story Intelligence); T-IMPL-01 BC/ADR prose de-pinned (`BC-2.02.014 v1.6 §TOML Contract` → `BC-2.02.014 §TOML Contract`, `ADR-057 v0.7 §D5` → `ADR-057 §D5`); §Token Budget ADR-057/BC-2.02.014/BC-2.02.006 version pins removed; §Token Budget total ~13,300→~13,600. POL-29 9a: SPEC-001 twin updated in same burst. 9b: T-IMPL-01 TOML block remains verbatim copy-source from BC-2.02.014 §TOML Contract (now including `required_filters`); 9b CLEAR. 9c: all new MUSTs carry AC+RGT anchors. |
| 1.7 | 2026-07-29 | story-writer | FB93 leg 2 — advance BC-2.02.006 pin v1.13 → v1.15 at two live sites (§Behavioral Contracts table row, §Token Budget Estimate row) and BC-2.02.014 pin v1.4 → v1.6 at three live sites (§Behavioral Contracts table row, T-IMPL-01 BC cross-reference prose, §Token Budget Estimate row). Pin-only; no mechanism or design content changed in either BC between previous and new versions (BC→story anchor-form conversion and convention note addition only). §Behavioral Contracts relevance cells for both BCs verified accurate at new versions; no description corrections needed. T-IMPL-01 TOML block (copy-source from BC-2.02.014 §TOML Contract) unchanged — 9b CLEAR. POL-29 9a: SPEC-001 twin updated in same burst; both now pin BC-2.02.006 at v1.15. 9c: no new MUSTs introduced. |
| 1.6 | 2026-07-29 | story-writer | FB91 story leg — propagate BC-2.02.006 v1.13 + BC-2.02.014 v1.4 + ADR-057 v0.7 spec corrections into ACTIVITY-001. (2a) T-IMPL-01 TOML header replaced (CRIT-002 + HIGH-004): `name`/`sensor_name` are not `TableSpec §TableSpec` fields; correct header per ADR-057 v0.7 §D5 transcribed verbatim from BC-2.02.014 v1.4 §TOML Contract: `table_name = "device_activity"` / `ocsf_class = "network_activity"`. `register_sensor §register_sensor` composition explained (`format!("{}_{}", "armis", "device_activity")` = `"armis_device_activity"`); double-prefix trap documented; mandatory-field requirement stated. SAP-2 `activity_id` comment updated from "serde_json (Json wrapper)" to "(StatusCode::OK, Json(body)).into_response()"; timestamp comment updated from "(ISO-8601 wire string)" to "(ISO-8601)". (2b) AC-008/RG-008 added (`test_armis_device_activity_seeded_mode_returns_activity_records_for_generated_device_ids`): MED-005 seeded-mode data-reachability gap — `routes::devices §get_device_activity` filters `state.activity_fixture` only; fixture IDs (`d-001` etc.) disjoint from generated IDs (`dev-{org_slug}-{seed}-{i}`); surface inert in seeded mode. T-IMPL-04 added: (a) activity-record builder in `generator.rs` following `build_asset §build_asset`/`build_alert §build_alert` pattern; (b) populate `state.activity_fixture` from both static fixture AND generator-produced records in both `new_with_seed §new_with_seed` and `new_with_scenario §new_with_scenario`. §Ground-Truth DTU State SAP-2 characterization corrected: absence of generated-records branch is the defect (MED-005), not its clearance. SAC-1: RG-008 placed before density check and implementation tasks. (2c) MED-007 — two "contradicts ADR-057 §D4" sites corrected: §Code-Reading Verdict "Note on ADR-057 §D4" and §Previous Story Intelligence both re-pointed to ADR-057 v0.7 as current ground truth; confirmed outcome HTTP 200 with `activities: [], total: 0` (not 404); AC-004/RG-004 red-gate premise remains valid. (2d) Red Gate density check 7→8; BC-2.02.006 v1.12→v1.13 and BC-2.02.014 v1.3→v1.4 in §Behavioral Contracts (relevance cell for v1.4 expanded to include EC-014-006 and MED-005/AC-008/RG-008 generator obligation); ADR-057 re-cited as v0.7 in Note on §D4 and §Previous Story Intelligence; all ADR-057 §… cites verified against real section headings (§D4, §D6, §Consequences (C1)) — no `§C1`/`§C2` bare-label cites in story body; points 5→8 (T-IMPL-04 scope: generator builder + seeded-state population; new test RG-008); estimated_days 1→3; `crates_touched` comment updated for prism-dtu-armis; §File Structure Requirements: added `generator.rs` (T-IMPL-04a) and `state.rs` (T-IMPL-04b) rows; `prism-dtu-armis/tests/` row updated to include RG-008; §Token Budget reassessed ~11,500→~13,300 (AC-008, T-IMPL-04, generator.rs/state.rs reads). POL-29 9a: twin S-WAVE-A-ARMIS-SPEC-001 updated in same burst; both pin BC-2.02.006 at v1.13 and BC-2.02.014 at v1.4. 9b: T-IMPL-01 TOML block is copy-target from BC-2.02.014 v1.4 §TOML Contract (itself from ADR-057 v0.7 §D5); transcribed verbatim per copy-source authority; 9b CLEAR. 9c: all new MUSTs in AC-008 and T-IMPL-04 carry AC+RGT anchors; no unanchored MUST introduced. |
| 1.5 | 2026-07-28 | story-writer | FB87 leg 2 — POL-23 stale-pin sweep: BC-2.02.006 v1.10 → v1.12 and BC-2.02.014 v1.2 → v1.3 (downstream of FB87 leg 1 product-owner bumps). Updated §Behavioral Contracts table rows for both BCs. Relevance cells verified accurate at new versions: BC-2.02.006 EC-02-014 description unaffected by v1.12 anchor-framing change; BC-2.02.014 cell describes push-down grammar/schema/error/edge-cases only (no emission mechanism text to correct). T-IMPL-01 §TOML Contract block independently verified as POL-29 9b downstream copy target: `serde_json::to_value` absent from story body; emission described as 'via serde_json (Json wrapper)' — consistent with (StatusCode::OK, Json(body)).into_response() corrected in BC-2.02.014 v1.3; PASS. No `ADR-057 §C1/§C2` references found in story body. POL-29 9a: S-WAVE-A-ARMIS-SPEC-001 twin updated in same burst; both stories now pin BC-2.02.006 at v1.12 (pre-burst asymmetry: twin was v1.11, this story was v1.10). 9b: PASS (see above). 9c: no new MUSTs authored. |
| 1.4 | 2026-07-28 | story-writer | FB86 — close F-WASE-P68-MED-004: delete banned authored-time RED_RATIO sentence (`RED_RATIO = 7/7 = 1.0 (meets the BC-5.38.001 threshold)`) from §Red Gate density check; normalize trailing deferral note to canonical exemplar wording (consistent with S-WAVE-A-ARMIS-SPEC-001 and the six FB61-cleaned stories). No content changes beyond §Red Gate density check paragraph. POL-29 9a: S-WAVE-A-ARMIS-SPEC-001 twin in same burst — same banned sentence deleted; symmetric. 9b: no downstream copy target. 9c: no new MUSTs introduced. |
| 1.3 | 2026-07-28 | story-writer | FB81 — F-WASE-P68-HIGH-003 Site 5. AC-002: removed false ADR-033 T1 citation ("via the ADR-033 T1 push-down extraction path") and false causal claim ("Without this option, the engine cannot extract the filter value for interpolation"). Replaced with accurate description aligned to ADR-057 §D5 v0.6 and BC-2.02.014 §Preconditions v1.2: `options = ["INDEX"]` declares push-down eligibility per BC-2.11.007 taxonomy (REQUIRED / INDEX / ADDITIONAL) for future T2 (`classify_predicates §classify_predicates`) integration; current routing is annotation-agnostic via `predicate_tree_to_filter_map §predicate_tree_to_filter_map` → `FetchContext.query_filters` → `execute_impl §execute_impl` pre-seed (ADR-057 §D4); ADR-033 T1 governs datetime time-window extraction only (`extract_time_window_from_ast §extract_time_window_from_ast`). RG-002: corrected "confirms push-down eligibility per ADR-033 T1" to "per BC-2.11.007 taxonomy and ADR-057 §D4". §Behavioral Contracts: BC-2.02.006 pin bumped v1.9 → v1.10, BC-2.02.014 pin bumped v1.0 → v1.2 (POL-23 stale-pin sweep including §Token Budget Estimate — no version pins present there). POL-29: 9a — S-WAVE-A-ARMIS-SPEC-001 does not carry the ADR-033 T1 claim or the false causal claim (story covers `devices` table only, cites ADR-023/ADR-028; no parameterized path routing content; absence verified by full story read); 9b — no downstream copy of this story's AC-002 text in other artifacts (story text is a consumer of the BC, not a copy-source for further artifacts); 9c — no new unanchored MUSTs introduced. |
| 1.2 | 2026-07-27 | story-writer | FB75: Complete story to `status: ready`. (1) Add BC-2.02.014 to `behavioral_contracts: [BC-2.02.006, BC-2.02.014]`; remove pending-BC status comment. (2) Replace placeholder ACs with real BC-traced criteria: AC-001..AC-007 derived from BC-2.02.014 postconditions, §Error Cases, §Edge Cases, invariant filter-required, and 2026-07-13 wire-shape discipline; AC-001 also traces to BC-2.02.006 EC-02-014 (deferral closure). (3) Code-reading verdict on `seed_missing_query_filter_vars §seed_missing_query_filter_vars`: confirmed empty-string pre-seed behavior — absent `device_id` filter produces path `/api/v1/devices//activity` → DTU HTTP 200 + empty array → silent empty result (violates BC-2.02.014 §Error Cases; spec wins per CLAUDE.md). ADR-057 §D4 analysis did not account for this function. (4) SAP-2 parity: `get_device_activity` handler has one static-fixture path only; all five `ActivityRecord` fields emitted via `Json(body)`; parity PASS. (5) SAC-1 compliance: enumerated RG-001..RG-007 with named test functions, BC-5.38.001 density check (7/7 = 1.0), red-then-green task ordering (all RG tests appear before all implementation tasks). (6) Add T-IMPL-02 for required-filter hard error mechanism — non-trivial engine fix: `seed_missing_query_filter_vars` must not pre-seed required filter slots with empty string. (7) Add §Code-Reading Verdict, §Forbidden Dependencies. (8) Bump points 3 → 5 (engine fix scope added). (9) Add SS-07 to subsystems; add `crates_touched` frontmatter. (10) POL-29 9a sweep: S-WAVE-A-ARMIS-SPEC-001 covers `devices` table; no content asymmetry in how either story describes `armis.sensor.toml`. (11) POL-29 9c: all MUSTs in §Acceptance Criteria and §Architecture Compliance Rules carry AC + RGT anchors. |
| 1.1 | 2026-07-27 | story-writer | FB72 story-writer leg 2: (1) Unblock story — architect confirmed `${query.filter.device_id}` filter-push-down grammar via ADR-057 adjudication (2026-07-27); rewrite §Blocking Dependency as §Architect Adjudication with confirmed path_template, single-device filter-required scope, per-record fan-out gap documented in ADR-057 §D6. (2) Set `behavioral_contracts: [BC-2.02.006]`; update BC status comment. (3) Add `## Authority` citing ADR-057 for SAC-2 bidirectionality. (4) Replace superseded `${variable.*}` phantom grammar in placeholder ACs, §Architecture Compliance Rules, §Tasks, §Behavioral Contracts. |
| 1.0 | 2026-07-27 | story-writer | Initial authoring (FB69). Created to satisfy Canonical Principle Rule 3: F-SAP2-MED-006 deferred gap `armis_device_activity` surface required a real story anchor. Ground-truth DTU state confirmed from code. Blocked on architect confirmation of variable injection grammar. Deferral cross-referenced to BC-2.02.006 EC-02-014 (FB68d). |
