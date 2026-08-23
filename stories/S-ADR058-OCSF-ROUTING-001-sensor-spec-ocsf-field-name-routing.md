---
document_type: story
story_id: S-ADR058-OCSF-ROUTING-001
title: "ADR-058 Stage 2 — OCSF Field-Name Routing: ocsf_column_naming Flag, Underscore-Flattened Arrow Names, Claroty Activation"
version: "1.54"
level: "L4"
status: draft
producer: story-writer
timestamp: "2026-08-12T00:00:00Z"
modified: "2026-08-23"
phase: 3
wave: claroty-live
epic_id: EPIC-OCSF-ROUTING
priority: P1
points: 8
tdd_mode: strict
target_module: prism-spec-engine
subsystems:
  - SS-01
  - SS-02
  - SS-10
  - SS-16
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters) owns this story's scope because `prism-sensors/specs/claroty.sensor.toml`
#     (KF-01..KF-12 TOML corrections + ocsf_column_naming flag) is in the prism-sensors crate,
#     which is listed under SS-01 per ARCH-INDEX. `prism-spec-engine` is also listed under SS-01
#     (sensor spec parsing boundary: `SensorSpec` gains the `ocsf_column_naming` field).
#   SS-02 (OCSF Normalization) owns this story's scope because `prism-ocsf/src/class_selector.rs`
#     (add CLASS_UID_ENTITY_MANAGEMENT = 3004, entity_management + inventory_info arms, deprecate
#     audit_activity, select() arms) is in prism-ocsf, which is the sole crate in SS-02 per ARCH-INDEX.
#   SS-10 (MCP Interface) owns this story's scope because `prism-mcp::tools::prism_describe`
#     (ColumnDescriptor name/description sourcing branches on ocsf_column_naming per ADR-058 §G)
#     is in prism-mcp, and `prism-bin::spec_driven_adapter` (`pipeline_result_to_record_batch`,
#     `build_column_array`, `ocsf_field_to_arrow_name` helper, ocsf.unknown_class_name warn)
#     is in prism-bin — both crates are in SS-10 per ARCH-INDEX.
#   SS-16 (Spec Engine) owns this story's scope because `prism-spec-engine::spec_parser`
#     (`SensorSpec` struct) gains the new `ocsf_column_naming` field and SS-16 is the canonical
#     owner of prism-spec-engine per ARCH-INDEX Subsystem Registry.
crates_touched:
  - prism-spec-engine
  - prism-bin
  - prism-mcp
  - prism-ocsf
  - prism-sensors
  - prism-query
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.003
  - BC-2.16.002
  - BC-2.01.013
  - BC-2.11.016
verification_properties:
  - VP-017
  - VP-016
holdout_scenarios: [HS-ROUTING-001-A-001, HS-ROUTING-001-A-002, HS-ROUTING-001-A-003, HS-ROUTING-001-A-004]
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
# `SELECT id FROM claroty_alerts` must become `SELECT finding_info_uid FROM claroty_alerts`).
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
input-hash: "e7455fb"
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

**ADR-058 v2.31: v1 Column Naming — OCSF Field-Path Routing with Underscore-Flattened Arrow
Names; DTU Migration Deferred.** Version `2.31`, status: accepted (2026-08-23). Read
§B2 (decision — **multi-valued array source fields with `ocsf_field == None`
MUST be serialized as compact JSON-list strings in `raw_extensions`, NOT as nested JSON arrays**),
§C (quoting convention — Option 4 chosen), §D (per-sensor scoping, flag
mechanism — **§D1: `pipeline_result_to_record_batch` MUST gain
`sensor_spec: &SensorSpec` as an explicit parameter threaded from the `fetch()` call site;
this is ADR-022 §C wiring (adding a previously absent parameter), not redesign**),
§E (blast radius), §G (prism_describe output spec — **Tier-1/Tier-2 model:
Tier-1 columns (`ocsf_field == Some`) emit ColumnDescriptor with
`name = ocsf_field_to_arrow_name(ocsf_field)` and `description = ocsf_field`;
Tier-2 columns (`ocsf_field == None`) MUST NOT emit individual ColumnDescriptors —
instead `prism_describe` MUST emit exactly ONE `raw_extensions` ColumnDescriptor with
four-field shape: `name = "raw_extensions"`, `col_type = prism_core::column::ColumnType::Json`,
`nullable = true`, and `description` identifying it as a JSON object and enumerating every
`ocsf_field == None` column's `col.name` as a source key**),
§H (Stage 1 confirmed
separate), §I (implementation guidance including **§I1 two-step form —
Step 1 signature addition (`sensor_spec: &SensorSpec` parameter), Step 2 field-name
computation inside the function body**; **§I1 canonical home of
`ocsf_field_to_arrow_name` is `prism-spec-engine::column_mapping` (NOT `prism-bin::spec_driven_adapter`);
both `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe` import from there**;
**§I2: `pipeline_result_to_record_batch` raw_extensions serialization MUST
produce compact JSON-list strings for multi-valued array fields (NOT nested JSON arrays)**;
**§I5 TOML + code correction obligations for
KF-01 through KF-12**; **§I5 process-gap obligation: `ocsf.unknown_class_name` WARN on
Err branch before `.unwrap_or(0)` in `pipeline_result_to_record_batch`; Path A / Path B
liveness determination; `select_by_class_name` two new arms: `"entity_management"→3004` and
`"inventory_info"→5001`; `"audit_activity"` arm becomes dead code pending deprecation annotation**),
**§J1–§J4 (flag-transition name shadowing adjudication, normative fail-closed rule, Claroty
`devices` table resolution, `ocsf_field` count 31 pre-correction per §J4 / 27 post-correction
per ADR-058 §Status across four tables (OQ-005 adds `audit_logs.id → metadata.uid` → count 26→27) — §J2: `pipeline_result_to_record_batch` MUST also fail-closed
(`Err(ArrowError::SchemaError(...))`) when any `ocsf_field` flattens via
`ocsf_field_to_arrow_name` to a synthesized/reserved name: `class_uid`, `category_uid`,
`_sensor`, or `raw_extensions`**, and **§K (OCSF v1.7.0
schema validation — §K4 finding summary KF-01..KF-12, §K5 divergence adjudication including
class_selector.rs KF-01 code defect confirmed and Armis sibling sweep)** in full before
implementing.
Path: `.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`.

**BC-2.16.003: Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec.** Version `1.26`, status: active
(modified 2026-08-23). §Column Routing postconditions, **§Claroty Contracted OCSF Mappings
(ground truth for all four Claroty tables with KF-01..KF-12 corrections)**, and
**§Interpretation A: Arrow Field Naming** govern the obligation that `ocsf_field` declarations
produce queryable Arrow field identifiers. **EC-016-013-023** (KF-01 entity_management class_uid
= 3004 wire-level postcondition) and **EC-016-013-024** (KF-02 inventory_info class_uid = 5001
regression-prevention) are the authoritative wire-shape obligations for AC covering class_uid
Arrow column values. **EC-016-013-027** (Tier-1/Tier-2 `prism_describe` model per
§Interpretation A: `ocsf_field == None` columns MUST NOT appear as individual
ColumnDescriptor names; `prism_describe` MUST emit exactly ONE `raw_extensions`
ColumnDescriptor per table with four-field shape `name = "raw_extensions"`,
`col_type = prism_core::column::ColumnType::Json`, `nullable = true`, and `description`
enumerating all `ocsf_field == None` source keys) is the
authoritative obligation for AC-006 Tier-2 prohibition and AC-007b `prism_describe`
`raw_extensions` ColumnDescriptor emission. **EC-016-013-028** (reworded in v1.18:
multi-valued array source fields with `ocsf_field == None` MUST be serialized as compact
JSON-list strings in `raw_extensions` — `pipeline_result_to_record_batch` is the synthesis
locus per ADR-058 §I2, applying the SAME source_path extraction + ENRICH-1
`Value::Array`→compact-JSON-list-string normalization as first-class columns — NOT a naive
`r.get(col.name)`; e.g., `ip_list = ["192.168.1.1","10.0.0.1"]` serializes as
`"[\"192.168.1.1\",\"10.0.0.1\"]"`) is the authoritative obligation for AC-007c and
RG-026. **EC-016-013-029** (NEW in v1.18: when any `ocsf_field` value, after applying
`ocsf_field_to_arrow_name`, equals a synthesized/reserved Arrow column name —
`class_uid`, `category_uid`, `_sensor`, or `raw_extensions` — `pipeline_result_to_record_batch`
MUST fail-closed returning `Err(ArrowError::SchemaError)`) is the authoritative obligation
for AC-013 and RG-027 (§J2 synthesized-name reservation guard, ADR-058 §J2).
**EC-016-013-032** (NEW in v1.24: `parse_and_validate_spec_toml` MUST reject §J1/§J2/§J4
OCSF column collisions at spec-load time via `validate_ocsf_column_collisions` (Validation
Rule 8); error contains E-SPEC-030 + collision tag ([§J1]/[§J2]/[§J4]); boot
ConfigInvalid → exit 2; hot-reload keeps prior spec; runtime `pipeline_result_to_record_batch`
§J guard remains as defense-in-depth) is the authoritative obligation for AC-021 and
RG-Q-012/013/014.
**EC-016-013-011** (corrected: `ocsf.unknown_class_name` WARN is a
RUNTIME emission on the `Err` branch of `select_by_class_name`, NOT a load-time/startup
warning) governs AC-011. This story brings the production path into conformance with those
postconditions for Claroty.
Path: `.factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md`.

**BC-2.16.002: Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation.** Version `2.34`, status: active
(modified 2026-08-23). Canonical Structured Event Catalog — `ocsf.unknown_class_name`
WARN — emitted by `pipeline_result_to_record_batch` on the `Err` branch of
`EventClassSelector::select_by_class_name` before `.unwrap_or(0)`. Fields: `ocsf_class: %display`,
`sensor_id: %display`, `table_name: %display`. SAP-1 / PG-LP11-001 obligation: the implementer
MUST add this `tracing::warn!` emission to `pipeline_result_to_record_batch` in the same commit
as the `select_by_class_name` arm additions. This is the source for AC-011 in this story.
**NEW in v2.34:** `ocsf.zero_tier1_table` WARN — emitted at spec-load/registration when
`ocsf_column_naming = true` and a table has zero Tier-1 columns (`ocsf_field == None` for
every column) but ≥1 Tier-2 column. Fields: `sensor_id: %display`, `table_name: %display`.
Emitted ONCE per table at registration time (not per-query). SAP-1 / PG-LP11-001 obligation:
implementer MUST add this emission in T-31 (same commit as the A+W raw_extensions projection
for zero-Tier-1-with-Tier-2 tables). This is the source for AC-019 sub-case A+W in this story.
Path: `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md`.

**BC-2.01.013: DataSource Trait Eliminates Per-Sensor Code Duplication.** Version `1.23`, status: active.
EC-01-025 records "ColumnMapper step is missing" as NON-CONFORMANT. Stage 2 resolves
EC-01-025 for Claroty per ADR-058 §B2 item 4 (OCSF field names now appear in the Arrow
schema for the flagged sensor).
Path: `.factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md`.

**BC-2.11.016: Query Planner Column Resolution.** Version `1.30`, status: active
(modified 2026-08-23). **EC-11-079** governs OCSF-mode column-resolution gate obligations
(covered by AC-016/017/018 and RG-Q-001..RG-Q-009). **EC-11-080** (amended A+W in v1.30):
when `ocsf_column_naming = true` and a sensor table has zero Tier-1 columns (no column
has an `ocsf_field` declaration):
- **Sub-case: zero Tier-1, zero Tier-2 (all columns have `ocsf_field == None` AND are not
  raw_extensions candidates):** `TableRegistry` MUST register `class_uid` (Integer) and
  `_sensor` (String) as the full available set — the available columns MUST be exactly
  `["_sensor", "class_uid"]`; no raw `col.name` value appears.
- **Sub-case A+W: zero Tier-1, ≥1 Tier-2 (at least one column has `ocsf_field == None`):**
  `TableRegistry` MUST register `class_uid` (Integer), `_sensor` (String), AND `raw_extensions`
  (Json) as the available set — the available columns MUST be exactly
  `["_sensor", "class_uid", "raw_extensions"]`; Tier-2 data is preserved via `raw_extensions`
  and MUST NOT be dropped. Additionally, a `ocsf.zero_tier1_table` WARN event MUST be emitted
  ONCE at spec-load/registration for that table (not per-query); fields: `sensor_id`, `table_name`.
  This is the Option A+Warning decision (human decision 2026-08-23; ADR-058 §J6).
EC-11-080 is the authoritative obligation for AC-019 and RG-Q-010/011/017. Traces to ADR-058 §J6.
Path: `.factory/specs/behavioral-contracts/BC-2.11.016-query-planner-column-resolution.md`.

---

## Narrative

As a Prism LLM agent querying Claroty sensor data, I want sensor columns to have
OCSF-semantic Arrow field names (e.g., `finding_info_uid` for `ocsf_field = "finding_info.uid"`),
so that I can use the column names returned by `prism_describe` verbatim in PrismQL
queries without any quoting ceremony, and cross-sensor joins on shared OCSF field names
work correctly.

---

## ADR-058 MUST Discharge: Mandate Anchor #1

**ADR-058 §D2 `ANCHOR-NEEDED`: DISCHARGED.** DISCHARGED — ADR-058 §D2 carries the inline
(Anchored: S-ADR058-OCSF-ROUTING-001 AC-001 / RG-001/RG-002) mark. No architect action required.

**ADR-058 §J2 `ANCHOR-NEEDED`: DISCHARGED.** DISCHARGED — ADR-058 §J2 carries the inline
(Anchored: S-ADR058-OCSF-ROUTING-001 EC-010 / T-21 / RG-010) mark for the flag-transition
shadow check (A≠B). The v2.28 §J2 synthesized-name reservation amendment is separately
anchored to S-ADR058-OCSF-ROUTING-001 AC-013 / T-21 clause (c) / RG-027.
No architect action required.

The mandate anchor records:

| MUST Statement | Story | AC | Red Gate Test | Status |
|---|---|---|---|---|
| `ocsf_column_naming: bool` field MUST be added to `SensorSpec` with `#[serde(default)]` (ADR-058 §D2) | S-ADR058-OCSF-ROUTING-001 | AC-001 | RG-001, RG-002 | DISCHARGED |
| `pipeline_result_to_record_batch` MUST check, when `ocsf_column_naming == true`, that no flattened `ocsf_field` name equals a DIFFERENT column's `col.name` in the same table (`A ≠ B` exclusion), fail-closed (ADR-058 §J2) | S-ADR058-OCSF-ROUTING-001 | EC-010, T-21 (shadow check extension) | RG-010 | DISCHARGED |
| `pipeline_result_to_record_batch` MUST gain `sensor_spec: &SensorSpec` as an explicit parameter threaded from the `fetch()` call site in `spec_driven_adapter.rs`; no placeholder construction (ADR-058 §D1, ADR-022 §C wiring) | S-ADR058-OCSF-ROUTING-001 | AC-012 | RG-024 | DISCHARGED |
| `prism_describe` MUST NOT emit an individual ColumnDescriptor for `ocsf_field == None` columns when `ocsf_column_naming = true`; MUST emit exactly ONE `raw_extensions` ColumnDescriptor with four-field shape: `name = "raw_extensions"`, `col_type = prism_core::column::ColumnType::Json`, `nullable = true`, and `description` identifying it as a JSON object and enumerating every `ocsf_field == None` column's `col.name` as a source key (ADR-058 §G; BC-2.16.003 EC-016-013-027 / §Interpretation A) | S-ADR058-OCSF-ROUTING-001 | AC-006 (Tier-2), AC-007b | RG-025 | DISCHARGED |
| `ocsf_field_to_arrow_name` MUST live in `prism-spec-engine::column_mapping`; both `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe` import it from there (no cycle); placing it in `prism-bin` is FORBIDDEN — `prism-mcp` is Level 6 in the topological ordering and `prism-bin` is Level 7 (`dependency-graph.md` §Dependency Rules Rule 2: lower-layer crates never depend on higher-layer crates); a `prism-mcp → prism-bin` edge would violate this rule (ADR-058 §I1) | S-ADR058-OCSF-ROUTING-001 | AC-002 | RG-003, RG-004 | DISCHARGED |
| Multi-valued array source fields with `ocsf_field == None` MUST be serialized as compact JSON-list strings in `raw_extensions` — NOT nested JSON arrays (BC-2.16.003 EC-016-013-028; ADR-058 §B2/§I2) | S-ADR058-OCSF-ROUTING-001 | AC-007c | RG-026 | DISCHARGED |
| `pipeline_result_to_record_batch` MUST fail-closed (`Err(ArrowError::SchemaError(...))`) when any `ocsf_field` flattens via `ocsf_field_to_arrow_name` to a synthesized/reserved name (`class_uid`, `category_uid`, `_sensor`, or `raw_extensions`) (ADR-058 §J2 synthesized-name guard; BC-2.16.003 EC-016-013-029) | S-ADR058-OCSF-ROUTING-001 | AC-013, T-21 clause (c) | RG-027 | DISCHARGED |
| `TableRegistry` MUST register OCSF-flattened names (`ocsf_field_to_arrow_name(ocsf_field)` for each Tier-1 column) for `ocsf_column_naming = true` sensor tables; raw `col.name` values for Tier-1 columns MUST NOT be registered; E-QUERY-038 `available_columns` in error responses MUST list OCSF-flattened names only (BC-2.11.016 EC-11-079 sub-cases (a) and (b)) | S-ADR058-OCSF-ROUTING-001 | AC-016 | RG-Q-001, RG-Q-002, RG-Q-004, RG-Q-005, RG-Q-006 | PENDING (Fix A — T-26) |
| E-QUERY-002/041 type-compat gate MUST resolve column types by OCSF-flattened name for `ocsf_column_naming = true` tables (BC-2.11.016 EC-11-079 sub-case (c)) | S-ADR058-OCSF-ROUTING-001 | AC-017 | RG-Q-003 | PENDING (Fix C — T-27) |
| The set of columns reported by E-QUERY-038 `available_columns`, `prism_describe`, and `SELECT *` MUST all agree on the OCSF-flattened name set for `ocsf_column_naming = true` tables (BC-2.11.016 EC-11-079 sub-case (d) name-agreement invariant) | S-ADR058-OCSF-ROUTING-001 | AC-018 | RG-Q-001..RG-Q-009 | PENDING (Fix A/B/C — T-26/T-27) |
| When `ocsf_column_naming = false` (flag-false green-lock), no OCSF-flattened name routing occurs; existing column resolution behavior MUST be unchanged (BC-2.11.016 EC-11-079 sub-case (b) flag-false invariant) | S-ADR058-OCSF-ROUTING-001 | AC-016 (green-lock) | RG-Q-007 | PENDING (Fix A — T-26) |
| `TableRegistry` MUST register `class_uid` (Integer) and `_sensor` (String) for OCSF-mode tables with zero Tier-1 columns; the outer `if !table.columns.is_empty()` guard on the OCSF branch of `register_sensor` MUST be removed; helpers `ocsf_projected_column_names` / `ocsf_projected_column_types` MUST be called unconditionally when `ocsf_column_naming = true` (ADR-058 §J6; BC-2.11.016 EC-11-080) | S-ADR058-OCSF-ROUTING-001 | AC-019 | RG-Q-010, RG-Q-011 | PENDING (LOW-1-FIX — T-28) |
| When `ocsf_column_naming = true` and a table has zero Tier-1 columns with ≥1 Tier-2 column, `register_sensor` MUST register `raw_extensions` (Json) in the plan-gate available set AND emit `ocsf.zero_tier1_table` WARN ONCE at spec-load/registration; available set MUST be exactly `["_sensor", "class_uid", "raw_extensions"]`; `ocsf_projected_column_names` helper MUST return this set for zero-Tier-1-with-Tier-2 tables (BC-2.11.016 EC-11-080 A+W sub-case; BC-2.16.002 `ocsf.zero_tier1_table` catalog row; ADR-058 §J6) | S-ADR058-OCSF-ROUTING-001 | AC-019 (A+W sub-case) | RG-Q-017 | PENDING (A+W-FIX — T-31) |
| `ocsf_projected_column_names` + `ocsf_projected_column_types` MUST be the single authoritative projection impl in `prism-spec-engine::column_mapping`; `ocsf_or_raw_column_names_for_table` in `engine.rs` MUST be a thin forward to `ocsf_projected_column_names`; registry column-name set MUST be byte-equal sorted to shared helper output (ADR-058 §I7) | S-ADR058-OCSF-ROUTING-001 | AC-020 | RG-Q-015 | PENDING (OBS-1-FIX — T-29) |
| `parse_and_validate_spec_toml` MUST reject §J1/§J2/§J4 OCSF column collisions via `validate_ocsf_column_collisions` (Validation Rule 8); error MUST contain E-SPEC-030 + collision tag ([§J1]/[§J2]/[§J4]); boot ConfigInvalid → exit 2; hot-reload keeps prior spec (ADR-058 §J7; BC-2.16.003 EC-016-013-032) | S-ADR058-OCSF-ROUTING-001 | AC-021 | RG-Q-012, RG-Q-013, RG-Q-014, RG-Q-016 | PENDING (OBS-2-FIX — T-30) |

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
| BC-2.16.003 | v1.26 | active | §Column Routing postconditions, §Claroty Contracted OCSF Mappings (ground truth — KF-01..KF-12 corrections for all four tables), §Interpretation A: Arrow Field Naming — `ocsf_field` declarations produce queryable Arrow field identifiers; EC-016-013-023 (audit_logs class_uid = 3004 wire-level) and EC-016-013-024 (devices class_uid = 5001 regression-prevention); EC-016-013-027 (Tier-1/Tier-2 `prism_describe` model: no individual ColumnDescriptor for `ocsf_field == None` columns; exactly one `raw_extensions` ColumnDescriptor with four-field shape: name + col_type=Json + nullable=true + description enumerating source keys); EC-016-013-028 (reworded v1.18: multi-valued array source fields → compact JSON-list string in raw_extensions via `pipeline_result_to_record_batch` source_path extraction + ENRICH-1 normalization — NOT naive `r.get(col.name)` — governs AC-007c / RG-026); EC-016-013-029 (NEW v1.18: flattened `ocsf_field` equal to synthesized reserved name → `Err(ArrowError::SchemaError)` fail-closed — governs AC-013 / RG-027); EC-016-013-032 (NEW v1.24: `parse_and_validate_spec_toml` rejects §J1/§J2/§J4 collisions with E-SPEC-030 at spec-load time; boot ConfigInvalid → exit 2; hot-reload keeps prior spec — governs AC-021 / RG-Q-012/013/014/016); EC-016-013-011 (runtime `ocsf.unknown_class_name` WARN on Err branch — governs AC-011); OBS-1 (v1.26): `validate_ocsf_column_collisions` §J7 signature drop — parameter removed per ADR-058 §J7 amendment (governs T-30/RG-Q-016) |
| BC-2.16.002 | v2.34 | active | Canonical Structured Event Catalog `ocsf.unknown_class_name` WARN — fields `ocsf_class`, `sensor_id`, `table_name`; SAP-1/PG-LP11-001 obligation on implementer to add the warn emission in the same commit as the `select_by_class_name` arm additions (AC-011); NEW v2.34: `ocsf.zero_tier1_table` WARN — emitted ONCE at spec-load/registration when `ocsf_column_naming = true` and table has zero Tier-1 + ≥1 Tier-2 columns; fields `sensor_id`, `table_name`; governs AC-019 A+W sub-case and RG-Q-017 |
| BC-2.01.013 | v1.23 | active | EC-01-025 NON-CONFORMANT annotation resolved for Claroty after this story merges; product-owner updates annotation |
| BC-2.11.016 | v1.30 | active | EC-11-079: E-QUERY-038 column-resolution gate and E-QUERY-002/041 type-compat gate MUST operate against the OCSF-flattened schema (not raw TOML `col.name`) for `ocsf_column_naming = true` tables; `available_columns` payload MUST list OCSF-flattened names only; raw `col.name` refs rejected as-if-absent; describe/select/query name-agreement invariant; covered by AC-016/AC-017/AC-018 and RG-Q-001..RG-Q-009 (RG-Q-008/009 cover multi-tenant `resolved_spec_map` path). EC-11-080 (A+W amendment v1.30): zero-Tier-1-with-Tier-2 OCSF table MUST project `["_sensor", "class_uid", "raw_extensions"]` + emit `ocsf.zero_tier1_table` WARN at registration; covered by AC-019 and RG-Q-010/011/017 |

---

## Red Gate Tests (SAC-1 — tdd_mode: strict)

All forty-six tests MUST be failing (RED) before any implementation code is written.
Test-writer dispatched FIRST; implementer only after all 46 confirmed failing.

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
  `prism-spec-engine::column_mapping` (ADR-058 §I1 canonical home; NOT
  `prism-bin::spec_driven_adapter` — placing it in prism-bin would be unreachable
  from prism-mcp without a forbidden cycle — `prism-mcp` is Level 6 / `prism-bin` is Level 7 in the
  topological ordering (`dependency-graph.md` §Dependency Rules Rule 2); a `prism-mcp → prism-bin`
  edge is forbidden because lower-layer crates never depend on higher-layer crates). Test is placed in
  `crates/prism-spec-engine/src/column_mapping.rs` `#[cfg(test)] mod tests`.
  `ocsf_field_to_arrow_name("finding.uid")` MUST return `"finding_uid"`;
  `ocsf_field_to_arrow_name("actor.user.name")` MUST return `"actor_user_name"`.
  The no-cycle importability guarantee is enforced at compile time: both
  `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe` add
  `use prism_spec_engine::column_mapping::ocsf_field_to_arrow_name` — this compiles
  because both crates already depend on prism-spec-engine; a prism-mcp→prism-bin
  dependency would not. Covers AC-002.

- **RG-004:** `test_ocsf_field_to_arrow_name_single_segment_is_unchanged` —
  fails until the function exists. `ocsf_field_to_arrow_name("status")` MUST return
  `"status"` (no dots, unchanged). Covers AC-002.

- **RG-005:** `test_pipeline_result_to_record_batch_ocsf_flag_true_uses_flattened_names` —
  fails until `pipeline_result_to_record_batch` branches on `sensor_spec.ocsf_column_naming`.
  A `SensorSpec` with `ocsf_column_naming = true` and a column with
  `ocsf_field = Some("finding.uid")` MUST produce an Arrow schema where the field is
  named `"finding_uid"`, not `"id"` or `"finding.uid"`. Covers AC-003.

- **RG-006:** `test_pipeline_result_to_record_batch_ocsf_flag_false_uses_col_name` —
  fails at compile time until `SensorSpec` gains the `ocsf_column_naming` field (E0063:
  missing field in exhaustive struct literals; same compile-fail as RG-001/RG-002). Once the
  field exists with `#[serde(default)]`, a `SensorSpec` with `ocsf_column_naming = false` (or
  absent) and a column with `name = "id"` and `ocsf_field = Some("finding.uid")` matches
  current production behavior — the flag-false path uses `col.name` unconditionally and the
  test passes without requiring the T-14 conditional branch. Covers AC-004.

- **RG-007:** `test_prism_describe_ocsf_column_naming_true_returns_flattened_name_and_dotted_description` —
  fails until `prism_describe` branches on `sensor_spec.ocsf_column_naming`. A
  `SensorSpec` with `ocsf_column_naming = true` and a column with `name = "id"`,
  `ocsf_field = Some("finding_info.uid")` MUST produce a `ColumnDescriptor` with
  `name = "finding_info_uid"` and `description = "finding_info.uid"`. Covers AC-006
  Tier-1 (positive path: `ocsf_field == Some` column gets flattened name). The Tier-2
  prohibition (`ocsf_field == None` columns must NOT appear as individual ColumnDescriptor
  names) and the `raw_extensions` ColumnDescriptor emission are covered by RG-025.

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
  in Claroty devices (verified per BC-2.16.003 §Claroty Contracted OCSF Mappings
  devices table — no other column has `col.name = "risk_score"`; note: the devices
  table grew to 20 columns via PR #236; use BC-2.16.003 §Claroty Contracted OCSF
  Mappings as ground truth rather than the inline subset). Without this assertion,
  an over-broad implementation that checks `ocsf_field_to_arrow_name(A) ≠ A.col_name`
  (no `A ≠ B` guard) would reject valid production Claroty config. Covers EC-010.

- **RG-011:** `test_class_selector_entity_management_and_inventory_info_arms` —
  fails until BOTH new `select_by_class_name` arms are added with the new constant:
  (1) `pub const CLASS_UID_ENTITY_MANAGEMENT: u32 = 3004;` added to `class_selector.rs`
  AND `select_by_class_name("entity_management")` returns `Ok(3004)` — the live string the
  corrected KF-01 TOML will emit; (2) `select_by_class_name("inventory_info")` returns
  `Ok(5001)` — the live string the corrected KF-02 TOML will emit; both assertions in a
  single test.
  Note: the test does NOT assert `select_by_class_name("audit_activity")` — that string
  is dead code after the TOML correction; asserting it would test a string no production
  TOML will ever emit. Covers AC-009 sub-obligations (a) and (b) (ADR-058 §I5/§K5
  Divergence 3). Currently fails because neither arm exists.

- **RG-012:** `test_class_selector_armis_audit_log_maps_to_entity_management_3004` —
  fails until the `("armis", "audit_log")` arm in `class_selector.rs select()` returns
  `Ok(CLASS_UID_ENTITY_MANAGEMENT)` (= `Ok(3004)`). Currently returns
  `Ok(CLASS_UID_ACCOUNT_CHANGE)` (= `Ok(3001)`). Covers AC-009 (KF-01 Armis sibling,
  TD-VSDD-097 dim-1 sibling sweep per ADR-058 §K5 Divergence 3 sibling note).

- **RG-013:** `test_claroty_note_comment_not_silently_dropped_under_entity_management` —
  fails until `CLASS_UID_ENTITY_MANAGEMENT = 3004` is added to `class_selector.rs` and the
  `entity_management` arm updated. The test builds a `DynamicMessage` keyed by
  `CLASS_UID_ENTITY_MANAGEMENT` (3004). Calls `set_nested_field` with path `"comment"` and
  value `"reviewed"`. Asserts the field IS set (entity_management has the `comment` attribute
  in its protobuf descriptor — call succeeds). ALSO contrasts with an `account_change` (3001)
  `DynamicMessage` where the same `set_nested_field("comment", ...)` call silently no-ops
  (account_change has NO `comment` attribute — data loss confirmed). The contrast assertion
  is load-bearing: without it a test checking only the 3004 path cannot distinguish a correct
  implementation from one that accepts any arbitrary path. Routed to prism-ocsf
  (DynamicMessage lives in prism-ocsf). Covers AC-009 (data-loss prevention, BC-2.16.003
  EC-016-013-023).
  **SAP-3 reachability note (defense-in-depth):** `set_nested_field` is reached via Path B
  (`normalize_with_mappers` in `normalizer.rs`), which has zero live production callers per
  ADR-058 §K5; this test exercises a non-production-reachable arm and is defense-in-depth per
  SAP-3 rule 3. The live Path A guarantee for EC-016-013-023 is covered at the wire level by
  RG-016 (`test_claroty_audit_logs_record_batch_class_uid_is_3004`).

- **RG-014:** `test_claroty_alerts_reserved_fields_go_to_raw_extensions_not_first_class_columns` —
  wire-shape assertion per CLAUDE.md §Conventions wire-shape assertion discipline. Fails
  until KF-08/09/10 TOML corrections remove `ocsf_field` from `category`,
  `alert_type_name`, and `devices_count` in `claroty.sensor.toml`. With the corrected TOML
  and `ocsf_column_naming = true`, materializes a Claroty alerts RecordBatch with those
  columns present. Asserts on the serialized Arrow JSON: (1) schema has NO first-class
  `class_name`, `type_name`, or `count` fields carrying vendor strings; (2) `raw_extensions`
  Utf8 blob contains JSON keys `category`, `alert_type_name`, `devices_count` with the
  original vendor values. Covers AC-010 (KF-08/09/10, BC-2.16.003 EC-016-013-013/014/015).

- **RG-015:** `test_claroty_alerts_finding_info_fields_wire_shape` —
  wire-shape assertion covering KF-03/04/12 as a single 3-field alerts record batch. Fails
  until KF-03 (`alerts.id` ocsf_field `"finding_info.uid"`), KF-04 (`alerts.alert_name`
  ocsf_field `"finding_info.title"`), and KF-12 (`alerts.updated_time` ocsf_field
  `"finding_info.modified_time"`) TOML corrections are ALL applied. With the corrected TOML
  and `ocsf_column_naming = true`, materializes an alerts RecordBatch with a record
  `{id: "132", alert_name: "Modbus Violation", updated_time: "2024-01-15T10:30:00Z"}`.
  Asserts on serialized JSON: (1) Arrow field `"finding_info_uid"` contains `"132"`;
  (2) Arrow field `"finding_info_title"` contains `"Modbus Violation"`;
  (3) Arrow field `"finding_info_modified_time"` contains `"2024-01-15T10:30:00Z"`;
  (4) no Arrow field named `"finding_uid"`, `"finding_title"`, or `"end_time"` exists.
  Covers AC-010 assertion 1 (KF-03/04/12 wire-shape, BC-2.16.003 EC-016-013-017).

- **RG-016:** `test_claroty_audit_logs_record_batch_class_uid_is_3004` —
  wire-shape assertion. Fails until the `"entity_management" => Ok(3004)` arm exists in
  `select_by_class_name`. Materializes a Claroty `audit_logs` RecordBatch via
  `pipeline_result_to_record_batch` with `ocsf_class = "entity_management"` in the table
  spec. Asserts: (1) the Arrow `class_uid` column (Int32) contains value `3004`; (2) does
  NOT contain `3001` (old wrong `account_change` value) and NOT `0` (BASE_EVENT fallback).
  Assertion must be at the `RecordBatch` / serialized column level, not only at the
  resolver unit-test string level (per ADR-058 §I5 wire-shape assertion obligation).
  Covers AC-009 sub-obligation (b) path-A integration; traces to BC-2.16.003
  EC-016-013-023 wire-level postcondition.

- **RG-017:** `test_claroty_devices_record_batch_class_uid_is_5001_regression_guard` —
  wire-shape regression-prevention assertion. Fails until the `"inventory_info" => Ok(5001)`
  arm exists in `select_by_class_name`. Materializes a Claroty `devices` RecordBatch via
  `pipeline_result_to_record_batch` with `ocsf_class = "inventory_info"` in the table spec.
  Asserts: (1) Arrow `class_uid` column (Int32) == `5001`; (2) NOT `0` (BASE_EVENT
  fallback). Without the `"inventory_info"` arm, the KF-02 TOML change from `"device"` to
  `"inventory_info"` silently regresses `class_uid` from the current 5001 to 0 via
  `.unwrap_or(0)`. This test is the explicit regression guard against that silent data loss.
  Covers AC-009 sub-obligation (b) path-A integration; traces to BC-2.16.003
  EC-016-013-024 wire-level regression-prevention postcondition.

- **RG-018:** `test_pipeline_result_to_record_batch_unknown_ocsf_class_emits_warn` —
  process-gap observability assertion. Fails until the `tracing::warn!(event_type =
  "ocsf.unknown_class_name", ...)` emission is added to `pipeline_result_to_record_batch`
  on the `Err` branch of `select_by_class_name`. Constructs a `SensorSpec` with
  `ocsf_class = "completely_unknown_class"` (not registered in `select_by_class_name`),
  calls `pipeline_result_to_record_batch` with `ocsf_column_naming = false` (to isolate
  the class_uid path), captures `tracing` events via `tracing_test` subscriber. Asserts:
  (1) a WARN event with `event_type = "ocsf.unknown_class_name"` was emitted; (2) the
  `ocsf_class` field on the event matches the unknown string; (3) the function still
  returns `Ok(...)` with `class_uid = 0` (graceful fallback preserved). Covers AC-011;
  traces to BC-2.16.002 §Canonical Structured Event Catalog `ocsf.unknown_class_name` (SAP-1/PG-LP11-001 obligation).

- **RG-019:** `test_claroty_audit_logs_record_batch_kf11_category_in_raw_extensions` —
  wire-shape assertion. Fails until KF-11 TOML correction removes `ocsf_field` from
  `audit_logs.category`. Materializes a Claroty `audit_logs` RecordBatch via
  `pipeline_result_to_record_batch` with `ocsf_column_naming = true` and a record
  `{"category": "Authentication", "action": "Login", "note": "reviewed"}`. Asserts on
  serialized JSON: (1) no first-class Arrow field named `"category_uid"` or `"category_name"`
  exists; (2) the `raw_extensions` JSON blob contains key `"category"` with value
  `"Authentication"` (vendor value preserved); (3) Arrow field `"activity_name"` contains
  `"Login"` (`action` → `activity_name` under entity_management 3004); (4) Arrow field
  `"comment"` contains `"reviewed"` (`note` → `comment` under entity_management 3004).
  Covers AC-010 (KF-11 `audit_logs.category` absent from first-class Arrow fields; also
  validates AC-009 entity_management field mappings at RecordBatch integration level).
  Traces to BC-2.16.003 §Claroty Contracted OCSF Mappings (audit_logs table, KF-11).

- **RG-020:** `test_claroty_device_alert_relations_record_batch_finding_info_uid_wire_shape` —
  wire-shape assertion. Fails until KF-07 TOML correction changes
  `device_alert_relations.alert_id` `ocsf_field` from `"finding.uid"` to `"finding_info.uid"`.
  Materializes a Claroty `device_alert_relations` RecordBatch via
  `pipeline_result_to_record_batch` with `ocsf_column_naming = true` and a record
  `{"device_uid": "dev-001", "alert_id": "alert-123"}`. Asserts on serialized JSON:
  (1) Arrow field `"finding_info_uid"` contains `"alert-123"` (KF-07 corrected Arrow name);
  (2) no Arrow field named `"finding_uid"` exists (stale pre-KF-07 name absent);
  (3) no Arrow field named `"finding.uid"` exists (dotted-path form absent at wire level).
  Covers AC-010 (KF-07 `device_alert_relations.alert_id` → `finding_info_uid` wire-shape).
  Traces to BC-2.16.003 §Claroty Contracted OCSF Mappings (device_alert_relations table, KF-07).

- **RG-021:** `test_claroty_audit_logs_id_produces_metadata_uid_top_level_arrow_field` —
  wire-shape assertion for OQ-005 (human decision 2026-08-21: `audit_logs.id` gets
  `ocsf_field = "metadata.uid"` → Tier-1 Arrow column `metadata_uid`). Fails until the
  OQ-005 TOML correction (setting `ocsf_field = "metadata.uid"` on `audit_logs.id`) is
  applied. With the corrected TOML and `ocsf_column_naming = true`, materializes an
  `audit_logs` RecordBatch via `pipeline_result_to_record_batch` with a record
  `{id: "al-999", action: "Login", note: "reviewed"}`. Asserts on serialized JSON:
  (1) Arrow field `"metadata_uid"` (Tier-1 String column, type String) contains `"al-999"`
  (the id value is routed as a top-level first-class Tier-1 column via the ocsf_field
  mapping `"metadata.uid"` → `ocsf_field_to_arrow_name` → `"metadata_uid"`);
  (2) no `"id"` key exists in the `raw_extensions` JSON blob (the value is NOT routed to
  raw_extensions — it has `ocsf_field = "metadata.uid"` and is Tier-1);
  (3) Arrow field `"activity_name"` contains `"Login"` (the `action` → `activity_name`
  mapping under entity_management 3004 remains correct).
  Wire-shape assertion on the serialized Arrow column name per CLAUDE.md §Conventions
  wire-shape assertion discipline.
  Covers AC-010 assertion 5 (OQ-005).
  Traces to BC-2.16.003 §Claroty Contracted OCSF Mappings (audit_logs table, OQ-005).

- **RG-022:** `test_claroty_devices_device_type_produces_device_type_label_arrow_field` —
  wire-shape assertion for KF-06 (PO decision: `devices.device_type` ocsf_field changed to
  `"device.type_label"` → Arrow `"device_type_label"`). Fails until the KF-06 TOML
  correction is applied. With the corrected TOML and `ocsf_column_naming = true`,
  materializes a `devices` RecordBatch via `pipeline_result_to_record_batch` with a record
  `{uid: "dev-001", device_type: "PLC", device_name: "Pump Controller"}`. Asserts on
  serialized JSON: (1) Arrow field `"device_type_label"` contains `"PLC"` (the
  `device_type` value routed via `device.type_label` → `device_type_label` under
  Interpretation A); (2) no Arrow field named `"device_type_name"` exists (stale
  pre-KF-06 vendor-ext path absent); (3) Arrow field `"device_name"` contains
  `"Pump Controller"` (unchanged VALID mapping). This assertion is demo-critical: the
  PrismQL filter `WHERE device_type_label = 'PLC'` (per BC-2.16.003 EC-016-013-021)
  depends on this exact Arrow field name.
  Covers AC-010 assertion 6 (KF-06).
  Traces to BC-2.16.003 §Claroty Contracted OCSF Mappings (devices table, KF-06).

- **RG-023:** `test_class_selector_claroty_audit_log_select_arm_maps_to_entity_management_3004` —
  unit test for AC-009 sub-obligation (c) Claroty path. Fails until the
  `("claroty", "audit_log")` arm in `class_selector.rs select()` is updated from
  `Ok(CLASS_UID_ACCOUNT_CHANGE)` (3001) to `Ok(CLASS_UID_ENTITY_MANAGEMENT)` (3004).
  Asserts: `select("claroty", "audit_log")` returns `Ok(CLASS_UID_ENTITY_MANAGEMENT)`.
  This is a forward-compat test for Path B (zero live production callers; called by
  `normalize_with_mappers` in `normalizer.rs`). RG-012 covers the Armis arm of the same
  sub-obligation; RG-023 covers the Claroty arm. Together they complete AC-009(c) coverage.
  Covers AC-009 sub-obligation (c) Claroty path.
  Traces to ADR-058 §K5 Div-3 + §I5 (TD-VSDD-097 dim-1 sibling pair with Armis).

- **RG-024:** `test_pipeline_result_to_record_batch_sensor_spec_parameter_gates_both_branches` —
  fails at compile time (E0061: wrong number of arguments) until `pipeline_result_to_record_batch`
  gains `sensor_spec: &SensorSpec` as an explicit parameter. The test constructs a `SensorSpec`
  with `ocsf_column_naming = true` and a column with `name = "id"` and
  `ocsf_field = Some("finding_info.uid")`. Calls `pipeline_result_to_record_batch` passing
  this `sensor_spec` and asserts the Arrow schema field is named `"finding_info_uid"`
  (OCSF-flattened, not `"id"`). Then constructs a SECOND call on the same column data with
  a `SensorSpec` where `ocsf_column_naming = false`, asserting the Arrow schema field is
  named `"id"` (`col.name` path). Both branches exercised from the threaded-parameter path in
  a single test. The E0061 compile failure is the Red Gate: no amount of function-body editing
  can make this pass until the parameter is present in the signature. Covers AC-012.
  Traces to ADR-058 §D1: `pipeline_result_to_record_batch` MUST gain `sensor_spec`
  as an explicit parameter threaded from `fetch()`; traces to ADR-022 §C: wiring not redesign.

- **RG-025:** `test_prism_describe_ocsf_column_naming_true_raw_extensions_descriptor_and_no_phantom_col_names` —
  fails until `prism_describe` implements the Tier-1/Tier-2 model per ADR-058 §G /
  BC-2.16.003 §Interpretation A EC-016-013-027.

  The test constructs a `SensorSpec` with `ocsf_column_naming = true` and a mixed-column
  table: column A with `name = "id"`, `ocsf_field = Some("finding_info.uid")` (Tier-1);
  column B with `name = "category"`, `ocsf_field = None` (Tier-2);
  column C with `name = "alert_type_name"`, `ocsf_field = None` (Tier-2).

  Calls `prism_describe` and asserts ALL FIVE of:
  (i) NO `ColumnDescriptor` has `name` equal to `"category"` or `"alert_type_name"`
      (Tier-2 prohibition — `ocsf_field == None` column `col.name` values MUST NOT appear
      as individual ColumnDescriptor names; pre-fix behavior emits them as phantom queryable
      names that the LLM agent would use in queries returning no data);
  (ii) exactly ONE `ColumnDescriptor` has `name = "raw_extensions"` (count must be exactly 1,
      not zero, not two);
  (iii) the `raw_extensions` ColumnDescriptor's `description` contains the string `"category"`
       AND the string `"alert_type_name"` as source key enumerations (the description must
       identify the aggregated vendor fields so the LLM agent can access them via
       `raw_extensions->'category'` etc.);
  (iv) the `raw_extensions` ColumnDescriptor has `col_type = prism_core::column::ColumnType::Json`
       (ADR-058 §G; ADR-024 canonical ColumnType variant for JSON payloads);
  (v) the `raw_extensions` ColumnDescriptor has `nullable = true`
      (ADR-058 §G / BC-2.16.003 §Interpretation A — `nullable = true`
      reflects TWO distinct conditions: (a) per-row: the column is null when ALL
      unmapped source values in a given row are null or absent (no vendor fields to
      aggregate); (b) per-table: a table with zero `ocsf_field == None` columns
      produces no `raw_extensions` column at all; queries on such a table must not
      fail because the column is structurally absent).

  **RED condition:** Prior to the fix, `prism_describe` emits one ColumnDescriptor per column
  using the pre-Tier-2 model — it emits ColumnDescriptors with `name = "category"` and
  `name = "alert_type_name"` as phantom names (assertion (i) fails), emits no
  `"raw_extensions"` ColumnDescriptor (assertion (ii) fails with count = 0), and emits no
  four-field shape (assertions (iii)-(v) all fail). Without the fix, all five assertions fail.

  Covers AC-006 Tier-2 prohibition and AC-007b `raw_extensions` ColumnDescriptor emission
  (full four-field shape). Traces to ADR-058 §G (Tier-2 MUST NOT emit individual;
  MUST emit `raw_extensions` ColumnDescriptor with `col_type = Json`, `nullable = true`, and
  description enumerating source keys) and BC-2.16.003 EC-016-013-027 / §Interpretation
  A v1.18 (POL-38 mandate anchor).

- **RG-026:** `test_claroty_devices_ip_list_in_raw_extensions_is_compact_json_list_string` —
  fails until `pipeline_result_to_record_batch` correctly serializes multi-valued array source
  fields as compact JSON-list strings in `raw_extensions` (BC-2.16.003 EC-016-013-028;
  ADR-058 §B2/§I2).

  The test constructs a Claroty `devices` `SensorSpec` with `ocsf_column_naming = true` and
  a `devices` table that includes `ip_list` with `source_path = "$.ip_list[*]"` and
  `ocsf_field = None`. It passes a synthetic pipeline result record with
  `ip_list = ["192.168.1.1", "10.0.0.1"]` through `pipeline_result_to_record_batch` and
  asserts ALL THREE of:
  (i) the output RecordBatch serialized to JSON contains a `raw_extensions` field;
  (ii) the `raw_extensions` JSON object contains an `"ip_list"` key;
  (iii) the `"ip_list"` value is the compact JSON-list STRING `"[\"192.168.1.1\",\"10.0.0.1\"]"`
       — NOT a nested array, NOT null. This assertion MUST be made on the wire-level
       serialized output bytes, not on a pre-serialization Rust struct.

  **RED condition:** Prior to the fix, `pipeline_result_to_record_batch` may serialize
  `ip_list` as a nested JSON array (violating the compact-string contract), silently drop
  the value, or emit it as null. Without the fix, assertion (iii) fails.

  Covers AC-007c. Traces to BC-2.16.003 EC-016-013-028 + ADR-058 §B2/§I2.

- **RG-027:** `test_pipeline_result_to_record_batch_ocsf_field_flattens_to_reserved_name_returns_error` —
  fails until `pipeline_result_to_record_batch` implements the §J2 reserved-name guard from
  ADR-058.

  The test constructs a `SensorSpec` with `ocsf_column_naming = true` and a table containing
  a column whose `ocsf_field` flattens to one of the four reserved names:
  `class_uid`, `category_uid`, `_sensor`, or `raw_extensions`. The test passes this
  `SensorSpec` through `pipeline_result_to_record_batch` and asserts:
  (i) the function returns `Err(ArrowError::SchemaError(...))` — NOT `Ok(...)`;
  (ii) the error message identifies the offending reserved name.

  The test should exercise all four reserved names (four sub-cases within one test function,
  or parameterized): e.g., a column with `ocsf_field = "class.uid"` → flattens to `class_uid`
  (reserved); `ocsf_field = "category.uid"` → `category_uid` (reserved); etc.

  **RED condition:** Prior to the fix, `pipeline_result_to_record_batch` does NOT check for
  reserved names and proceeds to build a malformed Arrow schema — the reserved-name guard
  does not exist, so any column with `ocsf_field` that flattens to `class_uid` silently
  produces a name collision with the synthesized `class_uid` field. Without the fix,
  assertion (i) fails (function returns `Ok(...)` instead of `Err(...)`).

  Covers AC-013 (§J2 synthesized-name fail-closed guard). Traces to ADR-058 §J2 +
BC-2.16.003 EC-016-013-029.

- **RG-PD-001:** `test_extract_time_window_from_ast_recognizes_ocsf_flattened_time_column_as_index_eligible` —
  fails until `prism-query::pushdown::extract_time_window_from_ast` is updated to insert BOTH
  the raw `col.name` (`"timestamp"`) AND the OCSF-flattened Arrow name
  (`ocsf_field_to_arrow_name(ocsf_field)` = `"time"` for `ocsf_field = "time"`) into
  `datetime_index_cols` when constructing the set of index-eligible datetime columns for a
  Claroty `audit_logs` table with `ocsf_column_naming = true`.

  The test constructs a filter expression on `time` (the OCSF-flattened Arrow name for
  `claroty.audit_logs.timestamp`) and passes it to `extract_time_window_from_ast`. The
  `datetime_index_cols` set contains only `"timestamp"` (the raw `col.name`) BEFORE the fix.
  Asserts: (1) the function returns an INDEX-eligible time window (not `None` / full-scan
  fallback); (2) the eligible window is derived from the `time` filter predicate (not
  a phantom `"timestamp"` filter that is absent from the query).

  **RED condition:** Before the fix, `datetime_index_cols` is built from `col.name` only.
  A PrismQL filter `WHERE time > '2024-01-01T00:00:00Z'` on a Claroty table uses the
  OCSF-flattened Arrow name `"time"`. Since `"time"` is not in `datetime_index_cols`
  (which holds `"timestamp"`), `extract_time_window_from_ast` fails to recognize it as
  index-eligible and falls through to a full scan. Assertion (1) fails.

  Place in `crates/prism-query/src/pushdown.rs` `#[cfg(test)] mod tests`.
  Covers AC-014. Traces to BC-2.16.003 §Interpretation A (OCSF-flattened Arrow names
  must be usable verbatim by LLM agents, including in index-eligible filter positions).

- **RG-028:** `test_prism_describe_ocsf_column_naming_true_emits_class_uid_and_sensor_descriptors` —
  fails until `prism_describe` in `prism-mcp::tools::prism_describe` emits synthesized
  ColumnDescriptors for `class_uid` and `_sensor` when `ocsf_column_naming = true`.

  The test constructs a `SensorSpec` with `ocsf_column_naming = true` and a table with at
  least one Tier-1 column (`ocsf_field == Some`) and one Tier-2 column (`ocsf_field == None`).
  Calls `prism_describe` and asserts ALL SIX of:
  (i) a ColumnDescriptor with `name = "class_uid"` exists in the output with
      `col_type = prism_core::column::ColumnType::Integer` and `nullable = false`;
  (ii) a ColumnDescriptor with `name = "_sensor"` exists in the output with
       `col_type = prism_core::column::ColumnType::String` and `nullable = false`;
  (iii) the `class_uid` ColumnDescriptor appears AFTER all Tier-1 flattened-name
        ColumnDescriptors and AFTER the single `raw_extensions` ColumnDescriptor (ordering);
  (iv) the `_sensor` ColumnDescriptor appears alongside `class_uid` (last two descriptors);
  (v) the `class_uid` ColumnDescriptor has `description = "OCSF event class identifier
      derived from sensor TOML ocsf_class. Example: 3004 for entity_management (audit_logs),
      2004 for detection_finding (alerts, device_alert_relations), 5001 for inventory_info
      (devices)."` (verbatim canonical string from ADR-058 §G / BC-2.16.003);
  (vi) the `_sensor` ColumnDescriptor has `description = "Sensor identifier. Value:
       <sensor_id> (e.g., 'claroty')."` (verbatim canonical string from ADR-058 §G /
       BC-2.16.003).

  Wire-shape assertion: serialize the `prism_describe` output to JSON and assert both
  ColumnDescriptor entries appear with the exact `name`, `col_type`, `nullable`, and
  `description` values. The assertion MUST be at the wire level (serialized JSON output),
  not only on pre-serialization Rust structures (wire-shape assertion discipline,
  CLAUDE.md §Conventions).

  **RED condition:** Prior to the fix, `prism_describe` does not emit `class_uid` or `_sensor`
  ColumnDescriptors — these synthesized columns exist in the Arrow schema produced by
  `pipeline_result_to_record_batch` but are invisible to the LLM agent. Assertions (i)-(vi)
  all fail.

  Place in `crates/prism-mcp/tests/` alongside RG-025.
  Covers AC-015. Traces to BC-2.16.003 §Interpretation A (synthesized columns produced by
  `pipeline_result_to_record_batch` must be advertised by `prism_describe` so the LLM
  agent can use them as filter targets).

- **RG-Q-001:** `test_BC_2_11_016_RG_Q_001` — when `ocsf_column_naming = true`, a
  `SELECT finding_info_uid FROM claroty_alerts` query passes E-QUERY-038 plan-time gate
  (`Ok(())`) — OCSF-flattened Arrow name is present in `TableRegistry`. RED condition:
  prior to Fix A, `TableRegistry` is seeded with raw `col.name` values (e.g., `"id"`);
  `"finding_info_uid"` is absent — E-QUERY-038 fires incorrectly (FP-001 violation).
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-016 sub-case (b). Traces to BC-2.11.016 EC-11-079 postcondition (b).

- **RG-Q-002:** `test_BC_2_11_016_RG_Q_002` — when `ocsf_column_naming = true`, a
  `SELECT time FROM claroty_alerts` query passes E-QUERY-038 (`Ok(())`) — confirms the
  full OCSF-mode `TableRegistry` registered set, not only a single column. RED condition:
  `"time"` absent from raw-col.name registry — E-QUERY-038 fires.
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-016 sub-case (b). Traces to BC-2.11.016 EC-11-079 postcondition (b).

- **RG-Q-003:** `test_BC_2_11_016_RG_Q_003` — when `ocsf_column_naming = true`, a
  `SELECT finding_info_uid FROM claroty_alerts WHERE finding_info_uid = 'x'` query
  passes E-QUERY-038 AND E-QUERY-002/041 type-compat gate (`Ok(())`) — the column type
  is resolved by OCSF-flattened name `"finding_info_uid"`. RED condition: `"finding_info_uid"`
  absent from raw registry — E-QUERY-038 fires; type-compat lookup fails by raw name.
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-016 sub-case (b) and AC-017 sub-case (c). Traces to BC-2.11.016 EC-11-079
  postconditions (b) and (c).

- **RG-Q-004:** `test_BC_2_11_016_RG_Q_004` — when `ocsf_column_naming = true`, a
  `SELECT id FROM claroty_alerts` query (where `"id"` is the raw Tier-1 col.name for the
  column with `ocsf_field = "finding_info.uid"`) fails with E-QUERY-038. The
  `available_columns` payload MUST contain `"finding_info_uid"` (OCSF-flattened) and MUST
  NOT contain `"id"` (raw col.name). RED condition: prior to Fix A, `"id"` is in the
  registry — gate passes (false negative, correct behavior inverted).
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-016 sub-case (a) Tier-1. Traces to BC-2.11.016 EC-11-079 postcondition (a).

- **RG-Q-005:** `test_BC_2_11_016_RG_Q_005` — when `ocsf_column_naming = true`, a
  `SELECT category FROM claroty_alerts` query (where `"category"` is a Tier-2
  `ocsf_field == None` column aggregated into `raw_extensions`) fails with E-QUERY-038.
  The `available_columns` payload MUST contain `"raw_extensions"` and MUST NOT contain
  `"category"`. RED condition: prior to Fix A, `"category"` is in the raw registry — gate
  passes (false negative).
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-016 sub-case (a) Tier-2. Traces to BC-2.11.016 EC-11-079 postcondition (a).

- **RG-Q-006:** `test_BC_2_11_016_RG_Q_006` — when `ocsf_column_naming = true`, the
  E-QUERY-038 `available_columns` payload for a nonexistent-column query MUST contain
  ONLY OCSF-flattened names and synthesized column names (`class_uid`, `_sensor`,
  `raw_extensions`); no raw `col.name` value may appear. Assert wire-shape by serializing
  the error payload to JSON (wire-shape assertion discipline, CLAUDE.md §Conventions). RED
  condition: raw col.names currently appear in `available_columns`.
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-016 `available_columns` constraint. Traces to BC-2.11.016 EC-11-079
  postcondition (a) `available_columns` MUST NOT contain raw col.names.

- **RG-Q-007:** `test_BC_2_11_016_RG_Q_007` — when `ocsf_column_naming = false` (the
  default), a `SELECT id FROM claroty_alerts` query against the raw-col.name `TableRegistry`
  passes E-QUERY-038 (`Ok(())`) — flag-false green-lock, no regression from existing
  behavior. This test MUST PASS before Fix A AND after Fix A (backward-compat assertion).
  If Fix A incorrectly applies OCSF-mode to flag-false tables, this test catches the
  regression.
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-016 flag-false backward-compatibility. Traces to BC-2.11.016 EC-11-079
  invariant (flag-false: existing behavior unchanged, FP-001 preserved).

- **RG-Q-008:** `test_BC_2_11_016_RG_Q_008_multitenant_ocsf_head_projection` — multi-tenant
  (`resolved_spec_map`) OCSF head-gate: when `ocsf_column_naming = true`, a SELECT using the
  OCSF-flattened name (e.g., `finding_info_uid`) resolves via `check_column_availability`
  (which calls the shared helper `ocsf_or_raw_column_names_for_table` in `engine.rs`), and a
  SELECT using the raw `col.name` (e.g., `id`) is rejected with E-QUERY-038 with
  `available_columns` listing only OCSF-flattened names. This test exercises the
  multi-tenant code path through `resolved_spec_map` — distinct from the single-tenant
  `TableRegistry` path exercised by RG-Q-001..007.
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-016 (multi-tenant head-gate path) and AC-018 (name-agreement invariant — head
  surface). Traces to BC-2.11.016 EC-11-079 postcondition (a) and (b): OCSF-flattened name
  resolves; raw col.name rejected with OCSF `available_columns`.

- **RG-Q-009:** `test_BC_2_11_016_RG_Q_009_multitenant_ocsf_pipe_stage` — multi-tenant OCSF
  PIPE stage: when `ocsf_column_naming = true`, `| where message` (OCSF-flattened name)
  resolves in the pipe stage and `| where description` (raw col.name) is rejected. Exercises
  `get_initial_available_columns` (Site E of the TD-VSDD-060 5-site sweep — the pipe-stage
  binding seed, which calls `ocsf_or_raw_column_names_for_table` via the multi-tenant
  `resolved_spec_map` code path). Verifies that the shared helper is consistently applied
  across BOTH the head gate (`check_column_availability`) and the pipe-stage seed
  (`get_initial_available_columns`) under multi-tenant dispatch.
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-016 (multi-tenant pipe-stage path) and AC-018 (name-agreement invariant — pipe
  surface). Traces to BC-2.11.016 EC-11-079 postcondition (a): OCSF-flattened pipe
  predicate resolves; raw col.name pipe predicate rejected.

- **RG-Q-010:** `test_BC_2_11_016_zero_col_ocsf_table_st_gate_accepts_class_uid_and_sensor`
  (prism-query): when `ocsf_column_naming = true` and a table has zero Tier-1 columns (no
  column has an `ocsf_field` declaration), the E-QUERY-038 plan-gate accepts `class_uid`
  and `_sensor` as valid column references (returning `Ok(())`).
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-019. Traces to BC-2.11.016 EC-11-080 postcondition 1 (zero-column OCSF table
  registers class_uid + _sensor in the plan-gate available set; ADR-058 §J6).

- **RG-Q-011:** `test_BC_2_11_016_zero_col_ocsf_table_st_gate_rejects_raw_col_name`
  (prism-query): when `ocsf_column_naming = true` and a table has zero Tier-1 columns, the
  available set equals `["_sensor", "class_uid"]` ONLY — no raw `col.name` value appears.
  E-QUERY-038 rejects any reference to a raw col.name for such a table.
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-019. Traces to BC-2.11.016 EC-11-080 postcondition 2 (available set contains
  exactly class_uid + _sensor; no raw col.name appears; ADR-058 §J6).

- **RG-Q-012:** `test_BC_2_16_003_ocsf_collision_j2_reserved_name_rejected_at_spec_load`
  (prism-spec-engine): a sensor TOML with a Tier-1 column whose `ocsf_field` flattens via
  `ocsf_field_to_arrow_name` to a reserved synthesized name (`class_uid`, `category_uid`,
  `_sensor`, or `raw_extensions`) causes `parse_and_validate_spec_toml` to return `Err`
  where the error string contains `"E-SPEC-030"` and `"[§J2]"`.
  Place in `crates/prism-spec-engine/src/add_sensor_spec.rs` `#[cfg(test)] mod tests`.
  Covers AC-021. Traces to BC-2.16.003 EC-016-013-032 postcondition 1 ([§J2] reserved-name
  collision rejected at spec-load with E-SPEC-030; ADR-058 §J7).

- **RG-Q-013:** `test_BC_2_16_003_ocsf_collision_j4_intra_table_duplicate_rejected_at_spec_load`
  (prism-spec-engine): a sensor TOML with two Tier-1 columns in the same table whose
  `ocsf_field` values both flatten to the same arrow name (intra-table duplicate) causes
  `parse_and_validate_spec_toml` to return `Err` where the error string contains
  `"E-SPEC-030"` and `"[§J4]"`.
  Place in `crates/prism-spec-engine/src/add_sensor_spec.rs` `#[cfg(test)] mod tests`.
  Covers AC-021. Traces to BC-2.16.003 EC-016-013-032 postcondition 1 ([§J4] intra-table
  duplicate rejected at spec-load with E-SPEC-030; ADR-058 §J7).

- **RG-Q-014:** `test_BC_2_16_003_ocsf_collision_j1_shadow_rejected_at_spec_load`
  (prism-spec-engine): a sensor TOML where a Tier-1 arrow name shadows another column's
  raw `col.name` within the same table (§J1 shadow collision) causes
  `parse_and_validate_spec_toml` to return `Err` where the error string contains
  `"E-SPEC-030"` and `"[§J1]"`.
  Place in `crates/prism-spec-engine/src/add_sensor_spec.rs` `#[cfg(test)] mod tests`.
  Covers AC-021. Traces to BC-2.16.003 EC-016-013-032 postcondition 1 ([§J1] shadow
  collision rejected at spec-load with E-SPEC-030; ADR-058 §J7).

- **RG-Q-015:** `test_ocsf_projected_names_all_surfaces_agree`
  (prism-query table_registry): `registry.columns_for_table(table_id)` returns a
  column-name set that is byte-equal (sorted) to
  `ocsf_projected_column_names(table_spec, ocsf_column_naming=true)` from
  `prism_spec_engine::column_mapping`. Enforces the consolidated-projection invariant —
  the registry and the shared helper produce identical name sets for the same table.
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-020. Traces to ADR-058 §I7 (Consolidated-Projection Invariant: the shared
  helpers are the single authoritative source; all projection surfaces MUST agree with them).
  **§I7 shape-exception sites (LOCAL pass-1 M2 closure):** RG-Q-015 now also binds the two
  ADR-058 §I7 shape-exception sites that were previously untested: (a) a prism-mcp assertion
  that `build_ocsf_column_descriptors` name-set equals `ocsf_projected_column_names` output
  (the descriptor names produced by `prism_describe` must agree with the shared helper); and
  (b) a prism-bin assertion that `pipeline_result_to_record_batch` Arrow schema field-names
  equal `ocsf_projected_column_names` output (the Arrow schema field names produced at query
  time must agree with the shared helper). These are sub-assertions within RG-Q-015 — no new
  RG-Q ID was assigned. Without these sub-assertions RG-Q-015 was tautological (it asserted
  registry == helper, but did not verify that the surfaces called the helper correctly).

- **RG-Q-016:** `test_BC_2_16_003_ocsf_collision_j1_shadow_tier1_vs_tier1_rejected_at_spec_load`
  (prism-spec-engine): a sensor TOML where a Tier-1 column's flattened arrow name (via
  `ocsf_field_to_arrow_name`) equals ANOTHER Tier-1 column's raw `col.name` in the same
  table (§J1 Tier-1-vs-Tier-1 shadow sub-case) causes `parse_and_validate_spec_toml` to
  return `Err` where the error string contains `"E-SPEC-030"` and `"[§J1]"`.
  Place in `crates/prism-spec-engine/src/add_sensor_spec.rs` `#[cfg(test)] mod tests`.
  This closes LOCAL pass-1 H1: the original §J1 validator in T-30 only checked
  Tier-1-vs-Tier-2 shadowing (a Tier-1 flattened name equalling a Tier-2 column's raw
  col.name); the Tier-1-vs-Tier-1 case (a Tier-1 flattened name equalling ANOTHER Tier-1
  column's raw col.name) was not covered. Example: Column A has `col.name = "finding_uid"` and
  `ocsf_field = Some("other.field")`; Column B has `col.name = "other_col"` and
  `ocsf_field = Some("finding.uid")` — B's flattened name `"finding_uid"` equals A's
  `col.name = "finding_uid"` (§J1 Tier-1-vs-Tier-1). The spec-load validator MUST detect this
  and return `Err(E-SPEC-030 [§J1])`. The existing §J1 Tier-1-vs-Tier-2 sub-case (covered by
  RG-Q-014) is a distinct code path; this test targets the Tier-1-vs-Tier-1 sub-case gap.
  Covers AC-021. Traces to BC-2.16.003 EC-016-013-032 postcondition 1 ([§J1] Tier-1-vs-Tier-1
  shadow rejected at spec-load with E-SPEC-030; ADR-058 §J7).

- **RG-Q-017:** `test_BC_2_11_016_zero_tier1_with_tier2_projects_raw_extensions_and_emits_warning`
  (prism-query): when `ocsf_column_naming = true` and a sensor table has zero Tier-1 columns
  (no column has `ocsf_field == Some(...)`) but ≥1 Tier-2 column (at least one column has
  `ocsf_field == None`), the E-QUERY-038 plan-gate MUST accept `class_uid`, `_sensor`, AND
  `raw_extensions` as valid column references; AND the `ocsf.zero_tier1_table` WARN event
  MUST be emitted exactly ONCE at spec-load/registration for that table (not per-query).

  The test constructs a `SensorSpec` with `ocsf_column_naming = true` and a table with
  zero Tier-1 columns and at least one Tier-2 column (e.g., one column with `ocsf_field = None`).
  Registers this spec in a `TableRegistry` (or invokes `register_sensor`). Asserts ALL THREE of:
  (i) E-QUERY-038 plan-gate returns `Ok(())` for `SELECT class_uid`, `SELECT _sensor`, AND
      `SELECT raw_extensions` against this table — the available set is exactly
      `["_sensor", "class_uid", "raw_extensions"]` (Tier-2 data preserved via raw_extensions,
      MUST NOT be dropped);
  (ii) E-QUERY-038 plan-gate returns `Err` for any reference to a raw `col.name` value from
      this table — no raw col.name appears in the available set;
  (iii) a `tracing::warn!` event with `event_type = "ocsf.zero_tier1_table"` was emitted
      exactly ONCE during registration (use `tracing_test` subscriber); fields `sensor_id`
      and `table_name` match the registered table's sensor and table name.

  **RED condition:** Prior to T-31, `register_sensor` for a zero-Tier-1-with-Tier-2 table:
  (a) registers only `["_sensor", "class_uid"]` and NOT `"raw_extensions"` (assertion (i)
      partially fails — `raw_extensions` query returns E-QUERY-038 incorrectly);
  (b) emits no `ocsf.zero_tier1_table` WARN (assertion (iii) fails).
  Without T-31, all three assertions fail.

  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Covers AC-019 A+W sub-case. Traces to BC-2.11.016 EC-11-080 sub-case A+W (v1.30):
  zero-Tier-1-with-Tier-2 table projects `["_sensor", "class_uid", "raw_extensions"]` + emits
  `ocsf.zero_tier1_table` WARN at registration (ADR-058 §J6; BC-2.16.002 `ocsf.zero_tier1_table`
  catalog row v2.34).

### BC-5.38.001 Density Check

Red Gate test count: **46** (RG-001..RG-027, RG-PD-001, RG-028, RG-Q-001..RG-Q-017).
Acceptance criteria: 21 (AC-001..AC-021). AC-008 is an `#[ignore]`'d test update — its
Red Gate is RG-005 (same mechanism: Arrow field name must be `device_uid` not `uid`).

Density: 46 RGTs / 21 ACs = **2.19 ≥ 0.5** — compliant with BC-5.38.001.

Note: AC-005 (claroty.sensor.toml — ocsf_column_naming + KF-01..KF-04, OQ-005, KF-06..KF-12 + flag + §J3 shadow fix)
is validated by wire-shape RGs that require the corrected TOML to pass: RG-014..RG-022
collectively assert the TOML corrections at the RecordBatch level. OQ-005 is asserted by
RG-021 (flipped from KF-05 raw_extensions to OQ-005 metadata_uid Tier-1); KF-06 is asserted by RG-022. AC-008 (e2e test update) has no standalone failing
Red Gate — RG-005 and RG-006 verify the column naming logic non-live.
RG-009 covers EC-009 (intra-table flattening collision detection). RG-010 covers EC-010
(flag-transition name shadowing). RG-011 covers AC-009 sub-obligations (a)+(b) at the
resolver level (`select_by_class_name` entity_management + inventory_info arms).
RG-012 covers AC-009 sub-obligation (c) Armis `select()` arm (TD-VSDD-097 dim-1 sibling).
RG-023 covers AC-009 sub-obligation (c) Claroty `select()` arm (completes (c) coverage;
RG-012 covers one half, RG-023 covers the other).
RG-013 covers AC-009 data-loss prevention (note→comment under entity_management 3004).
RG-014 covers AC-010 assertion 2 (KF-08/09/10 reserved fields → raw_extensions).
RG-015 covers AC-010 assertion 1 (KF-03/04/12 3-field alerts wire-shape).
RG-016 covers AC-009 sub-obligation (b) integration (audit_logs class_uid = 3004 wire-shape).
RG-017 covers AC-009 sub-obligation (b) regression guard (devices class_uid = 5001).
RG-018 covers AC-011 (process-gap warn). RG-019 covers AC-010 assertion 3 (KF-11).
RG-020 covers AC-010 assertion 4 (KF-07). RG-021 covers AC-010 assertion 5 (OQ-005 metadata_uid Tier-1).
RG-022 covers AC-010 assertion 6 (KF-06).
RG-024 covers AC-012 (`pipeline_result_to_record_batch` gains `sensor_spec: &SensorSpec`
parameter; both `ocsf_column_naming = true` and `ocsf_column_naming = false` branches
exercised from the threaded-parameter path).
RG-025 covers AC-006 Tier-2 prohibition (no phantom ColumnDescriptor for `ocsf_field == None`
columns) and AC-007b `raw_extensions` ColumnDescriptor four-field shape emission; traces to
ADR-058 §G / BC-2.16.003 §Interpretation A EC-016-013-027 / POL-38 mandate anchor.
RG-026 covers AC-007c (multi-valued array source field serialized as compact JSON-list string
in raw_extensions — wire-shape assertion at serialized output level; traces to
BC-2.16.003 EC-016-013-028 + ADR-058 §B2/§I2).
RG-027 covers AC-013 (§J2 synthesized-name fail-closed guard): `pipeline_result_to_record_batch`
MUST return `Err(ArrowError::SchemaError(...))` when any sensor-table `ocsf_field` flattens
to a synthesized/reserved name: `class_uid`, `category_uid`, `_sensor`, or `raw_extensions`
(ADR-058 §J2 + BC-2.16.003 EC-016-013-029).
RG-PD-001 covers AC-014 (push-down filter on OCSF-flattened `time` Arrow name recognized as
INDEX-eligible by `extract_time_window_from_ast` in `prism-query::pushdown`).
RG-028 covers AC-015 (prism_describe emits `class_uid` + `_sensor` synthesized ColumnDescriptors
after Tier-1 and `raw_extensions` descriptors under `ocsf_column_naming = true`).
RG-Q-001..RG-Q-007 cover AC-016 (OCSF-mode column resolution), AC-017 (type-compat by OCSF
name), and AC-018 (describe/select/query name-agreement invariant) per BC-2.11.016 EC-11-079.
RG-Q-008 covers AC-016 multi-tenant head-gate path (`resolved_spec_map` →
`check_column_availability` via shared helper `ocsf_or_raw_column_names_for_table`).
RG-Q-009 covers AC-016/AC-018 multi-tenant pipe-stage path (`get_initial_available_columns`
— Site E of the TD-VSDD-060 5-site sweep; same shared helper seeded consistently).
RG-Q-010 covers AC-019 (zero-column OCSF table: class_uid + _sensor accepted by E-QUERY-038).
RG-Q-011 covers AC-019 (zero-column OCSF table: available set = class_uid + _sensor only).
RG-Q-012 covers AC-021 ([§J2] reserved-name collision → E-SPEC-030 at spec-load; ADR-058 §J7).
RG-Q-013 covers AC-021 ([§J4] intra-table duplicate → E-SPEC-030 at spec-load; ADR-058 §J7).
RG-Q-014 covers AC-021 ([§J1] shadow collision → E-SPEC-030 at spec-load; ADR-058 §J7).
RG-Q-015 covers AC-020 (consolidated-projection invariant: registry name-set byte-equal to
shared helper `ocsf_projected_column_names`; ADR-058 §I7).
RG-Q-016 covers AC-021 ([§J1] Tier-1-vs-Tier-1 shadow → E-SPEC-030 at spec-load; ADR-058 §J7
§J1 sub-case: Tier-1 flattened arrow name equals another Tier-1 column's raw col.name).
RG-Q-017 covers AC-019 A+W sub-case (zero-Tier-1-with-Tier-2 OCSF table: available set =
`["_sensor", "class_uid", "raw_extensions"]`; `ocsf.zero_tier1_table` WARN emitted ONCE at
registration; BC-2.11.016 EC-11-080 sub-case A+W; BC-2.16.002 `ocsf.zero_tier1_table` catalog
row v2.34; ADR-058 §J6).

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
`prism-spec-engine::column_mapping` (ADR-058 §I1 canonical home). Both
`prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe` import it from
`prism-spec-engine::column_mapping` — no cycle, both crates already depend on prism-spec-engine.
The function replaces all occurrences of `.` (dot) in `ocsf_field` with `_` (underscore). Examples:

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

**Parameter threading note (ADR-058 §D1):** `sensor_spec` in the snippet below is
NOT a free variable — it is the new `sensor_spec: &SensorSpec` parameter added to
`pipeline_result_to_record_batch` by AC-012. The function signature after AC-012 is:

```
fn pipeline_result_to_record_batch(
    ..existing parameters..,
    sensor_spec: &SensorSpec,   // threaded from fetch(); carries ocsf_column_naming flag
) -> Result<RecordBatch, ArrowError>
```

This is ADR-022 §C wiring: adding a previously absent parameter from the `fetch()` call
site. `fetch()` in `spec_driven_adapter` threads `&self.sensor_spec.spec`
(`SpecDrivenSensorAdapter.sensor_spec` is `Arc<ResolvedSensorSpec>`;
`ResolvedSensorSpec.spec` is the `SensorSpec` carrying `ocsf_column_naming`).
No placeholder construction is permitted. AC-012 specifies
the full caller enumeration and the Red Gate test (RG-024) that enforces the signature.

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

This matches ADR-058 §I1 (Step 2: field name computation inside the function body; the
new parameter addition is Step 1 per AC-012) combined
with §I2 (raw_extensions routing for `ocsf_field == None` columns, owned by
`pipeline_result_to_record_batch`). The `.unwrap_or_else(|| col.name.clone())` fallback in
the `ocsf_column_naming = true` branch applies only to columns where `col.ocsf_field == Some`:
columns with `col.ocsf_field == None` are diverted to `raw_extensions` aggregation by
`pipeline_result_to_record_batch` per §I2 BEFORE individual-field naming runs and never
reach `Field::new(&arrow_name, ...)`. When `ocsf_column_naming = true` and `col.ocsf_field`
is `Some("finding_info.uid")`, the Arrow schema field is named `"finding_info_uid"`.

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

### AC-005: claroty.sensor.toml receives ocsf_column_naming = true AND all KF-01..KF-12 corrections

`crates/prism-sensors/specs/claroty.sensor.toml` receives all of the following changes
in the same TOML edit (all MUST land together — setting the flag without the ocsf_field
corrections would cause Claroty to fail closed at runtime under the EC-010 / §J2 shadow
check, and would emit semantically incorrect OCSF events for any session that proceeds
past the shadow gate):

**14 TOML changes enumerated (KF-01..KF-04, OQ-005, KF-06..KF-12 + flag + §J3 shadow fix):**

1. **Flag**: Add `ocsf_column_naming = true` at the sensor level (alongside `sensor_id`,
   `auth_type`, etc.). All four Claroty tables use OCSF-flattened Arrow field names after
   this change. CrowdStrike, Armis, Cyberint TOMLs are NOT modified.

2. **KF-01 (TOML part)**: `audit_logs` table: change `ocsf_class` from `"audit_activity"`
   to `"entity_management"`. (The `class_selector.rs` code fix is covered by AC-009.)
   Root cause: `"audit_activity"` is absent from OCSF v1.7.0; `entity_management`
   (class_uid 3004) has the `comment` attribute required for the `note → comment` mapping;
   `account_change` (3001) lacks `comment`, causing silent data loss.

3. **KF-02**: `devices` table: change `ocsf_class` from `"device"` to `"inventory_info"`.
   Root cause: `"device"` is an OCSF object, not a class; `inventory_info` (class_uid 5001)
   is the correct class; `device.*` paths resolve via `inventory_info.device` required attr.

4. **KF-03**: `alerts` table, `id` column: change `ocsf_field` from `"finding.uid"` to
   `"finding_info.uid"`. Root cause: `detection_finding` has required `finding_info`
   attribute; no bare `finding` attribute exists in OCSF v1.7.0.

5. **KF-04**: `alerts` table, `alert_name` column: change `ocsf_field` from
   `"finding.title"` to `"finding_info.title"`. Same root error as KF-03.

6. **OQ-005 (human decision 2026-08-21)**: `audit_logs` table, `id` column: set
   `ocsf_field = "metadata.uid"` (supersedes the prior KF-05 PO decision to remove
   `ocsf_field`). Human decision: `metadata.uid` is the canonical OCSF v1.7.0 field for
   unique event record identifier; `audit_logs.id` is the Claroty audit record ID and maps
   semantically to `metadata.uid`; the value routes as a Tier-1 Arrow column named
   `metadata_uid` rather than being placed in `raw_extensions`.

7. **KF-06 (PO decision)**: `devices` table, `device_type` column: change `ocsf_field`
   from `"device.type_name"` to `"device.type_label"`. PO decision: vendor-extended;
   `device.type_name` is absent from OCSF v1.7.0 device object; OT subcategory
   ("PLC", "HMI") is demo-critical for filtering; follows §J3 vendor-extension precedent.

8. **KF-07**: `device_alert_relations` table, `alert_id` column: change `ocsf_field`
   from `"finding.uid"` to `"finding_info.uid"`. Same root error as KF-03.

9. **KF-08**: `alerts` table, `category` column: remove `ocsf_field`. Root cause:
   `class_name` is OCSF-computed from `class_uid`; a vendor value would overwrite
   "Detection Finding" and corrupt OCSF class metadata.

10. **KF-09**: `alerts` table, `alert_type_name` column: remove `ocsf_field`. Root cause:
    `type_name` is OCSF-computed from `type_uid`; vendor value corrupts OCSF class metadata.

11. **KF-10**: `alerts` table, `devices_count` column: remove `ocsf_field`. Root cause:
    OCSF `count` = event dedup counter; `devices_count` = affected device count; distinct
    semantics — wrong field reused.

12. **KF-11**: `audit_logs` table, `category` column: remove `ocsf_field`. Root cause:
    `category_name` is OCSF-computed from `category_uid`; vendor value corrupts OCSF
    category metadata.

13. **KF-12**: `alerts` table, `updated_time` column: change `ocsf_field` from `"end_time"`
    to `"finding_info.modified_time"`. Root cause: `updated_time` = record last-modified;
    `finding_info.modified_time` confirmed in OCSF v1.7.0; `end_time` = event end time
    (different semantic).

14. **§J3 shadow fix**: `devices` table, `device_category` column: change `ocsf_field`
    from `"device.type"` to `"device.type_category"`. Resolves the §J2 shadow collision:
    `"device.type"` flattens to `"device_type"` which equals column `device_type`'s
    `col.name`. After fix, `device_category` flattens to `"device_type_category"` — no
    shadow. The collision is verified by RG-010 (fails before this fix, passes after).

**Contracted mapping tables (source of truth: BC-2.16.003 §Claroty Contracted OCSF Mappings):**

`alerts` table — `ocsf_class = "detection_finding"` (class_uid 2004):

| col.name | Contracted ocsf_field | Arrow name (Interp. A) | Note |
|----------|-----------------------|------------------------|------|
| `id` | `finding_info.uid` | `finding_info_uid` | KF-03 |
| `alert_type_name` | (none — removed) | `raw_extensions` | KF-09 |
| `category` | (none — removed) | `raw_extensions` | KF-08 |
| `status` | `status` | `status` | VALID |
| `detected_time` | `time` | `time` | VALID |
| `updated_time` | `finding_info.modified_time` | `finding_info_modified_time` | KF-12 |
| `devices_count` | (none — removed) | `raw_extensions` | KF-10 |
| `description` | `message` | `message` | VALID |
| `alert_class` | (none) | `raw_extensions` | no ocsf_field declared |
| `ot_devices_count` | (none) | `raw_extensions` | no ocsf_field declared |
| `alert_name` | `finding_info.title` | `finding_info_title` | KF-04 |

`audit_logs` table — `ocsf_class = "entity_management"` (class_uid 3004; KF-01):

| col.name | Contracted ocsf_field | Arrow name (Interp. A) | Note |
|----------|-----------------------|------------------------|------|
| `id` | `metadata.uid` | `metadata_uid` | OQ-005 human decision 2026-08-21 |
| `action` | `activity_name` | `activity_name` | VALID |
| `user_display_name` | `actor.user.name` | `actor_user_name` | VALID |
| `category` | (none — removed) | `raw_extensions` | KF-11 |
| `timestamp` | `time` | `time` | VALID |
| `details` | `message` | `message` | VALID |
| `username` | `actor.user.uid` | `actor_user_uid` | VALID; `column_type = "string"` — Rule 1 preempts `uid` numeric-suffix heuristic |
| `note` | `comment` | `comment` | VALID; requires entity_management (3004) |

`devices` table — `ocsf_class = "inventory_info"` (class_uid 5001; KF-02):

| col.name | Contracted ocsf_field | Arrow name (Interp. A) | Note |
|----------|-----------------------|------------------------|------|
| `uid` | `device.uid` | `device_uid` | VALID |
| `asset_id` | `device.instance_uid` | `device_instance_uid` | VALID |
| `device_category` | `device.type_category` | `device_type_category` | §J3 shadow fix |
| `device_type` | `device.type_label` | `device_type_label` | KF-06 PO decision |
| `risk_score` | `risk_score` | `risk_score` | VALID; self-match legal |
| `retired` | `status_code` | `status_code` | VALID |
| `device_name` | `device.name` | `device_name` | VALID |
| `os_category` | `device.os.name` | `device_os_name` | VALID |

`device_alert_relations` table — `ocsf_class = "detection_finding"` (class_uid 2004):

| col.name | Contracted ocsf_field | Arrow name (Interp. A) | Note |
|----------|-----------------------|------------------------|------|
| `device_uid` | `device.uid` | `device_uid` | VALID |
| `alert_id` | `finding_info.uid` | `finding_info_uid` | KF-07 |
| `device_alert_detected_time` | `time` | `time` | VALID |
| `device_risk_score` | `risk_score` | `risk_score` | VALID |
| `alert_note` | `comment` | `comment` | VALID |
| `device_alert_status` | `status` | `status` | VALID |

Post-corrections ocsf_field count: 27 (alerts: 6, audit_logs: 7, devices: 8, dar: 6).
Note: OQ-005 raises audit_logs from 6→7 (`id` now Tier-1 via `metadata.uid`).
Shadow check: no flattened name equals any other column's col.name in any Claroty table.
RG-009 passes (all flattened names within each table are distinct).
RG-010 passes (zero shadow collisions across all four tables).

(traces to BC-2.16.003 §Claroty Contracted OCSF Mappings: these are the
contracted-correct ocsf_field values per ADR-058 §K4 corrections KF-01..KF-12;
traces to BC-2.01.013 EC-01-025 which moves NON-CONFORMANT→CONFORMANT for Claroty
after this story merges; traces to BC-2.16.003 EC-016-013-012: two sensors both
mapping `device_ip → ocsf_field = "device.ip"` are both queryable as `device_ip`
once each sensor enables the flag)

### AC-006: prism_describe emits Tier-1 ColumnDescriptors for ocsf_field==Some columns and MUST NOT emit individual ColumnDescriptors for ocsf_field==None columns when ocsf_column_naming=true

Under `ocsf_column_naming = true`, `prism_describe` emits `ColumnDescriptor` entries in two
tiers per ADR-058 §G / BC-2.16.003 §Interpretation A:

**Tier-1** (`ocsf_field == Some(path)`): `prism_describe` MUST emit a `ColumnDescriptor` with:
- `name = ocsf_field_to_arrow_name(ocsf_field)` (the queryable Arrow identifier,
  e.g., `"finding_info_uid"` for `ocsf_field = "finding_info.uid"`)
- `description = ocsf_field` (original dotted OCSF path preserved as semantic annotation,
  e.g., `"finding_info.uid"`)

**Tier-2** (`ocsf_field == None`): `prism_describe` MUST NOT emit an individual
`ColumnDescriptor` for the column — the column's `col.name` is NOT a queryable field when
`ocsf_column_naming = true`; it is aggregated into `raw_extensions`. `prism_describe` MUST
NOT emit any `ColumnDescriptor` whose `name` equals any `ocsf_field == None` column's
`col.name`. Advertising a `col.name` as a `ColumnDescriptor` when the column is actually
aggregated into `raw_extensions` would cause the LLM agent to construct queries that
reference phantom fields and silently return no data.

Under `ocsf_column_naming = false` (Interpretation B), behavior is unchanged: all columns
emitted individually with `name = col.name`, `description = col.ocsf_field`.

LLM agents read `name: "finding_info_uid"` from `prism_describe` output and use it verbatim
in PrismQL queries — no quoting needed.

(traces to BC-2.16.003 §Interpretation A Tier-1 model: `ocsf_field == Some` columns
emit ColumnDescriptor with `name = ocsf_field_to_arrow_name(ocsf_field)`;
ADR-058 §G: Tier-2 prohibition — `prism_describe` MUST NOT emit an individual
ColumnDescriptor for `ocsf_field == None` columns when `ocsf_column_naming = true`;
BC-2.16.003 EC-016-013-027)

### AC-007: Columns without ocsf_field are collected into raw_extensions when flag=true; prism_describe emits exactly one raw_extensions ColumnDescriptor enumerating all source keys

AC-007 covers two related obligations:

**AC-007a — Query engine (`pipeline_result_to_record_batch`):** When
`sensor_spec.ocsf_column_naming = true`, any column with `col.ocsf_field == None` does NOT
appear as an individual Arrow schema field. Instead, these columns' values are collected into
a single Arrow `Utf8` field named `"raw_extensions"` containing a serialized JSON object.

The `raw_extensions` column is queryable via PrismQL: `SELECT raw_extensions FROM claroty_alerts`.

Per ADR-058 §I2: `pipeline_result_to_record_batch` (schema-fields construction loop) is
the synthesis locus for the `raw_extensions` aggregation. `pipeline_result_to_record_batch`
suppresses `ocsf_field == None` columns from the individual-field schema and aggregates
their values into the `"raw_extensions"` Utf8 blob. The aggregation MUST extract each
`ocsf_field == None` column's value via the SAME source_path-aware extraction and ENRICH-1
`Value::Array`→compact-JSON-list-string normalization as first-class columns (BC-2.16.003
EC-016-013-028 reworded; ADR-058 §I2) — NOT a naive `r.get(col.name)`. A multi-valued
array field like `ip_list` with `source_path = "$.ip_list[*]"` produces a compact
JSON-list string (e.g., `"[\"192.168.1.1\",\"10.0.0.1\"]"`) in `raw_extensions`, not a
nested JSON array; reuse the shared extraction/normalization pipeline rather than
bypassing it. The implementer MUST verify which Claroty columns currently have
`col.ocsf_field == None` in `claroty.sensor.toml` at dispatch time and confirm they
go to `raw_extensions` rather than being silently dropped.

**AC-007b — MCP tool (`prism_describe`):** Under `ocsf_column_naming = true`,
`prism_describe` MUST emit exactly ONE `raw_extensions` ColumnDescriptor per
ADR-058 §G / BC-2.16.003 §Interpretation A EC-016-013-027 with all four fields:
- `name = "raw_extensions"`
- `col_type = prism_core::column::ColumnType::Json`
- `nullable = true`
- `description` = a string that (1) identifies it as a JSON object and (2) enumerates every
  source key — the `col.name` of each `ocsf_field == None` column in the queried table.

Example: for Claroty `alerts` (where `alert_type_name`, `category`, `devices_count`,
`alert_class`, `ot_devices_count` have `ocsf_field == None` after KF-08/09/10 corrections),
the `raw_extensions` ColumnDescriptor description MUST enumerate `"alert_type_name"`,
`"category"`, `"devices_count"`, `"alert_class"`, `"ot_devices_count"` as source keys so
the LLM agent knows which vendor fields can be accessed via `raw_extensions`.

This `raw_extensions` ColumnDescriptor is the agent's sole discovery mechanism for
`ocsf_field == None` columns. Without it, vendor data preserved in `raw_extensions`
is invisible to the agent — it cannot know to write `raw_extensions->'category'` unless
`prism_describe` advertises that `"category"` is a source key.

(traces to BC-2.16.003 postcondition §Column Routing: "Columns without an ocsf_field
mapping are preserved in the raw_extensions JSON blob" (AC-007a);
BC-2.16.003 §Interpretation A and EC-016-013-027: `prism_describe` MUST emit
exactly ONE `raw_extensions` ColumnDescriptor with four-field shape
(name + col_type=Json + nullable=true + description enumerating source keys) (AC-007b);
ADR-058 §G (AC-007b); RG-025 (falsifiable Red Gate for AC-007b))

**AC-007c — Multi-valued array fields in `raw_extensions`:** When an `ocsf_field == None`
column's source data is a JSON array (e.g., Claroty `devices.ip_list` with
`source_path = "$.ip_list[*]"`), the array's content MUST be serialized as a
**compact JSON-list string** inside the `raw_extensions` JSON object — NOT as a nested
JSON array, NOT as null. Example: a devices row with `ip_list = ["192.168.1.1", "10.0.0.1"]`
produces `raw_extensions = {"ip_list": "[\"192.168.1.1\",\"10.0.0.1\"]"}` where the
`ip_list` value is the string `"[\"192.168.1.1\",\"10.0.0.1\"]"`.

This compact-string representation preserves the array data without requiring nested-JSON
parsing on the query side and is consistent with the existing `prism-spec-engine`
pattern for multi-valued fields. The LLM agent can extract individual IPs via
`raw_extensions->>'ip_list'` (returns the compact string) and then parse as needed.

`SELECT raw_extensions FROM claroty_devices` on a row with non-empty `ip_list` MUST
produce a `raw_extensions` value where the `"ip_list"` key maps to a compact JSON-list
string; this assertion MUST be verified at the serialized wire-output level (not
pre-serialization Rust structure).

(traces to BC-2.16.003 EC-016-013-028: multi-valued array source fields in raw_extensions
are serialized as compact JSON-list strings; ADR-058 §B2 / §I2: ip_list has
`ocsf_field == None` and routes to raw_extensions; RG-026 (wire-shape Red Gate for AC-007c))

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

### AC-009: class_selector.rs receives KF-01/KF-02 code fix — 4 sub-obligations per ADR-058 §I5

`crates/prism-ocsf/src/class_selector.rs` receives FOUR code changes per ADR-058 §I5
and §K5 Divergence 3. All four MUST land in the same atomic commit:

**Sub-obligation (a) — New constant:**
Add `pub const CLASS_UID_ENTITY_MANAGEMENT: u32 = 3004;` alongside the existing
`CLASS_UID_ACCOUNT_CHANGE`, `CLASS_UID_DETECTION_FINDING`, etc.

**Sub-obligation (b) — Two new `select_by_class_name` arms (Path A — live production):**
`select_by_class_name` is the live production resolver called by `pipeline_result_to_record_batch`
on Path A (the spec-driven Arrow materialization path). It MUST gain two new arms:

1. `"entity_management" => Ok(CLASS_UID_ENTITY_MANAGEMENT)` — resolves the KF-01 corrected
   TOML value `ocsf_class = "entity_management"` to 3004. Without this arm, the KF-01 TOML
   correction causes a regression: the `"audit_activity"` arm disappears from production TOMLs
   but the `"entity_management"` string falls to `Err(...)` → `.unwrap_or(0)` → `class_uid = 0`
   (BASE_EVENT) instead of the current wrong 3001.

2. `"inventory_info" => Ok(CLASS_UID_DEVICE_INVENTORY_INFO)` — resolves the KF-02 corrected
   TOML value `ocsf_class = "inventory_info"` to 5001. Without this arm, the KF-02 TOML
   correction from `"device"` to `"inventory_info"` silently regresses `class_uid` from the
   current 5001 (produced by the existing `"device"` arm) to 0. The existing `"device"` arm
   is retained as a transitional alias.

**Sub-obligation (c) — Two `select()` audit_log arms updated (Path B — forward-compat):**
`select()` is called by `normalize_with_mappers` in `crates/prism-ocsf/src/normalizer.rs`
(zero live production callers today; forward-compat path only per ADR-058 §K5 path-liveness).
Update BOTH arms to return `Ok(CLASS_UID_ENTITY_MANAGEMENT)` (3004):
- `("claroty", "audit_log") => Ok(CLASS_UID_ACCOUNT_CHANGE)` → `Ok(CLASS_UID_ENTITY_MANAGEMENT)`
- `("armis", "audit_log") => Ok(CLASS_UID_ACCOUNT_CHANGE)` → `Ok(CLASS_UID_ENTITY_MANAGEMENT)`

These are forward-compat fixes: Path B has zero production callers today, but when
`normalize_with_mappers` is eventually wired, the Armis and Claroty arms must be correct.
The Armis arm is the TD-VSDD-097 dim-1 sibling sweep — MUST be fixed in the same commit
as the Claroty arm.

**Sub-obligation (d) — Deprecate the now-dead `"audit_activity"` arm:**
After the KF-01 TOML correction, no production TOML will emit `ocsf_class = "audit_activity"`.
The existing arm in `select_by_class_name` for `"audit_activity"` becomes dead code. The
implementer MUST annotate it as a deprecated transitional entry pending removal (e.g.,
`// DEPRECATED: "audit_activity" was a non-OCSF string; no production TOML uses this after
// KF-01 correction (S-ADR058-OCSF-ROUTING-001). Remove after confirming zero TOML instances.`).

**In-file doc table update (F-P1-MED-001 prevention):**
The in-file module doc mapping tables in `class_selector.rs` (the table around module doc
lines documenting audit_activity→3001 and device→5001) MUST be updated to document the
corrected mappings. Stale doc tables that contradict the code are a P1-MED finding class.

**Data-loss rationale**: Under `account_change` (3001), any call to
`set_nested_field(..., "comment", value)` silently no-ops because `account_change` has no
`comment` attribute in its protobuf descriptor. The `note → comment` mapping in Claroty
`audit_logs` therefore drops every note value without error or warning. Under
`entity_management` (3004), the `comment` attribute is present; the mapping succeeds.

**Wire-shape assertion requirement (EC-016-013-023 and EC-016-013-024):**
Sub-obligation (b) resolver-unit tests (RG-011) asserting `Ok(3004)` / `Ok(5001)` are
NECESSARY but NOT SUFFICIENT. Per ADR-058 §I5 wire-shape obligation and CLAUDE.md
§Conventions wire-shape assertion discipline, integration-level wire-shape tests (RG-016,
RG-017) MUST assert the `class_uid` Arrow Int32 column value at the `RecordBatch` level.

(traces to BC-2.16.003 EC-016-013-023 wire-level postcondition: Claroty audit_logs
RecordBatch with `ocsf_class = "entity_management"` must carry `class_uid = 3004` in Arrow;
traces to BC-2.16.003 EC-016-013-024 regression-prevention: devices RecordBatch with
`ocsf_class = "inventory_info"` must carry `class_uid = 5001` in Arrow NOT 0;
traces to ADR-058 §K5 Div-3 + §I5 all four sub-obligations)

### AC-010: Wire-shape assertions on corrected finding_info.* fields and reserved-metadata raw_extensions

Tests covering the KF-03/04/OQ-005/KF-06/07/08/09/10/11/12 corrections MUST assert on the
**serialized JSON output** (the exact bytes the LLM agent consumes via MCP), not only on
pre-serialization Rust structures. Per CLAUDE.md §Conventions wire-shape assertion
discipline, any test covering an MCP-visible surface must include at least one assertion
on the serialized JSON output.

**Required wire-shape assertions (verified by RG-014, RG-015, RG-019, RG-020, RG-021, and RG-022):**

1. **KF-03/04/12 finding_info.* fields** (RG-015): A Claroty alerts record with
   `id = "132"`, `alert_name = "Modbus Violation"`, `updated_time = "2024-01-15T10:30:00Z"`
   processed through `pipeline_result_to_record_batch` with `ocsf_column_naming = true`
   produces an Arrow RecordBatch serialized to JSON where:
   - Arrow field `"finding_info_uid"` contains `"132"` (not absent, not `null`)
   - Arrow field `"finding_info_title"` contains `"Modbus Violation"` (KF-04)
   - Arrow field `"finding_info_modified_time"` contains `"2024-01-15T10:30:00Z"` (KF-12)
   - No Arrow field named `"finding_uid"`, `"finding_title"`, or `"end_time"` exists
     (stale pre-correction names must be absent at the wire level)

2. **KF-08/09/10/11 reserved-metadata in raw_extensions** (RG-014): A Claroty alerts
   record with `category = "OT Security"`, `alert_type_name = "Protocol Violation"`,
   `devices_count = 3` processed through `pipeline_result_to_record_batch` produces JSON
   where:
   - No first-class Arrow field named `"class_name"`, `"type_name"`, or `"count"` exists
   - The `raw_extensions` Arrow field (a JSON blob string) contains keys
     `"category"`, `"alert_type_name"`, `"devices_count"` with the vendor values
   - `"class_name"` and `"type_name"` keys are NOT present in `raw_extensions`
     (they were never written there; vendor values go under their `col.name` keys)

NULL vs absent distinction: an absent `category` column in the raw record must produce
an absent key in `raw_extensions` (not `"category": null`) per BC-2.16.003 §Invariants.

3. **KF-11 `audit_logs` `category` in raw_extensions + entity_management field mappings** (RG-019):
   A Claroty `audit_logs` record with `category = "Authentication"`, `action = "Login"`,
   `note = "reviewed"` processed through `pipeline_result_to_record_batch` with
   `ocsf_column_naming = true` and `ocsf_class = "entity_management"` produces JSON where:
   - No first-class Arrow field named `"category_uid"` or `"category_name"` exists
   - The `raw_extensions` JSON blob contains key `"category"` with value `"Authentication"`
   - Arrow field `"activity_name"` contains `"Login"` (`action` → `activity_name` per
     audit_logs contracted mapping under entity_management 3004)
   - Arrow field `"comment"` contains `"reviewed"` (`note` → `comment` under entity_management 3004)

4. **KF-07 `device_alert_relations` `alert_id` → `finding_info_uid`** (RG-020): A Claroty
   `device_alert_relations` record with `alert_id = "alert-123"` processed through
   `pipeline_result_to_record_batch` with `ocsf_column_naming = true` produces JSON where:
   - Arrow field `"finding_info_uid"` contains `"alert-123"` (KF-07 corrected name)
   - No Arrow field named `"finding_uid"` exists (stale pre-KF-07 name absent)
   - No Arrow field named `"finding.uid"` exists (dotted-path form absent at wire level)

5. **OQ-005 `audit_logs.id` → `metadata_uid` (Tier-1)** (RG-021): A Claroty `audit_logs`
   record with `id = "al-999"` processed through `pipeline_result_to_record_batch` with
   `ocsf_column_naming = true` and `audit_logs.id` having `ocsf_field = "metadata.uid"`
   produces JSON where:
   - Arrow field `"metadata_uid"` (Tier-1 String column) contains `"al-999"` (the id value
     is routed as a top-level first-class Tier-1 column via the ocsf_field mapping to
     `ocsf_field_to_arrow_name("metadata.uid")` = `"metadata_uid"`)
   - No `"id"` key exists in the `raw_extensions` JSON blob (the value is NOT routed to
     raw_extensions — it has `ocsf_field = "metadata.uid"` and is Tier-1)

6. **KF-06 `devices.device_type` → `device_type_label`** (RG-022): A Claroty `devices`
   record with `device_type = "PLC"` processed through `pipeline_result_to_record_batch`
   with `ocsf_column_naming = true` produces JSON where:
   - Arrow field `"device_type_label"` contains `"PLC"` (demo-critical: enables
     `WHERE device_type_label = 'PLC'` per BC-2.16.003 EC-016-013-021)
   - No Arrow field named `"device_type_name"` exists (stale pre-KF-06 path absent)

(traces to BC-2.16.003 §Claroty Contracted OCSF Mappings; traces to CLAUDE.md
§Conventions wire-shape assertion discipline — "any test covering an MCP-visible surface
must include at least one assertion on the serialized JSON output")

### AC-011: pipeline_result_to_record_batch emits ocsf.unknown_class_name WARN on unrecognized class

`pipeline_result_to_record_batch` in `prism-bin::spec_driven_adapter` MUST emit a
structured `tracing::warn!` on the `Err` branch of `EventClassSelector::select_by_class_name`
**before** the `.unwrap_or(0)` fallback that produces `class_uid = 0`. The emission:

```rust
tracing::warn!(
    event_type = "ocsf.unknown_class_name",
    ocsf_class = %table.ocsf_class,
    sensor_id = %sensor_id,
    table_name = %table.table_name,
    "sensor TOML declares unrecognised ocsf_class; class_uid defaulted to 0 (BASE_EVENT)"
);
```

The `.unwrap_or(0)` fallback is RETAINED — `pipeline_result_to_record_batch` continues
to return `Ok(...)` with `class_uid = 0` (graceful degradation preserved; SOUL.md §4
violation is addressed by making the silent fallback visible, not by making it fatal).

**Catalog obligation (SAP-1 / PG-LP11-001):** BC-2.16.002 §Canonical Structured Event
Catalog `ocsf.unknown_class_name` is the authoritative contract for this emission. The
implementer MUST NOT add a new BC-2.16.002 amendment — the row is ALREADY in BC-2.16.002
§Canonical Structured Event Catalog. The implementer only needs to write
the emission code matching the contracted field schema.

**Steady-state behavior:** Expected zero emissions when all sensor TOMLs declare valid
recognized OCSF class names. Non-zero during the TOML correction transition window (e.g.,
KF-01/KF-02 TOML fix merged before the corresponding `select_by_class_name` arms are added).
Recurrence: once per `pipeline_result_to_record_batch` invocation where `select_by_class_name`
returns `Err` (per table batch, NOT per record within the batch).

(traces to BC-2.16.002 §Canonical Structured Event Catalog `ocsf.unknown_class_name`;
traces to ADR-058 §I5 process-gap obligation: SOUL.md #4 silent-fallback observability;
traces to BC-2.16.003 EC-016-013-011 (corrected text: this WARN fires at runtime on the
`Err` branch of `select_by_class_name`, NOT at startup/load-time; the EC-016-013-011
correction in v1.16 removes all load-time/startup language);
SAP-1 standing probe obligation)

### AC-012: pipeline_result_to_record_batch gains sensor_spec: &SensorSpec as an explicit threaded parameter

`pipeline_result_to_record_batch` in `prism-bin::spec_driven_adapter` gains a new explicit
parameter `sensor_spec: &SensorSpec` per ADR-058 §D1. This is ADR-022 §C wiring:
adding a previously absent parameter from the `fetch()` call site — NOT a replacement of
any existing implementation. No placeholder construction.

The function signature becomes:

```
fn pipeline_result_to_record_batch(
    ..existing parameters..,
    sensor_spec: &SensorSpec,   // threaded from fetch(); carries ocsf_column_naming flag
) -> Result<RecordBatch, ArrowError>
```

**Production callers (1):**
- `fetch()` in `prism-bin::spec_driven_adapter` — the call site that threads
  `&self.sensor_spec.spec` (`SpecDrivenSensorAdapter.sensor_spec` is
  `Arc<ResolvedSensorSpec>`; `.spec` is the `SensorSpec` carrying `ocsf_column_naming`).
  Existing callers confirm the accessor: `self.sensor_spec.spec.sensor_id` is already used
  throughout `fetch()`. No placeholder construction.

**Pre-existing test caller in `#[cfg(test)] mod tests` of `spec_driven_adapter.rs` (1 — MUST
be updated to pass `&SensorSpec` with `ocsf_column_naming = false`):**
- `test_BC_2_01_013_crowdstrike_fql_datetime_index_col_string_equality_safe` — currently
  calls `super::pipeline_result_to_record_batch(result, &table, "crowdstrike", &push_down_filters)`
  with 4 positional args. When T-14A adds `sensor_spec: &SensorSpec` as the 5th parameter,
  this call breaks with E0061. Must be updated to pass a `SensorSpec` with
  `ocsf_column_naming = false` (preserving current CrowdStrike behavior — uses `col.name`).

**NEW test callers authored by this story in `#[cfg(test)] mod tests` of `spec_driven_adapter.rs`
(16 — Phase A Red Gate tests; all produce E0061 until T-14A adds the parameter):**
- RG-005 (`test_pipeline_result_to_record_batch_ocsf_flag_true_uses_flattened_names`)
- RG-006 (`test_pipeline_result_to_record_batch_ocsf_flag_false_uses_col_name`)
- RG-008 (`test_spec_driven_adapter_columns_without_ocsf_field_go_to_raw_extensions_schema`)
- RG-009 (`test_pipeline_result_to_record_batch_ocsf_collision_returns_error`)
- RG-010 (`test_pipeline_result_to_record_batch_ocsf_shadow_collision_returns_error`)
- RG-014 (`test_claroty_alerts_reserved_fields_go_to_raw_extensions_not_first_class_columns`)
- RG-015 (`test_claroty_alerts_finding_info_fields_wire_shape`)
- RG-016 (`test_claroty_audit_logs_record_batch_class_uid_is_3004`)
- RG-017 (`test_claroty_devices_record_batch_class_uid_is_5001_regression_guard`)
- RG-018 (`test_pipeline_result_to_record_batch_unknown_ocsf_class_emits_warn`)
- RG-019 (`test_claroty_audit_logs_record_batch_kf11_category_in_raw_extensions`)
- RG-020 (`test_claroty_device_alert_relations_record_batch_finding_info_uid_wire_shape`)
- RG-021 (`test_claroty_audit_logs_id_column_goes_to_raw_extensions_not_activity_uid`)
- RG-022 (`test_claroty_devices_device_type_produces_device_type_label_arrow_field`)
- RG-026 (`test_claroty_devices_ip_list_in_raw_extensions_is_compact_json_list_string`)
- RG-027 (`test_pipeline_result_to_record_batch_ocsf_field_flattens_to_reserved_name_returns_error`)

**New test caller (1):**
- RG-024 (`test_pipeline_result_to_record_batch_sensor_spec_parameter_gates_both_branches`)

**Call-site confirmation (TD-VSDD-060):** Before committing, implementer runs
`rg 'pipeline_result_to_record_batch' crates/prism-bin/ crates/prism-mcp/` to confirm
no additional callers exist outside this enumeration. The function is not `pub` — it is
`prism-bin`-internal; no callers in other crates are expected.

(traces to ADR-058 §D1: `pipeline_result_to_record_batch` MUST gain `SensorSpec` as
an explicit parameter threaded from `fetch()`; traces to ADR-022 §C: wiring not redesign —
adding a previously absent parameter is in-scope plumbing)

### AC-013: pipeline_result_to_record_batch fails-closed when ocsf_field flattens to a synthesized/reserved name

`pipeline_result_to_record_batch` in `prism-bin::spec_driven_adapter` MUST return
`Err(ArrowError::SchemaError(...))` — NOT `Ok(...)` — when any sensor-table `ocsf_field`
value, after applying `ocsf_field_to_arrow_name`, equals one of the four
synthesized/reserved Arrow column names that `pipeline_result_to_record_batch` itself
generates: `"class_uid"`, `"category_uid"`, `"_sensor"`, or `"raw_extensions"`.

This guard prevents a user-declared `ocsf_field` from silently colliding with Arrow
columns synthesized by `pipeline_result_to_record_batch` itself. Example: a column with
`ocsf_field = "class.uid"` flattens to `"class_uid"` — the same name as the synthesized
`class_uid` field; allowing this would produce two Arrow columns named `"class_uid"` in
the schema, with `Schema::column_with_name` silently returning the first match and
discarding the other.

The error message MUST identify the offending reserved name and the `ocsf_field` value
that produced it, so the sensor TOML author can correct the mapping.

The four reserved names are: `"class_uid"`, `"category_uid"`, `"_sensor"`,
`"raw_extensions"`.

Covered by RG-027 (`test_pipeline_result_to_record_batch_ocsf_field_flattens_to_reserved_name_returns_error`).

(traces to BC-2.16.003 EC-016-013-029: flattened `ocsf_field` equal to a synthesized
reserved name MUST produce `Err(ArrowError::SchemaError)` — fail-closed guard, NEW in
v1.18; ADR-058 §J2 synthesized-name reservation guard)

### AC-014: extract_time_window_from_ast recognizes ocsf_field_to_arrow_name result as index-eligible push-down target (OQ-001)

When `ocsf_column_naming = true`, `prism-query::pushdown::extract_time_window_from_ast`
MUST recognize push-down filters on BOTH the raw `col.name` (e.g., `"timestamp"` for
`claroty.audit_logs.timestamp`) AND the OCSF-flattened Arrow name produced by
`ocsf_field_to_arrow_name(ocsf_field)` (e.g., `"time"` for `ocsf_field = "time"`) as
INDEX-eligible.

**Implementation:** When building `datetime_index_cols`, insert BOTH:
- the raw `col.name` (e.g., `"timestamp"`) — preserves backward compatibility;
- `ocsf_field_to_arrow_name(ocsf_field)` (e.g., `"time"`) — enables index-eligible
  push-down for filters written against the OCSF-flattened Arrow names that `prism_describe`
  advertises to the LLM agent.

**Without this fix:** A PrismQL query `WHERE time > '2024-01-01T00:00:00Z'` on a Claroty
table (where `audit_logs.timestamp` maps to Arrow `"time"` via `ocsf_field = "time"`)
would not find `"time"` in `datetime_index_cols` (which held only `"timestamp"`), causing
`extract_time_window_from_ast` to fall through to a full scan instead of using the
time-index push-down path — defeating the performance benefit of OCSF field-name routing
for the most latency-critical filter type.

**Module:** `crates/prism-query/src/pushdown.rs`; also update the stale doc comment on
`extract_time_window_from_ast` to reflect that the eligible-column set now includes
both `col.name` forms and OCSF-flattened Arrow-name forms when `ocsf_column_naming` is
in effect.

Covered by RG-PD-001 (`test_extract_time_window_from_ast_recognizes_ocsf_flattened_time_column_as_index_eligible`).

(traces to BC-2.16.003 §Interpretation A: OCSF-flattened Arrow names must be usable
verbatim by the LLM agent including in index-eligible filter positions; OQ-001 human
decision 2026-08-21)

### AC-015: prism_describe emits class_uid and _sensor synthesized ColumnDescriptors under ocsf_column_naming (OQ-003)

Under `ocsf_column_naming = true`, `prism_describe` MUST emit, appended after all Tier-1
OCSF-flattened ColumnDescriptors and after the single Tier-2 `raw_extensions` ColumnDescriptor,
exactly TWO synthesized ColumnDescriptors:

1. **`class_uid` ColumnDescriptor:**
   - `name = "class_uid"`
   - `col_type = prism_core::column::ColumnType::Integer`
   - `nullable = false`
   - `description = "OCSF event class identifier derived from sensor TOML ocsf_class. Example: 3004 for entity_management (audit_logs), 2004 for detection_finding (alerts, device_alert_relations), 5001 for inventory_info (devices)."`

2. **`_sensor` ColumnDescriptor:**
   - `name = "_sensor"`
   - `col_type = prism_core::column::ColumnType::String`
   - `nullable = false`
   - `description = "Sensor identifier. Value: <sensor_id> (e.g., 'claroty')."`

These synthesized columns are produced by `pipeline_result_to_record_batch` itself (not
declared in the TOML spec) and MUST be advertised by `prism_describe` so the LLM agent
knows they are queryable. Without these ColumnDescriptors, the agent cannot discover
`class_uid` and `_sensor` as valid filter or projection targets — they appear in the
Arrow schema but are invisible through the `prism_describe` interface.

**Wire-shape assertion requirement:** Serialize the `prism_describe` output to JSON and
assert both ColumnDescriptor entries appear with the exact `name`, `col_type`, `nullable`,
and `description` values specified above. The assertion MUST be at the wire level
(serialized JSON output), not only on pre-serialization Rust structures, per CLAUDE.md
§Conventions wire-shape assertion discipline.

**Ordering requirement:** The `class_uid` and `_sensor` synthesized ColumnDescriptors
appear LAST in the list (after all Tier-1 descriptors and the `raw_extensions` descriptor),
so that sensor-declared columns appear first as they do in the TOML spec order.

Covered by RG-028 (`test_prism_describe_ocsf_column_naming_true_emits_class_uid_and_sensor_descriptors`).

(traces to BC-2.16.003 §Interpretation A: synthesized columns produced by
`pipeline_result_to_record_batch` must be advertised by `prism_describe` so the LLM
agent can use them as filter targets; OQ-003 human decision 2026-08-21)

### AC-016: OCSF-mode column resolution — E-QUERY-038 gate operates on OCSF-flattened schema

When `ocsf_column_naming = true`, the E-QUERY-038 plan-time column-availability gate in
`prism-query::engine` MUST operate against the OCSF-flattened schema, not the raw TOML
`col.name` entries.

**Sub-case (a) — raw col.name rejected:** A query referencing a raw TOML `col.name` value
(e.g., `SELECT id FROM claroty_alerts` where `"id"` is the raw col.name and
`"finding_info_uid"` is the OCSF-flattened Arrow name) MUST fire E-QUERY-038 with:
- `column: "id"` (the raw name supplied by the model)
- `available_columns` MUST contain `"finding_info_uid"` (OCSF-flattened Tier-1 names) and
  `"raw_extensions"` (when ≥1 Tier-2 column exists), `"class_uid"`, `"_sensor"`
- `available_columns` MUST NOT contain any raw `col.name` value (e.g., `"id"`, `"category"`)
- Tier-2 col.names (e.g., `"category"` — columns with `ocsf_field == None`) MUST also be
  rejected when queried directly; their queryable representation is `"raw_extensions"`

**Sub-case (b) — OCSF-flattened name resolves:** A query referencing the OCSF-flattened
Arrow name (e.g., `SELECT finding_info_uid FROM claroty_alerts`) MUST pass E-QUERY-038
with `Ok(())` — no false positive (FP-001 preserved).

**Sub-case (c) — flag-false backward-compat:** When `ocsf_column_naming = false` (the
default for all non-Claroty sensors), the gate MUST use raw `col.name` values as before —
no change in behavior (regression prevention).

**Implementation anchor:** Fix A in `crates/prism-query/src/table_registry.rs` — when
registering a sensor table with `ocsf_column_naming = true`, the `TableRegistry` MUST
register OCSF-flattened names (`ocsf_field_to_arrow_name(ocsf_field)` for Tier-1 columns)
plus the synthesized names `"class_uid"`, `"_sensor"`, and `"raw_extensions"` (when ≥1
Tier-2 column exists). Raw `col.name` values MUST NOT be registered for OCSF-mode tables
(they are rejected at the TOML layer, not available to queries).

Covered by RG-Q-001..RG-Q-009 (RG-Q-008/009 cover the multi-tenant `resolved_spec_map` path).

(traces to BC-2.11.016 EC-11-079: E-QUERY-038 column-resolution and `available_columns`
payload MUST use the OCSF-flattened schema for `ocsf_column_naming = true` tables; raw
col.names are absent from the registered set and thus trigger E-QUERY-038 as-if-absent)

### AC-017: E-QUERY-002/041 type-compat gate resolves column types by OCSF-flattened name

When `ocsf_column_naming = true`, the E-QUERY-002/041 operator-type-compatibility gate in
`prism-query::engine` (which checks that comparison operators are used with compatible column
types, e.g., `>` only with numeric/datetime columns) MUST resolve the column type by the
OCSF-flattened name, not the raw TOML `col.name`.

**SIBLING-GATE CONSISTENCY:** The type-compat gate (Fix C in `engine.rs`) MUST be updated
in the same fix as the E-QUERY-038 column-existence gate (Fix B). Both gates read from the
same `TableRegistry` — once Fix A seeds OCSF-flattened names into the registry, both gates
naturally resolve by the updated schema. The gating logic is orthogonal — E-QUERY-038 checks
existence; E-QUERY-002/041 checks type — but they share the same lookup mechanism.

**FP-001 constraint:** SIBLING-GATE CONSISTENCY clause from BC-2.11.016 §Invariants
applies: if DERIVED provenance is detected (e.g., a stats alias, enrich output), E-QUERY-002
MUST fail-open for that name — this is unchanged from the existing gate behavior.

Covered by RG-Q-003 (WHERE predicate with OCSF-flattened column name; type-compat `=`
equality on String type resolves correctly).

(traces to BC-2.11.016 EC-11-079 sub-case (c): E-QUERY-002/041 type-compat lookup resolves
by OCSF-flattened name `finding_info_uid`; type is read from the OCSF-mapped column schema,
not the raw `col.name` entry)

### AC-018: describe/select/query name-agreement invariant for OCSF-mode tables

The queryable column-name set MUST be identical across all three surfaces for an
`ocsf_column_naming = true` table:

1. `prism_describe` response (per AC-015 / BC-2.16.003 §Interpretation A)
2. `SELECT *` Arrow schema (per AC-003 / ADR-058 §I1)
3. E-QUERY-038 `available_columns` payload (per AC-016 / BC-2.11.016 EC-11-079)

**Invariant:** A column name that succeeds as a `SELECT <col>` projection target MUST also
appear in `prism_describe` output and MUST NOT trigger E-QUERY-038. Conversely, a column
name that triggers E-QUERY-038 MUST NOT appear in `prism_describe` output. Any surface
misalignment is a gate/surface mismatch and violates this invariant.

**Observed-behavior basis (ADR-058 §G):** Prior to S-ADR058-OCSF-ROUTING-001, the OCSF
renaming was applied at `prism_describe` and `pipeline_result_to_record_batch` (Arrow
schema) only. The E-QUERY-038 plan-time gate still checked raw `TableRegistry` col.names,
causing `SELECT *` (OCSF-flattened Arrow schema) to succeed while
`SELECT <ocsf_flattened_name>` and `WHERE <ocsf_flattened_name>` failed E-QUERY-038 —
violating this invariant. AC-016 (Fix A — TableRegistry) and AC-017 (Fix C — type-compat)
close this gap.

Covered by RG-Q-001..RG-Q-009 (all nine tests together verify this invariant: projection
OK, WHERE OK, type-compat OK, raw rejected, available_columns wire-shape, flag-false intact;
RG-Q-008/009 extend coverage to the multi-tenant `resolved_spec_map` code path).

(traces to BC-2.11.016 EC-11-079 sub-case (d): describe/select/query name-agreement
invariant — the `available` set used by the E-QUERY-038 gate for `ocsf_column_naming = true`
tables MUST be identical to the column-name set reported by `prism_describe` and materialized
in the `SELECT *` Arrow schema)

---

### AC-019: Zero-column OCSF table registers class_uid and _sensor in plan-gate available set; A+W sub-case adds raw_extensions and spec-load warning

When `ocsf_column_naming = true` and a sensor table has zero Tier-1 columns (no column has
an `ocsf_field == Some(...)` declaration), `TableRegistry` MUST register `class_uid` (Integer)
and `_sensor` (String) as available columns. The E-QUERY-038 plan-gate MUST accept `SELECT class_uid`
and `SELECT _sensor` against such a table and return `Ok(())`.

The outer `if !table.columns.is_empty()` guard that currently gates the entire OCSF branch in
`register_sensor` (`crates/prism-query/src/table_registry.rs`) MUST be removed from the OCSF
branch only. When `spec.ocsf_column_naming == true`, the shared helpers
`ocsf_projected_column_names` / `ocsf_projected_column_types` from
`prism_spec_engine::column_mapping` MUST always be called regardless of Tier-1 column count.
The `!columns.is_empty()` guard MUST be kept on the non-OCSF `else` branch (preserve
fail-open there).

No raw `col.name` value MUST appear in the available set for an OCSF-mode table with zero
Tier-1 columns — raw col.name values are always absent from OCSF-mode tables.

**Sub-case: zero Tier-1, zero Tier-2 (no columns at all, or all columns somehow non-Tier-2):**
The available set for such a table MUST be exactly `["_sensor", "class_uid"]`.

**Sub-case A+W: zero Tier-1, ≥1 Tier-2 (at least one column has `ocsf_field == None`):**
When a table has zero Tier-1 columns AND at least one Tier-2 column, `register_sensor` MUST
additionally:
1. Register `"raw_extensions"` (Json) in the plan-gate available set — the available set for
   such a table MUST be exactly `["_sensor", "class_uid", "raw_extensions"]`; Tier-2 data is
   preserved via `raw_extensions` and MUST NOT be dropped.
2. Emit a `tracing::warn!` event with `event_type = "ocsf.zero_tier1_table"`, fields
   `sensor_id: %display` and `table_name: %display`, ONCE at spec-load/registration for that
   table (not per-query). This is the Option A+Warning decision (human decision 2026-08-23;
   ADR-058 §J6). SAP-1/PG-LP11-001 obligation: the implementer MUST add this emission in T-31
   in the same commit as the `raw_extensions` registration for zero-Tier-1-with-Tier-2 tables.

**`TableRegistry` MUST register class_uid + _sensor for zero-Tier-1-column OCSF tables**
(RG-Q-010 is the anchor test that class_uid + _sensor queries are accepted;
RG-Q-011 is the anchor test that raw col.name is rejected and available set is exact;
RG-Q-017 is the anchor test for the A+W sub-case: `raw_extensions` in available set +
`ocsf.zero_tier1_table` WARN emitted once at registration).

Covered by RG-Q-010 (`test_BC_2_11_016_zero_col_ocsf_table_st_gate_accepts_class_uid_and_sensor`),
RG-Q-011 (`test_BC_2_11_016_zero_col_ocsf_table_st_gate_rejects_raw_col_name`), and
RG-Q-017 (`test_BC_2_11_016_zero_tier1_with_tier2_projects_raw_extensions_and_emits_warning`).
RG-Q-010 and RG-Q-011 MUST be failing (RED) before LOW-1-FIX is implemented (T-28).
RG-Q-017 MUST be failing (RED) before A+W fix is implemented (T-31).

(traces to BC-2.11.016 EC-11-080 postconditions 1, 2, and A+W sub-case: zero-column OCSF table
presents class_uid + _sensor in plan-gate; zero-Tier-1-with-Tier-2 additionally presents
raw_extensions + emits ocsf.zero_tier1_table WARN; ADR-058 §J6; BC-2.16.002 v2.34
`ocsf.zero_tier1_table` catalog row governs the warning emission)

---

### AC-020: Consolidated-projection invariant — ocsf_projected_column_names is the single authoritative impl

`ocsf_projected_column_names(tbl: &TableSpec, ocsf_column_naming: bool) -> Vec<String>` and
`ocsf_projected_column_types(tbl: &TableSpec, ocsf_column_naming: bool) -> HashMap<String, ColumnType>`
MUST be added to `prism-spec-engine::column_mapping` as the single authoritative projection
implementations. Their semantics:

- When `ocsf_column_naming = true`: Tier-1 `ocsf_field_to_arrow_name(ocsf_field)` names +
  `"class_uid"` + `"_sensor"` + `"raw_extensions"` iff any Tier-2 column exists
  (`ocsf_field == None`). Zero-Tier-1-column table → `["class_uid", "_sensor"]`.
- When `ocsf_column_naming = false`: raw `col.name` list.

The private helper `ocsf_or_raw_column_names_for_table` in `crates/prism-query/src/engine.rs`
MUST become a thin forward that delegates entirely to
`prism_spec_engine::column_mapping::ocsf_projected_column_names`. The `register_sensor`
function in `crates/prism-query/src/table_registry.rs` MUST use these helpers instead of
inline logic when seeding `columns_by_table` and `column_types_by_table`.

`build_ocsf_column_descriptors` (prism-mcp) and `pipeline_result_to_record_batch` (prism-bin)
remain inline as documented shape-exception sites (they need Arrow types / descriptor text
that the shared helpers do not provide). They carry a doc-comment referencing the
consolidated-projection invariant. RG-Q-015 enforces that the registry name-set and the shared
helper output are byte-equal (sorted) for any OCSF-mode table.

**The shared helpers MUST produce identical name sets to the registry and engine column-resolution
surfaces** (RG-Q-015 `test_ocsf_projected_names_all_surfaces_agree` is the anchor test for
this invariant; it MUST fail before OBS-1-FIX is implemented — T-29).

Covered by RG-Q-015 (`test_ocsf_projected_names_all_surfaces_agree`).

(traces to ADR-058 §I7: Consolidated-Projection Invariant — `ocsf_projected_column_names` /
`ocsf_projected_column_types` in prism-spec-engine are the single authoritative projection
impl; `ocsf_or_raw_column_names_for_table` (prism-query engine) becomes a thin forward;
`build_ocsf_column_descriptors` (prism-mcp) and `pipeline_result_to_record_batch` (prism-bin)
are documented shape-exception sites bound by RG-Q-015)

---

### AC-021: Spec-load collision validation — parse_and_validate_spec_toml rejects §J1/§J2/§J4 collisions

`parse_and_validate_spec_toml` MUST reject OCSF column collisions at spec-load time via a new
Validation Rule 8 implemented as `validate_ocsf_column_collisions(spec: &SensorSpec) -> Vec<String>`
in `crates/prism-spec-engine/src/add_sensor_spec.rs`. The function enforces:

1. **[§J2]**: Any Tier-1 column whose `ocsf_field_to_arrow_name(ocsf_field)` equals a reserved
   synthesized name (`class_uid`, `category_uid`, `_sensor`, or `raw_extensions`) produces an
   error string containing `E-SPEC-030` and `[§J2]` plus the sensor ID and table name.
2. **[§J4]**: Two Tier-1 columns in the same table that flatten to the same arrow name (intra-table
   duplicate) produce an error string containing `E-SPEC-030` and `[§J4]`.
3. **[§J1]**: A Tier-1 arrow name that equals another column's raw `col.name` within the same table
   (shadow collision — the `A ≠ B` self-match exclusion from T-21 still applies to the runtime
   check; this spec-load check mirrors that logic) produces an error string containing `E-SPEC-030`
   and `[§J1]`.

Validation Rule 8 is wired into `parse_and_validate_spec_toml` after Rule 7. The `ValidationError`
type uses `Vec<String>` — no new enum variant needed. Boot path: `ConfigInvalid → exit 2`
(existing behavior). Hot-reload: keeps prior spec on error (existing behavior).

The runtime `pipeline_result_to_record_batch` §J guard (checks (a)+(b)+(c) in T-21) MUST remain
as defense-in-depth — it is now unreachable in production (valid specs cannot have §J collisions
post-Validation-Rule-8) but correct to keep for belt-and-suspenders.

Note: E-SPEC-030 is the correct error code for OCSF column collision at spec-load time (not
E-SPEC-027, which is assigned to header_scheme validation). ALWAYS use E-SPEC-030 for §J
collision detection in this context.

**`parse_and_validate_spec_toml` MUST reject §J1/§J2/§J4 collisions with E-SPEC-030** at
spec-load time (RG-Q-012, RG-Q-013, RG-Q-014 are the anchor tests — each MUST fail before
OBS-2-FIX is implemented; each asserts E-SPEC-030 and the specific collision tag).

Covered by RG-Q-012 (`test_BC_2_16_003_ocsf_collision_j2_reserved_name_rejected_at_spec_load`),
RG-Q-013 (`test_BC_2_16_003_ocsf_collision_j4_intra_table_duplicate_rejected_at_spec_load`),
and RG-Q-014 (`test_BC_2_16_003_ocsf_collision_j1_shadow_rejected_at_spec_load`). All three
MUST return `Err` containing `"E-SPEC-030"` and their respective tag (`[§J2]`, `[§J4]`, `[§J1]`).

(traces to BC-2.16.003 EC-016-013-032 postcondition 1: `parse_and_validate_spec_toml` rejects
§J1/§J2/§J4 collisions with E-SPEC-030; boot ConfigInvalid → exit 2; hot-reload keeps prior
spec; ADR-058 §J7)

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Scope |
|-----------|--------|---------------|-------|
| `SensorSpec::ocsf_column_naming` field | `prism-spec-engine::spec_parser` | Pure (data struct) | New field added |
| `ocsf_field_to_arrow_name` | `prism-spec-engine::column_mapping` | Pure | New free function — no I/O, deterministic string transform; canonical home per ADR-058 §I1; imported by both `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe` |
| `pipeline_result_to_record_batch` | `prism-bin::spec_driven_adapter` | Effectful (Arrow I/O) | New parameter `sensor_spec: &SensorSpec` threaded from `fetch()` (ADR-058 §D1 / ADR-022 §C wiring); conditional branch on `sensor_spec.ocsf_column_naming` |
| `pipeline_result_to_record_batch` `raw_extensions` aggregation (§I2) | `prism-bin::spec_driven_adapter` | Effectful (Arrow I/O) | New path (ADR-058 §I2): when `ocsf_column_naming = true`, columns with `ocsf_field == None` are suppressed from individual Arrow schema fields and aggregated into a single `"raw_extensions"` Utf8 column; synthesis locus is `pipeline_result_to_record_batch` (schema-fields construction); aggregation loop MUST apply the same source_path extraction + ENRICH-1 `Value::Array`→compact-JSON-list-string normalization as first-class columns (BC-2.16.003 EC-016-013-028 reworded) — NOT naive `r.get(col.name)` |
| `prism_describe` | `prism-mcp::tools::prism_describe` | Effectful (MCP response) | Tier-1/Tier-2 model per ADR-058 §G: Tier-1 (`ocsf_field == Some`) → ColumnDescriptor with `name = ocsf_field_to_arrow_name(ocsf_field)` and `description = ocsf_field`; Tier-2 (`ocsf_field == None`) → NO individual ColumnDescriptor emitted; exactly ONE `raw_extensions` ColumnDescriptor emitted per table enumerating all `ocsf_field == None` source keys (col.names) |
| `claroty.sensor.toml` | `prism-sensors/specs/` | Configuration | Add `ocsf_column_naming = true` + all KF-01..KF-12 corrections + §J3 shadow fix (14 TOML changes per AC-005) |
| `class_selector.rs` | `prism-ocsf/src/` | Pure (lookup table) | Add `CLASS_UID_ENTITY_MANAGEMENT = 3004`; reroute `"audit_activity"` arm + Armis `("armis","audit_log")` arm to entity_management (3004) per AC-009 |
| `#[ignore]`'d e2e test | `crates/prism-bin/tests/` | Test (effectful) | Update `row.get("uid")` → `row.get("device_uid")` |
| `extract_time_window_from_ast` | `prism-query::pushdown` | Pure (predicate analysis) | Modify: dual-name `datetime_index_cols` insert — add `ocsf_field_to_arrow_name(ocsf_field)` result alongside `col.name`; update stale doc comment (OQ-001/AC-014/RG-PD-001) |

Architecture section files: `architecture/module-decomposition.md` (SS-01, SS-02, SS-10, SS-16),
`architecture/dependency-graph.md`.

---

## Purity Classification

| Component | Classification | Rationale |
|-----------|---------------|-----------|
| `ocsf_field_to_arrow_name` | Pure | `&str` → `String`; replaces `.` with `_`; deterministic, no I/O |
| `SensorSpec` deserialization | Pure (serde) | Derives `Deserialize`; `#[serde(default)]` is a declarative attribute |
| `pipeline_result_to_record_batch` | Effectful | Calls `RecordBatch::try_new` (Arrow schema validation error path); writes to caller's batch output; includes §I2 raw_extensions aggregation (schema-construction logic — not a separate pure function) |
| `prism_describe` | Effectful | Reads `SensorSpec` from registry, constructs MCP response |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Column with `ocsf_field = Some("status")` (single segment, no dot) | `ocsf_field_to_arrow_name("status")` = `"status"` — unchanged; not a regression |
| EC-002 | Column with `ocsf_field = None` when `ocsf_column_naming = true` | Value goes to `raw_extensions` JSON blob; no individual Arrow field for this column |
| EC-003 | Claroty `audit_logs` table `username` with `ocsf_field = "actor.user.uid"` | Arrow field name = `"actor_user_uid"` per BC-2.16.003 §Claroty Contracted OCSF Mappings; `column_type = "string"` prevents Rule 2 `uid` numeric-suffix coercion (EC-016-013-005) |
| EC-004 | CrowdStrike sensor (no `ocsf_column_naming` flag) queries `col.name` | `ocsf_column_naming` defaults to `false`; Arrow names stay as col.name; no change |
| EC-005 | Two sensors both declare `ocsf_field = "device.ip"` and both enable the flag | Both produce Arrow field `"device_ip"`; cross-sensor JOIN on `device_ip` works per BC-2.16.003 EC-016-013-012 |
| EC-006 | `ocsf_field = "finding.uid"` on a column already named `finding_uid` in `col.name` | `ocsf_field_to_arrow_name("finding.uid")` = `"finding_uid"` = `col.name` — no conflict, degenerate case |
| EC-007 | `raw_extensions` column name conflicts with an existing sensor column named `raw_extensions` | If a sensor column is explicitly named `raw_extensions` and has `ocsf_field != None`, it gets its flattened ocsf_field name; the `raw_extensions` blob column collects only columns with `ocsf_field = None`. No collision on Claroty (no Claroty column is named `raw_extensions`). |
| EC-008 | `just check` after Stage 2 (blast radius per ADR-058 §E1) | Must stay GREEN: spec-driven-adapter unit tests use inline ColumnSpec with `ocsf_field = None` so they get `col.name`; DTU parity tests assert DTU HTTP response JSON (not Arrow schema); `prism_describe` unit tests use inline ColumnDescriptor constructions; CrowdStrike/Armis/Cyberint unaffected; Claroty e2e test is `#[ignore]`'d |
| EC-009 | Two columns in the same table have `ocsf_field` values that flatten to the same Arrow name (e.g., `"a.b_c"` and `"a_b.c"` both → `"a_b_c"` via `ocsf_field_to_arrow_name`) | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed. Arrow 58 does NOT detect duplicate schema field names (`Schema::new` is infallible; `Schema::column_with_name` returns the first match — silent wrong-column resolution for the agent). Current Claroty TOML has no intra-table collision (verified by enumeration: 31 ocsf_field values pre-corrections across four tables — alerts: 9, audit_logs: 8, devices: 8, device_alert_relations: 6; post-KF corrections: 27 — alerts: 6, audit_logs: 7, devices: 8, dar: 6; ADR-058 §J4; OQ-005 adds audit_logs.id → metadata_uid). Future sensors must be collision-free before enabling the flag. See RG-009. |
| EC-010 | A flattened `ocsf_field` name from one column equals the `col.name` of a DIFFERENT column in the same table when `ocsf_column_naming = true` (flag-transition name shadowing per ADR-058 §J1/§J2). Example: `device_category` with `ocsf_field = "device.type"` → `device_type`, while column `device_type` has `col.name = "device_type"`. `SELECT device_type FROM claroty_devices` is valid in both flag states but returns different semantic content — high-level category vs type-within-category — with no error and no warning. | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed. The `A ≠ B` self-match exclusion is mandatory: a column whose flattened ocsf_field name equals its own `col.name` (e.g., `risk_score` → `risk_score`) is legal and MUST NOT fail. This collision class is resolved in `claroty.sensor.toml` by changing `device_category`'s ocsf_field from `"device.type"` to `"device.type_category"` (AC-005, same TOML edit). See RG-010. |
| EC-016-013-027 | `prism_describe` emits individual ColumnDescriptors for `ocsf_field == None` columns (phantom queryable names) when `ocsf_column_naming = true`, or emits no `raw_extensions` ColumnDescriptor, or emits a `raw_extensions` ColumnDescriptor that fails to enumerate source keys. Pre-fix: agent calls `prism_describe` for a Claroty table, sees `category` as a ColumnDescriptor name, writes `SELECT category FROM claroty_alerts`, gets no data because `category` is aggregated into `raw_extensions` under the OCSF routing model — silent semantic failure at the agent/query interface. | `prism_describe` MUST NOT emit any ColumnDescriptor with `name` equal to an `ocsf_field == None` column's `col.name`; MUST emit exactly ONE ColumnDescriptor with `name = "raw_extensions"` whose description enumerates all `ocsf_field == None` col.name values as source keys. Tested by RG-025 (five assertions: phantom prohibition (i), count exactly 1 (ii), source-key enumeration (iii), col_type=Json (iv), nullable=true (v)). Traces to AC-006 Tier-2, AC-007b, ADR-058 §G, BC-2.16.003 §Interpretation A. |
| EC-016-013-028 | A Claroty `devices` row has a non-empty `ip_list` field (multi-valued array source field). The field has `ocsf_field == None` and routes to `raw_extensions`. Pre-fix risk: `pipeline_result_to_record_batch` could serialize the array value as a nested JSON array inside `raw_extensions` (e.g., `{"ip_list": ["192.168.1.1", "10.0.0.1"]}`), or silently drop the value entirely, or emit it as null. Either scenario produces unexpected query behavior: the LLM agent expecting a compact JSON-list string receives a nested array or null instead, and cannot reliably parse or filter on it. | `pipeline_result_to_record_batch` MUST serialize the `ip_list` array value as a compact JSON-list STRING inside `raw_extensions` — e.g., `{"ip_list": "[\"192.168.1.1\",\"10.0.0.1\"]"}` where the `ip_list` value is the string `"[\"192.168.1.1\",\"10.0.0.1\"]"`, NOT a nested array, NOT null. This assertion MUST be verified at the serialized wire-output level (not pre-serialization Rust struct). Tested by RG-026. Traces to AC-007c, ADR-058 §B2 / §I2 (ip_list has `ocsf_field == None` routes to raw_extensions), BC-2.16.003 EC-016-013-028. |

---

## Token Budget Estimate

| Source | Estimated tokens |
|--------|-----------------|
| This story spec | ~6k |
| `spec_parser.rs` (SensorSpec struct) | ~4k |
| `spec_driven_adapter.rs` (pipeline_result_to_record_batch + build_column_array) | ~12k |
| `prism_describe.rs` (ColumnDescriptor construction) | ~3k |
| `claroty.sensor.toml` (full TOML spec for context) | ~4k |
| `class_selector.rs` (prism-ocsf — class name → class_uid lookup) | ~2k |
| BC-2.16.003 + BC-2.16.002 + BC-2.01.013 + BC-2.11.016 (governing contracts — 4 BCs) | ~8k |
| ADR-058 (§B2, §D, §G, §I, §J, §K in full) | ~7k |
| `bc_2_16_003_test.rs` + existing spec_driven_adapter tests (context for new tests) | ~4k |
| Tool outputs (just iter, cargo nextest) | ~1k |
| **Total** | **~50k** |

50k tokens is within a 200k agent context window (~25%). This story does NOT need
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
- T-06: Write RG-003 — `test_ocsf_field_to_arrow_name_replaces_dots_with_underscores` (MUST FAIL).
  Place in `crates/prism-spec-engine/src/column_mapping.rs` `#[cfg(test)] mod tests` (ADR-058 §I1
  canonical home; NOT in prism-bin).
- T-07: Write RG-004 — `test_ocsf_field_to_arrow_name_single_segment_is_unchanged` (MUST FAIL).
  Place in `crates/prism-spec-engine/src/column_mapping.rs` `#[cfg(test)] mod tests` (same
  canonical home as T-06/RG-003).
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
- T-11D: Write RG-011 — `test_class_selector_entity_management_and_inventory_info_arms`
  (MUST FAIL). In `prism-ocsf`, assert BOTH:
  (1) `select_by_class_name("entity_management")` == `Ok(3004)` — the live KF-01 string;
  (2) `select_by_class_name("inventory_info")` == `Ok(5001)` — the live KF-02 string.
  Do NOT assert `select_by_class_name("audit_activity")` — dead string post-KF-01 TOML fix.
  Currently fails because neither arm exists. Covers AC-009 sub-obligations (a) + (b).
- T-11E: Write RG-012 — `test_class_selector_armis_audit_log_maps_to_entity_management_3004`
  (MUST FAIL). In `prism-ocsf`, call `select("armis", "audit_log")` (or equivalent Armis
  dispatch) and assert result equals `Ok(3004)`. Currently fails. Covers AC-009
  (TD-VSDD-097 dim-1 sibling).
- T-11F: Write RG-013 — `test_claroty_note_comment_not_silently_dropped_under_entity_management`
  (MUST FAIL). Build a `DynamicMessage` for `entity_management` (class_uid 3004). Call
  `set_nested_field` with path `"comment"` and value `"reviewed"`. Assert the field is
  set (not silently dropped). Contrast with a `account_change` (3001) message where the
  same call no-ops (verifying the data-loss condition). Currently fails until
  CLASS_UID_ENTITY_MANAGEMENT = 3004 is added and the arm updated. Covers AC-009
  (data-loss prevention, EC-016-013-023).
- T-11G: Write RG-014 — `test_claroty_alerts_reserved_fields_go_to_raw_extensions_not_first_class_columns`
  (MUST FAIL). Wire-shape assertion: load the corrected `claroty.sensor.toml` alerts
  table spec (post-T-17, KF-08/09/10: `category`, `alert_type_name`, `devices_count`
  have no `ocsf_field`). Pass a record `{"category": "OT Security", "alert_type_name":
  "Protocol Violation", "devices_count": 3}` through `pipeline_result_to_record_batch`
  with `ocsf_column_naming = true`. Serialize the RecordBatch to JSON. Assert: no
  first-class Arrow fields named `"class_name"`, `"type_name"`, or `"count"` exist;
  the `raw_extensions` JSON blob contains keys `"category"`, `"alert_type_name"`,
  `"devices_count"` with vendor values. Currently fails because KF-08/09/10 corrections
  are not yet in the TOML. Covers AC-010 (EC-016-013-013/014/015).
- T-11H: Write RG-015 — `test_claroty_alerts_finding_info_fields_wire_shape`
  (MUST FAIL). Wire-shape assertion: load the corrected `claroty.sensor.toml` alerts
  table spec (post-T-17, KF-03/04/12: `id.ocsf_field = "finding_info.uid"`,
  `alert_name.ocsf_field = "finding_info.title"`, `updated_time.ocsf_field =
  "finding_info.modified_time"`). Pass a record `{"id": "132", "alert_name": "Modbus Violation",
  "updated_time": "2024-01-15T10:30:00Z"}` through `pipeline_result_to_record_batch`
  with `ocsf_column_naming = true`. Serialize to JSON. Assert: (1) Arrow field
  `"finding_info_uid"` contains `"132"` (KF-03); (2) Arrow field `"finding_info_title"`
  contains `"Modbus Violation"` (KF-04); (3) Arrow field `"finding_info_modified_time"`
  contains `"2024-01-15T10:30:00Z"` (KF-12); (4) no Arrow field named `"finding_uid"`,
  `"finding_title"`, or `"end_time"` exists. Currently fails because KF-03/04/12
  corrections are not yet in the TOML. Covers AC-010 (KF-03/04/12).
- T-11I: Write RG-016 — `test_claroty_audit_logs_record_batch_class_uid_is_3004`
  (MUST FAIL). Wire-shape integration test: build a minimal Claroty `audit_logs` `SensorSpec`
  (or use the production TOML table spec with `ocsf_class = "entity_management"` after KF-01
  TOML correction). Call `pipeline_result_to_record_batch` with one record. Inspect the
  returned `RecordBatch`: find the `class_uid` column (Arrow Int32). Assert value == `3004`.
  Assert NOT `3001` and NOT `0`. Currently fails because `"entity_management"` arm does not
  exist in `select_by_class_name` — falls to `.unwrap_or(0)`. Covers AC-009 sub-obligation
  (b) integration path; traces to BC-2.16.003 EC-016-013-023.
- T-11J: Write RG-017 — `test_claroty_devices_record_batch_class_uid_is_5001_regression_guard`
  (MUST FAIL). Wire-shape regression-prevention test: build a minimal Claroty `devices`
  `SensorSpec` with `ocsf_class = "inventory_info"` (KF-02 corrected value). Call
  `pipeline_result_to_record_batch` with one record. Assert Arrow `class_uid` Int32 == `5001`.
  Assert NOT `0`. Without the `"inventory_info"` arm, the KF-02 TOML change regresses
  class_uid from 5001 (current `"device"` arm) to 0 silently. Covers AC-009 sub-obligation
  (b) regression guard; traces to BC-2.16.003 EC-016-013-024.
- T-11K: Write RG-018 — `test_pipeline_result_to_record_batch_unknown_ocsf_class_emits_warn`
  (MUST FAIL). Build a `SensorSpec` with `ocsf_class = "completely_unknown_class"`. Use
  `tracing_test` subscriber. Call `pipeline_result_to_record_batch`. Assert: (1) a WARN
  event with `event_type = "ocsf.unknown_class_name"` was captured; (2) the event's
  `ocsf_class` field matches `"completely_unknown_class"`; (3) function returns `Ok(...)`
  (graceful fallback preserved). Currently fails because the warn emission does not exist.
  Covers AC-011; traces to BC-2.16.002 §Canonical Structured Event Catalog `ocsf.unknown_class_name`.
- T-11L: Write RG-019 — `test_claroty_audit_logs_record_batch_kf11_category_in_raw_extensions`
  (MUST FAIL). Wire-shape assertion: load the corrected `claroty.sensor.toml` audit_logs
  table spec (post-T-17, KF-11: `category` has no `ocsf_field`; KF-01: `ocsf_class =
  "entity_management"`). Pass a record `{"category": "Authentication", "action": "Login", "note": "reviewed"}`
  through `pipeline_result_to_record_batch` with `ocsf_column_naming = true`. Serialize
  the RecordBatch to JSON. Assert: (1) no first-class Arrow field `"category_uid"` or
  `"category_name"` exists; (2) `raw_extensions` blob contains `"category": "Authentication"`;
  (3) Arrow field `"activity_name"` contains `"Login"`; (4) Arrow field `"comment"` contains
  `"reviewed"`. Currently fails because KF-11 TOML correction not yet applied. Covers
  AC-010 (KF-11) and AC-009 entity_management field mappings at integration level.
- T-11M: Write RG-020 — `test_claroty_device_alert_relations_record_batch_finding_info_uid_wire_shape`
  (MUST FAIL). Wire-shape assertion: load the corrected `claroty.sensor.toml`
  device_alert_relations table spec (post-T-17, KF-07: `alert_id.ocsf_field =
  "finding_info.uid"`). Pass a record
  `{"device_uid": "dev-001", "alert_id": "alert-123"}` through `pipeline_result_to_record_batch`
  with `ocsf_column_naming = true`. Serialize to JSON. Assert: (1) Arrow field
  `"finding_info_uid"` contains `"alert-123"`; (2) no field named `"finding_uid"` exists;
  (3) no field named `"finding.uid"` exists. Currently fails because KF-07 TOML correction
  not yet applied. Covers AC-010 (KF-07).
- T-11N: Write RG-021 — `test_claroty_audit_logs_id_produces_metadata_uid_top_level_arrow_field`
  (MUST FAIL). Wire-shape assertion: load the corrected `claroty.sensor.toml` audit_logs
  table spec (post-T-17, OQ-005: `id.ocsf_field = "metadata.uid"`). Pass a record `{"id": "al-999",
  "action": "Login"}` through `pipeline_result_to_record_batch` with
  `ocsf_column_naming = true` and `ocsf_class = "entity_management"`. Serialize to JSON.
  Assert: (1) Arrow field `"metadata_uid"` (Tier-1 String column) contains `"al-999"`;
  (2) no `"id"` key exists in the `raw_extensions` JSON blob;
  (3) Arrow field `"activity_name"` contains `"Login"`.
  Wire-shape assertion on the serialized Arrow column name.
  Currently fails because OQ-005 TOML correction not yet applied.
  Covers AC-010 assertion 5 (OQ-005).
- T-11O: Write RG-022 — `test_claroty_devices_device_type_produces_device_type_label_arrow_field`
  (MUST FAIL). Wire-shape assertion: load the corrected `claroty.sensor.toml` devices
  table spec (post-T-17, KF-06: `device_type.ocsf_field = "device.type_label"`). Pass a record
  `{"device_uid": "dev-001", "device_type": "PLC"}` through `pipeline_result_to_record_batch`
  with `ocsf_column_naming = true` and `ocsf_class = "inventory_info"`. Serialize to JSON.
  Assert: (1) Arrow field `"device_type_label"` contains `"PLC"`; (2) no Arrow field named
  `"device_type_name"` exists. Demo-critical: without this field `WHERE device_type_label =
  'PLC'` fails. Currently fails because KF-06 TOML correction not yet applied. Covers
  AC-010 (KF-06).
- T-11P: Write RG-023 — `test_class_selector_claroty_audit_log_select_arm_maps_to_entity_management_3004`
  (MUST FAIL). Integration test in `crates/prism-ocsf/tests/`: call
  `select("claroty", "audit_log")` and assert `Ok(3004)` (entity_management). Currently
  fails because the `("claroty", "audit_log")` arm has not yet been updated. Covers
  AC-009(c) Claroty arm.
- T-11Q: Write RG-024 — `test_pipeline_result_to_record_batch_sensor_spec_parameter_gates_both_branches`
  (MUST FAIL at compile time — E0061). In `crates/prism-bin/src/spec_driven_adapter.rs`
  `#[cfg(test)] mod tests`: construct a `SensorSpec` with `ocsf_column_naming = true` and
  a column with `name = "id"` and `ocsf_field = Some("finding_info.uid")`. Call
  `pipeline_result_to_record_batch` passing this `sensor_spec` and assert the Arrow schema
  field is named `"finding_info_uid"`. Then construct a SECOND call with the same column
  data but `ocsf_column_naming = false` and assert the field is named `"id"`. The test
  currently fails to compile (E0061: wrong number of arguments) because
  `pipeline_result_to_record_batch` does not yet accept `sensor_spec`. Covers AC-012.
- T-11R: Write RG-025 — `test_prism_describe_ocsf_column_naming_true_raw_extensions_descriptor_and_no_phantom_col_names`
  (MUST FAIL). In `crates/prism-mcp/tests/` (or the appropriate prism-mcp test file for
  `prism_describe`): construct a `SensorSpec` with `ocsf_column_naming = true` and a
  mixed-column table — column A (`name = "id"`, `ocsf_field = Some("finding_info.uid")`),
  column B (`name = "category"`, `ocsf_field = None`), column C (`name = "alert_type_name"`,
  `ocsf_field = None`). Call `prism_describe` and assert ALL FIVE of: (i) NO ColumnDescriptor
  has `name` equal to `"category"` or `"alert_type_name"`; (ii) exactly ONE ColumnDescriptor
  has `name = "raw_extensions"`; (iii) the `raw_extensions` ColumnDescriptor's `description`
  contains `"category"` AND `"alert_type_name"` as source key enumerations; (iv) the
  `raw_extensions` ColumnDescriptor has `col_type = prism_core::column::ColumnType::Json`;
  (v) the `raw_extensions` ColumnDescriptor has `nullable = true`. RED condition:
  pre-fix `prism_describe` emits phantom ColumnDescriptors for `"category"` and
  `"alert_type_name"` (assertions (i)-(v) all fail) and no `raw_extensions` ColumnDescriptor
  with the correct four-field shape. Covers AC-006 Tier-2 and AC-007b; traces to ADR-058 §G /
  BC-2.16.003 §Interpretation A EC-016-013-027.
- T-11S: Write RG-026 — `test_claroty_devices_ip_list_in_raw_extensions_is_compact_json_list_string`
  (MUST FAIL). In `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`:
  construct a Claroty `devices` `SensorSpec` with `ocsf_column_naming = true` and a table
  column `ip_list` with `source_path = "$.ip_list[*]"` and `ocsf_field = None`. Pass a
  synthetic pipeline result record with `ip_list = ["192.168.1.1", "10.0.0.1"]` through
  `pipeline_result_to_record_batch` and serialize the RecordBatch to JSON. Assert ALL THREE:
  (i) a `raw_extensions` field exists in the JSON output; (ii) the `raw_extensions` JSON
  object contains an `"ip_list"` key; (iii) the `"ip_list"` value is the compact JSON-list
  STRING `"[\"192.168.1.1\",\"10.0.0.1\"]"` — NOT a nested JSON array, NOT null. Assertions
  MUST be made at wire-level (serialized JSON bytes). RED condition: without the fix,
  `ip_list` may be serialized as a nested array (assertion (iii) fails) or dropped/null.
  Covers AC-007c. Traces to BC-2.16.003 EC-016-013-028 + ADR-058 §B2/§I2.
- T-11T: Write RG-027 — `test_pipeline_result_to_record_batch_ocsf_field_flattens_to_reserved_name_returns_error`
  (MUST FAIL). In `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`:
  for each of the four reserved names (`class_uid`, `category_uid`, `_sensor`, `raw_extensions`),
  construct a `SensorSpec` with `ocsf_column_naming = true` and a table column whose
  `ocsf_field` flattens to that reserved name (e.g., `ocsf_field = "class.uid"` → `class_uid`;
  `ocsf_field = "category.uid"` → `category_uid`; `ocsf_field = "_sensor"` → `_sensor`;
  `ocsf_field = "raw.extensions"` → `raw_extensions`). Call `pipeline_result_to_record_batch`
  and assert `Err(ArrowError::SchemaError(...))` for each sub-case. RED condition: without
  the §J2 guard, the function returns `Ok(...)` with a malformed schema — assertion fails.
  Covers ADR-058 §J2 reserved-name guard.
- T-11U: Write RG-PD-001 — `test_extract_time_window_from_ast_recognizes_ocsf_flattened_time_column_as_index_eligible`
  (MUST FAIL). In `crates/prism-query/src/pushdown.rs` `#[cfg(test)] mod tests`: construct a
  filter expression on `time` (the OCSF-flattened Arrow name for `claroty.audit_logs.timestamp`
  via `ocsf_field = "time"`). Build a `datetime_index_cols` set containing only `"timestamp"`
  (the raw `col.name`) — BEFORE the fix. Pass the filter to `extract_time_window_from_ast`
  and assert: (1) the function returns an INDEX-eligible time window (not `None` / full-scan
  fallback); (2) the window is derived from the `time` predicate. RED condition: `"time"` is
  not in `datetime_index_cols` (which holds `"timestamp"` only), so the function returns
  `None` (full-scan). Assertion (1) fails. Covers AC-014.
- T-11V: Write RG-028 — `test_prism_describe_ocsf_column_naming_true_emits_class_uid_and_sensor_descriptors`
  (MUST FAIL). In `crates/prism-mcp/tests/` alongside RG-025: construct a `SensorSpec` with
  `ocsf_column_naming = true` and a table with at least one Tier-1 column and one Tier-2 column.
  Call `prism_describe` and assert ALL SIX of: (i) a ColumnDescriptor with `name = "class_uid"`,
  `col_type = Integer`, `nullable = false` exists; (ii) a ColumnDescriptor with
  `name = "_sensor"`, `col_type = String`, `nullable = false` exists; (iii) `class_uid`
  appears AFTER all Tier-1 descriptors and AFTER the `raw_extensions` descriptor;
  (iv) `_sensor` appears alongside `class_uid` (last two descriptors);
  (v) `class_uid` has `description = "OCSF event class identifier derived from sensor TOML
  ocsf_class. Example: 3004 for entity_management (audit_logs), 2004 for detection_finding
  (alerts, device_alert_relations), 5001 for inventory_info (devices)."`;
  (vi) `_sensor` has `description = "Sensor identifier. Value: <sensor_id> (e.g., 'claroty')."`.
  Wire-shape assertion at serialized JSON output level (name + col_type + nullable + description).
  RED condition: `prism_describe` does not emit these synthesized ColumnDescriptors. Assertions
  (i)-(vi) all fail. Covers AC-015.
- T-11W: Write RG-Q-001 (`test_BC_2_11_016_RG_Q_001`) and RG-Q-002
  (`test_BC_2_11_016_RG_Q_002`) (MUST FAIL). In
  `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`:
  RG-Q-001: build a test `TableRegistry` seeded with OCSF-mode Claroty schema (raw col.names
  only — no OCSF-flattened names yet). Call the E-QUERY-038 plan-time gate for
  `SELECT finding_info_uid FROM claroty_alerts`. Assert `Err(E-QUERY-038)` is returned
  (RED: `finding_info_uid` absent — gate fires incorrectly before Fix A).
  RG-Q-002: same setup, assert `Err(E-QUERY-038)` for `SELECT time FROM claroty_alerts`.
  Covers AC-016 sub-case (b) green-path precondition.
- T-11X: Write RG-Q-003 (`test_BC_2_11_016_RG_Q_003`) (MUST FAIL). Build a test
  `TableRegistry` with OCSF-mode Claroty schema (raw col.names). Invoke E-QUERY-038 +
  E-QUERY-002/041 for `SELECT finding_info_uid FROM claroty_alerts WHERE finding_info_uid = 'x'`.
  Assert gate returns error (RED: `finding_info_uid` absent from raw registry). Covers
  AC-016 sub-case (b) WHERE path and AC-017 sub-case (c). Place in
  `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
- T-11Y: Write RG-Q-004 (`test_BC_2_11_016_RG_Q_004`), RG-Q-005
  (`test_BC_2_11_016_RG_Q_005`), RG-Q-006 (`test_BC_2_11_016_RG_Q_006`), and RG-Q-007
  (`test_BC_2_11_016_RG_Q_007`) (MUST FAIL except RG-Q-007 which tests backward-compat).
  Place all in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  RG-Q-004: assert `SELECT id FROM claroty_alerts` fires E-QUERY-038 when the registry is
  seeded with OCSF-mode schema (OCSF-flattened names only). RED: currently `"id"` is
  registered — gate passes (false negative).
  RG-Q-005: assert `SELECT category FROM claroty_alerts` fires E-QUERY-038 (Tier-2 raw
  col.name absent from OCSF-mode available set). RED: currently `"category"` is registered.
  RG-Q-006: assert the E-QUERY-038 error payload's `available_columns` for an OCSF-mode
  table contains zero raw col.name values; serialize to JSON and assert wire-shape. RED:
  currently raw col.names appear in `available_columns`.
  RG-Q-007: assert `SELECT id FROM claroty_alerts` with a flag-false (`ocsf_column_naming =
  false`) `TableRegistry` returns `Ok(())`. This MUST PASS now AND after Fix A (backward-compat
  regression guard — not a failing test by design).
- T-11Z: Write RG-Q-010 (`test_BC_2_11_016_zero_col_ocsf_table_st_gate_accepts_class_uid_and_sensor`)
  and RG-Q-011 (`test_BC_2_11_016_zero_col_ocsf_table_st_gate_rejects_raw_col_name`) (MUST FAIL).
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  RG-Q-010: build a zero-Tier-1-column OCSF-mode `TableRegistry` (a table with one or more
  columns, all having `ocsf_field = None`). Call the E-QUERY-038 gate for `SELECT class_uid`
  and `SELECT _sensor`. Assert `Ok(())` returned for both. RED: currently zero-column OCSF
  tables fall through the `if !table.columns.is_empty()` guard and do not register class_uid
  or _sensor — E-QUERY-038 fires incorrectly.
  RG-Q-011: same setup, collect the available-columns set from the registry for the OCSF-mode
  table. Assert the set equals `["_sensor", "class_uid"]` sorted. Also assert E-QUERY-038
  returns `Err` for any reference to a raw `col.name`. RED: same root cause as RG-Q-010.
  Report: confirm both fail before LOW-1-FIX. Covers AC-019.
- T-11AA: Write RG-Q-012 (`test_BC_2_16_003_ocsf_collision_j2_reserved_name_rejected_at_spec_load`),
  RG-Q-013 (`test_BC_2_16_003_ocsf_collision_j4_intra_table_duplicate_rejected_at_spec_load`),
  and RG-Q-014 (`test_BC_2_16_003_ocsf_collision_j1_shadow_rejected_at_spec_load`) (MUST FAIL).
  Place in `crates/prism-spec-engine/src/add_sensor_spec.rs` `#[cfg(test)] mod tests`.
  RG-Q-012: construct a minimal `SensorSpec` TOML string (or build a `SensorSpec` directly)
  with a Tier-1 column whose `ocsf_field` value flattens to `"class_uid"` via
  `ocsf_field_to_arrow_name` (e.g., `ocsf_field = "class.uid"` → `"class_uid"`).
  Call `parse_and_validate_spec_toml`. Assert `Err(e)` where `e` contains `"E-SPEC-030"`
  and `"[§J2]"`. RED: currently `parse_and_validate_spec_toml` does not implement
  Validation Rule 8.
  RG-Q-013: build a spec with two Tier-1 columns in the same table both having `ocsf_field`
  values that flatten to the same arrow name (e.g., `"a.b_c"` and `"a_b.c"` both → `"a_b_c"`).
  Assert `Err(e)` containing `"E-SPEC-030"` and `"[§J4]"`.
  RG-Q-014: build a spec where a Tier-1 arrow name (from `ocsf_field_to_arrow_name`) equals
  a DIFFERENT column's raw `col.name` in the same table. Assert `Err(e)` containing
  `"E-SPEC-030"` and `"[§J1]"`.
  Report: confirm RG-Q-012, 013, 014 all fail before OBS-2-FIX. Covers AC-021.
- T-11AB: Write RG-Q-015 (`test_ocsf_projected_names_all_surfaces_agree`) (MUST FAIL).
  Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Build a `TableRegistry` seeded with a multi-column OCSF-mode table spec (both Tier-1 and
  Tier-2 columns). Call `registry.columns_for_table(table_id)` — sort the returned column
  names into a `Vec<String>`. Also call
  `prism_spec_engine::column_mapping::ocsf_projected_column_names(table_spec, true)` — sort
  into a `Vec<String>`. Assert the two sorted Vecs are equal (byte-for-byte).
  RED: before OBS-1-FIX, `ocsf_projected_column_names` does not exist in
  `prism-spec-engine::column_mapping` — compile error or missing function.
  Report: confirm RG-Q-015 fails before OBS-1-FIX. Covers AC-020.
- T-11AC: Write RG-Q-017 — `test_BC_2_11_016_zero_tier1_with_tier2_projects_raw_extensions_and_emits_warning`
  (MUST FAIL). Place in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`.
  Construct a `SensorSpec` with `ocsf_column_naming = true` and a table with zero Tier-1
  columns (no column has `ocsf_field == Some(...)`) and at least one Tier-2 column (at least
  one column has `ocsf_field = None`, e.g., a column `name = "category"` with `ocsf_field = None`).
  Register this spec in a `TableRegistry` (or invoke `register_sensor`) with a `tracing_test`
  subscriber capturing WARN events. Assert ALL THREE of:
  (i) E-QUERY-038 plan-gate returns `Ok(())` for `SELECT class_uid`, `SELECT _sensor`, AND
      `SELECT raw_extensions` (available set is exactly `["_sensor", "class_uid", "raw_extensions"]`);
  (ii) E-QUERY-038 plan-gate returns `Err` for any reference to a raw `col.name` (e.g.,
      `SELECT category` returns E-QUERY-038 — raw col.name absent from OCSF-mode available set);
  (iii) a WARN event with `event_type = "ocsf.zero_tier1_table"` was emitted exactly ONCE
      during registration; `sensor_id` and `table_name` fields match the registered table.
  RED condition: prior to T-31, `register_sensor` for zero-Tier-1-with-Tier-2 tables only
  registers `["_sensor", "class_uid"]` (assertion (i) partially fails — `raw_extensions`
  query triggers E-QUERY-038 incorrectly) and emits no warning (assertion (iii) fails).
  Covers AC-019 A+W sub-case. Traces to BC-2.11.016 EC-11-080 A+W + BC-2.16.002 v2.34
  `ocsf.zero_tier1_table` catalog row + ADR-058 §J6.

- T-GATE: Run `just iter prism-spec-engine --no-fail-fast`, `just iter prism-bin --no-fail-fast`,
  `just iter prism-mcp --no-fail-fast`, `just iter prism-ocsf --no-fail-fast`, and
  `just iter prism-query --no-fail-fast` — confirm RG-001..RG-027, RG-PD-001, RG-028, and
  RG-Q-001..RG-Q-017 fail with correct compile/test-failure reasons (RG-001..004 in
  prism-spec-engine; RG-Q-012/013/014/016 in prism-spec-engine (add_sensor_spec mod tests);
  RG-005..006/008..010/014..022/024/026/027 in prism-bin; RG-007, RG-025,
  and RG-028 in prism-mcp; RG-011/012/023 in prism-ocsf/tests/; RG-013 in
  prism-ocsf/src/mappers/spec_driven.rs mod tests; RG-PD-001 and RG-Q-001..015/RG-Q-017 in
  prism-query; RG-Q-016 in prism-spec-engine add_sensor_spec mod tests). Confirm no regressions
  in non-RG tests (note: RG-Q-007 is a backward-compat green-lock test — it MUST pass before
  Fix A; confirm it passes).
  Report density: 46/21 = 2.19 ≥ 0.5. STOP and wait for implementer dispatch.

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
  `crates/prism-spec-engine/src/column_mapping.rs` (ADR-058 §I1 canonical home —
  NOT `spec_driven_adapter.rs`). Implementation: `ocsf_field.replace('.', "_")`. Add
  `pub mod column_mapping;` to `crates/prism-spec-engine/src/lib.rs` if not already
  present. In `crates/prism-bin/src/spec_driven_adapter.rs`, add
  `use prism_spec_engine::column_mapping::ocsf_field_to_arrow_name;` (compile-time proof
  that prism-bin can import it without a cycle). In `crates/prism-mcp/src/tools/prism_describe.rs`,
  add the same `use prism_spec_engine::column_mapping::ocsf_field_to_arrow_name;` import
  (compile-time proof that prism-mcp can also import it). Run `just iter prism-spec-engine`.
  Makes RG-003 and RG-004 green.
- T-14A: Add `sensor_spec: &SensorSpec` as a new parameter to `pipeline_result_to_record_batch`
  in `spec_driven_adapter.rs` (ADR-058 §D1, ADR-022 §C wiring). Thread the parameter from
  the `fetch()` call site by passing `&self.sensor_spec.spec` (`SpecDrivenSensorAdapter.sensor_spec`
  is `Arc<ResolvedSensorSpec>`; `.spec` is the `SensorSpec` carrying `ocsf_column_naming`).
  Update ALL callers in the `#[cfg(test)] mod tests` block to pass a `SensorSpec` argument:
  (a) the 16 NEW Red Gate test callers (RG-005, RG-006, RG-008, RG-009, RG-010, RG-014 through
  RG-022, RG-026, RG-027) — pass a synthetic `SensorSpec` matching the test's assertion intent;
  (b) the 1 pre-existing test caller (`test_BC_2_01_013_crowdstrike_fql_datetime_index_col_string_equality_safe`)
  — pass a `SensorSpec` with `ocsf_column_naming = false` to preserve current CrowdStrike behavior.
  **TD-VSDD-060 call-site sweep:** Before committing, run
  `rg 'pipeline_result_to_record_batch' crates/prism-bin/ crates/prism-mcp/` to confirm
  no callers outside the enumerated 19 (1 production + 1 pre-existing test + 16 new RG tests + 1 new RG-024).
  Expect zero prism-mcp hits (the function is prism-bin internal). Run `just iter prism-bin --no-fail-fast`.
  After T-14A, RG-024 partially passes (signature now accepts `sensor_spec`), but
  `ocsf_column_naming` branch logic is not yet written — RG-005 still fails on assertion
  until T-14 adds the conditional. Makes RG-006 confirm green (flag-false path uses
  `col.name`; no branch logic change needed; compile error E0061 is resolved).
- T-14: Update `pipeline_result_to_record_batch` to use the conditional branch per
  ADR-058 §I1 Step 2 (see §Acceptance Criteria AC-003 for the exact logic; `sensor_spec`
  parameter is now present after T-14A). Run `just iter prism-bin`. Makes RG-005 green.
  Also makes RG-024 fully green (both branches now exercised via the threaded parameter).
- T-15: In `pipeline_result_to_record_batch`, implement the `ocsf_field == None` →
  `raw_extensions` aggregation path (ADR-058 §I2): when `sensor_spec.ocsf_column_naming =
  true`, suppress columns with `col.ocsf_field == None` from the individual-field schema
  and aggregate their values into a single `"raw_extensions"` Utf8 Arrow field (JSON blob).
  The raw_extensions aggregation loop MUST extract each `ocsf_field == None` column's value
  using the SAME source_path-aware extraction and ENRICH-1 `Value::Array`→compact-JSON-list-string
  normalization pipeline as first-class columns — NOT a naive `r.get(col.name)` (BC-2.16.003
  EC-016-013-028 reworded; ADR-058 §I2 / AC-007a). For a column with
  `source_path = "$.ip_list[*]"` and `ocsf_field = None`, the extraction and normalization
  steps that produce a compact JSON-list string for Array inputs MUST be reused within
  `pipeline_result_to_record_batch`'s aggregation loop; this ensures `ip_list` in
  `raw_extensions` serializes as `"[\"192.168.1.1\",\"10.0.0.1\"]"` (compact string), not
  a nested JSON array. Run `just iter prism-bin`. Makes RG-008 and RG-026 green.
- T-16: Update `prism_describe` per the Tier-1/Tier-2 model in ADR-058 §G /
  BC-2.16.003 §Interpretation A EC-016-013-027:
  (a) **Tier-1** (`ocsf_field == Some`): emit ColumnDescriptor with
      `name = ocsf_field_to_arrow_name(ocsf_field)` and `description = ocsf_field`.
  (b) **Tier-2 prohibition** (`ocsf_field == None`): MUST NOT emit an individual
      ColumnDescriptor for the column — skip it entirely from the per-column iteration.
  (c) **raw_extensions ColumnDescriptor**: after processing all columns, if
      `ocsf_column_naming = true` AND at least one column has `ocsf_field == None`,
      emit exactly ONE additional ColumnDescriptor with the FOUR-FIELD SHAPE:
      - `name = "raw_extensions"`
      - `col_type = prism_core::column::ColumnType::Json` (ADR-058 §G; ADR-024)
      - `nullable = true` (ADR-058 §G / BC-2.16.003 §Interpretation A)
      - `description` = a string identifying it as a JSON object and enumerating every
        `ocsf_field == None` column's `col.name` as a source key (e.g.,
        `"JSON object containing vendor fields not mapped to OCSF: category, alert_type_name, devices_count, alert_class, ot_devices_count"`)
  Run `just iter prism-mcp`. Makes RG-007 green (Tier-1 path) and RG-025 green (Tier-2
  prohibition + `raw_extensions` four-field ColumnDescriptor emission).
- T-16B: Extend `prism_describe` in `crates/prism-mcp/src/tools/prism_describe.rs` to emit
  synthesized ColumnDescriptors for `class_uid` and `_sensor` when `ocsf_column_naming = true`.
  After emitting all Tier-1 descriptors and the `raw_extensions` Tier-2 descriptor, append
  exactly TWO synthesized ColumnDescriptors:
  (a) `ColumnDescriptor { name: "class_uid", col_type: prism_core::column::ColumnType::Integer,
       nullable: false, description: "OCSF event class identifier derived from sensor TOML ocsf_class. Example: 3004 for entity_management (audit_logs), 2004 for detection_finding (alerts, device_alert_relations), 5001 for inventory_info (devices)." }`
  (b) `ColumnDescriptor { name: "_sensor", col_type: prism_core::column::ColumnType::String,
       nullable: false, description: "Sensor identifier. Value: <sensor_id> (e.g., 'claroty')." }`
  These synthesized columns are produced by `pipeline_result_to_record_batch` itself (not
  declared in the TOML spec) and MUST be advertised so the LLM agent knows they are queryable
  as filter targets. Wire-shape assertion: the serialized `prism_describe` output MUST contain
  both ColumnDescriptor entries at the wire level (MUST NOT be only pre-serialization struct).
  Run `just iter prism-mcp`. Makes RG-028 green. Traces to AC-015.
- T-17: Apply all 14 TOML changes to `claroty.sensor.toml` in a single edit per AC-005:
  (1) `ocsf_column_naming = true` at sensor level;
  (2) KF-01: `audit_logs.ocsf_class` = `"entity_management"`;
  (3) KF-02: `devices.ocsf_class` = `"inventory_info"`;
  (4) KF-03: `alerts.id.ocsf_field` = `"finding_info.uid"`;
  (5) KF-04: `alerts.alert_name.ocsf_field` = `"finding_info.title"`;
  (6) OQ-005: `audit_logs.id.ocsf_field` = `"metadata.uid"` (supersedes prior KF-05 remove; human decision 2026-08-21);
  (7) KF-06: `devices.device_type.ocsf_field` = `"device.type_label"`;
  (8) KF-07: `device_alert_relations.alert_id.ocsf_field` = `"finding_info.uid"`;
  (9) KF-08: `alerts.category.ocsf_field` removed;
  (10) KF-09: `alerts.alert_type_name.ocsf_field` removed;
  (11) KF-10: `alerts.devices_count.ocsf_field` removed;
  (12) KF-11: `audit_logs.category.ocsf_field` removed;
  (13) KF-12: `alerts.updated_time.ocsf_field` = `"finding_info.modified_time"`;
  (14) §J3: `devices.device_category.ocsf_field` = `"device.type_category"`.
  Run `just iter prism-spec-engine` to confirm TOML parses correctly; then run `just iter prism-bin`
  to confirm the six wire-shape RGs green. Makes RG-014,
  RG-015, RG-019, RG-020, RG-021, RG-022 green (KF-03/04/OQ-005/KF-06/07/08/09/10/11/12
  wire-shape assertions now pass with corrected TOML — all six reside in prism-bin; `just iter prism-spec-engine`
  alone cannot observe them). Note: RG-009 and RG-010 are
  code-level collision-detection unit tests that build inline synthetic SensorSpecs and
  do NOT depend on claroty.sensor.toml — they are greened by T-21.
- T-18: Update `test_BC_2_11_005_e2e_claroty_query_returns_data` to use `row.get("device_uid")`
  instead of `row.get("uid")`; update the `#[ignore]` comment per AC-008.
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

  **(c) Reserved-name guard (new per ADR-058 §J2, makes RG-027 green):**
  After computing each flattened arrow name `flat = ocsf_field_to_arrow_name(col.ocsf_field)`,
  check if `flat` is one of the four synthesized/reserved names: `"class_uid"`,
  `"category_uid"`, `"_sensor"`, or `"raw_extensions"`. If so, return
  `Err(ArrowError::SchemaError(format!("OCSF field flattening produces reserved Arrow name
  '{flat}' from ocsf_field '{ocsf_field}'; reserved names are class_uid, category_uid,
  _sensor, raw_extensions — change the ocsf_field declaration")))`.
  This guard prevents a user-declared `ocsf_field` from silently colliding with Arrow
  columns synthesized by `pipeline_result_to_record_batch` itself. Without it, a column
  with `ocsf_field = "class.uid"` would produce two `class_uid` columns in the schema
  (one synthesized by the OCSF class resolver, one from the flattened ocsf_field).

  This combined three-part pass is a fail-closed gate per ADR-058 §J2. Arrow 58 does NOT
  detect any of these collision classes; without these checks, collisions produce silent
  wrong-column resolution or malformed schemas. Run `just iter prism-bin`. Makes RG-009,
  RG-010, and RG-027 green.
- T-22: In `class_selector.rs` (`prism-ocsf`) — all in the SAME atomic commit (sub-obligations
  (a)+(b) from AC-009 — Path A live production resolver):
  (a) Add `pub const CLASS_UID_ENTITY_MANAGEMENT: u32 = 3004;`;
  (b-1) Add `"entity_management" => Ok(CLASS_UID_ENTITY_MANAGEMENT)` arm to
  `select_by_class_name` (resolves KF-01 corrected TOML → 3004);
  (b-2) Add `"inventory_info" => Ok(CLASS_UID_DEVICE_INVENTORY_INFO)` arm to
  `select_by_class_name` (resolves KF-02 corrected TOML → 5001; regression guard);
  (c) Update the in-file module doc mapping tables (around the table documenting
  class name→class_uid mappings — was: audit_activity→3001, device→5001) to document the
  corrected/added entries: entity_management→3004, inventory_info→5001, audit_activity→
  DEPRECATED. All three doc tables (the first module-level doc table, the second module-level doc table if present,
  and the inline `select_by_class_name` doc table) MUST be updated — stale doc tables are an F-P1-MED-001 class finding.
  Run `just iter prism-ocsf`. Makes RG-011 and RG-013 green. Then run `just iter prism-bin`. Also makes RG-016 and
  RG-017 green (the wire-shape integration tests in prism-bin depend on these arms existing; `just iter prism-ocsf` alone cannot observe them).
- T-23: In `class_selector.rs` — all in the SAME atomic commit as T-22 (sub-obligations
  (c)+(d) from AC-009 — Path B forward-compat + dead-code annotation):
  (c-1) Change `("claroty", "audit_log")` arm in `select()` from
  `Ok(CLASS_UID_ACCOUNT_CHANGE)` to `Ok(CLASS_UID_ENTITY_MANAGEMENT)` (forward-compat);
  (c-2) Change `("armis", "audit_log")` arm in `select()` from
  `Ok(CLASS_UID_ACCOUNT_CHANGE)` to `Ok(CLASS_UID_ENTITY_MANAGEMENT)` (forward-compat,
  TD-VSDD-097 dim-1 sibling sweep per ADR-058 §I5);
  (d) Annotate the now-dead `"audit_activity"` arm in `select_by_class_name` with a
  deprecation comment: `// DEPRECATED: "audit_activity" was a non-OCSF v1.7.0 string;
  // no production TOML uses this after KF-01 correction (S-ADR058-OCSF-ROUTING-001).
  // Remove after confirming zero TOML instances.`
  Run `just iter prism-ocsf`. Makes RG-012 and RG-023 green.
- T-24: In `spec_driven_adapter.rs::pipeline_result_to_record_batch`, replace the existing
  `.unwrap_or(0)` call on `EventClassSelector::select_by_class_name` with a match that
  emits `tracing::warn!(event_type = "ocsf.unknown_class_name", ocsf_class = %table.ocsf_class,
  sensor_id = %sensor_id, table_name = %table.table_name, "sensor TOML declares unrecognised
  ocsf_class; class_uid defaulted to 0 (BASE_EVENT)")` on the `Err` branch before returning 0
  (per AC-011, ADR-058 §I5 process-gap obligation). The `.unwrap_or(0)` graceful fallback
  is retained — only the observability WARN is added. Run `just iter prism-bin`. Makes RG-018
  green.
- T-25: In `prism-query/src/pushdown.rs`, function `extract_time_window_from_ast`:
  (a) When constructing the `datetime_index_cols` set for a given `SensorSpec` table, insert
  BOTH the raw `col.name` value (e.g., `"timestamp"`) AND the result of
  `ocsf_field_to_arrow_name(col.ocsf_field)` (e.g., `"time"` for `ocsf_field = "time"`) for
  every datetime column that has a non-empty `ocsf_field`. This enables push-down filters
  written against the OCSF-flattened Arrow column name (`"time"`) to be recognized as
  INDEX-eligible by the predicate analysis layer (OQ-001 / AC-014).
  (b) Update the stale doc comment on `extract_time_window_from_ast` (which currently
  references only `col.name` lookup) to reflect the dual-name insert pattern.
  Import `ocsf_field_to_arrow_name` from `prism-spec-engine::column_mapping` — this is the
  canonical home per ADR-058 §I1; no forbidden-cycle violation (prism-query may depend on
  prism-spec-engine per `dependency-graph.md §Dependency Rules Rule 2` Level 6/7 ordering).
  Run `just iter prism-query`. Makes RG-PD-001 green. Traces to AC-014.
- T-26: Fix A — Update `crates/prism-query/src/table_registry.rs` to register OCSF-flattened
  column names for `ocsf_column_naming = true` sensor tables. When seeding a table with
  `ocsf_column_naming = true`, the `TableRegistry` MUST register:
  (a) For each Tier-1 column (with `ocsf_field == Some`): `ocsf_field_to_arrow_name(ocsf_field)`
  instead of (or in addition to) raw `col.name`;
  (b) Synthesized columns: `"class_uid"` (Integer), `"_sensor"` (String);
  (c) `"raw_extensions"` (Json) when ≥1 Tier-2 column (`ocsf_field == None`) exists.
  Raw `col.name` values for OCSF-mode Tier-1 columns MUST NOT be registered (they are
  rejected as-if-absent per EC-11-079 sub-case (a)). Import `ocsf_field_to_arrow_name`
  from `prism-spec-engine::column_mapping` (no forbidden-cycle violation — `prism-query`
  depends on `prism-spec-engine` per `dependency-graph.md §Dependency Rules Rule 2`).
  Run `just iter prism-query`. Makes RG-Q-001, RG-Q-002, RG-Q-007 green; RG-Q-004 and
  RG-Q-005 also become green (raw col.names now absent from OCSF-mode registry). Traces to
  AC-016 (Fix A) and AC-018 (name-agreement invariant).
- T-27: Fix B/C — Update `crates/prism-query/src/engine.rs` to ensure the E-QUERY-038
  column-existence gate (Fix B) and E-QUERY-002/041 type-compat gate (Fix C) both look up
  columns by their OCSF-flattened names for OCSF-mode tables. After Fix A, the
  `TableRegistry` already contains OCSF-flattened names; Fix B/C ensures the gate logic
  reads from the correct registered set.
  (a) Fix B: Verify that the E-QUERY-038 gate in `execute_inner` calls
  `schema_columns(table, OrgId)` (or equivalent) which now returns OCSF-flattened names
  post-Fix-A — no gate logic change may be needed if the gate reads from the registry
  directly. If any hardcoded raw col.name lookup bypasses the registry, fix it to use the
  registry.
  (b) Fix C: Verify that the E-QUERY-002/041 type-compat gate resolves column types by
  the OCSF-flattened name via the same registry. If type lookups are keyed on raw col.name
  (outside the registry), update them to use the OCSF-flattened name for OCSF-mode tables.
  Run `just iter prism-query`. Makes RG-Q-003 and RG-Q-006 green. Also makes RG-Q-008 and
  RG-Q-009 green — these test the multi-tenant `resolved_spec_map` path through the shared
  helper `ocsf_or_raw_column_names_for_table` in `engine.rs`, which calls
  `check_column_availability` (head gate) and `get_initial_available_columns` (pipe-stage
  seed, Site E of the TD-VSDD-060 5-site sweep). Traces to AC-017 (Fix C), AC-016
  `available_columns` wire-shape (Fix B), and AC-016/AC-018 multi-tenant path (Fix B — Site E).
- T-28: LOW-1-FIX — Remove the outer `if !table.columns.is_empty()` guard on the OCSF branch of
  `register_sensor` in `crates/prism-query/src/table_registry.rs`. When `ocsf_column_naming = true`,
  the helpers `ocsf_projected_column_names` and `ocsf_projected_column_types` MUST be called
  unconditionally so that zero-Tier-1-column tables still register `class_uid` (Integer) and
  `_sensor` (String) in the plan-gate available set (ADR-058 §J6; BC-2.11.016 EC-11-080).
  After the fix, `class_uid` and `_sensor` MUST appear in the E-QUERY-038 `available_columns`
  payload even when no column has an `ocsf_field` declaration; raw `col.name` values MUST NOT
  appear in the available set for such tables. Run `just iter prism-query`. Makes RG-Q-010 and
  RG-Q-011 green. Traces to AC-019.
- T-29: OBS-1-FIX — Add `pub fn ocsf_projected_column_names(spec: &SensorSpec, table: &TableSpec) -> Vec<String>`
  and `pub fn ocsf_projected_column_types(spec: &SensorSpec, table: &TableSpec) -> Vec<ColumnType>` to
  `crates/prism-spec-engine/src/column_mapping.rs` as the single authoritative projection implementation
  (ADR-058 §I7 Consolidated-Projection Invariant). Make `ocsf_or_raw_column_names_for_table` in
  `crates/prism-query/src/engine.rs` a thin forward that delegates to `ocsf_projected_column_names`.
  The registry column-name set seeded in T-28 MUST produce a byte-equal sorted set to the output of
  `ocsf_projected_column_names`. Run `just iter prism-spec-engine` then `just iter prism-query`.
  Makes RG-Q-015 green. Traces to AC-020.
- T-30: OBS-2-FIX — Add `validate_ocsf_column_collisions` as Validation Rule 8 in
  `crates/prism-spec-engine/src/add_sensor_spec.rs`. The function must be called inside
  `parse_and_validate_spec_toml` (or equivalent spec-load entry point) before the spec is
  accepted. On a §J1 (Tier-1 name shadows raw col.name of another column), §J2 (flattened name
  equals a synthesized/reserved name: `class_uid`, `category_uid`, `_sensor`, `raw_extensions`),
  or §J4 (two `ocsf_field` declarations flatten to the same Arrow name) collision, return an
  error that contains error code E-SPEC-030 and the applicable collision tag ([§J1], [§J2], or
  [§J4]). At boot this produces a `ConfigInvalid` error and exit 2; on hot-reload, the prior
  valid spec is retained. The runtime `pipeline_result_to_record_batch` §J guard (T-21) MUST
  remain as defense-in-depth — T-30 adds the spec-load gate, it does NOT remove the runtime
  guard. Add a comment in `crates/prism-core/src/error.rs` at the E-SPEC-030 entry identifying
  it as the spec-load OCSF collision error code. Run `just iter prism-spec-engine`. Makes
  RG-Q-012, RG-Q-013, and RG-Q-014 green. Traces to AC-021.
- T-31: A+W-FIX — When `register_sensor` (or the equivalent registration path in
  `crates/prism-spec-engine/src/add_sensor_spec.rs` or `crates/prism-query/src/table_registry.rs`)
  processes an OCSF-mode table (`ocsf_column_naming = true`) that has **zero Tier-1 columns**
  (no column carries `ocsf_field = Some(...)`) and **at least one Tier-2 column** (at least one
  column carries `ocsf_field = None`), two changes are required in the same commit:
  (a) Register `raw_extensions` (Json column type) in the plan-gate available set, so the final
      set is exactly `["_sensor", "class_uid", "raw_extensions"]` (matches `ocsf_projected_column_names`
      output for this case — AC-019 A+W sub-case).
  (b) Emit `tracing::warn!(sensor_id = %sensor_id, table_name = %table_name, event_type = "ocsf.zero_tier1_table",
      "OCSF table has zero Tier-1 columns but ≥1 Tier-2 columns; raw_extensions added to available set")`.
      Emit EXACTLY ONCE at spec-load/registration time (SAP-1 / PG-LP11-001 — BC-2.16.002 v2.34
      `ocsf.zero_tier1_table` catalog row; fields: sensor_id, table_name; recurrence: ONCE per table
      at registration). Do NOT emit on every query.
  Update `ocsf_projected_column_names` helper (introduced in T-29) to return
  `["class_uid", "_sensor", "raw_extensions"]` when both tier-1 count == 0 and tier-2 count > 0
  (the A+W case), continuing to return `["class_uid", "_sensor"]` when tier-2 count == 0 as well.
  Apply OBS-1 param drop in the same commit: remove the `source_path` parameter from
  `validate_ocsf_column_collisions` per ADR-058 §J7 amendment (already anchored to T-30/RG-Q-016;
  this T-31 commit removes `source_path` if T-30 did not already — only if it has not been
  removed yet). Run `just iter prism-spec-engine` then `just iter prism-query`. Makes
  RG-Q-017 green. Traces to AC-019 A+W sub-case.

- T-19: Run `just iter prism-spec-engine`, `just iter prism-bin`, `just iter prism-mcp`,
  `just iter prism-ocsf`, and `just iter prism-query` — all 46 RGTs must pass
  (RG-001..004 in prism-spec-engine; RG-Q-012/RG-Q-013/RG-Q-014/RG-Q-016 in prism-spec-engine
  (add_sensor_spec mod tests); RG-005..006/008..010/014..022/024/026/027 in prism-bin;
  RG-007/RG-025/RG-028 in prism-mcp; RG-011/012/023 in prism-ocsf/tests/; RG-013 in
  prism-ocsf/src/mappers/spec_driven.rs mod tests; RG-PD-001 and
  RG-Q-001..RG-Q-017 in prism-query/src/tests/ocsf_column_routing_tests.rs, except
  RG-Q-012..014 which live in prism-spec-engine add_sensor_spec mod tests,
  RG-Q-015 which lives in prism-query/src/table_registry.rs mod tests, and
  RG-Q-017 which lives in prism-query/src/tests/ocsf_column_routing_tests.rs).
- T-20: Run `just check` — full workspace gate. Must stay GREEN per ADR-058 §E1
  blast-radius analysis. If any non-Claroty tests fail, STOP — do not push.

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

1. `ocsf_field_to_arrow_name` MUST live in `prism-spec-engine::column_mapping` (ADR-058
   §I1). Both `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe`
   import the helper from `prism-spec-engine::column_mapping`. Placing the helper in
   `prism-bin` is FORBIDDEN — `prism-mcp` is Level 6 in the crate topological ordering and
   `prism-bin` is Level 7 (`dependency-graph.md` §Dependency Rules Rule 2: lower-layer
   crates never depend on higher-layer crates); a `prism-mcp → prism-bin` dependency is
   therefore forbidden. The no-cycle guarantee: `prism-bin` and `prism-mcp` both depend on
   `prism-spec-engine` (Level 1); only the reverse direction (spec-engine → bin/mcp) would
   violate the layer rule.

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

11. `pipeline_result_to_record_batch` MUST NOT construct its `SensorSpec` internally
    (no placeholder construction via `SensorSpec::default()` or `SensorSpec::new()`).
    The `sensor_spec: &SensorSpec` parameter MUST be threaded from the `fetch()` call
    site, which holds the actual parsed spec for the sensor being queried. This is the
    ADR-022 §C wiring contract: "wiring, not redesign" means adding proper plumbing where
    it was missing, not constructing a default/placeholder. A placeholder would silently
    use `ocsf_column_naming = false` for all sensors regardless of their TOML config,
    defeating the purpose of Stage 2 entirely and producing a hard-to-diagnose bug.
    Per Standing Rule 3 §4 in CLAUDE.md: "Adding `Arc<dyn Foo>` to a constructor that
    lacked it is 'wiring, not redesign'" — the same principle applies to adding a
    required `&SensorSpec` parameter. (Anchored: S-ADR058-OCSF-ROUTING-001 AC-012 / RG-024)

---

## Library & Framework Requirements

| Library | Role | Constraint |
|---------|------|-----------|
| `serde` | `#[serde(default)]` attribute on `ocsf_column_naming` | Workspace-pinned version |
| `arrow` | `RecordBatch`, `Field`, `DataType` in `pipeline_result_to_record_batch` | Workspace-pinned version in root `Cargo.toml` |
| `serde_json` | JSON serialization for `raw_extensions` blob | Workspace-pinned version |
| `tracing-test` | Capture `tracing` events in RG-018 (`tracing_test` subscriber in `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`) | `tracing-test = "0.2"` in `prism-bin/Cargo.toml` `[dev-dependencies]` — provided by dependency S-ADR058-OCSF-COERCION-001 (which adds this entry for RG-009 first, since COERCION-001 merges before ROUTING-001); VERIFY present, add ONLY if absent — do not create a duplicate key |

`tracing-test = "0.2"` in `prism-bin/Cargo.toml` `[dev-dependencies]` is required for RG-018.
S-ADR058-OCSF-COERCION-001 (which is in `depends_on` and merges first) adds this entry for
its own RG-009; ROUTING-001 inherits it. Implementer MUST verify presence before adding — do
NOT create a duplicate key. The `string.replace('.', "_")` operation for `ocsf_field_to_arrow_name`
uses only `std` — no additional production crate needed.

Do NOT add new `reqwest` dependencies. Do NOT add `native-tls` features.

---

## File Structure Requirements

| File | Action |
|------|--------|
| `crates/prism-spec-engine/src/spec_parser.rs` | Modify: add `#[serde(default)] pub ocsf_column_naming: bool` to `SensorSpec` |
| `crates/prism-spec-engine/src/column_mapping.rs` | Create/Modify: add `pub fn ocsf_field_to_arrow_name(ocsf_field: &str) -> String` (ADR-058 §I1 canonical home); add RG-003..RG-004 to `#[cfg(test)] mod tests` block |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify: import `ocsf_field_to_arrow_name` from `prism_spec_engine::column_mapping` (NOT defined here); update `pipeline_result_to_record_batch` (individual-field naming per ADR-058 §I1 + `ocsf_field == None` → raw_extensions aggregation per ADR-058 §I2); the raw_extensions aggregation loop is schema-construction logic owned by `pipeline_result_to_record_batch`; it MUST reuse the same source_path extraction + ENRICH-1 `Value::Array`→compact-JSON-list-string normalization as first-class columns for each `ocsf_field == None` column value (BC-2.16.003 EC-016-013-028 reworded) |
| `crates/prism-mcp/src/tools/prism_describe.rs` | Modify: `ColumnDescriptor.name` sourcing branches on `sensor_spec.ocsf_column_naming` |
| `crates/prism-sensors/specs/claroty.sensor.toml` | Modify: apply all 14 TOML changes per AC-005 (ocsf_column_naming flag + KF-01..KF-12 + §J3 shadow fix — all in one edit) |
| `crates/prism-ocsf/src/class_selector.rs` | Modify: add `CLASS_UID_ENTITY_MANAGEMENT = 3004`; reroute `"audit_activity"` arm and `("armis","audit_log")` arm to entity_management (3004) per AC-009 |
| `crates/prism-bin/tests/` (e2e test file — TBD at dispatch) | Modify: update `test_BC_2_11_005_e2e_claroty_query_returns_data` assertion |
| `crates/prism-spec-engine/tests/` (new or existing test file) | Modify: add RG-001..RG-002 |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify: add `use prism_spec_engine::column_mapping::ocsf_field_to_arrow_name;` import; add RG-005..RG-006, RG-008..RG-010, RG-014..RG-022, RG-024, RG-026, RG-027 to `#[cfg(test)] mod tests` block (direct calls to `pipeline_result_to_record_batch` and imported `ocsf_field_to_arrow_name` — no public API surface expansion; RG-003..004 moved to prism-spec-engine/column_mapping.rs) |
| `crates/prism-mcp/tests/` (test file — TBD at dispatch) | Modify: add RG-007 and RG-028 |
| `crates/prism-query/src/pushdown.rs` | Modify: dual-name `datetime_index_cols` insert — for each datetime column with non-empty `ocsf_field`, insert BOTH `col.name` and `ocsf_field_to_arrow_name(ocsf_field)` into the index-eligible set; update stale doc comment on `extract_time_window_from_ast`; add RG-PD-001 to `#[cfg(test)] mod tests` block (OQ-001/AC-014) |
| `crates/prism-ocsf/tests/` (new or existing test file) | Modify: add RG-011, RG-012, RG-023 |
| `crates/prism-ocsf/src/mappers/spec_driven.rs` (`#[cfg(test)] mod tests` block) | Modify: add RG-013 (calls private `set_nested_field` — unreachable from `tests/` crate; E0603 if placed in integration test) |
| `crates/prism-bin/Cargo.toml` | Verify/Modify: confirm `tracing-test = "0.2"` is present in `[dev-dependencies]` (added by S-ADR058-OCSF-COERCION-001 for RG-009); add ONLY if absent — do not duplicate | Required for RG-018 `tracing_test` subscriber in `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`; COERCION-001 is the upstream provider (depends_on ordering) |
| `crates/prism-query/src/table_registry.rs` | Modify: when `ocsf_column_naming = true`, register OCSF-flattened column names (`ocsf_field_to_arrow_name(ocsf_field)` for each Tier-1 column) instead of raw `col.name`; also register synthesized columns `class_uid` (Integer) and `_sensor` (String), and `raw_extensions` (Json) when ≥1 Tier-2 column exists. Import `ocsf_field_to_arrow_name` from `prism_spec_engine::column_mapping`. Raw `col.name` values for OCSF-mode Tier-1 columns MUST NOT be registered. (Fix A — AC-016, AC-018, RG-Q-001..005, RG-Q-007) |
| `crates/prism-query/src/engine.rs` | Modify: verify E-QUERY-038 column-existence gate reads registered column names from `TableRegistry` (which now returns OCSF-flattened names post-Fix-A); verify E-QUERY-002/041 type-compat gate looks up column types by OCSF-flattened name for OCSF-mode tables. If any hardcoded raw col.name lookup bypasses the registry, fix it to use the registry. (Fix B/C — AC-016 `available_columns` wire-shape, AC-017, RG-Q-003, RG-Q-006) — **Shared-helper note (re-cascade P1 fix):** the fix introduced `ocsf_or_raw_column_names_for_table` as a shared private helper in this file (single source of truth for OCSF vs raw name set selection); it is called by BOTH `check_column_availability` (head gate — Site B/C) AND `get_initial_available_columns` (pipe-stage seed — Site E). TD-VSDD-060 5-site sweep: (A) `table_registry.rs` register_sensor seed; (B/C/E) `engine.rs` via shared helper; (D) `materialization.rs` `build_bundled_spec_schemas` = test-only path (unaffected, confirmed). |
| `crates/prism-query/src/tests/ocsf_column_routing_tests.rs` | Create: new test file containing RG-Q-001 through RG-Q-009 (`test_BC_2_11_016_RG_Q_001` through `test_BC_2_11_016_RG_Q_009`). These tests verify OCSF-flattened column resolution, raw col.name rejection, `available_columns` OCSF-name wire-shape, and the flag-false green-lock invariant. RG-Q-008 and RG-Q-009 additionally verify the multi-tenant `resolved_spec_map` path through the shared helper `ocsf_or_raw_column_names_for_table` in `engine.rs` (head gate and pipe-stage seed respectively — Site E of the TD-VSDD-060 5-site sweep). (AC-016, AC-017, AC-018, BC-2.11.016 EC-11-079) |

Implementer MUST add private-fn RGs (RG-005..006/008..010/014..022/024/026/027) to the `#[cfg(test)] mod tests` block in `crates/prism-bin/src/spec_driven_adapter.rs` — do NOT place them in `crates/prism-bin/tests/` (separate crate; cannot reach private fns). Similarly, RG-013 calls `set_nested_field`, a private free function in `crates/prism-ocsf/src/mappers/spec_driven.rs`; route RG-013 to the `#[cfg(test)] mod tests` block of that file, NOT to `crates/prism-ocsf/tests/` (E0603 if placed in the integration test crate). For the e2e test update (AC-008), verify file names via `find crates/prism-bin/tests -name "*.rs"` at dispatch.

Do NOT modify: any other sensor TOML spec (CrowdStrike, Armis, Cyberint); any BC or ADR body (product-owner / architect scope). Note: `column_mapping.rs` is IN SCOPE — create/modify `crates/prism-spec-engine/src/column_mapping.rs` per T-13. `class_selector.rs` is in scope for this story (AC-009 code obligation).

---

## Forbidden Dependencies

Build-time enforcement rules:

- `prism-spec-engine` MUST NOT import from `prism-bin`. If `cargo tree -p prism-spec-engine` shows `prism-bin` after this story, a forbidden import was introduced.

- `prism-mcp` MUST NOT import from `prism-bin`. The `ocsf_field_to_arrow_name` helper MUST live in `prism-spec-engine::column_mapping` (ADR-058 §I1) so both `prism-bin` and `prism-mcp` can import it without a forbidden edge. `prism-mcp` is Level 6 and `prism-bin` is Level 7 in the crate topological ordering (`dependency-graph.md` §Dependency Rules Rule 2); a `prism-mcp → prism-bin` dependency is forbidden because lower-layer crates never depend on higher-layer crates. If `cargo tree -p prism-mcp` shows `prism-bin` after this story, the helper was placed in the wrong crate.

- `prism-sensors` MUST NOT gain a dependency on `prism-spec-engine`. If `cargo tree -p prism-sensors` shows `prism-spec-engine`, the story introduced a forbidden import.

- `prism-bin` MUST NOT gain any new `native-tls` features. Verify `Cargo.toml` reqwest entries if any are modified.

---

## TD-VSDD-097 / POL-29 Three-Dimension Sweep Verdict

### v1.54 Amendment Sweep (A+W amendment — human decision 2026-08-23: zero-Tier-1-with-Tier-2 PRESERVES Tier-2 via raw_extensions; NEW ocsf.zero_tier1_table spec-load warning; RG-Q-017 added; density 46/21=2.19; §Authority re-pinned ADR-058 v2.31 / BC-2.11.016 v1.30 / BC-2.16.002 v2.34 / BC-2.16.003 v1.26)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): merged and terminal at v1.47 snapshot (ADR-058 v2.26, BC-2.16.003 v1.21, BC-2.16.002 v2.32). This v1.54 burst adds RG-Q-017 (A+W sub-case: zero-Tier-1-with-Tier-2 projects `raw_extensions` + emits `ocsf.zero_tier1_table` WARN) and the corresponding T-11AC / T-31 tasks. COERCION-001 carries no `ocsf.zero_tier1_table` warning obligation and no `raw_extensions` registration obligation for zero-Tier-1-with-Tier-2 tables — those surfaces belong exclusively to ROUTING-001 (Stage 2 OCSF field-name routing). VERDICT: SIBLING HISTORICAL SNAPSHOT PRESERVED — NO CHANGE NEEDED.

**Dimension 2 — Downstream copy target:**

Changed surfaces in this burst: (1) frontmatter version 1.53→1.54; modified date updated to 2026-08-23; (2) §Authority ADR-058 pin v2.30→v2.31 + status-date 2026-08-23; (3) §Authority BC-2.11.016 pin v1.29→v1.30 + A+W sub-case note (EC-11-080 A+W: zero-Tier-1-with-Tier-2 MUST register raw_extensions + emit ocsf.zero_tier1_table WARN ONCE); (4) §Authority BC-2.16.002 pin v2.33→v2.34 + ocsf.zero_tier1_table catalog row note; (5) §Authority BC-2.16.003 pin v1.24→v1.26 + OBS-1 §J7 signature-drop note; (6) §Behavioral Contracts table BC-2.11.016 v1.29→v1.30, BC-2.16.002 v2.33→v2.34, BC-2.16.003 v1.24→v1.26 (pin updates in "covered by" column); (7) §Red Gate Tests preamble "forty-five"→"forty-six", "45"→"46"; (8) RG-Q-017 entry added (`test_BC_2_11_016_zero_tier1_with_tier2_projects_raw_extensions_and_emits_warning`; three assertions: E-QUERY-038 Ok for raw_extensions, E-QUERY-038 Err for raw col.name, ocsf.zero_tier1_table WARN exactly once); (9) §BC-5.38.001 Density Check 45→46 RGTs, range RG-Q-001..016→RG-Q-001..017, density 2.14→2.19; RG-Q-017 A+W coverage note added; (10) AC-019 extended with A+W sub-case (zero-Tier-1-with-Tier-2 available set = ["_sensor","class_uid","raw_extensions"] + emit ocsf.zero_tier1_table WARN ONCE; SAP-1 obligation; implementation in T-31); (11) §Mandate Anchor table — A+W sub-case row added (AC-019 A+W, RG-Q-017, T-31, PENDING A+W-FIX); (12) §Tasks Phase A — T-11AC added (write RG-Q-017, MUST FAIL, before T-GATE); T-GATE range RG-Q-001..016→RG-Q-001..017; density 45/21=2.14→46/21=2.19; (13) §Tasks Phase B — T-31 added (A+W-FIX: register raw_extensions for zero-Tier-1-with-Tier-2 + emit ocsf.zero_tier1_table WARN ONCE; update ocsf_projected_column_names helper; apply OBS-1 param drop if not done in T-30); T-19 count 45→46; range extended to include RG-Q-017; (14) §TD-VSDD-097 — this v1.54 sweep added at top; (15) §Changelog — v1.54 row added at top. None of the changed loci are verbatim-copied into any downstream artifact — they are story-body spec prose consumed by implementer/test-writer agents at dispatch time. The §Authority pins are machine-readable version strings consumed by spec-steward tooling; no independent copy artifact transcribes them verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

New MUST introduced: "When `ocsf_column_naming = true` and a registered table has zero Tier-1 columns but ≥1 Tier-2 columns, `register_sensor` MUST register `raw_extensions` (Json) in the plan-gate available set AND MUST emit `ocsf.zero_tier1_table` WARN exactly once at spec-load time (BC-2.16.002 v2.34 catalog row; fields: sensor_id, table_name; recurrence: ONCE per table at registration)." This new MUST is anchored to: AC-019 A+W sub-case → RG-Q-017 (`test_BC_2_11_016_zero_tier1_with_tier2_projects_raw_extensions_and_emits_warning`) + T-31 (A+W-FIX implementation task). Both anchor targets are in this same burst. BC-2.11.016 EC-11-080 A+W is the contract source; BC-2.16.002 v2.34 `ocsf.zero_tier1_table` catalog row is the SAP-1 emit contract; ADR-058 §J6 is the architectural decision. VERDICT: NEW MUST ANCHORED TO RG-Q-017 + T-31 IN SAME BURST.

SAC-1 re-verified: 46 RGTs, density 46/21 = 2.19 ≥ 0.5, red-then-green ordering preserved (T-11AC test-authoring task in Phase A precedes T-31 implementation task in Phase B).

### v1.53 Amendment Sweep (LOCAL pass-1 fix-burst — RG-Q-016 added (H1 §J1 Tier-1-vs-Tier-1 closure); RG-Q-015 strengthened to bind §I7 shape-exception sites (M2 closure); density 45/21=2.14; human-directed 2026-08-22)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): merged and terminal at v1.47 snapshot (ADR-058 v2.26, BC-2.16.003 v1.21, BC-2.16.002 v2.32). This v1.53 burst adds RG-Q-016 (§J1 Tier-1-vs-Tier-1 shadow sub-case) — a sub-case of the §J1 validator already in ROUTING-001's AC-021/T-30 scope. COERCION-001 carries no `validate_ocsf_column_collisions` Tier-1-vs-Tier-1 obligation; the §J1 validation is specific to OCSF field-name routing (Stage 2). VERDICT: SIBLING HISTORICAL SNAPSHOT PRESERVED — NO CHANGE NEEDED.

**Dimension 2 — Downstream copy target:**

Changed surfaces in this burst: (1) frontmatter version 1.52→1.53; (2) §Red Gate Tests preamble 44→45; (3) RG-Q-015 entry: note added that it now also binds the two ADR-058 §I7 shape-exception sites (prism-mcp `build_ocsf_column_descriptors` name-set == `ocsf_projected_column_names` assertion and prism-bin `pipeline_result_to_record_batch` Arrow schema field-names == `ocsf_projected_column_names` assertion); these are RG-Q-015 sub-assertions, closing M2; (4) RG-Q-016 entry added (§J1 Tier-1-vs-Tier-1 shadow sub-case; E-SPEC-030 [§J1]; prism-spec-engine add_sensor_spec mod tests); (5) §BC-5.38.001 Density Check 44→45 RGTs, range RG-Q-001..015→RG-Q-001..016, density 2.10→2.14; RG-Q-016 coverage note added; (6) §Mandate Anchor table AC-021 row RG column: RG-Q-012/013/014 → RG-Q-012/013/014/016; (7) §Behavioral Contracts table BC-2.16.003 row EC-016-013-032 governs: /013/014 → /013/014/016; (8) T-GATE range RG-Q-001..015→RG-Q-001..016; prism-spec-engine distribution RG-Q-012/013/014→RG-Q-012/013/014/016; density 44/21=2.10→45/21=2.14; (9) T-19 count 44→45; prism-spec-engine distribution updated; (10) §TD-VSDD-097 — this v1.53 sweep added at top; (11) §Changelog — v1.53 row added at top. None of the changed loci are verbatim-copied into any downstream artifact — they are story-body spec prose consumed by implementer/test-writer agents at dispatch time. The §Authority pins are unchanged (ADR-058 v2.30, BC-2.16.003 v1.24, BC-2.11.016 v1.29, BC-2.16.002 v2.33 — no spec (ADR/BC/error-taxonomy) change in this burst). VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

RG-Q-016 is a sub-assertion of AC-021 (§J1 Tier-1-vs-Tier-1 sub-case). AC-021 already has a mandate anchor entry pointing to T-30 (`validate_ocsf_column_collisions` Validation Rule 8). RG-Q-016 extends the existing AC-021 Red Gate coverage — no new MUST was introduced in this burst (the MUST for §J1 collision rejection was already anchored in v1.52 via AC-021 → T-30). VERDICT: NO NEW MUSTs — EXISTING ANCHOR T-30 COVERS RG-Q-016.

SAC-1 re-verified: 45 RGTs, density 45/21 = 2.14 ≥ 0.5, red-then-green ordering preserved (no new Phase A test-writing task required — RG-Q-016 was written in LOCAL pass-1 fix-burst post-implementation; documented here for SAC-1 traceability only).

### v1.52 Amendment Sweep (re-cascade LOW-1/OBS-1/OBS-2 fix spec burst; AC-019/020/021 + RG-Q-010..015; §Authority re-pinned ADR-058 v2.28→v2.30, BC-2.11.016 v1.28→v1.29, BC-2.16.003 v1.23→v1.24; E-SPEC-030; human-directed 2026-08-22)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): merged and terminal at v1.47 snapshot (ADR-058 v2.26, BC-2.16.003 v1.21, BC-2.16.002 v2.32). This v1.52 burst adds three new ACs (AC-019/020/021) and six new RGTs (RG-Q-010..015) covering: zero-Tier-1-column OCSF table registration (ADR-058 §J6; BC-2.11.016 EC-11-080), consolidated-projection invariant (ADR-058 §I7), and spec-load collision validation via `validate_ocsf_column_collisions` (ADR-058 §J7; BC-2.16.003 EC-016-013-032). COERCION-001 carries none of these surfaces: no `register_sensor` zero-column guard, no `ocsf_projected_column_names`/`ocsf_projected_column_types` obligation, no `validate_ocsf_column_collisions` obligation, no EC-016-013-032 or EC-11-080. VERDICT: SIBLING HISTORICAL SNAPSHOT PRESERVED — NO CHANGE NEEDED.

**Dimension 2 — Downstream copy target:**

Changed surfaces in this burst: (1) frontmatter version 1.51→1.52; modified date 2026-08-22; (2) §Authority ADR-058 pin v2.28→v2.30 + status-date; (3) §Authority BC-2.16.003 pin v1.23→v1.24 + EC-016-013-032 note; (4) §Authority BC-2.11.016 pin v1.28→v1.29 + EC-11-080 entry (new §Authority paragraph); (5) §Behavioral Contracts table BC-2.16.003 v1.23→v1.24, BC-2.11.016 v1.28→v1.29; (6) §Red Gate Tests preamble 38→44; RG-Q-010..015 individual entries added; (7) §BC-5.38.001 Density Check 38→44 RGTs, 18→21 ACs, 2.11→2.10; RG-Q-010..015 coverage notes; (8) AC-019/020/021 added; (9) §Mandate Anchor table — three new PENDING rows for AC-019 (RG-Q-010/011, T-28), AC-020 (RG-Q-015, T-29), AC-021 (RG-Q-012/013/014, T-30); (10) §Tasks Phase A — T-11Z/T-11AA/T-11AB test-authoring tasks; T-GATE count/density updated; (11) §Tasks Phase B — T-28/T-29/T-30 implementation tasks added after T-27; T-19 count 38→44, range extended to RG-Q-001..015 with prism-spec-engine distribution; (12) §TD-VSDD-097 — this v1.52 sweep added at top; (13) §Changelog — v1.52 row added at top; (14) body ADR-058 section cites normalized to version-free per POL-39/POL-40 (15 body loci: lines with §J2, §B2/§I2, §B2/§I2, §J2 etc. stripped of v2.28 qualifier). None of the changed loci are verbatim-copied into any downstream artifact — they are story-body spec prose consumed by implementer/test-writer agents at dispatch time. The §Authority pins are terminal (consumed by spec-steward tooling); no independent copy artifact exists. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

New MUST statements introduced in AC-019/020/021:
- AC-019 MUST: `TableRegistry` MUST register `class_uid` + `_sensor` for zero-Tier-1-column OCSF tables; outer `if !table.columns.is_empty()` guard MUST be removed → anchored to RG-Q-010 (`test_BC_2_11_016_zero_col_ocsf_table_st_gate_accepts_class_uid_and_sensor`) + RG-Q-011 (`test_BC_2_11_016_zero_col_ocsf_table_st_gate_rejects_raw_col_name`) + T-28
- AC-020 MUST: `ocsf_projected_column_names` / `ocsf_projected_column_types` MUST be the single authoritative projection impl in `prism-spec-engine::column_mapping`; `ocsf_or_raw_column_names_for_table` MUST be a thin forward → anchored to RG-Q-015 (`test_ocsf_projected_names_all_surfaces_agree`) + T-29
- AC-021 MUST: `parse_and_validate_spec_toml` MUST reject §J1/§J2/§J4 collisions via `validate_ocsf_column_collisions` (Validation Rule 8); error MUST contain E-SPEC-030 + collision tag → anchored to RG-Q-012 (`test_BC_2_16_003_ocsf_collision_j2_reserved_name_rejected_at_spec_load`) + RG-Q-013 (`test_BC_2_16_003_ocsf_collision_j4_intra_table_duplicate_rejected_at_spec_load`) + RG-Q-014 (`test_BC_2_16_003_ocsf_collision_j1_shadow_rejected_at_spec_load`) + T-30

No unanchored MUSTs. VERDICT: ALL NEW MUSTs ANCHORED.

SAC-1 re-verified: 44 RGTs, density 44/21 = 2.10 ≥ 0.5, red-then-green ordering preserved (T-11Z/T-11AA/T-11AB test-authoring tasks in Phase A precede T-28/T-29/T-30 implementation tasks in Phase B).

### v1.51 Amendment Sweep (re-cascade P1 HIGH-001/MED-002 closure — RG-Q-008/009 multi-tenant + pipe coverage; shared-helper Site-E fix; human-directed 2026-08-22)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): merged and terminal at v1.47 snapshot (ADR-058 v2.26, BC-2.16.003 v1.21, BC-2.16.002 v2.32). This v1.51 burst adds two multi-tenant RGTs (RG-Q-008/009) and documents the shared-helper `ocsf_or_raw_column_names_for_table` in `engine.rs`. COERCION-001 carries no `engine.rs` column-routing surface, no `check_column_availability` or `get_initial_available_columns` obligations, and no `resolved_spec_map` multi-tenant path. VERDICT: SIBLING HISTORICAL SNAPSHOT PRESERVED — NO CHANGE NEEDED.

**Dimension 2 — Downstream copy target:**

Changed surfaces: (1) frontmatter version 1.50→1.51; (2) §Red Gate Tests preamble — "thirty-six"→"thirty-eight", "36"→"38"; (3) RG-Q-008 entry added (multi-tenant head-gate `check_column_availability` via shared helper); (4) RG-Q-009 entry added (multi-tenant pipe-stage `get_initial_available_columns` — Site E); (5) §BC-5.38.001 Density Check — count 36→38, range RG-Q-001..007→RG-Q-001..009, density 2.00→2.11; RG-Q-008/009 coverage note added; (6) AC-016 "Covered by" line updated to RG-Q-001..009; (7) AC-018 "Covered by" line updated to nine tests; (8) §Mandate Anchor table — AC-018 row RG range updated to RG-Q-001..009; (9) §Behavioral Contracts table — BC-2.11.016 row RG range updated; (10) T-GATE count 36→38, range RG-Q-001..007→RG-Q-001..009; density 36/18→38/18; (11) T-27 "Makes green" extended to include RG-Q-008/009 with shared-helper explanation; (12) T-19 count 36→38, range updated; (13) §File Structure Requirements — `engine.rs` row extended with shared-helper note and 5-site sweep; `ocsf_column_routing_tests.rs` row extended with RG-Q-008/009; (14) §TD-VSDD-097 — this v1.51 sweep added at top. None of the changed loci are verbatim-copied into any downstream artifact — they are story-body prose consumed by implementer/test-writer agents at dispatch time. The shared-helper name `ocsf_or_raw_column_names_for_table` is recorded as an implementation fact for traceability; it is not a BC or ADR obligation that propagates downstream. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST statements introduced. The shared helper `ocsf_or_raw_column_names_for_table` is recorded as implementation guidance (how the fix was delivered), not as a new MUST obligation. The MUST obligations that RG-Q-008/009 exercise are already anchored:
- AC-016 MUST (a): `TableRegistry` OCSF-flattened name registration → anchored to RG-Q-001/002/004/005/006 + T-26 (unchanged)
- AC-018 MUST: Name-agreement invariant → anchored to RG-Q-001..009 (range extended to include multi-tenant path)
- No new unanchored MUSTs introduced. VERDICT: ALL MUSTs ANCHORED.

SAC-1 re-verified: 38 RGTs, density 38/18 = 2.11 ≥ 0.5, red-then-green ordering (Phase A test-writing tasks precede Phase B implementation tasks; RG-Q-008/009 were written as part of the Fix B extension in T-27 scope, consistent with the existing red-then-green discipline for this cascade).

### v1.50 Amendment Sweep (holdout-gap query-surface formalization — BC-2.11.016 added; AC-016/017/018 + RG-Q-001..007; T-26/T-27 Fix A/B/C; file structure additions; human-directed 2026-08-22)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): merged and terminal at v1.47 snapshot (ADR-058 v2.26, BC-2.16.003 v1.21, BC-2.16.002 v2.32). These are the correct frozen pins for COERCION-001 at its merge state — this v1.50 burst adds OCSF-mode query-surface formalization (BC-2.11.016 EC-11-079) which is ROUTING-001 scope only. COERCION-001 carries no BC-2.11.016 obligations and no E-QUERY-038 / `TableRegistry` OCSF-name-routing surface. VERDICT: SIBLING HISTORICAL SNAPSHOT PRESERVED — NO CHANGE NEEDED.

**Dimension 2 — Downstream copy target:**

Changed surfaces: (1) frontmatter version 1.49→1.50; modified 2026-08-21→2026-08-22; (2) `behavioral_contracts:` frontmatter — BC-2.11.016 added as fourth entry; (3) §Behavioral Contracts body table — BC-2.11.016 row added (v1.28, active); (4) §Red Gate Tests — preamble updated to 36 tests; RG-Q-001..RG-Q-007 individual entries added; (5) §BC-5.38.001 Density Check — updated to 36 RGTs / 18 ACs = 2.00; (6) §Acceptance Criteria — AC-016, AC-017, AC-018 added; (7) Token Budget — "3 BCs" → "4 BCs"; (8) §Tasks Phase A — T-11W/T-11X/T-11Y added, T-GATE updated to 36 tests; (9) §Tasks Phase B — T-26 (Fix A), T-27 (Fix B/C) added; T-19 updated to 36 RGTs with RG-Q distribution; (10) §File Structure Requirements — three new rows for `table_registry.rs`, `engine.rs`, `ocsf_column_routing_tests.rs`; (11) §Mandate Anchor table — four new rows for BC-2.11.016 EC-11-079 MUSTs; (12) §TD-VSDD-097 — this v1.50 sweep added. BC-2.11.016 EC-11-079 is cited (not copied verbatim); it is the source-of-truth upstream for AC-016/017/018 and will not be transcribed into any downstream artifact verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

New MUST statements introduced in AC-016/017/018 and their derivation from BC-2.11.016 EC-11-079:
- AC-016 MUST (a): `TableRegistry` MUST register OCSF-flattened names for `ocsf_column_naming = true` tables → anchored to RG-Q-001/RG-Q-002 (`test_BC_2_11_016_RG_Q_001`, `test_BC_2_11_016_RG_Q_002`) + T-26
- AC-016 MUST (b): Raw TOML `col.name` values MUST NOT appear in `available_columns` for OCSF-mode tables → anchored to RG-Q-004/RG-Q-005 (`test_BC_2_11_016_RG_Q_004`, `test_BC_2_11_016_RG_Q_005`) + T-26
- AC-017 MUST: E-QUERY-002/041 type-compat MUST resolve by OCSF-flattened name → anchored to RG-Q-003 (`test_BC_2_11_016_RG_Q_003`) + T-27
- AC-018 MUST: Name-agreement invariant — `available_columns` set MUST match `prism_describe` and `SELECT *` column sets → anchored to RG-Q-001..007 + T-26/T-27

No unanchored MUSTs. VERDICT: ALL NEW MUSTs ANCHORED.

### v1.49 Amendment Sweep (records-tier pin-consistency sweep — BC-2.16.002 v2.32→v2.33; ADR-058 v2.26 straggler refs →v2.28; human-directed 2026-08-21)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): merged and terminal at v1.47 snapshot (ADR-058 v2.26, BC-2.16.003 v1.21, BC-2.16.002 v2.32). These are the correct frozen pins for COERCION-001 at its merge state — this v1.49 sweep applies only to ROUTING-001 scope. VERDICT: SIBLING HISTORICAL SNAPSHOT PRESERVED — NO CHANGE NEEDED.

**Dimension 2 — Downstream copy target:**

Changed surfaces: (1) frontmatter version 1.48→1.49; (2) §Authority BC-2.16.002 Version `2.32`→`2.33`; (3) §Behavioral Contracts table BC-2.16.002 row v2.32→v2.33; (4) §Authority BC-2.16.003 paragraph EC-016-013-029 inline `ADR-058 v2.26 §J2`→`ADR-058 v2.28 §J2` (straggler — never advanced past v2.26 in v1.47/v1.48 because not matched by the `"ADR-058 v2.26 §"` replace_all pattern; this ref has suffix `§J2` preceded by a space+parens boundary, not `§` with trailing alphanumeric directly); (5) RG-026 intro `ADR-058 v2.26 §B2/§I2)`→`ADR-058 v2.28 §B2/§I2)` (same straggler class — compound sub-section reference); (6) AC-007c trace `ADR-058 v2.26 §B2 / §I2`→`ADR-058 v2.28 §B2 / §I2` (same straggler class — slash-spaced compound form). Input-hash unchanged at b49d41f — BC-2.16.002 is NOT in ROUTING-001 tracked inputs (confirmed by v1.45 changelog: "BC-2.16.002 is NOT in ROUTING-001 inputs"), and no tracked input files (ADR-058 at v2.28, BC-2.16.003 at v1.23, code files at frozen feature HEAD dad86a1dc) were modified in this sweep. None of these loci are verbatim-copied into any downstream artifact — they are story-body spec prose consumed by implementer/test-writer agents at dispatch time. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced. All existing MUST anchors carried forward with behavioral content unchanged — only version-pin numerals updated. VERDICT: NO NEW UNANCHORED MUSTs.

### v1.48 Amendment Sweep (LOCAL pass-2 MED-1 + HIGH-1 spec-side closure — AC-015/T-16B description strings aligned to canonical; RG-028/T-11V strengthened with description assertions; ADR-058 v2.27→v2.28; BC-2.16.003 v1.22→v1.23; 2026-08-21)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): merged and terminal at v1.47 snapshot (ADR-058 v2.26, BC-2.16.003 v1.21, BC-2.16.002 v2.32). The MED-1 and HIGH-1 closures apply only to ROUTING-001 scope (`prism_describe` synthesized ColumnDescriptor description strings; RG-028/T-11V test assertion counts). COERCION-001 carries no synthesized-descriptor obligations and no RG-028 or T-11V surface. VERDICT: SIBLING VERIFIED CLEAN — NO CHANGE NEEDED.

**Dimension 2 — Downstream copy target:**

Changed surfaces: (1) §Authority ADR-058 v2.27→v2.28 (11 active-body occurrences via replace_all "ADR-058 v2.27 §"→"ADR-058 v2.28 §", plus 4 special cases: title pin, Version `2.27`→`2.28`, "The v2.27 §J2" mandate narrative, "ADR-058 v2.27." RG-027 intro); (2) §Authority BC-2.16.003 v1.22→v1.23 (§Authority entry + §Behavioral Contracts table); (3) AC-015 class_uid description: "OCSF class identifier synthesized from ocsf_class; queryable as INTEGER column" → verbatim canonical ADR-058 §G / BC-2.16.003 string; (4) AC-015 _sensor description: "Sensor identifier synthesized by pipeline_result_to_record_batch; queryable as STRING column" → verbatim canonical; (5) AC-015 wire-shape assertion: "name, col_type, and nullable" → "name, col_type, nullable, and description"; (6) RG-028 "asserts ALL FOUR"→"ALL SIX": assertions (v)+(vi) added for class_uid and _sensor description text; wire-shape updated to include description; RED condition "(i)-(iv) all fail"→"(i)-(vi) all fail"; (7) T-11V "assert ALL FOUR"→"ALL SIX": matching updates for assertions (v)+(vi) and wire-shape note; (8) T-16B class_uid description aligned to canonical; (9) T-16B _sensor description aligned to canonical; (10) version 1.47→1.48; input-hash f23f905→b49d41f (ADR-058 v2.28 and BC-2.16.003 v1.23 are the new input versions); (11) §v1.48 TD-VSDD-097 sweep added. None of these loci are verbatim-copied into any downstream artifact — they are story-body spec prose consumed by implementer/test-writer agents at dispatch time. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

Description-string MUSTs for synthesized ColumnDescriptors: the canonical class_uid description `"OCSF event class identifier derived from sensor TOML ocsf_class. Example: 3004 for entity_management (audit_logs), 2004 for detection_finding (alerts, device_alert_relations), 5001 for inventory_info (devices)."` and `_sensor` description `"Sensor identifier. Value: <sensor_id> (e.g., 'claroty')."` are now explicitly specified in AC-015 and required by RG-028 assertions (v)+(vi). These MUSTs are anchored: description strings → AC-015 → RG-028(v)+(vi); implementation obligation → T-16B → makes RG-028 green; test-authoring obligation → T-11V(v)+(vi). No new unanchored MUSTs introduced. VERDICT: ALL NEW MUSTs ANCHORED.

### v1.47 Amendment Sweep (Stage 2 spec-augmentation burst — OQ-001/OQ-003/OQ-005 human decisions 2026-08-21; ADR-058 v2.26→v2.27; BC-2.16.003 v1.21→v1.22; RG-021 flip; RG-PD-001/RG-028 new; AC-014/AC-015 new; T-11U/T-11V/T-16B/T-25 new; density 27/13→29/15=1.93; prism-query crate added)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): merged at v1.47 with correct pins (ADR-058 v2.26, BC-2.16.003 v1.21 active, BC-2.16.002 v2.32). COERCION-001 is merged and terminal — the OQ-001/003/005 changes apply only to ROUTING-001 scope (pushdown.rs, prism_describe.rs, claroty.sensor.toml `audit_logs.id` ocsf_field). The sibling story carries no pushdown or synthesized-descriptor obligations and is unaffected. VERDICT: SIBLING VERIFIED CLEAN — NO CHANGE NEEDED.

**Dimension 2 — Downstream copy target:**

Changed surfaces: (1) §Authority ADR-058 v2.26→v2.27 + date 2026-08-20→2026-08-21; (2) §Authority BC-2.16.003 v1.21→v1.22 + date 2026-08-20→2026-08-21; (3) §Behavioral Contracts table BC-2.16.003 row v1.21→v1.22; (4) Red Gate preamble 27→29; (5) Mandate Anchor v2.26→v2.27 (§J2 rows); (6) RG-026/027 traces v2.26→v2.27; (7) RG-021 full replacement (OQ-005 Tier-1 metadata_uid); (8) RG-PD-001 and RG-028 new Red Gate tests; (9) BC-5.38.001 density 27/13=2.08→29/15=1.93; (10) AC-005 entry 6 KF-05→OQ-005; (11) audit_logs mapping table id row; (12) ocsf_field count 26→27; (13) AC-010 assertion 5 KF-05→OQ-005; (14) AC-013 trace v2.26→v2.27; (15) EC-009 26/6→27/7; (16) EC-016-013-028 trace v2.26→v2.27; (17) AC-014 and AC-015 new; (18) §Architecture Mapping prism-query pushdown row added; (19) T-11N updated for OQ-005; T-11S/T-11T traces v2.26→v2.27; T-11U/T-11V/T-16B/T-25 new; (20) T-GATE 27→29; T-17 item 6 and note KF-05→OQ-005; T-21 clause (c) v2.26→v2.27; T-19 27→29 + prism-query; §File Structure prism-mcp/tests/ row updated + prism-query/pushdown.rs row added; crates_touched gains prism-query; input-hash ca528ff→f23f905. None of these loci are verbatim-copied into any downstream artifact — they are story-body spec prose consumed by implementer/test-writer agents at dispatch time. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

New MUSTs introduced via new tasks (T-25 dual-name insert MUST use `ocsf_field_to_arrow_name` canonical home per ADR-058 §I1) and RG-PD-001/RG-028 (which carry "fails until" obligations). Each new obligation is anchored: T-25 → AC-014 → RG-PD-001; T-16B → AC-015 → RG-028; OQ-005 TOML correction → AC-005 entry 6 → RG-021. No unanchored MUSTs introduced. VERDICT: ALL NEW MUSTs ANCHORED.

### v1.46 Amendment Sweep (Pre-delivery burst — version pin sweep v2.23→v2.26 / BC-2.16.003 v1.19→v1.21 active / BC-2.16.002 v2.30→v2.32; holdout_scenarios wired; input-hash refreshed 859dc7f→ca528ff)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): merged at v1.47 with correct pins (ADR-058 v2.26, BC-2.16.003 v1.21 active, BC-2.16.002 v2.32). No change needed. This pre-delivery burst applies to ROUTING-001 only — a story-only bump; COERCION-001 is already merged and its pins are at final frozen values. VERDICT: SIBLING VERIFIED CLEAN — NO CHANGE NEEDED.

**Dimension 2 — Downstream copy target:**

Changed surfaces: (1) §Authority ADR-058 leading pin v2.23→v2.26 + status-date 2026-08-19→2026-08-20; (2) §Authority BC-2.16.003 pin v1.19→v1.21 + status draft→active + date 2026-08-19→2026-08-20; (3) §Authority BC-2.16.002 pin v2.30→v2.32; (4) §Behavioral Contracts table BC-2.16.003 row v1.19 draft→v1.21 active; (5) §Behavioral Contracts table BC-2.16.002 row v2.30→v2.32; (6) replace_all sweep "ADR-058 v2.23 §"→"ADR-058 v2.26 §" (14 active-body occurrences: BC-2.16.003 §Authority body line 172; §Mandate Anchor table lines 226/227; §Red Gate Tests RG-026/RG-027 covers/traces lines 611/634; §BC-5.38.001 density check lines 672/676; §Acceptance Criteria AC-007c trace line 1050; §Edge Cases EC-016-013-029 trace line 1333 and EC-016-013-028 row line 1381; §Tasks T-11S line 1581, T-11T line 1591, T-21 clause (c) line 1725); (7) special-case "The v2.23 §J2"→"The v2.26 §J2" (line 213); (8) special-case "ADR-058 v2.23." → "ADR-058 v2.26." RG-027 intro (line 615); (9) holdout_scenarios []→[HS-ROUTING-001-A-001..HS-ROUTING-001-A-004]; (10) input-hash 859dc7f→ca528ff. None of these loci are verbatim-copied into any downstream artifact — they are story-body spec prose consumed by implementer/test-writer at dispatch time; no BC or ADR carries a verbatim copy. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced. All existing MUST anchors carried forward with behavioral content unchanged — only version-pin numerals updated. VERDICT: NO NEW UNANCHORED MUSTs.

### v1.44 Amendment Sweep (Leg 2 pin bump — BC-2.16.003 v1.18→v1.19 + BC-2.16.002 v2.28→v2.29; 13 §Interpretation A v1.18 inline stamps stripped to version-free form)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): amended in same burst (v1.39→v1.40) — §Authority BC-2.16.003 pin v1.18→v1.19; §Authority BC-2.16.002 pin v2.28→v2.29; §Behavioral Contracts table BC-2.16.003 v1.18→v1.19 and BC-2.16.002 v2.28→v2.29. VERDICT: SIBLING AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

Changed surfaces: (1) §Authority BC-2.16.003 `Version \`1.18\`` → `Version \`1.19\``; (2) §Authority BC-2.16.002 `Version \`2.28\`` → `Version \`2.29\``; (3) §Behavioral Contracts table BC-2.16.003 row v1.18→v1.19; (4) §Behavioral Contracts table BC-2.16.002 row v2.28→v2.29; (5) 13 inline `§Interpretation A v1.18` active-body stamps stripped to `§Interpretation A` across §BC-5.39.001 context (line 155), §Mandate Anchor table (line 224), §Red Gate Tests RG-025 (lines 551, 572, 669), §Acceptance Criteria AC-006 (line 951), §Acceptance Criteria AC-006 trace (line 973), §Acceptance Criteria AC-007b (line 1006), §Acceptance Criteria AC-007 traces (line 1026), §Edge Cases EC-016-013-027 row (line 1380), §Tasks T-11R (line 1569), §Tasks T-16 col_type line (line 1658), §Tasks T-16 nullable line (line 1668). None of these loci are verbatim-copied into any downstream artifact. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced. All existing MUST anchors carried forward with behavioral content unchanged — only version-stamp decorations removed and pin numbers bumped. VERDICT: NO NEW UNANCHORED MUSTs.

---

### v1.43 Amendment Sweep (FB-62/63 TERMINAL POL-39 normalization — complete active-body sweep of ADR-058 section version stamps)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): verified the entire active body of COERCION-001 via whole-file grep `§[A-Z][0-9]* v[0-9]` — zero hits in active text; only hit is on a §Changelog row (grandfathered). COERCION-001 requires no edit and receives no version bump. VERDICT: SIBLING VERIFIED CLEAN.

**Dimension 2 — Downstream copy target:**

The changed surfaces are the 22 version qualifier strips across the following active-body sections: §Mandate Anchor table (§D1/§G/§I1 row cites); §Red Gate Tests (RG-003 §I1; RG-024 §D1; RG-025 §G × 4 instances); §BC-5.38.001 density check (§G); §Acceptance Criteria (AC-002 §I1; AC-003 §D1; AC-006 §G × 2; AC-007b §G × 2; AC-012 §D1 × 2); §Architecture Mapping table (§I1; §D1; §G); §Architecture Compliance Rules Rule 1 (§I1); §File Structure Requirements column_mapping.rs row (§I1); §Forbidden Dependencies (§I1); §Edge Cases table EC-016-013-027 row (§G); §Tasks T-06 (§I1); T-11R (§G); T-13 (§I1); T-16 (§G × 3). None of these loci are verbatim-copied into any downstream artifact — they are spec-body prose consumed by implementer/test-writer agents at dispatch time, not transcribed into BC or ADR bodies. No BC or ADR carries a verbatim copy of these story-body cites. VERDICT: CLEAR — no downstream copy target requires simultaneous update.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced. All existing MUST anchors (§D1, §G, §I1, §I2, §J2) carried forward with behavioral content unchanged — only version-stamp decorations were removed. Each MUST retains its story + AC + Red Gate anchor: §D1 → AC-012/RG-024/T-14A; §G → AC-006/AC-007b/RG-025/T-11R/T-16; §I1 → AC-002/RG-003/RG-004/T-06/T-13; §I2 → AC-007a/T-15; §J2 → EC-010/T-21/RG-010 and AC-013/T-21 clause (c)/RG-027. All anchors verified present in story body. VERDICT: NO NEW UNANCHORED MUSTs.

**Correction of v1.42 false certification:** The v1.42 Amendment Sweep (Dimension-1 Sibling) stated "those stamps existed only in ROUTING-001 §Authority" and Dimension-2 stated "No downstream artifact copies these §Authority provenance-label loci verbatim" — both verdicts were scoped only to §Authority and were INCORRECT. The stamps permeated the entire active body across 22 loci. This v1.43 sweep issues the truthful whole-body discharge per the FB-62/63 dispatch.

---

### v1.42 Amendment Sweep (FB-58/60 records micro-burst: §Authority version-stamped provenance labels normalized to version-free form; ADR-058 status-date "(2026-08-18)"→"(2026-08-19)")

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): COERCION-001 amended in same burst (v1.38→v1.39) — ADR-058 §Authority status-date parenthetical corrected "(2026-08-18)"→"(2026-08-19)". Dimension-1 sibling sweep confirms COERCION-001 §Authority already uses clean version-free form for all provenance references (no §B2/§I2/§J2/§D1 sub-section version stamps — those stamps existed only in ROUTING-001 §Authority and are the source of the recurring drift class). COERCION-001 needs only the date fix.
VERDICT: SIBLING AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The changed surfaces are: (1) §Authority ADR-058 status-date parenthetical "(2026-08-18)"→"(2026-08-19)"; (2) §B2 reference: "v2.23 amendment: multi-valued array source fields" → "multi-valued array source fields"; (3) §D1 reference: "§D1 corrected v2.18:" → "§D1:"; (4) §G reference: "v2.23 Tier-1/Tier-2 model:" → "Tier-1/Tier-2 model:"; (5) §I1 reference (two-step form): "§I1 corrected v2.18: two-step form" → "§I1 two-step form"; (6) §I1 reference (canonical home): "§I1 corrected v2.21: canonical home" → "§I1 canonical home"; (7) §I2 reference: "§I2 v2.23 amendment:" → "§I2:"; (8) §J2 reference: "§J2 v2.23 amendment:" → "§J2:". No downstream artifact copies these §Authority provenance-label loci verbatim.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced. All existing MUST anchors (§D1, §G, §I1, §I2, §J2) carried forward — the behavioral obligations themselves are unchanged; only the version-stamp decorations are removed.
VERDICT: NO NEW UNANCHORED MUSTs.

---

### v1.41 Amendment Sweep (FB-55/56/57 LEG 2: Fix 4 §I1 provenance-label v2.23→v2.21; Fix 5 §Authority BC-2.16.003 modified-date "(modified 2026-08-19)")

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): COERCION-001 amended in same burst (v1.37→v1.38) — §Authority BC-2.16.003 pin corrected v1.17→v1.18 and modified-date "(modified 2026-08-18)"→"(modified 2026-08-19)". Both sibling §Authority BC-2.16.003 modified-date corrections applied in the same burst.
VERDICT: SIBLING AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The changed surfaces are: (1) §I1 origin-provenance label corrected v2.23→v2.21 (frozen at the ADR-058 version where the `ocsf_field_to_arrow_name` canonical-home correction actually landed per ADR §Changelog, per POL-39 preference for frozen origin labels); (2) §Authority BC-2.16.003 modified-date parenthetical "(modified 2026-08-18)"→"(modified 2026-08-19)". No downstream artifact copies these loci verbatim.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced. All existing MUST anchors carried forward unchanged.
VERDICT: NO NEW UNANCHORED MUSTs.

---

### v1.40 Amendment Sweep (FB-52/53/54 LEG 3: Fix 1 density 27/12→27/13; Fix 2+4 §Interpretation A v1.16→v1.18 (wrapped line); Fix 3 §Authority ocsf_field count per §J4/§Status; version re-pin ADR-058 v2.22→v2.23 + BC-2.16.003 v1.17→v1.18 active body)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): Pin bumps ADR-058 v2.22→v2.23 and BC-2.16.003
v1.17→v1.18 require sibling pin coordination; COERCION-001 §Authority + §Behavioral Contracts
table updated to v2.23/v1.18 in the same burst (v1.36→v1.37). Content unaffected in COERCION-001.
VERDICT: SIBLING PIN BUMP EXECUTED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The changed surfaces are: (1) density denominator 12→13 and ratio 2.25→2.08 (T-GATE); (2)
`§Interpretation A v1.16` wrapped-line instance → v1.18 (RG-025 Covers/Traces, combined Fix 2+4);
(3) §Authority parenthetical rewritten — `31 pre-correction per §J4 / 26 post-correction per
ADR-058 §Status` (Fix 3); (4) ADR-058 v2.22→v2.23 throughout active body (§Authority, §Mandate
Anchor, §Behavioral Contracts, RG-025..027, AC-002/006/007b/007c/013, Architecture Mapping,
Edge Cases, Tasks); (5) BC-2.16.003 v1.17→v1.18 throughout active body (§Authority, §Behavioral
Contracts, AC-006/007b, RG-025, Edge Cases, Tasks).
No downstream artifact copies these loci verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs added. All existing MUST anchors carried forward with updated version pins.
VERDICT: NO NEW UNANCHORED MUSTs.

---

### v1.39 Amendment Sweep (F-P51-MED-001 RG-026 orphaned: T-15 extended to name RG-026 as green target + source_path/ENRICH-1 normalization mandate; F-P49-MED-001 AC-007a build_column_array attribution removed; F-P49/51-MED-002 AC-013 added for §J2 synthesized-name guard; F-P49/51-BC-pin-sweep BC-2.16.003 v1.16→v1.17; F-P49-MED-003/F-P51-LOW-001 COERCION-001 §Authority+table pin v1.15/v1.16→v1.17)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P51-MED-001 / F-P49-MED-001 / F-P49/51-MED-002 are
ROUTING-001 scope only — RG-026, T-15, AC-007a, and AC-013 are specific to the
`pipeline_result_to_record_batch` raw_extensions aggregation path and the §J2 synthesized-name
fail-closed guard. COERCION-001 operates on `build_column_array` type-coercion and has no
raw_extensions aggregation path, no synthesized-name guard, and no T-15/AC-013 scope.
BC-2.16.003 pin v1.16→v1.17 requires sibling pin coordination; COERCION-001 is being amended
in this same burst (v1.35→v1.36, §Authority + §Behavioral Contracts pin sweep, content
unaffected). VERDICT: CONTENT UNAFFECTED IN COERCION-001; SIBLING PIN BUMP EXECUTED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The changed surfaces in this burst are: (1) input-hash aeafdff→90f6a36 (BC-2.16.003 updated
in Leg 1); (2) §Authority BC-2.16.003 version pin v1.16→v1.17 + EC-016-013-029 note added +
EC-016-013-028 reworded with `pipeline_result_to_record_batch` synthesis-locus attribution; (3)
§Behavioral Contracts BC-2.16.003 row v1.16→v1.17 + EC-016-013-029 and reworded EC-016-013-028
annotations; (4) §Mandate Anchor §J2 discharge narrative — v2.22 synthesized-name guard anchor
added (AC-013/T-21(c)/RG-027); (5) §Mandate Anchor table §J2 row — AC column updated from
`EC-010 (extended), T-21 clause (c)` to `AC-013, T-21 clause (c)` + BC-2.16.003 EC-016-013-029
added to MUST statement; (6) RG-027 Covers/Traces — updated from `§J2 guard` to `AC-013 +
BC-2.16.003 EC-016-013-029`; (7) BC-5.38.001 density check — AC count 12→13, density 2.25→2.08;
RG-027 coverage note updated to reference AC-013; (8) AC-007a — `build_column_array` attribution
paragraph removed; source_path extraction + ENRICH-1 normalization mandate added (reuse shared
pipeline, NOT naive `r.get(col.name)`); (9) AC-013 NEW — dedicated AC for §J2 synthesized-name
fail-closed guard (four reserved names, `Err(ArrowError::SchemaError)`, traces to
BC-2.16.003 EC-016-013-029 + ADR-058 v2.22 §J2); (10) §Architecture Mapping
`pipeline_result_to_record_batch §I2` row — `NOT build_column_array` scope note replaced with
source_path + ENRICH-1 normalization mandate; (11) §File Structure Requirements
spec_driven_adapter.rs row — `build_column_array raw_extensions path NOT added` note replaced
with source_path + ENRICH-1 normalization mandate; (12) T-15 — `NOT build_column_array /
cannot aggregate` misleading clause removed; source_path extraction + ENRICH-1 normalization
requirement added; RG-026 named as second green target alongside RG-008; (13) all
`§Interpretation A v1.16` instances in active text swept to `§Interpretation A v1.17` (covers
§Authority, §Mandate Anchor table, §Behavioral Contracts, AC-006, AC-007b, RG-025 Covers/Traces,
Edge Cases table, T-16).
No downstream artifact copies these loci verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

BC-2.16.003 EC-016-013-029 MUST (`pipeline_result_to_record_batch` MUST fail-closed
`Err(ArrowError::SchemaError)` when any `ocsf_field` flattens to a synthesized/reserved name):
anchored to `S-ADR058-OCSF-ROUTING-001 AC-013 / RG-027 / T-21 clause (c)`.
VERDICT: DISCHARGED IN THIS AMENDMENT.

BC-2.16.003 EC-016-013-028 reworded MUST (`pipeline_result_to_record_batch` raw_extensions
aggregation MUST apply source_path extraction + ENRICH-1 `Value::Array`→compact-JSON-list-string
normalization — NOT naive `r.get(col.name)`): anchored to `S-ADR058-OCSF-ROUTING-001 AC-007c /
AC-007a / RG-026 / T-15 (extended)`.
VERDICT: DISCHARGED IN THIS AMENDMENT.

---

### v1.38 Amendment Sweep (F-P46-MED-001 version-pin sweep AC-006 preamble/trace; F-P48-MED-001 EC-016-013-011 AC-011 trace; F-P48-MED-002 RG-026 + AC-007c EC-016-013-028; new RG-027 §J2 reserved-name guard; F-P48-LOW-001 RG-010 device-column refresh; F-P48-OBS-2 RG-025 nullable dual-condition; §Authority pins ADR v2.22 / BC-2.16.003 v1.16)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P46-MED-001/F-P48-MED-001/F-P48-MED-002 are
ROUTING-001 scope only — AC-006/AC-007/AC-011 and the new AC-007c/RG-026/RG-027 are specific to
the OCSF field-name routing implementation in `pipeline_result_to_record_batch` and `prism_describe`.
COERCION-001 operates on `build_column_array` type-coercion and the `SpecDrivenSensorAdapter`
coercion gate; it has no `raw_extensions` aggregation path, no Tier-1/Tier-2 ColumnDescriptor model,
and no ip_list compact-string serialization concern. ADR-058 pin v2.21→v2.22 and BC-2.16.003 pin
v1.15→v1.16 require sibling pin coordination; that pin bump to COERCION-001 is NOT silently edited
here — reported to state-manager for sibling-pin propagation (COERCION-001 v1.34→v1.35, content
unaffected). VERDICT: CONTENT UNAFFECTED; ADR/BC PIN BUMP REPORTED TO STATE-MANAGER.

**Dimension 2 — Downstream copy target:**

The changed surfaces in this burst are: (1) §Authority ADR-058 entry — v2.21→v2.22 with §B2/§I2/§J2
v2.22 amendment notes (EC-016-013-028 ip_list routing, §J2 reserved-name guard); (2) §Authority
BC-2.16.003 entry — v1.15→v1.16 with EC-016-013-028 and EC-016-013-011 notes; (3) §Behavioral
Contracts BC-2.16.003 row — v1.15→v1.16; (4) §Mandate Anchor table — §G row v2.21→v2.22, §I1 row
v2.21→v2.22, two new rows for RG-026 and RG-027; (5) RG-010 self-match exclusion — inline 5-column
list replaced with reference to BC-2.16.003 §Claroty Contracted OCSF Mappings devices table (20
columns per PR #236); (6) RG-025 intro line — v2.21/v1.15→v2.22/v1.16; (7) RG-025 assertion (iv)
— v2.21→v2.22; (8) RG-025 assertion (v) — v2.21/v1.15→v2.22/v1.16 with dual-condition rationale
(per-row null + per-table absence); (9) RG-025 Covers/Traces line — v2.21/v1.15→v2.22/v1.16; (10)
AC-006 preamble — v2.20/v1.14→v2.22/v1.16; (11) AC-006 trace — v1.14→v1.16, v2.20→v2.22; (12)
AC-007b header — v2.21/v1.15→v2.22/v1.16; (13) AC-007 traces — v1.15→v1.16, v2.21→v2.22; (14)
AC-007c NEW — EC-016-013-028 multi-valued array compact JSON-list string obligation; (15) AC-011
trace — added BC-2.16.003 EC-016-013-011 corrected runtime-WARN reference; (16) AC-012 — 14→16
new callers (add RG-026/027); (17) T-14A — (a) 14→16 RG callers, total 17→19; (18) BC-5.38.001
density check — 25→27 RGTs, 2.08→2.25, RG-026/027 coverage notes; (19) RG-026 NEW; (20) RG-027 NEW;
(21) T-11S NEW (write RG-026); (22) T-11T NEW (write RG-027); (23) T-21 extended with clause (c)
reserved-name guard; (24) T-GATE — 25→27, density 2.08→2.25, RG-026/027 in prism-bin distribution;
(25) T-19 — 25→27 RGTs, RG-026/027 in prism-bin distribution; (26) §Architecture Mapping
`prism_describe` row — v2.20→v2.22; (27) Architecture Compliance Rule 1 — v2.21→v2.22; (28)
Forbidden Dependencies prism-mcp bullet — v2.21→v2.22; (29) §File Structure Requirements
prism-bin row — RG-026/027 added; (30) Edge Cases table — EC-016-013-027 pins v2.20/v1.14→v2.22/v1.16,
new EC-016-013-028 row added; (31) all remaining v2.21/v1.15 pins in body text swept to v2.22/v1.16;
(32) T-13 canonical home — v2.21→v2.22; (33) T-16 col_type/nullable version refs — v2.21/v1.15→v2.22/v1.16.
No downstream artifact copies these loci verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

ADR-058 v2.22 §B2/§I2 MUST (ip_list with `ocsf_field == None` routes to `raw_extensions`; serialized
as compact JSON-list string NOT nested array): anchored to `S-ADR058-OCSF-ROUTING-001 AC-007c /
RG-026 / T-11S`. VERDICT: DISCHARGED.

ADR-058 v2.22 §J2 MUST (`pipeline_result_to_record_batch` MUST return `Err(ArrowError::SchemaError)`
when any `ocsf_field` flattens to reserved name `class_uid`, `category_uid`, `_sensor`, or
`raw_extensions`): anchored to `S-ADR058-OCSF-ROUTING-001 RG-027 / T-11T / T-21(c)`. VERDICT:
DISCHARGED.

BC-2.16.003 EC-016-013-011 (corrected: runtime WARN on `Err` branch, not load-time): anchored to
`S-ADR058-OCSF-ROUTING-001 AC-011 trace / RG-018`. VERDICT: DISCHARGED.

---

### v1.37 Amendment Sweep (F-P43-HIGH-001 `ocsf_field_to_arrow_name` relocated to `prism-spec-engine::column_mapping`; F-P43-MED-001 RG count 24→25 corrected; F-P44-OBS-001 RG-025 extended with col_type/nullable assertions; §Authority pins ADR v2.21 / BC-2.16.003 v1.15)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P43-HIGH-001 is ROUTING-001 scope only —
`ocsf_field_to_arrow_name` helper crate placement and the `prism-mcp → prism-bin` cycle
concern have no counterpart in COERCION-001, which operates on `build_column_array`
type-coercion and the `SpecDrivenSensorAdapter` coercion gate. Zero references to
`ocsf_field_to_arrow_name`, `column_mapping`, or `prism-spec-engine::column_mapping` in
COERCION-001's implementation scope. ADR-058 pin v2.20→v2.21 and BC-2.16.003 pin v1.14→v1.15
require sibling coordination; that pin bump to COERCION-001 is NOT silently edited here —
reported to state-manager for sibling-pin propagation. VERDICT: CONTENT UNAFFECTED;
ADR/BC PIN BUMP REPORTED TO STATE-MANAGER (NOT SILENTLY EDITED).

**Dimension 2 — Downstream copy target:**

The changed surfaces in this burst are: (1) §Authority ADR-058 entry — v2.20→v2.21 with
§I1 v2.21 crate-placement correction note and §G four-field `raw_extensions` shape;
(2) §Authority BC-2.16.003 entry — v1.14→v1.15 with EC-016-013-027 four-field shape;
(3) §Behavioral Contracts BC-2.16.003 row — v1.14→v1.15; (4) §Red Gate intro — "twenty-four"/"24"
corrected to "twenty-five"/"25"; (5) RG-003 — reworded to mandate `prism-spec-engine::column_mapping`
and explain the no-cycle import contract; (6) §Mandate Anchor table — last §G row updated with
four-field shape; new §I1 crate-placement MUST row added; (7) AC-002 — module reference updated;
(8) AC-007b — `col_type`/`nullable` fields added; version refs v2.20→v2.21, v1.14→v1.15; (9)
AC-007 traces updated; (10) §Architecture Mapping `ocsf_field_to_arrow_name` row — relocated to
`prism-spec-engine::column_mapping`; (11) §Architecture Compliance Rules Rule 1 — reworded to
mandate `prism-spec-engine::column_mapping`, explain cycle; (12) §Forbidden Dependencies — first
bullet fixed, new prism-mcp bullet added; (13) §File Structure Requirements — `column_mapping.rs`
row added, prism-bin row notes updated; (14) T-06/T-07 — file location note added;
(15) T-13 — target changed to `prism-spec-engine/src/column_mapping.rs` with import instructions;
(16) T-16 — four-field ColumnDescriptor shape added; version refs v2.20→v2.21, v1.14→v1.15;
(17) T-GATE/T-19 — RG distribution: `RG-001..004 in prism-spec-engine` (was `RG-001..002`);
`RG-005..006/008..010/014..022/024 in prism-bin` (was `RG-003..006/...`); (18) T-11R — five
assertions (i)-(v); version refs v2.21/v1.15; (19) RG-025 — extended with assertions (iv)
col_type=Json and (v) nullable=true; version refs v2.21/v1.15; (20) §BC-5.38.001 density note
for RG-025 — version refs updated. No downstream artifact copies any of these loci verbatim.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

ADR-058 §I1 v2.21 MUST (`ocsf_field_to_arrow_name` MUST live in `prism-spec-engine::column_mapping`;
both `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe` import it from there;
placing it in `prism-bin` is FORBIDDEN): anchored to `S-ADR058-OCSF-ROUTING-001 AC-002 / RG-003 /
RG-004 / T-13`. VERDICT: DISCHARGED.

ADR-058 §G v2.21 / BC-2.16.003 §Interpretation A v1.15 MUST (four-field `raw_extensions`
ColumnDescriptor: `col_type = prism_core::column::ColumnType::Json`, `nullable = true`): anchored
to `S-ADR058-OCSF-ROUTING-001 AC-006 (Tier-2) / AC-007b / RG-025 / T-11R / T-16`. VERDICT:
DISCHARGED.

---

### v1.36 Amendment Sweep (F-P40/P42-HIGH-001 Tier-1/Tier-2 prism_describe model propagated; RG-025 added; §Authority pins ADR v2.20 / BC-2.16.003 v1.14)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P40/P42-HIGH-001 is ROUTING-001 scope
only — `prism_describe` Tier-1/Tier-2 model and the `raw_extensions` ColumnDescriptor
emission have no counterpart in COERCION-001, which operates on `build_column_array`
type-coercion (`ColumnMapper::coerce_value`) and the `SpecDrivenSensorAdapter` coercion
gate. Zero references to `prism_describe`, `raw_extensions` ColumnDescriptor, Tier-1,
Tier-2, or EC-016-013-027 in COERCION-001's implementation scope. ADR-058 pin v2.19→v2.20
and BC-2.16.003 pin v1.13→v1.14 require sibling coordination; that pin bump to COERCION-001
is NOT silently edited here per constraint — reported to state-manager for sibling-pin
propagation. VERDICT: CONTENT UNAFFECTED; ADR/BC PIN BUMP REPORTED TO STATE-MANAGER
(NOT SILENTLY EDITED).

**Dimension 2 — Downstream copy target:**

The changed surfaces in this burst are: (1) §Authority ADR-058 entry — v2.19→v2.20 with
§G Tier-1/Tier-2 description added; (2) §Authority BC-2.16.003 entry — v1.13→v1.14, date
parenthetical 2026-08-17→2026-08-18, EC-016-013-027 reference added; (3) §Behavioral
Contracts BC-2.16.003 row — v1.13→v1.14 with EC-016-013-027 annotation; (4) §Mandate
Anchor table — new row for ADR-058 §G v2.20 / BC-2.16.003 EC-016-013-027 MUST; (5) RG-007
coverage note — "Covers AC-006" expanded to "Covers AC-006 Tier-1; Tier-2 covered by
RG-025"; (6) RG-025 added; (7) BC-5.38.001 density check — 24→25, 2.00→2.08, RG-025
coverage note appended; (8) AC-006 rewritten (Tier-1/Tier-2 model, Tier-2 prohibition);
(9) AC-007 expanded (AC-007a retained; AC-007b new: `prism_describe` `raw_extensions`
ColumnDescriptor obligation); (10) EC-016-013-027 added to edge cases table; (11)
§Architecture Mapping `prism_describe` row updated to Tier-1/Tier-2 description; (12) T-11R
added; (13) T-GATE updated 24→25, density 2.00→2.08, RG-025 in prism-mcp; (14) T-16
rewritten (Tier-1/Tier-2 implementation + makes RG-007 and RG-025 green); (15) T-19
updated 24→25, RG-025 in prism-mcp. No downstream artifact copies any of these loci
verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

ADR-058 §G v2.20 / BC-2.16.003 EC-016-013-027 MUST (`prism_describe` MUST NOT emit
individual ColumnDescriptor for `ocsf_field == None` columns; MUST emit exactly ONE
`raw_extensions` ColumnDescriptor enumerating source keys): anchored to
`S-ADR058-OCSF-ROUTING-001 AC-006 (Tier-2) / AC-007b / RG-025 / T-11R / T-16`.
VERDICT: DISCHARGED IN THIS AMENDMENT.

---

### v1.34 Amendment Sweep (F-P36-LOW-001 records-tier volatile-cite token in §Changelog v1.12 row neutralized)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P36-LOW-001 is ROUTING-001 §Changelog
record-tier only. Sweep of COERCION-001 §Changelog for volatile line-cite patterns
(`line ~NNN`, `lines NNN`, `file.rs:NNN`, `~L[0-9][0-9]`, bare `L[0-9][0-9][0-9]`): CLEAR
— zero instances found. VERDICT: SIBLING UNAFFECTED.

**Dimension 2 — Downstream copy target:**

The v1.12 changelog row is a records-tier audit entry. No downstream artifact copies
§Changelog rows verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. This is a records-tier hygiene fix with zero content/mechanism
change. VERDICT: N/A — no new mandates.

---

### v1.35 Amendment Sweep (F-P39-LOW-001 §Authority BC-2.16.002 date parenthetical corrected; comprehensive perimeter records-hygiene audit CLEAN)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P39-LOW-001 is a date parenthetical
in ROUTING-001 §Authority for BC-2.16.002 only. Comprehensive sweep of COERCION-001
§Authority for date parentheticals, version pins, and volatile line-cite tokens: CLEAR
— BC-2.16.003 "(modified 2026-08-17)" correct; ADR-058 "(2026-08-18)" correct; all
version pins accurate; zero volatile cite tokens found. VERDICT: SIBLING UNAFFECTED.

**Dimension 2 — Downstream copy target:**

The §Authority date parenthetical is a records-tier annotation. No downstream artifact
copies §Authority date parentheticals verbatim. Perimeter-wide sweep of ADR-058 v2.19:
zero "(modified YYYY-MM-DD)" body parentheticals found; all BC/ADR cites version-pin
only. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. This is a records-tier date-sync fix (POL-37/TD-VSDD-060) with
zero content/mechanism change. VERDICT: N/A — no new mandates.

---

### v1.33 Amendment Sweep (F-P34-MED-001 caller enumeration + F-P34-LOW-001 threading expression + ADR-058 pin v2.19)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P34-MED-001 (caller enumeration
correction) and F-P34-LOW-001 (threading expression `&self.sensor_spec.spec`) are
ROUTING-001 scope only — COERCION-001 has no `pipeline_result_to_record_batch` parameter
threading or related pre-existing test caller in its scope. ADR-058 pin v2.18→v2.19
requires a sibling pin bump in COERCION-001; that bump is NOT silently edited here per
constraint — reported to state-manager for sibling-pin propagation.
VERDICT: CONTENT UNAFFECTED; ADR PIN BUMP REPORTED TO STATE-MANAGER (NOT SILENTLY EDITED).

**Dimension 2 — Downstream copy target:**

Three loci carry the threading expression and were updated in this burst: (1) AC-003
parameter threading note — `&self.sensor_spec` → `&self.sensor_spec.spec` with type
clarification; (2) AC-012 production callers description — same correction; (3) T-14A
call-site description — same correction plus caller-count update (15→17 and two-part
(a)/(b) instruction). No downstream artifact copies these loci verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P34-MED-001 is a caller enumeration accuracy fix;
F-P34-LOW-001 is a threading-expression accuracy fix; ADR-058 pin is a records-tier
version update. VERDICT: N/A — no new mandates.

---

### v1.32 Amendment Sweep (F-P33-MED-001 signature gap: `pipeline_result_to_record_batch` gains `sensor_spec: &SensorSpec` parameter; ADR pin v2.18)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P33-MED-001 is ROUTING-001 scope only —
`pipeline_result_to_record_batch` parameter threading has no counterpart in COERCION-001,
which operates on `build_column_array` and `ColumnMapper::coerce_value`. Zero references
to `pipeline_result_to_record_batch`, `sensor_spec`, or `ocsf_column_naming` in
COERCION-001's implementation scope. COERCION-001 DOES need an ADR-058 pin bump
(v2.17→v2.18) for version-tracking consistency; that pin bump is NOT silently edited here
per user constraint — reported to orchestrator for routing. VERDICT: CONTENT UNAFFECTED;
ADR PIN BUMP REPORTED TO ORCHESTRATOR (NOT SILENTLY EDITED).

**Dimension 2 — Downstream copy target:**

Five loci carry the `sensor_spec` reference and were updated in this burst to clarify it
as a threaded parameter (not a free variable): (1) §Authority entry — ADR-058 §D1/§I1
notes added; (2) AC-003 — parameter threading note added before the code snippet;
(3) §Architecture Mapping — `pipeline_result_to_record_batch` row updated to note the new
parameter; (4) §Mandate Anchor table — new §D1 MUST row added; (5) §Architecture
Compliance Rules — Rule 11 added. All five loci now consistently describe `sensor_spec` as
the threaded parameter per ADR-058 §D1 v2.18. No downstream artifact copies these loci
verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

ADR-058 §D1 v2.18 MUST (`pipeline_result_to_record_batch` MUST gain `SensorSpec` as an
explicit parameter threaded from `fetch()`): anchored to `S-ADR058-OCSF-ROUTING-001
AC-012 / RG-024 / T-14A`. VERDICT: DISCHARGED IN THIS AMENDMENT.

---

### v1.31 Amendment Sweep (F-P32-MED-001 raw_extensions synthesis re-attributed to `pipeline_result_to_record_batch`; AC-003 reconciled to ADR-058 §I1+§I2; ADR pin v2.17)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P32-MED-001 is ROUTING-001 scope only — `pipeline_result_to_record_batch` raw_extensions aggregation has no counterpart in COERCION-001. ADR-058 pin v2.16→v2.17 swept to COERCION-001 in same burst (v1.29→v1.30). VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

AC-003 (§I1+§I2 reconciliation — `unwrap_or_else` fallback scoped to `ocsf_field == Some`), AC-007 (raw_extensions synthesis locus re-attributed to `pipeline_result_to_record_batch`), T-15 (task attribution), §Architecture Mapping (`build_column_array raw_extensions handling` row re-attributed to `pipeline_result_to_record_batch §I2`), §Purity Classification (stale `build_column_array (raw_extensions path)` row removed; `pipeline_result_to_record_batch` row updated to note §I2 aggregation), §File Structure Requirements (`spec_driven_adapter.rs` row — "update `build_column_array` `raw_extensions` path" replaced with ADR-058 §I2 attribution): six loci re-attributed. ADR-058 §Authority entry updated v2.16→v2.17. No downstream artifact copies these loci verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. Re-attribution corrects synthesis-locus description; no behavioral obligation changes. VERDICT: N/A — no new mandates.

---

### v1.30 Amendment Sweep (F-P31-LOW-001 T-11P RG-023 location reworded — records-only micro-burst TD-VSDD-096)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): COERCION-001 has no T-11P or class_selector tasks — no equivalent wording exists to sweep. VERDICT: NO SIBLING IMPACT; COERCION-001 UNAFFECTED.

**Dimension 2 — Downstream copy target:**

T-11P wording is not copied verbatim into any downstream artifact. Verification: §File Structure Requirements (`crates/prism-ocsf/tests/` row, RG-011/012/023), §T-GATE ("RG-011/012/023 in prism-ocsf/tests/"), and T-19 ("RG-011/012/023 in prism-ocsf/tests/") are the three authoritative loci; T-11P now agrees with all three. VERDICT: ALL FOUR LOCI AGREE — RG-023 → `crates/prism-ocsf/tests/`.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. Location-wording correction only. VERDICT: N/A.

---

### v1.29 Amendment Sweep (BC-2.16.003 pin v1.12→v1.13 — OCSF-correctness Claroty SPEC pass-30 sibling coordination)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): BC-2.16.003 pin v1.12→v1.13 swept to both stories in same burst. COERCION-001 amended in same burst (v1.28→v1.29). VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

BC-2.16.003 §OCSF Field Validation (v1.13) adds Path-A/Path-B qualifier: vendor-extended paths (`device.type_label` / `device.type_category`) produce first-class Arrow columns on Path A (Interpretation A, NOT raw_extensions). Downstream contradiction check: AC-005 §Claroty Contracted OCSF Mappings table lists `device_type_label` and `device_type_category` as first-class Arrow columns under Interpretation A; AC-010 / RG-022 asserts `device_type_label` at wire level (serialized JSON). No prose in this story repeats the old unqualified claim. VERDICT: NO DOWNSTREAM CONTRADICTION; no correction needed.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. BC-2.16.003 pin is a version-tracking update (PO bump — §OCSF Field Validation Path-A/Path-B qualifier). VERDICT: N/A — no new mandates.

---

### v1.28 Amendment Sweep (BC-2.16.002 pin v2.27→v2.28 — OCSF-correctness Claroty SPEC pass-28 sibling coordination)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): BC-2.16.002 pin v2.27→v2.28 swept to both stories in same burst. COERCION-001 amended in same burst (v1.27→v1.28 — BC-2.16.002 pin v2.27→v2.28 sibling coordination). VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

BC-2.16.002 §Authority entry and §Behavioral Contracts body table are the sole live BC-2.16.002 pin sites in this story. Both updated v2.27→v2.28. No downstream artifact copies these verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. BC-2.16.002 pin update is a records-tier version-tracking update (PO bump — §Canonical Structured Event Catalog ocsf.unknown_class_name row gains pending-wiring annotation in v2.28). VERDICT: N/A — no new mandates.

---

### v1.27 Amendment Sweep (ADR-058 pin v2.15→v2.16 sibling coordination — OCSF-correctness Claroty SPEC pass-26 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P26-MED-001 (RG-006 extension null+warn) is COERCION-001 scope only — ROUTING-001 has no `build_column_array` String arm or null-cell test to extend. ADR-058 pin v2.15→v2.16 swept to both stories in same burst. VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §Authority entry is the sole live ADR pin site in this story. Updated v2.15→v2.16. No other live current-state site in ROUTING-001 carries the v2.15 pin. No downstream artifact copies this entry verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. This is a records-tier ADR pin update. VERDICT: N/A — no new mandates.

---

### v1.26 Amendment Sweep (ADR-058 pin v2.15 + BC-2.16.003 pin v1.12 sibling coordination — OCSF-correctness Claroty SPEC pass-25 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P25-MED-001 is COERCION-001 scope only — AC-005/T-15 add-Object-retain-wildcard has no counterpart in ROUTING-001. ADR-058 pin v2.14→v2.15 and BC-2.16.003 pin v1.11→v1.12 swept to both stories in same burst. VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §Authority entry and BC-2.16.003 §Authority entry and §Behavioral Contracts body table are the sole live pin sites in this story. All three updated. No downstream artifact copies them verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. Pin sweeps are records-tier version-tracking updates. VERDICT: N/A — no new mandates.

---

### v1.25 Amendment Sweep (BC-2.16.003 v1.10→v1.11 sibling coordination — OCSF-correctness Claroty SPEC pass-24 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P24-HIGH-001 (Path-A Array-arm preserved, RG-007 retired) and F-P24-MED-001 (coerce_value signature correction) are COERCION-001 scope only — ROUTING-001 has no `build_column_array` String arm or `coerce_value` signature. BC-2.16.003 pin v1.10→v1.11 applied to ROUTING-001 in same burst. VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

BC-2.16.003 §Authority entry and §Behavioral Contracts body table are the two live BC-2.16.003 pin sites in this story. Both updated v1.10→v1.11. No other live copy of this pin exists in ROUTING-001. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. This is a BC version-pin update only. VERDICT: N/A — no new mandates.

---

### v1.24 Amendment Sweep (F-P23-MED-001 §Library & Framework + §File Structure Cargo.toml Notes text sync — OCSF-correctness Claroty SPEC pass-23 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P23-MED-001 is a same-defect-class fix applied symmetrically to both stories — COERCION-001 §Library & Framework Requirements tracing-test row for RG-009 also carried `prism-bin/tests/` text from the same stale origin (provisioned together in pass-19/pass-20 before pass-21 relocated only §Architecture Mapping, §T-GATE, and §File Structure prism-bin rows but not §Library & Framework). COERCION-001 amended in same burst (v1.22→v1.23). VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

All test-location-bearing surfaces in this story checked for `prism-bin/tests/` references to private-fn RGs:

- §Library & Framework Requirements tracing-test row (RG-018 location): corrected from `prism-bin/tests/` to `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`.
- §File Structure Requirements Cargo.toml row Notes cell: corrected from `prism-bin/tests/` to `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`.
- §Architecture Mapping prism-bin row: references `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests` — confirmed CURRENT (updated v1.22 per F-P21-MED-001).
- §File Structure Requirements prism-bin unit-test row: references `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests` — confirmed CURRENT (updated v1.22 per F-P21-MED-001).
- §File Structure Requirements note below table: "do NOT place them in `crates/prism-bin/tests/`" — this is a prohibition directive (correct); NOT a location directive for private-fn RGs; retained unchanged.
- §File Structure Requirements e2e test row: `crates/prism-bin/tests/` (e2e test — TBD at dispatch) — this is the PUBLIC-surface `#[ignore]`'d e2e test for AC-008; legitimately lives in `prism-bin/tests/`; NOT changed.
- §T-GATE: references `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests` — confirmed CURRENT (updated v1.22 per F-P21-MED-001).
- §Library prose paragraph below the table: does not contain `prism-bin/tests/` — CURRENT.
- §Purity Classification / §Density: no `prism-bin/tests/` references for private-fn RGs — CURRENT.
- §Red Gate Tests RG-018 text: references `tracing_test` subscriber without naming a file path — CURRENT.
- §Tasks T-19/T-GATE task text: no `prism-bin/tests/` reference for private-fn RGs — CURRENT.

Post-edit grep: ZERO `prism-bin/tests/` references for private-fn RGs remain in this story. VERDICT: COMPLETE; all location-bearing surfaces verified.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P23-MED-001 is a records-tier text-sync correction. VERDICT: N/A — no new mandates.

---

### v1.23 Amendment Sweep (F-P22-MED-001 RG-013 prism-ocsf routing + F-P22-MED-002 T-17/T-22 verify-command crate-coverage + F-P22-OBS-2 T-21 verify command + F-P22-OBS-3 T-22(c) doc-table count + ADR-058 pin sweep — OCSF-correctness Claroty SPEC pass-22 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P22-MED-001 is ROUTING-001 scope only — COERCION-001 has no prism-ocsf private-fn routing concern (no `set_nested_field` call in Stage 1). F-P22-MED-002, F-P22-OBS-2, F-P22-OBS-3 are ROUTING-001 scope only — COERCION-001 has no TOML-driven wire-shape RGs, no class_uid wire-shape RGs, and no T-21/T-22 collision/shadow tasks. ADR-058 §Authority pin v2.13→v2.14 swept to COERCION-001 in same burst (v1.21→v1.22). VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

§File Structure Requirements (prism-ocsf rows) and §Tasks (T-17/T-21/T-22 verify commands) are the authoritative dispatch locations for the test-writer and implementer. T-GATE and T-19 carry per-crate RG distribution summaries derived from §File Structure. All four sites updated consistently. No downstream artifact copies these rows verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P22-MED-001 is a test-location correctness fix; F-P22-MED-002, F-P22-OBS-2, F-P22-OBS-3 are verify-command accuracy and authoring-accuracy corrections; ADR-058 pin is a records-tier version update. VERDICT: N/A — no new mandates.

---

### v1.22 Amendment Sweep (F-P21-MED-001 prism-bin RG relocation to src mod tests + F-P21-LOW-001 T-12 attribution — OCSF-correctness Claroty SPEC pass-21 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P21-MED-001 applies to BOTH stories —
COERCION-001 §Architecture Mapping, §T-GATE, and §File Structure Requirements rows for
prism-bin RG-006..RG-009 (`build_column_array`) relocated to `src/spec_driven_adapter.rs`
`#[cfg(test)] mod tests`. COERCION-001 amended in same burst (v1.20→v1.21). F-P21-LOW-001
is ROUTING-001 scope only (T-12/T-14 attribution). VERDICT: SWEPT; COERCION-001 AMENDED
IN SAME BURST.

**Dimension 2 — Downstream copy target:**

§File Structure Requirements (prism-bin unit-test row) is the authoritative dispatch
location for the test-writer. No downstream artifact copies this row verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P21-MED-001 is a test-location correctness fix; F-P21-LOW-001
is a gate-attribution accuracy correction. VERDICT: N/A — no new mandates.

---

### v1.21 Amendment Sweep (F-P20-MED-002 tracing-test dependency-aware + F-P20-LOW-001a sweep reorder + F-P20-OBS-001/002 RG attribution — pass-20)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P20-LOW-001b corrects a false sentence in
COERCION-001 §v1.18 Amendment Sweep §Dimension 1 (which falsely claimed ROUTING-001 sweep
ordering was already correct); COERCION-001 amended in same burst (v1.19→v1.20). F-P20-LOW-002
adds BC-2.02.011 Token Budget row to COERCION-001 (§Token Budget count parity). F-P20-MED-002,
F-P20-LOW-001a, F-P20-OBS-001, and F-P20-OBS-002 are ROUTING-001 scope only — COERCION-001 has
no equivalent RG-006 green-driver issue, no sweep ordering defect, and no tracing-test dependency
confusion. VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

§Library & Framework Requirements (tracing-test row) and §File Structure Requirements
(prism-bin/Cargo.toml row) are the authoritative provisioning instructions for the
implementer. The dependency-aware wording replaces the unconditional "NOT yet present — MUST
add" instruction with "VERIFY present; S-ADR058-OCSF-COERCION-001 is the upstream provider."
No downstream artifact copies these tables verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P20-MED-002, F-P20-LOW-001a, F-P20-OBS-001, F-P20-OBS-002 are
accuracy/ordering/attribution corrections. VERDICT: N/A — no new mandates.

---

### v1.20 Amendment Sweep (F-P19-MED-001 prism-bin tracing-test provisioning — OCSF-correctness Claroty SPEC pass-19 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P19-MED-001 applies to BOTH stories —
COERCION-001 gains the same `prism-bin/Cargo.toml` provisioning row plus an expanded
§Architecture Mapping Constraints item 3 naming both Cargo.toml files; it also receives
F-P19-LOW-001 (T-12 nextest filter false-green fix), which has no counterpart in this story
(ROUTING-001 has no T-12 nextest filter). COERCION-001 amended in same burst (v1.18→v1.19).
VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

§Library & Framework Requirements (tracing-test row) and §File Structure Requirements
(prism-bin/Cargo.toml row) are the authoritative provisioning instructions for the
implementer. No downstream artifact copies these tables verbatim. The "No new crate
additions" note has been replaced with an accurate statement that one dev-dependency
addition is required. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The prism-bin tracing-test provisioning is a
dev-infrastructure correctness fix enabling RG-018 to compile and run in `prism-bin/tests/`.
VERDICT: N/A — no new mandates.

---

### v1.19 Amendment Sweep (F-P18-MED-002 §File Structure RG-011..023 gaps + prism-mcp row + F-P18-MED-003 RG-013 falsifiability fix + F-P18-MED-004 RG-007 prism-mcp routing + F-P18-OBS-002 device_type_label confirmation — OCSF-correctness Claroty SPEC pass-18 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): F-P18-MED-002, F-P18-MED-003, and F-P18-MED-004
are ROUTING-001 scope only — §File Structure Requirements, RG-013 mechanism, and T-GATE/T-19
prism-mcp command have no counterpart in COERCION-001 (which has only two crates: prism-spec-engine
and prism-bin). F-P18-OBS-002 confirms the settled `device_type_label` value is already used in
ROUTING-001 §AC-005 / §Claroty Contracted OCSF Mappings; COERCION-001 has no occurrence of
`device_type_name` or `device_type_label` in scope for this finding class. COERCION-001 receives
separate fixes this burst (F-P18-MED-001, F-P18-OBS-001) with its own sweep at v1.18.
VERDICT: SWEPT; COERCION-001 UNAFFECTED BY ROUTING-001-SPECIFIC FINDINGS.

**Dimension 2 — Downstream copy target:**

F-P18-MED-002: §File Structure Requirements table is the authoritative dispatch instruction for the
test-writer. The RG→crate routing now consistently reflected across §Red Gate Tests, §File Structure
Requirements, T-GATE, and T-19: prism-spec-engine (RG-001..002), prism-bin
(RG-003..006/008..010/014..022), prism-mcp (RG-007), prism-ocsf (RG-011..013/023). No downstream
artifact copies the §File Structure table verbatim. VERDICT: CLEAR.

F-P18-MED-003: RG-013 §Red Gate Tests description is the authoritative test specification for the
test-writer. The rewrite aligns §Red Gate Tests RG-013 to T-11F (DynamicMessage / set_nested_field
mechanism, 3004-vs-3001 contrast). No downstream artifact copies the RG-013 description verbatim.
VERDICT: CLEAR.

F-P18-MED-004: T-GATE and T-19 are the authoritative gate-command specifications. Adding
`just iter prism-mcp --no-fail-fast` to T-GATE and `just iter prism-mcp` to T-19 ensures RG-007
(prism-mcp) is observed in the RED gate and the final green gate. No downstream artifact copies
these task-plan sections. VERDICT: CLEAR.

F-P18-OBS-002: Confirmed no live current-state occurrence of stale `device_type_name` in AC-005
or §Claroty Contracted OCSF Mappings; the settled `device_type_label` is used throughout. Historical
v1.2 Amendment Sweep snapshot reference is grandfathered by TD-VSDD-091 ratchet scoping (pre-existing
unchanged line). No edit required. VERDICT: NO-CHANGE CONFIRMED.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. All four findings are task-plan coherence and
test-specification accuracy fixes. F-P18-OBS-002 is a no-change confirmation.
VERDICT: N/A — no new mandates.

---

### v1.18 Amendment Sweep (F-P16-MED-001 BC-2.01.013 title corrected + F-P16-OBS-001 BC-2.16.003/002 title expansion — OCSF-correctness Claroty adversary SPEC pass-16 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): COERCION-001 §Authority also carried an abbreviated BC-2.16.003 title ("Column-to-OCSF Mapping at Query Time" without the post-em-dash enrichment). Both stories amended in the same burst (COERCION-001 v1.15→v1.16): BC-2.16.003 title expanded to full H1 verbatim; BC-2.16.003 pin updated v1.9→v1.10; F-P16-MED-003 AC-007/RG-008/009 added. BC-2.01.013 is absent from COERCION-001's `behavioral_contracts` frontmatter — no sibling correction needed for F-P16-MED-001. BC-2.16.002 is already at full H1 title in COERCION-001 §Authority ("Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation") — no correction needed there. VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The §Authority BC entries are the authoritative title references in this story; the §Behavioral Contracts body table uses IDs and versions, not titles — no title propagation to that table is needed. The BC-2.01.013 wrong title ("DataSource Trait Adapter Pattern") was a historical authoring error, not a downstream copy of another artifact. BC-2.16.003 was bumped to v1.10 by the product-owner in this same pass-16 burst (EC-016-013-025 addition). BC-2.16.003 pin propagated v1.9→v1.10 at both live current-state sites in ROUTING-001: §Authority entry (`Version \`1.10\``) and §Behavioral Contracts body table (`v1.10`). Historical §Changelog rows are untouched per convention. VERDICT: SWEPT.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The title corrections and title expansions are authoring-accuracy fixes, not new behavioral obligations. VERDICT: N/A — no new mandates.

---

### v1.17 Amendment Sweep (F1 [MED, records-tier] BC-2.01.013 stale version pin v1.16→v1.23 — OCSF-correctness Claroty adversary SPEC pass-15 micro-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): COERCION-001 carries no BC-2.01.013 version pin in
either its §Authority section or its body BC table — BC-2.01.013 is not in COERCION-001's
`behavioral_contracts` frontmatter. No sibling update required. VERDICT: CLEAR.

**Dimension 2 — Downstream copy target:**

The BC-2.01.013 §Authority entry and body BC table row are the two live version-pin sites. Both
corrected in this burst (v1.16→v1.23). The historical §Changelog v1.0 row records "BC-2.01.013
v1.16 ... at authoring time" — that is the authoring-time snapshot, grandfathered per the
records-tier exception; it was NOT updated. No downstream artifact copies the §Authority
BC-2.01.013 pin. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. This is a records-only stale-pin correction. VERDICT: N/A — no new mandates.

---

### v1.16 Amendment Sweep (F2 T-11G/H/L/M/N/O task-wording fix + ADR-058 re-pin v2.12→v2.13)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): ADR-058 §Authority pin updated v2.12→v2.13 in
same burst. COERCION-001 has no §Tasks T-11* authoring-wording issue (it does not touch the TOML
— all its tasks are code-level with consistent inline-spec/code-RED patterns). VERDICT: COERCION-001
AMENDED IN SAME BURST (ADR pin only).

**Dimension 2 — Downstream copy target:**

T-17 "Makes green" list (RG-014/015/019/020/021/022) is the downstream copy of the RG attributions
in T-11G/H/L/M/N/O. The T-17 green list is unchanged — T-17 still claims those same 6 RGs. The
fix is authoring-instruction alignment: the tests now specify loading the real claroty.sensor.toml
(post-T-17), which is consistent with T-17's green-driver role. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. Task wording and ADR re-pin are consistency corrections.
VERDICT: N/A — no new mandates.

---

### v1.15 Amendment Sweep (ADR-058 re-pin v2.11→v2.12 + sibling sweep COERCION-001 LOW-2 coordination)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): LOW-2 AC-004 trace parentheticals added — BC-2.16.002
and BC-2.02.011 formal `(traces to …)` parentheticals now present on AC-004; ADR-058 §Authority pin
v2.11→v2.12; §v1.14 Amendment Sweep added in same burst. ROUTING-001 has no equivalent LOW-2 fix —
all three of its frontmatter BCs already had AC trace parentheticals. VERDICT: COERCION-001 AMENDED
IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §Authority pin is the only changed site in ROUTING-001 this burst (v2.11→v2.12). Sibling
sweep of both stories confirmed zero additional normative prose ADR-058/BC version pins.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. VERDICT: N/A — no new mandates.

---

### v1.14 Amendment Sweep (F3 §Mandate Anchor #1 provenance fix + ADR-058 re-pin v2.10→v2.11)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): ADR-058 §Authority pin updated v2.10→v2.11 in
same burst. COERCION-001 §Mandate Anchor #2 was cleaned in pass-7 and carries no version qualifiers
(DISCHARGED — ADR-058 §H carries the inline mark). The F3 fix brings §Mandate Anchor #1 to the
same version-free form. VERDICT: COERCION-001 AMENDED IN SAME BURST; CLEAN.

**Dimension 2 — Downstream copy target:**

§Mandate Anchor #1 is consumed by the implementer to verify discharge status. The change from
"DISCHARGED (v2.1)" / "anchored since v2.1" to "DISCHARGED — ADR-058 §D2/§J2 carries the inline
(...) mark" removes the drift source entirely: the inline mark in ADR-058 §D2/§J2 is now the sole
provenance reference, so the story cannot become stale when ADR version advances again. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. F3 fix and ADR re-pin are accuracy/hygiene corrections.
VERDICT: N/A — no new mandates.

---

### v1.13 Amendment Sweep (F2 AC-011 POL-39 prose pin removal + ADR-058 re-pin v2.9→v2.10)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): ADR-058 §Authority pin updated v2.9→v2.10 in
same burst. Sibling sweep of COERCION-001 normative prose (AC bodies, EC bodies, §Tasks,
MUST-Discharge sections): zero additional POL-39 doc-version pins found outside §Authority (exception),
body BC table (exception), and historical amendment-sweep/changelog sections (grandfathered by
TD-VSDD-091 ratchet scoping). VERDICT: COERCION-001 AMENDED IN SAME BURST; CLEAN.

**Dimension 2 — Downstream copy target:**

AC-011 §Catalog obligation prose is the source from which the implementer reads the obligation.
The F2 fix removes the stale "v2.27 (product-owner authored it in this fix-burst)" clause and
replaces it with the already-present section anchor "BC-2.16.002 §Canonical Structured Event
Catalog". The BC-2.16.002 §Canonical Structured Event Catalog section itself is unchanged; only
this story's normative prose reference is cleaned. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The F2 fix and ADR re-pin are accuracy/hygiene
corrections, not new behavioral obligations. VERDICT: N/A — no new mandates.

---

### v1.12 Amendment Sweep (comprehensive hygiene: F1 mandate-anchor, F4 §J4 count, F3 line-cite, ADR-058 re-pin)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): ADR-058 §Authority pin updated v2.8→v2.9 in
same burst. No §Mandate Anchor equivalents needed in COERCION-001 (its §Mandate Anchor #2 was
cleaned in pass-7). Comprehensive hygiene sweep of COERCION-001 found zero POL-39 narrative
prose violations, zero line-cites, and zero mandate-anchor accuracy issues outside historical
amendment-sweep sections (grandfathered by TD-VSDD-091 ratchet scoping). VERDICT: COERCION-001
AMENDED IN SAME BURST; CLEAN.

**Dimension 2 — Downstream copy target:**

§Authority ADR-058 pin is the sole live ADR pin site in this story (ADRs are not pinned in the
body BC table — that table covers BCs only). Updated v2.8→v2.9. The §Mandate Anchor #1 rewrite
(F1) removes all ADR-058 version pins from that normative prose section; it was the only
non-§Authority, non-changelog section carrying volatile ADR-058 version references. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

F1 §Mandate Anchor #1 rewrite marks both §D2 and §J2 mandates DISCHARGED with no new obligations.
F4 §Authority §J4 count correction (31 pre-correction / 26 post-correction) is an accuracy fix,
not a new mandate. F3 line-cite removal from the v1.9 changelog row addresses a TD-VSDD-091
violation, not a behavioral obligation. VERDICT: N/A — no new mandates.

---

### v1.11 Amendment Sweep (full task-plan audit — gate ordering + green-driver attribution)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): Task-plan cross-audit results — CLEAN. All
seven RGs (RG-001..RG-007) map to exactly one implementation task each; terminal gates (T-16
run-all-7-RGTs, T-17 prism-bin iter, T-18 just check) are in correct order after the last
implementation task (T-15). No F1/F2 equivalents exist in COERCION-001. VERDICT: SWEPT; CLEAN.

COERCION-001 F3 discharge fix applied in same burst (v1.9→v1.10): ADR-058 §H MUST Discharge
section updated from pending-architect-routing to DISCHARGED; volatile `v2.0` pins removed
from normative prose per POL-39. VERDICT: COERCION-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

T-17's "Makes green" list is the downstream copy target for the RG-attributions in the
RG catalog (§Red Gate Tests). After this fix, T-17 correctly attributes RG-014, RG-015,
RG-019, RG-020, RG-021, RG-022 — all six TOML-driven wire-shape tests. T-23 now correctly
attributes RG-012 AND RG-023 — both claroty and armis select() arm tests. No other downstream
copies of these lists exist. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The gate-reordering and attribution fixes are
task-plan-fidelity corrections, not new behavioral obligations. VERDICT: N/A — no new mandates.

---

### v1.10 Amendment Sweep (T-11H 3-field fix + T-11P API fix + F3 date cites)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): sibling-sweep for `ClassSelectorInput` /
`vendor:` / `class_name:` fabricated API shapes — **zero occurrences found** in COERCION-001.
F3 §Authority date cites updated to 2026-08-17 and `modified:` frontmatter field added in
same burst. No T-11H or T-11P equivalents exist in COERCION-001 (it has no class_selector
tasks). VERDICT: COERCION-001 DATE FIXES APPLIED IN SAME BURST; NO T-11H/T-11P EQUIVALENTS.

**Dimension 2 — Downstream copy target:**

T-11H test name is the source from which the RG-015 catalog entry and AC-010 assertion 1 derive.
The RG-015 catalog entry and AC-010 assertion 1 already carried the correct 3-field name
`test_claroty_alerts_finding_info_fields_wire_shape` since pass-4 — T-11H was the only stale
copy. After this fix, T-11H, the RG catalog, and AC-010 assertion 1 are all consistent. VERDICT:
CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The T-11H expansion and T-11P API correction are
test-authoring-accuracy fixes, not new behavioral obligations. VERDICT: N/A — no new mandates.

---

### v1.9 Amendment Sweep (ADR-058 v2.7→v2.8 + BC-2.16.003 v1.8→v1.9 pin sweep + F2 count fix)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): §Authority ADR-058 pin updated v2.7→v2.8;
§Authority BC-2.16.003 pin updated v1.8→v1.9; body BC table pin updated v1.8→v1.9. Same
pass-5 burst. F2 stale count fix applies only to ROUTING-001 (COERCION-001 has no Red-then-green
gate instruction with RG count). VERDICT: COERCION-001 PIN SWEEP APPLIED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §Authority entry and BC-2.16.003 §Authority entry carry the version pins; body BC
table duplicates the BC-2.16.003 pin. All three updated. No other live downstream copy of
these pins exists in this story. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The pin sweep and count fix are authoring-accuracy
updates, not new behavioral obligations. VERDICT: N/A — no new mandates.

---

### v1.8 Amendment Sweep (comprehensive KF→AC→RG coverage-matrix audit)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): BC-2.16.003 pin in COERCION-001 updated from
v1.7 → v1.8 in the same burst (v1.6→v1.7). VERDICT:
COERCION-001 SWEPT; BC PIN UPDATED.

**Dimension 2 — Downstream copy target:**

BC-2.16.003 §Claroty Contracted OCSF Mappings (v1.8) is the authoritative source for all 12
KF corrections. The AC-005 TOML obligations table in this story is derived from the BC; no
stale copy-source section remains in this story. AC-010 assertions now enumerate all 6
wire-shape assertions matching KF-03/04/05/06/07/12 corrections in the BC. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

New RGs introduced by this amendment:

- RG-021 (KF-05 `audit_logs.id` → raw_extensions): anchored to
  `S-ADR058-OCSF-ROUTING-001 AC-010 RG-021 T-11N`.
- RG-022 (KF-06 `devices.device_type` → `device_type_label`): anchored to
  `S-ADR058-OCSF-ROUTING-001 AC-010 RG-022 T-11O`.
- RG-023 (AC-009(c) Claroty `select()` arm): anchored to
  `S-ADR058-OCSF-ROUTING-001 AC-009 RG-023 T-11P`.

No unanchored MUSTs introduced by this amendment. VERDICT: DISCHARGED IN THIS AMENDMENT.

---

### v1.7 Amendment Sweep (F2 compile-error fix + F1 subsystem cross-check)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (Stage 1 sibling): swept for any `table.name`
field references in code snippets. COERCION-001 has no tracing emission snippet for
`ocsf.unknown_class_name` (that obligation belongs to Stage 2 only). Zero `table.name`
occurrences found in COERCION-001. VERDICT: SWEPT; CLEAR.

COERCION-001 subsystem mis-anchoring (F1) is addressed in the same burst (v1.5→v1.6):
`prism-bin` moved from SS-01 to SS-10; SS-10 added to subsystems frontmatter. VERDICT:
COERCION-001 AMENDED IN SAME BURST.

*RG-018 test name*: `test_pipeline_result_to_record_batch_unknown_ocsf_class_emits_warn` —
uses `tracing_test` subscriber asserting `event_type = "ocsf.unknown_class_name"`.
The `%table.table_name` field correction propagates to the test assertion (the test
validates the field schema, including `table_name`). The test-writer will use the corrected
field name from AC-011 when authoring RG-018. VERDICT: CAPTURED IN AC-011 (corrected).

**Dimension 2 — Downstream copy target:**

The AC-011 emission snippet is the source from which T-24 (implementer task) and the
BC-2.16.002 catalog row 94 field schema derive their `table_name` field. The BC-2.16.002
catalog row 94 already correctly lists `table_name` as a field (product-owner authored it
against the real `TableSpec` struct, not against this story's stale snippet). Only this
story's AC-011 and T-24 snippets carried the stale `%table.name` — both corrected in
this amendment. No downstream copy of the stale snippet exists. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The corrected `%table.table_name` field cite
is a compile-correctness fix, not a new obligation. VERDICT: N/A — no new mandates.

---

### v1.6 Amendment Sweep (F2 pin sweep + F3 wire-shape coverage + F4 SS attribution)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (sibling story, same epic): swept for stale BC-2.16.003 version
pins (v1.5→v1.7) and ADR-058 version pins (v2.6→v2.7). F5/F6 findings apply to COERCION-001
(missing BC-2.16.002 in frontmatter; stale §Catalog Row Obligation version prose). COERCION-001
is amended in the same fix-burst. VERDICT: SWEPT; COERCION-001 AMENDED IN SAME BURST.

*SS-10 subsystem justification prose*: F4 moved `prism-bin::spec_driven_adapter` attribution
from SS-01 to SS-10 — the SS-10 justification prose in this story was updated accordingly.
SS-01 justification retained for `prism-sensors/specs/claroty.sensor.toml` and
`prism-spec-engine`. No downstream copy of the SS attribution text exists. VERDICT: SWEPT; CLEAR.

**Dimension 2 — Downstream copy target:**

BC-2.16.003 §Claroty Contracted OCSF Mappings (ground truth for AC-005, AC-010) and
ADR-058 §K5 Div-3 + §I5 (ground truth for AC-009/AC-011) are source documents for this
story's obligations. This amendment strips version labels from narrative prose per POL-39 —
the §Authority pin table retains machine-readable version pins (v1.7, v2.7) as required
by prism convention; narrative prose now uses section-anchor-only cites. No downstream copy
of the version-label prose exists (labels appeared only in this story's body). VERDICT: CLEAR.

RG-019 and RG-020 are new obligations derived from BC-2.16.003 §Claroty Contracted OCSF
Mappings KF-11 and KF-07 clauses. The AC-010 assertion list now names all covered KFs;
the BC-2.16.003 source retains the ground-truth clause text — no copy divergence.
VERDICT: DISCHARGED IN THIS AMENDMENT.

**Dimension 3 — Mandate anchor:**

BC-2.16.003 §Claroty Contracted OCSF Mappings KF-11 (audit_logs category→raw_extensions):
anchored to `S-ADR058-OCSF-ROUTING-001 AC-010 RG-019 T-11L`.

BC-2.16.003 §Claroty Contracted OCSF Mappings KF-07 (device_alert_relations
alert_id→finding_info.uid): anchored to `S-ADR058-OCSF-ROUTING-001 AC-010 RG-020 T-11M`.

No unanchored MUSTs introduced by this amendment. VERDICT: DISCHARGED IN THIS AMENDMENT.

---

### v1.5 Amendment Sweep (ADR-058 v2.6 + BC-2.16.003 v1.6 + BC-2.16.002 v2.27 propagation)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-COERCION-001* (sibling story, same epic): swept in full for any reference to
`inventory_info`, the two new `select_by_class_name` arms, or the `ocsf.unknown_class_name`
warn emission. COERCION-001 scope is `ColumnMapper::coerce_value` and `build_column_array`
type-coercion gap closure — entirely orthogonal to class_selector resolver arms and the
process-gap warn. No update required. VERDICT: SWEPT; CLEAR.

*class_selector.rs in-file doc tables* (sub-obligation c, AC-009): The module-level doc table
and any inline summary table documenting class name→class_uid mappings are the sibling pair
for the code change. If T-22 adds the `entity_management` and `inventory_info` arms to the
resolver function without updating the doc tables, the doc tables become stale and will
contradict the code — an F-P1-MED-001 class finding. Captured in T-22 sub-obligation (c).
VERDICT: CAPTURED IN T-22.

**Dimension 2 — Downstream copy target:**

BC-2.16.003 v1.6 §Architecture Anchors and EC-016-013-023/024 are the source from which
this story's AC-009/RG-016/RG-017 wire-shape obligations were derived. This amendment
propagates those postconditions into the story — the downstream copy obligation is fulfilled
by writing the wire-shape obligations here (T-11I/T-11J, RG-016/RG-017, AC-009 sub-obligation (b)).

BC-2.16.002 v2.27 catalog row 94 (`ocsf.unknown_class_name`) is the source for AC-011. The
story now carries the obligation derived from that catalog row. No further downstream copy
target exists that requires simultaneous update. VERDICT: CAPTURED IN THIS AMENDMENT.

**Dimension 3 — Mandate anchor:**

ADR-058 §I5 (v2.6) process-gap obligation (ocsf.unknown_class_name WARN): anchored to
`S-ADR058-OCSF-ROUTING-001 AC-011 RG-018 T-24`. No unanchored MUSTs introduced.

BC-2.16.003 v1.6 EC-016-013-023 and EC-016-013-024 wire-level postconditions: anchored to
`S-ADR058-OCSF-ROUTING-001 AC-009 RG-016 T-11I` and `S-ADR058-OCSF-ROUTING-001 AC-009
RG-017 T-11J` respectively. VERDICT: DISCHARGED IN THIS AMENDMENT.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.54 | 2026-08-23 | story-writer | A+W amendment (human decision 2026-08-23, supersedes interim §J6-drop): zero-Tier-1-with-Tier-2 PRESERVES Tier-2 via raw_extensions; NEW ocsf.zero_tier1_table spec-load warning (BC-2.16.002 v2.34 catalog row: fields sensor_id + table_name; ONCE per table); RG-Q-017 added (SAC-1: 46 RGTs, density 46/21=2.19); §Authority re-pinned ADR-058 v2.30→v2.31 + BC-2.11.016 v1.29→v1.30 + BC-2.16.002 v2.33→v2.34 + BC-2.16.003 v1.24→v1.26; OBS-1 §J7 signature drop (validate_ocsf_column_collisions loses source_path param) anchored to T-30/T-31. Changes: (1) version 1.53→1.54. (2) §Authority ADR-058 v2.30→v2.31 + status-date 2026-08-23. (3) §Authority BC-2.11.016 v1.29→v1.30 + A+W sub-case paragraph: EC-11-080 A+W — zero-Tier-1-with-Tier-2 OCSF table MUST register raw_extensions (Json) in available set AND emit ocsf.zero_tier1_table WARN ONCE at spec-load. (4) §Authority BC-2.16.002 v2.33→v2.34 + ocsf.zero_tier1_table catalog row note (SAP-1/PG-LP11-001). (5) §Authority BC-2.16.003 v1.24→v1.26 + OBS-1 §J7 signature-drop note (validate_ocsf_column_collisions source_path removed). (6) §Behavioral Contracts table BC-2.11.016/BC-2.16.002/BC-2.16.003 version pins updated. (7) §Red Gate Tests preamble "forty-five"→"forty-six", "45"→"46". (8) RG-Q-017 entry added: three assertions (E-QUERY-038 Ok for raw_extensions; E-QUERY-038 Err for raw col.name; ocsf.zero_tier1_table WARN exactly once). Placed in ocsf_column_routing_tests.rs. (9) §BC-5.38.001 Density Check 45→46 RGTs, range ..RG-Q-016→..RG-Q-017, density 2.14→2.19; RG-Q-017 A+W coverage note added. (10) AC-019 extended with A+W sub-case (two sub-cases: zero-Tier-1+zero-Tier-2 → ["_sensor","class_uid"]; zero-Tier-1+≥1-Tier-2 A+W → ["_sensor","class_uid","raw_extensions"] + emit ocsf.zero_tier1_table WARN; implementation in T-31). (11) §Mandate Anchor table — AC-019 A+W row added (RG-Q-017, T-31, PENDING A+W-FIX). (12) §Tasks Phase A — T-11AC added (write RG-Q-017, MUST FAIL, before T-GATE); T-GATE range ..RG-Q-016→..RG-Q-017; density 45/21=2.14→46/21=2.19. (13) §Tasks Phase B — T-31 added (A+W-FIX: register raw_extensions for zero-Tier-1-with-Tier-2 OCSF table; emit ocsf.zero_tier1_table WARN ONCE; update ocsf_projected_column_names A+W branch; OBS-1 param drop if not done in T-30); T-19 count 45→46; range extended to RG-Q-017. (14) §TD-VSDD-097 — v1.54 sweep added at top (3-dim: Sibling COERCION-001 unaffected/terminal; Downstream no verbatim copy; Mandate anchored to RG-Q-017+T-31). (15) §Changelog — this v1.54 row added at top. SAC-1 re-verified: 46 RGTs, density 46/21=2.19 ≥ 0.5, red-then-green ordering preserved (T-11AC in Phase A precedes T-31 in Phase B). |
| 1.53 | 2026-08-22 | story-writer | LOCAL pass-1 fix-burst — RG-Q-016 added (H1 §J1 Tier-1-vs-Tier-1 closure); RG-Q-015 strengthened to bind §I7 shape-exception sites (M2 closure); §J4 message made verbatim per E-SPEC-030 template (M1 closure, POL-24); ADR §-citation drift corrected §J5→§J6/§I1→§I7 in code (L1 closure). Density 45/21=2.14. Code fix @891ee536c. No spec (ADR/BC/error-taxonomy) change. Changes: (1) version 1.52→1.53. (2) §Red Gate Tests preamble 44→45. (3) RG-Q-015 entry: note added that it now also binds the two ADR-058 §I7 shape-exception sites (prism-mcp `build_ocsf_column_descriptors` name-set == `ocsf_projected_column_names` assertion and prism-bin `pipeline_result_to_record_batch` Arrow schema field-names == `ocsf_projected_column_names` assertion); these are RG-Q-015 sub-assertions, closing M2 (RG-Q-015 was previously tautological). (4) RG-Q-016 (`test_BC_2_16_003_ocsf_collision_j1_shadow_tier1_vs_tier1_rejected_at_spec_load`) added to SAC-1 list: §J1 Tier-1-vs-Tier-1 shadow sub-case (a Tier-1 column's flattened arrow name equals another Tier-1 column's raw col.name in the same table); prism-spec-engine add_sensor_spec mod tests; E-SPEC-030 [§J1]; traces BC-2.16.003 EC-016-013-032 / ADR-058 §J7. Closes H1. (5) §BC-5.38.001 Density Check 44→45 RGTs, range RG-Q-001..015→RG-Q-001..016, density 2.10→2.14; RG-Q-016 coverage note added. (6) §Mandate Anchor table AC-021 row: RG-Q-012/013/014 → RG-Q-012/013/014/016. (7) §Behavioral Contracts table BC-2.16.003 row: EC-016-013-032 governs /013/014 → /013/014/016. (8) T-GATE: range RG-Q-001..015→RG-Q-001..016; prism-spec-engine list RG-Q-012/013/014→RG-Q-012/013/014/016; density 44/21=2.10→45/21=2.14. (9) T-19: count 44→45; prism-spec-engine list updated. (10) §TD-VSDD-097 — v1.53 sweep added at top. SAC-1 re-verified: 45 RGTs, density 45/21=2.14 ≥ 0.5, red-then-green ordering preserved (no new Phase A task required — RG-Q-016 was written in LOCAL pass-1 fix-burst post-implementation; documented here for SAC-1 traceability only). |
| 1.52 | 2026-08-22 | story-writer | Re-cascade LOW-1/OBS-1/OBS-2 fix spec burst; AC-019/020/021 + RG-Q-010..015 enumerated in SAC-1; §Authority re-pinned ADR-058 v2.28→v2.30, BC-2.11.016 v1.28→v1.29, BC-2.16.003 v1.23→v1.24; density 44/21=2.10; body ADR section-cites normalized to version-free per POL-39/POL-40 (15 loci); E-SPEC-030 (corrected from plan E-SPEC-027). Changes: (1) version 1.51→1.52. (2) §Authority ADR-058 v2.28→v2.30 + status-date 2026-08-22. (3) §Authority BC-2.16.003 v1.23→v1.24 + EC-016-013-032 note: `parse_and_validate_spec_toml` MUST reject §J1/§J2/§J4 collisions via `validate_ocsf_column_collisions`; E-SPEC-030 + collision tag; boot exit 2; hot-reload keeps prior; runtime §J guard stays as defense-in-depth. (4) §Authority BC-2.11.016 paragraph added (v1.29): EC-11-080 zero-Tier-1-column OCSF table MUST register class_uid + _sensor in plan-gate available set. (5) §Behavioral Contracts table BC-2.16.003 v1.23→v1.24; BC-2.11.016 v1.28→v1.29. (6) §Red Gate Tests preamble 38→44; RG-Q-010..015 individual entries added (RG-Q-010/011 prism-query zero-column OCSF; RG-Q-012/013/014 prism-spec-engine add_sensor_spec mod tests E-SPEC-030; RG-Q-015 prism-query table_registry §I7 invariant). (7) §BC-5.38.001 Density Check 38→44 RGTs, 18→21 ACs, density 2.11→2.10 ≥ 0.5. (8) AC-019 (zero-col OCSF table registers class_uid+_sensor; LOW-1-FIX; traces BC-2.11.016 EC-11-080; ADR-058 §J6), AC-020 (consolidated-projection invariant; OBS-1-FIX; traces ADR-058 §I7), AC-021 (spec-load collision validation; OBS-2-FIX; traces BC-2.16.003 EC-016-013-032; ADR-058 §J7; E-SPEC-030) added. (9) §Mandate Anchor table — AC-019 row (RG-Q-010/011, T-28, PENDING LOW-1-FIX), AC-020 row (RG-Q-015, T-29, PENDING OBS-1-FIX), AC-021 row (RG-Q-012/013/014, T-30, PENDING OBS-2-FIX) added. (10) §Tasks Phase A — T-11Z (write RG-Q-010/011), T-11AA (write RG-Q-012/013/014), T-11AB (write RG-Q-015) added before T-GATE; T-GATE count 38→44, density 44/21=2.10. (11) §Tasks Phase B — T-28 (LOW-1-FIX: remove outer guard, `register_sensor` OCSF branch, `table_registry.rs`), T-29 (OBS-1-FIX: `ocsf_projected_column_names`/`ocsf_projected_column_types` to `column_mapping.rs`; thin forward in `engine.rs`), T-30 (OBS-2-FIX: `validate_ocsf_column_collisions` as Validation Rule 8 in `add_sensor_spec.rs`; E-SPEC-030) added after T-27 and before T-19; T-19 count 38→44; range extended with RG-Q-010..015 and prism-spec-engine add_sensor_spec distribution. (12) §TD-VSDD-097 — v1.52 sweep added at top (all three dimensions CLEAR/ANCHORED). (13) Body ADR-058 section cites normalized to version-free form (15 body loci stripped of v2.28 qualifier per POL-39/POL-40; historical TD-VSDD-097 sweeps and changelog rows grandfathered). SAC-1 re-verified: 44 RGTs, density 44/21=2.10 ≥ 0.5, red-then-green ordering (T-11Z/T-11AA/T-11AB in Phase A precede T-28/T-29/T-30 in Phase B). |
| 1.51 | 2026-08-22 | story-writer | Re-cascade P1 HIGH-001/MED-002 closure: RG-Q-008/009 multi-tenant + pipe coverage; shared-helper Site-E fix; human-directed 2026-08-22. Changes: (1) version 1.50→1.51. (2) §Red Gate Tests preamble — "thirty-six"→"thirty-eight"; "36 confirmed failing"→"38 confirmed failing". (3) RG-Q-008 (`test_BC_2_11_016_RG_Q_008_multitenant_ocsf_head_projection`) added: multi-tenant `resolved_spec_map` OCSF head-gate via `check_column_availability` + shared helper `ocsf_or_raw_column_names_for_table`; OCSF name resolves, raw col.name rejected with OCSF `available_columns`; covers AC-016 multi-tenant head path + AC-018. (4) RG-Q-009 (`test_BC_2_11_016_RG_Q_009_multitenant_ocsf_pipe_stage`) added: multi-tenant OCSF pipe-stage `get_initial_available_columns` (Site E, TD-VSDD-060 5-site sweep); `| where message` resolves, `| where description` rejected; same shared helper; covers AC-016 multi-tenant pipe path + AC-018. (5) §BC-5.38.001 Density Check — count 36→38, range RG-Q-001..007→RG-Q-001..009, density 2.00→2.11 ≥ 0.5; coverage notes for RG-Q-008/009 added. (6) AC-016 "Covered by" → RG-Q-001..009. (7) AC-018 "Covered by" → nine tests. (8) §Mandate Anchor — AC-018 row range → RG-Q-001..009. (9) §Behavioral Contracts table — BC-2.11.016 row RG range → RG-Q-001..009. (10) T-GATE: count/range → 38/RG-Q-001..009; density → 38/18=2.11. (11) T-27: "Makes green" extended for RG-Q-008/009 with shared-helper + Site-E explanation. (12) T-19: count 36→38; range → RG-Q-001..009. (13) §File Structure Requirements — `engine.rs` row: shared-helper `ocsf_or_raw_column_names_for_table` noted + 5-site TD-VSDD-060 sweep (A table_registry.rs; B/C/E engine.rs via helper; D materialization.rs=test-only unaffected); `ocsf_column_routing_tests.rs` row: RG-Q-001..009. (14) §TD-VSDD-097 — v1.51 three-dim sweep added at top. Pins UNCHANGED: ADR-058 v2.28, BC-2.16.002 v2.33, BC-2.16.003 v1.23, BC-2.11.016 v1.28. Input-hash unchanged (no tracked input files modified). SAC-1: 38 RGTs, density 2.11≥0.5, red-then-green ordering preserved. |
| 1.50 | 2026-08-22 | story-writer | Holdout-gap query-surface formalization: BC-2.11.016 added to story; AC-016/017/018 + RG-Q-001..007; T-26/T-27 Fix A/B/C; human-directed 2026-08-22. Changes: (1) version 1.49→1.50; modified 2026-08-21→2026-08-22. (2) `behavioral_contracts:` frontmatter — BC-2.11.016 added as fourth entry. (3) §Behavioral Contracts body table — BC-2.11.016 row added (v1.28, active; EC-11-079 relevance: OCSF-mode column-resolution gate — E-QUERY-038 and E-QUERY-002/041 type-compat operate against OCSF-flattened schema). (4) Token Budget table — "3 BCs" → "4 BCs", ~6.5k → ~8k. (5) §Red Gate Tests preamble — updated to 36 tests; RG-Q-001 through RG-Q-007 individual test entries added (`test_BC_2_11_016_RG_Q_001` through `test_BC_2_11_016_RG_Q_007` in `crates/prism-query/src/tests/ocsf_column_routing_tests.rs`) before §BC-5.38.001 Density Check. (6) §BC-5.38.001 Density Check — updated to 36 RGTs / 18 ACs = 2.00 ≥ 0.5. (7) §Acceptance Criteria — AC-016 (OCSF-mode column resolution: E-QUERY-038 gate on OCSF-flattened schema, 3 sub-cases, Fix A anchor; traces to BC-2.11.016 EC-11-079), AC-017 (E-QUERY-002/041 type-compat by OCSF-flattened name, Fix C; traces to BC-2.11.016 EC-11-079), AC-018 (describe/select/query name-agreement invariant; traces to BC-2.11.016 EC-11-079) added after AC-015. (8) §Tasks Phase A — T-11W (write RG-Q-001/002), T-11X (write RG-Q-003), T-11Y (write RG-Q-004/005/006/007) added; T-GATE updated to 36 tests with density 36/18=2.00. (9) §Tasks Phase B — T-26 (Fix A: `table_registry.rs` OCSF-flattened name seeding), T-27 (Fix B/C: `engine.rs` E-QUERY-038 + E-QUERY-002/041 gates use OCSF-flattened names) added; T-19 updated to 36 RGTs with RG-Q-001..007 distribution in `prism-query/src/tests/ocsf_column_routing_tests.rs`. (10) §File Structure Requirements — three new rows: `crates/prism-query/src/table_registry.rs` (Fix A — AC-016/018/RG-Q-001..005/007), `crates/prism-query/src/engine.rs` (Fix B/C — AC-016 available_columns, AC-017, RG-Q-003/006), `crates/prism-query/src/tests/ocsf_column_routing_tests.rs` (Create: RG-Q-001..007). (11) §Mandate Anchor table — four new PENDING rows for BC-2.11.016 EC-11-079 MUSTs: TableRegistry OCSF-name registration (RG-Q-001/002/004/005/006; Fix A); E-QUERY-002/041 type-compat by OCSF-flattened name (RG-Q-003; Fix C); name-agreement invariant (RG-Q-001..007; Fix A/B/C); flag-false green-lock (RG-Q-007; Fix A). (12) §TD-VSDD-097 — v1.50 sweep added at top: Dim-1 COERCION-001 historical snapshot preserved; Dim-2 BC-2.11.016 EC-11-079 cited not copied, no downstream copy targets; Dim-3 all new MUSTs anchored to RG-Q-001..007. SAC-1 re-verified: 36 RGTs, density 2.00≥0.5, red-then-green ordering (T-11W/T-11X/T-11Y in Phase A precede T-26/T-27/T-19 in Phase B). |
| 1.49 | 2026-08-21 | spec-steward | Records-tier pin-consistency sweep closing LOCAL pass-8 OBS-1 + straggler ADR-058 v2.26 refs (human-directed 2026-08-21). (1) version 1.48→1.49. (2) §Authority BC-2.16.002 Version `2.32`→`2.33` + §Behavioral Contracts table BC-2.16.002 row v2.32→v2.33 (BC-2.16.002 updated to v2.33 in same burst; BC-2.16.002 is NOT in ROUTING-001 tracked inputs per v1.45 changelog, so input-hash b49d41f unchanged). (3) Three straggler ADR-058 v2.26 current-context refs advanced to v2.28: §Authority BC-2.16.003 paragraph EC-016-013-029 inline (`ADR-058 v2.26 §J2`→`ADR-058 v2.28 §J2`); RG-026 intro (`ADR-058 v2.26 §B2/§I2)`→`ADR-058 v2.28 §B2/§I2)`); AC-007c trace (`ADR-058 v2.26 §B2 / §I2`→`ADR-058 v2.28 §B2 / §I2`). These three v2.26 refs were not matched by the "ADR-058 v2.26 §" replace_all form used in v1.47 (compound sub-section forms §B2/§I2 and §J2-with-trailing-parens were not selected by the bare-§ pattern). Historical COERCION-001 sibling records in §v1.48/v1.47/v1.46 dim-1 entries (ADR-058 v2.26, BC-2.16.002 v2.32) preserved unchanged as correct frozen merge snapshots. TD-VSDD-097: Dim-1 COERCION-001 historical snapshot preserved (NO CHANGE); Dim-2 six active-body loci changed, no downstream copy targets; Dim-3 no new MUSTs. |
| 1.48 | 2026-08-21 | story-writer | LOCAL pass-2 MED-1 + HIGH-1 spec-side closure. (1) version 1.47→1.48; input-hash f23f905→b49d41f (ADR-058 v2.28, BC-2.16.003 v1.23 are the new input versions). (2) AC-015 class_uid description aligned to canonical ADR-058 §G / BC-2.16.003 string: "OCSF event class identifier derived from sensor TOML ocsf_class. Example: 3004 for entity_management (audit_logs), 2004 for detection_finding (alerts, device_alert_relations), 5001 for inventory_info (devices)." (3) AC-015 _sensor description aligned to canonical: "Sensor identifier. Value: <sensor_id> (e.g., 'claroty')." (4) AC-015 wire-shape assertion requirement: "name, col_type, and nullable" → "name, col_type, nullable, and description." (5) RG-028 "asserts ALL FOUR"→"ALL SIX": assertions (v)+(vi) added for class_uid/_sensor description text verbatim from canonical ADR-058 §G / BC-2.16.003; wire-shape updated to include description; RED condition "(i)-(iv)"→"(i)-(vi)." (6) T-11V "assert ALL FOUR"→"ALL SIX": matching updates for assertions (v)+(vi) and wire-shape note "(name + col_type + nullable + description)." (7) T-16B class_uid description: "OCSF class identifier synthesized from ocsf_class; queryable as INTEGER column" → canonical ADR-058 §G string. (8) T-16B _sensor description: "Sensor identifier synthesized by pipeline_result_to_record_batch; queryable as STRING column" → canonical ADR-058 §G string. (9) §Authority ADR-058 v2.27→v2.28 (11 active-body occurrences via replace_all + 4 special cases: title pin, Version `2.27`→`2.28`, "The v2.27 §J2" mandate narrative, "ADR-058 v2.27." RG-027 intro). (10) §Authority BC-2.16.003 v1.22→v1.23 (§Authority + §Behavioral Contracts table). (11) §v1.48 TD-VSDD-097 three-dimension sweep added. SAC-1 re-verified: 29 RGTs (RG-028 gains assertions (v)+(vi), count still 1 test), density 1.93≥0.5, red-then-green ordering intact. Root cause: AC-015/T-16B specified non-canonical description strings → implementer emitted description: None (MED-1); RG-028/T-11V omitted description assertion → paper-green gap (HIGH-1/SID-2). |
| 1.47 | 2026-08-21 | story-writer | Stage 2 spec-augmentation burst (OQ-001/OQ-003/OQ-005 human decisions 2026-08-21). (1) version 1.46→1.47; crates_touched gains prism-query; input-hash ca528ff→f23f905 (ADR-058 and BC-2.16.003 updated to v2.27/v1.22 in architect+PO leg of same burst). (2) §Authority ADR-058 v2.26→v2.27, date 2026-08-20→2026-08-21; ocsf_field count 26→27 (audit_logs.id gains OQ-005 mapping). (3) §Authority BC-2.16.003 v1.21→v1.22, date 2026-08-20→2026-08-21. (4) §Behavioral Contracts table BC-2.16.003 v1.21→v1.22. (5) Red Gate preamble 27→29 confirmed failing. (6) Mandate Anchor §J2 rows v2.26→v2.27. (7) RG-026/RG-027 traces v2.26→v2.27. (8) RG-021 FLIPPED: was `test_claroty_audit_logs_id_column_goes_to_raw_extensions_not_activity_uid` (KF-05 raw_extensions assertion) → now `test_claroty_audit_logs_id_produces_metadata_uid_top_level_arrow_field` (OQ-005 Tier-1 Arrow column `metadata_uid` wire-shape assertion). (9) RG-PD-001 NEW: `test_extract_time_window_from_ast_recognizes_ocsf_flattened_time_column_as_index_eligible` — OQ-001 push-down eligibility for OCSF-flattened Arrow name `"time"`. (10) RG-028 NEW: `test_prism_describe_ocsf_column_naming_true_emits_class_uid_and_sensor_descriptors` — OQ-003 synthesized descriptor emission. (11) BC-5.38.001 density 27/13=2.08→29/15=1.93 ≥ 0.5 — compliant. (12) AC-005 header and entry 6: KF-05 remove → OQ-005 set `ocsf_field = "metadata.uid"`. (13) audit_logs contracted mapping table id row: KF-05→OQ-005 (metadata.uid / metadata_uid). (14) Post-corrections ocsf_field count 26(audit_logs:6)→27(audit_logs:7). (15) AC-010 header and assertion 5: KF-05→OQ-005 Tier-1 metadata_uid. (16) AC-013 trace v2.26→v2.27. (17) EC-009 count 26→27, audit_logs 6→7. (18) EC-016-013-028 trace v2.26→v2.27. (19) AC-014 NEW: OQ-001 push-down eligibility; covered by RG-PD-001. (20) AC-015 NEW: OQ-003 prism_describe synthesized descriptors; covered by RG-028. (21) §Architecture Mapping: new row for `extract_time_window_from_ast` in prism-query::pushdown (Pure, OQ-001/AC-014/RG-PD-001). (22) T-11N updated: writes RG-021 (OQ-005 Tier-1 assertion). (23) T-11S/T-11T traces v2.26→v2.27. (24) T-11U NEW: write RG-PD-001 in prism-query/pushdown.rs mod tests. (25) T-11V NEW: write RG-028 in prism-mcp/tests/. (26) T-GATE: 27→29, adds prism-query, density 1.93. (27) T-16B NEW: prism_describe emit class_uid/_sensor synthesized ColumnDescriptors under ocsf_column_naming; makes RG-028 green. (28) T-17 item 6 and note: KF-05 remove → OQ-005 ocsf_field set. (29) T-21 clause (c) v2.26→v2.27. (30) T-19: 27→29 RGTs + adds prism-query + RG-PD-001/RG-028 distribution. (31) T-25 NEW: extract_time_window_from_ast dual-name datetime_index_cols insert + stale doc comment update; makes RG-PD-001 green. (32) §File Structure: prism-mcp/tests/ row adds RG-028; new prism-query/pushdown.rs row (OQ-001/AC-014). (33) §v1.47 TD-VSDD-097 three-dimension sweep added. SAC-1 re-verified: 29 RGTs, density 1.93≥0.5, red-then-green ordering preserved. |
| 1.46 | 2026-08-21 | story-writer | Pre-delivery burst: (1) Pin sweep — ADR-058 v2.23→v2.26 (18 occurrences: §Authority leading pin + status-date; 14 active-body "ADR-058 v2.23 §" via replace_all; "The v2.23 §J2" special case; "ADR-058 v2.23." RG-027 special case). (2) BC pin sweep — BC-2.16.003 v1.19 draft→v1.21 active (§Authority + §Behavioral Contracts table); BC-2.16.002 v2.30→v2.32 (§Authority + §Behavioral Contracts table). (3) input-hash refreshed 859dc7f→ca528ff (inputs drifted via `69d821be5` fix(claroty) PR on develop; ca528ff is the current computed hash of all story inputs). (4) holdout_scenarios wired: []→[HS-ROUTING-001-A-001, HS-ROUTING-001-A-002, HS-ROUTING-001-A-003, HS-ROUTING-001-A-004]. (5) §v1.46 TD-VSDD-097 three-dimension sweep added. SAC-1 confirmed compliant (27 RGTs, density 2.08≥0.5, red-then-green task ordering). SAC-2 confirmed (ADR-058 anchor_stories includes S-ADR058-OCSF-ROUTING-001). Remove-uncertainty pass: Arrow 58.2.0 ✓, DataFusion 53.1 ✓, tracing-test 0.2 ✓, SensorSpec exists in spec_parser.rs ✓, pipeline_result_to_record_batch exists in spec_driven_adapter.rs ✓, EventClassSelector::select_by_class_name exists in class_selector.rs ✓ — no uncertainties. Sibling COERCION-001 merged at v1.47 with correct pins — no sibling edit required. HOLDOUT-INDEX.md updated (HS-022 group registered, total_scenarios 89→93, total_groups 15→16, v1.18→v1.19). |
| 1.45 | 2026-08-20 | state-manager | D-2254 SAP-1/PG-LP11-001 discharge burst (state-manager leg): BC-2.16.002 §Authority pin v2.29→v2.30 + §Behavioral Contracts table pin v2.29→v2.30 (product-owner registered catalog row 95 `column_coercion_failure` WARN in BC-2.16.002 §Postconditions §Canonical Structured Event Catalog in same burst). Input-hash updated f490a3d→859dc7f (pre-existing drift from `69d821be5` fix(claroty) commit — `claroty.sensor.toml` and `spec_driven_adapter.rs` changed on develop; hash computed by validate-input-hash hook; BC-2.16.002 is NOT in ROUTING-001 inputs, so drift was from prior develop commit). Sibling COERCION-001 bumped to v1.43 (BC-2.16.002 pin v2.29→v2.30, input-hash 67f13c7→fb7a031) in same burst. NOT merged — develop still @69d821be; workspace_test_count stays 5743. §v1.45 Amendment Sweep: Dimension 1 (sibling pair) — COERCION-001 amended same burst (v1.42→v1.43 state-manager leg); CLEAR. Dimension 2 (downstream copy) — §Authority pin is terminal; no independent copy artifact; CLEAR. Dimension 3 (mandate anchor) — no new MUST blocks; CLEAR. |
| 1.44 | 2026-08-19 | story-writer | Leg 2 pin bump — BC-2.16.003 v1.18→v1.19 (BC-2.16.003 updated to v1.19 in Leg 1 of this burst); BC-2.16.002 v2.28→v2.29 (BC-2.16.002 updated to v2.29 in Leg 1); §Authority and §Behavioral Contracts table pins updated to current. Also stripped 13 remaining `§Interpretation A v1.18` inline stamps to version-free `§Interpretation A` per Bucket B terminal normalization (POL-39). Input-hash updated 5eac1dc→f490a3d (computed by validate-input-hash hook on first edit — inputs BC-2.16.003 and BC-2.16.002 changed in Leg 1). Sibling COERCION-001 bumped to v1.40 in same burst. §v1.44 Amendment Sweep added. |
| 1.43 | 2026-08-19 | story-writer | FB-62/63 TERMINAL POL-39 normalization — stripped ALL version qualifiers from ADR-058 section cites across ROUTING-001's entire active body (F-P62-MED-001 ≡ F-P63-MED-001, corroborated by two adversarial passes). The v1.42 sweep incorrectly claimed stamps existed "only in §Authority" — this burst sweeps the complete active body. Sections swept: §Mandate Anchor table (§D1/§G/§I1 row cites); §Red Gate Tests (RG-003 §I1; RG-024 §D1; RG-025 §G × 4 instances); §BC-5.38.001 density check (§G); §Acceptance Criteria (AC-002 §I1; AC-003 §D1; AC-006 §G × 2; AC-007b §G × 2; AC-012 §D1 × 2); §Architecture Mapping table (§I1; §D1; §G); §Architecture Compliance Rules Rule 1 (§I1); §File Structure Requirements column_mapping.rs row (§I1); §Forbidden Dependencies (§I1); §Edge Cases table EC-016-013-027 row (§G); §Tasks T-06 (§I1); T-11R (§G); T-13 (§I1); T-16 (§G × 3). 22 version qualifier strips total. Dependency pins preserved: §Authority "ADR-058 … Version `2.23`" and "BC-2.16.003 … Version `1.18`"; §Behavioral Contracts table rows "ADR-058 \| v2.23" and "BC-2.16.003 \| v1.18". COERCION-001 confirmed clean — no version bump. §v1.43 Amendment Sweep added (honest TD-VSDD-097 3-dim discharge). |
| 1.42 | 2026-08-19 | story-writer | FB-58/60 records micro-burst — categorically end §Authority provenance-label drift class (F-P58-LOW-002 + F-P58-LOW-001). Fix 1 (F-P58-LOW-002): §Authority §B2/§D1/§G/§I1 (both sub-labels)/§I2/§J2 provenance labels normalized to version-free form — removed "v2.23 amendment:", "corrected v2.18:", "corrected v2.21:", "v2.23 Tier-1/Tier-2 model:", "v2.23 amendment:" stamps per POL-39 (behavioral-anchor cites only; no vX.YZ origin-provenance decorations). The single CURRENT-version dependency pin "Version `2.23`" is retained. Fix 2 (F-P58-LOW-001): ADR-058 §Authority status-date parenthetical corrected "(2026-08-18)"→"(2026-08-19)" (ADR-058 frontmatter `modified:` is 2026-08-19; consistent with sibling BC-2.16.003 "(modified 2026-08-19)" cite corrected in v1.41). Sibling COERCION-001 bumped to v1.39 in same burst. §v1.42 Amendment Sweep added. |
| 1.41 | 2026-08-19 | story-writer | FB-55/56/57 LEG 2 — records-tier fixes. F-P57-LOW-001: §Authority §I1 provenance-label corrected — "§I1 corrected v2.23" was wrong origin; the canonical-home correction for `ocsf_field_to_arrow_name` landed in ADR-058 v2.21 per ADR §Changelog (§v1.37 Amendment Sweep entry confirms "§Authority ADR-058 pin v2.20→v2.21 with §I1 crate-placement correction note"); relabeled to "§I1 corrected v2.21" (origin-provenance labels are frozen at their originating version per POL-39; peer labels "§D1 corrected v2.18" and "§I1 corrected v2.18 (two-step form)" in the same §Authority block are correctly frozen). Fix 5: §Authority BC-2.16.003 modified-date parenthetical corrected "(modified 2026-08-18)"→"(modified 2026-08-19)" (BC-2.16.003 `modified:` is now 2026-08-19 per Leg 1 of this burst). Sibling COERCION-001 bumped to v1.38 in same burst. §v1.41 Amendment Sweep added. |
| 1.40 | 2026-08-19 | story-writer | FB-52/53/54 LEG 3 — records-tier fixes + exhaustive residual sweep. Fix 1: T-GATE density denominator 12→13, ratio 2.25→2.08 (AC-013 was added in v1.39 but density not updated). Fix 2+4 combined: `§Interpretation A v1.16` wrapped-line instance in RG-025 Covers/Traces updated to v1.18 (net; was missed by prior sweeps due to line-wrap). Fix 3: §Authority parenthetical rewritten to attribute 31 pre-correction count to §J4 and 26 post-correction count to ADR-058 §Status (section-pointer precision). Version re-pin sweep (Fix 4): all active-body ADR-058 v2.22→v2.23 and BC-2.16.003 v1.17→v1.18 references updated (§Authority, §Mandate Anchor, §Behavioral Contracts, RG-025..027, AC-002/006/007b/007c/013, Architecture Mapping, Edge Cases, Tasks, Architecture Compliance Rules, Forbidden Dependencies, File Structure). Sibling COERCION-001 bumped to v1.37 same burst. §v1.40 Amendment Sweep added. |
| 1.39 | 2026-08-18 | story-writer | FB-49/51 Leg 2 — round-2 adversary findings closed. F-P51-MED-001: RG-026 orphan resolved — T-15 extended to (a) mandate source_path extraction + ENRICH-1 `Value::Array`→compact-JSON-list-string normalization for `ocsf_field == None` columns in `pipeline_result_to_record_batch` raw_extensions aggregation loop (reuse shared pipeline, NOT naive `r.get(col.name)`; BC-2.16.003 EC-016-013-028 reworded; ADR-058 §I2) and (b) name RG-026 as second green target alongside RG-008. F-P49-MED-001: AC-007a rewritten — removed `build_column_array` attribution sentence; added source_path extraction + ENRICH-1 normalization mandate for `ocsf_field == None` columns. F-P49/51-MED-002: AC-013 added — dedicated AC for §J2 synthesized-name fail-closed guard (`Err(ArrowError::SchemaError)` when any `ocsf_field` flattens to `class_uid`, `category_uid`, `_sensor`, or `raw_extensions`; traces to BC-2.16.003 EC-016-013-029 + ADR-058 v2.22 §J2); RG-027 Covers/Traces updated to reference AC-013 + EC-016-013-029; §Mandate Anchor table §J2 row AC column `EC-010 (extended), T-21 clause (c)` → `AC-013, T-21 clause (c)` + EC-016-013-029 added; §Mandate Anchor §J2 discharge narrative extended with v2.22 synthesized-name guard anchor. BC-pin sweep: §Authority BC-2.16.003 v1.16→v1.17 + EC-016-013-029 note + EC-016-013-028 reworded; §Behavioral Contracts row v1.16→v1.17 + EC-016-013-029 annotation; all 12 `§Interpretation A v1.16` active-text instances → `v1.17`. Density recomputed: 27/13 = 2.08 ≥ 0.5. AC count 12→13. §Architecture Mapping §I2 row and §File Structure spec_driven_adapter.rs row updated to remove misleading `build_column_array NOT added` language and add source_path+ENRICH-1 normalization mandate. input-hash updated aeafdff→90f6a36 (BC-2.16.003 updated in Leg 1). §v1.39 TD-VSDD-097 amendment sweep added. |
| 1.38 | 2026-08-18 | story-writer | OCSF-correctness Claroty SPEC fix-burst leg 3 (F-P46-MED-001 version-pin sweep AC-006; F-P48-MED-001 EC-016-013-011 trace; F-P48-MED-002 EC-016-013-028 + §J2 guard; F-P48-LOW-001 RG-010 refresh; F-P48-OBS-2 RG-025 nullable dual-condition). Closed six findings. Changes: (1) §Authority ADR-058 v2.21→v2.22 with §B2/§I2/§J2 amendment notes; (2) §Authority BC-2.16.003 v1.15→v1.16 with EC-016-013-028/EC-016-013-011; (3) §Behavioral Contracts BC-2.16.003 row v1.15→v1.16; (4) §Mandate Anchor — §G/§I1 rows v2.21→v2.22; two new rows RG-026 (EC-016-013-028) and RG-027 (§J2 guard); (5) input-hash updated aeafdff (drift from legs 1-2 of burst changing ADR-058 and BC-2.16.003); (6) RG-010 self-match exclusion: inline 5-column enumeration replaced with reference to BC-2.16.003 §Claroty Contracted OCSF Mappings ground-truth devices table (20 columns per PR #236); (7) RG-025 intro v2.21/v1.15→v2.22/v1.16; (8) RG-025 assertion (iv) v2.21→v2.22; (9) RG-025 assertion (v) v2.21/v1.15→v2.22/v1.16 + dual-condition rationale (per-row null AND per-table absence); (10) RG-025 Covers/Traces v2.21/v1.15→v2.22/v1.16; (11) AC-006 preamble v2.20/v1.14→v2.22/v1.16 (THE v1.37 sibling-sweep gap); (12) AC-006 trace v1.14/v2.20→v1.16/v2.22; (13) AC-007b header v2.21/v1.15→v2.22/v1.16; (14) AC-007 traces v1.15/v2.21→v1.16/v2.22; (15) AC-007c NEW — EC-016-013-028 multi-valued array compact JSON-list string obligation; (16) AC-011 trace — added BC-2.16.003 EC-016-013-011 corrected runtime-WARN reference; (17) AC-012 — 14→16 new callers (RG-026/027 added); T-14A 14→16, total 17→19; (18) BC-5.38.001 — 25→27 RGTs, 2.08→2.25, RG-026/027 coverage notes, RG-025 trace v2.22/v1.16; (19) RG-026 NEW; (20) RG-027 NEW; (21) T-11S NEW (write RG-026); (22) T-11T NEW (write RG-027); (23) T-21 clause (c) NEW — §J2 reserved-name guard makes RG-027 green; (24) T-GATE 25→27, density 2.08→2.25, RG-026/027 in prism-bin; (25) T-19 25→27; (26) §Architecture Mapping prism_describe row v2.20→v2.22; all remaining v2.21/v1.15 body pins swept to v2.22/v1.16 (AC-002, RG-003, T-06, T-13, T-16 col_type/nullable, Architecture Compliance Rule 1, Forbidden Dependencies prism-mcp, File Structure column_mapping row); (27) Edge Cases table — EC-016-013-027 v2.20/v1.14→v2.22/v1.16 + RG-025 assertion count corrected from three to five; new EC-016-013-028 row; (28) §v1.38 TD-VSDD-097 Amendment Sweep added. |
| 1.37 | 2026-08-18 | story-writer | OCSF-correctness Claroty SPEC fix-burst leg 3 (F-P43-HIGH-001 + F-P43-MED-001 + F-P44-OBS-001): closed three findings from adversary passes P43/P44. F-P43-HIGH-001 [HIGH]: swept `ocsf_field_to_arrow_name` canonical home from `prism-bin::spec_driven_adapter` (unreachable from prism-mcp without forbidden cycle) to `prism-spec-engine::column_mapping` per ADR-058 §I1 v2.21. Changes: (1) §Authority ADR-058 pin v2.20→v2.21 with §I1 crate-placement correction note and §G four-field shape; (2) §Authority BC-2.16.003 pin v1.14→v1.15 with EC-016-013-027 four-field shape; (3) §Behavioral Contracts BC-2.16.003 row v1.14→v1.15; (4) RG-003 reworded: `prism-spec-engine::column_mapping` canonical home, no-cycle import contract from both prism-bin and prism-mcp; (5) §Mandate Anchor: §I1 crate-placement MUST row added (AC-002/RG-003/RG-004); (6) AC-002 updated: module ref `prism-spec-engine::column_mapping`; (7) §Architecture Mapping `ocsf_field_to_arrow_name` row relocated to `prism-spec-engine::column_mapping`; (8) §Architecture Compliance Rules Rule 1 reworded: `prism-spec-engine::column_mapping` MUST, forbidden-cycle explanation; (9) §Forbidden Dependencies: first bullet fixed (function not in prism-bin); new `prism-mcp MUST NOT import from prism-bin` bullet added; (10) §File Structure Requirements: `prism-spec-engine/src/column_mapping.rs` row added; prism-bin row updated to "import from"; test row RG-003..RG-006→RG-005..RG-006; (11) T-06/T-07: file location note added; (12) T-13: target changed to `column_mapping.rs` with import instructions for both prism-bin and prism-mcp; (13) T-GATE/T-19: `RG-001..004 in prism-spec-engine` (was `RG-001..002`); F-P43-MED-001 [MED]: §Red Gate intro "twenty-four"/"24" corrected to "twenty-five"/"25". F-P44-OBS-001 [OBS]: (14) RG-025 extended from three assertions (i)-(iii) to five: (iv) `col_type = prism_core::column::ColumnType::Json`; (v) `nullable = true`; RED condition updated; version refs v2.21/v1.15; (15) T-11R five assertions (i)-(v); (16) T-16 four-field ColumnDescriptor shape; (17) §BC-5.38.001 density note updated; (18) AC-007b `col_type`/`nullable` fields; version refs; (19) AC-007 traces updated; (20) §Mandate Anchor §G row updated with four-field shape. §v1.37 Amendment Sweep added. Post-authorship correction (same version — pre-commit): phantom-section cite "ADR-023 §D3" replaced with `dependency-graph.md §Dependency Rules Rule 2` (Level 6 prism-mcp / Level 7 prism-bin; lower-layer crates never depend on higher-layer crates) at four loci: §Mandate Anchor table, RG-003, §Architecture Compliance Rules Rule 1, §Forbidden Dependencies prism-mcp bullet. |
| 1.36 | 2026-08-18 | story-writer | OCSF-correctness Claroty SPEC fix-burst leg 3 (F-P40/P42-HIGH-001): propagated ratified Tier-1/Tier-2 `prism_describe` model from ADR-058 §G v2.20 / BC-2.16.003 §Interpretation A v1.14 into this story. Changes: (1) §Authority ADR-058 pin v2.19→v2.20 with §G Tier-1/Tier-2 description; (2) §Authority BC-2.16.003 pin v1.13→v1.14, date 2026-08-17→2026-08-18, EC-016-013-027 reference added; (3) §Behavioral Contracts BC-2.16.003 row v1.13→v1.14 with EC-016-013-027; (4) §Mandate Anchor table: new row for ADR-058 §G v2.20 / BC-2.16.003 EC-016-013-027 MUST (AC-006 Tier-2 / AC-007b / RG-025 / T-11R / T-16); (5) RG-007 coverage note updated: "Covers AC-006 Tier-1; Tier-2 covered by RG-025"; (6) RG-025 added (`test_prism_describe_ocsf_column_naming_true_raw_extensions_descriptor_and_no_phantom_col_names` — three-assertion: phantom prohibition (i), count exactly 1 (ii), source-key enumeration (iii)); (7) BC-5.38.001 density check 24→25 RGTs, 2.00→2.08, RG-025 coverage note; (8) AC-006 rewritten: Tier-1 positive path + Tier-2 prohibition (MUST NOT emit phantom ColumnDescriptor names); (9) AC-007 expanded: AC-007a (query engine, unchanged) + AC-007b (new: `prism_describe` MUST emit exactly ONE `raw_extensions` ColumnDescriptor enumerating source keys); (10) EC-016-013-027 added to edge cases table; (11) §Architecture Mapping `prism_describe` row rewritten for Tier-1/Tier-2; (12) T-11R added: Phase A task to write RG-025; (13) T-GATE updated 24→25, density 2.00→2.08, prism-mcp now carries RG-007 and RG-025; (14) T-16 rewritten: Tier-1/Tier-2 implementation steps, makes RG-007 + RG-025 green; (15) T-19 updated: 24→25 RGTs, RG-025 in prism-mcp. §v1.36 Amendment Sweep added. |
| 1.35 | 2026-08-18 | state-manager | OCSF-correctness Claroty SPEC pass-39 TD-VSDD-096 records-only micro-burst: F-P39-LOW-001 [LOW, POL-37/TD-VSDD-060 date-sync] §Authority BC-2.16.002 citation date parenthetical corrected "(modified 2026-08-16)"→"(modified 2026-08-17)" (BC-2.16.002 frontmatter `modified:` is 2026-08-17; v2.28 landed via D-2232 on 2026-08-17). Comprehensive perimeter-wide records-hygiene audit: COERCION-001 §Authority all parentheticals ACCURATE; ADR-058 no date parentheticals in body; version pins across all three perimeter artifacts ACCURATE; volatile line-cite tokens CLEAR; changelog L1/L7 PASS for all three artifacts. ZERO content/mechanism changes. §v1.35 Amendment Sweep added. |
| 1.34 | 2026-08-18 | state-manager | OCSF-correctness Claroty SPEC pass-36 TD-VSDD-096 records-only micro-burst: F-P36-LOW-001 [LOW, TD-VSDD-091] §Changelog v1.12 row contained a quoted volatile-line-cite token in record-tier text; rephrased F3 description to remove the bare line number while preserving meaning (§Changelog §v1.9 Red-then-green gate cite). Anti-whack-a-mole sweep of all three perimeter artifacts (ADR-058, COERCION-001, ROUTING-001 §Changelog + §TD-VSDD-097 + §Authority): zero additional volatile cite tokens found. ZERO content/mechanism changes. §v1.34 Amendment Sweep added. |
| 1.33 | 2026-08-18 | story-writer | OCSF-correctness Claroty SPEC pass-34 fix-burst: F-P34-MED-001 [MED] + F-P34-LOW-001 [LOW] + ADR-058 pin v2.18→v2.19. F-P34-MED-001: AC-012 caller enumeration corrected in both directions — (a) the 14 RG-005..022 entries relabeled from "Existing test callers" to "NEW test callers authored by this story" (Phase A Red Gate tests, not pre-existing callers); (b) `test_BC_2_01_013_crowdstrike_fql_datetime_index_col_string_equality_safe` added as new "Pre-existing test callers (1)" subsection — must be updated with `ocsf_column_naming = false` SensorSpec to preserve CrowdStrike behavior; count corrected in v1.32 changelog from "(1 production + 14 test + 1 new RG-024)" to "(1 production + 1 pre-existing test + 14 new story RG tests + 1 new RG-024)"; T-14A updated: threading expression `&self.sensor_spec` → `&self.sensor_spec.spec`, caller count 15→17, two-part (a)/(b) update instruction. F-P34-LOW-001: threading expression `&self.sensor_spec` (or equivalent) → `&self.sensor_spec.spec` at AC-003 parameter threading note and AC-012 production callers description; "or equivalent access" hedge removed — confirmed by existing production callers `self.sensor_spec.spec.sensor_id` in `fetch()`. ADR-058 pin v2.18→v2.19 at §Authority. Sibling: COERCION-001 ADR pin bump v2.18→v2.19 reported to state-manager (not silently edited). §v1.33 Amendment Sweep added. |
| 1.32 | 2026-08-18 | story-writer | OCSF-correctness Claroty SPEC pass-33 fix-burst: F-P33-MED-001 [MED] signature gap — `pipeline_result_to_record_batch` lacked `sensor_spec: &SensorSpec` as an explicit parameter (free-variable defect). Fix: (1) AC-012 added: new AC requiring the parameter addition with caller enumeration (1 production + 1 pre-existing test + 14 new story RG tests + 1 new RG-024); (2) RG-024 added: E0061 compile-fail Red Gate exercising both ocsf_column_naming=true and false branches from the threaded-parameter path; (3) T-11Q added: Phase A task to write RG-024; (4) T-14A added: Phase B task to add the parameter + update all 15 callers + TD-VSDD-060 grep confirmation; (5) T-14 updated: no longer claims to green RG-006 (now T-14A's responsibility); gains RG-024 full-green note; (6) AC-003 parameter-threading note added (sensor_spec is the threaded parameter from T-14A/AC-012, not a free variable); (7) §Architecture Mapping `pipeline_result_to_record_batch` row updated to note new parameter; (8) Architecture Compliance Rule 11 added: no placeholder construction of SensorSpec; (9) §Mandate Anchor discharge table: §D1 MUST row added; (10) T-GATE/T-19 updated: 23→24 RGTs, density 23/11→24/12=2.00; (11) ADR-058 pin v2.17→v2.18 at §Authority. §v1.32 Amendment Sweep added. Sibling: COERCION-001 ADR pin bump v2.17→v2.18 reported to orchestrator (not silently edited). |
| 1.31 | 2026-08-18 | story-writer | OCSF-correctness Claroty SPEC pass-32 fix-burst: F-P32-MED-001 [MED]: raw_extensions synthesis re-attributed to `pipeline_result_to_record_batch` (ADR-058 §I2) — AC-003 reconciled to clarified §I1+§I2 (`unwrap_or_else` fallback scoped to `ocsf_field == Some`; `ocsf_field == None` columns diverted by `pipeline_result_to_record_batch` before individual-field naming); AC-007 re-attributed; T-15 re-attributed; §Architecture Mapping `build_column_array raw_extensions handling` row replaced by `pipeline_result_to_record_batch §I2` row; §Purity Classification stale `build_column_array (raw_extensions path)` row removed; §File Structure Requirements `spec_driven_adapter.rs` row updated. ADR-058 pin v2.16→v2.17. Sibling coordination: COERCION-001 amended same burst (v1.29→v1.30 — F-P32-MED-002 SAP-3 annotations + ADR pin). §v1.31 Amendment Sweep added. |
| 1.30 | 2026-08-18 | story-writer | OCSF-correctness Claroty SPEC pass-31 records-only micro-burst (TD-VSDD-096): F-P31-LOW-001 [LOW, text-sync] — T-11P RG-023 location reworded from "Unit test in `class_selector.rs`" to "Integration test in `crates/prism-ocsf/tests/`", matching §File Structure Requirements (RG-011/012/023 row), §T-GATE, and T-19. `EventClassSelector::select` is `pub fn` — reachable from integration test crate (no E0603). Sibling sweep: COERCION-001 unaffected (no T-11P or class_selector tasks). §v1.30 Amendment Sweep added. |
| 1.29 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-30: BC-2.16.003 pin v1.12→v1.13 at §Authority and §Behavioral Contracts table (PO bump — §OCSF Field Validation Path-A/Path-B qualifier). Downstream contradiction check: AC-005/AC-010/RG-022 already use Interpretation A; no prose correction needed. Sibling coordination: COERCION-001 amended same burst (v1.28→v1.29). §v1.29 Amendment Sweep added. |
| 1.28 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-28 sibling coordination: BC-2.16.002 pin v2.27→v2.28 at §Authority entry and §Behavioral Contracts table (PO bumped BC-2.16.002 v2.28 with pending-wiring annotation on §Canonical Structured Event Catalog ocsf.unknown_class_name row). Sibling coordination: COERCION-001 amended same burst (v1.27→v1.28 — BC-2.16.002 pin v2.27→v2.28). §v1.28 Amendment Sweep added. |
| 1.27 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-26 fix-burst: ADR-058 pin v2.15→v2.16 at §Authority (architect bump — §H now cites AC-005/RG-006 for Path-A String+Object warn). Sibling coordination: COERCION-001 amended same burst (v1.25→v1.26 — F-P26-MED-001 RG-006 extended null+warn + pin sweep). §v1.27 Amendment Sweep added. |
| 1.26 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-25 fix-burst: ADR-058 pin v2.14→v2.15 at §Authority. BC-2.16.003 pin v1.11→v1.12 at §Authority and §Behavioral Contracts table. Sibling coordination: COERCION-001 amended same burst (v1.24→v1.25 — F-P25-MED-001 AC-005/T-15 add-Object-retain-wildcard + §Architecture Mapping `build_column_array` scope + pin sweeps). §v1.26 Amendment Sweep added. |
| 1.25 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-24 fix-burst: BC-2.16.003 pin v1.10→v1.11 (PO bump + EC-016-013-026 addition) at §Authority and §Behavioral Contracts body table. Sibling coordination: COERCION-001 amended same burst (v1.23→v1.24 — F-P24-HIGH-001 Object-only null-demote + RG-007 retirement + F-P24-MED-001 coerce_value signature + BC-2.16.003 pin). §v1.25 Amendment Sweep added. |
| 1.24 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-23 fix-burst: F-P23-MED-001 [MED, text-sync]: two loci corrected — (1) §Library & Framework Requirements tracing-test row: RG-018 location changed from `prism-bin/tests/` to `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`; (2) §File Structure Requirements Cargo.toml row Notes cell: RG-018 reference changed from `prism-bin/tests/` to `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`. Stale text was introduced in pass-20 (dependency-aware provisioning) before pass-21 relocation propagated to §Architecture Mapping, §T-GATE, and §File Structure prism-bin row but not to these two surfaces. Complete-sweep grep verified: zero `prism-bin/tests/` references for private-fn RGs remain; e2e test row for AC-008 (public surface) correctly retained. Sibling sweep: COERCION-001 amended in same burst (v1.22→v1.23 — F-P23-MED-001 §Library & Framework RG-009 location sync). §v1.24 Amendment Sweep added. |
| 1.23 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-22 fix-burst: (1) F-P22-MED-001 [MED, compile-correctness]: RG-013 re-routed from `crates/prism-ocsf/tests/` to `crates/prism-ocsf/src/mappers/spec_driven.rs` `#[cfg(test)] mod tests` — `set_nested_field` is a private free fn; `tests/` crate cannot reach it (E0603); §File Structure Requirements prism-ocsf row split into two rows (RG-011/012/023 in tests/, RG-013 in src/mappers/spec_driven.rs mod tests); note below table updated; T-GATE and T-19 crate attributions updated. (2) F-P22-MED-002 [MED, TDD gate coherence]: T-17 gains `just iter prism-bin` after `just iter prism-spec-engine` — six TOML-driven wire-shape RGs (RG-014..022 subset) reside in prism-bin; `just iter prism-spec-engine` alone cannot observe them. T-22 gains `just iter prism-bin` — RG-016/017 (class_uid wire-shape) reside in prism-bin; `just iter prism-ocsf` alone cannot observe them. (3) F-P22-OBS-2 [OBS, TDD gate coherence]: T-21 gains `just iter prism-bin` verify command so RG-009/010 are observable at T-21 (not only at terminal T-19). (4) F-P22-OBS-3 [OBS, authoring-accuracy]: T-22(c) "Both doc tables" → "all three doc tables" — `class_selector.rs` carries three class-name→uid doc tables (two module-level + the inline `select_by_class_name` doc table). (5) ADR-058 §Authority pin v2.13→v2.14 (architect bump; `anchor_stories` gain S-ADR058-DTU-PARITY-MIGRATION-001 + §H enumeration fix). Sibling sweep: COERCION-001 amended in same burst (v1.21→v1.22 — ADR-058 pin sweep). §v1.23 Amendment Sweep added. |
| 1.22 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-21 fix-burst: (1) F-P21-MED-001 [MED, compile-correctness]: §File Structure Requirements prism-bin unit-test row relocated from `crates/prism-bin/tests/ (unit test file — TBD at dispatch)` to `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests` — `pipeline_result_to_record_batch` and `ocsf_field_to_arrow_name` are module-private fns; `tests/` crate cannot reach them (E0603). All RG-003..006/008..010/014..022 are direct private-fn calls; note below table updated accordingly. (2) F-P21-LOW-001 [LOW, gate attribution]: T-12 makes-green updated: "RG-001, RG-002, and RG-006 green" → "RG-001 and RG-002 green" (`just iter prism-spec-engine` cannot observe prism-bin RG-006); T-14 updated: "(RG-006 already greened at T-12.)" → "(RG-006 confirmed at this just iter prism-bin run — causally greened by T-12)". `just iter` gate commands unchanged per finding. Sibling sweep: COERCION-001 amended in same burst (v1.20→v1.21 — F-P21-MED-001 `build_column_array` RGs relocated). §v1.22 Amendment Sweep added. |
| 1.21 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-20 fix-burst: (1) F-P20-MED-002 [MED, TDD gate coherence]: tracing-test dependency-aware provisioning — §Library & Framework Requirements tracing-test row updated from unconditional "NOT yet present; implementer MUST add" to "provided by S-ADR058-OCSF-COERCION-001 (upstream provider, merges first); VERIFY present, add ONLY if absent — do not create duplicate key"; §File Structure Requirements prism-bin/Cargo.toml row updated from "Modify: add tracing-test" to "Verify/Modify: confirm present (added by COERCION-001 for RG-009); add ONLY if absent"; §Library prose updated accordingly. (2) F-P20-LOW-001a [LOW, records-tier]: §TD-VSDD-097 Amendment Sweep subsections reordered to strict descending order (v1.21→v1.20→…→v1.1); prior non-monotonic order (v1.3,v1.2,v1.1,v1.5..v1.12,v1.18..v1.13,v1.20,v1.19) corrected. (3) F-P20-OBS-001 [OBS, SAP-3 reachability]: SAP-3 defense-in-depth rationale added to RG-013 — `set_nested_field` exercises Path B (`normalize_with_mappers`), zero production callers per ADR-058 §K5; defense-in-depth per SAP-3 rule 3; live Path A guarantee covered by RG-016. (4) F-P20-OBS-002 [OBS, records-tier]: RG-006 RED-reason corrected to compile-time failure (SensorSpec lacks ocsf_column_naming field, E0063); T-12 green-driver updated "Makes RG-001 and RG-002 green" → "Makes RG-001, RG-002, and RG-006 green"; T-14 green-driver updated "Makes RG-005 and RG-006 green" → "Makes RG-005 green. (RG-006 already greened at T-12.)". Sibling sweep: COERCION-001 amended in same burst (v1.19→v1.20 — F-P20-LOW-001b false-sibling-sentence fix + F-P20-LOW-002 Token Budget BC-2.02.011 row). §v1.21 Amendment Sweep added. |
| 1.20 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-19 fix-burst: (1) F-P19-MED-001 [MED, TDD gate coherence]: prism-bin `tracing-test` provisioning added — §Library & Framework Requirements gained a `tracing-test = "0.2"` row scoped to `prism-bin/Cargo.toml` `[dev-dependencies]` (required for RG-018 `tracing_test` subscriber in `prism-bin/tests/`; NOT yet present in prism-bin, mirrors `prism-spec-engine/Cargo.toml` existing dev-dep); "No new crate additions are anticipated" note replaced with accurate statement that one dev-dependency addition IS required; §File Structure Requirements added `crates/prism-bin/Cargo.toml` row (Modify: add `tracing-test = "0.2"` to `[dev-dependencies]`). Sibling sweep: COERCION-001 amended in same burst (v1.18→v1.19) — gains same prism-bin/Cargo.toml row plus T-12 filter fix (F-P19-LOW-001). §v1.20 Amendment Sweep added. |
| 1.19 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-18 fix-burst: (1) F-P18-MED-002 [MED, POL-8 / TDD gate coherence]: §File Structure Requirements expanded — added prism-mcp row (RG-007) and prism-ocsf row (RG-011..013, RG-023); prism-bin row corrected to RG-003..006/008..010/014..022 (removed RG-007 which belongs to prism-mcp). (2) F-P18-MED-003 [MED, TDD gate coherence]: RG-013 §Red Gate Tests rewritten from non-falsifiable ColumnMapper::map_record mechanism to correct DynamicMessage/set_nested_field mechanism per T-11F — builds DynamicMessage keyed by CLASS_UID_ENTITY_MANAGEMENT (3004), calls set_nested_field("comment", "reviewed"), asserts field IS set; contrasts with account_change (3001) where same call silently no-ops (data-loss contrast assertion is load-bearing). Routed to prism-ocsf. (3) F-P18-MED-004 [MED, POL-8 / TDD gate coherence]: RG-007 pinned to prism-mcp in §File Structure; `just iter prism-mcp --no-fail-fast` added to T-GATE; `just iter prism-mcp` added to T-19. T-GATE and T-19 now enumerate all four crates with explicit per-crate RG distribution. (4) F-P18-OBS-002 [OBS, no-change]: confirmed `device_type_label` is used in current-state AC-005 / §Claroty Contracted OCSF Mappings; no edit required; historical v1.2 snapshot grandfathered. Sibling sweep: COERCION-001 amended in same burst (v1.17→v1.18). §v1.19 Amendment Sweep added. |
| 1.18 | 2026-08-17 | story-writer | OCSF-correctness Claroty adversary SPEC pass-16 fix-burst: (1) F-P16-MED-001 [HIGH-floor, POL-7/POL-22]: BC-2.01.013 §Authority title corrected — "DataSource Trait Adapter Pattern" → "DataSource Trait Eliminates Per-Sensor Code Duplication" (authoritative H1 per BC-2.01.013 H1). (2) F-P16-OBS-001 [records-tier, POL-7]: BC-2.16.003 §Authority title expanded to full H1 verbatim ("Column-to-OCSF Mapping at Query Time" → "Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec"); BC-2.16.002 §Authority title expanded ("Multi-Step Fetch Pipeline Execution" → "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation"). (3) BC-2.16.003 pin propagated v1.9→v1.10 at both live current-state sites: §Authority entry + §Behavioral Contracts body table (PO bumped BC-2.16.003 to v1.10 with EC-016-013-025 addition in same burst; TD-VSDD-097 dim-2 swept). (4) input-hash updated 30215ef→e1c7cd2 (BC-2.16.003 input bumped by PO). Sibling sweep: COERCION-001 amended in same burst (v1.15→v1.16). §v1.18 Amendment Sweep added. |
| 1.17 | 2026-08-17 | story-writer | OCSF-correctness Claroty adversary SPEC pass-15 records-only micro-burst (TD-VSDD-096) — F1 [MED, records-tier]: stale BC-2.01.013 version pin v1.16→v1.23 corrected at 2 live current-state sites (§Authority entry + §Behavioral Contracts body table row); historical §Changelog v1.0 authoring-time row grandfathered/untouched. No substance change; frozen perimeter otherwise UNCHANGED. §v1.17 Amendment Sweep added. |
| 1.16 | 2026-08-17 | story-writer | Adversary pass-12 fix-burst: (1) F2 [LOW] §Tasks T-11G/H/L/M/N/O authoring-wording fixed — all six changed from "build a [table] SensorSpec with KF-xx corrections applied" to "load the corrected `claroty.sensor.toml` [table] table spec (post-T-17, KF-xx: ...)" — authoring instruction, RED-reason (TOML not yet corrected), and T-17 green-driver attribution now mutually consistent. (2) Comprehensive §Tasks audit: T-04–T-11 are consistent one-liners (no authoring body); T-11B/C (RG-009/010): inline spec + code-RED + T-21 green — CONSISTENT; T-11D/E/F (RG-011/012/013): direct API / DynamicMessage + code-RED + T-22/T-23 green — CONSISTENT; T-11I/J (RG-016/017): inline/optional-production-TOML + code-arm-RED + T-22 green — CONSISTENT; T-11K (RG-018): inline spec + code-RED + T-24 green — CONSISTENT; T-11P (RG-023): unit test + code-RED + T-23 green — CONSISTENT. (3) COERCION-001 tasks audit: CLEAN (all tasks code-level, no TOML dependency). (4) ADR-058 §Authority pin v2.12→v2.13 (concurrent architect bump). (5) Sibling sweep: zero normative prose version pins. (6) §v1.16 Amendment Sweep added. |
| 1.15 | 2026-08-17 | story-writer | Adversary pass-11 fix-burst: (1) ADR-058 §Authority pin v2.11→v2.12 (concurrent architect bump). (2) Sibling coordination: COERCION-001 LOW-2 AC-004 trace parentheticals added in same burst — all three COERCION frontmatter BCs now have AC `(traces to …)` parentheticals; ROUTING-001 already had BC-2.16.002 and BC-2.02.011 traces. (3) Sibling sweep: zero ADR-058/BC normative prose version pins found in either story outside exempt/grandfathered zones. (4) §v1.15 Amendment Sweep added. |
| 1.14 | 2026-08-17 | story-writer | Adversary pass-10 fix-burst: (1) F3 [LOW] §Mandate Anchor #1 provenance fix — both §D2 and §J2 discharge entries made version-free: removed "(v2.1)" / "since v2.1" version qualifiers from inline prose and table Status column; replaced with "DISCHARGED — ADR-058 §D2/§J2 carries the inline (Anchored: …) mark" form per POL-39. Eliminates drift source so story cannot go stale on future ADR bumps. (2) ADR-058 §Authority pin v2.10→v2.11 (concurrent architect bump). (3) Sibling sweep: zero additional normative prose ADR-058 or BC version pins found in either story outside §Authority (exempt) and historical amendment-sweep/changelog rows (grandfathered). (4) §v1.14 Amendment Sweep added. |
| 1.13 | 2026-08-17 | story-writer | Adversary pass-9 fix-burst: (1) F2 [LOW] AC-011 §Catalog obligation prose: removed volatile `v2.27` doc-version pin and stale '(product-owner authored it in this fix-burst)' temporal aside from normative AC prose; section-anchor cite `BC-2.16.002 §Canonical Structured Event Catalog` retained. (2) Sibling sweep — zero additional POL-39 doc-version pins found in normative prose of either story (body BC table and §Authority entries are exempt; historical amendment-sweep/changelog rows are grandfathered). (3) ADR-058 §Authority pin v2.9→v2.10 (concurrent architect bump). (4) §v1.13 Amendment Sweep added. |
| 1.12 | 2026-08-17 | story-writer | Adversary pass-8 fix-burst (comprehensive hygiene sweep): (1) F1 [HIGH] §Mandate Anchor #1 rewritten to mirror COERCION-001 §Mandate Anchor #2 — both §D2 and §J2 mandates marked DISCHARGED; stale 'ANCHOR-NEEDED' present-tense language removed; 'story does not yet exist' quote removed; unsatisfiable architect routing obligation removed; volatile ADR-058 version pins stripped from prose; Status column added to mandate-anchor table with DISCHARGED (v2.1). (2) F4 [LOW] §Authority §J4 description corrected: 'count correction 20→19' → 'count 31 pre-correction / 26 post-correction across four tables'. (3) F3 [LOW] v1.9 changelog row volatile-line-cite neutralized: the parenthetical containing a bare line number was replaced with '(Red-then-green gate instruction)'. (4) ADR-058 §Authority pin v2.8→v2.9 (concurrent architect bump). (5) §v1.12 Amendment Sweep added. |
| 1.11 | 2026-08-17 | story-writer | Adversary pass-7 fix-burst (full task-plan audit): (1) F1 gate ordering fixed: T-19 (run all 23 RGTs) + T-20 (just check) moved to AFTER T-24 — terminal gates are now the final two tasks. (2) F2(a) T-17 mis-attribution corrected: RG-009/RG-010 removed from T-17 "Makes green" list (those are code collision-detection unit tests greened by T-21, not by TOML edit). (3) F2(b) T-17 missing green-drivers added: RG-019 (KF-11 audit_logs.category→raw_extensions) and RG-020 (KF-07 device_alert_relations finding_info_uid) added to T-17 "Makes green" list. RG-023 added to T-23 "Makes green" list. (4) Full RG→green-driver matrix verified: all 23 RGs mapped to exactly one task. (5) COERCION-001 task-audit: CLEAN. (6) §v1.11 Amendment Sweep added. |
| 1.10 | 2026-08-17 | story-writer | Adversary pass-6 fix-burst: (1) F1 T-11H corrected: old name `test_claroty_alerts_id_produces_finding_info_uid_arrow_field_wire_shape` (single KF-03) → new name `test_claroty_alerts_finding_info_fields_wire_shape` (3-field: KF-03 `finding_info_uid` + KF-04 `finding_info_title` + KF-12 `finding_info_modified_time`); body updated to 3-field record and assertions. (2) F2 T-11P fabricated API fixed: `select(&ClassSelectorInput { vendor: "claroty", class_name: "audit_log", ... })` → `select("claroty", "audit_log")`; sibling-sweep: zero other `ClassSelectorInput`/`vendor:`/`class_name:` occurrences in either story. (3) F3 §Authority date cites ADR-058 + BC-2.16.003 updated 2026-08-16 → 2026-08-17 in both stories; `modified:` frontmatter field added as 2026-08-17 in both stories. (4) §v1.10 Amendment Sweep added. |
| 1.9 | 2026-08-17 | story-writer | Adversary pass-5 fix-burst: (1) ADR-058 re-pin v2.7→v2.8; BC-2.16.003 re-pin v1.8→v1.9 (concurrent architect/PO bumps); §Authority pins and body BC table updated in both ROUTING-001 and COERCION-001. (2) F2 stale count fix: `all 20 confirmed failing` → `all 23 confirmed failing` (Red-then-green gate instruction). Full grep of both stories for other stale `20`/`twenty` RG-count refs — zero additional instances found in normative sections. (3) §v1.9 Amendment Sweep added. |
| 1.8 | 2026-08-17 | story-writer | Adversary pass-4 fix-burst (comprehensive KF→AC→RG coverage-matrix audit): (1) BC-2.16.003 re-pin v1.7→v1.8. (2) F5 POL-39 volatile-pin strip: `v1.67` catalog-label and `row 94` positional cite removed from §Authority BC-2.16.002, body BC table, AC-011, RG-018; durable `event_type = "ocsf.unknown_class_name"` symbol anchor retained. (3) F2 RG-015 expanded to 3-field wire-shape (KF-03 `finding_info_uid` + KF-04 `finding_info_title` + KF-12 `finding_info_modified_time`); AC-010 assertion 1 now matches RG-015 reality. (4) F3 RG-021 added: KF-05 `audit_logs.id` → raw_extensions (no `activity_uid`/`id` Arrow field; value preserved in raw_extensions); RG-022 added: KF-06 `devices.device_type` → `device_type_label` Arrow field (demo-critical `WHERE device_type_label = 'PLC'`). AC-010 assertions 5 and 6 added. (5) F4 RG-023 added: AC-009(c) Claroty `select()` arm `("claroty","audit_log")` → Ok(3004); density-note corrected (RG-012 covers Armis half; RG-023 covers Claroty half). (6) T-11N/T-11O/T-11P added for RG-021/022/023 authoring; T-GATE/T-19 updated. (7) Density 20/11=1.82→23/11=2.09. AC-005 density-note corrected (KF-05/06 now have RGs). (8) §v1.8 Amendment Sweep added. |
| 1.7 | 2026-08-17 | story-writer | Adversary pass-3 fix-burst: (1) F2 compile-error-class fix — AC-011 and T-24 emission snippet `%table.name` → `%table.table_name` (`TableSpec` has no `name` field; correct field is `table_name` per `prism-spec-engine::spec_parser`; sibling sweep confirmed no other stale `table.name` references in COERCION-001). (2) F1 ROUTING-001 subsystem cross-check against ARCH-INDEX — SS-01/02/10/16 confirmed correct; justification prose already correct (prism-bin attributed to SS-10; prism-sensors+prism-spec-engine to SS-01; prism-ocsf to SS-02; prism-spec-engine to SS-16). (3) §v1.7 Amendment Sweep added. |
| 1.6 | 2026-08-16 | story-writer | Adversary pass-2 fix-burst: (1) F2 pin sweep — ADR-058 §Authority pin v2.6→v2.7; BC-2.16.003 §Authority pin v1.6→v1.7; body BC table v1.6→v1.7; narrative version labels stripped per POL-39 (section-anchor-only cites in RG-013/014/015, AC-005, AC-009, AC-010, AC-011, EC-003, T-11I/T-11J/T-24). (2) F3 wire-shape coverage added: RG-019 `test_claroty_audit_logs_record_batch_kf11_category_in_raw_extensions` (KF-11 audit_logs category→raw_extensions + entity_management field mappings); RG-020 `test_claroty_device_alert_relations_record_batch_finding_info_uid_wire_shape` (KF-07 device_alert_relations alert_id→finding_info_uid). AC-010 updated with RG-019/020 assertions; T-11L/T-11M added. Density 18/11=1.64→20/11=1.82. RG section header 15→20. T-19/T-GATE updated. (3) F4 SS-01 justification imprecision: removed `prism-bin::spec_driven_adapter` from SS-01 prose; moved to SS-10 prose (prism-bin is SS-10 per ARCH-INDEX; SS-01 justified by prism-sensors + prism-spec-engine only). (4) §v1.6 Amendment Sweep added. |
| 1.5 | 2026-08-16 | story-writer | ADR-058 v2.6 + BC-2.16.003 v1.6 + BC-2.16.002 v2.27 propagation (adversary pass-1 fix-burst). (1) Subsystems: [SS-07, SS-12, SS-16] → [SS-01, SS-02, SS-10, SS-16]; removed fabricated SS-07 ("Spec Engine" — SS-07 is Adapter Pagination & Response Cache per ARCH-INDEX) and SS-12 ("Sensor Adapters / DTU" — SS-12 is Scheduler per ARCH-INDEX); added SS-02 (OCSF Normalization, owns prism-ocsf/class_selector.rs) and SS-10 (MCP Interface, owns prism-mcp/prism_describe.rs); rewrote justification comments with correct ARCH-INDEX registry citations. (2) AC-009 REWORKED: 3 code changes → 4 sub-obligations per ADR-058 §I5: (a) const; (b) TWO new select_by_class_name arms: entity_management→3004 AND inventory_info→5001 (prevents devices regression 5001→0); (c) BOTH select() audit_log arms (claroty+armis) ACCOUNT_CHANGE→ENTITY_MANAGEMENT forward-compat; (d) deprecate-annotate "audit_activity" dead arm; plus in-file doc table update obligation. (3) RG-011 REWORKED: was assert select_by_class_name("audit_activity")==Ok(3004) — now asserts select_by_class_name("entity_management")==Ok(3004) AND select_by_class_name("inventory_info")==Ok(5001); "audit_activity" is dead code post-KF-01 TOML fix; test name updated accordingly. (4) RG-016 ADDED: wire-shape integration test — audit_logs RecordBatch class_uid Int32 == 3004 (NOT 3001, NOT 0); traces to BC-2.16.003 v1.6 EC-016-013-023. (5) RG-017 ADDED: regression-prevention wire-shape — devices RecordBatch class_uid == 5001 (NOT 0); traces to BC-2.16.003 v1.6 EC-016-013-024. (6) AC-011 ADDED: process-gap warn obligation — ocsf.unknown_class_name WARN on Err branch before .unwrap_or(0); traces to BC-2.16.002 v2.27 catalog row 94. (7) RG-018 ADDED: tracing_test assertion for ocsf.unknown_class_name event. (8) T-22/T-23 REWORKED to 4 sub-obligations; T-24 ADDED for warn emission. (9) T-11I/T-11J/T-11K ADDED for RG-016/017/018 authoring. (10) Density: 15/10=1.5 → 18/11=1.64. (11) BC-2.16.002 v2.27 added to behavioral_contracts frontmatter (POL-8: body BC table row, §Authority entry, Token Budget count). (12) ADR-058 v2.5→v2.6, BC-2.16.003 v1.5→v1.6. (13) TD-VSDD-097 v1.5 sweep added. |
| 1.4 | 2026-08-16 | story-writer | Consistency-validator fix-burst: MED-001 + LOW-001 + ADR-058 §K pin sweep. (1) AC-006: corrected stale pre-KF-03 examples: name=`"finding_uid"`→`"finding_info_uid"`, description=`"finding.uid"`→`"finding_info.uid"`, LLM agent example `name: "finding_uid"`→`name: "finding_info_uid"`. (2) Frontmatter risk comment: `SELECT finding_uid`→`SELECT finding_info_uid`. (3) §Authority ADR-058 pin: v2.4→v2.5. (4) Narrative prose ADR-058 v2.4 references converted to section-anchor form (ADR-058 §K4, §K5, §I5 — no version per POL-39). (5) TD-VSDD-097 v1.3 sweep section + v1.3 changelog row: v2.4 version label removed from record prose per TD-VSDD-091. |
| 1.3 | 2026-08-16 | story-writer | BC-2.16.003 v1.5 + ADR-058 §K downstream copy propagation (TD-VSDD-097 dim-2). (1) AC-005 fully rewritten: all four Claroty contracted tables per BC-2.16.003 v1.5 §Claroty Contracted OCSF Mappings (alerts, audit_logs, devices, device_alert_relations); KF-01..KF-12 TOML corrections enumerated including KF-06 PO decision device.type_label (was device.type_name), KF-05 PO decision audit_logs.id→raw_extensions; alerts table stale 5-row excerpt replaced with 11-column authoritative table; devices table device.type_name corrected to device.type_label (Arrow: device_type_label). (2) AC-009 added: class_selector.rs KF-01 code fix — add CLASS_UID_ENTITY_MANAGEMENT = 3004, reroute "audit_activity" arm + Armis ("armis","audit_log") arm (TD-VSDD-097 dim-1 sibling per ADR-058 §K5 Div-3). (3) AC-010 added: wire-shape assertions for corrected finding_info.* fields (finding_info_uid, finding_info_title, finding_info_modified_time) and reserved-metadata columns in raw_extensions (KF-08/09/10/11). (4) RG-011..RG-015 added: class_selector audit_activity→entity_management, Armis sibling, note→comment data-loss prevention, reserved-fields raw_extensions wire-shape, finding_info_uid wire-shape. (5) Density check updated: 15 RGTs / 10 ACs = 1.5 ≥ 0.5. (6) Authority section: BC-2.16.003 v1.4→v1.5 + ADR-058 v2.1→v2.4, §K added to reading list. (7) Narrative: finding_uid→finding_info_uid. (8) Behavioral Contracts table: BC-2.16.003 v1.4→v1.5. (9) EC-003 corrected: audit_logs.username maps to actor.user.uid (not actor.user.name; that is user_display_name). (10) EC-009 count corrected: 19→31 pre-corrections (four tables: alerts 9+audit_logs 8+devices 8+dar 6), post-corrections 26. (11) crates_touched: added prism-ocsf. (12) Architecture Mapping: class_selector.rs row added. (13) Token Budget: class_selector.rs ~2k added. (14) Tasks: T-11D..T-11H (Red Gate authoring for RG-011..RG-015), T-17 expanded to KF-01..KF-12 full TOML corrections, T-22 (class_selector.rs "audit_activity" arm), T-23 (Armis arm), T-GATE updated to include prism-ocsf. (15) File Structure: class_selector.rs row. (16) TD-VSDD-097: v1.3 amendment sweep added. |
| 1.2 | 2026-08-12 | story-writer | ADR-058 v2.1 §J amendment discharge. (1) RG-010 added: `test_pipeline_result_to_record_batch_ocsf_shadow_collision_returns_error` — fails until shadow check (flattened ocsf_field name ≠ different column's col.name in same table) is implemented; includes mandatory self-match exclusion assertion (`A ≠ B` guard) covering the legal `risk_score → risk_score` and `status → status` Claroty cases. (2) T-11C added (red-gate authoring task for RG-010; preserves red-then-green ordering). (3) T-21 extended: shadow check clause (b) added to the combined collision-detection pass; `A ≠ B` self-match exclusion specified. (4) EC-010 added: flag-transition name shadowing defect class. (5) AC-005 TOML scope extended: `device_category` ocsf_field changed from `"device.type"` to `"device.type_category"` in the same TOML edit as `ocsf_column_naming = true`; devices table post-fix Arrow names documented (`device_uid`, `device_instance_uid`, `device_type_category`, `device_type_name`, `risk_score`, `status_code`). (6) Architecture Compliance Rule 10 added: self-match exclusion obligation. (7) ADR-058 MUST Discharge second row added for §J2 mandate → RG-010. (8) Authority section updated to ADR-058 v2.1; §J1–§J4 sections added to reading list. (9) TD-VSDD-097 three-dimension verdict updated for this amendment: S-ADR058-OCSF-COERCION-001 swept (clear); S-ADR058-DTU-PARITY-MIGRATION-001 swept — no devices Arrow name invalidated (parity tests not yet written; depend on this story; will read post-amendment TOML at dispatch time). Density updated 9/8 = 1.125 → 10/8 = 1.25. |
| 1.1 | 2026-08-12 | story-writer | Remove-uncertainty pass: Q2 CORRECTED — Arrow 58 silently first-matches on duplicate field names; added EC-009 (intra-table flattening collision), RG-009 (collision detection test), T-11B (red gate for RG-009), T-21 (fail-closed collision check in `pipeline_result_to_record_batch`), Architecture Compliance Rules 8 and 9. Q3(c) CORRECTED — T-12 updated to name all three edit sites (`SensorSpec` struct, `impl Default`, `SensorSpec::new()`). Q1 CONFIRMED — AC-002 strengthened with Arrow 58 field-name basis, DataFusion SQL identifier preconditions, and PrismQL-lexer guarantee. Density updated 8/8 → 9/8 = 1.125. |
| 1.0 | 2026-08-12 | story-writer | Initial authorship — ADR-058 Stage 2 story. Discharges ADR-058 v2.0 §D2 ANCHOR-NEEDED mandate for ocsf_column_naming MUST (AC-001/RG-001/RG-002). Explicit scoping decision: ColumnMapper::map_record wiring gap stays out of scope; BC-2.01.013 EC-01-025 resolves via Arrow schema naming change per ADR-058 §B2 item 4. 8 ACs, 8 RGTs (density 1.0). BC-2.16.003 v1.4, BC-2.01.013 v1.16, ADR-058 v2.0 at authoring time. |
