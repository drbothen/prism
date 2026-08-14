---
document_type: behavioral-contract
level: L3
version: "1.7"
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
modified: "2026-08-13"
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
- **HTTP Error Classification (map_spec_engine_error_to_sensor_error):** When the sensor auth probe returns a non-zero HTTP status code, `map_spec_engine_error_to_sensor_error` in `crates/prism-bin/src/spec_driven_adapter.rs` MUST map `SpecEngineError::HttpRequestFailed { status_code > 0 }` to `SensorError::HttpError { sensor, status, body }`. This ensures the health classifier receives `SensorError::HttpError` and correctly resolves `auth_valid: false` (HTTP 4xx — sensor reachable, credentials rejected) or `ConnectivityStatus::Degraded` (HTTP 5xx — sensor reachable but erroring) rather than falling through to the `SensorError::Internal` catch-all arm, which incorrectly returns `ConnectivityStatus::Down` (implying unreachable) for sensors that ARE responding over HTTP. When `status_code = 0` (transport failure — no HTTP response received), the mapping continues to produce `SensorError::Internal`, which correctly resolves to `ConnectivityStatus::Down` (sensor unreachable). Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-001 + AC-ERR-002 + RG-001 (`test_map_error_http_401_maps_to_http_error_not_internal`) + RG-002 (`test_map_error_status_0_maps_to_internal`).
- **Persistent Auth Failure Classification (`AuthRefreshFailed` / `CookieAuthFailed`):** `map_spec_engine_error_to_sensor_error` MUST also map `SpecEngineError::AuthRefreshFailed` and `SpecEngineError::CookieAuthFailed` to `SensorError::HttpError { sensor, status: 401 }` (NOT `SensorError::Internal`). `AuthRefreshFailed` is produced by a double-401 after OAuth2 token re-acquisition for refreshable auth types (e.g., `Oauth2ClientCredentials`) — the pipeline retried once after the first 401 and received a second 401, aborting with `AuthRefreshFailed`. `CookieAuthFailed` is produced by HTTP 401 on a `CookieRoundtrip` sensor where no retry is attempted (static API key; calling `acquire_token()` again would re-read the same key — provably futile, BC-2.01.017 EC-017-002). Both conditions indicate the sensor IS REACHABLE (it sent an HTTP 401 response) but credentials are invalid. Mapping to `SensorError::Internal` suppresses the auth-validity signal and incorrectly reports `ConnectivityStatus::Down`; mapping to `SensorError::HttpError { status: 401 }` ensures the health classifier resolves `auth_valid: false` with `reachable: true`. Previously (pre-DEFECT-ADAPTER-TLS-XDOME-LIVE-001) these error variants fell through to the `SensorError::Internal` catch-all. Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-001 + AC-ERR-005 + RG-010 (`test_map_error_auth_refresh_failed_maps_to_http_error_401`) + RG-011 (`test_map_error_cookie_auth_failed_maps_to_http_error_401`).

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
| EC-08-006 | Sensor API is reachable but returns HTTP 4xx (e.g., 401 Unauthorized — invalid credentials) | `SensorError::HttpError` is produced by `map_spec_engine_error_to_sensor_error`; health classifier resolves `auth_valid: false`. Previously (pre-DEFECT-SENSOR-ERROR-FLATTEN-001), `SensorError::Internal` was returned for this case, causing `ConnectivityStatus::Down` to be reported — incorrectly implying the sensor was unreachable rather than that credentials were rejected. Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-005 (`test_probe_connectivity_403_returns_up_not_down`) + RG-007 (`test_sensor_health_wire_shape_403_reachable_auth_invalid`). |
| EC-08-007 | OAuth2 sensor double-401 — `AuthRefreshFailed` returned from pipeline; probe routes through `map_spec_engine_error_to_sensor_error` | `SensorError::HttpError { status: 401 }` produced; health classifier resolves `reachable: true, auth_valid: false`. Previously reported `ConnectivityStatus::Down` (incorrect — sensor responded with HTTP). Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-010 (`test_map_error_auth_refresh_failed_maps_to_http_error_401`). |
| EC-08-008 | `CookieRoundtrip` sensor returns HTTP 401 — `CookieAuthFailed` returned from pipeline; no retry attempted; probe routes through `map_spec_engine_error_to_sensor_error` | `SensorError::HttpError { status: 401 }` produced; health classifier resolves `reachable: true, auth_valid: false`. Previously reported `ConnectivityStatus::Down` (incorrect). Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-011 (`test_map_error_cookie_auth_failed_maps_to_http_error_401`). |
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
| 1.7 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-LOCAL-pass-F-P19-MED-001-fix | 2026-08-13 | product-owner | **F-P19-MED-001 closure: corrected §Postconditions HTTP Error Classification 5xx outcome mis-citation.** Replaced `ConnectivityStatus::Down (HTTP 5xx)` with `ConnectivityStatus::Degraded (HTTP 5xx)` in the HTTP Error Classification postcondition bullet; added explicit prose distinguishing the correct path (4xx → Up/auth_valid:false; 5xx → Degraded) from the buggy `SensorError::Internal` catch-all path (→ Down, implying unreachable) that the story eliminates. The prior v1.4/v1.5 prose was internally self-contradictory: both the CORRECT classification and the BUGGY catch-all were described as producing `ConnectivityStatus::Down`, erasing the semantic distinction. Ground truth: `probe_connectivity_inner` `Err(SensorError::HttpError { status, .. })` arm resolves `status >= 500` to `ConnectivityStatus::Degraded`, NOT Down; module-doc `connectivity.rs §module-doc` (2xx→Up, 429→Up, 5xx→Degraded, connection-error→Down) is the canonical design authority. TD-VSDD-097 9a sibling-pair: BC-2.01.013 (SS-01) — all `ConnectivityStatus::Down` references in BC-2.01.013 describe the buggy prior behavior being corrected, not a 5xx→Down assertion; no change needed. 9b downstream copy-target: story DEFECT-ADAPTER-TLS-XDOME-LIVE-001 does not contain `ConnectivityStatus::Down (HTTP 5xx)` as a correct-path assertion; §AC-ERR-005 correctly cites `ConnectivityStatus::Up` for 4xx; §EC-004 correctly cites `ConnectivityStatus::Down` for `status_code = 0` (transport failure — correct). No story edit needed. 9c mandate-anchor: no new MUST introduced; prose correction only; CLEAR. |
| 1.6 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-LOCAL-pass-3-F-1 | 2026-08-13 | product-owner | **pass-3 F-1 closure: corrected two phantom RG citations in EC-08-006 edge case.** (1) `RG-005 (test_probe_connectivity_401_returns_up_auth_invalid)` → `RG-005 (test_probe_connectivity_403_returns_up_not_down)`; (2) `RG-007 (test_sensor_health_wire_shape_auth_invalid)` → `RG-007 (test_sensor_health_wire_shape_403_reachable_auth_invalid)`. Canonical map per story v1.3 (code is authoritative per SAC-1). TD-VSDD-097 9a: BC-2.08.001 (SS-08 on-demand connectivity check) is the sibling-pair — no RG test citations in that BC; no change needed. 9b: EC-08-006 is not a verbatim copy-source in any downstream artifact; correction is self-contained. 9c: citation correction only — no new MUSTs introduced; no unanchored MUSTs. |
| 1.5 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-LOCAL-pass-1-spec-alignment | 2026-08-13 | product-owner | **Persistent auth failure error-mapping postcondition added (`AuthRefreshFailed` / `CookieAuthFailed` → `HttpError{401}`).** Extended `HTTP Error Classification` postcondition block: `map_spec_engine_error_to_sensor_error` MUST also map `SpecEngineError::AuthRefreshFailed` and `SpecEngineError::CookieAuthFailed` to `SensorError::HttpError { sensor, status: 401 }` (NOT `SensorError::Internal`). Both indicate the sensor is REACHABLE but credentials are invalid; prior `Internal` mapping incorrectly reported `ConnectivityStatus::Down` and suppressed `auth_valid:false`. Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-001 + AC-ERR-005 + RG-010 (`test_map_error_auth_refresh_failed_maps_to_http_error_401`) + RG-011 (`test_map_error_cookie_auth_failed_maps_to_http_error_401`). TD-VSDD-097 9a sibling-pair: BC-2.08.001 (SS-08 on-demand connectivity check, v1.5) — that BC handles `SensorError::HttpError` correctly on the calling side (RC-F9-B confirmed); no change needed in BC-2.08.001. 9b downstream copy-target: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 story references this postcondition by citation (no verbatim copy block); story-writer propagates under `bc_array_changes_propagate_to_body_and_acs`. 9c mandate-anchor: new MUST anchored to DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-001 + AC-ERR-005 + RG-010 + RG-011 — no unanchored MUSTs. |
| 1.4 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-spec-amendment | 2026-08-12 | product-owner | DEFECT-SENSOR-ERROR-FLATTEN-001 (bundled into DEFECT-ADAPTER-TLS-XDOME-LIVE-001): `map_spec_engine_error_to_sensor_error` error-classification fix spec. **New postcondition added:** `map_spec_engine_error_to_sensor_error` in `crates/prism-bin/src/spec_driven_adapter.rs` MUST map `SpecEngineError::HttpRequestFailed { status_code > 0 }` to `SensorError::HttpError { sensor, status, body }` (NOT `SensorError::Internal`); `status_code = 0` (transport failure — no HTTP response) continues to map to `SensorError::Internal`. This fixes the root cause (RC-F9-A) where HTTP 401 responses were mapped to `SensorError::Internal` → `ConnectivityStatus::Down` instead of `SensorError::HttpError` → `auth_valid: false`. **New edge case EC-08-006 added:** HTTP 4xx from sensor API (reachable but invalid credentials) now correctly classified as `auth_valid: false`. Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-001 + AC-ERR-002 + RG-001 (`test_map_error_http_401_maps_to_http_error_not_internal`) + RG-002 (`test_map_error_status_0_maps_to_internal`). **TD-VSDD-097 three-dimension sweep:** 9a sibling-pair: BC-2.08.001 (on-demand connectivity check, SS-08) — the health classifier in BC-2.08.001 already handles `SensorError::HttpError` correctly (RC-F9-B confirmed in design doc); no change needed. 9b downstream copy-target: no story currently reproduces BC-2.08.002 §Postconditions as a verbatim copy-source block; DEFECT-ADAPTER-TLS-XDOME-LIVE-001 story will include this postcondition by new-content reference. 9c mandate-anchor: new MUST anchored to DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-001 + AC-ERR-002 + RG-001 + RG-002 — no unanchored MUSTs. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
