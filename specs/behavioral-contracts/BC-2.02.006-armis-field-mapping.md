---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
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
scheduled_amendment_in: ADR-023
amendment_lifecycle: pending
introduced: cycle-1
modified: "2026-05-11"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.006: Armis Centrix Field Mapping to OCSF (7 Data Sources)

> **PENDING AMENDMENT — ADR-023**: This BC is being amended for plugin-only architecture per ADR-023. The sensor auth and field-mapping behavior described here will be replaced by TOML spec configuration and, where required, `.prx` WASM plugins. Full BC amendment language is authored in PLUGIN-MIGRATION-001-G (Wave 2/G). See PLUGIN-MIGRATION-001-D for replacement TOMLs.

## Description

The Armis normalizer maps records fetched via AQL GetSearch from 7 Armis sources to appropriate OCSF event classes. Key mappings are `ipaddress` → `device.ip`, `name` → `device.hostname`, alert severity → `severity_id`, and `riskLevel` → OCSF risk score fields. Armis-specific fields (`aqlResults`, `connectionType`, `riskFactors`) and records from sources with no dedicated OCSF class are preserved in `raw_extensions`. Timestamp fallback (pre-processed by the adapter) is used when no primary timestamp field exists.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.006-001 | Armis alert with ipaddress, name, severity | `device.ip`, `device.hostname`, `severity_id` set correctly |
| TV-BC-2.02.006-002 | Armis device with riskLevel | `riskLevel` mapped to OCSF risk score fields |
| TV-BC-2.02.006-003 | Armis connections record | Mapped to Network Activity class; source/dest IPs extracted |
| TV-BC-2.02.006-004 | Record missing all timestamp fallback fields | `time` = fetch timestamp; warning logged |
| TV-BC-2.02.006-005 | Severity in unexpected format | Best-effort mapping; unrecognized in `raw_extensions`; warning logged |

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
| 1.4 | prereq-f | 2026-05-11 | product-owner | PREREQ-F prefix note: added PENDING AMENDMENT — ADR-023 callout under H1 per ADR-023 L370 wording; added scheduled_amendment_in: ADR-023 and amendment_lifecycle: pending to frontmatter. No semantic change to BC body. Full amendment in Wave 2/G. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
