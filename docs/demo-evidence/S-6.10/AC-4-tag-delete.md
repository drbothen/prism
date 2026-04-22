# AC-4: Tag delete — DELETE removes tag; absent from subsequent query

## Acceptance Criterion

Given `DELETE /api/v1/devices/{device_id}/tags/ot-critical` after the tag is
added, Then the response is HTTP 200 `{"status": "removed"}` AND subsequent device
query returns the device without that tag.

## Tests

| Test function | File | Coverage |
|--------------|------|---------|
| `ac_4_delete_tag_returns_200_removed` | `tests/ac_4_tag_delete.rs` | DELETE after add → HTTP 200, body status="removed" |
| `ac_4_device_does_not_have_tag_after_delete` | `tests/ac_4_tag_delete.rs` | Tag absent from device record in subsequent GET |
| `ac_4_delete_tag_endpoint_requires_bearer_auth` | `tests/ac_4_tag_delete.rs` | DELETE without auth → HTTP 403 |
| `ec_003_delete_nonexistent_tag_returns_404` | `tests/ac_4_tag_delete.rs` | EC-003: DELETE tag never added → HTTP 404 with error |

## Test command

```
cargo test --features prism-dtu-armis/dtu --test ac_4_tag_delete
```

## Test output

```
running 4 tests
test ac_4_delete_tag_endpoint_requires_bearer_auth ... ok
test ec_003_delete_nonexistent_tag_returns_404 ... ok
test ac_4_delete_tag_returns_200_removed ... ok
test ac_4_device_does_not_have_tag_after_delete ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Implementation

- Route: `DELETE /api/v1/devices/{device_id}/tags/{tag_key}` — file `crates/prism-dtu-armis/src/routes/tags.rs`
- State: `ArmisState::remove_tag()` removes from `tag_store`; returns error if key absent

## Sequence (success path)

```
Client → POST /api/v1/devices/d-001/tags/
         Authorization: Bearer test-token
         Body: {"tag_key": "ot-critical"}
DTU    → HTTP 201 {"device_id": "d-001", "tag_key": "ot-critical", "status": "added"}

Client → DELETE /api/v1/devices/d-001/tags/ot-critical
         Authorization: Bearer test-token
DTU    → removes "ot-critical" from tag_store["d-001"]
       → HTTP 200 {"status": "removed"}

Client → GET /api/v1/devices?size=100
         Authorization: Bearer test-token
DTU    → d-001.tags does NOT contain "ot-critical"
```

## Sequence (error path — EC-003: tag never added)

```
Client → DELETE /api/v1/devices/d-001/tags/never-added-tag
         Authorization: Bearer test-token
DTU    → HTTP 404 {"error": "tag not found"}
```
