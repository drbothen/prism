---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-003"
title: "AllTargetsFailed Fan-Out — Per-Target Warning Events Emitted Before Propagation"
category: "integration-boundaries"
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
traces_to: "BC-2.01.010"
behavioral_contracts:
  - BC-2.01.010
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
notes: "HIDDEN, SINGLE-USE story-level holdout gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — AllTargetsFailed per-target WARN logging. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-003: AllTargetsFailed Fan-Out — Per-Target Warning Events Emitted Before Propagation

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.01.010 (Partial Failure Handling — AllTargetsFailed Per-Target Logging postcondition), BC-2.16.002 (Canonical Structured Event Catalog row 91)
**Gate:** Story-level holdout — runs after LOCAL 3-CLEAN, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario exercises the fan-out AllTargetsFailed logging path. When a multi-target query
fan-out results in every target failing, prism must emit one `fan_out_target_failed` WARN tracing
event per failing target before propagating the `AllTargetsFailed` error. The scenario asserts
on both the MCP wire-level response (the error propagation result) and the structured tracing
output (the per-target WARN events).

The prior behavior: `AllTargetsFailed` was returned silently — no per-target WARN events were
emitted, making it impossible for an operator to determine WHICH targets failed and WHY without
parsing raw error text. The fix adds a WARN loop that emits structured, AD-017-compliant events
with `event_type = "fan_out_target_failed"` before propagation.

**Behavioral assertions:**

1. Two org profiles for the Claroty sensor are configured; both `base_url` entries point to a
   mock server that returns HTTP 503 (or a DTU seeded to fail for all requests)
2. The evaluator invokes `list_sensor_data` (or equivalent) for the Claroty sensor across
   both org targets
3. Both fan-out targets fail with HTTP errors
4. The MCP response byte string contains an error indicating fan-out failure
   (e.g., contains `"AllTargetsFailed"` or an error code derived from E-SENSOR-030, or
   contains `"count": 2` or `"2"` in an error context indicating both targets failed)
5. The structured tracing log output (captured with `RUST_LOG=warn` or equivalent)
   contains exactly **2** WARN events with `event_type = "fan_out_target_failed"`
6. Each WARN event contains a `sensor_id` field matching the Claroty sensor identifier
   and an `org_id` field for the respective failing target
7. The WARN events appear in the log BEFORE the `AllTargetsFailed` error is returned
   in the MCP response

**BDD supplement:**

**Given** two org profiles for the Claroty sensor both pointing to a server that returns HTTP 503  
**When** `list_sensor_data` is invoked across both targets  
**Then** the MCP response is an error (AllTargetsFailed), AND the tracing log contains exactly
two `fan_out_target_failed` WARN events with `sensor_id` and `org_id` fields, neither containing
credential values (AD-017)

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.010 | AllTargetsFailed Per-Target Logging postcondition: each `FanOutError` MUST be logged at WARN before `AllTargetsFailed` propagates | Core assertion: 2 WARN events before AllTargetsFailed MCP response |
| BC-2.16.002 | Canonical Structured Event Catalog row 91: `fan_out_target_failed` WARN with fields `org_id`, `sensor_id`, `attempts`, `is_transient`, `error` | Structured event field schema validation |

---

## Verification Approach

1. Start a minimal mock HTTP server that returns HTTP 503 for any request; capture its address
   (e.g., `127.0.0.1:PORT_A`). Use the same server for both targets, OR start two
   instances at different ports (`PORT_A` and `PORT_B`).

2. Configure prism with two org profiles for the Claroty sensor:
   - Org `holdout-org-alpha`: `base_url = "http://127.0.0.1:PORT_A"`, placeholder token
   - Org `holdout-org-beta`: `base_url = "http://127.0.0.1:PORT_A"` (same server) or PORT_B,
     placeholder token
   AD-017: use placeholder tokens, not real credentials. The org identifiers are config values,
   not credentials.

3. Start the prism binary in MCP stdio mode with `RUST_LOG=warn` (or equivalent structured
   tracing subscriber) such that WARN events appear in stderr or a log file

4. Capture both the MCP stdout (wire response) and the tracing output (stderr or log file)

5. Over MCP stdio, invoke a data-fetching tool call for the Claroty sensor covering both orgs

6. Capture and assert:

   **Wire-level (MCP JSON response):**
   - Response is an error (does not contain data rows)
   - Response byte string contains `"2"` in an error context, OR contains
     `"AllTargetsFailed"`, OR contains error code `"E-SENSOR-030"` (whichever
     the implementation uses; all are valid error representations of the same failure)

   **Tracing output:**
   - Exactly 2 log lines or structured JSON objects contain `event_type.*fan_out_target_failed`
     (use grep or JSON parsing on the captured log)
   - Each event contains a `sensor_id` field value matching `"claroty"`
   - Each event contains a distinct `org_id` value (`"holdout-org-alpha"` and
     `"holdout-org-beta"` respectively, or the configured org identifiers)
   - No event contains a credential value (AD-017: placeholder tokens must not appear)

7. **Ordering check (optional but recommended):** Verify the WARN events appear in the log
   before the MCP error response is returned (timestamp-based if available, or log ordering
   if sequential)

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; weighted average. Satisfying threshold: ≥ 0.75.

- **Event count correctness** (weight: 0.4): Does the tracing log contain exactly 2
  `fan_out_target_failed` WARN events?
  Full credit: exactly 2 events with the correct sensor_id. Half credit: 1 event (partial).
  Zero credit: 0 events (pre-fix behavior — no per-target logging at all).

- **MCP wire correctness** (weight: 0.3): Does the MCP response reflect a 2-target failure?
  Full credit: error response with count-2 indication. Zero credit: success response or
  single-target error indication.

- **Field schema compliance** (weight: 0.2): Do the WARN events contain `sensor_id` and
  `org_id` fields? (Partial credit if one field present but not the other.)

- **AD-017 compliance** (weight: 0.1): Are placeholder tokens absent from the WARN event
  `error` or `org_id` or `sensor_id` fields? Full credit: no credential material in events.

---

## Edge Conditions

- **Zero targets fail:** Not exercised by this scenario (different scenario). This scenario
  specifically tests the all-targets-fail path.

- **Single target fails:** If only 1 of 2 targets fails, the behavior is partial failure
  (not AllTargetsFailed). This scenario configures both targets to fail, testing the
  all-fail case. The partial-fail case is tested by other scenarios.

- **Empty errors vec:** Not expected here (both targets fail, so `errors.len() == 2`). If
  the WARN loop is not added, exactly 0 WARN events appear, which is the clear FAIL signal.

- **org_id and sensor_id field presence:** The WARN event schema requires both fields (per
  BC-2.16.002 Canonical Structured Event Catalog row 91). Absence of either field is a
  schema violation, not just a missing-logging failure.

- **Error message contains target count:** The `AllTargetsFailed` Display implementation
  MUST remain count-only (e.g., `"E-SENSOR-030: all fan-out targets failed (2 errors)"`)
  per the BC contract. A multi-target error message that includes per-target detail in the
  AllTargetsFailed Display itself (not in the WARN loop) is an anti-pattern that mixes the
  logging and error propagation concerns — not a PASS.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-003 (satisfaction: X.XX) — AllTargetsFailed was returned without per-target structured warning events in the tracing log; individual target failure details are not observable"`

Do NOT disclose: the number of targets configured, the specific org identifiers used, the
server port numbers, or which of the two assertions (wire vs tracing) failed.

---

## Category: real-world-corpus

This scenario is grounded in MSSP multi-client operations: a prism operator managing multiple
client organizations (e.g., "monroe" and a second client on the same sensor type) needs to
know, when a sensor fetch fails for all clients, WHICH clients failed and WHY — without
parsing unstructured error text. The xDome API at `api.claroty.com` operates per client-org
with separate credentials; a credential expiry for one org should be diagnosable independently
of another org's failure.

| Field | Description |
|-------|-------------|
| corpus_source | MSSP multi-tenant Claroty deployment pattern; simulated via two mock HTTP targets in this evaluation |
| corpus_size | Two fan-out targets, both returning HTTP 503 (service unavailable) |
| known_edge_cases | Both targets fail with the same HTTP error code (count=2); empty errors vec (count=0); mixed success/failure (partial failure, not exercised here) |
| false_positive_threshold | Zero: spurious WARN events for targets that did NOT fail would contaminate operator logs |
| false_negative_threshold | Zero: missing WARN events for a target that DID fail leaves the operator with no per-target failure diagnostic |

**Known-good corpus:** Both targets succeed (HTTP 200) — expected result: no `fan_out_target_failed`
WARN events emitted, normal data response. Tests that the WARN loop does not fire on success.

**Known-problematic corpus:** Both targets return HTTP 503 — expected result: exactly 2
`fan_out_target_failed` WARN events followed by `AllTargetsFailed` error in MCP response.
This is the exact corpus this scenario exercises.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-materialization | 2026-08-12 | product-owner | Initial authoring. Story-level holdout gate for AllTargetsFailed per-target WARN logging. SINGLE-USE. |
