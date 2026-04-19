---
document_type: behavioral-contract
level: L3
version: "1.1"
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
---

# BC-2.04.003: Hierarchical Capability Resolution (BTreeMap, Most-Specific-Path Wins, Deny Support)

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Addresses | ADV-2-001 |
| Priority | P0 |
