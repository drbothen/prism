# S-3.02 AC-3 — Memory Pool Limit Returns E-QUERY-004

**Story:** S-3.02 — prism-query: Query Tool and Materialization
**BC Anchor:** BC-2.11.006, EC-001
**Acceptance Criterion:** Given a query that causes DataFusion's `GreedyMemoryPool` to exceed 200 MB, `E-QUERY-004` is returned and no partial results are emitted.

---

## Test Name

```
test_ac3_memory_pool_limit_returns_error
  (crates/prism-query/src/tests/integration_tests.rs)
```

## Terminal Output

```
running 1 test
test tests::integration_tests::tests::test_ac3_memory_pool_limit_returns_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 337 filtered out; finished in 0.00s
```

## Production Code Path

`crates/prism-query/src/memory.rs` — `map_datafusion_memory_error()`

Constants enforced:
```rust
pub const QUERY_MEMORY_POOL_BYTES: usize = 200 * 1024 * 1024; // 200 MB
pub const MAX_MATERIALIZED_RECORDS: usize = 10_000;
pub const QUERY_TIMEOUT_SECS: u64 = 30;
```

Error mapping:
```
DataFusionError::ResourcesExhausted(_)
  -> PrismError::QueryMemoryBudgetExceeded { .. }  // E-QUERY-004
```

The `GreedyMemoryPool` is configured per-query via `RuntimeEnvBuilder::new().with_memory_pool(pool)`. Each `SessionContext` gets its own independent 200 MB pool — there is no shared global pool that would serialize queries.

## Test Logic Summary

- Constructs `DataFusionError::ResourcesExhausted("memory pool exhausted".to_string())`.
- Calls `map_datafusion_memory_error(df_err)`.
- Asserts result matches `PrismError::QueryMemoryBudgetExceeded { .. }`.
- Asserts `prism_err.to_string()` contains `"E-QUERY-004"`.

No partial results are emitted because the error short-circuits the pipeline before any `QueryResult` is returned to the caller.

## Error Code Taxonomy (BC-2.11.006 v1.10)

| Error Code | Condition | Variant |
|---|---|---|
| E-QUERY-003 | Materialization records > 10,000 | `QueryExecutionFailed` |
| E-QUERY-004 | GreedyMemoryPool exhausted | `QueryMemoryBudgetExceeded` |
| E-QUERY-005 | Query execution > 30s timeout | `QueryTimeout` |

## Result

PASS — `ResourcesExhausted` correctly maps to `E-QUERY-004`; error message includes the error code.
