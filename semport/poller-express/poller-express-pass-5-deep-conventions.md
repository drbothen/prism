# Pass 5 Deep: Convention & Pattern Catalog -- poller-express (Round 1)

## Naming Conventions

### Package Naming
- **snake_case** for multi-word package paths: `internal/app/runner`, `internal/apperrors`
- Single-word packages preferred: `state`, `sink`, `health`, `config`, `collector`, `asset`
- `pkg/` for public/shared code: `pkg/cyberint`, `pkg/validate`
- `internal/` for private code: all application logic

### File Naming
- **snake_case** for Go files: `alert_collector.go`, `http_sender.go`, `cyberint_time.go`
- Test files: `*_test.go` colocated with source
- No `_internal_test.go` (white-box) pattern -- all tests are in the same package

### Type Naming
- **PascalCase** for exported types: `MemoryStore`, `HTTPSender`, `AssetCollector`
- Acronyms preserved: `HTTP`, `API`, `XMP`, `IOC`, `TLS`, `URL`
- Config structs: `{Domain}Config` (e.g., `CyberintConfig`, `SinkConfig`, `XMPConfig`, `LoggingConfig`, `CollectorConfig`, `AssetConfig`)
- State types: `PollState`, `AssetPollState`, `BatchReceipt`, `AssetBatchReceipt`

### Variable Naming
- Short variable names in tight scopes: `cfg`, `ctx`, `err`, `buf`, `enc`
- `*Env` suffix for environment variable constants: `cyberintURLEnv`, `sinkEndpointEnv`
- `*FileEnv` suffix for file-backed secret env vars: `cyberintURLFileEnv`, `sinkEndpointFileEnv`
- Collector options: `alertCollectorOpts`, `assetCollectorOpts`

### Error Naming
- `Err` prefix for sentinel errors: `ErrStateNotFound`, `ErrCursorRegression`
- Domain-qualified: `ErrCyberInt*`, `ErrSink*`, `ErrCollector*`
- **Inconsistency**: `ErrCyberInt` vs. type name `Cyberint` -- the sentinel errors use "CyberInt" (capital I) while the API client package uses "cyberint" (lowercase i). This casing inconsistency exists in 7 error names.

### Function Naming
- Constructors: `New{Type}()` -- `NewMemoryStore()`, `NewHTTPSender()`, `NewServer()`
- State queries: `Is{Condition}()` -- `IsZero()`
- Actions: verb-based -- `Execute()`, `Run()`, `Send()`, `Load()`, `Save()`
- Private helpers: camelCase -- `collectOnce()`, `filterNewAlerts()`, `enrichPayload()`, `ensureForwardProgress()`, `extractCustomerIDFromURL()`

---

## Module Organization Patterns

### Standard Go Layout
```
cmd/          -- entry points (one binary)
internal/     -- private application code
  app/        -- application-level orchestration
  {domain}/   -- domain packages
pkg/          -- public libraries
deploy/       -- deployment configs
scripts/      -- development scripts
tools/        -- Go tool dependencies
docs/         -- generated API docs
```

### Feature-by-Package
Each package owns one concept: `collector` (polling logic), `state` (persistence), `sink` (delivery), `health` (probes), `config` (loading), `asset` (API client). No barrel exports.

### Interface Definition Pattern
Interfaces are defined in the **consumer** package, not the provider:
- `sink.Sender` in `internal/sink/sink.go` -- consumed by collector, implemented by HTTPSender
- `state.Store` and `state.AssetStore` in `internal/state/store.go` -- consumed by collector, implemented by MemoryStore
- `health.Reporter` in `internal/health/server.go` -- consumed by collector, implemented by Server

This follows the Go proverb: "Accept interfaces, return structs."

### Tools Module Pattern
Development tools are in a separate Go module (`tools/go.mod`) to avoid polluting the main module's dependency tree. Tools are invoked via `go run -modfile=tools/go.mod`.

---

## Error Handling Patterns

### Pattern 1: Sentinel Errors with Wrapping
```go
// Definition (apperrors/errors.go)
var ErrSinkDelivery = errors.New("sink delivery failed")

// Usage (sink/http_sender.go)
return fmt.Errorf("%w: status=%d body=%s", apperrors.ErrSinkDelivery, resp.StatusCode, body)

// Matching (collector)
if errors.Is(err, apperrors.ErrSinkDelivery) { ... }
```

Applied consistently across all packages. No raw `errors.New()` outside `apperrors`.

### Pattern 2: Aggregated Validation Errors
```go
// config.go
var errs []error
if ... { errs = append(errs, fmt.Errorf("...")) }
if ... { errs = append(errs, fmt.Errorf("...")) }
return errors.Join(errs...)
```

Used only in `Config.Validate()`. Reports ALL problems, not just the first.

### Pattern 3: Error Wrapping Chain
All intermediate error handlers wrap with context:
```go
return cfg, fmt.Errorf("load Cyberint base URL secret: %w", err)
```

This creates readable chains like: `load Cyberint base URL secret: open /path: no such file`

### Pattern 4: No Panics
Zero `panic()` calls in the entire hand-written codebase. All errors are returned, not panicked.

### Pattern 5: Deferred Close Error Handling
```go
defer validate.Check(resp.Body.Close)
// or
defer func() {
    if cerr := resp.Body.Close(); cerr != nil {
        s.logger.Warn("failed to close...", "error", cerr)
    }
}()
```

Two patterns coexist: `validate.Check` (logs via slog) and inline deferred closers (log via charmbracelet/log). **Inconsistency**: the logging framework differs between these two patterns.

---

## Test Patterns

### Pattern 1: Table-Driven Tests
```go
tests := []struct {
    name     string
    input    ...
    expected ...
}{
    {"case one", ..., ...},
    {"case two", ..., ...},
}
for _, tt := range tests {
    t.Run(tt.name, func(t *testing.T) { ... })
}
```

Used consistently across all test files. Test names are descriptive phrases.

### Pattern 2: httptest.NewServer
```go
server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
    // verify request
    // return response
}))
defer server.Close()
```

Used in: asset/client_test.go, sink/http_sender_test.go, health/server_test.go, profiling/pprof_test.go.

### Pattern 3: Inline Mock Types
```go
type mockSink struct {
    records []string
    err     error
}
func (m *mockSink) Send(ctx context.Context, record any, recordID, recordType string) error {
    m.records = append(m.records, recordID)
    return m.err
}
```

No mock generation tools (no gomock, no mockery). All mocks are hand-written and defined inline in test files.

### Pattern 4: Testify Assertions
```go
require.NoError(t, err)
assert.Equal(t, expected, actual)
assert.Contains(t, errString, "substring")
assert.True(t, condition)
```

`require` for fatal preconditions, `assert` for non-fatal checks. Consistent usage.

### Pattern 5: Benchmark Tests
```go
func BenchmarkEnrichPayload(b *testing.B) {
    for i := 0; i < b.N; i++ { ... }
}
```

Only one benchmark exists: `BenchmarkEnrichPayload` in `http_sender_test.go`. Performance-critical hot path.

### Test File to Source File Ratio
| Source File | Test File | Approximate Tests |
|-------------|-----------|-------------------|
| alert_collector.go | alert_collector_test.go | ~5 tests |
| asset_collector.go | asset_collector_test.go | ~15 tests |
| client.go (asset) | client_test.go | ~9 tests |
| config.go | config_test.go | ~6 tests |
| server.go (health) | server_test.go | ~10 tests |
| http_sender.go | http_sender_test.go | ~6 tests |
| pprof.go | pprof_test.go | ~5 tests |

**Untested source files**: runner.go, errors.go, sink.go (interface only), store.go (only transitively), utils.go (validate), utils.go (config).

---

## Design Patterns

### Pattern: Options Struct for Configuration
```go
type Options struct {
    Logger         *log.Logger
    Interval       time.Duration
    Sink           sink.Sender
    HealthReporter health.Reporter
}
```

Used for both `collector.Options` and `collector.AssetCollectorOptions`. These are separate types (not unified) even though they have identical fields.

**Inconsistency**: The two Options types are structurally identical but defined as separate types. A single shared Options type would reduce duplication.

### Pattern: Factory Functions
- `state.NewMemoryStore()` -- zero-config constructor
- `sink.NewHTTPSender(cfg, xmpCfg, logger)` -- validated constructor that can return error
- `health.NewServer(addr)` -- constructor with default fallback
- `collector.New(cfg, client, store, opts)` -- dependency injection constructor
- `collector.NewAssetCollector(cfg, client, store, opts)` -- dependency injection constructor

### Pattern: Builder/Fluent Config
```go
healthServer := health.NewServer(cfg.Collector.HealthAddr)
// used in tests:
server.WithRateLimitConfig(config)
```

Minimal fluent API (only one method chain).

### Pattern: Custom RoundTripper for Cross-Cutting Auth
```go
type cookieTransport struct {
    apiKey string
    base   http.RoundTripper
}
```

Classic Go pattern for middleware on HTTP clients. Applied correctly (delegates to base, adds cookie before delegation).

### Pattern: Ticker-Based Polling Loop
```go
ticker := time.NewTicker(interval)
defer ticker.Stop()
for {
    select {
    case <-ctx.Done():
        return ctx.Err()
    case <-ticker.C:
        // collectOnce()
    }
}
```

Shared pattern between alert and asset collectors.

### Pattern: Double-Check Locking
```go
func (s *Server) getLimiter(ip string) *rate.Limiter {
    s.limitersMu.RLock()
    limiter, exists := s.limiters[ip]
    s.limitersMu.RUnlock()
    if !exists {
        s.limitersMu.Lock()
        limiter, exists = s.limiters[ip]  // double-check
        if !exists { ... }
        s.limitersMu.Unlock()
    }
    return limiter
}
```

Textbook implementation in health server rate limiter.

---

## Anti-Patterns and Code Smells

### Anti-Pattern 1: Duplicate Collector Implementations
The alert collector and asset collector have nearly identical structures:
- Both have `Run()` with the same ticker/retry loop
- Both have `initializeState()` with the same load/bootstrap/fingerprint logic
- Both have `collectOnce()` with fetch/sort/filter/send/advance pattern
- Both have `ensureForwardProgress()` with similar (but not identical) comparison logic

The code duplication is significant. A generic collector with type parameters would eliminate most of it.

### Anti-Pattern 2: Stringly-Typed Record Types
Record types are passed as strings (`"cyberint_alert"`, `"cyberint_asset"`) through the sink interface. No type safety, no enum.

### Anti-Pattern 3: Unbounded Rate Limiter Map
The health server's per-IP rate limiter map (`map[string]*rate.Limiter`) grows without bound. There is no eviction policy. In a long-running deployment hit by many unique IPs (e.g., scanning), this could cause memory growth. The map only grows, never shrinks.

### Anti-Pattern 4: Magic Numbers
- Page sizes: `100` (alerts) and `1000` (assets) are hardcoded in collector code, not configurable
- `2048` bytes for response body in sink error messages
- `10 * 1024 * 1024` (10 MiB) for asset response limit

### Code Smell: Duplicate Options Types
`collector.Options` and `collector.AssetCollectorOptions` have identical fields but are separate types.

---

## Consistency Assessment

| Pattern | Consistency | Notes |
|---------|-------------|-------|
| Sentinel error wrapping | HIGH | Applied everywhere except config (which sometimes uses bare fmt.Errorf) |
| Table-driven tests | HIGH | All test files use this pattern |
| httptest for HTTP tests | HIGH | All HTTP test cases |
| Inline mocks (no generation) | HIGH | Zero generated mocks |
| Structured logging | MEDIUM | charmbracelet/log everywhere EXCEPT pkg/validate (uses slog) |
| Deferred close handling | MEDIUM | Two different patterns coexist (validate.Check vs inline) |
| Options struct injection | MEDIUM | Used for collectors but not for sink or health |
| Error naming (CyberInt) | LOW | Sentinel errors use "CyberInt" but package/type is "cyberint" |
| Config struct naming | HIGH | Consistent `{Domain}Config` pattern |
| `nolint` directives | HIGH | Used sparingly, always with justification comments |

---

## Delta Summary
- New items added: 5 naming convention categories, 6 error handling patterns, 5 test patterns, 6 design patterns, 4 anti-patterns, consistency assessment table
- Existing items refined: Expanded from broad sweep's 6 pattern bullets to comprehensive catalog with code examples
- Remaining gaps: Should verify whether the alert and asset collector duplication is exact or has meaningful divergences. Should count exact test case numbers.

## Novelty Assessment
Novelty: SUBSTANTIVE
Key discoveries: (1) the CyberInt vs cyberint naming inconsistency across 7 error names, (2) the duplicate Options types, (3) the unbounded rate limiter map as a memory leak risk, (4) the logging framework split between charmbracelet/log and slog, (5) the deferred close dual-pattern inconsistency, (6) interface-in-consumer pattern verified as consistent, (7) the alert/asset collector code duplication as the primary anti-pattern. These inform the port's convention choices.

## Convergence Declaration
Another round needed -- should verify the exact scope of collector duplication and identify any subtle differences between alert and asset paths.

## State Checkpoint
```yaml
pass: 5
round: 1
status: complete
timestamp: 2026-04-13T23:45:00Z
novelty: SUBSTANTIVE
```
