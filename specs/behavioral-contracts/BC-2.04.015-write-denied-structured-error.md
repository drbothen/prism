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

# BC-2.04.015: Structured Error When Write Capability Is Denied

## Preconditions
- A write operation is attempted for a client where the capability is not enabled
- This can occur if the agent calls a tool that exists in the binary but is not registered for the current client (race condition or direct JSON-RPC call bypassing tool list)

## Postconditions
- The response is a structured error (not a generic "unknown tool" error) containing:
  - `code: "CAPABILITY_DENIED"`
  - `capability`: the path that was checked (e.g., `sensor.crowdstrike.containment`)
  - `client_id`: the client context
  - `reason`: "Not enabled in client config" or "Feature not compiled"
  - `suggestion`: actionable guidance (e.g., "Enable 'sensor.crowdstrike.containment' in [clients.acme.capabilities] and restart Prism")
- The error is audit-logged as a denied capability check

## Invariants
- DI-003: Denied operations produce actionable errors, not silent failures

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Structured error | Runtime flag denies the capability | Error with TOML path to enable and restart instruction |
| Structured error | Compile-time feature absent | Error explaining that the binary must be rebuilt with the feature flag |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-032 | Agent receives denial and asks "how do I enable this?" | The structured error already contains the exact config path and action needed |
| EC-04-033 | Capability path partially matches (e.g., `sensor.crowdstrike` enabled but `sensor.crowdstrike.containment` specifically denied) | If the capability system supports explicit deny entries, the deny wins; otherwise, parent match enables child |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
