---
document_type: holdout-scenario
level: L3
id: "HS-VULNS-001-002"
title: "claroty_vulnerabilities: SELECT finding_info_title succeeds (Tier-1 OCSF column queryable; no E-QUERY-038)"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-A"
story_source: "S-CLAROTY-VULNS-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-24"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.015-claroty-vulnerabilities-table.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "f043488"
traces_to: "BC-2.16.015"
behavioral_contracts:
  - BC-2.16.015
  - BC-2.11.016
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-VULNS-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-VULNS-001 (HS-024 group). Tests BC-2.16.015 §Postconditions 2: Tier-1 column finding_info_title (ocsf_field_to_arrow_name of finding_info.title) is queryable by OCSF Arrow field name; SELECT finding_info_title does NOT raise E-QUERY-038. Verifies that the ocsf_field_to_arrow_name transform has been applied at the plan-gate column registry, not just at serialization. ALSO verifies the complement: SELECT name (raw TOML col.name for a Tier-1 column) DOES raise E-QUERY-038 under ocsf_column_naming=true. Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-VULNS-001-002: claroty_vulnerabilities: SELECT finding_info_title succeeds (Tier-1 OCSF column queryable; no E-QUERY-038)

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-VULNS-001 (HS-024 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.015 §Postconditions 2 Tier-1 column plan-gate acceptance; BC-2.11.016 E-QUERY-038 column-not-found plan gate
**Gate:** Story-level holdout gate (HS-024) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

Under `ocsf_column_naming = true`, a Tier-1 column is registered in the plan-gate column registry under its Arrow field name (result of `ocsf_field_to_arrow_name(ocsf_field)`), NOT its raw TOML `name`. For the `claroty_vulnerabilities` table, the column `name` has `ocsf_field = "finding_info.title"`, which transforms to the Arrow field name `finding_info_title`.

This scenario validates two complementary assertions:

1. **Positive:** `SELECT finding_info_title FROM claroty.claroty_vulnerabilities LIMIT 5` succeeds at the plan gate (no E-QUERY-038). Rows are returned from the live sensor with `finding_info_title` values. This confirms `ocsf_field_to_arrow_name` was applied at plan-gate column registration time, not only at row serialization.

2. **Negative complement (partial):** `SELECT name FROM claroty.claroty_vulnerabilities LIMIT 1` raises E-QUERY-038 with `available_columns` containing `finding_info_title` but NOT `name`. This confirms the raw TOML column name is NOT exposed under `ocsf_column_naming = true` (the raw name is invisible at the plan gate — it is the ocsf-renamed name that is visible).

If assertion 1 fails (E-QUERY-038 on `finding_info_title`), the Tier-1 column mapping was not propagated to the plan-gate column registry. If assertion 2 fails (no E-QUERY-038 on `name`), the raw TOML name was incorrectly exposed alongside the OCSF-renamed name.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT finding_info_title FROM claroty.claroty_vulnerabilities LIMIT 5` is issued via the MCP `query` tool
**Then** the response is NOT an E-QUERY-038 error
**And** the response rows contain a `finding_info_title` column with string values
**When** `SELECT name FROM claroty.claroty_vulnerabilities LIMIT 1` is issued via the MCP `query` tool
**Then** the response IS an E-QUERY-038 error
**And** the `available_columns` field contains `finding_info_title` but NOT `name`

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 — do not log the credential value).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Wait for prism to be ready (startup log or first JSON-RPC prompt).

5. Issue first MCP `query` tool call: `{"sql": "SELECT finding_info_title FROM claroty.claroty_vulnerabilities LIMIT 5"}`. Capture the full response.

6. Issue second MCP `query` tool call: `{"sql": "SELECT name FROM claroty.claroty_vulnerabilities LIMIT 1"}`. Capture the full response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.015 | §Postconditions 2 Tier-1: name col → ocsf_field = "finding_info.title" → Arrow field finding_info_title; plan gate must accept finding_info_title | Assertion 1: SELECT finding_info_title does NOT raise E-QUERY-038 |
| BC-2.11.016 | E-QUERY-038: column not in registered set → error with available_columns | Assertion 2: SELECT name (raw TOML col.name) raises E-QUERY-038; available_columns contains finding_info_title, not name |
| ADR-058 | ocsf_field_to_arrow_name applied at plan-gate registration, not only serialization | Both assertions together verify the transform happens at column registry time |

---

## Verification Approach

1. Parse response from first query (`SELECT finding_info_title`).
   - If the response contains an `error_code` matching E-QUERY-038 or equivalent column-not-found: record FAIL on "finding_info_title queryable" dimension.
   - If the response is any other error (sensor unreachable, auth failure): record as SETUP-FAILURE.
   - If the response is success: inspect rows. Confirm `finding_info_title` is a column. Record PASS.

2. Parse response from second query (`SELECT name`).
   - If the response does NOT contain an E-QUERY-038 error: record FAIL on "name col.name rejected" dimension.
   - If E-QUERY-038 is present: inspect `available_columns`. Assert `finding_info_title` is in available_columns. Assert `name` is NOT in available_columns. Record PASS.

3. If live sensor returns zero rows on first query (empty vulnerability list): record as SETUP-FAILURE for the content assertion only; plan-gate acceptance still scoreable (success response with 0 rows = PASS on plan-gate dimension).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **finding_info_title queryable at plan gate (no E-QUERY-038)** (weight: 0.45): Does `SELECT finding_info_title` return a non-E-QUERY-038 response?
  Full credit (1.0): response is success or any non-E-QUERY-038 error (e.g., sensor unreachable is fine — plan gate accepted it).
  Zero credit (0.0): response is E-QUERY-038 — ocsf_field_to_arrow_name was not applied at column registry.

- **name (raw col.name) rejected with E-QUERY-038** (weight: 0.30): Does `SELECT name` raise E-QUERY-038?
  Full credit (1.0): E-QUERY-038 fires.
  Zero credit (0.0): no E-QUERY-038 — raw TOML column name incorrectly exposed.

- **available_columns for name error contains finding_info_title, not name** (weight: 0.25): When E-QUERY-038 fires on `SELECT name`, does `available_columns` contain `finding_info_title` but NOT `name`?
  Full credit (1.0): finding_info_title present, name absent.
  Partial credit (0.5): E-QUERY-038 fires but available_columns is incomplete (missing finding_info_title, or incorrectly includes name).
  Zero credit (0.0): E-QUERY-038 did not fire — this dimension cannot be scored.

---

## Edge Conditions

- **Both queries return E-QUERY-038:** Indicates `claroty_vulnerabilities` table has no columns registered in the plan gate at all (likely a TOML parse failure or ocsf_field_to_arrow_name not applied). Score zero on all dimensions.

- **finding_info_title query returns sensor-unreachable error (not E-QUERY-038):** PASS on plan-gate dimension (the plan gate accepted the column; the fetch failed for infrastructure reasons). This is acceptable behavior per BC-2.16.015 §Error Cases.

- **E-QUERY-038 on `name` with correct available_columns but `name` also present in available_columns:** Partial credit (0.5) — the rejection fires correctly but the available set exposure is wrong (raw name should not be visible).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-VULNS-001-002 (satisfaction: X.XX) — claroty_vulnerabilities Tier-1 column plan-gate gap; check ocsf_field_to_arrow_name transform applied at column registry (not only at serialization); BC-2.16.015 §Postconditions 2 Tier-1 / BC-2.11.016 E-QUERY-038 available_columns / ADR-058 plan-gate timing"`

Do NOT disclose: the specific column names queried, the LIMIT values, or the exact assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/vulnerabilities/ |
| corpus_size | LIMIT 5 for positive assertion (structural); LIMIT 1 for negative |
| known_edge_cases | Zero rows from live sensor (empty vuln list) — SETUP-FAILURE on content assertion, plan-gate dimension still scoreable |
| false_positive_threshold | Zero: plan-gate acceptance vs rejection is a binary structural test |
| false_negative_threshold | Zero: absence of E-QUERY-038 on raw col.name means the plan gate is exposing names it should hide |

**Known-good corpus:** Correctly implemented `claroty_vulnerabilities` table with `ocsf_field_to_arrow_name` applied — expected: `SELECT finding_info_title` succeeds; `SELECT name` raises E-QUERY-038 with correct available_columns.

**Known-problematic corpus:** A hypothetical implementation that registers both the raw TOML column name AND the OCSF field name in the plan gate — expected: `SELECT name` would NOT raise E-QUERY-038. This is the ADR-058 violation this scenario catches.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-a-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-024 group for S-CLAROTY-VULNS-001. Tier-1 column plan-gate query acceptance (finding_info_title) and raw col.name rejection (name → E-QUERY-038 with correct available_columns). BC-2.16.015 §Postconditions 2 + BC-2.11.016 + ADR-058. SINGLE-USE. |
