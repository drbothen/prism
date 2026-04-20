---
document_type: behavioral-contract
level: L3
version: "1.1"
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
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "365fb25"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.010: `case_metrics` MCP Tool — Aggregate MTTD/MTTR and Case Status Counts

## Description

The `case_metrics` MCP tool provides cross-case aggregate metrics for MSSP operational
reporting. It computes average MTTD and MTTR over resolved cases in a specified time
window, counts cases by current status, and identifies the top clients by open case
volume. Metrics computation is purely read-only and deterministic for the same case set.

Results include `_meta.trust_level: "internal"` since all data originates from Prism's
internal RocksDB state rather than external sensor APIs. Null values are returned when
no qualifying data exists; zero is never used to represent "no data."

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

## Error Conditions
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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — single client | `client_id="acme", since=null` | MTTD/MTTR averages over all resolved acme cases; status counts |
| No resolved cases | all cases in New/Investigating | `mttd_avg=null, mttr_avg=null` |
| Cross-client | `client_id=null` | Aggregates all clients; top_clients_by_open_cases populated |
| Invalid since | `since="not-a-date"` | `E-MCP-004` |
| Invalid client | `client_id="nonexistent"` | `E-CFG-001` |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify MTTD/MTTR null when no resolved cases |
| (placeholder) | VP to be assigned — verify cross-client aggregation correctness |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
