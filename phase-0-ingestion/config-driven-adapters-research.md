# Config-Driven Sensor Adapters, Multi-Endpoint Tables, and Hot Reloading

**Date:** 2026-04-13
**Type:** General (technology + architecture research)
**Status:** Complete
**Purpose:** Evaluate how Prism can make adding a new sensor require zero code changes -- just configuration files. Covers spec file design, multi-endpoint table orchestration, hot-reloading architecture, and the end-to-end zero-code flow.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [osquery's .table Spec Pattern -- Deep Analysis](#2-osquerys-table-spec-pattern--deep-analysis)
3. [Config-Driven Sensor Adapter Spec Design](#3-config-driven-sensor-adapter-spec-design)
4. [Complex Multi-Endpoint Table Definitions](#4-complex-multi-endpoint-table-definitions)
5. [Hot Reloading Architecture](#5-hot-reloading-architecture)
6. [Zero-Code Sensor Addition Flow](#6-zero-code-sensor-addition-flow)
7. [Existing Tools and Patterns](#7-existing-tools-and-patterns)
8. [Recommended Spec Format with Full Examples](#8-recommended-spec-format-with-full-examples)
9. [What Cannot Be Config-Driven](#9-what-cannot-be-config-driven)
10. [Technical Decisions and Recommendations](#10-technical-decisions-and-recommendations)
11. [Research Methods](#11-research-methods)

---

## 1. Executive Summary

**Recommendation: TOML sensor spec files interpreted at runtime, with an escape hatch to Rust plugin code for exotic cases.**

The analysis of osquery's codegen pattern, API gateway composition patterns, and Prism's specific requirements leads to these key conclusions:

1. **Runtime interpretation over codegen.** osquery uses Python-to-C++ codegen because it compiles to a native binary distributed to millions of endpoints -- build time cost is acceptable, runtime flexibility is not needed. Prism is a server-side tool used by an MSSP team of ~20 analysts. Runtime interpretation of sensor specs gives immediate "drop a file, reload, query" workflow without a build step.

2. **TOML is the right format.** Prism already standardized on TOML for all configuration (config-crate-research.md). TOML handles nested structures, inline tables, and arrays well enough for sensor specs. YAML's implicit typing and indentation sensitivity are unnecessary risks. A custom DSL is unjustified complexity.

3. **Multi-endpoint tables need a pipeline/step model.** The CrowdStrike QueryV2-then-PostEntities pattern and enrichment chains cannot be expressed as a single endpoint definition. A `[[table.steps]]` array with explicit data flow between steps (variable interpolation like `${step_name.field}`) handles sequential, fan-out, and conditional enrichment patterns.

4. **Hot reloading is feasible and valuable, but with clear boundaries.** Sensor spec files and credentials can be hot-reloaded. The MCP tool list (AxiQL table schema) can be refreshed via `notifications/tools/list_changed`. But HTTP client pools and auth state machines should NOT be hot-reloaded -- they require restart.

5. **OpenAPI as input is a nice-to-have, not a primary path.** Most security sensor APIs have non-standard quirks (CrowdStrike's two-step fetch, Claroty's POST-for-read, Armis's AQL) that OpenAPI specs don't capture. OpenAPI can seed a spec file template, but manual tuning will always be needed.

---

## 2. osquery's .table Spec Pattern -- Deep Analysis

### 2.1 How It Works (from local codebase analysis)

osquery's `.table` files are Python scripts executed at build time by `tools/codegen/gentable.py`. They use a declarative API defined in the codegen module:

```python
# From specs/example.table
table_name("example")
description("This is an example table spec.")
schema([
    Column("name", TEXT, "Description for name column"),
    Column("points", INTEGER, "This is a signed SQLite int column"),
    Column("action", TEXT, "Action performed in generation", required=True),
    Column("id", INTEGER, "An index of some sort", index=True),
    Column("path", TEXT, "Path of example", additional=True),
])
implementation("@genExample")
examples(["select * from example where id = 1"])
attributes(event_subscriber=False, utility=False)
```

**Key elements of the spec:**
- `table_name()` -- the virtual table name usable in SQL
- `schema()` -- column definitions with types (TEXT, INTEGER, BIGINT, DATETIME, DOUBLE, BLOB) and options (required, index, additional, optimized, hidden, collate)
- `extended_schema(platform, columns)` -- platform-conditional columns (e.g., Windows-only SID column on certificates table)
- `implementation("file@function")` -- points to the C++ implementation
- `attributes()` -- metadata: cacheable, event_subscriber, utility, user_data
- `examples()` -- sample queries for documentation and fuzz testing

### 2.2 The Codegen Pipeline

1. Build system (CMake) scans `specs/` for `.table` files
2. Each `.table` file is parsed by `gentable.py` (Python `ast.parse` + `exec`)
3. The `TableState` singleton collects all declarations
4. A Templite template (`default.cpp.in`) generates C++ code:
   - A `TablePlugin` subclass with `columns()` returning the schema
   - A `generate()` method that calls the implementation function
   - Plugin registration via `REGISTER()` macro
5. The generated C++ is compiled into the osquery binary

### 2.3 Column Options -- Direct Parallel to Prism

osquery's column options map directly to Prism's query engine needs:

| osquery Option | Meaning | Prism Equivalent |
|---------------|---------|-----------------|
| `required=True` | Column MUST appear in WHERE clause | Required filter parameter (e.g., CrowdStrike needs a time filter) |
| `index=True` | Hint that JOINing on this column improves performance | Column suitable for server-side filter push-down |
| `additional=True` | Column triggers additional/different data generation | Triggers enrichment endpoint call |
| `optimized=True` | Column is both index AND additional | Server-side filter that also enables data enrichment |
| `hidden=True` | Column exists but is hidden from `PRAGMA table_info` | Internal column not exposed in AxiQL schema |
| `collate` | Collation sequence (NOCASE, BINARY, etc.) | String comparison semantics for OCSF fields |

### 2.4 How osquery Handles Multi-API Tables

osquery tables that combine multiple OS API calls do NOT express this in the spec file. The spec defines only the output schema. The implementation function (C++ code) handles all the orchestration internally. For example:

- The `certificates` table calls different OS APIs per platform (Keychain on macOS, CryptoAPI on Windows, OpenSSL on Linux) but presents a unified schema
- The `curl` table takes a URL as a `required` column and performs the HTTP request in the implementation
- Event subscriber tables use a completely different execution model (push-based rather than pull)

**Key insight for Prism:** osquery's spec pattern separates schema definition (declarative) from data fetching (imperative code). For Prism's config-driven approach, we need to make the data fetching ALSO declarative -- which is the novel challenge.

### 2.5 osquery's Config Refresh Pattern

From `osquery/config/config.cpp`:

- `ConfigRefreshRunner` is an `InternalRunnable` (background thread) that calls `Config::refresh()` on an interval
- `Config::refresh()` calls the config plugin's `genConfig()` to fetch new config
- `Config::hashSource()` computes SHA1 of config content and returns `true` only if it changed
- On failure, uses accelerated retry interval (`config_accelerated_refresh`, default 300s)
- Supports config backup/restore via database persistence
- Protected by mutexes: `config_hash_mutex_`, `config_refresh_mutex_`, `config_backup_mutex_`
- Schedule modifications protected by `RecursiveMutex config_schedule_mutex_`

**Pattern summary:** Poll-based refresh, hash-based change detection, accelerated retry on failure, backup/restore for resilience. This maps well to Prism's needs.

---

## 3. Config-Driven Sensor Adapter Spec Design

### 3.1 What Goes in the Spec File

Based on analysis of all four sensor adapters (CrowdStrike, Cyberint, Claroty, Armis) from the recovered architecture:

| Category | Fields | Examples |
|----------|--------|---------|
| **Identity** | sensor_type, display_name, version | `sensor_type = "crowdstrike"`, `version = "1.0"` |
| **Connection** | base_url, timeout, user_agent | `base_url = "https://api.crowdstrike.com"` |
| **Authentication** | auth_type, token_url, scopes | `auth_type = "oauth2_client_credentials"` |
| **Rate Limiting** | requests_per_second, burst_size, retry_after_header | `requests_per_second = 10` |
| **Tables** (1..N) | table_name, description, ocsf_class | `table_name = "crowdstrike_alerts"` |
| **Per-Table Columns** | name, type, ocsf_mapping, options | `name = "severity"`, `ocsf_mapping = "severity_id"` |
| **Per-Table Steps** | method, path, body_template, pagination, response_path | See Section 4 |
| **Per-Table Merge Logic** | join_key, merge_strategy | `join_key = "device_id"` |

### 3.2 Authentication Types

From analyzing the four sensors:

| Auth Type | Sensors | Config Fields |
|-----------|---------|--------------|
| `oauth2_client_credentials` | CrowdStrike | `token_url`, `scopes[]` |
| `bearer_static` | Armis, Claroty | (none -- just a static API key) |
| `cookie_roundtrip` | Cyberint | `login_url`, `cookie_name` |

The spec file declares which auth type is needed. The actual credential values (client_id, client_secret, api_key) come from prism-credentials, NOT the spec file.

### 3.3 Pagination Types

From analyzing the four sensors:

| Pagination Type | Sensors | Config Fields |
|-----------------|---------|--------------|
| `offset_limit` | Claroty, Cyberint | `page_size`, `offset_param`, `limit_param` |
| `cursor_token` | CrowdStrike | `cursor_param`, `cursor_response_path` |
| `timestamp_cursor` | All four | `timestamp_field`, `id_field`, `sort_order` |
| `aql` | Armis | `aql_template`, `after_param` |

### 3.4 Code Generation vs Runtime Interpretation

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| **Build-time codegen** (osquery's approach) | Type safety at compile time; no runtime parsing overhead; compiler catches schema errors | Requires rebuild for any sensor change; cannot add sensors mid-session; build tooling complexity (proc macros or build.rs script) | Not recommended for Prism |
| **Runtime interpretation** (DSL interpreter) | Zero rebuild; hot-reloadable; analysts can experiment; new sensor in minutes | Runtime errors instead of compile-time; performance overhead (minimal -- spec parsing is not in hot path); schema validation must be explicit | Recommended for Prism |
| **Hybrid: codegen with runtime override** | Compiled fast path for known sensors; runtime path for new/experimental | Two code paths to maintain; complexity | Not recommended (over-engineering) |

**Decision rationale:** Prism serves ~20 MSSP analysts, not millions of endpoints. The deploy model is "server managed by security engineers," not "binary shipped to every host." Runtime interpretation gives the right trade-off: fast iteration over compile-time guarantees.

### 3.5 Format Choice: TOML

| Format | Pros | Cons |
|--------|------|------|
| **TOML** | Already Prism's standard; explicit types; good Rust ecosystem (`toml` crate); readable inline tables | Verbose for deeply nested structures; array-of-tables syntax (`[[]]`) is unusual |
| **YAML** | Compact; widely used for config; anchors/aliases for DRY | Implicit typing (Norway problem); indentation-sensitive; multiple parsers with subtly different behavior |
| **JSON** | Strict; universal parser support | No comments; verbose; poor human ergonomics |
| **Custom DSL** | Perfectly tailored to domain | Must build parser; learning curve for users; tooling gap (no syntax highlighting, no linter) |
| **JSON Schema + TOML** | Schema validation; tooling ecosystem | Complexity of maintaining both |

**Decision: TOML** with JSON Schema for validation (optional, not required for MVP). TOML matches existing Prism conventions and the `toml` crate already chosen in config-crate-research.md.

---

## 4. Complex Multi-Endpoint Table Definitions

### 4.1 Patterns Requiring Multiple API Calls

From analyzing the four sensors in recovered-architecture.md:

**Pattern 1: Sequential ID-then-Detail (CrowdStrike)**
```
POST /alerts/queries/alerts/v2 -> returns alert_ids[]
POST /alerts/entities/alerts/v1 body: {ids: alert_ids[]} -> returns alert details[]
```
The first call returns IDs only. The second call fetches full records by ID batch. This is the most common pattern in CrowdStrike's API.

**Pattern 2: Fan-out Enrichment (CrowdStrike alerts + host context)**
```
For each alert from Pattern 1:
  GET /devices/entities/devices/v2?ids={alert.device.device_id} -> host details
Merge: alert + host details on device_id
```

**Pattern 3: Conditional Enrichment (Claroty device + vulnerabilities)**
```
POST /assets/devices -> base device records
IF device.risk_level >= "high":
  POST /assets/device-vulnerability-relations?device_id={device.id} -> vulns
Merge: device + vulnerabilities on device_id
```

**Pattern 4: AQL Composition (Armis)**
```
GET /search?aql="type:device" -> devices
GET /search?aql="type:vulnerability AND deviceId:{device.id}" -> vulns per device
Merge: device + vulnerability on device.id
```

### 4.2 API Composition Patterns from Other Domains

**GraphQL Resolvers:** Each field in a GraphQL type can have its own resolver function. A `Device` type might resolve `hostname` from one API and `vulnerabilities` from another. The GraphQL execution engine handles batching (DataLoader pattern), parallel resolution, and error isolation per field. This is the closest architectural analog to Prism's multi-endpoint tables -- each "step" in a table spec is like a GraphQL resolver.

**API Gateway Composition (Kong, Tyk, AWS API Gateway):** API gateways support request-response transformation but generally NOT multi-step orchestration. They handle single upstream calls with response mapping. Multi-step orchestration requires a separate orchestration layer (Step Functions, BPMN).

**BFF (Backend-for-Frontend) Pattern:** A lightweight server-side layer that orchestrates multiple backend API calls into a single response. This is essentially what Prism's sensor adapter layer does -- it's a BFF for security sensor APIs.

**ETL Pipeline Composition (dbt, Airflow):** dbt models can reference other models (`ref('upstream_model')`), creating a DAG of transformations. This is analogous to multi-step table definitions where later steps reference earlier step outputs.

### 4.3 Proposed Config Schema for Multi-Endpoint Tables

The key design decision is how steps reference each other's outputs. Three options:

**Option A: Variable Interpolation (recommended)**
```toml
[[sensor.tables]]
name = "crowdstrike_alerts_enriched"
description = "CrowdStrike alerts with host enrichment"
ocsf_class = "security_finding"

[[sensor.tables.steps]]
name = "query_ids"
method = "POST"
path = "/alerts/queries/alerts/v2"
body_template = '{"filter": "${filter_expr}"}'
pagination = { type = "cursor_token", cursor_path = "meta.pagination.offset" }
response_path = "resources"
# Output: array of alert ID strings

[[sensor.tables.steps]]
name = "fetch_details"
method = "POST"
path = "/alerts/entities/alerts/v1"
body_template = '{"composite_ids": ${query_ids.results}}'
batch = { size = 500, source = "query_ids.results" }
response_path = "resources"
# Output: array of alert detail objects

[[sensor.tables.steps]]
name = "enrich_hosts"
method = "POST"
path = "/devices/entities/devices-v2"
body_template = '{"ids": ${fetch_details.results[*].device.device_id | unique}}'
batch = { size = 100, source = "fetch_details.results[*].device.device_id", deduplicate = true }
response_path = "resources"
merge = { into = "fetch_details", on = "device.device_id = device_id", strategy = "left_join" }

[sensor.tables.output]
primary_step = "fetch_details"
```

**Option B: Pipe-style (like Unix pipes)**
```toml
pipeline = "query_ids | batch_fetch(500) | enrich_hosts(device.device_id)"
```
Too terse -- hides too much complexity, hard to validate, impossible to express conditional enrichment.

**Option C: Explicit DAG with named edges**
```toml
[sensor.tables.dag]
nodes = ["query_ids", "fetch_details", "enrich_hosts"]
edges = [
  { from = "query_ids", to = "fetch_details", via = "results" },
  { from = "fetch_details", to = "enrich_hosts", via = "results[*].device.device_id" },
]
```
Over-engineered for the common case. Most tables are linear pipelines, not arbitrary DAGs.

**Recommended: Option A (Variable Interpolation)** because:
1. Each step is self-contained and readable
2. Variable references (`${step_name.field}`) are explicit about data flow
3. The `merge` directive on enrichment steps is clear about join semantics
4. Linear steps with optional merge handles 95% of cases
5. Easy to validate: parse variable references, check step names exist, verify field paths

### 4.4 Batch and Fan-out Configuration

The `batch` directive handles the common pattern where step N produces a list that step N+1 must process in chunks:

```toml
batch = { 
  size = 500,                              # Max items per batch request
  source = "query_ids.results",            # JSONPath to the list to batch over
  deduplicate = true,                      # Remove duplicate values before batching
  concurrency = 3,                         # Max parallel batch requests
}
```

This covers CrowdStrike's 500-ID batch limit, Armis's AQL page size, and Claroty's offset-limit pagination.

### 4.5 Conditional Steps

For patterns where enrichment is conditional:

```toml
[[sensor.tables.steps]]
name = "enrich_vulns"
method = "POST"
path = "/assets/device-vulnerability-relations"
condition = { field = "fetch_devices.results[*].risk_level", operator = "in", values = ["high", "critical"] }
# Only called for devices with high/critical risk
```

### 4.6 Response Parsing

Security APIs return responses in varied structures. The spec must handle:

```toml
[sensor.tables.steps.response]
path = "resources"                          # JSONPath to the data array
total_path = "meta.pagination.total"        # JSONPath to total count (for pagination)
error_path = "errors"                       # JSONPath to error array
timestamp_format = "rfc3339"                # or "epoch_seconds", "epoch_millis", "custom:%Y-%m-%d"
```

---

## 5. Hot Reloading Architecture

### 5.1 What the Product Brief Says

The config-crate-research.md explicitly states: "R12: No hot-reload needed (Prism is per-analyst stdio MCP, restarted per session)." However, the product brief also describes a per-analyst MCP server running continuously for a session. If sensor specs are config-driven AND hot-reloadable, an analyst could ask their AI agent to add a new sensor type mid-session -- a compelling workflow.

### 5.2 What Can vs Cannot Be Hot-Reloaded

| Category | Hot-Reloadable? | Rationale |
|----------|----------------|-----------|
| Sensor spec files (new tables) | YES | Schema-only change; no TCP connections affected |
| Sensor spec files (modified tables) | YES with caveats | In-flight queries must complete with old schema; new queries use new schema |
| Credentials | YES | Credential store is already abstracted behind a trait; new creds just work |
| Feature flags | YES | BTreeMap swap is atomic |
| Client config (new client) | YES | No existing state to invalidate |
| Client config (modified client) | YES with caveats | Must flush differential state for changed queries |
| HTTP client pools | NO | Connection pools, TLS state, OAuth token caches are expensive to rebuild |
| Auth state machines | NO | OAuth2 token lifecycle, cookie sessions -- too stateful |
| RocksDB column families | NO | Schema migration requires restart |
| Detection rules | YES | Rules are already designed for runtime creation via MCP tool |
| Query aliases | YES | Stateless mapping; already supports runtime CRUD |

### 5.3 Rust Patterns for Concurrent Config Access

**Option 1: `Arc<RwLock<Config>>`**
```rust
// Simple but has read contention under high load
let config = Arc::new(RwLock::new(load_config()?));

// Reader (hot path -- every query)
let cfg = config.read().await;
let table = cfg.get_table("crowdstrike_alerts")?;

// Writer (cold path -- reload)
let mut cfg = config.write().await;
*cfg = load_config()?;
```
Pros: Simple, well-understood. Cons: Write lock blocks all readers; potential priority inversion.

**Option 2: `arc-swap` (ArcSwap)**
```rust
// Lock-free reads, atomic pointer swap on write
let config = ArcSwap::from_pointee(load_config()?);

// Reader (hot path) -- no lock, just atomic load
let cfg = config.load();
let table = cfg.get_table("crowdstrike_alerts")?;

// Writer (cold path) -- atomic swap, old config dropped when last reader finishes
config.store(Arc::new(load_config()?));
```
Pros: Lock-free reads; writers never block readers; old config naturally garbage-collected when last reference drops. Cons: Extra dependency (`arc-swap` crate); double memory during transition.

**Option 3: `tokio::sync::watch` channel**
```rust
// Sender held by reload task, receivers cloned to every handler
let (tx, rx) = watch::channel(Arc::new(load_config()?));

// Reader (hot path)
let cfg = rx.borrow().clone(); // Arc clone is cheap

// Writer (cold path)
tx.send(Arc::new(load_config()?))?;
// All receivers see new value on next borrow()
```
Pros: Idiomatic tokio; receivers can `.changed().await` to react to updates; no extra crate. Cons: Slight overhead from channel machinery; `.borrow()` holds a read guard briefly.

**Recommendation: `arc-swap` for sensor spec registry, `tokio::sync::watch` for notification of changes.** The sensor spec registry is read on every query (hot path) -- `arc-swap` gives true lock-free reads. The watch channel is used to notify the MCP layer that it should send `notifications/tools/list_changed` when the tool list changes.

### 5.4 The `reload_config` MCP Tool

Proposed tool design:

```rust
/// Reload sensor spec files and client configuration from disk.
/// Returns a summary of what changed.
#[tool(name = "reload_config")]
async fn reload_config(&self, params: ReloadConfigInput) -> Result<ReloadConfigOutput> {
    // 1. Scan sensor spec directory for .toml files
    // 2. Parse and validate all specs
    // 3. Compare with current spec registry (hash-based, like osquery)
    // 4. If specs changed:
    //    a. Atomic swap of spec registry via ArcSwap
    //    b. Send notifications/tools/list_changed if table list changed
    //    c. Flush differential state for modified tables
    // 5. If credentials changed: refresh credential cache
    // 6. Return: { added_tables: [...], removed_tables: [...], modified_tables: [...], errors: [...] }
}
```

Output schema:
```json
{
  "added_tables": ["new_sensor_alerts"],
  "removed_tables": [],
  "modified_tables": ["crowdstrike_alerts"],
  "credential_changes": ["new_sensor: api_key added"],
  "errors": [],
  "previous_hash": "abc123",
  "current_hash": "def456"
}
```

### 5.5 osquery-Inspired Change Detection

Adapting osquery's `ConfigRefreshRunner` + `hashSource()` pattern:

```rust
struct SpecRegistry {
    specs: HashMap<String, SensorSpec>,
    file_hashes: HashMap<PathBuf, String>,  // SHA256 per spec file
    composite_hash: String,                   // Hash of all hashes
}

impl SpecRegistry {
    fn needs_reload(&self, spec_dir: &Path) -> bool {
        // Scan directory, hash each file, compare with stored hashes
        // Returns true if any file changed, added, or removed
    }
    
    fn reload(&mut self, spec_dir: &Path) -> ReloadResult {
        // Parse all spec files
        // Validate schemas (multi-error, like config-crate-research.md pattern)
        // Compute diff (added/removed/modified tables)
        // Update hashes
    }
}
```

### 5.6 MCP Tool List Refresh

When sensor specs change, the AxiQL schema changes (new tables, removed tables, modified columns). The MCP protocol supports `notifications/tools/list_changed` to inform the client that the tool list has been updated. The flow:

1. `reload_config` tool is called (or background file watcher triggers)
2. Spec registry is atomically swapped
3. If the set of table names or their column schemas changed:
   - Send `notifications/tools/list_changed` via MCP notification
   - The AI agent (Claude Code) will re-fetch `tools/list` to see the updated schema
4. The `query` tool's `outputSchema` dynamically reflects the current spec registry

Note: The `query` and `explain_query` tools themselves don't change -- only their available table schemas change. So `tools/list_changed` is about updating the AI agent's understanding of what tables/columns are queryable.

---

## 6. Zero-Code Sensor Addition Flow

### 6.1 End-to-End Workflow

```
Step 1: Security engineer creates sensors/acme_sensor.toml
        (copies from a template, fills in endpoints/columns)

Step 2: Adds to client config:
        [clients.acme.sensors.acme_sensor]
        enabled = true

Step 3: Sets credentials:
        prism set-credential --client acme --sensor acme_sensor --name api_key --value "..."
        (or via MCP tool: set_credential)

Step 4: Triggers reload:
        Via MCP tool: reload_config
        Or: prism validates on next query attempt

Step 5: New tables appear:
        explain_query("FROM acme_sensor_alerts") -- works immediately
        query("SELECT * FROM acme_sensor_alerts WHERE severity >= 'high'")
```

### 6.2 Time Estimate

| Step | Time | Who |
|------|------|-----|
| Copy template spec file | 1 min | Security engineer |
| Fill in base_url, endpoints, columns | 15-30 min | Security engineer (reading API docs) |
| Map columns to OCSF | 10-20 min | Security engineer (OCSF mapping guide) |
| Add client config entry | 1 min | Security engineer |
| Set credentials | 1 min | Security engineer |
| Reload and test | 2 min | Analyst via AI agent |
| **Total** | **30-55 min** | -- |

Compare to the current approach (new Rust adapter): 2-5 days of Rust development + code review + CI + deploy.

### 6.3 Challenges and Limitations

**Challenge 1: Custom authentication flows**
Some APIs use non-standard auth (HMAC signatures, mutual TLS, custom challenge-response). These cannot be expressed in config.

*Mitigation:* Support a finite set of auth plugins (oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key_header, api_key_query, basic_auth). If an API uses something exotic, it requires a Rust auth plugin -- but the table/column/endpoint definitions are still config-driven. Estimate: auth plugins cover 95% of security vendor APIs.

**Challenge 2: Non-JSON response formats**
Some APIs return XML, CSV, protobuf, or binary data.

*Mitigation:* The spec file declares a `response_format` field. Support JSON (default), JSON Lines, CSV, and XML (via `quick-xml` or `serde_xml_rs`). Binary protocols require Rust code.

**Challenge 3: Complex response parsing**
Some APIs embed data in deeply nested structures, use polymorphic fields (Claroty's string-or-number IDs), or require field transformation (Cyberint's multi-format timestamps).

*Mitigation:* The spec supports JSONPath for `response_path`, plus simple transforms:
```toml
[[sensor.tables.columns]]
name = "device_id"
source_path = "device.device_id"
type = "text"
# Polymorphic ID: could be string or number in API response
coerce = "to_string"
```

For truly exotic parsing (e.g., Cyberint's 4-format timestamp parsing), the spec supports a `transform` field that references a named transform function registered in Rust:
```toml
[[sensor.tables.columns]]
name = "created_at"
source_path = "created_date"
type = "datetime"
transform = "cyberint_timestamp"  # References a registered Rust function
```

This is the escape hatch: the spec is config-driven, but individual field transforms can be Rust code.

**Challenge 4: Rate limit nuances**
Different endpoints on the same API may have different rate limits. Global vs per-endpoint vs per-tenant limits.

*Mitigation:*
```toml
[sensor.rate_limits]
global_rps = 10

[sensor.rate_limits.per_endpoint]
"/alerts/queries/alerts/v2" = { rps = 5, burst = 10 }
"/devices/entities/devices-v2" = { rps = 20, burst = 50 }
```

**Challenge 5: Stateful protocols (WebSocket, gRPC streaming)**
Config-driven specs assume request-response HTTP. Streaming protocols need different abstractions.

*Mitigation:* Out of scope for v1. All four current sensors use REST. If a streaming sensor is needed, it requires a Rust adapter.

---

## 7. Existing Tools and Patterns

### 7.1 Grafana Data Source Provisioning

Grafana allows adding data sources via YAML files in `/etc/grafana/provisioning/datasources/`. Key elements of the pattern:

```yaml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
    access: proxy
    isDefault: true
    jsonData:
      httpMethod: POST
      keepCookies: []
    secureJsonData:
      httpHeaderValue1: "Bearer ${PROMETHEUS_TOKEN}"
```

**What Prism can learn:**
- Separation of `jsonData` (non-sensitive config) from `secureJsonData` (credentials) -- Prism already does this with prism-credentials
- `type` field selecting a pre-built plugin -- Prism's equivalent is the sensor spec file itself
- Hot-reload: Grafana watches the provisioning directory and reloads on file change
- `access: proxy` vs `access: direct` -- Prism is always "proxy" (server-side API calls)

**Key difference:** Grafana data source plugins contain code (Go/TypeScript). The YAML just configures an existing plugin. Prism's vision is more ambitious: the TOML spec file DEFINES the plugin behavior, not just configures it.

### 7.2 Terraform Provider Schemas

Terraform providers define resource schemas declaratively:

```go
func resourceExample() *schema.Resource {
    return &schema.Resource{
        Schema: map[string]*schema.Schema{
            "name": {
                Type:     schema.TypeString,
                Required: true,
            },
            "size": {
                Type:     schema.TypeInt,
                Optional: true,
                Default:  20,
            },
        },
        Create: resourceExampleCreate,
        Read:   resourceExampleRead,
    }
}
```

**What Prism can learn:**
- Schema definition is separate from CRUD operations
- `Required`/`Optional`/`Computed` flags -- maps to osquery's `required`/`additional`/`index`
- `ForceNew` -- a change to this field requires resource recreation (analogous to "requires restart" in hot-reload context)
- Providers are compiled Go code, NOT config files -- Terraform chose codegen over interpretation

**Key difference:** Terraform providers handle bidirectional state management (create, read, update, delete). Prism sensor specs are read-only (data fetching only). Write operations remain per-sensor MCP tools in Rust code.

### 7.3 Singer Taps Specification

The Singer specification defines a standard interface for data extraction. A "tap" (data source) produces a stream of SCHEMA, RECORD, and STATE messages on stdout:

```json
{"type": "SCHEMA", "stream": "users", "schema": {"properties": {"id": {"type": "integer"}, "name": {"type": "string"}}}, "key_properties": ["id"]}
{"type": "RECORD", "stream": "users", "record": {"id": 1, "name": "Alice"}}
{"type": "STATE", "value": {"users": {"last_id": 1}}}
```

Taps are configured via a JSON config file:
```json
{
  "api_key": "...",
  "start_date": "2024-01-01",
  "base_url": "https://api.example.com"
}
```

And a catalog file defines which streams to sync and how:
```json
{
  "streams": [{
    "stream": "users",
    "tap_stream_id": "users",
    "schema": {...},
    "metadata": [{"breadcrumb": [], "metadata": {"selected": true, "replication-method": "INCREMENTAL", "replication-key": "updated_at"}}]
  }]
}
```

**What Prism can learn:**
- **Catalog as schema declaration** -- The Singer catalog is the closest analog to Prism's sensor spec. It declares streams (tables), their schemas, and replication method.
- **State management for incremental sync** -- Singer's STATE messages track cursor position. Prism's differential engine serves a similar purpose.
- **Config vs catalog separation** -- Credentials/connection info (config.json) is separate from schema/behavior (catalog.json). Prism should maintain this separation.

**Key difference:** Singer taps are executable programs (Python scripts), not config files. Each tap is a separate codebase. Prism's vision eliminates per-sensor code entirely.

### 7.4 n8n / Zapier Connector Definitions

Workflow automation tools define API connectors declaratively (primarily for their node/step builders):

n8n nodes declare:
- Authentication method and credentials schema
- Available operations (GET alerts, POST device action)
- Per-operation: HTTP method, URL template, query/body parameters
- Response parsing and field mapping

**What Prism can learn:**
- n8n's credential types are reusable across nodes -- one OAuth2 credential type serves many API nodes. Prism should similarly decouple auth type definitions from sensor specs.
- Operation chaining: n8n workflows chain operations. Each operation's output feeds the next. This maps to Prism's multi-step table pipeline.
- Expression syntax: n8n uses `{{ $json.field }}` for variable interpolation. Prism needs similar syntax for inter-step references.

---

## 8. Recommended Spec Format with Full Examples

### 8.1 Complete CrowdStrike Alerts Spec

```toml
# sensors/crowdstrike.toml
# Sensor spec for CrowdStrike Falcon

[sensor]
type = "crowdstrike"
display_name = "CrowdStrike Falcon"
version = "1.0"
base_url = "https://api.crowdstrike.com"
# base_url can be overridden per-client for multi-region (us-1, us-2, eu-1, ap-1)

[sensor.auth]
type = "oauth2_client_credentials"
token_url = "/oauth2/token"
# Credential names reference prism-credentials store:
#   client_id = credential("crowdstrike", "client_id")
#   client_secret = credential("crowdstrike", "client_secret")

[sensor.defaults]
timeout_seconds = 30
user_agent = "prism/${prism_version}"

[sensor.rate_limits]
global_rps = 10
burst = 20

# ---------- Table: crowdstrike_alerts ----------

[[sensor.tables]]
name = "crowdstrike_alerts"
description = "CrowdStrike Falcon alerts with optional host enrichment."
ocsf_class = "security_finding"

# Step 1: Query for alert IDs
[[sensor.tables.steps]]
name = "query_ids"
method = "POST"
path = "/alerts/queries/alerts/v2"
headers = { "Content-Type" = "application/json" }
body_template = '''
{
  "filter": "${where_clause}",
  "sort": "created_timestamp|desc",
  "limit": 500
}
'''

[sensor.tables.steps.pagination]
type = "offset"
offset_param = "offset"
limit_param = "limit"
total_path = "meta.pagination.total"

[sensor.tables.steps.response]
data_path = "resources"
# resources is an array of alert composite ID strings

# Step 2: Fetch full alert details by ID batch
[[sensor.tables.steps]]
name = "fetch_details"
method = "POST"
path = "/alerts/entities/alerts/v1"
body_template = '{"composite_ids": ${query_ids.results}}'

[sensor.tables.steps.batch]
source = "query_ids.results"
size = 500

[sensor.tables.steps.response]
data_path = "resources"

# Step 3 (optional): Enrich with host details
[[sensor.tables.steps]]
name = "enrich_hosts"
method = "POST"
path = "/devices/entities/devices-v2"
body_template = '{"ids": ${fetch_details.results[*].device.device_id | unique}}'
optional = true  # Table works without this step; enrichment adds columns

[sensor.tables.steps.batch]
source = "fetch_details.results[*].device.device_id"
size = 100
deduplicate = true
concurrency = 3

[sensor.tables.steps.response]
data_path = "resources"

[sensor.tables.steps.merge]
into = "fetch_details"
on = "device.device_id = device_id"
strategy = "left_join"

# Output schema
[sensor.tables.output]
primary_step = "fetch_details"

[[sensor.tables.columns]]
name = "alert_id"
source_path = "composite_id"
type = "text"
options = ["required"]
ocsf_mapping = "finding_info.uid"
description = "CrowdStrike composite alert ID"

[[sensor.tables.columns]]
name = "severity"
source_path = "severity"
type = "integer"
ocsf_mapping = "severity_id"
description = "Alert severity (0-5)"

[[sensor.tables.columns]]
name = "created_timestamp"
source_path = "created_timestamp"
type = "datetime"
ocsf_mapping = "time"
description = "Alert creation time"
options = ["index"]

[[sensor.tables.columns]]
name = "hostname"
source_path = "device.hostname"
type = "text"
ocsf_mapping = "device.hostname"
description = "Host where alert originated"

[[sensor.tables.columns]]
name = "tactic"
source_path = "tactic"
type = "text"
ocsf_mapping = "attacks[0].tactic.name"
description = "MITRE ATT&CK tactic"

[[sensor.tables.columns]]
name = "technique"
source_path = "technique"
type = "text"
ocsf_mapping = "attacks[0].technique.name"
description = "MITRE ATT&CK technique"

[[sensor.tables.columns]]
name = "host_os"
source_path = "_enriched.enrich_hosts.os_product_name"
type = "text"
ocsf_mapping = "device.os.name"
description = "Host operating system (from enrichment)"

[[sensor.tables.columns]]
name = "host_external_ip"
source_path = "_enriched.enrich_hosts.external_ip"
type = "text"
ocsf_mapping = "device.ip"
description = "Host external IP (from enrichment)"
```

### 8.2 Minimal Single-Endpoint Spec (Armis Alerts)

```toml
# sensors/armis.toml

[sensor]
type = "armis"
display_name = "Armis Centrix"
version = "1.0"
base_url = "https://${instance}.armis.com/api/v1"

[sensor.auth]
type = "bearer_static"
# api_key = credential("armis", "api_key")

[sensor.rate_limits]
global_rps = 5

[[sensor.tables]]
name = "armis_alerts"
description = "Armis security alerts."
ocsf_class = "security_finding"

[[sensor.tables.steps]]
name = "fetch"
method = "GET"
path = "/search/"
query_params = { aql = 'in:alerts timeFrame:"${time_range}"', from = "${offset}", length = "1000" }

[sensor.tables.steps.pagination]
type = "offset_limit"
offset_param = "from"
limit_param = "length"
total_path = "data.total"

[sensor.tables.steps.response]
data_path = "data.data"

[sensor.tables.output]
primary_step = "fetch"

[[sensor.tables.columns]]
name = "alert_id"
source_path = "alertId"
type = "integer"
options = ["required"]
ocsf_mapping = "finding_info.uid"

[[sensor.tables.columns]]
name = "title"
source_path = "title"
type = "text"
ocsf_mapping = "finding_info.title"

[[sensor.tables.columns]]
name = "severity"
source_path = "severity"
type = "text"
ocsf_mapping = "severity"

[[sensor.tables.columns]]
name = "time"
source_path = "time"
type = "datetime"
ocsf_mapping = "time"
options = ["index"]
```

---

## 9. What Cannot Be Config-Driven

| Capability | Why Not Config-Driven | Escape Hatch |
|-----------|----------------------|-------------|
| Custom auth flows (HMAC, mutual TLS, custom challenge-response) | Require procedural logic, cryptographic operations | Rust auth plugin trait |
| Binary protocol parsing (gRPC, protobuf, MessagePack) | Need compiled deserializers | Rust response parser plugin |
| Stateful protocols (WebSocket streams, long-poll) | Request-response model assumption | Rust streaming adapter |
| Complex field transformations (Cyberint 4-format timestamp, Claroty polymorphic ID coercion) | Logic too complex for declarative config | Named transform functions registered in Rust |
| Write operations (containment, blocking, alert acknowledgment) | Different per sensor, require confirmation flows, risk classification | Rust MCP tool implementations (existing pattern) |
| OAuth2 token lifecycle management | Token caching, refresh logic, multi-region | Rust auth module (shared across sensors of same type) |
| Custom pagination with side effects | Some APIs require acknowledging pages or maintaining server-side cursors | Rust pagination plugin |

**Estimated coverage:** Config-driven specs handle ~80% of read operations for typical REST API security sensors. The remaining ~20% require Rust escape hatches, but even those sensors benefit from config-driven column/schema definitions.

---

## 10. Technical Decisions and Recommendations

### 10.1 Summary of Recommendations

| Decision | Recommendation | Confidence |
|----------|---------------|------------|
| Spec format | TOML | HIGH |
| Runtime vs codegen | Runtime interpretation | HIGH |
| Multi-step model | Variable interpolation (`${step.field}`) | HIGH |
| Hot reload mechanism | `arc-swap` for spec registry + `tokio::sync::watch` for notifications | MEDIUM (need to verify `arc-swap` current version) |
| Auth in spec vs code | Finite set of auth plugins (code), selected by type field in spec | HIGH |
| OCSF mapping | Per-column `ocsf_mapping` field in spec | HIGH |
| Escape hatch | Named Rust functions for transforms and custom auth | HIGH |
| MCP tool list refresh | `notifications/tools/list_changed` after spec reload | HIGH |
| OpenAPI as input | Nice-to-have template seeder, not primary path | MEDIUM |
| Spec file location | `sensors/` directory, one file per sensor type | HIGH |

### 10.2 Implementation Priority

1. **Phase 1 (MVP):** Single-step tables with simple pagination. Support bearer_static and oauth2_client_credentials auth. TOML spec files loaded at startup.
2. **Phase 2:** Multi-step tables with variable interpolation. Batch processing. Fan-out enrichment.
3. **Phase 3:** Hot-reload via `reload_config` MCP tool. `arc-swap` spec registry. `notifications/tools/list_changed`.
4. **Phase 4:** Conditional steps. Named Rust transform functions. OpenAPI template seeder.

### 10.3 Spec Validation

Every sensor spec file must be validated at load time with multi-error reporting:

```
Validation checks:
- All referenced steps exist (no dangling ${step_name.field} references)
- No circular step dependencies
- primary_step exists in steps list
- Column types are valid (text, integer, bigint, double, datetime, boolean)
- OCSF mappings reference valid OCSF paths (validated against ocsf-proto-gen output)
- Required columns have corresponding WHERE clause filter push-down configured
- Rate limits are positive numbers
- Auth type is one of the supported types
- Pagination type is one of the supported types
- Batch sizes are within bounds (1-10000)
```

### 10.4 Relationship to Existing Architecture

The config-driven sensor spec adds a new concept to the architecture but fits cleanly:

```
prism-config (loads sensor specs from TOML)
  -> SpecRegistry (Arc-swappable collection of SensorSpec)
    -> prism-sensors (generic DataSource impl interprets specs at runtime)
      -> prism-ocsf (column ocsf_mapping drives normalization)
        -> prism-query (table schemas from spec drive AxiQL schema)
```

The existing `SensorAdapter` and `DataSource<T>` traits remain. A new `ConfigDrivenAdapter` implements `SensorAdapter` by interpreting a `SensorSpec` at runtime, replacing the per-sensor `CrowdStrikeAdapter`, `ClarotyAdapter`, etc. for sensors that fit the config-driven model. Sensors with exotic requirements keep their Rust adapter implementations.

---

## 11. Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| WebSearch | 0 (denied) | Was attempted for Singer spec, Grafana provisioning, Rust hot reload patterns |
| WebFetch | 0 (denied) | Was attempted for Singer spec page, Grafana docs |
| Context7 | 0 (denied) | Was attempted for figment and tokio watch channel docs |
| Local file reads | 14 | osquery .table specs (example, curl, certificates), osquery codegen (gentable.py, default.cpp.in), osquery config refresh (config.h, config.cpp), product-brief.md, recovered-architecture.md, config-crate-research.md, phase-0 ingestion files |
| Local grep searches | 6 | ConfigRefreshRunner implementation, DataSource trait, multi-API table patterns |
| Training data | 6 areas | Singer tap specification format, Grafana data source provisioning YAML structure, Terraform provider schema patterns, n8n/Zapier connector definition patterns, Rust arc-swap crate API, API gateway composition patterns (GraphQL resolvers, BFF) |

**Total MCP tool calls:** 0 (all external research tools denied)
**Training data reliance:** HIGH -- Singer, Grafana, Terraform, n8n/Zapier patterns, and `arc-swap` crate details are from training data (knowledge cutoff May 2025). The core osquery analysis is from primary sources (local codebase). The spec design recommendations are derived from first-principles analysis of the four sensor adapters documented in recovered-architecture.md.

### Verification Needed Before Finalizing

| Item | How to Verify | Impact |
|------|--------------|--------|
| `arc-swap` current version and API | `cargo search arc-swap` or crates.io | LOW -- API is stable and well-known |
| Singer specification current status | singer.io or github.com/singer-io/getting-started | LOW -- used for pattern reference only |
| Grafana provisioning YAML current format | grafana.com/docs | LOW -- used for pattern reference only |
| `tokio::sync::watch` current API | tokio.rs docs | LOW -- core tokio, unlikely to have changed |
| MCP `notifications/tools/list_changed` spec | MCP specification | MEDIUM -- must verify this is the correct notification name |
