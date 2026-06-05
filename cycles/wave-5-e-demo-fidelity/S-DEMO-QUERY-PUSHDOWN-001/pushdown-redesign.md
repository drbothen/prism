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

**REVISED by §8 (human directive 2026-06-05):** Time-window push-down IS in scope for Armis
via AQL-clause augmentation. See §8 for the full design. The §1.2 row above documents
the DTU struct reality (no separate time-window param); §8 documents how the query-engine
layer augments the user's base AQL with a time-window clause appended into the same `aql`
param, so the DTU receives the combined AQL string containing both the entity discriminator
and the time filter. The DTU must also be extended to PARSE and HONOR the AQL time clause
so the scenario is load-bearing (§8.3). The "no time push-down" statement in the prior
version of this row is superseded for the v2 story scope.

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

---

## Section 8 — Armis AQL Full Wiring (Human Directive 2026-06-05)

> **Authority:** Human directive: "we will need to make sure we fully wired in Armis AQL
> into our DTU and our scenarios as well." This section SUPERSEDES the §1.2 "no time
> push-down" position for the v2 story scope. The §1.2 row has been annotated accordingly.

### 8.1 Production Reality Assessment

#### 8.1.1 What the DTU currently does with the `aql` string

From `crates/prism-dtu-armis/src/routes/search.rs` and `state.rs`:

The `get_search` handler receives `SearchQueryParams.aql: Option<String>`. It:

1. Calls `state.capture_aql(aql)` — appends the verbatim string to `aql_log` (no parsing, per R-DTU-002).
2. Does simple string-contains discrimination: `aql.contains("in:alerts") && !aql.contains("in:devices")` → alerts fixture; else → devices fixture.
3. Applies OffsetLimit pagination (`offset`/`limit` or `page`/`size`) to the selected fixture slice.
4. Returns the fixture slice without any further content-based filtering.

**Critical gap:** The DTU does NOT parse time-window clauses in the AQL string. If the query
engine appends `lastSeen:>"2026-01-01T00:00:00Z"` or any equivalent time clause to the AQL,
the DTU currently ignores it entirely and returns the full fixture. The round-trip scenario
for time-window push-down would be vacuous — the DTU would return the same dataset regardless
of the time filter in the AQL, making the scenario non-load-bearing.

This is confirmed by: the `#[ignore]`'d parity test in `prism-spec-engine/tests/parity/armis.rs`
line 383 uses `in:devices timeFrame:"Last 3 Hours"` as the AQL value, but the test never asserts
the returned dataset was time-filtered — it only asserts `aql_log` receipt and non-empty results.
The test is correctly `#[ignore]`'d because the scenario is currently vacuous.

#### 8.1.2 Real Armis AQL time-window syntax

The real Armis Centrix AQL time-window syntax is NOT definitively confirmed from the existing
codebase artifacts. The DTU `search.rs` module comment cites "research artifact 2026-06-01"
for `in:devices` / `in:alerts` entity syntax, but NO equivalent citation exists for time-window
AQL syntax. The `#[ignore]`'d parity test uses `timeFrame:"Last 3 Hours"` (a relative window),
but this was NOT grounded by a research artifact — it may be guessed.

**Known candidate syntaxes from existing project artifacts:**
- `timeFrame:"Last 3 Hours"` — appears in the parity test (line 383, armis.rs); no research citation
- `lastSeen:>"2026-01-01T00:00:00Z"` — appears as an example comment in §1.2 of this document;
  no research citation
- `after:"<timestamp>"` / `before:"<timestamp>"` — standard IS/was used in similar query languages;
  not confirmed for Armis AQL

**RESEARCH-AGENT VERIFICATION REQUIRED** (see §8.6). The implementer must NOT guess the AQL
wire syntax. The DTU extension (§8.3) can use any internal representation once the canonical
syntax is confirmed; the query-engine augmentation must emit the correct canonical form.

#### 8.1.3 Where push-down time-window becomes AQL content

The pipeline for Armis time-window push-down is a specialization of the Section 2 design,
applied at the AQL-augmentation layer rather than a separate query param:

```
PrismQL WHERE:
  aql = 'in:devices' AND last_seen > '2026-01-01T00:00:00Z'
                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^
                   Compare predicate on datetime INDEX column

→ run_materialization_pipeline (prism-query materialization.rs):
    [Step A] extract base AQL from equality predicate:  "in:devices"
    [Step B] extract time bounds from Compare predicates on datetime columns (Option T1):
               start_time = Some("2026-01-01T00:00:00Z")  [from `last_seen > 'T'`]
               end_time   = None
    [Step C] augment base AQL with time clause:
               augmented_aql = "in:devices <AQL_TIME_CLAUSE>"
               (AQL_TIME_CLAUSE = canonical Armis time syntax, research-confirmed)

→ QueryParams.filters["aql"] = augmented_aql
→ SpecDrivenSensorAdapter::fetch → FetchContext.query_filters["aql"] = augmented_aql
→ PipelineExecutor interpolates ${query.filter.aql} → DTU receives:
    GET /api/v1/search?aql=in:devices+<AQL_TIME_CLAUSE>
```

**Anti-double-filtering invariant:** The query-engine layer augments the AQL string; the
DataFusion post-filter STILL runs on the `last_seen` column. This is correct: the invariant
is result-equivalence (push-down is an optimization only). The DTU filtering is the
performance optimization; DataFusion filtering is the correctness backstop. No double-filtering
harm exists because DataFusion operates on the materialized OCSF output, not the AQL string.

**Base AQL + time-window combination rule:**
- If the user's AQL already contains a time clause, the query engine MUST NOT append another
  one (double-time-filter risk). Detection: check if the base AQL already contains the
  canonical time-clause keyword (the exact keyword is confirmed by research). If present,
  no augmentation — pass through verbatim. This preserves the user's explicit time scope.
- If no time clause is in the base AQL and the WHERE clause has a time Compare predicate on
  an Armis datetime column (`last_seen`, `first_seen`, `created_at`, `updated_at`), augment.
- If no time bounds are extracted (no Compare predicate on a datetime column), no augmentation —
  verbatim passthrough as before.

**Where the augmentation lives in the pipeline (prism-query):**
The `extract_push_down_filters_as_map` function (or its new sibling `extract_time_window_from_ast`)
already extracts `start_time`/`end_time` per Section 2.1 Option T1. The AQL-specific augmentation
is a per-sensor translation step: when the sensor is Armis (`sensor_id = "armis"`) and
`start_time`/`end_time` are populated, the `QueryParams.filters["aql"]` value is rewritten to
append the time clause. This translation belongs in the same place as the CrowdStrike FQL
injection — in `prism-spec-engine` or `prism-bin`'s per-sensor push-down translation layer,
applied AFTER time bounds are extracted from the AST.

### 8.2 AQL Augmentation Design

#### 8.2.1 Augmentation function (prism-spec-engine or prism-bin)

A new function `augment_armis_aql_with_time_window(base_aql: &str, start_time: Option<&str>, end_time: Option<&str>) -> String` is needed. Its contract:

- If both `start_time` and `end_time` are `None`: return `base_aql` unchanged.
- If base AQL already contains the canonical time keyword (to be confirmed by research): return
  `base_aql` unchanged (user's explicit time scope is preserved).
- Otherwise: append the time clause to `base_aql` using the canonical Armis AQL time syntax
  (research-confirmed). The exact syntax MUST be confirmed before implementation (§8.6).
- Result: a single augmented AQL string forwarded via `QueryParams.filters["aql"]`.

#### 8.2.2 Integration point

The augmentation is applied in the same code path as the CrowdStrike FQL injection:
- After `extract_time_window_from_ast` populates `start_time`/`end_time` on `QueryParams`
- Before `FetchContext.query_filters["aql"]` is committed to the pipeline executor
- Sensor-type check: only augment when `sensor_spec.sensor_id == "armis"` AND the table
  has an `aql` column with `options = ["INDEX"]` (to avoid augmenting hypothetical future
  sensors that also use AQL but with different syntax)

This means `prism-spec-engine` (pipeline.rs or the push-down translation module) gains an
Armis-specific AQL augmentation branch, parallel to the CrowdStrike FQL injection branch.
Alternatively, `prism-bin`'s `spec_driven_adapter.rs` can host it if per-sensor translation
already lives there. The implementer must choose the canonical location consistently with
where CrowdStrike FQL injection lands.

### 8.3 DTU Change Required (prism-dtu-armis)

**The DTU must be extended to parse and honor AQL time clauses so push-down scenarios are
load-bearing.** Without this change, any test that uses time-window AQL augmentation would
pass vacuously (DTU ignores the time clause, returns full dataset, result is the same as
without push-down).

#### 8.3.1 Minimal DTU change

Add time-clause parsing to `get_search` in `crates/prism-dtu-armis/src/routes/search.rs`:

1. **Parse time bounds from AQL string.** After `state.capture_aql(aql)` (R-DTU-002 capture
   is AQL-opaque and must not be removed), apply a SEPARATE parsing step to extract
   time bounds from the AQL string. This is NOT a violation of R-DTU-002 because:
   - R-DTU-002 prohibits validation/parsing that REJECTS or MODIFIES the AQL string.
   - Parsing for filtering purposes (using the parsed result to filter the fixture dataset)
     is within DTU scope — it makes the DTU behaviorally faithful to the real Armis API.
   - The AQL string is still captured verbatim (no modification, no rejection).

2. **Supported AQL time syntax in the DTU.** Support exactly the canonical time syntax
   confirmed by research (§8.6). The DTU only needs to support what the query engine emits;
   it does not need to implement the full Armis AQL grammar. Likely minimal set:
   - `after:"<ISO8601>"` or `lastSeen:>"<ISO8601>"` (whichever is canonical for start_time)
   - `before:"<ISO8601>"` or `lastSeen:<"<ISO8601>"` (whichever is canonical for end_time)
   - Regex or simple string extraction is sufficient; no full AQL parser required.

3. **Filter the fixture dataset by time bounds.** After entity-type discrimination
   (`in:devices` → `devices_ordered`; `in:alerts` → `alert_fixture`), apply time-bound
   filtering BEFORE pagination:
   - For devices: filter by `DeviceRecord.last_seen` (parse as ISO8601, compare to bounds).
     Handle `last_seen: null` per fixture convention (d-001 has `last_seen: null`) — null
     records FAIL the time filter unless `first_seen` is within bounds (mirrors real Armis
     behavior). The fallback chain for the filter check: `last_seen ?? first_seen`. If both
     are null, exclude the record from time-filtered results (conservative: no timestamp =
     cannot confirm in-window).
   - For alerts: filter by `AlertRecord.created_at` (parse as ISO8601, compare to bounds).
   - Open/closed interval semantics: `>` predicate → exclusive lower bound (start after T);
     `>=` → inclusive; `<` → exclusive upper bound; `<=` → inclusive. Match the semantics
     of the query-engine's `CompareOp::Gt/Ge/Lt/Le` so end-to-end correctness is verifiable.

4. **No change to `SearchQueryParams` struct.** The time bounds arrive embedded in the AQL
   string; no new struct fields are needed. `SearchQueryParams` stays as-is.

5. **No change to `capture_aql`.** The verbatim capture (R-DTU-002) is unaffected. The
   augmented AQL string (including the time clause) is captured as a single string — which
   allows scenario assertions to verify BOTH the entity discriminator AND the time clause
   appeared in the Aql log.

#### 8.3.2 Fixture data requirements for load-bearing scenarios

The existing `fixtures/devices.json` and `fixtures/alerts.json` must contain records with
timestamps that span the test scenario time windows. The implementer must verify:
- At least one device with `last_seen` BEFORE the test window (should be excluded).
- At least one device with `last_seen` WITHIN the test window (should be included).
- The `d-001` device (has `last_seen: null`, `first_seen: "2024-01-15T10:00:00Z"`) exercises
  the fallback chain.
- If the existing fixture data does not satisfy these requirements, the fixture JSON files
  must be extended with additional records. This is a DTU-internal change (no public API change).

### 8.4 Scenario Coverage Specification

The following scenarios are required for load-bearing Armis time-window push-down coverage.
They span three test locations.

#### 8.4.1 Unit tests — `prism-query/src/tests/aql_pushdown_tests.rs`

These extend the existing AC-014 unit tests (already present and green).

**New test: `test_BC_2_11_007_armis_time_window_augmented_into_aql_filter_map`**
- Input: PrismQL `WHERE aql = 'in:devices' AND last_seen > '2026-01-01T00:00:00Z'`
- Assertion: `extract_push_down_filters_as_map` + AQL augmentation step produce
  `FilterMap["aql"] = "in:devices <time_clause>"` where `<time_clause>` is the canonical
  Armis AQL time syntax for `last_seen > '2026-01-01T00:00:00Z'`.
- SID-1 compliance: no external dependency; exercises production code path.
- Load-bearing: fails if the augmentation function is absent or emits wrong syntax.

**New test: `test_BC_2_11_007_armis_base_aql_with_existing_time_clause_not_double_augmented`**
- Input: PrismQL `WHERE aql = 'in:devices after:"2026-01-01"'` AND `last_seen > '2026-01-01T00:00:00Z'`
- Assertion: `FilterMap["aql"]` is `'in:devices after:"2026-01-01"'` (no second time clause appended).
- Load-bearing: fails if the anti-double-filtering guard is absent.

**New test: `test_BC_2_11_007_armis_no_time_window_predicate_aql_passthrough_unchanged`**
- Input: PrismQL `WHERE aql = 'in:devices'` (no time Compare predicate)
- Assertion: `FilterMap["aql"]` is `'in:devices'` (no augmentation).
- Load-bearing: guards regression where augmentation is accidentally applied to base AQL.

#### 8.4.2 Pipeline-level integration tests — `prism-spec-engine/tests/parity/armis.rs`

These extend the existing AC-005 tests (already present and green for base AQL passthrough).

**New test: `test_BC_2_11_007_AC_005_aql_time_window_roundtrip_devices_pipeline`**
- Steps:
  1. Start Armis DTU clone (ephemeral port).
  2. Seed `FetchContext.query_filters["aql"]` with `"in:devices <time_clause>"` where the
     time clause filters to a known subset of the fixture dataset.
  3. Execute `PipelineExecutor::execute` for devices table.
  4. Assertion A (aql-log): DTU aql-log contains the augmented AQL string verbatim.
  5. Assertion B (filtering): result records are a PROPER SUBSET of the unfiltered devices
     fixture. The test runs a second query WITHOUT the time clause and asserts that
     `filtered_count < unfiltered_count` AND all filtered records have timestamps within
     the specified window. This is the load-bearing assertion — if DTU does not honor the
     time clause, `filtered_count == unfiltered_count` → test fails.
  6. Assertion C (result-equivalence): every record in the filtered result also appears in
     the unfiltered result (no fabrication by the DTU).
- Load-bearing: fails if (B) `filtered_count == unfiltered_count` (DTU ignores AQL time).
- Subsumes the vacuous parity test's `timeFrame:"Last 3 Hours"` AQL value. The correct AQL
  syntax for the time clause must use the research-confirmed form (§8.6), not `timeFrame:`.

**New test: `test_BC_2_11_007_AC_005_aql_time_window_roundtrip_alerts_pipeline`**
- Same structure as the devices test, using `"in:alerts <time_clause>"` against
  `AlertRecord.created_at`.

#### 8.4.3 E2E subprocess test — `prism-bin/tests/e2e_smoke.rs`

**New test: `test_BC_2_11_007_e2e_armis_aql_time_window_pushdown_dtu_roundtrip`**
- Extends the existing `test_BC_2_11_007_e2e_armis_aql_pushdown_devices_dtu_roundtrip`
  pattern.
- Issues: `SELECT * FROM armis_devices WHERE aql = 'in:devices' AND last_seen > '2024-01-01T00:00:00Z'`
- Assertion A: prism returns non-empty data rows.
- Assertion B: Armis DTU aql-log contains an entry with both the entity discriminator
  (`in:devices`) AND the time clause (confirming augmentation reached the DTU).
- Assertion C: result row count is <= full unfiltered row count from the same DTU instance.
- `#[ignore]` with `E2E-001` annotation (requires DTU + prism binary; un-gated via e2e profile).
- Load-bearing: Assertion B fails if query-engine augmentation is absent; Assertion C alone
  would be vacuous without the aql-log check.

#### 8.4.4 Result-equivalence invariant test (BC-2.11.007 §Invariants)

The existing AC-EQUIV-001 from §7 must be extended to cover Armis:

**Extended test: `test_BC_2_11_007_result_equivalence_armis_time_window_via_pushdown_vs_postfilter`**
- Two executions against the Armis DTU:
  - Execution A: PrismQL query WITH `last_seen > 'T'` push-down AND DataFusion post-filter.
  - Execution B: PrismQL query WITHOUT push-down (no AQL time clause) but WITH DataFusion
    post-filter on `last_seen > 'T'`.
- Assertion: result sets are identical (same records, same order-independent comparison).
- This is the canonical result-equivalence assertion per BC-2.11.007 §Invariants.
- Lives in `prism-spec-engine/tests/parity/armis.rs` or `prism-bin/tests/e2e_smoke.rs`.

### 8.5 Revised Crates Touched

| Crate | Change | Reason |
|-------|--------|--------|
| `prism-query` | `materialization.rs` + `pushdown.rs` — extract `start_time`/`end_time` from AST (Section 2 Option T1) | Wires time bounds from AST to `QueryParams` for all sensors including Armis |
| `prism-spec-engine` | Per-sensor push-down translation — add Armis AQL augmentation branch (alongside CrowdStrike FQL injection) | Translates `start_time`/`end_time` into AQL clause, appends to base AQL |
| `prism-bin` | `spec_driven_adapter.rs` — remove wrong `maxResults`/`timeFrame` injection; verify Armis AQL passthrough is correct; if AQL augmentation lives here, add it here | Correctness fix + augmentation wiring |
| `prism-dtu-armis` | `routes/search.rs` — add AQL time-clause parsing and fixture dataset filtering (§8.3) | Makes time-window scenarios load-bearing; without this, scenarios are vacuous |
| `prism-sensors` | `specs/armis.sensor.toml` — no structural change needed; `last_seen`, `first_seen`, `created_at`, `updated_at` are already declared as `column_type = "datetime"`; verify `options = ["INDEX"]` on at least one of them to enable time push-down via Option T1 | Confirm INDEX option exists on Armis datetime columns |

**Confirm on prism-sensors:** The current `armis.sensor.toml` does NOT declare `options = ["INDEX"]`
on `last_seen`, `first_seen`, `created_at`, or `updated_at`. Only `aql` has `options = ["INDEX"]`.
For Option T1 time-window extraction to work for Armis, at least `last_seen` (devices) and
`created_at` (alerts) must be declared `options = ["INDEX"]`. **This is a required prism-sensors
change.** Without it, Option T1 cannot identify these as push-down-eligible datetime columns
for Armis, and the query engine cannot extract time bounds for Armis AQL augmentation.

**Revised crates_touched for v2 story:** `[prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-sensors]`

### 8.6 AQL Wire-Syntax Research Flag

**RESEARCH-AGENT VERIFICATION REQUIRED before implementation.**

The canonical Armis AQL time-window syntax for use in `GET /api/v1/search?aql=...` is NOT
confirmed from existing project artifacts. The following questions must be answered:

1. **Absolute timestamp syntax:** Does real Armis AQL accept `lastSeen:>"2026-01-01T00:00:00Z"`?
   Or `after:"2026-01-01T00:00:00Z"`? Or a different form?
2. **Field name:** For the devices table time filter, is the AQL field name `lastSeen` (camelCase
   as in the real API) or `last_seen` (snake_case as in the TOML spec column name)?
3. **Relative window syntax:** Is `timeFrame:"Last 3 Hours"` (as used in the parity test, line 383)
   confirmed real Armis AQL syntax, or was it guessed?
4. **Alerts table time field:** Is the AQL field for alert time-filtering `createdAt`, `created_at`,
   `time`, or something else?

**Source to research:** Armis Centrix API documentation, the 1898 production poller (referenced
in `search.rs` module comment as "1898 production poller"), and any Armis community or partner
documentation. The research artifact cited in `search.rs` as "research-artifact 2026-06-01"
confirmed `in:devices`/`in:alerts` syntax but did not confirm time-window syntax.

Until research confirms the canonical syntax, the DTU extension (§8.3) should use a PLACEHOLDER
time syntax internally and expose a `#[cfg(test)]` constant `ARMIS_AQL_TIME_CLAUSE_SYNTAX` that
the story-writer can reference. The placeholder prevents shipping guessed wire syntax.

**Research-agent query to issue (route to `vsdd-factory:research-agent`):**
"What is the canonical Armis Centrix AQL syntax for filtering by absolute timestamp range in
a search query? Specifically: (1) the AQL field name for device last-seen time, (2) the operator
syntax for after/before an ISO8601 timestamp, (3) the AQL field for alert creation time.
Sources: Armis developer documentation, Armis community, Armis partner API reference."

### 8.7 BC + Story Hand-Off

#### 8.7.1 BC amendments required (route to product-owner)

**BC-2.01.013 Armis row (v1.13 → bump):**
- Current Armis row says: "AQL verbatim passthrough only; no time-window wiring."
- Required correction: "Armis time-window push-down IS in scope. Mechanism: query-engine
  augments the user's base AQL with a canonical Armis AQL time clause derived from
  `start_time`/`end_time` bounds extracted from the WHERE clause via Option T1 heuristic
  (ADR-033). The augmented AQL string is forwarded via the existing `${query.filter.aql}`
  path. The DTU honors the time clause by filtering its dataset (§8.3). Result-equivalence
  invariant preserved: DataFusion post-filter ensures correctness regardless of DTU filtering."
- The PO must also confirm the canonical AQL time syntax once research-agent verifies it.

**BC-2.11.007 Mechanism B (v1.7 → bump):**
- Current Mechanism B description says: "Remaining post-filter predicates are applied by
  DataFusion over the materialized OCSF table. This includes ALL time-window predicates for
  Armis, Cyberint, and Claroty sensors (which lack native DTU time params)."
- Required correction: Armis is removed from the "no native time params" list. The updated
  text should describe the AQL-clause augmentation: time-window predicates on Armis datetime
  columns (declared `options = ["INDEX"]`) are augmented into the AQL string. The DTU honors
  this augmentation. Cyberint and Claroty remain in the post-filter-only camp.
- The invariant section's "only CrowdStrike" statement must be updated: "CrowdStrike (FQL
  injection) and Armis (AQL-clause augmentation) support time-window push-down. Cyberint and
  Claroty do not, pending DTU extension."

#### 8.7.2 New ACs for story v2 (route to story-writer)

These supplement the ACs in §7 (which should be preserved). Add:

**AC-ARMIS-TW-001:** PrismQL query `SELECT * FROM armis_devices WHERE aql = 'in:devices' AND last_seen > 'T'`
produces a `QueryParams.filters["aql"]` value containing both `in:devices` AND the canonical
Armis AQL time clause for `last_seen > 'T'`. Verified at the FilterMap/QueryParams boundary
(unit test in `aql_pushdown_tests.rs`).

**AC-ARMIS-TW-002:** The Armis DTU, when receiving an AQL string containing both an entity
discriminator (`in:devices`) and a time clause, returns only records whose timestamp falls
within the specified window. The returned record count is strictly less than the unfiltered
count (fixture must have records both inside and outside the window). Verified by pipeline
integration test (`parity/armis.rs`).

**AC-ARMIS-TW-003:** If the user's base AQL string already contains a time clause, the
query engine does NOT append a second time clause. The forwarded AQL string equals the
user's literal value. Verified by unit test in `aql_pushdown_tests.rs`.

**AC-ARMIS-TW-004:** Armis result-equivalence invariant: query with AQL time push-down
returns the same records as query without push-down (DataFusion post-filter only on
`last_seen > 'T'`). Both paths produce identical row sets. Verified by integration test
against the Armis DTU.

**AC-ARMIS-TW-005 (E2E):** Full subprocess e2e: `SELECT * FROM armis_devices WHERE aql = 'in:devices' AND last_seen > 'T'` via prism binary, Armis DTU aql-log contains the augmented AQL string (both entity discriminator and time clause). `#[ignore]` per SID-1 / E2E-001.

#### 8.7.3 DTU-INDEX update for prism-sensors datetime options

The story-writer's ACs for v2 must include a gate that verifies `last_seen` (devices) and
`created_at` (alerts) in `armis.sensor.toml` carry `options = ["INDEX"]`. Without this, the
Option T1 time-window extraction cannot identify Armis datetime columns as push-down-eligible.
The implementer must add `options = ["INDEX"]` to these columns as part of the v2 story.

---

## Summary of §8 for Human Decision

**Revised Armis mechanism:** Time-window push-down is in scope via AQL-clause augmentation.
The query engine extracts time bounds from WHERE Compare predicates on Armis datetime INDEX
columns, and appends the canonical Armis AQL time clause to the user's base AQL string. The
combined AQL string is forwarded via the existing `${query.filter.aql}` path. The DTU is
extended to parse and honor the AQL time clause, making scenarios load-bearing.

**Revised crates_touched:** `[prism-query, prism-spec-engine, prism-bin, prism-dtu-armis, prism-sensors]`

**DTU change needed:** `prism-dtu-armis/src/routes/search.rs` — add AQL time-clause parsing
and fixture dataset filtering. `prism-sensors/specs/armis.sensor.toml` — add `options = ["INDEX"]`
to `last_seen` and `created_at` datetime columns.

**Scenario coverage plan:** 3 unit tests (aql_pushdown_tests.rs), 3 pipeline integration tests
(parity/armis.rs), 1 e2e subprocess test (e2e_smoke.rs). Each has load-bearing assertions.
The critical load-bearing assertion is Assertion B of §8.4.2: `filtered_count < unfiltered_count`.

**AQL wire-syntax research required:** YES. Dispatch research-agent before implementation
begins. The canonical Armis AQL time-window field names and operator syntax must be confirmed.
Do not implement §8.3 DTU parsing or §8.2 augmentation with guessed syntax.

**Hand-off:**
- Product-owner: amend BC-2.01.013 Armis row (v1.13→bump, time-window IN scope) and
  BC-2.11.007 Mechanism B (remove Armis from post-filter-only list, document AQL augmentation
  + DTU-honors contract). Wait for research-agent confirmation of AQL time syntax before
  specifying exact syntax in BCs.
- Story-writer: add ACs AC-ARMIS-TW-001 through AC-ARMIS-TW-005 to the v2 story. Add
  `prism-dtu-armis` and `prism-sensors` to `crates_touched`. Ensure SAP-2 standing gate
  applies: DTU time-clause filtering must be load-bearing (filtered ≠ unfiltered).
