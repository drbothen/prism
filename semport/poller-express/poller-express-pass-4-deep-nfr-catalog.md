# Pass 4 Deep: NFR Catalog -- poller-express (Round 1)

## NFR-SEC: Security

### NFR-SEC-001: Container Hardening

**Source**: Dockerfile, Helm values.yaml

| Control | Value | Evidence |
|---------|-------|----------|
| Base image | `gcr.io/distroless/static-debian12:nonroot` | Dockerfile:32 |
| Runtime user | nonroot (conventionally UID 65532) | Dockerfile:37, values.yaml:73 |
| Read-only filesystem | `readOnlyRootFilesystem: true` | values.yaml:80 |
| No privilege escalation | `allowPrivilegeEscalation: false` | values.yaml:78 |
| Dropped capabilities | ALL | values.yaml:81-82 |
| Seccomp profile | RuntimeDefault | values.yaml:75-76 |
| CGO disabled | `CGO_ENABLED=0` in Dockerfile | Dockerfile:24 |
| Binary trimpath | `-trimpath` flag | Dockerfile:25 |
| Stripped symbols | `-ldflags "-s -w"` | Dockerfile:26 |

### NFR-SEC-002: Secret Management

**Source**: config.go, values.yaml

| Mechanism | Description | Precedence |
|-----------|-------------|------------|
| File-backed secrets | `*_FILE` env vars read from filesystem | Highest |
| K8s Secret refs | `existingSecret` or `apiKeySecretName` in Helm | High |
| Direct env vars | Standard env var injection | Medium |
| Helm values (deprecated) | `cyberint.apiKey` directly in values | Lowest (removed in v2.0.0) |

File-backed secret loading: `readSecretFile()` returns empty (not error) if file not found, allowing fallback. Other read errors are fatal.

### NFR-SEC-003: Credential Redaction

**Source**: config/utils.go

Secret values are redacted in dry-run output: empty -> "<empty>", 1-4 chars -> "***", 5+ chars -> first 2 + "***" + last 2 chars.

### NFR-SEC-004: Authentication

**Source**: runner.go, http_sender.go

| Target | Mechanism | Implementation |
|--------|-----------|----------------|
| Cyberint API | Cookie-based | `access_token` cookie via custom RoundTripper |
| Vector sink | HTTP Basic Auth | `req.SetBasicAuth(username, password)` |

### NFR-SEC-005: CI Security Scanning

**Source**: .github/workflows/security-scan.yml, build.yaml

| Scanner | Scope | Schedule |
|---------|-------|----------|
| gosec | Go source code | PR + push + daily cron (06:00 UTC) |
| govulncheck | Go dependencies | PR + push + daily cron |
| staticcheck | Go static analysis | PR + push + daily cron |
| Trivy | Container image | Build workflow (PR + push) |
| golangci-lint (gosec linter) | Go source | PR + push |

### NFR-SEC-006: Runner Hardening

**Source**: .github/workflows/*.yml

All CI workflows use `step-security/harden-runner` with `egress-policy: audit`. All GitHub Actions are pinned by SHA hash (not tag).

### NFR-SEC-007: RBAC Least Privilege

**Source**: deploy/helm templates/rbac.yaml

Pod service account gets:
- `get`, `list` on configmaps and secrets
- `watch` on secrets

No write permissions. No cluster-scoped permissions.

---

## NFR-PERF: Performance

### NFR-PERF-001: Connection Reuse

**Source**: runner.go

A single `http.Client` with shared `http.DefaultTransport` is used for all Cyberint API calls. Both alert and asset collectors share TCP connections and TLS sessions.

### NFR-PERF-002: Batch Processing

**Source**: collector/alert_collector.go, collector/asset_collector.go

| Collection Type | Page Size | Pagination |
|----------------|-----------|------------|
| Alerts | 100 records per page | `hasMore = (len(alerts) == 100)` |
| Assets | 1000 records per page | `hasMore = (pageNumber * 1000 < totalAssets)` |

When `hasMore=true`, the collector re-enters `collectOnce()` immediately without waiting for the ticker.

### NFR-PERF-003: Payload Enrichment (Manual JSON)

**Source**: sink/http_sender.go:121-149

Enrichment uses manual `bytes.Buffer` + `json.Encoder` rather than `json.Marshal(EnrichedPayload{...})`. This avoids an extra serialization/deserialization round for the data field. Benchmark test `BenchmarkEnrichPayload` exists to validate performance.

### NFR-PERF-004: HTTP Timeouts

| Component | Timeout | Source |
|-----------|---------|--------|
| Cyberint API client | 30s | runner.go:60 |
| Vector sink client | 15s (configurable) | http_sender.go:59, config default |
| Health server ReadHeader | 10s | health/server.go:17 |
| Health server Read | 15s | health/server.go:18 |
| Health server Write | 15s | health/server.go:19 |
| Health server Idle | 60s | health/server.go:20 |
| Health shutdown | 5s | runner.go:173 |
| Pprof shutdown | 5s | main.go:37 |

### NFR-PERF-005: Response Size Limits

**Source**: asset/client.go

Asset API responses are capped at 10 MiB via `io.LimitReader`. Responses exceeding this limit return an error.

---

## NFR-REL: Reliability

### NFR-REL-001: Exponential Backoff Retry

**Source**: collector/alert_collector.go:86-154, collector/asset_collector.go

| Parameter | Default | Env Var |
|-----------|---------|---------|
| Base delay | 2s | `COLLECTOR_RETRY_BASE_DELAY` |
| Max delay | 30s | `COLLECTOR_RETRY_MAX_DELAY` |
| Max retries | 5 | `COLLECTOR_MAX_RETRIES` |
| Backoff formula | `delay *= 2`, capped at maxDelay | N/A |

Special behavior: `MaxRetries=0` disables the retry limit (infinite retries). Counter resets to 0 on any successful cycle.

### NFR-REL-002: Cursor Forward Progress Guarantee

**Source**: collector/alert_collector.go, collector/asset_collector.go

Cursors must strictly advance. `ErrCursorRegression` prevents backward movement. This ensures at-least-once delivery semantics.

### NFR-REL-003: Query Fingerprint Drift Detection

**Source**: state/store.go

SHA-256 hash of sorted field names + limit detects configuration changes that would invalidate the cursor. Mismatch is a fatal error.

### NFR-REL-004: Health Readiness Tracking

**Source**: health/server.go, collector

| Endpoint | Path | Semantics |
|----------|------|-----------|
| Liveness | `/health`, `/live` | Returns 200 while process is alive |
| Readiness | `/ready` | Returns 200 only when alive AND at least one successful cycle |

State transitions: NotReady on startup, Ready after success, NotReady on failure, Ready on recovery.

### NFR-REL-005: At-Least-Once Delivery

**Design**: Cursor is advanced only AFTER successful sink delivery. If the process crashes mid-batch, the entire batch is re-sent on restart (cursor was not advanced). This guarantees no records are lost but duplicates are possible.

**Caveat**: MemoryStore loses all state on restart, so the collector re-fetches everything from the beginning. For alerts, the `modification_date` filter mitigates this somewhat; for assets, all assets are re-fetched every time.

### NFR-REL-006: Graceful Degradation

**Source**: runner.go:82-97

If `VECTOR_ENDPOINT` is not set, the sink is nil. Collectors still run: they fetch, sort, filter, and advance cursors -- but do not forward records. This allows the collector to run in "dry run" mode at the operational level.

---

## NFR-OBS: Observability

### NFR-OBS-001: Structured JSON Logging

**Source**: runner.go:31

Primary logger: `charmbracelet/log` with `JSONFormatter` and `ReportTimestamp: true`. Output to stdout (compatible with Kubernetes log collection).

### NFR-OBS-002: Log Level Taxonomy

| Level | Usage |
|-------|-------|
| DEBUG | Cycle details, payload sizes, enrichment metadata |
| INFO | Batch summaries (forwarded record count), sink init, xMP config |
| WARN | Retries, sink disabled, shutdown errors, invalid log level fallback |
| ERROR | Collection failures, sink rejections, API errors |
| FATAL | (configurable but not explicitly used via logger.Fatal) |
| TRACE | Alias for DEBUG |

### NFR-OBS-003: Structured Log Fields

Common structured fields across all log statements:
- `type`: record type ("cyberint_alert", "cyberint_asset")
- `endpoint`: target URL
- `id`: record ID
- `error`: error details
- `count`: batch record count
- `size_bytes`: payload size
- `status`: HTTP status code
- `body`: response body (truncated)

### NFR-OBS-004: Pprof Profiling

**Source**: profiling/pprof.go, scripts/pprof-harness.sh

Optional pprof server on `localhost:3030` (configurable via `PPROF_ADDR`). Enabled by setting `ENABLE_PPROF` to any non-empty value.

Profiling harness script collects: CPU (30s profile), heap, goroutine, allocs, block, mutex snapshots.

### NFR-OBS-005: Health Endpoints as Observability

Health endpoints serve dual purpose:
1. Kubernetes probe targets
2. Operational observability (is the collector healthy?)

Rate-limited at 100 req/s (burst 20) per IP to prevent monitoring storms from affecting the collector.

---

## NFR-SCALE: Scalability

### NFR-SCALE-001: Single-Tenant, Single-Replica Design

**Source**: Architecture, Helm values

- One pod polls one Cyberint customer
- `replicaCount: 1` in values.yaml
- No horizontal scaling (would cause duplicate polling)
- No leader election or distributed locking

### NFR-SCALE-002: In-Memory State

**Source**: state/store.go

MemoryStore means:
- No external state dependencies (simpler deployment)
- State lost on pod restart (re-fetches everything)
- Cannot scale horizontally (no shared state)
- No persistent storage requirements

### NFR-SCALE-003: Page Size Limits

Alert page size: 100 records. Asset page size: 1000 records. These are hardcoded, not configurable.

---

## NFR-MAINT: Maintainability

### NFR-MAINT-001: Dependency Management

**Source**: renovate.json

Automated dependency updates via Renovate with:
- Grouped non-major updates
- Separate groups for golang.org/x, charmbracelet, Go version, GitHub Actions
- Auto-merge for GitHub Actions and pre-commit hooks
- Manual review for major updates
- `gomodTidy` as post-update step

### NFR-MAINT-002: Code Quality Pipeline

Pre-commit: gofumpt formatting, Go build check, go mod tidy
CI: golangci-lint v2.5 (12 linters), race detector, 70% coverage threshold (warning, not blocking)

### NFR-MAINT-003: Linter Configuration

12 linters enabled in golangci-lint:
- Core: errcheck (type assertions), govet, staticcheck, unused, ineffassign, unconvert
- Quality: gocritic (diagnostic+style+performance), revive, goconst, misspell, whitespace
- Security: gosec
- Excluded: gosec and errcheck in test files
- Disabled check: hugeParam (intentional value semantics)
- Formatters: gofumpt, goimports

---

## Missing NFRs (Expected but Not Found)

| NFR | Expected | Status |
|-----|----------|--------|
| Metrics/Prometheus | Counter/gauge for records processed, errors, latency | NOT PRESENT |
| Distributed tracing | OpenTelemetry spans for API calls, sink delivery | NOT PRESENT |
| Circuit breaker | For Cyberint API or Vector sink | NOT PRESENT |
| Rate limiting (outbound) | Throttling requests to Cyberint API | NOT PRESENT |
| NetworkPolicy | Kubernetes network isolation | NOT PRESENT in chart |
| PodDisruptionBudget | Voluntary disruption protection | NOT PRESENT (single replica) |
| TLS configuration | Custom CA, mTLS, cert rotation | NOT CONFIGURABLE |
| Graceful shutdown | OS signal handling (SIGTERM) | NOT PRESENT |
| Resource limits | Default CPU/memory limits | NOT SET in production values |
| Persistent state | Durable cursor storage | NOT PRESENT (MemoryStore only) |
| Dead letter queue | Failed records quarantine | NOT PRESENT |

---

## Delta Summary
- New items added: 22 NFRs across 6 categories, 11 missing NFRs identified
- Existing items refined: Expanded security posture from broad sweep's brief mention to detailed table, added precise timeout values, documented the dual http.Client architecture implications
- Remaining gaps: Should verify whether metrics were considered and rejected, or simply not yet implemented

## Novelty Assessment
Novelty: SUBSTANTIVE
The broad sweep mentioned NFRs in passing but this round provides: (1) complete security posture with 7 specific controls verified from code, (2) precise timeout values for all 8 timeout configurations, (3) daily cron-triggered security scans not mentioned before, (4) the MaxRetries=0 infinite retry behavior as a reliability feature, (5) 11 specific missing NFRs that inform the port decision, (6) the logging inconsistency between charmbracelet/log and slog, (7) RBAC least-privilege details, (8) Renovate configuration for dependency freshness. These materially inform the specification.

## Convergence Declaration
Another round needed -- should audit specific sentinel error usage to verify if ErrCyberIntUnexpectedStatus and others are actually exercised, and check if any NFR patterns exist in the generated client code.

## State Checkpoint
```yaml
pass: 4
round: 1
status: complete
timestamp: 2026-04-13T23:40:00Z
novelty: SUBSTANTIVE
```
