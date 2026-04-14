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
- The credential management MCP tools are registered (store_credential, get_credential_metadata, delete_credential, list_credentials)
- The analyst provides a valid `client_id`, `sensor_id`, and `credential_name`

## Postconditions
- `store_credential`: Stores a credential value in the active backend; returns success confirmation (never echoes the value)
- `get_credential_metadata`: Returns metadata (client, sensor, name, backend type, last_modified) but never the credential value
- `delete_credential`: Removes the credential from the backend; idempotent
- `list_credentials`: Returns all credential entries for a client/sensor combination (metadata only)
- All CRUD operations are audit-logged with client_id, sensor_id, credential_name, and operation type

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
| EC-03-012 | Agent attempts to read a credential value (not just metadata) | No MCP tool exposes credential values; `get_credential_metadata` returns metadata only |
| EC-03-013 | Credential value contains special characters (newlines, null bytes) | Stored as-is; the backend handles arbitrary byte sequences; value integrity preserved round-trip |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002, DI-004 |
| Priority | P0 |
