# Pass 4 Deep: NFR Catalog -- Round 2

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 2

---

## Hallucination Audit of Round 1

### NFR-1.5 "No rate limiting": CONFIRMED
No evidence of rate limiting, throttling, or 429 handling in any source file.

### NFR-2.3 "ErrCursorRegression unused": CONFIRMED
All 9 `ensure*ForwardProgress` functions use plain `fmt.Errorf("cursor did not advance: ...")` or `fmt.Errorf("server cursor did not advance: ...")` -- none wrap `apperrors.ErrCursorRegression`.

### NFR-2.7 "Run() retry loop has no test coverage": CONFIRMED
Pass 3 R2 confirmed this: no test for exponential backoff, maxRetries exceeded, or retryDelay doubling.

### NFR-3.3 "Page size not configurable via env vars": CONFIRMED
`config.go` `LoadFromEnvironment()` does not read any `*_LIMIT` environment variables. Page sizes are only set in `DefaultConfig()`.

### NFR-4.3 "pprof only via root main.go": CONFIRMED
`cmd/collector/main.go` has 14 lines with no pprof import or call.

---

## New Findings from Collector Deep-Dive

### NFR-7: Failure Isolation (NEW CATEGORY)

### NFR-7.1: Fail-Fast Collection Ordering
- `collectOnce()` calls all 9 sources sequentially in fixed order
- **Any single source failure blocks all subsequent sources**
- Order: alerts -> events -> auditLogs -> deviceAlertRelations -> deviceVulnRelations -> servers -> sites -> devices -> vulnerabilities
- Failure in alerts prevents events through vulnerabilities from being collected
- This is a **deliberate simplicity trade-off** over fault isolation

### NFR-7.2: No Source-Level Retry
- Retries happen at the `collectOnce()` level (all-or-nothing)
- A source that consistently fails will block all other sources from making progress
- No circuit breaker per source -- the entire collector enters retry mode

### NFR-7.3: Initialization Fail-Fast
- `initializeState()` has the same fail-fast pattern
- Fingerprint mismatch on any source is fatal for the entire collector
- No partial initialization recovery

### NFR-7.4: Panic Recovery (OCSF Only)
- Only the OCSF mapper has panic recovery (`defer func() { ... recover() ... }()`)
- No panic recovery in collector, state, or client code
- A panic in any collect* function would crash the entire process

---

## NFR Refinements from Architecture Deep-Dive

### NFR-2.1 Retry Strategy: UPDATED
From the actual `Run()` loop code:

```
Initial state:
  retryCount = 0
  retryDelay = baseDelay (2s)

On error:
  SetNotReady()
  retryCount++
  if maxRetries > 0 && retryCount > maxRetries:
    return ErrCollectorRetriesExceeded (FATAL)
  log warning with retryDelay
  waitWithContext(retryDelay)  // cancellable
  retryDelay *= 2
  if retryDelay > maxDelay: retryDelay = maxDelay
  continue (retry collectOnce)

On success:
  retryCount = 0
  retryDelay = baseDelay
  SetReady()
  if hasMore: continue immediately
  else: wait ticker or ctx cancellation
```

**Key detail**: `retryCount` exceeds `maxRetries` means the **6th** consecutive failure triggers the fatal error (retryCount goes 1,2,3,4,5,6 and 6 > 5). The error message says `attempts=5` because it reports `retryCount-1`.

### NFR-2.5 Graceful Shutdown: UPDATED
The `Run()` function has `defer c.reporter.SetNotReady()` at the top. This means:
- On ANY exit from Run() (success, error, or context cancellation), readiness is set to false
- The health server continues running during the 5s shutdown window
- During that window, `/ready` returns 503 while `/health` returns 200 (alive but not ready)

### NFR-3.5: Sequential Collection Impact (REFINED)
- All 9 sources polled in sequence means a single `collectOnce()` cycle could take up to 9 * 30s = 270s (9 API calls at 30s timeout each) in worst case
- With `hasMore`, a single source could dominate the collection cycle until exhausted
- No time-boxing per source -- a slow API response blocks all sources

---

## Additional NFR Findings from CI/CD

### NFR-8: Build and Release (NEW CATEGORY)

### NFR-8.1: Reproducible Builds
- Docker image uses `--mount=type=cache` for Go module and build cache
- `CGO_ENABLED=0` for static binary (no glibc dependency)
- `-trimpath` removes local paths from binary
- `-ldflags "-s -w"` strips debug info and symbol table

### NFR-8.2: Image Supply Chain
- Base Go image from internal ECR mirror (not Docker Hub directly)
- Runtime: `gcr.io/distroless/static-debian12:nonroot`
- OCI labels for source, revision, creation date

### NFR-8.3: Vulnerability Management
- `govulncheck` in CI (daily + PR)
- `gosec` static security analysis
- `staticcheck` static analysis
- `renovate.json` for automated dependency updates
- **All security scanners are non-blocking** -- they report but do not fail the build

### NFR-8.4: Test Pipeline
- CI runs `go test -v ./...` (all packages, not just collector)
- Build timeout: 5 minutes
- Test timeout: 10 minutes
- Test results uploaded as artifacts

### NFR-8.5: Image Registry
- Primary: Cloudsmith (`docker.cloudsmith.io/1898-and-co/poller-bear/poller-bear`)
- Only pushed on main branch merge
- Tagged with: appVersion, short SHA, `latest`

---

## NFR Summary Matrix

| ID | Category | Item | Status | Test Coverage |
|----|----------|------|--------|---------------|
| 1.1 | Security | TLS 1.2 minimum | Implemented | transport_test.go |
| 1.2 | Security | Secret file pattern | Implemented | config_test.go |
| 1.3 | Security | Runtime hardening | Implemented | N/A (infra) |
| 1.4 | Security | CI security scanning | Implemented | N/A (CI) |
| 1.5 | Security | Rate limiting | **MISSING** | - |
| 1.5b | Security | 429 handling | **MISSING** | - |
| 1.5c | Security | Credential rotation | **MISSING** | - |
| 2.1 | Reliability | Retry with backoff | Implemented | **UNTESTED** |
| 2.2 | Reliability | Atomic state writes | Implemented | file_store_test.go |
| 2.3 | Reliability | Forward progress | Implemented | collector_test.go (partial) |
| 2.4 | Reliability | At-least-once delivery | Implemented | collector_test.go |
| 2.5 | Reliability | Graceful shutdown | Implemented | (no test) |
| 2.6 | Reliability | Fingerprint drift detection | Implemented | store_test.go |
| 2.7 | Reliability | Circuit breaker | **MISSING** | - |
| 3.1 | Performance | Connection pooling | Implemented | transport_test.go |
| 3.2 | Performance | Timeout architecture | Implemented | transport_test.go |
| 3.3 | Performance | Page size tuning | Partial (hardcoded) | - |
| 3.4 | Performance | Sink batching | **MISSING** | - |
| 3.5 | Performance | Parallel collection | **MISSING** | - |
| 4.1 | Observability | Structured logging | Implemented | - |
| 4.2 | Observability | Health endpoints | Implemented | server_test.go |
| 4.3 | Observability | Profiling | Implemented | pprof_test.go |
| 4.4 | Observability | Batch receipts | Implemented | file_store_test.go |
| 4.5 | Observability | Metrics export | **MISSING** | - |
| 4.5b | Observability | Distributed tracing | **MISSING** | - |
| 5.1 | Scalability | Single instance only | By design | - |
| 7.1 | Isolation | Fail-fast ordering | By design | - |
| 7.2 | Isolation | Source-level retry | **MISSING** | - |
| 7.4 | Isolation | Panic recovery | Partial (OCSF only) | - |
| 8.1 | Build | Reproducible builds | Implemented | N/A (CI) |
| 8.3 | Build | Vuln management | Implemented | N/A (CI) |

---

## Delta Summary
- New items added: 4 failure isolation NFRs (NFR-7 category), 5 build/release NFRs (NFR-8 category), retry count off-by-one detail
- Existing items refined: Retry strategy with exact code flow, graceful shutdown with defer SetNotReady, sequential collection worst-case timing
- Remaining gaps: None significant

## Novelty Assessment
Novelty: SUBSTANTIVE
The fail-fast collection ordering (NFR-7.1) is architecturally significant -- it means a single flaky Claroty endpoint can block all 9 data sources. The build/release NFRs (ECR mirror, Cloudsmith, non-blocking security scans) are new infrastructure findings. The retry count off-by-one (6th failure triggers fatal, not 5th) is a behavioral detail that matters for spec accuracy.

## Convergence Declaration
Pass 4 is approaching convergence. The major NFRs are fully cataloged. Another round could investigate config loading edge cases but would likely be NITPICK.

## State Checkpoint
```yaml
pass: 4
round: 2
status: complete
files_scanned: 30
timestamp: 2026-04-14T00:05:00Z
novelty: SUBSTANTIVE
```
