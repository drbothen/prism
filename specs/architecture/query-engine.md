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
// Per-query: create, register, execute, drop
let ctx = SessionContext::new_with_config(session_config);
ctx.register_table("events", mem_table)?;
// Register UDFs
ctx.register_udf(subnet_contains_udf());
ctx.register_udf(time_window_udf());
ctx.register_udf(ioc_match_udf());
// Execute
let df = ctx.sql(&datafusion_sql).await?;
let batches = df.collect().await?;
// ctx drops here — all memory freed
```

### Security UDFs (CAP-027)

| UDF | Signature | Implementation |
|-----|----------|---------------|
| `subnet_contains` | `(cidr: Utf8, ip: Utf8) -> Boolean` | ipnet crate, vectorized over Arrow arrays |
| `time_window` | `(timestamp: Timestamp, duration: Utf8) -> Boolean` | Computes `now - duration <= timestamp` |
| `ioc_match` | `(field: Utf8, pattern_list: Utf8) -> Boolean` | Regex set from configurable IOC files |
| `stix_pattern_match` | `(field: Utf8, pattern: Utf8) -> Boolean` | STIX 2.1 comparison expression subset |
| `mitre_tactic` | `(technique_id: Utf8) -> Utf8` | Static ATT&CK v14 lookup table |
| `severity_label` | `(severity: Int32) -> Utf8` | Configurable threshold mapping |
| `json_extract_string` | `(json: Utf8, path: Utf8) -> Utf8` | JSONPath extraction from event_data column |

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
| External | Sensor APIs (ephemeral fan-out) | Per-query | `crowdstrike.alerts`, `claroty.devices` |
| Internal | RocksDB storage domains | Process lifetime | `prism.alerts`, `prism.cases`, `prism.rules` |

Both are queryable via the same `query` MCP tool and same AxiQL syntax. Cross-source JOINs are supported within a single DataFusion SessionContext. Internal tables are read-only via AxiQL — mutations go through dedicated MCP tools.
