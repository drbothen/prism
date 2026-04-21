---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-016"
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
input-hash: "365fb25"
traces_to: ["CAP-016"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.014: `delete_alias` MCP Tool

## Description

The `delete_alias` tool removes an alias at a specified scope, always requiring a confirmation token before execution. Deletion is blocked when dependent aliases exist — the tool returns `E-ALIAS-005` listing dependents. The `force: true` flag enables cascade deletion of all dependents atomically. Persistence follows file-first ordering (same as create_alias): `aliases.toml` is atomically rewritten before the in-memory registry is updated. An audit entry is emitted per DI-004.

## Preconditions
- The `delete_alias` MCP tool is invoked with:
  - `name`: alias identifier (required)
  - `scope`: `"global"` or `"client:<client_id>"` (required)
  - `force`: optional boolean (default `false`); when `true`, cascade-deletes all dependent aliases
- The alias must exist at the specified scope
- The `alias.write` capability must be enabled (same precondition as `create_alias` — see BC-2.11.008)

## Postconditions
- Deletion always requires confirmation (write-operation gating per CAP-006 pattern):
  - A confirmation token is returned with an `action_summary` describing the alias to be deleted
  - If other aliases reference this alias, `dependent_aliases` lists them as a warning
  - The agent must call `confirm_action` to complete the deletion
- Upon confirmation: (1) validate removal against in-memory state (check dependents unless `force: true`), (2) write `aliases.toml` atomically with the alias removed (temp file + fsync + rename, same pattern as credential state files). If the file write fails, the operation fails entirely with no partial state — the in-memory registry is unchanged. (3) THEN update the in-memory alias registry (remove alias). Cycle/depth validation of remaining aliases runs after removal. This file-first ordering ensures no divergence between in-memory and on-disk state.
- Deletion is BLOCKED when dependent aliases exist. The tool returns a structured error listing the dependent aliases. The analyst must delete dependents first, or use the `force: true` parameter for cascade deletion (all dependents are removed atomically in the same write).
- **Cascade deletion re-resolves dependents at confirmation time.** When `force: true` is used and the agent calls `confirm_action`, the system re-resolves the current dependent alias set at that moment. If new dependents appeared since the confirmation token was generated, the confirmation still succeeds but the response includes the updated dependent list (all dependents are deleted). The audit entry logs all deleted aliases (both the target and all cascade-deleted dependents).
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
| EC-11-041 | File write fails during `aliases.toml` atomic write (deletion) | Operation fails entirely; in-memory registry is unchanged (alias still exists). Error returned to caller with `E-IO-001` and suggestion to retry. No partial state is possible because file write precedes in-memory update. |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `delete_alias(name="high_sev", scope="global")` | Returns confirmation token; alias removed after confirm_action | happy-path |
| `delete_alias(name="alias_with_deps", scope="global")` | `Err(E-ALIAS-005)` with dependent list | error |
| `delete_alias(name="alias_with_deps", scope="global", force=true)` | Confirmation token returned; cascade deletion on confirm | edge-case |
| `delete_alias(name="nonexistent", scope="global")` | `Err(E-ALIAS-001)` | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| — | No specific VP; covered by alias depth/cycle VPs at creation | — |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 |
| L2 Invariants | DI-004 |
| Related BCs | BC-2.11.008 (create_alias), BC-2.11.009 (alias resolution) |
| Priority | P1 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
