---
document_type: architecture-section
level: L3
section: "concurrency-architecture"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, domain-spec/invariants.md]
traces_to: ARCH-INDEX.md
---

# Concurrency Architecture

## Threading Model

### Decision: Tokio Multi-Threaded Runtime (AD-013)

**Status:** accepted
**Context:** Prism needs concurrent sensor fan-out, DataFusion uses tokio internally, rmcp requires tokio.
**Decision:** Tokio multi-threaded runtime (default configuration).
**Rationale:** All major dependencies (rmcp, DataFusion, reqwest) require tokio. Multi-threaded runtime enables parallel sensor fan-out across CPU cores. The single-process model (DI-017) means all concurrency is within-process.
**Consequences:** All async functions run on the tokio runtime. Blocking operations (RocksDB reads, file I/O) use `tokio::task::spawn_blocking`. No manual thread management.

## Shared State

| State | Type | Protection | Access Pattern | Contention Risk |
|-------|------|-----------|---------------|-----------------|
| ConfigSnapshot | `ArcSwap<ConfigSnapshot>` | Lock-free read, atomic swap on reload | Read-heavy (every query), write-rare (reload) | None (lock-free) |
| AdapterRegistry | `Arc<AdapterRegistry>` | Rebuilt on config reload, swapped atomically | Read-heavy (every query) | None (immutable after construction) |
| RocksDB | `Arc<DB>` (multi-threaded mode) | RocksDB internal locking per column family | Mixed (reads per query, writes per schedule/detection) | Low (column family isolation) |
| ConfirmationTokenStore | `Arc<Mutex<HashMap<String, Token>>>` | Mutex | Write on create/consume, read on validate | Low (100 max tokens, fast operations) |
| CursorStore | `Arc<Mutex<HashMap<String, Cursor>>>` | Mutex | Write on create, read on page fetch, cleanup on expiry | Low (200 max cursors, per-query lifetime) |
| ResponseCache | `Arc<Mutex<LruCache>>` per (client, sensor) | Mutex per cache instance | Read/write per query | Low (per-client-per-sensor partitioning reduces contention) |
| Scheduler State | `Arc<Mutex<SchedulerState>>` | Mutex | Write on schedule creation/deletion, read on tick | Low (ticks are infrequent vs query load) |
| Schedule Semaphore | `Arc<Semaphore>` (16 permits) | Tokio semaphore | Acquire on schedule execution, release on completion | Medium (bounds concurrent API fan-out) |
| Decorator Cache | `Arc<RwLock<HashMap<String, Value>>>` | RwLock | Read-heavy (every query), write-rare (periodic refresh) | None (RwLock favors readers) |

## Concurrency Patterns

### Sensor Fan-Out

Per-query fan-out uses `tokio::JoinSet` for concurrent `(client_id, sensor_id)` API calls:

```rust
let mut join_set = JoinSet::new();
for (client_id, sensor_id) in scope {
    let adapter = registry.get(client_id, sensor_id)?;
    join_set.spawn(async move {
        adapter.fetch(table, push_down_filters).await
    });
}
// Collect results, partial failures go to sensor_errors
while let Some(result) = join_set.join_next().await { ... }
```

Cross-client fan-out is bounded by a configurable concurrency semaphore (default 10) per query to prevent API rate limit exhaustion.

### Schedule Execution

Scheduled queries run on spawned tokio tasks, gated by a 16-permit semaphore:

```rust
let permit = schedule_semaphore.acquire().await?;
tokio::spawn(async move {
    let _permit = permit; // held for duration
    execute_scheduled_query(schedule, query_engine).await
});
```

Excess executions are skipped (not queued) when all permits are held.

### Blocking I/O

RocksDB operations and credential store access use `spawn_blocking`:

```rust
let value = tokio::task::spawn_blocking(move || {
    db.get_cf(&cf_handle, key)
}).await??;
```

## Deadlock Prevention

1. **No nested locks.** Each shared state has exactly one lock. No code path acquires lock A then lock B.
2. **Lock ordering is trivial.** The only multi-lock scenario is ConfigSnapshot reload, which uses arc-swap (lock-free) — no Mutex involved.
3. **Timeouts on all async operations.** `tokio::time::timeout` wraps query execution (30s), sensor API calls (per NFR-001), and schedule execution.
4. **No blocking in async context.** All blocking operations go through `spawn_blocking`. Mutex guards are never held across `.await` points.

## Concurrency Invariants

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| CI-001 | ConfigSnapshot reads are wait-free | arc-swap: single atomic load, no lock |
| CI-002 | In-flight queries see a consistent config snapshot | Arc reference captured at query start; mid-query reloads do not affect |
| CI-003 | RocksDB access is serialized per column family | RocksDB internal locking + column family isolation |
| CI-004 | No Mutex guards held across await points | Code review convention + clippy lint `await_holding_lock` |
| CI-005 | Schedule execution is bounded at 16 concurrent | Tokio semaphore with fixed permit count |
| CI-006 | Cursor and token stores never exceed their caps | Cap check under lock before insertion |
