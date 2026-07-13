---
document_type: adr
adr_id: "ADR-052"
title: "PrismQL Native Temporal Typing — Datetime Columns and Literals from Arrow Utf8 to Timestamp(Microsecond, UTC)"
status: accepted
date: "2026-07-03"
version: "1.12"
producer: architect
subsystems_affected: [SS-09, SS-10, SS-11, SS-17]
supersedes: "ADR-044 §D4"
superseded_by: null
amends: null
anchor_stories: []
related_adrs: [ADR-024, ADR-033, ADR-040, ADR-043, ADR-044, ADR-051]
related_bcs: [BC-2.11.021, BC-2.11.003, BC-2.11.004, BC-2.11.001]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-052: PrismQL Native Temporal Typing — Datetime Columns and Literals from Arrow Utf8 to Timestamp(Microsecond, UTC)

## Status

ACCEPTED v1.12 (2026-07-13). Human ratification of core decisions recorded 2026-07-03
(D-1520). **§D4 v1.3 ACCEPTED (human-ratified 2026-07-04, Option A + String-column
coercion modification). v1.4 (pre-TDD, remove-uncertainty): `is_date_like` acceptance
set expanded from 2 to 7 format strings (date-only + T-sep/space-sep × full-seconds/
fractional/no-seconds); over-match disposition (unpadded digits, big/signed years)
documented as ACCEPTED BENIGN.**

**v1.5 (LOCAL adversary cascade fix-burst, 2026-07-04):** Three §D4 corrections: (A)
E-QUERY-001→E-QUERY-002 for `RawTemporalLiteral` against Integer/Float/Bool columns
and non-comparison positions — `QueryTypeMismatch`/`QueryPlanFailed` are the shipped
error variants (error-taxonomy.md v2.12 source of truth); (B) `check_temporal_literals`
DEFINED in `materialization.rs`, not `engine.rs` — `engine.rs` retains the early-gate
INVOCATION; (C) SQL-mode DataFusion emission path (`materialization.rs::execute_against_session`
`Ast::Sql` arm → `PqlNormalizer::normalize_literal`) also requires `arrow_cast(...)` for
`Literal::Timestamp` — currently emits bare `'{iso8601}'` violating BC-2.11.021; dedicated
`emit_literal_for_datafusion` function proposed as resolution in §D4 addendum. Blast radius
expanded to 22 rows.

**v1.6 (LOCAL adversary cascade spec↔code alignment, 2026-07-04):** Two §D4 SQL-mode
emission corrections. (FIX 1 / LOW-2) §D4 SQL-Mode Emission Addendum corrected to document
the SHIPPED mechanism: v1.5 described a phantom `emit_literal_for_datafusion` free function
in `materialization.rs`; the actual implementation uses a thread-local scoped-context in
`ast.rs` — `NORMALIZE_FOR_DATAFUSION: Cell<bool>` set by `PqlNormalizer::normalize_for_datafusion`
with save-and-restore drop-guard; `normalize_literal_dispatch` routes `Literal::Timestamp` to
`normalize_literal_for_datafusion` (arrow_cast emitter) within the context; `PqlNormalizer::normalize_literal`
(BC-2.11.018 round-trip) unchanged. Blast-radius row 22 reclassified `[VERIFY/NO-CHANGE]` →
`[CHANGE]` (4 new symbols added to `ast.rs`). (FIX 2 / HIGH-1 sibling) `Ast::SqlPipe` head-SQL
derivation (`materialization.rs::execute_against_session Ast::SqlPipe arm`) used round-trip
`normalize` — same RISK-1 bare-string violation as the `Ast::Sql` arm. Fix: `Ast::SqlPipe` arm
also routes through `PqlNormalizer::normalize_for_datafusion`. Mode-agnostic arrow_cast invariant
stated: NO DataFusion-executed query, in any mode, may emit a bare Utf8 timestamp literal against
a Timestamp column. Three-row emission-path table added to Addendum. Blast-radius row 21 updated
(SqlPipe arm added). BC-2.11.021 unchanged. Blast radius rows 21-22 updated (count remains 22).
ARCH-INDEX v2.165→v2.166.

**v1.12 (DEFECT-PQL-FNCALL-LHS-001 [H5c]: NonColumnLhsComparison arm grammar-reachable from pipe `| where` mode, 2026-07-13):** §D4 NonColumnLhsComparison arm (arm 4/5) note added: as of DEFECT-PQL-FNCALL-LHS-001 (live-audit [H5c], 2026-07-13), the pipe `| where` predicate grammar admits function-call LHS comparisons via the new `fn_call_comparison` production in `build_predicate_parser` (`FuncCall::Scalar` only), making this arm grammar-reachable from pipe mode — was defense-in-depth-only prior. Arm behavior unchanged: date-like RHS (`Literal::RawTemporalLiteral`) → E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison) at plan time; non-date-like RHS → valid, passes to DataFusion; fn-call args participate in E-QUERY-038 column walk. Option-(a) pipe stats-by/sort treatment unchanged. Blast-radius row 14 (`filter_parser.rs`) updated.

**v1.11 (§D4 GROUP BY/ORDER BY dead-arm fix: `Literal::Timestamp` co-trigger, 2026-07-10):** LOCAL adversary finding F-EQ42-P1-001 (HIGH) closing DEFECT-EQUERY042-GROUPBY-DEADARM-001. §D4 dispatch table arms (6) GROUP BY and (7) ORDER BY amended: `Literal::Timestamp` (RFC-3339 fast-path constant, e.g. `'2026-07-01T00:00:00Z'`) added as co-trigger alongside `Literal::RawTemporalLiteral` — a pre-parsed RFC-3339 constant in GROUP BY/ORDER BY is equally degenerate (constant grouping/ordering key); the `RawTemporalLiteral`-only wording in v1.10 was the dead-arm defect. Arm (5) NonColumnLhsComparison remains `RawTemporalLiteral`-only (code scope matches). Behavior Reference Table: two rows added for RFC-3339 form → `Literal::Timestamp` → E-QUERY-042 (GroupBy/OrderBy). Clarifying sentence added: `Literal::Timestamp` in SELECT projection / JOIN ON / function args passes through `check_temporal_literals` without coercion (already-validated RFC-3339 form; only the degenerate-position gates intercept it). Rejection arm narratives, BC Amendments detection semantics, Blast Radius row 21, Tasks-to-ADD, and Red Gate test table updated for co-trigger.

**v1.10 (§D4 GROUP BY/ORDER BY REJECT + non-column-LHS REJECT + pipe-mode mechanism, 2026-07-05):** Human-ratified refinement of the §D4 v1.8 non-comparison-position dispatch. The flat "no column type context → COERCE" arm (4 in v1.8 numbering) is split into four distinct arms: (4a) SELECT projection bare literal → COERCE to `Literal::String(s)` (OBS-2 preserved); (4b) GROUP BY position → REJECT with E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy); (4c) ORDER BY position → REJECT with E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy). A new arm (5) is added for non-column-LHS comparisons (LHS is a function/compound expression, date-like RHS): the walker cannot resolve the LHS type at plan time; silent coercion would reintroduce RISK-1 for datetime-valued expressions — REJECT with E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison). This closes the current bug where `WHERE lower(hostname) = '2026-06-24'` produces `QueryPlanFailed → -32000 INTERNAL_ERROR` instead of an analyst-readable `-32602 INVALID_PARAMS`. New error code E-QUERY-042 (`TemporalLiteralInvalidPosition`) specified with three position-specific messages (GroupBy, OrderBy, NonColumnLhsComparison); emitted by `PrismError::TemporalLiteralInvalidPosition { position: TemporalLiteralPosition, value_prefix: String }`; maps to MCP `-32602 INVALID_PARAMS`. Pipe-mode mechanism for `stats … by` / `sort` specified as **option (a)**: improved parse-time error message (lower complexity than grammar extension; these positions already reject at parse time; `check_temporal_literals` walker changes are SQL-mode only for GROUP/ORDER positions). BC-2.11.004 EC-11-004-004 guidance corrected: pipe stats-by literal must REJECT at parse time, not coerce. Arms 1/2/3 (Datetime-col→E-QUERY-041, String-col→COERCE, Integer/Float/Bool→E-QUERY-002) and emitter guard (Step 5, belt-and-suspenders E-QUERY-002 `QueryPlanFailed`) UNCHANGED. ARCH-INDEX v2.169→v2.170.

**v1.8 (OBS-2 human-ratified behavior change + F-LOW-1 thread-local addendum + OBS-1 Literal::String naming, 2026-07-04):** Three §D4 corrections. **(CHANGE 1 / OBS-2, human-ratified)** Non-comparison-position `RawTemporalLiteral` now COERCES to `Literal::String(s)` (SUCCESS) instead of E-QUERY-002 (`QueryPlanFailed`). A date-like literal in a projection, GROUP BY, ORDER BY, or any position with no column type to constrain it is a plain string constant — standard SQL `SELECT '2026-06-24'` returns the string; consistent with RISK-5 String-column coercion philosophy. The prior E-QUERY-002 (`QueryPlanFailed`) was over-strict and produced a misleading "-32000 internal error" for valid analyst queries. Coercion is byte-identical (`RawTemporalLiteral` carries the original string). Updated: Step-3 dispatch table last row (error→coerce), E-QUERY boundary table (new non-comparison coercion row added), coercion-arm narrative (new non-comparison subsection added), Step-5 guard prose updated (guard is now truly unreachable — all non-error paths coerce before emission), BC Amendments fourth-arm description, Tasks-to-ADD `check_temporal_literals` row, blast-radius row 21. The Datetime-col→E-QUERY-041, String-col→coerce, and Integer/Float/Bool-comparison→E-QUERY-002 arms are UNCHANGED. **(CHANGE 2 / F-LOW-1)** SQL-mode emission addendum prose self-contradiction fixed: v1.6 stated `PqlNormalizer::normalize()` "delegates to `normalize_literal` (not `normalize_literal_dispatch`) and therefore never consults the thread-local" — incorrect. The shipped code routes through `normalize_literal_dispatch` (via `normalize_literal_as_expr`), which checks `NORMALIZE_FOR_DATAFUSION`; isolation is by-flag (round-trip callers never invoke `normalize_for_datafusion` and thus never set the flag), not by-separate-method. Corrected paragraph now accurately describes the flag-based isolation. **(CHANGE 3 / OBS-1)** Four `Literal::Utf8` references in §D4 replaced with `Literal::String`: Step-2 parser code block (`else → Literal::String(s)`), `is_date_like` description prose (`those remain Literal::String`), non-match forms section header (`Forms that stay Literal::String`), E-QUERY boundary table `'not-a-date'` row. No `Literal::Utf8` variant exists in the codebase; actual enum variant is `Literal::String`. ARCH-INDEX v2.167→v2.168.

**v1.7 (LOCAL adversary MED-1: §D4 emitter-guard E-QUERY-001→E-QUERY-002, 2026-07-04):**
Four §D4 references to the `pipe_sql_emitter.rs` `Literal::RawTemporalLiteral` guard
corrected from E-QUERY-001 to E-QUERY-002 (QueryPlanFailed): §D4 Step-5 body, RISK-4
narrative, blast-radius row 9, and Tasks-to-ADD guard arm row. Source of truth: shipped
`PrismError::QueryPlanFailed` (= E-QUERY-002 per error-taxonomy.md), story RG-024, and
the §D4 dispatch table row "Non-comparison position with no resolvable String context →
E-QUERY-002 `QueryPlanFailed`" — which already said E-QUERY-002 in v1.5 but was not
propagated to Step-5/RISK-4/blast-row-9/task-table when FIX A landed. The guard is a
plan-time invariant violation (QueryPlanFailed), not a parse-time error (QueryParseFailed
= E-QUERY-001). ARCH-INDEX v2.166→v2.167.

v1.1 amends v1.0: D3 emitter form changed from `TIMESTAMP '...'` to explicit
`arrow_cast(...)` (DataFusion 53.1.0 verified: `TIMESTAMP '...'` produces
`Timestamp(Nanosecond, None)`, not Microsecond/UTC); D4 E-QUERY-041 detection
mechanism changed from DataFusion-cast-failure intercept to Prism-level literal
pre-validator using chrono RFC-3339 strictness (arrow-cast 58.2.0 is lenient —
accepts date-only and offset-less strings — so cast-failure cannot be the gate);
Arrow construction form corrected from `Some(Arc::new("UTC".into()))` (compiles to
`Arc<String>`) to `Some(Arc::from("UTC"))` (correct `Arc<str>`); RISK-1 narrative
corrected.

v1.2 (2026-07-04): OBS-4 typo fix — `Arc::from("UTF")` → `Arc::from("UTC")`.

**v1.3 §D4 ACCEPTED (human-ratified 2026-07-04):** §D4 redesigned from parse-fail
text-scanner (8 fix-bursts, Unicode byte-offset panic / VP-021 violation,
dotted/filter/qualified false positives) to **Option A — lenient-parse-then-AST-walk**.
`Literal::RawTemporalLiteral` AST node; parser emits it for date-only/offset-less;
`check_temporal_literals` plan-time walker uses schema + resolved AST. All text-scanner
functions deleted. D1–D3, D5–D8 unchanged. **Ratification modification:** `check_temporal_literals`
COERCES (not rejects) `RawTemporalLiteral` against String/Utf8 columns — rewrites to
`Literal::String(s)`, byte-identical to pre-ADR-052 behavior. RISK-5 eliminated by
design (see §D4 Step 3 coercion arm and RISK-5 below).

**v1.4 (pre-TDD, remove-uncertainty, 2026-07-04):** `is_date_like` acceptance set
expanded from 2 format strings to 7. Under-match defect (space-separator, no-seconds,
fractional-seconds forms) corrected; over-match forms (unpadded digits, big/signed years)
evaluated and accepted as BENIGN with documented rationale. New
`### is_date_like Acceptance Set (Canonical)` subsection added to §D4. E-QUERY-001↔
E-QUERY-041 boundary table updated with 3 new examples. Red Gate test table expanded
with 3 new forms. BC-2.11.021 amendment text updated to enumerate all 7 accepted forms.
D1–D3, D5–D8, RISK-1 through RISK-5 unchanged.

Supersedes **ADR-044 §D4** (planning-time constant injection as ISO-8601 string
comparison). ADR-044 §D1–D3, §D5–D7 remain valid and are NOT affected.

Sequencing constraint: **ADR-052 ships before ADR-051**. After ADR-052 is accepted
and implemented, ADR-051's D1 `datetime` row must be amended from
`DataType::Utf8` to `DataType::Timestamp(TimeUnit::Microsecond, Some("UTC"))`.
See §D8 — Sequencing and ADR-051 Interaction.

---

## Context

### Current State: String-Based Datetime

Prism's query engine represents OCSF `datetime` fields as Arrow `DataType::Utf8`
throughout the execution pipeline. The canonical mapping is in
`crates/prism-bin/src/spec_driven_adapter.rs:886`:

```rust
ColumnType::Datetime => DataType::Utf8,
```

This means:
- Every sensor `datetime` column is registered in DataFusion as a `Utf8` string column.
- Temporal predicates compiled from `NOW() - INTERVAL '24h'` are injected as ISO-8601
  string constants (`'2026-06-24T00:00:00Z'`), and DataFusion evaluates them as
  string comparisons against `Utf8` columns.
- `crates/prism-query/src/pipe_sql_emitter.rs:822` emits:
  `Literal::Timestamp(ts) => format!("'{}'", ts.iso8601)` — a bare quoted string.
- String comparisons on ISO-8601 strings are lexicographically correct ONLY for
  UTC timestamps with zero UTC offset (`Z` form). This is an accidental invariant
  maintained by the OCSF normalization layer but is not enforced by the type system.
- `crates/prism-core/src/column.rs` line 28 has a stale doc comment claiming
  "Arrow: TimestampMicrosecond" — contradicted by `spec_driven_adapter.rs` which is
  the implementation source of truth.

### Why Migrate Now

1. **Correctness gap**: String comparison for datetime is an invisible invariant.
   Any sensor adapter that emits a non-UTC-offset form (`+00:00` vs `Z`) or a
   sub-second precision that changes lexicographic ordering breaks silently.

2. **Enrichment typing dependency**: ADR-051 must decide the Arrow type for
   `output_type = "datetime"` enrichment fields. Setting it to `Utf8` creates a
   permanent two-representation split: sensor datetime columns are `Utf8`,
   enrichment datetime columns are `Utf8`, but for different reasons. After this
   migration both will be `Timestamp(Microsecond, UTC)` for the same reason.

3. **OCSF contract**: OCSF explicitly defines `datetime` as UTC timestamps (RFC-3339
   with UTC offset). Making this explicit in the Arrow schema (`Some("UTC")`) aligns
   the runtime type with the OCSF normalization contract.

4. **DataFusion arithmetic**: Temporal arithmetic on `Utf8` columns requires explicit
   CAST operations and string parsing at every comparison. `Timestamp` columns get
   native DataFusion time arithmetic, interval arithmetic, and ordering — features
   that will be required once feature-flagged write operations land.

### Feasibility Already Verified

`crates/prism-query/src/tests/high002_plan_pinning_tests.rs:313` confirms:
"DataFusion fails to compare `Utf8` against `Timestamp(Microsecond, None)`" — this
is the exact failure mode the current string-only model avoids by keeping both sides
`Utf8`. It is also the test that must be UPDATED (not fixed around) to show that
both sides are now `Timestamp(Microsecond, UTC)`.

---

## Decision

**D1 — Arrow Timestamp type: `Timestamp(Microsecond, Some("UTC"))`.**

The canonical Arrow type for all PrismQL datetime values (sensor columns, temporal
literals, enrichment datetime fields) is:
`DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))`.

**Canonical Rust construction form (compile-verified):**
```rust
DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))
```
The timezone field is `Option<Arc<str>>`. `Arc::from("UTC")` produces `Arc<str>` directly.
`Some(Arc::new("UTC".into()))` does NOT compile correctly — `"UTC".into()` infers to
`String`, producing `Arc<String>` which is the wrong type. Use `Arc::from("UTC")` or,
with explicit type context, `Some("UTC".into())`.

Rationale for `Microsecond` (not `Nanosecond`):
- `high002_plan_pinning_tests.rs:313` cites `Timestamp(Microsecond, None)` as the
  expected type from an earlier sensor-column probe — microsecond is the established
  precision baseline in this codebase.
- OCSF datetime resolution is second-level in most sensor APIs; microsecond is
  sufficient headroom without the overflow risk of nanosecond representation for
  timestamps near 2262 CE.
- DataFusion 53.1.0 verified: `TIMESTAMP '...'` SQL literals produce
  `Timestamp(Nanosecond, None)` — NOT Microsecond. The explicit `arrow_cast` emitter
  form (D3) is required to force Microsecond precision without relying on DataFusion's
  implicit `temporal_coercion_nonstrict_timezone` cast path.

Rationale for `Some("UTC")` (not `None`):
- OCSF datetime is explicitly UTC by specification. The OCSF normalization layer
  (`prism-ocsf`, `prism-bin/spec_driven_adapter.rs`) guarantees UTC normalization
  at the adapter boundary.
- Explicit UTC tagging prevents silent bugs if any future consumer (DataFusion
  window functions, external export) applies timezone-aware semantics to an
  untagged `None` timestamp.
- DataFusion 53.1.0: comparing `Timestamp(Nanosecond, None)` against
  `Timestamp(Microsecond, Some("UTC"))` does NOT error — `temporal_coercion_nonstrict_timezone`
  unifies None→Some(tz) and inserts a lossless cast. However, relying on this implicit
  path is non-deterministic across DataFusion versions. The `arrow_cast` emitter form
  (D3) eliminates the reliance.

**D2 — Sensor column registration change: `spec_driven_adapter.rs` and `column.rs`.**

`crates/prism-bin/src/spec_driven_adapter.rs`, function `column_type_to_arrow`,
line 886:
```rust
// Before:
ColumnType::Datetime => DataType::Utf8,
// After:
ColumnType::Datetime => DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC"))),
```

`crates/prism-core/src/column.rs`, `ColumnType::Datetime` doc comment, line 28:
```rust
// Before (stale):
/// ISO-8601 UTC datetime string. Arrow: TimestampMicrosecond
// After (correct):
/// ISO-8601 UTC datetime string, normalized to UTC at the adapter boundary.
/// Arrow: Timestamp(Microsecond, UTC-tagged). Stored and transmitted as RFC-3339.
```

The `column.rs` stale comment was also identified as a blast-radius item in
ADR-051 v1.1 §Blast Radius item 15 — this decision closes that item.

**D3 — SQL emission change: `pipe_sql_emitter.rs` `Literal::Timestamp` rendering.**

`crates/prism-query/src/pipe_sql_emitter.rs`, line 822:
```rust
// Before:
Literal::Timestamp(ts) => format!("'{}'", ts.iso8601),
// After (arrow_cast explicit form — Rust string with escaped inner quotes):
Literal::Timestamp(ts) => format!("arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')", ts.iso8601),
```

**Why `arrow_cast`, not `TIMESTAMP '...'`.**

DataFusion 53.1.0 (verified via remove-uncertainty PASS-1, 2026-07-03):
`TIMESTAMP '2026-07-03T00:00:00Z'` lowers to `Timestamp(Nanosecond, None)` — the `Z`
UTC offset is ignored and the precision is nanosecond, not microsecond. This would
require DataFusion's `temporal_coercion_nonstrict_timezone` implicit cast to reconcile
the literal against a `Timestamp(Microsecond, Some("UTC"))` column. While DataFusion
53.1.0 applies this cast successfully without error, relying on implicit coercion is
non-deterministic across DataFusion minor versions and obscures the intended types.

`arrow_cast('<rfc3339>', 'Timestamp(Microsecond, Some("UTC"))')` is the explicit form:
- The type string `'Timestamp(Microsecond, Some("UTC"))'` is DataFusion's canonical
  `arrow_cast` type-string grammar (verified as query Q-M1 in remove-uncertainty PASS-1).
- The literal is explicitly typed to `Timestamp(Microsecond, Some("UTC"))` — matching
  the column type exactly, with no precision or timezone coercion needed.
- No implicit DataFusion coercion path is invoked; the plan-pinning probe test will
  verify this holds across DataFusion version bumps.

**Rust format string note:** The inner `"UTC"` in `Some("UTC")` is enclosed in the
outer SQL single-quoted string `'Timestamp(Microsecond, Some("UTC"))'`. Because the
Rust format string uses `"..."` as its delimiter, the inner `"` characters must be
escaped as `\"`.

**Pushdown path (`pushdown.rs`) is UNCHANGED.** The pre-fan-out ADR-033 T1 extractor
at `pushdown.rs:450` already operates on `Literal::Timestamp.instant` (a
`chrono::DateTime<Utc>`) and calls `.to_rfc3339()` to produce the API string:
```rust
Expr::Literal(Literal::Timestamp(ts)) => ts.instant.to_rfc3339()
```
This path operates on the Prism AST layer, not on Arrow DataType. Sensor API calls
receive the same RFC-3339 UTC string before and after this migration — **zero
behavioral change at the adapter boundary** (see D5).

**D4 — E-QUERY-041: Lenient-parse-then-AST-walk (Option A).**

### v1.3 Redesign: Why the §D4 v1.2 design was retired

The v1.2 design (Prism plan-time pre-validator using `chrono::parse_from_rfc3339`) was
implemented with a PARSE-FAIL recovery path: the PrismQL parser rejected date-like
literals (E-QUERY-001), then engine.rs re-scanned the raw query string
(`extract_table_name_from_query_str`, `extract_column_name_adjacent_to_quoted_value`,
`is_bad_literal_in_datetime_column`) to determine if the comparison was against a
datetime column and upgrade the error to E-QUERY-041. This text-scanner was fragile:
- Dotted-source expressions (`source.column`) were mis-classified as missing sources
- Filter-mode `source |` prefix broke the table-name extractor
- Non-ASCII MCP input caused a **Unicode byte-offset panic** (to_lowercase slicing into
  raw bytes → VP-021 violation — unchecked panic on valid analyst input)
- Qualified nested columns (`other.timestamp`, `payload.event_time`) produced
  false E-QUERY-041 (`.last()` collapse matched the primary table)

8 fix-bursts failed to converge the text-scanner. The root cause is structural: a
text-scanner over unparsed query strings cannot correctly resolve dotted sources,
filter-mode syntax, or qualified columns — these require the AST + schema that are
only available at plan time.

**Key reframe:** E-QUERY-041 is a MESSAGE-SPECIFICITY UPGRADE, not a correctness gate.
Date-only and offset-less literals already FAIL the PrismQL parser and never reach
DataFusion — there is NO silent-wrong-result risk regardless. The purpose of E-QUERY-041
is pedagogical: upgrade a generic `E-QUERY-001: invalid ISO-8601 timestamp` into the
actionable `E-QUERY-041: use RFC-3339 for datetime columns`. The entire text-scanner
apparatus existed only for this upgrade. Option A achieves the same upgrade with zero
text scanning.

### Mechanism: Lenient-parse-then-AST-walk

**Step 1 — New AST node: `Literal::RawTemporalLiteral(String)` in `ast.rs`.**

A new variant is added to the `Literal` enum (alongside `Literal::Timestamp`):
```rust
/// A quoted string that resembles a date or datetime but is NOT valid RFC-3339.
/// Produced by the parser when `is_date_like` matches — covers 7 forms:
/// date-only (`'2026-06-24'`); T-sep full seconds (`'2026-06-24T12:00:00'`);
/// T-sep fractional (`'2026-06-24T12:00:00.123'`); T-sep no seconds
/// (`'2026-06-24T12:00'`); space-sep full seconds (`'2026-06-24 12:00:00'`);
/// space-sep fractional (`'2026-06-24 12:00:00.500'`); space-sep no seconds
/// (`'2026-06-24 12:00'`). Validated at plan time by `check_temporal_literals`.
/// Must never reach SQL emission.
RawTemporalLiteral(String),
```

**Step 2 — Parser change: lenient fallback in the timestamp literal production.**

The PrismQL parser's timestamp literal combinator (in `sql_parser.rs` and/or
`filter_parser.rs`) currently fails with E-QUERY-001 when `chrono::parse_from_rfc3339`
rejects the quoted string. Under Option A, it instead applies a LENIENT secondary
classification:

```
quoted_string → 
  if parse_from_rfc3339 succeeds → Literal::Timestamp   (unchanged)
  else if is_date_like(s) → Literal::RawTemporalLiteral(s)  (new: parse SUCCEEDS)
  else → Literal::String(s)                             (unchanged)
```

`is_date_like(s)` is a multi-format heuristic covering the common offset-less
date/datetime forms a SOC analyst might write. The canonical format-string list is
EXHAUSTIVE — the implementer transcribes these exact 7 format strings, no more,
no fewer:

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

All `parse_from_str` calls require full input consumption — trailing characters cause
rejection. Forms 2 and 3 are belt-and-suspenders: if chrono 0.4.44 `%.f` is a true
zero-or-more optional that consumes nothing when there is no decimal point, then form 3
subsumes form 2 and form 6 subsumes form 5; having both is safe (at most one extra parse
attempt per call, negligible cost). The heuristic DOES NOT match arbitrary strings (e.g.,
`'not-a-date'`, `'sensor-id-abc'`); those remain `Literal::String`.

With this change, parsing a query containing `'2026-06-24'`, `'2026-06-24T12:00'`,
`'2026-06-24 12:00:00'`, or `'2026-06-24T12:00:00.123'` SUCCEEDS and produces an
AST containing `Expr::Literal(Literal::RawTemporalLiteral("..."))`. No `E-QUERY-001`
is emitted at parse time for any of these forms.

### `is_date_like` Acceptance Set (Canonical)

**Forms that MATCH (produce `Literal::RawTemporalLiteral`)**

| Example | Form # | Matching Format |
|---------|--------|----------------|
| `'2026-06-24'` | 1 | `%Y-%m-%d` |
| `'2026-06-24T12:00:00'` | 2 | `%Y-%m-%dT%H:%M:%S` |
| `'2026-06-24T12:00:00.123'` | 3 | `%Y-%m-%dT%H:%M:%S%.f` |
| `'2026-06-24T12:00:00.123456'` | 3 | `%Y-%m-%dT%H:%M:%S%.f` |
| `'2026-06-24T12:00'` | 4 | `%Y-%m-%dT%H:%M` |
| `'2026-06-24 12:00:00'` | 5 | `%Y-%m-%d %H:%M:%S` |
| `'2026-06-24 12:00:00.500'` | 6 | `%Y-%m-%d %H:%M:%S%.f` |
| `'2026-06-24 12:00'` | 7 | `%Y-%m-%d %H:%M` |

**Over-matched forms (match but are ACCEPTED BENIGN)**

| Example | Why Over-matched | Disposition |
|---------|-----------------|-------------|
| `'2026-6-24'` (unpadded month/day) | `%m`/`%d` accept single digits in chrono | ACCEPTED BENIGN — not RFC-3339 regardless; E-QUERY-041 ("use RFC-3339") is an accurate and helpful message; against String col, coerced to string comparison (correct); against numeric/bool, E-QUERY-002 `QueryTypeMismatch` (correct). |
| `'12345-06-24'` (big year) | `%Y` accepts arbitrary-width year | ACCEPTED BENIGN — same rationale; security sensor data from year 12345 is not a real concern. |
| `'-0044-03-15'` (negative year) | `%Y` accepts signed year | ACCEPTED BENIGN — same rationale. |

**Decision — no year-width guard or regex layer:** A `%4Y` constraint or regex pre-filter
would add implementation complexity with no diagnostic benefit. For every over-matched
input, `check_temporal_literals` produces the correct outcome — E-QUERY-041 for a
Datetime column (the "use RFC-3339" message is accurate), coercion for a String column,
or E-QUERY-002 `QueryTypeMismatch` for a numeric/bool column. The benign-accept path is
the production-grade default.

**Forms that stay `Literal::String` (NOT matched by `is_date_like`)**

| Example | Reason Not Matched |
|---------|-------------------|
| `'not-a-date'` | No format matches |
| `'sensor-id-abc'` | No format matches |
| `'2026-06-24Z'` | Trailing `Z` is not consumed by any NaiveDate/NaiveDateTime format; `parse_from_str` rejects on leftover `Z` |
| `'2026-06-24T12:00:00Z'` | Matched by `parse_from_rfc3339` at Step 1; never reaches `is_date_like`; emitted as `Literal::Timestamp` |
| `'2026-06-24T12:00:00+00:00'` | Same — RFC-3339 form, `Literal::Timestamp` at Step 1 |
| `'abc-12-34'` | Non-numeric year rejected by `%Y`; no format matches |

---

**Step 3 — Plan-time validator: `check_temporal_literals` AST walker — DEFINED in `materialization.rs`, invoked as an early gate from `engine.rs`.**

After the AST is produced and the schema is resolved, `check_temporal_literals` walks
the full `Expr` tree:

| `RawTemporalLiteral` position | Schema check | Result |
|-------------------------------|--------------|--------|
| Comparison (`>`, `<`, `>=`, `<=`, `=`, `!=`) against Datetime/Timestamp column (LHS is bare `Field`) | `column_type == Timestamp(Microsecond, UTC)` | E-QUERY-041 (pedagogical upgrade — the primary purpose of this mechanism) |
| Comparison against String/Utf8 column (LHS is bare `Field`) | `column_type == DataType::Utf8` | COERCE: rewrite node in-place to `Literal::String(s)`; compare as ordinary string literal (SUCCESS — no error emitted, byte-identical to pre-ADR-052 behavior) |
| Comparison against Integer / Float / Bool column (LHS is bare `Field`) | numeric or boolean type | E-QUERY-002 (`QueryTypeMismatch` — date-shaped literal cannot equal a numeric or boolean column; carries structured `column`, `table`, `actual_type`, `operator` fields per BC-2.11.017) |
| Comparison where LHS is a function/compound expression (non-`Field`), date-like RHS | LHS type unresolvable at plan time | E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison) — REJECT with analyst-facing INVALID_PARAMS; silent coercion would reintroduce RISK-1 for datetime-valued LHS expressions (v1.10 NEW). Grammar-reachable from pipe `| where` mode via `fn_call_comparison` production in `build_predicate_parser` (`FuncCall::Scalar` only) as of DEFECT-PQL-FNCALL-LHS-001 (2026-07-13 [H5c]) — no longer defense-in-depth-only. Non-date-like RHS (`Literal::String`) → valid, passes to DataFusion without interception; fn-call args participate in E-QUERY-038 column walk. |
| Non-comparison position where surrounding type context resolves to String/Utf8 | `column_type == DataType::Utf8` | COERCE: rewrite to `Literal::String(s)` (SUCCESS) |
| Non-comparison: SELECT projection (bare literal in SELECT list, no column type context) | no column type context | COERCE: rewrite `RawTemporalLiteral(s)` → `Literal::String(s)` (SUCCESS — standard SQL `SELECT '2026-06-24'` returns the string constant; OBS-2 preserved) |
| Non-comparison: GROUP BY position — `Literal::RawTemporalLiteral` **or** `Literal::Timestamp` (bare literal in GROUP BY, no column type context) | no column type context | E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy) — REJECT: grouping by a literal constant (RFC-3339 fast-path constant or offset-less date form) is a degenerate no-op, almost always an analyst mistake (v1.10; `Literal::Timestamp` co-trigger added v1.11, DEFECT-EQUERY042-GROUPBY-DEADARM-001) |
| Non-comparison: ORDER BY position — `Literal::RawTemporalLiteral` **or** `Literal::Timestamp` (bare literal in ORDER BY, no column type context) | no column type context | E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy) — REJECT: ordering by a literal constant (RFC-3339 fast-path constant or offset-less date form) is a degenerate no-op, almost always an analyst mistake (v1.10; `Literal::Timestamp` co-trigger added v1.11, DEFECT-EQUERY042-GROUPBY-DEADARM-001) |

The schema-resolved column type is determined by the same path that resolves
`ColumnType::Datetime → DataType::Timestamp(...)` (D2). This path correctly handles:
- **Dotted expressions** (`source.column`, `payload.event_time`): resolved via the
  schema map, not by string parsing
- **Filter-mode** (`source | WHERE col > ...`): the AST captures the source and
  predicate structure; column resolution is identical to SQL mode
- **Qualified/nested columns** (`other.timestamp`): resolved against the schema of
  the correct source table, not the `.last()` segment
- **Unicode inputs**: operates on already-parsed `String` values (valid UTF-8); no
  raw byte-offset operations. VP-021 violation eliminated by construction.

**Coercion arm — String/Utf8 column (ratification modification, 2026-07-04):**

When `check_temporal_literals` encounters a `Literal::RawTemporalLiteral(s)` in
comparison position against a column whose resolved type is `DataType::Utf8` (a String
column), it rewrites the AST node in-place to `Literal::String(s)`. Because
`RawTemporalLiteral(String)` carries the exact original string bytes, and
`pipe_sql_emitter.rs` emits `Literal::String(s)` as a plain `'{escaped}'` SQL string
literal — the same emission path used before ADR-052 — this coercion is byte-identical
to pre-ADR-052 behavior. No error is emitted.

**Why coercion is correct for String columns:** String columns legitimately store
date-formatted values (partition keys, report-date labels, ISO-date-formatted external
IDs). Rejecting `WHERE string_col = '2026-06-24'` with E-QUERY-001 would make a valid
query unqueryable — a real behavior regression. The coercion arm eliminates this
false-positive entirely while preserving E-QUERY-041 exclusively for its intended
purpose: upgrading the error message when a non-RFC-3339 literal is compared against
an actual Datetime/Timestamp column.

E-QUERY-041 fires ONLY when `check_temporal_literals` resolves the comparison target
to `DataType::Timestamp(Microsecond, UTC)`. The coercion rule has no effect on that
path.

**Coercion arm — SELECT projection position (no column type context, v1.8, preserved in v1.10):**

When `check_temporal_literals` encounters a `Literal::RawTemporalLiteral(s)` in a
SELECT projection (bare literal in the SELECT expression list, no column type context),
it rewrites the node in-place to `Literal::String(s)`. Standard SQL
`SELECT '2026-06-24'` returns the string constant `2026-06-24`; rejecting this was
over-strict and produced a misleading "-32000 internal error" for a query any SQL
analyst would expect to succeed. The coercion is byte-identical: `RawTemporalLiteral`
carries the original string bytes, and `pipe_sql_emitter.rs` emits `Literal::String(s)`
as a plain `'{escaped}'` SQL string literal — the same emission path used for all other
string constants. (OBS-2 from v1.8 preserved unchanged in v1.10.)

**Rejection arm — GROUP BY position (v1.10; `Literal::Timestamp` co-trigger v1.11):**

When `check_temporal_literals` encounters a `Literal::RawTemporalLiteral(s)` **or a
`Literal::Timestamp`** in a GROUP BY expression, it REJECTS with E-QUERY-042
`TemporalLiteralInvalidPosition` (position: `GroupBy`). Grouping by a bare literal
constant (`GROUP BY '2026-06-24'` or `GROUP BY '2026-07-01T00:00:00Z'`) is a degenerate
no-op: every row maps to the same group keyed on the constant, which almost always
indicates an analyst mistake (e.g., intended to GROUP BY a column name but accidentally
quoted it). The `Literal::Timestamp` co-trigger closes the dead-arm defect
(DEFECT-EQUERY042-GROUPBY-DEADARM-001): an RFC-3339 fast-path constant (`Literal::Timestamp`
produced by the parser) in GROUP BY position is equally degenerate — the v1.10 wording
that named only `Literal::RawTemporalLiteral` was incorrect. The error carries an
analyst-facing INVALID_PARAMS message directing the analyst to reference a column or add
a WHERE filter instead.

**Rejection arm — ORDER BY position (v1.10; `Literal::Timestamp` co-trigger v1.11):**

When `check_temporal_literals` encounters a `Literal::RawTemporalLiteral(s)` **or a
`Literal::Timestamp`** in an ORDER BY expression, it REJECTS with E-QUERY-042
`TemporalLiteralInvalidPosition` (position: `OrderBy`). Ordering by a bare literal
constant (`ORDER BY '2026-06-24'` or `ORDER BY '2026-07-01T00:00:00Z'`) is a degenerate
no-op: sort order on a constant is undefined, almost always an analyst mistake. The
`Literal::Timestamp` co-trigger closes the same dead-arm defect as the GROUP BY arm
(DEFECT-EQUERY042-GROUPBY-DEADARM-001): an RFC-3339 fast-path constant in ORDER BY
position is equally degenerate. The error carries an analyst-facing INVALID_PARAMS message
directing the analyst to reference a column name instead.

**Rejection arm — non-column-LHS comparison (v1.10 NEW):**

When `check_temporal_literals` encounters a `Literal::RawTemporalLiteral(s)` in a
comparison position where the LHS is a function call or compound expression (not a
bare `Field` node), it REJECTS with E-QUERY-042 `TemporalLiteralInvalidPosition`
(position: `NonColumnLhsComparison`). The walker cannot resolve the LHS type at plan
time; silently coercing to `Literal::String(s)` would introduce RISK-1 for
datetime-valued expressions (e.g., `WHERE to_timestamp(col) = '2026-06-24'` would
compare a Timestamp against a string literal via implicit coercion). The error carries
an analyst-facing INVALID_PARAMS message directing the analyst to use RFC-3339 for
datetime column comparisons, a non-date-shaped string for string column comparisons,
or an explicit CAST. This arm closes the current bug where
`WHERE lower(hostname) = '2026-06-24'` produces `QueryPlanFailed → -32000
INTERNAL_ERROR` rather than an analyst-readable `-32602 INVALID_PARAMS`.

**Pipe `| where` grammar reachability (DEFECT-PQL-FNCALL-LHS-001, 2026-07-13 [H5c]):**

As of DEFECT-PQL-FNCALL-LHS-001, the pipe `| where` predicate grammar is extended to
admit function-call LHS comparisons via the `fn_call_comparison` production in
`build_predicate_parser` (`FuncCall::Scalar` only). This makes the NonColumnLhsComparison
arm grammar-reachable from pipe mode — previously this arm was defense-in-depth-only:
pipe predicates could not contain a non-`Field` LHS before the grammar extension, so only
SQL-mode queries could reach it. With `fn_call_comparison` grammar-valid in pipe `| where`,
`check_temporal_literals` now encounters this arm from pipe-mode queries as well. The arm
behavior is unchanged: a date-like RHS (`Literal::RawTemporalLiteral`) → E-QUERY-042
`TemporalLiteralInvalidPosition` (NonColumnLhsComparison) at plan time; a non-date-like RHS
(`Literal::String`) → valid, passes to DataFusion without interception by
`check_temporal_literals`. Function-call arguments in the LHS participate in the
E-QUERY-038 column-existence gate walk (`build_predicate_parser` enforces column validity
for the args). This arm remains `RawTemporalLiteral`-only (no `Literal::Timestamp`
co-trigger analogous to the GROUP BY/ORDER BY arms); an RFC-3339 string as RHS to a
function-call LHS comparison parses as `Literal::Timestamp` and passes through
`check_temporal_literals` to DataFusion unchanged, consistent with §D4 `Literal::Timestamp`
pass-through semantics outside degenerate-position gates.

**`Literal::Timestamp` in non-GROUP-BY/ORDER-BY positions (v1.11 clarification):**

`check_temporal_literals` intercepts `Literal::Timestamp` nodes ONLY in GROUP BY and
ORDER BY positions (the two degenerate-constant gates added by v1.11). In all other
positions — SELECT projection, JOIN ON predicates, function arguments, and column
comparisons (any column type) — a `Literal::Timestamp` node passes through
`check_temporal_literals` without coercion or rejection. This is correct by design:
`Literal::Timestamp` is the ALREADY-RESOLVED form produced when the parser successfully
parses a valid RFC-3339 string via `chrono::parse_from_rfc3339`. It carries a concrete
`DateTime<Utc>` and has been fully validated at parse time — no further type-level
intervention is needed outside the degenerate-position gates. Only
`Literal::RawTemporalLiteral` (the offset-less / date-only form that requires plan-time
column-type dispatch) triggers coercion in String-column comparisons and SELECT projection.
The emitter-guard in `pipe_sql_emitter.rs` (Step 5) is `RawTemporalLiteral`-only for the
same reason — `Literal::Timestamp` already has its own §D3 `arrow_cast` emitter path.

The two coercion arms (String-column comparison coerce, SELECT projection coerce) and
five rejection arms (Datetime-col E-QUERY-041, Integer/Float/Bool E-QUERY-002, GROUP BY
E-QUERY-042, ORDER BY E-QUERY-042, non-column-LHS E-QUERY-042) together ensure that all
`RawTemporalLiteral` nodes are fully consumed by `check_temporal_literals`. No
`RawTemporalLiteral` survives to the emitter in correct execution. The walker MUST
track the AST position context (SELECT projection / GROUP BY / ORDER BY / comparison
with field LHS / comparison with non-field LHS) to dispatch correctly.

**Step 4 — Deletion: text-scanner apparatus removed.**

The following functions and code paths are DELETED from `engine.rs`:
- `extract_table_name_from_query_str`
- `extract_column_name_adjacent_to_quoted_value`
- `is_bad_literal_in_datetime_column`
- The parse-fail branch that called the above functions

**Step 5 — Guard: `RawTemporalLiteral` must never reach SQL emission.**

`pipe_sql_emitter.rs` adds a `Literal::RawTemporalLiteral` arm that returns E-QUERY-002
(`QueryPlanFailed` — unvalidated temporal literal reached emission). Under correct plan
execution, `check_temporal_literals` processes ALL `RawTemporalLiteral` nodes before
emission via two coercion paths and five rejection paths:

- String-column comparison (bare Field LHS) → COERCE to `Literal::String(s)` (no error)
- SELECT projection (no column type context) → COERCE to `Literal::String(s)` (no error, OBS-2)
- Timestamp/Datetime column comparison (bare Field LHS) → E-QUERY-041 (error exit)
- Integer/Float/Bool column comparison (bare Field LHS) → E-QUERY-002 `QueryTypeMismatch` (error exit)
- GROUP BY position (`RawTemporalLiteral` or `Literal::Timestamp`) → E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy) (error exit, v1.10; `Literal::Timestamp` co-trigger v1.11)
- ORDER BY position (`RawTemporalLiteral` or `Literal::Timestamp`) → E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy) (error exit, v1.10; `Literal::Timestamp` co-trigger v1.11)
- Non-column-LHS comparison → E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison) (error exit, v1.10)

In every branch the `RawTemporalLiteral` node is consumed — no `RawTemporalLiteral`
survives to reach the SQL emitter in correct execution. The emission guard is a
belt-and-suspenders defensive check for internal invariant violations only (truly
unreachable when `check_temporal_literals` is correctly invoked).

### DataFusion Emission Addendum — SQL and SqlPipe Modes (HIGH-1, v1.5; SqlPipe sibling added v1.6)

**Problem — SQL-mode DataFusion emission path uses implicit coercion (RISK-1 violation).**

The `arrow_cast(...)` explicit-typing emitter specified in §D3 was applied ONLY to the
Pipe/Filter/SqlPipe-tail path (`pipe_sql_emitter.rs::literal_to_sql`). The **SQL-mode**
DataFusion emission path was not covered:

```
materialization.rs::execute_against_session (Ast::Sql arm)
  → ast.rs::PqlNormalizer::normalize
      → normalize_literal(Literal::Timestamp(ts))
          → currently emits: format!("'{}'", ts.iso8601)   ← BARE QUOTED STRING
```

This bare form `'2026-07-03T00:00:00Z'` is passed to DataFusion as a SQL expression. DataFusion
53.1.0 applies implicit `temporal_coercion_nonstrict_timezone` to reconcile it against a
`Timestamp(Microsecond, Some("UTC"))` column. This is exactly RISK-1: implicit coercion that
is non-deterministic across DataFusion minor versions. It also violates BC-2.11.021
§Postconditions, which states (mode-agnostically):

> "DataFusion sees ... `arrow_cast('...', 'Timestamp(Microsecond, Some(\"UTC\"))')` ... with no implicit coercion"

and EC-11-021-001 enumerates the SQL-mode form explicitly.

**Root-cause — `PqlNormalizer::normalize` is dual-purpose.**

`PqlNormalizer` in `ast.rs` serves TWO contracts:
1. **BC-2.11.018 PrismQL round-trip canonicalizer** — must emit a form the PrismQL grammar can
   re-parse. `Literal::Timestamp(ts)` emitted as `'2026-07-03T00:00:00Z'` (bare RFC-3339 in
   single quotes) is correct for this contract: it round-trips through the PrismQL parser.
2. **DataFusion SQL emitter** (Ast::Sql arm of `execute_against_session`) — must emit a form
   DataFusion evaluates with explicit types and no implicit coercion. The bare `'...'` form
   FAILS this requirement because DataFusion treats it as a `Utf8` string and applies implicit
   `temporal_coercion_nonstrict_timezone`.

Emitting `arrow_cast(...)` from `PqlNormalizer::normalize_literal` directly would break
BC-2.11.018: `arrow_cast('...', 'Timestamp(...)')` is not PrismQL grammar and cannot be
re-parsed. The dual-purpose constraint requires a separation.

**Resolution — thread-local scoped-context via `PqlNormalizer::normalize_for_datafusion` (ratified mechanism, v1.6).**

The ratified implementation uses a **thread-local scoped-context** in `ast.rs` to switch the
emission mode without threading an extra parameter through the entire recursive normalizer call
tree. Key additions to `ast.rs`:

```rust
// New thread-local flag (ast.rs)
thread_local! {
    static NORMALIZE_FOR_DATAFUSION: Cell<bool> = Cell::new(false);
}
```

`PqlNormalizer::normalize_for_datafusion(ast)` is a new method that:
1. Saves the current `NORMALIZE_FOR_DATAFUSION` value (save-and-restore drop-guard — re-entrant-safe).
2. Sets `NORMALIZE_FOR_DATAFUSION` to `true`.
3. Calls `normalize(ast)` (the existing recursive normalizer, unchanged).
4. Restores the saved value on drop via the guard.

Within this scoped context, `normalize_literal_dispatch` replaces `normalize_literal` as the
literal-emission dispatch point:

```
normalize_literal_dispatch(lit):
  if NORMALIZE_FOR_DATAFUSION → normalize_literal_for_datafusion(lit)
  else                        → PqlNormalizer::normalize_literal(lit)
```

`normalize_literal_for_datafusion` emits
`arrow_cast('...', 'Timestamp(Microsecond, Some("UTC"))')` for `Literal::Timestamp`; all other
variants delegate to `normalize_literal`.

`PqlNormalizer::normalize()` routes ALL literal emission through `normalize_literal_dispatch`
(via `normalize_literal_as_expr`), which checks the `NORMALIZE_FOR_DATAFUSION` thread-local.
When the flag is unset — as it always is for BC-2.11.018 round-trip callers that never invoke
`normalize_for_datafusion` — `normalize_literal_dispatch` delegates to `normalize_literal`,
emitting the bare re-parseable string `'{iso8601}'`. Isolation between the round-trip path and
the DataFusion emission path is **by-flag**, not by-separate-method: round-trip callers simply
never set `NORMALIZE_FOR_DATAFUSION`. The save-and-restore drop-guard in `normalize_for_datafusion`
handles nesting (correlated subqueries correctly restore the outer flag on inner-guard drop).

**Why thread-local scoped-context over an explicit emission-mode parameter:** Threading an
emission-mode flag through every recursive level of the `normalize_*` call tree would require
signature changes at every intermediate function. The thread-local installs the mode at the
`normalize_for_datafusion` entry point and makes it visible at the `normalize_literal_dispatch`
leaf with zero intermediate changes. The save-and-restore drop-guard makes it re-entrant-safe:
nested `normalize_for_datafusion` calls (e.g., correlated subqueries) correctly restore the
outer flag when the inner guard drops.

**Call site change in `materialization.rs::execute_against_session` (`Ast::Sql` arm):**

The `Ast::Sql` arm replaces its call to `PqlNormalizer::normalize(...)` for the DataFusion SQL
string derivation with `PqlNormalizer::normalize_for_datafusion(...)`. This installs the scoped
context so all `Literal::Timestamp` instances in that derivation emit `arrow_cast(...)`.

**Mode-Agnostic arrow_cast Invariant — SqlPipe Head Coverage (HIGH-1 sibling, v1.6).**

The v1.5 addendum covered only the `Ast::Sql` arm. The `execute_against_session` `Ast::SqlPipe`
arm derives `_sqlpipe_head` CTE SQL from the AST head and was using the round-trip `normalize`
(bare `'{iso8601}'`) — the same RISK-1 violation as the `Ast::Sql` arm.

**Fix:** The `Ast::SqlPipe` arm of `execute_against_session` also routes the head-SQL derivation
through `PqlNormalizer::normalize_for_datafusion` (same mechanism as the `Ast::Sql` arm).

**Mode-agnostic invariant (RISK-1):** No DataFusion-executed query, in any mode, may emit a
bare Utf8 timestamp literal against a Timestamp column. The three DataFusion-executed emission
paths and their `Literal::Timestamp` mechanisms:

| Emission path | Source location (behavioral anchor) | Mechanism |
|---------------|-------------------------------------|-----------|
| Pipe / Filter / SqlPipe-tail | `pipe_sql_emitter.rs::literal_to_sql` | Direct `format!("arrow_cast('{}', ...)", ts.iso8601)` (§D3) |
| SQL mode (`Ast::Sql`) | `materialization.rs::execute_against_session Ast::Sql arm` | `PqlNormalizer::normalize_for_datafusion` → `normalize_literal_dispatch` → `normalize_literal_for_datafusion` (arrow_cast) |
| SqlPipe head (`Ast::SqlPipe` → `_sqlpipe_head` CTE) | `materialization.rs::execute_against_session Ast::SqlPipe arm` | Same: `PqlNormalizer::normalize_for_datafusion` (v1.6 fix) |

The round-trip path (`PqlNormalizer::normalize`, BC-2.11.018) is NOT a DataFusion-executed path
and correctly continues to emit the bare re-parseable string.

**Invariant preserved (updated symbol names):**
- `PqlNormalizer::normalize(ast)` (round-trip, BC-2.11.018) — `Literal::Timestamp(ts)` emits
  `'2026-07-03T00:00:00Z'` (UNCHANGED — re-parseable by PrismQL grammar)
- `PqlNormalizer::normalize_for_datafusion(ast)` — `Literal::Timestamp(ts)` emits
  `arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` via
  `normalize_literal_dispatch` → `normalize_literal_for_datafusion` (NEW — BC-2.11.021 satisfied)

Key shipped symbols (behavioral anchors, not line numbers — TD-VSDD-091):
- `NORMALIZE_FOR_DATAFUSION` — thread-local flag in `ast.rs`
- `PqlNormalizer::normalize_for_datafusion` — scoped-context entry point in `ast.rs`
- `normalize_literal_dispatch` — flag-sensitive dispatch function in `ast.rs`
- `normalize_literal_for_datafusion` — `arrow_cast` emitter for `Literal::Timestamp` in `ast.rs`

**BC-2.11.021 is NOT weakened.** The BC correctly requires no implicit coercion for all query
modes. The thread-local scoped-context mechanism satisfies this requirement and now covers all
three DataFusion-executed emission paths.

### E-QUERY-001 / E-QUERY-002 / E-QUERY-041 dispatch boundary (v1.5 correction: numeric/bool arm uses E-QUERY-002)

| Input | Parser output | Validator result |
|-------|--------------|-----------------|
| `'2026-07-03T00:00:00Z'` (full RFC-3339) | `Literal::Timestamp` | No error |
| `'2026-06-24'` (date-only) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24T12:00:00'` (T-sep, full seconds) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24T12:00:00.123'` (T-sep, fractional) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24T12:00'` (T-sep, no seconds) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24 12:00:00'` (space-sep, full seconds) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24 12:00:00.500'` (space-sep, fractional) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24 12:00'` (space-sep, no seconds) vs Datetime/Timestamp col | `Literal::RawTemporalLiteral` | E-QUERY-041 |
| `'2026-06-24'` vs String/Utf8 col | `Literal::RawTemporalLiteral` | COERCE → `Literal::String("2026-06-24")`; compare as string literal (SUCCESS — zero regression from pre-ADR-052 behavior) |
| `'2026-06-24 12:00:00'` vs String/Utf8 col | `Literal::RawTemporalLiteral` | COERCE → `Literal::String("2026-06-24 12:00:00")`; compare as string literal (SUCCESS) |
| `'2026-06-24'` vs Integer / Float / Bool col | `Literal::RawTemporalLiteral` | E-QUERY-002 (`QueryTypeMismatch` — date-shaped literal cannot equal a numeric or boolean column) |
| `'2026-06-24'` in SELECT projection (`SELECT '2026-06-24' FROM t`) | `Literal::RawTemporalLiteral` | COERCE → `Literal::String("2026-06-24")`; returns the string constant (SUCCESS — standard SQL, OBS-2 preserved) |
| `'2026-06-24'` in GROUP BY position (`GROUP BY '2026-06-24'`) | `Literal::RawTemporalLiteral` | E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy) — grouping by a literal constant is a degenerate no-op (v1.10) |
| `'2026-06-24'` in ORDER BY position (`ORDER BY '2026-06-24'`) | `Literal::RawTemporalLiteral` | E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy) — ordering by a literal constant is a degenerate no-op (v1.10) |
| `'2026-07-01T00:00:00Z'` in GROUP BY position (`GROUP BY '2026-07-01T00:00:00Z'`) | `Literal::Timestamp` | E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy) — RFC-3339 fast-path constant in GROUP BY is a degenerate no-op; `Literal::Timestamp` co-trigger (v1.11, DEFECT-EQUERY042-GROUPBY-DEADARM-001) |
| `'2026-07-01T00:00:00Z'` in ORDER BY position (`ORDER BY '2026-07-01T00:00:00Z'`) | `Literal::Timestamp` | E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy) — RFC-3339 fast-path constant in ORDER BY is a degenerate no-op; `Literal::Timestamp` co-trigger (v1.11, DEFECT-EQUERY042-GROUPBY-DEADARM-001) |
| `WHERE lower(hostname) = '2026-06-24'` (non-column LHS, date-like RHS) | `Literal::RawTemporalLiteral` (RHS) | E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison) — LHS type unresolvable at plan time; prior behavior: `QueryPlanFailed → -32000 INTERNAL_ERROR` (v1.10 closes) |
| `'not-a-date'` anywhere | `Literal::String` | No temporal error (other type errors apply) |

### §D4 Pipe-mode Consistency — `stats … by` and `sort` Positions (v1.10)

**Current behavior:** `| stats count by '2026-06-24'` fails at parse time because the
pipe `stats` stage parser expects `FieldPath` (a bare identifier or `source.field`
dotted path) in the `by` key list. A string literal does not match `FieldPath` and
produces a parse error (E-QUERY-001). Similarly, `| sort '2026-06-24'` fails with
E-QUERY-001 because the `sort` key parser expects `FieldPath`. The
`check_temporal_literals` AST walker operates on plan-time resolved AST nodes — it
never sees a `RawTemporalLiteral` in a pipe `stats by` or `sort` position, because the
parser already rejected it before the AST is constructed.

**Mechanism: Option (a) — improve the pipe parse error message.**

The lower-complexity option is chosen over grammar extension (option (b)): the parser
already rejects literals in these positions; the improvement is a clearer, analyst-facing
parse-error message. No grammar changes, no new AST node variants, no
`check_temporal_literals` walker changes for pipe-mode `stats by` / `sort`.

**Implementer instruction:** In the pipe-stage parser combinators for `stats` (the `by`
key list) and `sort` (the sort key list), add context-aware error recovery: when the
`FieldPath` combinator fails on a token that begins with `'` (a single-quoted string
matching `is_date_like`), produce a message of the following form:

- For `stats … by`: `"pipe 'stats by' expects column references (field names), not
  literal values — '<value_prefix>' looks like a date-shaped literal, not a column name.
  Grouping by a literal constant has no effect. Did you mean to reference a column, or
  to add a '| where' filter before the stats stage?"`

- For `sort`: `"pipe 'sort' expects column references (field names), not literal values
  — '<value_prefix>' looks like a date-shaped literal, not a column name. Ordering by a
  literal constant has no effect. Did you mean to reference a column name?"`

These parse errors map to E-QUERY-001 (`QueryParseFailed`, MCP `-32602 INVALID_PARAMS`)
with the enhanced message text. They do NOT use the new E-QUERY-042 code (which is
plan-time only; these are parse-time rejections).

**Rationale for option (a) over option (b) (grammar extension + walker reject):**
- Zero grammar changes: pipe `stats by` and `sort` grammars remain `FieldPath`-only.
- Zero new AST node variants: no `PipeGroupByKey::Literal(...)` or
  `PipeSortKey::Literal(...)` required.
- Zero walker context additions: `check_temporal_literals` handles GROUP BY / ORDER BY
  in SQL-mode only; the pipe parse gate handles the pipe equivalents independently.
- MCP error code is identical (`-32602 INVALID_PARAMS`) — the analyst experience is
  equivalent.
- Option (b) requires: grammar extension + new AST enum variants + walker context
  tracking for pipe-mode GROUP/SORT positions + per-grammar changes. Complexity
  outweighs the benefit of strict symmetry through a shared code path.

**BC-2.11.004 EC-11-004-004 correction guidance (F-MED-1):**

The current BC-2.11.004 edge case EC-11-004-004 documents
`| stats count by '2026-06-24'` as a "coerce success" example. This is incorrect —
the pipe parser REJECTS this at parse time with E-QUERY-001. The product-owner must
amend EC-11-004-004 to reflect the actual behavior:
- **Behavior classification:** FORBIDDEN (parse-time rejection)
- **Correct outcome:** `| stats count by '2026-06-24'` → E-QUERY-001 parse error with
  message: `"pipe 'stats by' expects column references (field names), not literal values"`
- **Removed classification:** the "coerce success" classification must be deleted; there
  is no coercion path in pipe `stats by` position (the parser rejects before any walker
  sees the node)

**E-QUERY-041 message format (unchanged from v1.2):**
```
E-QUERY-041: The value '{first_50_chars}' cannot be interpreted as a UTC timestamp.
Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only
and offset-less forms are not accepted. For relative time filters, use
NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h').
```

**Chrono strictness invariant (AC-013, preserved):** The sensor-boundary datetime
parsing path (`spec_driven_adapter.rs`, ISO-8601 string → `i64` microseconds-since-epoch)
continues to use `chrono::DateTime::parse_from_rfc3339`. The plan-time validator uses
`chrono::NaiveDate::parse_from_str` / `chrono::NaiveDateTime::parse_from_str` for the
`is_date_like` heuristic (7 formats: date-only, T-sep full/fractional/no-seconds,
space-sep full/fractional/no-seconds) and implicitly rejects non-RFC-3339 forms
(the heuristic succeeds for these 7 accepted sub-forms, and `check_temporal_literals`
then raises E-QUERY-041, which is the rejection). The sensor boundary and query path
both reject non-RFC-3339 forms; the mechanism differs but the invariant is preserved.

**D5 — Pushdown / adapter-boundary semantics (explicit no-change statement).**

Sensor APIs (CrowdStrike FQL, Claroty AQL, Armis AQL, Cyberint REST, NVD REST)
accept date/time ranges as RFC-3339 or ISO-8601 UTC strings, not as Arrow Timestamps.
The adapter boundary converts Prism's typed `Timestamp` values to API strings at
the following points:

1. **ADR-033 T1 push-down extractor** (`pushdown.rs:450`): Extracts `Literal::Timestamp.instant`
   (chrono `DateTime<Utc>`) → `.to_rfc3339()` → passed to sensor adapter as `start_time`/
   `end_time` strings. **No change.**
2. **HttpLookup template interpolation** (`spec_driven_adapter.rs`, ADR-040): Any
   `datetime` column referenced in a template URL is already rendered as its ISO string
   value before API substitution. **No change required** — the interpolation reads
   the original sensor-stored value (an ISO string from the TOML spec or sensor API
   response), not the Arrow `Timestamp` representation.

The migration is a pure *internal-representation change*. Outbound API calls to sensor
adapters are unaffected. Inbound data from sensor APIs continues to arrive as ISO-8601
strings and is parsed to `Timestamp` at the `spec_driven_adapter.rs` normalization
boundary.

**Sensor timestamp parsing addition:** `spec_driven_adapter.rs` or the per-sensor
normalization path must parse incoming ISO-8601 datetime strings from sensor API
responses and convert them to `i64` microseconds-since-epoch (the in-memory
representation for `Timestamp(Microsecond, UTC)` in Arrow). This is the only
meaningful implementation work at the adapter boundary.

**D6 — RocksDB / ephemeral persistence: no migration required.**

Prism is an ephemeral federated query engine (ADR-002). Arrow `RecordBatch` objects
containing `Timestamp` columns are never persisted to RocksDB — they are materialized
in memory per query and discarded after the MCP tool response is returned.

The 19 RocksDB column families store operational state (detection rules, alert state,
cases, schedules, enrichment cache, diff result packs) as TOML-serialized or
JSON-serialized Rust structs, not as Arrow `RecordBatch` objects. Datetime values in
these stores are serialized as RFC-3339 strings and are not affected by the
Arrow-layer type change.

**Exception — `diff_results` CF:** Differential result packs may store serialized
snapshots of sensor data rows. If these snapshots include serialized Arrow IPC bytes
with `Utf8`-typed timestamp columns, they would be incompatible with a reader
expecting `Timestamp(Microsecond, UTC)`. Investigation by the implementer is required.
Mitigation: the diff_results CF has a TTL (short-lived operational data); the
implementation story MUST verify whether diff_result packs contain embedded Arrow IPC,
and if so, handle schema version compatibility (likely: clear old diffs on startup,
which is safe given the ephemeral model).

**D7 — ADR-044 §D4 supersession.**

ADR-044 §D4 states:
> "DataFusion sees `WHERE timestamp > '2026-06-24T00:00:00Z'` and applies it as a
> normal timestamp comparison."

After ADR-052, DataFusion sees:
> `WHERE timestamp > arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')`,
> where `timestamp` is `Timestamp(Microsecond, UTC)` — a fully explicit typed comparison,
> no implicit coercion.

The behavioral contract changes from: "string column compared to string literal" to
"timestamp column compared to explicitly-cast timestamp literal."

**Partial supersession scope**: Only ADR-044 §D4 is superseded. ADR-044 §D1 (`Expr::Now`),
§D2 (`INTERVAL` syntax), §D3 (`TimestampArithmetic` AST), §D5 (build_example_query),
§D6 (reference accuracy), and §D7 (SQL mode + Pipe mode support) remain fully valid
and unchanged.

**D8 — Sequencing and ADR-051 interaction.**

1. ADR-052 is authored and ratified FIRST (this ADR).
2. ADR-052 is implemented and merged before any ADR-051 implementation story ships.
3. Once ADR-052 is implemented, ADR-051 §D1 `datetime` row MUST be amended:
   - Before: `"datetime" → DataType::Utf8` (consistency with current sensor columns)
   - After: `"datetime" → DataType::Timestamp(TimeUnit::Microsecond, Some("UTC"))`
     (consistency with migrated sensor columns per ADR-052)
4. The ADR-051 product-owner BC amendment story must gate on ADR-052 being merged.
5. Failure to enforce this sequencing creates the two-representation split that ADR-052
   is designed to prevent.

---

## Rationale

1. **The pushdown path is safe.** The highest-risk concern — breaking sensor API
   time-window extraction — is eliminated by the structural observation that
   `pushdown.rs` operates at the Prism AST layer (`Literal::Timestamp.instant`),
   not at the Arrow DataType layer. The pushdown produces RFC-3339 strings for
   sensor APIs before SQL is ever emitted to DataFusion. The migration is invisible
   to the push-down subsystem.

2. **Three DataFusion-executed emission paths each require explicit `arrow_cast`, via
   path-appropriate mechanisms.**
   The `Literal::Timestamp` struct carries both `instant` (chrono) and `iso8601`
   (string) — emitters use `iso8601`, pushdown uses `instant`. These paths do not
   interfere. (a) **Pipe/Filter/SqlPipe-tail path** (`pipe_sql_emitter.rs::literal_to_sql`):
   direct `format!("arrow_cast('{}', ...)", ts.iso8601)` per §D3. (b) **SQL mode
   (`Ast::Sql`) and SqlPipe head (`Ast::SqlPipe`) paths**
   (`materialization.rs::execute_against_session` — both arms): each calls
   `PqlNormalizer::normalize_for_datafusion`, which installs a thread-local scoped
   context (`NORMALIZE_FOR_DATAFUSION`) so that `normalize_literal_dispatch` routes
   `Literal::Timestamp` to `normalize_literal_for_datafusion` (arrow_cast emitter).
   `PqlNormalizer::normalize_literal` (BC-2.11.018 round-trip, invoked outside the scoped
   context) remains unchanged. The explicit `arrow_cast` form eliminates DataFusion's implicit
   `temporal_coercion_nonstrict_timezone`, which is non-deterministic across DataFusion
   minor versions (RISK-1).

3. **OCSF UTC contract.** The OCSF normalization guarantee — all sensor datetime
   values are normalized to UTC at the adapter boundary — makes `Some("UTC")` the
   correct timezone tag. The UTC tagging is not new information; it is making an
   existing contract explicit in the Arrow schema.

4. **Eliminates the stale-comment debt.** ADR-051's blast-radius item 15 (stale
   `column.rs` comment "Arrow: TimestampMicrosecond") is fixed in-scope by D2.

5. **Unblocks ADR-051 datetime correctness.** Without this migration, ADR-051 must
   map `output_type = "datetime"` enrichment fields to `Utf8` to be consistent with
   sensor columns. After this migration, both sensor columns and enrichment fields
   can use `Timestamp(Microsecond, UTC)`, giving enriched datetime values the same
   type as sensor datetime values — enabling filter predicates like
   `WHERE device_last_seen_enriched > NOW() - INTERVAL '7d'` to work correctly.

---

## Consequences

### Positive

- OCSF datetime fields are typed at the DataFusion layer — type system enforces
  what the OCSF contract required.
- Temporal predicates (`NOW() - INTERVAL '24h'`) use native timestamp comparison
  instead of string comparison — eliminates the accidental correctness dependency
  on UTC+Z string form.
- Plan-pinning tests for datetime column types become definitively correct.
- Unblocks ADR-051 datetime row to use `Timestamp` rather than `Utf8`.
- Stale `column.rs` doc comment fixed.

### Negative / Trade-offs

- `spec_driven_adapter.rs` normalization must parse incoming datetime strings to
  microseconds-since-epoch. This is a bounded conversion step (`chrono::DateTime::parse_from_rfc3339`
  → `.timestamp_micros()`) but adds parse work at the adapter boundary. Expected
  impact: negligible (one `DateTime::parse_from_rfc3339` call per datetime field
  per result row — the same as what happens today to normalize timezone representations).
- `high002_plan_pinning_tests.rs` tests asserting `DataType::Utf8` for datetime
  columns must be updated to assert `DataType::Timestamp(Microsecond, UTC)`. These
  are the canonical plan-stability tests — updating them correctly is the primary
  verification work.
- Any external consumer (integration test or demo script) that constructs datetime
  comparison predicates as bare string literals must be reviewed; DataFusion's implicit
  casting should handle valid RFC-3339 strings, but malformed literals will now
  return E-QUERY-041 instead of a string comparison no-op.
- `diff_results` CF compatibility must be investigated before shipping (D6 exception).

---

## Risk

### RISK-1 (MEDIUM): DataFusion version-drift silent coercion

DataFusion 53.1.0 verified (remove-uncertainty PASS-1): `TIMESTAMP '...'` produces
`Timestamp(Nanosecond, None)`. Comparing `Timestamp(Nanosecond, None)` against
`Timestamp(Microsecond, Some("UTC"))` does NOT error in 53.1.0 —
`temporal_coercion_nonstrict_timezone` inserts a lossless cast. However, the coercion
path's behavior is not specified as a stable DataFusion API guarantee; a future
DataFusion minor version could tighten or change the coercion semantics.

**The risk is NOT "comparison errors."** The risk is: if the `arrow_cast` explicit form
is ever dropped or reverted to `TIMESTAMP '...'`, the implicit coercion path silently
returns correct results TODAY but may diverge on a future DataFusion version upgrade
without any compilation error.

**Mitigation (version-pinning probe)**: The implementer MUST add a DataFusion probe test
to `high002_plan_pinning_tests.rs` that registers a `Timestamp(Microsecond, Some("UTC"))`
column and verifies that `arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')
= Timestamp(Microsecond, Some("UTC"))` in the plan output. This test pins the behavior
to DataFusion 53.1.0 and will fail fast if a version upgrade changes `arrow_cast` semantics.
The explicit `arrow_cast` emitter form (D3) is the primary mitigation; the probe test
is the regression guard. Downgraded from HIGH to MEDIUM because the `arrow_cast` form
eliminates the direct comparison-failure risk.

### RISK-2 (MEDIUM): Two-representation transition window

Between ADR-052 landing and ADR-051 implementing, enrichment datetime fields will
temporarily be `Utf8` while sensor datetime columns are `Timestamp(Microsecond, UTC)`.
Analysts querying enriched datetime fields against temporal predicates in this window
will get string comparisons (old behavior for enrichment) vs. timestamp comparisons
(new behavior for sensors). This is inconsistent but not a regression — it preserves
the pre-existing behavior for enrichment fields.

**Mitigation**: The story sequencing constraint in D8 prevents ADR-051 from being
implemented before ADR-052. The transition window is the implementation gap between
the two stories. Document this window in the ADR-052 implementation story's "Known
Limitations" section. The window is expected to be short — the stories should ship
within the same wave.

### RISK-3 (LOW): `diff_results` CF Arrow IPC schema compatibility

If `diff_results` CF stores Arrow IPC with `Utf8` datetime columns, reading old
packs with a new reader expecting `Timestamp(Microsecond, UTC)` will fail with a
schema mismatch error.

**Mitigation**: Investigate during implementation. If Arrow IPC is stored, add a
startup migration step that clears the `diff_results` CF. Given the ephemeral model,
old diff packs have no persistent value — this is a safe operation.

### RISK-4 (MEDIUM): `Literal` enum sibling-site sweep (TD-VSDD-060)

Adding `Literal::RawTemporalLiteral` triggers the TD-VSDD-060 sibling-site sweep rule.
All internal `match` arms on `Literal` within `crates/prism-query/src/` must be
updated. External crates have `#[non_exhaustive]` wildcard arms already (enforced by
the perimeter-violation compile-fail gate). Missing internal arms will produce compile
errors — BUT only if the match is exhaustive (no wildcard). Some internal match arms
may silently fall through to a wildcard and produce incorrect behavior rather than a
compile error.

**Mitigation**: The implementer MUST run `grep -r 'Literal::' crates/prism-query/src/`
and audit every match. The mandatory `pipe_sql_emitter.rs` guard arm (Step 5 in §D4)
is the most critical — `RawTemporalLiteral` reaching the SQL emitter is an internal
logic error. All other arms must produce E-QUERY-002 (not silently succeed).

**Coercion arm blast-radius note (ratification modification):** The String-column
coercion arm is contained entirely within `check_temporal_literals` — it is one
additional branch on the resolved column type inside the existing AST walker. It does
not add new match sites on `Literal::RawTemporalLiteral` beyond what Option A already
requires; the TD-VSDD-060 sibling-site sweep scope is unchanged.

### RISK-5: `is_date_like` false positives in non-Datetime string comparisons — RESOLVED BY DESIGN

**Status: ELIMINATED** (hardened during human ratification, 2026-07-04).

This risk was identified in the v1.3 draft as a LOW accepted risk: the `is_date_like`
heuristic would classify `'2026-06-24'` as `RawTemporalLiteral`, causing
`check_temporal_literals` to reject `WHERE string_col = '2026-06-24'` with E-QUERY-001
when `string_col` is a String/Utf8 column. The human ratification review correctly
identified this as a real behavior regression, not "arguably a bug" — partition keys,
report-date labels, and ISO-date-formatted external IDs are legitimate String column
values, and analysts querying them with date-shaped string literals have a valid query
that must not break.

**Resolution:** The String-column coercion arm in `check_temporal_literals` detects
when a `RawTemporalLiteral(s)` is compared against a String/Utf8 column and rewrites
it in-place to `Literal::String(s)`. Because `pipe_sql_emitter.rs` emits
`Literal::String(s)` as a plain `'{escaped}'` SQL string literal — the same emission
path used before ADR-052 — the coercion is byte-identical to pre-ADR-052 behavior.
`WHERE string_col = '2026-06-24'` works exactly as it did before: no error, no
regression.

E-QUERY-041 fires ONLY against `DataType::Timestamp(Microsecond, UTC)` columns. The
false-positive is eliminated by construction. No E-QUERY-001 message guidance for this
case is needed because the case no longer produces an error.

---

## Recommended BC Amendments

### BC-2.11.021 — Temporal Grammar: NOW()/INTERVAL Planning-Time Constant Injection

**Current postcondition (§Postconditions, last point):**
> "DataFusion sees a concrete `WHERE timestamp > '2026-06-24T00:00:00Z'` comparison"

**Amended postcondition (after ADR-052):**
> "DataFusion sees a concrete
> `WHERE timestamp > arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some(\"UTC\"))')`
> comparison against a `Timestamp(Microsecond, UTC)` column."

**Current invariant (§Invariants):**
> "`Expr::Now` produces a UTC timestamp; the resulting `Literal::Timestamp` is
> RFC-3339 formatted with UTC offset"

**Amended invariant (after ADR-052):** No change needed — the invariant is about
`Literal::Timestamp` internal format (RFC-3339 string), which is unchanged. The SQL
emission format changes in D3 but the internal `Literal::Timestamp.iso8601` field
remains RFC-3339.

### BC-2.11.003 / BC-2.11.004 — Temporal filter predicates (if they specify column types)

Scan for any postcondition or assertion that describes Datetime columns as
`DataType::Utf8`. After ADR-052, all such assertions must be updated to
`DataType::Timestamp(Microsecond, UTC)`. The product-owner must perform this sweep
as part of the BC amendment burst triggered by ADR-052 ratification.

**E-QUERY-041 / E-QUERY-042 detection semantics for BC-2.11.003 / BC-2.11.004 (v1.10 revision):**
Any postcondition or AC describing E-QUERY-041 or E-QUERY-042 must specify the **Option A
lenient-parse-then-AST-walk mechanism** with the FULL 7-form acceptance set:

> "The PrismQL parser accepts the following offset-less date/datetime string literals
> as `Literal::RawTemporalLiteral` AST nodes (parse succeeds for all 7 forms):
> date-only (`'2026-06-24'`); T-separator full seconds (`'2026-06-24T12:00:00'`);
> T-separator fractional seconds (`'2026-06-24T12:00:00.123'`); T-separator no seconds
> (`'2026-06-24T12:00'`); space-separator full seconds (`'2026-06-24 12:00:00'`);
> space-separator fractional seconds (`'2026-06-24 12:00:00.500'`); space-separator
> no seconds (`'2026-06-24 12:00'`). The plan-time validator `check_temporal_literals`
> walks the resolved AST with the following seven-arm dispatch (v1.10): (1) for
> `RawTemporalLiteral` nodes in comparison position against a `Timestamp(Microsecond, UTC)`
> column (LHS is bare `Field`), E-QUERY-041 is raised; (2) for `RawTemporalLiteral` nodes
> in comparison position against a String/Utf8 column (LHS is bare `Field`), the node is
> rewritten in-place to `Literal::String(s)` and processing continues without error
> (byte-identical to pre-ADR-052 behavior); (3) for `RawTemporalLiteral` nodes against
> Integer/Float/Bool columns in comparison position (LHS is bare `Field`), E-QUERY-002 is
> raised (`QueryTypeMismatch` — date-shaped literal cannot equal a numeric or boolean
> column); (4) for `RawTemporalLiteral` nodes in comparison position where the LHS is a
> function or compound expression (non-`Field`), E-QUERY-042 is raised
> (`TemporalLiteralInvalidPosition`, NonColumnLhsComparison — LHS type cannot be resolved
> at plan time; silent coercion would reintroduce RISK-1 for datetime-valued LHS
> expressions); (5) for `RawTemporalLiteral` nodes in SELECT projection position (bare
> literal in SELECT list, no column type context), the node is coerced to
> `Literal::String(s)` (SUCCESS — standard SQL `SELECT '2026-06-24'` returns the string
> constant; OBS-2 preserved); (6) for `RawTemporalLiteral` **or `Literal::Timestamp`**
> nodes in GROUP BY position, E-QUERY-042 is raised (`TemporalLiteralInvalidPosition`,
> GroupBy — grouping by a literal constant, whether offset-less or RFC-3339 fast-path,
> is a degenerate no-op; v1.11 adds `Literal::Timestamp` as co-trigger per
> DEFECT-EQUERY042-GROUPBY-DEADARM-001); (7) for `RawTemporalLiteral` **or
> `Literal::Timestamp`** nodes in ORDER BY position, E-QUERY-042 is raised
> (`TemporalLiteralInvalidPosition`, OrderBy — ordering by a literal constant,
> whether offset-less or RFC-3339 fast-path, is a degenerate no-op; v1.11 adds
> `Literal::Timestamp` as co-trigger)."

Do NOT describe E-QUERY-041 as: a parse-time error, a DataFusion cast error, a text-
scanner result, or a raw-query-string scan. These descriptions applied to the retired
v1.2 design.

**BC-2.11.004 additional amendment guidance (v1.10 — F-MED-1):**

EC-11-004-004 currently describes pipe `| stats count by '2026-06-24'` as a "coerce
success" case. This is incorrect: the pipe parser rejects a string literal in `stats by`
position at parse time with E-QUERY-001. The product-owner must reclassify EC-11-004-004:

- Old classification: "coerce success — date-like literal in `stats by` coerced to string"
- New classification: FORBIDDEN / parse-time rejection — E-QUERY-001 with message
  "pipe 'stats by' expects column references (field names), not literal values"
- Rationale: the pipe `stats by` key parser accepts only `FieldPath`; a string literal
  never reaches `check_temporal_literals`; the rejection happens at the parser combinator
  level (see §D4 Pipe-mode Consistency section)

### Error Taxonomy (`prd-supplements/error-taxonomy.md`)

Add E-QUERY-041:
```
E-QUERY-041 | TemporalLiteralUnparseable | Query | Plan-time pre-validation |
The value '{first_50_chars}' cannot be interpreted as a UTC timestamp.
Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z').
Date-only and offset-less forms are not accepted. For relative time filters,
use NOW() - INTERVAL 'Nh'.
```

**Error taxonomy classification note (v1.3):** E-QUERY-041 emitter/phase = plan-time
AST validator (`check_temporal_literals`), raised when `Literal::RawTemporalLiteral`
is found in comparison position against a Datetime column. It is NOT a parse-time
error, NOT a DataFusion execution error, and NOT an Arrow `ArrowError::CastError`.

### Error Taxonomy — E-QUERY-042 (v1.10)

Add E-QUERY-042 with three position-specific messages:

```
E-QUERY-042 | TemporalLiteralInvalidPosition | Query | Plan-time AST validator |

(GroupBy message)
"E-QUERY-042: GROUP BY expects a column reference, not a literal constant.
'<first_50_chars>' is a date-shaped literal — grouping by a constant has no effect
and is almost certainly a query mistake. Did you mean to reference a column name,
or to add a WHERE filter before grouping?"

(OrderBy message)
"E-QUERY-042: ORDER BY expects a column reference, not a literal constant.
'<first_50_chars>' is a date-shaped literal — ordering by a constant has no effect.
Did you mean to reference a column name that contains this value?"

(NonColumnLhsComparison message)
"E-QUERY-042: A date-like literal compared against a computed expression cannot be
type-checked at plan time. Compare against a bare datetime column using RFC-3339
(e.g., '2026-07-03T00:00:00Z'), against a string column using a non-date-shaped
value, or wrap the expression in an explicit CAST."
```

**E-QUERY-042 taxonomy notes (for the PO to register):**
- MCP mapping: `-32602 INVALID_PARAMS` for all three position variants (caller-resolvable)
- Emitted by: `PrismError::TemporalLiteralInvalidPosition { position: TemporalLiteralPosition, value_prefix: String }` where `TemporalLiteralPosition` is a new enum `{ GroupBy, OrderBy, NonColumnLhsComparison }` in `prism-core/src/error.rs`
- `value_prefix` = first 50 chars of the offending literal (same truncation convention as E-QUERY-041 / E-INFUSE-014; AD-017 belt-and-suspenders guard)
- `map_prism_error` constraint: must add an explicit `-32602 INVALID_PARAMS` arm for `PrismError::TemporalLiteralInvalidPosition` — MUST NOT fall through to the catch-all `-32000`
- E-QUERY-042 is NOT a sub-code of E-QUERY-041: E-QUERY-041 fires for Datetime/Timestamp column comparisons specifically; E-QUERY-042 fires for structural/positional problems (wrong position, or unresolvable LHS type) regardless of column type
- Gate ordering: E-QUERY-001 (parse) → E-QUERY-041 / E-QUERY-042 (plan-time `check_temporal_literals`) → DataFusion execution; E-QUERY-041 and E-QUERY-042 can both be raised within the same `check_temporal_literals` invocation if the query has multiple predicate positions
- SAP-1 obligation: any `tracing::*!(event_type=…)` emitted at E-QUERY-042 detection time requires a BC-2.16.002 catalog row in the same commit

---

## Blast Radius (TD-VSDD-060 Sweep)

The following files require changes or verification. Items marked [CHANGE] require
code modifications; items marked [VERIFY] require review to confirm no change is needed.

| # | File | Nature | Change Required |
|---|------|--------|----------------|
| 1 | `crates/prism-bin/src/spec_driven_adapter.rs:886` | [CHANGE] | `ColumnType::Datetime => DataType::Timestamp(...)` |
| 2 | `crates/prism-core/src/column.rs:28` | [CHANGE] | Fix stale doc comment |
| 3 | `crates/prism-query/src/pipe_sql_emitter.rs:822` | [CHANGE] | `format!("arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')", ts.iso8601)` — explicit arrow_cast form (TIMESTAMP '...' produces Nanosecond/None in DF 53.1.0) |
| 4 | `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` | [CHANGE] | Update `DataType::Utf8` datetime column assertions to `DataType::Timestamp(Microsecond, UTC)` |
| 5 | `crates/prism-query/src/pushdown.rs` | [VERIFY] | Already uses `ts.instant.to_rfc3339()` — no change needed |
| 6 | `crates/prism-query/src/infusion_udf.rs` | [VERIFY] | After ADR-052 ships, the datetime row in the `output_type` mapping (ADR-051) must be updated to `Timestamp` — no change in ADR-052 story itself |
| 7 | `crates/prism-spec-engine/src/infusion/udf.rs` | [VERIFY] | `InfusionUdfDescriptor.output_type` is a `String` — no change at spec-engine level |
| 8 | `specs/infusions/*.infusion.toml` | [VERIFY] | No TOML schema change; `output_type = "datetime"` is the string that ADR-051 will map to the new type after ADR-052 |
| 9 | `crates/prism-query/src/pipe_sql_emitter.rs:817-818` | [CHANGE] | Update stale comment "Datetime fields is DataType::Utf8" to reflect the new type; add `Literal::RawTemporalLiteral` arm (E-QUERY-002 guard) |
| 10 | `.factory/specs/architecture/decisions/ADR-044-*.md` | [DONE] | `superseded_by` frontmatter added; §Status "PARTIALLY SUPERSEDED by ADR-052 v1.1" block added (2026-07-03) |
| 11 | `.factory/specs/prd-supplements/error-taxonomy.md` | [CHANGE] | Add E-QUERY-041 row; phase = "plan-time AST validator (`check_temporal_literals`)" |
| 12 | `.factory/specs/behavioral-contracts/BC-2.11.021-*.md` | [CHANGE] | Amend postcondition to Option A mechanism; E-QUERY-041 = `check_temporal_literals` walker, NOT parse-fail/text-scanner/DataFusion-cast-intercept |
| 13 | `crates/prism-query/src/ast.rs` | [CHANGE] | Add `Literal::RawTemporalLiteral(String)` variant; doc comment per §D4 |
| 14 | `crates/prism-query/src/sql_parser.rs` (+ `filter_parser.rs` if separate) | [CHANGE] | Modify timestamp literal production: lenient `is_date_like` fallback → `RawTemporalLiteral` instead of parse error. `filter_parser.rs::build_predicate_parser` now includes the `fn_call_comparison` production (`FuncCall::Scalar` only) as of DEFECT-PQL-FNCALL-LHS-001 (2026-07-13 [H5c]), admitting function-call LHS in pipe `| where` predicates and making the NonColumnLhsComparison arm in `check_temporal_literals` grammar-reachable from pipe mode (was defense-in-depth-only prior to this). |
| 15 | `crates/prism-query/src/engine.rs` | [CHANGE] | DELETE `extract_table_name_from_query_str`, `extract_column_name_adjacent_to_quoted_value`, `is_bad_literal_in_datetime_column`, parse-fail branch; ADD early-gate INVOCATION of `materialization.rs::check_temporal_literals` (DEFINITION lives in `materialization.rs` — see row 21) |
| 16 | `crates/prism-query/src/tests/` | [CHANGE] | Rewrite E-QUERY-041 RG tests from parse-fail path to `check_temporal_literals` path; add Unicode VP-021 regression test (non-ASCII input → no panic → correct E-QUERY-041/001) |
| 17 | `crates/prism-query/src/` — all `match` on `Literal` | [CHANGE] | TD-VSDD-060 sweep: `grep -r 'Literal::' crates/prism-query/src/` — add `Literal::RawTemporalLiteral` arm to every internal match |
| 18 | `crates/prism-query/src/tests/` — Utf8 datetime assertions | [CHANGE] | Grep for `DataType::Utf8` assertions on Datetime columns; update to `DataType::Timestamp(Microsecond, UTC)` |
| 19 | `crates/prism-sensors/` (normalization paths) | [CHANGE] | Add ISO-8601 string → microseconds-since-epoch parsing at OCSF normalization boundary for Datetime fields |
| 20 | `crates/prism-query/src/` — remaining Utf8 datetime refs | [VERIFY] | `grep -r 'DataType::Utf8' crates/prism-query/src/` to catch any residual hardcoded Utf8 for datetime columns |
| 21 | `crates/prism-query/src/materialization.rs` | [CHANGE] | ADD `check_temporal_literals` function DEFINITION — plan-time AST walker with seven-arm dispatch (§D4 Step 3, v1.10 + v1.11 co-trigger): (1) `RawTemporalLiteral` + Timestamp/Datetime col comparison (bare `Field` LHS) → E-QUERY-041; (2) `RawTemporalLiteral` + String/Utf8 col comparison (bare `Field` LHS) → COERCE to `Literal::String(s)` (SUCCESS); (3) `RawTemporalLiteral` + Integer/Float/Bool col comparison (bare `Field` LHS) → E-QUERY-002 `QueryTypeMismatch`; (4) `RawTemporalLiteral` in comparison with non-`Field` LHS → E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison); (5) `RawTemporalLiteral` in SELECT projection (no column type context) → COERCE to `Literal::String(s)` (SUCCESS, OBS-2); (6) GROUP BY position — `RawTemporalLiteral` **or `Literal::Timestamp`** → E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy); (7) ORDER BY position — `RawTemporalLiteral` **or `Literal::Timestamp`** → E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy). `Literal::Timestamp` nodes in all other positions pass through without coercion (already-resolved RFC-3339 form). Walker MUST track AST position context (SELECT projection / GROUP BY / ORDER BY / comparison with bare-`Field` LHS / comparison with non-`Field` LHS). Early-gate INVOCATION remains in `engine.rs` (row 15). `execute_against_session` `Ast::Sql` and `Ast::SqlPipe` arms continue to call `PqlNormalizer::normalize_for_datafusion` (v1.6 / unchanged). |
| 23 | pipe-stage parser combinator for `stats` and `sort` key lists | [CHANGE] | Improve parse error messages per §D4 Pipe-mode Consistency (v1.10 option (a)): when `FieldPath` combinator fails on a single-quoted date-like string at `stats … by` key position, produce analyst-facing message "pipe 'stats by' expects column references (field names), not literal values"; same flavor for `sort` key position. Maps to E-QUERY-001 (`QueryParseFailed`, MCP `-32602 INVALID_PARAMS`). No grammar extension. (Exact crate/file: `crates/prism-query/src/pipe_parser.rs` or equivalent stats/sort stage parser.) |
| 22 | `crates/prism-query/src/ast.rs` | [CHANGE] | ADD `NORMALIZE_FOR_DATAFUSION: Cell<bool>` thread-local; ADD `PqlNormalizer::normalize_for_datafusion(ast)` method with save-and-restore drop-guard; ADD `normalize_literal_dispatch` (flag-sensitive literal dispatch: `Literal::Timestamp` → `normalize_literal_for_datafusion` when flag set, else `normalize_literal`); ADD `normalize_literal_for_datafusion` (`arrow_cast` emitter for `Literal::Timestamp`). `PqlNormalizer::normalize_literal` (BC-2.11.018 round-trip) is NOT modified — continues to emit `'{iso8601}'`. |

---

## D4 Story Impact (S-PRISMQL-NATIVE-TEMPORAL-TYPING-001)

### Tasks to DELETE (text-scanner apparatus)

The following tasks/ACs in the story are deleted by Option A. The implementer must
remove these from the story spec before TDD begins (or the test-writer must write
zero tests for them):

| Deleted artifact | File | Reason |
|-----------------|------|--------|
| `extract_table_name_from_query_str` | `engine.rs` | Replaced by `check_temporal_literals` schema lookup |
| `extract_column_name_adjacent_to_quoted_value` | `engine.rs` | Same |
| `is_bad_literal_in_datetime_column` | `engine.rs` | Same |
| Parse-fail branch that invokes the above | `engine.rs` | E-QUERY-041 is no longer raised on parse failure |

### Tasks to ADD (Option A)

| New task | File | Description |
|---------|------|-------------|
| Add `Literal::RawTemporalLiteral(String)` | `ast.rs` | New variant with doc comment per §D4 |
| Modify timestamp literal parser combinator | `sql_parser.rs` (+ `filter_parser.rs` if separate) | Lenient `is_date_like` fallback; emit `RawTemporalLiteral` instead of parse error for date-only + offset-less |
| Implement `check_temporal_literals` | `materialization.rs` (DEFINITION); early-gate INVOCATION in `engine.rs` | AST walker with seven-arm dispatch (v1.10 + v1.11 co-trigger): (1) `RawTemporalLiteral` + Timestamp/Datetime col (bare `Field` LHS) → E-QUERY-041; (2) `RawTemporalLiteral` + String/Utf8 col (bare `Field` LHS) → COERCE in-place to `Literal::String(s)` (SUCCESS); (3) `RawTemporalLiteral` + Integer/Float/Bool col (bare `Field` LHS) → E-QUERY-002 `QueryTypeMismatch`; (4) `RawTemporalLiteral` in comparison where LHS is non-`Field` (function/compound expression) → E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison); (5) `RawTemporalLiteral` in SELECT projection (bare literal in SELECT list, no column type context) → COERCE in-place to `Literal::String(s)` (SUCCESS, OBS-2); (6) GROUP BY position — `RawTemporalLiteral` **or `Literal::Timestamp`** → E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy); (7) ORDER BY position — `RawTemporalLiteral` **or `Literal::Timestamp`** → E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy). `Literal::Timestamp` in all other positions passes through without coercion (already-resolved RFC-3339 form). Walker MUST track AST position context (SELECT projection / GROUP BY / ORDER BY / comparison with bare-`Field` LHS / comparison with non-`Field` LHS) to dispatch correctly. |
| Improve pipe `stats by` and `sort` parse error messages | pipe-stage parser combinator for `stats` and `sort` key lists | When `FieldPath` combinator fails on a single-quoted string at `stats … by` key position, produce: "pipe 'stats by' expects column references (field names), not literal values — '<value>' looks like a date-shaped literal, not a column name. Grouping by a literal constant has no effect." When `FieldPath` fails on single-quoted string at `sort` key position, produce: "pipe 'sort' expects column references (field names), not literal values — ordering by a literal constant has no effect." Both map to E-QUERY-001 (`QueryParseFailed`, MCP `-32602 INVALID_PARAMS`). No grammar extension — parser already rejects; this improves message quality only. |
| Implement `PqlNormalizer::normalize_for_datafusion` and `normalize_literal_dispatch` | `ast.rs` | Thread-local scoped-context DataFusion emission mechanism. ADD `NORMALIZE_FOR_DATAFUSION: Cell<bool>` thread-local; `normalize_for_datafusion(ast)` saves-and-restores flag via drop-guard (re-entrant-safe), sets flag to `true`, calls `normalize(ast)` (unchanged); `normalize_literal_dispatch` routes `Literal::Timestamp` to `normalize_literal_for_datafusion` (arrow_cast emitter) when flag set, else delegates to `normalize_literal` (unchanged, BC-2.11.018). Call sites: `execute_against_session` `Ast::Sql` arm AND `Ast::SqlPipe` head-SQL arm (both in `materialization.rs`). `PqlNormalizer::normalize_literal` is NOT modified. |
| Add `Literal::RawTemporalLiteral` guard arm | `pipe_sql_emitter.rs` | Must never reach emission — E-QUERY-002 (`QueryPlanFailed`) internal error guard |
| TD-VSDD-060 sibling-site sweep on `Literal` | `prism-query/src/*.rs` | Add `RawTemporalLiteral` arm to all internal `match` on `Literal` |

### Red Gate tests that change

| Old RG test | New RG test |
|------------|------------|
| "parse `WHERE ts > '2026-06-24'` → parse fails → E-QUERY-041 via text-scanner" | "parse `WHERE ts > '2026-06-24'` → parse succeeds → `check_temporal_literals` → E-QUERY-041" |
| "parse `WHERE ts > '2026-06-24T12:00:00'` → parse fails → E-QUERY-041" | Same, with `check_temporal_literals` path |
| VP-021 Unicode panic test (if exists) | "non-ASCII query input → no panic → E-QUERY-041 or E-QUERY-001 (no byte-offset crash)" |
| (new) | "`RawTemporalLiteral` vs Integer/Float/Bool column → E-QUERY-002 `QueryTypeMismatch` (not E-QUERY-041, not E-QUERY-001)" |
| (new) | "`RawTemporalLiteral` vs String/Utf8 column → COERCE → compare as string literal (SUCCESS — no E-QUERY error emitted; e.g. `WHERE string_col = '2026-06-24'` works)" |
| (new) | "filter-mode `source \| WHERE ts > '2026-06-24'` → E-QUERY-041 (not misclassified)" |
| (new) | "dotted column `payload.ts > '2026-06-24'` → E-QUERY-041 (schema resolves correctly)" |
| (new, v1.4) | "`WHERE ts > '2026-06-24T12:00'` (T-sep, no seconds) vs Datetime col → parse succeeds → E-QUERY-041" |
| (new, v1.4) | "`WHERE ts > '2026-06-24 12:00:00'` (space-sep) vs Datetime col → parse succeeds → E-QUERY-041" |
| (new, v1.4) | "`WHERE ts > '2026-06-24T12:00:00.123'` (fractional) vs Datetime col → parse succeeds → E-QUERY-041" |
| (new, v1.4) | "`WHERE string_col = '2026-06-24 12:00:00'` (space-sep vs String/Utf8 col) → COERCE → compare as string literal (SUCCESS — no E-QUERY error)" |
| (new, v1.8) | "`SELECT '2026-06-24' FROM t` (date-like literal in projection, no column type context) → COERCE → `Literal::String("2026-06-24")` returned as string constant (SUCCESS — no E-QUERY-002 `QueryPlanFailed`)" |
| (new, v1.8) | "`SELECT '2026-06-24T12:00:00' FROM t` (datetime-like literal in projection) → COERCE → `Literal::String("2026-06-24T12:00:00")` (SUCCESS)" |
| (new, v1.10) | "`SELECT hostname FROM t GROUP BY '2026-06-24'` (GROUP BY bare date literal) → E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy) — NOT coerce, NOT E-QUERY-002" |
| (new, v1.10) | "`SELECT hostname FROM t ORDER BY '2026-06-24'` (ORDER BY bare date literal) → E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy) — NOT coerce, NOT E-QUERY-002" |
| (new, v1.11) | "`SELECT hostname FROM t GROUP BY '2026-07-01T00:00:00Z'` (GROUP BY RFC-3339 constant, `Literal::Timestamp`) → E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy) — `Literal::Timestamp` co-trigger; DEFECT-EQUERY042-GROUPBY-DEADARM-001" |
| (new, v1.11) | "`SELECT hostname FROM t ORDER BY '2026-07-01T00:00:00Z'` (ORDER BY RFC-3339 constant, `Literal::Timestamp`) → E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy) — `Literal::Timestamp` co-trigger; DEFECT-EQUERY042-GROUPBY-DEADARM-001" |
| (new, v1.10) | "`WHERE lower(hostname) = '2026-06-24'` (non-column LHS function call, date-like RHS) → E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison), MCP `-32602 INVALID_PARAMS` — NOT `-32000 QueryPlanFailed`" |
| (new, v1.10) | "`SELECT '2026-06-24' FROM t` (SELECT projection, OBS-2 regression guard) → COERCE SUCCESS — E-QUERY-042 must NOT be raised for projection position" |
| (new, v1.10) | "`\| stats count by '2026-06-24'` (pipe stats-by position) → E-QUERY-001 parse error with message containing 'pipe stats by expects column references, not literal values'" |
| (new, v1.10) | "`\| sort '2026-06-24'` (pipe sort position) → E-QUERY-001 parse error with message containing 'pipe sort expects column references, not literal values'" |

---

## Considered Alternatives

**A1 — Keep Utf8, add CAST in predicates on query entry.**
Wrap analyst-authored datetime predicates in `CAST(? AS TIMESTAMP)` at the query
planner before emitting to DataFusion. This avoids changing the column registration
but pushes complexity into the query rewriting path and requires detecting "this
is a datetime column comparison" at the planner layer — exactly the kind of
context-dependent type inference that a proper type system makes unnecessary.
Rejected: complexity cost exceeds migration cost; doesn't fix the OCSF contract
gap; doesn't unblock ADR-051 datetime typing.

**A2 — Keep Utf8, enforce `Z` suffix in OCSF normalization.**
Guarantee that all datetime values emitted by sensor adapters end in `Z` (not
`+00:00`), making lexicographic ordering correct by construction. This maintains
the accidental-correctness property.
Rejected: This is a sentinel arrangement masking a type gap. It does not provide
temporal arithmetic, does not align with OCSF type semantics, and does not
unblock ADR-051.

**A3 — Use `Timestamp(Nanosecond, Some("UTC"))` instead of Microsecond.**
Nanosecond is the DataFusion default for `TIMESTAMP '...'` literals. Using
Nanosecond would eliminate precision-mismatch risk (RISK-1) at the cost of:
overflow at ~2262 CE timestamps (irrelevant for security sensor data),
minor memory overhead (8 bytes either way), and deviation from the Microsecond
baseline established in `high002_plan_pinning_tests.rs`.

**B — Parser-carried predicate context (rejected for §D4 redesign).**
On parse failure for an invalid timestamp literal, record the enclosing predicate's
column name and operator in the structured error type. The plan-time validator
consumes the structured error context to raise E-QUERY-041 without text scanning.
Rejected: the parser still doesn't have schema information at parse time — it can
record the column NAME but cannot resolve `source.column`, `payload.field`, or
filter-mode `source |` syntax into a concrete schema type. The structured error carries
a raw name that has the same resolution gap as the text-scanner. Option B replaces
a text-scanner with a name-in-error-struct; same failure modes, different data
structure. Option A (lenient-parse-then-AST-walk) eliminates the gap entirely by
deferring validation to plan time where schema is fully resolved.
Rejected: Microsecond is the established codebase baseline; overflow is irrelevant
for security sensor use cases; RISK-1 is mitigated by a probe test in the
implementation story.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.12 | 2026-07-13 | architect | **DEFECT-PQL-FNCALL-LHS-001 [H5c]: NonColumnLhsComparison arm grammar-reachable from pipe `| where` mode.** §D4 NonColumnLhsComparison arm (arm 4/5) note added: `fn_call_comparison` production in `build_predicate_parser` (`FuncCall::Scalar` only) extends pipe `| where` predicate grammar to admit function-call LHS comparisons (DEFECT-PQL-FNCALL-LHS-001, 2026-07-13, live-audit [H5c]), making this arm grammar-reachable from pipe mode — was defense-in-depth-only prior. Arm behavior unchanged: date-like RHS → E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison) at plan time; non-date-like RHS → valid, passes to DataFusion; fn-call args participate in E-QUERY-038 column walk. Pipe `| where` grammar reachability subsection added to rejection arm narrative. Blast-radius row 14 (`filter_parser.rs::build_predicate_parser`) updated. Option-(a) stats-by/sort treatment unchanged. |
| 1.11 | 2026-07-10 | architect | **LOCAL adversary finding F-EQ42-P1-001 (HIGH) closing DEFECT-EQUERY042-GROUPBY-DEADARM-001: `Literal::Timestamp` co-trigger for GROUP BY/ORDER BY rejection.** §D4 Step-3 dispatch table arms (6) GROUP BY and (7) ORDER BY: `Literal::Timestamp` (RFC-3339 fast-path constant, e.g. `'2026-07-01T00:00:00Z'`) added as co-trigger alongside `Literal::RawTemporalLiteral` — a pre-parsed RFC-3339 constant in GROUP BY/ORDER BY is equally degenerate (constant grouping/ordering key); the `RawTemporalLiteral`-only wording in v1.10 was the dead-arm defect. Arm (5) NonColumnLhsComparison remains `RawTemporalLiteral`-only (code scope). Behavior Reference Table: two rows added covering RFC-3339 form → `Literal::Timestamp` → E-QUERY-042 (GroupBy/OrderBy). Clarifying sentence added: `Literal::Timestamp` in SELECT projection / JOIN ON / function args passes through `check_temporal_literals` without coercion (already-validated RFC-3339 form); only the degenerate-position gates (GROUP BY/ORDER BY) intercept it. GROUP BY/ORDER BY rejection arm narratives updated; BC Amendments detection semantics arms (6)/(7) updated; Blast Radius row 21 arms (6)/(7) updated; Tasks-to-ADD `check_temporal_literals` row arms (6)/(7) updated; Red Gate test table: two new rows added (RFC-3339 GROUP BY and ORDER BY). |
| 1.10 | 2026-07-05 | architect | **Human-ratified §D4 refinement: GROUP BY/ORDER BY REJECT + non-column-LHS REJECT + pipe-mode mechanism.** The v1.8 flat "non-comparison-position → COERCE" arm is split: (4a) SELECT projection → COERCE `Literal::String(s)` (OBS-2 preserved); (4b) GROUP BY → E-QUERY-042 `TemporalLiteralInvalidPosition` (GroupBy); (4c) ORDER BY → E-QUERY-042 `TemporalLiteralInvalidPosition` (OrderBy). New arm (5): comparison with non-`Field` LHS (function/expression) and date-like RHS → E-QUERY-042 `TemporalLiteralInvalidPosition` (NonColumnLhsComparison) — closes current bug where `WHERE lower(hostname) = '2026-06-24'` produces `QueryPlanFailed → -32000 INTERNAL_ERROR`. New error code E-QUERY-042 (`TemporalLiteralInvalidPosition`) specified with three position-specific messages (GroupBy, OrderBy, NonColumnLhsComparison); emitted by `PrismError::TemporalLiteralInvalidPosition { position: TemporalLiteralPosition, value_prefix: String }`; maps to MCP `-32602 INVALID_PARAMS`. Pipe-mode mechanism: option (a) chosen (improved parse error message; lower complexity; `stats by` / `sort` already fail at parse; `check_temporal_literals` is SQL-mode only for GROUP/ORDER). BC-2.11.004 EC-11-004-004 correction guidance added: pipe stats-by literal must REJECT at parse time (E-QUERY-001), not coerce. Step 3 dispatch table: 5 rows → 8 rows. Coercion arm narrative split into projection-coerce + 3 rejection subsections. Step 5 guard updated: 4 paths → 7 paths. E-QUERY boundary table: 1 non-comparison coerce row → 4 rows. `§D4 Pipe-mode Consistency` subsection added. BC Amendments detection semantics block: four-way → seven-arm. E-QUERY-042 error specification added to Error Taxonomy section. Tasks-to-ADD: `check_temporal_literals` row updated (four-way→seven-arm + context tracking), pipe parser message task added. Red Gate tests: 6 new rows (GROUP BY REJECT, ORDER BY REJECT, non-column-LHS REJECT, projection OBS-2 regression guard, pipe stats-by parse reject, pipe sort parse reject). Blast radius: row 21 updated (four-way→seven-arm + context tracking), row 23 added (pipe parser message improvement). Arms 1/2/3 (Datetime-col→E-QUERY-041, String-col→COERCE, Int/Float/Bool→E-QUERY-002) and emitter guard (Step 5, belt-and-suspenders E-QUERY-002 `QueryPlanFailed`) UNCHANGED. ARCH-INDEX v2.169→v2.170. |
| 1.9 | 2026-07-05 | architect | **MED-1: correct §D4 dispatch label three-way→four-way (OBS-2 arm added in v1.8 but BC Amendments guidance block label was not updated).** Single live instance corrected: Recommended BC Amendments §E-QUERY-041 detection semantics guidance block for BC-2.11.003/BC-2.11.004 — "walks the resolved AST with a **three-way** column-type dispatch" changed to "**four-way** column-type dispatch". The block itself already enumerated all four arms (Timestamp-col→E-QUERY-041; String/Utf8-col→COERCE; Integer/Float/Bool-col→E-QUERY-002; non-comparison-position→COERCE); only the label was stale. Root cause: v1.8 updated the Tasks-to-ADD row and blast-radius row 21 from three-way→four-way but did not propagate the label correction to the BC Amendments guidance block. Confirmation: zero residual live `Literal::Utf8` in non-changelog sections; zero mis-coded `E-QUERY-001` in live behavioral specification (all remaining E-QUERY-001 references are historical context, section headers, or explicit "not E-QUERY-001" test assertions). ARCH-INDEX v2.168→v2.169. |
| 1.8 | 2026-07-04 | architect | **(OBS-2 human-ratified + F-LOW-1 + OBS-1): three §D4 corrections.** **(CHANGE 1 / OBS-2)** Non-comparison-position `RawTemporalLiteral` COERCES to `Literal::String(s)` (SUCCESS) instead of E-QUERY-002 (`QueryPlanFailed`). Human-ratified: a date-like literal in projection/GROUP BY/ORDER BY/unconstrained position has no column type to constrain it and is therefore a plain string constant; standard SQL `SELECT '2026-06-24'` returns the string; consistent with RISK-5 String-column coercion philosophy. Rejecting it was over-strict. Updated: Step-3 dispatch table last row (error→coerce), E-QUERY boundary table (new non-comparison coercion row), coercion-arm narrative (new non-comparison subsection added after String-column arm), Step-5 guard prose (guard now explicitly unreachable — all non-error paths coerce before emission, with enumerated four-branch summary), BC Amendments §D4 (3)/(4) clause (fourth arm: non-comparison → coerce, not error), Tasks-to-ADD `check_temporal_literals` row (three-way→four-way), blast-radius row 21 (three-way→four-way). Datetime-col→E-QUERY-041, String-col→coerce, Integer/Float/Bool-comparison→E-QUERY-002 arms UNCHANGED. **(CHANGE 2 / F-LOW-1)** SQL-mode DataFusion emission addendum self-contradiction fixed. v1.6 claimed `normalize()` "delegates to `normalize_literal` (not `normalize_literal_dispatch`)"; shipped code routes through `normalize_literal_dispatch` (via `normalize_literal_as_expr`). Isolation is by-flag (`NORMALIZE_FOR_DATAFUSION` unset for round-trip callers), not by-separate-method. Corrected paragraph: normalize() routes all literal emission through `normalize_literal_dispatch`, which checks the thread-local; when flag is unset (round-trip callers never set it), dispatch delegates to `normalize_literal` (bare string). **(CHANGE 3 / OBS-1)** Four `Literal::Utf8` references in §D4 replaced with `Literal::String`: (1) Step-2 parser code block `else` arm; (2) `is_date_like` description prose; (3) non-match forms section header; (4) E-QUERY boundary table `'not-a-date'` row. No `Literal::Utf8` variant exists; actual variant is `Literal::String`. ARCH-INDEX v2.167→v2.168. |
| 1.7 | 2026-07-04 | architect | **MED-1: reconcile §D4 Step-5 emitter-guard refs E-QUERY-001→E-QUERY-002 (align to shipped `QueryPlanFailed` + RG-024 + dispatch table).** Four §D4 references corrected from E-QUERY-001 to E-QUERY-002 (`QueryPlanFailed`): (1) §D4 Step-5 body — `pipe_sql_emitter.rs` guard arm return value; (2) RISK-4 narrative — "all other arms must produce E-QUERY-002"; (3) blast-radius row 9 — "E-QUERY-002 guard"; (4) Tasks-to-ADD guard arm row — "E-QUERY-002 (`QueryPlanFailed`) internal error guard". Root cause: §D4 v1.5 FIX A corrected the dispatch table rows and boundary table for Integer/Float/Bool columns and non-comparison positions, but did not propagate the correction to Step-5/RISK-4/blast-row-9/task-table. The `pipe_sql_emitter.rs` guard is a plan-time defensive check (invariant violated if `RawTemporalLiteral` reaches emission) — correctly classified as `QueryPlanFailed` (E-QUERY-002), not `QueryParseFailed` (E-QUERY-001). Source of truth: shipped `PrismError::QueryPlanFailed` (= E-QUERY-002 per error-taxonomy.md v2.12), story RG-024, §D4 dispatch table "Non-comparison position with no resolvable String context → E-QUERY-002 `QueryPlanFailed`". No code change — code is correct. ARCH-INDEX v2.166→v2.167. |
| 1.6 | 2026-07-04 | architect | **LOCAL adversary cascade spec↔code alignment (FIX 1 / LOW-2 + FIX 2 / HIGH-1 sibling).** (FIX 1 / LOW-2) §D4 SQL-Mode Emission Addendum corrected: v1.5 described a phantom `emit_literal_for_datafusion` free function in `materialization.rs`; the SHIPPED mechanism is a thread-local scoped-context in `ast.rs` — `NORMALIZE_FOR_DATAFUSION: Cell<bool>` set by `PqlNormalizer::normalize_for_datafusion` with save-and-restore drop-guard; `normalize_literal_dispatch` routes `Literal::Timestamp` to `normalize_literal_for_datafusion` (arrow_cast emitter) when flag set; `PqlNormalizer::normalize_literal` (BC-2.11.018 round-trip) unchanged. Rationale for thread-local: avoids threading emission-mode parameter through entire recursive normalizer call tree; save-and-restore makes it re-entrant-safe. Phantom `emit_literal_for_datafusion` symbol removed. Blast-radius row 22 reclassified `[VERIFY/NO-CHANGE]` → `[CHANGE]` (4 new symbols: `NORMALIZE_FOR_DATAFUSION`, `normalize_for_datafusion`, `normalize_literal_dispatch`, `normalize_literal_for_datafusion` all in `ast.rs`). Rationale point 2 updated (2 paths → 3 paths). Tasks-to-ADD table updated. Section header updated. (FIX 2 / HIGH-1 sibling) `execute_against_session` `Ast::SqlPipe` head-SQL derivation used round-trip `normalize` (bare `'{iso8601}'`) — same RISK-1 violation as the `Ast::Sql` arm. Fix: `Ast::SqlPipe` arm also routes through `PqlNormalizer::normalize_for_datafusion`. Mode-agnostic arrow_cast invariant stated: NO DataFusion-executed query, in any mode, may emit a bare Utf8 timestamp literal against a Timestamp column. Three-row emission-path table added to Addendum. Blast-radius row 21 updated (SqlPipe arm added). BC-2.11.021 unchanged (already requires mode-agnostic arrow_cast). Blast radius rows 21-22 updated (count remains 22). ARCH-INDEX v2.165→v2.166. |
| 1.5 | 2026-07-04 | architect | **LOCAL adversary cascade fix-burst: three §D4 corrections.** (FIX A / MED-1) E-QUERY-001→E-QUERY-002 for `RawTemporalLiteral` against Integer/Float/Bool columns (`QueryTypeMismatch` — has structured `column`/`table`/`actual_type`/`operator` fields) and non-comparison positions without String context (`QueryPlanFailed` — unresolvable literal position). Updated in: Step-3 dispatch table (2 rows), E-QUERY-002/E-QUERY-041 dispatch boundary table (1 row), Recommended BC Amendments §D4 (3) clause, Red Gate tests table (1 row), Tasks-to-ADD table. Error-taxonomy.md v2.12 is the source of truth confirming both arms use E-QUERY-002. (FIX B / LOW-1) `check_temporal_literals` DEFINED in `materialization.rs` (not `engine.rs`); `engine.rs` retains early-gate INVOCATION only. Corrected in: §D4 Step-3 heading, blast-radius row 15, Tasks-to-ADD table. (FIX C / HIGH-1) SQL-mode DataFusion emission path (`materialization.rs::execute_against_session` `Ast::Sql` arm → `PqlNormalizer::normalize_literal`) emits bare `'{iso8601}'` for `Literal::Timestamp`, violating BC-2.11.021 (implicit coercion, RISK-1). Added `### SQL-Mode DataFusion Emission Addendum` to §D4: dedicated `emit_literal_for_datafusion` function in `materialization.rs` wraps Timestamp in `arrow_cast(...)` for the DataFusion path; `PqlNormalizer::normalize_literal` UNCHANGED (BC-2.11.018 round-trip contract preserved). Blast radius expanded from 20 to 22 rows (rows 21–22). Rationale point 2 updated to note both emission paths. BC-2.11.021 NOT weakened. |
| 1.4 | 2026-07-04 | architect | §D4 `is_date_like` acceptance set expanded (pre-TDD remove-uncertainty): 2 format strings → 7 format strings — adds space-separator forms (`%Y-%m-%d %H:%M:%S`, `%Y-%m-%d %H:%M:%S%.f`, `%Y-%m-%d %H:%M`), T-separator no-seconds form (`%Y-%m-%dT%H:%M`), and T-separator fractional form (`%Y-%m-%dT%H:%M:%S%.f`). Over-match disposition (unpadded digits via `%m`/`%d`, big/signed years via `%Y`) documented as ACCEPTED BENIGN — no regex layer or year-width guard. `### is_date_like Acceptance Set (Canonical)` subsection added with matched/over-matched/rejected tables. E-QUERY-001↔E-QUERY-041 boundary table updated with 5 new example rows (3 new forms vs Datetime col, 1 new form vs String col). RG test table expanded with 4 new tests. BC-2.11.021 amendment text updated to enumerate all 7 accepted forms. D1–D3, D5–D8, all RISK entries, Blast Radius table: unchanged. |
| 1.1 (ratified) | 2026-07-03 | state-manager | Human ratification recorded 2026-07-03 (D-1520). Status: PROPOSED → ACCEPTED. No decision content changes; v1.1 content ratified as authored. |
| 1.3 | 2026-07-04 | architect | §D4 ACCEPTED (human-ratified 2026-07-04, Option A + String-column coercion modification): E-QUERY-041 detection replaced from parse-fail text-scanner to lenient-parse-then-AST-walk. New `Literal::RawTemporalLiteral` AST node; `check_temporal_literals` walker uses three-way column-type dispatch — Timestamp col → E-QUERY-041; String/Utf8 col → COERCE to `Literal::String(s)` (SUCCESS, byte-identical no-op, RISK-5 eliminated); Integer/Float/Bool col → E-QUERY-001. Text-scanner functions deleted. RISK-5 reclassified from LOW accepted to RESOLVED BY DESIGN. Blast radius 20 files; no new sibling-site sweep scope from coercion arm (contained in `check_temporal_literals`). Option B evaluated and rejected. |
| 1.2 | 2026-07-04 | architect | OBS-4 typo fix: `Arc::from("UTF")` → `Arc::from("UTC")` in §D1 canonical construction form (adversary LOCAL cascade catch) |
| 1.1 | 2026-07-03 | architect | remove-uncertainty PASS-1 amendments: D3 emitter changed to arrow_cast (TIMESTAMP '...' → Nanosecond/None in DF 53.1.0); D4 E-QUERY-041 changed from DataFusion cast-failure intercept to Prism-level chrono pre-validator (arrow-cast 58.2.0 lenient — accepts date-only); Arrow construction form corrected Arc::new("UTF".into())→Arc::from("UTC"); RISK-1 downgraded HIGH→MEDIUM (arrow_cast eliminates coercion reliance); BC-amendment guidance updated with pre-validator semantics |
| 1.0 | 2026-07-03 | architect | Initial PROPOSED — full PrismQL Utf8→Timestamp migration; supersedes ADR-044 §D4 |
