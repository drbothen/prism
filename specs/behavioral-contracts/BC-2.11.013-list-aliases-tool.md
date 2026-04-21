---
document_type: behavioral-contract
level: L3
version: "1.3"
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
input-hash: "e5de7f9"
traces_to: ["CAP-016"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.013: `list_aliases` MCP Tool

## Description

The `list_aliases` tool returns alias definitions filtered by optional scope (global, client-specific, or all). Unlike credentials, alias names and query templates are not sensitive — cross-scope listing is permitted (scope: null returns all). Results are sorted alphabetically by name within each scope group. An audit entry is emitted per DI-004. The tool is always visible in `tools/list` as a read-only operation.

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

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `list_aliases(scope=null)` with 3 global and 2 client aliases | All 5 aliases returned, sorted by name | happy-path |
| `list_aliases(scope="global")` | Only global aliases | happy-path |
| `list_aliases(scope="client:nonexistent")` | `Err(E-CFG-001)` | error |
| `list_aliases(scope=null)` with no aliases | Empty array | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| — | No specific VP; covered by audit invariant tests | — |

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
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
