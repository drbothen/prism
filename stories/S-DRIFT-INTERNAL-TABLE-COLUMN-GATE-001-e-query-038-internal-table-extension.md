---
document_type: story
story_id: "S-DRIFT-INTERNAL-TABLE-COLUMN-GATE-001"
title: "Extend E-QUERY-038 plan-time column gate to internal (prism_*) tables"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "0.2"
spec_version: "v0.2"
level: ops
producer: story-writer
timestamp: "2026-07-11"
modified: "2026-07-11"
input-hash: ""
inputs:
  - crates/prism-query/src/engine.rs
  - crates/prism-query/src/materialization.rs
  - .factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md
  - .factory/specs/behavioral-contracts/BC-2.11.012-virtual-fields.md
origin_finding: "DRIFT-INTERNAL-TABLE-COLUMN-GATE-001"
origin_cascade: "DEFECT-CSDEVICES-EMPTY-PIPELINE-001 LOCAL pass-21 F-CSD-P21-OBS-003; D-1670 (2026-07-10)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-11]
crates_touched:
  - prism-query
  - prism-core
target_module: "crates/prism-query/src/engine.rs"
behavioral_contracts:
  - BC-2.11.016
  - BC-2.11.012
# BC status: BC-2.11.016 v1.27 and BC-2.11.012 v1.11 are both active.
# BC-2.11.016 v1.27 §Design Constraints documents the internal-table gate extension:
# the unconditional prism_* skip is removed; gate now uses INTERNAL_TABLE_SPECS
# schema-aware checking. OQ-001 RESOLVED.
# BC-2.11.012 v1.11 records DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 as a UX gap in EC-11-035;
# this story resolves that gap.
# AC↔BC bidirectional traces required before status=ready (S-7.01).
verification_properties: []
depends_on: []
blocks: []
points: 8
estimated_days: 2.0
risk: MEDIUM
acceptance_criteria_count: 4
red_gate_tests: 5
estimated_passes: "3-4"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-DRIFT-INTERNAL-TABLE-COLUMN-GATE-001: Extend E-QUERY-038 plan-time column gate to internal (prism_*) tables

## §Origin — [drift] DRIFT-INTERNAL-TABLE-COLUMN-GATE-001

**Cascade:** DEFECT-CSDEVICES-EMPTY-PIPELINE-001 LOCAL pass-21 F-CSD-P21-OBS-003; D-1670 (2026-07-10)
**Unblocked by:** PR #221 develop@5f1b5771 (2026-07-11)

During CSDEVICES pass-21, adjudication of the `_source_type`-on-internal-table behavior
produced BC-2.11.012 v1.9 (Option b FENCE/EXCLUDE: `_source_type` is absent from internal
table schemas). That adjudication surfaced a UX gap recorded as
DRIFT-INTERNAL-TABLE-COLUMN-GATE-001: when a user writes
`SELECT nonexistent_col FROM prism_alerts`, the current code path returns an opaque
`PrismError::QueryExecutionFailed` (DataFusion field-not-found at runtime) rather than
the pedagogical `E-QUERY-038` with `available_columns` and `did_you_mean`. This is
because `check_query_column_availability` in `crates/prism-query/src/engine.rs` has an
explicit early-return for all `prism_*` table names:

```rust
// Skip prism_* internal tables (they have a separate capability gate).
if table_name.starts_with("prism_") {
    return Ok(());
}
```

Internal tables (`prism_rules`, `prism_alerts`, `prism_threats`, `prism_enrichments`,
`prism_raw`, `prism_cache`, `prism_meta`) have known, static column schemas defined in
`prism-core`'s `InternalTableDescriptor::full_schema()` and `INTERNAL_TABLE_SPECS`.
This story replaces the unconditional skip with a schema-aware gate that produces
E-QUERY-038 with the correct `available_columns` for internal tables — the same
pedagogical response analysts already receive for sensor tables.

The BC amendment (OQ-001) has been resolved — BC-2.11.016 v1.27 documents the internal-table gate extension. This story is unblocked pending S-7.01 AC↔BC trace check.

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.11.016 | E-QUERY-038 Column-Not-Found Plan-Time Gate (L4) | v1.27 | This story extends the gate scope to `prism_*` tables by replacing the unconditional `return Ok(())` skip with a schema-aware check. BC-2.11.016 v1.27 §Design Constraints documents the extended scope (OQ-001 resolved). |
| BC-2.11.012 | Virtual Fields in Queries — `_sensor`, `_client`, `_source_table`, `_source_type` | v1.11 | EC-11-035 in BC-2.11.012 v1.9 documents DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 as a UX gap. This story resolves EC-11-035. The existing EC-11-035 test vector (`SELECT _source_type FROM prism_alerts → QueryExecutionFailed`) remains correct: `_source_type` is absent from internal table schemas, so querying it produces E-QUERY-038 (with available_columns that do NOT include `_source_type`), not DataFusion runtime failure. |

## Acceptance Criteria

### AC-001 — `check_query_column_availability` produces E-QUERY-038 for a nonexistent column on an internal table
(traces to BC-2.11.016 v1.27 postcondition: "E-QUERY-038 fires at plan time for column references that are not in the available column set of the queried table")

When a PrismQL query targets a `prism_*` table and references a column name not present
in that table's `InternalTableDescriptor::full_schema()` column list, the plan-time gate
`check_query_column_availability` in `crates/prism-query/src/engine.rs` returns
`Err(PrismError::ColumnNotFound(Box<ColumnNotFoundDetails>))` with:
- `column`: the unrecognized column name
- `available_columns`: the actual column names from the internal table's static schema
- `did_you_mean`: a Levenshtein suggestion if a close match exists in the schema

Concrete example: `SELECT nonexistent_col FROM prism_alerts` fires E-QUERY-038 with
`available_columns` listing the real `prism_alerts` column set.

A Red Gate test `test_BC_2_11_016_internal_table_nonexistent_column_produces_E_QUERY_038`
constructs a query targeting a `prism_*` table with a fabricated column name and asserts
`PrismError::ColumnNotFound` is returned before any DataFusion plan execution.

### AC-002 — `SELECT _source_type FROM prism_*` continues to produce E-QUERY-038 (not QueryExecutionFailed)
(traces to BC-2.11.012 v1.11 EC-11-035 — existing test vector preserved: "`SELECT _source_type FROM prism_alerts → QueryExecutionFailed`" is corrected to E-QUERY-038 per this story's delivery)

**Note:** BC-2.11.012 v1.9 changelog recorded EC-11-035 test vector as
"`SELECT _source_type FROM prism_alerts` → `E-QUERY-038`" — see the v1.9 changelog row
directive: "Add unit test `test_BC_2_11_012_EC_11_035_source_type_on_internal_table_returns_E_QUERY_038`
— assert `DataFusionError::Plan` with message containing 'No field named _source_type'."
That directive was authored under the assumption that `_source_type` would fire E-QUERY-038
via the capability gate path. With this story extending the column gate to internal tables,
the test now asserts `PrismError::ColumnNotFound` (plan-time E-QUERY-038 via
`check_query_column_availability`), which is more pedagogical: `available_columns` lists
the real schema, `did_you_mean` suggests the closest match. The test from the v1.9
directive is superseded by the one in AC-001 (parameterized to cover `_source_type` as the
"nonexistent" column on `prism_alerts`).

A Red Gate test `test_BC_2_11_012_EC_11_035_source_type_on_internal_table_produces_E_QUERY_038_with_available_columns`
asserts that `SELECT _source_type FROM prism_alerts` returns `PrismError::ColumnNotFound`
with `available_columns` NOT containing `_source_type`.

### AC-003 — Internal table column gate does NOT fire on valid column references
(traces to BC-2.11.016 v1.27 postcondition: "E-QUERY-038 fires only for columns not in the available set — FP-001 invariant: no false positives for columns that exist in the schema")

When a PrismQL query targets a `prism_*` table and references a column that exists in
`InternalTableDescriptor::full_schema()` for that table, `check_query_column_availability`
returns `Ok(())` — no false E-QUERY-038. This preserves the FP-001 invariant (no
false positives on actually-available columns).

A Red Gate test `test_BC_2_11_016_internal_table_valid_column_does_not_produce_E_QUERY_038`
asserts that a query selecting a known-valid column from `prism_alerts` (or another
internal table) returns `Ok(())` from the gate.

### AC-004 — Capability-gated internal table columns excluded from internal-column gate when capability absent
(traces to BC-2.11.016 v1.27 §Design Constraints: "E-QUERY-038 is exclusively plan-time; the capability gate is a separate orthogonal check")

The internal-table column gate does NOT perform capability checking (that is the
responsibility of `check_internal_table_capabilities` in `engine.rs`, which runs as a
separate gate). If the user lacks the capability for a table like `prism_audit`, the
capability gate fires E-QUERY-039 (or equivalent) before the column gate runs. The
column gate for internal tables operates only on the column set; it does not duplicate
the capability gate.

A Red Gate test `test_BC_2_11_016_internal_table_column_gate_does_not_duplicate_capability_check`
asserts that a query like `SELECT existing_audit_col FROM prism_audit` — where the column
exists but the caller lacks `prism_audit` capability — returns the capability-gate error
(not E-QUERY-038 ColumnNotFound), confirming the gate ordering is preserved:
`check_internal_table_capabilities` BEFORE `check_query_column_availability`.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `check_query_column_availability` | `crates/prism-query/src/engine.rs` | Pure (reads schema, returns Result) |
| `InternalTableDescriptor::full_schema()` | `crates/prism-query/src/` (internal_tables.rs) | Pure (returns static schema) |
| `INTERNAL_TABLE_SPECS` | `crates/prism-query/src/` | Pure data (static map) |
| E-QUERY-038 gate orchestration | `crates/prism-query/src/engine.rs` `execute_inner` | Pure (plan-time only) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-11 PrismQL Query Engine
- `architecture/decisions/ADR-041-binding-context-walk.md` (14-position binding context walk)

**Anchor justifications (POL-4/POL-5):**
- SS-11 owns this story's scope because `check_query_column_availability` and the binding-context walk are SS-11 (PrismQL Query Engine) artifacts per ARCH-INDEX Subsystem Registry.
- No `depends_on` dependencies: this story is unblocked by PR #221 and has no other predecessor stories.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `SELECT * FROM prism_alerts` — wildcard, no specific column reference | Gate is not triggered (no specific column names to validate); `Ok(())` returned; DataFusion resolves the schema at execution time |
| EC-002 | `SELECT col FROM prism_audit` with capability present | Column gate runs after capability gate; if `col` is in the schema, `Ok(())`; if not, E-QUERY-038 |
| EC-003 | Internal table referenced in a JOIN clause | Current gate architecture targets the FROM source only (SQL mode). JOIN sources with `prism_*` names — behavior under the existing gate architecture. The implementer must check whether `check_query_column_availability` processes JOIN sources; if not, note the gap and add to OQ-001 amendment scope. |
| EC-004 | `prism_*` table name used as the base of a SqlPipe head | Same as SQL mode: the `prism_*` skip was in the SQL SELECT path; the SqlPipe head also routes through `check_query_column_availability`. Both modes require the same fix. |
| EC-005 | Column name that exists in one internal table but not another (e.g., `prism_audit.body` but not `prism_alerts.body`) | Gate correctly scopes `available_columns` to the specific queried table, not a union of all internal table schemas |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~150 | ~2,100 |
| BC-2.11.016 (E-QUERY-038 BC, 350+ lines) | ~350 | ~5,000 |
| BC-2.11.012 (virtual fields BC, 150 lines relevant section) | ~150 | ~2,100 |
| `crates/prism-query/src/engine.rs` `check_query_column_availability` (relevant section, ~400 lines) | ~400 | ~5,600 |
| `crates/prism-query/src/engine.rs` `check_internal_table_capabilities` (gate ordering, ~100 lines) | ~100 | ~1,400 |
| Internal table schema definitions (~200 lines) | ~200 | ~2,800 |
| BC files (2 BCs) | — | ~7,100 |
| **Total estimate** | | **~26,100 tokens** |

Fits within a 100k-token agent context window (~26%). No split required.

## Tasks

- [ ] Read `check_query_column_availability` in `crates/prism-query/src/engine.rs` in full to map the exact prism_* skip location and the surrounding column-lookup logic.
- [ ] Read `INTERNAL_TABLE_SPECS` / `InternalTableDescriptor::full_schema()` to confirm how to get the static column set for a given `prism_*` table name.
- [ ] Write Red Gate test `test_BC_2_11_016_internal_table_nonexistent_column_produces_E_QUERY_038` (AC-001) — RED must fail before code change.
- [ ] Write Red Gate test `test_BC_2_11_012_EC_11_035_source_type_on_internal_table_produces_E_QUERY_038_with_available_columns` (AC-002) — RED.
- [ ] Write Red Gate test `test_BC_2_11_016_internal_table_valid_column_does_not_produce_E_QUERY_038` (AC-003) — RED (passes with false positive if gate currently fires on valid columns; this test guards FP-001).
- [ ] Write Red Gate test `test_BC_2_11_016_internal_table_column_gate_does_not_duplicate_capability_check` (AC-004) — RED.
- [ ] Replace the `if table_name.starts_with("prism_") { return Ok(()); }` skip in `check_query_column_availability` with an internal-table schema lookup that feeds the column gate.
- [ ] Confirm the internal-table column set is accessible at gate time (no async I/O; purely from static `INTERNAL_TABLE_SPECS`).
- [ ] Run `just iter prism-query --no-fail-fast` to verify all tests GREEN.
- [ ] Run `just check` (full workspace) before declaring done.
- [ ] Flag any gap found in EC-003 (JOIN-clause handling) to orchestrator for OQ-001 product-owner routing.

## Previous Story Intelligence

**Prior cascade context:**
- `S-DEMO-PRISMQL-ONBOARDING-001-B` (merged PR #198 develop@5504c152 2026-06-22) introduced BC-2.11.016 with the 14-position binding-context walk. The `prism_*` skip was present from the initial implementation.
- `FIX-IEQ-ERRPATH-001` (PR #219, merged develop@8ea29823 2026-07-08) extended the binding-context walk to 14 positions but did not touch the `prism_*` skip.
- `DEFECT-CSDEVICES-EMPTY-PIPELINE-001` (PR #221, merged develop@5f1b5771 2026-07-11) adjudicated that `_source_type` is absent from internal table schemas (BC-2.11.012 v1.9 EC-11-035). This story delivers the pedagogical E-QUERY-038 improvement for all non-`_source_type` column misses on internal tables (and also improves the `_source_type` case from DataFusion failure to plan-time pedagogical error).

**Key invariant to preserve:** BC-2.11.016 v1.27 §Design Constraints: "E-QUERY-038 is EXCLUSIVELY plan-time." No DataFusion runtime errors should be routed through the E-QUERY-038 code path. The internal-table extension must be purely plan-time (i.e., the gate reads static schema, never executes a query to discover columns).

## Architecture Compliance Rules

- **BC-2.11.016 v1.27 §Design Constraints (plan-time only):** E-QUERY-038 MUST fire before DataFusion plan execution. Reading internal table schemas from `INTERNAL_TABLE_SPECS` is purely static — no I/O, no async, no DataFusion plan required.
- **FP-001 invariant:** E-QUERY-038 must never fire on a column that exists in the table's schema. The internal-table gate must use the same column lookup as the subsequent DataFusion execution path to avoid false positives.
- **Gate ordering invariant (BC-2.11.016 v1.27):** E-QUERY-037 → E-QUERY-038 → E-QUERY-039 → E-QUERY-041 → E-QUERY-042. The capability gate (`check_internal_table_capabilities`) must remain BEFORE the column gate for `prism_audit` and other capability-gated internal tables.
- **TD-VSDD-091:** Cite function names (`check_query_column_availability`, `InternalTableDescriptor::full_schema`, `INTERNAL_TABLE_SPECS`), not file/line numbers.
- **No `println!`:** All diagnostic output via `tracing::*!` with structured fields.
- **`#[non_exhaustive]` discipline:** If new error variant types are added to `ColumnNotFoundDetails` for internal tables, add `#[non_exhaustive]` and update `ci.yml EXPECTED` count.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `prism-core` | workspace-pinned | `InternalTableDescriptor`, `INTERNAL_TABLE_SPECS`, `PrismError::ColumnNotFound`, `ColumnNotFoundDetails` |
| `nextest` | workspace-pinned | `just iter prism-query` for fast inner loop |

No new external dependencies required. The internal table schema is purely in-process
static data already available in the `prism-core` crate.

**Forbidden dependencies:** `crates/prism-query` must not gain a dependency on
`prism-spec-engine` beyond what it currently has. Internal table schemas come from
`prism-core`, not from the spec engine's TOML parser.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-query/src/engine.rs` | Modify | Remove/replace `prism_*` skip in `check_query_column_availability`; add internal-table column lookup |
| `crates/prism-query/src/tests/` (engine_tests.rs or a new module) | Modify or create | 4 Red Gate tests (AC-001 through AC-004) |
| `.factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md` | **BC amended (v1.27)** | OQ-001 resolved — product-owner amended §Design Constraints (internal-table coverage paragraph, D-1670) and added EC-11-077/EC-11-078 |

No new crates. No Cargo.toml changes (assuming `prism-core`'s `INTERNAL_TABLE_SPECS`
is already accessible from `prism-query`).

## Open Questions

### OQ-001 — BC-2.11.016 amendment required before story can be dispatched as `ready` — RESOLVED (2026-07-11)

**Adjudication:** BC-2.11.016 amended to v1.27 (product-owner burst 2026-07-11). All five required changes from the original OQ are delivered:

1. **Skip clause removed/qualified:** §Design Constraints now states the `if table_name.starts_with("prism_") { return Ok(()); }` skip is removed. The prior design-constraint text describing the skip is replaced with the internal-table coverage paragraph.
2. **Schema-aware clause added:** §Design Constraints paragraph 2 (D-1670) documents that the gate covers `prism_*` tables using `InternalTableDescriptor::full_schema()` from `INTERNAL_TABLE_SPECS`.
3. **Gate-positions table:** The 14-position table is unchanged — internal tables route through the same SQL/pipe/filter mode positions as sensor tables; no new gate positions required for the `prism_*` path. The gate positions describe AST clause positions, not table types.
4. **Happy-path edge case:** EC-11-078 added — `SELECT valid_col FROM prism_alerts` → `Ok(())`, FP-001 preserved.
5. **EC-11-077 added:** `SELECT nonexistent_col FROM prism_alerts` → E-QUERY-038 with `available_columns` from internal table static schema.

**Note on OQ-001 point 5 (BC-2.11.012 EC-11-035 propagation):** BC-2.11.012 EC-11-035 already records the v1.9 directive. The implementer should confirm during delivery that the AC-002 test (`test_BC_2_11_012_EC_11_035_source_type_on_internal_table_produces_E_QUERY_038_with_available_columns`) supersedes the earlier DataFusion-plan-error form of the test; no BC-2.11.012 amendment is required since EC-11-035 already documents the OQ gap and this story's delivery resolves it at the code level.

**This story is now unblocked.** OQ-001 gate is cleared. Remaining gate before `status: ready`: S-7.01 AC↔BC bidirectional trace check.

### OQ-002 — JOIN-clause handling for internal tables
EC-003 in this story notes that `check_query_column_availability` may not currently
process `prism_*` tables referenced as JOIN targets (only as the primary FROM source).
The implementer must audit this during delivery and report back. If JOIN-target internal
tables also need the gate extension, this can be folded into this story's scope — report
to orchestrator before the PR is opened if scope expansion is needed.
