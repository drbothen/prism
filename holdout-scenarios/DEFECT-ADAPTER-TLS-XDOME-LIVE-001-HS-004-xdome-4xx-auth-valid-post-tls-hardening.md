---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-004"
title: "xDome 4xx After Transport Hardening — check_sensor_health Wire: reachable true, auth_valid false, suggestion present"
category: "behavioral-subtleties"
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
traces_to: "BC-2.08.002"
behavioral_contracts:
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
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (3-CLEAN converged @ a5b61b35b). Validates auth_valid semantics STILL HOLD after rustls+http2+UA transport hardening. Prior HS-TLS-XDOME-001 (same behavioral claim) is CONSUMED. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-004: xDome 4xx After Transport Hardening — check_sensor_health Wire: reachable true, auth_valid false, suggestion present

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story re-gate)
**BC Traced:** BC-2.08.002 (Auth Validity Check Per Sensor Per Client)
**Gate:** Story-level holdout re-gate — runs after LOCAL 3-CLEAN @ a5b61b35b. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the auth_valid semantic contract is intact after the story's transport
hardening changes (rustls TLS backend, HTTP/2 upgrade, User-Agent injection). The concern: adding
new transport layers (http2 framing, UA header injection at the reqwest layer) could inadvertently
affect how non-2xx responses are classified — a response arriving via HTTP/2 framing might not
trigger the same error path as HTTP/1.1. The evaluator therefore runs the same 4xx test the prior
gate exercised, but against the CURRENT binary with transport hardening.

The scenario additionally asserts the `suggestion` field — which was NOT checked in the consumed
HS-TLS-XDOME-001 — since D-2151 decision (a) specifies auth failures must include a diagnostic
`suggestion` in the wire output.

**BDD supplement:**

**Given** the Claroty/xDome DTU is running and reachable at a localhost address, configured to return
HTTP 401 for the health-probe endpoint, and prism is configured with an invalid placeholder bearer
token for the xDome sensor  
**When** `check_sensor_health` is called for the xDome/Claroty sensor  
**Then** the serialized MCP JSON response contains `"reachable": true`, `"auth_valid": false`, and a
non-empty `suggestion` field

**AD-017 compliance is mandatory throughout:** no real xDome credential value is used.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.08.002 | 4xx HTTP response → `auth_valid: false`; `reachable: true` when TCP connection succeeds | Core assertion: 401 from DTU → reachable:true, auth_valid:false at the wire level |
| BC-2.08.002 | Diagnostic `suggestion` field present on auth failure | `suggestion` key non-empty string in serialized response |

---

## Verification Approach

1. Start the Claroty DTU clone (`prism-dtu-claroty`): launch via `BehavioralClone::start_on("127.0.0.1:0", ...)` and capture the bound address (e.g., `127.0.0.1:PORT`).

2. Configure the DTU to return HTTP 401 with a JSON body `{"error":"Token expired"}` for the xDome device-list endpoint (the health probe contacts this endpoint).

3. Start the prism binary in MCP stdio mode with the Claroty/xDome sensor configured:
   - `base_url` set to `http://127.0.0.1:PORT` (DTU address; HTTP, not HTTPS, for local loopback — DTU bypasses TLS)
   - Bearer token set to a placeholder invalid value (e.g., env override `PRISM_CLAROTY_API_TOKEN=hs004-invalid-NOTREAL`; the exact mechanism uses the project's credential override facility; **no real credential value is used — AD-017 compliance is mandatory**; the placeholder must not appear in logs or error text)

4. Over MCP stdio, send a `check_sensor_health` tool call for the Claroty/xDome sensor (include the client/org identifier matching the configured test profile).

5. Capture the complete JSON response byte string from prism's MCP stdout.

6. Assert on the raw byte string (whitespace variants both acceptable):
   - `contains("\"reachable\": true")` OR `contains("\"reachable\":true")`
   - `contains("\"auth_valid\": false")` OR `contains("\"auth_valid\":false")`
   - `contains("\"suggestion\"")` — key present
   - Deserialize and verify the `suggestion` value is a non-empty string (not null, not `""`)
   - NOT `contains("\"reachable\": false")` AND NOT `contains("\"reachable\":false")`
   - NOT `contains("\"auth_valid\": true")` AND NOT `contains("\"auth_valid\":true")`

7. Verify AD-017: the serialized response does NOT contain the placeholder token value used in step 3.

---

## Edge Conditions

- **NULL vs absent vs boolean:** `auth_valid` must be a JSON boolean `false`, not the string `"false"`, not null, not absent. The wire-shape assertion distinguishes these.

- **suggestion vs detail:** The scenario asserts `suggestion` key presence (per D-2151 auth failure spec). If the implementation uses a different field name (e.g., `detail` only), partial credit applies — but the `suggestion` assertion is a MUST for full credit.

- **HTTP/2 framing must not change classification:** The transport hardening adds HTTP/2 support. If prism upgrades to HTTP/2 with the DTU (which serves HTTP/1.1), the connection negotiation must not mask the 401 response. If the DTU does not support HTTP/2 and prism correctly falls back to HTTP/1.1, the 401 must still flow through. A failure here indicates transport negotiation is interfering with error classification.

- **Connection error during test setup:** If the DTU fails to start or prism cannot connect, retry once; if it fails again, report SETUP-FAILURE (not a behavioral finding) with the DTU startup log.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Functional correctness** (weight: 0.4): Are `"reachable": true` AND `"auth_valid": false` present as JSON booleans? Full credit (1.0): both correct. Zero credit: `reachable: false` present OR `auth_valid` absent.

- **Suggestion field presence** (weight: 0.35): Is `suggestion` a non-empty string in the serialized response? Full credit: non-empty string. Half credit (0.5): key present but null or `""`. Zero credit: key absent.

- **Data integrity** (weight: 0.15): Are `reachable` and `auth_valid` proper JSON booleans (not strings, not null)? Full credit: correct types. Zero credit: wrong types.

- **AD-017 compliance** (weight: 0.1): Does the response avoid echoing the placeholder credential? Full credit: no leakage. Zero credit: placeholder token value appears in any response field.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-004 (satisfaction: X.XX) — check_sensor_health returned incorrect reachable/auth_valid values for a 4xx sensor response, or suggestion field absent; verify the auth classification path still holds after transport changes"`

Do NOT disclose: the specific assertion that failed, the exact expected byte string, the 401 HTTP status code used, or the DTU configuration.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API (`api.claroty.com`) — production behavior of HTTP 401 for expired credentials; simulated via `prism-dtu-claroty` in this evaluation |
| known_edge_cases | HTTP 401 after transport hardening; UA header injection must not affect error classification |
| false_positive_threshold | Zero: `reachable: false` when sensor IS reachable is a P1 diagnostic error |
| false_negative_threshold | Zero: `auth_valid: true` when credentials are invalid masks a real credential problem |

**Known-good corpus:** Same DTU with a valid placeholder bearer token — expected `reachable: true`, `auth_valid: true`. Tests transport hardening does not break the happy path.

**Known-problematic corpus:** Same DTU with an invalid bearer token (this scenario) — expected `reachable: true`, `auth_valid: false`, `suggestion` non-empty.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-pass38-readj-holdout | 2026-08-14 | product-owner | Initial authoring. Re-gate scenario for post-transport-hardening auth_valid semantics. SINGLE-USE. Supersedes consumed HS-TLS-XDOME-001 (adds suggestion assertion; runs against updated binary). |
