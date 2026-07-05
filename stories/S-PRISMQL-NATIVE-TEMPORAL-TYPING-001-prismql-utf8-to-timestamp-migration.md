---
document_type: story
story_id: S-PRISMQL-NATIVE-TEMPORAL-TYPING-001
title: "PrismQL Native Temporal Typing — migrate ColumnType::Datetime from Arrow Utf8 to Timestamp(Microsecond, UTC) (ADR-052)"
epic_id: EPIC-DEMO
version: "1.8"
status: draft
producer: story-writer
phase: 3
wave: wave-5-e-demo-fidelity
priority: P1
points: 8
tdd_mode: strict
# tdd_mode rationale: this story modifies production Rust code across prism-bin
# (spec_driven_adapter.rs), prism-core (error.rs), prism-query (ast.rs,
# sql_parser.rs, filter_parser.rs, engine.rs, pipe_sql_emitter.rs,
# tests/high002_plan_pinning_tests.rs), and prism-sensors (OCSF normalization boundary).
# All behavioral changes have corresponding Red Gate tests written as failing todo!() stubs
# BEFORE production code is modified. The grep-sweep ACs (AC-014, AC-023) and VERIFY-only
# ACs (AC-015) have no Red Gate tests but do not justify facade mode — production code is
# modified with new semantic behavior.
target_module: prism-query
subsystems: [SS-09, SS-10, SS-11]
depends_on: []
# depends_on: none blocking — operates on develop@122228e8 (S-DEMO-FIDELITY-REMEDIATION-001 merged)
blocks: []
# NOTE (D8 sequencing): This story MUST MERGE before any ADR-051 (typed-enrichment) implementation
# story is dispatched. ADR-051's D1 datetime row must be amended from DataType::Utf8 to
# DataType::Timestamp(Microsecond, Some("UTC")) once ADR-052 is implemented and merged.
# When the ADR-051 story is registered, add its ID to blocks: [].
behavioral_contracts: [BC-2.11.021, BC-2.11.003, BC-2.11.004, BC-2.11.001]
# BC behavioral anchors at authoring time (v1.6 story update for ADR-052 §D4 non-comparison coerce):
#   BC-2.11.021 (active) — amended per ADR-052 §D4 v1.10 / error-taxonomy v2.14:
#     SELECT projection → COERCE to Literal::String (success);
#     GROUP BY / ORDER BY bare literal (SQL) → E-QUERY-042 (TemporalLiteralInvalidPosition::GroupBy/OrderBy, INVALID_PARAMS);
#     non-column-LHS comparison (function/expr LHS, date-like RHS) → E-QUERY-042 (NonColumnLhsComparison, INVALID_PARAMS);
#     pipe stats-by / sort bare literal → parse-time E-QUERY-001 (enhanced message)
#   BC-2.11.003 (draft) — amended per ADR-052: same Option-A E-QUERY-041 semantics
#   BC-2.11.004 (active) — amended per ADR-052: same
#   BC-2.11.001 (active) — governs the query MCP tool pipeline (unchanged)
# Pre-done spec work (ADR-052-bc-amendment-burst 2026-07-03 + ADR-052 §D4 option-A amendment burst 2026-07-04 +
#   ADR-052 §D4 7-form amendment burst 2026-07-04):
#   error-taxonomy.md §E-QUERY-041: row updated to Option-A AST-walk mechanism + three-way dispatch (Integer/Float/Bool arm corrected E-QUERY-001→E-QUERY-002, F-P5-MED-2)
#   BC-2.11.021 §Postconditions: postcondition + E-QUERY-041 detection = Option-A; coercion arm added
#   BC-2.11.021 §Error Cases: is_date_like 7-form set enumerated; EC-11-021-010..014 added; over-match documented
#   BC-2.11.003 §Postconditions: same amendments
#   BC-2.11.004 §Postconditions: same amendments
# ADR-044 supersession pre-completed by architect (superseded_by frontmatter + §Status block already present);
# implementer VERIFIES only (AC-015) — no CHANGE to ADR-044 by implementer
verification_properties: [VP-021]
# VP-021 (fuzz, never panics): parser and emitter changes must not introduce panics.
# VP-021 is directly tested by RG-022 (unicode input → no panic — regression guard for
# the VP-021 violation that occurred in the text-scanner via raw byte-offset slicing).
# The existing vp021_parse_fuzz target continues to cover the parser.
assumption_validations: []
risk_mitigations: []
# risk_mitigations: ADR-052 RISK-1 (MEDIUM — version-drift silent coercion) addressed by
# AC-002 (RISK-1 mandatory DataFusion arrow_cast probe — RG-002).
# ADR-052 RISK-3 (diff_results CF Arrow IPC schema compatibility) addressed by AC-009.
# ADR-052 RISK-4 (Literal sibling-sweep blast radius) addressed by Task 15 (TD-VSDD-060
# sibling-sweep) and verified by AC-017/AC-022.
# ADR-052 RISK-5 (is_date_like false positives in non-Datetime string comparisons):
# RESOLVED BY DESIGN per ADR-052 §D4 coercion arm — String-column coercion arm eliminates all false
# E-QUERY-001 for date-like literals vs String/Utf8 columns; tested by RG-013 (AC-019).
red_gate_tests: 39
estimated_days: "3"
---

# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: PrismQL Native Temporal Typing

Migrate `ColumnType::Datetime` from Arrow `DataType::Utf8` to
`DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))` throughout
the PrismQL query pipeline, per ADR-052.

## Narrative

As a PrismQL implementer, I want all sensor `datetime` columns registered in DataFusion
as `Timestamp(Microsecond, Some("UTC"))` (not `Utf8`), the SQL emitter updated to emit
explicit `arrow_cast('...', 'Timestamp(Microsecond, Some("UTC"))')` typed literals,
E-QUERY-041 returned for bare non-RFC-3339 string literals in datetime column comparisons
via the **Option-A lenient-parse-then-AST-walk** mechanism (`Literal::RawTemporalLiteral`
produced by the parser; `check_temporal_literals` AST walker with four-way dispatch),
and String/Utf8 column comparisons against date-like literals safely COERCED (not
rejected), so that temporal predicates (`WHERE timestamp > NOW() - INTERVAL '24h'`) use
native typed timestamp comparison, E-QUERY-041 is delivered via a provably correct
schema-resolved mechanism, and VP-021 (no-panic) is maintained for all inputs.

## Background

### Pre-done spec work (do NOT redo in this story)

The following factory spec amendments were completed before TDD dispatch:

| Item | Status | Anchor |
|------|--------|--------|
| error-taxonomy.md: E-QUERY-041 row — Option-A AST-walk + three-way dispatch | DONE | §E-QUERY-041 |
| BC-2.11.021: postcondition + E-QUERY-041 = Option-A mechanism; coercion arm | DONE | §Postconditions |
| BC-2.11.003: same Option-A E-QUERY-041 semantics | DONE | §Postconditions |
| BC-2.11.004: same amendments | DONE | §Postconditions |

The only factory spec change IN SCOPE for this story is:

| Item | Status |
|------|--------|
| ADR-044 frontmatter `superseded_by: "ADR-052 (§D4 only)"` + Status section annotation | IN SCOPE (AC-015) |

### Why §D4 v1.2 was retired (8 failed fix-bursts)

The v1.2 design implemented E-QUERY-041 detection as a PARSE-FAIL recovery path: the
PrismQL parser rejected date-like literals with E-QUERY-001, then `engine.rs` re-scanned
the raw query string using text-scanner functions (`extract_table_name_from_query_str`,
`extract_column_name_adjacent_to_quoted_value`, `is_bad_literal_in_datetime_column`) to
upgrade the error to E-QUERY-041. This design failed 8 consecutive fix-bursts because:
- Dotted-source expressions (`source.column`) were mis-classified as missing sources
- Filter-mode `source |` prefix broke the table-name extractor
- Non-ASCII MCP input caused a **Unicode byte-offset panic** (raw byte-slice into
  to_lowercase output → VP-021 violation — unchecked panic on valid analyst input)
- Qualified nested columns (`other.timestamp`, `payload.event_time`) produced
  false E-QUERY-041 (`.last()` collapse matched the primary table)

**Key reframe:** E-QUERY-041 is a MESSAGE-SPECIFICITY UPGRADE, not a correctness gate.
Date-only and offset-less literals already FAIL the PrismQL parser and never reach
DataFusion — there is NO silent-wrong-result risk regardless. The text-scanner existed
only to upgrade the generic error message. Option A achieves the same upgrade with zero
text scanning, zero raw byte-offset operations, and full schema resolution.

### ADR-052 Decision Map

| Decision | What the implementer must do |
|----------|------------------------------|
| D1 | Use `DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))` everywhere — canonical Rust construction form |
| D2 | Change `column_type_to_arrow` Datetime arm in `spec_driven_adapter.rs`; fix stale doc comment in `column.rs` |
| D3 | Change `Literal::Timestamp` rendering in `pipe_sql_emitter.rs` to `arrow_cast(...)` form; ADD `Literal::RawTemporalLiteral` guard arm → E-QUERY-002 (`QueryPlanFailed`) |
| D4 | **Option-A lenient-parse-then-AST-walk** (see §D4 below); ADD `Literal::RawTemporalLiteral(String)` to ast.rs; lenient parser fallback; `check_temporal_literals` four-way dispatch; DELETE text-scanner functions |
| D5 | Confirm `pushdown.rs` T1 extractor still produces RFC-3339 via `.to_rfc3339()` — no change |
| D6 | Investigate `diff_results` CF: confirm no Arrow IPC stored, or add startup clear |
| D7 | Annotate ADR-044 with partial supersession scope (VERIFY ONLY — architect pre-completed) |
| D8 | This story ships before any ADR-051 implementation |

**D1 canonical Rust construction form (compile-verified):**

```rust
DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))
```

The timezone field is `Option<Arc<str>>`. `Arc::from("UTC")` produces `Arc<str>` directly.
`Some(Arc::new("UTC".into()))` does NOT compile correctly — `"UTC".into()` infers to
`String`, producing `Arc<String>` which is the wrong type. NEVER use `Some(Arc::new("UTC".into()))`.

**D3 — `arrow_cast` form required (DataFusion 53.1.0 verified):**

`TIMESTAMP '...'` SQL literal lowers to `Timestamp(Nanosecond, None)` in DataFusion
53.1.0. The explicit `arrow_cast` form produces exactly `Timestamp(Microsecond, Some("UTC"))`:

```rust
// Before:
Literal::Timestamp(ts) => format!("'{}'", ts.iso8601),
// After:
Literal::Timestamp(ts) => format!("arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')", ts.iso8601),
```

**D4 — Option-A lenient-parse-then-AST-walk (ADR-052, human-ratified 2026-07-04):**

**Step 1 — New AST node: `Literal::RawTemporalLiteral(String)` in `ast.rs`.**

A new variant is added to the `Literal` enum (alongside `Literal::Timestamp`):
```rust
/// A quoted string that resembles a date or datetime but is NOT valid RFC-3339.
/// Produced by the parser for date-only (`'2026-06-24'`) and offset-less
/// (`'2026-06-24T12:00:00'`) forms. Validated at plan time by
/// `check_temporal_literals`. Must never reach SQL emission.
RawTemporalLiteral(String),
```

**Step 2 — Parser lenient fallback in `sql_parser.rs` (and `filter_parser.rs` if separate).**

The timestamp literal combinator is changed from FAIL-on-non-RFC3339 to LENIENT:

```
quoted_string →
  if parse_from_rfc3339 succeeds → Literal::Timestamp  (unchanged)
  else if is_date_like(s) → Literal::RawTemporalLiteral(s)  (new: parse SUCCEEDS)
  else → Literal::String(s)  (unchanged: 'not-a-date', 'sensor-id-abc' stay here)
```

`is_date_like(s)` is a multi-format heuristic matching the **7 canonical forms** pinned in
ADR-052 §D4 and BC-2.11.021 §Error Cases. The implementer must transcribe these EXACT 7 format
strings — no more, no fewer:
- Form 1: `chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()` (date-only)
- Form 2: `NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok()` (T-sep full seconds)
- Form 3: `NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").is_ok()` (T-sep fractional)
- Form 4: `NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").is_ok()` (T-sep no seconds)
- Form 5: `NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").is_ok()` (space-sep full seconds)
- Form 6: `NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f").is_ok()` (space-sep fractional)
- Form 7: `NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").is_ok()` (space-sep no seconds)

The heuristic does NOT match arbitrary strings; `'not-a-date'` and `'2026-06-24extra'` remain
`Literal::String`. **Over-matched forms** (unpadded digits `'2026-6-24'`, big/signed years
`'12345-06-24'`) ALSO match (chrono `%m`/`%d`/`%Y` accept them) — this is **ACCEPTED BENIGN**
per ADR-052 §D4; no regex guard or year-width constraint is applied.

With this change, parsing a query containing `'2026-06-24'` SUCCEEDS and produces
an AST containing `Expr::Literal(Literal::RawTemporalLiteral("2026-06-24"))`.
No `E-QUERY-001` is emitted at parse time for this class of input.

**Step 3 — Plan-time validator: `check_temporal_literals` AST walker in `engine.rs`.**

After AST production and schema resolution, `check_temporal_literals` walks the full
`Expr` tree applying a refined dispatch on each `Literal::RawTemporalLiteral` node
(ADR-052 §D4 v1.10 / error-taxonomy v2.14):

| `RawTemporalLiteral` position | Schema check | Result |
|-------------------------------|--------------|--------|
| Comparison against Datetime/Timestamp column (column LHS) | `column_type == Timestamp(Microsecond, UTC)` | E-QUERY-041 (pedagogical upgrade) |
| Comparison against String/Utf8 column (column LHS) | `column_type == DataType::Utf8` | COERCE: rewrite to `Literal::String(s)` in-place → compare as ordinary string literal (SUCCESS — no error; byte-identical to pre-ADR-052 behavior) |
| Comparison against Integer / Float / Bool column (column LHS) | numeric/boolean type | E-QUERY-002 (type mismatch — `QueryTypeMismatch { column, table, actual_type, operator }`) |
| Comparison — non-column LHS (function/expr LHS, date-like RHS) | LHS is not a plain column reference | E-QUERY-042 (`TemporalLiteralInvalidPosition::NonColumnLhsComparison`, INVALID_PARAMS) — NOT `-32000` |
| SELECT projection / function arg (non-comparison, not GROUP BY/ORDER BY) | no column-comparison schema context | COERCE: rewrite to `Literal::String(s)` in-place → SUCCESS (returns string constant; ADR-052 §D4 v1.10) |
| GROUP BY bare literal (SQL mode) | non-comparison group-by position | E-QUERY-042 (`TemporalLiteralInvalidPosition::GroupBy`, INVALID_PARAMS) |
| ORDER BY bare literal (SQL mode) | non-comparison order-by position | E-QUERY-042 (`TemporalLiteralInvalidPosition::OrderBy`, INVALID_PARAMS) |
| Pipe `stats … by` bare literal | parse-time (before AST walker) | parse-time E-QUERY-001 (enhanced message; `stats by` only accepts field paths — rejected by `filter_parser.rs` before `check_temporal_literals` runs) |
| Pipe `sort` bare literal | parse-time (before AST walker) | parse-time E-QUERY-001 (enhanced message; `sort` only accepts field paths) |

The schema-resolved column type is determined by the same path that resolves
`ColumnType::Datetime → DataType::Timestamp(...)` (D2). This correctly handles:
- **Dotted expressions** (`source.column`, `payload.event_time`): resolved via schema map
- **Filter-mode** (`source | WHERE col > ...`): AST captures source + predicate structure
- **Qualified/nested columns** (`other.timestamp`): resolved against the CORRECT source table
- **Unicode inputs**: operates on already-parsed `String` values (valid UTF-8); NO raw byte-offset operations; VP-021 violation eliminated by construction

**Coercion arm — String/Utf8 column:** When `check_temporal_literals` finds
`RawTemporalLiteral(s)` against a String/Utf8 column, it rewrites in-place to
`Literal::String(s)`. Because `pipe_sql_emitter.rs` emits `Literal::String(s)` as
`'{escaped}'` (the same path as before ADR-052), this coercion is byte-identical to
pre-ADR-052 behavior. `WHERE string_col = '2026-06-24'` succeeds with no error
(partition keys, report-date labels, ISO-date-formatted external IDs are legitimate).
E-QUERY-041 fires ONLY when the comparison target resolves to `DataType::Timestamp(Microsecond, UTC)`.

**E-QUERY-042 — `TemporalLiteralInvalidPosition` (ADR-052 §D4 v1.10, error-taxonomy v2.14):**

Three additional positions where a `RawTemporalLiteral` must be REJECTED rather than coerced. Add
a companion error variant and enum to `crates/prism-core/src/error.rs`:

```rust
/// E-QUERY-042: A date-like temporal literal appeared in a structurally invalid position.
/// Positions: GroupBy, OrderBy, NonColumnLhsComparison.
TemporalLiteralInvalidPosition {
    position: TemporalInvalidPosition,
    value_prefix: String,  // first 50 chars, UTF-8 boundary-safe (same as E-QUERY-041)
},

/// Position variants for `PrismError::TemporalLiteralInvalidPosition` (E-QUERY-042).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TemporalInvalidPosition {
    GroupBy,
    OrderBy,
    NonColumnLhsComparison,
}
```

- **SQL `GROUP BY '2026-06-24'`**: `check_temporal_literals` detects `RawTemporalLiteral` in a
  GROUP BY clause → `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::GroupBy, value_prefix })`.
  Display per error-taxonomy.md §E-QUERY-042. Maps to `codes::INVALID_PARAMS` (-32602).
- **SQL `ORDER BY '2026-06-24'`**: same, with `TemporalInvalidPosition::OrderBy`.
- **Non-column-LHS comparison** (e.g., `WHERE lower(hostname) = '2026-06-24'`): the LHS of the
  comparison is a function call or expression, not a plain column reference; `check_temporal_literals`
  detects `RawTemporalLiteral` on the RHS when the LHS is non-column → `TemporalInvalidPosition::NonColumnLhsComparison`.
  Maps to `codes::INVALID_PARAMS` — NOT `-32000` (internal error).
- **Pipe-mode `stats … by '2026-06-24'`** and **`sort '2026-06-24'`**: rejected at PARSE TIME by
  `filter_parser.rs` with enhanced E-QUERY-001 messages before `check_temporal_literals` runs.
  The pipe grammar parser rejects bare literals in `stats by` and `sort` positions with an enhanced
  parse-error (e.g., "`stats by` only accepts field paths, not literal values"). These never reach
  the AST walker as `RawTemporalLiteral` nodes.

**Step 4 — Deletion: text-scanner apparatus removed from `engine.rs`.**

The following functions and code paths are DELETED:
- `extract_table_name_from_query_str`
- `extract_column_name_adjacent_to_quoted_value`
- `is_bad_literal_in_datetime_column`
- The parse-fail branch that called the above

**Step 5 — Guard: `RawTemporalLiteral` guard arm in `pipe_sql_emitter.rs`.**

`pipe_sql_emitter.rs` adds a `Literal::RawTemporalLiteral` arm → E-QUERY-002 (`QueryPlanFailed` — unvalidated temporal literal reached emission; belt-and-suspenders guard).
Under correct plan execution, `check_temporal_literals` consumes all `RawTemporalLiteral`
nodes before the emitter is called. The guard arm makes this invariant explicit and testable.

**Option-A dispatch table (E-QUERY-041 / COERCE / E-QUERY-002 boundary):**

| Input | Parser output | AST walker result |
|-------|--------------|------------------|
| `'2026-07-03T00:00:00Z'` (full RFC-3339) | `Literal::Timestamp` | No check (not RawTemporalLiteral) |
| `'2026-06-24'` (form 1: date-only) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24T12:00:00'` (form 2: T-sep full seconds) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24T12:00:00.123'` (form 3: T-sep fractional) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24T12:00'` (form 4: T-sep no seconds) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24 12:00:00'` (form 5: space-sep full seconds) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24 12:00:00.500'` (form 6: space-sep fractional) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24 12:00'` (form 7: space-sep no seconds) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-6-24'` (unpadded, over-match ACCEPTED BENIGN) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 (ACCEPTED BENIGN — "use RFC-3339" is accurate) |
| `'2026-06-24'` vs String/Utf8 col | `Literal::RawTemporalLiteral` | COERCE → `Literal::String("2026-06-24")` (SUCCESS) |
| `'2026-06-24 12:00:00'` (form 5) vs String/Utf8 col | `Literal::RawTemporalLiteral` | COERCE → `Literal::String("2026-06-24 12:00:00")` (SUCCESS) |
| `'2026-6-24'` (unpadded) vs String/Utf8 col | `Literal::RawTemporalLiteral` | COERCE → `Literal::String("2026-6-24")` (SUCCESS) |
| `'2026-06-24'` vs Integer / Float / Bool col | `Literal::RawTemporalLiteral` | E-QUERY-002 (QueryTypeMismatch) |
| `'2026-06-24'` in SELECT projection or function arg | `Literal::RawTemporalLiteral` | COERCE → `Literal::String("2026-06-24")` (SUCCESS — string constant; ADR-052 §D4 v1.10) |
| `SELECT count(*) FROM t GROUP BY '2026-06-24'` (SQL GROUP BY) | `Literal::RawTemporalLiteral` | E-QUERY-042 (TemporalLiteralInvalidPosition::GroupBy, INVALID_PARAMS) |
| `SELECT * FROM t ORDER BY '2026-06-24'` (SQL ORDER BY) | `Literal::RawTemporalLiteral` | E-QUERY-042 (TemporalLiteralInvalidPosition::OrderBy, INVALID_PARAMS) |
| `FROM t \| stats count by '2026-06-24'` (pipe `stats by`) | parse-time | parse-time E-QUERY-001 (enhanced message; `stats by` only accepts field paths) |
| `FROM t \| sort '2026-06-24'` (pipe `sort`) | parse-time | parse-time E-QUERY-001 (enhanced message; `sort` only accepts field paths) |
| `WHERE lower(hostname) = '2026-06-24'` (non-column-LHS comparison) | `Literal::RawTemporalLiteral` | E-QUERY-042 (TemporalLiteralInvalidPosition::NonColumnLhsComparison, INVALID_PARAMS) — NOT -32000 |
| `'not-a-date'` anywhere | `Literal::String` | No temporal error |
| `'2026-06-24extra'` (trailing chars, near-miss) anywhere | `Literal::String` | No temporal error (chrono full-consumption rejects) |

**E-QUERY-041 message format (unchanged from v1.2, POL-24):**
```
E-QUERY-041: The value '{first_50_chars}' cannot be interpreted as a UTC timestamp.
Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only
and offset-less forms are not accepted. For relative time filters, use
NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h').
```

**RISK-1 (MEDIUM — version-drift silent coercion, unchanged from v1.2):**

DataFusion 53.1.0 verified: `TIMESTAMP '...'` produces `Timestamp(Nanosecond, None)`.
Comparing `Timestamp(Nanosecond, None)` against `Timestamp(Microsecond, Some("UTC"))` does
NOT error — `temporal_coercion_nonstrict_timezone` inserts a lossless cast. The risk is:
if the `arrow_cast` explicit form is reverted, the implicit coercion works today but may
change in a future DataFusion minor version without any compilation error.

**Mitigation (mandatory probe test RG-002):** Verify that
`arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` produces
`Timestamp(Microsecond, Some("UTC"))` in the DataFusion plan output.

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) v1.8 | ~19,500 |
| ADR-052 (full ADR) | ~6,000 |
| BC-2.11.021 | ~3,500 |
| BC-2.11.003 | ~3,000 |
| BC-2.11.004 | ~2,500 |
| BC-2.11.001 | ~6,000 |
| error-taxonomy.md (E-QUERY-041 row) | ~2,500 |
| `crates/prism-bin/src/spec_driven_adapter.rs` | ~12,000 |
| `crates/prism-core/src/error.rs` | ~5,000 |
| `crates/prism-core/src/column.rs` | ~500 |
| `crates/prism-query/src/ast.rs` | ~3,000 |
| `crates/prism-query/src/sql_parser.rs` | ~10,000 |
| `crates/prism-query/src/filter_parser.rs` | ~6,000 |
| `crates/prism-query/src/engine.rs` | ~12,000 |
| `crates/prism-query/src/pipe_sql_emitter.rs` | ~8,000 |
| `crates/prism-query/src/tests/` | ~8,000 |
| `crates/prism-sensors/src/` (normalization paths) | ~6,000 |
| `crates/prism-mcp/src/error_mapping.rs` | ~4,000 |
| `crates/prism-query/src/pushdown.rs` (verify only) | ~5,000 |
| **Total** | **~122,000** |

Estimated at ~61% of a 200K context window. Within the per-story limit. No split required.

## Tasks

1. **Read** ADR-052 in full: `.factory/specs/architecture/decisions/ADR-052-prismql-native-temporal-typing-utf8-to-arrow-timestamp.md` — especially §D1 (Arc::from form), §D3 (arrow_cast emitter + RawTemporalLiteral guard), §D4 (Option-A steps 1–5, four-way dispatch table, coercion arm), §Blast Radius items 1–20, and §Risk RISK-1/RISK-4/RISK-5.

2. **Read** BC-2.11.021: `.factory/specs/behavioral-contracts/BC-2.11.021-temporal-grammar-now-interval-planning-time-constant-injection.md` — verify the Option-A E-QUERY-041 mechanism, coercion arm, four-way dispatch, and message format.

3. **Read** BC-2.11.003 and BC-2.11.004 — verify the ADR-052 D2 Timestamp typing assertions and Option-A E-QUERY-041 description (NOT chrono pre-validator).

4. **Read** `crates/prism-core/src/error.rs` — understand the existing `PrismError` enum structure. **Read** `crates/prism-mcp/src/error_mapping.rs` — understand the `map_prism_error` function and existing match arms.

5. **Write Red Gate test stubs FIRST** — add the following failing `todo!()` stub tests BEFORE writing any production code:

   a. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_datetime_column_registers_as_timestamp_micros_utc`
      in `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`

   b. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_risk1_datafusion_arrow_cast_probe`
      in `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`

   c. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sql_emitter_yields_arrow_cast_literal`
      in `crates/prism-query/src/` emitter test module

   d. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_sql_mode_date_only_string`
      in `crates/prism-query/src/tests/`

   e. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_pipe_mode_date_only_string`
      in `crates/prism-query/src/tests/`

   f. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_map_prism_error_invalid_params`
      in `crates/prism-mcp/src/` (error_mapping.rs test module or tests/ directory)

   g. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_valid_rfc3339_utc_string_not_rejected`
      in `crates/prism-query/src/tests/`

   h. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sensor_datetime_string_parsed_to_micros`
      in `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`

   i. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_high002_datetime_column_type_is_timestamp`
      in `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`

   j. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_output_plans_against_timestamp_column`
      in `crates/prism-query/src/tests/` (emitter test module or high002_plan_pinning_tests.rs)

   **New stubs for deep Red Gate coverage (Option-A mechanism):**

   k. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_full_rfc3339_regression_guard`
      in `crates/prism-query/src/tests/` — asserts parser emits `Literal::Timestamp` (not
      `RawTemporalLiteral`) for `'2026-07-03T00:00:00Z'`; `check_temporal_literals` does not
      fire; query succeeds. Regression guard: full RFC-3339 must never be downgraded.

   l. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_parser_emits_raw_temporal_for_date_only`
      in `crates/prism-query/src/tests/` (parser unit tests) — asserts parsing `'2026-06-24'`
      SUCCEEDS and produces `Literal::RawTemporalLiteral("2026-06-24")` in the AST (NOT a parse
      error). Same for offset-less `'2026-06-24T12:00:00'`.

   m. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_offset_less_datetime_col`
      in `crates/prism-query/src/tests/` — asserts `WHERE timestamp > '2026-06-24T12:00:00'`
      against a Datetime col → E-QUERY-041 (not E-QUERY-001 at parse time).

   n. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_date_only_succeeds`
      in `crates/prism-query/src/tests/` — asserts `WHERE string_col = '2026-06-24'` against a
      String/Utf8 col SUCCEEDS (no error); emitted SQL contains `string_col = '2026-06-24'`
      byte-identical to pre-ADR-052 behavior. RISK-5 regression guard.

   o. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_offset_less_succeeds`
      in `crates/prism-query/src/tests/` — same as stub n but for offset-less
      `'2026-06-24T12:00:00'` vs String col → COERCE, SUCCEEDS.

   p. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_integer_col_date_like_e_query_001`
      in `crates/prism-query/src/tests/` — asserts `'2026-06-24'` vs Integer col → E-QUERY-002 (`QueryTypeMismatch`). (Test function name retains `_e_query_001` suffix — not renamed per append-only naming policy.)

   q. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_float_col_date_like_e_query_001`
      in `crates/prism-query/src/tests/` — asserts `'2026-06-24'` vs Float col → E-QUERY-002 (`QueryTypeMismatch`).

   r. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_bool_col_date_like_e_query_001`
      in `crates/prism-query/src/tests/` — asserts `'2026-06-24'` vs Bool col → E-QUERY-002 (`QueryTypeMismatch`).

   s. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_non_date_like_stays_string_literal`
      in `crates/prism-query/src/tests/` — asserts `'not-a-date'` and `'sensor-id-abc'` remain
      `Literal::String` after parsing (NOT `RawTemporalLiteral`); `check_temporal_literals` ignores
      them; no temporal error emitted.

   t. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_dotted_source_column_resolution`
      in `crates/prism-query/src/tests/` — asserts `source.timestamp_col > '2026-06-24'` (dotted
      source + column) correctly resolves `timestamp_col` to the Datetime schema type → E-QUERY-041.
      Must NOT misclassify the dotted source as a missing table.

   u. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_qualified_nested_column_resolution`
      in `crates/prism-query/src/tests/` — asserts `other.event_time > '2026-06-24'` where
      `other` is a different source table resolves against `other`'s schema (not the primary
      table's schema); correct column type determined; no false E-QUERY-041 on wrong type.

   v. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_filter_pipe_syntax_e_query_041`
      in `crates/prism-query/src/tests/` — asserts `source | where ts_col > '2026-06-24'`
      (pipe syntax with `|`) → E-QUERY-041 (not misclassified; same as SQL mode result).

   w. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_unicode_input_no_panic`
      in `crates/prism-query/src/tests/` — asserts that a PrismQL query containing multi-byte
      Unicode characters adjacent to or surrounding a date-like literal (e.g., `'2026-06-24'`
      embedded in a query with CJK or emoji chars) does NOT panic. Assert graceful error or
      success; assert NOT a panic. VP-021 regression guard.

   x. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_projection_position_e_query_001`
      in `crates/prism-query/src/tests/` — asserts `RawTemporalLiteral` in a non-comparison
      projection position (e.g., `SELECT '2026-06-24' FROM t`) COERCES to `Literal::String` →
      query SUCCEEDS; emitted projection is `'2026-06-24'` (string constant; per ADR-052 §D4).
      (Test function name retains `_e_query_001` suffix — not renamed per append-only naming policy.)

   y. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_guard_raw_temporal_literal`
      in `crates/prism-query/src/` (emitter test module) — directly calls the
      `pipe_sql_emitter.rs` `Literal::RawTemporalLiteral` arm; asserts it returns E-QUERY-002 (`QueryPlanFailed`)
      (not a panic, not a string emission). Guard arm reachability test.

   z. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_message_format_byte_identical`
      in `crates/prism-query/src/tests/` — triggers E-QUERY-041 via `'2026-06-24'` vs Datetime
      col; asserts the `Display` string matches POL-24 template byte-for-byte:
      `"E-QUERY-041: The value '2026-06-24' cannot be interpreted as a UTC timestamp. Expected
      RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only and offset-less
      forms are not accepted. For relative time filters, use NOW() - INTERVAL 'Nh' (e.g., WHERE
      timestamp > NOW() - INTERVAL '24h')."`

   **New stubs for ADR-052 §D4 is_date_like 7-form set — Red Gate boundary coverage:**

   aa. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_fractional_t_sep_datetime_col`
       in `crates/prism-query/src/tests/` — asserts `'2026-06-24T12:00:00.123'` (form 3:
       T-sep fractional, `%Y-%m-%dT%H:%M:%S%.f`) vs Datetime col → E-QUERY-041 (RG-026).

   ab. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_no_seconds_t_sep_datetime_col`
       in `crates/prism-query/src/tests/` — asserts `'2026-06-24T12:00'` (form 4:
       T-sep no seconds, `%Y-%m-%dT%H:%M`) vs Datetime col → E-QUERY-041 (RG-027).

   ac. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_full_seconds_datetime_col`
       in `crates/prism-query/src/tests/` — asserts `'2026-06-24 12:00:00'` (form 5:
       space-sep full seconds, `%Y-%m-%d %H:%M:%S`) vs Datetime col → E-QUERY-041 (RG-028).

   ad. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_fractional_datetime_col`
       in `crates/prism-query/src/tests/` — asserts `'2026-06-24 12:00:00.500'` (form 6:
       space-sep fractional, `%Y-%m-%d %H:%M:%S%.f`) vs Datetime col → E-QUERY-041 (RG-029).

   ae. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_no_seconds_datetime_col`
       in `crates/prism-query/src/tests/` — asserts `'2026-06-24 12:00'` (form 7:
       space-sep no seconds, `%Y-%m-%d %H:%M`) vs Datetime col → E-QUERY-041 (RG-030).

   af. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_space_sep_succeeds`
       in `crates/prism-query/src/tests/` — asserts `'2026-06-24 12:00:00'` (form 5 space-sep)
       vs String/Utf8 col → COERCE → SUCCESS; emitted SQL byte-identical. RISK-5 extension
       to space-sep family (RG-031).

   ag. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_unpadded_date_overmatch_datetime_col`
       in `crates/prism-query/src/tests/` — asserts `'2026-6-24'` (unpadded month/day,
       over-match ACCEPTED BENIGN) vs Datetime col → E-QUERY-041; no regex guard applied (RG-032).

   ah. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_unpadded_date_succeeds`
       in `crates/prism-query/src/tests/` — asserts `'2026-6-24'` (unpadded, over-match)
       vs String/Utf8 col → COERCE → SUCCESS; byte-identical to pre-ADR-052 (RG-033).

   ai. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_near_miss_trailing_chars_stays_utf8`
       in `crates/prism-query/src/tests/` — asserts `'2026-06-24extra'` (trailing chars,
       near-miss) produces `Literal::String` (NOT `RawTemporalLiteral`); no temporal error;
       confirms chrono full-consumption requirement (RG-034).

   **New stubs for ADR-052 §D4 v1.8 non-comparison coerce — Red Gate coverage:**

   aj. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_projection_group_by_date_like_coerces`
       in `crates/prism-query/src/tests/` — FLIPPED per ADR-052 §D4 v1.10 / error-taxonomy v2.14:
       asserts a date-like literal (`'2026-06-24'`) in a SQL GROUP BY position
       (e.g., `SELECT count(*) FROM t GROUP BY '2026-06-24'`) → E-QUERY-042
       (`TemporalLiteralInvalidPosition::GroupBy`); maps to INVALID_PARAMS (-32602).
       Assert error IS `TemporalLiteralInvalidPosition`; assert it does NOT map to -32000. (RG-035)
       (Test function name unchanged per append-only naming policy — "coerces" is a historical artifact.)

   ak. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_order_by_date_like_coerces`
       in `crates/prism-query/src/tests/` — FLIPPED per ADR-052 §D4 v1.10 / error-taxonomy v2.14:
       asserts a date-like literal (`'2026-06-24'`) in a SQL ORDER BY position
       (e.g., `SELECT * FROM t ORDER BY '2026-06-24'`) → E-QUERY-042
       (`TemporalLiteralInvalidPosition::OrderBy`); maps to INVALID_PARAMS (-32602).
       Assert error IS `TemporalLiteralInvalidPosition`; assert it does NOT map to -32000. (RG-036)
       (Test function name unchanged per append-only naming policy.)

   al. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_stats_by_date_like_e_query_001`
       in `crates/prism-query/src/tests/` — asserts `FROM t | stats count by '2026-06-24'`
       (pipe-mode `stats by` clause with a bare date-like literal) → parse-time E-QUERY-001
       (enhanced message; "`stats by` only accepts field paths"). Assert error fires at parse
       time; assert error code is E-QUERY-001 (not E-QUERY-042). (RG-037)

   am. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sort_date_like_e_query_001`
       in `crates/prism-query/src/tests/` — asserts `FROM t | sort '2026-06-24'`
       (pipe-mode `sort` clause with a bare date-like literal) → parse-time E-QUERY-001
       (enhanced message; "`sort` only accepts field paths"). Assert error fires at parse
       time; assert error code is E-QUERY-001. (RG-038)

   an. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_non_column_lhs_date_like_e_query_042`
       in `crates/prism-query/src/tests/` — asserts `WHERE lower(hostname) = '2026-06-24'`
       (non-column LHS: function call expression, date-like RHS) → E-QUERY-042
       (`TemporalLiteralInvalidPosition::NonColumnLhsComparison`); maps to INVALID_PARAMS.
       Assert error IS `TemporalLiteralInvalidPosition::NonColumnLhsComparison`; assert it
       does NOT map to -32000. (RG-039)

   Verify ALL stubs FAIL (compile error for variants that don't exist yet, `todo!()` panic
   for the rest) before proceeding to step 6.

6. **Add** `PrismError::TemporalLiteralUnparseable { value_prefix: String }` variant to
   `crates/prism-core/src/error.rs`. Verify `PrismError` carries `#[non_exhaustive]` — if
   not, add it (CLAUDE.md `#[non_exhaustive]` discipline for pub-API types in prism-core).
   The Display for this variant MUST match (POL-24 byte-for-byte):
   ```
   "E-QUERY-041: The value '{value_prefix}' cannot be interpreted as a UTC timestamp.
   Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only
   and offset-less forms are not accepted. For relative time filters, use
   NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h')."
   ```
   where `{value_prefix}` is the `value_prefix` field (first 50 chars of offending literal,
   truncated at UTF-8 codepoint boundary per error-taxonomy.md §E-QUERY-041 convention).

   Also add **`PrismError::TemporalLiteralInvalidPosition`** variant and companion
   **`TemporalInvalidPosition`** enum (E-QUERY-042, error-taxonomy v2.14):
   ```rust
   /// E-QUERY-042: A date-like temporal literal appeared in a structurally invalid position.
   TemporalLiteralInvalidPosition {
       position: TemporalInvalidPosition,
       value_prefix: String,  // first 50 chars, UTF-8 boundary-safe (same truncation as E-QUERY-041)
   },
   ```
   And in the same file (or a companion module), define:
   ```rust
   /// Identifies the invalid position for `PrismError::TemporalLiteralInvalidPosition` (E-QUERY-042).
   #[derive(Debug, Clone, PartialEq)]
   #[non_exhaustive]
   pub enum TemporalInvalidPosition {
       GroupBy,
       OrderBy,
       NonColumnLhsComparison,
   }
   ```
   The Display for `TemporalLiteralInvalidPosition` MUST match error-taxonomy.md §E-QUERY-042 POL-24
   template for the specific position variant. `TemporalInvalidPosition` must carry `#[non_exhaustive]`
   per CLAUDE.md pub-API discipline. Both are in `prism-core::error`.

7. **Add** `map_prism_error` arms for BOTH new error variants in `crates/prism-mcp/src/error_mapping.rs`.
   Each MUST use the symbolic constant `codes::INVALID_PARAMS` per repo convention — every existing
   arm in `error_mapping.rs` uses `codes::` symbolic constants, NOT bare integer literals:
   - `PrismError::TemporalLiteralUnparseable { .. }` → `codes::INVALID_PARAMS`
   - `PrismError::TemporalLiteralInvalidPosition { .. }` → `codes::INVALID_PARAMS`
   Both must NOT fall through to the catch-all `codes::INTERNAL_ERROR`.

8. **Change** `crates/prism-bin/src/spec_driven_adapter.rs` `column_type_to_arrow` function:
   ```rust
   // Before:
   ColumnType::Datetime => DataType::Utf8,
   // After:
   ColumnType::Datetime => DataType::Timestamp(
       TimeUnit::Microsecond,
       Some(Arc::from("UTC")),
   ),
   ```
   Verify `TimeUnit` and `Arc` are in scope. `Arc::from("UTC")` requires `std::sync::Arc`.

9. **Add sensor datetime string parsing** in `crates/prism-bin/src/spec_driven_adapter.rs` or
   `crates/prism-sensors/src/` — at the OCSF normalization boundary where sensor API response
   ISO-8601 datetime strings are converted to Arrow column values. Incoming datetime strings
   must be parsed via `chrono::DateTime::parse_from_rfc3339` → `.timestamp_micros()` → `i64`
   microseconds-since-epoch (the in-memory representation for `Timestamp(Microsecond, UTC)`).

   Extract the parsing helper as a `pub(super)` function for direct testability (SID-1):
   ```rust
   pub(super) fn parse_datetime_to_micros(s: &str) -> Result<i64, SpecEngineError> {
       chrono::DateTime::parse_from_rfc3339(s)
           .map(|dt| dt.timestamp_micros())
           .map_err(|_| SpecEngineError::NormalizationError { ... })
   }
   ```

10. **Fix** `crates/prism-core/src/column.rs` `ColumnType::Datetime` doc comment (blast item 2):
    ```rust
    // Before (actual text — must match exactly):
    /// Microsecond-precision UTC timestamp. Arrow: TimestampMicrosecond.
    // After (correct):
    /// Microsecond-precision UTC timestamp, normalized to UTC at the adapter boundary.
    /// Arrow: Timestamp(Microsecond, UTC-tagged). Stored and transmitted as RFC-3339.
    ```

11. **Change** `crates/prism-query/src/pipe_sql_emitter.rs` with TWO modifications:

    **Modification A — `Literal::Timestamp` rendering** (blast item 3 — arrow_cast form):
    ```rust
    // Before:
    Literal::Timestamp(ts) => format!("'{}'", ts.iso8601),
    // After:
    Literal::Timestamp(ts) => format!("arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')", ts.iso8601),
    ```
    Also update the stale comment at the Datetime/Utf8 reference nearby (blast item 9):
    replace any comment asserting "Datetime fields is DataType::Utf8" with the correct
    "Datetime fields are DataType::Timestamp(Microsecond, Some(\"UTC\")) per ADR-052."

    **Modification B — `Literal::RawTemporalLiteral` guard arm** (ADR-052 §D4 Step 5):
    Add a guard arm that returns E-QUERY-002 (`QueryPlanFailed`) if a `RawTemporalLiteral` node ever reaches
    SQL emission (belt-and-suspenders; under correct plan execution, `check_temporal_literals`
    always consumes `RawTemporalLiteral` nodes before the emitter is called):
    ```rust
    Literal::RawTemporalLiteral(_) => {
        return Err(PrismError::QueryPlanFailed {
            detail: "internal error — unvalidated RawTemporalLiteral reached SQL emission; check_temporal_literals must run before emission".to_string(),
        });
    }
    ```

12. **Add** `Literal::RawTemporalLiteral(String)` variant to `crates/prism-query/src/ast.rs`
    (ADR-052 §D4 Step 1 — blast item 13). Add the variant with the doc comment exactly as
    specified in §D4 above. This variant is:
    - `pub` (part of the `Literal` enum public surface)
    - Non-exhaustive is NOT individually added per variant — the enum itself has `#[non_exhaustive]`
      from the perimeter gate or must gain it if missing
    - The variant carries a single `String` field (the raw literal value)

    After adding the variant, the compiler will surface every `match` arm on `Literal` within
    `crates/prism-query/src/` that lacks a `RawTemporalLiteral` arm. DO NOT PROCEED until
    all compile errors are resolved (see Task 15 — sibling-site sweep).

13. **Modify parser lenient fallback** in `crates/prism-query/src/sql_parser.rs` (AND
    `filter_parser.rs` if the filter/pipe grammar is in a separate file — verify before
    editing):

    In the timestamp literal combinator (the function that handles quoted string literals in
    datetime/comparison contexts), replace the current fail-on-non-RFC3339 behavior with the
    lenient three-case dispatch described in §D4 Step 2:

    ```rust
    // Before (conceptually):
    // match chrono::DateTime::parse_from_rfc3339(&literal_str) {
    //     Ok(_) => Literal::Timestamp(...),
    //     Err(_) => return parse_error(...),  // E-QUERY-001
    // }

    // After (Option-A lenient fallback):
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&literal_str) {
        // Full RFC-3339 → Timestamp (unchanged behavior)
        Literal::Timestamp(TimestampLiteral { instant: dt.into(), iso8601: literal_str })
    } else if is_date_like(&literal_str) {
        // Date-only or offset-less → RawTemporalLiteral (parse SUCCEEDS, validated at plan time)
        Literal::RawTemporalLiteral(literal_str)
    } else {
        // Non-date-like strings → String (unchanged behavior)
        Literal::String(literal_str)
    }
    ```

    where `is_date_like` is a new helper function implementing the **full 7-form acceptance
    set** from ADR-052 §D4 (EXHAUSTIVE — transcribe exactly these 7 format strings):
    ```rust
    fn is_date_like(s: &str) -> bool {
        use chrono::{NaiveDate, NaiveDateTime};
        // Form 1: date-only
        NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
        // Form 2: T-separator, full seconds, no fractional
        || NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok()
        // Form 3: T-separator, fractional seconds (%.f matches .NNN including the dot)
        || NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
        // Form 4: T-separator, no seconds (hour:minute only)
        || NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").is_ok()
        // Form 5: space-separator, full seconds, no fractional
        || NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").is_ok()
        // Form 6: space-separator, fractional seconds
        || NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f").is_ok()
        // Form 7: space-separator, no seconds (hour:minute only)
        || NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").is_ok()
    }
    ```

    IMPORTANT: `is_date_like` operates on `&str` (already-parsed UTF-8 strings from the
    tokenizer). There are NO raw byte-offset operations. Unicode safety is guaranteed by
    construction (the String is valid UTF-8 from the parser tokenizer).

    **Pipe-mode `stats by` and `sort` parse-time rejection (ADR-052 §D4 v1.10):**

    In `filter_parser.rs`, the grammar rules for `stats … by` and `sort` clauses must REJECT
    bare literal values (quoted strings, including any value that would match `is_date_like`)
    in the `by`/`sort` position at PARSE TIME, before the AST is produced:

    - `FROM t | stats count by '2026-06-24'` → parse-time E-QUERY-001 with enhanced message:
      "`stats by` expects a field path, not a literal value (e.g., `stats count by ts_col`)".
    - `FROM t | sort '2026-06-24'` → parse-time E-QUERY-001 with enhanced message:
      "`sort` expects a field path, not a literal value (e.g., `sort ts_col`)".

    This rejection happens BEFORE `check_temporal_literals` is called. In SQL mode,
    `GROUP BY` and `ORDER BY` with bare literals produce AST nodes that `check_temporal_literals`
    rejects with E-QUERY-042 (plan time). In pipe mode, the parser itself rejects literals
    in these positions with enhanced E-QUERY-001 (parse time). The distinction matters for
    error-code accuracy: pipe rejects at E-QUERY-001 (parse/syntax), SQL rejects at E-QUERY-042
    (plan/semantic).

14. **Implement `check_temporal_literals` AST walker** in `crates/prism-query/src/engine.rs`
    (ADR-052 §D4 Step 3):

    **Function signature:**
    ```rust
    fn check_temporal_literals(ast: &mut Ast, schema: &PrismSchema) -> Result<(), PrismError>
    ```
    (Note: `&mut Ast` because the coercion arm rewrites `RawTemporalLiteral` → `Literal::String`
    in-place. If the AST is immutable, produce a new AST via structural transformation instead.)

    **Call site:** Insert `check_temporal_literals(&mut ast, &schema)?;` AFTER
    `check_enrich_udf_availability(...)?;` and BEFORE `build_session_context` / the DataFusion
    planning call. Gate ordering:
    ```
    E-QUERY-001 (parse) → E-QUERY-037 (check_table_availability) →
    E-QUERY-038 (check_query_column_availability) →
    E-QUERY-039 (check_enrich_udf_availability) →
    check_temporal_literals → E-QUERY-041 →
    DataFusion execution
    ```

    **Implementation logic — four-way dispatch on `Literal::RawTemporalLiteral`:**

    Walk the full `Expr` tree recursively. For each `Expr::Literal(Literal::RawTemporalLiteral(s))`
    (ADR-052 §D4 v1.10 refined dispatch):

    1. Determine position context:
       a. If position is a **comparison** (`>`, `<`, `>=`, `<=`, `=`, `!=`):
          - If LHS is a **plain column reference**: resolve the column type (step 2)
          - If LHS is a **function call or other non-column expression** (non-column-LHS):
            → `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::NonColumnLhsComparison, value_prefix: first_50_chars(s) })` (E-QUERY-042; NOT E-QUERY-041)
       b. If position is **GROUP BY** (SQL mode):
          → `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::GroupBy, value_prefix: first_50_chars(s) })` (E-QUERY-042)
       c. If position is **ORDER BY** (SQL mode):
          → `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::OrderBy, value_prefix: first_50_chars(s) })` (E-QUERY-042)
       d. If position is **SELECT projection** or **function arg** (non-comparison, not GROUP BY/ORDER BY):
          COERCE — rewrite node: `*literal = Literal::String(s.clone())` → Ok(()) (ADR-052 §D4 v1.10)
    2. Resolve the column type from `schema` for the comparison operand (step 1a plain-column path only):
       - If `DataType::Timestamp(Microsecond, Some("UTC"))` → `Err(PrismError::TemporalLiteralUnparseable { value_prefix: first_50_chars(s) })` (E-QUERY-041)
       - If `DataType::Utf8` → rewrite node: `*literal = Literal::String(s.clone())` → Ok(()) (coerce)
       - If Integer / Float / Bool → `Err(PrismError::QueryTypeMismatch { column: column_name.to_string(), table: table_name.to_string(), actual_type: ct, operator: op.to_string() })` (E-QUERY-002; use `QueryTypeMismatch` per the established E-QUERY-002 taxonomy)
    3. `first_50_chars(s)` helper: truncates `s` at the last valid UTF-8 codepoint boundary
       at or before position 50 (DO NOT slice raw bytes; use `s.char_indices().take_while(|(i, _)| *i <= 50).last()` or equivalent).

    Note: Pipe-mode `stats … by` and `sort` bare literals are REJECTED AT PARSE TIME by
    `filter_parser.rs` with enhanced E-QUERY-001 messages (Task 13). They do NOT appear as
    `RawTemporalLiteral` nodes in this AST walker — the parser has already rejected the query.

    **Dotted/qualified column resolution:** Use the same schema-resolution path as
    `check_query_column_availability` (E-QUERY-038). Do NOT attempt text-based column name
    extraction. The schema contains the full resolved column type for dotted expressions.

    **DELETE TEXT-SCANNER FUNCTIONS:** In the same commit, remove the following functions
    from `engine.rs` (if they exist in the current workspace from a prior implementation attempt):
    - `extract_table_name_from_query_str`
    - `extract_column_name_adjacent_to_quoted_value`
    - `is_bad_literal_in_datetime_column`
    - Any parse-fail recovery branch that calls the above

    If these functions do NOT exist in the current workspace (i.e., the workspace is still
    at the v1.2 spec state with only `check_temporal_literals` as a chrono pre-validator),
    do NOT attempt to add and then delete them — simply implement the Option-A mechanism
    and skip the deletion step. Document the workspace state in the PR description.

15. **TD-VSDD-060 Literal sibling-sweep** (ADR-052 §Risk RISK-4 mitigation):

    Run:
    ```bash
    grep -rn 'Literal::' crates/prism-query/src/ --include="*.rs"
    ```

    For every `match` arm on `Literal` within `crates/prism-query/src/` (NOT external crates
    which already have `_ => {}` wildcard arms from the `#[non_exhaustive]` perimeter gate):
    - Add a `Literal::RawTemporalLiteral(_)` arm with appropriate semantics
    - For the `pipe_sql_emitter.rs` arm: must return E-QUERY-002 (`QueryPlanFailed`) (already done in Task 11B)
    - For `pushdown.rs` arms: `RawTemporalLiteral` must NOT be pushed down as a time predicate
      — either return `None` (no pushdown) or propagate E-QUERY-002 (`QueryPlanFailed`)
    - For any `Display`/`Debug` impl for `Literal`: add `RawTemporalLiteral(s) => write!(f, "RawTemporalLiteral({:?})", s)`
    - For any `PartialEq`/`Hash`/`Clone` derive: these apply automatically if `String: PartialEq + Hash + Clone` — verify no manual impls need updating

    Missing internal arms produce compile errors if the match is exhaustive (no wildcard).
    Some internal matches may have a wildcard that silently falls through — those are the
    dangerous ones. Verify each `_ => {}` wildcard arm is intentional for `RawTemporalLiteral`.

    The coercion arm in `check_temporal_literals` (Task 14) does NOT add new `match` sites
    on `Literal::RawTemporalLiteral` beyond what the AST walker already requires — the TD-VSDD-060
    scope is the existing `Literal::` match sites enumerated by this grep.

16. **Update** `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`:
    Update ALL existing assertions in this file that expect `DataType::Utf8` for datetime
    column schemas to instead assert `DataType::Timestamp(Microsecond, Some(Arc::from("UTC")))`.

17. **VERIFY** `crates/prism-query/src/pushdown.rs` — confirm the T1 extractor uses
    `Literal::Timestamp(ts) => ts.instant.to_rfc3339()` (or equivalent chrono `to_rfc3339` call).
    This path operates on the Prism AST layer, NOT on Arrow DataType — it is unaffected by D2.
    No change required (D5 explicit no-change statement).

18. **VERIFY** `crates/prism-query/src/infusion_udf.rs` — confirm `return_type` returns
    `DataType::Utf8` UNCONDITIONALLY (the function is a stub with no per-output_type mapping).
    Do NOT change this file in this story. Add a comment: `// ADR-052: sensor datetime
    columns → Timestamp(Microsecond, Some("UTC")) (ADR-052). ADR-051 (not yet implemented)
    will add a per-output_type branch here to bring enrichment datetime fields to the same type.`

19. **VERIFY** `crates/prism-spec-engine/src/infusion/udf.rs` — confirm `InfusionUdfDescriptor.output_type`
    is a `String`; no Arrow-level change occurs at spec-engine level. No change required.

20. **VERIFY** `specs/infusions/*.infusion.toml` — confirm `output_type = "datetime"` TOML string
    schema is unchanged. No change required.

21. **INVESTIGATE** `diff_results` CF Arrow IPC compatibility (D6 / RISK-3):
    ```bash
    grep -rn "diff_results\|DIFF_RESULTS" crates/ --include="*.rs" \
      | grep -i "ipc\|recordbatch\|arrow"
    ```
    Expected: no matches. If matches are found: investigate and add startup migration step.
    Document investigation result in the PR description (required regardless of outcome).

22. **SWEEP** remaining files — workspace grep for DataType::Utf8 datetime assertions
    (blast item 20):
    ```bash
    grep -rn 'DataType::Utf8' crates/prism-query/src/ --include="*.rs"
    ```
    For each hit that refers to a datetime column, update to `Timestamp(Microsecond, Some(Arc::from("UTC")))`.

23. **VERIFY** ADR-044 factory spec (blast item 10 — architect pre-completed):
    Confirm `.factory/specs/architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md`
    already contains frontmatter `superseded_by:` referencing `ADR-052` AND a
    `## Status` section containing `PARTIALLY SUPERSEDED by ADR-052`.
    Do NOT edit ADR-044. If either grep returns no output, escalate to the architect.

24. **SAP-1 check** — if any `tracing::*!(event_type = "...")` was added at E-QUERY-041
    detection time, a corresponding row MUST be added to BC-2.16.002 §Postconditions
    Canonical Structured Event Catalog in the SAME commit. If no new tracing emission
    was added, no action needed.

25. **Run** `cargo nextest run -p prism-query -p prism-bin -p prism-core -p prism-mcp \
    -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING)' --no-fail-fast` to verify all 39 Red Gate
    tests pass GREEN.

26. **Run** `just check` once to verify AC-016 (exit 0). Per CLAUDE.md TDD inner loop
    discipline, reserve `just check` for the final gate verification.

## Previous Story Intelligence

This is the first story in the `S-PRISMQL-NATIVE-TEMPORAL-TYPING-*` series.

**Relevant patterns from adjacent merged stories:**

- `S-DEMO-FIDELITY-REMEDIATION-001` (PR #208 — merged develop@122228e8): The three
  separate gate functions in `engine.rs` — `check_table_availability` (E-QUERY-037),
  `check_query_column_availability` (E-QUERY-038), `check_enrich_udf_availability`
  (E-QUERY-039) — are the model for the new `check_temporal_literals` gate (Task 14).
  The pre-validator is inserted after `check_enrich_udf_availability(...)?;` and before
  `build_session_context`, following the same sequential-gate architecture.
  `check_query_column_availability` also uses schema-resolved column types — use the same
  schema-lookup pattern for `check_temporal_literals` dotted/qualified column resolution.

- `S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001` (PR #203): The `inject_now` /
  `inject_now_sql_query` / `inject_now_pipe_stage` functions in `prism-query/src/lib.rs`
  perform planning-time `Expr::Now` → `Literal::Timestamp` substitution. These feed
  into the emitter's `Literal::Timestamp` arm. These functions are NOT modified in
  this story — only the emitter rendering changes.

- `S-PERF-GATE-008` (PR #213 — merged): Demonstrates correct `pub(super)` helper
  extraction pattern for testability (SID-1). The `parse_datetime_to_micros` helper
  in Task 9 follows this same extraction pattern.

- **TD-VSDD-060 sibling-site sweep (Task 15):** When changing `column_type_to_arrow` for
  `Datetime` AND adding a new `Literal::RawTemporalLiteral` variant, grep ALL callsites of
  `column_type_to_arrow` and ALL match arms on `Literal::` in `crates/prism-query/src/`.

**Option-A mechanism lessons (from 8 text-scanner fix-burst failures):**

- The text-scanner failed because it operated on raw query strings without schema resolution.
  Option-A operates on the fully-parsed AST with the schema resolved. Never attempt to extract
  column names or source names via string-scanning raw query input.
- The `first_50_chars` truncation helper MUST operate on codepoint boundaries using Rust's
  `char_indices()`, NOT on raw byte indices. Raw byte slicing on UTF-8 strings produces panics
  on multi-byte characters.
- `check_temporal_literals` must run BEFORE DataFusion's `build_session_context` — after
  schema resolution, but before query execution. The gate ordering is non-negotiable.

## Architecture Compliance Rules

Derived from ADR-052 and `.factory/specs/architecture/` section files:

| Rule | Constraint |
|------|------------|
| **Arrow timestamp type canonical form** | ALWAYS use `DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))`. NEVER `Timestamp(Nanosecond, ...)`, NEVER `Timestamp(Microsecond, None)` (untagged), NEVER `Some(Arc::new("UTC".into()))` (wrong `Arc<String>` type). |
| **Emitter arrow_cast form** | `pipe_sql_emitter.rs` MUST use `arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')` form. `TIMESTAMP '...'` is FORBIDDEN — DataFusion 53.1.0 lowers it to `Timestamp(Nanosecond, None)`. |
| **E-QUERY-041 detection is Option-A AST walker** | E-QUERY-041 is raised by `check_temporal_literals` walking the resolved AST. NO text-scanning of raw query strings. NO chrono pre-validator on `Literal::String` string literals. NO DataFusion cast-failure intercept. |
| **RawTemporalLiteral must not reach emission** | `pipe_sql_emitter.rs` MUST have a `Literal::RawTemporalLiteral` guard arm returning E-QUERY-002 (`QueryPlanFailed`). This node is an implementation artifact of plan time; it must be consumed by `check_temporal_literals` before emission. |
| **Coercion arm preserves pre-ADR-052 behavior** | `RawTemporalLiteral` against String/Utf8 column → rewrite in-place to `Literal::String(s)` → no error. Byte-identical to pre-ADR-052 behavior. RISK-5 eliminated by design. |
| **Pushdown boundary contract** | `pushdown.rs` T1 extractor (`Literal::Timestamp.instant.to_rfc3339()`) produces RFC-3339 strings for sensor APIs — UNCHANGED. Do NOT modify pushdown.rs. |
| **Unicode safety** | `first_50_chars(s)` truncation MUST use Rust `char_indices()`/codepoint-safe slicing. Never use raw byte indices on `String`/`&str` in this path. |
| **Structured event catalog (SAP-1)** | Any new `tracing::*!(event_type = "...")` at E-QUERY-041 detection time requires a BC-2.16.002 catalog row in the same commit. |
| **Error taxonomy discipline** | E-QUERY-041 Display MUST match error-taxonomy.md §E-QUERY-041 POL-24 template byte-for-byte. `value_prefix` is 50 chars max, UTF-8 boundary safe. |
| **E-QUERY-042 GROUP BY / ORDER BY / non-column-LHS must REJECT** | `check_temporal_literals` MUST return `Err(PrismError::TemporalLiteralInvalidPosition)` for GROUP BY and ORDER BY positions and for non-column-LHS comparisons. NEVER COERCE these positions — they are semantic errors, not benign type ambiguities. Maps to `codes::INVALID_PARAMS`, NOT `-32000`. |
| **Pipe stats-by / sort must REJECT at parse time** | `filter_parser.rs` must reject bare literals in `stats by` and `sort` positions with enhanced E-QUERY-001. These must NOT produce `RawTemporalLiteral` AST nodes. |
| **SELECT projection coerce is PRESERVED** | `RawTemporalLiteral` in SELECT projection or function arg COERCES to `Literal::String` (ADR-052 §D4 v1.10). This is unchanged from v1.7. The SELECT projection arm and the GROUP BY/ORDER BY reject arms are DIFFERENT code paths. |
| **Forbidden dependencies** | No new crate dependencies added to `prism-query` or `prism-core`. `chrono` is already a dependency; `Arc::from` is in `std::sync`. |
| **ADR-051 sequencing gate (D8)** | `infusion_udf.rs` `return_type` returns `DataType::Utf8` UNCONDITIONALLY after this story — ADR-051 will INTRODUCE the datetime→Timestamp mapping. Do not change `infusion_udf.rs` in this story. |

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `datafusion` | pinned in `[workspace.dependencies]` Cargo.toml | existing workspace pin |
| `arrow` / `arrow-schema` | pinned in workspace | existing workspace pin |
| `chrono` | pinned in workspace | `DateTime::parse_from_rfc3339` + `.timestamp_micros()` + `NaiveDate::parse_from_str` + `NaiveDateTime::parse_from_str` |

**Forbidden new dependencies**: No new `[dependencies]` entries in any `Cargo.toml`.
All required types (`TimeUnit`, `DataType`, `Arc`, `chrono::DateTime`, `chrono::NaiveDate`,
`chrono::NaiveDateTime`) are already available in the existing dependency graph.

## File Structure Requirements

### Files to CREATE:
None — no new files.

### Files to MODIFY:

| File | Change Type | What Changes |
|------|-------------|--------------|
| `crates/prism-core/src/error.rs` | [CHANGE] | Add `PrismError::TemporalLiteralUnparseable { value_prefix: String }` (E-QUERY-041) and `PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition, value_prefix: String }` (E-QUERY-042) variants + `TemporalInvalidPosition` enum (GroupBy/OrderBy/NonColumnLhsComparison, `#[non_exhaustive]`); add `Display` arms per error-taxonomy §E-QUERY-041/§E-QUERY-042; verify `PrismError` carries `#[non_exhaustive]` |
| `crates/prism-bin/src/spec_driven_adapter.rs` | [CHANGE] | `column_type_to_arrow`: `Datetime → DataType::Utf8` → `Timestamp(Microsecond, Some(Arc::from("UTC")))`; add `parse_datetime_to_micros` helper + sensor datetime string → microsecond parsing |
| `crates/prism-core/src/column.rs` | [CHANGE] | Fix stale doc comment on `ColumnType::Datetime` |
| `crates/prism-query/src/ast.rs` | [CHANGE] | Add `Literal::RawTemporalLiteral(String)` variant with doc comment per §D4 Step 1 |
| `crates/prism-query/src/sql_parser.rs` | [CHANGE] | Lenient fallback in timestamp literal combinator: RFC-3339→Timestamp; is_date_like→RawTemporalLiteral; else→Utf8; add `is_date_like` helper |
| `crates/prism-query/src/filter_parser.rs` | [CHANGE if exists] | Same lenient fallback as sql_parser.rs if pipe/filter grammar is in a separate file |
| `crates/prism-query/src/engine.rs` | [CHANGE] | Add `check_temporal_literals` AST walker (four-way dispatch + coercion); call after `check_enrich_udf_availability(...)?;`; DELETE `extract_table_name_from_query_str`, `extract_column_name_adjacent_to_quoted_value`, `is_bad_literal_in_datetime_column` (if present) |
| `crates/prism-query/src/pipe_sql_emitter.rs` | [CHANGE] | `Literal::Timestamp`: `arrow_cast(...)` form; add `Literal::RawTemporalLiteral` guard arm → E-QUERY-002 (`QueryPlanFailed`); update stale Utf8 comment (blast items 3, 9, +D4 guard) |
| `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` | [CHANGE] | Update `DataType::Utf8` datetime assertions to `Timestamp(Microsecond, Some(Arc::from("UTC")))`; add RISK-1 arrow_cast probe (RG-002), Timestamp type assertion (RG-009), emitter E2E test (RG-010) |
| `crates/prism-query/src/` — all internal `Literal::` match arms | [CHANGE] | TD-VSDD-060 sibling-sweep: add `Literal::RawTemporalLiteral` arm to every internal exhaustive match |
| `crates/prism-sensors/src/` (normalization paths) | [CHANGE] | Add ISO-8601 string → `i64` microseconds-since-epoch parsing at OCSF datetime normalization boundary |
| `crates/prism-mcp/src/error_mapping.rs` | [CHANGE] | Add explicit `codes::INVALID_PARAMS` arms for `PrismError::TemporalLiteralUnparseable` and `PrismError::TemporalLiteralInvalidPosition`; both must NOT fall through to `codes::INTERNAL_ERROR` |

### Files to VERIFY (no change expected):

| File | What to Verify |
|------|---------------|
| `crates/prism-query/src/pushdown.rs` | `Literal::Timestamp` arm uses `ts.instant.to_rfc3339()` — no change; also verify `Literal::RawTemporalLiteral` arm added (Task 15) returns None/no-pushdown |
| `crates/prism-query/src/infusion_udf.rs` | `return_type` returns `DataType::Utf8` UNCONDITIONALLY (stub); add ADR-052/ADR-051 comment |
| `crates/prism-spec-engine/src/infusion/udf.rs` | `InfusionUdfDescriptor.output_type` is `String` — no Arrow-level change |
| `specs/infusions/*.infusion.toml` | `output_type = "datetime"` is an opaque string — no TOML schema change |
| `.factory/specs/architecture/decisions/ADR-044-*.md` | Confirm `superseded_by:` references ADR-052 AND §Status contains "PARTIALLY SUPERSEDED by ADR-052" |

## Acceptance Criteria

### AC-001 — `ColumnType::Datetime` registers as `DataType::Timestamp(Microsecond, Some("UTC"))` in `column_type_to_arrow` (ADR-052 D1/D2, blast item 1)

```bash
grep -A1 'ColumnType::Datetime' crates/prism-bin/src/spec_driven_adapter.rs \
  | grep 'Timestamp.*Microsecond'
```

Expected: at least one match — the `DataType::Timestamp(TimeUnit::Microsecond, ...)` arm.

Verify the old `DataType::Utf8` arm for Datetime is absent:

```bash
grep -n 'Datetime.*Utf8\|Utf8.*Datetime' crates/prism-bin/src/spec_driven_adapter.rs
```

Expected output: no matches.

Verify the canonical `Arc::from` form is used (not `Arc::new("UTC".into())`):

```bash
grep -n 'Arc::new.*UTC.*into\(\)' crates/prism-bin/src/spec_driven_adapter.rs
```

Expected output: no matches.

Traces to: BC-2.11.003 §Postconditions (ADR-052 D2); BC-2.11.004 §Postconditions; ADR-052 D1 canonical form `Arc::from("UTC")`.

### AC-002 — RISK-1: DataFusion probe test confirms `arrow_cast(...)` literal produces `Timestamp(Microsecond, Some("UTC"))` in plan output (ADR-052 RISK-1 mandatory mitigation)

```bash
grep -c 'test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_risk1_datafusion_arrow_cast_probe' \
  crates/prism-query/src/tests/high002_plan_pinning_tests.rs
```

Expected output: `1`.

This test (RG-002):
- Registers a `Timestamp(Microsecond, Some(Arc::from("UTC")))` column in a DataFusion `SessionContext`
- Plans `SELECT * FROM t WHERE ts > arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')`
- Verifies the plan is produced without error
- Verifies the literal in the plan is typed `Timestamp(Microsecond, Some("UTC"))` — matching the column's type exactly, with no implicit cast node introduced

Traces to: BC-2.11.021 §Postconditions; ADR-052 §Risk RISK-1 mitigation.

### AC-003 — `pipe_sql_emitter.rs` `Literal::Timestamp` rendering emits `arrow_cast(...)` form (ADR-052 D3 v1.3, blast item 3)

```bash
grep -c "arrow_cast" crates/prism-query/src/pipe_sql_emitter.rs
```

Expected output: at least `1`.

Verify the `TIMESTAMP '...'` form is absent:

```bash
grep -n "TIMESTAMP '{}'" crates/prism-query/src/pipe_sql_emitter.rs
```

Expected output: no matches.

Verify the arrow_cast type string contains the canonical type:

```bash
grep -n "Timestamp(Microsecond, Some" crates/prism-query/src/pipe_sql_emitter.rs
```

Expected: at least one match in the `Literal::Timestamp` arm.

Traces to: BC-2.11.021 §Postconditions; ADR-052 D3.

### AC-004 — `NOW() - INTERVAL '24h'` lowers to an `arrow_cast(...)` typed literal comparison; `inject_now` path unbroken (ADR-052 D3/D7, BC-2.11.021)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sql_emitter_yields_arrow_cast_literal)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

Additionally, run the existing `inject_now` tests to confirm the planning-time constant injection pipeline is unbroken:

```bash
cargo nextest run -p prism-query -E 'test(inject_now)' --no-fail-fast 2>&1 | tail -5
```

Expected: all `inject_now`-related tests pass (zero FAIL).

Traces to: BC-2.11.021 §Postconditions ("Planning-time constant injection" bullet);
BC-2.11.021 §Invariants ("duration arithmetic is subtraction-only in v1").

### AC-005 — E-QUERY-041 fires at plan time (via `check_temporal_literals` Option-A AST walker) for date-only or offset-less string literals in Datetime column comparisons (ADR-052 §D4 Option-A; BC-2.11.021, BC-2.11.003, BC-2.11.004)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_sql_mode_date_only_string)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_pipe_mode_date_only_string)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

These tests verify:
- SQL mode: `SELECT * FROM t WHERE timestamp > '2026-06-24'` (date-only) →
  `PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24".into() }`
  raised by `check_temporal_literals` at plan time (NOT at parse time — parse SUCCEEDS
  and produces `Literal::RawTemporalLiteral("2026-06-24")` in the AST)
- Pipe mode: `FROM t | where timestamp > '2026-06-24'` → same error via same mechanism
- The error fires via the Option-A AST walker, NOT via `chrono::DateTime::parse_from_rfc3339`
  on string literals, NOT via a DataFusion cast-failure intercept, NOT via text-scanning

Verify the `PrismError::TemporalLiteralUnparseable` variant exists:

```bash
grep -c 'TemporalLiteralUnparseable' crates/prism-core/src/error.rs
```

Expected output: at least `2` (variant definition + Display arm).

Traces to: BC-2.11.021 §Error Cases E-QUERY-041 (Option-A AST walker mechanism);
BC-2.11.003 §Error Cases E-QUERY-041; BC-2.11.004 §Error Cases E-QUERY-041;
error-taxonomy.md §E-QUERY-041 three-way dispatch.

### AC-006 — `PrismError::TemporalLiteralUnparseable` maps to `codes::INVALID_PARAMS` via `map_prism_error` in `error_mapping.rs` (error-taxonomy §E-QUERY-041 `map_prism_error` constraint)

```bash
cargo nextest run \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_map_prism_error_invalid_params)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

Verify the explicit arm exists in `error_mapping.rs` using the symbolic constant:

```bash
grep -n 'TemporalLiteralUnparseable' crates/prism-mcp/src/error_mapping.rs
```

Expected: at least one match in the `map_prism_error` function body using `codes::INVALID_PARAMS`.
Must NOT fall through to catch-all `codes::INTERNAL_ERROR`.

Traces to: BC-2.11.001 §Postconditions (`map_prism_error` MCP error code contract); error-taxonomy.md §E-QUERY-041 `map_prism_error` constraint.

### AC-007 — Valid RFC-3339 UTC string literal `'2026-06-24T00:00:00Z'` in a Datetime comparison is NOT rejected (ADR-052 §D4 — RFC-3339 → Literal::Timestamp, check_temporal_literals not invoked)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_valid_rfc3339_utc_string_not_rejected)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

This test verifies:
- `SELECT * FROM t WHERE timestamp > '2026-06-24T00:00:00Z'` passes the parser as
  `Literal::Timestamp` (NOT `RawTemporalLiteral` — full RFC-3339 parse succeeds)
- `check_temporal_literals` encounters no `RawTemporalLiteral` nodes → does not fire
- Query proceeds to DataFusion execution successfully
- E-QUERY-041 is NOT raised for valid RFC-3339 inputs

Both `Z` and `+00:00` UTC offset forms are accepted by `chrono::DateTime::parse_from_rfc3339`.

Traces to: BC-2.11.003 §Edge Cases EC-11-003-002; BC-2.11.004 §Edge Cases EC-11-004-002.

### AC-008 — Pushdown boundary UNCHANGED: `pushdown.rs` T1 extractor produces RFC-3339 strings for sensor APIs (ADR-052 D5 explicit no-change statement)

```bash
grep -n 'to_rfc3339\|rfc_3339' crates/prism-query/src/pushdown.rs | head -5
```

Expected: at least one match showing `ts.instant.to_rfc3339()` or equivalent.

Verify no change was made to pushdown.rs:

```bash
git diff HEAD -- crates/prism-query/src/pushdown.rs
```

Expected: empty diff.

Traces to: BC-2.11.021 §Postconditions ("ADR-033 push-down benefit — no changes to `pushdown.rs` required").

### AC-009 — `diff_results` CF Arrow IPC compatibility confirmed (ADR-052 D6 / RISK-3)

```bash
grep -rn "diff_results\|DIFF_RESULTS" crates/ --include="*.rs" \
  | grep -i "ipc\|recordbatch\|arrow"
```

Expected output: no matches.

If matches ARE found: investigate and add startup migration step. Document investigation
result in the PR description.

Traces to: ADR-052 §D6; §Risk RISK-3.

### AC-010 — `column.rs` `ColumnType::Datetime` doc comment updated (ADR-052 D2, blast item 2)

```bash
grep -n 'TimestampMicrosecond' crates/prism-core/src/column.rs
```

Expected output: no matches (stale form replaced).

```bash
grep -n 'Timestamp(Microsecond, UTC-tagged)' crates/prism-core/src/column.rs
```

Expected: at least one match for the corrected doc comment.

Traces to: ADR-052 D2.

### AC-011 — Stale `pipe_sql_emitter.rs` "Datetime fields is DataType::Utf8" comment updated (ADR-052 blast item 9)

```bash
grep -n 'Datetime.*Utf8\|DataType::Utf8.*datetime\|DataType::Utf8.*Datetime' \
  crates/prism-query/src/pipe_sql_emitter.rs
```

Expected output: no matches.

Traces to: ADR-052 §Blast Radius item 9.

### AC-012 — `high002_plan_pinning_tests.rs` datetime column assertions updated from `DataType::Utf8` to `DataType::Timestamp(Microsecond, Some("UTC"))` (ADR-052 blast item 4)

```bash
cargo nextest run -p prism-query -E 'test(high002)' --no-fail-fast 2>&1 | tail -5
```

Expected: all `high002_*` tests pass GREEN.

```bash
grep -n 'Utf8' crates/prism-query/src/tests/high002_plan_pinning_tests.rs \
  | grep -iv 'sensor_id\|client_id\|string\|text\|varchar'
```

Expected output: no matches for `DataType::Utf8` on datetime-typed fields.

Traces to: ADR-052 §Blast Radius item 4; BC-2.11.003 §Postconditions.

### AC-013 — Sensor datetime string → `i64` microseconds-since-epoch conversion added at OCSF normalization boundary (ADR-052 D5)

```bash
grep -rn 'timestamp_micros\|parse_from_rfc3339' \
  crates/prism-bin/src/spec_driven_adapter.rs crates/prism-sensors/src/ \
  --include="*.rs" 2>/dev/null | head -10
```

Expected: at least one match showing `chrono::DateTime::parse_from_rfc3339` → `.timestamp_micros()` conversion.

```bash
cargo nextest run \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sensor_datetime_string_parsed_to_micros)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

Traces to: ADR-052 D5 "Sensor timestamp parsing addition"; chrono-strictness invariant.

### AC-014 — Workspace grep confirms no remaining `DataType::Utf8` hardcoded assertions for datetime columns in `crates/prism-query/src/` (ADR-052 blast item 20)

```bash
grep -rn 'DataType::Utf8' crates/prism-query/src/ --include="*.rs" \
  | grep -iv 'sensor_id\|client_id\|org_slug\|string\|varchar\|text\|severity\|status\|class_uid\|category_uid'
```

Expected output: no matches. Document grep output in the PR description.

Traces to: ADR-052 §Blast Radius item 20.

### AC-015 — ADR-044 already contains partial supersession scope in frontmatter and Status section (ADR-052 D7, blast item 10 — architect pre-completed; VERIFY ONLY)

```bash
grep -n 'superseded_by' \
  .factory/specs/architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md
```

Expected: at least one match showing `superseded_by:` referencing `ADR-052`.

```bash
grep -n 'PARTIALLY SUPERSEDED' \
  .factory/specs/architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md
```

Expected: at least one match. If EITHER grep returns no output: escalate to architect (do NOT edit ADR-044).

Traces to: ADR-052 §D7; §Blast Radius item 10.

### AC-016 — `just check` exits 0 with all changes applied (BC-5.39.001 delivery quality gate)

```bash
just check
echo "Exit: $?"
```

Expected output: `Exit: 0`.

Traces to: BC-5.39.001 §Postconditions.

### AC-017 — `Literal::RawTemporalLiteral(String)` variant exists in `ast.rs` with §D4 doc comment (ADR-052 §D4 Step 1)

```bash
grep -c 'RawTemporalLiteral' crates/prism-query/src/ast.rs
```

Expected output: at least `2` (variant definition line + doc comment line).

```bash
grep -n 'RawTemporalLiteral\|unvalidated temporal' crates/prism-query/src/ast.rs
```

Expected: shows the variant definition with its doc comment mentioning "date or datetime but is NOT valid RFC-3339" and "Must never reach SQL emission."

Traces to: ADR-052 §D4 Step 1; BC-2.11.021 §E-QUERY-041 detection mechanism.

### AC-018 — Parser emits `Literal::RawTemporalLiteral` for date-only and offset-less forms; parse SUCCEEDS, NOT E-QUERY-001 (ADR-052 §D4 Step 2)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_parser_emits_raw_temporal_for_date_only)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

This test verifies:
- Parsing a PrismQL query containing `'2026-06-24'` (date-only) SUCCEEDS and produces an AST
- The AST contains `Literal::RawTemporalLiteral("2026-06-24")` (NOT `Literal::Timestamp`, NOT parse error)
- Parsing a query containing `'2026-06-24T12:00:00'` (offset-less ISO) also SUCCEEDS with `RawTemporalLiteral`
- E-QUERY-001 is NOT returned at parse time for these inputs

Verify the lenient fallback is in the parser source:

```bash
grep -n 'RawTemporalLiteral\|is_date_like' crates/prism-query/src/sql_parser.rs
```

Expected: at least one match for each symbol.

Traces to: ADR-052 §D4 Step 2; BC-2.11.021 §E-QUERY-041 detection (parser lenient fallback).

### AC-019 — String/Utf8 column coercion: date-like literal vs String col → COERCE success, byte-identical emitted SQL (ADR-052 RISK-5 RESOLVED BY DESIGN)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_date_only_succeeds)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

This test verifies:
- `SELECT * FROM t WHERE string_col = '2026-06-24'` (String/Utf8 column, date-like literal)
  SUCCEEDS without error — no E-QUERY-041, no E-QUERY-001
- `check_temporal_literals` rewrites `RawTemporalLiteral("2026-06-24")` → `Literal::String("2026-06-24")`
- The emitted SQL comparison is `string_col = '2026-06-24'` — byte-identical to pre-ADR-052 behavior
- Partition keys, report-date labels, and ISO-date-formatted external IDs continue to work

Traces to: BC-2.11.003 §Postconditions (coercion success path); ADR-052 §D4 coercion arm (RISK-5 RESOLVED BY DESIGN).

### AC-020 — Integer/Float/Bool column type mismatch: date-like literal vs numeric/bool column → E-QUERY-002 (QueryTypeMismatch) (ADR-052 §D4 Step 3, three-way dispatch third arm)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_integer_col_date_like_e_query_001)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1` (also run float and bool variants via stubs p and q).

This test verifies `'2026-06-24'` (date-like, RawTemporalLiteral) compared against Integer,
Float, and Bool columns each return `E-QUERY-002` (`QueryTypeMismatch`) (not E-QUERY-041, not E-QUERY-038).

Traces to: ADR-052 §D4 Step 3; error-taxonomy §E-QUERY-041 three-way dispatch (Integer/Float/Bool arm → E-QUERY-002 `QueryTypeMismatch`; F-P5-MED-2 correction).

### AC-021 — Non-date-like strings remain `Literal::String` after parsing; no temporal error (ADR-052 §D4 Step 2 `is_date_like` negative case)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_non_date_like_stays_string_literal)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

This test verifies:
- `'not-a-date'` and `'sensor-id-abc'` produce `Literal::String` from the parser (NOT `RawTemporalLiteral`)
- `check_temporal_literals` does not fire (no `RawTemporalLiteral` nodes in AST)
- No E-QUERY-041 or E-QUERY-001 related to temporal validation

Traces to: ADR-052 §D4 Step 2 ("heuristic DOES NOT match arbitrary strings — `'not-a-date'`, `'sensor-id-abc'` remain `Literal::String`").

### AC-022 — `pipe_sql_emitter.rs` guard arm: `Literal::RawTemporalLiteral` reaching emission → E-QUERY-002 (QueryPlanFailed) (ADR-052 §D4 Step 5)

```bash
grep -c 'RawTemporalLiteral' crates/prism-query/src/pipe_sql_emitter.rs
```

Expected output: at least `1` (the guard arm).

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_guard_raw_temporal_literal)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. This test directly calls the `RawTemporalLiteral` arm in the emitter
and asserts it returns E-QUERY-002 (`QueryPlanFailed`) (not panic, not string output).

Traces to: ADR-052 §D4 Step 5 ("belt-and-suspenders defensive check").

### AC-023 — Text-scanner functions ABSENT from `engine.rs` (ADR-052 §D4 Step 4 deletion; VERIFY ABSENCE)

```bash
grep -n 'extract_table_name_from_query_str\|extract_column_name_adjacent_to_quoted_value\|is_bad_literal_in_datetime_column' \
  crates/prism-query/src/engine.rs
```

Expected output: no matches. These text-scanner functions must not exist in the implementation.

If this grep shows matches in the current workspace (i.e., a prior implementation attempt added
these functions), remove them as part of Task 14. If no matches are found (workspace is at spec
state), document that fact in the PR description — no deletion action needed.

Traces to: ADR-052 §D4 Step 4 ("Deletion: text-scanner apparatus removed").

### AC-024 — Unicode input (non-ASCII characters near literal) → no panic; VP-021 guard (ADR-052 §D4 Step 3 "Unicode inputs: no raw byte-offset operations")

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_unicode_input_no_panic)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. The test must NOT panic (test framework catches panics as FAIL).

The test verifies that multi-byte Unicode input adjacent to or surrounding a date-like
literal does not cause a byte-offset panic. Under Option-A, `check_temporal_literals`
operates on already-parsed `String` values (valid UTF-8 from the tokenizer) — no raw
byte slicing. VP-021 violation from the text-scanner approach is eliminated by construction.

Traces to: VP-021 (never panics); ADR-052 §D4 Step 3 ("Unicode inputs: operates on
already-parsed `String` values; no raw byte-offset operations. VP-021 violation eliminated
by construction").

### AC-025 — is_date_like forms 3-7 (fractional T-sep, no-seconds T-sep, space-sep family) each produce E-QUERY-041 against Datetime column (ADR-052 §D4 is_date_like 7-form acceptance set; BC-2.11.021 §Error Cases EC-11-021-010..012)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_fractional_t_sep_datetime_col)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (form 3 — `'2026-06-24T12:00:00.123'` vs Datetime col → E-QUERY-041)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_no_seconds_t_sep_datetime_col)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (form 4 — `'2026-06-24T12:00'` vs Datetime col → E-QUERY-041)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_full_seconds_datetime_col)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (form 5 — `'2026-06-24 12:00:00'` vs Datetime col → E-QUERY-041)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_fractional_datetime_col)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (form 6 — `'2026-06-24 12:00:00.500'` vs Datetime col → E-QUERY-041)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_no_seconds_datetime_col)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (form 7 — `'2026-06-24 12:00'` vs Datetime col → E-QUERY-041)

These five tests (RG-026 through RG-030) verify:
- Form 3: `'2026-06-24T12:00:00.123'` — parser produces `RawTemporalLiteral("2026-06-24T12:00:00.123")` via `%Y-%m-%dT%H:%M:%S%.f`; `check_temporal_literals` → `PrismError::TemporalLiteralUnparseable`
- Form 4: `'2026-06-24T12:00'` — parser produces `RawTemporalLiteral` via `%Y-%m-%dT%H:%M`; → E-QUERY-041
- Form 5: `'2026-06-24 12:00:00'` — space-sep full seconds via `%Y-%m-%d %H:%M:%S`; → E-QUERY-041
- Form 6: `'2026-06-24 12:00:00.500'` — space-sep fractional via `%Y-%m-%d %H:%M:%S%.f`; → E-QUERY-041
- Form 7: `'2026-06-24 12:00'` — space-sep no seconds via `%Y-%m-%d %H:%M`; → E-QUERY-041

In ALL cases: parse SUCCEEDS (no E-QUERY-001 at parse time); `check_temporal_literals` walks the AST, resolves the column as `Timestamp(Microsecond, UTC)`, returns `Err(PrismError::TemporalLiteralUnparseable)`.

Traces to: BC-2.11.021 §Error Cases E-QUERY-041 (7-form acceptance set); EC-11-021-010 (form 4), EC-11-021-011 (form 3), EC-11-021-012 (form 5); ADR-052 §D4 `is_date_like` Acceptance Set (Canonical), forms 3-7.

### AC-026 — Space-sep `is_date_like` form vs String/Utf8 column → COERCE success; extends RISK-5 coverage to space-sep family (ADR-052 §D4 coercion arm; BC-2.11.021 §Error Cases EC-11-021-013)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_space_sep_succeeds)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

This test (RG-031) verifies:
- `SELECT * FROM t WHERE string_col = '2026-06-24 12:00:00'` (form 5, space-sep, vs String/Utf8 col) SUCCEEDS without error
- `check_temporal_literals` resolves `string_col` → `DataType::Utf8` → COERCE: rewrites `RawTemporalLiteral("2026-06-24 12:00:00")` to `Literal::String("2026-06-24 12:00:00")` in-place
- Emitted SQL: `string_col = '2026-06-24 12:00:00'` — byte-identical to pre-ADR-052 behavior
- The coercion arm applies to all 7 `is_date_like` forms; RG-013 (date-only) and RG-014 (T-sep offset-less) are joined by RG-031 (space-sep) to make RISK-5 coverage family-complete

Traces to: BC-2.11.021 §Error Cases E-QUERY-041 coercion arm; EC-11-021-013 (space-sep form vs String/Utf8 col → COERCE); ADR-052 §D4 coercion arm (RISK-5 resolved by design for all 7 forms).

### AC-027 — Benign over-match `'2026-6-24'` (unpadded): Datetime col → E-QUERY-041 ACCEPTED BENIGN; String col → COERCE (ADR-052 §D4 over-match disposition; BC-2.11.021 §Error Cases EC-11-021-014)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_unpadded_date_overmatch_datetime_col)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (RG-032 — `'2026-6-24'` vs Datetime col → E-QUERY-041)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_unpadded_date_succeeds)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (RG-033 — `'2026-6-24'` vs String col → COERCE)

These two tests verify (per ADR-052 §D4 and BC-2.11.021 §Error Cases EC-11-021-014):
- `'2026-6-24'` (single-digit month and day): `chrono::NaiveDate::parse_from_str("2026-6-24", "%Y-%m-%d")` SUCCEEDS (chrono `%m`/`%d` accept single digits — this is the over-match) → `is_date_like` returns `true`
- Against Datetime/Timestamp col: `check_temporal_literals` → E-QUERY-041 (ACCEPTED BENIGN — "use RFC-3339" message is accurate and apt; unpadded forms are also non-RFC-3339)
- Against String/Utf8 col: `check_temporal_literals` → COERCE → SUCCESS
- No regex guard or year-width constraint is applied; the accepted-benign stance is the spec contract

Traces to: BC-2.11.021 §Error Cases EC-11-021-014; ADR-052 §D4 `is_date_like` Acceptance Set (Canonical) over-match disposition note.

### AC-028 — Near-miss `'2026-06-24extra'` (trailing chars) stays `Literal::String`; no temporal error; confirms chrono full-consumption negative boundary (ADR-052 §D4 `is_date_like` — all format strings require full consumption)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_near_miss_trailing_chars_stays_utf8)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

This test (RG-034) verifies:
- `'2026-06-24extra'` (trailing chars after a valid date prefix): `chrono::NaiveDate::parse_from_str("2026-06-24extra", "%Y-%m-%d")` fails (leftover input `extra`); all 6 `NaiveDateTime` variants also fail (leftover chars). `is_date_like` returns `false`.
- Parser emits `Literal::String("2026-06-24extra")` (NOT `RawTemporalLiteral`)
- `check_temporal_literals` does not fire
- No E-QUERY-041 or E-QUERY-001 related to temporal validation
- The negative boundary is tight: chrono `parse_from_str` requires FULL string consumption; prefix-matching of date-shaped strings with arbitrary suffixes does NOT occur

Verify the near-miss is not matched by `is_date_like` in the parser source:

```bash
grep -n 'is_date_like\|parse_from_str' crates/prism-query/src/sql_parser.rs | head -15
```

Expected: the `is_date_like` function uses `parse_from_str` (not `find` or `starts_with`) on all 7 format strings, confirming full-consumption semantics.

Traces to: ADR-052 §D4 `is_date_like` function (7 `parse_from_str` calls, each requiring full string consumption); ADR-052 §`is_date_like` Acceptance Set "Forms that stay `Literal::String` — NOT matched" table (`'2026-06-24Z'` entry demonstrates same full-consumption property).

### AC-029 — SELECT projection `RawTemporalLiteral` coerces to `Literal::String` → query SUCCESS (ADR-052 §D4 v1.10 projection-coerce-preserved; BC-2.11.021 §Postconditions)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_projection_position_e_query_001)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

This test (RG-023) verifies:
- `SELECT '2026-06-24' FROM t` (date-like literal in SELECT projection, no column comparison context)
- `check_temporal_literals` finds `RawTemporalLiteral("2026-06-24")` in non-comparison SELECT position
- Rewrites in-place to `Literal::String("2026-06-24")` → Ok(()) — query SUCCEEDS
- Emitted SQL projection is `'2026-06-24'` (string constant) — NOT E-QUERY-002, NOT E-QUERY-042

Regression guard: the projection coerce arm is PRESERVED by ADR-052 §D4 v1.10.
The Datetime-comparison arm (E-QUERY-041), numeric-comparison arm (E-QUERY-002), and
GROUP BY/ORDER BY reject arms (E-QUERY-042) are DISTINCT code paths.

Verify the projection coerce arm exists in `engine.rs`:

```bash
grep -n 'non.comparison\|projection\|SELECT' crates/prism-query/src/engine.rs \
  | grep -i 'coerce\|RawTemporal\|String' | head -5
```

Expected: at least one match confirming the SELECT projection coerce arm.

Traces to: BC-2.11.021 §Postconditions (non-comparison SELECT projection coerce arm); ADR-052 §D4 v1.10.

### AC-030 — SQL `GROUP BY` date-like literal → E-QUERY-042 (TemporalLiteralInvalidPosition::GroupBy, INVALID_PARAMS) (ADR-052 §D4 v1.10, error-taxonomy §E-QUERY-042)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_projection_group_by_date_like_coerces)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (RG-035 — GROUP BY position → E-QUERY-042 GroupBy reject)

This test verifies:
- `SELECT count(*) FROM t GROUP BY '2026-06-24'` (date-like literal in SQL GROUP BY clause)
- `check_temporal_literals` detects `RawTemporalLiteral("2026-06-24")` in GROUP BY position →
  `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::GroupBy, value_prefix: "2026-06-24".into() })`
- `map_prism_error` maps it to `codes::INVALID_PARAMS` (-32602)
- Does NOT map to -32000 (INTERNAL_ERROR) — this is a caller-resolvable semantic error

Verify the GROUP BY reject arm exists:

```bash
grep -n 'TemporalLiteralInvalidPosition\|GroupBy' crates/prism-query/src/engine.rs | head -5
```

Expected: at least one match for `TemporalLiteralInvalidPosition` in the `check_temporal_literals` body.

Traces to: BC-2.11.021 §Postconditions (GROUP BY position → E-QUERY-042); error-taxonomy §E-QUERY-042; ADR-052 §D4 v1.10.

### AC-031 — SQL `ORDER BY` date-like literal → E-QUERY-042 (TemporalLiteralInvalidPosition::OrderBy, INVALID_PARAMS) (ADR-052 §D4 v1.10, error-taxonomy §E-QUERY-042)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_order_by_date_like_coerces)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (RG-036 — ORDER BY position → E-QUERY-042 OrderBy reject)

This test verifies:
- `SELECT * FROM t ORDER BY '2026-06-24'` (date-like literal in SQL ORDER BY clause)
- `check_temporal_literals` → `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::OrderBy, value_prefix: "2026-06-24".into() })`
- `map_prism_error` → `codes::INVALID_PARAMS` — NOT -32000

Verify the `TemporalLiteralInvalidPosition` variant exists in `error.rs`:

```bash
grep -c 'TemporalLiteralInvalidPosition' crates/prism-core/src/error.rs
```

Expected: at least `2` (variant definition + Display arm).

Traces to: BC-2.11.021 §Postconditions (ORDER BY position → E-QUERY-042); error-taxonomy §E-QUERY-042; ADR-052 §D4 v1.10.

### AC-032 — Pipe `stats by` date-like literal → parse-time E-QUERY-001 (enhanced message; `stats by` only accepts field paths) (ADR-052 §D4 v1.10)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_stats_by_date_like_e_query_001)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (RG-037 — pipe `stats by` → parse-time E-QUERY-001)

This test verifies:
- `FROM t | stats count by '2026-06-24'` (pipe-mode `stats by` clause with bare literal)
- Parser (in `filter_parser.rs`) rejects the literal AT PARSE TIME with enhanced E-QUERY-001
- Error message indicates "`stats by` only accepts field paths, not literal values"
- Error code is E-QUERY-001 (parse/syntax), NOT E-QUERY-042 (plan/semantic)
- `check_temporal_literals` is NEVER reached — the query is rejected before AST production

Traces to: BC-2.11.004 §Postconditions (pipe stats-by parse rejection); ADR-052 §D4 v1.10.

### AC-033 — Pipe `sort` date-like literal → parse-time E-QUERY-001 (enhanced message; `sort` only accepts field paths) (ADR-052 §D4 v1.10)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sort_date_like_e_query_001)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (RG-038 — pipe `sort` → parse-time E-QUERY-001)

This test verifies:
- `FROM t | sort '2026-06-24'` (pipe-mode `sort` clause with bare literal)
- Parser rejects AT PARSE TIME with enhanced E-QUERY-001: "`sort` only accepts field paths"
- Error code is E-QUERY-001, NOT E-QUERY-042
- `check_temporal_literals` never reached

Traces to: BC-2.11.004 §Postconditions (pipe sort parse rejection); ADR-052 §D4 v1.10.

### AC-034 — Non-column-LHS comparison (function/expr LHS, date-like RHS) → E-QUERY-042 (NonColumnLhsComparison, INVALID_PARAMS) (ADR-052 §D4 v1.10, error-taxonomy §E-QUERY-042)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_non_column_lhs_date_like_e_query_042)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`. (RG-039 — non-column-LHS → E-QUERY-042 NonColumnLhsComparison)

This test verifies:
- `WHERE lower(hostname) = '2026-06-24'` (function call on LHS, date-like literal on RHS)
- `check_temporal_literals` detects `RawTemporalLiteral("2026-06-24")` on the RHS where the
  LHS is NOT a plain column reference (it is a function expression `lower(hostname)`)
- Returns `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::NonColumnLhsComparison, value_prefix: "2026-06-24".into() })` (E-QUERY-042)
- `map_prism_error` → `codes::INVALID_PARAMS` — NOT -32000 (INTERNAL_ERROR)

This is distinct from the plain-column-LHS path:
- `WHERE ts_col = '2026-06-24'` (plain column LHS) → E-QUERY-041 via column-type resolution
- `WHERE lower(hostname) = '2026-06-24'` (non-column LHS) → E-QUERY-042 NonColumnLhsComparison

Traces to: BC-2.11.021 §Postconditions (non-column-LHS comparison → E-QUERY-042); error-taxonomy §E-QUERY-042; ADR-052 §D4 v1.10.

## Red Gate

Thirty-nine Red Gate tests. All use `todo!()` stubs before implementation. Pre-implementation
compile state: tests referencing `PrismError::TemporalLiteralUnparseable` (RG-004 through
RG-006, RG-025 through RG-030, RG-032) FAIL TO COMPILE because the variant does not yet
exist. Tests referencing `PrismError::TemporalLiteralInvalidPosition` (RG-035, RG-036, RG-039)
FAIL TO COMPILE because the variant does not yet exist. Tests referencing `Literal::RawTemporalLiteral`
(RG-011, RG-013, RG-014, RG-015, RG-016, RG-017, RG-019, RG-020, RG-021, RG-022, RG-023,
RG-024, RG-031, RG-033, RG-035, RG-036, RG-039) FAIL TO COMPILE because the variant does not
yet exist. RG-037 and RG-038 (pipe parse-time E-QUERY-001) compile but PANIC with `todo!()`.
RG-034 compiles but PANICS with `todo!()`.

### RG-001 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_datetime_column_registers_as_timestamp_micros_utc`

**Location:** `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`

**Pre-implementation state:** `todo!()` panic — `column_type_to_arrow(ColumnType::Datetime)` still returns `DataType::Utf8`.

**Post-implementation state:** asserts `column_type_to_arrow(ColumnType::Datetime)` returns
`DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))` → PASS.

**Why load-bearing:** Foundation gate for the entire migration. If this returns Utf8, all timestamp-type behaviors are incoherent.

**SID-1 compliance:** `column_type_to_arrow` is a pure function — deterministic, no `#[ignore]`.

### RG-002 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_risk1_datafusion_arrow_cast_probe`

**Location:** `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`

**Pre-implementation state:** `todo!()` panic.

**Post-implementation state:**
- Creates DataFusion `SessionContext` with `Timestamp(Microsecond, Some(Arc::from("UTC")))` column
- Plans `SELECT * FROM t WHERE ts > arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')`
- Verifies plan produced without error
- Verifies literal is typed `Timestamp(Microsecond, Some("UTC"))` — NOT `Timestamp(Nanosecond, None)`

**Why load-bearing:** RISK-1 mitigation — pins arrow_cast behavior to DataFusion 53.1.0; fails fast on version upgrades.

**SID-1 compliance:** Uses DataFusion in-process — no external service, no `#[ignore]`.

### RG-003 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sql_emitter_yields_arrow_cast_literal`

**Location:** `crates/prism-query/src/` emitter test module

**Pre-implementation state:** emitter produces `"'2026-07-03T00:00:00Z'"` (bare string); test asserting `arrow_cast(...)` form FAILS.

**Post-implementation state:** emitter produces `"arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some(\"UTC\"))')"` → PASS.

**Why load-bearing:** The bare-string form caused DataFusion to see `Utf8` vs `Timestamp(Microsecond, UTC)`. The `arrow_cast(...)` form produces exact type match.

### RG-004 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_sql_mode_date_only_string`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `PrismError::TemporalLiteralUnparseable` does not exist → compile error.

**Post-implementation state (Option-A mechanism):** SQL query `SELECT * FROM t WHERE timestamp > '2026-06-24'` (date-only):
1. Parser SUCCEEDS, emits `Literal::RawTemporalLiteral("2026-06-24")` in the AST
2. `check_temporal_literals` walks the AST, resolves `timestamp` column to `Timestamp(Microsecond, UTC)`
3. Returns `Err(PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24".into() })`

The test asserts: error IS `TemporalLiteralUnparseable`; error is NOT from parse step (parse succeeds); error is NOT a DataFusion cast error.

**Why load-bearing:** Core behavior gate for E-QUERY-041 SQL mode.

### RG-005 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_pipe_mode_date_only_string`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** same compile error as RG-004.

**Post-implementation state (Option-A mechanism):** pipe query `FROM t | where timestamp > '2026-06-24'`:
1. Parser SUCCEEDS, emits `RawTemporalLiteral("2026-06-24")`
2. `check_temporal_literals` → E-QUERY-041

**Why load-bearing:** BC-2.11.004 §Edge Cases EC-11-004-001 — parity between SQL mode and pipe `| where` is required.

### RG-006 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_map_prism_error_invalid_params`

**Location:** `crates/prism-mcp/src/error_mapping.rs` test module

**Pre-implementation state:** compile error (variant missing) OR falls through to `-32000` catch-all.

**Post-implementation state:** `map_prism_error(PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24".into() })` returns MCP code `-32602 INVALID_PARAMS`.

**Why load-bearing:** E-QUERY-041 is a caller-resolvable error. `-32000` (internal error) misleads the MCP caller.

### RG-007 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_valid_rfc3339_utc_string_not_rejected`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `todo!()` panic.

**Post-implementation state (Option-A mechanism):**
- `SELECT * FROM t WHERE timestamp > '2026-06-24T00:00:00Z'` (full RFC-3339):
  1. Parser produces `Literal::Timestamp(...)` — NOT `RawTemporalLiteral`
  2. `check_temporal_literals` finds no `RawTemporalLiteral` nodes → no error
  3. Query proceeds to DataFusion execution successfully
- Regression guard: full RFC-3339 must never be downgraded to `RawTemporalLiteral`

**Why load-bearing:** If valid RFC-3339 is accidentally classified as `RawTemporalLiteral`, all existing analyst queries break silently.

**SID-1 compliance:** Uses in-process test execution — no `#[ignore]`.

### RG-008 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sensor_datetime_string_parsed_to_micros`

**Location:** `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`

**Pre-implementation state:** `todo!()` panic — `parse_datetime_to_micros` helper doesn't exist.

**Post-implementation state:** the `parse_datetime_to_micros` helper converts `"2026-07-03T00:00:00Z"` to the correct `i64` microseconds-since-epoch value, derived at test time via `chrono::DateTime::parse_from_rfc3339("2026-07-03T00:00:00Z").unwrap().timestamp_micros()`. Do NOT hardcode a magic constant (TD-VSDD-091).

**Why load-bearing:** Arrow `Timestamp(Microsecond, UTC)` columns store `i64` microseconds. Storing Utf8 values into a Timestamp schema column produces null/panic at materialization.

**SID-1 compliance:** `pub(super)` pure function — no external deps, deterministic.

### RG-009 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_high002_datetime_column_type_is_timestamp`

**Location:** `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`

**Pre-implementation state:** asserts `Timestamp(Microsecond, Some(Arc::from("UTC")))` but actual column type is still `Utf8` → FAILS.

**Post-implementation state:** column type is `Timestamp(Microsecond, Some(Arc::from("UTC")))` → PASS.

**Why load-bearing:** `high002_plan_pinning_tests.rs` is the canonical plan-stability test file explicitly identified by ADR-052 as the primary verification gate for D2.

### RG-010 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_output_plans_against_timestamp_column`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `todo!()` panic — emitter produces bare string; planning against Timestamp column would fail or produce wrong type.

**Post-implementation state:**
- Calls actual `pipe_sql_emitter.rs` `Literal::Timestamp` render function to obtain the emitted SQL string
- Registers `Timestamp(Microsecond, Some(Arc::from("UTC")))` column in DataFusion `SessionContext`
- Plans `SELECT * FROM t WHERE ts_col > {actual_emitter_output}` where `{actual_emitter_output}` is the real `arrow_cast(...)` form
- Verifies the plan succeeds without error

**Why load-bearing:** Closes the transitive coverage gap between RG-002 (hand-written `arrow_cast` query) and RG-003 (emitter output in isolation). If the emitter format string has a quoting or escaping mistake invisible in a string comparison, RG-010 will catch it at DataFusion plan time.

### RG-011 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_full_rfc3339_regression_guard`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `todo!()` panic.

**Post-implementation state:**
- Asserts parsing `'2026-07-03T00:00:00Z'` produces `Literal::Timestamp` (NOT `RawTemporalLiteral`)
- Asserts the AST contains NO `RawTemporalLiteral` nodes after parsing this input
- Asserts `check_temporal_literals` runs cleanly (no errors) when the AST has no `RawTemporalLiteral` nodes
- Regression guard: this test MUST fail if the parser accidentally classifies full RFC-3339 strings as `RawTemporalLiteral`

**Why load-bearing:** The most common analyst datetime input is full RFC-3339 (e.g., `NOW() - INTERVAL '24h'` injects this form). Misclassifying it would silently break all temporal queries.

### RG-012 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_offset_less_datetime_col`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (no `TemporalLiteralUnparseable` variant).

**Post-implementation state:**
- SQL: `SELECT * FROM t WHERE timestamp > '2026-06-24T12:00:00'` (offset-less ISO) against Datetime col
- Parser produces `RawTemporalLiteral("2026-06-24T12:00:00")` (is_date_like matches `NaiveDateTime`)
- `check_temporal_literals` → `PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24T12:00:00".into() }`

**Why load-bearing:** Offset-less ISO is the second most common malformed datetime form. The `is_date_like` heuristic must cover BOTH `NaiveDate` and `NaiveDateTime` patterns.

### RG-013 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_date_only_succeeds`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (no `RawTemporalLiteral` variant; also, without the coercion arm this test would return E-QUERY-002 (`QueryPlanFailed`) at runtime if only the reject arm existed).

**Post-implementation state:**
- `SELECT * FROM t WHERE string_col = '2026-06-24'` where `string_col` is `DataType::Utf8` (String column)
- Parser produces `RawTemporalLiteral("2026-06-24")`
- `check_temporal_literals` resolves `string_col` → `DataType::Utf8` → COERCE: rewrites to `Literal::String("2026-06-24")`
- Query returns `Ok(...)` — no error
- Test asserts the emitted SQL comparison is `string_col = '2026-06-24'` byte-identical to pre-ADR-052 behavior

**Why load-bearing:** RISK-5 regression guard — without the coercion arm, every query with a date-like literal against a String column would break. This is a real behavior regression that must not happen.

### RG-014 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_offset_less_succeeds`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** same as RG-013.

**Post-implementation state:**
- `SELECT * FROM t WHERE string_col = '2026-06-24T12:00:00'` against String/Utf8 col → COERCE, SUCCEEDS
- Emitted SQL comparison is `string_col = '2026-06-24T12:00:00'` — byte-identical to pre-ADR-052

**Why load-bearing:** Verifies the coercion arm handles the `NaiveDateTime` (offset-less) pattern too.

### RG-015 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_integer_col_date_like_e_query_001`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (no `RawTemporalLiteral` variant).

**Post-implementation state:**
- `SELECT * FROM t WHERE int_col = '2026-06-24'` against Integer column
- Parser produces `RawTemporalLiteral("2026-06-24")`
- `check_temporal_literals` resolves `int_col` → `DataType::Int64` (or similar Integer type)
- Returns `Err(PrismError::QueryTypeMismatch { column: "int_col", table: "t", actual_type: Int64, operator: "=" })` (E-QUERY-002)
- NOT E-QUERY-041 (that is only for Datetime/Timestamp columns)

**Why load-bearing:** The four-way dispatch must route Integer columns to E-QUERY-002 (`QueryTypeMismatch`), not E-QUERY-041.

### RG-016 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_float_col_date_like_e_query_001`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error.

**Post-implementation state:** `'2026-06-24'` vs Float column → E-QUERY-002 (`QueryTypeMismatch`). Same pattern as RG-015 but for Float type.

**Why load-bearing:** The four-way dispatch must be exhaustive across numeric types.

### RG-017 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_bool_col_date_like_e_query_001`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error.

**Post-implementation state:** `'2026-06-24'` vs Bool column → E-QUERY-002 (`QueryTypeMismatch`).

**Why load-bearing:** Boolean mismatch must not produce a temporal error.

### RG-018 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_non_date_like_stays_string_literal`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `todo!()` panic.

**Post-implementation state:**
- Parsing `'not-a-date'` in any query position produces `Literal::String("not-a-date")` (NOT `RawTemporalLiteral`)
- Parsing `'sensor-id-abc'` similarly produces `Literal::String("sensor-id-abc")`
- `is_date_like` returns `false` for these inputs
- No temporal error is emitted for these literals anywhere in the query pipeline

**Why load-bearing:** The `is_date_like` heuristic must have a tight negative case. Any arbitrary string that happens to partially match a date format must NOT be classified as `RawTemporalLiteral`.

### RG-019 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_dotted_source_column_resolution`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (no `TemporalLiteralUnparseable` variant).

**Post-implementation state:**
- Query `SELECT * FROM source WHERE source.ts_col > '2026-06-24'` where `source.ts_col` is a Datetime column
- `check_temporal_literals` resolves `source.ts_col` via the schema (dotted source lookup, NOT string split on `.`)
- Returns E-QUERY-041 with `value_prefix: "2026-06-24"`
- Must NOT return E-QUERY-037 (source not found) — the dotted source is valid

**Why load-bearing:** Dotted source expressions were the primary failure mode of the text-scanner (8 fix-bursts). This test proves the schema-resolution path handles dotted expressions correctly.

### RG-020 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_qualified_nested_column_resolution`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (no `TemporalLiteralUnparseable`).

**Post-implementation state:**
- Setup: two-source query with `t1` (has Datetime col `event_time`) and `t2` (has String col `event_time`)
- Query: `... WHERE t2.event_time = '2026-06-24'` (qualified against `t2` which has String col)
- `check_temporal_literals` resolves `t2.event_time` against `t2`'s schema → `DataType::Utf8` → COERCE, SUCCEEDS
- Does NOT mistakenly look up `event_time` in `t1` (`.last()` collapse prevented)

**Why load-bearing:** Qualified column resolution (`other.timestamp`) against the CORRECT source table was the second major failure mode of the text-scanner.

### RG-021 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_filter_pipe_syntax_e_query_041`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (no `TemporalLiteralUnparseable`).

**Post-implementation state:**
- Pipe-syntax query `source | where ts_col > '2026-06-24'` (the `|` pipe operator)
- Parser SUCCEEDS with `RawTemporalLiteral("2026-06-24")` in the pipe-stage predicate
- `check_temporal_literals` resolves `ts_col` against `source`'s schema → Datetime → E-QUERY-041
- NOT misclassified as "source missing" — the `source |` prefix is parsed correctly by the AST

**Why load-bearing:** Filter-mode with `source |` prefix was a known failure mode of the text-scanner. This test proves the AST-walk path handles pipe syntax correctly.

### RG-022 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_unicode_input_no_panic`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `todo!()` panic.

**Post-implementation state:**
- Query with multi-byte Unicode characters adjacent to a date-like literal:
  e.g., `SELECT * FROM t WHERE ts > '2026-06-24' /* 日本語コメント */`
  or `SELECT * FROM t WHERE name = '観測所' AND ts > '2026-06-24'`
- The test MUST NOT panic (test framework records panic as FAIL)
- Assert either: graceful E-QUERY-041 / E-QUERY-002 (error, not panic) OR query success

**Why load-bearing:** VP-021 regression guard. The text-scanner VP-021 violation was a Unicode byte-offset panic caused by raw byte slicing on UTF-8 strings. Under Option-A, `first_50_chars` uses `char_indices()` and `check_temporal_literals` operates on already-parsed strings — the panic path is eliminated by construction. This test confirms elimination.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-023 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_projection_position_e_query_001`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (no `RawTemporalLiteral`).

**Post-implementation state (ADR-052 §D4 — non-comparison coerce; BC-2.11.021 §Postconditions):**
- `SELECT '2026-06-24' FROM t` (date-like literal in SELECT projection, no column comparison)
- `check_temporal_literals` finds `Literal::RawTemporalLiteral("2026-06-24")` in non-comparison position
- COERCE: rewrites in-place to `Literal::String("2026-06-24")` → Ok(()) — query SUCCEEDS
- Emitted SQL projection is `'2026-06-24'` (string constant)
- NOT E-QUERY-002 (behavior reversed from pre-v1.8 spec; test function name unchanged per append-only naming policy)
- NOT E-QUERY-041 (that requires a Datetime column comparison)

(Regression guards: the Datetime-comparison arm in RG-004/005 still produces E-QUERY-041 — unchanged.
The numeric-comparison arm in RG-015/016/017 still produces E-QUERY-002 — unchanged.)

**Why load-bearing:** Confirms the non-comparison arm COERCEs (ADR-052 §D4) rather than erroring,
and that the coerced literal is safely emitted as a string constant without triggering the emitter guard arm.

### RG-024 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_guard_raw_temporal_literal`

**Location:** `crates/prism-query/src/` emitter test module

**Pre-implementation state:** compile error (no `RawTemporalLiteral`).

**Post-implementation state:**
- Directly calls the `pipe_sql_emitter.rs` emit function with a `Literal::RawTemporalLiteral("2026-06-24")` as input
- Asserts return is `Err(PrismError::QueryPlanFailed { detail: "..." })` with detail containing "internal error" or "RawTemporalLiteral" (E-QUERY-002)
- Asserts NOT a panic (the arm must return an error, not use `unreachable!()` or `panic!()`)

**Why load-bearing:** Guard arm reachability test. `check_temporal_literals` must consume all `RawTemporalLiteral` nodes before emission; this test proves the belt-and-suspenders arm is wired and functional even in hypothetical bypass scenarios.

### RG-025 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_message_format_byte_identical`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (no `TemporalLiteralUnparseable`).

**Post-implementation state:**
- Trigger E-QUERY-041 with `value_prefix = "2026-06-24"` (a known 10-char prefix, under 50)
- Assert the `Display` string on `PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24".to_string() }` matches EXACTLY (byte-for-byte, POL-24):

  ```
  E-QUERY-041: The value '2026-06-24' cannot be interpreted as a UTC timestamp. Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only and offset-less forms are not accepted. For relative time filters, use NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h').
  ```

**Why load-bearing:** POL-24 regression guard. The message format is part of the MCP contract — any deviation (extra whitespace, different punctuation, reordered clauses) is an API break.

### RG-026 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_fractional_t_sep_datetime_col`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (`PrismError::TemporalLiteralUnparseable` and `Literal::RawTemporalLiteral` variants do not exist).

**Post-implementation state (ADR-052 §D4 form 3 — T-sep fractional; BC-2.11.021 §Error Cases EC-11-021-011):**
- SQL: `SELECT * FROM t WHERE timestamp > '2026-06-24T12:00:00.123'` against Datetime col
- Parser: `parse_from_rfc3339` fails (no UTC offset); `is_date_like` → `NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")` → `true` → `Literal::RawTemporalLiteral("2026-06-24T12:00:00.123")`
- `check_temporal_literals` → `Err(PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24T12:00:00.123".into() })`

**Why load-bearing:** Fractional-seconds T-sep forms were absent from `is_date_like` in v1.2 (only 2 forms). If form 3 is missing, `'2026-06-24T12:00:00.123'` falls through to `Literal::String` and produces a DataFusion type-mismatch E-QUERY-002 (`QueryPlanFailed`) at plan time instead of the correct E-QUERY-041.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-027 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_no_seconds_t_sep_datetime_col`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (`TemporalLiteralUnparseable` and `RawTemporalLiteral` variants do not exist).

**Post-implementation state (ADR-052 §D4 form 4 — T-sep no seconds; BC-2.11.021 §Error Cases EC-11-021-010):**
- SQL: `SELECT * FROM t WHERE timestamp > '2026-06-24T12:00'` against Datetime col
- Parser: `is_date_like` → `NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M")` → `true` → `Literal::RawTemporalLiteral("2026-06-24T12:00")`
- `check_temporal_literals` → `Err(PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24T12:00".into() })`

**Why load-bearing:** Form 4 (T-sep, no seconds) is a common analyst shorthand. Without it in `is_date_like`, it falls through to `Literal::String` → wrong error code. EC-11-021-010 confirms this form.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-028 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_full_seconds_datetime_col`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (`TemporalLiteralUnparseable` and `RawTemporalLiteral` variants do not exist).

**Post-implementation state (ADR-052 §D4 form 5 — space-sep full seconds; BC-2.11.021 §Error Cases EC-11-021-012):**
- SQL: `SELECT * FROM t WHERE timestamp > '2026-06-24 12:00:00'` against Datetime col
- Parser: `is_date_like` → `NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")` → `true` → `Literal::RawTemporalLiteral("2026-06-24 12:00:00")`
- `check_temporal_literals` → `Err(PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24 12:00:00".into() })`

**Why load-bearing:** Space-separated datetime formats are common in SQL editors and log exports; entirely absent from `is_date_like` in v1.2. This anchors the space-sep sub-family. EC-11-021-012 confirms this form.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-029 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_fractional_datetime_col`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error.

**Post-implementation state (ADR-052 §D4 form 6 — space-sep fractional):**
- SQL: `SELECT * FROM t WHERE timestamp > '2026-06-24 12:00:00.500'` against Datetime col
- Parser: `is_date_like` → `NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")` → `true` → `Literal::RawTemporalLiteral("2026-06-24 12:00:00.500")`
- `check_temporal_literals` → E-QUERY-041

**Why load-bearing:** Fractional-seconds space-sep forms appear in sensor log exports. All 7 canonical forms must produce E-QUERY-041 against Datetime cols consistently.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-030 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_space_sep_no_seconds_datetime_col`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error.

**Post-implementation state (ADR-052 §D4 form 7 — space-sep no seconds):**
- SQL: `SELECT * FROM t WHERE timestamp > '2026-06-24 12:00'` against Datetime col
- Parser: `is_date_like` → `NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")` → `true` → `Literal::RawTemporalLiteral("2026-06-24 12:00")`
- `check_temporal_literals` → E-QUERY-041

**Why load-bearing:** Completes the space-sep sub-family coverage. Forms 5, 6, and 7 all travel the same code path; this test anchors form 7 specifically.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-031 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_space_sep_succeeds`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (`RawTemporalLiteral` variant does not exist; without the coercion arm, returns E-QUERY-002 (`QueryPlanFailed`) at runtime).

**Post-implementation state (ADR-052 §D4 coercion arm; BC-2.11.021 §Error Cases EC-11-021-013):**
- `SELECT * FROM t WHERE string_col = '2026-06-24 12:00:00'` where `string_col` is `DataType::Utf8`
- Parser: `is_date_like` → `true` (form 5) → `Literal::RawTemporalLiteral("2026-06-24 12:00:00")`
- `check_temporal_literals` resolves `string_col` → `DataType::Utf8` → COERCE: rewrites to `Literal::String("2026-06-24 12:00:00")`
- Query returns `Ok(...)` — no error; emitted SQL: `string_col = '2026-06-24 12:00:00'` (byte-identical to pre-ADR-052)

**Why load-bearing:** RISK-5 regression guard for the space-sep family. Sensors may store space-separated datetime strings as report labels or external IDs in String columns. RG-013 and RG-014 cover date-only and T-sep offset-less coercion; RG-031 confirms the coercion arm applies to space-sep forms as well (family-complete coverage).

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-032 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_unpadded_date_overmatch_datetime_col`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (`TemporalLiteralUnparseable` variant does not exist).

**Post-implementation state (ADR-052 §D4 over-match disposition; BC-2.11.021 §Error Cases EC-11-021-014):**
- SQL: `SELECT * FROM t WHERE timestamp > '2026-6-24'` (unpadded single-digit month/day) against Datetime col
- Parser: `is_date_like` → `NaiveDate::parse_from_str("2026-6-24", "%Y-%m-%d")` MATCHES (chrono `%m`/`%d` accept single digits — the over-match) → `true` → `Literal::RawTemporalLiteral("2026-6-24")`
- `check_temporal_literals` → E-QUERY-041 with `value_prefix: "2026-6-24"` (ACCEPTED BENIGN — "use RFC-3339" is accurate and apt; unpadded forms are also non-RFC-3339)
- No regex guard or year-width constraint is applied

**Why load-bearing:** Confirms the over-match disposition is intentional. If a future implementer adds a regex guard to reject `'2026-6-24'`, this test catches the spec-violating narrowing. The accepted-benign stance is part of the contract.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-033 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_string_col_coercion_unpadded_date_succeeds`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error (`RawTemporalLiteral` variant does not exist).

**Post-implementation state (ADR-052 §D4 coercion arm + over-match disposition):**
- `SELECT * FROM t WHERE string_col = '2026-6-24'` (unpadded) where `string_col` is `DataType::Utf8`
- Parser: `is_date_like` → `true` (over-match via form 1) → `Literal::RawTemporalLiteral("2026-6-24")`
- `check_temporal_literals` → `DataType::Utf8` → COERCE: rewrites to `Literal::String("2026-6-24")`
- Query returns `Ok(...)` — no error; emitted SQL: `string_col = '2026-6-24'` (correct: unpadded string comparison)

**Why load-bearing:** Confirms the coercion arm applies to over-matched forms. An unpadded date label in a String column (legitimate format in some sensor APIs) must not produce an error.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-034 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_near_miss_trailing_chars_stays_utf8`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `todo!()` panic.

**Post-implementation state (ADR-052 §D4 `is_date_like` negative boundary — full-consumption requirement):**
- `'2026-06-24extra'` (trailing chars after a valid date prefix): `chrono::NaiveDate::parse_from_str("2026-06-24extra", "%Y-%m-%d")` fails (leftover input `extra`); all 6 `NaiveDateTime` format variants also fail (leftover chars). `is_date_like` returns `false`.
- Parser emits `Literal::String("2026-06-24extra")` (NOT `RawTemporalLiteral`)
- `check_temporal_literals` does not fire (no `RawTemporalLiteral` nodes in AST)
- No E-QUERY-041 or E-QUERY-001 related to temporal validation

**Why load-bearing:** Confirms chrono's full-consumption property provides a tight negative bound. Without this test, a future `is_date_like` refactor that switches to `str::starts_with` or a prefix regex would accidentally match `'2026-06-24extra'`, silently treating sensor IDs and report labels as temporal literals. This test anchors the full-consumption contract.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-035 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_projection_group_by_date_like_coerces`

**Location:** `crates/prism-query/src/tests/`

**Note:** Function name unchanged per append-only naming policy; "coerces" is a historical artifact from v1.6. Behavior FLIPPED in v1.8 per ADR-052 §D4 v1.10 / error-taxonomy v2.14.

**Pre-implementation state (v1.8):** compile error — `Literal::RawTemporalLiteral` and `PrismError::TemporalLiteralInvalidPosition` variants do not exist.

**Post-implementation state (ADR-052 §D4 v1.10 — GROUP BY reject; error-taxonomy §E-QUERY-042):**
- Query: `SELECT count(*) FROM t GROUP BY '2026-06-24'` (date-like literal in SQL GROUP BY)
- Parser emits `Literal::RawTemporalLiteral("2026-06-24")` in the GROUP BY clause
- `check_temporal_literals` finds the literal in GROUP BY position → REJECT:
  `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::GroupBy, value_prefix: "2026-06-24".into() })`
- `map_prism_error` → `codes::INVALID_PARAMS` (-32602)
- Query FAILS with E-QUERY-042 — NOT success, NOT -32000

**Why load-bearing:** GROUP BY with date-like literals is a structurally invalid position per ADR-052 §D4 v1.10. Coercing it would silently produce semantically incorrect queries (grouping by a string constant is almost always an analyst mistake). The reject anchors the semantic boundary between "safe coerce" (SELECT projection) and "structural error" (GROUP BY/ORDER BY).

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-036 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_order_by_date_like_coerces`

**Location:** `crates/prism-query/src/tests/`

**Note:** Function name unchanged per append-only naming policy; "coerces" is a historical artifact from v1.6. Behavior FLIPPED in v1.8 per ADR-052 §D4 v1.10 / error-taxonomy v2.14.

**Pre-implementation state (v1.8):** compile error — `Literal::RawTemporalLiteral` and `PrismError::TemporalLiteralInvalidPosition` variants do not exist.

**Post-implementation state (ADR-052 §D4 v1.10 — ORDER BY reject; error-taxonomy §E-QUERY-042):**
- Query: `SELECT * FROM t ORDER BY '2026-06-24'` (date-like literal in SQL ORDER BY)
- Parser emits `Literal::RawTemporalLiteral("2026-06-24")` in the ORDER BY clause
- `check_temporal_literals` finds the literal in ORDER BY position → REJECT:
  `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::OrderBy, value_prefix: "2026-06-24".into() })`
- `map_prism_error` → `codes::INVALID_PARAMS` (-32602)
- Query FAILS with E-QUERY-042 — NOT success, NOT -32000

**Why load-bearing:** ORDER BY with date-like literals is a structurally invalid position. Together with RG-035 (GROUP BY reject), this test anchors that the reject arm covers BOTH ordering positions in SQL mode. The SELECT projection coerce arm (RG-023) remains UNCHANGED — only GROUP BY and ORDER BY are REJECTED in SQL mode.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-037 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_stats_by_date_like_e_query_001`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `todo!()` panic — `filter_parser.rs` does not yet reject literals in `stats by` position.

**Post-implementation state (ADR-052 §D4 v1.10 — pipe parse-time rejection):**
- Query: `FROM t | stats count by '2026-06-24'` (pipe-mode `stats by` with bare date-like literal)
- `filter_parser.rs` rejects the literal AT PARSE TIME with enhanced E-QUERY-001:
  message includes "`stats by` only accepts field paths, not literal values"
- Error code is E-QUERY-001 — NOT E-QUERY-042 (this is a parse/syntax error, not plan/semantic)
- `check_temporal_literals` is NEVER invoked — the query is rejected before AST production
- The test asserts: (a) error is E-QUERY-001; (b) error occurs before plan time (parse-time flag or absence of AST); (c) error message mentions "`stats by`" and "field path"

**Why load-bearing:** Anchors that `stats by` bare literals fail at PARSE TIME (E-QUERY-001), not at plan time (E-QUERY-042). If a future implementer allows `stats by` literals through the parser and instead handles them in `check_temporal_literals`, the error code changes from E-QUERY-001 to E-QUERY-042 — this test pins the expected error code and stage.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-038 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sort_date_like_e_query_001`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `todo!()` panic — `filter_parser.rs` does not yet reject literals in `sort` position.

**Post-implementation state (ADR-052 §D4 v1.10 — pipe parse-time rejection):**
- Query: `FROM t | sort '2026-06-24'` (pipe-mode `sort` with bare date-like literal)
- `filter_parser.rs` rejects AT PARSE TIME with enhanced E-QUERY-001:
  message includes "`sort` only accepts field paths, not literal values"
- Error code is E-QUERY-001 — NOT E-QUERY-042
- `check_temporal_literals` never invoked

**Why load-bearing:** Symmetric with RG-037 for the `sort` clause. The pipe grammar has two clause types (grouping via `stats by`, sorting via `sort`) that should both reject bare literals at parse time. This test ensures `sort` is not accidentally left accepting literals while `stats by` rejects them.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

### RG-039 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_non_column_lhs_date_like_e_query_042`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** compile error — `PrismError::TemporalLiteralInvalidPosition` and `Literal::RawTemporalLiteral` variants do not exist.

**Post-implementation state (ADR-052 §D4 v1.10 — non-column-LHS → E-QUERY-042; error-taxonomy §E-QUERY-042):**
- Query: `SELECT * FROM t WHERE lower(hostname) = '2026-06-24'` (function call LHS, date-like RHS)
- Parser produces `Literal::RawTemporalLiteral("2026-06-24")` on the RHS
- `check_temporal_literals` detects the literal in a comparison where the LHS is a non-column
  expression (function call `lower(hostname)`) → REJECT:
  `Err(PrismError::TemporalLiteralInvalidPosition { position: TemporalInvalidPosition::NonColumnLhsComparison, value_prefix: "2026-06-24".into() })`
- `map_prism_error` → `codes::INVALID_PARAMS` (-32602) — NOT -32000

The test MUST verify:
- Error IS `TemporalLiteralInvalidPosition::NonColumnLhsComparison`
- Error maps to INVALID_PARAMS, NOT INTERNAL_ERROR
- This is DISTINCT from the column-LHS path:
  `WHERE ts_col = '2026-06-24'` (plain column LHS) → E-QUERY-041 via column-type resolution

**Why load-bearing:** Prevents `-32000 INTERNAL_ERROR` from being returned for a structurally
identifiable analyst error. The non-column-LHS detection distinguishes a resolvable guidance error
from an unexpected internal failure. `-32000` would mislead the MCP caller into thinking something
broke server-side rather than telling them their query is malformed.

**SID-1 compliance:** in-process, deterministic — no `#[ignore]`.

## Behavioral Contracts

| BC | Title | Role in this story |
|----|-------|--------------------|
| BC-2.11.021 | Temporal Grammar — `NOW()` and `INTERVAL` Planning-Time Constant Injection | Amended §Postconditions per ADR-052 §D4 v1.10: emitter uses `arrow_cast(...)` form; E-QUERY-041 via Option-A `check_temporal_literals` AST walker + refined dispatch; 7-form is_date_like acceptance set; EC-11-021-010..014 added; SELECT projection COERCEs to Literal::String; GROUP BY/ORDER BY (SQL) → E-QUERY-042; non-column-LHS comparison → E-QUERY-042; pipe stats-by/sort → parse-time E-QUERY-001. AC-003, AC-004, AC-005, AC-025, AC-026, AC-027, AC-028, AC-029, AC-030, AC-031, AC-032, AC-033, AC-034 trace here. |
| BC-2.11.003 | PrismQL SQL Mode Parsing | Amended: ADR-052 D2 assertion — `Timestamp(Microsecond, Some("UTC"))`; Option-A E-QUERY-041 mechanism; coercion arm for String/Utf8 columns. AC-001, AC-005, AC-007, AC-019 trace here. |
| BC-2.11.004 | PrismQL Pipe Mode Parsing | Same D2 assertion for pipe `| where` stages; Option-A in pipe mode. AC-001, AC-005, AC-007, AC-019 trace here. |
| BC-2.11.001 | `query` MCP Tool Accepts Scoping + PrismQL Query String | Governs the query pipeline; E-QUERY-041 gate ordering; `map_prism_error` -32602 constraint. AC-006 traces here. |

## Subsystem Anchor Justifications

Per `architecture/ARCH-INDEX.md` Subsystem Registry:
- **SS-09** (Sensor Adapters): owns `crates/prism-bin/src/spec_driven_adapter.rs` (column type registration + sensor datetime parsing). SS-09 owns all sensor adapter code.
- **SS-10** (OCSF Normalization): owns the ISO-8601 → Timestamp conversion at the data ingest boundary (AC-013). SS-10 owns all OCSF normalization code.
- **SS-11** (Query Engine): owns `crates/prism-query/src/` — `ast.rs` (RawTemporalLiteral variant), `sql_parser.rs`/`filter_parser.rs` (lenient fallback), `engine.rs` (`check_temporal_literals` AST walker), `pipe_sql_emitter.rs` (emitter + guard arm), `pushdown.rs` (verify no change), `high002_plan_pinning_tests.rs` (plan stability). SS-11 owns all PrismQL query processing code.

## Architecture Mapping

| Component | Module | Pure/Effectful | Change |
|-----------|--------|---------------|--------|
| `column_type_to_arrow` function | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (ColumnType → DataType mapping) | D2 CHANGE: Datetime arm → Timestamp |
| `parse_datetime_to_micros` helper | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (string → i64, may error) | D5 ADD: ISO-8601 → microseconds |
| `Literal::RawTemporalLiteral(String)` | `crates/prism-query/src/ast.rs` | Pure (AST node definition) | D4 ADD: new variant; §D4 Step 1 doc comment |
| `is_date_like` helper | `crates/prism-query/src/sql_parser.rs` | Pure (string → bool) | D4 ADD: `NaiveDate` + `NaiveDateTime` heuristic |
| Parser lenient fallback | `crates/prism-query/src/sql_parser.rs` (+ `filter_parser.rs`) | Pure (parse combinator) | D4 CHANGE: RFC-3339→Timestamp; is_date_like→RawTemporalLiteral; else→String |
| `check_temporal_literals` function | `crates/prism-query/src/engine.rs` | Pure (AST walker + schema resolution; coercion mutates AST in-place) | D4 ADD: four-way dispatch; inserted after `check_enrich_udf_availability` |
| Text-scanner functions (DELETED) | `crates/prism-query/src/engine.rs` | N/A | D4 DELETE: `extract_table_name_from_query_str`, `extract_column_name_adjacent_to_quoted_value`, `is_bad_literal_in_datetime_column`, parse-fail branch (if present in workspace) |
| `PrismError::TemporalLiteralUnparseable` | `crates/prism-core/src/error.rs` | Pure (error type definition) | D4 ADD: new variant (E-QUERY-041) |
| `PrismError::TemporalLiteralInvalidPosition` + `TemporalInvalidPosition` | `crates/prism-core/src/error.rs` | Pure (error type definitions) | D4 ADD: new variant + enum (E-QUERY-042; positions: GroupBy/OrderBy/NonColumnLhsComparison) |
| `map_prism_error` arms | `crates/prism-mcp/src/error_mapping.rs` | Pure (error code mapping) | D4 ADD: `codes::INVALID_PARAMS` arms for both E-QUERY-041 and E-QUERY-042 variants |
| `pipe_sql_emitter.rs` `Literal::Timestamp` | `crates/prism-query/src/pipe_sql_emitter.rs` | Pure (AST → SQL string) | D3 CHANGE: `arrow_cast(...)` form |
| `pipe_sql_emitter.rs` `Literal::RawTemporalLiteral` guard | `crates/prism-query/src/pipe_sql_emitter.rs` | Pure (returns Err) | D4 ADD: E-QUERY-002 (`QueryPlanFailed`) belt-and-suspenders guard arm |
| `pushdown.rs` T1 extractor | `crates/prism-query/src/pushdown.rs` | Pure (AST → string extract) | D5 VERIFY: no change; also verify `RawTemporalLiteral` handled (no-pushdown arm) |
| `infusion_udf.rs` `return_type` | `crates/prism-query/src/infusion_udf.rs` | Pure (always returns Utf8, stub) | D8 VERIFY: unconditional stub; ADR-051 introduces per-type mapping |
| `column.rs` doc comment | `crates/prism-core/src/column.rs` | Pure (enum definition) | D2 CHANGE: doc comment only |
| ADR-044 frontmatter | `.factory/specs/architecture/decisions/ADR-044-*.md` | N/A (factory spec) | D7 VERIFY: superseded_by already present (architect pre-completed) |

## Edge Cases

| ID | Description | Expected Behavior | BC Anchor |
|----|-------------|-------------------|-----------|
| EC-001 | `WHERE timestamp > '2026-06-24'` (date-only, Datetime col) | E-QUERY-041 via `check_temporal_literals` (parser emits `RawTemporalLiteral`; AST walker resolves Timestamp col → E-QUERY-041) | BC-2.11.003 §Edge Cases EC-11-003-001, BC-2.11.004 §Edge Cases EC-11-004-001, BC-2.11.021 §Postconditions |
| EC-002 | `WHERE timestamp > '2026-06-24T00:00:00Z'` (valid RFC-3339 with `Z`) | Parser emits `Literal::Timestamp`; `check_temporal_literals` not invoked; succeeds | BC-2.11.003 §Edge Cases EC-11-003-002, BC-2.11.004 §Edge Cases EC-11-004-002 |
| EC-003 | `WHERE timestamp > '2026-06-24T00:00:00+00:00'` (valid RFC-3339 with `+00:00`) | Same as EC-002 — `parse_from_rfc3339` accepts `+00:00` form | BC-2.11.021 §Postconditions |
| EC-004 | `WHERE timestamp > NOW() - INTERVAL '24h'` (normal temporal predicate) | Injects `arrow_cast(...)` typed literal → Timestamp-vs-Timestamp comparison | BC-2.11.021 §Error Cases EC-11-021-001 |
| EC-005 | `WHERE timestamp > 'yesterday'` (non-date-like free-text, Datetime col) | `is_date_like` returns false → `Literal::String("yesterday")` → DataFusion plan-time type-mismatch E-QUERY-002 (`QueryPlanFailed`) (not E-QUERY-041) | error-taxonomy §E-QUERY-041 "Non-date-like forms" |
| EC-006 | `WHERE timestamp > '2026-06-24T12:00:00'` (missing UTC offset, Datetime col) | Parser emits `RawTemporalLiteral` (`NaiveDateTime` pattern); `check_temporal_literals` → E-QUERY-041 | error-taxonomy §E-QUERY-041 "Invalid forms" |
| EC-007 | Sensor API returns ISO-8601 datetime string with `+00:00` offset | `parse_datetime_to_micros` via `parse_from_rfc3339` handles both `Z` and `+00:00` | ADR-052 D5 (identical chrono strictness) |
| EC-008 | `diff_results` CF contains old Utf8-typed Arrow IPC bytes after upgrade | Startup migration step clears CF; no schema mismatch crash | ADR-052 D6 / RISK-3 mitigation |
| EC-009 | `value_prefix` from a 100-char offending literal | `value_prefix` truncated at UTF-8 codepoint boundary ≤ 50 chars (via `char_indices()`, NOT raw byte slicing) | error-taxonomy §E-QUERY-041 (AD-017 / E-INFUSE-014 truncation convention) |
| EC-010 | `WHERE string_col = '2026-06-24'` (date-like literal, String/Utf8 col) | `check_temporal_literals` resolves String col → COERCE `RawTemporalLiteral` → `Literal::String("2026-06-24")` in-place; query SUCCEEDS; emitted SQL byte-identical to pre-ADR-052 | ADR-052 §D4 coercion arm (RISK-5 RESOLVED BY DESIGN) |
| EC-011 | `WHERE int_col = '2026-06-24'` / `float_col = '2026-06-24'` / `bool_col = '2026-06-24'` | `check_temporal_literals` resolves numeric/bool col → E-QUERY-002 (`QueryTypeMismatch`) (NOT E-QUERY-041) | ADR-052 §D4 Step 3 three-way dispatch third arm |
| EC-012 | `'not-a-date'` or `'sensor-id-abc'` in any position | `is_date_like` returns false; parser emits `Literal::String`; `check_temporal_literals` ignores it; no temporal error | ADR-052 §D4 Step 2 (heuristic negative case) |
| EC-013 | Dotted source `source.ts_col > '2026-06-24'` where `ts_col` is Datetime | `check_temporal_literals` resolves via schema (NOT string split on `.`); correct Datetime type → E-QUERY-041; NOT E-QUERY-037 | ADR-052 §D4 Step 3 "Dotted expressions resolved via schema map" |
| EC-014 | Non-ASCII / multi-byte Unicode characters in query near a date-like literal | Parser produces valid UTF-8 strings; `check_temporal_literals` uses codepoint-safe `char_indices()` truncation; NO panic | VP-021; ADR-052 §D4 Step 3 "Unicode inputs: no raw byte-offset operations" |
| EC-015 | `WHERE timestamp > '2026-06-24T12:00:00.123'` (T-sep fractional — form 3 of `is_date_like`, Datetime col) | `Err(E-QUERY-041)`: parser emits `RawTemporalLiteral` via `%Y-%m-%dT%H:%M:%S%.f`; `check_temporal_literals` resolves Datetime col → E-QUERY-041 | BC-2.11.021 §Error Cases EC-11-021-011; ADR-052 §D4 form 3 |
| EC-016 | `WHERE timestamp > '2026-06-24T12:00'` (T-sep no-seconds — form 4, Datetime col) | `Err(E-QUERY-041)`: parser emits `RawTemporalLiteral` via `%Y-%m-%dT%H:%M`; → E-QUERY-041 | BC-2.11.021 §Error Cases EC-11-021-010; ADR-052 §D4 form 4 |
| EC-017 | `WHERE timestamp > '2026-06-24 12:00:00'` (space-sep full seconds — form 5, Datetime col) | `Err(E-QUERY-041)`: parser emits `RawTemporalLiteral` via `%Y-%m-%d %H:%M:%S`; → E-QUERY-041 | BC-2.11.021 §Error Cases EC-11-021-012; ADR-052 §D4 form 5 |
| EC-018 | `WHERE timestamp > '2026-06-24 12:00:00.500'` (space-sep fractional — form 6, Datetime col) | `Err(E-QUERY-041)`: parser emits `RawTemporalLiteral` via `%Y-%m-%d %H:%M:%S%.f`; → E-QUERY-041 | ADR-052 §D4 form 6 |
| EC-019 | `WHERE timestamp > '2026-06-24 12:00'` (space-sep no-seconds — form 7, Datetime col) | `Err(E-QUERY-041)`: parser emits `RawTemporalLiteral` via `%Y-%m-%d %H:%M`; → E-QUERY-041 | ADR-052 §D4 form 7 |
| EC-020 | `WHERE string_col = '2026-06-24 12:00:00'` (space-sep form vs String/Utf8 col) | COERCE: `check_temporal_literals` rewrites `RawTemporalLiteral` → `Literal::String`; query SUCCEEDS; byte-identical to pre-ADR-052 (RISK-5 extension to space-sep family) | BC-2.11.021 §Error Cases EC-11-021-013; ADR-052 §D4 coercion arm |
| EC-021 | `WHERE timestamp > '2026-6-24'` (unpadded month/day, over-match ACCEPTED BENIGN, Datetime col) | `Err(E-QUERY-041)`: `is_date_like` matches via `%Y-%m-%d` (chrono accepts single digits); "use RFC-3339" message is accurate; no regex guard applied | BC-2.11.021 §Error Cases EC-11-021-014; ADR-052 §D4 over-match ACCEPTED BENIGN |
| EC-022 | `WHERE string_col = '2026-6-24'` (unpadded, over-match, String/Utf8 col) | COERCE → SUCCESS; byte-identical to pre-ADR-052; unpadded date labels in String cols are legitimate | ADR-052 §D4 coercion arm + over-match disposition |
| EC-023 | `'2026-06-24extra'` (trailing chars, near-miss) in any query position | `is_date_like` returns `false` (chrono `parse_from_str` requires full consumption — leftover `extra` causes `Err`); parser emits `Literal::String`; no temporal error | ADR-052 §D4 `is_date_like` negative boundary (full-consumption property) |
| EC-024 | `SELECT '2026-06-24' FROM t` (date-like literal in projection position, no column comparison) | `check_temporal_literals` finds `RawTemporalLiteral` in non-comparison context → COERCE to `Literal::String("2026-06-24")` → query SUCCEEDS; emitted as `'2026-06-24'` string constant | ADR-052 §D4 (human-ratified 2026-07-05); BC-2.11.021 §Postconditions non-comparison coerce arm |
| EC-025 | `SELECT count(*) FROM t GROUP BY '2026-06-24'` (date-like literal in SQL GROUP BY) | `check_temporal_literals` → `Err(PrismError::TemporalLiteralInvalidPosition { position: GroupBy })` (E-QUERY-042); `map_prism_error` → INVALID_PARAMS — NOT -32000 | ADR-052 §D4 v1.10; BC-2.11.021 §Postconditions GROUP BY reject arm; error-taxonomy §E-QUERY-042 |
| EC-025b | `SELECT * FROM t ORDER BY '2026-06-24'` (date-like literal in SQL ORDER BY) | `check_temporal_literals` → `Err(PrismError::TemporalLiteralInvalidPosition { position: OrderBy })` (E-QUERY-042); INVALID_PARAMS | ADR-052 §D4 v1.10; error-taxonomy §E-QUERY-042 |
| EC-026 | `FROM t \| stats count by '2026-06-24'` or `FROM t \| sort '2026-06-24'` (pipe bare literal in stats-by/sort clause) | `filter_parser.rs` rejects AT PARSE TIME with enhanced E-QUERY-001 ("`stats by` / `sort` only accepts field paths"); `check_temporal_literals` never reached | ADR-052 §D4 v1.10; BC-2.11.004 §Postconditions |
| EC-027 | `WHERE lower(hostname) = '2026-06-24'` (non-column-LHS comparison: function expression LHS, date-like literal RHS) | `check_temporal_literals` → `Err(PrismError::TemporalLiteralInvalidPosition { position: NonColumnLhsComparison })` (E-QUERY-042); INVALID_PARAMS — NOT -32000; NOT E-QUERY-041 (that requires plain-column LHS resolving to Datetime) | ADR-052 §D4 v1.10; error-taxonomy §E-QUERY-042 |

## Known Limitations

**RISK-2: Two-representation transition window (ADR-052 §Risk RISK-2, MEDIUM)**

Between this story merging and the ADR-051 typed-enrichment story shipping, enrichment
`output_type = "datetime"` fields remain mapped to `DataType::Utf8` in `infusion_udf.rs`
(the function is a stub that always returns Utf8 — no per-output_type mapping exists yet).
This is inconsistent but not a regression — it preserves pre-existing behavior for enrichment
fields. ADR-051 will INTRODUCE the per-output_type datetime→Timestamp mapping, not amend an
existing row.

## Estimated Complexity

8 story points. Rationale: 10 source files modified (ast.rs, sql_parser.rs, filter_parser.rs,
engine.rs, pipe_sql_emitter.rs + sibling sweep, spec_driven_adapter.rs, column.rs, error.rs,
error_mapping.rs, high002_plan_pinning_tests.rs) + OCSF normalization addition in prism-sensors +
1 factory spec VERIFY (ADR-044 pre-done) + 39 Red Gate tests including full Option-A coverage
(parser/AST/walker/coercion/guard/sibling-sweep/unicode) + E-QUERY-042 positions (GROUP BY,
ORDER BY, non-column-LHS) + pipe stats-by/sort parse-time E-QUERY-001 + DataFusion RISK-1
arrow_cast probe + emitter E2E integration test (RG-010) + sensor normalization +
diff_results CF investigation + `map_prism_error` arms (E-QUERY-041 and E-QUERY-042) +
TD-VSDD-060 Literal sibling-sweep. No new crates. Story originally estimated at 2 days;
revised to 3 days due to Option-A redesign and deep Red Gate mandate.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.8 | S-PRISMQL-NATIVE-TEMPORAL-TYPING-001-D4-v1.10-refined-dispatch | 2026-07-05 | story-writer | **Refined non-comparison dispatch per ADR-052 §D4 v1.10 / error-taxonomy v2.14 (E-QUERY-042).** RG-035 (GROUP BY) and RG-036 (ORDER BY) FLIPPED from coerce success to E-QUERY-042 (`TemporalLiteralInvalidPosition::GroupBy/OrderBy`, INVALID_PARAMS) reject. RG-023 (SELECT projection coerce) UNCHANGED. THREE new RGs added: RG-037 (pipe `stats by` → parse-time E-QUERY-001), RG-038 (pipe `sort` → parse-time E-QUERY-001), RG-039 (non-column-LHS comparison → E-QUERY-042 `NonColumnLhsComparison`, INVALID_PARAMS). `red_gate_tests` 36→39. **New ACs:** AC-030 (GROUP BY → E-QUERY-042 GroupBy), AC-031 (ORDER BY → E-QUERY-042 OrderBy), AC-032 (pipe stats-by → parse E-QUERY-001), AC-033 (pipe sort → parse E-QUERY-001), AC-034 (non-column-LHS → E-QUERY-042 NonColumnLhsComparison); AC-029 narrowed to SELECT-projection-only coerce. **Spec additions:** §D4 dispatch table expanded to 9 rows; E-QUERY-042 `TemporalLiteralInvalidPosition` + `TemporalInvalidPosition` enum added to Task 6 + File Structure + Architecture Mapping; `map_prism_error` arm for E-QUERY-042 added to Task 7; Task 13 extended with pipe parse-time rejection; Task 14 dispatch logic updated (step 1 now 4-branch with non-column-LHS + GROUP BY + ORDER BY + projection). EC-025 updated (was coerce → now reject); EC-025b, EC-026, EC-027 added. Architecture Compliance Rules: 3 new rows for E-QUERY-042 discipline. Token budget ~120k→~122k. Frontmatter BC comment updated for §D4 v1.10. Stubs aj/ak descriptions updated (flip to reject; names unchanged per append-only policy); stubs al/am/an added. |
| 1.7 | S-PRISMQL-NATIVE-TEMPORAL-TYPING-001-LOW1-depinning | 2026-07-05 | story-writer | **LOW-1: de-pin stale anchored-artifact version cites (TD-VSDD-091).** Removed all vX.Y version pins from live narrative (frontmatter comments, Background table, Token Budget table, Tasks, Architecture Compliance Rules, AC headings/Traces, Red Gate post-implementation states, Behavioral Contracts table, Edge Cases table, Known Limitations). Replaced with ID + behavioral anchor form (e.g., BC-2.11.003 §Postconditions, ADR-052 §D4, error-taxonomy §E-QUERY-041). Story own version, changelog entries, and Red Gate AC↔test source-of-truth tables left intact per constraint. Behavioral Contracts table: Version column removed entirely (not needed — BCs are cited by §Postconditions/§Error Cases/§Edge Cases anchors in AC traces). |
| 1.6 | S-PRISMQL-NATIVE-TEMPORAL-TYPING-001-OBS2-noncomparison-coerce | 2026-07-05 | story-writer | **OBS-2 human-ratified non-comparison coerce + OBS-1 Literal::Utf8→String naming; align ADR-052 §D4 v1.8 / BC-2.11.021 v1.6.** OBS-2 (behavior change): `RawTemporalLiteral` in a non-comparison position (SELECT projection, GROUP BY, ORDER BY, function arg) now COERCES to `Literal::String(s)` → query SUCCEEDS, instead of returning E-QUERY-002/QueryPlanFailed. Changes: (1) §D4 Step 3 dispatch table row 4 updated (E-QUERY-002 → COERCE); (2) `THREE-WAY dispatch` → `FOUR-WAY dispatch` in §D4 heading, narrative, decision table D4, Task 14 heading, File Structure table, Architecture Mapping table, BC table; (3) Task 14 implementation logic updated: "If NO" branch now describes COERCE; last bullet replaced with non-comparison coerce; (4) Option-A dispatch table: new non-comparison COERCE row added; (5) Stub x updated (E-QUERY-002 → SUCCESS+coerce); (6) Stubs aj and ak added (GROUP BY and ORDER BY coerce success); (7) RG-023 post-implementation state updated (E-QUERY-002 → SUCCESS+coerce); (8) RG-035 and RG-036 added (GROUP BY, ORDER BY coerce success); (9) AC-029 added (non-comparison coerce success); (10) EC-024 and EC-025 added; (11) red_gate_tests 34 → 36; (12) Token budget story spec ~16k→~17.5k, total ~118.5k→~120k. OBS-1 (naming fix): ALL `Literal::Utf8` references in story body replaced with `Literal::String` (19 instances: §D4 Step 2 pseudocode, Step 2 text, Option-A dispatch table, Task 13 comment+code, Task 5 stub s, stub ai, Architecture Compliance Rules, File Structure table, AC-021 heading+body+traces, AC-028 heading+body+traces, RG-018, RG-026 why, RG-027 why, RG-034 body, EC-005, EC-012, EC-023). Changelog/historical references to `Literal::Utf8` (v1.3 changelog) left intact per constraint. Comparison-arm RG vectors (Datetime→E-QUERY-041: RG-004/005/012/025/026/027/028/029/030/032; numeric→E-QUERY-002: RG-015/016/017) are UNCHANGED. |
| 1.5 | S-PRISMQL-NATIVE-TEMPORAL-TYPING-001-F-P5-MED-2 | 2026-07-04 | product-owner | **F-P5-MED-2 spec-code drift correction: E-QUERY-001 references in the Integer/Float/Bool dispatch arm, emitter guard arm, and non-comparison position arm corrected to E-QUERY-002 (`QueryTypeMismatch` / `QueryPlanFailed`).** The v1.4 spec used `PrismError::InvalidQuery` (non-existent variant) and E-QUERY-001 for these three paths. The implementation correctly uses `PrismError::QueryTypeMismatch { column, table, actual_type, operator }` (E-QUERY-002) for the Integer/Float/Bool arm and `PrismError::QueryPlanFailed { detail }` (E-QUERY-002) for the emitter guard and non-comparison position. **Spec changes:** (1) Three-way dispatch table rows 3+4 updated; (2) Step 5 emitter guard description updated; (3) Option-A dispatch table header renamed (was "E-QUERY-001 ↔ E-QUERY-041 boundary"); (4) Task 11B code example updated to `QueryPlanFailed`; (5) Task 14 implementation guidance updated to `QueryTypeMismatch` and `QueryPlanFailed`; (6) Task 15 TD-VSDD-060 guidance updated; (7) AC-020 and AC-022 headings + bodies updated; (8) D3 decision table updated; (9) RG-015/016/017 post-implementation states updated; (10) RG-023/024 post-implementation states updated; (11) EC-005/EC-011 expected behavior updated; (12) Pre-implementation "would have" comments in RG-013/RG-031 updated; (13) Error taxonomy reference updated to v2.12. Story version bumped 1.4→1.5. |
| 1.4 | ADR-052-story-v1.4-is-date-like-7-form-rg-boundary | 2026-07-04 | story-writer | **ADR-052 §D4 v1.4 is_date_like 7-form set — Red Gate boundary vectors added.** `is_date_like` description in Background §D4 Step 2 and Task 13 expanded from 2-form (date-only + T-sep full seconds) to 7-form (+ T-sep fractional form 3, T-sep no-seconds form 4, space-sep full seconds form 5, space-sep fractional form 6, space-sep no-seconds form 7) per ADR-052 §D4 v1.4 canonical acceptance set. E-QUERY-001↔E-QUERY-041 boundary table expanded from 6 rows to 15 rows (new forms 3-7 vs Datetime, space-sep String coerce, over-match benign variants, near-miss negative). Task 5 stubs aa-ai added (9 new stubs: RG-026 through RG-034). Added AC-025 (forms 3-7 vs Datetime → E-QUERY-041, 5 RG tests), AC-026 (space-sep String col coercion → COERCE, 1 RG test), AC-027 (unpadded over-match accepted benign, 2 RG tests), AC-028 (near-miss trailing chars → Utf8, 1 RG test). Added RG-026 through RG-034 (9 new Red Gate entries). Added EC-015 through EC-023 (9 new edge case rows). Frontmatter: version 1.3→1.4, red_gate_tests 25→34, BC-2.11.021 version reference v1.3→v1.4. Token budget story spec ~14,000→~16,000, total ~116,500→~118,500. Estimated complexity: 25→34 Red Gate count. Task 25 run command count 25→34. Compile state note updated: TemporalLiteralUnparseable compile failures extended to RG-025 through RG-030 and RG-032; RawTemporalLiteral compile failures extended to RG-031 and RG-033; RG-034 is todo!() panic. |
| 1.3 | ADR-052-story-v1.3-option-a-deep-red-gate | 2026-07-04 | story-writer | **ADR-052 §D4 v1.3 Option-A ratification applied. Human deep-testing mandate applied.** Deleted 4 text-scanner task items (the parse-fail branch and text-scanner functions `extract_table_name_from_query_str`, `extract_column_name_adjacent_to_quoted_value`, `is_bad_literal_in_datetime_column` from the v1.2 Task 12 description). Added 4 new Option-A tasks: (12) `Literal::RawTemporalLiteral` in ast.rs; (13) parser lenient fallback with `is_date_like` heuristic; (14) `check_temporal_literals` AST walker with three-way dispatch + String-column coercion arm + text-scanner deletion; (15) TD-VSDD-060 Literal sibling-sweep. Updated Task 11 to add `RawTemporalLiteral` guard arm in pipe_sql_emitter.rs. Renumbered old Tasks 13-23 to 16-26. Task 5 stubs expanded a-j → a-z (16 new stubs for RG-011 through RG-025 + AC-018 parser test). AC-005 updated to describe Option-A mechanism (NOT chrono pre-validator on string literals). Added AC-017 through AC-024 (8 new ACs: RawTemporalLiteral variant, parser lenient emit, String-col coercion success, Integer/Float/Bool E-QUERY-001, non-date-like stays Utf8, emitter guard, text-scanner absence, unicode no-panic). RG-004 and RG-005 mechanism description updated: parse SUCCEEDS now, RawTemporalLiteral AST node, check_temporal_literals fires (not chrono pre-validator on string literals). RG-007 mechanism updated: full RFC-3339 → Literal::Timestamp (not RawTemporalLiteral) → check_temporal_literals not invoked. Added RG-011 through RG-025 (15 new RG tests covering: full RFC-3339 regression guard, offset-less vs Datetime col, String-col coercion date-only, String-col coercion offset-less, Integer/Float/Bool E-QUERY-001, non-date-like stays string, dotted source resolution, qualified nested column, filter pipe syntax, unicode no-panic, projection position, emitter guard, E-QUERY-041 message format byte-identical). red_gate_tests 10 → 25. Added EC-010 through EC-014 (coercion success, numeric mismatch, non-date-like negative, dotted column, unicode safety). Added ast.rs, sql_parser.rs, filter_parser.rs to File Structure MODIFY list and Architecture Mapping table. Background §D4 entirely rewritten for Option-A. Token budget updated (new files; story spec v1.2→v1.3 estimate increase; total ~78,500 → ~116,500). BC version refs updated: BC-2.11.021 v1.3, BC-2.11.003 v1.7, BC-2.11.004 v1.8, error-taxonomy v2.11. estimated_days "2" → "3". |
| 1.2 | ADR-052-story-decomposition-v1.2 | 2026-07-03 | story-writer | Applied remove-uncertainty PASS-2 corrections. AC-015/Task 20 re-scoped CHANGE→VERIFY. AC-006/Task 7: symbolic constant `codes::INVALID_PARAMS`. Task 12: pinned pre-validator host to engine.rs; named `check_temporal_literals`; corrected gate attribution. RG-010 added. red_gate_tests 9→10. |
| 1.1 | ADR-052-story-decomposition-v1.1 | 2026-07-03 | story-writer | Applied remove-uncertainty PASS-1 corrections. C1 RG-008 wrong year: fixed to chrono derivation. H3 wrong Arc construction: `Arc::new("UTC".into())` → `Arc::from("UTC")`. Detection architecture: E-QUERY-041 as Prism chrono pre-validator (not DataFusion cast intercept). Emitter form: `TIMESTAMP '...'` → `arrow_cast(...)`. RISK-1 downgraded HIGH→MEDIUM. |
| 1.0 | ADR-052-story-decomposition | 2026-07-03 | story-writer | Initial story — decomposed from ADR-052 v1.0 + amended BCs. 16 ACs, 9 Red Gate tests. |
