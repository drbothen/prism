---
document_type: holdout-scenario
level: L3
id: "HS-ROUTING-001-A-003"
title: "Claroty audit_logs query returns class_uid 3004 (entity_management) at wire level — KF-01 end-to-end"
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
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-ADR058-OCSF-ROUTING-001 — KF-01 end-to-end: audit_logs class_uid = 3004 (entity_management) and note→comment mapping at wire level. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ROUTING-001-A-003: Claroty audit_logs query returns class_uid 3004 (entity_management) at wire level — KF-01 end-to-end

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-ROUTING-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 EC-016-013-023 (Claroty audit_logs RecordBatch with ocsf_class = "entity_management" carries class_uid = 3004 in Arrow); ADR-058 §I5 wire-shape obligation; BC-2.16.003 §Claroty Contracted OCSF Mappings (audit_logs table note → comment under entity_management 3004)
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the KF-01 fix end-to-end at the wire level: after Stage 2, querying `SELECT class_uid, comment FROM claroty.audit_logs` returns `class_uid = 3004` (entity_management) and the OCSF field `comment` contains the value from the vendor field `note`. Three distinct failures would produce wrong results:

- **class_uid = 3001 (account_change):** The pre-KF-01 bug — `ocsf_class = "audit_activity"` was in the TOML, which fell to `.unwrap_or(0)` in the old code because `"audit_activity"` is not an OCSF v1.7.0 class. The OLD TOML mapped it to `CLASS_UID_ACCOUNT_CHANGE` via an incorrectly-named arm. After KF-01 correction, `ocsf_class = "entity_management"` must map to 3004.

- **class_uid = 0 (BASE_EVENT fallback):** The TOML was corrected to `ocsf_class = "entity_management"` but the `select_by_class_name` arm for `"entity_management"` was not added to `class_selector.rs`. The string falls to `Err(...)` → `.unwrap_or(0)`.

- **comment = null or absent:** Under `account_change` (class_uid 3001), the `comment` attribute does not exist in the protobuf descriptor for that class. On Path A (Arrow materialization), `ocsf_field = "comment"` maps to Arrow field `"comment"` — but the Arrow value comes from the source sensor data via source_path extraction, not from the protobuf descriptor. So `comment` field should contain the `note` value regardless of class_uid. If `comment` is null or absent, that indicates the contracted `note → comment` mapping was not applied (e.g., the OCSF mapping was not applied in pipeline_result_to_record_batch).

The scenario also tests the `ocsf.unknown_class_name` WARN behavior is NOT emitted when the class resolves correctly (no spurious warn for a known entity_management class).

**Behavioral assertions:**

1. A minimal HTTP mock server handles the Claroty audit_logs endpoint (verify the exact endpoint path from `crates/prism-sensors/specs/claroty.sensor.toml` audit_logs table — typically `POST /api/v1/audit-logs` or as declared in the TOML) returning one audit_log record where `action = "Login"`, `note = "reviewed"`, `username = "jsmith"`, `user_display_name = "John Smith"`.
2. prism is started in MCP stdio mode with the Claroty TOML (`ocsf_column_naming = true`, `ocsf_class = "entity_management"` for audit_logs).
3. A `query` MCP tool call issues `SELECT class_uid, comment, activity_name FROM claroty.audit_logs`.
4. The serialized JSON response contains at least one row where:
   - The key `"class_uid"` is present with integer value `3004` (entity_management class_uid)
   - The key `"comment"` is present with value `"reviewed"` (note → comment mapping under entity_management)
   - The key `"activity_name"` is present with value `"Login"` (action → activity_name mapping)
5. The `class_uid` value is NOT `0` (BASE_EVENT fallback — would indicate "entity_management" arm missing from select_by_class_name).
6. The `class_uid` value is NOT `3001` (account_change — would indicate old pre-KF-01 mapping still active).
7. A structured WARN log event with `event_type = "ocsf.unknown_class_name"` is NOT emitted (entity_management is now a known class; no unknown-class warn should fire).

**BDD supplement:**

**Given** a minimal HTTP mock serves the Claroty audit_logs endpoint returning one record where `action = "Login"`, `note = "reviewed"`
**And** prism MCP stdio is configured with Claroty `ocsf_column_naming = true` and `ocsf_class = "entity_management"` for audit_logs (KF-01 TOML correction applied)
**When** `SELECT class_uid, comment, activity_name FROM claroty.audit_logs` is issued via the MCP `query` tool
**Then** the serialized JSON response row contains `"class_uid": 3004` (entity_management class — not 0 or 3001)
**And** the serialized JSON response row contains `"comment": "reviewed"` (note → comment OCSF mapping under entity_management)
**And** the serialized JSON response row contains `"activity_name": "Login"` (action → activity_name mapping)
**And** NO `ocsf.unknown_class_name` WARN log event is emitted for the entity_management class

---

## Setup Instructions

1. Look up the audit_logs table endpoint in `crates/prism-sensors/specs/claroty.sensor.toml`. Find the `[[tables]]` block where `table_name = "audit_logs"` and identify the `path` or `url` field. The mock server must handle this exact path.

2. Start a minimal HTTP mock server on `127.0.0.1:0` (ephemeral port). It must handle the audit_logs endpoint (typically `POST /api/v1/audit-logs` — verify from the TOML) → return HTTP 200 with body:
   ```json
   {
     "audit_logs": [{
       "id": "al-555",
       "action": "Login",
       "user_display_name": "John Smith",
       "category": "Authentication",
       "timestamp": "2026-08-21T09:00:00Z",
       "details": "User login from 10.0.0.50",
       "username": "jsmith",
       "note": "reviewed"
     }],
     "total": 1,
     "page": 1
   }
   ```
   Note: the exact JSON structure of the Claroty audit_logs API response may vary — use whatever structure the Claroty TOML `source_path` expressions reference. The key field names are `action`, `note`, and `username`; confirm against the TOML.

3. Confirm `crates/prism-sensors/specs/claroty.sensor.toml` has `ocsf_column_naming = true` (AC-005) and `ocsf_class = "entity_management"` for the audit_logs table (KF-01 correction). If either is absent, record SETUP-FAILURE.

4. Configure a test Claroty sensor pointing the mock at `http://127.0.0.1:<PORT>` with bearer_token = any non-empty string.

5. Start prism in MCP stdio mode with `RUST_LOG=prism_bin=warn,prism_spec_engine=warn`. Capture stderr for log output.

6. Issue MCP `query` tool call with `{"sql": "SELECT class_uid, comment, activity_name FROM claroty.audit_logs"}`. Capture the full serialized JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-023: Claroty audit_logs RecordBatch with ocsf_class = "entity_management" carries class_uid = 3004 (entity_management) in Arrow Int32 column at wire level | class_uid == 3004 in serialized JSON |
| BC-2.16.003 | §Claroty Contracted OCSF Mappings audit_logs: note → ocsf_field = "comment" → Arrow "comment" under entity_management 3004 | comment = "reviewed" in serialized JSON |
| BC-2.16.003 | §Claroty Contracted OCSF Mappings audit_logs: action → ocsf_field = "activity_name" → Arrow "activity_name" | activity_name = "Login" in serialized JSON |
| ADR-058 §I5 | Wire-shape assertion obligation: class_uid value must be asserted at RecordBatch / serialized column level, not only at resolver unit-test string level | end-to-end path from TOML → select_by_class_name → Arrow column → MCP response |
| BC-2.16.002 | §Canonical Structured Event Catalog ocsf.unknown_class_name: WARN emitted ONLY on Err branch of select_by_class_name — NOT emitted for known classes | No ocsf.unknown_class_name WARN for entity_management |

---

## Verification Approach

1. Build the prism binary (`cargo build --release -p prism-bin` or `just build`).
2. Start the mock HTTP server as specified in §Setup Instructions. Capture the bound port.
3. Configure prism with Claroty sensor pointing at the mock at `http://127.0.0.1:<PORT>`.
4. Launch prism in MCP stdio mode, capturing stderr.
5. Send MCP `query` tool call: `{"sql": "SELECT class_uid, comment, activity_name FROM claroty.audit_logs"}`.
6. Receive the full MCP JSON response. Assert:
   - The response is valid JSON.
   - The response contains at least one row.
   - In the row, key `"class_uid"` is present with integer value `3004`. Check: the JSON value must be the integer 3004, NOT the string "3004", NOT null, NOT 0, NOT 3001.
   - In the row, key `"comment"` is present with value `"reviewed"` (the `note` field value from the audit_log record).
   - In the row, key `"activity_name"` is present with value `"Login"` (the `action` field value).
7. In the captured stderr, check that NO line contains `event_type = "ocsf.unknown_class_name"`. If such a line IS found, record as FAIL on the non-spurious-warn dimension with observation "unknown_class_name warn emitted despite entity_management being a registered class."
8. Verify class_uid is NOT 0: if `"class_uid": 0` is in the row, record as FAIL with observation "class_uid = 0 (BASE_EVENT) — entity_management arm missing from select_by_class_name."
9. Verify class_uid is NOT 3001: if `"class_uid": 3001` is in the row, record as FAIL with observation "class_uid = 3001 (account_change) — KF-01 TOML correction (entity_management) not applied or old arm still active."

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **class_uid = 3004 (entity_management)** (weight: 0.50): Is `class_uid = 3004` in the serialized JSON row?
  Full credit (1.0): class_uid is integer 3004.
  Zero credit (0.0): class_uid is 0 (BASE_EVENT), 3001 (account_change), absent, or non-integer 3004.

- **comment = "reviewed" (note → comment mapping)** (weight: 0.25): Is `comment = "reviewed"` in the row?
  Full credit (1.0): comment present with value "reviewed".
  Partial credit (0.3): comment present but null or wrong value — indicates ocsf_field mapping applied but wrong source_path or wrong data.
  Zero credit (0.0): comment absent from the row (mapping not applied).

- **activity_name = "Login" (action mapping)** (weight: 0.15): Is `activity_name = "Login"` in the row?
  Full credit (1.0): activity_name present with value "Login".
  Zero credit (0.0): activity_name absent or wrong value.

- **No spurious ocsf.unknown_class_name WARN** (weight: 0.10): Is there NO `ocsf.unknown_class_name` WARN event in stderr for the entity_management class?
  Full credit (1.0): no such warn emitted.
  Zero credit (0.0): warn emitted for entity_management — it is incorrectly classified as unknown even though the arm was added.

---

## Edge Conditions

- **KF-01 TOML correction missing (ocsf_class still "audit_activity" or "account_change"):** The select_by_class_name call falls to `.unwrap_or(0)` → class_uid = 0. Record as FAIL with observation "class_uid = 0 — KF-01 TOML correction not applied; ocsf_class is not 'entity_management'."

- **entity_management arm added to select_by_class_name but TOML not corrected:** class_uid still 0 or 3001 (depending on what is in the TOML). Same FAIL as above.

- **note → comment mapping correct but class_uid wrong:** Both the comment mapping AND the class_uid are independent aspects of the implementation. A partial result (comment correct, class_uid wrong) scores partial credit on the rubric.

- **Mock returns HTTP 400 or connection error:** Record as SETUP-FAILURE. Do NOT mark as behavioral FAIL.

- **audit_logs endpoint path differs from assumed:** The evaluator must check the TOML for the exact path. If the mock is set up with the wrong path and returns 404, record as SETUP-FAILURE.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ROUTING-001-A-003 (satisfaction: X.XX) — Claroty audit_logs class_uid not 3004 at wire level; check class_selector.rs entity_management arm addition (AC-009) and KF-01 TOML correction (ocsf_class = entity_management)"`

Do NOT disclose: the specific audit_log field values used, the exact assertion threshold, or the fixture JSON structure.

---

## Category: real-world-corpus

This scenario is grounded in the data-loss risk documented in BC-2.16.003 EC-016-013-023: under the old `account_change` (class_uid 3001) class, the `comment` attribute does not exist in the OCSF entity_management protobuf descriptor for account_change, causing the `note → comment` mapping to silently drop all note values. Under the corrected `entity_management` (class_uid 3004), the `comment` attribute exists and the mapping succeeds. This scenario validates the end-to-end fix at the wire level — a query-visible defect that 5,483 unit tests and adversarial cascades might miss because it requires the full TOML + class_selector + pipeline_result_to_record_batch pipeline to observe.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome audit_logs API simulated via mock; grounded in BC-2.16.003 EC-016-013-023 + §Claroty Contracted OCSF Mappings (audit_logs) |
| corpus_size | Single audit_log record; class_uid, comment, and activity_name columns exercised |
| known_edge_cases | class_uid = 0 (BASE_EVENT) is the most common wrong value when entity_management arm is missing; class_uid = 3001 (account_change) indicates TOML not corrected |
| false_positive_threshold | Zero: class_uid = 3004 at the MCP query wire level is an unambiguous KF-01 end-to-end success signal |
| false_negative_threshold | Zero: class_uid ≠ 3004 at wire level is an unambiguous defect regardless of unit test results |

**Known-good corpus:** Claroty TOML with `ocsf_class = "detection_finding"` for the alerts table — `class_uid = 2004` expected (alerts already had a correct class mapping). Tests that existing correct class mappings are not regressed by the entity_management arm addition.

**Known-problematic corpus:** Claroty TOML with `ocsf_class = "audit_activity"` (pre-KF-01 string, now dead code) — class_uid = 0 expected (BASE_EVENT fallback, ocsf.unknown_class_name WARN emitted). Tests that the unknown-class fallback path works correctly when the TOML uses an unrecognized class string.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-ROUTING-001-holdout-authoring | 2026-08-21 | product-owner | Initial authoring. Story-level holdout gate for S-ADR058-OCSF-ROUTING-001 — KF-01 end-to-end: audit_logs class_uid = 3004, note→comment mapping, no spurious unknown-class warn. Covers AC-009 sub-obligation (b) + BC-2.16.003 EC-016-013-023. SINGLE-USE. |
