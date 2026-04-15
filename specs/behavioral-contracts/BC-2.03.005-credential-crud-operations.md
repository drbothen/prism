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

# BC-2.03.005: Credential CRUD Operations via MCP Tools

## Preconditions
- The credential management MCP tools are registered (set_credential, delete_credential, list_credentials)
- The analyst provides a valid `client_id`, `sensor_id`, and `credential_name`
- Credential mutation tools (`set_credential`, `delete_credential`) are subject to feature flag gating under capability path `credential.write`. They follow the hidden-tools pattern (BC-2.04.005): if `credential.write` is denied for ALL configured clients, `set_credential` and `delete_credential` are omitted from `tools/list`. If denied for a specific client, invocation with that `client_id` returns `E-FLAG-001`. `list_credentials` is a read-only tool and is always visible regardless of feature flags.

## Postconditions
- `set_credential` (create): When no credential exists for the given `(client_id, sensor_id, credential_name)` tuple, the credential is created immediately and returns `status: "created"`. No confirmation token is required for initial creation — the operation is non-destructive (nothing is being overwritten).
- `set_credential` (update/overwrite): When a credential already exists, overwriting is gated behind the confirmation token flow (same as irreversible write operations per BC-2.04.009) — the tool returns a `ConfirmationToken` with `status: "confirmation_required"` and the caller must call `confirm_action` to execute the overwrite. This prevents accidental credential replacement.
- `delete_credential`: Removes the credential from the backend; idempotent. Deletion is gated behind the confirmation token flow (same as irreversible write operations per BC-2.04.009) — the tool returns a `ConfirmationToken` and the caller must call `confirm_action` to execute the deletion.
- `list_credentials`: Returns all credential entries for a client/sensor combination (metadata only, never credential values)
- All CRUD operations are audit-logged with client_id, sensor_id, credential_name, and operation type
- Credential values are never included in MCP responses, logs, or error messages

## Invariants
- DI-002: Credential isolation
- DI-004: Audit completeness -- every credential operation produces an audit entry
- Credential values are never included in MCP responses, logs, or error messages

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `credential_name` fails validation | Structured error with the rejected name and the allowed pattern `[a-zA-Z0-9_\-\.]+` |
| `PrismError::Credential` | Backend write fails | Structured error with backend type and suggestion |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-012 | Agent attempts to read a credential value (not just metadata) | No MCP tool exposes credential values; `list_credentials` returns metadata only (backend type, last_modified) but never the credential value itself |
| EC-03-013 | Credential value contains special characters (newlines, null bytes) | Stored as-is; the backend handles arbitrary byte sequences; value integrity preserved round-trip |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004, CAP-005 |
| L2 Invariants | DI-002, DI-003, DI-004 |
| Addresses | ADV-5-001 |
| Priority | P0 |
