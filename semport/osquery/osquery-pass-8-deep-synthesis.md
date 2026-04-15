# Codebase Deep Synthesis: osquery (Focused Architectural Analysis)

**Analysis scope:** Virtual table architecture, query engine, schema system, plugin system, caching, events, and performance patterns. Individual OS table implementations, daemon management, deployment, and RPC layers are excluded.

**Analysis date:** 2026-04-13

---

## Executive Summary

osquery is a SQL-powered operating system instrumentation framework that treats the OS as a relational database. Its core architectural innovation is the **virtual table abstraction**: every data source (processes, network connections, file hashes, etc.) is exposed as a SQLite virtual table backed by a C++ `TablePlugin`. When a user writes `SELECT * FROM processes WHERE pid = 1`, SQLite's query planner calls into osquery's virtual table module, which extracts WHERE clause constraints and passes them to the table plugin's `generate()` method. The plugin performs the actual OS API calls, optionally using the constraints to optimize (e.g., only querying a single PID). Results flow back as rows through SQLite's virtual table cursor protocol.

The architecture is cleanly layered: a **code-generation pipeline** converts `.table` spec files into C++ `TablePlugin` subclasses at build time, a **registry system** manages plugin discovery and lifecycle, and an **event framework** handles asynchronous OS events (like file changes or process creation) by persisting them to a backing store for time-windowed SQL queries.

---

## 1. Virtual Table Architecture

### 1.1 The Core Abstraction: TablePlugin

The `TablePlugin` class (`osquery/core/tables.h:669-916`) is the fundamental interface every data source must implement:

```cpp
class TablePlugin : public Plugin {
  virtual TableColumns columns() const;             // Schema declaration
  virtual TableAttributes attributes() const;       // Metadata flags
  virtual TableRows generate(QueryContext& context); // Main data generation
  virtual void generator(RowYield& yield, QueryContext& context); // Streaming alternative
  virtual bool usesGenerator() const;               // Toggle streaming mode
};
```

**Two generation modes exist:**

1. **Batch mode** (`generate`): Returns all rows at once as a `TableRows` vector. Simpler, supports caching, but holds all results in memory.

2. **Generator/coroutine mode** (`generator`): Uses Boost.Coroutine2 to yield rows one at a time. ~5% more CPU overhead for small tables (<100 rows) but dramatically better memory efficiency for large tables (>1000 rows). Explicitly trades cachability for memory efficiency -- generator results cannot be cached because the full result set is never materialized.

### 1.2 The SQLite Virtual Table Module

osquery implements SQLite's virtual table API (`sqlite3_module`) in `osquery/sql/virtual_table.cpp`. Each table gets a module with these critical callbacks:

| Callback | Purpose | Key behavior |
|----------|---------|-------------|
| `xCreate` | Table initialization | Calls `Registry::call("table", name, {{"action", "columns"}})` to get schema, then `sqlite3_declare_vtab` |
| `xBestIndex` | Query planning / cost estimation | Examines constraints, assigns costs based on `ColumnOptions`, tracks used columns |
| `xFilter` | Execute query with constraints | Builds `QueryContext`, calls `TablePlugin::generate()` or `TablePlugin::generator()` |
| `xColumn` | Retrieve column value | Reads from cursor's current row |
| `xNext` / `xEof` | Row iteration | Standard cursor advancement |
| `xUpdate` | INSERT/UPDATE/DELETE | Only for extension tables, serializes to JSON and forwards via Registry |

### 1.3 Constraint Push-Down (The Key Innovation for Prism)

The constraint push-down mechanism is the most architecturally significant pattern in osquery. Here is the complete flow:

**Phase 1: xBestIndex (planning time)**

SQLite calls `xBestIndex` potentially multiple times with different constraint combinations. For each call:

1. Iterate `pIdxInfo->aConstraint[]` -- each entry has a column index, operator, and usability flag.
2. Look up the column's `ColumnOptions` (INDEX, REQUIRED, ADDITIONAL, OPTIMIZED).
3. **Only constraints on INDEX/REQUIRED/ADDITIONAL columns are accepted.** Default columns are left for SQLite to post-filter. This is a critical design decision -- it prevents table plugins from receiving irrelevant constraints.
4. Each accepted constraint gets an `argvIndex` assignment, connecting it to `xFilter`'s argv.
5. Cost is set to 1 for indexed queries, `kMaxIndexCost` (1,000,000) for unindexed.
6. The constraint set (column name + operator pairs) is stored in `VirtualTableContent::constraints[idxNum]`.
7. Used columns are tracked via `colUsed` bitmask for column-pruning optimization.

**Phase 2: xFilter (execution time)**

1. Retrieve the stored constraint set for this `idxNum`.
2. Populate constraint expressions from SQLite's now-resolved `argv[]` values.
3. Handle `IN` operator specially -- iterate all values via `sqlite3_vtab_in_first/next`.
4. Build a `QueryContext` with the populated `ConstraintMap`.
5. Validate REQUIRED column satisfaction -- return `SQLITE_CONSTRAINT` if missing.
6. Call the table plugin's `generate(context)` or `generator(yield, context)`.

**Phase 3: Table Plugin (data generation)**

The table plugin receives a `QueryContext` and can:
- `context.hasConstraint("pid", EQUALS)` -- check if a constraint exists
- `context.constraints["pid"].getAll(EQUALS)` -- get all equality values
- `context.iteritems("path", EQUALS, callback)` -- iterate constraint expressions
- `context.isColumnUsed("cmdline")` -- skip expensive columns not in SELECT
- `context.expandConstraints("path", EQUALS, output, globber)` -- expand filesystem globs

### 1.4 Column Options Taxonomy

Column options (`osquery/core/sql/column.h`) control both the query planner behavior and table plugin contracts:

| Option | Planner effect | Plugin contract |
|--------|---------------|-----------------|
| `DEFAULT` | Not passed to plugin, SQLite post-filters | None -- generate all rows |
| `INDEX` | Cost = 1, constraint passed to plugin | Plugin SHOULD use this for efficient lookup |
| `REQUIRED` | Query rejected if not constrained | Plugin MUST have this to generate results |
| `ADDITIONAL` | Cost = 1, constraint passed to plugin | Plugin generates different/additional data when constrained |
| `OPTIMIZED` | Enables IN-operator batch processing | Plugin handles batch equality checks |
| `HIDDEN` | Column hidden from `SELECT *` | Column exists but not shown by default |

### 1.5 Table Registration Flow

1. **Build time:** Python codegen (`tools/codegen/gentable.py`) reads `.table` spec files and generates C++ using `templates/default.cpp.in`. The generated code creates a `TablePlugin` subclass with hard-coded `columns()`, `attributes()`, and a `generate()` or `generator()` method that delegates to the implementation function.

2. **Static initialization:** The `REGISTER` macro (from `registry_factory.h:278`) creates a static `registries::PI<T>` instance that calls `AutoRegisterInterface::autoloadPlugin()`, inserting the plugin into a global list.

3. **Runtime registration:** `registryAndPluginInit()` iterates all auto-registered plugins and adds them to the `RegistryFactory`.

4. **Table attachment:** When an `SQLiteDBInstance` is created, `attachVirtualTables()` iterates all registered table plugins and calls `sqlite3_create_module()` + `CREATE VIRTUAL TABLE` for each, making them queryable.

---

## 2. Query Engine

### 2.1 SQLite as the Core

osquery does NOT implement its own query parser or planner. It embeds SQLite as an in-memory database with virtual tables attached. The query flow:

```
User SQL --> SQLiteDBManager::get() --> sqlite3 in-memory DB
         --> SQLite parser/planner --> xBestIndex (cost estimation)
         --> SQLite executor --> xFilter (table plugin generate)
         --> SQLite post-filter/join/aggregate --> results
```

### 2.2 Database Instance Management

`SQLiteDBManager` (`sqlite_util.h:204-295`) manages SQLite database connections:

- **Primary instance:** A singleton `sqlite3*` protected by a mutex. Most queries use this.
- **Transient instances:** When the primary is contended, a new ephemeral `sqlite3*` is created with all virtual tables re-attached. This allows concurrent queries without serialization.
- **Soft heap limit:** `SQLITE_SOFT_HEAP_LIMIT` is set to 5MB, constraining SQLite's memory usage.
- **Memory DB settings:** Journal mode OFF, synchronous OFF, no cache -- optimized for ephemeral in-memory operation.

### 2.3 SQL Authorizer (Security)

`sqliteAuthorizer()` implements a strict allowlist of SQLite operations (`kAllowedSQLiteActionCodes`). Notable:
- `SQLITE_ATTACH` is **explicitly forbidden** (prevents arbitrary file writes).
- `SQLITE_READ`, `SQLITE_SELECT`, `SQLITE_FUNCTION` are allowed.
- `SQLITE_INSERT/UPDATE/DELETE` are allowed (for extension writable tables).
- Only specific pragmas are allowed (`kAllowedSQLitePragmas`).

### 2.4 Query Planner

`QueryPlanner` (`sqlite_util.h:307-363`) is a lightweight wrapper around SQLite's `EXPLAIN` output:
- Uses `EXPLAIN` to get the execution program (opcodes).
- Uses `EXPLAIN QUERY PLAN` to determine table scan order.
- Maps SQLite opcodes to result types (`kSQLOpcodes`) for column type inference.
- Primary use: `applyTypes()` fills in `UNKNOWN` column types from expression analysis.

### 2.5 Custom SQL Functions

osquery extends SQLite with domain-specific functions registered at DB initialization:
- **Version comparison:** Custom collation sequences for package version comparison (DPKG, RPM, etc.)
- **Hashing:** `md5()`, `sha1()`, `sha256()` functions
- **String functions:** `split()`, `regex_match()`, etc.
- **Math functions:** Extended math operations
- **Encoding:** `to_base64()`, `from_base64()`, etc.
- **Filesystem:** Path manipulation functions
- **Network:** IP/CIDR functions

---

## 3. Schema System

### 3.1 Table Spec Files

Table schemas are defined in Python-syntax `.table` files under `specs/`:

```python
table_name("processes")
description("All running processes on the host system.")
schema([
    Column("pid", BIGINT, "Process ID", index=True, optimized=True),
    Column("name", TEXT, "The process path or shorthand argv[0]"),
    Column("path", TEXT, "Path to executed binary"),
    # ...
])
extended_schema(WINDOWS, [
    Column("elevated_token", INTEGER, "Process uses elevated token"),
])
attributes(cacheable=True, strongly_typed_rows=True)
implementation("system/processes@genProcesses")
```

### 3.2 Column Types

Defined in `column.h`, the type system is intentionally simple:

| Type | SQLite affinity | C++ representation |
|------|----------------|-------------------|
| `TEXT_TYPE` | TEXT | `std::string` |
| `INTEGER_TYPE` | INTEGER | `int` |
| `BIGINT_TYPE` | BIGINT | `long long int` |
| `UNSIGNED_BIGINT_TYPE` | UNSIGNED BIGINT | `unsigned long long int` |
| `DOUBLE_TYPE` | DOUBLE | `double` |
| `BLOB_TYPE` | BLOB | `std::string` (binary) |

**Critical design choice:** All data flows through osquery as strings (`std::string`). The `SQL_TEXT()`, `INTEGER()`, `BIGINT()`, `DOUBLE()` macros are all just `std::to_string()` wrappers. Type affinity only matters for constraint matching (comparison operators) and the `ConstraintList::matches()` method, which casts based on affinity.

### 3.3 Platform-Specific Schema Extensions

`extended_schema(PLATFORM, [...])` allows platform-specific columns. The codegen emits preprocessor-guarded column additions. This means the schema for a table can differ across platforms while sharing a common core.

### 3.4 Row Representation

Two row types exist:

1. **DynamicTableRow** (`dynamic_table_row.h`): A `std::map<std::string, std::string>` wrapper. Default for most tables. Flexible but has map overhead.

2. **Strongly-typed rows** (generated via `typed_row.h.in`): Tables with `strongly_typed_rows=True` get a generated struct with fixed fields. Better cache locality and avoids map lookups.

---

## 4. Plugin / Registry Architecture

### 4.1 Registry Hierarchy

```
RegistryFactory (singleton)
  |-- "table" registry: RegistryType<TablePlugin>
  |     |-- "processes": ProcessesTablePlugin
  |     |-- "users": UsersTablePlugin
  |     |-- ...
  |-- "sql" registry: RegistryType<SQLPlugin>
  |     |-- "sql": SQLiteSQLPlugin
  |-- "event_publisher" registry: RegistryType<EventPublisherPlugin>
  |-- "event_subscriber" registry: RegistryType<EventSubscriberPlugin>
  |-- "config" registry: RegistryType<ConfigPlugin>
  |-- "logger" registry: RegistryType<LoggerPlugin>
  |-- ...
```

### 4.2 Plugin Communication Protocol

All plugin interaction goes through `PluginRequest` / `PluginResponse` (both `map<string, string>` / `vector<map<string, string>>`). This serializable format enables:

- **Local plugins:** Direct method calls through the registry.
- **Extension plugins:** Thrift RPC to out-of-process extensions.

The `TablePlugin::call()` method routes on an "action" key:
- `"generate"` -> calls `generate(context)`
- `"columns"` -> returns schema
- `"delete"` / `"insert"` / `"update"` -> for writable tables
- The `QueryContext` is serialized to JSON for extension transport.

### 4.3 Extension Support

External processes can register table plugins via Thrift RPC. The `addExternal()` / `removeExternal()` methods on `TablePlugin` handle dynamic attachment/detachment of virtual tables. Extension tables are automatically given `INDEX` on all columns (via `FLAGS_extensions_default_index`).

### 4.4 Table Enable/Disable

`SQLiteDBManager` maintains `disabled_tables_` and `enabled_tables_` sets, populated from CLI flags `--disable_tables` and `--enable_tables`. Disabled tables are simply not attached to the SQLite database.

---

## 5. Caching

### 5.1 Schedule-Level Caching

Tables with `cacheable=True` attribute support result caching between scheduled query runs:

```cpp
// In generated code (default.cpp.in):
TableRows generate(QueryContext& context) override {
    if (isCached(kCacheStep, context)) {
        return getCache();  // Deserialize from RocksDB
    }
    TableRows results = tables::genProcesses(context);
    setCache(kCacheStep, kCacheInterval, context, results);
    return results;
}
```

**Cache invalidation rules:**
- Cache freshness is based on the schedule step counter and interval.
- `isCached()` checks: `step < last_cached + last_interval`.
- Cache is NOT used if any INDEX, REQUIRED, ADDITIONAL, or OPTIMIZED column has constraints (meaning the query is filtered and the cached full-scan results don't apply).
- Cache is NOT used if column pruning is active (non-default columns selected).
- Cache is serialized as JSON to RocksDB (`kQueries` domain, key: `cache.<table_name>`).
- Caching is disabled globally via `--disable_caching` flag.

### 5.2 In-Query Caching (VirtualTableContent::cache)

The `VirtualTableContent` struct contains a per-query cache (`std::map<std::string, TableRowHolder>`). This is used within a single query execution to cache intermediate results between multiple `xFilter` calls on the same table (e.g., in JOINs where the same table is scanned multiple times with different constraints).

- Cache is keyed by an arbitrary string index (table-implementation defined).
- Expired after each query run via `clearAffectedTables()`.
- Accessible through `QueryContext::isCached()`, `getCache()`, `setCache()`.

### 5.3 Warm Query Cache

The `SQLiteDBInstance` tracks a `use_cache_` flag that can be set per-query. When enabled, tables check their `VirtualTableContent` cache before regenerating. This is a cooperative mechanism -- the query caller opts in, and the table plugin checks.

---

## 6. Event System

### 6.1 Architecture: Publisher/Subscriber

The event system is a pub/sub framework for asynchronous OS events:

```
EventPublisher (run loop thread)
  |-- Watches OS events (inotify, audit, etc.)
  |-- Fires EventContext to matching Subscriptions
  |
EventSubscriber (table plugin)
  |-- Subscribes to publisher with SubscriptionContext
  |-- Receives EventCallback with matching events
  |-- Stores events in RocksDB (time-indexed)
  |-- genTable() retrieves events by time window for SQL queries
```

### 6.2 EventPublisher

- Template class `EventPublisher<SC, EC>` parameterized on SubscriptionContext and EventContext types.
- Lifecycle: `setUp()` -> `configure()` -> `run()` (loop) -> `tearDown()`.
- Each publisher runs in its own thread, created by `EventFactory::delay()`.
- `run()` is called in a loop until the publisher returns failure or `isEnding()` is set.
- 200ms default pause between run loop iterations.
- `fire(EventContext)` iterates subscriptions, calls `shouldFire(SC, EC)` for filtering, then invokes the subscriber's callback.

### 6.3 EventSubscriber

- Template class `EventSubscriber<PUB>` connects a subscriber to a specific publisher type.
- `init()` sets up subscriptions via `subscribe(callback, subscriptionContext)`.
- Events are stored in RocksDB via `addBatch(vector<Row>)`.
- `genTable(RowYield, QueryContext)` is the SQL table entry point -- retrieves events from the backing store within a time window.
- Events have expiration (`getEventsExpiry()`) and max batch count (`getEventBatchesMax()`).
- Events are always `EVENT_BASED` tables (attribute flag), which affects how the scheduler treats differential results.

### 6.4 Event vs. Point-in-Time Tables

| Aspect | Point-in-time tables | Event-based tables |
|--------|---------------------|-------------------|
| Data source | Direct OS API call | Persistent event store (RocksDB) |
| Generation | `generate()` or `generator()` | `genTable()` reading from DB |
| Freshness | Always current | Historical, time-windowed |
| Caching | Schedule-level cacheable | Not cacheable (append-only) |
| Scheduler | Set difference computed | Always appending, no diff |
| Attribute | Various | `EVENT_BASED` |
| Example | `processes`, `users` | `process_events`, `file_events` |

### 6.5 Configuration Integration

`EventFactory::configUpdate()` scans the query schedule to find which event subscribers are needed, computes expiration intervals (3x the max query interval, rounded to 60s), and configures subscribers accordingly. This means event data retention is automatically tied to query frequency.

---

## 7. Performance Patterns

### 7.1 Memory Management

- **SQLite soft heap limit:** 5MB (`SQLITE_SOFT_HEAP_LIMIT`). Constrains SQLite's internal memory.
- **Generator mode:** Coroutine-based row yielding avoids materializing full result sets. Recommended for tables with >1000 rows.
- **Column pruning:** `QueryContext::isColumnUsed()` / `colsUsedBitset` allows table plugins to skip expensive column computations. The bitmask uses SQLite's `colUsed` 64-bit field (first 63 columns tracked individually, bit 64 is a catch-all).
- **Table delay:** `--table_delay` flag adds artificial delay between table scans (useful for reducing CPU pressure during scheduled queries).

### 7.2 Concurrency Model

- **Primary + transient database pattern:** One primary `sqlite3*` instance shared across queries. When contended, transient instances are created. Each transient instance re-attaches all virtual tables.
- **Attach mutex:** Table attachment is serialized via `kAttachMutex` (recursive mutex).
- **Thread-per-publisher:** Each event publisher gets its own thread.
- **No query parallelism:** Individual queries run single-threaded through SQLite. Cross-query parallelism comes from transient DB instances.

### 7.3 Query Cost Estimation

The `xBestIndex` cost model is binary:
- **Cost = 1:** At least one INDEX, REQUIRED, or ADDITIONAL constraint is present and usable.
- **Cost = kMaxIndexCost (1,000,000):** No useful constraints, or a REQUIRED column is used in the query but not constrained.

This is deliberately simple -- osquery does not estimate row counts or perform sophisticated cost modeling. The key insight: for live system data, the cost model is really about "can the table plugin avoid a full scan?" rather than "how many rows will this return?"

### 7.4 Large Result Set Handling

- **No built-in pagination.** Tables return all matching rows. The generator mode (`RowYield`) provides streaming to mitigate memory, but there is no LIMIT push-down to table plugins.
- **No query timeouts** at the virtual table level. SQLite's `sqlite3_interrupt()` can be used externally.
- **Row size tracking:** `SQLInternal::getSize()` counts total bytes in results for performance monitoring.

---

## 8. Configuration

### 8.1 Table Availability

- `--disable_tables`: Comma-delimited denylist. Disabled tables are not attached.
- `--enable_tables`: Comma-delimited allowlist. If set, ONLY these tables are attached.
- `--disable_events`: Disables the entire event pub/sub system.
- `--extensions_default_index`: When true (default), all extension table columns get INDEX option.

### 8.2 Events Configuration

The config JSON supports:
```json
{
  "events": {
    "enable_subscribers": ["process_events"],
    "disable_subscribers": ["socket_events"]
  }
}
```

### 8.3 Query Packs

The `Config` class manages "packs" -- named groups of scheduled queries. Packs can be defined inline or loaded from external files/URLs via the ConfigPlugin.

---

## 9. Key Architectural Patterns Summary

### Pattern 1: Spec-Driven Code Generation
`.table` spec (Python DSL) --> `gentable.py` --> Generated C++ `TablePlugin` subclass. This eliminates boilerplate and ensures schema consistency between the query engine and plugin implementations.

### Pattern 2: Cooperative Constraint Push-Down
The query planner and table plugin cooperate: the planner identifies which constraints are meaningful (based on column options), and the plugin checks which constraints are present and uses them to optimize data retrieval.

### Pattern 3: Registry as Service Locator
All plugin interaction goes through the `RegistryFactory` singleton with string-typed registry names and plugin names. This enables both in-process and out-of-process (extension) plugins with the same call interface.

### Pattern 4: SQLite as Embedded Query Engine
Rather than building a query engine, osquery embeds SQLite and implements the virtual table API. This gives it full SQL support (JOINs, aggregations, subqueries, CTEs) for free, at the cost of being bound to SQLite's single-threaded execution model and in-memory data model.

### Pattern 5: Dual-Mode Generation (Batch vs. Streaming)
Tables choose between `generate()` (batch, cacheable) and `generator()` (streaming, memory-efficient). The choice is made at the spec level and the framework handles the coroutine machinery transparently.

---

## Lessons for Prism

### P0: Must-Have Architectural Lessons

**P0-1: Constraint Push-Down via QueryContext is the core pattern to adopt.**
osquery's `QueryContext` is the bridge between the query planner and the data source. For Prism, this translates directly: when a user writes `SELECT * FROM crowdstrike_detections WHERE severity > 7 AND created_time > '2024-01-01'`, the query engine must extract those constraints and pass them to the CrowdStrike API adapter so it can use the API's native filtering. The `ColumnOptions` taxonomy (INDEX, REQUIRED, ADDITIONAL, OPTIMIZED) is directly applicable -- map these to which API query parameters are available on each endpoint.

**P0-2: REQUIRED columns prevent meaningless full-scans of remote APIs.**
osquery's `REQUIRED` column option causes the query to fail with `SQLITE_CONSTRAINT` if the column is not constrained in the WHERE clause. For Prism, this is critical for API-backed tables where a full scan is either impossible (API requires parameters) or disastrously expensive (fetching all detections ever from CrowdStrike). Prism must have an equivalent mechanism that rejects queries missing required constraints before making any API calls.

**P0-3: Column pruning saves API bandwidth and processing time.**
osquery tracks which columns are used in the query (`colsUsed` bitmask) and provides `isColumnUsed()` to table plugins. For API-backed tables, this directly maps to API field selection (many security APIs support `fields` or `select` parameters). Prism should propagate column usage from the query plan to the API adapter to minimize response payload size.

**P0-4: The table spec / code-generation pattern is worth adopting.**
osquery's `.table` spec files are the single source of truth for schema, column options, platform support, and documentation. The codegen eliminates boilerplate and ensures consistency. Prism should define a similar spec format (likely in Rust via proc macros or a build.rs script) that generates the adapter trait implementations, schema registrations, and documentation.

### P1: High-Value Patterns

**P1-1: Dual-mode generation (batch vs. streaming) maps to API response patterns.**
Some API endpoints return all results at once (batch); others require pagination (streaming). Prism should offer both modes: batch for small, bounded result sets, and an async streaming mode (using Rust async iterators or channels) for paginated API responses. osquery's coroutine-based generator is the direct analog; Rust's `async Stream` trait is the idiomatic equivalent.

**P1-2: The in-query cache prevents duplicate API calls during JOINs.**
When osquery executes `SELECT * FROM processes p JOIN process_open_sockets s ON p.pid = s.pid`, SQLite may call `xFilter` on `processes` multiple times. The `VirtualTableContent::cache` prevents redundant API calls within a single query. Prism must have this -- without it, a JOIN between two API-backed tables would make O(n) API calls for the inner table.

**P1-3: Cost estimation should reflect API latency, not just row count.**
osquery's binary cost model (1 vs. 1,000,000) is crude but effective for local OS queries. For Prism's remote API tables, cost estimation should factor in API latency, rate limits, and expected result size. A query planner that knows "CrowdStrike detections API takes ~500ms per call" can make better decisions about join order than one that treats all tables equally.

**P1-4: Table enable/disable is essential for multi-tenant deployments.**
osquery's `--disable_tables` and `--enable_tables` flags control which tables are available. Prism should support dynamic table availability based on which API integrations are configured (if no CrowdStrike credentials are provided, crowdstrike_* tables should not be registered).

### P2: Should-Have Patterns

**P2-1: The authorizer pattern provides defense-in-depth.**
osquery's `sqliteAuthorizer` allowlists specific SQL operations. Prism should implement similar guardrails -- e.g., preventing `DELETE` on read-only API tables, limiting recursion depth, and blocking operations that could cause unbounded API calls.

**P2-2: Event-based tables are the model for webhook/streaming API data.**
osquery's event system (publisher -> subscriber -> backing store -> time-windowed query) maps directly to ingesting webhook data or streaming API events. Prism could use this pattern for: CrowdStrike streaming API events, Sentinel One activity feeds, etc. The key insight is that event tables are append-only with time-based expiration, fundamentally different from point-in-time query tables.

**P2-3: Schedule-level caching with constraint-awareness.**
osquery only caches results for unconstrained queries (no INDEX/REQUIRED columns in WHERE clause). This is the right default for Prism: cache `SELECT * FROM crowdstrike_sensors` results for N seconds, but never cache `SELECT * FROM crowdstrike_sensors WHERE hostname = 'foo'` because the constraint changes the result set.

**P2-4: Extension/plugin architecture for third-party integrations.**
osquery's extension system allows out-of-process table plugins via Thrift. For Prism, consider a similar pattern (perhaps via gRPC or WASM) to allow users to add custom API integrations without modifying the core.

### P3: Nice-to-Have / Long-Term Patterns

**P3-1: Custom SQL functions for security domain.**
osquery adds domain-specific SQL functions (hashing, version comparison, CIDR matching). Prism should add security-relevant functions: `ioc_match()`, `subnet_contains()`, `time_window()`, `stix_pattern_match()`, etc.

**P3-2: ATC (Auto Table Construction) pattern for user-defined tables.**
osquery's ATC feature allows users to define tables from SQLite databases at runtime. The equivalent for Prism would be allowing users to define custom API-backed tables via configuration (e.g., "this REST endpoint returns JSON arrays, map these fields to columns").

**P3-3: Query performance tracking.**
osquery's `Config::recordQueryPerformance()` tracks wall time, result size, and resource usage per scheduled query. This is valuable for identifying slow API integrations and optimizing query schedules.

---

## Critical Design Decisions for Prism (Informed by osquery)

### Decision 1: Own Query Engine vs. Embed SQLite/DataFusion

osquery chose to embed SQLite. This gave it free SQL support but locked it into SQLite's limitations (single-threaded execution, no native async, limited type system). **Prism should evaluate DataFusion (Apache Arrow's Rust query engine)** which provides async execution, columnar data, and a more extensible type system -- better suited for remote API queries where async I/O is essential.

### Decision 2: String-Typed vs. Strongly-Typed Row Data

osquery's default `Row = map<string, string>` is flexible but type-unsafe and allocation-heavy. The `strongly_typed_rows` option improves this. **Prism should use Arrow RecordBatch as the native row format** -- columnar, strongly-typed, zero-copy, and compatible with DataFusion.

### Decision 3: Synchronous vs. Asynchronous Table Generation

osquery's `generate()` is synchronous (blocks the calling thread). The coroutine `generator()` is still synchronous -- it just yields. **Prism must be async-first** since every table "generation" involves network I/O to remote APIs. Rust's `async fn` + `Stream` trait is the natural fit.

### Decision 4: Monolithic vs. Federated Constraint Handling

osquery pushes constraints to a single table plugin. For federated queries across multiple APIs, Prism needs to handle constraint distribution: given `SELECT * FROM cs_detections d JOIN s1_threats t ON d.sha256 = t.sha256 WHERE d.severity > 7`, push `severity > 7` to CrowdStrike, then use the returned SHA256 values as constraints for SentinelOne. This is cross-source constraint propagation, which osquery never needs because all its tables query the local OS.

---

## Confidence Assessment

| Area | Confidence | Basis |
|------|------------|-------|
| Virtual table architecture | HIGH | Read all headers and implementation files |
| Constraint push-down mechanism | HIGH | Read complete xBestIndex + xFilter + QueryContext chain |
| Schema system | HIGH | Read specs, codegen, and column type definitions |
| Plugin/registry system | HIGH | Read registry interfaces and factory |
| Caching | HIGH | Read TablePlugin cache methods and generated template |
| Event system | HIGH | Read publisher, subscriber, and factory implementations |
| Performance patterns | MEDIUM | Inferred from code; no profiling data available |
| Configuration | MEDIUM | Read config.h but did not trace all config parsers |

## Gaps

1. **Rate limiting:** osquery has no rate limiting because local OS queries are essentially free. Prism's API-backed tables need explicit rate limit awareness and backpressure.
2. **Authentication:** osquery has no per-table authentication. Prism needs credential management per API integration.
3. **Pagination:** osquery returns all results or streams via coroutine. Prism needs explicit pagination support with cursor management for APIs that page results.
4. **Error recovery:** osquery catches exceptions in `xFilter` and can optionally ignore them. Prism needs more nuanced error handling -- API timeouts, rate limit responses (429), authentication failures, partial results.
5. **Result size limits:** osquery has no LIMIT push-down. Prism should push LIMIT to API calls where supported.

---

## State Checkpoint

```yaml
pass: 8
status: complete
scope: focused-architectural-analysis
files_scanned: 28
directories_analyzed:
  - osquery/sql/
  - osquery/core/
  - osquery/registry/
  - osquery/events/
  - osquery/config/
  - osquery/sdk/
  - specs/
  - tools/codegen/
timestamp: 2026-04-13T00:00:00Z
convergence: single-pass-focused (not multi-round)
```
