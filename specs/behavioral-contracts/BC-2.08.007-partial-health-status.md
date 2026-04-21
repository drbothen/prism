---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "5b48b9c"
traces_to: ["CAP-008"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-08"
capability: "CAP-008"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.007: Partial Health Status (Mixed Sensor Availability)

## Description

When a client has multiple sensors and some are unavailable, the `check_sensor_health` tool returns a successful response containing per-sensor health entries for all sensors — healthy and unhealthy alike. The prose summary includes "N of M sensors healthy"; the `structuredContent` includes a `summary` object with counts. Unhealthy sensor entries include a `suggestion` field. Even when all sensors are unhealthy, the tool response is a success (not an error).

## Preconditions
- A client has multiple sensors configured (e.g., CrowdStrike + Claroty + Armis)
- A health check is requested for all sensors (`sensor_id: null`)
- Some sensors are healthy and some are not

## Postconditions
- The tool returns a successful response (not an error) containing per-sensor health entries
- Healthy sensors show `reachable: true`, `auth_valid: true`
- Unhealthy sensors show their specific failure reason (unreachable, auth invalid, rate limited)
- The prose summary in `content[].text` includes a count: "2 of 3 sensors healthy"
- The `structuredContent` includes a `summary` object with `healthy_count`, `unhealthy_count`, `total_count`
- Each unhealthy sensor entry includes a `suggestion` field guiding resolution

## Invariants
- DI-004: Audit completeness -- one AuditEntry for the entire health check invocation, not per-sensor

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | All sensors unhealthy | Still a successful tool response with all sensors marked unhealthy; not a tool-level error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-013 | One sensor's health check hangs (timeout) while others complete quickly | The timed-out sensor reports `reachable: false`, `reason: "timeout"` while healthy sensors report normally. Total tool response time bounded by the longest sensor timeout. |
| EC-08-014 | Client has only one sensor, and it is unhealthy | Returns single-entry health array; `summary.healthy_count: 0, unhealthy_count: 1` |
| DEC-004 | Client configured with zero sensors | Returns empty health array with message "no sensors configured"; `summary.total_count: 0` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Client with 3 sensors, 2 healthy 1 unreachable | Success response; prose "2 of 3 sensors healthy"; `summary.healthy_count: 2, unhealthy_count: 1` | happy-path |
| All sensors unhealthy | Success response (not error); all entries show failure reasons and suggestions | edge-case |
| One sensor times out, others complete | Timed-out sensor `reachable: false, reason: "timeout"`; others normal | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | Partial health never raises tool-level error; one AuditEntry per invocation | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-004 |
| L2 Edge Cases | DEC-004 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
