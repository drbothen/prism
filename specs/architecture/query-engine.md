---
document_type: architecture-section
level: L3
section: "query-engine"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, domain-spec/axiql-grammar.md, domain-spec/architecture-concept.md]
traces_to: ARCH-INDEX.md
---

# Query Engine

## Architecture Overview

The query engine is Prism's central component. It transforms AxiQL query strings into orchestrated live API calls and DataFusion SQL execution. The engine lives in `prism-query` and owns the full pipeline from parse to result.

```
AxiQL string
  |
  v
[1. Alias Expansion] -- resolve aliases pre-parse (depth 3 max, cycle-checked)
  |
  v
[2. Chumsky Parser] -- zero-copy AST with error recovery, mode auto-detection
  |
  v
[3. Query Planner] -- scope resolution, push-down classification, security limit checks
  |
  v
[4. Sensor Fan-Out] -- parallel (client, sensor) API calls via adapter registry
  |
  v
[5. OCSF Normalization] -- DynamicMessage per record, field mapping, raw_extensions
  |
  v
[6. Arrow Materialization] -- RecordBatch construction, virtual field injection, 10K cap
  |
  v
[7. DataFusion Execution] -- ephemeral SessionContext, MemTable, post-filter, agg, sort, limit
  |
  v
[8. Response Construction] -- query_context echo, sensor_errors, decorator injection
  |
  v
[9. Teardown] -- SessionContext dropped, memory freed
```

## Parser Design (Chumsky 0.12)

### Decision: Chumsky 0.12 for AxiQL Parsing (AD-003)

**Status:** accepted
**Context:** Need a parser for AxiQL's three modes (filter, SQL, pipe) with error recovery and span tracking. Options: Chumsky, nom, pest, winnow.
**Options considered:**
1. Chumsky 0.12 — zero-copy, composable combinators, built-in error recovery, Rich error types
2. nom — mature, fast, but no built-in error recovery; lower-level
3. pest — PEG-based, good for grammars but weaker at error recovery
4. winnow — nom successor, better ergonomics but less ecosystem adoption
**Decision:** Chumsky 0.12 (latest stable).
**Rationale:** Reference: axiathon uses Chumsky 0.10 for AxiQL parsing, proving the pattern works. Chumsky 0.12 adds improved error types and recovery strategies. Zero-copy parsing aligns with our performance goals (parser should add <5ms overhead). Error recovery is critical for AI-generated queries that may have syntax errors.
**Consequences:** AxiQL parser is a pure function: `&str -> Result<Ast, Vec<RichError>>`. No I/O, no state. Fully testable and verifiable.

### Mode Auto-Detection

The parser examines the first token to determine mode:
- Starts with `SELECT` or field comparison → SQL mode or filter mode (disambiguated by SELECT keyword)
- Starts with `FROM` → pipe mode
- Default: filter mode (simple field predicates)

All three modes compile to the same `QueryPlan` struct. DataFusion receives a unified logical plan regardless of surface syntax.

## DataFusion Integration

### Decision: DataFusion as SQL Engine (AD-002)

**Status:** accepted
**Context:** Need a SQL execution engine for filtering, aggregation, sorting, and limiting over Arrow RecordBatches. Options: custom engine, DataFusion, DuckDB (FFI).
**Decision:** DataFusion 53 (latest stable).
**Rationale:** Arrow-native, async execution, UDF extensibility, per-query SessionContext model (ephemeral by design). Reference: axiathon used DataFusion 51 for the same pattern. Monthly release cadence ensures active maintenance.
**Consequences:** Prism's query semantics are bounded by what DataFusion supports. Custom operations are implemented as UDFs.

### SessionContext Lifecycle

```rust
// Per-query: create with memory pool, register, execute, drop
// DataFusion 53 memory pool API — GreedyMemoryPool enforces hard limit, no spill-to-disk
let memory_pool = Arc::new(GreedyMemoryPool::new(per_query_memory_budget));
let runtime_config = RuntimeConfig::new()
    .with_memory_pool(memory_pool);
let runtime_env = Arc::new(RuntimeEnv::new(runtime_config)?);
let session_config = SessionConfig::new()
    .with_target_partitions(1)  // Single-process, no inter-partition overhead
    .set("datafusion.execution.batch_size", &batch_size.to_string());
let ctx = SessionContext::new_with_config_rt(session_config, runtime_env);
ctx.register_table("events", mem_table)?;
// Register UDFs
ctx.register_udf(subnet_contains_udf());
ctx.register_udf(time_window_udf());
ctx.register_udf(ioc_match_udf());
// Execute — DataFusion enforces memory_limit internally for sort/agg/join buffers
let df = ctx.sql(&datafusion_sql).await?;
let batches = df.collect().await?;
// ctx drops here — all memory freed (including DataFusion internal allocations)
```

**DataFusion memory enforcement:** `GreedyMemoryPool` (from `datafusion::execution::memory_pool`) enforces the per-query budget on all intermediate allocations (sort buffers, hash tables for GROUP BY, join probe tables). When the pool limit is reached, DataFusion returns `ResourcesExhausted` error which Prism translates to `E-WATCHDOG-001`. This is critical because the watchdog's RecordBatch byte tracking (DI-027) only measures materialized data, not DataFusion's internal state. With both enforcement layers active, a complex aggregation query cannot silently exceed the memory budget through DataFusion's intermediate allocations.

**DataFusion memory pool API validation (ASM-013):** The exact DataFusion 53 memory pool API must be validated during implementation. If `GreedyMemoryPool` is unavailable or renamed:
1. **Preferred fallback:** Use `datafusion::execution::memory_pool::TrackConsumersPool` wrapping an `UnboundedMemoryPool` with manual size checks after each batch
2. **Minimum viable fallback:** Configure `SessionConfig::with_target_partitions(1)` and set `datafusion.execution.batch_size` to a small value (2048), then rely solely on the RecordBatch byte tracking in DI-027's watchdog for memory enforcement
3. **Unacceptable:** Running without any DataFusion memory enforcement — if no pool API is available, the per-query memory budget from DI-027's RecordBatch tracking must be the sole enforcement, and the two-check grace period must account for DataFusion's internal allocations (which can be 2-5x the RecordBatch size for complex aggregations)

Add a CI smoke test that verifies the DataFusion memory pool integration compiles and limits memory correctly.

### Security UDFs (CAP-027)

| UDF | Signature | Implementation |
|-----|----------|---------------|
| `subnet_contains` | `(cidr: Utf8, ip: Utf8) -> Boolean` | ipnet crate, vectorized over Arrow arrays |
| `time_window` | `(timestamp: Timestamp, duration: Utf8) -> Boolean` | Computes `now - duration <= timestamp` |
| `ioc_match` | `(field: Utf8, ioc_list_name: Utf8) -> Boolean` | Matches field against a named IOC list (see IOC File Specification below) |
| ~~`stix_pattern_match`~~ | ~~`(field: Utf8, pattern: Utf8) -> Boolean`~~ | **Deferred to post-v1.** STIX 2.1 comparison expression parsing requires a non-trivial grammar parser with no established Rust crate. Needs ADR and dependency evaluation before implementation. |
| `mitre_tactic` | `(technique_id: Utf8) -> Utf8` | Static ATT&CK v14 lookup table |
| `severity_label` | `(severity: Int32) -> Utf8` | Configurable threshold mapping |
| `json_extract_string` | `(json: Utf8, path: Utf8) -> Utf8` | JSONPath extraction from event_data column |

## QueryEngine API Contract

The `QueryEngine` provides two execution methods:

1. **`execute(query, scope) -> QueryResult`** — Standard ad-hoc query execution. Creates a `SessionContext`, executes the query, collects results, drops the context, returns `QueryResult` containing `Vec<RecordBatch>`, metadata, and `sensor_errors`.

2. **`execute_scheduled(query, scope) -> ScheduledQueryResult`** — Scheduled query execution for detection integration. Creates a `SessionContext` with `GreedyMemoryPool`, executes the query, collects results, but **does not drop the context**. Returns `ScheduledQueryResult` containing `Vec<RecordBatch>`, metadata, `sensor_errors`, and the live `SessionContext`. The caller (`prism-operations::Scheduler`) is responsible for: (a) computing differential results, (b) registering the differential as a MemTable in the returned `SessionContext`, (c) running detection evaluation, and (d) dropping the `SessionContext` when complete. This enables detection to reuse the same `GreedyMemoryPool` without creating a separate allocation.

   **Error path (REQUIRED):** The `SessionContext` MUST be dropped before any error is propagated from the scheduler task. Use `scopeguard::defer!(drop(ctx))` at the call site or wrap the `SessionContext` in an RAII guard type that drops on any exit path. Rust's implicit drop on stack unwind may defer the drop during `?` propagation, holding the `GreedyMemoryPool` allocation (up to 200 MB) alive until the task stack unwinds. With 16 concurrent schedule tasks, deferred drops during error cascades could temporarily hold 3.2 GB — exceeding the 512 MB RSS budget before the process-level watchdog fires. Verified by VP-036 (integration test).

## IOC File Specification

The `ioc_match` UDF matches field values against named indicator-of-compromise (IOC) lists.

**File format:** One indicator per line, plain text. Lines starting with `#` are comments. Empty lines are ignored. Each line is compiled as a `regex::Regex` pattern (finite automaton, no backtracking — CWE-1333 safe). The compiled patterns are aggregated into a `regex::RegexSet` for efficient multi-pattern matching.

**File location:** IOC files are stored in `{config_dir}/ioc/` (alongside `prism.toml`). Each file is named `{list_name}.ioc` — the `ioc_list_name` parameter in the UDF references the filename without extension (e.g., `ioc_match(src_endpoint.ip, "known_bad_ips")` reads from `ioc/known_bad_ips.ioc`).

**Loading:** IOC files are loaded at startup and on `reload_config` (Tier 3 — per-file independent, same as sensor specs). Each file is compiled into a `RegexSet` and cached in memory. Invalid patterns cause the file to be rejected with `E-IOC-001` (file logged as invalid, other IOC files still load).

**Size limits:** Max 100,000 patterns per IOC file. Max 10 MB per file. Max 50 IOC files. The compiled `RegexSet` memory is included in the baseline process memory budget (~2-5 MB per 10K patterns). Total IOC memory is bounded at ~50 MB worst-case (50 files × 100K patterns × ~10 bytes compiled per pattern).

**Missing file behavior:** If `ioc_match` references a list name that doesn't exist, the UDF returns `false` for all rows (no match) and a WARN is logged. This prevents a missing IOC file from crashing queries but may produce false negatives — the `check_sensor_health` tool reports IOC file status (loaded, missing, invalid) for operational visibility.

**Hot reload:** IOC files participate in `reload_config` Tier 3 (per-file independent validation). Changed files are recompiled; in-flight queries use the pre-reload RegexSet snapshot (arc-swap pattern, CI-002).

## Push-Down Filter Classification

During query planning, each predicate is classified:

| Classification | Behavior | Example |
|---------------|----------|---------|
| **Push-down** | Translated to sensor API parameters, reduces API response size | `severity >= high` on CrowdStrike → `?filter=severity:>=4` |
| **Post-filter** | Applied by DataFusion after materialization | `device.hostname LIKE '%prod%'` |
| **Required** | Must be constrained or query is rejected (DI-021) | `timestamp` on unbounded endpoints |

Push-down capability is declared per column in sensor spec files via `ColumnOptions` (REQUIRED, INDEX, ADDITIONAL, OPTIMIZED) — the same taxonomy as osquery's `QueryContext`.

## Unified Query Surface (CAP-028)

The query engine registers two table types in DataFusion:

| Table Type | Backed By | Lifetime | Example |
|-----------|----------|----------|---------|
| External (composite) | Sensor APIs (ephemeral fan-out) | Per-query | `EVENTS`, `ALERTS`, `DEVICES`, `ASSETS` — map to sensor-specific sources |
| External (specific) | Single sensor API | Per-query | `crowdstrike_detections`, `claroty_devices`, `armis_alerts`, etc. |
| Internal | RocksDB storage domains | Process lifetime | `prism_alerts`, `prism_cases`, `prism_rules` |

**Source disambiguation:** `FROM ALERTS` queries external sensor alert sources only (crowdstrike_detections, cyberint_alerts, etc. per axiql-grammar.md section 11.2). Internal Prism tables use underscore-delimited names that match the AxiQL `identifier` grammar: `FROM prism_alerts`, `FROM prism_cases`, `FROM prism_rules`. These are registered as DataFusion tables alongside the external sensor tables. The `prism_` prefix prevents collision with sensor table names (sensor tables use `{sensor_id}_{source}` format, and no sensor_id is `prism`).

Note: The original `prism.alerts` dotted notation was replaced with `prism_alerts` because dots are not valid in the AxiQL `source` production rule (`identifier` allows only letters, digits, underscores). All references to internal tables must use the underscore form.

Both are queryable via the same `query` MCP tool and same AxiQL syntax. Internal tables are read-only via AxiQL — mutations go through dedicated MCP tools.

### Cross-Source Correlation

Cross-source correlation is achieved through **composite sources**, not SQL JOINs. When an analyst queries `FROM EVENTS`, all event-type sources across sensors are materialized into a single MemTable with unified schema. The `_sensor` and `_source` virtual fields distinguish the origin, enabling correlation via DataFusion's standard `WHERE`, `GROUP BY`, and aggregation:

```sql
-- Correlate events from different sensors by IP
SELECT _sensor, device_hostname, COUNT(*) AS total
FROM EVENTS
WHERE device_ip = '10.0.1.50' AND time > 24h
GROUP BY _sensor, device_hostname
```

AxiQL does **not** include JOIN syntax. The grammar (axiql-grammar.md section 2) defines `sql_statement` with `FROM source` (single source), not `FROM source JOIN source`. Cross-sensor correlation works because the composite source already contains data from all sensors. This is intentional: JOIN syntax would require the analyst to know which sensor-specific tables to join, defeating the unified query surface.

If a future release requires explicit multi-table JOINs (e.g., joining `prism.alerts` with `EVENTS`), the grammar would need a `JOIN` production rule added to section 2. This is deferred — current use cases are fully served by composite sources.

### Virtual Fields

The query engine injects three virtual fields into every materialized RecordBatch:

| Virtual Field | Arrow Type | Description |
|--------------|-----------|-------------|
| `_sensor` | `Utf8` | Source sensor identifier (e.g., `crowdstrike`, `armis`) |
| `_client` | `Utf8` | Client identifier (TenantId value) |
| `_source` | `Utf8` | Specific table name (e.g., `crowdstrike_detections`, `armis_alerts`) |

These fields are prefixed with `_` to distinguish them from OCSF fields. They are queryable in `WHERE`, `GROUP BY`, `ORDER BY`, and `SELECT` clauses. The naming is consistent across all architecture documents — `_sensor`, `_client`, `_source` (with underscore prefix).
