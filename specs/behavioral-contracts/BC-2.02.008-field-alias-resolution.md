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
input-hash: "1e29f9d"
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

# BC-2.02.008: Four-Tier Field Alias Resolution

## Description

Field access on an `OcsfEvent` follows a deterministic four-tier priority order: (1) Prism metadata fields (e.g., `source_sensor`, `client_id`), (2) OCSF protobuf fields via recursive `DynamicMessage` descent using dot notation, (3) `raw_extensions` JSON blob fields by vendor name, (4) `None`. The first tier producing a value wins. This resolution order is invariant and ensures predictable behavior when field names overlap across tiers.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.008-001 | Access `source_sensor` (Prism metadata, tier 1) | Returns Prism metadata value; OCSF proto not consulted |
| TV-BC-2.02.008-002 | Access `device.hostname` (OCSF proto, tier 2) | Resolved via DynamicMessage descent |
| TV-BC-2.02.008-003 | Access `custom_vendor_field` (raw_extensions, tier 3) | Returns `raw_extensions["custom_vendor_field"]` |
| TV-BC-2.02.008-004 | Field `time` exists in both tier 1 and tier 2 | Tier 1 wins; Prism metadata `time` returned |
| TV-BC-2.02.008-005 | `attacks[5].technique.name` where index 5 out of bounds | Returns `None`; no error |
| TV-BC-2.02.008-006 | Field not found in any tier | Returns `None`; no error |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

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
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
