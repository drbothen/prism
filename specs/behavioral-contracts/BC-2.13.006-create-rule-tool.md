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

# BC-2.13.006: `create_rule` MCP Tool — Create Detection Rule with Scope

## Preconditions
- The `create_rule` MCP tool is invoked with required parameters: `source` (rule source in .detect format) and `scope` (one of: `global`, `client`, `analyst`)
- If `scope` is `client`: `client_id` parameter is required
- The `detection.write` capability is allowed (for `global` scope, `detection.write.global` is required). For `scope: global`, `detection.write` must be enabled for at least one client (same as hidden tools visibility rule). Rule creation fails with `E-FLAG-001` if no client has the capability enabled.
- The rule source passes parsing and security validation (BC-2.13.001)

## Postconditions
- The rule is parsed, validated, and stored per-scope (BC-2.13.011):
  - `global`: applies to all clients; stored in `rules:global:{rule_id}` in RocksDB
  - `client`: applies to a specific client; stored in `rules:client:{client_id}:{rule_id}`
  - `analyst`: applies only to the current analyst session; stored in memory only (not persisted across restarts)
- The detection engines (single-event, correlation, sequence) are rebuilt for affected scopes
  - **Stateful engine preservation:** in-progress correlation windows and sequence trackers are carried over during rebuild (not reset), unless the rebuilt rule changes the condition or window parameters
- Response includes: `rule_id`, `rule_name`, `rule_type` (single/correlation/sequence), `severity`, `scope`, `enabled`
- For `global` scope: confirmation token required (irreversible write affecting all clients, BC-2.04.009)
- For `client` scope: standard write gating (dry-run default, BC-2.04.008)
- For `analyst` scope: no confirmation required (session-scoped, non-persistent)
- An audit entry is emitted (DI-004)

## Invariants
- DI-004: Audit completeness
- A rule with the same identifier as an existing rule at the same scope replaces it (upsert semantics)
- Global rules cannot be overridden by create_rule with client scope using the same ID -- the client-scope rule coexists as an override (BC-2.13.011)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-RULE-001` through `E-RULE-005` | Rule parsing/validation failures | Structured error per BC-2.13.001 |
| `E-FLAG-001` | `detection.write` capability denied | Structured error (BC-2.04.015) |
| `E-RULE-006` | `scope: global` without `detection.write.global` capability | Structured error explaining elevated capability requirement |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-021 | Create rule with same ID as existing global rule at client scope | Both coexist; client-scope rule overrides global for that client (BC-2.13.011) |
| EC-13-022 | Create analyst-scope rule, then restart server | Rule is lost; analyst must re-create |
| EC-13-023 | Create global correlation rule while 100+ windows are active | Existing windows are preserved if rule condition and window parameters are unchanged |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-004 |
| Priority | P0 |
