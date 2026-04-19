---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-03"
capability: "CAP-004"
lifecycle_status: active
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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |
