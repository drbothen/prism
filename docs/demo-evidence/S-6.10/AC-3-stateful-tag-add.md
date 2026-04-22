# AC-3: Stateful tag add — POST tag persists in subsequent device query

## Acceptance Criterion

Given `POST /api/v1/devices/{device_id}/tags/` with `{"tag_key": "ot-critical"}`,
Then the response is HTTP 201 AND subsequent `GET /api/v1/devices` returns that device
with `"ot-critical"` in its `tags` array (stateful tagging).

## Tests

| Test function | File | Coverage |
|--------------|------|---------|
| `ac_3_post_tag_returns_201_with_device_id_and_tag_key` | `tests/ac_3_stateful_tag_add.rs` | POST tag → HTTP 201, body has device_id, tag_key, status="added" |
| `ac_3_added_tag_appears_in_subsequent_device_query` | `tests/ac_3_stateful_tag_add.rs` | Tag persists in tag_store; merged into device record on next GET |
| `ac_3_tag_endpoint_requires_bearer_auth_returns_403` | `tests/ac_3_stateful_tag_add.rs` | POST without auth → HTTP 403 (not 401) |

## Test command

```
cargo test --features prism-dtu-armis/dtu --test ac_3_stateful_tag_add
```

## Test output

```
running 3 tests
test ac_3_post_tag_returns_201_with_device_id_and_tag_key ... ok
test ac_3_tag_endpoint_requires_bearer_auth_returns_403 ... ok
test ac_3_added_tag_appears_in_subsequent_device_query ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Implementation

- Route: `POST /api/v1/devices/{device_id}/tags/` — file `crates/prism-dtu-armis/src/routes/tags.rs`
- State: `ArmisState::tag_store` — `Mutex<HashMap<String, HashSet<String>>>` in `src/state.rs`
- Merge: device query handler reads tag_store and merges into device records before returning

## Sequence (success path)

```
Client → POST /api/v1/devices/d-001/tags/
         Authorization: Bearer test-token
         Body: {"tag_key": "ot-critical"}

DTU    → inserts "ot-critical" into tag_store["d-001"]
       → HTTP 201 {"device_id": "d-001", "tag_key": "ot-critical", "status": "added"}

Client → GET /api/v1/devices?size=100
         Authorization: Bearer test-token

DTU    → merges tag_store into device records
       → d-001.tags contains "ot-critical"
       → HTTP 200 {"data": {"devices": [{..."tags": ["ot-critical"]...}, ...]}}
```

## Sequence (error path — missing auth)

```
Client → POST /api/v1/devices/d-001/tags/
         (no Authorization header)

DTU    → HTTP 403 {"error": "invalid or missing bearer token", "code": 403}
```
