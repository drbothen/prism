# Pass 5 Deep: Conventions & Pattern Catalog -- poller-cobra (Round 2)

> Convergence deepening round 2. Hallucination audit + Helm/CI convention patterns.

---

## Round 1 Hallucination Audit

### Test Count Verification

**Round 1 claimed:** 28 test functions across 3 files.
**Verified count:**
- api_test.go: TestHTTPClient_Ping (1 top-level with 6 subtests), TestHTTPClient_FetchAlerts_NilInner, TestHTTPClient_FetchDetections_NilInner, TestHTTPClient_FetchHosts_NilInner = **4 top-level functions** (10 including subtests)
- server_test.go: TestServer_Liveness, TestServer_Readiness_NotReady, TestServer_Readiness_Ready, TestServer_SetReady, TestServer_SetNotReady, TestServer_RateLimiting_AllowsNormalTraffic, TestServer_RateLimiting_BlocksExcessiveTraffic, TestServer_RateLimiting_PerIPIsolation, TestServer_RateLimiting_AllowsAfterWaiting, TestServer_RateLimiting_HandlesInvalidRemoteAddr, TestDefaultRateLimitConfig, TestWithRateLimitConfig = **12 functions**
- pprof_test.go: TestStart_DisabledWhenEnvUnset, TestStart_DisabledWhenSetToFalse, TestStart_LaunchesServer, TestStart_ShutdownStopsServer, TestStart_RespectsCustomAddr, TestStart_DefaultAddrWhenEnvEmpty, TestIsLoopback, TestPprofMux_CmdlineBlocked, TestStart_ReturnsErrorOnBindFailure = **9 functions** (isLoopback has 7 subtests)

**Total top-level test functions:** 4 + 12 + 9 = **25** (not 28). Round 1 likely counted subtests as top-level.

**Corrected:** 25 top-level test functions, 28 total test cases (including Ping's 6 subtests and isLoopback's 7 subtests = 38 total test cases if counting subtests).

### Test Coverage Percentage

**Round 1 claimed:** 30.3% by line count (681/2245). **Corrected LOC:** 684/2259 = 30.3% (ratio unchanged after LOC correction).
**Assessment:** This is test-lines-to-production-lines ratio, not actual code coverage. Actual test coverage would require running `go test -cover`, which we cannot do. The 30.3% is a reasonable proxy metric but should be labeled "test-to-production line ratio," not "coverage." PROFILING_FINDINGS.md estimates coverage at ~10% which likely means actual branch/statement coverage, since many production lines are in untested packages.

### Custom String Search Anti-Pattern

**Round 1 claimed:** api_test.go reimplements `strings.Contains`.
**Verified:** api_test.go:173-184 defines `contains()` and `searchString()`. These are byte-level string search implementations. `strings` package is not imported in api_test.go. **Claim is correct.**

### Constant Naming Inconsistency

**Round 1 claimed:** `defaultrequests` and `defaultburst` should be camelCase.
**Verified:** health/server.go:21-22:
```
defaultrequests = 100
defaultburst = 20
```
These lack camelCase word separation. All other private constants in the codebase use proper camelCase (e.g., `readHeaderTimeout`, `httpReadTimeout`). **Claim is correct.** This would trigger a linting warning if golangci-lint had `stylecheck` or `revive`'s `var-naming` rule enabled with stricter settings.

---

## Helm Chart Convention Patterns

### HC-001: Template Helper Functions

**Location:** `_helpers.tpl`
**Pattern:** 6 template helpers with `poller-cobra.` prefix namespace:
- `poller-cobra.name` (truncate to 63 chars)
- `poller-cobra.fullname` (release-name + chart-name, truncate)
- `poller-cobra.chart` (chart-name-version)
- `poller-cobra.labels` (Helm standard labels)
- `poller-cobra.selectorLabels` (minimal for pod selector)
- `poller-cobra.serviceAccountName`
- `poller-cobra.namespace` (fallback logic: Release.Namespace > Values.namespace > "poller-cobra")
**Consistency:** Standard Helm best practices. All templates use these helpers consistently.

### HC-002: Migration Guards Pattern

**Location:** deployment.yaml:1-16
**Pattern:** `{{- if .Values.crowdstrike.apiKey -}}{{- fail "..." -}}{{- end -}}`
**Purpose:** Hard-fail on deprecated v0.2.0 values
**Consistency:** Applied to all 5 deprecated fields

### HC-003: Secret Resolution Priority

**Location:** deployment.yaml:81-128
**Pattern:** Three-tier credential resolution:
1. `existingSecret` (external K8s secret)
2. `secretName` (chart-managed named secret)
3. Direct value (plaintext in values.yaml)
**Consistency:** Applied to both CrowdStrike and sink credentials

### HC-004: Conditional Resource Creation

**Location:** All templates
**Pattern:** Guard blocks like `{{- if .Values.rbac.create }}`, `{{- if .Values.persistence.enabled }}`
**Consistency:** All optional resources gated by boolean values

### HC-005: Values Structure Convention

**Pattern:** Top-level keys for each concern area:
```yaml
image: {}
crowdstrike: {}
sink: {}
collector: {}
logging: {}
persistence: {}
xmp: {}
podSecurityContext: {}
securityContext: {}
livenessProbe: {}
readinessProbe: {}
serviceAccount: {}
rbac: {}
service: {}
extra*: []  # extensibility hooks
```
**Consistency:** Clean separation. Extensibility via `extraEnv`, `extraVolumeMounts`, `extraVolumes`, `extraInitContainers`, `extraContainers`, `extraEnvFrom`.

---

## CI Workflow Convention Patterns

### CI-001: Runner Hardening First

**Pattern:** Every job starts with `step-security/harden-runner` as first step
**Consistency:** 100% across all 7 workflows, all jobs

### CI-002: Pinned Action SHAs

**Pattern:** All third-party actions referenced by full SHA with version comment
**Example:** `uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6`
**Consistency:** 100% across all workflows

### CI-003: Self-Hosted Runners

**Pattern:** `runs-on: [self-hosted, Ubuntu, Common]`
**Consistency:** All jobs except none -- 100%

### CI-004: Path-Scoped Triggers

**Pattern:** Workflows only trigger on changes to relevant paths
**Examples:**
- build.yml: Dockerfile, main.go, cmd/**, internal/**, go.mod, go.sum
- lint-test.yml: deploy/helm/poller-cobra/**
- security-scan.yml: **.go, go.mod, go.sum
**Consistency:** All workflows use path filtering (except validate-codeowners which triggers on all PRs)

### CI-005: Informational Security Scans

**Pattern:** Security tools run with `--no-fail` or `continue-on-error: true`
**Purpose:** Findings are reported but don't block merge
**Consistency:** gosec uses `--no-fail`, version-check uses `continue-on-error: true`

---

## Stale Documentation Pattern

### SD-001: README Drift

The `crowdstrike/README.md` documents an API surface that does not match the implementation:
- References `NewClient()` (actual: `NewHTTPClient()`)
- References `config.SourceConfig` (actual: `config.CrowdStrikeConfig`)
- Shows `source.client.FetchAlerts()` (client is unexported)
- Describes `Source` integration with collector (Source is unused dead code)

This represents a "documentation-first" development style where the README was written for the intended API before implementation diverged.

### SD-002: PROFILING_FINDINGS.md Drift

Finding #6 describes Ping as nil-check-only, but current Ping makes a real API call. This document was accurate for an earlier code version but was not updated when Ping was enhanced.

---

## Delta Summary
- New items added: 5 Helm chart convention patterns (HC-001 through HC-005), 5 CI workflow convention patterns (CI-001 through CI-005), 2 stale documentation patterns
- Existing items refined: test function count corrected (25 top-level, not 28), test coverage metric relabeled as "line ratio" not "coverage"
- Remaining gaps: None

## Novelty Assessment
Novelty: NITPICK
The Helm and CI convention patterns (HC-001 through HC-005, CI-001 through CI-005) are infrastructure-layer conventions that exist outside the Go application. They are useful for documenting the project's operational standards but do not change the application-level convention model used for the Rust rewrite. The test count correction (25 vs 28) and coverage metric relabeling are arithmetic refinements. The stale documentation patterns are observational, not architectural. Removing this round's findings would not change how you'd spec conventions for the system.

## Convergence Declaration
Pass 5 has converged -- findings are nitpicks, not gaps. The convention catalog covers Go source patterns, test patterns, error handling patterns, design patterns, anti-patterns, Helm chart patterns, and CI pipeline patterns.

## State Checkpoint
```yaml
pass: 5
round: 2
status: complete
files_scanned: all
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
