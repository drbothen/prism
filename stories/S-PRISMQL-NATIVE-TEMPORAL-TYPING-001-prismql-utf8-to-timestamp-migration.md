---
document_type: story
story_id: S-PRISMQL-NATIVE-TEMPORAL-TYPING-001
title: "PrismQL Native Temporal Typing — migrate ColumnType::Datetime from Arrow Utf8 to Timestamp(Microsecond, UTC) (ADR-052)"
epic_id: EPIC-DEMO
version: "1.2"
status: draft
producer: story-writer
phase: 3
wave: wave-5-e-demo-fidelity
priority: P1
points: 8
tdd_mode: strict
# tdd_mode rationale: this story modifies production Rust code across prism-bin
# (spec_driven_adapter.rs), prism-core (error.rs), prism-query (pipe_sql_emitter.rs,
# tests/high002_plan_pinning_tests.rs), and prism-sensors (OCSF normalization boundary).
# All behavioral changes have corresponding Red Gate tests written as failing todo!() stubs
# BEFORE production code is modified. The grep-sweep ACs (AC-014) and VERIFY-only AC (AC-015,
# ADR-044 pre-completed by architect) have no Red Gate tests but do not justify facade mode —
# production code is modified with new semantic behavior.
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
# BC versions at authoring time (v1.1 story update for ADR-052 v1.1 corrections):
#   BC-2.11.021 v1.2 (active) — amended per ADR-052 v1.1 (remove-uncertainty PASS-1 corrections):
#     arrow_cast form replacing TIMESTAMP '...'; E-QUERY-041 as Prism chrono pre-validator (not DataFusion intercept)
#   BC-2.11.003 v1.6 (draft) — amended per ADR-052 v1.1: same arrow_cast + chrono pre-validator
#   BC-2.11.004 v1.7 (active) — amended per ADR-052 v1.1: same
#   BC-2.11.001 v1.15 (active) — governs the query MCP tool pipeline
# Pre-done spec work (ADR-052-bc-amendment-burst 2026-07-03, including v1.1 corrections):
#   error-taxonomy.md v2.07: E-QUERY-041 row with corrected chrono pre-validator mechanism (done)
#   BC-2.11.021 v1.0→v1.1→v1.2: postcondition + error case amended (done)
#   BC-2.11.003 v1.4→v1.5→v1.6: postcondition + error case + edge cases amended (done)
#   BC-2.11.004 v1.5→v1.6→v1.7: postcondition + error case + edge cases amended (done)
# ADR-044 supersession pre-completed by architect (superseded_by frontmatter + §Status block already present);
# implementer VERIFIES only (AC-015) — no CHANGE to ADR-044 by implementer
verification_properties: [VP-021]
# VP-021 (fuzz, never panics): emitter and parser changes must not introduce panics.
# The existing vp021_parse_fuzz target covers the parser; the emitter change is
# pure string formatting and does not need a new VP.
assumption_validations: []
risk_mitigations: []
# risk_mitigations: ADR-052 RISK-1 (MEDIUM — version-drift silent coercion) is addressed
# by AC-002 (RISK-1 mandatory DataFusion arrow_cast probe test — RG-002).
# ADR-052 RISK-3 (diff_results CF Arrow IPC schema compatibility) is addressed by AC-009.
# ADR-052 RISK-2 (two-representation transition window) is documented in §Known Limitations.
red_gate_tests: 10
estimated_days: "2"
---

# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: PrismQL Native Temporal Typing

Migrate `ColumnType::Datetime` from Arrow `DataType::Utf8` to
`DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))` throughout
the PrismQL query pipeline, per ADR-052 v1.1.

## Narrative

As a PrismQL implementer, I want all sensor `datetime` columns registered in DataFusion
as `Timestamp(Microsecond, Some("UTC"))` (not `Utf8`), the SQL emitter updated to emit
explicit `arrow_cast('...', 'Timestamp(Microsecond, Some("UTC"))')` typed literals,
E-QUERY-041 returned for bare non-RFC-3339 string literals in datetime comparisons via
a Prism plan-time pre-validator (not DataFusion intercept — arrow-cast 58.2.0 is lenient
and would silently coerce date-only and offset-less strings), and outbound sensor API
calls confirmed unchanged (RFC-3339 string via pushdown), so that temporal predicates
(`WHERE timestamp > NOW() - INTERVAL '24h'`) use native typed timestamp comparison
with explicitly-matched types, eliminating the accidental lexicographic-correctness
invariant and unblocking ADR-051 (typed-enrichment datetime fields).

## Background

### Pre-done spec work (do NOT redo in this story)

The following factory spec amendments were completed in the `ADR-052-bc-amendment-burst`
(2026-07-03), including a v1.1 correction pass for remove-uncertainty PASS-1 findings:

| Item | Status |
|------|--------|
| error-taxonomy.md v2.07: E-QUERY-041 as Prism chrono pre-validator (not DataFusion intercept) | DONE |
| BC-2.11.021 v1.2: arrow_cast form; chrono pre-validator description | DONE |
| BC-2.11.003 v1.6: same corrections | DONE |
| BC-2.11.004 v1.7: same corrections | DONE |

The only factory spec change IN SCOPE for this story is:

| Item | Status |
|------|--------|
| ADR-044 frontmatter `superseded_by: "ADR-052 (§D4 only)"` + Status section annotation | IN SCOPE (AC-015) |

### ADR-052 v1.1 Decision Map

| Decision | What the implementer must do |
|----------|------------------------------|
| D1 | Use `DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))` everywhere — canonical Rust construction form (see note below) |
| D2 | Change `column_type_to_arrow` Datetime arm in `spec_driven_adapter.rs`; fix stale doc comment in `column.rs` (actual text: `/// Microsecond-precision UTC timestamp. Arrow: TimestampMicrosecond.`) |
| D3 | Change `Literal::Timestamp` rendering in `pipe_sql_emitter.rs` to `arrow_cast(...)` form — `TIMESTAMP '...'` is NOT correct (DataFusion 53.1.0 produces `Timestamp(Nanosecond, None)` for `TIMESTAMP '...'` literals) |
| D4 | Add `PrismError::TemporalLiteralUnparseable { value_prefix: String }` in `prism-core/src/error.rs`; wire a Prism plan-time pre-validator using `chrono::DateTime::parse_from_rfc3339` strictness that fires BEFORE DataFusion sees the query; wire `-32602` arm in `error_mapping.rs` |
| D5 | Confirm `pushdown.rs` T1 extractor still produces RFC-3339 via `.to_rfc3339()` — no change; add ISO-8601→microseconds parsing in sensor normalization boundary |
| D6 | Investigate `diff_results` CF: confirm no Arrow IPC stored, or add startup clear if needed |
| D7 | Annotate ADR-044 with partial supersession scope |
| D8 | This story ships before any ADR-051 implementation; sequencing enforced by `blocks:` array on the ADR-051 story once registered |

**D1 canonical Rust construction form (compile-verified):**

```rust
DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))
```

The timezone field is `Option<Arc<str>>`. `Arc::from("UTC")` produces `Arc<str>` directly.
`Arc::new("UTC".into())` does NOT compile correctly — `"UTC".into()` infers to `String`,
producing `Arc<String>` which is the wrong type. NEVER use `Some(Arc::new("UTC".into()))` —
it is the wrong type and was corrected in ADR-052 v1.1.

**D3 — `arrow_cast` form required (DataFusion 53.1.0 verified):**

`TIMESTAMP '...'` SQL literal lowers to `Timestamp(Nanosecond, None)` in DataFusion
53.1.0 — the `Z` UTC offset is ignored and precision is nanosecond, not microsecond.
The explicit `arrow_cast` form produces exactly `Timestamp(Microsecond, Some("UTC"))`:

```rust
// Before:
Literal::Timestamp(ts) => format!("'{}'", ts.iso8601),
// After:
Literal::Timestamp(ts) => format!("arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')", ts.iso8601),
```

This is a one-line change. The `\"UTC\"` escapes are required because the Rust string
is delimited with `"..."` and the SQL type-string `'Timestamp(Microsecond, Some("UTC"))'`
contains inner double-quotes.

**D4 — E-QUERY-041 is a Prism plan-time pre-validator (NOT a DataFusion cast intercept):**

`arrow-cast 58.2.0` (verified, remove-uncertainty PASS-1) is LENIENT: it ACCEPTS
date-only (`'2026-06-24'`) and offset-less ISO (`'2026-06-24T12:00:00'`) strings via
silent coercion at plan time. There is NO DataFusion planning error to intercept.
A DataFusion cast-failure intercept cannot serve as the E-QUERY-041 gate.

Prism must validate the string literal using `chrono::DateTime::parse_from_rfc3339`
BEFORE forwarding the query to DataFusion:
- REJECTS date-only (`'2026-06-24'`) → E-QUERY-041
- REJECTS offset-less ISO (`'2026-06-24T12:00:00'`) → E-QUERY-041
- ACCEPTS full RFC-3339 with `Z` (`'2026-06-24T00:00:00Z'`) → passes through
- ACCEPTS full RFC-3339 with numeric offset (`'2026-06-24T00:00:00+00:00'`) → passes through

The Prism plan-time pre-validator fires at query planning time (after E-QUERY-037/038/039,
before DataFusion execution). The error is deterministic and plan-time.

**RISK-1 (MEDIUM — downgraded from HIGH in ADR-052 v1.1):**

The risk is NOT "comparison errors." DataFusion 53.1.0 verified: comparing
`Timestamp(Nanosecond, None)` against `Timestamp(Microsecond, Some("UTC"))` does NOT
error — `temporal_coercion_nonstrict_timezone` inserts a lossless cast. The risk is
**version-drift silent coercion**: if `arrow_cast(...)` form is reverted to `TIMESTAMP '...'`,
the implicit cast works TODAY but may change in a future DataFusion minor version without
any compilation error.

**Mitigation (mandatory probe test RG-002):** Verify that
`arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')` produces
`Timestamp(Microsecond, Some("UTC"))` in the DataFusion plan output. This pins the
behavior to DataFusion 53.1.0 and fails fast if a version upgrade changes `arrow_cast`
semantics.

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) v1.1 | ~9,000 |
| ADR-052 v1.1 (full ADR) | ~5,500 |
| BC-2.11.021 v1.2 | ~3,500 |
| BC-2.11.003 v1.6 | ~3,000 |
| BC-2.11.004 v1.7 | ~2,500 |
| BC-2.11.001 v1.15 | ~6,000 |
| error-taxonomy.md (E-QUERY-041 row v2.07) | ~2,500 |
| `crates/prism-bin/src/spec_driven_adapter.rs` | ~12,000 |
| `crates/prism-core/src/error.rs` | ~5,000 |
| `crates/prism-core/src/column.rs` | ~500 |
| `crates/prism-query/src/pipe_sql_emitter.rs` | ~8,000 |
| `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` | ~6,000 |
| `crates/prism-sensors/src/` (normalization paths) | ~6,000 |
| `crates/prism-mcp/src/error_mapping.rs` | ~4,000 |
| `crates/prism-query/src/pushdown.rs` (verify only) | ~5,000 |
| **Total** | **~78,500** |

Estimated at ~39% of a 200K context window. Within the per-story limit. No split required.

## Tasks

1. **Read** ADR-052 v1.1 in full: `.factory/specs/architecture/decisions/ADR-052-prismql-native-temporal-typing-utf8-to-arrow-timestamp.md` — especially §D1 (correct Arc form), §D3 (arrow_cast emitter form), §D4 (Prism pre-validator), §Blast Radius items 1–15, and §Risk.

2. **Read** BC-2.11.021 v1.2: `.factory/specs/behavioral-contracts/BC-2.11.021-temporal-grammar-now-interval-planning-time-constant-injection.md` — verify the amended postcondition specifying `arrow_cast(...)` form and E-QUERY-041 as Prism chrono pre-validator.

3. **Read** BC-2.11.003 v1.6 and BC-2.11.004 v1.7 — verify the ADR-052 D2 Timestamp typing assertions and E-QUERY-041 chrono pre-validator description.

4. **Read** `crates/prism-core/src/error.rs` — understand the existing `PrismError` enum structure. **Read** `crates/prism-mcp/src/error_mapping.rs` — understand the `map_prism_error` function structure and existing match arms.

5. **Write Red Gate test stubs FIRST** — add the following failing `todo!()` stub tests BEFORE writing any production code:

   a. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_datetime_column_registers_as_timestamp_micros_utc`
      in `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`

   b. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_risk1_datafusion_arrow_cast_probe`
      in `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`

   c. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sql_emitter_yields_arrow_cast_literal`
      in `crates/prism-query/src/` emitter test module

   d. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_sql_mode_date_only_string`
      in `crates/prism-query/src/tests/` (requires `PrismError::TemporalLiteralUnparseable` variant)

   e. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_pipe_mode_date_only_string`
      in `crates/prism-query/src/tests/`

   f. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_map_prism_error_invalid_params`
      in `crates/prism-mcp/src/` (in error_mapping.rs test module or tests/ directory)

   g. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_valid_rfc3339_utc_string_not_rejected`
      in `crates/prism-query/src/tests/`

   h. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sensor_datetime_string_parsed_to_micros`
      in `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`

   i. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_high002_datetime_column_type_is_timestamp`
      in `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`

   j. `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_output_plans_against_timestamp_column`
      in `crates/prism-query/src/tests/` (end-to-end: takes the actual emitter output string and
      plans it through a DataFusion `SessionContext` with a `Timestamp(Microsecond, Some("UTC"))`
      column — closing the transitive gap between RG-002 and RG-003)

   Verify these stubs FAIL before proceeding to step 6.

6. **Add** `PrismError::TemporalLiteralUnparseable { value_prefix: String }` variant to
   `crates/prism-core/src/error.rs`. Verify that `PrismError` already carries `#[non_exhaustive]` —
   if not, add it (CLAUDE.md `#[non_exhaustive]` discipline for pub-API types in prism-core).
   The Display for this variant MUST match (POL-24 byte-for-byte):
   ```
   "E-QUERY-041: The value '{value_prefix}' cannot be interpreted as a UTC timestamp.
   Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only
   and offset-less forms are not accepted. For relative time filters, use
   NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h')."
   ```
   where `{value_prefix}` is the `value_prefix` field (first 50 chars of offending literal,
   truncated at UTF-8 codepoint boundary, per error-taxonomy.md v2.07 §E-QUERY-041 convention).

7. **Add** `map_prism_error` arm for `PrismError::TemporalLiteralUnparseable` in
   `crates/prism-mcp/src/error_mapping.rs`. Must use the symbolic constant `codes::INVALID_PARAMS`
   per repo convention — every existing arm in `error_mapping.rs` uses `codes::` symbolic
   constants, NOT bare integer literals. The function is at line ~24 of `error_mapping.rs`;
   add the explicit match arm returning `codes::INVALID_PARAMS`. Must NOT fall through to
   the catch-all `codes::INTERNAL_ERROR` arm.

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
   Verify the `arrow` / `datafusion::arrow` crate import has `TimeUnit` in scope.
   `Arc::from("UTC")` requires `std::sync::Arc` which is typically already in scope.

9. **Add sensor datetime string parsing** in `crates/prism-bin/src/spec_driven_adapter.rs` or
   `crates/prism-sensors/src/` — at the OCSF normalization boundary where sensor API response
   ISO-8601 datetime strings are converted to Arrow column values. Incoming datetime strings
   must be parsed via `chrono::DateTime::parse_from_rfc3339` → `.timestamp_micros()` → `i64`
   microseconds-since-epoch. This is the in-memory representation for
   `Timestamp(Microsecond, Some("UTC"))` in Arrow.

   Extract the parsing helper as a `pub(super)` function for direct testability (SID-1):
   ```rust
   pub(super) fn parse_datetime_to_micros(s: &str) -> Result<i64, SpecEngineError> {
       chrono::DateTime::parse_from_rfc3339(s)
           .map(|dt| dt.timestamp_micros())
           .map_err(|_| SpecEngineError::NormalizationError { ... })
   }
   ```
   Note: `parse_from_rfc3339` accepts both `Z` and `+00:00` offset forms. This is the
   SAME strictness level as the query-planner pre-validator (D4 invariant: identical strictness).

10. **Fix** `crates/prism-core/src/column.rs` `ColumnType::Datetime` doc comment (blast item 2):
    ```rust
    // Before (actual text — must match exactly):
    /// Microsecond-precision UTC timestamp. Arrow: TimestampMicrosecond.
    // After (correct):
    /// Microsecond-precision UTC timestamp, normalized to UTC at the adapter boundary.
    /// Arrow: Timestamp(Microsecond, UTC-tagged). Stored and transmitted as RFC-3339.
    ```

11. **Change** `crates/prism-query/src/pipe_sql_emitter.rs` `Literal::Timestamp` rendering
    (blast item 3 — the arrow_cast form, NOT `TIMESTAMP '...'`):
    ```rust
    // Before:
    Literal::Timestamp(ts) => format!("'{}'", ts.iso8601),
    // After:
    Literal::Timestamp(ts) => format!("arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')", ts.iso8601),
    ```
    Also update the stale comment at the Datetime/Utf8 reference nearby (blast item 9):
    replace any comment asserting "Datetime fields is DataType::Utf8" with the correct
    "Datetime fields are DataType::Timestamp(Microsecond, Some(\"UTC\")) per ADR-052."

12. **Wire E-QUERY-041 Prism plan-time pre-validator** in `crates/prism-query/src/engine.rs`
    (NOT a DataFusion cast-failure intercept):

    **Host module:** `crates/prism-query/src/engine.rs`. Add a new function
    `check_temporal_literals(ast: &Ast, schema: &PrismSchema) -> Result<(), PrismError>`
    and call it with `check_temporal_literals(&ast, &schema)?;` inserted AFTER
    `check_enrich_udf_availability(...)?;` and BEFORE `build_session_context` / the
    DataFusion planning call. This preserves the gate ordering:

    **Gate ordering** (per error-taxonomy.md E-QUERY-041):
    E-QUERY-001 (`parse` — grammar) →
    E-QUERY-037 (`check_table_availability` — table exists) →
    E-QUERY-038 (`check_query_column_availability` — column exists) →
    E-QUERY-039 (`check_enrich_udf_availability` — infusion registered) →
    `check_temporal_literals` → E-QUERY-041 →
    DataFusion execution (via `build_session_context` / `execute_inner`).

    Note: E-QUERY-037/038/039 are THREE SEPARATE functions (`check_table_availability`,
    `check_query_column_availability`, `check_enrich_udf_availability`), not one combined
    function. Do NOT conflate them.

    `check_temporal_literals` logic: For each string literal in a WHERE/filter predicate
    that is compared against a `Timestamp(Microsecond, Some("UTC"))` column:
    1. Extract the literal string value
    2. Call `chrono::DateTime::parse_from_rfc3339(literal_str)`
    3. If it returns `Err(...)`: construct `PrismError::TemporalLiteralUnparseable`
       with `value_prefix` = first 50 chars of the literal (UTF-8 codepoint boundary safe)
       and return `Err(...)` immediately — DO NOT forward the query to DataFusion

    **Why NOT a DataFusion intercept:** arrow-cast 58.2.0 is LENIENT. `'2026-06-24'`
    (date-only) would be silently coerced to midnight-local in the target timezone —
    producing a wrong comparison result with no error. The only deterministic gate is
    a Prism-level chrono pre-validator that fires before DataFusion sees the query.

    The function must cover all three query modes: SQL (`WHERE timestamp > '...'`),
    pipe (`| where timestamp > '...'`), and filter mode.

13. **Update** `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`:
    Update ALL existing assertions in this file that expect `DataType::Utf8` for datetime
    column schemas to instead assert `DataType::Timestamp(Microsecond, Some(Arc::from("UTC")))`.

14. **VERIFY** `crates/prism-query/src/pushdown.rs` — confirm the T1 extractor uses
    `Literal::Timestamp(ts) => ts.instant.to_rfc3339()` (or equivalent chrono `to_rfc3339` call).
    This path operates on the Prism AST layer, NOT on Arrow DataType — it is unaffected by D2.
    No change required (D5 explicit no-change statement).

15. **VERIFY** `crates/prism-query/src/infusion_udf.rs` — confirm `return_type` returns
    `DataType::Utf8` UNCONDITIONALLY (the function is a stub with no per-output_type mapping —
    the comment reads "Simplified: always returns Utf8 for the current implementation").
    There is NO existing "datetime row" to amend here. Do NOT change this file in this story.
    ADR-051 will INTRODUCE the per-output_type datetime→Timestamp mapping (a new branch
    in `return_type`), not amend an existing row. Add a comment: `// ADR-052: sensor datetime
    columns → Timestamp(Microsecond, Some("UTC")) (ADR-052). ADR-051 (not yet implemented)
    will add a per-output_type branch here to bring enrichment datetime fields to the same type.`

16. **VERIFY** `crates/prism-spec-engine/src/infusion/udf.rs` — confirm `InfusionUdfDescriptor.output_type`
    is a `String`; no Arrow-level change occurs at spec-engine level. No change required.

17. **VERIFY** `specs/infusions/*.infusion.toml` — confirm `output_type = "datetime"` TOML string
    schema is unchanged. No change required.

18. **INVESTIGATE** `diff_results` CF Arrow IPC compatibility (D6 / RISK-3):
    ```bash
    grep -rn "diff_results\|DIFF_RESULTS" crates/ --include="*.rs" \
      | grep -i "ipc\|recordbatch\|arrow"
    ```
    Expected: no matches (diff_results stores TOML/JSON-serialized Rust structs, not Arrow IPC).
    If matches ARE found: investigate and add startup migration step to clear the CF.
    Document the investigation result in the PR description.

19. **SWEEP** remaining files — workspace grep for DataType::Utf8 datetime assertions
    (blast item 15):
    ```bash
    grep -rn 'DataType::Utf8' crates/prism-query/src/ --include="*.rs"
    ```
    For each hit that refers to a datetime column, update to `Timestamp(Microsecond, Some(Arc::from("UTC")))`.

20. **VERIFY** ADR-044 factory spec (blast item 10 — architect pre-completed):
    Confirm `.factory/specs/architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md`
    already contains:
    - Frontmatter `superseded_by:` field referencing `ADR-052`
    - `## Status` section containing `PARTIALLY SUPERSEDED by ADR-052`

    Do NOT edit ADR-044 — the architect has already completed this annotation and may have
    richer content (reason text, scope notes) than the implementer should overwrite.
    The AC-015 verification grep commands below confirm presence without modifying the file.

21. **SAP-1 check** — if any `tracing::*!(event_type = "...")` was added at E-QUERY-041
    detection time, a corresponding row MUST be added to BC-2.16.002 §Postconditions
    Canonical Structured Event Catalog in the SAME commit. If no new tracing emission
    was added at the detection site, no action needed.

22. **Run** `cargo nextest run -p prism-query -p prism-bin -p prism-core -p prism-mcp -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING)' --no-fail-fast` to verify all 10 Red Gate tests pass GREEN.

23. **Run** `just check` once to verify AC-016 (exit 0). Per CLAUDE.md TDD inner loop
    discipline, reserve `just check` for the final gate verification.

## Previous Story Intelligence

This is the first story in the `S-PRISMQL-NATIVE-TEMPORAL-TYPING-*` series.

**Relevant patterns from adjacent merged stories:**

- `S-DEMO-FIDELITY-REMEDIATION-001` (PR #208 — merged develop@122228e8): The three
  separate gate functions in `engine.rs` — `check_table_availability` (E-QUERY-037),
  `check_query_column_availability` (E-QUERY-038), `check_enrich_udf_availability`
  (E-QUERY-039) — are the model for the new `check_temporal_literals` gate (Task 12).
  The pre-validator is inserted after `check_enrich_udf_availability(...)?;` and before
  `build_session_context`, following the same sequential-gate architecture.

- `S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001` (PR #203): The `inject_now` /
  `inject_now_sql_query` / `inject_now_pipe_stage` functions in `prism-query/src/lib.rs`
  perform planning-time `Expr::Now` → `Literal::Timestamp` substitution. These feed
  into the emitter's `Literal::Timestamp` arm. These functions are NOT modified in
  this story — only the emitter rendering changes.

- `S-PERF-GATE-008` (PR #213 — merged): Demonstrates correct `pub(super)` helper
  extraction pattern for testability (SID-1). The `parse_datetime_to_micros` helper
  in Task 9 follows this same extraction pattern.

- **TD-VSDD-060 sibling-site sweep**: When changing `column_type_to_arrow` for `Datetime`,
  grep all callers of `column_type_to_arrow` and all match arms on `ColumnType::Datetime`.

## Architecture Compliance Rules

Derived from ADR-052 v1.1 and `.factory/specs/architecture/` section files:

| Rule | Constraint |
|------|------------|
| **Arrow timestamp type canonical form** | ALWAYS use `DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))`. Never `Timestamp(Nanosecond, ...)`, never `Timestamp(Microsecond, None)` (untagged). NEVER use `Some(Arc::new("UTC".into()))` — wrong type (`Arc<String>` not `Arc<str>`). |
| **Emitter arrow_cast form** | `pipe_sql_emitter.rs` MUST use `arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')` form. `TIMESTAMP '...'` is FORBIDDEN — DataFusion 53.1.0 lowers it to `Timestamp(Nanosecond, None)`. |
| **E-QUERY-041 detection is Prism pre-validator** | E-QUERY-041 is raised by Prism's plan-time pre-validator using `chrono::DateTime::parse_from_rfc3339` BEFORE the query reaches DataFusion. Arrow-cast 58.2.0 is lenient; intercepting DataFusion/Arrow errors is NOT a valid detection mechanism for this error. |
| **Pushdown boundary contract** | `pushdown.rs` T1 extractor (`Literal::Timestamp.instant.to_rfc3339()`) produces RFC-3339 strings for sensor APIs — UNCHANGED. Do NOT modify pushdown.rs. |
| **Identical chrono strictness** | Query-planner pre-validator and sensor-boundary datetime parsing MUST use the same `chrono::DateTime::parse_from_rfc3339` strictness. They MUST NOT diverge in what they accept/reject. |
| **Structured event catalog (SAP-1)** | Any new `tracing::*!(event_type = "...")` at E-QUERY-041 detection time requires a BC-2.16.002 catalog row in the same commit. If no new emission site is added, no action. |
| **Error taxonomy discipline** | E-QUERY-041 Display MUST match error-taxonomy.md v2.07 POL-24 template byte-for-byte. `value_prefix` is 50 chars max, UTF-8 boundary safe. |
| **Forbidden dependencies** | This story does NOT add new crate dependencies to `prism-query` or `prism-core`. `chrono` is already a dependency; `Arc::from` is in `std::sync`. |
| **ADR-051 sequencing gate (D8)** | `infusion_udf.rs` `return_type` returns `DataType::Utf8` UNCONDITIONALLY after this story — it's a stub with no per-output_type mapping. ADR-051 will INTRODUCE the datetime→Timestamp mapping. Do not change `infusion_udf.rs` in this story. |

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `datafusion` | pinned in `[workspace.dependencies]` Cargo.toml | existing workspace pin |
| `arrow` / `arrow-schema` | pinned in workspace | existing workspace pin |
| `chrono` | pinned in workspace | existing dependency — `DateTime::parse_from_rfc3339` + `.timestamp_micros()` |

**Forbidden new dependencies**: No new `[dependencies]` entries in any `Cargo.toml`.
All required types (`TimeUnit`, `DataType`, `Arc`, `chrono::DateTime`) are already
available in the existing dependency graph.

## File Structure Requirements

### Files to CREATE:
None — no new files.

### Files to MODIFY:

| File | Change Type | What Changes |
|------|-------------|--------------|
| `crates/prism-core/src/error.rs` | [CHANGE] | Add `PrismError::TemporalLiteralUnparseable { value_prefix: String }` variant; add `Display` arm; verify `#[non_exhaustive]` present |
| `crates/prism-bin/src/spec_driven_adapter.rs` | [CHANGE] | `column_type_to_arrow`: `Datetime → DataType::Utf8` → `Timestamp(Microsecond, Some(Arc::from("UTC")))`; add `parse_datetime_to_micros` helper + sensor datetime string → microsecond parsing |
| `crates/prism-core/src/column.rs` | [CHANGE] | Fix stale doc comment on `ColumnType::Datetime` (actual current text: `/// Microsecond-precision UTC timestamp. Arrow: TimestampMicrosecond.`) |
| `crates/prism-query/src/pipe_sql_emitter.rs` | [CHANGE] | `Literal::Timestamp` rendering: bare `'...'` → `arrow_cast('...', 'Timestamp(Microsecond, Some("UTC"))')` (blast items 3+9) |
| `crates/prism-query/src/engine.rs` | [CHANGE] | Add `check_temporal_literals` function; call after `check_enrich_udf_availability(...)?;` before `build_session_context`; covers SQL + pipe + filter modes |
| `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` | [CHANGE] | Update all `DataType::Utf8` datetime assertions to `Timestamp(Microsecond, Some(Arc::from("UTC")))`; add RISK-1 arrow_cast probe test (RG-002), Timestamp type assertion test (RG-009), and emitter E2E test (RG-010) |
| `crates/prism-sensors/src/` (normalization paths) | [CHANGE] | Add ISO-8601 string → `i64` microseconds-since-epoch parsing at the OCSF datetime normalization boundary |
| `crates/prism-mcp/src/error_mapping.rs` | [CHANGE] | Add explicit `codes::INVALID_PARAMS` arm for `PrismError::TemporalLiteralUnparseable` |

### Files to VERIFY (no change expected):

| File | What to Verify |
|------|---------------|
| `crates/prism-query/src/pushdown.rs` | `Literal::Timestamp` arm uses `ts.instant.to_rfc3339()` — operates on Prism AST, not Arrow DataType; no change needed |
| `crates/prism-query/src/infusion_udf.rs` | `return_type` returns `DataType::Utf8` UNCONDITIONALLY (stub — no per-output_type datetime branch exists); ADR-051 will INTRODUCE the mapping, not amend an existing row; add comment citing D8 |
| `crates/prism-spec-engine/src/infusion/udf.rs` | `InfusionUdfDescriptor.output_type` is `String` — no Arrow-level change at spec-engine level |
| `specs/infusions/*.infusion.toml` | `output_type = "datetime"` is an opaque string — no TOML schema change |
| `crates/prism-query/src/tests/` (grep) | Remaining `DataType::Utf8` references for datetime columns — any found must be updated |
| `.factory/specs/architecture/decisions/ADR-044-*.md` | Confirm `superseded_by:` frontmatter references ADR-052 AND §Status contains "PARTIALLY SUPERSEDED by ADR-052" — if missing, escalate to architect (do NOT self-edit) |

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

Expected output: no matches (the wrong `Arc<String>` form must not appear).

Traces to: BC-2.11.003 v1.6 §Postconditions (ADR-052 D2); BC-2.11.004 v1.7 §Postconditions (same assertion for pipe mode); ADR-052 v1.1 D1 canonical form `Arc::from("UTC")`.

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
- Verifies the plan's literal type is `Timestamp(Microsecond, Some("UTC"))` — matching the column's type exactly, with no implicit cast node introduced

This test is the MANDATORY RISK-1 mitigation. If a DataFusion version upgrade changes
`arrow_cast` semantics for this type string, this test will fail fast.

Traces to: BC-2.11.021 v1.2 §Postconditions ("DataFusion sees a concrete `WHERE timestamp > arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some(\"UTC\"))')` comparison against a `Timestamp(Microsecond, UTC)` column"); ADR-052 v1.1 §Risk RISK-1 mitigation.

### AC-003 — `pipe_sql_emitter.rs` `Literal::Timestamp` rendering emits `arrow_cast(...)` form (ADR-052 D3 v1.1, blast item 3)

```bash
grep -c "arrow_cast" crates/prism-query/src/pipe_sql_emitter.rs
```

Expected output: at least `1`.

Verify the `TIMESTAMP '...'` form is absent:

```bash
grep -n "TIMESTAMP '{}'" crates/prism-query/src/pipe_sql_emitter.rs
```

Expected output: no matches (the `TIMESTAMP '...'` form must not appear — it produces `Nanosecond/None`).

Verify the arrow_cast type string contains the canonical type:

```bash
grep -n "Timestamp(Microsecond, Some" crates/prism-query/src/pipe_sql_emitter.rs
```

Expected: at least one match in the `Literal::Timestamp` arm.

Traces to: BC-2.11.021 v1.2 §Postconditions ("DataFusion sees a concrete `WHERE timestamp > arrow_cast('2026-06-24T00:00:00Z', 'Timestamp(Microsecond, Some(\"UTC\"))')` comparison"); ADR-052 v1.1 D3 ("arrow_cast form is preferred over `TIMESTAMP '...'` because DataFusion 53.1.0 produces `Timestamp(Nanosecond, None)` for the latter").

### AC-004 — `NOW() - INTERVAL '24h'` lowers to an `arrow_cast(...)` typed literal comparison; `inject_now` path unbroken (ADR-052 D3/D7, BC-2.11.021 v1.2)

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

Traces to: BC-2.11.021 v1.2 §Postconditions ("Planning-time constant injection" bullet);
BC-2.11.021 v1.2 §Invariants ("duration arithmetic is subtraction-only in v1").

### AC-005 — E-QUERY-041 fires at Prism plan time (via `chrono::DateTime::parse_from_rfc3339` pre-validator) for date-only or offset-less string literals in datetime comparisons (ADR-052 D4, BC-2.11.021 v1.2 EC-11-021-009, BC-2.11.003 v1.6 EC-11-003-001, BC-2.11.004 v1.7 EC-11-004-001)

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
  raised by Prism plan-time pre-validator BEFORE DataFusion sees the query
- Pipe mode: `FROM t | where timestamp > '2026-06-24'` → same error
- The error fires via `chrono::DateTime::parse_from_rfc3339` returning `Err(...)`,
  NOT via a DataFusion cast-failure intercept

Verify the `PrismError::TemporalLiteralUnparseable` variant exists:

```bash
grep -c 'TemporalLiteralUnparseable' crates/prism-core/src/error.rs
```

Expected output: at least `2` (the variant definition + the Display arm).

Traces to: BC-2.11.021 v1.2 §Error Cases E-QUERY-041 ("Prism's plan-time literal pre-validator
(`chrono::DateTime::parse_from_rfc3339` strictness) rejects date-only (`'2026-06-24'`) and
offset-less ISO (`'2026-06-24T12:00:00'`) forms before DataFusion execution — arrow-cast 58.2.0
is lenient and would silently coerce these forms"); BC-2.11.003 v1.6 §Error Cases E-QUERY-041;
BC-2.11.004 v1.7 §Error Cases E-QUERY-041.

### AC-006 — `PrismError::TemporalLiteralUnparseable` maps to `codes::INVALID_PARAMS` via `map_prism_error` in `error_mapping.rs` (error-taxonomy v2.07 E-QUERY-041 `map_prism_error` constraint)

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

Expected: at least one match in the `map_prism_error` function body. The arm must use
`codes::INVALID_PARAMS` (the symbolic constant = -32602), consistent with every other arm
in `error_mapping.rs`. Must NOT fall through to the catch-all `codes::INTERNAL_ERROR` arm.

Traces to: error-taxonomy.md v2.07 E-QUERY-041 `map_prism_error` constraint: "must add an
explicit `INVALID_PARAMS` arm — MUST NOT fall through to the catch-all `INTERNAL_ERROR`".

### AC-007 — Valid RFC-3339 UTC string literal `'2026-06-24T00:00:00Z'` in a datetime comparison is NOT rejected by the pre-validator (ADR-052 D4, BC-2.11.003 v1.6 EC-11-003-002, BC-2.11.004 v1.7 EC-11-004-002)

```bash
cargo nextest run -p prism-query \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_valid_rfc3339_utc_string_not_rejected)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

This test verifies that `SELECT * FROM t WHERE timestamp > '2026-06-24T00:00:00Z'` passes
the Prism chrono pre-validator (full RFC-3339 form with `Z` is accepted by
`chrono::DateTime::parse_from_rfc3339`) and does NOT trigger E-QUERY-041.
The query proceeds to DataFusion execution successfully.

Both `Z` and `+00:00` UTC offset forms are accepted by `parse_from_rfc3339`.

Traces to: BC-2.11.003 v1.6 §Edge Cases EC-11-003-002 ("Valid RFC-3339 UTC string literal
in datetime comparison — Valid; `chrono::DateTime::parse_from_rfc3339` accepts this form —
no E-QUERY-041 raised"); BC-2.11.004 v1.7 §Edge Cases EC-11-004-002.

### AC-008 — Pushdown boundary UNCHANGED: `pushdown.rs` T1 extractor produces RFC-3339 strings for sensor APIs (ADR-052 D5 explicit no-change statement)

```bash
grep -n 'to_rfc3339\|rfc_3339' crates/prism-query/src/pushdown.rs | head -5
```

Expected: at least one match showing `ts.instant.to_rfc3339()` or equivalent chrono
RFC-3339 call in the `Literal::Timestamp` extraction arm.

Verify no change was made to pushdown.rs:

```bash
git diff HEAD -- crates/prism-query/src/pushdown.rs
```

Expected: empty diff (no modifications).

Traces to: BC-2.11.021 v1.2 §Postconditions ("ADR-033 push-down benefit: Once lowered to a
`Literal::Timestamp`, the timestamp predicate is automatically recognized by ADR-033's T1
push-down heuristic in `pushdown.rs` — no changes to `pushdown.rs` required").

### AC-009 — `diff_results` CF Arrow IPC compatibility confirmed: no Arrow IPC stored in diff_results, or startup clear mechanism added (ADR-052 D6 / RISK-3)

```bash
grep -rn "diff_results\|DIFF_RESULTS" crates/ --include="*.rs" \
  | grep -i "ipc\|recordbatch\|arrow"
```

Expected output: **no matches** (confirming no Arrow IPC serialization path to diff_results CF).

If matches ARE found: investigate and add startup migration step. Document investigation
result in the PR description (required whether or not Arrow IPC was found).

Traces to: ADR-052 v1.1 §D6; §Risk RISK-3.

### AC-010 — `column.rs` `ColumnType::Datetime` doc comment updated from stale `Arrow: TimestampMicrosecond.` form (ADR-052 D2, blast item 2)

```bash
grep -n 'TimestampMicrosecond' crates/prism-core/src/column.rs
```

Expected output: no matches — the stale `Arrow: TimestampMicrosecond.` text (which currently
reads `/// Microsecond-precision UTC timestamp. Arrow: TimestampMicrosecond.`) is replaced.

```bash
grep -n 'Timestamp(Microsecond, UTC-tagged)' crates/prism-core/src/column.rs
```

Expected: at least one match for the corrected doc comment.

Traces to: ADR-052 v1.1 D2 ("Fix stale doc comment in `column.rs`"); ADR-052 §Rationale 4.

### AC-011 — Stale `pipe_sql_emitter.rs` "Datetime fields is DataType::Utf8" comment updated (ADR-052 blast item 9)

```bash
grep -n 'Datetime.*Utf8\|DataType::Utf8.*datetime\|DataType::Utf8.*Datetime' \
  crates/prism-query/src/pipe_sql_emitter.rs
```

Expected output: no matches (the stale "Utf8" comment near the Datetime rendering site is replaced).

Traces to: ADR-052 v1.1 §Blast Radius item 9.

### AC-012 — `high002_plan_pinning_tests.rs` datetime column assertions updated from `DataType::Utf8` to `DataType::Timestamp(Microsecond, Some("UTC"))` (ADR-052 blast item 4)

```bash
cargo nextest run -p prism-query -E 'test(high002)' --no-fail-fast 2>&1 | tail -5
```

Expected: all `high002_*` tests pass GREEN (zero FAIL).

Verify Utf8 datetime assertions are replaced:

```bash
grep -n 'Utf8' crates/prism-query/src/tests/high002_plan_pinning_tests.rs \
  | grep -iv 'sensor_id\|client_id\|string\|text\|varchar'
```

Expected output: no matches for raw `DataType::Utf8` on datetime-typed fields.

Traces to: ADR-052 v1.1 §Blast Radius item 4; BC-2.11.003 v1.6 §Postconditions.

### AC-013 — Sensor datetime string → `i64` microseconds-since-epoch conversion added at OCSF normalization boundary; same chrono strictness as pre-validator (ADR-052 D5 "Sensor timestamp parsing addition")

```bash
grep -rn 'timestamp_micros\|parse_from_rfc3339' \
  crates/prism-bin/src/spec_driven_adapter.rs crates/prism-sensors/src/ \
  --include="*.rs" 2>/dev/null | head -10
```

Expected: at least one match showing `chrono::DateTime::parse_from_rfc3339` →
`.timestamp_micros()` conversion.

```bash
cargo nextest run \
  -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sensor_datetime_string_parsed_to_micros)' \
  2>&1 | grep -c 'PASS'
```

Expected output: `1`.

Traces to: ADR-052 v1.1 D5 + D4 chrono-strictness invariant ("The same
`chrono::DateTime::parse_from_rfc3339` strictness is applied at the sensor-boundary
datetime parsing path — query-planner validation and sensor-boundary parsing diverge
from each other is forbidden — both use chrono strictness").

### AC-014 — Workspace grep confirms no remaining `DataType::Utf8` hardcoded assertions for datetime columns in `crates/prism-query/src/` (ADR-052 blast item 15)

```bash
grep -rn 'DataType::Utf8' crates/prism-query/src/ --include="*.rs" \
  | grep -iv 'sensor_id\|client_id\|org_slug\|string\|varchar\|text\|severity\|status\|class_uid\|category_uid'
```

Expected output: **no matches** for Utf8 typed to datetime-column positions. Document the grep output in the PR description.

Traces to: ADR-052 v1.1 §Blast Radius item 15.

### AC-015 — ADR-044 already contains partial supersession scope in frontmatter and Status section (ADR-052 D7, blast item 10 — architect pre-completed; VERIFY ONLY)

The architect completed this annotation before TDD dispatch. The implementer MUST verify
it is present but MUST NOT modify ADR-044.

```bash
grep -n 'superseded_by' \
  .factory/specs/architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md
```

Expected: at least one match showing a `superseded_by:` field referencing `ADR-052` in the frontmatter.

```bash
grep -n 'PARTIALLY SUPERSEDED' \
  .factory/specs/architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md
```

Expected: at least one match showing the partial-supersession annotation in the Status section.

If EITHER grep returns no output: escalate to the architect (do NOT add the annotation yourself —
the architect owns ADR spec content per the Agent Routing Table in CLAUDE.md).

Traces to: ADR-052 v1.1 §D7; §Blast Radius item 10.

### AC-016 — `just check` exits 0 with all changes applied (BC-5.39.001 delivery quality gate)

```bash
just check
echo "Exit: $?"
```

Expected output: `Exit: 0`.

Traces to: BC-5.39.001 §Postconditions (delivery quality — workspace gate must be green).

## Red Gate

Ten Red Gate tests. All use `todo!()` stubs before implementation. Pre-implementation state:
tests referencing `PrismError::TemporalLiteralUnparseable` (RG-004, RG-005, RG-006) FAIL TO
COMPILE because the variant does not yet exist. Tests RG-001, RG-002, RG-003, RG-007, RG-008,
RG-009, RG-010 compile but PANIC with `todo!()`.

### RG-001 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_datetime_column_registers_as_timestamp_micros_utc`

**Location:** `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`

**Pre-implementation state:** `todo!()` panic — `column_type_to_arrow(ColumnType::Datetime)`
still returns `DataType::Utf8`.

**Post-implementation state:** asserts `column_type_to_arrow(ColumnType::Datetime)` returns
`DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))` → PASS.

**Why load-bearing:** If the production code returns Utf8, all other timestamp-type behaviors
are incoherent. This test is the foundation gate for the entire migration.

**SID-1 compliance:** `column_type_to_arrow` is a pure function over `ColumnType` — deterministic, cannot be `#[ignore]`'d.

### RG-002 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_risk1_datafusion_arrow_cast_probe`

**Location:** `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`

**Pre-implementation state:** `todo!()` panic.

**Post-implementation state:**
- Creates a DataFusion `SessionContext` with a `Timestamp(Microsecond, Some(Arc::from("UTC")))` column
- Plans `SELECT * FROM t WHERE ts > arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')`
- Verifies the plan is produced without error
- Verifies the literal in the plan is typed `Timestamp(Microsecond, Some("UTC"))` —
  NOT `Timestamp(Nanosecond, None)` — confirming `arrow_cast` produces the correct type
  in DataFusion 53.1.0

**Why load-bearing (RISK-1 mitigation, MEDIUM severity):** The risk is version-drift silent
coercion — if the `arrow_cast` explicit form is reverted to `TIMESTAMP '...'`, the implicit
coercion works in DF 53.1.0 but is not a stable API guarantee. This test pins the behavior
to DataFusion 53.1.0 and will fail fast on a version upgrade that changes `arrow_cast` semantics.

**SID-1 compliance:** Uses DataFusion APIs in-process — no external service, no `#[ignore]` needed.

### RG-003 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sql_emitter_yields_arrow_cast_literal`

**Location:** `crates/prism-query/src/` emitter test module

**Pre-implementation state:** emitter produces `"'2026-07-03T00:00:00Z'"` (bare string);
the test asserting `"arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some(\"UTC\"))')"` FAILS.

**Post-implementation state:** emitter produces `"arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some(\"UTC\"))')"` → PASS.

**Why load-bearing:** The bare-string `'...'` form caused DataFusion to see `Utf8` vs
`Timestamp(Microsecond, UTC)`. The `arrow_cast(...)` form produces `Timestamp(Microsecond, UTC)`
matching the column type exactly. This is the single most critical code change in this story.

### RG-004 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_sql_mode_date_only_string`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `PrismError::TemporalLiteralUnparseable` does not exist →
compile error in any test module that references it.

**Post-implementation state:** SQL query `SELECT * FROM t WHERE timestamp > '2026-06-24'`
returns `Err(PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24".into() })`
raised by the Prism plan-time chrono pre-validator (NOT a DataFusion/Arrow error).

**Why load-bearing:** Without this gate, `arrow-cast 58.2.0` would silently coerce
`'2026-06-24'` to midnight-local in the target timezone — producing a wrong temporal
comparison with no error. The pre-validator provides the only deterministic rejection.

### RG-005 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_pipe_mode_date_only_string`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** same compile error as RG-004.

**Post-implementation state:** pipe query `FROM t | where timestamp > '2026-06-24'` →
`Err(PrismError::TemporalLiteralUnparseable { .. })` from the Prism pre-validator.

**Why load-bearing:** BC-2.11.004 v1.7 EC-11-004-001 specifies E-QUERY-041 in pipe mode
`| where` stages — parity with SQL mode is required.

### RG-006 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_map_prism_error_invalid_params`

**Location:** `crates/prism-mcp/src/error_mapping.rs` test module (or `crates/prism-mcp/tests/`)

**Pre-implementation state:** compile error (variant missing) OR falls through to `-32000` catch-all.

**Post-implementation state:** `map_prism_error(PrismError::TemporalLiteralUnparseable { value_prefix: "2026-06-24".into() })` returns MCP code `-32602 INVALID_PARAMS` (not `-32000`).

**Why load-bearing:** E-QUERY-041 is a caller-resolvable error. Returning `-32000` (internal
error) misleads the MCP caller into thinking the error is server-side.

### RG-007 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_valid_rfc3339_utc_string_not_rejected`

**Location:** `crates/prism-query/src/tests/`

**Pre-implementation state:** `todo!()` panic; after pre-validator wiring, this would
FAIL if the pre-validator is over-aggressive and rejects valid RFC-3339 forms.

**Post-implementation state:** `SELECT * FROM t WHERE timestamp > '2026-06-24T00:00:00Z'`
succeeds — `chrono::DateTime::parse_from_rfc3339` accepts the full RFC-3339 UTC form;
pre-validator passes through; query reaches DataFusion execution.

**Why load-bearing:** E-QUERY-041 must fire ONLY for non-RFC-3339 forms. If it fires for
valid RFC-3339 strings, existing analyst queries would break silently.

**SID-1 compliance:** Uses in-process test execution — no external service, no `#[ignore]`.

### RG-008 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sensor_datetime_string_parsed_to_micros`

**Location:** `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`

**Pre-implementation state:** `todo!()` panic — the `parse_datetime_to_micros` helper
(or equivalent) doesn't exist yet.

**Post-implementation state:** the `parse_datetime_to_micros` helper (extracted as a
`pub(super)` function per SID-1) converts `"2026-07-03T00:00:00Z"` to the correct `i64`
microseconds-since-epoch value, derived at test time via
`chrono::DateTime::parse_from_rfc3339("2026-07-03T00:00:00Z").unwrap().timestamp_micros()`.
Do NOT hardcode a magic constant — derive via chrono in the test body (TD-VSDD-091:
behavioral anchors over magic constants).

**Why load-bearing:** Arrow `Timestamp(Microsecond, Some("UTC"))` columns store `i64`
microseconds since epoch. If incoming sensor datetime strings are stored as-is (Utf8 values)
into a `Timestamp` schema column, Arrow will produce null or panic at materialization time
— a silent data loss pattern.

**SID-1 compliance:** Extract parsing as a `pub(super)` helper — pure function, no external dependencies, deterministic.

### RG-009 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_high002_datetime_column_type_is_timestamp`

**Location:** `crates/prism-query/src/tests/high002_plan_pinning_tests.rs`

**Pre-implementation state:** this test is WRITTEN to assert `DataType::Timestamp(Microsecond, Some(Arc::from("UTC")))` but the actual column type is still `Utf8` at writing time → test FAILS.

**Post-implementation state:** column type is `Timestamp(Microsecond, Some(Arc::from("UTC")))` → PASS.

**Why load-bearing:** `high002_plan_pinning_tests.rs` are the canonical plan-stability tests.
ADR-052 v1.1 explicitly identifies this file as the primary verification gate for D2.

### RG-010 — `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_output_plans_against_timestamp_column`

**Location:** `crates/prism-query/src/tests/` (can be co-located with RG-003 in the emitter test module or in `high002_plan_pinning_tests.rs`)

**Pre-implementation state:** `todo!()` panic — emitter still produces a bare string `'...'`
that fails to plan against a `Timestamp(Microsecond, Some("UTC"))` column without error.

**Post-implementation state:**
- Calls the actual `pipe_sql_emitter.rs` `Literal::Timestamp` render function with a known
  timestamp value (e.g. `Literal::Timestamp(...)`) to obtain the emitted SQL string
- Registers a `Timestamp(Microsecond, Some(Arc::from("UTC")))` column in a DataFusion `SessionContext`
- Plans `SELECT * FROM t WHERE ts_col > {emitter_output_string}` where `{emitter_output_string}`
  is the actual output from RG-003 (the `arrow_cast('...', 'Timestamp(Microsecond, Some("UTC"))')` form)
- Verifies the plan succeeds without error — confirming the emitter output is accepted
  by DataFusion without type-coercion failure

**Why load-bearing:** RG-002 hand-writes the `arrow_cast(...)` query string; RG-003 tests the
emitter's string output in isolation. RG-010 closes the transitive coverage gap by taking the
ACTUAL emitter output (not a hand-written form) and proving it plans correctly against a
`Timestamp(Microsecond, Some("UTC"))` column. If the emitter format string has a quoting or
escaping mistake invisible in a string comparison, RG-010 will catch it at DataFusion plan time.

**SID-1 compliance:** Uses in-process DataFusion + emitter — no external service, no `#[ignore]`.

## Behavioral Contracts

| BC | Title | Version | Role in this story |
|----|-------|---------|-------------------|
| BC-2.11.021 | Temporal Grammar — `NOW()` and `INTERVAL` Planning-Time Constant Injection | v1.2 | Amended §Postconditions: emitter uses `arrow_cast(...)` form; E-QUERY-041 as Prism chrono pre-validator. AC-003, AC-004, AC-005 trace here. |
| BC-2.11.003 | PrismQL SQL Mode Parsing | v1.6 | Amended: ADR-052 D2 assertion — `Timestamp(Microsecond, Some("UTC"))`; chrono pre-validator pre-validates string literals at plan time. AC-001, AC-005, AC-007 trace here. |
| BC-2.11.004 | PrismQL Pipe Mode Parsing | v1.7 | Same D2 assertion for pipe `| where` stages; chrono pre-validator in pipe mode. AC-001, AC-005, AC-007 trace here. |
| BC-2.11.001 | `query` MCP Tool Accepts Scoping + PrismQL Query String | v1.15 | Governs the query pipeline; E-QUERY-041 gate ordering; `map_prism_error` -32602 constraint. AC-006 traces here. |

## Subsystem Anchor Justifications

Per `architecture/ARCH-INDEX.md` Subsystem Registry:
- **SS-09** (Sensor Adapters): owns `crates/prism-bin/src/spec_driven_adapter.rs` (column type registration + sensor datetime parsing). SS-09 owns all sensor adapter code.
- **SS-10** (OCSF Normalization): owns the ISO-8601 → Timestamp conversion at the data ingest boundary (AC-013). SS-10 owns all OCSF normalization code.
- **SS-11** (Query Engine): owns `crates/prism-query/src/pipe_sql_emitter.rs` (emitter), `pushdown.rs` (pushdown), `high002_plan_pinning_tests.rs` (plan stability), the temporal pre-validator wiring, and the DataFusion integration layer. SS-11 owns all PrismQL query processing code.

## Architecture Mapping

| Component | Module | Pure/Effectful | Change |
|-----------|--------|---------------|--------|
| `column_type_to_arrow` function | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (ColumnType → DataType mapping) | D2 CHANGE: Datetime arm → Timestamp |
| `parse_datetime_to_micros` helper | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (string → i64, may error) | D5 ADD: ISO-8601 → microseconds |
| `check_temporal_literals` function | `crates/prism-query/src/engine.rs` | Effectful (calls chrono, returns Err) | D4 ADD: chrono pre-validator; inserted after `check_enrich_udf_availability` |
| `PrismError::TemporalLiteralUnparseable` | `crates/prism-core/src/error.rs` | Pure (error type definition) | D4 ADD: new variant |
| `map_prism_error` arm | `crates/prism-mcp/src/error_mapping.rs` | Pure (error code mapping) | D4 ADD: -32602 arm |
| `pipe_sql_emitter.rs` `Literal::Timestamp` | `crates/prism-query/src/pipe_sql_emitter.rs` | Pure (AST → SQL string) | D3 CHANGE: `arrow_cast(...)` form |
| `pushdown.rs` T1 extractor | `crates/prism-query/src/pushdown.rs` | Pure (AST → string extract) | D5 VERIFY: no change |
| `infusion_udf.rs` `return_type` | `crates/prism-query/src/infusion_udf.rs` | Pure (always returns Utf8, stub) | D8 VERIFY: unconditional stub; ADR-051 introduces per-type mapping |
| `column.rs` doc comment | `crates/prism-core/src/column.rs` | Pure (enum definition) | D2 CHANGE: doc comment only |
| ADR-044 frontmatter | `.factory/specs/architecture/decisions/ADR-044-*.md` | N/A (factory spec) | D7 VERIFY: superseded_by already present (architect pre-completed) |

## Edge Cases

| ID | Description | Expected Behavior | BC Anchor |
|----|-------------|-------------------|-----------|
| EC-001 | `WHERE timestamp > '2026-06-24'` (date-only, no time or UTC offset) | `E-QUERY-041` from Prism chrono pre-validator | BC-2.11.003 v1.6 EC-11-003-001, BC-2.11.004 v1.7 EC-11-004-001, BC-2.11.021 v1.2 EC-11-021-009 |
| EC-002 | `WHERE timestamp > '2026-06-24T00:00:00Z'` (valid RFC-3339 with `Z`) | Passes pre-validator; succeeds | BC-2.11.003 v1.6 EC-11-003-002, BC-2.11.004 v1.7 EC-11-004-002 |
| EC-003 | `WHERE timestamp > '2026-06-24T00:00:00+00:00'` (valid RFC-3339 with `+00:00`) | Passes pre-validator (`parse_from_rfc3339` accepts `+00:00`); succeeds | BC-2.11.021 v1.2 (accepted forms) |
| EC-004 | `WHERE timestamp > NOW() - INTERVAL '24h'` (normal temporal predicate) | Injects `arrow_cast(...)` typed literal → Timestamp-vs-Timestamp comparison | BC-2.11.021 v1.2 EC-11-021-001 |
| EC-005 | `WHERE timestamp > 'yesterday'` (unparseable free-text) | `E-QUERY-041` with `value_prefix = "yesterday"` | error-taxonomy v2.07 E-QUERY-041 "Invalid forms" |
| EC-006 | `WHERE timestamp > '2026-06-24T12:00:00'` (missing UTC offset) | `E-QUERY-041` — `parse_from_rfc3339` rejects offset-less ISO | error-taxonomy v2.07 E-QUERY-041 "Invalid forms" |
| EC-007 | Sensor API returns ISO-8601 datetime string with `+00:00` offset | Must parse correctly to microseconds — `parse_from_rfc3339` handles both `Z` and `+00:00` | ADR-052 v1.1 D5 (identical chrono strictness) |
| EC-008 | `diff_results` CF contains old Utf8-typed Arrow IPC bytes after upgrade | Startup migration step clears CF; no schema mismatch crash | ADR-052 v1.1 D6 / RISK-3 mitigation |
| EC-009 | `E-QUERY-041` `value_prefix` contains exactly 50 chars from a 100-char offending literal | `value_prefix` truncated at UTF-8 codepoint boundary ≤ 50 chars | error-taxonomy v2.07 E-QUERY-041 (AD-017 / E-INFUSE-014 truncation convention) |

## Known Limitations

**RISK-2: Two-representation transition window (ADR-052 v1.1 §Risk, MEDIUM)**

Between this story merging and the ADR-051 typed-enrichment story shipping, enrichment
`output_type = "datetime"` fields remain mapped to `DataType::Utf8` in `infusion_udf.rs`
(the function is a stub that always returns Utf8 — no per-output_type mapping exists yet).
This is inconsistent but not a regression — it preserves pre-existing behavior for enrichment
fields. ADR-051 will INTRODUCE the per-output_type datetime→Timestamp mapping, not amend an
existing row.

## Estimated Complexity

8 story points. Rationale: 7 source files modified + 1 factory spec VERIFY (ADR-044, pre-done) +
10 Red Gate tests + DataFusion RISK-1 arrow_cast probe + emitter E2E integration test (RG-010) +
sensor normalization addition + E-QUERY-041 Prism plan-time pre-validator in engine.rs +
`diff_results` CF investigation + `map_prism_error` arm addition in `error_mapping.rs`. No new crates.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | ADR-052-story-decomposition-v1.2 | 2026-07-03 | story-writer | **Applied remove-uncertainty PASS-2 corrections (coordinator-relayed).** AC-015/Task 20 re-scoped CHANGE→VERIFY (architect pre-completed ADR-044 supersession; implementer must not overwrite). AC-006/Task 7: symbolic constant `codes::INVALID_PARAMS` replaces literal `-32602` (repo convention). Task 12: pinned pre-validator host to `crates/prism-query/src/engine.rs`; named function `check_temporal_literals`; corrected gate attribution (E-QUERY-037=`check_table_availability`, E-QUERY-038=`check_query_column_availability`, E-QUERY-039=`check_enrich_udf_availability`; insertion point after `check_enrich_udf_availability(...)?;`). RG-010 added (emitter E2E test — takes actual emitter output, plans through DataFusion SessionContext against Timestamp(µs, UTC) column; closes RG-002/RG-003 transitive gap). red_gate_tests 9→10. File Structure, Architecture Mapping, Estimated Complexity, Task 5j, Task 22 all updated. |
| 1.1 | ADR-052-story-decomposition-v1.1 | 2026-07-03 | story-writer | **Applied remove-uncertainty PASS-1 corrections (coordinator-relayed, ADR-052 v1.1 + BC corrections).** C1 (RG-008 wrong year): fixed constant to chrono derivation form (TD-VSDD-091). H3 (wrong Arc construction): replaced ALL `Some(Arc::new("UTC".into()))` with `Some(Arc::from("UTC"))` throughout. Detection architecture reshape: E-QUERY-041 is a Prism plan-time chrono pre-validator (NOT DataFusion cast intercept — arrow-cast 58.2.0 is lenient); rewrote Task 12, AC-005/006/007, RG-004/005/006/007 + edge cases accordingly. Emitter form: `TIMESTAMP '...'` → `arrow_cast('...', 'Timestamp(Microsecond, Some(\"UTC\"))')` in Task 11, AC-003, RG-003. RISK-1: downgraded HIGH→MEDIUM; probe tests arrow_cast correctness (not TIMESTAMP literal mismatch). M3: AC-010 "before" text corrected to actual column.rs text (`/// Microsecond-precision UTC timestamp. Arrow: TimestampMicrosecond.`). M4: infusion_udf.rs VERIFY description corrected (unconditional stub, no datetime row; ADR-051 introduces mapping). M5: map_prism_error location fixed to `crates/prism-mcp/src/error_mapping.rs`. BC versions updated: BC-2.11.021 v1.1→v1.2, BC-2.11.003 v1.5→v1.6, BC-2.11.004 v1.6→v1.7. ADR-052 references updated to v1.1. |
| 1.0 | ADR-052-story-decomposition | 2026-07-03 | story-writer | Initial story — decomposed from ADR-052 v1.0 + amended BCs (BC-2.11.021 v1.1, BC-2.11.003 v1.5, BC-2.11.004 v1.6, BC-2.11.001 v1.15) + error-taxonomy.md E-QUERY-041. 16 ACs, 9 Red Gate tests. ACTIVE-NEXT on demo roadmap; blocks ADR-051 typed-enrichment (D8 sequencing). |
