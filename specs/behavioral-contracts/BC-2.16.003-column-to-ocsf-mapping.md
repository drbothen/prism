---
document_type: behavioral-contract
level: L3
version: "1.16"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: 2026-08-18
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
input-hash: "8d29f53"
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

**Interpretation A — Underscore-flattened Arrow Field Names (ADR-058 §B2/§I1/§G):**
When a sensor spec sets `ocsf_column_naming = true` (ADR-058 §B2), columns are
handled in two tiers based on `ocsf_field`:

- **Tier 1** (`ocsf_field == Some(path)`): the Arrow RecordBatch field name is
  `ocsf_field_to_arrow_name(ocsf_field)` (dots replaced with underscores, per ADR-058
  §I1). Example: `ocsf_field = "finding_info.uid"` produces Arrow field
  `finding_info_uid`. `prism_describe` emits a `ColumnDescriptor` with
  `name = ocsf_field_to_arrow_name(ocsf_field)` and `description = the original
  dotted ocsf_field path` (e.g., `"finding_info.uid"`) per ADR-058 §G.

- **Tier 2** (`ocsf_field == None`): the column is NOT an individual top-level Arrow
  field — its value is aggregated into the single synthesized `raw_extensions`
  (Utf8/JSON blob) Arrow column per ADR-058 §I2 and §B2 item 1. The column's
  `col.name` is NOT a queryable field name; it appears only as a source-key label
  inside the `raw_extensions` ColumnDescriptor's description (see §Postconditions
  §Interpretation A for the full describe model).

This two-tier model makes OCSF-semantic identifiers directly queryable in PrismQL
without quoting (e.g., `SELECT finding_info_uid FROM claroty_alerts`), while
preserving unmapped vendor columns in the `raw_extensions` blob. Claroty is the
first sensor to receive `ocsf_column_naming = true`; all four Claroty tables
(alerts, audit_logs, devices, device_alert_relations) operate under Interpretation A
once Stage 2 (S-ADR058-OCSF-ROUTING-001) ships. The contracted OCSF classes and
column-to-OCSF mappings for all four Claroty tables are specified in §Postconditions
§Claroty Contracted OCSF Mappings.

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

When `sensor_spec.ocsf_column_naming == true` (ADR-058 §B2), columns are processed
in two tiers per ADR-058 §G v2.22:

**Tier 1 — OCSF-mapped columns (`ocsf_field` is `Some(path)`):**

- For each column with `ocsf_field = "a.b.c"`, the Arrow RecordBatch schema field name is `a_b_c` (all dots replaced with underscores; `ocsf_field_to_arrow_name` helper function per ADR-058 §I1).
- `prism_describe` emits a `ColumnDescriptor` with `name = ocsf_field_to_arrow_name(ocsf_field)` (underscore-flattened) and `description = col.ocsf_field` (the original dotted OCSF path, e.g., `"finding_info.uid"` — preserved as semantic annotation for LLM agents per ADR-058 §G).

**Tier 2 — Unmapped columns (`ocsf_field` is `None`):**

- For each column with `ocsf_field == None`: the column is NOT an individual top-level Arrow field. Its value is aggregated into the single synthesized `raw_extensions` (Arrow `Utf8`) JSON blob column per ADR-058 §I2 and §B2 item 1. The column's `col.name` is NOT a queryable field name.
- The `raw_extensions` column is itself an Arrow `Utf8` column containing a serialized JSON object; it is queryable as `SELECT raw_extensions FROM <table>` but nested keys are not independently filterable without JSON path functions.
- `prism_describe` MUST NOT emit an individual `ColumnDescriptor` for any `ocsf_field == None` column. Advertising unmapped columns as individual descriptors creates phantom column names that cause agent query failures (DataFusion "column not found") — exactly the failure mode ADR-058 §C1 exists to prevent (see EC-016-013-027). Instead, `prism_describe` MUST emit exactly ONE `ColumnDescriptor` for the synthesized `raw_extensions` column with the following shape (per ADR-058 §G): `name = "raw_extensions"`, `col_type = prism_core::column::ColumnType::Json` (the physical Arrow column is `Utf8` holding a serialized JSON object; `Json` is the correct semantic variant per the canonical enum: `String / Integer / Float / Boolean / Datetime / Json`), `nullable = true` (two conditions both legitimately yield null: (1) **per-row** — the cell is null when all unmapped source values in that row are null or absent; (2) **per-table** — a table with zero `ocsf_field == None` columns produces no raw_extensions Arrow column at all per ADR-058 §I2/§B2; both conditions make `nullable = true` the correct declaration), `description` = a string that (1) identifies the column as a JSON object and (2) enumerates every source key — the `col.name` of each `ocsf_field == None` column in the queried table. Example for `claroty_alerts` (after KF-08..KF-10 TOML corrections): `"JSON object containing un-mapped source columns: alert_class, ot_devices_count, category, alert_type_name, devices_count"`. This discoverability model is required for correct agent behavior: without the `raw_extensions` descriptor an agent cannot discover the column exists; without the source-key enumeration the agent cannot determine which unmapped fields are accessible inside the blob (ADR-058 §G). **Mandate anchor: S-ADR058-OCSF-ROUTING-001** — story-writer leg adds the AC and Red Gate test (RG-025) asserting (1) `prism_describe` for a Claroty table with `ocsf_column_naming = true` emits NO individual `ColumnDescriptor` for any `ocsf_field == None` column; (2) emits exactly one `raw_extensions` ColumnDescriptor with `name = "raw_extensions"`, `col_type = prism_core::column::ColumnType::Json`, `nullable = true`; and (3) whose description enumerates the `col.name` values of all `ocsf_field == None` columns in that table.

- A fail-closed collision check is enforced at `pipeline_result_to_record_batch` execution time: no flattened `ocsf_field` name may equal the `col.name` of a different column in the same table (ADR-058 §J2); violation returns `Err(ArrowError::SchemaError(...))`, blocking schema construction.

### Claroty Contracted OCSF Mappings

The following tables specify the contracted-correct `ocsf_field` values and Arrow field
names for all four Claroty sensor tables under Interpretation A. These values reflect
ADR-058 §K4 corrections (KF-01 through KF-12). The TOML corrections and
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

**Path key**: **Path A** = `build_column_array` (`spec_driven_adapter.rs`, sole live production path per ADR-058 §A1); **Path B** = `coerce_value` (`column_mapping.rs`, zero live callers, unwired per ADR-058 §A1). Rules 1–3 in the Rule column describe Path B behavior; rows labeled **Path A** below additionally document live production (Path A) behavior.

| declared `column_type` | Input JSON type | OCSF path | Rule | Wire output | Outcome |
|------------------------|-----------------|-----------|------|-------------|---------|
| String | String | any | Rule 1 | String (unchanged) | OCSF field |
| String | Number | any | **Path A: retained wildcard `other.to_string()` (LIVE-DRIFT-003 — CORRECT)**; Path B: Rule 1 (same outcome) | Path A: `"132"` etc. via `other.to_string()` wildcard (sole live path per ADR-058 §A1); Path B: `String(n.to_string())` | OCSF field — CORRECT on both paths (not a defect) |
| String | Bool | any | **Path A: retained wildcard `other.to_string()` (LIVE-DRIFT-003 — CORRECT)**; Path B: Rule 1 (same outcome) | Path A: `"true"`/`"false"` via `other.to_string()` wildcard (sole live path per ADR-058 §A1); Path B: `String(b.to_string())` | OCSF field — CORRECT on both paths (not a defect) |
| String | Null | any | Rule 1 | Null | OCSF field |
| String | Array | any | Rule 1 gap **(Path B / `coerce_value` only)** | Array (WRONG) on Path B; JSON-list String on Path A | OCSF field (Path B — KNOWN GAP, EC-016-013-007); Arrow StringArray cell (Path A — CORRECT, ENRICH-1 preserved, EC-016-013-026) |
| String | Object | any | **Path A: retained wildcard (WRONG pre-AC-005 fix; EC-016-013-008)**; Path B: Rule 1 gap (pass-through) | Path A currently `Some(stringified_object)` (WRONG); contracted: `None` + `column_coercion_failure` warn (AC-005); Path B: Object pass-through (WRONG) | OCSF field — KNOWN GAP (both paths, EC-016-013-008) |
| Integer | Number | any | Rule 3 | Number (unchanged) | OCSF field |
| Integer | String | numeric suffix | Rule 2 | Number(parse) or Err | OCSF field or raw_extensions |
| Integer | String | non-numeric suffix | Rule 3 gap | String (WRONG) | OCSF field — KNOWN GAP |
| Integer | Bool/Null/Array/Object | any | Rule 3 | Unchanged | OCSF field (no coercion) |
| Float | (any) | any | Rule 3 | Unchanged | OCSF field (no float coercion in v1) |
| Boolean | (any) | any | Rule 3 | Unchanged | OCSF field (no bool coercion in v1) |
| Datetime | (any) | any | Rule 3 | Unchanged | OCSF field; datetime parsing is downstream |
| Json | (any) | any | Rule 3 | Unchanged | OCSF field (any JSON is valid) |

**KNOWN GAPs (require a fix story — see §Traceability):**
- EC-016-013-007: `column_type = "string"` + Array input — **Path B (`coerce_value`) only**: currently passes array to OCSF field; MUST divert to `raw_extensions` with `Err(CoercionWarning)` (AC-001). **Path A (`build_column_array`) is NOT a gap:** its dedicated `serde_json::Value::Array(arr)` arm correctly serializes arrays to JSON-list strings per ENRICH-1 Design Decision 2 (EC-016-013-026); null-demotion on Path A applies to `Value::Object` only.
- EC-016-013-008: `column_type = "string"` + Object input — **both Path A and Path B**: same defect class as EC-016-013-007 on Path B; on Path A the `other => Some(other.to_string())` wildcard incorrectly stringifies the object (MUST divert to null cell + warn)
- EC-016-013-009: `column_type = "integer"` + String input on non-numeric OCSF path — Path B (`coerce_value`): currently passes string to OCSF field; MUST parse and divert on failure. See also EC-016-013-025 for Path A (`build_column_array`) Integer+String behavior

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
    column_type = %warning.expected_ocsf_type,
    actual_json_kind = %actual_kind,
    event_type = "column_coercion_failure",
    "coerce_value: type mismatch; field diverted to raw_extensions"
);
```
This `event_type` value MUST be registered in BC-2.16.002 §Postconditions Canonical
Structured Event Catalog per PG-LP11-001.

## OCSF Field Validation
- At spec load time (BC-2.16.009), each `ocsf_field` value is validated against the compiled OCSF protobuf schema; this load-time warning fires on both Path A and Path B for schema-invalid `ocsf_field` values, but does not reject the spec on either path.

**Path A — `build_column_array` (`spec_driven_adapter.rs`; sole live production path per ADR-058 §K5):**

Under `ocsf_column_naming = true` (Interpretation A), Arrow field naming is PURELY MECHANICAL: the Arrow field name is computed from the `ocsf_field` string via dot-to-underscore flattening (`ocsf_field_to_arrow_name`). OCSF schema validity is NEVER consulted on the Arrow-materialization surface.

Routing to `raw_extensions` on Path A is governed SOLELY by `ocsf_field == None` (ADR-058 §I2). A schema-invalid or vendor-extended `Some(path)` value — including `"device.type_label"` (KF-06 / §J3) and `"device.type_category"` (§J3) — STILL materializes as a first-class flattened Arrow column (`device_type_label`, `device_type_category`). It is NOT skipped and does NOT go to `raw_extensions` on Path A.

An implementer MUST NOT wire schema-validity-based `raw_extensions` routing into Path A; doing so would silently divert `device_type_label` and `device_type_category` to `raw_extensions`, breaking the demo-critical `WHERE device_type_label = 'PLC'` filter (EC-016-013-021; S-ADR058-OCSF-ROUTING-001).

**Path B — `ColumnMapper::map_record` (`column_mapping.rs`; zero live production callers per ADR-058 §K5):**

Invalid OCSF field paths produce a warning at load time but do not reject the spec; the mapping is skipped at runtime and the column goes to `raw_extensions`. This is a warning, not an error, because OCSF schema extensions may introduce fields not in the compiled schema.

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
| Warning (non-fatal) | Invalid `ocsf_class` in table spec (class absent from OCSF v1.7.0, e.g. `audit_activity` / `device`) | **Current behavior:** silent `.unwrap_or(0)` fallback to `base_event` (OCSF class 0) — no startup warning, no load-time validation of `ocsf_class` (only `ocsf_field` paths are validated at spec load per BC-2.16.009). **Contracted observability:** runtime `tracing::warn!(event_type = "ocsf.unknown_class_name", ...)` on the `Err` branch of `select_by_class_name` inside `pipeline_result_to_record_batch` (ADR-058 §I5; S-ADR058-OCSF-ROUTING-001 AC-011/RG-018 per EC-016-013-024). |
| KNOWN GAP | `column_type = "string"` + **Object** input (both paths); **Array** input (Path B / `coerce_value` only) | Path B: `coerce_value` passes Array or Object to OCSF field instead of diverting to `raw_extensions` (EC-016-013-007 for Array, EC-016-013-008 for Object). Path A: `build_column_array` passes Object via `other.to_string()` wildcard (EC-016-013-008); Array is correctly handled by the dedicated ENRICH-1 arm — JSON-list string, not a gap (EC-016-013-026). |
| KNOWN GAP | `column_type = "integer"` + String input on non-numeric OCSF path — Path B (`coerce_value`) passes string to OCSF field; Path A (`build_column_array`) silently returns null for all String inputs, losing valid numeric strings | EC-016-013-009 (Path B fix: AC-003); EC-016-013-025 (Path A fix: AC-007) |
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
| EC-016-013-007 | `column_type = "string"`, `Value::Array([...])` input — **Path B (`coerce_value`) only** | KNOWN GAP on Path B: `coerce_value` pass-through; array placed in OCSF string field (incorrect — MUST divert to `raw_extensions` with `Err(CoercionWarning)`). Fix: AC-001 (S-ADR058-OCSF-COERCION-001). **Path A (`build_column_array`) is NOT affected:** its dedicated `serde_json::Value::Array(arr)` arm correctly serializes all arrays to a compact JSON-list string per ENRICH-1 Design Decision 2 — this is CORRECT behavior (EC-016-013-026). Null-demotion on Path A applies to `Value::Object` only (EC-016-013-008 / AC-005). |
| EC-016-013-008 | `column_type = "string"`, `Value::Object({...})` input | KNOWN GAP: same defect class as EC-016-013-007. Fix: S-ADR058-OCSF-COERCION-001 |
| EC-016-013-009 | `column_type = "integer"`, `Value::String("42")`, OCSF path `"device.bytes"` (non-numeric suffix) — **Path B (`coerce_value`) only** | KNOWN GAP on Path B: Rule 3 pass-through; string placed in integer-typed OCSF field (incorrect — MUST parse and divert on failure). Fix: S-ADR058-OCSF-COERCION-001 AC-003. See also EC-016-013-025 for the corresponding Path A (`build_column_array`) gap where `other.as_i64()` silently returns null for all String inputs. |
| EC-016-013-010 | `column_type = "boolean"`, `Value::Bool(true)` | Rule 3 pass-through; bool placed in OCSF field unchanged |
| EC-016-013-011 | Invalid `ocsf_class` (e.g., `"made_up_class"` or OCSF object name like `"device"` or absent-class like `"audit_activity"`) | **Current behavior:** silent `.unwrap_or(0)` fallback — records use `base_event` (OCSF class 0), NO load-time warning (no load-time validation of `ocsf_class` exists; only `ocsf_field` paths are validated at spec load per BC-2.16.009). **Contracted observability:** runtime `tracing::warn!(event_type = "ocsf.unknown_class_name", ...)` on the `select_by_class_name` `Err` branch inside `pipeline_result_to_record_batch` (ADR-058 §I5; S-ADR058-OCSF-ROUTING-001 AC-011/RG-018). Consistent with EC-016-013-024: the silent fallback path persists until the contracted WARN is implemented. |
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
| EC-016-013-023 | KF-01 class correction: `entity_management` (3004) `comment` attribute is accessible; `audit_logs.note → comment` mapping produces data; **wire-level: `class_uid = 3004` in Arrow `class_uid` Int32 column (Path A)** | Under `account_change` (3001, the prior wrong class), `set_nested_field` silently no-ops the `note → comment` mapping because `account_change` has no `comment` attr in its protobuf descriptor — the value is silently dropped; under `entity_management` (3004), `comment` is a valid class-level attr and the mapping resolves; test: load `entity_management` record with `note` value populated; assert Arrow field `comment` carries the note value (not null, not missing). **Wire-level postcondition (ADR-058 §I5 wire-shape assertion obligation):** a `RecordBatch` materialized from Claroty `audit_logs` data with `ocsf_class = "entity_management"` via `pipeline_result_to_record_batch` (Path A) MUST carry `class_uid = 3004` in the Arrow `class_uid` Int32 column — NOT 3001 (prior wrong `account_change` arm value) and NOT 0 (`.unwrap_or(0)` BASE_EVENT fallback). Wire-level test: assert the serialized `class_uid` column value equals `3004` at the `RecordBatch` level (not only at the resolver unit-test level — per wire-shape assertion discipline). Anchored: S-ADR058-OCSF-ROUTING-001. |
| EC-016-013-025 | `build_column_array` (Path A — sole live production path per ADR-058 §K5) ColumnType::Integer + `Value::String` input on any OCSF path | Parse succeeds (e.g. `"42"`): `Some(42)` returned — string-encoded integer materialized as an integer value in the Arrow Int64 column; no data loss. Parse fails (e.g. `"not-a-number"`): `None` returned (null cell in Arrow column) + `tracing::warn!(event_type = "column_coercion_failure", column = %col.name, column_type = "integer", actual_json_kind = "string")` emitted. **Current behavior (pre-fix):** `other.as_i64()` in the `ColumnType::Integer` arm returns `None` for ALL String inputs — silently drops valid numeric strings (e.g. `"42"` → null, data loss) and drops non-parseable strings without a warning. Path A is the sole live production path; Path B (`coerce_value`) fix covered by EC-016-013-009 / AC-003. Fix: S-ADR058-OCSF-COERCION-001 AC-007 (RG-008, RG-009). |
| EC-016-013-026 | `build_column_array` (Path A — sole live production path per ADR-058 §K5) ColumnType::String + `Value::Array` input (any array, including ENRICH-1 wildcard source_path results) | **CORRECT behavior — no fix required.** The dedicated `serde_json::Value::Array(arr)` arm at the String branch serializes all arrays to a compact JSON-list string (e.g., `["192.168.1.1","10.0.0.1"]`). Integer/bool array elements are stringified via `other.to_string()`. Empty array → `"[]"` (empty JSON-list string, NOT null — consistent with `map_record`'s documented design decision "Empty array → `"[]"` (not null)"). This arm fires for ALL `Value::Array` inputs regardless of whether `col.source_path` is set; it is the sole implementation of ENRICH-1 Design Decision 2 on Path A. ENRICH-1 wildcard columns (`ip_list`, `mac_list`, `network_list`, `vlan_list` on Claroty `devices`) rely on this arm exclusively. **The `Value::Array` arm MUST NOT be changed to null-demotion** — doing so regresses all ENRICH-1 wildcard-array columns. Null-demotion on Path A applies ONLY to `Value::Object` (EC-016-013-008 / AC-005). Tests: `test_build_column_array_claroty_ip_list_string_elements_serialize_to_json_list_string`, `test_build_column_array_claroty_vlan_list_integer_elements_stringify_to_json_list_string` in `crates/prism-bin/src/spec_driven_adapter.rs` §tests. |
| EC-016-013-027 | `prism_describe` under Interpretation A (`ocsf_column_naming = true`) — phantom individual descriptor prohibition and `raw_extensions` descriptor requirement | `prism_describe` MUST NOT emit individual `ColumnDescriptors` for `ocsf_field == None` columns; doing so creates phantom column names that cause DataFusion "column not found" failures when an agent copies the name into a query (ADR-058 §C1/§G). Instead, `prism_describe` MUST emit exactly ONE `raw_extensions` ColumnDescriptor with `name = "raw_extensions"`, `col_type = prism_core::column::ColumnType::Json`, `nullable = true`, and `description` enumerating the `col.name` of every unmapped column in the table (per ADR-058 §G). Test assertion: for `claroty_alerts` with `ocsf_column_naming = true`, assert the `prism_describe` response contains no ColumnDescriptor named `alert_class` or `ot_devices_count`; assert exactly one descriptor named `raw_extensions` with `col_type = prism_core::column::ColumnType::Json` and `nullable = true` (per ADR-058 §G); assert its description includes the strings `"alert_class"` and `"ot_devices_count"`. **POL-38 obligation: story-writer leg of S-ADR058-OCSF-ROUTING-001 MUST add AC + Red Gate test (RG-025) covering the three MUST assertions in §Postconditions §Interpretation A Tier-2 describe model (phantom-descriptor prohibition, col_type=Json/nullable=true ColumnDescriptor shape, and description source-key enumeration).** |
| EC-016-013-024 | KF-02 class correction: `inventory_info` (5001) `device.*` attribute paths resolve via the `device` required attribute; **wire-level: `class_uid = 5001` in Arrow `class_uid` Int32 column (Path A) — regression-prevention** | `devices` table columns `uid`, `asset_id`, `device_category`, `device_type`, `device_name`, `os_category` all resolve via `inventory_info.device.*` path hierarchy; class-level `risk_score` and `status_code` attrs confirmed at `inventory_info` class level directly (not nested under `device`); test: assert Arrow schema for `claroty_devices` under Interpretation A includes `device_uid`, `device_instance_uid`, `risk_score`, `status_code` as first-class columns. **Wire-level postcondition — regression-prevention (ADR-058 §I5 wire-shape assertion obligation):** a `RecordBatch` materialized from Claroty `devices` data with `ocsf_class = "inventory_info"` via `pipeline_result_to_record_batch` (Path A) MUST carry `class_uid = 5001` in the Arrow `class_uid` Int32 column — NOT 0 (BASE_EVENT). This is an explicit regression-prevention assertion: without the `"inventory_info" => Ok(CLASS_UID_DEVICE_INVENTORY_INFO)` arm in `select_by_class_name`, the KF-02 TOML change from `ocsf_class = "device"` to `ocsf_class = "inventory_info"` regresses `class_uid` from the current 5001 (produced by the existing `"device"` arm) to 0 (`.unwrap_or(0)` BASE_EVENT fallback). The regression path is silent without the `ocsf.unknown_class_name` WARN (BC-2.16.002 §Canonical Structured Event Catalog). Wire-level test: assert the serialized `class_uid` column value equals `5001` at the `RecordBatch` level. Anchored: S-ADR058-OCSF-ROUTING-001. |
| EC-016-013-028 | ENRICH-1 wildcard source_path columns (`ocsf_field == None` with `source_path = "$.X[*]"`) in `raw_extensions` value representation — Claroty `devices` columns `ip_list`, `mac_list`, `network_list`, `vlan_list` | The value stored under `col.name` in the `raw_extensions` JSON object MUST be the **ENRICH-1-normalized value**: the compact JSON-list string produced by `build_column_array`'s dedicated `serde_json::Value::Array` arm for `column_type = "string"` input (per EC-016-013-026 and ADR-058 v2.22 §B2/§I2) — e.g., `["192.168.1.1","10.0.0.1"]` serialized as the string `"[\"192.168.1.1\",\"10.0.0.1\"]"`. NOT a raw JSON array value from the API response and NOT the top-level JSON structure before ENRICH-1 wildcard processing. The ENRICH-1 pipeline runs before `pipeline_result_to_record_batch` and produces a `Value::Array` from wildcard-collected values; `build_column_array`'s String+Array arm then serializes that array to a JSON-list string; the resulting string is stored as the value under `col.name` in `raw_extensions`. Wire-shape assertion: `SELECT raw_extensions FROM claroty_devices` for a row with non-empty `ip_list` MUST yield a `raw_extensions` JSON object containing an `"ip_list"` key whose value is a JSON-encoded compact array string — not a nested JSON array, not null. **POL-38 obligation: S-ADR058-OCSF-ROUTING-001 story-writer leg MUST add an AC and wire-shape Red Gate test asserting this representation (parse `raw_extensions` JSON for a Claroty devices row, access the `"ip_list"` key, assert its Go/Rust-level type is a string whose content parses as a JSON array — not a raw array type).** |

## Canonical Test Vectors

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — full mapping | all columns have `ocsf_field`; all types match | OcsfEvent with all fields mapped; `raw_extensions` empty |
| Mixed mapping | some columns have `ocsf_field`, some don't | Mapped columns in OCSF proto; unmapped in `raw_extensions` |
| Coercion failure — non-parseable string | `"not-a-number"` for integer field on numeric-suffix path | Field in `raw_extensions`; `CoercionWarning` emitted; record included |
| Integer JSON on String column | `Value::Number(132)` on `finding_info.uid`, `column_type = "string"` | Wire output `Value::String("132")`; test: `test_coerce_value_string_type_normalizes_integer_to_string` |
| String username on uid path | `Value::String("analyst")` on `actor.user.uid`, `column_type = "string"` | String preserved in OCSF field; no CoercionWarning; test: `test_coerce_value_string_type_preserves_string_username_against_uid_heuristic` |
| Invalid ocsf_class | table has unknown `ocsf_class` or OCSF object name | base_event (class 0) used; **current: silent fallback, no load-time warning**; contracted: runtime `ocsf.unknown_class_name` WARN (ADR-058 §I5; S-ADR058-OCSF-ROUTING-001 AC-011/RG-018) |
| Interpretation A — finding_info.uid routing | Claroty `alerts.id` = `"132"`, `ocsf_column_naming = true` | Arrow field `finding_info_uid` = `"132"`; no `finding_uid` column in schema (old wrong name absent) |
| Interpretation A — reserved field removed | Claroty `alerts.category` = `"OT Threat"`, `ocsf_column_naming = true` | `category` value in `raw_extensions` blob; no first-class Arrow column carries vendor category string as `class_name` |
| Interpretation A — entity_management comment | Claroty `audit_logs.note` = `"reviewed"`, `ocsf_column_naming = true` | Arrow field `comment` = `"reviewed"` (entity_management 3004 class has comment attr) |
| Interpretation A — OT device subcategory | Claroty `devices.device_type` = `"PLC"`, `ocsf_column_naming = true` | Arrow field `device_type_label` = `"PLC"`; queryable as `SELECT * FROM claroty_devices WHERE device_type_label = 'PLC'` |
| Interpretation A — `prism_describe` Tier-2 raw_extensions descriptor | `claroty_alerts` table with `ocsf_column_naming = true`; `alert_class` and `ot_devices_count` have `ocsf_field == None` | `prism_describe` response: NO individual `ColumnDescriptor` named `alert_class` or `ot_devices_count`; exactly ONE `ColumnDescriptor` named `raw_extensions` with `col_type = prism_core::column::ColumnType::Json`, `nullable = true`, and description containing `"alert_class"` and `"ot_devices_count"` as enumerated source keys (EC-016-013-027; ADR-058 §G; mandate anchor S-ADR058-OCSF-ROUTING-001 RG-025) |

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
- `crates/prism-ocsf/src/class_selector.rs` — **Path A (live production):** `select_by_class_name` is the resolver called by `pipeline_result_to_record_batch` on the spec-driven Arrow materialization path (Path A). It must gain two new arms: `"entity_management" => Ok(CLASS_UID_ENTITY_MANAGEMENT)` (3004) and `"inventory_info" => Ok(CLASS_UID_DEVICE_INVENTORY_INFO)` (5001). New const `CLASS_UID_ENTITY_MANAGEMENT = 3004` required. Without the `"entity_management"` arm, the KF-01 TOML correction (`ocsf_class = "audit_activity"` → `"entity_management"`) falls to `.unwrap_or(0)`, producing `class_uid = 0` (BASE_EVENT — a regression from the current wrong 3001). Without the `"inventory_info"` arm, the KF-02 TOML correction (`ocsf_class = "device"` → `"inventory_info"`) falls to `.unwrap_or(0)`, regressing `class_uid` from the current 5001 to 0. The `"audit_activity"` arm becomes dead code after the KF-01 TOML fix (no production TOML will declare `ocsf_class = "audit_activity"` post-correction) and MUST carry a deprecation annotation as a transitional entry pending removal. **Path B (forward-compat, zero production callers):** `select()` called from `normalize_with_mappers` (`crates/prism-ocsf/src/normalizer.rs`) — zero live query traffic today; Path B Claroty and Armis `audit_log` arms also require `Ok(CLASS_UID_ENTITY_MANAGEMENT)` correction for forward-compat when Path B is eventually wired (ADR-058 §I5 §K5). **Process-gap obligation (ADR-058 §I5):** `pipeline_result_to_record_batch` must emit `tracing::warn!(event_type = "ocsf.unknown_class_name", ...)` on the `Err` branch before `.unwrap_or(0)` (SOUL.md #4; SAP-1 obligation discharged in BC-2.16.002 §Canonical Structured Event Catalog). Armis `("armis", "audit_log")` arm in Path B requires same `CLASS_UID_ENTITY_MANAGEMENT` correction (TD-VSDD-097 sibling sweep; KF-01 code obligation per ADR-058 §I5/§K5 Divergence 3)
- `crates/prism-sensors/specs/claroty.sensor.toml` — Claroty TOML spec receiving KF-01..KF-12 corrections per S-ADR058-OCSF-ROUTING-001 AC-005 / §I5
- `crates/prism-bin/src/spec_driven_adapter.rs` — `pipeline_result_to_record_batch`; Stage 2 wiring adds `ocsf_field_to_arrow_name` dispatch per ADR-058 §I1

## Story Anchor

**Stage 1 — S-ADR058-OCSF-COERCION-001 (status: draft):** Implements:
1. Fix EC-016-013-008: `column_type = "string"` + Object input → `None` (null cell) + `tracing::warn!(event_type = "column_coercion_failure")` emission (ADR-058 §H item 1) — AC-001, AC-002, AC-005
2. Fix EC-016-013-009 (Path B, `coerce_value`): `column_type = "integer"` + String input on non-numeric OCSF path → `ColumnMapper::coerce_value` parses `s.parse::<i64>()` on ALL Integer+String combinations (not only numeric-suffix paths) or diverts to `raw_extensions` on failure (ADR-058 §H item 2) — AC-003
2b. Fix EC-016-013-025 (Path A, `build_column_array`): `ColumnType::Integer` arm + `Value::String` → parse `s.parse::<i64>()`: success returns `Some(n)` (no silent data loss); failure returns `None` + `column_coercion_failure` warn (ADR-058 §H item 2 "or dispatching through it") — AC-007
3. Fix EC-016-013-007 (Path B, `coerce_value` only): `column_type = "string"` + Array input → `Err(CoercionWarning)` → `raw_extensions` (AC-001). Path A (`build_column_array`) is NOT affected — the dedicated ENRICH-1 `Value::Array` arm (EC-016-013-026) correctly serializes all arrays to JSON-list strings; no Path A change required for Array input.
4. Add `column_coercion_failure` tracing emission in `ColumnMapper::map_record` at demotion time (ADR-058 §H item 3) — AC-004
5. Register `column_coercion_failure` in BC-2.16.002 §Postconditions Canonical Structured Event Catalog (SAP-1 / PG-LP11-001 obligation) — AC-004

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
| Known-Gap Story Needed | ANCHORED — S-ADR058-OCSF-COERCION-001 (Stage 1): CoercionWarning tracing emission (AC-004), EC-016-013-007/008 structured-type demotion fix (AC-001/AC-002/AC-005), EC-016-013-009 Path B integer-column coerce_value fix (AC-003), EC-016-013-025 Path A build_column_array integer-column String parse fix (AC-007 / RG-008 / RG-009 — added per F-P16-MED-003 adjudication). S-ADR058-OCSF-ROUTING-001 (Stage 2): ocsf_column_naming flag, Claroty TOML/code corrections KF-01..KF-12, EC-016-013-027 `prism_describe` Tier-2 raw_extensions descriptor (three MUST assertions in §Postconditions §Interpretation A: phantom-descriptor prohibition, col_type=Json/nullable=true ColumnDescriptor shape, description source-key enumeration — story-writer leg adds AC + Red Gate RG-025, POL-38 obligation). EC-016-013-028 ENRICH-1 source_path columns in `raw_extensions` value representation (ADR-058 v2.22 §B2/§I2): story-writer leg of S-ADR058-OCSF-ROUTING-001 MUST add AC + wire-shape Red Gate test asserting compact JSON-list string in `raw_extensions["ip_list"]` (POL-38 obligation). |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.16 | ocsf-correctness-f-p48-p46-coordinated-burst | 2026-08-18 | product-owner | F-P46-MED-001: re-pin ADR-058 §G version from v2.20 to v2.22 in §Postconditions §Interpretation A intro clause. F-P48-MED-001: rewrote §Error Conditions "Invalid `ocsf_class`" row and EC-016-013-011 to eliminate the false "startup warning at spec load time" mechanism — actual behavior is silent `.unwrap_or(0)` fallback to base_event (class 0) with no load-time validation; contracted observability is a runtime `tracing::warn!(event_type = "ocsf.unknown_class_name", ...)` on the `Err` branch of `select_by_class_name` inside `pipeline_result_to_record_batch` (ADR-058 §I5; S-ADR058-OCSF-ROUTING-001 AC-011/RG-018); consistent with EC-016-013-024 which correctly stated "the regression path is silent without the `ocsf.unknown_class_name` WARN". Also fixed §Canonical Test Vectors "Invalid ocsf_class" row for consistency. F-P48-MED-002: added EC-016-013-028 specifying that ENRICH-1 source_path columns (`ip_list`, `mac_list`, `network_list`, `vlan_list` on Claroty `devices`) with `ocsf_field == None`, when aggregated into `raw_extensions`, MUST store the ENRICH-1-normalized compact JSON-list string value per ADR-058 v2.22 §B2/§I2 and EC-016-013-026 — not the raw top-level JSON value; POL-38 obligation flagged for story-writer leg. F-P48-OBS-2: harmonized `nullable = true` rationale in §Postconditions §Interpretation A Tier-2 prose to acknowledge both conditions: (1) per-row — cell is null when all unmapped source values in that row are null/absent; (2) per-table — a table with zero `ocsf_field == None` columns produces no raw_extensions Arrow column. TD-VSDD-097: (1) sibling pair — BC-2.16.003 has no named split-event twin; CLEAR. (2) downstream copy targets — EC-016-013-028 POL-38 flagged for ROUTING-001 story-writer (AC + wire-shape Red Gate test); `nullable = true` rationale update flagged for ROUTING-001 RG-025; §Error Conditions/EC-011 text must be consistent with ROUTING-001 AC-011/RG-018. (3) mandate anchor — EC-016-013-028 new MUST anchored to S-ADR058-OCSF-ROUTING-001 via POL-38 obligation; no unanchored MUSTs introduced. |
| 1.15 | spec-f-p44-obs-001-raw-extensions-descriptor-completeness | 2026-08-18 | product-owner | F-P44-OBS-001 closure: added `col_type = prism_core::column::ColumnType::Json` and `nullable = true` to the `raw_extensions` ColumnDescriptor shape contract in §Postconditions §Interpretation A Tier-2, EC-016-013-027, §Canonical Test Vectors, and §Traceability, verbatim per ADR-058 §G. (A) §Postconditions §Interpretation A Tier-2: updated "MUST emit exactly ONE ColumnDescriptor" to list the full four-field shape (name + col_type + nullable + description); mandate anchor updated to three-assertion list adding col_type/nullable assertion; added RG-025 label for story-writer leg. (B) EC-016-013-027: updated test assertion to assert `col_type = prism_core::column::ColumnType::Json` and `nullable = true`; POL-38 obligation updated to reference RG-025 and three assertions. (C) §Canonical Test Vectors Interpretation A Tier-2 describe row: added `col_type`/`nullable` to expected output. (D) §Traceability Known-Gap: updated "two MUST assertions" → "three MUST assertions (phantom-descriptor prohibition, col_type=Json/nullable=true shape, description source-key enumeration)". TD-VSDD-097: (1) sibling pair — BC-2.16.002 (cascade sibling); its `schema_enumeration.*` catalog rows do not reference `raw_extensions` ColumnDescriptor col_type/nullable; no change required; CLEAR. (2) downstream copy target — ROUTING-001 RG-025 must assert col_type/nullable; flagged for story-writer leg per POL-38; do not edit here. (3) mandate anchor — Tier-2 describe MUSTs (name + col_type + nullable + description) anchored to S-ADR058-OCSF-ROUTING-001 RG-025; CLEAR. |
| 1.14 | spec-bc-describe-align-f40-f42-high-001 | 2026-08-18 | product-owner | F-P40/P42-HIGH-001 §Interpretation A Tier-1/Tier-2 describe alignment to ADR-058 §G v2.20. (A) §Description: replaced contradictory phantom-column model ("Columns without `ocsf_field` retain `col.name` as their Arrow field name") with the correct two-tier model — Tier-1 Arrow field = `ocsf_field_to_arrow_name(ocsf_field)`; Tier-2 columns are NOT individual Arrow fields, values aggregate into `raw_extensions` blob. (B) §Postconditions §Interpretation A: rewrote with explicit Tier-1/Tier-2 headers; removed `col.name` fallback language from Tier-2; added ADR-058 §G v2.20 two-tier `prism_describe` model — Tier-1 emits underscore-flattened ColumnDescriptor with dotted-path description; Tier-2 emits NO individual descriptor for unmapped columns, emits exactly ONE `raw_extensions` ColumnDescriptor whose description enumerates all `col.name` source keys; two new MUSTs anchored to S-ADR058-OCSF-ROUTING-001 (story-writer leg adds AC + Red Gate). (C) Added EC-016-013-027: `prism_describe` Tier-2 phantom-descriptor prohibition + raw_extensions descriptor requirement; POL-38 obligation flagged for story-writer. (D) Added §Canonical Test Vectors row for Interpretation A describe Tier-2 behavior. (E) §Traceability Known-Gap updated with EC-016-013-027 / S-ADR058-OCSF-ROUTING-001 AC+RG obligation. TD-VSDD-097: (1) sibling pair — BC-2.16.002 (cascade sibling) references `prism_describe` only via audit events (schema_enumeration.*), not §Interpretation A column-naming; CLEAR. (2) downstream copy targets — ROUTING-001 §G/AC-006/AC-007 copy this describe model; flagged for story-writer leg; do not edit here. (3) mandate anchor — Tier-2 `prism_describe` MUSTs anchored to S-ADR058-OCSF-ROUTING-001 (story-writer leg adds AC + Red Gate); CLEAR. |
| 1.13 | ocsf-correctness-pass30-f1-field-validation | 2026-08-17 | product-owner | F-P30-MED-001 §OCSF Field Validation Path-A/Path-B qualifier: Path A (`build_column_array`) naming is purely mechanical; raw_extensions governed by `ocsf_field == None` only (ADR-058 §I2); vendor-extended `Some(path)` values (`device.type_label`, `device.type_category`) stay first-class Arrow columns on Path A. Implementer MUST NOT wire schema-validity routing into Path A — EC-016-013-021 demo-critical filter at risk (S-ADR058-OCSF-ROUTING-001). Skipped→raw_extensions is Path B (`ColumnMapper::map_record`) only. No new ECs added (POL-38: CLEAR). TD-VSDD-097: (1) no split-event twin BC; CLEAR. (2) §OCSF Field Validation not verbatim copy-source in downstream artifacts; CLEAR. (3) new MUST anchored to EC-016-013-021 and S-ADR058-OCSF-ROUTING-001; CLEAR. |
| 1.12 | ocsf-coercion-pass25-f1-path-a-string-completeness | 2026-08-17 | product-owner | F-P25-MED-001 §Full Coercion Matrix Path-A String-arm completeness: added Path-key preamble (Path A = `build_column_array`, sole live path per ADR-058 §A1; Path B = `coerce_value`, unwired); Number/Bool rows now document Path-A retained-wildcard `other.to_string()` as CORRECT behavior (LIVE-DRIFT-003 — not a defect); Object row now distinguishes Path-A current behavior (`Some(stringified_object)`, WRONG pre-AC-005) from contracted target (`None` + `column_coercion_failure` warn, EC-016-013-008 / AC-005) and Path-B pass-through (WRONG). Array→JSON-list immutable (EC-016-013-026) unchanged. No new ECs added. No AC obligations changed beyond what AC-005 already covers (POL-38: CLEAR). TD-VSDD-097 three-dimension sweep: (1) Sibling pair — BC-2.16.003 has no named split-event twin; CLEAR. (2) Downstream copy target — §Full Coercion Matrix Number/Bool rows are not verbatim copy-sources in downstream artifacts; story-writer handles S-ADR058-OCSF-COERCION-001 AC-005/T-15 wording update per task handoff. (3) Mandate anchor — no new unanchored MUSTs introduced; all existing mandates anchored to AC-005 unchanged; CLEAR. |
| 1.11 | ocsf-coercion-pass24-f1-enrich1-path-scope | 2026-08-17 | product-owner | **F-P24-HIGH-001 adjudication: scope EC-016-013-007 Array-null-demotion to Path B only; add EC-016-013-026 protecting ENRICH-1 `Value::Array` arm on Path A.** (A) EC-016-013-007 §Edge Cases row: added "Path B (`coerce_value`) only" scope qualifier; documented that Path A `build_column_array` dedicated `Value::Array` arm is correct ENRICH-1 behavior, cross-references EC-016-013-026. (B) §Full Coercion Matrix: narrowed "String \| Array \| any" row to Path B / `coerce_value` only; added Path A behavior column showing JSON-list String (ENRICH-1 preserved, EC-016-013-026). §KNOWN GAPs bullets: EC-016-013-007 scoped to Path B; EC-016-013-008 clarified as both paths. (C) §Error Conditions KNOWN GAP row: scoped Array part to Path B; added Path A ENRICH-1 arm note; retained Object as applying to both paths. (D) Added EC-016-013-026: documents that `build_column_array` Path A `serde_json::Value::Array(arr)` arm is CORRECT behavior (not a gap) — serializes all arrays including ENRICH-1 wildcard source_path results to JSON-list string; empty array → `"[]"`; null-demotion on Path A applies to `Value::Object` only; MUST NOT be changed to null-demotion. Tests: two existing MEDIUM-6 Claroty ip_list/vlan_list tests confirm production behavior. (E) §Story Anchor item 3: removed incorrect "AC-005" citation for the Array fix — AC-005 covers Object only on Path A; Array null-demotion on Path A is WRONG per adjudication. (F) input-hash updated from 08bcb32 to f7d5e31 (pre-existing drift from ADR-058 input changes). TD-VSDD-097 three-dimension sweep: (1) Sibling pair — BC-2.16.003 has no named split-event twin; CLEAR. (2) Downstream copy target — story S-ADR058-OCSF-COERCION-001 AC-005/RG-007/EC-001/T-15 carry pre-adjudication Array null-demotion intent; story-writer must apply adjudication rewrite spec per handoff note in product-owner output. (3) Mandate anchor — EC-016-013-026 `MUST NOT` anchored to ENRICH-1 Design Decision 2 established behavior; no new unanchored story-level obligations. |
| 1.10 | ocsf-coercion-pass16-f3-path-a-integer-gap | 2026-08-17 | product-owner | **F-P16-MED-003 closure (Option A): add EC-016-013-025 for `build_column_array` Path A Integer+String gap.** ADR-058 §H item 2 explicitly states the fix for EC-016-013-009 applies to `build_column_array` ("or dispatching through it"); the story's own story-level EC-009 described the Path A behavior change but had no BC EC backing it. (A) EC-016-013-009 §Edge Cases row: added "Path B (`coerce_value`) only" qualifier and cross-reference to EC-016-013-025. (B) EC-016-013-009 §Full Coercion Matrix KNOWN GAPs bullet: added Path B qualifier and cross-ref to EC-016-013-025. (C) EC-016-013-009 §Error Conditions row: added Path A complement note. (D) Added EC-016-013-025: `build_column_array` (Path A, sole live production path per ADR-058 §K5) ColumnType::Integer + `Value::String` — parse succeeds → `Some(n)` in Arrow column; parse fails → `None` (null cell) + `column_coercion_failure` warn. Current behavior: `other.as_i64()` silently returns `None` for all String inputs (valid numeric strings silently lost). Fix: S-ADR058-OCSF-COERCION-001 AC-007 (RG-008, RG-009). (E) §Story Anchor Stage 1 item 2: split into Path B (AC-003) and added item 2b for EC-016-013-025 Path A fix (AC-007). (F) §Traceability Known-Gap Story Needed: added EC-016-013-025 / AC-007. TD-VSDD-097 three-dimension sweep: (1) Sibling pair — BC-2.16.003 has no named split-event twin; CLEAR. (2) Downstream copy target — EC-016-013-025 is new content; no prior version to propagate; CLEAR. (3) Mandate anchor — new EC-016-013-025 MUST anchored to S-ADR058-OCSF-COERCION-001 AC-007 RG-008/RG-009; story-writer must add AC-007 / RG-008 / RG-009 per adjudication output. |
| 1.9 | adr058-ocsf-pass5-f1-coercion-value-sources | 2026-08-17 | product-owner | **Adversary pass-5 F1 [MED] closure: corrected `column_coercion_failure` tracing snippet value-source expressions to match real `CoercionWarning` struct fields and S-ADR058-OCSF-COERCION-001 AC-004 exactly.** Pass-4 fix aligned the emitted field KEYS (`column`, `column_type`, `actual_json_kind`) but broke the VALUE expressions — `%warning.column_type` and `%warning.actual_json_kind` reference non-existent `CoercionWarning` fields; the struct has `{column_name, expected_ocsf_type, actual_value}`. Fix: `column_type = %warning.expected_ocsf_type` (KEY stays `column_type`; value reads the real struct field `expected_ocsf_type`); `actual_json_kind = %actual_kind` (KEY stays `actual_json_kind`; value is a computed local `actual_kind` — NOT a struct field; per AD-017 spirit, log the JSON kind, not raw data). `column = %warning.column_name` unchanged. Value expressions now match AC-004 verbatim. TD-VSDD-097 three-dimension sweep: (1) Sibling pair — none; CLEAR. (2) Downstream copy target — BC-2.16.002 `column_coercion_failure` catalog row does not yet exist; no stale copy; CLEAR. (3) Mandate anchor — existing MUST anchored to S-ADR058-OCSF-COERCION-001; no new unanchored MUSTs. |
| 1.8 | adr058-ocsf-pass4-f1-coercion-field-schema | 2026-08-17 | product-owner | **Adversary pass-4 F1 [MED] closure: aligned §Coercion Warning Observability tracing field schema to story S-ADR058-OCSF-COERCION-001 AC-004 / catalog-row obligation.** Prior field keys `expected_ocsf_type` / `actual_value` contradicted the story schema `{column, column_type, actual_json_kind}`. Both name the same SAP-1-governed event `column_coercion_failure`, so a permanent catalog contradiction would have arisen when BC-2.16.002 received the catalog row. Resolution: updated the tracing macro snippet to use the story-canonical keys `column_type` (was `expected_ocsf_type`) and `actual_json_kind` (was `actual_value`). `actual_json_kind` logs the JSON kind/type rather than raw field data — safer under the credential/data-opacity posture (AD-017 spirit; do not log raw values). No behavioral change; no new MUSTs introduced. `column = %warning.column_name` key unchanged. TD-VSDD-097 three-dimension sweep: (1) Sibling pair — no named split-event twin; CLEAR. (2) Downstream copy target — BC-2.16.002 §Canonical Structured Event Catalog does not yet contain the `column_coercion_failure` row (it is gated on S-ADR058-OCSF-COERCION-001); no stale copy to propagate; CLEAR. (3) Mandate anchor — the existing MUST anchoring this emission to S-ADR058-OCSF-COERCION-001 is unchanged; no unanchored MUSTs introduced. |
| 1.7 | adr058-ocsf-pass2-f7-volatile-pin-sweep | 2026-08-16 | product-owner | **Adversary pass-2 F7 [LOW] closure: POL-39 volatile-version-pin sweep.** Removed all `ADR-058 v2.x` and `(v1.x)` self-label pins from narrative prose (frontmatter `version:` and §Changelog rows exempt per POL-39). Changes: (A) §Description `(ADR-058 v2.0)` → `(ADR-058 §B2/§I1)`. (B) §Claroty Contracted OCSF Mappings section header: removed `(v1.5)` label — version-free to avoid re-pinning drift. (C) §Claroty Contracted OCSF Mappings intro: `ADR-058 v2.4 §K4` → `ADR-058 §K4`. (D) EC-016-013-023 and EC-016-013-024 wire-level postconditions: `ADR-058 §I5 v2.6 wire-shape assertion obligation` → `ADR-058 §I5 wire-shape assertion obligation`. (E) EC-016-013-023 and EC-016-013-024: `BC-2.16.002 §Canonical Structured Event Catalog v1.67 row` → `BC-2.16.002 §Canonical Structured Event Catalog`. (F) §Architecture Anchors: `ADR-058 §I5 v2.6 §K5` → `ADR-058 §I5 §K5`; `ADR-058 §I5 v2.6` (two occurrences) → `ADR-058 §I5`; `BC-2.16.002 §Canonical Structured Event Catalog v1.67 row` → `BC-2.16.002 §Canonical Structured Event Catalog`. No behavioral change; no new MUSTs. TD-VSDD-097 three-dimension sweep: (1) Sibling pair — none; CLEAR. (2) Downstream copy target — none of these narrative phrases are verbatim copy-sources in downstream artifacts; CLEAR. (3) Mandate anchor — no new MUSTs; CLEAR. |
| 1.6 | adr058-ocsf-routing-propagation | 2026-08-16 | product-owner | **ADR-058 v2.6 path-liveness propagation.** (A) §Architecture Anchors: replaced stale `"audit_activity"`-only `class_selector.rs` reference with full Path A / Path B distinction per ADR-058 §I5 (v2.6) and §K5. Path A (live): `select_by_class_name` in `pipeline_result_to_record_batch` must gain `"entity_management"→3004` and `"inventory_info"→5001` arms + new const `CLASS_UID_ENTITY_MANAGEMENT = 3004`; `"audit_activity"` arm becomes dead code pending deprecation annotation. Path B (zero production callers, forward-compat only): `select()` in `normalize_with_mappers`. Process-gap `ocsf.unknown_class_name` WARN obligation noted (SAP-1 discharged in BC-2.16.002 catalog v1.67). (B) EC-016-013-023: augmented with wire-level postcondition — Claroty audit_logs batch with `ocsf_class = "entity_management"` MUST produce `class_uid = 3004` in the Arrow `class_uid` Int32 column via Path A (NOT 3001 from the prior wrong `account_change` arm, NOT 0 from unknown-class fallback). (C) EC-016-013-024: augmented with wire-level postcondition — Claroty devices batch with `ocsf_class = "inventory_info"` MUST produce `class_uid = 5001` in the Arrow `class_uid` Int32 column via Path A (regression-prevention assertion: without the `"inventory_info"` arm, the KF-02 TOML change regresses class_uid from the current 5001 to 0). TD-VSDD-097 three-dimension sweep: (1) Sibling pair — BC-2.16.003 has no named split-event twin; CLEAR. (2) Downstream copy target — §Architecture Anchors and EC augmentations are not verbatim copy-sources in downstream artifacts; CLEAR. (3) Mandate anchor — wire-level `class_uid` postconditions anchored to S-ADR058-OCSF-ROUTING-001 (the implementing story for both `"entity_management"` and `"inventory_info"` arms); no unanchored MUSTs introduced. |
| 1.5 | adr058-schema-reconcile | 2026-08-16 | product-owner | ADR-058 v2.4 schema validation reconciliation (KF-01..KF-12). (A) §Description: added Interpretation A paragraph — underscore-flattened Arrow field naming under `ocsf_column_naming = true`, Claroty-first scope, reference to §Postconditions §Claroty Contracted OCSF Mappings. (B) §Preconditions: added `ocsf_column_naming` flag precondition for Claroty. (C) §Postconditions §Column Routing: updated to note valid OCSF class requirement (object names not permitted). (D) §Postconditions: added §Interpretation A: Arrow Field Naming subsection (ADR-058 §B2/§I1/§G/§J2 obligations: underscore-flattened names, raw_extensions blob, prism_describe sourcing, fail-closed collision check). (E) §Postconditions: added §Claroty Contracted OCSF Mappings — four-table ground-truth mapping tables encoding corrected ocsf_field values and Arrow names for all Claroty columns; KF-05 PO decision (audit_logs.id → raw_extensions; activity_uid absent from OCSF v1.7.0); KF-06 PO decision (devices.device_type → vendor-extended device.type_label → Arrow device_type_label; OT subcategory demo-critical). (F) §Invariants: added Interpretation A class-correctness and shadow-prevention invariants. (G) §Error Conditions: added reserved-field overwriting row; updated invalid ocsf_class row to include concrete examples (audit_activity, device). (H) EC-016-013-004: corrected exemplar ocsf_field from `finding.uid` to `finding_info.uid` per KF-03. (I) EC-016-013-012: updated queryable name from `device.ip` to `device_ip` (Arrow field name under Interpretation A underscore-flattening; per ADR-058 §I3 obligation). (J) EC-016-013-013..024: twelve new edge cases — KF-08..KF-11 reserved-field removal testability (013..016), KF-03/04/12 finding_info.* path corrections (017..019), KF-05/06 PO decisions with rationale and test expectations (020..021), KF-07 alert_id correction (022), KF-01 entity_management comment validity (023), KF-02 inventory_info device.* resolution (024). (K) §Canonical Test Vectors: added four Interpretation A scenarios. (L) §Related BCs: added BC-2.01.013 reference (EC-01-025 NON-CONFORMANT resolution). (M) §Architecture Anchors: added class_selector.rs, claroty.sensor.toml, spec_driven_adapter.rs references. (N) §Story Anchor: replaced "no story exists" placeholder with concrete S-ADR058-OCSF-COERCION-001 (Stage 1) and S-ADR058-OCSF-ROUTING-001 (Stage 2) anchor references. (O) §Traceability Known-Gap: updated from "no ID" to anchored story IDs. (P) inputs: added ADR-058; input-hash marked stale-needs-refresh (state-manager must run compute-input-hash after this burst). TD-VSDD-097: (1) sibling pair — no twin BC; N/A. (2) downstream copy target — BC-INDEX title column unchanged (H1 title unchanged); S-ADR058-OCSF-ROUTING-001 §AC-005 mapping tables carry pre-v2.4 column listing — story-writer amendment required per ADR-058 §Status v2.4. (3) mandate anchor — no new MUST statements introduced; existing anchors to S-ADR058-OCSF-COERCION-001 and S-ADR058-OCSF-ROUTING-001 carry all obligations. |
| 1.4 | coercion-gap-closure | 2026-08-11 | product-owner | Human-authorized gap closure (CLAUDE.md §Source-of-Truth item 7): expanded coercion matrix section (String-type-first rule LIVE-DRIFT-003), full column_type × JSON-type matrix, EC-016-013-001..012 edge case catalog with IDs, CoercionWarning observability defect flag, KNOWN GAP annotations for structured-type and integer-column gaps, capability anchor justification, Related BCs, Architecture Anchors, Story Anchor, VP Anchors sections added. Two implementing tests (`test_coerce_value_string_type_normalizes_integer_to_string`, `test_coerce_value_string_type_preserves_string_username_against_uid_heuristic`) cited as evidence for EC-016-013-004 and EC-016-013-005. |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
