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
capability: "CAP-006"
---

# BC-2.04.008: Dry-Run Default for Reversible Write Operations

## Preconditions
- A reversible write tool is invoked (e.g., `acknowledge_alert`, `update_alert_status`)
- The tool input struct has a `dry_run: bool` field with default value `true`

## Postconditions
- When `dry_run: true` (default): the tool simulates the operation and returns a preview showing what would change, without making any API call to the sensor
- When `dry_run: false` (explicit): the tool executes the actual write operation against the sensor API
- The response clearly indicates whether it was a dry run: `_meta.dry_run: true|false`
- Dry-run responses include: the target entity, the proposed change, and a confirmation prompt for the agent

## Invariants
- Default is always `true`; the agent must explicitly opt in to execution
- Dry-run mode never modifies any state in the sensor or in Prism

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Dry-run passes but actual execution fails (e.g., permission denied at the sensor) | Structured error on the `dry_run: false` call; the dry-run success does not guarantee execution success |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-016 | Sensor API does not support dry-run natively | Prism simulates the dry-run by validating parameters and checking permissions without calling the sensor API |
| EC-04-017 | Agent sends `dry_run: false` on first call (skipping preview) | Allowed; the dry-run default is a suggestion, not a hard gate; the operation executes immediately |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-003 |
| Priority | P1 |
