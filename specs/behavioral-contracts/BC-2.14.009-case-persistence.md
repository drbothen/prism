---
document_type: behavioral-contract
level: L3
version: "1.3"
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
input-hash: "85d7741"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.009: Case Persistence — RocksDB Domain for Case State, Timeline, Disposition, Metrics

## Description

Case records and their associated indexes are stored in the `cases` RocksDB column
family using a key schema designed for efficient client-scoped lookups and multi-field
filtering. All writes use RocksDB WriteBatch to atomically update both the case record
and all secondary indexes, preventing inconsistent index state under concurrent writes
or crash scenarios.

Secondary indexes exist for status, severity, assignee, and time-based sorting,
enabling the `list_cases` tool (BC-2.14.004) to perform efficient scans without full
table reads. Case data is retained indefinitely with no automatic purging.

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

## Error Conditions
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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — persist and retrieve | create case, then get by key | Round-trip preserves all fields |
| Status index consistency | update case status; list by old status | Case no longer appears under old status index |
| Crash during write | simulate WriteBatch abort | Case state unchanged; no orphaned index entries |
| Deserialization failure | corrupt value at known key | Case excluded from listing; get_case returns structured error |
| 10K cases | bulk create | RocksDB prefix scans by client_id remain efficient |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | WriteBatch atomicity is a RocksDB WAL guarantee, not a Prism code property; index consistency after transitions is an integration test on the storage layer; covered by integration tests in S-1.02 / S-4.06 test suites. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-008 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
