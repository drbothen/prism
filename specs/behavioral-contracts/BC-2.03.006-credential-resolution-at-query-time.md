---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-03"
capability: "CAP-004"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "8e43eb2"
traces_to: ["CAP-004"]
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

# BC-2.03.006: Credential Resolution at Sensor Query Time

## Description

When a sensor query is initiated, the credential for the `(client_id, sensor_id, credential_name)` tuple is resolved from the active backend and passed to the `SensorAuth` implementation as a `SecretString`. Resolution is audit-logged (namespace only, never the value). If resolution fails for any reason, the query fails with a clear error before any API call is attempted — there is no partial-auth state.

## Preconditions
- A sensor query is initiated for a `(client_id, sensor_id)` pair
- The sensor configuration includes a `credential_ref` pointing to the credential store

## Postconditions
- The credential is resolved from the active backend using the `(client_id, sensor_id, credential_name)` namespace
- The resolved credential is passed to the `SensorAuth` implementation (OAuth2, Cookie, Bearer) as a `SecretString`
- Credential resolution is audit-logged (tenant, sensor, credential name -- never the value)
- If resolution fails, the sensor query fails with a clear error before any API call is attempted

## Invariants
- DI-002: Credential isolation
- Credential values remain in `SecretString` wrappers until consumed by the auth middleware

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Credential not found for the configured `credential_ref` | `category: "configuration"`, suggestion: "Run credential setup for client '{client}' sensor '{sensor}'. Expected credential: '{name}'" |
| `PrismError::Credential` | Backend unavailable at query time | `category: "configuration"`, suggestion varies by backend (keyring locked vs file permission) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-014 | CrowdStrike requires two credentials (client_id + client_secret) | Both resolved independently from the credential store; both must succeed or the query fails |
| EC-03-015 | Credential was rotated in the store mid-session | Next query picks up the new credential; previously cached auth tokens (e.g., OAuth2) are invalidated on next 401 |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.006-001 | Query initiated; credential exists | Credential resolved as SecretString; audit log entry (no value); sensor API call proceeds |
| TV-BC-2.03.006-002 | Query initiated; credential missing from store | `PrismError::Credential` with setup suggestion; no API call made |
| TV-BC-2.03.006-003 | CrowdStrike: both credentials present | Both resolved; query proceeds |
| TV-BC-2.03.006-004 | CrowdStrike: client_secret missing | `PrismError::Credential`; query fails before first API call |
| TV-BC-2.03.006-005 | Credential rotated mid-session | New value used on next query; cached OAuth2 token invalidated on next 401 |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-011 | Credential name sanitization: rejects path traversal (kani) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
