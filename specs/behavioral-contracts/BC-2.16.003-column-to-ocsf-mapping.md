---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: 2026-08-16
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "b6ce69f"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.003: Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec

## Description

After a spec-driven table's multi-step fetch pipeline returns raw records, columns with
`ocsf_field` mappings are translated to the corresponding OCSF protobuf fields using
the standard four-tier resolution from BC-2.02.008. Columns without mappings are
preserved in the `raw_extensions` JSON blob per BC-2.02.007. Type coercion is applied
for mismatched types with non-fatal fallback to `raw_extensions` on failure.

The coercion rule follows a **String-type-first** precedence: when the TOML spec
declares `column_type = "string"`, any scalar JSON value (Number, Bool) from the API
is normalized to a JSON string before the OCSF-path numeric-suffix heuristic fires.
This prevents integer API IDs from landing in string columns as un-coerced numbers, and
prevents string usernames mapped to `*.uid`-style OCSF paths from being incorrectly
demoted to `raw_extensions` by the numeric heuristic (LIVE-DRIFT-003, human-authorized
gap closure 2026-08-11 per CLAUDE.md §Source-of-Truth Precedence item 7; this was a
genuine absence of specification, not a code-vs-spec conflict).

The resulting `OcsfEvent` is uniform across all sensors: downstream consumers
(detection rules, cross-sensor correlation, decorators) cannot distinguish spec-driven
data from built-in adapter data. Invalid OCSF field paths produce a warning at spec
load time (not a hard error) because OCSF schema extensions may introduce fields not
in the compiled schema.

**Interpretation A — Underscore-flattened Arrow Field Names (ADR-058 v2.0):**
When a sensor spec sets `ocsf_column_naming = true` (ADR-058 §B2), the Arrow
RecordBatch field name for each mapped column uses the underscore-flattened
`ocsf_field` path (dots replaced with underscores). Example: `ocsf_field =
"finding_info.uid"` produces Arrow field `finding_info_uid`. Columns without
`ocsf_field` retain `col.name` as their Arrow field name and are collected into
a `raw_extensions` JSON blob column. This naming convention makes OCSF-semantic
identifiers directly queryable in PrismQL without quoting (e.g., `SELECT finding_info_uid
FROM claroty_alerts`). Claroty is the first sensor to receive `ocsf_column_naming
= true`; all four Claroty tables (alerts, audit_logs, devices, device_alert_relations)
operate under Interpretation A once Stage 2 (S-ADR058-OCSF-ROUTING-001) ships.
The contracted OCSF classes and column-to-OCSF mappings for all four Claroty tables
are specified in §Postconditions §Claroty Contracted OCSF Mappings.

## Preconditions
- A spec-driven table has been fetched via the multi-step pipeline (BC-2.16.002) and raw records are available
- The table's `ColumnSpec` entries include `ocsf_field` mappings (some columns may have `ocsf_field: None`, meaning no OCSF mapping)
- The OCSF normalizer (CAP-003) is available
- `ColumnSpec.column_type` is one of `String | Integer | Float | Boolean | Datetime | Json` (prism_core::column::ColumnType variants per ADR-024)
- When the sensor's `SensorSpec.ocsf_column_naming == true`: the contracted `ocsf_field` values for each column have been validated against OCSF v1.7.0 and corrected per ADR-058 §K4 (KF-01 through KF-12); the corrected TOML and `class_selector.rs` code changes are in place per S-ADR058-OCSF-ROUTING-001 AC-005 / §I5

## Postconditions

### Column Routing

For each record fetched from the spec-driven sensor:
- Columns with an `ocsf_field` value are mapped to the corresponding OCSF field in the DynamicMessage protobuf representation
- The mapping follows the standard four-tier field resolution (BC-2.02.008): Prism metadata fields, proto descriptor fields, unmapped JSON blob, None
- Columns without an `ocsf_field` mapping are preserved in the `raw_extensions` JSON blob (consistent with BC-2.02.007)
- The `ocsf_class` declared at the table level determines which OCSF event class the DynamicMessage uses; the declared class must be a valid OCSF v1.7.0 class name (e.g., `detection_finding`, `entity_management`, `inventory_info`, `network_activity`) — not an OCSF object name

### Interpretation A: Arrow Field Naming (ocsf_column_naming = true)

When `sensor_spec.ocsf_column_naming == true` (ADR-058 §B2):

- For each column with `ocsf_field = "a.b.c"`, the Arrow RecordBatch schema field name is `a_b_c` (all dots replaced with underscores; `ocsf_field_to_arrow_name` helper function per ADR-058 §I1).
- For each column with no `ocsf_field`, the Arrow field name is `col.name` (fallback) and the value is collected into the `raw_extensions` JSON blob column.
- The `raw_extensions` column is itself an Arrow `Utf8` column containing a serialized JSON object; it is queryable as `SELECT raw_extensions FROM <table>` but nested keys are not independently filterable without JSON path functions.
- `prism_describe` for a sensor with `ocsf_column_naming = true` returns `name: "<underscore_flattened_name>"` and `description: "<original.dotted.path>"` for each mapped column (ADR-058 §G).
- A fail-closed collision check is enforced at `pipeline_result_to_record_batch` execution time: no flattened `ocsf_field` name may equal the `col.name` of a different column in the same table (ADR-058 §J2); violation returns `Err(ArrowError::SchemaError(...))`, blocking schema construction.

### Claroty Contracted OCSF Mappings (v1.5)

The following tables specify the contracted-correct `ocsf_field` values and Arrow field
names for all four Claroty sensor tables under Interpretation A. These values reflect
ADR-058 v2.4 §K4 corrections (KF-01 through KF-12). The TOML corrections and
`class_selector.rs` code changes are delivered by S-ADR058-OCSF-ROUTING-001 AC-005 / §I5.
Columns listed as going to `raw_extensions` remain accessible as blob keys but are not
independently filterable.

**`alerts` table — `ocsf_class = "detection_finding"` (class_uid 2004; VALID):**

| col.name | Contracted ocsf_field | Arrow name (Interp. A) | Note |
|----------|-----------------------|------------------------|------|
| `id` | `finding_info.uid` | `finding_info_uid` | KF-03: was `finding.uid`; `detection_finding` has required `finding_info` attr, no bare `finding` attr |
| `alert_type_name` | (none — removed) | `raw_extensions` | KF-09: `type_name` is OCSF-computed from `type_uid`; vendor value corrupts OCSF class metadata |
| `category` | (none — removed) | `raw_extensions` | KF-08: `class_name` is OCSF-computed from `class_uid`; vendor value overwrites "Detection Finding" |
| `status` | `status` | `status` | VALID |
| `detected_time` | `time` | `time` | VALID |
| `updated_time` | `finding_info.modified_time` | `finding_info_modified_time` | KF-12: was `end_time` (event-end time); `updated_time` = record last-modified; `finding_info.modified_time` confirmed in OCSF v1.7.0 |
| `devices_count` | (none — removed) | `raw_extensions` | KF-10: OCSF `count` = event dedup counter; `devices_count` = affected device count; distinct semantics |
| `description` | `message` | `message` | VALID |
| `alert_class` | (none) | `raw_extensions` | no ocsf_field declared |
| `ot_devices_count` | (none) | `raw_extensions` | no ocsf_field declared |
| `alert_name` | `finding_info.title` | `finding_info_title` | KF-04: was `finding.title`; same root error as KF-03 |

**`audit_logs` table — `ocsf_class = "entity_management"` (class_uid 3004; KF-01: was `audit_activity`, absent from OCSF v1.7.0; `entity_management` selected because it has `comment` attr required for `note → comment` mapping; `account_change` 3001 lacks `comment`, causing silent data loss):**

| col.name | Contracted ocsf_field | Arrow name (Interp. A) | Note |
|----------|-----------------------|------------------------|------|
| `id` | (none — removed) | `raw_extensions` | KF-05 PO decision: `activity_uid` absent from OCSF v1.7.0; `activity_id` is numeric enum (not UID); audit record ID preserved in `raw_extensions` as deduplication reference |
| `action` | `activity_name` | `activity_name` | VALID |
| `user_display_name` | `actor.user.name` | `actor_user_name` | VALID |
| `category` | (none — removed) | `raw_extensions` | KF-11: `category_name` is OCSF-computed from `category_uid`; vendor value corrupts OCSF category metadata |
| `timestamp` | `time` | `time` | VALID |
| `details` | `message` | `message` | VALID |
| `username` | `actor.user.uid` | `actor_user_uid` | VALID; `column_type = "string"` — Rule 1 preempts `uid` numeric-suffix heuristic (EC-016-013-005) |
| `note` | `comment` | `comment` | VALID; requires `entity_management` (3004); `account_change` (3001) lacks `comment` causing data loss |

**`devices` table — `ocsf_class = "inventory_info"` (class_uid 5001; KF-02: was `device`, which is an OCSF object, not a class; `device.*` paths resolve via `inventory_info.device` required attribute; `risk_score` and `status_code` are class-level attrs on `inventory_info` directly):**

| col.name | Contracted ocsf_field | Arrow name (Interp. A) | Note |
|----------|-----------------------|------------------------|------|
| `uid` | `device.uid` | `device_uid` | VALID |
| `asset_id` | `device.instance_uid` | `device_instance_uid` | VALID |
| `device_category` | `device.type_category` | `device_type_category` | Vendor-extended (§J3): Claroty "OT Device"/"IT Device" values outside OCSF controlled vocabulary |
| `device_type` | `device.type_label` | `device_type_label` | KF-06 PO decision: vendor-extended; `device.type_name` absent from OCSF v1.7.0 device object; OT subcategory ("PLC", "HMI") is demo-critical for filtering; follows §J3 vendor-extension precedent; no flag-transition shadow against any `col.name` in the `devices` table |
| `risk_score` | `risk_score` | `risk_score` | VALID; class-level attr on `inventory_info` |
| `retired` | `status_code` | `status_code` | VALID; class-level attr on `inventory_info` |
| `device_name` | `device.name` | `device_name` | VALID |
| `os_category` | `device.os.name` | `device_os_name` | VALID |
| `ip_list` | (none) | `raw_extensions` | no ocsf_field declared; ENRICH-1 wildcard column |
| `mac_list` | (none) | `raw_extensions` | no ocsf_field declared; ENRICH-1 wildcard column |
| `network_list` | (none) | `raw_extensions` | no ocsf_field declared; ENRICH-1 wildcard column |
| `vlan_list` | (none) | `raw_extensions` | no ocsf_field declared; ENRICH-1 wildcard column |
| `purdue_level` | (none) | `raw_extensions` | no ocsf_field declared |
| `site_name` | (none) | `raw_extensions` | no ocsf_field declared |
| `device_subcategory` | (none) | `raw_extensions` | no ocsf_field declared |
| `device_type_family` | (none) | `raw_extensions` | no ocsf_field declared |
| `criticality` | (none) | `raw_extensions` | no ocsf_field declared |
| `is_online` | (none) | `raw_extensions` | no ocsf_field declared |
| `manufacturer` | (none) | `raw_extensions` | no ocsf_field declared |
| `model` | (none) | `raw_extensions` | no ocsf_field declared |

**`device_alert_relations` table — `ocsf_class = "detection_finding"` (class_uid 2004; VALID):**

| col.name | Contracted ocsf_field | Arrow name (Interp. A) | Note |
|----------|-----------------------|------------------------|------|
| `device_uid` | `device.uid` | `device_uid` | VALID; `detection_finding.device.uid` confirmed |
| `alert_id` | `finding_info.uid` | `finding_info_uid` | KF-07: was `finding.uid`; same root error as KF-03 |
| `device_alert_detected_time` | `time` | `time` | VALID |
| `device_risk_score` | `risk_score` | `risk_score` | VALID |
| `alert_note` | `comment` | `comment` | VALID |
| `device_alert_status` | `status` | `status` | VALID |
| `network_signature_severity` | (none) | `raw_extensions` | no ocsf_field declared |
| `network_signature_confidence` | (none) | `raw_extensions` | no ocsf_field declared |
| `malicious_ip_severity` | (none) | `raw_extensions` | no ocsf_field declared |
| `external_ip` | (none) | `raw_extensions` | no ocsf_field declared |

### Type Coercion Algorithm

`ColumnMapper::coerce_value` applies the following precedence:

**Rule 1 — String-type-first (LIVE-DRIFT-003):**
When `column_type = "string"`, any scalar JSON value is normalized to a JSON string value
before the OCSF path heuristic is consulted:

| Input JSON type | Wire output | Notes |
|-----------------|-------------|-------|
| String          | String (unchanged) | No transformation |
| Number          | String(n.to_string()) | e.g. `132` → `"132"` |
| Bool            | String(b.to_string()) | `true` → `"true"`, `false` → `"false"` |
| Null            | Null (pass-through) | See EC-016-013-006 |
| Array           | Array (pass-through) | KNOWN GAP — see EC-016-013-007 |
| Object          | Object (pass-through) | KNOWN GAP — see EC-016-013-008 |

Rule 1 applies regardless of the OCSF field path's numeric suffix. This correctly
overrides the `is_numeric_ocsf_field` heuristic for string-declared columns, because
the spec author's declared `column_type` is authoritative over a path-name pattern.

**Rule 2 — OCSF numeric-path heuristic:**
For non-`String` columns only, when the OCSF field path's last segment is one of
`event_code`, `class_uid`, `activity_id`, `type_uid`, `severity_id`, `status_id`,
`action_id`, `count`, `duration`, `port`, `pid`, `uid`, `code`:
- If the incoming value is `Value::String`, attempt `i64` parse
- On parse success: return `Value::Number(n)` — string-encoded integer coerced
- On parse failure: return `Err(CoercionWarning)` — demotion to `raw_extensions`

NOTE: the `uid` suffix correctly coerces `class_uid`/`type_uid` (OCSF integer enum
codes). It would incorrectly trigger for `actor.user.uid` (a string identifier) unless
the spec declares `column_type = "string"` — in which case Rule 1 preempts it.
The canonical `actor.user.uid` usage pattern requires `column_type = "string"` in the
TOML spec.

**Rule 3 — Pass-through default:**
All other cases: return the value unchanged. No coercion error.

### Full Coercion Matrix

| declared `column_type` | Input JSON type | OCSF path | Rule | Wire output | Outcome |
|------------------------|-----------------|-----------|------|-------------|---------|
| String | String | any | Rule 1 | String (unchanged) | OCSF field |
| String | Number | any | Rule 1 | String(n.to_string()) | OCSF field |
| String | Bool | any | Rule 1 | String(b.to_string()) | OCSF field |
| String | Null | any | Rule 1 | Null | OCSF field |
| String | Array | any | Rule 1 gap | Array (WRONG) | OCSF field — KNOWN GAP |
| String | Object | any | Rule 1 gap | Object (WRONG) | OCSF field — KNOWN GAP |
| Integer | Number | any | Rule 3 | Number (unchanged) | OCSF field |
| Integer | String | numeric suffix | Rule 2 | Number(parse) or Err | OCSF field or raw_extensions |
| Integer | String | non-numeric suffix | Rule 3 gap | String (WRONG) | OCSF field — KNOWN GAP |
| Integer | Bool/Null/Array/Object | any | Rule 3 | Unchanged | OCSF field (no coercion) |
| Float | (any) | any | Rule 3 | Unchanged | OCSF field (no float coercion in v1) |
| Boolean | (any) | any | Rule 3 | Unchanged | OCSF field (no bool coercion in v1) |
| Datetime | (any) | any | Rule 3 | Unchanged | OCSF field; datetime parsing is downstream |
| Json | (any) | any | Rule 3 | Unchanged | OCSF field (any JSON is valid) |

**KNOWN GAPs (require a fix story — see §Traceability):**
- EC-016-013-007: `column_type = "string"` + Array input: currently passes array to OCSF field; MUST divert to raw_extensions with CoercionWarning
- EC-016-013-008: `column_type = "string"` + Object input: same defect class as EC-016-013-007
- EC-016-013-009: `column_type = "integer"` + String input on non-numeric OCSF path: currently passes string to OCSF field; MUST parse and divert on failure

### Coercion Warning Observability

`ColumnMapper::coerce_value` returns `Err(CoercionWarning)` on failure; the caller
(`ColumnMapper::map_record`) places the value in `raw_extensions` and records the
warning in `MappingResult.coercion_warnings`.

**DEFECT — missing `tracing::warn!`:** The current implementation does NOT emit a
`tracing::warn!` at the point of demotion. This violates BC-2.02.011 §Postconditions
("A warning-level log entry is emitted for each normalization issue") and the
§Error Conditions table below. Per BC-5.39.001, this defect routes to the implementer
for fix in the next cascade. Until fixed, `CoercionWarning` is only observable via the
returned `MappingResult.coercion_warnings` vec — it is NOT surfaced to operators or
the audit trail. See §Story Anchor for the anchoring story.

The required emission at demotion time is:
```
tracing::warn!(
    column = %warning.column_name,
    expected_ocsf_type = %warning.expected_ocsf_type,
    actual_value = %warning.actual_value,
    event_type = "column_coercion_failure",
    "coerce_value: type mismatch; field diverted to raw_extensions"
);
```
This `event_type` value MUST be registered in BC-2.16.002 §Postconditions Canonical
Structured Event Catalog per PG-LP11-001.

## OCSF Field Validation
- At spec load time (BC-2.16.009), each `ocsf_field` value is validated against the compiled OCSF protobuf schema
- Invalid OCSF field paths produce a warning at load time but do not reject the spec (the mapping is skipped at runtime, and the column goes to `raw_extensions`)
- This is a warning, not an error, because OCSF schema extensions may introduce fields not in the compiled schema
- Columns that write vendor values into OCSF-computed reserved fields (`class_name`, `type_name`, `category_name`, `count`) are NOT caught by schema-path validation (these paths ARE valid OCSF paths); they are caught by semantic review per ADR-058 §K5 Divergence 1; the correct fix is to remove the `ocsf_field` declaration so the column falls to `raw_extensions` (KF-08 through KF-11)

## Invariants
- Coercion failures are non-fatal: the field value is preserved in `raw_extensions` (record is NEVER dropped due to type mismatch)
- The `ocsf_class` at table level determines the OCSF event class for all records in that table; the declared class MUST be a valid OCSF v1.7.0 class name (not an object name); Claroty contracted classes: `detection_finding` (alerts, device_alert_relations), `entity_management` (audit_logs), `inventory_info` (devices)
- Spec-driven OcsfEvents are indistinguishable from built-in adapter OcsfEvents to downstream consumers
- The declared `column_type` in the TOML spec is the authoritative wire shape for the column; the OCSF path name heuristic is a secondary fallback only for non-String columns
- NULL vs absent: a column absent from the raw record is SKIPPED (not placed in either `mapped_fields` or `raw_extensions`); a column present with `Value::Null` is placed in its destination (either OCSF field or `raw_extensions`) as a JSON null value
- Under Interpretation A (`ocsf_column_naming = true`): no flattened `ocsf_field` name in a table may equal the `col.name` of a different column in the same table (flag-transition shadow prevention, ADR-058 §J2); violation fails closed at schema construction time

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning (non-fatal) | Coercion failure for a column value | Field diverted to `raw_extensions`; `CoercionWarning` created; MUST emit `tracing::warn!(event_type = "column_coercion_failure")` per BC-2.02.011 — DEFECT: not yet emitted |
| Warning (non-fatal) | Invalid `ocsf_class` in table spec (class absent from OCSF v1.7.0, e.g. `audit_activity` / `device`) | All records use generic `base_event` class (OCSF class 0) with startup warning |
| KNOWN GAP | `column_type = "string"` + Array/Object input | Currently passes structured value to OCSF field instead of diverting to raw_extensions (EC-016-013-007, EC-016-013-008) |
| KNOWN GAP | `column_type = "integer"` + String input on non-numeric OCSF path | Currently passes string to OCSF field instead of coercing/diverting (EC-016-013-009) |
| Schema error (fail-closed) | Under `ocsf_column_naming = true`, flattened `ocsf_field` name equals `col.name` of a different column in the same table | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — flag cannot activate until collision is resolved |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-013-001 | Column with `ocsf_field: None` | Column value placed in `raw_extensions`; no coercion attempted |
| EC-016-013-002 | `column_type = "integer"`, string `"42"` on numeric-suffix OCSF path | Parsed as integer 42; wire output is `Value::Number(42)`; placed in OCSF field |
| EC-016-013-003 | `column_type = "integer"`, string `"not-a-number"` on numeric-suffix OCSF path | Parse fails; `Err(CoercionWarning)` returned; field diverted to `raw_extensions`; record included |
| EC-016-013-004 | `column_type = "string"`, API returns `Value::Number(132)`, OCSF path `"finding_info.uid"` (uid suffix) | Rule 1 fires before heuristic; wire output is `Value::String("132")`; placed in OCSF field. Concrete exemplar: Claroty `alerts.id` (LIVE-DRIFT-003; ocsf_field corrected from `finding.uid` to `finding_info.uid` per KF-03) |
| EC-016-013-005 | `column_type = "string"`, string username `"analyst"`, OCSF path `"actor.user.uid"` (uid suffix) | Rule 1 fires; `is_numeric_ocsf_field` NOT consulted; string preserved; placed in OCSF field without CoercionWarning. Concrete exemplar: Claroty audit_logs `username` column (LIVE-DRIFT-003) |
| EC-016-013-006 | `column_type = "string"`, `Value::Null` input | Rule 1 pass-through; `Value::Null` placed in OCSF field; absent key is DISTINCT from null (see wire-shape invariant in §Invariants) |
| EC-016-013-007 | `column_type = "string"`, `Value::Array([...])` input | KNOWN GAP: Rule 1 pass-through; array placed in OCSF string field (incorrect — MUST divert to raw_extensions). Fix: S-ADR058-OCSF-COERCION-001 |
| EC-016-013-008 | `column_type = "string"`, `Value::Object({...})` input | KNOWN GAP: same defect class as EC-016-013-007. Fix: S-ADR058-OCSF-COERCION-001 |
| EC-016-013-009 | `column_type = "integer"`, `Value::String("42")`, OCSF path `"device.bytes"` (non-numeric suffix) | KNOWN GAP: Rule 3 pass-through; string placed in integer-typed OCSF field (incorrect — MUST parse and divert on failure). Fix: S-ADR058-OCSF-COERCION-001 |
| EC-016-013-010 | `column_type = "boolean"`, `Value::Bool(true)` | Rule 3 pass-through; bool placed in OCSF field unchanged |
| EC-016-013-011 | Invalid `ocsf_class` (e.g., `"made_up_class"` or OCSF object name like `"device"` or absent-class like `"audit_activity"`) | Records use base_event (class 0); startup warning at spec load time |
| EC-016-013-012 | Two sensors both map a column → `ocsf_field = "device.ip"` | Under Interpretation A (`ocsf_column_naming = true`): both queryable as `device_ip` (Arrow field name under underscore-flattened naming); cross-sensor JOIN on `device_ip` works transparently |
| EC-016-013-013 | KF-08: `alerts.category` previously mapped to reserved OCSF field `class_name` | Contracted fix: `ocsf_field` removed from `alerts.category`; column goes to `raw_extensions`; `class_name` on the record reflects OCSF-computed "Detection Finding", not the Claroty vendor category string; test: verify `raw_extensions` blob contains `category` key; verify Arrow schema has no `class_name` column carrying Claroty vendor string |
| EC-016-013-014 | KF-09: `alerts.alert_type_name` previously mapped to reserved OCSF field `type_name` | Contracted fix: `ocsf_field` removed; `alert_type_name` value preserved in `raw_extensions`; `type_name` reflects OCSF-computed value derived from `type_uid`, not the Claroty alert type label; test: verify `raw_extensions` blob contains `alert_type_name` key |
| EC-016-013-015 | KF-10: `alerts.devices_count` previously mapped to OCSF dedup counter field `count` | Contracted fix: `ocsf_field` removed; `devices_count` integer preserved in `raw_extensions`; OCSF `count` semantics (number of times events in same logical group occurred) are distinct from affected device count; test: verify `raw_extensions` blob contains `devices_count` key |
| EC-016-013-016 | KF-11: `audit_logs.category` previously mapped to reserved OCSF field `category_name` | Contracted fix: `ocsf_field` removed; `category` value preserved in `raw_extensions`; `category_name` reflects OCSF-computed label from `category_uid`; test: verify `raw_extensions` blob contains `category` key |
| EC-016-013-017 | KF-03: `alerts.id` maps to corrected `ocsf_field = "finding_info.uid"` (Arrow: `finding_info_uid`) | `detection_finding` class has required `finding_info` attribute (not bare `finding`); Arrow schema under Interpretation A carries `finding_info_uid` column; `SELECT finding_info_uid FROM claroty_alerts` returns the Claroty alert ID; test: assert serialized Arrow schema field name is `finding_info_uid` (wire-shape assertion per CLAUDE.md §Conventions) |
| EC-016-013-018 | KF-04: `alerts.alert_name` maps to corrected `ocsf_field = "finding_info.title"` (Arrow: `finding_info_title`) | `detection_finding.finding_info.title` confirmed in OCSF v1.7.0; same root error as KF-03; test: assert Arrow field `finding_info_title` carries the Claroty alert name string |
| EC-016-013-019 | KF-12: `alerts.updated_time` maps to corrected `ocsf_field = "finding_info.modified_time"` (Arrow: `finding_info_modified_time`) | `end_time` = "time event ended" (wrong semantic); `updated_time` = "when alert record was last modified in source system"; `finding_info.modified_time` confirmed in OCSF v1.7.0 `finding_info` object; test: assert Arrow field `finding_info_modified_time` carries the datetime; verify no `end_time` Arrow field for this column |
| EC-016-013-020 | KF-05 PO decision: `audit_logs.id` (record UID) has no valid standard OCSF target; goes to `raw_extensions` | `activity_uid` absent from OCSF v1.7.0 `dictionary_attributes`; `activity_id` is a numeric enum (not a record UID); vendor extension not warranted because audit log record IDs are deduplication/reference fields, not primary operational filter targets; data preserved in `raw_extensions` blob; test: verify `raw_extensions` blob for an audit_log record contains `id` key; verify no first-class Arrow column named `activity_uid` |
| EC-016-013-021 | KF-06 PO decision: `devices.device_type` (OT subcategory, e.g. "PLC", "HMI") uses vendor-extended `ocsf_field = "device.type_label"` (Arrow: `device_type_label`) | `device.type_name` absent from OCSF v1.7.0 device object; OT subcategory labels are demo-critical for filtering (`SELECT * FROM claroty_devices WHERE device_type_label = 'PLC'`); vendor-extension follows §J3 precedent for `device_category`; `device.type_label` does not appear in any other Claroty devices column `col.name`, satisfying the flag-transition shadow rule (ADR-058 §J2); test: assert Arrow field `device_type_label` carries the OT subcategory string |
| EC-016-013-022 | KF-07: `device_alert_relations.alert_id` maps to corrected `ocsf_field = "finding_info.uid"` (Arrow: `finding_info_uid`) | Same root error as KF-03; `detection_finding` has no bare `finding` attribute; test: assert Arrow field `finding_info_uid` in the `device_alert_relations` result carries the Claroty alert ID (string, polymorphic via Rule 1) |
| EC-016-013-023 | KF-01 class correction: `entity_management` (3004) `comment` attribute is accessible; `audit_logs.note → comment` mapping produces data; **wire-level: `class_uid = 3004` in Arrow `class_uid` Int32 column (Path A)** | Under `account_change` (3001, the prior wrong class), `set_nested_field` silently no-ops the `note → comment` mapping because `account_change` has no `comment` attr in its protobuf descriptor — the value is silently dropped; under `entity_management` (3004), `comment` is a valid class-level attr and the mapping resolves; test: load `entity_management` record with `note` value populated; assert Arrow field `comment` carries the note value (not null, not missing). **Wire-level postcondition (ADR-058 §I5 v2.6 wire-shape assertion obligation):** a `RecordBatch` materialized from Claroty `audit_logs` data with `ocsf_class = "entity_management"` via `pipeline_result_to_record_batch` (Path A) MUST carry `class_uid = 3004` in the Arrow `class_uid` Int32 column — NOT 3001 (prior wrong `account_change` arm value) and NOT 0 (`.unwrap_or(0)` BASE_EVENT fallback). Wire-level test: assert the serialized `class_uid` column value equals `3004` at the `RecordBatch` level (not only at the resolver unit-test level — per wire-shape assertion discipline). Anchored: S-ADR058-OCSF-ROUTING-001. |
| EC-016-013-024 | KF-02 class correction: `inventory_info` (5001) `device.*` attribute paths resolve via the `device` required attribute; **wire-level: `class_uid = 5001` in Arrow `class_uid` Int32 column (Path A) — regression-prevention** | `devices` table columns `uid`, `asset_id`, `device_category`, `device_type`, `device_name`, `os_category` all resolve via `inventory_info.device.*` path hierarchy; class-level `risk_score` and `status_code` attrs confirmed at `inventory_info` class level directly (not nested under `device`); test: assert Arrow schema for `claroty_devices` under Interpretation A includes `device_uid`, `device_instance_uid`, `risk_score`, `status_code` as first-class columns. **Wire-level postcondition — regression-prevention (ADR-058 §I5 v2.6 wire-shape assertion obligation):** a `RecordBatch` materialized from Claroty `devices` data with `ocsf_class = "inventory_info"` via `pipeline_result_to_record_batch` (Path A) MUST carry `class_uid = 5001` in the Arrow `class_uid` Int32 column — NOT 0 (BASE_EVENT). This is an explicit regression-prevention assertion: without the `"inventory_info" => Ok(CLASS_UID_DEVICE_INVENTORY_INFO)` arm in `select_by_class_name`, the KF-02 TOML change from `ocsf_class = "device"` to `ocsf_class = "inventory_info"` regresses `class_uid` from the current 5001 (produced by the existing `"device"` arm) to 0 (`.unwrap_or(0)` BASE_EVENT fallback). The regression path is silent without the `ocsf.unknown_class_name` WARN (BC-2.16.002 §Canonical Structured Event Catalog v1.67 row). Wire-level test: assert the serialized `class_uid` column value equals `5001` at the `RecordBatch` level. Anchored: S-ADR058-OCSF-ROUTING-001. |

## Canonical Test Vectors

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — full mapping | all columns have `ocsf_field`; all types match | OcsfEvent with all fields mapped; `raw_extensions` empty |
| Mixed mapping | some columns have `ocsf_field`, some don't | Mapped columns in OCSF proto; unmapped in `raw_extensions` |
| Coercion failure — non-parseable string | `"not-a-number"` for integer field on numeric-suffix path | Field in `raw_extensions`; `CoercionWarning` emitted; record included |
| Integer JSON on String column | `Value::Number(132)` on `finding_info.uid`, `column_type = "string"` | Wire output `Value::String("132")`; test: `test_coerce_value_string_type_normalizes_integer_to_string` |
| String username on uid path | `Value::String("analyst")` on `actor.user.uid`, `column_type = "string"` | String preserved in OCSF field; no CoercionWarning; test: `test_coerce_value_string_type_preserves_string_username_against_uid_heuristic` |
| Invalid ocsf_class | table has unknown `ocsf_class` or OCSF object name | base_event class used; warning at load |
| Interpretation A — finding_info.uid routing | Claroty `alerts.id` = `"132"`, `ocsf_column_naming = true` | Arrow field `finding_info_uid` = `"132"`; no `finding_uid` column in schema (old wrong name absent) |
| Interpretation A — reserved field removed | Claroty `alerts.category` = `"OT Threat"`, `ocsf_column_naming = true` | `category` value in `raw_extensions` blob; no first-class Arrow column carries vendor category string as `class_name` |
| Interpretation A — entity_management comment | Claroty `audit_logs.note` = `"reviewed"`, `ocsf_column_naming = true` | Arrow field `comment` = `"reviewed"` (entity_management 3004 class has comment attr) |
| Interpretation A — OT device subcategory | Claroty `devices.device_type` = `"PLC"`, `ocsf_column_naming = true` | Arrow field `device_type_label` = `"PLC"`; queryable as `SELECT * FROM claroty_devices WHERE device_type_label = 'PLC'` |

See `.factory/specs/prd-supplements/test-vectors.md` for extended canonical vector tables.

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-017 | OCSF normalization: unmapped fields preserved in raw_extensions (proptest) — coercion failures fall into the same preservation guarantee |
| VP-016 | OCSF normalization: output is valid protobuf — coercion failures do not produce malformed protobufs (record still encodes; field merely moves to raw_extensions) |

## Related BCs

- BC-2.02.007 (composes with): governs the raw_extensions blob that coercion failures and unmapped Claroty columns land in
- BC-2.02.008 (depends on): four-tier field resolution used for OCSF field placement
- BC-2.02.011 (depends on): defines the warning-emission obligation for each coercion failure
- BC-2.16.002 (depends on): multi-step fetch pipeline whose output records are consumed here; also hosts the `column_coercion_failure` Canonical Structured Event Catalog row (SAP-1 obligation)
- BC-2.01.013 (depends on): EC-01-025 NON-CONFORMANT annotation for the ColumnMapper wiring gap — resolved for Claroty once S-ADR058-OCSF-ROUTING-001 merges

## Architecture Anchors

- `crates/prism-spec-engine/src/column_mapping.rs` — `ColumnMapper::coerce_value`, `ColumnMapper::map_record`, `is_numeric_ocsf_field`
- `crates/prism-spec-engine/tests/bc_2_16_003_test.rs` — integration tests for column routing and coercion
- `crates/prism-spec-engine/src/column_mapping.rs` `CoercionWarning` struct — returned data structure (not yet observed at wire level; SAP-1 violation until `event_type = "column_coercion_failure"` is added to BC-2.16.002 catalog)
- `crates/prism-ocsf/src/class_selector.rs` — **Path A (live production):** `select_by_class_name` is the resolver called by `pipeline_result_to_record_batch` on the spec-driven Arrow materialization path (Path A). It must gain two new arms: `"entity_management" => Ok(CLASS_UID_ENTITY_MANAGEMENT)` (3004) and `"inventory_info" => Ok(CLASS_UID_DEVICE_INVENTORY_INFO)` (5001). New const `CLASS_UID_ENTITY_MANAGEMENT = 3004` required. Without the `"entity_management"` arm, the KF-01 TOML correction (`ocsf_class = "audit_activity"` → `"entity_management"`) falls to `.unwrap_or(0)`, producing `class_uid = 0` (BASE_EVENT — a regression from the current wrong 3001). Without the `"inventory_info"` arm, the KF-02 TOML correction (`ocsf_class = "device"` → `"inventory_info"`) falls to `.unwrap_or(0)`, regressing `class_uid` from the current 5001 to 0. The `"audit_activity"` arm becomes dead code after the KF-01 TOML fix (no production TOML will declare `ocsf_class = "audit_activity"` post-correction) and MUST carry a deprecation annotation as a transitional entry pending removal. **Path B (forward-compat, zero production callers):** `select()` called from `normalize_with_mappers` (`crates/prism-ocsf/src/normalizer.rs`) — zero live query traffic today; Path B Claroty and Armis `audit_log` arms also require `Ok(CLASS_UID_ENTITY_MANAGEMENT)` correction for forward-compat when Path B is eventually wired (ADR-058 §I5 v2.6 §K5). **Process-gap obligation (ADR-058 §I5 v2.6):** `pipeline_result_to_record_batch` must emit `tracing::warn!(event_type = "ocsf.unknown_class_name", ...)` on the `Err` branch before `.unwrap_or(0)` (SOUL.md #4; SAP-1 obligation discharged in BC-2.16.002 §Canonical Structured Event Catalog v1.67 row). Armis `("armis", "audit_log")` arm in Path B requires same `CLASS_UID_ENTITY_MANAGEMENT` correction (TD-VSDD-097 sibling sweep; KF-01 code obligation per ADR-058 §I5/§K5 Divergence 3)
- `crates/prism-sensors/specs/claroty.sensor.toml` — Claroty TOML spec receiving KF-01..KF-12 corrections per S-ADR058-OCSF-ROUTING-001 AC-005 / §I5
- `crates/prism-bin/src/spec_driven_adapter.rs` — `pipeline_result_to_record_batch`; Stage 2 wiring adds `ocsf_field_to_arrow_name` dispatch per ADR-058 §I1

## Story Anchor

**Stage 1 — S-ADR058-OCSF-COERCION-001 (status: draft):** Implements:
1. Fix EC-016-013-008: `column_type = "string"` + Object input → `None` (null cell) + `tracing::warn!(event_type = "column_coercion_failure")` emission (ADR-058 §H item 1)
2. Fix EC-016-013-009: `column_type = "integer"` + String input on non-numeric OCSF path → coercion via `ColumnMapper::coerce_value` or divert to `raw_extensions` (ADR-058 §H item 2)
3. Fix EC-016-013-007: `column_type = "string"` + Array input → `raw_extensions` with `CoercionWarning` (structured-type demotion; same burst as items 1–2)
4. Add `column_coercion_failure` tracing emission in `ColumnMapper::map_record` at demotion time (ADR-058 §H item 3)
5. Register `column_coercion_failure` in BC-2.16.002 §Postconditions Canonical Structured Event Catalog (SAP-1 / PG-LP11-001 obligation)

**Stage 2 — S-ADR058-OCSF-ROUTING-001 (status: draft):** Implements:
1. `ocsf_field_to_arrow_name` helper function and `pipeline_result_to_record_batch` update: use underscore-flattened Arrow field names when `sensor_spec.ocsf_column_naming == true` (ADR-058 §I1)
2. `ocsf_column_naming = true` flag in `claroty.sensor.toml` plus all §K4 TOML corrections (KF-01 through KF-12): KF-01 requires TOML class change AND `class_selector.rs` code change (`CLASS_UID_ENTITY_MANAGEMENT = 3004`; `"audit_activity"` arm + Armis `audit_log` arm); KF-03/04/07/12 require `ocsf_field` corrections; KF-05/06/08/09/10/11 require `ocsf_field` removals; KF-02 requires `ocsf_class` change to `"inventory_info"`; KF-06 requires new vendor-extended `"device.type_label"` value (AC-005 / ADR-058 §I5)
3. Flag-transition shadow collision check extension (RG-010): `pipeline_result_to_record_batch` checks that no flattened `ocsf_field` name equals another column's `col.name` in the same table; fail-closed (ADR-058 §J2)
4. Update `test_BC_2_11_005_e2e_claroty_query_returns_data`: `row.get("uid")` → `row.get("device_uid")` (ADR-058 §E1)
5. Resolve BC-2.01.013 EC-01-025 NON-CONFORMANT annotation after merge

## VP Anchors

No VPs directly verify the coercion matrix at the property level. VP-017 (proptest,
raw_extensions preservation) covers the coercion-failure demotion path indirectly.
A dedicated VP for the coercion matrix (exhaustive column_type × JSON-type combinatorics
via proptest) is recommended as part of S-ADR058-OCSF-COERCION-001.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — the column-to-OCSF mapping is explicitly named as a spec-engine capability: "tables with columns (typed, with ColumnOptions and OCSF mappings)" and "Column OCSF mappings are validated against the compiled protobuf schema (warnings for invalid paths, not errors)" |
| L2 Invariants | DI-005 (no vendor data silently dropped) |
| Related BCs | BC-2.02.007 (raw_extensions preservation), BC-2.02.008 (four-tier field resolution), BC-2.02.011 (normalization error handling) |
| Priority | P0 |
| Known-Gap Story Needed | ANCHORED — S-ADR058-OCSF-COERCION-001 (Stage 1): CoercionWarning tracing emission, EC-016-013-007/008 structured-type demotion fix, EC-016-013-009 integer-column string-input coercion fix. S-ADR058-OCSF-ROUTING-001 (Stage 2): ocsf_column_naming flag, Claroty TOML/code corrections KF-01..KF-12. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.6 | adr058-ocsf-routing-propagation | 2026-08-16 | product-owner | **ADR-058 v2.6 path-liveness propagation.** (A) §Architecture Anchors: replaced stale `"audit_activity"`-only `class_selector.rs` reference with full Path A / Path B distinction per ADR-058 §I5 (v2.6) and §K5. Path A (live): `select_by_class_name` in `pipeline_result_to_record_batch` must gain `"entity_management"→3004` and `"inventory_info"→5001` arms + new const `CLASS_UID_ENTITY_MANAGEMENT = 3004`; `"audit_activity"` arm becomes dead code pending deprecation annotation. Path B (zero production callers, forward-compat only): `select()` in `normalize_with_mappers`. Process-gap `ocsf.unknown_class_name` WARN obligation noted (SAP-1 discharged in BC-2.16.002 catalog v1.67). (B) EC-016-013-023: augmented with wire-level postcondition — Claroty audit_logs batch with `ocsf_class = "entity_management"` MUST produce `class_uid = 3004` in the Arrow `class_uid` Int32 column via Path A (NOT 3001 from the prior wrong `account_change` arm, NOT 0 from unknown-class fallback). (C) EC-016-013-024: augmented with wire-level postcondition — Claroty devices batch with `ocsf_class = "inventory_info"` MUST produce `class_uid = 5001` in the Arrow `class_uid` Int32 column via Path A (regression-prevention assertion: without the `"inventory_info"` arm, the KF-02 TOML change regresses class_uid from the current 5001 to 0). TD-VSDD-097 three-dimension sweep: (1) Sibling pair — BC-2.16.003 has no named split-event twin; CLEAR. (2) Downstream copy target — §Architecture Anchors and EC augmentations are not verbatim copy-sources in downstream artifacts; CLEAR. (3) Mandate anchor — wire-level `class_uid` postconditions anchored to S-ADR058-OCSF-ROUTING-001 (the implementing story for both `"entity_management"` and `"inventory_info"` arms); no unanchored MUSTs introduced. |
| 1.5 | adr058-schema-reconcile | 2026-08-16 | product-owner | ADR-058 v2.4 schema validation reconciliation (KF-01..KF-12). (A) §Description: added Interpretation A paragraph — underscore-flattened Arrow field naming under `ocsf_column_naming = true`, Claroty-first scope, reference to §Postconditions §Claroty Contracted OCSF Mappings. (B) §Preconditions: added `ocsf_column_naming` flag precondition for Claroty. (C) §Postconditions §Column Routing: updated to note valid OCSF class requirement (object names not permitted). (D) §Postconditions: added §Interpretation A: Arrow Field Naming subsection (ADR-058 §B2/§I1/§G/§J2 obligations: underscore-flattened names, raw_extensions blob, prism_describe sourcing, fail-closed collision check). (E) §Postconditions: added §Claroty Contracted OCSF Mappings — four-table ground-truth mapping tables encoding corrected ocsf_field values and Arrow names for all Claroty columns; KF-05 PO decision (audit_logs.id → raw_extensions; activity_uid absent from OCSF v1.7.0); KF-06 PO decision (devices.device_type → vendor-extended device.type_label → Arrow device_type_label; OT subcategory demo-critical). (F) §Invariants: added Interpretation A class-correctness and shadow-prevention invariants. (G) §Error Conditions: added reserved-field overwriting row; updated invalid ocsf_class row to include concrete examples (audit_activity, device). (H) EC-016-013-004: corrected exemplar ocsf_field from `finding.uid` to `finding_info.uid` per KF-03. (I) EC-016-013-012: updated queryable name from `device.ip` to `device_ip` (Arrow field name under Interpretation A underscore-flattening; per ADR-058 §I3 obligation). (J) EC-016-013-013..024: twelve new edge cases — KF-08..KF-11 reserved-field removal testability (013..016), KF-03/04/12 finding_info.* path corrections (017..019), KF-05/06 PO decisions with rationale and test expectations (020..021), KF-07 alert_id correction (022), KF-01 entity_management comment validity (023), KF-02 inventory_info device.* resolution (024). (K) §Canonical Test Vectors: added four Interpretation A scenarios. (L) §Related BCs: added BC-2.01.013 reference (EC-01-025 NON-CONFORMANT resolution). (M) §Architecture Anchors: added class_selector.rs, claroty.sensor.toml, spec_driven_adapter.rs references. (N) §Story Anchor: replaced "no story exists" placeholder with concrete S-ADR058-OCSF-COERCION-001 (Stage 1) and S-ADR058-OCSF-ROUTING-001 (Stage 2) anchor references. (O) §Traceability Known-Gap: updated from "no ID" to anchored story IDs. (P) inputs: added ADR-058; input-hash marked stale-needs-refresh (state-manager must run compute-input-hash after this burst). TD-VSDD-097: (1) sibling pair — no twin BC; N/A. (2) downstream copy target — BC-INDEX title column unchanged (H1 title unchanged); S-ADR058-OCSF-ROUTING-001 §AC-005 mapping tables carry pre-v2.4 column listing — story-writer amendment required per ADR-058 §Status v2.4. (3) mandate anchor — no new MUST statements introduced; existing anchors to S-ADR058-OCSF-COERCION-001 and S-ADR058-OCSF-ROUTING-001 carry all obligations. |
| 1.4 | coercion-gap-closure | 2026-08-11 | product-owner | Human-authorized gap closure (CLAUDE.md §Source-of-Truth item 7): expanded coercion matrix section (String-type-first rule LIVE-DRIFT-003), full column_type × JSON-type matrix, EC-016-013-001..012 edge case catalog with IDs, CoercionWarning observability defect flag, KNOWN GAP annotations for structured-type and integer-column gaps, capability anchor justification, Related BCs, Architecture Anchors, Story Anchor, VP Anchors sections added. Two implementing tests (`test_coerce_value_string_type_normalizes_integer_to_string`, `test_coerce_value_string_type_preserves_string_username_against_uid_heuristic`) cited as evidence for EC-016-013-004 and EC-016-013-005. |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
