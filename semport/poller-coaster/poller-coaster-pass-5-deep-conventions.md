# Pass 5 Deep: Conventions & Patterns -- Round 1

**Project:** poller-coaster
**Date:** 2026-04-13
**Basis:** All source files, test files, CI/CD, Helm chart, config files, cross-referenced with broad sweep conventions section and Phase A outputs

---

## Code Organization Conventions

### Package Structure

| Convention | Consistency | Notes |
|-----------|------------|-------|
| `internal/` for all non-main packages | 100% | No `pkg/` directory, everything is internal |
| Package per concern | 100% | armis, collector, config, health, profiling, sink, state, apperrors |
| `cmd/` for alternative entrypoints | 100% | cmd/collector/main.go |
| Test files co-located with source | 100% | `*_test.go` in same package |
| No barrel exports (index files) | N/A | Go does not use barrel exports |
| One sub-collector per file | 100% | alert_collector.go, activity_collector.go, etc. |
| Interface defined by consumer | ~75% | collector defines SearchClient (mirrors armis.Client), but sink.Sender defined in sink package |

### File Naming

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| snake_case.go | file_store.go, http_sender.go, risk_factor_collector.go | 100% |
| _test.go suffix | collector_test.go, config_test.go | 100% |
| Interface file separate from impl | sink.go (interface) + http_sender.go (impl) | Partial -- only sink does this; state has Store interface in store.go alongside MemoryStore |
| Package doc comment in first file | All packages | 100% |

### Directory Layout

```
.
+-- main.go                          # Root entrypoint
+-- cmd/collector/main.go            # Docker entrypoint (identical)
+-- internal/
|   +-- app/runner/runner.go         # Single file orchestration
|   +-- apperrors/errors.go          # Single file, all sentinels
|   +-- armis/api.go                 # Single file, SDK wrapper
|   +-- collector/
|   |   +-- collector.go             # Main orchestrator + types
|   |   +-- {source}_collector.go    # 7 per-source files
|   |   +-- {source}_collector_test.go  # 5 per-source test files
|   |   +-- collector_test.go        # Orchestrator tests
|   +-- config/
|   |   +-- config.go                # Main config (large: defaults + loading + validation)
|   |   +-- utils.go                 # ValidateConfig + redactSecret
|   |   +-- config_test.go           # 30+ tests
|   +-- health/
|   |   +-- server.go                # Health server + rate limiting
|   |   +-- server_test.go           # 11 tests
|   +-- profiling/
|   |   +-- pprof.go                 # Pprof server
|   |   +-- pprof_test.go            # 8 tests
|   +-- sink/
|   |   +-- sink.go                  # Sender interface
|   |   +-- http_sender.go           # Implementation
|   +-- state/
|   |   +-- store.go                 # Store interface + types + MemoryStore
|   |   +-- file_store.go            # FileStore implementation
|   |   +-- file_store_test.go       # FileStore tests
|   |   +-- store_test.go            # trimReceipts tests
+-- tools/
|   +-- tools.go                     # Build-tagged tool pins
|   +-- go.mod / go.sum              # Tool dependencies
+-- deploy/helm/poller-coaster/      # Helm chart
+-- scripts/                         # Dev scripts
+-- docs/                            # Documentation
```

---

## Naming Conventions

### Type Names

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| PascalCase structs | AlertCollector, DevicePollState, EnrichedPayload | 100% |
| Config suffix for config types | ArmisConfig, SinkConfig, CollectorConfig, StateConfig | 100% |
| Store suffix for persistence | FileStore, MemoryStore, AlertStore, DeviceStore | 100% |
| Cursor suffix for cursor types | AlertCursor, DeviceCursor, VulnerabilityCursor | 100% |
| PollState suffix for state types | AlertPollState, ConnectionPollState | 100% |
| BatchReceipt suffix for receipts | AlertBatchReceipt, DeviceBatchReceipt | 100% |
| Collector suffix for collectors | AlertCollector, VulnerabilityCollector | 100% |

### Function Names

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| NewXxx constructors | NewFileStore, NewHTTPClient, NewHTTPSender, NewServer | 100% |
| WithXxx options | WithMaxReceipts, WithRateLimitConfig | 100% |
| Send prefix for delivery | SendAlert, SendDevice, SendVulnerability | 100% |
| Load/Save for persistence | Load, Save, LoadDevice, SaveDevice | ~95% (AlertStore uses unprefixed Load/Save) |
| is/ensure prefix for checks | isCursorAhead, ensureXxxForwardProgress | 100% |
| parse prefix for parsing | parseXxxTimestamp, parseLogLevel | 100% |
| filter prefix for filtering | filterNewXxx | 100% |
| xxxResultCursor for cursor extraction | alertResultCursor, deviceResultCursor | 100% |
| handle prefix for HTTP handlers | handleLiveness, handleReadiness | 100% |

### Variable/Constant Names

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| SCREAMING_SNAKE for env vars | ARMIS_API_URL, COLLECTOR_MAX_RETRIES | 100% |
| camelCase for local vars | alertSink, healthServer, armisClient | 100% |
| Err prefix for sentinel errors | ErrStateNotFound, ErrCursorRegression | 100% |
| lowercase unexported constants | defaultrequests, defaultburst | 100% (but slightly unconventional -- usually camelCase) |

### JSON/Wire Format

| Convention | Examples | Consistency |
|-----------|---------|-------------|
| snake_case for record types | armis_alert, armis_device, armis_audit_log | 100% |
| snake_case for JSON keys | record_type, cluster_name, node_name | 100% |
| omitempty on optional fields | XMPMetadata fields | 100% |

---

## Error Handling Patterns

### Pattern 1: Sentinel Error Wrapping

```go
fmt.Errorf("%w: %v", apperrors.ErrSinkDelivery, err)
```

- Wraps with sentinel using `%w` for errors.Is() matching
- Inner error uses `%v` (NOT `%w`), so only sentinel is matchable
- **Consistency:** 100% across sink, armis, collector packages

### Pattern 2: Validation Error Aggregation

```go
var errs []error
if condition { errs = append(errs, fmt.Errorf("...")) }
return errors.Join(errs...)
```

- Used in config.Validate()
- Returns nil when errs is empty (no error)
- **Consistency:** Used only in config.Validate(); not adopted elsewhere

### Pattern 3: Context Wrapping in Constructors

```go
if apiKey == "" { return nil, errors.New("armis: api key is required") }
```

- Constructors (NewHTTPClient, NewHTTPSender) validate inputs and return plain errors
- NOT wrapped with sentinel errors (unlike runtime methods)
- **Consistency:** 100% across constructors

### Inconsistency: Forward Progress Error Handling

| Collectors | Error Pattern |
|-----------|--------------|
| Connection, Device, Vulnerability | `fmt.Errorf("%w: ...", apperrors.ErrCursorRegression)` |
| Alert, Activity, AuditLog, RiskFactor | `fmt.Errorf("cursor did not advance: ...")` (plain, no sentinel) |

This means `errors.Is(err, ErrCursorRegression)` only works for 3 of 7 collectors. The other 4 produce un-matchable errors.

---

## Testing Patterns

### Pattern 1: Table-Driven Tests

- Used in: file_store_test.go (trimReceipts), config_test.go (validation)
- Each test case is a struct with name, input, expected output
- Subtest per case via t.Run(tc.name, ...)

### Pattern 2: Mock Interfaces in Test Files

- Each test file defines its own mock implementations
- Mock SearchClient in collector tests
- Mock Sender in collector tests
- No shared mock package or mock generation tool
- **Consistency:** 100% across test files -- all define local mocks

### Pattern 3: Parallel Tests

- `t.Parallel()` called in every test function
- **Consistency:** 100% (all tests are parallel)

### Pattern 4: Test Naming

| Convention | Examples |
|-----------|---------|
| TestType_Method_Scenario | TestDeviceCollector_Collect_NoResults |
| TestNewType_Scenario | TestNewFileStore_CreatesDirectory |
| TestType_Scenario | TestConfig_Validate_MultipleErrors |

**Consistency:** ~95% -- most follow this pattern, minor variations exist.

### Pattern 5: TempDir for State Tests

- file_store_test.go uses `t.TempDir()` for all file-based tests
- Ensures no test pollution between parallel tests
- **Consistency:** 100% in state tests

### Test Coverage Gaps

| Component | Has Tests? | Coverage |
|-----------|-----------|----------|
| AlertCollector | NO | 0 dedicated tests (relies on code symmetry with DeviceCollector) |
| ActivityCollector | NO | 0 dedicated tests |
| HTTPSender | NO | No dedicated tests (only tested indirectly via collector tests) |
| runner.go | NO | No tests (orchestration logic untested) |
| armis/api.go | NO | No tests (thin SDK wrapper) |

---

## Design Patterns

### Strategy Pattern

- **Store interface** with FileStore + MemoryStore implementations
- **Sender interface** with HTTPSender implementation
- **Client interface** with HTTPClient implementation
- **Reporter interface** with Server implementation
- Chosen at runtime by runner based on config

### Functional Options

- **FileStoreOption** type: `type FileStoreOption func(*FileStore)`
- `WithMaxReceipts(n int)` is the only option
- **WithRateLimitConfig** on health.Server (builder pattern, not functional option)
- **Inconsistency:** FileStore uses functional options, health.Server uses builder-style `.WithRateLimitConfig()` returning *Server

### Interface Segregation (ISP)

- Store composes 7 sub-interfaces: AlertStore, ActivityStore, AuditLogStore, etc.
- Each sub-interface has exactly 2 methods: Load + Save
- **Naming inconsistency:** AlertStore uses `Load()`/`Save()` (unprefixed); others use `LoadXxx()`/`SaveXxx()`

### Null Object Pattern

- Sink is nil-safe: all collector code checks `if c.sink != nil` before calling Send
- Reporter is nil-safe: collector checks `if c.reporter != nil` before calling SetReady/SetNotReady
- **Consistency:** 100% -- all optional dependencies are nil-checked

### Composite Cursor

- Every cursor is `(Timestamp, TypeSpecificID)` with lexicographic string comparison as tiebreaker
- **Consistency:** 100% across all 7 data sources

### Dependency Injection (Constructor Injection)

- All components receive dependencies via constructor parameters
- No service locator, no global state
- Logger passed as parameter to all constructors
- **Consistency:** 100% -- no component creates its own dependencies (except Logger fallback to stdout in armis.go and http_sender.go when nil)

---

## Anti-Patterns and Code Smells

### 1. Seven-Way Code Duplication (CRITICAL)

The 7 collectors are **structurally identical** with the same algorithm, differing only in:
- Timestamp field selection (1-3 fields, source-specific)
- ID extraction chain (2-4 fields, source-specific)
- Record type string ("armis_alert", "armis_device", etc.)
- Forward progress error handling (sentinel vs plain -- likely accidental)

Each collector has ~200 lines. The total duplication is ~1,200 lines that could be ~200 lines with a generic pattern.

### 2. Duplicate Entrypoints

main.go and cmd/collector/main.go are byte-for-byte identical. One should import from the other, or there should be a single entrypoint.

### 3. Duplicate Interface Definitions

`armis.Client` and `collector.SearchClient` have the same signature but are defined independently. This is Go convention (consumer-defined interfaces), but the duplication means changes to the SDK interface must be updated in two places.

### 4. Large Config File

config.go contains DefaultConfig + LoadFromEnvironment + Validate -- three distinct responsibilities in one file (~700 lines estimated). Could be split into config_defaults.go, config_loader.go, config_validate.go.

### 5. No Sink Tests

HTTPSender has no dedicated unit tests. It is only tested indirectly via the collector tests' mock Sender. The enrichment logic, error body handling, and URL validation are untested.

### 6. Inconsistent Duration Parsing

Timeout env vars (ARMIS_API_TIMEOUT, VECTOR_TIMEOUT_SECONDS) accept both Go duration strings and plain integers. Interval/delay env vars (COLLECTOR_INTERVAL, COLLECTOR_RETRY_*) accept only Go duration strings. This inconsistency will confuse operators.

### 7. Missing Limit Validation (Bug)

AuditLogLimit and RiskFactorLimit are NOT validated by config.Validate(). A limit of 0 would pass validation and silently disable hasMore pagination for those sources.

### 8. Rate Limiter Memory Leak

health.Server.limiters map grows without bound. Each unique IP address creates a new rate.Limiter entry that is never evicted. Under high-cardinality IP traffic (e.g., behind a load balancer), this could leak memory.

---

## Consistent vs. Inconsistent Patterns

### Fully Consistent (applied everywhere)

- snake_case file names
- PascalCase types
- Constructor injection
- t.Parallel() in all tests
- Local mock definitions in test files
- nil-safe optional dependencies
- Sentinel error wrapping with %w/%v pattern
- Composite cursor design

### Partially Consistent (applied sporadically)

| Pattern | Applied In | Missing From | Impact |
|---------|-----------|-------------|--------|
| ErrCursorRegression sentinel | Connection, Device, Vulnerability | Alert, Activity, AuditLog, RiskFactor | Error matching broken for 4/7 sources |
| Functional options | FileStore | Health Server (uses builder pattern) | Minor style inconsistency |
| Prefixed Load/Save methods | 6/7 sub-stores | AlertStore (uses unprefixed Load/Save) | API asymmetry |
| Dedicated unit tests | 5/7 collectors, FileStore, config, health, profiling | AlertCollector, ActivityCollector, HTTPSender, runner | Coverage gaps |
| Duration integer fallback | ARMIS_API_TIMEOUT, VECTOR_TIMEOUT_SECONDS | COLLECTOR_INTERVAL, COLLECTOR_RETRY_* | Operator confusion |
| Limit validation | 5/7 sources | AuditLog, RiskFactor | Bug: limit=0 passes validation |

---

## Delta Summary

- New items added: Complete convention catalog (6 categories), 8 anti-patterns documented, consistency assessment table, test pattern analysis with coverage gaps
- Existing items refined: Error handling patterns classified into 3 distinct patterns, naming conventions exhaustively documented
- Remaining gaps: None that would change the convention model

## Novelty Assessment

Novelty: SUBSTANTIVE

This round provides the first systematic catalog of 8 anti-patterns (vs. the broad sweep's mention of only the 7-way duplication), identifies the rate limiter memory leak (new), documents the AlertStore naming asymmetry, maps the exact consistency/inconsistency boundaries for all patterns, and provides the test coverage gap analysis. These findings change how you would spec the system's conventions and identify what needs to be cleaned up in a port.

## Convergence Declaration

Another round needed -- should verify: (1) whether stretchr/testify is actually used (affects testing pattern documentation), (2) any conventions in the Helm chart templates, (3) hallucination audit of all claimed line numbers and patterns.

## State Checkpoint

```yaml
pass: 5
round: 1
status: complete
files_scanned: 48
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
next_action: round 2 -- hallucination audit, verify testify usage, check Helm conventions
```
