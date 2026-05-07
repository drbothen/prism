# S-3.06 Demo Evidence — PrismQL Write Parser Extensions

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.004 | **Test count:** 62 passed, 0 failed
**Date:** 2026-05-06 | **Branch:** feature/S-3.06 | **HEAD:** f4a61f08

## Test Run Summary

All 62 tests pass across integration (`tests/write_parser_tests.rs`) and unit
(`src/tests/write_parser_unit_tests.rs`) suites.

```
test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Acceptance Criteria

| File | AC | Criterion | Status |
|------|----|-----------|--------|
| [AC-1.md](AC-1.md) | AC-1 | Pipe write no args — WriteNode with verb="contain", args=[] | PASS |
| [AC-2.md](AC-2.md) | AC-2 / EC-11-060 | Write stage not terminal — E-QUERY-024 | PASS |
| [AC-3.md](AC-3.md) | AC-3 / EC-11-061 | prism_* table write protection — E-QUERY-010 at parse time | PASS |
| [AC-4.md](AC-4.md) | AC-4 / EC-11-064 | Filter mode rejects write verbs — grammar-level rejection | PASS |
| [AC-5.md](AC-5.md) | AC-5 / EC-11-063 | Unknown verb suggestion — E-QUERY-023 with verb list | PASS |
| [AC-6.md](AC-6.md) | AC-6 | Pipe write with args — WriteArg key=value pairs | PASS |
| [AC-7.md](AC-7.md) | AC-7 / EC-11-062 | Unbounded DML rejection — E-QUERY-022 | PASS |
| [AC-8.md](AC-8.md) | AC-8 | INSERT INTO SELECT — DmlNode::InsertInto | PASS |

## Edge Cases

| File | EC | Description | Status |
|------|----|-------------|--------|
| [EC-11-060.md](EC-11-060.md) | EC-11-060 | Write verb in non-terminal pipe position | PASS |
| [EC-11-061.md](EC-11-061.md) | EC-11-061 | SQL DML targeting prism_* table | PASS |
| [EC-11-062.md](EC-11-062.md) | EC-11-062 | DELETE FROM with no WHERE clause | PASS |
| [EC-11-063.md](EC-11-063.md) | EC-11-063 | Unrecognised verb in terminal position | PASS |
| [EC-11-064.md](EC-11-064.md) | EC-11-064 | Write verb in filter mode | PASS |
| [EC-11-065.md](EC-11-065.md) | EC-11-065 | WriteVerbRegistry with empty verb set | PASS |

## Verification Properties

| File | VP | Description | Status |
|------|----|-------------|--------|
| [VP-021.md](VP-021.md) | VP-021 | Panic safety on write-mode inputs (fuzz corpus extension) | PASS |

## Security Perimeter

> **Post-S-3.06 update (BC-2.11.006 v1.16, 2026-05-07):** Symbol counts updated to **26 total / 27 expected E-errors / 9 layer-4 symbols** after `maintenance/clippy-unwrap-cleanup` removed dead-code `build_dml_parser`. Body counts below reflect the v1.14 snapshot at S-3.06 merge time.

| File | Description | Status |
|------|-------------|--------|
| [PERIMETER-EXPANSION.md](PERIMETER-EXPANSION.md) | 10 new restricted symbols (BC-2.11.006 v1.14) | PASS |

Perimeter guard: 28 total compile errors (was 18 after S-3.01). Exit code 101 confirmed.

## Key Source Files

| File | Purpose |
|------|---------|
| `crates/prism-query/src/write_ast.rs` | WriteNode, DmlNode, WriteArg, DmlOperation, Assignment AST types |
| `crates/prism-query/src/write_verb_registry.rs` | WriteVerbRegistry + WriteVerbSource trait |
| `crates/prism-query/src/pipe_parser.rs` | parse_pipe_with_write, build_write_stage_parser, build_write_arg_parser, extract_sensor_prefix |
| `crates/prism-query/src/sql_parser.rs` | parse_sql_dml, build_dml_parser, is_internal_prism_table, check_unbounded_write |
| `crates/prism-query/src/filter_parser.rs` | reject_write_verbs_in_filter |
| `crates/prism-query/src/error.rs` | E-QUERY-010/022/023/024 constructors |
| `crates/prism-query/tests/write_parser_tests.rs` | 62 integration tests |
| `crates/prism-query/src/tests/write_parser_unit_tests.rs` | pub(crate) unit tests |
| `tests/external/perimeter-violation/src/main.rs` | 27-symbol (28 E-error) perimeter compile-fail test |

## Note on VHS

VHS was not used for this story. Prism has no compiled CLI binary — all behavior is
in the parser library (`prism_query::PrismQlParser`). Evidence is provided as Rust
test assertions (the only correct demo medium for a pure parser library). All 62 test
pass results above constitute the equivalent of VHS recordings for this artifact type.
