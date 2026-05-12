# AC-4 Evidence — Offset Pagination

**Story:** S-PLUGIN-PREREQ-B  
**HEAD SHA at capture:** `b75f317e`  
**Status:** SATISFIED

---

## AC Text (verbatim)

> **AC-4 (Offset pagination):** When a `FetchStep` has
> `pagination: Some(PaginationConfig::OffsetLimit { page_size })`, `execute` iterates pages using
> an incrementing offset: `offset = 0, page_size, 2*page_size, ...`. Stops when a page returns
> fewer records than `page_size`. All records concatenated.
>
> (traces to BC-2.16.002 postcondition — pagination within a step follows the sensor spec's
> declared pagination config)

---

## Implementation Evidence

**Offset pagination type:** `crates/prism-spec-engine/src/spec_parser.rs` — `PaginationConfig::OffsetLimit { page_size }` is the TOML-declared offset config.

**Execute logic:** `crates/prism-spec-engine/src/pipeline.rs` — when `pagination` is `OffsetLimit`, the inner pagination loop: (1) starts at `offset = 0`, (2) appends `?offset=N&limit=page_size` query params to the URL on each iteration, (3) breaks when `page_records.len() < page_size` (short page = no more data), (4) concatenates all page records.

---

## Linked Tests

| Test | File | Line | Status |
|------|------|------|--------|
| `test_BC_2_16_002_execute_iterates_offset_pagination_until_short_page` | `tests/pipeline_http_integration.rs` | 396 | PASS |

---

## Linked BC Postcondition

BC-2.16.002 (v1.8) — Row: "Pagination within a step follows the sensor spec's declared pagination config."

---

## Demo Output (real test run)

```
cargo nextest run -p prism-spec-engine \
  -E 'test(offset_pagination_until_short_page)' \
  --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
────────────
 Nextest run ID e110c64b-2e90-41a3-9864-13341d66c3fd with nextest profile: default
    Starting 1 test across 15 binaries (297 tests skipped)
        PASS [   0.011s] (1/1) prism-spec-engine::pipeline_http_integration test_BC_2_16_002_execute_iterates_offset_pagination_until_short_page
────────────
     Summary [   0.012s] 1 test run: 1 passed, 297 skipped
```

The test uses `page_size = 3`. Page 1 wiremock returns 3 records (full page — continue). Page 2 wiremock returns 2 records (short page — stop). The test asserts `records.len() == 5`, proving offset pagination iterates until a short page and concatenates all results.
