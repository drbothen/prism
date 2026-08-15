---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-007"
title: "xDome 5xx Response — check_sensor_health Wire: ConnectivityStatus Degraded, Not Down"
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
last_eval_satisfaction: "FAIL"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (3-CLEAN converged @ a5b61b35b). Tests 5xx → Degraded semantic: server-side error must NOT be classified as Down (which implies TCP-level unreachability). D-2151 decision: 4xx→auth_valid:false / 5xx→Degraded. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-007: xDome 5xx Response — check_sensor_health Wire: ConnectivityStatus Degraded, Not Down

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story re-gate)
**BC Traced:** BC-2.08.002 (5xx response → ConnectivityStatus::Degraded, not Down)
**Gate:** Story-level holdout re-gate — runs after LOCAL 3-CLEAN @ a5b61b35b. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario tests the 5xx classification path: when the xDome API returns a server-side error
(HTTP 5xx), the sensor is reachable at the TCP level (connection succeeded) but the service is
degraded. The prism binary must classify this as `Degraded` — NOT as `Down` (which implies TCP
unreachability) and NOT as an auth failure (which is a 4xx concern). This distinction is critical
for MSSP operators: `Degraded` means "the sensor is up but its API is overloaded/faulted — wait
and retry"; `Down` means "the sensor is unreachable at the network level — check connectivity."

The scenario asserts the `Degraded` string appears in the serialized wire output and that `Down`
does NOT appear.

**BDD supplement:**

**Given** the Claroty/xDome DTU is running and reachable at a localhost address, configured to
return HTTP 503 with body `"Service Unavailable"` for the xDome health-probe endpoint  
**When** `check_sensor_health` is called for the xDome/Claroty sensor  
**Then** the serialized MCP JSON response contains `"Degraded"` as a string value and does NOT
contain `"Down"` as a status value

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.08.002 | 5xx HTTP response → `ConnectivityStatus::Degraded` (not Down, not auth failure) | Core assertion: 503 → "Degraded" in serialized wire output |
| BC-2.08.002 | `Down` reserved for TCP-level unreachability; must NOT appear for server-side 5xx | Negative assertion: "Down" absent |

---

## Verification Approach

1. Start the Claroty DTU clone (`prism-dtu-claroty`): `BehavioralClone::start_on("127.0.0.1:0", ...)` and capture the bound port.

2. Configure the DTU to return HTTP 503 with body `"Service Unavailable"` for the xDome device-list endpoint (the health probe contacts this endpoint).

3. Start the prism binary in MCP stdio mode with the Claroty/xDome sensor configured:
   - `base_url` pointing at the DTU loopback address
   - Bearer token set to any valid placeholder value (the 503 is returned regardless of auth status — this simulates a server-side overload, not an auth failure)
   - AD-017: no real credential value is used

4. Over MCP stdio, send a `check_sensor_health` tool call for the Claroty/xDome sensor.

5. Capture the complete serialized JSON response byte string from prism's MCP stdout.

6. Assert on the raw byte string:

   **MUST be true:**
   - `contains("Degraded")` — the `Degraded` status string appears somewhere in the response
   - The response is valid JSON (parseable without error)

   **MUST NOT be true:**
   - The response contains a connectivity status of `"Down"` — specifically, `contains("\"Down\"")` or `contains("\"status\":\"Down\"")` or `contains("\"connectivity_status\":\"Down\"")` must all be false. If the status field uses a different serialization form, adapt the assertion to the specific field name used.
   - `contains("\"auth_valid\": false")` — a 503 is NOT an auth failure; auth_valid should not be false (either absent or true, depending on whether the field is present for 5xx responses)

7. Optionally check: does a `detail` or `error` field surface the 503 status code? Not a gate assertion, but captures diagnostic quality.

---

## Edge Conditions

- **Degraded vs Down distinction is the core semantic gate:** A response showing `"Down"` when TCP connection succeeded is the exact regression this scenario prevents. The DTU is running and TCP connects successfully; only the HTTP response is 503. If prism reports `Down`, it has conflated "server error" with "unreachable."

- **auth_valid interpretation for 5xx:** A 5xx response is not an authentication failure. The evaluator checks that `auth_valid` is NOT `false` for a 503. If `auth_valid` is absent (the field is optional on non-auth-failure paths), that is acceptable. If `auth_valid` is `true`, that is also acceptable. `auth_valid: false` would be a classification error (5xx mislabeled as auth failure).

- **Serialized form of Degraded:** The exact JSON field path (`"status"`, `"connectivity_status"`, or another key) is not specified here — the evaluator checks for the string `"Degraded"` anywhere in the response. If the implementation serializes the status as a different string (e.g., `"degraded"` lowercase), the assertion may need to be case-insensitive. If the string is absent entirely, the scenario fails.

- **Connection error during test setup:** If the DTU fails to start or prism cannot connect, retry once; if it fails again, report SETUP-FAILURE (not a behavioral finding) with the DTU startup log.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Degraded classification** (weight: 0.6): Does the serialized response contain the `"Degraded"` string (or equivalent serialized form of the Degraded connectivity status)? Full credit (1.0): present. Zero credit: absent.

- **Down absent** (weight: 0.3): Is the `"Down"` status string absent from the serialized response? Full credit (1.0): absent. Zero credit: present (TCP-level status incorrectly applied to server-error case).

- **auth_valid not false for 5xx** (weight: 0.1): Is `auth_valid` either absent or `true` (NOT `false`)? Full credit: not false. Zero credit: false (5xx mislabeled as auth failure).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-007 (satisfaction: X.XX) — check_sensor_health returned incorrect connectivity status for a server-side error response; the Degraded/Down distinction was not respected; verify the 5xx classification path in the health probe error handler"`

Do NOT disclose: the specific HTTP status code tested (503), the body string used, or which assertion failed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API — production behavior of HTTP 503 during service maintenance or rate-limit exhaustion; simulated via `prism-dtu-claroty` in this evaluation |
| known_edge_cases | 503 (server overload), 502 (gateway), 504 (timeout) — all 5xx variants → Degraded; only TCP-refused / DNS-failure → Down |
| false_positive_threshold | Zero: reporting `Down` for a reachable-but-overloaded sensor causes incorrect incident escalation |
| false_negative_threshold | Zero: reporting `Degraded` as `Down` would cause an MSSP to investigate network connectivity when the actual issue is a sensor API overload |

**Known-good corpus:** Same DTU returning HTTP 200 — expected: `Degraded` absent, `reachable: true`, `auth_valid: true`. Tests that a healthy sensor does not show Degraded.

**Known-problematic corpus:** DTU returning HTTP 503 (this scenario) — expected: `Degraded` present, `Down` absent.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-pass38-readj-holdout | 2026-08-14 | product-owner | Initial authoring. Re-gate scenario for 5xx → Degraded (not Down) classification. SINGLE-USE. No prior consumed scenario covered this surface. |
