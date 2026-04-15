---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "Query Engine & Aliases"
capability: "CAP-016"
---

# BC-2.11.015: `explain_alias` MCP Tool

## Preconditions
- The `explain_alias` MCP tool is invoked with:
  - `name`: alias identifier (required)
  - `scope`: optional -- `"global"`, `"client:<client_id>"`, or null (resolves using default scope precedence: per-client overrides global)

## Postconditions
- Returns the alias definition with its fully expanded form after recursive alias resolution:
  - `name`, `scope`, `query` (raw template), `expanded` (fully expanded query), `parameters`, `description`
  - `composition_chain`: ordered list of aliases expanded during resolution (e.g., `["my_alias", "inner_alias"]`)
  - `composition_depth`: integer depth of the composition chain
- If `scope` is null, the alias is resolved using default precedence (per-client alias overrides global if a client context is available)
- Parse validation runs on the expanded query; parse errors are returned as structured errors
- An audit entry is emitted for the invocation (DI-004)

## Invariants
- DI-004: Audit completeness -- one AuditEntry emitted per invocation
- Read-only operation; no configuration is modified
- No sensor API calls are made

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-ALIAS-001` | Alias does not exist at the specified scope | Structured error with the alias name and available aliases |
| `E-ALIAS-002` | Alias resolution reveals a cycle (should not occur if create_alias validated correctly) | Structured error with the cycle chain |
| `E-ALIAS-003` | Alias composition depth exceeds 3 during expansion | Structured error with the composition chain |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-037 | Explaining a parameterized alias | Returns the template with parameter placeholders and their defaults |
| EC-11-038 | Explaining a global alias when a per-client override exists | If scope is explicit, returns the requested scope. If scope is null with client context, returns the per-client version. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 |
| L2 Invariants | DI-004 |
| Related BCs | BC-2.11.008 (create_alias), BC-2.11.009 (alias resolution) |
| Priority | P1 |
