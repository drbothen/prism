# Pass 4 Deep: NFR Catalog -- Round 2

**Project:** poller-coaster
**Date:** 2026-04-13
**Basis:** Round 1 outputs + hallucination audit + cross-reference with Pass 0/1/2/3 findings

---

## Hallucination Audit from Round 1

### Verified Claims

1. **NFR-S-022 "io.LimitReader(resp.Body, 2048)"** -- VERIFIED: http_sender.go:198 `io.ReadAll(io.LimitReader(resp.Body, 2048))`.
2. **NFR-R-009 "100 req/s per IP, burst 20"** -- VERIFIED: health/server.go:21-22 constants `defaultrequests = 100`, `defaultburst = 20`.
3. **NFR-S-017 "GH Action pin to SHA"** -- VERIFIED: all workflow files use commit SHAs for actions (e.g., `actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd`).
4. **NFR-P-009 "Trimmed binary"** -- VERIFIED: Dockerfile:25 `-trimpath -ldflags "-s -w"`, Makefile:16 `-ldflags="-s -w"`.
5. **"Pprof shutdown 5s + runner shutdown 5s = 10s worst case"** -- VERIFIED: main.go:40-44 has pprof 5s timeout, runner.go:111 has health 5s timeout. But these run sequentially (pprof deferred after runner returns), so worst case is indeed 10s total.
6. **Health server timeouts** -- VERIFIED all 4 values from server.go:17-20.
7. **Pprof server timeouts** -- VERIFIED all 4 values from pprof.go:22-25. WriteTimeout is 120s (not just "2 min") for long-running CPU profiles.

### Corrections

1. **"RBAC grants watch on secrets -- unused"** -- Claim is accurate for the Go application code. However, the RBAC permission may be used by Kubernetes itself (e.g., for automatic token refresh of service account). The over-provisioning assessment stands for application-level usage.

---

## New Findings from Cross-Reference with Other Passes

### 1. NFR-R-016: Potentially Unused Sentinel Errors (from Pass 0 R2)

4 sentinel errors are defined but likely unused in current code:
- `ErrArmisConfigMissing` -- armis.NewHTTPClient uses plain errors.New, not this sentinel
- `ErrArmisRequestBuild` -- armis package has no request building beyond SDK call
- `ErrArmisUnexpectedStatus` -- SDK handles status codes internally
- `ErrArmisDecode` -- SDK handles JSON decoding internally

These represent **forward-looking error definitions** for a more elaborate Armis client that was never built (or was replaced by the thin SDK wrapper). This is a maintenance burden: defined errors that could mislead developers into thinking they are active.

### 2. NFR-S-023: CI Coverage Threshold is Warning Only

go-test.yml:57 checks `$COVERAGE < 70` but only emits `::warning::` -- it does NOT fail the job. This means coverage can drop below 70% without blocking PRs. The threshold is informational, not enforced.

### 3. NFR-O-010: Go Version Source Inconsistency in CI

security-scan.yml staticcheck job uses `.go-version` while all other jobs use `go.mod` for Go version. If these files diverge, staticcheck would run against a different Go version.

### 4. NFR-P-010: Collector Interval NOT Exposed in Helm

From Pass 1 R2: COLLECTOR_INTERVAL is not in the deployment template despite `collector.interval: 30s` being in values.yaml. The values.yaml field is a dead reference -- it is never used in any template. The actual poll interval can only be configured via extraEnv.

**Correction to Pass 1 R2:** Looking again at the deployment.yaml, I see no template reference to `.Values.collector.interval`. The `collector.containerPort` IS used (line 158), but `collector.interval` is NOT. This means the values.yaml `collector.interval: 30s` is **dead configuration** -- it has no effect.

### 5. NFR-R-017: Health Server Bind Failure Does Not Stop Collector

From Pass 1 R2 shutdown analysis: if the health server fails to bind (port in use), the error is buffered in healthErrCh and only checked AFTER the collector's Run loop completes. The collector runs without health endpoints. This means:
- K8s probes (if enabled) would fail, triggering pod restart
- But if probes are disabled (default), the collector runs silently without health monitoring
- The health bind error is logged only when the collector eventually exits

### 6. NFR Completeness Check Against Broad Sweep

| Broad Sweep NFR | Pass 4 R1 Coverage | Status |
|----------------|-------------------|--------|
| Configurable poll interval | NFR-P-001 | Covered |
| Per-request result limit | NFR-P-002 | Covered |
| Immediate re-poll on hasMore | NFR-P-003 | Covered |
| HTTP client timeout | NFR-P-004, NFR-P-005 | Covered |
| Secret file support | NFR-S-001 | Covered |
| Bearer auth for Armis | NFR-S-003 | Covered |
| Basic auth for sink | NFR-S-004 | Covered |
| Secret redaction | NFR-S-005 | Covered |
| Non-root container | NFR-S-006 | Covered |
| Read-only root filesystem | NFR-S-007 | Covered |
| Drop all capabilities | NFR-S-008 | Covered |
| Seccomp profile | NFR-S-009 | Covered |
| pprof cmdline blocked | NFR-S-011 | Covered |
| pprof loopback warning | NFR-S-012 | Covered |
| No secrets in logs | NFR-S-013 | Covered |
| Structured JSON logging | NFR-O-001 | Covered |
| Configurable log levels | NFR-O-002 | Covered |
| Health endpoints | NFR-O-003 | Covered |
| Batch processing logs | NFR-O-004 | Covered |
| Sink delivery logs | NFR-O-005 | Covered |
| Opt-in pprof | NFR-O-006 | Covered |
| Exponential backoff | NFR-R-001 | Covered |
| Configurable max retries | NFR-R-002 | Covered |
| Atomic file writes | NFR-R-003 | Covered |
| Forward progress invariant | NFR-R-004 | Covered |
| Query fingerprint validation | NFR-R-005 | Covered |
| Graceful shutdown | NFR-R-006 | Covered |
| Receipt auditing | NFR-R-007 | Covered |
| Health state transitions | NFR-R-008 | Covered |
| Rate-limited health | NFR-R-009 | Covered |
| Missing: No rate limiting for API | Missing NFRs table | Covered |
| Missing: No circuit breaker | Missing NFRs table | Covered |
| Missing: No Prometheus metrics | Missing NFRs table | Covered |
| Missing: No distributed locking | Missing NFRs table | Covered |
| Missing: No sink retry | Missing NFRs table | Covered |
| Missing: No mTLS/OAuth for sink | Missing NFRs table | Covered |

All broad sweep NFRs accounted for. No gaps in coverage.

---

## Delta Summary

- New items added: 5 (unused sentinel errors as maintenance burden, CI coverage non-enforcement, dead Helm config field, health bind failure edge case, Go version source inconsistency)
- Existing items refined: 1 (RBAC watch permission context clarified)
- Remaining gaps: None

## Novelty Assessment

Novelty: NITPICK

Round 2 findings are refinements: the dead Helm config field (collector.interval) is informative but the system functions correctly via defaults. The unused sentinel errors are a code quality note, not an architectural gap. The CI coverage non-enforcement and Go version inconsistency are CI hygiene items. None of these would change how you would spec the system's non-functional requirements.

## Convergence Declaration

Pass 4 has converged -- findings are nitpicks, not gaps. All NFRs from the broad sweep are verified and accounted for, with 22+ security NFRs, 10 performance NFRs, 10 observability NFRs, 17 reliability NFRs, and 5 scalability NFRs documented. The missing NFR analysis is comprehensive.

## State Checkpoint

```yaml
pass: 4
round: 2
status: complete
files_scanned: 48
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
convergence: converged
```
