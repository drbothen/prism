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

# BC-2.12.002: `list_schedules` MCP Tool — List Active Schedules with Next Run Times

## Preconditions
- The `list_schedules` MCP tool is invoked
- Optional parameters: `client_id` (filter to schedules targeting a specific client), `enabled_only` (boolean, default true)

## Postconditions
- Returns an array of schedule summaries, each containing: `name`, `query` (original PrismQL string), `interval`, `splay_percent`, `enabled`, `snapshot`, `removed`, `clients` (targeted client IDs), `last_run` (map of client_id to last execution timestamp or null), `next_run` (map of client_id to next scheduled execution), `epoch` (map of client_id to current epoch counter), `created_at`
- If `client_id` is provided, only schedules that target that client are returned, and `last_run`/`next_run`/`epoch` maps contain only that client's entry
- Schedules are sorted by `next_run` (earliest first) for the first targeted client
- An audit entry is emitted for the tool invocation (DI-004)
- This is a read-only tool -- always visible in `tools/list`, not gated by write capabilities

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry emitted
- DI-008: Client data separation -- when `client_id` is specified, only that client's timing metadata is returned

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-MCP-004` | `client_id` is not a valid configured client | Structured error with rejected value |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-004 | No schedules exist | Empty array, not an error |
| EC-12-005 | Schedule exists but is disabled | Included only if `enabled_only: false` |
| EC-12-006 | Schedule targets client that was removed from config since creation | Schedule listed with warning annotation; `next_run` for removed client shows `null` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
