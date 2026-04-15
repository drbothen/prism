# osquery Feature Deep Dive for Prism

Deep architectural analysis of 10 osquery features, evaluated for Prism adoption
(an ephemeral federated query engine over security sensor APIs for MSSP analysts).

---

## 1. Scheduled Queries

### How osquery implements it

**Key files:** `osquery/dispatcher/scheduler.h`, `osquery/dispatcher/scheduler.cpp`, `osquery/core/sql/scheduled_query.h`

The scheduler is a `SchedulerRunner` thread (an `InternalRunnable` dispatched via osquery's `Dispatcher`). It operates on a simple **tick-based loop** where the counter `i` increments once per second (the interval). Each tick, it iterates all scheduled queries via `Config::get().scheduledQueries()` and fires any query where `i % query.splayed_interval == 0`.

The `ScheduledQuery` struct captures:
- `interval` -- how often to run (seconds)
- `splayed_interval` -- the actual execution interval, jittered by `schedule_splay_percent` (default 10%) to avoid thundering herds
- `startup_priority` -- queries with a priority run on first tick regardless of interval
- `options["snapshot"]` -- if true, emit full results (no diff); if false, compute differential
- `options["removed"]` -- whether to report removed rows in differential mode

**Differential result computation** happens in `launchQuery()`:
1. Execute the SQL query
2. Populate a `QueryLogItem` with metadata (host identifier, timestamp, epoch, decorations)
3. If snapshot mode: emit all rows directly via `logSnapshotQuery()`
4. If differential mode: create a `Query` database object, call `addNewResults()` which compares current results against the stored previous results and produces `DiffResults` (added/removed rows)
5. If no changes and not a periodic snapshot, skip logging entirely
6. Otherwise `logQueryLogItem()` sends the diff to the logger plugin

**Splay persistence:** The `restoreSplayedValue()` function saves the computed splay to RocksDB keyed as `interval.<name>`. On restart, if the query name and interval match, the same splay is reused for deterministic scheduling.

**Time drift compensation:** `calculateTimeDriftAndMaybePause()` accumulates drift from slow query execution and compensates by shortening future pauses, up to `schedule_max_drift` (default 60s), after which drift is dropped.

**Performance monitoring:** Before and after each query, the scheduler snapshots the process's `resident_size`, `user_time`, and `system_time` from the `processes` table and records the delta via `Config::recordQueryPerformance()`.

### How it maps to Prism

Prism's equivalent: "check all clients for critical CrowdStrike alerts every 5 minutes and notify me of changes." The scheduler pattern maps directly to Prism running federated queries on intervals, with differential results being the core value proposition for an MSSP ("what changed since last check?").

**Prism-tailored version:**
- `ScheduledFederatedQuery`: wraps a Prism SQL query + target clients + interval
- Splay by client hash to spread API load (osquery splays by hostname SHA1 -- Prism splays by client_id hash)
- Differential engine stores previous results per (query_name, client_id) pair in memory or SQLite
- Result notification: emit only changes (new alerts, resolved alerts) to a notification channel
- Epoch/counter tracking for exactly-once delivery semantics

### Priority: **P1 (should-have)**

The interactive REPL is P0. Scheduled queries add enormous value for continuous monitoring ("alert me when something changes across my 50 clients") but the initial release can ship with ad-hoc queries only. Scheduled queries should follow quickly.

---

## 2. Packs

### How osquery implements it

**Key files:** `osquery/config/packs.h`, `osquery/config/packs.cpp`, `packs/incident-response.conf`

A **Pack** is a named collection of `ScheduledQuery` objects with metadata:
- `name`, `source` -- identity and provenance
- `platform` -- restrict to "darwin", "linux", "windows", or "any"
- `version` -- minimum osquery version required
- `shard` -- percentage-based sampling (0-100); only hosts whose `SHA1(hostname) % 255 * 100/255 < shard` execute the pack
- `discovery` -- an array of SQL queries that must all return >= 1 row for the pack to be active (e.g., `SELECT 1 FROM os_version WHERE major >= 10`)
- `queries` -- the actual query definitions with interval, snapshot/removed options, and per-query platform/version/shard overrides

**Discovery caching:** `Pack::checkDiscovery()` caches the discovery result for `pack_refresh_interval` (default 3600s) seconds, avoiding re-running expensive discovery queries every tick.

**Pack sources:** Packs can be defined inline in the config JSON or as external file paths. When a pack value is a string path, the active `ConfigPlugin` resolves it via `genPack()`.

**Example from `incident-response.conf`:** 28 queries covering persistence mechanisms (launchd, crontab, startup_items), firewall settings, login history, open sockets, listening ports, kernel modules, shell history, disk encryption. Each query has an interval (3600s or 86400s), platform filter, and a human-readable `description` and `value` explaining why the query matters for IR.

**Schedule ordering:** Queries within a pack are sorted by `startup_priority` then by name, allowing critical queries to run first.

### How it maps to Prism

Prism's equivalent: **Query Packs** -- curated bundles of federated queries for specific MSSP workflows.

**Prism-tailored version:**
```toml
[packs.incident-response]
description = "Standard IR data collection across all sensors"
discovery = "SELECT 1 FROM prism_clients WHERE sensor = 'crowdstrike'"

[packs.incident-response.queries.recent_detections]
query = "SELECT * FROM cs_detections WHERE created_time > datetime('now', '-24 hours')"
interval = 300
clients = "all"

[packs.incident-response.queries.quarantined_hosts]
query = "SELECT * FROM s1_agents WHERE network_status = 'quarantined'"
interval = 600
clients = "all"
```

- **Discovery queries** in Prism would check client connectivity or sensor availability before running a pack
- **Shard equivalent:** target a percentage of clients (useful for testing new packs on a subset)
- **Platform filter equivalent:** sensor type filter (CrowdStrike-only queries, S1-only queries, etc.)
- Packs map perfectly to Prism's planned **alias system** -- an alias is just a single-query pack

### Priority: **P1 (should-have)**

Packs are the natural extension of aliases. They bundle related queries for workflows like "incident response", "compliance check", "threat hunt". MSSPs will want to define and share standard packs across analyst teams.

---

## 3. Decorators

### How osquery implements it

**Key files:** `plugins/config/parsers/decorators.h`, `plugins/config/parsers/decorators.cpp`

Decorators are **SQL queries whose result columns are injected as metadata into every log line**. They come in three flavors:

1. **`load`** -- Run once when config is loaded. Results persist until config changes. Example: `SELECT hostname FROM system_info` -- adds `hostname` to every log.
2. **`always`** -- Run before every scheduled query execution. Example: `SELECT user FROM logged_in_users LIMIT 1` -- adds current user context.
3. **`interval`** -- Run on a time interval (must be a multiple of 60s). Example: every 300s, refresh the IP address.

**Data flow:**
1. `DecoratorsConfigParserPlugin` parses the `decorators` config key into `load_`, `always_`, and `intervals_` maps (source -> list of SQL queries)
2. The scheduler calls `runDecorators(DECORATE_ALWAYS)` before each query in `launchQuery()`
3. The scheduler calls `maybeRunDecorators()` every 60 seconds for interval decorators
4. Each decorator query executes, and the first row's columns are stored in `kDecorations` (a static `DecorationStore` -- `map<source, map<column_name, value>>`)
5. `getDecorations()` copies all decorations into the `QueryLogItem.decorations` map
6. The decorations are serialized alongside query results in every log line

**Thread safety:** Two separate mutexes -- `kDecorationsConfigMutex` for config updates and `kDecorationsMutex` for reading/writing decoration values.

**Important behaviors:**
- Only the first row of a decorator query is used (multi-row = undefined behavior + warning log)
- Duplicate column names across decorators = last-write-wins
- `decorations_top_level` flag controls whether decorations are nested under `"decorations"` key or merged as top-level JSON fields

### How it maps to Prism

This is **critical** for Prism. Every query result needs MSSP context:
- Which client organization does this result belong to?
- Which sensor instance provided it?
- Which analyst ran the query?
- What was the query pack/alias that generated it?

**Prism-tailored version:**
```rust
/// Prism auto-decorates every result row with context
struct PrismDecorations {
    client_id: String,        // "acme-corp"
    client_name: String,      // "Acme Corporation"
    sensor_type: String,      // "crowdstrike"
    sensor_instance: String,  // "us-1"
    analyst_id: String,       // "jsmith"
    query_source: String,     // "pack:incident-response" or "interactive"
    prism_version: String,
}
```

Unlike osquery's SQL-query-based decorators, Prism's decorations are deterministic metadata from the config and session context. No need to run SQL queries to generate them -- they come from the TOML config (`[clients]` section) and the current session.

**Implementation:**
- `load`-equivalent: read from config at startup (client metadata, sensor endpoints)
- `always`-equivalent: inject per-row during result materialization (analyst_id, query_source)
- `interval`-equivalent: periodically refresh client connectivity status

### Priority: **P0 (must-have)**

Every result leaving Prism must be attributed to a client and sensor. Without decorations, MSSP analysts cannot distinguish results from different clients in their logs and exports. This is not optional.

---

## 4. Distributed Queries

### How osquery implements it

**Key files:** `osquery/distributed/distributed.h`, `osquery/distributed/distributed.cpp`, `osquery/dispatcher/distributed_runner.h`, `osquery/dispatcher/distributed_runner.cpp`

Distributed queries are **ad-hoc queries pushed from a central server to endpoints**. The model is pull-based:

1. **`DistributedRunner`** polls every `distributed_interval` (default 60s) seconds
2. It calls `Distributed::pullUpdates()` which invokes the active `DistributedPlugin` (typically TLS) with `getQueries` action
3. The server responds with JSON: `{"queries": {"id1": "SELECT ...", "id2": "SELECT ..."}, "discovery": {"id1": "SELECT 1 WHERE ..."}, "accelerate": 30}`
4. `acceptWork()` runs discovery queries first. If a discovery query returns 0 rows, the corresponding distributed query is skipped. Remaining queries are stored in the RocksDB `kDistributedQueries` domain.
5. `runQueries()` pops each pending query, executes it with performance monitoring, and batches results
6. `flushCompleted()` calls `writeResults` on the `DistributedPlugin` to POST results back

**Anti-abuse protections:**
- **Denylisting:** `checkAndSetAsRunning()` hashes each query (SHA-256) and stores a timestamp. If the same query hash appears within `distributed_denylist_duration` (default 86400s / 1 day), it's skipped. This prevents malicious or accidental repeated heavy queries.
- **Acceleration:** The server can send an `"accelerate": N` field to temporarily reduce the polling interval to 5 seconds for N seconds, enabling near-real-time query response.
- **Performance stats:** Each distributed query result includes `wall_time_ms`, `user_time`, `system_time`, and `memory` metrics.

**Result format returned to server:**
```json
{
  "queries": {"id1": [{"col1": "val1"}]},
  "statuses": {"id1": 0},
  "messages": {"id1": ""},
  "stats": {"id1": {"wall_time_ms": 42, "memory": 1024}}
}
```

### How it maps to Prism

In Prism's MSSP context, this maps to a **central management plane pushing queries to analyst Prism instances**. But the direction is slightly different -- Prism is already a centralized query engine, not a distributed agent. The distributed pattern is more relevant for:

1. **Multi-analyst coordination:** A SOC lead pushes a query to all analyst Prism instances ("everyone run this IOC sweep across your assigned clients")
2. **Automated response:** An orchestration layer (SOAR) pushes queries to Prism based on alert triggers
3. **Scheduled query results distribution:** Results from scheduled packs are pushed/pulled by downstream consumers

**Prism-tailored version:**
- The "server" is a Prism management API or a SOAR integration
- The "endpoint" is an individual Prism instance (per-analyst or per-SOC)
- Pull-based: Prism instances poll for new ad-hoc queries from a shared queue (Redis, PostgreSQL, or a simple REST endpoint)
- Acceleration: when an incident fires, reduce poll interval to near-real-time
- Denylisting: rate-limit expensive cross-client queries

### Priority: **P2 (nice-to-have)**

For initial release, Prism is a single-analyst tool. Distributed query coordination becomes relevant when Prism serves a multi-analyst SOC team, which is a later scaling concern.

---

## 5. Logging Architecture

### How osquery implements it

**Key files:** `osquery/core/plugins/logger.h`, `osquery/logger/data_logger.h`, `plugins/logger/buffered.h`, `plugins/logger/filesystem_logger.h`, `plugins/logger/tls_logger.h`

osquery has a **two-tier logging architecture:**

**Tier 1: Status logs** (operational telemetry)
- Glog-based severity levels: INFO, WARNING, ERROR, FATAL
- `StatusLogLine` struct: severity, filename, line, message, calendar_time, time, identifier
- Buffered in `BufferedLogSink` until the logger plugin initializes
- Can be forwarded to the logger plugin or kept in Glog's file-based logs

**Tier 2: Result logs** (query output)
- `QueryLogItem` struct: name, identifier, time, epoch, counter, calendar_time, decorations, DiffResults or snapshot_results
- Routed through `logQueryLogItem()` for differential results or `logSnapshotQuery()` for snapshots
- `logString()` is the generic raw-string logging API

**LoggerPlugin interface:**
- `logString(s)` -- primary log receiver (REQUIRED)
- `logStatus(log)` -- opt-in Glog status forwarding
- `logSnapshot(s)` -- opt-in separate snapshot handling (defaults to `logString`)
- `logEvent(s)` -- opt-in direct event forwarding
- `logStringBatch(event_batch)` -- opt-in batch event forwarding

**BufferedLogForwarder** (used by TLS, Kafka, AWS Kinesis, AWS Firehose):
- Writes logs to RocksDB-backed buffer (keyed by timestamp-based index)
- A forwarder thread wakes every `log_period_` (default: configurable per plugin)
- Scans up to `max_log_lines_` entries, sorts into status vs result types
- Calls `send()` (implemented by subclass) for each batch
- On success: deletes from buffer. On failure: exponential backoff
- `purge()` enforces `buffered_log_max` to prevent unbounded growth

**Available backends:** filesystem, TLS, syslog, stdout, AWS Kinesis, AWS Firehose, Kafka, Windows Event Log

### How it maps to Prism

Prism needs both tiers:

**Status/audit logs:** Every action Prism takes must be auditable for MSSP compliance:
- Which analyst queried which client's data at what time
- API calls made to each sensor (rate limits, errors, retries)
- Config changes, pack activations, credential access

**Result logs:** Query results need structured output for:
- SIEM ingestion (JSON structured logs to Splunk/Elastic)
- Ticketing integration (results to Jira/ServiceNow)
- Export to client-specific storage

**Prism-tailored version:**
```rust
trait PrismLogger: Send + Sync {
    /// Log query results (differential or snapshot)
    fn log_results(&self, item: &QueryResultItem) -> Result<()>;
    /// Log audit events (who did what)  
    fn log_audit(&self, event: &AuditEvent) -> Result<()>;
    /// Log operational status
    fn log_status(&self, entry: &StatusEntry) -> Result<()>;
}
```

The `BufferedLogForwarder` pattern is directly applicable: buffer logs in SQLite when the downstream sink is unavailable, flush in batches with exponential backoff.

### Priority: **P1 (should-have)**

Basic stdout/file logging is P0 (just print results). Structured audit logging with pluggable backends (SIEM, file, database) is P1. The buffered forwarder pattern is P1 for reliability.

---

## 6. File Carving

### How osquery implements it

**Key files:** `osquery/carver/carver.h`, `osquery/carver/carver_utils.h`

File carving is a **deferred file extraction system:**

1. A distributed query or table invocation calls `carvePaths()` with a set of file paths
2. The request is stored in RocksDB under `kCarves` domain with status "SCHEDULED" and a GUID
3. The scheduler periodically calls `scheduleCarves()` which dispatches a `CarverRunner` if `kCarverPendingCarves` is true
4. `CarverRunner::start()` scans all carve requests and processes them serially
5. For each carve: copies files to a temp directory via `blockwiseCopy()` (default 8KB blocks), creates a tar archive, compresses with zstd, then POSTs to the carve endpoint in chunks
6. The `postCarve()` method uses a start/continue protocol with session IDs for resumable uploads

**Key design decisions:**
- Deferred execution prevents carves from blocking queries
- Serial execution prevents resource exhaustion from parallel carves
- Block-wise copy allows monitoring and cancellation
- GUID-based tracking ties carve results to the originating distributed query via `requestId`
- Atomic pending flag (`kCarverPendingCarves`) avoids polling overhead when no carves are pending

### How it maps to Prism

Prism's equivalent: **extracting large binary data from sensor APIs** (e.g., downloading a suspicious file from CrowdStrike RTR, pulling a memory dump from SentinelOne, fetching a PCAP from network sensors).

**Prism-tailored version:**
- Not file system carving, but **API artifact extraction**
- Same deferred pattern: queue the download, process asynchronously, stream to storage
- Block-wise transfer for large artifacts (memory dumps can be GBs)
- GUID tracking to correlate artifact with the query/alert that triggered it
- Storage to object storage (S3/GCS) rather than TLS POST

### Priority: **P2 (nice-to-have)**

Artifact extraction from sensor APIs is valuable but secondary to query capabilities. The initial release should focus on querying data, not downloading artifacts. The deferred-execution pattern is worth borrowing for any async operation.

---

## 7. Configuration Refresh

### How osquery implements it

**Key files:** `osquery/config/config.h`, `osquery/config/config.cpp` (lines 300-340, 1326-1341)

**ConfigRefreshRunner** is a simple thread:
```cpp
void ConfigRefreshRunner::start() {
    while (!interrupted()) {
        pause(std::chrono::seconds(refresh_sec_));
        if (interrupted()) return;
        Config::get().refresh();
    }
}
```

**Configuration lifecycle:**
1. **Initial load:** `Config::load()` calls `refresh()` which invokes the active `ConfigPlugin::genConfig()` to retrieve config (from filesystem, TLS, etc.)
2. **Periodic refresh:** If `config_refresh > 0` (seconds), a `ConfigRefreshRunner` thread is started. It sleeps for `refresh_sec_` seconds, then calls `Config::get().refresh()`.
3. **Accelerated refresh:** On failure, `config_accelerated_refresh` (default 300s) provides a faster retry interval.
4. **Backup/restore:** `config_enable_backup` flag enables config persistence to RocksDB. If a refresh fails, the last good config is restored via `restoreConfigBackup()`.
5. **Hash-based change detection:** `Config::hashSource()` computes SHA1 of each config source. If the hash hasn't changed, the update is skipped (avoiding unnecessary parser re-runs).
6. **Parser chain:** On update, `applyParsers()` iterates all registered `ConfigParserPlugin`s. Each parser receives the config keys it registered for (e.g., decorators parser gets "decorators", file_paths parser gets "file_paths").
7. **Purge:** Before applying updates, `purge()` removes outdated query results, timestamps, and saved intervals.

**Config sources:** Config can come from multiple sources (multiple files, multiple TLS endpoints). Sources are merged by key -- dictionaries merge by key, arrays concatenate.

### How it maps to Prism

Prism uses TOML config files. The refresh pattern maps to:
1. **File-watch based refresh:** Use `notify` / `inotify` / `kqueue` to detect TOML changes (more responsive than polling)
2. **Hash-based skip:** If TOML SHA256 hasn't changed, skip re-parse
3. **Hot reload of specific sections:** Credential rotation, new client additions, pack changes should apply without restart
4. **Backup/restore:** Keep last-known-good config in memory; roll back if new config fails validation

**Prism-tailored version:**
```rust
struct ConfigManager {
    current: Arc<RwLock<PrismConfig>>,
    hash: AtomicU64,  // xxhash of config file
    watcher: notify::RecommendedWatcher,
}

impl ConfigManager {
    fn on_file_change(&self) {
        let new_content = fs::read_to_string(&self.path)?;
        let new_hash = xxhash(&new_content);
        if new_hash == self.hash.load(Ordering::Relaxed) { return; }
        
        let new_config: PrismConfig = toml::from_str(&new_content)?;
        new_config.validate()?;
        *self.current.write() = new_config;
        self.hash.store(new_hash, Ordering::Relaxed);
        self.notify_subscribers();
    }
}
```

### Priority: **P1 (should-have)**

Initial release can require restart for config changes. Hot reload is important for credential rotation (API keys expiring) and adding new clients without downtime. Signal-based (filesystem watch) is better than interval-based for a CLI tool.

---

## 8. Watchdog

### How osquery implements it

**Key files:** `osquery/core/watcher.h`, `osquery/core/watcher.cpp`

osquery uses a **watcher-worker process model:**

1. The **watcher** process spawns a **worker** process that does all actual query execution
2. The watcher monitors the worker's resource usage at `INTERVAL` (default 3s)
3. If the worker exceeds limits, the watcher kills and respawns it

**WatchdogLimitType levels** (3 levels: normal=0, restrictive=1, permissive=-1):

| Limit | Normal (0) | Restrictive (1) | Permissive (-1) |
|-------|-----------|-----------------|-----------------|
| MEMORY_LIMIT (MB) | 200 | 100 | 10000 |
| UTILIZATION_LIMIT (% CPU) | 10 | 5 | 100 |
| RESPAWN_LIMIT (s) | 4 | 4 | 1000 |
| RESPAWN_DELAY (s) | 5 | 5 | 1 |
| LATENCY_LIMIT (s sustained) | 12 | 6 | 1000 |
| INTERVAL (s between checks) | 3 | 3 | 3 |

**Override flags:** `watchdog_memory_limit`, `watchdog_utilization_limit`, `watchdog_latency_limit` allow per-deployment tuning.

**PerformanceState tracking:**
- `sustained_latency` -- counter of consecutive intervals where CPU exceeded limit
- `user_time`, `system_time` -- last-checked CPU times
- `initial_footprint` -- baseline memory at start
- `last_respawn_time` -- for respawn rate limiting

**WatcherWatcherRunner:** The worker spawns a thread that watches the watcher. If the watcher dies, the worker shuts down. This prevents orphaned worker processes.

**Query denylisting on crash:** If a worker crashes while executing a query, the `kExecutingQuery` key in RocksDB will still be set. On next start, `Schedule` reads this key, logs a warning, and (if watchdog is enabled) denylists the query for 86400s (1 day).

### How it maps to Prism

Prism's equivalent: **resource budgeting for API queries and memory management.**

**Prism-tailored version:**
```rust
struct ResourceWatchdog {
    memory_limit_mb: usize,       // e.g., 512MB for Prism process
    api_rate_limit: RateLimiter,  // per-sensor API rate limits
    query_timeout: Duration,      // max time for a single federated query
    concurrent_limit: usize,      // max concurrent API calls
}

impl ResourceWatchdog {
    /// Check if a query should be killed
    fn check_query_health(&self, query: &RunningQuery) -> QueryAction {
        if query.memory_usage() > self.memory_limit_mb { return QueryAction::Kill; }
        if query.elapsed() > self.query_timeout { return QueryAction::Kill; }
        if query.rows_buffered() > self.max_rows { return QueryAction::Spill; }
        QueryAction::Continue
    }
    
    /// Denylist queries that previously crashed or timed out
    fn denylist_query(&self, query_hash: &str, duration: Duration);
}
```

**Key differences from osquery:**
- No watcher-worker process split needed (Prism is not a daemon, it's a CLI tool)
- Memory limits apply to result buffering (a query across 50 clients could return millions of rows)
- Rate limiting is per-sensor-API, not per-CPU
- Query denylisting prevents re-running expensive queries that timeout

### Priority: **P0 (must-have)**

Memory budgeting and query timeouts are essential for initial release. An MSSP analyst running `SELECT * FROM cs_detections` across 50 clients without limits could OOM the process or exhaust API rate limits. The watchdog's graduated limit levels (normal/restrictive/permissive) are a good UX pattern.

---

## 9. FIM (File Integrity Monitoring)

### How osquery implements it

**Key files:** `osquery/events/linux/inotify.h`, `osquery/events/darwin/fsevents.h`, `osquery/events/eventsubscriberplugin.h`, `plugins/config/parsers/file_paths.cpp`

FIM is implemented as an **event-driven pipeline:**

1. **Configuration:** `file_paths` config key defines categories of paths to monitor:
   ```json
   {"file_paths": {"etc": ["/etc/%%"], "homes": ["/home/%/.ssh/%%"]}}
   ```
   `file_paths_query` allows dynamically computing paths via SQL. `exclude_paths` filters unwanted matches.

2. **EventPublisher:** Platform-specific kernel event listeners:
   - Linux: `INotifyEventPublisher` wraps `inotify(7)` -- watches file descriptors for IN_CREATE, IN_MODIFY, IN_DELETE, IN_MOVED_TO, etc.
   - macOS: `FSEventsEventPublisher` wraps FSEvents API
   - The publisher maintains `descriptor_paths_` mapping watch descriptors to paths

3. **EventSubscriber:** `file_events` table subscribes to the publisher. When a file change fires, the subscriber's callback creates a Row with: `target_path`, `category`, `action`, `transaction_id`, `md5/sha1/sha256` (optional hashing).

4. **Storage:** Events are stored in RocksDB with time-based indexing. `EventSubscriberPlugin::addBatch()` writes event rows. `generateRows()` retrieves events within a time window.

5. **Expiration:** Events are purged based on `events_expiry` (default 86400s) and `events_max` (maximum batches). The `expireEventBatches()` and `removeOverflowingEventBatches()` methods handle cleanup.

6. **Query-time optimization:** `shouldOptimize()` allows event subscribers to track which queries have consumed events, enabling `events_optimize` to skip re-reading already-delivered events.

### How it maps to Prism

FIM's "monitor changes over time" pattern maps to Prism's need for **continuous sensor data monitoring:**

**Prism-tailored version:** "Monitor these data sources for changes"
```rust
/// Watch for changes in sensor data over time
struct SensorMonitor {
    /// Data source to monitor (e.g., "cs_detections", "s1_threats")
    source: String,
    /// Filter predicate (e.g., "severity >= 'critical'")
    filter: Option<String>,
    /// How often to poll the API
    poll_interval: Duration,
    /// Client scope
    clients: Vec<ClientId>,
}
```

The pattern is:
1. Periodically query a sensor API (Prism's equivalent of inotify watch)
2. Diff against previous results (Prism already has differential engine from scheduled queries)
3. Store deltas with timestamps
4. On `SELECT * FROM monitored_changes WHERE source = 'cs_detections'`, return accumulated changes

This is essentially scheduled queries + event storage, which means FIM-style monitoring is an emergent capability from combining features 1 (scheduling) + existing differential engine + a small event ring buffer.

### Priority: **P2 (nice-to-have)**

This is essentially an enhancement to scheduled queries that stores change events in a queryable buffer. The core differential engine (already known from prior analysis) plus scheduling (P1) provides 90% of this capability. The dedicated event store and monitoring UX is P2.

---

## 10. Extensions

### How osquery implements it

**Key files:** `osquery/extensions/extensions.h`, `osquery/extensions/interface.h`, `osquery/extensions/impl_thrift.cpp`

osquery's extension system uses **Thrift IPC over UNIX domain sockets:**

1. **ExtensionManager** runs in the osquery daemon, listening on a UNIX socket
2. **Extensions** are separate processes that connect to the manager socket
3. On startup, an extension calls `registerExtension()` with its metadata and registry routes
4. The manager assigns a `RouteUUID` and records the extension's advertised tables/loggers/config plugins
5. When a query references an extension's table, the registry routes the call through `callExtension()` which makes a Thrift RPC to the extension process
6. Extensions can provide: tables, loggers, config plugins, distributed plugins

**Extension lifecycle:**
- `extensions_autoload` -- file listing extension binaries to load automatically
- `ExtensionWatcher` -- monitors extension health via periodic `ping()`
- If an extension dies, the watcher deregisters its routes and optionally respawns it
- `WatcherRunner` manages extension process lifecycle (spawn, monitor, respawn)

**API surface** (`ExtensionManagerInterface`):
- `registerExtension(info, registry)` -- register routes
- `deregisterExtension(uuid)` -- unregister
- `query(sql)` -- execute SQL in core (extensions can use core's SQLite)
- `getQueryColumns(sql)` -- introspect column types
- `extensions()` -- list active extensions
- `options()` -- read core's flag values

**SDK:** Extensions include `osquery/sdk.h` which provides macros for table registration, a main loop, and the Thrift client infrastructure.

### How it maps to Prism

This is the **user-written sensor adapter** pattern. MSSPs may need custom sensors for:
- Internal tools with proprietary APIs
- Niche security products not covered by built-in adapters
- Custom data enrichment (threat intel feeds, asset databases)

**Prism-tailored version:**

Instead of Thrift IPC and separate processes, Prism could use:

**Option A: gRPC-based out-of-process extensions**
```protobuf
service PrismExtension {
    rpc RegisterAdapter(AdapterInfo) returns (RegistrationResult);
    rpc Query(QueryRequest) returns (stream Row);
    rpc GetSchema() returns (TableSchema);
}
```

**Option B: WASM-based in-process extensions** (more sandboxed)
```rust
/// Extension trait that WASM modules implement
trait SensorAdapter {
    fn schema(&self) -> Vec<ColumnDef>;
    fn query(&self, context: &QueryContext) -> Vec<Row>;
}
```

**Option C: Lua/Rhai scripting** (simplest, least powerful)
```lua
-- custom_adapter.lua
function schema()
    return {
        {name="id", type="TEXT"},
        {name="status", type="TEXT"},
    }
end

function query(context)
    local resp = http.get("https://internal-api/status")
    return json.decode(resp.body)
end
```

### Priority: **P2 (nice-to-have)**

For initial release, built-in adapters for CrowdStrike, SentinelOne, and Defender cover the most common MSSP sensors. An extension API becomes valuable once Prism has traction and users want to integrate proprietary tools. The gRPC approach is most aligned with osquery's proven pattern.

---

## Summary: Priority Matrix

| Feature | Priority | Rationale |
|---------|----------|-----------|
| **Decorators** | P0 | Every result must have client/sensor/analyst context |
| **Watchdog** | P0 | Memory/rate limits prevent OOM and API exhaustion |
| **Scheduled Queries** | P1 | Continuous monitoring is core MSSP value prop |
| **Packs** | P1 | Natural extension of aliases for workflow bundles |
| **Logging Architecture** | P1 | Audit trail and SIEM integration required |
| **Config Refresh** | P1 | Hot reload for credential rotation |
| **Distributed Queries** | P2 | Multi-analyst coordination is later scaling |
| **FIM** | P2 | Emergent from scheduling + differential engine |
| **Extensions** | P2 | Custom sensor adapters needed after initial traction |
| **File Carving** | P2 | Artifact extraction is secondary to querying |

## Key Architectural Patterns to Adopt

1. **Splay/jitter for API calls** -- osquery's `splayValue()` with persistent splay caching is directly applicable to spreading API calls across clients
2. **Deferred execution** -- carver's pattern of queue-then-process prevents blocking; apply to any async operation
3. **BufferedLogForwarder with exponential backoff** -- essential for reliable log delivery when SIEM/storage is temporarily unavailable
4. **Discovery queries for conditional execution** -- packs only run if discovery succeeds; Prism should skip sensor queries if the sensor API is unreachable
5. **Query denylisting on crash/timeout** -- automatically protect against repeated expensive queries
6. **Hash-based change detection** -- skip config re-parse if TOML hash unchanged; skip diff computation if result hash unchanged
7. **Graduated limit levels** -- osquery's normal/restrictive/permissive watchdog levels are excellent UX for letting users tune resource constraints
