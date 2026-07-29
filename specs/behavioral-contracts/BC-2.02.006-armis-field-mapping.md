---
document_type: behavioral-contract
level: L3
version: "1.12"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-02"
capability: "CAP-003"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
traces_to: ["CAP-003"]
extracted_from: ".factory/specs/prd.md"
scheduled_amendment_in: null
amendment_lifecycle: null
introduced: cycle-1
modified: "2026-07-28"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.006: Armis Centrix Field Mapping to OCSF (7 Data Sources)

## Description

> **Amendment — ADR-023 (PLUGIN-MIGRATION-001-G):** This BC previously described a
> hardcoded Rust mapper module (`prism-ocsf/src/mappers/armis.rs`). That
> implementation was deleted in PLUGIN-MIGRATION-001-C (PR #158). The field-mapping behavior
> described here is now delivered by `SpecDrivenMapper` reading `ocsf_field` column
> annotations from the Armis TOML sensor spec. The behavioral contract itself
> is unchanged — the same OCSF field mappings must be produced; they are now
> data-driven via TOML annotations per ADR-023 Rule 1.

`SpecDrivenMapper` reads `ocsf_field` column annotations from the Armis TOML sensor spec and converts records fetched via AQL GetSearch from 7 Armis sources to appropriate OCSF event classes. Key mappings are `ipaddress` → `device.ip`, `name` → `device.hostname`, alert severity → `severity_id`, and `riskLevel` → OCSF risk score fields. Armis-specific fields (`aqlResults`, `connectionType`, `riskFactors`) and records from sources with no dedicated OCSF class are preserved in `raw_extensions`. Timestamp fallback is used when no primary timestamp field exists.

## Preconditions
- An Armis record has been fetched via AQL GetSearch from one of the 7 sources
- Timestamp extraction used the per-source fallback chain successfully (or fell back to fetch timestamp)

## Postconditions
- Armis `ipaddress` maps to OCSF `device.ip`
- Armis `name` (device name) maps to OCSF `device.hostname`
- Armis alert severity maps to OCSF `severity_id`
- Armis `riskLevel` maps to OCSF risk score fields
- Armis-specific fields (e.g., `aqlResults`, `connectionType`, `riskFactors`) are preserved in `raw_extensions`
- Each of the 7 Armis sources maps to an appropriate OCSF event class
- Six previously undeclared `DeviceRecord` fields are now covered (F-SAP2-MED-005 / FB68d). All six are emitted on the wire by the **static-fixture path** of `routes::search::get_search` (canonical pipeline-facing handler for `from armis.devices` queries, `POST /api/v1/search`) via `serde_json::to_value(&merged)` on the full `DeviceRecord` struct. `routes::devices::paginate_devices` (GET /api/v1/devices) also serializes the same `DeviceRecord` but is NOT the pipeline-facing path for PrismQL sensor queries (SAP-2 §Rule 6 wire-emission-site authority correction; F-WASE-P66-OBS-001). The generated-records path (`fixture_gen_seeded=true` branch in `get_search §get_search`, F-P2-CRIT-001) has path-dependent per-column coverage — see §TOML Contract §Generated-Records Path Coverage (F-WASE-P68-MED-003 / FB85). Coverage decisions and OCSF mappings:
  - `os_version: Option<String>` → TOML column `os_version`, `column_type = "string"`, `ocsf_field = "device.os.version"`. SOC analyst value: OS version is required to assess CVE applicability; `os_name` without `os_version` leaves the agent unable to determine patch posture.
  - `risk_factors: Vec<String>` → TOML column `risk_factors`, `column_type = "json"`, `ocsf_field = "raw_extensions.risk_factors"`. This is the explanatory companion of `risk_score` — a numeric risk score without its factor labels is an unexplainable verdict. An SOC agent reading `risk_score = 85` with no `risk_factors = ["unpatched_cve","open_ports"]` cannot prioritize or explain the finding.
  - `network_id: Option<String>` → TOML column `network_id`, `column_type = "string"`, `ocsf_field = "raw_extensions.network_id"`. SOC analyst value: network segment membership is essential for blast-radius and lateral movement assessment.
  - `site: Option<String>` → TOML column `site`, `column_type = "string"`, `ocsf_field = "raw_extensions.site"`. SOC analyst value: physical/logical deployment site enables geographic/organizational scoping of incidents (production vs test, HQ vs remote).
  - `tags: Vec<String>` → TOML column `tags`, `column_type = "json"`, `ocsf_field = "raw_extensions.tags"`. Tags are analyst-managed device labels (e.g., "HIPAA", "PCI-scope", "critical-infra") — they directly improve SOC agent classification and response prioritization. Note: at query time, tags are merged with the per-org `tag_store` by `routes::devices::paginate_devices` (BC-3.2.001).
  - `device_cves: Vec<String>` → TOML column `device_cves`, `column_type = "json"`, `ocsf_field = "raw_extensions.device_cves"`. Provides the full CVE ID array for the device, complementing the existing `device_cves_first` scalar column (used for enrichment UDF input). The agent receives complete CVE context, not only the first element.
- `armis_device_activity` surface (DTU route `GET /api/v1/devices/{device_id}/activity`, types `ActivityRecord` / `ActivityResponse` / `ActivityData`) is DELIBERATELY DEFERRED to story `S-WAVE-A-ARMIS-ACTIVITY-001` — the full behavioral contract for this surface is specified in BC-2.02.014. **Resolution per ADR-057 (2026-07-27):** the correct fetch grammar is `path_template = "/api/v1/devices/${query.filter.device_id}/activity"` using the existing `${query.filter.*}` push-down namespace — the same mechanism as `${query.filter.aql}` in the `armis_devices` and `armis_alerts` tables (shipping exemplar in `crates/prism-sensors/specs/armis.sensor.toml`: `path_template = "/api/v1/search?aql=${query.filter.aql}"`). Ground truth: `PipelineExecutor.execute_impl` (block comment `F-LP1-HIGH-004` in `pipeline.rs §execute_impl`) pre-seeds `step_vars` with all `FetchContext.query_filters` entries under the `query.filter.{key}` namespace via `for (k, v) in &context.query_filters { step_vars.insert(format!("query.filter.{k}"), ...) }`. The `device_id` column MUST carry `options = ["INDEX"]` to declare push-down eligibility per the BC-2.11.007 taxonomy (REQUIRED / INDEX / ADDITIONAL) for future T2 (`classify_predicates §classify_predicates`) integration; the current routing is annotation-agnostic: `predicate_tree_to_filter_map §predicate_tree_to_filter_map` collects all case-sensitive `field = 'string'` equality predicates regardless of annotation into `FetchContext.query_filters`; `execute_impl §execute_impl` then pre-seeds `step_vars["query.filter.device_id"]` from that map via the `${query.filter.*}` pre-seed mechanism (ADR-057 §D4). Scope is single-device, filter-required; an absent `device_id` filter raises `SpecEngineError::HttpRequestFailed` (hard error — no silent empty result; implementation obligation noted in BC-2.02.014 §TOML Contract). Fleet-wide activity iteration is explicitly out of scope for Wave-A and requires the per-record fan-out capability gap (ADR-057 §D6) to be addressed in a future story. The previously noted architectural uncertainty (per-scalar-vs-per-batch, phantom `${step_name.field}` grammar corrected in v1.8 FB71) is fully resolved by ADR-057 §D3: per-record fan-out is confirmed a genuine capability gap, NOT expressible in the current engine. Story `S-WAVE-A-ARMIS-ACTIVITY-001` is UNBLOCKED. SAP-2 deliberate-exclusion status updated in EC-02-014 below.

## Invariants
- DI-005: OCSF schema validity

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | Armis record missing all timestamp fallback fields | OCSF `time` set to fetch timestamp; warning logged (DEC-013) |
| Warning | Armis severity/risk value in unexpected format | Best-effort mapping; unrecognized values go to `raw_extensions` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-009 | Armis `connections` records (network flow data) | Mapped to OCSF Network Activity class; source/destination IPs extracted from connection fields |
| EC-02-010 | Armis `risk_factors` records (metadata about risk scoring) | Mapped to generic OCSF event; risk factor details in `raw_extensions` for agent consumption |
| EC-02-011 | `risk_factors` is an empty array `[]` | `risk_factors` column emits `"[]"` (not null); valid — device has no flagged risk contributors at this time |
| EC-02-012 | `tags` is an empty array `[]` | `tags` column emits `"[]"` (not null); valid — analyst has not yet tagged this device; merged tag_store is also empty for this device (BC-3.2.001) |
| EC-02-013 | `device_cves` is an empty array `[]`; `device_cves_first` resolves to absent/null because `source_path` is not yet declared on the column entry | `device_cves` column emits `"[]"` (not null) — valid; device has no unpatched CVEs. `device_cves_first` currently resolves to absent/null on BOTH DTU paths: (a) static-fixture path calls `serde_json::to_value(&merged)` on `DeviceRecord`, which has no `device_cves_first` field in `prism-dtu-armis/src/types.rs` — key is absent from serialized output; (b) generated-records path injects `device_cves_first` as a temporary key then immediately removes it via `obj.remove("device_cves_first")` in `routes::search::get_search`. The required TOML remedy: add `source_path = "$.device_cves[0]"` to the `device_cves_first` column entry in `armis.sensor.toml` — the spec-engine will then extract the first element of the `device_cves` JSON array via JSONPath. Post-fix, when `device_cves` is `[]`: `device_cves_first` = null (empty array has no first element). Anchored to proposed `S-WAVE-A-ARMIS-SPEC-001` AC-007 (F-WASE-P66-HIGH-004). |
| EC-02-014 | `armis_device_activity` surface — SAP-2 deliberate-exclusion tracking sentinel | DTU route `GET /api/v1/devices/{device_id}/activity` exists and serves `ActivityRecord` / `ActivityResponse` / `ActivityData` (types in `prism-dtu-armis::types`). No TOML table is declared in the current `armis.sensor.toml`. This exclusion is DELIBERATE and TEMPORARY, deferred to `S-WAVE-A-ARMIS-ACTIVITY-001`. **Resolution status per ADR-057 (2026-07-27):** the blocking architectural question (push-down grammar) is RESOLVED — the correct grammar is `path_template = "/api/v1/devices/${query.filter.device_id}/activity"` with `device_id` column `options = ["INDEX"]`; story `S-WAVE-A-ARMIS-ACTIVITY-001` is UNBLOCKED; the full behavioral contract is specified in BC-2.02.014. **Per-record fan-out gap (ADR-057 §D6):** fleet-wide activity queries (iterating all devices without a `device_id` filter) require a `for_each` engine capability not present in the current pipeline; this gap is NOT resolved by ADR-057 and is NOT required for Wave-A. A future story addressing this capability MUST cite ADR-057 §D6 as the architecture anchor. A future SAP-2 pass that finds "no TOML table for armis_device_activity" should NOT re-mint a finding while this EC is present — the exclusion is documented with a concrete story anchor, the grammar is resolved, and the implementing story is UNBLOCKED. |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.006-001 | Armis alert with ipaddress, name, severity | `device.ip`, `device.hostname`, `severity_id` set correctly |
| TV-BC-2.02.006-002 | Armis device with riskLevel | `riskLevel` mapped to OCSF risk score fields |
| TV-BC-2.02.006-003 | Armis connections record | Mapped to Network Activity class; source/dest IPs extracted |
| TV-BC-2.02.006-004 | Record missing all timestamp fallback fields | `time` = fetch timestamp; warning logged |
| TV-BC-2.02.006-005 | Severity in unexpected format | Best-effort mapping; unrecognized in `raw_extensions`; warning logged |
| TV-BC-2.02.006-006 | DeviceRecord with `os_version = "22H2"` | `os_version` column = `"22H2"`; `ocsf_field = "device.os.version"` populated |
| TV-BC-2.02.006-007 | DeviceRecord with `risk_factors = ["unpatched_cve", "open_ports"]` | `risk_factors` column = JSON array string `["unpatched_cve","open_ports"]`; present alongside `risk_score` |
| TV-BC-2.02.006-008 | DeviceRecord with `network_id = "net-001"` | `network_id` column = `"net-001"`; raw extension preserved |
| TV-BC-2.02.006-009 | DeviceRecord with `site = "HQ-Floor3"` | `site` column = `"HQ-Floor3"`; raw extension preserved |
| TV-BC-2.02.006-010 | DeviceRecord with `tags = ["HIPAA", "PCI-scope"]` | `tags` column = JSON array string `["HIPAA","PCI-scope"]`; reflects merged tag_store output (BC-3.2.001) |
| TV-BC-2.02.006-011 | DeviceRecord with `device_cves = ["CVE-2024-1234", "CVE-2024-5678"]`; `armis.sensor.toml` `device_cves_first` column entry has `source_path = "$.device_cves[0]"` (post-`S-WAVE-A-ARMIS-SPEC-001` state) | `device_cves` column = JSON array string `["CVE-2024-1234","CVE-2024-5678"]`; `device_cves_first` = `"CVE-2024-1234"` (first element extracted via JSONPath `$.device_cves[0]`). **Pre-fix state** (no `source_path`): `device_cves_first` is absent/null because `DeviceRecord` has no `device_cves_first` field and the generated-records path strips the key. (F-WASE-P66-HIGH-004) |
| TV-BC-2.02.006-012 | Activity endpoint queried before `armis_device_activity` TOML table exists (deferral EC-02-014 in effect) | Query against `armis_device_activity` table name raises spec-engine surface-not-found error; no silent empty result |

## TOML Contract

The following six columns MUST be added to `crates/prism-sensors/specs/armis.sensor.toml` inside the `devices` table `[[tables.columns]]` block (F-SAP2-MED-005 / FB68d resolution). All six fields are verified on the **static-fixture path** of `routes::search::get_search` (canonical pipeline-facing handler, `POST /api/v1/search`) via `serde_json::to_value(&merged)` on the full `DeviceRecord` struct (SAP-2 §Rule 6 wire-emission-site correction; F-WASE-P66-OBS-001). For generated-records path per-column coverage see §Generated-Records Path Coverage below (F-WASE-P68-MED-003 / FB85):

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
  # Note: device_cves_first requires source_path = "$.device_cves[0]" in its column entry —
  # without it the column resolves to absent/null (no DeviceRecord field backing). See EC-02-013.
  # Provides complete CVE context for agent reasoning.
  [[tables.columns]]
  name = "device_cves"
  column_type = "json"
  ocsf_field = "raw_extensions.device_cves"
```

Additionally, the EXISTING `device_cves_first` column entry in `armis.sensor.toml` MUST be amended to add `source_path = "$.device_cves[0]"`. Without this, `device_cves_first` resolves to absent/null because `DeviceRecord` in `prism-dtu-armis/src/types.rs` has no `device_cves_first` field — `serde_json::to_value(&merged)` (static path) omits it; the generated-records path injects it then strips it via `obj.remove("device_cves_first")` in `routes::search::get_search`. The JSONPath `source_path` directive tells the spec-engine to extract the first element of the `device_cves` array (F-WASE-P66-HIGH-004).

**§Generated-Records Path Coverage (F-WASE-P68-MED-003 / FB85):** `get_search §get_search` is dual-path: when `state.fixture_gen_seeded=true` (the F-P2-CRIT-001 guard under `#[cfg(feature = "fixture-gen")]`), the handler serves `state.generated_records` as raw `serde_json::Value` directly — bypassing `DeviceRecord` struct serialization entirely. These raw values originate from `generator::build_asset §build_asset` in `prism-dtu-armis`. Empirical analysis of the `json!` macro in `build_asset §build_asset` confirms the following per-column state on the generated-records path:

| Column | Generated-records path | Notes |
|--------|----------------------|-------|
| `os_version` | **ABSENT** — key not emitted | DTU defect: `build_asset §build_asset` has no `os_version` key |
| `risk_factors` | **ABSENT** — key not emitted | DTU defect: `build_asset §build_asset` has no `risk_factors` key |
| `network_id` | **ABSENT** — key not emitted | DTU defect: `build_asset §build_asset` has no `network_id` key |
| `site` | **PRESENT** — `format!("site-{}", id_index % 5)` for standard assets; `null` for tombstones (`build_tombstone §build_tombstone`) | No defect; null is valid for `site: Option<String>` |
| `tags` | **ABSENT** — key not emitted | DTU defect: `build_asset §build_asset` has no `tags` key |
| `device_cves` | **ABSENT** — key not emitted | DTU defect: `build_asset §build_asset` has no `device_cves` key |

**Contracted obligation — option (a) (production-grade default):** The generated-records path MUST emit all six columns with the same key names as the static-fixture path. Per SAP-2 Rule 6, a field absent from the generated-records path yields path-dependent behavior: the T13 demo seeded path serves absent columns to the LLM agent, silently degrading sensor data quality. Silence is not an acceptable contract (SAP-2 Rule 4). `site` is already compliant. `os_version`, `risk_factors`, `network_id`, `tags`, and `device_cves` are five confirmed DTU defects requiring implementer routing to `prism-dtu-armis::generator`.

**DTU defect — implementer routing required:** The implementer MUST add five keys to `build_asset §build_asset`'s `json!` macro in `prism-dtu-armis::generator`. Suggested values for healthy-device archetypes: `os_version` — a realistic OS version string (can use the same pool-and-stable-offset pattern as `os_name`); `risk_factors` — `[]` for healthy devices, non-empty for `CompromisedEndpoint`; `network_id` — `format!("net-{}", id_index % 10)` deterministic string; `tags` — `[]` (no analyst tags on fresh generated devices); `device_cves` — `[]` for healthy devices, non-empty for `CompromisedEndpoint`. The contract obligation is key PRESENCE with type-compatible values — exact pool content is implementer-determined.

**POL-29 9c anchor for generated-records path MUSTs (F-WASE-P68-MED-003 / FB85):** Five new MUSTs for the generated-records path are anchored to `S-WAVE-A-ARMIS-SPEC-001`. story-writer MUST add the following ACs and Red Gate tests and expand the story scope to include `prism-dtu-armis` (crate `prism-dtu-armis`, `generator::build_asset §build_asset`):

- **AC-008** (`os_version` generated-records parity): `get_search §get_search` generated-records path emits `os_version` key in device records. **RG-008:** `test_armis_dtu_get_search_generated_records_device_has_os_version` — issues `GET /api/v1/search?aql=in:devices` against an `ArmisState` with `fixture_gen_seeded=true` and at least one asset record; asserts serialized JSON response device record has `"os_version"` key present (wire-shape assertion, CLAUDE.md §Wire-shape assertion discipline).
- **AC-009** (`risk_factors` generated-records parity): generated-records path emits `risk_factors` key. **RG-009:** `test_armis_dtu_get_search_generated_records_device_has_risk_factors` — same pattern as RG-008; asserts `"risk_factors"` key present in serialized device record.
- **AC-010** (`network_id` generated-records parity): generated-records path emits `network_id` key. **RG-010:** `test_armis_dtu_get_search_generated_records_device_has_network_id`.
- **AC-011** (`tags` generated-records parity): generated-records path emits `tags` key. **RG-011:** `test_armis_dtu_get_search_generated_records_device_has_tags`.
- **AC-012** (`device_cves` generated-records parity): generated-records path emits `device_cves` key. **RG-012:** `test_armis_dtu_get_search_generated_records_device_has_device_cves`.

**POL-29 9c anchor (F-WASE-P66-HIGH-002 + F-WASE-P66-HIGH-004):** The 7 TOML-column-spec MUSTs in this §TOML Contract section (six column additions and `device_cves_first` source_path fix) are anchored to `S-WAVE-A-ARMIS-SPEC-001` v1.3 (story exists; `tdd_mode: strict`; SAC-1 compliant; 12 ACs / 12 RGTs total). The 5 generated-records path MUSTs (AC-008..AC-012 / RG-008..RG-012) are covered by the §Generated-Records Path Coverage anchor block above. Together the two POL-29 9c blocks account for all 12 ACs and 12 RGTs in `S-WAVE-A-ARMIS-SPEC-001` v1.3 without overlap or gap. The 7 TOML-column-spec AC/RGT assignments (AC-001..AC-007 / RG-001..RG-007):

- **AC-001** (`os_version` column): `armis.sensor.toml` `devices` table declares `os_version` column (`column_type = "string"`, `ocsf_field = "device.os.version"`). **RG-001:** test asserting parsed `armis.sensor.toml` spec has an `os_version` column with `ColumnType::String` in the `devices` table, mirroring the existing column Red Gate test pattern.
- **AC-002** (`risk_factors` column): `armis.sensor.toml` `devices` table declares `risk_factors` column (`column_type = "json"`, `ocsf_field = "raw_extensions.risk_factors"`). **RG-002:** test asserting parsed spec has a `risk_factors` column with `ColumnType::Json`.
- **AC-003** (`network_id` column): `armis.sensor.toml` `devices` table declares `network_id` column (`column_type = "string"`, `ocsf_field = "raw_extensions.network_id"`). **RG-003:** test asserting parsed spec has a `network_id` column with `ColumnType::String`.
- **AC-004** (`site` column): `armis.sensor.toml` `devices` table declares `site` column (`column_type = "string"`, `ocsf_field = "raw_extensions.site"`). **RG-004:** test asserting parsed spec has a `site` column with `ColumnType::String`.
- **AC-005** (`tags` column): `armis.sensor.toml` `devices` table declares `tags` column (`column_type = "json"`, `ocsf_field = "raw_extensions.tags"`). **RG-005:** test asserting parsed spec has a `tags` column with `ColumnType::Json`.
- **AC-006** (`device_cves` column): `armis.sensor.toml` `devices` table declares `device_cves` column (`column_type = "json"`, `ocsf_field = "raw_extensions.device_cves"`). **RG-006:** test asserting parsed spec has a `device_cves` column with `ColumnType::Json`.
- **AC-007** (`device_cves_first` source_path fix): `armis.sensor.toml` `device_cves_first` column entry has `source_path = "$.device_cves[0]"`. **RG-007:** test asserting parsed spec `device_cves_first` column carries `source_path = "$.device_cves[0]"`, distinct from the full-array `device_cves` column.

**Story `S-WAVE-A-ARMIS-SPEC-001` v1.3 AC/RGT assignments verified for AC-001..AC-007 / RG-001..RG-007 (originally confirmed at v1.0 per FB73 — ITEM 4; all seven survive unchanged through v1.3). POL-29 9c discharged for 7 TOML-column-spec MUSTs (AC-001..AC-007):**
- AC-001 / RG-001: `test_armis_toml_devices_table_has_os_version_column_string_type` — anchored to `S-WAVE-A-ARMIS-SPEC-001`
- AC-002 / RG-002: `test_armis_toml_devices_table_has_risk_factors_column_json_type` — anchored
- AC-003 / RG-003: `test_armis_toml_devices_table_has_network_id_column_string_type` — anchored
- AC-004 / RG-004: `test_armis_toml_devices_table_has_site_column_string_type` — anchored
- AC-005 / RG-005: `test_armis_toml_devices_table_has_tags_column_json_type` — anchored
- AC-006 / RG-006: `test_armis_toml_devices_table_has_device_cves_column_json_type` — anchored
- AC-007 / RG-007: `test_armis_toml_device_cves_first_column_has_source_path_device_cves_0` — anchored

All assignments confirmed from `S-WAVE-A-ARMIS-SPEC-001` v1.3 §Tasks RG-001..RG-007 and AC-001..AC-007 rows (originally authored FB72 leg 1 Item 6; present and unchanged in all versions through v1.3). Combined with the §Generated-Records Path Coverage anchor block above (AC-008..AC-012 / RG-008..RG-012), POL-29 9c is fully discharged for all 12 ACs and 12 RGTs in `S-WAVE-A-ARMIS-SPEC-001` v1.3.

The TOML block comment enumerating `DeviceRecord` fields (`# DeviceRecord fields per prism-dtu-armis/src/types.rs: device_id (String), name, ip_address, mac_address, device_type, manufacturer, os_name, os_version, risk_score, risk_factors, last_seen, first_seen, network_id, site, tags`) must be updated to reflect that all listed fields now have corresponding columns (os_version, risk_factors, network_id, site, tags — previously present in the comment but absent from columns).

`armis_device_activity` TOML table: deliberately NOT authored in this BC per the feature-order deferral documented in EC-02-014. The `ActivityRecord` / `ActivityResponse` / `ActivityData` types exist in `prism-dtu-armis::types` and the route `GET /api/v1/devices/{device_id}/activity` is registered. **Resolution per ADR-057 (2026-07-27):** the blocking architectural question (push-down grammar) is RESOLVED — the correct grammar is `path_template = "/api/v1/devices/${query.filter.device_id}/activity"` with `device_id` column `options = ["INDEX"]`. Full TOML table spec and behavioral contract are specified in BC-2.02.014 and implemented by `S-WAVE-A-ARMIS-ACTIVITY-001`. The implementing story is UNBLOCKED.

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |
| VP-017 | OCSF normalization: unmapped fields preserved (proptest) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.12 | FB87 | 2026-07-28 | product-owner | LOW-002 (F-WASE-P68-LOW-002) — stale v1.0 pin in §TOML Contract POL-29 9c anchor block. Adjudication: option (a) — advance pins to current story version v1.3 and reconcile framing for 12-AC/12-RGT coherence. (1) §TOML Contract POL-29 9c header updated: "proposed story" and "story-writer MUST create" language removed (story exists at v1.3); 7-MUST block scope made explicit (AC-001..AC-007 / RG-001..RG-007 — TOML column spec MUSTs); reference added to §Generated-Records Path Coverage anchor block (AC-008..AC-012 / RG-008..RG-012) so two blocks together account for all 12 without overlap or gap. (2) Block header: "v1.0" → "v1.3"; "discharged for all 7 MUSTs" → "discharged for 7 TOML-column-spec MUSTs (AC-001..AC-007)"; historical FB73 origin note retained. (3) Closing sentence: "v1.0 §Tasks" → "v1.3 §Tasks"; historical origin (FB72 leg 1 Item 6) retained; full-12 discharge summary added referencing §Generated-Records Path Coverage block. POL-29: 9a — BC-2.02.014 is the named twin (device-activity surface vs devices surface); BC-2.02.006's emission cite (`serde_json::to_value(&merged)` on each `DeviceRecord` individually in `routes::search::get_search §get_search` static-fixture path) verified correct from source — this per-record `to_value` feeds a `Vec<serde_json::Value>` that the outer `Json(SearchResponse)` wraps; correct and distinct from the `serde_json::to_value(&body)` misstatement corrected in BC-2.02.014 v1.3 (same burst); 9a CLEAR. 9b — §TOML Contract POL-29 9c block is prose-only; no downstream verbatim copy target identified. 9c — no new unanchored MUSTs introduced. |
| 1.11 | FB85 | 2026-07-28 | product-owner | F-WASE-P68-MED-003 — corrected dual-path coverage gap. §Postconditions "All six are emitted" sentence now qualifies as **static-fixture path** and adds forward reference to §Generated-Records Path Coverage. §TOML Contract preamble "All fields are emitted" replaced with "All six fields are verified on the static-fixture path" and forward reference added. New §Generated-Records Path Coverage subsection inserted (between `device_cves_first` paragraph and existing POL-29 9c block) documenting empirical analysis of `build_asset §build_asset` in `prism-dtu-armis::generator`: `site` PRESENT (`format!("site-{}", id_index % 5)` standard assets; `null` tombstones via `build_tombstone §build_tombstone`); five columns ABSENT (`os_version`, `risk_factors`, `network_id`, `tags`, `device_cves`) — confirmed DTU defects requiring implementer routing to `prism-dtu-armis::generator`. Contracted option (a): all six MUST be present on generated-records path. Five new MUSTs anchored to `S-WAVE-A-ARMIS-SPEC-001` AC-008/RG-008 through AC-012/RG-012; story-writer must expand story scope to include `prism-dtu-armis`. POL-29: 9a — BC-2.02.004 is Cyberint alert field mapping (not Armis alerts); no Armis-alerts BC found in BC inventory; sweep CLEAR; 9b — §Generated-Records Path Coverage block is self-contained in this file; no downstream verbatim copy target identified; 9c — five new MUSTs anchored to `S-WAVE-A-ARMIS-SPEC-001` AC-008..AC-012 / RG-008..RG-012. LOW-002 (stale `v1.0` pin in existing POL-29 9c block, FB87) deliberately untouched. |
| 1.10 | FB81 | 2026-07-28 | product-owner | F-WASE-P68-HIGH-003 — ADR-033 T1 mis-citation corrected in §Postconditions armis_device_activity deferral bullet (dimension 9b downstream-copy-target fix). "via the push-down extraction path (ADR-033 T1)" replaced with annotation-agnostic routing description aligned to ADR-057 §D5 v0.5: `device_id` column `options = ["INDEX"]` declares push-down eligibility per BC-2.11.007 taxonomy (REQUIRED / INDEX / ADDITIONAL) for future T2 (`classify_predicates §classify_predicates`) integration; current routing is annotation-agnostic via `predicate_tree_to_filter_map §predicate_tree_to_filter_map` → `FetchContext.query_filters` → `execute_impl §execute_impl` pre-seed (ADR-057 §D4). Phrasing aligned with ADR-057 §D5 v0.5 corrected source. POL-29: 9a — BC-2.02.006 is parent of BC-2.02.014, not a split-event twin; no sibling pair sweep required; 9b — this edit IS the downstream-copy-target fix (§Postconditions armis_device_activity deferral bullet propagated the T1 mis-citation from ADR-057 §D5 pre-correction source); confirmed by `.factory/specs/` ADR-033-T1 grep that no further copies of the false claim exist outside this BC and BC-2.02.014; 9c — no new unanchored MUSTs introduced (the existing MUST for `options = ["INDEX"]` retains its story anchor from v1.9). |
| 1.9 | FB73 | 2026-07-27 | product-owner | ITEM 1: corrected §Postconditions `armis_device_activity` deferral bullet — replaced phantom `${step_name.field}` grammar (authored in error by FB71) with adjudicated `${query.filter.device_id}` push-down grammar per ADR-057 §D5; removed stale per-scalar-vs-per-batch handoff note and BLOCKED status; documented single-device filter-required scope, hard-error on absent filter, ADR-057 §D6 future fan-out gap, UNBLOCKED story status, and reference to BC-2.02.014. Ground truth quote: shipping exemplar `path_template = "/api/v1/search?aql=${query.filter.aql}"` in `crates/prism-sensors/specs/armis.sensor.toml`; engine pre-seed via `F-LP1-HIGH-004` `for (k, v) in &context.query_filters { step_vars.insert(format!("query.filter.{k}"), ...) }` in `pipeline.rs §execute_impl`. ITEM 2: updated EC-02-014 — grammar RESOLVED, scope single-device filter-required, story UNBLOCKED, per-record fan-out gap documented with ADR-057 §D6 anchor, SAP-2 re-mint suppression language updated. ITEM 4: POL-29 9c anchor backfill — seven MUSTs in §TOML Contract now discharged against `S-WAVE-A-ARMIS-SPEC-001` v1.0 AC-001..AC-007 / RG-001..RG-007, verified from story v1.0 §Tasks. §TOML Contract activity-table deferral note updated to reflect ADR-057 resolution. POL-29 9a: no named twin (BC-2.02.003/004/005 are distinct sensors). 9b: ADR-057 §D5 is a verbatim copy-source for the `path_template` text transcribed into §Postconditions — swept faithfully, no stale copy-source section retained. 9c: all seven MUSTs now carry real story+AC+RGT anchors; BC-2.02.014 MUST (required-filter implementation) anchored to `S-WAVE-A-ARMIS-ACTIVITY-001`. |
| 1.8 | FB71 | 2026-07-27 | product-owner | F-WASE-P66-OBS-001: corrected wire-emission site from `routes::devices::paginate_devices` to `routes::search::get_search` (canonical pipeline-facing handler for `from armis.devices` per SAP-2 §Rule 6; two sites: §Postconditions DeviceRecord bullet + §TOML Contract preamble). F-WASE-P66-HIGH-005: replaced phantom `${variable.device_id}` interpolation namespace (zero occurrences workspace-wide) with real `${step_name.field}` grammar per `prism-spec-engine/src/interpolation.rs`; added per-scalar-vs-per-batch architectural handoff note; `S-WAVE-A-ARMIS-ACTIVITY-001` BLOCKED status documented. F-WASE-P66-HIGH-004: corrected EC-02-013 (false "generator-projection semantics" claim; real mechanism: no `DeviceRecord` field, static path omits, generated path strips via `obj.remove`; required remedy: `source_path = "$.device_cves[0]"`); corrected TV-BC-2.02.006-011 (removed false `device_cves_first = "CVE-2024-1234"` assertion; pre-fix/post-fix states distinguished); added `device_cves_first` source_path MUST to §TOML Contract. F-WASE-P66-HIGH-002: added POL-29 9c anchor block to §TOML Contract naming proposed `S-WAVE-A-ARMIS-SPEC-001` with 7 ACs and 7 RGTs (6 per column + 1 for device_cves_first source_path). POL-29 9a: no named twin (distinct sensor); 9b: §TOML Contract references code block — no known downstream verbatim copy; 9c: story-writer handoff documented with exact AC and RGT content for proposed `S-WAVE-A-ARMIS-SPEC-001`. |
| 1.7 | wave-5-e-demo-fidelity-fix-burst-68d | 2026-07-27 | product-owner | F-SAP2-MED-005 + F-SAP2-MED-006: Armis DeviceRecord field coverage decisions and activity surface deferral. MED-005 ground truth: all six fields (`os_version: Option<String>`, `risk_factors: Vec<String>`, `network_id: Option<String>`, `site: Option<String>`, `tags: Vec<String>`, `device_cves: Vec<String>`) are emitted on the wire by `routes::devices::paginate_devices` static-fixture path via `serde_json::to_value(&merged)` on the full `DeviceRecord` struct. Decisions: EXPOSE all six — see §Postconditions for per-field OCSF mappings and agent-reasoning rationale. Risk_factors specifically called out as the explanatory companion of risk_score. MED-006: `armis_device_activity` TOML table DEFERRED to `S-WAVE-A-ARMIS-ACTIVITY-001` (parameterized per-device fan-out pattern requires spec-engine validation). Added: §Postconditions (6 DeviceRecord field bullets + activity deferral); §Edge Cases EC-02-011..014; §Canonical Test Vectors TV-006..012; §TOML Contract (exact column specs for implementer + activity surface deferral note). |
| 1.6 | wave-a-spec-evolution-fix-burst-38 | 2026-07-24 | product-owner | F-WASE-P49-LOW-001 sibling-sweep extension: `scheduled_amendment_in` cleared (ADR-023 amendment completed in v1.5 PLUGIN-MIGRATION-001-G, 2026-05-27); set to `null`; added `amendment_lifecycle: null` per BC-2.01.006 cleared-state convention. |
| 1.5 | PLUGIN-MIGRATION-001-G | 2026-05-27 | product-owner | AC-002 amendment: removed PENDING AMENDMENT banner; added Amendment Note to Description; updated mechanism language from deleted `prism-ocsf/src/mappers/armis.rs` to SpecDrivenMapper + ocsf_field TOML annotations; removed adapter reference from timestamp fallback prose; bumped status draft→active; removed amendment_lifecycle: pending. |
| 1.4 | prereq-f | 2026-05-11 | product-owner | PREREQ-F prefix note: added PENDING AMENDMENT — ADR-023 callout under H1 per ADR-023 L370 wording; added scheduled_amendment_in: ADR-023 and amendment_lifecycle: pending to frontmatter. No semantic change to BC body. Full amendment in Wave 2/G. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
