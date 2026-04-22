# AC-5: Missing bearer token returns HTTP 403 (not 401)

## Acceptance Criterion

Given a request without `Authorization` header, Then the response is HTTP 403
`{"error": "invalid or missing bearer token", "code": 403}` — note: Armis uses 403
not 401, per API spec.

## Tests

| Test function | File | Coverage |
|--------------|------|---------|
| `ac_5_get_devices_without_auth_returns_403` | `tests/ac_5_missing_bearer_403.rs` | GET /api/v1/devices no auth → 403, body has error + code=403 |
| `ac_5_get_alerts_without_auth_returns_403` | `tests/ac_5_missing_bearer_403.rs` | GET /api/v1/alerts no auth → 403 |
| `ac_5_get_device_activity_without_auth_returns_403` | `tests/ac_5_missing_bearer_403.rs` | GET /api/v1/devices/d-001/activity no auth → 403 |
| `ac_5_get_device_risk_without_auth_returns_403` | `tests/ac_5_missing_bearer_403.rs` | GET /api/v1/devices/d-001/risk no auth → 403 |
| `ac_5_empty_bearer_value_returns_403` | `tests/ac_5_missing_bearer_403.rs` | `Authorization: Bearer ` (empty token) → 403 |
| `ac_5_wrong_scheme_returns_403` | `tests/ac_5_missing_bearer_403.rs` | `Authorization: Basic ...` → 403 (wrong scheme) |
| `ac_5_dtu_internal_endpoints_do_not_require_auth` | `tests/ac_5_missing_bearer_403.rs` | /dtu/health, /dtu/aql-log, /dtu/reset → 200 without auth |

## Test command

```
cargo test --features prism-dtu-armis/dtu --test ac_5_missing_bearer_403
```

## Test output

```
running 7 tests
test ac_5_get_devices_without_auth_returns_403 ... ok
test ac_5_get_device_risk_without_auth_returns_403 ... ok
test ac_5_wrong_scheme_returns_403 ... ok
test ac_5_get_alerts_without_auth_returns_403 ... ok
test ac_5_empty_bearer_value_returns_403 ... ok
test ac_5_get_device_activity_without_auth_returns_403 ... ok
test ac_5_dtu_internal_endpoints_do_not_require_auth ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Architecture note

Armis Centrix returns HTTP 403 (Forbidden) for missing or invalid auth — not 401
(Unauthorized) as per Bearer-standard behavior. This is per the Armis Centrix API
specification and is intentionally different. Prism's error-code mapping must not
normalize this to 401. The 403 behavior is enforced by the DTU bearer auth middleware.

## Sequence (error path — missing auth header)

```
Client → GET /api/v1/devices
         (no Authorization header)

DTU    → HTTP 403
         {"error": "invalid or missing bearer token", "code": 403}
```

## Sequence (error path — empty Bearer token)

```
Client → GET /api/v1/devices
         Authorization: Bearer 

DTU    → HTTP 403
         {"error": "invalid or missing bearer token", "code": 403}
```

## Sequence (success path — DTU internal endpoint bypasses auth)

```
Client → GET /dtu/aql-log
         (no Authorization header)

DTU    → HTTP 200 {"aql_strings": [...]}
```
