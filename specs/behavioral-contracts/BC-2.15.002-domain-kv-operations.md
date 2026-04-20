---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-15"
capability: "CAP-019"
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
input-hash: "[pending-recompute]"
traces_to:
  - "CAP-019"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.002: Domain-Based Key-Value Operations — get/put/putBatch/remove/removeRange/scan per Domain

## Description

The `StorageEngine` trait exposes a typed, domain-scoped KV API over RocksDB column
families. Each operation is scoped to a named domain (column family), preventing
cross-domain data contamination. Batch writes via `put_batch` use RocksDB WriteBatch
for atomicity; serialization is the caller's responsibility using `bincode` or
`serde_json`. All operations are synchronous.

The key-ordering model is lexicographic (byte-wise), which callers exploit via
prefixed key schemas (e.g., `case:{client_id}:`) to enable efficient client-scoped
prefix scans without full-table reads.

## Preconditions
- The RocksDB database is initialized and all column families are accessible (BC-2.15.001)
- The caller specifies a domain (column family name) for each operation

## Postconditions
- The `StorageEngine` trait provides the following operations, each scoped to a specific column family (domain):
  - **`get(domain, key) -> Option<Vec<u8>>`**: retrieve a single value by key; returns None if key not found
  - **`put(domain, key, value)`**: write a single key-value pair; overwrites if key exists
  - **`put_batch(domain, entries: Vec<(key, value)>)`**: atomically write multiple key-value pairs via RocksDB WriteBatch; all entries succeed or all fail
  - **`remove(domain, key)`**: delete a single key; no-op if key does not exist
  - **`remove_range(domain, start_key, end_key)`**: delete all keys in the range [start_key, end_key); uses RocksDB `DeleteRange` for efficiency
  - **`scan(domain, prefix, limit) -> Vec<(key, value)>`**: scan keys matching the given prefix, returning up to `limit` entries; results are in lexicographic key order
  - **`scan_range(domain, start_key, end_key, limit) -> Vec<(key, value)>`**: scan keys in the range [start_key, end_key) up to `limit`
- All operations are synchronous and block until the underlying RocksDB operation completes
- Values are opaque byte arrays; serialization/deserialization is the caller's responsibility (using `bincode` or `serde_json`)
- WriteBatch operations are atomic: either all entries in the batch are applied or none are

## Invariants
- Domain isolation: operations on one column family never affect another
- Key ordering is lexicographic (byte-wise); callers design keys with this in mind (e.g., `case:acme:` prefix for client-scoped scans)
- WriteBatch atomicity is guaranteed by RocksDB WAL

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-001` | Unknown domain (column family) | Panic at debug; structured error at release -- indicates a programming error |
| `E-STORE-003` | RocksDB disk full on write | Error returned to caller; no partial write |
| `E-STORE-008` | I/O error during read | Error returned to caller with OS error details |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-005 | `get` for non-existent key | Returns None, not error |
| EC-15-006 | `remove` for non-existent key | No-op, not error |
| EC-15-007 | `scan` with prefix that matches 10,000 keys but limit is 100 | Returns first 100 keys in lexicographic order |
| EC-15-008 | `put_batch` with 0 entries | No-op; returns success |
| EC-15-009 | `remove_range` where start_key == end_key | No-op; empty range |
| EC-15-010 | Value exceeding 1MB | Warning logged; write proceeds (RocksDB supports large values but performance degrades) |
| EC-15-011 | Deserialization fails for a stored value (format change between Prism versions) | The entry is logged as corrupted (key, domain, error details) and skipped. A `prism --migrate-storage` CLI command is the documented recovery path for upgrading stored data to the current format. The corrupted entry is not deleted automatically — it remains in RocksDB for manual inspection or migration. See ASM-012. |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — put/get round-trip | `put(domain, "key1", b"value1")`, then `get(domain, "key1")` | `Some(b"value1")` |
| Get missing key | `get(domain, "nonexistent")` | `None` |
| put_batch atomicity | batch of 5 writes; disk full on write 3 | All 5 fail; none applied |
| scan with prefix | 1000 keys, prefix matches 50, limit=25 | 25 keys in lex order |
| remove non-existent | `remove(domain, "missing")` | No-op; no error |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify put_batch atomicity under simulated write failure |
| (placeholder) | VP to be assigned — verify domain isolation (write to A does not appear in B) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-019 |
| L2 Invariants | -- |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
