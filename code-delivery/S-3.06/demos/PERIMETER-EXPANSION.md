# PERIMETER-EXPANSION — 9 New Restricted Symbols (BC-2.11.006 v1.11)

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.006 v1.11 | **INV:** INV-SEC-PERIMETER-001

## Summary

S-3.06 adds 9 new `pub(crate)` write-parser symbols to the security perimeter guard.
The perimeter-violation crate now enforces 27 total compile errors (was 18 after S-3.01).

| Metric | S-3.01 Baseline | S-3.06 After |
|--------|----------------|--------------|
| E0603 errors (function private) | 13 | 22 |
| E0624 errors (method/fn private) | 5 | 5 |
| Total errors | 18 | 27 |
| Net new symbols guarded | — | +9 |

## Verification Command

```bash
cargo check --color=never \
  --manifest-path tests/external/perimeter-violation/Cargo.toml
```

## Actual Output (all 27 errors)

```
error[E0603]: function `parse_filter` is private
error[E0603]: function `parse_filter_with_limits` is private
error[E0603]: function `parse_sql` is private
error[E0603]: function `parse_sql_with_limits` is private
error[E0603]: function `parse_pipe` is private
error[E0603]: function `parse_pipe_with_limits` is private
error[E0603]: function `build_pipe_parser` is private
error[E0603]: function `build_predicate_parser` is private
error[E0603]: function `build_source_ref_parser` is private
error[E0603]: function `build_string_parser` is private
error[E0603]: function `build_literal_parser` is private
error[E0603]: function `build_expr_parser` is private
error[E0603]: function `build_pipe_mode_parser` is private
error[E0603]: function `parse_pipe_with_write` is private        ← S-3.06 NEW
error[E0603]: function `build_write_stage_parser` is private     ← S-3.06 NEW
error[E0603]: function `build_write_arg_parser` is private       ← S-3.06 NEW
error[E0603]: function `extract_sensor_prefix` is private        ← S-3.06 NEW
error[E0603]: function `parse_sql_dml` is private                ← S-3.06 NEW
error[E0603]: function `build_dml_parser` is private             ← S-3.06 NEW
error[E0603]: function `is_internal_prism_table` is private      ← S-3.06 NEW
error[E0603]: function `check_unbounded_write` is private        ← S-3.06 NEW
error[E0603]: function `reject_write_verbs_in_filter` is private ← S-3.06 NEW
error[E0624]: associated function `snapshot` is private
error[E0624]: method `install_thread_local` is private
error[E0624]: associated function `snapshot` is private
error[E0624]: associated function `clear_thread_local` is private
error[E0624]: associated function `current_regex_limit` is private

error: could not compile `perimeter-violation` due to 27 previous errors
```

Exit code: 101 (compile failure = security guard is active).

## 9 New Symbols (S-3.06)

| Symbol | Module | Guard Type | Rationale |
|--------|--------|------------|-----------|
| `parse_pipe_with_write` | `pipe_parser` | E0603 | Write-stage entry point — external bypass would allow unregistered verb injection |
| `build_write_stage_parser` | `pipe_parser` | E0603 | Chumsky builder — external code must not construct write parsers directly |
| `build_write_arg_parser` | `pipe_parser` | E0603 | Chumsky builder — argument parser internal to write stage |
| `extract_sensor_prefix` | `pipe_parser` | E0603 | Sensor prefix extractor — internal helper; no external contract |
| `parse_sql_dml` | `sql_parser` | E0603 | DML entry point — external bypass skips prism_* table guard |
| `build_dml_parser` | `sql_parser` | E0603 | Chumsky DML builder — internal composition detail |
| `is_internal_prism_table` | `sql_parser` | E0603 | Table guard predicate — external code must not call this independently |
| `check_unbounded_write` | `sql_parser` | E0603 | Unbounded write guard — must not be bypassed by external callers |
| `reject_write_verbs_in_filter` | `filter_parser` | E0603 | Filter-mode write rejection — external code must not use this to probe verb registration |

## Baseline (S-3.01 — 18 errors)

The S-3.01 perimeter file (`2d7040b1`) contained 14 `use` statements producing 18
expected E-errors (13 E0603 + 5 E0624). These covered sub-parser entry points, builder
factories, and the thread-local ParseLimits API.

The S-3.06 perimeter file (`f37332ca`) extends to 27 expected E-errors by adding 9
new `use` statements (all E0603). The E0624 count remains 5 (unchanged — no new
`ParseLimits` methods were added).

## CI Gate

The `perimeter-compile-fail` CI job checks that `cargo check -p perimeter-violation`
exits non-zero. If any listed symbol ever becomes accidentally `pub`, the job passes
(exit 0) and blocks the merge with a security regression alert.

The current exit code is 101 — all 27 guards are active.

## Git Commit

Perimeter expansion committed in: `f37332ca`
Commit message: `feat(S-3.06): add 9 new restricted symbols to perimeter-violation crate`
