# Pipe Execution Engine Design — ENRICH-4-B

**Status:** Design  
**Authors:** Architect  
**Traces to:** BC-2.11.004 (Pipe Mode), BC-2.19.001 (Infusion UDFs), BC-2.11.006 (Security Limits)  
**Scope:** `crates/prism-query/src/materialization.rs` — `execute_against_session` Pipe/Filter arm

---

## 1. Problem Statement

`execute_against_session` in `materialization.rs` handles `Ast::Pipe(_)` and `Ast::Filter(_)` by
returning the raw union of all fan-out `table_batches` verbatim (lines 875-879). This means:

- `| enrich infusion(col)` — silent no-op; the UDF registered by `register_infusion_udfs` is never
  called because DataFusion never executes a plan.
- `| where field = 'val'` — silent no-op; all rows pass through.
- `| stats count(*)` — silent no-op; raw rows returned instead of the aggregate.
- `| head 10` / `| limit 10` — silent no-op; no row truncation.
- `| sort field asc` — silent no-op; arbitrary order.
- `| fields col1, col2` — silent no-op; full projection.
- `| dedup field` — silent no-op; duplicates retained.

Only `Ast::Sql(Select)` executes through `session_ctx.sql()` and therefore actually invokes
the DataFusion plan, the GreedyMemoryPool, and the registered async UDFs. The fix must make the
Pipe arm take the same proven path.

---

## 2. Lowering Approach Decision

**Decision: Lower pipe AST → executable SQL string, then route through `session_ctx.sql()` — the same
path used by `Ast::Sql(Select)`.**

### Why NOT programmatic DataFusion `LogicalPlan`/`DataFrame` builder

The DataFrame API would work, but has material drawbacks for this codebase:

1. **Import surface.** The DataFrame API requires `datafusion::prelude::*`, `Expr`, `col()`, `lit()`,
   `count()`, `sum()`, `avg()`, etc. — a significant new dependency surface that all needs to be
   tested and maintained for correctness. The `session_ctx.sql()` path already handles all of this
   through DataFusion's own SQL parser.

2. **Memory pool plumbing.** `collect_record_batch_stream` and `map_datafusion_memory_error` are
   already wired in the SQL arm. The DataFrame `collect()` method bypasses this error classifier
   unless explicitly mapped, creating a silent regression risk for BC-2.11.006.

3. **UDF registration.** Registered UDFs are resolved by DataFusion's planner from the SQL string;
   they are also accessible via `DataFrame::select(vec![col("udf_name").call(...)])` but the exact
   call-site syntax differs and is more verbose. The SQL path calls the UDF by name in the same way
   analysts write it, closing the gap between what the analyst types and what executes.

4. **execute_stream vs collect.** The SQL arm uses `execute_stream()` + `collect_record_batch_stream`
   for streaming, memory-budget-aware collection. The DataFrame API's `collect()` bypasses this;
   replicating the same stream collection machinery correctly is additional risk.

5. **Prior investigation recommendation.** The issue description explicitly notes the SQL lowering path
   as the lower-risk option. This design confirms that assessment.

### Why the SQL-lowering approach works

- `build_session_context(memory_pool_bytes)` has already created the `GreedyMemoryPool`-backed context.
- `register_infusion_udfs_with_cache` has already registered all enrichment UDFs in that context.
- `register_mem_table` has already registered the sensor data as `MemTable`s in that context.
- `session_ctx.sql(generated_sql)` plans against those tables and UDFs — exactly as if the analyst
  had written the equivalent SQL query.

### What is NOT `normalize_pipe`

`PqlNormalizer::normalize_pipe` (in `ast.rs`) emits canonical PQL (e.g., `ENRICH ioc_match(field)`),
which is NOT valid SQL. A new function `pipe_to_executable_sql` is required that emits SQL DataFusion
can parse and execute. This is the central new artifact.

---

## 3. Per-Stage SQL Lowering Map

The function `pipe_to_executable_sql(pipe: &PipeQuery, source_table: &str) -> Result<String, PrismError>`
emits a complete SQL `SELECT` statement against the registered `MemTable`(s).

### Source resolution

The pipe `source` field identifies the sensor table. The table name used in the SQL `FROM` clause
must match the name passed to `register_mem_table` — i.e., the underscore form (e.g., `crowdstrike_detections`).
When the source `raw` uses dot notation (`crowdstrike.detections`), the emitter converts it to
underscore form by replacing the first `.` with `_`.

For multi-source pipes or joins, this is more complex (see Section 7). For MVP scope (all stages
except `PipeStage::Join`), the FROM clause is a single table.

### Stage lowering table

| PipeStage variant | SQL equivalent | Notes |
|---|---|---|
| `Enrich(EnrichStage { infusion, field })` | `SELECT *, <infusion>(<field_sql>) AS <infusion_output_col> FROM ...` | `infusion` is the UDF name as registered. `field` is the column passed as input. Output column name: `infusion` (the UDF's registered name). See §3.1 for UDF name resolution. |
| `Where(Predicate)` | `WHERE <predicate_sql>` | Append to the WHERE clause. Multiple `where` stages: AND them together. |
| `Stats(StatsStage)` | `SELECT <agg_exprs> [, <by_fields>] [FROM ...] [GROUP BY <by_fields>]` | Replaces SELECT `*` entirely. After a Stats stage, the SELECT clause is the aggregate list. |
| `Sort(Vec<SortExpr>)` | `ORDER BY <field> [ASC|DESC] [, ...]` | Appended to the query. |
| `Limit(n)` / `head N` | `LIMIT n` | Appended. If multiple Limit stages, the minimum applies. |
| `Tail(n)` | Not lowerable to simple SQL | See §3.2. |
| `Fields(FieldsStage)` | `SELECT <field_list>` (include) or `SELECT * EXCEPT(<fields>)` (exclude) | DataFusion 53.x does not support `EXCEPT`; use a schema-aware projection for the exclude case. See §3.3. |
| `Dedup(Vec<FieldPath>)` | Wrap in subquery: `SELECT DISTINCT <dedup_fields> FROM (inner_query)` or `SELECT DISTINCT *` | Standard SQL DISTINCT. |
| `Join(JoinStage)` | Out of initial scope | Deferred — requires registering the join-target table in the same session. See §7. |

### 3.1 Enrich UDF name resolution

The UDF is registered with the name `descriptor.name` — which is the `[[infusion.fields]]` field's
`name` in the infusion TOML spec (e.g., `ioc_match`, `geoip_country`, `threat_score`).

The pipe syntax is `enrich <infusion_name>(<field_path>)`, where `<infusion_name>` matches
`EnrichStage.infusion`. The emitter produces:

```sql
SELECT *, <infusion_name>(<field_sql>) AS <infusion_name> FROM <source_table>
```

This means:
- The original columns are preserved (`SELECT *` part via CTE or subquery).
- The enriched column is added with the infusion name as its column name.
- The input to the UDF is the raw column value at the `field_path` (e.g., `iocs_value`).

**ENRICH-1 interaction (source_path / JSON-list column).** ENRICH-1 extended the infusion spec with a
`source_path` field that resolves which column of the sensor data to pass into the UDF. That column
(e.g., `iocs_value`, a JSON-list string) is the value the UDF receives. The lowering uses
`EnrichStage.field.segments.join(".")` as the column name directly — it must be a flat column name in
the registered MemTable schema. If the column is a nested path (`device.ip`), DataFusion's SQL parser
handles it as a qualified column reference. This works correctly as long as the column exists in the
Arrow schema of the MemTable (it does — the normalizer flattens OCSF fields).

### 3.2 Tail stage

`PipeStage::Tail(n)` — "take the last N rows" — has no direct SQL equivalent without `ORDER BY`.
In the absence of a preceding `sort` stage, the concept of "last N" is not well-defined on a
DataFrame with no inherent ordering. Two options:

**Option A (recommended for MVP):** Map `tail N` to `LIMIT N` with a reversed ORDER BY on the first
sortable column, or treat it as equivalent to `LIMIT N` on the existing order. This matches the
behavior most analysts expect in a streaming context (last N rows fetched from the sensor).

**Option B:** Reject at plan time with a clear error: "`tail` requires a preceding `sort` stage."

This design recommends **Option A** initially (treat `tail N` as `LIMIT N`) with a TODO comment noting
the semantic gap. The analyst-visible behavior change is minimal since sensor data has no guaranteed
order. This avoids a breaking error on existing queries that use `tail`.

### 3.3 Fields exclude case

`PipeStage::Fields(FieldsStage { include: false, fields })` — "exclude these columns".

DataFusion 53.x SQL does not support `SELECT * EXCEPT(col1, col2)`. The lowering must use a
schema-aware projection:

1. After fan-out, the MemTable schema is known.
2. Compute `retained_cols = schema.fields - excluded_fields`.
3. Emit `SELECT col1, col2, col3, ... FROM source_table` with the retained columns.

This requires the schema to be available at lowering time. The schema is available from the batches
already collected during fan-out (each batch carries its Arrow schema). A helper
`schema_from_batches(batches: &[RecordBatch]) -> Option<Arc<Schema>>` returns the first batch's
schema, which is authoritative for the table.

---

## 4. Multi-stage Query Construction Algorithm

The emitter builds a SQL string by walking the stage list linearly and accumulating clauses:

```
struct PipeQueryBuilder {
    // FROM table
    source_table: String,
    // SELECT items (default: ["*"])
    select_items: Vec<String>,
    // WHERE predicates (ANDed together)
    where_clauses: Vec<String>,
    // GROUP BY fields (set by Stats)
    group_by: Vec<String>,
    // ORDER BY clauses
    order_by: Vec<String>,
    // LIMIT value (minimum of all Limit stages)
    limit: Option<u64>,
    // Whether a Stats stage was encountered (changes SELECT semantics)
    has_stats: bool,
    // Schema of the source MemTable (needed for Fields-exclude projection)
    schema: Option<Arc<Schema>>,
    // List of WITH-clause CTEs for multi-step enrichment
    ctes: Vec<(String, String)>,  // (alias, sql)
}
```

Walk `pipe.stages` in order. For each stage:

- **Enrich:** Wrap the current query in a CTE or subquery, add the UDF call to the SELECT list.
  If `select_items == ["*"]`, transition to `["*, udf_name(field) AS udf_name"]`.
  Multiple enrich stages each add one column; later stages can reference the enriched column.
- **Where:** Push predicate string onto `where_clauses`.
- **Stats:** Replace `select_items` with aggregate expressions. Set `has_stats = true`. Populate
  `group_by` from `by_fields`.
- **Sort:** Push sort expression onto `order_by`.
- **Limit/Head:** Set `limit = min(current_limit, n)`.
- **Tail:** Set `limit = min(current_limit, n)` (with §3.2 semantics).
- **Fields:** Replace `select_items` with the projection (include) or retained schema columns (exclude).
- **Dedup:** Wrap current query in `SELECT DISTINCT ...`.
- **Join:** Return `Err(PrismError::QueryExecutionFailed { detail: "JOIN in pipe mode not yet supported" })` for MVP.

Final assembly:
```sql
SELECT <select_items>
FROM <source_table>
[WHERE <where_clauses joined by AND>]
[GROUP BY <group_by>]
[ORDER BY <order_by>]
[LIMIT <limit>]
```

For Enrich stages and Dedup, the nesting approach is: wrap the preceding result as a subquery
CTE rather than modifying the single-level clauses. This avoids name conflicts when multiple
enrich stages are chained.

**CTE pattern for chained enrichments:**
```sql
WITH base AS (SELECT * FROM crowdstrike_detections),
     enriched1 AS (SELECT *, ioc_match(iocs_value) AS ioc_match FROM base)
SELECT *, geoip_country(src_ip) AS geoip_country FROM enriched1
WHERE ioc_match IS NOT NULL
LIMIT 100
```

---

## 5. Invariant Preservation (BC-2.11.006)

The new code path MUST route through the same machinery as the SQL arm. The implementation
of `execute_against_session` for `Ast::Pipe` becomes:

```rust
Ast::Pipe(pipe) => {
    let pool_bytes = crate::memory::session_memory_pool_bytes(session_ctx);
    let sql = crate::pipe_sql_emitter::pipe_to_executable_sql(pipe, &table_batches)?;
    let df = session_ctx.sql(&sql).await.map_err(|e| {
        tracing::error!(error = %e, "pipe-to-sql DataFusion planning error");
        PrismError::QueryExecutionFailed {
            detail: "pipe SQL planning error: <redacted; see server logs>".to_string(),
        }
    })?;
    let stream = df
        .execute_stream()
        .await
        .map_err(|e| crate::memory::map_datafusion_memory_error(e, pool_bytes))?;
    collect_record_batch_stream(stream, pool_bytes).await
}
```

Invariants satisfied:

| Invariant | Mechanism |
|---|---|
| GreedyMemoryPool memory budget (BC-2.11.006, E-WATCHDOG-001) | `map_datafusion_memory_error` — identical to SQL arm |
| Async UDF execution | `execute_stream()` runs the DataFusion async execution engine which calls `invoke_async_with_args` |
| Streaming collection (no unbounded in-memory accumulation before limit) | `collect_record_batch_stream` — same helper |
| 10K materialized record cap | Already enforced in the fan-out loop before `execute_against_session` is called — unchanged |
| 30s timeout | Enforced by the outer `tokio::time::timeout` in `execute()` / `execute_scheduled()` — unchanged |

The `table_batches` parameter passed to `execute_against_session` is taken as `HashMap<String, Vec<RecordBatch>>`.
The pipe emitter needs the table schema (for `Fields` exclude) from these batches. The function signature
can pass this map by reference to the emitter for schema inspection.

---

## 6. Blast Radius and Regression Strategy

### What changes behavior

Every `Ast::Pipe` query currently returns all raw materialized rows. After this fix:
- `| where` predicates FILTER rows (could return fewer rows than before).
- `| stats` aggregates rows (could return a single aggregate row instead of many sensor rows).
- `| head N` / `| limit N` truncates rows (fewer rows returned).
- `| sort` reorders rows.
- `| fields` projects to a subset of columns.
- `| dedup` removes duplicates.
- `| enrich` adds an enriched column.

All of these are correct per BC-2.11.004 semantics — the previous behavior (returning raw rows
regardless of stages) was WRONG. The blast radius is intentional: we are fixing silent no-ops.

`Ast::Filter` queries are **excluded from this change** (see §6.1).

### 6.1 Scope decision: Ast::Filter is NOT changed in this delivery

`Ast::Filter` is a distinct query mode (`source | predicate` syntax, no pipe stages). It has its
own parsing path and its own materialization behavior. The existing behavior of `Ast::Filter`
(returning raw rows) may be intentional for some callers. Changing `Ast::Filter` in the same
burst as `Ast::Pipe` would increase risk unnecessarily and could break existing tests that rely
on filter-mode returning unfiltered rows (e.g., tests that parse a filter query and assert a
specific raw row count from the stub adapter).

**Decision:** `Ast::Filter` arm continues to return raw unioned batches. It is scoped to a separate
follow-up story (`ENRICH-4-C: filter-mode execution`) once the pipe-mode path is proven stable.
The `Ast::Filter | Ast::Pipe` combined arm in `execute_against_session` is split into two separate
arms.

### 6.2 Existing tests that will change behavior

Any test that:
1. Parses a pipe query with non-trivial stages (where/stats/limit/sort/fields)
2. Calls `execute_against_session` or the full `execute()` pipeline
3. Asserts on the number of returned rows or column contents

...will observe changed behavior. At time of writing, surveying `tests/` and `src/` for such tests:

- `tests/execute_integration_tests.rs` — uses `StubAdapter` with a fixed row set. Any pipe query with
  `| where` or `| head N` will now actually filter/truncate. Tests using pipe-mode without stages
  (source-only) are unaffected.
- `tests/write_pipeline_tests.rs` — tests write paths; unlikely to assert on pipe read behavior.
- `src/materialization.rs` unit tests — the inline tests for `execute_against_session` directly test
  the Filter/Pipe arm returning all rows. These tests MUST be updated to reflect the new behavior.
- `src/engine.rs` unit tests — tests that call `execute()` with pipe queries will observe changed
  row counts. Must audit all `Ast::Pipe` test cases.

**Implementer action:** Before modifying `execute_against_session`, grep for all test bodies that
match `Ast::Pipe` or pass a pipe-syntax query string to `execute()`, and update their expected
assertions to reflect correct stage execution semantics.

### 6.3 Tests that are explicitly unaffected

- All SQL-mode tests (`Ast::Sql(Select)`) — the SQL arm is untouched.
- All filter-mode tests (`Ast::Filter`) — the filter arm is untouched (§6.1).
- All parse-only tests (pipe parser tests, AST structure tests) — no execution path.
- All UDF registration tests (`bc_2_19_001_plugin_udf_registration_test.rs`) — these register and
  invoke UDFs directly in a `SessionContext` without going through `execute_against_session`.

---

## 7. Interaction with ENRICH-1 (source_path / iocs_value column)

ENRICH-1 extended the infusion spec with `source_path`, which designates which column of the sensor
data contains the IOC values to enrich. For example, `source_path = "iocs_value"` means the
`iocs_value` column in the sensor's Arrow schema (a JSON-list string) is the input to the UDF.

The pipe syntax `enrich cyberint_ioc(iocs_value)` has `EnrichStage.field = FieldPath { segments: ["iocs_value"] }`.
The lowering emits:

```sql
SELECT *, cyberint_ioc(iocs_value) AS cyberint_ioc FROM crowdstrike_detections
```

The `cyberint_ioc` UDF receives the `iocs_value` column value (a JSON-array string like
`["1.2.3.4", "5.6.7.8"]`) as a `Utf8` argument. The UDF's `invoke_async_with_args` implementation
(already delivered by ENRICH-1) handles parsing the JSON-list internally.

**Column name validation:** The lowering should verify that `EnrichStage.field` resolves to a column
that exists in the MemTable schema. If the column does not exist (e.g., analyst typo in the field
path), DataFusion will return a planning error — this is correct behavior (E-QUERY-038 / column not
found). No special handling needed; the plan-time column gate and DataFusion's own error surfacing
handle this.

**UDF name vs infusion name:** `EnrichStage.infusion` is the infusion identifier (the `[[infusions]]`
name in the TOML spec). The UDF registered in the `SessionContext` uses `descriptor.name` (the
`[[infusion.fields]]` field name). For the single-field case these are typically identical. For
multi-field infusions (one infusion spec with multiple `[[infusion.fields]]` entries), the analyst
must use the specific field name in the `enrich` stage (e.g., `enrich ioc_match(iocs_value)` where
`ioc_match` is the field name, not the top-level infusion spec name).

The emitter does NOT need to perform any resolution here — it emits `EnrichStage.infusion` as the
UDF function call name directly. DataFusion's `session_ctx.sql()` will resolve it against the
registered UDFs; if the name doesn't match, a planning error is returned.

---

## 8. New Module: `pipe_sql_emitter.rs`

The pipe-to-SQL translation is implemented as a new sub-module of `prism-query`:

**File:** `crates/prism-query/src/pipe_sql_emitter.rs`

Public surface (within crate):
```rust
pub(crate) fn pipe_to_executable_sql(
    pipe: &crate::ast::PipeQuery,
    table_batches: &std::collections::HashMap<String, Vec<arrow::record_batch::RecordBatch>>,
) -> Result<String, prism_core::PrismError>
```

Responsibilities:
- Resolve the source table name (dot → underscore conversion).
- Walk stages and build the SQL string per §4.
- Return `PrismError::QueryExecutionFailed` for unsupported stages (e.g., Join in MVP).
- No I/O, no async — pure function over AST and batch schema metadata.

This function is pure core (no side effects) and is therefore formally verifiable and unit-testable
without a running `SessionContext`.

**Predicate-to-SQL translation:** The `Where` stage lowering requires converting `ast::Predicate` to
a SQL string. The `PqlNormalizer::normalize_predicate` function in `ast.rs` produces canonical PQL
predicates (uppercase keywords, normalized whitespace). However, PQL predicate syntax diverges from
standard SQL in a few places:

| PQL predicate | DataFusion SQL equivalent |
|---|---|
| `field CONTAINS 'x'` | `field LIKE '%x%'` |
| `field STARTSWITH 'x'` | `field LIKE 'x%'` |
| `field ENDSWITH 'x'` | `field LIKE '%x'` |
| `field ICONTAINS 'x'` | `lower(field) LIKE lower('%x%')` |
| `HAS field` | `field IS NOT NULL` |
| `MISSING field` | `field IS NULL` |
| `field IN CIDR '10.0.0.0/8'` | `subnet_contains('10.0.0.0/8', field)` — uses registered UDF |
| `field =~ 'pattern'` | `regexp_match(field, 'pattern') IS NOT NULL` |

A new helper `predicate_to_datafusion_sql(pred: &Predicate) -> Result<String, PrismError>` handles
this translation. It is implemented in `pipe_sql_emitter.rs` and is distinct from
`PqlNormalizer::normalize_predicate` (which emits PQL, not SQL).

---

## 9. Files to Change

| File | Change type | Description |
|---|---|---|
| `crates/prism-query/src/materialization.rs` | Modify | Split the `Ast::Filter | Ast::Pipe` arm into two separate arms. Replace the `Ast::Pipe` arm body with: generate SQL via `pipe_to_executable_sql`, then route through `session_ctx.sql()` → `execute_stream()` → `collect_record_batch_stream`. Update the function doc comment. |
| `crates/prism-query/src/pipe_sql_emitter.rs` | New file | `pipe_to_executable_sql` + `predicate_to_datafusion_sql` + `PipeQueryBuilder`. ~300-400 lines. |
| `crates/prism-query/src/lib.rs` | Modify | Add `pub(crate) mod pipe_sql_emitter;` |
| Existing tests in `materialization.rs` inline tests | Modify | Update `execute_against_session` Pipe-arm tests to assert correct stage semantics. |
| `crates/prism-query/tests/execute_integration_tests.rs` | Modify | Audit all pipe-syntax queries; update expected row counts and column presence assertions. |
| New test file: `crates/prism-query/tests/pipe_execution_tests.rs` (or add to `execute_integration_tests.rs`) | New/modify | Tests per §10. |

### Files explicitly NOT changed in this delivery

- `crates/prism-query/src/ast.rs` — no AST changes needed.
- `crates/prism-query/src/pipe_parser.rs` — no parse changes needed.
- `crates/prism-query/src/engine.rs` — no changes needed; the lowering happens inside `execute_against_session`.
- `crates/prism-query/src/infusion_udf.rs` — no changes needed; UDFs already registered.
- Any BC or spec files — behavior is already specified in BC-2.11.004. No BC amendment needed.

---

## 10. Test Plan

### 10.1 Red Gate tests (new, in `pipe_execution_tests.rs`)

Each test uses `StubAdapter` + `MaterializationContext::new` + `build_session_context` to run
the full materialized pipeline.

| Test name | Description | Assertion |
|---|---|---|
| `test_pipe_enrich_stage_invokes_registered_udf` | Pipe query `FROM tbl \| enrich test_udf(field)`. Register a stub UDF with sentinel output. | Output batch contains the enriched column with the sentinel value. UDF call counter > 0. |
| `test_pipe_where_stage_filters_rows` | Pipe query `FROM tbl \| where status = 'active'`. StubAdapter returns 3 rows: 2 with `status='active'`, 1 with `status='inactive'`. | 2 rows returned, all with `status='active'`. |
| `test_pipe_limit_stage_truncates_rows` | Pipe query `FROM tbl \| head 2`. StubAdapter returns 5 rows. | 2 rows returned. |
| `test_pipe_stats_count_stage` | Pipe query `FROM tbl \| stats count(*)`. StubAdapter returns 4 rows. | 1 row returned with `count(*) = 4`. |
| `test_pipe_sort_stage_orders_rows` | Pipe query `FROM tbl \| sort severity desc`. StubAdapter returns 3 rows with known severity values. | Rows returned in descending severity order. |
| `test_pipe_fields_include_stage` | Pipe query `FROM tbl \| fields + col1, col2`. StubAdapter returns rows with 4 columns. | Output has exactly 2 columns: `col1` and `col2`. |
| `test_pipe_chained_enrich_then_where` | Pipe query `FROM tbl \| enrich test_udf(ip) \| where test_udf IS NOT NULL`. StubAdapter returns 2 rows: 1 with non-null enrichment, 1 with null (UDF returns None for it). | 1 row returned (the non-null enrichment). |
| `test_pipe_memory_budget_error_surfaces_e_watchdog_001` | Pipe query with enrich stage on large dataset. Set `memory_pool_bytes` to a tiny value. | Returns `PrismError::QueryMemoryBudgetExceeded` (E-WATCHDOG-001), not `QueryExecutionFailed`. |

### 10.2 Regression tests to UPDATE (not add)

| Test location | Current assertion | Updated assertion |
|---|---|---|
| `materialization.rs` inline `test_execute_against_session_filter_returns_all` (if exists) | Returns all rows for `Ast::Filter` query | Still returns all rows (Filter arm unchanged). |
| `materialization.rs` inline test for `Ast::Pipe` | Returns all raw rows | Must be updated: pipe with no stages returns all rows; pipe with `| head 1` returns 1 row. |
| `execute_integration_tests.rs` tests using pipe syntax with stages | Asserts raw row count | Update to assert the filtered/truncated/projected result. |

### 10.3 Regression tests NOT affected

All SQL-mode (`SELECT ...`) tests remain green — the SQL arm is unchanged.

---

## 11. Effort and Risk Assessment

| Concern | Assessment |
|---|---|
| **Effort** | Medium. The new `pipe_sql_emitter.rs` module is approximately 350-450 lines of pure Rust (no async, no I/O). The predicate-to-SQL translation is the most complex part (~100 lines to handle all `Predicate` variants). |
| **Risk: SQL generation correctness** | Medium. DataFusion's SQL parser is strict; malformed generated SQL fails at plan time with a `QueryExecutionFailed` error (not a silent wrong-answer). This fail-loud property limits silent correctness bugs. |
| **Risk: `DISTINCT` semantics** | Low. DataFusion's `SELECT DISTINCT` works on Arrow schemas without special casing. |
| **Risk: Stats + WHERE ordering** | Low. The `WHERE` clause is applied before aggregation in SQL; this matches pipe semantics (filter before aggregate). |
| **Risk: CTE support in DataFusion** | Low. DataFusion 53.x supports CTEs. This has been confirmed by the existing SQL tests in the codebase. |
| **Risk: Blast radius on existing tests** | Medium-High. The fix changes behavior for ALL pipe queries. The test audit (§10.2) is mandatory before merge. Run `cargo nextest run -p prism-query` and triage all failures before declaring Green. |
| **Risk: `Tail` semantic gap** | Low. Treating `tail N` as `LIMIT N` is conservative and clearly documented. |
| **Risk: Fields-exclude schema dependency** | Low. The schema is available from the fan-out batches passed into `execute_against_session`. |

---

## 12. Recommended Staging

**Stage 1 (this story, ENRICH-4-B):** Implement the full pipe execution engine including all stage
types except `Join`. This includes enrich, where, stats, sort, limit/head, tail, fields, dedup.
Ship together because partial stage support creates confusing analyst-visible behavior (some stages
execute, others silently no-op).

**Stage 2 (follow-up story, ENRICH-4-C):** Apply the same SQL-lowering approach to `Ast::Filter` —
the filter predicate becomes a `WHERE` clause in a `SELECT * FROM source WHERE predicate` query.
This is simpler than pipe lowering (no stage list) and carries lower risk.

**Stage 3 (future story):** Pipe `Join` stage lowering. Requires registering the join-target table in
the same `SessionContext` (fan-out for the join source) before constructing the SQL.

---

## 13. ADR Recommendation

A full ADR is **not required** for this change. The decision (SQL-lowering over DataFrame API) is
local to the execution path, the rationale is fully documented in §2, and the change does not affect
the architecture's module boundaries, dependency graph, or public API surface. The existing
`architecture/query-engine.md` should have a paragraph added noting that pipe execution is lowered
to SQL at execution time, with a cross-reference to this document.

If the implementer encounters any decision point not covered here (e.g., a DataFusion limitation
that forces a different CTE structure), record it as a decision row in `STATE.md` and surface to
the architect for adjudication before proceeding.
