---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-13"
capability: "CAP-020"
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
traces_to: ["CAP-020"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.13.007: `list_rules` MCP Tool — List Active Rules by Scope

## Description

The `list_rules` tool returns all detection rules visible in the current context, optionally filtered by scope, client_id, rule_type, and enabled_only. When a `client_id` is provided, the effective rule set after three-scope resolution (BC-2.13.011) is returned with each rule annotated by `effective_scope`. Rules are sorted scope-first (global, client, analyst) then by name. An audit entry is emitted per DI-004. The tool is always visible in `tools/list` as a read-only operation.

## Preconditions
- The `list_rules` MCP tool is invoked
- Optional parameters: `scope` (filter to `global`, `client`, or `analyst`), `client_id` (when scope is `client`, list rules for that client), `rule_type` (filter to `single`, `correlation`, or `sequence`), `enabled_only` (boolean, default true)

## Postconditions
- Returns an array of rule summaries, each containing: `rule_id`, `rule_name`, `rule_type`, `severity`, `scope`, `enabled`, `mitre_technique` (optional), `description` (from meta), `source` (original .detect source), `created_at`
- If no scope filter: returns all rules visible to the current context (global + all client-scoped + analyst-scoped)
- If `client_id` is provided: returns the effective rule set for that client after three-scope resolution (BC-2.13.011) -- global baseline merged with client overrides, annotated with which scope each rule came from
- Rules are sorted by scope (global first, then client, then analyst), then by rule_name
- An audit entry is emitted (DI-004)
- This is a read-only tool -- always visible in `tools/list`

## Invariants
- DI-004: Audit completeness
- DI-008: Client-scoped rules are only visible when the matching `client_id` is specified or when listing all scopes

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-MCP-004` | `client_id` is not a valid configured client | Structured error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-024 | No rules exist at any scope | Empty array, not an error |
| EC-13-025 | Global rule and client rule have the same ID | Both listed; client rule annotated as `overrides: "global:{rule_id}"` |
| EC-13-026 | Analyst-scope rule exists but is disabled | Included only if `enabled_only: false` |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `list_rules()` with 5 global and 2 client rules | 7 rules sorted: global first, then client | happy-path |
| `list_rules(client_id="acme")` | Effective set after three-scope resolution; each rule annotated | happy-path |
| `list_rules(client_id="nonexistent")` | `Err(E-MCP-004)` | error |
| `list_rules()` with no rules | Empty array | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-030 | Schedule/rule count caps: rejects beyond limits (at creation) | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
