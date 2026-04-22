# AC-2: Timestamp fallback — device d-001 has null last_seen, non-null first_seen

## Acceptance Criterion

Given `fixtures/devices.json` contains device `"d-001"` with `last_seen: null`
and `first_seen: "2024-01-15T10:00:00Z"`, When the device detail is returned by the DTU,
Then the response JSON contains `last_seen: null` and a non-null `first_seen` —
exercising Prism's timestamp fallback path in the TOML spec.

## Tests

| Test function | File | Coverage |
|--------------|------|---------|
| `ac_2_device_d001_has_null_last_seen_and_non_null_first_seen` | `tests/ac_2_timestamp_fallback_fixture.rs` | d-001: last_seen=null, first_seen="2024-01-15T10:00:00Z" |
| `ac_2_device_d002_has_both_timestamps_populated` | `tests/ac_2_timestamp_fallback_fixture.rs` | Contrast: d-002 has both timestamps set |
| `ac_2_device_risk_endpoint_returns_risk_score` | `tests/ac_2_timestamp_fallback_fixture.rs` | Risk endpoint: device_id, risk_score (number), risk_factors (array) |
| `ec_002_risk_endpoint_returns_404_for_unknown_device` | `tests/ac_2_timestamp_fallback_fixture.rs` | EC-002: unknown device_id → HTTP 404 with error field |

## Test command

```
cargo test --features prism-dtu-armis/dtu --test ac_2_timestamp_fallback_fixture
```

## Test output

```
running 4 tests
test ec_002_risk_endpoint_returns_404_for_unknown_device ... ok
test ac_2_device_risk_endpoint_returns_risk_score ... ok
test ac_2_device_d002_has_both_timestamps_populated ... ok
test ac_2_device_d001_has_null_last_seen_and_non_null_first_seen ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Fixture requirement

File: `crates/prism-dtu-armis/fixtures/devices.json`

```json
{
  "device_id": "d-001",
  "last_seen": null,
  "first_seen": "2024-01-15T10:00:00Z",
  ...
}
```

Device `"d-002"` has both timestamps set — confirming d-001 is the intentional fallback case,
not a data error.

## Sequence (success path — timestamp fallback)

```
Client → GET /api/v1/devices?size=100
         Authorization: Bearer test-token

DTU    → returns d-001 with last_seen=null, first_seen="2024-01-15T10:00:00Z"
       → Prism sensor TOML spec detects last_seen=null and falls back to first_seen
```

## Sequence (error path — EC-002: unknown device)

```
Client → GET /api/v1/devices/d-NONEXISTENT-9999/risk
         Authorization: Bearer test-token

DTU    → HTTP 404 {"error": "device not found"}
```
