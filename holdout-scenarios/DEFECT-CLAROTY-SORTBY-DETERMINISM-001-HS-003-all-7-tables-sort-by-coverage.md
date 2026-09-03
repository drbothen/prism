---
document_type: holdout-scenario
level: L3
id: "HS-SORTBY-001-003"
title: "All 7 Claroty tables have sort_by in body_template — no table accidentally omitted; count of sort_by keys in claroty.sensor.toml equals exactly 7 for the affected fetch steps"
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
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.019-claroty-server-interfaces-table.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.020-claroty-org-zone-domain.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.021-claroty-org-firewall-domain.md"
input-hash: "[pending-recompute]"
traces_to: "BC-2.16.015"
behavioral_contracts:
  - BC-2.16.015
  - BC-2.16.013
  - BC-2.16.019
  - BC-2.16.020
  - BC-2.16.021
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
notes: "HIDDEN, SINGLE-USE story-level holdout for DEFECT-CLAROTY-SORTBY-DETERMINISM-001 (HS-031 group). Broad coverage gate: counts the number of fetch step body_templates in claroty.sensor.toml that contain the sort_by key. The story fixes 7 tables — vulnerabilities, audit_logs, server_interfaces, organization_zones, zone_policies, firewall_groups, firewall_policies. A count below 7 means at least one table was accidentally missed. Test-writer and implementer MUST NOT read this file."
---

# HS-SORTBY-001-003: All 7 Claroty tables have sort_by in body_template — no table accidentally omitted; count of sort_by keys in claroty.sensor.toml equals at least 7 for the affected fetch steps

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-CLAROTY-SORTBY-DETERMINISM-001 (HS-031 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.015 (vulnerabilities), BC-2.16.013 (audit_logs), BC-2.16.019
(server_interfaces), BC-2.16.020 (organization_zones + zone_policies), BC-2.16.021
(firewall_groups + firewall_policies) — all 5 BCs contain sort-by postconditions that
this story is supposed to satisfy.
**Gate:** Story-level holdout gate (HS-031) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates BREADTH of coverage: that ALL 7 tables received the sort_by fix, not
just a subset. The story scope is 7 tables across 5 BC domains. A partial implementation
(e.g., 5 of 7 tables fixed) would pass all 10 targeted RG tests for the fixed tables while
leaving 2 tables non-deterministic.

The evaluator counts occurrences of `"sort_by"` as a JSON key in `body_template` strings
within `claroty.sensor.toml`. The expected count is ≥ 7 (one per affected fetch step). A
count below 7 means at least one table's sort_by was not added.

**Assertion set:**

1. The total number of `"sort_by"` occurrences in `claroty.sensor.toml` that appear within
   `body_template` string values is AT LEAST 7. The evaluator counts by grepping for the
   `"sort_by"` string within the file.

2. Each of the 7 named table categories below is represented by at least one `"sort_by"`
   key in a body_template. The evaluator identifies the steps by the table data they fetch
   (using the surrounding context: fields array, URL template, or step name). The 7 categories:
   - Vulnerability findings table
   - Audit log events table
   - Server interfaces table
   - Organization zones table
   - Organization zone policies table
   - Organization firewall groups table
   - Organization firewall policies table

3. For each identified sort_by, the value contains at least one `"field"` key and one
   `"order"` key — confirming it is a well-formed sort_by array (not a placeholder or empty
   array that would silently disable sorting).

4. Issue one MCP smoke query per domain pair to confirm no table errors:
   a. `SELECT * FROM claroty.claroty_server_interfaces LIMIT 1`
   b. `SELECT * FROM claroty.claroty_organization_zones LIMIT 1`
   The responses must be non-error. (This samples 2 of the 7 tables beyond the ones covered
   by HS-001 and HS-002 smoke tests.)

**BDD supplement:**

**Given** the story branch is built and `crates/prism-sensors/specs/claroty.sensor.toml` is
present at story HEAD
**When** the evaluator greps the file for the `"sort_by"` key within body_template strings
**Then** the count of `"sort_by"` occurrences is ≥ 7

**And Given** the evaluator identifies each body_template step by the table it fetches
**Then** each of the 7 expected table categories has at least one step with `"sort_by"` present

**And Given** prism MCP stdio is started with claroty sensor configured against the DTU
**When** `SELECT * FROM claroty.claroty_server_interfaces LIMIT 1` is issued
**Then** the response is non-error
**When** `SELECT * FROM claroty.claroty_organization_zones LIMIT 1` is issued
**Then** the response is non-error

---

## Setup Instructions

1. Confirm the story branch is checked out and the binary is built from HEAD.

2. Locate `crates/prism-sensors/specs/claroty.sensor.toml`.

3. Count `"sort_by"` occurrences:
   ```bash
   grep -c '"sort_by"' crates/prism-sensors/specs/claroty.sensor.toml
   ```
   Record the count. Expected: ≥ 7. If count = 0, no table was fixed. If count = 1–6,
   some tables were fixed but not all.

4. Read the file and locate each `[[tables.steps]]` block that contains a `body_template`
   field. For each such block, note (a) whether `"sort_by"` is present, and (b) which
   table the step fetches (inferred from the fields array, step name, or URL template).
   Build a checklist against the 7 expected table categories.

5. For each found sort_by value, verify the JSON array has at least one element with
   `"field"` and `"order"` keys (not an empty `[]` or malformed placeholder).

6. Start prism in MCP stdio mode with the Claroty DTU running.

7. Issue `SELECT * FROM claroty.claroty_server_interfaces LIMIT 1` — capture response.

8. Issue `SELECT * FROM claroty.claroty_organization_zones LIMIT 1` — capture response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.015 | §Postconditions §1 sort-by postcondition: vulnerabilities fetch step has sort_by | Assertion 2: vulnerabilities category covered |
| BC-2.16.013 | §Postconditions §1 audit_logs sort-by postcondition: audit_logs fetch step has sort_by | Assertion 2: audit_logs category covered |
| BC-2.16.019 | §Postconditions §1 sort-by postcondition: server_interfaces fetch step has sort_by | Assertion 2: server_interfaces category covered; assertion 4a: MCP smoke |
| BC-2.16.020 | §Postconditions §1 zones sort-by: organization_zones fetch step has sort_by | Assertion 2: org_zones category covered; assertion 4b: MCP smoke |
| BC-2.16.020 | §Postconditions §2 zone_policies sort-by: zone_policies fetch step has sort_by | Assertion 2: zone_policies category covered |
| BC-2.16.021 | §Postconditions §1 firewall_groups sort-by: firewall_groups fetch step has sort_by | Assertion 2: firewall_groups category covered |
| BC-2.16.021 | §Postconditions §2 firewall_policies sort-by: firewall_policies fetch step has sort_by | Assertion 2: firewall_policies category covered |

---

## Verification Approach

1. Run the grep count (Setup step 3). If count < 7, record FAIL for assertion 1 immediately;
   the partial count tells you how many tables were fixed.

2. Read the file and build the 7-category checklist (Setup step 4). For each category, record
   PRESENT or ABSENT.

3. For each present sort_by, extract the array value and check it contains at least one
   `{"field": "...", "order": "..."}` object (not empty or malformed).

4. Run the two MCP smoke queries (Setup steps 7–8). Record each as PASS (non-error) or FAIL.

5. Tabulate: count of present categories (out of 7), count of well-formed sort_by arrays
   (out of present), count of MCP smoke passes (out of 2).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **sort_by count ≥ 7** (weight: 0.40): Does the grep count of `"sort_by"` in the file
  equal or exceed 7?
  Full credit (1.0): count ≥ 7.
  Partial credit (count/7): count between 1 and 6 — partial implementation.
  Zero credit (0.0): count = 0 — no table was fixed.

- **All 7 categories present** (weight: 0.40): Are all 7 table categories represented by
  a sort_by-bearing body_template?
  Full credit (1.0): all 7 categories confirmed.
  Partial credit (present/7): use the fraction of categories confirmed.
  Zero credit (0.0): no category confirmed.

- **sort_by values well-formed (not empty arrays)** (weight: 0.10): Do all found sort_by
  arrays contain at least one element with `"field"` and `"order"`?
  Full credit (1.0): all non-empty and well-formed.
  Partial credit (0.5): some empty or missing field/order keys.
  Zero credit (0.0): all empty or malformed.

- **MCP smoke non-error (2 tables)** (weight: 0.10): Do both server_interfaces and
  organization_zones MCP queries return non-error responses?
  Full credit (1.0): both pass.
  Partial credit (0.5): one passes.
  Zero credit (0.0): both fail.

---

## Edge Conditions

- **`"sort_by"` appears in a TOML comment line:** Comments (`#`) in TOML are not part of
  string values. The evaluator must confirm `"sort_by"` appears inside a `body_template =
  '...'` string value, not in a comment. A count from `grep '"sort_by"'` may include
  comments — the evaluator should subtract any comment-line hits.

- **The same fetch step appears multiple times (pagination continuation steps):** Some sensors
  use a separate continuation step for subsequent pages. Count each unique step that has its
  OWN sort_by, not the total lines matching the pattern.

- **sort_by present but empty array (`"sort_by": []`):** An empty array disables sorting.
  This PASSES assertion 1 (key present) but FAILS assertion 3 (well-formed). Score dimension 3
  as 0.0 for any empty array.

- **DTU returns zero records for smoke queries:** SETUP-FAILURE — not a behavioral FAIL.
  Structural TOML assertions are independent of DTU data.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-SORTBY-001-003 (satisfaction: X.XX) — sort_by coverage gap; claroty.sensor.toml has fewer than 7 body_templates with explicit sort_by arrays; at least one of the 7 targeted fetch steps is missing sort_by (breadth coverage across all 5 BC domains: BC-2.16.015, BC-2.16.013, BC-2.16.019, BC-2.16.020, BC-2.16.021)"`

Do NOT disclose: which specific tables are missing, the exact grep count found, or the
exact assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | `crates/prism-sensors/specs/claroty.sensor.toml` (shipped sensor spec) + 2 live MCP smoke queries against Claroty DTU |
| corpus_size | Single TOML file (all 7 affected fetch steps) + LIMIT 1 smoke queries for 2 tables |
| known_edge_cases | Partial implementation (1–6 tables fixed) → PARTIAL; all 7 missing → FAIL; sort_by empty array → PARTIAL on well-formed dimension |
| false_positive_threshold | Zero: grep count of "sort_by" ≤ 6 is an unambiguous coverage gap |
| false_negative_threshold | Zero: if any of 7 tables is missing sort_by, that table's pagination is still non-deterministic |

**Known-good corpus:** Story branch with all 7 fetch steps updated — expected: grep count ≥ 7,
all 7 categories present, well-formed arrays, MCP smoke queries non-error.

**Known-problematic corpus:** Branch with partial implementation (e.g., only vulnerabilities
and audit_logs fixed) — expected: grep count = 2, 5 categories absent, partial satisfaction.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | defect-claroty-sortby-holdout-authoring | 2026-09-02 | product-owner | Initial authoring. HS-031 group for DEFECT-CLAROTY-SORTBY-DETERMINISM-001. Breadth coverage gate: grep count of "sort_by" in claroty.sensor.toml ≥ 7 across all 7 affected tables (vulnerabilities, audit_logs, server_interfaces, organization_zones, zone_policies, firewall_groups, firewall_policies). SINGLE-USE. |
