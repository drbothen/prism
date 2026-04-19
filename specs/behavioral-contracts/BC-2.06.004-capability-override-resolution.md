---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
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

# BC-2.06.004: Capability Overrides Merge with Defaults Using More-Specific-Wins

## Preconditions
- TOML configuration includes a `[defaults.capabilities]` section and/or `[clients.{id}.capabilities]` sections
- The config loader is building the resolved `ClientCapability` set for each client

## Postconditions
- Each client's resolved capability set is the merge of `[defaults.capabilities]` and `[clients.{id}.capabilities]`
- More-specific client overrides take precedence over defaults
- The resolved set is stored as a `BTreeMap<String, Effect>` where `Effect` is `Allow` or `Deny`
- Resolution walks from the exact capability path upward through parent segments; the most-specific matching rule determines the effect
- At the same specificity level, `Deny` overrides `Allow`
- If a client has no `[clients.{id}.capabilities]` section, the defaults apply unchanged
- If no `[defaults.capabilities]` section exists, clients only have capabilities explicitly declared in their own section
- Deny-by-default: any capability path not matched by any rule in the resolved map is implicitly denied

## Invariants
- DI-003: Feature flag deny-by-default -- the fallback at every level of the hierarchy is deny

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | Capability path in config is syntactically invalid (e.g., empty string, leading/trailing dots) | Validation error: "Invalid capability path '{path}' in clients.{id}.capabilities" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-004 | Client overrides `sensor.crowdstrike.write: false` when default is `true` | Client-level override wins; CrowdStrike write operations are denied for this client |
| EC-06-005 | Client enables `sensor.crowdstrike.containment` but default denies `sensor.crowdstrike.write` | More-specific path (`containment`) is enabled; the resolution checks the most-specific matching path first |
| EC-06-006 | Both `[defaults.capabilities]` and `[clients.{id}.capabilities]` are absent | Client has zero capabilities; all write operations denied; only read operations available |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-003 |
| Priority | P0 |
