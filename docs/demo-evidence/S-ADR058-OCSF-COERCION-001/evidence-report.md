# Demo Evidence Report — S-ADR058-OCSF-COERCION-001

**Story:** S-ADR058-OCSF-COERCION-001 — OCSF coercion path hardening (ADR-058)
**Feature HEAD:** `26d0362246972f77c185f0926540ddc9f78ef761`
**Recorded:** 2026-08-20

## Recording Method

All 8 acceptance criteria have VHS terminal recordings rendered as `.gif`
(PR-embeddable) and `.webm` (archival). Recordings were produced with VHS
0.11.0 (`/opt/homebrew/bin/vhs`), FiraCode Nerd Font Mono, Catppuccin Mocha
theme. The `.tape` scripts use `Sleep 15s` in place of `Wait+Line` due to
a VHS 0.11.0 regression where `Wait+Line` times out before shell output is
available; this substitution is correct for a warm build where nextest
completes in ~5 seconds.

Each recording shows the exact `cargo nextest run` invocation for the AC's
Red Gate test(s) and the PASS + Summary lines from nextest output.

All 10 Red Gate tests pass on this branch. The full workspace check
(`just check`, 5765 tests) also passes on this branch.

## Coverage Table

| AC    | Evidence Artifact(s)                                                    | Observed Result                                                                                                                                                                                                          | Verdict |
|-------|-------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------|
| AC-001 | `AC-001-coerce_value-string-array-err.gif`, `AC-001-coerce_value-string-array-err.webm`, `AC-001-nextest-output.txt` | `PASS [0.013s] prism-spec-engine::bc_2_16_003_test test_coerce_value_string_type_array_input_returns_err_coercion_warning` — `coerce_value` with `ColumnType::String` receiving a JSON Array returns `Err` carrying a `column_coercion_failure` warning. | PASS |
| AC-002 | `AC-002-coerce_value-string-object-err.gif`, `AC-002-coerce_value-string-object-err.webm`, `AC-002-nextest-output.txt` | `PASS [0.013s] prism-spec-engine::bc_2_16_003_test test_coerce_value_string_type_object_input_returns_err_coercion_warning` — `coerce_value` with `ColumnType::String` receiving a JSON Object returns `Err` carrying a `column_coercion_failure` warning. | PASS |
| AC-003 | `AC-003-coerce_value-integer-string-parse.gif`, `AC-003-coerce_value-integer-string-parse.webm`, `AC-003-nextest-output.txt` | `PASS [0.017s] (1/2) test_coerce_value_integer_type_string_non_numeric_path_parse_success_returns_number` — parseable string `"42"` is coerced to integer `42`. `PASS [0.017s] (2/2) test_coerce_value_integer_type_string_non_numeric_path_parse_failure_returns_err` — non-parseable string `"many"` returns `Err`. | PASS |
| AC-004 | `AC-004-map_record-coercion-failure-warn.gif`, `AC-004-map_record-coercion-failure-warn.webm`, `AC-004-nextest-output.txt`; live-MCP wire (HS-001) | `PASS [0.019s] prism-spec-engine column_mapping::tests::test_map_record_string_object_input_demotes_to_raw_extensions_and_emits_warning` — `map_record` with a String-typed column receiving a JSON Object demotes the value to `raw_extensions` and emits a `column_coercion_failure` warn event. Live-MCP holdout HS-001 confirmed: wire row contains `"description":null` (key **present**, value null — not absent) alongside a `column_coercion_failure` warn log line. | PASS |
| AC-005 | `AC-005-build_column_array-string-object-null.gif`, `AC-005-build_column_array-string-object-null.webm`, `AC-005-nextest-output.txt`; live-MCP wire (HS-001) | `PASS [0.046s] prism-bin spec_driven_adapter::tests::test_build_column_array_string_type_object_input_returns_null_and_emits_warning` — `build_column_array` Path-A: String column receiving Object produces a `null` cell (null-not-absent, key present in serialized row) and emits a warning. Consistent with HS-001 wire shape. | PASS |
| AC-006 | `AC-006-non-regression-string-coerce.gif`, `AC-006-non-regression-string-coerce.webm`, `AC-006-nextest-output.txt` | `PASS [0.016s] (1/2) test_coerce_value_string_type_normalizes_integer_to_string` — integer `42` is coerced to string `"42"`. `PASS [0.018s] (2/2) test_coerce_value_string_type_preserves_string_username_against_uid_heuristic` — string `"jdoe"` is preserved as-is against the UID numeric heuristic. Non-regression: valid scalar-to-String coercions are not broken by the new Array/Object rejection path. | PASS |
| AC-007 | `AC-007-build_column_array-integer-string-parse.gif`, `AC-007-build_column_array-integer-string-parse.webm`, `AC-007-nextest-output.txt`; live-MCP wire (HS-002, HS-003) | `PASS [0.039s] (1/2) test_build_column_array_integer_type_string_parseable_returns_integer` — parseable string `"42"` yields integer `42`. `PASS [0.041s] (2/2) test_build_column_array_integer_type_string_non_parseable_returns_null_and_emits_warning` — non-parseable string yields null + warning. Live-MCP holdout HS-002 confirmed `"42"` wire value → `42` (integer); HS-003 confirmed `"many"` wire value → `null` + `column_coercion_failure` warn. | PASS |
| AC-008 | `AC-008-integer-object-null-both-paths.gif`, `AC-008-integer-object-null-both-paths.webm`, `AC-008-nextest-output.txt`; live-MCP wire (HS-004) | `PASS [0.033s] (1/2) prism-spec-engine column_mapping::tests::test_coerce_value_integer_type_object_input_returns_err_coercion_warning` — `coerce_value` path: Integer column receiving Object returns `Err`. `PASS [0.045s] (2/2) prism-bin spec_driven_adapter::tests::test_build_column_array_integer_type_object_input_returns_null_and_emits_warning` — `build_column_array` path: Integer column receiving Object produces null + warning. Live-MCP holdout HS-004 confirmed structured/array values are handled consistently on the ENRICH-1 path (Array preserved as JSON-list string, no silent data loss). | PASS |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria | 8 |
| Red Gate tests | 10 |
| Tests passing | 10 |
| ACs with VHS terminal recordings (GIF + WebM) | 8 / 8 |
| ACs with nextest evidence | 8 / 8 |
| ACs with live-MCP wire evidence (holdout) | 4 / 4 (AC-004, AC-005, AC-007, AC-008) |
| Overall verdict | ALL PASS |

## POL-10 Compliance

Evidence lives under `docs/demo-evidence/S-ADR058-OCSF-COERCION-001/`
(story-scoped subfolder). No files placed at flat `docs/demo-evidence/*.md`.
