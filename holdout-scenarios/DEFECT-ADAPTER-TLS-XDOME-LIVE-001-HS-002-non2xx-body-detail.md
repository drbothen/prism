---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-002"
title: "Non-2xx Response Body Snippet in HttpRequestFailed Detail — Wire-Level Assertion"
category: "edge-case-combinations"
must_pass: true
priority: P0
epic_id: "engine-defects"
story_source: "DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
version: "1.0"
status: consumed
producer: product-owner
timestamp: "2026-08-12T00:00:00Z"
phase: 3
inputs:
  - stories/DEFECT-ADAPTER-TLS-XDOME-LIVE-001-live-xdome-https-fails-waf-h1-no-ua.md
input-hash: "abada5b"
traces_to: "BC-2.16.002"
behavioral_contracts:
  - BC-2.16.002
  - BC-2.08.002
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
notes: "HIDDEN, SINGLE-USE story-level holdout gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — Non-2xx body capture in error detail. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-002: Non-2xx Response Body Snippet in HttpRequestFailed Detail — Wire-Level Assertion

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.002 (Non-2xx Response Body Capture postcondition)
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario exercises the non-2xx response body capture path through the full MCP stdio
pipeline. When the sensor API returns a non-success HTTP status with a non-empty body, the
prism pipeline must capture a snippet of that body (up to 256 bytes, sanitized) and surface
it in the `detail` field of the error that propagates to the MCP response.

The prior behavior: `detail` contained only the HTTP status code (e.g., `"HTTP 403"`), discarding
the response body. An operator receiving a 403 with a WAF policy message in the body (as with
the xDome/Claroty edge) had no diagnostic information about WHY the request was blocked.
The fix captures and includes the body snippet alongside the status code.

**Behavioral assertions:**

1. The sensor is configured to hit a mock HTTP server (or a specifically seeded DTU route)
   that returns HTTP 403 with a short, known body string
2. The evaluator invokes `list_sensor_data` (or equivalent data-fetching MCP tool) for the
   Claroty sensor
3. The mock server returns HTTP 403 with body `"access_denied_by_security_policy"` (a 33-byte
   string chosen to be under the 256-byte cap and free of control characters)
4. The serialized MCP error response bytes contain BOTH the HTTP status code (`"403"`) AND
   a substring of the body (`"access_denied"`)
5. The body snippet does not exceed 256 bytes in the response (truncation check)
6. A secondary body-read failure does NOT discard the primary HTTP status from `detail`

**BDD supplement:**

**Given** the Claroty sensor backend returns HTTP 403 with body `"access_denied_by_security_policy"`  
**When** a data-fetch MCP tool call is issued for the Claroty sensor  
**Then** the serialized MCP error response contains both `"403"` and `"access_denied"` in the
error detail field, as the exact bytes an LLM agent would consume

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.002 | Non-2xx Response Body Capture postcondition: `HttpRequestFailed.detail` MUST include HTTP status AND ≤256-byte sanitized body snippet | Core assertion: 403 body appears in detail alongside status code |
| BC-2.08.002 | HTTP Error Classification postcondition: `status_code > 0` maps to `SensorError::HttpError` | 403 flows through HttpError, not Internal — prerequisite for body surfacing |

---

## Verification Approach

1. Start a minimal mock HTTP server on localhost that returns HTTP 403 with body
   `"access_denied_by_security_policy"` for any inbound request. Options:
   - A one-line Python server: `python3 -c "import http.server; ..."`
   - `wiremock` already available in the workspace
   - Or configure the Claroty DTU to return 403 for a specific route if the DTU supports
     response override via fixture; fall back to a separate mock server if not
   Capture the bound address (e.g., `127.0.0.1:PORT`)

2. Start the prism binary in MCP stdio mode with the Claroty sensor `base_url` pointing
   to `http://127.0.0.1:PORT` (the mock server address). Use any syntactically valid but
   non-real bearer token (AD-017: no real credentials).

3. Over MCP stdio, invoke a data-fetching tool call: `list_sensor_data` for the Claroty sensor,
   `devices` table (or any table that triggers a fetch pipeline execution)

4. Capture the complete MCP JSON error response bytes

5. Assert on the raw byte string:
   - `contains("403")`
   - `contains("access_denied")` (a substring of the mock body that is not the status code itself)
   - The `detail` or error `message` field does NOT contain more than 256 bytes of body text
     (verify total length of the body snippet in the field)

6. **Truncation sub-assertion:** Repeat steps 1–5 with a mock server returning a 400-byte
   body (`"A" * 400`). Assert that the `detail` field contains `"403"` AND a body snippet
   of ≤ 256 bytes (the body does not appear untruncated). A body snippet of exactly 256 'A'
   characters or fewer is a PASS; 400 'A' characters is a FAIL.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; weighted average. Satisfying threshold: ≥ 0.75.

- **Functional correctness** (weight: 0.5): Does the serialized MCP error contain both the
  HTTP status (`"403"`) AND a non-empty body snippet (`"access_denied"`)?
  Full credit: both present. Half credit: status present, body absent (pre-fix behavior).
  Zero credit: neither status nor body in detail, or the response is a success response.

- **Truncation enforcement** (weight: 0.2): For the 400-byte body sub-assertion, is the
  captured snippet ≤ 256 bytes? Full credit: truncated correctly. Zero credit: full 400-byte
  body appears in detail.

- **Data integrity** (weight: 0.2): Is the MCP response valid JSON? Does the detail field
  exist as a string? Are there no serialization panics?

- **Fallback correctness** (weight: 0.1): If the body-read succeeds but body is empty,
  does detail fall back gracefully to `"HTTP 403"` (status only)? Test by repeating with
  a 403 + empty body; full credit if `"403"` still appears and no panic.

---

## Edge Conditions

- **Empty body:** Mock returns 403 with body `""` (zero bytes). Expected: `detail` contains
  `"403"` with no body snippet (fallback to status-only). This tests that the body-capture
  logic does not append an empty string or error when the body is absent.

- **Body exceeds 256 bytes:** See truncation sub-assertion above.

- **Control characters in body:** Mock returns 403 with body containing `\x00\x01\x07`
  (NUL + BEL). Expected: control characters are stripped from the detail snippet; the status
  code is still surfaced. Full-credit requires sanitized body (no control chars in output).

- **Concurrent pipeline steps:** The body capture on the first-request non-2xx and the
  401-retry non-2xx are symmetric. Both are exercised by this scenario (the 403 case
  never retries, testing only the first-request path; a separate 401 then 403 sequence
  would test the retry path — the evaluator may optionally add this sub-case).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-002 (satisfaction: X.XX) — error detail for a non-2xx sensor response is missing the response body snippet; only the status code appears in the detail field"`

Do NOT disclose: the specific body string used in testing, the truncation threshold asserted,
or the mock server implementation.

---

## Category: real-world-corpus

This scenario is grounded in the Claroty xDome WAF behavior that motivated this story: the
xDome API at `api.claroty.com` is fronted by an AWS Global Accelerator + WAF configuration
that returns non-2xx responses with policy-specific error body text (e.g., `"Request blocked"`
or AWS WAF JSON rejection envelopes). Without body capture, an operator sees only `"HTTP 403"`
and has no information about which WAF rule blocked the request.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API at `api.claroty.com` WAF rejection behavior; simulated via local mock HTTP server in this evaluation |
| corpus_size | Single non-2xx response interaction; error body strings are 10–400 bytes typical |
| known_edge_cases | Empty body (status only), short body (full capture), long body (truncation at 256B), control characters (sanitization) |
| false_positive_threshold | Not applicable (error scenario; no false positives in the detection sense) |
| false_negative_threshold | Zero tolerance: missing body snippet means the operator cannot diagnose the WAF rule that blocked the request |

**Known-good corpus:** Sensor API returns HTTP 200 with valid OCSF data — expected result:
no error detail, normal `list_sensor_data` response with rows. Tests that the body-capture
change does not affect the success path.

**Known-problematic corpus:** Sensor API returns HTTP 403 with a non-empty body — expected
result: `detail` contains the status code AND a body snippet. This is the exact corpus
this scenario exercises.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-materialization | 2026-08-12 | product-owner | Initial authoring. Story-level holdout gate for non-2xx body capture in error detail. SINGLE-USE. |
