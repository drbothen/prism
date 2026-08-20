---
document_type: holdout-scenario
level: L3
id: "HS-COERCION-001-A-002"
title: "Integer column receives parseable string — correct integer materialized, no silent data loss"
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
input-hash: "593878a"
traces_to: "BC-2.16.003"
behavioral_contracts:
  - BC-2.16.003
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
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-ADR058-OCSF-COERCION-001 — Integer column parseable-string no-loss (AC-007 happy path). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-COERCION-001-A-002: Integer column receives parseable string — correct integer materialized, no silent data loss

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-COERCION-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 EC-016-013-025 (Path A `build_column_array` Integer+String parse-success returns correct integer; story AC-007 RG-008)
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the AC-007 happy path in `build_column_array` (Path A): when a `column_type = "integer"` column receives a JSON string value that is parseable as i64, the string-encoded integer is correctly materialized in the Arrow Int64Array — not silently dropped to null.

The pre-fix behavior: `build_column_array` ColumnType::Integer arm calls `other.as_i64()` on `Value::String("42")`, which returns `None` (serde_json's `as_i64()` method on a String variant always returns None). This silently drops valid integer data — the column appears as null in the MCP output even though the sensor returned a valid numeric string.

The post-fix behavior: a new `Value::String(s)` arm precedes the wildcard in the Integer block. When `s.parse::<i64>()` succeeds (e.g., `"42"` parses to 42), the arm returns `Some(42)` — the correct integer value is materialized in the Arrow Int64Array. The serialized MCP response shows `"devices_count": 42` (an integer), not null.

This validates prevention of the silent data loss class identified as EC-016-013-025 and story AC-007.

**Behavioral assertions:**

1. A minimal HTTP mock server is started that handles `POST /api/v1/alerts` and returns exactly one alert record where the `devices_count` field is the JSON string `"42"` (not the integer 42). All other fields are valid per the Claroty alert schema.
2. prism is started in MCP stdio mode with the Claroty sensor configured to point at the mock server.
3. A `query` MCP tool call is issued: `SELECT id, devices_count FROM claroty.alerts`.
4. The serialized JSON response is inspected at the byte level.
5. The response row's `devices_count` column contains the integer value `42` — not null, not the string `"42"`.
6. No `column_coercion_failure` log warn event is emitted for the `devices_count` column (successful parse is NOT an error).

**BDD supplement:**

**Given** a minimal HTTP mock serves `POST /api/v1/alerts` returning one alert where `devices_count = "42"` (a JSON string, not an integer)
**And** prism MCP stdio is configured with the Claroty sensor pointing at the mock
**When** `SELECT id, devices_count FROM claroty.alerts` is issued via the MCP `query` tool
**Then** the serialized JSON response row contains `"devices_count": 42` (integer value 42, not null, not string `"42"`)
**And** no `column_coercion_failure` tracing event is emitted for `devices_count`

---

## Setup Instructions

1. Start a minimal HTTP mock server on `127.0.0.1:0` (ephemeral port). It must handle:
   - `POST /api/v1/alerts` → return HTTP 200 with body:
     ```json
     {
       "alerts": [{
         "id": 9002,
         "alert_type_name": "Test Alert",
         "category": "OT",
         "status": "open",
         "detected_time": "2026-08-19T10:00:00Z",
         "updated_time": "2026-08-19T10:01:00Z",
         "devices_count": "42",
         "description": "Normal string description",
         "alert_class": "OT",
         "ot_devices_count": 1,
         "alert_name": "Coercion Test"
       }],
       "total": 1,
       "page": 1
     }
     ```
     Note: `"devices_count": "42"` is a JSON string, not an integer.
   - Subsequent pagination calls may return `{"alerts": [], "total": 1, "page": 2}`.

2. Configure a test Claroty sensor TOML pointing at the mock (base_url = `http://127.0.0.1:<PORT>`, bearer_token = any non-empty string).

3. Start prism in MCP stdio mode with `RUST_LOG=prism_bin=warn,prism_spec_engine=warn` to capture warn-level logs. Capture stderr.

4. Issue MCP `query` tool call with `{"sql": "SELECT id, devices_count FROM claroty.alerts"}`.

5. Capture the full serialized JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-025: Path A `build_column_array` ColumnType::Integer + `Value::String("42")` → parse succeeds → `Some(42)` (no data loss) | Core assertion: integer value 42 present, not null |
| BC-2.16.003 | §Full Coercion Matrix row "Integer / String / non-numeric suffix / Rule 3 gap (pre-fix: null) → contracted: parse-attempt (AC-007)" | AC-007 Path A parse-success observed at wire level |
| BC-2.16.003 | §Invariants "The declared `column_type` is the authoritative wire shape" | Integer column receives parseable string → integer in output |

---

## Verification Approach

1. Build the prism binary.
2. Start the mock HTTP server as specified in §Setup Instructions. Capture the bound port.
3. Write a test TOML config pointing at the mock.
4. Launch prism in MCP stdio mode, capturing stderr.
5. Send the MCP `query` tool call: `{"sql": "SELECT id, devices_count FROM claroty.alerts"}`.
6. Receive the full MCP JSON response. Assert:
   - The response is valid JSON.
   - The response contains at least one row.
   - In the row for alert id=9002, locate the `devices_count` column (use `prism_describe claroty.alerts` first if needed to resolve the exact Arrow field name).
   - Assert: the value IS the integer 42 — NOT null, NOT the JSON string `"42"`.
   - Assert: the value type in the JSON is a number (not a string, not null).
7. In the captured stderr, check that NO `column_coercion_failure` event was emitted for `devices_count`. Successful parse is not an error.
8. (Regression guard) Also check that `id = 9002` appears as an integer or the correct type — confirming the overall pipeline produced the row correctly.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **Correct integer materialization** (weight: 0.60): Does `devices_count` appear as integer 42 (not null, not string) in the serialized JSON row?
  Full credit (1.0): value is the integer 42.
  Zero credit (0.0): value is null OR value is the string `"42"` OR field is absent.

- **No spurious coercion warning** (weight: 0.25): Is there NO `column_coercion_failure` event in logs for `devices_count`?
  Full credit (1.0): no such event found.
  Zero credit (0.0): a `column_coercion_failure` event IS emitted for `devices_count` (false positive — successful parse should not warn).

- **Record presence** (weight: 0.15): Is the row for id=9002 present in the response?
  Full credit (1.0): row present.
  Zero credit (0.0): row absent.

---

## Edge Conditions

- **String `"0"`:** If the mock returns `"0"` instead of `"42"`, expected behavior is `devices_count = 0` (integer zero, not null). This validates that zero-value parse success is handled correctly.

- **String `"-1"` (negative number):** Expected behavior is `devices_count = -1` (negative integer, since i64 is signed).

- **Mock returns HTTP 400 or connection error:** Record as SETUP-FAILURE.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-COERCION-001-A-002 (satisfaction: X.XX) — Integer column with string-encoded integer input produced null instead of the correct integer; check the Value::String arm in the ColumnType::Integer block of build_column_array"`

Do NOT disclose: the specific column name, the string value used, or the exact assertion threshold.

---

## Category: real-world-corpus

Grounded in EC-016-013-025 (BC-2.16.003): the pre-fix behavior of `build_column_array` ColumnType::Integer arm silently discards valid string-encoded integers via `other.as_i64()` returning None. Real MSSP sensors (including Claroty and similar APIs) sometimes return numeric values as JSON strings, especially in paginated or filtered API responses. This scenario verifies that the post-fix behavior materializes the correct integer.

| Field | Description |
|-------|-------------|
| corpus_source | Simulated via a minimal mock server; grounded in EC-016-013-025 known gap in BC-2.16.003 |
| corpus_size | Single alert record; one `devices_count` Integer column with string input |
| known_edge_cases | Parseable zero, negative, and large i64-range integers should all return correct integers |
| false_positive_threshold | Zero: a correct integer in the output is unambiguous |
| false_negative_threshold | Zero: a null in an integer column for valid numeric input is a clear silent data loss |

**Known-good corpus:** Mock with `devices_count = 3` (integer, the normal API shape) — expected result: `devices_count = 3` in output. Tests that the normal integer path is not regressed.

**Known-problematic corpus:** Mock with `devices_count = "42"` (string) — expected result: `devices_count = 42` (integer) in output. Tests that string-encoded integers are now correctly parsed.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-COERCION-001-holdout-authoring | 2026-08-19 | product-owner | Initial authoring. Story-level holdout gate for S-ADR058-OCSF-COERCION-001 AC-007 happy path — Integer column parseable-string no data loss. SINGLE-USE. |
