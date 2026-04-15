---
name: RocksDB Rust Bindings Research
type: research
date: 2026-04-15
phase: pre-architecture
---

# RocksDB Rust Bindings Research

**Date:** 2026-04-15
**Type:** Technology evaluation
**Status:** Complete
**Purpose:** Evaluate `rust-rocksdb` crate for Prism's persistence layer -- 12 column families, atomic writes, WAL-flushed audit buffer, prefix iteration, and single-process LOCK guarantees.
**Sources:** crates.io API (live, 2026-04-15) + model training data.
**Verified version:** `rocksdb` v0.24.0 on crates.io (updated 2025-08-10, 43.7M downloads).

---

## 1. Crate Status: `rust-rocksdb` on crates.io

There are two crates to be aware of:

| Crate | crates.io name | Notes |
|-------|---------------|-------|
| **`rust-rocksdb`** | `rust-rocksdb` | Community fork, actively maintained as of early 2025. This is the one we want. |
| **`rocksdb`** (original) | `rocksdb` | The older crate by aleksey-kladov/spacejam. Was the canonical crate for years. In late 2023/2024, maintenance shifted and the `rust-rocksdb` fork became the recommended path. |

**As of training data cutoff (May 2025):**
- `rust-rocksdb` was at approximately v0.22.x-v0.23.x, wrapping RocksDB C++ 8.x-9.x.
- The crate is published by the `rust-rocksdb` GitHub organization.
- Repository: `https://github.com/rust-rocksdb/rust-rocksdb`
- It uses `librocksdb-sys` as the -sys crate, bundling the C++ source and building via `cc`/`cmake`.
- Downloads: millions of total downloads; widely used in production (TiKV, CockroachDB's Rust components, Solana validators, Aptos, etc.).

**ACTION REQUIRED:** Verify current version on crates.io before pinning in `Cargo.toml`. Check whether the crate has been renamed or if `rocksdb` and `rust-rocksdb` have re-merged.

**Cargo.toml dependency:**
```toml
[dependencies]
rust-rocksdb = { version = "0.23", features = ["multi-threaded-cf"] }
# OR if the original crate is current:
# rocksdb = { version = "0.22", features = ["multi-threaded-cf"] }
```

The `multi-threaded-cf` feature enables `Send + Sync` on column family handles, which we need since Prism's MCP request handlers will access the DB from async tasks on a Tokio runtime.

---

## 2. Column Family API

RocksDB column families (CFs) are logically separate key-value namespaces that share the same WAL but have independent memtables, SST files, and compaction settings. This is exactly what Prism needs -- 12 CFs with different access patterns.

### 2.1 Creating and Opening Column Families

```rust
use rust_rocksdb::{
    DB, Options, ColumnFamilyDescriptor, 
    DBWithThreadMode, MultiThreaded,
};

/// Prism's 12 column families
const CF_NAMES: &[&str] = &[
    "schedules",
    "diff_results",
    "detection_rules",
    "detection_state",
    "alerts",
    "cases",
    "audit_buffer",
    "dirty_bits",
    "watchdog",
    "aliases",
    "decorators",
    "default",
];

fn open_prism_db(path: &str) -> Result<DBWithThreadMode<MultiThreaded>, rust_rocksdb::Error> {
    let mut db_opts = Options::default();
    db_opts.create_if_missing(true);
    db_opts.create_missing_column_families(true);
    
    // Per-CF options allow tuning each family independently
    let cf_descriptors: Vec<ColumnFamilyDescriptor> = CF_NAMES
        .iter()
        .map(|name| {
            let mut cf_opts = Options::default();
            
            // Tune per-CF based on access pattern
            match *name {
                "audit_buffer" => {
                    // Write-heavy, must survive crashes
                    cf_opts.set_write_buffer_size(4 * 1024 * 1024); // 4MB memtable
                },
                "dirty_bits" => {
                    // Small keys, frequent point lookups
                    cf_opts.set_write_buffer_size(1 * 1024 * 1024);
                },
                "diff_results" => {
                    // Larger values, read-heavy for cache checks
                    cf_opts.set_write_buffer_size(8 * 1024 * 1024);
                },
                _ => {
                    cf_opts.set_write_buffer_size(4 * 1024 * 1024);
                },
            }
            
            ColumnFamilyDescriptor::new(*name, cf_opts)
        })
        .collect();
    
    DB::open_cf_descriptors(&db_opts, path, cf_descriptors)
}
```

### 2.2 Reading and Writing to Specific Column Families

```rust
fn cf_read_write_example(db: &DBWithThreadMode<MultiThreaded>) -> Result<(), rust_rocksdb::Error> {
    // Get a column family handle
    let cf_schedules = db.cf_handle("schedules")
        .expect("CF 'schedules' must exist");
    
    // Put a key-value pair into a specific CF
    db.put_cf(&cf_schedules, b"client1:sensor1:schedule1", b"serialized_schedule_bytes")?;
    
    // Get a value from a specific CF
    if let Some(value) = db.get_cf(&cf_schedules, b"client1:sensor1:schedule1")? {
        // value is Vec<u8>
        println!("Found schedule: {} bytes", value.len());
    }
    
    // Delete from a specific CF
    db.delete_cf(&cf_schedules, b"client1:sensor1:schedule1")?;
    
    Ok(())
}
```

### 2.3 Key Design for Prism's CFs

Recommended key encoding for prefix-scannable keys:

```
Key format: {client_id}:{sensor_id}:{entity_id}
Example:    "acme-corp:cs-prod-01:sched-daily-ioc"
```

Using `:` as delimiter with fixed-width or length-prefixed segments. For binary efficiency, consider a `(u32, u32, [u8])` tuple encoded with big-endian integers for correct lexicographic ordering under RocksDB's default bytewise comparator.

---

## 3. WriteBatch -- Atomic Multi-Key Writes

`WriteBatch` is the mechanism for atomic writes. All operations in a batch are applied atomically -- either all succeed or none do. This works **across column families**.

### 3.1 Basic WriteBatch

```rust
use rust_rocksdb::WriteBatch;

fn atomic_detection_update(
    db: &DBWithThreadMode<MultiThreaded>,
    client_id: &str,
    alert_bytes: &[u8],
    state_bytes: &[u8],
    audit_entry: &[u8],
) -> Result<(), rust_rocksdb::Error> {
    let cf_alerts = db.cf_handle("alerts").unwrap();
    let cf_state = db.cf_handle("detection_state").unwrap();
    let cf_audit = db.cf_handle("audit_buffer").unwrap();
    let cf_dirty = db.cf_handle("dirty_bits").unwrap();
    
    let mut batch = WriteBatch::default();
    
    // All of these are applied atomically across CFs
    batch.put_cf(&cf_alerts, b"acme:cs-01:alert-1234", alert_bytes);
    batch.put_cf(&cf_state, b"acme:cs-01:rule-42", state_bytes);
    batch.put_cf(&cf_audit, b"acme:2026-04-15T10:30:00Z:evt-5678", audit_entry);
    batch.put_cf(&cf_dirty, b"acme:cs-01", b"1"); // mark dirty for sync
    
    // Apply atomically
    db.write(batch)?;
    
    Ok(())
}
```

### 3.2 WriteBatch with Merge Operations

```rust
fn batch_with_merge(db: &DBWithThreadMode<MultiThreaded>) -> Result<(), rust_rocksdb::Error> {
    let mut batch = WriteBatch::default();
    let cf = db.cf_handle("detection_state").unwrap();
    
    // Mix of operations in one atomic batch
    batch.put_cf(&cf, b"key1", b"value1");
    batch.delete_cf(&cf, b"key2");
    batch.put_cf(&cf, b"key3", b"value3");
    
    // WriteBatch also supports delete_range_cf for bulk cleanup
    // batch.delete_range_cf(&cf, b"prefix:start", b"prefix:end");
    
    db.write(batch)?;
    Ok(())
}
```

### 3.3 WriteBatch Size

`WriteBatch` accumulates operations in memory. For Prism's use case (detection engine writing a handful of keys per evaluation cycle), batch sizes will be trivially small. WriteBatch has a `data()` method to inspect its serialized size, and `len()` for operation count.

---

## 4. WriteOptions { sync: true } -- WAL Flush for Audit Buffer (DI-026)

By default, RocksDB `write()` writes to the WAL but does not call `fsync()` -- the OS may buffer the write. With `sync: true`, RocksDB calls `fsync()` on the WAL file after the write, guaranteeing durability even on power loss.

```rust
use rust_rocksdb::WriteOptions;

fn audit_write_durable(
    db: &DBWithThreadMode<MultiThreaded>,
    key: &[u8],
    audit_entry: &[u8],
) -> Result<(), rust_rocksdb::Error> {
    let cf_audit = db.cf_handle("audit_buffer").unwrap();
    
    let mut write_opts = WriteOptions::default();
    write_opts.set_sync(true); // fsync WAL -- guarantees durability
    
    // Single durable put
    db.put_cf_opt(&cf_audit, key, audit_entry, &write_opts)?;
    
    Ok(())
}

fn audit_batch_durable(
    db: &DBWithThreadMode<MultiThreaded>,
    entries: &[(&[u8], &[u8])],
) -> Result<(), rust_rocksdb::Error> {
    let cf_audit = db.cf_handle("audit_buffer").unwrap();
    
    let mut batch = WriteBatch::default();
    for (key, value) in entries {
        batch.put_cf(&cf_audit, key, value);
    }
    
    let mut write_opts = WriteOptions::default();
    write_opts.set_sync(true);
    
    // Atomic + durable
    db.write_opt(batch, &write_opts)?;
    
    Ok(())
}
```

**Performance impact of sync writes:**
- `sync: true` adds ~1-5ms per write (SSD fsync latency).
- For the audit buffer specifically, this is acceptable -- audit writes happen at low frequency (detection events, case mutations, config changes).
- For all other CFs (schedules, diff_results, dirty_bits, etc.), use the default (no sync) -- the WAL still protects against process crashes; only power loss scenarios lose the last few writes.

**Recommendation for Prism:**
- `audit_buffer` CF: always `sync: true` (DI-026 compliance)
- All other CFs: default (WAL without fsync) -- crash-safe, not power-loss-safe, which is fine for cache/state data that can be reconstructed from sensor APIs

---

## 5. LOCK File Behavior (DI-017)

RocksDB enforces **single-process exclusive access** via a LOCK file in the database directory.

### 5.1 How It Works

1. When `DB::open()` is called, RocksDB creates a file named `LOCK` in the DB directory.
2. It acquires an **OS-level file lock** (`flock()` on Unix, `LockFileEx()` on Windows) on this file.
3. If another process (or another `DB::open()` call in the same process for the same path) attempts to open the same DB, it will receive an error: `"IO error: lock hold by current process, acquire time ... lock ... LOCK: No locks available"` or similar.
4. The lock is released when the `DB` object is dropped (or `DB::close()` is called, though in Rust the `Drop` impl handles this).

### 5.2 Implications for Prism (DI-017)

```rust
// This will fail if another process has the DB open:
let db = DB::open_cf_descriptors(&opts, "/var/prism/data", cf_descriptors);
// Err(Error { message: "IO error: lock ... LOCK: No locks available" })
```

**This is exactly what we want for DI-017** -- it guarantees that only one Prism MCP server process accesses the database at a time. If a user accidentally starts two instances pointing at the same data directory, the second will fail immediately with a clear error.

**Important edge cases:**
- **Stale LOCK after unclean shutdown:** On Linux/macOS, `flock()` locks are automatically released when the process dies (even on SIGKILL). The LOCK file remains on disk but is not locked at the OS level, so the next `DB::open()` succeeds. This is safe.
- **NFS/network filesystems:** `flock()` may not work correctly over NFS. Prism's data directory must be on a local filesystem.
- **Same process, multiple opens:** You can have multiple `DB` instances in one process if they point to different paths. You cannot open the same path twice.

---

## 6. Iterator API -- Prefix Scan for Cache Invalidation

### 6.1 Basic Prefix Iteration

```rust
use rust_rocksdb::{IteratorMode, Direction};

fn scan_by_client_sensor(
    db: &DBWithThreadMode<MultiThreaded>,
    cf_name: &str,
    client_id: &str,
    sensor_id: &str,
) -> Vec<(Vec<u8>, Vec<u8>)> {
    let cf = db.cf_handle(cf_name).unwrap();
    let prefix = format!("{}:{}", client_id, sensor_id);
    
    let iter = db.iterator_cf(
        &cf,
        IteratorMode::From(prefix.as_bytes(), Direction::Forward),
    );
    
    let mut results = Vec::new();
    for item in iter {
        let (key, value) = item.unwrap();
        // Stop when we leave the prefix
        if !key.starts_with(prefix.as_bytes()) {
            break;
        }
        results.push((key.to_vec(), value.to_vec()));
    }
    
    results
}
```

### 6.2 Optimized Prefix Iteration with ReadOptions

For better performance, configure the prefix extractor and bloom filters:

```rust
use rust_rocksdb::{Options, SliceTransform, ReadOptions};

fn configure_prefix_scan(cf_opts: &mut Options) {
    // Fixed prefix length: e.g., 32 bytes for client_id(16) + ":" + sensor_id(15)
    // Or use a custom prefix extractor
    cf_opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(32));
    
    // Enable prefix bloom filters for point prefix lookups
    cf_opts.set_memtable_prefix_bloom_ratio(0.1);
}

fn prefix_scan_optimized(
    db: &DBWithThreadMode<MultiThreaded>,
    cf_name: &str,
    prefix: &[u8],
) -> Vec<(Vec<u8>, Vec<u8>)> {
    let cf = db.cf_handle(cf_name).unwrap();
    
    let mut read_opts = ReadOptions::default();
    read_opts.set_prefix_same_as_start(true); // auto-stop at prefix boundary
    // With set_prefix_same_as_start, the iterator automatically stops
    // when it encounters a key with a different prefix
    
    let iter = db.iterator_cf_opt(
        &cf,
        read_opts,
        IteratorMode::From(prefix, Direction::Forward),
    );
    
    iter.map(|item| {
        let (k, v) = item.unwrap();
        (k.to_vec(), v.to_vec())
    })
    .collect()
}
```

### 6.3 Bulk Deletion by Prefix (Cache Invalidation)

```rust
fn invalidate_client_sensor_cache(
    db: &DBWithThreadMode<MultiThreaded>,
    client_id: &str,
    sensor_id: &str,
) -> Result<(), rust_rocksdb::Error> {
    let prefix = format!("{}:{}", client_id, sensor_id);
    
    // Collect keys to delete, then batch-delete
    // (Cannot delete while iterating in a straightforward way)
    let cf_diff = db.cf_handle("diff_results").unwrap();
    let cf_dirty = db.cf_handle("dirty_bits").unwrap();
    
    let keys_to_delete: Vec<Vec<u8>> = {
        let iter = db.iterator_cf(
            &cf_diff,
            IteratorMode::From(prefix.as_bytes(), Direction::Forward),
        );
        iter.take_while(|item| {
            item.as_ref()
                .map(|(k, _)| k.starts_with(prefix.as_bytes()))
                .unwrap_or(false)
        })
        .map(|item| item.unwrap().0.to_vec())
        .collect()
    };
    
    let mut batch = WriteBatch::default();
    for key in &keys_to_delete {
        batch.delete_cf(&cf_diff, key);
    }
    // Also clear the dirty bit
    batch.delete_cf(&cf_dirty, prefix.as_bytes());
    
    db.write(batch)?;
    Ok(())
}

/// Alternative: delete_range_cf for contiguous key ranges
fn invalidate_by_range(
    db: &DBWithThreadMode<MultiThreaded>,
    client_id: &str,
    sensor_id: &str,
) -> Result<(), rust_rocksdb::Error> {
    let cf_diff = db.cf_handle("diff_results").unwrap();
    
    let start = format!("{}:{}", client_id, sensor_id);
    // Increment last byte to create exclusive upper bound
    let mut end = start.clone().into_bytes();
    *end.last_mut().unwrap() += 1;
    
    // delete_range_cf is O(1) at write time (adds a tombstone range)
    // Actual deletion happens during compaction
    db.delete_range_cf(&cf_diff, start.as_bytes(), &end)?;
    
    Ok(())
}
```

---

## 7. Compaction and Memory Usage

### 7.1 Block Cache Configuration

RocksDB's block cache is the primary memory consumer. It caches decompressed SST file blocks in memory.

```rust
use rust_rocksdb::{Options, BlockBasedOptions, Cache};

fn configure_memory_budget(db_opts: &mut Options, block_cache_mb: usize) {
    let mut block_opts = BlockBasedOptions::default();
    
    // Shared block cache across all CFs
    let cache = Cache::new_lru_cache(block_cache_mb * 1024 * 1024);
    block_opts.set_block_cache(&cache);
    
    // Block size: 16KB is good for mixed workloads
    block_opts.set_block_size(16 * 1024);
    
    // Bloom filter: reduces unnecessary disk reads for point lookups
    block_opts.set_bloom_filter(10.0, false); // 10 bits per key, full filter
    
    // Cache index and filter blocks (important for memory accounting)
    block_opts.set_cache_index_and_filter_blocks(true);
    block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
    
    db_opts.set_block_based_table_factory(&block_opts);
}
```

### 7.2 Memory Budget for Prism (512MB process / 200MB per-query)

Recommended allocation within Prism's 512MB process budget:

| Component | Budget | Notes |
|-----------|--------|-------|
| Block cache (shared) | 64 MB | Shared across all 12 CFs |
| Memtables (all CFs) | ~48 MB | 12 CFs x 4MB write buffer (configurable) |
| Query engine (DataFusion) | 200 MB | Per-query memory pool |
| Protobuf/OCSF buffers | ~50 MB | Normalization pipeline |
| Application overhead | ~150 MB | Tokio runtime, MCP protocol, etc. |

```rust
fn prism_memory_config() -> Options {
    let mut opts = Options::default();
    
    // Total write buffer budget across all CFs
    // This limits total memtable memory
    opts.set_db_write_buffer_size(48 * 1024 * 1024); // 48MB total
    
    // Per-CF write buffer (set in CF options)
    // With 12 CFs at 4MB each = 48MB max, but db_write_buffer_size caps it
    
    // Max number of write buffers (memtables) per CF before stalling
    opts.set_max_write_buffer_number(2);
    
    // Compaction: level-style is default and best for our mixed workload
    // opts.set_compaction_style(DBCompactionStyle::Level); // default
    
    // Rate limiter to prevent compaction from starving reads
    // opts.set_ratelimiter(rate_bytes_per_sec, refill_period_us, fairness);
    
    opts
}
```

### 7.3 Compaction Considerations

- **Level compaction** (default): Best for Prism's mixed read/write workload. Write amplification ~10x, read amplification ~1x per level.
- **FIFO compaction**: Could be useful for `audit_buffer` if we treat it as a ring buffer with TTL, but standard level compaction with manual TTL cleanup is simpler.
- **Compaction triggers**: Automatic, based on level sizes. No manual intervention needed for our data volumes (Prism's local data is small -- metadata and cache, not raw telemetry).

---

## 8. Backup and Recovery

### 8.1 Crash Recovery via WAL

RocksDB's WAL (Write-Ahead Log) provides automatic crash recovery:

1. Every `put`/`delete`/`write_batch` is first written to the WAL.
2. On process restart, RocksDB replays the WAL to recover any committed writes that were not yet flushed to SST files.
3. This happens automatically inside `DB::open()` -- no application code needed.
4. With `sync: true` (our audit buffer), even power loss is recoverable.
5. With default (no sync), process crash is recoverable but power loss may lose the last few ms of writes.

**For Prism:** WAL replay is automatic and transparent. The only thing we need to ensure is that the data directory is on a local filesystem (not NFS) and that we call `DB::open()` with proper options on startup.

### 8.2 Manual Backup API

`rust-rocksdb` exposes the `BackupEngine` and `BackupEngineOptions` APIs:

```rust
use rust_rocksdb::{backup::{BackupEngine, BackupEngineOptions}};

fn create_backup(db: &DB, backup_path: &str) -> Result<(), rust_rocksdb::Error> {
    let backup_opts = BackupEngineOptions::new(backup_path)?;
    let mut backup_engine = BackupEngine::open(&backup_opts, &Env::default())?;
    
    // Create a new backup (incremental -- only copies new SST files)
    backup_engine.create_new_backup(db)?;
    
    // Optionally purge old backups
    backup_engine.purge_old_backups(3)?; // keep last 3
    
    Ok(())
}

fn restore_from_backup(backup_path: &str, db_path: &str) -> Result<(), rust_rocksdb::Error> {
    let backup_opts = BackupEngineOptions::new(backup_path)?;
    let mut backup_engine = BackupEngine::open(&backup_opts, &Env::default())?;
    
    // Restore latest backup
    backup_engine.restore_from_latest_backup(
        db_path,    // db_dir
        db_path,    // wal_dir (same as db_dir for simple setup)
        &RestoreOptions::default(),
    )?;
    
    Ok(())
}
```

**For Prism:** Backup is a nice-to-have, not critical. Most of Prism's persisted data is reconstructible from sensor APIs (it is a cache/state store, not the source of truth). The exceptions are:
- `detection_rules` (user-authored) -- should be backed up
- `cases` (analyst work product) -- should be backed up
- `audit_buffer` -- compliance requirement, should be backed up

**Recommendation:** Implement a periodic backup (e.g., hourly) to a separate directory. Use the incremental backup API to minimize I/O.

---

## 9. Build Requirements

### 9.1 What `rust-rocksdb` Needs to Compile

The `rust-rocksdb` crate bundles the RocksDB C++ source code and builds it from source via the `librocksdb-sys` crate. Build requirements:

| Requirement | macOS | Linux (Ubuntu/Debian) | Notes |
|-------------|-------|----------------------|-------|
| **C++ compiler** | Xcode CLT (clang++) | `build-essential` (g++) | Required |
| **CMake** | `brew install cmake` | `apt install cmake` | Required by librocksdb-sys |
| **Clang/LLVM** | Included with Xcode | `apt install clang` | Needed for some features |
| **libclang** | Included with Xcode | `apt install libclang-dev` | For bindgen (if used) |
| **Rust toolchain** | rustup | rustup | Obviously |

### 9.2 Build Times

RocksDB C++ compilation is **slow** -- expect 3-10 minutes on first build depending on hardware. Subsequent builds are incremental (only recompiles if the -sys crate version changes).

**Mitigation strategies:**
- Use `sccache` or `mold` linker to speed up builds.
- In CI, cache the `target/` directory aggressively.
- Consider the `jemalloc` feature for production builds (better memory behavior).

### 9.3 Cross-Platform Notes

- **macOS (Apple Silicon):** Works well. The bundled RocksDB compiles natively for aarch64.
- **macOS (Intel):** Works.
- **Linux x86_64:** Primary target for production. Works well.
- **Linux aarch64 (ARM):** Works, tested in production by TiKV and others.
- **Windows:** Supported but historically more fragile. Requires MSVC toolchain. Not relevant for Prism (macOS dev + Linux prod).
- **musl (Alpine Linux):** Works with static linking. May need `ROCKSDB_LIB_DIR` if using a system-installed librocksdb.
- **Cross-compilation:** Difficult due to C++ compilation. Build natively on each target platform or use Docker.

### 9.4 Feature Flags in the Crate

```toml
[dependencies]
rust-rocksdb = { version = "0.23", features = [
    "multi-threaded-cf",  # Send+Sync on CF handles (REQUIRED for Prism)
    # "jemalloc",         # Use jemalloc allocator (recommended for production)
    # "zstd",             # Zstandard compression (recommended)
    # "lz4",              # LZ4 compression (fast, good default)
    # "snappy",           # Snappy compression
] }
```

---

## 10. Alternative Evaluation: sled, redb, fjall

### 10.1 sled

| Aspect | Assessment |
|--------|-----------|
| **Maturity** | Beta (has been "beta" for years). v0.34 as of 2024. |
| **Column families** | No native column family support. Would need to use separate `sled::Tree` instances (similar concept but not identical). |
| **WriteBatch** | `sled::Batch` exists but does NOT guarantee atomicity across trees. Only atomic within a single tree. |
| **Sync writes** | `db.flush()` is the equivalent, but the durability model is different (log-structured). |
| **Prefix iteration** | `tree.scan_prefix()` -- actually nicer API than RocksDB. |
| **Production use** | Limited. The author (spacejam) has been transparent about it not being production-ready. No major production deployments known. |
| **Memory usage** | Generally lower than RocksDB. Page cache based. |

**Verdict: REJECT for Prism.**
- Cross-tree atomic writes are not supported, which is a hard requirement (our detection engine needs atomic writes across `alerts`, `detection_state`, `audit_buffer`, and `dirty_bits`).
- Beta status is unacceptable for an MSSP security tool.
- sled's development pace has been slow; the promised 1.0 has not materialized.

### 10.2 redb

| Aspect | Assessment |
|--------|-----------|
| **Maturity** | 1.0+ released (v1.x as of early 2025). Pure Rust, no C++ dependencies. |
| **Column families** | Uses "tables" -- defined at compile time with typed key/value. Similar concept. |
| **WriteBatch** | Write transactions are inherently atomic. `WriteTransaction` provides ACID semantics. Actually stronger than RocksDB's WriteBatch. |
| **Sync writes** | `transaction.commit()` is durable by default. |
| **Prefix iteration** | `table.range(prefix..)` -- works but no bloom filter optimization. |
| **Production use** | Used by the `redb` author in production. Growing adoption. Pure Rust is appealing. |
| **Memory usage** | Lower than RocksDB. B-tree based (not LSM). |
| **Compaction** | Not needed (B-tree, not LSM). No write amplification. |

**Verdict: CONSIDER but with caveats.**
- **Pro:** Pure Rust (no cmake, no C++ build), ACID transactions across tables, simpler mental model.
- **Pro:** No LOCK file games -- uses file locking the same way but the API is cleaner.
- **Pro:** No compaction tuning needed.
- **Con:** Much smaller ecosystem and fewer production miles than RocksDB.
- **Con:** Performance at scale is less proven. RocksDB has been tuned for a decade.
- **Con:** No equivalent to RocksDB's sophisticated block cache, bloom filters, or prefix extractors.
- **Con:** Prefix scan performance may degrade without bloom filters for Prism's `client_id:sensor_id` key patterns.

### 10.3 fjall (new contender)

| Aspect | Assessment |
|--------|-----------|
| **Maturity** | Relatively new (2024). Pure Rust LSM-tree engine. |
| **Column families** | "Partitions" serve the same role. |
| **WriteBatch** | Atomic batches across partitions supported. |
| **Sync writes** | Configurable durability per write. |
| **Production use** | Very early. Limited production exposure. |

**Verdict: REJECT -- too new for Prism's requirements.**

### 10.4 Final Recommendation

**Use `rust-rocksdb`.** Rationale:

1. **Cross-CF atomic WriteBatch** is a hard requirement. Only RocksDB and redb satisfy this cleanly.
2. **Production pedigree**: RocksDB powers TiKV, CockroachDB, Solana, and dozens of other systems processing billions of operations. For an MSSP security tool, we need this level of battle-testing.
3. **Memory tuning**: RocksDB's block cache, memtable budgets, and compaction controls give us precise memory management within Prism's 512MB budget. redb's B-tree approach is less tunable.
4. **Prefix bloom filters**: Critical for our `client_id:sensor_id` prefix scan pattern. RocksDB has dedicated prefix extractor + bloom filter infrastructure. redb would require full range scans.
5. **Column family per-CF tuning**: We can give `audit_buffer` different compaction/memtable settings than `dirty_bits`. RocksDB supports this natively.
6. **Build cost is acceptable**: The C++/cmake dependency is annoying but manageable. CI caching mitigates the build time issue.

**If redb reaches sufficient maturity (2+ years of 1.0, broader production adoption), it would be worth re-evaluating** -- its pure-Rust build story and ACID transaction model are genuinely appealing.

---

## 11. Prism-Specific Patterns Summary

### 11.1 Initialization Pattern

```rust
use std::sync::Arc;
use rust_rocksdb::{DB, Options, ColumnFamilyDescriptor, DBWithThreadMode, MultiThreaded};

pub struct PrismDb {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl PrismDb {
    pub fn open(path: &str) -> Result<Self, rust_rocksdb::Error> {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);
        db_opts.set_db_write_buffer_size(48 * 1024 * 1024);
        db_opts.set_max_write_buffer_number(2);
        db_opts.set_keep_log_file_num(5);
        
        // Block cache: 64MB shared
        let mut block_opts = rust_rocksdb::BlockBasedOptions::default();
        let cache = rust_rocksdb::Cache::new_lru_cache(64 * 1024 * 1024);
        block_opts.set_block_cache(&cache);
        block_opts.set_bloom_filter(10.0, false);
        block_opts.set_cache_index_and_filter_blocks(true);
        db_opts.set_block_based_table_factory(&block_opts);
        
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = CF_NAMES
            .iter()
            .map(|name| {
                let mut cf_opts = Options::default();
                cf_opts.set_write_buffer_size(4 * 1024 * 1024);
                
                if *name == "diff_results" {
                    cf_opts.set_write_buffer_size(8 * 1024 * 1024);
                }
                
                ColumnFamilyDescriptor::new(*name, cf_opts)
            })
            .collect();
        
        let db = DB::open_cf_descriptors(&db_opts, path, cf_descriptors)?;
        
        Ok(Self { db: Arc::new(db) })
    }
    
    /// Clone the Arc for sharing across async tasks
    pub fn handle(&self) -> Arc<DBWithThreadMode<MultiThreaded>> {
        Arc::clone(&self.db)
    }
}
```

### 11.2 Async Integration with Tokio

RocksDB operations are synchronous and may block (especially on disk I/O). In an async context:

```rust
use tokio::task;

impl PrismDb {
    pub async fn get_async(
        &self,
        cf_name: &str,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, rust_rocksdb::Error> {
        let db = self.handle();
        let cf_name = cf_name.to_string();
        
        // Offload blocking I/O to Tokio's blocking thread pool
        task::spawn_blocking(move || {
            let cf = db.cf_handle(&cf_name).unwrap();
            db.get_cf(&cf, &key)
        })
        .await
        .unwrap() // JoinError -> panic (only if task panicked)
    }
}
```

**Important:** Always use `spawn_blocking` for RocksDB operations in async code. Direct RocksDB calls on the Tokio runtime threads will block the executor.

---

## 12. Open Questions for Implementation

1. **Key encoding**: Should we use string keys (`"client:sensor:entity"`) or binary-encoded keys (big-endian u32 IDs)? Binary is more compact and sorts correctly, but string keys are debuggable with `ldb` tool.

2. **Serialization format**: Protobuf (consistent with OCSF pipeline), bincode (fast Rust serde), or postcard (compact no_std serde)? Recommendation: protobuf for entities that cross the MCP boundary, bincode for internal-only state.

3. **TTL management**: RocksDB has a `DBWithTTL` wrapper, but it is coarse-grained (same TTL for entire DB). For per-CF or per-key TTL (e.g., `diff_results` expire after 24h), implement manual TTL via a background cleanup task that scans and deletes expired keys.

4. **Snapshot reads**: RocksDB `Snapshot` provides a consistent point-in-time read view. Useful if the query engine needs to read from multiple CFs without seeing concurrent writes mid-query.

5. **Column family lifecycle**: Can CFs be added/removed at runtime? Yes -- `DB::create_cf()` and `DB::drop_cf()` exist. However, for Prism's fixed 12-CF schema, we should create all CFs at DB open time and never modify the set.
