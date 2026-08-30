---
document_type: holdout-scenario
level: L3
id: "HS-EARLY-STOP-001-002"
title: "LIMIT-aware early-stop end-to-end: SQL LIMIT 1 on Claroty DTU returns 1 row with is_truncated: false (partial-page path)"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-XDOME-EXPANSION"
story_source: "S-ENGINE-LIMIT-EARLY-STOP-001"
version: "1.0"
status: active
used: true
last_evaluated: "2026-08-30"
last_eval_satisfaction: 1.0
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
traces_to: "BC-2.16.002"
behavioral_contracts:
  - BC-2.16.002
  - BC-2.11.001
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-ENGINE-LIMIT-EARLY-STOP-001 (HS-030 group). Tests the primary story motivation — SQL LIMIT 1 query on a sensor table completes correctly at the MCP wire level with the early-stop mechanism wired end-to-end. Against the Claroty DTU (10 records, page_size=1000): page 1 returns 10 records (partial page < page_size), early-stop fires after page 1, discriminator sets early_stopped=false (partial page), DataFusion LIMIT 1 trims to 1 row, is_truncated=false. Verifies the full wiring chain: SQL LIMIT → params.limit → FetchContext.early_stop_limit → execute_impl check → FetchOutput.any_early_stopped → FanOutResult → MaterializationOutput → engine Step 6 is_truncated formula. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-EARLY-STOP-001-002: LIMIT-aware early-stop end-to-end: SQL LIMIT 1 on Claroty DTU returns 1 row with is_truncated: false (partial-page path)

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ENGINE-LIMIT-EARLY-STOP-001 (HS-030 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop Pagination (ADR-060 §D8) —
wiring from `SpecDrivenSensorAdapter::fetch` through `FetchContext.early_stop_limit` through
`execute_impl` loop check through `FetchOutput.any_early_stopped` propagation chain.
Also BC-2.11.001 §Edge Cases EC-11-094 (partial-final-page path: `early_stopped = false`
for partial page → `is_truncated = false`).
**Gate:** Story-level holdout gate (HS-030) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario tests the end-to-end wiring of the LIMIT-aware early-stop feature using the
primary motivating query pattern (`SELECT * ... LIMIT 1`) against the Claroty DTU.

The story motivation (ADR-060 §Context DEFECT-2): on real claroty_vulnerabilities data with
thousands of records, `LIMIT 1` would previously fetch ALL pages (no early-stop), exhausting
the 30-second budget. After this story, `LIMIT 1` stops after the first page.

For the Claroty DTU (alerts fixture, 10 records, page_size=1000), the observable behavior is:

1. `SQL LIMIT 1` in the query text → `params.limit = 1` → `FetchContext.early_stop_limit = Some(1)`.
2. Page 1 fetched from DTU: 10 records returned.
3. After page 1: `all_records.len() = 10 >= early_stop_limit = 1` → early-stop check fires.
   The pipeline stops without requesting page 2.
4. Discriminator: `page_record_count = 10 < page_size = 1000` → PARTIAL page → `early_stopped = false`.
5. `FetchOutput { batches: [10 records], any_early_stopped: false, pipeline_truncated: false }`.
6. DataFusion SQL `LIMIT 1` → 1 row in MaterializationOutput.
7. Engine Step 6: `is_truncated = (1 > tool_limit) OR false = false`.

The wire response should contain: 1 row, `is_truncated: false`, `returned_results: 1`.

**Why `is_truncated: false` is correct here (not a signal of bug):** The Claroty DTU returned
all 10 of its records in one partial page (10 < 1000 = page_size). From the API's perspective,
the source is exhausted — there are no more pages. DataFusion's SQL `LIMIT 1` then trimmed the
10 fetched records to 1. The signal `is_truncated: false` means "no unread data remains in the
sensor API"; it does NOT mean "your SQL LIMIT was not applied". Per ADR-060 §D8 and EC-11-094,
this is the correct behavior for the partial-page path. The FULL-page path (page_record_count >=
page_size, indicating more data exists) is exercised by large live-sensor datasets and covered
in the S-CLAROTY-VULNS-001 holdout (HS-024) which depends on this story.

**BDD supplement:**

**Given** prism is built from the S-ENGINE-LIMIT-EARLY-STOP-001 story branch
**And** prism MCP stdio is started with the Claroty DTU configured (10 alerts records, page_size=1000)
**When** `SELECT * FROM claroty.alerts LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** `query_context.returned_results` equals `1`
**And** `query_context.is_truncated` equals `false`
**And** the `rows` array contains exactly 1 row object

---

## Setup Instructions

1. Confirm prism is built from the S-ENGINE-LIMIT-EARLY-STOP-001 story branch HEAD commit.

2. Start the Claroty DTU (prism-dtu-claroty) locally. It serves `POST /api/v1/alerts` with
   10 fixture records from `crates/prism-dtu-claroty/fixtures/alerts.json`.

3. Start prism in MCP stdio mode with the claroty sensor spec configured to point to the local
   DTU instance. Capture full MCP stdio output and stderr.

4. Wait for prism to be ready (startup completion log message or first JSON-RPC prompt).

5. Issue the MCP `query` tool call:
   `{"sql": "SELECT * FROM claroty.alerts LIMIT 1"}`.
   Note: the `LIMIT 1` is part of the SQL query text, NOT the MCP tool `limit` parameter.

6. Capture the full raw wire-level JSON response, including the complete `query_context` object.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.002 | §Postconditions LIMIT-Aware Early-Stop: `FetchContext.early_stop_limit` wired from `params.limit`; check placed IMMEDIATELY AFTER DI-019 in `execute_impl` | Wiring chain: SQL LIMIT 1 → FetchContext → execute_impl check → FetchOutput |
| BC-2.16.002 | §Postconditions LIMIT-Aware Early-Stop: OffsetLimit pagination mode; `active_page_size = page_size` from TOML | Mode-scope: claroty.alerts uses offset_limit with page_size=1000 |
| BC-2.16.002 | §Postconditions LIMIT-Aware Early-Stop: `truncated` field NOT set on early-stop (reserved for DI-019); early-stop is a SUCCESS-PATH non-error exit | `pipeline_truncated = false` in FetchOutput (DI-019 cap not hit) |
| BC-2.11.001 | §Edge Cases EC-11-094: partial-final-page → `early_stopped = false` → `is_truncated = false` (via is_truncated formula: `(total_rows > limit) OR any_early_stopped`) | `is_truncated: false` in wire response |

---

## Verification Approach

1. Parse the wire-level JSON response from the MCP `query` tool call.

2. If the response is an error object: record FAIL with observation "query returned error;
   check FetchContext wiring and execute_impl early-stop check (BC-2.16.002 §Postconditions
   LIMIT-Aware Early-Stop)."

3. Count the entries in the `rows` array. Assert count equals 1. If count != 1 (e.g., count=10):
   record FAIL on "DataFusion LIMIT applied" dimension — DataFusion SQL LIMIT 1 did not trim
   the fetched records.

4. Locate `query_context.returned_results`. Assert `returned_results == 1`.

5. Locate `query_context.is_truncated`. Assert `is_truncated == false`.
   If `is_truncated == true`: record PARTIAL (not FAIL) — the early-stop wiring may be setting
   `any_early_stopped = true` unconditionally (discriminator defect; this is HS-001's primary
   assertion dimension); note in report that HS-001 should be inspected.

6. The response should complete within 5 seconds. An unusually slow response (> 10 seconds)
   may indicate the pipeline is fetching unnecessary pages (early-stop check not wired), which
   is the DEFECT-2 motivation. Note the execution time from `query_context.execution_time_ms`.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query succeeds (no error)** (weight: 0.25): Non-error response with ≥1 row?
  Full credit (1.0): non-error response.
  Zero credit (0.0): error response.

- **`returned_results = 1` (DataFusion LIMIT applied correctly)** (weight: 0.35): Does
  `query_context.returned_results` equal 1?
  Full credit (1.0): `returned_results = 1`.
  Partial credit (0.5): `returned_results > 1` but < 10 (LIMIT partially applied).
  Zero credit (0.0): `returned_results = 10` (LIMIT 1 not applied) or error.

- **`is_truncated = false` (partial-page path: discriminator correct)** (weight: 0.25):
  Full credit (1.0): `is_truncated: false`.
  Partial credit (0.5): `is_truncated: true` (discriminator defect — see HS-001 for primary
  coverage of this dimension).
  Zero credit (0.0): `is_truncated` field absent.

- **Completes in reasonable time** (weight: 0.15): Does `query_context.execution_time_ms`
  indicate completion within 5000ms?
  Full credit (1.0): `execution_time_ms <= 5000`.
  Partial credit (0.5): `execution_time_ms` between 5000–10000ms.
  Zero credit (0.0): `execution_time_ms > 10000` or timeout (suggests excess page fetching).

---

## Edge Conditions

- **DTU not running:** Record as SETUP-FAILURE.

- **`returned_results = 10` (LIMIT 1 not applied):** This indicates DataFusion SQL LIMIT is
  not being applied. Record FAIL on "DataFusion LIMIT applied" dimension.

- **`is_truncated = true` with `returned_results = 1`:** The discriminator is broken (always
  setting `early_stopped = true`). Record PARTIAL here and note that HS-001 covers this
  dimension with higher weight.

- **Response time > 10 seconds on DTU:** Suspicious — the DTU serves 10 records in-process
  with no real latency. An unusually slow response indicates the pipeline is making unnecessary
  extra HTTP requests. Record in evaluation notes.

- **`returned_results = 0`:** The early-stop check or DataFusion LIMIT produced an empty
  result where 1 row was expected. Record FAIL.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-EARLY-STOP-001-002 (satisfaction: X.XX) — LIMIT-1 query end-to-end gap; check SQL LIMIT → params.limit → FetchContext.early_stop_limit wiring in SpecDrivenSensorAdapter::fetch and DataFusion LIMIT application (BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop ADR-060 §D8.1)"`

Do NOT disclose: the SQL LIMIT value used, the fixture row count, or the exact timing threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty DTU (prism-dtu-claroty) — `POST /api/v1/alerts`, 10 records |
| corpus_size | SQL LIMIT 1; 1 row in response; 10 rows fetched from API in one partial page |
| known_edge_cases | `returned_results = 10` (DataFusion LIMIT not applied); `is_truncated = true` (discriminator defect — see HS-001) |
| false_positive_threshold | Zero: `returned_results = 1` is an unambiguous assertion |
| false_negative_threshold | Minimal: if wiring is broken the query may return an error or wrong row count |

**Known-good corpus:** Claroty DTU with LIMIT 1 query — expected: 1 row, `is_truncated: false`,
sub-second completion.

**Known-problematic corpus:** An implementation where `params.limit` is not mapped to
`FetchContext.early_stop_limit` — for the DTU, the result is the same (partial page ensures
no over-fetching). The primary test for "early-stop prevents over-fetching" on large datasets
is covered by S-CLAROTY-VULNS-001 HS-024 (which depends on this story landing).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | s-engine-limit-early-stop-001-holdout-authoring | 2026-08-30 | product-owner | Initial authoring. HS-030 group for S-ENGINE-LIMIT-EARLY-STOP-001. End-to-end LIMIT 1 wiring test: SQL LIMIT 1 → FetchContext.early_stop_limit = Some(1) → execute_impl check → partial page → is_truncated=false → returned_results=1. BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop (ADR-060 §D8.1 wiring) + BC-2.11.001 EC-11-094. SINGLE-USE. |
