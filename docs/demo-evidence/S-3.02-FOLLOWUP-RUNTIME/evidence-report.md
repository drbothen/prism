# Evidence Report — S-3.02-FOLLOWUP-RUNTIME

**Story:** prism-query: QueryEngine Execution Pipeline — Fill todo!() Sites
**Commit SHA:** `20829c80`
**Branch:** feature/S-3.02-FOLLOWUP-RUNTIME
**Evidence Date:** 2026-05-10
**Adversarial Cascade:** 9 passes (6 BLOCKED + 3 CLEAN = convergence)

---

## Acceptance Criteria Verification Matrix

| AC | Description | BC Traced | Proving Test | Log File | Verdict |
|----|-------------|-----------|-------------|---------|---------|
| AC-1 | `_sensor` virtual field populated for crowdstrike fan-out | BC-2.11.001 | `test_AC_1_query_engine_execute_with_dtu_returns_results` | ac-1-query_engine_execute_with_dtu_returns_results.log | SATISFIED |
| AC-2 | `run_materialization_pipeline` registers MemTable in fresh SessionContext | BC-2.11.005 | `test_AC_2_materialization_pipeline_non_vacuous_assertion` | ac-2-materialization_pipeline_non_vacuous_assertion.log | SATISFIED |
| AC-3 | 10K-row record cap returns E-QUERY-003 BEFORE DataFusion execution | BC-2.11.006 | `test_AC_3_size_limit_returns_e_query_003` | ac-3-size_limit_returns_e_query_003.log | SATISFIED |
| AC-4 | Filter predicate passed to SensorAdapter::fetch as push-down | BC-2.11.007 | `test_AC_4_filter_pushdown_passed_to_adapter` | ac-4-filter_pushdown_passed_to_adapter.log | SATISFIED |
| AC-5 | `register_internal_tables` makes prism_audit queryable | BC-2.15.011 | `test_AC_5_register_internal_tables_then_query_prism_audit` | ac-5-register_internal_tables_then_query_prism_audit.log | SATISFIED |
| AC-6 | Cross-client ALL scope fanned out with `_client` virtual field per org | BC-2.11.011 | `test_AC_6_cross_client_query_all_scope_fans_out` | ac-6-cross_client_query_all_scope_fans_out.log | SATISFIED |
| AC-7 | `_sensor`, `_client`, `_source_table` non-null Utf8 for every row | BC-2.11.012 | `test_AC_7_virtual_fields_present_in_all_results` | ac-7-virtual_fields_present_in_all_results.log | SATISFIED |
| AC-8 | Zero `todo!()` / `unimplemented!()` at all 9 implementation sites | POL-12 | `test_AC_8_no_todo_or_unimplemented_remains` + grep | ac-8-no_todo_or_unimplemented_remains.log + ac-8-stub-residue-clean.log | SATISFIED |

All 8 ACs: **SATISFIED**

---

## Per-AC Evidence Files

All files under `docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/` in the worktree:

```
ac-1-query_engine_execute_with_dtu_returns_results.log
ac-2-materialization_pipeline_non_vacuous_assertion.log
ac-3-size_limit_returns_e_query_003.log
ac-4-filter_pushdown_passed_to_adapter.log
ac-5-register_internal_tables_then_query_prism_audit.log
ac-6-cross_client_query_all_scope_fans_out.log
ac-7-virtual_fields_present_in_all_results.log
ac-8-no_todo_or_unimplemented_remains.log
ac-8-stub-residue-clean.log
```

Each log was captured via `cargo nextest run -p prism-query -E 'test(<name>)' 2>&1 | tee <log>`. All tests exit 0.

---

## AC-8 Stub Residue Check

Command:
```
rg --type rust 'todo!\(|unimplemented!\(' \
  crates/prism-query/src/engine.rs \
  crates/prism-query/src/materialization.rs \
  crates/prism-query/src/internal_tables.rs
```

Result: **zero matches** (rg exit code 0, empty output).
Evidence: `ac-8-stub-residue-clean.log`

Implementation sites verified clean:
- `engine.rs`: `QueryEngine::execute` (line 276), `QueryEngine::execute_scheduled` (line 317)
- `materialization.rs`: `run_materialization_pipeline` (line 241), `resolve_source_refs` (line 263)
- `internal_tables.rs`: `RocksDbTableProvider::schema` (line 125), `::table_type` (line 129), `::scan` (line 139), `::supports_filters_pushdown` (line 146), `register_internal_tables` (line 168)

---

## Walker Exhaustiveness Sweep (LP-Prefix Tests)

Command:
```
cargo nextest run -p prism-query -E 'test(/test_LP/)'
```

Result: **16/16 passed** (exit 0).
Evidence: `walker-exhaustiveness.log`

Tests run:

| Test | Module | Severity |
|------|--------|---------|
| test_LP2_CRIT_1_subquery_in_where_blocked_without_audit_read | execute_integration_tests | CRIT |
| test_LP2_CRIT_1_with_audit_read_capability_subquery_allowed | execute_integration_tests | CRIT |
| test_LP2_CRIT_1_having_subquery_blocked_without_audit_read | execute_integration_tests | CRIT |
| test_LP2_CRIT_1_scan_time_gate_rejects_without_audit_read | execute_integration_tests | CRIT |
| test_LP2_CRIT_1_descriptor_driven_non_audit_table_also_gated | execute_integration_tests | CRIT |
| test_LP2_CRIT_1_scan_time_gate_allows_with_audit_read | execute_integration_tests | CRIT |
| test_LP2_MED_2_cache_key_includes_filters | execute_integration_tests | MED |
| test_LP2_LOW_1_limit_exceeded_returns_query_limit_exceeded_variant | execute_integration_tests | LOW |
| test_LP3_CRIT_1_join_on_subquery_discovered_by_layer1 | materialization::walker_coverage_tests | CRIT |
| test_LP3_CRIT_1_group_by_subquery_discovered_by_layer1 | materialization::walker_coverage_tests | CRIT |
| test_LP3_CRIT_1_order_by_subquery_discovered_by_layer1 | materialization::walker_coverage_tests | CRIT |
| test_LP4_MED_1_func_call_args_subquery_discovered_by_layer1 | materialization::walker_coverage_tests | MED |
| test_LP5_LOW_1_pipe_join_internal_table_discovered_by_layer1 | materialization::walker_coverage_tests | LOW |
| test_LP6_LOW_1_dml_source_select_subquery_discovered_by_layer1 | materialization::walker_coverage_tests | LOW |
| test_LP6_LOW_1_dml_filter_subquery_discovered_by_layer1 | materialization::walker_coverage_tests | LOW |
| test_LP6_LOW_1_dml_source_select_appears_in_explain_sensors | explain::walker_coverage_tests | LOW |

---

## 9-Pass Adversarial Cascade Summary

See `cascade-summary.md` for the full pass-by-pass breakdown.

| Pass | Verdict | Key Finding |
|------|---------|-------------|
| pass-1 | BLOCKED-hard (0/3) | 5 CRIT: execute wiring, fan-out credential, org resolution, vacuous tests, partial failure |
| pass-2 | BLOCKED-soft (0/3) | 1 CRIT: subquery capability gate bypass (pre-execution AST walk incomplete) |
| pass-3 | BLOCKED-soft (0/3) | 1 CRIT: walker incompleteness — JOIN/GROUP BY/ORDER BY subquery positions |
| pass-4 | BLOCKED-soft (0/3) | 0 CRIT, 1 MED: FuncCall args position unwalked (treated as production-grade blocker) |
| pass-5 | BLOCKED-soft (0/3) | 0 CRIT, 1 LOW: PipeJoin walker position unwalked |
| pass-6 | BLOCKED-soft (0/3) | 0 CRIT, 1 LOW: DML source-select position unwalked in explain.rs |
| pass-7 | CLEAN (1/3) | 0 novel findings; 3 kudos |
| pass-8 | CLEAN (2/3) | 0 novel findings; idempotency holds |
| pass-9 | CLEAN (3/3) | 0 novel findings; convergence_declared: true |

Convergence path: 6 fix-passes across 6 blocked adversary passes → 3 consecutive CLEAN passes.

---

## Test Counts

| Scope | Count |
|-------|-------|
| prism-query tests | 891 |
| workspace total | 3489 |

## just check Result

`just check`: **PASS** (exit 0)

Confirmed via fix-pass-6 closure report (`target_sha: 20829c80`, `just_check_result: PASS exit 0`).

---

## Behavioral Contracts Coverage

| BC ID | Title | Satisfied By |
|-------|-------|-------------|
| BC-2.11.001 | query MCP Tool Accepts Scoping + PrismQL Query String | AC-1 |
| BC-2.11.005 | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable | AC-2 |
| BC-2.11.006 | Query Security Limits Enforcement | AC-3 |
| BC-2.11.007 | Sensor Filter Push-Down | AC-4 |
| BC-2.11.011 | Cross-Client Query Scoping | AC-6 |
| BC-2.11.012 | Virtual Fields in Queries — `_sensor`, `_client`, `_source_table` | AC-7 |
| BC-2.15.011 | Internal Table Registration — RocksDB Domains as DataFusion Tables | AC-5 |

---

## Implementation Sites Confirmed Complete

All 9 sites listed in the Objective are implemented (AC-8 verified):

1. `engine.rs`: `QueryEngine::execute`
2. `engine.rs`: `QueryEngine::execute_scheduled`
3. `materialization.rs`: `run_materialization_pipeline`
4. `materialization.rs`: `resolve_source_refs`
5. `internal_tables.rs`: `RocksDbTableProvider::schema`
6. `internal_tables.rs`: `RocksDbTableProvider::table_type`
7. `internal_tables.rs`: `RocksDbTableProvider::scan`
8. `internal_tables.rs`: `RocksDbTableProvider::supports_filters_pushdown`
9. `internal_tables.rs`: `register_internal_tables`
