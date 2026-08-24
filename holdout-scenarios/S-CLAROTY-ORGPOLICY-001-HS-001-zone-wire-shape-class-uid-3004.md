---
document_type: holdout-scenario
level: L3
id: "HS-ORGPOL-001-001"
title: "claroty_organization_zones SELECT * wire shape: class_uid=3004, name Tier-1 REQUIRED present from zone_name→entity_management name, raw_extensions with zone Tier-2 fields"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-C"
story_source: "S-CLAROTY-ORGPOLICY-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-24"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.020-claroty-org-zone-domain.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "f660fcc"
traces_to: "BC-2.16.020"
behavioral_contracts:
  - BC-2.16.020
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-ORGPOLICY-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-ORGPOLICY-001 (HS-028 group). Tests BC-2.16.020 §Postconditions 1 (TOML table contract — ocsf_class = 'entity_management' → class_uid 3004) and §Postconditions 3 Tier-1 (zone_name → name REQUIRED; zone_description → comment; enabled → status_code; updated_by → actor_user_name) and Tier-2 (raw_extensions present with zone fields). Runs against live monroe sensor — requires bearer_token credential configured. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ORGPOL-001-001: claroty_organization_zones SELECT * wire shape: class_uid=3004, name Tier-1 REQUIRED present from zone_name→entity_management name, raw_extensions with zone Tier-2 fields

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-ORGPOLICY-001 (HS-028 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.020 §Postconditions 1 (TOML table contract — ocsf_class = "entity_management"
→ class_uid 3004 from the existing class_selector arm) and §Postconditions 3 Tier-1
(`zone_name → name` REQUIRED; `zone_description → comment`; `enabled → status_code`;
`updated_by → actor_user_name`) and Tier-2 (`raw_extensions` present with zone inventory fields)
**Gate:** Story-level holdout gate (HS-028) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the `claroty_organization_zones` table is registered and queryable
via PrismQL against the live Claroty xDome (monroe) sensor, and that the wire output matches
BC-2.16.020 §Postconditions 1 and 3:

1. The returned JSON rows carry `class_uid = 3004` — the integer class_uid for
   `entity_management`. If the class_selector resolved to a wrong class (e.g., 3004 is correct;
   2002 or 5001 are wrong) or if class_uid is absent from the response, this assertion fails.

2. The returned JSON rows carry a column named `name` — the Arrow field name for the Tier-1
   mapping of `zone_name` (source: `ocsf_field = "name"`, then `ocsf_field_to_arrow_name` →
   `name`). This is the REQUIRED Tier-1 column; its absence means the TOML spec was not parsed
   correctly or the REQUIRED attribute was not applied.

3. The `name` value in at least one row is a non-null, non-empty string — evidence that real
   zone data was retrieved from the live sensor.

4. The returned rows carry a `raw_extensions` column containing a JSON object with at least one
   zone-specific Tier-2 key (e.g., `zone_source`, `priority`, `device_conditions`,
   `attributed_devices`, or `last_update`) — evidence that Tier-2 columns were aggregated
   correctly.

5. The rows do NOT carry a column named `zone_name` as a standalone Arrow field — if `zone_name`
   appears as a top-level column instead of `name`, the Tier-1 OCSF rename was not applied.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT * FROM claroty.claroty_organization_zones LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response wire JSON contains a row with a column `class_uid` equal to `3004`
**And** the response wire JSON contains a row with a column `name` that is a non-null, non-empty string
**And** the response wire JSON contains a row with a column `raw_extensions` that is a JSON object with at least one zone-specific Tier-2 key
**And** the response wire JSON does NOT contain a top-level column named `zone_name`

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor. Do NOT include
   the credential value in any output — reference the credential by the configuration key only (AD-017).

3. Start prism in MCP stdio mode with the claroty sensor spec included. Capture the full MCP stdio
   output and any stderr.

4. Wait for prism to be ready (startup completion log message or first JSON-RPC prompt).

5. Issue the MCP `query` tool call:
   `{"sql": "SELECT * FROM claroty.claroty_organization_zones LIMIT 1"}`.

6. Capture the full raw wire-level JSON response from the MCP tool call.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.020 | §Postconditions 1: ocsf_class = "entity_management" → class_uid 3004 | Assertion 1: class_uid = 3004 in wire output |
| BC-2.16.020 | §Postconditions 3 Tier-1: zone_name → ocsf_field = "name" → Arrow field name REQUIRED | Assertion 2: name column present and non-null; not zone_name |
| BC-2.16.020 | §Postconditions 3 Tier-2: 7 Tier-2 columns aggregate into raw_extensions | Assertion 3: raw_extensions JSON object with zone-specific keys present |
| BC-2.16.020 | §Postconditions 1: POST /api/v1/organization_zones/, response_path = $.organization_zones | End-to-end: table successfully queries live sensor |

---

## Verification Approach

1. Parse the wire-level JSON response from the MCP `query` tool call.

2. Locate the `rows` (or equivalent row array) in the response payload. If the response is an
   error object (contains `error_code` or similar), record as FAIL with observation "query
   returned error."

3. Inspect the first row's column list. Find the column named `class_uid`. Assert its integer
   value equals `3004`. If the column is absent or the value differs, record FAIL on the
   "class_uid=3004" dimension.

4. Inspect the first row's column list for `name`. Assert its value is a non-null, non-empty
   string. If the column is absent, record FAIL on "name present" dimension.

5. Assert no top-level column named `zone_name` exists in the first row. If `zone_name` appears
   as a standalone Arrow column, record FAIL on "Tier-1 rename applied" dimension.

6. Inspect the first row's column list for `raw_extensions`. Assert it is a JSON object (not
   null, not a string) containing at least one key that belongs to the Tier-2 column set
   (`zone_source`, `priority`, `device_conditions`, `attributed_devices`,
   `exportable_attributed_devices`, `created_time`, `last_update`). If `raw_extensions` is
   absent or null, record FAIL on "raw_extensions Tier-2 aggregation" dimension.

7. Do NOT assert specific zone name values or specific Tier-2 field values — the live sensor's
   content varies; structural assertions are sufficient.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query succeeds (no error)** (weight: 0.20): Does the MCP query return a non-error response
  with at least one row?
  Full credit (1.0): non-error response with ≥1 row.
  Zero credit (0.0): error response of any kind.

- **class_uid = 3004 in wire output** (weight: 0.35): Does at least one returned row carry
  `class_uid = 3004`?
  Full credit (1.0): class_uid column present, value is integer 3004.
  Partial credit (0.3): class_uid column present but value is wrong (e.g., 2002 or 5001).
  Zero credit (0.0): class_uid column absent or query errored.

- **name present and non-null (REQUIRED Tier-1)** (weight: 0.25): Does at least one returned
  row carry a non-null `name` string, AND does the row NOT carry a standalone `zone_name` column?
  Full credit (1.0): `name` present non-null; `zone_name` absent as standalone column.
  Partial credit (0.5): `name` present but null or empty.
  Zero credit (0.0): `name` column absent OR `zone_name` appears as standalone (Tier-1 rename
  not applied).

- **raw_extensions present with Tier-2 keys** (weight: 0.20): Does at least one returned row
  carry a `raw_extensions` JSON object with at least one zone-specific Tier-2 key?
  Full credit (1.0): raw_extensions present, is a JSON object, contains at least one Tier-2 key.
  Partial credit (0.5): raw_extensions present but null or empty object.
  Zero credit (0.0): raw_extensions column absent.

---

## Edge Conditions

- **Live sensor returns empty result set (zero zones):** Record as SETUP-FAILURE (no zones
  configured in the xDome instance) — not a behavioral FAIL.

- **Sensor authentication failure (E-SENSOR-001 / 401):** Record as SETUP-FAILURE — not a
  behavioral FAIL.

- **`claroty_organization_zones` table not registered (E-QUERY-038 or "table not found"):**
  This IS a behavioral FAIL — the TOML table block was not added or not parsed correctly.

- **`class_uid` present as string `"3004"` rather than integer:** Record as PARTIAL (0.5) on
  the class_uid dimension.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ORGPOL-001-001 (satisfaction: X.XX) — claroty_organization_zones wire-shape gap; check TOML table block registration and OCSF class_uid=3004 mapping (BC-2.16.020 §Postconditions 1) and zone_name→name Tier-1 REQUIRED column ocsf_field_to_arrow_name transform (BC-2.16.020 §Postconditions 3)"`

Do NOT disclose: the specific column values expected, the LIMIT value used, or the exact
assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/organization_zones/ |
| corpus_size | LIMIT 1 (single row sufficient for structural assertion) |
| known_edge_cases | Empty result set (no zones configured in xDome — SETUP-FAILURE, not behavioral FAIL) |
| false_positive_threshold | Zero: class_uid=3004 and name Tier-1 REQUIRED are structural wire-shape assertions |
| false_negative_threshold | Zero: if name is absent, the OCSF Tier-1 column mapping is broken |

**Known-good corpus:** monroe Claroty xDome with ≥1 organization zone — expected: non-error
response, class_uid=3004, name non-null string, raw_extensions with zone Tier-2 keys.

**Known-problematic corpus:** An environment where the `claroty_organization_zones` TOML table
block was not added — expected: "table not found" error or E-QUERY-038. This is the failure
mode BC-2.16.020 §Postconditions 1 guards against.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-028 group for S-CLAROTY-ORGPOLICY-001. Wire-shape assertion: class_uid=3004 and name Tier-1 REQUIRED column (from zone_name→entity_management name) present in live monroe sensor output. BC-2.16.020 §Postconditions 1 and 3. SINGLE-USE. |
