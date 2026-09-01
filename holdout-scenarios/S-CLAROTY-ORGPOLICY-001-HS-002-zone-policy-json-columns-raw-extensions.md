---
document_type: holdout-scenario
level: L3
id: "HS-ORGPOL-001-002"
title: "claroty_organization_zone_policies Json columns serialized into raw_extensions: communication_conditions, related_alerts_ids, applied_zone_pairs present as JSON values"
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
  - ".factory/specs/behavioral-contracts/BC-2.16.020-claroty-org-zone-domain.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "a91185c"
traces_to: "BC-2.16.020"
behavioral_contracts:
  - BC-2.16.020
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-ORGPOLICY-001 (HS-028 group). Tests BC-2.16.020 §Postconditions 4 Json column serialization — communication_conditions, related_alerts_ids, applied_zone_pairs must appear as JSON values inside raw_extensions (not null, not raw string tokens). Also tests Tier-2 plan-gate: SELECT communication_conditions → E-QUERY-038. Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ORGPOL-001-002: claroty_organization_zone_policies Json columns serialized into raw_extensions: communication_conditions, related_alerts_ids, applied_zone_pairs present as JSON values

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-ORGPOLICY-001 (HS-028 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.020 §Postconditions 4 (Json column serialization — `communication_conditions`,
`related_alerts_ids`, `applied_zone_pairs` must appear in `raw_extensions` as JSON-typed values,
not null and not raw string tokens) and §Invariants (Tier-2 columns raise E-QUERY-038 by raw name)
**Gate:** Story-level holdout gate (HS-028) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the three Json columns of `claroty_organization_zone_policies`
(`communication_conditions`, `related_alerts_ids`, `applied_zone_pairs`) are correctly handled
by the spec-engine:

1. When `SELECT raw_extensions FROM claroty.claroty_organization_zone_policies LIMIT 5` is
   issued, the `raw_extensions` JSON object in at least one row contains the keys
   `communication_conditions`, `related_alerts_ids`, and `applied_zone_pairs`. This proves the
   Json columns were aggregated into `raw_extensions`.

2. The value of `communication_conditions` in `raw_extensions` is a JSON array (or null if
   the policy has no conditions), NOT a raw string serialization of the array (e.g., NOT
   `"[{\"src_zone\": ...}]"`). A raw string means `column_type = "String"` was used instead
   of `"Json"` — this is the P1 TOML authoring defect guarded by BC-2.16.020 §Invariants.

3. When `SELECT communication_conditions FROM claroty.claroty_organization_zone_policies LIMIT 1`
   is issued, the result is an E-QUERY-038 error (column-not-found at plan time), NOT a result
   set. This confirms the Tier-2 plan-gate is active.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT raw_extensions FROM claroty.claroty_organization_zone_policies LIMIT 5` is issued via the MCP `query` tool
**Then** the response is not an error
**And** at least one row's `raw_extensions` JSON object contains keys `communication_conditions`, `related_alerts_ids`, and `applied_zone_pairs`
**And** the value of `communication_conditions` in `raw_extensions` is a JSON array or null, NOT a quoted-string serialization of an array

**And When** `SELECT communication_conditions FROM claroty.claroty_organization_zone_policies LIMIT 1` is issued
**Then** the response is an E-QUERY-038 error (column not found at plan time)
**And** the error's `available_columns` includes `raw_extensions` but does NOT include `communication_conditions`

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 — do
   not include credential value in output).

3. Start prism in MCP stdio mode. Wait for ready.

4. Issue the first MCP `query` tool call:
   `{"sql": "SELECT raw_extensions FROM claroty.claroty_organization_zone_policies LIMIT 5"}`.
   Capture the full raw wire-level JSON response.

5. Issue the second MCP `query` tool call:
   `{"sql": "SELECT communication_conditions FROM claroty.claroty_organization_zone_policies LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.020 | §Postconditions 4: Json columns aggregated into raw_extensions | Assertion 1: raw_extensions contains communication_conditions, related_alerts_ids, applied_zone_pairs keys |
| BC-2.16.020 | §Postconditions 4: Json columns serialized as JSON values, not string tokens | Assertion 2: communication_conditions value is JSON array or null (not a quoted string) |
| BC-2.16.020 | §Invariants: Tier-2 columns raise E-QUERY-038 by raw name | Assertion 3: SELECT communication_conditions → E-QUERY-038 |

---

## Verification Approach

1. Parse the wire-level JSON response from the first MCP `query` call.

2. Locate the `rows` array. If the response is an error, record as FAIL on "raw_extensions
   query succeeds" dimension.

3. For at least one row, inspect `raw_extensions`. Assert it is a JSON object (not null). Assert
   it contains the key `communication_conditions`. Also check for `related_alerts_ids` and
   `applied_zone_pairs`. If fewer than 2 of the 3 Json column keys are present, record FAIL on
   "Json columns in raw_extensions" dimension.

4. Inspect the value of `communication_conditions` in `raw_extensions`. If it is a string
   (i.e., the JSON type is "string" and the string starts with `[` or `{`), record FAIL on
   "Json not string-serialized" dimension. Accepted values: JSON array (including empty `[]`),
   or null (if the policy has no communication conditions). A quoted string like
   `"[{\"src_zone\":\"Zone A\"}]"` is a FAIL — it means `column_type = "String"` was used.

5. Parse the wire-level JSON response from the second MCP `query` call. Assert it is an error
   response (contains `error_code` matching E-QUERY-038 pattern, or a "column not found"
   message). If the second query returns a result set instead of an error, record FAIL on
   "Tier-2 plan-gate E-QUERY-038" dimension.

6. If E-QUERY-038 is returned, optionally assert `available_columns` contains `raw_extensions`
   but NOT `communication_conditions`. This confirms the error surfaces the correct available set.

7. **Empty zone_policies case:** If the sensor has zones but zero zone_policies (all zero rows),
   record as SETUP-FAILURE for the raw_extensions assertion only. The plan-gate assertion (step 5)
   can still be evaluated without any rows.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **raw_extensions query succeeds** (weight: 0.15): Does the first MCP query return a non-error
  response?
  Full credit (1.0): non-error response with ≥1 row.
  Zero credit (0.0): error response or zero rows (SETUP-FAILURE, not behavioral FAIL — exclude
  from scoring if zero rows).

- **Json columns present in raw_extensions** (weight: 0.30): Do returned rows carry
  `communication_conditions`, `related_alerts_ids`, and `applied_zone_pairs` as keys in
  `raw_extensions`?
  Full credit (1.0): all 3 Json column keys present.
  Partial credit (0.5): 1-2 of 3 keys present.
  Zero credit (0.0): raw_extensions absent or none of the 3 keys present.

- **Json columns not string-serialized** (weight: 0.30): Is `communication_conditions` a JSON
  array or null (not a quoted string)?
  Full credit (1.0): JSON array value (including empty `[]`) or null.
  Zero credit (0.0): string value starting with `[` or `{` (indicates String type was used).

- **Tier-2 plan-gate E-QUERY-038** (weight: 0.25): Does `SELECT communication_conditions`
  return E-QUERY-038?
  Full credit (1.0): E-QUERY-038 error; `available_columns` contains `raw_extensions`.
  Partial credit (0.5): error returned but not E-QUERY-038 (still a plan-gate).
  Zero credit (0.0): result set returned instead of error (Tier-2 guard not working).

---

## Edge Conditions

- **All zone policies have empty `communication_conditions` arrays:** Value is `[]` (JSON array) —
  full credit on "Json not string-serialized" dimension; `[]` is a valid JSON array, not null.

- **Live sensor has zero zone_policies rows:** Record "raw_extensions query succeeds" and "Json
  columns present" as SETUP-FAILURE (exclude from scoring). Tier-2 plan-gate assertion is still
  evaluable.

- **`applied_zone_pairs` present but `related_alerts_ids` absent (no triggered alerts):**
  Record 2/3 keys present (partial credit on "Json columns in raw_extensions").

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ORGPOL-001-002 (satisfaction: X.XX) — claroty_organization_zone_policies Json column gap; check column_type='json' for communication_conditions/related_alerts_ids/applied_zone_pairs in TOML spec (BC-2.16.020 §Postconditions 4) and Tier-2 plan-gate E-QUERY-038 (BC-2.16.020 §Invariants)"`

Do NOT disclose: the specific column names queried, the LIMIT value, or the exact assertion
threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/organization_zone_policies/ |
| corpus_size | LIMIT 5 for raw_extensions query; LIMIT 1 for plan-gate query |
| known_edge_cases | Zero zone_policies (SETUP-FAILURE for raw_extensions assertion); empty communication_conditions arrays (valid: JSON `[]`) |
| false_positive_threshold | Zero: Json-vs-string type is a structural serialization assertion |
| false_negative_threshold | Zero: if communication_conditions is a quoted string, column_type is wrong |

**Known-good corpus:** monroe with ≥1 zone policy — expected: raw_extensions contains
communication_conditions as JSON array; E-QUERY-038 on direct column reference.

**Known-problematic corpus:** A claroty.sensor.toml where communication_conditions is declared
as `column_type = "String"` — expected: communication_conditions value in raw_extensions is a
quoted JSON string (starting with `[`), not a JSON array.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-028 group for S-CLAROTY-ORGPOLICY-001. Json column serialization assertion: communication_conditions/related_alerts_ids/applied_zone_pairs in raw_extensions as JSON values (not string tokens). Tier-2 plan-gate: SELECT communication_conditions → E-QUERY-038. BC-2.16.020 §Postconditions 4 + §Invariants. SINGLE-USE. |
