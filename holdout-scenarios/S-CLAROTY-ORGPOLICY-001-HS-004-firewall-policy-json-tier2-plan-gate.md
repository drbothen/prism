---
document_type: holdout-scenario
level: L3
id: "HS-ORGPOL-001-004"
title: "claroty_organization_firewall_policies applied_group_pairs Json column in raw_extensions and Tier-2 plan-gate E-QUERY-038 for communication_conditions"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-C"
story_source: "S-CLAROTY-ORGPOLICY-001"
version: "1.0"
status: active
used: true
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-09-01"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.021-claroty-org-firewall-domain.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "f0cbf03"
traces_to: "BC-2.16.021"
behavioral_contracts:
  - BC-2.16.021
verification_properties: []
lifecycle_status: consumed
introduced: "S-CLAROTY-ORGPOLICY-001"
last_evaluated: "2026-09-01"
last_eval_satisfaction: 1.00
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-ORGPOLICY-001 (HS-028 group). Tests BC-2.16.021 §Postconditions 4 Json columns (applied_group_pairs in raw_extensions; communication_conditions as JSON array not string) and §Invariants (SELECT communication_conditions → E-QUERY-038; SELECT applied_group_pairs → E-QUERY-038). Also tests activity_name Tier-1 mapping from policy_action. Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ORGPOL-001-004: claroty_organization_firewall_policies applied_group_pairs Json column in raw_extensions and Tier-2 plan-gate E-QUERY-038 for communication_conditions

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-ORGPOLICY-001 (HS-028 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.021 §Postconditions 4 (Json column serialization — `communication_conditions`,
`related_alerts_ids`, `applied_group_pairs` in `raw_extensions` as JSON-typed values; `applied_group_pairs`
specifically distinguishes firewall domain from zone domain's `applied_zone_pairs`) and §Invariants
(Tier-2 plan-gate E-QUERY-038 by raw name) and §Postconditions 3 Tier-1 (`policy_action →
activity_name`)
**Gate:** Story-level holdout gate (HS-028) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the firewall policies table on three dimensions:

1. **Json column aggregation and typing:** `SELECT raw_extensions FROM claroty.claroty_organization_firewall_policies LIMIT 5`
   must return rows where `raw_extensions` contains `applied_group_pairs` as a key with a JSON
   array value (or null). This distinguishes the firewall domain's `applied_group_pairs` from
   the zone domain's `applied_zone_pairs` — confirming the correct firewall-specific column
   name was used.

2. **Tier-2 plan-gate:** `SELECT communication_conditions FROM claroty.claroty_organization_firewall_policies LIMIT 1`
   must return E-QUERY-038 (not a result set). Same assertion as HS-002 but for the firewall
   policies table to confirm parity between the two policy tables.

3. **Tier-1 activity_name mapping:** `SELECT activity_name FROM claroty.claroty_organization_firewall_policies LIMIT 3`
   must succeed and return rows where `activity_name` is "Allow", "Deny", or null — confirming
   the `policy_action → activity_name` Tier-1 mapping was applied.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)

**When** `SELECT raw_extensions FROM claroty.claroty_organization_firewall_policies LIMIT 5` is issued
**Then** the response is not an error
**And** at least one row's `raw_extensions` JSON object contains the key `applied_group_pairs`
**And** the value of `applied_group_pairs` is a JSON array or null (NOT a quoted-string serialization)
**And** the key is `applied_group_pairs` (NOT `applied_zone_pairs`)

**And When** `SELECT communication_conditions FROM claroty.claroty_organization_firewall_policies LIMIT 1` is issued
**Then** the response is an E-QUERY-038 error

**And When** `SELECT activity_name FROM claroty.claroty_organization_firewall_policies LIMIT 3` is issued
**Then** the response is not an error
**And** at least one row has `activity_name` that is "Allow", "Deny", or null

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017).

3. Start prism in MCP stdio mode. Wait for ready.

4. Issue the first MCP `query` tool call:
   `{"sql": "SELECT raw_extensions FROM claroty.claroty_organization_firewall_policies LIMIT 5"}`.
   Capture the full raw wire-level JSON response.

5. Issue the second MCP `query` tool call:
   `{"sql": "SELECT communication_conditions FROM claroty.claroty_organization_firewall_policies LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

6. Issue the third MCP `query` tool call:
   `{"sql": "SELECT activity_name FROM claroty.claroty_organization_firewall_policies LIMIT 3"}`.
   Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.021 | §Postconditions 4: applied_group_pairs Json column in raw_extensions as JSON array | Assertion 1: applied_group_pairs key present in raw_extensions with JSON array value |
| BC-2.16.021 | §Postconditions 4: applied_group_pairs (not applied_zone_pairs) — correct firewall column name | Assertion 1b: key name is applied_group_pairs, confirming firewall domain column was used |
| BC-2.16.021 | §Invariants: Tier-2 columns raise E-QUERY-038 by raw name | Assertion 2: SELECT communication_conditions → E-QUERY-038 |
| BC-2.16.021 | §Postconditions 3 Tier-1: policy_action → activity_name | Assertion 3: SELECT activity_name succeeds with Allow/Deny/null values |

---

## Verification Approach

1. Parse the wire-level JSON response from the first MCP `query` call.

2. If the response is an error or zero rows, record as FAIL/SETUP-FAILURE on "raw_extensions
   query succeeds" dimension.

3. For at least one row, inspect `raw_extensions`. Assert it contains the key `applied_group_pairs`.
   **Assert the key name is exactly `applied_group_pairs` (NOT `applied_zone_pairs`.)** A key
   named `applied_zone_pairs` indicates the firewall_policies table has a TOML authoring defect —
   it used the zone domain's column name by mistake (EC-016-021-010 in BC-2.16.021).

4. Inspect the value of `applied_group_pairs`. Assert it is a JSON array (including empty `[]`)
   or null — NOT a quoted string starting with `[` or `{`.

5. Parse the wire-level JSON response from the second MCP `query` call. Assert it is an error
   response (E-QUERY-038 or "column not found"). If it returns a result set, record FAIL on
   "Tier-2 plan-gate" dimension.

6. Parse the wire-level JSON response from the third MCP `query` call. Assert it is NOT an error.
   Assert at least one row carries `activity_name` with value "Allow", "Deny", or null. If
   `activity_name` column is absent (E-QUERY-038), the `policy_action → activity_name` Tier-1
   mapping was not applied — record FAIL on "activity_name Tier-1 mapping" dimension.

7. Do NOT assert the specific count of Allow vs Deny policies or the exact content of the
   applied_group_pairs arrays.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **raw_extensions query succeeds** (weight: 0.15): Non-error response with ≥1 row.
  Full credit (1.0): non-error, ≥1 row.
  Zero credit (0.0): error or zero rows (SETUP-FAILURE if genuinely no policies).

- **applied_group_pairs in raw_extensions as JSON** (weight: 0.30): `applied_group_pairs` key
  present in `raw_extensions`, value is JSON array or null (not quoted string), and key name is
  exactly `applied_group_pairs` (not `applied_zone_pairs`).
  Full credit (1.0): all conditions met.
  Partial credit (0.5): key present but value is quoted string (String type used).
  Zero credit (0.0): key absent OR key is `applied_zone_pairs` (wrong column name used).

- **Tier-2 plan-gate E-QUERY-038** (weight: 0.30): `SELECT communication_conditions` returns
  E-QUERY-038.
  Full credit (1.0): E-QUERY-038 returned.
  Partial credit (0.5): any error returned (plan-gate active but wrong code).
  Zero credit (0.0): result set returned instead of error.

- **activity_name Tier-1 mapping** (weight: 0.25): `SELECT activity_name` succeeds; at least
  one row has `activity_name` that is "Allow", "Deny", or null.
  Full credit (1.0): succeeds; activity_name present with valid value.
  Partial credit (0.3): succeeds but activity_name is null for all rows.
  Zero credit (0.0): E-QUERY-038 on activity_name (Tier-1 mapping not applied).

---

## Edge Conditions

- **Zero firewall_policies rows:** Record "raw_extensions query succeeds" and "applied_group_pairs"
  as SETUP-FAILURE. Tier-2 plan-gate and activity_name assertions are still evaluable.

- **`applied_zone_pairs` key in raw_extensions instead of `applied_group_pairs`:** This is a
  TOML authoring defect (used zone domain column name in firewall policies table). Record as
  FAIL on "applied_group_pairs" dimension. Send failure guidance without naming the column.

- **`communication_conditions` absent from raw_extensions (despite being a Json column):** This
  is distinct from the Tier-2 plan-gate — the plan-gate test queries it by raw name and expects
  E-QUERY-038. The raw_extensions test checks the aggregated JSON; if `communication_conditions`
  is missing from raw_extensions, it means that Json column was not serialized (TOML column
  declaration missing). Score "Json columns in raw_extensions" dimension accordingly.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ORGPOL-001-004 (satisfaction: X.XX) — claroty_organization_firewall_policies gap; check applied_group_pairs Json column in raw_extensions (must be JSON array not string — BC-2.16.021 §Postconditions 4), Tier-2 plan-gate E-QUERY-038 (BC-2.16.021 §Invariants), and policy_action→activity_name Tier-1 mapping (BC-2.16.021 §Postconditions 3)"`

Do NOT disclose: the column names queried, the LIMIT values, or the specific type-check
(JSON array vs string) detail.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/organization_fw_group_policies/ |
| corpus_size | LIMIT 5 for raw_extensions; LIMIT 1 for plan-gate; LIMIT 3 for activity_name |
| known_edge_cases | Zero rows (no firewall policies — SETUP-FAILURE); applied_zone_pairs key instead of applied_group_pairs (TOML naming defect); communication_conditions as quoted string (wrong column_type) |
| false_positive_threshold | Zero: applied_group_pairs name check and JSON array type check are structural assertions |
| false_negative_threshold | Zero: applied_zone_pairs key instead of applied_group_pairs is caught by exact key name check |

**Known-good corpus:** monroe with ≥1 firewall policy — expected: applied_group_pairs in
raw_extensions as JSON array; E-QUERY-038 on communication_conditions direct select;
activity_name present with "Allow"/"Deny" values.

**Known-problematic corpus:** A claroty.sensor.toml where `applied_group_pairs` is accidentally
declared as `applied_zone_pairs` (copy-paste from zone_policies) — expected: `applied_zone_pairs`
key in raw_extensions, `applied_group_pairs` absent (wrong column name used).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-028 group for S-CLAROTY-ORGPOLICY-001. Three-assertion scenario: (1) applied_group_pairs Json in raw_extensions as JSON array (distinguishes firewall vs zone domain by exact column name check), (2) Tier-2 plan-gate E-QUERY-038 for communication_conditions, (3) activity_name Tier-1 mapping from policy_action. BC-2.16.021 §Postconditions 3, 4, §Invariants. SINGLE-USE. |
