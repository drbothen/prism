---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Health"
capability: "CAP-008"
---

# BC-2.08.002: Auth Validity Check Per Sensor Per Client

## Preconditions
- A valid `client_id` and `sensor_id` are provided
- The sensor is configured and enabled for the specified client
- Credentials exist in the credential store for `(client_id, sensor_id)`

## Postconditions
- The health response includes `auth_valid: true` or `auth_valid: false`
- For CrowdStrike: OAuth2 token request is attempted; success means auth is valid
- For Cyberint: cookie-based auth flow is attempted; valid session means auth is valid
- For Claroty/Armis: bearer token is used in a lightweight API call; HTTP 200 means auth is valid
- Auth failure details are included in the health response (expired, invalid, revoked) but never credential values

## Invariants
- DI-002: Credential isolation per client -- only the specified client's credentials are accessed
- DI-008: Client data separation -- auth check uses only the specified client's sensor config

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Credentials missing from store | Health status reports `auth_valid: false`, `reason: "credentials_not_found"` |
| `PrismError::Credential` | OS keyring locked | Health status reports `auth_valid: false`, `reason: "keyring_unavailable"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-004 | CrowdStrike OAuth2 token is expired but refresh succeeds | Auth reported as `auth_valid: true`; the refresh is transparent |
| EC-08-005 | Sensor API is unreachable (auth cannot be verified) | `auth_valid: null` with `reason: "sensor_unreachable_cannot_verify"` |
| DEC-011 | OS keyring locked on macOS | `auth_valid: false`, `reason: "keyring_locked"`, `suggestion: "Unlock keychain or configure encrypted file fallback"` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-002, DI-008 |
| Priority | P1 |
