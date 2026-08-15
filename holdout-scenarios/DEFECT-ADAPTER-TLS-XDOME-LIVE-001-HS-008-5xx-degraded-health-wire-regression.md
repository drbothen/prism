---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-008"
title: "5xx → Degraded Health Wire: reachable:true, auth_valid:true, error:service_unavailable (EC-08-009 Regression Gate)"
category: "behavioral-subtleties"
must_pass: true
priority: P0
epic_id: "engine-defects"
story_source: "DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
version: "1.0"
status: consumed
producer: product-owner
timestamp: "2026-08-15T00:00:00Z"
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
last_evaluated: 2026-08-15
last_eval_satisfaction: 1.00
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (3-CLEAN converged @ 8dd8d4285, post-HS-007-fix). Direct regression gate for EC-08-009: asserts wire-level reachable:true (NOT false) and error:service_unavailable for a single sensor returning HTTP 500. HS-007 FAIL revealed reachable:false was emitted for 5xx; this scenario asserts the corrected wire contract at the field level. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-008: 5xx → Degraded Health Wire: reachable:true, auth_valid:true, error:service_unavailable (EC-08-009 Regression Gate)

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story re-gate)
**BC Traced:** BC-2.08.002 EC-08-009 (HTTP 5xx → Degraded wire contract)
**Gate:** Story-level holdout re-gate — runs after LOCAL 3-CLEAN @ 8dd8d4285. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario is the direct regression gate for EC-08-009 (authored after HS-007 FAIL): when the
xDome API returns HTTP 500, the `check_sensor_health` MCP tool MUST serialize the result with
`reachable: true` (TCP connection succeeded), `auth_valid: true` (500 is not an auth failure),
and `error: "service_unavailable"`. The `overall_status` MUST be `"partial"` (not `"unhealthy"`
which implies unreachability) and `summary_counts.healthy_count` MUST be 0 (the sensor is not
fully healthy because its error field is set).

The prior HS-007 scenario exposed the defect using an imprecise string assertion ("Degraded" in
the response). This scenario asserts the specific wire fields that distinguish Degraded from Down:
`reachable: true` (Down would have `reachable: false`) and `error: "service_unavailable"` (Down
would have `error: "sensor_unreachable_cannot_verify"`).

**BDD supplement:**

**Given** the Claroty/xDome DTU is running at a loopback address and configured to return HTTP 500
for the health-probe endpoint  
**When** `check_sensor_health` is called for the single configured xDome/Claroty sensor  
**Then** the serialized MCP JSON response contains `"reachable": true` (boolean true, not false),
`"auth_valid": true`, `"error": "service_unavailable"`, `"overall_status": "partial"`,
`summary_counts.healthy_count` = 0, `summary_counts.unhealthy_count` = 1, and DOES NOT contain
`"reachable": false`

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.08.002 | EC-08-009: HTTP 5xx → `reachable:true` on wire (NOT false); distinct from Down (`reachable:false`) | Core field assertion: `reachable` boolean is `true` for 500 response |
| BC-2.08.002 | EC-08-009: `auth_valid:true` for 5xx (not an auth failure) | `auth_valid` boolean is `true` (not false, not null) |
| BC-2.08.002 | EC-08-009: `error:"service_unavailable"` for 5xx | `error` string value |
| BC-2.08.002 | `is_fully_healthy()` excludes sensors where `error` is set | `summary_counts.healthy_count` == 0 |
| BC-2.08.002 | `overall_status:"partial"` for a reachable-but-erroring sensor | `overall_status` field value |

---

## Verification Approach

1. Start the Claroty DTU clone (`prism-dtu-claroty`): `BehavioralClone::start_on("127.0.0.1:0", ...)` and capture the bound port.

2. Configure the DTU to return HTTP 500 with body `{"error": "internal server error (injected)", "code": 500}` for all requests to the xDome devices endpoint (the health probe contacts this endpoint).

3. Start the prism binary in MCP stdio mode with a single Claroty/xDome sensor configured:
   - `base_url` pointing at the DTU loopback address
   - Bearer token set to any valid placeholder value (the 500 is returned regardless of auth — simulates server-side overload)
   - AD-017: no real credential value is used; use project credential override facility with a placeholder (e.g., `PRISM_CLAROTY_API_TOKEN=hs008-placeholder-NOTREAL`)

4. Over MCP stdio, send a `check_sensor_health` tool call.

5. Capture the complete serialized JSON response byte string from prism's MCP stdout.

6. Assert on the raw byte string (both `"field":value` and `"field": value` whitespace variants are acceptable):

   **MUST be true — reachable field:**
   - `contains("\"reachable\":true")` OR `contains("\"reachable\": true")` — reachable is boolean true

   **MUST be true — auth_valid field:**
   - `contains("\"auth_valid\":true")` OR `contains("\"auth_valid\": true")` — auth_valid is boolean true

   **MUST be true — error field:**
   - `contains("\"error\":\"service_unavailable\"")` OR `contains("\"error\": \"service_unavailable\"")` — error string matches

   **MUST be true — aggregated status:**
   - `contains("\"overall_status\":\"partial\"")` OR `contains("\"overall_status\": \"partial\"")` — overall is partial
   - The `sensors` array, when deserialized, is non-empty (at least one sensor result present)

   **MUST be true — summary counts:**
   - Deserialize `summary_counts.healthy_count` — MUST equal 0 (degraded sensor is NOT fully healthy)
   - Deserialize `summary_counts.unhealthy_count` — MUST equal 1
   - Deserialize `summary_counts.total_count` — MUST equal 1

   **MUST NOT be true:**
   - `contains("\"reachable\":false")` OR `contains("\"reachable\": false")` — the corrected wire MUST NOT have reachable:false for a TCP-connected sensor
   - `contains("\"overall_status\":\"unhealthy\"")` — unhealthy implies auth failure or unreachability, not 5xx degraded
   - `contains("\"auth_valid\":false")` OR `contains("\"auth_valid\": false")` — 5xx is NOT an auth failure

7. Verify the response is valid JSON (parseable without error).

---

## Edge Conditions

- **reachable:false is the regression target:** The prior defect emitted `reachable:false` for the exact configuration this scenario exercises. A `reachable:false` result is the FAIL condition; do not accept it as partial credit.

- **overall_status "partial" vs "unhealthy":** With one sensor that has `reachable:true` + error set, the sensor is partially available. `overall_status:"partial"` is correct. `overall_status:"unhealthy"` would indicate the old pre-fix behavior where the sensor was treated as Down-equivalent.

- **healthy_count must be 0:** `is_fully_healthy()` requires `error.is_none()`. A sensor with `error:"service_unavailable"` fails this predicate and must NOT be counted in `healthy_count`. If `healthy_count` is 1, the `is_fully_healthy` predicate is missing the `error.is_none()` check.

- **Down-sensor contrast:** Down sensors (`reachable:false`, `auth_valid:null`, `error:"sensor_unreachable_cannot_verify"`) would show `overall_status:"unhealthy"` and `healthy_count:0`. The wire distinction between Degraded and Down is `reachable:true` vs `reachable:false`. This scenario explicitly asserts the Degraded side.

- **DTU startup failure:** If the DTU fails to start, report SETUP-FAILURE (not a behavioral finding). Retry once.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **reachable:true present** (weight: 0.35): Is `"reachable":true` (or `"reachable": true`) present in the sensors array entry for the 500-returning sensor? Full credit (1.0): present. Zero credit: absent or `reachable:false` present instead.

- **overall_status:partial** (weight: 0.25): Is `overall_status` == `"partial"`? Full credit (1.0): yes. Zero credit: `"unhealthy"` or `"healthy"` instead.

- **healthy_count == 0** (weight: 0.20): Is `summary_counts.healthy_count` exactly 0? Full credit: yes. Zero credit: 1 (degraded sensor miscounted as healthy).

- **error:service_unavailable** (weight: 0.10): Is `error` == `"service_unavailable"` in the sensor entry? Full credit: yes. Zero credit: null or absent (error not surfaced) or wrong string.

- **reachable:false absent** (weight: 0.10): Is `"reachable":false` absent from the entire response? Full credit: absent. Zero credit: present (regression confirmed).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-008 (satisfaction: X.XX) — check_sensor_health returned incorrect wire fields for a server-side error response; verify the reachable field value and overall_status for the 5xx path"`

Do NOT disclose: the specific HTTP status code tested, the body string, the exact field values expected, or which assertion failed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API — HTTP 500 returned during backend service maintenance or internal failure; simulated via `prism-dtu-claroty` |
| known_edge_cases | Wire-level reachable/auth_valid/error field values for 5xx; healthy_count exclusion for erroring sensors |
| false_positive_threshold | Zero: `reachable:true` present when TCP connects — reporting false otherwise misleads MSSP operator |
| false_negative_threshold | Zero: `healthy_count:1` for a degraded sensor falsely inflates health metrics |

**Known-good corpus:** Same DTU in 200-success mode — expected: `reachable:true`, `auth_valid:true`, `error` absent/null, `overall_status:"healthy"`, `healthy_count:1`. Tests that the healthy path is unchanged.

**Known-problematic corpus:** DTU in 500-error mode (this scenario) — expected: `reachable:true`, `auth_valid:true`, `error:"service_unavailable"`, `overall_status:"partial"`, `healthy_count:0`.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-pass50-reauth-holdout | 2026-08-15 | product-owner | Initial authoring. EC-08-009 regression gate: wire-level reachable:true assertion for 5xx, post-HS-007-fix. SINGLE-USE. |
