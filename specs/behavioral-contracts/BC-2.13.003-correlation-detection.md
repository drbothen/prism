---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-13"
capability: "CAP-020"
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
input-hash: "365fb25"
traces_to: ["CAP-020"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.13.003: Correlation Detection — Threshold Over Sliding Window with Group-By, Reset-After-Fire

## Description

Correlation detection fires when a threshold count of matching events accumulates for the same group key within a time window. Each new/added record from differential results is evaluated against the rule's condition; matching records are added to the per-group-key sliding window in RocksDB. Expired entries (older than `within` duration) are evicted on each evaluation. When the threshold is met, an alert fires and the window resets (reset-after-fire). Window state is persisted to RocksDB after each scheduled execution cycle. Record timestamps (event_time), not wall-clock time, drive window expiry for consistent behavior with historical data.

## Preconditions
- One or more enabled correlation rules exist (rules with `match count(event where <condition>) <op> <threshold> group_by <fields> within <duration>`)
- Materialized OCSF records are available from a scheduled query execution
- The correlation state (sliding windows per group key) is loaded from RocksDB (BC-2.13.012)

## Postconditions
- For each correlation rule, each record in the differential results (new/added records from CAP-018) is evaluated against the rule's condition
- If the condition matches, the group key is constructed by concatenating `group_by` field values with `|` separator (e.g., for `group_by src_endpoint.ip, user.name`: `"10.0.0.1|root"`)
- The new record is added to the persisted sliding window for (rule_id, group_key) with its timestamp and event UID
- Expired entries (older than the rule's `within` duration) are evicted from the window
- The threshold comparison is evaluated over the full sliding window (persisted historical entries plus newly added records from this differential) using the supported operators (`>=`, `>`, `==`, `<`, `<=`)
- If the threshold is met:
  - An alert is generated (BC-2.13.005) with all event UIDs in the window as trigger events
  - The window for that (rule_id, group_key) is **cleared** (reset-after-fire) to prevent duplicate alerts from the same accumulation
  - A new accumulation must start from zero for the same group key to fire again
- Updated sliding window state is persisted to RocksDB (BC-2.13.012) after each scheduled execution cycle

## Invariants
- Reset-after-fire: after firing, N-1 additional events do NOT fire the rule; only a fresh accumulation of N events fires again
- Group keys are deterministic: the same field values always produce the same group key
- Window expiry uses record timestamps (event_time), not wall-clock time, for consistent behavior with historical data

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-DETECT-003` | Group-by field is null for a record | Record is excluded from correlation for this rule (no alert); warning logged |
| `E-DETECT-004` | Window state deserialization failure on startup | Affected windows are reset to empty; warning logged; rule continues from clean state |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-008 | Threshold `>= 5`: exactly 5 failures arrive in one scheduled execution | Alert fires; window cleared |
| EC-13-009 | Threshold `>= 5`: 4 failures in epoch N, 3 failures in epoch N+1 (same group key, within window) | If all 7 are within the window duration, alert fires at event 5; remaining 2 start a new accumulation |
| EC-13-010 | 1000 unique group keys across 50 clients | 1000 independent sliding windows maintained; memory bounded by window duration and event rate |
| EC-13-011 | Correlation rule with `within 5m` but scheduled query runs every 10m | Window may contain events from multiple scheduled executions; events older than 5m are evicted regardless of execution boundary |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| 5 events (threshold=5) from same group key within window | Alert fires; window cleared | happy-path |
| 4 events from group key A, 5 from group key B | Alert fires for B only; A window at 4 | happy-path |
| Null group-by field in record | Record excluded from this rule; warning logged | edge-case |
| Window state deserialization failure on startup | Window reset; rule continues from clean state | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-027 | Alert dedup key: correct per match mode | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-008 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col Version | Burst | Date | Author | Change form. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
