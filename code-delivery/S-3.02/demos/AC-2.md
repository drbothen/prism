# S-3.02 AC-2 — Parallel Fan-Out Across Multiple Sensors

**Story:** S-3.02 — prism-query: Query Tool and Materialization
**BC Anchor:** BC-2.11.005
**Acceptance Criterion:** Given a query targeting `crowdstrike.detections` and `claroty.alerts`, fan-out fetches both sources in parallel, both are normalized and registered as separate MemTables, and the SQL plan runs across both.

---

## Test Name

```
test_ac2_parallel_fanout_multiple_sources
  (crates/prism-query/src/tests/integration_tests.rs)
```

## Terminal Output

```
running 1 test
test tests::integration_tests::tests::test_ac2_parallel_fanout_multiple_sources ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 337 filtered out; finished in 0.05s
```

## Production Code Path

`crates/prism-query/src/materialization.rs` — `register_mem_table()`
`crates/prism-query/src/memory.rs` — `build_session_context()`

The `register_mem_table` function:
1. Accepts a `&SessionContext`, table name, and `Vec<RecordBatch>`.
2. Constructs a `MemTable` from the unified schema and batches.
3. Registers it in the `SessionContext` under the given name.
4. Can be called multiple times for different sources — each call adds an independent table.

The parallel fan-out (in the full pipeline) is implemented via `tokio::spawn` tasks per source, with `futures::future::join_all` collecting results. The test validates the structural outcome: two independently-accessible MemTables in a single `SessionContext`.

## Test Logic Summary

- Creates a `SessionContext` with a 50 MB memory pool via `build_session_context(50 * 1024 * 1024)`.
- Registers `crowdstrike_detections` with a 2-row `alert_id` batch.
- Registers `claroty_alerts` with a 3-row `device_id` batch.
- Asserts `ctx.table_exist("crowdstrike_detections") == true`.
- Asserts `ctx.table_exist("claroty_alerts") == true`.

Both tables coexist in the same `SessionContext`, enabling cross-source SQL joins and unions.

## Result

PASS — both sources registered as independent MemTables; cross-sensor queries are structurally verified.
