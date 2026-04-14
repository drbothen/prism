# Pass 5 Deep: Conventions & Pattern Catalog -- poller-cobra (Round 1)

> Convergence deepening round 1. Extracted from all source files, tests, and configuration.

---

## Naming Conventions

### Package Names

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| Short, lowercase, single-word | `collector`, `config`, `sink`, `state`, `health`, `profiling` | 100% consistent |
| `apperrors` (abbreviation) | `internal/apperrors` | Acceptable Go convention |
| Nested package path | `internal/app/runner` | Only one nested package; rest are flat under `internal/` |

### Type Names

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| PascalCase | `AlertCollector`, `HTTPClient`, `HTTPSender`, `MemoryStore`, `PollState` | 100% |
| Acronyms capitalized | `HTTPClient`, `HTTPSender`, `XMPConfig`, `XMPMetadata` | 100% |
| Config suffix | `CrowdStrikeConfig`, `CollectorConfig`, `SinkConfig`, `LoggingConfig`, `XMPConfig`, `StateConfig`, `RateLimitConfig` | 100% for config types |
| Interface naming | `Client`, `Sender`, `Store`, `Reporter`, `Record` | No `I` prefix (correct Go style) |

### Function Names

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| Constructor: `New{Type}` | `NewHTTPClient`, `NewHTTPSender`, `NewMemoryStore`, `NewServer`, `New`, `NewAlertCollector`, `NewQueryFingerprint` | 100% |
| Method: verb-first | `FetchAlerts`, `FetchDetections`, `FetchHosts`, `SetReady`, `SetNotReady`, `ListenAndServe` | 100% |
| Private helpers: camelCase | `alertToMap`, `safeString`, `filterNewAlerts`, `isCursorAhead`, `ensureForwardProgress`, `parseLogLevel`, `waitWithContext`, `isLoopback`, `enrichPayload`, `newPprofMux`, `getLimiter`, `readSecretFile`, `redactSecret` | 100% |
| Test helpers: camelCase | `newTestClient`, `strPtr`, `contains`, `searchString`, `waitForServer` | 100% |

### Variable Names

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| Short receivers | `c` (Collector), `a` (AlertCollector), `s` (HTTPSender, Server), `m` (MemoryStore) | 100% |
| Descriptive locals | `alertIDs`, `queryParams`, `entitiesParams`, `retryCount`, `retryDelay`, `baseDelay`, `maxDelay` | 100% |
| Error vars: `err`, `cerr` | `err` for primary, `cerr` for close errors | Consistent |

### Constant Names

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| camelCase for private | `readHeaderTimeout`, `readTimeout`, `writeTimeout`, `idleTimeout`, `defaultrequests`, `defaultburst` | Mostly consistent |
| ALL_CAPS env var strings | `"CROWDSTRIKE_CLIENT_ID"`, `"VECTOR_ENDPOINT"` | 100% in const block |

**Inconsistency found:** `defaultrequests` and `defaultburst` (health/server.go:21-22) are not separated by underscore or camelCase. Should be `defaultRequests` and `defaultBurst`. Minor naming inconsistency.

### Environment Variable Naming

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| `{SERVICE}_{FIELD}` | `CROWDSTRIKE_CLIENT_ID`, `VECTOR_ENDPOINT`, `XMP_SITE` | 100% |
| `{SERVICE}_{FIELD}_FILE` for secret files | `CROWDSTRIKE_CLIENT_ID_FILE`, `VECTOR_ENDPOINT_FILE` | 100% |
| Underscore separated, ALL_CAPS | All env vars | 100% |

---

## Module Organization Patterns

### Flat Internal Layout

```
internal/
  app/runner/     -- only nested package (orchestration)
  apperrors/      -- sentinel errors
  collector/      -- polling loop + alert collector
  config/         -- configuration
  crowdstrike/    -- API client
  health/         -- health server
  profiling/      -- pprof server
  sink/           -- HTTP delivery
  state/          -- persistence
```

**Pattern:** Flat `internal/` with one-level packages. No `pkg/` directory. No barrel exports. Each package has 1-3 files.

**Exception:** `internal/app/runner/` is the only two-level nesting, creating an `app` grouping. This is inconsistent with the flat pattern but benign.

### File Organization Within Packages

| Package | Files | Pattern |
|---------|-------|---------|
| collector | collector.go, alert_collector.go | Split by sub-domain |
| config | config.go, utils.go | Core + utilities |
| crowdstrike | api.go, source.go, api_test.go, README.md | Implementation + unused alt + test + docs |
| sink | sink.go (interface), http_sender.go (impl) | Interface + implementation split |
| state | store.go (all-in-one) | Single file for small package |
| health | server.go, server_test.go | Impl + test |
| profiling | pprof.go, pprof_test.go | Impl + test |
| apperrors | errors.go | Single file |

**Pattern:** Interface and implementation in the same package. No separate `interface.go` files except `sink/sink.go` which contains only the `Sender` interface. This is the "accept interfaces, return structs" Go idiom applied inconsistently -- `sink` separates the interface, other packages embed it in their main file.

### Test File Colocation

Tests are colocated with implementation files (same directory, `_test.go` suffix). This follows Go convention. Tests use the same package (not `_test` package suffix), giving access to internal types.

---

## Error Handling Patterns

### Pattern 1: Sentinel Errors with fmt.Errorf %w

**Location:** Throughout codebase
**Example:**
```go
return nil, fmt.Errorf("fetch alerts: %w", apperrors.ErrClientNotInitialized)
return fmt.Errorf("%w: attempts=%d", apperrors.ErrCollectorRetriesExceeded, retryCount-1)
```
**Consistency:** Used in crowdstrike, collector, sink, state. 11 of 17 sentinels are used this way. 6 sentinels are defined but never used.

### Pattern 2: Plain errors.New for construction guards

**Location:** crowdstrike/api.go:73-78
**Example:**
```go
return nil, errors.New("crowdstrike: client ID is required")
return nil, errors.New("crowdstrike: client secret is required")
```
**Note:** These construction errors do NOT use sentinel wrapping. They are plain error strings. This is inconsistent with the sentinel pattern used elsewhere.

### Pattern 3: errors.Join for validation aggregation

**Location:** config/config.go:361, 441
**Example:**
```go
var errs []error
errs = append(errs, fmt.Errorf("source.clientID is required"))
// ... more checks
return errors.Join(errs...)
```
**Consistency:** Used only in `Config.Validate()`. All other functions return on first error.

### Pattern 4: errors.Is for sentinel checking

**Location:** collector/collector.go:182
**Example:**
```go
case errors.Is(err, apperrors.ErrStateNotFound):
```
**Consistency:** Only one `errors.Is` check in production code. Tests use it more frequently.

### Pattern 5: Error context pattern

**Convention:** `"noun phrase: %w"` format
**Examples:** `"crowdstrike ping: %w"`, `"fetch alerts: %w"`, `"query alerts: %w"`, `"initialize sink: %w"`, `"load configuration: %w"`
**Consistency:** 95%+ consistent. One exception: `ensureForwardProgress` uses descriptive message without sentinel wrapping.

### Anti-pattern: ErrCursorRegression Unused

**Location:** apperrors/errors.go:20, alert_collector.go:149
**Issue:** `ErrCursorRegression` sentinel exists but `ensureForwardProgress` creates a plain `fmt.Errorf` without wrapping it. Callers cannot use `errors.Is(err, ErrCursorRegression)`.

---

## Test Patterns

### Pattern 1: Table-Driven Tests

**Location:** api_test.go:36-130, pprof_test.go:154-176
**Example:**
```go
tests := []struct {
    name      string
    client    *HTTPClient
    wantErr   bool
    errIs     error
    errSubstr string
}{...}
for _, tt := range tests {
    t.Run(tt.name, func(t *testing.T) {
```
**Consistency:** Used in Ping tests and isLoopback tests. NOT used in health tests (each is a separate function).

### Pattern 2: Direct Assertion (No Testify)

**Location:** All test files
**Pattern:** `if got != want { t.Errorf(...) }` or `if err == nil { t.Fatal(...) }`
**Consistency:** 100% -- no testify despite being an indirect dependency. All assertions use stdlib testing.

### Pattern 3: t.Parallel()

**Location:** api_test.go (both outer and inner tests), pprof_test.go:155 (isLoopback)
**Consistency:** Partial. Ping tests use `t.Parallel()`. Health tests do NOT (time-dependent rate limit tests). Pprof tests do NOT (port binding tests).

### Pattern 4: t.Setenv() for Environment Tests

**Location:** pprof_test.go
**Example:** `t.Setenv("ENABLE_PPROF", "1")`
**Consistency:** Used only in profiling tests. Config tests do not exist.

### Pattern 5: Manual Mocks (No Generated Mocks)

**Location:** api_test.go:17-25
**Pattern:** `mockAlertsClient` struct embedding `alerts.ClientService` and overriding one method
**Consistency:** Only mock in codebase. No `go:generate` directives. `mockgen` is declared in tools but unused.

### Pattern 6: Helper Functions

**Location:** api_test.go:27-33, 171-184
**Helpers:** `newTestClient`, `strPtr`, `contains`, `searchString`
**Note:** Custom `contains` and `searchString` reimplemented instead of using `strings.Contains`. This is a minor code smell but may be intentional to avoid importing `strings` in test file.

### Test Coverage Gaps

| Package | Has Tests | Test Count | Coverage Assessment |
|---------|-----------|------------|---------------------|
| crowdstrike | Yes | 10 funcs | Ping well-tested (6 cases). FetchAlerts/Detections/Hosts only nil-inner case. |
| health | Yes | 10 funcs | Liveness, readiness, rate limiting well-tested. No Shutdown test. |
| profiling | Yes | 8 funcs | Lifecycle well-tested. isLoopback table-driven. |
| collector | **No** | 0 | Most complex business logic. Zero tests. |
| config | **No** | 0 | 462 lines of loading/validation. Zero tests. |
| sink | **No** | 0 | HTTP delivery logic. Zero tests. |
| state | **No** | 0 | Persistence abstractions. Zero tests. |
| runner | **No** | 0 | Orchestration. Zero tests. |
| apperrors | **No** | 0 | Constants only. Tests would be trivial. |

**Test ratio:** 681 test lines / 2,245 production lines = 30.3% by line count. 28 test functions across 3 files. 5 of 9 packages have zero tests.

---

## Design Patterns

### Pattern 1: Dependency Injection via Interfaces

**Location:** collector/collector.go:22-27 (CrowdStrikeClient), sink/sink.go (Sender), state/store.go:19-26 (Store), health/server.go:25-28 (Reporter)
**Usage:** Collector accepts 4 interfaces. Runner wires concrete implementations.
**Consistency:** All major integration points use interfaces. Small utility types (QueryFingerprint, Cursor) do not.

### Pattern 2: Consumer-Defined Interfaces

**Location:** collector/collector.go:22-27 (`CrowdStrikeClient`)
**Pattern:** The collector package defines its own interface for the CrowdStrike client, identical to `crowdstrike.Client`. This follows the Go idiom "accept interfaces, return structs."
**Inconsistency:** Only the CrowdStrike client gets this treatment. The collector imports `sink.Sender`, `state.Store`, and `health.Reporter` directly from their defining packages.

### Pattern 3: Options Struct

**Location:** collector/collector.go:30-41
**Pattern:** `Options` struct with optional fields (Logger, Interval, Sink, HealthReporter). Zero values have sensible defaults.
**Usage:** `collector.New(cfg, client, store, opts Options)`
**Consistency:** Only used by Collector. Other constructors use direct parameters.

### Pattern 4: Config-Default-Override

**Location:** config/config.go:140-178 (DefaultConfig), config.go:184-355 (LoadFromEnvironment)
**Flow:** `DefaultConfig()` -> `LoadFromEnvironment(cfg)` -> `cfg.Validate()`
**Pattern:** Functional: returns new config, does not mutate global state.

### Pattern 5: Pointer-Safe Dereferencing

**Location:** crowdstrike/api.go:196-201
**Pattern:** `safeString(s *string) string` - nil-safe string pointer dereference
**Usage:** Used extensively in alertToMap for gofalcon SDK types which use pointer fields.

### Pattern 6: Composite Value Objects

**Location:** state/store.go:76-99
**Pattern:** Cursor, QueryFingerprint, BatchReceipt are plain structs with no methods (value objects). PollState is the aggregate that composes them.

### Pattern 7: State Machine (Implicit)

**Location:** collector/collector.go:99-167
**Pattern:** Collector lifecycle managed through imperative control flow (not explicit state type). States are: NotReady -> Initializing -> Ready -> Collecting -> Retrying -> Failed/Shutdown.
**Assessment:** State transitions are correct but encoded in control flow, not in a state machine abstraction.

---

## Anti-Patterns and Code Smells

### AP-001: Dead Code (source.go)

**Location:** crowdstrike/source.go (183 lines)
**Issue:** Entire file is unused. `Source`, `Record`, `AlertRecord`, `DetectionRecord`, `HostRecord`, `NewSource`, `NewSourceFromEnv`, `FetchRecords` are never called.
**Impact:** Maintenance burden, confusion about intended architecture.

### AP-002: Unused Sentinel Errors

**Location:** apperrors/errors.go
**Issue:** 6 of 17 sentinel errors are never referenced: `ErrCursorRegression`, `ErrSourceConfigMissing`, `ErrSourceRequestBuild`, `ErrSourceUnexpectedStatus`, `ErrSourceDecode`, `ErrConfigLoad`
**Impact:** Dead code. Some may have been intended for unimplemented features.

### AP-003: Unused Tool Dependencies

**Location:** tools/tools.go
**Issue:** `mockgen` and `stringer` imported but never used. No `//go:generate` directives exist.
**Impact:** Bloated tools module download.

### AP-004: Inconsistent Error Wrapping

**Location:** api.go:73-78 vs api.go:113
**Issue:** Construction guards use `errors.New()` while operation guards use `fmt.Errorf("...: %w", sentinel)`. `ensureForwardProgress` creates plain error despite `ErrCursorRegression` sentinel existing.

### AP-005: Missing Test Coverage for Business Logic

**Location:** collector/, config/, sink/, state/
**Issue:** The most critical business logic packages (collector, config) have zero test coverage. Only infrastructure packages (health, profiling) and one API test file exist.

### AP-006: Health Server Not Gracefully Shut Down

**Location:** runner.go:111-116
**Issue:** Health server goroutine is started but never shut down. `healthServer.Shutdown()` exists but is never called.

### AP-007: Hardcoded MemoryStore Despite Config Support

**Location:** runner.go:61
**Issue:** `state.NewMemoryStore()` hardcoded. Config supports file/memory selection. Helm chart sets `STATE_STORE_TYPE=file`.
**Impact:** State lost on every pod restart.

### AP-008: Constant Naming Inconsistency

**Location:** health/server.go:21-22
**Issue:** `defaultrequests` and `defaultburst` not camelCase (should be `defaultRequests`, `defaultBurst`)

### AP-009: Custom String Search in Tests

**Location:** api_test.go:173-184
**Issue:** `contains()` and `searchString()` reimplemented instead of using `strings.Contains()`
**Impact:** Minor code smell. No functional issue.

---

## Consistency Assessment

| Pattern | Applied Where | Consistency |
|---------|---------------|-------------|
| Interface-based DI | All 4 collector dependencies | HIGH |
| Sentinel errors with %w | 11 of 17 sentinels | MEDIUM (6 unused, 1 unwrapped) |
| Table-driven tests | api_test.go, pprof_test.go | PARTIAL (health tests don't use it) |
| t.Parallel() | api_test.go | LOW (other test files don't use it) |
| Constructor `New*` | All packages | HIGH |
| Config env var naming | All 30+ env vars | HIGH |
| JSON structured logging | All log calls | HIGH |
| File-backed secret pattern | CrowdStrike + Sink creds | HIGH |
| gofumpt formatting | Enforced by pre-commit + lint | HIGH (toolchain enforced) |
| Import grouping | stdlib / third-party / internal | HIGH (enforced by goimports in golangci.yml) |

---

## Delta Summary
- New items added: 9 anti-patterns cataloged, 7 design patterns documented, naming convention audit across 6 categories, test coverage assessment (30.3% by lines, 5/9 packages untested), consistency matrix
- Existing items refined: Broad sweep mentioned some patterns in passing; now systematically cataloged with locations and consistency ratings
- Remaining gaps: None significant. All Go source, test, config, and CI files analyzed.

## Novelty Assessment
Novelty: SUBSTANTIVE
The systematic pattern catalog reveals several new findings: (1) AP-009 custom string search in tests, (2) the import grouping enforcement via golangci.yml was not previously documented, (3) AP-008 constant naming inconsistency was not caught, (4) the test coverage metric (30.3% by lines, 5/9 packages untested) is a precise quantification not previously available, (5) the consumer-defined interface pattern is applied inconsistently (only for CrowdStrike, not for Sender/Store/Reporter). These findings inform the Rust rewrite's convention decisions.

## Convergence Declaration
Another round needed -- should audit for any coding patterns in the Helm templates and CI workflows not yet cataloged.

## State Checkpoint
```yaml
pass: 5
round: 1
status: complete
files_scanned: all
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
```
