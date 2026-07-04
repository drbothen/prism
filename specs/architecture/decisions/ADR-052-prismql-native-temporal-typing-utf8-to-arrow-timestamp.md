---
document_type: adr
adr_id: "ADR-052"
title: "PrismQL Native Temporal Typing — Datetime Columns and Literals from Arrow Utf8 to Timestamp(Microsecond, UTC)"
status: accepted
date: "2026-07-03"
version: "1.1"
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

ACCEPTED v1.1 (2026-07-03). Human ratification recorded 2026-07-03 (D-1520).
Ready for product-owner BC amendments and story decomposition.

v1.1 amends v1.0: D3 emitter form changed from `TIMESTAMP '...'` to explicit
`arrow_cast(...)` (DataFusion 53.1.0 verified: `TIMESTAMP '...'` produces
`Timestamp(Nanosecond, None)`, not Microsecond/UTC); D4 E-QUERY-041 detection
mechanism changed from DataFusion-cast-failure intercept to Prism-level literal
pre-validator using chrono RFC-3339 strictness (arrow-cast 58.2.0 is lenient —
accepts date-only and offset-less strings — so cast-failure cannot be the gate);
Arrow construction form corrected from `Some(Arc::new("UTC".into()))` (compiles to
`Arc<String>`) to `Some(Arc::from("UTC"))` (correct `Arc<str>`); RISK-1 narrative
corrected.

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
`String`, producing `Arc<String>` which is the wrong type. Use `Arc::from("UTF")` or,
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

**Pushdown path (`pushdown.rs`) is UNCHANGED.** The pre-fan-out ADR-033 T1 extractor
at `pushdown.rs:450` already operates on `Literal::Timestamp.instant` (a
`chrono::DateTime<Utc>`) and calls `.to_rfc3339()` to produce the API string:
```rust
Expr::Literal(Literal::Timestamp(ts)) => ts.instant.to_rfc3339()
```
This path operates on the Prism AST layer, not on Arrow DataType. Sensor API calls
receive the same RFC-3339 UTC string before and after this migration — **zero
behavioral change at the adapter boundary** (see D5).

**D4 — E-QUERY-041: Prism-level temporal literal pre-validator (NOT a DataFusion cast intercept).**

**Background — why DataFusion cast-failure cannot be the detection mechanism:**
arrow-cast 58.2.0 (verified via remove-uncertainty PASS-1, 2026-07-03) implements
Utf8→Timestamp casting with LENIENT semantics. It ACCEPTS:
- Date-only form: `'2026-06-24'` → midnight of that date in the target timezone
- Offset-less ISO: `'2026-06-24T12:00:00'` → treated as target timezone (not UTC)
- Full RFC-3339 with `Z`: `'2026-06-24T00:00:00Z'` → accepted
- Full RFC-3339 with numeric offset: `'2026-06-24T00:00:00+00:00'` → accepted

Genuinely malformed strings (e.g., `'not-a-date'`) only error at EXECUTION time as
`ArrowError::CastError` — not at planning time. There is no planning-time DataFusion
cast error to intercept. A date-only or offset-less comparison against a sensor
`Timestamp(Microsecond, UTC)` column would silently produce wrong comparisons
(comparing against midnight-local rather than UTC). This is the gap that Prism's
pre-validator must close.

**E-QUERY-041 — `TemporalLiteralUnparseable` — Prism pre-validator at parse/plan time:**

When the query planner encounters a string literal compared against a datetime column
(`Timestamp(Microsecond, UTC)`), Prism validates the literal using
`chrono::DateTime::parse_from_rfc3339` BEFORE the SQL is forwarded to DataFusion.

`chrono::DateTime::parse_from_rfc3339` applies STRICT RFC-3339 semantics:
- REJECTS date-only (`'2026-06-24'`) — E-QUERY-041 raised
- REJECTS offset-less ISO (`'2026-06-24T12:00:00'`) — E-QUERY-041 raised
- ACCEPTS full RFC-3339 with `Z` (`'2026-06-24T00:00:00Z'`) — passes
- ACCEPTS full RFC-3339 with numeric offset (`'2026-06-24T00:00:00+00:00'`) — passes

Prism raises E-QUERY-041 with the offending value BEFORE DataFusion sees the query.
The error is deterministic and plan-time (not execution-time). This eliminates the
silent-wrong-answer risk from arrow-cast's lenient coercion.

**Chrono strictness as the single source of truth (AC-013):** The same
`chrono::DateTime::parse_from_rfc3339` validation is applied at the sensor-boundary
datetime parsing path (the incoming ISO-8601 string → `i64` microseconds-since-epoch
conversion in `spec_driven_adapter.rs`). Query-planner validation and sensor-boundary
parsing diverge from each other is forbidden — both use chrono strictness.

**E-QUERY-041 message format:**
```
E-QUERY-041: The value '{first_50_chars}' cannot be interpreted as a UTC timestamp.
Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only
and offset-less forms are not accepted. For relative time filters, use
NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h').
```

E-QUERY-041 is the next available code in the `E-QUERY-NNN` namespace. The error is a
query-level user error (not an internal error). The `{first_50_chars}` truncation follows
E-INFUSE-014 convention — credential values MUST NOT appear in query literals; truncation
prevents inadvertent secret exposure.

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

**E-QUERY-041 detection semantics for BC-2.11.003 / BC-2.11.004:** Any postcondition
or AC describing E-QUERY-041 must specify it as a **Prism plan-time literal
pre-validation** (chrono `DateTime::parse_from_rfc3339` strictness, raised before
DataFusion sees the query), NOT as an intercepted DataFusion or Arrow cast error.
The distinction is: arrow-cast 58.2.0 accepts date-only and offset-less forms
leniently — only full chrono RFC-3339 strictness provides the correct gate.

### Error Taxonomy (`prd-supplements/error-taxonomy.md`)

Add E-QUERY-041:
```
E-QUERY-041 | TemporalLiteralUnparseable | Query | Plan-time pre-validation |
The value '{first_50_chars}' cannot be interpreted as a UTC timestamp.
Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z').
Date-only and offset-less forms are not accepted. For relative time filters,
use NOW() - INTERVAL 'Nh'.
```

**Error taxonomy classification note:** E-QUERY-041 emitter/phase = Prism
plan-time literal pre-validation (deterministic, raised by `chrono::DateTime::
parse_from_rfc3339`). It is NOT classified as a DataFusion execution error or an
Arrow `ArrowError::CastError` — those only surface at execution time and are too
lenient to serve as the gate (arrow-cast 58.2.0 verified).

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
| 9 | `crates/prism-query/src/pipe_sql_emitter.rs:817-818` | [CHANGE] | Update stale comment "Datetime fields is DataType::Utf8" to reflect the new type |
| 10 | `.factory/specs/architecture/decisions/ADR-044-*.md` | [CHANGE] | Add `superseded_by: "ADR-052 (§D4 only)"` to frontmatter; add "PARTIALLY SUPERSEDED by ADR-052 (§D4)" to Status section |
| 11 | `.factory/specs/prd-supplements/error-taxonomy.md` | [CHANGE] | Add E-QUERY-041 row |
| 12 | `.factory/specs/behavioral-contracts/BC-2.11.021-*.md` | [CHANGE] | Amend postcondition; E-QUERY-041 description must specify Prism plan-time literal pre-validator (chrono strictness), not DataFusion/Arrow cast intercept |
| 13 | `crates/prism-query/src/tests/` | [VERIFY] | Grep all test files for `DataType::Utf8` assertions on fields identified as Datetime columns; update each |
| 14 | `crates/prism-sensors/` (normalization paths) | [CHANGE] | Add ISO-8601 string → microseconds-since-epoch parsing at the OCSF normalization boundary for Datetime fields |
| 15 | `crates/prism-query/src/` — remaining files | [VERIFY] | `grep -r 'Utf8\|DataType::Utf8' crates/prism-query/src/` to catch any hardcoded Utf8 assertions for datetime columns |

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
Rejected: Microsecond is the established codebase baseline; overflow is irrelevant
for security sensor use cases; RISK-1 is mitigated by a probe test in the
implementation story.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 (ratified) | 2026-07-03 | state-manager | Human ratification recorded 2026-07-03 (D-1520). Status: PROPOSED → ACCEPTED. No decision content changes; v1.1 content ratified as authored. |
| 1.1 | 2026-07-03 | architect | remove-uncertainty PASS-1 amendments: D3 emitter changed to arrow_cast (TIMESTAMP '...' → Nanosecond/None in DF 53.1.0); D4 E-QUERY-041 changed from DataFusion cast-failure intercept to Prism-level chrono pre-validator (arrow-cast 58.2.0 lenient — accepts date-only); Arrow construction form corrected Arc::new("UTF".into())→Arc::from("UTC"); RISK-1 downgraded HIGH→MEDIUM (arrow_cast eliminates coercion reliance); BC-amendment guidance updated with pre-validator semantics |
| 1.0 | 2026-07-03 | architect | Initial PROPOSED — full PrismQL Utf8→Timestamp migration; supersedes ADR-044 §D4 |
