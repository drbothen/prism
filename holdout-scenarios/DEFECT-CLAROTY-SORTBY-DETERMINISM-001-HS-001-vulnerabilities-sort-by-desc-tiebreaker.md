---
document_type: holdout-scenario
level: L3
id: "HS-SORTBY-001-001"
title: "claroty_vulnerabilities fetch_step body_template: sort_by present with adjusted_vulnerability_score DESC first, unique tiebreaker ASC second — DI-019 truncation retains highest-risk records"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-DEFECTS"
story_source: "DEFECT-CLAROTY-SORTBY-DETERMINISM-001"
version: "1.0"
status: active
used: true
single_use: true
producer: product-owner
timestamp: "2026-09-02T00:00:00Z"
modified: "2026-09-02"
phase: 3
inputs:
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - ".factory/specs/behavioral-contracts/BC-2.16.015-claroty-vulnerabilities-table.md"
input-hash: "[pending-recompute]"
traces_to: "BC-2.16.015"
behavioral_contracts:
  - BC-2.16.015
verification_properties: []
lifecycle_status: consumed
introduced: "DEFECT-CLAROTY-SORTBY-DETERMINISM-001"
last_evaluated: 2026-09-02
last_eval_satisfaction: 0.90
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for DEFECT-CLAROTY-SORTBY-DETERMINISM-001 (HS-031 group). Tests that the claroty.sensor.toml fetch_vulnerabilities body_template contains a sort_by array with adjusted_vulnerability_score DESC as the primary key and a provably-unique tiebreaker ASC second — guaranteeing (a) deterministic offset pagination, and (b) DI-019 truncation keeps highest-risk CVEs. Test-writer and implementer MUST NOT read this file."
---

# HS-SORTBY-001-001: claroty_vulnerabilities fetch_step body_template: sort_by present with adjusted_vulnerability_score DESC first, unique tiebreaker ASC second — DI-019 truncation retains highest-risk records

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-CLAROTY-SORTBY-DETERMINISM-001 (HS-031 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.015 §Postconditions §1 sort-by postcondition — `sort_by` array in
fetch_vulnerabilities `body_template`; EC-016-015-009 offset-pagination determinism; DI-019
truncation-relevance (DESC primary ensures highest-risk records survive the 10K cap).
**Gate:** Story-level holdout gate (HS-031) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the Claroty sensor spec (`claroty.sensor.toml`) has been updated
so that the step fetching the `claroty_vulnerabilities` table sends an explicit deterministic
`sort_by` array in every outgoing HTTP POST request body.

The evaluator reads the shipped sensor spec file `crates/prism-sensors/specs/claroty.sensor.toml`
directly (this is a data artifact, not source code internals — it defines the delivered sensor
behavior). The evaluator also issues a live MCP query to confirm the query does not error.

**Assertion set:**

1. The string `"sort_by"` appears as a JSON key in the `body_template` field of the
   `fetch_vulnerabilities` step block inside `claroty.sensor.toml`.

2. The sort_by value contains the string `"adjusted_vulnerability_score"` paired with
   `"order":"desc"` (or `"order": "desc"` with any whitespace). Case-sensitive.

3. The sort_by value contains a second element whose `"order"` is `"asc"`. This second element
   is the unique tiebreaker (provably-unique field such as CVE ID / advisory title) that makes
   the sort order total, guaranteeing no tie can exist between two rows.

4. The `"desc"` element appears BEFORE the `"asc"` tiebreaker element in the array literal.
   Reversed ordering (ASC first, DESC second) would mean DI-019 truncation keeps lowest-risk
   records — the opposite of the intended behavior.

5. Issue MCP tool call `{"sql": "SELECT name FROM claroty.claroty_vulnerabilities LIMIT 1"}`
   against the DTU. The response must be non-error (the sort_by addition must not break the
   POST body structure or cause a 400/422 from the DTU).

**DI-019 truncation-relevance implication (assertion 4 rationale):**
The Claroty xDome vulnerability dataset may exceed 10,000 records. The pipeline hard-caps
results at 10K (DI-019). With `adjusted_vulnerability_score DESC` as the primary sort, the
10K cap retains the highest adjusted_vulnerability_score records — the ones the analyst most
needs to see. With `adjusted_vulnerability_score ASC` (wrong ordering), the cap would retain
the lowest-risk records, inverting the security triage priority.

**BDD supplement:**

**Given** the story branch is built and `crates/prism-sensors/specs/claroty.sensor.toml` is
present at the story HEAD
**When** the evaluator reads the file and locates the `[[tables.steps]]` block with
`name = "fetch_vulnerabilities"` (or equivalent fetch step for the vulnerabilities table)
**Then** the `body_template` field of that step contains the substring `"sort_by"`
**And** the `body_template` contains `"adjusted_vulnerability_score"` with adjacent `"desc"`
**And** the `body_template` contains a second sort element with `"asc"` order
**And** the `"desc"` element appears before the `"asc"` element in the array literal

**And Given** prism MCP stdio is started with the claroty sensor configured against the DTU
**When** `SELECT name FROM claroty.claroty_vulnerabilities LIMIT 1` is issued via MCP `query` tool
**Then** the response is non-error (sort_by did not break the POST body JSON structure)

---

## Setup Instructions

1. Confirm the story branch is checked out and the binary is built from HEAD.

2. Locate `crates/prism-sensors/specs/claroty.sensor.toml` — this is the sensor spec file that
   was modified by this story. It is a TOML data file, not Rust source code.

3. Read the file and find the `[[tables.steps]]` block responsible for fetching vulnerability
   data from the Claroty xDome API (look for a `name` field identifying the vulnerabilities
   fetch step, or a `url_template` or `body_template` referencing the vulnerabilities endpoint).

4. Inspect the `body_template` string in that step block.

5. Start prism in MCP stdio mode with the Claroty DTU running. Capture full MCP stdio output.

6. Issue the MCP `query` tool call: `{"sql": "SELECT name FROM claroty.claroty_vulnerabilities LIMIT 1"}`.

7. Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.015 | §Postconditions §1 sort-by postcondition: sort_by array with adjusted_vulnerability_score DESC + unique tiebreaker ASC | Assertions 1–4: sort_by key present; DESC element first; ASC tiebreaker second |
| BC-2.16.015 | EC-016-015-009: offset pagination determinism — total sort order guaranteed by unique tiebreaker | Assertion 3: second element (ASC tiebreaker) makes sort total |
| BC-2.16.015 | DI-019 truncation-relevance: DESC primary ensures 10K cap retains highest-risk records | Assertion 4: DESC before ASC (not reversed) |
| BC-2.16.015 | §Postconditions §1: body_template is valid JSON that the DTU/API accepts | Assertion 5: MCP query non-error |

---

## Verification Approach

1. Use `Bash` to search for the vulnerabilities sort_by:
   ```bash
   grep -A 5 '"sort_by"' crates/prism-sensors/specs/claroty.sensor.toml | head -40
   ```
   Or use the `Read` tool to read the full file and locate the relevant `body_template` string.

2. Identify the fetch step for `claroty_vulnerabilities` (look for the `body_template` field
   containing `"fields"` with vulnerability column names, or the step whose table name matches
   the vulnerabilities table).

3. Extract the `body_template` string value for that step. It is a TOML single-quoted string
   literal (`'...'`).

4. In the extracted string, verify `"sort_by"` appears as a JSON key.

5. Verify `"adjusted_vulnerability_score"` appears in the sort_by array with `"order":"desc"`
   (allowing any whitespace around the colon).

6. Verify a second element exists with `"order":"asc"`. Extract the `"field"` value of this
   second element — record it as the tiebreaker field name (do NOT assert specific field name;
   assert only that the second element exists and has order asc).

7. Verify the DESC element's position in the JSON array string precedes the ASC element's
   position (the DESC element index is lower in the array literal).

8. Run the MCP query against the DTU. Record response as PASS (non-error) or FAIL (error
   with message).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75. Must_pass: FAIL < 0.75 blocks merge.

- **sort_by key present in body_template** (weight: 0.20): Does the fetch_vulnerabilities
  body_template contain `"sort_by"` as a JSON key?
  Full credit (1.0): `"sort_by"` found.
  Zero credit (0.0): `"sort_by"` absent — the fix was not applied.

- **adjusted_vulnerability_score DESC present** (weight: 0.35): Does the sort_by array contain
  `adjusted_vulnerability_score` with order `desc`?
  Full credit (1.0): both field name and order present.
  Partial credit (0.5): field name present but order is `asc` instead of `desc` (reversed).
  Zero credit (0.0): field name absent.

- **Unique tiebreaker ASC present** (weight: 0.25): Does the sort_by array contain a second
  element with order `asc`?
  Full credit (1.0): second element with `asc` present.
  Zero credit (0.0): only one element (no tiebreaker) — pagination is non-deterministic at ties.

- **DESC before ASC (DI-019 truncation relevance)** (weight: 0.10): Does the DESC element
  appear before the ASC element in the JSON array literal?
  Full credit (1.0): DESC at index 0, ASC at index 1 (correct for highest-risk-first truncation).
  Zero credit (0.0): ASC at index 0, DESC at index 1 (inverted — lowest-risk records survive cap).

- **MCP query non-error** (weight: 0.10): Does the MCP SELECT on claroty_vulnerabilities
  LIMIT 1 return a non-error response?
  Full credit (1.0): non-error response.
  Zero credit (0.0): error response (sort_by broke the POST body structure).

---

## Edge Conditions

- **DTU returns zero records:** SETUP-FAILURE — not a behavioral FAIL. The structural assertions
  on the TOML file are independent of DTU data availability.

- **fetch_vulnerabilities step uses a different step name:** The evaluator must locate the correct
  step by table name or URL template, not by a literal step name assumption.

- **sort_by key appears in body_template but with different whitespace:** Assertions must be
  whitespace-tolerant (e.g., `"sort_by" :` with a space is equivalent to `"sort_by":`).

- **sort_by array has MORE than 2 elements:** PASS — additional tiebreakers beyond the second
  element do not violate the contract; the evaluator asserts the minimum required structure.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-SORTBY-001-001 (satisfaction: X.XX) — claroty_vulnerabilities sort_by gap; check body_template for the vulnerabilities fetch step in claroty.sensor.toml contains explicit sort_by array with adjusted_vulnerability_score DESC first and unique ASC tiebreaker second (BC-2.16.015 §Postconditions §1 sort-by postcondition + EC-016-015-009)"`

Do NOT disclose: the specific tiebreaker field name expected, the LIMIT value used, or the
exact assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | `crates/prism-sensors/specs/claroty.sensor.toml` (shipped sensor spec) + live MCP query against Claroty DTU |
| corpus_size | Single TOML file + LIMIT 1 smoke query |
| known_edge_cases | Zero DTU records → SETUP-FAILURE (structural TOML assertions are independent); sort_by absent → FAIL; DESC/ASC reversed → PARTIAL |
| false_positive_threshold | Zero: sort_by present + DESC first + ASC second is a direct structural assertion on the delivered artifact |
| false_negative_threshold | Zero: if sort_by is absent, the pagination determinism guarantee is broken for any dataset size |

**Known-good corpus:** Story branch with sort_by correctly added — expected: sort_by present,
adjusted_vulnerability_score DESC at index 0, unique tiebreaker ASC at index 1, MCP query non-error.

**Known-problematic corpus:** Branch without the sort_by addition (pre-fix state) — expected:
`"sort_by"` absent from body_template; MCP query succeeds but is non-deterministic.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | defect-claroty-sortby-holdout-authoring | 2026-09-02 | product-owner | Initial authoring. HS-031 group for DEFECT-CLAROTY-SORTBY-DETERMINISM-001. Vulnerabilities sort_by: DESC primary for DI-019 truncation-relevance + unique ASC tiebreaker for total sort order. BC-2.16.015 §Postconditions §1 + EC-016-015-009. SINGLE-USE. |
