---
document_type: story
story_id: S-WAVE-A-ARMIS-SPEC-001
title: "Armis Sensor Spec — Six Column Additions and device_cves_first source_path Fix"
version: "1.6"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P1
points: 8
tdd_mode: strict
target_module: prism-sensors
subsystems: ["SS-06 (SensorSpec)", "SS-07 (SpecEngine)", "SS-12 (DTU-Armis)"]
crates_touched:
  - prism-sensors      # armis.sensor.toml modification
  - prism-spec-engine  # Red Gate tests parse and assert the TOML spec
  - prism-dtu-armis    # generator::build_asset §build_asset: add five keys for generated-records path parity (AC-008..AC-012)
depends_on: []
blocks: []
behavioral_contracts:
  - BC-2.02.006
verification_properties: []
assumption_validations: []
risk_mitigations: []
estimated_days: 3
modified: "2026-07-29"
# BC status: BC-2.02.006 v1.15 §TOML Contract + §Generated-Records Path Coverage specify
# all 15 ACs and 15 RGTs; status may transition to ready once Red Gate tests are authored
# and BC-2.02.006 is confirmed active.
---

# S-WAVE-A-ARMIS-SPEC-001: Armis Sensor Spec — Six Column Additions and device_cves_first source_path Fix

## Authority

**ADR-023 v1.24 §Rule 1** (Spec-Driven OCSF Mapping) is the authority for the `ocsf_field`
TOML column annotation contract. ADR-023 Rule 1 establishes that OCSF field mapping is
declarative via TOML — each `SensorSpec.columns[N].ocsf_field` annotation drives the
`SpecDrivenMapper` without a Rust mapper module. The six new columns added by this story
must carry `ocsf_field` annotations following ADR-023 Rule 1's closed grammar:
`.factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md`

**ADR-028 v1.28 §D1** (TOML Spec Grounding vs DTU Routes) is the authority for the
DTU column parity requirement. ADR-028 §D1 establishes that every TOML column must be
grounded in a corresponding DTU struct field that is emitted on the wire (SAP-2 protocol).
All six new columns are verified against `DeviceRecord` in `prism-dtu-armis/src/types.rs`
and are emitted by `routes::search::get_search` (canonical pipeline-facing handler for
`from armis.devices` queries):
`.factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md`

---

## Narrative

As a Prism maintainer, I want `crates/prism-sensors/specs/armis.sensor.toml` to declare six
additional device columns (`os_version`, `risk_factors`, `network_id`, `site`, `tags`,
`device_cves`) and to fix the `device_cves_first` column entry with `source_path =
"$.device_cves[0]"` — so that SOC agents querying `from armis.devices` receive OS version,
risk factor labels, network context, site, analyst tags, and full CVE arrays rather than
silent absent columns.

---

## Background

SAP-2 probe F-SAP2-MED-005 (FB68d) identified that six `DeviceRecord` fields exist in the
DTU (`prism-dtu-armis/src/types.rs`) and are emitted on the wire by
`routes::search::get_search` via `serde_json::to_value(&merged)` on the full `DeviceRecord`
struct (canonical pipeline-facing handler for `from armis.devices` queries,
`GET /api/v1/search?aql=<AQL>`),
but have no corresponding `[[tables.columns]]` entries in `armis.sensor.toml`.
The `SpecDrivenMapper` cannot extract or map fields that have no TOML column declaration.

F-WASE-P66-HIGH-004 identified that `device_cves_first` has no `DeviceRecord` field in
`prism-dtu-armis/src/types.rs`. Without `source_path = "$.device_cves[0]"`, the spec-engine
cannot derive `device_cves_first` from the `device_cves` array. Behavior across four branches
(HIGH-002 correction; BC-2.02.006 v1.15 §Generated-Records Path Coverage):
(a) **static-fixture path**: `serde_json::to_value(&merged)` on `DeviceRecord` omits the key
(no such field); (b) **generated scenario branch, stage ≥ 4** (`mask.device_cves == true`):
`device_cves_first` is RETAINED — this is the load-bearing T13 NVD/CVSS pivot value; the
`obj.remove("device_cves_first")` call is conditional on `!mask.device_cves` and is NOT
reached at stages ≥ 4; (c) **generated scenario branch, stages 0–3** (`mask.device_cves ==
false`): `obj.remove("device_cves_first")` fires and strips the key per BC-2.06.019 PC-4;
(d) **seeded-no-scenario branch** (`timeline.is_none()`): no injection and no removal; key is
absent. The `source_path` directive tells the spec-engine to extract the first element of the
`device_cves` JSON array — this works correctly end-to-end once `device_cves` is populated on
the generated path per the HIGH-001 adjudication (see AC-007 / T-02).

---

## Acceptance Criteria

### AC-001: `os_version` column declared in `armis.sensor.toml` devices table
(traces to BC-2.02.006 §TOML Contract postcondition — `os_version` exposed as TOML column)

`armis.sensor.toml` `devices` table declares an `os_version` column with
`column_type = "string"` and `ocsf_field = "device.os.version"`.

DTU ground truth: `DeviceRecord.os_version: Option<String>` in
`prism-dtu-armis/src/types.rs`; emitted by `routes::search::get_search` via
`serde_json::to_value(&merged)`.

### AC-002: `risk_factors` column declared in `armis.sensor.toml` devices table
(traces to BC-2.02.006 §TOML Contract postcondition — `risk_factors` exposed as TOML column)

`armis.sensor.toml` `devices` table declares a `risk_factors` column with
`column_type = "json"` and `ocsf_field = "raw_extensions.risk_factors"`.

DTU ground truth: `DeviceRecord.risk_factors: Vec<String>` in
`prism-dtu-armis/src/types.rs`; `Vec<String>` serializes as a JSON array of strings;
emitted by `routes::search::get_search` via `serde_json::to_value(&merged)`.

### AC-003: `network_id` column declared in `armis.sensor.toml` devices table
(traces to BC-2.02.006 §TOML Contract postcondition — `network_id` exposed as TOML column)

`armis.sensor.toml` `devices` table declares a `network_id` column with
`column_type = "string"` and `ocsf_field = "raw_extensions.network_id"`.

DTU ground truth: `DeviceRecord.network_id: Option<String>` in
`prism-dtu-armis/src/types.rs`; emitted by `routes::search::get_search` via
`serde_json::to_value(&merged)`.

### AC-004: `site` column declared in `armis.sensor.toml` devices table
(traces to BC-2.02.006 §TOML Contract postcondition — `site` exposed as TOML column)

`armis.sensor.toml` `devices` table declares a `site` column with
`column_type = "string"` and `ocsf_field = "raw_extensions.site"`.

DTU ground truth: `DeviceRecord.site: Option<String>` in
`prism-dtu-armis/src/types.rs`; emitted by `routes::search::get_search` via
`serde_json::to_value(&merged)`.

### AC-005: `tags` column declared in `armis.sensor.toml` devices table
(traces to BC-2.02.006 §TOML Contract postcondition — `tags` exposed as TOML column)

`armis.sensor.toml` `devices` table declares a `tags` column with
`column_type = "json"` and `ocsf_field = "raw_extensions.tags"`.

DTU ground truth: `DeviceRecord.tags: Vec<String>` in
`prism-dtu-armis/src/types.rs`; `Vec<String>` serializes as a JSON array of strings;
emitted by `routes::search::get_search` via `serde_json::to_value(&merged)` (merged with
per-org `tag_store` at query time per BC-3.2.001).

### AC-006: `device_cves` column declared in `armis.sensor.toml` devices table
(traces to BC-2.02.006 §TOML Contract postcondition — `device_cves` exposed as TOML column)

`armis.sensor.toml` `devices` table declares a `device_cves` column with
`column_type = "json"` and `ocsf_field = "raw_extensions.device_cves"`.

DTU ground truth: `DeviceRecord.device_cves: Vec<String>` in
`prism-dtu-armis/src/types.rs`; `Vec<String>` serializes as a JSON array of strings;
emitted by `routes::search::get_search` via `serde_json::to_value(&merged)`. Complements
the existing `device_cves_first` scalar column.

### AC-007: `device_cves_first` column entry carries `source_path = "$.device_cves[0]"`
(traces to BC-2.02.006 §TOML Contract postcondition — `device_cves_first` derives its
value from the `device_cves` array via JSONPath extraction; EC-02-013 remedy; HIGH-001
adjudication specifies the generator obligation that makes this work end-to-end)

The existing `device_cves_first` column entry in the `armis.sensor.toml` devices table is
amended to add `source_path = "$.device_cves[0]"`. With this directive, the spec-engine
extracts the first element of the `device_cves` JSON array as the value for this column.

**HIGH-001 design adjudication (BC-2.02.006 v1.15 §TOML Contract):** `source_path =
"$.device_cves[0]"` is the correct and complete mechanism for `device_cves_first` resolution
on all paths, once the generator obligation below is met. The following four invariants MUST
hold simultaneously:

1. **T13 NVD/CVSS pivot preserved:** `device_cves_first` = `catalog_device_cves[0]` on
   `CompromisedEndpoint` at stage ≥ 4. The generator `§generate_with_scenario_cves` MUST
   stamp `device_cves = catalog_device_cves` (the FULL array from the scenario catalog, not
   an implementer-chosen array) onto `CompromisedEndpoint` records. `source_path` then
   extracts `device_cves[0]` = `catalog_device_cves[0]`. Pivot is preserved via the array,
   not via a direct scalar stamp.
2. **Pivot selectivity preserved:** `device_cves_first` is absent/null on non-scenario records
   and when the catalog is empty, so `has device_cves_first` correctly yields 0 results. All
   non-`CompromisedEndpoint` archetypes MUST carry `device_cves = []` on the generated path;
   `source_path` extraction of `[][0]` = null.
3. **Path agreement:** static-fixture and generated paths agree on key presence and value shape.
   `device_cves_first` is absent from the static-fixture path (no `DeviceRecord` field) —
   consistent with the generated seeded-no-scenario branch (no stamping, key absent).
4. **Stage-gate compliance (BC-2.06.019 PC-4):** both `device_cves` and `device_cves_first`
   MUST be stripped from device records alongside each other in BOTH `§paginate_devices` and
   `§get_search` when `!mask.device_cves` (stages 0–3). Anchor: AC-013 / RG-013.

**Mechanism correction (HIGH-002):** The `obj.remove("device_cves_first")` call is NOT
unconditional. It fires only on the generated scenario sub-branch at stages 0–3 when
`!mask.device_cves`; at stages ≥ 4 the key is RETAINED as the load-bearing pivot value. On
the seeded-no-scenario branch, no injection or removal occurs — key is absent.

**Generator obligation (see T-02 Part B):** `§generate_with_scenario_cves` MUST stamp
`device_cves = catalog_device_cves` (full array) on `CompromisedEndpoint` records, and
`device_cves = []` on all other archetypes. The previous direct scalar stamp
`device_cves_first = catalog_device_cves[0]` becomes REDUNDANT once `source_path =
"$.device_cves[0]"` is declared and `device_cves` carries the catalog array. The implementer
MUST remove the direct scalar stamp to prevent dead code and potential double-key confusion.

Post-fix: when `device_cves = ["CVE-2024-1234", "CVE-2024-5678"]`, `device_cves_first` =
`"CVE-2024-1234"`. When `device_cves = []`, `device_cves_first` = null.

The `device_cves_first` entry is DISTINCT from the `device_cves` column (AC-006). Both are
required: `device_cves` for full CVE array context, `device_cves_first` for enrichment UDF
scalar input (ADR-051 D4).

### AC-008: `os_version` key present on `get_search §get_search` generated-records path
(traces to BC-2.02.006 §Generated-Records Path Coverage — `os_version` MUST be emitted by
`build_asset §build_asset` on the `fixture_gen_seeded=true` path; contracted option (a))

`GET /api/v1/search?aql=in:devices` against an `ArmisState` with `fixture_gen_seeded=true`
and at least one asset record returns a serialized JSON device record containing the
`"os_version"` key. Assertion MUST be on the **serialized JSON response** (wire-shape
assertion per CLAUDE.md §Wire-shape assertion discipline). Key presence is the contracted
obligation; exact value is implementer-determined.

DTU defect: `build_asset §build_asset` in `prism-dtu-armis::generator` currently lacks an
`os_version` key. Fix requires T-03 in this story.

### AC-009: `risk_factors` key present on `get_search §get_search` generated-records path
(traces to BC-2.02.006 §Generated-Records Path Coverage — `risk_factors` MUST be emitted by
`build_asset §build_asset` on the `fixture_gen_seeded=true` path; contracted option (a))

`GET /api/v1/search?aql=in:devices` against an `ArmisState` with `fixture_gen_seeded=true`
returns a serialized JSON device record containing the `"risk_factors"` key. Wire-shape
assertion on the serialized response required (same pattern as AC-008). Key presence
contracted; `[]` for healthy-device archetypes is acceptable per BC-2.02.006.

### AC-010: `network_id` key present on `get_search §get_search` generated-records path
(traces to BC-2.02.006 §Generated-Records Path Coverage — `network_id` MUST be emitted by
`build_asset §build_asset` on the `fixture_gen_seeded=true` path; contracted option (a))

`GET /api/v1/search?aql=in:devices` against an `ArmisState` with `fixture_gen_seeded=true`
returns a serialized JSON device record containing the `"network_id"` key. Wire-shape
assertion on the serialized response required. Suggested implementer value:
`format!("net-{}", id_index % 10)` deterministic string per BC-2.02.006.

### AC-011: `tags` key present on `get_search §get_search` generated-records path
(traces to BC-2.02.006 §Generated-Records Path Coverage — `tags` MUST be emitted by
`build_asset §build_asset` on the `fixture_gen_seeded=true` path; contracted option (a))

`GET /api/v1/search?aql=in:devices` against an `ArmisState` with `fixture_gen_seeded=true`
returns a serialized JSON device record containing the `"tags"` key. Wire-shape assertion
on the serialized response required. `[]` (empty JSON array) for fresh generated devices
is acceptable per BC-2.02.006.

### AC-012: `device_cves` key present on `get_search §get_search` generated-records path
(traces to BC-2.02.006 §Generated-Records Path Coverage — `device_cves` MUST be emitted by
`build_asset §build_asset` on the `fixture_gen_seeded=true` path; contracted option (a))

`GET /api/v1/search?aql=in:devices` against an `ArmisState` with `fixture_gen_seeded=true`
returns a serialized JSON device record containing the `"device_cves"` key. Wire-shape
assertion on the serialized response required. `[]` for healthy-device archetypes is
acceptable per BC-2.02.006; non-empty for `CompromisedEndpoint` archetype.

### AC-013: `device_cves` stripped at stage 0 in both routes (CRIT-001)
(traces to BC-2.02.006 §TOML Contract HIGH-001 adjudication constraint 4 — `device_cves`
MUST be stripped alongside `device_cves_first` in BOTH `§paginate_devices` and `§get_search`
when `StageMask.device_cves == false`; BC-2.06.019 PC-4 compliance)

`GET /api/v1/devices` and `GET /api/v1/search?aql=in:devices` against an `ArmisState` with
`StageMask.device_cves == false` (stages 0–3) MUST return serialized device records with the
`"device_cves"` key **absent**. This is a wire-level assertion (CLAUDE.md §Wire-shape
assertion discipline) — assert on serialized JSON, not a pre-serialization Rust struct. Both
routes MUST be asserted independently in the same test: CRIT-001 requires the strip to occur
in BOTH `GET /api/v1/devices` AND `GET /api/v1/search?aql=in:devices`.

A `device_cves` array present at stages 0–3 while `device_cves_first` is withheld leaks the
full CVE list in violation of BC-2.06.019 PC-4.

Anchor: RG-013 (`test_armis_dtu_device_cves_stripped_at_stage_0_in_both_routes`).

### AC-014: `SchemaDrift` archetype `records[0]` carries all five contracted column keys (MED-003)
(traces to BC-2.02.006 §Generated-Records Path Coverage three-emitter table —
`generate_schema_drift §generate_schema_drift` inline `drifted` record MUST carry all five
contracted column keys with type-compatible values)

`GET /api/v1/search?aql=in:devices` against an `ArmisState` that serves the `SchemaDrift`
archetype returns `records[0]` (the inline `drifted` record) with all five contracted column
keys — `"os_version"`, `"risk_factors"`, `"network_id"`, `"tags"`, `"device_cves"` — present
with non-null values. Wire-shape assertion on the serialized response required.

Per the `P12-01` comment in `generate_schema_drift §generate_schema_drift`, the intended
drift is ONLY the missing `id` key. Populating these columns is the intended behavior, not a
drift violation. `records[0]` for the `SchemaDrift` archetype IS the `drifted` record —
the first device returned by `GET /api/v1/search?aql=in:devices`. Without this AC, Red Gate
outcomes from RG-008..RG-012 become archetype-dependent: passing for standard archetypes and
failing for `SchemaDrift`.

Anchor: RG-014 (`test_armis_dtu_schema_drift_archetype_has_contracted_columns`).

### AC-015: `HighChurn` tombstone records emit `[]` for `Vec<String>` columns (MED-004)
(traces to BC-2.02.006 §Generated-Records Path Coverage three-emitter table —
`build_tombstone §build_tombstone` MUST emit `risk_factors = []`, `tags = []`,
`device_cves = []`; `Vec<String>` fields MUST NOT emit `null`)

`GET /api/v1/search?aql=in:devices` against an `ArmisState` with `HighChurn` archetype
returns at least one tombstone record with `"risk_factors": []`, `"tags": []`,
`"device_cves": []` at the wire level — `[]` not `null`. Wire-shape assertion on the
serialized response required (CLAUDE.md §Wire-shape assertion discipline). `"site": null`
remains correct because `site` is `Option<String>`.

`null` is type-incompatible with `Vec<String>` fields. The static-fixture path serializes
`DeviceRecord.risk_factors`, `tags`, and `device_cves` as `[]` when empty — the generated
path MUST match to satisfy wire-shape parity.

Anchor: RG-015 (`test_armis_dtu_tombstone_records_vec_columns_emit_empty_array`).

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Architecture Section |
|-----------|--------|---------------|----------------------|
| `armis.sensor.toml` (6 new columns + 1 source_path fix) | `crates/prism-sensors/specs/` | Pure (config data) | `architecture/module-decomposition.md §SS-06 SensorSpec` |
| Red Gate tests RG-001..RG-007 (parse and assert TOML spec) | `crates/prism-spec-engine/tests/` | Pure (test only) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `build_asset §build_asset` (add five generated-records keys) | `crates/prism-dtu-armis/src/generator.rs` | Pure (record construction) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |
| `generate_schema_drift §generate_schema_drift` (add five columns to inline `drifted` record) | `crates/prism-dtu-armis/src/generator.rs` | Pure (record construction) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |
| `build_tombstone §build_tombstone` (add `Vec<String>` columns as `[]`) | `crates/prism-dtu-armis/src/generator.rs` | Pure (record construction) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |
| Red Gate tests RG-008..RG-015 (wire-shape assertions on generated-records path, stage-gate, emitter completeness) | `crates/prism-dtu-armis/tests/` | Pure (test only) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |

---

## Behavioral Contracts

| BC | Version | Relevance to This Story |
|----|---------|------------------------|
| BC-2.02.006 | v1.15 | Armis Centrix Field Mapping to OCSF — §TOML Contract specifies all 6 new columns and the `device_cves_first` source_path fix; §TOML Contract HIGH-001 adjudication specifies the generator obligation (`§generate_with_scenario_cves` stamps full `catalog_device_cves`, removes redundant scalar stamp) and the four simultaneous constraints; §Postconditions specifies per-field OCSF mappings and CRIT-001 `device_cves` stage-gate obligation (AC-013/RG-013); §Generated-Records Path Coverage (three-emitter table) specifies all 8 generated-records path MUSTs (AC-008..AC-015 / RG-008..RG-015) including `generate_schema_drift §generate_schema_drift` (AC-014/RG-014) and `build_tombstone §build_tombstone` Vec<String> obligation (AC-015/RG-015) |

---

## UX / Operator Surfaces

None — this story produces no user-facing UI changes and no new sensor IDs. Existing
`armis.sensor.toml` gains 6 new columns in the `devices` table and a `source_path`
amendment on an existing column.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `device_cves` is `[]`; `device_cves_first` queried | `device_cves_first` = null (empty array has no first element); `device_cves` = `"[]"` (not null); no error |
| EC-002 | `risk_factors` is `[]`; column queried | `risk_factors` column emits `"[]"` (not null); valid — device has no flagged risk contributors |
| EC-003 | `tags` is `[]`; column queried | `tags` column emits `"[]"` (not null); merged tag_store output is also empty (BC-3.2.001) |
| EC-004 | `os_version` is `null` (device has no OS info) | `os_version` column emits null; not an error; nullable per `Option<String>` |
| EC-005 | `network_id` is `null`; column queried | `network_id` column emits null; valid per `Option<String>` |
| EC-006 | `site` is `null`; column queried | `site` column emits null; valid per `Option<String>` |

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation)

- [ ] **RG-001**: `test_armis_toml_devices_table_has_os_version_column_string_type` — AC-001
  _(Parses `crates/prism-sensors/specs/armis.sensor.toml`; asserts the `devices` table
  contains a column named `"os_version"` with `column_type = "string"` (i.e., `ColumnType::String`
  after spec loading); mirrors the existing `test_ac_index_001_armis_toml_last_seen_created_at_have_index_option`
  column assertion pattern; confirms `DeviceRecord.os_version: Option<String>` is covered)_

- [ ] **RG-002**: `test_armis_toml_devices_table_has_risk_factors_column_json_type` — AC-002
  _(Parses `armis.sensor.toml`; asserts `devices` table contains `"risk_factors"` column with
  `ColumnType::Json`; confirms `DeviceRecord.risk_factors: Vec<String>` serializes as JSON
  array and is covered by the TOML spec)_

- [ ] **RG-003**: `test_armis_toml_devices_table_has_network_id_column_string_type` — AC-003
  _(Parses `armis.sensor.toml`; asserts `devices` table contains `"network_id"` column with
  `ColumnType::String`; confirms `DeviceRecord.network_id: Option<String>` is covered)_

- [ ] **RG-004**: `test_armis_toml_devices_table_has_site_column_string_type` — AC-004
  _(Parses `armis.sensor.toml`; asserts `devices` table contains `"site"` column with
  `ColumnType::String`; confirms `DeviceRecord.site: Option<String>` is covered)_

- [ ] **RG-005**: `test_armis_toml_devices_table_has_tags_column_json_type` — AC-005
  _(Parses `armis.sensor.toml`; asserts `devices` table contains `"tags"` column with
  `ColumnType::Json`; confirms `DeviceRecord.tags: Vec<String>` serializes as JSON array
  and is covered by the TOML spec)_

- [ ] **RG-006**: `test_armis_toml_devices_table_has_device_cves_column_json_type` — AC-006
  _(Parses `armis.sensor.toml`; asserts `devices` table contains `"device_cves"` column with
  `ColumnType::Json`; confirms `DeviceRecord.device_cves: Vec<String>` is covered; verifies
  distinct from `device_cves_first` scalar column)_

- [ ] **RG-007**: `test_armis_toml_device_cves_first_column_has_source_path_device_cves_0` — AC-007
  _(Parses `armis.sensor.toml`; asserts the `device_cves_first` column entry in the `devices`
  table carries `source_path = "$.device_cves[0]"`; confirms the fix is distinct from the
  full-array `device_cves` column (which has no source_path override); covers EC-001 edge
  case where `device_cves_first` correctly yields null when `device_cves` is empty)_

- [ ] **RG-008**: `test_armis_dtu_get_search_generated_records_device_has_os_version` — AC-008
  _(Issues `GET /api/v1/search?aql=in:devices` against an `ArmisState` with
  `fixture_gen_seeded=true` and at least one asset record; deserializes the serialized JSON
  response; asserts the first device record in the response has `"os_version"` key present.
  Wire-shape assertion per CLAUDE.md §Wire-shape assertion discipline — assertion on
  serialized bytes, not pre-serialization Rust struct. Test FAILS until T-03 adds
  `"os_version"` to `build_asset §build_asset`'s `json!` macro.
  Preferred location: `crates/prism-dtu-armis/tests/`.)_

- [ ] **RG-009**: `test_armis_dtu_get_search_generated_records_device_has_risk_factors` — AC-009
  _(Same pattern as RG-008; asserts `"risk_factors"` key present in first serialized device
  record. Test FAILS until T-03 adds `"risk_factors"` to `build_asset §build_asset`.
  Preferred location: `crates/prism-dtu-armis/tests/`.)_

- [ ] **RG-010**: `test_armis_dtu_get_search_generated_records_device_has_network_id` — AC-010
  _(Same pattern as RG-008; asserts `"network_id"` key present in first serialized device
  record. Test FAILS until T-03 adds `"network_id"` to `build_asset §build_asset`.
  Preferred location: `crates/prism-dtu-armis/tests/`.)_

- [ ] **RG-011**: `test_armis_dtu_get_search_generated_records_device_has_tags` — AC-011
  _(Same pattern as RG-008; asserts `"tags"` key present in first serialized device record.
  Test FAILS until T-03 adds `"tags"` to `build_asset §build_asset`.
  Preferred location: `crates/prism-dtu-armis/tests/`.)_

- [ ] **RG-012**: `test_armis_dtu_get_search_generated_records_device_has_device_cves` — AC-012
  _(Same pattern as RG-008; asserts `"device_cves"` key present in first serialized device
  record. Test FAILS until T-03 adds `"device_cves"` to `build_asset §build_asset`.
  Preferred location: `crates/prism-dtu-armis/tests/`.)_

- [ ] **RG-013**: `test_armis_dtu_device_cves_stripped_at_stage_0_in_both_routes` — AC-013
  _(Constructs an `ArmisState` with `StageMask.device_cves = false`; issues both
  `GET /api/v1/devices` and `GET /api/v1/search?aql=in:devices`; deserializes each
  serialized JSON response; asserts `"device_cves"` key is ABSENT in device records on
  BOTH routes. Wire-shape assertion per CLAUDE.md §Wire-shape assertion discipline — assert
  on serialized bytes, not pre-serialization Rust struct. Test FAILS until T-03 adds the
  stage-gate strip for `device_cves` in both routes alongside `device_cves_first`.
  Preferred location: `crates/prism-dtu-armis/tests/`.)_

- [ ] **RG-014**: `test_armis_dtu_schema_drift_archetype_has_contracted_columns` — AC-014
  _(Constructs an `ArmisState` that serves the `SchemaDrift` archetype; issues
  `GET /api/v1/search?aql=in:devices`; deserializes the serialized JSON response;
  asserts `records[0]` (the inline `drifted` record) has all five contracted keys —
  `"os_version"`, `"risk_factors"`, `"network_id"`, `"tags"`, `"device_cves"` — present
  with non-null values. Wire-shape assertion. Test FAILS until T-03 adds these columns
  to `generate_schema_drift §generate_schema_drift`'s inline `drifted` `json!` literal.
  Preferred location: `crates/prism-dtu-armis/tests/`.)_

- [ ] **RG-015**: `test_armis_dtu_tombstone_records_vec_columns_emit_empty_array` — AC-015
  _(Constructs an `ArmisState` with `HighChurn` archetype; issues
  `GET /api/v1/search?aql=in:devices`; finds at least one tombstone record in the
  serialized JSON response; asserts `"risk_factors": []`, `"tags": []`, `"device_cves": []`
  — `[]` not `null`. Wire-shape assertion. Test FAILS until T-03 adds Vec<String> columns
  to `build_tombstone §build_tombstone` as `json!([])`.
  Preferred location: `crates/prism-dtu-armis/tests/`.)_

**Red Gate density check** (BC-5.38.001): **15 failing tests** before implementation begins.
RG-001 covers AC-001 (`os_version` TOML column); RG-002 covers AC-002 (`risk_factors` TOML
column); RG-003 covers AC-003 (`network_id` TOML column); RG-004 covers AC-004 (`site` TOML
column); RG-005 covers AC-005 (`tags` TOML column); RG-006 covers AC-006 (`device_cves` TOML
column); RG-007 covers AC-007 (`device_cves_first` source_path fix); RG-008 covers AC-008
(`os_version` generated-records parity); RG-009 covers AC-009 (`risk_factors`
generated-records parity); RG-010 covers AC-010 (`network_id` generated-records parity);
RG-011 covers AC-011 (`tags` generated-records parity); RG-012 covers AC-012 (`device_cves`
generated-records parity); RG-013 covers AC-013 (`device_cves` stage-gate CRIT-001 in both
routes); RG-014 covers AC-014 (`SchemaDrift` archetype contracted column completeness);
RG-015 covers AC-015 (`HighChurn` tombstone `Vec<String>` columns emit `[]` not `null`).
RED_RATIO is computed by the orchestrator at Step 3.5 per per-story-delivery.md from actual
Red Gate results; BC-5.38.002 and BC-5.38.003 define the exempt test classes (green-by-design
and wiring-exempt) that reduce the denominator.

### Implementation tasks

### T-01: Add six new columns to `armis.sensor.toml` devices table
**Files:** `crates/prism-sensors/specs/armis.sensor.toml` (MODIFY)

Locate the `devices` table `[[tables.columns]]` block. Append the following six column
entries (exact TOML from BC-2.02.006 §TOML Contract):

```toml
  # F-SAP2-MED-005 / FB68d: os_version — OS version string.
  # DTU: DeviceRecord.os_version: Option<String> in prism-dtu-armis/src/types.rs.
  [[tables.columns]]
  name = "os_version"
  column_type = "string"
  ocsf_field = "device.os.version"

  # F-SAP2-MED-005 / FB68d: risk_factors — explanatory companion of risk_score.
  # DTU: DeviceRecord.risk_factors: Vec<String> (e.g. ["unpatched_cve", "open_ports"]).
  # column_type = "json": Vec<String> serializes as a JSON array of strings.
  # ocsf_field: Armis-specific; no OCSF standard field for risk factor labels.
  # Flows to raw_extensions per BC-2.02.007 preservation contract.
  [[tables.columns]]
  name = "risk_factors"
  column_type = "json"
  ocsf_field = "raw_extensions.risk_factors"

  # F-SAP2-MED-005 / FB68d: network_id — Armis network segment identifier.
  # DTU: DeviceRecord.network_id: Option<String>.
  [[tables.columns]]
  name = "network_id"
  column_type = "string"
  ocsf_field = "raw_extensions.network_id"

  # F-SAP2-MED-005 / FB68d: site — physical/logical deployment site.
  # DTU: DeviceRecord.site: Option<String>.
  [[tables.columns]]
  name = "site"
  column_type = "string"
  ocsf_field = "raw_extensions.site"

  # F-SAP2-MED-005 / FB68d: tags — analyst-managed device labels (Vec<String>).
  # DTU: DeviceRecord.tags: Vec<String>, merged with per-org tag_store at query time (BC-3.2.001).
  # column_type = "json": Vec<String> serializes as a JSON array of strings.
  # Direct agent-reasoning value: labels like "HIPAA" and "critical-infra" classify devices.
  [[tables.columns]]
  name = "tags"
  column_type = "json"
  ocsf_field = "raw_extensions.tags"

  # F-SAP2-MED-005 / FB68d: device_cves — full CVE ID array.
  # DTU: DeviceRecord.device_cves: Vec<String> (added S-DEMO-ENRICHMENT-PIVOT-002).
  # column_type = "json": Vec<String> serializes as a JSON array of strings.
  # Complements existing device_cves_first (scalar for enrichment UDF input per ADR-051 D4).
  # Note: device_cves_first requires source_path = "$.device_cves[0]" — see T-02.
  # Provides complete CVE context for agent reasoning.
  [[tables.columns]]
  name = "device_cves"
  column_type = "json"
  ocsf_field = "raw_extensions.device_cves"
```

SAP-2 compliance check: every column name added above must correspond to a `DeviceRecord`
field in `crates/prism-dtu-armis/src/types.rs` that is emitted by `routes::search::get_search`
via `serde_json::to_value(&merged)`. All six fields are confirmed present and emitted per
BC-2.02.006 §Postconditions.

Also update the TOML block comment that enumerates `DeviceRecord` fields to enumerate the
ACTUAL COMPLETE `DeviceRecord` field set including `device_cves` (added by
`S-DEMO-ENRICHMENT-PIVOT-002`): `device_id`, `name`, `ip_address`, `mac_address`,
`device_type`, `manufacturer`, `os_name`, `os_version`, `risk_score`, `risk_factors`,
`last_seen`, `first_seen`, `network_id`, `site`, `tags`, `device_cves`. All listed fields
MUST have corresponding columns in the TOML `devices` table. The prior comment omitting
`device_cves` would leave the field permanently unlisted even after it gains a column,
perpetuating future SAP-2 misses (LOW-001).

### T-02: Amend `device_cves_first` column entry to add `source_path` AND fulfill HIGH-001 generator obligation
**Files:** `crates/prism-sensors/specs/armis.sensor.toml` (MODIFY), `crates/prism-dtu-armis/src/generator.rs` (MODIFY)

**Part A — TOML amendment (precondition for `source_path` to work):**

Locate the existing `device_cves_first` column entry in the `devices` table. Add
`source_path = "$.device_cves[0]"` to that entry. Example:

```toml
  [[tables.columns]]
  name = "device_cves_first"
  column_type = "string"
  ocsf_field = "raw_extensions.device_cves_first"
  source_path = "$.device_cves[0]"
```

(Retain existing `column_type`, `ocsf_field`, and any other existing fields — only ADD
`source_path`; do NOT change existing entries.)

**Part B — HIGH-001 generator obligation (makes `source_path` work end-to-end):**

`source_path = "$.device_cves[0]"` works correctly on the generated-records path ONLY
if `device_cves` is populated per the scenario catalog. The `§generate_with_scenario_cves`
function MUST:

1. Stamp `device_cves = catalog_device_cves` (**the FULL scenario CVE array**, NOT a direct
   scalar) on `CompromisedEndpoint` device records. `device_cves` discriminates assets from
   alerts via presence of `asset_id`, consistent with existing `§generate_with_scenario_cves`
   design.
2. Stamp `device_cves = []` on all OTHER archetypes (non-`CompromisedEndpoint`).
3. **REMOVE** the previous direct `device_cves_first = catalog_device_cves[0]` scalar stamp —
   it is REDUNDANT once `source_path = "$.device_cves[0]"` is declared and `device_cves`
   carries the catalog array. This is dead code that must be removed to prevent potential
   double-key confusion.

**Four invariants that must hold simultaneously after both parts are complete:**

1. `device_cves_first` = `catalog_device_cves[0]` on `CompromisedEndpoint` at stage ≥ 4
   (T13 NVD/CVSS pivot preserved via the array, not a direct scalar stamp).
2. `device_cves_first` absent/null on non-scenario records and when catalog is empty
   (`has device_cves_first` correctly yields 0 results; pivot selectivity preserved).
3. Static-fixture and generated paths agree: `device_cves_first` absent on both static-fixture
   path and seeded-no-scenario branch; present and correct on scenario branch at stage ≥ 4.
4. Both `device_cves` and `device_cves_first` stage-gated per BC-2.06.019 PC-4 in BOTH
   `§paginate_devices` and `§get_search`. Anchor: AC-013 / RG-013.

After the complete fix (both parts):
- When `device_cves = ["CVE-2024-1234"]`, `device_cves_first` extracts `"CVE-2024-1234"`.
- When `device_cves = []`, `device_cves_first` extracts null (no first element in empty array).
- When `device_cves` key is absent (static-fixture path), `device_cves_first` is null.

The `device_cves_first` column is intentionally scalar (for enrichment UDF scalar input per
ADR-051 D4), while AC-006's `device_cves` column carries the full array.

### T-03: Add contracted columns to all three `prism-dtu-armis::generator` asset emitters
**Files:** `crates/prism-dtu-armis/src/generator.rs` (MODIFY)

**PREREQUISITE: RG-008..RG-015 must be authored as failing tests before this task begins.**

**Why three emitters matter:** RG-008..RG-012 assert on "the first device record" returned by
`GET /api/v1/search?aql=in:devices`. For the `SchemaDrift` archetype, `records[0]` IS the
`drifted` record — it is the first record served. Without fixing `generate_schema_drift
§generate_schema_drift`, RG-008..RG-012 Red Gate outcomes become archetype-dependent: passing
for standard archetypes, failing for `SchemaDrift`. Similarly, `build_tombstone
§build_tombstone` must emit `[]` for Vec<String> fields — `null` is type-incompatible.

#### 3a. `build_asset §build_asset` — add five keys

The `json!` macro inside `build_asset §build_asset` must emit five additional keys so the
generated-records path has parity with the static-fixture path (BC-2.02.006 §Generated-Records
Path Coverage contracted option (a)):

- `"os_version"`: realistic OS version string; use same pool-and-offset pattern as `os_name`
  to produce deterministic, archetype-appropriate values
- `"risk_factors"`: `json!([])` for healthy-device archetypes; non-empty array (e.g.,
  `json!(["unpatched_cve", "open_ports"])`) for `CompromisedEndpoint` archetype
- `"network_id"`: `format!("net-{}", id_index % 10)` deterministic string
- `"tags"`: `json!([])` — no analyst tags on fresh generated devices
- `"device_cves"`: `json!([])` for healthy-device archetypes on the seeded-no-scenario
  branch. For `CompromisedEndpoint` on the scenario branch, the `§generate_with_scenario_cves`
  obligation (T-02 Part B) stamps the full `catalog_device_cves` array. Anchor: AC-012.

`site` is already present and compliant per BC-2.02.006 §Generated-Records Path Coverage
(`format!("site-{}", id_index % 5)` for standard assets; `null` for tombstones via
`build_tombstone §build_tombstone` per `Option<String>` nullable contract).

The contracted obligation is key PRESENCE with type-compatible values — exact pool content
is implementer-determined except where the HIGH-001 adjudication specifies otherwise.
RG-008..RG-012 assert key presence; they do not assert exact values.

#### 3b. `generate_schema_drift §generate_schema_drift` — add five columns to the inline `drifted` record

The inline `drifted` `json!` literal in `generate_schema_drift §generate_schema_drift` MUST
carry all five contracted columns (`"os_version"`, `"risk_factors"`, `"network_id"`,
`"tags"`, `"device_cves"`) with type-compatible values. Per the `P12-01` comment in the
source, the intended drift is ONLY the missing `id` key. Populating these columns is the
intended behavior, not a drift violation. Suggested values follow the same pattern as
`build_asset §build_asset` (e.g., `"os_version": "22H2"`, `"risk_factors": []`,
`"network_id": "net-0"`, `"tags": []`, `"device_cves": []`). Anchor: AC-014 / RG-014.

#### 3c. `build_tombstone §build_tombstone` — add `Vec<String>` columns as `[]`

`build_tombstone §build_tombstone` for `HighChurn` archetype tombstone records MUST emit:

- `"risk_factors"`: `json!([])` — `Vec<String>` serializes as `[]`, NEVER `null`
- `"tags"`: `json!([])` — same reasoning
- `"device_cves"`: `json!([])` — same reasoning

`"site": null` remains correct because `site` is `Option<String>` (nullable).

`null` is type-incompatible with `Vec<String>` fields and violates wire-shape parity
(CLAUDE.md §Wire-shape assertion discipline). The static-fixture path serializes
`DeviceRecord.risk_factors`, `tags`, and `device_cves` as `[]` when empty — the generated
tombstone path MUST match. Anchor: AC-015 / RG-015.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~3,000 |
| `crates/prism-sensors/specs/armis.sensor.toml` (current, to read before amending) | ~2,500 |
| `crates/prism-dtu-armis/src/types.rs` (DeviceRecord ground truth) | ~1,000 |
| BC-2.02.006 v1.15 (§TOML Contract, §TOML Contract HIGH-001 adjudication, §Postconditions, §Generated-Records Path Coverage three-emitter table) | ~2,800 |
| `crates/prism-dtu-armis/src/generator.rs` (all three emitters to read before amending) | ~1,500 |
| Existing column Red Gate test pattern (prism-spec-engine/tests/ or prism-dtu-armis/tests/) | ~800 |
| Running test output (nextest per-crate) | ~500 |
| **Total estimate** | **~12,100** |

12,100 tokens is within the 20% context window limit. No split required.

---

## Previous Story Intelligence

**N/A — first story in `E-WAVE-A-SENSOR-REMEDIATION` epic for `armis.sensor.toml` column
additions. No preceding story in this scope chain.**

The Armis remediation story (`S-WAVE-A-ARMIS-REMEDIATION-001`) changes `auth_type` and
the `[auth_acquisition]` block — auth wiring only. This story does not interact with auth;
it only adds device data columns. `S-WAVE-A-ARMIS-ACTIVITY-001` is the sibling story in
this epic — it adds the `armis_device_activity` table to `armis.sensor.toml` (a new,
separate table block for the filter-push-down activity fetch path, per ADR-057
adjudication). Scope division is clean: this story owns the `devices` table only (six new
columns + `device_cves_first` `source_path` fix); ACTIVITY-001 owns the new
`armis_device_activity` table section. Both stories have `depends_on: []` and modify
`armis.sensor.toml` in non-overlapping sections. If both are implemented in the same wave,
rebase before final merge to avoid a TOML append-conflict.

Relevant lessons from adjacent work:
- SAP-2 (from PLUGIN-MIGRATION-001-D): every column name must match a DTU struct field
  that is emitted on the wire. All six new columns are pre-verified against `DeviceRecord`
  in BC-2.02.006 §Postconditions — do not guess at field names.
- The canonical pipeline-facing handler for `from armis.devices` queries is
  `routes::search::get_search` (`GET /api/v1/search?aql=<AQL>`), NOT
  `routes::devices::paginate_devices` (`GET /api/v1/devices`) — this distinction was
  corrected in BC-2.02.006 v1.8 (F-WASE-P66-OBS-001). Red Gate tests that assert
  wire-shape should target the `get_search` path.

---

## Architecture Compliance Rules

1. **ADR-023 §Rule 1 — `ocsf_field` annotation required.** Every column added must carry
   an `ocsf_field` annotation so `SpecDrivenMapper` can perform OCSF field mapping without
   a WASM plugin. Columns without `ocsf_field` require a `.prx` plugin per ADR-023.

2. **ADR-028 §D1 — DTU-grounded spec authoring.** Every TOML column must be grounded in
   a `DeviceRecord` field that is emitted by the pipeline-facing handler
   (`routes::search::get_search`). Column in TOML with no DTU equivalent is a P1 CRITICAL
   per SAP-2 protocol.

3. **CLAUDE.md §SAP-2 §Rule 6 — wire-emission-site authority.** The wire-emission site
   (`routes::search::get_search` `serde_json::to_value(&merged)` on the full `DeviceRecord`)
   is authoritative over the struct definition. Verify at the emission site, not just at
   the type definition.

4. **No new `#[non_exhaustive]` types.** This story modifies a TOML file only and adds
   Rust tests. No new public types are introduced. The non-exhaustive gate in
   `scripts/check-non-exhaustive-per-symbol.py` does not change.

5. **`device_cves_first` is a scalar extract, `device_cves` is the full array.** Do NOT
   conflate them. `device_cves_first` uses `source_path` to extract a single element;
   `device_cves` carries the complete array. Both must exist in the TOML simultaneously.

---

## Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|----------------|
| `toml` (TOML parser for test assertions) | workspace pinned | `architecture/dependency-graph.md §External Dependencies` |
| `serde_json` (for any wire-shape assertions) | workspace pinned | same |

No new external dependencies introduced by this story.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-sensors/specs/armis.sensor.toml` | MODIFY | T-01: add 6 new `[[tables.columns]]` entries to `devices` table; T-02: add `source_path = "$.device_cves[0]"` to existing `device_cves_first` column entry |
| `crates/prism-spec-engine/tests/` (new or existing test file) | MODIFY or CREATE | RG-001..RG-007: 7 failing tests that parse `armis.sensor.toml` and assert column presence, types, and `source_path` |
| `crates/prism-dtu-armis/src/generator.rs` | MODIFY | T-03: (a) add five keys to `build_asset §build_asset`'s `json!` macro; (b) add five columns to `generate_schema_drift §generate_schema_drift` inline `drifted` record; (c) add `Vec<String>` columns as `json!([])` to `build_tombstone §build_tombstone`; (d) T-02 Part B: `§generate_with_scenario_cves` stamps `device_cves = catalog_device_cves` on `CompromisedEndpoint`, removes redundant scalar stamp |
| `crates/prism-dtu-armis/tests/` (new or existing test file) | MODIFY or CREATE | RG-008..RG-015: 8 failing wire-shape tests; RG-008..RG-012 assert key presence on generated-records path; RG-013 asserts `device_cves` stripped at stage 0 in both routes (CRIT-001); RG-014 asserts `SchemaDrift` archetype `records[0]` has all five contracted column keys; RG-015 asserts `HighChurn` tombstone `Vec<String>` columns emit `[]` not `null` |

---

## Verification Properties

None assigned yet. Column schema correctness (TOML spec) is verified via RG-001..RG-007 Red
Gate tests; generated-records path wire coverage is verified via RG-008..RG-012. Downstream
SAP-2 probe on any subsequent adversarial pass re-verifies both surfaces.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.6 | 2026-07-29 | story-writer | FB93 leg 2 — advance BC-2.02.006 pin v1.13 → v1.15 at five live sites (frontmatter §BC status comment, §Background inline cite, AC-007 §HIGH-001 adjudication cite, §Behavioral Contracts table row, §Token Budget Estimate row). Pin-only; no mechanism or design content changed in BC-2.02.006 between v1.13 and v1.15 (§Traceability §Capability Anchor Justification row addition and BC→story anchor-form conversion only). §Behavioral Contracts relevance cell verified accurate at v1.15. POL-29 9a: ACTIVITY-001 twin updated in same burst; both pin BC-2.02.006 at v1.15. 9b: no downstream copy target affected. 9c: no new MUSTs introduced. |
| 1.5 | 2026-07-29 | story-writer | FB91 story leg — propagate BC-2.02.006 v1.13 + BC-2.02.014 v1.4 + ADR-057 v0.7 spec corrections into SPEC-001. (1a) Add AC-013/RG-013 (`test_armis_dtu_device_cves_stripped_at_stage_0_in_both_routes`), AC-014/RG-014 (`test_armis_dtu_schema_drift_archetype_has_contracted_columns`), AC-015/RG-015 (`test_armis_dtu_tombstone_records_vec_columns_emit_empty_array`) after AC-012; 12→15 ACs/RGTs, 12→15 density check, 12→15 in §Behavioral Contracts relevance cell. (1b) AC-007 expanded with HIGH-001 adjudication: four simultaneous constraints, mechanism correction (obj.remove NOT unconditional), generator obligation (§generate_with_scenario_cves stamps full catalog_device_cves; remove redundant scalar stamp); T-02 expanded with Part B HIGH-001 generator obligation, four invariants. (1c) T-03 expanded to all three emitters: build_asset §build_asset (3a), generate_schema_drift §generate_schema_drift (3b), build_tombstone §build_tombstone (3c); removed false "Do NOT modify build_tombstone" instruction; added rationale for why SchemaDrift matters for RG-008..RG-012. (1d) §Background MED-002: POST /api/v1/search → GET /api/v1/search?aql=<AQL> at two sites (§Background + §Previous Story Intelligence); §Background HIGH-002: four-branch mechanism description replaces false "injects and strips via obj.remove" unconditional claim; T-01 LOW-001: enumerate complete DeviceRecord field set including device_cves. (1e) BC-2.02.006 v1.12 → v1.13 in frontmatter # BC status, §Behavioral Contracts table, §Token Budget Estimate; reassessed Token Budget ~8,800→~12,100 (3 new ACs/RGTs + generator.rs read + v1.13 sections); points 5→8 (3 new ACs, two additional emitters, scalar-stamp removal); estimated_days 2→3; §Architecture Mapping expanded with generate_schema_drift and build_tombstone rows; §File Structure Requirements updated for all three emitters + 8 wire-shape tests. SAC-1: RG-013..RG-015 placed before implementation tasks. 9a: twin ARMIS-ACTIVITY-001 updated in same burst; both pin BC-2.02.006 at v1.13. 9b: §Background and §Previous Story Intelligence POST citation corrected at emission-site read; AC-007 mechanism claim verified against BC-2.02.006 v1.13 §Generated-Records Path Coverage four-branch description; 9b CLEAR. 9c: all new MUSTs in AC-013..AC-015 and T-03 carry AC+RGT anchors; no unanchored MUST introduced. |
| 1.4 | 2026-07-28 | story-writer | FB87 leg 2 — POL-23 stale-pin sweep: BC-2.02.006 v1.11 → v1.12 (downstream of FB87 leg 1 product-owner bump). Updated: frontmatter # BC status comment, §Behavioral Contracts table row, §Token Budget Estimate BC row. Relevance cell verified accurate at v1.12 — §Generated-Records Path Coverage framing (7-MUST TOML-column block + 5-MUST generated-records block = 12 ACs/RGTs) consistent with leg 1 reconciliation; no text correction needed. `serde_json::to_value` references in story body describe `get_search §get_search` devices-table handler only — correct as-is (activity handler not in scope; not applicable). No `ADR-057 §C1/§C2` references found. POL-29 9a: S-WAVE-A-ARMIS-ACTIVITY-001 twin updated in same burst; both stories now pin BC-2.02.006 at v1.12 (pre-burst asymmetry v1.10 vs v1.11 closed). 9b: ARMIS-SPEC-001 §TOML Contract block is not a downstream copy target for any further artifact. 9c: no new MUSTs authored. |
| 1.3 | 2026-07-28 | story-writer | FB86 — close F-WASE-P68-MED-004: delete banned authored-time RED_RATIO sentence (`7 Red Gate tests for 7 ACs — RED_RATIO = 7/7 = 1.0`) from §Red Gate density check; construct is banned regardless of arithmetic correctness per FB61/D-2041. Close F-WASE-P68-LOW-001: advance §Token Budget Estimate BC-2.02.006 pin from v1.8 to v1.11 and extend section list to include §Generated-Records Path Coverage (added in FB85 scope); reassess token estimate ~1,500→~2,000, total ~8,300→~8,800. §Previous Story Intelligence `was corrected in BC-2.02.006 v1.8` historically-scoped prose verified correct-as-written (not swept). v1.0 and v1.2 changelog BC-2.02.006 references are immutable historical record (not swept). POL-29 9a: S-WAVE-A-ARMIS-ACTIVITY-001 twin in same burst — same banned sentence deleted and trailing deferral note normalized; symmetric. 9b: no downstream copy target. 9c: no new MUSTs introduced. |
| 1.2 | 2026-07-28 | story-writer | FB85 story half — SAP-2 Rule 6 dual-path wire-coverage gap for five new Armis device columns. Add AC-008..AC-012 and RG-008..RG-012 anchored to BC-2.02.006 v1.11 §Generated-Records Path Coverage; exact test names match BC anchors for POL-29 9c bidirectionality. Expand `crates_touched` to include `prism-dtu-armis`; update `subsystems` to include SS-12 (DTU-Armis); bump `points` 3→5 and `estimated_days` 1→2 for generator scope. Add T-03 (implementer task: add five keys to `build_asset §build_asset`). Update §Architecture Mapping with generator and wire-shape test rows. Update §File Structure Requirements with `generator.rs` MODIFY row and `prism-dtu-armis/tests/` row for RG-008..RG-012. Update density paragraph: 7→12 failing tests, add RG-008..RG-012 coverage; RED_RATIO sentence and §Token Budget Estimate BC pin left unchanged (FB86 scope). BC-2.02.006 pin updated v1.9→v1.11 in §Behavioral Contracts table and frontmatter comment. Red-then-green ordering preserved: RG-008..RG-012 checkboxes appear before T-03. POL-29 9a: `S-WAVE-A-ARMIS-ACTIVITY-001` — `get_device_activity §get_device_activity` has ONE path only (static fixture, no generated-records branch; confirmed from story v1.3 §Ground-Truth DTU State); SAP-2 Rule 6 dual-path coverage gap does NOT apply; 9a CLEAR. 9b: §Generated-Records Path Coverage block is new content self-contained in this story, not a copy of a downstream target. 9c: all five new MUSTs carry AC+RGT anchors matching BC-2.02.006 v1.11 exactly. |
| 1.1 | 2026-07-27 | story-writer | FB78: POL-29 9a sibling sweep — replace stale volatile framing of `S-WAVE-A-ARMIS-ACTIVITY-001` ("blocked on architect confirmation") with durable scope-division description: ACTIVITY-001 covers `armis_device_activity` table (ADR-057 filter-push-down path); this story covers `devices` table only; both have `depends_on: []` and modify non-overlapping sections; rebase note for co-land. Update BC-2.02.006 version pin from v1.8 to v1.9 in §Behavioral Contracts table and frontmatter comment. |
| 1.0 | 2026-07-27 | story-writer | FB72 leg 1 Item 6: create `S-WAVE-A-ARMIS-SPEC-001` per BC-2.02.006 v1.8 §TOML Contract POL-29 9c anchor. Scope: 6 column additions (`os_version`, `risk_factors`, `network_id`, `site`, `tags`, `device_cves`) + `device_cves_first` `source_path` fix. 7 ACs, 7 RGTs, `tdd_mode: strict`, `## Authority` citing ADR-023 §Rule 1 and ADR-028 §D1. |
