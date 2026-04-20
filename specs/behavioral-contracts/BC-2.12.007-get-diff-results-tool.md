---
document_type: behavioral-contract
level: L3
version: "1.1"
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
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "[pending-recompute]"
traces_to: ["CAP-018"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.12.007: `get_diff_results` MCP Tool — Retrieve Differential Results for a Scheduled Query

## Description

The `get_diff_results` tool retrieves persisted differential results for a named schedule from the RocksDB `diff_results` domain. Results are returned most-recent-first, optionally filtered by client_id and since_epoch. The limit parameter (max 100) prevents unbounded reads. Results from deleted (orphaned) schedules are still accessible with a `schedule_deleted: true` annotation. An audit entry is emitted per DI-004.

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

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `get_diff_results(schedule_name="hourly_alerts")` with 3 past executions | 3 DiffResults entries, most-recent first | happy-path |
| `get_diff_results(schedule_name="hourly_alerts", client_id="acme")` | Only acme's results | happy-path |
| `get_diff_results(schedule_name="nonexistent")` | `Err(E-SCHED-001)` | error |
| `get_diff_results(schedule_name="deleted_schedule")` with orphaned data | Results returned with `schedule_deleted: true` | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-019 | Diff computation: deterministic | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |

## Changelog
| Version | Date | Burst | Change |
|---------|------|-------|--------|
| 1.0 | 2026-04-13 | cycle-1 | Initial contract |
| 1.1 | 2026-04-20 | pre-build-sweep | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
