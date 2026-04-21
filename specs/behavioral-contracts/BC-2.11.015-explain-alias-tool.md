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
input-hash: "7948920"
traces_to: ["CAP-016"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.015: `explain_alias` MCP Tool

## Description

The `explain_alias` tool returns the full definition and recursively expanded form of a named alias, including the composition chain (ordered list of aliases expanded) and composition depth. Parse validation runs on the expanded query. The tool is read-only — no sensor API calls, no configuration modifications. An audit entry is emitted per DI-004. When scope is null, per-client alias takes precedence over global if a client context is available.

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

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `explain_alias(name="high_sev")` for simple alias | `composition_depth: 1`, `composition_chain: ["high_sev"]`, `expanded: "severity = 'high'"` | happy-path |
| `explain_alias(name="composed_alias")` for depth-2 composed alias | `composition_chain: ["composed_alias", "inner_alias"]`, depth=2 | happy-path |
| `explain_alias(name="nonexistent")` | `Err(E-ALIAS-001)` with available aliases | error |
| `explain_alias(name="parameterized_alias")` | Template shown with parameter placeholders and defaults | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-012 | Alias depth: rejects composition beyond depth 3 | kani |

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
