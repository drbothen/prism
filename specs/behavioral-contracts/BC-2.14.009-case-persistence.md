---
document_type: behavioral-contract
level: L3
version: "1.0"
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
---

# BC-2.14.009: Case Persistence — RocksDB Domain for Case State, Timeline, Disposition, Metrics

## Preconditions
- The RocksDB database is initialized with the `cases` column family (BC-2.15.001)

## Postconditions
- The `cases` column family stores:
  - Case records: key = `case:{client_id}:{case_id}`, value = serialized Case struct (includes all fields: title, description, status, severity, assignee, disposition, source_alert_ids, annotations, timeline, timestamps)
  - Case index by status: key = `case_idx:status:{client_id}:{status}:{case_id}`, value = empty (existence index for efficient status filtering)
  - Case index by severity: key = `case_idx:severity:{client_id}:{severity}:{case_id}`, value = empty (existence index)
  - Case index by assignee: key = `case_idx:assignee:{assignee}:{client_id}:{case_id}`, value = empty (existence index)
  - Case index by time: key = `case_idx:time:{client_id}:{updated_at}:{case_id}`, value = empty (range scan index for time-based sorting)
- Index entries are updated atomically with the case record via WriteBatch:
  - On status change: old status index entry removed, new one added
  - On severity change: old severity index entry removed, new one added
  - On assignee change: old assignee index entry removed, new one added
- Case data survives server restarts
- Case retention is unlimited (no automatic purging); manual archival is a future capability

## Invariants
- Case data and all indexes are updated atomically via WriteBatch
- Index entries are always consistent with case records (no orphaned indexes)
- Client data separation: all keys are prefixed with client_id for efficient client-scoped scans

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-001` | RocksDB column family does not exist | Fatal startup error |
| `E-STORE-003` | Deserialization failure for case record | Log error; affected case excluded from listings; get_case returns structured error |
| `E-STORE-003` | RocksDB disk full | Write fails; case update rejected with structured error; in-memory state not updated |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-053 | 10,000 cases across 50 clients | RocksDB handles efficiently via LSM tree; index scans bound by client_id prefix |
| EC-14-033 | Case with 1000 timeline entries | Single large value in RocksDB; serialized size may reach 100KB+; acceptable |
| EC-14-034 | Server crashes during WriteBatch | RocksDB provides atomic WriteBatch; either all changes apply or none |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-008 |
| Priority | P0 |
