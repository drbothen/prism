# Demo Evidence Report — S-PRISMQL-NATIVE-TEMPORAL-TYPING-001

**Story:** ADR-052 §D4 v1.10 — PrismQL Native Temporal Typing  
**Feature HEAD:** 9346153e  
**Recorded:** 2026-07-05  
**Product type:** CLI / Rust library (VHS terminal recordings)  
**Output directory:** `docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/`

---

## Coverage Summary

| Recording | ACs | Red Gates | Behavior | Result |
|-----------|-----|-----------|----------|--------|
| [AC-007-rfc3339-datetime-success](#ac-007) | AC-007 | RG-001 | Full RFC-3339 UTC vs Datetime col → `arrow_cast(…Timestamp)` emitted, SUCCESS | PASS |
| [AC-005-date-only-e-query-041](#ac-005) | AC-005 | RG-002, RG-014 | Date-only / offset-less vs Datetime col → E-QUERY-041 pedagogical error | PASS |
| [AC-019-string-col-coerce-success](#ac-019) | AC-019 | RG-015, RG-016, RG-017 | Date-like vs String col → COERCE to `Literal::String`, SUCCESS | PASS |
| [AC-020-numeric-col-e-query-002](#ac-020) | AC-020 | RG-016, RG-017, RG-018 | Date-like vs Integer/Float/Bool col → E-QUERY-002 type mismatch | PASS |
| [AC-030-031-group-by-order-by-e-query-042](#ac-030-031) | AC-030, AC-031 | RG-035, RG-036 | `GROUP BY '2026-06-24'` / `ORDER BY '2026-06-24'` → E-QUERY-042 analyst error | PASS |
| [AC-034-non-col-lhs-e-query-042](#ac-034) | AC-034 | RG-039 | `lower(hostname) = '2026-06-24'` / `HAVING max(ts) > '2026-06-24'` → E-QUERY-042 NonColumnLhsComparison | PASS |
| [AC-032-033-pipe-stats-sort-e-query-001](#ac-032-033) | AC-032, AC-033 | RG-037, RG-038 | Pipe `stats … by '<date>'` / `sort '<date>'` → parse-time E-QUERY-001 | PASS |
| [AC-029-select-projection-coerce-success](#ac-029) | AC-029 | RG-023 | `SELECT '2026-06-24' FROM t` projection → COERCE to String constant, SUCCESS | PASS |

**Total ACs demonstrated:** 10 (AC-005, AC-007, AC-019, AC-020, AC-029, AC-030, AC-031, AC-032, AC-033, AC-034)  
**Total Red Gates covered:** RG-001, RG-002, RG-014 through RG-018, RG-023, RG-035 through RG-039  
**All recordings:** PASS (59 production tests driving them are green on feature HEAD 9346153e)

---

## Recordings

### AC-007 — Full RFC-3339 UTC vs Datetime column: SUCCESS {#ac-007}

**Behavior:** A fully-qualified RFC-3339 timestamp (`2026-07-03T00:00:00Z`) compared against a Datetime column passes `check_temporal_literals` and the `pipe_sql_emitter` emits `arrow_cast('…', 'Timestamp(Microsecond, Some("UTC"))')`. No error is raised.

**Traces to:** AC-007 (BC-2.11.004 §Datetime-column success path), RG-001

**Tests shown:**
- `valid_rfc3339_utc_string_not_rejected`
- `full_rfc3339_regression_guard`

**Files:**
- `AC-007-rfc3339-datetime-success.gif`
- `AC-007-rfc3339-datetime-success.webm`
- `AC-007-rfc3339-datetime-success.tape`

---

### AC-005 — Date-only / offset-less vs Datetime column: E-QUERY-041 {#ac-005}

**Behavior:** A date-only (`2026-06-24`) or offset-less datetime string compared against a Datetime column is rejected with `E-QUERY-041: TemporalLiteralUnparseable`. The error message is pedagogical — it names the column type and explains the required RFC-3339 form.

**Traces to:** AC-005 (BC-2.11.004 §Datetime-column rejection), RG-002, RG-014

**Tests shown:**
- `e_query_041_sql_mode_date_only`
- `e_query_041_pipe_mode_date_only`

**Files:**
- `AC-005-date-only-e-query-041.gif`
- `AC-005-date-only-e-query-041.webm`
- `AC-005-date-only-e-query-041.tape`

---

### AC-019 — Date-like literal vs String column: COERCE SUCCESS {#ac-019}

**Behavior:** A date-like string compared against a String column is coerced: `check_temporal_literals` rewrites `RawTemporalLiteral` to `Literal::String` and the query succeeds. Enables partition key and report-date label patterns.

**Traces to:** AC-019 (BC-2.11.021 §String-column coerce), RG-015, RG-016, RG-017

**Tests shown:**
- `string_col_coercion_date_only_succeeds`
- `string_col_coercion_offset_less_succeeds`
- `string_col_coercion_space_sep_succeeds`

**Files:**
- `AC-019-string-col-coerce-success.gif`
- `AC-019-string-col-coerce-success.webm`
- `AC-019-string-col-coerce-success.tape`

---

### AC-020 — Date-like literal vs Integer/Float/Boolean column: E-QUERY-002 {#ac-020}

**Behavior:** A date-like string compared against a numeric or Boolean column is rejected with `E-QUERY-002: QueryTypeMismatch`. The three-way dispatch in `check_temporal_literals` correctly identifies the numeric arm and propagates the error.

**Traces to:** AC-020 (error-taxonomy §E-QUERY-041 three-way dispatch F-P5-MED-2), RG-016, RG-017, RG-018

**Tests shown:**
- `integer_col_date_like_e_query_002`
- `float_col_date_like_e_query_002`
- `bool_col_date_like_e_query_002`

**Files:**
- `AC-020-numeric-col-e-query-002.gif`
- `AC-020-numeric-col-e-query-002.webm`
- `AC-020-numeric-col-e-query-002.tape`

---

### AC-030 / AC-031 — GROUP BY / ORDER BY date literal: E-QUERY-042 {#ac-030-031}

**Behavior:** `GROUP BY '2026-06-24'` and `ORDER BY '2026-06-24'` are rejected with `E-QUERY-042: TemporalLiteralInvalidPosition` (variant `GroupBy` / `OrderBy`). MCP error code is `INVALID_PARAMS (-32602)`, NOT `INTERNAL_ERROR (-32000)`.

**Traces to:** AC-030, AC-031 (BC-2.11.021 §GROUP-BY/ORDER-BY rejection), RG-035, RG-036

**Tests shown:**
- `group_by_date_like_rejects_e_query_042`
- `order_by_date_like_rejects_e_query_042`

**Files:**
- `AC-030-031-group-by-order-by-e-query-042.gif`
- `AC-030-031-group-by-order-by-e-query-042.webm`
- `AC-030-031-group-by-order-by-e-query-042.tape`

---

### AC-034 — Non-column LHS comparison (HAVING / function expr): E-QUERY-042 {#ac-034}

**Behavior:** `WHERE lower(hostname) = '2026-06-24'` and `HAVING max(ts) > '2026-06-24'` are rejected with `E-QUERY-042: TemporalLiteralInvalidPosition` (variant `NonColumnLhsComparison`). The analyst-facing message explains that the temporal literal is being compared against a computed expression, not a column.

**Traces to:** AC-034 (BC-2.11.021 §non-column-LHS rejection), RG-039

**Tests shown:**
- `non_column_lhs_date_like_e_query_042`
- `having_agg_date_only_raises_e_query_042`

**Files:**
- `AC-034-non-col-lhs-e-query-042.gif`
- `AC-034-non-col-lhs-e-query-042.webm`
- `AC-034-non-col-lhs-e-query-042.tape`

---

### AC-032 / AC-033 — Pipe `stats … by '<date>'` / `sort '<date>'`: parse-time E-QUERY-001 {#ac-032-033}

**Behavior:** Pipe-mode `stats … by '2026-06-24'` and `sort '2026-06-24'` are rejected at **parse time** (E-QUERY-001), not semantic analysis time. `filter_parser.rs` rejects bare literals in `stats-by` and `sort` positions before the AST reaches `check_temporal_literals`.

**Traces to:** AC-032 (BC-2.11.004 pipe stats-by rejection), AC-033 (sort), RG-037, RG-038

**Tests shown:**
- `pipe_stats_by_date_like_e_query_001`
- `pipe_sort_date_like_e_query_001`

**Files:**
- `AC-032-033-pipe-stats-sort-e-query-001.gif`
- `AC-032-033-pipe-stats-sort-e-query-001.webm`
- `AC-032-033-pipe-stats-sort-e-query-001.tape`

---

### AC-029 — SELECT projection date literal: COERCE SUCCESS {#ac-029}

**Behavior:** `SELECT '2026-06-24' FROM t` — a date-like literal in a non-comparison SELECT projection position — is coerced by `check_temporal_literals` from `RawTemporalLiteral` to `Literal::String`. The query succeeds and returns the string constant as a column value.

**Traces to:** AC-029 (BC-2.11.021 §non-comparison projection coerce), RG-023

**Tests shown:**
- `projection_position_coerces_to_string`

**Files:**
- `AC-029-select-projection-coerce-success.gif`
- `AC-029-select-projection-coerce-success.webm`
- `AC-029-select-projection-coerce-success.tape`

---

## Implementation Verification

All recordings drive the **production engine code directly** via `cargo nextest run`. No tests are `#[ignore]`'d. The 59 Red Gate tests (53 in `prism-query` + 6 in `prism-bin`/`prism-mcp`) are all green on feature HEAD 9346153e.

Key production modules exercised:
- `crates/prism-query/src/pipe_sql_emitter.rs` — `check_temporal_literals` four-way dispatch + `arrow_cast` emitter
- `crates/prism-core/src/error.rs` — `TemporalLiteralUnparseable` (E-QUERY-041), `TemporalLiteralInvalidPosition` (E-QUERY-042), `TemporalLiteralPosition` enum
- `crates/prism-query/src/filter_parser.rs` — parse-time rejection for pipe `stats-by`/`sort` literal positions
- `crates/prism-mcp/src/error_mapping.rs` — E-QUERY-041/042 → INVALID_PARAMS (-32602) MCP mapping
