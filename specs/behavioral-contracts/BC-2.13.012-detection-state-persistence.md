---
document_type: behavioral-contract
level: L3
version: "1.0"
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
---

# BC-2.13.012: Detection State Persistence — RocksDB Domain for Correlation Windows, Sequence State, Alert History

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
