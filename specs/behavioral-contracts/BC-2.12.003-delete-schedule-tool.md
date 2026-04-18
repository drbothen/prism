---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Scheduler"
capability: "CAP-017"
---

# BC-2.12.003: `delete_schedule` MCP Tool — Remove a Schedule (Confirmation Required)

## Preconditions
- The `delete_schedule` MCP tool is invoked with required parameter: `schedule_id` (schedule identifier)
- The `schedule.write` capability is allowed for the invoking context
- A schedule with the given `schedule_id` exists

## Postconditions
- If invoked without a valid `confirmation_token`: returns a preview of the schedule to be deleted (name, query, interval, targeted clients, total differential results that will be orphaned) plus a `ConfirmationToken` with 300s expiry (BC-2.04.009)
- If invoked with a valid `confirmation_token` (via `confirm_action`): the schedule is removed from the RocksDB `schedules` domain, its splay offsets are removed, its epoch counters are removed, and its differential result history is retained (orphaned but queryable via `get_diff_results`)
- The schedule is deregistered from the execution loop; any in-flight execution for this schedule is allowed to complete but no new executions are started
- An audit entry is emitted for the tool invocation (DI-004)
- The `delete_schedule` tool is gated by `schedule.write` capability and follows the hidden-tools pattern (BC-2.04.005)

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry emitted
- Confirmation required: deletion is an irreversible write operation requiring the two-step confirmation flow (BC-2.04.009, BC-2.04.010)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SCHED-001` | No schedule with given `name` exists | Structured error |
| `E-FLAG-001` | `schedule.write` capability denied | Structured error (BC-2.04.015) |
| `E-FLAG-008` | Confirmation token expired or invalid | Structured error with recovery guidance (BC-2.04.011) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-007 | Delete while an execution is in-flight | In-flight execution completes; results are stored; schedule is not re-queued |
| EC-12-008 | Delete a schedule referenced by a pack | Warning in response noting pack reference; deletion proceeds (pack will skip the missing schedule) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-004 |
| Priority | P0 |
