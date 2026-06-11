---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-005"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "566def3"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.002: Runtime Per-Client TOML Feature Flag Configuration

## Description

The runtime tier of Prism's feature flag system resolves per-client capability sets from
`prism.toml` at startup. Global defaults from `[defaults.capabilities]` apply to all clients
and are overridden by per-client entries in `[clients.{id}.capabilities]`. The resolved
capability set for each client is a flat `HashSet<String>` of enabled paths computed once
at config load time and held immutable for the session.

This is the second tier of Prism's two-tier write gate (see BC-2.04.004): even when the
compile tier permits (registry-derived per BC-2.04.001 — matching `[[write_endpoints]]`
declaration in the sensor's TOML spec), the runtime TOML flag must also permit the
operation for the specific client making the request.

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.002.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| Global default only | `[defaults.capabilities]` allows `sensor.crowdstrike.read`; no per-client override | Client A resolves `sensor.crowdstrike.read: Allow`; write paths denied |
| Per-client override | Default denies writes; `[clients.acme.capabilities]` allows `sensor.crowdstrike.containment` | `acme` resolves `sensor.crowdstrike.containment: Allow`; other clients denied |
| Empty defaults | `[defaults.capabilities]` is empty; `[clients.acme.capabilities]` allows `sensor.crowdstrike.read` | `acme` has read; all other clients have zero capabilities |

## Verification Properties

- **VP-020** (Feature flag: compile AND runtime must both permit) — verifies the runtime tier is not bypassed even when the compile tier permits (registry-derived per BC-2.04.001).

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | MCP cascade pass-1 P1-02 BC sibling sweep (2026-06-10 review-cycle PO micro-burst) | 2026-06-10 | product-owner | Stale compile-tier phrasing reworded at two sites — Description "even when a write feature is compiled in" and VP-020 trailing description "even when compile-time feature is present" — both now "even when the compile tier permits (registry-derived per BC-2.04.001)": sensor-write compile tier is `[[write_endpoints]]`-declaration-derived, not cargo-feature-derived (error-taxonomy v1.67 E-FLAG-002 row). Runtime-tier contract content (TOML resolution, defaults/overrides, deny-by-default) unchanged. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
