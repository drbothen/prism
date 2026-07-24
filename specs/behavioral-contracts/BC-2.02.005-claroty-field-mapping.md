---
document_type: behavioral-contract
level: L3
version: "1.6"
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
modified: "2026-07-24"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.005: Claroty xDome Field Mapping to OCSF (9 Data Sources)

## Description

> **Amendment — ADR-023 (PLUGIN-MIGRATION-001-G):** This BC previously described a
> hardcoded Rust mapper module (`prism-ocsf/src/mappers/claroty.rs`). That
> implementation was deleted in PLUGIN-MIGRATION-001-C (PR #158). The field-mapping behavior
> described here is now delivered by `SpecDrivenMapper` reading `ocsf_field` column
> annotations from the Claroty TOML sensor spec. The behavioral contract itself
> is unchanged — the same OCSF field mappings must be produced; they are now
> data-driven via TOML annotations per ADR-023 Rule 1.

`SpecDrivenMapper` reads `ocsf_field` column annotations from the Claroty TOML sensor spec and handles 9 distinct xDome data sources, mapping each to an appropriate OCSF event class: alerts to Security Finding (2004), devices to Device Inventory Info (5001), vulnerabilities to Vulnerability Finding (2002), and audit logs to Audit Activity (3001). Polymorphic IDs are pre-normalized before field mapping occurs. OT-specific fields (e.g., `zone`, `protocol`, `firmware_version`) with no OCSF equivalent are preserved in `raw_extensions`.

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
| 1.6 | wave-a-spec-evolution-fix-burst-38 | 2026-07-24 | product-owner | F-WASE-P49-LOW-001 sibling-sweep extension: `scheduled_amendment_in` cleared (ADR-023 amendment completed in v1.5 PLUGIN-MIGRATION-001-G, 2026-05-27); set to `null`; added `amendment_lifecycle: null` per BC-2.01.006 cleared-state convention. |
| 1.5 | PLUGIN-MIGRATION-001-G | 2026-05-27 | product-owner | AC-002 amendment: removed PENDING AMENDMENT banner; added Amendment Note to Description; updated mechanism language from deleted `prism-ocsf/src/mappers/claroty.rs` to SpecDrivenMapper + ocsf_field TOML annotations; updated Description to remove adapter reference; bumped status draft→active; removed amendment_lifecycle: pending. |
| 1.4 | prereq-f | 2026-05-11 | product-owner | PREREQ-F prefix note: added PENDING AMENDMENT — ADR-023 callout under H1 per ADR-023 L370 wording; added scheduled_amendment_in: ADR-023 and amendment_lifecycle: pending to frontmatter. No semantic change to BC body. Full amendment in Wave 2/G. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
