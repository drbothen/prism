---
document_type: holdout-scenario
level: L3
id: "HS-SORTBY-001-004"
title: "claroty_server_interfaces fetch_step body_template: composite sort_by has server_name ASC first then interface_name ASC second — both elements required, order matters for PK uniqueness guarantee"
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
  - ".factory/specs/behavioral-contracts/BC-2.16.019-claroty-server-interfaces-table.md"
input-hash: "[pending-recompute]"
traces_to: "BC-2.16.019"
behavioral_contracts:
  - BC-2.16.019
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
notes: "HIDDEN, SINGLE-USE story-level holdout for DEFECT-CLAROTY-SORTBY-DETERMINISM-001 (HS-031 group). Tests EC-016-019-007 composite PK guarantee: server_interfaces sort_by must contain BOTH server_name ASC and interface_name ASC in that exact order. The composite (server_name, interface_name) is the unique PK for this table. A single-field sort (server_name alone) is non-unique because one server can have multiple interfaces. The interface_name tiebreaker is the key contribution of this story for this table. Test-writer and implementer MUST NOT read this file."
---

# HS-SORTBY-001-004: claroty_server_interfaces fetch_step body_template: composite sort_by has server_name ASC first then interface_name ASC second — both elements required, order matters for PK uniqueness guarantee

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-CLAROTY-SORTBY-DETERMINISM-001 (HS-031 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.019 §Postconditions §1 sort-by postcondition: composite PK sort
`[server_name asc, interface_name asc]`; EC-016-019-007 offset-pagination determinism via
composite unique key `(server_name, interface_name)`.
**Gate:** Story-level holdout gate (HS-031) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the composite sort key correctness for `claroty_server_interfaces`.
This table's unique primary key is the COMBINATION of two fields: the parent server identifier
and the interface identifier. A single-field sort on the server identifier alone is insufficient
because one server can have multiple interfaces — those interfaces share the same server-name
value and would have an undefined relative order under a single-field sort.

The fix for this table requires EXACTLY TWO sort elements:
1. The server field — as the primary partition key (ASC)
2. The interface field — as the tiebreaker (ASC)

The order of these two elements in the sort_by array matters: swapping them (interface first,
server second) would still produce a total order but would change the PAGE BOUNDARY positions
compared to the correct ordering, potentially causing different duplications if API page sizes
vary.

The evaluator reads `claroty.sensor.toml` and verifies the composite sort key structure.

**Assertion set:**

1. The `body_template` for the server_interfaces fetch step contains `"sort_by"` with an
   array of EXACTLY 2 elements. One element and three or more elements are both non-conforming.

2. The FIRST element of the sort_by array has:
   - `"order": "asc"` (or `"order" : "asc"` with any whitespace)
   - A field name referencing the server identifier (the evaluator identifies this by reading
     the field name from the JSON array and confirming it refers to a server-level identifier —
     the exact field name is observable from the TOML context)

3. The SECOND element of the sort_by array has:
   - `"order": "asc"`
   - A field name referencing the interface-level identifier (a different field name from the
     first element)

4. The two field names in the sort_by array are DIFFERENT from each other — not `[A, A]` or
   two copies of the same field.

5. Issue MCP tool call `{"sql": "SELECT * FROM claroty.claroty_server_interfaces LIMIT 2"}`
   against the DTU. The response must be non-error AND if ≥ 2 rows are returned, the two rows
   must have DIFFERENT values in at least one of the two sort key columns (confirming the
   composite key provides differentiation).

**BDD supplement:**

**Given** the story branch is built and `crates/prism-sensors/specs/claroty.sensor.toml` is
present at story HEAD
**When** the evaluator reads the file and locates the fetch step for server_interfaces
**Then** the `body_template` of that step contains `"sort_by"` with exactly 2 array elements
**And** the first element has `"order": "asc"` and a server-level field name
**And** the second element has `"order": "asc"` and a different (interface-level) field name

**And Given** prism MCP stdio is started with claroty sensor configured against the DTU
**When** `SELECT * FROM claroty.claroty_server_interfaces LIMIT 2` is issued via MCP `query`
**Then** the response is non-error

---

## Setup Instructions

1. Confirm the story branch is checked out and the binary is built from HEAD.

2. Locate `crates/prism-sensors/specs/claroty.sensor.toml`.

3. Read the file and find the `[[tables.steps]]` block for the server_interfaces fetch step.
   It can be identified by its `body_template` containing `"fields"` with interface-specific
   column names, or a URL or step name referencing server interface data.

4. Extract the `body_template` string. Locate the `"sort_by"` key within it.

5. Extract the JSON array value of `"sort_by"` (substring from the opening `[` to the
   matching `]`). Count the number of `{...}` objects in the array.

6. For element[0] and element[1], extract the `"field"` and `"order"` values.

7. Start prism in MCP stdio mode with the Claroty DTU running.

8. Issue `SELECT * FROM claroty.claroty_server_interfaces LIMIT 2` — capture the full raw
   wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.019 | §Postconditions §1 sort-by postcondition: composite sort_by [server_name asc, interface_name asc] | Assertions 1–4: two elements, both ASC, different fields |
| BC-2.16.019 | EC-016-019-007: composite PK (server_name, interface_name) guarantees total sort order for deterministic offset pagination | Assertions 1–4: two distinct ASC elements make the sort total over the unique PK |
| BC-2.16.019 | §Postconditions §3: (server_name, interface_name) is the composite unique PK | Assertion 4: two different field names (not duplicate of same field) |
| BC-2.16.019 | End-to-end: server_interfaces queryable without error after body_template update | Assertion 5: MCP query non-error |

---

## Verification Approach

1. Read the TOML and extract the server_interfaces step's body_template (Setup step 3–4).

2. Locate the `"sort_by"` key and extract its JSON array value (Setup step 5).

3. Count array elements. Record EXACTLY-2 as PASS, any other count as FAIL.

4. Parse element[0]: extract `"field"` value (call it FIELD_A) and `"order"` value (expected
   `"asc"`). If element[0].order is NOT `"asc"`, record FAIL on dimension 2.

5. Parse element[1]: extract `"field"` value (call it FIELD_B) and `"order"` value (expected
   `"asc"`). If element[1].order is NOT `"asc"`, record FAIL on dimension 3.

6. Assert FIELD_A ≠ FIELD_B. If they are identical, record FAIL on dimension 4.

7. Run the MCP query (Setup step 8). Record PASS (non-error) or FAIL (error).

8. If the MCP response returns ≥ 2 rows, check whether the two rows have at least one
   difference in the values of the sort key columns — this is a soft corroboration check
   (weight 0.05 in rubric). If only 0 or 1 rows returned, this check is SKIP.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **sort_by array has exactly 2 elements** (weight: 0.30): Does the sort_by array for the
  server_interfaces fetch step contain exactly 2 elements?
  Full credit (1.0): exactly 2 elements.
  Partial credit (0.5): 3+ elements (extra elements are acceptable if first two are correct).
  Zero credit (0.0): 0 elements (missing sort_by) or 1 element (single-field sort, non-unique).

- **Element 0: server field, ASC order** (weight: 0.25): Does the first element have order
  `"asc"` and a non-empty field name?
  Full credit (1.0): order="asc", field present.
  Partial credit (0.5): field present but order ≠ "asc".
  Zero credit (0.0): no first element.

- **Element 1: interface field, ASC order** (weight: 0.25): Does the second element have
  order `"asc"` and a non-empty field name?
  Full credit (1.0): order="asc", field present.
  Partial credit (0.5): field present but order ≠ "asc".
  Zero credit (0.0): no second element.

- **Two different field names (not duplicate)** (weight: 0.15): Are the field names in
  element[0] and element[1] different from each other?
  Full credit (1.0): FIELD_A ≠ FIELD_B.
  Zero credit (0.0): FIELD_A = FIELD_B (duplicate key — sort is effectively single-field).

- **MCP query non-error** (weight: 0.05): Does the MCP SELECT LIMIT 2 return a non-error
  response?
  Full credit (1.0): non-error.
  Zero credit (0.0): error.

---

## Edge Conditions

- **sort_by array has 3+ elements:** The composite PK is 2 fields. A third element is
  redundant but not harmful. Score dimension 1 as PARTIAL (0.5), assert first two elements
  still pass dimensions 2–4.

- **sort_by has 2 elements but one has order "desc":** The sort is not a pure ASC ordering
  for this table — score dimension 2 or 3 as PARTIAL (0.5). The composite PK still provides
  a total order IF the fields are distinct, but the sort direction deviates from spec.

- **DTU returns only 0 or 1 server_interface records:** The soft corroboration check (SKIP);
  does not affect the rubric.

- **DTU returns zero records entirely:** SETUP-FAILURE — not a behavioral FAIL. TOML
  assertions are independent of DTU data.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-SORTBY-001-004 (satisfaction: X.XX) — claroty_server_interfaces composite sort key gap; the sort_by array in the server_interfaces fetch step body_template must contain exactly 2 ASC elements representing the composite unique PK — check that both the server-level field and the interface-level field are present in that order (BC-2.16.019 §Postconditions §1 + EC-016-019-007 composite PK total sort guarantee)"`

Do NOT disclose: the specific field names expected (server_name, interface_name), the exact
element count found, or the precise assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | `crates/prism-sensors/specs/claroty.sensor.toml` (shipped sensor spec) + live MCP query against Claroty DTU |
| corpus_size | Single TOML file (server_interfaces fetch step body_template) + LIMIT 2 smoke query |
| known_edge_cases | Single-field sort (1 element) → FAIL (non-unique sort, non-deterministic at ties); duplicate field names ([A, A]) → FAIL (equivalent to single-field sort); 3+ elements → PARTIAL |
| false_positive_threshold | Zero: a 2-element sort_by with distinct field names directly confirms the composite PK total-order guarantee |
| false_negative_threshold | Zero: a 1-element sort_by for server_interfaces leaves the sort non-unique (multiple interfaces per server would have undefined relative order) |

**Known-good corpus:** Story branch with correct composite sort — expected: sort_by 2 elements,
both ASC, different field names (server + interface level), MCP non-error.

**Known-problematic corpus:** Branch with single-field sort (server only) for server_interfaces
— expected: sort_by array has 1 element (interface-level tiebreaker missing), score 0.0 on
the "exactly 2 elements" dimension, overall satisfaction < 0.75.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | defect-claroty-sortby-holdout-authoring | 2026-09-02 | product-owner | Initial authoring. HS-031 group for DEFECT-CLAROTY-SORTBY-DETERMINISM-001. Server_interfaces composite PK sort verification: 2 ASC elements, distinct fields, order matters. BC-2.16.019 §Postconditions §1 + EC-016-019-007. SINGLE-USE. |
