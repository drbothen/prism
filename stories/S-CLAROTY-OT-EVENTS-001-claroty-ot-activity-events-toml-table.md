---
document_type: story
story_id: S-CLAROTY-OT-EVENTS-001
title: "Claroty xDome OT activity events table — TOML [[tables]] block, 21-column Tier-1/Tier-2 spec, live structural tests, no DTU (Wave A G2)"
level: "L4"
wave: xdome-wave-a
epic_id: E-XDOME-EXPANSION
priority: P0
status: ready
# BC status: BC-2.16.016 — MED-1 bare table_name + MED-3 build_column_array mechanism re-anchor applied 2026-08-31.
producer: story-writer
timestamp: "2026-08-24T00:00:00Z"
version: "1.6"
modified: "2026-08-31"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.016-claroty-ot-activity-events-table.md"
  - ".factory/objectives/xdome-endpoint-expansion-plan.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "5d0da54"
# input-hash: run `compute-input-hash <this-file> --update` after writing
traces_to: "BC-2.16.016"
points: 5
estimated_days: 1
tdd_mode: strict
subsystems: [SS-01, SS-16, SS-22]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters) owns this story's scope because
#     `crates/prism-sensors/specs/claroty.sensor.toml` — the TOML spec file being
#     modified — lives in the prism-sensors crate, which is listed under SS-01 per
#     ARCH-INDEX. The `claroty_ot_activity_events` [[tables]] block is a sensor-adapter
#     configuration artifact, exactly the surface SS-01 governs.
#   SS-16 (Spec Engine) owns this story's scope because
#     `crates/prism-spec-engine/src/spec_parser.rs` must parse the new [[tables]]
#     block without validation error. RG-001/RG-002 are spec-parser unit tests that
#     exercise SS-16's ColumnSpec and FetchStep deserialization. SS-16 is the canonical
#     owner of prism-spec-engine per ARCH-INDEX Subsystem Registry.
#   SS-22 (Process Lifecycle) owns this story's scope because
#     `crates/prism-bin` — the process-lifecycle crate per ARCH-INDEX — hosts the
#     authoritative E-QUERY-038 end-to-end gate (RG-003) and wire-shape serialization
#     assertion (RG-010), both of which exercise the spec_driven_adapter
#     (§pipeline_result_to_record_batch / `build_column_array`) that lives in prism-bin.
#     SS-22 is the canonical owner of prism-bin per ARCH-INDEX Subsystem Registry.
target_module: prism-sensors
crates_touched: [prism-sensors, prism-spec-engine, prism-bin]
# crates_touched:
#   prism-sensors: claroty.sensor.toml — new [[tables]] block for claroty_ot_activity_events
#   prism-spec-engine: RG-001/RG-002 spec-parser unit tests; no production code changes
#   prism-bin: authoritative RG-003 end-to-end E-QUERY-038 gate + wire-shape assertions
#              (bc_2_16_016_claroty_ot_activity_events_wire_shape.rs); AC-005/EC-009 raw
#              detection_time E-QUERY-038 test; RG-010 EC-002-WIRE related_alert_ids
#              JSON-array wire assertion; arrow-json dev-dep + [[test]] entry in Cargo.toml
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.016
  # BC-2.16.016 — Claroty xDome OT Activity Events Table: TOML table contract
  # (§Postconditions §1), 21-column Tier-1/Tier-2 classification (§Postconditions §2,
  # 4 Tier-1 + 17 Tier-2), Option B OCSF class rationale (§Postconditions §3),
  # SAP-2 DTU parity N/A (§Postconditions §4), EC-016-016-001..006 edge cases.
  # All 9 ACs trace to this BC.
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios during remove-uncertainty
# pass (same touchpoint as remove-uncertainty); scenarios live under the holdout-scenarios
# directory that test-writer and implementer MUST NOT read (contamination control).
# The story-level holdout gate (human-approved 2026-07-13) is BLOCKING before demo/push.
depends_on: []
# depends_on justification: S-ADR058-OCSF-ROUTING-001 (which established
# ocsf_column_naming=true for claroty.sensor.toml) is already MERGED (PR #242,
# develop@3f1e66179). No delivery-time scheduling dependency remains.
blocks: []
acceptance_criteria_count: 9
risk: MEDIUM
# Risk justification:
#   No DTU exists for this endpoint. All behavioral validation relies entirely on live
#   monroe tests. If the live sensor returns unexpected shapes (e.g., `event_id` as String
#   instead of Integer, or network 5-tuple fields under different key names), the TOML
#   column definitions must be revised. Validate against monroe before story-level holdout
#   gate. See §Notes for Implementer for per-field type validation notes.
assumption_validations: []
risk_mitigations: []
---

# S-CLAROTY-OT-EVENTS-001: Claroty xDome OT Activity Events Table — TOML Block + Live Tests (No DTU)

## Authority

**BC-2.16.016 §Postconditions §1 — TOML Table Contract** governs the exact `[[tables]]`
block structure: `table_name = "ot_activity_events"` (bare name; `{sensor_id}_{table_name}` derives the registered/queryable name `claroty_ot_activity_events`), `ocsf_class = "detection_finding"`,
step name `"fetch_ot_activity_events"`, `path_template = "/api/v1/ot_activity_events/"`,
`response_path = "$.ot_activity_events"`, pagination `type = "offset_limit"` / `page_size = 1000`,
and the 21-field `body_template`. Read §Postconditions §1 in full before authoring the TOML.

**BC-2.16.016 §Postconditions §2 — Column Tier Classification** governs Arrow field naming:
- Tier-1: `event_id` (`ocsf_field = "finding_info.uid"` → `finding_info_uid`, Integer, REQUIRED),
  `detection_time` (`ocsf_field = "time"` → `time`, Datetime),
  `event_type` (`ocsf_field = "activity_name"` → `activity_name`, String),
  `description` (`ocsf_field = "message"` → `message`, String).
- Tier-2 (17 columns): all 17 aggregate into `raw_extensions` under `ocsf_column_naming = true`.
  Network 5-tuple (`source_ip`, `dest_ip`, `protocol`, `dest_port`, `source_port`, `ip_protocol`)
  is Tier-2 by DELIBERATE design — see §Postconditions §3 Option B rationale.

**BC-2.16.016 §Postconditions §3 — Option B OCSF Class Rationale**: `detection_finding`
(class_uid 2004) is used — NOT `network_activity` (class_uid 4001). Authority: spike-findings
§Spike 2 §Decision. The governing constraint "NO new OCSF class_selector arms required
(pragmatic mappings)" in xdome-endpoint-expansion-plan.md §Current Coverage is a design
constraint, not a suggestion. The existing `detection_finding` arm covers this table.
`related_alert_ids` signals that these events are part of the detection workflow, confirming
Option B as the semantically correct choice.

**BC-2.16.016 §Postconditions §4 — SAP-2 N/A**: No DTU exists for
`claroty_ot_activity_events`. SAP-2 DTU-parity probe is explicitly not applicable for this
delivery. Near-term tests run against the live monroe sensor only. DTU creation is deferred
per D-2200. Once the deferred DTU story executes, BC-2.16.016 MUST be amended with DTU
route/types references and SAP-2 exclusion documentation.

**ADR-058 §B2** — Tier-2 columns aggregate into `raw_extensions`. ADR-058 §C — Arrow field
names from `ocsf_field_to_arrow_name`: `"finding_info.uid"` → `"finding_info_uid"`,
`"time"` → `"time"`, `"activity_name"` → `"activity_name"`, `"message"` → `"message"`.

**ADR-028 §D8-B** — `detection_time` column (Datetime type) omits `timestamp_formats`; the
implicit iso8601 default applies. This is intentional.

**spike-findings §Spike 2** is the authority for the OCSF class decision (Option B), the
21-column set (23 OTActivityEvent fields minus 2 excluded: `dest_network` and `source_network`),
and the Tier-1/Tier-2 classification.

**S-ADR058-OCSF-ROUTING-001** (merged PR #242) activated `ocsf_column_naming = true` at
the sensor level. The `detection_finding` / class_uid 2004 arm already exists in
`class_selector.rs::select_by_class_name` — used by the `alerts` and `device_alert_relations`
tables. No new class_selector arm is required (governing constraint verified).

---

## Narrative

As a SOC analyst querying Claroty xDome OT activity data via PrismQL,
I want a `claroty_ot_activity_events` table with OCSF `detection_finding` class,
so that I can query OT protocol activity events — configuration uploads, mode changes,
and monitored OT operations — with Tier-1 fields (`finding_info_uid`, `time`,
`activity_name`, `message`) directly queryable and Tier-2 network 5-tuple fields
accessible via `raw_extensions`.

## Background

The xDome sensor currently exposes 4 or 5 tables depending on whether S-CLAROTY-VULNS-001 (G1)
has merged: `alerts`, `audit_logs`, `devices`, `device_alert_relations` (always present), plus
optionally `claroty_vulnerabilities`. The `POST /api/v1/ot_activity_events/` endpoint (Gap G2)
is unaddressed.

This story delivers the complete Wave A G2 addition:
1. **`claroty.sensor.toml`** — add `[[tables]]` block for `claroty_ot_activity_events` (21 columns,
   offset_limit pagination, response_path `$.ot_activity_events`).
2. **Tests** — TOML parse unit tests + live structural Variant-1 tests against monroe (wire-level
   JSON assertions). SAP-2 is explicitly N/A for this story.

**OCSF Class Decision (Option B, see §Authority):** `detection_finding` / class_uid 2004 is used
because: (a) the governing plan constraint prohibits new class_selector arms; (b) `related_alert_ids`
links events to the Claroty detection workflow; (c) network 5-tuple fields are Tier-2 under
`ocsf_column_naming = true` — no data is lost in raw_extensions. Option A (`network_activity` /
4001) was rejected as scope not justified.

**SAP-2 Status — N/A for this story:**

No DTU exists for `claroty_ot_activity_events`. SAP-2 DTU-parity probe does not apply
to this delivery. Near-term tests are LIVE against monroe only. State explicitly in test
files: `// SAP-2 N/A: no DTU exists for claroty_ot_activity_events; deferred per D-2200`.

**Live-test approach (per xdome-endpoint-expansion-plan.md §Per-Story Pipeline):**

- **Variant-1 (structural, required):** Live `#[ignore]`'d integration tests against the
  monroe sensor. Wire-level JSON assertions on the serialized response (class_uid=2004,
  field presence, raw_extensions). Tests carry comment:
  `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually`.
- **Variant-2 (agent, optional):** Deferred to live-validation milestone.
- **Type validation note:** `event_id` is declared Integer in BC-2.16.016 §2 per spike-findings
  §Spike 2 (OTActivityEvent.event_id typed as integer in fields_enum). Verify on monroe live
  before story-level holdout gate. If the API actually returns event_id as a string,
  change `column_type` to `"string"` and update this story via an in-scope correction.

**Story-level holdout gate:** After LOCAL 3-CLEAN adversary convergence and BEFORE
demo recording / push to origin, the holdout-evaluator runs 2–4 hidden SINGLE-USE
scenarios (authored by PO at remove-uncertainty time; stored under the holdout directory;
contamination-controlled — test-writer and implementer MUST NOT read them). The gate is
BLOCKING: unsatisfied scenarios reset the LOCAL streak per BC-5.39.001.

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.016 | Claroty xDome OT Activity Events Table — Queryable Surface and OCSF detection_finding Mapping (No DTU) | v1.4 | §Postconditions §1 TOML table contract (bare table_name "ot_activity_events"; step, path, body_template, pagination, response_path); §Postconditions §2 21-column Tier-1/Tier-2 (4 Tier-1, 17 Tier-2); §Postconditions §3 Option B OCSF class rationale; §Postconditions §4 SAP-2 N/A; §Invariants (REQUIRED push-down semantics; absent-field passthrough via build_column_array within pipeline_result_to_record_batch); EC-016-016-001..006 edge cases |

## Acceptance Criteria

### AC-001: TOML block parses without validation error; 21 columns declared; 4 Tier-1, 17 Tier-2; pagination offset_limit 1000 (traces to BC-2.16.016 postcondition 1 — TOML Table Contract)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "ot_activity_events"` (bare name; `{sensor_id}_{table_name}` derives the registered/queryable name `claroty_ot_activity_events`), `ocsf_class = "detection_finding"`,
a step named `"fetch_ot_activity_events"` with `method = "POST"`,
`path_template = "/api/v1/ot_activity_events/"`,
`response_path = "$.ot_activity_events"`, pagination `type = "offset_limit"` / `page_size = 1000`,
and a `body_template` containing all 21 fields.

`SpecLoader::parse` on the modified TOML returns `Ok(SensorSpec)`. The parsed spec reports
21 `ColumnSpec` entries for `claroty_ot_activity_events`.

**Test:** `test_BC_2_16_016_claroty_ot_activity_events_toml_block_parses`

### AC-002: Exactly 4 Tier-1 columns declared with correct ocsf_field; Arrow names match ADR-058 §C (traces to BC-2.16.016 postcondition 2 — Tier-1 column classification)

The 4 Tier-1 `[[tables.columns]]` blocks declare:
- `event_id`: `column_type = "integer"`, `ocsf_field = "finding_info.uid"`, `options = ["REQUIRED"]`
- `detection_time`: `column_type = "datetime"`, `ocsf_field = "time"`
- `event_type`: `column_type = "string"`, `ocsf_field = "activity_name"`
- `description`: `column_type = "string"`, `ocsf_field = "message"`

Under `ocsf_column_naming = true`, the Arrow field names are `finding_info_uid`, `time`,
`activity_name`, `message` respectively (per `ocsf_field_to_arrow_name`). Exactly 4 of 21
columns have `ocsf_field == Some(_)`; exactly 17 have `ocsf_field == None`.

**Test:** `test_BC_2_16_016_claroty_ot_activity_events_four_tier1_columns`

### AC-003: Tier-2 network 5-tuple query raises E-QUERY-038; `available_columns` contains `raw_extensions` not `source_ip` (traces to BC-2.16.016 invariant — network 5-tuple fields are Tier-2; error case E-QUERY-038)

`SELECT source_ip FROM claroty.claroty_ot_activity_events LIMIT 1` raises E-QUERY-038 at
plan time. The error's `available_columns` MUST contain `raw_extensions`, `finding_info_uid`,
`time`, `activity_name`, `message`, `class_uid`, `_sensor` and MUST NOT contain `source_ip`,
`dest_ip`, `protocol`, `dest_port`, `source_port`, or `ip_protocol` as standalone column names.

Same applies for all 17 Tier-2 columns (network 5-tuple + others like `mode`, `related_alert_ids`).

**Test:** `test_BC_2_16_016_claroty_ot_activity_events_tier2_source_ip_raises_e_query_038`

### AC-004: Live Variant-1 wire-shape: `SELECT * LIMIT 1` serialized JSON contains class_uid=2004, finding_info_uid, raw_extensions with network 5-tuple (traces to BC-2.16.016 postcondition 1 class_uid; postcondition 2 Tier-1/Tier-2 wire representation)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_ot_activity_events LIMIT 1`
serialized JSON response (the MCP-visible wire shape per 2026-07-13 wire-shape discipline):
1. `class_uid` key is present with value `2004`
2. `finding_info_uid` key is present (integer or null; REQUIRED → null row if absent)
3. `time` key is present (ISO 8601 datetime string or null)
4. `activity_name` key is present (event type string or null)
5. `message` key is present (description string or null)
6. `raw_extensions` key is present as a JSON object (not null, not absent)
7. `raw_extensions` JSON object contains at least one of: `source_ip`, `dest_ip`, `protocol`

None of `source_ip`, `dest_ip`, `protocol` appear as standalone top-level keys.

**Test:** `test_BC_2_16_016_claroty_ot_activity_events_live_wire_shape_class_uid_and_tier1`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var pointing to monroe)

### AC-005: `SELECT time LIMIT 1` succeeds; `time` is the Arrow field name for Tier-1 detection_time (traces to BC-2.16.016 postcondition 2 — detection_time→time Tier-1 mapping)

Against the live monroe sensor, `SELECT time FROM claroty.claroty_ot_activity_events LIMIT 1`
succeeds (no E-QUERY-038). `time` is the OCSF Arrow field name for the `detection_time` Tier-1
column (`ocsf_field_to_arrow_name("time")` = `"time"`). `SELECT detection_time FROM ...` MUST
raise E-QUERY-038 (raw TOML column name, not an Arrow column name under `ocsf_column_naming = true`).

**Test:** `test_BC_2_16_016_claroty_ot_activity_events_live_time_column_succeeds`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL`)

### AC-006: `SELECT raw_extensions LIMIT 5` succeeds; JSON object contains network 5-tuple keys (traces to BC-2.16.016 postcondition 2 — Tier-2 network 5-tuple columns in raw_extensions)

Against the live monroe sensor, `SELECT raw_extensions FROM claroty.claroty_ot_activity_events LIMIT 5`
returns rows where `raw_extensions` is a non-null JSON object. The deserialized object contains
at minimum `source_ip` and `dest_ip` keys (values may be null if network info absent for some events).
No E-QUERY-038 is raised on `raw_extensions` itself.

Additionally, when `related_alert_ids` data is present in any fetched row, the `raw_extensions` JSON
object MUST contain `related_alert_ids` as a native JSON array (e.g., `[1, 2, 3]` or `[]`) — NOT
as a stringified JSON string (e.g., `"[1,2,3]"`) — matching EC-002 (EC-016-016-002). Uses
`column_type = "json"`; native JSON array preserved in `raw_extensions`, per the
`vulnerabilities.cve_ids` precedent (S-CLAROTY-VULNS-001); validate array pass-through at the
wire level explicitly.

**Test:** `test_BC_2_16_016_claroty_ot_activity_events_live_raw_extensions_contains_network_fields`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL`)

### AC-007: Missing `event_id` field → null `finding_info_uid` cell; `time` and `raw_extensions` remain populated; row NOT dropped; subsequent rows unaffected (traces to BC-2.16.016 §Invariants — event_id REQUIRED push-down flag; absent-field passthrough via build_column_array within pipeline_result_to_record_batch; edge case EC-016-016-001)

The `event_id` column carries `options = ["REQUIRED"]` in the TOML, which marks it as a mandatory
push-down parameter per BC-2.11.007 (`pushdown.rs::classify_predicates` REQUIRED priority ordering)
— REQUIRED does NOT control null-row behavior. When the API response contains a row where `event_id`
is absent or null, `build_column_array` (within `pipeline_result_to_record_batch`, reached via
`SpecDrivenSensorAdapter::fetch` in `crates/prism-bin/src/spec_driven_adapter.rs`) produces a null
`finding_info_uid` cell via absent-field passthrough; the row is NOT dropped — `time` and
`raw_extensions` remain populated and subsequent rows continue to materialize normally. No hard
error is raised. (`ColumnMapper::map_record` is a non-production reference mirror — it has zero
production callers and is NOT the production path.)

**Test:** `test_BC_2_16_016_claroty_ot_activity_events_required_event_id_absent_produces_null_row`
(unit test with mock response containing a row missing `event_id`)

### AC-008: `detection_time` null passthrough; ISO 8601 implicit default per ADR-028 §D8-B (traces to BC-2.16.016 invariant — detection_time ISO 8601 implicit default; edge case EC-016-016-003)

The `detection_time` column omits `timestamp_formats`; `effective_formats` returns `["iso8601"]`
as the implicit default (ADR-028 §D8-B). When `detection_time` is null/absent in the API response,
the cell value is null for that row — no E-SPEC-018, no error, pagination continues. When a
non-ISO-8601 string appears, E-SPEC-018 is raised for that row (null-demoted with warning);
pagination continues.

**Test:** `test_BC_2_16_016_claroty_ot_activity_events_detection_time_null_passthrough`
(unit test with mock response containing a row with `"detection_time": null`)

### AC-009: SAP-2 probe is N/A; no DTU exists; live-only validation documented (traces to BC-2.16.016 postcondition 4 — SAP-2 DTU parity status)

The story test file contains a visible comment: `// SAP-2 N/A: no DTU exists for
claroty_ot_activity_events; near-term validation is live against monroe only; DTU creation
deferred per D-2200`. The adversarial review MUST NOT file SAP-2 parity findings against
this story. Once the deferred DTU story (D-2200) executes, BC-2.16.016 §Postconditions §4
MUST be amended with DTU route/types references.

**Test:** `test_BC_2_16_016_claroty_ot_activity_events_sap2_na_documented`
(trivial marker test asserting a constant `SAP2_STATUS: &str = "N/A: no DTU; deferred D-2200"`;
ensures the constant is present in test file for adversarial review verification)

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_016_claroty_ot_activity_events_toml_block_parses` | Unit (SpecLoader::parse) | AC-001: TOML block parses Ok; 21 column entries returned for claroty_ot_activity_events; `related_alert_ids` ColumnType::Json confirmed (uses json column_type, per vulnerabilities.cve_ids precedent; not stringified) |
| RG-002 | `test_BC_2_16_016_claroty_ot_activity_events_four_tier1_columns` | Unit (ColumnSpec inspection) | AC-002: exactly 4 Tier-1 columns (ocsf_field == Some); event_id REQUIRED; detection_time→time; event_type→activity_name; description→message |
| RG-003 | `test_BC_2_16_016_claroty_ot_activity_events_tier2_source_ip_raises_e_query_038` | Integration end-to-end (prism-bin, via QueryEngine::execute — authoritative; prism-sensors version is defense-in-depth per SAP-3 rule 3) | AC-003: SELECT source_ip raises E-QUERY-038 end-to-end; available_columns excludes source_ip; includes raw_extensions, finding_info_uid, time, activity_name, message |
| RG-004 | `test_BC_2_16_016_claroty_ot_activity_events_live_wire_shape_class_uid_and_tier1` | Live Variant-1 (`#[ignore]`) | AC-004: wire JSON contains class_uid=2004; finding_info_uid, time, activity_name, message present; raw_extensions present with network fields; no Tier-2 standalone keys |
| RG-005 | `test_BC_2_16_016_claroty_ot_activity_events_live_raw_extensions_contains_network_fields` | Live Variant-1 (`#[ignore]`) | AC-006: raw_extensions JSON contains source_ip, dest_ip keys; no E-QUERY-038 on raw_extensions |
| RG-006 | `test_BC_2_16_016_claroty_ot_activity_events_required_event_id_absent_produces_null_row` | Unit (mock response) | AC-007: row missing event_id → null row; no hard error; subsequent rows continue |
| RG-007 | `test_BC_2_16_016_claroty_ot_activity_events_detection_time_null_passthrough` | Unit (mock response) | AC-008: detection_time null → null cell; no E-SPEC-018; no pagination halt |
| RG-008 | `test_BC_2_16_016_claroty_ot_activity_events_sap2_na_documented` | Marker (constant assertion) | AC-009: SAP-2 N/A constant documented in test file; adversarial reviewer can verify |
| RG-009 | `test_BC_2_16_016_claroty_ot_activity_events_raw_detection_time_raises_e_query_038` | Integration end-to-end (prism-bin, via QueryEngine::execute) | EC-009 / AC-005 MUST clause: SELECT detection_time by raw TOML name raises E-QUERY-038 (not an Arrow column under ocsf_column_naming=true; use `time` instead) |
| RG-010 | `test_BC_2_16_016_claroty_ot_activity_events_wire_related_alert_ids_json_array` | Unit/integration (prism-bin, wire-shape serialization assertion) | EC-002-WIRE / AC-006: related_alert_ids serialized as native JSON array (e.g., `[1,2,3]` or `[]`), NOT as a stringified string — uses column_type="json" (per vulnerabilities.cve_ids precedent); wire-level assertion mandatory |

**BC-5.38.001 density check:** 10 Red Gate tests / 9 acceptance criteria = 1.11 ≥ 0.5 threshold.
PASS. RG-001..RG-008 each gate a primary AC (AC-005 covered within RG-004's `time` key assertion).
RG-009 and RG-010 provide supplementary prism-bin wire-shape coverage: EC-009 raw-name rejection
and EC-002-WIRE JSON-array passthrough. The authoritative RG-003 (prism-bin end-to-end) and
defense-in-depth RG-003 (prism-sensors plan-time) share a test name; counted once in density.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `claroty_ot_activity_events` TOML block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| TOML parse validation | `crates/prism-spec-engine/src/spec_parser.rs §spec_parser` | Pure (TOML deserialization; no I/O) |
| Tier-1/Tier-2 Arrow schema computation | `crates/prism-spec-engine/src/column_mapping.rs §ocsf_field_to_arrow_name` | Pure (string transformation; no I/O) |
| OffsetLimit POST-body injection | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute` | Effectful (HTTP POST to xDome; merges offset/limit into body_template) |
| response_path extraction | `crates/prism-bin/src/spec_driven_adapter.rs §pipeline_result_to_record_batch` | Effectful (processes HTTP response; builds Arrow RecordBatch) |
| `detection_finding` class arm (existing) | `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` | Pure (constant → u32 lookup; arm already exists; returns 2004) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (prism-sensors; claroty.sensor.toml)
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; spec_parser, pipeline, column_mapping)
- ADR-058 §B2 (Tier-2 raw_extensions aggregation), §C (Arrow field naming underscore-flatten), §D (ocsf_column_naming per-sensor flag)
- ADR-028 §D8-B (implicit iso8601 default for datetime without timestamp_formats)
- spike-findings §Spike 2 (OCSF class decision; 21-column set; 2 excluded fields)

## Purity Classification

- **Pure (no I/O, deterministic):** `SpecLoader::parse` (TOML deserialization); `ocsf_field_to_arrow_name`
  (string → string, deterministic); `select_by_class_name("detection_finding")` (constant lookup,
  returns 2004); RG-001/RG-002 TOML parse + column inspection assertions; RG-008 marker constant.
- **Effectful (I/O, network):** `PipelineExecutor::execute` (HTTP POST to `/api/v1/ot_activity_events/`;
  pagination loop); `pipeline_result_to_record_batch` (HTTP response to Arrow RecordBatch);
  RG-004/RG-005 live Variant-1 integration tests (require running monroe sensor).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Row missing `event_id` field in API response | `finding_info_uid` cell is null; `time` and `raw_extensions` remain populated; row is NOT dropped; no hard error; subsequent rows continue. Attribution: `build_column_array` absent-field passthrough within `pipeline_result_to_record_batch` (`SpecDrivenSensorAdapter::fetch` path) — independent of the REQUIRED push-down-parameter flag. (`ColumnMapper::map_record` is a non-production reference mirror.) (BC-2.16.016 §Invariants; EC-016-016-001) |
| EC-002 | `related_alert_ids` is empty array `[]` | Serialized as `[]` JSON in `raw_extensions`; not null (EC-016-016-002) |
| EC-003 | `detection_time` is null/absent | Null Datetime cell; ADR-028 §D8-B null-passthrough; no E-SPEC-018 (EC-016-016-003) |
| EC-004 | `mode` field absent (not all event types change mode) | Null string cell in `raw_extensions`; no error (EC-016-016-004) |
| EC-005 | Network 5-tuple fields partially absent (e.g., source_ip present, dest_ip absent) | Present fields serialized into raw_extensions; absent fields not serialized; no error (EC-016-016-005) |
| EC-006 | SELECT source_ip by raw Tier-2 name | E-QUERY-038; available_columns includes raw_extensions but NOT source_ip (EC-016-016-006) |
| EC-007 | `detection_time` non-ISO-8601 string | E-SPEC-018 TimestampParseFailure — null demoted with warning; row continues; no pagination halt |
| EC-008 | API returns non-200 HTTP for POST /api/v1/ot_activity_events/ | E-SENSOR-001 structured error; partial results returned for previously fetched pages |
| EC-009 | SELECT detection_time by raw TOML name | E-QUERY-038 — raw TOML name not an Arrow column under ocsf_column_naming=true; use `time` instead |
| EC-010 | `event_id` type mismatch: API returns string, TOML declares integer | E-SPEC-018 coercion failure or null-demote per ADR-058 §D coercion path; verify event_id type on monroe live |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~7,500 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (existing 4 tables as pattern reference) | ~5,500 |
| BC-2.16.016 (full) | ~5,000 |
| ADR-058 §B2/§C/§D sections (ocsf_column_naming flag mechanism) | ~4,000 |
| spike-findings §Spike 2 (OCSF class decision; column set; 2 excluded fields) | ~2,500 |
| prism-spec-engine/src/spec_parser.rs (ColumnSpec + FetchStep section) | ~3,000 |
| prism-spec-engine/src/column_mapping.rs (ocsf_field_to_arrow_name) | ~1,500 |
| Test files (8 RGTs; unit + live integration; SAP-2 N/A marker) | ~5,500 |
| ADR-028 §D8-B (implicit iso8601 default reference) | ~1,000 |
| **Total estimate** | **~35,500 tokens** |

Well within 20-30% of a 200K window. If context is tight, load `claroty.sensor.toml` sections
by reading only the `alerts` table block first as the canonical pattern, then the pagination section.

## Tasks

- [ ] **Task 1 (Red Gate — test first):** Write RG-001: `test_BC_2_16_016_claroty_ot_activity_events_toml_block_parses` in `crates/prism-spec-engine/src/spec_parser.rs #[cfg(test)] mod tests`. Call `SpecLoader::parse` on `claroty.sensor.toml` (or a test fixture containing the new block). Assert `Ok(SensorSpec)` returned, `claroty_ot_activity_events` table present, 21 `ColumnSpec` entries. MUST fail before Task 6 (block not yet in TOML).

- [ ] **Task 2 (Red Gate — test first):** Write RG-002: `test_BC_2_16_016_claroty_ot_activity_events_four_tier1_columns` in same test module. Parse TOML; find `claroty_ot_activity_events` table; assert exactly 4 columns have `ocsf_field == Some(_)`: `event_id` → `"finding_info.uid"` with `options = ["REQUIRED"]`; `detection_time` → `"time"`; `event_type` → `"activity_name"`; `description` → `"message"`. Assert 17 columns have `ocsf_field == None`. MUST fail before Task 6.

- [ ] **Task 3 (Red Gate — test first):** Write RG-006, RG-007, RG-008 — unit tests in `crates/prism-sensors/tests/bc_2_16_016_claroty_ot_activity_events.rs`. RG-006: mock response with row missing event_id → null row, no hard error. RG-007: mock response with `detection_time: null` → null cell, no error. RG-008: trivial marker test with `SAP2_STATUS = "N/A: no DTU; deferred D-2200"` constant + assertion. All MUST fail (or not exist) before Task 6. Add SAP-2 N/A comment at file header: `// SAP-2 N/A: no DTU exists for claroty_ot_activity_events; deferred per D-2200`.

- [ ] **Task 4 (Red Gate — test first):** Write RG-003: `test_BC_2_16_016_claroty_ot_activity_events_tier2_source_ip_raises_e_query_038`. Drive `SELECT source_ip FROM claroty.claroty_ot_activity_events LIMIT 1` through plan-time validation. Assert E-QUERY-038; `available_columns` includes `raw_extensions`, `finding_info_uid`, `time`, `activity_name`, `message`; excludes `source_ip`. MUST fail before Task 6.

- [ ] **Task 5 (Red Gate — test first):** Write RG-004 and RG-005 — live Variant-1 `#[ignore]`'d integration tests. Each carries comment `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe`. RG-004 asserts wire JSON: `class_uid == 2004`; `finding_info_uid` present; `raw_extensions` present as JSON object; no Tier-2 keys at top level. RG-005 asserts raw_extensions JSON contains `source_ip`, `dest_ip` keys. Both MUST fail when `#[ignore]` removed if TOML block absent.

- [ ] **Task 6 (Implementation — TOML block):** Add the `[[tables]]` block to `crates/prism-sensors/specs/claroty.sensor.toml`. Follow exactly BC-2.16.016 §Postconditions §1. Columns in order: 4 Tier-1 first (`event_id` with REQUIRED, `detection_time`, `event_type`, `description`), then 17 Tier-2 (network 5-tuple first: `source_ip`, `dest_ip`, `protocol`, `dest_port`, `source_port`, `ip_protocol`, then remaining 11). Step: `fetch_ot_activity_events`, POST, `/api/v1/ot_activity_events/`, body_template with all 21 fields, response_path `$.ot_activity_events`, pagination offset_limit 1000. Add comments: DTU deferred note, SAP-2 N/A note, Option B OCSF class rationale note, D-2200 deferred DTU anchor.

  After editing: run `just iter prism-spec-engine` — RG-001, RG-002 MUST turn GREEN.

- [ ] **Task 7 (Implementation — verify unit tests green):** Run `just iter prism-spec-engine --no-fail-fast`. Confirm RG-001, RG-002, RG-006, RG-007, RG-008 all GREEN. Run `just iter prism-sensors` — confirm TOML file is valid. No existing tests should regress.

- [ ] **Task 8 (Type validation — live check):** Before story-level holdout gate, run Variant-1 live test (RG-004) against monroe (with `CLAROTY_INSTANCE_URL` set). Verify `event_id` type: if the API returns `event_id` as a string instead of integer, update `column_type = "integer"` → `column_type = "string"` in the TOML and this story spec. Document the finding. Update RG-004 assertion accordingly.

- [ ] **Task 9 (SAP-1 self-check):** Confirm no new `tracing::*!(event_type = ...)` emissions added (TOML-only change + unit tests). If any new emission appears, add a BC-2.16.002 catalog row per PG-LP11-001.

- [ ] **Task 10 (Final gate):** Run `just check` (full workspace). Confirm non-`#[ignore]` Red Gate tests pass (RG-001, RG-002, RG-003, RG-006, RG-007, RG-008). Confirm `claroty.sensor.toml` contains the `claroty_ot_activity_events` table and all prior tables remain intact (count-agnostic: 5 or 6 tables depending on whether S-CLAROTY-VULNS-001 G1 has already merged). After `just check` passes, hold for story-level holdout gate before pushing to origin.

## Previous Story Intelligence

1. **S-ADR058-OCSF-ROUTING-001 (merged PR #242):** Activated `ocsf_column_naming = true` at the
   sensor level. The `detection_finding` / class_uid 2004 arm is confirmed existing in
   `class_selector.rs::select_by_class_name` — it is used by both the `alerts` and
   `device_alert_relations` tables. No new class_selector arm needed for this story.

2. **S-DEMO-CLAROTY-DAR-001 (merged):** Added `device_alert_relations` (also `detection_finding`
   class_uid 2004). This is the closest predecessor pattern: same OCSF class, same POST-for-read
   step pattern, same offset_limit pagination. Read its `[[tables]]` block as the secondary
   pattern (after `alerts`) before authoring `claroty_ot_activity_events`.

3. **S-CLAROTY-AUDITLOG-TIMEBOX-001 (merged):** Added time-filter push-down to `audit_logs`.
   No direct impact on OT events, but confirms the Claroty TOML authoring + test pattern in
   this project. Story format is the canonical template reference.

4. **S-ADR058-OCSF-COERCION-001 (merged PR #240):** Closed EC-016-013-007/008/009 coercion
   fixes. The `claroty_ot_activity_events` table includes `detection_time` (Datetime) and
   `event_id` (Integer). Verify coercion path handles these types correctly via RG-007 and
   the live Variant-1 test.

5. **Governing plan constraint (xdome-endpoint-expansion-plan.md §Current Coverage):**
   "NO new OCSF class_selector arms required (pragmatic mappings)." This is enforced by the
   Option B decision. Do not introduce any new arm in `class_selector.rs` for this story.

## Architecture Compliance Rules

From ADR-058 §D (ocsf_column_naming flag):
- `ocsf_column_naming = true` is already declared at the sensor level in `claroty.sensor.toml`.
  New `[[tables]]` blocks inherit this automatically. No per-table flag needed.
- Per ADR-058 §B2: Tier-2 columns MUST aggregate into `raw_extensions`. The network 5-tuple
  (`source_ip`, `dest_ip`, `protocol`, `dest_port`, `source_port`, `ip_protocol`) is
  intentionally Tier-2 per BC-2.16.016 §Postconditions §3 Option B. Do not add `ocsf_field`
  to network 5-tuple columns to promote them to Tier-1 — that would violate the BC and contradict
  the Option B design decision.

From ADR-028 §D8-B:
- `detection_time` column type `datetime` with NO `timestamp_formats` key is valid — `effective_formats`
  returns `["iso8601"]`. Do NOT add `timestamp_formats = ["iso8601"]` unnecessarily.

From governing plan §Governing Directive:
- `class_selector.rs::select_by_class_name` MUST NOT gain a new arm for `"ot_activity_events"`,
  `"network_activity"`, or any other new class for this delivery. The existing `"detection_finding"`
  arm already returns 2004 and is sufficient.

From BC-2.16.016 §Postconditions §3:
- `ocsf_class = "detection_finding"` is the ONLY valid value for this table. Do not use
  `ocsf_class = "network_activity"` — Option A was rejected.

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `prism-spec-engine` | workspace path | `SpecLoader::parse`, `ColumnSpec`, `FetchStep`, `PaginationConfig::OffsetLimit` |
| `prism-ocsf` | workspace path | `class_selector.rs::select_by_class_name("detection_finding")` → 2004 (existing arm — read only; no modification) |
| `serde_json` | per workspace Cargo.toml | Mock response construction in unit tests (RG-006, RG-007) |
| `tokio` | per workspace Cargo.toml | Async test runtime for live integration tests (RG-004, RG-005) |

Do NOT add new Cargo.toml production dependencies.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-sensors/specs/claroty.sensor.toml` | Add `[[tables]]` block for `claroty_ot_activity_events` after existing `device_alert_relations` (or `claroty_vulnerabilities` if that block was added first) |
| CREATE | `crates/prism-sensors/tests/bc_2_16_016_claroty_ot_activity_events.rs` | RG-003 (defense-in-depth, plan-time; SAP-3 rule 3 comment required), RG-004, RG-005 (`#[ignore]` live Variant-1 with `LIVE-MONROE-001` comment), RG-006, RG-007, RG-008 unit/marker tests; SAP-2 N/A comment at file header |
| CREATE | `crates/prism-bin/tests/bc_2_16_016_claroty_ot_activity_events_wire_shape.rs` | Authoritative end-to-end tests: RG-003 (SELECT source_ip → E-QUERY-038 via QueryEngine::execute), wire-shape assertions (class_uid=2004, Tier-1 field presence, raw_extensions — AC-004/RG-004 wire companion), RG-009 (SELECT detection_time → E-QUERY-038 — EC-009), RG-010 EC-002-WIRE (related_alert_ids native JSON array); SAP-2 N/A comment at file header |
| MODIFY | `crates/prism-bin/Cargo.toml` | Add `arrow-json` dev-dependency (wire-shape serialization in end-to-end tests); add `[[test]]` entry for `bc_2_16_016_claroty_ot_activity_events_wire_shape` |

Files that MUST NOT be modified:
- `crates/prism-ocsf/src/class_selector.rs` — `detection_finding` arm already exists; no changes
- `crates/prism-spec-engine/src/spec_parser.rs` — no production code changes needed; RG-001/RG-002 may add unit tests in-module
- `crates/prism-dtu-claroty/` — SAP-2 N/A for this story; no reads or modifications required

## Forbidden Dependencies

`prism-sensors` MUST NOT gain any new production dependency on `prism-dtu-claroty`. There is no
DTU for `claroty_ot_activity_events` and no SAP-2 probe in this delivery. `prism-spec-engine`
MUST NOT gain a new dependency on `prism-sensors` (direction is prism-sensors → prism-spec-engine).
`class_selector.rs` MUST NOT gain new arms for this story.

## Notes for Implementer

1. **Option B is locked.** `ocsf_class = "detection_finding"` is non-negotiable. Do not change
   to `"network_activity"` — that would require a new class_selector arm, violate the governing
   plan constraint, and contradict BC-2.16.016 §Postconditions §3.

2. **Network 5-tuple stays Tier-2.** The 6 network 5-tuple columns (`source_ip`, `dest_ip`,
   `protocol`, `dest_port`, `source_port`, `ip_protocol`) have NO `ocsf_field`. They are
   intentionally Tier-2 — aggregate into `raw_extensions`. Do not add `ocsf_field` to them.

3. **SAP-2 probe is N/A.** Do NOT read `crates/prism-dtu-claroty/src/` for parity checks.
   Add `// SAP-2 N/A: no DTU exists for claroty_ot_activity_events; deferred per D-2200`
   at the top of the test file.

4. **`event_id` type: Integer per BC and spike-findings.** The OTActivityEvent fields_enum
   lists `event_id` with an integer-compatible type. If the live monroe sensor returns
   `event_id` as a string in the actual API response, update `column_type` to `"string"` as
   an in-scope correction and note the finding. Do not block on this — verify with Variant-1
   live test (RG-004) at Task 8.

5. **2 fields deliberately excluded.** `dest_network` and `source_network` (from the 23-field
   OTActivityEvent fields_enum) are NOT in the 21-column set — excluded per spike-findings
   §Spike 2. Do not add them. Do not treat their absence as a gap or adversarial finding.

6. **Live tests are `#[ignore]`'d.** RG-004 and RG-005 require the live monroe sensor. Mark
   `#[ignore]` per SID-1: also provide non-ignored unit tests (RG-001, RG-002, RG-006, RG-007,
   RG-008) as the non-live coverage for CI.

7. **Holdout gate is BLOCKING.** After LOCAL adversary 3-CLEAN and BEFORE push to origin, the
   holdout-evaluator runs hidden scenarios (PO-authored at remove-uncertainty pass). Do not push
   until the gate passes.

---

## References

- BC-2.16.016 — §Postconditions §1 TOML bare table_name "ot_activity_events" (registered/queryable name derived as {sensor_id}_{table_name}); §Postconditions §2 21-column Tier-1/Tier-2; §Postconditions §3 Option B rationale; §Postconditions §4 SAP-2 N/A; §Invariants (REQUIRED push-down semantics per BC-2.11.007; absent-field null passthrough via build_column_array within pipeline_result_to_record_batch, SpecDrivenSensorAdapter::fetch path); EC-016-016-001..006
- ADR-058 §B2 — Tier-2 columns aggregate into raw_extensions; §C — underscore-flattened Arrow names; §D — per-sensor ocsf_column_naming flag
- ADR-028 §D8-B — implicit iso8601 default for datetime columns without timestamp_formats
- spike-findings §Spike 2 — OCSF class decision (Option B over Option A); 21-column set; 2 excluded fields (dest_network, source_network); Tier-1/Tier-2 classification authority
- xdome-endpoint-expansion-plan.md §Near-Term Stories, §Per-Story Pipeline, §Governing Directive — Wave A G2 context; no-DTU live test approach; no-new-arm constraint
- `crates/prism-sensors/specs/claroty.sensor.toml §device_alert_relations` — closest predecessor pattern (same detection_finding class, same POST-for-read step); also `§alerts` as canonical pattern
- S-ADR058-OCSF-ROUTING-001 (merged PR #242) — activated ocsf_column_naming=true; detection_finding arm confirmed existing

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.6 | 2026-08-31 | story-writer | LOW records-accuracy fix: removed false "first production use of column_type=json" absolute claim from §AC-006 body, §Red Gate Tests RG-001 and RG-010 rows — S-CLAROTY-VULNS-001 vulnerabilities.cve_ids on develop already uses json column_type, so the absolute was false; replaced with "per vulnerabilities.cve_ids precedent" wording throughout. Proactive sweep: §Background "4 tables" absolute made merge-order-safe ("4 or 5 tables depending on whether S-CLAROTY-VULNS-001 G1 has merged"), matching Task 10 count-agnostic pattern. No AC coverage, BC-trace, or behavioral content changes. |
| 1.5 | 2026-08-31 | story-writer | SS-22 (Process Lifecycle) added to frontmatter `subsystems:` with justification comment — story touches prism-bin via authoritative E-QUERY-038 end-to-end test and wire-shape assertions (`bc_2_16_016_claroty_ot_activity_events_wire_shape.rs`); §Architecture Mapping cites `crates/prism-bin/src/spec_driven_adapter.rs §pipeline_result_to_record_batch`. SS-22 canonical subsystem ID confirmed against ARCH-INDEX Subsystem Registry. |
| 1.4 | 2026-08-31 | story-writer | F-OTE-MED-002 (POL-39 anti-volatile-pin + stale cross-artifact drift): removed volatile vX.Y version pins from §Authority prose (BC-2.16.016 §Postconditions §1–§4 headings), §References, §Token Budget table, and frontmatter prose comments; synced §Behavioral Contracts table Version column v1.3→v1.4 (POL-40 structural pin). No behavioral content changes. |
| 1.3 | 2026-08-31 | story-writer | MED-1 + MED-3 mechanism re-anchor: §Authority + AC-001 bare table_name corrected from `"claroty_ot_activity_events"` to `"ot_activity_events"` (derivation note added — `{sensor_id}_{table_name}` = registered/queryable name `claroty_ot_activity_events`). AC-007 title + body + EC-001 re-anchored from non-production `ColumnMapper::map_record` to production mechanism `build_column_array` within `pipeline_result_to_record_batch` (reached via `SpecDrivenSensorAdapter::fetch`, `crates/prism-bin/src/spec_driven_adapter.rs`); `map_record` retained as explicitly-labeled non-production reference mirror only. All BC-2.16.016 refs bumped v1.2→v1.3. |
| 1.2 | 2026-08-31 | story-writer | MED-1 (POL-4) REQUIRED-semantics fix: AC-007 §Acceptance Criteria and EC-001 §Edge Cases causal attribution corrected — absent-field null passthrough attributed to ColumnMapper::map_record default absent-field handling, independent of REQUIRED push-down-parameter flag per BC-2.11.007 §Invariants (BC-2.16.016 v1.2 §Invariants and EC-016-016-001 authoritative wording); AC-007 title updated to reflect row-NOT-dropped + time/raw_extensions-remain semantics. MED-2 prism-bin traceability: frontmatter crates_touched adds prism-bin; §Architecture Mapping spec_driven_adapter.rs corrected to crates/prism-bin/src/ (not prism-spec-engine); §Red Gate Tests RG-003 row updated to prism-bin authoritative end-to-end + prism-sensors defense-in-depth SAP-3 note; RG-009 (EC-009 raw detection_time E-QUERY-038) and RG-010 (EC-002-WIRE JSON-array wire assertion) added; §File Structure Requirements adds CREATE crates/prism-bin/tests/bc_2_16_016_claroty_ot_activity_events_wire_shape.rs and MODIFY crates/prism-bin/Cargo.toml; density check updated to 10/9. All BC-2.16.016 refs bumped v1.1→v1.2. |
| 1.1 | 2026-08-30 | story-writer | G2 propagation + promote: F-1 downstream-copy sweep — no body_template copy present in story (confirmed). Status promoted draft→ready (pre-TDD remove-uncertainty CLEAN 2026-08-30; F-1 fixed in BC v1.1). All BC refs updated v1.0→v1.1. Task 10 reworded count-agnostic (5 or 6 tables, G1 merge-order dependent). AC-006 + RG-001 explicit JSON-array assertion added for related_alert_ids (first json column_type use; must not stringify; matches EC-002). |
| 1.0 | 2026-08-24 | story-writer | Initial authoring — F3 story materialization for S-CLAROTY-OT-EVENTS-001 (Wave A G2). BC-2.16.016 v1.1 traceability; 21-column Tier-1/Tier-2 spec; 9 ACs; 8 RGTs; density 0.89; SAC-1 compliant; SAP-2 explicitly N/A (no DTU; D-2200 deferred); Option B OCSF class locked; live-test approach per xdome-endpoint-expansion-plan.md §Per-Story Pipeline; 2 deliberately excluded fields documented. |
