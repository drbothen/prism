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

# BC-2.05.005: Credential Access Events Are Audit-Logged with Context

## Preconditions
- A credential is accessed (read, write, delete) via the `CredentialStore` trait
- The access is performed in the context of a specific `client_id` and `sensor_id`

## Postconditions
- A structured log event is emitted recording:
  - `event_type: "credential_access"`
  - `operation` (`"get"`, `"set"`, `"delete"`, `"list"`)
  - `client_id` (the `TenantId` of the credential being accessed)
  - `sensor_id` (the sensor the credential belongs to)
  - `credential_name` (the name, e.g., `"api_key"`, `"client_secret"`)
  - `result` (`"success"` or `"error"` with category)
  - `timestamp` (ISO 8601 UTC)
- The credential value itself is NEVER present in the log event

## Invariants
- DI-002: Credential isolation per client -- credential values never logged
- DI-004: Audit completeness -- all credential access is audit-logged

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Credential not found | `get()` called for a non-existent credential | Log event includes `result: "not_found"` with the credential name and context |
| Backend error | OS keyring locked or file backend I/O failure | Log event includes `result: "error"` with category but no backend-specific secrets |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-008 | Credential `list()` operation for a client | Log event records `operation: "list"`, `credential_name: "*"` (wildcard), and the count of credentials returned |
| EC-05-009 | Credential rotation (`set()` overwriting existing) | Log event records `operation: "set"` with no distinction between create and update; the old value is not logged |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-002, DI-004 |
| Priority | P0 |
