---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "OCSF Normalization"
capability: "CAP-003"
---

# BC-2.02.007: Vendor Extension Preservation in raw_extensions

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
