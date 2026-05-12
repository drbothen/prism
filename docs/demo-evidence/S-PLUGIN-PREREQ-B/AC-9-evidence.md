# AC-9 Evidence — VP-PLUGIN-002 Integration Test Passes

**Story:** S-PLUGIN-PREREQ-B  
**HEAD SHA at capture:** `b75f317e`  
**Status:** SATISFIED

---

## AC Text (verbatim)

> **AC-9 (VP-PLUGIN-002 integration test passes):** The test
> `test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock` in
> `tests/pipeline_http_integration.rs` passes: a `wiremock::MockServer` returns a two-record
> JSON fixture; `PipelineExecutor::execute` returns
> `PipelineResult { records: [r1, r2], request_count: 1, .. }`. This is the acceptance criterion
> for VP-PLUGIN-002 in this story's scope.
>
> (traces to BC-2.16.002 postcondition — the final step's response records are collected; and to
> VP-PLUGIN-002 — PipelineExecutor::execute returns non-empty records against at least one wiremock
> mock, replacing the Ok(Vec::new()) stub)

---

## Implementation Evidence

**Canonical VP-PLUGIN-002 test:** `crates/prism-spec-engine/tests/pipeline_http_integration.rs:89-126` — the test:
1. Starts `wiremock::MockServer`
2. Registers `GET /api/detections` returning `{"resources": [{"id": "det-001", ...}, {"id": "det-002", ...}]}`
3. Builds `SensorSpec` with `base_url = mock_server.uri()`, single FetchStep `response_path = "$.resources"`
4. Calls `PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await`
5. Asserts `result.records.len() == 2` and `result.truncated == false`

This test directly replaces the `Ok(Vec::new())` architectural fraud noted in ADR-023 §C2.

**BC-2.16.002 Structured Event Catalog cross-reference:** The test exercises the 14-row catalog's primary postcondition — "the final step's response records are collected into `PipelineResult.records`."

---

## Linked Tests

| Test | File | Line | Status |
|------|------|------|--------|
| `test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock` | `tests/pipeline_http_integration.rs` | 89 | PASS |

---

## Linked BC / VP Postconditions

- BC-2.16.002 (v1.8) — "The final step's response records are collected into `PipelineResult.records`."
- VP-PLUGIN-002 — "PipelineExecutor::execute returns non-empty records against at least one wiremock mock."

---

## Demo Output (real test run)

```
cargo nextest run -p prism-spec-engine \
  -E 'test(BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock)' \
  --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
────────────
 Nextest run ID 635c6281-d44e-4208-a404-da64addda351 with nextest profile: default
    Starting 1 test across 15 binaries (297 tests skipped)
        PASS [   0.011s] (1/1) prism-spec-engine::pipeline_http_integration test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock
────────────
     Summary [   0.012s] 1 test run: 1 passed, 297 skipped
```

VP-PLUGIN-002 is satisfied. The stub `Ok(Vec::new())` is gone. The canonical acceptance test returns 2 records from a wiremock mock server in 11ms. The architectural fraud documented in ADR-023 §C2 is closed.
