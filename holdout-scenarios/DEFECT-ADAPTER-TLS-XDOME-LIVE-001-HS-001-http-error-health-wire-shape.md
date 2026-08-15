---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-001"
title: "HTTP 401 Sensor Response — check_sensor_health Wire Output: reachable True, auth_valid False"
category: "behavioral-subtleties"
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
traces_to: "BC-2.08.002"
behavioral_contracts:
  - BC-2.08.002
  - BC-2.16.002
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
notes: "HIDDEN, SINGLE-USE story-level holdout gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — F9 error-surfacing path, wire-shape verification. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-001: HTTP 401 Sensor Response — check_sensor_health Wire Output: reachable True, auth_valid False

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.08.002 (Auth Validity Check Per Sensor Per Client), BC-2.16.002 (HTTP Client Compliance postcondition)
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario exercises the F9 error-surfacing path end-to-end via real MCP stdio communication
with the prism binary and the Claroty DTU clone. When the sensor API returns HTTP 401, the prism
binary must correctly classify the outcome as a reachable sensor with invalid credentials — not a
down sensor — and surface that classification in the serialized JSON output of the `check_sensor_health`
MCP tool.

**Behavioral assertions:**

1. The Claroty DTU clone is running on localhost; prism is configured to point at it with an
   intentionally invalid bearer token (not a real credential — a placeholder value the
   evaluator controls; AD-017: no real credential values transit AI context or evaluation logs)
2. The evaluator invokes `check_sensor_health` via MCP stdio with the Claroty sensor identifier
3. The DTU, receiving a request with an unrecognized token, returns HTTP 401
4. The serialized JSON bytes returned by prism to the MCP client contain `"reachable": true`
   (JSON boolean, not the string `"true"`), `"auth_valid": false` (JSON boolean, not absent),
   and a `detail` field whose text contains the substring `"401"`
5. The JSON bytes do NOT contain `"reachable": false`

**BDD supplement:**

**Given** the Claroty DTU is running and reachable at a localhost address, and prism is configured
with an invalid bearer token for the Claroty sensor  
**When** `check_sensor_health` is called for the Claroty sensor  
**Then** the serialized MCP JSON response contains `"reachable": true`, `"auth_valid": false`,
and a `detail` field surfacing the HTTP 401 status code

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.08.002 | EC-08-006: HTTP 4xx sensor response → `auth_valid: false`; HTTP Error Classification postcondition: `map_spec_engine_error_to_sensor_error` with `status_code > 0` → `SensorError::HttpError` | Core assertion: 401 → reachable:true, auth_valid:false at the wire level |
| BC-2.16.002 | HTTP Client Compliance postcondition; Non-2xx Response Body Capture postcondition | HTTP error status code surfaces in serialized detail field |

---

## Verification Approach

1. Start the Claroty DTU clone: start `ClarotyClone` via `BehavioralClone::start_on("127.0.0.1:0", ...)`;
   capture the bound address (e.g., `127.0.0.1:PORT`)

2. Start the prism binary in MCP stdio mode with the Claroty sensor configured:
   - `base_url` set to `http://127.0.0.1:PORT` (the DTU address; HTTP, not HTTPS, for local loopback)
   - Bearer token set to a placeholder invalid value (e.g., env override
     `PRISM_CLAROTY_API_TOKEN=holdout-test-invalid-NOTREAL`; the exact mechanism uses the
     project's credential override facility; **no real credential value is used — AD-017 compliance
     is mandatory throughout**; the placeholder must not appear in logs or error text)

3. Over MCP stdio, send a `check_sensor_health` tool call for the Claroty sensor
   (include the client/org identifier matching the configured test profile)

4. Capture the complete JSON response byte string from prism's MCP stdout

5. Assert on the raw byte string (both whitespace variants acceptable):
   - `contains("\"reachable\": true")` OR `contains("\"reachable\":true")`
   - `contains("\"auth_valid\": false")` OR `contains("\"auth_valid\":false")`
   - `contains("401")`
   - NOT `contains("\"reachable\": false")` AND NOT `contains("\"reachable\":false")`

6. Verify AD-017: the `detail` field text does NOT contain the placeholder token value used
   in step 2

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Functional correctness** (weight: 0.5): Do `"reachable": true` AND `"auth_valid": false`
  appear as JSON boolean values in the serialized response? Is `"401"` present in detail?
  Full credit (1.0): all three byte-string assertions pass. Half credit (0.5): reachable/auth_valid
  correct but 401 absent from detail. Zero credit (0.0): `"reachable": false` present OR
  `auth_valid` absent from the response.

- **Data integrity** (weight: 0.3): Is the response valid JSON? Are `reachable` and `auth_valid`
  proper JSON booleans (not strings `"true"` / `"false"`, not null, not absent)?
  Full credit: well-formed JSON with boolean types. Zero credit: malformed JSON or wrong types.

- **Error quality** (weight: 0.1): Is the `detail` field non-empty and human-readable?
  Does it describe the 401 condition in a way an operator could act on?

- **AD-017 compliance** (weight: 0.1): Does the error response avoid echoing the credential
  placeholder value? Full credit: no token value leakage. Zero credit: credential value in output.

---

## Edge Conditions

- **NULL vs absent vs boolean:** `auth_valid` must be a JSON boolean `false`, not the JSON
  string `"false"`, not null, not absent. The wire-shape assertion distinguishes these.

- **reachable must be true:** The DTU is running and reachable; TCP connection succeeds.
  A result of `reachable: false` is a behavioral defect (the prior bug), not ambiguous.

- **detail must surface the status code:** If `detail` is empty or generic (no "401"), the
  error-surfacing path is incomplete even if auth_valid is correct.

- **Connection error during test setup:** If the DTU fails to start or prism cannot connect,
  retry once; if it fails again, report SETUP-FAILURE (not a behavioral finding) with the
  DTU startup log.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-001 (satisfaction: X.XX) — check_sensor_health returned incorrect reachable/auth_valid values or missing status code for a 401 sensor response; verify the error classification path in the health probe pipeline"`

Do NOT disclose: the specific assertion that failed, the exact expected byte string, or
the DTU configuration used.

---

## Category: real-world-corpus

This scenario is grounded in the production behavior of the Claroty xDome API (`api.claroty.com`)
as observed during the incident that triggered this story: the live API returned HTTP 401 for an
expired credential, but the prism binary reported `reachable: false`, preventing an operator
from distinguishing a credential problem from a network outage.

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API at `api.claroty.com` — production MSSP deployment; simulated via `prism-dtu-claroty` DTU clone in this evaluation |
| corpus_size | Single health-probe interaction; structured health object response (no data rows) |
| known_edge_cases | HTTP 401 (expired/invalid token), HTTP 403 (wrong scope), transport failure (TCP refused) — these three must produce distinct `reachable`/`auth_valid` outcomes at the wire level |
| false_positive_threshold | Zero: `reachable: false` when sensor IS reachable is a P1 diagnostic error |
| false_negative_threshold | Zero: `auth_valid: true` when credentials are invalid masks a real credential problem |

**Known-good corpus:** Claroty DTU with a correctly seeded bearer token — expected result:
`reachable: true`, `auth_valid: true`. Tests that a working sensor is not incorrectly flagged.

**Known-problematic corpus:** Claroty DTU with an invalid bearer token — expected result:
`reachable: true`, `auth_valid: false`, `detail` contains "401". This is the exact corpus this
scenario exercises.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-materialization | 2026-08-12 | product-owner | Initial authoring. Story-level holdout gate for F9 error-surfacing wire-shape. SINGLE-USE. |
