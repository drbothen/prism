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

# BC-2.15.001: RocksDB Initialization — Create/Open Database, Initialize Column Families for All Domains

## Preconditions
- The Prism server is starting up
- The configured state directory path exists and is writable (default: `./state`, set via `--state-dir` CLI flag)
- RocksDB native library is available (bundled via `rust-rocksdb` crate with static linking)

## Postconditions
- A RocksDB database is opened (or created if not existing) at `{state_dir}/prism.db` where `state_dir` defaults to `./state` (set via `--state-dir` CLI flag)
- The following column families are created if not already present (16 total):
  - `default` -- general-purpose key-value storage
  - `schedules` -- scheduled query definitions, splay offsets, epoch counters, timing state (BC-2.12.010)
  - `diff_results` -- differential result history (BC-2.12.010)
  - `detection_rules` -- detection rule storage by scope (BC-2.13.012)
  - `detection_state` -- correlation windows, sequence trackers (BC-2.13.012)
  - `alerts` -- alert records and indexes (BC-2.13.012)
  - `cases` -- case records, timeline, indexes (BC-2.14.009)
  - `audit_buffer` -- buffered audit log entries awaiting forwarding (BC-2.15.003)
  - `dirty_bits` -- crash recovery markers (BC-2.15.005)
  - `watchdog` -- query denylist entries and watchdog state (BC-2.15.008)
  - `aliases` -- alias definitions, splay offsets, and metadata (BC-2.11.008)
  - `decorators` -- periodic decorator cache values (BC-2.15.010)
  - `action_state` -- action delivery retry state, dead-letter records (BC-2.18.001)
  - `infusion_cache` -- per-query dedup cache for infusion UDF results (BC-2.19.002)
  - `plugin_state` -- WASM plugin registration and hot-reload metadata (BC-2.17.005)
  - `event_buffer` -- buffered sensor events for event-stream table abstraction (S-2.08; osquery event publisher pattern)
- RocksDB options are configured for Prism's workload:
  - Write buffer size: 64MB (default)
  - Max open files: 256
  - Compression: LZ4 for all levels
  - WAL (Write-Ahead Log) enabled for crash recovery
  - Bloom filters enabled for point lookups
- A startup health check verifies: database opens successfully, all column families are accessible, a test write/read/delete succeeds
- If the database is corrupted: attempt automatic repair via `DB::repair()`; if repair fails, log fatal error and exit with code 3

## Invariants
- Column family creation is idempotent: existing families are not modified
- The database is exclusively locked: only one Prism process can open it at a time
- WAL ensures atomicity of WriteBatch operations across crash boundaries

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-001` | Data directory does not exist and cannot be created | Fatal startup error with path and OS error |
| `E-STORE-005` | Database lock held by another process | Fatal startup error: "Another Prism instance is using {path}" |
| `E-STORE-006` | Database corruption detected | Attempt repair; if repair fails, fatal error with guidance to delete and re-initialize |
| `E-STORE-007` | Insufficient disk space | Fatal startup error with available/required space |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-001 | First startup (no existing database) | Database and all column families created fresh; no error |
| EC-15-002 | Upgrade adds new column family | New column family created; existing families and data preserved |
| EC-15-003 | Database from a newer Prism version (unknown column families) | Unknown families are left intact; log warning; Prism uses only its known families |
| EC-15-004 | Data directory on network filesystem (NFS) | Warning logged: "Network filesystem detected; RocksDB performance may be degraded" |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| L2 Invariants | -- |
| Priority | P0 |
