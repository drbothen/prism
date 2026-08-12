---
document_type: story
story_id: S-ADR058-OCSF-ROUTING-001
title: "ADR-058 Stage 2 — OCSF Field-Name Routing: ocsf_column_naming Flag, Underscore-Flattened Arrow Names, Claroty Activation"
version: "1.2"
level: "L4"
status: draft
producer: story-writer
timestamp: "2026-08-12T00:00:00Z"
phase: 3
wave: claroty-live
epic_id: EPIC-OCSF-ROUTING
priority: P1
points: 8
tdd_mode: strict
target_module: prism-spec-engine
subsystems:
  - SS-07
  - SS-12
  - SS-16
# Subsystem anchor justifications:
#   SS-07 (Spec Engine) owns this story's scope because `prism-spec-engine::spec_parser`
#     (`SensorSpec` struct) gains the new `ocsf_column_naming` field, and
#     `prism-spec-engine::column_mapping` (`ColumnMapper::map_record`) is in SS-07 per
#     ARCH-INDEX. The flag is parsed and propagated within the spec engine boundary.
#   SS-12 (Sensor Adapters / DTU) owns this story's scope because
#     `prism-bin::spec_driven_adapter` (`pipeline_result_to_record_batch`, `build_column_array`,
#     `ocsf_field_to_arrow_name` helper) and `claroty.sensor.toml` (in `prism-sensors/specs/`)
#     are both in the sensor adapter subsystem per ADR-058 `subsystems_affected: [SS-07, SS-12]`.
#   SS-16 owns this story's scope because BC-2.16.003 (the governing contract for OCSF column
#     routing) is assigned to SS-16, and `prism-mcp::tools::prism_describe` (ColumnDescriptor
#     naming change per ADR-058 §G) falls within the spec-driven pipeline surface governed by
#     BC-2.16.002/BC-2.16.003 in SS-16.
crates_touched:
  - prism-spec-engine
  - prism-bin
  - prism-mcp
  - prism-sensors
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.003
  - BC-2.01.013
verification_properties:
  - VP-017
  - VP-016
holdout_scenarios: []
depends_on:
  - S-ADR058-OCSF-COERCION-001
# depends_on justification: S-ADR058-OCSF-ROUTING-001 depends on S-ADR058-OCSF-COERCION-001
# because enabling ocsf_column_naming=true for Claroty activates the build_column_array
# path for all Claroty rows. EC-016-013-007/008 (String column + Array/Object input)
# produce silent data corruption — the wildcard to_string() path — unless Stage 1's
# fixes are in production first. Stage 1 must land before Stage 2.
blocks:
  - S-ADR058-DTU-PARITY-MIGRATION-001
# blocks justification: S-ADR058-DTU-PARITY-MIGRATION-001 depends on Stage 2 being in
# production so that its tests can assert ocsf_field column names in the Arrow schema.
# Without Stage 2, the Arrow schema still uses col.name — the parity migration tests
# cannot pass against the pre-Stage-2 production path.
estimated_days: 3
risk: MEDIUM
# Risk justification: The flag mechanism and underscore-flattening helper are low risk.
# The pipeline_result_to_record_batch branch is moderate risk — it changes the Arrow
# schema for all Claroty queries, breaking any existing SQL that uses col.name (e.g.,
# `SELECT id FROM claroty_alerts` must become `SELECT finding_uid FROM claroty_alerts`).
# The existing e2e test is #[ignore]'d so just check stays green, but live Claroty
# users must rewrite queries. Scoped to Claroty only; CrowdStrike/Armis/Cyberint
# are unaffected (default false).
assumption_validations: []
risk_mitigations: []
cycle: "v1.0.0-brownfield"
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - "crates/prism-mcp/src/tools/prism_describe.rs"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "7acfc89"
traces_to:
  - "BC-2.16.003"
  - "BC-2.01.013"
tags:
  - ocsf-routing
  - adr-058
  - stage2
  - ocsf_column_naming
  - underscore-flattening
  - claroty-live
  - pipeline_result_to_record_batch
  - prism_describe
---

# S-ADR058-OCSF-ROUTING-001: ADR-058 Stage 2 — OCSF Field-Name Routing

## Authority

**ADR-058 v2.1: v1 Column Naming — OCSF Field-Path Routing with Underscore-Flattened Arrow
Names; DTU Migration Deferred.** Version `2.1`, status: accepted (2026-08-12). Read
§B2 (decision), §C (quoting convention — Option 4 chosen), §D (per-sensor scoping, flag
mechanism), §E (blast radius), §G (prism_describe output spec), §H (Stage 1 confirmed
separate), §I (implementation guidance), and **§J1–§J4 (flag-transition name shadowing
adjudication, normative fail-closed rule, Claroty `devices` table resolution,
`ocsf_field` count correction 20→19 — v2.1 only)** in full before implementing.
Path: `.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`.

**BC-2.16.003: Column-to-OCSF Mapping at Query Time.** Version `1.4`, status: draft
(modified 2026-08-11). §Column Routing postconditions govern the obligation that
`ocsf_field` declarations produce queryable Arrow field identifiers. This story brings
the production path into conformance with those postconditions for Claroty.
Path: `.factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md`.

**BC-2.01.013: DataSource Trait Adapter Pattern.** Version `1.16`, status: active.
EC-01-025 records "ColumnMapper step is missing" as NON-CONFORMANT. Stage 2 resolves
EC-01-025 for Claroty per ADR-058 §B2 item 4 (OCSF field names now appear in the Arrow
schema for the flagged sensor).
Path: `.factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md`.

---

## Narrative

As a Prism LLM agent querying Claroty sensor data, I want sensor columns to have
OCSF-semantic Arrow field names (e.g., `finding_uid` for `ocsf_field = "finding.uid"`),
so that I can use the column names returned by `prism_describe` verbatim in PrismQL
queries without any quoting ceremony, and cross-sensor joins on shared OCSF field names
work correctly.

---

## ADR-058 MUST Discharge: Mandate Anchor #1

**ADR-058 v2.0 §D2 carries an `ANCHOR-NEEDED` annotation (TD-VSDD-097 dim-3 obligation):**
> "MUST to add this [ocsf_column_naming] field — unanchored per TD-VSDD-097"
> (Stage 2 OCSF routing story does not yet exist)

**This story discharges that mandate.** The mandate anchor is:

| MUST Statement | Story | AC | Red Gate Test |
|---|---|---|---|
| `ocsf_column_naming: bool` field MUST be added to `SensorSpec` with `#[serde(default)]` (ADR-058 §D2) | S-ADR058-OCSF-ROUTING-001 | AC-001 | RG-001, RG-002 |
| `pipeline_result_to_record_batch` MUST check, when `ocsf_column_naming == true`, that no flattened `ocsf_field` name equals a DIFFERENT column's `col.name` in the same table (`A ≠ B` exclusion), fail-closed (ADR-058 §J2) | S-ADR058-OCSF-ROUTING-001 | EC-010, T-21 (shadow check extension) | RG-010 |

**Architect routing obligation:** After this story reaches `status: ready`, the architect
MUST update ADR-058 §D2 (already anchored in v2.1) and verify that the §J2 mandate anchor
row (`S-ADR058-OCSF-ROUTING-001 RG-010`) is present in ADR-058 v2.1 §J2.

---

## ColumnMapper::map_record Wiring Gap — Explicit Scoping Decision

**ADR-058 §A1 / BC-2.01.013 EC-01-025 KNOWN ARCHITECTURAL FINDING:** `ColumnMapper::map_record`
in `prism-spec-engine::column_mapping` has ZERO non-test callers in production. The function
produces `MappingResult::mapped_fields` keyed by `ocsf_field` path strings for the
OcsfEvent/protobuf data flow. This wiring gap was documented in D-924 and confirmed in
STATE.md D-2101 item (1).

**Stage 2 scope decision (explicit, per Canonical Principle Rule 6):**

Stage 2 does NOT wire `ColumnMapper::map_record` into `pipeline_result_to_record_batch`.
The two paths serve different purposes:

- `pipeline_result_to_record_batch` (Arrow RecordBatch path) — the DataFusion query
  surface. Stage 2 changes the Arrow schema field names to `ocsf_field_to_arrow_name(col)`
  via a direct call in `pipeline_result_to_record_batch`. This is the mechanism ADR-058
  §I1 specifies.
- `ColumnMapper::map_record` (OcsfEvent/protobuf path) — produces native protobuf output
  for consumers that decode OCSF protobuf events. This path is a separate data flow from
  the Arrow RecordBatch used by DataFusion.

BC-2.01.013 EC-01-025 NON-CONFORMANT annotation resolves for Claroty because the OCSF
field names now appear in the Arrow schema (the query surface) per ADR-058 §B2 item 4.
The full `ColumnMapper::map_record` wiring into the OcsfEvent production path is a
separate future obligation.

**Future story anchor:** Wiring `ColumnMapper::map_record` into the production OcsfEvent
path requires a dedicated story covering: the `prism-ocsf` integration boundary,
`MappingResult::mapped_fields` → DynamicMessage field-setting, and the `raw_extensions`
blob serialization for the protobuf path. No such story exists in the current corpus.
Orchestrator must route story-writer to create this story at appropriate Wave B planning.
Until then, `ColumnMapper::map_record` remains test-only.

---

## Behavioral Contracts

| BC | Version | Status | Relevance |
|----|---------|--------|-----------|
| BC-2.16.003 | v1.4 | draft | §Column Routing postconditions — `ocsf_field` declarations produce queryable Arrow field identifiers (this story brings production into conformance) |
| BC-2.01.013 | v1.16 | active | EC-01-025 NON-CONFORMANT annotation resolved for Claroty after this story merges; product-owner updates annotation |

---

## Red Gate Tests (SAC-1 — tdd_mode: strict)

All ten tests MUST be failing (RED) before any implementation code is written.
Test-writer dispatched FIRST; implementer only after all 10 confirmed failing.

- **RG-001:** `test_sensor_spec_ocsf_column_naming_defaults_to_false` —
  fails until `SensorSpec` gains the `ocsf_column_naming` field (with `#[serde(default)]`).
  A `SensorSpec` deserialized from TOML without the field MUST have
  `ocsf_column_naming == false`. Currently fails at compile time (field does not exist).
  Covers AC-001.

- **RG-002:** `test_sensor_spec_ocsf_column_naming_parses_true_from_toml` —
  fails until `SensorSpec` gains the `ocsf_column_naming` field. A TOML string with
  `ocsf_column_naming = true` MUST deserialize to `ocsf_column_naming == true`.
  Covers AC-001.

- **RG-003:** `test_ocsf_field_to_arrow_name_replaces_dots_with_underscores` —
  fails until `ocsf_field_to_arrow_name` free function is added to
  `prism-bin::spec_driven_adapter`. `ocsf_field_to_arrow_name("finding.uid")` MUST
  return `"finding_uid"`; `ocsf_field_to_arrow_name("actor.user.name")` MUST return
  `"actor_user_name"`. Covers AC-002.

- **RG-004:** `test_ocsf_field_to_arrow_name_single_segment_is_unchanged` —
  fails until the function exists. `ocsf_field_to_arrow_name("status")` MUST return
  `"status"` (no dots, unchanged). Covers AC-002.

- **RG-005:** `test_pipeline_result_to_record_batch_ocsf_flag_true_uses_flattened_names` —
  fails until `pipeline_result_to_record_batch` branches on `sensor_spec.ocsf_column_naming`.
  A `SensorSpec` with `ocsf_column_naming = true` and a column with
  `ocsf_field = Some("finding.uid")` MUST produce an Arrow schema where the field is
  named `"finding_uid"`, not `"id"` or `"finding.uid"`. Covers AC-003.

- **RG-006:** `test_pipeline_result_to_record_batch_ocsf_flag_false_uses_col_name` —
  fails until the conditional branch is implemented. A `SensorSpec` with
  `ocsf_column_naming = false` (or absent) and a column with `name = "id"` and
  `ocsf_field = Some("finding.uid")` MUST still produce an Arrow schema field named
  `"id"` (non-Claroty sensors unaffected). Covers AC-004.

- **RG-007:** `test_prism_describe_ocsf_column_naming_true_returns_flattened_name_and_dotted_description` —
  fails until `prism_describe` branches on `sensor_spec.ocsf_column_naming`. A
  `SensorSpec` with `ocsf_column_naming = true` and a column with `name = "id"`,
  `ocsf_field = Some("finding.uid")` MUST produce a `ColumnDescriptor` with
  `name = "finding_uid"` and `description = "finding.uid"`. Covers AC-006.

- **RG-008:** `test_spec_driven_adapter_columns_without_ocsf_field_go_to_raw_extensions_schema` —
  fails until `pipeline_result_to_record_batch` implements the `raw_extensions` schema
  column when `ocsf_column_naming = true` and a column has `col.ocsf_field == None`.
  The Arrow schema MUST contain a `raw_extensions` field of type `Utf8` instead of an
  individual column for the unmapped column. Covers AC-007.

- **RG-009:** `test_pipeline_result_to_record_batch_ocsf_collision_returns_error` —
  fails until `pipeline_result_to_record_batch` detects duplicate flattened Arrow names
  and returns `Err(ArrowError::SchemaError(...))`. A `SensorSpec` with
  `ocsf_column_naming = true`, column A with `ocsf_field = Some("a.b_c")`, and column B
  with `ocsf_field = Some("a_b.c")` MUST produce an `Err` (not a silently wrong schema
  that returns the wrong column on every query). Without this gate, Arrow 58's
  `Schema::column_with_name` silently returns the first match for duplicate field names
  — the second column is permanently shadow-lost. Covers EC-009.

- **RG-010:** `test_pipeline_result_to_record_batch_ocsf_shadow_collision_returns_error` —
  fails until `pipeline_result_to_record_batch` also detects flag-transition name
  shadowing (ADR-058 §J2): a flattened `ocsf_field` name from one column equalling a
  DIFFERENT column's `col.name` in the same table.

  The test constructs a `SensorSpec` with `ocsf_column_naming = true` and two columns:
  - Column A: `col.name = "device_category"`, `ocsf_field = Some("device.type")` →
    flattens to `"device_type"`
  - Column B: `col.name = "device_type"`, `ocsf_field = Some("device.type_name")`

  Column A's flattened name (`"device_type"`) equals Column B's `col.name` (`"device_type"`),
  where A ≠ B. The function MUST return `Err(ArrowError::SchemaError(...))`.

  **Self-match exclusion assertion (load-bearing):** The test MUST also assert that a
  configuration where a column's flattened name equals its OWN `col.name` (`A = B`,
  the self-match case) does NOT produce an `Err`. For example, a column with
  `col.name = "risk_score"` and `ocsf_field = Some("risk_score")` (single segment,
  flattens to `"risk_score"`) is legal — `"risk_score" ≠ any other column's col.name`
  in Claroty devices (confirmed: `uid`, `asset_id`, `device_category`, `device_type`,
  `retired` — no match). Without this assertion, an over-broad implementation that
  checks `ocsf_field_to_arrow_name(A) ≠ A.col_name` (no `A ≠ B` guard) would reject
  valid production Claroty config. Covers EC-010.

### BC-5.38.001 Density Check

Red Gate test count: **10** (RG-001..RG-010).
Acceptance criteria: 8 (AC-001..AC-008). AC-008 is an `#[ignore]`'d test update — its
Red Gate is RG-005 (same mechanism: Arrow field name must be `device_uid` not `uid`).

Density: 10 RGTs / 8 ACs = **1.25 ≥ 0.5** — compliant with BC-5.38.001.

Note: AC-005 (claroty.sensor.toml change) and AC-008 (e2e test update) are exercised
by existing RGTs rather than dedicated additional tests. RG-009 covers EC-009 (intra-table
flattening collision detection). RG-010 covers EC-010 (flag-transition name shadowing —
flattened ocsf_field name equals a DIFFERENT column's col.name). The density check is
based on the 10 distinct failing tests enumerated above.

---

## Acceptance Criteria

### AC-001: SensorSpec gains ocsf_column_naming field with serde(default)

`SensorSpec` in `prism-spec-engine::spec_parser` has a new field:

```rust
#[serde(default)]
pub ocsf_column_naming: bool,
```

A TOML sensor spec without `ocsf_column_naming` deserializes with `ocsf_column_naming == false`.
A TOML sensor spec with `ocsf_column_naming = true` deserializes with `ocsf_column_naming == true`.
All existing sensor TOML files parse without error (backward compatible — `#[serde(default)]`).

**Serde default basis:** `#[serde(default)]` at the field level uses `bool::default()` =
`false`. `SensorSpec` carries no container-level `#[serde(default)]`; the struct's
manual `Default` impl is irrelevant to deserialization of individual fields. This
guarantee holds only while `SensorSpec` lacks container-level `#[serde(default)]` —
that precondition should not change without revisiting this AC.

**Compile-time note:** `SensorSpec` uses exhaustive struct literals in both
`impl Default for SensorSpec` and `SensorSpec::new()`. Both sites will fail to compile
(E0063: missing field) without explicitly adding `ocsf_column_naming: false`. See T-12
for all three required edit sites.

(traces to BC-2.16.003 postcondition §Column Routing: "columns with an ocsf_field value
are mapped to the corresponding OCSF field" — the flag is the mechanism that activates
this postcondition in the production path)

### AC-002: ocsf_field_to_arrow_name helper correctly flattens dotted OCSF paths

A free function `ocsf_field_to_arrow_name(ocsf_field: &str) -> String` is added to
`prism-bin::spec_driven_adapter`. The function replaces all occurrences of `.` (dot)
in `ocsf_field` with `_` (underscore). Examples:

| ocsf_field | ocsf_field_to_arrow_name result |
|---|---|
| `"finding.uid"` | `"finding_uid"` |
| `"actor.user.name"` | `"actor_user_name"` |
| `"device.hw_info.vendor_name"` | `"device_hw_info_vendor_name"` |
| `"status"` | `"status"` (single segment, unchanged) |
| `"time"` | `"time"` (single segment, unchanged) |

This is the underscore-flattening convention chosen in ADR-058 §C2 Option 4.

**Arrow field-name legality:** Arrow 58 `Field::new` stores names as bare `String` with
no character-set validation beyond valid UTF-8. Underscore-flattened names are
unconditionally legal as Arrow field names in the pinned `arrow` 58.2.0.

**DataFusion SQL identifier legality:** sqlparser `GenericDialect` (used by DataFusion
53.1.0) accepts `_` as both identifier-start and identifier-part. Lowercase
alphanumeric-plus-underscore names (`finding_uid`, `actor_user_name`) are unquoted-legal
SQL identifiers. DataFusion's `enable_ident_normalization` defaults to `true` (folds to
lowercase), which is a no-op for already-lowercase names. Note: SQL keywords `TIME`,
`COUNT`, `STATUS`, `MESSAGE` are keyword tokens but are NOT in sqlparser's
`RESERVED_FOR_IDENTIFIER` set (`EXISTS`/`INTERVAL`/`STRUCT`/`TRIM`), so they parse as
column references unquoted.

**PrismQL-lexer guarantee (stronger argument):** The PrismQL pipe parser
(`prism-query::pipe_parser`) accepts identifiers matching `[A-Za-z_][A-Za-z0-9_]*`.
Underscore-flattened OCSF names satisfy this grammar by construction.
`prism-query::pipe_sql_emitter` quotes only names containing characters outside ASCII
alnum+underscore — all flattened names take the unquoted path. Dotted OCSF paths (e.g.,
`finding.uid`) would be unreachable from PrismQL entirely since `.` is not a valid
PrismQL identifier character, making underscore-flattening the only viable choice from
the PrismQL surface alone.

(traces to BC-2.16.003 postcondition §Column Routing: OCSF field paths become Arrow
field identifiers; the flattening ensures they are valid DataFusion identifiers without
quoting per ADR-058 §C4)

### AC-003: pipeline_result_to_record_batch uses flattened names when ocsf_column_naming=true

`pipeline_result_to_record_batch` in `prism-bin::spec_driven_adapter` computes the Arrow
schema field name for each column using:

```
let arrow_name = if sensor_spec.ocsf_column_naming {
    col.ocsf_field.as_deref()
        .map(ocsf_field_to_arrow_name)
        .unwrap_or_else(|| col.name.clone())
} else {
    col.name.clone()
};
```

This matches ADR-058 §I1 exactly. When `ocsf_column_naming = true` and `col.ocsf_field`
is `Some("finding.uid")`, the Arrow schema field is named `"finding_uid"`.

(traces to BC-2.16.003 postcondition §Column Routing: "columns with an ocsf_field value
are mapped to the corresponding OCSF field" — the `ocsf_field_to_arrow_name` result IS
the Arrow field identifier in the query surface)

### AC-004: pipeline_result_to_record_batch uses col.name when ocsf_column_naming=false

When `sensor_spec.ocsf_column_naming == false` (the default), `pipeline_result_to_record_batch`
uses `col.name` for all Arrow schema field names — identical behavior to the pre-Stage-2
production path. No regression for CrowdStrike, Armis, Cyberint sensors.

(traces to BC-2.01.013 postcondition 1: every spec-declared column survives into the
RecordBatch with the correct type — the col.name behavior is unchanged for non-flagged
sensors)

### AC-005: claroty.sensor.toml has ocsf_column_naming = true AND device_category ocsf_field corrected

`crates/prism-sensors/specs/claroty.sensor.toml` receives TWO changes in the same TOML
edit (both MUST land together — setting the flag without the ocsf_field fix would cause
Claroty to fail closed at runtime under the new EC-010 / §J2 shadow check):

1. `ocsf_column_naming = true` added at the sensor-level (not table-level). After this
   change, all Claroty tables use OCSF-flattened Arrow field names. The Claroty sensor
   is the only sensor with this flag set in Stage 2 (CrowdStrike, Armis, Cyberint TOML
   specs are unchanged).

2. The `device_category` column in the `devices` table has its `ocsf_field` changed from
   `"device.type"` to `"device.type_category"`. This resolves the §J2 shadow collision
   that would otherwise cause `device_category`'s flattened name (`device_type`) to equal
   column `device_type`'s `col.name`. Both the collision and the fix are verified by the
   RG-009 + RG-010 gate pair: after this TOML change, RG-009 passes (all six devices
   flattened names are distinct) and RG-010 passes (no flattened name equals a different
   column's col.name).

   **Blast radius of ocsf_field change:** Zero under `flag=false` (col.name is unchanged;
   DTU extraction uses `r.get("device_category")` — unchanged). Under `flag=true` the
   Arrow field for high-level category becomes `device_type_category` rather than
   `device_type`. Since `flag=true` has not shipped, no production queries break.

Mapping examples per ADR-058 §E2 (Claroty `alerts` table excerpt):

| col.name | ocsf_field | Arrow field name after Stage 2 |
|---|---|---|
| `id` | `finding.uid` | `finding_uid` |
| `alert_class` | `finding.title` | `finding_title` |
| `username` | `actor.user.name` | `actor_user_name` |
| `status` | `status` | `status` (unchanged — no dot) |
| `detected_time` | `time` | `time` (unchanged — no dot) |

Claroty `devices` table — post-fix Arrow names under `flag=true` (per ADR-058 §J3):

| col.name | ocsf_field (post-fix) | Arrow field name (flag=true) |
|---|---|---|
| `uid` | `device.uid` | `device_uid` |
| `asset_id` | `device.instance_uid` | `device_instance_uid` |
| `device_category` | `device.type_category` | `device_type_category` |
| `device_type` | `device.type_name` | `device_type_name` |
| `risk_score` | `risk_score` | `risk_score` (unchanged — no dot; self-match legal) |
| `retired` | `status_code` | `status_code` |

Shadow check after fix: no flattened name equals any other column's col.name. RG-009
passes (all six flattened names distinct). RG-010 passes (zero shadow collisions).

(traces to BC-2.16.003 EC-016-013-012: two sensors both mapping `device_ip →
ocsf_field = "device.ip"` are both queryable as `device_ip` once each sensor enables
the flag; and to BC-2.01.013 EC-01-025 which moves NON-CONFORMANT→CONFORMANT for
Claroty after this story merges)

### AC-006: prism_describe returns flattened name and dotted description for flagged sensors

`ColumnDescriptor` returned by `prism_describe` for a sensor with `ocsf_column_naming = true`:
- `name` = `ocsf_field_to_arrow_name(col.ocsf_field)` (the queryable Arrow identifier,
  e.g., `"finding_uid"`)
- `description` = `col.ocsf_field.clone()` (original dotted OCSF path preserved as
  semantic annotation, e.g., `"finding.uid"`)

For a sensor with `ocsf_column_naming = false`, both fields use their existing behavior
(`name = col.name`, `description = col.ocsf_field`).

This is the exact spec in ADR-058 §G. LLM agents read `name: "finding_uid"` from
`prism_describe` output and use it verbatim in PrismQL queries — no quoting needed.

(traces to BC-2.16.003 postcondition §Column Routing invariant: "declared column_type
in the TOML spec is the authoritative wire shape" — the describe output must reflect the
same identifiers the agent will use in queries)

### AC-007: Columns without ocsf_field are collected into raw_extensions column when flag=true

When `sensor_spec.ocsf_column_naming = true`, any column with `col.ocsf_field == None`
does NOT appear as an individual Arrow schema field. Instead, these columns' values are
collected into a single Arrow `Utf8` field named `"raw_extensions"` containing a
serialized JSON object.

The `raw_extensions` column is queryable via PrismQL: `SELECT raw_extensions FROM claroty_alerts`.

Per ADR-058 §I2: this is the `ColumnMapper::map_record` design intent for unmapped
columns. The implementer MUST verify which Claroty columns currently have
`col.ocsf_field == None` in `claroty.sensor.toml` at dispatch time and confirm they
go to `raw_extensions` rather than being silently dropped.

(traces to BC-2.16.003 postcondition §Column Routing: "Columns without an ocsf_field
mapping are preserved in the raw_extensions JSON blob")

### AC-008: test_BC_2_11_005_e2e_claroty_query_returns_data updated to use device_uid

The `#[ignore]`'d test `test_BC_2_11_005_e2e_claroty_query_returns_data` (in
`crates/prism-bin/tests/` or equivalent e2e test file) is updated to assert
`row.get("device_uid")` instead of `row.get("uid")` (the Claroty devices table `uid`
column has `ocsf_field = "device.uid"` → Arrow name `"device_uid"` after Stage 2).

The test remains `#[ignore]`'d after this update (it requires a live Claroty server or
a running DTU clone per SID-1 rule 4). The `#[ignore]` attribute comment is updated to:
`// Requires live Claroty DTU server; SID-1 dependency: DTU parity tests pending
// S-ADR058-DTU-PARITY-MIGRATION-001`.

The non-`#[ignore]`'d companion unit test RG-005 and RG-006 provide the non-live
verification that the column naming logic branches correctly.

(traces to ADR-058 §D3: "`test_BC_2_11_005_e2e_claroty_query_returns_data`: row.get('uid')
→ row.get('device_uid')")

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Scope |
|-----------|--------|---------------|-------|
| `SensorSpec::ocsf_column_naming` field | `prism-spec-engine::spec_parser` | Pure (data struct) | New field added |
| `ocsf_field_to_arrow_name` | `prism-bin::spec_driven_adapter` | Pure | New free function — no I/O, deterministic string transform |
| `pipeline_result_to_record_batch` | `prism-bin::spec_driven_adapter` | Effectful (Arrow I/O) | Conditional branch added on `sensor_spec.ocsf_column_naming` |
| `build_column_array` `raw_extensions` handling | `prism-bin::spec_driven_adapter` | Pure (data transformation) | New path: columns with `ocsf_field = None` go to `raw_extensions` JSON when flag=true |
| `prism_describe` | `prism-mcp::tools::prism_describe` | Effectful (MCP response) | `ColumnDescriptor.name` sourcing branches on `sensor_spec.ocsf_column_naming` per ADR-058 §G |
| `claroty.sensor.toml` | `prism-sensors/specs/` | Configuration | Add `ocsf_column_naming = true` at sensor level |
| `#[ignore]`'d e2e test | `crates/prism-bin/tests/` | Test (effectful) | Update `row.get("uid")` → `row.get("device_uid")` |

Architecture section files: `architecture/module-decomposition.md` (SS-07, SS-12, SS-16),
`architecture/dependency-graph.md`.

---

## Purity Classification

| Component | Classification | Rationale |
|-----------|---------------|-----------|
| `ocsf_field_to_arrow_name` | Pure | `&str` → `String`; replaces `.` with `_`; deterministic, no I/O |
| `SensorSpec` deserialization | Pure (serde) | Derives `Deserialize`; `#[serde(default)]` is a declarative attribute |
| `pipeline_result_to_record_batch` | Effectful | Calls `RecordBatch::try_new` (Arrow schema validation error path); writes to caller's batch output |
| `prism_describe` | Effectful | Reads `SensorSpec` from registry, constructs MCP response |
| `build_column_array` (raw_extensions path) | Pure (data transformation) | JSON serialization of column values to UTF-8 blob; no I/O |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Column with `ocsf_field = Some("status")` (single segment, no dot) | `ocsf_field_to_arrow_name("status")` = `"status"` — unchanged; not a regression |
| EC-002 | Column with `ocsf_field = None` when `ocsf_column_naming = true` | Value goes to `raw_extensions` JSON blob; no individual Arrow field for this column |
| EC-003 | Claroty `audit_logs` table `username` with `ocsf_field = "actor.user.name"` | Arrow field name = `"actor_user_name"` per ADR-058 §E2 |
| EC-004 | CrowdStrike sensor (no `ocsf_column_naming` flag) queries `col.name` | `ocsf_column_naming` defaults to `false`; Arrow names stay as col.name; no change |
| EC-005 | Two sensors both declare `ocsf_field = "device.ip"` and both enable the flag | Both produce Arrow field `"device_ip"`; cross-sensor JOIN on `device_ip` works per BC-2.16.003 EC-016-013-012 |
| EC-006 | `ocsf_field = "finding.uid"` on a column already named `finding_uid` in `col.name` | `ocsf_field_to_arrow_name("finding.uid")` = `"finding_uid"` = `col.name` — no conflict, degenerate case |
| EC-007 | `raw_extensions` column name conflicts with an existing sensor column named `raw_extensions` | If a sensor column is explicitly named `raw_extensions` and has `ocsf_field != None`, it gets its flattened ocsf_field name; the `raw_extensions` blob column collects only columns with `ocsf_field = None`. No collision on Claroty (no Claroty column is named `raw_extensions`). |
| EC-008 | `just check` after Stage 2 (blast radius per ADR-058 §E1) | Must stay GREEN: spec-driven-adapter unit tests use inline ColumnSpec with `ocsf_field = None` so they get `col.name`; DTU parity tests assert DTU HTTP response JSON (not Arrow schema); `prism_describe` unit tests use inline ColumnDescriptor constructions; CrowdStrike/Armis/Cyberint unaffected; Claroty e2e test is `#[ignore]`'d |
| EC-009 | Two columns in the same table have `ocsf_field` values that flatten to the same Arrow name (e.g., `"a.b_c"` and `"a_b.c"` both → `"a_b_c"` via `ocsf_field_to_arrow_name`) | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed. Arrow 58 does NOT detect duplicate schema field names (`Schema::new` is infallible; `Schema::column_with_name` returns the first match — silent wrong-column resolution for the agent). Current Claroty TOML has no intra-table collision (verified by enumeration of all 19 `ocsf_field` values — ADR-058 §J4 corrects the earlier count of 20). Future sensors must be collision-free before enabling the flag. See RG-009. |
| EC-010 | A flattened `ocsf_field` name from one column equals the `col.name` of a DIFFERENT column in the same table when `ocsf_column_naming = true` (flag-transition name shadowing per ADR-058 §J1/§J2). Example: `device_category` with `ocsf_field = "device.type"` → `device_type`, while column `device_type` has `col.name = "device_type"`. `SELECT device_type FROM claroty_devices` is valid in both flag states but returns different semantic content — high-level category vs type-within-category — with no error and no warning. | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed. The `A ≠ B` self-match exclusion is mandatory: a column whose flattened ocsf_field name equals its own `col.name` (e.g., `risk_score` → `risk_score`) is legal and MUST NOT fail. This collision class is resolved in `claroty.sensor.toml` by changing `device_category`'s ocsf_field from `"device.type"` to `"device.type_category"` (AC-005, same TOML edit). See RG-010. |

---

## Token Budget Estimate

| Source | Estimated tokens |
|--------|-----------------|
| This story spec | ~6k |
| `spec_parser.rs` (SensorSpec struct) | ~4k |
| `spec_driven_adapter.rs` (pipeline_result_to_record_batch + build_column_array) | ~12k |
| `prism_describe.rs` (ColumnDescriptor construction) | ~3k |
| `claroty.sensor.toml` (full TOML spec for context) | ~4k |
| BC-2.16.003 + BC-2.01.013 (governing contracts) | ~5k |
| ADR-058 v2.1 (§B2, §D, §G, §I, §J in full) | ~5k |
| `bc_2_16_003_test.rs` + existing spec_driven_adapter tests (context for new tests) | ~4k |
| Tool outputs (just iter, cargo nextest) | ~1k |
| **Total** | **~44k** |

42.5k tokens is well within a 200k agent context window (~21%). This story does NOT need
splitting. Note: this is at the upper-safe boundary for a single dispatch — the
implementer MUST load only the files listed, not the full architecture directory.

---

## Tasks

### Phase A: Red Gate (test-writer dispatched FIRST — before implementer)

- T-01: Read `spec_parser.rs` `SensorSpec` struct — confirm `ocsf_column_naming` does not exist yet
- T-02: Read `spec_driven_adapter.rs` — confirm no `ocsf_field_to_arrow_name` function; confirm `pipeline_result_to_record_batch` uses `col.name` unconditionally
- T-03: Read `prism_describe.rs` `ColumnDescriptor` construction — confirm current `name` sourcing
- T-04: Write RG-001 — `test_sensor_spec_ocsf_column_naming_defaults_to_false` (MUST FAIL)
- T-05: Write RG-002 — `test_sensor_spec_ocsf_column_naming_parses_true_from_toml` (MUST FAIL)
- T-06: Write RG-003 — `test_ocsf_field_to_arrow_name_replaces_dots_with_underscores` (MUST FAIL)
- T-07: Write RG-004 — `test_ocsf_field_to_arrow_name_single_segment_is_unchanged` (MUST FAIL)
- T-08: Write RG-005 — `test_pipeline_result_to_record_batch_ocsf_flag_true_uses_flattened_names` (MUST FAIL)
- T-09: Write RG-006 — `test_pipeline_result_to_record_batch_ocsf_flag_false_uses_col_name` (MUST FAIL)
- T-10: Write RG-007 — `test_prism_describe_ocsf_column_naming_true_returns_flattened_name_and_dotted_description` (MUST FAIL)
- T-11: Write RG-008 — `test_spec_driven_adapter_columns_without_ocsf_field_go_to_raw_extensions_schema` (MUST FAIL)
- T-11B: Write RG-009 — `test_pipeline_result_to_record_batch_ocsf_collision_returns_error`
  (MUST FAIL before T-21). Build a `SensorSpec` with `ocsf_column_naming = true`,
  two columns where column A has `ocsf_field = Some("a.b_c")` and column B has
  `ocsf_field = Some("a_b.c")`. Call `pipeline_result_to_record_batch` with empty data
  and assert the result is `Err`. (Currently fails because no collision check exists —
  returns `Ok` with a silently wrong schema.)
- T-11C: Write RG-010 — `test_pipeline_result_to_record_batch_ocsf_shadow_collision_returns_error`
  (MUST FAIL before T-21 shadow extension). Build a `SensorSpec` with
  `ocsf_column_naming = true` and two columns:
  - Column A: `col.name = "device_category"`, `ocsf_field = Some("device.type")` →
    flattens to `"device_type"`
  - Column B: `col.name = "device_type"`, `ocsf_field = Some("device.type_name")`
  Call `pipeline_result_to_record_batch` and assert the result is `Err` (A's flattened
  name equals B's col.name, A ≠ B). ALSO assert a SECOND call with a column where
  `col.name = "risk_score"` and `ocsf_field = Some("risk_score")` (self-match A = B)
  returns `Ok` — the self-match exclusion must NOT trigger Err.
  (Currently fails because no shadow check exists — the shadow case returns `Ok`.)
- T-GATE: Run `just iter prism-spec-engine --no-fail-fast` and `just iter prism-bin --no-fail-fast` — confirm RG-001..RG-010 fail with correct compile/test-failure reasons. Confirm no regressions in non-RG tests. Report density: 10/8 = 1.25 ≥ 0.5. STOP and wait for implementer dispatch.

### Phase B: Implementation (implementer dispatched AFTER T-GATE)

- T-12: Add `#[serde(default)] pub ocsf_column_naming: bool` to `SensorSpec` in
  `spec_parser.rs`. The field must have `#[non_exhaustive]` NOT added (it is a plain
  bool field, not a newtype). **ALSO** update the following two exhaustive struct
  literals — both will fail to compile (E0063: missing field) without this update:
  (a) `impl Default for SensorSpec` — add `ocsf_column_naming: false` to the struct
  body (the `fn default()` implementation in `spec_parser.rs`);
  (b) `SensorSpec::new()` — add `ocsf_column_naming: false` to the struct body of the
  `Self { ... }` return expression. Note: `SensorSpec::new()` does not expose
  `ocsf_column_naming` as a parameter (consistent with other optional fields like
  `auth_plugin`, `mode`, `probe_table`). Run `just iter prism-spec-engine`. Makes
  RG-001 and RG-002 green.
- T-13: Add `pub fn ocsf_field_to_arrow_name(ocsf_field: &str) -> String` to
  `spec_driven_adapter.rs`. Implementation: `ocsf_field.replace('.', "_")`. Run
  `just iter prism-bin`. Makes RG-003 and RG-004 green.
- T-14: Update `pipeline_result_to_record_batch` to use the conditional branch per
  ADR-058 §I1 (see §Acceptance Criteria AC-003 for the exact logic). Run
  `just iter prism-bin`. Makes RG-005 and RG-006 green.
- T-15: Update `build_column_array` to handle `raw_extensions` path when
  `sensor_spec.ocsf_column_naming = true` and `col.ocsf_field = None`. Run
  `just iter prism-bin`. Makes RG-008 green.
- T-16: Update `prism_describe` `ColumnDescriptor.name` sourcing per ADR-058 §G.
  Run `just iter prism-mcp`. Makes RG-007 green.
- T-17: Add `ocsf_column_naming = true` to `claroty.sensor.toml` at the sensor level
  (alongside `sensor_id`, `auth_type`, etc.). Run `just iter prism-spec-engine` to
  confirm TOML parses correctly.
- T-18: Update `test_BC_2_11_005_e2e_claroty_query_returns_data` to use `row.get("device_uid")`
  instead of `row.get("uid")`; update the `#[ignore]` comment per AC-008.
- T-19: Run `just iter prism-spec-engine` and `just iter prism-bin` — all 10 RGTs must pass.
- T-20: Run `just check` — full workspace gate. Must stay GREEN per ADR-058 §E1
  blast-radius analysis. If any non-Claroty tests fail, STOP — do not push.
- T-21: In `pipeline_result_to_record_batch`, after computing all arrow names for a
  table when `sensor_spec.ocsf_column_naming = true`, perform a COMBINED collision
  check (both conditions below) in a single pass before building the Arrow schema:

  **(a) Intra-flattened-name duplicate check (existing, makes RG-009 green):**
  Collect all flattened arrow names into a `std::collections::HashSet<&str>`.
  For each name, check `.insert(name.as_str())` — if insert returns `false`
  (duplicate), return `Err(ArrowError::SchemaError(format!("OCSF field flattening
  collision: two columns flatten to Arrow name '{name}'; fix ocsf_field declarations
  in the sensor TOML to produce unique names")))`.

  **(b) Flag-transition shadow check (new per ADR-058 §J2, makes RG-010 green):**
  For each column A with `ocsf_field = Some(...)`, let `flat_A =
  ocsf_field_to_arrow_name(A.ocsf_field)`. For each OTHER column B in the same table
  (where `A ≠ B`), if `flat_A == B.col_name`, return `Err(ArrowError::SchemaError(
  format!("OCSF field-name shadow collision: column '{a_col_name}' flattened ocsf_field
  '{flat_A}' equals col.name of column '{b_col_name}'; change ocsf_field declaration
  to avoid ambiguity")))`.

  **Self-match exclusion (`A ≠ B`) is mandatory:** The check iterates over pairs where
  A and B are DISTINCT columns. A column whose `ocsf_field_to_arrow_name` result equals
  its OWN `col.name` (e.g., `risk_score` with `ocsf_field = "risk_score"`) is legal
  and MUST NOT produce `Err`. Implement as: skip when iterating over B if B is the
  same column as A (compare by column index or identity). Without this guard, valid
  production Claroty config (two columns where ocsf_field flattens to col.name, e.g.,
  `status` → `status`) would be incorrectly rejected.

  This combined pass is a fail-closed gate per ADR-058 §J2. Arrow 58 does NOT detect
  either class of collision; without this check, a shadow collision produces silent
  wrong-column resolution for every query in that flag state. Makes RG-009 and RG-010
  green.

---

## Previous Story Intelligence

This is the second story in EPIC-OCSF-ROUTING (after S-ADR058-OCSF-COERCION-001 which
fixes the coercion gaps). No predecessor Stage 2 implementation exists.

Key lessons from the ADR-058 design process:

1. DTU generators do NOT need changes (ADR-058 §F1 correction). `build_column_array`
   reads raw JSON by `col.name`; Arrow schema field names change but the extraction
   key is unchanged. Do NOT modify DTU generator code.

2. The `ColumnMapper::map_record` wiring gap is explicitly out of scope (see §ColumnMapper
   Wiring Gap section above). Do NOT wire it into `pipeline_result_to_record_batch`.

3. `just check` stays green with per-sensor scoping. The blast-radius analysis in
   ADR-058 §E1 confirms all existing tests remain unaffected. Do NOT apply the flag
   to CrowdStrike, Armis, or Cyberint TOMLs in this story.

4. The quoting convention (underscore-flattening) is finalized in ADR-058 §C2 Option 4.
   Do NOT use dotted Arrow names, double-quoted SQL identifiers, or backtick forms.

---

## Architecture Compliance Rules

From `architecture/module-decomposition.md`, ADR-023, ADR-028, and ADR-058:

1. `ocsf_field_to_arrow_name` MUST live in `prism-bin::spec_driven_adapter`, not in
   `prism-spec-engine`. `prism-bin` imports from `prism-spec-engine`; the reverse
   is forbidden per ADR-023 §D3 crate boundary.

2. `pipeline_result_to_record_batch` MUST NOT call `ColumnMapper::map_record`
   (see §ColumnMapper Wiring Gap section). The Arrow field name is computed by
   `ocsf_field_to_arrow_name`, not by `ColumnMapper`.

3. `SensorSpec::ocsf_column_naming` MUST use `#[serde(default)]` (not `Option<bool>`).
   Per ADR-058 §D2 and ADR-028 §D1, backward compatibility with existing TOML specs is
   non-negotiable. `#[serde(default)]` for a `bool` field defaults to `false`.

4. `SensorSpec` is a `pub` type that is `#[non_exhaustive]` — verify this annotation
   before adding the field. Adding a new field to an existing `#[non_exhaustive]` struct
   is backward compatible with downstream crates.

5. The `raw_extensions` column name is hardcoded as `"raw_extensions"` per BC-2.16.003
   §Column Routing and ADR-058 §I2. It MUST be a `Utf8` (string) Arrow data type, not
   a `LargeUtf8` or a `Struct` type.

6. `reqwest` dependencies: if any test file or new module requires `reqwest`, MUST declare
   `default-features = false, features = ["rustls-tls"]` per ADR-050 D1.

7. No `println!` in production code. Any debug output uses `tracing::debug!` with
   structured fields per CLAUDE.md §Conventions.

8. When adding `ocsf_column_naming: bool` to `SensorSpec`, the exhaustive struct
   literals in `impl Default for SensorSpec` and `SensorSpec::new()` MUST both be
   updated to include `ocsf_column_naming: false`. Rust does not auto-fill exhaustive
   struct literals (E0063). Furthermore, `Default::default()` and `#[serde(default)]`
   must agree: if `Default::default().ocsf_column_naming` were `true`, test fixtures
   built via `..Default::default()` would diverge from TOML-parsed specs (which yield
   `false` for absent keys). Pin both to `false`.

9. The collision detection in T-21 MUST be gated on `sensor_spec.ocsf_column_naming ==
   true`. When the flag is `false`, all sensors use `col.name` (unique by TOML
   validation); the uniqueness check does not apply and MUST NOT run for non-OCSF-named
   sensors.

10. The shadow check added by T-21 clause (b) (ADR-058 §J2) MUST enforce the `A ≠ B`
    self-match exclusion. A column whose `ocsf_field_to_arrow_name` result equals its
    own `col.name` (e.g., `risk_score` with `ocsf_field = "risk_score"`, or `status`
    with `ocsf_field = "status"`) is legal and MUST NOT produce `Err`. Implement by
    comparing column positions or identities, not by string equality of the flattened
    name against the same column's col.name. Failure to implement the exclusion
    rejects valid Claroty config (confirmed: two Claroty columns rely on self-match
    legality — `alerts.status` and `devices.risk_score`).

---

## Library & Framework Requirements

| Library | Role | Constraint |
|---------|------|-----------|
| `serde` | `#[serde(default)]` attribute on `ocsf_column_naming` | Workspace-pinned version |
| `arrow` | `RecordBatch`, `Field`, `DataType` in `pipeline_result_to_record_batch` | Workspace-pinned version in root `Cargo.toml` |
| `serde_json` | JSON serialization for `raw_extensions` blob | Workspace-pinned version |

No new crate additions are anticipated. The `string.replace('.', "_")` operation for
`ocsf_field_to_arrow_name` uses only `std` — no external crate needed.

Do NOT add new `reqwest` dependencies. Do NOT add `native-tls` features.

---

## File Structure Requirements

| File | Action |
|------|--------|
| `crates/prism-spec-engine/src/spec_parser.rs` | Modify: add `#[serde(default)] pub ocsf_column_naming: bool` to `SensorSpec` |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify: add `ocsf_field_to_arrow_name` fn; update `pipeline_result_to_record_batch`; update `build_column_array` `raw_extensions` path |
| `crates/prism-mcp/src/tools/prism_describe.rs` | Modify: `ColumnDescriptor.name` sourcing branches on `sensor_spec.ocsf_column_naming` |
| `crates/prism-sensors/specs/claroty.sensor.toml` | Modify: add `ocsf_column_naming = true` at sensor level AND change `device_category` ocsf_field from `"device.type"` to `"device.type_category"` (both in same edit per AC-005) |
| `crates/prism-bin/tests/` (e2e test file — TBD at dispatch) | Modify: update `test_BC_2_11_005_e2e_claroty_query_returns_data` assertion |
| `crates/prism-spec-engine/tests/` (new or existing test file) | Modify: add RG-001..RG-002 |
| `crates/prism-bin/tests/` (unit test file — TBD at dispatch) | Modify: add RG-003..RG-008, RG-009, RG-010 |

Implementer MUST verify file names via `find crates/prism-spec-engine/tests crates/prism-bin/tests -name "*.rs"` at dispatch. Do NOT create new test files if existing `bc_2_01_013_spec_driven_adapter.rs` or similar applies.

Do NOT modify: any other sensor TOML spec (CrowdStrike, Armis, Cyberint); `column_mapping.rs`; any BC or ADR body (product-owner / architect scope).

---

## Forbidden Dependencies

Build-time enforcement rules:

- `prism-spec-engine` MUST NOT import from `prism-bin`. The `ocsf_field_to_arrow_name` function lives in `prism-bin`, not `prism-spec-engine`. If `cargo tree -p prism-spec-engine` shows `prism-bin` after this story, a forbidden import was introduced.

- `prism-sensors` MUST NOT gain a dependency on `prism-spec-engine`. If `cargo tree -p prism-sensors` shows `prism-spec-engine`, the story introduced a forbidden import.

- `prism-bin` MUST NOT gain any new `native-tls` features. Verify `Cargo.toml` reqwest entries if any are modified.

---

## TD-VSDD-097 / POL-29 Three-Dimension Sweep Verdict

### v1.2 Amendment Sweep (ADR-058 §J2 shadow-check addition + AC-005 TOML fix)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (the sibling story in EPIC-OCSF-ROUTING): swept in full.
Findings: no reference to `device_category`, `device_type`, `device_type_category`,
`device.type`, shadow collision, or the §J defect class. The coercion story's scope is
`ColumnMapper::coerce_value` and `build_column_array` type-coercion gaps — entirely
orthogonal to the TOML naming fix and the shadow-check gate. No update required to
S-ADR058-OCSF-COERCION-001. VERDICT: SWEPT; CLEAR.

*S-ADR058-DTU-PARITY-MIGRATION-001* (downstream, depends on this story): swept for
`devices`-table Arrow name assertions that would be invalidated by the
`device_type_category` change. Finding: S-ADR058-DTU-PARITY-MIGRATION-001 RG-002
(`test_claroty_dtu_devices_arrow_schema_field_names_equal_ocsf_field_values`) is
designed to assert that Arrow names equal the `ocsf_field` values from the
then-current `claroty.sensor.toml`. That story depends on this story
(`depends_on: [S-ADR058-OCSF-ROUTING-001]`) and its tests are NOT yet written —
they will be authored AFTER this story merges, reading the post-amendment TOML
(`ocsf_field = "device.type_category"` for `device_category`). The parity test
writer will therefore assert `device_type_category`, not the pre-amendment
`device_type`. **No invalidation — no update required to S-ADR058-DTU-PARITY-MIGRATION-001.**
VERDICT: SWEPT; CLEAR.

**Dimension 2 — Downstream copy target:**

The `devices` table Arrow name mapping in AC-005 (this story) is the source that
a later agent leg (test-writer for S-ADR058-DTU-PARITY-MIGRATION-001) will use as
authoritative ground truth for the post-amendment schema. AC-005 now documents the
full `devices` post-fix Arrow names (`device_uid`, `device_instance_uid`,
`device_type_category`, `device_type_name`, `risk_score`, `status_code`). This IS
the downstream copy target — the parity test writer will transcribe these names
into test assertions. The AC-005 table in this story is now the canonical source
for that transcription. VERDICT: CAPTURED IN THIS AMENDMENT (AC-005 now carries
the authoritative devices table; parity migration story does not need simultaneous
update because it is not yet dispatched).

The `ColumnDescriptor.name` downstream copy concern (from v1.0/v1.1) remains:
implementer MUST sweep `ColumnDescriptor` docstrings at dispatch time for stale
`col.name` references. VERDICT: MITIGATED (standing implementer obligation).

**Dimension 3 — Mandate anchor:**

ADR-058 v2.1 §J2 carries the mandate anchor for the shadow check: "story-writer
must amend S-ADR058-OCSF-ROUTING-001 to add RG-010." This amendment discharges
that anchor. The §ADR-058 MUST Discharge section now has two rows:
- Row 1: §D2 MUST → AC-001 / RG-001/RG-002 (discharged in v1.0)
- Row 2: §J2 MUST → EC-010 / T-21 (shadow check) / RG-010 (discharged in this v1.2)

Both rows name the story + EC/AC + Red Gate test. VERDICT: DISCHARGED IN THIS
AMENDMENT.

ADR-058 v2.1 `anchor_stories:` already includes `S-ADR058-OCSF-ROUTING-001`
(SAC-2 — populated by architect in the v2.1 amendment). No further ADR update
required for anchor_stories from this amendment.

### v1.1 Sweep Record (preserved for reference)

**Dimension 1 — Sibling pair:** `claroty.sensor.toml` is the only sensor TOML that gains
`ocsf_column_naming = true`. Per ADR-058 §D3, CrowdStrike, Armis, and Cyberint TOMLs
are NOT modified. Confirmed those three TOMLs must NOT gain the flag in this story;
their absence of the flag is correct behavior (defaulting to `false`). VERDICT: SWEPT;
CLEAR.

**Dimension 2 — Downstream copy target:** `ColumnDescriptor.name` sourcing in
`prism_describe` (AC-006) — implementer MUST sweep ColumnDescriptor docstrings for
stale col.name references at dispatch. VERDICT: MITIGATED.

**Dimension 3 — Mandate anchor:** ADR-058 v2.0 §D2 ANCHOR-NEEDED discharged to
AC-001 / RG-001/RG-002. ADR-058 `anchor_stories:` populated by architect in v2.1.
VERDICT: DISCHARGED.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.2 | 2026-08-12 | story-writer | ADR-058 v2.1 §J amendment discharge. (1) RG-010 added: `test_pipeline_result_to_record_batch_ocsf_shadow_collision_returns_error` — fails until shadow check (flattened ocsf_field name ≠ different column's col.name in same table) is implemented; includes mandatory self-match exclusion assertion (`A ≠ B` guard) covering the legal `risk_score → risk_score` and `status → status` Claroty cases. (2) T-11C added (red-gate authoring task for RG-010; preserves red-then-green ordering). (3) T-21 extended: shadow check clause (b) added to the combined collision-detection pass; `A ≠ B` self-match exclusion specified. (4) EC-010 added: flag-transition name shadowing defect class. (5) AC-005 TOML scope extended: `device_category` ocsf_field changed from `"device.type"` to `"device.type_category"` in the same TOML edit as `ocsf_column_naming = true`; devices table post-fix Arrow names documented (`device_uid`, `device_instance_uid`, `device_type_category`, `device_type_name`, `risk_score`, `status_code`). (6) Architecture Compliance Rule 10 added: self-match exclusion obligation. (7) ADR-058 MUST Discharge second row added for §J2 mandate → RG-010. (8) Authority section updated to ADR-058 v2.1; §J1–§J4 sections added to reading list. (9) TD-VSDD-097 three-dimension verdict updated for this amendment: S-ADR058-OCSF-COERCION-001 swept (clear); S-ADR058-DTU-PARITY-MIGRATION-001 swept — no devices Arrow name invalidated (parity tests not yet written; depend on this story; will read post-amendment TOML at dispatch time). Density updated 9/8 = 1.125 → 10/8 = 1.25. |
| 1.1 | 2026-08-12 | story-writer | Remove-uncertainty pass: Q2 CORRECTED — Arrow 58 silently first-matches on duplicate field names; added EC-009 (intra-table flattening collision), RG-009 (collision detection test), T-11B (red gate for RG-009), T-21 (fail-closed collision check in `pipeline_result_to_record_batch`), Architecture Compliance Rules 8 and 9. Q3(c) CORRECTED — T-12 updated to name all three edit sites (`SensorSpec` struct, `impl Default`, `SensorSpec::new()`). Q1 CONFIRMED — AC-002 strengthened with Arrow 58 field-name basis, DataFusion SQL identifier preconditions, and PrismQL-lexer guarantee. Density updated 8/8 → 9/8 = 1.125. |
| 1.0 | 2026-08-12 | story-writer | Initial authorship — ADR-058 Stage 2 story. Discharges ADR-058 v2.0 §D2 ANCHOR-NEEDED mandate for ocsf_column_naming MUST (AC-001/RG-001/RG-002). Explicit scoping decision: ColumnMapper::map_record wiring gap stays out of scope; BC-2.01.013 EC-01-025 resolves via Arrow schema naming change per ADR-058 §B2 item 4. 8 ACs, 8 RGTs (density 1.0). BC-2.16.003 v1.4, BC-2.01.013 v1.16, ADR-058 v2.0 at authoring time. |
