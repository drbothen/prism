---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Platform Infrastructure"
capability: "CAP-024"
---

# BC-2.15.002: Domain-Based Key-Value Operations — get/put/putBatch/remove/removeRange/scan per Domain

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

## Error Cases
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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| L2 Invariants | -- |
| Priority | P0 |
