# AC-3 Evidence — Cursor Pagination

**Story:** S-PLUGIN-PREREQ-B  
**HEAD SHA at capture:** `b75f317e`  
**Status:** SATISFIED

---

## AC Text (verbatim)

> **AC-3 (Cursor pagination):** When a `FetchStep` has
> `pagination: Some(PaginationConfig::CursorToken { cursor_response_path })`, `execute` iterates
> pages: on each response, extract the cursor at `cursor_response_path`; if cursor is present and
> non-null, issue a subsequent request with the cursor value substituted; stop when cursor is null
> or the page is empty. All page records are concatenated into `PipelineResult.records`.
>
> (traces to BC-2.16.002 postcondition — pagination within a step follows the sensor spec's
> declared pagination config, iterating until the API returns an empty page or the cursor is null)

---

## Implementation Evidence

**Pagination enum:** `crates/prism-spec-engine/src/spec_parser.rs` — `PaginationConfig::CursorToken { cursor_response_path }` is the TOML-declared cursor pagination config consumed by the executor.

**Execute loop:** `crates/prism-spec-engine/src/pipeline.rs` — the `'steps` loop contains an inner pagination loop that: (1) extracts the cursor at `cursor_response_path` from each page response via the JSONPath helper, (2) breaks when cursor is `null` or absent, (3) concatenates all page records into `step_records`. The `MAX_PAGES_PER_STEP` guard (line 37, value 1,000) prevents infinite pagination on broken APIs.

---

## Linked Tests

| Test | File | Line | Status |
|------|------|------|--------|
| `test_BC_2_16_002_execute_iterates_cursor_pagination_until_null` | `tests/pipeline_http_integration.rs` | 298 | PASS |
| `test_BC_2_16_002_execute_aborts_on_non_advancing_cursor` | `tests/pipeline_http_integration.rs` | 1312 | PASS |
| `test_BC_2_16_002_execute_coerces_numeric_cursor_to_string` | `tests/pipeline_http_integration.rs` | 1460 | PASS |
| `test_BC_2_16_002_execute_aborts_at_max_pages_per_step` | `tests/pipeline_http_integration.rs` | 1558 | PASS |
| `test_BC_2_16_002_execute_percent_encodes_opaque_cursor` | `tests/pipeline_http_integration.rs` | 584 | PASS |

---

## Linked BC Postcondition

BC-2.16.002 (v1.8) — Row: "Pagination within a step follows the sensor spec's declared pagination config, iterating until the API returns an empty page or the cursor is null."

---

## Demo Output (real test run)

```
cargo nextest run -p prism-spec-engine \
  -E 'test(cursor_pagination_until_null)' \
  --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
────────────
 Nextest run ID 51f3c500-4a14-4f4a-8863-3262a6c0333a with nextest profile: default
    Starting 1 test across 15 binaries (297 tests skipped)
        PASS [   0.013s] (1/1) prism-spec-engine::pipeline_http_integration test_BC_2_16_002_execute_iterates_cursor_pagination_until_null
────────────
     Summary [   0.014s] 1 test run: 1 passed, 297 skipped
```

The test runs a two-page wiremock: page 1 returns `{"data": [e1, e2], "pagination": {"cursor": "page2-cursor"}}`, page 2 returns `{"data": [e3, e4], "pagination": {"cursor": null}}`. The test asserts `records.len() == 4`, proving all pages are concatenated and iteration stops on null cursor.
