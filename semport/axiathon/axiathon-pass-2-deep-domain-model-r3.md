# Pass 2 Deep: Domain Model -- Round 3

**Project:** Axiathon
**Pass:** 2 (Domain Model)
**Round:** 3
**Date:** 2026-04-13

---

## Purpose

This round is a correctness audit of R2's NITPICK declaration. R2 discovered 12 new types and called them "peripheral." The human architect flagged this as borderline. This round exhaustively cross-references every `pub struct/enum/trait/type` in the codebase against the R1+R2 catalog to determine whether the domain model is genuinely converged.

---

## 1. Methodology

Executed `grep` for `^pub (struct|enum|trait|type)` across all `.rs` files in the codebase. Produced 210 lines of public type definitions. Cross-referenced each against R1 and R2 catalogs. Classified uncataloged types into three tiers:

- **Tier A: Would change how you spec the system** -- new bounded contexts, state machines, domain relationships, or architectural contracts
- **Tier B: Fill in detail within an already-known bounded context** -- sub-types, helper structs, implementation details within known subsystems
- **Tier C: API request/response DTOs, stubs, and generated types** -- pure wire format, no domain logic

---

## 2. Audit of R2's 12 "Peripheral" Types

R2 discovered these and called them NITPICK:

| Type | R2 Characterization | R3 Reassessment |
|------|---------------------|-----------------|
| TenantFilterRule | "peripheral" | **Correct -- Tier B.** Implementation detail of DataFusion query planning. The contract ("all queries are tenant-filtered") is already documented in architecture. |
| QueryEngine | "peripheral" | **Correct -- Tier B.** Orchestration wrapper. The interesting contracts (COALESCE wrapping, json_extract_string UDF, table-per-class routing) are implementation details of the storage+query layer already documented. |
| json_extract_string UDF | "peripheral" | **Correct -- Tier B.** DataFusion UDF for tier-2 JSON field access. Implementation mechanism for the two-tier storage pattern already documented. |
| PluginManifest | "peripheral" | **Borderline -- Tier A/B.** The manifest defines a plugin contract (PluginId with namespace:name format, PluginKind with 6 variants, JSON Schema config, permissions list). This IS domain knowledge about the plugin extensibility model. However, the plugin system is spike-only and not core to Prism's normalization focus. |
| PluginBase (AxiathonPlugin trait) | "peripheral" | **Borderline -- Tier A/B.** Same reasoning as PluginManifest. Defines the lifecycle contract (initialize, health_check, shutdown) for the entire plugin system. |
| 5 plugin connector types | "peripheral" | **Correct -- Tier C.** Concrete implementations of known plugin patterns. |
| PluginRegistry ecosystem | "peripheral" | **Correct -- Tier B.** Runtime management infrastructure. |
| Vault | "peripheral" | **Correct -- Tier B.** Credential storage implementation. |
| CompactionTask | "peripheral" | **Correct -- Tier B.** Storage maintenance. |
| FieldPromotion | "peripheral" | **Partially incorrect -- Tier B+.** FieldPromotion defines the mechanism for promoting frequently-accessed vendor fields from tier-2 JSON to tier-1 columns. This concept IS architecturally important (it's how the two-tier storage evolves over time) and was already mentioned in the broad sweep's recommendations. But its struct definition (column_name, json_key, iceberg_type) adds no new domain insight. |
| API route inventory | "peripheral" | **Correct -- Tier C.** |

**Verdict on R2's NITPICK call:** R2 was **largely correct** to call these peripheral. PluginManifest and AxiathonPlugin trait are the most significant, but they belong to the plugin bounded context which is spike-only and tangential to Prism's normalization focus. R2's assessment was reasonable but aggressive -- it undercharacterized the plugin SDK by collapsing it to two bullet points when it defines a 6-kind extensibility architecture with 7 trait contracts.

---

## 3. Comprehensive Gap Analysis: Types Not in R1 or R2

### 3.1 Types That Should Have Been in R1/R2 (Tier A: Domain-relevant omissions)

#### ComparisonOp (detection/ast.rs:109) -- MISSED

```rust
#[non_exhaustive]
pub enum ComparisonOp {
    Eq, NotEq, Gt, Gte, Lt, Lte,
}
```

This is **separate from PredicateOp** (which handles field-value predicates) and is used exclusively for correlation threshold comparisons (`count >= N`). R1 cataloged PredicateOp but missed ComparisonOp. This matters for the detection DSL spec because correlation rules use `ComparisonOp` for threshold comparisons while field conditions use `PredicateOp`.

**Impact:** LOW. The concept was captured (R1 shows `MatchClause::Correlation { condition, op, threshold, group_by, window }`) but the `op` type was not separately identified. A spec writer would correctly infer a comparison operator from context.

#### Spike AxiQL Parser Types (spike/axiql.rs) -- MISSED ENTIRELY

R1 and R2 documented the production AxiQL parser (Chumsky-based) thoroughly. But the spike has a **second, completely different AxiQL parser** built on Pest, with its own AST:

- **QueryExpr** -- FieldFilter{field, op, value} | And | Or | Not | MatchAll
- **FilterOp** -- Eq, Ne, Gt, Gte, Lt, Lte, Wildcard (7 variants)
- **TimeRange** -- Last(Duration) | Absolute{from, to} | None
- **ParsedQuery** -- expr + time_range + limit + offset

This uses `field:value` syntax (e.g., `user.name:root`, `severity_id:>=4`) instead of the production parser's `field = value` syntax. Operators are embedded in the value string (`severity_id:>=4` is parsed by stripping `>=` from the value). Time ranges use `--last 1h` CLI-style flags. The Pest grammar is referenced as `axiql.pest`.

**Impact:** MEDIUM. This reveals that axiathon has TWO completely different AxiQL syntaxes: (1) the spike Pest-based `field:value` syntax used by the query planner and (2) the production Chumsky-based `field = value` SQL/pipe/filter syntax. The QueryEngine in the spike uses the Pest parser, not the production parser. This is a significant architectural detail -- the spike's entire query execution path uses a different query language than the production parser being developed. A spec writer needs to know which syntax the detection engine and storage layer actually consume.

#### QueryResult (spike/planner.rs:382) -- MISSED

```rust
pub struct QueryResult {
    pub batches: Vec<RecordBatch>,
    pub total: usize,
    pub query_time_ms: u64,
}
```

The return type of QueryEngine.execute(). Carries timing metadata for observability.

**Impact:** LOW. Standard query result envelope.

#### CaseEvent (case.rs:195) -- MISSED

```rust
pub struct CaseEvent {
    pub case_id: String,
    pub tenant_id: TenantId,
    pub event_type: TimelineEventType,
}
```

Broadcast via tokio broadcast channel when cases are created/modified. This is the event sourcing mechanism for case management -- enables real-time UI updates and audit trails.

**Impact:** LOW. R1 mentioned AlertStore has a broadcast channel, but CaseStore's equivalent (CaseEvent) was not documented. The pattern is the same.

### 3.2 Plugin SDK Trait System (Tier A/B: Significant subsystem detail)

R2 mentioned "PluginBase trait" and "Plugin Type Categories" in passing but did not catalog the actual trait hierarchy. The plugin SDK defines **7 concrete trait contracts**:

| Trait | Crate | Purpose | Key Methods |
|-------|-------|---------|-------------|
| `AxiathonPlugin` | sdk/core | Base lifecycle | `manifest()`, `initialize(config)`, `health_check()`, `shutdown()` |
| `ConnectorPlugin` | sdk/ingestion | Data collection | `start(tx: mpsc::Sender<RawEvent>)`, `stop()`, `health()` |
| `ParserPlugin` | sdk/ingestion | Event normalization | `can_parse(raw)`, `parse(raw) -> Vec<AxiathonEvent>` |
| `AsyncParserPlugin` | sdk/ingestion | Async normalization | Same as ParserPlugin but async |
| `EventEnricher` | sdk/ingestion | Post-parse enrichment | `enrich(event)`, `capabilities()` |
| `ProtocolDissector` | sdk/network | Network parsing | `can_dissect(hint)`, `dissect(data) -> JSON` |
| `NotificationChannel` | sdk/notification | Alert delivery | `send(notification)`, `test_connection()` |
| `ResponseAction` | sdk/action | Automated response | `execute(params)`, `capabilities()`, `validate_params()` |

Key architectural detail: All traits are **object-safe** (verified by compile-time tests in each crate). The connector trait pushes raw events via `mpsc::Sender<RawEvent>` -- a channel-based architecture rather than pull/callback.

Supporting types:
- **RawEvent** -- data: Vec<u8>, source: String, received_at: DateTime, tenant_id: TenantId, metadata: HashMap
- **PluginHealth** -- Healthy | Degraded(String) | Unhealthy(String)
- **PluginId** -- namespace:name format (builtin, community, reverse-domain vendor)
- **PluginKind** -- 6 variants matching the 6 trait types (minus the base trait)
- **EnricherCapabilities** -- input_fields + output_fields
- **EnrichmentResult** -- fields_added + source
- **ActionCapabilities** -- supported_actions + reversible flag
- **ActionResult** -- success + message + reversible + action_id
- **Notification** -- title + body + severity + metadata HashMap
- **ProtocolInfo** -- name + description + default_ports

**Impact assessment:** This is the MOST SIGNIFICANT gap in R1/R2. The plugin SDK defines the extensibility architecture for the entire data ingestion pipeline: how data flows from sources into the system (ConnectorPlugin -> mpsc channel -> ParserPlugin -> EventEnricher -> storage). Prism's normalization layer WILL need an analogous extensibility model. However, this is entirely within the spike codebase and the bounded context was already identified. The question is whether knowing `ConnectorPlugin uses mpsc::Sender<RawEvent>` vs "plugins push events" changes how you'd spec Prism. It adds architectural precision but doesn't change the model.

### 3.3 Plugin Store Lifecycle (Tier B: Subsystem detail)

R2 mentioned "PluginRegistry ecosystem" but missed the PluginStore's lifecycle model:

- **PluginStatus**: Available -> Registered -> Active -> Disabled -> Error (5 states, no enforced transition constraints -- any status can be set directly)
- **PluginCategory**: Network | OtIcs | Cloud | Identity | Endpoint | SecurityTools | Infrastructure (7 categories for UI organization)
- **HealthStatus**: Healthy | Degraded{message} | Unhealthy{message} | Unknown (note: different from PluginHealth in SDK -- this one has Unknown and uses tagged JSON serialization)
- **PluginEntry**: Full catalog entry combining manifest + status + health + config
- **PluginMetrics**: Aggregate counts by type and category

There's also a WASM plugin system (WasmSandboxConfig, WasmPluginHost, WasmPluginInstance) and a packaging system (.axpkg format with AxpkgContents, PackageError, ValidationIssue).

**Impact:** LOW. This is operational infrastructure, not domain modeling. The PluginStatus enum notably does NOT enforce valid transitions (unlike CaseStatus which does) -- it's just a status flag.

### 3.4 Credential Vault Domain (Tier B: Subsystem detail)

R2 mentioned "Vault" in one bullet. The actual domain model:

- **CredentialVault** -- per-tenant encrypted JSON files, AES-256-GCM encryption, Argon2id key derivation
- **CredentialType** -- ApiKey | BasicAuth | BearerToken | Certificate | OAuth2
- **CredentialInfo** -- public metadata (no secret exposed): name, type, description, integration_refs, created_at, last_used_at
- **Integration refs** -- credentials link to plugins that use them (many-to-many via string IDs)
- **Tenant isolation** -- each tenant has a separate encrypted file (`{tenant_id}.vault.json`)
- **SECURITY note:** Spike uses hardcoded static salt for Argon2id -- production must use random per-installation salt

**Impact:** LOW-MEDIUM. The credential-plugin integration ref pattern is worth knowing, and the tenant isolation model (file-per-tenant) is an architectural decision. But this is firmly within the vault bounded context already identified.

### 3.5 Claroty xDome Types (Tier C: Vendor-specific wire types)

13 types for the Claroty REST API: request/response shapes, cursor-based pagination, alert/device join logic. The `EnrichedXDomeAlert` enrichment pattern (join alerts with device relations, skip retired devices) is the most interesting -- it shows how vendor data normalization works in practice.

**Impact:** LOW. These are vendor-specific implementation types. The pattern of "fetch from vendor API, join related entities, normalize to OCSF" is already captured at the architectural level.

### 3.6 API Request/Response DTOs (Tier C: Wire format)

~25 types for REST API request/response shapes. Standard CRUD patterns. DetectionRule in state.rs is a thin wrapper around detection::Rule for API serialization.

The AppState struct composition is worth noting:
- DetectionServices: RuleEngine + CorrelationState + SequenceState + AlertStore + CaseStore + rules
- StorageServices: StorageWriter + CompactionTask + GcTask + Catalog + promoted_fields
- PluginServices: PluginStore + PluginRegistry + NativePluginFactory + TenantPluginManager
- CredentialServices: CredentialVault

This shows how the bounded contexts compose at the application level.

**Impact:** LOW. These are assembly-level types, not domain types.

### 3.7 Miscellaneous Uncataloged Types

- **Annotation** struct (case.rs:110): id, annotation_type, content, author, created_at -- R1 mentioned it as part of Case but never listed its fields
- **TimelineEntry** struct (case.rs:133): id, timestamp, event_type, description, actor -- same situation
- **GcConfig/GcTask** (gc.rs): garbage collection configuration and task for orphaned data files
- **SyslogConfig** (syslog connector): host, port, default_tenant_id, max_message_size, FramingMode
- **FramingMode** (syslog connector): LineFeed | OctetCounting -- two TCP framing modes for syslog
- **ConnectorMetrics** (syslog connector): events_received, bytes_received, parse_errors, last_event_at

**Impact:** LOW. All within already-documented subsystems.

---

## 4. Cross-Referencing the Two AxiQL Parsers

This is the most significant finding of R3. The codebase contains two completely different AxiQL implementations:

| Aspect | Production Parser (Chumsky) | Spike Parser (Pest) |
|--------|---------------------------|---------------------|
| Crate | `crates/axiathon-query` | `spike/crates/axiathon-query/src/axiql.rs` |
| Parser framework | Chumsky 0.10 | Pest |
| Syntax style | `field = "value"` (SQL-like) | `field:value` (Lucene-like) |
| Modes | Filter, SQL SELECT, Pipe | Single filter mode |
| AST root | `AxiQLStatement` (3 variants) | `QueryExpr` (5 variants) |
| Time ranges | Not in parser | `--last 1h`, `--from ... --to ...` |
| Pagination | Not in parser | `limit` and `offset` on ParsedQuery |
| Used by | Nothing in spike | `QueryEngine` (spike planner.rs) |
| Error recovery | Planned (Chumsky supports it) | None (Pest) |
| Security limits | Max length, depth, stages | None |

The spike's query execution path is: `axiql.rs::parse_axiql()` -> `planner.rs::QueryEngine` -> DataFusion. The production parser is NOT wired to any query engine. This means the entire query-to-execution pipeline in the spike uses a different, simpler query language than the one being developed for production.

**R1/R2 completely missed this duality.** The broad sweep and R1 documented the production Chumsky parser exhaustively but never mentioned that the spike uses a different parser. R2 mentioned "QueryEngine" but didn't note that it consumes the Pest-based syntax, not the Chumsky-based syntax.

---

## 5. Verification: Are All Bounded Contexts Captured?

| Bounded Context | R1/R2 Coverage | R3 Findings |
|-----------------|---------------|-------------|
| Core Domain (axiathon-core) | COMPLETE | No new types |
| OCSF Event Modeling (spike/core) | COMPLETE | No new types |
| Query Language -- Production (axiathon-query) | COMPLETE | No new types |
| Query Language -- Spike (spike/axiathon-query) | **MISSED** | 4 new AST types + dual-parser duality |
| Detection (spike/detection) | NEAR-COMPLETE | ComparisonOp missed, CaseEvent missed, Annotation/TimelineEntry fields not listed |
| Storage (spike/storage) | NEAR-COMPLETE | GcConfig/GcTask not listed |
| Plugin SDK (spike/plugin-sdk) | **INADEQUATELY COVERED** | 7 traits + 10 supporting types not cataloged |
| Plugin Infrastructure (spike/plugin) | MENTIONED BUT NOT DETAILED | 15+ types not cataloged |
| Vault (spike/vault) | MENTIONED IN ONE BULLET | 3 types not cataloged |
| API Layer (spike/api) | ROUTE INVENTORY ONLY | 25+ DTO types, AppState composition not detailed |
| Vendor Connectors (spike/plugin-*) | MENTIONED AS LIST | 13+ Claroty types, 5+ syslog types not cataloged |

---

## 6. Assessment: Would the Uncataloged Types Change How You'd Spec Prism?

### Types that WOULD change the spec (Tier A):

1. **The spike AxiQL parser duality** (QueryExpr, FilterOp, TimeRange, ParsedQuery). A spec writer needs to know that the spike's query execution pipeline uses Lucene-style `field:value` syntax, not the SQL-like syntax being developed in production. The production parser is orphaned -- not connected to any execution engine. This affects decisions about which query syntax Prism should adopt and how to wire parsing to execution.

2. **Plugin SDK trait hierarchy** (7 traits). The ingestion pipeline architecture is: ConnectorPlugin(mpsc::Sender<RawEvent>) -> ParserPlugin(can_parse, parse) -> EventEnricher(enrich in-place) -> StorageWriter. This is the concrete data flow pattern. A spec writer designing Prism's normalization pipeline needs this level of detail to know whether to adopt or diverge from this architecture.

### Types that would NOT change the spec (Tier B/C):

Everything else -- plugin store lifecycle, vault crypto, Claroty wire types, API DTOs, WASM stubs, packaging, etc. These are implementation details within already-identified bounded contexts.

---

## Delta Summary
- New items added by R3: 4 spike AxiQL AST types (QueryExpr, FilterOp, TimeRange, ParsedQuery), ComparisonOp, CaseEvent, complete plugin SDK trait hierarchy (7 traits + 10 supporting types)
- Existing items refined: Plugin bounded context expanded from 2-bullet description to full trait/type catalog
- Remaining gaps: Syslog parser internals, detection parser grammar details (`.pest` file for spike axiql not read), connector implementation details

## Novelty Assessment
Novelty: **SUBSTANTIVE**

R2's NITPICK declaration was **incorrect**, though understandably so. The 12 types R2 discovered were indeed individually peripheral, and R2's logic was defensible. But R2 missed the bigger picture:

1. **The dual-parser architecture** was completely undocumented. The spike's entire query execution pipeline uses a different query language (Lucene-style `field:value`) than the production parser (SQL-like `field = value`). This is a significant architectural fact that affects how Prism should approach query language design.

2. **The plugin SDK trait hierarchy** defines the concrete ingestion pipeline architecture: Connector -> Parser -> Enricher -> Storage. This is not "peripheral" -- it's the data flow architecture for the entire system. R2 collapsed 7 trait contracts and 10 supporting types into "PluginBase trait" and "Plugin Type Categories."

Both findings change how you'd spec Prism's normalization layer. The dual-parser issue affects query language decisions. The plugin trait hierarchy affects pipeline architecture decisions.

## Convergence Declaration
Another round needed -- the dual-parser finding is substantive and its implications for the domain model (particularly the relationship between the query language bounded context and the storage/execution bounded context) need to be integrated into the entity catalog and relationship map. Round 4 should:
1. Add the spike AxiQL types to the entity catalog
2. Add the plugin SDK traits to the entity catalog
3. Update the bounded context map to show the spike vs. production query language split
4. Update the relationship map to show the ingestion pipeline data flow through plugin traits
5. Verify no further gaps remain after these additions

## State Checkpoint
```yaml
pass: 2
round: 3
status: complete
files_scanned: 55
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
next_action: R4 should integrate dual-parser and plugin SDK findings into entity catalog
```
