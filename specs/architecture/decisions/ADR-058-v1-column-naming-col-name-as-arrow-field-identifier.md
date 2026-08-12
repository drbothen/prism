---
document_type: adr
adr_id: "ADR-058"
title: "v1 Column Naming: OCSF Field-Path Routing with Underscore-Flattened Arrow Names; DTU Migration Deferred"
status: accepted
date: "2026-08-11"
modified: "2026-08-12"
version: "2.1"
producer: architect
subsystems_affected: [SS-07, SS-12]
supersedes: null
superseded_by: null
amends: null
anchor_stories:
  - S-ADR058-OCSF-COERCION-001
  - S-ADR058-OCSF-ROUTING-001
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
input-hash: "8bd973b"
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
using `SELECT id FROM claroty_alerts` must become `SELECT finding_uid FROM claroty_alerts`.
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
`ocsf_field` on at least some columns: claroty=32, crowdstrike=17, armis=21, cyberint=12
declarations). That would break integration tests for CrowdStrike and others that assert on
`col.name` values.

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

Claroty `alerts` table — columns whose Arrow names change:

| col.name | ocsf_field | Arrow name under Interp. A |
|----------|-----------|---------------------------|
| `id` | `finding.uid` | `finding_uid` |
| `alert_type_name` | `type_name` | `type_name` (unchanged) |
| `category` | `class_name` | `class_name` |
| `status` | `status` | `status` (unchanged) |
| `detected_time` | `time` | `time` |
| `updated_time` | `end_time` | `end_time` |
| `devices_count` | `count` | `count` |
| `description` | `message` | `message` |
| `alert_class` | `finding.title` | `finding_title` |

Claroty `audit_logs` table — notable dotted paths:
| col.name | ocsf_field | Arrow name |
|----------|-----------|------------|
| `username` | `actor.user.name` | `actor_user_name` |

Columns with single-segment ocsf_field (no dot) already have the same Arrow name whether using
Interpretation A or B for that field (e.g., `status` → `status`).

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

For LLM agents: the agent reads `name: "finding_uid"` and uses it verbatim in queries. The
`description: "finding.uid"` field provides OCSF semantic context without being required in
queries.

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

---

## §J Flag-Transition Name Shadowing (Architectural Adjudication, v2.1)

### §J1 The Defect Class

RG-009 (S-ADR058-OCSF-ROUTING-001) guards against intra-table flattened-name duplicates: two
columns whose `ocsf_field` values flatten to the same Arrow field name when
`ocsf_column_naming = true`. The Claroty TOML has no intra-table flattened duplicate; RG-009
passes cleanly. The `devices` table produces six distinct flattened names: `device_uid`,
`device_instance_uid`, `device_type`, `device_type_name`, `risk_score`, `status_code`.
Cross-table `time` values (`alerts.detected_time` and `audit_logs.timestamp`) are cross-table
and harmless. The total `ocsf_field` count across all three Claroty tables is **19** (alerts: 8,
audit_logs: 5, devices: 6) — see §J4.

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

| col.name | ocsf_field | Arrow name (flag=true) |
|---|---|---|
| `uid` | `device.uid` | `device_uid` |
| `asset_id` | `device.instance_uid` | `device_instance_uid` |
| `device_category` | `device.type_category` | `device_type_category` |
| `device_type` | `device.type_name` | `device_type_name` |
| `risk_score` | `risk_score` | `risk_score` |
| `retired` | `status_code` | `status_code` |

Shadow check after fix: `device_type_category` vs col.names (`uid`, `asset_id`,
`device_category`, `device_type`, `risk_score`, `retired`) — no match. `device_type_name` vs
col.names — no match. No flag-transition shadow remains. RG-009 passes (all six flattened names
are distinct). RG-010 (see §J2) passes (no flattened name equals another column's col.name).

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
  from the subcategory `device_type_name` and is self-describing.

**Blast radius:** zero under flag=false (col.name unchanged); under flag=true the Arrow field for
high-level category becomes `device_type_category` rather than `device_type` — but flag=true
has not shipped, so no production queries break.

**Stage 2 scope:** This fix fits within S-ADR058-OCSF-ROUTING-001. AC-005 already modifies
`claroty.sensor.toml` to add `ocsf_column_naming = true`. Adding the `device_category`
`ocsf_field` change to that same TOML edit requires no new story, no new AC, and no new
machinery. Story-writer's amendment to S-ADR058-OCSF-ROUTING-001 (dispatched by human) covers
both the TOML fix and RG-010 for the extended collision check.

### §J4 `ocsf_field` Count Correction

S-ADR058-OCSF-ROUTING-001 EC-009 note cited "20 ocsf_field values" across the three Claroty
tables. The correct count is **19**: alerts (8) + audit_logs (5) + devices (6) = 19. No story
amendment is required for the count alone — the behavioral correctness of the collision
detection is verified by construction via RG-009, not by count.

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
- LLM agents querying Claroty can use OCSF-semantic column names (`finding_uid`, `class_name`)
  without any quoting ceremony.
- Cross-sensor joins on OCSF-normalized field names (`finding_uid` from any sensor with
  `ocsf_field = "finding.uid"`) become possible once other sensors are migrated.
- PrismQL grammar requires no changes — underscore identifiers are already valid.
- DataFusion query planning requires no changes — standard column identifiers.

### Negative / Trade-offs

- Existing Claroty queries using `col.name` values (`id`, `username`, `alert_class`) break
  under Interpretation A and must be rewritten to OCSF-flattened names (`finding_uid`,
  `actor_user_name`, `finding_title`). See §E2 for the full mapping table.
- A `raw_extensions` JSON blob column holds unmapped columns; it is queryable as a column but
  nested keys are not filterable without JSON path functions.
- BC-2.01.013, BC-2.16.003, and BC-2.16.002 each require product-owner amendment after Stage 2
  ships (see §I3 for the full amendment obligation list).

### Status as of v2.1 (2026-08-12)

Decision accepted. Stage 1 (coercion fixes, `column_coercion_failure` emission) is implemented by
`S-ADR058-OCSF-COERCION-001` (v1.1, status: draft; mandate anchor discharged at §H). Stage 2
(OCSF routing wiring — `ocsf_column_naming` flag, `ocsf_field_to_arrow_name` helper,
`pipeline_result_to_record_batch` update) is implemented by `S-ADR058-OCSF-ROUTING-001` (v1.1,
status: draft; mandate anchor discharged at §D2). Both stories are anchored. See §J for the
flag-transition name shadowing adjudication that further constrains Claroty activation scope
within Stage 2 — `ocsf_column_naming = true` cannot ship until the `devices` table collision
is resolved per §J3.

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
| 2.1 | 2026-08-12 | architect | Human-routed amendment (5 mechanical + 1 architectural adjudication). (A) §D2 ANCHOR-NEEDED discharged: anchored to S-ADR058-OCSF-ROUTING-001 AC-001, RG-001/RG-002. (B) §H ANCHOR-NEEDED discharged: anchored to S-ADR058-OCSF-COERCION-001 AC-004, RG-005. (C) anchor_stories populated: S-ADR058-OCSF-COERCION-001 and S-ADR058-OCSF-ROUTING-001 (SAC-2; stale "verified empty" comment block deleted). (D) §"Status as of v2.0" corrected and retitled v2.1: both stories now exist at v1.1 draft; false "story does not yet exist" claim and dangling §D2 ANCHOR-NEEDED cross-reference removed — both were dimension-2 failures (story-writer TD-VSDD-097 report covered only (A)–(C); (D) and the cross-reference were falsified by the same correction burst but not swept). (E) §J added: flag-transition name shadowing adjudication — normative fail-closed rule (§J2, mandate anchored to S-ADR058-OCSF-ROUTING-001 RG-010 amendment pending story-writer dispatch); Claroty devices table device_category ocsf_field changed "device.type" → "device.type_category" (§J3, fits Stage 2 AC-005 scope, zero flag=false blast); ocsf_field count correction 20 → 19 (§J4). input-hash updated to 8bd973b (input drift from original 0514b6b). |
| 2.0 | 2026-08-12 | architect | Human override 2026-08-12: reverses Interpretation B decision. New decision: Interpretation A with underscore-flattened Arrow field names, per-sensor TOML flag, Claroty-first. Adds §C quoting convention analysis (Option 4 chosen), §D per-sensor scoping, §E blast radius (just check stays green), §F DTU generator correction (generators need no changes — earlier claim wrong), §G prism_describe output spec, §H Stage 1 confirmed in scope. Tombstones v1.0 §B decision. |
| 1.0 | 2026-08-11 | architect | Initial authorship — adjudicates col.name vs ocsf_field as v1 Arrow field identifier; documents two-stage resolution path for ColumnMapper wiring gap (human-directed 2026-08-11). SUPERSEDED 2026-08-12 by v2.0 human override. |
