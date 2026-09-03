---
document_type: holdout-scenario
level: L3
id: "HS-SORTBY-001-002"
title: "claroty_audit_logs fetch_step body_template: filter_by preserved AND sort_by added — coexistence invariant, 7-day window not regressed, timestamp fallback acceptable"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-DEFECTS"
story_source: "DEFECT-CLAROTY-SORTBY-DETERMINISM-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-09-02T00:00:00Z"
modified: "2026-09-02"
phase: 3
inputs:
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
input-hash: "[pending-recompute]"
traces_to: "BC-2.16.013"
behavioral_contracts:
  - BC-2.16.013
verification_properties: []
lifecycle_status: active
introduced: "DEFECT-CLAROTY-SORTBY-DETERMINISM-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for DEFECT-CLAROTY-SORTBY-DETERMINISM-001 (HS-031 group). Tests that the audit_logs fetch step body_template was EXTENDED (sort_by added) not REPLACED (filter_by dropped). The highest-risk failure mode for audit_logs is an implementer accidentally replacing the filter_by injection with sort_by, eliminating the 7-day time-window guard established by S-CLAROTY-AUDITLOG-TIMEBOX-001. EC-016-013-011 offset-pagination determinism. Timestamp-only fallback acceptable if id sort field was rejected by live API. Test-writer and implementer MUST NOT read this file."
---

# HS-SORTBY-001-002: claroty_audit_logs fetch_step body_template: filter_by preserved AND sort_by added — coexistence invariant, 7-day window not regressed, timestamp fallback acceptable

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-CLAROTY-SORTBY-DETERMINISM-001 (HS-031 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.013 §Postconditions §1 audit_logs sort-by postcondition (within the
bounded filter_by push-down block); EC-016-013-011 offset-pagination determinism; the
coexistence invariant that sort_by EXTENDS the existing template (never replaces filter_by).
**Gate:** Story-level holdout gate (HS-031) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the audit_logs body_template was extended (both `filter_by` AND
`sort_by` present) rather than erroneously replaced (sort_by present, filter_by absent).

The highest-risk failure mode for this table is an implementer replacing the existing
`filter_by` injection with `sort_by`. If `filter_by` is removed, the audit_logs query
sends NO time-window to the API, returning ALL audit events (potentially millions) instead
of the bounded 7-day window. This is a silent regression — the query succeeds but returns
a vastly larger result set with no error signal.

The evaluator reads `claroty.sensor.toml` and issues an MCP smoke query.

**Assertion set:**

1. The `body_template` field for the audit_logs fetch step contains BOTH the substring
   `"filter_by"` AND the substring `"sort_by"` within the same string literal. One key
   absent means the other replaced it (regression) rather than extending it (correct).

2. The `"filter_by"` key appears in the body_template alongside a variable interpolation
   placeholder (e.g., `${query.filter._claroty_audit_filter_by}` or equivalent). The key
   must reference a query variable that pushes the 7-day time-window filter to the API.
   The evaluator does NOT need to parse the variable name precisely — the presence of `$`
   or `${` adjacent to `filter_by` in the template is sufficient evidence.

3. The `body_template` contains `"sort_by"` with a value that includes `"timestamp"` as a
   sort field. This is the required base field for audit_log ordering. The evaluator asserts
   `"timestamp"` appears within the sort_by portion of the body_template.

4. **Fallback form is acceptable:** If the body_template contains `"sort_by"` with only
   `"timestamp"` and NO additional tiebreaker field (i.e., the id tiebreaker was omitted
   because live API validation showed it is rejected), this STILL PASSES assertions 1, 2,
   and 3 fully. The fallback (timestamp-only) is a deliberate architectural decision per
   a documented architectural decision — not a defect.

5. Issue MCP tool call `{"sql": "SELECT * FROM claroty.claroty_audit_logs LIMIT 1"}` against
   the DTU. The response must be non-error. In particular, the extended body_template must
   produce valid JSON that the DTU accepts without a 400/422.

**BDD supplement:**

**Given** the story branch is built and `crates/prism-sensors/specs/claroty.sensor.toml` is
present at story HEAD
**When** the evaluator reads the file and locates the fetch step for `claroty_audit_logs`
**Then** the `body_template` of that step contains the substring `"filter_by"`
**And** the `body_template` contains the substring `"sort_by"`
**And** the `body_template` contains `"timestamp"` within the sort_by portion
**And** the `body_template` contains `${` or `$` adjacent to the filter_by key (variable
interpolation present — 7-day window guard active)

**And Given** prism MCP stdio is started with the claroty sensor configured against the DTU
**When** `SELECT * FROM claroty.claroty_audit_logs LIMIT 1` is issued via MCP `query` tool
**Then** the response is non-error

---

## Setup Instructions

1. Confirm the story branch is checked out and the binary is built from HEAD.

2. Locate `crates/prism-sensors/specs/claroty.sensor.toml`.

3. Read the file and find the `[[tables.steps]]` block for the audit_logs fetch step (look
   for a `body_template` that previously contained only `filter_by`, or a step with a URL
   or name referencing audit_log data from the Claroty xDome API).

4. Inspect the `body_template` string. It is a TOML single-quoted string (`'...'`).

5. Start prism in MCP stdio mode with the Claroty DTU running.

6. Issue the MCP `query` tool call: `{"sql": "SELECT * FROM claroty.claroty_audit_logs LIMIT 1"}`.

7. Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.013 | §Postconditions §1 audit_logs sort-by postcondition: BOTH filter_by AND sort_by present | Assertion 1: coexistence of both keys in body_template |
| BC-2.16.013 | filter_by push-down preserved: variable interpolation active (7-day time-window guard) | Assertion 2: filter_by references a query variable ($ placeholder present) |
| BC-2.16.013 | sort_by array contains timestamp as base sort field | Assertion 3: "timestamp" in sort_by portion |
| BC-2.16.013 | EC-016-013-011 fallback form (timestamp-only) acceptable when id tiebreaker rejected by API | Assertion 4: fallback form PASSES fully |
| BC-2.16.013 | Extended body_template is valid JSON accepted by API | Assertion 5: MCP query non-error |

---

## Verification Approach

1. Use `Bash` to search the TOML file:
   ```bash
   grep -n 'filter_by\|sort_by\|audit_log' crates/prism-sensors/specs/claroty.sensor.toml
   ```
   Or use the `Read` tool to read the full file.

2. Locate the fetch step for audit_logs. Extract the `body_template` string.

3. Check that `"filter_by"` appears as a substring. Record PASS or FAIL.

4. Check that `"sort_by"` appears as a substring. Record PASS or FAIL.

5. Check that `${` or `$` appears within the body_template string adjacent to the filter_by
   clause (evidence that the variable interpolation for the 7-day window is still active).

6. Extract the sort_by array portion of the body_template (the substring between `"sort_by":`
   and the closing `]`). Verify `"timestamp"` appears as a field name within it.

7. Note whether an `"id"` field also appears in the sort_by array (preferred form) or only
   `"timestamp"` appears (fallback form). Both are PASS for assertion 4 — record which form
   is present for the evaluator's notes but do NOT score differently.

8. Run the MCP query. Record non-error (PASS) or error with message (FAIL).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **filter_by still present (regression prevention)** (weight: 0.40): Does the audit_logs
  body_template contain `"filter_by"` as a JSON key (not just as part of a variable name)?
  Full credit (1.0): `"filter_by"` present as JSON key.
  Zero credit (0.0): `"filter_by"` absent — the 7-day window guard was accidentally dropped.
  This is the MOST CRITICAL assertion — filter_by absence is a silent silent regression.

- **sort_by present** (weight: 0.25): Does the body_template contain `"sort_by"` as a JSON key?
  Full credit (1.0): present.
  Zero credit (0.0): absent — the fix was not applied to audit_logs.

- **timestamp in sort_by** (weight: 0.20): Does the sort_by portion of the body_template
  contain `"timestamp"` as a sort field?
  Full credit (1.0): present.
  Zero credit (0.0): absent.

- **Variable interpolation active for filter_by** (weight: 0.05): Does the body_template
  contain `${` or `$` adjacent to the filter_by clause (variable reference for time window)?
  Full credit (1.0): variable placeholder present.
  Zero credit (0.0): filter_by is a literal value rather than a variable reference (hardcoded
  or blank filter — 7-day guard is defeated).

- **MCP query non-error** (weight: 0.10): Does the MCP SELECT on claroty_audit_logs LIMIT 1
  return a non-error response?
  Full credit (1.0): non-error response.
  Zero credit (0.0): error response (sort_by addition broke the POST body JSON structure).

---

## Edge Conditions

- **DTU returns zero audit records:** SETUP-FAILURE on the MCP query assertion — not a
  behavioral FAIL. The structural TOML assertions are independent.

- **filter_by and sort_by both absent:** Each absence is scored independently. Both absent
  means overall satisfaction ≤ 0.15 (only MCP smoke test could partially pass).

- **filter_by key appears only in a comment or variable name, not as a JSON key:**
  The evaluator MUST confirm `"filter_by"` appears within the JSON object literal of the
  body_template, not as a TOML comment or step name. Use the surrounding `{` ... `}`
  context to distinguish.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-SORTBY-001-002 (satisfaction: X.XX) — claroty_audit_logs body_template coexistence gap; check that filter_by was NOT removed when sort_by was added to the audit_logs fetch step in claroty.sensor.toml — both keys MUST coexist (BC-2.16.013 §Postconditions §1 audit_logs sort-by postcondition + EC-016-013-011)"`

Do NOT disclose: the specific variable name expected, the LIMIT value used, or the exact
assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | `crates/prism-sensors/specs/claroty.sensor.toml` (shipped sensor spec) + live MCP query against Claroty DTU |
| corpus_size | Single TOML file + LIMIT 1 smoke query |
| known_edge_cases | filter_by accidentally replaced → FAIL (most critical regression); sort_by absent → FAIL; id tiebreaker absent (fallback form) → PASS (acceptable per BC-2.16.013 §Postconditions §1 fallback clause) |
| false_positive_threshold | Zero: filter_by + sort_by coexistence is a direct structural assertion |
| false_negative_threshold | Zero: if filter_by is absent, the 7-day window guard is silently broken (BC-2.16.013 §coexistence invariant violated) |

**Known-good corpus:** Story branch with sort_by correctly ADDED alongside existing filter_by
— expected: both keys present, `timestamp` in sort_by, variable interpolation active, MCP non-error.

**Known-problematic corpus:** Branch where filter_by was accidentally replaced with sort_by
(regression) — expected: `"filter_by"` absent from body_template; all unbounded audit events
returned; 0.00 on the filter_by dimension.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | defect-claroty-sortby-holdout-authoring | 2026-09-02 | product-owner | Initial authoring. HS-031 group for DEFECT-CLAROTY-SORTBY-DETERMINISM-001. Audit logs coexistence: filter_by must not be dropped when sort_by is added. BC-2.16.013 §Postconditions §1 audit_logs sort-by postcondition + EC-016-013-011. Fallback form (timestamp-only) accepted per BC-2.16.013 §Postconditions §1 fallback clause. SINGLE-USE. |
