---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-005"
title: "Query sensor_errors Wire Format — xDome 403 Entry: HTTP status + sanitized body, no doubled prefix"
category: "edge-case-combinations"
must_pass: true
priority: P0
epic_id: "engine-defects"
story_source: "DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
version: "1.0"
status: consumed
producer: product-owner
timestamp: "2026-08-14T00:00:00Z"
phase: 3
inputs:
  - stories/DEFECT-ADAPTER-TLS-XDOME-LIVE-001-live-xdome-https-fails-waf-h1-no-ua.md
input-hash: "babbc13"
traces_to: "BC-2.11.001"
behavioral_contracts:
  - BC-2.11.001
  - BC-2.10.007
verification_properties: []
lifecycle_status: consumed
introduced: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
last_evaluated: "2026-08-14"
last_eval_satisfaction: "PASS"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (3-CLEAN converged @ a5b61b35b). Tests D-2151 decision (b): per-target HTTP status + sanitized body in sensor_errors wire entries. No prior scenario covered this surface. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-005: Query sensor_errors Wire Format — xDome 403 Entry: HTTP status + sanitized body, no doubled prefix

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story re-gate)
**BC Traced:** BC-2.11.001 (query MCP tool `sensor_errors` per-target HTTP detail), BC-2.10.007 (Rule 1 count-only aggregate Display unchanged)
**Gate:** Story-level holdout re-gate — runs after LOCAL 3-CLEAN @ a5b61b35b. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario exercises D-2151 decision (b): when an xDome sensor target returns a non-2xx HTTP
response during a `query` tool call, the `sensor_errors` array in the wire response must carry a
per-target entry in the format `"<table>: HTTP <status>: <sanitized-body>"` — not the old
error-code-only form (`"sensor error (E-SENSOR-XXX)"`) and not a doubled prefix (`"HTTP HTTP 403"`)
that would occur if the format string were applied twice or if the body already contained `"HTTP "`.

The DTU is configured to return HTTP 403 with a short ASCII body for the xDome devices table.
The evaluator asserts the exact wire format of the `sensor_errors` entry at the byte level.

**BDD supplement:**

**Given** the Claroty/xDome DTU is running and configured to return HTTP 403 with body `"Access Denied"` for the `xdome_devices` table endpoint  
**When** the `query` tool is called with `query="FROM xdome_devices"` targeting that xDome sensor  
**Then** the serialized MCP JSON response contains `"sensor_errors"` as a non-empty array with one entry matching `"xdome_devices: HTTP 403: Access Denied"`, and no entry containing the pattern `"sensor error (E-SENSOR-"` or `"HTTP HTTP"`

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | Per-target HTTP detail in `sensor_errors` (D-2151): `"<table>: HTTP <status>: <body-snippet>"` format | Wire-level: `sensor_errors[0]` == `"xdome_devices: HTTP 403: Access Denied"` |
| BC-2.11.001 | `sensor_errors` serialization invariants: non-null, non-empty array, non-empty string elements | Wire assertions on array shape |
| BC-2.10.007 | Rule 1: aggregate AllTargetsFailed Display string remains COUNT-ONLY; per-target detail NOT in aggregate | Negative assertion: the old `"sensor error (E-SENSOR-XXX)"` form is absent |

---

## Verification Approach

1. Start the Claroty DTU clone (`prism-dtu-claroty`): `BehavioralClone::start_on("127.0.0.1:0", ...)` and capture the bound port.

2. Configure the DTU so the `xdome_devices` table endpoint returns HTTP 403 with response body `"Access Denied"` (short, all-ASCII, no control characters).

3. Start the prism binary in MCP stdio mode with the Claroty/xDome sensor configured:
   - `base_url` pointing at the DTU loopback address
   - Bearer token set to any placeholder value (the 403 will be returned regardless of auth — the DTU is configured to return 403 for this table)
   - AD-017: no real credential value is used

4. Over MCP stdio, send a `query` tool call with `query="FROM xdome_devices"` and any required client/org scoping parameter.

5. Capture the complete serialized JSON response byte string from prism's MCP stdout.

6. Assert on the raw byte string:

   **MUST be true:**
   - `contains("\"sensor_errors\"")` — the key is present
   - `contains("\"xdome_devices: HTTP 403: Access Denied\"")` — exact entry format
   - The `sensor_errors` value, when deserialized, is a non-empty JSON array
   - Each element of `sensor_errors` is a non-empty string

   **MUST NOT be true:**
   - `contains("sensor error (E-SENSOR-")` — old error-code-only form must be absent
   - `contains("HTTP HTTP")` — doubled prefix is a formatting defect
   - `sensor_errors` value is `null` or `[]`

7. Verify the `rows` field: because ALL targets failed, `rows` should be empty (`[]`) or absent.

---

## Edge Conditions

- **Exact format match:** The entry must be `"xdome_devices: HTTP 403: Access Denied"` — not `"xdome_devices: HTTP 403 Access Denied"` (missing colon separator after status code) and not `"xdome_devices: 403: Access Denied"` (missing `HTTP` keyword). The colon separators and keyword are part of the contract.

- **Body sanitization boundary:** The test body `"Access Denied"` is short and all-ASCII, so `sanitize_body_snippet_bytes` truncation at 256 bytes is not exercised here. The body must appear verbatim in the entry. Control-character stripping is also not triggered since the body has none.

- **sensor_errors absent on no-failure path:** If the DTU is accidentally misconfigured and returns 200, `sensor_errors` should be absent entirely from the response. A `sensor_errors: null` or `sensor_errors: []` on a successful query is itself a defect (checked by MUST NOT in step 6).

- **AllTargetsFailed vs success:** When all targets for a single-client query fail, the entire query returns a response with `sensor_errors` and an empty `rows`. The aggregate error Display (E-SENSOR-030) must NOT appear in `sensor_errors` — only per-target entries.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Wire format correctness** (weight: 0.5): Does `sensor_errors[0]` equal `"xdome_devices: HTTP 403: Access Denied"` verbatim? Full credit (1.0): exact match. Partial credit (0.5): format partially correct (e.g., status present but body missing). Zero credit: old `"sensor error (E-SENSOR-XXX)"` form, or doubled `"HTTP HTTP"` prefix.

- **Array shape invariants** (weight: 0.3): Is `sensor_errors` a non-null, non-empty array of non-empty strings? Full credit: correct shape. Zero credit: null, `[]`, or element is null/empty-string.

- **Negative assertions** (weight: 0.15): Are the MUST-NOT forms absent? Full credit: both absent. Partial (0.5): one absent, one present. Zero credit: both present.

- **rows empty on all-fail** (weight: 0.05): Is `rows` empty (`[]`) or absent when all targets failed? Full credit: empty/absent. Zero credit: non-empty `rows` on a total-failure query.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-005 (satisfaction: X.XX) — query sensor_errors entry has wrong format for a non-2xx xDome response; verify the per-target HTTP status+body format is applied in the error surfacing path"`

Do NOT disclose: the specific HTTP status code tested, the exact body string, the table name, or which assertion failed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API (`api.claroty.com`) — production behavior of HTTP 403 from WAF/auth layer; simulated via `prism-dtu-claroty` in this evaluation |
| known_edge_cases | 403 with ASCII body; format contract (HTTP keyword, colon separators, no doubled prefix) |
| false_positive_threshold | Zero: format regression that reverts to `sensor error (E-SENSOR-XXX)` loses all diagnostic value for the LLM agent consumer |
| false_negative_threshold | Low: a correctly-formatted entry with wrong status code would still surface actionable info, but is a separate contract violation |

**Known-good corpus:** Same query succeeding (DTU returns 200 with records) — `sensor_errors` should be ABSENT entirely. Tests the false-alarm direction.

**Known-problematic corpus:** DTU returning HTTP 403 with body (this scenario) — `sensor_errors` must carry the HTTP format entry.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-pass38-readj-holdout | 2026-08-14 | product-owner | Initial authoring. Re-gate scenario for D-2151 decision (b) sensor_errors per-target HTTP wire format. SINGLE-USE. No prior consumed scenario covered this surface. |
