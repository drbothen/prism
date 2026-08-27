---
document_type: story
story_id: S-ENGINE-LIMIT-EARLY-STOP-001
title: "LIMIT-aware early-stop pagination — FetchContext.early_stop_limit field + execute_impl check + spec_driven_adapter wiring (ADR-060 §D8)"
level: "L4"
wave: xdome-wave-a
epic_id: E-XDOME-EXPANSION
priority: P0
status: draft
# BC status: BC-2.16.002 active — §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)" postcondition
# authored and anchored to this story ID. BC-2.16.015 draft — EC-016-015-007 and
# TV-BC-2.16.015-006 referenced (trace-only); promoted to active by S-CLAROTY-VULNS-001 merge per POL-14, not this story.
producer: story-writer
timestamp: "2026-08-26T00:00:00Z"
version: "1.11"
modified: "2026-08-27"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.015-claroty-vulnerabilities-table.md"
  - ".factory/specs/architecture/decisions/ADR-060-limit-aware-early-stop-pagination.md"
input-hash: "2153bc0"
# input-hash: updated 2026-08-27 (v1.11); ADR-060 v1.4 + BC-2.16.002 v2.40 + BC-2.16.015 v1.8 inputs
traces_to: ["BC-2.16.002", "BC-2.16.015"]
points: 8
estimated_days: 2
tdd_mode: strict
subsystems: [SS-01, SS-07, SS-11, SS-16]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-16 (Spec Engine) owns this story's scope because the primary implementation
#     is in `crates/prism-spec-engine/src/pipeline.rs` — `FetchContext` struct and
#     `PipelineExecutor::execute_impl` loop. SS-16 is the canonical owner of
#     prism-spec-engine per ARCH-INDEX Subsystem Registry.
#   SS-01 (Sensor Adapters) owns this story's scope because `SpecDrivenSensorAdapter::fetch`
#     in `crates/prism-bin/src/spec_driven_adapter.rs` is the sole production caller of
#     `FetchContext::new` and the wiring point that maps `params.limit` to `early_stop_limit`.
#     SS-01 governs the outbound sensor HTTP adapter surface per ARCH-INDEX.
#   SS-11 (Query Execution) owns this story's scope because `crates/prism-query/src/materialization.rs`
#     is the enforcement site for the `ast_is_reducing_plan` plan-shape gate (§D8.7) and
#     `run_materialization_pipeline` `fetch_limit` derivation (§D8.8). SS-11 governs
#     the query execution and materialization surface per ARCH-INDEX Subsystem Registry.
#   SS-07 (Adapter Pagination & Response Cache) owns this story's scope because the per-page
#     early-stop check in `PipelineExecutor::execute_impl` (§D8.2) fires within the pagination
#     loop and `derive_response_cache_key` fetch_limit coherence (§D8.8) is the single-binding
#     invariant that ties cache key to fan-out target. SS-07 governs the adapter pagination
#     and response cache surface per ARCH-INDEX Subsystem Registry.
target_module: prism-spec-engine
crates_touched: [prism-spec-engine, prism-bin, prism-query]
# crates_touched:
#   prism-query:
#     MODIFY src/materialization.rs:
#       (a) Add `ast_is_reducing_plan(ast: &Ast) -> bool` function
#       (b) Add `expr_contains_aggregate_or_window(expr: &Expr) -> bool` helper (three-part detection: Aggregate variants, FuncCall::Window, recursion into FuncCall::Scalar::args)
#       (c) Update `fetch_limit` derivation in `run_materialization_pipeline` to use plan-shape gate
#           (BEFORE fan-out target construction; where_filters computed for push-down + cache key
#            but NOT passed to gate) per ADR-060 §D8.7
#   prism-spec-engine:
#     MODIFY src/pipeline.rs:
#       (a) Add `early_stop_limit: Option<usize>` field to `FetchContext` struct
#       (b) Add `early_stop_limit: Option<usize>` parameter to `FetchContext::new`
#       (c) Add early-stop check in `PipelineExecutor::execute_impl` loop (after DI-019 check)
#     Callers inside pipeline.rs #[cfg(test)] blocks: ~15 in-file test sites — all pass `None`
#   prism-bin:
#     MODIFY src/spec_driven_adapter.rs:
#       `SpecDrivenSensorAdapter::fetch` — map `params.limit` to `early_stop_limit` and
#       pass to `FetchContext::new`
#   Integration test files that call FetchContext::new must be updated to pass `None`:
#     crates/prism-spec-engine/tests/pipeline_http_integration.rs
#     crates/prism-spec-engine/tests/bc_2_11_007_pushdown_test.rs
#     crates/prism-spec-engine/tests/parity/armis.rs
#     crates/prism-spec-engine/tests/parity/crowdstrike.rs
#     crates/prism-spec-engine/tests/parity/claroty.rs
#     crates/prism-spec-engine/tests/parity/cyberint.rs
#     crates/prism-spec-engine/tests/pipeline_oauth_retry.rs
#     crates/prism-spec-engine/tests/bc_2_16_002_crowdstrike_two_step.rs
#     crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs
#     crates/prism-spec-engine/tests/bc_2_01_017_static_cookie_auth_provider.rs
#     crates/prism-spec-engine/tests/bc_2_16_002_test.rs
#     crates/prism-spec-engine/tests/bc_2_16_013_crowdstrike_multiregion.rs
#     crates/prism-spec-engine/tests/plugin_integration_tests.rs
#     crates/prism-spec-engine/tests/defect_csdevices_fetch_devices_fan_out.rs
#     tests/external/non-exhaustive-violation/src/struct_violations.rs (doc-comment; read-only verify)
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.002
  # BC-2.16.002 — §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)":
  # PipelineExecutor::execute_impl stops at complete page boundaries when early_stop_limit
  # satisfied; truncated=false (reserved for DI-019); DataFusion trims post-fetch;
  # OffsetLimit and CursorToken only; D8.5 ORDER BY limitation documented.
  # Plan-Shape Gate (ADR-060 §D8.7 v1.3): ast_is_reducing_plan Conditions A–J + conservative default suppress early-stop;
  # where_filters NOT forwarded to gate; EC-016-002-001..018 cover each suppression condition and ORDER BY positive control.
  # (BC-2.16.015 is trace-only — in traces_to: only; promoted by S-CLAROTY-VULNS-001 per POL-14)
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios at remove-uncertainty time.
# Stored under the holdout directory; test-writer and implementer MUST NOT read them.
# Story-level holdout gate is BLOCKING before demo/push (human-approved 2026-07-13).
depends_on: []
# depends_on: No delivery-time scheduling dependency. S-ENGINE-H2-LARGE-RESPONSE-001 is
#   independent and parallel to this story. Both stories unblock S-CLAROTY-VULNS-001 live-green
#   independently — they are parallel, not sequential. DEFECT-2 (ADR-060 §Source) is distinct
#   from DEFECT-1 (ADR-059 §Source); each fix is self-contained.
blocks: [S-CLAROTY-VULNS-001]
# blocks: S-CLAROTY-VULNS-001 live-green on `LIMIT N` queries. Even after the h2 window fix
#   (S-ENGINE-H2-LARGE-RESPONSE-001), a `SELECT * LIMIT 1` against claroty_vulnerabilities
#   fetches ALL pages (DEFECT-2, ADR-060 §Context), exhausting the 30s budget. This story
#   prevents that by stopping after ceil(1/1000) = 1 page (ADR-060 §Consequences).
acceptance_criteria_count: 7
risk: MEDIUM
# Risk justification:
#   FetchContext::new signature expansion is a BREAKING CHANGE for all callers.
#   ~15 in-file test sites + ~14 integration test files must each pass `None`.
#   TD-VSDD-060 sibling-sweep is mandatory and explicitly enumerated in Tasks.
#   The `#[non_exhaustive]` attribute on FetchContext prevents struct-literal construction
#   outside the crate, so the only callers affected are those using FetchContext::new —
#   enumerated in the crates_touched comment above and in Task 5.
#   ADR-060 §D8.1 phrasing discrepancy: RESOLVED.
assumption_validations:
  - claim: "Expanding FetchContext::new (constructor of a #[non_exhaustive] struct) by one parameter is source-compatible in-workspace when all callers are updated in-tree; no API-surface gate beyond the non-exhaustive audit is triggered."
    verdict: "CONFIRMED (remove-uncertainty 2026-08-26). Ground-truthed against code: crates/prism-spec-engine/src/pipeline.rs FetchContext is #[non_exhaustive] #[derive(Debug, Clone)] with new(client_id, query_filters). Adding a parameter to `new` is a signature change requiring all callers updated (breaking only if external callers existed — none do; #[non_exhaustive] blocks external struct-literal construction and the crate is workspace-internal). Adding a FIELD to an already-registered #[non_exhaustive] type introduces NO new symbol, so EXPECTED_SYMBOLS in scripts/check-non-exhaustive-per-symbol.py needs no update. Story AC-001 and §Architecture Compliance Rules state this correctly."
  - claim: "No DataFusion research/dependency is needed: LIMIT is available as QueryParams.limit: u64 at the adapter, pre-extracted before the call to SpecDrivenSensorAdapter::fetch. ADR-060 §D8.1 previously contained an inaccurate description of the LIMIT-extraction path; ADR-060 corrected §D8.1 to match this reality."
    verdict: "CONFIRMED (remove-uncertainty 2026-08-26). Ground-truthed: QueryParams.limit: u64 field defined in prism-sensors sensor adapter module; the 0 = no-limit sentinel corroborated by materialization options.limit.unwrap_or(0) in prism-query materialization module. SpecDrivenSensorAdapter::fetch receives params.limit pre-extracted; no physical-plan inspection required. ADR-060 corrected §D8.1 to match this implementation; §Authority note updated accordingly (2026-08-26); discrepancy RESOLVED. No DataFusion API research required."
risk_mitigations:
  - "FetchContext::new signature expansion is fully swept in Task 5 (TD-VSDD-060); all in-file + integration callers pass None. Verified constructor shape against live code 2026-08-26."
---

# S-ENGINE-LIMIT-EARLY-STOP-001: LIMIT-Aware Early-Stop Pagination — FetchContext Field, execute_impl Check, and spec_driven_adapter Wiring

## Authority

**BC-2.16.002 §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)"** is
the primary governing contract. Read this postcondition in full before implementing. It
specifies: `FetchContext.early_stop_limit: Option<usize>`; check placement IMMEDIATELY AFTER
DI-019 in `PipelineExecutor::execute_impl`; `truncated` NOT set on early-stop; `OffsetLimit`
and `CursorToken` only; `None` = unchanged full pagination; D8.5 ORDER BY limitation text.
Also read the atomicity-reconciliation scope clause in the partial-record-discard postcondition
(amended by ADR-060 §Atomicity Reconciliation) confirming that early-stop is COMPATIBLE with
the "all-or-nothing" error-path invariant.

**BC-2.16.015 §Edge Cases EC-016-015-007** and **§Canonical Test Vectors TV-BC-2.16.015-006**
are Claroty-specific trace references (BC-2.16.015 is in `traces_to:` only — not in
`behavioral_contracts:`; core contract delivery is by S-CLAROTY-VULNS-001): `LIMIT 1` against
page_size=1000 triggers early-stop after 1 page; `PipelineResult.truncated=false`; DataFusion
trims to 1 row. EC-016-015-008 (COUNT suppresses early-stop via §D8.7 Condition A) is also
referenced.

**ADR-060 §D8** is the decision. Read §D8.1 through §D8.5 (FetchContext field, execute_impl
check placement, post-break semantics, applicable pagination modes, ORDER BY documentation).
**Read §D8.7 (Plan-Shape Gate v1.3):** `ast_is_reducing_plan(&ast)` function in `materialization.rs`;
Conditions A–J + conservative default; `where_filters` computed for push-down and cache key derivation
but NOT forwarded to gate (gate performs own AST inspection via `has_client_side_where`); temporal-only
WHERE safety; ORDER BY non-suppression; gate application in `run_materialization_pipeline`. **Read §D8.8
(Single-Binding Coherence):** `fetch_limit` feeds both cache-key and fan-out target; gate
preserves the invariant.
§D8.6 (timeout_secs overlay wiring) is DEFERRED to S-ENGINE-TIMEOUT-OVERLAY-WIRE-001.

**ADR-060 §D8.1 phrasing discrepancy — RESOLVED:** ADR-060 §D8.1 now
correctly specifies reading `QueryParams.limit: u64` (0 = no limit). The `limit` value is
pre-extracted into `QueryParams` before the call to `SpecDrivenSensorAdapter::fetch`; no
DataFusion plan-step inspection is required. The wiring
`if params.limit == 0 { None } else { Some(params.limit as usize) }` matches the corrected
ADR exactly. The implementation and the ADR now AGREE; no further architect correction is
outstanding.

**ADR-060 §Atomicity Reconciliation ruling:** "Atomic" means all-or-nothing on HTTP ERROR, not
"must fetch the entire dataset." LIMIT early-stop is a SUCCESS-PATH non-error exit at a
COMPLETE page boundary. The `truncated` field is semantically reserved for DI-019 capacity
overflow ONLY (NOT for query-driven early stops per ADR-060 §D8.3).

---

## Narrative

As a SOC analyst issuing a `SELECT * FROM claroty_vulnerabilities | LIMIT 1` query,
I want the pipeline to stop fetching pages once it has accumulated enough records to satisfy
the LIMIT,
so that a `LIMIT 1` query issues 1 HTTP page request (not 5+) and completes within the
per-page time budget rather than exhausting the 30s query timeout.

## Background

Even after the h2 window fix (S-ENGINE-H2-LARGE-RESPONSE-001), a `LIMIT 1` query against
`claroty_vulnerabilities` (page_size=1000, ~1.1 MiB/page) still fetches ALL pages because
`PipelineExecutor::execute_impl` loops until API pagination exhaustion or DI-019 10K cap.
DataFusion applies its LIMIT operator only on the fully-materialized result set — too late
to prevent excess HTTP fetches. Concretely: `LIMIT 1` triggers 5+ HTTP POST requests (~5.5 MiB
total); at ~5-10s/page after the h2 fix, this easily exceeds the 30s budget (DEFECT-2,
ADR-060 §Context).

The fix threads an `early_stop_limit: Option<usize>` field through `FetchContext` and inserts
a check in the `execute_impl` pagination loop IMMEDIATELY AFTER the DI-019 truncation check.
The sole production wiring point is `SpecDrivenSensorAdapter::fetch`, where `params.limit`
(already pre-extracted into `QueryParams`) is mapped to the new field.

**Story-level holdout gate:** After LOCAL 3-CLEAN and BEFORE demo/push, holdout-evaluator runs
hidden SINGLE-USE scenarios. The gate is BLOCKING — unsatisfied scenarios reset the LOCAL
streak per BC-5.39.001.

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | v2.40 | §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)": FetchContext field, execute_impl check placement, truncated=false semantics, applicable pagination modes, D8.5 ORDER BY limitation. Plan-Shape Gate (ADR-060 §D8.7 v1.3): Conditions A–J + conservative default suppress early-stop; where_filters NOT forwarded to gate; `fetch_limit=0` sentinel flow through `QueryParams.limit=0` → `FetchContext::early_stop_limit=None`; EC-016-002-001..018 edge cases. Atomicity-reconciliation scope clause. |

*BC-2.16.015 (trace-only — not in behavioral_contracts): EC-016-015-007 (LIMIT 1 early-stop, unaffected by §D8.7), EC-016-015-008 (COUNT suppresses early-stop), TV-BC-2.16.015-006. Core contract delivered by S-CLAROTY-VULNS-001; promoted to active on that story's merge per POL-14, not this story. See §References.*

## Acceptance Criteria

### AC-001: `FetchContext` gains `early_stop_limit: Option<usize>` field; constructor signature updated; non_exhaustive and derive preserved (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop, FetchContext field ADR-060 §D8.1)

`FetchContext` in `crates/prism-spec-engine/src/pipeline.rs` gains:
```rust
pub early_stop_limit: Option<usize>,
```
`FetchContext::new(client_id: OrgSlug, query_filters: HashMap<String, String>, early_stop_limit: Option<usize>)` is the new constructor signature. `#[non_exhaustive]` and `#[derive(Debug, Clone)]` are preserved. `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` does NOT need updating because `FetchContext` is already registered (no new type is introduced; only a field is added to an existing `#[non_exhaustive]` struct).

**Test:** `test_BC_2_16_002_early_stop_fetch_context_new_stores_early_stop_limit`

### AC-002: `PipelineExecutor::execute_impl` checks `early_stop_limit` immediately after DI-019; breaks without setting `truncated = true` (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop §D8.2, §D8.3)

In `PipelineExecutor::execute_impl` in `crates/prism-spec-engine/src/pipeline.rs`, the
early-stop check is inserted immediately after the DI-019 block (the block that sets
`truncated = true; break 'steps` when `all_records.len() >= MAX_PIPELINE_RECORDS`):

```rust
if let Some(limit) = context.early_stop_limit {
    if all_records.len() >= limit {
        break 'steps;
    }
}
```

`truncated` is NOT set in this block. `break 'steps` exits the outer steps loop. The check
fires only at COMPLETE page boundaries (after a full page has been received and appended).

**Test:** `test_BC_2_16_002_early_stop_pipeline_stops_without_setting_truncated`

### AC-003: When `early_stop_limit = None`, full pagination proceeds unchanged; when `early_stop_limit = Some(N)`, only `ceil(N / page_size)` pages are fetched (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop None-branch and early-stop behavior)

Behavioral proof via wiremock multi-page mock (page_size=10, 3 pages available):
- `early_stop_limit = None` → 3 HTTP requests issued (all pages fetched); behavior unchanged.
- `early_stop_limit = Some(1)` → 1 HTTP request issued (first page has 10 rows ≥ 1); `truncated = false`.

**Test:** `test_BC_2_16_002_early_stop_none_fetches_all_pages` (None case)
**Test:** `test_BC_2_16_002_early_stop_pipeline_stops_without_setting_truncated` (RG-002; covers Some case: 1 page fetched when early_stop_limit=Some(1), truncated=false)
**Test:** `test_BC_2_16_002_early_stop_multi_page_stops_after_second_page` (k>1 proof: early_stop_limit=Some(11), page_size=10, 3 pages available; 2 pages fetched, stops after second page accumulates 20 ≥ 11 records; truncated=false)

### AC-004: DI-019 10K truncation check fires BEFORE early-stop; `truncated = true` when DI-019 fires (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop §D8 ordering; DI-019 unchanged)

When `all_records.len() >= MAX_PIPELINE_RECORDS` (10000) AND `early_stop_limit = Some(N)` for
some N > MAX_PIPELINE_RECORDS, the DI-019 block fires first (it precedes the early-stop block
in the source), sets `truncated = true`, and breaks. The early-stop block is never reached.
DI-019 behavior is UNCHANGED by this story.

**Test:** `test_BC_2_16_002_early_stop_di019_fires_before_early_stop_check`

### AC-005: `SpecDrivenSensorAdapter::fetch` maps `params.limit` to `early_stop_limit`; passes to `FetchContext::new` (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop; BC-2.16.015 EC-016-015-007 and TV-BC-2.16.015-006 are trace references — BC-2.16.015 is not in behavioral_contracts; governing contract is BC-2.16.002)

In `crates/prism-bin/src/spec_driven_adapter.rs`, `SpecDrivenSensorAdapter::fetch` maps
`params.limit: u64` to `early_stop_limit`:
```rust
let early_stop_limit = if params.limit == 0 { None } else { Some(params.limit as usize) };
```
This value is passed as the third argument to `FetchContext::new`. When no LIMIT clause is
present in the query, `params.limit` is 0 and `early_stop_limit = None` → full pagination
unchanged. Behavioral proof (BC-2.16.015 TV-BC-2.16.015-006): wiremock mock with
claroty-style page_size=1000, early_stop_limit=Some(1) → exactly 1 mock request issued;
`truncated = false`.

**Test:** `test_BC_2_16_002_early_stop_spec_driven_adapter_maps_params_limit_to_early_stop_limit`

### AC-006: All existing `FetchContext::new` callers pass `None` for `early_stop_limit`; no caller regressions (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop; ADR-060 §Consequences "all callers updated")

All non-production callers of `FetchContext::new` (the ~15 in-file pipeline.rs test sites +
~14 integration test files enumerated in Task 5) are updated to pass `None` as the third
argument. The workspace builds without error and all existing tests continue to pass.
Verification: `just check` exits 0.

**Test:** Compilation success (`just check` gate — a missed `FetchContext::new` caller produces a compile error, enforcing the sweep automatically; there is no single named compilation-sentinel test for this gate).

### AC-007: `ast_is_reducing_plan` returns `true` for all Condition A–J + conservative default inputs and `false` for bare-projection and ORDER-BY-only inputs; `run_materialization_pipeline` sets `fetch_limit = 0` for reducing plans and `options.limit` otherwise (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop Plan-Shape Gate, ADR-060 §D8.7)

A new function `ast_is_reducing_plan(ast: &Ast) -> bool` in
`crates/prism-query/src/materialization.rs` classifies the plan shape. The
`where_filters: &FilterMap` parameter present in v1.2 is REMOVED — the gate performs its own
AST inspection via `has_client_side_where`; `where_filters` continues to be computed and used
for push-down and cache key derivation but is NOT forwarded to the gate. It returns `true`
(early-stop suppressed, `fetch_limit = 0`) when ANY of the following conditions holds:
- **Condition A** (revised) — aggregation or window function in SELECT items or ORDER BY
  expressions, recursive into scalar-UDF args: detected via `expr_contains_aggregate_or_window`,
  which handles `FuncCall::Aggregate` (COUNT, SUM, AVG, MIN, MAX, COUNT DISTINCT),
  `FuncCall::Window` (window functions requiring full frame materialization), and recursion into
  `FuncCall::Scalar::args` (e.g., `severity_label(max(severity_id))` — closes F-R12-CRIT-001)
- **Condition B** — GROUP BY non-empty
- **Condition C** — `SelectClause::distinct = true` (SELECT DISTINCT)
- **Condition D** — HAVING clause present (`having.is_some()`)
- **Condition E** — `PipeStage::Stats` in pipe stages
- **Condition F** — `PipeStage::Dedup` in pipe stages
- **Condition G** (revised) — `has_client_side_where(&ast)` returns `true`: any WHERE-position
  predicate applied client-side by DataFusion post-fetch. Covers all four AST modes
  (`Ast::Filter`, `Ast::Sql`, `Ast::Pipe`, `Ast::SqlPipe`) and all non-temporal predicate forms
  (equality, CONTAINS, BETWEEN, IN-list, CIDR, Regex, Has, Missing, IsNull, Wildcard, logical
  combinations). Temporal-only predicates remain safe (pushed server-side via ADR-033 T1;
  `has_client_side_where` returns `false` for purely temporal predicates)
- **Condition H** (new; closes F-R12-HIGH-001) — SQL JOIN: `!sql.joins.is_empty()` or
  `!spq.head.joins.is_empty()`; applies to all JOIN kinds (Inner, Left, Right, FullOuter,
  Cross); early-stopping a join input truncates that input before DataFusion applies the JOIN
- **Condition I** (new) — Pipe Tail stage: any `PipeStage::Tail(_)` in `pipe.stages` or
  `spq.stages`; selecting last N rows requires seeing all rows; early-stop returns the tail
  of a truncated subset, not the true tail of the full dataset
- **Condition J** (new; defensive) — Pipe Join stage: any `PipeStage::Join(_)` in `pipe.stages`
  or `spq.stages`; currently errors at runtime (not yet supported, ENRICH-4-C); gate is
  future-proofed so that when Pipe Join is implemented, early-stop is already suppressed
- **Conservative Default** (new in v1.3) — unknown `Ast` variants, unknown `PipeStage` variants,
  unknown `FuncCall` variants all SUPPRESS (`_ => true` catch-all allowlist). PERMIT allow-list:
  bare projection, ORDER BY without aggregate in ORDER BY expressions (§D8.5), temporal-only
  WHERE, `PipeStage::Sort`, `PipeStage::Limit`, `PipeStage::Fields`, `PipeStage::Enrich`.
  Any shape not on this list suppresses.

It returns `false` for bare projections (`SELECT *` / `SELECT cols` with no reducing operator,
no client-side WHERE) and ORDER-BY-only queries (§D8.5: ORDER BY alone does NOT suppress
early-stop; `ORDER BY aggregate_fn(col)` WITHOUT GROUP BY DOES suppress via Condition A).

`run_materialization_pipeline` computes `fetch_limit` using the gate (immediately BEFORE fan-out
target construction; `where_filters` computed for push-down + cache key derivation but NOT
passed to the gate):
```rust
// Plan-shape gate (ADR-060 §D8.7): suppress early-stop for reducing plans.
// Note: where_filters is NOT passed — gate performs its own AST inspection.
let fetch_limit: u64 = if ast_is_reducing_plan(&ast) {
    0 // suppress: reducing plan needs full pagination for correctness
} else {
    options.limit.map(|l| l as u64).unwrap_or(0)
};
```
When `fetch_limit = 0`: `QueryParams.limit = 0` → `FetchContext::early_stop_limit = None` →
early-stop does NOT fire; full pagination to DI-019 10K cap (pre-story behavior).

**Tests:** RG-PSG-001 through RG-PSG-019 (see §Red Gate Tests).

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_002_early_stop_fetch_context_new_stores_early_stop_limit` | Unit — prism-spec-engine, `FetchContext::new` | AC-001: `FetchContext::new("id", HashMap::new(), Some(5))` stores `early_stop_limit = Some(5)`; `FetchContext::new("id", HashMap::new(), None)` stores `early_stop_limit = None`. Fails before `early_stop_limit` field is added. |
| RG-002 | `test_BC_2_16_002_early_stop_pipeline_stops_without_setting_truncated` | Integration — prism-spec-engine, wiremock multi-page mock (page_size=10); `early_stop_limit=Some(1)` | AC-002: 1 mock request issued; `PipelineResult.truncated = false`; 10 records returned (full page); DataFusion trims downstream. Fails before early-stop check is added to `execute_impl`. |
| RG-003 | `test_BC_2_16_002_early_stop_none_fetches_all_pages` | Integration — prism-spec-engine, wiremock (3 pages, page_size=10); `early_stop_limit=None` | AC-003 None case: all 3 pages fetched (3 mock requests); `truncated = false`; 30 records returned. Passes before and after (no early-stop when None). Regression sentinel. |
| RG-004 | `test_BC_2_16_002_early_stop_di019_fires_before_early_stop_check` | Unit — prism-spec-engine, pipeline internal; inject 10001 records on first page with `early_stop_limit=Some(5)` | AC-004: DI-019 check fires; `truncated = true`; records truncated to 10000. Fails if early-stop check is placed BEFORE DI-019 check (ordering validation). |
| RG-005 | `test_BC_2_16_002_early_stop_spec_driven_adapter_maps_params_limit_to_early_stop_limit` | Integration — prism-bin, wiremock claroty-style mock (page_size=1000, 1 record returned); `params.limit=1` | AC-005: `FetchContext` constructed with `early_stop_limit=Some(1)`; 1 mock request issued; `truncated=false`. Fails before AC-005 wiring. Also tests `params.limit=0 → None`. |
| RG-006 | `test_BC_2_16_002_early_stop_claroty_page_size_1000_limit_1_single_page` + `test_BC_2_16_002_early_stop_large_page_size_truncated_false` | Integration — prism-spec-engine or prism-bin, wiremock claroty-style (page_size=1000, 3 pages available, each 1000 records); `early_stop_limit=Some(1)` | BC-2.16.015 EC-016-015-007 / TV-BC-2.16.015-006: exactly 1 mock request issued (NOT 3); `truncated=false`; result has 1000 records pre-DataFusion-trim. This is the concrete claroty_vulnerabilities behavioral proof. `test_BC_2_16_002_early_stop_large_page_size_truncated_false` (PipelineExecutor layer, page_size=1000, asserts `!truncated`) explicitly discharges TV-BC-2.16.015-006's `truncated=false` promise at claroty scale. |

| RG-PSG-001 | `test_BC_2_16_002_plan_shape_gate_count_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter`; 3 pages × 100 rows (300 total), `options.limit=25` | AC-007 Condition A: AST with `COUNT(*)` aggregate → `ast_is_reducing_plan = true` → `fetch_limit = 0` → all 3 pages fetched (300 records); COUNT computed over full dataset (asserts COUNT=300). MUST FAIL before Task 11 (gate absent → `fetch_limit = 25` → early-stop fires after 1 page, COUNT computes over 100 records only). |
| RG-PSG-002 | `test_BC_2_16_002_plan_shape_gate_group_by_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition B: AST with GROUP BY only (no COUNT; GROUP-BY-ONLY to isolate Condition B) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched; group membership computed over full dataset. MUST FAIL before Task 11. |
| RG-PSG-003 | `test_BC_2_16_002_plan_shape_gate_distinct_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition C: AST with `SELECT DISTINCT col FROM t` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched; distinct values computed over full dataset. MUST FAIL before Task 11. |
| RG-PSG-004 | `test_BC_2_16_002_plan_shape_gate_non_temporal_where_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter`; equality WHERE predicate | AC-007 Condition G revised: `has_client_side_where` returns `true` for non-temporal equality predicate (`WHERE col = 'val'`) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched; DataFusion applies equality predicate client-side on full result. MUST FAIL before Task 11. |
| RG-PSG-005 | `test_BC_2_16_002_plan_shape_gate_pipe_stats_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition E: AST with `PipeStage::Stats` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 11. |
| RG-PSG-006 | `test_BC_2_16_002_plan_shape_gate_pipe_dedup_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition F: AST with `PipeStage::Dedup` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 11. |
| RG-PSG-007 | `test_BC_2_16_002_plan_shape_gate_bare_projection_early_stop_fires` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter`; bare `SELECT *`, `options.limit=5`, 3-page mock | POSITIVE CONTROL: no reducing operator → `ast_is_reducing_plan = false` → `fetch_limit = 5` → early-stop fires after `ceil(5/10) = 1` page; confirms gate does NOT over-suppress. MUST PASS after Task 11 (early-stop still fires for bare projections). |
| RG-PSG-008 | `test_BC_2_16_002_plan_shape_gate_order_by_limit_early_stop_fires` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter`; `ORDER BY col LIMIT N`, 3-page mock | POSITIVE CONTROL (§D8.5): ORDER BY alone is NOT a suppression condition → `ast_is_reducing_plan = false` → `fetch_limit = N` → early-stop fires; records in API-declared order within fetched subset. Confirms ORDER BY non-suppression (§D8.5 accepted limitation). MUST PASS after Task 11. |
| RG-PSG-009 | `test_BC_2_16_002_plan_shape_gate_having_suppresses_early_stop` | IN-CRATE UNIT on the gate (defense-in-depth, SAP-3 rule-3 reachability rationale; located in `materialization.rs` `#[cfg(test)] mod plan_shape_gate_unit_tests`) | AC-007 Condition D: calls `ast_is_reducing_plan` directly with AST for `GROUP BY col HAVING count(*) > N LIMIT 25`; asserts `ast_is_reducing_plan = true` → gate suppresses. HAVING path is reachable from the parser; unit test provides defense-in-depth isolation of Condition D. MUST FAIL before Task 11. |
| RG-PSG-010 | `test_BC_2_16_002_plan_shape_gate_nested_agg_in_scalar_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition A revised (F-R12-CRIT-001): aggregate nested inside scalar UDF arg (`severity_label(max(severity_id))`) — `expr_contains_aggregate_or_window` recurses into `FuncCall::Scalar::args` and detects inner aggregate → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 11 (without `FuncCall::Scalar::args` recursion, outer Scalar escapes Condition A → early-stop fires after 1 page). |
| RG-PSG-011 | `test_BC_2_16_002_plan_shape_gate_order_by_aggregate_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition A revised: aggregate in ORDER BY (`ORDER BY MAX(severity)` without GROUP BY) → `expr_contains_aggregate_or_window` applied to `OrderExpr` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 11 (ORDER BY expressions not scanned in v1.2). |
| RG-PSG-012 | `test_BC_2_16_002_plan_shape_gate_window_function_suppresses_early_stop` | IN-CRATE UNIT on the gate (defense-in-depth, SAP-3 rule-3 reachability rationale; located in `materialization.rs` `#[cfg(test)] mod plan_shape_gate_unit_tests`) | AC-007 Condition A revised: calls `ast_is_reducing_plan` directly with AST containing `FuncCall::Window` in SELECT → `expr_contains_aggregate_or_window` detects `FuncCall::Window` → `ast_is_reducing_plan = true`. Window functions require full frame materialization; early-stop severs the frame. MUST FAIL before Task 11 (`FuncCall::Window` not detected in v1.2). |
| RG-PSG-013 | `test_BC_2_16_002_plan_shape_gate_filter_mode_where_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition G revised: Filter-mode non-temporal predicate (`Ast::Filter` with severity equality) → `has_client_side_where` returns `true` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 11 (v1.2 `where_filters` always empty for `Ast::Filter` mode → Condition G INCORRECTLY PERMITTED early-stop, under-returning rows). |
| RG-PSG-014 | `test_BC_2_16_002_plan_shape_gate_pipe_where_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition G revised: Pipe-stage WHERE non-temporal predicate (`PipeStage::Where(severity = 'HIGH')`) → `has_client_side_where` iterates pipe stages → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched; asserts RAW filtered count (100 rows, gate suppressed, `fetch_limit = 0`) on `run_materialization_pipeline` output — materialization returns the FULL filtered set; tool-level cap + truncation signal (`is_truncated`/`total_available`) are engine.rs Step 6's responsibility, NOT materialization's (materialization MUST NOT apply a tool-level pre-cap). MUST FAIL before Task 11 (v1.2 `where_filters` always empty for `Ast::Pipe` stages → Condition G INCORRECTLY PERMITTED early-stop). |
| RG-PSG-015 | `test_BC_2_16_002_plan_shape_gate_non_equality_sql_where_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition G revised: non-equality SQL WHERE (`WHERE status LIKE '%page2%'` — LIKE predicate, non-equality SQL form; CONTAINS is a pipe StringOp/UDF, not a SQL predicate) → `has_client_side_where` returns `true` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 11 (v1.2 `where_filters` equality-only — non-equality predicates missed → early-stop INCORRECTLY PERMITTED, under-returning rows). Note: BC-2.16.002 EC-016-002-014 CONTAINS example remains valid (CONTAINS also suppresses via `has_client_side_where`); only this story RG row is aligned to the test's actual LIKE vehicle — no BC change needed. |
| RG-PSG-016 | `test_BC_2_16_002_plan_shape_gate_sql_join_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition H (closes F-R12-HIGH-001): SQL JOIN (`!sql.joins.is_empty()`) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; both inputs fully paginated to DI-019 cap. MUST FAIL before Task 11 (JOIN not a suppression condition in v1.2 → early-stop truncated join input). |
| RG-PSG-017 | `test_BC_2_16_002_plan_shape_gate_pipe_tail_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition I: Pipe Tail stage (`PipeStage::Tail(_)`) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; full pagination. `| tail N` selects last N rows — requires all rows; early-stop returns tail of truncated subset. MUST FAIL before Task 11. |
| RG-PSG-018 | `test_BC_2_16_002_plan_shape_gate_pipe_join_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition J (defensive): Pipe Join stage (`PipeStage::Join(_)`) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; full pagination. Note: Pipe Join currently errors at runtime (not yet supported, ENRICH-4-C); gate is defensive and future-proofed. MUST FAIL before Task 11. |
| RG-PSG-019 | `test_BC_2_16_002_plan_shape_gate_conservative_default_suppresses_early_stop` | IN-CRATE UNIT on the gate (defense-in-depth, SAP-3 rule-3 reachability rationale; located in `materialization.rs` `#[cfg(test)] mod plan_shape_gate_unit_tests`) | AC-007 Conservative Default: calls `ast_is_reducing_plan` directly with a synthetic `PipeStage` variant not in the PERMIT allow-list → pipe-stage loop catch-all `_ => return true` → `ast_is_reducing_plan = true`. Verifies the allowlist posture: unknown stage types SUPPRESS rather than PERMIT. MUST FAIL before Task 11 (no conservative default in v1.2 → unknown stage variant INCORRECTLY PERMITTED). |
| RG-PSG-020 | `test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter` | END-TO-END / Integration via `QueryEngine::execute` (in `crates/prism-query/tests/execute_integration_tests.rs`) | Truncation signal correctness: for a filter query whose WHERE clause suppresses early-stop (gate fires, `fetch_limit = 0`, full pagination), assert `is_truncated = true`, `total_available = 100` (true pre-cap count from materialization), `returned_results = 25` (tool-level cap applied by engine.rs Step 6). Verifies that materialization returns the FULL filtered set (100 rows) and engine.rs Step 6 is responsible for the cap + signal — a `truncate_result_to_limit` pre-cap in materialization would cause Step 6 to see only 25 rows as `total_available`, producing `is_truncated = false` (incorrect). MUST FAIL if materialization applies a tool-level pre-cap before returning to Step 6. |

**BC-5.38.001 density check:** 26 Red Gate tests (RG-001 through RG-006 + RG-PSG-001 through RG-PSG-020; RG-003, RG-PSG-007, RG-PSG-008 are regression/positive-control sentinels that pass in both states) / 7 acceptance criteria ≈ 3.71 ≥ 0.5 threshold. PASS.

**Note on RG-003 semantics:** RG-003 (`early_stop_limit=None` fetches all pages) passes BOTH before and after the implementation because `None` must preserve the current behavior. It is a regression gate confirming the existing full-pagination path is not broken.

**Note on RG-PSG-007 and RG-PSG-008 semantics (positive controls):** These pass before Task 11 is implemented because early-stop already works for bare projections. They MUST CONTINUE to pass after Task 11 — if they fail after the gate is added, the gate is over-suppressing. They gate against false negatives (gate incorrectly suppressing non-reducing plans). RG-PSG-009, RG-PSG-012, and RG-PSG-019 are in-crate unit tests that call `ast_is_reducing_plan` directly (defense-in-depth per SAP-3 rule-3; the corresponding paths are also reachable end-to-end but the unit tests provide faster, isolated gate verification).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `FetchContext` struct (field addition) | `crates/prism-spec-engine/src/pipeline.rs §FetchContext` | Pure (data struct; no I/O) |
| `FetchContext::new` (signature expansion) | `crates/prism-spec-engine/src/pipeline.rs §FetchContext::new` | Pure |
| Early-stop check in `execute_impl` | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute_impl` | Effectful (HTTP pagination loop; the check is a branch point within the loop) |
| `params.limit → early_stop_limit` mapping | `crates/prism-bin/src/spec_driven_adapter.rs §SpecDrivenSensorAdapter::fetch` | Effectful (production sensor adapter fetch path) |
| `ast_is_reducing_plan` predicate + `fetch_limit` gate | `crates/prism-query/src/materialization.rs §run_materialization_pipeline` | Pure (predicate; no I/O; evaluated before fan-out construction) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; FetchContext, PipelineExecutor)
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (prism-bin; spec_driven_adapter)
- `architecture/module-decomposition.md` §SS-11 Query Execution (prism-query; `ast_is_reducing_plan` plan-shape gate; `run_materialization_pipeline` `fetch_limit` derivation)
- `architecture/module-decomposition.md` §SS-07 Adapter Pagination & Response Cache (execute_impl per-page early-stop check §D8.2; `fetch_limit` coherence §D8.8)
- ADR-060 §D8 — FetchContext field, execute_impl check placement, truncated semantics, pagination modes
- ADR-060 §D8.7 v1.3 — Plan-Shape Gate: `ast_is_reducing_plan(&ast)` Conditions A–J + conservative default; `where_filters` NOT forwarded; enforcement in `materialization.rs §run_materialization_pipeline` before fan-out construction; temporal WHERE safety; ORDER BY non-suppression
- ADR-060 §D8.8 — Single-Binding Coherence: `fetch_limit` feeds both cache-key derivation and fan-out construction; gate preserves the invariant
- ADR-060 §Atomicity Reconciliation — why early-stop is compatible with "all-or-nothing" error-path invariant
- ADR-028 §D1 (OffsetLimit and CursorToken pagination configs)

---

## Purity Classification

| Element | Classification | Rationale |
|---------|---------------|-----------|
| `FetchContext` field addition (`early_stop_limit: Option<usize>`) | **Pure** | Plain data field on a value struct; no I/O. |
| `FetchContext::new` signature expansion | **Pure** | Value construction; no side effects. |
| Early-stop check in `PipelineExecutor::execute_impl` | **Pure decision inside an Effectful loop** | The `all_records.len() >= limit` comparison and `break 'steps` are pure control flow; they live within the effectful HTTP pagination loop and only reduce the number of effectful fetches performed. |
| `params.limit → early_stop_limit` mapping in `SpecDrivenSensorAdapter::fetch` | **Pure mapping on an Effectful path** | The `if params.limit == 0 { None } else { Some(...) }` mapping is pure; it feeds the effectful sensor-fetch path. |
| `ast_is_reducing_plan` + `fetch_limit` gate in `run_materialization_pipeline` | **Pure** | Predicate classification of the AST; no I/O. The `fetch_limit` u64 binding is a value derivation. Both are pure computations gating effectful fan-out construction. |

The pure-core / effectful-I/O boundary is respected: the early-stop policy is a pure predicate
threaded through `FetchContext` (data) and evaluated at a complete-page boundary; the only
effectful behavior change is fetching FEWER pages, never adding new I/O.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `early_stop_limit = Some(0)` | Check `all_records.len() >= 0` fires immediately (0 records always >= 0); loop exits after 0 records. **Coverage note: intentionally non-behavioral-tested.** `Some(0)` is unreachable in production: `params.limit == 0` maps to `None` at the adapter boundary (QueryParams sentinel), so `Some(0)` never reaches the pipeline. Covered at the constructor level by RG-001 (stores any `Option<usize>` value); no separate behavioral test warranted. |
| EC-002 | `early_stop_limit = Some(N)` where N > total available records | Early-stop never fires; pagination loop completes normally when API signals exhaustion (empty page or null cursor). `truncated = false`. **Test:** `test_BC_2_16_002_early_stop_limit_exceeds_total_fetches_all_pages` |
| EC-003 | `early_stop_limit = Some(N)` where N exactly equals `page_size` | Early-stop fires at end of first page (exactly N records). 1 page request issued. **Test:** `test_BC_2_16_002_early_stop_limit_equals_page_size_boundary` |
| EC-004 | DI-019 cap (10000) reached before `early_stop_limit` | DI-019 fires first; `truncated = true`; early-stop block NOT reached. Both checks present simultaneously; DI-019 order-precedence preserved. **Test:** `test_BC_2_16_002_early_stop_di019_fires_before_early_stop_check` (shared coverage with AC-004 / RG-004). |
| EC-005 | `PaginationConfig::None` (single-page fetch) with `early_stop_limit = Some(1)` | Pagination loop body executes once then breaks at `Some(PaginationConfig::None) \| None => break`. Early-stop check fires at end of the single page (after DI-019 check). **Coverage note: intentionally non-behavioral-tested.** This is a documented no-op: `PaginationConfig::None` exits the loop after one page regardless of `early_stop_limit`; there is no multi-page loop to terminate early. The early-stop check evaluates and breaks, but the loop was already about to exit — net behavior is identical to the `None` case. No separate behavioral test warranted; the single-page termination path is covered by existing `PaginationConfig::None` pipeline tests. |
| EC-006 | `ORDER BY` combined with `LIMIT` in query | DataFusion applies ORDER BY on the early-stopped result. Records are in API-declared order (not globally sorted top N). Documented in BC-2.16.002 D8.5 limitation text and in story §Background (no implementation impact). |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~8,000 |
| BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop section + §D8.7 plan-shape gate + EC-016-002-001..018 + §Atomicity Reconciliation clause (1 BC in behavioral_contracts) | ~2,500 |
| BC-2.16.015 EC-016-015-007, EC-016-015-008 + TV-BC-2.16.015-006 (trace reference — not in behavioral_contracts; relevant sections only) | ~500 |
| ADR-060 §D8 (full, including §D8.7 and §D8.8) | ~4,000 |
| `crates/prism-spec-engine/src/pipeline.rs` (FetchContext struct + execute_impl loop region) | ~4,000 |
| `crates/prism-bin/src/spec_driven_adapter.rs` (fetch function FetchContext::new call site region) | ~2,500 |
| `crates/prism-query/src/materialization.rs` (`fetch_limit` derivation + `ast_is_reducing_plan` gate + `run_materialization_pipeline`) | ~3,000 |
| ~14 integration test files (skimmed for FetchContext::new call sites; read only affected lines) | ~5,000 |
| ~15 in-file test sites (pipeline.rs #[cfg(test)] FetchContext::new calls) | ~3,000 |
| **Total estimate** | **~32,500 tokens** |

Well within 20-30% of a 200K context window. For the sibling sweep (Task 5), load each
integration test file targeted by reading only the `FetchContext::new` call site (grep-then-read
pattern) to minimize context consumption.

## Tasks

- [ ] **Task 1 (Red Gate — test first):** Write RG-001:
  `test_BC_2_16_002_early_stop_fetch_context_new_stores_early_stop_limit`
  in `crates/prism-spec-engine/src/pipeline.rs #[cfg(test)] mod tests` (or adjacent integration
  test file). Call `FetchContext::new(OrgSlug::new("org"), HashMap::new(), Some(5))` and assert
  `ctx.early_stop_limit == Some(5)`. Call with `None` and assert `None`. MUST FAIL before Task 6
  (field does not exist yet → compile error = fail).

- [ ] **Task 2 (Red Gate — test first):** Write RG-002:
  `test_BC_2_16_002_early_stop_pipeline_stops_without_setting_truncated`
  in `crates/prism-spec-engine/tests/bc_2_16_002_test.rs` (or new
  `tests/bc_2_16_002_early_stop_tests.rs`). Use wiremock multi-page mock: 3 pages available,
  page_size=10, each page returns 10 records. Build `FetchContext` with `early_stop_limit=Some(1)`.
  Execute pipeline. Assert: wiremock received exactly 1 request, `PipelineResult.truncated=false`,
  `PipelineResult.records.len() == 10` (one full page; DataFusion trims downstream).
  MUST FAIL before Task 7.

- [ ] **Task 3 (Red Gate — test first):** Write RG-003:
  `test_BC_2_16_002_early_stop_none_fetches_all_pages` in the same test file.
  Same 3-page mock; `early_stop_limit=None`. Assert 3 requests issued; 30 records;
  `truncated=false`. This PASSES before and after — it is a regression sentinel.

- [ ] **Task 4 (Red Gate — test first):** Write RG-004:
  `test_BC_2_16_002_early_stop_di019_fires_before_early_stop_check`
  in the same test file. Construct a mock that returns 10001 records on the first page
  (or a test helper that bypasses HTTP and directly calls the accumulation logic with
  a 10001-record vector). Set `early_stop_limit=Some(5)`. Assert `PipelineResult.truncated=true`.
  Assert `records.len() == MAX_PIPELINE_RECORDS` (10000). MUST FAIL if DI-019 check is removed
  or the early-stop check is incorrectly placed before DI-019.

- [ ] **Task 5 (TD-VSDD-060 sibling-sweep — MANDATORY before any implementation):**
  `FetchContext::new` is a public constructor whose signature is changing. Before implementing,
  run:
  ```
  rg 'FetchContext::new' crates/ --type rust
  ```
  and confirm ALL call sites are accounted for. The known callers are:
  - Production (1): `SpecDrivenSensorAdapter::fetch` in `crates/prism-bin/src/spec_driven_adapter.rs`
  - In-file pipeline.rs test sites (~15): all in `pipeline.rs #[cfg(test)]` blocks
    (search for `FetchContext::new` in pipeline.rs)
  - Integration tests (~14 files) listed in `crates_touched` frontmatter comment above

  UPDATE EVERY CALLER to pass `None` as the third argument before or during Task 6.
  Missing even one caller causes a compile error (the signature change is a breaking change).
  Document in the commit message: "TD-VSDD-060: FetchContext::new signature sweep — N callers
  updated to pass None."

  Additionally, verify `tests/external/non-exhaustive-violation/src/struct_violations.rs`:
  this file contains a doc-comment referencing `FetchContext`. It does NOT construct a
  `FetchContext` via struct literal (prevented by `#[non_exhaustive]`). Read the file to
  confirm no `FetchContext::new` call exists there; if one does, update it too.

- [ ] **Task 6 (Implementation — FetchContext field and constructor):** Add the
  `early_stop_limit: Option<usize>` field to `FetchContext` and expand `FetchContext::new`
  in `crates/prism-spec-engine/src/pipeline.rs`. Store the field in `Self`. Preserve
  `#[non_exhaustive]` and `#[derive(Debug, Clone)]`. After editing: run `cargo build -p prism-spec-engine`
  to confirm the crate compiles. Then run `just iter prism-spec-engine` — RG-001 MUST turn GREEN.
  All in-file test sites that were updated in Task 5 must compile.

- [ ] **Task 7 (Implementation — execute_impl early-stop check):** Insert the early-stop block
  in `PipelineExecutor::execute_impl` in `crates/prism-spec-engine/src/pipeline.rs` immediately
  after the DI-019 `MAX_PIPELINE_RECORDS` truncation block (the block ending in `truncated = true; break 'steps;`):
  ```rust
  // ADR-060 §D8.2: LIMIT-aware early-stop. Fires at COMPLETE page boundary, after DI-019.
  // truncated is NOT set — this is a success-path query-driven early exit, not a capacity overflow.
  if let Some(limit) = context.early_stop_limit {
      if all_records.len() >= limit {
          break 'steps;
      }
  }
  ```
  Applies to `OffsetLimit` and `CursorToken` pagination (the outer loop label `'steps:` covers
  both; `PaginationConfig::None` breaks naturally after one iteration). After editing: run
  `just iter prism-spec-engine` — RG-002 MUST turn GREEN; RG-003 MUST remain GREEN.

- [ ] **Task 8 (Red Gate — test first):** Write RG-005:
  `test_BC_2_16_002_early_stop_spec_driven_adapter_maps_params_limit_to_early_stop_limit`
  in `crates/prism-bin/tests/` or adjacent integration test file. Use wiremock mock with
  page_size=1000, 2 pages, `params.limit=1`. Assert mock received exactly 1 request.
  Assert `truncated=false`. Also test `params.limit=0 → None` (3 pages all fetched).
  MUST FAIL before Task 9 (spec_driven_adapter wiring not yet in place).

  Write RG-006:
  `test_BC_2_16_002_early_stop_claroty_page_size_1000_limit_1_single_page`
  in the same file. Use wiremock mock with `page_size=1000`, 3 pages of 1000 records each,
  `early_stop_limit=Some(1)`. Assert exactly 1 request issued. `truncated=false`.
  Records count = 1000 (first page; DataFusion trims to 1 downstream).
  This is the direct test vector for BC-2.16.015 TV-BC-2.16.015-006.

- [ ] **Task 9 (Implementation — spec_driven_adapter wiring):** In `SpecDrivenSensorAdapter::fetch`
  in `crates/prism-bin/src/spec_driven_adapter.rs`, insert immediately before
  `let context = FetchContext::new(...)`:
  ```rust
  // ADR-060 §D8.1: the query LIMIT is pre-extracted into QueryParams.limit (u64) before this call;
  // map 0 => None else Some(n) into FetchContext.early_stop_limit.
  // params.limit == 0 means "no LIMIT clause" (QueryParams convention); map to None → unchanged behavior.
  let early_stop_limit = if params.limit == 0 { None } else { Some(params.limit as usize) };
  ```
  Update the `FetchContext::new` call to:
  ```rust
  let context = FetchContext::new(self.sensor_spec.org_slug.clone(), query_filters, early_stop_limit);
  ```
  After editing: run `just iter prism-bin` — RG-005 MUST turn GREEN.

- [ ] **Task 10 (Red Gate — test first):** Write RG-PSG-001 through RG-PSG-019 in
  `crates/prism-query/tests/plan_shape_gate_tests.rs` (or extend `materialization_tests.rs` if that
  file exists). RG-PSG-009, RG-PSG-012, RG-PSG-019 are in-crate unit tests in
  `materialization.rs` `#[cfg(test)] mod plan_shape_gate_unit_tests`. All tests MUST be
  authored before Task 11.

  **Suppression tests (gate MUST fire; early-stop MUST be suppressed) — END-TO-END / Integration:**
  - RG-PSG-001 (`test_BC_2_16_002_plan_shape_gate_count_suppresses_early_stop`): in-process
    `PlanShapeGateMockAdapter`, 3 pages × 100 rows (300 total), `options.limit=25`. Assert
    `ast_is_reducing_plan = true` → `fetch_limit = 0` → all 3 pages fetched; COUNT=300.
    MUST FAIL before Task 11 (without gate, `fetch_limit = 25` → early-stop fires after 1 page).
  - RG-PSG-002 (`test_BC_2_16_002_plan_shape_gate_group_by_suppresses_early_stop`): GROUP-BY-ONLY
    (no COUNT) AST to isolate Condition B. Assert `ast_is_reducing_plan = true`; `fetch_limit = 0`.
    MUST FAIL before Task 11.
  - RG-PSG-003 (`test_BC_2_16_002_plan_shape_gate_distinct_suppresses_early_stop`): AST with
    `SELECT DISTINCT col FROM t`. Assert `ast_is_reducing_plan = true`; `fetch_limit = 0`.
    MUST FAIL before Task 11.
  - RG-PSG-004 (`test_BC_2_16_002_plan_shape_gate_non_temporal_where_suppresses_early_stop`):
    AST with equality WHERE predicate (`WHERE col = 'val'`); `has_client_side_where` returns `true`.
    Assert `ast_is_reducing_plan = true`; `fetch_limit = 0`. MUST FAIL before Task 11.
  - RG-PSG-005 (`test_BC_2_16_002_plan_shape_gate_pipe_stats_suppresses_early_stop`): AST with
    `PipeStage::Stats` (e.g., `| stats count()`). Assert `ast_is_reducing_plan = true`;
    `fetch_limit = 0`. MUST FAIL before Task 11.
  - RG-PSG-006 (`test_BC_2_16_002_plan_shape_gate_pipe_dedup_suppresses_early_stop`): AST with
    `PipeStage::Dedup` (e.g., `| dedup col`). Assert `ast_is_reducing_plan = true`;
    `fetch_limit = 0`. MUST FAIL before Task 11.
  - RG-PSG-010 (`test_BC_2_16_002_plan_shape_gate_nested_agg_in_scalar_suppresses_early_stop`):
    AST with aggregate nested inside scalar UDF arg (e.g., `severity_label(max(severity_id))`).
    Assert `ast_is_reducing_plan = true` (Condition A revised; `expr_contains_aggregate_or_window`
    recurses into `FuncCall::Scalar::args`). MUST FAIL before Task 11.
  - RG-PSG-011 (`test_BC_2_16_002_plan_shape_gate_order_by_aggregate_suppresses_early_stop`):
    AST with aggregate in ORDER BY (e.g., `ORDER BY MAX(severity)` without GROUP BY). Assert
    `ast_is_reducing_plan = true` (Condition A revised; `OrderExpr` scanned). MUST FAIL before Task 11.
  - RG-PSG-013 (`test_BC_2_16_002_plan_shape_gate_filter_mode_where_suppresses_early_stop`):
    `Ast::Filter` with non-temporal predicate. Assert `has_client_side_where = true`;
    `ast_is_reducing_plan = true`. MUST FAIL before Task 11 (v1.2 `where_filters` empty for Filter mode).
  - RG-PSG-014 (`test_BC_2_16_002_plan_shape_gate_pipe_where_suppresses_early_stop`):
    `Ast::Pipe` with `PipeStage::Where(non-temporal-pred)`. Assert `has_client_side_where = true`;
    `ast_is_reducing_plan = true`. MUST FAIL before Task 11 (v1.2 `where_filters` empty for Pipe stages).
  - RG-PSG-015 (`test_BC_2_16_002_plan_shape_gate_non_equality_sql_where_suppresses_early_stop`):
    SQL WHERE with LIKE predicate (`WHERE status LIKE '%page2%'` — non-equality SQL predicate form;
    CONTAINS is a pipe StringOp/UDF, not a SQL predicate). Assert `has_client_side_where = true`;
    `ast_is_reducing_plan = true`. MUST FAIL before Task 11 (v1.2 `where_filters` equality-only).
  - RG-PSG-016 (`test_BC_2_16_002_plan_shape_gate_sql_join_suppresses_early_stop`): AST with
    SQL JOIN (`!sql.joins.is_empty()`). Assert `ast_is_reducing_plan = true`. MUST FAIL before Task 11.
  - RG-PSG-017 (`test_BC_2_16_002_plan_shape_gate_pipe_tail_suppresses_early_stop`): AST with
    `PipeStage::Tail(_)` in pipe stages. Assert `ast_is_reducing_plan = true`. MUST FAIL before Task 11.
  - RG-PSG-018 (`test_BC_2_16_002_plan_shape_gate_pipe_join_suppresses_early_stop`): AST with
    `PipeStage::Join(_)` in pipe stages. Assert `ast_is_reducing_plan = true` (Condition J
    defensive; Pipe Join errors at runtime but gate must already suppress). MUST FAIL before Task 11.

  **Suppression tests — IN-CRATE UNIT (defense-in-depth; SAP-3 rule-3; in `materialization.rs` plan_shape_gate_unit_tests):**
  - RG-PSG-009 (`test_BC_2_16_002_plan_shape_gate_having_suppresses_early_stop`): call
    `ast_is_reducing_plan` directly with AST for `GROUP BY col HAVING count(*) > N`. Assert
    `ast_is_reducing_plan = true` (Condition D). MUST FAIL before Task 11.
  - RG-PSG-012 (`test_BC_2_16_002_plan_shape_gate_window_function_suppresses_early_stop`): call
    `ast_is_reducing_plan` directly with AST containing `FuncCall::Window` in SELECT. Assert
    `ast_is_reducing_plan = true` (Condition A revised: `FuncCall::Window` detected). MUST FAIL before Task 11.
  - RG-PSG-019 (`test_BC_2_16_002_plan_shape_gate_conservative_default_suppresses_early_stop`):
    call `ast_is_reducing_plan` directly with a synthetic `PipeStage` not in the PERMIT allow-list.
    Assert `ast_is_reducing_plan = true` (conservative default: `_ => true`). MUST FAIL before Task 11.

  **Positive controls (gate MUST NOT fire; early-stop MUST proceed):**
  - RG-PSG-007 (`test_BC_2_16_002_plan_shape_gate_bare_projection_early_stop_fires`): bare
    `SELECT * FROM t`, `options.limit=5`, 3-page mock (page_size=10). Assert
    `ast_is_reducing_plan = false`; `fetch_limit = 5`; early-stop fires after 1 page.
    MUST PASS before AND after Task 11 (confirms gate does NOT over-suppress).
  - RG-PSG-008 (`test_BC_2_16_002_plan_shape_gate_order_by_limit_early_stop_fires`): AST for
    `SELECT * FROM t ORDER BY col LIMIT N`. Assert `ast_is_reducing_plan = false` (§D8.5:
    ORDER BY alone is NOT a suppression condition); `fetch_limit = N`; early-stop fires.
    MUST PASS before AND after Task 11.

  **Truncation-signal correctness (END-TO-END / Integration via `QueryEngine::execute`):**
  - RG-PSG-020 (`test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter`) in
    `crates/prism-query/tests/execute_integration_tests.rs`. For a filter query whose WHERE
    clause suppresses early-stop (gate fires, `fetch_limit = 0`, full pagination, 100 rows match
    predicate), assert `is_truncated = true`, `total_available = 100` (true pre-cap count),
    `returned_results = 25` (tool-level cap applied by engine.rs Step 6). Verifies engine.rs
    Step 6 is responsible for the cap + signal; materialization MUST return the full set of
    100 rows WITHOUT applying a pre-cap. MUST FAIL if materialization applies a tool-level
    pre-cap before returning to Step 6 (pre-cap causes Step 6 to see 25 rows as `total_available`,
    producing `is_truncated = false` — incorrect). Write this test RED before implementing the
    engine.rs Step 6 fix; make it GREEN by ensuring materialization returns the full filtered
    set and Step 6 applies the cap and computes the truncation signal.

- [ ] **Task 11 (Implementation — `ast_is_reducing_plan` + `run_materialization_pipeline` gate):**
  In `crates/prism-query/src/materialization.rs`, add:

  ```rust
  // ADR-060 §D8.7 Condition A (revised v1.3): detects FuncCall::Aggregate, FuncCall::Window,
  // and recurses into FuncCall::Scalar::args to find nested aggregates.
  fn expr_contains_aggregate_or_window(expr: &Expr) -> bool {
      match expr {
          Expr::FuncCall(FuncCall::Aggregate { .. }) => true,
          Expr::FuncCall(FuncCall::Window { .. }) => true,
          Expr::FuncCall(FuncCall::Scalar { args, .. }) =>
              args.iter().any(|e| expr_contains_aggregate_or_window(e)),
          // ... recurse into Compare, Logical, Not, TimestampArithmetic; false for leaves
          _ => false,
      }
  }

  // ADR-060 §D8.7 v1.3: returns true (suppress early-stop, fetch_limit=0) for reducing plans.
  // where_filters is NOT a parameter — gate performs its own AST inspection via has_client_side_where.
  fn ast_is_reducing_plan(ast: &Ast) -> bool {
      // Condition A (revised): aggregation or window in SELECT/ORDER BY, recursive into Scalar args
      // Condition B: GROUP BY non-empty
      // Condition C: SELECT DISTINCT
      // Condition D: HAVING clause present
      // Condition E: PipeStage::Stats
      // Condition F: PipeStage::Dedup
      // Condition G (revised): has_client_side_where — covers all 4 AST modes + all non-temporal forms
      // Condition H: SQL JOIN (!sql.joins.is_empty())
      // Condition I: PipeStage::Tail
      // Condition J: PipeStage::Join (defensive)
      // Conservative default: _ => true for unknown Ast/PipeStage/FuncCall variants
      // ... full implementation per ADR-060 §D8.7 v1.3 Conditions A–J + conservative default
  }
  ```

  Update `fetch_limit` derivation in `run_materialization_pipeline` BEFORE fan-out target
  construction (per ADR-060 §D8.7; `where_filters` is NOT passed to the gate — the gate
  performs its own AST inspection; `where_filters` continues to be used for push-down +
  cache key per §D8.8 single-binding coherence):
  ```rust
  // Plan-shape gate (ADR-060 §D8.7): suppress early-stop for reducing plans.
  // Note: where_filters is NOT passed — gate performs its own AST inspection.
  let fetch_limit: u64 = if ast_is_reducing_plan(&ast) {
      0 // suppress: reducing plan needs full pagination for correctness
  } else {
      options.limit.map(|l| l as u64).unwrap_or(0)
  };
  ```
  The `0` sentinel flows unchanged: `QueryParams.limit = 0` → `FetchContext::early_stop_limit = None`
  (per existing `if params.limit == 0 { None }` mapping in `spec_driven_adapter.rs`).

  After editing: run `just iter prism-query` — RG-PSG-001 through RG-PSG-006, RG-PSG-009 through
  RG-PSG-019 MUST turn GREEN; RG-PSG-007 and RG-PSG-008 (positive controls) MUST remain GREEN.

- [ ] **Task 12 (Integration sweep — update all remaining callers):** Run `just check --no-fail-fast`
  across the full workspace. All integration test files listed in `crates_touched` that were
  updated in Task 5 should compile. If any callers were missed in Task 5, find them now via the
  compile errors and update each to pass `None`. Run `just iter prism-spec-engine` to confirm
  all pipeline.rs-adjacent tests pass. Run `just iter prism-bin` to confirm all prism-bin tests
  pass.

- [ ] **Task 13 (SAP-1 self-check):** Confirm that no new `tracing::*!(event_type = ...)` emissions
  are added. BC-2.16.002 SAP-1 declaration states: "ADR-060 introduces NO new `event_type`
  values; the existing `pipeline_truncated` WARN event (DI-019 cap only) is NOT altered; catalog
  count unchanged at 96." The early-stop branch and the plan-shape gate have no emissions —
  this is intentional and documented.

- [ ] **Task 14 (Final gate):** Run `just check` (full workspace). Confirm all non-`#[ignore]`
  Red Gate tests pass: RG-001, RG-002, RG-003, RG-004, RG-005, RG-006, RG-PSG-001 through
  RG-PSG-020. Confirm `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` does
  NOT need updating (no new `#[non_exhaustive]` type is introduced by the plan-shape gate —
  `ast_is_reducing_plan` and `expr_contains_aggregate_or_window` are private functions).
  Confirm no new `unwrap()`/`expect()` in production code paths. After `just check` passes,
  hold for story-level holdout gate before pushing to origin.

## Previous Story Intelligence

1. **S-DEMO-CLAROTY-PAGINATION-001 (merged):** Added POST-body OffsetLimit pagination for Claroty.
   This story builds on that pagination infrastructure — the `OffsetLimit` branch in
   `execute_impl` is the exact branch where the early-stop check fires.

2. **S-ADR058-OCSF-ROUTING-001 / S-ADR058-OCSF-COERCION-001 (merged):** These stories modified
   pipeline.rs. Read the current pipeline.rs to confirm the DI-019 check is at the DI-019
   `MAX_PIPELINE_RECORDS` truncation block before inserting the early-stop check. Use the
   `// AC-8 / DI-019` comment as the anchor (TD-VSDD-091).

3. **`FetchContext::new` call-site distribution (confirmed from codebase):** The in-file pipeline.rs
   test sites use the `default_context()` helper in the `pipeline.rs` `#[cfg(test)]` test module, which calls `FetchContext::new(OrgSlug::new("test-org"), HashMap::new())`. This helper must ALSO be updated to pass `None`. The integration
   tests that call `FetchContext::new` directly are enumerated in `crates_touched` above.

4. **Wiremock multi-page mock pattern:** Existing tests in `pipeline_http_integration.rs`,
   `ac_1_cursor_page_size_test.rs`, and `bc_2_16_002_test.rs` already use wiremock multi-page
   mocks. Use these as the template for RG-002, RG-003, RG-005, RG-006 test setup.
   The mock hit count assertion pattern: `mock_server.received(matcher).await` or equivalent
   wiremock assertion API.

5. **`params.limit` semantics:** `QueryParams.limit: u64` is set to `0` when no LIMIT clause
   is in the query. Non-zero means the user specified `| LIMIT N`. The `0 → None` mapping
   is the correct sentinel (already used for the CrowdStrike `query.limit` injection check at
   the same location in `fetch`).

## Architecture Compliance Rules

From ADR-060 §D8.2:
- The early-stop check MUST be placed IMMEDIATELY AFTER the DI-019 block, not before it.
  Placing it before would allow early-stop to fire before the 10K safety cap — violating
  the DI-019 precedence ordering.

From ADR-060 §D8.3:
- `truncated` MUST NOT be set in the early-stop block. `truncated` is semantically reserved
  for DI-019 capacity overflow ONLY. Setting it on early-stop would misclassify a normal
  LIMIT query as a data-cap overflow to callers and MCP users.

From ADR-060 §D8.4:
- Early-stop applies ONLY to `OffsetLimit` and `CursorToken` pagination modes. For
  `PaginationConfig::None`, the loop body naturally exits after one iteration. No special
  handling needed.

From ADR-060 §D8.1 note:
- `early_stop_limit` is DISTINCT from `query_filters["query.limit"]`. The latter is for
  injecting LIMIT into the sensor API request URL/body (e.g., CrowdStrike `DetectionListParams.limit`).
  Do NOT overload or conflate the two.

From CLAUDE.md §`#[non_exhaustive]` discipline:
- `FetchContext` already has `#[non_exhaustive]`. Adding a field does NOT require updating
  `EXPECTED_SYMBOLS` (no new type; existing type gets a new field). The compile-fail gate
  at `tests/external/non-exhaustive-violation/` tests struct-LITERAL construction, not field
  addition. No changes needed to the perimeter-violation test.

From ADR-060 §D8.7 (plan-shape gate enforcement):
- `ast_is_reducing_plan` MUST be evaluated BEFORE fan-out targets are constructed in
  `run_materialization_pipeline`. The gate performs its own AST inspection via
  `has_client_side_where`; it does NOT read `where_filters`. `where_filters` continues to be
  computed (for push-down and cache key derivation per §D8.8) but is NOT passed to the gate.
  Passing `where_filters` to the gate would be WRONG: it is equality-only and SQL-mode-only,
  and was the root cause of v1.2 under-detection for Filter-mode, Pipe-stage WHERE, and
  non-equality SQL predicates (ADR-060 §D8.7 Condition G revised, v1.3).

From ADR-060 §D8.7 Condition G revised (temporal-only WHERE safety):
- Temporal-only WHERE predicates flow server-side via ADR-033 T1 push-down.
  `has_client_side_where` returns `false` for purely temporal predicates (they match the
  `is_purely_temporal_predicate` accept condition). Early-stop fires normally for
  time-window-only filtered queries. This interaction MUST be preserved: do NOT treat
  temporal predicates as client-side for gate purposes.

From ADR-060 §D8.8 (single-binding coherence):
- The `fetch_limit` binding feeds BOTH the response-cache key derivation AND the fan-out
  target construction (`QueryParams.limit`). Do NOT create a split (`limit_for_cache` vs
  `limit_for_pipeline`) — the invariant that both consumers see the same value MUST be
  preserved. The gate modifies only the derivation of `fetch_limit`; the subsequent uses
  are unchanged.

From ADR-060 §D8.7 and §D8.8 (materialization result-cap responsibility boundary):
- `run_materialization_pipeline` MUST return the full filtered/aggregated result set to engine.rs
  Step 6. It MUST NOT apply a tool-level result cap (e.g., a `truncate_result_to_limit` pre-cap)
  before returning. The tool-level cap and truncation signal (`is_truncated`/`total_available`) are
  engine.rs Step 6's responsibility, NOT materialization's. Introducing a pre-cap in materialization
  causes Step 6 to compute `total_available` from a pre-capped subset rather than the true match
  count, silently producing `is_truncated = false` (incorrect) for queries whose unfiltered row
  count exceeds the tool limit. The `fetch_limit` binding controls ONLY the early-stop check in the
  pagination loop — it does NOT authorize materialization to cap the result set returned to Step 6.
  (Anchored: RG-PSG-020 `test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter`)

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `wiremock` | workspace Cargo.lock | Existing test dependency in prism-spec-engine; multi-page mock pattern |
| `serde_json` | workspace Cargo.lock | Mock response construction for pagination test pages |
| `tokio` | workspace Cargo.lock | Async test runtime |

No new Cargo.toml production dependencies. The `Option<usize>` field uses only stdlib types.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-spec-engine/src/pipeline.rs` | (a) Add `early_stop_limit` field to `FetchContext`; (b) expand `FetchContext::new` signature; (c) add early-stop check after DI-019 in `execute_impl`; (d) update ~15 in-file test sites (including `default_context()` helper) to pass `None` |
| MODIFY | `crates/prism-bin/src/spec_driven_adapter.rs` | Add `early_stop_limit` mapping and pass to `FetchContext::new` |
| MODIFY (×14) | Integration test files listed in `crates_touched` frontmatter comment | Update each `FetchContext::new` call to pass `None` as third arg |
| CREATE or EXTEND | `crates/prism-spec-engine/tests/bc_2_16_002_early_stop_tests.rs` OR extend `bc_2_16_002_test.rs` | RG-001, RG-002, RG-003, RG-004 |
| CREATE or EXTEND | `crates/prism-bin/tests/bc_2_16_002_early_stop_adapter_tests.rs` OR extend existing | RG-005, RG-006 |
| MODIFY | `crates/prism-query/src/materialization.rs` | (a) Add `expr_contains_aggregate_or_window(expr: &Expr) -> bool` helper (three-part: Aggregate variants, FuncCall::Window, recursion into FuncCall::Scalar::args); (b) add `ast_is_reducing_plan(ast: &Ast) -> bool` function (Conditions A–J + conservative default; `where_filters` NOT a parameter); (c) update `fetch_limit` derivation in `run_materialization_pipeline` to use plan-shape gate (before fan-out construction; `where_filters` NOT passed to gate) |
| CREATE or EXTEND | `crates/prism-query/tests/plan_shape_gate_tests.rs` OR extend `materialization_tests.rs` | RG-PSG-001 through RG-PSG-019; RG-PSG-009/012/019 are in-crate unit tests in `materialization.rs` `#[cfg(test)] mod plan_shape_gate_unit_tests` |
| MODIFY or EXTEND | `crates/prism-query/tests/execute_integration_tests.rs` | RG-PSG-020: `test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter` — END-TO-END truncation signal test via `QueryEngine::execute`; asserts `is_truncated=true` / `total_available=100` / `returned_results=25` |

Files that MUST NOT be modified:
- `tests/external/non-exhaustive-violation/src/struct_violations.rs` — read-only verification; no changes
- `scripts/check-non-exhaustive-per-symbol.py` — no new type; no EXPECTED_SYMBOLS update
- `crates/prism-dtu-*/` — DTU scope excluded from this story

## Forbidden Dependencies

`prism-spec-engine` MUST NOT gain a new production dependency on `prism-bin`. The
`early_stop_limit` field and the early-stop check live entirely within prism-spec-engine.
The prism-bin wiring (`params.limit` mapping) is the only prism-bin change — no new
prism-spec-engine import added to prism-bin.

---

## References

- BC-2.16.002 §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)" — governing postcondition
- BC-2.16.002 §Postconditions "Partial-record discard" atomicity-reconciliation scope clause
- BC-2.16.002 §Edge Cases EC-016-002-001..018 — per-condition suppression edge cases (Conditions A–J + conservative default + ORDER BY positive control)
- BC-2.16.015 EC-016-015-007 (trace reference — not in behavioral_contracts) — Claroty LIMIT 1 early-stop; UNAFFECTED by §D8.7 (bare projection, ast_is_reducing_plan=false)
- BC-2.16.015 EC-016-015-008 (trace reference) — COUNT suppresses early-stop via §D8.7 Condition A; full dataset fetched
- BC-2.16.015 TV-BC-2.16.015-006 (trace reference) — LIMIT 1 single-page test vector; promoted to active by S-CLAROTY-VULNS-001 merge per POL-14
- ADR-060 §D8 — FetchContext field, execute_impl check, truncated semantics, modes, ORDER BY, timeout deferral
- ADR-060 §D8.7 v1.3 — Plan-Shape Gate: `ast_is_reducing_plan(&ast)` Conditions A–J + conservative default; `where_filters` NOT forwarded to gate; enforcement in `run_materialization_pipeline` before fan-out construction; temporal WHERE safety via `has_client_side_where`; ORDER BY non-suppression
- ADR-060 §D8.8 — Single-Binding Coherence: `fetch_limit` feeds cache-key derivation and fan-out construction
- ADR-060 §Atomicity Reconciliation — "atomic" = error-path invariant; early-stop is compatible
- `crates/prism-spec-engine/src/pipeline.rs §FetchContext` — struct + constructor to modify
- `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute_impl` — DI-019 block to extend
- `crates/prism-bin/src/spec_driven_adapter.rs §SpecDrivenSensorAdapter::fetch` — production wiring point
- `crates/prism-query/src/materialization.rs §run_materialization_pipeline` — plan-shape gate + `fetch_limit` derivation site

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.11 | 2026-08-27 | story-writer | **Round-13 propagation: SS-07/SS-11 subsystem anchoring, truncation-signal remediation, RG-PSG-015 LIKE alignment.** (1) Frontmatter `subsystems:` updated from `[SS-01, SS-16]` to `[SS-01, SS-07, SS-11, SS-16]` per ADR-060 v1.4 `subsystems_affected` (F-R13-LENSC-HIGH-001); SS-11 and SS-07 justification comments added; §Architecture Mapping "Architecture section references" list extended with `module-decomposition.md §SS-11` (Query Execution: ast_is_reducing_plan plan-shape gate, run_materialization_pipeline fetch_limit derivation) and `§SS-07` (Adapter Pagination & Response Cache: execute_impl per-page early-stop check §D8.2, fetch_limit coherence §D8.8). (2) Truncation-signal remediation (F-R13-CRIT-001 / MED-001 / MED-002): RG-PSG-014 (pipe-WHERE) row updated to assert RAW filtered count (100 rows, gate suppressed, fetch_limit=0) on `run_materialization_pipeline` output — materialization returns FULL set; tool-level cap + signal are engine.rs Step 6's responsibility, NOT materialization's. RG-PSG-020 registered (`test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter`, END-TO-END via QueryEngine::execute, `crates/prism-query/tests/execute_integration_tests.rs`): asserts is_truncated=true / total_available=100 / returned_results=25 for filter query whose match count exceeds tool limit. Materialization-no-cap rule added to §Architecture Compliance Rules (anchored to RG-PSG-020). RG-PSG-020 added to §Tasks Task 10 (red-then-green) and Task 14 final gate list. (3) RG-PSG-015 LIKE alignment (F-LENSB-P13-004): test vehicle updated from CONTAINS/StringOp to `WHERE status LIKE '%page2%'` (LIKE predicate; CONTAINS is a pipe StringOp/UDF, not a SQL predicate) in §Red Gate Tests row and §Tasks Task 10; BC-2.16.002 EC-016-002-014 CONTAINS example is unchanged (CONTAINS also suppresses via `has_client_side_where`). (4) Density recomputed: 25→26 RGTs (RG-001..006 + RG-PSG-001..020) / 7 ACs ≈ 3.71. (5) input-hash comment updated: ADR-060 v1.3→v1.4. TD-VSDD-097: Dim-1 — ADR-060 v1.4 `subsystems_affected=[SS-01,SS-07,SS-11,SS-16]`; story subsystems now MATCH; CONFIRMED. Dim-2 — whole-artifact sweep: RG count 26 consistent in §Red Gate Tests table (26 rows), density paragraph (26/7≈3.71), Task 10 (RG-PSG-020 added), Task 14 (..020); subsystems `[SS-01,SS-07,SS-11,SS-16]` consistent in frontmatter and justification block and §Architecture Mapping refs; FULL. Dim-3 — RG-PSG-020 MUST anchored to `test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter` in §Red Gate Tests + §Architecture Compliance Rules + §Tasks; materialization-no-cap rule MUST anchored to RG-PSG-020; no unanchored MUSTs introduced; CLEAR. |
| 1.10 | 2026-08-27 | story-writer | **ADR-060 §D8.7 v1.3 + BC-2.16.002 v2.40 propagation (complete-plan-shape-gate spec changes).** (1) AC-007 signature change: `ast_is_reducing_plan(&ast, &where_filters)` → `ast_is_reducing_plan(&ast)` (`where_filters` parameter REMOVED; gate performs own AST inspection via `has_client_side_where`; `where_filters` still computed for push-down + cache key derivation but NOT passed to gate); `expr_contains_aggregate` → `expr_contains_aggregate_or_window` (three-part: Aggregate variants, FuncCall::Window, recursion into FuncCall::Scalar::args). (2) BC-2.16.002 pin row: v2.39→v2.40; EC range EC-016-002-001..007 → EC-016-002-001..018; condition description updated to "Conditions A–J + conservative default suppress early-stop; where_filters NOT forwarded to gate". (3) Over-claim fixes (round-12 lens-C MED-2): "Conditions A–G" → "Conditions A–J + conservative default" throughout (§Authority, AC-007, §Architecture Compliance Rules, §References, §File Structure Requirements, frontmatter bc comment); "EC-016-002-001..007" → "EC-016-002-001..018"; "RG-PSG-001 through RG-PSG-009" → "RG-PSG-001 through RG-PSG-019". (4) AC-007 coverage prose added for Conditions H (SQL JOIN), I (Pipe Tail), J (Pipe Join defensive), and Conservative Default allowlist posture. (5) RG-PSG-010..019 registered in §Red Gate Tests and §Tasks: RG-PSG-010 nested_agg_in_scalar, 011 order_by_aggregate, 012 window_function (IN-CRATE UNIT), 013 filter_mode_where, 014 pipe_where, 015 non_equality_sql_where, 016 sql_join, 017 pipe_tail, 018 pipe_join, 019 conservative_default (IN-CRATE UNIT). (6) Round-12 lens-C MED-1 fixes: test types corrected from "Unit" to "END-TO-END / Integration via run_materialization_pipeline, PlanShapeGateMockAdapter" for RG-PSG-001..008 (except 009→IN-CRATE UNIT); RG-PSG-001 params fixed to PlanShapeGateMockAdapter, 3 pages × 100 rows (300 total), LIMIT 25, asserts COUNT=300; RG-PSG-002 fixed to GROUP-BY-ONLY (no COUNT); RG-PSG-009 re-formed as IN-CRATE UNIT in materialization.rs plan_shape_gate_unit_tests. (7) BC-5.38.001 density: 15→25 Red Gate tests (RG-001..006 + RG-PSG-001..019) / 7 ACs ≈ 3.57. (8) input-hash: stale — inputs ADR-060 v1.3 + BC-2.16.002 v2.40; state-manager to recompute. TD-VSDD-097: Dim-1 — no named twin for this story; CLEAR. Dim-2 — RG count consistent across §Red Gate Tests table (25 rows), density paragraph (25/7≈3.57), Task 10 (..019), Task 11 (RG-PSG-001..006 + 009..019 green gate), Task 14 (..019), AC-007 Tests citation (..019); EC range EC-016-002-001..018 consistent in §Token Budget, §Behavioral Contracts table, §References, frontmatter bc comment; condition set "A–J + conservative default" consistent in §Authority, AC-007, §Architecture Compliance Rules, §References, §File Structure Requirements; FULL. Dim-3 — all RG-PSG-010..019 MUSTs anchored to S-ENGINE-LIMIT-EARLY-STOP-001 with named tests test_BC_2_16_002_plan_shape_gate_{name}_suppresses_early_stop; no unanchored MUSTs introduced; CLEAR. |
| 1.9 | 2026-08-26 | story-writer | RG-PSG-009 HAVING suppression registration (Condition D end-to-end coverage added by test-writer): `test_BC_2_16_002_plan_shape_gate_having_suppresses_early_stop` added as RG-PSG-009 in §Red Gate Tests; Task 10 RG-PSG authoring updated to include RG-PSG-009; Task 11 green-gate range updated (RG-PSG-001..RG-PSG-006 + RG-PSG-009); Task 14 final gate list updated; AC-007 Tests citation updated to RG-PSG-001..RG-PSG-009; BC-5.38.001 density check updated 14→15 RGTs (15/7 ≈ 2.14). TD-VSDD-097: Dim-1 — no named split-event twin; CLEAR. Dim-2 — RG count consistent across §Red Gate Tests table (15 rows), density paragraph (15), Task 10 (RG-PSG-009 added), Task 11 (range updated), Task 14 (range updated), AC-007 Tests citation (updated to ..009). Dim-3 — RG-PSG-009 MUST anchored to this story + `test_BC_2_16_002_plan_shape_gate_having_suppresses_early_stop`; no unanchored MUSTs. |
| 1.8 | 2026-08-26 | story-writer | **F-R11-CRIT-001 plan-shape gate (AC-007 + RG-PSG list + crates_touched prism-query + BC-2.16.002 v2.39 pin):** AC-007 added — `ast_is_reducing_plan` Conditions A–G; `run_materialization_pipeline` `fetch_limit` gate (ADR-060 §D8.7). RG-PSG-001..RG-PSG-008 added to §Red Gate Tests (6 suppression + 2 positive controls). `crates_touched` extended with `prism-query` (enforcement site: `materialization.rs §run_materialization_pipeline`). BC-2.16.002 §Behavioral Contracts table version v2.38→v2.39; plan-shape gate clause added to Role. `acceptance_criteria_count` 6→7; density check updated to 14/7 = 2.0. Tasks 10–11 added (RG-PSG authoring before implementation; `ast_is_reducing_plan` implementation); old Tasks 10–12 renumbered to Tasks 12–14. Token Budget updated: BC-2.16.002 section ~2,000→~2,500 (§D8.7 + EC-016-002-001..007); `prism-query/src/materialization.rs` row added (~3,000); story spec ~7,500→~8,000. **F-R11-OBS-001 BC-2.16.015 trace-only demotion:** BC-2.16.015 removed from `behavioral_contracts:` (trace-only in `traces_to:`). BC-status comment updated: promoted to active by S-CLAROTY-VULNS-001 merge per POL-14, not this story. BC-2.16.015 row removed from §Behavioral Contracts body table; added as trace reference in §References with EC-016-015-007/008 + TV-006. AC-005 BC-2.16.015 reference updated to "trace reference." **F-R11-LOW-001 AC-003 citation:** `test_BC_2_16_002_early_stop_multi_page_stops_after_second_page` added as AC-003 Test citation (k>1 proof). TD-VSDD-097: Dim-1 — no named split-event twin for this story; CLEAR. Dim-2 — cross-ref sweep: all task-ordinal cross-references updated (Task 10→12, Task 11→13, Task 12→14); "MUST FAIL before Task 9" and "MUST FAIL before Task 7" retain correct ordinals (Tasks 7 and 9 unchanged). Dim-3 — AC-007 RG-PSG MUSTs anchored to this story + named RG-PSG-001..008 tests; no unanchored MUSTs. |
| 1.7 | 2026-08-26 | story-writer | SAC-1 rule-3 task-ordering fix (F-R10-LOW-001): moved RG-005/RG-006 test authoring from old Task 10 to new Task 8, positioned before spec_driven_adapter wiring (new Task 9); restored "(Red Gate — test first)" label on Task 8; added "MUST FAIL before Task 9" clause; old Tasks 8 and 9 renumbered to Tasks 9 and 10 respectively; Tasks 11 and 12 unchanged. Dim-2 cross-reference sweep: no task-ordinal references outside §Tasks section required updating (all inter-task references in body use Task 5 and Task 6, both of which retain their ordinals). No AC, RG, EC, BC, or code content changed. |
| 1.6 | 2026-08-26 | story-writer | POL-7 title-sync fix (F-R7-MED-001): BC-2.16.015 §Behavioral Contracts Title cell corrected to verbatim H1 — appended "— Queryable Surface and OCSF vulnerability_finding Mapping" suffix. BC-2.16.002 Title cell confirmed verbatim-correct (unchanged). |
| 1.5 | 2026-08-26 | story-writer | Volatile-line-cite strip (TD-VSDD-091/L9): removed three numeric line-number cites in §Tasks Task 7 and §Previous Story Intelligence; replaced with symbol/section anchors (`MAX_PIPELINE_RECORDS` truncation block, `// AC-8 / DI-019` comment anchor, `pipeline.rs` `#[cfg(test)]` test module). |
| 1.4 | 2026-08-26 | story-writer | Fix governance lifecycle mislabel (F-R5-MED-001): BC-2.16.015 status label corrected from "active" to "draft" in frontmatter BC-status comment; POL-14 auto-promotion note added. Comprehensive audit: BC-2.16.002 active claim verified correct; no ADR status mislabels found in file body. |
| 1.3 | 2026-08-26 | story-writer | POL-39 anti-pin sweep (F-R3-MED-001): stripped numeric release pins for BC-2.16.002, BC-2.16.015, and ADR-060 from all narrative prose (~15 sites); updated BC-2.16.002 §Behavioral Contracts structural table Version column to current (was stale); BC-2.16.015 structural table pin verified current (unchanged). EC-004 test citation added per F-EARLYSTOP-P3-LOW-001 (shared coverage with AC-004/RG-004). |
| 1.2 | 2026-08-26 | story-writer | Comprehensive R2 adversarial sweep: (1) Task 8 prescribed comment rewritten to DELIVERED form per ADR-060 v1.1; (2) risk frontmatter comment updated to RESOLVED; (3) §Authority RESOLVED paragraph rewritten to remove quoted old phrasing; (4) assumption_validations claim reworded to remove quoted old phrasing; (5) AC-003 Some-case phantom test replaced with RG-002 test (test_BC_2_16_002_early_stop_pipeline_stops_without_setting_truncated); (6) AC-006 phantom compilation-sentinel test removed — compile gate only; (7) EC-001 and EC-005 annotated as intentionally non-behavioral-tested with rationale; (8) EC-002 and EC-003 cited with test-writer-added test names; (9) RG-006 augmented with test_BC_2_16_002_early_stop_large_page_size_truncated_false for TV-BC-2.16.015-006 truncated=false discharge. |
| 1.1 | 2026-08-26 | story-writer | §Authority sweep — ADR-060 v1.1 correction propagated; §D8.1 phrasing discrepancy marked RESOLVED; assumption_validations second entry updated to remove open-discrepancy framing (F-LENS3-OBS-001 closure). ACs, RG list, and tasks unchanged. |
| 1.0 | 2026-08-26 | story-writer | Initial authoring — ADR-060 §D8 implementation story. 6 ACs, 6 RGTs, density 1.0. SAC-1 compliant. TD-VSDD-060 sibling-sweep fully enumerated in Task 5 with all ~14 integration test file names. ADR-060 §D8.1 phrasing discrepancy noted in §Authority and Task 8. |
