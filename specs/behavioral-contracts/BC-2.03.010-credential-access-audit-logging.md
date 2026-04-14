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

# BC-2.03.010: Credential Access Audit Logging

## Preconditions
- Any credential store operation (get, set, delete, list) is invoked

## Postconditions
- A `tracing::info!` structured log entry is emitted with:
  - `event_type: "credential_access"`
  - `operation`: "get" | "set" | "delete" | "list"
  - `client_id`: the tenant ID
  - `sensor_id`: the sensor
  - `credential_name`: the credential key name
  - `backend`: "keyring" | "encrypted_file"
  - `result`: "success" | "not_found" | "error"
  - `timestamp`: UTC
- The credential value is NEVER included in the log entry
- Failed access attempts are logged with the same detail level as successful ones

## Invariants
- DI-004: Audit completeness -- every credential operation is logged
- DI-002: Credential values never in audit entries

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Tracing subscriber fails to emit the log | Credential operation still proceeds; best-effort stderr warning |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-025 | Rapid successive credential reads (e.g., fan-out query resolves credentials for 10 clients) | Each credential read produces its own audit entry; no batching or deduplication of audit logs |
| EC-03-026 | Credential operation during startup (before tracing subscriber fully initialized) | Audit entry buffered or emitted to stderr; startup credential operations must still be auditable |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002, DI-004 |
| Priority | P0 |
