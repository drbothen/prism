# Ephemeral Data Lake Caching Strategy Research

**Date:** 2026-04-13
**Type:** General (technology/architecture)
**Status:** Complete
**Purpose:** Optimal caching strategy for Prism's ephemeral OCSF data lake, specifically caching sensor API responses that feed into the query engine.
**Training data reliance:** HIGH -- all external research tools (Context7, WebSearch, WebFetch) were denied during this research session. All findings below are from model training data (cutoff: May 2025). Version numbers and API details MUST be verified against live registries before implementation.

---

## 1. Hybrid Push-Down Strategy

### 1.1 Filters That CAN Be Pushed Down to Sensor APIs

Based on the behavioral contracts recovered from the four sensor pollers:

| Sensor | Push-Down Capable Filters | Native Syntax |
|--------|--------------------------|---------------|
| **CrowdStrike** | `created_date`, `updated_date`, `severity` (1-5), `status` (new/in_progress/etc), `hostname`, `tactic_id` | FQL filter syntax: `severity:>3+created_date:>'2026-04-01'` |
| **Cyberint** | `created_date` range, `severity`, `status`, `type` (52 AlertData subtypes) | JSON body params on POST |
| **Claroty xDome** | `timestamp` range, `risk_level`, `alert_status`, `site_id` | POST body filter arrays |
| **Armis** | `time` range, `riskLevel`, `type`, any AQL-expressible field | AQL WHERE clauses (full query language) |

**Key insight:** Armis is the most capable for push-down because AQL is a full query language. CrowdStrike FQL is moderately expressive. Cyberint and Claroty have limited filter vocabularies.

### 1.2 Filters That MUST Be Applied in the Query Engine

These filters only exist after OCSF normalization and cannot be translated back to sensor-native queries:

- **OCSF `device.ip`** -- CrowdStrike has `local_ip`, Armis has `ipaddress`, Claroty has `ip_address`. No universal push-down possible; the field only exists as a unified concept post-normalization.
- **OCSF `device.hostname`** -- similar; different source fields per sensor.
- **OCSF `severity_id`** -- OCSF severity enum (0-6) maps differently from each sensor's native severity scale. CrowdStrike uses 1-5, Cyberint uses string labels, etc.
- **Cross-sensor correlation predicates** -- e.g., "alerts where device.ip matches across CrowdStrike AND Claroty" -- inherently post-normalization.
- **OCSF `category_uid`, `class_uid`, `type_uid`** -- event classification is assigned during normalization.
- **`raw_extensions.*` fields** -- while these are sensor-native, they arrive in OCSF wrapper and may not map back cleanly.

### 1.3 Push-Down Decision at Query Planning Time

Recommended approach -- a two-phase query planner:

**Phase 1: Parse AxiQL and classify predicates**

```
For each WHERE predicate:
  1. Is the field an OCSF-only field? -> post-filter
  2. Is the field a sensor-native field with known mapping? -> push-down candidate
  3. Is the predicate supported by the target sensor's query syntax? -> push-down
  4. Otherwise -> post-filter
```

**Phase 2: Build cache key from push-down params**

The cache key should be: `(client_id, sensor_id, source_id, pushed_down_params_hash)`

Where `pushed_down_params_hash` is a deterministic hash of the normalized push-down parameters (sorted by field name, canonicalized values). This is similar to the existing `query_params_hash` in `CacheKey` but explicitly scoped to only the pushed-down portion.

### 1.4 Cache Key Design Implications

**Current design:** `(TenantId, SensorId, query_params_hash)` -- the full query hash means different AxiQL queries that need the same underlying sensor data will not share cache entries.

**Recommended design:** `(TenantId, SensorId, SourceId, pushed_down_params_hash)`

This means:
- Two queries asking for "CrowdStrike alerts severity > 3 last 24h" with different OCSF-level filters will share the same cache entry for the sensor fetch.
- The OCSF-level filtering happens post-cache, over the cached (and normalized) data.
- Cache hit rate increases substantially because the cache key captures only what the sensor API actually receives.

**Trade-off:** More data is cached than any single query needs. This wastes some memory but dramatically improves cache sharing. Given that Prism caches are already bounded (1000 entries per client), this is acceptable.

---

## 2. Cache Granularity: Raw vs. Normalized

### 2.1 Option A: Cache Raw Sensor Responses (Pre-Normalization)

**Pros:**
- Saves normalization cost on cache hit
- Preserves full sensor fidelity; future normalization changes do not invalidate cache
- Simpler cache invalidation (sensor response is the ground truth)

**Cons:**
- Different queries cannot share cache if they need different raw fields (but with push-down-only keys, this is less of an issue since the raw data is the complete sensor response)
- OCSF normalization must run on every cache hit before query execution
- Cache entries are in sensor-native format, making cross-sensor queries require normalization before joining

### 2.2 Option B: Cache Normalized OCSF Data (Post-Normalization)

**Pros:**
- Higher cache reuse across queries -- any AxiQL query can use cached OCSF data directly
- Cross-sensor joins work directly from cache
- Query engine operates on a uniform schema
- Avoids redundant normalization on repeated access

**Cons:**
- Normalization cost on every cache write (but normalization happens once per cache miss, not per query)
- If OCSF mapping logic changes, cached data may be stale (mitigated by TTL)
- Slightly larger cache entries if OCSF adds wrapper fields

### 2.3 Option C: Two-Layer Cache (Raw + Normalized)

**Pros:**
- Maximum flexibility; can serve both use cases
- Raw cache useful for debugging/audit

**Cons:**
- Double memory usage
- Complex invalidation logic
- Premature optimization for a system with 60s/300s TTLs

### 2.4 Recommendation: Cache Normalized OCSF Data (Option B)

**Rationale:**

1. **Prism's primary use case is cross-sensor OCSF queries.** Every query ultimately needs OCSF data. Caching raw data means normalization runs on every cache hit, which is wasteful.

2. **Normalization is not free but it is not the bottleneck.** The sensor API HTTP round-trip (100-2000ms) dominates. Normalization (DynamicMessage field mapping) is CPU-bound and likely <10ms for a typical batch of records. The cost of normalizing once per cache write is negligible compared to the API call it replaces.

3. **Cache sharing is the primary goal.** With push-down-only cache keys, multiple AxiQL queries share the same cached OCSF data. This only works if the cache stores the normalized form.

4. **TTLs are short (60s-300s).** OCSF mapping changes are a deployment event, not a runtime event. A 60-second TTL means stale OCSF mappings last at most 60 seconds, which is acceptable.

5. **Memory is the scarce resource.** Two-layer caching doubles memory usage for marginal benefit in Prism's use case.

**Cache entry format:**

```rust
struct CacheEntry {
    /// OCSF-normalized records as Arrow RecordBatch
    records: RecordBatch,
    /// Original sensor response metadata (pagination cursors, total count)
    metadata: SensorResponseMetadata,
    /// Insertion time for TTL calculation
    inserted_at: Instant,
    /// Number of records (for memory budget tracking)
    record_count: usize,
    /// Approximate memory size in bytes
    approx_bytes: usize,
}
```

---

## 3. Memory Bounds for Materialized Tables

### 3.1 Arrow RecordBatch vs. Vec<serde_json::Value> Memory Footprint

**Arrow RecordBatch (columnar):**
- Fixed overhead: ~200-500 bytes per column (ArrayData metadata, buffers, null bitmaps)
- String columns: raw bytes + offsets array (4 bytes per entry) + validity bitmap
- Integer columns: 8 bytes per value (i64) + validity bitmap
- Typical OCSF event (~20 fields, mix of strings and ints): ~500-800 bytes per record
- For 10K records: approximately 5-8 MB
- For 100K records: approximately 50-80 MB
- **Key advantage:** columnar format means the query engine (DataFusion) can operate directly on RecordBatch without conversion. Zero-copy query execution.

**Vec<serde_json::Value> (row-oriented JSON DOM):**
- Per-value overhead: serde_json::Value is an enum, 32 bytes on 64-bit (8 bytes tag + 24 bytes payload)
- String values: 24 bytes (String struct) + heap allocation (string bytes + allocator overhead)
- Object/Map: BTreeMap or IndexMap with per-entry overhead (~80-120 bytes per key-value pair)
- Typical OCSF event (~20 fields): ~2000-4000 bytes per record (4-5x larger than Arrow)
- For 10K records: approximately 20-40 MB
- For 100K records: approximately 200-400 MB
- **Key disadvantage:** the query engine must convert to Arrow before executing SQL/DataFusion queries, adding both time and peak memory (2x during conversion).

**Conclusion:** Arrow RecordBatch is 3-5x more memory efficient than serde_json::Value for structured data, and avoids the conversion cost. Use RecordBatch as the canonical in-memory format.

### 3.2 Recommended Memory Bounds

| Metric | Recommended Limit | Rationale |
|--------|-------------------|-----------|
| Max records per materialized table | 50,000 | Balances completeness with memory. At ~700 bytes/record in Arrow, this is ~35 MB. |
| Max total materialized bytes | 256 MB | Hard limit across all concurrent materializations. Prevents OOM on a typical analyst workstation (8-16 GB RAM). |
| Max records per sensor response (pre-materialization) | 10,000 | Per-sensor cap prevents a single sensor from consuming the entire budget. |
| Max concurrent materializations | 4 | One per active query; prevents memory multiplication. |

**Scaling math for worst case:**
- 50 clients x 4 sensors x 3 sources = 600 potential cache entries
- At 1000 entries per client (existing LRU bound), this is well-bounded
- But materialized tables for cross-client queries (`tenant_id: null`) could be large
- 50 clients x 10K records per sensor = 500K records -- this MUST be refused

### 3.3 Communicating "Narrow Your Query" to the AI Agent

Return structured error in the MCP `CallToolResult` with actionable guidance:

```json
{
  "isError": true,
  "content": [{
    "type": "text",
    "text": "Query scope too broad: estimated 250,000 records across 50 clients and 4 sensors exceeds the 50,000 record materialization limit.\n\nSuggestions to narrow scope:\n1. Add time_range filter: WHERE time > '2026-04-12T00:00:00Z'\n2. Specify client_id instead of querying all clients\n3. Filter by severity: WHERE severity_id >= 4\n4. Limit to specific sensors: sensor='crowdstrike'\n\nCurrent query would fetch:\n- crowdstrike: ~80,000 records (50 clients)\n- cyberint: ~60,000 records (50 clients)\n- claroty: ~110,000 records (50 clients)\n\nEstimated memory: ~175 MB (limit: 256 MB)"
  }],
  "_meta": {
    "error_category": "scope_too_broad",
    "estimated_records": 250000,
    "record_limit": 50000,
    "estimated_bytes": 183500800,
    "byte_limit": 268435456,
    "narrowing_suggestions": [
      {"type": "time_range", "suggestion": "Add WHERE time > now() - 24h"},
      {"type": "client_scope", "suggestion": "Specify client_id parameter"},
      {"type": "severity_filter", "suggestion": "Add WHERE severity_id >= 4"},
      {"type": "sensor_filter", "suggestion": "Limit to specific sensor"}
    ]
  }
}
```

**Key design principle:** The AI agent reads the structured `_meta` and the human-readable text. Provide both. Include specific numbers (estimated records, limits) so the agent can make informed decisions about how much to narrow.

### 3.4 Scope Estimation Before Fetch

Before actually fetching, Prism should estimate the result size:

1. **Count-only pre-query:** Some sensor APIs support count-only queries (CrowdStrike's QueryV2 returns `total_count`, Armis AQL supports `COUNT`). Use these before fetching full records.
2. **Historical cache statistics:** Track average record counts per (client, sensor, source, time_range). Use this as an estimate for new queries.
3. **Conservative fallback:** If no estimate is available, assume worst case and require explicit time_range or client_id.

---

## 4. Cache Warming and Prefetch

### 4.1 On-Demand vs. Prefetch Analysis

| Strategy | Pros | Cons |
|----------|------|------|
| **Pure on-demand (current)** | Simple; no wasted API calls; no rate limit consumption at idle | First query per session is always a cache miss (cold start) |
| **Startup warm** | First query is fast | Wastes API calls if analyst does not query that sensor; consumes rate budget; delays startup |
| **Background refresh** | Frequently-queried data stays fresh | Complex; API rate limit management; unclear which combinations to refresh |

### 4.2 Recommendation: On-Demand with Optional Lazy Warming

**Phase 1 (initial implementation):** Pure on-demand caching. This is correct for Prism's use case:
- Analyst sessions are interactive and unpredictable. Prefetching "all clients x all sensors" wastes rate budget.
- ASM-004 (rate limit assumption) is still UNVALIDATED. Until rate limits are characterized, conservative API usage is wise.
- Cache TTLs are short (60s-300s), so prefetched data expires before the analyst might use it.

**Phase 2 (if rate limits allow):** Lazy background refresh for cache entries that have been accessed at least N times (N=2) within their TTL window.

```rust
/// When a cache entry is accessed, record the access.
/// If the entry has been accessed >= REFRESH_THRESHOLD times
/// and is within REFRESH_WINDOW of expiry, schedule a background refresh.
const REFRESH_THRESHOLD: u32 = 2;
const REFRESH_WINDOW: Duration = Duration::from_secs(15); // refresh when < 15s of TTL remains

fn on_cache_access(&self, key: &CacheKey, entry: &CacheEntry) {
    entry.access_count.fetch_add(1, Ordering::Relaxed);
    let remaining_ttl = entry.ttl() - entry.age();
    if entry.access_count() >= REFRESH_THRESHOLD && remaining_ttl < REFRESH_WINDOW {
        self.background_refresh_queue.send(key.clone());
    }
}
```

This avoids startup cost, avoids wasting rate budget, and only refreshes data that the analyst is actively using.

### 4.3 Explicit Warm Command (Optional Phase 3)

Expose an MCP tool `warm_cache` that the AI agent can call proactively:

```
warm_cache(client_id: "acme", sensors: ["crowdstrike"], time_range: "last_24h")
```

This gives the agent control over warming without Prism guessing.

---

## 5. Existing Patterns from Federated Query Engines

### 5.1 Trino/Presto Caching for Federated Queries

**NOTE:** This section is from training data (cutoff May 2025). Verify against current Trino documentation.

Trino (formerly PrestoSQL) uses a multi-level caching strategy for federated queries:

1. **Connector-level caching:** Each connector (MySQL, Hive, Cassandra, etc.) can implement its own result caching. The `CachingConnectorMetadata` wrapper caches table/column metadata. Data caching is connector-specific.

2. **Split-level caching (Rubix/Alluxio):** Trino can cache data splits in a local cache (SSD or memory) via Alluxio integration. This caches the raw data read from connectors before query processing. Cache key is the split identifier (file path + offset + length for file-based connectors).

3. **Query result caching (Trino 400+):** Trino has experimental query result caching where identical queries return cached results. This is at the query layer, not the connector layer.

4. **Predicate push-down:** Trino aggressively pushes predicates to connectors. The `ConnectorMetadata.applyFilter()` API lets connectors declare which predicates they can handle. Remaining predicates are applied by Trino's query engine. This is directly analogous to Prism's push-down strategy.

**Relevance to Prism:**
- Prism's sensor adapters are analogous to Trino connectors.
- The push-down pattern (connector declares capabilities, engine applies remainder) maps directly.
- Trino's split-level caching is analogous to caching normalized OCSF data per sensor fetch.
- Trino does NOT cache at the connector response level by default -- connectors are expected to handle their own caching.

### 5.2 DataFusion Caching of Table Scans

**NOTE:** This section is from training data (cutoff May 2025). DataFusion may have changed.

Apache DataFusion (the Rust-native query engine) does not have built-in caching of table scans. Its architecture:

1. **TableProvider trait:** Custom data sources implement `TableProvider` and `ExecutionPlan`. DataFusion calls `scan()` on each query, which produces a stream of `RecordBatch`es.

2. **No implicit caching:** DataFusion does not cache `RecordBatch` results between queries. Each `scan()` call is expected to produce fresh data.

3. **User-space caching:** Projects using DataFusion implement caching in their `TableProvider::scan()` implementation. This is exactly where Prism should cache -- in the sensor adapter's scan implementation.

4. **MemTable:** DataFusion provides `MemTable` for in-memory tables backed by `Vec<RecordBatch>`. This is the materialization target for Prism's ephemeral data lake.

**Relevance to Prism:**
- Prism should implement `TableProvider` for each sensor, with caching inside the `scan()` method.
- The materialized ephemeral table is a `MemTable` containing cached + freshly-fetched `RecordBatch`es.
- DataFusion handles query execution (filter, project, join, aggregate) over the `MemTable`.
- This is a clean separation: caching is in the data source layer, query execution is in DataFusion.

### 5.3 Rust Crates for Async LRU Caching

**NOTE:** Version numbers from training data. MUST verify against crates.io before use.

| Crate | Last Known Version | Features | Async Support | Fit for Prism |
|-------|--------------------|----------|---------------|---------------|
| **moka** | 0.12.x (as of early 2025) | Concurrent cache inspired by Java's Caffeine. LRU/LFU hybrid eviction. Per-entry TTL. Max capacity by entry count or weighted size. Async support via `moka::future::Cache`. Entry-level expiration listeners. | Yes, first-class via `future::Cache` | **Best fit.** Concurrent, async-native, weighted size eviction (critical for memory budget), per-entry TTL. |
| **cached** | 0.53.x (as of early 2025) | Proc-macro based caching (`#[cached]`). Multiple backends (HashMap, LRU, Redis, disk). Async support. TTL and TTI (time-to-idle). | Yes, via `#[cached]` on async fns | Good for simple function-level caching. Less suitable for Prism's structured cache with composite keys and weighted eviction. |
| **mini-moka** | 0.10.x (as of early 2025) | Subset of moka without crossbeam dependency. Single-thread focused. | Limited | Not recommended -- Prism needs concurrent access from multiple tokio tasks. |
| **lru** | 0.12.x (as of early 2025) | Simple LRU cache. No TTL, no async, no weighted eviction. Sync only. | No | Currently in ADR-011. Too simple for the data lake model -- no TTL, no weighted eviction, no async. |
| **quick_cache** | 0.6.x (as of early 2025) | Concurrent cache with weighted eviction. Based on CLOCK-Pro algorithm. | Yes | Alternative to moka. Less mature but potentially faster. |

### 5.4 Recommendation: moka

**moka** is the strongest fit for Prism's ephemeral data lake cache because:

1. **Weighted eviction:** Prism needs to bound total cache memory, not just entry count. moka supports `weigher` functions that return the byte size of each entry, with a `max_capacity` in total weight. This directly implements the 256 MB memory budget.

2. **Per-entry TTL:** Different data types have different TTLs (60s for alerts, 300s for devices). moka supports `expire_after` with per-entry control.

3. **Async-native:** `moka::future::Cache` works directly with tokio. Cache misses can trigger async sensor fetches without blocking.

4. **Concurrent access:** Multiple AxiQL queries running concurrently can read/write the cache without external locking. moka uses lock-free concurrent data structures internally.

5. **Entry loading (get_with):** `cache.get_with(key, async || { fetch_from_sensor().await })` automatically coalesces concurrent requests for the same key -- if two queries need the same sensor data simultaneously, only one API call is made.

**Migration from `lru` crate:** ADR-011 currently specifies the `lru` crate. This should be updated to `moka` with the following changes:
- Replace `lru::LruCache` with `moka::future::Cache`
- Replace `max_entries` config with `max_capacity` (in bytes)
- Add `weigher` function that returns `entry.approx_bytes`
- Replace manual TTL checking with moka's `expire_after`
- Use `get_with` for coalesced async loading

---

## 6. Integrated Cache Architecture Recommendation

### 6.1 Cache Key Design

```rust
/// Cache key for sensor data. Only includes push-down parameters,
/// NOT AxiQL-level filters (those are applied post-cache).
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct SensorCacheKey {
    tenant_id: TenantId,
    sensor_id: SensorId,
    source_id: SourceId,
    /// Hash of the canonicalized push-down parameters sent to the sensor API.
    /// Produced by sorting params by name and hashing with a deterministic hasher.
    pushed_params_hash: u64,
}
```

### 6.2 Cache Entry Design

```rust
struct SensorCacheEntry {
    /// OCSF-normalized data in Arrow columnar format
    batches: Vec<RecordBatch>,
    /// Metadata from the sensor response (pagination state, total count)
    sensor_metadata: SensorResponseMetadata,
    /// Number of records across all batches
    record_count: usize,
}

impl SensorCacheEntry {
    /// Approximate memory size for moka's weigher
    fn weight(&self) -> u32 {
        let arrow_bytes: usize = self.batches.iter()
            .map(|b| b.get_array_memory_size())
            .sum();
        // Cap at u32::MAX for moka's weigher API
        arrow_bytes.min(u32::MAX as usize) as u32
    }
}
```

### 6.3 Query Execution Flow (Updated)

```
1. Parse AxiQL query
2. Resolve scope: clients x sensors x sources
3. Classify predicates: push-down vs. post-filter
4. For each (client, sensor, source):
   a. Compute SensorCacheKey from push-down params
   b. cache.get_with(key, || async {
        fetch_from_sensor(push_down_params).await
        normalize_to_ocsf(raw_response).await
        build_record_batches(ocsf_records)
      })
   c. Accumulate RecordBatches
5. Check total record count against limit (50K)
6. If over limit: return structured "narrow your query" error
7. Materialize as MemTable (Vec<RecordBatch>)
8. Register MemTable with DataFusion SessionContext
9. Execute AxiQL (translated to SQL) with post-filters
10. Return results with trust_level and provenance metadata
```

### 6.4 Configuration

```toml
[cache]
# Total memory budget for cached sensor data
max_bytes = "256MB"

# Per-data-type TTL
[cache.ttl]
alerts = "60s"
devices = "300s"
vulnerabilities = "300s"
activity_events = "120s"
default = "120s"

# Materialization limits
[cache.materialization]
max_records = 50000
max_bytes = "256MB"
max_concurrent = 4

# Background refresh (Phase 2, disabled by default)
[cache.refresh]
enabled = false
access_threshold = 2
refresh_window = "15s"
```

---

## 7. Open Questions and Risks

| ID | Question | Impact | Recommendation |
|----|----------|--------|----------------|
| OQ-1 | What are actual sensor API rate limits? (ASM-004 unvalidated) | Determines whether any caching strategy is sufficient, or if request coalescing is mandatory | Validate ASM-004 before Phase 2 cache warming |
| OQ-2 | Does DataFusion's `MemTable` support efficient predicate push-down for post-filters? | If not, post-filter performance may suffer on large materialized tables | Test with DataFusion; MemTable should support filter push-down into Arrow compute kernels |
| OQ-3 | What is moka's actual memory overhead per entry? | Affects the 256 MB budget accuracy | Benchmark with realistic OCSF RecordBatches |
| OQ-4 | Should cross-client queries (`tenant_id: null`) have a separate, lower record limit? | 50 clients x 10K records = 500K, which is way over the 50K limit | Yes; cross-client limit should be the same 50K but with explicit per-client caps (e.g., 1000 per client in cross-client mode) |
| OQ-5 | How to handle sensor APIs that do not support count-only queries? | Cannot estimate scope before fetching | Use streaming with early termination: fetch up to limit+1, if more exist return "scope too broad" |
| OQ-6 | moka crate version and API stability | Training data version may be outdated | MUST verify current moka version on crates.io before adding to Cargo.toml |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Context7 | 3 (denied) | Attempted: moka, DataFusion, Arrow crate documentation |
| WebSearch | 3 (denied) | Attempted: moka features, DataFusion caching, Trino caching |
| WebFetch | 3 (denied) | Attempted: crates.io API for moka, cached, mini-moka versions |
| Training data | 6 areas | Trino/Presto caching architecture, DataFusion TableProvider/MemTable patterns, Arrow RecordBatch memory model, moka/cached/lru crate comparison, sensor API push-down patterns (from project specs), memory estimation math |

**Total MCP tool calls:** 9 (all denied)
**Training data reliance:** HIGH -- All external research tools were denied. Every version number, API detail, and crate feature claim in this document is from model training data (cutoff May 2025) and MUST be verified against live sources before implementation decisions are finalized.
