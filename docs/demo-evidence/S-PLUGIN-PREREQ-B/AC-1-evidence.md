# AC-1 Evidence — HTTP Execution

**Story:** S-PLUGIN-PREREQ-B  
**HEAD SHA at capture:** `b75f317e`  
**Status:** SATISFIED

---

## AC Text (verbatim)

> **AC-1 (HTTP execution):** `PipelineExecutor::execute` issues at least one real HTTP request per
> `FetchStep` in the table spec. Given a `SensorSpec` with one `FetchStep` and a wiremock server
> registered to respond at `spec.base_url + step.path_template`, `execute` returns a
> `PipelineResult` whose `records` field is non-empty and matches the mock's JSON response body
> extracted at `step.response_path`.
>
> (traces to BC-2.16.002 postcondition — each step produces an HTTP request using method,
> path_template, and body_template as declared; response is parsed according to response_path)

---

## Implementation Evidence

**Key implementation site:**

`crates/prism-spec-engine/src/pipeline.rs:116-122` — `PipelineExecutor::execute` signature with injected HTTP client:

```rust
pub async fn execute(
    spec: &SensorSpec,
    table: &TableSpec,
    context: &FetchContext,
    http_client: &reqwest::Client,
    auth_provider: &dyn AuthProvider,
) -> Result<PipelineResult, SpecEngineError>
```

The stub `Ok(Vec::new())` from before PREREQ-B has been replaced with a real implementation that iterates `table.steps`, issues HTTP requests via `http_client`, extracts JSON at `step.response_path`, and returns non-empty `PipelineResult.records`.

---

## Linked Tests

| Test | File | Line | Status |
|------|------|------|--------|
| `test_BC_2_16_002_execute_issues_http_request_and_returns_nonempty_records` | `tests/pipeline_http_integration.rs` | 137 | PASS |
| `test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock` | `tests/pipeline_http_integration.rs` | 89 | PASS |

---

## Linked BC Postcondition

BC-2.16.002 (v1.8) — Row: "Each step produces an HTTP request using method, path_template, and body_template as declared; response is parsed according to response_path."

---

## Demo Output (real test run)

```
cargo nextest run -p prism-spec-engine \
  -E 'test(execute_issues_http_request_and_returns_nonempty_records)' \
  --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
────────────
 Nextest run ID f8b1dcbb-84b1-4afd-93a1-3e0917168018 with nextest profile: default
    Starting 1 test across 15 binaries (297 tests skipped)
        PASS [   0.011s] (1/1) prism-spec-engine::pipeline_http_integration test_BC_2_16_002_execute_issues_http_request_and_returns_nonempty_records
────────────
     Summary [   0.012s] 1 test run: 1 passed, 297 skipped
```

The test configures a wiremock server returning 3 JSON records at `/alerts` under `$.data`, then asserts `result.records.len() == 3` and `result.request_count >= 1`. Both assertions pass, proving `execute` issues real HTTP requests and returns non-empty records.
