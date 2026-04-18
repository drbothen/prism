---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Audit Trail"
capability: "CAP-007"
---

# BC-2.05.001: Every MCP Tool Invocation Produces Exactly One Audit Entry (Fail-Closed for Writes)

## Preconditions
- An MCP tool invocation is dispatched through the tool dispatch middleware
- The `tracing` subscriber is initialized

## Postconditions
- Exactly one `AuditEntry` is emitted as a structured JSON log event via `tracing::info!`
- The entry is emitted regardless of whether the tool invocation succeeds or fails
- Successful invocations include `result_summary` with outcome details
- Failed invocations include `result_summary` with the error category and message
- **Write operations fail-closed on audit failure**: if audit emission fails for a write operation (including confirmation token generation, credential mutation, or confirmed action execution), the write operation is aborted and a structured error `E-AUDIT-001` is returned. The write is never executed without a successful audit record.
- **Read operations proceed on audit failure**: if audit emission fails for a read-only operation, the operation still proceeds. A warning `_meta.audit_warning: "audit emission failed"` is included in the response.

## Invariants
- DI-004: Audit completeness -- every MCP tool invocation produces exactly one AuditEntry
- Write operations are fail-closed with respect to audit: no unaudited writes

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Audit` | Tracing subscriber failure during write operation audit | Write operation aborted; structured error: `code: "E-AUDIT-001"`, `message: "Audit emission failed; write operation blocked"`, `category: "transient"`, `retryable: true`, `suggestion: "Retry the operation. If the error persists, check tracing subscriber health."` |
| Warning | Tracing subscriber failure during read operation audit | Read operation proceeds; `_meta.audit_warning: "audit emission failed"` set on response |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-014 | Tracing subscriber encounters an error during audit emission for a write operation | Write operation is aborted; `E-AUDIT-001` error returned; the write is never executed |
| EC-05-001 | Tool invocation panics before audit middleware completes | Panic is caught by the MCP transport layer; an audit entry is still emitted with `result_summary: "panic"` if the middleware uses a catch-unwind guard |
| EC-05-002 | Audit emission fails for a read-only query | Query proceeds and returns results; response includes `_meta.audit_warning` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004 |
| Addresses | ADV-2-009 |
| Priority | P0 |
