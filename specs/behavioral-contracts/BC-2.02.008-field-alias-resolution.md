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

# BC-2.02.008: Three-Tier Field Alias Resolution

## Preconditions
- A field is being accessed on an `OcsfEvent` (either for query or display purposes)
- The field name may be a Prism metadata name, an OCSF proto field name, or a vendor-specific name

## Postconditions
- Field resolution follows the four-tier priority order:
  1. Prism-specific metadata fields (e.g., `source_sensor`, `source_record_type`, `client_id`)
  2. Proto descriptor fields via recursive descent into `DynamicMessage` (e.g., `device.hostname`, `severity_id`)
  3. Unmapped JSON blob fields from `raw_extensions` (vendor-specific names)
  4. `None` (field absent)
- The first tier that produces a value wins; later tiers are not consulted
- Nested OCSF fields use dot notation (e.g., `device.hostname` resolves by descending into the `device` sub-message)

## Invariants
- Resolution order is deterministic and documented; same input always produces same output

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Field name not found in any tier | Returns `None`; not an error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-013 | Field name `time` exists in both Prism metadata and OCSF proto | Prism metadata tier wins (tier 1 > tier 2) |
| EC-02-014 | Deeply nested OCSF field path (e.g., `attacks[0].technique.name`) | Array indexing supported in resolution; returns `None` if index out of bounds |
| EC-02-015 | `raw_extensions` contains a field with the same name as an OCSF field | OCSF proto field (tier 2) takes precedence over `raw_extensions` (tier 3) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
