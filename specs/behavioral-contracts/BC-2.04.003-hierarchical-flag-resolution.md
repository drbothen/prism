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

# BC-2.04.003: Hierarchical Capability Resolution (Most-Specific to Least-Specific with Deny Fallback)

## Preconditions
- A capability check is requested for a dot-separated path (e.g., `sensor.crowdstrike.containment`)
- The client's resolved `ClientCapabilities` `HashSet<String>` is available

## Postconditions
- Resolution checks for the exact path first: `sensor.crowdstrike.containment`
- If no exact match, walks up the hierarchy: `sensor.crowdstrike.write`, `sensor.crowdstrike`, `sensor.write`, `sensor`
- The first match in the hierarchy wins (returns `true`)
- If no path in the hierarchy matches any entry in the `HashSet`, returns `false` (deny)
- The resolution is deterministic: same input always produces the same result

## Invariants
- DI-003: Final fallback is always deny (`false`)
- Resolution is pure (no I/O, no side effects)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Empty capability path string | Returns `false` (denied); no match possible |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-005 | `sensor.crowdstrike` is enabled but `sensor.crowdstrike.containment` is explicitly not in the set | `sensor.crowdstrike` (parent) matches during hierarchy walk; containment is allowed because the parent is enabled |
| EC-04-006 | Only `sensor.write` is enabled (broad grant) | All sensor write operations match via hierarchy walk; equivalent to enabling all write operations |
| EC-04-007 | Capability path with 4+ levels (e.g., `sensor.crowdstrike.rtr.execute`) | Hierarchy walk checks: exact, `sensor.crowdstrike.rtr`, `sensor.crowdstrike`, `sensor`; all levels checked |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
