---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Health"
capability: "CAP-008"
---

# BC-2.08.007: Partial Health Status (Mixed Sensor Availability)

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-004 |
| L2 Edge Cases | DEC-004 |
| Priority | P1 |
