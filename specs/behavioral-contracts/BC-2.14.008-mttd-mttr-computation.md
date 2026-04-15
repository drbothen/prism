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

# BC-2.14.008: MTTD/MTTR Auto-Computation — From Alert Timestamps to Case State Transitions

## Preconditions
- A case exists with at least one linked alert (for MTTD) or has been resolved/closed (for MTTR)

## Postconditions
- **MTTD (Mean Time to Detect):** computed as `case.created_at - min(alert.created_at for alert in case.source_alert_ids)`. This measures the time between the earliest triggering alert and the creation of the investigation case.
  - If no alerts are linked: MTTD is null
  - If the earliest alert `created_at` is after `case.created_at` (alert linked retroactively): MTTD is 0 (floor)
- **MTTR (Mean Time to Resolve):** computed as `case.closed_at - case.created_at`. This measures the total investigation duration from case creation to resolution/closure.
  - If case is not yet resolved/closed: MTTR is null
  - If case was reopened and re-resolved: MTTR uses the most recent `closed_at` (total elapsed time, not excluding reopen periods)
- **Time in current status:** computed as `now - timestamp_of_last_status_change`
- **Per-status duration breakdown:** computed from timeline entries; shows time spent in each status (New: 5m, Acknowledged: 2m, Investigating: 45m, etc.)
- Metrics are computed on-demand (not pre-computed) from case data and timeline entries
- Metrics are returned by `get_case` (BC-2.14.005) and available in aggregate form for cross-case reporting

## Invariants
- MTTD and MTTR are always non-negative (floored at 0)
- Metrics computation is deterministic: the same case state always produces the same metrics
- Metrics do not modify case state (pure computation)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| (none) | Metrics computation never fails | Missing data produces null metrics, not errors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-027 | Case resolved in under 1 second | MTTR is sub-second duration (e.g., "0.8s"); not rounded to 0 |
| EC-14-028 | Case reopened 3 times | MTTR uses final `closed_at - created_at`; per-status breakdown shows all reopen cycles |
| EC-14-029 | Linked alert created 1 hour before case | MTTD = 1 hour |
| EC-14-030 | Alert linked after case creation (retroactive linking) | MTTD recalculated; if newly linked alert is earliest, MTTD may increase |
| EC-14-031 | Cross-case aggregate MTTD/MTTR for a client | Computed as average of per-case MTTD/MTTR for resolved cases; available via future reporting tool |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004 |
| Priority | P1 |
