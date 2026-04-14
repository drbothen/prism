---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Feature Flag System"
capability: "CAP-005"
---

# BC-2.04.013: Feature Flag Evaluation Audit Logging for Write Operations

## Preconditions
- A write operation is about to be evaluated (either at tool registration or at invocation time)

## Postconditions
- A `CapabilityCheckEvent` is emitted via `tracing::info!` with structured fields:
  - `event_type: "capability_check"`
  - `client_id`: the tenant
  - `capability`: the checked path (e.g., `sensor.crowdstrike.containment`)
  - `result`: "allowed" | "denied"
  - `tool_name`: the MCP tool that triggered the check
  - `denied_reason` (if denied): "Feature not compiled" or "Not enabled in client config" or "No matching capability path"
  - `timestamp`: UTC
- Read operations do not emit capability check events (they are always allowed)

## Invariants
- DI-004: Audit completeness -- every write capability check is logged
- DI-003: The audit trail proves deny-by-default behavior

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Tracing subscriber fails | Capability check still proceeds; best-effort stderr warning |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-027 | Cross-client query triggers capability checks for 10 clients | 10 separate capability check events emitted (one per client) |
| EC-04-028 | Capability denied at compile-time tier | Event still emitted with `result: "denied"` and `denied_reason: "Feature not compiled"` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003, DI-004 |
| Priority | P0 |
