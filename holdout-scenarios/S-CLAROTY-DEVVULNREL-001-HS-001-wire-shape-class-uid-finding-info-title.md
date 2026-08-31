---
document_type: holdout-scenario
level: L3
id: "HS-DEVVULNREL-001-001"
title: "claroty_device_vulnerability_relations SELECT * wire shape: class_uid=2002 and finding_info_title column present"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-B"
story_source: "S-CLAROTY-DEVVULNREL-001"
version: "1.0"
status: active
used: true
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-31"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.017-claroty-device-vulnerability-relations-table.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "3ad5d86"
traces_to: "BC-2.16.017"
behavioral_contracts:
  - BC-2.16.017
  - BC-2.02.005
verification_properties: []
lifecycle_status: consumed
introduced: "S-CLAROTY-DEVVULNREL-001"
last_evaluated: "2026-08-31"
last_eval_satisfaction: 1.00
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-DEVVULNREL-001 (HS-026 group). Tests BC-2.16.017 §Postconditions 1 (TOML table contract — ocsf_class = 'vulnerability_finding' → class_uid 2002) and §Postconditions 2 (Tier-1 columns: vulnerability_name → finding_info_title REQUIRED). Runs against live monroe sensor — requires bearer_token credential configured. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-DEVVULNREL-001-001: claroty_device_vulnerability_relations SELECT * wire shape: class_uid=2002 and finding_info_title column present

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-DEVVULNREL-001 (HS-026 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.017 §Postconditions 1 (TOML table contract — ocsf_class = "vulnerability_finding" → class_uid 2002 from the existing class_selector arm) and §Postconditions 2 Tier-1 columns (`vulnerability_name → finding_info_title` REQUIRED via `ocsf_field_to_arrow_name("finding_info.title")`)
**Gate:** Story-level holdout gate (HS-026) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the `claroty_device_vulnerability_relations` table is registered and
queryable via PrismQL against the live Claroty xDome (monroe) sensor, and that the TOML contract
specified in BC-2.16.017 §Postconditions 1 is correctly realized in the wire output:

1. The returned JSON rows carry `class_uid = 2002` — the integer class_uid for `vulnerability_finding`.
   This is the primary OCSF class contract verification: if the spec-engine dispatched the wrong OCSF
   class (e.g., 2004 detection_finding or 5001 inventory_info), or if the class_uid field is absent,
   this assertion fails. Note that `claroty_vulnerabilities` (BC-2.16.015) also uses class_uid 2002;
   a cross-table mix-up where the spec-engine dispatches the wrong table's class selector would
   also be caught here.

2. The returned JSON rows carry a column named `finding_info_title` — the Arrow field name for the
   Tier-1 mapping of `vulnerability_name` (source: `ocsf_field = "finding_info.title"`, then
   `ocsf_field_to_arrow_name` → `finding_info_title`). This is the REQUIRED Tier-1 column; its
   absence means the TOML spec was not parsed correctly or the ocsf_field_to_arrow_name transform
   was not applied.

3. The `finding_info_title` value in at least one row is a non-null, non-empty string — evidence
   that real device-vulnerability relation data was retrieved from the live sensor.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT * FROM claroty.claroty_device_vulnerability_relations LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response wire JSON contains a row with a column `class_uid` equal to `2002`
**And** the response wire JSON contains a row with a column `finding_info_title` that is a non-null, non-empty string

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor. Do NOT include
   the credential value in any output — reference the credential by the configuration key only (AD-017).

3. Start prism in MCP stdio mode with the claroty sensor spec included. Capture the full MCP stdio
   output and any stderr.

4. Wait for prism to be ready (startup completion log message or first JSON-RPC prompt).

5. Issue the MCP `query` tool call:
   `{"sql": "SELECT * FROM claroty.claroty_device_vulnerability_relations LIMIT 1"}`.

6. Capture the full raw wire-level JSON response from the MCP tool call.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.017 | §Postconditions 1: ocsf_class = "vulnerability_finding" → class_uid 2002 | Assertion 1: class_uid = 2002 in wire output |
| BC-2.16.017 | §Postconditions 2 Tier-1: vulnerability_name → ocsf_field = "finding_info.title" → Arrow field finding_info_title REQUIRED | Assertion 2: finding_info_title column present in row |
| BC-2.16.017 | §Postconditions 1: POST /api/v1/device_vulnerability_relations/, response_path = $.devices_vulnerabilities | End-to-end: table successfully queries live sensor |
| BC-2.02.005 | Claroty xDome OCSF field mapping — vulnerability_finding class_uid | Cross-verification of class_uid value |

---

## Verification Approach

1. Parse the wire-level JSON response from the MCP `query` tool call.

2. Locate the `rows` (or equivalent row array) in the response payload. If the response is an error
   object (contains `error_code` or similar), record as FAIL with observation "query returned error."

3. Inspect the first row's column list. Find the column named `class_uid`. Assert its integer value
   equals `2002`. If the column is absent or the value differs, record FAIL on "class_uid=2002"
   dimension.

4. Inspect the first row's column list for `finding_info_title`. Assert its value is a non-null,
   non-empty string. If the column is absent, record FAIL on "finding_info_title present" dimension.
   If the column is present but null or empty, record PARTIAL on the same dimension.

5. Do NOT assert any specific vulnerability name value — the live sensor's content varies; the
   structural assertion is sufficient.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query succeeds (no error)** (weight: 0.30): Does the MCP query return a non-error response
  with at least one row?
  Full credit (1.0): non-error response with ≥1 row.
  Zero credit (0.0): error response (column-not-found, sensor-unreachable, spec-parse error, or any other error).

- **class_uid = 2002 in wire output** (weight: 0.40): Does at least one returned row carry
  `class_uid = 2002`?
  Full credit (1.0): class_uid column present, value is integer 2002.
  Partial credit (0.3): class_uid column present but value is wrong (e.g., 2004 — wrong class mapping).
  Zero credit (0.0): class_uid column absent or query errored.

- **finding_info_title present and non-null** (weight: 0.30): Does at least one returned row carry
  a non-null `finding_info_title` string?
  Full credit (1.0): finding_info_title present, non-null, non-empty string.
  Partial credit (0.5): finding_info_title present but null or empty.
  Zero credit (0.0): finding_info_title column absent from row (ocsf_field_to_arrow_name transform not applied).

---

## Edge Conditions

- **Live sensor returns empty result set (zero rows):** Record as SETUP-FAILURE (sensor has no
  device-vulnerability relations) — not a behavioral FAIL. Note the observation and do not score
  on row-content dimensions. Score only the "query succeeds" dimension (which should be 1.0 if
  the response is a non-error empty result).

- **Sensor authentication failure (E-SENSOR-001 / 401):** Record as SETUP-FAILURE (credential
  misconfiguration or monroe connectivity issue) — not a behavioral FAIL.

- **`claroty_device_vulnerability_relations` table not registered (E-QUERY-038 or "table not
  found"):** This IS a behavioral FAIL — the TOML table block was not added or not parsed correctly.

- **`class_uid` is present but as a string `"2002"` rather than integer `2002`:** Record as PARTIAL
  (0.5) on the class_uid dimension — field present but type coercion incorrect.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DEVVULNREL-001-001 (satisfaction: X.XX) — claroty_device_vulnerability_relations wire-shape gap; check TOML table block registration and OCSF class_uid=2002 mapping (BC-2.16.017 §Postconditions 1) and finding_info_title Tier-1 column ocsf_field_to_arrow_name transform (BC-2.16.017 §Postconditions 2)"`

Do NOT disclose: the specific column values expected, the LIMIT value used, or the exact assertion
threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/device_vulnerability_relations/ |
| corpus_size | LIMIT 1 (single row sufficient for structural assertion) |
| known_edge_cases | Empty result set (no device-vulnerability relations in monitored OT network — record as SETUP-FAILURE, not behavioral FAIL); `devices_vulnerabilities` envelope key mismatch (E-SPEC parse error if response_path is wrong) |
| false_positive_threshold | Zero: class_uid=2002 and finding_info_title are structural wire-shape assertions, not content assertions |
| false_negative_threshold | Zero: if finding_info_title is absent, the OCSF Tier-1 column mapping is broken |

**Known-good corpus:** monroe Claroty xDome with ≥1 device-vulnerability relation — expected:
non-error response, class_uid=2002, finding_info_title non-null string.

**Known-problematic corpus:** An environment where the `claroty_device_vulnerability_relations`
table block was not added to the TOML spec — expected: E-QUERY-038 or table-not-found error.
This is the failure mode BC-2.16.017 §Postconditions 1 guards against.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-b-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-026 group for S-CLAROTY-DEVVULNREL-001. Wire-shape assertion: class_uid=2002 and finding_info_title Tier-1 column present in live monroe sensor output. BC-2.16.017 §Postconditions 1 and 2. SINGLE-USE. |
