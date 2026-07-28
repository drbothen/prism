---
document_type: story
story_id: S-WAVE-A-ARMIS-SPEC-001
title: "Armis Sensor Spec — Six Column Additions and device_cves_first source_path Fix"
version: "1.1"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P1
points: 3
tdd_mode: strict
target_module: prism-sensors
subsystems: ["SS-06 (SensorSpec)", "SS-07 (SpecEngine)"]
crates_touched:
  - prism-sensors    # armis.sensor.toml modification
  - prism-spec-engine  # Red Gate tests read and parse the TOML spec
depends_on: []
blocks: []
behavioral_contracts:
  - BC-2.02.006
verification_properties: []
assumption_validations: []
risk_mitigations: []
estimated_days: 1
modified: "2026-07-27"
# BC status: BC-2.02.006 v1.9 §TOML Contract specifies all 7 ACs and 7 RGTs; status may
# transition to ready once Red Gate tests are authored and BC-2.02.006 is confirmed active.
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

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Architecture Section |
|-----------|--------|---------------|----------------------|
| `armis.sensor.toml` (6 new columns + 1 source_path fix) | `crates/prism-sensors/specs/` | Pure (config data) | `architecture/module-decomposition.md §SS-06 SensorSpec` |
| Red Gate tests (parse and assert spec) | `crates/prism-spec-engine/tests/` or `crates/prism-dtu-armis/tests/` | Pure (test only) | `architecture/module-decomposition.md §SS-07 SpecEngine` |

---

## Behavioral Contracts

| BC | Version | Relevance to This Story |
|----|---------|------------------------|
| BC-2.02.006 | v1.9 | Armis Centrix Field Mapping to OCSF — §TOML Contract specifies all 6 new columns and the `device_cves_first` source_path fix; §Postconditions specifies per-field OCSF mappings |

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

**Red Gate density check** (BC-5.38.001): **7 failing tests** before implementation begins.
RG-001 covers AC-001 (`os_version`); RG-002 covers AC-002 (`risk_factors`); RG-003 covers
AC-003 (`network_id`); RG-004 covers AC-004 (`site`); RG-005 covers AC-005 (`tags`); RG-006
covers AC-006 (`device_cves`); RG-007 covers AC-007 (`device_cves_first` source_path fix).
7 Red Gate tests for 7 ACs — RED_RATIO = 7/7 = 1.0 (meets BC-5.38.001 threshold).
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

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~2,000 |
| `crates/prism-sensors/specs/armis.sensor.toml` (current, to read before amending) | ~2,500 |
| `crates/prism-dtu-armis/src/types.rs` (DeviceRecord ground truth) | ~1,000 |
| BC-2.02.006 v1.8 (§TOML Contract, §Postconditions) | ~1,500 |
| Existing column Red Gate test pattern (prism-spec-engine/tests/ or prism-dtu-armis/tests/) | ~800 |
| Running test output (nextest per-crate) | ~500 |
| **Total estimate** | **~8,300** |

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
| `crates/prism-sensors/specs/armis.sensor.toml` | MODIFY | Task T-01: add 6 new `[[tables.columns]]` entries to `devices` table; Task T-02: add `source_path = "$.device_cves[0]"` to existing `device_cves_first` column entry |
| `crates/prism-spec-engine/tests/` (new or existing test file) | MODIFY or CREATE | RG-001 through RG-007: 7 failing tests that parse `armis.sensor.toml` and assert column presence and types |

---

## Verification Properties

None assigned yet. Column schema correctness is verified via RG-001..RG-007 Red Gate tests
and downstream SAP-2 probe on any subsequent adversarial pass.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.1 | 2026-07-27 | story-writer | FB78: POL-29 9a sibling sweep — replace stale volatile framing of `S-WAVE-A-ARMIS-ACTIVITY-001` ("blocked on architect confirmation") with durable scope-division description: ACTIVITY-001 covers `armis_device_activity` table (ADR-057 filter-push-down path); this story covers `devices` table only; both have `depends_on: []` and modify non-overlapping sections; rebase note for co-land. Update BC-2.02.006 version pin from v1.8 to v1.9 in §Behavioral Contracts table and frontmatter comment. |
| 1.0 | 2026-07-27 | story-writer | FB72 leg 1 Item 6: create `S-WAVE-A-ARMIS-SPEC-001` per BC-2.02.006 v1.8 §TOML Contract POL-29 9c anchor. Scope: 6 column additions (`os_version`, `risk_factors`, `network_id`, `site`, `tags`, `device_cves`) + `device_cves_first` `source_path` fix. 7 ACs, 7 RGTs, `tdd_mode: strict`, `## Authority` citing ADR-023 §Rule 1 and ADR-028 §D1. |
