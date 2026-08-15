---
document_type: holdout-scenario
level: L3
id: "HS-TLS-XDOME-006"
title: "Cross-Client Partial Failure — One xDome Client 403, One Succeeds: sensor_errors Entry + rows Non-Empty"
category: "integration-boundaries"
must_pass: true
priority: P0
epic_id: "engine-defects"
story_source: "DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
version: "1.0"
status: draft
producer: product-owner
timestamp: "2026-08-14T00:00:00Z"
phase: 3
inputs:
  - stories/DEFECT-ADAPTER-TLS-XDOME-LIVE-001-live-xdome-https-fails-waf-h1-no-ua.md
input-hash: "abada5b"
traces_to: "BC-2.11.001"
behavioral_contracts:
  - BC-2.11.001
  - BC-2.01.010
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
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (3-CLEAN converged @ a5b61b35b). Tests cross-client partial failure: one client 403s, the other succeeds. sensor_errors carries the failing client entry AND rows carries the succeeding client's data. No prior consumed scenario covered this surface. Test-writer and implementer must NOT read this file."
---

# HS-TLS-XDOME-006: Cross-Client Partial Failure — One xDome Client 403, One Succeeds: sensor_errors Entry + rows Non-Empty

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001
**Must Pass:** YES (P0 — blocks story re-gate)
**BC Traced:** BC-2.11.001 (cross-client partial failure EC-11-091), BC-2.01.010 (partial failure handling)
**Gate:** Story-level holdout re-gate — runs after LOCAL 3-CLEAN @ a5b61b35b. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario tests the real-world operational condition where an MSSP analyst queries the xDome
sensor across two client organizations simultaneously — one client's xDome API has an auth or
policy issue (returns HTTP 403), while the other client's xDome API is healthy and returns records.
The prism binary must surface BOTH outcomes in a single query response: partial records from the
succeeding client in `rows`, and a per-target HTTP error entry for the failing client in
`sensor_errors`.

The key behavioral guarantee: `sensor_errors` MUST NOT be empty or absent (the failing client's
error must be surfaced), AND `rows` MUST NOT be empty (the succeeding client's data must appear).
These are the two simultaneous invariants that make partial results useful for the analyst.

**BDD supplement:**

**Given** prism is configured with two client organizations (org-a and org-b), both mapped to the
same Claroty/xDome DTU, where the DTU is configured to return HTTP 403 with body `"Forbidden"` for
org-a's requests and return seeded device records for org-b's requests  
**When** the `query` tool is called with `query="FROM xdome_devices"` and `clients=["org-a","org-b"]`  
**Then** the serialized response contains `"sensor_errors"` with an entry `"xdome_devices: HTTP 403: Forbidden"` AND `"rows"` is a non-empty array containing org-b's device records

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | EC-11-091: live cross-client partial failure — AllTargetsFailed arm for failing client + rows from succeeding client | `sensor_errors` non-empty AND `rows` non-empty simultaneously in a single response |
| BC-2.11.001 | Per-target HTTP detail format in `sensor_errors` entries | Entry format `"xdome_devices: HTTP 403: Forbidden"` |
| BC-2.01.010 | Partial failure handling: succeeding results reach the client even when some targets fail | `rows` non-empty with org-b data |

---

## Verification Approach

1. Start the Claroty DTU clone (`prism-dtu-claroty`): `BehavioralClone::start_on("127.0.0.1:0", ...)` and capture the bound port.

2. Configure the DTU with per-client response routing:
   - Requests bearing org-a's identity → HTTP 403 with body `"Forbidden"`
   - Requests bearing org-b's identity → HTTP 200 with at least one seeded device record (e.g., `{"device_id":"hs006-device-01","asset_type":"OT","risk_score":42}`)
   - The mechanism for per-client routing depends on DTU capabilities: token-based, header-based, or URL-based routing; any mechanism that reliably differentiates the two clients is acceptable.

3. Start the prism binary in MCP stdio mode with:
   - org-a configured: `base_url` → DTU loopback address, bearer token → placeholder value that triggers the 403 route
   - org-b configured: `base_url` → same DTU loopback address, bearer token → placeholder value that triggers the 200 route
   - AD-017: no real credential values are used

4. Over MCP stdio, send a `query` tool call with `query="FROM xdome_devices"` and `clients=["org-a","org-b"]`.

5. Capture the complete serialized JSON response byte string from prism's MCP stdout.

6. Assert on the raw byte string:

   **MUST be true:**
   - `contains("\"sensor_errors\"")` — key present
   - `contains("\"xdome_devices: HTTP 403: Forbidden\"")` — org-a's error entry in HTTP format
   - `contains("\"rows\"")` — rows key present
   - `sensor_errors`, when deserialized, is a non-empty array of non-empty strings
   - `rows`, when deserialized, is a non-empty array (org-b data present)

   **MUST NOT be true:**
   - `sensor_errors` value is `null` or `[]`
   - `rows` value is `null` or `[]`
   - `contains("sensor error (E-SENSOR-")` — old error-code-only form absent

7. Optionally verify that the org-b result rows contain the seeded device fields (e.g., `"hs006-device-01"` appears in the serialized response). This confirms the data path, not just the `rows` array shape.

---

## Edge Conditions

- **Concurrent fan-out ordering:** The two clients are queried concurrently. The response ordering of `sensor_errors` entries and `rows` entries is non-deterministic, but the presence of each is deterministic. The scenario does NOT assert ordering.

- **Partial-failure response must NOT be an error response:** The HTTP 403 on org-a is a SENSOR failure, not a tool-call failure. The MCP tool call itself MUST succeed (return a valid tool result, not an MCP error). A tool-level error response when any sensor fails is a separate defect.

- **sensor_errors entry count:** With two clients and one failing, there should be exactly ONE entry in `sensor_errors` (for org-a's table). If `sensor_errors` contains two entries (one for each client, including a spurious entry for org-b), that is a correctness defect.

- **rows content:** The scenario asserts `rows` is non-empty. The exact count of rows depends on DTU seeding; asserting non-zero is sufficient for the gate.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **sensor_errors correctness** (weight: 0.4): Is `sensor_errors` a non-null, non-empty array with an entry matching `"xdome_devices: HTTP 403: Forbidden"`? Full credit (1.0): exact format match. Partial (0.5): entry present but wrong format (e.g., missing body snippet or wrong prefix). Zero credit: `sensor_errors` null/`[]` or entry uses old error-code form.

- **rows non-empty** (weight: 0.35): Is `rows` a non-null, non-empty array? Full credit (1.0): non-empty. Zero credit: null, `[]`, or absent — meaning org-b's successful data was lost.

- **Tool-call success** (weight: 0.15): Did the MCP `query` call return a tool result (not an MCP JSON-RPC error)? Full credit: valid tool result envelope. Zero credit: MCP-level error (tool failure instead of partial-success response).

- **Negative assertion** (weight: 0.1): Is the old `"sensor error (E-SENSOR-"` form absent from `sensor_errors`? Full credit: absent. Zero credit: present.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT LOW: HS-TLS-XDOME-006 (satisfaction: X.XX) — cross-client query returned incorrect partial-failure response; either sensor_errors is missing/wrong-format for the failing client, or rows is empty despite a succeeding client; verify the partial-result fan-out path"`

Do NOT disclose: the client IDs used (org-a/org-b), the HTTP status or body tested, the seeded device data, or which specific assertion failed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty xDome API — production MSSP scenario where multiple client orgs share the same sensor type; simulated via `prism-dtu-claroty` in this evaluation |
| known_edge_cases | Concurrent fan-out ordering; partial-failure vs tool-error distinction; sensor_errors array cardinality |
| false_positive_threshold | Low: a spurious `sensor_errors` entry for the succeeding client (org-b) is a separate defect, caught by count assertion |
| false_negative_threshold | Zero: dropping org-b's successful `rows` data is a silent data-loss defect |

**Known-good corpus:** Same query with `clients=["org-b"]` only (no org-a) — expected: `rows` non-empty, `sensor_errors` ABSENT. Tests that the success path is clean.

**Known-problematic corpus:** Query with both clients (this scenario) — expected: `sensor_errors` non-empty (org-a's HTTP format entry) AND `rows` non-empty (org-b data).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-pass38-readj-holdout | 2026-08-14 | product-owner | Initial authoring. Re-gate scenario for cross-client partial failure wire output. SINGLE-USE. No prior consumed scenario covered this surface. |
