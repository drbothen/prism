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

# BC-2.11.008: `create_alias` MCP Tool

## Preconditions
- The `create_alias` MCP tool is invoked with:
  - `name`: alias identifier (required, must match `[a-zA-Z_][a-zA-Z0-9_]*`)
  - `scope`: `"global"` or `"client:<client_id>"` (required)
  - `query`: the AxiQL expression or template string (required)
  - `parameters`: optional map of parameter names to default values (if parameterized)
  - `description`: optional human-readable description
- If `scope` is `"client:<client_id>"`, the client must exist in configuration

## Postconditions
- If the alias name does not exist at the specified scope, the alias is created immediately
- If the alias name already exists at the specified scope, this is treated as an update:
  - A confirmation token is returned (write-operation gating per CAP-006 pattern)
  - The agent must call `confirm_action` to complete the update
- The alias query template is validated by parsing it through the Chumsky parser (with parameter placeholders treated as valid tokens)
- If parameterized, all parameters must have defaults specified
- The alias is stored in the TOML config (written to disk)
- Alias composition validation runs: if the new alias references other aliases, depth is checked (max 3) and cycles are detected
- Response includes the created/updated alias definition and its expanded form

## Invariants
- DI-020: Composition depth and cycle detection validated before accepting the alias
- Alias names must not conflict with AxiQL keywords (`SELECT`, `FROM`, `WHERE`, `AND`, `OR`, `NOT`, etc.)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-MCP-004` | Alias name contains invalid characters | Structured error with the name and allowed pattern |
| `E-MCP-004` | Alias name conflicts with AxiQL keyword | Structured error listing the conflicting keyword |
| `E-QUERY-001` | Alias query template is not valid AxiQL | Parse error with position and suggestion |
| `E-CFG-001` | Client ID in scope does not exist | Structured error listing valid client IDs |
| `E-ALIAS-004` | Parameterized alias missing defaults for some parameters | Structured error listing parameters without defaults |
| `E-ALIAS-003` | New alias creates composition depth > 3 | Error with the alias chain that exceeds depth |
| `E-ALIAS-002` | New alias creates a cycle | Error with the exact cycle chain |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-021 | Creating a per-client alias with the same name as a global alias | Valid; per-client alias overrides global for that client |
| EC-11-022 | Deleting an alias that is referenced by another alias | `delete_alias` requires confirmation token; error message warns about dependent aliases |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 |
| L2 Invariants | DI-020 |
| Priority | P1 |
