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

# BC-2.11.014: `delete_alias` MCP Tool

## Preconditions
- The `delete_alias` MCP tool is invoked with:
  - `name`: alias identifier (required)
  - `scope`: `"global"` or `"client:<client_id>"` (required)
  - `force`: optional boolean (default `false`); when `true`, cascade-deletes all dependent aliases
- The alias must exist at the specified scope

## Postconditions
- Deletion always requires confirmation (write-operation gating per CAP-006 pattern):
  - A confirmation token is returned with an `action_summary` describing the alias to be deleted
  - If other aliases reference this alias, `dependent_aliases` lists them as a warning
  - The agent must call `confirm_action` to complete the deletion
- Upon confirmation, the alias is removed from `aliases.toml` via atomic write (temp file + fsync + rename, same pattern as credential state files)
- Deletion is BLOCKED when dependent aliases exist. The tool returns a structured error listing the dependent aliases. The analyst must delete dependents first, or use the `force: true` parameter for cascade deletion (all dependents are removed atomically in the same write).
- An audit entry is emitted for the invocation (DI-004)

## Invariants
- DI-004: Audit completeness -- one AuditEntry emitted per invocation (and a second for the confirm_action)
- Deletion is a write operation; confirmation token required

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-ALIAS-001` | Alias does not exist at the specified scope | Structured error with the alias name and scope |
| `E-ALIAS-005` | Alias has dependent aliases and `force` is not `true` | Structured error listing the dependent aliases; analyst must delete dependents first or use `force: true` for cascade deletion |
| `E-CFG-001` | `scope` references a client ID that does not exist | Structured error listing valid client IDs |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-035 | Deleting a global alias that is overridden by per-client aliases | Per-client overrides remain; only the global alias is removed |
| EC-11-036 | Deleting an alias that is referenced by another alias | Deletion is BLOCKED with `E-ALIAS-005` listing dependents. Use `force: true` for cascade deletion or delete dependents individually first. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 |
| L2 Invariants | DI-004 |
| Related BCs | BC-2.11.008 (create_alias), BC-2.11.009 (alias resolution) |
| Priority | P1 |
