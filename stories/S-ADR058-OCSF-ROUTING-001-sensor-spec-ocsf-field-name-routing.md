---
document_type: story
story_id: S-ADR058-OCSF-ROUTING-001
title: "ADR-058 Stage 2 — OCSF Field-Name Routing: ocsf_column_naming Flag, Underscore-Flattened Arrow Names, Claroty Activation"
version: "1.37"
level: "L4"
status: draft
producer: story-writer
timestamp: "2026-08-12T00:00:00Z"
modified: "2026-08-18"
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
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.003
  - BC-2.16.002
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
input-hash: "0a58dc4"
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

**ADR-058 v2.21: v1 Column Naming — OCSF Field-Path Routing with Underscore-Flattened Arrow
Names; DTU Migration Deferred.** Version `2.21`, status: accepted (2026-08-18). Read
§B2 (decision), §C (quoting convention — Option 4 chosen), §D (per-sensor scoping, flag
mechanism — **§D1 corrected v2.18: `pipeline_result_to_record_batch` MUST gain
`sensor_spec: &SensorSpec` as an explicit parameter threaded from the `fetch()` call site;
this is ADR-022 §C wiring (adding a previously absent parameter), not redesign**),
§E (blast radius), §G (prism_describe output spec — **v2.21 Tier-1/Tier-2 model:
Tier-1 columns (`ocsf_field == Some`) emit ColumnDescriptor with
`name = ocsf_field_to_arrow_name(ocsf_field)` and `description = ocsf_field`;
Tier-2 columns (`ocsf_field == None`) MUST NOT emit individual ColumnDescriptors —
instead `prism_describe` MUST emit exactly ONE `raw_extensions` ColumnDescriptor with
four-field shape: `name = "raw_extensions"`, `col_type = prism_core::column::ColumnType::Json`,
`nullable = true`, and `description` identifying it as a JSON object and enumerating every
`ocsf_field == None` column's `col.name` as a source key**),
§H (Stage 1 confirmed
separate), §I (implementation guidance including **§I1 corrected v2.18: two-step form —
Step 1 signature addition (`sensor_spec: &SensorSpec` parameter), Step 2 field-name
computation inside the function body**; **§I1 corrected v2.21: canonical home of
`ocsf_field_to_arrow_name` is `prism-spec-engine::column_mapping` (NOT `prism-bin::spec_driven_adapter`);
both `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe` import from there**; **§I5 TOML + code correction obligations for
KF-01 through KF-12**; **§I5 process-gap obligation: `ocsf.unknown_class_name` WARN on
Err branch before `.unwrap_or(0)` in `pipeline_result_to_record_batch`; Path A / Path B
liveness determination; `select_by_class_name` two new arms: `"entity_management"→3004` and
`"inventory_info"→5001`; `"audit_activity"` arm becomes dead code pending deprecation annotation**),
**§J1–§J4 (flag-transition name shadowing adjudication, normative fail-closed rule, Claroty
`devices` table resolution, `ocsf_field` count 31 pre-correction / 26 post-correction across four tables)**, and **§K (OCSF v1.7.0
schema validation — §K4 finding summary KF-01..KF-12, §K5 divergence adjudication including
class_selector.rs KF-01 code defect confirmed and Armis sibling sweep)** in full before
implementing.
Path: `.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`.

**BC-2.16.003: Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec.** Version `1.15`, status: draft
(modified 2026-08-18). §Column Routing postconditions, **§Claroty Contracted OCSF Mappings
(ground truth for all four Claroty tables with KF-01..KF-12 corrections)**, and
**§Interpretation A: Arrow Field Naming** govern the obligation that `ocsf_field` declarations
produce queryable Arrow field identifiers. **EC-016-013-023** (KF-01 entity_management class_uid
= 3004 wire-level postcondition) and **EC-016-013-024** (KF-02 inventory_info class_uid = 5001
regression-prevention) are the authoritative wire-shape obligations for AC covering class_uid
Arrow column values. **EC-016-013-027** (Tier-1/Tier-2 `prism_describe` model per
§Interpretation A v1.15: `ocsf_field == None` columns MUST NOT appear as individual
ColumnDescriptor names; `prism_describe` MUST emit exactly ONE `raw_extensions`
ColumnDescriptor per table with four-field shape `name = "raw_extensions"`,
`col_type = prism_core::column::ColumnType::Json`, `nullable = true`, and `description`
enumerating all `ocsf_field == None` source keys) is the
authoritative obligation for AC-006 Tier-2 prohibition and AC-007b `prism_describe`
`raw_extensions` ColumnDescriptor emission. This story brings the production path into
conformance with those postconditions for Claroty.
Path: `.factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md`.

**BC-2.16.002: Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation.** Version `2.28`, status: active
(modified 2026-08-17). Canonical Structured Event Catalog — `ocsf.unknown_class_name`
WARN — emitted by `pipeline_result_to_record_batch` on the `Err` branch of
`EventClassSelector::select_by_class_name` before `.unwrap_or(0)`. Fields: `ocsf_class: %display`,
`sensor_id: %display`, `table_name: %display`. SAP-1 / PG-LP11-001 obligation: the implementer
MUST add this `tracing::warn!` emission to `pipeline_result_to_record_batch` in the same commit
as the `select_by_class_name` arm additions. This is the source for AC-011 in this story.
Path: `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md`.

**BC-2.01.013: DataSource Trait Eliminates Per-Sensor Code Duplication.** Version `1.23`, status: active.
EC-01-025 records "ColumnMapper step is missing" as NON-CONFORMANT. Stage 2 resolves
EC-01-025 for Claroty per ADR-058 §B2 item 4 (OCSF field names now appear in the Arrow
schema for the flagged sensor).
Path: `.factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md`.

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
(Anchored: S-ADR058-OCSF-ROUTING-001 EC-010 / T-21 / RG-010) mark. No architect action required.

The mandate anchor records:

| MUST Statement | Story | AC | Red Gate Test | Status |
|---|---|---|---|---|
| `ocsf_column_naming: bool` field MUST be added to `SensorSpec` with `#[serde(default)]` (ADR-058 §D2) | S-ADR058-OCSF-ROUTING-001 | AC-001 | RG-001, RG-002 | DISCHARGED |
| `pipeline_result_to_record_batch` MUST check, when `ocsf_column_naming == true`, that no flattened `ocsf_field` name equals a DIFFERENT column's `col.name` in the same table (`A ≠ B` exclusion), fail-closed (ADR-058 §J2) | S-ADR058-OCSF-ROUTING-001 | EC-010, T-21 (shadow check extension) | RG-010 | DISCHARGED |
| `pipeline_result_to_record_batch` MUST gain `sensor_spec: &SensorSpec` as an explicit parameter threaded from the `fetch()` call site in `spec_driven_adapter.rs`; no placeholder construction (ADR-058 §D1 v2.18, ADR-022 §C wiring) | S-ADR058-OCSF-ROUTING-001 | AC-012 | RG-024 | DISCHARGED |
| `prism_describe` MUST NOT emit an individual ColumnDescriptor for `ocsf_field == None` columns when `ocsf_column_naming = true`; MUST emit exactly ONE `raw_extensions` ColumnDescriptor with four-field shape: `name = "raw_extensions"`, `col_type = prism_core::column::ColumnType::Json`, `nullable = true`, and `description` identifying it as a JSON object and enumerating every `ocsf_field == None` column's `col.name` as a source key (ADR-058 §G v2.21; BC-2.16.003 EC-016-013-027 / §Interpretation A v1.15) | S-ADR058-OCSF-ROUTING-001 | AC-006 (Tier-2), AC-007b | RG-025 | DISCHARGED |
| `ocsf_field_to_arrow_name` MUST live in `prism-spec-engine::column_mapping`; both `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe` import it from there (no cycle); placing it in `prism-bin` is FORBIDDEN — `prism-mcp` is Level 6 in the topological ordering and `prism-bin` is Level 7 (`dependency-graph.md` §Dependency Rules Rule 2: lower-layer crates never depend on higher-layer crates); a `prism-mcp → prism-bin` edge would violate this rule (ADR-058 §I1 v2.21) | S-ADR058-OCSF-ROUTING-001 | AC-002 | RG-003, RG-004 | DISCHARGED |

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
| BC-2.16.003 | v1.15 | draft | §Column Routing postconditions, §Claroty Contracted OCSF Mappings (ground truth — KF-01..KF-12 corrections for all four tables), §Interpretation A: Arrow Field Naming — `ocsf_field` declarations produce queryable Arrow field identifiers; EC-016-013-023 (audit_logs class_uid = 3004 wire-level) and EC-016-013-024 (devices class_uid = 5001 regression-prevention); EC-016-013-027 (Tier-1/Tier-2 `prism_describe` model: no individual ColumnDescriptor for `ocsf_field == None` columns; exactly one `raw_extensions` ColumnDescriptor with four-field shape: name + col_type=Json + nullable=true + description enumerating source keys) |
| BC-2.16.002 | v2.28 | active | Canonical Structured Event Catalog `ocsf.unknown_class_name` WARN — fields `ocsf_class`, `sensor_id`, `table_name`; SAP-1/PG-LP11-001 obligation on implementer to add the warn emission in the same commit as the `select_by_class_name` arm additions (AC-011) |
| BC-2.01.013 | v1.23 | active | EC-01-025 NON-CONFORMANT annotation resolved for Claroty after this story merges; product-owner updates annotation |

---

## Red Gate Tests (SAC-1 — tdd_mode: strict)

All twenty-five tests MUST be failing (RED) before any implementation code is written.
Test-writer dispatched FIRST; implementer only after all 25 confirmed failing.

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
  `prism-spec-engine::column_mapping` (ADR-058 §I1 v2.21 canonical home; NOT
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
  in Claroty devices (confirmed: `uid`, `asset_id`, `device_category`, `device_type`,
  `retired` — no match). Without this assertion, an over-broad implementation that
  checks `ocsf_field_to_arrow_name(A) ≠ A.col_name` (no `A ≠ B` guard) would reject
  valid production Claroty config. Covers EC-010.

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

- **RG-021:** `test_claroty_audit_logs_id_column_goes_to_raw_extensions_not_activity_uid` —
  wire-shape assertion for KF-05 (PO decision: `audit_logs.id` `ocsf_field` REMOVED). Fails
  until the KF-05 TOML correction (removing `ocsf_field` from `audit_logs.id`) is applied.
  With the corrected TOML and `ocsf_column_naming = true`, materializes an `audit_logs`
  RecordBatch via `pipeline_result_to_record_batch` with a record `{id: "al-999",
  action: "Login", note: "reviewed"}`. Asserts on serialized JSON: (1) no top-level Arrow
  field named `"activity_uid"` or `"id"` exists (the `id` value does NOT become an
  `activity_uid` Arrow field); (2) the `raw_extensions` JSON blob contains key `"id"` with
  value `"al-999"` (audit record ID preserved in raw_extensions as the deduplication
  reference per PO decision); (3) Arrow field `"activity_name"` contains `"Login"` (the
  `action` → `activity_name` mapping under entity_management 3004 remains correct).
  Covers AC-010 assertion 5 (KF-05).
  Traces to BC-2.16.003 §Claroty Contracted OCSF Mappings (audit_logs table, KF-05).

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
  Traces to ADR-058 §D1 (v2.18): `pipeline_result_to_record_batch` MUST gain `sensor_spec`
  as an explicit parameter threaded from `fetch()`; traces to ADR-022 §C: wiring not redesign.

- **RG-025:** `test_prism_describe_ocsf_column_naming_true_raw_extensions_descriptor_and_no_phantom_col_names` —
  fails until `prism_describe` implements the Tier-1/Tier-2 model per ADR-058 §G v2.21 /
  BC-2.16.003 §Interpretation A v1.15 EC-016-013-027.

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
       (ADR-058 §G v2.21; ADR-024 canonical ColumnType variant for JSON payloads);
  (v) the `raw_extensions` ColumnDescriptor has `nullable = true`
      (ADR-058 §G v2.21 / BC-2.16.003 §Interpretation A v1.15 — a table with zero
      `ocsf_field == None` columns produces no `raw_extensions` entry; queries must not
      fail when the column is absent from a specific row).

  **RED condition:** Prior to the fix, `prism_describe` emits one ColumnDescriptor per column
  using the pre-Tier-2 model — it emits ColumnDescriptors with `name = "category"` and
  `name = "alert_type_name"` as phantom names (assertion (i) fails), emits no
  `"raw_extensions"` ColumnDescriptor (assertion (ii) fails with count = 0), and emits no
  four-field shape (assertions (iii)-(v) all fail). Without the fix, all five assertions fail.

  Covers AC-006 Tier-2 prohibition and AC-007b `raw_extensions` ColumnDescriptor emission
  (full four-field shape). Traces to ADR-058 §G v2.21 (Tier-2 MUST NOT emit individual;
  MUST emit `raw_extensions` ColumnDescriptor with `col_type = Json`, `nullable = true`, and
  description enumerating source keys) and BC-2.16.003 EC-016-013-027 / §Interpretation
  A v1.15 (POL-38 mandate anchor).

### BC-5.38.001 Density Check

Red Gate test count: **25** (RG-001..RG-025).
Acceptance criteria: 12 (AC-001..AC-012). AC-008 is an `#[ignore]`'d test update — its
Red Gate is RG-005 (same mechanism: Arrow field name must be `device_uid` not `uid`).

Density: 25 RGTs / 12 ACs = **2.08 ≥ 0.5** — compliant with BC-5.38.001.

Note: AC-005 (claroty.sensor.toml — ocsf_column_naming + KF-01..KF-12 full corrections)
is validated by wire-shape RGs that require the corrected TOML to pass: RG-014..RG-022
collectively assert the TOML corrections at the RecordBatch level. KF-05 is asserted by
RG-021; KF-06 is asserted by RG-022. AC-008 (e2e test update) has no standalone failing
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
RG-020 covers AC-010 assertion 4 (KF-07). RG-021 covers AC-010 assertion 5 (KF-05).
RG-022 covers AC-010 assertion 6 (KF-06).
RG-024 covers AC-012 (`pipeline_result_to_record_batch` gains `sensor_spec: &SensorSpec`
parameter; both `ocsf_column_naming = true` and `ocsf_column_naming = false` branches
exercised from the threaded-parameter path).
RG-025 covers AC-006 Tier-2 prohibition (no phantom ColumnDescriptor for `ocsf_field == None`
columns) and AC-007b `raw_extensions` ColumnDescriptor four-field shape emission; traces to
ADR-058 §G v2.21 / BC-2.16.003 §Interpretation A v1.15 EC-016-013-027 / POL-38 mandate anchor.
The density check is based on the 25 distinct failing tests enumerated above.

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
`prism-spec-engine::column_mapping` (ADR-058 §I1 v2.21 canonical home). Both
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

**Parameter threading note (ADR-058 §D1 v2.18):** `sensor_spec` in the snippet below is
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

**14 TOML changes enumerated (KF-01..KF-12 + flag + §J3 shadow fix):**

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

6. **KF-05 (PO decision)**: `audit_logs` table, `id` column: remove `ocsf_field`.
   PO decision: `activity_uid` is absent from OCSF v1.7.0; `activity_id` is a numeric
   enum, not a UID; audit record ID preserved in `raw_extensions` as deduplication reference.

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
| `id` | (none — removed) | `raw_extensions` | KF-05 PO decision |
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

Post-corrections ocsf_field count: 26 (alerts: 6, audit_logs: 6, devices: 8, dar: 6).
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
tiers per ADR-058 §G v2.20 / BC-2.16.003 §Interpretation A v1.14:

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

(traces to BC-2.16.003 §Interpretation A v1.14 Tier-1 model: `ocsf_field == Some` columns
emit ColumnDescriptor with `name = ocsf_field_to_arrow_name(ocsf_field)`;
ADR-058 §G v2.20: Tier-2 prohibition — `prism_describe` MUST NOT emit an individual
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
the synthesis locus for the `raw_extensions` aggregation. `build_column_array` is a
per-column value function that receives only columns already included in the schema and
structurally cannot suppress a column from the schema or aggregate multiple columns into
a blob. `pipeline_result_to_record_batch` suppresses `ocsf_field == None` columns from
the individual-field schema and aggregates their values into the `"raw_extensions"` Utf8
blob. The implementer MUST verify which Claroty columns currently have
`col.ocsf_field == None` in `claroty.sensor.toml` at dispatch time and confirm they
go to `raw_extensions` rather than being silently dropped.

**AC-007b — MCP tool (`prism_describe`):** Under `ocsf_column_naming = true`,
`prism_describe` MUST emit exactly ONE `raw_extensions` ColumnDescriptor per
ADR-058 §G v2.21 / BC-2.16.003 §Interpretation A v1.15 EC-016-013-027 with all four fields:
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
BC-2.16.003 §Interpretation A v1.15 and EC-016-013-027: `prism_describe` MUST emit
exactly ONE `raw_extensions` ColumnDescriptor with four-field shape
(name + col_type=Json + nullable=true + description enumerating source keys) (AC-007b);
ADR-058 §G v2.21 (AC-007b); RG-025 (falsifiable Red Gate for AC-007b))

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

Tests covering the KF-03/04/05/06/07/08/09/10/11/12 corrections MUST assert on the
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

5. **KF-05 `audit_logs.id` → `raw_extensions`** (RG-021): A Claroty `audit_logs` record
   with `id = "al-999"` processed through `pipeline_result_to_record_batch` with
   `ocsf_column_naming = true` produces JSON where:
   - No top-level Arrow field named `"activity_uid"` or `"id"` exists (PO decision:
     `audit_logs.id` ocsf_field removed; value does NOT become `activity_uid`)
   - The `raw_extensions` JSON blob contains key `"id"` with value `"al-999"`
     (audit record ID preserved in raw_extensions as deduplication reference)

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
SAP-1 standing probe obligation)

### AC-012: pipeline_result_to_record_batch gains sensor_spec: &SensorSpec as an explicit threaded parameter

`pipeline_result_to_record_batch` in `prism-bin::spec_driven_adapter` gains a new explicit
parameter `sensor_spec: &SensorSpec` per ADR-058 §D1 (v2.18). This is ADR-022 §C wiring:
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
(14 — Phase A Red Gate tests; all produce E0061 until T-14A adds the parameter):**
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

**New test caller (1):**
- RG-024 (`test_pipeline_result_to_record_batch_sensor_spec_parameter_gates_both_branches`)

**Call-site confirmation (TD-VSDD-060):** Before committing, implementer runs
`rg 'pipeline_result_to_record_batch' crates/prism-bin/ crates/prism-mcp/` to confirm
no additional callers exist outside this enumeration. The function is not `pub` — it is
`prism-bin`-internal; no callers in other crates are expected.

(traces to ADR-058 §D1 v2.18: `pipeline_result_to_record_batch` MUST gain `SensorSpec` as
an explicit parameter threaded from `fetch()`; traces to ADR-022 §C: wiring not redesign —
adding a previously absent parameter is in-scope plumbing)

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Scope |
|-----------|--------|---------------|-------|
| `SensorSpec::ocsf_column_naming` field | `prism-spec-engine::spec_parser` | Pure (data struct) | New field added |
| `ocsf_field_to_arrow_name` | `prism-spec-engine::column_mapping` | Pure | New free function — no I/O, deterministic string transform; canonical home per ADR-058 §I1 v2.21; imported by both `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe` |
| `pipeline_result_to_record_batch` | `prism-bin::spec_driven_adapter` | Effectful (Arrow I/O) | New parameter `sensor_spec: &SensorSpec` threaded from `fetch()` (ADR-058 §D1 v2.18 / ADR-022 §C wiring); conditional branch on `sensor_spec.ocsf_column_naming` |
| `pipeline_result_to_record_batch` `raw_extensions` aggregation (§I2) | `prism-bin::spec_driven_adapter` | Effectful (Arrow I/O) | New path (ADR-058 §I2): when `ocsf_column_naming = true`, columns with `ocsf_field == None` are suppressed from individual Arrow schema fields and aggregated into a single `"raw_extensions"` Utf8 column; synthesis locus is `pipeline_result_to_record_batch` (schema-fields construction), NOT `build_column_array` |
| `prism_describe` | `prism-mcp::tools::prism_describe` | Effectful (MCP response) | Tier-1/Tier-2 model per ADR-058 §G v2.20: Tier-1 (`ocsf_field == Some`) → ColumnDescriptor with `name = ocsf_field_to_arrow_name(ocsf_field)` and `description = ocsf_field`; Tier-2 (`ocsf_field == None`) → NO individual ColumnDescriptor emitted; exactly ONE `raw_extensions` ColumnDescriptor emitted per table enumerating all `ocsf_field == None` source keys (col.names) |
| `claroty.sensor.toml` | `prism-sensors/specs/` | Configuration | Add `ocsf_column_naming = true` + all KF-01..KF-12 corrections + §J3 shadow fix (14 TOML changes per AC-005) |
| `class_selector.rs` | `prism-ocsf/src/` | Pure (lookup table) | Add `CLASS_UID_ENTITY_MANAGEMENT = 3004`; reroute `"audit_activity"` arm + Armis `("armis","audit_log")` arm to entity_management (3004) per AC-009 |
| `#[ignore]`'d e2e test | `crates/prism-bin/tests/` | Test (effectful) | Update `row.get("uid")` → `row.get("device_uid")` |

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
| EC-009 | Two columns in the same table have `ocsf_field` values that flatten to the same Arrow name (e.g., `"a.b_c"` and `"a_b.c"` both → `"a_b_c"` via `ocsf_field_to_arrow_name`) | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed. Arrow 58 does NOT detect duplicate schema field names (`Schema::new` is infallible; `Schema::column_with_name` returns the first match — silent wrong-column resolution for the agent). Current Claroty TOML has no intra-table collision (verified by enumeration: 31 ocsf_field values pre-corrections across four tables — alerts: 9, audit_logs: 8, devices: 8, device_alert_relations: 6; post-KF corrections: 26 — alerts: 6, audit_logs: 6, devices: 8, dar: 6; ADR-058 §J4). Future sensors must be collision-free before enabling the flag. See RG-009. |
| EC-010 | A flattened `ocsf_field` name from one column equals the `col.name` of a DIFFERENT column in the same table when `ocsf_column_naming = true` (flag-transition name shadowing per ADR-058 §J1/§J2). Example: `device_category` with `ocsf_field = "device.type"` → `device_type`, while column `device_type` has `col.name = "device_type"`. `SELECT device_type FROM claroty_devices` is valid in both flag states but returns different semantic content — high-level category vs type-within-category — with no error and no warning. | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed. The `A ≠ B` self-match exclusion is mandatory: a column whose flattened ocsf_field name equals its own `col.name` (e.g., `risk_score` → `risk_score`) is legal and MUST NOT fail. This collision class is resolved in `claroty.sensor.toml` by changing `device_category`'s ocsf_field from `"device.type"` to `"device.type_category"` (AC-005, same TOML edit). See RG-010. |
| EC-016-013-027 | `prism_describe` emits individual ColumnDescriptors for `ocsf_field == None` columns (phantom queryable names) when `ocsf_column_naming = true`, or emits no `raw_extensions` ColumnDescriptor, or emits a `raw_extensions` ColumnDescriptor that fails to enumerate source keys. Pre-fix: agent calls `prism_describe` for a Claroty table, sees `category` as a ColumnDescriptor name, writes `SELECT category FROM claroty_alerts`, gets no data because `category` is aggregated into `raw_extensions` under the OCSF routing model — silent semantic failure at the agent/query interface. | `prism_describe` MUST NOT emit any ColumnDescriptor with `name` equal to an `ocsf_field == None` column's `col.name`; MUST emit exactly ONE ColumnDescriptor with `name = "raw_extensions"` whose description enumerates all `ocsf_field == None` col.name values as source keys. Tested by RG-025 (three-assertion: phantom prohibition (i), count exactly 1 (ii), source-key enumeration (iii)). Traces to AC-006 Tier-2, AC-007b, ADR-058 §G v2.20, BC-2.16.003 §Interpretation A v1.14. |

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
| BC-2.16.003 + BC-2.16.002 + BC-2.01.013 (governing contracts — 3 BCs) | ~6.5k |
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
  v2.21 canonical home; NOT in prism-bin).
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
- T-11N: Write RG-021 — `test_claroty_audit_logs_id_column_goes_to_raw_extensions_not_activity_uid`
  (MUST FAIL). Wire-shape assertion: load the corrected `claroty.sensor.toml` audit_logs
  table spec (post-T-17, KF-05: `id.ocsf_field` removed). Pass a record `{"id": "al-999",
  "action": "Login"}` through `pipeline_result_to_record_batch` with
  `ocsf_column_naming = true` and `ocsf_class = "entity_management"`. Serialize to JSON.
  Assert: (1) no top-level Arrow field named `"activity_uid"` exists; (2) no top-level
  Arrow field named `"id"` exists; (3) `raw_extensions` JSON blob contains key `"id"` with
  value `"al-999"`. Currently fails because KF-05 TOML correction not yet applied.
  Covers AC-010 (KF-05).
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
  with the correct four-field shape. Covers AC-006 Tier-2 and AC-007b; traces to ADR-058 §G
  v2.21 / BC-2.16.003 §Interpretation A v1.15 EC-016-013-027.
- T-GATE: Run `just iter prism-spec-engine --no-fail-fast`, `just iter prism-bin --no-fail-fast`,
  `just iter prism-mcp --no-fail-fast`, and `just iter prism-ocsf --no-fail-fast` — confirm
  RG-001..RG-025 fail with correct compile/test-failure reasons (RG-001..004 in
  prism-spec-engine; RG-005..006/008..010/014..022/024 in prism-bin; RG-007 and RG-025 in
  prism-mcp; RG-011/012/023 in prism-ocsf/tests/; RG-013 in prism-ocsf/src/mappers/spec_driven.rs
  mod tests). Confirm no regressions in non-RG tests. Report density:
  25/12 = 2.08 ≥ 0.5. STOP and wait for implementer dispatch.

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
  `crates/prism-spec-engine/src/column_mapping.rs` (ADR-058 §I1 v2.21 canonical home —
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
  (a) the 14 NEW Red Gate test callers (RG-005, RG-006, RG-008, RG-009, RG-010, RG-014 through
  RG-022) — pass a synthetic `SensorSpec` matching the test's assertion intent;
  (b) the 1 pre-existing test caller (`test_BC_2_01_013_crowdstrike_fql_datetime_index_col_string_equality_safe`)
  — pass a `SensorSpec` with `ocsf_column_naming = false` to preserve current CrowdStrike behavior.
  **TD-VSDD-060 call-site sweep:** Before committing, run
  `rg 'pipeline_result_to_record_batch' crates/prism-bin/ crates/prism-mcp/` to confirm
  no callers outside the enumerated 17 (1 production + 1 pre-existing test + 14 new RG tests + 1 new RG-024).
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
  This is schema-construction logic in `pipeline_result_to_record_batch`, NOT
  `build_column_array` — `build_column_array` is a per-column value function that cannot
  suppress columns or aggregate multiple columns into a blob. Run `just iter prism-bin`.
  Makes RG-008 green.
- T-16: Update `prism_describe` per the Tier-1/Tier-2 model in ADR-058 §G v2.21 /
  BC-2.16.003 §Interpretation A v1.15 EC-016-013-027:
  (a) **Tier-1** (`ocsf_field == Some`): emit ColumnDescriptor with
      `name = ocsf_field_to_arrow_name(ocsf_field)` and `description = ocsf_field`.
  (b) **Tier-2 prohibition** (`ocsf_field == None`): MUST NOT emit an individual
      ColumnDescriptor for the column — skip it entirely from the per-column iteration.
  (c) **raw_extensions ColumnDescriptor**: after processing all columns, if
      `ocsf_column_naming = true` AND at least one column has `ocsf_field == None`,
      emit exactly ONE additional ColumnDescriptor with the FOUR-FIELD SHAPE:
      - `name = "raw_extensions"`
      - `col_type = prism_core::column::ColumnType::Json` (ADR-058 §G v2.21; ADR-024)
      - `nullable = true` (ADR-058 §G v2.21 / BC-2.16.003 §Interpretation A v1.15)
      - `description` = a string identifying it as a JSON object and enumerating every
        `ocsf_field == None` column's `col.name` as a source key (e.g.,
        `"JSON object containing vendor fields not mapped to OCSF: category, alert_type_name, devices_count, alert_class, ot_devices_count"`)
  Run `just iter prism-mcp`. Makes RG-007 green (Tier-1 path) and RG-025 green (Tier-2
  prohibition + `raw_extensions` four-field ColumnDescriptor emission).
- T-17: Apply all 14 TOML changes to `claroty.sensor.toml` in a single edit per AC-005:
  (1) `ocsf_column_naming = true` at sensor level;
  (2) KF-01: `audit_logs.ocsf_class` = `"entity_management"`;
  (3) KF-02: `devices.ocsf_class` = `"inventory_info"`;
  (4) KF-03: `alerts.id.ocsf_field` = `"finding_info.uid"`;
  (5) KF-04: `alerts.alert_name.ocsf_field` = `"finding_info.title"`;
  (6) KF-05: `audit_logs.id.ocsf_field` removed;
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
  RG-015, RG-019, RG-020, RG-021, RG-022 green (KF-03/04/05/06/07/08/09/10/11/12
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

  This combined pass is a fail-closed gate per ADR-058 §J2. Arrow 58 does NOT detect
  either class of collision; without this check, a shadow collision produces silent
  wrong-column resolution for every query in that flag state. Run `just iter prism-bin`. Makes RG-009 and RG-010
  green.
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
- T-19: Run `just iter prism-spec-engine`, `just iter prism-bin`, `just iter prism-mcp`, and
  `just iter prism-ocsf` — all 25 RGTs must pass (RG-001..004 in prism-spec-engine;
  RG-005..006/008..010/014..022/024 in prism-bin; RG-007 and RG-025 in prism-mcp;
  RG-011/012/023 in prism-ocsf/tests/; RG-013 in prism-ocsf/src/mappers/spec_driven.rs
  mod tests).
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
   §I1 v2.21). Both `prism-bin::spec_driven_adapter` and `prism-mcp::tools::prism_describe`
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
| `crates/prism-spec-engine/src/column_mapping.rs` | Create/Modify: add `pub fn ocsf_field_to_arrow_name(ocsf_field: &str) -> String` (ADR-058 §I1 v2.21 canonical home); add RG-003..RG-004 to `#[cfg(test)] mod tests` block |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify: import `ocsf_field_to_arrow_name` from `prism_spec_engine::column_mapping` (NOT defined here); update `pipeline_result_to_record_batch` (individual-field naming per ADR-058 §I1 + `ocsf_field == None` → raw_extensions aggregation per ADR-058 §I2); `build_column_array` raw_extensions path NOT added — §I2 aggregation is schema-construction logic in `pipeline_result_to_record_batch`, not a per-column value path |
| `crates/prism-mcp/src/tools/prism_describe.rs` | Modify: `ColumnDescriptor.name` sourcing branches on `sensor_spec.ocsf_column_naming` |
| `crates/prism-sensors/specs/claroty.sensor.toml` | Modify: apply all 14 TOML changes per AC-005 (ocsf_column_naming flag + KF-01..KF-12 + §J3 shadow fix — all in one edit) |
| `crates/prism-ocsf/src/class_selector.rs` | Modify: add `CLASS_UID_ENTITY_MANAGEMENT = 3004`; reroute `"audit_activity"` arm and `("armis","audit_log")` arm to entity_management (3004) per AC-009 |
| `crates/prism-bin/tests/` (e2e test file — TBD at dispatch) | Modify: update `test_BC_2_11_005_e2e_claroty_query_returns_data` assertion |
| `crates/prism-spec-engine/tests/` (new or existing test file) | Modify: add RG-001..RG-002 |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify: add `use prism_spec_engine::column_mapping::ocsf_field_to_arrow_name;` import; add RG-005..RG-006, RG-008..RG-010, RG-014..RG-022, RG-024 to `#[cfg(test)] mod tests` block (direct calls to `pipeline_result_to_record_batch` and imported `ocsf_field_to_arrow_name` — no public API surface expansion; RG-003..004 moved to prism-spec-engine/column_mapping.rs) |
| `crates/prism-mcp/tests/` (test file — TBD at dispatch) | Modify: add RG-007 |
| `crates/prism-ocsf/tests/` (new or existing test file) | Modify: add RG-011, RG-012, RG-023 |
| `crates/prism-ocsf/src/mappers/spec_driven.rs` (`#[cfg(test)] mod tests` block) | Modify: add RG-013 (calls private `set_nested_field` — unreachable from `tests/` crate; E0603 if placed in integration test) |
| `crates/prism-bin/Cargo.toml` | Verify/Modify: confirm `tracing-test = "0.2"` is present in `[dev-dependencies]` (added by S-ADR058-OCSF-COERCION-001 for RG-009); add ONLY if absent — do not duplicate | Required for RG-018 `tracing_test` subscriber in `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`; COERCION-001 is the upstream provider (depends_on ordering) |

Implementer MUST add private-fn RGs (RG-005..006/008..010/014..022/024) to the `#[cfg(test)] mod tests` block in `crates/prism-bin/src/spec_driven_adapter.rs` — do NOT place them in `crates/prism-bin/tests/` (separate crate; cannot reach private fns). Similarly, RG-013 calls `set_nested_field`, a private free function in `crates/prism-ocsf/src/mappers/spec_driven.rs`; route RG-013 to the `#[cfg(test)] mod tests` block of that file, NOT to `crates/prism-ocsf/tests/` (E0603 if placed in the integration test crate). For the e2e test update (AC-008), verify file names via `find crates/prism-bin/tests -name "*.rs"` at dispatch.

Do NOT modify: any other sensor TOML spec (CrowdStrike, Armis, Cyberint); any BC or ADR body (product-owner / architect scope). Note: `column_mapping.rs` is IN SCOPE — create/modify `crates/prism-spec-engine/src/column_mapping.rs` per T-13. `class_selector.rs` is in scope for this story (AC-009 code obligation).

---

## Forbidden Dependencies

Build-time enforcement rules:

- `prism-spec-engine` MUST NOT import from `prism-bin`. If `cargo tree -p prism-spec-engine` shows `prism-bin` after this story, a forbidden import was introduced.

- `prism-mcp` MUST NOT import from `prism-bin`. The `ocsf_field_to_arrow_name` helper MUST live in `prism-spec-engine::column_mapping` (ADR-058 §I1 v2.21) so both `prism-bin` and `prism-mcp` can import it without a forbidden edge. `prism-mcp` is Level 6 and `prism-bin` is Level 7 in the crate topological ordering (`dependency-graph.md` §Dependency Rules Rule 2); a `prism-mcp → prism-bin` dependency is forbidden because lower-layer crates never depend on higher-layer crates. If `cargo tree -p prism-mcp` shows `prism-bin` after this story, the helper was placed in the wrong crate.

- `prism-sensors` MUST NOT gain a dependency on `prism-spec-engine`. If `cargo tree -p prism-sensors` shows `prism-spec-engine`, the story introduced a forbidden import.

- `prism-bin` MUST NOT gain any new `native-tls` features. Verify `Cargo.toml` reqwest entries if any are modified.

---

## TD-VSDD-097 / POL-29 Three-Dimension Sweep Verdict

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
