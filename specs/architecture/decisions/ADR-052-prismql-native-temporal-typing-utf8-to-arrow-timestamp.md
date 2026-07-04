---
document_type: adr
adr_id: "ADR-052"
title: "PrismQL Native Temporal Typing — Datetime Columns and Literals from Arrow Utf8 to Timestamp(Microsecond, UTC)"
status: accepted
date: "2026-07-03"
version: "1.4"
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

ACCEPTED v1.4 (2026-07-04). Human ratification of core decisions recorded 2026-07-03
(D-1520). **§D4 v1.3 ACCEPTED (human-ratified 2026-07-04, Option A + String-column
coercion modification). v1.4 (pre-TDD, remove-uncertainty): `is_date_like` acceptance
set expanded from 2 to 7 format strings (date-only + T-sep/space-sep × full-seconds/
fractional/no-seconds); over-match disposition (unpadded digits, big/signed years)
documented as ACCEPTED BENIGN.**

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
  else → Literal::Utf8(s)                               (unchanged)
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
`'not-a-date'`, `'sensor-id-abc'`); those remain `Literal::Utf8`.

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
| `'2026-6-24'` (unpadded month/day) | `%m`/`%d` accept single digits in chrono | ACCEPTED BENIGN — not RFC-3339 regardless; E-QUERY-041 ("use RFC-3339") is an accurate and helpful message; against String col, coerced to string comparison (correct); against numeric/bool, E-QUERY-001 (correct). |
| `'12345-06-24'` (big year) | `%Y` accepts arbitrary-width year | ACCEPTED BENIGN — same rationale; security sensor data from year 12345 is not a real concern. |
| `'-0044-03-15'` (negative year) | `%Y` accepts signed year | ACCEPTED BENIGN — same rationale. |

**Decision — no year-width guard or regex layer:** A `%4Y` constraint or regex pre-filter
would add implementation complexity with no diagnostic benefit. For every over-matched
input, `check_temporal_literals` produces the correct outcome — E-QUERY-041 for a
Datetime column (the "use RFC-3339" message is accurate), coercion for a String column,
or E-QUERY-001 for a numeric/bool column. The benign-accept path is the production-grade
default.

**Forms that stay `Literal::Utf8` (NOT matched by `is_date_like`)**

| Example | Reason Not Matched |
|---------|-------------------|
| `'not-a-date'` | No format matches |
| `'sensor-id-abc'` | No format matches |
| `'2026-06-24Z'` | Trailing `Z` is not consumed by any NaiveDate/NaiveDateTime format; `parse_from_str` rejects on leftover `Z` |
| `'2026-06-24T12:00:00Z'` | Matched by `parse_from_rfc3339` at Step 1; never reaches `is_date_like`; emitted as `Literal::Timestamp` |
| `'2026-06-24T12:00:00+00:00'` | Same — RFC-3339 form, `Literal::Timestamp` at Step 1 |
| `'abc-12-34'` | Non-numeric year rejected by `%Y`; no format matches |

---

**Step 3 — Plan-time validator: `check_temporal_literals` AST walker in `engine.rs`.**

After the AST is produced and the schema is resolved, `check_temporal_literals` walks
the full `Expr` tree:

| `RawTemporalLiteral` position | Schema check | Result |
|-------------------------------|--------------|--------|
| Comparison (`>`, `<`, `>=`, `<=`, `=`, `!=`) against Datetime/Timestamp column | `column_type == Timestamp(Microsecond, UTC)` | E-QUERY-041 (pedagogical upgrade — the primary purpose of this mechanism) |
| Comparison against String/Utf8 column | `column_type == DataType::Utf8` | COERCE: rewrite node in-place to `Literal::String(s)`; compare as ordinary string literal (SUCCESS — no error emitted, byte-identical to pre-ADR-052 behavior) |
| Comparison against Integer / Float / Bool column | numeric or boolean type | E-QUERY-001 (type mismatch — date-shaped string cannot equal a number or bool) |
| Non-comparison position where surrounding type context resolves to String/Utf8 | `column_type == DataType::Utf8` | COERCE: rewrite to `Literal::String(s)` (SUCCESS) |
| Non-comparison position with no resolvable String context (projection, function arg, etc.) | no schema context or non-String type | E-QUERY-001 (invalid literal) |

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

**Step 4 — Deletion: text-scanner apparatus removed.**

The following functions and code paths are DELETED from `engine.rs`:
- `extract_table_name_from_query_str`
- `extract_column_name_adjacent_to_quoted_value`
- `is_bad_literal_in_datetime_column`
- The parse-fail branch that called the above functions

**Step 5 — Guard: `RawTemporalLiteral` must never reach SQL emission.**

`pipe_sql_emitter.rs` adds a `Literal::RawTemporalLiteral` arm that returns E-QUERY-001
(internal error: unvalidated temporal literal reached emission). Under correct plan
execution, `check_temporal_literals` runs before emission and consumes all
`RawTemporalLiteral` nodes (producing E-QUERY-041 or E-QUERY-001). The emission guard
is a belt-and-suspenders defensive check.

### E-QUERY-001 ↔ E-QUERY-041 boundary (preserved from v1.2, mechanism changed)

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
| `'2026-06-24'` vs Integer / Float / Bool col | `Literal::RawTemporalLiteral` | E-QUERY-001 (type mismatch — date-shaped string cannot equal a number or bool) |
| `'not-a-date'` anywhere | `Literal::Utf8` | No temporal error (other type errors apply) |

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

2. **The emitter change is one line with a verified form.** The `arrow_cast` form
   in `pipe_sql_emitter.rs` is the only emitter change required. The
   `Literal::Timestamp` struct carries both `instant` (chrono) and `iso8601`
   (string) — the emitter uses `iso8601`, pushdown uses `instant`. These paths do
   not interfere. The explicit `arrow_cast` form is preferred over `TIMESTAMP '...'`
   because DataFusion 53.1.0 produces `Timestamp(Nanosecond, None)` for the latter,
   requiring an implicit coercion that `arrow_cast` eliminates entirely.

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
logic error. All other arms must produce E-QUERY-001 (not silently succeed).

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

**E-QUERY-041 detection semantics for BC-2.11.003 / BC-2.11.004 (v1.4 revision):**
Any postcondition or AC describing E-QUERY-041 must specify the **Option A
lenient-parse-then-AST-walk mechanism** with the FULL 7-form acceptance set:

> "The PrismQL parser accepts the following offset-less date/datetime string literals
> as `Literal::RawTemporalLiteral` AST nodes (parse succeeds for all 7 forms):
> date-only (`'2026-06-24'`); T-separator full seconds (`'2026-06-24T12:00:00'`);
> T-separator fractional seconds (`'2026-06-24T12:00:00.123'`); T-separator no seconds
> (`'2026-06-24T12:00'`); space-separator full seconds (`'2026-06-24 12:00:00'`);
> space-separator fractional seconds (`'2026-06-24 12:00:00.500'`); space-separator
> no seconds (`'2026-06-24 12:00'`). The plan-time validator `check_temporal_literals`
> walks the resolved AST with a three-way column-type dispatch: (1) for
> `RawTemporalLiteral` nodes in comparison position against a
> `Timestamp(Microsecond, UTC)` column, E-QUERY-041 is raised; (2) for
> `RawTemporalLiteral` nodes in comparison position against a String/Utf8 column,
> the node is rewritten in-place to `Literal::String(s)` and processing continues
> without error (byte-identical to pre-ADR-052 behavior); (3) for
> `RawTemporalLiteral` nodes against Integer/Float/Bool columns or in non-comparison
> positions without a resolvable String context, E-QUERY-001 is raised."

Do NOT describe E-QUERY-041 as: a parse-time error, a DataFusion cast error, a text-
scanner result, or a raw-query-string scan. These descriptions applied to the retired
v1.2 design.

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
| 9 | `crates/prism-query/src/pipe_sql_emitter.rs:817-818` | [CHANGE] | Update stale comment "Datetime fields is DataType::Utf8" to reflect the new type; add `Literal::RawTemporalLiteral` arm (E-QUERY-001 guard) |
| 10 | `.factory/specs/architecture/decisions/ADR-044-*.md` | [DONE] | `superseded_by` frontmatter added; §Status "PARTIALLY SUPERSEDED by ADR-052 v1.1" block added (2026-07-03) |
| 11 | `.factory/specs/prd-supplements/error-taxonomy.md` | [CHANGE] | Add E-QUERY-041 row; phase = "plan-time AST validator (`check_temporal_literals`)" |
| 12 | `.factory/specs/behavioral-contracts/BC-2.11.021-*.md` | [CHANGE] | Amend postcondition to Option A mechanism; E-QUERY-041 = `check_temporal_literals` walker, NOT parse-fail/text-scanner/DataFusion-cast-intercept |
| 13 | `crates/prism-query/src/ast.rs` | [CHANGE] | Add `Literal::RawTemporalLiteral(String)` variant; doc comment per §D4 |
| 14 | `crates/prism-query/src/sql_parser.rs` (+ `filter_parser.rs` if separate) | [CHANGE] | Modify timestamp literal production: lenient `is_date_like` fallback → `RawTemporalLiteral` instead of parse error |
| 15 | `crates/prism-query/src/engine.rs` | [CHANGE] | DELETE `extract_table_name_from_query_str`, `extract_column_name_adjacent_to_quoted_value`, `is_bad_literal_in_datetime_column`, parse-fail branch; ADD `check_temporal_literals` AST walker |
| 16 | `crates/prism-query/src/tests/` | [CHANGE] | Rewrite E-QUERY-041 RG tests from parse-fail path to `check_temporal_literals` path; add Unicode VP-021 regression test (non-ASCII input → no panic → correct E-QUERY-041/001) |
| 17 | `crates/prism-query/src/` — all `match` on `Literal` | [CHANGE] | TD-VSDD-060 sweep: `grep -r 'Literal::' crates/prism-query/src/` — add `Literal::RawTemporalLiteral` arm to every internal match |
| 18 | `crates/prism-query/src/tests/` — Utf8 datetime assertions | [CHANGE] | Grep for `DataType::Utf8` assertions on Datetime columns; update to `DataType::Timestamp(Microsecond, UTC)` |
| 19 | `crates/prism-sensors/` (normalization paths) | [CHANGE] | Add ISO-8601 string → microseconds-since-epoch parsing at OCSF normalization boundary for Datetime fields |
| 20 | `crates/prism-query/src/` — remaining Utf8 datetime refs | [VERIFY] | `grep -r 'DataType::Utf8' crates/prism-query/src/` to catch any residual hardcoded Utf8 for datetime columns |

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
| Implement `check_temporal_literals` | `engine.rs` | AST walker with three-way column-type dispatch: `RawTemporalLiteral` + Timestamp/Datetime col → E-QUERY-041; `RawTemporalLiteral` + String/Utf8 col → COERCE in-place to `Literal::String(s)` (SUCCESS, no error); `RawTemporalLiteral` + Integer/Float/Bool col → E-QUERY-001; non-comparison position without String context → E-QUERY-001 |
| Add `Literal::RawTemporalLiteral` guard arm | `pipe_sql_emitter.rs` | Must never reach emission — E-QUERY-001 internal error guard |
| TD-VSDD-060 sibling-site sweep on `Literal` | `prism-query/src/*.rs` | Add `RawTemporalLiteral` arm to all internal `match` on `Literal` |

### Red Gate tests that change

| Old RG test | New RG test |
|------------|------------|
| "parse `WHERE ts > '2026-06-24'` → parse fails → E-QUERY-041 via text-scanner" | "parse `WHERE ts > '2026-06-24'` → parse succeeds → `check_temporal_literals` → E-QUERY-041" |
| "parse `WHERE ts > '2026-06-24T12:00:00'` → parse fails → E-QUERY-041" | Same, with `check_temporal_literals` path |
| VP-021 Unicode panic test (if exists) | "non-ASCII query input → no panic → E-QUERY-041 or E-QUERY-001 (no byte-offset crash)" |
| (new) | "`RawTemporalLiteral` vs Integer/Float/Bool column → E-QUERY-001 (not E-QUERY-041)" |
| (new) | "`RawTemporalLiteral` vs String/Utf8 column → COERCE → compare as string literal (SUCCESS — no E-QUERY error emitted; e.g. `WHERE string_col = '2026-06-24'` works)" |
| (new) | "filter-mode `source \| WHERE ts > '2026-06-24'` → E-QUERY-041 (not misclassified)" |
| (new) | "dotted column `payload.ts > '2026-06-24'` → E-QUERY-041 (schema resolves correctly)" |
| (new, v1.4) | "`WHERE ts > '2026-06-24T12:00'` (T-sep, no seconds) vs Datetime col → parse succeeds → E-QUERY-041" |
| (new, v1.4) | "`WHERE ts > '2026-06-24 12:00:00'` (space-sep) vs Datetime col → parse succeeds → E-QUERY-041" |
| (new, v1.4) | "`WHERE ts > '2026-06-24T12:00:00.123'` (fractional) vs Datetime col → parse succeeds → E-QUERY-041" |
| (new, v1.4) | "`WHERE string_col = '2026-06-24 12:00:00'` (space-sep vs String/Utf8 col) → COERCE → compare as string literal (SUCCESS — no E-QUERY error)" |

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
| 1.4 | 2026-07-04 | architect | §D4 `is_date_like` acceptance set expanded (pre-TDD remove-uncertainty): 2 format strings → 7 format strings — adds space-separator forms (`%Y-%m-%d %H:%M:%S`, `%Y-%m-%d %H:%M:%S%.f`, `%Y-%m-%d %H:%M`), T-separator no-seconds form (`%Y-%m-%dT%H:%M`), and T-separator fractional form (`%Y-%m-%dT%H:%M:%S%.f`). Over-match disposition (unpadded digits via `%m`/`%d`, big/signed years via `%Y`) documented as ACCEPTED BENIGN — no regex layer or year-width guard. `### is_date_like Acceptance Set (Canonical)` subsection added with matched/over-matched/rejected tables. E-QUERY-001↔E-QUERY-041 boundary table updated with 5 new example rows (3 new forms vs Datetime col, 1 new form vs String col). RG test table expanded with 4 new tests. BC-2.11.021 amendment text updated to enumerate all 7 accepted forms. D1–D3, D5–D8, all RISK entries, Blast Radius table: unchanged. |
| 1.1 (ratified) | 2026-07-03 | state-manager | Human ratification recorded 2026-07-03 (D-1520). Status: PROPOSED → ACCEPTED. No decision content changes; v1.1 content ratified as authored. |
| 1.3 | 2026-07-04 | architect | §D4 ACCEPTED (human-ratified 2026-07-04, Option A + String-column coercion modification): E-QUERY-041 detection replaced from parse-fail text-scanner to lenient-parse-then-AST-walk. New `Literal::RawTemporalLiteral` AST node; `check_temporal_literals` walker uses three-way column-type dispatch — Timestamp col → E-QUERY-041; String/Utf8 col → COERCE to `Literal::String(s)` (SUCCESS, byte-identical no-op, RISK-5 eliminated); Integer/Float/Bool col → E-QUERY-001. Text-scanner functions deleted. RISK-5 reclassified from LOW accepted to RESOLVED BY DESIGN. Blast radius 20 files; no new sibling-site sweep scope from coercion arm (contained in `check_temporal_literals`). Option B evaluated and rejected. |
| 1.2 | 2026-07-04 | architect | OBS-4 typo fix: `Arc::from("UTF")` → `Arc::from("UTC")` in §D1 canonical construction form (adversary LOCAL cascade catch) |
| 1.1 | 2026-07-03 | architect | remove-uncertainty PASS-1 amendments: D3 emitter changed to arrow_cast (TIMESTAMP '...' → Nanosecond/None in DF 53.1.0); D4 E-QUERY-041 changed from DataFusion cast-failure intercept to Prism-level chrono pre-validator (arrow-cast 58.2.0 lenient — accepts date-only); Arrow construction form corrected Arc::new("UTF".into())→Arc::from("UTC"); RISK-1 downgraded HIGH→MEDIUM (arrow_cast eliminates coercion reliance); BC-amendment guidance updated with pre-validator semantics |
| 1.0 | 2026-07-03 | architect | Initial PROPOSED — full PrismQL Utf8→Timestamp migration; supersedes ADR-044 §D4 |
