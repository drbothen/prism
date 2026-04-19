---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-12"
capability: "CAP-018"
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

# BC-2.12.007: `get_diff_results` MCP Tool — Retrieve Differential Results for a Scheduled Query

## Preconditions
- The `get_diff_results` MCP tool is invoked with required parameter: `schedule_name`
- Optional parameters: `client_id` (filter to one client), `since_epoch` (return results from this epoch onward), `limit` (max result sets to return, default 10, max 100)

## Postconditions
- Returns an array of `DiffResults` entries from the RocksDB `diff_results` domain, ordered by epoch descending (most recent first)
- Each entry contains: `schedule_name`, `client_id`, `epoch`, `counter`, `timestamp`, `added` (array of OCSF records), `removed` (array of OCSF records), `query_execution_time_ms`
- If `client_id` is provided, only that client's results are returned
- If `since_epoch` is provided, only results with `epoch >= since_epoch` are returned
- Results respect the `limit` parameter; if more results exist, `is_truncated: true` and `oldest_epoch_returned` are included
- An audit entry is emitted for the tool invocation (DI-004)
- This is a read-only tool -- always visible in `tools/list`

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry emitted
- DI-008: Client data separation -- results are scoped to configured clients only

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SCHED-001` | No schedule with given `schedule_name` exists (and no orphaned results) | Structured error |
| `E-MCP-004` | `client_id` is not a valid configured client | Structured error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-019 | Schedule exists but has never executed | Empty array, not an error |
| EC-12-020 | All executions produced identical results (no diffs) | Empty array (no DiffResults stored for no-change epochs) |
| EC-12-021 | Schedule was deleted but results remain (orphaned) | Results are still returned; response includes `schedule_deleted: true` annotation |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
