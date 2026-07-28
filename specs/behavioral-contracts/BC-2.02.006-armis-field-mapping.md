---
document_type: behavioral-contract
level: L3
version: "1.9"
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
modified: "2026-07-27"
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
- Six previously undeclared `DeviceRecord` fields are now covered (F-SAP2-MED-005 / FB68d). All six are emitted on the wire by `routes::search::get_search` (canonical pipeline-facing handler for `from armis.devices` queries, `POST /api/v1/search`) via `serde_json::to_value(&merged)` on the full `DeviceRecord` struct. `routes::devices::paginate_devices` (GET /api/v1/devices) also serializes the same `DeviceRecord` but is NOT the pipeline-facing path for PrismQL sensor queries (SAP-2 §Rule 6 wire-emission-site authority correction; F-WASE-P66-OBS-001). Coverage decisions and OCSF mappings:
  - `os_version: Option<String>` → TOML column `os_version`, `column_type = "string"`, `ocsf_field = "device.os.version"`. SOC analyst value: OS version is required to assess CVE applicability; `os_name` without `os_version` leaves the agent unable to determine patch posture.
  - `risk_factors: Vec<String>` → TOML column `risk_factors`, `column_type = "json"`, `ocsf_field = "raw_extensions.risk_factors"`. This is the explanatory companion of `risk_score` — a numeric risk score without its factor labels is an unexplainable verdict. An SOC agent reading `risk_score = 85` with no `risk_factors = ["unpatched_cve","open_ports"]` cannot prioritize or explain the finding.
  - `network_id: Option<String>` → TOML column `network_id`, `column_type = "string"`, `ocsf_field = "raw_extensions.network_id"`. SOC analyst value: network segment membership is essential for blast-radius and lateral movement assessment.
  - `site: Option<String>` → TOML column `site`, `column_type = "string"`, `ocsf_field = "raw_extensions.site"`. SOC analyst value: physical/logical deployment site enables geographic/organizational scoping of incidents (production vs test, HQ vs remote).
  - `tags: Vec<String>` → TOML column `tags`, `column_type = "json"`, `ocsf_field = "raw_extensions.tags"`. Tags are analyst-managed device labels (e.g., "HIPAA", "PCI-scope", "critical-infra") — they directly improve SOC agent classification and response prioritization. Note: at query time, tags are merged with the per-org `tag_store` by `routes::devices::paginate_devices` (BC-3.2.001).
  - `device_cves: Vec<String>` → TOML column `device_cves`, `column_type = "json"`, `ocsf_field = "raw_extensions.device_cves"`. Provides the full CVE ID array for the device, complementing the existing `device_cves_first` scalar column (used for enrichment UDF input). The agent receives complete CVE context, not only the first element.
- `armis_device_activity` surface (DTU route `GET /api/v1/devices/{device_id}/activity`, types `ActivityRecord` / `ActivityResponse` / `ActivityData`) is DELIBERATELY DEFERRED to story `S-WAVE-A-ARMIS-ACTIVITY-001` — the full behavioral contract for this surface is specified in BC-2.02.014. **Resolution per ADR-057 (2026-07-27):** the correct fetch grammar is `path_template = "/api/v1/devices/${query.filter.device_id}/activity"` using the existing `${query.filter.*}` push-down namespace — the same mechanism as `${query.filter.aql}` in the `armis_devices` and `armis_alerts` tables (shipping exemplar in `crates/prism-sensors/specs/armis.sensor.toml`: `path_template = "/api/v1/search?aql=${query.filter.aql}"`). Ground truth: `PipelineExecutor.execute_impl` (block comment `F-LP1-HIGH-004` in `pipeline.rs §execute_impl`) pre-seeds `step_vars` with all `FetchContext.query_filters` entries under the `query.filter.{key}` namespace via `for (k, v) in &context.query_filters { step_vars.insert(format!("query.filter.{k}"), ...) }`. The `device_id` column MUST carry `options = ["INDEX"]` so the query planner routes `WHERE device_id = '...'` predicates into `FetchContext.query_filters["device_id"]` via the push-down extraction path (ADR-033 T1). Scope is single-device, filter-required; an absent `device_id` filter raises `SpecEngineError::HttpRequestFailed` (hard error — no silent empty result; implementation obligation noted in BC-2.02.014 §TOML Contract). Fleet-wide activity iteration is explicitly out of scope for Wave-A and requires the per-record fan-out capability gap (ADR-057 §D6) to be addressed in a future story. The previously noted architectural uncertainty (per-scalar-vs-per-batch, phantom `${step_name.field}` grammar corrected in v1.8 FB71) is fully resolved by ADR-057 §D3: per-record fan-out is confirmed a genuine capability gap, NOT expressible in the current engine. Story `S-WAVE-A-ARMIS-ACTIVITY-001` is UNBLOCKED. SAP-2 deliberate-exclusion status updated in EC-02-014 below.

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

The following six columns MUST be added to `crates/prism-sensors/specs/armis.sensor.toml` inside the `devices` table `[[tables.columns]]` block (F-SAP2-MED-005 / FB68d resolution). All fields are emitted by the static-fixture path of `routes::search::get_search` (canonical pipeline-facing handler, `POST /api/v1/search`) via `serde_json::to_value(&merged)` on the full `DeviceRecord` struct (SAP-2 §Rule 6 wire-emission-site correction; F-WASE-P66-OBS-001):

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

**POL-29 9c anchor (F-WASE-P66-HIGH-002 + F-WASE-P66-HIGH-004):** All MUST obligations in this §TOML Contract section are bound to proposed story `S-WAVE-A-ARMIS-SPEC-001` (Armis sensor spec — six column additions and `device_cves_first` source_path fix). story-writer MUST create `S-WAVE-A-ARMIS-SPEC-001` with the following ACs and Red Gate tests. story-writer MUST set `tdd_mode: strict` and include an enumerated §Red Gate Tests section per SAC-1:

- **AC-001** (`os_version` column): `armis.sensor.toml` `devices` table declares `os_version` column (`column_type = "string"`, `ocsf_field = "device.os.version"`). **RG-001:** test asserting parsed `armis.sensor.toml` spec has an `os_version` column with `ColumnType::String` in the `devices` table, mirroring the existing column Red Gate test pattern.
- **AC-002** (`risk_factors` column): `armis.sensor.toml` `devices` table declares `risk_factors` column (`column_type = "json"`, `ocsf_field = "raw_extensions.risk_factors"`). **RG-002:** test asserting parsed spec has a `risk_factors` column with `ColumnType::Json`.
- **AC-003** (`network_id` column): `armis.sensor.toml` `devices` table declares `network_id` column (`column_type = "string"`, `ocsf_field = "raw_extensions.network_id"`). **RG-003:** test asserting parsed spec has a `network_id` column with `ColumnType::String`.
- **AC-004** (`site` column): `armis.sensor.toml` `devices` table declares `site` column (`column_type = "string"`, `ocsf_field = "raw_extensions.site"`). **RG-004:** test asserting parsed spec has a `site` column with `ColumnType::String`.
- **AC-005** (`tags` column): `armis.sensor.toml` `devices` table declares `tags` column (`column_type = "json"`, `ocsf_field = "raw_extensions.tags"`). **RG-005:** test asserting parsed spec has a `tags` column with `ColumnType::Json`.
- **AC-006** (`device_cves` column): `armis.sensor.toml` `devices` table declares `device_cves` column (`column_type = "json"`, `ocsf_field = "raw_extensions.device_cves"`). **RG-006:** test asserting parsed spec has a `device_cves` column with `ColumnType::Json`.
- **AC-007** (`device_cves_first` source_path fix): `armis.sensor.toml` `device_cves_first` column entry has `source_path = "$.device_cves[0]"`. **RG-007:** test asserting parsed spec `device_cves_first` column carries `source_path = "$.device_cves[0]"`, distinct from the full-array `device_cves` column.

**Story `S-WAVE-A-ARMIS-SPEC-001` v1.0 AC/RGT assignments verified 2026-07-27 (FB73 — ITEM 4). POL-29 9c discharged for all 7 MUSTs:**
- AC-001 / RG-001: `test_armis_toml_devices_table_has_os_version_column_string_type` — anchored to `S-WAVE-A-ARMIS-SPEC-001`
- AC-002 / RG-002: `test_armis_toml_devices_table_has_risk_factors_column_json_type` — anchored
- AC-003 / RG-003: `test_armis_toml_devices_table_has_network_id_column_string_type` — anchored
- AC-004 / RG-004: `test_armis_toml_devices_table_has_site_column_string_type` — anchored
- AC-005 / RG-005: `test_armis_toml_devices_table_has_tags_column_json_type` — anchored
- AC-006 / RG-006: `test_armis_toml_devices_table_has_device_cves_column_json_type` — anchored
- AC-007 / RG-007: `test_armis_toml_device_cves_first_column_has_source_path_device_cves_0` — anchored

All assignments confirmed from `S-WAVE-A-ARMIS-SPEC-001` v1.0 §Tasks RG-001..RG-007 and AC-001..AC-007 rows (authored FB72 leg 1 Item 6).

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
