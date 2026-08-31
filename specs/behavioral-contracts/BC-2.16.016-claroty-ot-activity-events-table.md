---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-08-24T00:00:00Z
phase: 3
origin: brownfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: draft
inputs:
  - ".factory/objectives/xdome-endpoint-expansion-plan.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "4f7d01e"
traces_to: ["CAP-029"]
extracted_from: ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
introduced: "2026-08-24"
modified: 2026-08-31
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.16.016: Claroty xDome OT Activity Events Table — Queryable Surface and OCSF detection_finding Mapping (No DTU)

## Description

The `claroty_ot_activity_events` TOML table block in `claroty.sensor.toml` exposes Claroty xDome
OT (Operational Technology) activity events — monitored OT protocol operations such as
Configuration Upload/Download — as a queryable PrismQL table. The table follows the standard
Claroty POST-for-read pattern with offset/limit pagination. `detection_finding` (class_uid 2004)
is used as the OCSF class per Spike 2 Option B decision (Spike 2 §Decision): the events represent
Claroty's OT monitoring/detection workflow, as evidenced by `related_alert_ids` linking events
to Claroty alerts. Under `ocsf_column_naming = true`, four fields map to Tier-1 OCSF columns;
all 17 remaining fields — including the full network 5-tuple — are Tier-2 aggregated into
`raw_extensions`. No DTU exists for this endpoint; near-term tests run against the live
monroe sensor (see §Invariants).

## Preconditions

- `claroty.sensor.toml` includes the `claroty_ot_activity_events` [[tables]] block as specified
  in S-CLAROTY-OT-EVENTS-001
- `ocsf_column_naming = true` is declared at the sensor level in `claroty.sensor.toml`
- The `detection_finding` / class_uid 2004 arm exists in
  `prism-ocsf/src/class_selector.rs::select_by_class_name` (existing arm — same arm used
  by `alerts` and `device_alert_relations` tables; no new arm required)
- The Claroty bearer token credential is configured for the requesting client
- S-PLUGIN-PREREQ-A through S-PLUGIN-PREREQ-E have all merged (spec-engine pipeline active)

## Postconditions

### 1. TOML Table Contract

The `claroty_ot_activity_events` table MUST be declared in `claroty.sensor.toml` with:

```toml
[[tables]]
table_name = "ot_activity_events"  # bare name; TableRegistry derives the registered/queryable name as {sensor_id}_{table_name} = "claroty_ot_activity_events"
ocsf_class = "detection_finding"   # class_uid 2004 (existing arm; same as alerts table)
```

**Step definition:**

```toml
[[tables.steps]]
name = "fetch_ot_activity_events"
method = "POST"
path_template = "/api/v1/ot_activity_events/"
body_template = '{"fields": ["event_id", "detection_time", "event_type", "description", "source_ip", "dest_ip", "protocol", "dest_port", "source_port", "ip_protocol", "source_asset_id", "dest_asset_id", "source_device_name", "dest_device_name", "source_device_type", "dest_device_type", "source_site_name", "dest_site_name", "source_username", "related_alert_ids", "mode"]}'
response_path = "$.ot_activity_events"
variables_produced = []
[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

### 2. Column Tier Classification (ADR-058)

Under `ocsf_column_naming = true`, columns are classified as follows:

**Tier-1 columns** (have `ocsf_field`; exposed as Arrow field name =
`ocsf_field_to_arrow_name(ocsf_field)`):

| Column (TOML name) | ColumnType | ocsf_field | Arrow Field Name | Options |
|--------------------|-----------|------------|-----------------|---------|
| `event_id` | Integer | `finding_info.uid` | `finding_info_uid` | REQUIRED |
| `detection_time` | Datetime | `time` | `time` | — |
| `event_type` | String | `activity_name` | `activity_name` | — |
| `description` | String | `message` | `message` | — |

**Tier-2 columns** (no `ocsf_field`; values aggregate into `raw_extensions` JSON object):

| Column (TOML name) | ColumnType | Notes |
|--------------------|-----------|-------|
| `source_ip` | String | Network 5-tuple — source IP address |
| `dest_ip` | String | Network 5-tuple — destination IP address |
| `protocol` | String | OT protocol e.g. "CIP", "Modbus" |
| `dest_port` | Integer | Network 5-tuple — destination port |
| `source_port` | Integer | Network 5-tuple — source port |
| `ip_protocol` | String | IP protocol e.g. "TCP", "UDP" |
| `source_asset_id` | String | Claroty source asset ID |
| `dest_asset_id` | String | Claroty destination asset ID |
| `source_device_name` | String | |
| `dest_device_name` | String | |
| `source_device_type` | String | e.g. "Engineering Station" |
| `dest_device_type` | String | e.g. "PLC" |
| `source_site_name` | String | Site of source device |
| `dest_site_name` | String | Site of destination device |
| `source_username` | String | OT user who initiated event |
| `related_alert_ids` | Json | Array of related Claroty alert IDs; serialized as JSON |
| `mode` | String | Mode Change event target mode |

**Total declared columns:** 21 (4 Tier-1, 17 Tier-2).

### 3. OCSF Class Rationale — Option B decision_finding/2004

`detection_finding` (class_uid 2004) is used per spike findings §Spike 2 §Decision: Option B:

1. The governing plan constraint is "NO new OCSF `class_selector` arms required (pragmatic mappings)"
   (xdome-endpoint-expansion-plan.md §Current Coverage) — this is a design constraint, not a suggestion.
2. The events ARE detections: Claroty's OT visibility platform surfaces them as "monitored/detected
   OT activity." `related_alert_ids` is the definitive signal that these events are part of the
   alert/detection workflow.
3. Under `ocsf_column_naming = true`, all 6 network 5-tuple fields are Tier-2 — they aggregate
   into `raw_extensions` and remain queryable via `raw_extensions` key access. No data is lost.
4. `network_activity` (class_uid 4001) was evaluated as Option A and rejected: it would require
   a new `CLASS_UID_NETWORK_ACTIVITY: u32 = 4001` const and two new match arms in
   `class_selector.rs::select_by_class_name` and `select` — scope not justified by incremental
   semantic improvement.

**Anchor:** spike-findings §Spike 2 §Decision; xdome-endpoint-expansion-plan.md §Governing Directive.

### 4. SAP-2 DTU Parity Status

SAP-2 probe is **N/A** for G2 (no DTU exists for this endpoint). The deferred DTU creation story
is tracked as D-2200 (per xdome-endpoint-expansion-plan.md §Deferred DTU-Creation Stories).
Once the DTU story for `ot_activity_events` executes, SAP-2 probe applies retroactively and
this BC MUST be amended with:
- DTU route file references
- DTU types.rs field equivalencies
- SAP-2 exclusion documentation for any deliberately excluded fields

Until the DTU story executes, near-term tests run against the live monroe sensor only.

## Invariants

- DI-005: OCSF schema validity — `detection_finding` class_uid 2004 is a valid OCSF class
- `event_id` (Integer, REQUIRED) is the platform-unique event identifier; REQUIRED flags it as
  a mandatory push-down parameter — not a null-row control (BC-2.11.007; see
  `pushdown.rs::classify_predicates` REQUIRED priority ordering). When a response row omits the
  `event_id` field, `build_column_array` (within `pipeline_result_to_record_batch`, reached via
  `SpecDrivenSensorAdapter::fetch` in `crates/prism-bin/src/spec_driven_adapter.rs`) produces a
  null `finding_info_uid` cell via absent-field passthrough; the row is NOT dropped — `time` and
  `raw_extensions` remain populated and the row continues. (`ColumnMapper::map_record` is a
  non-production reference mirror — it has zero production callers and is NOT the production path.)
- Network 5-tuple fields (`source_ip`, `dest_ip`, `protocol`, `dest_port`, `source_port`,
  `ip_protocol`) are Tier-2 — they are NOT exposed as standalone Arrow columns; queries against
  them by raw TOML name MUST raise E-QUERY-038 with `available_columns` containing `raw_extensions`
- `detection_time` ISO 8601 parsing uses ADR-028 §D8-B implicit iso8601 default
  (`timestamp_formats` omitted → `effective_formats` returns `["iso8601"]`; null passthrough
  when absent)

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SENSOR-001` | Claroty API returns non-200 HTTP for POST /api/v1/ot_activity_events/ | Structured error with sensor=claroty, status, body; partial results returned for previously fetched pages |
| `E-QUERY-038` | Query references `source_ip`, `dest_ip`, `protocol` or any other Tier-2 column by its raw TOML name | Column-not-found at plan time; `available_columns` contains `raw_extensions`, `finding_info_uid`, `time`, `activity_name`, `message`, `class_uid`, `_sensor` |
| `E-SPEC-018` | Datetime parse failure on `detection_time` for a non-null non-ISO-8601 value | `E-SPEC-018 TimestampParseFailure` — null demoted with warning; row continues |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-016-001 | Row missing `event_id` field in API response | `finding_info_uid` cell is null; `time` and `raw_extensions` remain populated; row is NOT dropped; no hard error; subsequent rows continue. Attribution: `build_column_array` absent-field passthrough within `pipeline_result_to_record_batch` (`SpecDrivenSensorAdapter::fetch` path) — independent of the REQUIRED push-down-parameter flag. (`ColumnMapper::map_record` is a non-production reference mirror.) |
| EC-016-016-002 | `related_alert_ids` is an empty array `[]` | Serialized as `[]` JSON in `raw_extensions`; not null |
| EC-016-016-003 | `detection_time` is null/absent | Null Datetime cell; no fallback chain declared (optional field); ADR-028 §D8-B null-passthrough |
| EC-016-016-004 | `mode` field absent (not all event types change mode) | Null string cell in raw_extensions; no error |
| EC-016-016-005 | Network 5-tuple fields partially absent (e.g., only source_ip present, dest_ip absent) | Present fields serialized into raw_extensions; absent fields not serialized; no error |
| EC-016-016-006 | Query against Tier-2 network field `source_ip` by raw name | E-QUERY-038; `available_columns` contains `raw_extensions` but NOT `source_ip` |

## Related BCs

- BC-2.16.013: Bundled Sensor Spec Authoring — parent spec for the Claroty sensor; this BC
  adds the `claroty_ot_activity_events` table to the Claroty sensor surface (depends on)
- BC-2.02.005: Claroty xDome Field Mapping to OCSF (9 Data Sources) — the "9 data sources"
  enumeration already includes ot_activity_events; this BC specifies the concrete column contract (composes with)
- BC-2.01.007: Claroty Bearer Token Auth with Polymorphic ID Handling — auth mechanism unchanged;
  preconditions list includes ot_activity_events in the 9 endpoint enumeration (depends on)

## Architecture Anchors

- `crates/prism-sensors/specs/claroty.sensor.toml` — TOML spec file authoring target
- `crates/prism-spec-engine/src/spec_parser.rs` — ColumnSpec, FetchStep deserialization
- `crates/prism-spec-engine/src/pipeline.rs` — OffsetLimit POST-body injection
- `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` — `"detection_finding"` arm (existing)
- `crates/prism-bin/src/spec_driven_adapter.rs` — `pipeline_result_to_record_batch`
- `.factory/objectives/xdome-v1-validation/endpoint-spike-findings.md §Spike 2` — OCSF class decision authority

## Story Anchor

S-CLAROTY-OT-EVENTS-001 (draft — Wave A)

## VP Anchors

(none — no formal verification properties defined; structural tests via story RG list)

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.16.016-001 | `SELECT finding_info_uid, activity_name FROM claroty.claroty_ot_activity_events LIMIT 5` against live monroe | Succeeds (no E-QUERY-038); rows have non-null `finding_info_uid` (event_id); `activity_name` contains event type strings |
| TV-BC-2.16.016-002 | `SELECT * FROM claroty.claroty_ot_activity_events LIMIT 1` | Wire JSON contains `class_uid = 2004`; `raw_extensions` present with network 5-tuple fields; `finding_info_uid` present |
| TV-BC-2.16.016-003 | `SELECT source_ip FROM claroty.claroty_ot_activity_events LIMIT 1` | E-QUERY-038; `available_columns` contains `raw_extensions`, `finding_info_uid`, `time`, `activity_name`; does NOT contain `source_ip` |
| TV-BC-2.16.016-004 | `SELECT raw_extensions FROM claroty.claroty_ot_activity_events LIMIT 5` | Succeeds; raw_extensions JSON contains `source_ip`, `dest_ip`, `protocol` keys |
| TV-BC-2.16.016-005 | `SELECT time FROM claroty.claroty_ot_activity_events LIMIT 1` | Succeeds; `time` is the OCSF Arrow field name for `detection_time` Tier-1 mapping |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — structural tests cover via story RG list per S-CLAROTY-OT-EVENTS-001; holdout evaluator exercises live monroe surface |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC specifies the TOML table contract for the Claroty xDome `claroty_ot_activity_events` table, defining columns (typed with OCSF mappings), multi-step fetch pipeline (POST-for-read, offset_limit pagination, trailing-slash path), and Tier-1/Tier-2 OCSF column classification per ADR-058. This is exactly what CAP-029 defines: sensor adapters defined in TOML spec files with tables, columns, pipelines, and pagination config. |
| L2 Invariants | DI-005 |
| Priority | P0 |
| Story | S-CLAROTY-OT-EVENTS-001 |
| DTU Status | NONE — no DTU exists; near-term tests against live monroe sensor only; DTU deferred to D-2200 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | f-ote-p1-high-001-arch-anchor | 2026-08-31 | product-owner | F-OTE-P1-HIGH-001: §Architecture Anchors `spec_driven_adapter.rs` crate corrected `crates/prism-spec-engine` → `crates/prism-bin` (`crates/prism-bin/src/spec_driven_adapter.rs — pipeline_result_to_record_batch`); §Invariants already cited the correct `crates/prism-bin/src/` path — §Architecture Anchors is now internally consistent. |
| 1.3 | med-1-med-3-mechanism-correction | 2026-08-31 | product-owner | MED-1: §Postconditions §1 TOML bare table_name corrected from `"claroty_ot_activity_events"` to `"ot_activity_events"`; added derivation note (`{sensor_id}_{table_name}` = registered/queryable name). MED-3: §Invariants and EC-016-016-001 re-anchored from non-production `ColumnMapper::map_record` to production mechanism `build_column_array` within `pipeline_result_to_record_batch` (reached via `SpecDrivenSensorAdapter::fetch`); `map_record` retained as explicitly-labeled non-production reference mirror only. |
| 1.2 | med-1-required-semantics-correction | 2026-08-31 | product-owner | MED-1 (POL-4) REQUIRED-semantics fix — §Invariants and EC-016-016-001 prose corrected: causal attribution of absent-field passthrough moved from "spec-engine REQUIRED semantics" to `ColumnMapper::map_record` default absent-field handling; §Invariants now explains REQUIRED as mandatory push-down-parameter flag per BC-2.11.007 / `pushdown.rs::classify_predicates` priority ordering; EC-016-016-001 Expected Behavior updated to reflect that `finding_info_uid` is null while `time` and `raw_extensions` remain populated (row not dropped). No postcondition mechanics, column list, test vectors, or ACs changed. |
| 1.1 | f-1-remove-uncertainty | 2026-08-30 | product-owner | F-1 pre-TDD remove-uncertainty fix — TOML literal-string body_template corrected to single-line double-quoted string; zero semantic change; 21 fields unchanged. |
| 1.0 | xdome-wave-a-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring — Claroty xDome OT activity events queryable surface contract per xdome-endpoint-expansion-plan.md Wave A G2 and spike-findings §Spike 2. TOML table contract, 21-column Tier-1/Tier-2 classification per ADR-058, Option B OCSF class rationale (detection_finding/2004 over network_activity/4001 per plan governing constraint), SAP-2 N/A documentation (no DTU), D-2200 deferred DTU anchor. |
