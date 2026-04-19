---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-14"
capability: "CAP-022"
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

# BC-2.14.010: `case_metrics` MCP Tool — Aggregate MTTD/MTTR and Case Status Counts

## Preconditions
- The `case_metrics` MCP tool is invoked with optional parameters: `client_id` (string or null for cross-client), `since` (ISO8601 timestamp or null for all time)
- Cases exist in the RocksDB Cases domain (BC-2.14.009)

## Postconditions
- The tool aggregates metrics over all resolved cases within the specified time window (cases with `created_at >= since`):
  - **`mttd_avg`**: Mean Time to Detect — average of per-case TTD values (BC-2.14.008) for resolved cases with non-null TTD. Null if no qualifying cases.
  - **`mttr_avg`**: Mean Time to Resolve — average of per-case TTR values (BC-2.14.008) for resolved cases with non-null TTR. Uses `resolved_at - created_at` (not `closed_at`). Null if no qualifying cases.
  - **`cases_by_status`**: Counts of cases grouped by current status (New, Acknowledged, Investigating, Resolved, Closed) within the time window.
  - **`top_clients_by_open_cases`**: Top 10 clients ranked by count of non-closed cases (status in New, Acknowledged, Investigating). Each entry includes `client_id`, `open_count`, and `oldest_open_case_age`.
  - **`total_cases`**: Total number of cases matching the filter criteria.
- If `client_id` is specified, only cases for that client are included. If null, all clients are aggregated.
- If `since` is null, all cases are included regardless of creation time.
- An audit entry is emitted for the tool invocation (DI-004)
- Response includes `_meta.trust_level: "internal"` (no external sensor data)

## Invariants
- Metrics computation is read-only — does not modify any case state
- Aggregate metrics (MTTD, MTTR averages) are deterministic for the same case set and time window
- All duration values are non-negative (floored at 0)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CFG-001` | `client_id` is not a valid configured client | Structured error |
| `E-MCP-004` | Invalid `since` timestamp format | Structured error with expected ISO8601 format |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-054 | No cases exist for the specified client/time window | All metric values are null; `cases_by_status` has zero counts; `total_cases` is 0 |
| EC-14-055 | `client_id: null` with 50+ clients | Cross-client aggregation; `top_clients_by_open_cases` shows top 10 by open count |
| EC-14-035 | Cases exist but none are resolved | `mttd_avg` and `mttr_avg` are null; `cases_by_status` shows counts in non-resolved statuses |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
