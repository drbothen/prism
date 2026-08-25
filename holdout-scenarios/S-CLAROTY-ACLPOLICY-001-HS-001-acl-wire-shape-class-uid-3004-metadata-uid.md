---
document_type: holdout-scenario
level: L3
id: "HS-ACLPOLICY-001-001"
title: "claroty_organization_acl_policies SELECT * wire shape: class_uid=3004, metadata_uid Tier-1 REQUIRED present from policy_id→metadata.uid, name Tier-1 present, raw_extensions with ACL Tier-2 fields"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-C"
story_source: "S-CLAROTY-ACLPOLICY-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-24"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.022-claroty-org-acl-policies.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "68aa88c"
traces_to: "BC-2.16.022"
behavioral_contracts:
  - BC-2.16.022
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-ACLPOLICY-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-ACLPOLICY-001 (HS-029 group). Tests BC-2.16.022 §Postconditions 1 (TOML table contract — ocsf_class = 'entity_management' → class_uid 3004 from existing class_selector arm) and §Postconditions 2 Tier-1 (policy_id → ocsf_field = 'metadata.uid' → Arrow field 'metadata_uid' REQUIRED; policy_name → name; policy_updated_by → actor_user_name; policy_notes → comment) and Tier-2 (raw_extensions present with ACL fields). Runs against live monroe sensor — requires bearer_token credential configured. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ACLPOLICY-001-001: claroty_organization_acl_policies SELECT * wire shape: class_uid=3004, metadata_uid Tier-1 REQUIRED present from policy_id→metadata.uid, name Tier-1 present, raw_extensions with ACL Tier-2 fields

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-ACLPOLICY-001 (HS-029 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.022 §Postconditions 1 (TOML table contract — ocsf_class = "entity_management"
→ class_uid 3004 from the existing class_selector arm) and §Postconditions 2 Tier-1
(`policy_id → metadata.uid → metadata_uid` REQUIRED; `policy_name → name`;
`policy_updated_by → actor_user_name`; `policy_notes → comment`) and Tier-2
(`raw_extensions` present with ACL policy fields)
**Gate:** Story-level holdout gate (HS-029) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the `claroty_organization_acl_policies` table is registered and
queryable via PrismQL against the live Claroty xDome (monroe) sensor, and that the wire output
matches BC-2.16.022 §Postconditions 1 and 2:

1. The returned JSON rows carry `class_uid = 3004` — the integer class_uid for
   `entity_management`. If the class_selector resolved to a wrong class (e.g., 2002 or 5001),
   or if class_uid is absent, this assertion fails.

2. The returned JSON rows carry a column named `metadata_uid` — the Arrow field name for the
   Tier-1 mapping of `policy_id` (source: `ocsf_field = "metadata.uid"`, then
   `ocsf_field_to_arrow_name` → `metadata_uid` by replacing dot with underscore). This is the
   REQUIRED Tier-1 column. Its absence means the TOML spec was not parsed correctly or the
   `ocsf_field_to_arrow_name` transform was not applied.

3. The `metadata_uid` value in at least one row is a non-null, non-empty string — evidence that
   real ACL policy data was retrieved from the live sensor.

4. The returned rows carry a column named `name` — the Arrow field name for the Tier-1 mapping
   of `policy_name` (source: `ocsf_field = "name"` → Arrow `name`).

5. The returned rows carry a `raw_extensions` column containing a JSON object with at least one
   ACL-specific Tier-2 key (e.g., `policy_source`, `policy_acl_type`, `policy_acl`,
   `applied_models`, `matching_devices`, `policy_creation_date`, or `policy_last_updated`).

6. The rows do NOT carry a column named `policy_id` as a standalone Arrow field — if `policy_id`
   appears as a top-level column instead of `metadata_uid`, the Tier-1 OCSF rename was not
   applied.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT * FROM claroty.claroty_organization_acl_policies LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response wire JSON contains a row with a column `class_uid` equal to `3004`
**And** the response wire JSON contains a row with a column `metadata_uid` that is a non-null, non-empty string
**And** the response wire JSON contains a row with a column `name` (present, possibly null)
**And** the response wire JSON contains a row with a column `raw_extensions` that is a JSON object with at least one ACL-specific Tier-2 key
**And** the response wire JSON does NOT contain a top-level column named `policy_id`

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor. Do NOT
   include the credential value in any output — reference the credential by the configuration
   key only (AD-017).

3. Start prism in MCP stdio mode with the claroty sensor spec included. Capture the full MCP
   stdio output and any stderr.

4. Wait for prism to be ready (startup completion log message or first JSON-RPC prompt).

5. Issue the MCP `query` tool call:
   `{"sql": "SELECT * FROM claroty.claroty_organization_acl_policies LIMIT 1"}`.

6. Capture the full raw wire-level JSON response from the MCP tool call.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.022 | §Postconditions 1: ocsf_class = "entity_management" → class_uid 3004 | Assertion 1: class_uid = 3004 in wire output |
| BC-2.16.022 | §Postconditions 2 Tier-1: policy_id → ocsf_field = "metadata.uid" → Arrow metadata_uid REQUIRED | Assertion 2+3: metadata_uid column present non-null; policy_id NOT standalone |
| BC-2.16.022 | §Postconditions 2 Tier-1: policy_name → ocsf_field = "name" → Arrow name | Assertion 4: name column present |
| BC-2.16.022 | §Postconditions 2 Tier-2: 7 Tier-2 columns aggregate into raw_extensions | Assertion 5: raw_extensions JSON object with ACL-specific keys present |
| BC-2.16.022 | §Postconditions 1: POST /api/v1/organization_acl_policies/, response_path = $.organization_acl_policies | End-to-end: table successfully queries live sensor |

---

## Verification Approach

1. Parse the wire-level JSON response from the MCP `query` tool call.

2. Locate the `rows` (or equivalent row array) in the response payload. If the response is an
   error object (contains `error_code` or similar), record as FAIL with observation "query
   returned error."

3. Inspect the first row's column list. Find the column named `class_uid`. Assert its integer
   value equals `3004`. If the column is absent or the value differs, record FAIL on the
   "class_uid=3004" dimension.

4. Inspect the first row's column list for `metadata_uid`. Assert its value is a non-null,
   non-empty string. If the column is absent, record FAIL on "metadata_uid present" dimension.

5. Assert no top-level column named `policy_id` exists in the first row. If `policy_id` appears
   as a standalone Arrow column, record FAIL on "Tier-1 rename applied" dimension.

6. Inspect the first row's column list for `name`. Assert the column is present (value may be
   null if the policy_name field was absent in that row). If the column is entirely absent,
   record PARTIAL (0.5) on the "name Tier-1 mapped" dimension.

7. Inspect the first row's column list for `raw_extensions`. Assert it is a JSON object (not
   null, not a string) containing at least one key that belongs to the Tier-2 column set
   (`policy_source`, `policy_acl_type`, `policy_acl`, `applied_models`, `matching_devices`,
   `policy_creation_date`, `policy_last_updated`). If `raw_extensions` is absent or null,
   record FAIL on "raw_extensions Tier-2 aggregation" dimension.

8. Do NOT assert specific policy_id values, specific policy_name strings, or specific
   raw_extensions key values — the live sensor's content varies; structural assertions are
   sufficient.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query succeeds (no error)** (weight: 0.15): Does the MCP query return a non-error response
  with at least one row?
  Full credit (1.0): non-error response with ≥1 row.
  Zero credit (0.0): error response of any kind.

- **class_uid = 3004 in wire output** (weight: 0.30): Does at least one returned row carry
  `class_uid = 3004`?
  Full credit (1.0): class_uid column present, value is integer 3004.
  Partial credit (0.3): class_uid column present but value is wrong (e.g., 2002 or 5001).
  Zero credit (0.0): class_uid column absent or query errored.

- **metadata_uid present and non-null (REQUIRED Tier-1)** (weight: 0.30): Does at least one
  returned row carry a non-null `metadata_uid` string, AND does the row NOT carry a standalone
  `policy_id` column?
  Full credit (1.0): `metadata_uid` present non-null; `policy_id` absent as standalone column.
  Partial credit (0.5): `metadata_uid` present but null, OR `policy_id` appears as standalone.
  Zero credit (0.0): `metadata_uid` column absent entirely.

- **name Tier-1 mapped** (weight: 0.10): Does the row carry a column `name` (may be null)?
  Full credit (1.0): `name` column present.
  Zero credit (0.0): `name` column absent.

- **raw_extensions present with Tier-2 keys** (weight: 0.15): Does at least one returned row
  carry a `raw_extensions` JSON object with at least one ACL-specific Tier-2 key?
  Full credit (1.0): raw_extensions present, is a JSON object, contains at least one Tier-2 key.
  Partial credit (0.5): raw_extensions present but null or empty object.
  Zero credit (0.0): raw_extensions column absent.

---

## Edge Conditions

- **Live sensor returns empty result set (zero ACL policies):** Record as SETUP-FAILURE (no
  ACL policies configured in the xDome instance) — not a behavioral FAIL.

- **Sensor authentication failure (E-SENSOR-001 / 401):** Record as SETUP-FAILURE — not a
  behavioral FAIL.

- **`claroty_organization_acl_policies` table not registered (E-QUERY-038 or "table not found"):**
  This IS a behavioral FAIL — the TOML table block was not added or not parsed correctly.

- **API returns 422 validation error ("policy_acl_syntax required"):** This IS a behavioral FAIL
  — the `body_template` does not include the mandatory `policy_acl_syntax` field.

- **`class_uid` present as string `"3004"` rather than integer:** Record as PARTIAL (0.5) on
  the class_uid dimension.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ACLPOLICY-001-001 (satisfaction: X.XX) — claroty_organization_acl_policies wire-shape gap; check TOML table block registration and OCSF class_uid=3004 mapping (BC-2.16.022 §Postconditions 1) and policy_id→metadata.uid→metadata_uid REQUIRED Tier-1 column ocsf_field_to_arrow_name transform (BC-2.16.022 §Postconditions 2)"`

Do NOT disclose: the specific column values expected, the LIMIT value used, or the exact
assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/organization_acl_policies/ |
| corpus_size | LIMIT 1 (single row sufficient for structural assertion) |
| known_edge_cases | Empty result set (no ACL policies configured in xDome — SETUP-FAILURE, not behavioral FAIL); 422 from missing policy_acl_syntax (body_template misconfiguration — FAIL) |
| false_positive_threshold | Zero: class_uid=3004 and metadata_uid REQUIRED are structural wire-shape assertions |
| false_negative_threshold | Zero: if metadata_uid is absent, the OCSF Tier-1 column mapping is broken |

**Known-good corpus:** monroe Claroty xDome with ≥1 ACL policy — expected: non-error response,
class_uid=3004, metadata_uid non-null UUID string, raw_extensions with ACL Tier-2 keys.

**Known-problematic corpus:** An environment where the `claroty_organization_acl_policies` TOML
table block was not added — expected: "table not found" error or E-QUERY-038. This is the
failure mode BC-2.16.022 §Postconditions 1 guards against.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-c-f2-spec-evolution-g6 | 2026-08-24 | product-owner | Initial authoring. HS-029 group for S-CLAROTY-ACLPOLICY-001. Wire-shape assertion: class_uid=3004 and metadata_uid REQUIRED Tier-1 column (from policy_id→metadata.uid via ocsf_field_to_arrow_name) present in live monroe sensor output. BC-2.16.022 §Postconditions 1 and 2. SINGLE-USE. |
