---
document_type: architecture-section
level: L3
section: "data-layer"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, domain-spec/entities.md]
traces_to: ARCH-INDEX.md
---

# Data Layer

## Storage Architecture

Prism has two distinct data paths: **ephemeral** (sensor query data, in-memory only) and **persistent** (operational state, RocksDB).

### Ephemeral Data Path (Query Execution)

Sensor data exists only during query execution. The materialization chain:

1. **Raw JSON** — HTTP responses from sensor APIs (reqwest response body)
2. **DynamicMessage** — OCSF-normalized protobuf messages (prost-reflect)
3. **Arrow RecordBatch** — Columnar in-memory format (apache-arrow)
4. **DataFusion MemTable** — Registered in ephemeral SessionContext
5. **Query results** — Returned to MCP client
6. **Teardown** — SessionContext dropped, all memory freed

No sensor data touches disk. The response cache (CAP-014) holds serialized adapter responses in memory with TTL-based expiration.

### Persistent Data Path (RocksDB)

RocksDB stores operational state organized by 12 column families. Each column family maps to a `StorageDomain` enum variant.

| Column Family | Domain | Key Pattern | Value Format | Access Pattern |
|--------------|--------|------------|-------------|---------------|
| `default` | Default | varies | bincode | General-purpose |
| `schedules` | Schedules | `{query_name}:{client_id}` | bincode | Read on tick, write on execution |
| `diff_results` | DiffResults | `{query_name}:{client_id}` | bincode (Arrow RecordBatch) | Read/write per schedule execution |
| `detection_rules` | DetectionRules | `{scope}:{rule_id}` | bincode | Read per detection evaluation |
| `detection_state` | DetectionState | `{rule_id}:{group_key}` | bincode | Read/write per detection evaluation |
| `alerts` | Alerts | `{alert_id}` (UUID v7, time-sortable) | bincode | Append-only, scan by prefix |
| `cases` | Cases | `{case_id}` (UUID v7) | bincode | CRUD on case lifecycle |
| `audit_buffer` | AuditBuffer | `{timestamp_nanos}:{trace_id}` | bincode | Append, sequential scan, delete on ack |
| `dirty_bits` | DirtyBits | `{query_hash}` | bincode | Set before query, clear after |
| `watchdog` | Watchdog | `{query_hash}` | bincode | Read on query start, write on denylist |
| `aliases` | Aliases | `{scope}:{alias_name}` | bincode | Read on query, write on create/delete |
| `decorators` | Decorators | `{decorator_name}` | bincode | Read per query, write on periodic refresh |

### Decision: Bincode for Value Serialization (AD-012)

**Status:** accepted
**Context:** RocksDB values need a serialization format. Options: JSON, bincode, MessagePack, postcard.
**Options considered:**
1. JSON — human-readable but 3-5x larger than binary; parsing overhead
2. bincode — compact binary, zero-copy deserialization for simple types, Rust-native
3. MessagePack — compact, cross-language, but no zero-copy in Rust
4. postcard — embedded-focused, smaller but less ecosystem adoption
**Decision:** bincode 2.x for all RocksDB values.
**Rationale:** Prism is a single-language system (Rust). Bincode provides the smallest encoding with the fastest deserialization. Schema evolution is handled by versioned keys — when a schema changes, a migration reads old-format values and writes new-format values.
**Consequences:** RocksDB data is not human-readable. Debugging requires a Prism CLI tool to decode values. Cross-language access (if ever needed) would require a format migration.

## Arrow Schema Design

The materialized table uses a two-tier columnar layout:

**Hot columns** (flat Arrow columns for common OCSF fields):
- `severity_id: Int32` — OCSF severity
- `device_hostname: Utf8` — normalized device name
- `device_ip: Utf8` — normalized IP address
- `time: TimestampMicrosecond` — event timestamp
- `message: Utf8` — event summary
- `sensor: Utf8` — virtual field (source sensor)
- `client_id: Utf8` — virtual field
- `source: Utf8` — virtual field (table name)

**Cold column** (full event data):
- `event_data: Utf8` — full OCSF event as JSON string, accessed via `json_extract_string()` UDF for ad-hoc field access

This two-tier design keeps hot-path queries (filtering by severity, hostname, IP) operating on native Arrow columns while preserving access to the full event for deep inspection.

## Cache Architecture

| Cache | Scope | Eviction | Invalidation |
|-------|-------|----------|-------------|
| Response cache | per (client_id, sensor_id, source_id, query_hash) | LRU, 50 entries per client per sensor | Synchronous on write operations; bypass with force_refresh |
| In-query cache | per query execution | Dropped with SessionContext | N/A (per-query lifetime) |
| Discovery cache | per (pack_id, client_id) | TTL 3600s | Config reload |
| Decorator cache | per decorator_name | Configurable refresh interval | Refresh failure uses stale value |

## RocksDB Configuration

```
state_dir: ./state (configurable via --state-dir)
WAL: enabled (crash safety)
Column families: 12 (created at first open)
LOCK file: prevents multi-process access (DI-017)
Sync writes: enabled for audit_buffer domain only (DI-026)
Compaction: level-based (default)
```

### Decision: StorageBackend Trait for Testability (AD-004)

**Status:** accepted
**Context:** Tests need to run without RocksDB on disk.
**Decision:** Define `StorageBackend` trait with `get/put/put_batch/remove/scan/prefix_scan` methods. Two implementations: `RocksDbBackend` (production) and `InMemoryBackend` (tests, uses `BTreeMap`).
**Rationale:** Reference: osquery uses a similar pattern with `DatabasePlugin` trait backed by RocksDB in production and in-memory stores for testing. This enables fast, isolated unit tests for all storage-dependent code.
**Consequences:** All storage access goes through the trait. No direct RocksDB calls outside `prism-storage`.
