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
version: "1.20"
modified: "2026-08-28"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.015-claroty-vulnerabilities-table.md"
  - ".factory/specs/architecture/decisions/ADR-060-limit-aware-early-stop-pagination.md"
  - ".factory/specs/architecture/decisions/ADR-061-multi-tenant-cache-key-isolation-authoritative-slug-resolution.md"
input-hash: "248f3c0"
# input-hash: updated 2026-08-27 (v1.17); ADR-061 v1.0 added (cache-key isolation); BC-2.16.002 v2.42 (query.org_slug_resolution_failure catalog row)
traces_to: ["BC-2.16.002", "BC-2.16.015", "BC-2.11.001"]
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
crates_touched: [prism-spec-engine, prism-bin, prism-query, prism-sensors, prism-core]
# crates_touched:
#   prism-query:
#     MODIFY src/materialization.rs:
#       (a) Add `ast_is_reducing_plan(ast: &Ast) -> bool` function
#       (b) Add `expr_contains_aggregate_or_window(expr: &Expr) -> bool` helper (three-part detection: Aggregate variants, FuncCall::Window, recursion into FuncCall::Scalar::args)
#       (c) Update `fetch_limit` derivation in `run_materialization_pipeline` to use plan-shape gate
#           (BEFORE fan-out target construction; where_filters computed for push-down + cache key
#            but NOT passed to gate) per ADR-060 §D8.7
#       (d) Add `pub any_early_stopped: bool` field to `MaterializationOutput`
#       (e) Pick up `any_early_stopped` from `FanOutResult` after fan-out completes
#       (f) DO NOT add heuristic `total_fetched_rows >= fetch_limit` — this is wrong on
#           multi-sensor fan-out (multiple sensors each return < fetch_limit but their sum
#           equals fetch_limit, none early-stopped → heuristic produces wrong is_truncated=true)
#     MODIFY src/engine.rs:
#       Step 6: change `let is_truncated = total_rows > limit;`
#           to `let is_truncated = total_rows > limit || materialization_output.any_early_stopped;`
#           (ADR-060 §D8.9 authoritative formula)
#   prism-sensors:
#     MODIFY src/adapter.rs:
#       (a) Define `pub struct FetchOutput { pub batches: Vec<RecordBatch>, pub any_early_stopped: bool }`
#       (b) Change `SensorAdapter::fetch` return type from `Result<Vec<RecordBatch>, SensorError>`
#           to `Result<FetchOutput, SensorError>` (all impl sites must update)
#     MODIFY src/fanout.rs:
#       (a) Add `pub any_early_stopped: bool` field to `FanOutResult`
#       (b) OR-aggregate `any_early_stopped` across all sensor results in `fan_out()`
#           (`any_early_stopped = results.iter().any(|r| r.any_early_stopped)`)
#   prism-spec-engine:
#     MODIFY src/pipeline.rs:
#       (a) Add `early_stop_limit: Option<usize>` field to `FetchContext` struct
#       (b) Add `early_stop_limit: Option<usize>` parameter to `FetchContext::new`
#       (c) Add `pub early_stopped: bool` field to `PipelineResult`
#       (d) Add early-stop check in `PipelineExecutor::execute_impl` loop (after DI-019 check)
#       (e) SET `early_stopped = true` BEFORE `break 'steps` in the §D8.2 early-stop block
#     Callers inside pipeline.rs #[cfg(test)] blocks: ~15 in-file test sites — all pass `None`
#   prism-bin:
#     MODIFY src/spec_driven_adapter.rs:
#       `SpecDrivenSensorAdapter::fetch` — map `params.limit` to `early_stop_limit`,
#       pass to `FetchContext::new`, and return
#       `FetchOutput { batches: result.batches, any_early_stopped: pipeline_result.early_stopped }`
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
#   SensorAdapter::fetch return-type sweep (all impl stubs must wrap batches as FetchOutput):
#     PRODUCTION (returns real early_stopped signal):
#       crates/prism-bin/src/spec_driven_adapter.rs — `FetchOutput { batches, any_early_stopped: pipeline_result.early_stopped }`
#     TEST STUBS (mechanical wrap — `any_early_stopped: false`):
#       crates/prism-bin/tests/boot_steps_7_8_tests.rs
#       crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs
#       crates/prism-mcp/src/server.rs
#       crates/prism-mcp/tests/bc_2_11_001_null_row_shape_test.rs
#       crates/prism-mcp/tests/bc_s_5_04_health_test.rs
#       crates/prism-mcp/tests/defect_t13_audit_ecode_sap3_test.rs
#       crates/prism-mcp/tests/normalized_pql.rs
#       crates/prism-mcp/tests/query_tool_sensor_errors_test.rs
#       crates/prism-query/src/materialization.rs (3 in-crate mock stubs)
#       crates/prism-query/src/tests/defect_csdevices_empty_memtable_tests.rs
#       crates/prism-query/tests/execute_integration_tests.rs
#       crates/prism-query/tests/filter_mode.rs
#       crates/prism-query/tests/pipe_execution_tests.rs
#       crates/prism-sensors/src/tests/bc_2_01_002.rs
#       crates/prism-sensors/src/tests/bc_2_01_010.rs
#       crates/prism-sensors/src/tests/bc_2_01_013.rs
#       crates/prism-sensors/src/tests/bc_2_01_013_sensorid.rs
#       crates/prism-sensors/tests/cr013_fan_out_org_id_consistency.rs
#       crates/prism-sensors/tests/multi_tenant_dtu_routing_integration.rs
#       crates/prism-sensors/tests/org_id_binding.rs
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
acceptance_criteria_count: 13
red_gate_tests: 41
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
**Read §D8.7 (Plan-Shape Gate v1.3) and §D8.9 (temporal-exemption soundness v1.5):**
`ast_is_reducing_plan(&ast)` function in `materialization.rs`; Conditions A–J + conservative default;
`where_filters` computed for push-down and cache key derivation but NOT forwarded to gate (gate performs
own AST inspection via `has_client_side_where(ast, datetime_index_cols)`); **v1.5 temporal-exemption
soundness**: `Ast::Filter` unconditionally suppressed; `PipeStage::Where` unconditionally suppressed;
`is_pushed_temporal_predicate` replaces `is_purely_temporal_predicate` for SQL/SqlPipe-head WHERE only
(requires range-op + INDEX datetime column + `Literal::Timestamp`); ORDER BY non-suppression;
gate application in `run_materialization_pipeline`. **Read §D8.3/§D8.9 (`any_early_stopped` propagation):**
`PipelineResult.early_stopped` field; `FetchOutput { batches, any_early_stopped }` return type;
propagation chain to engine Step 6; `is_truncated = (total_rows > limit) OR any_early_stopped`.
**Read §D8.8 (Single-Binding Coherence):** `fetch_limit` feeds both cache-key and fan-out target; gate
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
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | v2.42 | §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)": FetchContext field, execute_impl check placement, truncated=false semantics, applicable pagination modes, D8.5 ORDER BY limitation. Plan-Shape Gate (ADR-060 §D8.7 v1.3 / §D8.9 v1.5): Conditions A–J + conservative default suppress early-stop; where_filters NOT forwarded to gate; `fetch_limit=0` sentinel flow through `QueryParams.limit=0` → `FetchContext::early_stop_limit=None`; EC-016-002-001..018, EC-01-030..033 edge cases. Temporal-exemption soundness (§D8.9 v1.5): `Ast::Filter` unconditionally suppressed; `PipeStage::Where` unconditionally suppressed; `is_pushed_temporal_predicate` (range-op + INDEX datetime column + `Literal::Timestamp` RHS) replaces `is_purely_temporal_predicate` for SQL/SqlPipe-head WHERE. `PipelineResult.early_stopped: bool` + `FetchOutput { batches, any_early_stopped }` return-type contract; `any_early_stopped` propagation chain: FetchOutput → FanOutResult → MaterializationOutput → engine Step 6. Atomicity-reconciliation scope clause. |

*BC-2.16.002 v2.42 addendum (ADR-061 D8): Canonical Structured Event Catalog row 97 — `query.org_slug_resolution_failure` WARN added (two emission sites in `crates/prism-query/src/materialization.rs`: `resolve_source_refs` ALL-scope D5 arm and bare-filter Step 3b D4 arm). Catalog count 96→97; catalog label `(v1.70)` → `(v1.71)`. SAP-1 obligation: both `tracing::warn!` emission sites must appear in the same commit as the ADR-061 D2/D4/D5 fix (anchored to RG-SLUG-001, RG-SLUG-003).*

*BC-2.16.015 (trace-only — not in behavioral_contracts): EC-016-015-007 (LIMIT 1 early-stop, unaffected by §D8.7), EC-016-015-008 (COUNT suppresses early-stop), TV-BC-2.16.015-006. Core contract delivered by S-CLAROTY-VULNS-001; promoted to active on that story's merge per POL-14, not this story. See §References.*

*BC-2.11.001 (trace-only — not in behavioral_contracts): EC-11-092 (`any_early_stopped` feeds `is_truncated`; exact-limit boundary: `is_truncated = (total_rows > limit) OR any_early_stopped`; `total_available` is a LOWER BOUND when `any_early_stopped = true`), EC-11-093 (Step 6 as SOLE owner of tool-level cap; materialization returns full set without pre-cap; F-R13-CRIT-001 prohibited). Governing contract for the MCP tool response layer. See §References.*

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
- **Conservative Default** (new in v1.3) — the conservative-default posture applies at ALL
  dispatch levels, including the `Expr`-recursion level in `expr_contains_aggregate_or_window`:
  unknown `Ast` variants, unknown `PipeStage` variants, unknown `FuncCall` variants, and unknown
  `Expr` variants all SUPPRESS (`_ => true` catch-all at each level). Specifically,
  `expr_contains_aggregate_or_window` uses `_ => true` as its terminal arm (conservative SUPPRESS),
  with known non-aggregate leaf `Expr` variants (e.g., `Expr::Column`, `Expr::Literal`, comparison
  types) enumerated explicitly returning `false`; an unknown or future `Expr` variant (e.g., a CASE
  expression) is thereby treated as potentially-aggregate → SUPPRESS, preventing silent early-stop
  mis-permission. This is a defensive design-level property — no CASE variant exists today and this
  arm is not reachable by any existing test; no new RG is added for it. PERMIT allow-list for
  `ast_is_reducing_plan`: bare projection, ORDER BY without aggregate in ORDER BY expressions
  (§D8.5), temporal-only WHERE, `PipeStage::Sort`, `PipeStage::Limit`, `PipeStage::Fields`,
  `PipeStage::Enrich`. Any shape not on this list suppresses.

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

### AC-008: `has_client_side_where` temporal-exemption soundness — Filter-mode and Pipe-`|where` are unconditionally suppressed; SQL temporal PERMIT requires range-op + INDEX-datetime column + `Literal::Timestamp` RHS (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop Condition G revised v1.5, ADR-060 §D8.7/§D8.9)

`has_client_side_where(ast, datetime_index_cols)` implements temporal-exemption soundness per ADR-060 §D8.9:

(a) **Filter-mode unconditional suppression** (`Ast::Filter`): returns `true` UNCONDITIONALLY for all filter-mode predicates, including purely temporal ones. `extract_time_bounds_from_predicate` (ADR-033 T1) does NOT process `Ast::Filter` mode — temporal predicates in filter-mode queries are DataFusion client-side filters. The v1.3 `!is_purely_temporal_predicate` check for this arm was UNSOUND and is removed.

(b) **Pipe `| where` unconditional suppression** (`Ast::Pipe`, `Ast::SqlPipe` stages): returns `true` UNCONDITIONALLY whenever any `PipeStage::Where(_)` is present, regardless of predicate form. Pipe `| where` stages push NOTHING server-side; `PipeStage::Where` is REMOVED from the PERMIT allow-list.

(c) **SQL temporal PERMIT preconditions** (`Ast::Sql`, `Ast::SqlPipe` head WHERE only): `is_pushed_temporal_predicate(pred, datetime_index_cols)` returns `true` (PERMIT) iff ALL hold: range operator (Gt/Ge/Lt/Le — NOT Eq/Ne), LHS field in `datetime_index_cols` (INDEX datetime column), RHS `Expr::Literal(Literal::Timestamp)` (concrete absolute timestamp). Temporal equality, `Expr::Now`, `Expr::Interval`, relative expressions, and non-INDEX datetime columns SUPPRESS.

**Tests:** RG-PSG-021, RG-PSG-022, RG-PSG-023, RG-PSG-024, RG-PSG-029

### AC-009: `any_early_stopped` propagates through FetchOutput → engine Step 6; `is_truncated = (total_rows > limit) OR any_early_stopped`; Step 6 is the SOLE owner of tool-level cap (traces to BC-2.11.001 EC-11-092, EC-11-093; ADR-060 §D8.3/§D8.9)

(a) **`any_early_stopped` propagation**: When `execute_impl` fires the §D8.2 `break 'steps`, `PipelineResult.early_stopped = true` is set (DISTINCT from `truncated`). The early-stop signal propagates: `FetchOutput { batches, any_early_stopped: bool }` (new `SensorAdapter::fetch` return type per architect design) → `FanOutResult` → `MaterializationOutput` → engine.rs Step 6.

(b) **`is_truncated` formula at exact-limit boundary**: `is_truncated = (total_rows > limit) OR any_early_stopped`. When early-stop fires at the exact-limit boundary (`total_rows == limit`), `total_rows > limit` is false but `any_early_stopped` is true → `is_truncated: true` (correct). Without the `any_early_stopped` OR term, the response would incorrectly return `is_truncated: false` at the boundary, hiding that pagination was halted by the server.

(c) **Step 6 is the SOLE owner of tool-level cap**: `run_materialization_pipeline` MUST return the full filtered/aggregated result set to engine.rs Step 6 WITHOUT applying a tool-level pre-cap. Engine.rs Step 6 computes `total_available` from the full pre-cap count returned by materialization, then applies the cap. A pre-cap inside materialization causes Step 6 to see the pre-capped count as `total_available`, producing `is_truncated: false` (incorrect) when the unfiltered count exceeds the tool limit (F-R13-CRIT-001 prohibited behavior).

(d) **Wire-level MCP assertion (wire-shape discipline):** The `is_truncated` signal MUST be verifiable at the serialized JSON layer of the `prism_query` MCP tool response — not only at the pre-serialization Rust-struct layer (`QueryResult` is not `Serialize`; RG-PSG-025 covers only the struct layer). Two wire assertions are required (traces to BC-2.11.001 EC-11-092, EC-11-093):
- **Exact-limit (bare projection):** a `prism_query` MCP call with bare-projection `LIMIT N` where N equals the mock page_size (exact-limit boundary, `any_early_stopped = true`) returns `"is_truncated": true` in the serialized `CallToolResult` JSON, confirming the `any_early_stopped` OR term of EC-11-092 propagates through to the MCP wire.
- **Temporal-WHERE suppression:** a `prism_query` MCP call with a temporal `| where` predicate + `LIMIT N` at the exact boundary returns `"is_truncated": false` in the JSON (`ast_is_reducing_plan = true` via Condition G unconditional Pipe-WHERE suppression → early-stop disabled → `any_early_stopped = false` → formula reduces to `total_rows > limit = false`). Confirms suppression logic does not bleed through to produce false-positive `is_truncated: true` at the wire.

**Test:** RG-PSG-025 (Rust-struct layer), RG-PSG-026 (MCP wire layer)

### AC-010: `resolve_source_refs` ALL-scope — `org_registry: Some(reg)` + slug missing → fan-out target SKIPPED + `tracing::warn!(event_type = "query.org_slug_resolution_failure")`; `org_registry: None` → D3 synthetic slug used, target IS included (traces to BC-2.16.002 §`query.org_slug_resolution_failure` catalog row v2.42; ADR-061 D2, D3, D5)

The unified `let Some(client_slug) = org_registry.as_ref().and_then(...)` `else` branch in `resolve_source_refs` (`crates/prism-query/src/materialization.rs`) is replaced with a three-arm `match` dispatch per ADR-061 D5:

- **`Some(slug)` arm**: authoritative slug from `OrgRegistry` — D1 path. `FanOutTarget` constructed with this `client_id`. Cache-key derivation receives an authoritative `OrgSlug`.
- **`None if org_registry.is_some()` arm**: registry present, slug absent — D2 fail-closed path. No `FanOutTarget` pushed for this org; a structured `tracing::warn!` with `event_type = "query.org_slug_resolution_failure"` and `org_id = %org_id` is emitted; `continue` to next org. AD-017 tenant isolation is satisfied (no data served under wrong identity).
- **`None` arm** (D3 test/MVP mode): `org_registry` is entirely absent. Synthetic slug from deterministic prefix form used; `FanOutTarget` IS included.

The unified `else` branch that previously synthesized a slug for BOTH conditions (registry absent AND registry present but slug missing) is removed. `mat_ctx.org_registry` is already available at this callsite.

**Tests:** RG-SLUG-001 (D2 path: skip + warn), RG-SLUG-002 (D3 path: synthetic slug included)

### AC-011: Bare-filter fan-out Step 3b (`Ast::Filter` adapter loop) — same three-arm dispatch: `org_registry: Some(reg)` + slug missing → target NOT pushed, warn fired; `org_registry: None` → synthetic slug, target IS pushed (traces to BC-2.16.002 §`query.org_slug_resolution_failure` catalog row v2.42; ADR-061 D2, D3, D4)

The bare-filter `Ast::Filter` adapter loop in `crates/prism-query/src/materialization.rs` (bare-filter fan-out Step 3b) is updated per ADR-061 D4. The existing line that synthesized `OrgSlug::new(format!("org-{}", &org_id.to_string()[..8]))` **without ever consulting `mat_ctx.org_registry`** is replaced with the same three-arm dispatch pattern as AC-010. `mat_ctx.org_registry` is already available at this callsite via `pub(crate) org_registry` on `MaterializationContext` — no new field threading required.

The code comment at Site 1 claiming "no OrgRegistry available in bare-filter test path" is factually wrong in the current codebase and is removed as part of this fix.

**Tests:** RG-SLUG-003 (D2 path: skip + warn), RG-SLUG-004 (D3 path: synthetic slug included)

### AC-012: `"synthetic-unmapped"` sentinel ABSENT from all production code paths; D3 test-mode synthesis produces `org-{8hex}` OrgSlug valid by construction; `crates/prism-core/tests/org_slug_from_uuid_prefix.rs` deleted (traces to ADR-061 D3, D7)

The static sentinel `OrgSlug::new("synthetic-unmapped")` is removed unconditionally from all production (non-`#[cfg(test)]`) code paths in `crates/prism-query/src/materialization.rs`. It is superseded by the D3 deterministic-prefix form: `format!("org-{}", &org_id.to_string()[..8])`, which is valid by construction — the `"org-"` literal prefix guarantees ORG_SLUG_PATTERN (`^[a-zA-Z0-9_-]{1,64}$`) compliance regardless of the hex characters that follow. No digit-prefix special case is required: a leading digit in the hex slice is fully valid because ORG_SLUG_PATTERN permits digits in any position. This path fires only when `org_registry == None` (D3 test/MVP mode); production multi-tenant isolation is always provided by `OrgRegistry` slug resolution (D1 authoritative path or D2 fail-closed skip). Any defensive fallback branch (e.g., an `"org-x"` alternative when the first hex char is a digit) is unreachable by construction. The `"synthetic-unmapped"` constant collapses ALL orgs into a SINGLE shared cache partition — a total cross-tenant collapse risk with no valid production use case.

`crates/prism-core/tests/org_slug_from_uuid_prefix.rs` is deleted per ADR-061 D7. This test asserts the 8-hex synthesis pattern as correct behavior; leaving it would allow adversarial review to class-close the defect as "tested and valid."

**Test:** RG-SLUG-006 (`test_rg_slug_006_synthetic_unmapped_sentinel_absent`)

### AC-013: Wire-level cross-tenant isolation — two `OrgId`s with matching first-8-hex-char prefixes produce DISTINCT cache keys via `OrgRegistry`; serialized JSON for tenant B contains ZERO rows from tenant A (traces to ADR-061 D1, D9 RG-SLUG-005; CWE-284/CWE-340/OWASP A01 regression closed; BC-2.16.002 §`query.org_slug_resolution_failure` catalog row v2.42)

Two `OrgId` values engineered so that `org_id_a.to_string()[..8] == org_id_b.to_string()[..8]` (simulating concurrent onboarding within the same ~65-second UUIDv7 timestamp window) are registered in an `OrgRegistry` with DISTINCT slugs (`"tenant-alpha"` / `"tenant-beta"`). Each org has a distinct adapter seeded with a distinct provider value — adapter-A returns `"alpha-001"` rows, adapter-B returns `"beta-001"` rows.

A **single ALL-scope bare-filter query** (`clients: None`) is issued against a `QueryEngine` wired with an **EMPTY `ClientRegistry`** (no explicit-client slugs registered). The empty `ClientRegistry` forces `resolve_clients(None, empty)` → `[]` → the D4 bare-filter Step 3b fan-out path fires, enumerating all adapters from the `adapter_registry`. Assertion is on the **serialized JSON** of the `provider` column values collected across all result batches (wire-shape discipline, CLAUDE.md §Conventions):

- The serialized wire JSON **CONTAINS `"beta-001"`**, confirming adapter-B was fetched independently under a distinct cache key from adapter-A.

> **Why a populated `ClientRegistry` cannot exercise D4:** with slugs registered, `resolve_clients(None, registry)` returns those slugs as an explicit client list; `run_materialization_pipeline` then routes through `resolve_source_refs` (D5), which already consults `OrgRegistry` — bypassing Step 3b entirely. The described per-tenant explicit-client vehicle would false-green even with the D4 collision present. The empty `ClientRegistry` is required to force the D4 defect path.

This confirms ADR-061 D1 (cache-key identity invariant): after the D4 fix, `derive_response_cache_key` receives distinct authoritative `OrgSlug`s for each org, producing distinct cache partitions. Before the fix, Step 3b synthesized the same 8-hex slug `"org-deadbeef"` for both orgs — colliding them into one cache partition — so adapter-B's fetch was a cache HIT returning adapter-A's rows and `"beta-001"` was absent from the result.

**Test:** RG-SLUG-005 (`test_rg_slug_005_cross_tenant_wire_isolation_collision_resistant_cache_keys`)

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_002_early_stop_fetch_context_new_stores_early_stop_limit` | Unit — prism-spec-engine, `FetchContext::new` | AC-001: `FetchContext::new("id", HashMap::new(), Some(5))` stores `early_stop_limit = Some(5)`; `FetchContext::new("id", HashMap::new(), None)` stores `early_stop_limit = None`. Fails before `early_stop_limit` field is added. |
| RG-002 | `test_BC_2_16_002_early_stop_pipeline_stops_without_setting_truncated` | Integration — prism-spec-engine, wiremock multi-page mock (page_size=10); `early_stop_limit=Some(1)` | AC-002: 1 mock request issued; `PipelineResult.truncated = false`; 10 records returned (full page); DataFusion trims downstream. Fails before early-stop check is added to `execute_impl`. |
| RG-003 | `test_BC_2_16_002_early_stop_none_fetches_all_pages` | Integration — prism-spec-engine, wiremock (3 pages, page_size=10); `early_stop_limit=None` | AC-003 None case: all 3 pages fetched (3 mock requests); `truncated = false`; 30 records returned. Passes before and after (no early-stop when None). Regression sentinel. |
| RG-004 | `test_BC_2_16_002_early_stop_di019_fires_before_early_stop_check` | Unit — prism-spec-engine, pipeline internal; inject 10001 records on first page with `early_stop_limit=Some(5)` | AC-004: DI-019 check fires; `truncated = true`; records truncated to 10000. Fails if early-stop check is placed BEFORE DI-019 check (ordering validation). |
| RG-005 | `test_BC_2_16_002_early_stop_spec_driven_adapter_maps_params_limit_to_early_stop_limit` | Integration — prism-bin, wiremock claroty-style mock (page_size=1000, 1 record returned); `params.limit=1` | AC-005: `FetchContext` constructed with `early_stop_limit=Some(1)`; 1 mock request issued; `truncated=false`. Fails before AC-005 wiring. Also tests `params.limit=0 → None`. |
| RG-006 | `test_BC_2_16_002_early_stop_claroty_page_size_1000_limit_1_single_page` + `test_BC_2_16_002_early_stop_large_page_size_truncated_false` | Integration — prism-spec-engine or prism-bin, wiremock claroty-style (page_size=1000, 3 pages available, each 1000 records); `early_stop_limit=Some(1)` | BC-2.16.015 EC-016-015-007 / TV-BC-2.16.015-006: exactly 1 mock request issued (NOT 3); `truncated=false`; result has 1000 records pre-DataFusion-trim. This is the concrete claroty_vulnerabilities behavioral proof. `test_BC_2_16_002_early_stop_large_page_size_truncated_false` (PipelineExecutor layer, page_size=1000, asserts `!truncated`) explicitly discharges TV-BC-2.16.015-006's `truncated=false` promise at claroty scale. |

| RG-PSG-001 | `test_BC_2_16_002_plan_shape_gate_count_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter`; 3 pages × 100 rows (300 total), `options.limit=25` | AC-007 Condition A: AST with `COUNT(*)` aggregate → `ast_is_reducing_plan = true` → `fetch_limit = 0` → all 3 pages fetched (300 records); COUNT computed over full dataset (asserts COUNT=300). MUST FAIL before Task 12 (gate absent → `fetch_limit = 25` → early-stop fires after 1 page, COUNT computes over 100 records only). |
| RG-PSG-002 | `test_BC_2_16_002_plan_shape_gate_group_by_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition B: AST with GROUP BY only (no COUNT; GROUP-BY-ONLY to isolate Condition B) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched; group membership computed over full dataset. MUST FAIL before Task 12. |
| RG-PSG-003 | `test_BC_2_16_002_plan_shape_gate_distinct_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition C: AST with `SELECT DISTINCT col FROM t` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched; distinct values computed over full dataset. MUST FAIL before Task 12. |
| RG-PSG-004 | `test_BC_2_16_002_plan_shape_gate_non_temporal_where_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter`; equality WHERE predicate | AC-007 Condition G revised: `has_client_side_where` returns `true` for non-temporal equality predicate (`WHERE col = 'val'`) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched; DataFusion applies equality predicate client-side on full result. MUST FAIL before Task 12. |
| RG-PSG-005 | `test_BC_2_16_002_plan_shape_gate_pipe_stats_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition E: AST with `PipeStage::Stats` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 12. |
| RG-PSG-006 | `test_BC_2_16_002_plan_shape_gate_pipe_dedup_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition F: AST with `PipeStage::Dedup` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 12. |
| RG-PSG-007 | `test_BC_2_16_002_plan_shape_gate_bare_projection_early_stop_fires` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter`; bare `SELECT *`, `options.limit=5`, 3-page mock | POSITIVE CONTROL: no reducing operator → `ast_is_reducing_plan = false` → `fetch_limit = 5` → early-stop fires after `ceil(5/10) = 1` page; confirms gate does NOT over-suppress. MUST PASS after Task 12 (early-stop still fires for bare projections). |
| RG-PSG-008 | `test_BC_2_16_002_plan_shape_gate_order_by_limit_early_stop_fires` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter`; `ORDER BY col LIMIT N`, 3-page mock | POSITIVE CONTROL (§D8.5): ORDER BY alone is NOT a suppression condition → `ast_is_reducing_plan = false` → `fetch_limit = N` → early-stop fires; records in API-declared order within fetched subset. Confirms ORDER BY non-suppression (§D8.5 accepted limitation). MUST PASS after Task 12. |
| RG-PSG-009 | `test_BC_2_16_002_plan_shape_gate_having_suppresses_early_stop` | IN-CRATE UNIT on the gate (defense-in-depth, SAP-3 rule-3 reachability rationale; located in `materialization.rs` `#[cfg(test)] mod plan_shape_gate_unit_tests`) | AC-007 Condition D: calls `ast_is_reducing_plan` directly with AST for `GROUP BY col HAVING count(*) > N LIMIT 25`; asserts `ast_is_reducing_plan = true` → gate suppresses. HAVING path is reachable from the parser; unit test provides defense-in-depth isolation of Condition D. MUST FAIL before Task 12. |
| RG-PSG-010 | `test_BC_2_16_002_plan_shape_gate_nested_agg_in_scalar_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition A revised (F-R12-CRIT-001): aggregate nested inside scalar UDF arg (`severity_label(max(severity_id))`) — `expr_contains_aggregate_or_window` recurses into `FuncCall::Scalar::args` and detects inner aggregate → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 12 (without `FuncCall::Scalar::args` recursion, outer Scalar escapes Condition A → early-stop fires after 1 page). |
| RG-PSG-011 | `test_BC_2_16_002_plan_shape_gate_order_by_aggregate_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition A revised: aggregate in ORDER BY (`ORDER BY MAX(severity)` without GROUP BY) → `expr_contains_aggregate_or_window` applied to `OrderExpr` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 12 (ORDER BY expressions not scanned in v1.2). |
| RG-PSG-012 | `test_BC_2_16_002_plan_shape_gate_window_function_suppresses_early_stop` | IN-CRATE UNIT on the gate (defense-in-depth, SAP-3 rule-3 reachability rationale; located in `materialization.rs` `#[cfg(test)] mod plan_shape_gate_unit_tests`) | AC-007 Condition A revised: calls `ast_is_reducing_plan` directly with AST containing `FuncCall::Window` in SELECT → `expr_contains_aggregate_or_window` detects `FuncCall::Window` → `ast_is_reducing_plan = true`. Window functions require full frame materialization; early-stop severs the frame. MUST FAIL before Task 12 (`FuncCall::Window` not detected in v1.2). |
| RG-PSG-013 | `test_BC_2_16_002_plan_shape_gate_filter_mode_where_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition G revised: Filter-mode non-temporal predicate (`Ast::Filter` with severity equality) → `has_client_side_where` returns `true` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 12 (v1.2 `where_filters` always empty for `Ast::Filter` mode → Condition G INCORRECTLY PERMITTED early-stop, under-returning rows). |
| RG-PSG-014 | `test_BC_2_16_002_plan_shape_gate_pipe_where_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition G revised: Pipe-stage WHERE non-temporal predicate (`PipeStage::Where(severity = 'HIGH')`) → `has_client_side_where` iterates pipe stages → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched; asserts RAW filtered count (100 rows, gate suppressed, `fetch_limit = 0`) on `run_materialization_pipeline` output — materialization returns the FULL filtered set; tool-level cap + truncation signal (`is_truncated`/`total_available`) are engine.rs Step 6's responsibility, NOT materialization's (materialization MUST NOT apply a tool-level pre-cap). MUST FAIL before Task 12 (v1.2 `where_filters` always empty for `Ast::Pipe` stages → Condition G INCORRECTLY PERMITTED early-stop). |
| RG-PSG-015 | `test_BC_2_16_002_plan_shape_gate_non_equality_sql_where_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition G revised: non-equality SQL WHERE (`WHERE status LIKE '%page2%'` — LIKE predicate, non-equality SQL form; CONTAINS is a pipe StringOp/UDF, not a SQL predicate) → `has_client_side_where` returns `true` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL before Task 12 (v1.2 `where_filters` equality-only — non-equality predicates missed → early-stop INCORRECTLY PERMITTED, under-returning rows). Note: BC-2.16.002 EC-016-002-014 CONTAINS example remains valid (CONTAINS also suppresses via `has_client_side_where`); only this story RG row is aligned to the test's actual LIKE vehicle — no BC change needed. |
| RG-PSG-016 | `test_BC_2_16_002_plan_shape_gate_sql_join_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition H (closes F-R12-HIGH-001): SQL JOIN (`!sql.joins.is_empty()`) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; both inputs fully paginated to DI-019 cap. MUST FAIL before Task 12 (JOIN not a suppression condition in v1.2 → early-stop truncated join input). |
| RG-PSG-017 | `test_BC_2_16_002_plan_shape_gate_pipe_tail_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition I: Pipe Tail stage (`PipeStage::Tail(_)`) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; full pagination. `| tail N` selects last N rows — requires all rows; early-stop returns tail of truncated subset. MUST FAIL before Task 12. |
| RG-PSG-018 | `test_BC_2_16_002_plan_shape_gate_pipe_join_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-007 Condition J (defensive): Pipe Join stage (`PipeStage::Join(_)`) → `ast_is_reducing_plan = true` → `fetch_limit = 0`; full pagination. Note: Pipe Join currently errors at runtime (not yet supported, ENRICH-4-C); gate is defensive and future-proofed. MUST FAIL before Task 12. |
| RG-PSG-019 | `test_BC_2_16_002_plan_shape_gate_conservative_default_suppresses_early_stop` | IN-CRATE UNIT on the gate (defense-in-depth, SAP-3 rule-3 reachability rationale; located in `materialization.rs` `#[cfg(test)] mod plan_shape_gate_unit_tests`) | AC-007 Conservative Default: calls `ast_is_reducing_plan` directly with `PipeStage::Stats` (SUPPRESS — in the deny set) and `PipeStage::Sort` (PERMIT — in the allow set), verifying the PERMIT/SUPPRESS boundary of the allowlist. The conservative `_ => true` catch-all in the pipe-stage scan loop is structurally guaranteed by the allowlist posture and is NOT reachable by any test today — all `PipeStage` variants are enumerated in current grammar; the catch-all fires only for a future `#[non_exhaustive]` variant (SAP-3 rule-3 defense-in-depth; unreachable by current grammar). MUST FAIL before Task 12 (no conservative default in v1.2). |
| RG-PSG-020 | `test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter` | END-TO-END / Integration via `QueryEngine::execute` (in `crates/prism-query/tests/execute_integration_tests.rs`) | Truncation signal correctness: for a filter query whose WHERE clause suppresses early-stop (gate fires, `fetch_limit = 0`, full pagination), assert `is_truncated = true`, `total_available = 100` (true pre-cap count from materialization), `returned_results = 25` (tool-level cap applied by engine.rs Step 6). Verifies that materialization returns the FULL filtered set (100 rows) and engine.rs Step 6 is responsible for the cap + signal — a `truncate_result_to_limit` pre-cap in materialization would cause Step 6 to see only 25 rows as `total_available`, producing `is_truncated = false` (incorrect). MUST FAIL if materialization applies a tool-level pre-cap before returning to Step 6. |
| RG-PSG-021 | `test_psg_filter_mode_temporal_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-008(a) EC-01-030: Filter-mode temporal predicate (e.g., `timestamp >= '2024-01-01T00:00:00Z' AND timestamp < '2025-01-01T00:00:00Z'` in filter-mode query) — `Ast::Filter` → `has_client_side_where` returns `true` UNCONDITIONALLY → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL if Filter-mode temporal predicates are permitted (v1.3 unsound behavior: `!is_purely_temporal_predicate` = false → early-stop fires → under-returned rows). Located in `plan_shape_gate_tests.rs`. |
| RG-PSG-022 | `test_psg_pipe_where_temporal_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-008(b) EC-01-031: Pipe-mode `PipeStage::Where` with temporal predicate (e.g., `FROM table \| where timestamp >= '2024-01-01T00:00:00Z' AND timestamp < '2025-01-01T00:00:00Z' \| head 100`) — `Ast::Pipe` with `PipeStage::Where` → `has_client_side_where` returns `true` UNCONDITIONALLY → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL if Pipe-WHERE temporal predicates are permitted. `PipeStage::Where` removed from PERMIT allow-list. Located in `plan_shape_gate_tests.rs`. |
| RG-PSG-023 | `test_psg_sql_eq_temporal_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-008(c) EC-01-032: SQL temporal equality predicate (`WHERE timestamp = '2024-06-15T00:00:00Z'` — Eq operator, not a range op) — `is_pushed_temporal_predicate` returns `false` (Eq is NOT Gt/Ge/Lt/Le; `extract_time_bounds_from_predicate` does not extract temporal equality predicates) → `has_client_side_where = true` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL if temporal equality is treated as a server-side push-down. Located in `plan_shape_gate_tests.rs`. |
| RG-PSG-024 | `test_psg_sql_non_index_temporal_suppresses_early_stop` | END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter` | AC-008(c) EC-01-033: SQL temporal range on non-INDEX datetime column (`WHERE updated_at >= '2024-01-01T00:00:00Z'` where `updated_at` is NOT in `datetime_index_cols`) — `is_pushed_temporal_predicate` returns `false` (column not in `datetime_index_cols` — only columns declared `index: true` + `column_type = "Datetime"` in sensor TOML are eligible) → `has_client_side_where = true` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages fetched. MUST FAIL if non-INDEX temporal ranges are treated as pushed. Located in `plan_shape_gate_tests.rs`. |
| RG-PSG-025 | `test_psg_exact_limit_is_truncated_true` | END-TO-END / Integration via `QueryEngine::execute` (in `crates/prism-query/tests/execute_integration_tests.rs`) | AC-009(b) EC-11-092/EC-11-093: early-stop fires at the exact-limit boundary (`options.limit = 100`, 3-page mock where first page returns exactly 100 rows, `any_early_stopped = true`); `is_truncated = (total_rows > limit) OR any_early_stopped = true` even though `total_rows == limit` (so `total_rows > limit` is false). Without the `any_early_stopped` OR term, the response would return `is_truncated: false` at the boundary — silently hiding that pagination was halted. Also verifies Step 6 sole-owner (EC-11-093): `total_available = 100` (full pre-cap count; materialization did not pre-cap), `returned_results = 100` (cap = limit = 100). MUST FAIL before `any_early_stopped` propagation chain is implemented. |
| RG-PSG-026 | `test_psg_rg026_prism_query_wire_surfaces_truncation_signal` | MCP integration — prism-bin MCP tool tests (`crates/prism-bin/tests/mcp_integration_tests.rs` or equivalent MCP stdio test file; spawns prism start subprocess or uses in-process MCP dispatch with a DTU-style mock) | AC-009(d) EC-11-092/EC-11-093 wire-level: issues two `prism_query` MCP calls and asserts on the SERIALIZED `CallToolResult.content[0].text` JSON — (1) bare-projection `LIMIT N` where N equals mock page_size (exact-limit boundary, `any_early_stopped = true`): asserts `"is_truncated": true` in wire JSON; (2) temporal `\| where` + `LIMIT N` at same exact boundary (gate Condition G unconditional Pipe-WHERE suppression → early-stop disabled → `any_early_stopped = false` → `total_rows == limit` → `total_rows > limit` false): asserts `"is_truncated": false` in wire JSON. Wire-shape discipline (CLAUDE.md 2026-07-13): any test covering an MCP-visible surface MUST assert on the serialized JSON the LLM agent consumes. RG-PSG-025 covers only the Rust-struct layer (`QueryResult` is not `Serialize`); this test closes the MCP wire gap. MUST FAIL before `any_early_stopped` propagation chain is implemented AND before the `prism_query` tool response correctly serializes `is_truncated` from the `(total_rows > limit) OR any_early_stopped` formula. |

| RG-PSG-027 | `test_psg_multi_sensor_fanout_exact_limit_one_early_stopped_is_truncated_true` | END-TO-END / Integration via `QueryEngine::execute` (in `crates/prism-query/tests/execute_integration_tests.rs`) | AC-009(a) multi-sensor OR-aggregation: 2-sensor fan-out, `options.limit = 50`; sensor1 returns 40 rows (no early-stop); sensor2 returns 10 rows and early-stops at the page boundary; `FanOutResult.any_early_stopped = true` (OR-aggregate of sensor1.any_early_stopped=false OR sensor2.any_early_stopped=true); `total_rows = 50 = limit`; Step 6: `is_truncated = (50 > 50) || true = true`. MUST FAIL before `FetchOutput.any_early_stopped` propagation chain is implemented (any_early_stopped will be false if chain is missing → is_truncated=false wrongly). Gates the OR-aggregation path through fanout.rs → MaterializationOutput → engine.rs Step 6 on a multi-sensor exact-limit boundary. |
| RG-PSG-028 | `test_psg_multi_sensor_fanout_exact_total_no_early_stop_is_not_truncated` | END-TO-END / Integration via `QueryEngine::execute` (`crates/prism-query/tests/execute_integration_tests.rs`) + wire-level MCP assertion (`crates/prism-bin/tests/mcp_integration_tests.rs`) | AC-009(a) heuristic-rejection gate: 2-sensor fan-out, `options.limit = 50`; sensor1 returns 25 rows (no early-stop); sensor2 returns 25 rows (no early-stop); `FanOutResult.any_early_stopped = false`; `total_rows = 50 = limit`; Step 6: `is_truncated = (50 > 50) || false = false`. MUST FAIL with the heuristic `total_fetched_rows >= fetch_limit` (50 >= 50 = true → wrong is_truncated=true). MUST PASS only with the correct `any_early_stopped` OR term. Wire-level assertion in `crates/prism-bin/tests/mcp_integration_tests.rs` asserts `"is_truncated": false` in the serialized `CallToolResult.content[0].text` JSON (wire-shape discipline, CLAUDE.md 2026-07-13). This is the canonical multi-sensor heuristic-rejection test: the heuristic gives the wrong answer; the correct chain gives the right answer. |
| RG-PSG-029 | `test_psg_relative_temporal_now_interval_suppresses_early_stop` | END-TO-END via `QueryEngine::execute` from a real SQL string (`crates/prism-query/tests/plan_shape_gate_tests.rs`) | AC-008(c) regression guard — folded relative-temporal path (F-R16-P1-CRIT-001): SQL `WHERE <non_index_datetime_col> >= now() - interval '7d' LIMIT N`. Real mechanism: `inject_now` folds `now() - interval '7d'` to `Expr::Literal(Literal::Timestamp)` BEFORE `ast_is_reducing_plan` is called; the plan-shape gate never sees `Expr::Now`/`Expr::Interval`/`Expr::TimestampArithmetic`. After folding, `is_pushed_temporal_predicate` receives a concrete absolute timestamp RHS but returns `false` because the column is NOT in `datetime_index_cols` (non-INDEX datetime column; precondition (b) fails — same suppression mechanism as RG-PSG-024) → `has_client_side_where = true` → `ast_is_reducing_plan = true` → `fetch_limit = 0` → early-stop SUPPRESSED. Retained as regression guard overlapping RG-PSG-024; unique value: exercises the `inject_now` fold path. MUST FAIL against current code (which PERMITs via `is_temporal_expr` inspecting the pre-fold AST, before `inject_now` runs). Located in `crates/prism-query/tests/plan_shape_gate_tests.rs`. |

| RG-SLUG-001 | `test_rg_slug_001_resolve_source_refs_registry_present_slug_missing_skips_target_emits_warn` | Unit — prism-query, `resolve_source_refs` with `org_registry: Some(reg)` populated but no slug for test org_id (`crates/prism-query/tests/slug_isolation_tests.rs` or in-crate unit test) | AC-010 D2 path: after `resolve_source_refs` executes, the target list does NOT contain a `FanOutTarget` for the unmapped org_id; a `tracing::warn!` with `event_type = "query.org_slug_resolution_failure"` and `org_id = %unmapped_org_id` was captured. MUST FAIL before Task 18 (D5 fix not applied — unified `else` branch still synthesizes a slug and pushes the target). |
| RG-SLUG-002 | `test_rg_slug_002_resolve_source_refs_registry_absent_synthetic_slug_included` | Unit — prism-query, `resolve_source_refs` with `org_registry: None` (no registry injected — D3 test mode) | AC-010 D3 path: synthetic slug is generated from org_id prefix; `FanOutTarget` IS included in the result list. Regression sentinel — PASSES both before and after Task 18 (D3 preservation must not be broken). |
| RG-SLUG-003 | `test_rg_slug_003_bare_filter_step3b_registry_present_slug_missing_skips_target_emits_warn` | Unit — prism-query, bare-filter Step 3b `Ast::Filter` adapter loop with `org_registry: Some(reg)` but no slug for test org_id | AC-011 D2 path: `FanOutTarget` NOT pushed in the bare-filter adapter loop for the unmapped org; a `tracing::warn!` with `event_type = "query.org_slug_resolution_failure"` captured. MUST FAIL before Task 18 (D4 fix not applied — bare-filter Step 3b never consulted `mat_ctx.org_registry`, so the unmapped org gets a synthetic `client_id` and a `FanOutTarget` IS pushed). |
| RG-SLUG-004 | `test_rg_slug_004_bare_filter_step3b_registry_absent_synthetic_slug_included` | Unit — prism-query, bare-filter Step 3b `Ast::Filter` adapter loop with `org_registry: None` (D3 test mode) | AC-011 D3 path: synthetic slug generated; `FanOutTarget` IS pushed. Regression sentinel — PASSES both before and after Task 18. |
| RG-SLUG-005 | `test_rg_slug_005_cross_tenant_wire_isolation_collision_resistant_cache_keys` | END-TO-END — `QueryEngine::execute` in `crates/prism-query/tests/execute_integration_tests.rs`; two `OrgId`s constructed so their first-8-hex-char prefix values are identical (`"deadbeef"`); both registered in a shared `OrgRegistry` with DISTINCT slugs (`"tenant-alpha"` / `"tenant-beta"`); adapter-A seeded with `"alpha-001"` rows, adapter-B with `"beta-001"` rows; **SINGLE ALL-scope bare-filter query** (`clients: None`) with an **EMPTY `ClientRegistry`** — forces `resolve_clients(None, empty)` → `[]` → D4 bare-filter Step 3b fan-out fires; WIRE-LEVEL assertion on serialized JSON of collected `provider` column values. (A populated `ClientRegistry` would route through D5 `resolve_source_refs`, bypassing Step 3b and making the test vacuously pass.) | AC-013: serialized wire JSON CONTAINS `"beta-001"` — both tenants' adapter rows present, proving distinct cache keys survive the first-8-hex-char UUID prefix collision. Before the fix: Step 3b synthesizes slug `"org-deadbeef"` for both org_ids → same `CacheKey` partition → adapter-B cache HIT returns adapter-A's rows → `"beta-001"` ABSENT → FAILS. After fix: distinct slugs `"tenant-alpha"` / `"tenant-beta"` from `OrgRegistry` → distinct cache keys → both adapters fetched → `"beta-001"` PRESENT → PASSES. Confirms ADR-061 D1 cache-key identity invariant. Wire-shape discipline (CLAUDE.md §Conventions): assertion on serialized bytes, not pre-serialization Rust structs. MUST FAIL before Task 18 (D4 Step 3b fix not applied). |
| RG-SLUG-006 | `test_rg_slug_006_synthetic_unmapped_sentinel_absent` | `crates/prism-query/tests/slug_isolation_tests.rs` (EXTERNAL — in-crate placement avoided: an in-crate `include_str!` test would include the test file's own source in the scan, false-passing after production removal because the test body contains the string `"synthetic-unmapped"`); reads `crates/prism-query/src/materialization.rs` via `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/materialization.rs"))` and asserts that the string literal `"synthetic-unmapped"` is absent from the production source | AC-012: `"synthetic-unmapped"` sentinel removed. MUST FAIL before Task 19 (Site 3 sentinel not yet removed). Green after Task 19. |

**BC-5.38.001 density check:** 41 Red Gate tests (RG-001 through RG-006 + RG-PSG-001 through RG-PSG-029 + RG-SLUG-001 through RG-SLUG-006; RG-003, RG-PSG-007, RG-PSG-008, RG-SLUG-002, RG-SLUG-004 are regression/positive-control sentinels that pass in both states) / 13 acceptance criteria ≈ 3.15 ≥ 0.5 threshold. PASS.

**Note on RG-003 semantics:** RG-003 (`early_stop_limit=None` fetches all pages) passes BOTH before and after the implementation because `None` must preserve the current behavior. It is a regression gate confirming the existing full-pagination path is not broken.

**Note on RG-PSG-007 and RG-PSG-008 semantics (positive controls):** These pass before Task 12 is implemented because early-stop already works for bare projections. They MUST CONTINUE to pass after Task 12 — if they fail after the gate is added, the gate is over-suppressing. They gate against false negatives (gate incorrectly suppressing non-reducing plans). RG-PSG-009 (HAVING), RG-PSG-012 (window function), and RG-PSG-019 (PERMIT/SUPPRESS boundary) are in-crate unit tests calling `ast_is_reducing_plan` directly (SAP-3 rule-3 defense-in-depth). RG-PSG-009's HAVING path is reachable end-to-end (grammar-expressible via `GROUP BY col HAVING count(*) > N`). RG-PSG-012 (window-function stub; S-3.06 future grammar path) and RG-PSG-019 (the conservative `_ => true` catch-all, only reachable by a future `#[non_exhaustive]` variant) are NOT reachable end-to-end by current grammar — the unit tests provide isolated gate verification for these defense-in-depth paths only.

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
| This story spec | ~10,500 |
| BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop section + §D8.7/§D8.9 plan-shape gate + EC-016-002-001..018 + EC-01-030..033 + §Atomicity Reconciliation clause (1 BC in behavioral_contracts) | ~3,000 |
| BC-2.11.001 EC-11-092, EC-11-093 (trace reference — not in behavioral_contracts; relevant sections only) | ~500 |
| BC-2.16.015 EC-016-015-007, EC-016-015-008 + TV-BC-2.16.015-006 (trace reference — not in behavioral_contracts; relevant sections only) | ~500 |
| ADR-060 §D8 (full, including §D8.7, §D8.8, §D8.9) | ~4,500 |
| `crates/prism-spec-engine/src/pipeline.rs` (FetchContext struct + execute_impl loop region) | ~4,000 |
| `crates/prism-bin/src/spec_driven_adapter.rs` (fetch function FetchContext::new call site region) | ~2,500 |
| `crates/prism-query/src/materialization.rs` (`fetch_limit` derivation + `ast_is_reducing_plan` gate + `run_materialization_pipeline`) | ~3,000 |
| ~14 integration test files (skimmed for FetchContext::new call sites; read only affected lines) | ~5,000 |
| ~15 in-file test sites (pipeline.rs #[cfg(test)] FetchContext::new calls) | ~3,000 |
| `crates/prism-bin/tests/mcp_integration_tests.rs` (MCP wire test region; RG-PSG-026 — read only the relevant `prism_query` tool test area) | ~1,500 |
| **Total estimate** | **~34,500 tokens** |

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

- [ ] **Task 7 (Implementation — execute_impl early-stop check + PipelineResult.early_stopped field):**
  First, add `pub early_stopped: bool` to `PipelineResult` in `crates/prism-spec-engine/src/pipeline.rs`
  and initialize `let mut early_stopped = false;` before the `'steps:` loop in `execute_impl`.
  Then insert the early-stop block immediately after the DI-019 `MAX_PIPELINE_RECORDS` truncation
  block (the block ending in `truncated = true; break 'steps;`):
  ```rust
  // ADR-060 §D8.2: LIMIT-aware early-stop. Fires at COMPLETE page boundary, after DI-019.
  // truncated is NOT set — this is a success-path query-driven early exit, not a capacity overflow.
  // CRITICAL: early_stopped MUST be set to true BEFORE break 'steps.
  if let Some(limit) = context.early_stop_limit {
      if all_records.len() >= limit {
          early_stopped = true;  // ADR-060 §D8.3: set BEFORE break — propagates to FetchOutput.any_early_stopped
          break 'steps;
      }
  }
  ```
  Set `early_stopped` in the returned `PipelineResult` so the production adapter can read it.
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

- [ ] **Task 9 (Implementation — spec_driven_adapter wiring + FetchOutput return):** In
  `SpecDrivenSensorAdapter::fetch` in `crates/prism-bin/src/spec_driven_adapter.rs`, insert
  immediately before `let context = FetchContext::new(...)`:
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
  Update the return to wrap in `FetchOutput` (ADR-060 §D8.3/§D8.9 chain):
  ```rust
  Ok(FetchOutput { batches: pipeline_result.batches, any_early_stopped: pipeline_result.early_stopped })
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
    MUST FAIL before Task 12 (without gate, `fetch_limit = 25` → early-stop fires after 1 page).
  - RG-PSG-002 (`test_BC_2_16_002_plan_shape_gate_group_by_suppresses_early_stop`): GROUP-BY-ONLY
    (no COUNT) AST to isolate Condition B. Assert `ast_is_reducing_plan = true`; `fetch_limit = 0`.
    MUST FAIL before Task 12.
  - RG-PSG-003 (`test_BC_2_16_002_plan_shape_gate_distinct_suppresses_early_stop`): AST with
    `SELECT DISTINCT col FROM t`. Assert `ast_is_reducing_plan = true`; `fetch_limit = 0`.
    MUST FAIL before Task 12.
  - RG-PSG-004 (`test_BC_2_16_002_plan_shape_gate_non_temporal_where_suppresses_early_stop`):
    AST with equality WHERE predicate (`WHERE col = 'val'`); `has_client_side_where` returns `true`.
    Assert `ast_is_reducing_plan = true`; `fetch_limit = 0`. MUST FAIL before Task 12.
  - RG-PSG-005 (`test_BC_2_16_002_plan_shape_gate_pipe_stats_suppresses_early_stop`): AST with
    `PipeStage::Stats` (e.g., `| stats count()`). Assert `ast_is_reducing_plan = true`;
    `fetch_limit = 0`. MUST FAIL before Task 12.
  - RG-PSG-006 (`test_BC_2_16_002_plan_shape_gate_pipe_dedup_suppresses_early_stop`): AST with
    `PipeStage::Dedup` (e.g., `| dedup col`). Assert `ast_is_reducing_plan = true`;
    `fetch_limit = 0`. MUST FAIL before Task 12.
  - RG-PSG-010 (`test_BC_2_16_002_plan_shape_gate_nested_agg_in_scalar_suppresses_early_stop`):
    AST with aggregate nested inside scalar UDF arg (e.g., `severity_label(max(severity_id))`).
    Assert `ast_is_reducing_plan = true` (Condition A revised; `expr_contains_aggregate_or_window`
    recurses into `FuncCall::Scalar::args`). MUST FAIL before Task 12.
  - RG-PSG-011 (`test_BC_2_16_002_plan_shape_gate_order_by_aggregate_suppresses_early_stop`):
    AST with aggregate in ORDER BY (e.g., `ORDER BY MAX(severity)` without GROUP BY). Assert
    `ast_is_reducing_plan = true` (Condition A revised; `OrderExpr` scanned). MUST FAIL before Task 12.
  - RG-PSG-013 (`test_BC_2_16_002_plan_shape_gate_filter_mode_where_suppresses_early_stop`):
    `Ast::Filter` with non-temporal predicate. Assert `has_client_side_where = true`;
    `ast_is_reducing_plan = true`. MUST FAIL before Task 12 (v1.2 `where_filters` empty for Filter mode).
  - RG-PSG-014 (`test_BC_2_16_002_plan_shape_gate_pipe_where_suppresses_early_stop`):
    `Ast::Pipe` with `PipeStage::Where(non-temporal-pred)`. Assert `has_client_side_where = true`;
    `ast_is_reducing_plan = true`. MUST FAIL before Task 12 (v1.2 `where_filters` empty for Pipe stages).
  - RG-PSG-015 (`test_BC_2_16_002_plan_shape_gate_non_equality_sql_where_suppresses_early_stop`):
    SQL WHERE with LIKE predicate (`WHERE status LIKE '%page2%'` — non-equality SQL predicate form;
    CONTAINS is a pipe StringOp/UDF, not a SQL predicate). Assert `has_client_side_where = true`;
    `ast_is_reducing_plan = true`. MUST FAIL before Task 12 (v1.2 `where_filters` equality-only).
  - RG-PSG-016 (`test_BC_2_16_002_plan_shape_gate_sql_join_suppresses_early_stop`): AST with
    SQL JOIN (`!sql.joins.is_empty()`). Assert `ast_is_reducing_plan = true`. MUST FAIL before Task 12.
  - RG-PSG-017 (`test_BC_2_16_002_plan_shape_gate_pipe_tail_suppresses_early_stop`): AST with
    `PipeStage::Tail(_)` in pipe stages. Assert `ast_is_reducing_plan = true`. MUST FAIL before Task 12.
  - RG-PSG-018 (`test_BC_2_16_002_plan_shape_gate_pipe_join_suppresses_early_stop`): AST with
    `PipeStage::Join(_)` in pipe stages. Assert `ast_is_reducing_plan = true` (Condition J
    defensive; Pipe Join errors at runtime but gate must already suppress). MUST FAIL before Task 12.

  **Suppression tests — IN-CRATE UNIT (defense-in-depth; SAP-3 rule-3; in `materialization.rs` plan_shape_gate_unit_tests):**
  - RG-PSG-009 (`test_BC_2_16_002_plan_shape_gate_having_suppresses_early_stop`): call
    `ast_is_reducing_plan` directly with AST for `GROUP BY col HAVING count(*) > N`. Assert
    `ast_is_reducing_plan = true` (Condition D). MUST FAIL before Task 12.
  - RG-PSG-012 (`test_BC_2_16_002_plan_shape_gate_window_function_suppresses_early_stop`): call
    `ast_is_reducing_plan` directly with AST containing `FuncCall::Window` in SELECT. Assert
    `ast_is_reducing_plan = true` (Condition A revised: `FuncCall::Window` detected). MUST FAIL before Task 12.
  - RG-PSG-019 (`test_BC_2_16_002_plan_shape_gate_conservative_default_suppresses_early_stop`):
    call `ast_is_reducing_plan` directly with a synthetic `PipeStage` not in the PERMIT allow-list.
    Assert `ast_is_reducing_plan = true` (conservative default: `_ => true`). MUST FAIL before Task 12.

  **Positive controls (gate MUST NOT fire; early-stop MUST proceed):**
  - RG-PSG-007 (`test_BC_2_16_002_plan_shape_gate_bare_projection_early_stop_fires`): bare
    `SELECT * FROM t`, `options.limit=5`, 3-page mock (page_size=10). Assert
    `ast_is_reducing_plan = false`; `fetch_limit = 5`; early-stop fires after 1 page.
    MUST PASS before AND after Task 12 (confirms gate does NOT over-suppress).
  - RG-PSG-008 (`test_BC_2_16_002_plan_shape_gate_order_by_limit_early_stop_fires`): AST for
    `SELECT * FROM t ORDER BY col LIMIT N`. Assert `ast_is_reducing_plan = false` (§D8.5:
    ORDER BY alone is NOT a suppression condition); `fetch_limit = N`; early-stop fires.
    MUST PASS before AND after Task 12.

  **Truncation-signal correctness (END-TO-END / Integration via `QueryEngine::execute`):**
  - RG-PSG-020 (`test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter`) in
    `crates/prism-query/tests/execute_integration_tests.rs`. For a filter query whose WHERE
    clause suppresses early-stop (gate fires, `fetch_limit = 0`, full pagination, 100 rows match
    predicate), assert `is_truncated = true`, `total_available = 100` (true pre-cap count),
    `returned_results = 25` (tool-level cap applied by engine.rs Step 6). Verifies engine.rs
    Step 6 is responsible for the cap + signal; materialization MUST return the full set of
    100 rows WITHOUT applying a pre-cap. MUST FAIL if materialization applies a tool-level
    pre-cap before returning to Step 6 (pre-cap causes Step 6 to see 25 rows as `total_available`,
    producing `is_truncated = false` — incorrect). Write this test RED; Task 11 implements the
    materialization result boundary fix that makes it GREEN.

  **Temporal-exemption soundness (END-TO-END / Integration via `run_materialization_pipeline`, in-process `PlanShapeGateMockAdapter`):**
  - RG-PSG-021 (`test_psg_filter_mode_temporal_suppresses_early_stop`) in
    `crates/prism-query/tests/plan_shape_gate_tests.rs`. Filter-mode query with purely temporal
    predicate. Assert `ast_is_reducing_plan = true`; `fetch_limit = 0`; all pages fetched.
    MUST FAIL before Task 12 (v1.3 unsound: temporal filter-mode → PERMIT → under-returned rows).
  - RG-PSG-022 (`test_psg_pipe_where_temporal_suppresses_early_stop`) in the same file.
    Pipe-mode query with `PipeStage::Where` containing temporal predicate. Assert
    `ast_is_reducing_plan = true`; `fetch_limit = 0`. MUST FAIL before Task 12 (v1.3 unsound:
    pipe `| where` temporal → PERMIT → under-returned rows).
  - RG-PSG-023 (`test_psg_sql_eq_temporal_suppresses_early_stop`) in the same file.
    SQL query `WHERE timestamp = '2024-06-15T00:00:00Z'` (Eq operator). Assert
    `ast_is_reducing_plan = true`; `fetch_limit = 0`. MUST FAIL before Task 12.
  - RG-PSG-024 (`test_psg_sql_non_index_temporal_suppresses_early_stop`) in the same file.
    SQL query `WHERE updated_at >= '2024-01-01T00:00:00Z'` where `updated_at` is NOT an INDEX
    datetime column. Assert `ast_is_reducing_plan = true`; `fetch_limit = 0`. MUST FAIL before
    Task 12.
  - RG-PSG-029 (`test_psg_relative_temporal_now_interval_suppresses_early_stop`) in the same file.
    SQL query `WHERE <non_index_datetime_col> >= now() - interval '7d' LIMIT N`. Real mechanism:
    `inject_now` folds `now() - interval '7d'` to `Expr::Literal(Literal::Timestamp)` BEFORE
    `ast_is_reducing_plan` is called; the gate never sees a relative RHS. After folding,
    `is_pushed_temporal_predicate` returns `false` because the column is NOT in `datetime_index_cols`
    (non-INDEX; precondition (b) fails — same mechanism as RG-PSG-024) →
    `has_client_side_where = true` → `ast_is_reducing_plan = true` → `fetch_limit = 0`; all pages
    fetched. Regression guard overlapping RG-PSG-024; also exercises the `inject_now` fold path.
    MUST FAIL against current code (which PERMITs via `is_temporal_expr` before fold).
    Write RED; Task 12 makes it GREEN. (Anchored: AC-008(c); F-R16-P1-CRIT-001)

  **`any_early_stopped` propagation / exact-limit truncation signal (END-TO-END via `QueryEngine::execute`):**
  - RG-PSG-025 (`test_psg_exact_limit_is_truncated_true`) in
    `crates/prism-query/tests/execute_integration_tests.rs`. Early-stop fires at the
    exact-limit boundary (`options.limit = 100`; first page returns exactly 100 rows;
    `any_early_stopped = true`). Assert `is_truncated = true` (from `any_early_stopped` OR term;
    `total_rows == limit` so `total_rows > limit` is false); `total_available = 100`;
    `returned_results = 100`. MUST FAIL before `any_early_stopped` propagation chain is
    implemented. Write RED; Task 11 (`any_early_stopped` wiring) or Task 12 (gate impl)
    makes it GREEN.

  **MCP wire-level (MCP integration in `crates/prism-bin/tests/mcp_integration_tests.rs` or equivalent MCP stdio test file):**
  - RG-PSG-026 (`test_psg_rg026_prism_query_wire_surfaces_truncation_signal`): spawns the MCP
    server (or uses in-process MCP dispatch) with a DTU-style mock returning exactly N records
    per page. Makes two `prism_query` MCP calls: (1) bare-projection `LIMIT N` at exact
    boundary — asserts `"is_truncated": true` in the serialized `CallToolResult` JSON content
    text; (2) temporal `| where` predicate + `LIMIT N` at same boundary (gate Condition G
    unconditional Pipe-WHERE suppression → early-stop disabled → `any_early_stopped = false`
    → `total_rows == limit` → formula `total_rows > limit = false`) — asserts
    `"is_truncated": false` in wire JSON. Wire-shape discipline (CLAUDE.md 2026-07-13):
    MCP-visible surfaces MUST be asserted at the serialized JSON level; `QueryResult` is not
    `Serialize` so RG-PSG-025 cannot reach the wire. MUST FAIL before `any_early_stopped`
    chain is wired through to MCP serialization. Write RED; GREEN after Tasks 11 and 12 both
    complete AND `is_truncated` MCP serialization correctly applies the formula.

  **Multi-sensor fan-out / heuristic-rejection (in `crates/prism-query/tests/execute_integration_tests.rs` and `crates/prism-bin/tests/mcp_integration_tests.rs`):**
  - RG-PSG-027 (`test_psg_multi_sensor_fanout_exact_limit_one_early_stopped_is_truncated_true`):
    2-sensor fan-out with `options.limit = 50`; sensor1 returns 40 rows (no early-stop);
    sensor2 returns 10 rows and early-stops. `FanOutResult.any_early_stopped = true` via
    OR-aggregation. `total_rows = 50 = limit`. Assert `is_truncated = true` (from
    `any_early_stopped` OR term). MUST FAIL before the `FetchOutput.any_early_stopped`
    propagation chain is implemented. Write RED; GREEN after Task 16 (chain wiring) completes.
    (Anchored: AC-009(a); test multi-sensor OR-aggregation path through fanout.rs)
  - RG-PSG-028 (`test_psg_multi_sensor_fanout_exact_total_no_early_stop_is_not_truncated`):
    2-sensor fan-out with `options.limit = 50`; both sensors return 25 rows WITHOUT
    early-stopping. `FanOutResult.any_early_stopped = false`. `total_rows = 50 = limit`.
    Assert `is_truncated = false` (formula: `(50 > 50) || false = false`).
    MUST FAIL if heuristic `total_fetched_rows >= fetch_limit` is used (50 >= 50 = true → wrong).
    Wire-level assertion: also assert `"is_truncated": false` in serialized `CallToolResult`
    JSON in `crates/prism-bin/tests/mcp_integration_tests.rs`. Write RED; GREEN after Task 16.
    (Anchored: AC-009(a); canonical heuristic-rejection test)

- [ ] **Task 11 (Implementation — materialization result boundary):** Remove any
  `truncate_result_to_limit` pre-cap applied within `run_materialization_pipeline` before
  returning to engine.rs Step 6. `run_materialization_pipeline` MUST return the full
  filtered/aggregated result set to Step 6 without capping at the tool-level limit.
  Engine.rs Step 6 is the sole owner of the tool-level cap and the `is_truncated` /
  `total_available` truncation signal (ADR-060 §D8.7/§D8.8 result-cap responsibility boundary;
  §Architecture Compliance Rules "materialization result-cap responsibility boundary" entry).
  Verify that engine.rs Step 6 reads the full pre-cap row count from the materialization
  result and computes `is_truncated` and `total_available` correctly before applying the cap.
  After editing: run `just iter prism-query` —
  RG-PSG-020 (`test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter`) MUST turn GREEN.

- [ ] **Task 12 (Implementation — `ast_is_reducing_plan` + `run_materialization_pipeline` gate):**
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
          // Known non-aggregate leaf Expr variants (Expr::Column, Expr::Literal, comparisons,
          // etc.) are enumerated explicitly above returning false.
          // ADR-060 §D8.7 conservative default: unknown/future Expr variants (e.g., CASE) →
          // true (SUPPRESS). See §Architecture Compliance Rules. Do NOT change to false.
          _ => true,
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
  RG-PSG-019, RG-PSG-021 through RG-PSG-024, RG-PSG-029 MUST turn GREEN; RG-PSG-007 and
  RG-PSG-008 (positive controls) MUST remain GREEN. (RG-PSG-021..024 test temporal-exemption
  soundness via the updated `has_client_side_where` logic: Filter-mode unconditional suppress,
  Pipe-WHERE unconditional suppress, Eq-operator temporal suppress, non-INDEX column suppress.
  RG-PSG-029 tests the folded relative-temporal path: `inject_now` folds `now() - interval '7d'`
  to `Literal::Timestamp` BEFORE the gate; suppression is then via non-INDEX column check —
  same mechanism as RG-PSG-024, also exercises the `inject_now` fold path.)

- [ ] **Task 13 (Integration sweep — update all remaining callers):** Run `just check --no-fail-fast`
  across the full workspace. All integration test files listed in `crates_touched` that were
  updated in Task 5 should compile. If any callers were missed in Task 5, find them now via the
  compile errors and update each to pass `None`. Run `just iter prism-spec-engine` to confirm
  all pipeline.rs-adjacent tests pass. Run `just iter prism-bin` to confirm all prism-bin tests
  pass.

- [ ] **Task 14 (SAP-1 self-check):** Confirm that no new `tracing::*!(event_type = ...)` emissions
  are added. BC-2.16.002 SAP-1 declaration states: "ADR-060 introduces NO new `event_type`
  values; the existing `pipeline_truncated` WARN event (DI-019 cap only) is NOT altered; catalog
  count unchanged at 96." The early-stop branch and the plan-shape gate have no emissions —
  this is intentional and documented.

- [ ] **Task 15 (Final gate — ADR-060 paths):** Run `just check` (full workspace). Confirm all
  non-`#[ignore]` Red Gate tests pass: RG-001, RG-002, RG-003, RG-004, RG-005, RG-006,
  RG-PSG-001 through RG-PSG-029. Confirm `EXPECTED_SYMBOLS` in
  `scripts/check-non-exhaustive-per-symbol.py` does NOT need updating for private functions
  (`ast_is_reducing_plan`, `expr_contains_aggregate_or_window`, `has_client_side_where`,
  `is_pushed_temporal_predicate`). `FetchOutput` is a new `pub` struct — ADD it to
  `EXPECTED_SYMBOLS` with `#[non_exhaustive]` attribute.
  Confirm no new `unwrap()`/`expect()` in production code paths. NOTE: RG-SLUG-001..006
  (ADR-061) are authored in Task 17 and green in Tasks 18–19; Task 15 covers only the ADR-060
  gate range. See Task 19 for the ADR-061 final gate.

- [ ] **Task 16 (Implementation — full `any_early_stopped` chain wiring):** Wire the
  `any_early_stopped` signal through the complete propagation chain (ADR-060 §D8.9):

  **Step A — `crates/prism-sensors/src/adapter.rs`:** Define `FetchOutput` struct and change
  `SensorAdapter::fetch` return type (done in Task 9 prep; verify trait definition is updated).

  **Step B — `crates/prism-sensors/src/fanout.rs`:** Add `pub any_early_stopped: bool` to
  `FanOutResult`. In `fan_out()`, OR-aggregate across results:
  ```rust
  any_early_stopped: results.iter().any(|r| r.any_early_stopped),
  ```

  **Step C — `crates/prism-query/src/materialization.rs`:** Add `pub any_early_stopped: bool`
  to `MaterializationOutput`. After `fan_out()` completes, propagate:
  ```rust
  any_early_stopped: fan_out_result.any_early_stopped,
  ```
  DO NOT add `total_fetched_rows >= fetch_limit` heuristic. The `any_early_stopped` chain IS
  the correct mechanism (RG-PSG-028 enforces this).

  **Step D — `crates/prism-query/src/engine.rs` Step 6:** Update `is_truncated` formula:
  ```rust
  let is_truncated = total_rows > limit || materialization_output.any_early_stopped;
  ```

  **Step E — test-stub sweep (21 files):** Update all test stubs that implement `SensorAdapter`
  to return `Ok(FetchOutput { batches, any_early_stopped: false })`. See `crates_touched`
  frontmatter comment for the complete list. This is a mechanical change.

  After steps A–E: run `just iter prism-query` — RG-PSG-027 and RG-PSG-028 MUST turn GREEN.
  Run `just iter prism-sensors` — fan-out tests must pass. Run `just iter prism-bin` — MCP
  integration tests including RG-PSG-028 wire assertion must pass.

- [ ] **Task 17 (Red Gate — test first):** Write RG-SLUG-001 through RG-SLUG-006 in
  `crates/prism-query/tests/slug_isolation_tests.rs` (new file, or extend `materialization_tests.rs`
  if it exists). RG-SLUG-006 MUST be in `crates/prism-query/tests/slug_isolation_tests.rs`
  (NOT in-crate — an in-crate `include_str!` test would include itself in the source scan,
  false-passing after production removal; self-reference false-pass avoidance). ALL SIX tests MUST be authored
  and RED before Task 18 begins (SAC-1 red-then-green ordering).

  **D2 skip-with-warn gates (RG-SLUG-001, RG-SLUG-003):** Build an `OrgRegistry` with a
  slug entry for `org_id_A` but NO entry for `org_id_B`. Exercise `resolve_source_refs`
  (RG-SLUG-001) and the bare-filter Step 3b adapter loop (RG-SLUG-003) respectively.
  Capture `tracing::warn!` events using `tracing_test::traced_test` or a
  `tracing::subscriber::with_default` block. Assert: (a) no `FanOutTarget` in the result list
  for `org_id_B`; (b) exactly one `warn!` event captured with `event_type` field equal to
  `"query.org_slug_resolution_failure"` and the `org_id` field matching `org_id_B`.
  MUST FAIL before Task 18 — the current code pushes a target with a synthetic `client_id`
  and emits NO warn event.

  **D3 synthetic-slug preservation (RG-SLUG-002, RG-SLUG-004):** Set `org_registry = None`.
  Assert: a synthetic slug is generated and the `FanOutTarget` IS included. These PASS before
  AND after Task 18 (regression sentinels for D3 test-mode preservation).

  **Wire-level cross-tenant isolation gate (RG-SLUG-005):** Construct two `OrgId` values
  `A` and `B` so that `A.to_string()[..8] == B.to_string()[..8]` (identical 8-hex timestamp
  prefix — simulates same UUIDv7 window). Register both in a shared `OrgRegistry` with
  DISTINCT slugs (`"tenant-alpha"` / `"tenant-beta"`). Wire two adapters: adapter-A returns
  `"alpha-001"` rows, adapter-B returns `"beta-001"` rows.
  Issue a **single ALL-scope bare-filter query** (`clients: None`) against a `QueryEngine`
  wired with an **EMPTY `ClientRegistry`** (no explicit-client slugs). With an empty
  `ClientRegistry`, `resolve_clients(None, empty)` → `[]` → the D4 bare-filter Step 3b
  fan-out path fires, enumerating all adapters from the `adapter_registry`. Collect the
  `provider` column values from all result batches and serialize to JSON (wire-shape
  discipline, CLAUDE.md §Conventions). Assert on the **serialized JSON bytes**:
  - The serialized JSON CONTAINS `"beta-001"` — both adapters fetched under distinct cache keys.
  Note: a populated `ClientRegistry` would route through D5 (`resolve_source_refs`) which
  already resolves `OrgRegistry`, bypassing Step 3b — the test would false-green even with
  the D4 collision present. The empty `ClientRegistry` is required to force the D4 defect path.
  MUST FAIL before Task 18 — D4 Step 3b synthesizes `"org-deadbeef"` for both orgs →
  same `CacheKey` partition → adapter-B cache HIT → `"beta-001"` absent from result.

  **Sentinel absence gate (RG-SLUG-006):** Write `test_rg_slug_006_synthetic_unmapped_sentinel_absent`
  in `crates/prism-query/tests/slug_isolation_tests.rs` — NOT in-crate. An in-crate
  `include_str!("../src/materialization.rs")` test would include the test file's own source
  in the scan (self-reference false-pass: the string `"synthetic-unmapped"` in the test body
  would be found, causing the test to pass even after the sentinel is removed from production
  code). The external test reads `crates/prism-query/src/materialization.rs` via
  `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/materialization.rs"))` and asserts
  that `"synthetic-unmapped"` does not appear in it.
  MUST FAIL before Task 19 (sentinel not yet removed).

  After authoring all six tests: run `just iter prism-query` — all six MUST be RED
  (RG-SLUG-002 and RG-SLUG-004 may be GREEN already — they are regression sentinels).

- [ ] **Task 18 (Implementation — ADR-061 Sites 1+2: `resolve_source_refs` ALL-scope + bare-filter Step 3b):**
  In `crates/prism-query/src/materialization.rs`, apply the three-arm registry-first dispatch
  to BOTH defect sites per ADR-061 D4 + D5:

  **Site 2 — `resolve_source_refs` ALL-scope (ADR-061 D5):** Replace the unified
  `let Some(client_slug) = org_registry.as_ref().and_then(...) else { /* synthesize */ }` pattern
  with the `match` dispatch: `Some(slug) → slug` (D1 authoritative path);
  `None if org_registry.is_some() → tracing::warn!(event_type = "query.org_slug_resolution_failure") + continue` (D2 fail-closed);
  `None → synthetic_from_org_id_d3(&org_id)` (D3 test mode). Per AC-010.

  **Site 1 — bare-filter Step 3b (ADR-061 D4):** Replace the `OrgSlug::new(format!("org-{}", &org_id.to_string()[..8]))` line that never consulted `mat_ctx.org_registry` with the same three-arm dispatch. The comment claiming `mat_ctx.org_registry` is unavailable is INCORRECT and MUST be removed. Per AC-011.

  For D3, the deterministic-prefix helper (usable at both sites):
  `format!("org-{}", &org_id.to_string()[..8])` — the `"org-"` literal prefix guarantees
  ORG_SLUG_PATTERN compliance regardless of the hex characters; no digit-prefix special case is
  required (digits are valid in any position per the pattern). Any existing defensive fallback
  branch in the code (e.g., an `"org-x"` path when the first hex char is a digit) is unreachable
  by construction.

  After editing: run `just iter prism-query` — RG-SLUG-001, RG-SLUG-002, RG-SLUG-003,
  RG-SLUG-004, and RG-SLUG-005 MUST turn GREEN. RG-SLUG-006 remains RED (sentinel not removed until Task 19).

- [ ] **Task 19 (Implementation — ADR-061 Site 3 sentinel removal + D7 test deletion):**

  **Site 3 (ADR-061 D3 sentinel removal):** Remove `OrgSlug::new("synthetic-unmapped")` from
  ALL production (non-`#[cfg(test)]`) code paths in `crates/prism-query/src/materialization.rs`.
  The `"synthetic-unmapped"` literal MUST NOT appear in any non-test code path. It is
  unconditionally superseded by the deterministic `x`-prefix form from Task 18. Verify by
  grep: `rg '"synthetic-unmapped"' crates/prism-query/src/ --type rust` must return no hits.

  **D7 (ADR-061 D7 test deletion):** Delete `crates/prism-core/tests/org_slug_from_uuid_prefix.rs`.
  This test asserts the 8-hex synthesis pattern is always correct — it legitimizes the defect.
  Before deleting: verify the file covers ONLY the synthesis pattern. If it covers other
  unrelated behavior, extract those assertions to `crates/prism-core/tests/org_slug_tests.rs`
  before deletion (TD-VSDD-060 sibling-sweep for test coverage). Commit the deletion with:
  "ADR-061 D7: remove org_slug_from_uuid_prefix.rs — test legitimized the defect (8-hex synthesis
  bypasses OrgRegistry); replaced by RG-SLUG-001/003/005 in prism-query."

  After editing: run `just iter prism-query` — RG-SLUG-006 MUST turn GREEN.
  Run `just iter prism-core` — workspace compiles without the deleted test file.
  Run `just check` — full workspace GREEN including `EXPECTED_SYMBOLS`
  in `scripts/check-non-exhaustive-per-symbol.py` (no new types; the deleted test file
  has no registered symbols). After `just check` passes, hold for story-level holdout
  gate before pushing to origin.

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

6. **Implementer NOTE (round-16, comment-only):** The `plan_shape_gate_tests.rs` module-level
   doc header still cites "Task 11" for the plan-shape gate implementation. This was the correct
   ordinal before the round-14 renumbering; after that renumber, the correct task ordinal is
   Task 12. Fix the module-level doc comment in the round-16 implementation burst (`// ADR-060
   §D8.7... Task 11` → `Task 12`; comment-only, no logic change). Lens-C FINDING-1 / lens-B
   LOW-1. Do NOT edit code in this story-writer burst.

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

From ADR-060 §D8.7 Condition A, `expr_contains_aggregate_or_window` conservative default:
- The function's terminal `match` arm MUST be `_ => true` (conservative SUPPRESS), NOT `_ => false`.
  Known non-aggregate leaf `Expr` variants are enumerated explicitly returning `false`; an
  unknown or future `Expr` variant (e.g., a CASE expression) is treated as potentially-aggregate
  → SUPPRESS. This extends the conservative-default posture already applied at the `Ast`,
  `PipeStage`, and `FuncCall` dispatch levels to the `Expr`-recursion level (per AC-007
  Conservative Default). This is a design-time invariant; there is no reachable test for unknown
  `Expr` variants today — the rule is enforced by code review, not a Red Gate test.

From ADR-060 §D8.7/§D8.9 (temporal-exemption soundness — v1.5):
- `has_client_side_where` MUST treat `Ast::Filter` mode as UNCONDITIONALLY SUPPRESSED regardless
  of predicate form. Filter-mode predicates are always DataFusion client-side filters;
  `extract_time_bounds_from_predicate` (ADR-033 T1) does NOT process `Ast::Filter` predicates.
  The v1.3 `!is_purely_temporal_predicate` check for this arm was UNSOUND and is removed.
- `has_client_side_where` MUST treat any `PipeStage::Where(_)` in `Ast::Pipe` or `Ast::SqlPipe`
  stages as UNCONDITIONALLY SUPPRESSED. Pipe `| where` stages push NOTHING server-side.
  `PipeStage::Where` is REMOVED from the PERMIT allow-list. Do NOT add it back.
- `is_pushed_temporal_predicate(pred, datetime_index_cols)` MUST require ALL THREE preconditions:
  (a) range operator (Gt/Ge/Lt/Le — NOT Eq/Ne); (b) LHS field in `datetime_index_cols` (INDEX
  datetime column — sensor TOML `index: true` + `column_type = "Datetime"`); (c) RHS
  `Expr::Literal(Literal::Timestamp)` (concrete absolute timestamp). Temporal equality (`Eq`),
  `Expr::Now`, `Expr::Interval`, `Expr::TimestampArithmetic`, and non-INDEX datetime columns
  MUST SUPPRESS. This mirrors `extract_time_bounds_from_predicate` (ADR-033 T1) exactly.
  (Anchored: RG-PSG-021 through RG-PSG-024, RG-PSG-029)

From ADR-060 §D8.3/§D8.9 (`any_early_stopped` propagation chain):
- `PipelineResult.early_stopped: bool` MUST be set `true` on the §D8.2 `break 'steps` exit
  (DISTINCT from `truncated`: `truncated` = DI-019 capacity exceeded; `early_stopped` =
  query-driven early exit at the limit boundary).
  **CRITICAL ORDER:** `early_stopped = true` MUST be assigned BEFORE `break 'steps`. Setting it
  after the break (or omitting it) silently zeros the signal on every pipeline exit.
  (Anchored: RG-PSG-025 `test_psg_exact_limit_is_truncated_true`)
- `SensorAdapter::fetch` return type MUST change from `Result<Vec<RecordBatch>, SensorError>` to
  `Result<FetchOutput, SensorError>` where `pub struct FetchOutput { pub batches: Vec<RecordBatch>, pub any_early_stopped: bool }`.
  Defined in `crates/prism-sensors/src/adapter.rs`. All 21 test stubs MUST return
  `Ok(FetchOutput { batches, any_early_stopped: false })`. The production adapter MUST return
  `Ok(FetchOutput { batches: result.batches, any_early_stopped: pipeline_result.early_stopped })`.
- `FanOutResult.any_early_stopped: bool` MUST be added and OR-aggregated across all sensor results
  in `fan_out()`: `any_early_stopped = results.iter().any(|r| r.any_early_stopped)`.
  This is the AUTHORITATIVE OR-aggregation point (ADR-060 §D8.9).
  (Anchored: RG-PSG-027 `test_psg_multi_sensor_fanout_exact_limit_one_early_stopped_is_truncated_true`)
- **HEURISTIC REJECTION (Anchored: RG-PSG-028):** The `is_truncated` signal MUST NOT be derived
  from `total_fetched_rows >= fetch_limit`. This heuristic produces wrong `is_truncated=true` on
  multi-sensor fan-out when multiple sensors each return fewer than `fetch_limit` rows but their
  sum equals `fetch_limit` and none early-stopped. The ONLY correct formula is
  `is_truncated = (total_rows > limit) || any_early_stopped` (engine.rs Step 6; ADR-060 §D8.9).
  RG-PSG-028 (`test_psg_multi_sensor_fanout_exact_total_no_early_stop_is_not_truncated`) MUST FAIL
  if the heuristic is used — the test is specifically designed to expose the heuristic's blind spot.
  (Anchored: RG-PSG-028 `test_psg_multi_sensor_fanout_exact_total_no_early_stop_is_not_truncated`)
- `MaterializationOutput.any_early_stopped: bool` MUST be added and populated from `FanOutResult`.
  The `any_early_stopped` signal MUST propagate the full chain:
  `FetchOutput → FanOutResult → MaterializationOutput → engine.rs Step 6` (ADR-060 §D8.9).
  Do NOT lose or discard the `any_early_stopped` signal at any intermediate layer.
- Engine.rs Step 6 MUST use `let is_truncated = total_rows > limit || materialization_output.any_early_stopped;`.
  This is the SOLE site where `is_truncated` is computed from the `any_early_stopped` chain.
- `run_materialization_pipeline` MUST NOT apply a tool-level pre-cap before returning to
  engine.rs Step 6. The full filtered/aggregated result is returned; Step 6 is the SOLE owner
  of the tool-level cap and `is_truncated`/`total_available` semantics (BC-2.11.001 EC-11-093;
  F-R13-CRIT-001 prohibited behavior).
  (Anchored: RG-PSG-025 `test_psg_exact_limit_is_truncated_true`)

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
| MODIFY | `crates/prism-sensors/src/adapter.rs` | (a) Define `pub struct FetchOutput { pub batches: Vec<RecordBatch>, pub any_early_stopped: bool }`; (b) change `SensorAdapter::fetch` return type from `Result<Vec<RecordBatch>, SensorError>` to `Result<FetchOutput, SensorError>` — all impl sites must update |
| MODIFY | `crates/prism-sensors/src/fanout.rs` | (a) Add `pub any_early_stopped: bool` field to `FanOutResult`; (b) OR-aggregate `any_early_stopped` across all sensor results in `fan_out()` — set to `true` if ANY sensor result's `any_early_stopped` flag is `true`, using `Iterator::any` on the results slice (ADR-060 §D8.9) |
| MODIFY | `crates/prism-spec-engine/src/pipeline.rs` | (a) Add `early_stop_limit` field to `FetchContext`; (b) expand `FetchContext::new` signature; (c) add `pub early_stopped: bool` field to `PipelineResult`; (d) add early-stop check after DI-019 in `execute_impl`; (e) **SET `early_stopped = true` BEFORE `break 'steps`** in the §D8.2 early-stop block; (f) update ~15 in-file test sites (including `default_context()` helper) to pass `None` |
| MODIFY | `crates/prism-bin/src/spec_driven_adapter.rs` | Add `early_stop_limit` mapping and `FetchContext::new` pass; return `FetchOutput { batches: result.batches, any_early_stopped: pipeline_result.early_stopped }` |
| MODIFY | `crates/prism-query/src/materialization.rs` | (a) Add `expr_contains_aggregate_or_window(expr: &Expr) -> bool` helper (three-part: Aggregate variants, FuncCall::Window, recursion into FuncCall::Scalar::args); (b) add `ast_is_reducing_plan(ast: &Ast) -> bool` function (Conditions A–J + conservative default; `where_filters` NOT a parameter); (c) update `fetch_limit` derivation in `run_materialization_pipeline` to use plan-shape gate (before fan-out construction; `where_filters` NOT passed to gate); (d) add `pub any_early_stopped: bool` to `MaterializationOutput`; (e) pick up `any_early_stopped` from `FanOutResult` after fan-out; (f) **DO NOT add heuristic `total_fetched_rows >= fetch_limit`** — wrong on multi-sensor fan-out; **(g) ADR-061 D5 — `resolve_source_refs` ALL-scope: split unified `else` branch into three-arm `match` dispatch (D1 authoritative / D2 skip-with-warn / D3 synthetic); remove unified synthesis path**; **(h) ADR-061 D4 — bare-filter Step 3b: replace unconditional 8-hex `client_id` synthesis with registry-first dispatch; remove incorrect "no OrgRegistry available" code comment**; **(i) ADR-061 D3/Site 3: remove `"synthetic-unmapped"` sentinel from ALL production code paths; replace with deterministic `x`-prefix form** |
| MODIFY | `crates/prism-query/src/engine.rs` | Step 6: update `is_truncated` formula to `(total_rows > limit) OR materialization_output.any_early_stopped` — adds the `any_early_stopped` OR-term per ADR-060 §D8.9 authoritative formula. |
| MODIFY (×14) | Integration test files listed in `crates_touched` frontmatter comment | Update each `FetchContext::new` call to pass `None` as third arg |
| MODIFY (×21) | Test stub sweep — all `impl SensorAdapter` in test files (see `crates_touched` frontmatter comment) | Mechanical wrap: `Ok(Vec<RecordBatch>)` → `Ok(FetchOutput { batches, any_early_stopped: false })` for all test stubs |
| CREATE or EXTEND | `crates/prism-spec-engine/tests/bc_2_16_002_early_stop_tests.rs` OR extend `bc_2_16_002_test.rs` | RG-001, RG-002, RG-003, RG-004 |
| CREATE or EXTEND | `crates/prism-bin/tests/bc_2_16_002_early_stop_adapter_tests.rs` OR extend existing | RG-005, RG-006 |
| CREATE or EXTEND | `crates/prism-query/tests/plan_shape_gate_tests.rs` OR extend `materialization_tests.rs` | RG-PSG-001 through RG-PSG-019, RG-PSG-021 through RG-PSG-024; RG-PSG-009/012/019 are in-crate unit tests in `materialization.rs` `#[cfg(test)] mod plan_shape_gate_unit_tests`; RG-PSG-021..024 are temporal-exemption soundness E2E tests (Filter-mode/Pipe-WHERE/SQL-Eq/non-INDEX) |
| MODIFY or EXTEND | `crates/prism-query/tests/execute_integration_tests.rs` | RG-PSG-020: `test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter` (suppressed-filter full-pagination signal); RG-PSG-025: `test_psg_exact_limit_is_truncated_true` (exact-limit boundary `any_early_stopped → is_truncated=true`); RG-PSG-027: `test_psg_multi_sensor_fanout_exact_limit_one_early_stopped_is_truncated_true`; RG-PSG-028: `test_psg_multi_sensor_fanout_exact_total_no_early_stop_is_not_truncated` |
| CREATE or EXTEND | `crates/prism-bin/tests/mcp_integration_tests.rs` (or equivalent MCP stdio test file in `crates/prism-bin/tests/`) | RG-PSG-026: `test_psg_rg026_prism_query_wire_surfaces_truncation_signal` — MCP wire-level assertion on `prism_query` `CallToolResult` JSON for `is_truncated`; RG-PSG-028 wire assertion: `"is_truncated": false` for 2-sensor no-early-stop exact-total scenario; wire-shape discipline (CLAUDE.md 2026-07-13) |
| CREATE or EXTEND | `crates/prism-query/tests/slug_isolation_tests.rs` (new file, or extend `materialization_tests.rs`) | RG-SLUG-001: `test_rg_slug_001_resolve_source_refs_registry_present_slug_missing_skips_target_emits_warn`; RG-SLUG-002: `test_rg_slug_002_resolve_source_refs_registry_absent_synthetic_slug_included`; RG-SLUG-003: `test_rg_slug_003_bare_filter_step3b_registry_present_slug_missing_skips_target_emits_warn`; RG-SLUG-004: `test_rg_slug_004_bare_filter_step3b_registry_absent_synthetic_slug_included`; RG-SLUG-005: `test_rg_slug_005_cross_tenant_wire_isolation_collision_resistant_cache_keys` (END-TO-END via `QueryEngine::execute`, wire-level JSON assertion); RG-SLUG-006: `test_rg_slug_006_synthetic_unmapped_sentinel_absent` (MUST be external in `crates/prism-query/tests/slug_isolation_tests.rs` — in-crate placement causes self-reference false-pass; see Task 17 rationale) |
| DELETE | `crates/prism-core/tests/org_slug_from_uuid_prefix.rs` | ADR-061 D7 — this test asserts the 8-hex synthesis pattern as correct behavior; it legitimizes the defect. Must be deleted (or its non-synthesis assertions migrated to a separate test file) before the PR merges. |

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
- BC-2.11.001 EC-11-092 (trace reference — not in behavioral_contracts) — `any_early_stopped` feeds `is_truncated`; exact-limit boundary: `is_truncated = (total_rows > limit) OR any_early_stopped`; `total_available` is a LOWER BOUND when `any_early_stopped = true`; pagination halted, true dataset size unknown
- BC-2.11.001 EC-11-093 (trace reference) — Step 6 as SOLE owner of tool-level cap; materialization returns full set without pre-cap; `total_available = results_after_DataFusion_before_cap`; pre-cap-removal behavior (F-R13-CRIT-001 — applying the row cap inside materialization before returning to engine) is PROHIBITED
- BC-2.16.015 EC-016-015-007 (trace reference — not in behavioral_contracts) — Claroty LIMIT 1 early-stop; UNAFFECTED by §D8.7 (bare projection, ast_is_reducing_plan=false)
- BC-2.16.015 EC-016-015-008 (trace reference) — COUNT suppresses early-stop via §D8.7 Condition A; full dataset fetched
- BC-2.16.015 TV-BC-2.16.015-006 (trace reference) — LIMIT 1 single-page test vector; promoted to active by S-CLAROTY-VULNS-001 merge per POL-14
- ADR-060 §D8 — FetchContext field, execute_impl check, truncated semantics, modes, ORDER BY, timeout deferral
- ADR-060 §D8.7 v1.3 / §D8.9 v1.5 — Plan-Shape Gate: `ast_is_reducing_plan(&ast)` Conditions A–J + conservative default; `where_filters` NOT forwarded to gate; enforcement in `run_materialization_pipeline` before fan-out construction; temporal-exemption soundness (`has_client_side_where` Filter-mode/Pipe-WHERE unconditional suppress; `is_pushed_temporal_predicate` for SQL/SqlPipe-head); `FetchOutput`/`any_early_stopped` propagation chain; ORDER BY non-suppression
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
| 1.20 | 2026-08-28 | story-writer | **F-R16-P4-LOW-001 — AC-012 and Task 18 false digit-prefix rationale corrected.** AC-012 heading + body and Task 18 D3-helper paragraph described a false `"org-x{}"` branch for digit-prefixed hex UUIDs. The actual synthesis is `format!("org-{}", &org_id.to_string()[..8])`; the `"org-"` literal prefix guarantees ORG_SLUG_PATTERN compliance regardless of the hex characters, so no digit-prefix special case exists. The `"org-x"` branch is unreachable by construction, not a digit-prefix workaround. (1) AC-012 heading: replaced "deterministic `x`-prefix form used for D3 test-mode UUID-prefix-validation failures" with "D3 test-mode synthesis produces `org-{8hex}` OrgSlug valid by construction". (2) AC-012 body: removed false two-branch conditional description; replaced with single `format!("org-{}", &org_id.to_string()[..8])` valid-by-construction explanation; added `"org-"` prefix / ORG_SLUG_PATTERN rationale; noted this is TEST-MODE-ONLY (`org_registry == None`) and that any defensive fallback is unreachable by construction. (3) Task 18 D3-helper paragraph: replaced digit-conditional branching with single `format!("org-{}", ...)` instruction plus note that any defensive fallback is unreachable by construction. No AC, RG, task, or count changes; spec-content-only corrections. TD-VSDD-097: Dim-1 — no named story-twin for S-ENGINE-LIMIT-EARLY-STOP-001; ADR-061 has no sibling ADR at the same decision level; CLEAR. Dim-2 — AC-012 heading, body, and Task 18 D3 paragraph updated consistently; frontmatter `version: "1.19"→"1.20"` and `modified: "2026-08-28"` (date unchanged); `acceptance_criteria_count: 13` and `red_gate_tests: 41` UNCHANGED; STORY-INDEX row updated; FULL. Dim-3 — no new MUSTs introduced; corrected text describes the existing anchored test `test_rg_slug_006_synthetic_unmapped_sentinel_absent` (already anchored to AC-012); CLEAR. |
| 1.19 | 2026-08-28 | story-writer | **F-R16-P3-MED-001 — AC-013 and RG-SLUG-005 test-vehicle reconciliation.** The prior description of AC-013 and RG-SLUG-005 specified issuing a bare-predicate filter query **as each tenant separately** (per-tenant explicit-client-scoped queries), with mutual-exclusion assertions (tenant-A response has zero tenant-B rows, tenant-B response has zero tenant-A rows). This vehicle is VACUOUS for the D4 defect: per-tenant explicit-client queries route through `resolve_source_refs` (D5), which already consults `OrgRegistry` correctly — bypassing D4 bare-filter Step 3b entirely. The test would false-green even with the D4 collision present. The actual test (`test_rg_slug_005_cross_tenant_wire_isolation_collision_resistant_cache_keys`) correctly uses a **single ALL-scope query** (`clients: None`) with an **EMPTY `ClientRegistry`**, forcing `resolve_clients(None, empty)` → `[]` → Step 3b fires. The single assertion: serialized wire JSON CONTAINS `"beta-001"`, proving both tenants' rows appear under distinct cache keys (collision-resistance). (1) AC-013 body: replaced per-tenant two-query vehicle with single ALL-scope / empty `ClientRegistry` vehicle; updated assertion to `wire_json.contains("\"beta-001\"")`; added explicit note explaining why per-tenant explicit-client queries cannot exercise D4. (2) §Red Gate Tests RG-SLUG-005 row: replaced per-tenant description with single ALL-scope + EMPTY `ClientRegistry` description; replaced mutual-exclusion assertion with collision-resistance assertion (`"beta-001"` present). (3) Task 17 RG-SLUG-005 description: replaced per-tenant two-query setup with single ALL-scope / empty `ClientRegistry` setup; updated assertion. Security intent intact (CWE-284/CWE-340 cross-tenant cache-key isolation still proven; no weakening). TD-VSDD-097: Dim-1 — no named story-twin; CLEAR. Dim-2 — AC-013 body, §Red Gate Tests RG-SLUG-005 row, and Task 17 RG-SLUG-005 description all updated consistently to the corrected vehicle; frontmatter counts (`acceptance_criteria_count: 13`, `red_gate_tests: 41`) and density paragraph UNCHANGED (no ACs or RGs added/removed); FULL. Dim-3 — no new MUSTs introduced; corrected text describes existing anchored test `test_rg_slug_005_cross_tenant_wire_isolation_collision_resistant_cache_keys` already anchored to AC-013 and ADR-061 D4; CLEAR. |
| 1.18 | 2026-08-28 | story-writer | **Records-tier spec fixes: F-R16-P2-LOW-002 (RG-SLUG-006 location) + F-R16-P2-OBS-001 (RG-PSG-029 mechanism).** (1) LOW-002: RG-SLUG-006 location corrected from in-crate `materialization.rs #[cfg(test)]` to `crates/prism-query/tests/slug_isolation_tests.rs` (EXTERNAL) in four locations: §Red Gate Tests row, Task 17 description, Task 19 sentinel-absence gate, §File Structure Requirements. Rationale: in-crate `include_str!` would include the test file's own source in the scan — the test body contains `"synthetic-unmapped"`, causing a self-reference false-pass after production removal. Code location wins over story spec per CLAUDE.md source-of-truth precedence. (2) OBS-001: RG-PSG-029 causal mechanism corrected in §Red Gate Tests row, Task 10 bullet, and Task 12 note. Original narrative claimed `Expr::Now`/`Expr::Interval`/`Expr::TimestampArithmetic` RHS causes `is_pushed_temporal_predicate` to return `false`. Correct mechanism: `inject_now` folds all relative-temporal forms to `Expr::Literal(Literal::Timestamp)` BEFORE `ast_is_reducing_plan` is called; the plan-shape gate never receives a relative RHS. Suppression is via non-INDEX datetime column (precondition (b) fails; same mechanism as RG-PSG-024). Column label changed from `<index_datetime_col>` to `<non_index_datetime_col>` in affected rows. RG-PSG-029 retained as regression guard — unique value is exercising the `inject_now` fold path. TD-VSDD-097: Dim-1 — no named story-twin; CLEAR. Dim-2 — changelog rows record prior history; no downstream artifact copies these narratives verbatim; CLEAR. Dim-3 — no new MUSTs introduced; records-tier narrative corrections only; CLEAR. |
| 1.17 | 2026-08-27 | story-writer | **F-R16-P1-HIGH-001 (CRITICAL cache-key isolation) — ADR-061 multi-tenant cache-key isolation folded in.** CWE-284/CWE-340/CWE-200, OWASP A01: three `prism-query::materialization` sites synthesize `client_id` from 8-hex `OrgId` truncation or `"synthetic-unmapped"` sentinel, bypassing `OrgRegistry`. (1) AC-010 added: `resolve_source_refs` ALL-scope fail-closed dispatch — `org_registry: Some(reg)` + slug missing → SKIP + `tracing::warn!(event_type = "query.org_slug_resolution_failure")`; `org_registry: None` → D3 synthetic slug preserved (ADR-061 D2/D3/D5; traces to BC-2.16.002 v2.42 catalog row). (2) AC-011 added: bare-filter Step 3b same dispatch — `mat_ctx.org_registry: Some(reg)` + slug missing → target NOT pushed, warn fired; `None` → D3 synthetic (ADR-061 D2/D3/D4). (3) AC-012 added: `"synthetic-unmapped"` sentinel ABSENT from production code paths; `org_slug_from_uuid_prefix.rs` deleted (ADR-061 D3/D7). (4) AC-013 added: wire-level cross-tenant isolation gate — two OrgIds with identical first-8-hex-char prefix, DISTINCT registry slugs, distinct seeded rows; serialized JSON for tenant B contains ZERO tenant A rows (ADR-061 D1/D9 RG-SLUG-005; wire-shape discipline). (5) RG-SLUG-001..006 added to §Red Gate Tests: warn+skip (001, 003), synthetic-slug preservation (002, 004), wire-level collision-resistance (005), sentinel-absence (006). (6) Tasks 17–19 added (SAC-1 red-then-green: Task 17 RG-SLUG-001..006 authoring RED; Task 18 Sites 1+2 implementation; Task 19 Site 3 + D7 deletion). (7) §File Structure Requirements: materialization.rs items (g)/(h)/(i) added; DELETE row for `crates/prism-core/tests/org_slug_from_uuid_prefix.rs`; CREATE/EXTEND row for slug isolation tests. (8) Frontmatter: `acceptance_criteria_count: 9→13`; `red_gate_tests: 35→41`; `crates_touched` extended with `prism-core` (D7 deletion); BC-2.16.002 body-table pin v2.41→v2.42. BC-5.38.001 density: 41/13≈3.15 ≥ 0.5. PASS. TD-VSDD-097: Dim-1 — no named story-twin for S-ENGINE-LIMIT-EARLY-STOP-001; ADR-061 has no sibling ADR at same decision level; CLEAR. Dim-2 — BC-2.16.002 v2.42 catalog row already added by PO in that file (confirmed); story body-table BC-2.16.002 version updated v2.41→v2.42; `red_gate_tests: 41` consistent in frontmatter, density paragraph (41/13≈3.15), Task 17 (RG-SLUG-001..006 bullets), Task 18 (green-gate range extended); `acceptance_criteria_count: 13` consistent with 13 ACs in body; `crates_touched` array includes `prism-core`; §File Structure Requirements DELETE row; FULL. Dim-3 — RG-SLUG-001..006 MUSTs anchored to named tests + AC references; AC-010..013 MUSTs anchored to ADR-061 D-section + BC-2.16.002 catalog row; no unanchored MUSTs introduced; CLEAR. |
| 1.16 | 2026-08-27 | story-writer | **F-R16-P1-CRIT-001 remediation: RG-PSG-029 relative-temporal RHS soundness gap.** `WHERE <index_col> >= now() - interval '7d' LIMIT N` was incorrectly PERMITting early-stop because `is_temporal_expr` accepted `Expr::Now`/`Expr::Interval`/`Expr::TimestampArithmetic`; `is_pushed_temporal_predicate` precondition (c) requires `Expr::Literal(Literal::Timestamp)` RHS and was already correct, but no Red Gate existed to enforce the boundary. (1) RG-PSG-029 (`test_psg_relative_temporal_now_interval_suppresses_early_stop`) added to §Red Gate Tests table: END-TO-END via `QueryEngine::execute` from real SQL string; relative-temporal RHS → `is_pushed_temporal_predicate=false` → `has_client_side_where=true` → `fetch_limit=0` → early-stop SUPPRESSED; MUST FAIL against current code; optional `Expr::TimestampArithmetic` sub-case. (2) Frontmatter: `red_gate_tests: 34→35`. (3) BC-5.38.001 density check: 34→35 RGTs, `RG-PSG-001 through RG-PSG-029`, ≈3.78→≈3.89. (4) AC-008(c) Tests citation: extended to include RG-PSG-029. (5) §Architecture Compliance Rules `is_pushed_temporal_predicate` MUST anchor: `(Anchored: RG-PSG-021 through RG-PSG-024, RG-PSG-029)`. (6) Task 10 temporal-exemption soundness bullets: RG-PSG-029 authoring bullet added after RG-PSG-024. (7) Task 12 green-gate range: extended to include RG-PSG-029 with relative-temporal description. (8) Task 15 final gate range: `RG-PSG-001 through RG-PSG-029`. TD-VSDD-097: Dim-1 — no named twin for this story; CLEAR. Dim-2 — `red_gate_tests: 35` consistent in frontmatter, density paragraph (35/9≈3.89), Task 10 (RG-PSG-029 bullet added), Task 12 (range extended), Task 15 (range extended); `acceptance_criteria_count: 9` UNCHANGED (RG-PSG-029 closes gap in existing AC-008(c)); FULL. Dim-3 — RG-PSG-029 MUST anchored to AC-008(c) + `test_psg_relative_temporal_now_interval_suppresses_early_stop` + F-R16-P1-CRIT-001 in §Red Gate Tests, Task 10, §Architecture Compliance Rules; no unanchored MUSTs introduced; CLEAR. |
| 1.15 | 2026-08-27 | story-writer | **Round-16 remediation: AC-009(a) `any_early_stopped` chain spec-completion + multi-sensor coverage (RG-PSG-027/028, heuristic-rejection gate).** The round-16 implementer substituted `total_fetched_rows >= fetch_limit` for the real `any_early_stopped` propagation chain, producing wrong `is_truncated=true` on multi-sensor fan-out where sensors each return fewer rows than `fetch_limit` but their sum equals `fetch_limit` (none early-stopped). This version completes the spec: (1) §File Structure Requirements expanded: `crates/prism-sensors/src/adapter.rs` (define `FetchOutput { batches, any_early_stopped }` struct; change `SensorAdapter::fetch` return type); `crates/prism-sensors/src/fanout.rs` (add `FanOutResult.any_early_stopped: bool`; OR-aggregate in `fan_out()`); `crates/prism-query/src/engine.rs` (Step 6 `is_truncated = total_rows > limit \|\| any_early_stopped`); all-test-stub sweep (21 files, mechanical wrap `Ok(FetchOutput { batches, any_early_stopped: false })`); `crates/prism-spec-engine/src/pipeline.rs` note extended (add `PipelineResult.early_stopped: bool` field; set `early_stopped = true` before `break 'steps` in §D8.2 block); `crates/prism-bin/src/spec_driven_adapter.rs` note extended (return `FetchOutput { batches, any_early_stopped: pipeline_result.early_stopped }`); `crates/prism-query/src/materialization.rs` note extended (add `MaterializationOutput.any_early_stopped`; REMOVE heuristic). (2) RG-PSG-027 (`test_psg_multi_sensor_fanout_exact_limit_one_early_stopped_is_truncated_true`): 2-sensor `execute_integration_tests.rs` test; sensor1=40 rows (no early-stop), sensor2=10 rows (early-stopped at exact limit boundary); total=50=limit, any_early_stopped=true via OR-aggregation → is_truncated=true; gates the OR-aggregation path and exact-limit multi-sensor case. (3) RG-PSG-028 (`test_psg_multi_sensor_fanout_exact_total_no_early_stop_is_not_truncated`): 2-sensor `execute_integration_tests.rs` test plus wire-level assertion in `crates/prism-bin/tests/mcp_integration_tests.rs`; both sensors return 25 rows without early-stopping; total=50=limit, any_early_stopped=false → is_truncated=false; MUST FAIL with heuristic `total_fetched_rows >= fetch_limit` (50 >= 50 = true → wrong); wire assertion confirms `"is_truncated": false` in serialized MCP response. (4) Task 7 updated: `early_stopped = true` required before `break 'steps`; `PipelineResult.early_stopped: bool` field addition specified. (5) Task 10 extended: RG-PSG-027/028 authoring bullets. (6) Task 15 gate range extended →028. (7) Task 16 added: full chain wiring (`adapter.rs` + `fanout.rs` + `spec_driven_adapter.rs` + `materialization.rs` + `engine.rs`). (8) §Architecture Compliance Rules: heuristic-rejection MUST + chain-wiring MUST + OR-aggregation MUST added. (9) `red_gate_tests: 34`; density 34/9≈3.78. (10) Frontmatter: `crates_touched` extended with `prism-sensors`; `risk` comment updated. TD-VSDD-097: Dim-1 — no named twin for this story; CLEAR. Dim-2 — whole-artifact sweep: `red_gate_tests: 34` consistent in frontmatter, density paragraph (34/9≈3.78), Task 10 (RG-PSG-027/028 bullets added), Task 15 (range →028); §File Structure Requirements 5 updated/new rows + 1 test-stub sweep row; `acceptance_criteria_count: 9` UNCHANGED (no new ACs — RG-PSG-027/028 gate AC-009(a) via existing AC); `crates_touched` extended `prism-sensors` consistent with new §File Structure rows; FULL. Dim-3 — RG-PSG-027 MUST anchored to AC-009(a) + `test_psg_multi_sensor_fanout_exact_limit_one_early_stopped_is_truncated_true`; RG-PSG-028 MUST anchored to AC-009(a) + `test_psg_multi_sensor_fanout_exact_total_no_early_stop_is_not_truncated` + heuristic-rejection named; chain-wiring MUSTs anchored to ADR-060 §D8.9 + RG-PSG-027/028; no unanchored MUSTs; CLEAR. |
| 1.14 | 2026-08-27 | story-writer | **Wire-level MCP coverage gap closure: RG-PSG-026 + AC-009(d).** EC-11-092/093 (BC-2.11.001) contracts that `any_early_stopped`/`is_truncated` surface on the `prism_query` MCP tool response — an MCP-visible surface. Wire-shape assertion discipline (CLAUDE.md 2026-07-13) requires at least one test asserting on the serialized JSON. RG-PSG-025 reaches only `QueryEngine::execute` at the Rust-struct level (`QueryResult` is not `Serialize`) — wire-level gap confirmed genuinely missing. (1) Frontmatter: `red_gate_tests: 32` added (new field); version 1.13→1.14; input-hash b60edc0→fd3b8df. (2) AC-009: sub-section (d) added — wire-level MCP assertion for `is_truncated` in serialized `CallToolResult` JSON; exact-limit case (`any_early_stopped = true → is_truncated: true`) and temporal-WHERE suppression case (`any_early_stopped = false → is_truncated: false`); traces to BC-2.11.001 EC-11-092/093. (3) §Red Gate Tests: RG-PSG-026 (`test_psg_rg026_prism_query_wire_surfaces_truncation_signal`) added in `crates/prism-bin/tests/mcp_integration_tests.rs`; density recomputed 31→32 RGTs / 9 ACs ≈ 3.56. (4) §Tasks: RG-PSG-026 authoring bullet added to Task 10 (red-then-green preserved); Task 15 final gate range extended to RG-PSG-026. (5) Token Budget: story spec ~10,000→~10,500; MCP test file row added (~1,500); total ~32,500→~34,500. (6) §File Structure Requirements: MCP integration test file row added. TD-VSDD-097: Dim-1 — no named twin for this story; additive-only change; CLEAR. Dim-2 — whole-artifact sweep: `red_gate_tests: 32` (frontmatter), RG-PSG-026 row (§Red Gate Tests table, 32 rows), density paragraph (32/9≈3.56), Task 10 (PSG-026 bullet added), Task 15 (range →026), Token Budget (story spec + MCP file rows), §File Structure Requirements (MCP file row); `acceptance_criteria_count: 9` UNCHANGED (no new AC; sub-section extends existing AC-009); FULL. Dim-3 — RG-PSG-026 wire-shape MUST anchored to S-ENGINE-LIMIT-EARLY-STOP-001 AC-009(d) + named test `test_psg_rg026_prism_query_wire_surfaces_truncation_signal`; no unanchored MUSTs; CLEAR. |
| 1.13 | 2026-08-27 | story-writer | **Round-15 remediation: temporal-exemption soundness (EC-01-030..033), `any_early_stopped` truncation signal (EC-11-092/093), BC-2.11.001 trace addition, RG-PSG-019 desc fix, BC-2.16.002 v2.41 pin.** (1) Frontmatter: `traces_to` extended with `BC-2.11.001` (trace-only, resolves lens-C FINDING-2); `acceptance_criteria_count` 7→9; version 1.12→1.13; input-hash comment: ADR-060 v1.4→v1.5, BC-2.16.002 v2.40→v2.41. (2) §Behavioral Contracts table: BC-2.16.002 version pin v2.40→v2.41; description extended with §D8.9 temporal-soundness redesign (Filter-mode/Pipe-WHERE unconditional suppress; `is_pushed_temporal_predicate`; `FetchOutput`/`any_early_stopped` propagation chain; EC-01-030..033); BC-2.11.001 trace-only row added. (3) AC-008 added: temporal-exemption soundness (Filter-mode unconditional suppress; Pipe-WHERE unconditional suppress; SQL `is_pushed_temporal_predicate` with range-op + INDEX-column + `Literal::Timestamp` preconditions; EC-01-030..033; RG-PSG-021..024). (4) AC-009 added: `any_early_stopped` propagation chain and `is_truncated = (total_rows > limit) OR any_early_stopped` formula; Step 6 as SOLE owner of tool-level cap (EC-11-092/EC-11-093; RG-PSG-025). (5) §Red Gate Tests: RG-PSG-021..025 registered; density recomputed 26→31 RGTs / 9 ACs ≈ 3.44; RG-PSG-019 description corrected (PERMIT/SUPPRESS boundary; catch-all structurally guaranteed, NOT reachable by any test; SAP-3 rule-3); §Note reachability claims corrected for PSG-012/PSG-019. (6) §Architecture Compliance Rules: temporal-exemption soundness rules added (Filter/Pipe-WHERE unconditional suppress; `is_pushed_temporal_predicate` preconditions); `any_early_stopped` propagation chain rules added (`FetchOutput`; `is_truncated` formula; Step-6 sole-owner; anchored to RG-PSG-025). (7) §References: BC-2.11.001 EC-11-092/EC-11-093 trace entries added. (8) §Tasks: RG-PSG-021..025 authoring bullets added to Task 10; Task 12 green-gate range extended to RG-PSG-024; Task 15 final gate range extended to RG-PSG-025; implementer NOTE for module-doc Task-11 comment fix added to §Previous Story Intelligence. (9) §File Structure Requirements: plan_shape_gate_tests.rs and execute_integration_tests.rs notes updated. TD-VSDD-097: Dim-1 — no named twin for this story; CLEAR. Dim-2 — whole-artifact sweep: RG count 31 consistent in §Red Gate Tests table (31 rows), density paragraph (31/9≈3.44), Task 10 (RG-PSG-021..025 added), Task 15 (..025); `acceptance_criteria_count: 9` consistent with 9 ACs in body; `traces_to` carries BC-2.11.001 and BC-2.11.001 appears in §Behavioral Contracts trace note + §References; FULL. Dim-3 — RG-PSG-021..025 MUSTs anchored to story + named tests + ECs; `any_early_stopped` MUST anchored to RG-PSG-025 + EC-11-092/093; no unanchored MUSTs; CLEAR. |
| 1.12 | 2026-08-27 | story-writer | **Round-14 records-tier story fixes (OBS-1 dedicated impl task + F-R14-LOW-001 Expr-level conservative-default note).** (1) OBS-1 (SAC-1 task-structure hygiene): added dedicated Task 11 (Implementation — materialization result boundary) between old Task 10 (RG-PSG-020 test authoring) and old Task 11 (ast_is_reducing_plan implementation); trimmed "make it GREEN by..." implementation guidance from the Task 10 RG-PSG-020 authoring bullet; old Tasks 11–14 renumbered to Tasks 12–15. All "MUST FAIL before Task 11" updated to "MUST FAIL before Task 12"; "MUST PASS after Task 11" and "MUST PASS before AND after Task 11" updated to Task 12; positive-controls note updated. Red-then-green ordering preserved: Task 10 (test authoring) → Task 11 (truncation-signal impl) → Task 12 (gate impl). RG count UNCHANGED at 26; density UNCHANGED at 26/7≈3.71. (2) F-R14-LOW-001 (Expr-level conservative default): AC-007 Conservative Default description expanded to state the posture applies at ALL dispatch levels including the Expr-recursion level; `expr_contains_aggregate_or_window` terminal arm documented as `_ => true` (conservative SUPPRESS) with explicit leaf-variant enumeration; noted as defensive/design-level — no CASE variant exists today, no new RG added. Code stub in new Task 12 updated: `_ => false` → `_ => true` with corresponding comment. New §Architecture Compliance Rules entry added for the Expr-level conservative default invariant. TD-VSDD-097: Dim-1 — no named twin for this story; CLEAR. Dim-2 — whole-artifact task-ordinal sweep: "MUST FAIL before Task 12/12" updated consistently in §Red Gate Tests (19 occurrences), §Note on positive controls (2 occurrences), Task 10 body (1 cross-ref added); task headings 11–15 consistent; RG count 26 unchanged in §Red Gate Tests table, density paragraph, Task 10 list, Task 12 green-gate range, Task 15 final gate list; FULL. Dim-3 — no new MUSTs added beyond existing; Expr-level conservative default is design-level defensive with no new test anchor required (per-instructions: not reachable today); CLEAR. |
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
