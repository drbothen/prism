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
input-hash: "b1e4604"
traces_to: ["CAP-020"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.13.012: Detection State Persistence — RocksDB Domain for Correlation Windows, Sequence State, Alert History

## Description

Detection state is persisted across restarts using three RocksDB column families: `detection_rules` (global and client-scoped rule sources with metadata index), `detection_state` (correlation sliding windows and sequence trackers, flushed via WriteBatch after each execution cycle), and `alerts` (alert records with two index keys: by rule and by time range). A 5-minute background cleanup task purges expired windows, trackers, and alerts beyond retention (default 90 days). Deserialization failures reset affected state entries with a warning rather than crashing.

## Preconditions
- The RocksDB database is initialized with the `detection_rules`, `detection_state`, and `alerts` column families (BC-2.15.001)

## Postconditions
- The `detection_rules` column family stores:
  - Global rules: key = `rules:global:{rule_id}`, value = serialized rule source + parsed metadata
  - Client rules: key = `rules:client:{client_id}:{rule_id}`, value = serialized rule source + parsed metadata
  - Rule metadata index: key = `rules:index:{rule_type}`, value = list of rule_ids by type for fast lookup
- The `detection_state` column family stores:
  - Correlation windows: key = `corr:{rule_id}:{group_key}`, value = serialized SlidingWindow (VecDeque of (timestamp, event_uid) pairs)
  - Sequence trackers: key = `seq:{rule_id}:{key_value}`, value = serialized SequenceTracker (current_step, per-step counts, per-step events, start_time)
  - Correlation/sequence state is flushed to RocksDB after each scheduled execution cycle via WriteBatch
- The `alerts` column family stores:
  - Alerts: key = `alert:{client_id}:{alert_id}`, value = serialized Alert struct
  - Alert index by rule: key = `alert_idx:rule:{rule_id}:{alert_id}`, value = empty (existence index)
  - Alert index by time: key = `alert_idx:time:{client_id}:{timestamp}:{alert_id}`, value = empty (range scan index)
- **Periodic cleanup:** expired correlation windows (beyond `within` duration + 1 hour buffer) and expired sequence trackers are purged during a background cleanup task running every 5 minutes
- Alert retention is configurable (default 90 days); expired alerts and their index entries are purged during cleanup

## Invariants
- Detection state survives server restarts
- WriteBatch ensures atomic updates within a single execution cycle
- Alert index supports efficient queries by client_id + time range and by rule_id
- Cleanup never removes active (non-expired) windows or trackers

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-001` | RocksDB column family does not exist | Fatal startup error |
| `E-STORE-003` | Deserialization failure for detection state | Affected state entries are reset; warning logged |
| `E-STORE-003` | RocksDB disk full during write | Write fails; dirty bit set (BC-2.15.005); alert logged to stderr as fallback |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-042 | 10,000 active correlation windows | All fit within RocksDB; memory-mapped access keeps working set in memory |
| EC-13-043 | Server crashes during WriteBatch for detection state | Dirty bit detects incomplete write on restart; affected state entries are reset |
| EC-13-044 | Alert retention set to 0 days | Alerts are purged immediately after creation during the next cleanup cycle; not recommended |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Server restart after 100 correlation window updates | Windows loaded from RocksDB; detection continues | happy-path |
| Deserialization failure for one window | That window reset; others unaffected | error |
| Crash during WriteBatch | Dirty bit set; affected state entries reset on next startup | error |
| Alert retention = 0 days | Alerts purged during next cleanup cycle | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-027 | Alert dedup key: correct per match mode | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
