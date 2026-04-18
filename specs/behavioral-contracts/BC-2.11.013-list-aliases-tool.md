---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "Query Execution"
capability: "CAP-016"
---

# BC-2.11.013: `list_aliases` MCP Tool

## Preconditions
- The `list_aliases` MCP tool is invoked with:
  - `scope`: optional filter -- `"global"`, `"client:<client_id>"`, or null for all aliases
  - **Note:** Aliases are scoped by `scope` parameter (global/client:id), not `client_id`, because global aliases have no client association. This differs from most other tools which use `client_id` for scoping.

## Postconditions
- Returns all aliases matching the scope filter, including: name, scope, query template, parameters (if parameterized), and description
- If `scope` is null, returns both global aliases and all per-client aliases
- If `scope` is `"client:<client_id>"`, returns only aliases defined for that client (does not include global aliases)
- If `scope` is `"global"`, returns only global aliases
- Results are sorted alphabetically by name within each scope group
- An audit entry is emitted for the invocation (DI-004)

## Invariants
- DI-004: Audit completeness -- one AuditEntry emitted per invocation
- Read-only operation; no configuration is modified
- Unlike `list_credentials`, `list_aliases` permits cross-scope listing (scope: null returns all aliases) because alias names and query templates are not considered sensitive client data. Aliases are reusable query shortcuts, not secrets.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CFG-001` | `scope` references a client ID that does not exist | Structured error listing valid client IDs |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-033 | No aliases defined anywhere | Empty aliases array, not an error |
| EC-11-034 | `scope: "client:acme"` but no per-client aliases for acme | Empty aliases array, not an error |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 |
| L2 Invariants | DI-004 |
| Related BCs | BC-2.11.008 (create_alias), BC-2.11.009 (alias resolution) |
| Priority | P1 |
