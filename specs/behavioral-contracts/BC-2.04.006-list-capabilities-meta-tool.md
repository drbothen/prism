---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
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

# BC-2.04.006: list_capabilities Meta-Tool for Capability Discovery

## Preconditions
- The `list_capabilities` MCP tool is always registered (not gated by any feature flag)
- The caller provides an optional `client_id` parameter

## Postconditions
- Returns a complete capability matrix showing all possible tools and their enablement status
- For each capability path, reports:
  - `enabled: bool` (the combined result of both tiers)
  - `compile_time: bool` (whether the cargo feature is present in the binary)
  - `runtime: bool` (whether the runtime TOML flag permits it for this client)
  - `reason: String` (human-readable explanation when disabled, e.g., "Feature not compiled (crowdstrike-write)" or "Not enabled in client config")
- If `client_id` is provided, shows capabilities for that specific client
- If `client_id` is null, shows capabilities for all clients in a per-client breakdown

## Invariants
- `list_capabilities` is always available regardless of feature flags
- The reported status is consistent with what `tools/list` shows

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | Provided `client_id` not found | Structured error: "Client '{id}' not found in configuration" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-012 | Agent calls `list_capabilities` with no client context | Returns global capability matrix showing all clients; useful for "which clients can I contain hosts for?" queries |
| EC-04-013 | Binary built with zero write features | All write capabilities show `compile_time: false, enabled: false` with reason "Feature not compiled" |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
