# Pass 4 Deep: NFR Catalog -- poller-cobra (Round 1)

> Convergence deepening round 1. Extracted from all source files, Helm chart, CI workflows, and Dockerfile.

---

## Performance

### P-001: HTTP Client Timeout

**Location:** sink/http_sender.go:76
**Pattern:** `http.Client{Timeout: timeout}` where timeout defaults to 15s (configurable via `VECTOR_TIMEOUT_SECONDS`)
**Impact:** Per-request timeout prevents indefinite hangs on sink delivery
**Configuration:** `VECTOR_TIMEOUT_SECONDS` (accepts duration string or plain integer seconds)

### P-002: Single-Record Delivery (No Batching)

**Location:** alert_collector.go:82-86
**Pattern:** Each alert is sent individually via `sink.Send()` in a for loop
**Impact:** For 100 alerts per batch, this means 100 HTTP round-trips. Significant overhead at high volume.
**Missing:** No connection pooling configuration, no HTTP/2, no batch POST endpoint

### P-003: Burst-Fetch on Backlog

**Location:** collector.go:156-159
**Pattern:** When `hasMore=true` (newAlerts >= limit), collector immediately continues without waiting for ticker
**Impact:** Catches up on backlogs efficiently by running fetch cycles back-to-back
**Limitation:** The hasMore heuristic compares *filtered* count against limit, which can under-report (see Pass 3 hasMore edge case)

### P-004: Polling Interval

**Location:** collector.go:122, config.go:152
**Pattern:** `time.NewTicker(interval)` with default 30s. Configurable via `COLLECTOR_INTERVAL`.
**Impact:** Minimum latency floor of 30s for alert ingestion when no backlog exists

### P-005: Go Module Download Caching in Docker

**Location:** Dockerfile:15-18
**Pattern:** `--mount=type=cache,target=/go/pkg/mod` and `--mount=type=cache,target=/root/.cache/go-build`
**Impact:** Faster Docker builds via BuildKit layer caching

### P-006: Static Binary Build

**Location:** Dockerfile:23-27
**Pattern:** `CGO_ENABLED=0`, `-trimpath`, `-ldflags "-s -w"`
**Impact:** Minimal binary size, no glibc dependency, distroless-compatible

### P-007: Opt-in Profiling

**Location:** profiling/pprof.go
**Pattern:** Pprof server gated behind `ENABLE_PPROF=1`, separate port (default localhost:3030)
**Endpoints:** CPU profile, heap, goroutine, allocs, block, mutex
**Support script:** `scripts/pprof-harness.sh` collects all profile types

### P-008: No Connection Reuse Configuration

**Location:** sink/http_sender.go:76
**Pattern:** Default `http.Client{}` with no transport configuration
**Impact:** Uses Go's default `http.Transport` which has connection pooling (MaxIdleConnsPerHost=2). For a single-endpoint sink this is adequate but not optimized for burst scenarios.

### P-009: No Response Body Consumption on Success

**Location:** sink/http_sender.go:117 (returns after logging)
**Pattern:** On success (status < 400), response body is never read/drained
**Impact:** The deferred `resp.Body.Close()` will close without draining. For HTTP/1.1, this means the connection cannot be reused. Reduces connection pool effectiveness.

---

## Security

### S-001: OAuth2 Client Credentials

**Location:** crowdstrike/api.go:86-91
**Pattern:** gofalcon SDK handles OAuth2 token lifecycle transparently
**Scope:** CrowdStrike Falcon API authentication
**Risk:** Token refresh failures are opaque (handled within SDK)

### S-002: HTTP Basic Auth for Sink

**Location:** sink/http_sender.go:97
**Pattern:** `req.SetBasicAuth(s.username, s.password)` on every request
**Risk:** Credentials sent per-request, no token caching. But this is over internal network (K8s pod-to-pod).

### S-003: Secret File Loading (K8s Secret Mounts)

**Location:** config/config.go:448-462
**Pattern:** Read from file path -> trim whitespace -> return. `os.ErrNotExist` returns empty (graceful fallback).
**File path sanitization:** `filepath.Clean(path)` applied before read
**Priority:** File-backed > direct env var

### S-004: Secret Redaction in Dry-Run Output

**Location:** config/utils.go:49-57
**Pattern:** First 2 + last 2 chars visible, rest masked with `***`. Strings <= 4 chars fully masked.
**Scope:** Only in `--dry-run` output. Runtime logs never print secrets.

### S-005: Pprof Cmdline Blocked

**Location:** profiling/pprof.go:30
**Pattern:** `/debug/pprof/cmdline` mapped to `http.NotFound`
**Purpose:** Prevents exposure of process arguments (which may contain secrets in env vars)

### S-006: Pprof Non-Loopback Warning

**Location:** profiling/pprof.go:68-69
**Pattern:** If pprof address is not loopback, log warning
**Risk:** Advisory only -- does not prevent binding to non-loopback

### S-007: Container Hardening

**Location:** values.yaml:92-108, Dockerfile:37
**Controls:**
- runAsNonRoot + UID 65532
- readOnlyRootFilesystem
- Drop ALL capabilities
- No privilege escalation
- Seccomp RuntimeDefault
- Distroless base (no shell, no package manager)

### S-008: RBAC Least Privilege

**Location:** deploy/helm/poller-cobra/templates/rbac.yaml
**Permissions:** get/list configmaps+secrets, watch secrets
**Purpose:** Read-only access for config and credential rotation

### S-009: CI Runner Hardening

**Location:** All 7 CI workflow files
**Pattern:** `step-security/harden-runner` with `egress-policy: audit` on every job
**Purpose:** Audit network egress from CI runners, supply chain security

### S-010: Pinned Action References

**Location:** All CI workflows
**Pattern:** Actions referenced by full SHA hash (e.g., `actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6`)
**Purpose:** Prevent supply chain attacks via mutable tags

### S-011: No TLS Certificate Validation Configuration

**Location:** sink/http_sender.go
**Missing:** No custom TLS config, no cert pinning, no InsecureSkipVerify
**Impact:** Uses Go's default TLS behavior (system CA store). This is correct for internal K8s networking but there is no way to configure custom CA certs for the sink endpoint.

### S-012: No Input Validation on FQL Filter

**Location:** config.go:223-225, api.go:127
**Pattern:** Filter string is passed directly from env var to CrowdStrike API with no sanitization
**Risk:** Low -- FQL is a query language, not executable code, and the CrowdStrike API validates it server-side

---

## Observability

### O-001: Structured JSON Logging

**Location:** runner.go:37
**Library:** `charmbracelet/log` with JSON formatter + timestamps
**Field convention:** Lowercase snake_case (e.g., `cursor_timestamp`, `retry_in`, `size_bytes`)
**Levels:** DEBUG, INFO, WARN, ERROR

### O-002: Log Level Bug

**Location:** runner.go:131-141
**Issue:** `parseLogLevel` only handles "", "INFO", "DEBUG", "TRACE". Config validation accepts WARN/ERROR/FATAL but the parser rejects them and falls back to INFO.
**Impact:** Cannot set log level to WARN, ERROR, or FATAL despite config validation passing

### O-003: Health Endpoints

**Location:** health/server.go
**Endpoints:** `/health` (liveness), `/live` (liveness alias), `/ready` (readiness)
**Protocol:** HTTP on port 7322 (configurable via `HEALTH_ADDR`)
**K8s integration:** Liveness and readiness probes defined in values.yaml but **disabled by default**

### O-004: No Metrics

**Missing:** No Prometheus metrics endpoint, no OpenTelemetry, no custom metric counters
**Impact:** No visibility into: alerts processed/sec, fetch latency, sink delivery latency, retry counts, cursor position, batch sizes
**Note:** The go.mod includes `prometheus/client_golang` as an indirect dep (via gofalcon) but it is not used by poller-cobra itself

### O-005: No Distributed Tracing

**Missing:** No OpenTelemetry spans, no trace context propagation
**Note:** `go.opentelemetry.io/otel` appears in go.mod as indirect dep (via gofalcon SDK) but is not used by application code

### O-006: Logging Gaps

**Missing:**
- No log on successful state persistence
- No log on QueryFingerprint computation
- No log of the computed fingerprint hash on startup (would help debug ErrQueryFingerprintMismatch)
- No log of total production Go LOC or dependency count at startup (not expected, just noting)
- No request/response timing logs for CrowdStrike API calls

### O-007: Pprof Profiling (Opt-in)

**Location:** profiling/pprof.go
**Available profiles:** CPU, heap, goroutine, allocs, block, mutex, symbol, trace
**Blocked:** /debug/pprof/cmdline (404)
**HTTP timeouts:** read=10s, write=120s (write is long to accommodate CPU profiling)

---

## Reliability

### R-001: Exponential Backoff Retry

**Location:** collector.go:125-147
**Config:** Base delay 2s, max delay 30s, max retries 5 (all configurable)
**Behavior:** `delay *= 2` per failure, capped at `maxDelay`. Reset on success.
**Edge case:** MaxRetries=0 means unlimited retries (condition `MaxRetries > 0 && retryCount > MaxRetries` is false when MaxRetries=0)

### R-002: Fail-Fast on Startup

**Location:** runner.go:95-97
**Pattern:** `csClient.Ping(ctx)` before entering poll loop. Validates credentials and connectivity.
**Impact:** Avoids entering retry loop for configuration errors

### R-003: Query Fingerprint Drift Detection

**Location:** collector.go:177-179
**Pattern:** SHA-256 of `[region, sourceType, filter, limit]` compared on startup
**Impact:** Prevents using stale cursor after config change. Requires manual state reset.

### R-004: Forward-Only Cursor

**Location:** alert_collector.go:145-152
**Pattern:** `ensureForwardProgress()` fails if cursor does not advance after processing
**Impact:** Prevents reprocessing loops

### R-005: Graceful Shutdown via Context Cancellation

**Location:** runner.go:33, collector.go:162-163
**Pattern:** `signal.NotifyContext(SIGTERM, SIGINT)` -> context cancelled -> collector exits
**Gap:** Health server is not gracefully shut down (see Architecture deep dive)

### R-006: At-Least-Once Delivery

**Location:** collector.go:224-230
**Pattern:** State updated in-memory before persisting to store. If persist fails, in-memory is ahead.
**Impact:** If process crashes between sink delivery and state save, alert is re-sent on restart. Downstream must be idempotent.

### R-007: No Sink-Level Retry

**Location:** sink/http_sender.go
**Missing:** No per-request retry at sink layer. If `Send()` fails, entire batch is aborted and collector-level retry kicks in.
**Impact:** A transient sink error causes refetch of the entire batch from CrowdStrike

### R-008: No Circuit Breaker

**Missing:** No circuit breaker on CrowdStrike API or sink endpoint
**Impact:** During extended outages, the retry loop will continuously attempt connections until MaxRetries exhausted

### R-009: No Dead Letter Queue

**Missing:** No DLQ for failed deliveries. If max retries exceeded, the process terminates.
**Impact:** Requires external restart mechanism (K8s will restart pod if probes are enabled)

### R-010: MemoryStore Persistence Gap

**Location:** runner.go:61 (hardcoded `state.NewMemoryStore()`)
**Impact:** All cursor state lost on pod restart. Re-fetches all historical alerts on restart (since bootstrap cursor is zero-time).
**Mitigation:** Helm chart provisions PVC and sets `STATE_STORE_TYPE=file`, but runner ignores it.

### R-011: Rate Limiter Memory Growth

**Location:** health/server.go:89-108
**Pattern:** Per-IP `rate.Limiter` stored in map, never evicted
**Impact:** In long-running deployments with many unique source IPs (e.g., health check rotators), the map grows indefinitely
**Severity:** Low -- health endpoint is internal, limited IP diversity expected

---

## Scalability

### SC-001: Single Replica by Design

**Location:** values.yaml:14 (`replicaCount: 1`)
**Reason:** Cursor-based polling is single-consumer. Multiple replicas would duplicate work.
**Impact:** Vertical scaling only. Processing capacity limited by single pod resources.

### SC-002: No Horizontal Partitioning

**Missing:** No sharding by region, source type, or filter. A single collector handles one data source.
**Pattern:** Each deployment polls one CrowdStrike tenant with one filter.

### SC-003: No Queue/Buffer

**Missing:** No internal queue between fetch and sink. Alerts are processed synchronously in-order.
**Impact:** Sink latency directly affects fetch throughput. A slow sink backs up the entire pipeline.

### SC-004: Fixed Batch Size

**Location:** config.go:96 (default limit=100, configurable via `CROWDSTRIKE_LIMIT`)
**Impact:** Batch size is the primary knob for throughput tuning. Larger batches = fewer API calls but more memory per cycle.

---

## Missing NFRs (Expected but Not Found)

| NFR Category | Expected | Status |
|-------------|----------|--------|
| Metrics | Prometheus/OTel metrics endpoint | Not implemented |
| Tracing | Distributed tracing | Not implemented |
| Alerting | Alert on retry exhaustion | Not implemented (process exits) |
| Rate limiting (source) | CrowdStrike API rate limit handling | Delegated to SDK (opaque) |
| Compression | gzip for sink delivery | Not implemented |
| TLS configuration | Custom CA certs for sink | Not configurable |
| Connection draining | HTTP connection reuse | Not optimized (response body not drained on success) |
| File-backed state | Durable cursor persistence | Config supports it, implementation missing |
| Resource limits | K8s resource requests/limits | Defined in values.yaml but empty by default |
| Network policy | K8s NetworkPolicy | Not included in Helm chart |
| PodDisruptionBudget | PDB for availability | Not included (singleton makes this moot) |

---

## Delta Summary
- New items added: 12 security controls (S-001 through S-012), 7 observability items (O-001 through O-007), 11 reliability items (R-001 through R-011), 4 scalability items (SC-001 through SC-004), 9 performance items (P-001 through P-009), 11 missing NFRs
- Existing items refined: Broad sweep's NFR coverage was scattered across sections; now consolidated into a single catalog with locations and impact assessments
- Remaining gaps: docs/PROFILING_FINDINGS.md may contain performance NFR findings

## Novelty Assessment
Novelty: SUBSTANTIVE
Several new findings change the NFR model: (1) P-009 response body not drained on success affects connection reuse and performance at scale. (2) S-011 no custom TLS config for sink limits deployment flexibility. (3) O-004 the prometheus and OpenTelemetry indirect deps exist but are unused -- important for the Rust rewrite to know there is zero metrics/tracing to preserve. (4) R-010 the MemoryStore persistence gap combined with zero-time bootstrap means every pod restart re-fetches ALL historical alerts. (5) O-002 the log level bug means WARN/ERROR/FATAL are unreachable in production. These findings would change how you'd spec NFRs.

## Convergence Declaration
Another round needed -- should verify PROFILING_FINDINGS.md content and audit for any NFR patterns in CI workflows.

## State Checkpoint
```yaml
pass: 4
round: 1
status: complete
files_scanned: all
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
```
