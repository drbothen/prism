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

# BC-2.05.008: Audit Entries Satisfy SOC 2 Type II and ISO 27001 Requirements

## Preconditions
- An MCP tool invocation has completed (success or failure)
- The audit middleware is constructing the final `AuditEntry`

## Postconditions
- **SOC 2 Type II** fields are present:
  - **Who**: `user_identity` identifies the analyst
  - **What**: `tool_name` and `parameters` (redacted) describe the action
  - **When**: `timestamp` records the time in ISO 8601 UTC
  - **Where**: `client_id` and `sensor` scope the action to a specific client and sensor
  - **Outcome**: `result_summary` records success, failure, or denial
  - **Authorization**: `capability_checks` records feature flag evaluations for write operations
- **ISO 27001** fields are present:
  - Access control evidence: `capability_checks` demonstrates least-privilege enforcement
  - Incident response support: `trace_id` enables correlation of events across a session
  - Credential access: credential operations include `event_type: "credential_access"` (per BC-2.05.005)
- All fields are machine-parseable (structured JSON, not free-text prose)

## Invariants
- DI-004: Audit completeness
- DI-003: Feature flag deny-by-default -- audit trail proves least-privilege enforcement

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Missing `user_identity` | MCP session does not provide user identity | `user_identity` is set to `"unknown"` with an `audit_warning` noting the missing identity; the entry is still emitted |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-013 | Read-only tool invocation (no capability check needed) | `capability_checks` array is empty (not omitted); this is valid -- read ops do not require authorization evidence |
| EC-05-014 | Tool invocation that triggers multiple capability checks (e.g., a write that falls back through the flag hierarchy) | All evaluated capability paths are recorded in the `capability_checks` array, showing the full resolution chain |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-003, DI-004 |
| Priority | P0 |
