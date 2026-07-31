---
document_type: story
story_id: S-QUERY-SCOPE-PARAMS-001
title: "prism-mcp + prism-query: query Tool Scope-Params Plumbing — sensors / sources / time_range End-to-End Wiring (BC-2.11.001 Declared-Param Gap Closure)"
# wave: NOT wave-scheduled — post-demo backlog per orchestrator sequencing directive
# (2026-06-10 MCP cascade P2-04 anchor burst). Demo-needs check performed against the
# multi-client SOC demo task ledger (.factory/objectives/multi-client-soc-demo-tasks.md
# v1.13): the demo critical path (T5 Story B → T6 → T8) requires CLIENT targeting +
# per-client data segregation only; sensor-level narrowing appears solely in OPTIONAL
# capability-discovery task T15 (S-5.02 / S-3.13 / S-5.04 — "show this client's
# available sensors"), which is discovery surface, not query-tool scoping. No demo
# story needs `sensors` scoping in the `query` tool → post-demo backlog confirmed.
wave: post-demo-backlog
epic_id: maintenance
priority: P1
# Priority rationale: BC-2.11.001 is a P0 active contract whose declared optional
# scoping params (`sensors`, `sources`, `time_range`) have ZERO production plumbing
# (implementer-verified 2026-06-10, MCP cascade P2-04 STOP ×3). The interim behavior
# is fail-closed (`deny_unknown_fields` → MCP -32602), so there is no silent-widening
# or silent-ignore hazard TODAY — which is why this is P1 wiring debt rather than a
# P0 demo-blocking defect. Same POL-15-class declared-but-unreachable contract surface
# as S-WATCHDOG-WIRING-001.
# Status rationale: draft (NOT ready) despite non-empty behavioral_contracts —
# (1) orchestrator-directed sequencing: post-demo backlog; must not dispatch before
#     the live-demo objective (T5 Story B → T6 → T8) completes;
# (2) remove-uncertainty must run against the post-demo develop baseline: server.rs
#     QueryToolParams, engine.rs execute(), and materialization.rs are active
#     surfaces in the in-flight demo stories and the fix/review-2026-06-10-query-core
#     branch (same precaution as siblings S-WATCHDOG-WIRING-001 /
#     S-CACHE-SPEC-COMPLIANCE-001).
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-06-10T00:00:00Z"
created: "2026-06-10"
phase: 3
tdd_mode: strict
subsystems: [SS-10, SS-11]
# Subsystem anchor justifications (per ARCH-INDEX Subsystem Registry):
#   SS-10 (MCP Interface) owns the `query` / `explain_query` tool parameter surface
#     (prism-mcp server.rs QueryToolParams / ExplainQueryParams), the -32602
#     validation path, and the QueryOptions forwarding seam this story extends.
#   SS-11 (Query Execution) owns scoping resolution (prism-query scoping.rs),
#     the materialization fan-out target set (run_materialization_pipeline),
#     the ADR-033 T1 time-window extraction helpers, and QueryResultContext —
#     all consumption points this story wires.
#   prism-core is touched (crates_touched) for the PrismError surface of the
#     E-CFG-100 no-match condition; SS-10/SS-11 cover the behavioral scope —
#     the error-type touch is a supporting edit, not a subsystem anchor.
target_module: prism-query
crates_touched: [prism-mcp, prism-query, prism-core]
behavioral_contracts: [BC-2.11.001]
# BC anchor justification: BC-2.11.001 v1.7 (active) is the SOLE contract for the
# `query` tool's scoping-parameter surface. Its v1.7 interim-status annotation
# (MCP cascade P2-04, 2026-06-10) explicitly declares: "production wiring tracked by
# the query scope-params story per the 2026-06-10 review (story-writer burst
# registering the anchor in parallel)" — THIS story is that anchor. The postconditions
# referencing sensors/sources/time_range resolution (scoping resolution, predicate
# intersection, time_range_applied in query_context) are marked TARGET behavior in
# v1.7; this story delivers them. Existing-anchor check performed first per dispatch
# STEP 1: S-3.02 + S-3.02-FOLLOWUP-RUNTIME (BC-2.11.001 implementation owners) are
# MERGED (PR #129 / PR #162) and shipped `clients`/`limit`/`force_refresh` only;
# S-3.13 (proxy anchor) owns dynamic TABLE availability, not scope params; S-5.02
# owns BC-2.10.x CLIENT scoping at the MCP layer, not BC-2.11.001 sensors/sources/
# time_range; no unmerged story claims this wiring → new story required.
verification_properties: []
# Deliberately []: BC-2.11.001's VP table (VP-014/VP-015 Kani query-limit proofs,
# VP-021 parser fuzz) covers parser security properties anchored at their existing
# proof vehicles (prism-query proofs/ + fuzz target, merged). Scope-param plumbing
# adds no new formally-provable pure property; correctness is carried by the Red
# Gate suite (narrowing-intersection + merge-bound unit tests on pure helpers).
assumption_validations: []
risk_mitigations:
  - "BC-2.11.007 result-equivalence guard: the tool-level time_range MUST also be
    applied as a post-materialization filter (not push-down-only), because Armis/
    Cyberint/Claroty adapters ignore QueryParams.start_time/end_time per ADR-033 —
    push-down-only forwarding would return out-of-range rows from non-consuming
    adapters. Red Gate test 9 proves equivalence with a non-consuming adapter."
  - "Narrowing-never-widening invariant (BC-2.11.001 postcondition 3): tool params
    intersect with AST predicates; the effective scope is always a subset of both.
    Tests must prove the WIDENING direction is impossible (tool param naming a
    sensor the AST predicate excludes yields the intersection, not the union)."
  - "deny_unknown_fields stays: adding the three fields to QueryToolParams must NOT
    relax serde strictness for genuinely unknown fields. Regression-run the existing
    -32602 unknown-field rejection test unchanged against a still-unknown field."
depends_on:
  - S-DEMO-DTU-LIVE-SCENARIO-001-B
  - S-DEMO-MULTI-TENANT-DTU-001
  - S-DEMO-004
  # Dependency anchors — ORCHESTRATOR-DIRECTED SEQUENCING, not build-order:
  #   These three stories constitute the live-demo objective (T5 Story B → T6 → T8).
  #   The 2026-06-10 MCP cascade P2-04 disposition places this wiring story in the
  #   post-demo backlog (interim fail-closed -32602 behavior is correct and safe; the
  #   demo needs client targeting only — see wave comment). No compile-time dependency
  #   on any of the three exists; the edges prevent the wave scheduler from pulling
  #   this story ahead of the demo. Same pattern as siblings
  #   S-CACHE-SPEC-COMPLIANCE-001 and S-WATCHDOG-WIRING-001.
blocks: []
points: 8
# Points justification:
#   1. QueryOptions.sensors consumption: resolve_sensors in scoping.rs + fan-out
#      target filtering in run_materialization_pipeline + E-CFG-100 no-match +
#      AST-predicate intersection: 2.5 pts
#   2. QueryOptions.sources field + resolution (external source names + prism.*
#      internal tables) + intersection with AST `source` predicates: 1.5 pts
#   3. time_range: TimeRangeParam type (relative|absolute) + validation + ADR-033 T1
#      merge-with-AST-bounds (max-start/min-end narrowing) + post-materialization
#      equivalence filter + time_range_applied in QueryResultContext: 2.5 pts
#   4. QueryToolParams/ExplainQueryParams forwarding + injection-scan coverage +
#      -32602 validation + tool description sync: 0.5 pts
#   5. Red Gate suite (~12 tests) + BC backlink/companion sweeps: 1 pt
estimated_days: 3
risk: MEDIUM
# Risk justification: the merge of tool-level time_range with ADR-033 T1 AST-extracted
# bounds touches the push-down path that BC-2.11.007 result-equivalence depends on;
# an incorrect merge direction (union instead of intersection) silently WIDENS scope —
# the exact hazard the interim fail-closed behavior exists to prevent. Mitigated by
# pure-function extraction (merge_time_bounds) with exhaustive unit tests before any
# pipeline integration. The sensors/sources narrowing itself is low-risk (pure
# pre-fan-out set filtering).
acceptance_criteria_count: 12
red_gate_tests: 12
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md"
  - ".factory/specs/architecture/decisions/ADR-033-push-down-time-window-extraction-strategy-pre-fan-out-heuristic.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
---

# S-QUERY-SCOPE-PARAMS-001 — query Tool Scope-Params Plumbing (sensors / sources / time_range)

## Narrative

As an MSSP analyst (and the LLM agent driving the `query` MCP tool), I want the
BC-2.11.001-declared optional scoping parameters `sensors`, `sources`, and
`time_range` to be accepted by the tool and actually consumed by the query engine —
narrowing fan-out targets and the effective time window, never widening them — so
that I can scope an investigation to specific sensor types, specific data sources
(including `prism.*` internal tables), and a bounded time range without encoding
everything as PrismQL predicates, and so the tool's declared contract surface matches
its shipped behavior.

## Behavioral Contracts

| BC | Title | Version at authoring | Role |
|----|-------|---------------------|------|
| BC-2.11.001 | `query` MCP Tool Accepts Scoping + PrismQL Query String | v1.7 (active) | Sole anchor. v1.7 interim-status annotation marks the sensors/sources/time_range postconditions as TARGET behavior pending this story; preconditions define the param shapes; postconditions 2-3 define resolution + narrowing intersection; the Error Cases table binds E-CFG-100 to the no-match condition; the `query_context` postcondition requires `time_range_applied`. |

Cross-referenced (NOT anchored — no AC traces to them; listed for implementer context):
- **BC-2.11.007** (result-equivalence invariant): the time_range post-materialization
  filter exists to uphold it (see risk_mitigations).
- **ADR-033 T1**: defines the existing AST datetime-bound extraction this story's
  `time_range` merge composes with; T1's `QueryParams.start_time`/`end_time`
  `Option<String>` interface is consumed unchanged (ADR-033 Rationale point 4).

## Current State (implementer-verified 2026-06-10, MCP cascade P2-04 STOP ×3)

- `QueryToolParams` (prism-mcp `server.rs`) carries `query` + `clients` only.
  `#[serde(deny_unknown_fields)]` rejects `sensors`/`sources`/`time_range` with MCP
  `-32602` INVALID_PARAMS — **fail-closed, correct interim behavior** (never silently
  ignored). The `query` tool handler builds `QueryOptions { clients, ..Default::default() }`.
- `QueryOptions` (prism-query `engine.rs`) has a `sensors: Option<Vec<SensorId>>`
  field with **zero execute-path readers** — the only readers live in `explain.rs`
  (plan rendering). `sources` and any time-range field are **absent** from the struct.
- `run_materialization_pipeline` (prism-query `materialization.rs`) resolves fan-out
  targets from `options.clients` + config; it never consults `options.sensors`.
- ADR-033 T1 helpers in `materialization.rs` extract `start_time`/`end_time` from AST
  datetime Compare predicates into `QueryParams`; there is no tool-level time-range
  input to merge.
- `QueryResultContext` (engine.rs) has `clients_queried`/`sensors_queried` but **no
  `time_range_applied` field** (BC-2.11.001 `query_context` postcondition — target
  behavior per v1.7 annotation).
- Naive forwarding of the existing dead `sensors` field would have been WRONG: with no
  execute-path reader, accepting the param and ignoring it silently WIDENS effective
  scope relative to the caller's intent — the precise hazard the `deny_unknown_fields`
  interim closes. This story replaces fail-closed rejection with real consumption.

## Acceptance Criteria

### AC-001: QueryToolParams accepts sensors/sources/time_range; deny_unknown_fields retained
`QueryToolParams` gains `sensors: Option<Vec<String>>`, `sources: Option<Vec<String>>`,
and `time_range: Option<TimeRangeParam>`. `#[serde(deny_unknown_fields)]` is retained;
a genuinely unknown field still yields MCP `-32602`. All three new params are covered
by the BC-2.09.001 injection scan and length/charset validation before any domain
logic (same `validate_string_vec_field` discipline as `clients`).
*(traces to BC-2.11.001 precondition 2 — optional scoping parameters declared shapes)*

### AC-002: QueryOptions.sensors consumed — fan-out narrowed to listed sensor types
With `sensors: ["crowdstrike"]`, `run_materialization_pipeline` resolves fan-out
targets to only (client, sensor) combinations whose sensor type is in the list;
no fetch is issued to excluded sensors. `sensors: null`/absent preserves current
all-sensors behavior byte-identically. `"prism"` in the list admits internal tables.
*(traces to BC-2.11.001 postcondition 2 — scoping parameters resolved to concrete client/sensor combinations)*

### AC-003: AST predicate intersection — narrowing, never widening
When the query AST contains `sensor` or `source` equality predicates AND the
corresponding tool param is present, the effective scope is the set INTERSECTION.
A tool param can never ADD a target the AST predicates exclude, and vice versa.
A valid-but-disjoint intersection (e.g., `sensors: ["armis"]` + `sensor = "crowdstrike"`
predicate) returns an EMPTY result set with explanatory metadata — not an error
(EC-11-001 client-analogue semantics).
*(traces to BC-2.11.001 postcondition 3 — predicates intersected with tool parameters, narrowing never widening; edge case EC-11-001)*

### AC-004: E-CFG-100 for scope params matching nothing configured
When `sensors` or `sources` names match NO configured sensor/source for the resolved
clients (e.g., `sensors: ["nonexistent"]`), the tool returns the structured error
E-CFG-100 listing the configured clients/sensors — distinguishing misconfiguration
(no such sensor configured → error) from valid-but-empty intersection (AC-003 → empty
results). No new error code is introduced; E-CFG-100 is the existing taxonomy code
BC-2.11.001 binds to this condition.
*(traces to BC-2.11.001 Error Cases — E-CFG-100 "No matching clients/sensors found for scoping parameters")*

### AC-005: QueryOptions.sources field added and consumed
`QueryOptions` gains `sources: Option<Vec<String>>`. Resolution: external source names
narrow which sensor tables are registered/fanned-out; `prism.*` names (e.g.
`"prism.alerts"`) select internal RocksDB-backed tables. `sources: null` preserves
current behavior. Sources resolution composes with sensors resolution (both narrow).
*(traces to BC-2.11.001 precondition 2 — sources param incl. prism.* internal names; postcondition 5 — internal tables path)*

### AC-006: TimeRangeParam type — relative or absolute, validated
`TimeRangeParam` is defined in prism-mcp with `#[serde(deny_unknown_fields)]`:
fields `start: Option<String>` (RFC3339), `end: Option<String>` (RFC3339),
`last: Option<String>` (relative duration, grammar `^[0-9]+(m|h|d)$`). Validation
(pre-execution, surfaced as `-32602`): at least one field present; `last` mutually
exclusive with `start`/`end`; `start < end` when both present; RFC3339 parse enforced.
`last` resolves to `[now - duration, now]` at tool-call wall-clock time.
Rationale for this shape (decided in-scope per production-grade rule 6): BC-2.11.001
says only "relative or absolute"; a single struct with mutual-exclusion validation is
the JSON-schema-friendliest encoding for `rmcp` + `schemars` (no untagged-enum schema
ambiguity for the consuming LLM agent).
*(traces to BC-2.11.001 precondition 2 — time_range (relative or absolute))*

### AC-007: time_range merged with ADR-033 T1 AST bounds — narrowing intersection
A pure function `merge_time_bounds` in prism-query composes the tool-level resolved
`[start, end]` with the ADR-033 T1 AST-extracted datetime bounds:
effective start = MAX of present starts; effective end = MIN of present ends
(interval intersection — narrowing, never widening). The merged bounds populate
`QueryParams.start_time` / `end_time` (the unchanged T1 `Option<String>` interface,
per ADR-033 Rationale point 4). An empty merged interval (start ≥ end) yields an
empty result set with explanatory metadata — not an error (AC-003 semantics).
*(traces to BC-2.11.001 postcondition 3 — narrowing intersection applied to the time dimension)*

### AC-008: time_range push-down reaches consuming adapters
With a `time_range` tool param and no AST datetime predicates, the merged bounds are
forwarded into `QueryParams.start_time`/`end_time` and injected into the CrowdStrike
FQL filter exactly as ADR-033 specifies for AST-extracted bounds (`created_timestamp:`
clauses). Adapters without native time params receive the values and ignore them
(ADR-033 no-op contract) — correctness is then carried by AC-009.
*(traces to BC-2.11.001 postcondition 2 — scoping resolution; mechanism per ADR-033 T1)*

### AC-009: time_range result-equivalence — post-materialization filter
The tool-level effective time window is ALSO applied as a post-materialization
DataFusion filter on the canonical normalized event timestamp column, so results from
adapters that ignore push-down (Armis/Cyberint/Claroty per ADR-033) are equivalently
bounded. Red Gate proof: identical result sets for a time-bounded query against a
push-down-consuming adapter and a non-consuming adapter over the same logical data.
*(traces to BC-2.11.001 postcondition 3 narrowing guarantee; upholds BC-2.11.007 result-equivalence — cross-referenced, not anchored)*

### AC-010: query_context gains time_range_applied
`QueryResultContext` gains `time_range_applied: Option<TimeRangeApplied>` (effective
RFC3339 `start`/`end` actually enforced, post-merge; `None` when no bound was active
from either source). The MCP response envelope's `query_context` serializes it.
*(traces to BC-2.11.001 postcondition — response query_context includes time_range_applied)*

### AC-011: sensors_queried / clients_queried reflect the narrowed scope
`query_context.sensors_queried` lists only the sensors actually in the post-
intersection fan-out set (not the pre-narrowing configured set); `clients_queried`
is unchanged in semantics. Verified for the sensors-narrowed, sources-narrowed, and
disjoint-intersection (empty) cases.
*(traces to BC-2.11.001 postcondition — query_context clients_queried / sensors_queried)*

### AC-012: explain_query parity
`ExplainQueryParams` gains the same three params with identical validation;
`explain_query` forwards them into `QueryOptions` so the rendered plan reflects the
narrowed scope (explain.rs already reads `options.sensors` for plan display — this AC
makes that reader reachable from the tool surface and extends it to sources/
time_range). Sibling-sweep obligation (TD-VSDD-060): the two param structs and both
tool descriptions must not drift.
*(traces to BC-2.11.001 postcondition 2 — scoping resolution applies to the declared tool surface; parity prevents a contract-drift recurrence)*

## Red Gate Test Plan

| # | Test (name sketch) | AC | Kind |
|---|--------------------|----|------|
| 1 | `test_BC_2_11_001_query_params_accept_sensors_sources_time_range_and_reject_unknown` | AC-001 | unit (prism-mcp, serde) |
| 2 | `test_BC_2_11_001_sensors_scope_narrows_fanout_targets` | AC-002 | unit (prism-query, mock adapter seam per SID-1) |
| 3 | `test_BC_2_11_001_sensor_predicate_intersects_tool_param_never_widens` | AC-003 | unit (pure intersection fn) |
| 4 | `test_BC_2_11_001_disjoint_sensor_intersection_returns_empty_not_error` | AC-003 | unit |
| 5 | `test_BC_2_11_001_unconfigured_sensor_scope_returns_e_cfg_100_with_configured_listing` | AC-004 | unit |
| 6 | `test_BC_2_11_001_sources_scope_selects_internal_prism_tables_and_narrows_external` | AC-005 | unit |
| 7 | `test_BC_2_11_001_time_range_param_validation_matrix` (absent/relative/absolute/mixed/inverted/malformed) | AC-006 | unit (prism-mcp) |
| 8 | `test_BC_2_11_001_merge_time_bounds_is_interval_intersection` (tool-only / AST-only / both / empty interval) | AC-007 | unit (pure fn) |
| 9 | `test_BC_2_11_001_time_range_pushdown_reaches_crowdstrike_fql` | AC-008 | unit (QueryParams assertion at adapter seam) |
| 10 | `test_BC_2_11_001_time_range_result_equivalence_nonconsuming_adapter` | AC-009 | unit/integration (two mock adapters, same logical data) |
| 11 | `test_BC_2_11_001_query_context_time_range_applied_and_narrowed_sensors_queried` | AC-010, AC-011 | unit |
| 12 | `test_BC_2_11_001_explain_query_param_parity_and_plan_reflects_scope` | AC-012 | unit (prism-mcp + explain) |

All non-trivial new function bodies start as `todo!()` (tdd_mode: strict; Red Gate
density ≥0.5 before Step 4 dispatch).

## Token Budget Estimate

| Input | Est. tokens |
|-------|------------|
| This story | ~7k |
| BC files (1 BC): BC-2.11.001 v1.7 | ~3k |
| ADR-033 (full) | ~3k |
| error-taxonomy.md (E-CFG-100 + E-QUERY rows only, targeted read) | ~2k |
| prism-mcp server.rs (QueryToolParams/ExplainQueryParams + query/explain handlers, targeted) | ~8k |
| prism-query engine.rs (QueryOptions/QueryResultContext/execute, targeted) | ~8k |
| prism-query materialization.rs (pipeline + T1 helpers, targeted) | ~10k |
| prism-query scoping.rs + explain.rs (targeted) | ~6k |
| New + touched test files | ~10k |
| Tool outputs (just iter cycles, rg sweeps) | ~8k |
| **Total** | **~65k (≈ 33% of 200k window — acceptable; targeted reads keep the three big prism-query files partial, not full)** |

## Tasks

1. **prism-mcp param surface (AC-001, AC-006, AC-012):** define `TimeRangeParam`
   (deny_unknown_fields, schemars); extend `QueryToolParams` + `ExplainQueryParams`;
   add validation fns (`validate_time_range`, sensors/sources via existing
   `validate_string_vec_field`); wire all new values into the injection-scan `inputs`
   vec; sync both tool description strings (the `query` description's stale
   "limit (optional)" mention is corrected in the same edit — sibling-sweep).
2. **prism-query QueryOptions (AC-005):** add `sources: Option<Vec<String>>` and the
   time-bound carrier (resolved `start`/`end` as `Option<String>` RFC3339 — resolved
   in prism-mcp so the engine never sees relative forms); `#[non_exhaustive]`
   discipline check on QueryOptions (pub type; compile-fail gate EXPECTED bump in
   ci.yml if not already counted).
3. **Scoping resolution (AC-002, AC-004):** `resolve_sensors` / `resolve_sources` in
   scoping.rs (mirror `resolve_clients` shape); E-CFG-100 no-match surfacing — reuse
   the existing taxonomy code; if a new `PrismError` variant is required to carry the
   configured-listing payload, it MUST display the existing `E-CFG-100` code (no new
   E- code; flag the taxonomy "Used in" companion row to product-owner at delivery —
   see §Out of Scope item 2).
4. **Fan-out narrowing (AC-002, AC-003, AC-005):** filter the resolved (client,
   sensor) target set in `run_materialization_pipeline` by sensors∩AST and
   sources∩AST (pure intersection helpers, unit-tested first); empty-intersection →
   empty-result metadata path.
5. **Time merge (AC-007, AC-008, AC-009):** pure `merge_time_bounds`; populate
   `QueryParams.start_time`/`end_time` from the merged interval (composing with the
   existing ADR-033 T1 extraction, not replacing it); post-materialization DataFusion
   filter on the canonical normalized timestamp column for the tool-level window.
6. **query_context (AC-010, AC-011):** add `time_range_applied` to
   `QueryResultContext`; thread effective bounds + narrowed `sensors_queried` from
   materialization output; serialize in the MCP response envelope.
7. **Red Gate suite:** 12 tests per the plan above; regression-run the existing
   -32602 unknown-field test, the BC-2.11.006 timeout suite, and the ADR-033
   CrowdStrike FQL injection tests unchanged.
8. **Spec backlinks (delivery-time):** BC-2.11.001 Traceability "Stories" row +=
   S-QUERY-SCOPE-PARAMS-001; v1.7 interim-status annotation retired by product-owner
   when this story merges (PO-owned edit — routing flag, not implementer scope).

## Previous Story Intelligence

- **S-3.02-FOLLOWUP-RUNTIME (merged PR #162):** established the execute() pipeline
  shape this story extends (parse → scope resolve → push-down classify → fan-out);
  its lesson — `..Default::default()` construction silently dropping a param
  (F-PASS12-CRIT-2, `clients` not forwarded) — is the EXACT defect class this story
  closes for the remaining three params. Do not repeat it for `capabilities` when
  rebuilding `QueryOptions` in the handler.
- **S-DEMO-QUERY-PUSHDOWN-001 v2 (ADR-033):** time-window extraction is pre-fan-out
  and heuristic; adapters ignoring `start_time`/`end_time` is BY DESIGN — never
  "fix" a non-consuming adapter by fabricating params it lacks (SAP-2: read the DTU
  types.rs before touching any adapter param injection).
- **S-WATCHDOG-WIRING-001 (sibling backlog story, same cycle):** POL-15-class
  declared-but-unreachable surface pattern; mirror its sequencing-edge frontmatter
  conventions and its taxonomy-verification discipline (verify codes against
  error-taxonomy.md CURRENT version at dispatch; v1.70 at authoring).
- **MCP cascade P2-04 (this story's trigger):** three consecutive STOP verdicts
  confirmed forwarding-without-consumption would silently widen scope. The interim
  fail-closed `-32602` is the correct baseline; every AC here must strictly improve
  on it (consumption, never silent acceptance).

## Architecture Compliance Rules

- **Narrowing-never-widening** (BC-2.11.001 postcondition 3) is the governing
  invariant for every resolution step: sensors, sources, AND time bounds compose by
  intersection. Any code path where a tool param can expand the AST-implied scope is
  a defect.
- **ADR-033 T1 interface stability:** consume `QueryParams.start_time`/`end_time`
  `Option<String>` unchanged (Rationale point 4 — T2 will consume the same fields);
  do NOT restructure fan-out orchestration (REQUIRED-column plan-time enforcement is
  anchored to `S-REQUIRED-COL-GATE-001`, which uses `resolved_spec_map` pre-fan-out per
  the T1 access pattern; full T2 fan-out restructuring for non-REQUIRED dimensions is a
  separate future story not yet scoped).
- **Ephemeral SessionContext** (BC-2.11.005): no scope state held across calls;
  relative `time_range` resolves per-call at the MCP boundary.
- **BC-2.09.001 injection scan BEFORE domain logic** for all new string inputs.
- **`#[non_exhaustive]`** on any new/extended pub TOML-deserialized or pub-API type
  (TimeRangeParam, QueryOptions if not already gated; check `tests/external/
  non-exhaustive-violation/` EXPECTED count).
- **Error taxonomy:** E-CFG-100 only for the no-match condition; `-32602` for param
  validation; NO new E- codes (any taxonomy companion row is a PO-routed flag).
- **SAP-1:** any new `tracing::*!(event_type=…)` emission requires a same-commit
  BC-2.16.002 catalog row. Expected: zero new emissions (scope resolution reuses
  existing audit/event paths); if one proves necessary, the catalog row ships with it.
- **No `unwrap()`/`expect()`** in the new resolution/merge paths; RFC3339 parse
  failures surface as structured `-32602`, interval math uses checked operations.

## Library & Framework Requirements

| Dependency | Version source | Use |
|------------|---------------|-----|
| rmcp | workspace pin (S-5.01-FOLLOWUP-MCP-BOOT baseline, rmcp 1.7 line) | tool param schemas, -32602 ErrorData |
| serde / schemars | workspace pins | TimeRangeParam deny_unknown_fields + JSON schema |
| chrono | workspace pin | RFC3339 parse, relative-duration resolution |
| datafusion / arrow | workspace pins (do NOT invent versions; centralized in root Cargo.toml) | post-materialization time filter |
| tokio | workspace pin | unchanged async surface |

No new external dependencies. All versions come from the workspace `Cargo.toml`
pins — never from training data.

## File Structure Requirements

| File | Action | Content |
|------|--------|---------|
| `crates/prism-mcp/src/server.rs` | MODIFY | QueryToolParams/ExplainQueryParams fields; TimeRangeParam type + validation; handler forwarding into QueryOptions; tool description sync; injection-scan inputs |
| `crates/prism-query/src/engine.rs` | MODIFY | QueryOptions.sources + resolved time-bound fields; QueryResultContext.time_range_applied; execute() threading |
| `crates/prism-query/src/scoping.rs` | MODIFY | resolve_sensors / resolve_sources (+ E-CFG-100 no-match) |
| `crates/prism-query/src/materialization.rs` | MODIFY | fan-out target intersection filtering; merge_time_bounds composition with ADR-033 T1 helpers; post-materialization time filter; narrowed sensors_queried output |
| `crates/prism-query/src/explain.rs` | MODIFY | sources/time_range plan rendering (sensors reader already present) |
| `crates/prism-core/src/error.rs` | MODIFY (conditional) | only if E-CFG-100 no-match needs a payload-carrying variant; Display must emit the existing code verbatim per taxonomy Message Format |
| `crates/prism-query/src/tests/` + `crates/prism-mcp/src/` `#[cfg(test)]` | CREATE/MODIFY | Red Gate tests 1-12 (in-process unit seams per SID-1; no #[ignore]'d substitutes) |
| `tests/external/non-exhaustive-violation/` + `ci.yml` | MODIFY (conditional) | EXPECTED bump if TimeRangeParam/QueryOptions gate coverage changes |

Forbidden: no edits to DTU clone crates (`prism-dtu-*`) — ADR-033 establishes
non-consuming adapters ignore time params by design; no fan-out orchestration
restructuring (T2 scope); no `.factory/` spec edits by the implementer except the
SAP-1 same-commit catalog rule if triggered.

## Forbidden Dependencies

- `prism-mcp` must NOT gain a dependency on `prism-storage` or DTU crates for this
  story (param validation is pure; resolution lives in prism-query). If the build
  gains such an edge, the build MUST fail review.
- `prism-core` must NOT depend on `prism-query`/`prism-mcp` (existing layering;
  the conditional error-variant edit is core-side only).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `sensors: []` (empty array, not null) | `-32602` pre-execution validation: empty array is ambiguous (all? none?) — reject with message directing to omit the field for all-sensors. Decided in-scope: fail-closed beats guessing. |
| EC-002 | `sensors: ["armis"]` + AST `sensor = "crowdstrike"` | Empty intersection → empty result set + metadata (AC-003); NOT E-CFG-100 (both names are configured) |
| EC-003 | `sensors: ["nonexistent"]` | E-CFG-100 listing configured sensors (AC-004) |
| EC-004 | `sensors: ["prism"]` + `sources: ["crowdstrike.detections"]` | Intersection: prism-only sensor scope excludes external source → empty result set + metadata |
| EC-005 | `time_range: {last: "24h", start: "..."}` | `-32602`: relative and absolute are mutually exclusive (AC-006) |
| EC-006 | `time_range: {start: "2026-06-10T00:00:00Z", end: "2026-06-09T00:00:00Z"}` | `-32602`: inverted interval rejected pre-execution |
| EC-007 | Tool `time_range` [Jan, Dec] + AST `created_time > Mar AND created_time < Jun` | Effective [Mar, Jun] (max-start/min-end); `time_range_applied` reports [Mar, Jun] |
| EC-008 | Tool `time_range` and AST bounds with EMPTY intersection | Empty result set + metadata, not error (AC-007) |
| EC-009 | `time_range` with only `end` | Valid: bounded above, unbounded below; `start: None` forwarded |
| EC-010 | Non-consuming adapter (Armis) with `time_range` | Push-down ignored per ADR-033; post-materialization filter bounds results (AC-009) |
| EC-011 | EC-11-032 interplay: time-narrowed query still exceeding `limit` | Unchanged truncation semantics: `is_truncated`/`total_available` computed AFTER the time filter |

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| TimeRangeParam + validation | prism-mcp `server.rs` | Pure (validation) |
| resolve_sensors / resolve_sources | prism-query `scoping.rs` | Pure (config snapshot in, target set out) |
| Scope intersection helpers | prism-query `materialization.rs` | Pure |
| merge_time_bounds | prism-query `materialization.rs` (alongside T1 helpers) | Pure |
| Fan-out narrowing + post-filter | prism-query `materialization.rs` / `engine.rs` | Effectful (pipeline) |
| query_context threading | prism-query `engine.rs` → prism-mcp envelope | Effectful |

## Out of Scope (explicit routing flags — NOT silent deferrals)

1. **ADR-033 T2** (per-sensor `classify_predicates` post-resolution integration):
   The plan-time E-QUERY-009 enforcement gate for REQUIRED columns is anchored to
   `S-REQUIRED-COL-GATE-001` — it uses `resolved_spec_map` pre-fan-out per
   BC-2.11.007 §REQUIRED Column Runtime Mechanism and requires no fan-out
   restructuring. Full per-sensor post-resolution `classify_predicates` integration
   covering non-REQUIRED push-down dimensions (the broader T2 scope) remains deferred;
   a superseding ADR + story will be authored when fan-out restructuring is designed.
   This story (S-QUERY-SCOPE-PARAMS-001) composes with T1 only.
2. **error-taxonomy E-CFG-100 companion row touch:** the taxonomy v1.70 row's Message
   Format is client-centric (`PrismError::ClientNotFound`); BC-2.11.001 binds the
   same code to the broader "no matching clients/sensors" condition. If delivery
   requires a sensors-flavored message under the same code, the taxonomy "Used in"/
   Message Format companion edit routes to **product-owner** (ADR-035 canonical-row
   convention) in the delivery cascade — no new code, message-surface ratification only.
3. **BC-2.11.001 v1.7 interim-annotation retirement:** PO-owned spec edit at merge
   (POL-14 burst window) — listed in Tasks item 8 as a routing flag.
4. **`limit`/`force_refresh` MCP-surface audit:** BC v1.7 records them as wired;
   the `query` tool description mentions `limit` but `QueryToolParams` does not carry
   it (engine-level `QueryOptions.limit` has readers; the tool struct does not expose
   it). The description-string correction ships in Task 1 (sibling-sweep); whether
   the tool SURFACE should expose `limit`/`force_refresh` params is a BC-conformance
   question routed to **product-owner** via the orchestrator — it predates this gap
   and is not part of the P2-04 disposition.

## Story Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-10 | story-writer | Authored per MCP cascade P2-04 STOP ×3 disposition (2026-06-10): anchor the BC-2.11.001 v1.7 declared-but-unwired scope-params gap (`sensors`/`sources`/`time_range`). Existing-anchor check: S-3.13 (table availability), S-5.02 (BC-2.10.x client scoping), S-3.02/S-3.02-FOLLOWUP-RUNTIME (merged; shipped clients/limit/force_refresh only) — none owns this surface; new story required. Post-demo backlog sequencing (task-ledger v1.13 check: demo critical path needs client targeting only; T15 capability-discovery is optional and a different surface). 12 ACs all tracing to BC-2.11.001 v1.7; 12 Red Gate tests; 8 pts; tdd_mode strict. |
