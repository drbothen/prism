# Axiathon Codebase Ingestion Analysis

**Project:** Axiathon -- Open-source Security Lake / SIEM
**Analyzed:** 2026-04-13
**Purpose:** Reference ingestion for Prism's data normalization layer
**Focus areas:** OCSF event modeling, protobuf schema design, query language (AxiQL), data normalization, OCSF extensions

---

## Executive Summary

Axiathon is an early-stage Rust SIEM built on OCSF (Open Cybersecurity Schema Framework) v1.7.0 with a Cargo workspace of 8 crates. Only two crates have substantial implementation: `axiathon-core` (domain types, shared query primitives) and `axiathon-query` (AxiQL parser). The `spike/` directory contains a much richer prototype including protobuf-backed OCSF events, Arrow schema generation, a detection DSL with parser and engine, Iceberg-based storage, a plugin system, and a syslog/Claroty ingestion pipeline.

The codebase demonstrates a mature approach to OCSF adoption: events are backed by proto3 descriptors accessed via `prost-reflect` DynamicMessage, enabling runtime field access across all 83 OCSF event classes without per-type code. The two-tier columnar storage pattern (hot Parquet columns + full event JSON) and the three-tier field alias system (analyst shortcut -> AxiQL canonical -> OCSF canonical) are particularly relevant for Prism.

---

## Pass 0: Inventory

### Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Language | Rust | Edition 2024, MSRV 1.85 |
| Event Schema | OCSF | v1.7.0 |
| Storage | Apache Parquet + Apache Iceberg | arrow 57, parquet 57 |
| Query Engine | Apache DataFusion | 51 |
| Parser (AxiQL) | Chumsky | 0.10 |
| Parser (Detection DSL) | Pest | (spike only) |
| Protobuf | prost + prost-reflect | prost 0.14, prost-reflect 0.15 |
| HTTP Framework | Axum | 0.8 |
| Serialization | serde + serde_json | 1.x |
| Testing | insta (snapshot) + proptest (property) | 1.x |
| Error handling | thiserror | 2 |
| Observability | tracing + tracing-subscriber | 0.1 / 0.3 |

### Crate Manifest

| Crate | Status | Lines (approx) | Purpose |
|-------|--------|----------------|---------|
| `axiathon-core` | **Implemented** | ~600 | OCSF types, TenantContext, error types, shared query primitives (FieldRef, CompareOp, Value) |
| `axiathon-query` | **Implemented** | ~1200 | AxiQL parser (Chumsky 0.10), AST, type system, field alias registry, multi-version OCSF support |
| `axiathon-detection` | Stub | 2 | Detection engine (see spike for prototype) |
| `axiathon-ingestion` | Stub | 2 | Data ingestion pipeline |
| `axiathon-storage` | Stub | 2 | Tiered storage with Parquet/Iceberg |
| `axiathon-client` | Stub | 2 | API client |
| `axiathon-server` | Stub | 2 | API server (Axum) |
| `axiathon-ot` | Stub | 2 | OT/ICS protocol parsing |

### Spike Crate Manifest (prototype implementations)

| Spike Crate | Lines (approx) | Purpose |
|-------------|----------------|---------|
| `spike/axiathon-core` | ~2,977 | Proto-backed OCSF events, DynamicMessage wrapper, Arrow schema generation, field catalog, proto integration |
| `spike/axiathon-detection` | ~1500 | Detection DSL (Pest grammar), rule engine, correlation, sequence detection |
| `spike/axiathon-storage` | ~800 | Iceberg catalog, Parquet writer, compaction, garbage collection |
| `spike/axiathon-query` | ~1,944 | DataFusion AxiQL transpiler, tenant filter injection, query planner, Pest-based parser |
| `spike/axiathon-api` | ~600 | Axum REST API, tenant middleware, pipeline wiring |
| `spike/axiathon-plugin-*` | ~1000 | Plugin SDK, syslog/Claroty/firewall/DNS connectors |
| `spike/axiathon-vault` | ~300 | Credential vault |

### Entry Points

- `crates/axiathon-core/src/lib.rs` -- foundation types re-exports
- `crates/axiathon-query/src/lib.rs` -- query module re-exports
- `crates/axiathon-query/src/parser.rs` -- `parse_axiql()` main entry
- `spike/crates/axiathon-api/src/main.rs` -- API server entry
- `spike/crates/axiathon-core/src/event.rs` -- `AxiathonEvent` DynamicMessage wrapper

---

## Pass 1: Architecture

### Layer Structure

```
axiathon-core          (no internal dependencies)
  |
  +-- axiathon-storage
  |     |
  |     +-- axiathon-ingestion
  |     +-- axiathon-ot
  |     +-- axiathon-query
  |           |
  |           +-- axiathon-detection
  |
  +-- axiathon-server  (depends on all domain crates)
  +-- axiathon-client  (API client only, depends on core)
```

Dependencies are **strictly acyclic** -- enforced by `depgraph-rules.toml` and a `cargo-deny` check. Shared query primitives (`FieldRef`, `CompareOp`, `Value`, `StringOp`) live in `axiathon-core` specifically to break the circular dependency between `axiathon-query` and `axiathon-detection` -- both need these types.

### Data Flow (from spike)

```
Sources (syslog, Claroty API, etc.)
  |
  v
Ingestion (parsers normalize raw data -> OCSF events)
  |
  v
Detection Engine (inline, sub-second)
  |           |
  v           v
Storage     Alerts -> Integrations (Slack, etc.)
  |
  v
Query Engine (DataFusion + AxiQL)
  |
  v
TUI / WebUI / API
```

### Key Architectural Decisions

1. **OCSF as canonical schema** -- all events normalize to OCSF regardless of source
2. **Proto-backed events via prost-reflect** -- DynamicMessage enables runtime field access without per-class code
3. **Two-tier columnar storage** -- hot Parquet columns for frequent queries + event_data JSON for full access
4. **Chumsky for AxiQL** (production), **Pest for Detection DSL** (spike) -- different parsers for different needs
5. **Tenant context is always explicit** -- `TenantContext` / `SystemContext` in every function signature
6. **Detection runs inline with ingestion** -- sub-second alert latency

### Cross-Cutting Concerns

- **Tenant isolation**: `TenantId` newtype, `TenantContext` with private fields, `TenantScoped` trait
- **Error handling**: `thiserror` enums, `#[non_exhaustive]` on all extensible enums, security comments on Display impls
- **Observability**: tracing spans with tenant_id and trace_id (target architecture)
- **Security**: `#![forbid(unsafe_code)]` in every crate, CWE citations on security decisions

---

## Pass 2: Domain Model

### OCSF Event Hierarchy

The OCSF schema organizes events into **categories** and **classes**:

| Category UID | Category | Implemented Classes |
|-------------|----------|-------------------|
| 2xxx | Findings | `SecurityFinding` (class_uid: 2001) |
| 3xxx | Identity & Access | `Authentication` (class_uid: 3002) |
| 4xxx | Network Activity | `NetworkActivity` (class_uid: 4001) |

Each event class shares **common base fields** (from OCSF base event):

```
class_uid, class_name, category_uid, category_name,
severity_id, severity, activity_id, activity_name,
status_id, status, status_detail, status_code,
time, start_time, end_time, duration, count,
message, raw_data, type_uid, type_name,
unmapped (catch-all JSON string for vendor extensions)
```

### Core Domain Types

#### AxiathonEvent (spike, production-grade design)

```rust
pub struct AxiathonEvent {
    pub tenant_id: TenantId,       // Not in OCSF; Axiathon-specific
    pub event_uid: String,          // UUIDv7; not in OCSF
    pub inner: DynamicMessage,      // prost-reflect, holds any OCSF class
}
```

Field access via `get_field(path)` supporting:
- Flat fields: `severity_id`, `message`, `class_uid`
- Dotted paths: `src_endpoint.ip`, `user.name`
- Axiathon-specific: `tenant_id`, `event_uid`
- Vendor extensions via unmapped JSON: `claroty.alert_type`

#### OcsfEvent (spike, early version -- enum-based)

```rust
pub enum OcsfEvent {
    AuthenticationActivity(AuthenticationActivity),
    SecurityFinding(SecurityFinding),
}
```

This enum approach was replaced by the DynamicMessage approach to scale to all 83 OCSF classes.

#### FieldRef (shared query primitive)

```rust
pub struct FieldRef {
    segments: Vec<FieldSegment>,  // Named("ip") or Index("answers", "0")
    has_array: bool,
}
```

Supports: `src_endpoint.ip`, `answers[0].value`, `unmapped['vendor.field']`

#### Value (query literal)

```rust
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Regex { pattern: String, flags: String },
    Duration(Duration),
}
```

#### TenantId

- Production: validated as UUID format
- Spike: validated as alphanumeric + hyphens + underscores
- Max length: 128 chars
- Dual constructor pattern: `new()` validates (trust boundaries), `new_unchecked()` skips (tests/internal)

#### SeverityId (OCSF 1.7.0)

```
Unknown = 0, Informational = 1, Low = 2, Medium = 3,
High = 4, Critical = 5, Fatal = 6
```

### OCSF Object Model (from protobuf)

Key shared objects used across event classes:

| Object | Key Fields | Purpose |
|--------|-----------|---------|
| `NetworkEndpoint` | ip, port, hostname, domain, mac | Source/destination endpoints |
| `User` | name, uid, email_addr, domain | Actor identity |
| `Actor` | user, process, session, app_name | Who performed the action |
| `Metadata` | product, version, uid, correlation_uid | Event source metadata |
| `Attack` | tactic, technique, sub_technique, version | MITRE ATT&CK mapping |
| `Device` | hostname, ip, os, type_id | Target device |
| `Finding` | type, title, desc, uid | Security finding details |
| `Enrichment` | name, value, type | Post-ingestion enrichments |

### Vendor Extension Pattern (Claroty xDome example)

Vendor-specific fields not in OCSF go into the `unmapped` JSON string field:

```json
{
  "claroty.alert_type": "unauthorized_access",
  "claroty.device_type": "PLC",
  "claroty.purdue_level": "Level_1",
  "claroty.network_zone": "OT-Zone-1",
  "claroty.risk_score": 8.5
}
```

The detection engine accesses these via `event.get_field("claroty.alert_type")` which:
1. Checks proto fields first (returns immediately if found)
2. Falls back to parsing the `unmapped` JSON blob
3. Tries both direct key lookup and dotted path navigation

### Ubiquitous Language

| Term | Definition |
|------|-----------|
| **Event** | An OCSF-normalized security occurrence |
| **class_uid** | OCSF event class identifier (e.g., 3002 = Authentication) |
| **category_uid** | OCSF event category (e.g., 3xxx = IAM) |
| **severity_id** | OCSF severity level 0-6 |
| **activity_id** | What happened (e.g., 1=Logon, 2=Logoff for Authentication) |
| **status_id** | Outcome (e.g., 1=Success, 2=Failure) |
| **src_endpoint** / **dst_endpoint** | Network endpoints (IP, port, hostname) |
| **unmapped** | JSON catch-all for vendor extensions not in OCSF standard |
| **FieldRef** | Dotted path reference to an OCSF field |
| **TenantContext** | User-initiated operation context (tenant, user, roles, trace) |
| **SystemContext** | Background job context (tenant, component, trace) |
| **hot column** | Tier 1 Parquet column for fast predicate pushdown |
| **event_data** | Tier 2 JSON column containing complete event |

---

## Pass 3: Behavioral Contracts

### BC-001: TenantId.new() rejects invalid UUIDs

**Preconditions:** Input string provided
**Postconditions:** Returns `Err` for empty, >128 chars, or non-UUID strings. Returns `Ok(TenantId)` for valid UUIDs.
**Error Cases:** Empty -> "TenantId cannot be empty"; >128 chars -> "exceeds 128 characters"; non-UUID -> "must be a valid UUID"
**Evidence:** `crates/axiathon-core/tests/core_types_integration.rs`, `crates/axiathon-core/tests/property_types.rs`
**Confidence:** HIGH (directly from test assertions)

### BC-002: FieldRef.new() validates field paths

**Preconditions:** Path string provided
**Postconditions:** Rejects empty paths and paths with empty segments. Parses bracket notation (`answers[0]`). Splits on dots but respects brackets.
**Error Cases:** Empty -> "cannot be empty"; `a..b` -> "contains empty segment"
**Evidence:** `crates/axiathon-core/tests/property_fieldref.rs`
**Confidence:** HIGH (property-based tests)

### BC-003: parse_axiql() returns partial results on error

**Preconditions:** Input string (any)
**Postconditions:** Returns `(Option<AxiQLStatement>, Vec<AxiQLError>)`. Empty input returns EOF error. Input >64KB returns size error. Successful parse returns Some + empty errors. Parse failure returns None + error list.
**Error Cases:** Empty -> unexpected_eof; >64KB -> query limit exceeded; >128 nesting -> depth exceeded; >64 pipe stages -> stage limit exceeded
**Evidence:** `crates/axiathon-query/tests/parser_test.rs`, doc examples on `parse_axiql()`
**Confidence:** HIGH (extensive test coverage)

### BC-004: AxiQL supports three query modes

**Preconditions:** Valid AxiQL query string
**Postconditions:**
- **Filter mode** (Splunk-style): `severity = "high" AND src_endpoint.ip IN CIDR "10.0.0.0/8"` -> `AxiQLStatement::Filter(FilterExpr)`
- **SQL mode**: `SELECT ... FROM events WHERE ... GROUP BY ... ORDER BY ... LIMIT N` -> `AxiQLStatement::Select { ... }`
- **Pipe mode** (KQL-style): `severity = "critical" | stats count by src_endpoint.ip | sort count desc | head 10` -> `AxiQLStatement::Pipe { filter, stages }`
**Evidence:** Parser doc examples, `crates/axiathon-query/tests/parser_test.rs`
**Confidence:** HIGH

### BC-005: AxiQL keywords are case-insensitive

**Preconditions:** Any keyword in query (SELECT, WHERE, AND, OR, NOT, etc.)
**Postconditions:** Keywords match regardless of case (SELECT = select = SeLeCt)
**Implementation:** `kw()` helper uses `text::ident().try_map()` with `eq_ignore_ascii_case`
**Evidence:** Parser source comments citing AxiQL grammar spec line 67
**Confidence:** HIGH

### BC-006: AxiQL regex patterns validated at parse time

**Preconditions:** Query contains `field =~ "pattern"` or `field MATCHES "pattern"`
**Postconditions:** Invalid regex syntax returns parse error immediately. Pattern length >1024 bytes rejected. Uses Rust `regex` crate (finite automaton, immune to catastrophic backtracking / CWE-1333).
**Evidence:** `crates/axiathon-query/src/parser.rs` lines 570-590
**Confidence:** HIGH

### BC-007: CIDR notation validated at parse time

**Preconditions:** Query contains `field IN CIDR "x.x.x.x/n"`
**Postconditions:** Invalid IP address or out-of-range prefix (>32 for IPv4, >128 for IPv6) returns parse error. Supports both IPv4 and IPv6.
**Evidence:** `validate_cidr()` in parser.rs
**Confidence:** HIGH

### BC-008: Three-tier field alias resolution

**Preconditions:** Field reference in AxiQL query
**Postconditions:** Resolution order:
1. Alias registry (analyst shortcuts + AxiQL canonical) -> OCSF canonical
2. Direct OCSF field match -> pass through
3. Unknown -> pass through (supports vendor extensions)

**Default aliases:**
- `src_ip` -> `src.ip` -> `src_endpoint.ip` (OCSF)
- `dst_ip` -> `dst.ip` -> `dst_endpoint.ip` (OCSF)
- `user` -> `user.name` -> `actor.user.name` (OCSF)
- `hostname` -> `device.hostname` -> `device.hostname` (OCSF)

**Evidence:** `crates/axiathon-query/src/aliases.rs`, `crates/axiathon-query/tests/aliases_test.rs`
**Confidence:** HIGH

### BC-009: AxiathonEvent.get_field() resolves fields from multiple sources

**Preconditions:** Event + field path string
**Postconditions:** Resolution order:
1. Axiathon-specific fields (tenant_id, event_uid) -> immediate return
2. Proto descriptor fields (flat + dotted paths via recursive descent) -> immediate return
3. Unmapped JSON blob (vendor extensions) -> parse JSON, try direct key then dotted navigation
4. Not found -> None

**Important:** Empty proto3 default strings ("") treated as absent (returns None)
**Evidence:** `spike/crates/axiathon-core/src/event.rs` tests, 8 test functions
**Confidence:** HIGH

### BC-010: Detection rule engine evaluates single-event rules

**Preconditions:** Rules loaded, events flowing
**Postconditions:** Each event evaluated against all enabled single-event rules. Disabled rules skipped. Regex patterns pre-compiled and cached. Returns list of `RuleMatch` (rule + event pairs).
**Error Cases:** Invalid regex in rule -> warning log, pattern never matches (not a crash)
**Evidence:** `spike/crates/axiathon-detection/src/engine.rs` tests
**Confidence:** HIGH (spike code, well-tested)

### BC-011: Correlation engine uses sliding windows

**Preconditions:** Correlation rules loaded, events flowing
**Postconditions:** Events tracked in per-(rule_id, group_by_value) sliding windows. Expired entries evicted on each add. Fires when count meets threshold within window duration. Thread-safe via DashMap.
**Evidence:** `spike/crates/axiathon-detection/src/correlation.rs`
**Confidence:** MEDIUM (spike code, fewer tests)

### BC-012: Arrow schema derived from proto descriptors at runtime

**Preconditions:** OCSF class_uid provided
**Postconditions:** Schema generated with:
- 3 Axiathon columns (tenant_id, event_uid, event_time)
- Tier 1 hot columns (top-level scalars + one-level nested for hot objects)
- 1 event_data JSON column (tier 2)
- Total columns under 200 per class (Iceberg comfort zone)

**Hot nested objects:** src_endpoint, dst_endpoint, user, service, finding
**NOT flattened (too wide):** actor, metadata, device, session
**Evidence:** `spike/crates/axiathon-core/src/schema.rs` tests
**Confidence:** HIGH

---

## Pass 4: NFR Catalog

### Performance

| NFR | Value | Source |
|-----|-------|--------|
| Ingestion target | 100K+ EPS single-node | README |
| Query p50 target | <100ms on 1TB | README |
| Query p99 target | <500ms on 1TB | README |
| Detection latency target | <1s per event | README |
| Event-to-alert target | <5 seconds | README |
| Max query length | 64KB (CWE-400 mitigation) | parser.rs:36 |
| Max nesting depth | 128 (CWE-674 mitigation) | parser.rs:39 |
| Max pipe stages | 64 (CWE-400 mitigation) | parser.rs:42 |
| Max regex pattern | 1024 bytes (CWE-1333 mitigation) | parser.rs:570 |
| Default query timeout | 30s | QueryConfig |
| Max query timeout | 300s | QueryConfig |
| Max result rows | 10,000 | QueryConfig |
| Max concurrent queries | 50 | QueryConfig |
| Max memory per query | 512MB | QueryConfig |
| Storage buffer size | 1000 events | WriterConfig |
| Storage flush interval | 5 seconds | WriterConfig |
| Parquet compression | Zstd | writer.rs |

### Security

| NFR | Implementation | CWE/OWASP |
|-----|---------------|-----------|
| No unsafe code | `#![forbid(unsafe_code)]` in every crate | -- |
| Input validation at trust boundaries | Validated constructors (`new()` vs `new_unchecked()`) | CWE-20 |
| Query DoS prevention | Length, depth, stage count limits | CWE-400, OWASP A05:2021 |
| Regex DoS prevention | Pattern length limit + finite automaton engine | CWE-1333 |
| Integer overflow prevention | `i128` intermediate for i64::MIN parsing | CWE-190 |
| Error sanitization | Display impl for logging only; security comments on every error type | CWE-209 |
| Tenant isolation | TenantId newtype, TenantContext required parameter, private fields | -- |
| No eval/command injection | Shell rules forbid `eval`, use `"$@"` expansion | CWE-78 |
| CIDR validation at parse time | IP and prefix validated before execution | CWE-20 |

### Observability

- `tracing` crate for structured logging (planned: tenant_id + trace_id in all spans)
- `tracing-subscriber` with env-filter and JSON output
- Currently basic; SOUL.md marks this as target pattern to apply as crates grow

### Reliability

- Regex cache in detection engine (pre-compiled patterns)
- Hot-reload via `arc-swap` (planned for config, detection rules)
- DashMap for thread-safe correlation state
- Parquet buffer with time-based flush interval

---

## Pass 5: Convention and Pattern Catalog

### Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| Modules, functions, variables | snake_case | `parse_event`, `rule_id` |
| Types, traits, enums | PascalCase | `DetectionRule`, `AlertSeverity` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_BATCH_SIZE` |
| Proto packages | `ocsf.v1_7_0.events.{category}` | `ocsf.v1_7_0.events.network` |
| Detection rule files | kebab-case `.axd` | `brute-force.axd` |
| OCSF field paths | snake_case with dots | `src_endpoint.ip` |

### Design Patterns

#### 1. Validated Constructor / Newtype Pattern (PERVASIVE)

Every ID type and trust-boundary input uses:
- `new()` -- validates, returns Result (API input, deserialization)
- `new_unchecked()` -- skips validation (tests, database reads)
- Private inner field, public getters

Applied to: `TenantId`, `EventId`, `AlertId`, `FieldRef`

#### 2. Non-Exhaustive Enum Pattern (PERVASIVE)

All enums that may gain variants use `#[non_exhaustive]`:
- `AxiathonError`, `Value`, `CompareOp`, `StringOp`, `AxiQLStatement`, `FilterExpr`, `SelectExpr`, `Source`, `PipeStage`
- Exception: semantically closed enums like `SortDirection` (Asc/Desc)

#### 3. Dual-Context Pattern (CONSISTENT)

Two context types, one trait:
- `TenantContext` -- user operations (user_id, roles, permissions)
- `SystemContext` -- background jobs (system_component)
- `TenantScoped` trait -- `tenant_id()` + `trace_id()`, accepted by any function that needs either

#### 4. Case-Insensitive Keyword Parser Pattern (CONSISTENT)

```rust
fn kw<'a>(keyword: &'a str) -> Boxed<...> {
    text::ident().try_map(move |ident: &str, span| {
        if ident.eq_ignore_ascii_case(keyword) { Ok(()) }
        else { Err(Rich::custom(span, format!("expected keyword '{keyword}'"))) }
    }).boxed()
}
```

Used throughout the AxiQL parser. Replaces Chumsky's `text::keyword()` which is case-sensitive.

#### 5. Two-Tier Storage Pattern (spike)

- **Tier 1 (hot columns)**: Flat Parquet columns derived from proto descriptors. Configurable via `HOT_NESTED_OBJECTS` list. Supports predicate pushdown.
- **Tier 2 (event_data)**: JSON column with complete event. All fields accessible via `json_extract_string()`.

Column count kept under 200 per class (Iceberg comfort zone <300).

#### 6. Proto-Derived Schema Pattern (spike)

```
OCSF JSON export -> ocsf-proto-gen (Rust) -> .proto files -> prost-build -> Rust structs
                                                                         -> prost-reflect -> DynamicMessage
                                                                         -> Arrow schema
```

Runtime field access via `DynamicMessage.get_field_by_name()` + recursive path resolution. No per-class match blocks needed.

#### 7. Detection DSL Grammar Pattern (spike, Pest)

```
rule <id> {
  meta { name "..." severity high mitre "T1078" }
  match event where <condition>
  match count(event where <condition>) >= N group_by <fields> within <duration>
  match sequence by <field> within <duration> { step <name>: ... }
  alert { title "..." description "..." }
}
```

Three match types: SingleEvent, Correlation (count+threshold+window), Sequence (ordered steps).

#### 8. Comment-Stripping Preprocessor Pattern (AxiQL)

Comments (`//` and `#`) stripped before parsing by replacing comment characters with spaces (preserves byte offsets for error span reporting). Tracks string literal context to avoid false positives.

### Anti-Patterns and Technical Debt

1. **Public fields on AxiathonEvent (spike)** -- 78 call sites use struct literals. Marked with TODO for production migration to private fields + getters.
2. **Public tenant_id on TenantContext (spike)** -- 93 call sites access directly. Security-critical field that must be private.
3. **Duplicate Severity enums** -- `axiathon-core::ocsf::SeverityId` (0-6 OCSF) and `axiathon-detection::ast::Severity` (Info/Low/Medium/High/Critical) are semantically overlapping but structurally different.
4. **No ReDoS protection in detection DSL** -- spike detection engine compiles user regex without size limits. Marked with SECURITY(CWE-1333) comment. Production AxiQL parser does validate.
5. **Two different parser frameworks** -- Chumsky 0.10 for AxiQL (production), Pest for detection DSL (spike). Likely intentional: Chumsky supports error recovery, Pest is simpler for a declarative grammar.

---

## Pass 6: Synthesis

### Critical Design Decisions for Prism

#### 1. OCSF Event Modeling: DynamicMessage over Enum

The evolution from `OcsfEvent` enum (2 variants) to `AxiathonEvent` wrapping `DynamicMessage` is the key architectural insight. The enum approach requires a match arm per event class (83 in OCSF); the DynamicMessage approach scales to all classes with zero per-class code. Field access is by string path at runtime.

**Prism implication:** If Prism normalizes to OCSF, adopt the DynamicMessage pattern. Use `prost-reflect` for runtime field access. Build a custom proto generator from the OCSF JSON export (the `ocsf-proto-gen` tool is open source at `github.com/1898andCo/ocsf-proto-gen`).

#### 2. Two-Tier Columnar Storage

The decision to NOT use nested Parquet structs is grounded in measured reality: DataFusion's predicate pushdown for nested fields is partially broken (arrow-rs #5699, DataFusion #2581), and flat columns are 20-60x faster. Instead:

- Tier 1: Flat columns for hot fields (configurable, ~100-150 per class)
- Tier 2: `event_data` JSON column for complete events

This means Prism can:
- Use predicate pushdown on tier 1 fields for fast queries
- Access any OCSF field via `json_extract_string()` on tier 2
- Add new hot columns without data migration (Iceberg schema evolution)

**Prism implication:** Adopt two-tier storage. Use the `HOT_NESTED_OBJECTS` pattern to control which nested objects become flat columns.

#### 3. Three-Tier Field Alias Resolution

Analyst-friendly shortcuts map through AxiQL canonical names to OCSF canonical paths:

```
src_ip (analyst) -> src.ip (AxiQL) -> src_endpoint.ip (OCSF)
```

This decouples the query language from OCSF schema evolution. When a field moves between OCSF versions, only the alias registry updates -- queries don't change.

**Prism implication:** Build a similar alias layer. The `OcsfVersionAliasMap` provides version-conditional resolution for fields that moved between OCSF versions.

#### 4. Vendor Extension Pattern via Unmapped JSON

OCSF's `unmapped` field (a JSON string) is the escape hatch for vendor-specific data. The `get_field()` resolution chain:
1. Proto fields first (fast, typed)
2. Unmapped JSON fallback (slower, dynamic)

The detection engine can reference vendor fields in rules: `claroty.alert_type == "unauthorized_access"`. These are stored in the unmapped JSON and resolved at evaluation time.

**Prism implication:** Support vendor extensions via a JSON catch-all field. Parse and navigate it lazily. Consider promoting frequently-accessed vendor fields to tier 1 columns via the field promotion API (`events_to_record_batch_with_promotions`).

#### 5. Proto Generation Pipeline

```
OCSF schema (schema.ocsf.io/export/schema?version=1.7.0)
  -> ocsf-proto-gen generate (custom Rust tool)
  -> .proto files (one per category + shared objects.proto + enums.proto)
  -> prost-build + prost-reflect-build (build.rs)
  -> Rust types + DESCRIPTOR_POOL (runtime reflection)
```

Key learnings from the spike:
- The OCSF JSON export is a single 3.3MB file with all classes fully resolved (no inheritance to walk)
- `google.protobuf.Struct` must be replaced with `string` (prost Struct lacks serde support)
- Deprecated fields must be skipped entirely
- Sequential field numbering is deterministic and sufficient (the spike explored FNV-1a hash-based numbering for cross-version stability but settled on sequential)

#### 6. Detection DSL Design

The `.axd` grammar supports three progressively complex match types:

```
# Single event: immediate evaluation
match event where <condition>

# Correlation: sliding window count
match count(event where <condition>) >= N group_by <fields> within <duration>

# Sequence: ordered steps within window
match sequence by <key_field> within <duration> {
  step <name>: event where <condition>
  step <name>: count(event where <condition>) >= N
}
```

Operators: `==`, `!=`, `>`, `>=`, `<`, `<=`, `contains`, `matches` (regex), `cidr`, `in`
Boolean: `and`, `or`, `not`, parenthesized grouping
Dotted field paths: `user.name`, `src_endpoint.ip`, `claroty.alert_type`

**Prism implication:** If Prism needs a detection language, this grammar is a solid foundation. The three-tier match system (single/correlation/sequence) covers the standard SIEM use cases.

### Confidence Assessment

| Area | Confidence | Basis |
|------|------------|-------|
| OCSF event modeling | HIGH | Comprehensive spike with 100+ tests, production code matches |
| Protobuf schema design | HIGH | Custom generator built and validated, 67 prost-build compile tests |
| Query language (AxiQL) | HIGH | Full Chumsky parser with 1000+ LOC, extensive tests, doc examples |
| Data normalization | HIGH | Proto-backed events, DynamicMessage field access, vendor extension pattern |
| Two-tier storage | HIGH | Arrow schema generation tested, RecordBatch conversion proven |
| Field alias system | HIGH | Three-tier resolution with provenance tracking, version-aware aliases |
| Detection DSL | HIGH | Pest grammar, AST, engine, correlation, sequence -- all tested |
| Storage (Iceberg) | MEDIUM | Spike code with writer/reader/compaction, but fewer integration tests |
| API layer | LOW | Spike has routes but minimal testing |
| OT protocols | LOW | Plugin stubs only |

### Gaps and Risks

1. **No error recovery in AxiQL parser** -- marked as TODO(Story 5.2). Parser stops at first error rather than collecting multiple errors.
2. **No schema validation for cross-version queries** -- `CrossVersionProjection` is a placeholder struct.
3. **Detection DSL has no parse-time security limits** -- unlike AxiQL which has max query length, nesting depth, etc.
4. **Field catalog uses cycle detection via visited set** -- works but fragile for deeply circular OCSF references.
5. **Storage writer uses `RwLock` + `Mutex` + `mpsc`** -- complex concurrency model in spike, may have subtle issues.
6. **No migration path from spike to production** -- spike and production crates have duplicate types (TenantId, AxiathonError) with different validation rules.

### Recommendations for Prism

1. **Adopt the DynamicMessage pattern** for OCSF events -- it's the only approach that scales to 83+ event classes without per-class code.
2. **Use the two-tier storage architecture** -- flat Parquet hot columns + event_data JSON. This is validated against DataFusion's actual capabilities.
3. **Build a three-tier field alias registry** early -- it decouples query language from schema evolution.
4. **Reference `ocsf-proto-gen`** (MIT licensed) for proto generation, or adapt its approach.
5. **Copy the security hardening patterns** from the AxiQL parser: max length, max depth, max stages, regex validation at parse time, CIDR validation at parse time. These are all CWE-cited and well-justified.
6. **Use the validated constructor / newtype pattern** for all ID types -- it's pervasive in Axiathon and prevents entire categories of bugs.
7. **Study the detection DSL grammar** for correlation and sequence patterns -- the `count + group_by + within` and `sequence by + steps` patterns are well-designed for SIEM use cases.
