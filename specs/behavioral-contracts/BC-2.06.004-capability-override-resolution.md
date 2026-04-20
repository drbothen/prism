---
document_type: behavioral-contract
level: L3
version: "1.1"
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
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.004: Capability Overrides Merge with Defaults Using More-Specific-Wins

## Description

The config loader builds each client's resolved capability set by merging
`[defaults.capabilities]` with `[clients.{id}.capabilities]`. More-specific client
overrides take precedence over defaults, following the same hierarchical resolution
algorithm as BC-2.04.003. The resolved set is stored as a `BTreeMap<String, Effect>`.
Deny-by-default applies: any capability path not matched by any rule in the resolved map
is implicitly denied. If neither section is present for a client, the client has zero
capabilities.

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.004.

| Scenario | Defaults | Client Override | Resolved Result for Client |
|----------|----------|-----------------|---------------------------|
| Client inherits defaults | `sensor.crowdstrike.read: Allow` | None | `sensor.crowdstrike.read: Allow` |
| Client overrides deny | `sensor.crowdstrike.write: Allow` | `sensor.crowdstrike.write: Deny` | `sensor.crowdstrike.write: Deny` |
| More-specific client enable | Default denies `sensor.crowdstrike.write` | `sensor.crowdstrike.containment: Allow` | `containment: Allow`; parent `write` still denied |
| Neither section present | None | None | Zero capabilities; all write denied |

## Verification Properties

- **VP-002** (Capability resolution: deny-by-default) — verifies the fallback is always `Deny` when no path matches, including in the merged per-client map.
- **VP-003** (Capability resolution: most-specific-path wins) — verifies client overrides at more-specific paths take precedence over defaults at parent paths.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-003 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
