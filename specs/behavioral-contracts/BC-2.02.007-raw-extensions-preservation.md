---
document_type: behavioral-contract
level: L3
version: "1.3"
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
input-hash: "248b3b0"
traces_to: ["CAP-003"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.007: Vendor Extension Preservation in raw_extensions

## Description

Any sensor record field that has no defined OCSF mapping is preserved verbatim in the `raw_extensions` JSON blob attached to the `OcsfEvent`, using the original vendor field name. This ensures no vendor data is silently discarded during normalization, and that AI agents can access vendor-specific context through structured response content. If `raw_extensions` exceeds 1MB, it is truncated with a `_truncated: true` marker and a warning log naming the largest fields.

## Preconditions
- A sensor record is being normalized to OCSF
- The record contains fields that have no OCSF mapping defined in the per-sensor mapper

## Postconditions
- All unmapped vendor-specific fields are preserved in the `raw_extensions` JSON blob
- `raw_extensions` is a `serde_json::Value` (JSON object) attached to the `OcsfEvent`
- Field names in `raw_extensions` use the original vendor field names (not transformed)
- `raw_extensions` is accessible to the AI agent via structured response content
- No vendor data is silently dropped during normalization

## Invariants
- DI-005: Unknown fields preserved in `raw_extensions`, not silently dropped

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | `raw_extensions` JSON blob exceeds 1MB | Logged as warning; blob truncated with `_truncated: true` marker; largest fields listed in warning |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-007 | CrowdStrike `custom_tags` field | Preserved in `raw_extensions.custom_tags`; debug-level log records the unmapped field |
| EC-02-011 | Sensor record consists entirely of unmapped fields | Valid OCSF message with empty mapped fields; entire record content in `raw_extensions` |
| EC-02-012 | Vendor field name collides with an OCSF-mapped field due to naming | Both preserved: mapped value in OCSF message, original in `raw_extensions` with `_vendor_` prefix |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.007-001 | CrowdStrike alert with `custom_tags` field | `raw_extensions.custom_tags` set; debug log entry; field not dropped |
| TV-BC-2.02.007-002 | Record with all unmapped fields | Valid OCSF message (minimal); all fields in `raw_extensions` |
| TV-BC-2.02.007-003 | `raw_extensions` blob = 1.1MB | Truncated to fit; `_truncated: true` marker; largest fields in warning log |
| TV-BC-2.02.007-004 | Vendor field name matches an OCSF-mapped field | Mapped value in OCSF message; vendor original in `raw_extensions` with `_vendor_` prefix |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
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
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties with VP-017; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
