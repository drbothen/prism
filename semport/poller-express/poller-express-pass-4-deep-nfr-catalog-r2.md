# Pass 4 Deep: NFR Catalog -- poller-express (Round 2)

## Audit of Round 1 Claims

### NFR-SEC-001: Container Hardening
Verified all 8 controls from Dockerfile and values.yaml. All correct.

### NFR-PERF-004: HTTP Timeouts
Round 1 listed 8 timeout configurations. Verified:
- Cyberint API client: 30s (runner.go:60) -- Correct
- Vector sink client: 15s default (http_sender.go:59) -- Correct
- Health server ReadHeaderTimeout: 10s (server.go:17) -- Correct
- Health server ReadTimeout: 15s (server.go:18) -- Correct
- Health server WriteTimeout: 15s (server.go:19) -- Correct
- Health server IdleTimeout: 60s (server.go:20) -- Correct
- Health shutdown: 5s (runner.go:173) -- Correct
- Pprof shutdown: 5s (main.go:37) -- Correct

All 8 values confirmed.

### NFR-REL-001: MaxRetries=0 Behavior
Round 1 stated "MaxRetries=0 disables the retry limit." Verified from both collectors: `c.cfg.Collector.MaxRetries > 0 && retryCount > c.cfg.Collector.MaxRetries`. When MaxRetries=0, the first condition is false, so the retry check is never reached. Confirmed.

### Unused Sentinel Errors as NFR Gap
5 sentinel errors (`ErrCyberIntConfigMissing`, `ErrCyberIntRequestBuild`, `ErrCyberIntUnexpectedStatus`, `ErrCyberIntDecode`, `ErrConfigLoad`) are defined but unused. This suggests planned error handling paths that were never implemented. Specifically:
- No CyberInt configuration validation error (config validation uses plain `fmt.Errorf`)
- No request build error wrapping in the alert collector (uses generated client which handles this internally)
- No unexpected status handling (generated client handles HTTP errors internally)
- No decode error wrapping (generated client handles JSON decoding)
- No config load wrapping (LoadFromEnvironment returns plain `fmt.Errorf`)

This means the error taxonomy is OVER-specified -- the sentinel errors define categories that the code does not actually distinguish.

### Asset Client Error Patterns vs NFR
The asset client (`internal/asset/client.go`) does NOT use sentinel errors from apperrors. All errors are plain `fmt.Errorf` wraps. This is inconsistent with the rest of the codebase where sentinel errors are the norm. Implications:
- Callers cannot use `errors.Is()` to distinguish asset API errors from other errors
- The collector wraps all asset errors as `ErrCyberIntRequestExec` regardless of whether it's a marshal error, network error, or API error

### HTTP Status Code Handling Inconsistency (New NFR Finding)

| Component | Error Threshold | Effect |
|-----------|----------------|--------|
| Asset client | `>= 300` | 3xx redirects treated as errors |
| Sink (HTTPSender) | `>= 400` | 3xx responses silently accepted |

This discrepancy means: if the Cyberint API returns a 301/302 redirect, the asset client fails. If Vector returns a 301/302, the sink silently accepts it (but the response body would not be the expected JSON). Neither component follows redirects explicitly (both use `http.Client` which follows redirects by default for GET but not POST).

**Correction**: Since both Cyberint and Vector use POST requests, and Go's `http.Client` does NOT follow redirects for POST by default, a 3xx response would be returned as-is to the caller. The asset client would error; the sink would silently succeed. This is a latent bug -- if either endpoint starts redirecting, behavior differs.

---

## Additional NFR Patterns Found in Round 2

### NFR-SEC-008: gosec Suppression Inventory

Only 2 `nolint:gosec` directives in the entire hand-written codebase:
1. `config.go:19`: `CYBERINT_API_KEY_FILE` constant (G101: hardcoded credentials false positive)
2. `config.go:21`: `CYBERINT_API_KEY` constant (G101: hardcoded credentials false positive)

Both are justified false positives (env var names, not actual secrets). Clean security posture.

### NFR-REL-007: Error Propagation Taxonomy

Errors fall into 3 categories by how the collector handles them:

| Category | Errors | Behavior |
|----------|--------|----------|
| Fatal (non-retryable) | `ErrQueryFingerprintMismatch`, `ErrCollectorRetriesExceeded` | Process exits |
| Retryable | `ErrCyberIntRequestExec`, `ErrSinkDelivery`, `ErrCursorRegression` | Exponential backoff retry |
| Bootstrap | `ErrStateNotFound` | Triggers state initialization (not an error path) |

The collector does NOT distinguish between retryable errors. All errors in the retry category get the same exponential backoff treatment. There is no separate fast-retry path for transient errors vs slow-retry for persistent errors.

### NFR-OBS-006: Log Field Consistency

Examining structured log fields across all packages:

| Field | Used In | Consistent? |
|-------|---------|-------------|
| `error` | All packages | YES |
| `type` | collector, sink | YES (always "cyberint_alert" or "cyberint_asset") |
| `endpoint` | sink | YES |
| `id` / `alert_id` / `asset_id` | collector, sink | **NO** -- sink uses "id", collectors use "alert_id"/"asset_id" |
| `count` | collector | YES |
| `size_bytes` | sink | YES |
| `status` | sink | YES |
| `retry_in` | collector | YES |
| `customer_id` | asset collector | YES |

**Inconsistency**: Record ID field name varies: `id` in sink, `alert_id` and `asset_id` in collectors. This makes log correlation harder.

### NFR-MAINT-004: Test Coverage Architecture

From CI workflow `go-test.yml`:
- Tests run with `-race` flag (race condition detector)
- Coverage profile generated (`-coverprofile=coverage.out`)
- 70% threshold is a **warning**, not a failure
- Coverage HTML report uploaded as artifact
- No branch-level or function-level coverage thresholds

---

## Delta Summary
- New items added: Error propagation taxonomy (3 categories), HTTP status code handling inconsistency, log field name inconsistency, gosec suppression inventory, test coverage architecture
- Existing items refined: Verified all 8 timeout values, confirmed MaxRetries=0 behavior, documented unused sentinel error implications
- Remaining gaps: None significant

## Novelty Assessment
Novelty: NITPICK
The error propagation taxonomy provides useful structure but the underlying behavior was already documented in Pass 3 behavioral contracts. The HTTP status code inconsistency (300 vs 400) is a minor edge case. The log field naming inconsistency is a style issue. The unused sentinel error analysis is interesting but does not change the NFR catalog -- it is a code quality observation. No new NFR categories or missing NFRs were discovered beyond what Round 1 identified.

## Convergence Declaration
Pass 4 has converged -- findings are nitpicks, not gaps. The NFR catalog is complete for specification purposes.

## State Checkpoint
```yaml
pass: 4
round: 2
status: complete
timestamp: 2026-04-14T00:00:00Z
novelty: NITPICK
```
