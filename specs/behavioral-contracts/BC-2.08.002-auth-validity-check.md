---
document_type: behavioral-contract
level: L3
version: "1.4"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "4a1f396"
traces_to: ["CAP-008"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-08"
capability: "CAP-008"
lifecycle_status: active
introduced: cycle-1
modified: "2026-08-12"
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
- **HTTP Error Classification (map_spec_engine_error_to_sensor_error):** When the sensor auth probe returns a non-zero HTTP status code, `map_spec_engine_error_to_sensor_error` in `crates/prism-bin/src/spec_driven_adapter.rs` MUST map `SpecEngineError::HttpRequestFailed { status_code > 0 }` to `SensorError::HttpError { sensor, status, body }`. This ensures the health classifier receives `SensorError::HttpError` and correctly resolves `auth_valid: false` (HTTP 4xx) or `ConnectivityStatus::Down` (HTTP 5xx) rather than falling through to the `SensorError::Internal` catch-all arm which incorrectly returns `ConnectivityStatus::Down` for reachable-but-auth-failed sensors. When `status_code = 0` (transport failure — no HTTP response received), the mapping continues to produce `SensorError::Internal`. Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-001 + AC-ERR-002 + RG-001 (`test_map_error_http_401_maps_to_http_error_not_internal`) + RG-002 (`test_map_error_status_0_maps_to_internal`).

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
| EC-08-006 | Sensor API is reachable but returns HTTP 4xx (e.g., 401 Unauthorized — invalid credentials) | `SensorError::HttpError` is produced by `map_spec_engine_error_to_sensor_error`; health classifier resolves `auth_valid: false`. Previously (pre-DEFECT-SENSOR-ERROR-FLATTEN-001), `SensorError::Internal` was returned for this case, causing `ConnectivityStatus::Down` to be reported — incorrectly implying the sensor was unreachable rather than that credentials were rejected. Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-005 (`test_probe_connectivity_401_returns_up_auth_invalid`) + RG-007 (`test_sensor_health_wire_shape_auth_invalid`). |
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
| 1.4 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-spec-amendment | 2026-08-12 | product-owner | DEFECT-SENSOR-ERROR-FLATTEN-001 (bundled into DEFECT-ADAPTER-TLS-XDOME-LIVE-001): `map_spec_engine_error_to_sensor_error` error-classification fix spec. **New postcondition added:** `map_spec_engine_error_to_sensor_error` in `crates/prism-bin/src/spec_driven_adapter.rs` MUST map `SpecEngineError::HttpRequestFailed { status_code > 0 }` to `SensorError::HttpError { sensor, status, body }` (NOT `SensorError::Internal`); `status_code = 0` (transport failure — no HTTP response) continues to map to `SensorError::Internal`. This fixes the root cause (RC-F9-A) where HTTP 401 responses were mapped to `SensorError::Internal` → `ConnectivityStatus::Down` instead of `SensorError::HttpError` → `auth_valid: false`. **New edge case EC-08-006 added:** HTTP 4xx from sensor API (reachable but invalid credentials) now correctly classified as `auth_valid: false`. Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-001 + AC-ERR-002 + RG-001 (`test_map_error_http_401_maps_to_http_error_not_internal`) + RG-002 (`test_map_error_status_0_maps_to_internal`). **TD-VSDD-097 three-dimension sweep:** 9a sibling-pair: BC-2.08.001 (on-demand connectivity check, SS-08) — the health classifier in BC-2.08.001 already handles `SensorError::HttpError` correctly (RC-F9-B confirmed in design doc); no change needed. 9b downstream copy-target: no story currently reproduces BC-2.08.002 §Postconditions as a verbatim copy-source block; DEFECT-ADAPTER-TLS-XDOME-LIVE-001 story will include this postcondition by new-content reference. 9c mandate-anchor: new MUST anchored to DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-001 + AC-ERR-002 + RG-001 + RG-002 — no unanchored MUSTs. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
