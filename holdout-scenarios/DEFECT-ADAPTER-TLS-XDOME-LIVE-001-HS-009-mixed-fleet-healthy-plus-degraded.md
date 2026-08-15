---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-009"
title: "Mixed Fleet: One Healthy + One 500-Degraded Sensor — overall_status:partial, healthy_count:1, unhealthy_count:1"
category: "integration-boundaries"
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
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (3-CLEAN converged @ 8dd8d4285, post-HS-007-fix). Tests mixed fleet aggregation: one healthy sensor (200) + one degraded sensor (500) → overall_status:partial, healthy_count:1, unhealthy_count:1. Validates both the EC-08-009 fix and the HealthSummary aggregation logic in a realistic multi-client configuration. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-009: Mixed Fleet: One Healthy + One 500-Degraded Sensor — overall_status:partial, healthy_count:1, unhealthy_count:1

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story re-gate)
**BC Traced:** BC-2.08.002 (HealthSummary aggregation with Degraded sensor in fleet)
**Gate:** Story-level holdout re-gate — runs after LOCAL 3-CLEAN @ 8dd8d4285. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario tests the realistic MSSP operational condition where an analyst runs
`check_sensor_health` across a fleet that includes one fully healthy sensor and one server-degraded
sensor (HTTP 500). The prism binary must correctly aggregate the mixed fleet into `overall_status:
"partial"` — neither "healthy" (some sensors are not fully healthy) nor "unhealthy" (at least one
sensor is reachable and partially available).

The critical invariants being tested simultaneously:
1. The healthy sensor is counted in `healthy_count` (is_fully_healthy predicate: reachable + auth_valid + no error + no rate_limit)
2. The degraded sensor is NOT counted in `healthy_count` (error field is set → not fully healthy)
3. The degraded sensor IS counted in `unhealthy_count` (total - healthy)
4. `overall_status` is `"partial"` (not "unhealthy", because the degraded sensor is still partially available)

This scenario catches a class of aggregation bugs where:
- The EC-08-009 fix corrects the `reachable` field but `is_fully_healthy` still miscounts the degraded sensor as healthy
- OR the aggregation arm selects "unhealthy" instead of "partial" because it incorrectly excludes the degraded sensor from the `any_partially_available` check

**BDD supplement:**

**Given** two client organizations are configured (client-a → DTU instance A at port X returning HTTP 200
with valid device data; client-b → DTU instance B at port Y returning HTTP 500)  
**When** `check_sensor_health` is called across both clients  
**Then** the serialized MCP JSON response contains `"overall_status": "partial"`,
`summary_counts.healthy_count` = 1, `summary_counts.unhealthy_count` = 1,
`summary_counts.total_count` = 2, and the `sensors` array contains 2 entries where one has
`error` absent/null and one has `error: "service_unavailable"`

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.08.002 | `is_fully_healthy()` predicate — requires `error.is_none()` | Degraded sensor NOT counted in healthy_count |
| BC-2.08.002 | `any_partially_available` arm: degraded sensor with `reachable:true` + `auth_valid:true` IS partially available | overall_status selects "partial" not "unhealthy" |
| BC-2.08.002 | `HealthSummary` healthy_count reflects actual fully-healthy sensors only | healthy_count == 1 for mixed fleet |
| BC-2.08.002 | EC-08-009: 500-returning sensor has `reachable:true` in its individual sensors[] entry | Degraded sensor wire shape in sensors array |

---

## Verification Approach

1. Start two instances of the Claroty DTU clone (`prism-dtu-claroty`):
   - DTU-A: `BehavioralClone::start_on("127.0.0.1:0", ...)` in normal (200-success) mode — returns valid device data when queried. Capture bound port (PORT-A).
   - DTU-B: `BehavioralClone::start_on("127.0.0.1:0", ...)` in error (500) mode. Capture bound port (PORT-B).

2. Start the prism binary in MCP stdio mode with two client organizations configured:
   - client-a: `base_url` = `http://127.0.0.1:PORT-A`, bearer token = valid placeholder for the 200 path
   - client-b: `base_url` = `http://127.0.0.1:PORT-B`, bearer token = any placeholder (500 is returned regardless)
   - AD-017: no real credential values are used; use project credential override facility with placeholders (e.g., `hs009-client-a-placeholder-NOTREAL`, `hs009-client-b-placeholder-NOTREAL`)

3. Over MCP stdio, send a `check_sensor_health` tool call targeting both client organizations.

4. Capture the complete serialized JSON response byte string from prism's MCP stdout.

5. Assert on the raw byte string:

   **MUST be true — overall status:**
   - `contains("\"overall_status\":\"partial\"")` OR `contains("\"overall_status\": \"partial\"")` — mixed fleet → partial

   **MUST be true — summary counts (deserialize and verify numerically):**
   - `summary_counts.healthy_count` == 1 (client-a only)
   - `summary_counts.unhealthy_count` == 1 (client-b only)
   - `summary_counts.total_count` == 2
   - `summary_counts.rate_limited_count` == 0

   **MUST be true — individual sensor entries (deserialize sensors array):**
   - `sensors` array has length == 2
   - At least one entry: `reachable:true`, `auth_valid:true`, no `error` (client-a healthy entry)
   - At least one entry: `reachable:true`, `auth_valid:true`, `error:"service_unavailable"` (client-b degraded entry)

   **MUST NOT be true:**
   - `contains("\"overall_status\":\"healthy\"")` — one degraded sensor prevents "healthy"
   - `contains("\"overall_status\":\"unhealthy\"")` — degraded (not Down) prevents "unhealthy"
   - `summary_counts.healthy_count` == 2 — the degraded sensor must NOT be counted as healthy
   - `summary_counts.healthy_count` == 0 — the healthy sensor MUST be counted
   - Any sensors[] entry has `"reachable":false` — neither sensor is Down (both are TCP-connected)

---

## Edge Conditions

- **Two DTU instances required:** This scenario cannot be run with a single DTU instance unless the DTU supports per-client routing. Two separately-started DTU instances at different loopback ports are the simplest configuration.

- **healthy_count == 2 is the pre-fix regression pattern:** If the `is_fully_healthy` predicate lacks the `error.is_none()` check, both sensors would be counted as healthy. A `healthy_count:2` result means the error-field exclusion is missing.

- **overall_status "partial" requires correct any_partially_available:** The degraded sensor has `reachable:true` + `auth_valid:true`, which satisfies `any_partially_available`. If the aggregation arm incorrectly excludes it (e.g., because its error field is set), `overall_status` would be "unhealthy". "unhealthy" is the FAIL condition for this scenario.

- **Non-deterministic sensors[] ordering:** The two client results may appear in any order within the sensors array. The scenario asserts on element CONTENT, not element position.

- **AD-017 dual-credential check:** The response must not echo either placeholder credential value.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **overall_status:partial** (weight: 0.30): Is `overall_status` == `"partial"`? Full credit (1.0): yes. Zero credit: `"healthy"` or `"unhealthy"` (aggregation defect).

- **healthy_count == 1** (weight: 0.30): Is `summary_counts.healthy_count` exactly 1? Full credit: yes. Partial (0.3): 0 (healthy sensor missed). Zero credit: 2 (degraded sensor incorrectly counted).

- **unhealthy_count == 1** (weight: 0.20): Is `summary_counts.unhealthy_count` exactly 1? Full credit: yes. Zero credit: 0 or 2.

- **sensors array shape** (weight: 0.15): Do the sensors[] entries contain both a clean entry (no error) and a degraded entry (error:"service_unavailable")? Full credit: both present with correct reachable:true. Partial (0.5): one entry correct, one missing or wrong. Zero credit: both entries show error (healthy sensor miscounted as degraded).

- **reachable:false absent** (weight: 0.05): Is `"reachable":false` absent from the entire response? Full credit: absent. Zero credit: present (old EC-08-009 defect still present).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-009 (satisfaction: X.XX) — check_sensor_health with mixed sensor fleet returned incorrect aggregated status or summary counts; verify the healthy_count predicate and the partial-status aggregation arm"`

Do NOT disclose: the specific client IDs, DTU ports, HTTP status codes, or which exact count assertion failed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API — MSSP multi-client deployment where some clients have healthy xDome instances and others have degraded ones; simulated via two `prism-dtu-claroty` instances |
| known_edge_cases | Mixed fleet aggregation; healthy_count predicate; any_partially_available arm; non-deterministic sensor ordering |
| false_positive_threshold | Zero: `overall_status:"unhealthy"` for a partially-available fleet misguides MSSP operator into treating degraded-but-reachable sensors as fully down |
| false_negative_threshold | Zero: `healthy_count:2` for a fleet with one degraded sensor inflates operational health metrics silently |

**Known-good corpus:** Both clients pointing to healthy DTU (200) — expected: `overall_status:"healthy"`, `healthy_count:2`, `unhealthy_count:0`. Tests the all-healthy baseline.

**Known-problematic corpus:** Mixed fleet (this scenario) — expected: `overall_status:"partial"`, `healthy_count:1`, `unhealthy_count:1`.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-pass50-reauth-holdout | 2026-08-15 | product-owner | Initial authoring. Mixed fleet aggregation gate: EC-08-009 + HealthSummary counts for healthy + degraded sensors. SINGLE-USE. |
