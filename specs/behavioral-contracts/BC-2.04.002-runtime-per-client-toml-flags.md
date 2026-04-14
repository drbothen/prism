---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Feature Flag System"
capability: "CAP-005"
---

# BC-2.04.002: Runtime Per-Client TOML Feature Flag Configuration

## Preconditions
- `prism.toml` contains `[defaults.capabilities]` and optionally `[clients.{id}.capabilities]` sections
- Configuration is loaded at startup

## Postconditions
- Global defaults from `[defaults.capabilities]` apply to all clients
- Per-client overrides in `[clients.{id}.capabilities]` merge with and override defaults
- The resolved capability set per client is a flat `HashSet<String>` of enabled capability paths
- Capabilities are resolved at config load time (not on every check)
- Config changes require server restart (no hot-reload in stdio per-analyst model)

## Invariants
- DI-003: Deny-by-default -- if no flag explicitly enables a capability, it is denied
- More-specific flags override less-specific flags

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | Invalid capability path syntax in TOML | Multi-error validation: all problems reported in one pass with exact TOML paths |
| `PrismError::Config` | Unknown capability path (typo) | Warning logged but not a fatal error; the typo'd path simply never matches any capability check |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-006 | Config changed while Prism is running | Running session uses startup-time capabilities; restart required for changes to take effect |
| EC-04-003 | Client has no `[capabilities]` section | Inherits all defaults; if defaults deny writes, client is read-only |
| EC-04-004 | `[defaults.capabilities]` is empty | All capabilities denied for all clients unless explicitly enabled per-client |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
