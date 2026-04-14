# Pass 5 Deep: Conventions -- Round 1

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 1

---

## Convention 1: Naming Conventions

### C-1.1: Package Names
- Lowercase, single-word: `claroty`, `collector`, `config`, `health`, `ocsf`, `profiling`, `sink`, `state`, `transport`, `apperrors`
- Exception: `apperrors` is a compound name (Go convention for error packages)
- Runner uses path-based organization: `internal/app/runner` -- the `app` intermediate directory is unusual (most Go projects would use `internal/runner` directly)

### C-1.2: Struct Field Naming
- Go standard PascalCase for exported fields
- JSON tags use `snake_case` matching Claroty API field names
- Struct fields mirror API names in Go style: `device_uid` -> `DeviceUID`, `alert_name` -> `Name` (sometimes abbreviated)
- **Inconsistency**: Some struct fields match API names directly (`AlertID`, `DeviceUID`) while others abbreviate (`alert_name` -> `Name`, `alert_class` -> `Class`). The abbreviation happens in the top-level entity structs but NOT in the relation structs where full prefixes are used (`AlertID`, `AlertName`, `AlertClass` in `DeviceAlertRelation`).

### C-1.3: Environment Variable Naming
- All `SCREAMING_SNAKE_CASE`
- Prefix patterns: `CLAROTY_*`, `VECTOR_*`, `XMP_*`, `OCSF_*`, `STATE_STORE_*`, `COLLECTOR_*`, `POLLER_BEAR_*`, `ENABLE_*`, `PPROF_*`
- Secret file variants: `*_FILE` suffix
- **Inconsistency**: Log level uses `POLLER_BEAR_LOG_LEVEL` (with project prefix) but health address uses `COLLECTOR_HEALTH_ADDR` (without). Most other env vars use domain prefixes (`CLAROTY_*`, `VECTOR_*`).

### C-1.4: Test Function Naming
- Pattern: `Test<FunctionName>_<Scenario>` (e.g., `TestCollectAlerts_ClarotyClientError`)
- Integration tests use `Integration` suffix: `TestCollectAlertsIntegration`
- Table-driven tests use descriptive `name` field
- Benchmark tests: `Benchmark<FunctionName>` (e.g., `BenchmarkParseClarotyFloat`)

### C-1.5: Interface Naming
- Single-method style not used (interfaces are larger)
- Action-noun pattern: `Client` (claroty), `Sender` (sink), `Store` (state), `Reporter` (health)
- Sub-interfaces: `AlertStore`, `ServerStore`, etc. (entity-prefixed)
- The composite `Store` interface embeds all 9 sub-interfaces

### C-1.6: Error Variable Naming
- `Err<Component><Condition>` pattern: `ErrStateNotFound`, `ErrClarotyDecode`, `ErrSinkDelivery`
- Component prefix groups: `State`, `QueryFingerprint`, `Cursor`, `Collector`, `Claroty`, `Sink`, `Config`

---

## Convention 2: Code Organization Patterns

### C-2.1: Internal Package Layout
- All application code under `internal/` (Go access restriction)
- Feature-based packages: each package owns its types, interfaces, and implementation
- No `pkg/` directory -- everything is internal
- `tools/` directory with build-tagged `tools.go` for tool dependency pinning

### C-2.2: Interface + Implementation Separation
- Interfaces declared in separate files from implementations:
  - `sink/sink.go` (interface) vs `sink/http_sender.go` (implementation)
  - `state/store.go` (interfaces + types) vs `state/file_store.go` / `state/memory_store.go` (implementations)
- Exception: `claroty/api.go` contains both the `Client` interface and all domain types
- `go:generate mockgen` directives placed on interface files

### C-2.3: Domain Type Placement
- API-facing types (structs decoded from Claroty responses) in `claroty/api.go`
- State types (cursors, poll states, receipts) in `state/store.go`
- Output types (EnrichedPayload, XMPMetadata) in `sink/http_sender.go`
- OCSF types in `ocsf/detection_finding.go`
- **Cursor type duplication**: Both `claroty` and `state` packages define cursor types with the same names but slightly different field naming (confirmed in Pass 2)

### C-2.4: Single-File-Per-Concern
- `apperrors/errors.go` -- single file for all sentinel errors
- `transport/http.go` -- single file for transport config + factory
- `health/server.go` -- single file for health server
- `profiling/pprof.go` -- single file for pprof server
- Larger packages split by concept: `state/store.go` (types) + `state/file_store.go` (file impl) + `state/memory_store.go` (memory impl)

---

## Convention 3: Error Handling Patterns

### C-3.1: Sentinel + Wrap Pattern
```go
// Definition (apperrors/errors.go)
var ErrClarotyDecode = errors.New("claroty response decode failed")

// Usage (claroty/http_client.go)
return fmt.Errorf("%w: %v", apperrors.ErrClarotyDecode, err)
```
- Sentinels in dedicated `apperrors` package
- Wrapped with `%w` for `errors.Is()` matching
- Additional context via `%v` of the cause error
- **Consistency**: Applied uniformly across all packages

### C-3.2: Validation-First Constructor Pattern
```go
func NewHTTPSender(cfg, ...) (*HTTPSender, error) {
    endpoint := strings.TrimSpace(cfg.Endpoint)
    if endpoint == "" {
        return nil, fmt.Errorf("%w: endpoint is empty", apperrors.ErrSinkConfigMissing)
    }
    // ... more validation
    // ... then construction
}
```
- Constructor functions validate all inputs before constructing
- Return `(nil, error)` on validation failure
- Applied to: `NewHTTPSender`, `NewHTTPClient`, `NewTransport`, `NewFileStore`

### C-3.3: Deferred Cleanup
- `defer resp.Body.Close()` on all HTTP responses
- `defer os.Remove(tempFile.Name())` in atomic file writes
- `defer cancel()` for context timeouts
- Pprof: `defer func() { ... shutdownPprof(ctx) }()` in main.go

### C-3.4: Error Type Classification in State Loading
```go
state, err := store.Load(ctx)
if err != nil {
    if errors.Is(err, apperrors.ErrStateNotFound) {
        // Bootstrap: first run
    } else {
        // Fatal: unexpected error
    }
}
```
- Used consistently across all 9 source initialization functions

---

## Convention 4: Testing Patterns

### C-4.1: Fakes Over Mocks
- Primary testing pattern: hand-written fake implementations
- `fakeClarotyClient` in `collector_test.go`: stores function responses and tracks call counts
- `fakeSink` in `collector_test.go`: records all sent records and optionally returns errors
- Mock generation available (`go:generate mockgen`) but fakes are preferred for collector tests

### C-4.2: Table-Driven Tests
- Consistently used for parameterized scenarios
- Pattern: `tests := []struct{ name string; ... }{ ... }; for _, tc := range tests { t.Run(tc.name, ...) }`
- Applied in: `config_test.go`, `http_client_test.go`, `transport/http_test.go`, `profiling/pprof_test.go`, `health/server_test.go`

### C-4.3: Parallel Execution
- `t.Parallel()` used for independent tests
- **Not used with `t.Setenv()`** (incompatible in Go -- correctly observed in the codebase)
- Profiling and config tests that use `t.Setenv()` are NOT parallel

### C-4.4: httptest Usage
- `httptest.NewServer` for Claroty client tests (http_client_test.go)
- `httptest.NewRecorder` for health server tests (unit-level handler testing)
- Integration test in health: uses `httptest.NewServer(server.httpServer.Handler)` for full HTTP stack

### C-4.5: Temp Directory Pattern
- `t.TempDir()` for file store tests -- auto-cleaned
- Used for state file persistence testing

### C-4.6: Golden File Testing
- OCSF mapping uses golden file pattern
- Input files in `testdata/input/`, expected output in `testdata/golden/`
- 3 test cases: basic-high, critical-mitre, low-no-endpoints

### C-4.7: Benchmark Tests
- Present in 3 packages: `claroty`, `state`, `sink`
- File naming: `*_bench_test.go`
- Used for hot-path functions (parsing, serialization, state persistence)

---

## Convention 5: Design Patterns

### C-5.1: 9x Repetition Pattern (Anti-Pattern)
The most prominent pattern in the codebase is the **9x repetition** across all data sources:
- 9 entity structs in `claroty/api.go`
- 9 `Fetch*` methods on `Client` interface
- 9 `Send*` methods on `Sender` interface
- 9 sub-interfaces on `Store`
- 9 `collect*` functions in collector
- 9 `initialize*State` functions
- 9 `ensure*ForwardProgress` functions
- 9 cursor types (x2: claroty and state packages)
- 9 batch types
- 9 request types
- 9 poll state types
- 9 batch receipt types

This is the **primary structural anti-pattern**: the codebase has ~100 types and ~80 functions that follow the exact same pattern but are copy-pasted with entity-specific field names. A generic/trait-based design could reduce this by 80-90%.

### C-5.2: Options/Functional Options (Partial)
- `state.WithMaxReceipts(n)` uses functional options pattern for FileStore construction
- Not used elsewhere -- `collector.Options` is a plain struct, `claroty.Config` is a plain struct
- **Partially adopted** pattern

### C-5.3: Strategy Pattern (State Store)
- `state.Store` interface with two implementations: `FileStore` and `MemoryStore`
- Selected by `createStore(cfg.State, logger)` switch in runner
- Clean strategy pattern -- implementations are interchangeable

### C-5.4: Enrichment/Decorator Pattern (Sink)
- `enrichPayload(payload, recordType)` wraps raw records in `EnrichedPayload`
- Adds xMP metadata and optional OCSF normalization
- Applied uniformly to all 9 record types via `sendPayload` helper

### C-5.5: Cursor Pagination Pattern
Two distinct patterns coexist:
1. **Timestamp + ID cursor** (5 sources): Composite cursor with OR filter logic
2. **Offset + sort key** (4 sources): Simple offset-based with secondary sort for determinism
- Each pattern has its own forward progress enforcement logic

### C-5.6: Panic Recovery (OCSF Mapper)
```go
func (s *HTTPSender) mapOCSF(payload any, recordType string) (result json.RawMessage) {
    defer func() {
        if r := recover(); r != nil {
            s.logger.Error("ocsf mapper panic recovered", ...)
            result = nil
        }
    }()
    // ...
}
```
- Only instance of panic recovery in the codebase
- Defensive: OCSF mapping failure should not crash the collector
- Currently a TODO stub, but the recovery guard is already in place

---

## Convention 6: Configuration Patterns

### C-6.1: Default + Override Pattern
```go
cfg := config.DefaultConfig()        // Step 1: hardcoded defaults
cfg, err = config.LoadFromEnvironment(cfg)  // Step 2: env var overlay
```
- Two-phase configuration: defaults first, environment overlay second
- No config file support (env-only)
- No CLI flag support

### C-6.2: Secret File Precedence
```go
if fileValue, err := readSecretFile(os.Getenv(envFileVar)); err != nil {
    return cfg, err
} else if fileValue != "" {
    cfg.Field = fileValue
} else if fromEnv := os.Getenv(envVar); fromEnv != "" {
    cfg.Field = fromEnv
}
```
- File takes precedence over direct env var
- Non-existent file silently returns empty string (not an error)
- Applied consistently to all 7 secret-capable fields

### C-6.3: Duration Parsing
- `VECTOR_TIMEOUT_SECONDS` supports both `time.ParseDuration` format ("15s") and plain integer (parsed as seconds)
- Other durations (interval, retry delays) are not configurable via env vars -- only through `DefaultConfig`
- **Inconsistency**: Sink timeout is configurable but collection interval, retry delays, and client timeout are not

---

## Convention 7: Go-Specific Idioms

### C-7.1: go:embed for Static Data
```go
//go:embed data/severity-map.yaml
var severityMapData []byte
```
- Used for OCSF severity and adjustment YAML files
- Embeds at compile time -- no filesystem dependency at runtime

### C-7.2: go:generate for Code Generation
```go
//go:generate go run go.uber.org/mock/mockgen -source=sink.go -destination=mock_sender.go -package=sink
```
- Mock generation for interfaces
- Source mode (`-source`) not interface mode (`-destination`)

### C-7.3: Build Tags for Tools
```go
//go:build tools
package tools
import _ "github.com/golangci/golangci-lint/v2/cmd/golangci-lint"
```
- Standard Go pattern for pinning tool versions via `tools/go.mod`

### C-7.4: nolint Directives
- `//nolint:gosec // G101` on credential env var constants (false positive)
- `//nolint:gosec // G402` on InsecureSkipVerify (intentional)
- Always include explanation comment after `//`

---

## Consistency Assessment

| Pattern | Applied Everywhere | Applied Sporadically | Notes |
|---------|-------------------|---------------------|-------|
| Sentinel error wrapping | YES | - | All packages use `%w` wrapping |
| Table-driven tests | - | YES | Some packages use it, others use individual test functions |
| Fake over mock | - | YES | Collector uses fakes; sink/state have mock generation |
| t.Parallel() | - | YES | Used where compatible with t.Setenv |
| Constructor validation | YES | - | All constructors validate before constructing |
| JSON tag snake_case | YES | - | All struct tags match API conventions |
| go:generate mockgen | - | YES | Only on sink and state interfaces |
| Functional options | - | PARTIAL | Only `WithMaxReceipts`; others use plain structs |
| Env var configurability | - | PARTIAL | Sink timeout configurable; client timeout/interval not |
| 9x copy-paste | YES | - | The dominant structural pattern |

---

## Delta Summary
- New items added: 7 convention categories with 25 sub-items, 1 major anti-pattern (9x repetition), 3 inconsistencies identified (field naming, env var prefixes, configurability)
- Existing items refined: Error handling pattern fully documented with examples, testing patterns categorized by type
- Remaining gaps: Exact code patterns in collector.go (initialization, Run loop), benchmark test patterns

## Novelty Assessment
Novelty: SUBSTANTIVE
The 9x repetition anti-pattern is the single most important structural finding for spec crystallization -- it fundamentally shapes how the system should be redesigned. The inconsistencies in env var naming and configurability are design decisions that need resolution. The panic recovery in mapOCSF is a defensive pattern worth preserving.

## Convergence Declaration
Another round needed -- should verify patterns within collector.go (the largest file), confirm benchmark test patterns, and check for any undiscovered conventions in the legacy Python code.

## State Checkpoint
```yaml
pass: 5
round: 1
status: complete
files_scanned: 30
timestamp: 2026-04-13T23:45:00Z
novelty: SUBSTANTIVE
next_pass: 5-r2
```
