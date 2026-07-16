---
document_type: story
story_id: "S-PQLFN-GAP2-SOURCESELECT-GATE-001"
title: "Close Gap-2 — E-QUERY-039 gate for INSERT source_select join predicates and projection columns"
wave: post-demo
epic_id: maintenance
priority: P2
status: draft
version: "0.3"
spec_version: "v0.3"
level: L4
producer: story-writer
timestamp: "2026-07-15"
modified: "2026-07-15"
input-hash: ""
inputs:
  - .factory/specs/behavioral-contracts/BC-2.11.019-e-query-039-enrich-udf-not-found.md
  - .factory/specs/architecture/decisions/ADR-048-prismql-having-predicate-grammar-divergence-aggregate-fn-predicate-lhs.md
  - crates/prism-query/src/engine.rs
origin_finding: "DRIFT-PQLFN-OD7-GAP2-S307 — source_select {join_on, projections} un-gated after OD-7; deferred per BC-2.11.019 §OBS-003 (corrected in BC v1.17) and ADR-048 §D.7.6"
scheduling_note: "POST-DEMO — human directive 2026-07-15: create story to capture deferral, execute as part of work post-demo."
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-11]
# Subsystem anchor justification:
#   SS-11 (Query Execution Engine) owns check_enrich_udf_availability in
#   crates/prism-query/src/engine.rs — the sole production change site for this story.
#   No MCP surface changes; no SS-10 involvement.
target_module: "crates/prism-query"
crates_touched:
  - prism-query
behavioral_contracts: [BC-2.11.019]
# BC status: BC-2.11.019 is active (v1.26, modified 2026-07-15). This story amends it at
# delivery — BC-2.11.019 §OBS-003 "What remains un-gated" section must be updated in the
# SAME BURST as the code PR to record Gap-2 closure (per BC-2.11.019 §OBS-003 closing
# note: "When DML write-path enrichment support is added in a future story, this BC must
# be updated to extend the gate to cover source_select.join_on and source_select.projections").
# Keep BC status and version untouched NOW — amendment is a same-burst delivery obligation.
verification_properties: []
depends_on: [S-3.07]
# Dependency anchor justification:
#   Gap-2 closure (join_on + projections gate) is contractually tied to S-3.07 "prism-query:
#   Write Execution Pipeline" because the gap positions are only reachable once the DML
#   write-path enrichment execution path is live. Before S-3.07, DML execution no-ops to
#   Ok(vec![]) — adding the gate would be dead code. The gate must land in the same wave
#   as or immediately after S-3.07's write-path stubs (F-AUD-D1-04/05/06) are wired up.
blocks: []
points: 3
estimated_days: 1
risk: P2
acceptance_criteria_count: 5
red_gate_tests: 5
estimated_passes: "2"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-PQLFN-GAP2-SOURCESELECT-GATE-001: Close Gap-2 — E-QUERY-039 gate for INSERT source_select join predicates and projection columns

## §Origin — DRIFT-PQLFN-OD7-GAP2-S307

**Cascade:** DEFECT-PQL-FNCALL-LHS-001 pass-32 (F-PQLFN-P32-OBS-001) → OD-7 extension
**Session record:** BC-2.11.019 §OBS-003 (corrected in BC v1.17, 2026-07-14); ADR-048 §D.7.6 (introduced as OD-7 in ADR v1.13)
**Human directive:** 2026-07-15 — "make a story to capture the deferral that we need to execute as part of our work post demo"
**Drift anchor:** DRIFT-PQLFN-OD7-GAP2-S307

OD-7 (ADR-048 §D.7.6, introduced in ADR v1.13) extended `check_enrich_udf_availability` to gate
INSERT source_select WHERE predicates (Position 7 of the ADR-048 §D.7.1 predicate walk).
After OD-7, the remaining un-gated positions in `dml.source_select` are:

- `source_select.having` — **intentionally and permanently exempt** per ADR-048 §D.7.3
  (HAVING may legitimately reference aggregate functions; this is a design decision, not a
  story deferral and must not be gated by this story)
- `source_select.join_on` — JOIN predicates, Gap-2, THIS STORY
- `source_select.projections` — SELECT column list, Gap-2, THIS STORY

**Rationale for deferral from PR #223 (DEFECT-PQL-FNCALL-LHS-001):**

1. **Dead code pre-S-3.07.** The DML execution path currently no-ops to `Ok(vec![])` — the
   `prism-write` feature gate rejects DML at the surface before execution. The join predicate
   and projection positions are statically unreachable; adding the gate now would be dead code.
2. **Non-opaque failure mode.** If the positions became reachable today, an unregistered non-
   DataFusion-builtin fn-call would produce a DataFusion function-not-found error — degraded
   but not the opaque `E-INT-001` crash that motivated the original E-QUERY-039 gate.
3. **Sibling gate parity not yet required.** `check_temporal_literals` already walks the full
   `source_select` subtree; the asymmetry with the enrich gate is documented and acceptable
   while the write-path is unreachable.

**Why this must not be lost (post-S-3.07 urgency):**

Once S-3.07's write-path enrichment execution path is live, the join predicate and projection
positions become reachable. An analyst writing `INSERT INTO t SELECT threat_score(col) FROM src`
(projection) or `INSERT INTO t SELECT ... JOIN b ON threat_score(src.id) = b.id` (join) with a
typo in the infusion name will hit a raw DataFusion function-not-found error instead of the
targeted `E-QUERY-039` response with `available_infusions` and `did_you_mean`. This is an
analyst-facing taxonomy gap that becomes visible as soon as the write path is exercised.

## Narrative

As a Prism query engine maintainer delivering the DML write-path enrichment execution path
(S-3.07), I want `check_enrich_udf_availability` to fire `E-QUERY-039` at plan time for
unregistered enrichment UDF calls in INSERT source_select join predicates and SELECT projections,
so that analysts writing INSERT INTO ... SELECT enrichment queries receive the same targeted
error (with `available_infusions` and `did_you_mean`) as they would for equivalent SELECT or
pipe-mode queries, rather than a raw DataFusion function-not-found failure.

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.11.019 | E-QUERY-039 Enrich-UDF-Not-Found Plan-Time Gate | v1.26 | Primary anchor. §OBS-003 documents the Gap-2 deferral (source_select {join_on, projections}); §Postconditions defines the E-QUERY-039 firing condition and error payload shape. This story amends §OBS-003 at delivery to record Gap-2 closure. |

## Acceptance Criteria

### AC-001 — E-QUERY-039 fires for unknown UDF in INSERT source_select projection
(traces to BC-2.11.019 v1.26 postcondition — gate firing condition, SQL mode projection position)

Test: `test_gap2_pqlfn_insert_source_select_projection_unknown_udf_fires_e_query_039`

Given an `InfusionRegistry` with `threat_score` registered, when `check_enrich_udf_availability`
receives `INSERT INTO t (col) SELECT badudf(src_col) FROM src`, then
`PrismError::EnrichUdfNotFound` is returned with:
- `infusion == "badudf"`
- `available_infusions` contains `"threat_score"` (all registered names, lexicographically sorted)
- error Display renders byte-verbatim as `"E-QUERY-039: enrichment infusion 'badudf' is not registered; available: [threat_score]"` per BC-2.11.019 canonical template

This test is RED before the DML arm in `check_enrich_udf_availability` is extended. Prior to
this story, the `_ => {}` catch-all silently passes the query and the projection position is
never walked.

### AC-002 — Known UDF in INSERT source_select projection passes gate
(traces to BC-2.11.019 v1.26 postcondition — gate non-firing condition for registered UDF)

Test: `test_gap2_pqlfn_insert_source_select_projection_known_udf_passes_gate`

Given an `InfusionRegistry` with `threat_score` registered, when `check_enrich_udf_availability`
receives `INSERT INTO t (col) SELECT threat_score(src_col) FROM src`, then `Ok(())` is returned
(no E-QUERY-039). The projection position must not over-gate registered names.

### AC-003 — E-QUERY-039 fires for unknown UDF in INSERT source_select join predicate
(traces to BC-2.11.019 v1.26 postcondition — gate firing condition, SQL mode join-on position)

Test: `test_gap2_pqlfn_insert_source_select_join_on_unknown_udf_fires_e_query_039`

Given an `InfusionRegistry` with `threat_score` registered, when `check_enrich_udf_availability`
receives `INSERT INTO t (id) SELECT src.id FROM src JOIN b ON badudf(src.id) = b.id`,
then `PrismError::EnrichUdfNotFound` is returned with:
- `infusion == "badudf"`
- `available_infusions` contains `"threat_score"` (all registered names, lexicographically sorted)
- error Display renders byte-verbatim as `"E-QUERY-039: enrichment infusion 'badudf' is not registered; available: [threat_score]"` per BC-2.11.019 canonical template

This test is RED before the DML arm is extended to walk join predicates in `source_select`.

### AC-004 — Known UDF in INSERT source_select join predicate passes gate
(traces to BC-2.11.019 v1.26 postcondition — gate non-firing condition for registered UDF)

Test: `test_gap2_pqlfn_insert_source_select_join_on_known_udf_passes_gate`

Given an `InfusionRegistry` with `threat_score` registered, when `check_enrich_udf_availability`
receives `INSERT INTO t (id) SELECT src.id FROM src JOIN b ON threat_score(src.id) = b.id`,
then `Ok(())` is returned (no E-QUERY-039). The join-on position must not over-gate registered names.

### AC-005 — HAVING exemption regression lock: no E-QUERY-039 in source_select.having
(traces to BC-2.11.019 v1.26 postcondition — gate non-firing condition; ADR-048 §D.7.3 permanent exemption)

Test: `test_gap2_pqlfn_insert_source_select_having_exemption_does_not_fire_e_query_039`

Given an `InfusionRegistry` (with or without registrations), when `check_enrich_udf_availability`
receives `INSERT INTO t (col, cnt) SELECT col, count(*) FROM src GROUP BY col HAVING count(*) > 5`,
then `Ok(())` is returned. `source_select.having` is permanently exempt per ADR-048 §D.7.3
(HAVING may legitimately reference aggregate functions). This test must remain GREEN and must
NOT regress if the story implementation inadvertently walks into the having position.

**Boundary marker:** this test is the load-bearing guard that prevents `source_select.having`
from being silently included in the gap closure. Any implementation that walks
`collect_unknown_scalars_from_sql_query` on the full `source_select` SqlQuery (which includes
HAVING) will fail this test unless the HAVING position is explicitly excluded.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `check_enrich_udf_availability` (DML arm extension) | `crates/prism-query/src/engine.rs` | Pure (plan-time gate; no I/O; returns `Result`) |
| `collect_unknown_scalar_from_expr` (reused for projection items) | `crates/prism-query/src/engine.rs` | Pure |
| `collect_unknown_scalar_from_predicate` (reused for join.on conditions, if applicable) | `crates/prism-query/src/engine.rs` | Pure |
| BC-2.11.019 §OBS-003 amendment | `.factory/specs/behavioral-contracts/BC-2.11.019-e-query-039-enrich-udf-not-found.md` | Doc |

Architecture section references:
- `architecture/module-decomposition.md` §SS-11 Query Execution Engine

**Anchor justifications (POL-4/POL-5):**

SS-11 is the sole subsystem because `check_enrich_udf_availability` lives entirely within
`crates/prism-query/src/engine.rs`, which is the core of the SS-11 Query Execution Engine per
the ARCH-INDEX Subsystem Registry. No MCP interface (SS-10) changes are required — the gate
fires inside the query pipeline before any MCP response is composed.

**Walker implementation note (TD-VSDD-091 — no line numbers):**

The implementer MUST NOT call `collect_unknown_scalars_from_sql_query` on the whole
`dml.source_select` SqlQuery, because that function walks ALL positions including HAVING —
which is permanently exempt (AC-005 regression lock). Instead, walk the two Gap-2 positions
individually:
1. `source_select.select.items` — call `collect_unknown_scalar_from_expr` on each `SelectItem::Expr { expr, .. }`.
2. `source_select.joins[*].on` — call `collect_unknown_scalar_from_expr` on each join's `.on` field.

The `sql_unknown_names` Vec (DataFusion built-in filter applied) is the correct accumulator
for these positions, consistent with other SQL-mode positions in `check_enrich_udf_availability`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | INSERT with empty projections (only `*`) | `SELECT *` expands to field paths, not FuncCall nodes — E-QUERY-039 does not fire; gate is a no-op. |
| EC-002 | INSERT source_select projection with a DataFusion built-in function (`SELECT lower(col) FROM src`) | DataFusion built-in exclusion (DATAFUSION_BUILTIN_FUNCTION_NAMES) applies — `lower` passes the gate; E-QUERY-039 does not fire. Matches existing Select-mode behavior. |
| EC-003 | INSERT source_select join ON with a DataFusion built-in function | Same DataFusion built-in exclusion applies to join.on position as to projection position. |
| EC-004 | INSERT with no source_select (e.g., simple DELETE WHERE) | `dml.source_select` is `None` — the DML arm skips the source_select walk; no panic; gate is a no-op for that query. |
| EC-005 | INSERT source_select projection with a registered infusion AND an unregistered function in HAVING | E-QUERY-039 fires for the unregistered projection name; HAVING position is not walked (AC-005 exemption intact). Gate reports the projection finding, not the HAVING position. |
| EC-006 | BC-2.11.019 §OBS-003 amendment missing at delivery | This is a process failure, not a functional edge case. The BC amendment is a same-burst delivery obligation — the PR cannot merge without the §OBS-003 "What remains un-gated" section updated to confirm Gap-2 closure. |

## Token Budget Estimate

| Item | Lines (est.) | Tokens (est.) |
|------|-------------|--------------|
| Story spec (this file) | ~220 | ~3,100 |
| BC-2.11.019 v1.26 (E-QUERY-039 Enrich-UDF-Not-Found Plan-Time Gate) | ~370 | ~5,200 |
| ADR-048 §D.7 sections (§D.7.1, §D.7.3, §D.7.6 OD-7) | ~120 | ~1,700 |
| `check_enrich_udf_availability` function in engine.rs (~135 lines) | ~135 | ~1,900 |
| `collect_unknown_scalars_from_sql_query` function in engine.rs (~60 lines) | ~60 | ~850 |
| `collect_unknown_scalar_from_expr` function in engine.rs (~40 lines) | ~40 | ~560 |
| New/modified test functions (5 tests, ~80 lines) | ~80 | ~1,100 |
| BC-2.11.019 §OBS-003 amendment (~20 lines changed) | ~20 | ~280 |
| **Total estimate** | | **~14,700 tokens** |

Fits within a 100k-token agent context window (~15%). No split required. Load only the
`check_enrich_udf_availability` and supporting walker functions from engine.rs — do NOT load
the entire 12,000-line file. The test cases for this story are entirely unit tests in
`#[cfg(test)] mod tests` within engine.rs (no DTU fixture required).

## Tasks

- [ ] Read BC-2.11.019 v1.26 §OBS-003 and §Postconditions (gate firing condition + E-QUERY-039 payload shape) before writing any code.
- [ ] Read ADR-048 §D.7.1, §D.7.3 (HAVING exemption), and §D.7.6 (OD-7, Position 7) from the ADR file.
- [ ] Read `check_enrich_udf_availability` and the three walker functions (`collect_unknown_scalars_from_sql_query`, `collect_unknown_scalar_from_expr`, `collect_unknown_scalar_from_predicate`) in `crates/prism-query/src/engine.rs`.
- [ ] Confirm `DmlNode` struct fields (specifically `source_select: Option<SqlQuery>` and `SqlQuery.select.items`, `SqlQuery.joins[*].on`) in `crates/prism-query/src/ast.rs`.
- [ ] Add 5 Red Gate tests to the E-QUERY-039 test section in `crates/prism-query/src/engine.rs` `#[cfg(test)] mod tests`. All 5 must fail RED before production changes (TDD strict).
- [ ] Run `cargo nextest run -p prism-query -E 'test(gap2_pqlfn)'` — confirm all 5 tests RED.
- [ ] Extend the `Ast::Sql(SqlStatement::Dml(dml))` arm of `check_enrich_udf_availability`:
  - Walk `dml.source_select.select.items` via `collect_unknown_scalar_from_expr` into `sql_unknown_names`.
  - Walk `dml.source_select.joins[*].on` via `collect_unknown_scalar_from_expr` into `sql_unknown_names`.
  - EXPLICITLY DO NOT walk `dml.source_select.having` (permanent exemption — AC-005 regression lock).
  - DO NOT call `collect_unknown_scalars_from_sql_query` on the full `source_select` (it walks HAVING — see AC-005).
- [ ] Run `cargo nextest run -p prism-query -E 'test(gap2_pqlfn)'` — confirm all 5 tests GREEN.
- [ ] Run `just iter prism-query` — full crate test suite must remain GREEN (no regressions).
- [ ] **Same-burst BC amendment:** Update BC-2.11.019 §OBS-003 "What remains un-gated" section to record Gap-2 closure. The section must state that `source_select.join_on` and `source_select.projections` are now gated (reference this story's ID), `source_select.having` remains permanently exempt per ADR-048 §D.7.3, and the walker asymmetry with `check_temporal_literals` is fully resolved (except the ratified having exemption). Bump BC version and add changelog row. Route to product-owner for BC amendment.
- [ ] Run `just check` (full workspace) once before declaring done.

## Previous Story Intelligence

This is the first story in this gap-closure track. Context from the deferral:

- **OD-7 (ADR-048 §D.7.6, introduced in ADR v1.13)** extended the aggregate-in-predicate gate (E-QUERY-001) to
  cover INSERT source_select WHERE (Position 7). This exposed that `check_enrich_udf_availability`
  had a corresponding gap: the E-QUERY-039 gate did not walk DML positions at all (`_ => {}`
  catch-all). OD-7 was the correct fix for the aggregate gate; the E-QUERY-039 DML extension
  was deferred as Gap-2.

- **BC-2.11.019 §OBS-003 (corrected in BC v1.17)** documents the post-OD-7 state: `source_select.where_` is
  gated, `source_select.having` is permanently exempt (not a story deferral), and
  `source_select.{join_on, projections}` are Gap-2 deferred to S-3.07.

- **DEFECT-PQL-FNCALL-LHS-001 PR #223** is the source PR that closes OD-7 and creates the
  documented DRIFT-PQLFN-OD7-GAP2-S307 drift item.

- **S-3.07** (Write Execution Pipeline) has `status: partial-merge` with write-path stubs
  F-AUD-D1-04/05/06 in flight. This story MUST NOT be dispatched until those stubs are live
  (otherwise the gate is dead code and the tests exercise unreachable paths).

## Architecture Compliance Rules

- **No full-SqlQuery walk on source_select:** Do NOT call `collect_unknown_scalars_from_sql_query`
  on the full `dml.source_select`. That function walks ALL six positions including HAVING, which
  is permanently exempt (AC-005 regression lock). Walk the two Gap-2 positions individually.
- **DataFusion built-in exclusion applies to new positions:** New positions use `sql_unknown_names`
  (not `pipe_enrich_names`) — the `DATAFUSION_BUILTIN_FUNCTION_NAMES` filter applies when the
  names are later validated. This is consistent with all other SQL-mode positions.
- **`#[non_exhaustive]` discipline:** No new public types are introduced. If a new error variant
  or detail struct is needed, consult CLAUDE.md §Conventions for the `#[non_exhaustive]` gate.
- **No `unwrap()` or `expect()` in production code:** Use `?` propagation or structured error
  returns consistent with the existing `check_enrich_udf_availability` implementation.
- **SAP-1:** No new `event_type =` tracing emissions. This gate returns `Err(...)` via `?`
  propagation — no `tracing::*!(event_type=...)` call is needed or appropriate. D-765 precedent:
  `?`-propagation provides the audit trail via `AuditEntry` without a separate catalog row.
- **TD-VSDD-091:** Cite function names (`check_enrich_udf_availability`,
  `collect_unknown_scalar_from_expr`, `collect_unknown_scalars_from_sql_query`) and structural
  anchors in comments — NOT `engine.rs:NNN` line numbers.
- **Forbidden dependencies:** `prism-query` MUST NOT gain new crate dependencies for this story.
  The extension reuses existing helper functions already in scope.
- **POL-24 byte-verbatim error templates:** AC-001 and AC-003 assert the E-QUERY-039 Display
  output byte-for-byte. The canonical template (from BC-2.11.019): `"E-QUERY-039: enrichment
  infusion '{infusion}' is not registered; available: [{available_infusions}]{did_you_mean}"`.
  Tests must assert this exact format using `format!("{}", err)` or equivalent, not a substring
  contains check.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `strsim` | workspace-pinned | Levenshtein for `did_you_mean` — already a dependency in `prism-query`; reuse existing call pattern |
| `nextest` | workspace-pinned | `just iter prism-query` for fast inner loop |
| DataFusion | workspace-pinned (v53) | `DATAFUSION_BUILTIN_FUNCTION_NAMES` LazyLock already initialized; no new DataFusion API calls |

No new dependencies. All walker functions (`collect_unknown_scalar_from_expr`,
`collect_unknown_scalar_from_predicate`) are already in scope within `engine.rs`.

**Forbidden dependencies (build-time enforcement):** `prism-query` MUST NOT import `prism-mcp`,
`prism-sensors`, or any new external crates for this story.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-query/src/engine.rs` | Modify | (1) Add 5 Red Gate tests to the E-QUERY-039 `#[cfg(test)] mod tests` block. (2) Extend `check_enrich_udf_availability` DML arm: add `dml.source_select.select.items` and `dml.source_select.joins[*].on` walks. |
| `BC-2.11.019-e-query-039-enrich-udf-not-found.md` | Modify (same-burst) | Update §OBS-003 "What remains un-gated" section: record Gap-2 closure, confirm `source_select.having` permanent exemption intact, note walker asymmetry fully resolved. Bump version + changelog row. Route to product-owner. |

## References

| Artifact | Location | Relevance |
|----------|----------|-----------|
| BC-2.11.019 v1.26 | `.factory/specs/behavioral-contracts/BC-2.11.019-e-query-039-enrich-udf-not-found.md` | Primary behavioral contract; §OBS-003 documents Gap-2 deferral; §Postconditions defines E-QUERY-039 firing logic and payload. |
| ADR-048 v1.17 | `.factory/specs/architecture/decisions/ADR-048-prismql-having-predicate-grammar-divergence-aggregate-fn-predicate-lhs.md` | §D.7.3 HAVING permanent exemption; §D.7.6 OD-7 (INSERT source_select WHERE gated, introduced in ADR v1.13). |
| DRIFT-PQLFN-OD7-GAP2-S307 | STATE.md decision log | Drift item anchoring this deferral; records rationale and S-3.07 dependency. |
| S-3.07 | `.factory/stories/S-3.07-write-execution.md` | Write Execution Pipeline — prerequisite; Gap-2 gate is dead code until S-3.07's DML execution path is live. |

## Changelog

| Version | Date | Change | Source |
|---------|------|--------|--------|
| v0.3 | 2026-07-15 | Date-accuracy correction: 6 wrong-date occurrences corrected to 2026-07-15 (frontmatter timestamp/modified, scheduling_note, §Origin human-directive line, v0.2 changelog date, v0.1 changelog date). Orchestrator dispatch-date error; directive was given 2026-07-15 per coordinator correction. |
| v0.2 | 2026-07-15 | POL-23/TD-VSDD-091 pin-accuracy sweep. Six sites corrected: (1) frontmatter `origin_finding` — removed ambiguous `OBS-003 v1.17` bare pin, replaced with explicit historical framing `(corrected in BC v1.17)`; (2) §Origin Session record — `BC-2.11.019 §OBS-003 v1.17` → `(corrected in BC v1.17, 2026-07-14)` and `ADR-048 §D.7.6 v1.13` → `(introduced as OD-7 in ADR v1.13)`; (3) §Origin body — `v1.13` → `introduced in ADR v1.13`; (4) §Previous Story Intelligence OD-7 line — same historical reframe; (5) §Previous Story Intelligence BC line — `v1.17` → `(corrected in BC v1.17)`; (6) §References ADR-048 pin WRONG (`v1.14`) → live pin `v1.17`. Changelog v0.1 row left intact (historical record, TD-VSDD-091 exempt). | POL-23; TD-VSDD-091 |
| v0.1 | 2026-07-15 | Initial draft — 5 ACs, 5 Red Gate tests, same-burst BC amendment obligation, POST-DEMO scheduling note per human directive 2026-07-15. | Human directive 2026-07-15; DRIFT-PQLFN-OD7-GAP2-S307; BC-2.11.019 §OBS-003 v1.17 |
