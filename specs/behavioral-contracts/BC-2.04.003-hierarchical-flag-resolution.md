---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-005"
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
input-hash: "5b48b9c"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.003: Hierarchical Capability Resolution (BTreeMap, Most-Specific-Path Wins, Deny Support)

## Description

Capability checks use a deterministic hierarchical resolution algorithm over a
`BTreeMap<String, Effect>`. For a capability path such as `sensor.crowdstrike.containment`,
resolution checks for the most-specific matching entry first (exact match), then walks
up the hierarchy (`sensor.crowdstrike`, then `sensor`). The first matching entry wins;
`Deny` at a more-specific level overrides `Allow` at a parent level. If no path in the
hierarchy matches any entry, the final fallback is `Deny` (deny-by-default per DI-003).

The algorithm is pure: no I/O, no side effects, and same inputs always produce the same
result. The `BTreeMap` ensures sorted key order for predictable iteration and debugging.

## Preconditions
- A capability check is requested for a dot-separated path (e.g., `sensor.crowdstrike.containment`)
- The client's resolved capabilities are stored as a `BTreeMap<String, Effect>` where `Effect` is `Allow` or `Deny`

## Postconditions
- Resolution checks for the most-specific matching path first, then walks up the hierarchy to less-specific paths
- Resolution order for `sensor.crowdstrike.containment`: exact match, then `sensor.crowdstrike`, then `sensor`
- The most-specific path that has an entry wins. If the most-specific entry is `Deny`, the capability is denied even if a parent path is `Allow`.
- `Deny` at the same level or a more specific level always overrides `Allow` at a less specific level. This enables patterns like: allow `sensor.crowdstrike` but deny `sensor.crowdstrike.containment`.
- If no path in the hierarchy matches any entry in the `BTreeMap`, the final fallback is `Deny` (deny-by-default)
- The resolution is deterministic: same input always produces the same result
- The `BTreeMap` ensures sorted key order for predictable iteration and debugging

## Invariants
- DI-003: Final fallback is always deny
- Resolution is pure (no I/O, no side effects)
- Most-specific path wins; deny at same or more specific level overrides allow at parent

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Empty capability path string | Returns `Deny`; no match possible |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-005 | `sensor.crowdstrike` is `Allow` but `sensor.crowdstrike.containment` is `Deny` | `sensor.crowdstrike.containment` (more specific) wins; containment is denied even though the parent is allowed |
| EC-04-006 | Only `sensor` is `Allow` (broad grant), no more-specific deny entries | All sensor operations match via hierarchy walk; equivalent to enabling all sensor operations |
| EC-04-007 | Capability path with 4+ levels (e.g., `sensor.crowdstrike.rtr.execute`) | Hierarchy walk checks: exact, `sensor.crowdstrike.rtr`, `sensor.crowdstrike`, `sensor`; most-specific match wins |
| EC-04-032 | `sensor.crowdstrike` is `Deny` but `sensor.crowdstrike.read` is `Allow` | `sensor.crowdstrike.read` (more specific) wins; read is allowed despite parent deny |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.003.

| Scenario | BTreeMap | Path Checked | Expected Result |
|----------|----------|-------------|----------------|
| Deny-by-default | empty | `sensor.crowdstrike.containment` | `Deny` |
| Parent allow, child absent | `{sensor.crowdstrike: Allow}` | `sensor.crowdstrike.containment` | `Allow` (inherited from parent) |
| Parent allow, child deny | `{sensor.crowdstrike: Allow, sensor.crowdstrike.containment: Deny}` | `sensor.crowdstrike.containment` | `Deny` (more-specific wins) |
| Exact match wins | `{sensor.crowdstrike: Deny, sensor.crowdstrike.read: Allow}` | `sensor.crowdstrike.read` | `Allow` (more-specific wins) |

## Verification Properties

- **VP-002** (Capability resolution: deny-by-default) — verifies the fallback is always `Deny` when no path matches.
- **VP-003** (Capability resolution: most-specific-path wins) — verifies the hierarchy walk returns the most-specific match.
- **VP-004** (Capability resolution: deny overrides allow at same specificity) — verifies `Deny` wins at the same specificity level.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Addresses | ADV-2-001 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
| 1.1 | Phase 1 | 2026-04-14 | product-owner | Previous version |
