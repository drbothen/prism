---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Audit & Compliance"
capability: "CAP-007"
---

# BC-2.05.009: Feature Flag Evaluations for Write Operations Are Audit-Logged

## Preconditions
- A write/mutation MCP tool is invoked
- The feature flag system evaluates the capability path for the target client

## Postconditions
- The audit entry's `capability_checks` array records each flag evaluated during hierarchical resolution
- Each entry includes: `capability_path`, `compile_time_enabled`, `runtime_enabled`, `result`
- The resolution chain is recorded in evaluation order (most-specific to least-specific, ending at the global deny default)
- The final `result` field reflects the deny-by-default outcome: `"permitted"` only if both compile-time and runtime tiers allow it

## Invariants
- DI-003: Feature flag deny-by-default
- DI-004: Audit completeness

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Compile-time feature absent | The cargo feature for the sensor's write capability is not compiled in | `capability_checks` records `compile_time_enabled: false` and `result: "denied"`; the tool is not registered and this path should not normally be reached |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-015 | A write tool is invoked for a client where the capability is enabled at `sensor.crowdstrike.write` but not at the more-specific `sensor.crowdstrike.containment` | The audit records both paths evaluated and the final resolution per the hierarchy rules |
| EC-05-016 | `list_capabilities` meta-tool invocation | This is a read operation; `capability_checks` is empty; the tool's response data (the capability matrix) is recorded in `result_summary` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-003, DI-004 |
| Priority | P0 |
