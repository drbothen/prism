---
document_type: holdout-scenario
level: L3
id: "HS-DEVVULNREL-001-002"
title: "claroty_device_vulnerability_relations join-key correctness: vulnerability_name and device_uid both in raw_extensions"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-B"
story_source: "S-CLAROTY-DEVVULNREL-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-24"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.017-claroty-device-vulnerability-relations-table.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "fdccb7c"
traces_to: "BC-2.16.017"
behavioral_contracts:
  - BC-2.16.017
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-DEVVULNREL-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-DEVVULNREL-001 (HS-026 group). Tests BC-2.16.017 §Postconditions 2 (Tier-2 aggregation into raw_extensions) — specifically that the composite join keys (vulnerability_name + device_uid) are both present in raw_extensions, enabling cross-table joins. Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-DEVVULNREL-001-002: claroty_device_vulnerability_relations join-key correctness: vulnerability_name and device_uid both in raw_extensions

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-DEVVULNREL-001 (HS-026 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.017 §Postconditions 2 Tier-2 (composite join keys — `vulnerability_name` and `device_uid` aggregate into `raw_extensions`; accessible via `SELECT raw_extensions`); BC-2.16.017 §Postconditions 3 (composite PK rationale: `(vulnerability_name, device_uid)`)
**Gate:** Story-level holdout gate (HS-026) — SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the Tier-2 join-key correctness for `claroty_device_vulnerability_relations`.
The table's semantic value is cross-table join capability: each row links a specific device to a
specific vulnerability. The two join keys that enable downstream analytical queries are:

- `vulnerability_name` — links to `claroty_vulnerabilities.name` (BC-2.16.015 §PC3: `name` is the
  canonical PK of claroty_vulnerabilities). Under `ocsf_column_naming = true`, `vulnerability_name`
  is also the Tier-1 REQUIRED column mapped to `finding_info_title`. Since it is REQUIRED and
  Tier-1, it must also appear in `raw_extensions` under the Tier-2 aggregation path when querying
  `raw_extensions` directly.
- `device_uid` — Tier-2 column (no `ocsf_field`); must appear in `raw_extensions` so it can be
  used for JOIN predicates against device tables.

The key behavioral assertion: `SELECT raw_extensions FROM claroty.claroty_device_vulnerability_relations`
must return a JSON object that contains BOTH `vulnerability_name` and `device_uid` as keys in the
same row. If either key is absent from `raw_extensions`, join-based MSSP queries cannot function
correctly even though the table appears to load successfully.

This scenario also tests the negative case: `SELECT device_uid FROM claroty.claroty_device_vulnerability_relations`
MUST raise E-QUERY-038 (device_uid is Tier-2 — it is NOT a standalone Arrow column), while
`SELECT raw_extensions` succeeds.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT raw_extensions FROM claroty.claroty_device_vulnerability_relations LIMIT 1` is issued via MCP `query` tool
**Then** the response is not an error
**And** the raw_extensions JSON object in the row contains a key `vulnerability_name` with a non-null value
**And** the raw_extensions JSON object in the row contains a key `device_uid` with a non-null value
**When** `SELECT device_uid FROM claroty.claroty_device_vulnerability_relations LIMIT 1` is issued via MCP `query` tool
**Then** the response is an E-QUERY-038 column-not-found error

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 —
   reference by config key only, never include credential value in output).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Wait for prism to be ready.

5. Issue Query A: `{"sql": "SELECT raw_extensions FROM claroty.claroty_device_vulnerability_relations LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

6. Issue Query B: `{"sql": "SELECT device_uid FROM claroty.claroty_device_vulnerability_relations LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.017 | §Postconditions 2 Tier-2: device_uid — Tier-2, aggregates into raw_extensions; NOT exposed as standalone Arrow column | Query B: E-QUERY-038 on SELECT device_uid |
| BC-2.16.017 | §Postconditions 2 Tier-2: vulnerability_name, device_uid in raw_extensions | Query A: raw_extensions JSON contains both keys |
| BC-2.16.017 | §Postconditions 3: Composite PK (vulnerability_name, device_uid) — both join keys must be accessible | Core: join capability requires both keys accessible via raw_extensions |
| BC-2.16.017 | §Invariants: queries against Tier-2 columns by raw TOML name raise E-QUERY-038 | Query B: confirms Tier-2 plan-gate enforcement |

---

## Verification Approach

**Query A: raw_extensions join-key content:**

1. Parse the wire-level JSON response from Query A.
2. If the response is an error (contains error_code), record FAIL — raw_extensions not queryable.
3. Locate the `raw_extensions` value in the first row. Assert it is a JSON object (not null, not string).
4. Assert the JSON object contains a key `vulnerability_name` with a non-null, non-empty string value.
5. Assert the JSON object contains a key `device_uid` with a non-null, non-empty string value.
6. Optionally verify: `vulnerability_cvss_v3_score` key is also present in raw_extensions
   (confirms Tier-2 aggregation works beyond just join keys).

**Query B: Tier-2 plan-gate rejection:**

1. Parse the wire-level JSON response from Query B.
2. Assert the response is an ERROR (not a data response). Specifically, confirm:
   - The error indicates a column-not-found condition (E-QUERY-038 or equivalent)
   - The error response mentions `device_uid` as the unknown column
   - The error response's `available_columns` list contains `raw_extensions` but does NOT contain `device_uid`
3. If Query B succeeds with data rows (device_uid returned as a standalone column), record FAIL —
   the Tier-2 plan-gate enforcement is broken.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **raw_extensions queryable (Query A succeeds)** (weight: 0.25): Does Query A return a non-error
  response with ≥1 row containing a raw_extensions JSON object?
  Full credit (1.0): non-error response, raw_extensions is a non-null JSON object.
  Zero credit (0.0): error response or raw_extensions null/absent.

- **vulnerability_name in raw_extensions** (weight: 0.25): Does raw_extensions contain a
  `vulnerability_name` key with a non-null string value?
  Full credit (1.0): key present, value non-null, non-empty string.
  Partial credit (0.5): key present but null or empty.
  Zero credit (0.0): key absent.

- **device_uid in raw_extensions** (weight: 0.25): Does raw_extensions contain a `device_uid` key
  with a non-null string value?
  Full credit (1.0): key present, value non-null, non-empty string.
  Partial credit (0.5): key present but null (device_uid was null in source).
  Zero credit (0.0): key absent from raw_extensions.

- **Tier-2 plan-gate enforced (Query B fails with E-QUERY-038)** (weight: 0.25): Does Query B
  return a column-not-found error for `device_uid`?
  Full credit (1.0): error response, column-not-found for device_uid, raw_extensions in available_columns.
  Partial credit (0.5): error response but available_columns lacks raw_extensions.
  Zero credit (0.0): Query B succeeds (Tier-2 gate not enforced).

---

## Edge Conditions

- **Raw extensions object does not contain device_uid (null device from API):** Record PARTIAL
  (0.5) on device_uid dimension — the column was included in the fields projection, but the API
  returned null for this row. This is distinct from the column being absent.

- **Live sensor returns empty result set (zero rows on Query A):** Record as SETUP-FAILURE for
  row-content dimensions. Score Query A "succeeds" dimension 1.0 (empty is not an error) and
  the join-key dimensions as unscored.

- **Query B returns a different error (e.g., syntax error):** Record FAIL on the Tier-2 gate
  dimension — the error must be specifically a column-not-found, not an unrelated query error.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DEVVULNREL-001-002 (satisfaction: X.XX) — claroty_device_vulnerability_relations join-key gap; check raw_extensions Tier-2 aggregation for vulnerability_name + device_uid (BC-2.16.017 §Postconditions 2) and Tier-2 plan-gate rejection for device_uid (BC-2.16.017 §Invariants)"`

Do NOT disclose: the specific SQL queries used, the exact keys checked, or the assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/device_vulnerability_relations/ |
| corpus_size | LIMIT 1 (single row sufficient for structural join-key assertion) |
| known_edge_cases | device_uid null for a specific row (partial credit, not full fail); empty result set on live sensor (SETUP-FAILURE) |
| false_positive_threshold | Zero: asserting key presence in JSON is structural, not content-dependent |
| false_negative_threshold | Zero: if device_uid is absent from raw_extensions, join capability is broken |

**Known-good corpus:** monroe Claroty xDome with ≥1 device-vulnerability relation — expected:
raw_extensions JSON contains both vulnerability_name and device_uid; SELECT device_uid raises E-QUERY-038.

**Known-problematic corpus:** A table configuration where `device_uid` was accidentally given an
`ocsf_field` mapping, promoting it to Tier-1 and removing it from raw_extensions — expected:
SELECT device_uid succeeds (wrong) instead of E-QUERY-038. This is BC-2.16.017 §Invariants' primary
guard against accidental Tier promotion.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-b-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-026 group for S-CLAROTY-DEVVULNREL-001. Join-key correctness: raw_extensions contains vulnerability_name + device_uid (BC-2.16.017 §PC2 Tier-2 aggregation + §PC3 composite PK); Tier-2 plan-gate: SELECT device_uid raises E-QUERY-038 (BC-2.16.017 §Invariants). SINGLE-USE, live monroe only (no DTU per D-2200). |
