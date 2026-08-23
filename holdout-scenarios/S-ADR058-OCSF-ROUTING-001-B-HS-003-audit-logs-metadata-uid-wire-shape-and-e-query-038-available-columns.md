---
document_type: holdout-scenario
level: L3
id: "HS-ROUTING-001-B-003"
title: "Claroty audit_logs metadata_uid wire shape (OQ-005 KF fix) + E-QUERY-038 available_columns OCSF-mode correctness"
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
timestamp: "2026-08-23T00:00:00Z"
modified: "2026-08-23"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "8e21f85"
traces_to: "BC-2.11.016"
behavioral_contracts:
  - BC-2.11.016
  - BC-2.16.003
  - BC-2.16.002
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
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for S-ADR058-OCSF-ROUTING-001 (HS-023 group — A+W amendment coverage; HS-022 consumed at D-2270). Tests: (1) audit_logs.id OQ-005 fix wire shape: SELECT metadata_uid returns the audit record ID; (2) BC-2.11.016 EC-11-079 E-QUERY-038 available_columns OCSF-mode correctness: SELECT id triggers E-QUERY-038 with available_columns listing metadata_uid (not id). Different column than HS-022-003 (which covered class_uid, comment, activity_name). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ROUTING-001-B-003: Claroty audit_logs metadata_uid wire shape (OQ-005 KF fix) + E-QUERY-038 available_columns OCSF-mode correctness

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-ROUTING-001 (HS-023 re-gate group — A+W amendment + E-QUERY-038 available_columns coverage; HS-022 consumed at D-2270)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.11.016 EC-11-079 sub-cases (a) and (b) — E-QUERY-038 plan-gate for `ocsf_column_naming = true` tables: raw `col.name` rejected as-if-absent; `available_columns` lists OCSF-flattened names only; BC-2.16.003 §Claroty Contracted OCSF Mappings audit_logs table: `id` column `ocsf_field = "metadata.uid"` → Arrow `"metadata_uid"` (OQ-005 fix); ADR-058 §I5 / §I1 wire-shape obligation
**Gate:** Story-level holdout re-gate (HS-023) — runs after LOCAL 3-CLEAN convergence at code @8aeaf06c4, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario tests two complementary behaviors that HS-022 did NOT cover:

**1. Claroty audit_logs `metadata_uid` wire shape (OQ-005 fix):** The `audit_logs.id` column received the OQ-005 fix (human decision 2026-08-21): `id → ocsf_field = "metadata.uid"` → Arrow field `"metadata_uid"`. When `ocsf_column_naming = true` is active for Claroty, a query that selects `metadata_uid` from `claroty.audit_logs` must return a wire-level row containing `"metadata_uid"` with the audit record's raw `id` value. The raw `col.name` `"id"` must NOT appear as a top-level key in the row.

HS-022-003 covered `class_uid`, `comment`, and `activity_name` from `claroty.audit_logs`. This scenario covers the `metadata_uid` column — a DIFFERENT column on the same table, with no overlap with HS-022-003's assertions.

**2. E-QUERY-038 `available_columns` OCSF-mode correctness (BC-2.11.016 EC-11-079):** When a query references a raw `col.name` (e.g., `SELECT id FROM claroty.audit_logs`), the E-QUERY-038 plan-gate must reject it AND the error payload's `available_columns` list MUST contain ONLY OCSF-flattened names (including `"metadata_uid"` for the `id` column), NOT the raw col.name `"id"`. This tests the `available_columns` payload content — critical for LLM agent self-correction (the agent reads `available_columns` from E-QUERY-038 to retry with the correct column name).

HS-022-001 verified that `SELECT id FROM claroty.alerts` fails (second query in HS-022-001). However, HS-022-001 did NOT assert the content of `available_columns` in the E-QUERY-038 response. This scenario adds that missing assertion using the audit_logs table.

**Behavioral assertions:**

1. A minimal HTTP mock server handles `POST /api/v1/audit_log/get` returning one audit_log entry with `id = "audit-log-abc-123"`, `action = "ConfigChange"`, and `user_display_name = "Jane Doe"`.
2. prism is started in MCP stdio mode with Claroty sensor TOML (`ocsf_column_naming = true` — per AC-005 of this story; OQ-005 fix applied: `audit_logs.id` has `ocsf_field = "metadata.uid"`).
3. A `query` MCP tool call issues `SELECT metadata_uid, action FROM claroty.audit_logs`.
4. The serialized JSON response contains at least one row where:
   - The key `"metadata_uid"` is present with value `"audit-log-abc-123"` (the OQ-005 fix: `id` value mapped to OCSF Arrow field `metadata_uid`).
   - The key `"action"` is NOT expected as a top-level key — after KF corrections, `action` maps to `ocsf_field = "activity_name"` → Arrow field `"activity_name"`. Confirm whether `action` or `activity_name` is the query-surface name by reading the Claroty sensor TOML. If `action` col.name maps to `ocsf_field = "activity_name"`, query as `activity_name` (use `SELECT metadata_uid, activity_name FROM claroty.audit_logs`).
   - The serialized row does NOT contain a top-level key named `"id"` (raw col.name must not appear in the wire response for an `ocsf_column_naming = true` table).
5. A second `query` MCP tool call issues `SELECT id FROM claroty.audit_logs` (the raw col.name).
6. This second query returns an E-QUERY-038 error (not a successful result). The E-QUERY-038 error JSON payload must include an `available_columns` field whose value:
   - CONTAINS `"metadata_uid"` (the OCSF-flattened name for the `id` column, per OQ-005 fix).
   - Does NOT contain `"id"` (the raw col.name — must not appear in the OCSF-mode available set).
   - CONTAINS `"class_uid"` and `"_sensor"` (unconditional synthesized columns always present for OCSF-mode tables).

**BDD supplement:**

**Given** a minimal HTTP mock serves `POST /api/v1/audit_log/get` returning one record with `id = "audit-log-abc-123"` and `action = "ConfigChange"`
**And** prism MCP stdio is configured with Claroty `ocsf_column_naming = true` and OQ-005 fix applied (`audit_logs.id` has `ocsf_field = "metadata.uid"`)
**When** `SELECT metadata_uid FROM claroty.audit_logs` is issued via the MCP `query` tool
**Then** the serialized JSON response row contains `"metadata_uid": "audit-log-abc-123"` (OQ-005 fix: `id` value accessible as `metadata_uid`)
**And** the serialized row does NOT contain a top-level key named `"id"` (raw col.name absent post-Stage-2)
**When** `SELECT id FROM claroty.audit_logs` is issued via the MCP `query` tool
**Then** the response is an E-QUERY-038 error
**And** the E-QUERY-038 error's `available_columns` payload CONTAINS `"metadata_uid"` and `"class_uid"` and `"_sensor"`
**And** the `available_columns` payload does NOT contain `"id"` (raw col.name absent from OCSF-mode available set)

---

## Setup Instructions

1. Verify `crates/prism-sensors/specs/claroty.sensor.toml` (in the story branch) has:
   - `ocsf_column_naming = true` (set by AC-005).
   - In the `audit_logs` table, the `id` column has `ocsf_field = "metadata.uid"` (OQ-005 fix; `ocsf_field_to_arrow_name("metadata.uid") = "metadata_uid"`).
   If either condition is absent, record SETUP-FAILURE.

2. Also verify the `action` column in the `audit_logs` table. It should have `ocsf_field = "activity_name"` (KF-01 family correction). The query in assertion 3 should use the OCSF-flattened name for the action column — determine this from the TOML (it will be `ocsf_field_to_arrow_name(ocsf_field_of_action)`, which is `"activity_name"` if `ocsf_field = "activity_name"`).

3. Start a minimal HTTP mock server on `127.0.0.1:0` (ephemeral port). It must handle:
   - `POST /api/v1/audit_log/get` → return HTTP 200 with body:
     ```json
     {
       "audit_log": [{
         "id": "audit-log-abc-123",
         "action": "ConfigChange",
         "user_display_name": "Jane Doe",
         "username": "jdoe",
         "timestamp": "2026-08-23T10:00:00Z",
         "details": "Updated network policy rule 42"
       }],
       "total": 1
     }
     ```
   - Pagination requests may return `{"audit_log": [], "total": 1}`.
   Note: the exact JSON envelope structure may vary — use what the Claroty TOML `response_path = "$.audit_log"` declaration expects (the key `audit_log`, matching the `ClarotyAuditLogEntry` DTU response format).

4. Configure a test Claroty sensor pointing the mock at `http://127.0.0.1:<PORT>` with bearer_token = any non-empty string.

5. Start prism in MCP stdio mode with `RUST_LOG=warn`. Capture stderr.

6. Look up the OCSF-flattened name for the `action` column from the Claroty sensor TOML (`ocsf_field = "activity_name"` → Arrow name `"activity_name"`). Adjust the first query if needed.

7. First query: issue MCP `query` tool call with `{"sql": "SELECT metadata_uid, activity_name FROM claroty.audit_logs"}`. Capture the full serialized JSON response.

8. Second query: issue MCP `query` tool call with `{"sql": "SELECT id FROM claroty.audit_logs"}`. Capture the full response (expect E-QUERY-038 error).

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.016 | EC-11-079 sub-case (a): `SELECT id FROM claroty.audit_logs` — `id` is raw TOML col.name; E-QUERY-038 fires; `available_columns` lists OCSF-flattened names ONLY (no `"id"`) | Assertion 6: E-QUERY-038 on `SELECT id`; `available_columns` contains `"metadata_uid"` not `"id"` |
| BC-2.11.016 | EC-11-079 sub-case (b): `SELECT metadata_uid FROM claroty.audit_logs` — `metadata_uid` is the correct OCSF-flattened name; E-QUERY-038 MUST NOT fire (FP-001 preserved) | Assertion 3/4: query succeeds, row present with `"metadata_uid"` key |
| BC-2.11.016 | EC-11-079 sub-case (a) `available_columns` postcondition: MUST list OCSF-flattened names only; raw TOML col.name MUST NOT appear; `class_uid` and `_sensor` always present | Assertion 6: `available_columns` contains `"metadata_uid"`, `"class_uid"`, `"_sensor"`; NOT `"id"` |
| BC-2.16.003 | §Claroty Contracted OCSF Mappings audit_logs: `id` column `ocsf_field = "metadata.uid"` (OQ-005 fix) → Arrow field `"metadata_uid"` when `ocsf_column_naming = true` | Assertion 4: `"metadata_uid": "audit-log-abc-123"` present in wire row |
| BC-2.16.003 | §Interpretation A: Arrow Field Naming — `ocsf_field` declarations produce queryable Arrow field identifiers; raw col.name absent from wire rows for `ocsf_column_naming = true` tables | Assertion 4: `"id"` absent from serialized row |
| ADR-058 §I5 | Wire-shape assertion obligation for OQ-005 fix: `audit_logs.id` → `metadata.uid` → `metadata_uid` Arrow field; accessible in query result | End-to-end path from TOML → pipeline_result_to_record_batch → Arrow column → MCP wire |

---

## Verification Approach

1. Build the prism binary from the story branch at commit @8aeaf06c4.
2. Start the mock HTTP server and configure prism with Claroty pointing at the mock.
3. Launch prism in MCP stdio mode.
4. Send first MCP `query` tool call: `SELECT metadata_uid, activity_name FROM claroty.audit_logs`.
5. Receive the full MCP JSON response. Assert:
   - The response is valid JSON.
   - The response contains at least one row.
   - In the row, key `"metadata_uid"` is present with string value `"audit-log-abc-123"`.
   - In the row, key `"id"` is NOT a top-level column key (raw col.name absent post-Stage-2).
6. Send second MCP `query` tool call: `SELECT id FROM claroty.audit_logs`.
7. Receive the full MCP JSON response. Assert:
   - The response is an E-QUERY-038 error (not a success response, not empty rows — an error response with an error code indicating column-not-found).
   - Parse the `available_columns` field from the E-QUERY-038 error JSON.
   - Assert `"metadata_uid"` IS present in `available_columns` (the correct OCSF-flattened name for the `id` column).
   - Assert `"id"` is NOT present in `available_columns` (raw col.name excluded from OCSF-mode available set).
   - Assert `"class_uid"` IS present in `available_columns` (unconditional synthesized column).
   - Assert `"_sensor"` IS present in `available_columns` (unconditional synthesized column).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **metadata_uid wire shape present (OQ-005 fix)** (weight: 0.35): Does the first query response row contain `"metadata_uid": "audit-log-abc-123"`?
  Full credit (1.0): `"metadata_uid"` present with the correct string value.
  Partial credit (0.5): `"metadata_uid"` present but value differs (possible source_path extraction issue or wrong mock data).
  Zero credit (0.0): `"metadata_uid"` absent from the row — OQ-005 fix not applied or pipeline_result_to_record_batch still uses raw col.name `"id"`.

- **raw col.name "id" absent from wire row** (weight: 0.15): Does the first query row NOT contain `"id"` as a top-level key?
  Full credit (1.0): `"id"` is not a top-level key in any row.
  Zero credit (0.0): `"id"` appears as a top-level key — raw col.name behavior still active (pre-Stage-2 regression).

- **E-QUERY-038 fires for SELECT id** (weight: 0.20): Does the second query (`SELECT id FROM claroty.audit_logs`) return an E-QUERY-038 error?
  Full credit (1.0): E-QUERY-038 error returned.
  Zero credit (0.0): second query returns rows or empty result — raw col.name `id` is still registered as queryable.

- **available_columns contains metadata_uid, NOT id** (weight: 0.25): Does the E-QUERY-038 error's `available_columns` list contain `"metadata_uid"` AND exclude `"id"`?
  Full credit (1.0): `"metadata_uid"` present; `"id"` absent; `"class_uid"` and `"_sensor"` also present.
  Partial credit (0.5): `"metadata_uid"` present and `"id"` absent but `"class_uid"` or `"_sensor"` missing (incomplete synthesized columns in available_columns).
  Partial credit (0.3): `"metadata_uid"` absent from available_columns (OCSF name not registered in E-QUERY-038 plan-gate for audit_logs).
  Zero credit (0.0): `"id"` present in available_columns — raw col.name incorrectly included in OCSF-mode available set.

- **Row present on first query** (weight: 0.05): Is at least one row returned for the first query?
  Full credit (1.0): row present.
  Zero credit (0.0): empty result on first query despite mock returning data — pipeline failure.

---

## Edge Conditions

- **OQ-005 fix not applied (audit_logs.id has no ocsf_field):** The `id` column falls into Tier-2 (ocsf_field == None); it aggregates into `raw_extensions`. `SELECT metadata_uid FROM claroty.audit_logs` would fail with E-QUERY-038 (`metadata_uid` is not a registered Tier-1 name). Record as SETUP-FAILURE if the TOML shows the `id` column without `ocsf_field = "metadata.uid"`.

- **ocsf_column_naming not set on Claroty TOML:** All columns use raw col.name. First query returns `"id"` not `"metadata_uid"`. Second query (`SELECT id`) succeeds. Record as SETUP-FAILURE.

- **Mock returns HTTP 400 or connection error:** Record as SETUP-FAILURE. Do NOT mark as behavioral FAIL.

- **First query returns DataFusion error for metadata_uid:** If E-QUERY-038 fires for `SELECT metadata_uid`, check if OQ-005 TOML fix was applied. If the fix is present, this indicates pipeline_result_to_record_batch or the plan-gate registration failed to map `metadata.uid` to Arrow name `metadata_uid`.

- **available_columns payload absent or empty in E-QUERY-038 response:** BC-2.11.016 EC-11-079 postcondition (a) states `available_columns` MUST be present and MUST list OCSF-flattened names. If absent, record FAIL on "available_columns contains metadata_uid" dimension with observation "available_columns field absent from E-QUERY-038 JSON payload."

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ROUTING-001-B-003 (satisfaction: X.XX) — Claroty audit_logs OQ-005 wire shape or E-QUERY-038 available_columns gap; check OQ-005 TOML fix (audit_logs.id ocsf_field = metadata.uid → Arrow metadata_uid), pipeline_result_to_record_batch OQ-005 column routing, and TableRegistry available_columns for ocsf_column_naming=true tables (BC-2.11.016 EC-11-079; BC-2.16.003 §Claroty audit_logs; AC-016 Fix-A T-26)"`

Do NOT disclose: the specific audit record ID used, the exact column names in the first query, or the exact assertion threshold.

---

## Category: real-world-corpus

This scenario is grounded in two production-grade behavioral obligations:

1. **OQ-005 fix correctness (audit_logs.id → metadata_uid):** An analyst or LLM agent querying Claroty audit logs for audit record IDs must use `metadata_uid` — the OCSF-semantically-correct column name — rather than the raw API field name `id`. The OQ-005 fix overrides the initial KF-05 direction (which had removed ocsf_field from audit_logs.id entirely) based on the human decision that `metadata.uid` is the correct OCSF path for audit record IDs. A query-surface failure here means LLM agents cannot reliably identify or correlate specific audit records.

2. **E-QUERY-038 `available_columns` OCSF-mode correctness:** An LLM agent that issues a query with the wrong column name (e.g., `SELECT id FROM claroty.audit_logs`) relies on the E-QUERY-038 `available_columns` list to self-correct. If `available_columns` lists raw `col.name` values instead of OCSF-flattened names, the agent's self-correction loop fails (it retries with a name that is also invalid under the OCSF-mode schema). This is the core motivating case for BC-2.11.016 EC-11-079.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome audit_log API simulated via mock; grounded in BC-2.16.003 §Claroty Contracted OCSF Mappings audit_logs (OQ-005 fix) + BC-2.11.016 EC-11-079 available_columns postcondition |
| corpus_size | Single audit_log record; metadata_uid column tested for wire shape + E-QUERY-038 available_columns |
| known_edge_cases | `metadata_uid` is NOT covered by HS-022-003 (which covered class_uid, comment, activity_name); `available_columns` content was NOT verified by HS-022-001 (which only verified that SELECT id fails, not what available_columns contains) |
| false_positive_threshold | Zero: `"metadata_uid": "audit-log-abc-123"` in the wire row is an unambiguous OQ-005 end-to-end success signal |
| false_negative_threshold | Zero: `"id"` in available_columns means the OCSF-mode plan-gate is still operating against raw col.name entries |

**Known-good corpus:** Claroty audit_logs with `ocsf_column_naming = false` (Interpretation B / default behavior before Stage 2) — expected: `"id"` appears as the column name in query results; `SELECT id FROM claroty.audit_logs` succeeds. Tests that the flag=false path is not regressed.

**Known-problematic corpus:** Claroty audit_logs with `ocsf_column_naming = true` after Stage 2 but before OQ-005 fix (i.e., `audit_logs.id` has no `ocsf_field`) — expected: `metadata_uid` query fails E-QUERY-038; `id` value not accessible by its OCSF name; `raw_extensions` contains it. This is the pre-OQ-005-fix behavior that the OQ-005 correction resolves.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-ROUTING-001-B-HS-023-re-gate | 2026-08-23 | product-owner | Initial authoring. HS-023 re-gate group for S-ADR058-OCSF-ROUTING-001 — Claroty audit_logs metadata_uid wire shape (OQ-005) + E-QUERY-038 available_columns OCSF-mode correctness. NOT covered by consumed HS-022 group (D-2270). Different column than HS-022-003 (class_uid/comment/activity_name). AC-016 Fix-A T-26 + OQ-005 TOML correction surface. SINGLE-USE. |
