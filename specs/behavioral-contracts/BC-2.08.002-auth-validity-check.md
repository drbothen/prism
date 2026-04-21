---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
traces_to: ["CAP-008"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-08"
capability: "CAP-008"
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

# BC-2.08.002: Auth Validity Check Per Sensor Per Client

## Description

The health check validates authentication for a specific `(client_id, sensor_id)` pair by attempting the sensor-specific auth flow: OAuth2 token request for CrowdStrike, cookie-based auth for Cyberint, and a lightweight bearer-token API call for Claroty/Armis. Auth failure details (expired, invalid, revoked) are included in the health response, but credential values are never exposed. Per DI-002, only the specified client's credentials are accessed.

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

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| CrowdStrike with valid OAuth2 credentials | `auth_valid: true` | happy-path |
| Missing credentials in credential store | `auth_valid: false`, `reason: "credentials_not_found"` | error |
| Sensor unreachable (cannot verify auth) | `auth_valid: null`, `reason: "sensor_unreachable_cannot_verify"` | edge-case |
| macOS keyring locked | `auth_valid: false`, `reason: "keyring_locked"`, includes suggestion | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | Auth check never exposes credential values in response | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-002, DI-008 |
| Priority | P1 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col Version | Burst | Date | Author | Change form. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
