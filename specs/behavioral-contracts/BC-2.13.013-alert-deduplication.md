---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-13"
capability: "CAP-021"
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
input-hash: "8bd996e"
traces_to: ["CAP-021"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.13.013: Alert Deduplication — Per-Match-Mode Dedup Keys Prevent Duplicate Alerts

## Description

Alert deduplication is a secondary guard (complementing reset-after-fire in BC-2.13.003) that prevents duplicate alerts when the same triggering condition is re-evaluated. The dedup key is computed per match mode: (rule_id, event_uid) for single-event; (rule_id, group_by_value_hash, window_bucket) for correlation; (rule_id, sequence_completion_hash) for sequence. Dedup is evaluated before alert persistence — suppressed alerts are never written. Dedup keys are persisted to RocksDB and expire after a configurable TTL (default 24h). On dedup index read failure, the system fails open (alert persisted) to prioritize detection over dedup strictness.

## Preconditions
- A detection rule has fired and an alert is about to be generated (BC-2.13.005)
- The alert deduplication state is accessible (in-memory index backed by RocksDB under the Alerts domain)

## Postconditions
- Before persisting a new alert, the deduplication key is computed based on the rule's match mode:
  - **Single-event mode:** dedup key = `(rule_id, event_uid)`. The same event (identified by its unique ID) cannot trigger the same rule more than once. This prevents duplicate alerts when the same record appears in consecutive differential results due to timing overlaps.
  - **Correlation mode:** dedup key = `(rule_id, group_by_value_hash, window_bucket)`. The same group key within the same time window bucket cannot fire the same rule more than once. The window bucket is derived from the correlation rule's `within` duration (e.g., a 5-minute window produces 5-minute-aligned buckets). This works in conjunction with reset-after-fire (BC-2.13.003) — dedup is a secondary guard.
  - **Sequence mode:** dedup key = `(rule_id, sequence_completion_hash)`. The completion hash is computed from the sequence's key field value and the event UIDs of all steps. The same completed sequence (same key, same events) cannot fire more than once. This prevents duplicate alerts if sequence state persistence replays a completed sequence.
- If the dedup key already exists in the dedup index: the alert is suppressed (not persisted, not notified), and a debug-level log entry is emitted
- If the dedup key does not exist: the alert is persisted (BC-2.13.005), the dedup key is added to the index, and notification proceeds
- Dedup keys are persisted to RocksDB for durability across restarts
- Dedup keys expire after a configurable TTL (default: 24 hours) to prevent unbounded growth

## Invariants
- Dedup is evaluated before alert persistence — a suppressed alert is never written to the Alerts storage domain
- Dedup key computation is deterministic: the same inputs always produce the same key
- Dedup does not cross rule boundaries: two different rules firing on the same event produce two alerts

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-DETECT-010` | Dedup index read failure from RocksDB | Alert is persisted (fail-open for dedup — better to have a duplicate than to miss an alert); warning logged |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-017 | Same event matches two different single-event rules | Two alerts generated (different rule_ids, different dedup keys) |
| EC-13-018 | Correlation rule fires, window resets, accumulates again within same window_bucket | Second fire is suppressed by dedup (same window_bucket); fires on next bucket boundary |
| EC-13-019 | Sequence completes with same key but different events than previous completion | Different completion_hash; new alert generated |
| EC-13-020 | Server restart mid-dedup-check | Dedup keys restored from RocksDB; no duplicate alerts on restart |
| EC-13-021 | Dedup TTL expires for a single-event key; same event reappears in differential | Alert fires again (dedup window has passed); this is expected behavior for stale differentials |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Same event_uid triggers same rule twice in consecutive diffs | Second alert suppressed; debug log emitted | happy-path |
| Same event matches two different rules | Two alerts generated | edge-case |
| Dedup index read failure | Alert persisted (fail-open); E-DETECT-010 warning | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-027 | Alert dedup key: correct per match mode | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-021 |
| L2 Invariants | DI-008 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
