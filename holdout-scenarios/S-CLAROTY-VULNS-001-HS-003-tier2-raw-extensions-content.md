---
document_type: holdout-scenario
level: L3
id: "HS-VULNS-001-003"
title: "claroty_vulnerabilities: raw_extensions JSON contains Tier-2 fields (cve_ids, severity_score present as keys)"
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
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "90ea8c4"
traces_to: "BC-2.16.015"
behavioral_contracts:
  - BC-2.16.015
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-VULNS-001 (HS-024 group). Tests BC-2.16.015 §Postconditions 2 Tier-2 columns: raw_extensions JSON column is present in wire output AND contains at least two Tier-2 column keys. Specific keys verified: 'cve_ids' and 'severity_score'. Tests that Tier-2 aggregation into raw_extensions is functioning, not just that raw_extensions exists as a column. Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-VULNS-001-003: claroty_vulnerabilities: raw_extensions JSON contains Tier-2 fields (cve_ids, severity_score present as keys)

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-VULNS-001 (HS-024 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.015 §Postconditions 2 Tier-2 columns — 17 columns aggregate into raw_extensions JSON object under ocsf_column_naming = true
**Gate:** Story-level holdout gate (HS-024) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

Under `ocsf_column_naming = true`, all Tier-2 columns (no `ocsf_field` declared) aggregate into the `raw_extensions` JSON column. For `claroty_vulnerabilities`, 17 columns are Tier-2 — including `cve_ids` (a JSON array of CVE identifiers) and `severity_score` (a Float risk score from 0–10). These are highly informative fields for a vulnerability analyst.

This scenario validates that the Tier-2 aggregation is correctly implemented by asserting the wire-level `raw_extensions` JSON blob for a live vulnerability row from the monroe sensor:

1. `SELECT raw_extensions FROM claroty.claroty_vulnerabilities LIMIT 1` returns a non-error response where `raw_extensions` is present as a column.

2. The `raw_extensions` value for at least one row is a JSON object (not null, not a string representation of null, not an empty object `{}`).

3. The JSON object contains at least the keys `cve_ids` and `severity_score` — two Tier-2 columns that carry high-value vulnerability data. Their presence confirms Tier-2 aggregation is functioning, not just that the `raw_extensions` column was added as a stub.

If `raw_extensions` is present but empty `{}`, it indicates the Tier-2 fields were not aggregated (they were silently dropped). If `raw_extensions` is absent entirely, the Tier-2 aggregation machinery was not invoked at all.

**Note:** The actual values of `cve_ids` and `severity_score` are NOT asserted — they depend on the live sensor's vulnerability data. Only their presence as keys in `raw_extensions` is asserted.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT raw_extensions FROM claroty.claroty_vulnerabilities LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response rows contain a `raw_extensions` column
**And** the `raw_extensions` value in at least one row is a JSON object
**And** the JSON object contains the key `cve_ids`
**And** the JSON object contains the key `severity_score`

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 — do not log the credential value).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Wait for prism to be ready.

5. Issue the MCP `query` tool call: `{"sql": "SELECT raw_extensions FROM claroty.claroty_vulnerabilities LIMIT 1"}`.

6. Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.015 | §Postconditions 2 Tier-2: 17 columns aggregate into raw_extensions JSON object | Assertion 2: raw_extensions is a JSON object (not null, not empty) |
| BC-2.16.015 | §Postconditions 2 Tier-2 column list: cve_ids (Json) and severity_score (Float) are Tier-2 | Assertion 3: cve_ids and severity_score present as keys in raw_extensions |
| ADR-058 | Tier-2 aggregation: columns without ocsf_field → aggregate into raw_extensions JSON blob | End-to-end: Tier-2 data is present in wire output, not silently dropped |

---

## Verification Approach

1. Parse the wire-level JSON response from the MCP `query` tool call.

2. If the response is an error (any error code): record FAIL if E-QUERY-038 (raw_extensions should be in available set); record SETUP-FAILURE for other errors (sensor unreachable, auth failure).

3. Locate the `raw_extensions` column in the first returned row. If absent: record FAIL on "raw_extensions column present" dimension.

4. Inspect the value of `raw_extensions`. Parse it as JSON if it arrives as a string blob.
   - If null or `{}` (empty object): record FAIL on "raw_extensions non-empty" dimension (Tier-2 fields not aggregated).
   - If a non-empty JSON object: record PASS on this dimension.

5. Check for the key `cve_ids` in the parsed JSON object.
   - Present (any value including null array): PASS on "cve_ids key present" dimension.
   - Absent: PARTIAL on this dimension (field may be null-suppressed or Tier-2 aggregation partial).

6. Check for the key `severity_score` in the parsed JSON object.
   - Present (any value including null): PASS on "severity_score key present" dimension.
   - Absent: PARTIAL on this dimension.

7. Note: if the live sensor returns a row where `cve_ids` is genuinely null AND `severity_score` is genuinely null, they may be null-suppressed in the JSON blob (implementation detail). If BOTH are absent, attempt a second row (if multiple rows were retrieved). If still absent after all returned rows, record as PARTIAL — may be a null-suppression edge case rather than a Tier-2 aggregation failure.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query succeeds (no error)** (weight: 0.25): Does the MCP query return a non-error response?
  Full credit (1.0): non-error response (including empty result set).
  Zero credit (0.0): error response (any error code).

- **raw_extensions column present and non-null/non-empty** (weight: 0.35): Is raw_extensions a non-null, non-empty JSON object in at least one row?
  Full credit (1.0): raw_extensions present, non-null, non-empty JSON object.
  Partial credit (0.4): raw_extensions present but empty `{}` — column registered but aggregation not functioning.
  Zero credit (0.0): raw_extensions column absent entirely.

- **cve_ids key present in raw_extensions** (weight: 0.20): Does raw_extensions contain `cve_ids` as a key?
  Full credit (1.0): key present in at least one row.
  Partial credit (0.5): key absent in returned rows but raw_extensions is non-empty (possible null-suppression — evaluator must note).
  Zero credit (0.0): raw_extensions absent or empty.

- **severity_score key present in raw_extensions** (weight: 0.20): Does raw_extensions contain `severity_score` as a key?
  Full credit (1.0): key present in at least one row.
  Partial credit (0.5): key absent but raw_extensions non-empty (possible null for this specific row).
  Zero credit (0.0): raw_extensions absent or empty.

---

## Edge Conditions

- **Live sensor returns null for both cve_ids and severity_score for all returned rows:** If the implementation null-suppresses keys with null values in the raw_extensions JSON blob, both keys may be absent in the output even though they are Tier-2 columns. Record as PARTIAL (0.5) on key-presence dimensions with observation "both keys null-suppressed; possible null-suppression implementation — check with additional rows or verify Tier-2 aggregation at spec-engine level."

- **raw_extensions is present but contains only UNKNOWN Tier-2 keys (keys not from the BC-2.16.015 Tier-2 column list):** Record as PARTIAL on key-presence dimensions. The Tier-2 aggregation is functioning but the column names don't match the spec.

- **SELECT raw_extensions raises E-QUERY-038:** FAIL — `raw_extensions` must be in the available set for a table with Tier-2 columns.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-VULNS-001-003 (satisfaction: X.XX) — claroty_vulnerabilities Tier-2 raw_extensions aggregation gap; check that Tier-2 columns are aggregated into raw_extensions JSON blob at pipeline_result_to_record_batch (BC-2.16.015 §Postconditions 2 Tier-2; ADR-058 Tier-2 aggregation path)"`

Do NOT disclose: the specific key names checked, the LIMIT value, or the exact assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/vulnerabilities/ — vulnerability rows with CVE and risk score data |
| corpus_size | LIMIT 1 (one row sufficient; evaluator may inspect more if key-presence is ambiguous) |
| known_edge_cases | Null-suppressed Tier-2 keys for rows where those fields are null; empty result set (no vulnerabilities) |
| false_positive_threshold | Low: presence of cve_ids and severity_score as keys is a structural assertion on the Tier-2 aggregation mechanism |
| false_negative_threshold | Zero for raw_extensions column; low for specific key presence (null-suppression caveat) |

**Known-good corpus:** Monroe with a vulnerability that has CVE IDs and a risk score — expected: raw_extensions non-empty JSON with cve_ids and severity_score keys.

**Known-problematic corpus:** An implementation that drops Tier-2 fields (no aggregation, empty raw_extensions) — expected: raw_extensions = `{}` or absent. This is the failure mode this scenario guards against.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-a-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-024 group for S-CLAROTY-VULNS-001. Tier-2 raw_extensions aggregation content: cve_ids and severity_score keys present in raw_extensions JSON wire output. BC-2.16.015 §Postconditions 2 Tier-2 + ADR-058. SINGLE-USE. |
