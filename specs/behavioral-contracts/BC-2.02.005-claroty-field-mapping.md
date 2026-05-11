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

# BC-2.02.005: Claroty xDome Field Mapping to OCSF (9 Data Sources)

> **PENDING AMENDMENT — ADR-023**: This BC is being amended for plugin-only architecture per ADR-023. The sensor auth and field-mapping behavior described here will be replaced by TOML spec configuration and, where required, `.prx` WASM plugins. Full BC amendment language is authored in PLUGIN-MIGRATION-001-G (Wave 2/G). See PLUGIN-MIGRATION-001-D for replacement TOMLs.

## Description

The Claroty normalizer handles 9 distinct xDome data sources, mapping each to an appropriate OCSF event class: alerts to Security Finding (2004), devices to Device Inventory Info (5001), vulnerabilities to Vulnerability Finding (2002), and audit logs to Audit Activity (3001). Polymorphic IDs are pre-normalized by the Claroty adapter before field mapping occurs. OT-specific fields (e.g., `zone`, `protocol`, `firmware_version`) with no OCSF equivalent are preserved in `raw_extensions`.

## Preconditions
- A Claroty xDome record has been fetched from one of the 9 endpoints
- Polymorphic IDs have been normalized by the Claroty adapter

## Postconditions
- Claroty `device_name` maps to OCSF `device.hostname`
- Claroty device IP fields map to OCSF `device.ip`
- Claroty alert severity maps to OCSF `severity_id`
- Claroty OT-specific fields (e.g., `zone`, `protocol`, `firmware_version`) are preserved in `raw_extensions`
- Each of the 9 Claroty sources maps to an appropriate OCSF event class (alerts to Security Finding, devices to Inventory Info, vulnerabilities to Vulnerability Finding)

## Invariants
- DI-005: OCSF schema validity

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | Claroty record with polymorphic ID that cannot be stringified | ID placed in `raw_extensions` as raw JSON; OCSF ID field left absent |
| Warning | Claroty source type has no defined OCSF event class mapping | Record normalized to generic OCSF Base Event (class 0); all fields go to `raw_extensions` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-007 | Claroty `device_alert_relations` records (join table, not primary entity) | Mapped to OCSF with both device and alert references in the message; primarily useful for correlation |
| EC-02-008 | Claroty audit_log records (admin actions, not security events) | Mapped to OCSF Audit Activity (class 3001); admin-specific fields in `raw_extensions` |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.005-001 | Claroty alert record with severity and device_name | Mapped to Detection Finding 2004; `severity_id` and `device.hostname` set |
| TV-BC-2.02.005-002 | Claroty device record with OT fields (zone, protocol) | Mapped to Device Inventory Info 5001; `zone` and `protocol` in `raw_extensions` |
| TV-BC-2.02.005-003 | Claroty vulnerability record | Mapped to Vulnerability Finding 2002; CVE fields mapped |
| TV-BC-2.02.005-004 | Claroty audit_log record | Mapped to Audit Activity 3001; admin action fields in `raw_extensions` |
| TV-BC-2.02.005-005 | Unknown Claroty source type | Falls back to Base Event class 0; all fields in `raw_extensions`; warning logged |

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
