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
version: "1.1"
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
verification_properties: []
lifecycle_status: consumed
introduced: "S-CLAROTY-DEVVULNREL-001"
last_evaluated: "2026-08-31"
last_eval_satisfaction: 0.75
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-DEVVULNREL-001 (HS-026 group). Tests BC-2.16.017 §Postconditions 2 (Tier-2 aggregation into raw_extensions) — specifically that the composite join keys (vulnerability_name + device_uid) are both present in raw_extensions, enabling cross-table joins. Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-DEVVULNREL-001-002: claroty_device_vulnerability_relations join-key correctness: vulnerability_name via Tier-1 finding_info_title, device_uid in raw_extensions

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-DEVVULNREL-001 (HS-026 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.017 §Postconditions 2 Tier-1 (`vulnerability_name` → OCSF `finding_info.title` → Arrow field `finding_info_title`; exposed as top-level Tier-1 column, NOT in `raw_extensions` per ADR-058 §B2); BC-2.16.017 §Postconditions 2 Tier-2 (`device_uid` — no `ocsf_field`; aggregates into `raw_extensions`; accessible via `SELECT raw_extensions`); BC-2.16.017 §Postconditions 3 (composite PK rationale: `(vulnerability_name, device_uid)`)
**Gate:** Story-level holdout gate (HS-026) — SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the join-key correctness for `claroty_device_vulnerability_relations`.
The table's semantic value is cross-table join capability: each row links a specific device to a
specific vulnerability. The two join keys that enable downstream analytical queries are:

- `vulnerability_name` — links to `claroty_vulnerabilities.name` (BC-2.16.015 §PC3: `name` is the
  canonical PK of claroty_vulnerabilities). Under `ocsf_column_naming = true`, `vulnerability_name`
  is a Tier-1 REQUIRED column mapped to OCSF `finding_info.title` (Arrow field `finding_info_title`).
  Per ADR-058 §B2, Tier-1 columns are exposed ONLY as top-level Arrow fields and are NOT aggregated
  into `raw_extensions` — the tiers are mutually exclusive. The correct join-key access path for
  `vulnerability_name` is the Tier-1 column `finding_info_title`, not `raw_extensions`. (The prior
  v1.0 version of this scenario incorrectly asserted that a REQUIRED Tier-1 column "must also appear
  in raw_extensions under the Tier-2 aggregation path" — that claim contradicts ADR-058 §B2 and was
  corrected in v1.1.)
- `device_uid` — Tier-2 column (no `ocsf_field`); must appear in `raw_extensions` so it can be
  used for JOIN predicates against device tables.

The key behavioral assertions:

- `SELECT finding_info_title FROM claroty_device_vulnerability_relations LIMIT 1` must return
  `finding_info_title` as a top-level Arrow column with a non-null string value (e.g. a CVE id or
  advisory title) — confirming the Tier-1 OCSF column access path for `vulnerability_name` is
  correctly wired (BC-2.16.017 §Postconditions 2 Tier-1; ADR-058 §B2).
- `SELECT raw_extensions FROM claroty_device_vulnerability_relations LIMIT 1` must return a JSON
  object that contains `device_uid` as a key — confirming the Tier-2 aggregation path for the
  device-side join key. If `device_uid` is absent from `raw_extensions`, join-based MSSP queries
  against device tables cannot function correctly even though the table appears to load successfully.

This scenario also tests the negative case: `SELECT device_uid FROM claroty_device_vulnerability_relations`
MUST raise E-QUERY-038 (device_uid is Tier-2 — it is NOT a standalone Arrow column), while
`SELECT raw_extensions` succeeds and `SELECT finding_info_title` succeeds.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT finding_info_title FROM claroty_device_vulnerability_relations LIMIT 1` is issued via MCP `query` tool
**Then** the response is not an error
**And** `finding_info_title` is present at the row top level as a non-null string (e.g. 'CVE-2024-38213')
**When** `SELECT raw_extensions FROM claroty_device_vulnerability_relations LIMIT 1` is issued via MCP `query` tool
**Then** the response is not an error
**And** the raw_extensions JSON object in the row contains a key `device_uid` with a non-null value
**When** `SELECT device_uid FROM claroty_device_vulnerability_relations LIMIT 1` is issued via MCP `query` tool
**Then** the response is an E-QUERY-038 column-not-found error

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 —
   reference by config key only, never include credential value in output).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Wait for prism to be ready.

5. Issue Query A: `{"sql": "SELECT finding_info_title FROM claroty_device_vulnerability_relations LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

6. Issue Query B: `{"sql": "SELECT raw_extensions FROM claroty_device_vulnerability_relations LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

7. Issue Query C: `{"sql": "SELECT device_uid FROM claroty_device_vulnerability_relations LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.017 | §Postconditions 2 Tier-1: vulnerability_name → `finding_info_title` (OCSF `finding_info.title`); exposed as top-level Arrow column, NOT in raw_extensions (ADR-058 §B2) | Query A: `finding_info_title` top-level, non-null string |
| BC-2.16.017 | §Postconditions 2 Tier-2: device_uid — Tier-2, aggregates into raw_extensions; NOT exposed as standalone Arrow column | Query B: raw_extensions JSON contains device_uid; Query C: E-QUERY-038 on SELECT device_uid |
| BC-2.16.017 | §Postconditions 3: Composite PK (vulnerability_name, device_uid) — both join keys must be accessible | Core: join capability requires finding_info_title (Tier-1) and device_uid via raw_extensions (Tier-2) |
| BC-2.16.017 | §Invariants: queries against Tier-2 columns by raw TOML name raise E-QUERY-038 | Query C: confirms Tier-2 plan-gate enforcement |

---

## Verification Approach

**Query A: finding_info_title Tier-1 access:**

1. Parse the wire-level JSON response from Query A.
2. If the response is an error (contains error_code), record FAIL — finding_info_title Tier-1 column not returned.
3. Locate the `finding_info_title` value in the first row. Assert it is a non-null, non-empty string
   (e.g. a CVE identifier such as "CVE-2024-38213" or a vulnerability advisory title).
4. This confirms `vulnerability_name` is correctly wired as Tier-1 OCSF column `finding_info_title`
   per BC-2.16.017 §Postconditions 2 Tier-1 and ADR-058 §B2 (Tier-1 columns are top-level Arrow
   fields; they do NOT appear in raw_extensions).

**Query B: raw_extensions device_uid content:**

1. Parse the wire-level JSON response from Query B.
2. If the response is an error (contains error_code), record FAIL — raw_extensions not queryable.
3. Locate the `raw_extensions` value in the first row. Assert it is a JSON object (not null, not string).
4. Assert the JSON object contains a key `device_uid` with a non-null, non-empty string value.
5. Optionally verify: `vulnerability_cvss_v3_score` key is also present in raw_extensions
   (confirms Tier-2 aggregation works beyond just join keys).

**Query C: Tier-2 plan-gate rejection:**

1. Parse the wire-level JSON response from Query C.
2. Assert the response is an ERROR (not a data response). Specifically, confirm:
   - The error indicates a column-not-found condition (E-QUERY-038 or equivalent)
   - The error response mentions `device_uid` as the unknown column
   - The error response's `available_columns` list contains `raw_extensions` but does NOT contain `device_uid`
3. If Query C succeeds with data rows (device_uid returned as a standalone column), record FAIL —
   the Tier-2 plan-gate enforcement is broken.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **raw_extensions queryable (Query B succeeds)** (weight: 0.25): Does Query B return a non-error
  response with ≥1 row containing a raw_extensions JSON object?
  Full credit (1.0): non-error response, raw_extensions is a non-null JSON object.
  Zero credit (0.0): error response or raw_extensions null/absent.

- **vulnerability_name accessible as Tier-1 OCSF column finding_info_title** (weight: 0.25):
  Does `SELECT finding_info_title FROM claroty_device_vulnerability_relations LIMIT 1` return
  `finding_info_title` as a top-level column with a non-null string value (e.g. a CVE id or
  advisory title)?
  Rationale: `vulnerability_name` is mapped to OCSF `finding_info.title` (Arrow field
  `finding_info_title`) per BC-2.16.017 §Postconditions 2 Tier-1. Per ADR-058 §B2, Tier-1 columns
  are exposed ONLY as top-level Arrow fields, never in `raw_extensions` (tiers are mutually
  exclusive). The over-specified prior version (v1.0) incorrectly asserted that a REQUIRED Tier-1
  column "must also appear in raw_extensions" — corrected in v1.1.
  Full credit (1.0): `finding_info_title` present at row top level, non-null non-empty string.
  Partial credit (0.5): `finding_info_title` present but null or empty.
  Zero credit (0.0): `finding_info_title` absent from response OR response is an error.

- **device_uid in raw_extensions** (weight: 0.25): Does raw_extensions contain a `device_uid` key
  with a non-null string value?
  Full credit (1.0): key present, value non-null, non-empty string.
  Partial credit (0.5): key present but null (device_uid was null in source).
  Zero credit (0.0): key absent from raw_extensions.

- **Tier-2 plan-gate enforced (Query C fails with E-QUERY-038)** (weight: 0.25): Does Query C
  return a column-not-found error for `device_uid`?
  Full credit (1.0): error response, column-not-found for device_uid, raw_extensions in available_columns.
  Partial credit (0.5): error response but available_columns lacks raw_extensions.
  Zero credit (0.0): Query C succeeds (Tier-2 gate not enforced).

---

## Edge Conditions

- **Raw extensions object does not contain device_uid (null device from API):** Record PARTIAL
  (0.5) on device_uid dimension — the column was included in the fields projection, but the API
  returned null for this row. This is distinct from the column being absent.

- **Live sensor returns empty result set (zero rows on Query A or Query B):** Record as SETUP-FAILURE
  for row-content dimensions. Score the respective "succeeds" dimension 1.0 (empty is not an error)
  and the content dimensions as unscored.

- **Query C returns a different error (e.g., syntax error):** Record FAIL on the Tier-2 gate
  dimension — the error must be specifically a column-not-found, not an unrelated query error.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DEVVULNREL-001-002 (satisfaction: X.XX) — claroty_device_vulnerability_relations join-key gap; check Tier-1 OCSF column finding_info_title for vulnerability_name access (BC-2.16.017 §Postconditions 2 Tier-1; ADR-058 §B2), raw_extensions Tier-2 aggregation for device_uid (BC-2.16.017 §Postconditions 2 Tier-2), and Tier-2 plan-gate rejection for device_uid (BC-2.16.017 §Invariants)"`

Do NOT disclose: the specific SQL queries used, the exact keys checked, or the assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/device_vulnerability_relations/ |
| corpus_size | LIMIT 1 (single row sufficient for structural join-key assertion) |
| known_edge_cases | device_uid null for a specific row (partial credit, not full fail); empty result set on live sensor (SETUP-FAILURE) |
| false_positive_threshold | Zero: asserting top-level column presence and raw_extensions key presence is structural, not content-dependent |
| false_negative_threshold | Zero: if finding_info_title absent, vulnerability join-key is broken; if device_uid absent from raw_extensions, device join-key is broken |

**Known-good corpus:** monroe Claroty xDome with ≥1 device-vulnerability relation — expected:
`finding_info_title` is accessible as a top-level Tier-1 column; raw_extensions JSON contains `device_uid`; SELECT device_uid raises E-QUERY-038.

**Known-problematic corpus:** A table configuration where `device_uid` was accidentally given an
`ocsf_field` mapping, promoting it to Tier-1 and removing it from raw_extensions — expected:
SELECT device_uid succeeds (wrong) instead of E-QUERY-038. This is BC-2.16.017 §Invariants' primary
guard against accidental Tier promotion.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | xdome-wave-b-adjudication | 2026-08-31 | product-owner | Over-specification correction (verdict A). Dimension 2 corrected: `vulnerability_name` is Tier-1 (OCSF `finding_info.title` → Arrow field `finding_info_title`) per BC-2.16.017 §Postconditions 2 Tier-1 and ADR-058 §B2; Tier-1 columns are NOT in `raw_extensions` (tiers mutually exclusive). Prior v1.0 dimension "vulnerability_name in raw_extensions" was wrong and replaced with "vulnerability_name accessible as Tier-1 OCSF column finding_info_title". BDD: new Query A tests `SELECT finding_info_title FROM claroty_device_vulnerability_relations LIMIT 1`; And-assertion replaced with "finding_info_title is present at the row top level as a non-null string (e.g. 'CVE-2024-38213')". Query B tests raw_extensions for device_uid (Tier-2, semantics unchanged, renumbered from old Query A). Query C tests E-QUERY-038 plan-gate for device_uid (semantics unchanged, renumbered from old Query B). All SQL uses bare table name — no schema prefix. |
| 1.0 | xdome-wave-b-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-026 group for S-CLAROTY-DEVVULNREL-001. Join-key correctness: raw_extensions contains vulnerability_name + device_uid (BC-2.16.017 §PC2 Tier-2 aggregation + §PC3 composite PK); Tier-2 plan-gate: SELECT device_uid raises E-QUERY-038 (BC-2.16.017 §Invariants). SINGLE-USE, live monroe only (no DTU per D-2200). |
