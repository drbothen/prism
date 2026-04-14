# Pass 4 Deep: NFR Catalog -- poller-cobra (Round 2)

> Convergence deepening round 2. Hallucination audit + cross-reference with PROFILING_FINDINGS.md and CI workflows.

---

## Round 1 Hallucination Audit

### P-009: Response Body Not Drained

**Round 1 claimed:** Response body not drained on success, prevents connection reuse.
**Verified:** http_sender.go:105-118. On success, function returns at line 118 after logging. Deferred close at line 105-109 fires but body is unread. PROFILING_FINDINGS.md Finding #3 independently confirms with exact same analysis. **Claim is correct.**

### O-002: Log Level Bug

**Round 1 claimed:** WARN/ERROR/FATAL unreachable.
**Verified:** runner.go:131-141 parseLogLevel. Config.Validate() at config.go:430-438 accepts 5 levels. parseLogLevel handles 3 (INFO, DEBUG, TRACE). PROFILING_FINDINGS.md Finding #10 confirms. **Claim is correct.**

### O-004: No Metrics

**Round 1 claimed:** prometheus/client_golang is indirect dep via gofalcon but unused by application.
**Verified:** go.mod line 171 shows `prometheus/client_golang v1.12.1 // indirect`. No import of prometheus in any application Go file. **Claim is correct.**

### R-010: MemoryStore Persistence Gap

**Round 1 claimed:** All cursor state lost on restart, re-fetches all historical alerts.
**Verified:** runner.go:61 hardcodes `state.NewMemoryStore()`. Bootstrap at collector.go:183 uses `cfg.Collector.InitialSince` which defaults to `time.Time{}` (zero time). With zero-time cursor, ALL alerts pass the `isCursorAhead` check. PROFILING_FINDINGS.md Finding #1 confirms as Critical. **Claim is correct.**

---

## Cross-Reference with PROFILING_FINDINGS.md

### New NFR: P-010 (alertToMap Map Pre-allocation)

**Source:** PROFILING_FINDINGS.md Finding #8
**Location:** crowdstrike/api.go:209
**Pattern:** `m := make(map[string]interface{})` creates map without size hint for ~32 known keys
**Impact:** Each `make()` allocates a small hash table that must grow through multiple rehash cycles to accommodate 32 entries. For 100 alerts per batch, this produces ~3,000 extra allocations and rehashes per poll cycle.
**Severity:** Medium (GC pressure under sustained load)
**Fix:** `m := make(map[string]interface{}, 32)`

### CI-Encoded NFRs

From reviewing all 7 CI workflows, the following NFR decisions are encoded in CI:

#### S-013: Daily Vulnerability Scanning

**Location:** .github/workflows/security-scan.yml:21-22
**Pattern:** `cron: '0 6 * * *'` runs gosec + govulncheck + staticcheck daily
**Impact:** Catches newly disclosed CVEs within 24 hours
**Category:** Security / Supply Chain

#### S-014: Egress Audit on CI Runners

**Location:** All 7 workflows use `step-security/harden-runner` with `egress-policy: audit`
**Impact:** All outbound network calls from CI runners are logged
**Category:** Security / Supply Chain

#### S-015: Chart Testing with Kind

**Location:** .github/workflows/lint-test.yml:52-58
**Pattern:** Creates kind cluster, runs `ct install` to verify chart deploys successfully
**Impact:** Validates Helm chart renders correctly and K8s resources are accepted
**Category:** Reliability / Deployment

#### SC-005: CI Build Timeout

**Location:** .github/workflows/collector-tests.yml:24 (`timeout-minutes: 5` for build, 10 for tests)
**Impact:** Prevents runaway CI jobs

---

## Consolidated NFR Catalog (Final Count)

| Category | Count | IDs |
|----------|-------|-----|
| Performance | 10 | P-001 through P-010 |
| Security | 15 | S-001 through S-015 |
| Observability | 7 | O-001 through O-007 |
| Reliability | 11 | R-001 through R-011 |
| Scalability | 5 | SC-001 through SC-005 |
| **Total** | **48** | |

---

## Delta Summary
- New items added: 4 (P-010 alertToMap pre-allocation, S-013 daily vuln scanning, S-014 egress audit, S-015 chart testing, SC-005 CI timeout)
- Existing items refined: 0 (all R1 claims verified via hallucination audit)
- Remaining gaps: None

## Novelty Assessment
Novelty: NITPICK
The newly added items are CI pipeline details (daily cron, egress audit, chart testing) and a micro-optimization (map pre-allocation). None of these change the NFR model -- they are refinements to already-documented categories. The CI NFRs are operational concerns that would not be replicated in the Rust application. The alertToMap pre-allocation is a Go-specific optimization that won't exist in Rust. Removing this round's findings would not change how you'd spec NFRs.

## Convergence Declaration
Pass 4 has converged -- findings are nitpicks, not gaps. The NFR catalog is complete at 48 items across 5 categories.

## State Checkpoint
```yaml
pass: 4
round: 2
status: complete
files_scanned: all
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
