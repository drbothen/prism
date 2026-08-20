---
document_type: holdout-scenario
level: L3
id: "HS-COERCION-001-A-004"
title: "String column Array input serializes as JSON-list string — ENRICH-1 arm not displaced by Object fix"
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
notes: "HIDDEN, SINGLE-USE story-level holdout gate for S-ADR058-OCSF-COERCION-001 — ENRICH-1 Array arm non-regression after AC-005 Object arm insertion (AC-006 scope). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-COERCION-001-A-004: String column Array input serializes as JSON-list string — ENRICH-1 arm not displaced by Object fix

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-COERCION-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 EC-016-013-026 (Path A `build_column_array` String + Array → JSON-list string; ENRICH-1 arm preserved after AC-005); BC-2.16.003 §Full Coercion Matrix row "String / Array / any / Path A: CORRECT, ENRICH-1 preserved"
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates a critical non-regression property of AC-005: the addition of an explicit `Value::Object(_) => None` arm in `build_column_array`'s String branch MUST NOT displace or modify the existing `serde_json::Value::Array(arr)` arm that correctly serializes arrays to compact JSON-list strings (ENRICH-1 Design Decision 2, EC-016-013-026).

The subtlety: the AC-005 fix inserts an Object arm into the match statement. A misimplementation could accidentally position this arm to also catch Array values (e.g., using a too-broad pattern like `Value::Object(_) | Value::Array(_) => None`), or could inadvertently change the match arm ordering so the Array arm no longer fires before a broader pattern.

The required match arm order after AC-005 is:
```
Null | String(s) | Array(arr) | Object(_) [new, null-demotion] | other => [wildcard]
```

If the Array arm is displaced or broadened to null-demote, this scenario catches it immediately: ENRICH-1 wildcard columns (`ip_list`, `mac_list`, `network_list`, `vlan_list` on Claroty devices) return arrays that must serialize as compact JSON-list strings — not null cells.

**Behavioral assertions:**

1. A minimal HTTP mock server is started that handles `POST /api/v1/devices` and returns exactly one device record where `ip_list` contains a JSON Array of IP address strings.
2. prism is started in MCP stdio mode with the Claroty sensor configured to point at the mock server.
3. A `query` MCP tool call is issued: `SELECT uid, ip_list FROM claroty.devices` (or `SELECT * FROM claroty.devices LIMIT 1` if `ip_list` is in `raw_extensions` under the current schema).
4. The serialized JSON response is inspected at the byte level.
5. The `ip_list` value (either as a first-class column or as a key inside the `raw_extensions` JSON blob) is a JSON string containing a compact JSON array representation — NOT null, NOT absent, NOT a raw JSON array type.
6. No `column_coercion_failure` log warn event is emitted for `ip_list` (Array input for a String column is CORRECT behavior per ENRICH-1 — it is not a coercion failure).

**BDD supplement:**

**Given** a minimal HTTP mock serves `POST /api/v1/devices` returning one device where `ip_list = ["192.168.1.1", "10.0.0.2"]` (a JSON Array of strings)
**And** prism MCP stdio is configured with the Claroty sensor pointing at the mock
**When** `SELECT uid, ip_list FROM claroty.devices` is issued via the MCP `query` tool (or equivalent to access ip_list data)
**Then** the `ip_list` value in the response is a JSON string containing a compact array representation (e.g., `"[\"192.168.1.1\",\"10.0.0.2\"]"`) — NOT null, NOT a raw JSON array
**And** no `column_coercion_failure` tracing event is emitted for `ip_list`

---

## Setup Instructions

1. Start a minimal HTTP mock server on `127.0.0.1:0` (ephemeral port). It must handle:
   - `POST /api/v1/devices` → return HTTP 200 with body:
     ```json
     {
       "devices": [{
         "uid": "dev-enrich1-test",
         "asset_id": "asset-001",
         "device_category": "OT Device",
         "device_type": "PLC",
         "risk_score": "High",
         "retired": false,
         "ip_list": ["192.168.1.1", "10.0.0.2"],
         "mac_list": ["AA:BB:CC:DD:EE:FF"],
         "device_name": "PLC-001",
         "os_category": "VxWorks"
       }],
       "total": 1,
       "page": 1
     }
     ```
   - Subsequent pagination calls may return `{"devices": [], "total": 1, "page": 2}`.

2. Configure a test Claroty sensor TOML pointing at the mock (base_url = `http://127.0.0.1:<PORT>`, bearer_token = any non-empty string).

3. Start prism in MCP stdio mode with `RUST_LOG=prism_bin=warn,prism_spec_engine=warn` to capture warn-level logs. Capture stderr.

4. Use `prism_describe claroty.devices` (via the `describe` MCP tool) to determine whether `ip_list` appears as a first-class column or inside the `raw_extensions` blob in the current schema. This informs how to assert on its value.

5. Issue the appropriate MCP `query` tool call to retrieve the device row and its `ip_list` data:
   - If `ip_list` is a first-class column: `SELECT uid, ip_list FROM claroty.devices`
   - If `ip_list` is in `raw_extensions`: `SELECT uid, raw_extensions FROM claroty.devices`, then inspect the `ip_list` key inside the `raw_extensions` JSON string.

6. Capture the full serialized JSON response and stderr log output.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-026: Path A String + `Value::Array` arm serializes as compact JSON-list string; MUST NOT be changed to null-demotion | Core assertion: JSON-list string present, not null |
| BC-2.16.003 | §Full Coercion Matrix row "String / Array / any / Path A: CORRECT, ENRICH-1 preserved (EC-016-013-026)" | Non-regression: ENRICH-1 arm behavior unchanged after AC-005 Object arm insertion |
| BC-2.16.003 | §Invariants "The declared `column_type` is the authoritative wire shape" | String column with Array input → JSON-list string (correct ENRICH-1 behavior) |
| BC-2.16.003 | AC-005 story constraint "existing `serde_json::Value::Array(arr)` arm... MUST NOT be modified" | Arm ordering preserved; Object arm insertion did not displace Array arm |

---

## Verification Approach

1. Build the prism binary.
2. Start the mock HTTP server as specified in §Setup Instructions. Capture the bound port.
3. Write a test TOML config pointing at the mock.
4. Launch prism in MCP stdio mode, capturing stderr.
5. Use `describe` MCP tool to check `claroty.devices` schema: confirm the schema loads without error and note whether `ip_list` is a first-class column or part of `raw_extensions`.
6. Issue the appropriate `query` MCP tool call to retrieve the device record.
7. Receive the full MCP JSON response. Assert:
   - The response is valid JSON.
   - The row for uid=`dev-enrich1-test` is present.
   - Locate the `ip_list` value:
     - If first-class column: find it directly in the row.
     - If in `raw_extensions`: parse the `raw_extensions` JSON string and access the `"ip_list"` key.
   - Assert: the `ip_list` value IS a JSON string (not JSON null, not a raw JSON array object type, not absent).
   - Assert: when the string value is parsed as JSON, it yields an array: `["192.168.1.1", "10.0.0.2"]`.
   - Assert: the serialized bytes do NOT contain `null` as the `ip_list` value.
8. In the captured stderr, confirm NO `column_coercion_failure` event was emitted for `ip_list`. Array→JSON-list-string is CORRECT behavior, not a coercion failure.
9. If `ip_list` is null or absent in the response, record as FAIL with observation "ENRICH-1 Array arm displaced — AC-005 Object arm insertion incorrectly null-demoted Array input."

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **JSON-list string value** (weight: 0.55): Is the `ip_list` value a JSON string whose content parses as a JSON array?
  Full credit (1.0): value is a JSON string; parsing it as JSON yields an array containing `"192.168.1.1"` and `"10.0.0.2"`.
  Zero credit (0.0): value is null, absent, or a raw JSON array object type (not a string).

- **No spurious coercion warning** (weight: 0.30): Is there NO `column_coercion_failure` event for `ip_list`?
  Full credit (1.0): no such event found.
  Zero credit (0.0): a `column_coercion_failure` event IS emitted for `ip_list` — Array→JSON-list-string incorrectly treated as a coercion failure.

- **Schema loads and record present** (weight: 0.15): Does the schema load without error and the device row appear in the response?
  Full credit (1.0): schema loaded, row present.
  Zero credit (0.0): schema error or row absent.

---

## Edge Conditions

- **Empty array `[]`**: `ip_list = []` → expected `ip_list` value is the string `"[]"` (not null; per EC-016-013-026 "Empty array → `[]` (empty JSON-list string, NOT null)").

- **Single-element array**: `["192.168.1.1"]` → expected JSON-list string `"[\"192.168.1.1\"]"`.

- **Array of integers (vlan_list pattern)**: `vlan_list = [100, 200]` → expected JSON-list string `"[\"100\",\"200\"]"` (integers stringified per EC-016-013-026 "Integer/bool array elements are stringified via `other.to_string()`").

- **Mock returns HTTP 400 or connection error:** Record as SETUP-FAILURE.

- **`ip_list` not in schema:** If `prism_describe` shows that `ip_list` does not appear at all (neither as a column nor as a key described by `raw_extensions`), this is a schema discoverability gap (separate from the coercion issue) — record as SETUP-FAILURE with observation "ip_list not discoverable in schema; cannot assert wire-level value."

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-COERCION-001-A-004 (satisfaction: X.XX) — Array input for a String column produced null instead of a JSON-list string; the ENRICH-1 Array arm in build_column_array was displaced or modified by the recent changes; verify match arm ordering in spec_driven_adapter.rs"`

Do NOT disclose: the specific column name, the specific IP addresses used in the fixture, or the exact assertion threshold.

---

## Category: real-world-corpus

Grounded in EC-016-013-026 (BC-2.16.003) and the ENRICH-1 Design Decision 2: Claroty devices API returns array-valued fields (`ip_list`, `mac_list`, `network_list`, `vlan_list`) that must be serialized as compact JSON-list strings for LLM agent consumption. This scenario verifies that the AC-005 Object-arm insertion in `build_column_array` did not break this existing behavior.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty devices API behavior simulated via a minimal mock; grounded in EC-016-013-026 and ENRICH-1 Design Decision 2 of BC-2.16.003 |
| corpus_size | Single device record; one `ip_list` String column with Array input |
| known_edge_cases | Empty array → `"[]"` (not null); single-element array → compact JSON string; integer-element array → stringified elements |
| false_positive_threshold | Zero: a JSON-list string for an array-valued ENRICH-1 column is always correct |
| false_negative_threshold | Zero: a null for an array-valued ENRICH-1 column after the AC-005 change is a clear regression of correct existing behavior |

**Known-good corpus:** Mock with `ip_list = ["192.168.1.1", "10.0.0.2"]` (array, the normal ENRICH-1 case) — expected result: `ip_list` value is a JSON-list string. Tests that AC-005 did not regress ENRICH-1.

**Known-problematic corpus (regression detection):** A hypothetical buggy implementation where the Object arm is declared as `Value::Object(_) | Value::Array(_) => None` — would produce `ip_list = null` instead of a JSON-list string. This scenario catches that bug immediately.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-COERCION-001-holdout-authoring | 2026-08-19 | product-owner | Initial authoring. Story-level holdout gate for S-ADR058-OCSF-COERCION-001 — ENRICH-1 Array arm non-regression after AC-005 Object arm insertion. SINGLE-USE. |
