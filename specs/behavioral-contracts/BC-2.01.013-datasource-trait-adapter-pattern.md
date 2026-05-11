---
document_type: behavioral-contract
level: L3
version: "1.4"
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
input-hash: "76729b7"
traces_to: ["CAP-001"]
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

# BC-2.01.013: DataSource Trait Eliminates Per-Sensor Code Duplication

## Description

The `DataSource<T>` generic trait provides a uniform interface for all sensor data feeds, with
shared infrastructure handling cursor management, forward-progress enforcement, and page assembly.
Each adapter implements only sensor-specific concerns: API call construction, response
deserialization, and field extraction. Adapter implementations are produced from TOML SensorSpec
declarations at runtime; runtime validation (not compile-time sealing) prevents cross-sensor auth
composition per the three rules in ADR-023 Rule 2. Record types follow the `<sensor>_<entity>`
naming convention (e.g., `crowdstrike_alert`, `armis_device`).

> **PENDING AMENDMENT — ADR-023**: The sealed-trait enforcement of `SensorAuth` described in
> earlier versions of this BC is superseded by spec-driven runtime validation. The `SensorAuth`
> trait is no longer sealed. Cross-sensor auth-composition prevention is enforced at spec-load
> time via three runtime rejection rules (see Rule 2 of ADR-023 and the amended DI-012).

## Preconditions
- A sensor adapter implements the `SensorAdapter` trait and one or more `DataSource<T>` implementations
- Each `DataSource<T>` corresponds to a single sensor data feed (e.g., `crowdstrike_alert`, `claroty_device`)
- A valid TOML SensorSpec declaration exists for the sensor, specifying a single `auth_type` value

## Postconditions
- The generic `DataSource<T>` trait provides `fetch_page()` and `cursor_from_record()` methods
- All pagination logic (cursor management, forward-progress enforcement, page assembly) is handled by shared infrastructure, not per-adapter code
- Each adapter only implements sensor-specific concerns: API call construction, response deserialization, field extraction
- Adapter implementations are produced from TOML SensorSpec declarations at runtime; no hand-written adapter code outside `prism-sensors` is required for TOML-expressible sensors
- `record_type` follows the `<sensor>_<entity>` naming convention (e.g., `crowdstrike_alert`, `armis_device`)
- Cross-sensor auth-composition is prevented by three runtime validation rules enforced at spec-load time (ADR-023 Rule 2):
  1. `SensorSpec.auth_type` accepts exactly one value from the enumerated set; arrays or mixed types are rejected at spec-load
  2. Each auth method declares exactly one `credential_ref` binding; multiple credential bindings per auth method are rejected at spec-load
  3. The credential record schema must structurally match the declared `auth_type`; mismatches are rejected at spec-load

## Invariants
- Each `DataSource<T>` produces records of a single type
- The `SensorAuth` trait is NOT sealed — it is open for plugin implementations (ADR-023 Rule 2). Cross-sensor auth-composition is prevented by three runtime rejection rules (see Postconditions), not by compile-time sealed-supertrait enforcement (DI-012 amended)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-010` | SensorSpec declares multiple auth types | Rejected at spec-load with error citing Rule 1 |
| `E-SPEC-011` | Auth method has multiple credential_ref bindings | Rejected at spec-load with error citing Rule 2 |
| `E-SPEC-012` | Credential schema does not structurally match declared auth_type | Rejected at credential-resolution time with error citing Rule 3 |
| `PrismError::Sensor` | Adapter's `fetch_page()` encounters an unrecognized API response structure | Structured error with the sensor name, source, and raw response snippet for debugging |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-01-020 | A sensor API adds a new data source type not yet implemented | The new source is ignored; `list_capabilities` shows only implemented sources |
| EC-01-021 | Adapter bound to one (Client, Sensor) pair is accidentally shared | Type system prevents this: `SensorAdapter` requires `tenant_id()` returning the bound `TenantId` |
| EC-01-022 | SensorSpec declares `auth_type: [oauth2_client_credentials, bearer_static]` (array) | Rejected at spec-load with E-SPEC-010 citing Rule 1 (single auth_type required) |
| EC-01-023 | SensorSpec auth method has two credential_ref entries | Rejected at spec-load with E-SPEC-011 citing Rule 2 (single credential_ref per method) |
| EC-01-024 | SensorSpec declares `auth_type: oauth2_client_credentials` but credential is a cookie record | Rejected at credential-resolution time with E-SPEC-012 citing Rule 3 (structural mismatch) |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.013-001 | CrowdStrike adapter implementing `DataSource<CrowdStrikeAlert>` | `fetch_page()` delegates to adapter; shared infrastructure manages cursor; adapter code has no cursor logic |
| TV-BC-2.01.013-002 | SensorSpec with `auth_type: [oauth2_client_credentials, bearer_static]` | Spec-load rejected with E-SPEC-010; Rule 1 cited in error |
| TV-BC-2.01.013-003 | Adapter returns unrecognized API response structure | `PrismError::Sensor` with sensor name, source, raw response snippet |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-PLUGIN-006 | OCSF column mapping fixture catalog verifying SpecDrivenMapper correctness |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-012 (amended — runtime enforcement replaces compile-time sealed trait per ADR-023 Rule 2) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | prereq-f | 2026-05-11 | product-owner | ADR-023 v1.17 PREREQ-F amendment: removed sealed-trait language; replaced with spec-driven adapter pattern where implementations are produced from TOML SensorSpec declarations at runtime; replaced compile-time SensorAuth sealing with three runtime cross-sensor auth-composition rejection rules per ADR-023 Rule 2; updated Error Cases, Edge Cases, Canonical Test Vectors, and Verification Properties accordingly. DI-012 reference updated to reflect amended runtime enforcement. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
