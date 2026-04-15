---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Config-Driven Adapters & Hot Reload"
capability: "CAP-030"
---

# BC-2.16.006: Arc-Swap Config Access on Hot Path — Lock-Free Reads for Query-Time Config Access

## Preconditions
- Prism has loaded a `ConfigSnapshot` at startup
- The `ConfigSnapshot` is stored in an `arc_swap::ArcSwap<ConfigSnapshot>` shared across all async tasks

## Postconditions
- All query-time config reads use `ArcSwap::load()` which returns an `arc_swap::Guard<Arc<ConfigSnapshot>>` — a lock-free atomic read
- No mutex, RwLock, or other blocking synchronization is on the query hot path for config access
- The `Guard` holds a reference to the `Arc<ConfigSnapshot>` that was current at the time of `load()` — subsequent swaps do not affect the guard's reference
- A query that begins execution with ConfigSnapshot v1 continues using v1 for its entire lifecycle, even if a reload swaps in v2 mid-query (DEC-037)
- The `arc_swap::ArcSwap::store()` method is used by `reload_config` (BC-2.16.005) to atomically replace the current snapshot — this is the only write path
- `store()` is called from the `reload_config` tool handler, which runs on the Tokio runtime — no blocking I/O occurs during the swap itself

## Performance Characteristics
- `ArcSwap::load()` is wait-free on x86_64 (single atomic load + reference count increment)
- No contention between concurrent query executions reading config
- The swap operation (`store()`) is O(1) and does not block readers
- Old `ConfigSnapshot` values are freed when the last `Guard` referencing them is dropped (automatic via `Arc` reference counting)

## Memory Management
- At most 2 `ConfigSnapshot` instances exist simultaneously: the current one and the one being replaced (while old guards are still held by in-flight queries)
- `ConfigSnapshot` is immutable after construction — no interior mutability
- Each `ConfigSnapshot` contains cloned data (not references to parsed TOML nodes) so the original file contents can be freed after parsing

## Error Handling
- `ArcSwap` operations are infallible — there are no error conditions for `load()` or `store()`
- If the `ArcSwap` is somehow uninitialized (should never happen after startup), accessing it panics — this is a programming error caught by tests

## Traces
- CAP-030 (Hot Configuration Reload)
- DEC-037 (In-flight query uses old schema snapshot)
- DEC-039 (In-flight execution uses old credentials)
