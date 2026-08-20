---
document_type: holdout-scenario
level: L3
id: "HS-COERCION-001-A-003"
title: "Integer column receives non-parseable string — null cell emitted with column_coercion_failure warn"
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
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-ADR058-OCSF-COERCION-001 — Integer column non-parseable-string null+warn (AC-007 failure path). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-COERCION-001-A-003: Integer column receives non-parseable string — null cell emitted with column_coercion_failure warn

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-COERCION-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 EC-016-013-025 (Path A `build_column_array` Integer+String parse-failure → null + column_coercion_failure; story AC-007 RG-009); BC-2.02.011 §Postconditions warning-emission obligation
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the AC-007 failure path in `build_column_array` (Path A): when a `column_type = "integer"` column receives a JSON string value that is NOT parseable as i64, the column produces a null cell AND emits a `column_coercion_failure` structured warn event.

The pre-fix behavior: `build_column_array` ColumnType::Integer arm calls `other.as_i64()` on `Value::String("many")`, which returns `None` silently. No warning is emitted. The column appears as null in the MCP output with no operator visibility into the data quality issue.

The post-fix behavior: the new `Value::String(s)` arm attempts `s.parse::<i64>()`. When it fails (e.g., `"many"` is not a valid i64), the arm returns `None` (null cell in Arrow) AND emits `tracing::warn!(event_type = "column_coercion_failure", column = ..., column_type = "integer", actual_json_kind = "string")`. The serialized MCP response shows `"devices_count": null` — key PRESENT, value null (via `with_explicit_nulls(true)`). The structured warn provides operator visibility.

**Key distinction from HS-002:** both HS-002 and HS-003 receive a JSON string for `devices_count`. The difference is parsability: `"42"` parses successfully (HS-002 produces `42`); `"many"` fails to parse (HS-003 produces `null` + warn). The distinguishing assertion between these two scenarios is: does a parse-failure scenario produce a warn while a parse-success scenario does not?

**Behavioral assertions:**

1. A minimal HTTP mock server is started that handles `POST /api/v1/alerts` and returns exactly one alert record where `devices_count` is the JSON string `"many"` (not parseable as i64). All other fields are valid.
2. prism is started in MCP stdio mode with the Claroty sensor configured to point at the mock.
3. A `query` MCP tool call is issued: `SELECT id, devices_count FROM claroty.alerts`.
4. The serialized JSON response is inspected at the byte level.
5. The `devices_count` column is **`null`** in the response — key PRESENT, value null (not absent key).
6. A `column_coercion_failure` structured warn event is emitted with `column_type = "integer"` and `actual_json_kind = "string"`.
7. The row for the alert IS present in the response — the record is NOT dropped due to the coercion failure.

**BDD supplement:**

**Given** a minimal HTTP mock serves `POST /api/v1/alerts` returning one alert where `devices_count = "many"` (non-parseable JSON string)
**And** prism MCP stdio is configured with the Claroty sensor pointing at the mock
**When** `SELECT id, devices_count FROM claroty.alerts` is issued via the MCP `query` tool
**Then** the serialized JSON response row contains `"devices_count": null` (key present, value null — NOT absent)
**And** a `column_coercion_failure` tracing warn is emitted with `column_type = "integer"` and `actual_json_kind = "string"`
**And** the record (id=9003) is present in the response (not dropped)

---

## Setup Instructions

1. Start a minimal HTTP mock server on `127.0.0.1:0` (ephemeral port). It must handle:
   - `POST /api/v1/alerts` → return HTTP 200 with body:
     ```json
     {
       "alerts": [{
         "id": 9003,
         "alert_type_name": "Test Alert",
         "category": "OT",
         "status": "open",
         "detected_time": "2026-08-19T10:00:00Z",
         "updated_time": "2026-08-19T10:01:00Z",
         "devices_count": "many",
         "description": "Normal string description",
         "alert_class": "OT",
         "ot_devices_count": 1,
         "alert_name": "Parse Failure Test"
       }],
       "total": 1,
       "page": 1
     }
     ```
     Note: `"devices_count": "many"` is a JSON string that fails i64 parsing.
   - Subsequent pagination calls may return `{"alerts": [], "total": 1, "page": 2}`.

2. Configure a test Claroty sensor TOML pointing at the mock (base_url = `http://127.0.0.1:<PORT>`, bearer_token = any non-empty string).

3. Start prism in MCP stdio mode with `RUST_LOG=prism_bin=warn,prism_spec_engine=warn` to capture warn-level structured log output. Capture stderr.

4. Issue MCP `query` tool call with `{"sql": "SELECT id, devices_count FROM claroty.alerts"}`.

5. Capture the full serialized JSON response and stderr log output.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-025: Path A `build_column_array` ColumnType::Integer + `Value::String("many")` → parse fails → `None` + warn | Core assertion: null cell + warn emitted |
| BC-2.16.003 | §Invariants "Coercion failures are non-fatal: record is NEVER dropped due to type mismatch" | Record id=9003 is present in the response (not dropped) |
| BC-2.16.003 | §Coercion Warning Observability DEFECT (closed by AC-004 / AC-007): warn MUST be emitted at demotion point | column_coercion_failure warn observed in logs |
| BC-2.02.011 | §Postconditions "A warning-level log entry is emitted for each normalization issue" | column_coercion_failure warn emitted for the parse failure |
| BC-2.11.001 | EC-11-079 null-not-absent wire shape | null cell appears as `"devices_count": null` (key present), not as absent key |

---

## Verification Approach

1. Build the prism binary.
2. Start the mock HTTP server as specified in §Setup Instructions. Capture the bound port.
3. Write a test TOML config pointing at the mock.
4. Launch prism in MCP stdio mode, capturing stderr.
5. Send the MCP `query` tool call: `{"sql": "SELECT id, devices_count FROM claroty.alerts"}`.
6. Receive the full MCP JSON response. Assert:
   - The response is valid JSON.
   - The row for id=9003 IS present in the response (record not dropped).
   - Locate the `devices_count` column in the row (use `prism_describe` to resolve the exact field name if needed).
   - Assert: the value IS the JSON null literal (`null`) — NOT absent, NOT the string `"many"`, NOT an integer.
   - Assert the null key is present: the serialized JSON bytes contain a `"devices_count"` key followed by `null`, not simply the absence of the key. This distinction is critical (BC-2.11.001 EC-11-079).
7. In the captured stderr, locate a structured log event where:
   - `event_type = "column_coercion_failure"` (or equivalent field)
   - `column_type = "integer"` (or equivalent field)
   - `actual_json_kind = "string"` (or equivalent field)
8. If the null key is absent (key not present in the serialized row), record as FAIL with observation "null key absent — with_explicit_nulls(true) chokepoint may not be active on this path."
9. If no warn event is found for `devices_count`, record as FAIL with observation "column_coercion_failure warn not emitted for parse-failure case — AC-007 / BC-2.02.011 obligation not fulfilled."

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **Null-cell wire shape with key present** (weight: 0.45): Does `devices_count` appear as `null` (key present, value null) in the serialized JSON row?
  Full credit (1.0): key present, value is JSON null.
  Zero credit (0.0): key absent (null-not-absent violation) OR value is the string `"many"` OR value is an integer.

- **column_coercion_failure warn emitted** (weight: 0.35): Was a warn event with `event_type = "column_coercion_failure"`, `column_type = "integer"`, `actual_json_kind = "string"` emitted?
  Full credit (1.0): event found in log output.
  Zero credit (0.0): no such event found (silent failure — AC-007 / BC-2.02.011 not implemented).

- **Record presence** (weight: 0.20): Is the row for id=9003 present (not dropped)?
  Full credit (1.0): row present.
  Zero credit (0.0): row absent (record dropped — BC-2.16.003 §Invariants violated).

---

## Edge Conditions

- **String `""`** (empty string): `"".parse::<i64>()` fails → null + warn. Same behavior as `"many"`.

- **String `"3.14"`** (float): `"3.14".parse::<i64>()` fails → null + warn. Float strings are NOT valid i64.

- **Mock returns HTTP 400 or connection error:** Record as SETUP-FAILURE.

- **Both HS-002 and HS-003 run in sequence:** The evaluator may choose to run both against different mock endpoints. The critical distinguisher: `"42"` → integer 42 (no warn); `"many"` → null + warn. The column name and column_type in the warn event must match.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-COERCION-001-A-003 (satisfaction: X.XX) — Integer column parse-failure did not produce null+warn; either the null key is absent (wire-shape violation) or the column_coercion_failure event was not emitted; check the Value::String failure branch and tracing emission in build_column_array ColumnType::Integer arm"`

Do NOT disclose: the specific non-parseable string used, the column name, or the exact assertion threshold.

---

## Category: real-world-corpus

Grounded in EC-016-013-025 (BC-2.16.003) and BC-2.02.011 §Postconditions: real MSSP sensors may return non-numeric strings for fields declared as `column_type = "integer"`. Before the AC-007 fix, prism silently discarded these values without any operator visibility. This scenario verifies that the post-fix behavior both produces a correct null cell AND emits the structured warn that operators use for data quality diagnosis.

| Field | Description |
|-------|-------------|
| corpus_source | Simulated via a minimal mock server; grounded in EC-016-013-025 known gap and BC-2.02.011 DEFECT section of BC-2.16.003 |
| corpus_size | Single alert record; one `devices_count` Integer column with non-parseable string input |
| known_edge_cases | Empty string, float strings, hex strings all fail i64 parse and produce null + warn |
| false_positive_threshold | Zero: a null + warn for a non-parseable string is the correct coercion-failure behavior |
| false_negative_threshold | Zero: a silent null without a warn (the pre-fix behavior) is a clear BC-2.02.011 violation |

**Known-good corpus:** Mock with `devices_count = 3` (integer) — expected result: `devices_count = 3`, no coercion warn. Tests that valid integer input is not affected.

**Known-problematic corpus:** Mock with `devices_count = "many"` (non-parseable string) — expected result: `devices_count = null` (key present), `column_coercion_failure` warn emitted.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-COERCION-001-holdout-authoring | 2026-08-19 | product-owner | Initial authoring. Story-level holdout gate for S-ADR058-OCSF-COERCION-001 AC-007 failure path — Integer column non-parseable-string null+warn. SINGLE-USE. |
