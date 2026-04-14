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

# BC-2.05.002: Audit Entries Use Structured JSON Format with Complete Fields

## Preconditions
- An MCP tool invocation has been dispatched
- The audit middleware is constructing an `AuditEntry`

## Postconditions
- The emitted audit entry is valid JSON containing all required fields:
  - `timestamp` (ISO 8601 UTC)
  - `trace_id` (unique per invocation)
  - `client_id` (the `TenantId` from the tool call, or `"cross_client"` for `client_id: null`)
  - `tool_name` (the MCP tool name, e.g., `query_crowdstrike_alerts`)
  - `parameters` (the tool input parameters as JSON, with secrets redacted)
  - `user_identity` (the analyst identity from the MCP session)
  - `result_summary` (success/failure outcome)
  - `capability_checks` (array of capability evaluations, may be empty for read ops)
  - `safety_flags` (array of triggered prompt injection flags, may be empty)
- The JSON structure is compatible with the existing Vector pipeline for ingestion

## Invariants
- DI-004: Audit completeness -- no field is omitted even if the value is empty or null

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Missing `client_id` | Tool call lacks `client_id` entirely (malformed request) | Audit entry records `client_id: "MISSING"` and the tool returns `PrismError::InvalidInput` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-002 | Cross-client query (`client_id: null`) | Audit entry records `client_id: "cross_client"` to distinguish from single-client invocations |
| EC-05-003 | Tool name is a meta-tool (`list_capabilities`) | Audit entry is still emitted with the same field completeness requirements |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004 |
| Priority | P0 |
