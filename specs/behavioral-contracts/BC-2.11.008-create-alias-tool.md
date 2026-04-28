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
input-hash: "412c872"
traces_to: ["CAP-016"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.008: `create_alias` MCP Tool

## Description

The `create_alias` tool creates or updates a named PrismQL shorthand at either global or per-client scope. Alias creation is immediate for new names; updating an existing alias requires a confirmation token (write-gating per CAP-006). The alias query template is validated through the Chumsky parser at creation time. Composition constraints (max depth 3, no cycles) are enforced via DI-020. Persistence follows a file-first pattern: `aliases.toml` is atomically written before the in-memory registry is updated, ensuring no divergence between disk and memory on failure.

## Preconditions
- The `create_alias` MCP tool is invoked with:
  - `name`: alias identifier (required, must match `[a-zA-Z_][a-zA-Z0-9_]*`)
  - `scope`: `"global"` or `"client:<client_id>"` (required)
  - `query`: the PrismQL expression or template string (required)
  - `parameters`: optional map of parameter names to default values (if parameterized)
  - `description`: optional human-readable description
- If `scope` is `"client:<client_id>"`, the client must exist in configuration
- The `alias.write` capability must be enabled (compile-time cargo feature + runtime TOML). For client-scoped aliases, the capability is checked against the target client. For global alias creation, `alias.write` must be enabled for at least one configured client (visibility check). The operation is authorized if any single client's capability set includes `alias.write = Allow`. This is consistent with the hidden tools pattern (tools/list shows the tool if any client allows it).

## Postconditions
- If the alias name does not exist at the specified scope, the alias is created immediately
- If the alias name already exists at the specified scope, this is treated as an update:
  - A confirmation token is returned (write-operation gating per CAP-006 pattern)
  - The ConfirmationToken `client_id` is derived from the `scope` parameter: for `scope: "client:<client_id>"`, the token's `client_id` is set to the extracted `<client_id>`; for `scope: "global"`, the token's `client_id` is set to the sentinel value `"__global__"`. The agent must call `confirm_action` with the matching `client_id` (including `"__global__"` for global-scope aliases).
  - The agent must call `confirm_action` to complete the update
- The alias query template is validated by parsing it through the Chumsky parser (with parameter placeholders treated as valid tokens)
- If parameterized, all parameters must have defaults specified
- **Persistence order (file-first):** (1) Validate the proposed alias against the current in-memory state (cycle/depth checks via DI-020, keyword conflicts, parse validation). (2) Write `aliases.toml` atomically (temp file + fsync + rename, same pattern as credential state files). If the file write fails, the operation fails entirely with no partial state — the in-memory registry is unchanged. (3) THEN update the in-memory alias registry. This ordering ensures the persisted file is always the source of truth and no divergence can occur between in-memory and on-disk state.
- Alias composition validation runs: if the new alias references other aliases, depth is checked (max 3) and cycles are detected
- Response includes the created/updated alias definition and its expanded form

## Invariants
- DI-020: Composition depth and cycle detection validated before accepting the alias
- Alias names must not conflict with PrismQL keywords (`SELECT`, `FROM`, `WHERE`, `AND`, `OR`, `NOT`, etc.)
- Alias names must not match known OCSF field names (e.g., `severity`, `activity_name`, `src_endpoint`, `dst_endpoint`, `device`, `actor`, etc.). The reserved name list is derived from the OCSF protobuf descriptor loaded at startup.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-MCP-004` | Alias name contains invalid characters (not matching `[a-zA-Z_][a-zA-Z0-9_]*`) | Structured error with the name and allowed pattern |
| `E-QUERY-001` | Alias query template is not valid PrismQL | Parse error with position and suggestion |
| `E-CFG-001` | Client ID in scope does not exist | Structured error listing valid client IDs |
| `E-ALIAS-004` | Parameter value fails type validation (not a simple literal) | Structured error listing the invalid parameter and expected format |
| `E-ALIAS-003` | New alias creates composition depth > 3 | Error with the alias chain that exceeds depth |
| `E-ALIAS-002` | New alias creates a cycle | Error with the exact cycle chain |
| `E-ALIAS-006` | Alias name conflicts with a reserved OCSF field name or PrismQL keyword | Structured error listing the conflicting name and whether it is a keyword or OCSF field |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-021 | Creating a per-client alias with the same name as a global alias | Valid; per-client alias overrides global for that client |
| EC-11-022 | Deleting an alias that is referenced by another alias | `delete_alias` is BLOCKED with `E-ALIAS-005` listing dependents; use `force: true` for cascade deletion |
| EC-11-040 | File write fails during `aliases.toml` atomic write | Operation fails entirely; in-memory registry is unchanged. Error returned to caller with `E-IO-001` and suggestion to retry. No partial state is possible because file write precedes in-memory update. |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `create_alias(name="high_sev", scope="global", query="severity = 'high' OR severity = 'critical'")` | Alias created; response includes definition and expanded form | happy-path |
| `create_alias(name="high_sev", scope="global", query="...")` when alias already exists | Returns confirmation token with action_summary | happy-path |
| `create_alias(name="SELECT", scope="global", query="severity = 'high'")` | `Err(E-ALIAS-006)` keyword conflict | error |
| `create_alias(name="a", scope="global", query="b")` where `b` is depth-3 composed alias | `Err(E-ALIAS-003)` depth exceeded | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-012 | Alias depth: rejects composition beyond depth 3 | kani |
| VP-013 | Alias cycles: detects and rejects cyclic references | proptest |
| VP-037 | Alias expansion: never panics on arbitrary alias graphs | fuzz |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 |
| L2 Invariants | DI-020 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
