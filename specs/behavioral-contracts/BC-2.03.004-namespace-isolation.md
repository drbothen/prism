---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Credential Management"
capability: "CAP-004"
---

# BC-2.03.004: Credential Namespace Isolation by (client_id, sensor_id, credential_name)

## Preconditions
- A credential operation (get, set, delete, list) is invoked

## Postconditions
- The credential is uniquely identified by the 3-tuple `(client_id, sensor_id, credential_name)`
- Two clients with the same sensor type have completely independent credential storage
- The namespace prevents Client A's CrowdStrike `client_secret` from being accessible when operating as Client B

## Invariants
- DI-002: Credential isolation per client -- accessing Client A's credentials while operating in Client B's context is a type error
- The `TenantId` type parameter enforces isolation at compile time

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Compile error | Code attempts credential access without providing `TenantId` | Rust type system rejects the call |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-009 | Same `credential_name` used across different sensors for the same client | Independent storage; `sensor_id` differentiates them |
| EC-03-010 | Client ID contains valid but unusual characters (e.g., `client-with-dashes`) | Valid per `[a-zA-Z0-9_-]` pattern; stored correctly in both keyring and file backends |
| EC-03-011 | Cross-client credential query during `client_id: null` fan-out | Each fan-out leg resolves credentials for its own `client_id`; no cross-tenant credential access |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |
