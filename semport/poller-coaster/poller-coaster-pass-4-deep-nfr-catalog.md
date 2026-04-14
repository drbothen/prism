# Pass 4 Deep: NFR Catalog -- Round 1

**Project:** poller-coaster
**Date:** 2026-04-13
**Basis:** All source files, Helm chart, CI/CD workflows, Dockerfile, security-scan.yml, cross-referenced with broad sweep NFR section and Phase A outputs

---

## Performance

### Verified NFRs

| ID | Pattern | Location | Details | Evidence |
|----|---------|----------|---------|----------|
| NFR-P-001 | Configurable poll interval | COLLECTOR_INTERVAL env var, config.go | Default 30s, parsed via time.ParseDuration only (no integer fallback) | values.yaml:41, config.go |
| NFR-P-002 | Per-request result limit | ARMIS_xxx_LIMIT env vars | Default 100 per data source, configurable per-source | config.go |
| NFR-P-003 | Immediate re-poll on hasMore | collector.go:168-172 | Skips ticker wait when any source has more data | collector.go |
| NFR-P-004 | HTTP client timeout (Armis) | ARMIS_API_TIMEOUT | Default 30s, applies per SDK request | armis/api.go:49 |
| NFR-P-005 | HTTP client timeout (Sink) | VECTOR_TIMEOUT_SECONDS | Default 15s, applies per delivery request | http_sender.go:63 |
| NFR-P-006 | Sequential source collection | collector.go:492-529 | 7 sources polled sequentially, not in parallel | collector.go |
| NFR-P-007 | Per-record delivery (no batching) | http_sender.go | Each record gets its own HTTP POST, no batching | http_sender.go |
| NFR-P-008 | Docker build caching | Dockerfile | BuildKit mount caches for go module cache and build cache | Dockerfile:16-17 |
| NFR-P-009 | Trimmed binary | Dockerfile:25, Makefile:16 | -trimpath -ldflags "-s -w" strips debug info and paths | Dockerfile, Makefile |

### Performance Implications

- **Sequential collection** (NFR-P-006) means a slow source blocks all subsequent sources. If alerts take 10s, devices won't be polled until alerts complete.
- **Per-record delivery** (NFR-P-007) means N records = N HTTP roundtrips. For large batches (limit=100), this could be 700 HTTP calls per cycle (100 x 7 sources).
- **No connection pooling configuration** -- uses Go's default http.Client which maintains a connection pool, but no explicit tuning.

---

## Security

### Verified NFRs

| ID | Pattern | Location | Details | Evidence |
|----|---------|----------|---------|----------|
| NFR-S-001 | Secret file support | config.go | *_FILE env vars for K8s secret mounts (5 secrets supported) | config.go:319-329 |
| NFR-S-002 | File takes priority over env var | config.go:319-325 | When both *_FILE and direct env var set, file wins | TestLoadFromEnvironment_FilePrecedence |
| NFR-S-003 | Bearer auth for Armis | armis/api.go | API key injected via centrix SDK | api.go:59 |
| NFR-S-004 | Basic auth for sink | http_sender.go:183 | req.SetBasicAuth() on every request | http_sender.go |
| NFR-S-005 | Secret redaction in logs | config/utils.go:46-54 | Shows first 2 + last 2 chars, "***" for <= 4 chars | utils.go |
| NFR-S-006 | Non-root container | Dockerfile:32 | distroless:nonroot, USER nonroot:nonroot | Dockerfile |
| NFR-S-007 | Read-only root filesystem | values.yaml:104 | readOnlyRootFilesystem: true | values.yaml |
| NFR-S-008 | Drop all capabilities | values.yaml:102-103 | capabilities.drop: [ALL] | values.yaml |
| NFR-S-009 | Seccomp profile | values.yaml:96-97 | RuntimeDefault seccomp | values.yaml |
| NFR-S-010 | Non-root UID/GID | values.yaml:92-95 | runAsNonRoot, UID/GID 65532, fsGroup 65532 | values.yaml |
| NFR-S-011 | pprof cmdline blocked | pprof.go:30 | /debug/pprof/cmdline returns 404 (prevents exposing process args) | TestPprofMux_CmdlineBlocked |
| NFR-S-012 | pprof loopback warning | pprof.go:68-69 | Warns if non-loopback address, does not block | TestIsLoopback |
| NFR-S-013 | No secrets in logs | CLAUDE.md convention | "never log API keys or secrets" | CLAUDE.md |
| NFR-S-014 | Heap profile security warning | SECURITY.md (referenced in CLAUDE.md) | Heap profiles can capture in-memory secrets | CLAUDE.md |
| NFR-S-015 | CI security scanning | security-scan.yml | gosec + govulncheck + staticcheck, daily cron at 06:00 UTC | security-scan.yml |
| NFR-S-016 | CI runner hardening | All workflows | step-security/harden-runner with egress-policy: audit | All .yml files |
| NFR-S-017 | GH Action pin to SHA | All workflows | All actions pinned to commit SHA (not mutable tags) | All .yml files |
| NFR-S-018 | Minimal RBAC | rbac.yaml | get/list configmaps+secrets, watch secrets only | rbac.yaml |
| NFR-S-019 | Allowable privilege escalation | values.yaml:100 | allowPrivilegeEscalation: false | values.yaml |
| NFR-S-020 | Large file check | .pre-commit-config.yaml | check-added-large-files with 500kb limit | .pre-commit-config.yaml |
| NFR-S-021 | Whitespace trimming on secrets | config.go | TrimSpace on all env var and file values | TestLoadFromEnvironment_WhitespaceHandling |
| NFR-S-022 | Sink error body limited | http_sender.go:198 | io.LimitReader(resp.Body, 2048) prevents large error bodies | http_sender.go |

### New Finding: RBAC Watch Permission Unused

NFR-S-018 grants `watch` on secrets, but the Go code has NO secret-watching logic. This is an over-provisioned permission. The RBAC likely anticipates a future credential rotation feature that was never implemented.

### New Finding: Armis API Key Validation Gaps

The armis/api.go:38-39 validates API key with TrimSpace+empty check, but config.go:327-329 also validates API key presence during LoadFromEnvironment. There are **two independent validation points** for the same field, which is defense-in-depth but also means error messages differ depending on which validation catches the issue first.

---

## Observability

### Verified NFRs

| ID | Pattern | Location | Details | Evidence |
|----|---------|----------|---------|----------|
| NFR-O-001 | Structured JSON logging | runner.go:29 | charmbracelet/log with JSONFormatter, ReportTimestamp | runner.go |
| NFR-O-002 | Configurable log levels | POLLER_COASTER_LOG_LEVEL | DEBUG/INFO/WARN/ERROR/FATAL, default INFO | config.go, runner.go:130-144 |
| NFR-O-003 | Health endpoints | :7322 /health, /ready, /live | K8s probe targets | health/server.go |
| NFR-O-004 | Batch processing logs | Every collectXxx() | Logs count, timestamp, ID, version per batch | collector sources |
| NFR-O-005 | Sink delivery logs | http_sender.go:176,203 | Debug: type, endpoint, id, size_bytes; Info: forwarded record | http_sender.go |
| NFR-O-006 | Opt-in pprof | ENABLE_PPROF=1 | localhost:3030, CPU/memory/goroutine/allocs/block/mutex | pprof.go, pprof-harness.sh |
| NFR-O-007 | Coverage reporting | go-test.yml | Coverage threshold 70% (warning), HTML report artifact | go-test.yml:50-76 |
| NFR-O-008 | Dry-run validation output | config/utils.go | --dry-run prints redacted config to stdout | utils.go |
| NFR-O-009 | pprof harness script | scripts/pprof-harness.sh | Automated collection of 6 profile types with analysis instructions | pprof-harness.sh |

### New Finding: HTTP Server Timeouts on Health Server

The health server has explicit timeout configuration (previously undocumented in the broad sweep):

| Setting | Value | Location |
|---------|-------|----------|
| ReadHeaderTimeout | 10s | health/server.go:17 |
| ReadTimeout | 15s | health/server.go:18 |
| WriteTimeout | 15s | health/server.go:19 |
| IdleTimeout | 60s | health/server.go:20 |

Similarly, the pprof server:

| Setting | Value | Location |
|---------|-------|----------|
| ReadTimeout | 10s | pprof.go:22 |
| WriteTimeout | 120s (2 min, for CPU profiles) | pprof.go:23 |
| ReadHeaderTimeout | 5s | pprof.go:24 |
| IdleTimeout | 60s | pprof.go:25 |

---

## Reliability

### Verified NFRs

| ID | Pattern | Location | Details | Evidence |
|----|---------|----------|---------|----------|
| NFR-R-001 | Exponential backoff | collector.go | 2s base, 30s max, configurable via env vars | collector_test.go |
| NFR-R-002 | Configurable max retries | COLLECTOR_MAX_RETRIES | Default 10, 0=unlimited | collector_test.go |
| NFR-R-003 | Atomic file writes | file_store.go | temp+fsync+rename, cleanup on error | file_store_test.go |
| NFR-R-004 | Forward progress invariant | All 7 collectors | Prevents cursor regression | Source code |
| NFR-R-005 | Query fingerprint validation | initializeXxxState() | Detects config drift, prevents silent resume with wrong params | Source code |
| NFR-R-006 | Graceful shutdown | runner.go:105-127 | Context cancellation + 5s shutdown timeout for health server | runner.go |
| NFR-R-007 | Receipt-based auditing | BatchReceipt types | Tracks what was fetched, versioned | state/store.go |
| NFR-R-008 | Health state transitions | collector.go | SetReady on success, SetNotReady on failure | collector.go |
| NFR-R-009 | Rate-limited health endpoint | health/server.go | 100 req/s per IP, burst 20, token bucket | server_test.go |
| NFR-R-010 | Pprof eager bind | pprof.go:72 | Returns error immediately if port unavailable, not delayed to goroutine | TestStart_ReturnsErrorOnBindFailure |
| NFR-R-011 | PVC persistence | values.yaml:49-64 | 100Mi RWO PVC, existingClaim support | pvc.yaml |
| NFR-R-012 | State survives restart | file_store_test.go | Verified: all 7 states survive process restart | TestFileStore_AllSevenStatesSurviveRestart |
| NFR-R-013 | Receipt trimming | file_store.go | Keeps maxReceipts (default 100) most recent per source | TestFileStore_WithMaxReceipts |
| NFR-R-014 | Retry counter reset | collector.go | Resets to 0 + base delay after any successful collectOnce | TestCollector_MaxRetries_ResetsAfterSuccess |
| NFR-R-015 | ErrStateNotFound bootstrap | collector.go | First run initializes with zero cursor, not an error | Source code |

### New Finding: Pprof Shutdown in Entrypoint

Both main.go and cmd/collector/main.go have a deferred pprof shutdown with 5s timeout. This means if pprof shutdown hangs, the process will wait up to 5s before exiting. The runner also has a 5s shutdown for the health server. In the worst case, shutdown takes 10s (5s runner + 5s pprof).

### New Finding: Health Server Error Channel Handling

runner.go:100 creates `healthErrCh` with buffer size 1. After collector.Run returns:
- **On error path (line 106):** health server shutdown with Background context (no timeout), health error channel NOT drained
- **On success path (line 118-124):** Non-blocking select on healthErrCh, only returns error if it's not ErrServerClosed

The error path could potentially miss health server errors (they're ignored), which is acceptable since the collector already failed.

---

## Scalability

### Verified NFRs

| ID | Pattern | Location | Details |
|----|---------|----------|---------|
| NFR-SC-001 | Single-instance only | values.yaml:13 | replicaCount: 1, no distributed locking |
| NFR-SC-002 | RWO PVC | values.yaml:56 | ReadWriteOnce access mode prevents multi-pod access |
| NFR-SC-003 | No HPA | Helm chart | No HorizontalPodAutoscaler template |
| NFR-SC-004 | Configurable result limits | config.go | Can increase per-source limits to handle more data per cycle |
| NFR-SC-005 | hasMore immediate re-poll | collector.go | High-volume sources get polled continuously until caught up |

---

## Missing NFRs (expected but not found)

| Category | Missing Pattern | Impact |
|----------|----------------|--------|
| Performance | No Armis API rate limiting | Could hit API rate limits under heavy load |
| Performance | No connection pooling configuration | Uses Go defaults (may be suboptimal for high-throughput) |
| Performance | No batch delivery to sink | 1 HTTP call per record, inefficient for large volumes |
| Reliability | No circuit breaker | Continuous retries even when API is completely down |
| Reliability | No sink retry | Sink failure bubbles up to collector retry loop (retries entire cycle) |
| Reliability | No dead letter queue | Failed deliveries are retried as part of entire batch, no individual retry |
| Security | No mTLS for sink | Basic auth only, no certificate-based auth |
| Security | No OAuth for either API | Bearer token (Armis) and basic auth (sink) only |
| Observability | No Prometheus metrics | No /metrics endpoint, no counters/histograms |
| Observability | No distributed tracing | No OpenTelemetry or trace propagation |
| Observability | No audit log for config changes | No logging when config reloaded or fingerprint changes |
| Scalability | No distributed locking | Cannot run multiple instances safely |
| Scalability | No sharding | Cannot split data sources across instances |
| Availability | No PodDisruptionBudget | No PDB template in Helm chart |
| Availability | No anti-affinity | No pod anti-affinity rules |

---

## Configuration Values Encoding NFR Decisions

| Env Var | Default | NFR Encoded |
|---------|---------|-------------|
| COLLECTOR_INTERVAL | 30s | Poll frequency / API load |
| COLLECTOR_MAX_RETRIES | 10 | Failure tolerance before giving up |
| COLLECTOR_RETRY_BASE_DELAY | 2s | Minimum backoff |
| COLLECTOR_RETRY_MAX_DELAY | 30s | Maximum backoff cap |
| ARMIS_API_TIMEOUT | 30s | API call patience |
| VECTOR_TIMEOUT_SECONDS | 15s | Sink delivery patience |
| STATE_STORE_MAX_RECEIPTS | 100 | Audit trail depth |
| ARMIS_xxx_LIMIT | 100 | Batch size / memory usage tradeoff |

---

## Delta Summary

- New items added: 22 security NFRs (up from 10 in broad sweep), HTTP server timeouts for health+pprof, pprof shutdown timing, health error channel handling, 5 scalability NFRs, 15 missing NFRs
- Existing items refined: All broad sweep NFRs verified and given IDs, performance implications documented
- Remaining gaps: None that would change the NFR model

## Novelty Assessment

Novelty: SUBSTANTIVE

This round adds 12 new security NFRs (CI scanning, runner hardening, SHA pinning, whitespace trimming, error body limiting, etc.), documents the health/pprof server timeout configurations for the first time, identifies the RBAC over-provisioning, and provides a comprehensive missing-NFR analysis. These findings change how you would spec the system's security posture and operational requirements.

## Convergence Declaration

Another round needed -- should verify: (1) whether there are any NFRs encoded in the Helm test values or ci directory, (2) any reliability patterns in the collector that were missed, (3) hallucination audit of all claimed NFR locations.

## State Checkpoint

```yaml
pass: 4
round: 1
status: complete
files_scanned: 48
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
next_action: round 2 -- hallucination audit, verify all NFR locations, check for missed patterns
```
