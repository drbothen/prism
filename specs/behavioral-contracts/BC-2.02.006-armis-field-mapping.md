---
document_type: behavioral-contract
level: L3
version: "1.7"
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
- Six previously undeclared `DeviceRecord` fields are now covered (F-SAP2-MED-005 / FB68d). All six are emitted on the wire by `routes::devices::paginate_devices` static-fixture path via `serde_json::to_value(&merged)` on the full `DeviceRecord` struct. Coverage decisions and OCSF mappings:
  - `os_version: Option<String>` → TOML column `os_version`, `column_type = "string"`, `ocsf_field = "device.os.version"`. SOC analyst value: OS version is required to assess CVE applicability; `os_name` without `os_version` leaves the agent unable to determine patch posture.
  - `risk_factors: Vec<String>` → TOML column `risk_factors`, `column_type = "json"`, `ocsf_field = "raw_extensions.risk_factors"`. This is the explanatory companion of `risk_score` — a numeric risk score without its factor labels is an unexplainable verdict. An SOC agent reading `risk_score = 85` with no `risk_factors = ["unpatched_cve","open_ports"]` cannot prioritize or explain the finding.
  - `network_id: Option<String>` → TOML column `network_id`, `column_type = "string"`, `ocsf_field = "raw_extensions.network_id"`. SOC analyst value: network segment membership is essential for blast-radius and lateral movement assessment.
  - `site: Option<String>` → TOML column `site`, `column_type = "string"`, `ocsf_field = "raw_extensions.site"`. SOC analyst value: physical/logical deployment site enables geographic/organizational scoping of incidents (production vs test, HQ vs remote).
  - `tags: Vec<String>` → TOML column `tags`, `column_type = "json"`, `ocsf_field = "raw_extensions.tags"`. Tags are analyst-managed device labels (e.g., "HIPAA", "PCI-scope", "critical-infra") — they directly improve SOC agent classification and response prioritization. Note: at query time, tags are merged with the per-org `tag_store` by `routes::devices::paginate_devices` (BC-3.2.001).
  - `device_cves: Vec<String>` → TOML column `device_cves`, `column_type = "json"`, `ocsf_field = "raw_extensions.device_cves"`. Provides the full CVE ID array for the device, complementing the existing `device_cves_first` scalar column (used for enrichment UDF input). The agent receives complete CVE context, not only the first element.
- `armis_device_activity` surface (DTU route `GET /api/v1/devices/{device_id}/activity`, types `ActivityRecord` / `ActivityResponse` / `ActivityData`) is DELIBERATELY DEFERRED to story `S-WAVE-A-ARMIS-ACTIVITY-001` (to be created). This is a feature-order deferral per Canonical Principle Rule 2, not a perpetual exclusion. Rationale: the activity endpoint is parameterized by `device_id` and requires a per-device fan-out fetch pattern (`path_template = "/api/v1/devices/${variable.device_id}/activity"`) that must be validated against the spec-engine fan-out contract before TOML authoring. The surface is architecturally distinct from the existing flat-table AQL-driven queries. SAP-2 deliberate-exclusion is documented in EC-02-014 below.

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
| EC-02-013 | `device_cves` is an empty array `[]`; `device_cves_first` is absent or null | `device_cves` column emits `"[]"` (not null); `device_cves_first` emits null per its existing generator-projection semantics. Normal case for devices with no unpatched CVEs. |
| EC-02-014 | `armis_device_activity` surface — SAP-2 deliberate-exclusion tracking sentinel | DTU route `GET /api/v1/devices/{device_id}/activity` exists and serves `ActivityRecord` / `ActivityResponse` / `ActivityData` (types in `prism-dtu-armis::types`). No TOML table is declared in `armis.sensor.toml` in wave-a-spec-evolution. This exclusion is DELIBERATE and TEMPORARY: the surface requires a per-device parameterized fan-out pattern that must be validated against the spec-engine before TOML authoring. Deferral target: `S-WAVE-A-ARMIS-ACTIVITY-001` (new story to be created). When that story ships and the `armis_device_activity` TOML table is authored, this EC must be updated to reflect the resolved state. A future SAP-2 pass that finds "no TOML table for armis_device_activity" should NOT re-mint a finding if this EC is present — the exclusion is documented and has a concrete resolution anchor. |

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
| TV-BC-2.02.006-011 | DeviceRecord with `device_cves = ["CVE-2024-1234", "CVE-2024-5678"]` | `device_cves` column = JSON array string `["CVE-2024-1234","CVE-2024-5678"]`; `device_cves_first` = `"CVE-2024-1234"` (existing scalar column) |
| TV-BC-2.02.006-012 | Activity endpoint queried before `armis_device_activity` TOML table exists (deferral EC-02-014 in effect) | Query against `armis_device_activity` table name raises spec-engine surface-not-found error; no silent empty result |

## TOML Contract

The following six columns MUST be added to `crates/prism-sensors/specs/armis.sensor.toml` inside the `devices` table `[[tables.columns]]` block (F-SAP2-MED-005 / FB68d resolution). All fields are emitted by the static-fixture path of `routes::devices::paginate_devices` via `serde_json::to_value(&merged)` on the full `DeviceRecord` struct:

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
  # Provides complete CVE context for agent reasoning.
  [[tables.columns]]
  name = "device_cves"
  column_type = "json"
  ocsf_field = "raw_extensions.device_cves"
```

The TOML block comment enumerating `DeviceRecord` fields (`# DeviceRecord fields per prism-dtu-armis/src/types.rs: device_id (String), name, ip_address, mac_address, device_type, manufacturer, os_name, os_version, risk_score, risk_factors, last_seen, first_seen, network_id, site, tags`) must be updated to reflect that all listed fields now have corresponding columns (os_version, risk_factors, network_id, site, tags — previously present in the comment but absent from columns).

`armis_device_activity` TOML table: deliberately NOT authored in this fix-burst per the feature-order deferral documented in EC-02-014. The `ActivityRecord` / `ActivityResponse` / `ActivityData` types exist in `prism-dtu-armis::types` and the route `GET /api/v1/devices/{device_id}/activity` is registered, but the per-device parameterized fan-out pattern requires spec-engine validation first. Deferred to `S-WAVE-A-ARMIS-ACTIVITY-001`.

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
| 1.7 | wave-5-e-demo-fidelity-fix-burst-68d | 2026-07-27 | product-owner | F-SAP2-MED-005 + F-SAP2-MED-006: Armis DeviceRecord field coverage decisions and activity surface deferral. MED-005 ground truth: all six fields (`os_version: Option<String>`, `risk_factors: Vec<String>`, `network_id: Option<String>`, `site: Option<String>`, `tags: Vec<String>`, `device_cves: Vec<String>`) are emitted on the wire by `routes::devices::paginate_devices` static-fixture path via `serde_json::to_value(&merged)` on the full `DeviceRecord` struct. Decisions: EXPOSE all six — see §Postconditions for per-field OCSF mappings and agent-reasoning rationale. Risk_factors specifically called out as the explanatory companion of risk_score. MED-006: `armis_device_activity` TOML table DEFERRED to `S-WAVE-A-ARMIS-ACTIVITY-001` (parameterized per-device fan-out pattern requires spec-engine validation). Added: §Postconditions (6 DeviceRecord field bullets + activity deferral); §Edge Cases EC-02-011..014; §Canonical Test Vectors TV-006..012; §TOML Contract (exact column specs for implementer + activity surface deferral note). |
| 1.6 | wave-a-spec-evolution-fix-burst-38 | 2026-07-24 | product-owner | F-WASE-P49-LOW-001 sibling-sweep extension: `scheduled_amendment_in` cleared (ADR-023 amendment completed in v1.5 PLUGIN-MIGRATION-001-G, 2026-05-27); set to `null`; added `amendment_lifecycle: null` per BC-2.01.006 cleared-state convention. |
| 1.5 | PLUGIN-MIGRATION-001-G | 2026-05-27 | product-owner | AC-002 amendment: removed PENDING AMENDMENT banner; added Amendment Note to Description; updated mechanism language from deleted `prism-ocsf/src/mappers/armis.rs` to SpecDrivenMapper + ocsf_field TOML annotations; removed adapter reference from timestamp fallback prose; bumped status draft→active; removed amendment_lifecycle: pending. |
| 1.4 | prereq-f | 2026-05-11 | product-owner | PREREQ-F prefix note: added PENDING AMENDMENT — ADR-023 callout under H1 per ADR-023 L370 wording; added scheduled_amendment_in: ADR-023 and amendment_lifecycle: pending to frontmatter. No semantic change to BC body. Full amendment in Wave 2/G. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
