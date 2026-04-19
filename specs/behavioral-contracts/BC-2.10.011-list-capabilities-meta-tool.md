---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-10"
capability: "CAP-005"
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

# BC-2.10.011: list_capabilities Meta-Tool

## Preconditions
- The `list_capabilities` tool is always registered (not gated by feature flags)
- The tool accepts `client_id: Option<String>` (required)

## Postconditions
- When `client_id` is provided: returns the full capability matrix for that client, showing all capabilities with their status:
  - `enabled`: both compile-time and runtime flags permit the capability
  - `runtime_disabled`: compile-time feature present but runtime TOML denies it
  - `compile_time_disabled`: cargo feature not compiled in
- When `client_id` is null: returns per-client capability summaries for all clients
- Response includes the hierarchical capability path (e.g., `sensor.crowdstrike.containment`) and whether it resolves to enabled or denied
- Response includes the resolution chain showing which level of the hierarchy determined the result
- Response uses `trust_level: "internal"` (capability data is Prism-generated)
- Tool annotations: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: false`

## Invariants
- DI-003: Feature flag deny-by-default -- the capability matrix reflects deny-by-default semantics

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Invalid `client_id` format | Structured error with validation details |
| `PrismError::Config` | `client_id` not found | Structured error with suggestion |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-021 | Client with zero capabilities enabled | Returns full matrix with all capabilities showing `runtime_disabled` or `compile_time_disabled` |
| EC-10-022 | All write features compiled in but all runtime-disabled | Matrix shows all write capabilities as `runtime_disabled` with TOML paths for enabling |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
