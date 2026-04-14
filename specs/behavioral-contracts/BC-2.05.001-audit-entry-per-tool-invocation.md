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

# BC-2.05.001: Every MCP Tool Invocation Produces Exactly One Audit Entry

## Preconditions
- An MCP tool invocation is dispatched through the tool dispatch middleware
- The `tracing` subscriber is initialized

## Postconditions
- Exactly one `AuditEntry` is emitted as a structured JSON log event via `tracing::info!`
- The entry is emitted regardless of whether the tool invocation succeeds or fails
- Successful invocations include `result_summary` with outcome details
- Failed invocations include `result_summary` with the error category and message

## Invariants
- DI-004: Audit completeness -- every MCP tool invocation produces exactly one AuditEntry

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Tracing subscriber failure | stderr pipe broken or subscriber error | Tool invocation still proceeds; best-effort warning to stderr; response includes `_meta.audit_warning: "audit emission may have failed"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-014 | Tracing subscriber encounters an error during audit emission for a write operation | Tool completes; audit completeness is compromised but operation is not rolled back; `_meta.audit_warning` field is set on the response |
| EC-05-001 | Tool invocation panics before audit middleware completes | Panic is caught by the MCP transport layer; an audit entry is still emitted with `result_summary: "panic"` if the middleware uses a catch-unwind guard |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004 |
| Priority | P0 |
