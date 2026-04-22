# AC-7: Reset clears tag store and AQL log; fixture data survives

## Acceptance Criterion

Given `reset()` is called, Then the tag store is cleared, the AQL log is
cleared, and subsequent device queries return devices with empty `tags` arrays.

## Tests

| Test function | File | Coverage |
|--------------|------|---------|
| `ac_story_7_reset_clears_tag_store_and_aql_log` | `tests/reset_state_invariants.rs` | POST tag + AQL query → reset → tag absent + AQL log empty |
| `ac_story_7_reset_does_not_remove_fixture_data` | `tests/reset_state_invariants.rs` | After reset: 25 fixture devices still present |
| `activity_endpoint_returns_200_with_activities_array` | `tests/reset_state_invariants.rs` | Shape: data.activities array + data.total |
| `alerts_endpoint_returns_200_with_alerts_array` | `tests/reset_state_invariants.rs` | Shape: data.alerts array + data.total >= 12 |
| `alerts_pagination_beyond_last_returns_empty_array` | `tests/reset_state_invariants.rs` | Alerts pagination: page=999 → empty array, total > 0 |

## Test command

```
cargo test --features prism-dtu-armis/dtu --test reset_state_invariants
```

## Test output

```
running 5 tests
test activity_endpoint_returns_200_with_activities_array ... ok
test alerts_pagination_beyond_last_returns_empty_array ... ok
test alerts_endpoint_returns_200_with_alerts_array ... ok
test ac_story_7_reset_does_not_remove_fixture_data ... ok
test ac_story_7_reset_clears_tag_store_and_aql_log ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Implementation

- `ArmisState::reset()` in `crates/prism-dtu-armis/src/state.rs`:
  - Clears `tag_store` (HashMap is cleared, not replaced)
  - Clears `aql_log` (Vec is truncated to 0)
  - Fixture data is read-only — not affected by reset
- `POST /dtu/reset` calls `state.reset()` and returns `{"status": "ok"}`

## Sequence (success path — reset clears mutable state)

```
Client → POST /api/v1/devices/d-001/tags/
         Authorization: Bearer test-token
         Body: {"tag_key": "pre-reset-tag"}
DTU    → HTTP 201

Client → GET /api/v1/devices?aql=in:type=switch
         Authorization: Bearer test-token
DTU    → HTTP 200, aql_log = ["in:type=switch"]

Client → POST /dtu/reset
DTU    → HTTP 200 {"status": "ok"}
       → tag_store = {}, aql_log = []

Client → GET /dtu/aql-log
DTU    → {"aql_strings": []}  (empty)

Client → GET /api/v1/devices?size=100
         Authorization: Bearer test-token
DTU    → d-001.tags = []  (pre-reset-tag absent)
       → data.total = 25  (fixture data intact)
```
