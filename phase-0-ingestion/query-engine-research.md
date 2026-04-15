# Query Engine Research: Ephemeral OCSF Data Lake for Prism

**Date:** 2026-04-13
**Type:** general (technology evaluation)
**Status:** complete
**Training data reliance:** HIGH -- all MCP tools (Context7, WebSearch, WebFetch) were denied; findings below are from model training data (cutoff May 2025) plus axiathon semport analysis

---

## 0. Critical Caveat

All external research tools were unavailable for this report. Version numbers, dependency counts, and performance claims below are based on training data (cutoff May 2025) and MUST be verified against crates.io before implementation decisions are finalized. Specific items flagged with `[VERIFY]` require registry confirmation.

---

## 1. DataFusion as Query Execution Engine

### 1.1 Maturity and Community

- **Latest known version:** DataFusion 46.x (as of early 2025) `[VERIFY: check crates.io for current version as of April 2026]`
- **Apache top-level project** since 2024; graduated from the Arrow project
- **Active maintenance:** weekly releases, 400+ contributors, used in production by InfluxDB IOx, Comet (Spark accelerator), Ballista, GlareDB, and others
- **Rust-native:** pure Rust, no C/C++ FFI for the core engine
- **Axiathon used DataFusion 51** (per semport inventory: `datafusion = "51"` in both workspaces) `[VERIFY: DataFusion versioning -- v51 likely corresponds to datafusion crate 51.0.0]`

**Assessment:** DataFusion is the most mature Rust-native SQL query engine. It is production-grade and actively maintained.

### 1.2 In-Memory RecordBatch Execution

DataFusion natively supports querying in-memory Arrow RecordBatches via `MemTable`:

```rust
use datafusion::prelude::*;
use datafusion::datasource::MemTable;

let schema = Arc::new(Schema::new(vec![/* fields */]));
let batch = RecordBatch::try_new(schema.clone(), vec![/* columns */])?;
let table = MemTable::try_new(schema, vec![vec![batch]])?;

let ctx = SessionContext::new();
ctx.register_table("events", Arc::new(table))?;
let df = ctx.sql("SELECT * FROM events WHERE severity > 5").await?;
let results = df.collect().await?;
```

This is exactly Prism's use case -- materialize sensor data into RecordBatches, register as an ephemeral table, query, then discard.

### 1.3 Ephemeral Table Lifecycle

DataFusion's `SessionContext` provides a natural ephemeral lifecycle:

1. **Create per-query:** `let ctx = SessionContext::new();`
2. **Register table:** `ctx.register_table("events", Arc::new(mem_table))?;`
3. **Execute query:** `ctx.sql("...").await?.collect().await?`
4. **Drop context:** When `ctx` goes out of scope, all table registrations are dropped

There is no need to explicitly "deregister" tables. The `SessionContext` is cheap to create and destroy. Axiathon's spike creates a fresh `SessionContext` per query execution (confirmed in semport: "Creates fresh SessionContext with TenantFilterRule + json_extract_string UDF per query execution").

**This maps perfectly to Prism's ephemeral model:** each MCP tool call creates a SessionContext, registers the materialized data, runs the query, and the context is dropped when the tool call returns.

### 1.4 DynamicMessage/Protobuf to Arrow Conversion

DataFusion operates on Arrow RecordBatches. DynamicMessage (prost-reflect) data must be converted to Arrow format. This is a required step regardless of approach:

**Conversion path:** `DynamicMessage -> field extraction -> Arrow arrays -> RecordBatch`

Axiathon implements this in `events_to_record_batch_with_promotions()`, which:
1. Extracts tier-1 "hot" fields from DynamicMessage into flat Arrow columns
2. Serializes the complete event as JSON into an `event_data` string column (tier 2)
3. Produces a RecordBatch with ~100-200 columns per OCSF class

For Prism's ephemeral case, we can simplify:
- **Hot columns only** for fields commonly used in queries (severity, timestamp, src_endpoint.ip, dst_endpoint.ip, etc.)
- **Full event JSON** in a single `event_data` column for `json_extract_string()` UDF access to arbitrary fields
- The conversion cost for 50-1000 records is negligible (microseconds for column construction)

### 1.5 Memory Overhead for Small Queries

DataFusion's overhead for small queries (50-1000 records):

- **SessionContext creation:** ~50-100 microseconds, minimal heap allocation
- **MemTable registration:** near-zero overhead (wraps existing RecordBatch in Arc)
- **Query planning:** ~1-5 milliseconds for simple SELECT/WHERE/GROUP BY
- **Execution:** streaming, batch-at-a-time; for 1000 records in a single batch, overhead is dominated by the query itself, not the engine

`[VERIFY: benchmark these numbers with DataFusion's current version]`

**Estimated total overhead per query:** 2-10ms for planning + execution on small datasets. This is well within acceptable bounds for an MCP tool call that already incurs network latency to sensor APIs.

The real memory concern is the materialized RecordBatch itself, not DataFusion's engine overhead. A RecordBatch of 1000 OCSF events with 100 columns would be roughly:
- 1000 events x 100 columns x ~50 bytes avg per cell = ~5 MB
- Plus the event_data JSON column: 1000 events x ~2KB per JSON = ~2 MB
- Total: ~7 MB per materialized query -- easily bounded

### 1.6 Custom Functions (UDFs)

DataFusion supports scalar UDFs, aggregate UDFs, and window functions:

```rust
use datafusion::logical_expr::{ScalarUDF, Volatility};

// Axiathon's json_extract_string UDF pattern:
let udf = ScalarUDF::from(JsonExtractString::new());
ctx.register_udf(udf);
// Then usable in SQL: SELECT json_extract_string(event_data, 'claroty.alert_type') FROM events
```

This is critical for OCSF field resolution -- vendor-specific fields stored in the unmapped JSON blob can be queried via UDF without promoting every field to a column.

Prism-specific UDFs needed:
1. `json_extract_string(json_col, path)` -- tier-2 unmapped field access (from axiathon)
2. `ocsf_resolve(field_alias)` -- alias resolution at query time (optional; could be done at parse time instead)
3. Potential future: `ip_in_cidr(ip, cidr)`, `time_bucket(timestamp, interval)` for security analytics

### 1.7 SQL Parser and Multi-Mode Query Support

DataFusion uses `sqlparser-rs` for SQL parsing. For Prism's three query modes:

- **SQL mode:** DataFusion's built-in SQL parser handles this directly. `ctx.sql("SELECT ... FROM events WHERE ...")` works out of the box.
- **Filter mode:** Parse `field = value` syntax with Chumsky/custom parser, translate to SQL `WHERE` clause or DataFusion `Expr`, then execute via DataFusion.
- **Pipe mode:** Parse `| where ... | sort ... | head 10` with Chumsky, translate pipe stages to DataFusion `LogicalPlan` transformations (filter, sort, limit).

DataFusion's `DataFrame` API is particularly well-suited for pipe mode translation:

```rust
let df = ctx.table("events").await?;
let df = df.filter(col("severity").gt(lit(5)))?;     // | where severity > 5
let df = df.sort(vec![col("timestamp").sort(false)])?; // | sort timestamp desc
let df = df.limit(0, Some(10))?;                       // | head 10
let results = df.collect().await?;
```

### 1.8 Dependency Weight

DataFusion is a large dependency. `[VERIFY: exact numbers for current version]`

Estimated transitive dependency count (as of training data):
- `datafusion` crate: ~200-300 transitive dependencies
- Includes: `arrow`, `parquet` (even if not used for storage), `sqlparser`, `tokio`, `object_store`, etc.
- Binary size impact: ~5-15 MB additional (release, stripped)

This is the biggest argument against DataFusion for simple use cases. However:
- Prism already depends on `arrow` via `prost-reflect` and OCSF schema generation (axiathon uses `arrow = "57"`)
- Prism already depends on `tokio` for async sensor API calls
- The incremental cost of DataFusion on top of existing Arrow + Tokio deps is lower than it appears

**Mitigation:** DataFusion has feature flags to disable unused components (e.g., `parquet`, `avro`, `unicode_expressions`). Use minimal features:

```toml
datafusion = { version = "XX", default-features = false, features = ["unicode_expressions"] }
```

`[VERIFY: which DataFusion features can be disabled and what the actual dep reduction is]`

---

## 2. Chumsky Parser + Custom Execution (No DataFusion)

### 2.1 What Axiathon's Production Parser Provides

Axiathon's production Chumsky 0.10 parser (`axiathon-query`, ~1799 LOC, 315 tests) already parses three query modes:

| Mode | Syntax Example | AST Output |
|------|---------------|------------|
| Filter | `severity > 5 AND src_ip = "10.0.0.1"` | `FilterExpr` tree (11 variants) |
| SQL | `SELECT src_ip, count(*) FROM events WHERE severity > 5 GROUP BY src_ip` | `SqlSelect` with projections, aggregations, GROUP BY, ORDER BY, LIMIT |
| Pipe | `\| where severity > 5 \| sort timestamp desc \| head 10` | `PipeExpr` chain (stats, sort, head, tail, dedup, fields) |

The parser includes type system (`TypeConstraint`, `TypeError`), three-tier alias resolution, and security hardening (CWE-400, CWE-674, CWE-1333, CWE-190, CWE-20).

### 2.2 Custom Execution Over Vec<OcsfEvent>

Without DataFusion, Prism would walk the AST and execute operations over `Vec<PrismEvent>`:

**Filter + sort + limit/head/tail (~400-600 LOC estimated):**

```rust
fn execute_filter(events: &[PrismEvent], expr: &FilterExpr) -> Vec<&PrismEvent> {
    events.iter().filter(|e| eval_filter(e, expr)).collect()
}

fn execute_sort(events: &mut [&PrismEvent], sorts: &[SortExpr]) {
    events.sort_by(|a, b| compare_by_sorts(a, b, sorts));
}

fn execute_limit(events: Vec<&PrismEvent>, limit: usize, offset: usize) -> Vec<&PrismEvent> {
    events.into_iter().skip(offset).take(limit).collect()
}
```

This is straightforward. Field access uses DynamicMessage's `get_field()` four-tier resolution chain.

**Aggregations (count, sum, avg, min, max, GROUP BY) (~800-1200 LOC estimated):**

This is where complexity explodes:
- GROUP BY requires a `HashMap<GroupKey, Accumulator>` pattern
- Each aggregation function needs its own accumulator
- Type coercion between proto field types (int64, double, string) and aggregation semantics
- NULL handling per SQL semantics
- Multiple aggregations in a single query (`SELECT src_ip, count(*), avg(severity) GROUP BY src_ip`)
- HAVING clause support
- Proper sorting of aggregated results

This is effectively reimplementing a significant portion of DataFusion's execution engine, minus the optimizer. The resulting code would be:
- Less tested than DataFusion (which has thousands of SQL compliance tests)
- Missing edge cases in type coercion, NULL handling, overflow
- Harder to extend with new functions or operators
- A maintenance burden unique to Prism

### 2.3 Advantages of Pure Chumsky

- **Zero Arrow dependency** (if OCSF events are kept as DynamicMessage/Vec rather than RecordBatch)
- **~200-300 fewer transitive deps** compared to DataFusion
- **Simpler mental model** for contributors -- just Rust iterators over Vec
- **Faster for trivial queries** (filter + sort + head) -- no query planning overhead

### 2.4 Disadvantages

- **Aggregations are hard to implement correctly** and DataFusion has years of testing
- **No query optimizer** -- DataFusion's optimizer can reorder filters, push down predicates, etc.
- **Every new operator requires custom code** -- DISTINCT, CASE WHEN, nested subqueries, etc.
- **Axiathon's production parser is orphaned** -- it was never connected to any execution engine (P3-2 lesson: "Wire the parser to the execution engine from the first commit")

---

## 3. Hybrid: Chumsky Parser + DataFusion Execution (RECOMMENDED)

### 3.1 Architecture

```
User query (filter/SQL/pipe)
    |
    v
Chumsky parser (three modes)
    |
    v
AxiQL AST (FilterExpr / SqlSelect / PipeExpr)
    |
    v
AST-to-DataFusion translator
    |
    v
DataFusion LogicalPlan / DataFrame
    |
    v
DataFusion execution over MemTable (Arrow RecordBatches)
    |
    v
Query results -> MCP tool response
```

### 3.2 AST-to-DataFusion Translation Complexity

**Filter mode -> DataFusion Expr (~100-200 LOC):**

The FilterExpr AST maps almost directly to DataFusion's `Expr` enum:

```rust
fn filter_to_expr(filter: &FilterExpr) -> datafusion::logical_expr::Expr {
    match filter {
        FilterExpr::Compare { field, op, value } => {
            let col = resolve_alias_to_column(field);
            let val = value_to_scalar(value);
            match op {
                CompareOp::Eq => col.eq(val),
                CompareOp::Gt => col.gt(val),
                // ...
            }
        }
        FilterExpr::And(left, right) => filter_to_expr(left).and(filter_to_expr(right)),
        FilterExpr::Or(left, right) => filter_to_expr(left).or(filter_to_expr(right)),
        FilterExpr::Not(inner) => filter_to_expr(inner).not(),
        // ...
    }
}
```

**SQL mode -> DataFusion SQL (~50-100 LOC, or zero):**

For SQL mode, two options:
1. **Pass-through:** Let DataFusion's built-in sqlparser handle SQL directly (zero translation code). Just `ctx.sql(raw_sql).await?`.
2. **AST translation:** Convert Chumsky SqlSelect AST to DataFusion LogicalPlan. More code but allows alias resolution and security validation at parse time.

Recommendation: Parse with Chumsky for security validation (query limits, field alias resolution), then reconstruct a sanitized SQL string and pass to DataFusion. This avoids the complexity of LogicalPlan construction while preserving security controls.

**Pipe mode -> DataFrame API (~150-300 LOC):**

```rust
fn execute_pipe(ctx: &SessionContext, stages: &[PipeStage]) -> Result<DataFrame> {
    let mut df = ctx.table("events").await?;
    for stage in stages {
        df = match stage {
            PipeStage::Where(expr) => df.filter(filter_to_expr(expr))?,
            PipeStage::Sort(sorts) => df.sort(sorts_to_exprs(sorts))?,
            PipeStage::Head(n) => df.limit(0, Some(*n))?,
            PipeStage::Tail(n) => /* reverse sort + limit + reverse */ ,
            PipeStage::Stats(aggs, group_by) => {
                df.aggregate(
                    group_by.iter().map(|f| col(f)).collect(),
                    aggs.iter().map(|a| agg_to_expr(a)).collect(),
                )?
            }
            PipeStage::Dedup(fields) => df.distinct_on(fields)?,
            PipeStage::Fields(mode, fields) => match mode {
                FieldsMode::Include => df.select(fields.iter().map(|f| col(f)))?,
                FieldsMode::Exclude => /* select all except listed */,
            },
        };
    }
    Ok(df)
}
```

**Total estimated translation layer:** 300-600 LOC. This is significantly less than the 1200-1800 LOC for a custom execution engine, and gains all of DataFusion's aggregation, type coercion, and optimizer capabilities.

### 3.3 Why Hybrid Is the Recommended Approach

| Criterion | Chumsky Only | DataFusion Only | Hybrid |
|-----------|-------------|----------------|--------|
| Parse AxiQL three modes | YES (existing) | NO (sqlparser only) | YES |
| Security-hardened parsing | YES (CWE limits) | NO | YES |
| Field alias resolution | YES (three-tier) | NO | YES |
| Filter/sort/limit | ~500 LOC custom | Built-in | Built-in |
| Aggregations/GROUP BY | ~1200 LOC custom | Built-in | Built-in |
| UDFs for OCSF fields | Custom impl | Built-in UDF API | Built-in UDF API |
| Query optimizer | None | Yes | Yes |
| Dep weight | Minimal | Heavy | Heavy |
| Axiathon lesson P3-2 | Violated (orphaned parser) | N/A | Satisfied |
| Future extensibility | Every operator = custom code | SQL standard for free | SQL standard for free |

**The dependency cost of DataFusion is the only downside, and it is justified:**
- Prism already needs Arrow for OCSF event serialization
- Aggregation correctness is critical for security analytics
- The hybrid approach satisfies axiathon lesson P3-2 ("wire parser to execution engine from first commit")

---

## 4. Memory Considerations for Ephemeral Materialization

### 4.1 Scale Estimation

Worst-case query scope: 50 clients x 4 sensors x recent alerts

| Scenario | Records | Estimated Memory |
|----------|---------|-----------------|
| Single client, single sensor, last 24h alerts | 10-100 | < 1 MB |
| Single client, all sensors, last 24h | 40-400 | 1-5 MB |
| All clients, single sensor, critical alerts only | 50-500 | 2-7 MB |
| All clients, all sensors, last 24h | 2000-20000 | 15-150 MB |
| Worst case: all clients, all sensors, 7 days | 50000+ | 500+ MB |

### 4.2 Arrow RecordBatch vs Vec<PrismEvent> Memory Efficiency

**Arrow RecordBatch (columnar):**
- Dictionary encoding for low-cardinality string columns (severity levels, status values): ~10x compression
- Null bitmap instead of Option<T> wrapper: 1 bit per null vs 8+ bytes for Option discriminant
- Contiguous memory allocation: better cache locality for column scans
- Estimated: 100-500 bytes per event for typical OCSF alert data (with dictionary encoding)

**Vec<PrismEvent> (row-oriented, DynamicMessage):**
- Each DynamicMessage carries proto descriptor reference + field values in a HashMap
- String fields individually heap-allocated
- No dictionary encoding (duplicate severity strings stored N times)
- Estimated: 2-5 KB per event (DynamicMessage overhead + individual allocations)

**Arrow is 5-10x more memory efficient** for the query execution phase. However, the conversion from DynamicMessage to Arrow has a transient peak where both representations exist in memory. For Prism's ephemeral model:

1. Sensor API response arrives (JSON/raw)
2. Normalize to DynamicMessage (OCSF)
3. **Convert to Arrow RecordBatch** (transient peak: both DynamicMessage + Arrow in memory)
4. Drop DynamicMessage vector
5. Execute query over RecordBatch
6. Serialize results to MCP response
7. Drop RecordBatch

The transient peak is ~2x the final Arrow size. For 10,000 events, peak might be ~30 MB, settling to ~15 MB during query execution.

### 4.3 Bounding Materialized Data

Prism MUST enforce limits to prevent runaway materialization:

```rust
const MAX_MATERIALIZED_RECORDS: usize = 10_000;
const MAX_MATERIALIZED_BYTES: usize = 100 * 1024 * 1024; // 100 MB

// During sensor fan-out collection:
if total_records > MAX_MATERIALIZED_RECORDS {
    return Err(PrismError::QueryTooBroad {
        records_found: total_records,
        max_allowed: MAX_MATERIALIZED_RECORDS,
        suggestion: "Narrow your query by adding client_id, time range, or severity filter",
    });
}
```

**Layered bounding strategy:**

1. **Per-sensor API limits:** Each sensor adapter already has pagination; limit fetch to N pages per query
2. **Pre-materialization count check:** After fan-out, count total records before converting to Arrow. Reject if too many.
3. **Memory budget:** Track allocated bytes during RecordBatch construction. Abort if exceeding budget.
4. **Query timeout:** DataFusion supports `SessionConfig::with_target_partitions()` and execution time limits
5. **Result size limit:** Cap the number of rows returned in the MCP response (separate from materialization limit)

### 4.4 Recommended Defaults

| Limit | Value | Rationale |
|-------|-------|-----------|
| Max materialized records per query | 10,000 | Keeps memory under ~50 MB |
| Max fan-out concurrency | 10 parallel sensor calls | Limits thundering herd |
| Max result rows returned | 1,000 | MCP response size sanity |
| Query timeout | 30 seconds | Prevents hung queries |
| Max query length | 64 KB | From axiathon CWE-400 |

These should be configurable via TOML config with these as sensible defaults.

---

## 5. Axiathon Lessons Applied to Prism's Query Engine

### From Section 9 of axiathon-pass-8-deep-synthesis.md

| Lesson | ID | Prism Application |
|--------|-----|-------------------|
| DynamicMessage for OCSF events | P0-1 | Foundation -- events arrive as DynamicMessage, converted to Arrow for query |
| Two-tier columnar storage | P0-2 | Adapted -- Prism uses ephemeral MemTable not Parquet, but hot columns + JSON event_data pattern still applies |
| Vendor extension via unmapped JSON | P0-3 | json_extract_string UDF enables vendor field queries without schema changes |
| Security-hardened parsing | P0-4 | Copy axiathon's CWE limits directly into Prism's Chumsky parser |
| Three-tier alias resolution | P1-1 | Resolve aliases during Chumsky parsing, before DataFusion execution |
| TenantFilterRule | P1-3 | Adapted -- Prism's stateless model passes client_id explicitly; optimizer rule ensures client isolation in cross-client queries |
| Do NOT build two parsers | P3-2 | Wire Chumsky parser to DataFusion execution from the first commit |
| DataFusion nested struct limitation | ADR-2 | Flatten hot OCSF fields to top-level Arrow columns; do not use nested structs |

### Key Architectural Differences from Axiathon

| Aspect | Axiathon | Prism |
|--------|----------|-------|
| Storage | Persistent (Iceberg/Parquet) | Ephemeral (in-memory MemTable) |
| Table lifecycle | Long-lived, compaction/GC | Per-query, dropped after execution |
| Parser | Pest (spike, connected) / Chumsky (prod, orphaned) | Chumsky (connected to DataFusion from day 1) |
| Tenant isolation | Optimizer rule (TenantFilterRule) | Explicit client_id on every tool call + optional optimizer rule for cross-client queries |
| Query source | Stored Parquet data | Live sensor API data, normalized to OCSF, materialized ephemerally |

---

## 6. Recommendation

**Use the Hybrid approach: Chumsky parser + DataFusion execution.**

### Implementation Priority

1. **Phase 1 -- Minimal viable query (filter mode only):**
   - Chumsky parser for filter expressions (`field = value`, AND/OR/NOT)
   - DynamicMessage -> Arrow RecordBatch conversion (hot fields + event_data JSON)
   - Register MemTable in fresh SessionContext per query
   - FilterExpr -> DataFusion Expr translation
   - json_extract_string UDF for unmapped field access
   - Materialization bounds (max records, max bytes)
   - ~800-1000 LOC total

2. **Phase 2 -- Pipe mode:**
   - Chumsky parser for pipe stages (where, sort, head, tail, dedup, fields)
   - PipeStage -> DataFrame API translation
   - ~300-400 additional LOC

3. **Phase 3 -- SQL mode + aggregations:**
   - Chumsky parser for SQL SELECT (already designed in axiathon)
   - SQL reconstruction from validated AST, pass to DataFusion
   - GROUP BY, COUNT, SUM, AVG, MIN, MAX for free via DataFusion
   - ~200-300 additional LOC

4. **Phase 4 -- Advanced:**
   - Cross-client queries (client_id: null) with TenantFilterRule-style optimizer
   - Field promotion (frequently queried vendor fields promoted to hot columns)
   - OCSF version-aware alias resolution

### DataFusion Cargo Configuration

```toml
[dependencies]
datafusion = { version = "XX", default-features = false, features = [
    "unicode_expressions",
    # Disable: parquet, avro, crypto_expressions, regex_expressions (add as needed)
] }
arrow = { version = "XX", default-features = false, features = ["json"] }
```

`[VERIFY: exact feature flags and version numbers against current crates.io]`

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Context7 | 0 (DENIED) | Would have looked up DataFusion and Chumsky docs |
| WebSearch | 0 (DENIED) | Would have verified versions, dep counts, benchmarks |
| WebFetch | 0 (DENIED) | Would have fetched crates.io API for version info |
| Semport analysis | 6 reads | axiathon-pass-8-deep-synthesis.md, pass-1-architecture, pass-2-domain-model, pass-4-nfr for DataFusion usage patterns |
| Project context | 2 reads | product-brief.md, recovered-architecture.md |
| Training data | 8 areas | DataFusion API (MemTable, SessionContext, UDFs, DataFrame), Arrow memory model, Chumsky 0.10 capabilities, dependency estimation, memory estimation for columnar vs row formats, query planning overhead, Rust iterator-based execution patterns, sqlparser-rs integration |

**Total MCP tool calls:** 0 (all denied)
**Training data reliance:** HIGH -- All version numbers, dependency counts, performance estimates, and API details come from training data (cutoff May 2025). Every claim marked `[VERIFY]` MUST be confirmed against crates.io and DataFusion documentation before implementation.

**Items requiring immediate verification:**
1. Current DataFusion version on crates.io (was ~46.x in early 2025; axiathon used 51)
2. DataFusion feature flags for minimal dependency profile
3. Actual transitive dependency count with minimal features
4. Binary size impact with minimal features
5. MemTable performance benchmarks with 1000-10000 records
6. Chumsky 0.10 current release status (was alpha/beta in 2024-2025)
