# Pass 4 Deep: NFR Catalog -- Round 1

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 1

---

## NFR-1: Security

### NFR-1.1: TLS Configuration
- **Minimum TLS 1.2** enforced via `transport.DefaultConfig().MinTLSVersion = tls.VersionTLS12`
- Applied to both Claroty API client and sink HTTP client
- `InsecureSkipVerify` available in transport config but **not exposed via environment variables** -- requires code change
- When `InsecureSkipVerify` is set, a warning is logged: "TLS certificate verification disabled"
- `//nolint:gosec // G402` suppression with explanatory comment

### NFR-1.2: Credential Management
- **Secret file pattern**: Every sensitive config supports `*_FILE` variant for Kubernetes secret mounts
- File variant takes precedence over direct env var
- `readSecretFile()` reads + trims whitespace, returns empty string for non-existent files (graceful degradation)
- Bearer token is trimmed of whitespace/newlines on construction (`strings.TrimSpace`, tested in `TestNewHTTPClient_TrimsToken`)
- Credentials never logged (log statements use "base_url" for Claroty, not the token)

### NFR-1.3: Runtime Security
- Docker: distroless base image, nonroot user (UID 65532)
- Kubernetes: `readOnlyRootFilesystem: true`, `allowPrivilegeEscalation: false`, `capabilities.drop: ALL`
- PVC has `fsGroup: 65532` for write access as nonroot
- pprof `/debug/pprof/cmdline` explicitly blocked (returns 404) to prevent process argument exposure
- pprof non-loopback binding triggers warning log

### NFR-1.4: CI Security
- `step-security/harden-runner` with egress audit on all CI jobs
- All GitHub Actions pinned to commit SHAs (not floating tags)
- gosec, govulncheck, staticcheck run on every PR and daily cron
- Security scan results are **non-blocking** (informational PR comments)
- `check-added-large-files` pre-commit hook (500KB limit)

### NFR-1.5: Missing Security Controls
- **No rate limiting** toward Claroty API -- relies solely on page size (100) and polling interval (30s)
- **No HTTP 429 handling** -- retries are generic (exponential backoff at collection loop level)
- **No request signing or HMAC** -- relies on Bearer token alone
- **No credential rotation support** -- token is read once at startup; requires pod restart for rotation
- **No audit logging of credential access** -- only logs "initializing claroty client" with base_url

---

## NFR-2: Reliability

### NFR-2.1: Retry Strategy
- Exponential backoff: base 2s, max 30s, multiplier 2x
- Max retries: 5 (configurable, 0 = unlimited)
- On success: reset retry count and delay to base
- Retries are context-aware (`waitWithContext` ensures cancellation during sleep)
- **Health readiness tracks retry state**: `SetNotReady()` on error, `SetReady()` on success

### NFR-2.2: Atomic State Persistence
- Write-to-temp + fsync + rename pattern in `FileStore.persist()`
- Temp file pattern: `.poller-state-*.tmp` in same directory as state file
- Cleanup: `defer os.Remove(tempFile)` on failure paths
- Prevents partial writes from corrupting state on crash

### NFR-2.3: Forward Progress Enforcement
- 9 separate `ensure*ForwardProgress` functions
- Compare new cursor vs. previous cursor on primary key (timestamp/offset, then ID fields)
- Prevents infinite re-fetch loops when API returns identical pages
- **Note**: `ErrCursorRegression` sentinel exists but is unused -- forward progress errors use plain `fmt.Errorf`

### NFR-2.4: At-Least-Once Delivery
- Records sent to sink individually, in order
- Cursor only advances after ALL records in batch are delivered
- On mid-batch failure: cursor not saved, entire batch re-fetched on retry
- **Implication**: Duplicates possible on restart after partial batch delivery

### NFR-2.5: Graceful Shutdown
- `signal.NotifyContext` for SIGTERM/SIGINT
- Health server: 5s shutdown window
- pprof server: 5s shutdown window (root main.go only)
- In-flight HTTP requests cancelled via context propagation

### NFR-2.6: Query Fingerprint Drift Detection
- SHA-256 hash of `sorted(fields) + "|" + limit`
- Compared on startup; mismatch is fatal (`ErrQueryFingerprintMismatch`)
- Prevents silently using stale cursors when field lists change
- User must delete state file to recover

### NFR-2.7: Missing Reliability Controls
- **No circuit breaker** -- continuous retry against down APIs
- **No dead letter queue** -- failed records are retried, not quarantined
- **No idempotency tokens** -- duplicates must be handled downstream
- **No health check of Claroty API** before starting collection
- **Run() retry loop has no test coverage** (confirmed in Pass 3 R2)

---

## NFR-3: Performance

### NFR-3.1: Connection Pooling
- `MaxIdleConns: 100` -- global idle pool
- `MaxIdleConnsPerHost: 10` -- per-host idle connections
- `MaxConnsPerHost: 20` -- per-host total connections
- `IdleConnTimeout: 90s` -- matches typical keep-alive settings
- HTTP/2 enabled (`ForceAttemptHTTP2: true`)
- KeepAlive: 30s on dialer
- Shared transport config between Claroty client and sink

### NFR-3.2: Timeout Architecture
- Claroty API client: 30s overall timeout (hardcoded in `runner.go`)
- Sink client: 15s overall timeout (configurable via `VECTOR_TIMEOUT_SECONDS`)
- Transport-level: Dial 10s, TLS handshake 10s, response header 30s, expect-continue 1s
- Health server: `ReadHeaderTimeout: 10s`
- Pprof server: Read 10s, Write 120s, ReadHeader 5s, Idle 60s

### NFR-3.3: Page Size
- All 9 sources default to 100 records per page
- Not configurable via environment variables (hardcoded in `DefaultConfig`)
- `hasMore` optimization: when batch size equals limit, skip ticker wait and fetch immediately

### NFR-3.4: Serialization
- `json.NewEncoder(&buf).Encode(payload)` with trailing newline trim for sink enrichment
- Per-record HTTP POST -- no batching at sink level
- Benchmark tests exist: `http_client_bench_test.go`, `file_store_bench_test.go`, `http_sender_bench_test.go`

### NFR-3.5: Missing Performance Controls
- **No connection reuse metrics** -- no visibility into pool utilization
- **No request/response size limits** beyond `io.LimitReader(resp.Body, 2048)` for error responses
- **Sequential polling** -- all 9 sources polled in sequence within `collectOnce()`; no parallel collection
- **Per-record HTTP POST** to sink -- no batching; for high-volume sources this could be a bottleneck

---

## NFR-4: Observability

### NFR-4.1: Structured Logging
- `charmbracelet/log` with JSON formatter
- Timestamps always enabled
- Configurable level: INFO (default), DEBUG, TRACE (maps to DEBUG)
- Key structured fields: `type`, `endpoint`, `id`, `error`, `base_url`, `path`, `max_receipts`

### NFR-4.2: Health Endpoints
- `/health` (liveness) -- returns 200 when alive, 503 when shutting down
- `/live` (alias for `/health`)
- `/ready` (readiness) -- returns 200 when alive AND ready, 503 otherwise
- Default port: 7321 (`COLLECTOR_HEALTH_ADDR`)
- Thread-safe via `atomic.Bool` for both `ready` and `alive` flags

### NFR-4.3: Profiling
- Optional pprof server gated by `ENABLE_PPROF`
- Default: `localhost:3030`
- Endpoints: `/debug/pprof/` (index), `/debug/pprof/profile` (CPU), `/debug/pprof/symbol`, `/debug/pprof/trace`
- `/debug/pprof/cmdline` blocked for security
- Only available via root `main.go`, NOT via `cmd/collector` Docker entry point

### NFR-4.4: Batch Receipts (Audit Trail)
- Every successful batch saves a receipt with: version, request hash, count, first/last IDs, fetch timestamp, cursor applied
- Bounded to `maxReceipts` per source (default 100)
- Persisted in the state file alongside cursor state
- Enables post-hoc reconciliation and debugging

### NFR-4.5: Missing Observability
- **No metrics export** (Prometheus, OpenTelemetry) -- logs only
- **No distributed tracing** (no trace IDs, no spans)
- **No alerting integration** -- relies on Kubernetes probes + external monitoring
- **No request/response logging** at DEBUG level for Claroty API responses (only sink forwarding is logged)
- **No batch size or latency metrics** -- no visibility into collection performance

---

## NFR-5: Scalability

### NFR-5.1: Single Instance Design
- `replicaCount: 1` in Helm values
- No leader election or distributed locking
- State file is a single JSON document -- concurrent writers would corrupt
- PVC is ReadWriteOnce -- physically prevents multi-pod access

### NFR-5.2: Vertical Scaling Only
- Page size (100) is the only throughput knob
- Polling interval (30s) controls how often the API is hit
- No horizontal scaling support for the collector itself

### NFR-5.3: State File Growth
- Bounded by `maxReceipts` per source (default 100)
- 9 sources x 100 receipts = 900 max receipt entries
- Each receipt is small (< 200 bytes) -- state file stays under 1MB even with full receipt history

---

## NFR-6: Maintainability

### NFR-6.1: Interface-Driven Testing
- 3 core interfaces: `claroty.Client`, `sink.Sender`, `state.Store`
- All testable via fakes (`fakeClarotyClient`, `fakeSink`) or mocks (`mockgen`)
- Health `Reporter` interface enables nil or mock injection

### NFR-6.2: Code Generation
- `go:generate mockgen` for sink and state interfaces
- `go:embed` for OCSF YAML data files
- `tools/tools.go` pins tool versions with build tag

### NFR-6.3: Pre-commit Hooks
- `go-fumpt` formatting
- `go-build-mod` compilation check
- `go-mod-tidy` dependency cleanup
- Trailing whitespace and EOF fixer
- Large file guard (500KB)

---

## Delta Summary
- New items added: 19 NFR items across 6 categories, 11 missing NFR items identified
- Existing items refined: Connection pool settings now documented with architectural context (shared between both clients), timeout hierarchy fully mapped
- Remaining gaps: Exact retry behavior in Run() loop (no tests), OCSF mapper stub impact on observability

## Novelty Assessment
Novelty: SUBSTANTIVE
The disabled-by-default health probes in Kubernetes, the pprof unavailability in Docker builds, the absent metrics/tracing, and the single-instance-only design are all NFR findings that change how the system would be specified. The security hardening details (cmdline blocking, credential rotation gap) are new architectural insights.

## Convergence Declaration
Another round needed -- should verify config loading edge cases, confirm batch receipt growth bounds, and check for any undiscovered observability patterns in collector.go.

## State Checkpoint
```yaml
pass: 4
round: 1
status: complete
files_scanned: 20
timestamp: 2026-04-13T23:40:00Z
novelty: SUBSTANTIVE
next_pass: 4-r2
```
