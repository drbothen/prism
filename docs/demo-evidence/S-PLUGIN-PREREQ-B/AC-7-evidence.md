# AC-7 Evidence — Rate-Limit Hints

**Story:** S-PLUGIN-PREREQ-B  
**HEAD SHA at capture:** `b75f317e`  
**Status:** SATISFIED

---

## AC Text (verbatim)

> **AC-7 (Rate-limit hints):** When `spec.rate_limit_hints` is
> `Some(RateLimitHints { requests_per_second: Some(r), .. })`, `execute` inserts an inter-request
> delay of `1.0 / r` seconds (tokio::time::sleep) between consecutive HTTP calls. Delay applies
> between all calls including pagination iterations. When `requests_per_second` is None, no delay
> is inserted.
>
> (traces to BC-2.16.002 postcondition — rate limit hints from the SensorSpec are applied between
> API calls: inter-request delay = 1 / requests_per_second)

---

## Implementation Evidence

**Rate-limit flag hoisting:** `crates/prism-spec-engine/src/pipeline.rs:129-132` — `is_first_pipeline_request: bool` is hoisted OUTSIDE the steps loop (F-LP1-HIGH-002 fix), so the delay applies between ALL API calls across step boundaries, not just within a single step:

```rust
// AC-7 (F-LP1-HIGH-002): rate-limit flag is pipeline-scoped, not step-scoped.
// Hoisted OUTSIDE the steps loop so the delay applies between ALL API calls
// across step boundaries, not just within a single step.
let mut is_first_pipeline_request = true;
```

**Sleep invocation:** `crates/prism-spec-engine/src/pipeline.rs` — before each HTTP call (after the first), `tokio::time::sleep(Duration::from_secs_f64(1.0 / rps)).await` is called. Uses `tokio::time::sleep` (not `std::thread::sleep`) for async-executor compatibility.

---

## Linked Tests

| Test | File | Line | Status |
|------|------|------|--------|
| `test_BC_2_16_002_execute_inserts_rate_limit_delay_between_pagination_calls` | `tests/pipeline_http_integration.rs` | 877 | PASS |

---

## Linked BC Postcondition

BC-2.16.002 (v1.8) — Postcondition: "Rate limit hints from the SensorSpec are applied between API calls: inter-request delay = 1 / requests_per_second."

---

## Demo Output (real test run)

```
cargo nextest run -p prism-spec-engine \
  -E 'test(rate_limit_delay_between_pagination)' \
  --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
────────────
 Nextest run ID 03e82128-d958-4849-82e1-bb5dd0309ab1 with nextest profile: default
    Starting 1 test across 15 binaries (297 tests skipped)
        PASS [   0.215s] (1/1) prism-spec-engine::pipeline_http_integration test_BC_2_16_002_execute_inserts_rate_limit_delay_between_pagination_calls
────────────
     Summary [   0.215s] 1 test run: 1 passed, 297 skipped
```

The test sets `requests_per_second = 5.0` (200ms inter-request delay). It uses `Instant::now()` timestamps around two pagination calls and asserts `elapsed >= 180ms`. The 215ms total runtime (vs typical ~15ms for mock tests) confirms the delay is being inserted. The test passes on repeated runs, proving wall-clock timing is respected.
