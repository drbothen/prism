---
document_type: adr
adr_id: ADR-033
status: accepted
date: 2026-06-05
introduced: 2026-06-05
version: "v1.1"
subsystems_affected:
  - SS-11
  - SS-16
  - SS-01
supersedes: null
superseded_by: null
anchor_stories:
  - S-DEMO-QUERY-PUSHDOWN-001  # §Authority not verified (story predates the ##Authority convention); retained on implementation-citation evidence
  # S-REQUIRED-COL-GATE-001 cites ADR-033 §Decision in its §Architecture Compliance Rules
  # but lacks a ##Authority section — per SAC-2, cannot be added until story-writer adds
  # ##Authority section to S-REQUIRED-COL-GATE-001.
traces_to:
  - BC-2.11.007
  - BC-2.01.013
---

# ADR-033: Push-Down Time-Window Extraction Strategy — Pre-Fan-Out Heuristic (T1) vs Post-Resolution classify_predicates (T2)

> **One-per-file:** Each architectural decision lives in its own file.
> Filename convention: `ADR-NNN-<short-name>.md`
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.
> Frontmatter `subsystems_affected`: SS-11 (Query Execution), SS-16 (Spec Engine), SS-01 (Sensor Adapters).

## Context

`prism-query`'s materialization pipeline (`run_materialization_pipeline` in
`materialization.rs`) is the sole production callsite that constructs `QueryParams`
for sensor fan-out. As of LOCAL adversary pass-6, the construction at lines ~434–440
hardcodes `start_time: None`, `end_time: None`, and `cursor: None` unconditionally.
All time-window and cursor translation logic in `apply_push_down_to_request()` and
`apply_push_down_to_json_body()` is dead code in production: `limit` is the only
push-down dimension that actually reaches the sensor adapter (F-P6-CRIT-001,
S-DEMO-QUERY-PUSHDOWN-001 v1 adversarial cascade).

The PrismQL AST does represent time predicates: a `Predicate::Compare` node where
`lhs = Expr::Field(FieldPath)` names a datetime column, `op ∈ {Gt, Ge, Lt, Le}`,
and `rhs = Expr::Literal(Literal::Timestamp(t))`. The existing
`extract_push_down_filters_as_map` / `predicate_tree_to_filter_map` functions walk
AND-conjunctions but collect only equality (`=`) predicates; range predicates fall
through silently. The function's doc-comment notes that per-sensor `classify_predicates`
integration is deferred (code TODO; see `extract_push_down_filters_as_map
§extract_push_down_filters_as_map` source). The wave reference in that comment is a code
TODO — propagating it here as a normative ADR commitment would convert a code TODO into an
untracked architectural obligation; see §Decision for the correct deferral mechanism.

A design choice arises: at the `extract_push_down_filters_as_map` callsite in
`materialization.rs`, the per-sensor `ColumnSpec` is not yet resolved (resolution
happens later in the fan-out step, per-target). Two architecturally distinct options
exist for extracting the time-window bounds:

**Option T1 — Pre-fan-out column-name heuristic.** Walk `Predicate::Compare` nodes
at the `run_materialization_pipeline` level, before per-sensor fan-out. Identify
datetime columns by looking up the column name in the sensor specs available via
`MaterializationContext.resolved_spec_map` (the `ConfigSnapshot.sensor_specs` is
accessible here). When a Compare predicate's lhs column is declared
`column_type = "datetime"` in the resolved spec, extract `t.instant.to_rfc3339()` as
`start_time` (Gt/Ge) or `end_time` (Lt/Le). Populate `QueryParams.start_time` /
`QueryParams.end_time` at the materialization callsite (currently hardcoded `None`).

**Option T2 — Restructured per-sensor classify_predicates integration.** Route
`classify_predicates` (which already takes `ColumnSpec` slices and classifies
predicates correctly) to run per-target after fan-out resolution. This requires
restructuring the fan-out orchestration so classification executes post-resolution.
Architecturally cleaner but larger scope: the fan-out call sequence must change to
pass per-sensor specs into the classification step.

This decision records the choice made for S-DEMO-QUERY-PUSHDOWN-001 v2; T2 is partially
anchored (`S-REQUIRED-COL-GATE-001` for the REQUIRED-column plan-time enforcement
portion) and partially an open obligation (full fan-out restructuring — no story yet).

Cross-reference: design note `.factory/cycles/wave-5-e-demo-fidelity/S-DEMO-QUERY-PUSHDOWN-001/pushdown-redesign.md` §§2, 5; BC-2.11.007 (result-equivalence invariant, AQL passthrough Mechanism B); BC-2.01.013 (per-sensor push-down translation table).

## Decision

We adopt **Option T1 (pre-fan-out heuristic)** for the v2 scope of
S-DEMO-QUERY-PUSHDOWN-001.

At the `run_materialization_pipeline` callsite in `prism-query/src/materialization.rs`,
a new function (or extension to `extract_push_down_filters_as_map`) walks
`Predicate::Compare` nodes in the query's predicate tree. When (a) the lhs `FieldPath`
names a column declared `column_type = "datetime"` in the sensor `ColumnSpec` obtained
from `MaterializationContext.resolved_spec_map`, (b) the operator is `Gt`, `Ge`,
`Lt`, or `Le`, and (c) the rhs is `Literal::Timestamp(t)`, the value
`t.instant.to_rfc3339()` is extracted as `start_time` (for Gt/Ge) or `end_time` (for
Lt/Le). These strings are populated into `QueryParams.start_time` and
`QueryParams.end_time` at lines ~437–438 (currently `None`).

Sensor adapters consume these values only where native time-window params exist in the
DTU. Currently that means CrowdStrike exclusively: `start_time` and `end_time` are
injected into the `filter` FQL query param on the `query_detection_ids` Step 1 as
`created_timestamp:>'<ISO8601>'` (start) and `created_timestamp:<'<ISO8601>'` (end),
combined with `+` when both are present. Adapters without native time params (Armis,
Cyberint, Claroty) silently receive non-None values and ignore them; DataFusion
post-filters to uphold BC-2.11.007 result-equivalence.

**Full per-sensor `classify_predicates` integration (Option T2)** is split:

**REQUIRED-column plan-time enforcement (uses T1 access pattern, not T2):** The plan-time
E-QUERY-009 gate that rejects queries missing a REQUIRED column constraint is anchored to
`S-REQUIRED-COL-GATE-001` (status: draft, depends_on: S-3.02). Ground truth in this ADR's
§Decision confirms that `MaterializationContext.resolved_spec_map` is already accessible
pre-fan-out — T1 uses it for datetime column type lookup; `S-REQUIRED-COL-GATE-001` reuses
the same access pattern. No fan-out orchestration restructuring is needed for that gate.
ADR-057 §D7's earlier characterization of T2 as a prerequisite for this gate was a false
premise, corrected by `S-REQUIRED-COL-GATE-001 §Origin`.

**Full T2 (post-resolution per-sensor `classify_predicates` for all push-down dimensions)
— OPEN OBLIGATION:** Restructuring fan-out orchestration to pass per-sensor `ColumnSpec`
into `classify_predicates` post-resolution (for range predicates, inequality push-down,
etc.) is not covered by `S-REQUIRED-COL-GATE-001`. This broader T2 work remains an open
obligation with no story yet authored. When that story is created, a follow-up ADR will
supersede this one for the full T2 scope. No wave number is assigned here — code comments
describing this work as deferred are code TODOs; they are not story anchors and their wave
references are not normative ADR obligations.

## Rationale

Option T1 is the minimal correct design that closes the dead-code gap without
restructuring fan-out orchestration:

1. **BC-2.11.007 result-equivalence is preserved.** Adapters that cannot consume
   `start_time`/`end_time` natively ignore them; DataFusion filters apply post-fetch.
   The invariant holds regardless of whether push-down occurs.

2. **The scope is localized and verifiable.** T1 adds ~100–150 lines to `prism-query`
   (`materialization.rs` + `pushdown.rs`) plus correctness fixes in `prism-spec-engine`
   and `prism-bin`. No fan-out orchestration restructuring is required.

3. **SAP-2 compliance is achievable.** DTU clone `DetectionListParams` field shapes
   are fully known; the ADR-031 fidelity principle ensures DTU routes mirror real API
   shapes. End-to-end CrowdStrike FQL injection tests are grounded in production spec
   fixtures (not fabricated constructors).

4. **Option T2 is not regressed.** T1 populates `QueryParams.start_time`/`end_time`
   as `Option<String>`; the existing `QueryParams` type is unchanged. T2 will consume
   the same fields when it lands — no rework of the extraction output interface is
   required.

5. **Per-sensor translation correctness.** Section 1 of the design note establishes
   that Armis has no native time-window param (`SearchQueryParams` lacks `timeFrame`
   or equivalent), Cyberint's DTU accepts only `cursor`, and Claroty's DTU uses URL
   OffsetLimit with no time fields. Injecting wrong params (the current wrong code)
   is a correctness defect, not a push-down gap. T1 produces the correct no-op for
   these sensors.

Option T2 was not adopted for v2 because restructuring the fan-out call sequence
(to pass per-sensor `ColumnSpec` into `classify_predicates` pre-fan-out) is a larger
architectural change that requires a dedicated spec + story and is not required to
close the CrowdStrike time-window push-down gap. Premature adoption of T2 would
expand the scope beyond what BC-2.01.013 v2 requires.

## Consequences

### Positive

- `start_time`/`end_time` fields in `QueryParams` become reachable end-to-end for
  sensors with native time-window params in their DTU (CrowdStrike detections and
  devices tables, once verified).
- The dead-code gap (F-P6-CRIT-001) is closed: `apply_push_down_to_request()` and
  the FQL injection path become live code exercised by actual queries.
- Wrong Armis (`maxResults`/`timeFrame`) and Cyberint (`from_date`/`to_date` POST
  body) translations are removed, eliminating a silent correctness defect that could
  inject unexpected params into production sensor requests.
- BC-2.11.007 result-equivalence invariant is end-to-end exercisable via DTU clone
  integration tests (real materialization path, not direct `FetchContext` construction).

### Negative / Trade-offs

- The T1 heuristic relies on column names being consistent across sensor specs that
  share the same logical field. If two sensors name their primary timestamp column
  differently (e.g., `created_timestamp` vs `detected_time`), each column must be
  individually declared `column_type = "datetime"` in its TOML spec. The heuristic
  is not global; it is per-spec-lookup. This is a documentation burden, not a
  correctness risk, since `column_type` is already a required field in the TOML schema.
- Option T2 (full per-sensor `classify_predicates` integration) remains unimplemented.
  Queries whose push-down behavior requires per-target classification (e.g., predicates
  whose column type differs between sensor specs) will fall back to post-filter until
  T2 lands.
- `MaterializationContext.resolved_spec_map` must be populated before the time-window
  extraction call; if it is `None` (spec not resolved), extraction silently produces
  `None` values and no push-down occurs. This is the correct safe default.

### Status as of 2026-06-05

ACCEPTED. Human explicitly approved v2 scope expansion (D-1006 2026-06-05 exception
decision: "Approve — re-implement v2"). ADR-033 promoted proposed→accepted. v2
re-implementation of S-DEMO-QUERY-PUSHDOWN-001 authorized to begin; worktree reset
to develop@752e407a; test-writer Red Gate dispatch is next.

## Alternatives Considered

- **Option T2 — Restructured per-sensor classify_predicates integration:** Deferred
  rather than rejected. Would deliver correct per-sensor predicate classification but
  requires restructuring fan-out orchestration to pass `ColumnSpec` post-resolution.
  A future ADR will cover T2 when the fan-out restructuring story is authored (named
  follow-up in design note §3 deferred-scope table).

- **No change (keep hardcoded None):** Rejected. `start_time`/`end_time` remain dead
  code; CrowdStrike FQL time-window push-down stays unreachable; F-P6-CRIT-001 is
  not closed. Production-grade default forbids this.

- **Global column-name pattern matching (no spec lookup):** Rejected. Matching any
  column named `*_timestamp`, `*_time`, `*_date` without consulting the sensor spec
  risks misclassifying non-datetime columns with similar names, injecting incorrect
  `start_time`/`end_time` into adapters. Spec-grounded lookup is the correct approach.

## Source / Origin

- Design note §§2–5: `.factory/cycles/wave-5-e-demo-fidelity/S-DEMO-QUERY-PUSHDOWN-001/pushdown-redesign.md`
- LOCAL adversary pass-6 finding F-P6-CRIT-001: `materialization.rs` lines ~434–440
  hardcode `start_time: None, end_time: None, cursor: None`
- `prism-query/src/materialization.rs` — `run_materialization_pipeline` + `extract_push_down_filters_as_map`
- `prism-query/src/pushdown.rs` — `classify_predicates`, `predicate_tree_to_filter_map`
- BC-2.11.007 result-equivalence invariant + Mechanism B (AQL passthrough)
- BC-2.01.013 per-sensor push-down translation table (v2 amendment pending PO)
- `prism-dtu-crowdstrike/src/routes/detections.rs` — `DetectionListParams` struct (`filter`, `limit`, `offset` fields)
- `prism-sensors/specs/crowdstrike.sensor.toml` — Step 1 `query_detection_ids` shape

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.1 | 2026-07-31 | architect | FB109 — wave-granularity deferral correction. Two LIVE normative sites fixed. §Context: removed wave-5 propagation from code-doc-comment quote — "deferred to wave-5" is a code TODO; quoting the wave number into an ADR converts a code TODO into an untracked normative obligation. Replaced with: doc-comment notes per-sensor classify_predicates is deferred (code TODO); wave reference in that comment is not imported here. §Decision: "a future wave-6 story" removed; replaced with split deferral: (1) REQUIRED-column plan-time E-QUERY-009 gate anchored to `S-REQUIRED-COL-GATE-001` (status: draft, depends_on: S-3.02) — that story confirmed `MaterializationContext.resolved_spec_map` is accessible pre-fan-out via T1's access pattern; no T2 fan-out restructuring needed; ADR-057 §D7 false-prerequisite claim corrected bidirectionally; (2) full T2 (post-resolution per-sensor classify_predicates for all push-down dimensions) remains OPEN OBLIGATION, no story yet. version: missing → "v1.1" added to frontmatter. anchor_stories: `S-DEMO-QUERY-PUSHDOWN-001` retained; SAC-2 annotation added. POL-29 9a: decisions-directory sweep for wave-granularity patterns — file-path references to cycle dirs and historical changelog rows excluded; only ADR-033 §Context (code-comment quote) and §Decision ("a future wave-6 story") were LIVE normative sites, both fixed here. Code docs in pushdown.rs and materialization.rs still carry "deferred to wave-5" — reported, not edited (implementer routing). 9b: §Decision is not a verbatim copy-source for any downstream artifact. 9c: `S-REQUIRED-COL-GATE-001` verified on disk; no new unanchored MUSTs introduced. |
| v1.0 | 2026-06-05 | state-manager | Promoted proposed→accepted per D-1006 human approval: "Approve — re-implement v2." v2 scope expansion to prism-query (+SS-11) authorized. Re-implementation TDD cycle begins; worktree reset to develop@752e407a. ARCH-INDEX v2.111→v2.112. |
| v0.1 | 2026-06-05 | architect | Initial proposed ADR for ADR-033 via create-adr workflow. T1 heuristic decision for S-DEMO-QUERY-PUSHDOWN-001 v2. |
