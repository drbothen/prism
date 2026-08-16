---
document_type: holdout-scenario
level: L3
id: "HS-AUDITLOG-001-A-002"
title: "Claroty audit_logs SELECT without time filter returns bounded rows from last 7 days only"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "defects-and-drift"
story_source: "S-CLAROTY-AUDITLOG-TIMEBOX-001"
version: "1.0"
status: consumed
producer: product-owner
timestamp: "2026-08-15T00:00:00Z"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
input-hash: "9fe4d01"
traces_to: "BC-2.16.013"
behavioral_contracts:
  - BC-2.16.013
verification_properties: []
lifecycle_status: consumed
introduced: DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001
last_evaluated: "2026-08-15"
last_eval_satisfaction: 1.0
staleness_check: null
stale_reason: null
retired: null
assumption_source: "ASM-CLAROTY-AUDITLOG-001"
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-CLAROTY-AUDITLOG-TIMEBOX-001 — Layer 1 bounded window scope validation. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-AUDITLOG-001-A-002: Claroty audit_logs SELECT without time filter returns bounded rows from last 7 days only

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-AUDITLOG-TIMEBOX-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.013 §Postconditions §1 Claroty `audit_logs` bullet — Default bounded look-back (Layer 1); EC-016-013-010 (known truncation edge)
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that an unbounded `SELECT *` against `claroty.audit_logs` returns rows
only from the last 7 days — not the entire audit history — confirming that the Layer 1 `body_template`
effectively bounds the xDome request. This is the complement to HS-AUDITLOG-001-A-001: rather than
testing that the query completes promptly (absence of E-QUERY-004), this test verifies the scope of
the bounded result.

The DTU clone is seeded with two groups of audit log fixtures:
- **Recent group (within 7 days):** Records timestamped within the last 7 days of DTU-simulated time.
- **Old group (beyond 7 days):** Records timestamped more than 7 days before DTU-simulated "now".

After Layer 1, only the recent group should be returned. The old records exist in the DTU fixture
but should not appear in the response because xDome's `after_seconds_ago: 604800` filter pre-filters
server-side.

**Behavioral assertions:**

1. The Claroty DTU clone is running with two distinct groups of audit log fixtures:
   - 5 records with timestamps `≤ now − 604800s` (recent, within 7 days)
   - 5 records with timestamps `> now − 604800s − any_delta` (old, beyond 7 days; the exact age
     must be > 7 days in DTU-simulated time so the `after_seconds_ago` filter excludes them)
2. The evaluator sends a `query` MCP tool call: `SELECT * FROM claroty.audit_logs LIMIT 20`
   (no explicit time filter in the WHERE clause)
3. The response JSON contains between 1 and 10 rows (the recent group only — not the old group)
4. The row timestamps in the response are all within the 7-day window relative to DTU "now"
5. The response does NOT contain timestamps from the old group
6. The query completes within 5 seconds (cross-validation with HS-AUDITLOG-001-A-001)

**BDD supplement:**

**Given** the Claroty DTU has audit_log fixtures split between recent (last 7d) and old (>7d ago) records  
**When** `SELECT * FROM claroty.audit_logs LIMIT 20` is issued with no time filter  
**Then** only recent records (within 7 days) appear in the result  
**And** old records are not returned (server-side pre-filtered by `after_seconds_ago`)  
**And** the query completes within 5 seconds

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.013 | §Postconditions §1 Claroty `audit_logs` — Default bounded look-back (Layer 1) | Core assertion: result set bounded to last 7 days |
| BC-2.16.013 | EC-016-013-010 (known truncation at Layer 1) | Confirms the truncation is observable via the DTU fixture split |

---

## Verification Approach

1. Start the Claroty DTU clone with seeded fixtures. The DTU `audit_log` handler must support
   filtering based on the `after_seconds_ago` filter in the request body. Seed two record groups:
   - Group A (recent): 5 records with `timestamp` values within DTU-simulated "now − 604800s"
     (last 7 days). Use unique identifiers in these records (e.g., `audit_id` prefix `"recent-"`).
   - Group B (old): 5 records with `timestamp` values older than `now − 604800s − 86400s` (older
     than 8 days, safely outside the 7-day window). Use `audit_id` prefix `"old-"`.

2. Start prism MCP stdio with the Claroty sensor pointing at the DTU clone.

3. Send `query` tool call: `{"sql": "SELECT * FROM claroty.audit_logs LIMIT 20"}`.

4. Capture the serialized JSON response.

5. Assert:
   - Response is valid JSON and not an error envelope
   - The JSON contains between 1 and 10 data rows (only Group A; Group B excluded)
   - None of the returned rows contain an `audit_id` starting with `"old-"` (or equivalent
     fixture identifier for Group B records)
   - Query completes in < 5 seconds
   - Response does not contain `"E-QUERY-004"` or `"timed out"`

6. If the DTU returns both groups (all 10 records), report FAIL: the bounded `body_template`
   is not being applied or the DTU does not respect the `after_seconds_ago` filter.

7. If the DTU returns HTTP 400 for the `after_seconds_ago` filter body (ASM-CLAROTY-AUDITLOG-001
   assumption incorrect), record as SETUP-FAILURE and escalate.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Scope correctness** (weight: 0.5): Does the result contain only records from within the 7-day window?
  Full credit (1.0): only recent records present, old records absent.
  Half credit (0.5): recent records present but old records also present (filter not applied).
  Zero credit (0.0): no records returned at all (cannot distinguish from fixture setup failure).

- **Timing** (weight: 0.3): Did the query complete in under 5 seconds?
  Full credit (1.0): ≤ 5s. Zero credit (0.0): > 15s.

- **Error absence** (weight: 0.2): Does the response avoid E-QUERY-004 and timeout language?
  Full credit (1.0): clean response, no error codes. Zero credit (0.0): error present.

---

## Edge Conditions

- **DTU does not implement filter_by semantics:** If the DTU's audit_log handler ignores the
  `filter_by` body and returns all records regardless, both Group A and Group B appear — this is a
  FAIL (the Layer 1 fix is ineffective against the real DTU behavior). Evaluator should check
  whether the DTU handler processes the `filter_by` field.

- **Fixture timestamps outside DTU time simulation:** Ensure Group B timestamps are genuinely
  older than `now − 604800s` in the DTU's simulated time, not just older than wall-clock time.

- **ASM-CLAROTY-AUDITLOG-001 validation:** If the DTU returns HTTP 400 for the `after_seconds_ago`
  filter, the assumption is wrong — escalate immediately (SETUP-FAILURE).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-AUDITLOG-001-A-002 (satisfaction: X.XX) — claroty.audit_logs SELECT returned records outside the expected bounded window or included records that should have been filtered by the server-side time constraint; verify body_template injection and DTU filter handling"`

Do NOT disclose: the specific fixture structure, the `audit_id` naming convention, the exact
window size, or which fixture group was/wasn't returned.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001-po-bc-amendments | 2026-08-15 | product-owner | Initial authoring. Story-level holdout gate for S-CLAROTY-AUDITLOG-TIMEBOX-001 Layer 1 — bounded window scope validation. SINGLE-USE. |
