---
document_type: story
story_id: S-WAVE-A-ARMIS-SPEC-001
title: "Armis Sensor Spec — Six Column Additions and device_cves_first source_path Fix"
version: "1.3"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P1
points: 5
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
estimated_days: 2
modified: "2026-07-28"
# BC status: BC-2.02.006 v1.11 §TOML Contract + §Generated-Records Path Coverage specify
# all 12 ACs and 12 RGTs; status may transition to ready once Red Gate tests are authored
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
struct (canonical pipeline-facing handler for `from armis.devices` queries, `POST /api/v1/search`),
but have no corresponding `[[tables.columns]]` entries in `armis.sensor.toml`.
The `SpecDrivenMapper` cannot extract or map fields that have no TOML column declaration.

F-WASE-P66-HIGH-004 identified that `device_cves_first` has no `DeviceRecord` field in
`prism-dtu-armis/src/types.rs`. Without `source_path = "$.device_cves[0]"`, the spec-engine
cannot derive `device_cves_first` from the `device_cves` array and the column always resolves
to absent/null: (a) static-fixture path: `serde_json::to_value(&merged)` on `DeviceRecord`
omits the key (no such field); (b) generated-records path: the handler injects the key as a
temporary value then strips it via `obj.remove("device_cves_first")`. The `source_path`
directive tells the spec-engine to extract the first element of the `device_cves` JSON array.

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
value from the `device_cves` array via JSONPath extraction; EC-02-013 remedy)

The existing `device_cves_first` column entry in the `armis.sensor.toml` devices table is
amended to add `source_path = "$.device_cves[0]"`. With this directive, the spec-engine
extracts the first element of the `device_cves` JSON array as the value for this column.

Without `source_path`: `DeviceRecord` has no `device_cves_first` field, so the static-fixture
path (`serde_json::to_value(&merged)`) omits the key entirely; the generated-records path
injects and strips it via `obj.remove("device_cves_first")`. After fix: when `device_cves`
is `["CVE-2024-1234", "CVE-2024-5678"]`, `device_cves_first` = `"CVE-2024-1234"`. When
`device_cves` is `[]`, `device_cves_first` = null (no first element in empty array).

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

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Architecture Section |
|-----------|--------|---------------|----------------------|
| `armis.sensor.toml` (6 new columns + 1 source_path fix) | `crates/prism-sensors/specs/` | Pure (config data) | `architecture/module-decomposition.md §SS-06 SensorSpec` |
| Red Gate tests RG-001..RG-007 (parse and assert TOML spec) | `crates/prism-spec-engine/tests/` | Pure (test only) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `build_asset §build_asset` (add five generated-records keys) | `crates/prism-dtu-armis/src/generator.rs` | Pure (record construction) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |
| Red Gate tests RG-008..RG-012 (wire-shape assertions on generated-records path) | `crates/prism-dtu-armis/tests/` | Pure (test only) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |

---

## Behavioral Contracts

| BC | Version | Relevance to This Story |
|----|---------|------------------------|
| BC-2.02.006 | v1.11 | Armis Centrix Field Mapping to OCSF — §TOML Contract specifies all 6 new columns and the `device_cves_first` source_path fix; §Postconditions specifies per-field OCSF mappings; §Generated-Records Path Coverage (added v1.11 / FB85) specifies `build_asset §build_asset` dual-path gap and the five generated-records MUSTs anchored to AC-008..AC-012 |

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

**Red Gate density check** (BC-5.38.001): **12 failing tests** before implementation begins.
RG-001 covers AC-001 (`os_version` TOML column); RG-002 covers AC-002 (`risk_factors` TOML
column); RG-003 covers AC-003 (`network_id` TOML column); RG-004 covers AC-004 (`site` TOML
column); RG-005 covers AC-005 (`tags` TOML column); RG-006 covers AC-006 (`device_cves` TOML
column); RG-007 covers AC-007 (`device_cves_first` source_path fix); RG-008 covers AC-008
(`os_version` generated-records parity); RG-009 covers AC-009 (`risk_factors`
generated-records parity); RG-010 covers AC-010 (`network_id` generated-records parity);
RG-011 covers AC-011 (`tags` generated-records parity); RG-012 covers AC-012 (`device_cves`
generated-records parity).
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

Also update the TOML block comment that enumerates `DeviceRecord` fields (if present) to
reflect that all listed fields now have corresponding columns.

### T-02: Amend `device_cves_first` column entry to add `source_path`
**Files:** `crates/prism-sensors/specs/armis.sensor.toml` (MODIFY)

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

After this fix:
- When `device_cves = ["CVE-2024-1234"]`, `device_cves_first` extracts `"CVE-2024-1234"`.
- When `device_cves = []`, `device_cves_first` extracts null (empty array has no first element).
- When `device_cves` key is absent, `device_cves_first` is null.

The `device_cves_first` column is intentionally scalar (for enrichment UDF scalar input per
ADR-051 D4), while AC-006's `device_cves` column carries the full array.

### T-03: Add five keys to `build_asset §build_asset` in `prism-dtu-armis::generator`
**Files:** `crates/prism-dtu-armis/src/generator.rs` (MODIFY)

**PREREQUISITE: RG-008..RG-012 must be authored as failing tests before this task begins.**

The `json!` macro inside `build_asset §build_asset` in `prism-dtu-armis::generator` must
emit five additional keys so the generated-records path has parity with the static-fixture
path (BC-2.02.006 §Generated-Records Path Coverage contracted option (a)):

- `"os_version"`: realistic OS version string; use same pool-and-offset pattern as `os_name`
  to produce deterministic, archetype-appropriate values
- `"risk_factors"`: `json!([])` for healthy-device archetypes; non-empty array (e.g.,
  `json!(["unpatched_cve", "open_ports"])`) for `CompromisedEndpoint` archetype
- `"network_id"`: `format!("net-{}", id_index % 10)` deterministic string
- `"tags"`: `json!([])` — no analyst tags on fresh generated devices
- `"device_cves"`: `json!([])` for healthy-device archetypes; non-empty (e.g.,
  `json!(["CVE-2024-1234"])`) for `CompromisedEndpoint` archetype

`site` is already present and compliant per BC-2.02.006 §Generated-Records Path Coverage
(`format!("site-{}", id_index % 5)` for standard assets). Do NOT modify `build_tombstone
§build_tombstone` for these keys — tombstones emit null for `site` (valid per
`Option<String>` nullable contract); any additional nullable fields on the tombstone path
should follow the same null pattern.

The contracted obligation is key PRESENCE with type-compatible values — exact pool content
is implementer-determined (BC-2.02.006 §TOML Contract §Generated-Records Path Coverage).
RG-008..RG-012 assert key presence; they do not assert exact values.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~2,000 |
| `crates/prism-sensors/specs/armis.sensor.toml` (current, to read before amending) | ~2,500 |
| `crates/prism-dtu-armis/src/types.rs` (DeviceRecord ground truth) | ~1,000 |
| BC-2.02.006 v1.11 (§TOML Contract, §Postconditions, §Generated-Records Path Coverage) | ~2,000 |
| Existing column Red Gate test pattern (prism-spec-engine/tests/ or prism-dtu-armis/tests/) | ~800 |
| Running test output (nextest per-crate) | ~500 |
| **Total estimate** | **~8,800** |

8,300 tokens is well within the 20% context window limit. No split required.

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
  `routes::search::get_search` (`POST /api/v1/search`), NOT `routes::devices::paginate_devices`
  (`GET /api/v1/devices`) — this distinction was corrected in BC-2.02.006 v1.8
  (F-WASE-P66-OBS-001). Red Gate tests that assert wire-shape should target the
  `get_search` path.

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
| `crates/prism-dtu-armis/src/generator.rs` | MODIFY | T-03: add five keys (`os_version`, `risk_factors`, `network_id`, `tags`, `device_cves`) to `build_asset §build_asset`'s `json!` macro for generated-records path parity |
| `crates/prism-dtu-armis/tests/` (new or existing test file) | MODIFY or CREATE | RG-008..RG-012: 5 failing wire-shape tests against `GET /api/v1/search?aql=in:devices` with `fixture_gen_seeded=true`; each asserts a specific key is present in the serialized JSON response device record |

---

## Verification Properties

None assigned yet. Column schema correctness (TOML spec) is verified via RG-001..RG-007 Red
Gate tests; generated-records path wire coverage is verified via RG-008..RG-012. Downstream
SAP-2 probe on any subsequent adversarial pass re-verifies both surfaces.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.3 | 2026-07-28 | story-writer | FB86 — close F-WASE-P68-MED-004: delete banned authored-time RED_RATIO sentence (`7 Red Gate tests for 7 ACs — RED_RATIO = 7/7 = 1.0`) from §Red Gate density check; construct is banned regardless of arithmetic correctness per FB61/D-2041. Close F-WASE-P68-LOW-001: advance §Token Budget Estimate BC-2.02.006 pin from v1.8 to v1.11 and extend section list to include §Generated-Records Path Coverage (added in FB85 scope); reassess token estimate ~1,500→~2,000, total ~8,300→~8,800. §Previous Story Intelligence `was corrected in BC-2.02.006 v1.8` historically-scoped prose verified correct-as-written (not swept). v1.0 and v1.2 changelog BC-2.02.006 references are immutable historical record (not swept). POL-29 9a: S-WAVE-A-ARMIS-ACTIVITY-001 twin in same burst — same banned sentence deleted and trailing deferral note normalized; symmetric. 9b: no downstream copy target. 9c: no new MUSTs introduced. |
| 1.2 | 2026-07-28 | story-writer | FB85 story half — SAP-2 Rule 6 dual-path wire-coverage gap for five new Armis device columns. Add AC-008..AC-012 and RG-008..RG-012 anchored to BC-2.02.006 v1.11 §Generated-Records Path Coverage; exact test names match BC anchors for POL-29 9c bidirectionality. Expand `crates_touched` to include `prism-dtu-armis`; update `subsystems` to include SS-12 (DTU-Armis); bump `points` 3→5 and `estimated_days` 1→2 for generator scope. Add T-03 (implementer task: add five keys to `build_asset §build_asset`). Update §Architecture Mapping with generator and wire-shape test rows. Update §File Structure Requirements with `generator.rs` MODIFY row and `prism-dtu-armis/tests/` row for RG-008..RG-012. Update density paragraph: 7→12 failing tests, add RG-008..RG-012 coverage; RED_RATIO sentence and §Token Budget Estimate BC pin left unchanged (FB86 scope). BC-2.02.006 pin updated v1.9→v1.11 in §Behavioral Contracts table and frontmatter comment. Red-then-green ordering preserved: RG-008..RG-012 checkboxes appear before T-03. POL-29 9a: `S-WAVE-A-ARMIS-ACTIVITY-001` — `get_device_activity §get_device_activity` has ONE path only (static fixture, no generated-records branch; confirmed from story v1.3 §Ground-Truth DTU State); SAP-2 Rule 6 dual-path coverage gap does NOT apply; 9a CLEAR. 9b: §Generated-Records Path Coverage block is new content self-contained in this story, not a copy of a downstream target. 9c: all five new MUSTs carry AC+RGT anchors matching BC-2.02.006 v1.11 exactly. |
| 1.1 | 2026-07-27 | story-writer | FB78: POL-29 9a sibling sweep — replace stale volatile framing of `S-WAVE-A-ARMIS-ACTIVITY-001` ("blocked on architect confirmation") with durable scope-division description: ACTIVITY-001 covers `armis_device_activity` table (ADR-057 filter-push-down path); this story covers `devices` table only; both have `depends_on: []` and modify non-overlapping sections; rebase note for co-land. Update BC-2.02.006 version pin from v1.8 to v1.9 in §Behavioral Contracts table and frontmatter comment. |
| 1.0 | 2026-07-27 | story-writer | FB72 leg 1 Item 6: create `S-WAVE-A-ARMIS-SPEC-001` per BC-2.02.006 v1.8 §TOML Contract POL-29 9c anchor. Scope: 6 column additions (`os_version`, `risk_factors`, `network_id`, `site`, `tags`, `device_cves`) + `device_cves_first` `source_path` fix. 7 ACs, 7 RGTs, `tdd_mode: strict`, `## Authority` citing ADR-023 §Rule 1 and ADR-028 §D1. |
