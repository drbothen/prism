---
document_type: adr
adr_id: "ADR-058"
title: "v1 Column Naming: OCSF Field-Path Routing with Underscore-Flattened Arrow Names; DTU Migration Deferred"
status: accepted
date: "2026-08-11"
modified: "2026-08-16"
version: "2.7"
producer: architect
subsystems_affected: [SS-01, SS-02, SS-10, SS-16]
supersedes: null
superseded_by: null
amends: null
anchor_stories:
  - S-ADR058-OCSF-COERCION-001
  - S-ADR058-OCSF-ROUTING-001
  - S-OCSF-FIDELITY-CROWDSTRIKE-001
  - S-OCSF-FIDELITY-CYBERINT-001
  - S-OCSF-FIDELITY-ARMIS-001
related_adrs: [ADR-023, ADR-028, ADR-052, ADR-055]
related_bcs: [BC-2.01.013, BC-2.16.002, BC-2.16.003]
inputs:
  - crates/prism-spec-engine/src/column_mapping.rs
  - crates/prism-spec-engine/src/spec_parser.rs
  - crates/prism-bin/src/spec_driven_adapter.rs
  - crates/prism-sensors/specs/claroty.sensor.toml
  - crates/prism-sensors/specs/crowdstrike.sensor.toml
  - crates/prism-sensors/specs/armis.sensor.toml
  - crates/prism-sensors/specs/cyberint.sensor.toml
  - crates/prism-mcp/src/tools/prism_describe.rs
  - .factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md
  - .factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md
  - crates/prism-ocsf/src/normalizer.rs
  - crates/prism-ocsf/src/mappers/spec_driven.rs
  - crates/prism-ocsf/src/class_selector.rs
  - crates/prism-ocsf/ocsf-schema/1.7.0/schema.json
input-hash: "3ce4c0d"
---

# ADR-058: v1 Column Naming — OCSF Field-Path Routing with Underscore-Flattened Arrow Names; DTU Migration Deferred

## §A Context

### §A1 The Unresolved Architectural Question

`ColumnMapper::map_record` (in `prism-spec-engine::column_mapping`) produces a `MappingResult`
whose `mapped_fields` hashmap is keyed by `ocsf_field` path strings (e.g., `"finding.uid"`,
`"actor.user.name"`). This function is fully implemented and tested.

`pipeline_result_to_record_batch` (in `prism-bin::spec_driven_adapter`) converts a
`PipelineResult` to an Arrow `RecordBatch` and uses `col.name` — the raw sensor column
identifier (e.g., `"id"`, `"username"`) — as the Arrow schema field name. It does NOT call
`ColumnMapper::map_record`. The result: `ocsf_field` declarations have no effect on emitted
rows in the current production path.

This creates an implicit architectural decision that was never made explicit: **should the Arrow
RecordBatch schema field names be `col.name` (raw sensor identifiers) or `ocsf_field` paths
(OCSF normalized identifiers)?**

### §A2 How the Gap Arose

`ColumnMapper::map_record` was implemented by the S-1.11 spec-loading story. The integration
seam back to `pipeline_result_to_record_batch` in `prism-bin` was in a different crate and was
never included in S-1.11's scope. The gap is an oversight: no ADR, no tech-debt register entry,
and no STATE.md decision row records a deliberate deferral.

BC-2.01.013 v1.8 EC-01-025 (added D-924 burst, 2026-05-31) explicitly marks
"ColumnMapper step is missing" as NON-CONFORMANT — a conformance rule that recognized the gap,
not one that found a compliant implementation.

### §A3 The Two Interpretations in Conflict

**Interpretation A (BC-2.16.003-aligned, full OCSF routing):**
`ColumnMapper::map_record` routes each column value to its `ocsf_field` key in `mapped_fields`.
Arrow RecordBatch field names become the `ocsf_field` path values (e.g., `"finding.uid"`,
`"actor.user.name"`). Columns without an `ocsf_field` go to a `raw_extensions` JSON blob column.

**Interpretation B (production-current, col.name-preserving):**
Arrow RecordBatch field names are `col.name` values (e.g., `"id"`, `"username"`, `"device_uid"`).
`ocsf_field` is treated as semantic annotation used for `prism_describe` column `description`
only.

### §A4 DataFusion Quoting Constraint Under Interpretation A

Arrow allows any Unicode string as a field name, including strings with dots. However, DataFusion
SQL interprets dotted names as qualified identifiers: `finding.uid` parses as schema `finding`,
column `uid`, not as a column named `"finding.uid"`. This constraint is the central design
challenge for Interpretation A. See §C for the full quoting convention analysis.

### §A5 Human Override — 2026-08-12

ADR-058 v1.0 (2026-08-11) decided **Interpretation B**, deferring full OCSF routing to a future
story. On 2026-08-12 the human overrode this decision with two scoping constraints:

1. **DTU migration is deferred to a future story.** All four DTU generators (claroty, crowdstrike,
   armis, cyberint) are out of scope for v1 Stage 2 wiring.
2. **Development and validation target the LIVE Claroty sensor.** Claroty is the first sensor to
   receive Interpretation A wiring; the other three sensors remain on Interpretation B until their
   respective migration stories land.

The v1.0 §B decision (Interpretation B) is SUPERSEDED. The v2.0 §B below supersedes it.

---

## §B Decision

### §B1 — v1.0 Decision (SUPERSEDED 2026-08-12)

The v1.0 decision was: **use Interpretation B** (col.name as Arrow RecordBatch field name;
ocsf_field as semantic metadata only). This decision is superseded by human override per §A5.
The rationale for why Interpretation A could not ship as a single burst — DataFusion quoting
constraint, missing story, high blast radius — remains valid analysis. The human override
changes the scope and delivery strategy, not the accuracy of the constraint analysis.

### §B2 — v2.0 Decision (ACTIVE)

**ADR-058 v2.0 Decision: v1 uses Interpretation A with underscore-flattened Arrow field names,
enabled per-sensor via a TOML flag, targeting Claroty first. DTU migration is deferred.**

Specifically:

1. **Arrow field names are underscore-flattened OCSF paths.** For a column with
   `ocsf_field = "finding.uid"`, the Arrow RecordBatch schema field name is `finding_uid` (all
   dots replaced with underscores). For a column with `ocsf_field = "actor.user.name"`, the
   Arrow field name is `actor_user_name`. Columns without `ocsf_field` use `col.name` (fallback)
   and are collected into a `raw_extensions` JSON blob column per the original `ColumnMapper`
   design.

2. **Activation is per-sensor via a new TOML flag `ocsf_column_naming = true`.** This flag is
   added to `SensorSpec` with a default of `false`. Only sensors with `ocsf_column_naming = true`
   in their TOML spec use OCSF-flattened Arrow field names; all others retain Interpretation B
   behavior. Claroty's TOML spec is the first to set this flag.

3. **DTU generators require NO changes.** `build_column_array` reads raw JSON records by
   `col.name` (or `source_path`). This extraction logic is unchanged. Only the Arrow schema
   field name changes; the JSON extraction key does not.

4. **BC-2.01.013 EC-01-025 NON-CONFORMANT status is resolved by this stage** for Claroty.
   The `ColumnMapper` wiring gap is closed when `pipeline_result_to_record_batch` uses
   `ocsf_flattened_name(col)` instead of `col.name` for the Arrow field name (gated on the
   sensor flag).

---

## §C Quoting Convention

### §C1 The Problem

If Arrow field names contain dots (e.g., `finding.uid`), DataFusion SQL interprets
`SELECT finding.uid FROM t` as qualified name (`finding.uid`), not as a column reference. All
PrismQL queries would require explicit quoting, creating a reliability tax on every LLM agent
query — agents frequently omit quoting because `prism_describe` returns the column name as a
plain string and the agent copies it verbatim.

### §C2 Options Evaluated

**Option 1 — Dotted Arrow names + double-quoted DataFusion identifiers:**
Arrow field name: `finding.uid`. DataFusion SQL: `SELECT "finding.uid" FROM t`.
- Rejected: LLM agents copy column names from `prism_describe` without quoting. A single
  unquoted `finding.uid` in a query produces a DataFusion qualified-name error with no obvious
  signal that quoting was the issue. This creates a recurring agent failure mode.
- Additional: the Chumsky PrismQL pipe-mode parser segments on dots. `finding.uid` would parse
  as `FieldPath { segments: ["finding", "uid"] }`. Converting this to a DataFusion SQL expression
  would require the SQL emitter to join the segments with a dot (producing `finding.uid`) — which
  DataFusion then misinterprets as qualified name. Correct emission would require emitting
  `"finding.uid"` as a delimited identifier, which requires parser and SQL-emitter changes.

**Option 2 — Backtick-quoted identifiers:**
Arrow field name: `finding.uid`. PrismQL query: SELECT \`finding.uid\` FROM t.
- Rejected: same agent-ergonomics problem as Option 1, plus requires adding backtick-quoted
  identifier support to the Chumsky parser (grammar change). The parser's current `ident_char`
  filter excludes backtick.

**Option 3 — Projection/alias layer:**
Arrow field names remain as `col.name` for extraction; a DataFusion view maps `col.name` to
`ocsf_alias`. This is effectively Interpretation B with a view on top.
- Rejected: does not achieve the goal of OCSF-path routing. Agents would query by OCSF alias but
  the underlying data model remains col.name. Cross-sensor joins keyed on OCSF paths (EC-016-013-012)
  would not work without view-level de-aliasing.

**Option 4 — Underscore-flattened Arrow names (CHOSEN):**
`ocsf_field` dots are replaced with underscores to form the Arrow field name. `finding.uid`
→ `finding_uid`. `actor.user.name` → `actor_user_name`. `device.hw_info.vendor_name` →
`device_hw_info_vendor_name`.
- PrismQL queries use `finding_uid` as a plain identifier — no quoting needed.
- The Chumsky parser already handles `[a-zA-Z0-9_]` identifiers; no grammar changes required.
- DataFusion handles underscore-separated names as standard column identifiers.
- `prism_describe` returns `name: "finding_uid"` — agent copies it verbatim and the query works.
- Cross-sensor joins on `finding_uid` work correctly when both sensors declare `ocsf_field = "finding.uid"`.

### §C3 Backward Compatibility

The quoting convention change is backward-incompatible for existing Claroty queries. Queries
using `SELECT id FROM claroty_alerts` must become `SELECT finding_info_uid FROM claroty_alerts`
(KF-03: `id` maps to `finding_info.uid` → Arrow `finding_info_uid`).
This is expected and scoped: Claroty via the live sensor was not producing useful query results
before this change (the LIVE-DRIFT series of defects). The breaking change is bounded to Claroty
and is part of the known blast radius of enabling Interpretation A.

No existing CrowdStrike, Armis, or Cyberint queries are affected until those sensors receive
`ocsf_column_naming = true` in their TOML specs (deferred to future stories).

### §C4 Grammar and Component Impact

- **PrismQL grammar (Chumsky parser, `filter_parser.rs`, `pipe_parser.rs`, `sql_parser.rs`):**
  No changes needed. Underscore-flattened names are valid identifiers under the existing parser.
- **DataFusion SQL planning layer:** No changes needed. Standard identifiers.
- **`pipeline_result_to_record_batch` (`prism-bin::spec_driven_adapter`):** One logic branch added:
  compute `arrow_field_name_for(col, sensor_spec)` which returns `ocsf_flattened_name(ocsf_field)`
  when `sensor_spec.ocsf_column_naming == true && col.ocsf_field.is_some()`, else `col.name`.
- **`prism_describe` (`prism-mcp::tools::prism_describe`):** `ColumnDescriptor.name` sourcing
  changes per §F when the sensor has `ocsf_column_naming = true`.

---

## §D Per-Sensor Scoping

### §D1 Is Per-Sensor Scoping Possible?

**Yes.** `ColumnMapper` and `pipeline_result_to_record_batch` are shared functions, but they
receive `&TableSpec` (and transitively the sensor's `SensorSpec`) as input. The naming logic
can branch on a per-sensor flag without structural redesign.

Without the flag mechanism, enabling Interpretation A by applying it to every column with an
`ocsf_field` declaration would affect all four sensors simultaneously (all four declare
`ocsf_field` on at least some columns: claroty=31, crowdstrike=17, armis=21, cyberint=12
declarations across all tables). That would break integration tests for CrowdStrike and others
that assert on `col.name` values.

### §D2 The Flag Mechanism

A new optional TOML field `ocsf_column_naming = true` is added to the sensor-level spec
(alongside `sensor_id`, `auth_type`, etc.). The corresponding `SensorSpec` struct in
`prism-spec-engine::spec_parser` gains an optional field:

```rust
#[serde(default)]
pub ocsf_column_naming: bool,
```

`default = false` ensures all existing sensor TOMLs parse without the field and retain
Interpretation B behavior. Only `claroty.sensor.toml` sets `ocsf_column_naming = true` in v1.

(Anchored: S-ADR058-OCSF-ROUTING-001 AC-001, RG-001/RG-002)

### §D3 Why Interpretation A Cannot Be All-or-Nothing at This Stage

DTU integration tests for CrowdStrike assert on `col.name` values:
- `bc_2_01_013_spec_driven_adapter.rs` uses inline `ColumnSpec` constructions where `ocsf_field
  = None`, so those tests are NOT directly affected.
- However, integration tests that load the actual crowdstrike.sensor.toml (e.g., e2e tests)
  would fail on Arrow schema field name assertions if CrowdStrike were enabled.
- The e2e Claroty test (`test_BC_2_11_005_e2e_claroty_query_returns_data`) is `#[ignore]`'d and
  asserts on `row.get("uid")` — this assertion needs updating in the Stage 2 story.

---

## §E Blast Radius Under v2.0 Decision

### §E1 `just check` Compatibility

**`just check` stays green.** The baseline is 5690 passing tests on branch
`fix/claroty-live-api-fidelity`. Under Interpretation A with per-sensor scoping (claroty-only
flag):

| Category | Impact |
|----------|--------|
| `bc_2_01_013_spec_driven_adapter.rs` unit tests | NOT affected — use inline `ColumnSpec` with `ocsf_field = None`; Arrow names stay `col.name` |
| `e2e_smoke.rs` Claroty test | NOT counted in `just check` — `#[ignore]`'d |
| `prism_describe` unit tests | NOT affected — use inline `ColumnDescriptor` constructions |
| DTU parity tests (`prism-dtu-claroty/tests/`) | NOT affected — assert on DTU HTTP output JSON, not Arrow schema |
| CrowdStrike / Armis / Cyberint tests | NOT affected — sensor flag `ocsf_column_naming` defaults to `false` |

Tests that WILL need updating in the Stage 2 story (currently `#[ignore]`'d):
- `test_BC_2_11_005_e2e_claroty_query_returns_data`: `row.get("uid")` → `row.get("device_uid")`
  (claroty_devices `uid` column, `ocsf_field = "device.uid"`)
- Any prism_describe integration test asserting Claroty column names by `col.name` value

### §E2 Column Name Mapping Under Interpretation A for Claroty

All `ocsf_field` declarations across the four Claroty tables, with resulting Arrow field names
when `ocsf_column_naming = true`. Columns without `ocsf_field` go to the `raw_extensions`
JSON blob. Two Stage 2 changes are pending delivery: the `ocsf_column_naming = true` flag
is not yet in the TOML, and the `device_category` `ocsf_field` fix (`"device.type"` →
`"device.type_category"`) is also pending Stage 2 per §J3.

**Claroty `alerts` table (9 ocsf_field declarations; ocsf_class = `detection_finding` — VALID, class_uid 2004):**

| col.name | ocsf_field | Arrow name under Interp. A | Note |
|----------|-----------|---------------------------|------|
| `id` | `finding_info.uid` | `finding_info_uid` | KF-03: corrected from `finding.uid` — `detection_finding` has `finding_info` attr (required), not `finding`; TOML fix pending §K4 |
| `alert_type_name` | `type_name` | `type_name` | KF-09: SEMANTIC-WRONG — `type_name` is OCSF-computed from `type_uid`; remove `ocsf_field` pending §I5 |
| `category` | `class_name` | `class_name` | KF-08: SEMANTIC-WRONG — `class_name` is OCSF-computed from `class_uid`; remove `ocsf_field` pending §I5 |
| `status` | `status` | `status` | single segment, unchanged |
| `detected_time` | `time` | `time` | single segment, unchanged |
| `updated_time` | `end_time` | `end_time` | KF-12: SEMANTIC-WRONG — correct target: `finding_info.modified_time` (Arrow: `finding_info_modified_time`); TOML fix pending §I5 |
| `devices_count` | `count` | `count` | KF-10: SEMANTIC-WRONG — `count` is OCSF dedup counter ≠ device count; remove `ocsf_field` pending §I5 |
| `description` | `message` | `message` | single segment, unchanged |
| `alert_name` | `finding_info.title` | `finding_info_title` | KF-04: corrected from `finding.title`; TOML fix pending §K4 |
| `alert_class`, `ot_devices_count` | (none) | — | → `raw_extensions` blob |

**Claroty `audit_logs` table (8 ocsf_field declarations; ocsf_class = `audit_activity` — WRONG per KF-01; correct class: `entity_management`, class_uid 3004; TOML fix pending §K4):**

| col.name | ocsf_field | Arrow name under Interp. A | Note |
|----------|-----------|---------------------------|------|
| `id` | `activity_uid` | `activity_uid` | KF-05: `activity_uid` absent from OCSF v1.7.0 dictionary_attributes; no direct standard path; recommend remove ocsf_field (→ `raw_extensions`) or vendor-extend; TOML fix pending §K4 |
| `action` | `activity_name` | `activity_name` | single segment, unchanged |
| `user_display_name` | `actor.user.name` | `actor_user_name` | dotted path |
| `category` | `category_name` | `category_name` | KF-11: SEMANTIC-WRONG — `category_name` is OCSF-computed from `category_uid`; remove `ocsf_field` pending §I5 |
| `timestamp` | `time` | `time` | single segment, unchanged |
| `details` | `message` | `message` | single segment, unchanged |
| `username` | `actor.user.uid` | `actor_user_uid` | dotted path; `column_type = "string"` — Rule 1 preempts `uid` heuristic (EC-016-013-005) |
| `note` | `comment` | `comment` | single segment, unchanged; `entity_management` has `comment` — this attribute drives class selection over `api_activity` (which lacks `comment`) |

All audit_logs columns currently carry an ocsf_field declaration; if KF-05 recommendation to
remove `activity_uid` is adopted, `id` goes to `raw_extensions`.

**Claroty `devices` table (8 ocsf_field declarations; ocsf_class = `device` — WRONG per KF-02; correct class: `inventory_info`, class_uid 5001; `device.*` fields resolve via the required `device` attribute on `inventory_info`; `device_category` pending §J3 fix; TOML fixes pending §K4):**

| col.name | ocsf_field (current TOML) | Arrow name under Interp. A | Note |
|----------|--------------------------|---------------------------|------|
| `uid` | `device.uid` | `device_uid` | |
| `asset_id` | `device.instance_uid` | `device_instance_uid` | |
| `device_category` | `device.type` | `device_type` | **SHADOW** — §J3 fix changes to `device.type_category` → `device_type_category`; `device.type` is a valid OCSF path (device.type exists); `device.type_category` is vendor-extended (not in OCSF schema, consistent with §J3 rationale — Claroty values "OT Device"/"IT Device" fall outside OCSF device.type vocabulary) |
| `device_type` | `device.type_label` | `device_type_label` | KF-06: `device.type_name` absent from OCSF v1.7.0 device object; settled fix: `device.type_label` (vendor-extended path for OT subcategory); Arrow name `device_type_label` |
| `risk_score` | `risk_score` | `risk_score` | single segment; self-match (legal); `inventory_info.risk_score` confirmed at class level |
| `retired` | `status_code` | `status_code` | single segment, unchanged; `inventory_info.status_code` confirmed at class level |
| `device_name` | `device.name` | `device_name` | self-match: `device.name` flattens to `device_name` = col.name; A = B excluded by §J2 rule |
| `os_category` | `device.os.name` | `device_os_name` | three-segment path; device has `os` attr (os object), os object has `name` confirmed |
| 12 columns without ocsf_field | (none) | — | → `raw_extensions` blob |

**Claroty `device_alert_relations` table (6 ocsf_field declarations; ocsf_class = `detection_finding` — VALID, class_uid 2004):**

| col.name | ocsf_field | Arrow name under Interp. A | Note |
|----------|-----------|---------------------------|------|
| `device_uid` | `device.uid` | `device_uid` | `detection_finding.device.uid` confirmed |
| `alert_id` | `finding_info.uid` | `finding_info_uid` | KF-07: corrected from `finding.uid` — same root error as KF-03; TOML fix pending §K4 |
| `device_alert_detected_time` | `time` | `time` | single segment, unchanged |
| `device_risk_score` | `risk_score` | `risk_score` | single segment, unchanged |
| `alert_note` | `comment` | `comment` | single segment, unchanged |
| `device_alert_status` | `status` | `status` | single segment, unchanged |
| 4 columns without ocsf_field | (none) | — | → `raw_extensions` blob |

Shadow check for `device_alert_relations`: all 6 flattened names verified against all 10
col.names in the table — no flag-transition shadow found. See §J5.

---

## §F DTU Generator Correction

### §F1 Earlier Claim was Wrong

ADR-058 v1.0 §A4 item 3 stated: "Every existing PrismQL integration test that queries sensor
columns by `col.name` would fail under Interpretation A. Every DTU generator that produces
records keyed by `col.name` would need update."

**This was incorrect for DTU generators.** The correction:

`build_column_array` reads raw JSON records by `col.name` (or `source_path`) at the extraction
step. DTU generators produce JSON keyed by the API field names (e.g., `{"id": 132, ...}`), and
`col.name = "id"` in the TOML spec matches those keys. Under Interpretation A, the extraction
logic is UNCHANGED — `r.get(&col.name)` still reads `r.get("id")`. What changes is ONLY the
Arrow schema field name: `Field::new(&col.name, ...)` becomes `Field::new(&arrow_name, ...)`.

Therefore: **DTU generators need no changes for Interpretation A.** The coupling is only in
DTU parity tests that assert on Arrow schema field names (not on DTU HTTP response JSON). Those
tests use inline specs without ocsf_field and are not affected by per-sensor scoping.

---

## §G prism_describe Output Specification

Under Interpretation A (when `sensor_spec.ocsf_column_naming == true`):

- `ColumnDescriptor.name` = `arrow_field_name_for(col)` (underscore-flattened ocsf_field, or
  `col.name` fallback for columns without ocsf_field)
- `ColumnDescriptor.description` = `col.ocsf_field.clone()` (the original dotted OCSF path,
  e.g., `"finding.uid"`, preserved as semantic annotation)

This is consistent with Interpretation B for the description field: the dotted OCSF path
continues to appear in the description, but the name now carries the queryable identifier.

For LLM agents: the agent reads (e.g.) `name: "finding_info_uid"` and uses it verbatim in
queries. The `description: "finding_info.uid"` field provides OCSF semantic context without
being required in queries. (KF-03 corrects the earlier example `finding_uid` / `finding.uid`
— `detection_finding` uses `finding_info`, not `finding`.)

`ColumnDescriptor.description` sourcing for Interpretation B sensors is unchanged:
`col.ocsf_field.clone()` (same as before).

---

## §H Stage 1 Scope (Unchanged)

Stage 1 (coercion integration) is unchanged from v1.0. It remains in scope for the Claroty
live-API work regardless of the Interpretation A decision. Stage 1 does not require the
`ocsf_column_naming` flag — it operates within `build_column_array` for any sensor.

Stage 1 deliverables:

1. **Fix EC-016-013-008 (String column + Object input):** The `ColumnType::String` arm in
   `build_column_array` must return `None` (null cell) and emit `tracing::warn!(event_type =
   "column_coercion_failure", ...)` for `Value::Object` inputs. The current wildcard fallback
   (`other => other.to_string()`) produces a JSON string from the object, which is wrong.

2. **Fix EC-016-013-009 (Integer column + String input on non-numeric OCSF path):**
   `build_column_array`'s `ColumnType::Integer` arm currently returns `None` for String inputs
   regardless of OCSF path. `ColumnMapper::coerce_value` attempts `s.parse::<i64>()` when the
   OCSF path has a numeric suffix. Integrating this coercion into `build_column_array` (or
   dispatching through it) fixes the behavior. The String-type-first rule (LIVE-DRIFT-003)
   already implemented in `coerce_value` is unaffected.

3. **`column_coercion_failure` tracing emission:** A `tracing::warn!(event_type =
   "column_coercion_failure", column = %col.name, column_type = %col.column_type, actual_json_kind
   = %kind)` emission must be added to `build_column_array` at the demotion point (String column
   + Object input). A corresponding row must be added to BC-2.16.002 §Postconditions Canonical
   Structured Event Catalog (SAP-1 / PG-LP11-001 obligation).

(Anchored: S-ADR058-OCSF-COERCION-001 AC-004, RG-005)

---

## §I Implementation Guidance

### §I1 Arrow Field Name Helper

Add a free function `ocsf_field_to_arrow_name(ocsf_field: &str) -> String` to
`prism-bin::spec_driven_adapter` (or `prism-spec-engine`) that replaces all dots with
underscores. Example: `ocsf_field_to_arrow_name("actor.user.name")` → `"actor_user_name"`.

Update `pipeline_result_to_record_batch` to use:
```rust
let arrow_name = if sensor_spec.ocsf_column_naming {
    col.ocsf_field.as_deref()
        .map(ocsf_field_to_arrow_name)
        .unwrap_or_else(|| col.name.clone())
} else {
    col.name.clone()
};
Field::new(&arrow_name, column_type_to_arrow(&col.column_type), true)
```

### §I2 raw_extensions Handling

When `sensor_spec.ocsf_column_naming == true`, columns with `col.ocsf_field == None` go to a
`raw_extensions` Json column (a single Arrow column of type `Utf8` containing a serialized JSON
object). This is the `ColumnMapper::map_record` design intent. The `raw_extensions` column is
queryable via `SELECT raw_extensions FROM claroty_alerts` — LLMs can inspect its content but
cannot filter on nested keys without JSON path functions.

### §I3 BC Amendment Obligations (for product-owner)

- **BC-2.01.013 EC-01-025**: update the NON-CONFORMANT annotation to reference the Stage 2
  story once it is created. After the Stage 2 story merges, update to CONFORMANT.
- **BC-2.16.003 §Story Anchor**: add the Stage 2 story reference. The current §Story Anchor
  lists only Stage 1 context; Stage 2 (OCSF routing wiring) needs its own anchor row.
- **BC-2.16.003 EC-016-013-012**: this EC presupposes `device.ip` is queryable as `device.ip`
  (dotted path). Under Interpretation A with underscore-flattening, the queryable name is
  `device_ip`. This EC must be updated to reference `device_ip` once sensors are migrated.
- **BC-2.16.002**: add `column_coercion_failure` catalog row (Stage 1 obligation).

### §I4 Non-Interference with ADR-055 and ADR-028

ADR-055 governs `SpecValidator::validate` call sites. ADR-028 governs TOML spec grammar. Neither
is superseded by ADR-058 v2.0. The new `ocsf_column_naming` field is a backward-compatible TOML
extension that does not change the parsing behavior for other fields.

### §I5 OCSF Schema Correction TOML Obligations (§K4 findings)

Seven schema-validation findings (§K4, KF-01 through KF-07) require corrections to
`claroty.sensor.toml`. Since `S-ADR058-OCSF-ROUTING-001` AC-005 already modifies
`claroty.sensor.toml` (adds `ocsf_column_naming = true` and the §J3 `device_category` fix),
these corrections fit within that story's scope or require a companion story. Story-writer and
product-owner must adjudicate scope. Corrections required:

- **KF-01:** `[tables] table_name = "audit_logs"` — change `ocsf_class` from `"audit_activity"`
  (class absent from OCSF v1.7.0) to `"entity_management"` (class_uid 3004).
- **KF-02:** `[tables] table_name = "devices"` — change `ocsf_class` from `"device"` (OCSF
  object, not class) to `"inventory_info"` (class_uid 5001, "Device Inventory Info").
- **KF-03:** `alerts.id` column — change `ocsf_field` from `"finding.uid"` to
  `"finding_info.uid"`. Arrow field name changes from `finding_uid` → `finding_info_uid`.
- **KF-04:** `alerts.alert_name` column — change `ocsf_field` from `"finding.title"` to
  `"finding_info.title"`. Arrow field name changes from `finding_title` → `finding_info_title`.
- **KF-05:** `audit_logs.id` column — `ocsf_field = "activity_uid"` is invalid (`activity_uid`
  absent from OCSF v1.7.0 dictionary_attributes). Product-owner decides: (a) remove `ocsf_field`
  (column goes to `raw_extensions`), or (b) use vendor-extended path. No standard direct path
  for Claroty audit log record identifier exists in OCSF v1.7.0.
- **KF-06:** `devices.device_type` column — `ocsf_field = "device.type_name"` is invalid
  (`device` object has no `type_name` attribute; only `type` and `type_id` exist). Settled
  (v2.5): change `ocsf_field` to `"device.type_label"` (vendor-extended path). Arrow field
  name: `device_type_label`. No standard OCSF equivalent for Claroty's OT subcategory values
  ("PLC", "HMI"); vendor-extended path is the correct representation.
- **KF-07:** `device_alert_relations.alert_id` column — change `ocsf_field` from `"finding.uid"`
  to `"finding_info.uid"`. Arrow field name changes from `finding_uid` → `finding_info_uid`.
- **KF-01 (code obligation, v2.6 redesign):** `crates/prism-ocsf/src/class_selector.rs` requires four changes in one atomic commit:

  **(a) Add resolver constant:**
  Add `pub const CLASS_UID_ENTITY_MANAGEMENT: u32 = 3004;` alongside the existing UID constants.

  **(b) Add `select_by_class_name` arms — two new arms required:**
  - Add `"entity_management" => Ok(CLASS_UID_ENTITY_MANAGEMENT)` — resolves the corrected TOML `ocsf_class` value to 3004. Without this arm, `"entity_management"` falls to `Err(...)` → `.unwrap_or(0)` → `class_uid = 0` (BASE_EVENT), a regression from the current 3001 to 0.
  - Add `"inventory_info" => Ok(CLASS_UID_DEVICE_INVENTORY_INFO)` — resolves the corrected KF-02 TOML value to 5001. Without this arm, changing the devices TOML from `"device"` to `"inventory_info"` regresses from 5001 to 0. The existing `"device"` arm (resolves to 5001) is retained as a transitional alias.

  **(c) Update `select()` — both audit_log arms (forward-compatibility):**
  - Change `("claroty", "audit_log") => Ok(CLASS_UID_ACCOUNT_CHANGE)` to `Ok(CLASS_UID_ENTITY_MANAGEMENT)`. Claroty is the primary subject of this cascade.
  - Change `("armis", "audit_log") => Ok(CLASS_UID_ACCOUNT_CHANGE)` to `Ok(CLASS_UID_ENTITY_MANAGEMENT)`. Same semantic defect; same fix (TD-VSDD-097 dimension 1 sibling sweep, amended v2.6).
  These are forward-compatibility fixes for when Path B is wired; Path B has zero production callers today per §K5 path-liveness determination.

  **(d) Dead-code annotation:** After the TOML changes, the existing `"audit_activity"` arm in `select_by_class_name` becomes dead code (no TOML will emit that string). The implementer MUST annotate it as a deprecated transitional entry pending removal, and AC-009/RG-011 must assert the LIVE strings (`"entity_management"`, `"inventory_info"`) rather than the now-dead `"audit_activity"`/`"device"`.

  **Production defect being fixed (Path A):** `select_by_class_name("audit_activity").unwrap_or(0)` currently produces `class_uid = 3001` in Arrow output, misclassifying Claroty audit events as "Identity & Access Management / Account Change" for all downstream consumers. After the KF-01 TOML change to `"entity_management"`, `class_uid` must become 3004 — but only works if arm (b) above is added first.

  **Wire-shape assertion obligation:** The story AC covering this change MUST include a wire-shape test that materializes a `RecordBatch` from Claroty audit_logs data (simulated pipeline result with `ocsf_class = "entity_management"`) and asserts the `class_uid` Arrow column value equals `3004`. A parallel assertion for devices with `ocsf_class = "inventory_info"` must assert `class_uid = 5001`. Assertions must be at the `RecordBatch` / serialized-column level (BC-2.11.001 wire-shape discipline) — not at the resolver-unit-test string level. A resolver unit test asserting `select_by_class_name("entity_management") == Ok(3004)` is necessary but NOT sufficient; it cannot catch a `pipeline_result_to_record_batch` integration gap. Anchored to `S-ADR058-OCSF-ROUTING-001`.
- **KF-08:** `alerts.category` column — remove `ocsf_field = "class_name"`. `class_name` is
  OCSF-computed from `class_uid`; vendor category string overwrites "Detection Finding". Column
  goes to `raw_extensions` blob.
- **KF-09:** `alerts.alert_type_name` column — remove `ocsf_field = "type_name"`. `type_name` is
  OCSF-computed from `type_uid`; vendor alert type string overwrites computed OCSF event type name.
  Column goes to `raw_extensions` blob.
- **KF-10:** `alerts.devices_count` column — remove `ocsf_field = "count"`. OCSF `count` =
  "number of times events in the same logical group occurred" (dedup counter); `devices_count` =
  number of devices affected by the alert; semantic mismatch corrupts OCSF consumers. Column goes
  to `raw_extensions` blob.
- **KF-11:** `audit_logs.category` column — remove `ocsf_field = "category_name"`. `category_name`
  is OCSF-computed from `category_uid`; vendor category string overwrites the OCSF category label
  (e.g., "Identity & Access Management"). Column goes to `raw_extensions` blob.
- **KF-12:** `alerts.updated_time` column — change `ocsf_field` from `"end_time"` to
  `"finding_info.modified_time"`. `end_time` = "time event ended"; `updated_time` = "when the
  alert record was last modified in the source system"; `finding_info.modified_time` is confirmed
  in OCSF v1.7.0 `finding_info` object and is the semantically correct target. Arrow field name
  changes from `end_time` → `finding_info_modified_time`.

KF-03, KF-04, KF-07, KF-12 are definitive (single correct replacement). KF-08, KF-09, KF-10,
KF-11 are definitively recommended for removal; PO may override with documented semantic acceptance
of the OCSF metadata corruption. KF-01, KF-02 are definitive (class corrections); KF-01 also
requires the `class_selector.rs` code change above. KF-05, KF-06 require product-owner semantic
decision on vendor extension vs omission.

**Process-gap obligation (v2.6 — silent-fallback warn):** `spec_driven_adapter.rs::pipeline_result_to_record_batch` uses `EventClassSelector::select_by_class_name(&table.ocsf_class).unwrap_or(0)` with a comment citing intentional fallback (D-925). This silently converts any unknown `ocsf_class` string to `class_uid = 0` (BASE_EVENT) with no diagnostic. This is SOUL.md #4: a silent failure that let CRIT-001 (unknown class names producing 0) pass CI undetected. Obligation (anchored to `S-ADR058-OCSF-ROUTING-001`): replace `.unwrap_or(0)` with a match that emits `tracing::warn!(event_type = "ocsf.unknown_class_name", ocsf_class = %table.ocsf_class, sensor_id = %sensor_id, table_name = %table.table_name, "sensor TOML declares unrecognised ocsf_class; class_uid defaulted to 0 (BASE_EVENT)")` on the `Err` branch before returning 0. SAP-1 obligation: the new `event_type = "ocsf.unknown_class_name"` requires a BC-2.16.002 Canonical Structured Event Catalog row with full field schema, audit role, and recurrence policy before the PR merges.

**Out-of-perimeter note (follow-up recommendation, not in this cascade's scope):** Multiple committed sensor TOMLs in the workspace use `ocsf_class = "device_inventory_info"` — this string is also absent from `select_by_class_name` and resolves silently to 0. A broader resolver audit covering all TOML-declared `ocsf_class` strings across all four sensor TOMLs is recommended as a follow-up story. Not expanding this cascade's perimeter to fix that; the process-gap `tracing::warn!` above will make these misses observable at runtime once implemented.

---

## §J Flag-Transition Name Shadowing (Architectural Adjudication, v2.1)

### §J1 The Defect Class

RG-009 (S-ADR058-OCSF-ROUTING-001) guards against intra-table flattened-name duplicates: two
columns whose `ocsf_field` values flatten to the same Arrow field name when
`ocsf_column_naming = true`. The Claroty TOML has the one shadow documented below
(`device_category`/`device_type`); RG-009 passes cleanly after the §J3 TOML fix. The
`devices` table produces eight distinct flattened names after the §J3 fix: `device_uid`,
`device_instance_uid`, `device_type_category`, `device_type_label`, `risk_score`, `status_code`,
`device_name`, `device_os_name`.
Cross-table `time` values (`alerts.detected_time`, `audit_logs.timestamp`, and
`device_alert_relations.device_alert_detected_time`) are cross-table and harmless. The total
`ocsf_field` count across all four Claroty tables is **31** (alerts: 9, audit_logs: 8,
devices: 8, device_alert_relations: 6) — see §J4.

A distinct defect class evades RG-009: **flag-transition name shadowing.** A flattened
`ocsf_field` name from one column equals the `col.name` of a DIFFERENT column in the same table.
In the Claroty `devices` table:

| col.name | ocsf_field | Arrow name (flag=false) | Arrow name (flag=true) |
|---|---|---|---|
| `device_category` | `device.type` | `device_category` | **`device_type`** |
| `device_type` | `device.type_name` | **`device_type`** | `device_type_name` |

`SELECT device_type FROM claroty_devices` is a valid PrismQL query in BOTH flag states and
returns DIFFERENT semantic content:
- flag=false: returns `device_type` column data — type-within-category ("PLC", "HMI")
- flag=true: returns `device_category` data via its flattened name — high-level category
  ("OT Device", "IT Device")

No runtime error, no schema construction failure, no warning. Arrow 58 sees distinct names
(`device_type` and `device_type_name`) in the flag=true schema; the shadowing is cross-mode and
is invisible to the intra-flag-state duplicate check in RG-009.

Story-writer's TD-VSDD-097 report for S-ADR058-OCSF-ROUTING-001 covered only the three ANCHOR-
NEEDED anchor-discharge edits (A)–(C). The §"Status as of v2.0" stale-claim correction (D), the
dangling §D2 ANCHOR-NEEDED cross-reference (E), and this adjudication (§J) were absent. Edits
(D) and (E) were falsified by the same correction burst that authored those stories — a textbook
dimension-2 failure: the co-authored artifact retained stale narrative that its own corrected
work had already falsified without sweeping it.

### §J2 Normative Rule

**Rule (ADR-058 v2.1):** When `ocsf_column_naming = true` is active for a sensor, no flattened
`ocsf_field` name derived from any column in a table may equal the `col.name` of any OTHER column
in the same table.

Formally: for every table T in a sensor with `ocsf_column_naming = true`, for every pair of
distinct columns A and B in T where `A.ocsf_field` is `Some`:

```
ocsf_field_to_arrow_name(A.ocsf_field) ≠ B.col_name   (A ≠ B)
```

**Enforcement:** `pipeline_result_to_record_batch` MUST check this condition when
`sensor_spec.ocsf_column_naming == true` and return `Err(ArrowError::SchemaError(...))` on
violation, fail-closed. The check runs in the same collision-detection pass as the existing
intra-flattened-name duplicate check (T-21 / RG-009, S-ADR058-OCSF-ROUTING-001).

Mandate anchor for the extended check: story-writer must amend S-ADR058-OCSF-ROUTING-001 to
add RG-010 (a Red Gate test that constructs a sensor spec with a flattened ocsf_field name equal
to a different column's col.name in the same table and asserts `Err`) and extend T-21 to also
sweep flattened names against other columns' col.name values. The human will dispatch
story-writer for this amendment after this ADR amendment lands.

**Rationale for fail-closed:** Silent wrong-column data at the query surface is a correctness
defect with no diagnostic signal — the LLM agent queries `device_type` and receives category
data ("OT Device") with no indication that the semantics have changed from flag=false behavior
("PLC"). A permitted-migration path is not viable because no migration period exists in which
both flag states return the same column for the same query name. Fail-closed at schema
construction is the only sound gate.

**Scope:** Per-sensor, at flag activation time (when `pipeline_result_to_record_batch` runs
with `ocsf_column_naming = true`). Sensors with `ocsf_column_naming = false` (the default for
all four current sensors in Stage 2) are not checked. If a future story enables the flag for
CrowdStrike, Armis, or Cyberint, a pre-activation shadow check against that sensor's TOML MUST
be included in the story's Red Gate tests before the flag can be set to `true`.

### §J3 Claroty `devices` Table Resolution

Under the fail-closed rule, `ocsf_column_naming = true` cannot ship for Claroty until the
`devices` table collision is resolved.

**Decision:** Change `device_category`'s `ocsf_field` from `"device.type"` to
`"device.type_category"`. After this change, the `devices` table flattened names under flag=true
are:

| col.name | ocsf_field (post-fix) | Arrow name (flag=true) |
|---|---|---|
| `uid` | `device.uid` | `device_uid` |
| `asset_id` | `device.instance_uid` | `device_instance_uid` |
| `device_category` | `device.type_category` | `device_type_category` |
| `device_type` | `device.type_label` (KF-06: settled; vendor-extended path for OT subcategory) | `device_type_label` |
| `risk_score` | `risk_score` | `risk_score` |
| `retired` | `status_code` | `status_code` |
| `device_name` | `device.name` | `device_name` |
| `os_category` | `device.os.name` | `device_os_name` |

Shadow check after fix (all 20 devices col.names, 8 ocsf_field columns, A ≠ B rule):
`device_type_category` vs all 19 other col.names — no match. `device_type_label` vs all other
col.names — no match. `device_uid` vs all other col.names — no match. `device_instance_uid`
vs all other col.names — no match. `risk_score` vs all other col.names — self-match only
(`risk_score.col_name = risk_score`; A = B, excluded). `status_code` vs all other col.names
— no match. `device_name` vs all other col.names — self-match only (`device_name.col_name
= device_name`; A = B, excluded; `device.name` flattens to `device_name` = col.name).
`device_os_name` vs all other col.names — no match. No flag-transition shadow remains.
RG-009 passes (all eight flattened names are distinct). RG-010 (see §J2) passes (no flattened
name equals another column's col.name).

**Why this option over the alternatives:**

- *Rename `device_category` col.name:* requires `source_path` TOML grammar support (currently
  unconfirmed as a grammar-level feature) to preserve extraction from the DTU JSON
  `device_category` key. Adding `source_path` grammar support adds scope to Stage 2.
- *Rename `device_type` col.name:* same `source_path` requirement; additionally breaks
  flag=false queries for existing Claroty device type data — a wider blast than necessary.
- *Change `ocsf_field` on `device_category` (chosen):* TOML-only; zero impact on col.name,
  DTU extraction (`r.get("device_category")` unchanged), DTU parity tests (assert DTU HTTP
  response JSON, not Arrow schema), or flag=false Arrow schema. `device.type_category` is a
  vendor-extended OCSF path — Claroty's "OT Device"/"IT Device" values are high-level category
  strings outside the standard OCSF `device.type_id` controlled vocabulary, and a vendor-
  extended path is the honest representation. The `_category` suffix distinguishes the field
  from the subcategory `device_type_label` and is self-describing.

**Blast radius:** zero under flag=false (col.name unchanged); under flag=true the Arrow field for
high-level category becomes `device_type_category` rather than `device_type` — but flag=true
has not shipped, so no production queries break.

**Stage 2 scope:** This fix fits within S-ADR058-OCSF-ROUTING-001. AC-005 already modifies
`claroty.sensor.toml` to add `ocsf_column_naming = true`. Adding the `device_category`
`ocsf_field` change to that same TOML edit requires no new story, no new AC, and no new
machinery. Story-writer's amendment to S-ADR058-OCSF-ROUTING-001 (dispatched by human) covers
both the TOML fix and RG-010 for the extended collision check.

### §J4 `ocsf_field` Count Correction

S-ADR058-OCSF-ROUTING-001 EC-009 note cited "19 ocsf_field values" across three Claroty
tables. The correct count is **31** across four tables: alerts (9) + audit_logs (8) +
devices (8) + device_alert_relations (6) = 31. The v2.1 count of 19 covered only three
tables and used stale per-table figures (alerts: 8, audit_logs: 5, devices: 6); see §E2
ground-truth tables for the corrected per-table inventory. No story amendment is required for
the count alone — the behavioral correctness of the collision detection is verified by
construction via RG-009 and RG-010, not by count.

### §J5 `device_alert_relations` Shadow Analysis

`device_alert_relations` is a fourth Claroty table added by PR #236 (2026-08-12), after
ADR-058 v2.1 was authored. All six ocsf_field declarations were analyzed against the
flag-transition shadow rule (§J2): no flattened `ocsf_field` name equals a different
column's `col.name` in the same table.

Shadow verification (6 ocsf_field columns, 10 col.names total, A ≠ B rule):

| col.name | ocsf_field | Flattened | Shadow match against other col.names? |
|---|---|---|---|
| `device_uid` | `device.uid` | `device_uid` | self-match only (A = B, excluded) — no other col.name = `device_uid` ✓ |
| `alert_id` | `finding_info.uid` (KF-07: corrected from `finding.uid`) | `finding_info_uid` | no other col.name = `finding_info_uid` ✓ |
| `device_alert_detected_time` | `time` | `time` | no other col.name = `time` ✓ |
| `device_risk_score` | `risk_score` | `risk_score` | no other col.name = `risk_score` ✓ |
| `alert_note` | `comment` | `comment` | no other col.name = `comment` ✓ |
| `device_alert_status` | `status` | `status` | no other col.name = `status` ✓ |

**Conclusion:** `device_alert_relations` has zero flag-transition shadows. When a future
migration story enables `ocsf_column_naming = true` scope to include this table, the
pre-activation shadow check required by §J2 passes cleanly with the current column set.

---

## §K OCSF v1.7.0 Schema Validation

### §K1 Methodology

All `ocsf_class` and `ocsf_field` declarations in `claroty.sensor.toml` were validated against
the committed OCSF v1.7.0 schema at `crates/prism-ocsf/ocsf-schema/1.7.0/schema.json`, pinned
via `OCSF_PINNED_VERSION = "1.7.0"` in `crates/prism-ocsf/build.rs` (BC-2.02.009). Validation
procedure: (1) enumerate all 83 OCSF v1.7.0 classes by key; (2) for each table's `ocsf_class`,
verify the declared name is a class key (not an object, not absent); (3) for each `ocsf_field`,
verify every path segment resolves — first segment against the class's own attributes, subsequent
segments against the corresponding object's attributes.

All object attribute sets were verified directly from the schema JSON. Key findings about the
schema structure: `detection_finding` (class_uid 2004) carries `finding_info` as a required
attribute, NOT a bare `finding` attribute; `device` appears only as an object (not a class);
`device` object has `type` and `type_id` but NO `type_name`; `activity_uid` does not appear in
`dictionary_attributes` (only `activity_id` does); `entity_management` (3004) has `comment` but
`api_activity` (6003) does not.

### §K2 OCSF Class Verdicts

| Table | Declared ocsf_class | Verdict | Correct class (class_uid) |
|-------|---------------------|---------|--------------------------|
| `alerts` | `detection_finding` | VALID | — confirmed |
| `audit_logs` | `audit_activity` | WRONG (KF-01) | `entity_management` (3004) |
| `devices` | `device` | WRONG (KF-02) | `inventory_info` (5001) |
| `device_alert_relations` | `detection_finding` | VALID | — confirmed |

**KF-01 rationale:** `audit_activity` is not among the 83 OCSF v1.7.0 classes. `entity_management`
(3004) is the correct replacement: it has `actor`, `comment`, `message`, `category_name`, `time`,
`status`, `status_code`, and `activity_name` — every attribute required by the declared
`ocsf_field` values. `api_activity` (6003) covers all those except `comment`; the `note → comment`
mapping constrains the choice to `entity_management`.

**KF-01 code defect (v2.4):** `class_selector.rs` `select_by_class_name("audit_activity")` maps
to `CLASS_UID_ACCOUNT_CHANGE = 3001` (AccountChange), NOT 3004 (entity_management). This is a
confirmed code defect (see §K5 Divergence 3). The code comment on `CLASS_UID_ACCOUNT_CHANGE`
explicitly flags: "S-1.05 field mappers will verify this is semantically correct or propose an
alternative class_uid" — that verification is now complete. `account_change` (3001) lacks `comment`;
the `note → comment` TOML mapping silently fails under the current mapping (data loss). KF-01 fix
requires BOTH a TOML change AND a `class_selector.rs` code change; see §I5.

**KF-02 rationale:** `device` exists in OCSF v1.7.0 only as an object, not a class. The correct
class for device inventory data is `inventory_info` (5001, "Device Inventory Info"), which declares
`device` as a required primary attribute. Under `inventory_info`, all `device.*` field paths
resolve via the `device` object. `inventory_info` also has `risk_score` and `status_code` at the
class level, satisfying the `retired → status_code` and `risk_score → risk_score` column mappings.

### §K3 ocsf_field Path Verdicts

All 31 `ocsf_field` declarations across four tables validated against OCSF v1.7.0.

**Table: `alerts` (ocsf_class = `detection_finding`, VALID)**

| col.name | ocsf_field (current TOML) | Verdict | Notes |
|----------|--------------------------|---------|-------|
| `id` | `finding.uid` | WRONG (KF-03) | `detection_finding` has `finding_info` (required); no `finding` attr; correct: `finding_info.uid` |
| `alert_type_name` | `type_name` | WRONG (semantic) (KF-09) | path `detection_finding.type_name` confirmed; semantic defect: OCSF `type_name` = "the event type name, as defined by type_uid" — writing vendor alert type string overwrites computed OCSF type; remove `ocsf_field` (→ `raw_extensions`) |
| `category` | `class_name` | WRONG (semantic) (KF-08) | path `detection_finding.class_name` confirmed; semantic defect: OCSF `class_name` = "the event class name, as defined by class_uid" — writing vendor category string overwrites "Detection Finding"; remove `ocsf_field` (→ `raw_extensions`) |
| `status` | `status` | VALID | `detection_finding.status` confirmed |
| `detected_time` | `time` | VALID | `detection_finding.time` confirmed |
| `updated_time` | `end_time` | WRONG (semantic) (KF-12) | path `detection_finding.end_time` confirmed; semantic defect: `end_time` = "time event ended"; `updated_time` = "when alert record was last modified in source system"; correct: `finding_info.modified_time` (OCSF v1.7.0 `finding_info` object has `modified_time`); Arrow name under Interp. A: `finding_info_modified_time` |
| `devices_count` | `count` | WRONG (semantic) (KF-10) | path `detection_finding.count` confirmed; semantic defect: OCSF `count` = "number of times events in the same logical group occurred" (dedup counter); `devices_count` = number of devices affected by alert — different semantic; remove `ocsf_field` (→ `raw_extensions`) |
| `description` | `message` | VALID | `detection_finding.message` confirmed |
| `alert_name` | `finding.title` | WRONG (KF-04) | Same root error as KF-03; correct: `finding_info.title` |

**Table: `audit_logs` (ocsf_class = `audit_activity` → correct: `entity_management`, KF-01)**

| col.name | ocsf_field (current TOML) | Verdict | Notes |
|----------|--------------------------|---------|-------|
| `id` | `activity_uid` | WRONG (KF-05) | `activity_uid` absent from OCSF v1.7.0 `dictionary_attributes`; `activity_id` (numeric enum) exists but is not a record UID; no standard direct path; recommend remove or vendor-extend |
| `action` | `activity_name` | VALID | `entity_management.activity_name` confirmed |
| `user_display_name` | `actor.user.name` | VALID | `actor.user` resolves to `user` object; `user.name` confirmed |
| `category` | `category_name` | WRONG (semantic) (KF-11) | path `entity_management.category_name` confirmed; semantic defect: OCSF `category_name` = "event category name, as defined by category_uid" — writing vendor category string (e.g., "Login") overwrites OCSF computed category label; remove `ocsf_field` (→ `raw_extensions`) |
| `timestamp` | `time` | VALID | `entity_management.time` confirmed |
| `details` | `message` | VALID | `entity_management.message` confirmed |
| `username` | `actor.user.uid` | VALID | `user.uid` confirmed |
| `note` | `comment` | VALID | `entity_management.comment` confirmed; this attribute drives class choice over `api_activity` |

**Table: `devices` (ocsf_class = `device` → correct: `inventory_info`, KF-02)**

Under `inventory_info`, `device.*` paths resolve via the required `device` object attribute.

| col.name | ocsf_field (current TOML) | Verdict | Notes |
|----------|--------------------------|---------|-------|
| `uid` | `device.uid` | VALID | `inventory_info.device.uid` confirmed |
| `asset_id` | `device.instance_uid` | VALID | `inventory_info.device.instance_uid` confirmed |
| `device_category` | `device.type` (current) / `device.type_category` (§J3 fix) | VALID path / vendor-extended | `device.type` is a valid OCSF path; §J3 fix to `device.type_category` is vendor-extended (not in OCSF schema) — consistent with §J3's explicit vendor-extension characterization; Claroty's "OT Device"/"IT Device" values fall outside OCSF's controlled vocabulary for `device.type`, confirming the vendor-extension rationale |
| `device_type` | `device.type_name` | WRONG (KF-06) | `device` object has no `type_name` attr; only `type` (string) and `type_id` (integer) exist; no standard OCSF equivalent for Claroty OT subcategory ("PLC", "HMI"); recommend vendor-extended path or remove |
| `risk_score` | `risk_score` | VALID | `inventory_info.risk_score` confirmed at class level |
| `retired` | `status_code` | VALID | `inventory_info.status_code` confirmed at class level |
| `device_name` | `device.name` | VALID | `device.name` confirmed |
| `os_category` | `device.os.name` | VALID | `device.os` resolves to `os` object; `os.name` confirmed |

**Table: `device_alert_relations` (ocsf_class = `detection_finding`, VALID)**

| col.name | ocsf_field (current TOML) | Verdict | Notes |
|----------|--------------------------|---------|-------|
| `device_uid` | `device.uid` | VALID | `detection_finding.device.uid` confirmed (`detection_finding` has `device` attr) |
| `alert_id` | `finding.uid` | WRONG (KF-07) | Same root error as KF-03; correct: `finding_info.uid` |
| `device_alert_detected_time` | `time` | VALID | `detection_finding.time` confirmed |
| `device_risk_score` | `risk_score` | VALID | `detection_finding.risk_score` confirmed |
| `alert_note` | `comment` | VALID | `detection_finding.comment` confirmed |
| `device_alert_status` | `status` | VALID | `detection_finding.status` confirmed |

### §K4 Finding Summary

| Finding | Location | Current Value | Schema Verdict | Corrected Value |
|---------|----------|---------------|----------------|-----------------|
| KF-01 | `audit_logs` table `ocsf_class` | `audit_activity` | WRONG — class absent from OCSF v1.7.0 | `entity_management` (+ `class_selector.rs` code change; see §K2 and §I5) |
| KF-02 | `devices` table `ocsf_class` | `device` | WRONG — OCSF object, not class | `inventory_info` |
| KF-03 | `alerts.id` `ocsf_field` | `finding.uid` | WRONG — no `finding` attr on detection_finding | `finding_info.uid` |
| KF-04 | `alerts.alert_name` `ocsf_field` | `finding.title` | WRONG — same root as KF-03 | `finding_info.title` |
| KF-05 | `audit_logs.id` `ocsf_field` | `activity_uid` | WRONG — attr absent from v1.7.0 | remove or vendor-extend (PO decision) |
| KF-06 | `devices.device_type` `ocsf_field` | `device.type_name` | WRONG — attr absent on device object | `device.type_label` (Arrow: `device_type_label`; vendor-extended; settled v2.5) |
| KF-07 | `device_alert_relations.alert_id` `ocsf_field` | `finding.uid` | WRONG — same root as KF-03 | `finding_info.uid` |
| KF-08 | `alerts.category` `ocsf_field` | `class_name` | PATH-VALID / SEMANTIC-WRONG — `class_name` is OCSF-computed from `class_uid`; vendor value overwrites | remove `ocsf_field` (→ `raw_extensions`) |
| KF-09 | `alerts.alert_type_name` `ocsf_field` | `type_name` | PATH-VALID / SEMANTIC-WRONG — `type_name` is OCSF-computed from `type_uid`; vendor value overwrites | remove `ocsf_field` (→ `raw_extensions`) |
| KF-10 | `alerts.devices_count` `ocsf_field` | `count` | PATH-VALID / SEMANTIC-WRONG — OCSF `count` = event dedup counter; `devices_count` = device count | remove `ocsf_field` (→ `raw_extensions`) |
| KF-11 | `audit_logs.category` `ocsf_field` | `category_name` | PATH-VALID / SEMANTIC-WRONG — `category_name` is OCSF-computed from `category_uid`; vendor value overwrites | remove `ocsf_field` (→ `raw_extensions`) |
| KF-12 | `alerts.updated_time` `ocsf_field` | `end_time` | PATH-VALID / SEMANTIC-WRONG — `end_time` = event-end time; `updated_time` = record last-modified | `finding_info.modified_time` (Arrow: `finding_info_modified_time`) |

7 of 35 declarations are WRONG by schema-path analysis (2 ocsf_class + 5 ocsf_field); an additional
5 are PATH-VALID but SEMANTIC-WRONG (KF-08..KF-12) confirmed via normalizer code analysis (§K5).
Total: 12 of 35 WRONG; 23 VALID.

KF-03, KF-04, KF-07, KF-12 are definitive — single correct replacement confirmed. KF-08, KF-09,
KF-10, KF-11 are definitively recommended for removal (→ `raw_extensions`); PO may override with
documented semantic acceptance of the OCSF metadata corruption. KF-01, KF-02 are definitive —
unique correct replacement confirmed; KF-01 also requires the `class_selector.rs` code change
documented in §I5. KF-05, KF-06 require product-owner semantic decision (no standard OCSF path
exists for the underlying concept). TOML and code fix obligations: see §I5.

### §K5 Divergence Adjudication and Path-Liveness Determination (v2.4, amended v2.6)

Independent research-agent cross-validation confirmed KF-01..KF-07 and raised four divergences.
All adjudicated against the committed OCSF v1.7.0 schema and prism's normalizer code
(`crates/prism-ocsf/src/normalizer.rs`, `crates/prism-ocsf/src/mappers/spec_driven.rs`,
`crates/prism-ocsf/src/class_selector.rs`).

**Path-Liveness Determination (v2.6):**

Two paths exist for materializing OCSF `class_uid` for spec-driven sensor data:

- **Path A (spec-driven Arrow — LIVE):** `spec_driven_adapter.rs::pipeline_result_to_record_batch` calls `EventClassSelector::select_by_class_name(&table.ocsf_class).unwrap_or(0)` and emits `class_uid` as an Int32 Arrow column. This is the live production path for ALL spec-driven Claroty (and other TOML-defined) sensors.
- **Path B (protobuf normalizer — NOT LIVE):** `normalizer.rs::normalize_with_mappers` → `EventClassSelector::select(sensor, record_type)` → OcsfEvent with nested protobuf fields via `SpecDrivenMapper::set_nested_field`. This path is defined but has zero production callers.

**Path A is the ONLY live production path.** Evidence: `test_adapter_normalization.rs` module-level comment (§Production caller status OBS-003): "`normalize_with_mappers` has zero production callers today — it is defined in `normalizer.rs` but called only from test code and integration fixtures. These tests lock the contract for the future protobuf-export path where a real `SensorMapper` will be wired (per BC-2.02.013). Until that wiring lands the tests act as forward-compatibility guardrails." No production code outside test files and integration fixtures calls `normalize_with_mappers`.

**Implications for all findings below:**
- Defects in `select_by_class_name` (Path A) are **production defects** — wrong `class_uid` in Arrow output today.
- Defects in `select()` (Path B) are **forward-compatibility defects** — no production impact until Path B is wired, but must be fixed before wiring.
- The `note → comment` data-loss rationale for KF-01 (which depends on `set_nested_field` in Path B) is **moot on the current production path** but architecturally valid for when Path B is wired.

**Divergence 1 — Reserved OCSF base-event metadata fields: CONFIRMED-DEFECT (KF-08..KF-11)**

Claim: four columns write vendor values into OCSF-computed fields (`class_name`, `type_name`,
`category_name`, `count`).

Evidence:
- `normalizer.rs` `normalize_with_mappers`: post-pass handles only
  `OCSF_ENUM_LABEL_FIELDS = ["severity", "status", "activity_name", "disposition"]` — does NOT
  recompute `class_name`, `type_name`, `category_name`, or `count` from their uid fields.
- `spec_driven.rs` `set_nested_field`: writes source values into any valid OCSF field path —
  no guard against reserved metadata fields.
- OCSF v1.7.0 schema: `class_name` = "the event class name, as defined by class_uid value";
  `type_name` = "the event/finding type name, as defined by the type_uid"; `category_name` =
  "the event category name, as defined by category_uid value"; `count` = "the number of times
  that events in the same logical group occurred during the event Start Time period" — all four
  carry OCSF-computed semantics, not free-text vendor slots.

Verdict: CONFIRMED-DEFECT (KF-08..KF-11). Vendor values corrupt OCSF computed metadata:
consumers see "Alerts" instead of "Detection Finding" for `class_name`, incorrect type names,
incorrect OCSF category labels, and device counts in a field that should carry event dedup counts.

**Divergence 2 — `devices.risk_score → "risk_score"` nesting: NOT-A-DEFECT**

Claim: `inventory_info` has no top-level `risk_score`; correct path should be `device.risk_score`.

Evidence: Python query against committed schema confirms `inventory_info` (5001) attributes
include `risk_score` at the class level directly. The research-agent's claim was factually
incorrect. The current mapping `risk_score → risk_score` is semantically correct — class-level
`inventory_info.risk_score` represents the overall device risk score.

Note: `device` object also has `risk_score`; the class-level field is the more appropriate target
for a device-wide risk metric.

**Divergence 3 — `audit_activity` → `entity_management` vs `account_change`: CONFIRMED-DEFECT in code (amended v2.6)**

Claim: `class_selector.rs` maps `"audit_activity"` → `CLASS_UID_ACCOUNT_CHANGE = 3001`
(AccountChange), not `entity_management` (3004) as KF-01 recommends.

Evidence:
- `class_selector.rs` `select_by_class_name("audit_activity")` → `Ok(CLASS_UID_ACCOUNT_CHANGE)`. This is a Path A defect: production Arrow output carries `class_uid = 3001` for all Claroty audit_logs today.
- Code comment on `CLASS_UID_ACCOUNT_CHANGE`: "S-1.05 field mappers will verify this is semantically correct or propose an alternative class_uid" — explicitly flagged as pending verification; that verification is now complete.
- Semantic analysis: Claroty xDome audit logs record device configuration changes, policy updates, and system events — entity management events. `account_change` (3001) is for IAM user account modifications (password resets, role changes). `entity_management` (3004) is for changes to any managed entity. xDome audit data is entity management.
- `entity_management` (3004) has `actor`, `comment`, `message`, `category_name`, `time`, `status`, `status_code`, `activity_name` — every attribute required by the declared `ocsf_field` values. `account_change` (3001) lacks `comment`.
- **Path B context (moot on current production path):** The TOML maps `note → comment`. Under Path B, `set_nested_field` would silently no-op `comment` on AccountChange (3001) because `comment` is not in AccountChange's protobuf descriptor — data loss on every `note` value. Under entity_management, the field resolves correctly. This rationale is architecturally valid but is moot today: Path B has zero production callers (see path-liveness determination above). The path-A defect (`class_uid = 3001` in Arrow output) is the live production issue.

**Verdict: CONFIRMED-DEFECT (production severity on Path A).** ADR-058 KF-01 recommendation (`entity_management`, 3004) is correct. `class_uid = 3001` in Arrow output misroutes Claroty audit events as "Identity & Access Management / Account Change" events to all downstream consumers. Both the TOML change (`"audit_activity"` → `"entity_management"`) and the `class_selector.rs` code change are required atomically. See §I5 for the redesigned code obligation.

**Sibling note (TD-VSDD-097 dimension 1, amended v2.6):** `class_selector.rs` `select()` has TWO defective arms:
1. `("claroty", "audit_log") => Ok(CLASS_UID_ACCOUNT_CHANGE)` — Claroty is the PRIMARY subject of this cascade. Path B is not currently live, so this is a forward-compatibility defect (HIGH), not a production defect today. If Path B is wired before this arm is corrected, this becomes the production note→comment data-loss site.
2. `("armis", "audit_log") => Ok(CLASS_UID_ACCOUNT_CHANGE)` — same semantic defect. Armis audit logs are entity management events; same fix applies.
Both arms must be updated to `Ok(CLASS_UID_ENTITY_MANAGEMENT)` in the same commit. Scope: story-writer dispatched for §I5 obligations must include both arms.

**Divergence 4 — SUBOPTIMAL-but-valid cases**

Five cases evaluated against committed schema and normalizer code:

| Column mapping | Verdict | Evidence |
|----------------|---------|----------|
| `alerts.updated_time → end_time` | CONFIRMED-DEFECT (KF-12) | `finding_info.modified_time` confirmed in OCSF v1.7.0 `finding_info` object; `end_time` = "time event ended"; `updated_time` = "when alert record was last modified in source system" — semantically wrong class |
| `devices.asset_id → device.instance_uid` | NOT-A-DEFECT | `device.instance_uid` confirmed in schema; valid target for vendor-specific asset identifier; `device.uid` already taken by `uid` column |
| `devices.device_category → device.type` | NOT-A-DEFECT | `device.type` confirmed as free-text field in schema; Claroty "OT Device"/"IT Device" category values are valid device type descriptors (§J3 fix changes this to `device.type_category` vendor extension to resolve flag-transition shadow — §J3 rationale unchanged) |
| `devices.os_category → device.os.name` | NOT-A-DEFECT | `device.os.name` confirmed in schema (`device.os` object has `name`); semantically correct — xDome `os_category` values ("Windows", "Linux") are OS names |
| `devices.retired → status_code` | NOTE-ONLY | Boolean `true/false` written to free-text `status_code` is schema-valid; `inventory_info.status_code` confirmed at class level; no clearly superior standard OCSF field for a boolean retirement flag; no KF assigned |

---

## Rationale

Interpretation A (OCSF field-path routing with underscore-flattened Arrow field names) was chosen
over Interpretation B (col.name-preserving) for three reasons:

1. **BC-2.16.003 conformance.** BC-2.16.003 explicitly contracts that `ocsf_field` declarations
   produce queryable OCSF-pathed column names. Interpretation B leaves `ocsf_field` as semantic
   annotation with no effect on queryable names, which is a NON-CONFORMANT state documented in
   BC-2.01.013 EC-01-025.

2. **Agent ergonomics at the query surface.** Underscore-flattened names (`finding_uid`,
   `actor_user_name`) are plain DataFusion identifiers — no quoting required. LLM agents copy
   the name from `prism_describe` verbatim and queries work. All three dotted-name alternatives
   (raw dots, double-quoted, backtick) require agents to apply quoting that is not present in
   the `prism_describe` output, creating a recurring failure mode at the MCP surface.

3. **Per-sensor flag eliminates blast radius.** The `ocsf_column_naming = true` flag ensures
   zero breakage in `just check` for the three non-Claroty sensors while enabling OCSF routing
   for Claroty immediately.

The human override on 2026-08-12 resolved the v1.0 vs v2.0 conflict; §A5 records the decision
provenance. The detailed quoting convention analysis (four options evaluated) is in §C2.

## Consequences

### Positive

- BC-2.01.013 EC-01-025 NON-CONFORMANT annotation resolves for Claroty once Stage 2 wiring
  ships; `ColumnMapper::map_record` is no longer a dead code path.
- LLM agents querying Claroty can use OCSF-semantic column names (`finding_info_uid`,
  `class_name`) without any quoting ceremony.
- Cross-sensor joins on OCSF-normalized field names (`finding_info_uid` from any sensor with
  `ocsf_field = "finding_info.uid"`) become possible once other sensors are migrated.
- PrismQL grammar requires no changes — underscore identifiers are already valid.
- DataFusion query planning requires no changes — standard column identifiers.

### Negative / Trade-offs

- Existing Claroty queries using `col.name` values (`id`, `username`, `alert_name`) break
  under Interpretation A and must be rewritten to OCSF-flattened names (`finding_info_uid`,
  `actor_user_uid`, `finding_info_title`); `alert_class` and other columns without `ocsf_field`
  become accessible only via `raw_extensions`. See §E2 for the full mapping table. (KF-03/KF-04
  corrected from earlier `finding_uid`/`finding_title` — `detection_finding` uses `finding_info`.)
- A `raw_extensions` JSON blob column holds unmapped columns; it is queryable as a column but
  nested keys are not filterable without JSON path functions.
- BC-2.01.013, BC-2.16.003, and BC-2.16.002 each require product-owner amendment after Stage 2
  ships (see §I3 for the full amendment obligation list).

### Status as of v2.7 (2026-08-16)

Decision accepted. Stage 1 (coercion fixes, `column_coercion_failure` emission) is implemented by
`S-ADR058-OCSF-COERCION-001` (status: draft; mandate anchor discharged at §H). Stage 2
(OCSF routing wiring — `ocsf_column_naming` flag, `ocsf_field_to_arrow_name` helper,
`pipeline_result_to_record_batch` update) is implemented by `S-ADR058-OCSF-ROUTING-001`
(status: draft; mandate anchor discharged at §D2 and §J2). Both stories are anchored. See §J for
the flag-transition name shadowing adjudication — `ocsf_column_naming = true` cannot ship until
the `devices` table collision is resolved per §J3. `device_alert_relations` (fourth table, PR
#236) is clean per §J5.

**Product-owner handoff obligations (v2.6):**
- **BC-2.16.003 §Architecture Anchors:** Update to reflect v2.6 path-liveness determination (Path A is the sole live production path; Path B is future-wired).
- **BC-2.16.003 EC-016-013-023 and EC-016-013-024 (class_uid routing):** Verify that the EC text for audit_logs class_uid obligation reflects `entity_management` (3004) as the correct target, not `account_change` (3001). Update if stale.
- **BC-2.16.002 §Canonical Structured Event Catalog:** Add row for `ocsf.unknown_class_name` (new event_type from process-gap obligation in §I5) with fields: `ocsf_class` (string), `sensor_id` (string), `table_name` (string); audit role: DIAGNOSTIC; recurrence policy: per table batch (not per record) — `class_uid` is derived once per batch in `pipeline_result_to_record_batch` before the record loop, so one warn fires per unresolved table batch, not once per row.
- **Subsystem reconciliation:** BC-2.16.003 and related BCs that reference SS-07 or SS-12 in the context of OCSF column naming must be updated to SS-01/SS-02/SS-10/SS-16.

**Story-writer handoff obligations (v2.6, supersedes v2.4 and v2.5 notes):**
`S-ADR058-OCSF-ROUTING-001` must be amended to incorporate:
1. **KF-01 redesigned code obligation (§I5):** (a) Add `CLASS_UID_ENTITY_MANAGEMENT = 3004` constant; (b) add `"entity_management"` and `"inventory_info"` arms in `select_by_class_name`; (c) update `("claroty","audit_log")` AND `("armis","audit_log")` arms in `select()` to 3004; (d) annotate `"audit_activity"` arm as deprecated dead-code post-TOML-change; (e) wire-shape RecordBatch assertion for `class_uid = 3004` (audit_logs) and `class_uid = 5001` (devices); (f) update AC-009 to assert `"entity_management"` string (not `"audit_activity"`); update RG-011 to assert live-string production.
2. **Process-gap warn obligation (§I5):** add `tracing::warn!(event_type = "ocsf.unknown_class_name", ...)` in `spec_driven_adapter.rs::pipeline_result_to_record_batch` on `Err` branch before `.unwrap_or(0)`; add BC-2.16.002 catalog row (coordinate with PO dispatch above).
3. **KF-08..KF-12 TOML corrections** and §E2 copy-text updates (carry-over from v2.4).
4. **§AC-005 mapping tables** must carry v2.4/v2.6 verdicts (not stale VALID for KF-08..KF-12 columns; carry-over from v2.4).
5. **§EC-003 and §EC-009** count reference carry stale §E2/§J copy-text from v2.2 (carry-over from v2.3).
6. **Both stories' subsystem frontmatter:** correct `subsystems_affected` from stale SS-07/SS-12 to SS-01/SS-02/SS-10/SS-16 (or the applicable subset per each story's scope).
7. **In-file doc-table updates at `class_selector.rs` §select() mapping table (lines ~18-37) and §select_by_class_name mapping table (lines ~105-112):** the module-level doc tables must reflect the corrected routing after the code change lands. These are in-file documentation, not spec artifacts — story-writer annotates the obligation; implementer executes.
Routing: orchestrator dispatches story-writer for the amendment.

## Alternatives Considered

- **Option 1 — Dotted Arrow names with double-quoted DataFusion identifiers:** Arrow field name
  `finding.uid`, SQL `SELECT "finding.uid" FROM t`. Rejected because LLM agents copy names from
  `prism_describe` without quoting, producing consistent DataFusion qualified-name parse errors;
  and because the PrismQL Chumsky pipe-mode parser segments on dots, requiring grammar and SQL
  emitter changes to produce delimited identifiers. See §C2 for full analysis.

- **Option 2 — Backtick-quoted identifiers:** Arrow field name `finding.uid`, PrismQL query using
  backtick-delimited names. Rejected for the same agent-ergonomics reason as Option 1, plus
  requiring a grammar change to add backtick-quoted identifier support to the Chumsky parser's
  `ident_char` filter.

- **Option 3 — Projection/alias layer:** Arrow field names remain as `col.name`; a DataFusion view
  maps each to an OCSF alias. Rejected because this is effectively Interpretation B with a view
  layer — the underlying data model stays `col.name`-keyed and cross-sensor joins on OCSF paths
  would require view-level de-aliasing that does not generalize cleanly.

- **Interpretation B (v1.0 decision, superseded):** Use `col.name` as Arrow field name; treat
  `ocsf_field` as semantic metadata for `prism_describe` description only. Superseded by human
  override 2026-08-12 (§A5). The DataFusion constraint analysis from v1.0 was accurate; the
  override changes the delivery strategy, not the validity of that analysis.

## Source / Origin

- **BC-2.01.013 EC-01-025** (v1.8, added D-924 burst 2026-05-31): marks "ColumnMapper step is
  missing" as NON-CONFORMANT — the conformance rule that documented the gap between
  `ColumnMapper::map_record` and `pipeline_result_to_record_batch`.
- **BC-2.16.003** (Column-to-OCSF Mapping contract): specifies that `ocsf_field` declarations
  produce queryable column names, establishing the Interpretation A obligation.
- **Human override 2026-08-12** (§A5): the authoritative decision event; reverses v1.0
  Interpretation B and establishes the delivery constraints (per-sensor flag, Claroty-first,
  DTU migration deferred).
- **Live Claroty API fidelity investigation (LIVE-DRIFT series):** the empirical context showing
  that `ColumnMapper::coerce_value` produced correct normalization but its output was never
  reaching the Arrow RecordBatch because `pipeline_result_to_record_batch` bypassed it.

---

## §Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 2.7 | 2026-08-16 | architect | Adversary pass-2 fix-burst. F1 [MED]: `ocsf.unknown_class_name` emission spec corrected to match BC-2.16.002 catalog row 94 as authoritative source of truth. (a) Field schema: added `table_name` to §I5 emission snippet (`tracing::warn!` now specifies `ocsf_class`, `sensor_id`, `table_name`) and to §Status BC-2.16.002 PO-handoff field list (three fields, not two). (b) Recurrence: §Status BC-2.16.002 PO-handoff corrected from "per-record" to "per table batch (not per record)" — `class_uid` is derived once per batch in `pipeline_result_to_record_batch` before the record loop, consistent with live code and BC-2.16.002 row 94. F8 [LOW]: Phantom PO-handoff obligation removed — §Status BC-2.16.003 §Architecture Anchors bullet no longer instructs updating "subsystem anchors from stale SS-07/SS-12"; BC-2.16.003 has no SS-07/SS-12 references (its `subsystem:` frontmatter is single-valued SS-16); the real SS-07/SS-12 mis-anchoring lived in the stories and ADR frontmatter, both already corrected in v2.6. TD-VSDD-097: (1) sibling pair — no ADR twin; N/A; (2) downstream copy target — §Status story-writer handoff process-gap item 2 remains accurate (references §I5 for the obligation text; §I5 is the source, §Status is a pointer — no independent copy); (3) mandate anchor — no new MUST statements; all obligations remain anchored to `S-ADR058-OCSF-ROUTING-001` per §I5. |
| 2.6 | 2026-08-16 | architect | Adversary pass-1 fix-burst. (A) frontmatter: `subsystems_affected` corrected from stale [SS-07, SS-12] to [SS-01, SS-02, SS-10, SS-16] per ARCH-INDEX Subsystem Registry (SS-07 = Adapter Pagination & Response Cache; SS-12 = Scheduler — neither is an ADR-058 subject; correct: SS-01 Sensor Adapters, SS-02 OCSF Normalization, SS-10 MCP Interface, SS-16 Spec Engine). (B) §K5 path-liveness determination added: Path A (`spec_driven_adapter.rs::pipeline_result_to_record_batch` → `select_by_class_name`) is the ONLY live production path; Path B (`normalize_with_mappers`) has zero production callers confirmed by `test_adapter_normalization.rs` module-level comment (§Production caller status OBS-003). (C) §K5 Divergence 3 amended: `note→comment` data-loss rationale reframed as moot on current production path (Path B not wired); production defect is `class_uid = 3001` in Arrow output on Path A; both Path A and Path B fix obligations recorded. (D) §K5 sibling note corrected: both `("claroty","audit_log")` (primary subject, HIGH forward-compat) AND `("armis","audit_log")` (same defect) named; v2.4 named only Armis. (E) §I5 KF-01 code obligation redesigned: four sub-obligations (a) add `CLASS_UID_ENTITY_MANAGEMENT = 3004`; (b) add `"entity_management"` + `"inventory_info"` arms in `select_by_class_name` (prevents KF-02 regression to 0); (c) update both `("claroty","audit_log")` and `("armis","audit_log")` arms in `select()` to 3004; (d) annotate `"audit_activity"` arm as deprecated dead-code. Wire-shape RecordBatch assertion obligation added for `class_uid` column (3004/5001). AC-009/RG-011 must assert live TOML strings. (F) §I5 process-gap obligation added: `tracing::warn!(event_type = "ocsf.unknown_class_name", ...)` on `Err` branch before `.unwrap_or(0)` in `pipeline_result_to_record_batch`; SAP-1 BC-2.16.002 catalog row required. (G) §I5 out-of-perimeter note added: broader resolver audit for `"device_inventory_info"` et al. deferred. (H) §Status retitled v2.6; PO handoff obligations added (BC-2.16.003 §Architecture Anchors, EC-016-013-023/024, BC-2.16.002 `ocsf.unknown_class_name` catalog row); story-writer handoff consolidated and expanded to cover v2.6 redesign. TD-VSDD-097: (1) sibling pair — §K5 sibling note now names both Claroty and Armis `audit_log` arms explicitly; (2) downstream copy target — §Status story-writer handoff expanded to cover §I5 redesign changes; (3) mandate anchor — all new obligations anchored to `S-ADR058-OCSF-ROUTING-001` per §I5. |
| 2.5 | 2026-08-16 | architect | Consistency-validator fix-burst. HIGH-001: KF-06 settled value `device.type_label` back-propagated to three stale spots in ADR-058 — §E2 devices table (ocsf_field and Arrow name corrected, TBD annotation removed), §J1 eight-name enumeration (`device_type_name`→`device_type_label`), §J3 shadow-check table and analysis text (`device_type_name`→`device_type_label`, TBD note dropped). Zero `device_type_name` residue in prescriptive positions (historical diagnostic references in §J1 shadow table, §K3 WRONG verdict row retained as correct defect descriptions). §I5 KF-06 updated from PO-decision language to settled value; §K4 KF-06 corrected value updated from "PO decision" to `device.type_label`. MED-002: anchor_stories SAC-2 fix — added S-OCSF-FIDELITY-CROWDSTRIKE-001, S-OCSF-FIDELITY-CYBERINT-001, S-OCSF-FIDELITY-ARMIS-001 (all three cite ADR-058 §K in §Authority). LOW-003: §Status volatile version pins removed per POL-39 — S-ADR058-OCSF-COERCION-001 and S-ADR058-OCSF-ROUTING-001 now cited by ID + status only, no version numbers. TD-VSDD-097: (1) sibling pair — HIGH-001 IS the dim-2 downstream-copy miss from v2.4 §I5 (§E2 and §J3 were copy targets of the §I5 KF-06 decision; both swept and corrected in this burst); (2) downstream copy target — no new copy targets introduced; (3) mandate anchor — no new MUST statements; KF-06 obligations remain anchored to S-ADR058-OCSF-ROUTING-001 per §I5. |
| 2.4 | 2026-08-16 | architect | Divergence adjudication pass (independent research-agent cross-validation). 5 additional CONFIRMED-DEFECT findings (KF-08..KF-12). (A) §K3 alerts table reclassified: `category→class_name` (KF-08), `alert_type_name→type_name` (KF-09), `devices_count→count` (KF-10), `updated_time→end_time` (KF-12) from VALID to WRONG (semantic). (B) §K3 audit_logs reclassified: `category→category_name` (KF-11) from VALID to WRONG (semantic). (C) §K4 extended with KF-08..KF-12 rows; total WRONG count 7→12 of 35. (D) §K5 added: Div-1 CONFIRMED (KF-08..KF-11 — normalizer does not recompute OCSF metadata fields; spec_driven writes vendor values into reserved slots); Div-2 NOT-A-DEFECT (inventory_info class-level risk_score confirmed in schema; research-agent claim was incorrect); Div-3 CONFIRMED code defect (class_selector.rs maps audit_activity→CLASS_UID_ACCOUNT_CHANGE=3001; account_change lacks comment; note→comment fails silently; entity_management=3004 correct); Div-4 mixed (KF-12 CONFIRMED; asset_id/device_category/os_category NOT-A-DEFECT; retired→status_code NOTE-ONLY). (E) §K2 KF-01 code defect note added. (F) §I5 extended: KF-01 class_selector.rs code obligation + Armis sibling sweep; KF-08..KF-12 TOML/removal obligations. (G) §E2 updated: KF annotations on 5 affected columns. (H) Status retitled v2.4. TD-VSDD-097: (1) sibling pair — Armis audit_log→CLASS_UID_ACCOUNT_CHANGE carries same defect; documented in §I5 KF-01 code obligation; (2) downstream copy target — S-ADR058-OCSF-ROUTING-001 §AC-005 mapping tables carry stale VALID for KF-08..KF-11 columns; story-writer amendment required per §Status v2.4; (3) mandate anchor — KF-08..KF-12 and KF-01 code change obligations anchored to S-ADR058-OCSF-ROUTING-001 per §I5; companion story may be needed for KF-01 code change if scope requires separation. |
| 2.3 | 2026-08-16 | architect | OCSF v1.7.0 schema validation — all 35 ocsf_class and ocsf_field declarations validated against committed schema (BC-2.02.009 pinned version). 7 WRONG declarations identified. (A) §K added: §K1 methodology, §K2 class verdicts, §K3 per-field path verdicts for all 31 ocsf_field declarations, §K4 finding summary (KF-01 through KF-07). (B) §E2 updated: alerts table — id ocsf_field corrected finding.uid → finding_info.uid per KF-03 (Arrow name finding_uid → finding_info_uid); alert_name ocsf_field corrected finding.title → finding_info.title per KF-04 (Arrow finding_title → finding_info_title); per-table ocsf_class validity annotations added. audit_logs table — KF-01 header note (audit_activity → entity_management); id row KF-05 note (activity_uid absent). devices table — KF-02 header note (device → inventory_info); device_type row KF-06 note (device.type_name absent). device_alert_relations — alert_id corrected finding.uid → finding_info.uid per KF-07. (C) §J3 device_type ocsf_field column annotated KF-06. (D) §J5 alert_id row corrected to finding_info.uid per KF-07. (E) §C3, §G, Consequences Positive and Negative: Arrow-name examples corrected finding_uid → finding_info_uid, finding_title → finding_info_title. (F) §I5 added: 7-item TOML correction obligations for KF-01 through KF-07. (G) Status updated to v2.3 with story-writer amendment obligations. TD-VSDD-097 three-dimension sweep: (1) Sibling pair — no architectural twin ADR; N/A. (2) Downstream copy target — S-ADR058-OCSF-ROUTING-001 §EC-003, §AC-005 mapping tables, §EC-009 count reference carry copy-text now stale vs v2.3; story-writer amendment required and noted in §Status v2.3. (3) Mandate anchor — no new MUST statements introduced; existing §J2 mandate anchor (→ S-ADR058-OCSF-ROUTING-001 §RG-010) unchanged. |
| 2.2 | 2026-08-16 | architect | Ground-truth realignment against claroty.sensor.toml at develop HEAD. (A) §D1 claroty ocsf_field count corrected 32 → 31 (four-table ground-truth: alerts 9 + audit_logs 8 + devices 8 + device_alert_relations 6 = 31; input-hash updated e68fa9a). (B) §E2 fully rewritten: (B1) alerts table corrected — `alert_name` carries `ocsf_field = "finding.title"` (not `alert_class`, which has no ocsf_field); `alert_class` and `ot_devices_count` documented as raw_extensions; 9-column inventory replaces stale 8-column table. (B2) audit_logs table expanded to all 8 ocsf_field mappings — `user_display_name → actor.user.name → actor_user_name` replaces wrong `username → actor.user.name`; `username → actor.user.uid → actor_user_uid` added. (B3) devices table expanded to all 8 ocsf_field mappings (adds `device_name → device.name → device_name` and `os_category → device.os.name → device_os_name`). (B4) device_alert_relations table added (fourth table, absent from v2.1, 6 ocsf_field declarations). (C) §J1 updated: "three tables" → "four tables"; devices ocsf_field count 6 → 8; total count 19 → 31. (D) §J3 shadow check table expanded to 8 columns; shadow check passage confirmed for `device_name` (self-match) and `os_category` (no col.name match); RG-009 and RG-010 verdicts updated. (E) §J4 count corrected 19 → 31 with four-table breakdown. (F) §J5 added: device_alert_relations shadow analysis — all 6 ocsf_field columns verified clean against 10 col.names. (G) Status section retitled v2.2; story-writer amendment obligation noted. TD-VSDD-097 three-dimension sweep: (1) Sibling pair — no architectural twin ADR identified; N/A. (2) Downstream copy target — S-ADR058-OCSF-ROUTING-001 §EC-003, §AC-005 mapping tables, and §EC-009 count reference carry stale §E2/§J copy-text; story-writer amendment required. (3) Mandate anchor — no new MUST statements introduced; existing §J2 mandate anchor (→ S-ADR058-OCSF-ROUTING-001 §RG-010) unchanged. |
| 2.1 | 2026-08-12 | architect | Human-routed amendment (5 mechanical + 1 architectural adjudication). (A) §D2 ANCHOR-NEEDED discharged: anchored to S-ADR058-OCSF-ROUTING-001 AC-001, RG-001/RG-002. (B) §H ANCHOR-NEEDED discharged: anchored to S-ADR058-OCSF-COERCION-001 AC-004, RG-005. (C) anchor_stories populated: S-ADR058-OCSF-COERCION-001 and S-ADR058-OCSF-ROUTING-001 (SAC-2; stale "verified empty" comment block deleted). (D) §"Status as of v2.0" corrected and retitled v2.1: both stories now exist at v1.1 draft; false "story does not yet exist" claim and dangling §D2 ANCHOR-NEEDED cross-reference removed — both were dimension-2 failures (story-writer TD-VSDD-097 report covered only (A)–(C); (D) and the cross-reference were falsified by the same correction burst but not swept). (E) §J added: flag-transition name shadowing adjudication — normative fail-closed rule (§J2, mandate anchored to S-ADR058-OCSF-ROUTING-001 RG-010 amendment pending story-writer dispatch); Claroty devices table device_category ocsf_field changed "device.type" → "device.type_category" (§J3, fits Stage 2 AC-005 scope, zero flag=false blast); ocsf_field count correction 20 → 19 (§J4). input-hash updated to 8bd973b (input drift from original 0514b6b). |
| 2.0 | 2026-08-12 | architect | Human override 2026-08-12: reverses Interpretation B decision. New decision: Interpretation A with underscore-flattened Arrow field names, per-sensor TOML flag, Claroty-first. Adds §C quoting convention analysis (Option 4 chosen), §D per-sensor scoping, §E blast radius (just check stays green), §F DTU generator correction (generators need no changes — earlier claim wrong), §G prism_describe output spec, §H Stage 1 confirmed in scope. Tombstones v1.0 §B decision. |
| 1.0 | 2026-08-11 | architect | Initial authorship — adjudicates col.name vs ocsf_field as v1 Arrow field identifier; documents two-stage resolution path for ColumnMapper wiring gap (human-directed 2026-08-11). SUPERSEDED 2026-08-12 by v2.0 human override. |
