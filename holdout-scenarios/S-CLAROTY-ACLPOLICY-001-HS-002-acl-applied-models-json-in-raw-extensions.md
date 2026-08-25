---
document_type: holdout-scenario
level: L3
id: "HS-ACLPOLICY-001-002"
title: "claroty_organization_acl_policies: applied_models Json column in raw_extensions is a JSON array value (not a quoted string), confirming column_type=json applied"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-ACLPOLICY-001 (HS-029 group). Tests BC-2.16.022 §Postconditions 5 (Json column serialization behavior) and §Invariants (applied_models MUST be declared column_type=json; String would serialize array as quoted string token). The key behavioral distinction: an array value in raw_extensions.applied_models should be a JSON array [...], not a string '[...]'. Runs against live monroe sensor — requires bearer_token credential configured. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ACLPOLICY-001-002: claroty_organization_acl_policies: applied_models Json column in raw_extensions is a JSON array value (not a quoted string), confirming column_type=json applied

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-ACLPOLICY-001 (HS-029 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.022 §Postconditions 5 (Json column serialization behavior —
`applied_models` is a nested array of device model strings; the spec-engine serializes it into
`raw_extensions` as a JSON-typed value when `column_type = "json"` is declared; an empty array
serializes as `[]` not null; declaring as `String` would produce a quoted string token)
and §Invariants (`applied_models` MUST be declared `column_type = "json"`)
**Gate:** Story-level holdout gate (HS-029) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the `applied_models` column in `claroty_organization_acl_policies`
is correctly typed as Json and serialized as a proper JSON array value (not a quoted string)
inside `raw_extensions`. This is the critical property that distinguishes `column_type = "json"`
from `column_type = "string"`:

1. `SELECT raw_extensions FROM claroty.claroty_organization_acl_policies LIMIT 5` should
   return rows where `raw_extensions` is a JSON object containing the key `applied_models`.

2. The value at `raw_extensions.applied_models` should be a **JSON array** (either `[]` for
   empty, or `["model1", "model2", ...]` for populated). It must NOT be a quoted string like
   `"[\"model1\", \"model2\"]"` — that would indicate `column_type = "string"` was used.

3. `SELECT applied_models FROM claroty.claroty_organization_acl_policies LIMIT 1` should raise
   `E-QUERY-038` — `applied_models` is a Tier-2 column and MUST NOT be accessible as a
   standalone Arrow field.

4. The `policy_acl` Tier-2 column (raw ACL text string) should also be present as a string key
   inside `raw_extensions` — this confirms that both string Tier-2 columns and Json Tier-2
   columns are aggregated together.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT raw_extensions FROM claroty.claroty_organization_acl_policies LIMIT 5` is issued via the MCP `query` tool
**Then** the response is not an error
**And** at least one returned row's `raw_extensions` JSON object contains the key `applied_models`
**And** the value of `raw_extensions.applied_models` is a JSON array (either empty `[]` or a non-empty array of strings)
**And** the value of `raw_extensions.applied_models` is NOT a quoted string (e.g., not `"[...]"`)

**And when** `SELECT applied_models FROM claroty.claroty_organization_acl_policies LIMIT 1` is issued
**Then** the response is an error with error code `E-QUERY-038`

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor. Do NOT
   include the credential value in any output — reference the credential by the configuration
   key only (AD-017).

3. Start prism in MCP stdio mode with the claroty sensor spec included. Capture all MCP stdio
   output and stderr.

4. Wait for prism to be ready (startup completion log message or first JSON-RPC prompt).

5. Issue query 1: `{"sql": "SELECT raw_extensions FROM claroty.claroty_organization_acl_policies LIMIT 5"}`. Capture the full raw wire-level JSON response.

6. Issue query 2: `{"sql": "SELECT applied_models FROM claroty.claroty_organization_acl_policies LIMIT 1"}`. Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.022 | §Postconditions 5: applied_models serialized as JSON array in raw_extensions | Assertion 1+2: raw_extensions.applied_models is a JSON array, not a quoted string |
| BC-2.16.022 | §Postconditions 5: empty array serialized as [] not null (EC-016-022-005) | Assertion 2: even empty applied_models is [] not null |
| BC-2.16.022 | §Invariants: Tier-2 columns raise E-QUERY-038 on direct reference (EC-016-022-003) | Assertion 3: SELECT applied_models raises E-QUERY-038 |
| BC-2.16.022 | §Postconditions 2 Tier-2: policy_acl String column aggregated into raw_extensions | Assertion 4: policy_acl key present in raw_extensions as string value |

---

## Verification Approach

**Query 1 — raw_extensions inspection:**

1. Parse the wire-level JSON response from query 1.

2. If the response is an error object, record as FAIL: "raw_extensions query returned error."

3. For each returned row, locate the `raw_extensions` value. Assert it is a JSON object (not
   null, not a string).

4. Locate the `applied_models` key inside `raw_extensions`. Assert its value is either:
   - A JSON array (empty `[]` or non-empty `["model_a", ...]`) — record PASS on Json-type dim.
   - NOT a quoted string like `"[\"model_a\"]"` — if it IS a quoted string, record FAIL: "applied_models serialized as String, not Json."

5. Locate the `policy_acl` key inside `raw_extensions` (if any row has ACL text). Assert its
   value is a string (ACL text content). This confirms the String Tier-2 column is also
   aggregated alongside the Json column.

**Query 2 — Tier-2 plan-gate:**

6. Parse the wire-level JSON response from query 2.

7. Assert the response is an error with an error code indicating column-not-found (E-QUERY-038
   or equivalent). If the query SUCCEEDS and returns rows with `applied_models` as a standalone
   column, record FAIL on "Tier-2 plan-gate enforced" dimension — this means the column was
   accidentally promoted to Tier-1 or the plan-gate was not applied.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query 1 succeeds (raw_extensions accessible)** (weight: 0.20): Does query 1 return rows
  with a non-null `raw_extensions` JSON object?
  Full credit (1.0): non-error response with ≥1 row carrying `raw_extensions` JSON object.
  Zero credit (0.0): error response or `raw_extensions` absent/null.

- **applied_models is a JSON array (not quoted string)** (weight: 0.40): Is the
  `raw_extensions.applied_models` value a JSON array?
  Full credit (1.0): `applied_models` key present in `raw_extensions` with JSON array value.
  Partial credit (0.3): `applied_models` key present but value is a quoted string (wrong type).
  Zero credit (0.0): `applied_models` key absent from `raw_extensions` entirely.

- **Tier-2 plan-gate enforced (query 2 raises E-QUERY-038)** (weight: 0.30): Does
  `SELECT applied_models` raise an error?
  Full credit (1.0): query 2 returns E-QUERY-038 or equivalent column-not-found error.
  Zero credit (0.0): query 2 succeeds (Tier-2 plan-gate not applied).

- **policy_acl String column in raw_extensions** (weight: 0.10): Is `policy_acl` present as a
  string key in `raw_extensions`?
  Full credit (1.0): `policy_acl` key present with string value (may be null/empty for row).
  Zero credit (0.0): `policy_acl` key absent from `raw_extensions`.

---

## Edge Conditions

- **All ACL policies have empty applied_models (`[]`):** The Json-type assertion still passes
  for `[]` — `[]` is a valid JSON array. The key must be present even when empty
  (EC-016-022-005).

- **`raw_extensions.applied_models` key absent entirely:** If no ACL policy rows have any
  device model associations, the key may be absent. Record PARTIAL (0.5) on the Json-type
  dimension — absence of key vs presence of wrong-type value are distinct.

- **Live sensor returns no rows:** Record as SETUP-FAILURE.

- **Bearer token authentication failure (E-SENSOR-001 / 401):** Record as SETUP-FAILURE.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ACLPOLICY-001-002 (satisfaction: X.XX) — claroty_organization_acl_policies applied_models Json column gap; check column_type='json' declaration in TOML spec (BC-2.16.022 §Postconditions 5 + §Invariants) and Tier-2 plan-gate for applied_models (BC-2.16.022 §Invariants EC-016-022-003)"`

Do NOT disclose: whether applied_models was a quoted string vs absent, the exact row count
tested, or the specific assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/organization_acl_policies/ |
| corpus_size | LIMIT 5 for raw_extensions inspection; LIMIT 1 for plan-gate test |
| known_edge_cases | All rows have empty applied_models=[]: valid; applied_models key absent entirely: PARTIAL |
| false_positive_threshold | Zero: JSON array vs quoted string is an unambiguous serialization assertion |
| false_negative_threshold | Zero: if applied_models is a quoted string, column_type="json" was not applied |

**Known-good corpus:** monroe Claroty xDome with ACL policies that have applied_models populated
— expected: raw_extensions.applied_models is a JSON array value.

**Known-problematic corpus:** A TOML spec where `applied_models` was declared `column_type =
"string"` — expected: raw_extensions.applied_models is a quoted string like `"[...]"`. This is
the failure mode BC-2.16.022 §Invariants guards against.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-c-f2-spec-evolution-g6 | 2026-08-24 | product-owner | Initial authoring. HS-029 group for S-CLAROTY-ACLPOLICY-001. Json column type assertion: applied_models in raw_extensions must be a JSON array, not a quoted string. BC-2.16.022 §Postconditions 5 and §Invariants. Tier-2 plan-gate check: SELECT applied_models must raise E-QUERY-038. SINGLE-USE. |
