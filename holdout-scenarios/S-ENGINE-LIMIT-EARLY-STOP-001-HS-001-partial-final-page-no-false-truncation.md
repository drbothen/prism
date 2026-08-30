---
document_type: holdout-scenario
level: L3
id: "HS-EARLY-STOP-001-001"
title: "LIMIT-aware early-stop: LIMIT = exact dataset size on partial final page → is_truncated: false (EC-11-094 discriminator)"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-XDOME-EXPANSION"
story_source: "S-ENGINE-LIMIT-EARLY-STOP-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-30T00:00:00Z"
modified: "2026-08-30"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md"
  - ".factory/specs/architecture/decisions/ADR-060-limit-aware-early-stop-pagination.md"
input-hash: "6c22d50"
traces_to: "BC-2.11.001"
behavioral_contracts:
  - BC-2.11.001
  - BC-2.16.002
verification_properties: []
lifecycle_status: active
introduced: "S-ENGINE-LIMIT-EARLY-STOP-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-ENGINE-LIMIT-EARLY-STOP-001 (HS-030 group). Tests EC-11-094 partial-final-page discriminator (ADR-060 §D8.2/§D8.3): when SQL LIMIT N exactly equals the DTU fixture row count, the final page from the API is PARTIAL (10 < page_size=1000), the discriminator sets early_stopped=false, and is_truncated must be FALSE. A broken discriminator that unconditionally sets early_stopped=true when early-stop fires would emit is_truncated=true — this scenario catches that defect. Runs against Claroty DTU (claroty.alerts, 10 records, page_size=1000). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-EARLY-STOP-001-001: LIMIT-aware early-stop: LIMIT = exact dataset size on partial final page → is_truncated: false (EC-11-094 discriminator)

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ENGINE-LIMIT-EARLY-STOP-001 (HS-030 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.11.001 §Edge Cases EC-11-094 (partial-final-page discriminator —
`early_stopped = page_record_count >= page_size`; PARTIAL final page → `early_stopped = false`
→ `is_truncated = false`; ADR-060 §D8.2/§D8.3 worked example (a): LIMIT N on tenant with
exactly N rows, page_size >> N → partial page → is_truncated = false) and
BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop (ADR-060 §D8) — EC-01-041 partial-final-page
arm: OffsetLimit only; `active_page_size = page_size`; `early_stopped = page_record_count >= page_size`.
**Gate:** Story-level holdout gate (HS-030) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **partial-final-page discriminator** (ADR-060 §D8.2/§D8.3) at
wire level. The Claroty DTU `alerts` fixture contains exactly 10 records, and the TOML spec
declares `page_size = 1000`. A SQL `LIMIT 10` query therefore:

1. Causes `FetchContext.early_stop_limit = Some(10)` to be set at the adapter.
2. Fetches page 1: the DTU returns all 10 records in a single response.
3. After page 1: `all_records.len() = 10 >= early_stop_limit = 10` → the early-stop check fires.
4. **Discriminator fires:** `page_record_count = 10`, `page_size = 1000`.
   `early_stopped = (10 >= 1000) = false` — the page is PARTIAL (partial page = API returned
   fewer rows than page_size, indicating source is exhausted; no further pages exist).
5. The pipeline stops with `FetchOutput { batches, any_early_stopped: false, pipeline_truncated: false }`.
6. DataFusion applies SQL `LIMIT 10` → 10 rows returned.
7. `is_truncated = (10 > tool_limit) OR false OR false = false`.

**The defect this scenario catches:** A broken discriminator that unconditionally sets
`early_stopped = true` whenever the early-stop check fires (i.e., omits the
`page_record_count >= page_size` condition) would produce `any_early_stopped = true` → `is_truncated = true`
even though the entire dataset was returned (F-P31-LENSA-OBS-001 root cause).

**BDD supplement:**

**Given** prism is built from the S-ENGINE-LIMIT-EARLY-STOP-001 story branch
**And** prism MCP stdio is started with the Claroty DTU configured (DTU serving claroty.alerts
with 10 fixture records, `type = "offset_limit"`, `page_size = 1000`)
**When** `SELECT * FROM claroty.alerts LIMIT 10` is issued via the MCP `query` tool
**Then** the response is not an error
**And** `query_context.returned_results` equals `10`
**And** `query_context.total_available` equals `10`
**And** `query_context.is_truncated` equals `false`
**And** the `rows` array contains exactly 10 row objects

---

## Setup Instructions

1. Confirm prism is built from the S-ENGINE-LIMIT-EARLY-STOP-001 story branch HEAD commit.

2. Start the Claroty DTU (prism-dtu-claroty) locally. It will serve the standard fixture data:
   `crates/prism-dtu-claroty/fixtures/alerts.json` (10 records) at `POST /api/v1/alerts`.

3. Start prism in MCP stdio mode with the claroty sensor spec configured to point to the local
   DTU instance. Capture full MCP stdio output and stderr.

4. Wait for prism to be ready (startup completion log message or first JSON-RPC prompt).

5. Issue the MCP `query` tool call:
   `{"sql": "SELECT * FROM claroty.alerts LIMIT 10"}`.

6. Capture the full raw wire-level JSON response from the MCP tool call, including the complete
   `query_context` object.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | §Edge Cases EC-11-094: partial-final-page arm — `page_record_count < page_size` → `early_stopped = false` → `is_truncated = false` | Primary assertion: `is_truncated: false` when LIMIT = exact row count (partial page) |
| BC-2.11.001 | §Edge Cases EC-11-094: `total_available` is EXACT when `any_early_stopped = false` | Supporting assertion: `total_available = 10 = returned_results` |
| BC-2.16.002 | §Postconditions LIMIT-Aware Early-Stop EC-01-041: OffsetLimit partial-final-page arm; `early_stopped = page_record_count >= page_size`; `active_page_size = page_size` from TOML | Wire-level proof: the discriminator fires correctly for OffsetLimit mode |
| BC-2.16.002 | §Postconditions LIMIT-Aware Early-Stop: `truncated` NOT set on early-stop (reserved for DI-019 capacity overflow) | `any_pipeline_truncated = false` (DI-019 not triggered on 10-record dataset) |

---

## Verification Approach

1. Parse the wire-level JSON response from the MCP `query` tool call.

2. If the response is an error object (contains `error_code` or equivalent): record FAIL with
   observation "query returned error; early-stop wiring may have broken the execute_impl loop."

3. Locate `query_context` in the response. Assert `query_context.is_truncated == false`.
   **If `is_truncated == true`**: record FAIL on "partial-final-page discriminator" dimension.
   This is the primary defect this scenario targets — `is_truncated: true` on a query that
   returned the complete dataset is an incorrect signal (F-P31-LENSA-OBS-001 defect class).

4. Assert `query_context.returned_results == 10`. If not 10: record FAIL on "row count" dimension.

5. Assert `query_context.total_available == 10`. If `total_available != returned_results`:
   record PARTIAL on "total_available exact" dimension (acceptable if both are 10).

6. Count the entries in the `rows` array. Assert the count equals 10.

7. Do NOT assert specific alert field values — fixture content may change across branches.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query succeeds (no error)** (weight: 0.20): Does the MCP query return a non-error response?
  Full credit (1.0): non-error response with ≥1 row.
  Zero credit (0.0): error response of any kind.

- **`is_truncated = false` (partial-final-page discriminator correct)** (weight: 0.45): Is
  `query_context.is_truncated` exactly `false`?
  Full credit (1.0): `is_truncated: false`.
  Zero credit (0.0): `is_truncated: true` — the discriminator is broken; early-stop is
  unconditionally setting `early_stopped = true` even on partial pages (F-P31-LENSA-OBS-001).

- **`returned_results = 10` (DataFusion LIMIT correct)** (weight: 0.20): Does
  `query_context.returned_results` equal 10?
  Full credit (1.0): `returned_results = 10`.
  Partial credit (0.5): `returned_results < 10` (early-stop fired before all 10 records
  were accumulated — indicates a premature pagination break).
  Zero credit (0.0): error or `returned_results = 0`.

- **`total_available = 10` (exact count, no truncation)** (weight: 0.15): Does
  `query_context.total_available` equal 10?
  Full credit (1.0): `total_available = 10`.
  Zero credit (0.0): `total_available != 10` or field absent.

---

## Edge Conditions

- **DTU not running or connection refused:** Record as SETUP-FAILURE — not a behavioral FAIL.

- **prism starts but fails to load the claroty sensor spec:** Record as SETUP-FAILURE.

- **`returned_results = 10` but `is_truncated = true`:** This IS the primary defect —
  `is_truncated: true` with `returned_results = total_available` is a self-contradictory
  signal. Record FAIL on "partial-final-page discriminator" dimension.

- **`returned_results < 10` (e.g., 1):** This indicates the early-stop check fired too early
  AND the discriminator was not applied correctly (DataFusion should still return all 10 records
  accumulated in one partial page). Record FAIL on "row count" dimension.

- **`total_available = 10` but `returned_results = 1`:** This would indicate the MCP tool
  `limit` parameter (not the SQL LIMIT clause) is being used as `early_stop_limit`, which would
  be a wiring error. Record FAIL.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-EARLY-STOP-001-001 (satisfaction: X.XX) — partial-final-page discriminator defect; is_truncated signal incorrect for a LIMIT query on a small dataset; check early_stopped discriminator condition in execute_impl (ADR-060 §D8.2 — early_stopped = page_record_count >= page_size, not unconditionally true) and is_truncated formula in engine Step 6 (BC-2.11.001 EC-11-094 + BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop EC-01-041)"`

Do NOT disclose: the specific SQL LIMIT value used, the fixture row count, or the exact expected values.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty DTU (prism-dtu-claroty) — `POST /api/v1/alerts`, fixture `crates/prism-dtu-claroty/fixtures/alerts.json` (10 records) |
| corpus_size | SQL LIMIT matches exact fixture count; all 10 records returned in one partial page (page_size=1000 >> 10) |
| known_edge_cases | `is_truncated: true` with `returned_results = total_available` — the self-contradictory discriminator defect this scenario targets |
| false_positive_threshold | Zero: `is_truncated: false` when LIMIT = dataset_size is an unambiguous discriminator correctness assertion |
| false_negative_threshold | Zero: if `returned_results < 10`, the accumulation loop stopped too early |

**Known-good corpus:** Claroty DTU with 10 alerts records and `page_size = 1000` — expected:
non-error response, `returned_results = 10`, `total_available = 10`, `is_truncated = false`.

**Known-problematic corpus:** An implementation where the partial-final-page discriminator is
absent (always `early_stopped = true` when early-stop fires) — expected: `is_truncated = true`
with the same 10-row result, which is the F-P31-LENSA-OBS-001 self-contradictory signal this
story's ADR-060 §D8.2/§D8.3 discriminator resolves.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | s-engine-limit-early-stop-001-holdout-authoring | 2026-08-30 | product-owner | Initial authoring. HS-030 group for S-ENGINE-LIMIT-EARLY-STOP-001. Partial-final-page discriminator test (EC-11-094): SQL LIMIT = DTU fixture count (partial page: count < page_size) → is_truncated=false. Catches broken discriminator (unconditional early_stopped=true). BC-2.11.001 EC-11-094 + BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop EC-01-041 (ADR-060 §D8.2/§D8.3). SINGLE-USE. |
