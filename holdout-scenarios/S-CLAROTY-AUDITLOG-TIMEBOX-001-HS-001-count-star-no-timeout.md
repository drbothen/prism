---
document_type: holdout-scenario
level: L3
id: "HS-AUDITLOG-001-A-001"
title: "Claroty audit_logs COUNT(*) returns promptly with bounded rows — no E-QUERY-004"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "defects-and-drift"
story_source: "S-CLAROTY-AUDITLOG-TIMEBOX-001"
version: "1.0"
status: active
producer: product-owner
timestamp: "2026-08-15T00:00:00Z"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
input-hash: null
traces_to: "BC-2.16.013"
behavioral_contracts:
  - BC-2.16.013
verification_properties: []
lifecycle_status: active
introduced: DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: "ASM-CLAROTY-AUDITLOG-001"
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-CLAROTY-AUDITLOG-TIMEBOX-001 — Layer 1 bounded look-back validation. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-AUDITLOG-001-A-001: Claroty audit_logs COUNT(*) returns promptly with bounded rows — no E-QUERY-004

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-AUDITLOG-TIMEBOX-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.013 §Postconditions §1 Claroty `audit_logs` bullet — Default bounded look-back (Layer 1)
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the Layer 1 fix (bounded `body_template`) eliminates the E-QUERY-004
timeout on Claroty audit_logs queries. A `COUNT(*)` query against `claroty.audit_logs` must complete
promptly and return a row count — not hang for 30 seconds and fail with a timeout error.

The scenario exercises the primary symptom of the defect: any query against `claroty.audit_logs`
without an explicit time filter would previously trigger a 30-second client timeout because the
xDome `POST /api/v1/audit_log/get` endpoint returned the entire audit history for an empty body.
Layer 1 fixes this by injecting a default 7-day look-back into the request body.

**LIVE-API ASSUMPTION (ASM-CLAROTY-AUDITLOG-001):** The Layer 1 `body_template` assumes that
`after_seconds_ago` is a valid xDome `GetAuditLogParameters.filter_by` operation. The evaluator
observes bounded behavior (prompt return, row count > 0 or row count = 0 with no error); the
specific filter wire format is confirmed by the DTU clone's response behavior. If the DTU clone
returns HTTP 400 or an error response for the injected body, record that as a SETUP-FAILURE finding
(assumption mismatch) and escalate — do NOT mark it as a PASS or behavioral finding.

**Behavioral assertions:**

1. The Claroty DTU clone is running on localhost with at least 10 audit log fixture records seeded
   in the time window corresponding to the last 7 days of DTU-simulated time.
2. The prism binary is configured to point at the DTU clone at the correct base URL.
3. The evaluator sends a `query` MCP tool call with SQL: `SELECT COUNT(*) FROM claroty.audit_logs`
   (or pipe-mode equivalent: `FROM claroty.audit_logs | LIMIT 1`)
4. The query completes within 5 seconds (not 30s timeout).
5. The serialized JSON response does NOT contain `"E-QUERY-004"` in any field.
6. The serialized JSON response does NOT contain `"timed out"` or `"timeout"` (case-insensitive).
7. The serialized JSON response IS valid JSON (not an error envelope).
8. The result set contains a numeric count value (for COUNT(*)) or at least one row (for the LIMIT 1
   variant) — the query returns data, not an empty error.

**BDD supplement:**

**Given** the Claroty DTU clone is running with audit log fixtures seeded for the last 7 days  
**And** prism is configured with the Layer 1 TOML spec (bounded `body_template`)  
**When** `SELECT COUNT(*) FROM claroty.audit_logs` is issued via the MCP `query` tool  
**Then** the MCP response arrives within 5 seconds  
**And** the response does not contain `E-QUERY-004` or any timeout indication  
**And** the response contains a valid numeric count

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.013 | §Postconditions §1 Claroty `audit_logs` bullet — Default bounded look-back (Layer 1): `body_template` injects `after_seconds_ago: 604800` | Core assertion: query completes promptly, no E-QUERY-004 |
| BC-2.16.013 | §Edge Cases EC-016-013-010 | Background: silent truncation is acceptable at Layer 1; the scenario does NOT test truncation (that is tested in HS-AUDITLOG-001-A-002) |

---

## Verification Approach

1. Start the Claroty DTU clone: construct `ClarotyClone` and call `BehavioralClone::start_on("127.0.0.1:0", ...)`;
   capture the bound address. Ensure the clone has audit_log fixture records seeded within a 7-day
   simulated window (the DTU `GET /api/v1/audit_log/get` handler must return at least 1 record
   when the `after_seconds_ago: 604800` filter is applied).

2. Start the prism binary in MCP stdio mode with the Claroty sensor configured to point at the DTU
   clone (base_url = `http://127.0.0.1:PORT`). The bearer token can be any non-empty placeholder.

3. Record the wall-clock time `t0`.

4. Over MCP stdio, send a `query` tool call with:
   `{"sql": "SELECT COUNT(*) FROM claroty.audit_logs"}` (adjust to the deployed query tool schema).

5. Record wall-clock time `t1` when the MCP response arrives.

6. Assert:
   - `(t1 - t0) < 5.0 seconds` — prompt completion
   - The full serialized JSON response bytes do NOT contain `"E-QUERY-004"` (substring match)
   - The full serialized JSON response bytes do NOT contain `"timed out"` or `"timed_out"` (case-insensitive)
   - The response is valid JSON (parse without error)
   - The response contains a numeric result row (either `[{"count(*)": N}]` or similar, N >= 0)

7. If the DTU returns an error for the Layer 1 `body_template` (e.g., HTTP 400 or a JSON error body
   containing `"invalid operation"`), record as SETUP-FAILURE (ASM-CLAROTY-AUDITLOG-001 assumption
   incorrect) and escalate to the orchestrator — do NOT mark as PASS or behavioral FAIL.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **Timing** (weight: 0.5): Did the query complete in under 5 seconds?
  Full credit (1.0): response arrived in ≤ 5s. Zero credit (0.0): response arrived after 30s or timed out.
  Partial credit (0.5): response arrived between 5s and 15s (unusual; could indicate partial improvement).

- **Error absence** (weight: 0.35): Does the response avoid E-QUERY-004 and timeout language?
  Full credit (1.0): no E-QUERY-004, no "timed out", response is valid JSON.
  Zero credit (0.0): E-QUERY-004 present OR "timed out" present OR invalid JSON.

- **Result quality** (weight: 0.15): Does the response contain meaningful data (count or rows)?
  Full credit (1.0): response contains a numeric count or at least one data row.
  Zero credit (0.0): response is an empty array or null result with no explanation.

---

## Edge Conditions

- **Zero audit log records in DTU within 7-day window:** If the DTU returns an empty result (0 rows)
  because no fixtures fall within `after_seconds_ago: 604800`, the query still PASSES — it returns
  promptly with count=0 and no error. This is correct bounded behavior. Ensure the DTU fixture has
  at least one record within the 7-day window to make the test more diagnostic.

- **DTU returns HTTP 400 for `after_seconds_ago` filter:** This is an assumption failure
  (ASM-CLAROTY-AUDITLOG-001). Record as SETUP-FAILURE, not a behavioral finding.

- **Connection error during test setup:** If the DTU fails to start, retry once. If it fails again,
  report SETUP-FAILURE with the DTU startup log.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-AUDITLOG-001-A-001 (satisfaction: X.XX) — claroty.audit_logs COUNT(*) did not return promptly or returned E-QUERY-004; verify that the audit_logs body_template in claroty.sensor.toml injects a bounded look-back filter"`

Do NOT disclose: the specific timing threshold, the expected filter format, the DTU fixture structure,
or the exact assertion that failed.

---

## Category: real-world-corpus

This scenario is grounded in the production behavior observed during DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001:
the live xDome API returned an unbounded audit history for an empty-body `POST /api/v1/audit_log/get`,
causing every Claroty audit_log query to timeout at 30 seconds with E-QUERY-004.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API at `api.claroty.com` — production MSSP deployment; simulated via `prism-dtu-claroty` DTU clone with audit_log fixtures |
| corpus_size | Single COUNT(*) query; evaluates the request body format and response latency |
| known_edge_cases | Empty body → full history (pre-fix); bounded body → 7-day window (post-fix); ASM-CLAROTY-AUDITLOG-001 validation |
| false_positive_threshold | Zero: a query completing in < 5s is a real improvement |
| false_negative_threshold | Zero: E-QUERY-004 on a COUNT(*) is a clear regression |

**Known-good corpus:** Claroty DTU with Layer 1 `body_template` — expected result: COUNT(*) completes
in < 5s, count ≥ 0, no error. Tests that the bounded default works.

**Known-problematic corpus:** Claroty DTU with old `body_template: '{}'` — expected result: 30s hang
then E-QUERY-004. Tests that the prior defect is not reintroduced. (Evaluator may optionally test this
regression path by temporarily using `body_template = '{}'` in the TOML during evaluation setup.)

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001-po-bc-amendments | 2026-08-15 | product-owner | Initial authoring. Story-level holdout gate for S-CLAROTY-AUDITLOG-TIMEBOX-001 Layer 1 — COUNT(*) no-timeout validation. SINGLE-USE. |
