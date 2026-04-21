---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-12"
capability: "CAP-017"
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
input-hash: "e5de7f9"
traces_to: ["CAP-017"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.12.003: `delete_schedule` MCP Tool — Remove a Schedule (Confirmation Required)

## Description

The `delete_schedule` tool permanently removes a named schedule, requiring a two-step confirmation (BC-2.04.009) before any deletion occurs. Without a confirmation token the tool returns a preview of what will be deleted; with a valid token it removes the schedule from RocksDB, deregisters it from the execution loop, and retains orphaned differential result history queryable via `get_diff_results`. In-flight executions complete before deregistration takes effect. Gated by `schedule.write` capability.

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

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `delete_schedule(schedule_id="hourly_alerts")` (no token) | Preview + confirmation token returned | happy-path |
| `confirm_action` with valid token from above | Schedule removed from RocksDB; diff history retained | happy-path |
| `delete_schedule(schedule_id="nonexistent")` | `Err(E-SCHED-001)` | error |
| `confirm_action` with expired token | `Err(E-FLAG-008)` | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-007 | Confirmation token expiry: expired at boundary (inclusive) | kani |
| VP-008 | Confirmation token: single-use enforcement | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
