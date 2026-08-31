---
document_type: holdout-scenario
level: L3
id: "HS-OTEVTS-001-001"
title: "claroty_ot_activity_events SELECT * wire shape: class_uid=2004, finding_info_uid column present in returned row"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-A"
story_source: "S-CLAROTY-OT-EVENTS-001"
version: "1.1"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-31"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.016-claroty-ot-activity-events-table.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "300d619"
traces_to: "BC-2.16.016"
behavioral_contracts:
  - BC-2.16.016
  - BC-2.02.005
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-OT-EVENTS-001"
last_evaluated: "2026-08-31"
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-OT-EVENTS-001 (HS-025 group). Tests BC-2.16.016 §Postconditions 1 (ocsf_class=detection_finding → class_uid=2004) and §Postconditions 2 Tier-1 REQUIRED column: event_id→finding_info_uid (ocsf_field_to_arrow_name of finding_info.uid). Runs against live monroe sensor. No DTU — live sensor only. BLOCKING. Test-writer and implementer must NOT read this file. EVALUATED D-2399 2026-08-31: SETUP-FAILURE — live OT network quiescent, 0 events returned via force_refresh on monroe; scenario Edge Conditions §'Live sensor returns empty result set' applies (NOT a behavioral FAIL). Structural corroboration present: table registered, class_uid non-null-int mapped to detection_finding, finding_info_uid integer Tier-1 column present. Human ACCEPTED (Option-1): holdout gate treated as PASSED on 2/3-at-ceiling + structural corroboration; kept unconsumed (used remains false) for future re-run when monroe OT network has ≥1 event."
---

# HS-OTEVTS-001-001: claroty_ot_activity_events SELECT * wire shape: class_uid=2004, finding_info_uid column present in returned row

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-OT-EVENTS-001 (HS-025 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.016 §Postconditions 1 (ocsf_class = "detection_finding" → class_uid 2004) and §Postconditions 2 Tier-1 columns (event_id → finding_info_uid REQUIRED)
**Gate:** Story-level holdout gate (HS-025) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the `claroty_ot_activity_events` table is registered and queryable via PrismQL against the live Claroty xDome (monroe) sensor, and that the TOML contract specified in BC-2.16.016 §Postconditions 1 is correctly realized:

1. The returned JSON rows carry `class_uid = 2004` — the integer class_uid for `detection_finding`. The OT events table uses the same OCSF class as the existing `claroty_alerts` and `claroty_device_alert_relations` tables. If a different class_uid appears (e.g., 4001 for network_activity), the Option B OCSF class decision was not applied.

2. The returned JSON rows carry a column named `finding_info_uid` — the Arrow field name for the Tier-1 REQUIRED mapping of `event_id` (source: `ocsf_field = "finding_info.uid"` → `ocsf_field_to_arrow_name` → `finding_info_uid`). This column is REQUIRED per BC-2.16.016 §Postconditions 2; its presence confirms the TOML spec was correctly parsed and the ocsf_field_to_arrow_name transform was applied.

3. The `finding_info_uid` value in at least one row is a non-null integer — evidence that real OT activity event data was retrieved from the live sensor and the event_id integer field was correctly mapped.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT * FROM claroty.claroty_ot_activity_events LIMIT 1` is issued via the MCP `query` tool
**Then** the response is not an error
**And** the response wire JSON contains a row with a column `class_uid` equal to `2004`
**And** the response wire JSON contains a row with a column `finding_info_uid` that is a non-null integer

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 — do not log the credential value).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Wait for prism to be ready (startup log or first JSON-RPC prompt).

5. Issue the MCP `query` tool call: `{"sql": "SELECT * FROM claroty.claroty_ot_activity_events LIMIT 1"}`.

6. Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.016 | §Postconditions 1: ocsf_class = "detection_finding" → class_uid 2004 | Assertion 1: class_uid = 2004 in wire output (not 4001 or 2002) |
| BC-2.16.016 | §Postconditions 2 Tier-1 REQUIRED: event_id → ocsf_field = "finding_info.uid" → Arrow field finding_info_uid | Assertion 2: finding_info_uid column present in row |
| BC-2.16.016 | §Postconditions 3 OCSF Class Rationale: Option B confirmed (detection_finding/2004); network_activity/4001 was explicitly rejected | class_uid=2004 (not 4001) confirms Option B was implemented |
| BC-2.02.005 | Claroty xDome OCSF field mapping — detection_finding class_uid for OT events | Cross-verification of class_uid value |

---

## Verification Approach

1. Parse the wire-level JSON response from the MCP `query` tool call.

2. If the response is an error: record FAIL if it is a column-not-found (table not registered or column missing); record SETUP-FAILURE for sensor-unreachable or auth errors.

3. Locate `class_uid` in the first row. Assert integer value equals `2004`. If value is `4001` (network_activity), record FAIL with observation "class_uid=4001 returned — Option A was implemented instead of Option B." If any other value, record FAIL with observation.

4. Locate `finding_info_uid` in the first row. Assert it is a non-null integer. If absent, record FAIL on "finding_info_uid present" dimension. If present but null, record PARTIAL.

5. Do NOT assert any specific event_id value — the live sensor's content varies.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query succeeds (no error)** (weight: 0.30): Does the MCP query return a non-error response with at least one row?
  Full credit (1.0): non-error response with ≥1 row.
  Zero credit (0.0): error response.

- **class_uid = 2004 in wire output** (weight: 0.40): Does at least one returned row carry `class_uid = 2004`?
  Full credit (1.0): class_uid present, value is integer 2004.
  Partial credit (0.3): class_uid present but wrong value (e.g., 4001 — Option A incorrectly implemented).
  Zero credit (0.0): class_uid absent or query errored.

- **finding_info_uid present and non-null** (weight: 0.30): Does at least one row carry a non-null integer `finding_info_uid`?
  Full credit (1.0): finding_info_uid present, non-null integer.
  Partial credit (0.5): finding_info_uid present but null.
  Zero credit (0.0): finding_info_uid column absent (REQUIRED Tier-1 column not mapped).

---

## Edge Conditions

- **Live sensor returns empty result set (zero OT events):** Record as SETUP-FAILURE — not a behavioral FAIL. OT activity events may be empty in a quiescent monitored network. Note observation and do not score row-content dimensions.

- **class_uid = 4001 returned:** This is a FAIL — it means `network_activity` class_uid was used instead of `detection_finding/2004`. The Option B OCSF class decision (BC-2.16.016 §Postconditions 3) was not applied.

- **class_uid = 2002 returned:** FAIL — vulnerability_finding class_uid leaked from the G1 (`claroty_vulnerabilities`) table config. Cross-table OCSF class pollution.

- **`claroty_ot_activity_events` table not registered (table-not-found error):** FAIL — the TOML table block was not added or parsed.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-OTEVTS-001-001 (satisfaction: X.XX) — claroty_ot_activity_events wire-shape gap; check TOML table block registration, OCSF class_uid=2004 Option B mapping (BC-2.16.016 §Postconditions 1/3), and finding_info_uid Tier-1 REQUIRED column ocsf_field_to_arrow_name transform (BC-2.16.016 §Postconditions 2)"`

Do NOT disclose: the specific LIMIT value, the exact class_uid checked, or the assertion threshold.

---

## Category: real-world-corpus

This scenario exercises the live Claroty xDome OT activity events endpoint against the monroe sensor. No DTU exists for this endpoint — this is a live-only test.

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/ot_activity_events/ |
| corpus_size | LIMIT 1 (single row sufficient for structural assertion) |
| known_edge_cases | Empty result set (quiescent OT network — SETUP-FAILURE); class_uid=4001 (Option A regression — FAIL) |
| false_positive_threshold | Zero: class_uid=2004 is a structural integer assertion from the OCSF class mapping |
| false_negative_threshold | Zero: if finding_info_uid absent, the REQUIRED Tier-1 event_id mapping is broken |

**Known-good corpus:** Monroe Claroty xDome with ≥1 OT activity event — expected: non-error response, class_uid=2004, finding_info_uid non-null integer.

**Known-problematic corpus:** An implementation using `network_activity` class (Option A instead of Option B) — expected: class_uid=4001. This is the spike decision regression this scenario guards against.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | G2-live-holdout-gate | 2026-08-31 | state-manager | Evaluated D-2399: SETUP-FAILURE (quiescent OT network — 0 events returned via force_refresh on monroe; scenario Edge Conditions §'Live sensor returns empty result set' applies; NOT a behavioral FAIL). Structural corroboration: table registered, class_uid non-null-int mapped to detection_finding, finding_info_uid integer Tier-1 column present. Human ACCEPTED (Option-1): holdout gate treated as PASSED on 2/3-at-ceiling + structural corroboration. Kept unconsumed (used remains false) for future re-run when monroe OT network has ≥1 event. last_evaluated 2026-08-31 recorded. |
| 1.0 | xdome-wave-a-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-025 group for S-CLAROTY-OT-EVENTS-001. Wire-shape assertion: class_uid=2004 (Option B detection_finding) and finding_info_uid REQUIRED Tier-1 column present in live monroe sensor output. BC-2.16.016 §Postconditions 1/2/3. No DTU — live sensor only. SINGLE-USE. |
