# AC-6 Evidence — Fan-Out Reuse

**Story:** S-PLUGIN-PREREQ-B  
**HEAD SHA at capture:** `b75f317e`  
**Status:** SATISFIED

---

## AC Text (verbatim)

> **AC-6 (Fan-out reuse):** When a variable from step N resolves to an array, `execute` calls the
> existing `PipelineExecutor::fan_out_batches()` to split the array into batches of
> `step.fan_out_batch_size.unwrap_or(100)`, and executes the downstream step once per batch. All
> batch results are concatenated into a single result set for that step. The `fan_out_batches`
> pure function is NOT duplicated — it is called directly.
>
> (traces to BC-2.16.002 Fan-Out Behavior — when a variable interpolation resolves to an array,
> the step is executed in batches; fan-out results are concatenated into a single result set)

---

## Implementation Evidence

**Fan-out detection:** `crates/prism-spec-engine/src/pipeline.rs:191-` — the `'steps` loop calls `find_fan_out_array` (F-LP2-HIGH-001 fix) to locate the first variable in the step's templates that resolves to an array from `step_vars`. If found, the existing `fan_out_batches()` pure function is called directly.

**`fan_out_batches` signature:** `crates/prism-spec-engine/src/pipeline.rs` — the pure batch-splitting function takes an array and `batch_size: usize`, returns `Vec<Vec<Value>>`. It was unchanged from pre-PREREQ-B implementation per architectural rule "fan_out_batches MUST NOT be duplicated."

**Validation guard:** `crates/prism-spec-engine/src/validation.rs` — `fan_out_batch_size = 0` is rejected at spec-load time with a structured validation error (AC-6 EC-009 guard).

---

## Linked Tests

| Test | File | Line | Status |
|------|------|------|--------|
| `test_BC_2_16_002_execute_fan_out_invokes_step_per_batch` | `tests/pipeline_http_integration.rs` | 775 | PASS |
| `test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls` | `tests/pipeline_http_integration.rs` | 1151 | PASS |
| `test_BC_2_16_002_fanout_ambiguous_multi_array_emits_structured_event` | `tests/pipeline_http_integration.rs` | 2535 | PASS |
| `test_BC_2_16_002_fanout_invalid_source_type_emits_structured_event_for_object` | `tests/pipeline_http_integration.rs` | 2704 | PASS |
| `test_BC_2_16_002_spec_with_multi_array_fan_out_template_rejected` | `tests/pipeline_http_integration.rs` | 2303 | PASS |

---

## Linked BC Postcondition

BC-2.16.002 (v1.8) — Fan-Out Behavior: "When a variable interpolation resolves to an array, the step is executed in batches; fan-out results are concatenated into a single result set."

---

## Demo Output (real test run)

```
cargo nextest run -p prism-spec-engine \
  -E 'test(fan_out_invokes_step_per_batch) + test(fan_out_sends_distinct_batch_urls)' \
  --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
────────────
 Nextest run ID 2a1865d8-2f03-4b96-97fa-8aae5dde061e with nextest profile: default
    Starting 2 tests across 15 binaries (296 tests skipped)
        PASS [   0.013s] (1/2) prism-spec-engine::pipeline_http_integration test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls
        PASS [   0.013s] (2/2) prism-spec-engine::pipeline_http_integration test_BC_2_16_002_execute_fan_out_invokes_step_per_batch
────────────
     Summary [   0.013s] 2 tests run: 2 passed, 296 skipped
```

The fan-out invocation test: step 1 returns 250 IDs; step 2 has `fan_out_batch_size=100`. The wiremock expects exactly 3 calls to `/details` (100 + 100 + 50). The `expect(3)` wiremock assertion passes, proving `fan_out_batches()` was called and the loop executed 3 batch requests.
