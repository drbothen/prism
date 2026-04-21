---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "dc078d2"
traces_to: ["CAP-020"]
extracted_from: ".factory/specs/prd.md"
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
---

# BC-2.13.006: `create_rule` MCP Tool — Create Detection Rule with Scope

## Description

The `create_rule` MCP tool accepts a detection rule in `.detect` format and persists it at the requested scope (`global`, `client`, or `analyst`). It validates the rule source via BC-2.13.001, enforces the active rule cap (DI-028) before storage, and rebuilds the affected detection engines while preserving in-progress correlation windows. An audit entry is emitted for every successful creation (DI-004).

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
- DI-028: Rule capacity cap enforced -- before activating a new rule, the detection engine checks the count of active (non-deleted) rules across all scopes against `max_rules` (default 1000, configurable via `[defaults.limits].max_rules`). If the count is at or above the cap, the rule is rejected with `E-RULE-011` and is never stored or registered with the detection engine.
- A rule with the same identifier as an existing rule at the same scope replaces it (upsert semantics)
- Global rules cannot be overridden by create_rule with client scope using the same ID -- the client-scope rule coexists as an override (BC-2.13.011)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-RULE-011` | Active rule count is at or above `max_rules` cap (default 1000) | Structured error with `current_count`, `max_count`, and suggestion to delete unused rules; rule is never stored or activated (DI-028) |
| `E-RULE-001` through `E-RULE-005` | Rule parsing/validation failures | Structured error per BC-2.13.001 |
| `E-FLAG-001` | `detection.write` capability denied | Structured error (BC-2.04.015) |
| `E-RULE-006` | `scope: global` without `detection.write.global` capability | Structured error explaining elevated capability requirement |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-021 | Create rule with same ID as existing global rule at client scope | Both coexist; client-scope rule overrides global for that client (BC-2.13.011) |
| EC-13-022 | Create analyst-scope rule, then restart server | Rule is lost; analyst must re-create |
| EC-13-023 | Create global correlation rule while 100+ windows are active | Existing windows are preserved if rule condition and window parameters are unchanged |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `create_rule(source="<valid single-event rule>", scope="analyst")` | `{rule_id, rule_name, rule_type: "single", severity, scope: "analyst", enabled: true}` | happy-path |
| `create_rule(source="<valid rule>", scope="client", client_id="c1")` | Rule stored at `rules:client:c1:{rule_id}`; confirmation not required | happy-path |
| `create_rule(source="<valid rule>", scope="global")` | Requires confirmation token; stored at `rules:global:{rule_id}` after confirmation | happy-path |
| `create_rule(source="<valid rule>", scope="analyst")` when active rule count == `max_rules` | `Err(E-RULE-011)` with `current_count` and `max_count`; rule not stored | error |
| `create_rule(source="<invalid syntax rule>", scope="analyst")` | `Err(E-RULE-001..E-RULE-005)` per BC-2.13.001 | error |
| `create_rule(source="<valid rule>", scope="global")` without `detection.write.global` | `Err(E-RULE-006)` | error |
| `create_rule` with same rule ID as existing rule at same scope | Rule replaced (upsert); existing correlation windows preserved if condition/window unchanged | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-030 | Rule store rejects create when active rule count >= `max_rules` cap (DI-028) | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-004, DI-024, DI-028 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 1.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.3 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: standardized inputs/input-hash/traces_to/extracted_from frontmatter to Wave 4 convention; version bump 1.2 → 1.3. |
| 1.2 | burst-41 | 2026-04-19 | product-owner | P3P39-A-OBS-001: added DI-024 to L2 Invariants; added missing template sections (Description, Canonical Test Vectors, Verification Properties) and frontmatter fields to satisfy hook |
| 1.1 | deferred-cleanup-track-1 | 2026-04-19 | product-owner | Added DI-028 cap-check invariant, E-RULE-011 error case |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
