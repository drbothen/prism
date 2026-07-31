---
document_type: story
story_id: S-DEMO-QUERY-PUSHDOWN-001
title: "prism-query + prism-spec-engine + prism-bin: Correct per-sensor push-down wiring — CrowdStrike FQL time-window + limit; Armis AQL correctness; materialization.rs wiring (ADR-033 T1)"
wave: wave-5-e-demo-fidelity
epic_id: E-DEMO
priority: P2
status: merged
version: "2.8"
level: "L3"
producer: story-writer
revised_by: state-manager
timestamp: "2026-06-06T00:00:00Z"
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
#   Note (v2.2): prism-dtu-crowdstrike also touches SS-16 scope — the CrowdStrike DTU clone
#     now honors the filter= FQL time-window param by parsing created_timestamp bounds and
#     filtering its fixture dataset (AC-CWS-DTU-001, parallel to prism-dtu-armis §8.3 work).
#     No new subsystem ID is required; the DTU clone behavioral contract falls under SS-16.
crates_touched: [prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-dtu-crowdstrike, prism-sensors]
# crates_touched v2 change: prism-query ADDED (was [prism-spec-engine, prism-bin] in v1.x).
# prism-query is required because ADR-033 Option T1 places the time-window extraction function
# in materialization.rs / pushdown.rs inside prism-query — this is the primary new scope in v2.
# crates_touched v2.1 change: prism-dtu-armis ADDED — DTU must parse and honor AQL time clauses
# (pushdown-redesign.md §8.3) to make Armis time-window scenarios load-bearing (non-vacuous).
# prism-sensors ADDED — armis.sensor.toml needs options=["INDEX"] on last_seen (devices) and
# created_at (alerts) datetime columns so Option T1 can identify them as push-down-eligible
# (pushdown-redesign.md §8.5 + AC-INDEX-001).
# crates_touched v2.2 change: prism-dtu-crowdstrike ADDED — the OBS-001 fix-burst (LOCAL adversary
# pass 2) added FQL time-window honoring to the CrowdStrike DTU: state.rs FQL parsing
# (parse_fql_time_bounds), /dtu/filter-log capture route (mod.rs), and
# detections.rs fixture filtering (filtered < unfiltered). Without this entry the
# spec omitted a crate that was materially modified — POLICY 13 spec/impl consistency
# violation (DRIFT-P1-001 ADV-P02-HIGH-001). One-line AC-CWS-DTU-001 closes the gap.
target_module: prism-query
capabilities: [CAP-015]
behavioral_contracts:
  - BC-2.01.013  # DataSource Trait / SpecDrivenSensorAdapter Push-Down Scope Clause (v1.14):
                 # Per-sensor push-down translation table corrected per pushdown-redesign.md §6+§8.
                 # v2.1 adds Armis AQL-clause augmentation (time-window IN scope per human directive
                 # 2026-06-05); anti-double-filter guard; DTU-honors-AQL-time-clause contract;
                 # research-doc armis-aql-time-window-syntax-2026-06.md + ADR-033 cited.
  - BC-2.11.005  # Ephemeral Materialization (v1.6): cache misses on first/query-plan step
                 # trigger sensor API calls with push-down filters. As of v2, start_time/end_time
                 # reach the fan-out via run_materialization_pipeline per ADR-033 T1.
  - BC-2.11.007  # Sensor Filter Push-Down (v1.8): push-down is optimization only; result must
                 # be identical whether or not push-down occurs (result-equivalence invariant).
                 # Time-range push-down: CrowdStrike (FQL injection) AND Armis (AQL-clause
                 # augmentation, v1.8 per human directive). Cyberint/Claroty remain post-filter only.
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
#   - materialization.rs callsite wiring (QueryParams construction inside run_materialization_pipeline's per-target fan-out loop, None → extracted values): ~1 pt
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
acceptance_criteria_count: 18
red_gate_tests: 20
estimated_passes: "3-4 LOCAL adversary passes"
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
  - "crates/prism-dtu-crowdstrike/src/state.rs"
  - "crates/prism-dtu-crowdstrike/src/routes/detections.rs"
  - "crates/prism-dtu-crowdstrike/src/routes/mod.rs"
  - "crates/prism-dtu-armis/src/routes/search.rs"
  - "crates/prism-dtu-armis/src/routes/state.rs"
  - "crates/prism-dtu-cyberint/src/routes/alerts.rs"
  - "crates/prism-spec-engine/tests/parity/armis.rs"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.005-ephemeral-materialization.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md"
  - ".factory/specs/architecture/decisions/ADR-033-push-down-time-window-extraction-strategy-pre-fan-out-heuristic.md"
  - ".factory/cycles/wave-5-e-demo-fidelity/S-DEMO-QUERY-PUSHDOWN-001/pushdown-redesign.md"
  - ".factory/research/armis-aql-time-window-syntax-2026-06.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-QUERY-PUSHDOWN-001 v2.8 — Correct per-sensor push-down wiring (ADR-033 T1 + Armis AQL full wiring + CrowdStrike DTU FQL honoring)

**Story ID:** S-DEMO-QUERY-PUSHDOWN-001
**Status:** in_progress
**Version:** v2.8
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
| BC-2.01.013 | v1.14 | DataSource Trait Eliminates Per-Sensor Code Duplication — Pagination/Push-Down Scope Clause: CrowdStrike FQL wiring via ADR-033 T1; Armis AQL-clause augmentation (time-window IN scope v1.14 per human directive 2026-06-05) with anti-double-filter guard + DTU-honors-AQL-time-clause contract; Cyberint cursor-only; Claroty OffsetLimit URL only. TV-BC-2.01.013-006 re-cast: asserts both start_time AND end_time reach FQL filter via run_materialization_pipeline. Research-confirmed AQL syntax: bare unquoted `after:YYYY-MM-DDTHH:MM:SS` / `before:YYYY-MM-DDTHH:MM:SS`. |
| BC-2.11.005 | v1.6 | Ephemeral Materialization — cache misses trigger push-down filters; as of v2 start_time/end_time reach fan-out via run_materialization_pipeline per ADR-033 T1; per-sensor translation corrected per BC-2.01.013 v1.14. |
| BC-2.11.007 | v1.8 | Sensor Filter Push-Down — push-down is optimization only; result must be identical with or without push-down. Time-range push-down: CrowdStrike (FQL injection) AND Armis (AQL-clause augmentation via Mechanism B, v1.8 per human directive 2026-06-05); Cyberint/Claroty time-window is DataFusion post-filter only. |

---

## Acceptance Criteria

### AC-CWS-001: CrowdStrike limit reaches DetectionListParams (traces to BC-2.01.013 v1.14 Pagination/Push-Down Scope Clause postcondition)

Given: A PrismQL query with `LIMIT 50` is executed against a CrowdStrike sensor.
When: `run_materialization_pipeline` constructs `QueryParams` and the CrowdStrike adapter builds the request.
Then: The CrowdStrike DTU receives `DetectionListParams.limit = Some(50)` as a query param on the `query_detection_ids` Step 1.
An empty LIMIT clause produces no `limit` query param (or `limit` is absent from the request).

Red Gate test: `test_ac_cws_001_crowdstrike_limit_reaches_detection_list_params`
SAP-2 mandate: fixture must use production `crowdstrike.sensor.toml` shape (GET step, no body_template).

(traces to BC-2.01.013 v1.14 Pagination/Push-Down Scope Clause — `limit` query param wired)

### AC-CWS-002: CrowdStrike FQL time-window with both start_time and end_time (traces to BC-2.01.013 v1.14 TV-BC-2.01.013-006)

Given: A PrismQL query with `WHERE created_timestamp > '2026-01-01T00:00:00Z' AND created_timestamp < '2026-06-01T00:00:00Z'` against a CrowdStrike sensor.
When: `run_materialization_pipeline` extracts time-window via ADR-033 T1 heuristic and the CrowdStrike adapter builds the Step 1 request.
Then:
(a) `QueryParams.start_time = Some("2026-01-01T00:00:00Z")` and `QueryParams.end_time = Some("2026-06-01T00:00:00Z")` are populated by `run_materialization_pipeline` before fan-out.
(b) `DetectionListParams.filter` contains `created_timestamp:>'2026-01-01T00:00:00Z'+created_timestamp:<'2026-06-01T00:00:00Z'` (both components present, combined with `+`).
(c) Step 2 (`fetch_detections` POST) receives NO `filter` or time-window params — `FetchContext::default()` on Step 2.
(d) Wiring occurs via `run_materialization_pipeline` (NOT by constructing `FetchContext` directly at a call site — direct construction would bypass the ADR-033 T1 extraction).

Red Gate test: `test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline`
SAP-2 mandate: fixture must derive from production `crowdstrike.sensor.toml` shape.

(traces to BC-2.01.013 v1.14 TV-BC-2.01.013-006 — both start_time AND end_time reach FQL via run_materialization_pipeline)

### AC-CWS-003: CrowdStrike empty-filter case — no filter param when no time predicates (traces to BC-2.11.007 v1.8 result-equivalence invariant)

Given: A PrismQL query against CrowdStrike with NO time-window predicates in the WHERE clause.
When: `run_materialization_pipeline` constructs `QueryParams` and the CrowdStrike adapter builds the Step 1 request.
Then: No `filter` query param is appended to the `query_detection_ids` request (or `filter` is absent / empty string is not sent).
Existing behavior for non-time predicates (e.g., `severity = 'critical'`) is unaffected.

Red Gate test: `test_ac_cws_003_no_filter_param_when_no_time_predicates`
SAP-2 mandate: production `crowdstrike.sensor.toml` shape.

(traces to BC-2.11.007 v1.8 postcondition — push-down reduces data fetched; when no predicate, no param injected; result-equivalence preserved)

### AC-CWS-WIRE-001: CrowdStrike FQL filter AND limit both reach the DTU wire simultaneously — wire-level combined verification (traces to BC-2.01.013 v1.14 Pagination/Push-Down Scope Clause + BC-2.11.007 v1.8 Mechanism A)

Given: A PrismQL query with a time-window predicate (`created_timestamp > '2026-01-20T00:00:00Z'`) AND a `LIMIT 3` clause against a CrowdStrike sensor.
When: `run_materialization_pipeline` → `SpecDrivenSensorAdapter::fetch` → `PipelineExecutor::execute` → CrowdStrike DTU (`/queries/detections/v1`).
Then:
(a) The production `crowdstrike.sensor.toml` `query_detection_ids` step structurally declares BOTH a FQL filter slot (`query.filter.*`) AND a limit slot (`query.limit`) in its `path_template`.
(b) The CrowdStrike DTU `/dtu/filter-log` response contains a `filter_strings` entry that includes `created_timestamp` — confirming the FQL time-window reached the DTU wire via `path_template` interpolation.
(c) `result.records.len() <= 3` — the LIMIT is honored by the DTU simultaneously with the FQL filter.
(d) `result.records` is non-empty — fixture contains records after `2026-01-20T00:00:00Z`.

This is a wire-level combined verification: both the FQL time-window (AC-CWS-002) and the limit (AC-CWS-001) reach the DTU simultaneously. The AC-CWS-001 and AC-CWS-002 tests validate each dimension independently; this AC is the combined wire-level gate (closes LOCAL F-P1-HIGH-003).

Red Gate test: `test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu`
Location: `crates/prism-spec-engine/tests/bc_2_11_007_pushdown_test.rs`
(Existing test — 18 code sites cite this AC ID. The test verifies structural slots exist in the TOML, then exercises both FQL filter + limit=3 in a single combined pipeline execution, asserts filter reached DTU via filter-log, and asserts result.len() <= 3.)

(traces to BC-2.01.013 v1.14 Pagination/Push-Down Scope Clause — both `limit` query param and FQL `filter` param wired simultaneously; BC-2.11.007 v1.8 Mechanism A — CrowdStrike FQL injection is optimization only; result is bounded subset consistent with both constraints)

### AC-CWS-DTU-001: CrowdStrike DTU honors filter= FQL time-window — filtered_count < unfiltered_count (traces to BC-2.11.007 v1.8 result-equivalence invariant + Mechanism A CrowdStrike FQL postcondition)

Given: The `prism-dtu-crowdstrike` clone receives `GET /queries/detections/v1?filter=created_timestamp:>'2026-01-01T00:00:00Z'` where the fixture dataset contains detection records both BEFORE and AFTER the timestamp.
When: The DTU processes the FQL `filter` param by invoking `parse_fql_time_bounds` (state.rs) and filtering the fixture dataset in `detections.rs`.
Then:
(a) The returned detection IDs contain ONLY records whose `created_timestamp` is > 2026-01-01T00:00:00Z.
(b) `filtered_count < unfiltered_count` — the DTU MUST filter its fixture dataset by the FQL time clause; if `filtered_count == unfiltered_count` the test FAILS (vacuous scenario).
(c) Every record in the filtered result also appears in the unfiltered result (no record fabrication).
(d) A `GET /dtu/filter-log` request returns the most-recent filter expression applied (captured via the `/dtu/filter-log` capture route added in mod.rs).
(e) The filter applies to BOTH `created_timestamp:>'T'` (lower-bound) and `created_timestamp:<'T'` (upper-bound) FQL clauses; combined `+` form filters both bounds simultaneously.

LOAD-BEARING assertion: item (b) is the critical gate. Without DTU-side FQL honoring, AC-CWS-002's end-to-end push-down scenario is vacuous — the DTU would return all records regardless of the `filter=` param, making time-window assertions unreliable.
SAP-2 mandate: production `crowdstrike.sensor.toml` shape; fixture dataset must span the test time boundary (records on both sides of the threshold).

Red Gate test: `test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window`
(new test authored by implementer in this fix-burst — must FAIL before `parse_fql_time_bounds` + detections.rs filtering are implemented, GREEN after; `filtered_count < unfiltered_count` assertion is the load-bearing line)

(traces to BC-2.11.007 v1.8 Mechanism A — CrowdStrike FQL injection; DTU must parse and honor created_timestamp FQL bounds so push-down scenarios are non-vacuous; parallel to AC-ARMIS-TW-002 for Armis DTU)

### AC-ARMIS-001: Armis AQL passthrough — aql param forwarded; no maxResults or timeFrame (traces to BC-2.11.007 v1.8 §Mechanism B postcondition)

Given: A PrismQL query `FROM armis_devices WHERE aql = 'in:devices after:2026-01-01T00:00:00' LIMIT 100` against the Armis sensor.
When: `run_materialization_pipeline` fans out and the Armis adapter builds the request.
Then:
(a) The DTU receives `GET /api/v1/search?aql=in:devices+after:2026-01-01T00:00:00` (or equivalent URL-encoded form) — AQL forwarded verbatim.
(b) `SearchQueryParams` contains NO `maxResults` field.
(c) `SearchQueryParams` contains NO `timeFrame` field.
(d) `SearchQueryParams` does NOT contain any injected time-window parameters beyond what is embedded in the AQL string itself.

NOTE: The `after:2026-01-01T00:00:00` form is the research-confirmed canonical Armis AQL absolute
lower-bound syntax (bare, unquoted, timezone-naive `YYYY-MM-DDTHH:MM:SS`). The prior form
`lastSeen:>"2026-01-01"` used in the v2.0 story was NOT a confirmed Armis AQL operator — it is a
comparison-operator form absent from all attested Armis sources (see
`.factory/research/armis-aql-time-window-syntax-2026-06.md` §2.3). MUST NOT revert to the
`lastSeen:>"T"` form.

Red Gate test: `test_ac_armis_001_aql_passthrough_no_maxresults_no_timeframe`
SAP-2 mandate: fixture must derive from production `armis.sensor.toml` shape (GET, AQL passthrough, OffsetLimit pagination, no body_template).

(traces to BC-2.11.007 v1.8 §Mechanism B — Armis AQL verbatim passthrough; no translation layer; `SearchQueryParams` struct does not have timeFrame/maxResults fields per pushdown-redesign.md §1.2)

### AC-ARMIS-002: Armis push-down produces no additional params beyond aql, offset, limit (traces to BC-2.11.007 v1.8 §Mechanism B postcondition)

Given: A PrismQL query against Armis with a time-window predicate in the WHERE clause (e.g., `WHERE aql = 'in:devices' AND detected_time > '2026-01-01T00:00:00Z'`).
When: `run_materialization_pipeline` fans out and the Armis adapter builds the request.
Then:
(a) `SearchQueryParams` contains only `aql` (from the AQL passthrough), `offset`, and `limit` (OffsetLimit pagination).
(b) The time-window predicate `detected_time > '2026-01-01T00:00:00Z'` is NOT injected as a separate query param — it is either embedded in the AQL string (if the user put it there) or post-filtered by DataFusion.
(c) No 400 or 422 error is returned by the DTU due to unexpected params.

Red Gate test: `test_ac_armis_002_no_additional_params_beyond_aql_offset_limit`
SAP-2 mandate: production `armis.sensor.toml` shape.

(traces to BC-2.11.007 v1.8 invariant — push-down does not change results; base AQL is forwarded verbatim without additional query params beyond aql/offset/limit; Armis time-window handled via AQL-clause augmentation in AC-ARMIS-TW-001..005)

### AC-ARMIS-TW-001: Armis time-window AQL augmentation — PrismQL last_seen predicate appended as `after:` clause (traces to BC-2.01.013 v1.14 Mechanism B AQL augmentation + BC-2.11.007 v1.8 §Mechanism B)

Given: A PrismQL query `SELECT * FROM armis_devices WHERE aql = 'in:devices' AND last_seen > '2026-01-01T00:00:00Z'` against the Armis sensor, where `last_seen` is declared `column_type = "datetime"` with `options = ["INDEX"]` in `armis.sensor.toml`.
When: `run_materialization_pipeline` extracts time bounds via ADR-033 T1 and applies Armis AQL-clause augmentation.
Then:
(a) `QueryParams.filters["aql"]` is `"in:devices after:2026-01-01T00:00:00"` — base AQL plus the canonical time clause (bare, unquoted, timezone-naive `YYYY-MM-DDTHH:MM:SS` per research-doc `armis-aql-time-window-syntax-2026-06.md` §2.2).
(b) For a bounded range (`last_seen > 'T1' AND last_seen < 'T2'`): `QueryParams.filters["aql"]` is `"in:devices after:T1 before:T2"` (space-separated, no `AND` keyword).
(c) Assertion occurs at the FilterMap/QueryParams boundary (unit test in `prism-query` — no external dependency required; SID-1 compliant).
(d) MUST NOT emit `lastSeen:>"T"` form — this is NOT a confirmed Armis AQL filter operator.

Red Gate test: `test_ac_armis_tw_001_time_window_augmented_into_aql`
(Unit test in `prism-query/src/tests/` or inline `pushdown.rs`; exercises `augment_armis_aql_with_time_window` at the FilterMap boundary without DTU dependency.)

(traces to BC-2.01.013 v1.14 Mechanism B AQL-clause augmentation; BC-2.11.007 v1.8 §Mechanism B — time-window push-down for Armis via AQL string augmentation)

### AC-ARMIS-TW-002: Armis DTU returns only records within time window (LOAD-BEARING — filtered < unfiltered) (traces to BC-2.11.007 v1.8 §Mechanism B DTU-honors-AQL-time-clause contract)

Given: The `prism-dtu-armis` clone receives `GET /api/v1/search?aql=in:devices+after:2026-01-01T00:00:00` where the fixture dataset contains device records both BEFORE and AFTER the timestamp.
When: The DTU processes the AQL string including the `after:` clause.
Then:
(a) The returned device records contain ONLY records whose `last_seen` (or `first_seen` as fallback if `last_seen` is null) is >= 2026-01-01T00:00:00.
(b) `filtered_count < unfiltered_count` — the DTU MUST filter its fixture dataset by the time clause; if `filtered_count == unfiltered_count` the test FAILS (vacuous scenario).
(c) Every record in the filtered result also appears in the unfiltered result (no record fabrication).
(d) Records with both `last_seen: null` and `first_seen: null` are EXCLUDED from time-filtered results.

LOAD-BEARING assertion: item (b) is the critical gate. The DTU must honor the `after:`/`before:` clause in the AQL string by filtering `routes/search.rs` fixture dataset accordingly (pushdown-redesign.md §8.3). Without this, time-window scenarios are vacuous.
SAP-2 mandate: pipeline integration test in `prism-spec-engine/tests/parity/armis.rs` using real Armis DTU clone (ungated per SID-1 §2 — DTU is internal, always available in CI).

Red Gate test: `test_ac_armis_tw_002_dtu_filters_fixture_by_time_window`

(traces to BC-2.11.007 v1.8 §Mechanism B DTU-honors-AQL-time-clause contract — DTU must parse and honor AQL after:/before: clauses)

### AC-ARMIS-TW-003: Anti-double-filter guard — user AQL with existing time clause forwarded verbatim (traces to BC-2.01.013 v1.14 Mechanism B anti-double-filter guard)

Given: A PrismQL query `SELECT * FROM armis_devices WHERE aql = 'in:devices after:2026-01-01T00:00:00' AND last_seen > '2026-01-01T00:00:00Z'` where the base AQL already contains `after:`.
When: The AQL augmentation logic runs.
Then:
(a) `QueryParams.filters["aql"]` is `"in:devices after:2026-01-01T00:00:00"` — forwarded VERBATIM. No second `after:` clause appended.
(b) The check applies to ALL canonical time keywords: if the base AQL contains any of `after:`, `before:`, or `timeFrame:`, no augmentation occurs.
(c) Unit test verifies the exact forwarded AQL string equals the user's literal value.

Red Gate test: `test_ac_armis_tw_003_anti_double_filter_guard`

(traces to BC-2.01.013 v1.14 Mechanism B anti-double-filter guard — user's explicit time scope is preserved when already embedded in AQL)

### AC-ARMIS-TW-004: Armis result-equivalence — AQL time push-down returns same records as DataFusion post-filter (traces to BC-2.11.007 v1.8 result-equivalence invariant)

Given: Two PrismQL queries against the Armis DTU clone:
(a) Query WITH `last_seen > 'T'` push-down (AQL augmented; DTU filters fixture) AND DataFusion post-filter.
(b) Query WITHOUT push-down (no AQL time clause; DTU returns full fixture) but WITH DataFusion post-filter on `last_seen > 'T'`.
When: Both queries complete against the same Armis DTU clone instance.
Then: The result sets from (a) and (b) are IDENTICAL — same records, order-independent comparison. No record appears in (a) that is not in (b), and vice versa (for the same time predicate).

This AC validates BC-2.11.007 invariant: push-down is an optimization only. The correctness backstop is DataFusion post-filter regardless of DTU filtering.

Red Gate test: `test_ac_armis_tw_004_result_equivalence_pushdown_vs_postfilter`
SAP-2 mandate: integration test against real Armis DTU clone; production `armis.sensor.toml` shape.

(traces to BC-2.11.007 v1.8 result-equivalence invariant — push-down must not change query results for Armis)

### AC-ARMIS-TW-005: E2E — prism binary AQL log contains augmented AQL with time clause (traces to BC-2.11.007 v1.8 §Mechanism B end-to-end)

Given: The prism binary is running against the Armis DTU clone, and a PrismQL query `SELECT * FROM armis_devices WHERE aql = 'in:devices' AND last_seen > '2024-01-01T00:00:00Z'` is issued.
When: The query executes end-to-end.
Then:
(a) prism returns non-empty data rows.
(b) The Armis DTU aql-log contains an entry with BOTH the entity discriminator (`in:devices`) AND the time clause (`after:2024-01-01T00:00:00`) — confirming augmentation reached the DTU wire.
(c) Result row count <= full unfiltered row count from the same DTU.

`#[ignore]` annotation required per SID-1 / E2E-001 (requires DTU + prism binary; un-gated via e2e profile).
LOAD-BEARING: Assertion (b) fails if the query-engine AQL augmentation is absent from the wire path.

Red Gate test: `test_ac_armis_tw_005_e2e_aql_log_contains_augmented_aql`

(traces to BC-2.11.007 v1.8 §Mechanism B — end-to-end AQL augmentation confirmed via DTU aql-log)

### AC-INDEX-001: armis.sensor.toml — last_seen (devices) and created_at (alerts) declare `options = ["INDEX"]` (traces to BC-2.01.013 v1.14 Mechanism B — AQL augmentation requires INDEX datetime columns)

Given: The `armis.sensor.toml` file declares device and alert tables.
When: The spec is loaded and Option T1 time-window extraction runs.
Then:
(a) The `last_seen` column in the `[[tables]]` block for `armis_devices` (or equivalent table name) declares `options = ["INDEX"]` in addition to `column_type = "datetime"`.
(b) The `created_at` column in the `[[tables]]` block for `armis_alerts` (or equivalent) declares `options = ["INDEX"]` in addition to `column_type = "datetime"`.
(c) With this change, `extract_time_window_from_ast` can identify `last_seen` and `created_at` as push-down-eligible datetime columns for Armis via the Option T1 heuristic.

NOTE: The existing `armis.sensor.toml` has `options = ["INDEX"]` only on `aql`. The `last_seen` and `created_at` columns are declared `column_type = "datetime"` but WITHOUT `options = ["INDEX"]`, making them invisible to Option T1 time-window extraction (pushdown-redesign.md §8.5). This AC closes that gap.

Red Gate test: Inline TOML parse test OR `test_ac_index_001_armis_toml_last_seen_created_at_have_index_option` (asserts `ColumnOptions::Index` present on both columns after spec loading).

(traces to BC-2.01.013 v1.14 Mechanism B AQL augmentation — Option T1 requires INDEX datetime columns to identify push-down-eligible predicates in armis.sensor.toml)

### AC-INDEX-CWS-001: crowdstrike.sensor.toml — created_timestamp declares `options = ["INDEX"]` (traces to BC-2.01.013 v1.14 Mechanism A — FQL time-window push-down requires INDEX datetime column)

Given: The `crowdstrike.sensor.toml` file declares the `crowdstrike_detections` table.
When: The spec is loaded and Option T1 time-window extraction runs against a PrismQL query with a `created_timestamp` predicate.
Then:
(a) The `created_timestamp` column in the `[[tables]]` block for `crowdstrike_detections` declares `column_type = "datetime"` AND `options = ["INDEX"]`.
(b) With this declaration, `extract_time_window_from_ast` identifies `created_timestamp` as push-down-eligible and populates `QueryParams.start_time` / `QueryParams.end_time` via the ADR-033 T1 heuristic.
(c) Without `options = ["INDEX"]`, Option T1 extraction silently skips the `created_timestamp` Compare predicate — making CrowdStrike FQL time-window push-down silently vacuous (parallel defect to the Armis AC-INDEX-001 gap).

NOTE: This is the CrowdStrike parallel to AC-INDEX-001 (Armis). The `crowdstrike.sensor.toml` MUST declare `options = ["INDEX"]` on `created_timestamp` so that `extract_time_window_from_ast` can recognize it as push-down-eligible. Without it, AC-CWS-002's end-to-end time-window scenario is vacuous even if the FQL injection code is correct.

Red Gate test: `test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option`
Location: `crates/prism-spec-engine/tests/bc_2_11_007_pushdown_test.rs`
(Existing test, already passes — asserts `ColumnOptions::Index` present on `created_timestamp` in the `crowdstrike_detections` table after spec loading.)

(traces to BC-2.01.013 v1.14 Mechanism A CrowdStrike FQL time-window — Option T1 requires `options = ["INDEX"]` on `created_timestamp` in crowdstrike.sensor.toml to identify it as push-down-eligible; without it, FQL time-window push-down is silently vacuous)

### AC-CYB-001: Cyberint fetch_alerts receives no from_date, to_date, or page_size (traces to BC-2.01.013 v1.14 Pagination/Push-Down Scope Clause — Cyberint row)

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

(traces to BC-2.01.013 v1.14 Pagination/Push-Down Scope Clause — Cyberint row: GET endpoint, cursor-only; POST-body injection was WRONG and is now removed)

### AC-CLAR-001: Claroty fetch receives no time-window body fields; body_template remains empty (traces to BC-2.01.013 v1.14 Pagination/Push-Down Scope Clause — Claroty row)

Given: A PrismQL query against Claroty with a time-window predicate in the WHERE clause.
When: `run_materialization_pipeline` fans out and the Claroty adapter builds the request.
Then:
(a) The POST body to the DTU is `{}` (empty object — `body_template: '{}'`).
(b) The body contains NO time-window fields (no `detected_after`, `detected_before`, `start_time`, `end_time`, or similar).
(c) Pagination is via URL params `?offset=N&limit=M` (OffsetLimit pipeline — existing correct behavior).
(d) No body-injection fields are added.

Red Gate test: `test_ac_clar_001_claroty_body_template_remains_empty_no_time_fields`
SAP-2 mandate: fixture must derive from production `claroty.sensor.toml` shape (POST, `body_template: '{}'`, OffsetLimit URL params per pushdown-redesign.md §1.4).

(traces to BC-2.01.013 v1.14 Pagination/Push-Down Scope Clause — Claroty row: OffsetLimit URL params only; body-based injection was WRONG and is now removed)

### AC-WIRE-001: run_materialization_pipeline populates QueryParams.start_time and end_time from PrismQL AST (traces to ADR-033 §Decision — T1 heuristic; BC-2.01.013 v1.14 TV-BC-2.01.013-006)

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

(traces to ADR-033 §Decision — T1 pre-fan-out heuristic; BC-2.01.013 v1.14 TV-BC-2.01.013-006 wiring via run_materialization_pipeline)

### AC-EQUIV-001: Result-equivalence invariant via real materialization path (traces to BC-2.11.007 v1.8 invariant — push-down is optimization only)

Given: The same PrismQL query (e.g., `FROM crowdstrike_detections WHERE created_timestamp > '2026-01-01T00:00:00Z' LIMIT 20`) is executed:
(a) With the push-down path active (via `run_materialization_pipeline` → ADR-033 T1 → CrowdStrike FQL injection → DTU clone).
(b) Without push-down (time predicate applied post-materialization only by DataFusion; DTU returns all available records).
When: Both executions complete against the CrowdStrike DTU clone.
Then: The result set from (a) is a subset of the result set from (b) that is consistent with the time-window and LIMIT applied post-materialization. No row appears in (a) that was not in (b) for the same time range. Push-down must not fabricate or drop rows beyond what the predicate specifies.

This AC validates BC-2.11.007 invariant: "push-down is an optimization only; the query result must be identical whether or not push-down occurs."

CRITICAL: This test MUST exercise the REAL materialization path (`run_materialization_pipeline` → fan-out → `SpecDrivenSensorAdapter::fetch` → DTU clone). A unit test that constructs `FetchContext` directly and bypasses `run_materialization_pipeline` does NOT satisfy this AC (it would miss the dead-code gap F-P6-CRIT-001 that motivated ADR-033).

Red Gate test: `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline`
Location: `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`
This test drives `run_materialization_pipeline` end-to-end and asserts the BC-2.11.007 subset/no-fabrication invariant: every row in the push-down result set is present in the full (unpushed) result set for the same time range, and no row is fabricated or silently dropped beyond what the predicate specifies.
Supplementary (boundary test, prism-spec-engine): `test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary` — exercises the PipelineExecutor boundary directly; does NOT satisfy this AC alone (bypasses `run_materialization_pipeline`).
SAP-2 mandate: production `crowdstrike.sensor.toml` shape; real DTU clone (ungated integration test per SID-1).

(traces to BC-2.11.007 v1.8 invariant — result-equivalence; also closes F-P6-MED-001)

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
| Step 2 (`fetch_detections`) MUST receive `FetchContext::default()` — no push-down | BC-2.01.013 v1.14 EC-01-027 | AC-CWS-002 item (c) verifies Step 2 receives no filter/limit |
| CrowdStrike FQL combines start+end with `+` when both present | BC-2.01.013 v1.14 per-sensor table | AC-CWS-002 item (b) verifies combined form |
| Armis: REMOVAL of maxResults/timeFrame; AQL passthrough + time-window AQL-clause augmentation | BC-2.01.013 v1.14 Mechanism B + BC-2.11.007 v1.8 §Mechanism B | AC-ARMIS-001/002 assert absence of extra params; AC-ARMIS-TW-001..005 assert correct augmentation |
| Cyberint: REMOVAL of from_date/to_date/page_size body injection; GET with cursor only | BC-2.01.013 v1.14 Cyberint row | AC-CYB-001 asserts absence |
| Claroty: body_template remains `'{}'`; no time-window injection | BC-2.01.013 v1.14 Claroty row | AC-CLAR-001 asserts empty body |
| Result-equivalence invariant preserved (BC-2.11.007) | BC-2.11.007 v1.8 invariant | AC-EQUIV-001 + AC-ARMIS-TW-004 via real materialization path |
| `FetchContext` additions MUST be `Option<T>` with `Default::default() = None` | BC-2.01.013 v1.14 FetchContext clause | All existing callers pass `FetchContext::default()` unaffected |
| `MaterializationContext.resolved_spec_map` None = safe default (no push-down) | ADR-033 §Consequences — Negative trade-offs | AC-WIRE-001b Red Gate test verifies |
| Forbidden dependency: `prism-query` MUST NOT gain a new dependency on `prism-bin` | ADR-022 §C wiring constraints | Implementer must verify Cargo.toml dependency direction |
| Armis AQL augmentation: `after:YYYY-MM-DDTHH:MM:SS` / `before:YYYY-MM-DDTHH:MM:SS` — bare, unquoted, timezone-naive | research-doc `armis-aql-time-window-syntax-2026-06.md` §2.2 | MUST NOT use `lastSeen:>"T"` form (unattested); MUST NOT append `Z` suffix |
| Anti-double-filter guard: if base AQL contains `after:`, `before:`, or `timeFrame:` → forward verbatim, no augmentation | BC-2.01.013 v1.14 Mechanism B anti-double-filter guard | AC-ARMIS-TW-003 verifies guard; failure → duplicate time clauses on wire |
| Armis DTU `routes/search.rs`: parse `after:`/`before:` from AQL string and filter fixture dataset | pushdown-redesign.md §8.3 + BC-2.01.013 v1.14 DTU-honors-AQL-time-clause contract | AC-ARMIS-TW-002 LOAD-BEARING assertion: `filtered_count < unfiltered_count` |
| `armis.sensor.toml`: `last_seen` (devices) and `created_at` (alerts) MUST declare `options = ["INDEX"]` | pushdown-redesign.md §8.5 | AC-INDEX-001 verifies; without this, Option T1 cannot extract Armis time bounds |
| `crowdstrike.sensor.toml`: `created_timestamp` MUST declare `options = ["INDEX"]` | ADR-033 T1 heuristic — Option T1 only matches columns with `options = ["INDEX"]` | AC-INDEX-CWS-001 verifies; without this, CrowdStrike FQL time-window push-down is silently vacuous (parallel to AC-INDEX-001) |
| R-DTU-002 (opaque AQL capture) is UNAFFECTED: `capture_aql()` still called verbatim before any filtering | pushdown-redesign.md §8.3.1 | AC-ARMIS-TW-005 aql-log assertion confirms verbatim capture of augmented string |
| CrowdStrike DTU (`prism-dtu-crowdstrike`): `state.rs` `parse_fql_time_bounds` parses FQL `created_timestamp:>'T'` and `created_timestamp:<'T'` clauses; `detections.rs` filters fixture dataset; `/dtu/filter-log` capture route records applied filter expression | DRIFT-P1-001 ADV-P02-HIGH-001 — OBS-001 fix-burst; parallel to Armis §8.3 DTU requirement | AC-CWS-DTU-001 LOAD-BEARING: `filtered_count < unfiltered_count`; without this, AC-CWS-002 end-to-end scenario is vacuous |

---

## Out-of-Scope (Named Follow-Ups)

These are **entire features** deferred to named stories — not partial implementations:

| Deferred scope | Reason | Follow-up anchor |
|----------------|--------|-----------------|
| Cyberint `page_size` push-down | `AlertListParams` has no `page_size` field; DTU-EXT-005 open | DTU-EXT-005 + new story when that gap closes |
| Claroty body-based offset/limit | Gap-CL-004; real Claroty API expects body offset/limit; DTU currently accepts URL OffsetLimit only | `S-DEMO-CLAROTY-PAGINATION-001` (open, P1) |
| Claroty native time-window push-down | No time-window param in current DTU structs; DTU must be extended first | `S-DEMO-CLAROTY-TIME-001` (stub registered, draft) |
| Full `classify_predicates` integration (Option T2) | REQUIRED-column plan-time enforcement gate (E-QUERY-009) anchored to `S-REQUIRED-COL-GATE-001` — uses `resolved_spec_map` pre-fan-out per BC-2.11.007 §REQUIRED Column Runtime Mechanism, no fan-out restructuring required. Full post-resolution per-sensor `classify_predicates` integration for non-REQUIRED push-down dimensions requires fan-out orchestration restructuring — separate future story not yet scoped. | `S-REQUIRED-COL-GATE-001` (plan-time E-QUERY-009 gate); future story for remaining T2 fan-out restructure (non-REQUIRED dimensions) |
| Cursor seeding from PrismQL WHERE clause | No PrismQL syntax for initial cursor value yet | Future story when PrismQL cursor syntax is defined |

**Production-grade compliance:** every deferral above is an entire feature (not a partial implementation with correctness gaps). The v2.1 story either delivers CORRECT behavior for a dimension or explicitly anchors it to a named story above.

**Armis time-window is IN scope in v2.1** (human directive 2026-06-05, §8 of pushdown-redesign.md). It is NOT deferred. The implementation uses AQL-clause augmentation via `after:`/`before:` syntax (research-confirmed bare, unquoted, timezone-naive form). The prior v2.0 note treating Armis time-window as "post-filter only" is superseded.

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `prism-query` (workspace) | current workspace path | materialization.rs + pushdown.rs — ADR-033 T1 extraction; AQL augmentation |
| `prism-spec-engine` (workspace) | current workspace path | pipeline.rs — remove wrong Cyberint/Claroty/Armis translations; Armis AQL augmentation branch |
| `prism-bin` (workspace) | current workspace path | spec_driven_adapter.rs — Armis AQL passthrough verification; CrowdStrike FQL wiring |
| `prism-dtu-armis` (workspace) | current workspace path | routes/search.rs — AQL time-clause parsing + fixture dataset filtering (§8.3) |
| `prism-dtu-crowdstrike` (workspace) | current workspace path | state.rs `parse_fql_time_bounds` + routes/detections.rs fixture filtering + routes/mod.rs `/dtu/filter-log` capture route — FQL time-window honoring (AC-CWS-DTU-001) |
| `prism-sensors` (workspace) | current workspace path | specs/armis.sensor.toml — add `options = ["INDEX"]` to last_seen + created_at datetime columns |
| `chrono` | workspace version | DateTime<Utc> / strftime `%Y-%m-%dT%H:%M:%S` for timezone-naive ISO8601 in T1 extraction and AQL augmentation |
| `serde_json` | workspace version | Claroty POST body assertions in tests |
| `reqwest` | workspace version | Query param assertion helpers in integration tests |

Version source: workspace `Cargo.toml` `[dependencies]` table. Do not pin versions independently. Use only workspace-pinned versions — do NOT introduce new crate dependencies.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-query/src/pushdown.rs` | MODIFY | Add `extract_time_window_from_ast` (or extension of `extract_push_down_filters_as_map`) — ADR-033 T1 heuristic; walks Compare nodes on datetime columns |
| `crates/prism-query/src/materialization.rs` | MODIFY | Wire T1 extraction at the `QueryParams` construction inside `run_materialization_pipeline`'s per-target fan-out loop; populate `QueryParams.start_time` / `QueryParams.end_time` from extracted values (replaces hardcoded `None`) |
| `crates/prism-spec-engine/src/pipeline.rs` | MODIFY | Remove wrong Cyberint `from_date`/`to_date` POST-body injection; remove wrong Claroty time-window body injection; add Armis AQL-clause augmentation branch (`augment_armis_aql_with_time_window`) |
| `crates/prism-bin/src/spec_driven_adapter.rs` | MODIFY | Verify Armis AQL passthrough is correct (no extra params); verify CrowdStrike FQL injection uses start+end combined with `+` |
| `crates/prism-query/src/tests/` or inline `#[cfg(test)]` in `pushdown.rs` / `materialization.rs` | MODIFY | Red Gate tests for AC-WIRE-001, AC-WIRE-001b, AC-EQUIV-001, AC-ARMIS-TW-001, AC-ARMIS-TW-003 |
| `crates/prism-spec-engine/src/pipeline/tests.rs` or inline | MODIFY | Red Gate tests for AC-CWS-001/002/003, AC-ARMIS-001/002, AC-CYB-001, AC-CLAR-001 |
| `crates/prism-dtu-crowdstrike/src/state.rs` | MODIFY | Add `parse_fql_time_bounds` — parses `created_timestamp:>'T'` (lower) and `created_timestamp:<'T'` (upper) FQL clauses from the `filter` query param; returns `(Option<DateTime<Utc>>, Option<DateTime<Utc>>)` |
| `crates/prism-dtu-crowdstrike/src/routes/detections.rs` | MODIFY | Apply `parse_fql_time_bounds` result to filter fixture detection dataset before pagination; ensures `filtered_count < unfiltered_count` for load-bearing test AC-CWS-DTU-001 |
| `crates/prism-dtu-crowdstrike/src/routes/mod.rs` | MODIFY | Add `/dtu/filter-log` capture route that records the most-recent FQL `filter` expression applied; enables AC-CWS-DTU-001 item (d) assertion without reading internal state |
| `crates/prism-dtu-armis/src/routes/search.rs` | MODIFY | Add AQL time-clause parsing and fixture dataset filtering per §8.3 — parse `after:`/`before:` from AQL string, filter devices by `last_seen` and alerts by `created_at` |
| `crates/prism-sensors/specs/armis.sensor.toml` | MODIFY | Add `options = ["INDEX"]` to `last_seen` (devices table) and `created_at` (alerts table) datetime columns |
| Integration test (ungated, `prism-spec-engine/tests/parity/armis.rs`) | MODIFY | AC-ARMIS-TW-002 (LOAD-BEARING: filtered < unfiltered), AC-ARMIS-TW-004 (result-equivalence), AC-EQUIV-001 for CrowdStrike |
| E2E test (`prism-bin/tests/e2e_smoke.rs`, `#[ignore]`) | MODIFY | AC-ARMIS-TW-005 — aql-log confirms augmented AQL including entity discriminator + time clause |

**Note on pre-existing fabricated fixtures:** If `make_crowdstrike_like_spec`, `make_cyberint_like_spec`, or `make_armis_like_spec` exist in the test suite from v1.x, they MUST be replaced or verified to match production TOML shape before any test using them is considered load-bearing. If they diverge, they must be deleted and replaced with production-TOML-derived fixtures.

---

## Tasks

1. **Read** `crates/prism-query/src/materialization.rs` — locate `run_materialization_pipeline`, `extract_push_down_filters_as_map`, and the `QueryParams` construction inside `run_materialization_pipeline`'s per-target fan-out loop. Understand why `start_time`/`end_time` are hardcoded `None`.
2. **Read** `crates/prism-query/src/pushdown.rs` — understand `predicate_tree_to_filter_map`, `classify_predicates`, and the existing predicate-walk logic. Identify where the T1 extraction function should be added.
3. **Read** `crates/prism-spec-engine/src/pipeline.rs` — locate `build_request()` and the per-sensor translation arms. Identify the Cyberint POST-body injection and Claroty time-window injection sites to remove.
4. **Read** `crates/prism-bin/src/spec_driven_adapter.rs` — confirm AQL passthrough path and CrowdStrike FQL injection site.
5. **Read** production sensor TOMLs — `crowdstrike.sensor.toml`, `armis.sensor.toml`, `cyberint.sensor.toml`, `claroty.sensor.toml` — to understand step structure, method, body_template presence/absence, pagination type.
6. **Read** DTU route structs — `prism-dtu-crowdstrike/src/routes/detections.rs` (`DetectionListParams`), `prism-dtu-armis/src/routes/search.rs` (`SearchQueryParams`), `prism-dtu-cyberint/src/routes/alerts.rs` (`AlertListParams`) — to understand what fields are actually available.
6b. **Read** `crates/prism-dtu-crowdstrike/src/state.rs`, `src/routes/detections.rs`, and `src/routes/mod.rs` — understand existing FQL parsing state, fixture dataset structure, and the `/dtu/filter-log` capture route. This is the CrowdStrike DTU that was extended in the OBS-001 fix-burst; AC-CWS-DTU-001 verifies that extension is load-bearing.
6c. **Read** `crates/prism-dtu-armis/src/routes/search.rs` and `state.rs` — understand `get_search` handler, `capture_aql`, entity-type discrimination, and pagination logic. This is the DTU that must be extended for time-window filtering (§8.3).
6d. **Read** `crates/prism-spec-engine/tests/parity/armis.rs` — understand existing AQL passthrough tests and the `#[ignore]`'d parity test so the new load-bearing tests complement rather than duplicate.
6e. **Read** `.factory/research/armis-aql-time-window-syntax-2026-06.md` — confirm canonical AQL time syntax: `after:YYYY-MM-DDTHH:MM:SS` (bare, unquoted, no `Z` suffix). This is authoritative; do NOT use `lastSeen:>"T"` form.
7. **Write stubs** — stub out T1 extraction function with `todo!()` in `pushdown.rs`. Stub CrowdStrike FQL combined-form injection with `todo!()`. Stub `augment_armis_aql_with_time_window` with `todo!()` in `prism-spec-engine/pipeline.rs` or `prism-bin/spec_driven_adapter.rs`.
8. **Write Red Gate tests** (all non-`#[ignore]` tests must FAIL before implementation; 20 total RGTs including 2 from EC-009, 1 for AC-INDEX-CWS-001, and 1 for AC-CWS-WIRE-001):
   - `test_ac_cws_001_crowdstrike_limit_reaches_detection_list_params`
   - `test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline`
   - `test_ac_cws_003_no_filter_param_when_no_time_predicates`
   - `test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu` (AC-CWS-WIRE-001; `crates/prism-spec-engine/tests/bc_2_11_007_pushdown_test.rs`; already passes — EXISTING test, wire-level combined verification)
   - `test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window`
   - `test_ac_armis_001_aql_passthrough_no_maxresults_no_timeframe`
   - `test_ac_armis_002_no_additional_params_beyond_aql_offset_limit`
   - `test_ac_armis_tw_001_time_window_augmented_into_aql`
   - `test_ac_armis_tw_002_dtu_filters_fixture_by_time_window`
   - `test_ac_armis_tw_003_anti_double_filter_guard`
   - `test_ac_armis_tw_004_result_equivalence_pushdown_vs_postfilter`
   - `test_ac_armis_tw_005_e2e_aql_log_contains_augmented_aql` (`#[ignore]` per SID-1/E2E-001)
   - `test_ac_cyb_001_no_from_date_to_date_page_size_in_alert_list_params`
   - `test_ac_clar_001_claroty_body_template_remains_empty_no_time_fields`
   - `test_ac_wire_001_materialization_pipeline_populates_start_time_from_ast`
   - `test_ac_wire_001b_safe_default_when_spec_map_is_none`
   - `test_ac_index_001_armis_toml_last_seen_created_at_have_index_option`
   - `test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option` (AC-INDEX-CWS-001; `crates/prism-spec-engine/tests/bc_2_11_007_pushdown_test.rs`; already passes — EXISTING test)
   - `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline` (EC-009)
   - `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline` (EC-009)
9. **Verify Red Gate fails** — `just iter prism-query` and `just iter prism-spec-engine` and `just iter prism-dtu-armis` and `just iter prism-dtu-crowdstrike` must show all Red Gate tests FAILING.
10. **Implement** ADR-033 T1: add `extract_time_window_from_ast` in `pushdown.rs`; wire into the `QueryParams` construction inside `run_materialization_pipeline`'s per-target fan-out loop in `materialization.rs`.
11. **Implement** CrowdStrike FQL combined form: `start+end` with `+`; Step 2 receives `FetchContext::default()`.
12. **Remove** wrong Cyberint `from_date`/`to_date` POST-body injection from `pipeline.rs`.
13. **Remove** wrong Claroty time-window body injection from `pipeline.rs`.
14. **Verify** Armis AQL passthrough in `spec_driven_adapter.rs`: confirm `maxResults`/`timeFrame` are absent.
14b. **Implement** `augment_armis_aql_with_time_window` in `prism-spec-engine/pipeline.rs` (or `prism-bin/spec_driven_adapter.rs` — keep consistent with where CrowdStrike FQL injection lands): construct `after:YYYY-MM-DDTHH:MM:SS` form from extracted `start_time`; `before:YYYY-MM-DDTHH:MM:SS` from `end_time`; space-separated; no `AND`, no quotes, no `Z` suffix. Include anti-double-filter guard.
14c. **Extend** `crates/prism-dtu-armis/src/routes/search.rs`: after `capture_aql()` (do not move/remove it), parse `after:`/`before:` clauses from AQL string (regex or simple string extraction); filter `devices_ordered` by `last_seen` (with `first_seen` fallback for nulls) and `alert_fixture` by `created_at` before pagination. Null timestamps excluded. Verify fixture data has records spanning the test time window (add records if needed).
14d. **Add** `options = ["INDEX"]` to `last_seen` column (devices table) and `created_at` column (alerts table) in `crates/prism-sensors/specs/armis.sensor.toml`.
15. **Write** AC-EQUIV-001 integration test (ungated, against CrowdStrike DTU clone): `run_materialization_pipeline` → DTU → result-equivalence assertion.
16. **Run** `just iter prism-query --no-fail-fast` + `just iter prism-spec-engine` + `just iter prism-dtu-armis` + `just iter prism-dtu-crowdstrike` — all 20 Red Gate tests GREEN (AC-ARMIS-TW-005 is `#[ignore]`, excluded from this count; AC-INDEX-CWS-001 and AC-CWS-WIRE-001 tests already pass — EXISTING tests).
17. **Run** `just iter prism-spec-engine` and `just iter prism-bin` — no regressions.
18. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence

- **S-DEMO-001** (MERGED PR #166): Delivers `SpecDrivenSensorAdapter`, `PipelineExecutor::execute()`, `FetchContext`, and `build_request()`. This story extends those exact types. The S-DEMO-001 implementation is the ground truth for the current FetchContext struct definition and `build_request()` signature.
- **S-DEMO-002** (MERGED PR #171): Established AQL seeding convention (`FetchContext.query_filters["aql"]` → `${query.filter.aql}` interpolation). AC-ARMIS-001/002 verify correct non-injection behavior; S-DEMO-002 must be merged (SATISFIED).
- **v1.x implementation (SUPERSEDED):** The prior v1.x implementation introduced wrong per-sensor translations (Armis `maxResults`/`timeFrame`, Cyberint POST-body injection, Claroty body injection) and missed the materialization.rs `None` hardcode. All v1.x test code using fabricated fixtures is superseded. Do NOT build on v1.x code — re-derive from this corrected spec.
- **LOCAL adversary passes 5+6 findings (2026-06-05):**
  - F-P6-CRIT-001: the `QueryParams` construction inside `run_materialization_pipeline`'s per-target fan-out loop in `materialization.rs` hardcodes `start_time: None, end_time: None` — all time-window push-down is dead code. Closed by AC-WIRE-001.
  - F-P6-MED-001: result-equivalence AC-005 in v1.x used direct `FetchContext` construction, bypassing `run_materialization_pipeline`. Closed by AC-EQUIV-001 (real materialization path mandate).
  - Per-sensor factual errors: Armis/Cyberint/Claroty translation bugs. Closed by AC-ARMIS-001/002, AC-CYB-001, AC-CLAR-001.
- **ADR-033** (proposed, 2026-06-05): Records the T1 vs T2 architecture decision. T1 adopted for v2 scope. T2 deferred.
- **BC-2.11.007 v1.8 invariant:** "push-down is an optimization only; the query result must be identical whether or not push-down occurs." This is the non-negotiable correctness invariant that AC-EQUIV-001 and AC-ARMIS-TW-004 validate.
- **BC-2.01.013 v1.14 TV-BC-2.01.013-006:** Re-cast in v1.14 to assert BOTH `start_time` AND `end_time` reach the CrowdStrike FQL filter via `run_materialization_pipeline`. AC-CWS-002 is the story-level assertion for this.
- **pushdown-redesign.md §8 (human directive 2026-06-05):** Armis time-window push-down IS in scope via AQL-clause augmentation. The §1.2 "no native time param" position is superseded. The mechanism: extract time bounds via Option T1 → augment base AQL string with `after:T` / `before:T` clauses → forward via existing `${query.filter.aql}` path → DTU parses and filters its fixture dataset. Critical: the DTU MUST honor the time clauses for scenarios to be load-bearing (AC-ARMIS-TW-002).
- **Research-confirmed AQL syntax** (`armis-aql-time-window-syntax-2026-06.md`, HIGH confidence, 6 sources): `after:YYYY-MM-DDTHH:MM:SS` for lower bound, `before:YYYY-MM-DDTHH:MM:SS` for upper bound, space-separated, bare/unquoted, timezone-naive. `lastSeen:>"T"` is NOT a confirmed Armis AQL form — zero sources use field-comparison operators in the AQL string for temporal filtering. DO NOT implement or test with `lastSeen:>"T"`.
- **S-DEMO-ARMIS-AQL-001** (MERGED PR #168): Established `in:devices` / `in:alerts` entity discriminator convention in the Armis DTU and parity tests. The new time-window ACs extend the existing `tests/parity/armis.rs` test suite.

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
| EC-009 | Inclusive time predicate `>=` / `<=` (CompareOp::Ge/Le) with a record whose timestamp == the bound (boundary record) | DTU time-window filtering MUST be inclusive at the boundary: records with `ts == bound` are KEPT (never excluded). Push-down result is a superset of the exact predicate result; DataFusion post-filter narrows to the exact set. BC-2.11.007 result-equivalence invariant holds. Second root cause fixed: timestamp normalization MUST use `to_rfc3339_opts(SecondsFormat::Secs, true)` (emits `Z` suffix) rather than `to_rfc3339()` (emits `+00:00`); because `+`(ASCII 43) < `Z`(ASCII 90) lexicographically, the `+00:00` form caused exact-boundary records to be silently dropped at DataFusion's string-comparison layer. Red Gate tests (in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`): `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline` (drives `run_materialization_pipeline` with `>=` predicate; asserts the boundary record is present in the result set) and `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline` (same for Armis). Both tests MUST FAIL before the inclusive-boundary fix and PASS after. (traces to BC-2.11.007 v1.8 result-equivalence invariant — push-down must never under-fetch; Ge/Le already scoped by BC-2.11.007 v1.8) |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec (v2.8) | ~8,200 |
| BC files (3 BCs: BC-2.01.013 v1.14 + BC-2.11.005 v1.6 + BC-2.11.007 v1.8) | ~9,500 |
| ADR-033 | ~2,500 |
| pushdown-redesign.md (design note incl. §8) | ~6,000 |
| armis-aql-time-window-syntax-2026-06.md (research) | ~3,000 |
| crates/prism-query/src/materialization.rs (post-S-DEMO-001) | ~10,000 |
| crates/prism-query/src/pushdown.rs | ~4,000 |
| crates/prism-spec-engine/src/pipeline.rs (post-S-DEMO-001) | ~10,000 |
| crates/prism-bin/src/spec_driven_adapter.rs (post-S-DEMO-001) | ~4,000 |
| crates/prism-dtu-crowdstrike/src/state.rs + routes/detections.rs + routes/mod.rs | ~2,500 |
| crates/prism-dtu-armis/src/routes/search.rs + state.rs | ~3,000 |
| crates/prism-spec-engine/tests/parity/armis.rs | ~2,500 |
| Production sensor TOMLs (4 files) | ~6,000 |
| DTU route structs (4 files: detections.rs + search.rs + alerts.rs + Claroty) | ~4,000 |
| Test outputs (cargo nextest) | ~2,000 |
| **Total estimate** | **~77,000 tokens (~30% of 256K context)** |

At the 20-30% budget ceiling. Implementer SHOULD split into two sub-tasks if context pressure appears:
- **Sub-task A (prism-query + sensors):** ADR-033 T1 wiring in materialization.rs + pushdown.rs; armis.sensor.toml INDEX options (AC-WIRE-001, AC-WIRE-001b, AC-INDEX-001).
- **Sub-task B (prism-spec-engine + prism-bin + prism-dtu-armis + prism-dtu-crowdstrike):** Correctness fixes; Armis AQL augmentation; Armis + CrowdStrike DTU time-clause filtering; integration + E2E tests (AC-CWS-001/002/003, AC-CWS-WIRE-001, AC-CWS-DTU-001, AC-ARMIS-001/002, AC-ARMIS-TW-001..005, AC-CYB-001, AC-CLAR-001, AC-EQUIV-001).

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 2.8 | 2026-06-06 | story-writer | F-P16-LOW-001 fix: TD-VSDD-091 volatile line-pin sweep — replaced all 5 non-excepted `lines ~NNN`/`materialization.rs:NNN` citations with function-name + behavioral anchors (`QueryParams` construction inside `run_materialization_pipeline`'s per-target fan-out loop; `extract_time_window_from_ast` call). Sites fixed: points justification comment (line 83), File Structure Requirements materialization.rs row, Tasks step 1, Tasks step 10, Previous Story Intelligence F-P6-CRIT-001 narrative. Changelog row (line 769) is TD-VSDD-091-EXCEPT; unchanged. No AC change; no RGT change. H1 + body header version 2.7→2.8. |
| 2.7 | 2026-06-06 | story-writer | F-P09-MED-001 fix-burst: added AC-CWS-WIRE-001 — wire-level combined verification that CrowdStrike FQL time-window AND limit reach the DTU simultaneously. The test `test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu` (18 code/test sites in `crates/prism-spec-engine/tests/bc_2_11_007_pushdown_test.rs`) cited AC-CWS-WIRE-001 but no story AC heading existed, creating the last dangling-AC traceability gap for this story. Placed after AC-CWS-003 (per-dimension CrowdStrike ACs); before AC-CWS-DTU-001 (DTU-internal FQL honoring). Tasks step 8: added `test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu` with note (EXISTING test, already passes); count 19→20. Step 16: 19→20 Red Gate tests. acceptance_criteria_count 17→18; red_gate_tests 19→20. Token Budget: story spec ~7,900→~8,200; total ~76,700→~77,000. Sub-task B: added AC-CWS-WIRE-001 to test enumeration. H1 + body header version 2.6→2.7. |
| 2.6 | 2026-06-05 | story-writer | F-P08-MED-001 fix-burst: added AC-INDEX-CWS-001 (crowdstrike.sensor.toml `created_timestamp` declares `options = ["INDEX"]`). This is the CrowdStrike parallel to AC-INDEX-001 (Armis): without `options = ["INDEX"]` on `created_timestamp`, ADR-033 Option T1 extraction silently skips the column and CrowdStrike FQL time-window push-down is silently vacuous. Red Gate test: EXISTING `test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option` (`crates/prism-spec-engine/tests/bc_2_11_007_pushdown_test.rs`) — 11 code/test sites cite this AC ID; the test already passes. Architecture Compliance Rules: added CrowdStrike INDEX row (parallel to Armis INDEX row). Tasks step 8: added `test_ac_index_cws_001_*` to Red Gate test list with full list of 19 RGTs. Step 16: 16→19 Red Gate test count. acceptance_criteria_count 16→17; red_gate_tests 18→19. H1 + body header version 2.5→2.6. |
| 2.5 | 2026-06-05 | story-writer | ADV-P08-MED-001 fix-burst: added EC-009 documenting inclusive-boundary push-down behavior (CompareOp::Ge/Le). DTU time-window filtering is inclusive at the boundary (records with `ts == bound` are kept, never excluded); push-down result is a superset of the exact predicate result; DataFusion post-filter narrows to exact set; BC-2.11.007 result-equivalence invariant holds. Second root cause: timestamp normalization changed from `to_rfc3339()` (`+00:00` suffix, ASCII 43) to `to_rfc3339_opts(SecondsFormat::Secs, true)` (`Z` suffix, ASCII 90) — `+00:00` < `Z` lexicographically causing exact-boundary records to be silently dropped at DataFusion string-comparison. Red Gate tests added: `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline` + `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline` (both in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`; drive `run_materialization_pipeline`; assert boundary record present in `>=` result). red_gate_tests 16→18. BC array unchanged (BC-2.11.007 v1.8 already scopes Ge/Le). acceptance_criteria_count unchanged (16). H1 + body header version 2.4→2.5. |
| 2.4 | 2026-06-05 | story-writer | LOCAL pass-5 fix-burst test-citation drift correction. AC-EQUIV-001 Red Gate test renamed/relocated by pass-5 fix-burst: OLD misnamed prism-spec-engine test `test_ac_equiv_001_result_equivalence_via_real_materialization_path` → RENAMED to `test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary` (PipelineExecutor-boundary only; does NOT satisfy AC-EQUIV-001 alone). NEW authoritative test `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline` in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` drives `run_materialization_pipeline` end-to-end and asserts BC-2.11.007 subset/no-fabrication invariant. Supplementary boundary test noted for context. H1 + body header version 2.3→2.4. |
| 2.3 | 2026-06-05 | story-writer | ADV-P04-HIGH-001 fix: identifier-accuracy correction — `parse_created_timestamp_bounds` → `parse_fql_time_bounds` at all 6 spec sites (frontmatter comment line 40, AC-CWS-DTU-001 When clause, Architecture Compliance Rules CrowdStrike row, Library & Framework Requirements prism-dtu-crowdstrike row, File Structure Requirements state.rs + detections.rs rows). Per Source-of-Truth Rule 7 code wins for code-vs-spec: actual implemented function is `parse_fql_time_bounds` (authored by implementer in this fix-burst per state.rs + routes/detections.rs); spec had the wrong name. ADV-P04-LOW-001 fix: updated AC-CWS-DTU-001 Red Gate test annotation from "existing test from OBS-001 fix-burst" to "new test authored by implementer in this fix-burst" — `parse_fql_time_bounds` is new code; the test is new. Test name `test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window` retained (matches implementer naming convention). H1 + body header version 2.2→2.3. |
| 2.2 | 2026-06-05 | story-writer | DRIFT-P1-001 (ADV-P02-HIGH-001) fix: reconcile spec to match OBS-001 fix-burst implementation. crates_touched: adds `prism-dtu-crowdstrike` → [prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-dtu-crowdstrike, prism-sensors]. inputs[]: adds prism-dtu-crowdstrike/src/state.rs + routes/detections.rs + routes/mod.rs. Library & Framework Requirements: adds prism-dtu-crowdstrike row. File Structure Requirements: adds 3 prism-dtu-crowdstrike MODIFY rows (state.rs parse_created_timestamp_bounds; detections.rs fixture filtering; mod.rs /dtu/filter-log route). Added AC-CWS-DTU-001: CrowdStrike DTU honors filter= FQL time-window — filtered_count < unfiltered_count (LOAD-BEARING; Red Gate test: test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window; parallel to AC-ARMIS-TW-002). Architecture Compliance Rules: added CrowdStrike DTU FQL-honoring row. Subsystem comment: notes prism-dtu-crowdstrike falls under SS-16 scope. acceptance_criteria_count 15→16; red_gate_tests 15→16. Token Budget: story spec ~7,000→~7,500; added prism-dtu-crowdstrike ~2,500; total ~73,500→~76,500 (~30% of 256K). Tasks: step 6b added (read CrowdStrike DTU source); 6c/6d renumbered to 6d/6e; step 9 adds prism-dtu-crowdstrike; step 16 updated (15→16 Red Gate tests, adds prism-dtu-crowdstrike iter); sub-task B updated to include prism-dtu-crowdstrike + AC-CWS-DTU-001. H1 + body header version 2.1→2.2. BC array unchanged (BC-2.11.007 v1.8 already covers CrowdStrike FQL as Mechanism A; no BC edit required). |
| 2.1 | 2026-06-05 | story-writer | Armis AQL full-wiring scope addition per human directive + pushdown-redesign.md §8 + BC-2.01.013 v1.14 + BC-2.11.007 v1.8. BC version pins: BC-2.01.013 v1.13→v1.14; BC-2.11.007 v1.7→v1.8 (both frontmatter comments + body BC table + all AC traces). crates_touched: adds prism-dtu-armis + prism-sensors → [prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-sensors]. AC-ARMIS-001: fixed disproven `lastSeen:>"2026-01-01"` → `after:2026-01-01T00:00:00` (bare, unquoted, timezone-naive per research-doc §2.2). Added AC-ARMIS-TW-001 (AQL augmentation at FilterMap boundary, unit test), AC-ARMIS-TW-002 (DTU fixture filtering LOAD-BEARING: filtered<unfiltered), AC-ARMIS-TW-003 (anti-double-filter guard), AC-ARMIS-TW-004 (Armis result-equivalence), AC-ARMIS-TW-005 (E2E #[ignore] aql-log confirmation). Added AC-INDEX-001 (armis.sensor.toml last_seen+created_at must have options=["INDEX"]). acceptance_criteria_count 9→15; red_gate_tests 9→15. Architecture Compliance Rules: added Armis AQL syntax rule (MUST use after:/before: bare unquoted), anti-double-filter rule, DTU-honors rule, INDEX-option rule, R-DTU-002 unaffected rule. File Structure Requirements: added prism-dtu-armis/routes/search.rs MODIFY + armis.sensor.toml MODIFY + parity/armis.rs MODIFY + e2e_smoke.rs MODIFY. Library Requirements: added prism-dtu-armis + prism-sensors entries. Out-of-scope: Armis time-window is IN scope in v2.1 (was post-filter-only in §1.2 design note; §8 supersedes). Remaining deferrals unchanged (Cyberint page_size, Claroty body pagination, Claroty time-window, Option T2). Token Budget updated to ~73,500 tokens (~29% of 256K); sub-task split guidance added. Previous Story Intelligence: added §8 directive note, research-confirmed syntax note, S-DEMO-ARMIS-AQL-001 note. inputs[] expanded: adds prism-dtu-armis/routes/state.rs + parity/armis.rs + research doc. Tasks expanded: added steps 6b/6c/6d (read DTU, parity tests, research) + 14b/14c/14d (implement augmentation, DTU extension, TOML options); all 15 Red Gate test names listed. |
| 2.0 | 2026-06-05 | story-writer | Major re-author (v1.3 → v2.0). Motivation: LOCAL adversary passes 5+6 established v1.x implementation is inert against production sensor shapes (materialization.rs hardcodes None; wrong Armis/Cyberint/Claroty translations). New scope: crates_touched adds prism-query (ADR-033 T1 time-window extraction in materialization.rs + pushdown.rs). subsystems adds SS-11 (Query Execution). target_module changed to prism-query. points 5→8 (T1 extraction + SAP-2 compliant test suite). AC set fully replaced: AC-CWS-001/002/003 (CrowdStrike limit + FQL time-window both start+end + empty-filter); AC-ARMIS-001/002 (AQL passthrough; assert NO maxResults/NO timeFrame); AC-CYB-001 (cursor-only GET; assert NO from_date/to_date/page_size); AC-CLAR-001 (empty body; assert NO time-window injection); AC-WIRE-001 (run_materialization_pipeline populates start_time+end_time per ADR-033 T1); AC-EQUIV-001 (result-equivalence via REAL materialization path — not direct FetchContext construction). SAP-2 Standing AC Gate added (production-TOML fixture mandate; fabricated-fixture P1 CRITICAL gate). Out-of-scope follow-ups anchored: Cyberint page_size → DTU-EXT-005; Claroty body pagination → S-DEMO-CLAROTY-PAGINATION-001; Claroty time-window → S-DEMO-CLAROTY-TIME-001 (new stub); full classify_predicates (Option T2) → future wave-6. BC table body updated with v1.13/v1.6/v1.7 version citations. Token Budget BC count updated to 3 BCs (unchanged). inputs[] expanded: adds prism-query src files + sensor TOMLs + DTU route structs + ADR-033 + pushdown-redesign.md. depends_on adds S-DEMO-002 (SATISFIED). v1.x implementation superseded. |
| 1.3 | 2026-06-05 | state-manager | F-PUSHDOWN2-MED-001: status sync — frontmatter `status: ready`→`in_progress`; body header `**Status:** ready`→`in_progress`; body H1 version label v1.2→v1.3. D-1002 burst introduced asymmetry between STORY-INDEX (badged `in_progress v1.2`) and this story file (still `ready`). Source-of-Truth Rule 5: active LOCAL cascade → `in_progress` is canonical. Version bumped to v1.3 to maintain POLICY 32 monotonic-descending changelog. |
| 1.2 | 2026-06-05 | story-writer | F-PUSHDOWN-006: removed VP-031 from verification_properties (VP-031 covers required-column rejection in prism-query / S-3.02 — unrelated to push-down threading; mis-anchor). No push-down VP exists in VP-INDEX (156 VPs checked); new-VP need flagged for PO/architect in frontmatter note. F-PUSHDOWN-007: updated BC-2.11.005 row in Behavioral Contracts table with PO-specified affected-but-indirectly-tested relationship note. Body header version/status updated to v1.2/ready. Token Budget BC count (3 BCs) remains consistent. |
| 1.1 | 2026-06-03 | state-manager | D-990 Phase-A-close: status draft→ready; depends_on S-DEMO-001 SATISFIED (merged PR #166); BC-2.01.013 v1.11 active + BC-2.11.005 active + BC-2.11.007 active — S-7.01 gate CLEARED. |
| 1.0 | 2026-05-31 | story-writer | Initial draft — created per S-DEMO-001 v1.5 AC-010 scope note and BC-2.01.013 v1.8 Pagination/Push-Down Scope Clause (D-924). Scope: thread FetchContext push-down fields (cursor/limit/start_time/end_time) from SpecDrivenSensorAdapter::fetch() into PipelineExecutor build_request(). P2 non-blocking — correctness holds via DataFusion post-materialization. |
