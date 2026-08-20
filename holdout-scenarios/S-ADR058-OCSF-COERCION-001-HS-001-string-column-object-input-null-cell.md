---
document_type: holdout-scenario
level: L3
id: "HS-COERCION-001-A-001"
title: "String column receives JSON Object input — null cell emitted, not stringified object"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-OCSF-ROUTING"
story_source: "S-ADR058-OCSF-COERCION-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-19T00:00:00Z"
modified: "2026-08-19"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.011-normalization-error-handling.md"
input-hash: "a5fc868"
traces_to: "BC-2.16.003"
behavioral_contracts:
  - BC-2.16.003
  - BC-2.02.011
verification_properties: []
lifecycle_status: active
introduced: "S-ADR058-OCSF-COERCION-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-ADR058-OCSF-COERCION-001 — String column Object-input null-demotion (AC-005 Path A). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-COERCION-001-A-001: String column receives JSON Object input — null cell emitted, not stringified object

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-COERCION-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 §Type Coercion Algorithm Rule 1 EC-016-013-008 (String + Object → null cell + column_coercion_failure); BC-2.02.011 §Postconditions warning-emission obligation
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the AC-005 fix in `build_column_array` (Path A, sole live production path) correctly null-demotes a JSON Object value received for a `column_type = "string"` column, instead of stringifying it via the `other.to_string()` wildcard arm (the pre-fix behavior).

The pre-fix behavior: `build_column_array` receives `Value::Object({"summary": "...", "confidence": 0.95})` for the `description` column (declared `column_type = "string"`) and falls through to the `other => Some(other.to_string())` wildcard, producing a stringified JSON object in the Arrow StringArray — a corrupt value. The serialized MCP response therefore shows `"description": "{\"summary\":\"compromised PLC detected\",\"confidence\":0.95}"`.

The post-fix behavior: an explicit `Value::Object(_) => None` arm (with `column_coercion_failure` warn emission) fires before the wildcard, returning a null Arrow cell. The serialized MCP response shows `"description": null` — key PRESENT, value null. The downstream `with_explicit_nulls(true)` serializer path (BC-2.11.001 EC-11-079) ensures the null key is present, not absent.

**Behavioral assertions:**

1. A minimal HTTP mock server is started that handles `POST /api/v1/alerts` and returns exactly one alert record where the `description` field is a JSON Object (not a string). All other fields are valid per the Claroty alert schema.
2. prism is started in MCP stdio mode with the Claroty sensor configured to point at the mock server.
3. A `query` MCP tool call is issued: `SELECT id, description FROM claroty.alerts`.
4. The serialized JSON response is inspected at the byte level.
5. The response row's `description` field (or its Arrow/OCSF-mapped equivalent column name) is **`null`** — key present, value null — in the JSON output.
6. The response row does NOT contain a stringified object value for that column (no `{\"summary\"` substring in the value string, no quote-wrapped JSON object).
7. A structured log warn event with `event_type = "column_coercion_failure"`, `column_type = "string"`, and `actual_json_kind = "object"` is emitted by the prism binary.

**BDD supplement:**

**Given** a minimal HTTP mock serves `POST /api/v1/alerts` returning one alert where `description = {"summary": "compromised PLC detected", "confidence": 0.95}` (a JSON Object, not a string)
**And** prism MCP stdio is configured with the Claroty sensor pointing at the mock
**When** `SELECT id, description FROM claroty.alerts` is issued via the MCP `query` tool
**Then** the serialized JSON response row contains `"description": null` (key present, value null — not absent)
**And** the serialized JSON row does NOT contain any stringified representation of the Object (no substring match on `{\"summary\"`)
**And** a `column_coercion_failure` tracing warn is emitted with `column_type = "string"` and `actual_json_kind = "object"`

---

## Setup Instructions

1. Start a minimal HTTP mock server on `127.0.0.1:0` (ephemeral port). It must handle:
   - `POST /api/v1/alerts` → return HTTP 200 with body:
     ```json
     {
       "alerts": [{
         "id": 9001,
         "alert_type_name": "Test Alert",
         "category": "OT",
         "status": "open",
         "detected_time": "2026-08-19T10:00:00Z",
         "updated_time": "2026-08-19T10:01:00Z",
         "devices_count": 1,
         "description": {"summary": "compromised PLC detected", "confidence": 0.95},
         "alert_class": "OT",
         "ot_devices_count": 1,
         "alert_name": "PLC Compromise"
       }],
       "total": 1,
       "page": 1
     }
     ```
   - Handle any `POST /api/v1/alerts` call (for pagination; subsequent pages may return `{"alerts": [], "total": 1, "page": 2}`).

2. Configure a test Claroty sensor TOML pointing at the mock (use base_url = `http://127.0.0.1:<PORT>`, bearer_token = any non-empty string).

3. Start prism in MCP stdio mode with `RUST_LOG=prism_bin=warn,prism_spec_engine=warn` (or equivalent) to capture warn-level structured log output. Capture stderr.

4. Issue MCP `query` tool call with `{"sql": "SELECT id, description FROM claroty.alerts"}`.

5. Capture the full serialized JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | §Type Coercion Algorithm Rule 1 EC-016-013-008: String + Object → `Err(CoercionWarning)` on Path B; null cell + warn on Path A | Core assertion: null cell present (not stringified object) |
| BC-2.16.003 | §Full Coercion Matrix row "String / Object / any / Path A: WRONG pre-fix → contracted: None + warn (AC-005)" | AC-005 Path A fix observed at wire level |
| BC-2.16.003 | §Invariants "Coercion failures are non-fatal: field value preserved in raw_extensions; record is NEVER dropped" | Record is returned (not missing); only the column is null |
| BC-2.02.011 | §Postconditions "A warning-level log entry is emitted for each normalization issue" | column_coercion_failure warn event emitted |
| BC-2.11.001 | EC-11-079 null-not-absent wire shape | null cell appears as `"description": null` (key present), not as absent key |

---

## Verification Approach

1. Build the prism binary (`cargo build --release -p prism-bin` or `just build`).
2. Start the mock HTTP server as specified in §Setup Instructions. Capture the bound port.
3. Write a test TOML config pointing at the mock (base_url = `http://127.0.0.1:<PORT>`).
4. Launch prism in MCP stdio mode against the test TOML, capturing stderr for log output.
5. Send the MCP `query` tool call: `{"sql": "SELECT id, description FROM claroty.alerts"}`.
6. Receive the full MCP JSON response. Assert:
   - The response is valid JSON (parse without error).
   - The response contains at least one row.
   - In the row for alert id=9001, locate the column corresponding to `description` (use `prism_describe claroty.alerts` first if needed to resolve the Arrow field name).
   - Assert: the column value IS the JSON null literal — not a string, not absent.
   - Assert: the raw serialized bytes do NOT contain the substring `{\"summary\"` anywhere in the row.
7. In the captured stderr, check for a structured log line where `event_type = "column_coercion_failure"`, `column_type = "string"`, `actual_json_kind = "object"` (field names may appear in any order in the structured log output).
8. If the record is entirely missing from the response (no row for id=9001), record as FAIL with observation "record was dropped — coercion failure caused record loss, violating BC-2.16.003 §Invariants."

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **Null-cell wire shape** (weight: 0.50): Does the `description` column appear as `null` (key present) in the serialized JSON row?
  Full credit (1.0): column is JSON null, key present.
  Zero credit (0.0): column is a stringified object, OR key is absent, OR record is missing.

- **No-stringification** (weight: 0.25): Does the response NOT contain a stringified representation of the JSON Object?
  Full credit (1.0): no `{\"summary\"` or similar substring in the column value.
  Zero credit (0.0): stringified JSON object found in column value.

- **column_coercion_failure event** (weight: 0.15): Was a warn event with `event_type = "column_coercion_failure"`, `column_type = "string"`, `actual_json_kind = "object"` emitted?
  Full credit (1.0): event found in log output.
  Zero credit (0.0): no such event found.

- **Record presence** (weight: 0.10): Is the record present (not dropped) in the response?
  Full credit (1.0): the row for id=9001 is present in the result.
  Zero credit (0.0): record is absent — coercion caused record loss.

---

## Edge Conditions

- **Multiple alerts with mix of valid and Object descriptions:** If the mock returns two alerts — one with a normal string description and one with an Object description — the evaluator should assert: the normal-string row has its description as a string value; the Object-description row has description as null. Both rows must be present.

- **Mock returns HTTP 400 or connection error:** Record as SETUP-FAILURE. Do NOT mark as behavioral FAIL.

- **Prism cannot find the `description` column in the schema:** Use `prism_describe claroty.alerts` to identify the actual Arrow field name for this column before issuing the SELECT. If the schema exposes no column for `description`, record as SETUP-FAILURE with observation "column missing from schema — evaluator cannot assert wire-level value."

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-COERCION-001-A-001 (satisfaction: X.XX) — String column with unexpected JSON Object input was not null-demoted; check build_column_array String arm for the Object/wildcard ordering in spec_driven_adapter.rs"`

Do NOT disclose: the specific column name used, the Object value content, the exact assertion threshold, or the fixture JSON structure.

---

## Category: real-world-corpus

This scenario is grounded in the EC-016-013-008 production gap documented in BC-2.16.003: real MSSP sensor APIs (including Claroty) occasionally return nested JSON Objects for fields that the TOML spec declares as `column_type = "string"`. Before the AC-005 fix, prism silently stringified these objects and placed the corrupt string value in the OCSF field. This scenario verifies that the fix produces the correct null-cell output.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome alerts API behavior simulated via a minimal mock server; grounded in EC-016-013-008 known gap documented in BC-2.16.003 |
| corpus_size | Single alert record; one `description` column with Object input |
| known_edge_cases | Empty Object `{}` → still null-demoted (AC-005 handles all Object inputs, per story EC-001) |
| false_positive_threshold | Zero: a null cell for an Object input is a clear behavioral improvement |
| false_negative_threshold | Zero: a stringified object in a typed string column is a clear defect |

**Known-good corpus:** Mock with `description = "Malware activity on PLC device"` (normal string) — expected result: description appears as a string value in the output. Tests that the fix does not over-demote valid string inputs.

**Known-problematic corpus:** Mock with `description = {"summary": "compromised PLC detected", "confidence": 0.95}` (JSON Object) — expected result: description is null in the output. Tests that the Object arm fires correctly.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-COERCION-001-holdout-authoring | 2026-08-19 | product-owner | Initial authoring. Story-level holdout gate for S-ADR058-OCSF-COERCION-001 AC-005 — String column Object-input null-demotion. SINGLE-USE. |
