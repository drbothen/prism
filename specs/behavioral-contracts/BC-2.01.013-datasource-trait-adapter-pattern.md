---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "248b3b0"
traces_to: ["CAP-001"]
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

# BC-2.01.013: DataSource Trait Eliminates Per-Sensor Code Duplication

## Description

The `DataSource<T>` generic trait provides a uniform interface for all sensor data feeds, with shared infrastructure handling cursor management, forward-progress enforcement, and page assembly. Each adapter implements only sensor-specific concerns: API call construction, response deserialization, and field extraction. The `SensorAuth` trait is sealed so external crates cannot add unauthorized authentication mechanisms. Record types follow the `<sensor>_<entity>` naming convention (e.g., `crowdstrike_alert`, `armis_device`).

## Preconditions
- A sensor adapter implements the `SensorAdapter` trait and one or more `DataSource<T>` implementations
- Each `DataSource<T>` corresponds to a single sensor data feed (e.g., `crowdstrike_alert`, `claroty_device`)

## Postconditions
- The generic `DataSource<T>` trait provides `fetch_page()` and `cursor_from_record()` methods
- All pagination logic (cursor management, forward-progress enforcement, page assembly) is handled by shared infrastructure, not per-adapter code
- Each adapter only implements sensor-specific concerns: API call construction, response deserialization, field extraction
- `record_type` follows the `<sensor>_<entity>` naming convention (e.g., `crowdstrike_alert`, `armis_device`)

## Invariants
- Each `DataSource<T>` produces records of a single type
- The `SensorAuth` trait is sealed -- external crates cannot implement it (DI-012)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Compile error | External crate attempts to implement `SensorAuth` | Sealed trait prevents compilation |
| `PrismError::Sensor` | Adapter's `fetch_page()` encounters an unrecognized API response structure | Structured error with the sensor name, source, and raw response snippet for debugging |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-01-020 | A sensor API adds a new data source type not yet implemented | The new source is ignored; `list_capabilities` shows only implemented sources |
| EC-01-021 | Adapter bound to one (Client, Sensor) pair is accidentally shared | Type system prevents this: `SensorAdapter` requires `tenant_id()` returning the bound `TenantId` |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.013-001 | CrowdStrike adapter implementing `DataSource<CrowdStrikeAlert>` | `fetch_page()` delegates to adapter; shared infrastructure manages cursor; adapter code has no cursor logic |
| TV-BC-2.01.013-002 | External crate attempts `impl SensorAuth for ExternalAuth` | Compile-time error; sealed trait rejects the impl |
| TV-BC-2.01.013-003 | Adapter returns unrecognized API response structure | `PrismError::Sensor` with sensor name, source, raw response snippet |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-012 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
