---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "67e5667"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.001: Every MCP Tool Invocation Produces Exactly One Audit Entry (Fail-Closed for Writes)

## Description

The audit middleware ensures that every MCP tool invocation — whether it succeeds or fails —
produces exactly one `AuditEntry` as a structured JSON log event. Write operations are
fail-closed with respect to audit: if audit emission fails for any write operation (including
confirmation token generation, credential mutation, or confirmed action execution), the write
is aborted and `E-AUDIT-001` is returned. The write is never executed without a successful
audit record. Read operations are fail-open: a tracing failure during read audit produces a
`_meta.audit_warning` in the response but does not block the operation.

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.001.

| Scenario | Tool Type | Audit Failure? | Expected Behavior |
|----------|-----------|---------------|-------------------|
| Normal write | `crowdstrike_contain_host` | No | Audit entry emitted; token or execution proceeds |
| Write with audit failure | Any write tool | Yes | Write aborted; `E-AUDIT-001` returned; no write executed |
| Normal read | `query_crowdstrike_alerts` | No | Audit entry emitted; results returned |
| Read with audit failure | Any read tool | Yes | Results returned; `_meta.audit_warning` set |

## Verification Properties

- **VP-033** (Audit buffer: RocksDB write completes before delivery attempt) — verifies ordering of audit persistence relative to execution for the DTU CrowdStrike clone.

No VP in VP-INDEX v1.5 directly covers the general fail-closed write behavior. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004 |
| Addresses | ADV-2-009 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
| 1.1 | Phase 1 | 2026-04-14 | product-owner | Previous version |
