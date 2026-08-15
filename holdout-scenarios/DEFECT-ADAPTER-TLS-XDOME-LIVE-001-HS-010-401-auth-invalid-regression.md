---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-010"
title: "401 Auth-Invalid Health Wire: reachable:true, auth_valid:false, suggestion present — Regression After EC-08-009 Fix"
category: "behavioral-subtleties"
must_pass: true
priority: P0
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
traces_to: "BC-2.08.002"
behavioral_contracts:
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
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (3-CLEAN converged @ 8dd8d4285, post-HS-007-fix). Regression gate: confirms the EC-08-009 fix (reachable != Down for 5xx) did NOT break the 401 auth-failure path. 401 must still produce reachable:true + auth_valid:false + non-empty suggestion. Prior HS-TLS-XDOME-004 (same surface) is CONSUMED; this scenario runs against the updated binary. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-010: 401 Auth-Invalid Health Wire: reachable:true, auth_valid:false, suggestion present — Regression After EC-08-009 Fix

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story re-gate)
**BC Traced:** BC-2.08.002 (auth_valid:false path, post-EC-08-009-fix regression gate)
**Gate:** Story-level holdout re-gate — runs after LOCAL 3-CLEAN @ 8dd8d4285. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

The EC-08-009 fix changed the `reachable` predicate from `== Up` to `!= Down`, and added
`error.is_none()` to the `is_fully_healthy()` predicate. Both changes touch the health evaluation
logic that also handles 4xx responses. This scenario confirms the fix did NOT introduce a
regression on the 401 auth-failure path:

- 401 → `Up` connectivity (TCP connected, HTTP exchange occurred) → `reachable: true`
- 401 → `AuthStatus::Invalid` → `auth_valid: false`
- 401 → NOT fully healthy → `overall_status: "unhealthy"` (auth failure, not degraded)
- `suggestion` field must be non-empty (D-2151 auth failure spec)

The concern: the `!= Down` predicate change could cause a subtle 401 regression if the
implementation conflated auth-failure connectivity with Degraded connectivity. This scenario
explicitly verifies the 401 path remains distinct from both Degraded (5xx) and Down (TCP fail).

**BDD supplement:**

**Given** the Claroty/xDome DTU is running at a loopback address and configured to return HTTP 401
for the health-probe endpoint, and prism is configured with a placeholder invalid bearer token  
**When** `check_sensor_health` is called for the single configured xDome/Claroty sensor  
**Then** the serialized MCP JSON response contains `"reachable": true`, `"auth_valid": false`,
a non-empty `suggestion` field, and `"overall_status": "unhealthy"`

**AD-017 compliance is mandatory throughout:** no real xDome credential value is used.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.08.002 | 4xx HTTP response → `auth_valid:false`; `reachable:true` when TCP connection succeeded | Core: 401 → reachable:true, auth_valid:false |
| BC-2.08.002 | Diagnostic `suggestion` field present on auth failure (D-2151 auth failure spec) | `suggestion` key is present and non-empty string |
| BC-2.08.002 | 401 auth failure → `overall_status:"unhealthy"` (not "partial" — auth failure is not "partially available") | overall_status field value |
| BC-2.08.002 | 4xx path is distinct from 5xx Degraded path — `auth_valid:false` for 401, `auth_valid:true` for 500 | Negative: `auth_valid:true` must NOT be present for 401 |

---

## Verification Approach

1. Start the Claroty DTU clone (`prism-dtu-claroty`): `BehavioralClone::start_on("127.0.0.1:0", ...)` and capture the bound port.

2. Configure the DTU to return HTTP 401 for the health-probe endpoint. The 401 body format expected by the DTU is its hardcoded auth-failure response.

3. Start the prism binary in MCP stdio mode with the Claroty/xDome sensor configured:
   - `base_url` pointing at the DTU loopback address (HTTP, not HTTPS — DTU bypasses TLS)
   - Bearer token set to a placeholder invalid value (e.g., `PRISM_CLAROTY_API_TOKEN=hs010-invalid-NOTREAL`)
   - AD-017: no real credential value is used; the placeholder must not appear in logs or error text

4. Over MCP stdio, send a `check_sensor_health` tool call.

5. Capture the complete serialized JSON response byte string from prism's MCP stdout.

6. Assert on the raw byte string (whitespace variants acceptable):

   **MUST be true:**
   - `contains("\"reachable\":true")` OR `contains("\"reachable\": true")` — TCP connected → reachable
   - `contains("\"auth_valid\":false")` OR `contains("\"auth_valid\": false")` — 401 → auth invalid
   - `contains("\"suggestion\"")` — suggestion key is present
   - Deserialize: `suggestion` value is a non-empty string (not null, not `""`)
   - `contains("\"overall_status\":\"unhealthy\"")` OR `contains("\"overall_status\": \"unhealthy\"")` — auth failure → unhealthy

   **MUST NOT be true:**
   - `contains("\"reachable\":false")` — sensor IS reachable (TCP connected for 401)
   - `contains("\"auth_valid\":true")` — 401 is an auth failure; auth_valid must be false
   - `contains("\"auth_valid\":null")` — null would indicate Down, not auth failure
   - `contains("\"overall_status\":\"partial\"")` — partial is for 5xx Degraded; 401 is auth failure → unhealthy

7. AD-017 verification: the serialized response does NOT contain the placeholder token value used in step 3.

---

## Edge Conditions

- **401 vs 500 distinction:** The evaluator explicitly checks that `auth_valid:false` is present (this is the 401 path), not `auth_valid:true` (which would indicate the 500 Degraded path). The two paths MUST remain distinct after the EC-08-009 fix.

- **suggestion vs error distinction for auth failures:** On the 401 path, `auth_valid:false` replaces the `error` field as the primary diagnostic signal. The `suggestion` field provides the operator guidance. `error` may be present or absent on the 401 path — the evaluator checks `suggestion`, not `error`, as the diagnostic field for auth failures.

- **overall_status "unhealthy" for single 401 sensor:** With one sensor returning 401:
  - `reachable:true`, `auth_valid:false` → `is_fully_healthy()` fails (auth_valid != Some(true))
  - `any_partially_available`: `reachable != Some(false)` is true, but `auth_valid != Some(false)` is false (auth_valid IS Some(false)) → `any_partially_available` is false
  - Result: neither healthy nor partially available → `overall_status:"unhealthy"`

- **Contrast with Degraded (500) path:** A 500-returning sensor has `auth_valid:true` (not false) and `error:"service_unavailable"` (not null). The `any_partially_available` check is true for 500 (because auth_valid != Some(false)). Result: 500 → `"partial"`, 401 → `"unhealthy"`. The scenario asserts `"unhealthy"` specifically.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **auth_valid:false present** (weight: 0.35): Is `"auth_valid":false` present? Full credit (1.0): yes. Zero credit: `auth_valid:true` (401 treated as Degraded, regression from EC-08-009 fix) or `auth_valid:null` (401 treated as Down).

- **reachable:true present** (weight: 0.25): Is `"reachable":true` present? Full credit: yes. Zero credit: `reachable:false` (sensor IS TCP-reachable for 401 — same regression as HS-007 but on auth path).

- **suggestion non-empty** (weight: 0.20): Is `suggestion` a non-empty string? Full credit: yes. Partial (0.5): key present but null or empty string. Zero credit: key absent.

- **overall_status:unhealthy** (weight: 0.15): Is `overall_status` == `"unhealthy"`? Full credit: yes. Zero credit: `"partial"` (auth failure incorrectly treated as Degraded) or `"healthy"` (auth failure ignored).

- **AD-017 compliance** (weight: 0.05): Does the response NOT contain the placeholder token? Full credit: clean. Zero credit: token visible.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-010 (satisfaction: X.XX) — check_sensor_health returned incorrect wire fields for an auth-failure response; verify the auth_valid and reachable fields on the 4xx classification path still hold after recent health-probe changes"`

Do NOT disclose: the specific HTTP status code tested, the DTU configuration, the credential placeholder used, or which exact assertion failed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API — HTTP 401 for expired or revoked API credentials; MSSP operator needs accurate auth-failure diagnosis to take credential rotation action; simulated via `prism-dtu-claroty` |
| known_edge_cases | auth_valid:false for 401; suggestion present; overall_status:"unhealthy" for single auth-failing sensor; 401 path distinct from 500 Degraded path |
| false_positive_threshold | Zero: `reachable:false` for a TCP-connected sensor with auth failure would mislead operator into checking network connectivity instead of rotating credentials |
| false_negative_threshold | Zero: `auth_valid:true` for an auth-failing sensor masks a real credential problem |

**Known-good corpus:** Same DTU configured to accept the bearer token (200 response) — expected: `reachable:true`, `auth_valid:true`, `overall_status:"healthy"`. Tests the positive auth path.

**Known-problematic corpus:** DTU returning 401 with invalid token (this scenario) — expected: `reachable:true`, `auth_valid:false`, `suggestion` non-empty, `overall_status:"unhealthy"`.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-pass50-reauth-holdout | 2026-08-15 | product-owner | Initial authoring. Regression gate for 401 auth-failure path after EC-08-009 fix. Supersedes consumed HS-TLS-XDOME-004 (same surface, post-fix binary). SINGLE-USE. |
