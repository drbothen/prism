# S-3.02 Demo Evidence Index

**Story:** S-3.02 — prism-query: Query Tool and Materialization  
**Branch:** `feature/S-3.02` (worktree: `.worktrees/S-3.02`)  
**Commit at capture:** `a35290f1`  
**All 356 tests GREEN** at time of recording.

**VHS available:** YES (`/opt/homebrew/bin/vhs`)  
**Recording method:** Markdown evidence sidecars with captured terminal output (VHS terminal recording not used — the product is a Rust library, not a CLI binary; all ACs are exercised via `cargo test --nocapture`. VHS `.tape` recordings would show only test runner output, providing no additional signal over captured stdout.)

---

## Acceptance Criterion Summary

| AC | Title | BC Anchor | Test Name(s) | Status |
|---|---|---|---|---|
| AC-1 | Virtual fields injected in every row | BC-2.11.001, BC-2.11.012 | `test_ac1_virtual_fields_present_in_every_row` | PASS |
| AC-2 | Parallel fan-out to multiple sensors | BC-2.11.005 | `test_ac2_parallel_fanout_multiple_sources` | PASS |
| AC-3 | Memory pool limit returns E-QUERY-004 | BC-2.11.006 | `test_ac3_memory_pool_limit_returns_error` | PASS |
| AC-4 | REQUIRED column push-down | BC-2.11.007 | `test_ac4_required_column_push_down` | PASS |
| AC-5 | None clients fans out to all | BC-2.11.011 | `test_ac5_none_clients_fans_out_to_all` | PASS |
| AC-6 | Cross-client merge with `_client` field | BC-2.11.011 | `test_ac6_cross_client_data_merged_with_client_field` | PASS |
| AC-7 | SessionContext dropped after execute() (RAII) | BC-2.11.005 | `test_ac7_session_context_dropped_after_execute` | PASS |
| AC-8 | VP-031 push-down property test corpus | BC-2.11.007, VP-031 | `prop_required_columns_always_push_down` + 9 more | PASS (10/10 props) |
| AC-9 | Cold-start fallback execution (S-2.08 inherited) | BC-2.11.005, BC-2.11.007 | `test_ac9a/b`, `test_ac9_subsequent_query_returns_buffer_scan` | PASS |

---

## File Index

| File | AC | Description |
|---|---|---|
| `AC-1.md` | AC-1 | Virtual fields — `_sensor`, `_client`, `_source_table` in every row |
| `AC-2.md` | AC-2 | Parallel fan-out — two sources as independent MemTables |
| `AC-3.md` | AC-3 | GreedyMemoryPool exhaustion → E-QUERY-004; error code taxonomy |
| `AC-4.md` | AC-4 | REQUIRED column predicate in `push_down`; BC-2.11.007 taxonomy table |
| `AC-5.md` | AC-5 | `clients: None` → all 3 configured clients resolved |
| `AC-6.md` | AC-6 | `_client` field distinguishes acme vs. contoso rows in merged MemTable |
| `AC-7.md` | AC-7 | `SessionScope` RAII drop; `into_arc()` for `execute_scheduled` |
| `AC-8.md` | AC-8 | 10 VP-031 proptest cases; full column option taxonomy coverage |
| `AC-9.md` | AC-9 | Cold-start `_source_type=live`, buffer-scan `_source_type=buffered`; 3 tests |

---

## Notes

- All tests ran from `/Users/jmagady/Dev/prism/.worktrees/S-3.02` against the `feature/S-3.02` branch.
- AC-8 ran with `PROPTEST_CASES=32` (CI default is 256; all properties held).
- AC-9 includes both the inherited S-2.08 AC-5b execution path and the companion "warm buffer" test.
- No ACs required follow-up; all 9 have complete evidence.
