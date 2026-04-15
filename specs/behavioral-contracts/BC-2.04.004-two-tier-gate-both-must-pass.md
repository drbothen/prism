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

# BC-2.04.004: Two-Tier Gate -- Both Compile-Time and Runtime Must Permit Operation

## Preconditions
- A write operation tool is being registered or invoked
- The binary was built with or without the corresponding cargo feature
- The client has or does not have the runtime capability enabled

## Postconditions
- Tool registration check: `#[cfg(feature = "sensor-write")]` gate must pass at compile time AND `client_capabilities.is_enabled("sensor.{sensor}.{operation}")` must return `true` at runtime
- If compile-time feature is absent, the tool code does not exist in the binary (no runtime check possible)
- If compile-time feature is present but runtime flag denies, the tool is not registered for that client
- Both tiers produce a clear reason when they block: compile-time ("Feature not compiled") vs runtime ("Not enabled in client config")

## Invariants
- DI-003: Deny-by-default at both tiers

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A (tool absent) | Compile-time feature disabled | Tool does not exist; agent cannot invoke it; `list_capabilities` reports "Feature not compiled" |
| N/A (tool hidden) | Runtime flag disabled | Tool exists in binary but not registered for this client; `list_capabilities` reports "Not enabled in client config" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-008 | Compile-time enabled, runtime enabled for Client A, disabled for Client B | Tool visibility is determined per-invocation based on the `client_id` parameter — there is no session-level "active client". A tool call with `client_id: "client_a"` sees the tool available; a subsequent call with `client_id: "client_b"` does not. Different `client_id` values in successive tool calls may see different tool availability based on per-client capability configuration. No `notifications/tools/list_changed` is sent because tool registration is static (all compile-time-enabled tools are registered); runtime gating is evaluated at invocation time. |
| EC-04-009 | All write features compiled in but all runtime flags deny | Binary has write code but no client can use it; effectively read-only deployment with latent write capability |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
