---
document_type: holdout-scenario
level: L3
id: "HS-EARLY-STOP-001-003"
title: "Plan-shape gate: COUNT(*) aggregate query returns full count without early-stop interference (ADR-060 §D8.7 Condition A)"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-ENGINE-LIMIT-EARLY-STOP-001 (HS-030 group). Tests the Plan-Shape Gate (ADR-060 §D8.7 Condition A): a SELECT COUNT(*) aggregation query is identified as a reducing plan by ast_is_reducing_plan(), fetch_limit is set to None (early-stop suppressed), and the full dataset is fetched and aggregated correctly. Against Claroty DTU (10 alerts records): COUNT(*) must return 10, not a partial count. This directly validates that the gate function is wired in materialization.rs and that it correctly suppresses early-stop for aggregate functions per Condition A. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-EARLY-STOP-001-003: Plan-shape gate: COUNT(*) aggregate query returns full count without early-stop interference (ADR-060 §D8.7 Condition A)

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ENGINE-LIMIT-EARLY-STOP-001 (HS-030 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop Pagination (ADR-060 §D8) —
Plan-Shape Gate (ADR-060 §D8.7): `ast_is_reducing_plan()` function in `materialization.rs`;
Condition A — presence of aggregate functions (COUNT, SUM, AVG, etc.) → `is_reducing_plan = true`
→ `fetch_limit = None` (early-stop suppressed; full dataset fetched before aggregation).
**Gate:** Story-level holdout gate (HS-030) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **Plan-Shape Gate** (ADR-060 §D8.7) for aggregate queries.
The gate suppresses early-stop for reducing plans to prevent partial aggregates.

A `SELECT COUNT(*) FROM claroty.alerts` query:
1. Has an aggregate function (COUNT) → `ast_is_reducing_plan()` returns `true` (Condition A).
2. The gate sets `fetch_limit = None` (overriding the SQL `LIMIT` if any) → `FetchContext.early_stop_limit = None`.
3. The full dataset is fetched: all 10 records from the Claroty DTU alerts fixture.
4. DataFusion computes COUNT(*) = 10 over all 10 records.
5. The response contains 1 row with a count value of 10.

**The defect this scenario catches at large scale:** Without the Plan-Shape Gate, a
`SELECT COUNT(*) FROM claroty_vulnerabilities LIMIT 1` query would set
`FetchContext.early_stop_limit = Some(1)` → stop after the first page (1000 records) → return
COUNT = 1000 instead of the true total count. The gate prevents this by suppressing early-stop
for all reducing plans (Conditions A–K in ADR-060 §D8.7, conservative default).

**For the DTU (10 records, page_size=1000):** The gate suppression has no observable impact
because all 10 records always fit in one partial page regardless of early-stop. However, this
scenario verifies that the COUNT(*) aggregation works correctly end-to-end and that the gate
does not accidentally prevent the query from executing (gate wiring bug where it blocks all
queries rather than selectively suppressing early-stop).

**Two assertions in this scenario:**

**Part A — COUNT(*) without SQL LIMIT:**
- `SELECT COUNT(*) FROM claroty.alerts`
- Expected: 1 aggregate row, COUNT(*) value = 10, `is_truncated: false`

**Part B — COUNT(*) combined with SQL LIMIT 1 (gate must suppress early-stop):**
- `SELECT COUNT(*) FROM claroty.alerts LIMIT 1`
- With gate WORKING: `fetch_limit = None`, COUNT(*) aggregates all 10 rows, result is
  {count=10}. SQL `LIMIT 1` then returns this single aggregate row. Response: 1 row, count=10.
- With gate BROKEN: `fetch_limit = Some(1)`, early-stop fires after 10 records (partial page,
  `early_stopped=false`), COUNT(*) still gets 10 records (same result for DTU). BUT on large
  sensors, the gate failure would return a wrong count.
- For DTU: both paths return the same result. This part documents the intended behavior for
  evaluator awareness.

**BDD supplement (Part A):**

**Given** prism is built from the S-ENGINE-LIMIT-EARLY-STOP-001 story branch
**And** prism MCP stdio is started with the Claroty DTU configured (10 alerts records)
**When** `SELECT COUNT(*) FROM claroty.alerts` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the `rows` array contains exactly 1 row
**And** the single returned row contains a numeric count column with value 10
**And** `query_context.is_truncated` equals `false`

---

## Setup Instructions

1. Confirm prism is built from the S-ENGINE-LIMIT-EARLY-STOP-001 story branch HEAD commit.

2. Start the Claroty DTU (prism-dtu-claroty) locally. Serves 10 records at `POST /api/v1/alerts`.

3. Start prism in MCP stdio mode with the claroty sensor spec configured to point to the local
   DTU instance. Capture full MCP stdio output and stderr.

4. Wait for prism to be ready (startup completion log message or first JSON-RPC prompt).

5. Issue Part A query via MCP `query` tool:
   `{"sql": "SELECT COUNT(*) FROM claroty.alerts"}`.
   Capture the full raw wire-level JSON response.

6. Issue Part B query via MCP `query` tool:
   `{"sql": "SELECT COUNT(*) FROM claroty.alerts LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.002 | §Postconditions LIMIT-Aware Early-Stop: Plan-Shape Gate (ADR-060 §D8.7) — `ast_is_reducing_plan()` Condition A: aggregate functions | Part A: COUNT(*) query is identified as reducing → `fetch_limit = None` |
| BC-2.16.002 | §Postconditions LIMIT-Aware Early-Stop: `where_filters` NOT forwarded to gate; gate performs own AST inspection | Gate must work without where_filters; pure structural aggregate detection |
| BC-2.16.002 | §Postconditions LIMIT-Aware Early-Stop: EC-016-002-001..018 cover Plan-Shape Gate suppression conditions | Condition A (aggregate) is a named suppression condition; gate wiring must be in materialization.rs |
| BC-2.11.001 | §Edge Cases EC-11-093: materialization returns full set without applying tool-level cap internally; engine Step 6 sole owner of cap | COUNT(*) aggregation over full 10-record dataset; no pre-cap in materialization |

---

## Verification Approach

**Part A verification:**

1. Parse the wire-level JSON response from Part A.

2. If the response is an error: record FAIL on "COUNT(*) executes" dimension.

3. Count the entries in the `rows` array. Assert count equals 1 (a COUNT(*) aggregate is
   a single-row result). If `rows.length != 1`: record FAIL.

4. Inspect the single row. Find the numeric column (likely named `count(*)` or `COUNT(*)`
   or `count` — the exact DataFusion output column name varies). Assert the numeric value
   equals 10. If value != 10: record FAIL on "correct aggregate count" dimension.

5. Assert `query_context.is_truncated == false`.

**Part B verification:**

6. Parse the wire-level JSON response from Part B.

7. Assert Part B also produces 1 row with count value = 10. The `LIMIT 1` in the SQL applies
   to the RESULT rows, not the input rows; since COUNT(*) returns 1 result row, `LIMIT 1` is
   a no-op on the aggregate output.

8. Assert `query_context.is_truncated == false`.

9. Note: For the DTU (10 records, partial pages), both gate-present and gate-absent
   implementations produce count=10 for Part B. This is a known limitation — the gate impact
   on partial-page DTU data is not observable at this scale. If Parts A and B both pass, the
   end-to-end aggregation path is verified. The gate's correctness on large multi-page datasets
   is validated separately against live sensor data.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Part A succeeds (COUNT(*) executes without error)** (weight: 0.20):
  Full credit (1.0): non-error response with 1 row.
  Zero credit (0.0): error response or 0 rows.

- **COUNT(*) value = 10 (full dataset aggregated)** (weight: 0.35): Does the aggregate
  row contain a count value equal to 10?
  Full credit (1.0): count = 10.
  Partial credit (0.3): count > 0 but != 10 (partial dataset aggregated — indicates a
  pagination problem, though unlikely with 10-record DTU).
  Zero credit (0.0): count = 0 or count field absent.

- **`is_truncated = false` (aggregate is not truncated)** (weight: 0.20): 
  Full credit (1.0): `is_truncated: false`.
  Zero credit (0.0): `is_truncated: true` (the aggregate result is not truncated; an
  aggregate row is a final computation result, not a subset of raw rows).

- **Part B: COUNT(*) LIMIT 1 also returns count=10** (weight: 0.25): Does Part B
  (with SQL `LIMIT 1`) also return count=10 in the single aggregate row?
  Full credit (1.0): Part B returns 1 row, count=10, `is_truncated: false`.
  Partial credit (0.5): Part B returns non-error but count != 10.
  Zero credit (0.0): Part B returns error or 0 rows.

---

## Edge Conditions

- **COUNT(*) column name differs** (e.g., `count(claroty.claroty_alerts.*)` vs `count(*)`):
  The exact column name produced by DataFusion for COUNT(*) may vary. Accept any single
  numeric column with value 10 — do not fail on column naming alone.

- **Part A returns 0 rows:** The aggregation table registration may have failed (sensor or
  table not loaded). Record as FAIL (not SETUP-FAILURE — the alert table is part of the
  base claroty spec).

- **Part B count differs from Part A count:** If Part A returns count=10 and Part B returns
  count=1, this indicates `LIMIT 1` was applied to the INPUT rows (not the aggregate output),
  which would be a DataFusion query compilation issue. Record FAIL on Part B.

- **DTU not running:** Record as SETUP-FAILURE for both parts.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-EARLY-STOP-001-003 (satisfaction: X.XX) — aggregate query (plan-shape gate) gap; COUNT(*) result incorrect or query errors; check ast_is_reducing_plan() function wiring in materialization.rs (ADR-060 §D8.7 Condition A — aggregate functions) and that COUNT(*) executes over the full fetched dataset (BC-2.11.001 EC-11-093 sole-owner rule + BC-2.16.002 §Postconditions Plan-Shape Gate)"`

Do NOT disclose: the expected count value, the SQL queries used, or the fixture row count.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty DTU (prism-dtu-claroty) — `POST /api/v1/alerts`, 10 records |
| corpus_size | COUNT(*) aggregate — 1 result row regardless of input count |
| known_edge_cases | COUNT(*) column name variance in DataFusion output; `LIMIT 1` applied to aggregate output vs input rows |
| false_positive_threshold | Zero: count value = 10 is an unambiguous assertion |
| false_negative_threshold | Low: gate failure on large datasets (not detectable with 10-record DTU); live sensor coverage deferred to S-CLAROTY-VULNS-001 |

**Known-good corpus:** Claroty DTU with 10 alerts records — expected: COUNT(*) = 10, 1 aggregate
row, `is_truncated: false` for both Part A (no SQL LIMIT) and Part B (SQL LIMIT 1 applied to
aggregate output).

**Known-problematic corpus:** An implementation where `ast_is_reducing_plan()` is not wired
and the gate is absent — for live sensors with thousands of records, COUNT(*) LIMIT N would
return N instead of the true total count. For the DTU (10 records, all in one partial page),
the result is the same regardless of gate presence.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | s-engine-limit-early-stop-001-holdout-authoring | 2026-08-30 | product-owner | Initial authoring. HS-030 group for S-ENGINE-LIMIT-EARLY-STOP-001. Plan-Shape Gate test (ADR-060 §D8.7 Condition A): SELECT COUNT(*) on claroty.alerts (10-record DTU) returns count=10 (Part A: no SQL LIMIT; Part B: SQL LIMIT 1 applied to aggregate output). Gate suppresses early-stop for aggregate functions. BC-2.16.002 §Postconditions Plan-Shape Gate + BC-2.11.001 EC-11-093 (full-set contract). SINGLE-USE. |
