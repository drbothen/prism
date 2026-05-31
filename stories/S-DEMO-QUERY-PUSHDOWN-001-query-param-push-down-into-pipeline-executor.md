---
document_type: story
story_id: S-DEMO-QUERY-PUSHDOWN-001
title: "prism-spec-engine: Thread QueryParams push-down (limit/cursor/time-window) into PipelineExecutor via FetchContext"
wave: wave-5-e-demo-fidelity
epic_id: E-DEMO
priority: P2
status: draft
version: "1.0"
level: "L3"
producer: story-writer
revised_by: null
timestamp: "2026-05-31T12:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns SpecDrivenSensorAdapter::fetch() which receives QueryParams
#     from callers; this story threads those params through to the actual HTTP request builder.
#   SS-16 (Spec Engine) owns PipelineExecutor and FetchContext; the push-down wiring lives
#     inside PipelineExecutor::execute() → build_request() and the FetchContext struct.
crates_touched: [prism-spec-engine, prism-bin]
target_module: prism-spec-engine
capabilities: [CAP-015]
behavioral_contracts:
  - BC-2.01.013  # SpecDrivenSensorAdapter Pagination/Push-Down Scope Clause: this story
                 # implements the deferred push-down optimization referenced in D-924.
                 # After this story, fetch() does translate limit/cursor/time-window into
                 # sensor-native API request parameters.
  - BC-2.11.005  # Ephemeral Materialization: push-down affects how materialization collects
                 # data from sensor APIs. After this story, the query-plan push-down filters
                 # propagated in QueryPlan are honored at the API-request level.
  - BC-2.11.007  # Sensor Filter Push-Down: push-down is an optimization only — the query
                 # result MUST be identical whether or not push-down occurs (invariant).
                 # This story implements the optimization while preserving that invariant.
# BC status: pending PO authorship for any new push-down-specific BC clauses.
# BC-2.01.013 and BC-2.11.007 are the primary contracts; PO may author a dedicated
# BC for FetchContext field additions if warranted before this story flips to ready.
verification_properties:
  - VP-031  # Required column enforcement — push-down correctness test coverage
depends_on:
  - S-DEMO-001   # Must merge first: SpecDrivenSensorAdapter and PipelineExecutor wiring
                 # delivered by S-DEMO-001 are the extension points this story modifies.
                 # Cannot thread FetchContext params into a pipeline that doesn't exist yet.
blocks: []
# Dependency anchor justifications:
#   depends_on S-DEMO-001: S-DEMO-001 delivers SpecDrivenSensorAdapter::fetch(), PipelineExecutor::execute(),
#     and build_request(). This story adds FetchContext fields and threads them through those
#     exact entry points. Implementing push-down before those entry points exist is impossible.
points: 5
# Points justification:
#   - FetchContext struct additions (cursor, limit, time_window fields): ~0.5 pts
#   - SpecDrivenSensorAdapter::fetch() signature change to accept/extract QueryParams: ~0.5 pts
#   - PipelineExecutor::execute() → build_request() param threading: ~1 pt
#   - Per-sensor API translation (4 sensors — CrowdStrike FQL limit/time; Cyberint POST body
#     time-range; Claroty POST body limit/offset; Armis AQL LIMIT/time-filter): ~1.5 pts
#   - Result-equivalence validation tests (BC-2.11.007 invariant): ~1 pt
#   - Edge case tests (limit=0, no cursor, time-window missing start): ~0.5 pts
#   Total: 5 points (~1.5 days focused TDD work)
estimated_days: 2
risk: MEDIUM
# Risk justification: The primary risk is per-sensor API syntax for push-down translation.
# Each sensor has a different query syntax (FQL vs POST-body vs AQL). Partial push-down
# (only some fields supported on a given sensor) must degrade gracefully without breaking
# correctness. BC-2.11.007 invariant — result must be identical with or without push-down —
# is the safety net: any push-down implementation that changes results is a bug.
acceptance_criteria_count: 6
red_gate_tests: 4
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Result-equivalence invariant (BC-2.11.007): every push-down test MUST be paired with
    a result-equivalence assertion — the same query with push-down and without push-down
    must produce identical result sets (modulo row order). This is the regression gate."
  - "Partial push-down: if a sensor API does not support a given param (e.g., cursor not
    supported), the FetchContext field is silently ignored for that sensor and DataFusion
    applies the filter post-materialization. Never propagate unsupported params as API
    query strings — this causes 400 errors from the sensor API."
  - "FetchContext is additive: existing fields on FetchContext are not changed; only new
    optional fields are added (cursor: Option<String>, limit: Option<u32>, start_time:
    Option<DateTime<Utc>>, end_time: Option<DateTime<Utc>>). All existing callers continue
    to work with Default::default() FetchContext."
inputs:
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-spec-engine/src/fetch_context.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - "crates/prism-sensors/src/traits.rs"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.005-ephemeral-materialization.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md"
  - ".factory/stories/S-DEMO-001-spec-driven-sensor-adapter-and-boot-step-9a.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-QUERY-PUSHDOWN-001 v1.0 — prism-spec-engine: Query-Param Push-Down into PipelineExecutor

**Story ID:** S-DEMO-QUERY-PUSHDOWN-001
**Status:** draft
**Version:** v1.0
**Wave:** wave-5-e-demo-fidelity
**Priority:** P2
**Points:** 5

---

## Origin

Deferred from S-DEMO-001 v1.5 per BC-2.01.013 v1.8 Pagination/Push-Down Scope Clause (D-924).

S-DEMO-001 established that `SpecDrivenSensorAdapter::fetch()` returns ALL pages bounded only
by `MAX_PAGES_PER_STEP` / `MAX_REQUESTS_PER_PIPELINE`, and that query-param push-down
(`limit`, `cursor`, `start_time`, `end_time`) is explicitly OUT OF SCOPE for that story.
DataFusion already applies `LIMIT` post-materialization, so correctness holds without
push-down. This story implements the push-down optimization.

**Why P2 (non-blocking):** The absence of push-down does not affect query correctness
(BC-2.11.007 invariant). The demo works correctly without it. Push-down is a performance
optimization that reduces data transferred from sensor APIs. It should be delivered after the
core demo infrastructure is proven working.

---

## Narrative

As the Prism query engine, I want `limit`, `cursor`, `start_time`, and `end_time` from the
query caller's parameters threaded from `SpecDrivenSensorAdapter::fetch()` through
`FetchContext` and into `PipelineExecutor::build_request()` so that sensor API requests carry
native query filters, reducing data transferred and improving response latency — without
changing query results.

---

## Story-Level Goal

After this story merges:
1. `FetchContext` gains four optional fields: `cursor: Option<String>`, `limit: Option<u32>`,
   `start_time: Option<DateTime<Utc>>`, `end_time: Option<DateTime<Utc>>`.
2. `SpecDrivenSensorAdapter::fetch()` extracts these from the query context and passes them
   into `PipelineExecutor::execute()` via `FetchContext`.
3. `PipelineExecutor::build_request()` inspects `FetchContext` and translates non-None fields
   into sensor-native API request parameters:
   - CrowdStrike: `filter` FQL time range (`created_timestamp:>'<start_time>'`) + `limit` param
   - Cyberint: POST body `from_date` / `to_date` + `page_size` fields
   - Claroty: POST body `limit` / `offset` fields (cursor-based via offset)
   - Armis: AQL `timeFrame` param + `maxResults` field
4. When a sensor API does not support a given parameter, it is silently ignored (not an error).
5. All existing callers pass `FetchContext::default()` and are unaffected.
6. Push-down does not change query results (BC-2.11.007 invariant verified by tests).

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication (Pagination/Push-Down Scope Clause — this story implements the deferred push-down feature) |
| BC-2.11.005 | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable |
| BC-2.11.007 | Sensor Filter Push-Down — push-down is optimization only; result must be identical with or without push-down |

---

## Acceptance Criteria

### AC-001: FetchContext carries push-down fields (cursor, limit, start_time, end_time)
Given: `FetchContext` is the struct passed to `PipelineExecutor::execute()`.
When: AC-001 is complete.
Then:
(a) `FetchContext` has four new optional fields: `cursor: Option<String>`, `limit: Option<u32>`,
`start_time: Option<DateTime<Utc>>`, `end_time: Option<DateTime<Utc>>`.
(b) `FetchContext::default()` populates all four as `None` (existing callers unaffected).
(c) The struct derives or implements `Default`.
(traces to BC-2.01.013 Pagination/Push-Down Scope Clause — this AC closes the "FetchContext
field additions" prerequisite for push-down wiring)
Red Gate test: `test_BC_2_01_013_fetch_context_push_down_fields_default_to_none`

### AC-002: CrowdStrike — limit and time-window pushed into FQL filter
Given: A `FetchContext` with `limit = Some(50)` and `start_time = Some(t)` is passed to
`PipelineExecutor::execute()` for a CrowdStrike sensor.
When: `build_request()` constructs the CrowdStrike HTTP request.
Then: The request carries `limit=50` as a query parameter AND the `filter` FQL string includes
`created_timestamp:>'<t as ISO8601>'`. An empty `FetchContext` (all None) produces neither.
(traces to BC-2.11.007 postcondition — push-down translation to sensor-native syntax)
Red Gate test: `test_BC_2_11_007_crowdstrike_limit_and_time_window_pushed_to_fql`

### AC-003: Cyberint — time-window pushed into POST body
Given: A `FetchContext` with `start_time = Some(t0)` and `end_time = Some(t1)` is passed for
a Cyberint sensor (POST-body API).
When: `build_request()` constructs the Cyberint POST body.
Then: The POST body JSON includes `from_date: "<t0 as ISO8601>"` and `to_date: "<t1 as ISO8601>"`.
A `FetchContext` with all-None values produces no `from_date`/`to_date` fields.
(traces to BC-2.11.007 postcondition)

### AC-004: Claroty — limit pushed into POST body; Armis — limit and time-frame pushed into AQL
Given: A `FetchContext` with `limit = Some(100)` for Claroty; a `FetchContext` with
`limit = Some(200)` and `start_time = Some(t)` for Armis.
When: `build_request()` constructs each sensor's request.
Then: Claroty POST body includes `limit: 100`. Armis AQL request includes `maxResults: 200`
and `timeFrame` or equivalent time-filter consistent with the Armis AQL spec.
(traces to BC-2.11.007 postcondition)

### AC-005: Push-down does not change query results (BC-2.11.007 result-equivalence invariant)
Given: The same PrismQL query is executed against the DTU clone (a) with push-down params
set (limit=20, start_time) in FetchContext, and (b) with FetchContext default (no push-down).
When: Both executions complete.
Then: The result set returned by (a) is a subset of the result set returned by (b), consistent
with the time-window and limit applied post-materialization. No row appears in (a) that was
not also in (b) for the same time range — push-down must not fabricate or drop rows beyond
what the limit/time-window specify.
This AC validates the BC-2.11.007 invariant: "push-down is an optimization only; the query
result must be identical whether or not push-down occurs."
(traces to BC-2.11.007 invariant — result-equivalence)
Red Gate test: `test_BC_2_11_007_push_down_result_equivalence_invariant`

### AC-006: Unsupported push-down param silently ignored (no API error)
Given: A `FetchContext` with `cursor = Some("page-token-abc")` is passed to a sensor that does
not support cursor-based pagination (e.g., the sensor uses offset-only pagination and has no
cursor field in its API).
When: `build_request()` runs for that sensor.
Then: The HTTP request does NOT include the cursor value as a query param or body field.
No error is returned; the pipeline continues normally. DataFusion handles post-materialization
filtering.
(traces to BC-2.01.013 Pagination/Push-Down Scope Clause — graceful degradation for
unsupported params)

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `FetchContext` additions MUST be `Option<T>` with `Default::default() = None` | All existing callers must be unaffected | Verify: `cargo build` succeeds with no changes to existing callers after FetchContext change |
| Push-down params MUST NOT change query results (BC-2.11.007 invariant) | BC-2.11.007 invariant | Red Gate test AC-005 validates result-equivalence |
| Unsupported params MUST be silently ignored, not propagated as API query strings | ADR-028 §D — sensor API fidelity; sending unsupported params causes 400 errors | No API 400 errors in test suite for FetchContext with unsupported fields |
| `FetchContext` lives in `prism-spec-engine` | SS-16 owns the pipeline | Do NOT move FetchContext to prism-bin or prism-sensors |
| `SpecDrivenSensorAdapter::fetch()` extracts QueryParams and builds FetchContext | SS-01/SS-16 boundary | The extraction happens in prism-bin (adapter), the consumption in prism-spec-engine (pipeline) |
| Push-down translation is per-sensor-auth-type-aware | Each sensor has different API syntax | CrowdStrike FQL / Cyberint POST-body / Claroty POST-body / Armis AQL — see Story-Level Goal §3 |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `prism-spec-engine` (workspace) | current workspace path | FetchContext struct, PipelineExecutor::build_request amendment |
| `prism-bin` (workspace) | current workspace path | SpecDrivenSensorAdapter::fetch() QueryParams extraction |
| `chrono` | workspace version | DateTime<Utc> fields in FetchContext |
| `serde_json` | workspace version | POST body field injection for Cyberint/Claroty |
| `reqwest` | workspace version | Query param injection for CrowdStrike/Armis |

Version source: workspace `Cargo.toml` `[dependencies]` table. Do not pin versions independently.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-spec-engine/src/fetch_context.rs` (or `pipeline.rs` if inline) | MODIFY | Add `cursor`, `limit`, `start_time`, `end_time` optional fields to `FetchContext` |
| `crates/prism-spec-engine/src/pipeline.rs` | MODIFY | `build_request()` reads FetchContext push-down fields; per-sensor translation |
| `crates/prism-bin/src/spec_driven_adapter.rs` | MODIFY | `fetch()` extracts push-down params from query context and populates FetchContext |
| `crates/prism-spec-engine/src/pipeline/tests.rs` (or inline `#[cfg(test)]`) | MODIFY | Add Red Gate tests for AC-001, AC-002, AC-005 (result-equivalence) |

---

## Tasks

1. **Read** `crates/prism-spec-engine/src/pipeline.rs` — locate `FetchContext` definition and
   `build_request()` function signature; understand per-sensor request construction path.
2. **Read** `crates/prism-bin/src/spec_driven_adapter.rs` (post-S-DEMO-001 implementation) —
   understand `SpecDrivenSensorAdapter::fetch()` signature and how it calls `PipelineExecutor`.
3. **Read** `crates/prism-sensors/src/traits.rs` — confirm `SensorAdapter::fetch()` signature
   and what query context / params are available at call time.
4. **Read** BC-2.11.007 §Postconditions — review push-down translation tables for each sensor
   to understand the expected per-sensor API parameter names and syntax.
5. **Amend** `FetchContext` — add four optional fields with `#[derive(Default)]` or manual
   `Default` impl ensuring all fields are `None` by default.
6. **Write stub** — stub out `build_request()` push-down translation with `todo!()` bodies
   for each sensor (Red Gate setup).
7. **Write Red Gate tests** — AC-001, AC-002, AC-005 test names must fail before implementation.
8. **Implement** `build_request()` push-down translation for all 4 sensors per the dispatch
   table in Story-Level Goal §3. Use `if let Some(v) = fetch_ctx.field` pattern — silently
   skip when `None`.
9. **Implement** `SpecDrivenSensorAdapter::fetch()` changes — extract push-down fields from
   query context and populate `FetchContext` before calling `PipelineExecutor::execute()`.
10. **Verify result-equivalence** — write AC-005 test using DTU clone: same query with and
    without push-down must return equivalent results.
11. **Run** `just iter prism-spec-engine` and `just iter prism-bin` — all Red Gate tests GREEN.
12. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence

- **S-DEMO-001** (depends_on, must merge first): Delivers `SpecDrivenSensorAdapter`,
  `PipelineExecutor::execute()`, `FetchContext`, and `build_request()`. This story extends
  those exact types. Read the S-DEMO-001 implementation before implementing push-down wiring —
  the exact `FetchContext` struct definition and `build_request()` signature are ground truth.
- **BC-2.01.013 v1.8 Pagination/Push-Down Scope Clause (D-924):** Explicitly documents that
  this story is the deferred push-down feature. The scope-out note in S-DEMO-001 AC-010 is the
  forward-pointer to this story.
- **BC-2.11.007 invariant:** "push-down is an optimization only; the query result must be
  identical whether or not push-down occurs." This is the non-negotiable correctness invariant.
  Every push-down change must be tested against it (AC-005).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `FetchContext.limit = Some(0)` | Treat as no limit (`None`) — a limit of 0 is semantically meaningless; silently ignore and let DataFusion apply limit post-materialization |
| EC-002 | `FetchContext.start_time` set but `end_time` is `None` (open-ended range) | Push start_time to API if supported; end_time omitted from request. Sensor API determines response bounds. |
| EC-003 | `FetchContext.cursor` set but sensor has already returned exhausted page (null cursor in prior page) | Push cursor value to API; sensor returns empty response or 404; PipelineExecutor handles normally as empty page |
| EC-004 | `FetchContext.limit` larger than sensor API's max page size | Sensor API caps the response at its own max; multiple pages may still be fetched by PipelineExecutor; total records bounded by `MAX_PAGES_PER_STEP` |
| EC-005 | `start_time > end_time` | Log warning; push both params to API; API returns empty result or error; PipelineExecutor propagates as empty page or error |
| EC-006 | Sensor API does not support time-range filtering | `start_time`/`end_time` silently ignored for that sensor; DataFusion applies time-window post-materialization; no API 400 error |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~3,000 |
| BC files (3 BCs) | ~6,000 |
| crates/prism-spec-engine/src/pipeline.rs (post-S-DEMO-001) | ~10,000 |
| crates/prism-bin/src/spec_driven_adapter.rs (post-S-DEMO-001) | ~4,000 |
| crates/prism-sensors/src/traits.rs | ~2,000 |
| S-DEMO-001 story (context for FetchContext definition) | ~5,000 |
| Test outputs (cargo nextest) | ~2,000 |
| **Total estimate** | **~32,000 tokens (~12% of 256K context)** |

Well within the 20-30% budget. Single-story delivery is viable.

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-05-31 | story-writer | Initial draft — created per S-DEMO-001 v1.5 AC-010 scope note and BC-2.01.013 v1.8 Pagination/Push-Down Scope Clause (D-924). Scope: thread FetchContext push-down fields (cursor/limit/start_time/end_time) from SpecDrivenSensorAdapter::fetch() into PipelineExecutor build_request(). P2 non-blocking — correctness holds via DataFusion post-materialization. |
