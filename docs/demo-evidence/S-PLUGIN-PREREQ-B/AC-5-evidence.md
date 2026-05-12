# AC-5 Evidence — 401 Retry via AuthProvider

**Story:** S-PLUGIN-PREREQ-B  
**HEAD SHA at capture:** `b75f317e`  
**Status:** SATISFIED

---

## AC Text (verbatim)

> **AC-5 (401 retry via AuthProvider):** When any HTTP step returns 401-Unauthorized, `execute`
> calls `auth_provider.acquire_token(spec, client_id)` exactly once per 401 occurrence, replaces
> the `Authorization: Bearer` header with the fresh token, and retries the failed step ONCE. If
> the retry also returns 401, the pipeline aborts with a structured `SpecEngineError`. The
> `AuthProvider` trait is defined in `prism-spec-engine/src/auth_provider.rs` and re-exported
> via `lib.rs`.
>
> (traces to BC-2.16.002 precondition — credentials for the sensor's auth_type have been resolved
> for the target client_id; and BC-2.01.013 postcondition — adapter implementations are produced
> from TOML SensorSpec declarations at runtime; auth re-acquisition is spec-driven not hardcoded)

---

## Implementation Evidence

**AuthProvider trait definition:** `crates/prism-spec-engine/src/auth_provider.rs:89-105` — object-safe trait with boxed future return, supporting `&dyn AuthProvider` call sites.

**Test helpers available:** `auth_provider.rs:120-324` — `NullAuthProvider`, `MockAuthProvider`, `FailingAuthProvider`, `ChainAuthProvider` — all feature-gated under `cfg(test)` or `test-helpers`.

**Eager token acquisition:** `crates/prism-spec-engine/src/pipeline.rs:143-174` — `acquire_token` is called before the steps loop starts (eager acquisition pattern, F-LP5-LOW-003 closure). On 401 mid-pipeline, `acquire_token` is called a second time and the step is retried once. Double-401 on retry returns `SpecEngineError::AuthRefreshFailed`.

**Trait object safety proof:** `crates/prism-spec-engine/src/auth_provider.rs:343-353` — compile-time Red Gate test confirms `&dyn AuthProvider` coercion compiles.

---

## Linked Tests

| Test | File | Line | Status |
|------|------|------|--------|
| `test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401` | `tests/pipeline_oauth_retry.rs` | 75 | PASS |
| `test_BC_2_16_002_execute_aborts_on_double_401` | `tests/pipeline_oauth_retry.rs` | 158 | PASS |
| `test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request` | `tests/pipeline_oauth_retry.rs` | 214 | PASS |
| `test_BC_2_16_002_eager_auth_initial_failed_aborts_pipeline_immediately` | `tests/pipeline_oauth_retry.rs` | 284 | PASS |
| `test_BC_2_16_002_no_auth_refresh_triggered_on_legitimate_execution` | `tests/pipeline_oauth_retry.rs` | 337 | PASS |
| `test_BC_2_16_002_auth_provider_trait_object_is_object_safe` | `src/auth_provider.rs` | 343 | PASS |

---

## Linked BC Postconditions

- BC-2.16.002 (v1.8) — Precondition: "credentials for the sensor's auth_type have been resolved for the target client_id"
- BC-2.01.013 (v1.6) — Postcondition: "adapter implementations are produced from TOML SensorSpec declarations at runtime; auth re-acquisition is spec-driven not hardcoded"

---

## Demo Output (real test run)

```
cargo nextest run -p prism-spec-engine \
  -E 'test(auth_provider_acquire_token_on_401) + test(aborts_on_double_401) + test(acquires_token_eagerly)' \
  --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
────────────
 Nextest run ID 321fd218-b188-443d-aae7-c090ce4d58e9 with nextest profile: default
    Starting 3 tests across 15 binaries (295 tests skipped)
        PASS [   0.012s] (1/3) prism-spec-engine::pipeline_oauth_retry test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request
        PASS [   0.012s] (2/3) prism-spec-engine::pipeline_oauth_retry test_BC_2_16_002_execute_aborts_on_double_401
        PASS [   0.012s] (3/3) prism-spec-engine::pipeline_oauth_retry test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401
────────────
     Summary [   0.013s] 3 tests run: 3 passed, 295 skipped
```

The 401-retry test asserts: (a) `auth_provider.calls() == 2` (1 eager + 1 on-401 refresh), (b) `result.records.len() == 2` (retry succeeded), (c) `result.request_count >= 2` (initial 401 + retry). All pass.
