---
document_type: design-note
story: S-DEMO-QUERY-PUSHDOWN-001
version: "1.0"
date: 2026-06-05
author: architect
status: draft
purpose: >
  Ground-truth re-spec input for product-owner (BC amendments) and story-writer
  (story v2 authoring). Does NOT modify BCs, stories, or code.
authority: CLAUDE.md §Source-of-Truth Precedence Rule 1 + Canonical Principle Rule 2
---

# Push-Down Redesign Note — S-DEMO-QUERY-PUSHDOWN-001

## Background

LOCAL adversary passes 5 and 6 established that the current implementation is
largely inert against production sensor shapes. The root causes are two independent
defects that compound:

1. **materialization.rs hardcodes None.** `prism-query/src/materialization.rs` lines
   ~434–440 (the sole production callsite building `QueryParams`) always sets
   `start_time: None`, `end_time: None`, `cursor: None`. All time-window and cursor
   translation code in `apply_push_down_to_request()` and `apply_push_down_to_json_body()`
   is unreachable dead code in production. `limit` is populated; that is the only
   push-down dimension that actually reaches the sensor adapter.

2. **Per-sensor translation targets are wrong.** Even if the materialization callsite
   were wired, the existing translation for Armis injects `maxResults` and `timeFrame`
   which do not exist in `SearchQueryParams`; the Cyberint translation injects into a
   POST body that does not exist (real spec is GET with cursor, no body_template); and
   the Claroty translation injects `limit` into a body_template of `'{}'` (no-op).

This note defines the correct design per production reality and recommends a scope
slice that delivers verified correct push-down without deferring entire dimensions
silently.

---

## Section 1 — Per-Sensor Correct Push-Down Mechanism Table

This table is the authoritative ground truth for re-spec. Every entry is grounded
from the production `*.sensor.toml` file and the corresponding DTU route struct.
"None — no native push-down" is a correct documented answer; invented params are not.

### 1.1 CrowdStrike

**Step architecture:** Two-step pipeline. Step 1 (`query_detection_ids`) is the
query-plan step. Step 2 (`fetch_detections`) is a POST-body fan-out batch step;
it accepts only `{"ids": [...]}` and does not take filter params.

Push-down applies to Step 1 ONLY.

| Dimension | DTU struct field | Location | Param name | Correct value | TOML source |
|-----------|-----------------|----------|------------|---------------|-------------|
| Limit | `DetectionListParams.limit` | Query param | `limit` | `u64` | `detections.rs:29` — `pub limit: Option<usize>` |
| Filter (FQL) | `DetectionListParams.filter` | Query param | `filter` | FQL string, e.g. `created_timestamp:>'2026-01-01T00:00:00Z'` | `detections.rs:26` — `pub filter: Option<String>` |
| Offset | `DetectionListParams.offset` | Query param | `offset` | `usize` | `detections.rs:28` — `pub offset: Option<usize>` |
| Time-window (start) | `DetectionListParams.filter` (FQL content) | Query param | `filter` | FQL: `created_timestamp:>'<ISO8601>'` | No dedicated time param; injected into FQL string |
| Time-window (end) | `DetectionListParams.filter` (FQL content) | Query param | `filter` | FQL: `created_timestamp:<'<ISO8601>'` (combined with start if both) | Same |
| Cursor | None | — | — | **None. DTU uses offset pagination, not cursor.** The `DetectionListParams` struct has `limit`+`offset`, not `cursor`. | `crowdstrike.sensor.toml` step 1 has no `[tables.steps.pagination]` block (no cursor declared). |

**Summary for CrowdStrike:** Push-down dimensions that are real and reach the DTU:
`limit` (query param), `filter` (FQL string for time-window if applicable). Cursor: none.
The devices table follows the same two-step pattern with the same params on
`GET /devices/queries/devices/v1`.

### 1.2 Armis

**Step architecture:** Single-step. `GET /api/v1/search?aql=${query.filter.aql}`.
Pagination is OffsetLimit via `offset`/`limit` query params (DTU `SearchQueryParams`
accepts `offset`/`limit` with prism OffsetLimit convention).

| Dimension | DTU struct field | Location | Param name | Correct value | TOML source |
|-----------|-----------------|----------|------------|---------------|-------------|
| AQL filter | `SearchQueryParams.aql` | Query param | `aql` | Verbatim AQL string (Mechanism B per BC-2.11.007) | `armis.sensor.toml` devices+alerts `path_template = "/api/v1/search?aql=${query.filter.aql}"` |
| Limit | `SearchQueryParams.limit` | Query param | `limit` | `u32` (OffsetLimit convention) | `search.rs:79` — `pub limit: Option<u32>` |
| Offset | `SearchQueryParams.offset` | Query param | `offset` | `u32` | `search.rs:76` — `pub offset: Option<u32>` |
| Page (Armis native) | `SearchQueryParams.page` | Query param | `page` | `u32` (1-based) | `search.rs:72` — `pub page: Option<u32>` |
| Size (Armis native) | `SearchQueryParams.size` | Query param | `size` | `u32` | `search.rs:74` — `pub size: Option<u32>` |
| Time-window | **None** | — | — | **None — no native time-window param.** `SearchQueryParams` has no `timeFrame`, `from_date`, `to_date`, or similar field. Time range MUST be embedded inside the AQL string if the user wants it: `WHERE aql = 'in:devices lastSeen:>"2026-01-01"'`. The current `maxResults`/`timeFrame` injection is wrong. | `search.rs:68–81` (full struct listed) |
| Cursor | **None** | — | — | **None — no cursor.** Armis /api/v1/search is OffsetLimit, not cursor-token. | `armis.sensor.toml` `[tables.steps.pagination] type = "offset_limit"` |

**Summary for Armis:** The ONLY push-down mechanism is AQL passthrough per BC-2.11.007
Mechanism B. The `aql` string is forwarded verbatim as a query param. No separate
time-window param exists. If a user wants time-filtered Armis data they embed the
time clause in the AQL string. `limit`/`offset` are pagination params handled by the
existing OffsetLimit pipeline; they are not a new push-down dimension but a correct
existing behavior.

### 1.3 Cyberint

**Step architecture:** Single-step. `GET /api/v1/alerts` with cursor pagination.
DTU `AlertListParams` accepts ONE field: `cursor: Option<String>`. There is no POST
body; the step has no `body_template`.

| Dimension | DTU struct field | Location | Param name | Correct value | TOML source |
|-----------|-----------------|----------|------------|---------------|-------------|
| Cursor | `AlertListParams.cursor` | Query param | `cursor` | String cursor value | `alerts.rs:39` — `pub cursor: Option<String>` |
| Limit / page_size | **None** | — | — | **None — DTU-EXT-005 is OPEN.** `AlertListParams` has no `page_size` or `limit` field. The DTU does not accept page_size. This is documented in `cyberint.sensor.toml` F-LP2-MEDIUM-001 and DTU-EXT-005. | `cyberint.sensor.toml` line ~111: `# F-LP2-MEDIUM-001: page_size removed from this block` |
| Time-window | **None** | — | — | **None — no time-window params in DTU.** `AlertListParams` is cursor-only. The existing `from_date`/`to_date` POST-body injection is completely wrong: (a) the step is GET not POST, (b) there is no body_template, (c) the struct has no such fields. | `alerts.rs:37–40` |
| Limit | **None (production)** | — | — | See page_size above — same result. | — |

**Summary for Cyberint:** The ONLY real push-down dimension is cursor passthrough
for pagination. The cursor value ("page2" in the DTU, real cursor string in production)
is passed as `?cursor=<value>` on subsequent pages. No time-window push-down is
possible against the current DTU. Any time-filtering is post-materialization only.
Until DTU-EXT-005 lands, no `page_size` push-down either.

### 1.4 Claroty

**Step architecture:** Single-step POST for all three tables (alerts, audit_logs,
devices). Body is always `'{}'` (empty object). Pagination is OffsetLimit via URL
params (`?offset=N&limit=M`) appended by `build_paged_url_impl`. Gap-CL-004 tracks
that the real Claroty API expects offset/limit in the POST body — that fix is
deferred to the open story `S-DEMO-CLAROTY-PAGINATION-001`.

The DTU `list_alerts`, `list_devices`, `list_audit_logs` handlers accept:
- `Option<Json<GetAlertsBody>>` / `Option<Json<GetDevicesBody>>` / `Option<Json<GetAuditLogBody>>`
- Pagination via `page`/`page_size` OR `offset`/`limit` in the body (devices.rs
  line ~292). For alerts and audit_log the body is currently ignored beyond auth
  (no filter fields parsed out of `GetAlertsBody`).

| Dimension | DTU struct | Location | Param name | Correct value | TOML source |
|-----------|-----------|----------|------------|---------------|-------------|
| Limit (URL) | URL params | Query param | `limit` | `u32` via OffsetLimit pipeline | `devices.rs:303` — `let limit = params.limit.unwrap_or(u32::MAX) as usize` |
| Offset (URL) | URL params | Query param | `offset` | `u32` via OffsetLimit pipeline | `devices.rs:302` — `params.offset` |
| Page/page_size (body) | `GetDevicesBody.page`, `GetDevicesBody.page_size` | POST body | `page`, `page_size` | `u32` | `devices.rs:292` — devices only; alerts body has no pagination fields |
| Time-window | **None** | — | — | **None.** No time-window param exists in any Claroty DTU route struct. Claroty's real API may support `detected_after`/`detected_before` but those are NOT in the DTU. | All Claroty route handlers |
| Limit (POST body) | Deferred | — | — | **Deferred to S-DEMO-CLAROTY-PAGINATION-001.** Gap-CL-004: real Claroty API expects offset/limit in body; pipeline currently sends URL params. This story is OPEN and must close first before Claroty body-pagination push-down is considered here. | `claroty.sensor.toml` lines 15-18 |

**Summary for Claroty:** No push-down dimensions are available for S-DEMO-QUERY-PUSHDOWN-001
v2 scope. Pagination is handled by the existing OffsetLimit pipeline (URL params to DTU).
True push-down (body-based offset/limit to the real API) is deferred to
`S-DEMO-CLAROTY-PAGINATION-001`. Time-window is non-existent in the current DTU.

---

## Section 2 — Time-Window + Cursor Wiring Design

### 2.1 Where time predicates live in the query plan

The PrismQL AST represents time predicates as `Predicate::Compare` nodes where:
- `lhs` is `Expr::Field(FieldPath)` naming a datetime column (e.g.,
  `created_timestamp`, `detected_time`, `last_seen`)
- `op` is `CompareOp::Gt`, `CompareOp::Ge`, `CompareOp::Lt`, or `CompareOp::Le`
- `rhs` is `Expr::Literal(Literal::Timestamp(TimestampLiteral))` — the parsed
  `DateTime<Utc>` is in `TimestampLiteral.instant`

The WHERE clause is already extracted as a flat `FilterMap` (equality only) in
`materialization.rs::extract_push_down_filters_as_map`. That function calls
`pushdown::predicate_tree_to_filter_map`, which walks AND-conjunctions and collects
`field = 'value'` pairs. Inequality/range predicates (`>`, `>=`, `<`, `<=`) are NOT
collected by this function — they fall through silently.

To extract time-window bounds, a new function (or an extension to the existing one)
must walk the predicate tree looking for:

```
Predicate::Compare { lhs: Field(fp), op: Gt|Ge, rhs: Literal(Timestamp(t)) }
  => start_time candidate: fp is a known datetime column
Predicate::Compare { lhs: Field(fp), op: Lt|Le, rhs: Literal(Timestamp(t)) }
  => end_time candidate: fp is a known datetime column
```

The challenge is that at the `extract_push_down_filters_as_map` call site in
`materialization.rs`, no per-sensor `ColumnSpec` is available (the function is
called before per-sensor fan-out resolution). Two design options exist:

**Option T1 — Column-name heuristic (simple, scope-appropriate):**
At the pre-fan-out stage, identify time-window predicates by checking whether the
column name is a known datetime column in the loaded sensor specs. The
`ConfigSnapshot.sensor_specs` is available at the `MaterializationContext` level.
For a given `source_names` set, look up the matching specs and identify which
columns are `column_type = "datetime"` + `options = ["INDEX"]`. If a Compare
predicate references one of these, extract it as `start_time` or `end_time`.

This requires the `ConfigSnapshot` to be threaded into `extract_push_down_filters_as_map`
(or a new sibling function). The `MaterializationContext` already holds
`resolved_spec_map: Option<Arc<...>>` which gives access to the sensor specs.

**Option T2 — Defer to per-sensor classify_predicates (deferred, larger scope):**
The existing `classify_predicates` function in `pushdown.rs` takes `ColumnSpec` slices
and classifies predicates against them. Wiring this per-sensor at the pre-fan-out
stage requires restructuring the fan-out orchestration so classify_predicates runs
per-target (after resolution) rather than pre-resolution. This is the ADR-022 §C
"wave-5 future work" mentioned in `predicate_tree_to_filter_map`'s doc comment.

**Recommendation for this story's scope:** Option T1 is appropriate. Extract
time-window bounds from the AST by inspecting `Predicate::Compare` nodes against
datetime columns from the resolved sensor spec. The resulting `start_time` and
`end_time` are passed as `Option<String>` (ISO8601 formatted) into `QueryParams`.
The `SpecDrivenSensorAdapter` in `prism-bin` then uses them only for CrowdStrike
(FQL injection) — Armis/Cyberint/Claroty do not have usable native time params
as established in Section 1.

### 2.2 Cursor extraction

Cursor is simpler. The existing `FetchContext` in `prism-spec-engine` handles
cursor progression internally during pagination — `PipelineExecutor` manages the
cursor from step to step. The `QueryParams.cursor` at the fan-out level is an
INITIAL cursor seed (for resuming a paginated session), not the intra-step cursor.

For the sensor types in scope:
- CrowdStrike: no cursor; offset-based. `cursor` field in `QueryParams` is irrelevant
  for Step 1 (`DetectionListParams` has `offset`, not `cursor`).
- Armis: no cursor; OffsetLimit. Cursor irrelevant.
- Cyberint: cursor-token. The INITIAL cursor is typically null/absent; subsequent pages
  are driven by the cursor the API returns. The `QueryParams.cursor` field would seed
  the first page cursor if resuming. This is out of scope for v2 (no user-facing
  cursor-resume API in PrismQL yet).
- Claroty: OffsetLimit. No cursor.

**Verdict for cursor wiring:** Cursor seeding from `QueryParams` is a future concern.
No PrismQL syntax exists today to express an initial cursor value in a WHERE clause.
Remove the `cursor: None` hardcode defensively by ensuring it remains `None` (no
change needed in materialization.rs for cursor); document why in a code comment.

### 2.3 Scope verdict: in-story vs separate story

**Time-window wiring into prism-query (materialization.rs) is in-story scope for v2.**
The fix is localized: extend `extract_push_down_filters_as_map` (or add a sibling
`extract_time_window_from_ast`) to extract GT/GE/LT/LE predicates on datetime
columns, then populate `start_time`/`end_time` in the `QueryParams` at lines ~437–438.
This is ~100–150 lines of new Rust in `prism-query`, a single function addition, and
requires no new crate dependencies.

**Crates touched for v2:**
- `prism-query` (materialization.rs — wire `start_time`/`end_time`; pushdown.rs —
  add time-window extraction function)
- `prism-spec-engine` (per-sensor translation: only CrowdStrike FQL injection remains;
  Cyberint/Armis/Claroty correctly produce no-op for time-window)
- `prism-bin` (spec_driven_adapter.rs — correct Armis to use AQL passthrough, no
  `maxResults`/`timeFrame`; CrowdStrike FQL time-window injection if it lives here)

The scope is NOT large. It is one new function in `prism-query` plus corrections in
`prism-spec-engine`/`prism-bin` to remove the wrong Armis/Cyberint/Claroty translations.

---

## Section 3 — Recommended Scope Slice for Re-Spec

The human values "all sensors / no scope compromises." This means: deliver CORRECT
push-down for every dimension that is reachable now, and anchor every deferral to a
named follow-up story with a concrete reason.

### What S-DEMO-QUERY-PUSHDOWN-001 v2 MUST deliver (correct and load-bearing)

1. **materialization.rs wiring (prism-query):** Extract `start_time` and `end_time`
   from the PrismQL AST predicate tree (Compare nodes on datetime INDEX columns).
   Populate `QueryParams.start_time` and `QueryParams.end_time` at the fan-out callsite.
   This closes the fundamental dead-code gap (F-P6-CRIT-001).

2. **CrowdStrike FQL time-window injection (prism-spec-engine or prism-bin):**
   When `start_time` and/or `end_time` are set, inject into the `filter` FQL param
   on the CrowdStrike `query_detection_ids` step (Step 1 only). Format:
   `created_timestamp:>'<ISO8601>'` for start, `created_timestamp:<'<ISO8601>'` for
   end, combined with `+` if both.

3. **CrowdStrike limit injection:** Already partially wired (`limit` is populated from
   `options.limit`). Verify it reaches `DetectionListParams.limit` in the DTU; this
   was the one verified real-effect push-down from pass 4. Test must use real production
   spec shape (not `make_crowdstrike_like_spec`).

4. **Armis AQL passthrough (correctness fix, mandatory):** Remove the `maxResults`/
   `timeFrame` injection. Armis push-down is AQL passthrough per BC-2.11.007
   Mechanism B — the `aql` value from `QueryParams.filters["aql"]` (seeded by the
   user's WHERE clause `aql = '...'`) is already interpolated via `${query.filter.aql}`
   in the path_template. No additional push-down translation is needed or correct.
   This is a REMOVAL not an addition.

5. **Cyberint time-window (correctness fix, mandatory):** Remove the `from_date`/
   `to_date` body injection. It reaches a GET endpoint with no body. The correct
   behavior is: no time-window push-down for Cyberint. DataFusion post-filters.

6. **Claroty time-window (correctness fix, mandatory):** Remove any body injection
   for time-window. Claroty has no native time-window param. DataFusion post-filters.
   NOTE: Do NOT confuse this with the S-DEMO-CLAROTY-PAGINATION-001 offset/limit
   body fix — that is a separate deferred story.

7. **BC-2.01.013 per-sensor translation table (product-owner amendment, mandatory):**
   The table at lines ~107–110 falsely claims `Cyberint: POST body from_date/to_date +
   page_size` and `Claroty: POST body limit/offset` as "implemented." These must be
   corrected to document the actual correct behavior per this design note. PO owns this.

8. **Tests grounded in production spec shapes (mandatory per SAP-2):**
   ALL tests for push-down correctness MUST:
   - Load specs from real `*.sensor.toml` files (or assert against DTU struct fields
     derived from production TOML), NOT from `make_<sensor>_like_spec` fabricated
     constructors.
   - For end-to-end integration: use the real DTU clones (Wiremock pointing at
     `prism-dtu-crowdstrike`, etc.) and assert that the HTTP request received by the
     DTU carries the expected params.
   - SAP-2 applies as a standing gate on every adversarial pass: fabricated-fixture
     parity must be verified in pass 1.

### What is CLEANLY DEFERRED (named follow-up stories)

| Deferred scope | Reason | Follow-up story |
|----------------|--------|----------------|
| Cyberint page_size push-down | `AlertListParams` has no `page_size` field (DTU-EXT-005 open) | DTU-EXT-005 + new story when that gap closes |
| Claroty body-based offset/limit | Gap-CL-004 explicitly deferred to `S-DEMO-CLAROTY-PAGINATION-001` (open) | `S-DEMO-CLAROTY-PAGINATION-001` |
| Per-sensor `classify_predicates` integration (Option T2 above) | Requires fan-out orchestration restructuring; current story does not need this depth | New story at Wave 6 or ADR for push-down predicate architecture |
| Cursor seeding from PrismQL WHERE clause | No PrismQL syntax for initial cursor; speculative feature | Future story when PrismQL cursor syntax is defined |
| CrowdStrike devices table time-window | Shares same FQL mechanism as detections; can be added in v2 in the same fix burst if devices table path is verified. Include only if the devices DTU route struct is confirmed identical to detections (it is: same `DetectionListParams` pattern). | In-scope for v2 if devices is confirmed; else DTU-EXT-001 follow-on |
| Claroty time-window (native API params) | Real Claroty API may have `detected_after`/`detected_before` but these are NOT in the DTU; DTU must be extended first | `S-DEMO-CLAROTY-TIME-001` (new story, after DTU extension) |

**Production-grade rule 2 compliance:** every deferral above is an entire feature
(not a partial implementation). The v2 story either delivers CORRECT behavior for
a dimension or explicitly anchors it to a named story. No half-wired code.

---

## Section 4 — Testing Mandate

The adversarial passes identified a systematic gap: tests validated fabricated spec
fixtures rather than production shapes. SAP-2 as a standing gate applies.

### Mandatory test rules for v2 re-spec

1. **Spec fixtures must derive from production TOMLs.** Tests may either:
   a. Load the real `crates/prism-sensors/specs/<sensor>.sensor.toml` file via
      `include_str!` or the spec loader, OR
   b. Construct a minimal test spec that is a strict subset of the production TOML
      shape — with the same `method`, `body_template` presence/absence, pagination
      `type`, and step count. The test spec must NOT have a `body_template` if the
      production spec lacks one.

2. **DTU-grounded integration tests (preferred).** For each sensor's push-down
   behavior that IS real (CrowdStrike `filter`/`limit`, Armis `aql` passthrough),
   the gold-standard test is: start the real DTU clone, execute a PrismQL query
   with the push-down predicate, assert the DTU received the expected query params.
   Use `state.capture_aql()` / DTU session registry to verify.

3. **SAP-2 pass-1 gate.** The adversary MUST in pass 1 of v2 verify that every
   test fixture's `method`, `body_template`, and pagination type matches the
   production TOML for that sensor. Any fabricated-fixture mismatch is a
   P1 CRITICAL finding that resets the 3-CLEAN streak.

4. **AC-005 result-equivalence (F-P6-MED-001 closure).** The result-equivalence
   invariant test (BC-2.11.007: query result must be identical whether push-down
   occurs or not) MUST exercise the real two-step materialization path:
   `run_materialization_pipeline` → fan-out → `SpecDrivenSensorAdapter::fetch` →
   DTU clone. Not a unit test that constructs `FetchContext` directly.

5. **No `#[ignore]` for push-down ACs.** Per SID-1: if integration tests require
   DTU clones, the tests must either run against the DTU (ungated) or have a unit
   substitute per SID-1 §2. Push-down behavior that can only be proven by hitting
   the real DTU (AQL forwarding, FQL injection) must be ungated integration tests.

---

## Section 5 — ADR Recommendation

**A new ADR is warranted.** The push-down predicate-extraction architecture introduces
a concrete decision point:

> How does `prism-query`'s materialization layer extract time-window predicates from
> the PrismQL AST, and how are per-sensor ColumnSpec constraints applied pre-fan-out
> when the per-sensor spec is not yet resolved?

The current code has an explicit comment: "Per-sensor classify_predicates integration
deferred to wave-5 (see extract_push_down_filters_as_map docs for rationale)." The
v2 story closes this deferral for the time-window dimension using Option T1
(column-name heuristic against resolved specs). This is an architecture decision that
should be recorded.

**Recommended ADR decision statement** (do not allocate ADR-NNN ID yourself —
recommend it for `create-adr`):

> **Decision:** Time-window predicates (Gt/Ge/Lt/Le comparisons on `column_type =
> "datetime"` columns) are extracted from the PrismQL AST in `prism-query`'s
> `run_materialization_pipeline` before fan-out resolution, by walking `Predicate::Compare`
> nodes and matching lhs column names against datetime-typed columns in the resolved
> sensor specs available from `MaterializationContext.resolved_spec_map`. The extracted
> bounds are passed as `QueryParams.start_time`/`end_time` (ISO8601 strings) to all
> sensor adapters. Adapters that support native time-window params (currently: CrowdStrike,
> via FQL injection) consume these values; adapters without native support silently ignore
> them, falling back to post-materialization DataFusion filtering per BC-2.11.007
> result-equivalence invariant. Full per-sensor `classify_predicates` integration
> (Option T2) is deferred until fan-out orchestration is restructured to run
> classification post-target-resolution.

> **Rationale:** Option T1 is the minimal correct design for v2: it closes the
> dead-code gap without restructuring fan-out orchestration, delivers verified CrowdStrike
> time-window push-down, and correctly produces no-ops for sensors without native time params.
> Option T2 would deliver correct per-sensor predicate classification but requires
> restructuring the fan-out call sequence — a larger scope deferred to a future ADR.

Recommend routing to `create-adr` after the human approves the v2 scope slice.
Title suggestion: "ADR-NNN: Push-Down Time-Window Extraction Strategy (pre-fan-out
heuristic vs post-resolution classify_predicates)".

---

## Section 6 — BC-2.01.013 Factual Errors to Correct (Product-Owner Hand-Off)

BC-2.01.013 v1.12 §Postconditions "Per-sensor push-down translation" table (lines ~107–110)
contains four FACTUALLY WRONG claims that must be corrected. These were introduced by the
F-PUSHDOWN-008 product-owner fix-burst and are now known-wrong against production reality.

| Sensor | Current (Wrong) claim | Correct claim per this design note |
|--------|----------------------|------------------------------------|
| CrowdStrike | `filter` FQL time range (`created_timestamp:>'<start_time>'`) + `limit` query param | CORRECT direction, but `start_time` and `end_time` must both be populated at the materialization callsite (currently hardcoded None). FQL injection is the right mechanism once wired. |
| Cyberint | POST body `from_date` / `to_date` + `page_size` fields | **WRONG.** Cyberint `fetch_alerts` is GET with cursor-only (`AlertListParams.cursor`). No body_template. No `from_date`/`to_date` fields. Correct: no time-window push-down; cursor only; no page_size (DTU-EXT-005 open). |
| Claroty | POST body `limit` / `offset` fields | **WRONG.** `body_template: '{}'`; pagination via URL params (OffsetLimit pipeline). Body injection is a no-op. Correct: OffsetLimit via URL params (existing behavior); body-based pagination deferred to S-DEMO-CLAROTY-PAGINATION-001. |
| Armis | AQL `timeFrame` param + `maxResults` field | **WRONG.** `SearchQueryParams` has no `timeFrame` or `maxResults`. Correct: AQL verbatim passthrough (Mechanism B). No separate time-window param. |

The product-owner must amend BC-2.01.013 §Postconditions (per-sensor table + surrounding
narrative) and TV-BC-2.01.013-006 (should assert FQL injection for start_time AND end_time,
not just start_time, and must assert that wiring occurs via `run_materialization_pipeline`
not just `FetchContext` direct construction). BC-2.11.007 Invariants section claims "Time
range push-down is always attempted (all initial sensors support time-based filtering)" —
this must be qualified: only CrowdStrike has a usable native time param in the current DTU;
Armis/Cyberint/Claroty time-window is post-filter only until their DTUs expose native params.

---

## Section 7 — Story-Writer Hand-Off Instructions

The story-writer (re-authoring S-DEMO-QUERY-PUSHDOWN-001 v2) should use the following
as the ground truth for AC design.

### Story title (suggested)
"S-DEMO-QUERY-PUSHDOWN-001 v2: Correct per-sensor push-down wiring — CrowdStrike FQL
time-window + limit; Armis AQL correctness; materialization.rs wiring"

### Crates touched (v2)
- `prism-query` (materialization.rs + pushdown.rs)
- `prism-spec-engine` (pipeline.rs — remove wrong Cyberint/Claroty/Armis translations)
- `prism-bin` (spec_driven_adapter.rs — verify Armis AQL passthrough; CrowdStrike FQL)

### ACs to author (guidance — PO owns final wording)

- **AC-CWS-001:** CrowdStrike `query_detection_ids` step receives `limit=N` query param
  when `LIMIT N` appears in the PrismQL query. Test: DTU `DetectionListParams.limit`
  receives the value. Production spec shape: GET step, no body_template.

- **AC-CWS-002:** CrowdStrike `query_detection_ids` step receives `filter` query param
  containing FQL time constraint when PrismQL WHERE has `created_timestamp > 'T'` (and/or
  `< 'T'`). Fixture: production `crowdstrike.sensor.toml` shape. DTU
  `DetectionListParams.filter` receives the FQL string.

- **AC-CWS-003:** When neither start_time nor end_time is present in the query, no `filter`
  param is appended (or an empty filter is not sent). Existing behavior preserved.

- **AC-ARMIS-001:** Armis `fetch_devices` step receives `?aql=<value>` from the WHERE clause
  `aql = '<value>'` passthrough. No `maxResults` or `timeFrame` params appear. Production
  spec shape: GET, AQL passthrough, `path_template = "/api/v1/search?aql=${query.filter.aql}"`.

- **AC-ARMIS-002:** Armis push-down produces NO additional query params beyond `aql`,
  `offset`, `limit` (OffsetLimit pagination). Time-window is post-filter. DTU
  `SearchQueryParams` receives only these fields.

- **AC-CYB-001:** Cyberint `fetch_alerts` step receives NO `from_date`, `to_date`, or
  `page_size` params. These were wrong. Production spec: GET, no body_template, cursor-only.
  Time-window is post-filter only.

- **AC-CLAR-001:** Claroty `fetch_alerts` step receives NO time-window body fields.
  `body_template: '{}'` remains empty. OffsetLimit URL params (`?offset=N&limit=M`) are
  the pagination mechanism as before.

- **AC-WIRE-001:** `run_materialization_pipeline` populates `QueryParams.start_time` and
  `QueryParams.end_time` from the PrismQL AST when the WHERE clause contains
  Compare predicates (Gt/Ge/Lt/Le) on datetime-typed columns declared in the sensor spec.
  Verified by: run a PrismQL query against the CrowdStrike DTU with
  `WHERE created_timestamp > '2026-01-01T00:00:00Z'`; assert the DTU's
  `DetectionListParams.filter` contains `created_timestamp:>` content.

- **AC-EQUIV-001 (replaces fabricated AC-005):** Result-equivalence invariant: query with
  push-down predicates returns the SAME records as query without push-down (post-filter
  only). Test exercises the REAL materialization path: `run_materialization_pipeline` →
  DTU clone. Not a direct `FetchContext` construction test.

### Standing test mandate (SAP-2)
Every test that constructs a sensor spec MUST use production TOML shape. Adversarial
pass 1 will verify this. `make_crowdstrike_like_spec`, `make_cyberint_like_spec`,
`make_armis_like_spec` are forbidden in this story unless verified identical to
production shape.

### Out-of-scope for this story (anchor to named stories)
- Cyberint page_size: DTU-EXT-005
- Claroty body pagination: S-DEMO-CLAROTY-PAGINATION-001
- Claroty time-window: future story S-DEMO-CLAROTY-TIME-001 (new — not yet authored)
- Full `classify_predicates` integration: future ADR + story

---

## Summary: Human Decision Required Before Dispatch

This design note constitutes the architect's recommendation. Before the product-owner
and story-writer can proceed, the human should confirm:

1. **Scope slice acceptance:** Accept Option T1 (time-window extraction via column-name
   heuristic in `run_materialization_pipeline`) for the v2 story. This expands
   `crates_touched` to include `prism-query`.

2. **ADR creation:** Authorize `create-adr` for the push-down predicate-extraction
   architecture decision described in Section 5. (Route to architect after human approval.)

3. **BC-2.01.013 amendment:** Direct product-owner to correct the per-sensor table
   (Section 6) as part of this scope cycle before the story is re-authored.

4. **New follow-up story S-DEMO-CLAROTY-TIME-001:** Authorize creating this story
   (Claroty native time-window push-down, after DTU extension) to cleanly anchor
   that deferral.
