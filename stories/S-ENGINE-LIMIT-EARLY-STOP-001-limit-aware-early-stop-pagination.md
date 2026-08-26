---
document_type: story
story_id: S-ENGINE-LIMIT-EARLY-STOP-001
title: "LIMIT-aware early-stop pagination — FetchContext.early_stop_limit field + execute_impl check + spec_driven_adapter wiring (ADR-060 §D8)"
level: "L4"
wave: xdome-wave-a
epic_id: E-XDOME-EXPANSION
priority: P0
status: draft
# BC status: BC-2.16.002 v2.36 active — LIMIT-Aware Early-Stop Pagination postcondition
# authored and anchored to this story ID. BC-2.16.015 v1.7 active — EC-016-015-007 and
# TV-BC-2.16.015-006 anchored to this story. Status remains draft until remove-uncertainty CLEAN.
producer: story-writer
timestamp: "2026-08-26T00:00:00Z"
version: "1.1"
modified: "2026-08-26"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.015-claroty-vulnerabilities-table.md"
  - ".factory/specs/architecture/decisions/ADR-060-limit-aware-early-stop-pagination.md"
input-hash: "07db7cb"
# input-hash: recomputed 2026-08-26 after §Authority sweep (v1.1) — BC-2.16.002 v2.37 + BC-2.16.015 v1.7 + ADR-060 v1.1
traces_to: ["BC-2.16.002", "BC-2.16.015"]
points: 8
estimated_days: 2
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-16 (Spec Engine) owns this story's scope because the primary implementation
#     is in `crates/prism-spec-engine/src/pipeline.rs` — `FetchContext` struct and
#     `PipelineExecutor::execute_impl` loop. SS-16 is the canonical owner of
#     prism-spec-engine per ARCH-INDEX Subsystem Registry.
#   SS-01 (Sensor Adapters) owns this story's scope because `SpecDrivenSensorAdapter::fetch`
#     in `crates/prism-bin/src/spec_driven_adapter.rs` is the sole production caller of
#     `FetchContext::new` and the wiring point that maps `params.limit` to `early_stop_limit`.
#     SS-01 governs the outbound sensor HTTP adapter surface per ARCH-INDEX.
target_module: prism-spec-engine
crates_touched: [prism-spec-engine, prism-bin]
# crates_touched:
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
  # BC-2.16.002 v2.36 — §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)":
  # PipelineExecutor::execute_impl stops at complete page boundaries when early_stop_limit
  # satisfied; truncated=false (reserved for DI-019); DataFusion trims post-fetch;
  # OffsetLimit and CursorToken only; D8.5 ORDER BY limitation documented.
  - BC-2.16.015
  # BC-2.16.015 v1.7 — EC-016-015-007 (LIMIT 1 early-stop; 1 page fetched; truncated=false)
  # and TV-BC-2.16.015-006 (LIMIT 1 single-page test vector).
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
acceptance_criteria_count: 6
risk: MEDIUM
# Risk justification:
#   FetchContext::new signature expansion is a BREAKING CHANGE for all callers.
#   ~15 in-file test sites + ~14 integration test files must each pass `None`.
#   TD-VSDD-060 sibling-sweep is mandatory and explicitly enumerated in Tasks.
#   The `#[non_exhaustive]` attribute on FetchContext prevents struct-literal construction
#   outside the crate, so the only callers affected are those using FetchContext::new —
#   enumerated in the crates_touched comment above and in Task 5.
#   ADR-060 §D8.1 phrasing discrepancy noted (see AC-006 and Task 8).
assumption_validations:
  - claim: "Expanding FetchContext::new (constructor of a #[non_exhaustive] struct) by one parameter is source-compatible in-workspace when all callers are updated in-tree; no API-surface gate beyond the non-exhaustive audit is triggered."
    verdict: "CONFIRMED (remove-uncertainty 2026-08-26). Ground-truthed against code: crates/prism-spec-engine/src/pipeline.rs FetchContext is #[non_exhaustive] #[derive(Debug, Clone)] with new(client_id, query_filters). Adding a parameter to `new` is a signature change requiring all callers updated (breaking only if external callers existed — none do; #[non_exhaustive] blocks external struct-literal construction and the crate is workspace-internal). Adding a FIELD to an already-registered #[non_exhaustive] type introduces NO new symbol, so EXPECTED_SYMBOLS in scripts/check-non-exhaustive-per-symbol.py needs no update. Story AC-001 and §Architecture Compliance Rules state this correctly."
  - claim: "No DataFusion research/dependency is needed: LIMIT is available as QueryParams.limit: u64 at the adapter; ADR-060 §D8.1 previously said 'extract from DataFusion physical plan' — corrected to match this reality in ADR-060 v1.1."
    verdict: "CONFIRMED (remove-uncertainty 2026-08-26). Ground-truthed: QueryParams.limit: u64 field defined in prism-sensors sensor adapter module; the 0 = no-limit sentinel corroborated by materialization options.limit.unwrap_or(0) in prism-query materialization module. SpecDrivenSensorAdapter::fetch receives params.limit pre-extracted; no physical-plan inspection required. ADR-060 v1.1 corrected §D8.1 to match this implementation; §Authority note updated accordingly (v1.1 sweep, 2026-08-26); discrepancy RESOLVED. No DataFusion API research required."
risk_mitigations:
  - "FetchContext::new signature expansion is fully swept in Task 5 (TD-VSDD-060); all in-file + integration callers pass None. Verified constructor shape against live code 2026-08-26."
---

# S-ENGINE-LIMIT-EARLY-STOP-001: LIMIT-Aware Early-Stop Pagination — FetchContext Field, execute_impl Check, and spec_driven_adapter Wiring

## Authority

**BC-2.16.002 v2.36 §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)"** is
the primary governing contract. Read this postcondition in full before implementing. It
specifies: `FetchContext.early_stop_limit: Option<usize>`; check placement IMMEDIATELY AFTER
DI-019 in `PipelineExecutor::execute_impl`; `truncated` NOT set on early-stop; `OffsetLimit`
and `CursorToken` only; `None` = unchanged full pagination; D8.5 ORDER BY limitation text.
Also read the atomicity-reconciliation scope clause in the partial-record-discard postcondition
(amended by ADR-060 §Atomicity Reconciliation) confirming that early-stop is COMPATIBLE with
the "all-or-nothing" error-path invariant.

**BC-2.16.015 v1.7 §Edge Cases EC-016-015-007** and **§Canonical Test Vectors TV-BC-2.16.015-006**
are the Claroty-specific anchors: `LIMIT 1` against page_size=1000 triggers early-stop after 1
page; `PipelineResult.truncated=false`; DataFusion trims to 1 row.

**ADR-060 §D8** is the decision. Read §D8.1 through §D8.5 (FetchContext field, execute_impl
check placement, post-break semantics, applicable pagination modes, ORDER BY documentation).
§D8.6 (timeout_secs overlay wiring) is DEFERRED to S-ENGINE-TIMEOUT-OVERLAY-WIRE-001.

**ADR-060 §D8.1 phrasing discrepancy — RESOLVED (ADR-060 v1.1):** §D8.1 previously said
"Callers (`spec_driven_adapter.rs`) extract the LIMIT from the DataFusion physical plan and
pass it," which was imprecise. ADR-060 v1.1 has corrected §D8.1 to specify reading
`QueryParams.limit: u64` (0 = no limit); no DataFusion physical-plan inspection is required.
The wiring `if params.limit == 0 { None } else { Some(params.limit as usize) }` is exactly
what the corrected ADR describes. The implementation and the ADR now AGREE; no further
architect correction is outstanding.

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
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | v2.36 | §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)": FetchContext field, execute_impl check placement, truncated=false semantics, applicable pagination modes, D8.5 ORDER BY limitation. Atomicity-reconciliation scope clause. |
| BC-2.16.015 | Claroty xDome Vulnerability Findings Table | v1.7 | EC-016-015-007 (LIMIT 1 early-stop, 1 page, truncated=false); TV-BC-2.16.015-006 (LIMIT 1 single-page test vector, ≤1 HTTP POST request) |

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
**Test:** `test_BC_2_16_002_early_stop_fires_after_one_page_when_limit_satisfied` (Some case)

### AC-004: DI-019 10K truncation check fires BEFORE early-stop; `truncated = true` when DI-019 fires (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop §D8 ordering; DI-019 unchanged)

When `all_records.len() >= MAX_PIPELINE_RECORDS` (10000) AND `early_stop_limit = Some(N)` for
some N > MAX_PIPELINE_RECORDS, the DI-019 block fires first (it precedes the early-stop block
in the source), sets `truncated = true`, and breaks. The early-stop block is never reached.
DI-019 behavior is UNCHANGED by this story.

**Test:** `test_BC_2_16_002_early_stop_di019_fires_before_early_stop_check`

### AC-005: `SpecDrivenSensorAdapter::fetch` maps `params.limit` to `early_stop_limit`; passes to `FetchContext::new` (traces to BC-2.16.002 postcondition — LIMIT-Aware Early-Stop; BC-2.16.015 EC-016-015-007 and TV-BC-2.16.015-006)

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

**Test:** Compilation success (`just check` gate); `test_BC_2_16_002_early_stop_all_existing_callers_compile_with_none` (a single compilation-sentinel test confirming the updated test files compile)

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_002_early_stop_fetch_context_new_stores_early_stop_limit` | Unit — prism-spec-engine, `FetchContext::new` | AC-001: `FetchContext::new("id", HashMap::new(), Some(5))` stores `early_stop_limit = Some(5)`; `FetchContext::new("id", HashMap::new(), None)` stores `early_stop_limit = None`. Fails before `early_stop_limit` field is added. |
| RG-002 | `test_BC_2_16_002_early_stop_pipeline_stops_without_setting_truncated` | Integration — prism-spec-engine, wiremock multi-page mock (page_size=10); `early_stop_limit=Some(1)` | AC-002: 1 mock request issued; `PipelineResult.truncated = false`; 10 records returned (full page); DataFusion trims downstream. Fails before early-stop check is added to `execute_impl`. |
| RG-003 | `test_BC_2_16_002_early_stop_none_fetches_all_pages` | Integration — prism-spec-engine, wiremock (3 pages, page_size=10); `early_stop_limit=None` | AC-003 None case: all 3 pages fetched (3 mock requests); `truncated = false`; 30 records returned. Passes before and after (no early-stop when None). Regression sentinel. |
| RG-004 | `test_BC_2_16_002_early_stop_di019_fires_before_early_stop_check` | Unit — prism-spec-engine, pipeline internal; inject 10001 records on first page with `early_stop_limit=Some(5)` | AC-004: DI-019 check fires; `truncated = true`; records truncated to 10000. Fails if early-stop check is placed BEFORE DI-019 check (ordering validation). |
| RG-005 | `test_BC_2_16_002_early_stop_spec_driven_adapter_maps_params_limit_to_early_stop_limit` | Integration — prism-bin, wiremock claroty-style mock (page_size=1000, 1 record returned); `params.limit=1` | AC-005: `FetchContext` constructed with `early_stop_limit=Some(1)`; 1 mock request issued; `truncated=false`. Fails before AC-005 wiring. Also tests `params.limit=0 → None`. |
| RG-006 | `test_BC_2_16_002_early_stop_claroty_page_size_1000_limit_1_single_page` | Integration — prism-spec-engine or prism-bin, wiremock claroty-style (page_size=1000, 3 pages available, each 1000 records); `early_stop_limit=Some(1)` | BC-2.16.015 EC-016-015-007 / TV-BC-2.16.015-006: exactly 1 mock request issued (NOT 3); `truncated=false`; result has 1000 records pre-DataFusion-trim. This is the concrete claroty_vulnerabilities behavioral proof. |

**BC-5.38.001 density check:** 6 Red Gate tests (RG-001 through RG-006, where RG-003 is a regression sentinel that passes in both states) / 6 acceptance criteria = 1.0 ≥ 0.5 threshold. PASS.

**Note on RG-003 semantics:** RG-003 (`early_stop_limit=None` fetches all pages) passes BOTH before and after the implementation because `None` must preserve the current behavior. It is a regression gate confirming the existing full-pagination path is not broken.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `FetchContext` struct (field addition) | `crates/prism-spec-engine/src/pipeline.rs §FetchContext` | Pure (data struct; no I/O) |
| `FetchContext::new` (signature expansion) | `crates/prism-spec-engine/src/pipeline.rs §FetchContext::new` | Pure |
| Early-stop check in `execute_impl` | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute_impl` | Effectful (HTTP pagination loop; the check is a branch point within the loop) |
| `params.limit → early_stop_limit` mapping | `crates/prism-bin/src/spec_driven_adapter.rs §SpecDrivenSensorAdapter::fetch` | Effectful (production sensor adapter fetch path) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; FetchContext, PipelineExecutor)
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (prism-bin; spec_driven_adapter)
- ADR-060 §D8 — FetchContext field, execute_impl check placement, truncated semantics, pagination modes
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

The pure-core / effectful-I/O boundary is respected: the early-stop policy is a pure predicate
threaded through `FetchContext` (data) and evaluated at a complete-page boundary; the only
effectful behavior change is fetching FEWER pages, never adding new I/O.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `early_stop_limit = Some(0)` | Check `all_records.len() >= 0` fires immediately (0 records always >= 0); loop exits after 0 records. Not a practical input (params.limit=0 maps to None), but the check is mathematically sound. |
| EC-002 | `early_stop_limit = Some(N)` where N > total available records | Early-stop never fires; pagination loop completes normally when API signals exhaustion (empty page or null cursor). `truncated = false`. |
| EC-003 | `early_stop_limit = Some(N)` where N exactly equals `page_size` | Early-stop fires at end of first page (exactly N records). 1 page request issued. |
| EC-004 | DI-019 cap (10000) reached before `early_stop_limit` | DI-019 fires first; `truncated = true`; early-stop block NOT reached. Both checks present simultaneously; DI-019 order-precedence preserved. |
| EC-005 | `PaginationConfig::None` (single-page fetch) with `early_stop_limit = Some(1)` | Pagination loop body executes once then breaks at `Some(PaginationConfig::None) \| None => break`. Early-stop check fires at end of the single page (after DI-019 check). Effectively a no-op since pagination already terminates after one page. |
| EC-006 | `ORDER BY` combined with `LIMIT` in query | DataFusion applies ORDER BY on the early-stopped result. Records are in API-declared order (not globally sorted top N). Documented in BC-2.16.002 D8.5 limitation text and in story §Background (no implementation impact). |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~7,500 |
| BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop section + §Atomicity Reconciliation clause | ~2,000 |
| BC-2.16.015 EC-016-015-007 + TV-BC-2.16.015-006 | ~800 |
| ADR-060 §D8 (full) | ~3,500 |
| `crates/prism-spec-engine/src/pipeline.rs` (FetchContext struct + execute_impl loop region) | ~4,000 |
| `crates/prism-bin/src/spec_driven_adapter.rs` (fetch function FetchContext::new call site region) | ~2,500 |
| ~14 integration test files (skimmed for FetchContext::new call sites; read only affected lines) | ~5,000 |
| ~15 in-file test sites (pipeline.rs #[cfg(test)] FetchContext::new calls) | ~3,000 |
| **Total estimate** | **~28,300 tokens** |

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
  after the DI-019 block (after `break 'steps;` on line ~564):
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

- [ ] **Task 8 (Implementation — spec_driven_adapter wiring):** In `SpecDrivenSensorAdapter::fetch`
  in `crates/prism-bin/src/spec_driven_adapter.rs`, insert immediately before
  `let context = FetchContext::new(...)`:
  ```rust
  // ADR-060 §D8: map params.limit to early_stop_limit for LIMIT-aware early-stop pagination.
  // params.limit == 0 means "no LIMIT clause" (QueryParams convention); map to None → unchanged behavior.
  // NOTE: ADR-060 §D8.1 says "extract from DataFusion physical plan" — this is imprecise.
  // params.limit is already pre-extracted into QueryParams.limit before this call. Flag for
  // a minor architect correction to ADR-060 §D8.1 prose (no ADR edit by implementer).
  let early_stop_limit = if params.limit == 0 { None } else { Some(params.limit as usize) };
  ```
  Update the `FetchContext::new` call to:
  ```rust
  let context = FetchContext::new(self.sensor_spec.org_slug.clone(), query_filters, early_stop_limit);
  ```
  After editing: run `just iter prism-bin` — RG-005 MUST turn GREEN.

- [ ] **Task 9 (Integration sweep — update all remaining callers):** Run `just check --no-fail-fast`
  across the full workspace. All integration test files listed in `crates_touched` that were
  updated in Task 5 should compile. If any callers were missed in Task 5, find them now via the
  compile errors and update each to pass `None`. Run `just iter prism-spec-engine` to confirm
  all pipeline.rs-adjacent tests pass. Run `just iter prism-bin` to confirm all prism-bin tests
  pass.

- [ ] **Task 10 (Red Gate — tests for RG-005 and RG-006):** Write RG-005:
  `test_BC_2_16_002_early_stop_spec_driven_adapter_maps_params_limit_to_early_stop_limit`
  in `crates/prism-bin/tests/` or adjacent integration test file. Use wiremock mock with
  page_size=1000, 2 pages, `params.limit=1`. Assert mock received exactly 1 request.
  Assert `truncated=false`. Also test `params.limit=0 → None` (3 pages all fetched).

  Write RG-006:
  `test_BC_2_16_002_early_stop_claroty_page_size_1000_limit_1_single_page`
  in the same file. Use wiremock mock with `page_size=1000`, 3 pages of 1000 records each,
  `early_stop_limit=Some(1)`. Assert exactly 1 request issued. `truncated=false`.
  Records count = 1000 (first page; DataFusion trims to 1 downstream).
  This is the direct test vector for BC-2.16.015 TV-BC-2.16.015-006.

- [ ] **Task 11 (SAP-1 self-check):** Confirm that no new `tracing::*!(event_type = ...)` emissions
  are added. BC-2.16.002 v2.36 SAP-1 declaration states: "ADR-060 introduces NO new `event_type`
  values; the existing `pipeline_truncated` WARN event (DI-019 cap only) is NOT altered; catalog
  count unchanged at 96." The early-stop branch has no emission — this is intentional and
  documented.

- [ ] **Task 12 (Final gate):** Run `just check` (full workspace). Confirm all non-`#[ignore]`
  Red Gate tests pass: RG-001, RG-002, RG-003, RG-004, RG-005, RG-006. Confirm
  `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` does NOT need updating
  (no new `#[non_exhaustive]` type is introduced). Confirm no new `unwrap()`/`expect()` in
  production code paths. After `just check` passes, hold for story-level holdout gate before
  pushing to origin.

## Previous Story Intelligence

1. **S-DEMO-CLAROTY-PAGINATION-001 (merged):** Added POST-body OffsetLimit pagination for Claroty.
   This story builds on that pagination infrastructure — the `OffsetLimit` branch in
   `execute_impl` is the exact branch where the early-stop check fires.

2. **S-ADR058-OCSF-ROUTING-001 / S-ADR058-OCSF-COERCION-001 (merged):** These stories modified
   pipeline.rs. Read the current pipeline.rs to confirm the DI-019 check is at the expected
   location (around line 551-565) before inserting the early-stop check. Do not rely on line
   numbers — use the `// AC-8 / DI-019` comment as the anchor (TD-VSDD-091).

3. **`FetchContext::new` call-site distribution (confirmed from codebase):** The in-file pipeline.rs
   test sites use the helper `default_context()` (line ~3667) which calls `FetchContext::new(OrgSlug::new("test-org"), HashMap::new())`. This helper must ALSO be updated to pass `None`. The integration
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

- BC-2.16.002 v2.36 §Postconditions "LIMIT-Aware Early-Stop Pagination (ADR-060 §D8)" — governing postcondition
- BC-2.16.002 v2.36 §Postconditions "Partial-record discard" atomicity-reconciliation scope clause
- BC-2.16.015 v1.7 EC-016-015-007 — Claroty LIMIT 1 early-stop edge case
- BC-2.16.015 v1.7 TV-BC-2.16.015-006 — LIMIT 1 single-page test vector
- ADR-060 §D8 — FetchContext field, execute_impl check, truncated semantics, modes, ORDER BY, timeout deferral
- ADR-060 §Atomicity Reconciliation — "atomic" = error-path invariant; early-stop is compatible
- `crates/prism-spec-engine/src/pipeline.rs §FetchContext` — struct + constructor to modify
- `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute_impl` — DI-019 block to extend
- `crates/prism-bin/src/spec_driven_adapter.rs §SpecDrivenSensorAdapter::fetch` — production wiring point

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-08-26 | story-writer | §Authority sweep — ADR-060 v1.1 correction propagated; §D8.1 phrasing discrepancy marked RESOLVED; assumption_validations second entry updated to remove open-discrepancy framing (F-LENS3-OBS-001 closure). ACs, RG list, and tasks unchanged. |
| 1.0 | 2026-08-26 | story-writer | Initial authoring — ADR-060 §D8 implementation story. 6 ACs, 6 RGTs, density 1.0. SAC-1 compliant. TD-VSDD-060 sibling-sweep fully enumerated in Task 5 with all ~14 integration test file names. ADR-060 §D8.1 phrasing discrepancy noted in §Authority and Task 8. |
