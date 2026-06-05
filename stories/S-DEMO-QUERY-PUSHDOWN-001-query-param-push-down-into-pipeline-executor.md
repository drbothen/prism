---
document_type: story
story_id: S-DEMO-QUERY-PUSHDOWN-001
title: "prism-query + prism-spec-engine + prism-bin: Correct per-sensor push-down wiring — CrowdStrike FQL time-window + limit; Armis AQL correctness; materialization.rs wiring (ADR-033 T1)"
wave: wave-5-e-demo-fidelity
epic_id: E-DEMO
priority: P2
status: in_progress
version: "2.0"
level: "L3"
producer: story-writer
revised_by: null
timestamp: "2026-06-05T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-11, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns SpecDrivenSensorAdapter::fetch() in prism-bin which
#     receives QueryParams from callers and translates push-down fields into per-sensor
#     API request params. Armis AQL correctness fix lives here.
#   SS-11 (Query Execution) owns run_materialization_pipeline in prism-query/src/materialization.rs
#     — the sole production callsite that constructs QueryParams. ADR-033 T1 time-window
#     extraction (the new function in pushdown.rs/materialization.rs) lives here.
#   SS-16 (Spec Engine) owns PipelineExecutor and the per-sensor translation logic in
#     prism-spec-engine/src/pipeline.rs. Wrong Cyberint/Claroty translations removed here.
crates_touched: [prism-query, prism-spec-engine, prism-bin]
# crates_touched v2 change: prism-query ADDED (was [prism-spec-engine, prism-bin] in v1.x).
# prism-query is required because ADR-033 Option T1 places the time-window extraction function
# in materialization.rs / pushdown.rs inside prism-query — this is the primary new scope in v2.
target_module: prism-query
capabilities: [CAP-015]
behavioral_contracts:
  - BC-2.01.013  # DataSource Trait / SpecDrivenSensorAdapter Push-Down Scope Clause (v1.13):
                 # Per-sensor push-down translation table corrected per pushdown-redesign.md §6.
                 # v2 implements the correct CrowdStrike FQL wiring + removes wrong Armis/
                 # Cyberint/Claroty translations. ADR-033 referenced.
  - BC-2.11.005  # Ephemeral Materialization (v1.6): cache misses on first/query-plan step
                 # trigger sensor API calls with push-down filters. As of v2, start_time/end_time
                 # reach the fan-out via run_materialization_pipeline per ADR-033 T1.
  - BC-2.11.007  # Sensor Filter Push-Down (v1.7): push-down is optimization only; result must
                 # be identical whether or not push-down occurs (result-equivalence invariant).
                 # Time-range push-down qualified: CrowdStrike only in current DTU set. Armis/
                 # Cyberint/Claroty time-window is post-filter via DataFusion.
verification_properties: []
# VP note: No existing VP covers push-down / filter threading behavior. If a dedicated
# push-down VP is warranted, this is flagged for product-owner / architect authorship
# (story-writer cannot author VPs per agent routing policy; ADR-033 is the architecture anchor).
depends_on:
  - S-DEMO-001   # Must merge first: SpecDrivenSensorAdapter and PipelineExecutor wiring
                 # delivered by S-DEMO-001 are the extension points this story modifies.
                 # Cannot thread FetchContext params into a pipeline that doesn't exist yet.
                 # STATUS: SATISFIED — merged PR #166 develop@5dd3df02 2026-06-01.
  - S-DEMO-002   # S-DEMO-002 established the AQL seeding convention (query_filters["aql"]
                 # in FetchContext seeded in PipelineExecutor). AC-ARMIS-001/002 depend on
                 # that seeding being present. STATUS: SATISFIED — merged PR #171 develop@fdd12251
                 # 2026-06-04.
blocks: []
# Dependency anchor justifications:
#   depends_on S-DEMO-001: S-DEMO-001 delivers SpecDrivenSensorAdapter::fetch(), PipelineExecutor::execute(),
#     FetchContext, and build_request(). This story extends those exact types and call sites.
#   depends_on S-DEMO-002: The AQL seeding path (FetchContext.query_filters["aql"] → ${query.filter.aql}
#     interpolation in path_template) was established by S-DEMO-002's cross-story dependency
#     recording (D-935). AC-ARMIS-001/002 verify correct non-injection behavior against the
#     already-wired AQL passthrough; S-DEMO-002 must be merged to avoid conflicting with that path.
points: 8
# Points justification (v2 re-estimate, expanded from v1 5pts):
#   - ADR-033 T1: new extract_time_window_from_ast function in prism-query/pushdown.rs: ~1.5 pts
#   - materialization.rs callsite wiring (lines ~437-438 None → extracted values): ~1 pt
#   - CrowdStrike FQL injection correctness (start+end combined with '+'; Step 1 only): ~1 pt
#   - Armis: remove maxResults/timeFrame (REMOVAL — small but must be verified): ~0.5 pts
#   - Cyberint: remove from_date/to_date POST-body injection (GET endpoint; correctness fix): ~0.5 pts
#   - Claroty: remove time-window body injection (correctness fix): ~0.5 pts
#   - AC-EQUIV-001: result-equivalence via real materialization path (not direct FetchContext): ~1.5 pts
#   - SAP-2 compliant test fixtures (production TOML shape or verified strict subset): ~1 pt
#   Total: 8 points (~2.5 days focused TDD work)
# v1 prior implementation (prism-query missing; wrong Armis/Cyberint/Claroty translations):
# superseded by v2. v1.x implementation will be re-done from this corrected spec.
estimated_days: 3
risk: MEDIUM
# Risk justification (v2):
# Primary risk is SAP-2 test fixture compliance — adversary pass 1 will verify every fixture
# against production TOML shape. Secondary risk: ADR-033 T1 requires threading
# MaterializationContext.resolved_spec_map into the time-window extraction call; if
# resolved_spec_map is None at extraction time, extraction must silently return None (safe default).
# BC-2.11.007 result-equivalence invariant is the correctness safety net.
acceptance_criteria_count: 9
red_gate_tests: 9
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Result-equivalence invariant (BC-2.11.007): every push-down test MUST be paired with
    a result-equivalence assertion — the same query with push-down and without push-down
    must produce identical result sets (modulo row order). This is the regression gate."
  - "SAP-2 standing gate (every adversary pass 1): every test constructing a sensor spec
    fixture MUST use production TOML shape (or verified strict subset — same method,
    body_template presence/absence, pagination type, step count). Fabricated fixtures
    (make_crowdstrike_like_spec / make_cyberint_like_spec / make_armis_like_spec) that
    diverge from production shape are FORBIDDEN. Adversary must explicitly grep for
    fabricated-fixture helper functions in pass 1."
  - "Correctness removals: Armis maxResults/timeFrame, Cyberint from_date/to_date, and
    Claroty time-window body injection are REMOVALS. The test-writer must add Red Gate tests
    that ASSERT ABSENCE of these params in the generated request — a test that passes because
    a param is simply not injected is not load-bearing unless it explicitly verifies absence."
  - "ADR-033 T1 safe default: if MaterializationContext.resolved_spec_map is None at
    extract_time_window_from_ast call time, start_time and end_time MUST be None (no push-down).
    This is the correct safe default. Implementation must not panic on None spec map."
inputs:
  - "crates/prism-query/src/materialization.rs"
  - "crates/prism-query/src/pushdown.rs"
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - "crates/prism-sensors/specs/crowdstrike.sensor.toml"
  - "crates/prism-sensors/specs/armis.sensor.toml"
  - "crates/prism-sensors/specs/cyberint.sensor.toml"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - "crates/prism-dtu-crowdstrike/src/routes/detections.rs"
  - "crates/prism-dtu-armis/src/routes/search.rs"
  - "crates/prism-dtu-cyberint/src/routes/alerts.rs"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.005-ephemeral-materialization.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md"
  - ".factory/specs/architecture/decisions/ADR-033-push-down-time-window-extraction-strategy-pre-fan-out-heuristic.md"
  - ".factory/cycles/wave-5-e-demo-fidelity/S-DEMO-QUERY-PUSHDOWN-001/pushdown-redesign.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-QUERY-PUSHDOWN-001 v2.0 — Correct per-sensor push-down wiring (ADR-033 T1 + correctness fixes)

**Story ID:** S-DEMO-QUERY-PUSHDOWN-001
**Status:** in_progress
**Version:** v2.0
**Wave:** wave-5-e-demo-fidelity
**Priority:** P2
**Points:** 8

---

## Origin

This is a **major re-author** (v1.3 → v2.0). The v1.x implementation is superseded.

LOCAL adversary passes 5 and 6 (2026-06-05) established that the v1.x implementation
was largely inert against production sensor shapes. Two compounding defects:

1. **materialization.rs hardcodes None.** `prism-query/src/materialization.rs` lines
   ~434–440 (the sole production callsite building `QueryParams`) always sets
   `start_time: None`, `end_time: None`, `cursor: None`. All time-window translation
   code in the push-down path is dead code in production. `limit` is the only
   dimension that actually reaches the sensor adapter.

2. **Per-sensor translations are wrong.** Armis injects `maxResults` / `timeFrame`
   (not in `SearchQueryParams`). Cyberint injects into a POST body that does not
   exist (real spec is GET, cursor-only, no body_template). Claroty injects `limit`
   into `body_template: '{}'` (no-op). These are correctness defects, not push-down
   gaps.

The v2 re-spec corrects both defects. It also adds `prism-query` to `crates_touched`
(required for ADR-033 Option T1 time-window extraction in `materialization.rs` /
`pushdown.rs`).

**v1.x prior implementation notes are retained for historical context only.
All v1.x test code and translation logic will be re-done from this corrected spec.**

---

## Narrative

As the Prism query engine, I want time-window predicates in the PrismQL AST extracted
by `run_materialization_pipeline` (ADR-033 T1) and propagated as `QueryParams.start_time`
/ `QueryParams.end_time` so that CrowdStrike's `filter` FQL param receives the correct
time constraint — and I want incorrect Armis, Cyberint, and Claroty push-down
translations removed — so that sensor API requests carry only correct native parameters,
without changing query results (BC-2.11.007 result-equivalence invariant).

---

## Story-Level Goal

After this story merges:

1. **prism-query — ADR-033 T1 wiring:**
   A new function (or extension of `extract_push_down_filters_as_map`) in
   `prism-query/src/pushdown.rs` walks `Predicate::Compare` nodes with
   `op ∈ {Gt, Ge, Lt, Le}` and matches lhs column names against
   `column_type = "datetime"` columns in `MaterializationContext.resolved_spec_map`.
   Extracted ISO8601 strings populate `QueryParams.start_time` (Gt/Ge) and
   `QueryParams.end_time` (Lt/Le) at the materialization callsite (currently `None`).

2. **CrowdStrike — FQL time-window + limit (correct behavior retained + completed):**
   When `start_time` and/or `end_time` are set, they are injected into the `filter`
   FQL query param on CrowdStrike Step 1 (`query_detection_ids`) as
   `created_timestamp:>'<ISO8601>'` (start) and `created_timestamp:<'<ISO8601>'`
   (end), combined with `+` when both are present. `limit` query param wiring
   (already partially working) is verified against the production TOML shape.
   Step 2 (`fetch_detections`) receives `FetchContext::default()` — no push-down.

3. **Armis — AQL passthrough correctness (removal):**
   `maxResults` and `timeFrame` injection is removed. Armis push-down is exclusively
   AQL verbatim passthrough per BC-2.11.007 Mechanism B: the `aql` value from
   `FetchContext.query_filters["aql"]` is already interpolated via
   `${query.filter.aql}` in the TOML `path_template`. No additional params are added.

4. **Cyberint — time-window removal (correctness fix):**
   `from_date` / `to_date` POST-body injection is removed. The endpoint is GET with
   cursor-only (`AlertListParams.cursor`); there is no body_template. Time-window
   is post-filtered by DataFusion. `page_size` push-down remains absent (DTU-EXT-005
   open).

5. **Claroty — time-window removal (correctness fix):**
   Any time-window body injection is removed. `body_template: '{}'` remains empty.
   OffsetLimit URL params (`?offset=N&limit=M`) are the existing correct pagination
   mechanism. Body-based pagination remains deferred to `S-DEMO-CLAROTY-PAGINATION-001`.

6. **All tests use production TOML spec shapes (SAP-2):**
   No fabricated fixtures (`make_crowdstrike_like_spec` etc.) that diverge from
   production shape. End-to-end push-down behavior is tested via the real DTU clones.

7. **AC-EQUIV-001 exercises the real materialization path:**
   Result-equivalence invariant (BC-2.11.007) is verified via
   `run_materialization_pipeline` → DTU clone, not by direct `FetchContext`
   construction.

---

## Behavioral Contracts

| BC ID | Version | Title |
|-------|---------|-------|
| BC-2.01.013 | v1.13 | DataSource Trait Eliminates Per-Sensor Code Duplication — Pagination/Push-Down Scope Clause corrected: CrowdStrike FQL wiring via ADR-033 T1; Armis AQL passthrough only; Cyberint cursor-only; Claroty OffsetLimit URL only. TV-BC-2.01.013-006 re-cast: asserts both start_time AND end_time reach FQL filter via run_materialization_pipeline. |
| BC-2.11.005 | v1.6 | Ephemeral Materialization — cache misses trigger push-down filters; as of v2 start_time/end_time reach fan-out via run_materialization_pipeline per ADR-033 T1; per-sensor translation corrected per BC-2.01.013 v1.13. |
| BC-2.11.007 | v1.7 | Sensor Filter Push-Down — push-down is optimization only; result must be identical with or without push-down. Time-range push-down qualified: CrowdStrike only (native DTU param); Armis/Cyberint/Claroty time-window is DataFusion post-filter. |

---

## Acceptance Criteria

### AC-CWS-001: CrowdStrike limit reaches DetectionListParams (traces to BC-2.01.013 v1.13 Pagination/Push-Down Scope Clause postcondition)

Given: A PrismQL query with `LIMIT 50` is executed against a CrowdStrike sensor.
When: `run_materialization_pipeline` constructs `QueryParams` and the CrowdStrike adapter builds the request.
Then: The CrowdStrike DTU receives `DetectionListParams.limit = Some(50)` as a query param on the `query_detection_ids` Step 1.
An empty LIMIT clause produces no `limit` query param (or `limit` is absent from the request).

Red Gate test: `test_ac_cws_001_crowdstrike_limit_reaches_detection_list_params`
SAP-2 mandate: fixture must use production `crowdstrike.sensor.toml` shape (GET step, no body_template).

(traces to BC-2.01.013 v1.13 Pagination/Push-Down Scope Clause — `limit` query param wired)

### AC-CWS-002: CrowdStrike FQL time-window with both start_time and end_time (traces to BC-2.01.013 v1.13 TV-BC-2.01.013-006)

Given: A PrismQL query with `WHERE created_timestamp > '2026-01-01T00:00:00Z' AND created_timestamp < '2026-06-01T00:00:00Z'` against a CrowdStrike sensor.
When: `run_materialization_pipeline` extracts time-window via ADR-033 T1 heuristic and the CrowdStrike adapter builds the Step 1 request.
Then:
(a) `QueryParams.start_time = Some("2026-01-01T00:00:00Z")` and `QueryParams.end_time = Some("2026-06-01T00:00:00Z")` are populated by `run_materialization_pipeline` before fan-out.
(b) `DetectionListParams.filter` contains `created_timestamp:>'2026-01-01T00:00:00Z'+created_timestamp:<'2026-06-01T00:00:00Z'` (both components present, combined with `+`).
(c) Step 2 (`fetch_detections` POST) receives NO `filter` or time-window params — `FetchContext::default()` on Step 2.
(d) Wiring occurs via `run_materialization_pipeline` (NOT by constructing `FetchContext` directly at a call site — direct construction would bypass the ADR-033 T1 extraction).

Red Gate test: `test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline`
SAP-2 mandate: fixture must derive from production `crowdstrike.sensor.toml` shape.

(traces to BC-2.01.013 v1.13 TV-BC-2.01.013-006 — both start_time AND end_time reach FQL via run_materialization_pipeline)

### AC-CWS-003: CrowdStrike empty-filter case — no filter param when no time predicates (traces to BC-2.11.007 v1.7 result-equivalence invariant)

Given: A PrismQL query against CrowdStrike with NO time-window predicates in the WHERE clause.
When: `run_materialization_pipeline` constructs `QueryParams` and the CrowdStrike adapter builds the Step 1 request.
Then: No `filter` query param is appended to the `query_detection_ids` request (or `filter` is absent / empty string is not sent).
Existing behavior for non-time predicates (e.g., `severity = 'critical'`) is unaffected.

Red Gate test: `test_ac_cws_003_no_filter_param_when_no_time_predicates`
SAP-2 mandate: production `crowdstrike.sensor.toml` shape.

(traces to BC-2.11.007 v1.7 postcondition — push-down reduces data fetched; when no predicate, no param injected; result-equivalence preserved)

### AC-ARMIS-001: Armis AQL passthrough — aql param forwarded; no maxResults or timeFrame (traces to BC-2.11.007 v1.7 §Mechanism B postcondition)

Given: A PrismQL query `FROM armis_devices WHERE aql = 'in:devices lastSeen:>"2026-01-01"' LIMIT 100` against the Armis sensor.
When: `run_materialization_pipeline` fans out and the Armis adapter builds the request.
Then:
(a) The DTU receives `GET /api/v1/search?aql=in:devices+lastSeen:>"2026-01-01"` (or equivalent URL-encoded form) — AQL forwarded verbatim.
(b) `SearchQueryParams` contains NO `maxResults` field.
(c) `SearchQueryParams` contains NO `timeFrame` field.
(d) `SearchQueryParams` does NOT contain any injected time-window parameters beyond what is embedded in the AQL string itself.

Red Gate test: `test_ac_armis_001_aql_passthrough_no_maxresults_no_timeframe`
SAP-2 mandate: fixture must derive from production `armis.sensor.toml` shape (GET, AQL passthrough, OffsetLimit pagination, no body_template).

(traces to BC-2.11.007 v1.7 §Mechanism B — Armis AQL verbatim passthrough; no translation layer; `SearchQueryParams` struct does not have timeFrame/maxResults fields per pushdown-redesign.md §1.2)

### AC-ARMIS-002: Armis push-down produces no additional params beyond aql, offset, limit (traces to BC-2.11.007 v1.7 §Mechanism B postcondition)

Given: A PrismQL query against Armis with a time-window predicate in the WHERE clause (e.g., `WHERE aql = 'in:devices' AND detected_time > '2026-01-01T00:00:00Z'`).
When: `run_materialization_pipeline` fans out and the Armis adapter builds the request.
Then:
(a) `SearchQueryParams` contains only `aql` (from the AQL passthrough), `offset`, and `limit` (OffsetLimit pagination).
(b) The time-window predicate `detected_time > '2026-01-01T00:00:00Z'` is NOT injected as a separate query param — it is either embedded in the AQL string (if the user put it there) or post-filtered by DataFusion.
(c) No 400 or 422 error is returned by the DTU due to unexpected params.

Red Gate test: `test_ac_armis_002_no_additional_params_beyond_aql_offset_limit`
SAP-2 mandate: production `armis.sensor.toml` shape.

(traces to BC-2.11.007 v1.7 invariant — push-down does not change results; Armis has no native time-window param; post-filter applies)

### AC-CYB-001: Cyberint fetch_alerts receives no from_date, to_date, or page_size (traces to BC-2.01.013 v1.13 Pagination/Push-Down Scope Clause — Cyberint row)

Given: A PrismQL query against Cyberint with a time-window predicate in the WHERE clause.
When: `run_materialization_pipeline` fans out and the Cyberint adapter builds the request.
Then:
(a) The HTTP request to the DTU is GET (no POST body).
(b) `AlertListParams` contains NO `from_date` or `to_date` field.
(c) `AlertListParams` contains NO `page_size` field.
(d) The only field that MAY appear in `AlertListParams` is `cursor: Option<String>` (for pagination progression).
(e) No 400 or unexpected-field error from the DTU.

Red Gate test: `test_ac_cyb_001_no_from_date_to_date_page_size_in_alert_list_params`
SAP-2 mandate: fixture must derive from production `cyberint.sensor.toml` shape (GET, cursor-only, NO body_template per pushdown-redesign.md §1.3).

(traces to BC-2.01.013 v1.13 Pagination/Push-Down Scope Clause — Cyberint row: GET endpoint, cursor-only; POST-body injection was WRONG and is now removed)

### AC-CLAR-001: Claroty fetch receives no time-window body fields; body_template remains empty (traces to BC-2.01.013 v1.13 Pagination/Push-Down Scope Clause — Claroty row)

Given: A PrismQL query against Claroty with a time-window predicate in the WHERE clause.
When: `run_materialization_pipeline` fans out and the Claroty adapter builds the request.
Then:
(a) The POST body to the DTU is `{}` (empty object — `body_template: '{}'`).
(b) The body contains NO time-window fields (no `detected_after`, `detected_before`, `start_time`, `end_time`, or similar).
(c) Pagination is via URL params `?offset=N&limit=M` (OffsetLimit pipeline — existing correct behavior).
(d) No body-injection fields are added.

Red Gate test: `test_ac_clar_001_claroty_body_template_remains_empty_no_time_fields`
SAP-2 mandate: fixture must derive from production `claroty.sensor.toml` shape (POST, `body_template: '{}'`, OffsetLimit URL params per pushdown-redesign.md §1.4).

(traces to BC-2.01.013 v1.13 Pagination/Push-Down Scope Clause — Claroty row: OffsetLimit URL params only; body-based injection was WRONG and is now removed)

### AC-WIRE-001: run_materialization_pipeline populates QueryParams.start_time and end_time from PrismQL AST (traces to ADR-033 §Decision — T1 heuristic; BC-2.01.013 v1.13 TV-BC-2.01.013-006)

Given: A PrismQL query with `WHERE created_timestamp > '2026-01-01T00:00:00Z'` against a CrowdStrike sensor whose spec declares `created_timestamp` as `column_type = "datetime"`.
When: `run_materialization_pipeline` (in `prism-query/src/materialization.rs`) executes and constructs `QueryParams` for fan-out.
Then:
(a) `QueryParams.start_time = Some("2026-01-01T00:00:00Z")` is populated before fan-out.
(b) `QueryParams.end_time = None` (no end predicate in this query).
(c) The extraction occurs inside `run_materialization_pipeline` via the ADR-033 T1 function — NOT by constructing `FetchContext` directly at the callsite.
(d) When `MaterializationContext.resolved_spec_map` is `None`, extraction silently returns `None` values (safe default — no panic, no push-down).

Red Gate test: `test_ac_wire_001_materialization_pipeline_populates_start_time_from_ast`
Complementary test: `test_ac_wire_001b_safe_default_when_spec_map_is_none`
SAP-2 mandate: uses real `run_materialization_pipeline` + production spec shape.

(traces to ADR-033 §Decision — T1 pre-fan-out heuristic; BC-2.01.013 v1.13 TV-BC-2.01.013-006 wiring via run_materialization_pipeline)

### AC-EQUIV-001: Result-equivalence invariant via real materialization path (traces to BC-2.11.007 v1.7 invariant — push-down is optimization only)

Given: The same PrismQL query (e.g., `FROM crowdstrike_detections WHERE created_timestamp > '2026-01-01T00:00:00Z' LIMIT 20`) is executed:
(a) With the push-down path active (via `run_materialization_pipeline` → ADR-033 T1 → CrowdStrike FQL injection → DTU clone).
(b) Without push-down (time predicate applied post-materialization only by DataFusion; DTU returns all available records).
When: Both executions complete against the CrowdStrike DTU clone.
Then: The result set from (a) is a subset of the result set from (b) that is consistent with the time-window and LIMIT applied post-materialization. No row appears in (a) that was not in (b) for the same time range. Push-down must not fabricate or drop rows beyond what the predicate specifies.

This AC validates BC-2.11.007 invariant: "push-down is an optimization only; the query result must be identical whether or not push-down occurs."

CRITICAL: This test MUST exercise the REAL materialization path (`run_materialization_pipeline` → fan-out → `SpecDrivenSensorAdapter::fetch` → DTU clone). A unit test that constructs `FetchContext` directly and bypasses `run_materialization_pipeline` does NOT satisfy this AC (it would miss the dead-code gap F-P6-CRIT-001 that motivated ADR-033).

Red Gate test: `test_ac_equiv_001_result_equivalence_via_real_materialization_path`
SAP-2 mandate: production `crowdstrike.sensor.toml` shape; real DTU clone (ungated integration test per SID-1).

(traces to BC-2.11.007 v1.7 invariant — result-equivalence; also closes F-P6-MED-001)

---

## SAP-2 Standing AC Gate

**MANDATORY for adversary pass 1:** Verify that every test fixture constructing a sensor spec satisfies ALL of the following:

| Sensor | Required fixture properties | FORBIDDEN |
|--------|----------------------------|-----------|
| CrowdStrike | GET step (Step 1), no body_template, `DetectionListParams` fields: `filter`, `limit`, `offset` | Any fixture with a body_template on Step 1; any fixture injecting `timeFrame` or `maxResults` |
| Armis | GET, `path_template = "/api/v1/search?aql=..."`, OffsetLimit pagination, no body_template | Any fixture with `timeFrame`, `maxResults`, or body injection |
| Cyberint | GET, NO body_template, `AlertListParams.cursor` only, no `from_date`/`to_date`/`page_size` | Any fixture with a POST body_template; any fixture injecting body fields |
| Claroty | POST, `body_template: '{}'` (empty), OffsetLimit URL params | Any fixture injecting time-window body fields |

If a fabricated-fixture helper function (`make_crowdstrike_like_spec`, `make_cyberint_like_spec`, `make_armis_like_spec`) is present in the test code AND does not match the above production properties, that is a **P1 CRITICAL finding** that resets the 3-CLEAN streak.

Preferred approach: load specs via `include_str!` from production `crates/prism-sensors/specs/*.sensor.toml`, or construct a minimal strict subset that shares the same `method`, `body_template` presence/absence, pagination `type`, and step count.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| ADR-033 T1: time-window extraction lives in `prism-query` (materialization.rs + pushdown.rs) | ADR-033 §Decision | Do NOT implement extraction in prism-spec-engine or prism-bin; it must be in prism-query |
| `run_materialization_pipeline` must call the T1 extraction before constructing `QueryParams` | ADR-033 §Decision | AC-WIRE-001 Red Gate test verifies; direct FetchContext construction at call site is NON-CONFORMANT |
| Step 2 (`fetch_detections`) MUST receive `FetchContext::default()` — no push-down | BC-2.01.013 v1.13 EC-01-027 | AC-CWS-002 item (c) verifies Step 2 receives no filter/limit |
| CrowdStrike FQL combines start+end with `+` when both present | BC-2.01.013 v1.13 per-sensor table | AC-CWS-002 item (b) verifies combined form |
| Armis: REMOVAL of maxResults/timeFrame; AQL passthrough ONLY | BC-2.11.007 v1.7 §Mechanism B | AC-ARMIS-001/002 assert absence; no extra params |
| Cyberint: REMOVAL of from_date/to_date/page_size body injection; GET with cursor only | BC-2.01.013 v1.13 Cyberint row | AC-CYB-001 asserts absence |
| Claroty: body_template remains `'{}'`; no time-window injection | BC-2.01.013 v1.13 Claroty row | AC-CLAR-001 asserts empty body |
| Result-equivalence invariant preserved (BC-2.11.007) | BC-2.11.007 v1.7 invariant | AC-EQUIV-001 via real materialization path |
| `FetchContext` additions MUST be `Option<T>` with `Default::default() = None` | BC-2.01.013 v1.13 FetchContext clause | All existing callers pass `FetchContext::default()` unaffected |
| `MaterializationContext.resolved_spec_map` None = safe default (no push-down) | ADR-033 §Consequences — Negative trade-offs | AC-WIRE-001b Red Gate test verifies |
| Forbidden dependency: `prism-query` MUST NOT gain a new dependency on `prism-bin` | ADR-022 §C wiring constraints | Implementer must verify Cargo.toml dependency direction |

---

## Out-of-Scope (Named Follow-Ups)

These are **entire features** deferred to named stories — not partial implementations:

| Deferred scope | Reason | Follow-up anchor |
|----------------|--------|-----------------|
| Cyberint `page_size` push-down | `AlertListParams` has no `page_size` field; DTU-EXT-005 open | DTU-EXT-005 + new story when that gap closes |
| Claroty body-based offset/limit | Gap-CL-004; real Claroty API expects body offset/limit; DTU currently accepts URL OffsetLimit only | `S-DEMO-CLAROTY-PAGINATION-001` (open, P1) |
| Claroty native time-window push-down | No time-window param in current DTU structs; DTU must be extended first | `S-DEMO-CLAROTY-TIME-001` (stub registered, draft) |
| Full `classify_predicates` integration (Option T2) | Requires fan-out orchestration restructuring; wave-6 scope | Future ADR + story when fan-out restructuring is designed |
| Cursor seeding from PrismQL WHERE clause | No PrismQL syntax for initial cursor value yet | Future story when PrismQL cursor syntax is defined |

**Production-grade compliance:** every deferral above is an entire feature (not a partial implementation with correctness gaps). The v2 story either delivers CORRECT behavior for a dimension or explicitly anchors it to a named story above.

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `prism-query` (workspace) | current workspace path | materialization.rs + pushdown.rs — ADR-033 T1 extraction |
| `prism-spec-engine` (workspace) | current workspace path | pipeline.rs — remove wrong Cyberint/Claroty/Armis translations |
| `prism-bin` (workspace) | current workspace path | spec_driven_adapter.rs — Armis AQL passthrough verification; CrowdStrike FQL wiring |
| `chrono` | workspace version | DateTime<Utc> / to_rfc3339() for ISO8601 formatting in T1 extraction |
| `serde_json` | workspace version | Claroty POST body assertions in tests |
| `reqwest` | workspace version | Query param assertion helpers in integration tests |

Version source: workspace `Cargo.toml` `[dependencies]` table. Do not pin versions independently. Use only workspace-pinned versions — do NOT introduce new crate dependencies.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-query/src/pushdown.rs` | MODIFY | Add `extract_time_window_from_ast` (or extension of `extract_push_down_filters_as_map`) — ADR-033 T1 heuristic; walks Compare nodes on datetime columns |
| `crates/prism-query/src/materialization.rs` | MODIFY | Wire T1 extraction at lines ~434–440; populate `QueryParams.start_time` / `QueryParams.end_time` from extracted values (replaces hardcoded `None`) |
| `crates/prism-spec-engine/src/pipeline.rs` | MODIFY | Remove wrong Cyberint `from_date`/`to_date` POST-body injection; remove wrong Claroty time-window body injection; verify Armis has no `maxResults`/`timeFrame` injection |
| `crates/prism-bin/src/spec_driven_adapter.rs` | MODIFY | Verify Armis AQL passthrough is correct (no extra params); verify CrowdStrike FQL injection uses start+end combined with `+` |
| `crates/prism-query/src/tests/` or inline `#[cfg(test)]` in `pushdown.rs` / `materialization.rs` | MODIFY | Red Gate tests for AC-WIRE-001, AC-WIRE-001b, AC-EQUIV-001 |
| `crates/prism-spec-engine/src/pipeline/tests.rs` or inline | MODIFY | Red Gate tests for AC-CWS-001/002/003, AC-ARMIS-001/002, AC-CYB-001, AC-CLAR-001 |
| Integration test (ungated) | CREATE | AC-EQUIV-001 integration test against real DTU clone — `run_materialization_pipeline` → CrowdStrike DTU |

**Note on pre-existing fabricated fixtures:** If `make_crowdstrike_like_spec`, `make_cyberint_like_spec`, or `make_armis_like_spec` exist in the test suite from v1.x, they MUST be replaced or verified to match production TOML shape before any test using them is considered load-bearing. If they diverge, they must be deleted and replaced with production-TOML-derived fixtures.

---

## Tasks

1. **Read** `crates/prism-query/src/materialization.rs` — locate `run_materialization_pipeline`, `extract_push_down_filters_as_map`, and the `QueryParams` construction at lines ~434–440. Understand why `start_time`/`end_time` are hardcoded `None`.
2. **Read** `crates/prism-query/src/pushdown.rs` — understand `predicate_tree_to_filter_map`, `classify_predicates`, and the existing predicate-walk logic. Identify where the T1 extraction function should be added.
3. **Read** `crates/prism-spec-engine/src/pipeline.rs` — locate `build_request()` and the per-sensor translation arms. Identify the Cyberint POST-body injection and Claroty time-window injection sites to remove.
4. **Read** `crates/prism-bin/src/spec_driven_adapter.rs` — confirm AQL passthrough path and CrowdStrike FQL injection site.
5. **Read** production sensor TOMLs — `crowdstrike.sensor.toml`, `armis.sensor.toml`, `cyberint.sensor.toml`, `claroty.sensor.toml` — to understand step structure, method, body_template presence/absence, pagination type.
6. **Read** DTU route structs — `prism-dtu-crowdstrike/src/routes/detections.rs` (`DetectionListParams`), `prism-dtu-armis/src/routes/search.rs` (`SearchQueryParams`), `prism-dtu-cyberint/src/routes/alerts.rs` (`AlertListParams`) — to understand what fields are actually available.
7. **Write stubs** — stub out T1 extraction function with `todo!()` in `pushdown.rs`. Stub CrowdStrike FQL combined-form injection with `todo!()`.
8. **Write Red Gate tests** (all 9 must FAIL before implementation):
   - `test_ac_cws_001_crowdstrike_limit_reaches_detection_list_params`
   - `test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline`
   - `test_ac_cws_003_no_filter_param_when_no_time_predicates`
   - `test_ac_armis_001_aql_passthrough_no_maxresults_no_timeframe`
   - `test_ac_armis_002_no_additional_params_beyond_aql_offset_limit`
   - `test_ac_cyb_001_no_from_date_to_date_page_size_in_alert_list_params`
   - `test_ac_clar_001_claroty_body_template_remains_empty_no_time_fields`
   - `test_ac_wire_001_materialization_pipeline_populates_start_time_from_ast`
   - `test_ac_wire_001b_safe_default_when_spec_map_is_none`
9. **Verify Red Gate fails** — `just iter prism-query` and `just iter prism-spec-engine` must show all 9 Red Gate tests FAILING.
10. **Implement** ADR-033 T1: add `extract_time_window_from_ast` in `pushdown.rs`; wire in `materialization.rs` at lines ~434–440.
11. **Implement** CrowdStrike FQL combined form: `start+end` with `+`; Step 2 receives `FetchContext::default()`.
12. **Remove** wrong Cyberint `from_date`/`to_date` POST-body injection from `pipeline.rs`.
13. **Remove** wrong Claroty time-window body injection from `pipeline.rs`.
14. **Verify** Armis AQL passthrough in `spec_driven_adapter.rs`: confirm `maxResults`/`timeFrame` are absent.
15. **Write** AC-EQUIV-001 integration test (ungated, against CrowdStrike DTU clone): `run_materialization_pipeline` → DTU → result-equivalence assertion.
16. **Run** `just iter prism-query --no-fail-fast` — all 9 Red Gate tests GREEN.
17. **Run** `just iter prism-spec-engine` and `just iter prism-bin` — no regressions.
18. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence

- **S-DEMO-001** (MERGED PR #166): Delivers `SpecDrivenSensorAdapter`, `PipelineExecutor::execute()`, `FetchContext`, and `build_request()`. This story extends those exact types. The S-DEMO-001 implementation is the ground truth for the current FetchContext struct definition and `build_request()` signature.
- **S-DEMO-002** (MERGED PR #171): Established AQL seeding convention (`FetchContext.query_filters["aql"]` → `${query.filter.aql}` interpolation). AC-ARMIS-001/002 verify correct non-injection behavior; S-DEMO-002 must be merged (SATISFIED).
- **v1.x implementation (SUPERSEDED):** The prior v1.x implementation introduced wrong per-sensor translations (Armis `maxResults`/`timeFrame`, Cyberint POST-body injection, Claroty body injection) and missed the materialization.rs `None` hardcode. All v1.x test code using fabricated fixtures is superseded. Do NOT build on v1.x code — re-derive from this corrected spec.
- **LOCAL adversary passes 5+6 findings (2026-06-05):**
  - F-P6-CRIT-001: `materialization.rs` lines ~434–440 hardcode `start_time: None, end_time: None` — all time-window push-down is dead code. Closed by AC-WIRE-001.
  - F-P6-MED-001: result-equivalence AC-005 in v1.x used direct `FetchContext` construction, bypassing `run_materialization_pipeline`. Closed by AC-EQUIV-001 (real materialization path mandate).
  - Per-sensor factual errors: Armis/Cyberint/Claroty translation bugs. Closed by AC-ARMIS-001/002, AC-CYB-001, AC-CLAR-001.
- **ADR-033** (proposed, 2026-06-05): Records the T1 vs T2 architecture decision. T1 adopted for v2 scope. T2 deferred.
- **BC-2.11.007 invariant:** "push-down is an optimization only; the query result must be identical whether or not push-down occurs." This is the non-negotiable correctness invariant that AC-EQUIV-001 validates.
- **BC-2.01.013 v1.13 TV-BC-2.01.013-006:** Re-cast in v1.13 to assert BOTH `start_time` AND `end_time` reach the CrowdStrike FQL filter via `run_materialization_pipeline`. AC-CWS-002 is the story-level assertion for this.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `QueryParams.start_time` set but `end_time` is `None` (open-ended range) | Push start_time to CrowdStrike FQL as `created_timestamp:>'<ISO8601>'`; no end component. For Armis/Cyberint/Claroty: no injection (they have no native time param). |
| EC-002 | Both start_time and end_time set for CrowdStrike | FQL: `created_timestamp:>'<start>'` + `created_timestamp:<'<end>'` combined with `+` — single combined `filter` param. |
| EC-003 | `start_time > end_time` (inverted range) | Log warning at WARN level (structured field: `event_type = "push_down.inverted_time_range"`); push both params to the sensor API; sensor returns empty result or error; `PipelineExecutor` handles normally. Result-equivalence invariant is preserved because DataFusion post-filter would produce the same empty result for the inverted range. |
| EC-004 | `column_type` on lhs column is not `"datetime"` | T1 extraction silently skips this Compare predicate; it falls through to DataFusion post-filter. No push-down. |
| EC-005 | `MaterializationContext.resolved_spec_map` is `None` at T1 extraction time | Extraction silently returns `None` for both `start_time` and `end_time`. No push-down occurs. No panic. |
| EC-006 | Armis query has NO `aql` predicate in WHERE clause (no AQL passthrough possible) | AQL passthrough is absent; `${query.filter.aql}` in the path_template interpolates to empty string or is omitted per spec engine behavior. DataFusion post-filters. No crash. |
| EC-007 | Claroty query has time-window predicate; `body_template: '{}'` | Body remains `{}` (empty). Time predicate is post-filtered by DataFusion. No injection. |
| EC-008 | FetchContext.limit = Some(0) | Treat as no limit — a limit of 0 is semantically meaningless; silently ignore; DataFusion applies LIMIT post-materialization. |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec (v2.0) | ~5,000 |
| BC files (3 BCs: BC-2.01.013 v1.13 + BC-2.11.005 v1.6 + BC-2.11.007 v1.7) | ~9,000 |
| ADR-033 | ~2,500 |
| pushdown-redesign.md (design note) | ~4,000 |
| crates/prism-query/src/materialization.rs (post-S-DEMO-001) | ~10,000 |
| crates/prism-query/src/pushdown.rs | ~4,000 |
| crates/prism-spec-engine/src/pipeline.rs (post-S-DEMO-001) | ~10,000 |
| crates/prism-bin/src/spec_driven_adapter.rs (post-S-DEMO-001) | ~4,000 |
| Production sensor TOMLs (4 files) | ~6,000 |
| DTU route structs (3 files: detections.rs + search.rs + alerts.rs) | ~3,000 |
| Test outputs (cargo nextest) | ~2,000 |
| **Total estimate** | **~59,500 tokens (~23% of 256K context)** |

Within the 20-30% budget. Single-story delivery is viable. If context becomes tight, the implementer may split into two sub-tasks: (1) ADR-033 T1 wiring in prism-query; (2) correctness fixes in prism-spec-engine + prism-bin.

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 2.0 | 2026-06-05 | story-writer | Major re-author (v1.3 → v2.0). Motivation: LOCAL adversary passes 5+6 established v1.x implementation is inert against production sensor shapes (materialization.rs hardcodes None; wrong Armis/Cyberint/Claroty translations). New scope: crates_touched adds prism-query (ADR-033 T1 time-window extraction in materialization.rs + pushdown.rs). subsystems adds SS-11 (Query Execution). target_module changed to prism-query. points 5→8 (T1 extraction + SAP-2 compliant test suite). AC set fully replaced: AC-CWS-001/002/003 (CrowdStrike limit + FQL time-window both start+end + empty-filter); AC-ARMIS-001/002 (AQL passthrough; assert NO maxResults/NO timeFrame); AC-CYB-001 (cursor-only GET; assert NO from_date/to_date/page_size); AC-CLAR-001 (empty body; assert NO time-window injection); AC-WIRE-001 (run_materialization_pipeline populates start_time+end_time per ADR-033 T1); AC-EQUIV-001 (result-equivalence via REAL materialization path — not direct FetchContext construction). SAP-2 Standing AC Gate added (production-TOML fixture mandate; fabricated-fixture P1 CRITICAL gate). Out-of-scope follow-ups anchored: Cyberint page_size → DTU-EXT-005; Claroty body pagination → S-DEMO-CLAROTY-PAGINATION-001; Claroty time-window → S-DEMO-CLAROTY-TIME-001 (new stub); full classify_predicates (Option T2) → future wave-6. BC table body updated with v1.13/v1.6/v1.7 version citations. Token Budget BC count updated to 3 BCs (unchanged). inputs[] expanded: adds prism-query src files + sensor TOMLs + DTU route structs + ADR-033 + pushdown-redesign.md. depends_on adds S-DEMO-002 (SATISFIED). v1.x implementation superseded. |
| 1.3 | 2026-06-05 | state-manager | F-PUSHDOWN2-MED-001: status sync — frontmatter `status: ready`→`in_progress`; body header `**Status:** ready`→`in_progress`; body H1 version label v1.2→v1.3. D-1002 burst introduced asymmetry between STORY-INDEX (badged `in_progress v1.2`) and this story file (still `ready`). Source-of-Truth Rule 5: active LOCAL cascade → `in_progress` is canonical. Version bumped to v1.3 to maintain POLICY 32 monotonic-descending changelog. |
| 1.2 | 2026-06-05 | story-writer | F-PUSHDOWN-006: removed VP-031 from verification_properties (VP-031 covers required-column rejection in prism-query / S-3.02 — unrelated to push-down threading; mis-anchor). No push-down VP exists in VP-INDEX (156 VPs checked); new-VP need flagged for PO/architect in frontmatter note. F-PUSHDOWN-007: updated BC-2.11.005 row in Behavioral Contracts table with PO-specified affected-but-indirectly-tested relationship note. Body header version/status updated to v1.2/ready. Token Budget BC count (3 BCs) remains consistent. |
| 1.1 | 2026-06-03 | state-manager | D-990 Phase-A-close: status draft→ready; depends_on S-DEMO-001 SATISFIED (merged PR #166); BC-2.01.013 v1.11 active + BC-2.11.005 active + BC-2.11.007 active — S-7.01 gate CLEARED. |
| 1.0 | 2026-05-31 | story-writer | Initial draft — created per S-DEMO-001 v1.5 AC-010 scope note and BC-2.01.013 v1.8 Pagination/Push-Down Scope Clause (D-924). Scope: thread FetchContext push-down fields (cursor/limit/start_time/end_time) from SpecDrivenSensorAdapter::fetch() into PipelineExecutor build_request(). P2 non-blocking — correctness holds via DataFusion post-materialization. |
