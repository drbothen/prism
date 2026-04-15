---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Case Management"
capability: "CAP-022"
---

# BC-2.14.003: `update_case` MCP Tool — Transition State, Set Disposition, Add Annotation

## Preconditions
- The `update_case` MCP tool is invoked with required parameters: `case_id` (UUID), `client_id`
- Optional parameters (at least one required): `status` (target state for transition), `disposition` (TruePositive/FalsePositive/Benign/Inconclusive with per-variant metadata), `severity` (update severity), `assignee` (update assignee), `annotation` (object with `type` and `content`), `link_alert_ids` (array of additional alert IDs to link)
- The `case.write` capability is allowed
- The case exists and belongs to the specified `client_id`

## Postconditions
- Each provided field is applied in order: disposition -> status -> severity -> assignee -> annotation -> link_alert_ids
- **Disposition update:** stored on the case; generates a `DispositionSet` timeline entry
- **Status transition:** validated against the state machine (BC-2.14.002); generates a `StatusChanged` timeline entry
- **Severity update:** generates a `PriorityChanged` timeline entry (using "priority" terminology for axiathon compatibility)
- **Assignee update:** generates a timeline entry with previous and new assignee
- **Annotation:** added to the case's annotations array; generates an `AnnotationAdded` timeline entry; annotation type must be one of the 5 valid types (BC-2.14.007)
- **Link alert IDs:** each alert is validated (exists, belongs to client), deduplicated against existing links, and added; generates an `AlertLinked` timeline entry per new alert
- `updated_at` is set to current UTC timestamp
- The updated case is persisted to RocksDB (BC-2.14.009)
- An audit entry is emitted (DI-004)
- The `update_case` tool is gated by `case.write` capability and follows the hidden-tools pattern (BC-2.04.005)

## Invariants
- DI-004: Audit completeness
- DI-008: Client data separation -- case must belong to the specified client_id
- Order of operations matters: disposition is set before status transition so that the Resolved transition's disposition requirement (BC-2.14.002) can be satisfied in a single call

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CASE-007` | Case does not exist | Structured error |
| `E-CASE-008` | Case belongs to a different client than specified | Structured error |
| `E-CASE-004` | Invalid state transition | Structured error per BC-2.14.002 |
| `E-CASE-001` | `link_alert_ids` contains invalid alert ID | Structured error; no partial updates (entire call fails) |
| `E-CAP-001` | `case.write` capability denied | Structured error (BC-2.04.015) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-009 | Set disposition to FalsePositive and transition to Resolved in one call | Disposition set first, then transition succeeds (disposition requirement satisfied) |
| EC-14-010 | Update with no optional parameters provided | Structured error: "At least one update field is required" |
| EC-14-011 | Set assignee to null (unassign) | Valid; generates timeline entry "Unassigned from {previous_assignee}" |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P1 |
