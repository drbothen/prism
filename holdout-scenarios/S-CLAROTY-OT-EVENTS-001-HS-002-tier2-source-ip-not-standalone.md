---
document_type: holdout-scenario
level: L3
id: "HS-OTEVTS-001-002"
title: "claroty_ot_activity_events: SELECT source_ip raises E-QUERY-038 (Tier-2 network field not standalone); raw_extensions queryable"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-CLAROTY-XDOME-WAVE-A"
story_source: "S-CLAROTY-OT-EVENTS-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-24"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.016-claroty-ot-activity-events-table.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "8453d6f"
traces_to: "BC-2.16.016"
behavioral_contracts:
  - BC-2.16.016
  - BC-2.11.016
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-OT-EVENTS-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-OT-EVENTS-001 (HS-025 group). Tests BC-2.16.016 §Invariants: network 5-tuple Tier-2 fields not exposed as standalone columns; SELECT source_ip raises E-QUERY-038 with raw_extensions in available_columns but NOT source_ip. ALSO tests BC-2.16.016 §EC-016-016-006 edge case. Complement: SELECT raw_extensions succeeds. No DTU — live sensor only. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-OTEVTS-001-002: claroty_ot_activity_events: SELECT source_ip raises E-QUERY-038 (Tier-2 network field not standalone); raw_extensions queryable

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-OT-EVENTS-001 (HS-025 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.016 §Invariants (network 5-tuple Tier-2 fields not standalone; E-QUERY-038 on raw name); BC-2.16.016 EC-016-016-006; BC-2.11.016 E-QUERY-038 column-not-found plan gate
**Gate:** Story-level holdout gate (HS-025) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

The spike findings for G2 (xdome-endpoint-expansion-plan.md + endpoint-spike-findings.md §Spike 2) established that the network 5-tuple fields — `source_ip`, `dest_ip`, `protocol`, `dest_port`, `source_port`, `ip_protocol` — are Tier-2 under `ocsf_column_naming = true`. They are NOT Tier-1 (no `ocsf_field` declarations). Therefore they must NOT be exposed as standalone Arrow columns; they aggregate into `raw_extensions`.

This scenario validates two complementary assertions:

1. **Tier-2 network field rejected:** `SELECT source_ip FROM claroty.claroty_ot_activity_events LIMIT 1` raises E-QUERY-038. The `available_columns` field in the error payload contains `raw_extensions` (the correct aggregation column) but does NOT contain `source_ip`.

2. **raw_extensions queryable (complement):** `SELECT raw_extensions FROM claroty.claroty_ot_activity_events LIMIT 1` succeeds at the plan gate (no E-QUERY-038). This confirms that the Tier-2 data path is accessible and the analyst can retrieve network 5-tuple data via `raw_extensions`.

If assertion 1 fails (no E-QUERY-038 on `source_ip`), the network field was incorrectly exposed as a standalone Arrow column. If assertion 2 fails (E-QUERY-038 on `raw_extensions`), the Tier-2 aggregation column itself was not added to the available set — which means ALL Tier-2 data is inaccessible.

**BDD supplement:**

**Given** prism MCP stdio is started with the claroty sensor configured (bearer_token credential set for monroe)
**When** `SELECT source_ip FROM claroty.claroty_ot_activity_events LIMIT 1` is issued via the MCP `query` tool
**Then** the response IS an E-QUERY-038 error
**And** the `available_columns` field contains `raw_extensions` but NOT `source_ip`
**When** `SELECT raw_extensions FROM claroty.claroty_ot_activity_events LIMIT 1` is issued via the MCP `query` tool
**Then** the response is NOT an E-QUERY-038 error

---

## Setup Instructions

1. Confirm prism is built from the story branch at the current story HEAD commit.

2. Confirm the claroty bearer_token credential is configured for the monroe sensor (AD-017 — do not log the credential value).

3. Start prism in MCP stdio mode with the claroty sensor spec included.

4. Wait for prism to be ready.

5. Issue first MCP `query` tool call: `{"sql": "SELECT source_ip FROM claroty.claroty_ot_activity_events LIMIT 1"}`. Capture the full response.

6. Issue second MCP `query` tool call: `{"sql": "SELECT raw_extensions FROM claroty.claroty_ot_activity_events LIMIT 1"}`. Capture the full response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.016 | §Invariants: network 5-tuple fields Tier-2 — NOT exposed as standalone Arrow columns; queries against them by raw TOML name MUST raise E-QUERY-038 | Assertion 1: SELECT source_ip raises E-QUERY-038 |
| BC-2.16.016 | §Invariants: available_columns in E-QUERY-038 for Tier-2 raw name contains raw_extensions, NOT source_ip | Assertion 1: available_columns verification |
| BC-2.16.016 | EC-016-016-006: query against Tier-2 network field source_ip by raw name → E-QUERY-038; available_columns contains raw_extensions but NOT source_ip | Direct edge case verification |
| BC-2.11.016 | E-QUERY-038: available_columns is the set of registered Arrow columns | Assertion 1: available_columns format and content |
| ADR-058 | Tier-2 cols no ocsf_field → aggregate to raw_extensions; raw TOML name NOT in plan-gate registry | Both assertions together confirm ADR-058 Tier-2 path |

---

## Verification Approach

1. Parse response from first query (`SELECT source_ip`).
   - If response does NOT contain E-QUERY-038: record FAIL on "source_ip rejected" dimension with observation "source_ip was not rejected — network field incorrectly exposed as standalone Arrow column."
   - If E-QUERY-038 present: inspect `available_columns`.
     - Assert `raw_extensions` is present in available_columns.
     - Assert `source_ip` is NOT present in available_columns.
     - Also check: `finding_info_uid`, `time`, `activity_name`, `message` should be present in available_columns (Tier-1 columns).
     - Record PASS if raw_extensions present and source_ip absent; PARTIAL if raw_extensions present but source_ip also present.

2. Parse response from second query (`SELECT raw_extensions`).
   - If response is E-QUERY-038: record FAIL on "raw_extensions queryable" dimension — Tier-2 aggregation column not in available set.
   - If response is any other error (sensor unreachable, auth failure): PASS on plan-gate dimension.
   - If response is success: PASS.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **source_ip raises E-QUERY-038** (weight: 0.35): Does `SELECT source_ip` return E-QUERY-038?
  Full credit (1.0): E-QUERY-038 fires.
  Zero credit (0.0): no E-QUERY-038 — source_ip incorrectly exposed as standalone column.

- **available_columns contains raw_extensions but NOT source_ip** (weight: 0.35): When E-QUERY-038 fires on source_ip, is available_columns correct?
  Full credit (1.0): raw_extensions present, source_ip absent.
  Partial credit (0.5): E-QUERY-038 fires but available_columns missing raw_extensions, OR source_ip unexpectedly present.
  Zero credit (0.0): E-QUERY-038 did not fire — dimension cannot be scored.

- **raw_extensions queryable (no E-QUERY-038)** (weight: 0.30): Does `SELECT raw_extensions` return a non-E-QUERY-038 response?
  Full credit (1.0): non-E-QUERY-038 response (any non-column-not-found outcome including sensor unreachable = PASS).
  Zero credit (0.0): E-QUERY-038 on raw_extensions — Tier-2 aggregation column not in available set.

---

## Edge Conditions

- **`claroty_ot_activity_events` not registered at all (table-not-found):** Both queries fail with table-not-found. Record as behavioral FAIL on all dimensions (not SETUP-FAILURE) — the table was not added.

- **E-QUERY-038 fires on source_ip but available_columns is empty or wrong:** PARTIAL (0.5) on available_columns dimension. The rejection is correct but the error payload is incomplete.

- **raw_extensions raises E-QUERY-038 (same error as source_ip):** Both Tier-2 field AND aggregation column are missing from plan gate — indicates ocsf_column_naming Tier-2 path was not implemented at all. Score zero on raw_extensions dimension.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-OTEVTS-001-002 (satisfaction: X.XX) — claroty_ot_activity_events Tier-2 network-field plan-gate gap; check that Tier-2 columns (no ocsf_field) are NOT registered as standalone Arrow columns and that raw_extensions IS in the available set (BC-2.16.016 §Invariants, EC-016-016-006; BC-2.11.016 E-QUERY-038; ADR-058 Tier-2 path)"`

Do NOT disclose: the specific field name queried, the LIMIT value, or the exact assertion threshold.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Live monroe Claroty xDome sensor — POST /api/v1/ot_activity_events/ (plan-gate assertions, not content assertions — sensor may return empty result or unreachable) |
| corpus_size | LIMIT 1 (plan-gate assertions fire before any HTTP fetch; sensor content irrelevant for E-QUERY-038 assertions) |
| known_edge_cases | Empty sensor result or unreachable sensor for raw_extensions query — PASS on plan-gate dimension (E-QUERY-038 did not fire = plan gate accepted column) |
| false_positive_threshold | Zero: E-QUERY-038 is a plan-gate assertion — fires before any data fetch |
| false_negative_threshold | Zero: absence of E-QUERY-038 on source_ip means Tier-2 path not implemented |

**Known-good corpus:** Correctly implemented `claroty_ot_activity_events` TOML with source_ip as Tier-2 — expected: source_ip raises E-QUERY-038; raw_extensions accepted by plan gate.

**Known-problematic corpus:** An implementation that exposes ALL columns (including Tier-2 fields) as standalone Arrow columns — expected: no E-QUERY-038 on source_ip. This is the ADR-058 Tier-2 contract violation this scenario catches.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-a-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring. HS-025 group for S-CLAROTY-OT-EVENTS-001. Tier-2 network field plan-gate rejection: source_ip E-QUERY-038 with raw_extensions in available_columns; complement: raw_extensions accepted. BC-2.16.016 §Invariants + EC-016-016-006 + BC-2.11.016 + ADR-058. No DTU — live sensor only. SINGLE-USE. |
