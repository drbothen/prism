---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Detection Engine"
capability: "CAP-020"
---

# BC-2.13.007: `list_rules` MCP Tool — List Active Rules by Scope

## Preconditions
- The `list_rules` MCP tool is invoked
- Optional parameters: `scope` (filter to `global`, `client`, or `analyst`), `client_id` (when scope is `client`, list rules for that client), `rule_type` (filter to `single`, `correlation`, or `sequence`), `enabled_only` (boolean, default true)

## Postconditions
- Returns an array of rule summaries, each containing: `rule_id`, `rule_name`, `rule_type`, `severity`, `scope`, `enabled`, `mitre_technique` (optional), `description` (from meta), `source` (original .axd source), `created_at`
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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
