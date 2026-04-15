---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Detection Engine"
capability: "CAP-021"
---

# BC-2.13.004: Sequence Detection — Ordered Multi-Event Pattern Matching Within Time Window

## Preconditions
- One or more enabled sequence rules exist (rules with `match sequence by <key_field> within <duration> { step <name>: ... }`)
- Materialized OCSF records are available from a scheduled query execution, ordered by `event_time`
- The sequence state (trackers per key value) is loaded from RocksDB (BC-2.13.012)

## Postconditions
- For each sequence rule, records are processed in `event_time` order
- For each record, the key field value is extracted (e.g., `src_endpoint.ip` = "10.0.0.1")
- A `SequenceTracker` is retrieved or created for (rule_id, key_value), tracking: current step index, per-step matched events, per-step counts, start time, window duration
- If the tracker has expired (first matched event is older than `within` duration): tracker is reset to step 0
- The record is evaluated against the current step's condition:
  - `StepType::Event(condition)`: if condition matches, record event UID and advance to next step
  - `StepType::Count { condition, op, threshold }`: if condition matches, increment step count; if threshold met, advance to next step
- Steps are strictly ordered: step N+1 cannot advance before step N completes
- If all steps complete: an alert is generated (BC-2.13.005) with per-step event UIDs and counts; tracker is **reset**
- Updated sequence state is persisted to RocksDB (BC-2.13.012) after each scheduled execution cycle

## Invariants
- Strict step ordering: events matching step 2 are ignored until step 1 has completed
- Reset-after-fire: a completed sequence requires an entirely new sequence to fire again
- Events beyond a step's threshold are ignored after the step advances (no double-counting)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-DETECT-005` | Key field is null for a record | Record excluded from sequence tracking for this rule; warning logged |
| `E-DETECT-006` | Sequence state deserialization failure on startup | Affected trackers reset to step 0; warning logged |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-012 | Brute-force-then-success: 3 failures then 1 success from same IP within 10m | Alert fires with step "failures" count=3 and step "success" event UID |
| EC-13-013 | 5 failures (threshold 3) then success | Step advances after 3rd failure; failures 4-5 ignored; success completes sequence |
| EC-13-014 | Sequence window expires mid-accumulation | Tracker reset on next event for that key; partial accumulation is lost |
| EC-13-015 | Cross-sensor sequence: CrowdStrike alert then Claroty event from same IP | Works if both records share the key field value (`src_endpoint.ip`); sequence engine is sensor-agnostic |
| EC-13-016 | Records from scheduled execution are not perfectly ordered by event_time | Records are sorted by event_time before sequence evaluation; ties broken by record insertion order |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-021 |
| L2 Invariants | DI-008 |
| Priority | P1 |
