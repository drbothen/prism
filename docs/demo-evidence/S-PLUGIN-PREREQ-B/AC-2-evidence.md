# AC-2 Evidence — Variable Interpolation Survives HTTP Boundary

**Story:** S-PLUGIN-PREREQ-B  
**HEAD SHA at capture:** `b75f317e`  
**Status:** SATISFIED

---

## AC Text (verbatim)

> **AC-2 (Variable interpolation survives HTTP boundary):** A two-step pipeline where step 2's
> `path_template` contains `${step1.access_token}` correctly resolves the token value from step 1's
> HTTP response and injects it into step 2's request URL. The existing `Interpolator` is reused for
> string substitution; the HTTP layer provides the actual response JSON for step 1.
>
> (traces to BC-2.16.002 postcondition — path_template is interpolated against variables from prior
> steps; step_name.field resolves to the field path in the step's parsed response)

---

## Implementation Evidence

**Step variable accumulation:** `crates/prism-spec-engine/src/pipeline.rs:126` — `step_vars: HashMap<String, serde_json::Value>` is populated after each step executes. Keys take the form `"step_name.field"` and are available to all subsequent steps.

**Variable seeding from query context:** `crates/prism-spec-engine/src/pipeline.rs:176-187` — `query.client_id` and `query.filter.*` keys are seeded before steps loop, making push-down filters available for interpolation.

The existing `Interpolator` (in `crates/prism-spec-engine/src/interpolation.rs`) is invoked on each step's `path_template` before the HTTP request is issued, substituting `${step_name.field}` references from `step_vars`.

---

## Linked Tests

| Test | File | Line | Status |
|------|------|------|--------|
| `test_BC_2_16_002_execute_interpolates_step1_var_into_step2_url` | `tests/pipeline_http_integration.rs` | 196 | PASS |
| `test_BC_2_16_002_two_step_pipeline_step2_uses_step1_token` | `tests/bc_2_16_002_test.rs` | 241 | PASS |

---

## Linked BC Postcondition

BC-2.16.002 (v1.8) — Row: "path_template is interpolated against variables from prior steps; `step_name.field` resolves to the field path in the step's parsed response."

---

## Demo Output (real test run)

```
cargo nextest run -p prism-spec-engine \
  -E 'test(interpolates_step1_var_into_step2_url)' \
  --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
────────────
 Nextest run ID 1dbdc734-2af8-40dc-9de7-1487ae7247ef with nextest profile: default
    Starting 1 test across 15 binaries (297 tests skipped)
        PASS [   0.012s] (1/1) prism-spec-engine::pipeline_http_integration test_BC_2_16_002_execute_interpolates_step1_var_into_step2_url
────────────
     Summary [   0.013s] 1 test run: 1 passed, 297 skipped
```

The test runs a two-step wiremock pipeline: step 1 returns `{"access_token": "tok-abc"}`, step 2's `path_template` is `/data/${step1.access_token}`. The wiremock mock is registered at `/data/tok-abc`. The test asserts non-empty records from step 2, proving the interpolation crossed the HTTP boundary correctly.
