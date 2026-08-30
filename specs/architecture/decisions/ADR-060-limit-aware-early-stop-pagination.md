---
document_type: adr
adr_id: "ADR-060"
title: "LIMIT-Aware Early-Stop Pagination for Offset/Limit and Cursor Sensor Tables"
status: ACCEPTED
date: "2026-08-26"
modified: "2026-08-30"
version: "1.18"
producer: architect
subsystems_affected: [SS-01, SS-07, SS-11, SS-16]
supersedes: []
superseded_by: null
amends: null
anchor_stories:
  - S-ENGINE-LIMIT-EARLY-STOP-001
related_adrs: [ADR-028, ADR-033, ADR-058]
related_bcs: [BC-2.16.002, BC-2.16.015, BC-2.01.010]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-060: LIMIT-Aware Early-Stop Pagination for Offset/Limit and Cursor Sensor Tables

## Status

ACCEPTED v1.18 (2026-08-30) — F-B1V-002 (MEDIUM/spec-accuracy): §D8.3 worked example (d) corrected — arithmetically unreachable numbers (page_size=1000, LIMIT=5, partial-page=3) replaced with reachable scenario from `test_early_stop_multi_batch_partial_page_is_truncated` (page_size=10, LIMIT=5; batch-0 non-final is_last_batch=false returns partial page 5 records (5 < 10), accumulated=5 ≥ limit=5 → early-stop fires, discriminator `(5 >= 10) || !is_last_batch = false || true = true` → `early_stopped=true`; batch-1 abandoned by `break 'steps`); §D8.2/§D8.9 discriminator formula and disambiguation note unchanged; rows (a)/(b)/(c) unchanged. Closes F-B1V-002. v1.17 (2026-08-30) — F-B1-001 (MEDIUM): §D8.2 discriminator formula extended with intra-pipeline step fan-out term (`early_stopped = (page_record_count >= active_page_size) || !is_last_batch`); §D8.3 worked example (d) added (non-final batch → `early_stopped = true`); disambiguation note added (intra-pipeline step fan-out vs multi-sensor `FanOutResult.any_early_stopped` OR-aggregation, §D8.9/§D8.10) in §D8.2 and §D8.3; documents implemented+verified behavior (code @704aac24a; `test_early_stop_multi_batch_partial_page_is_truncated`, GREEN). Closes F-B1-001. v1.16 (2026-08-29) — F-P9-LENSA-001 DECISION (A): §D8.4 spec-reconcile — `PaginationConfig::None` moved from contradictory "does NOT apply to None" exclusion to explicit conservative-bucket documentation; `_ => 0` catch-all in `execute_impl` captures None same as CursorToken; exact-LIMIT corner (`row_count == LIMIT` → `is_truncated: true`) documented as accepted safe over-report; §D8.2 NOTE updated to cite None alongside CursorToken; zero v1 exposure; no code change; feature HEAD 62e50205b frozen unchanged. v1.15 (2026-08-29) — F-P2-LENSC2-001: §D8.4 Dim-3 discharge note stale version pin removed (STORY-INDEX reference now cites draft status without version number; POL-39 compliant); F-P2-LENSC2-002: §Status banner POL-39 sweep count corrected from ~14 to 20 (matches §Changelog enumeration of 20 items; banner and changelog now self-consistent). v1.14 (2026-08-29) — F-P1B-LENSC2-001/002: §D8.4 TD-VSDD-097 Dim-3 note updated (S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001 now registered in STORY-INDEX, deferral anchored to existing story, prohibitive MUST active); §D8.7 heading volatile version stamp removed; POL-39 sweep: 20 normative in-body version pins anchor-ized across §D8.7/§D8.9; ~6 decision-history version refs preserved as intentional with convention note added to §D8.7. v1.13 (2026-08-29) — F-FP1-LENSA-001 DECISION (B): cursor page-fill discriminator is unsound — revert + narrow + anchor. §D8.2 code comment: CursorToken collapsed to `_ => 0` catch-all (removes `CursorToken { page_size: Some(ps) } => ps as usize` arm added v1.12); comment variable renamed `active_page_size` (was `page_size`) throughout to match §D8.4 + ratified impl naming (F-FP1-LENSC2-002). §D8.4: CursorToken narrowed to conservative-only across ALL sub-cases; rationale: page-fill is NOT a valid cursor exhaustion signal (partial cursor page + non-empty next cursor = more data exists — under-report is the DANGEROUS direction); all CursorToken → `active_page_size = 0` → `early_stopped = true` (safe over-report); precise next-cursor-presence-based detection deferred to S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001 (post-v1, blocked on S-OCSF-FIDELITY-CYBERINT-001). v1.12 (2026-08-29) — F-P1-LENSC2-001/002/003: §D8.2 `page_size` derivation comment extended to all early-stop-eligible modes (OffsetLimit + CursorToken); §D8.3 forward-reference anchor discharged; in-body version-pin sweep: 0 found. v1.11 (2026-08-28) — F-P31-LENSA-OBS-001: partial-final-page discriminator for early-stop signal (§D8.2, §D8.3, §D8.9); exact-limit/partial-final-page LIMIT query no longer emits self-contradictory `is_truncated: true` with `total_available == returned_results`. v1.10 (2026-08-28) — §D8.9 FetchOutput 3-field reconciliation + DI-019 propagation arm, F-P20-LENSC-MED-001. v1.9 (2026-08-28) — F-R16-P18-LENSA-MED-001 (DI-019 truncation-signal propagation gap): `PipelineResult.truncated` (DI-019 cap) was dropped at the adapter boundary; new §D8.10 threads `pipeline_truncated` through `FetchOutput → FanOutResult.any_pipeline_truncated → MaterializationOutput.any_pipeline_truncated`; cache-completeness gate updated to `errors.is_empty() && !any_early_stopped && !any_pipeline_truncated`; engine Step 6 formula updated to `(total_rows > limit) || any_early_stopped || any_pipeline_truncated`; scheduled path `is_truncated: false` hardcode replaced by `any_early_stopped || any_pipeline_truncated` (F-R16-P18-LENSA-OBS-001 sibling-sweep); RG-PSG-035/036 required. v1.8 (2026-08-28) — F-R16-P16-LENSA-HIGH-001: source-scoped `datetime_index_cols` via `resolved_col_map` (§D8.9); F-R16-P16-LENSA-LOW-001: reversed-operand prohibition explicit in `is_pushed_temporal_predicate` (§D8.7); F-R16-P16-LENSB-LOW-001: Condition K multi-INDEX-datetime conservative suppression (§D8.7); structural-reuse `collect_datetime_index_cols` helper; RG-PSG-032/033/030b required. v1.7 (2026-08-28) — AND-arm direction-count constraint (§D8.7) + OCSF-name gap in `datetime_index_cols` (§D8.9). v1.6: ADR-059 citation reframe. v1.5 (2026-08-27) — Temporal-exemption soundness redesign (§D8.9): `is_pushed_temporal_predicate` replaces `is_purely_temporal_predicate`; `Ast::Filter` + `PipeStage::Where` unconditionally SUPPRESS in `has_client_side_where`; `expr_contains_aggregate_or_window` catch-all `_ => false` → `_ => true`; `any_early_stopped` truncation-signal chain added (§D8.9). v1.4: Subsystem-anchoring correction: SS-11 + SS-07 added. v1.3: Comprehensive plan-shape surface audit. §D8.7 closes F-R12-CRIT-001
(aggregate recursion gap) and F-R12-HIGH-001 (JOIN not suppressed), plus six additional gaps
discovered by exhaustive grammar enumeration: ORDER BY aggregate escapes Condition A; Condition G
was based on `where_filters` (equality push-down map) which is always empty for `Ast::Filter` mode
and `Ast::Pipe` stages, and misses non-equality client-side predicates (CONTAINS, BETWEEN, etc.);
`PipeStage::Tail` not suppressed; `FuncCall::Window` not suppressed; no conservative default
posture for unknown AST/PipeStage variants. Gate redesigned with complete condition set A–K plus
conservative allowlist default. Signature change: `where_filters` parameter removed (gate performs
its own AST inspection). All out-of-grammar shapes documented. Verdict: surface is bounded and
complete — no deferral recommended. v1.2: §D8.7 plan-shape gate, Conditions A–G.
v1.1: §D8.1 prose correction. v1.0: initial D8 LIMIT-aware early-stop.

---

## Context

### Defect Evidence

Live monroe validation of S-CLAROTY-VULNS-001 revealed that a query
`SELECT * FROM claroty_vulnerabilities | LIMIT 1` downloads the FULL dataset (5000+
vulnerability records across multiple pages) before DataFusion applies the LIMIT clause,
consistently exceeding the 30s query budget (E-QUERY-004).

**Note on DEFECT-1 (ADR-059, WITHDRAWN):** ADR-059 is WITHDRAWN (D-2312: the h2
flow-control window hypothesis was falsified by live wire evidence; no transport change was
applied). DEFECT-2 (this ADR) is **independent** — the LIMIT over-fetching defect exists
regardless of h2 transport behavior. The original framing that implied DEFECT-2 was observed
"even after ADR-059 was applied" was imprecise and is corrected here: both defects were
investigated in the same S-CLAROTY-VULNS-001 live session, but DEFECT-2 does not depend on
DEFECT-1 having been resolved first.

Root cause: `PipelineExecutor::execute_impl` fetches ALL pages until the API signals pagination
exhaustion or the 10K DI-019 cap is hit. There is no mechanism to stop fetching when the
accumulated record count satisfies the query's LIMIT. DataFusion applies its LIMIT operator on
the final materialized record batch, too late to prevent excessive HTTP fetches.

Concretely: `claroty_vulnerabilities` page_size = 1000 rows, ~1.1 MB/page. `LIMIT 1` requires
1 record but triggers 5+ HTTP requests (~5.5 MB total). At the per-page h2 fix latency
(estimated 5-10s/page for large pages), the 30s budget is easily exceeded.

### Atomicity Reconciliation

**CRITICAL:** Before specifying the fix, the "atomic" language in existing contracts must be
adjudicated to determine compatibility.

**BC-2.16.015 §Error Cases uses "atomic-fail" in two rows:**
- `E-SENSOR-001` (HTTP non-200): "the entire fetch returns the structured error … no partial/accumulated pages are returned (atomic-fail; Option-A fail-fast)"
- `E-SPEC-018` (timestamp parse failure): "the `?` discards the entire accumulated result — the fetch fails atomically and NO partial pages are returned"

**BC-2.16.002 §Postconditions states:**
- "Partial-record discard on mid-pipeline HTTP failure: ALL records accumulated from prior successfully-completed steps are discarded … This is the 'all-or-nothing' semantic"

**Ruling — "Atomic" means all-or-nothing on HTTP ERROR, not "must fetch the entire dataset"**

The "atomic" guarantee is an ERROR-PATH invariant: when the pipeline fails mid-pagination
(HTTP non-200, network timeout, parse error), the partial result is discarded and an `Err()`
is returned. This prevents misleading partial data from reaching OCSF mappers.

LIMIT-aware early-stop is NOT an error path. It is a deliberate, successful, non-error early
exit driven by query semantics. The pipeline successfully fetches some number of COMPLETE pages,
accumulates enough records to satisfy the LIMIT, and returns those records to DataFusion. No
`Err()` is returned; no data is discarded.

Evidence that this interpretation is correct:

1. **DI-019 precedent**: the existing 10K truncation (AC-8) already halts pagination before the
   full dataset is returned, sets `truncated: true`, and returns a valid `PipelineResult`. This
   is exactly the same pattern — non-error early stop — and has never been considered a
   violation of the atomicity guarantee.

2. **Textual scope**: both atomic-fail citations appear in `§Error Cases` tables, not in
   `§Postconditions` or `§Invariants`. Their scope is explicitly about failure behavior.

3. **Design intent**: the "all-or-nothing" postcondition in BC-2.16.002 provides the rationale:
   "partial PipelineResult could mislead downstream OCSF mappers into producing schema-mismatched
   rows." This risk does not apply when early-stopping at a COMPLETE page boundary on the SUCCESS
   path — each page is fully received and parsed; no row is partially constructed.

**Compatibility verdict:** LIMIT push-down early-stop is COMPATIBLE with the existing atomicity
guarantee. BCs do not need to weaken the error-path atomic invariant; they need only add a new
postcondition describing the success-path early-stop behavior.

### Sort-Order and Fan-Out

**Sort-order:** When a query includes `ORDER BY`, DataFusion applies sorting post-fetch on the
accumulated records. LIMIT push-down may return only the first N pages (in API-declared order),
which may not be the globally-sorted first N records. This is INTENTIONAL: the engine cannot
sort across pages it has not fetched. Consumers wanting globally-sorted top-N results MUST either:
(a) omit LIMIT and use ORDER BY + explicit LIMIT post-sort, or
(b) ensure the sensor API supports server-side ORDER BY (declared via future TOML `sort_by`).
This limitation is documented as a BC postcondition.

**Fan-out (multi-sensor queries):** Each sensor pipeline in a fan-out executes independently.
LIMIT push-down applies per-pipeline independently. Each pipeline fetches the minimum pages to
satisfy the LIMIT. DataFusion applies the global LIMIT across the combined fan-out results. This
means each pipeline may return up to `LIMIT` records, and the combined result before DataFusion
trim may have up to `(sensor_count × LIMIT)` records — acceptable for the query planner.

---

## Decision

**D8 — PipelineExecutor SHALL stop fetching additional pages once accumulated records satisfy
the query's LIMIT (early-stop pagination)**

### D8.1 — LIMIT threading via FetchContext

**Gating precondition (§D8.7):** The `fetch_limit` that flows into `QueryParams.limit` is set
to 0 whenever the plan is classified as "reducing" by `ast_is_reducing_plan`. When
`fetch_limit = 0`, `QueryParams.limit = 0`, and `FetchContext::early_stop_limit = None`, so
early-stop does not fire. The threading described below applies only when the plan-shape gate
permits early-stop (i.e., `ast_is_reducing_plan` returns `false`).

A new `early_stop_limit: Option<usize>` field is added to `FetchContext`. This field is distinct
from the `query.limit` entry in `query_filters` (which is for TOML path_template interpolation,
e.g., CrowdStrike `DetectionListParams.limit`). The two are independent:
- `query_filters["query.limit"]` = limit value to inject INTO the sensor API request URL/body
- `early_stop_limit` = limit on how many records prism will accumulate before stopping pagination

`FetchContext::new()` gains a parameter `early_stop_limit: Option<usize>`. Callers
(`spec_driven_adapter.rs`) read `QueryParams.limit: u64` — the pre-extracted query LIMIT field
already present on `QueryParams` before `FetchContext` is constructed; no DataFusion physical-plan
inspection is required. Callers pass `Some(params.limit as usize)` when `params.limit > 0`, or
`None` when `params.limit == 0` (meaning no LIMIT was specified in the query). The behavior is
unchanged when no LIMIT is present.

### D8.2 — Check point in execute_impl

After each complete page is accumulated (immediately after the DI-019 truncation check), the
pagination loop adds:

```rust
if let Some(limit) = context.early_stop_limit {
    if all_records.len() >= limit {
        // Discriminator formula (§D8.3):
        // page_record_count = records returned by this page (pre-OCSF count, i.e. raw page len).
        // active_page_size  = mode's declared maximum records per page:
        //   OffsetLimit { page_size }  => page_size as usize
        //   CursorToken { .. } | _    => 0  (conservative: page-fill is NOT a valid cursor
        //                                    exhaustion signal; `_` also captures
        //                                    PaginationConfig::None — see §D8.4 for rationale;
        //                                    precise cursor detection deferred to
        //                                    S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001)
        // is_last_batch = (batch_idx + 1 == batch_count) — INTRA-PIPELINE step fan-out only
        //   (iterates fan_out_batches / fan_out_batch_size within this sensor's pipeline;
        //    evaluated inside batch-loop → step-loop of execute_impl; DISTINCT from
        //    multi-sensor fan-out at the FanOutResult layer — see §D8.9/§D8.10 and §D8.3
        //    disambiguation note).
        //
        // early_stopped = (page_record_count >= active_page_size) || !is_last_batch
        //
        // Non-final batch (!is_last_batch = true): break 'steps abandons remaining step-fan-out
        //   batches → data genuinely incomplete → early_stopped = true (page fill irrelevant).
        // Final batch (is_last_batch = true): falls back to page-fill discriminator.
        //   Full page  (page_record_count >= active_page_size): source may have more pages → true.
        //   Partial page (page_record_count < active_page_size): source exhausted → false.
        // Common no-fan-out case (batch_count == 1): is_last_batch always true →
        //   formula reduces to page_record_count >= active_page_size (unchanged from prior behavior).
        // NOTE: For CursorToken and PaginationConfig::None, active_page_size = 0 always →
        //       page-fill arm: 0 >= 0 = true → early_stopped = true (conservative; see §D8.4).
        early_stopped = (page_record_count >= active_page_size) || !is_last_batch;
        break 'steps;
    }
}
```

This check fires only after a COMPLETE page has been received and its records appended to
`all_records`. It does NOT fire mid-page. The page atomicity guarantee is preserved: either
the entire page arrives (and is accumulated), or a fetch error discards everything.
The `early_stopped` local variable records the discriminator formula result so that
`PipelineResult::early_stopped` is set correctly per §D8.3.

### D8.3 — Post-break semantics

When early-stop fires (not DI-019 cap), the `truncated` flag is NOT set. `PipelineResult.early_stopped`
is set using the **discriminator formula** captured at break time (§D8.2):
`early_stopped = (page_record_count >= active_page_size) || !is_last_batch`.

- **Non-final intra-pipeline step fan-out batch** (`!is_last_batch = true`, where `is_last_batch = (batch_idx + 1 == batch_count)` inside `execute_impl` batch-loop): `break 'steps` abandons the remaining step-fan-out batches — data is genuinely incomplete — `early_stopped = true` regardless of page fill. (See disambiguation note below for the distinction between intra-pipeline and multi-sensor fan-out.)
- **Final batch, full page** (`is_last_batch = true`, `page_record_count >= active_page_size`): the source may have more pages — `early_stopped = true`. This includes the exact-full-page-boundary corner (a full final page is treated conservatively as "more may exist" because exhaustion was not confirmed without fetching the next page).
- **Final batch, partial page** (`is_last_batch = true`, `page_record_count < active_page_size`): the source is exhausted (no more pages remain); the returned data IS the complete dataset — `early_stopped = false` (does NOT contribute to `is_truncated`).

The pipeline returns a valid `PipelineResult` with `truncated: false` and `early_stopped` per the
discriminator, containing at most `limit + (page_size - 1)` records. DataFusion applies the precise
LIMIT on this result. The implementer MUST NOT set `truncated: true` for LIMIT early-stop — `truncated`
is semantically reserved for capacity-exceeded conditions (DI-019), not for query-driven early stops.
(Anchor: S-ENGINE-LIMIT-EARLY-STOP-001 AC-014 + RG-PSG-039 `test_BC_2_16_002_early_stop_partial_final_page_not_early_stopped` + RG-PSG-040 `test_psg_rg040_partial_final_page_is_truncated_false_wire`, GREEN.)

**Worked examples (discriminator formula):**

| Scenario | page_size | Tenant rows | LIMIT | Final page shape / fan-out context | early_stopped | total_rows > limit | is_truncated | Correctness |
|----------|-----------|-------------|-------|-------------------------------------|---------------|--------------------|--------------|-------------|
| (a) Exact-limit on exhausted tenant: `LIMIT 5`, tenant has 5 rows | 1000 | 5 | 5 | Partial (5 < 1000), is_last_batch=true | false | false | false | Correct: complete dataset returned |
| (b) Normal early-stop: `LIMIT 5`, tenant has 1000+ rows | 1000 | 1000+ | 5 | Full (1000 >= 1000), is_last_batch=true | true | false | true | Correct: more data exists |
| (c) Exact-full-page corner: `LIMIT 1000`, tenant has exactly 1000 rows | 1000 | 1000 | 1000 | Full (1000 >= 1000), is_last_batch=true | true | false | true | Conservative: exhaustion unconfirmed without fetching next page (accepted corner) |
| (d) Non-final intra-pipeline fan-out batch: `LIMIT 5`, page_size=10; 4 IDs at fan_out_batch_size=2 → 2 batches; batch-0 (batch_idx=0, non-final, is_last_batch=false) returns partial page 5 records (5 < 10), accumulated=5 ≥ limit=5 → early-stop fires; batch-1 abandoned by `break 'steps` (proven by test's 2-HTTP-request assertion) | 10 | 5+ (batch-1 abandoned) | 5 | Partial (5 < 10), is_last_batch=false | true (`!is_last_batch`): `(5 >= 10) \|\| true = false \|\| true = true` | false | true | Correct: remaining batches abandoned; data genuinely incomplete; matches `test_early_stop_multi_batch_partial_page_is_truncated` (RG-PSG-044) |

Example (c) is an accepted conservative corner: `is_truncated = true` even though the dataset may be
complete. An analyst who receives `is_truncated: true` with `total_available == returned_results`
should re-query without LIMIT to confirm completeness; the full-paginate query returns the same 1000
rows with `is_truncated: false`.

The `early_stopped` signal propagates to engine Step 6 where it contributes to the `is_truncated`
formula (§D8.9).

**Fan-out disambiguation — intra-pipeline step fan-out vs multi-sensor fan-out:** The `is_last_batch` term (`batch_idx + 1 == batch_count`) in the discriminator formula refers exclusively to **intra-pipeline step fan-out** within a single sensor's pipeline: `batch_idx` and `batch_count` are local to `execute_impl`'s batch-loop, which iterates over the step-level `fan_out_batches` / `fan_out_batch_size` mechanism for that one sensor. When `break 'steps` fires on a non-final batch, the remaining step-fan-out batches for that sensor are abandoned, so the sensor's accumulated data is genuinely incomplete — `early_stopped = true` is the correct truncation signal. This is DISTINCT from **multi-sensor fan-out** — the `FanOutResult.any_early_stopped` OR-aggregation that combines `PipelineResult.early_stopped` signals from independent sensor pipelines running in parallel across a multi-sensor query (§D8.9/§D8.10). The `is_last_batch` guard is purely intra-pipeline scope; it has no counterpart at the `FanOutResult` layer.

### D8.4 — Applicable pagination modes

LIMIT early-stop applies to both `PaginationConfig::OffsetLimit` and `PaginationConfig::CursorToken`
pagination modes.

For **OffsetLimit**, the partial-final-page discriminator (§D8.3) applies precisely:
`active_page_size = page_size` from the TOML declaration; a partial final page (fewer records
than the declared page size) correctly emits `early_stopped = false` because a partial offset
page is a genuine source-exhaustion signal — offset/limit APIs return a full page whenever more
data exists.

For **CursorToken** (ALL sub-cases, including `page_size: Some(ps)` and `page_size: None`),
`active_page_size = 0` → `page_record_count >= 0` is always `true` → `early_stopped = true`
(conservative over-report on all cursor early-stop exits).

**Rationale for CursorToken conservative treatment:** The page-fill discriminator is NOT a
valid cursor exhaustion signal. A cursor API may return a partial page (fewer records than the
declared `page_size`) while still providing a non-empty next cursor pointing to additional data.
Using `page_record_count < page_size` as an exhaustion signal for cursor mode produces the
DANGEROUS under-report direction: `early_stopped = false` → `is_truncated = false` → the MCP
consumer believes the dataset is complete when the source has more rows. The authoritative
exhaustion signal for cursor pagination is next-cursor absence, not page fill. The conservative
over-report (`early_stopped = true` always for cursor) is safe: an analyst receiving
`is_truncated: true` can re-query without LIMIT to confirm completeness.

**v1 scope:** No v1 sensor uses cursor pagination (all Claroty xDome tables declare
`type = "offset_limit"`). The conservative over-report has zero v1 behavioral impact.

**Precise cursor detection deferred:** Next-cursor-presence-based discriminator — when
early-stop fires in CursorToken mode, read the next cursor from the current page response
BEFORE breaking; if a non-empty next cursor exists → `early_stopped = true`; if no cursor →
`early_stopped = false` — is deferred to proposed story `S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001`
(post-v1; blocked on `S-OCSF-FIDELITY-CYBERINT-001`, the first cursor-pagination sensor
delivery). Implementer MUST NOT add a `CursorToken { page_size: Some(ps) } => ps as usize`
arm to the `active_page_size` derivation in `execute_impl` before `S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001`
ships; any such arm introduces the unsound under-report identified in F-FP1-LENSA-001.
**TD-VSDD-097 Dim-3 (DISCHARGED):** `S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001` exists and is
registered in STORY-INDEX (draft; blocked_by `S-OCSF-FIDELITY-CYBERINT-001`). The
prohibitive MUST above (do not add a `CursorToken { page_size: Some(ps) } => ps as usize` arm
before that story ships) is anchored to the registered draft story
`S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001`. TD-VSDD-097 Dim-3 mandate-anchor: DISCHARGED.

For **PaginationConfig::None** (single-page fetch), `active_page_size = 0` (falls into the `_ => 0` catch-all) → `page_record_count >= 0` is always `true` → `early_stopped = true` whenever the single page has `>= limit` records. A None-paginated table has no additional pages to stop early; the `break 'steps` performs no fetch-work reduction. The sole behavioral consequence is at the **exact-LIMIT corner** (single page holds exactly `LIMIT` rows): `is_truncated: true` is reported with `total_available == returned_results`. This is a safe over-report — the analyst can re-query without LIMIT to confirm completeness (same records returned, `is_truncated: false`). The corner is **narrow** (only at `row_count == LIMIT`; `row_count < LIMIT` never fires the check; `row_count > LIMIT` sets `is_truncated: true` via `total_rows > limit` regardless) and **latent for v1** (no v1 sensor uses `PaginationConfig::None` for a LIMIT-queryable table). Accepted as a documented conservative corner, consistent with the CursorToken conservative treatment and in the safe over-report direction.

The 10K DI-019 cap is not affected by D8 early-stop and remains unchanged; it fires before D8 when applicable.

### D8.5 — Sort-order and ORDER BY documentation

The LIMIT early-stop postcondition in BC-2.16.002 (and relevant table BCs) MUST include: "When
ORDER BY is combined with LIMIT in the absence of server-side sort support, the engine returns
the first N records in API-declared order, which may not be the globally sorted top N. Consumers
requiring globally sorted top-N MUST omit LIMIT or ensure the sensor API returns data in the
desired sort order."

### D8.6 — timeout_secs overlay wiring: deferred to story 3

The `timeout_secs` overlay field is accepted but emits `overlay.timeout_secs_ignored` (WARN in
`overlay.rs`). Wiring it to the reqwest client requires threading the overlay timeout through
`ResolvedSensorSpec` → caller → `FetchContext` (or creating a per-org client cache with the
configured timeout). This is architecturally independent of D8 and adds complexity to
`FetchContext` that would blur the D8 change. Deferring to a separate story
`S-ENGINE-TIMEOUT-OVERLAY-WIRE-001`. Architectural direction for that story: the caller
(`spec_driven_adapter.rs`) reads `resolved_spec.provenance.timeout_secs_from_overlay` and, when
`true`, constructs a fresh reqwest client via a variant of `build_http_client_with_custom_timeout`
parameterized by the overlay timeout. The PipelineExecutor receives the correctly-configured
client; no change to `FetchContext` needed.

### D8.7 — Plan-Shape Gate for Early-Stop Suppression

#### Problem

The MCP tool layer always sets `options.limit` to a non-zero value (default 25, user-supplied
otherwise). `run_materialization_pipeline` was deriving `fetch_limit` unconditionally from
`options.limit` without consulting the AST plan shape. This caused early-stop to fire for
reducing queries, curtailing the raw multi-page fetch BEFORE DataFusion applies the reducing
operator.

Concrete regressions (F-R11-CRIT-001): `SELECT COUNT(*) FROM claroty_vulnerabilities` returned
approximately one page worth of records instead of the true total; `GROUP BY severity` computed
group counts from a single page; queries with non-push-down WHERE predicates under-returned rows
after DataFusion filtered.

Round-12 fresh-context adversarial review found two additional reachable corruption paths
(F-R12-CRIT-001, F-R12-HIGH-001). The comprehensive audit documented in v1.3 enumerated every
grammar-expressible plan shape and found six additional gaps, documented below.

#### Out-of-Grammar Shapes (Not Gated — Confirmed by Code Inspection)

The following shapes are NOT expressible in PrismQL as of this ADR and therefore require no gate
condition. Each was explicitly checked against `ast.rs`, `sql_parser.rs`, and `pipe_parser.rs`:

| Shape | Expressible? | Notes |
|-------|-------------|-------|
| UNION / INTERSECT / EXCEPT | No | Not in grammar; no `SetOp` AST node |
| CTE (WITH clause) | No | Not in grammar; noted in `SqlPipeQuery` comment as S-3.06 future |
| FROM subquery / derived table | No | `FromClause.source` is `SourceRef`, not `SqlQuery` |
| OFFSET | No | `SqlQuery` has no `offset` field |
| Correlated subquery in FROM | No | Same as FROM subquery |

When any of the above are added to the grammar, the first implementation step MUST classify
them for the gate before they are reachable at runtime. The conservative default posture (see
below) ensures that unknown variants suppress early-stop as a safety net.

#### Complete Shape Classification Table

Every expressible plan shape classified as SUPPRESS (early-stop off) or PERMIT (early-stop on):

| Shape | Classification | Rationale / Condition |
|-------|---------------|----------------------|
| Bare projection: `SELECT * FROM t LIMIT N` | PERMIT | No reducing op; first N rows are semantically correct result |
| Projection + ORDER BY + LIMIT | PERMIT | §D8.5 accepted: returns rows in API-declared order, not globally sorted. ORDER BY does not change row count |
| Projection + temporal-only WHERE + LIMIT | PERMIT | Temporal pred fully server-side via ADR-033 T1; no client-side filter post-fetch |
| Projection + non-temporal equality WHERE + LIMIT | SUPPRESS | Condition G: client-side DataFusion filter; curtailing fetch under-returns rows |
| Projection + non-temporal non-equality WHERE (CONTAINS, BETWEEN, IN-list, CIDR, Regex, etc.) | SUPPRESS | Condition G revised: all non-temporal predicates are client-side |
| Filter mode with non-temporal predicate + LIMIT | SUPPRESS | Condition G revised: Filter mode was NOT covered by old `where_filters` check |
| Pipe where non-temporal predicate + LIMIT | SUPPRESS | Condition G revised: Pipe stages were NOT covered by old `where_filters` check |
| SQL aggregate in SELECT: `SELECT COUNT(*), MAX(x)` | SUPPRESS | Condition A: `FuncCall::Aggregate` in select items |
| SQL aggregate in ORDER BY: `SELECT * ORDER BY MAX(x)` | SUPPRESS | Condition A revised: aggregate in ORDER BY implicitly groups all rows; early-stop corrupts aggregate |
| GROUP BY (with or without visible aggregate) | SUPPRESS | Condition B: GROUP BY deduplicates and groups across full dataset |
| SELECT DISTINCT | SUPPRESS | Condition C: de-duplication requires full dataset scan |
| HAVING clause (any predicate) | SUPPRESS | Condition D: HAVING implies post-aggregation filtering; full aggregation input required |
| SQL JOIN (INNER/LEFT/RIGHT/FULL/CROSS) | SUPPRESS | Condition H: JOIN inputs independently fetched; early-stopping either input truncates join |
| SqlPipe head with SQL JOIN | SUPPRESS | Condition H: same reasoning as SQL JOIN |
| Pipe stats stage | SUPPRESS | Condition E: aggregation requires full dataset |
| Pipe dedup stage | SUPPRESS | Condition F: deduplication requires full dataset scan |
| Pipe tail stage | SUPPRESS | Condition I: selecting last N rows requires seeing all rows; early-stop severs the tail |
| Pipe join stage (currently errors; future-proofed) | SUPPRESS | Condition J: JOIN input truncation same as SQL JOIN |
| `FuncCall::Window {}` in SELECT/ORDER BY | SUPPRESS | Condition A revised: window functions compute over partitioned frames; requires full frame materialization |
| Aggregate nested inside scalar UDF arg: `severity_label(max(x))` | SUPPRESS | Condition A revised: recursion into `FuncCall::Scalar::args` required |
| InSubquery in WHERE: `WHERE f IN (SELECT ...)` | SUPPRESS | Condition G revised: IN-subquery check is client-side; early-stop under-returns matches |
| Pipe fields stage (column projection/exclusion) | PERMIT | Row count unchanged; projection is row-preserving |
| Pipe enrich stage | PERMIT | Enrichment adds columns per row; row count unchanged; applied post-fetch per row |
| Pipe sort stage | PERMIT | §D8.5: same reasoning as SQL ORDER BY; row count unchanged |
| Pipe head/limit stage | PERMIT | Explicit row limit; early-stop correctly bounds fetch |
| Pipe where temporal predicate | PERMIT | Temporal pred is server-side; no client-side filter |
| SQL DML (INSERT/UPDATE/DELETE) | SUPPRESS | Default posture: DML uses `write_pipeline.rs`, not `run_materialization_pipeline`; gate result is irrelevant but must safely return SUPPRESS for any path that reaches it |
| Unknown/future `Ast` variant | SUPPRESS | Conservative default: `_ => true` catch-all |
| Unknown/future `PipeStage` variant | SUPPRESS | Conservative default: stage loop falls through to SUPPRESS |
| Any queried source table with ≥2 Datetime+INDEX columns | SUPPRESS | Condition K: `count_temporal_bound_directions` does not track per-column direction; single-datetime-INDEX-per-table invariant enforced structurally |

#### Enforcement Site

`materialization.rs::run_materialization_pipeline` — at the `fetch_limit` derivation, BEFORE
fan-out targets are constructed. The gate is a guard on the single `fetch_limit` binding.

Subsystem scope: SS-11 (Query Execution) owns `run_materialization_pipeline` — the `fetch_limit`
derivation and `ast_is_reducing_plan` call site. SS-07 (Adapter Pagination & Response Cache) owns
`execute_impl` — the per-page early-stop check (§D8.2) — and the response-cache-key coherence
path where `fetch_limit` is the cache-key limit component (§D8.8).

Note: `where_filters` (the `FilterMap` from `extract_push_down_filters_as_map`) is no longer
passed to `ast_is_reducing_plan`. The gate performs its own AST inspection for client-side
predicate detection. `where_filters` continues to be computed and used for push-down and cache
key derivation; it is simply not forwarded to the gate.

#### Gate Function Signature

```
pub(crate) fn ast_is_reducing_plan(ast: &Ast) -> bool
```

The `where_filters: &FilterMap` parameter is absent; the gate performs its own AST walk via
`has_client_side_where`. (Decision history: `extract_push_down_filters_as_map` was never correct
for Filter-mode or Pipe-mode predicates — that function only processes `Ast::Sql` and
`Ast::SqlPipe` head WHERE; it returned an empty map for Filter and Pipe modes.)

#### Supporting Function: `expr_contains_aggregate_or_window`

This replaces the v1.2 `expr_contains_aggregate` function. The name change signals the expanded
scope: window functions are now also detected and cause suppression.

```
fn expr_contains_aggregate_or_window(expr: &Expr) -> bool
```

**Returns `true` for:**
- `Expr::FuncCall(FuncCall::Aggregate { .. })` — direct aggregate call
- `Expr::FuncCall(FuncCall::Window { .. })` — window function stub (S-3.06); full frame
  required regardless of field count in stub

**Recurses into sub-expressions for:**
- `Expr::FuncCall(FuncCall::Scalar { args, .. })` — recurse into every element of `args`
  (F-R12-CRIT-001 root cause: `severity_label(max(severity_id))` escaped detection because the
  outer `Scalar` was not recursed)
- `Expr::Compare { lhs, rhs, .. }` — recurse into both
- `Expr::Logical { lhs, rhs, .. }` — recurse into both
- `Expr::Not(inner)` — recurse
- `Expr::TimestampArithmetic { base, .. }` — recurse into `base`
- `Expr::InSubquery { .. }` — the subquery's SELECT is a separate `SqlQuery`; the outer `Expr`
  does not directly contain the subquery's aggregate. Return `false` here (the subquery's
  aggregation is independent of the outer plan's row count). The IN condition itself is caught
  by `has_client_side_where` via Condition G.

**Returns `false` (leaf, no recursion) for:**
`Expr::Literal`, `Expr::Field`, `Expr::VirtualField`, `Expr::Star`, `Expr::Now`,
`Expr::Interval`, `Expr::In { .. }` (literal values, no sub-expressions)

**Conservative catch-all:** `_ => true` for all unknown future `Expr` variants. Known
non-aggregate leaf variants (`Expr::Literal`, `Expr::Field`, `Expr::VirtualField`, `Expr::Star`,
`Expr::Now`, `Expr::Interval`, `Expr::In`) are enumerated explicitly returning `false`; unknown
or future `Expr` variants (e.g., a CASE expression) are treated as potentially-aggregate →
SUPPRESS. This extends the conservative-default posture to the Expr-recursion level: the
terminal arm MUST be `_ => true`, NOT `_ => false`. The prior v1.3 description erroneously
stated `_ => false` (leaf assumption); v1.5 corrects this per F-R14-LOW-001. For `FuncCall`
variants specifically, the catch-all is also `_ => true` (unknown function call types may be
aggregates; conservative suppression preferred over a false PERMIT). (Anchored:
S-ENGINE-LIMIT-EARLY-STOP-001 AC-007; correctness enforced by exhaustive explicit enumeration
of all known non-aggregate leaf Exprs returning `false` — any unlisted variant hits `_ => true`.)

#### Supporting Function: `has_client_side_where`

```
fn has_client_side_where(ast: &Ast, datetime_index_cols: &[&str]) -> bool
```

Returns `true` iff any WHERE-position predicate in the AST will be applied client-side by
DataFusion after fetching (i.e., is NOT guaranteed to be fully resolved server-side).

Only temporal range predicates on INDEX datetime columns with concrete `Literal::Timestamp` RHS,
as determined by `is_pushed_temporal_predicate(pred, datetime_index_cols)` (§D8.9), are
guaranteed server-side for `Ast::Sql` and `Ast::SqlPipe` head WHERE. **`Ast::Filter` predicates
and `Ast::Pipe / Ast::SqlPipe` pipe-stage WHERE predicates are ALWAYS client-side regardless of
predicate form** (unconditional suppression; see arm descriptions below). All other
predicate forms — equality comparisons, IN lists, `InSubquery`, CONTAINS/STARTSWITH/ENDSWITH
(StringOp), BETWEEN, CIDR, Regex, Has, Missing, IsNull, Wildcard, and any logical combinations
— are client-side.

**AST-mode dispatch:**

- `Ast::Filter(f)`: returns `true` UNCONDITIONALLY for all filter-mode predicates, including
  purely temporal ones. `extract_time_bounds_from_predicate` (ADR-033 T1) does NOT process
  `Ast::Filter` mode — temporal predicates in filter-mode queries are evaluated client-side by
  DataFusion after the full fetch, not server-side. The v1.3 `!is_purely_temporal_predicate`
  check for this arm was UNSOUND and is removed in v1.5; closes F-R15-LENSA-CRIT-001
  (filter-mode path). Note: the v1.2 `where_filters` approach was also INCORRECT for this mode —
  `extract_push_down_filters_as_map` always returned an empty map for `Ast::Filter`.

- `Ast::Sql(SqlStatement::Select(sql))`: returns
  `sql.where_.as_ref().map(|p| !is_pushed_temporal_predicate(p, datetime_index_cols)).unwrap_or(false)`.
  When the WHERE clause is absent, returns `false` (no client-side filter). When present,
  PERMIT (`false`) only if `is_pushed_temporal_predicate` determines the whole predicate is
  a fully server-side temporal range on an INDEX datetime column with concrete `Literal::Timestamp`
  RHS. Note: the v1.2 `where_filters` approach was correct for the equality-predicate sub-case
  but missed non-equality client-side predicates (CONTAINS, BETWEEN, etc.).

- `Ast::Pipe(pipe)`: returns `true` UNCONDITIONALLY whenever any `PipeStage::Where(_)` is
  present in `pipe.stages`, regardless of predicate form. Pipe `| where` stages push NOTHING
  server-side; `PipeStage::Where` is not in the PERMIT allow-list. The v1.3
  `!is_purely_temporal_predicate(pred)` check for this arm was UNSOUND because `Ast::Pipe`
  predicates are never resolved server-side by `extract_time_bounds_from_predicate`; closes
  F-R15-LENSA-CRIT-001 (pipe-mode path). Note: the v1.2 `where_filters` approach was also
  INCORRECT — `extract_push_down_filters_as_map` always returned an empty map for `Ast::Pipe`.

- `Ast::SqlPipe(spq)`: returns `true` iff (`spq.head.where_` is present AND
  `!is_pushed_temporal_predicate(where_pred, datetime_index_cols)`) OR any `PipeStage::Where(_)`
  is present in `spq.stages` (pipe-WHERE stages are unconditionally suppressed; see `Ast::Pipe`
  arm rationale above).

- `Ast::Sql(SqlStatement::Dml(_))` and `_ =>`: returns `false` (DML does not use this
  pipeline; unknown variants handled by the outer gate's `_ => true` catch-all).

**`is_pushed_temporal_predicate(pred: &Predicate, datetime_index_cols: &[&str]) -> bool`:**
Returns `true` (PERMIT early-stop for the calling WHERE clause) iff the predicate is fully
handled server-side by the ADR-033 T1 temporal push-down mechanism. Mirrors
`extract_time_bounds_from_predicate` exactly. Replaces the v1.3 `is_purely_temporal_predicate`
which unsoundly permitted `Expr::Now`, `Expr::Interval`, `Expr::TimestampArithmetic` (relative
time expressions evaluated post-fetch) and non-INDEX datetime columns.

**Returns `true` (PERMIT) iff ALL THREE preconditions hold:**
1. **Range operator:** `Gt | Ge | Lt | Le` — NOT `Eq` or `Ne`. Temporal equality predicates
   (`timestamp = X`) are not extractable by `extract_time_bounds_from_predicate` and remain
   client-side.
2. **LHS is an INDEX datetime column:** `Expr::Field(name)` where `name` appears in
   `datetime_index_cols` (columns declared `index: true` + `column_type = "Datetime"` in sensor
   TOML). Non-INDEX datetime columns are not pushed server-side.
3. **RHS is a concrete absolute timestamp:** `Expr::Literal(Literal::Timestamp)`. Relative
   expressions — `Expr::Now`, `Expr::Interval`, `Expr::TimestampArithmetic` — are evaluated
   by DataFusion after fetch, not by the server.

**Reversed-operand form (`Literal::Timestamp OP Field`) returns `false` (SUPPRESS):** A predicate where a `Literal::Timestamp` appears on the LHS and a `Field` appears on the RHS is NOT permitted. `extract_time_bounds_from_predicate` (ADR-033 T1) only processes predicates where `lhs` is `Expr::Field`; the reversed form is not extracted server-side and remains client-side. The gate MUST mirror the extractor: only Field-on-LHS predicates receive PERMIT. This form is grammar-unreachable in the PrismQL parser today (latent), but any code path permitting reversed operands is unsound in the PERMIT direction — it would classify a DataFusion-client-side filter as server-pushed, enabling early-stop against an unfiltered server result set. (F-R16-P16-LENSA-LOW-001)

**`Predicate::Logical { op: AND, predicates }`:** Two-step check:

**Step 1 — all leaves individually pushed:** `predicates.iter().all(|p| is_pushed_temporal_predicate(p, datetime_index_cols))` must return `true`. Catches non-range operators, non-INDEX datetime columns, and relative-expression RHS in any leaf.

**Step 2 — first-wins direction-count constraint:** A private helper `count_temporal_bound_directions(predicates, &mut lower, &mut upper)` recursively counts Gt/Ge leaves as lower-direction and Lt/Le leaves as upper-direction across the flattened AND tree. Returns `true` (PERMIT) only when `lower <= 1 && upper <= 1`.

**Rationale:** `extract_time_bounds_from_predicate` uses first-wins semantics: the first Gt/Ge on any INDEX datetime column sets `start_time`; the first Lt/Le sets `end_time`; any subsequent same-direction bound is silently skipped server-side and applied by DataFusion client-side post-fetch. A predicate tree with two Gt/Ge bounds (e.g., `col > X AND col > Y`) passes Step 1 (each leaf individually satisfies the three preconditions) but fails Step 2 — `lower = 2 > 1` → SUPPRESS. Without Step 2, such a tree was incorrectly classified PERMIT → server fetches page 1 filtered by the first bound; DataFusion applies the second bound; rows satisfying only the first but not the second produce zero hits, though matching rows exist on unfetched pages (silent LIMIT under-return). **`count_temporal_bound_directions` MUST NOT be inlined into `is_pushed_temporal_predicate`'s own recursion; it is a standalone private helper in `materialization.rs` that walks only the AND-tree spine**, to avoid double-counting from nested recursion through individual leaf checks.

**Canonical PERMIT case:** `timestamp >= X AND timestamp < Y` → lower=1, upper=1 → Step 1 passes, Step 2 passes → PERMIT. **Canonical SUPPRESS case:** `timestamp > X AND timestamp > Y` → lower=2 > 1 → SUPPRESS. **Multi-column case:** `created_at >= X AND updated_at < Y` (two different INDEX datetime cols) → lower=1, upper=1 → PERMIT (extract_time_bounds_from_predicate extracts start from created_at and end from updated_at; both fully consumed).

**All other predicates return `false` (SUPPRESS):**
- Temporal equality (`Eq` operator): not range-extractable
- `Expr::Now`, `Expr::Interval`, `Expr::TimestampArithmetic`: relative; evaluated post-fetch
- LHS field not in `datetime_index_cols`: non-INDEX columns not pushed server-side
- `Predicate::Logical { op: OR, .. }`: OR-combined predicates not handled by
  `extract_time_bounds_from_predicate`; conservative suppression
- Any other predicate form: conservative suppression

#### Complete Condition Set

`ast_is_reducing_plan(ast: &Ast) -> bool` returns `true` (SUPPRESS) when ANY of the following
holds:

**Condition A — Aggregation or window function in SELECT items or ORDER BY expressions
(closes F-R12-CRIT-001):**
- Any `SelectItem::Expr { expr, .. }` in `select.items` (SQL) or `head.select.items` (SqlPipe)
  where `expr_contains_aggregate_or_window(expr)` returns `true`.
- Any `OrderExpr { expr, .. }` in `order_by` (SQL or SqlPipe head) where
  `expr_contains_aggregate_or_window(expr)` returns `true`.
  Rationale: `ORDER BY MAX(severity)` without GROUP BY performs a global aggregation; early-stop
  corrupts the result. ORDER BY uses the same `Expr` parser as SELECT, so aggregate calls ARE
  parseable in ORDER BY position.
- Applies to: `Ast::Sql(SqlStatement::Select(sql))` and `Ast::SqlPipe(spq)` (head).

**Condition B — GROUP BY:**
`sql.group_by.is_empty() == false` (or `spq.head.group_by`). GROUP BY groups across the full
dataset; early-stop yields incorrect group membership and counts.

**Condition C — DISTINCT:**
`sql.select.distinct == true` (or `spq.head.select.distinct`). De-duplication requires a full
dataset scan; early-stop produces false-unique results.

**Condition D — HAVING:**
`sql.having.is_some()` (or `spq.head.having`). HAVING always implies post-aggregation filtering.
Conservatively suppresses even `HAVING non_agg_expr` (which is unusual SQL), because HAVING is
semantically coupled to GROUP BY / aggregation and full-dataset evaluation.

**Condition E — Pipe Stats stage:**
Any `PipeStage::Stats(_)` in `pipe.stages` or `spq.stages`. Aggregation requires full dataset.

**Condition F — Pipe Dedup stage:**
Any `PipeStage::Dedup(_)` in `pipe.stages` or `spq.stages`. Deduplication requires full dataset.

**Condition G — Client-side WHERE predicate:**
`has_client_side_where(ast)` returns `true`. This replaces the insufficient
`!where_filters.is_empty()` check. The old check failed for three cases: (1) `Ast::Filter` mode
(always returned empty `where_filters`), (2) `Ast::Pipe` stages (always returned empty
`where_filters`), and (3) non-equality SQL WHERE predicates (CONTAINS, BETWEEN, IN-list, CIDR,
Regex, Has, Missing, etc.) that are client-side DataFusion filters but not equality predicates.
The new check directly inspects the AST predicate form.

ADR-033 cross-reference: temporal range predicates (extracted by
`extract_time_window_from_ast_from_query`) are fully server-side via T1 push-down and do not
suppress early-stop. All other predicate forms are treated as client-side until Wave 5
per-sensor push-down classification (ADR-033 extension) enables the engine to classify
individual equality predicates as server-side.

**Condition H — SQL JOIN (new; closes F-R12-HIGH-001):**
`!sql.joins.is_empty()` (or `!spq.head.joins.is_empty()`). Each table in a JOIN is an
independent fan-out target fetched separately. Early-stopping the raw fetch from any source
truncates that input before DataFusion applies the JOIN. The resulting joined set is computed
over incomplete inputs — missing rows from the truncated side that would have matched are
silently absent.
Applies to all JOIN kinds (`Inner`, `Left`, `Right`, `FullOuter`, `Cross`).

**Condition I — Pipe Tail stage (new):**
Any `PipeStage::Tail(_)` in `pipe.stages` or `spq.stages`. `| tail N` semantically selects the
LAST N rows of the dataset. Early-stop fetches only the first few pages, making the last N rows
of the fetched subset the last N of a truncated dataset — not the true tail of the full dataset.
Note: the current `pipe_sql_emitter.rs` lowers `PipeStage::Tail(N)` to `LIMIT N` (known semantic
gap §3.2). Under the lowered form, suppression is technically immaterial for the current behavior
(LIMIT N on partial or full dataset both give the first N rows). Condition I is specified for
correctness when Tail is properly implemented.

**Condition J — Pipe Join stage (new; closes F-R12-HIGH-001 for pipe mode):**
Any `PipeStage::Join(_)` in `pipe.stages` or `spq.stages`. Same reasoning as Condition H.
Note: `pipe_sql_emitter.rs` currently returns an error for `PipeStage::Join` (not yet supported,
ENRICH-4-C), so this condition is defensive and future-proof. When Pipe Join is implemented, the
gate MUST already suppress early-stop for it.

**Condition K — Multi-INDEX-datetime conservative suppression (F-R16-P16-LENSB-LOW-001):**
When `collect_datetime_index_cols` (§D8.9) returns `suppress_multi_index = true` — at least one
source table in the queried `source_names` exposes ≥2 Datetime+INDEX columns — `fetch_limit = 0`
(SUPPRESS) regardless of other gate results.

Rationale: `count_temporal_bound_directions` (§D8.7 AND-arm Step 2) counts Gt/Ge and Lt/Le
leaves GLOBALLY across the AND-tree, not per-column. For a table with two INDEX Datetime columns
`col_a` and `col_b`, `WHERE col_a > X AND col_b < Y` yields `(lower=1, upper=1)` → Step 2
PERMIT. But `extract_time_bounds_from_predicate` first-wins takes start from `col_a` and end
from `col_b`, fabricating a mixed-column time window that the server may not correctly apply
to both columns independently. No current shipped sensor TOML has two Datetime+INDEX columns
per table (the single-datetime-INDEX-per-table invariant holds throughout all Wave 3+ sensors);
Condition K never fires in production today and costs zero performance. It prevents an incorrect
PERMIT if a future TOML violates this invariant.

`collect_datetime_index_cols` tracks per-source Datetime+INDEX column counts while building
the column name set (§D8.9). If any source in `resolved_col_map` contributes ≥2 such columns,
the helper sets `suppress_multi_index = true` in its return tuple. The call site in
`run_materialization_pipeline` checks this flag and short-circuits to `fetch_limit = 0`.

#### Conservative Default Posture

The `Ast` enum, `PipeStage` enum, and `FuncCall` enum are all `#[non_exhaustive]`. New variants
may be added without a compile error in the gate's `match` arm. The default posture is:

**SUPPRESS (return `true`) for any unknown or unclassified variant.**

Implementation:
- `ast_is_reducing_plan`: the outer `match ast { ... _ => true }` arm suppresses for any future
  `Ast` variant not explicitly listed.
- Pipe-stage scan loop: unknown `PipeStage` variants trigger suppression via `_ => return true`
  (or an allowlist of known-safe stages: `PipeStage::Where`, `PipeStage::Sort`,
  `PipeStage::Limit`, `PipeStage::Fields`, `PipeStage::Enrich` — any stage not in this list
  suppresses early-stop).
- `FuncCall` catch-all in `expr_contains_aggregate_or_window`: `FuncCall::_unknown => true`
  (unknown function calls may be aggregates; suppress conservatively).

**Rationale for conservative default (allowlist over denylist):** The cost of incorrect
SUPPRESS is degraded performance (full pagination instead of early-stop). The cost of incorrect
PERMIT is a silent correctness regression: `truncated=false` results computed over a partial
dataset with no signal to the consumer. Given the asymmetric cost, the gate defaults to SUPPRESS
for all uncertainty. Only shapes explicitly proven safe receive PERMIT.

#### ORDER BY Does NOT Suppress Early-Stop (§D8.5 Preserved — Unchanged)

`PipeStage::Sort` and `SqlQuery::order_by` alone (without an aggregate in the ORDER BY
expression) are NOT suppression conditions. This preserves the §D8.5 accepted limitation.

A bare projection with ORDER BY + LIMIT returns records in API-declared order within the fetched
subset, which may not be the globally sorted top N. This is the §D8.5 accepted trade-off.

See §Alternatives Alt-D for the rejected alternative of suppressing early-stop for ORDER BY.

IMPORTANT: `ORDER BY aggregate_fn(col)` WITHOUT GROUP BY DOES suppress early-stop via
Condition A (aggregate in ORDER BY). The non-suppression applies only to ORDER BY expressions
that contain no aggregate or window function.

#### Gate Application in `run_materialization_pipeline`

```
// Plan-shape gate (ADR-060 §D8.7): suppress early-stop for reducing plans.
// Note: where_filters is NOT passed — gate performs its own AST inspection.
let fetch_limit: u64 = if ast_is_reducing_plan(&ast) {
    0 // suppress: reducing plan needs full pagination for correctness
} else {
    options.limit.map(|l| l as u64).unwrap_or(0)
};
```

The `0` sentinel flows unchanged through the existing pipeline:
- `QueryParams.limit = 0` → `FetchContext::early_stop_limit = None` (per existing
  `if params.limit == 0 { None }` mapping in `spec_driven_adapter.rs`)
- `FetchContext::early_stop_limit = None` → early-stop check in `execute_impl` does not fire
- Full pagination proceeds up to the DI-019 10K cap, as before this story

**§D8.7 POL-39 decision-history convention:** "v1.N" references in the arm descriptions above (e.g., "was UNSOUND and is removed in v1.5", "replaces the v1.2 check", "was INCORRECT for this mode") are intentional intra-ADR decision-history prose documenting the gate's design evolution — they are NOT normative version pins and are POL-39-exempt; do not re-mint findings against them.

### D8.8 — Single-Binding Coherence with Plan-Shape Gate

The existing SINGLE-BINDING COHERENCE invariant (comment in `run_materialization_pipeline`:
"this binding feeds BOTH the response-cache key derivation AND the fan-out target construction")
is preserved by the plan-shape gate.

When `fetch_limit = 0` (reducing plan), the response-cache key uses 0 as the limit component.
This means all reducing-plan queries with the same filters and time window share a cache entry
that holds the full dataset (fetched without early-stop). A `SELECT COUNT(*)` and a
`SELECT COUNT(*) | LIMIT 25` both receive `fetch_limit = 0` and share a cache entry — correct,
since both need the full dataset.

When `fetch_limit = N > 0` (non-reducing plan, gate permits early-stop), the cache key uses N.
Different LIMIT values produce different cache entries — correct, since `LIMIT 10` and `LIMIT 100`
may stop at different pages.

The v1.3 signature change (removal of `where_filters` parameter from `ast_is_reducing_plan`)
does not affect coherence. `where_filters` continues to be computed and used in the cache key
derivation; it is no longer forwarded to the gate function. The single `fetch_limit` binding
remains the sole source feeding both cache key and `QueryParams.limit`.

### D8.9 — `any_early_stopped` Truncation-Signal Propagation Chain and `datetime_index_cols` Threading

#### Motivation

When the §D8.2 early-stop `break 'steps` fires at the exact-limit boundary
(`all_records.len() == limit`, so `total_rows == limit`), the naive formula `total_rows > limit`
evaluates to `false`. The partial-final-page discriminator (§D8.3) provides the additional signal:

- When the final page was **full** (`page_record_count >= page_size`): more pages may exist →
  `early_stopped = true` → `is_truncated = true` (correct: data may be incomplete).
- When the final page was **partial** (`page_record_count < page_size`): the source is exhausted →
  `early_stopped = false` → `is_truncated = false` (correct: the complete dataset was returned).

Without this discriminator, a LIMIT query whose row-count exactly matches the tenant dataset size
(partial final page) emits `is_truncated: true` alongside `total_available == returned_results` — a
self-contradictory signal to the MCP consumer. The discriminator resolves the contradiction.

#### `PipelineResult.early_stopped: bool`

When the §D8.2 `break 'steps` fires (early-stop, NOT DI-019), `PipelineResult.early_stopped` is set
using the partial-final-page discriminator (§D8.3): `true` when the final page was full
(`page_record_count >= page_size`, more pages may exist); `false` when the final page was partial
(`page_record_count < page_size`, source exhausted). This field is DISTINCT from `truncated`:
`truncated` signals DI-019 capacity overflow (§D8.3 invariant: implementer MUST NOT set `truncated`
on early-stop); `early_stopped = true` signals a query-driven early exit where more pages may exist.

#### `FetchOutput` Return Type

`SensorAdapter::fetch` return type changes to carry the early-stop signal out of the per-sensor
pipeline and into the fan-out layer:

```rust
pub struct FetchOutput {
    pub batches: Vec<RecordBatch>,
    pub any_early_stopped: bool,
    pub pipeline_truncated: bool,
}
```

`any_early_stopped` is set from `PipelineResult.early_stopped` of the sensor's pipeline
execution. `pipeline_truncated` is set from `PipelineResult.truncated` (DI-019 capacity cap;
see §D8.10 for the full chain).

#### Propagation Chain

Both truncation signals propagate from the per-sensor level to engine.rs Step 6:

```
any_early_stopped chain (query-driven early exit):
  PipelineResult.early_stopped
    → FetchOutput.any_early_stopped
    → FanOutResult.any_early_stopped   (OR-combined across all sensors in the fan-out)
    → MaterializationOutput.any_early_stopped

pipeline_truncated chain (DI-019 capacity cap — §D8.10):
  PipelineResult.truncated
    → FetchOutput.pipeline_truncated
    → FanOutResult.any_pipeline_truncated   (OR-combined across all sensors in the fan-out)
    → MaterializationOutput.any_pipeline_truncated

Both signals feed engine.rs Step 6:
  is_truncated = (total_rows > limit) || any_early_stopped || any_pipeline_truncated
```

#### `is_truncated` Formula at Step 6 (BC-2.11.001 EC-11-092; updated §D8.10)

```rust
let is_truncated = total_rows > limit || materialization_output.any_early_stopped
    || materialization_output.any_pipeline_truncated;
```

When `total_rows == limit` (exact-limit boundary) AND final page was FULL (`any_early_stopped = true`,
per §D8.3 discriminator):
- `total_rows > limit` = false
- `any_early_stopped` = true
- Result: `is_truncated = true` — correctly signals that more data may exist (full-page boundary;
  conservative: exhaustion unconfirmed without fetching next page).

When `total_rows == limit` (exact-limit boundary) AND final page was PARTIAL (`any_early_stopped = false`,
per §D8.3 partial-final-page discriminator):
- `total_rows > limit` = false
- `any_early_stopped` = false (discriminator: partial page → source exhausted)
- `any_pipeline_truncated` = false
- Result: `is_truncated = false` — correctly signals that the complete dataset was returned;
  `total_available == returned_results` is no longer self-contradictory (closes F-P31-LENSA-OBS-001).

When DI-019 cap fires (no LIMIT query) AND `any_pipeline_truncated = true`:
- `total_rows > limit` = false (limit = usize::MAX when no LIMIT specified)
- `any_early_stopped` = false (DI-019 path is NOT the early-stop path)
- `any_pipeline_truncated` = true
- Result: `is_truncated = true` — correctly surfaces DI-019 truncation at the analyst level.

`total_available` is a LOWER BOUND when `any_early_stopped = true` OR `any_pipeline_truncated = true`:
the true dataset size is unknown because pagination was halted before exhaustion. When both signals
are `false` (either no early-stop fired, or early-stop fired on a partial final page per the §D8.3
discriminator), `total_available` is exact — the source was fully exhausted. See §D8.10 for the full
DI-019 truncation-signal chain.

#### Step 6 is the SOLE Owner of Tool-Level Cap (BC-2.11.001 EC-11-093)

`run_materialization_pipeline` MUST return the full filtered/aggregated result set to engine.rs
Step 6 WITHOUT applying a tool-level pre-cap. Engine.rs Step 6 reads the full pre-cap row count
from the materialization output, computes `total_available`, sets `is_truncated`, and then
applies the cap. A `truncate_result_to_limit` pre-cap inside `run_materialization_pipeline`
causes Step 6 to see the pre-capped count as `total_available`, silently producing
`is_truncated: false` when the unfiltered count exceeds the tool limit (F-R13-CRIT-001
prohibited behavior).

The `fetch_limit` binding controls ONLY the early-stop check in the pagination loop; it does
NOT authorize `run_materialization_pipeline` to cap the result set returned to Step 6.
(Anchored: RG-PSG-025 `test_psg_exact_limit_is_truncated_true`)

#### `datetime_index_cols` Threading

`has_client_side_where(ast, datetime_index_cols)` (§D8.7) receives `datetime_index_cols` from
`run_materialization_pipeline`. The caller derives `datetime_index_cols` from the resolved
sensor spec. Construction MUST mirror `extract_time_window_from_ast`'s `datetime_index_cols` set
(ADR-033 T1 / ADR-058 §I6) — both functions determine which column names receive server-side
temporal push-down; they must agree on the set.

**Construction rules:** Use the source-scoped `resolved_col_map` built by
`build_source_column_map(spec_map, source_names)` — already computed earlier in
`run_materialization_pipeline` for the push-down path. Do NOT iterate
`resolved_spec_map.values()`, which spans all sensors regardless of the current query's
`source_names`. For each column in `resolved_col_map[source_name]` for each queried
`source_name`, where the column is declared `column_type = "Datetime"` AND
`options.contains(Index)`:
1. Insert `col.name` (always).
2. When `ocsf_column_naming = true` for this source AND `col.ocsf_field` is `Some(field)`:
   also insert `ocsf_field_to_arrow_name(field)` (e.g. `"time"` for claroty.audit_logs
   `col.name = "timestamp"`, `ocsf_field = "time"`).

The per-source `ocsf_column_naming` flag is read from `ocsf_naming_map` keyed by the
`"{sensor_id}.{table_name}"` dot-separated source name. Deduplicate insertions via a `HashSet`
or `seen: HashSet<String>` guard (`resolved_col_map` stores both dot-separated and
underscore-separated key forms pointing to the same column list; iterating all keys without
deduplication would double-insert column names).

**Structural reuse — `collect_datetime_index_cols` shared helper (F-R16-P16-LENSA-HIGH-001):**
Introduce a private helper in `materialization.rs`:

```
fn collect_datetime_index_cols(
    col_map: &HashMap<String, Vec<ColumnSpec>>,
    ocsf_naming_map: Option<&HashMap<String, bool>>,
) -> (Vec<String>, bool /* suppress_multi_index */)
```

This helper encapsulates the construction rules above and is called from TWO sites:
1. **Gate** (`run_materialization_pipeline`): provides the `datetime_index_cols` argument
   for `has_client_side_where(ast, &datetime_index_cols)`.
2. **Push-down** (`extract_time_window_from_ast`, pushdown.rs): replaces the inline
   `datetime_index_cols: HashSet<String>` construction that currently mirrors this logic
   independently.

Sharing the helper eliminates the possibility of independent re-implementations drifting apart
— the root-cause structural defect class that produced F-R16-P16-LENSA-HIGH-001 (two
implementations that diverged on source-scoping) and the predecessor finding it corrects.

The second return value `suppress_multi_index` is `true` when any source in `col_map`
contributes ≥2 Datetime+INDEX column names (before deduplication within that source). The
call site checks this flag and applies Condition K (§D8.7).

**Why source-scoping matters (F-R16-P16-LENSA-HIGH-001):** Prior to v1.8, construction
iterated `resolved_spec_map.values()` without filtering to the queried `source_names`. Example:
Armis `devices.last_seen` is declared `options = ["INDEX"]` (required for AQL time-window
augmentation per ADR-033 T1); CrowdStrike `devices.last_seen` is NOT (`column_type = "datetime"`
only, no `options`). A query `SELECT * FROM crowdstrike_devices WHERE last_seen > '...' LIMIT 100`
caused the gate to find `"last_seen"` in the (unscoped) `datetime_index_cols` — contributed by
Armis — and PERMIT early-stop. The push-down extractor (`extract_time_window_from_ast`, always
source-scoped) correctly excluded CrowdStrike `last_seen` from push-down. DataFusion applied the
`WHERE last_seen > '...'` filter client-side against an early-stopped page set → silent under-return.

**OCSF-name gap:** `extract_time_window_from_ast` registers the OCSF Arrow
name `"time"` when `ocsf_column_naming = true` (ADR-058 §I6). A query `WHERE time > '...' LIMIT n`
on claroty.audit_logs is therefore pushed server-side. The construction rules above include this
registration; `collect_datetime_index_cols` implements it for both the gate and push-down sites.

The parameter is passed through to `is_pushed_temporal_predicate` at the predicate inspection level.

### D8.10 — DI-019 Truncation-Signal Propagation Chain (F-R16-P18-LENSA-MED-001)

#### Problem

`PipelineResult.truncated` (the DI-019 10K cap signal set in `execute_impl`) was silently dropped at the adapter boundary. `spec_driven_adapter.rs::fetch` called `FetchOutput::new(all_batches, any_early_stopped)` without reading `result.truncated`. A single-sensor fetch that hit the DI-019 cap produced:
- `PipelineResult.truncated = true, early_stopped = false`
- `FetchOutput.any_early_stopped = false` (truncated is NOT early_stopped)
- `fan_result.any_early_stopped = false`
- Cache-completeness gate: `errors.is_empty() && !false = true` → cached as COMPLETE
- Subsequent cache hit: served with `is_truncated = false` in the MCP response

This is the same "partial served as complete" class as EC-01-039/RG-PSG-034, on the DI-019 sibling path. DI-019 truncation is a CAPACITY condition (not a query-driven early exit), but it must be treated as incomplete for cache purposes — the dataset is larger than 10K but only 10K rows were returned.

#### New Field Chain

Thread a DI-019 truncation signal along the SAME structural chain that `any_early_stopped` uses. The naming convention mirrors `any_early_stopped` at each layer:

```
PipelineResult.truncated                       (existing; prism-spec-engine::pipeline.rs)
  → FetchOutput.pipeline_truncated: bool       (new; prism-sensors::adapter.rs)
  → FanOutResult.any_pipeline_truncated: bool  (new; prism-sensors::fanout.rs — OR-aggregate)
  → MaterializationOutput.any_pipeline_truncated: bool  (new; prism-query::materialization.rs)
  → cache-completeness gate (materialization.rs)
  → engine.rs analyst Step 6 is_truncated formula
  → engine.rs scheduled path is_truncated field
```

#### Field Naming Rationale

`pipeline_truncated` (not `truncated`) at the `FetchOutput` level: the word `truncated` alone is ambiguous (could mean result trimming). The prefix `pipeline_` signals the DI-019 capacity condition originating inside `PipelineExecutor`. At the `FanOutResult` and `MaterializationOutput` levels, `any_pipeline_truncated` mirrors the `any_early_stopped` convention (OR-aggregate: true when at least one sensor hit the cap).

#### FetchOutput Constructor Change

`FetchOutput::new(batches, any_early_stopped)` gains a third parameter:

```rust
pub fn new(batches: Vec<RecordBatch>, any_early_stopped: bool, pipeline_truncated: bool) -> Self
```

All construction sites not modeling DI-019 truncation MUST pass `false` for `pipeline_truncated`. The single production site in `spec_driven_adapter.rs::fetch` passes the OR-aggregate of `result.truncated` across all executed tables. The struct literal in `prism-sensors::fanout.rs` (internal crate construction — `#[non_exhaustive]` does not restrict within the defining crate) must add `pipeline_truncated: false`.

#### FanOutResult Field Addition

`FanOutResult` derives `Default`; adding `any_pipeline_truncated: bool` initializes it to `false` automatically. The `fan_out` accumulation loop gains:

```rust
result.any_pipeline_truncated |= fetch_output.pipeline_truncated;
```

The `fan_out_with_overlay_map` function has one `FetchOutput` struct literal (internal crate construction); it must add `pipeline_truncated: false`.

#### Cache-Completeness Gate Update

The gate in `run_materialization_pipeline` (ADR-060 §D8.3) must guard against BOTH incomplete signals:

```rust
let complete = fan_result.errors.is_empty()
    && !fan_result.any_early_stopped
    && !fan_result.any_pipeline_truncated;
```

DI-019-capped results are DISCARDED (not stored) by the cache layer; the subsequent identical-key query fetches fresh from the sensor API. The cache-poisoning vector is the same as for early-stopped results: a 10K-truncated result has no `errors` and would otherwise be stored as complete.

Governing cacheability contract: BC-2.07.003 §Postconditions. EC-01-040 added to BC-2.16.002 to document this gate (owned by this file where `any_pipeline_truncated` is defined). Anchor: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-035 (`test_psg_rg035_di019_truncated_response_not_cached_as_complete`).

#### Engine Step 6 — Analyst Path (Updated §D8.9 Formula)

The `is_truncated` formula at engine.rs Step 6 must reflect BOTH incomplete-signal sources:

```rust
let is_truncated = total_rows > limit
    || output.any_early_stopped
    || output.any_pipeline_truncated;
```

When `any_pipeline_truncated = true` and `any_early_stopped = false` (pure DI-019 path, no LIMIT query):
- `total_rows > limit` = false (limit = `usize::MAX` when no LIMIT specified)
- `any_pipeline_truncated` = true
- Result: `is_truncated = true` — correctly surfaces DI-019 truncation at the MCP layer.

`total_available` is a LOWER BOUND when `any_pipeline_truncated = true`: the dataset is larger than the 10K cap but the true size is unknown. This is the same lower-bound semantics as `any_early_stopped = true`.

Anchor: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-036 (`test_psg_rg036_di019_truncated_step6_is_truncated_true`).

#### Engine Scheduled Path — Hardcode Replacement (F-R16-P18-LENSA-OBS-001)

`execute_scheduled_inner` (engine.rs) hardcodes `is_truncated: false` in its `QueryResult` assembly block. This is a latent correctness trap: scheduled queries have `early_stop_limit = None` (no LIMIT clause → `any_early_stopped` is always `false`), but DI-019 CAN fire on large scheduled scans. If a detection-engine query runs against a DI-019-truncated dataset, the scheduled `QueryResult` must report `is_truncated: true` so the caller knows the result is incomplete.

Replace the hardcode with:

```rust
is_truncated: output.any_early_stopped || output.any_pipeline_truncated,
```

The `|| any_early_stopped` term is included for future robustness; it is always `false` on the current scheduled path. No `total_rows > limit` term is needed because scheduled queries use `limit = usize::MAX` and `total_rows <= 10_000` (DI-019 capped), so `total_rows > limit` is never true.

**Classification:** This finding is MED (latent — currently the scheduled path has `effective_options.limit = None` so DI-019 is the only truncation source; the hardcode produces `is_truncated: false` even when the detection engine sees a truncated dataset). The production-grade fix is to consume the new flag in the same burst as the analyst path.

---

## Rationale

**Why stop at COMPLETE page boundaries:** Stopping mid-page would violate the atomicity
guarantee (partially-received page → partial records, potential schema mismatch). Stopping only
at complete-page boundaries preserves the invariant that every record in `all_records` was
fully received and parsed.

**Why DataFusion applies precise LIMIT post-fetch:** The engine cannot know which record within
the first overfull page satisfies the LIMIT exactly. Fetching one complete page and letting
DataFusion trim is the cleanest separation of concerns: the pipeline layer handles transport;
the query layer handles record-level selection.

**Why not push LIMIT into the API request body/URL for OffsetLimit sensors:** For Claroty
`vulnerabilities` (POST body injection, page_size = 1000), pushing `limit = 1` into the API
would fetch a 1-record page, which is a different API call with potentially different
server-side behavior. The canonical mechanism for single-record fetches is `LIMIT` at the
DataFusion layer. The page_size in the TOML is calibrated for efficient batched fetching.

---

## Consequences

### Positive
- `SELECT ... FROM claroty_vulnerabilities | LIMIT N` (small N) fetches only `ceil(N / 1000)`
  pages instead of the full dataset. `LIMIT 1` → 1 page (~1.1 MB, ~5s) instead of 5+ pages
  (~5.5 MB, >30s). Unblocks S-CLAROTY-VULNS-001 live green.
- Applies sensor-agnostically to ALL offset/limit and cursor-paginated tables in the engine.
  CrowdStrike, Armis, Cyberint sensor queries with LIMIT benefit automatically.
- No behavioral change when LIMIT is absent (`early_stop_limit = None`); full pagination
  proceeds as before.
- DI-019 10K cap is unchanged; it continues to fire as the outer safety net.

### Negative / Trade-offs
- `FetchContext::new()` signature expands by one parameter. All callers must be updated.
  Currently one production caller (`spec_driven_adapter.rs`). The `#[non_exhaustive]` on
  `FetchContext` prevents external struct-literal construction; the `new()` function change
  is a breaking change for downstream code using the public `new()` constructor. Acceptable
  given the engineering need.
- `ast_is_reducing_plan` signature changes from `(&Ast, &FilterMap) -> bool` to `(&Ast) -> bool`
  in v1.3. The call site in `run_materialization_pipeline` and any existing Red Gate tests that
  pass `where_filters` must be updated. The BC-2.16.002 postcondition text must be amended by
  the product owner to reflect the new signature and the revised Condition G.
- When LIMIT early-stop fires and the first page has 1000 rows but LIMIT is 1, DataFusion
  materializes 999 unnecessary records before discarding them. This is the irreducible cost
  of page-granularity stopping. For typical LIMIT values (5–100) against page_size = 1000,
  the overhead is negligible compared to avoiding the extra HTTP fetches.
- Queries combining ORDER BY + LIMIT (without aggregate in ORDER BY) do NOT return globally
  sorted top-N (documented in D8.5). This is an expected limitation of a federated query engine
  without server-side sort propagation.
- Queries with any non-temporal WHERE predicate (including non-equality forms like CONTAINS,
  BETWEEN, etc., plus ALL predicates in filter-mode and pipe-stage WHERE) have early-stop
  suppressed (§D8.7 Condition G revised). They paginate fully to the DI-019 10K cap. This is
  the correct safe scope until Wave 5 per-sensor push-down classification (ADR-033 extension)
  can identify which predicates are fully server-side. At that point, Condition G can be
  refined to exempt proven server-side predicates.
- Queries with SQL JOINs or Pipe Join stages also have early-stop suppressed (Conditions H, J).
  Both join partners paginate fully to DI-019, which matches pre-story behavior.
- Early-stop's performance benefit is scoped to bare projections with no WHERE clause (or
  temporal-only WHERE), no JOINs, no reducing operators: `SELECT [cols/*] FROM table LIMIT N`
  and its pipe-mode equivalents. This is the correct safe scope; the conservative gate ensures
  no silent correctness regressions for the SOC-analyst aggregation and filtered queries that
  are the v1 core use case.

---

## Alternatives Considered

**Alt-A: Push LIMIT into OffsetLimit POST body as the page_size** — Rejected. Changing
`page_size` from 1000 to 1 is a different API semantic: the server may enforce minimum
page sizes, respond differently for tiny page requests, or charge differently per call.
The TOML page_size is a calibrated transport parameter; it should not be overridden by
query semantics.

**Alt-B: Two-level truncation (record-level early-stop)** — Rejected. Stopping mid-page
after appending N records from a page of M records would violate the page-atomicity guarantee.
All records from a received page are either kept or discarded together.

**Alt-C: Engine-level LIMIT annotation on sensor table scans** — Considered as an alternative
to FetchContext threading. DataFusion supports custom TableProvider with LIMIT pushdown hints.
Rejected for this iteration: it requires a more significant refactor of `SensorTableProvider`
and is architectural scope for a separate effort. FetchContext threading is the minimum viable
mechanism; the TableProvider approach is a future optimization.

**Alt-D: Suppress early-stop for ORDER BY + LIMIT** — Rejected. Suppressing early-stop whenever
`order_by` is non-empty would require a full-dataset scan before sorting, eliminating the
optimization for common "show recent N" queries. The §D8.5 accepted-limitation (results in
API-declared order, not globally sorted) is preferable for queries like
`SELECT * FROM alerts ORDER BY severity LIMIT 100` where the consumer wants any 100 alerts
sorted by severity, not specifically the globally-ranked top 100. Consumers needing globally
ranked top-N should use time-window predicates to bound the dataset server-side, or use ORDER BY
without LIMIT (accepting full-scan cost). ORDER BY is different from GROUP BY/aggregation: it
does not reduce the row count or change semantic correctness of a "return N rows" request —
it only changes which N rows are returned.

---

## Source / Origin

DEFECT-2 (S-CLAROTY-VULNS-001 live monroe validation, 2026-08-26). The `| LIMIT 1` query
exhausted the 30s query budget fetching the full `claroty_vulnerabilities` dataset. ADR-059
is WITHDRAWN (D-2312: h2 flow-control window hypothesis falsified; no transport change was
adopted); DEFECT-2 is independent — the LIMIT over-fetching defect was observed in isolation
and does not depend on any h2 fix. The DI-019 precedent (10K truncation as non-error early
stop) confirmed that page-boundary early stopping is consistent with the existing atomicity
contract.

**F-R11-CRIT-001** (LOCAL cascade round-11, 2026-08-26): early-stop was firing for reducing
queries (`SELECT COUNT(*)`, `GROUP BY severity`, WHERE-filtered projections) because
`fetch_limit` was derived unconditionally from `options.limit` (always non-zero on the MCP
path, default 25). The plan-shape gate in §D8.7 closes this regression.

**F-R12-CRIT-001** (LOCAL cascade round-12, 2026-08-26): `expr_contains_aggregate` did not
recurse into `FuncCall::Scalar` arguments. A query `SELECT severity_label(max(severity_id))
FROM t LIMIT 5` escaped Condition A. Closed by v1.3 (renamed to
`expr_contains_aggregate_or_window`; recursion into `FuncCall::Scalar::args` added).

**F-R12-HIGH-001** (LOCAL cascade round-12, 2026-08-26): SQL JOINs and Pipe Join stages were
not suppression conditions. `SELECT * FROM a JOIN b ON a.id = b.id LIMIT 5` erroneously
permitted early-stop, truncating the join inputs. Closed by v1.3 (Conditions H and J).

**v1.3 comprehensive audit (2026-08-27)**: Human-directed exhaustive grammar enumeration
found six additional gaps: Condition A did not scan ORDER BY expressions; Condition G was
based on `where_filters` (always empty for Filter/Pipe modes, and missing non-equality
SQL predicates); PipeStage::Tail not suppressed; FuncCall::Window not suppressed; no
conservative default posture. All closed in v1.3.

**F-R16-P16-LENSA-HIGH-001** (correctness round 16 pass 16, 2026-08-28): `datetime_index_cols`
construction in `run_materialization_pipeline` iterated `resolved_spec_map.values()` (all sensors)
rather than the source-scoped `resolved_col_map` (filtered to the queried `source_names`). Armis
`devices.last_seen` is declared `options = ["INDEX"]`; CrowdStrike `devices.last_seen` is NOT.
A query `SELECT * FROM crowdstrike_devices WHERE last_seen > '...' LIMIT 100` caused the gate to
find `"last_seen"` in the unscoped `datetime_index_cols` (contributed by Armis), PERMIT early-stop,
while the push-down extractor — which IS source-scoped — correctly excluded CrowdStrike `last_seen`
from push-down. DataFusion applied the client-side filter against the early-stopped page set →
silent under-return. Root cause: the gate's column-eligibility classifier was an independent
parallel implementation of the push-down extractor's column set, rather than a shared helper.
Closed by v1.8 §D8.9 source-scoped construction + `collect_datetime_index_cols` shared helper.

**F-R16-P16-LENSA-LOW-001** (correctness round 16 pass 16, 2026-08-28): `is_pushed_temporal_predicate`
implementation contained a `rhs_pushed` branch permitting reversed-operand form
`Literal::Timestamp OP Field`. `extract_time_bounds_from_predicate` (ADR-033 T1) only extracts
when `lhs` is `Expr::Field`; the reversed form is not processed server-side. The `rhs_pushed`
branch was parser-unreachable (PrismQL grammar does not emit this form) but unsound in the PERMIT
direction. The ADR §D8.7 precondition spec was already correct (it specified LHS=Field,
RHS=Timestamp); the implementation deviated from the spec. Closed by v1.8 §D8.7 explicit
reversed-operand prohibition (implementer removes `rhs_pushed` branch from code to match spec).

**F-R16-P16-LENSB-LOW-001** (correctness round 16 pass 16, 2026-08-28): `count_temporal_bound_directions`
counts direction totals globally, not per-column. The single-datetime-INDEX-per-table invariant
(upheld by all shipped Wave 3+ sensor TOMLs) ensures correctness today, but the gate did not
enforce this invariant. A future TOML adding a second Datetime+INDEX column to any table would
enable an incorrect PERMIT via global `(lower=1, upper=1)` from mixed-column predicates.
Closed by v1.8 Condition K: conservative suppression when `collect_datetime_index_cols` detects
≥2 Datetime+INDEX columns on any queried source table.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.18 | 2026-08-30 | architect | F-B1V-002 (MEDIUM/spec-accuracy): §D8.3 worked example (d) corrected — arithmetically unreachable numbers (page_size=1000, LIMIT=5, partial-page=3) replaced with reachable scenario matching `test_early_stop_multi_batch_partial_page_is_truncated` (RG-PSG-044): page_size=10, LIMIT=5; 4 IDs at fan_out_batch_size=2 → 2 intra-pipeline batches; batch-0 (batch_idx=0, non-final, is_last_batch=false) returns partial page 5 records (5 < page_size 10), accumulated=5 ≥ limit=5 → early-stop fires; discriminator `(page_record_count 5 >= active_page_size 10) \|\| !is_last_batch = false \|\| true = true` → `early_stopped=true` → `is_truncated=true`; batch-1 abandoned by `break 'steps` (proven by test's 2-HTTP-request assertion). §D8.2 discriminator formula and disambiguation note unchanged. Rows (a)/(b)/(c) unchanged. Closes F-B1V-002. |
| 1.17 | 2026-08-30 | architect | F-B1-001 (MEDIUM): §D8.2 discriminator formula extended — `early_stopped = (page_record_count >= active_page_size) \|\| !is_last_batch`; `is_last_batch = (batch_idx + 1 == batch_count)` refers to intra-pipeline step fan-out batches within `execute_impl` batch-loop (over `fan_out_batches`/`fan_out_batch_size`), DISTINCT from multi-sensor fan-out at the `FanOutResult` layer (§D8.9/§D8.10). Non-final batch (`!is_last_batch = true`): `break 'steps` abandons remaining step-fan-out batches → data genuinely incomplete → `early_stopped = true` regardless of page fill. Final batch: falls back to page-fill discriminator. Common no-fan-out case (`batch_count == 1`): reduces exactly to `page_record_count >= active_page_size`. §D8.3 updated: description cites full discriminator formula; bullet list extended (non-final-batch case added); worked example (d) added (batch 1 of 2, partial page → `early_stopped = true`); disambiguation note added (intra-pipeline step fan-out vs multi-sensor `FanOutResult.any_early_stopped` OR-aggregation). Documents implemented+verified behavior (code @704aac24a; `test_early_stop_multi_batch_partial_page_is_truncated`, GREEN). Closes F-B1-001. |
| 1.16 | 2026-08-29 | architect | F-P9-LENSA-001 DECISION (A): §D8.4 spec-reconcile — `PaginationConfig::None` moved from contradictory "does NOT apply to None" exclusion to explicit conservative-bucket documentation; `active_page_size = 0` (via `_ => 0` catch-all) → `early_stopped = true` at exact-LIMIT corner (safe over-report, narrow, latent for v1); accepted conservative corner consistent with CursorToken treatment; §D8.2 comment updated to note `_` captures None; §D8.2 NOTE updated to cite None alongside CursorToken. No code change; feature HEAD 62e50205b frozen unchanged. |
| 1.15 | 2026-08-29 | architect | F-P2-LENSC2-001: §D8.4 TD-VSDD-097 Dim-3 discharge note — stale STORY-INDEX version pin removed; "(v1.0 draft" → "(draft"; POL-39 compliant and decay-proof. F-P2-LENSC2-002: §Status banner POL-39 sweep count corrected from ~14 to 20 to match §Changelog 1.14 row enumeration (20 items verified); banner and changelog now self-consistent. |
| 1.14 | 2026-08-29 | architect | F-P1B-LENSC2-001: §D8.4 TD-VSDD-097 Dim-3 note updated — S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001 now exists and is registered in STORY-INDEX (v1.0 draft, blocked_by S-OCSF-FIDELITY-CYBERINT-001); prohibitive MUST anchored to registered story; "does not yet exist / MUST create" language removed; Dim-3 status: DISCHARGED. F-P1B-LENSC2-002: (1) §D8.7 heading volatile stamp `(v1.3 — Comprehensive Audit)` removed; (2) POL-39 in-body sweep — 20 normative version pins anchor-ized: `#### Gate Function Signature (v1.3)`, `where_filters present in v1.2 is REMOVED`, `(v1.5 unconditional suppression)`, Pipe-arm `removed from PERMIT allow-list in v1.5`, SqlPipe-arm `unconditionally suppressed in v1.5`, `Two-step check (v1.7 — replaces single-step all() of v1.5/v1.6)`, Condition A `(revised from v1.2)`, Conditions B/C/D/E/F `(unchanged from v1.2)` labels (×5), Condition G `(revised from v1.2)`, shape-table Condition K `(v1.8)`, Condition K header `(v1.8)`, `#### Conservative Default Posture (new in v1.3)`, `#### Gate Application (v1.3)`, `**Construction rules (v1.8)**`, `**OCSF-name gap (preserved from v1.7)**`, canonical-PERMIT `(unchanged from v1.5)`; ~6 decision-history version refs preserved as intentional; §D8.7 POL-39 convention note added. |
| 1.13 | 2026-08-29 | architect | F-FP1-LENSA-001 DECISION (B): cursor page-fill discriminator unsound — revert + narrow + anchor. §D8.2: CursorToken unified under `_ => 0` catch-all (removes `CursorToken { page_size: Some(ps) } => ps as usize` arm from v1.12); comment variable renamed `active_page_size` (was `page_size`) throughout §D8.2 to match §D8.4 + ratified impl naming (F-FP1-LENSC2-002 naming fix). §D8.4: CursorToken narrowed to conservative-only across ALL sub-cases; removes the `Some(ps)` precise arm; rationale: page-fill is NOT a valid cursor exhaustion signal (partial cursor page with non-empty next cursor means more data exists — under-report is DANGEROUS); all CursorToken → `active_page_size = 0` → `early_stopped = true` (safe over-report); precise next-cursor-presence detection deferred to S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001 (post-v1, blocked on S-OCSF-FIDELITY-CYBERINT-001). No v1 sensor uses cursor pagination; zero v1 behavioral impact. TD-VSDD-097 Dim-3: deferral anchored to real story ID S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001. |
| 1.12 | 2026-08-29 | architect | F-P1-LENSC2-003 DECISION (A) code-extension: §D8.2 `page_size` derivation comment extended — `OffsetLimit { page_size } => page_size as usize`; `CursorToken { page_size: Some(ps) } => ps as usize`; `CursorToken { page_size: None } \| _ => 0` (conservative full-page). §D8.4 extended to document the `None` fallback and the precise partial-final-page discriminator semantics for CursorToken. §D8.4 decision text unchanged — already correctly declared both modes. F-P1-LENSC2-001 (TD-VSDD-097 Dim-3): §D8.3 forward-reference "will be added" replaced with discharged anchor `AC-014 + RG-PSG-039 + RG-PSG-040 (S-ENGINE-LIMIT-EARLY-STOP-001), GREEN`. F-P1-LENSC2-002 (POL-39 sweep): 0 in-body artifact-version pins found/stripped in ADR-060 body prose; BC-version refs in §Changelog rows are EXEMPT. |
| 1.11 | 2026-08-28 | architect | F-P31-LENSA-OBS-001 (human-approved Option 2 refinement). Introduced **partial-final-page discriminator** for `PipelineResult.early_stopped`: when the §D8.2 break fires and the triggering page was partial (`page_record_count < page_size`), source is exhausted — `early_stopped = false`; when the page was full (`>= page_size`), more pages may exist — `early_stopped = true` (unchanged for full-page case, including exact-full-page corner treated conservatively). Closes self-contradictory `is_truncated: true` + `total_available == returned_results` on exact-limit / partial-final-page queries (e.g., `LIMIT 5` on a 5-row tenant). §D8.2 code sketch updated with discriminator capture; §D8.3 post-break semantics redesigned with discriminator rule + three worked examples (partial-exhausted → `is_truncated: false`; full-page normal → `is_truncated: true`; exact-full-page corner → `is_truncated: true` accepted conservative). §D8.9 Motivation updated; `PipelineResult.early_stopped` description updated; Step 6 formula commentary extended with partial-page CLEAN arm; `total_available` lower-bound note clarified (exact when both signals false). Engine Step 6 `is_truncated` formula (code) UNCHANGED — discriminator operates by setting `early_stopped = false` at source, not by changing formula. Anchor story S-ENGINE-LIMIT-EARLY-STOP-001; story-writer/test-writer add new AC + Red Gate test. Downstream sweep required: BC-2.11.001 §EC-11-092 (Step 6 formula commentary), BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop arm. |
| 1.10 | 2026-08-28 | architect | F-P20-LENSC-MED-001 (§D8.9 source-vs-copy struct inversion). §D8.9 `FetchOutput Return Type` struct block reconciled to the canonical 3-field form matching §D8.10 authoritative definition: `pub pipeline_truncated: bool` added as third field. §D8.9 Propagation Chain extended with parallel DI-019 truncation arm: `PipelineResult.truncated → FetchOutput.pipeline_truncated → FanOutResult.any_pipeline_truncated → MaterializationOutput.any_pipeline_truncated`, feeding the same engine.rs Step 6 `is_truncated` formula. The §D8.9 struct block was the only remaining artifact carrying the 2-field shape; downstream copies (S-ENGINE-LIMIT-EARLY-STOP-001 story, BC-2.11.001 §EC-11-092) were already reconciled at the §D8.10 introduction. |
| 1.9 | 2026-08-28 | architect | F-R16-P18-LENSA-MED-001 + F-R16-P18-LENSA-OBS-001. §D8.10 DI-019 truncation-signal propagation chain: `PipelineResult.truncated` was dropped at the `spec_driven_adapter.rs` adapter boundary — `FetchOutput::new()` consumed only `result.early_stopped`, not `result.truncated`. A >10K single-sensor fetch produced `truncated = true, early_stopped = false`; `FanOutResult.any_early_stopped = false`; cache-completeness gate passed; DI-019-capped partial cached as COMPLETE. Subsequent cache hit served it with `is_truncated = false` (same class as EC-01-039/RG-PSG-034, on the DI-019 sibling path). Fix mirrors the established `any_early_stopped` plumbing: (1) `FetchOutput` gains `pipeline_truncated: bool` field (signals DI-019 cap from `PipelineResult.truncated`); `FetchOutput::new()` gains 3rd argument; 21+ construction sites updated; (2) `spec_driven_adapter.rs` `fetch` OR-aggregates `result.truncated` into `any_pipeline_truncated` and passes to `FetchOutput::new()`; (3) `FanOutResult` gains `any_pipeline_truncated: bool` (OR-aggregate, derives `Default = false`); (4) `MaterializationOutput` gains `any_pipeline_truncated: bool`; (5) cache-completeness gate (ADR-060 §D8.3) updated: `complete = errors.is_empty() && !any_early_stopped && !any_pipeline_truncated`; (6) engine.rs analyst Step 6 formula updated: `is_truncated = (total_rows > limit) \|\| any_early_stopped \|\| any_pipeline_truncated`; §D8.9 formula updated; (7) F-R16-P18-LENSA-OBS-001 (TD-VSDD-060 dim-b sibling-sweep): `execute_scheduled_inner` hardcoded `is_truncated: false` replaced by `output.any_early_stopped \|\| output.any_pipeline_truncated` — latent but preventable; scheduled path has `early_stop_limit = None` so `any_early_stopped` is always `false`, but DI-019 can fire on large scans. Two new Red Gate tests required: RG-PSG-035 (`test_psg_rg035_di019_truncated_response_not_cached_as_complete` — cache-completeness gate), RG-PSG-036 (`test_psg_rg036_di019_truncated_step6_is_truncated_true` — analyst Step 6 formula). Product-owner amends BC-2.16.002 to add EC-01-040 (DI-019 cache-completeness) and update EC-01-039 Step 6 formula reference; update Step 6 postcondition bullet; update `is_truncated` formula bullet. |
| 1.8 | 2026-08-28 | architect | F-R16-P16-LENSA-HIGH-001 + F-R16-P16-LENSA-LOW-001 + F-R16-P16-LENSB-LOW-001. §D8.9 source-scoped `datetime_index_cols` (HIGH-001): construction now uses source-scoped `resolved_col_map` from `build_source_column_map(spec_map, source_names)` rather than `resolved_spec_map.values()` (all sensors); eliminates cross-sensor column-name pollution (Armis INDEX `last_seen` appearing in CrowdStrike query's index set). Shared `collect_datetime_index_cols(col_map, ocsf_naming_map) -> (Vec<String>, bool)` helper introduced in `materialization.rs`; both gate and `extract_time_window_from_ast` (pushdown.rs) MUST call it to prevent future column-eligibility divergence. §D8.7 reversed-operand prohibition explicit (LOW-001): `Literal::Timestamp OP Field` returns `false` (SUPPRESS); mirrors `extract_time_bounds_from_predicate` which requires Field on LHS. Implementer removes `rhs_pushed` branch from code (ADR spec was already correct; code deviated from spec). §D8.7 Condition K (LENSB-LOW-001): conservative suppression when any queried source table has ≥2 Datetime+INDEX columns; prevents direction-count global-vs-per-column semantic gap from producing incorrect PERMIT if a future TOML adds a second INDEX Datetime column. Three new Red Gate tests required: RG-PSG-032 (`crowdstrike_devices WHERE last_seen > '...' LIMIT 100` → SUPPRESS; cross-sensor source-scope), RG-PSG-033 (`armis_devices WHERE last_seen > '...' LIMIT 100` → PERMIT; Armis last_seen IS INDEX), RG-PSG-030b (two-upper-bound `(0,2)` → SUPPRESS). Product-owner amends BC-2.16.002 v2.43→v2.44: source-scoped `datetime_index_cols` description; EC for cross-sensor SUPPRESS; EC for reversed-operand SUPPRESS; EC for multi-INDEX-datetime SUPPRESS; POL-39 fixes in EC-01-034/035 (strip bare version pins to section anchors). |
| 1.7 | 2026-08-28 | architect | F-R16-P15-LENSA-HIGH-001 + F-R16-P15-LENSA-MED-001 + F-R16-P15-LENSC-LOW-001. §D8.7 `is_pushed_temporal_predicate` AND-arm soundness redesign (HIGH-001): AND-arm gains a second step — after all-leaves-individually-pushed check, a direction-count helper `count_temporal_bound_directions` verifies at most one Gt/Ge leaf and at most one Lt/Le leaf exist in the flattened AND tree; mirrors `extract_time_bounds_from_predicate` first-wins semantics (single-lower + single-upper fully consumed server-side; any second same-direction bound silently dropped to DataFusion client-side). Previously, `WHERE col > X AND col > Y` was incorrectly classified PERMIT → silent LIMIT under-return; now correctly SUPPRESS. §D8.9 `datetime_index_cols` OCSF-name gap (MED-001): `datetime_index_cols` construction MUST mirror `extract_time_window_from_ast` — per-source lookup consulting `ocsf_naming_map`; when sensor `ocsf_column_naming = true` AND `col.ocsf_field` is Some, `ocsf_field_to_arrow_name(ocsf_field)` is also inserted. Previously, OCSF-named sensors (e.g., claroty.audit_logs `col.name="timestamp"`, Arrow field `"time"`) had early-stop wrongly suppressed for `WHERE time > '...'`. §Status banner (LOW-001): updated to ACCEPTED v1.7 (2026-08-28). `related_adrs` gains ADR-058 (OCSF column naming). Anchored: S-ENGINE-LIMIT-EARLY-STOP-001 AC-007; two new Red Gate tests required (IDs RG-PSG-030 + RG-PSG-031 — next available after RG-PSG-029 per story; story-writer assigns): (1) RG-PSG-030 — redundant-lower-bound suppresses early-stop; (2) RG-PSG-031 — OCSF-flattened Arrow name permits early-stop. Product-owner amends BC-2.16.002 v2.41→v2.42 (EC-01-034 redundant-bound suppress; EC-01-035 OCSF-name permit; "mirrors" claim correction; AND-arm direction-count spec; RG-PSG-030/031 anchors). |
| 1.6 | 2026-08-27 | architect | MED-001 (F-R16-P1-MED-001): ADR-059 citation reframe — §Context §Defect Evidence and §Source/Origin clauses corrected. ADR-059 is WITHDRAWN (hypothesis falsified, D-2312); DEFECT-2 (this ADR) is independent and was observed in isolation. The prior framing implied DEFECT-2 was contingent on DEFECT-1 being applied first; that is false — the LIMIT over-fetching defect exists regardless of h2 transport behavior. No behavioral decision changes. |
| 1.5 | 2026-08-27 | architect | Temporal-exemption soundness redesign (§D8.9): `is_pushed_temporal_predicate(pred, datetime_index_cols: &[&str])` replaces `is_purely_temporal_predicate`; mirrors `extract_time_bounds_from_predicate` (ADR-033 T1) exactly — requires range op (Gt/Ge/Lt/Le) + LHS in `datetime_index_cols` (INDEX datetime col) + RHS `Expr::Literal(Literal::Timestamp)`. `Ast::Filter` unconditionally SUPPRESS in `has_client_side_where` (closes F-R15-LENSA-CRIT-001 filter-mode path). `PipeStage::Where` unconditionally SUPPRESS in `has_client_side_where` (closes F-R15-LENSA-CRIT-001 pipe-mode path). `expr_contains_aggregate_or_window` catch-all corrected: `_ => false` (stale) → `_ => true` (conservative SUPPRESS; per F-R14-LOW-001). `datetime_index_cols: &[&str]` param threaded through `has_client_side_where` and `is_pushed_temporal_predicate`. §D8.9 `any_early_stopped` truncation-signal propagation chain: `PipelineResult.early_stopped` → `FetchOutput { batches, any_early_stopped }` → `FanOutResult.any_early_stopped` → `MaterializationOutput.any_early_stopped` → engine Step 6 `is_truncated = (total_rows > limit) \|\| any_early_stopped` (closes F-R15-LENSA-HIGH-001 exact-limit boundary). |
| 1.4 | 2026-08-27 | architect | Subsystem-anchoring correction (F-R13-LENSC-HIGH-001): SS-11 (Query Execution) and SS-07 (Adapter Pagination & Response Cache) added to `subsystems_affected`. SS-11 owns `prism-query::materialization.rs` — the `fetch_limit` derivation and plan-shape gate enforcement site (§D8.7). SS-07 owns `execute_impl` — the per-page early-stop check (§D8.2) — and the response-cache-key coherence path where `fetch_limit` is the cache-key limit component (§D8.8). No behavioral change; frontmatter correction only. |
| 1.3 | 2026-08-27 | architect | §D8.7 comprehensive plan-shape surface audit. Closes F-R12-CRIT-001 (aggregate recursion gap: `expr_contains_aggregate_or_window` now recurses into `FuncCall::Scalar` args and detects `FuncCall::Window`). Closes F-R12-HIGH-001 (SQL JOIN → Condition H; Pipe Join stage → Condition J). Six additional gaps closed: Condition A extended to scan `order_by` expressions; Condition G redesigned — replaced `where_filters` (equality push-down map, always empty for Filter/Pipe modes) with `has_client_side_where()` covering all four AST modes and all non-temporal predicate forms; Condition I added (PipeStage::Tail); conservative default posture added (`_ => true` catch-all for unknown AST/PipeStage variants). Signature change: `where_filters` parameter removed — gate performs its own AST inspection. Out-of-grammar shapes documented (UNION/INTERSECT/EXCEPT, CTEs, FROM subquery, OFFSET: not gated). Complete shape-classification table added. §D8.7 replaced in full; §D8.8 coherence note updated for new signature; §Consequences updated; §Source updated. |
| 1.2 | 2026-08-26 | architect | §D8.7 plan-shape gate: closes F-R11-CRIT-001. Suppresses early-stop (`fetch_limit=0`) for reducing plans (aggregation, GROUP BY, DISTINCT, HAVING, Stats, Dedup, non-temporal WHERE). §D8.1 annotated with gating precondition. §D8.8 single-binding coherence clarification. §Consequences and §Alternatives updated. |
| 1.1 | 2026-08-26 | architect | §D8.1 prose correction: LIMIT is read from `QueryParams.limit: u64` (pre-extracted; 0 = no limit), not from DataFusion physical-plan inspection. Behavioral decision D8 unchanged. |
| 1.0 | 2026-08-26 | architect | Initial — D8 LIMIT-aware early-stop pagination, atomicity reconciliation ruling, timeout_secs deferral. |
