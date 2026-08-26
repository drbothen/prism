---
document_type: adr
adr_id: "ADR-060"
title: "LIMIT-Aware Early-Stop Pagination for Offset/Limit and Cursor Sensor Tables"
status: ACCEPTED
date: "2026-08-26"
modified: "2026-08-26"
version: "1.1"
producer: architect
subsystems_affected: [SS-01, SS-16]
supersedes: []
superseded_by: null
amends: null
anchor_stories:
  - S-ENGINE-LIMIT-EARLY-STOP-001   # story to be created; §Authority will cite this ADR
related_adrs: [ADR-028, ADR-033]
related_bcs: [BC-2.16.002, BC-2.16.015, BC-2.01.010]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-060: LIMIT-Aware Early-Stop Pagination for Offset/Limit and Cursor Sensor Tables

## Status

ACCEPTED v1.1 (2026-08-26) — D8: pagination loop in `PipelineExecutor::execute_impl` stops
fetching pages once accumulated records satisfy the query's LIMIT. Atomicity reconciliation
ruling: "atomic" in existing BCs means all-or-nothing on HTTP ERROR, not "always fetch the
entire dataset." v1.1 corrects §D8.1 prose: LIMIT is read from `QueryParams.limit: u64`
(pre-extracted; 0 = no limit), not from DataFusion physical-plan inspection.

---

## Context

### Defect Evidence

Live monroe validation of S-CLAROTY-VULNS-001 revealed that even after DEFECT-1 (h2 window fix,
ADR-059) is applied, a query `SELECT * FROM claroty_vulnerabilities | LIMIT 1` downloads the
FULL dataset (5000+ vulnerability records across multiple pages) before DataFusion applies the
LIMIT clause, consistently exceeding the 30s query budget (E-QUERY-004).

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
        break 'steps;
    }
}
```

This check fires only after a COMPLETE page has been received and its records appended to
`all_records`. It does NOT fire mid-page. The page atomicity guarantee is preserved: either
the entire page arrives (and is accumulated), or a fetch error discards everything.

### D8.3 — Post-break semantics

When early-stop fires (not DI-019 cap), the `truncated` flag is NOT set. The pipeline returns a
valid `PipelineResult` with `truncated: false` containing at most `limit + (page_size - 1)` records.
DataFusion applies the precise LIMIT on this result. The implementer MUST NOT set `truncated: true`
for LIMIT early-stop — `truncated` is semantically reserved for capacity-exceeded conditions
(DI-019), not for query-driven early stops.

### D8.4 — Applicable pagination modes

LIMIT early-stop applies to both `PaginationConfig::OffsetLimit` and `PaginationConfig::CursorToken`
pagination modes. It does NOT apply to `PaginationConfig::None` (single-page fetch; no loop to
terminate early) or to the 10K DI-019 cap (which remains unchanged and fires before D8 when
applicable).

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
- When LIMIT early-stop fires and the first page has 1000 rows but LIMIT is 1, DataFusion
  materializes 999 unnecessary records before discarding them. This is the irreducible cost
  of page-granularity stopping. For typical LIMIT values (5–100) against page_size = 1000,
  the overhead is negligible compared to avoiding the extra HTTP fetches.
- Queries combining ORDER BY + LIMIT do NOT return globally sorted top-N (documented in
  D8.5). This is an expected limitation of a federated query engine without server-side sort
  propagation.

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

---

## Source / Origin

DEFECT-2 (S-CLAROTY-VULNS-001 live monroe validation, 2026-08-26). The `| LIMIT 1` query
exhausted the 30s query budget fetching the full `claroty_vulnerabilities` dataset even after
the h2 window fix (ADR-059) was applied. The DI-019 precedent (10K truncation as non-error
early stop) confirmed that page-boundary early stopping is consistent with the existing
atomicity contract.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-08-26 | architect | §D8.1 prose correction: LIMIT is read from `QueryParams.limit: u64` (pre-extracted; 0 = no limit), not from DataFusion physical-plan inspection. Behavioral decision D8 unchanged. |
| 1.0 | 2026-08-26 | architect | Initial — D8 LIMIT-aware early-stop pagination, atomicity reconciliation ruling, timeout_secs deferral. |
