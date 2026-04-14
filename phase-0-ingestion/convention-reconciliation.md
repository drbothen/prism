# Convention Reconciliation -- Prism Multi-Repo Phase 0

**Date:** 2026-04-13
**Input:** Pass-8 synthesis and Pass-5 convention files for 9 reference repos
**Purpose:** Resolve convention conflicts across Go, TypeScript, and Rust codebases; recommend Rust-idiomatic resolutions for Prism

---

## Repo Classification

| Repo | Language | Type | Role in Prism |
|------|----------|------|---------------|
| poller-bear | Go | Claroty xDome poller | Sensor adapter reference (9 data sources) |
| poller-coaster | Go | Armis Centrix poller | Sensor adapter reference (7 data sources) |
| poller-cobra | Go | CrowdStrike Falcon poller | Sensor adapter reference (alerts + stubs) |
| poller-express | Go | Cyberint Argos poller | Sensor adapter reference (alerts + assets) |
| mcp-claroty-xdome | TypeScript | MCP server for Claroty | MCP server pattern reference |
| serveMyAPI | TypeScript | MCP server for credentials | Credential management reference |
| axiathon | Rust | SIEM / security lake | OCSF normalization + storage reference |
| ocsf-proto-gen | Rust | Proto codegen tool | OCSF type mapping reference |
| tally | Rust | Findings tracker MCP server | Rust MCP server pattern reference |

---

## 1. Naming Conventions

### What Each Repo Does

**Go pollers (4 repos):**
- Types: `PascalCase` (Go convention) -- `AlertCollector`, `HTTPSender`, `DevicePollState`
- Functions: `PascalCase` (exported), `camelCase` (unexported) -- `NewHTTPClient`, `collectOnce`, `filterNewAlerts`
- Variables: `camelCase` -- `alertSink`, `healthServer`
- Constants: `camelCase` (unexported), `SCREAMING_SNAKE` (env var string literals)
- JSON wire format: `snake_case` tags -- `record_type`, `cluster_name`
- Files: `snake_case.go` -- `alert_collector.go`, `http_sender.go`
- Packages: lowercase single-word -- `collector`, `config`, `sink`, `state`
- Env vars: `SCREAMING_SNAKE_CASE` with domain prefixes -- `CLAROTY_API_TOKEN`, `VECTOR_ENDPOINT`
- Test functions: `Test<Type>_<Scenario>` -- `TestCollectAlerts_ClarotyClientError`
- Sentinel errors: `Err<Component><Condition>` -- `ErrStateNotFound`, `ErrCursorRegression`

**TypeScript MCP servers (2 repos):**
- Types/Classes: `PascalCase` -- `AlertService`, `KeychainService`, `GetAlertsToolHandler`
- Functions/methods: `camelCase` -- `storeKey`, `findAlerts`
- Variables: `camelCase` -- `baseUrl`, `apiToken`
- Constants: `SCREAMING_SNAKE_CASE` -- `SERVICE_NAME`, `PERMISSION_MARKER`
- MCP tool names: `snake_case` (mcp-claroty-xdome: `get_alerts`) or `kebab-case` (serveMyAPI: `store-api-key`)
- Files: `kebab-case.ts` -- `alert-service.ts`, `get-alerts-handler.ts`
- JSON wire format: `camelCase` properties (native JS serialization)

**Rust tools (3 repos):**
- Types: `PascalCase` -- `Finding`, `AxiathonEvent`, `OcsfSchema`, `TallyMcpServer`
- Functions: `snake_case` -- `compute_fingerprint`, `resolve_object_graph`, `handle_record`
- Variables: `snake_case` -- `version_slug`, `field_num`
- Constants: `SCREAMING_SNAKE_CASE` -- `MAX_QUERY_LENGTH`, `FINDINGS_DIR`
- Modules: `snake_case` -- `type_map`, `codegen`, `git_store`
- Files: `snake_case.rs` -- `finding.rs`, `state_machine.rs`
- Crate names: `kebab-case` -- `axiathon-core`, `tally-ng`
- Test functions: `snake_case` descriptive -- `tenant_id_new_rejects_empty`, `empty_object_type_emits_string`
- MCP tool names: `snake_case` -- `record_finding`, `query_findings`, `update_status`

### Conflicts

| Element | Go | TypeScript | Rust |
|---------|-----|-----------|------|
| Type names | PascalCase | PascalCase | PascalCase |
| Function names | camelCase/PascalCase | camelCase | snake_case |
| Variable names | camelCase | camelCase | snake_case |
| File names | snake_case.go | kebab-case.ts | snake_case.rs |
| MCP tool names | N/A | snake_case or kebab-case | snake_case |
| JSON wire format | snake_case (via tags) | camelCase (native) | snake_case (via serde) |
| Env var constants | string literals in code | SCREAMING_SNAKE | SCREAMING_SNAKE |

### Recommended Resolution for Prism

| Element | Convention | Rationale |
|---------|-----------|-----------|
| Type names | `PascalCase` | Rust convention. Universal across all 9 repos. |
| Function names | `snake_case` | Rust convention. Enforced by compiler warning. |
| Variable names | `snake_case` | Rust convention. Enforced by compiler warning. |
| File names | `snake_case.rs` | Rust convention. Matches axiathon, ocsf-proto-gen, tally. |
| Crate names | `kebab-case` | Rust convention. Matches axiathon (`axiathon-core`), tally (`tally-ng`). |
| Module names | `snake_case` | Rust convention. |
| Constants | `SCREAMING_SNAKE_CASE` | Universal across all 9 repos. |
| MCP tool names | `snake_case` | Matches mcp-claroty-xdome (`get_alerts`) and tally (`record_finding`). MCP spec does not mandate a convention; snake_case is Rust-natural and AI-agent-friendly. |
| JSON wire format | `snake_case` via `#[serde(rename_all = "snake_case")]` | Matches all Go pollers' wire format. Preserves downstream compatibility with Vector/SIEM pipelines. |
| Env var names | `PRISM_{DOMAIN}_{FIELD}` with `_FILE` suffix for secrets | Consolidates the Go pollers' multi-prefix chaos (5 different schemes in poller-bear). Single `PRISM_` prefix for all Prism config. |
| Test function names | `{subject}_{action}_{expected_outcome}` | Matches tally and axiathon conventions. More descriptive than Go's `Test<Type>_<Scenario>`. |
| Error type variants | `PascalCase` enum variants | Rust convention. Maps to Go's `Err<Component><Condition>` semantically but uses Rust's type system. |
| MCP input DTOs | `{ToolName}Input` suffix | Matches tally: `RecordFindingInput`, `QueryFindingsInput`. |

---

## 2. Error Handling Patterns

### What Each Repo Does

**Go pollers (4 repos) -- Sentinel errors:**
```go
// Definition in apperrors/errors.go
var ErrStateNotFound = errors.New("state not found")
var ErrCursorRegression = errors.New("cursor regression")

// Usage with wrapping
return fmt.Errorf("%w: status=%d", apperrors.ErrSinkDelivery, resp.StatusCode)

// Matching
if errors.Is(err, apperrors.ErrStateNotFound) { /* bootstrap */ }
```
- 15-17 sentinel errors per poller, defined in dedicated `apperrors` package
- Wrapped with `%w` for `errors.Is()` matching
- Construction guards use plain `errors.New()` (inconsistent)
- Validation uses `errors.Join()` for multi-error aggregation (config only)
- Known bugs: unused sentinels (6 in poller-cobra, 5 in poller-coaster, 5 in poller-express), `ErrCursorRegression` defined but never wrapped in 4 of 7 poller-coaster collectors

**TypeScript MCP servers (2 repos) -- Typed error hierarchy:**
```typescript
// mcp-claroty-xdome: Class hierarchy
export class McpError extends Error { constructor(message, code, data?) {} }
export class ValidationError extends McpError { /* code: -32602 */ }
export class AuthenticationError extends McpError { /* code: -32001 */ }
export class IntegrationError extends McpError { /* code: -32007 */ }

// serveMyAPI: String errors
catch (error) { return { isError: true, text: `Error: ${(error as Error).message}` }; }
```
- mcp-claroty-xdome: 10 error classes with JSON-RPC 2.0 code mapping
- serveMyAPI: Unstructured string errors with `(error as Error).message` cast (unsafe)
- Error propagation: only API client layer catches; domain/tool layers are transparent

**Rust tools (3 repos) -- thiserror enums:**
```rust
// tally: Structured error with actionable context
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TallyError {
    #[error("invalid transition from {from} to {to} (valid: {valid:?})")]
    InvalidTransition { from: String, to: String, valid: Vec<String> },
    #[error("branch '{branch}' not found (run `tally init`)")]
    BranchNotFound { branch: String },
    #[error(transparent)]
    Git(#[from] git2::Error),
}

// ocsf-proto-gen: 7-variant error enum
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("schema error: {0}")]
    Schema(String),
    #[error("read {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
}

// axiathon: Per-crate error with Result alias
pub type Result<T> = std::result::Result<T, AxiathonError>;
```

### Conflicts

| Aspect | Go | TypeScript | Rust |
|--------|-----|-----------|------|
| Error definition | `var Err* = errors.New(...)` | `class *Error extends McpError` | `enum Error { Variant {...} }` |
| Error matching | `errors.Is(err, sentinel)` | `instanceof` check | `match` on enum variant |
| Error context | `fmt.Errorf("%w: context", sentinel)` | Constructor parameters | Structured fields in variant |
| Multi-error | `errors.Join()` | N/A | `Vec<Error>` |
| Error-to-MCP mapping | N/A (pollers don't use MCP) | HTTP status -> JSON-RPC code | `to_mcp_err()` free function |

### Recommended Resolution for Prism

**Use `thiserror` with structured, actionable error variants following tally's pattern:**

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PrismError {
    // Sensor errors (per-sensor variant with source)
    #[error("sensor {sensor} error: {message}")]
    Sensor { sensor: String, message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>> },

    // State errors
    #[error("state not found for sensor {sensor} (run initialization)")]
    StateNotFound { sensor: String },

    #[error("cursor regression: new cursor {new:?} <= stored {stored:?}")]
    CursorRegression { new: String, stored: String },

    #[error("query fingerprint mismatch (stored: {stored}, current: {current}). Delete state file to reset.")]
    FingerprintMismatch { stored: String, current: String },

    // Config errors
    #[error("configuration error: {0}")]
    Config(String),

    // MCP protocol errors
    #[error("invalid input: {0}")]
    InvalidInput(String),

    // Infrastructure
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
```

**Rationale:**
- `thiserror` is used by all 3 Rust repos (axiathon, ocsf-proto-gen, tally). It is the Rust ecosystem standard.
- `#[non_exhaustive]` from axiathon and tally prevents breaking downstream match arms when adding variants.
- Structured fields (not string-only) from tally's `InvalidTransition { from, to, valid }` pattern enable programmatic error handling.
- Actionable messages from tally: "run initialization", "delete state file" guide operators.
- Per-crate `Result<T>` type alias from axiathon: `pub type Result<T> = std::result::Result<T, PrismError>;`

**MCP error mapping -- centralize (improve on tally's distributed pattern):**

```rust
impl From<PrismError> for McpError {
    fn from(e: PrismError) -> McpError {
        match &e {
            PrismError::InvalidInput(_) => McpError::new(ErrorCode::INVALID_REQUEST, e.to_string()),
            PrismError::StateNotFound { .. } => McpError::new(ErrorCode::INVALID_REQUEST, e.to_string()),
            PrismError::Config(_) => McpError::new(ErrorCode(-32009), e.to_string()),
            _ => McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string()),
        }
    }
}
```

**Rationale:** Tally uses a free function `to_mcp_err()` that always returns `ErrorCode(-1)` with inline INVALID_REQUEST construction in each tool method. mcp-claroty-xdome maps HTTP status codes to 10 distinct JSON-RPC codes. Prism should centralize the mapping in a `From` impl, adopting mcp-claroty-xdome's code taxonomy but applying it to Rust error variants.

**Config validation multi-error -- adopt from Go pollers:**

```rust
pub fn validate_config(config: &Config) -> Result<(), Vec<ConfigError>> {
    let mut errors = Vec::new();
    // ... all checks
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

**Rationale:** All 4 Go pollers use `errors.Join()` for config validation so operators see all problems at once. Tally does not have multi-error validation. This is a quality-of-life improvement for operators.

---

## 3. Logging/Observability

### What Each Repo Does

**Go pollers (4 repos) -- charmbracelet/log:**
```go
log.Info("collecting alerts", "source", "alerts", "cursor", cursor.Timestamp)
log.Error("failed to deliver", "error", err, "record_id", id)
```
- Structured JSON logging with `charmbracelet/log`
- Module-scoped loggers (some repos)
- Configurable level via env var (`POLLER_BEAR_LOG_LEVEL`, `POLLER_COASTER_LOG_LEVEL`)
- No metrics, no tracing, no spans
- poller-cobra bug: log level parsing rejects WARN/ERROR/FATAL despite config accepting them

**TypeScript MCP servers (2 repos) -- Winston / console:**
```typescript
// mcp-claroty-xdome: Winston with module-scoped loggers
const logger = createLogger('AlertService');
logger.info('Cache hit', { cacheKey });

// serveMyAPI: console.error only
console.error('Error storing API key:', error);
```
- mcp-claroty-xdome: Winston with JSON formatter, colorized dev output, module-scoped loggers
- serveMyAPI: `console.error` only, no structured logging, no levels

**Rust tools (3 repos) -- tracing:**
```rust
// tally: tracing with skip_all pattern
#[tracing::instrument(skip_all, fields(uuid = %id))]
pub fn handle_update_status(store: &GitFindingsStore, id: &str, status: &str) -> Result<()> {
    tracing::debug!("updating status");
}

// axiathon: tracing present but minimal in spike
// ocsf-proto-gen: eprintln! for warnings, no tracing crate
```

### Conflicts

| Aspect | Go (charmbracelet) | TS (Winston) | Rust (tracing) |
|--------|-------------------|-------------|----------------|
| Structured logging | Yes (KV pairs) | Yes (JSON) | Yes (spans + fields) |
| Async-aware | No (Go routines) | N/A | Yes (spans propagate across .await) |
| Span/trace context | No | No | Yes (instrument macro) |
| Output format | JSON | JSON + colorized | Subscriber-configurable |
| Metrics | None | None | Via tracing-opentelemetry |
| Log levels | debug/info/warn/error | error/warn/info/debug/silly | trace/debug/info/warn/error |

### Recommended Resolution for Prism

**Use `tracing` + `tracing-subscriber` with JSON output, adopting tally's instrumentation pattern:**

```rust
// Cargo.toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

// Instrumentation pattern (from tally)
#[tracing::instrument(skip_all, fields(sensor = %sensor_name, cursor = ?current_cursor))]
pub async fn collect_once<S: DataSource>(&self, source: &S) -> Result<CollectionResult> {
    tracing::info!("starting collection cycle");
    // ...
    tracing::debug!(records = batch.len(), "batch fetched");
}

// Subscriber setup
tracing_subscriber::fmt()
    .json()
    .with_env_filter(EnvFilter::from_env("PRISM_LOG_LEVEL"))
    .with_target(true)
    .with_thread_ids(true)
    .init();
```

**Key conventions:**
- `skip_all` always (from tally) -- never dump full struct contents into spans
- Cherry-pick relevant fields: `sensor`, `cursor`, `record_count`, `duration`
- stderr for all diagnostics (stdout reserved for MCP JSON-RPC)
- JSON format for production, pretty format for development
- Level from env: `PRISM_LOG_LEVEL=info` (default), supports `trace/debug/info/warn/error`

**Metrics -- add from day one (absent from all 9 repos):**

```rust
// Via tracing-opentelemetry or prometheus crate
// Key metrics (identified as missing NFR across all pollers):
// - prism_records_collected_total{sensor, source}
// - prism_records_delivered_total{sensor, source}
// - prism_collection_duration_seconds{sensor, source}
// - prism_delivery_duration_seconds{sensor}
// - prism_cursor_lag_seconds{sensor, source}
// - prism_retry_count_total{sensor, source}
// - prism_cache_hit_ratio{sensor}
// - prism_active_sessions
```

**Rationale:** All 4 Go pollers explicitly lack metrics and tracing (documented as missing NFRs). mcp-claroty-xdome's Python variant adds performance monitoring but the TypeScript production code has none. Only tally uses `tracing` in production Rust code. Prism should adopt tracing from day one and add OpenTelemetry metrics that all pollers identified as missing.

---

## 4. Configuration Patterns

### What Each Repo Does

**Go pollers (4 repos) -- env vars with `_FILE` variants:**
```go
// poller-bear config.go
baseURL := os.Getenv("CLAROTY_BASE_URL")
if file := os.Getenv("CLAROTY_BASE_URL_FILE"); file != "" {
    content, err := os.ReadFile(file)
    if err == nil { baseURL = strings.TrimSpace(string(content)) }
}
```
- All config from env vars -- no config files, no CLI flags (except `--dry-run`)
- `*_FILE` suffix for K8s secret mounts (file takes precedence over direct env var)
- Multi-prefix chaos: poller-bear uses 5 different prefixes (`CLAROTY_*`, `VECTOR_*`, `POLLER_BEAR_*`, `COLLECTOR_*`, `ENABLE_*`)
- Duration parsing inconsistency: some accept both integers and Go duration strings, others only duration strings
- Hardcoded defaults for some critical values (client timeout, page sizes) with no env var override
- Helm-config mismatches: poller-bear sets 4 env vars in Helm that config.go never reads

**TypeScript MCP servers (2 repos) -- env vars (no `_FILE`):**
```typescript
// mcp-claroty-xdome: direct env var access
const baseUrl = process.env.CLAROTY_XDOME_BASE_URL;
const apiToken = process.env.CLAROTY_XDOME_API_TOKEN;

// serveMyAPI: 4 env vars
DOCKER_ENV, STORAGE_DIR, PORT, NODE_ENV
```
- mcp-claroty-xdome: 5 env vars, no `_FILE` variants, no validation
- serveMyAPI: 4 env vars, no validation, `DOCKER_ENV` is a boolean flag

**Rust tools (3 repos) -- mixed approaches:**
```rust
// tally: CLI args via clap, no env vars for core config
#[derive(Parser)]
pub enum Command {
    Record { #[arg(long)] severity: Option<String>, ... },
    McpServer,
}

// axiathon: Hardcoded defaults in Default impls
impl Default for WriterConfig {
    fn default() -> Self { WriterConfig { buffer_size: 1000, flush_interval: Duration::from_secs(5) } }
}

// ocsf-proto-gen: CLI args via clap derive
#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "1.7.0")]
    ocsf_version: String,
}
```

### Conflicts

| Aspect | Go pollers | TS MCP servers | Rust tools |
|--------|-----------|---------------|------------|
| Source | Env vars only | Env vars only | CLI args (clap) |
| Secret handling | `_FILE` env vars | None | None |
| Validation | Multi-error aggregation | None | Via clap derive |
| Defaults | Mix of code and env | Hardcoded | clap `default_value` or Default impl |
| Config file | None | None | None |

### Recommended Resolution for Prism

**Layered configuration with precedence: CLI args > env vars > config file > defaults:**

```rust
use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Deserialize, Debug)]
#[command(name = "prism")]
pub struct Config {
    #[arg(long, env = "PRISM_LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    #[command(flatten)]
    pub server: ServerConfig,

    #[command(flatten)]
    pub sensors: SensorConfig,
}

#[derive(Parser, Deserialize, Debug)]
pub struct ServerConfig {
    #[arg(long, env = "PRISM_HEALTH_ADDR", default_value = "127.0.0.1:7321")]
    pub health_addr: String,

    #[arg(long, env = "PRISM_MCP_TRANSPORT", default_value = "stdio")]
    pub mcp_transport: String,
}
```

**Secret file support (adopt from all 4 Go pollers):**

```rust
/// Resolve a secret value from file or environment variable.
/// Precedence: file (if exists and readable) > env var > error.
pub fn resolve_secret(file_env: &str, direct_env: &str) -> Result<String, ConfigError> {
    if let Ok(path) = std::env::var(file_env) {
        let content = std::fs::read_to_string(path.trim())?;
        return Ok(content.trim().to_string());
    }
    if let Ok(value) = std::env::var(direct_env) {
        return Ok(value.trim().to_string());
    }
    Err(ConfigError::MissingSecret { env_var: direct_env.to_string() })
}
```

**Env var prefix convention:**

```
PRISM_LOG_LEVEL          -- global
PRISM_HEALTH_ADDR        -- server
PRISM_MCP_TRANSPORT      -- server

PRISM_CLAROTY_BASE_URL         -- per-sensor
PRISM_CLAROTY_API_TOKEN        -- per-sensor credential
PRISM_CLAROTY_API_TOKEN_FILE   -- K8s secret mount variant
PRISM_CLAROTY_TIMEOUT          -- per-sensor operational param

PRISM_ARMIS_API_URL            -- per-sensor
PRISM_ARMIS_API_SECRET_FILE    -- K8s secret mount

PRISM_SINK_ENDPOINT            -- delivery
PRISM_SINK_USERNAME            -- delivery credential
PRISM_SINK_PASSWORD_FILE       -- K8s secret mount

PRISM_XMP_SITE                 -- enrichment
PRISM_XMP_CLUSTER_NAME         -- enrichment
PRISM_XMP_NODE_NAME            -- enrichment
```

**Rationale:**
- Single `PRISM_` prefix (fixes poller-bear's 5-prefix problem)
- `_FILE` suffix for all credentials (adopted from all 4 Go pollers; this is the K8s-native pattern)
- clap's `env` attribute unifies CLI args and env vars with zero additional code
- Multi-error validation (adopted from Go pollers' `errors.Join()` pattern)
- `--dry-run` flag (adopted from poller-cobra, poller-coaster, poller-express) for deployment verification
- Uniform duration parsing: always accept `30s`, `5m`, `1h` via humantime crate (fixes poller-coaster's inconsistency)
- All operational parameters configurable (fixes poller-bear's hardcoded timeouts and page sizes)

---

## 5. API Versioning

### What Each Repo Does

**Go pollers (4 repos) -- no versioning of poller APIs; consume versioned external APIs:**
- poller-bear: Consumes Claroty `/api/v1/*` endpoints -- version embedded in URL path
- poller-coaster: Consumes Armis via `armis-sdk-go/v2` -- SDK version, not API version
- poller-cobra: Consumes CrowdStrike `QueryV2`, `PostEntitiesAlertsV1` -- version in operation name
- poller-express: Consumes Cyberint `/alert/api/v1/alerts` -- version in URL path
- None of the pollers version their own output (the xMP envelope format is unversioned)

**TypeScript MCP servers (2 repos) -- MCP protocol handles versioning:**
- mcp-claroty-xdome: MCP protocol negotiation via `protocolVersion` in server info. Tool names are unversioned. Schema changes via Zod enum expansion (breaking: new required fields).
- serveMyAPI: No versioning. 4 tools with stable names.

**Rust tools (3 repos) -- schema versioning or semantic versioning:**
- axiathon: OCSF version in proto package path (`ocsf.v1_7_0.events.iam`). Multi-version support planned via `OcsfVersionAliasMap`.
- ocsf-proto-gen: OCSF version as CLI arg, embedded in output directory structure. Proto field numbers NOT stable across OCSF versions (ADR-05, P0-02).
- tally: Schema version in Finding JSON (`schema_version: "1.1.0"`). `#[serde(default)]` on all fields for backward compatibility.

### Recommended Resolution for Prism

**MCP tool versioning:**

Prism's MCP tools should NOT be versioned in their names. The MCP protocol handles capability negotiation. Tool evolution follows these rules:

1. **Additive changes are non-breaking:** New optional parameters, new response fields. Use `Option<T>` for all new parameters. Use `#[serde(default)]` for all new fields.
2. **Removal or semantic changes are breaking:** Require a new tool name (e.g., `query_findings_v2`). Deprecate old tool but keep it functional for at least one release.
3. **Server info includes version:** `server_info.version` from `CARGO_PKG_VERSION`. Clients can check compatibility.

**Sensor API versioning:**

```rust
pub struct SensorConfig {
    /// API version to use (e.g., "v1", "v2"). Determines endpoint paths.
    pub api_version: String,
}
```

Each sensor adapter owns its API version. The version affects URL construction, request format, and response parsing. No cross-sensor API version coordination needed.

**Data format versioning (adopt tally's pattern):**

```rust
pub struct CursorState {
    /// Schema version for forward compatibility
    pub schema_version: String,  // "1.0.0"

    #[serde(default)]
    pub cursors: HashMap<String, SensorCursor>,

    #[serde(default)]
    pub fingerprints: HashMap<String, String>,
}
```

All persisted data structures include `schema_version`. All fields use `#[serde(default)]` for backward compatibility. This is proven by tally's Finding schema and axiathon's storage format.

**OCSF versioning (adopt from ocsf-proto-gen + axiathon):**

- OCSF version embedded in proto package path: `ocsf.v1_7_0.events.*`
- Single OCSF version per Prism release (simplest approach; ocsf-proto-gen P0-02)
- Proto field numbers NOT stable across versions (sequential alphabetical); never mix proto data across versions
- Version-conditional field aliases planned for future (axiathon's `OcsfVersionAliasMap`)

---

## 6. Testing Patterns

### What Each Repo Does

**Go pollers (4 repos) -- table-driven + fakes:**
```go
// Table-driven tests (all 4 pollers)
tests := []struct {
    name    string
    input   InputType
    wantErr bool
}{
    {"success case", validInput, false},
    {"empty token", "", true},
}
for _, tc := range tests {
    t.Run(tc.name, func(t *testing.T) { ... })
}

// Fakes preferred over mocks (poller-bear)
type fakeClarotyClient struct {
    alertsResponse *AlertsBatch
    alertsErr      error
    callCount      int
}

// httptest for API simulation (all 4)
server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { ... }))

// Golden file testing (poller-bear OCSF)
// Benchmark tests (poller-bear, poller-express)
```

**TypeScript MCP servers (2 repos) -- describe/it + vi.mock:**
```typescript
// mcp-claroty-xdome: Vitest with AAA pattern
describe("AlertService", () => {
    let service: AlertService;
    beforeEach(() => { service = new AlertService(mockClient, mockCache, mockLogger); });
    it("should return cached result on cache hit", async () => {
        mockCache.get.mockReturnValue(expected);
        const result = await service.findAlerts(params);
        expect(result).toEqual(expected);
    });
});

// serveMyAPI: Zero tests (npm test is a placeholder)
```

**Rust tools (3 repos) -- #[test] + proptest + insta:**
```rust
// tally: Four-tier test strategy
// 1. Unit: #[test] in same file or tests/ dir
#[test]
fn state_machine_rejects_self_transition() { ... }

// 2. Property: proptest for invariants
proptest! {
    #[test]
    fn fingerprint_is_deterministic(file in ".*", line in 0u32..1000, rule in ".*") {
        let fp1 = compute_fingerprint(&file, line, &rule);
        let fp2 = compute_fingerprint(&file, line, &rule);
        prop_assert_eq!(fp1, fp2);
    }
}

// 3. Integration: assert_cmd for CLI subprocess testing
// 4. E2E: Full workflow scenarios

// axiathon: Property tests + snapshot tests (insta)
// ocsf-proto-gen: String assertion pattern (contains/!contains)
```

### Conflicts

| Aspect | Go | TypeScript | Rust |
|--------|-----|-----------|------|
| Test runner | `go test` | Vitest | `cargo test` |
| Parameterized | Table-driven `struct` | `describe.each` / loop | `#[test_case]` crate or manual loop |
| Mocking | Hand-written fakes | `vi.mock` + `vi.fn` | Hand-written fakes + mockall crate |
| Property testing | None | None | proptest |
| Snapshot testing | Golden files | None | insta |
| Benchmarks | `testing.B` | None | criterion |
| HTTP mocking | `httptest.NewServer` | N/A | wiremock or custom axum server |
| Test naming | `Test<Type>_<Scenario>` | `"should <behavior>"` | `{subject}_{action}_{outcome}` |

### Recommended Resolution for Prism

**Four-tier test strategy (adopt from tally, enhanced with Go patterns):**

| Tier | Framework | Location | Purpose |
|------|-----------|----------|---------|
| Unit | `#[test]` | `#[cfg(test)] mod tests` in source file | Single function behavior |
| Property | `proptest` | `tests/property_*.rs` | Invariants over random input |
| Integration | `#[tokio::test]` + wiremock | `tests/integration_*.rs` | API client + state store |
| E2E | assert_cmd or in-process | `tests/e2e_*.rs` | Full MCP tool workflow |

**Test naming convention:**

```rust
#[test]
fn cursor_advance_rejects_regression() { ... }
#[test]
fn config_validate_reports_all_errors_at_once() { ... }
#[test]
fn collect_once_delivers_all_records_before_advancing_cursor() { ... }
```

Pattern: `{subject}_{action}_{expected_outcome}` from tally. No `test_` prefix (Rust convention).

**Fakes over mocks (adopt from Go pollers + tally):**

```rust
struct FakeSensorClient {
    responses: Vec<Result<Batch, PrismError>>,
    call_count: AtomicUsize,
}

impl SensorClient for FakeSensorClient {
    async fn fetch(&self, cursor: &Cursor) -> Result<Batch, PrismError> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        self.responses[idx].clone()
    }
}
```

**Rationale:** All 4 Go pollers prefer hand-written fakes for core integration tests. Tally uses the same approach. Fakes are more readable and debuggable than macro-generated mocks. Use mockall only for complex interfaces where fake construction is prohibitive.

**Table-driven pattern (adopt from Go, translate to Rust):**

```rust
#[test]
fn parse_duration_handles_all_formats() {
    let cases = vec![
        ("30s", Duration::from_secs(30)),
        ("5m", Duration::from_secs(300)),
        ("1h", Duration::from_secs(3600)),
    ];
    for (input, expected) in cases {
        assert_eq!(parse_duration(input).unwrap(), expected, "input: {input}");
    }
}
```

**Golden file testing (adopt from poller-bear for OCSF mapping):**

Store expected outputs in `testdata/golden/`. Compare actual output against golden files. Use `insta` crate for snapshot management (update snapshots with `cargo insta review`).

**Benchmark testing (adopt from Go pollers for hot paths):**

```rust
// benches/collection.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_cursor_comparison(c: &mut Criterion) {
    c.bench_function("cursor_is_ahead", |b| {
        b.iter(|| cursor_a.is_ahead_of(&cursor_b))
    });
}
```

---

## 7. State Management

### What Each Repo Does

**Go pollers (4 repos) -- FileStore + MemoryStore:**
- poller-bear: FileStore (production), MemoryStore (testing). Atomic write: temp + fsync + rename. RWMutex for concurrency. Single JSON file with all 9 source states + receipts.
- poller-coaster: Same pattern. FileStore with atomic write. 7 source states.
- poller-cobra: Config supports FileStore but runner HARDCODES MemoryStore. All state lost on restart. Known bug.
- poller-express: MemoryStore only. No persistence. All state lost on restart. Known limitation.
- All: Cursor = `(Timestamp, RecordID)` composite. Forward progress invariant. Query fingerprint drift detection via SHA-256.

**TypeScript MCP servers (2 repos) -- in-memory only:**
- mcp-claroty-xdome: InMemoryCacheManager per service, 5-min TTL, no bounds, no persistence.
- serveMyAPI: OS keyring (keytar) or plaintext files (Docker fallback). No cursor state.

**Rust tools (3 repos) -- git-backed or file-based:**
- tally: Git orphan branch storage. One JSON file per finding. Index.json regenerable. Per-commit persistence.
- axiathon: In-memory stores (spike). Iceberg/Parquet for event storage (production concept).
- ocsf-proto-gen: No state. Pure transformation pipeline.

### Conflicts

| Aspect | Go pollers | TS MCP servers | Rust tools |
|--------|-----------|---------------|------------|
| Cursor persistence | File (atomic JSON) | None | Git (tally) |
| State format | Single JSON with all sources | In-memory Map | One file per entity |
| Concurrency | RWMutex | N/A | Git (natural isolation) |
| Crash safety | temp + fsync + rename | N/A | Git commit |
| Store interface | `Store` with sub-interfaces | No abstraction | `GitFindingsStore` |

### Recommended Resolution for Prism

**Trait-based state store with file-backed default (adopt from Go pollers, generalize):**

```rust
#[async_trait]
pub trait StateStore: Send + Sync {
    /// Load cursor state for a sensor source. Returns None if no state exists.
    async fn load_cursor(&self, sensor: &str, source: &str) -> Result<Option<CursorState>>;

    /// Save cursor state atomically. Previous state unchanged on error.
    async fn save_cursor(&self, sensor: &str, source: &str, state: &CursorState) -> Result<()>;

    /// Load query fingerprint for drift detection.
    async fn load_fingerprint(&self, sensor: &str, source: &str) -> Result<Option<String>>;

    /// Save query fingerprint.
    async fn save_fingerprint(&self, sensor: &str, source: &str, fingerprint: &str) -> Result<()>;
}
```

**Implementations:**

| Implementation | Use Case | Reference |
|---------------|----------|-----------|
| `FileStore` | Production (default) | poller-bear's atomic write pattern |
| `MemoryStore` | Testing | All repos |
| `SqliteStore` | Future (multi-sensor, distributed) | N/A (new for Prism) |

**Atomic write pattern (adopt from poller-bear/poller-coaster):**

```rust
pub async fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let temp = path.with_extension("tmp");
    let mut file = tokio::fs::File::create(&temp).await?;
    file.write_all(data).await?;
    file.sync_all().await?;  // fsync
    drop(file);
    tokio::fs::rename(&temp, path).await?;  // atomic rename
    Ok(())
}
```

**Cursor model (unified from all 4 Go pollers):**

```rust
pub struct CursorState {
    pub timestamp: DateTime<Utc>,
    pub record_id: String,
    /// Optional third key for 3-tuple cursors (e.g., poller-bear device relations)
    pub secondary_id: Option<String>,
    /// Offset for offset-based pagination sources
    pub offset: Option<u64>,
}
```

**Forward progress enforcement (fix Go pollers' inconsistency):**

```rust
pub fn ensure_forward_progress(new: &CursorState, stored: &CursorState) -> Result<()> {
    if new <= stored {
        return Err(PrismError::CursorRegression {
            new: format!("{:?}", new),
            stored: format!("{:?}", stored),
        });
    }
    Ok(())
}
```

Always use the typed error variant (fixes the bug where 4 of 7 poller-coaster collectors use plain `fmt.Errorf` instead of the `ErrCursorRegression` sentinel).

**Rationale:**
- File-backed state from day one (fixes poller-cobra's hardcoded MemoryStore bug and poller-express's missing persistence)
- Atomic write pattern from poller-bear is crash-safe and proven
- Trait abstraction enables MemoryStore for tests and future SqliteStore for distributed deployments
- Unified cursor model covers both timestamp-based (5/9 poller-bear sources) and offset-based (4/9 poller-bear sources) pagination
- State saved AFTER successful sink delivery (fixes poller-cobra's P0-2 bug where in-memory state updates before persistence)

---

## 8. Authentication Patterns

### What Each Repo Does

**Go pollers (4 repos) -- diverse auth mechanisms per sensor API:**
- poller-bear (Claroty): Bearer token. `Authorization: Bearer <token>`. Token from env var or `_FILE`. Trimmed of whitespace.
- poller-coaster (Armis): Bearer token via `armis-sdk-go/v2` SDK. Token managed by SDK.
- poller-cobra (CrowdStrike): OAuth2 Client Credentials flow via `gofalcon` SDK. Client ID + Client secret. Multi-region support.
- poller-express (Cyberint): Cookie-based auth. `access_token` cookie injected via custom `http.RoundTripper`.
- All: Basic auth to downstream sink (Vector): `VECTOR_USERNAME`, `VECTOR_PASSWORD`.

**TypeScript MCP servers (2 repos) -- token-based:**
- mcp-claroty-xdome: Static bearer token from env var. No refresh, no rotation.
- serveMyAPI: OS keyring for credential storage. No auth on MCP endpoints themselves.

**Rust tools (3 repos) -- minimal auth:**
- tally: Git credentials via 4-strategy chain (credential helper, SSH agent, env var, interactive prompt).
- axiathon: Hardcoded vault passphrase (spike anti-pattern). AES-256-GCM with Argon2 KDF (correct crypto, wrong key source).
- ocsf-proto-gen: No auth (reads local files or public OCSF API).

### Conflicts

| Auth Mechanism | Used By | Prism Need |
|---------------|---------|------------|
| Bearer token | poller-bear, poller-coaster, mcp-claroty-xdome | Claroty, Armis sensors |
| OAuth2 Client Credentials | poller-cobra | CrowdStrike sensor |
| Cookie-based | poller-express | Cyberint sensor |
| Basic auth (sink) | All 4 Go pollers | Downstream delivery |
| OS keyring | serveMyAPI | Credential storage |
| Git credentials | tally | N/A for Prism |

### Recommended Resolution for Prism

**Unified credential management with per-sensor auth adapters:**

```rust
/// Credential resolved from file or env var
pub struct Credential {
    value: secrecy::SecretString,
    source: CredentialSource,
}

pub enum CredentialSource {
    File(PathBuf),
    EnvVar(String),
    Keyring(String),
}

/// Per-sensor authentication adapter
#[async_trait]
pub trait SensorAuth: Send + Sync {
    /// Apply authentication to an outgoing request
    async fn authenticate(&self, request: reqwest::RequestBuilder) -> Result<reqwest::RequestBuilder>;

    /// Refresh credentials if needed (no-op for static tokens)
    async fn refresh(&self) -> Result<()>;
}
```

**Implementations:**

```rust
/// Bearer token auth (Claroty, Armis)
pub struct BearerTokenAuth {
    token: Credential,
}

impl SensorAuth for BearerTokenAuth {
    async fn authenticate(&self, req: reqwest::RequestBuilder) -> Result<reqwest::RequestBuilder> {
        Ok(req.bearer_auth(self.token.value.expose_secret()))
    }
}

/// OAuth2 Client Credentials (CrowdStrike)
pub struct OAuth2Auth {
    client_id: Credential,
    client_secret: Credential,
    token_url: String,
    cached_token: RwLock<Option<(String, Instant)>>,
}

/// Cookie-based auth (Cyberint)
pub struct CookieAuth {
    cookie_name: String,
    cookie_value: Credential,
}

/// Basic auth (sink delivery)
pub struct BasicAuth {
    username: Credential,
    password: Credential,
}
```

**Credential storage (adopt from serveMyAPI, fix security issues):**

```rust
pub trait CredentialStore: Send + Sync {
    async fn store(&self, name: &str, secret: &str) -> Result<()>;
    async fn get(&self, name: &str) -> Result<Option<String>>;
    async fn delete(&self, name: &str) -> Result<bool>;
    async fn list(&self) -> Result<Vec<String>>;
}

// Implementations:
// 1. KeyringStore -- native OS keyring via keyring-rs (adopt from serveMyAPI)
// 2. EncryptedFileStore -- AES-256-GCM for containers (fix serveMyAPI's plaintext)
// 3. MemoryStore -- testing only
```

**Key conventions:**
- Use `secrecy::SecretString` for all credential values (prevents accidental logging)
- Credential names sanitized to `[a-zA-Z0-9_.-]` (fixes serveMyAPI's path traversal vulnerability)
- File-backed secrets with `_FILE` env var suffix (adopted from all 4 Go pollers)
- Token trimming on load (adopted from poller-bear: `strings.TrimSpace`)
- OAuth2 token caching with expiry tracking (improve on poller-cobra's SDK-delegated approach)
- Keyring probe at startup for macOS permission dialog (adopt from serveMyAPI's permission marker pattern)
- Credential index for `keyring-rs` enumeration gap (adopt from serveMyAPI analysis P0-2)

**Rationale:**
- Each sensor API uses a different auth mechanism. Prism cannot standardize on one. The `SensorAuth` trait abstracts this.
- The `secrecy` crate prevents credential values from appearing in logs, debug output, or error messages (improvement over all 9 repos).
- `_FILE` env vars are the K8s-native pattern used by all 4 Go pollers. Prism must support this for production deployments.
- The `CredentialStore` trait enables MCP tools for credential management (serveMyAPI's core use case) while fixing its security issues (plaintext storage, path traversal).
- OAuth2 Client Credentials flow is required for CrowdStrike. No official Rust SDK exists. Prism must implement this directly using the `oauth2` crate or `reqwest` with manual token management.

---

## Summary of Decisions

| Area | Decision | Primary Reference |
|------|----------|-------------------|
| Naming | Rust-idiomatic snake_case everywhere; `PRISM_` env prefix | tally, axiathon |
| Error handling | `thiserror` enum with structured variants; centralized MCP mapping | tally (pattern), mcp-claroty-xdome (codes) |
| Logging | `tracing` + JSON subscriber; `skip_all` instrumentation | tally (pattern), all pollers (missing metrics NFR) |
| Configuration | clap + env vars + `_FILE` secrets; `--dry-run` | Go pollers (`_FILE`), tally (clap) |
| API versioning | MCP tools unversioned (additive); sensor APIs version in config | mcp-claroty-xdome (MCP), ocsf-proto-gen (OCSF) |
| Testing | 4-tier (unit/property/integration/e2e); fakes over mocks | tally (4-tier), Go pollers (fakes, table-driven) |
| State management | Trait-based store; FileStore default; atomic write | poller-bear (FileStore), all pollers (cursor model) |
| Authentication | Per-sensor auth trait; `secrecy` for credentials; keyring + encrypted file | All 4 pollers (diverse auth), serveMyAPI (keyring) |
