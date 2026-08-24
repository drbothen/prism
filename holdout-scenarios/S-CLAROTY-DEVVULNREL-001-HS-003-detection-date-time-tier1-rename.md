---
document_type: holdout-scenario
level: L3
id: "HS-DEVVULNREL-001-003"
title: "claroty_device_vulnerability_relations Tier-1 rename: SELECT time accepts (detection date); SELECT device_vulnerability_detection_date rejected"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-DEVVULNREL-001 (HS-026 group). Tests BC-2.16.017 §Postconditions 2 (Tier-1 OCSF rename: device_vulnerability_detection_date maps to ocsf_field='time', Arrow field name='time'). SELECT time succeeds; SELECT device_vulnerability_detection_date raises E-QUERY-038. Runs against live monroe sensor. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-DEVVULNREL-001-003: claroty_device_vulnerability_relations Tier-1 rename: SELECT time accepts (detection date); SELECT device_vulnerability_detection_date rejected

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-DEVVULNREL-001 (HS-026 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.017 §Postconditions 2 Tier-1 (`device_vulnerability_detection_date` → `ocsf_field = "time"` → `ocsf_field_to_arrow_name("time")` → Arrow field `time`); BC-2.16.017 §Invariants (Tier-2 raw name raises E-QUERY-038; `time` is in available_columns)
**Gate:** Story-level holdout gate (HS-026) — SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the Tier-1 OCSF column rename for
`claroty_device_vulnerability_relations`. The TOML column `device_vulnerability_detection_date`
carries `ocsf_field = "time"`, which means:

- The raw TOML column name `device_vulnerability_detection_date` is NOT exposed as a
  standalone Arrow column — it is replaced by the OCSF canonical name `time`.
- `SELECT time FROM claroty.claroty_device_vulnerability_relations` MUST succeed.
- `SELECT device_vulnerability_detection_date FROM claroty.claroty_device_vulnerability_relations`
  MUST raise E-QUERY-038, with `time` listed in `available_columns` (the OCSF name is the
  queryable name, not the raw TOML name).

This mirrors the behavior established for `claroty_ot_activity_events` (BC-2.16.016 §PC2,
EC-016-016-003, TV-BC-2.16.016-005): the OCSF rename pattern is uniform across all sensor tables
using `ocsf_column_naming = true`. If the rename is not applied, MSSP analysts using the standard
OCSF `time` column for temporal queries will get E-QUERY-038 on `time` instead of a result —
a severe behavioral regression.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT time FROM claroty.claroty_device_vulnerability_relations LIMIT 1` is issued via MCP `query` tool
**Then** the response is not an error
**And** the response wire JSON contains at least one row (or empty result but not column-not-found)
**When** `SELECT device_vulnerability_detection_date FROM claroty.claroty_device_vulnerability_relations LIMIT 1` is issued via MCP `query` tool
**Then** the response is an E-QUERY-038 column-not-found error
**And** the error's available_columns list contains `time`

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 —
   reference by config key only, never include credential value in output).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Wait for prism to be ready.

5. Issue Query A: `{"sql": "SELECT time FROM claroty.claroty_device_vulnerability_relations LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

6. Issue Query B: `{"sql": "SELECT device_vulnerability_detection_date FROM claroty.claroty_device_vulnerability_relations LIMIT 1"}`.
   Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.017 | §Postconditions 2 Tier-1: device_vulnerability_detection_date → ocsf_field = "time" → Arrow field "time" | Query A: SELECT time succeeds (OCSF renamed column queryable) |
| BC-2.16.017 | §Invariants: raw TOML name raises E-QUERY-038; available_columns contains OCSF names | Query B: SELECT device_vulnerability_detection_date raises E-QUERY-038 with time in available_columns |
| BC-2.16.017 | EC-016-017-001 (implicit): REQUIRED vulnerability_name present; table loads and executes queries | Both queries exercise a live, functional table load |

---

## Verification Approach

**Query A: OCSF rename acceptance:**

1. Parse the wire-level JSON response from Query A.
2. If the response is an error with column-not-found for `time`, record FAIL on the Tier-1 rename
   dimension — the OCSF rename was not applied.
3. If the response is a non-error (either rows present or empty result for no data), record PASS
   on the rename dimension. An empty result from a live sensor (0 rows) is acceptable — it means
   the query was syntactically and semantically accepted.
4. If rows are returned, optionally confirm the `time` column values are ISO 8601 datetime strings
   or null (not random values — e.g., not integers or booleans).

**Query B: Raw name rejection:**

1. Parse the wire-level JSON response from Query B.
2. Assert the response is an ERROR (column-not-found / E-QUERY-038). If it succeeds, record FAIL.
3. Assert the error's `available_columns` (or equivalent field) contains `time`.
4. Assert the error's `available_columns` does NOT contain `device_vulnerability_detection_date`
   (the raw TOML name must NOT be available).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **SELECT time succeeds (Query A)** (weight: 0.50): Does Query A return a non-error response?
  Full credit (1.0): non-error response (empty result or rows with time column).
  Zero credit (0.0): column-not-found error for `time` (OCSF rename not applied).

- **SELECT device_vulnerability_detection_date rejected (Query B)** (weight: 0.35): Does Query B
  return a column-not-found error?
  Full credit (1.0): error response, column-not-found, `time` in available_columns, raw name NOT in available_columns.
  Partial credit (0.5): error response, column-not-found, but available_columns missing or incorrect.
  Zero credit (0.0): Query B succeeds (raw TOML name exposed as Arrow column — rename not applied).

- **time column values are Datetime-typed (Query A, if rows returned)** (weight: 0.15): If Query
  A returned ≥1 non-null time value, is it a datetime-typed value (not integer or string "null")?
  Full credit (1.0): datetime value present, ISO 8601 formatted.
  Neutral (0.5): time value is null (valid — undetected vulnerabilities may have no date).
  Zero credit (0.0): time value is present but non-datetime type (coercion failed).

---

## Edge Conditions

- **Query A returns empty result (0 rows — all detection dates null, REQUIRED column null):**
  Record as non-error (PASS on Query A success dimension). The `time` column being queryable
  but null is acceptable; the assertion is about plan-gate acceptance, not data presence.

- **Query A returns E-QUERY-038 for `time` (rename not applied):** This is the primary failure
  mode. Record FAIL (0.0) on Query A dimension. Indicates the `ocsf_field = "time"` Tier-1
  mapping was not processed by `ocsf_field_to_arrow_name`.

- **Query B's available_columns contains both `time` AND `device_vulnerability_detection_date`:**
  Record PARTIAL on Query B dimension — the raw name should not appear in available_columns if
  the rename is correctly applied.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DEVVULNREL-001-003 (satisfaction: X.XX) — claroty_device_vulnerability_relations OCSF rename gap; check that device_vulnerability_detection_date ocsf_field='time' produces Arrow field 'time' (BC-2.16.017 §Postconditions 2 Tier-1) and that the raw TOML name raises E-QUERY-038 (BC-2.16.017 §Invariants)"`

Do NOT disclose: the specific SQL queries used, the exact available_columns expected, or the
assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/device_vulnerability_relations/ |
| corpus_size | LIMIT 1 (single row sufficient; empty result also acceptable for positive query) |
| known_edge_cases | All detection dates null (empty time column — not a fail); table missing from TOML (both queries fail differently) |
| false_positive_threshold | Zero: plan-gate acceptance/rejection is deterministic and not data-dependent |
| false_negative_threshold | Zero: if SELECT time raises E-QUERY-038, the OCSF rename is broken |

**Known-good corpus:** monroe Claroty xDome with the claroty_device_vulnerability_relations table
registered in TOML — expected: SELECT time succeeds; SELECT device_vulnerability_detection_date
raises E-QUERY-038 with time in available_columns.

**Known-problematic corpus:** A table configuration where `ocsf_field = "time"` was omitted from
the `device_vulnerability_detection_date` column, leaving it as Tier-2 — expected: SELECT time
raises E-QUERY-038 (wrong — the detection date should be Tier-1 per BC-2.16.017 §PC2).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-b-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-026 group for S-CLAROTY-DEVVULNREL-001. Tier-1 rename: device_vulnerability_detection_date → ocsf_field='time' → Arrow field 'time'. SELECT time succeeds; SELECT device_vulnerability_detection_date raises E-QUERY-038 with 'time' in available_columns (BC-2.16.017 §PC2 + §Invariants). Mirrors BC-2.16.016 pattern for detection_time→time rename. SINGLE-USE, live monroe only (no DTU per D-2200). |
