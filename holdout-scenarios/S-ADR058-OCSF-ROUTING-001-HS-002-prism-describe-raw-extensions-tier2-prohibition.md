---
document_type: holdout-scenario
level: L3
id: "HS-ROUTING-001-A-002"
title: "prism_describe returns raw_extensions ColumnDescriptor and suppresses phantom col.name descriptors for Claroty alerts"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-OCSF-ROUTING"
story_source: "S-ADR058-OCSF-ROUTING-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-21T00:00:00Z"
modified: "2026-08-21"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "caaa833"
traces_to: "BC-2.16.003"
behavioral_contracts:
  - BC-2.16.003
verification_properties:
  - VP-017
lifecycle_status: active
introduced: "S-ADR058-OCSF-ROUTING-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-ADR058-OCSF-ROUTING-001 — prism_describe Tier-2 prohibition: no phantom col.name ColumnDescriptors for ocsf_field==None columns; exactly one raw_extensions ColumnDescriptor with four-field shape enumerating source keys; finding_info_uid Tier-1 descriptor present. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ROUTING-001-A-002: prism_describe returns raw_extensions ColumnDescriptor and suppresses phantom col.name descriptors for Claroty alerts

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-ROUTING-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 §Interpretation A EC-016-013-027 (Tier-2 prohibition + raw_extensions ColumnDescriptor four-field shape); ADR-058 §G (Tier-1/Tier-2 prism_describe model)
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the agent-discoverability wire shape for the MCP `prism_describe` tool after Stage 2. When `ocsf_column_naming = true` for Claroty, `prism_describe` for `claroty.alerts` must implement the Tier-1/Tier-2 model per ADR-058 §G / BC-2.16.003 §Interpretation A:

**Tier-1 behavior (ocsf_field == Some):** `prism_describe` emits a `ColumnDescriptor` with `name = ocsf_field_to_arrow_name(ocsf_field)`. For the `id` column (`ocsf_field = "finding_info.uid"`), this is `name = "finding_info_uid"`.

**Tier-2 behavior (ocsf_field == None):** `prism_describe` MUST NOT emit individual ColumnDescriptors for columns with `ocsf_field == None` (e.g., `category`, `alert_type_name`, `devices_count`, `alert_class`, `ot_devices_count` after KF-08/09/10 corrections). Instead, it MUST emit exactly ONE `raw_extensions` ColumnDescriptor with all four required fields: `name = "raw_extensions"`, `col_type = Json`, `nullable = true`, and `description` enumerating the `ocsf_field == None` source keys.

The pre-Stage-2 (or failed Stage-2) behavior: `prism_describe` emits individual ColumnDescriptors for all columns including `category`, `alert_type_name`, `devices_count`. An LLM agent reading these descriptors would construct `SELECT category FROM claroty.alerts`, which succeeds with the old schema but returns no data with the new schema (because `category` is now in `raw_extensions`). The phantom column names create a silent query failure at the agent/query interface.

The post-Stage-2 behavior: `prism_describe` emits `name = "finding_info_uid"` (not `"id"`) for the Tier-1 alert ID column, emits NO descriptor for `category`, `alert_type_name`, `devices_count`, `alert_class`, or `ot_devices_count`, and emits exactly ONE `raw_extensions` ColumnDescriptor with `col_type = Json`, `nullable = true`, and description text that names those five columns as source keys.

**Behavioral assertions:**

1. prism is started in MCP stdio mode with the Claroty TOML (`ocsf_column_naming = true` applied by AC-005 of this story).
2. A `prism_describe` MCP tool call requests the schema for `claroty.alerts` (no mock server needed — `prism_describe` reads the TOML spec, not the live sensor API).
3. The serialized JSON response lists ColumnDescriptors. Assert ALL FOUR:
   (i) NO ColumnDescriptor has `name = "category"`, `name = "alert_type_name"`, `name = "devices_count"`, `name = "alert_class"`, or `name = "ot_devices_count"` — these are Tier-2 columns (ocsf_field == None after KF-08/09/10 corrections) and MUST NOT appear as individual queryable names.
   (ii) EXACTLY ONE ColumnDescriptor has `name = "raw_extensions"` — count must be exactly 1, not zero, not two.
   (iii) The `raw_extensions` ColumnDescriptor has `col_type = "Json"` (or equivalent JSON type representation) and `nullable = true`.
   (iv) The `raw_extensions` ColumnDescriptor description text contains the string `"category"` AND `"alert_type_name"` AND `"devices_count"` as source key enumerations.
4. The response DOES contain a ColumnDescriptor with `name = "finding_info_uid"` (Tier-1 — `id` column's OCSF-flattened name).
5. The response does NOT contain a ColumnDescriptor with `name = "id"` (the raw col.name — not advertised as a queryable field after Stage 2).

**BDD supplement:**

**Given** prism MCP stdio is configured with Claroty TOML having `ocsf_column_naming = true`
**When** `prism_describe claroty.alerts` is called via the MCP `prism_describe` tool
**Then** the serialized JSON response contains a ColumnDescriptor with `name = "finding_info_uid"` (Tier-1 — KF-03 corrected)
**And** the response does NOT contain any ColumnDescriptor with `name = "category"`, `name = "alert_type_name"`, or `name = "devices_count"` (Tier-2 prohibition — phantom queryable names absent)
**And** the response contains EXACTLY ONE ColumnDescriptor with `name = "raw_extensions"`, `col_type = Json`, `nullable = true`
**And** the `raw_extensions` ColumnDescriptor description enumerates `"category"`, `"alert_type_name"`, and `"devices_count"` as source keys

---

## Setup Instructions

1. Confirm `crates/prism-sensors/specs/claroty.sensor.toml` has `ocsf_column_naming = true` and the KF-08/09/10 corrections applied (removing `ocsf_field` from `alerts.category`, `alerts.alert_type_name`, `alerts.devices_count`). If not, record SETUP-FAILURE.

2. Start prism in MCP stdio mode (no mock HTTP server needed — `prism_describe` reads the TOML spec at startup, not the live sensor API). Capture stderr.

3. Issue the `prism_describe` MCP tool call targeting `claroty.alerts`. The exact MCP tool interface: the tool is named `prism_describe` (or equivalent schema-describe tool in the prism MCP server); pass `claroty.alerts` as the target. Capture the full serialized JSON response.

4. Parse the response to extract the list of ColumnDescriptors returned for the `alerts` table.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-027: prism_describe MUST NOT emit individual ColumnDescriptor for ocsf_field==None columns when ocsf_column_naming=true | Assertion (i): no category/alert_type_name/devices_count descriptors |
| BC-2.16.003 | EC-016-013-027: prism_describe MUST emit exactly ONE raw_extensions ColumnDescriptor per table | Assertion (ii): count == 1 |
| BC-2.16.003 | §Interpretation A EC-016-013-027: raw_extensions ColumnDescriptor col_type=Json, nullable=true | Assertion (iii): four-field shape |
| BC-2.16.003 | §Interpretation A: description enumerates ocsf_field==None col.name source keys | Assertion (iv): "category", "alert_type_name", "devices_count" in description text |
| ADR-058 §G | Tier-1: ocsf_field==Some columns emit ColumnDescriptor with name=ocsf_field_to_arrow_name(ocsf_field) | Assertion (4): finding_info_uid present |
| ADR-058 §G | Tier-2: ocsf_field==None columns MUST NOT emit individual ColumnDescriptor names | Assertion (i): Tier-2 prohibition |
| BC-2.16.003 | §Interpretation A: prism_describe describes what is actually queryable — phantom names cause agent to write broken queries | Prevents LLM agent from constructing SELECT category FROM claroty.alerts which would fail |

---

## Verification Approach

1. Build the prism binary (`cargo build --release -p prism-bin` or `just build`).
2. Launch prism in MCP stdio mode.
3. Send the `prism_describe` MCP tool call for `claroty.alerts`.
4. Receive the full MCP JSON response. Extract the ColumnDescriptors list.
5. Assert ALL of:
   - The response is valid JSON (parse without error).
   - Count of ColumnDescriptors with `name` equal to any of `"category"`, `"alert_type_name"`, `"devices_count"`, `"alert_class"`, `"ot_devices_count"` is ZERO. Each is a Tier-2 column (ocsf_field == None after KF corrections) and must not be advertised individually.
   - Count of ColumnDescriptors with `name = "raw_extensions"` is EXACTLY ONE.
   - The single `raw_extensions` ColumnDescriptor has: (a) `col_type` corresponding to JSON type (the string value of the type field should indicate Json/JSON); (b) `nullable = true` (not false, not null).
   - The `raw_extensions` ColumnDescriptor `description` field (as a string) contains the substring `"category"` AND the substring `"alert_type_name"` AND the substring `"devices_count"`.
   - Count of ColumnDescriptors with `name = "finding_info_uid"` is AT LEAST ONE (Tier-1 mapping for alerts.id → ocsf_field = "finding_info.uid").
   - Count of ColumnDescriptors with `name = "id"` is ZERO (raw col.name must not be advertised as queryable).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **Tier-2 phantom prohibition** (weight: 0.35): Are there zero ColumnDescriptors with names `"category"`, `"alert_type_name"`, `"devices_count"`, `"alert_class"`, or `"ot_devices_count"`?
  Full credit (1.0): none of these phantom names appear in the descriptor list.
  Partial credit (0.5): some but not all phantom names are suppressed.
  Zero credit (0.0): any of these phantom names appears as a ColumnDescriptor — the old pre-Stage-2 behavior is active and would mislead an LLM agent.

- **raw_extensions ColumnDescriptor present with correct shape** (weight: 0.35): Is there exactly ONE `raw_extensions` ColumnDescriptor with `col_type = Json` and `nullable = true`?
  Full credit (1.0): exactly one present with both shape fields correct.
  Partial credit (0.5): present but shape incomplete (wrong col_type or nullable).
  Zero credit (0.0): absent (count = 0) or count > 1.

- **Tier-1 finding_info_uid present** (weight: 0.20): Does the descriptor list include `name = "finding_info_uid"` (Tier-1 mapping for alerts.id)?
  Full credit (1.0): finding_info_uid present; id absent.
  Partial credit (0.5): finding_info_uid present but id also present (old col.name still advertised).
  Zero credit (0.0): finding_info_uid absent.

- **Source key enumeration in description** (weight: 0.10): Does the `raw_extensions` ColumnDescriptor description mention `"category"`, `"alert_type_name"`, and `"devices_count"` as source keys?
  Full credit (1.0): all three substrings present in description text.
  Partial credit (0.5): at least one but not all three present.
  Zero credit (0.0): description empty or none of the three present.

---

## Edge Conditions

- **ocsf_column_naming = false (flag not applied):** If the Claroty TOML still has `ocsf_column_naming = false` or absent, `prism_describe` emits individual ColumnDescriptors for all columns using the old col.name model. All Tier-2 prohibition assertions fail and raw_extensions is absent. Record as SETUP-FAILURE ("claroty TOML flag not applied").

- **KF-08/09/10 corrections not applied (category/alert_type_name/devices_count still have ocsf_field):** These columns are Tier-1 in this case (they have an ocsf_field), so they appear as individual descriptors with their OCSF-flattened names (e.g., `"class_name"`, `"type_name"`, `"count"`). The Tier-2 phantom prohibition test still passes (no `"category"` descriptor), but the overall mapping would be wrong. The evaluator should distinguish this from a behavioral fail — flag it as KF correction gap, not a Stage 2 routing failure.

- **prism_describe not implemented for table-level describe:** Record as SETUP-FAILURE. Do NOT mark as behavioral FAIL.

- **raw_extensions ColumnDescriptor has description = null:** This is a FAIL on the source-key enumeration dimension (weight 0.10), but not a FAIL on the shape dimension if col_type and nullable are correct.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ROUTING-001-A-002 (satisfaction: X.XX) — prism_describe for Claroty alerts emits phantom col.name descriptors or missing/malformed raw_extensions ColumnDescriptor; check prism_describe Tier-1/Tier-2 branching in prism_describe.rs (ADR-058 §G / BC-2.16.003 EC-016-013-027)"`

Do NOT disclose: the specific column names tested, the exact assertion threshold, or the five Tier-2 column names checked.

---

## Category: real-world-corpus

This scenario is grounded in the LLM agent discoverability failure documented in BC-2.16.003 EC-016-013-027: before Stage 2, `prism_describe` advertises `col.name` values as queryable field names. After Stage 2 with `ocsf_column_naming = true`, querying by col.name returns no data (because the Arrow schema uses OCSF-flattened names). An LLM agent calling `prism_describe` BEFORE the Stage 2 fix would construct valid-looking queries that silently return nothing — the phantom queryable name problem. This scenario directly validates that `prism_describe` advertises the correct queryable names after Stage 2.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome alerts TOML spec (post-KF-08/09/10 corrections); grounded in BC-2.16.003 §Interpretation A EC-016-013-027 |
| corpus_size | One sensor TOML (claroty.sensor.toml), alerts table, five Tier-2 columns checked |
| known_edge_cases | Tables with zero Tier-2 columns (all columns have ocsf_field) must not emit a raw_extensions ColumnDescriptor at all — not asserted here (alerts table always has some Tier-2 columns after KF corrections) |
| false_positive_threshold | Zero: raw_extensions ColumnDescriptor present is an unambiguous Stage 2 postcondition |
| false_negative_threshold | Zero: phantom col.name in prism_describe causes LLM agents to construct broken queries silently |

**Known-good corpus:** Claroty TOML with `ocsf_column_naming = false` (Interpretation B) — expected result: all columns emitted individually with col.name as descriptor name; NO raw_extensions ColumnDescriptor. Tests that the flag=false path is not regressed by Stage 2.

**Known-problematic corpus:** Claroty TOML with `ocsf_column_naming = true` before the Tier-2 fix (pre-Stage-2 prism_describe) — expected result (what the broken implementation would return): `category`, `alert_type_name`, `devices_count` appear as individual ColumnDescriptor names; no raw_extensions descriptor. This is the exact pre-fix behavior that Stage 2 must eliminate.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-ROUTING-001-holdout-authoring | 2026-08-21 | product-owner | Initial authoring. Story-level holdout gate for S-ADR058-OCSF-ROUTING-001 — prism_describe Tier-2 prohibition and raw_extensions ColumnDescriptor four-field shape. Covers AC-006 Tier-2, AC-007b. SINGLE-USE. |
