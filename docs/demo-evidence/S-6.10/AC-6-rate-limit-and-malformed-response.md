# AC-6: Rate limit returns HTTP 429; EC-006 malformed response mode

## Acceptance Criterion

Given `FailureMode::RateLimit` configured, When threshold exceeded, Then
HTTP 429 is returned — maps to `E-SENSOR-003`.

Also covers EC-006: `FailureLayer::MalformedResponse` returns invalid JSON, exercising
Prism's parse-error path.

## Tests

| Test function | File | Coverage |
|--------------|------|---------|
| `ac_6_rate_limit_429_after_threshold_exceeded_via_configure` | `tests/ac_6_rate_limit_429.rs` | Configure after_n=0 → next request returns 429 |
| `ac_6_rate_limit_allows_requests_before_threshold` | `tests/ac_6_rate_limit_429.rs` | Configure after_n=3 → 3 requests succeed, 4th returns 429 |
| `ec_006_malformed_response_mode_returns_non_parseable_body` | `tests/ac_6_rate_limit_429.rs` | EC-006: malformed_response mode → body fails JSON parsing |

## Test command

```
cargo test --features prism-dtu-armis/dtu --test ac_6_rate_limit_429
```

## Test output

```
running 3 tests
test ac_6_rate_limit_429_after_threshold_exceeded_via_configure ... ok
test ec_006_malformed_response_mode_returns_non_parseable_body ... ok
test ac_6_rate_limit_allows_requests_before_threshold ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Implementation

- `FailureLayer` from `prism-dtu-common` mounted in Armis stub router (`src/clone.rs`)
- Configured at runtime via `POST /dtu/configure` with `failure_mode` field
- `rate_limit` mode: counts requests, returns 429 once budget exhausted
- `malformed_response` mode: returns non-JSON body bytes

## Sequence (success path — rate limit)

```
Client → POST /dtu/configure
         Body: {"failure_mode": "rate_limit", "after_n_requests": 0, "retry_after_secs": 30}
DTU    → HTTP 200 {"status": "ok"}

Client → GET /api/v1/devices
         Authorization: Bearer test-token
DTU    → HTTP 429  (FailureLayer intercepts — budget=0)
```

## Sequence (success path — rate limit with budget)

```
Client → POST /dtu/configure
         Body: {"failure_mode": "rate_limit", "after_n_requests": 3, "retry_after_secs": 30}
DTU    → HTTP 200

Client → GET /api/v1/devices (request 1)  → HTTP 200
Client → GET /api/v1/devices (request 2)  → HTTP 200
Client → GET /api/v1/devices (request 3)  → HTTP 200
Client → GET /api/v1/devices (request 4)  → HTTP 429  (budget exhausted)
```

## Sequence (EC-006 — malformed response)

```
Client → POST /dtu/configure
         Body: {"failure_mode": "malformed_response"}
DTU    → HTTP 200

Client → GET /api/v1/devices
         Authorization: Bearer test-token
DTU    → HTTP 200, body = <invalid JSON bytes>

Prism  → serde_json::from_slice() → Err(...)  (parse-error path triggered)
```
