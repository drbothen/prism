# Pass 2 Deep: Domain Model -- Round 2

**Project:** Axiathon
**Pass:** 2 (Domain Model)
**Round:** 2
**Date:** 2026-04-13

---

## Purpose

This round targets gaps identified in Round 1: parser value types, pipe stage semantics, wildcard semantics, the detection parser grammar, plugin SDK types, and the spike query subsystem types.

---

## 1. Parser Value Types -- Refined

### Duration Value
- Parsed from integer + unit suffix: `30s`, `5m`, `24h`, `7d`
- Units: `s` (seconds), `m` (minutes * 60), `h` (hours * 3600), `d` (days * 86400)
- Stored as `Value::Duration(std::time::Duration)`
- `0s` is valid (zero duration)
- Bare integer without suffix is NOT a duration (e.g., `1024` is `Value::Integer`)
- Float without suffix is NOT a duration (e.g., `2.72` is `Value::Float`)
- Negative durations not supported (`-5m` fails to parse)

### Signed Numeric Values
- Negative integers: `-100` -> `Value::Integer(-100)`
- Negative floats: `-2.75` -> `Value::Float(-2.75)`
- Negative zero integer: `-0` -> `Value::Integer(0)`
- Negative in IN lists: `IN (-1, 0, 1)` -> three Value::Integer
- Bare minus without digits: `-` -> parse error
- Implementation note: Chumsky's `text::int(10)` only parses non-negative digits; negation handled via optional `-` prefix at value level

### String Escape Sequences
- Known escapes: `\"` -> `"`, `\\` -> `\`, `\n` -> newline, `\r` -> carriage return, `\t` -> tab
- Unknown escapes (e.g., `\.` in regex): pass through literally as backslash + character
- Unterminated string after escape -> parse error
- Empty string `""` is valid -> `Value::String("")`

## 2. FilterExpr Variants -- Verified from Tests

### FilterExpr::Has (NEW -- test evidence confirmed)
- Syntax: `HAS field_path`
- Semantics: Field existence check -- true if field has a non-null value
- Test: `parse_filter_has_existence` at parser_test.rs:530

### FilterExpr::Missing (NEW -- test evidence confirmed)
- Syntax: `MISSING field_path`
- Semantics: Field absence check -- true if field is null or not present
- Test: `parse_filter_missing_existence` at parser_test.rs:541

### FilterExpr::Wildcard (NEW -- test evidence confirmed)
- Syntax: `field = "pattern*"` (eq) or `field != "pattern*"` (negated)
- Semantics: Glob-style wildcard matching on string values. String values containing `*` or `?` are automatically promoted from Comparison to Wildcard.
- `negated` flag: `false` for `=`, `true` for `!=`
- Ordering operators (`>=`, `<`) with wildcards are **rejected** (parse error)
- Tests: `parse_filter_wildcard_eq`, `parse_filter_wildcard_ne_preserves_negation`, `parse_filter_wildcard_ordering_op_rejects`, `parse_filter_wildcard_lt_rejects`

### FilterExpr::StringMatch operators
- `CONTAINS`, `STARTSWITH`, `ENDSWITH` -> case-sensitive
- `ICONTAINS`, `ISTARTSWITH`, `IENDSWITH` -> case-insensitive
- Test evidence: `parse_filter_contains` at parser_test.rs:638

### Operator Aliases
- `==` is treated as `=` (CompareOp::Eq) -- test: `parse_filter_double_equals_is_eq`
- `!` prefix is treated as `NOT` -- tests: `parse_filter_bang_prefix_is_not`, `parse_filter_bang_no_space_is_not`

## 3. PipeStage Variants -- All Verified

### PipeStage::Stats
- Syntax: `stats func(field) [AS alias] [, func(field) AS alias ...] [BY|by field1, field2]`
- Functions: count, sum, avg, min, max, dc (distinct count), percentile
- `count` accepts bare form (no parens): `stats count by field`
- Other functions REQUIRE parens: `stats sum by field` -> rejected
- `count(*)` and `count()` both valid (field=None)
- Multiple aggregations: comma-separated: `stats count(*) AS total, sum(bytes) AS total_bytes BY ip`
- `AS` is case-insensitive
- Empty parens for non-count: `stats sum() by field` -> rejected
- Tests: 15 tests in parser_test.rs (lines 908-1310)

### PipeStage::Sort
- Syntax: `sort field1 [asc|desc] [, field2 [asc|desc]]`
- Default direction: ascending (when omitted)
- Test: `parse_pipe_sort` at parser_test.rs:1108

### PipeStage::Head
- Syntax: `head N`
- N is u64
- Test: `parse_pipe_filter_then_head` at parser_test.rs:894

### PipeStage::Tail
- Syntax: `tail N`
- N is u64
- Test: `parse_pipe_tail` at parser_test.rs:1171

### PipeStage::Dedup
- Syntax: `dedup field1 [, field2 ...]`
- Deduplicates by listed fields
- Test: `parse_pipe_dedup` at parser_test.rs:1123

### PipeStage::Fields
- Syntax: `fields [+|-] field1, field2 [, ...]`
- `+` -> `FieldsMode::Include` (select only these fields)
- `-` -> `FieldsMode::Exclude` (remove these fields)
- Bare `fields` (no +/-) -> defaults to Include (per Splunk/Sumo Logic consensus)
- Empty field list -> rejected (at least 1 field required)
- `fields +` with no fields -> rejected
- Tests: 6 tests (lines 1136-1261)

### Multiple Pipe Stages
- Chained via `|`: `filter | stats count by ip | sort count desc | head 10`
- Up to MAX_PIPE_STAGES (64) stages allowed
- Post-parse validation: >64 stages -> error (CWE-400)
- Tests: `parse_pipe_multiple_stages`, `parse_pipe_stages_at_max_succeeds`, `parse_pipe_stages_exceeding_max_returns_error`

## 4. SQL Mode FROM Sources -- Verified

### Source Variants (from tests)
- `FROM EVENTS` -> `Source::Events` (test: `parse_sql_select_star`)
- `FROM ALERTS` -> `Source::Alerts` (test: `parse_sql_from_alerts`)
- `FROM SESSIONS` -> `Source::Sessions` (inferred from enum, no test found)
- `FROM ASSETS` -> `Source::Assets` (inferred from enum, no test found)
- `FROM custom_table` -> `Source::Custom("custom_table")` (inferred from enum, no test found)

### SQL Clauses Verified
- `SELECT * FROM EVENTS` -- star projection
- `SELECT field1, field2 FROM EVENTS` -- field projection
- `SELECT field AS alias FROM EVENTS` -- aliased field
- `SELECT COUNT(*) AS total FROM EVENTS` -- aliased aggregation
- `SELECT ... WHERE condition` -- filter clause
- `SELECT ... GROUP BY field` -- grouping
- `SELECT ... ORDER BY field DESC LIMIT N` -- ordering and limiting

## 5. Spike Query Planner Entities (NEW)

### TenantFilterRule (`spike/crates/axiathon-query/src/tenant_filter.rs`)
- DataFusion optimizer rule that injects `tenant_id = ?` filter into all query plans
- Referenced in storage/tests/tenant_isolation.rs
- Even if user explicitly writes `WHERE tenant_id = 'wrong_tenant'`, the optimizer overrides
- Registered in SessionStateBuilder: `.with_optimizer_rule(Arc::new(TenantFilterRule::new(tenant_id)))`

### QueryEngine (`spike/crates/axiathon-query/src/lib.rs`)
- High-level query execution with COALESCE wrapping for promoted fields
- `execute(query, tenant_id, limit, timeout)` -- entry point
- `refresh()` -- reload table metadata after compaction
- Registers json_extract_string UDF for unmapped field access
- Used in field_promotion.rs tests

### json_extract_string UDF
- DataFusion scalar UDF for extracting fields from JSON strings
- `json_extract_string(unmapped, 'syslog.hostname')` -> string value
- Registered via `axiathon_query::register_json_udfs(&ctx)`

## 6. Plugin SDK Domain Types (NEW)

### PluginManifest (`spike/crates/axiathon-plugin-sdk/core/src/manifest.rs`)
- Plugin metadata: name, version, type, capabilities
- Serializable via serde

### PluginBase trait (`spike/crates/axiathon-plugin-sdk/core/src/base.rs`)
- Common interface for all plugin types
- Methods for lifecycle (init, start, stop)

### Plugin Type Categories (from SDK sub-crates)
- `ingestion/` -- data source connectors (syslog, Claroty)
- `action/` -- response actions
- `network/` -- network protocol parsers
- `notification/` -- alert notification channels (Slack)

### Specific Plugin Types
- **Claroty connector** (`spike/crates/axiathon-plugin-claroty/`): REST API connector, mock server for testing, parser for xDome alerts, Claroty-specific types (ClarotyAlert, ClarotyDevice)
- **Syslog connector** (`spike/crates/axiathon-plugin-syslog/`): UDP/TCP listener, RFC 3164/5424 parser, field mapping to OCSF
- **Firewall plugin** (`spike/crates/axiathon-plugin-firewall/`): firewall log normalization
- **DNS plugin** (`spike/crates/axiathon-plugin-dns/`): DNS query/response normalization
- **GeoIP plugin** (`spike/crates/axiathon-plugin-geoip/`): IP geolocation enrichment
- **Slack plugin** (`spike/crates/axiathon-plugin-slack/`): alert notification via Slack webhooks

### Plugin Registry and Hot Reload
- `PluginRegistry` -- in-memory registry of loaded plugins
- `TenantRegistry` -- per-tenant plugin configurations
- `GlobalRegistry` -- singleton via `once_cell`
- `hot_reload` module -- arc-swap based config reload
- `PluginStore` -- persistent storage for plugin state
- `PluginFactory` -- creates plugin instances from manifests
- **WASM support** (`wasm.rs`) -- plugin loading via WebAssembly (stub)
- **Packaging** (`packaging.rs`) -- `.axpkg` archive format for plugin distribution
  - Round-trip test: `spike/crates/axiathon-plugin/tests/axpkg_round_trip.rs`

## 7. Storage Subsystem Entities -- Refined

### IcebergCatalogConfig (`spike/crates/axiathon-storage/src/catalog.rs`)
- Configuration for SQLite-backed Iceberg catalog
- `new(warehouse_path)` -- creates config pointing to local filesystem catalog

### CompactionTask / CompactionConfig (`spike/crates/axiathon-storage/src/compaction.rs`)
- `max_files_per_partition: usize` -- threshold for triggering compaction
- `check_interval: Duration` -- how often to check
- `with_promotions()` -- enables backfill extraction during compaction
- Compaction reads old files, applies field promotion backfill, writes merged file

### FieldPromotion (`spike/crates/axiathon-storage/`)
- `column_name: String` -- new column name in Iceberg schema
- `json_key: String` -- key to extract from unmapped JSON
- `iceberg_type: Type` -- Iceberg column type (e.g., PrimitiveType::String)
- `promote_fields()` function: idempotent schema evolution -- adds columns only if not present

### GarbageCollection (`spike/crates/axiathon-storage/src/gc.rs`)
- Cleanup of orphaned data files after compaction
- Not deeply analyzed (low priority for Prism)

### ParquetTableProvider (`spike/crates/axiathon-storage/src/reader.rs`)
- DataFusion TableProvider implementation for reading Parquet files
- Used in tenant_isolation tests (though currently `#[ignore]`)

## 8. Vault Entities (NEW)

### Vault (`spike/crates/axiathon-vault/src/vault.rs`)
- Encrypted credential storage
- Key-value store for plugin credentials (API keys, tokens)

### Crypto (`spike/crates/axiathon-vault/src/crypto.rs`)
- Encryption/decryption primitives for vault

## 9. API Route Entities (NEW -- from spike)

### Routes identified from `spike/crates/axiathon-api/src/routes/`:
- `admin.rs` -- admin operations
- `alerts.rs` -- alert listing/management
- `cases.rs` -- case CRUD
- `health.rs` -- health check endpoint
- `ingest.rs` -- event ingestion endpoint
- `mssp.rs` -- multi-tenant MSSP operations
- `plugins.rs` -- plugin management
- `query.rs` -- AxiQL query execution
- `rules.rs` -- detection rule management
- `vault.rs` -- credential vault operations

### Pipeline (`spike/crates/axiathon-api/src/pipeline.rs`)
- Wires ingestion -> detection -> storage pipeline
- Central orchestration point

### State (`spike/crates/axiathon-api/src/state.rs`)
- Axum application state (shared between handlers)
- Contains references to all services

### Tenant Middleware (`spike/crates/axiathon-api/src/middleware/tenant.rs`)
- Extracts TenantContext from request headers
- Applied to all tenant-scoped routes

---

## 10. Proto Schema Structure (Verified)

```
spike/proto/ocsf/v1_7_0/
  objects/
    objects.proto          -- shared objects (NetworkEndpoint, User, Actor, etc.)
    enums/enums.proto      -- shared enums
  events/
    findings/
      findings.proto       -- SecurityFinding (class_uid: 2001)
      enums/enums.proto    -- finding-specific enums
    iam/
      iam.proto            -- Authentication (class_uid: 3002)
      enums/enums.proto    -- IAM-specific enums
    network/
      network.proto        -- NetworkActivity (class_uid: 4001)
      enums/enums.proto    -- network-specific enums
```

Proto package naming: `ocsf.v1_7_0.events.{category}` and `ocsf.v1_7_0.objects`

---

## Delta Summary
- New items added: 12 (TenantFilterRule, QueryEngine, json_extract_string UDF, PluginManifest, PluginBase, 5 plugin connector types, PluginRegistry ecosystem, Vault, CompactionTask, FieldPromotion, API route inventory)
- Existing items refined: All PipeStage variants now have complete syntax documentation with test evidence. FilterExpr::Has/Missing/Wildcard confirmed with tests. Value parsing for Duration/Signed/Escape fully documented. Source enum partially verified.
- Remaining gaps: Source::Sessions/Assets/Custom have no test evidence (may not be implemented). WASM plugin loading is a stub. Vault crypto details not analyzed.

## Novelty Assessment
Novelty: NITPICK
The new items discovered in Round 2 (TenantFilterRule, QueryEngine, plugin types, vault, API routes) are peripheral to the core domain model. The most significant findings were confirmations -- verifying that FilterExpr::Has/Missing/Wildcard and all PipeStage variants have test coverage. The core entity model, relationship graph, and bounded context map from Round 1 remain unchanged. No new entities or relationships were discovered that would change how you'd spec the system.

## Convergence Declaration
Pass 2 has converged -- findings are refinements and confirmations, not gaps. The entity catalog is comprehensive for all production and spike subsystems that matter for Prism's normalization layer.

## State Checkpoint
```yaml
pass: 2
round: 2
status: complete
files_scanned: 42
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
