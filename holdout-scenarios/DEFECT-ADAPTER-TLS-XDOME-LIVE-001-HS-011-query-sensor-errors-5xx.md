---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-011"
title: "Query sensor_errors for 5xx Response: xdome_devices entry contains HTTP 500 status in per-target format"
category: "edge-case-combinations"
must_pass: true
priority: P1
epic_id: "engine-defects"
story_source: "DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
version: "1.0"
status: draft
producer: product-owner
timestamp: "2026-08-15T00:00:00Z"
phase: 3
inputs:
  - stories/DEFECT-ADAPTER-TLS-XDOME-LIVE-001-live-xdome-https-fails-waf-h1-no-ua.md
input-hash: "babbc13"
traces_to: "BC-2.11.001"
behavioral_contracts:
  - BC-2.11.001
  - BC-2.08.002
verification_properties: []
lifecycle_status: active
introduced: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (3-CLEAN converged @ 8dd8d4285, post-HS-007-fix). Tests query path (not health path) for 5xx: FROM xdome_devices against a 500-returning sensor → sensor_errors entry with HTTP 500 in per-target format, rows empty. Complements HS-008 (health path) by exercising the query path. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-011: Query sensor_errors for 5xx Response: xdome_devices entry contains HTTP 500 status in per-target format

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P1 — blocking but lower weight than P0 scenarios)
**BC Traced:** BC-2.11.001 (per-target HTTP detail in sensor_errors), BC-2.08.002 (5xx classification on query path)
**Gate:** Story-level holdout re-gate — runs after LOCAL 3-CLEAN @ 8dd8d4285. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

HS-008 verified the health-check path for 5xx (via `check_sensor_health`). This scenario exercises
the parallel query path: when `FROM xdome_devices` is executed against a sensor that returns HTTP
500 from the DTU, the `query` tool response MUST surface the error in `sensor_errors` using the
per-target HTTP format introduced by D-2151 decision (b). The `rows` field must be empty (no data
returned from the failing sensor).

This scenario cross-checks that the 5xx handling is consistent between the health path and the
query path — a 5xx during a query is a data-fetch failure, and the error MUST appear in
`sensor_errors` rather than being swallowed silently (which would produce `rows: []` with no
diagnostic signal).

**BDD supplement:**

**Given** the Claroty/xDome DTU is running at a loopback address and configured to return HTTP 500
for requests to the devices endpoint  
**When** the `query` tool is called with `query="FROM xdome_devices"` targeting that sensor  
**Then** the serialized MCP JSON response contains a `"sensor_errors"` key with a non-empty array,
where at least one entry contains `"HTTP 500"` as part of the per-target error format, and `"rows"`
is empty or absent

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | Per-target HTTP detail in `sensor_errors` (D-2151): `"<table>: HTTP <status>: <body-snippet>"` | sensor_errors entry contains "HTTP 500" and "xdome_devices" |
| BC-2.11.001 | `sensor_errors` is non-empty when all targets return non-2xx | sensor_errors is a non-null, non-empty array |
| BC-2.11.001 | `rows` is empty when all targets fail | rows == [] or absent |
| BC-2.08.002 | 5xx on query path surfaces the failure (not swallowed silently) | sensor_errors non-empty, not silent Vec::new() return |

---

## Verification Approach

1. Start the Claroty DTU clone (`prism-dtu-claroty`): `BehavioralClone::start_on("127.0.0.1:0", ...)` configured to return HTTP 500 with body `{"error": "internal server error (injected)", "code": 500}` for requests to the devices endpoint. Capture the bound port.

2. Start the prism binary in MCP stdio mode with the Claroty/xDome sensor configured:
   - `base_url` pointing at the DTU loopback address
   - Bearer token set to any placeholder value (the 500 is returned regardless of auth)
   - AD-017: no real credential value is used (e.g., `PRISM_CLAROTY_API_TOKEN=hs011-placeholder-NOTREAL`)

3. Over MCP stdio, send a `query` tool call with `query="FROM xdome_devices"`.

4. Capture the complete serialized JSON response byte string from prism's MCP stdout.

5. Assert on the raw byte string:

   **MUST be true:**
   - `contains("\"sensor_errors\"")` — the key is present
   - The `sensor_errors` value, when deserialized, is a non-empty array
   - At least one element of `sensor_errors` contains `"HTTP 500"` as a substring — the 500 status appears in the per-target format
   - At least one element of `sensor_errors` contains `"xdome_devices"` — the table name is present

   **MUST NOT be true:**
   - `sensor_errors` value is `null` or `[]`
   - `contains("sensor error (E-SENSOR-")` — old error-code-only form must be absent
   - `rows` contains any actual device records (all targets failed → rows must be empty)

6. Optionally verify: the per-target format includes the body snippet. The expected format from D-2151 decision (b) is `"xdome_devices: HTTP 500: <body>"`. The body is `{"error": "internal server error (injected)", "code": 500}` — some or all of this should appear in the entry. If the body snippet truncation or JSON-within-JSON escaping makes exact assertion difficult, assert only on `"HTTP 500"` and `"xdome_devices"` presence.

7. Verify `rows` is empty: `rows` deserialized == `[]` or `rows` key absent.

---

## Edge Conditions

- **5xx on query path vs health path:** The health path (`check_sensor_health`) classifies 5xx as Degraded and surfaces it via `reachable:true` + `error:"service_unavailable"`. The query path (`query`) surfaces it via `sensor_errors`. Both surfaces must handle 5xx non-silently. This scenario targets the query surface.

- **JSON body in sensor_errors entry:** The DTU 500 body is valid JSON. The `sanitize_body_snippet_bytes` function strips control characters but does not parse JSON. The resulting entry may contain JSON-escaped content. The evaluator asserts on `"HTTP 500"` substring presence rather than exact body format to avoid false failures due to escaping behavior.

- **MCP tool call must succeed:** The `query` tool call must return a tool result (not an MCP JSON-RPC error). A tool-level error for sensor failure is a separate defect from sensor_errors population.

- **sensor_errors absent on happy path:** If the DTU is accidentally misconfigured to return 200, `sensor_errors` should be absent. A present-but-empty `sensor_errors:[]` on a successful query is itself a defect.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **sensor_errors non-empty with HTTP 500** (weight: 0.45): Does `sensor_errors` contain at least one entry with `"HTTP 500"` as a substring? Full credit (1.0): yes. Partial (0.5): sensor_errors present but lacks "HTTP 500" or has wrong format. Zero credit: `sensor_errors` null/`[]` (5xx swallowed silently).

- **table name in entry** (weight: 0.25): Does the sensor_errors entry contain `"xdome_devices"` as a substring? Full credit: yes. Partial (0.4): entry present but table name absent. Zero credit: entry absent.

- **rows empty** (weight: 0.20): Is `rows` == `[]` or absent? Full credit: empty/absent. Zero credit: non-empty (data fabricated or from a wrong source despite 500).

- **old format absent** (weight: 0.10): Is `"sensor error (E-SENSOR-"` absent from the response? Full credit: absent. Zero credit: present (D-2151 per-target format regression).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-011 (satisfaction: X.XX) — query returned incorrect sensor_errors for a server-side error response on the xDome table; verify the per-target HTTP format is applied on the 5xx query path and errors are not swallowed silently"`

Do NOT disclose: the specific HTTP status code, the DTU body content, the table name queried, or which assertion failed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API — HTTP 500 during a query fetch (backend overload, internal error); MSSP analyst needs per-target error details to route remediation; simulated via `prism-dtu-claroty` |
| known_edge_cases | JSON body in sensor_errors entry (escaping behavior); query path vs health path 5xx handling consistency |
| false_positive_threshold | Zero: `sensor_errors:[]` for a 500-failing sensor silently drops the error, giving the LLM agent no diagnostic signal |
| false_negative_threshold | Low: correct error present but with minor format variation still provides actionable signal |

**Known-good corpus:** Same query against DTU in 200-success mode with seeded records — expected: `sensor_errors` ABSENT, `rows` non-empty. Tests the false-alarm direction.

**Known-problematic corpus:** DTU returning HTTP 500 (this scenario) — expected: `sensor_errors` non-empty with HTTP 500 entry, `rows` empty.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-pass50-reauth-holdout | 2026-08-15 | product-owner | Initial authoring. Query-path 5xx sensor_errors gate, complementing HS-008 (health path). SINGLE-USE. |
