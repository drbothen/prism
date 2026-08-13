---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001, CAP-002"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "4a1f396"
traces_to: ["CAP-001", "CAP-002"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: "2026-08-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.010: Partial Failure Handling for Paginated and Cross-Client Queries

## Description

When a sensor query (single-client or cross-client) encounters a failure after at least one successful page or client response, Prism returns the data already fetched rather than discarding it. The response is annotated with `truncated: true`, a `truncation_reason`, and (for cross-client queries) a `partial_failures` array listing each failed client. The cursor advances only to the last successfully delivered page, enabling safe resume on next invocation.

## Preconditions
- A sensor query (single-client or cross-client) is in progress
- At least one page or one client's query has succeeded before a failure occurs

## Postconditions
- Successfully fetched data is returned to the caller (not discarded)
- Response includes `truncated: true` when pagination was interrupted
- Response includes `truncation_reason` describing the failure (e.g., "sensor_unavailable", "rate_limited", "authentication_expired")
- For cross-client queries, `partial_failures` array lists each failed client with error category and suggestion
- Cursor advances only to the last successfully fetched and delivered page
- **AllTargetsFailed Per-Target Logging:** When `SensorError::AllTargetsFailed` is about to be returned from `fanout()` in `crates/prism-sensors/src/fanout.rs`, each `FanOutError` in the `errors` vec MUST be logged at WARN level with `event_type = "fan_out_target_failed"` (per BC-2.16.002 Canonical Structured Event Catalog row 91 — see that BC for the full field schema including `org_id`, `sensor_id`, `attempts`, `is_transient`, `error`) before the error propagates. The E-SENSOR-030 Display remains count-only per BC-2.10.007 Rule 1 (E-SENSOR-* errors are MCP-redacted); per-target diagnostic detail is observable via the WARN events before `AllTargetsFailed` reaches the MCP surface. Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-004 + AC-SAP1-001 + RG-004 (`test_fanout_all_failed_emits_fan_out_target_failed_warn`).

## Invariants
- DI-001: Cursor advances only for successfully delivered pages (ephemeral in-memory cursor is not advanced beyond the last successful page)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | HTTP 503 mid-pagination | Not a tool-level error; partial results returned with metadata |
| N/A | HTTP 429 after backoff exhaustion | Not a tool-level error; partial results returned with `truncation_reason: "rate_limited"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-001 | HTTP 503 after some pages fetched | Return fetched pages with `truncated: true` and `truncation_reason: "sensor_unavailable"`; cursor at last successful page |
| EC-01-014 | First page fails (no data fetched) | Empty results with full error in metadata; this is still not a tool-level error for cross-client queries |
| EC-01-015 | Network timeout during a single-client query | Return any fetched pages as partial; if no pages fetched, return structured error with timeout details and retry suggestion |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.010-001 | 3-page query; HTTP 503 on page 3 | Pages 1-2 returned; `truncated: true`; `truncation_reason: "sensor_unavailable"`; cursor at end of page 2 |
| TV-BC-2.01.010-002 | Cross-client query; client B credentials expired | Client A results returned; `partial_failures` lists client B with `category: "authentication"` |
| TV-BC-2.01.010-003 | HTTP 429 after retry exhaustion | Partial results with `truncation_reason: "rate_limited"`; not a tool-level error |
| TV-BC-2.01.010-004 | First page fails immediately | Empty `events` array; error metadata populated; not a tool-level error for cross-client |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001, CAP-002 |
| L2 Invariants | DI-001 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.6 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-LOCAL-pass-2-MED-1b | 2026-08-13 | product-owner | **MED-1b closure: corrected stale RG-004 test citation (citation mis-anchor) to canonical name `test_fanout_all_failed_emits_fan_out_target_failed_warn`.** Two locations corrected: (1) §Postconditions AllTargetsFailed Per-Target Logging bullet (live contract); (2) v1.5 changelog row (factual correction — the prior mis-anchored name never existed as a test). Canonical test is `test_fanout_all_failed_emits_fan_out_target_failed_warn` in `mod fan_out_target_failed_warn_tests` in `crates/prism-sensors/src/fanout.rs`. Finding MED-1b from LOCAL adversary pass-2 for DEFECT-ADAPTER-TLS-XDOME-LIVE-001. TD-VSDD-097 9a: sibling-pair BC-2.16.002 corrected in same burst (v2.15→v2.16; stale citation fixed in catalog row `fan_out_target_failed` §SAP-1 obligation, §Postconditions AllTargetsFailed Per-Target Logging bullet, and v2.14 changelog row). 9b: no downstream artifact reproduces this postcondition bullet verbatim as a copy-source block. 9c: citation correction only — no new MUSTs introduced; no unanchored MUSTs. |
| 1.5 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001-spec-amendment | 2026-08-12 | product-owner | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (F10+F9) spec amendment: **new postcondition added** — when `SensorError::AllTargetsFailed` is about to be returned from `fanout()` in `crates/prism-sensors/src/fanout.rs`, each `FanOutError` in the `errors` vec MUST be logged at WARN with `event_type = "fan_out_target_failed"` (per BC-2.16.002 Canonical Structured Event Catalog row 91) before the error propagates. The E-SENSOR-030 Display remains count-only per BC-2.10.007 Rule 1 (E-SENSOR-* redacted at MCP boundary); per-target diagnostic detail is observable via the `fan_out_target_failed` event before `AllTargetsFailed` propagates. Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-004 + AC-SAP1-001 + RG-004 (`test_fanout_all_failed_emits_fan_out_target_failed_warn`). **TD-VSDD-097 three-dimension sweep:** 9a sibling-pair: BC-2.01.006 (Assets) and BC-2.01.018 (Alerts) are SS-01 siblings that also go through `fanout()` — the `fan_out_target_failed` event obligation applies uniformly to `AllTargetsFailed` regardless of which sensor adapter triggered it; no separate postcondition is needed in BC-2.01.006 or BC-2.01.018 because this BC governs the `AllTargetsFailed` failure-handling contract and those BCs govern their sensor-specific fetch success paths. 9b downstream copy-target: no current story reproduces BC-2.01.010 §Postconditions as a verbatim copy-source block. 9c mandate-anchor: new MUST anchored to DEFECT-ADAPTER-TLS-XDOME-LIVE-001 AC-ERR-004 + AC-SAP1-001 + RG-004 — no unanchored MUSTs. |
| 1.4 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Normalized capability frontmatter from YAML array to string scalar per corpus convention (IMP-006). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
