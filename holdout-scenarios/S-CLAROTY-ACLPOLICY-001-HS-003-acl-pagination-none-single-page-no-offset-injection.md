---
document_type: holdout-scenario
level: L3
id: "HS-ACLPOLICY-001-003"
title: "claroty_organization_acl_policies: non-paginated single-page fetch returns full result set with no offset/limit injection; SELECT policy_id raises E-QUERY-038 while SELECT metadata_uid succeeds"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-ACLPOLICY-001 (HS-029 group). Tests the KEY NOVELTY of BC-2.16.022: PaginationConfig::None (type='none') — the non-paginated single-page fetch contract unique among all Claroty tables. Verifies: (1) full result set returned from a single request (no looping); (2) no count column in wire output; (3) Tier-1 rename enforced (SELECT policy_id → E-QUERY-038; SELECT metadata_uid → success). Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ACLPOLICY-001-003: claroty_organization_acl_policies: non-paginated single-page fetch returns full result set with no offset/limit injection; SELECT policy_id raises E-QUERY-038 while SELECT metadata_uid succeeds

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-ACLPOLICY-001 (HS-029 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.022 §Postconditions 4 (Pagination-None contract — no offset/limit
injection; single-page fetch; no count field in response; full result set returned) and
§Postconditions 2 Tier-1 (policy_id is the TOML column name; Arrow field name is metadata_uid
→ SELECT policy_id must raise E-QUERY-038; SELECT metadata_uid must succeed) and
§Invariants (PaginationConfig::None MUST NOT inject offset/limit)
**Gate:** Story-level holdout gate (HS-029) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the KEY NOVELTY of this table: `claroty_organization_acl_policies` uses
`PaginationConfig::None` — a non-paginated single-page fetch. This is the only Claroty table
with this behavior. All other Claroty tables (alerts, devices, audit_logs, device_alert_relations,
vulnerabilities, ot_activity_events, device_vulnerability_relations, servers, server_interfaces,
organization_zones, zone_policies, firewall_groups, firewall_policies) use `offset_limit/1000`.

The scenario also tests the `policy_id → metadata_uid` Tier-1 rename as a secondary confirmation.

**Part A — Non-paginated fetch (the key novelty):**

1. `SELECT * FROM claroty.claroty_organization_acl_policies` (no LIMIT clause) should succeed
   and return all ACL policies without requiring multiple page fetches. The wire output should
   NOT contain a `count` column (count is not in the response envelope per the API schema).

2. The query should complete in a single HTTP round-trip to
   POST /api/v1/organization_acl_policies/ — not a loop of paginated requests. The evaluator
   can infer this from the absence of a `count` column and the successful completion of the
   unbounded query.

3. If the implementation mistakenly injected `offset`/`limit` into the POST body (i.e., treated
   this as `offset_limit`), the Claroty API would return a 422 validation error (schema does not
   accept those fields). The evaluator observes whether the query errors with E-SENSOR-001
   indicating a 422 API error — this IS a behavioral FAIL for this scenario.

**Part B — Tier-1 rename enforcement (policy_id → metadata_uid):**

4. `SELECT policy_id FROM claroty.claroty_organization_acl_policies LIMIT 1` should raise
   E-QUERY-038. `policy_id` is the TOML column name; its Arrow field name is `metadata_uid`
   (from `ocsf_field = "metadata.uid"` → `ocsf_field_to_arrow_name` → `metadata_uid`). The
   raw TOML name must not be accessible as a standalone Arrow column.

5. `SELECT metadata_uid FROM claroty.claroty_organization_acl_policies LIMIT 1` should succeed
   and return non-null UUID string values.

**BDD supplement (Part A):**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT * FROM claroty.claroty_organization_acl_policies` (unbounded) is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response wire JSON does NOT contain a top-level column named `count`
**And** the response returns all available ACL policies (the complete result set in one response)

**BDD supplement (Part B):**

**When** `SELECT policy_id FROM claroty.claroty_organization_acl_policies LIMIT 1` is issued
**Then** the response is an error with error code `E-QUERY-038`
**And** the `available_columns` in the error response contains `metadata_uid`

**When** `SELECT metadata_uid FROM claroty.claroty_organization_acl_policies LIMIT 1` is issued
**Then** the response is not an error
**And** at least one returned row has a non-null `metadata_uid` UUID string value

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor. Do NOT
   include the credential value in any output — reference the credential by the configuration
   key only (AD-017).

3. Start prism in MCP stdio mode with the claroty sensor spec included. Capture all MCP stdio
   output and stderr, including any HTTP request logs.

4. Wait for prism to be ready (startup completion log message or first JSON-RPC prompt).

5. Issue query A (unbounded): `{"sql": "SELECT * FROM claroty.claroty_organization_acl_policies"}`. Capture the full raw wire-level JSON response.

6. Issue query B (plan-gate fail): `{"sql": "SELECT policy_id FROM claroty.claroty_organization_acl_policies LIMIT 1"}`. Capture the full raw wire-level JSON response.

7. Issue query C (plan-gate pass): `{"sql": "SELECT metadata_uid FROM claroty.claroty_organization_acl_policies LIMIT 1"}`. Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.022 | §Postconditions 4: PaginationConfig::None — no offset/limit injection; single-page fetch | Query A: succeeds without API error (no 422 from offset/limit injection); no count column in wire output |
| BC-2.16.022 | §Postconditions 1: response_path = $.organization_acl_policies, NO count field | Query A: count column absent from wire output |
| BC-2.16.022 | §Invariants: PaginationConfig::None MUST NOT inject offset/limit (EC-016-022-007) | Query A: success vs E-SENSOR-001 422 distinguishes correct vs incorrect pagination config |
| BC-2.16.022 | §Postconditions 2 Tier-1: policy_id TOML name NOT standalone Arrow column (EC-016-022-004) | Query B: E-QUERY-038 with metadata_uid in available_columns |
| BC-2.16.022 | §Postconditions 2 Tier-1: metadata_uid Arrow field name REQUIRED, accessible | Query C: succeeds; metadata_uid column non-null UUID string |

---

## Verification Approach

**Query A — non-paginated single-page fetch:**

1. Parse the wire-level JSON response from query A.

2. If the response is an error object containing `E-SENSOR-001` with a message indicating a 422
   or 400 API error (likely from offset/limit injection): record FAIL on "no offset/limit
   injection" dimension. This is the failure mode BC-2.16.022 §Invariants EC-016-022-007
   describes.

3. If the response is a non-error with rows, inspect the column list of any returned row.
   Assert that no column named `count` appears as a standalone Arrow field. The absence of
   `count` confirms the response envelope had no count field (as expected for this non-paginated
   endpoint). Record PASS on "no count column" dimension.

4. Observe the response completion: a non-paginated fetch completes in one response. A timeout
   or hanging query could indicate the implementation is waiting for pages that never arrive
   (wrong pagination config). Record as SETUP-FAILURE if timeout occurs.

**Query B — Tier-1 plan-gate (policy_id rejected):**

5. Parse the wire-level JSON response from query B.

6. Assert the response is an error indicating column-not-found (E-QUERY-038 or equivalent).
   If the query SUCCEEDS with rows containing a standalone `policy_id` column, record FAIL:
   "Tier-1 rename not applied — policy_id accessible as standalone column."

7. If E-QUERY-038 is returned, inspect `available_columns` for the presence of `metadata_uid`.
   Assert `metadata_uid` appears in `available_columns`. Record PASS on "available_columns
   correct" dimension.

**Query C — Tier-1 plan-gate (metadata_uid accepted):**

8. Parse the wire-level JSON response from query C.

9. Assert the response is NOT an error. If query C returns E-QUERY-038, record FAIL:
   "metadata_uid Arrow field name not accessible — ocsf_field_to_arrow_name transform not
   applied for metadata.uid."

10. Assert at least one returned row has a non-null `metadata_uid` value that is a UUID-format
    string. Record PASS on "metadata_uid accessible and non-null" dimension.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Non-paginated fetch succeeds without 422 error** (weight: 0.35): Does query A complete
  without an API validation error (i.e., no offset/limit injected)?
  Full credit (1.0): non-error response with ≥1 row.
  Zero credit (0.0): E-SENSOR-001 with 422 API error (offset/limit injected); OR query A times
  out.

- **No count column in wire output** (weight: 0.15): Does the unbounded query response lack a
  standalone `count` Arrow column?
  Full credit (1.0): `count` absent as a standalone column in wire output.
  Zero credit (0.0): `count` present as a standalone Arrow column (pagination response format
  used for non-paginated endpoint).

- **policy_id raises E-QUERY-038 (Tier-1 rename enforced)** (weight: 0.25): Does query B
  raise an error, AND does available_columns contain metadata_uid?
  Full credit (1.0): E-QUERY-038 raised; `metadata_uid` in available_columns.
  Partial credit (0.5): E-QUERY-038 raised but available_columns does not contain metadata_uid.
  Zero credit (0.0): query B succeeds (policy_id accessible as standalone column).

- **metadata_uid accessible (Tier-1 Arrow field accepted)** (weight: 0.25): Does query C
  succeed with non-null UUID values?
  Full credit (1.0): non-error response; metadata_uid column non-null UUID string in ≥1 row.
  Partial credit (0.5): non-error response; metadata_uid column present but null in all rows.
  Zero credit (0.0): query C raises E-QUERY-038 (metadata_uid not accessible).

---

## Edge Conditions

- **Monroe xDome has zero ACL policies:** Query A returns empty result set. This is SETUP-FAILURE
  for Part A (cannot assert no-count from empty result). For Part B/C, queries still exercise
  the plan-gate and column-resolution path — record PASS/FAIL on B/C independently.

- **Authentication failure (E-SENSOR-001 / 401):** Record as SETUP-FAILURE for all parts.

- **Query A returns very large result set (>1000 rows):** This is EXPECTED and a PASS signal —
  it confirms the non-paginated fetch returned more rows than the 1000-row page size used by
  paginated endpoints. Note the row count in the evaluation report.

- **`count` appears as a key INSIDE `raw_extensions` (not as a standalone Arrow column):**
  This is acceptable — `count` as a top-level key in the OCSF envelope is what we're checking,
  not nested JSON content.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ACLPOLICY-001-003 (satisfaction: X.XX) — claroty_organization_acl_policies pagination/rename gap; check PaginationConfig::None in TOML [tables.steps.pagination] section and policy_id→metadata.uid→metadata_uid Tier-1 column Arrow name (BC-2.16.022 §Postconditions 4 + §Invariants EC-016-022-007 and §Postconditions 2 EC-016-022-004)"`

Do NOT disclose: the specific API error code from a 422, the row count returned, or the exact
assertion thresholds.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/organization_acl_policies/ |
| corpus_size | Unbounded query for Part A; LIMIT 1 for Part B and C |
| known_edge_cases | Zero ACL policies in xDome → SETUP-FAILURE for Part A; very large result set → expected PASS; 422 from offset/limit injection → FAIL |
| false_positive_threshold | Zero: 422 API error from offset/limit injection is an unambiguous structural failure |
| false_negative_threshold | Zero: E-QUERY-038 on policy_id and success on metadata_uid are unambiguous column-resolution assertions |

**Known-good corpus:** monroe Claroty xDome with ≥1 ACL policy and `type = "none"` correctly
declared — expected: query A succeeds without 422, no count column, query B raises E-QUERY-038,
query C succeeds with metadata_uid values.

**Known-problematic corpus:** A TOML spec where `pagination.type = "offset_limit"` was used
for this endpoint — expected: 422 API validation error from injected offset/limit fields. This
is the exact failure mode BC-2.16.022 §Invariants EC-016-022-007 documents.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-c-f2-spec-evolution-g6 | 2026-08-24 | product-owner | Initial authoring. HS-029 group for S-CLAROTY-ACLPOLICY-001. Tests the KEY NOVELTY: PaginationConfig::None (type='none') non-paginated single-page fetch. Part A: unbounded query succeeds without 422 (no offset/limit injection); no count column in wire output (confirms non-paginated envelope). Part B: SELECT policy_id raises E-QUERY-038 with metadata_uid in available_columns. Part C: SELECT metadata_uid succeeds with UUID values. BC-2.16.022 §Postconditions 4 (pagination-none contract) + §Postconditions 2 (metadata_uid Tier-1 rename) + §Invariants (EC-016-022-004/007). SINGLE-USE. |
