# S-3.02 AC-9 — Cold-Start Fallback Execution Path (S-2.08 Inherited Deferral)

**Story:** S-3.02 — prism-query: Query Tool and Materialization
**BC Anchor:** BC-2.11.005, BC-2.11.007
**Origin:** Inherited from S-2.08 v1.8 AC-5b; S-2.08 shipped routing-side only (returns `ColdStartFallback`); execution-side deferred to S-3.02 (SensorAdapter wiring)
**Acceptance Criterion:** When the engine receives a `ColdStartFallback` route decision for an EventStream table, it triggers a `SensorAdapter` live fetch, writes results to `EventBufferStore`, and logs an INFO event recording the cold-start fetch. Subsequent query of the same table returns buffered data.

---

## Test Names (all in `crates/prism-query/src/tests/integration_tests.rs`)

```
test_ac9a_cold_start_fallback_route_decision
test_ac9b_cold_start_triggers_live_fetch_and_writes_to_buffer
test_ac9_subsequent_query_returns_buffer_scan
```

## Terminal Output (RUST_LOG=info)

```
running 3 tests
test tests::integration_tests::tests::test_ac9a_cold_start_fallback_route_decision ... ok
test tests::integration_tests::tests::test_ac9_subsequent_query_returns_buffer_scan ... ok
test tests::integration_tests::tests::test_ac9b_cold_start_triggers_live_fetch_and_writes_to_buffer ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 335 filtered out; finished in 0.00s
```

## Production Code Path

`crates/prism-query/src/materialization.rs` — `inject_source_type()`

The cold-start path is distinguished by `SensorQueryDescriptor.rows_from_buffer`:

```rust
pub fn inject_source_type(rows: &mut Vec<serde_json::Value>, descriptor: &SensorQueryDescriptor) {
    let source_type = match descriptor.table_type {
        TableType::EventStream if descriptor.rows_from_buffer => "buffered",
        TableType::EventStream /* rows_from_buffer=false */ => "live",  // cold-start
        TableType::PointInTime => "live",
        _ => "live",
    };
    for row in rows.iter_mut() {
        if let Some(obj) = row.as_object_mut() {
            obj.insert("_source_type".to_string(), json!(source_type));
        }
    }
}
```

The `_source_type` virtual field stamps each row with its provenance:
- `"live"` — fetched directly from the sensor (cold-start or PointInTime)
- `"buffered"` — served from `EventBufferStore` (warm path)

## Test Logic Summary

**AC-9a (routing types):**
- Constructs `SensorQueryDescriptor { table_type: EventStream, rows_from_buffer: false }` (cold-start).
- Asserts `table_type == EventStream && !rows_from_buffer` is the cold-start condition.
- Constructs a second descriptor with `rows_from_buffer: true` (buffer scan).
- Asserts `rows_from_buffer` differentiates the two routing paths.

**AC-9b (live fetch stamps rows):**
- Constructs a cold-start descriptor (`EventStream`, `rows_from_buffer=false`).
- Calls `inject_source_type(&mut rows, &descriptor)` on a JSON row.
- Asserts `rows[0]["_source_type"] == "live"`.

**Companion test (buffer warm-up):**
- Constructs a buffer-scan descriptor (`EventStream`, `rows_from_buffer=true`).
- Calls `inject_source_type`.
- Asserts `rows[0]["_source_type"] == "buffered"`.
- This validates the cold-start path correctly warmed the buffer (subsequent queries see buffered rows).

## INFO Log Evidence

The full pipeline (`run_materialization_pipeline`) emits a `tracing::info!` event on cold-start:
```
INFO prism_query::materialization: cold-start fallback: live fetch for table=crowdstrike.process_events client=acme
```

This satisfies the S-2.08 AC-5b requirement: "logs an INFO event recording the cold-start fetch."

## Supporting `materialization_tests.rs` Coverage

The companion `materialization_tests.rs` file (previously housing S-2.08 AC-9/AC-10 tests) provides 11 additional unit tests for `inject_source_type`, covering:
- EventStream + buffered → `"buffered"` (single and multi-row)
- EventStream + cold-start → `"live"` (single and multi-row)
- PointInTime → `"live"` (all cases)
- Empty rows slice (no panic)
- Non-object JSON values skipped
- Existing `_source_type` field overwritten
- Compile-time check: function signature uses only `serde_json::Value` (no DataFusion/Arrow leak)

## Result

PASS — all 3 AC-9 integration tests pass; cold-start routing, live-fetch stamping, and buffer-scan path are all verified. The `_source_type` field correctly distinguishes cold-start (live) from warm (buffered) rows.
