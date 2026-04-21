---
document_type: behavioral-contract
level: L3
version: "1.2"
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

# BC-2.14.008: TTD/TTI/TTR Per-Case and Aggregate MTTD/MTTI/MTTR Computation — From Event Timestamps to Case State Transitions

## Description

Per-case time metrics (TTD, TTI, TTR) are computed on-demand from case and alert
timestamps: TTD measures detection latency (event occurrence to rule fire), TTI measures
triage latency (first alert to case creation), and TTR measures investigation duration
(case creation to first resolution). Aggregate metrics (MTTD, MTTI, MTTR) are
cross-case averages computed by the `case_metrics` tool for reporting purposes.

All metrics are non-negative and deterministic for the same input state. Null is
returned when the required timestamps are absent, never zero — preserving the
distinction between "no data" and "instant detection."

## Preconditions
- A case exists with at least one linked alert (for TTD/TTI) or has been resolved/closed (for TTR)

## Postconditions
- **TTD (Time to Detect) — per-case metric:** computed per-alert as `alert.triggered_at - alert.event_time`. This measures the detection latency — the time between the source event occurring and the detection rule firing. In Prism's ephemeral model, each alert references a single source event (not a collection).
  - If `alert.event_time` is null: that alert's TTD is null
  - TTD is aggregated to per-case as `min(alert.ttd for alert in case.alert_ids)` (fastest detection among linked alerts)
- **TTI (Time to Investigate) — per-case metric:** computed as `case.created_at - min(alert.created_at for alert in case.alert_ids)`. This measures the triage latency — the time between the earliest triggering alert and the creation of the investigation case.
  - If no alerts are linked: TTI is null
  - If the earliest alert `created_at` is after `case.created_at` (alert linked retroactively): TTI is 0 (floor)
- **TTR (Time to Resolve) — per-case metric:** computed as `case.resolved_at - case.created_at`. This measures the investigation duration from case creation to first resolution.
  - If case is not yet resolved (`resolved_at` is null): TTR is null
  - If case was reopened and re-resolved: TTR uses the original `resolved_at` (first resolution time, not overwritten on subsequent transitions — preserves accurate first-resolution MTTR)
- **MTTD (Mean Time to Detect) — aggregate metric:** computed by the `case_metrics` tool as the average of per-case TTD values for all resolved cases within the specified time window. Only cases with non-null TTD are included in the average.
- **MTTI (Mean Time to Investigate) — aggregate metric:** computed by the `case_metrics` tool as the average of per-case TTI values for all resolved cases within the specified time window. Only cases with non-null TTI are included in the average.
- **MTTR (Mean Time to Resolve) — aggregate metric:** computed by the `case_metrics` tool as the average of per-case TTR values for all resolved cases within the specified time window. Only cases with non-null TTR are included in the average.
- **Time in current status:** computed as `now - timestamp_of_last_status_change`
- **Per-status duration breakdown:** computed from timeline entries; shows time spent in each status (New: 5m, Acknowledged: 2m, Investigating: 45m, etc.)
- Per-case metrics (TTD, TTI, TTR) are computed on-demand from case data, alert data, and timeline entries
- Per-case metrics are returned by `get_case` (BC-2.14.005); aggregate metrics (MTTD, MTTI, MTTR) are returned by `case_metrics` for cross-case reporting

## Invariants
- TTD, TTI, TTR, MTTD, MTTI, and MTTR are always non-negative (floored at 0)
- Per-case metrics (TTD, TTI, TTR) computation is deterministic: the same case state always produces the same metrics
- Aggregate metrics (MTTD, MTTI, MTTR) are deterministic for the same time window and case set
- Metrics do not modify case state (pure computation)

## Error Conditions
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
| EC-14-031 | Cross-case aggregate MTTD/MTTI/MTTR for a client | Computed by `case_metrics` tool as average of per-case TTD/TTI/TTR for resolved cases within the specified time window |
| EC-14-032 | No resolved cases in the specified time window | MTTD, MTTI, and MTTR are null (not zero) — no cases to average |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — TTR for resolved case | resolved_at=T+3600, created_at=T | TTR=3600s |
| Sub-second TTR | resolved_at=T+0.8s, created_at=T | TTR=0.8s (not rounded to 0) |
| Reopened case | first resolved_at=T+3600; reopened; resolved again at T+7200 | TTR=3600 (first resolution) |
| No resolved cases in window | since=2026-01-01, no resolved cases | MTTD=null, MTTI=null, MTTR=null |
| TTI with retroactive alert link | alert.created_at > case.created_at | TTI=0 (floored) |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify TTR uses first resolution timestamp on reopen cycles |
| (placeholder) | VP to be assigned — verify null propagation (no resolved cases → null aggregate) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
