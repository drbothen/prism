# AC-1: AQL capture — GET and POST device query logs AQL verbatim

## Acceptance Criterion

Given `GET /api/v1/devices?aql=in:type%3Dswitch` with a valid Bearer token,
Then the response is HTTP 200 with a `data.devices` array AND the received AQL string
`"in:type=switch"` is logged in `GET /dtu/aql-log` (AQL capture works).

## Tests

| Test function | File | Coverage |
|--------------|------|---------|
| `ac_1_get_devices_with_aql_returns_200_and_logs_aql` | `tests/ac_1_aql_capture_and_device_list.rs` | GET with AQL query param → 200 + AQL logged |
| `ac_1_post_devices_with_aql_body_returns_200_and_logs_aql` | `tests/ac_1_aql_capture_and_device_list.rs` | POST with AQL JSON body → 200 + AQL logged (EC-005) |
| `ac_1_devices_response_contains_pagination_fields` | `tests/ac_1_aql_capture_and_device_list.rs` | Response shape: data.devices, data.total, data.page |
| `ec_001_aql_special_characters_stored_verbatim` | `tests/ac_1_aql_capture_and_device_list.rs` | EC-001: `<`, `>`, `=` stored without parsing |
| `ec_004_pagination_beyond_last_page_returns_empty_array` | `tests/ac_1_aql_capture_and_device_list.rs` | EC-004: page=100 → empty devices, correct total |

## Test command

```
cargo test --features prism-dtu-armis/dtu --test ac_1_aql_capture_and_device_list
```

## Test output

```
running 5 tests
test ec_004_pagination_beyond_last_page_returns_empty_array ... ok
test ac_1_devices_response_contains_pagination_fields ... ok
test ac_1_post_devices_with_aql_body_returns_200_and_logs_aql ... ok
test ac_1_get_devices_with_aql_returns_200_and_logs_aql ... ok
test ec_001_aql_special_characters_stored_verbatim ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Implementation

- Route: `GET /api/v1/devices` and `POST /api/v1/devices` — file `crates/prism-dtu-armis/src/routes/devices.rs`
- AQL capture: `crates/prism-dtu-armis/src/state.rs` — `ArmisState::capture_aql()`
- AQL retrieval: `GET /dtu/aql-log` — file `crates/prism-dtu-armis/src/routes/dtu.rs`

## Sequence (success path)

```
Client → GET /api/v1/devices?aql=in:type=switch
         Authorization: Bearer test-token

DTU    → captures "in:type=switch" in aql_log
       → returns HTTP 200 {"data": {"devices": [...], "total": 25, "page": 0}}

Client → GET /dtu/aql-log
DTU    → returns HTTP 200 {"aql_strings": ["in:type=switch"]}
```

## Sequence (error path — EC-001 special chars)

```
Client → GET /api/v1/devices?aql=risk_score>80 AND type=switch AND name<Z
DTU    → stores verbatim: "risk_score>80 AND type=switch AND name<Z"
       → returns HTTP 200 (AQL not parsed or rejected)
Client → GET /dtu/aql-log
DTU    → {"aql_strings": ["risk_score>80 AND type=switch AND name<Z"]}
```
