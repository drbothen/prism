---
document_type: holdout-scenario
level: L3
id: "HS-ORGPOL-001-003"
title: "claroty_organization_firewall_groups SELECT * wire shape: class_uid=3004, name Tier-1 REQUIRED present from firewall_group_name→entity_management name, URL vs envelope key asymmetry verified"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-C"
story_source: "S-CLAROTY-ORGPOLICY-001"
version: "1.1"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-ORGPOLICY-001 (HS-028 group). Tests BC-2.16.021 §Postconditions 1 (TOML table contract — ocsf_class = 'entity_management' → class_uid 3004; URL /api/v1/organization_fw_groups/ with envelope $.organization_firewall_groups — asymmetry verified) and §Postconditions 3 Tier-1 (firewall_group_name → name REQUIRED; raw_extensions present). Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ORGPOL-001-003: claroty_organization_firewall_groups SELECT * wire shape: class_uid=3004, name Tier-1 REQUIRED present from firewall_group_name→entity_management name, URL vs envelope key asymmetry verified

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-ORGPOLICY-001 (HS-028 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.021 §Postconditions 1 (TOML table contract — ocsf_class = "entity_management"
→ class_uid 3004; URL `/api/v1/organization_fw_groups/` with envelope `$.organization_firewall_groups`
— the URL vs envelope key asymmetry that causes silent data loss if wrong) and §Postconditions 3
Tier-1 (`firewall_group_name → name` REQUIRED; `raw_extensions` present with Tier-2 keys)
**Gate:** Story-level holdout gate (HS-028) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the `claroty_organization_firewall_groups` table is registered and
returns data correctly from the live sensor — specifically exercising the critical URL vs
envelope key asymmetry documented in BC-2.16.021 §PC1:

1. The query returns a non-empty result set (at least one row). An empty result set indicates
   either no firewall groups in the xDome instance (SETUP-FAILURE) OR — the more dangerous case
   — the `response_path` was set to `$.organization_fw_groups` (abbreviated) instead of
   `$.organization_firewall_groups` (full spelling), which produces silent data loss. The evaluator
   MUST distinguish these two causes.

2. The returned JSON rows carry `class_uid = 3004` — the integer class_uid for `entity_management`.

3. The returned JSON rows carry a column named `name` — the Arrow field name for the Tier-1
   mapping of `firewall_group_name` (source: `ocsf_field = "name"` → Arrow `name`, REQUIRED).
   The rows do NOT carry a standalone column named `firewall_group_name`.

4. The returned rows carry a `raw_extensions` column containing a non-null JSON-serialized STRING
   (Arrow Utf8 per ADR-058 §I2) that, when parsed via `serde_json::from_str`, contains at least
   one firewall-group-specific Tier-2 key (e.g., `firewall_group_source`, `priority`,
   `device_conditions`, `attributed_devices`, or `last_update`). The wire-level value is a string
   (`"raw_extensions":"{...}"`), NOT a native JSON object; a native object is structurally wrong
   per ADR-058 §I2 (D-2381 native-JSON rule applies only to column_type="json" values INSIDE
   raw_extensions, not the container).

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT * FROM claroty.claroty_organization_firewall_groups LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response contains at least one row (non-empty result set)
**And** the response wire JSON contains a row with a column `class_uid` equal to `3004`
**And** the response wire JSON contains a row with a column `name` that is a non-null, non-empty string
**And** the response wire JSON does NOT contain a top-level column named `firewall_group_name`
**And** the response wire JSON contains a row with a column `raw_extensions` that is a non-null JSON-serialized string (Arrow Utf8 per ADR-058 §I2) whose parsed content contains at least one firewall-group Tier-2 key

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017).

3. Start prism in MCP stdio mode. Wait for ready.

4. Issue the MCP `query` tool call:
   `{"sql": "SELECT * FROM claroty.claroty_organization_firewall_groups LIMIT 1"}`.

5. Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.021 | §Postconditions 1: ocsf_class = "entity_management" → class_uid 3004 | Assertion: class_uid = 3004 in wire output |
| BC-2.16.021 | §Postconditions 1: URL /api/v1/organization_fw_groups/ with response_path $.organization_firewall_groups (URL vs envelope key asymmetry) | Assertion: non-empty result set (empty = possible response_path bug or setup failure) |
| BC-2.16.021 | §Postconditions 3 Tier-1: firewall_group_name → ocsf_field = "name" → Arrow field name REQUIRED | Assertion: name column present and non-null; firewall_group_name absent as standalone |
| BC-2.16.021 | §Postconditions 3 Tier-2: 7 Tier-2 columns aggregate into raw_extensions | Assertion: raw_extensions present as serialized JSON string (Arrow Utf8, ADR-058 §I2) whose parsed object contains firewall-group Tier-2 keys |

---

## Verification Approach

1. Parse the wire-level JSON response.

2. If the response is an error, record FAIL on "query succeeds" dimension.

3. If the response is non-error but returns zero rows: evaluate two causes:
   - Check if `claroty_organization_firewall_groups` table exists (test with
     `SELECT name FROM claroty.claroty_organization_firewall_groups LIMIT 1` — if it gives
     "table not found" that's a FAIL; if it gives E-QUERY-038 on `name`, there's a TOML bug).
   - If the table exists but returns zero rows on a live monroe instance that normally has
     firewall groups, suspect the `response_path` used `$.organization_fw_groups` (abbreviated)
     — this produces zero rows with no error. Record as SUSPICIOUS-FAIL on
     "non-empty result set" dimension with note "possible response_path asymmetry bug."
   - If monroe genuinely has no firewall groups, record as SETUP-FAILURE.

4. For a row with at least one result: assert `class_uid = 3004`. Fail on wrong value.

5. Inspect the row's column list. Assert `name` is present and non-null. Assert `firewall_group_name`
   is NOT a standalone column. Record appropriately.

6. Inspect `raw_extensions`. Per ADR-058 §I2, raw_extensions is an Arrow Utf8 column; the wire
   emits it as a JSON-serialized STRING (`"raw_extensions":"{...}"`), NOT a native JSON object.
   Assert: (a) the value is a non-null string; (b) parsing the string via `serde_json::from_str`
   succeeds; (c) the parsed object contains at least one Tier-2 key from (`firewall_group_source`,
   `priority`, `device_conditions`, `attributed_devices`, `exportable_attributed_devices`,
   `created_time`, `last_update`).
   If `raw_extensions` is present as a native JSON object (not a string), record PARTIAL (0.5) —
   data is present but the encoding is structurally wrong per ADR-058 §I2; D-2381 native-JSON rule
   applies only to column_type="json" values INSIDE raw_extensions, not the container itself.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query succeeds and non-empty** (weight: 0.25): Returns a non-error response with ≥1 row.
  Full credit (1.0): non-error, ≥1 row.
  Partial credit (0.3): zero rows but table exists and no error (possible response_path bug — SUSPICIOUS).
  Zero credit (0.0): error response or table not found.

- **class_uid = 3004** (weight: 0.35): class_uid column present with integer value 3004.
  Full credit (1.0): present, value 3004.
  Partial credit (0.3): present but wrong value.
  Zero credit (0.0): absent.

- **name Tier-1 REQUIRED present** (weight: 0.25): `name` column present non-null;
  `firewall_group_name` NOT a standalone column.
  Full credit (1.0): name present non-null; firewall_group_name absent.
  Partial credit (0.5): name present but null.
  Zero credit (0.0): name absent OR firewall_group_name appears as standalone.

- **raw_extensions with Tier-2 keys** (weight: 0.15): raw_extensions present as a non-null
  JSON-serialized string (Arrow Utf8, ADR-058 §I2) whose parsed object contains at least one
  firewall-group Tier-2 key.
  Full credit (1.0): present as non-null string; `serde_json::from_str` succeeds; parsed object
  contains ≥1 Tier-2 key from the firewall-group column set.
  Partial credit (0.5): present as a native JSON object (structurally wrong per ADR-058 §I2 —
  D-2406 adjudication; data is accessible but wire encoding is wrong); OR present as a string
  that parses to an empty object.
  Zero credit (0.0): absent or null.

---

## Edge Conditions

- **Zero firewall groups on live sensor:** Record as SETUP-FAILURE. Firewall domain likely
  not used on this xDome instance.

- **`response_path = "$.organization_fw_groups"` (abbreviated bug):** Produces zero rows with
  no error — the silent data-loss defect BC-2.16.021 §Invariants guards against. Record as
  SUSPICIOUS-FAIL.

- **`firewall_group_name` as standalone column:** Means the Tier-1 rename was not applied.
  The `ocsf_field = "name"` mapping produces Arrow column `name`; if the implementer omitted
  `ocsf_field` in the TOML, the column stays as `firewall_group_name` and is Tier-2.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ORGPOL-001-003 (satisfaction: X.XX) — claroty_organization_firewall_groups wire-shape gap; check URL vs envelope key asymmetry (path /api/v1/organization_fw_groups/ vs $.organization_firewall_groups — BC-2.16.021 §Postconditions 1), class_uid=3004, and firewall_group_name→name Tier-1 REQUIRED rename (BC-2.16.021 §Postconditions 3)"`

Do NOT disclose: the specific column values expected, the LIMIT value, zero-row diagnostic steps,
or the response_path correction.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/organization_fw_groups/ |
| corpus_size | LIMIT 1 (single row sufficient for structural assertion) |
| known_edge_cases | Zero rows (either no firewall groups or response_path asymmetry bug); firewall_group_name as standalone (Tier-1 rename missing) |
| false_positive_threshold | Zero: class_uid=3004 and name Tier-1 REQUIRED are structural assertions |
| false_negative_threshold | Zero: zero rows from wrong response_path is caught by SUSPICIOUS-FAIL classification |

**Known-good corpus:** monroe with ≥1 firewall group and correct response_path — expected:
non-empty result, class_uid=3004, name non-null string, raw_extensions as non-null serialized JSON
string (Arrow Utf8) with firewall-group Tier-2 keys in the parsed object.

**Known-problematic corpus:** A claroty.sensor.toml where `response_path = "$.organization_fw_groups"`
(abbreviated) is used — expected: zero rows with no error (silent data loss). This is the failure
mode BC-2.16.021 §Invariants guards against.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | xdome-wave-c-g5-hs-raw-extensions-correction | 2026-09-01 | product-owner | Correct raw_extensions encoding expectation per ADR-058 §I2 and D-2406 adjudication. raw_extensions is an Arrow Utf8 column emitted as a JSON-serialized STRING on the wire (`"raw_extensions":"{...}"`), not a native JSON object. Removes "not a string" / "native object" / "JSON object" wording from §Scenario point 4, §BDD supplement, §Behavioral Contract Linkage, §Verification Approach step 6, and §Evaluation Rubric. PARTIAL credit assigned when raw_extensions is a native object (structurally wrong per ADR-058 §I2). D-2381 native-JSON rule applies only to column_type="json" values INSIDE raw_extensions, not the container. Mirrors G4 HS-001/HS-003 v1.1 amendment. |
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-028 group for S-CLAROTY-ORGPOLICY-001. Wire-shape assertion: class_uid=3004, firewall_group_name→name Tier-1 REQUIRED rename, raw_extensions aggregation. URL vs envelope key asymmetry (path _fw_groups vs envelope organization_firewall_groups) explicitly verified via non-empty result set check. BC-2.16.021 §Postconditions 1 and 3. SINGLE-USE. |
