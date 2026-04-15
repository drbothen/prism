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

# BC-2.14.008: TTD/TTR Per-Case and Aggregate MTTD/MTTR Computation — From Alert Timestamps to Case State Transitions

## Preconditions
- A case exists with at least one linked alert (for TTD) or has been resolved/closed (for TTR)

## Postconditions
- **TTD (Time to Detect) — per-case metric:** computed as `case.created_at - min(alert.created_at for alert in case.source_alert_ids)`. This measures the time between the earliest triggering alert and the creation of the investigation case.
  - If no alerts are linked: TTD is null
  - If the earliest alert `created_at` is after `case.created_at` (alert linked retroactively): TTD is 0 (floor)
- **TTR (Time to Resolve) — per-case metric:** computed as `case.resolved_at - case.created_at`. This measures the investigation duration from case creation to first resolution.
  - If case is not yet resolved (`resolved_at` is null): TTR is null
  - If case was reopened and re-resolved: TTR uses the original `resolved_at` (first resolution time, not overwritten on subsequent transitions — preserves accurate first-resolution MTTR)
- **MTTD (Mean Time to Detect) — aggregate metric:** computed by the `case_metrics` tool as the average of per-case TTD values for all resolved cases within the specified time window. Only cases with non-null TTD are included in the average.
- **MTTR (Mean Time to Resolve) — aggregate metric:** computed by the `case_metrics` tool as the average of per-case TTR values for all resolved cases within the specified time window. Only cases with non-null TTR are included in the average.
- **Time in current status:** computed as `now - timestamp_of_last_status_change`
- **Per-status duration breakdown:** computed from timeline entries; shows time spent in each status (New: 5m, Acknowledged: 2m, Investigating: 45m, etc.)
- Per-case metrics (TTD, TTR) are computed on-demand from case data and timeline entries
- Per-case metrics are returned by `get_case` (BC-2.14.005); aggregate metrics (MTTD, MTTR) are returned by `case_metrics` for cross-case reporting

## Invariants
- TTD, TTR, MTTD, and MTTR are always non-negative (floored at 0)
- Per-case metrics (TTD, TTR) computation is deterministic: the same case state always produces the same metrics
- Aggregate metrics (MTTD, MTTR) are deterministic for the same time window and case set
- Metrics do not modify case state (pure computation)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| (none) | Metrics computation never fails | Missing data produces null metrics, not errors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-027 | Case resolved in under 1 second | TTR is sub-second duration (e.g., "0.8s"); not rounded to 0 |
| EC-14-028 | Case reopened 3 times | TTR uses original `resolved_at - created_at` (first resolution time); per-status breakdown shows all reopen cycles |
| EC-14-029 | Linked alert created 1 hour before case | TTD = 1 hour |
| EC-14-030 | Alert linked after case creation (retroactive linking) | TTD recalculated; if newly linked alert is earliest, TTD may increase |
| EC-14-031 | Cross-case aggregate MTTD/MTTR for a client | Computed by `case_metrics` tool as average of per-case TTD/TTR for resolved cases within the specified time window |
| EC-14-032 | No resolved cases in the specified time window | MTTD and MTTR are null (not zero) — no cases to average |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004 |
| Priority | P0 |
