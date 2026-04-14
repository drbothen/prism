# Coverage Audit -- poller-coaster

**Date:** 2026-04-13
**Method:** Grep-driven coverage matrix -- every source file/directory checked against all 13 prior analysis files
**Source:** /Users/jmagady/Dev/prism/.references/poller-coaster/
**Analysis corpus:** 13 files (1 broad sweep + 12 deepening rounds across passes 0-5)

---

## Coverage Matrix

### Go Source Packages

| Package / File | Pass 0 | Pass 1 | Pass 2 | Pass 3 | Pass 4 | Pass 5 | Verdict |
|---------------|--------|--------|--------|--------|--------|--------|---------|
| main.go | YES | YES | YES | YES | YES | YES | COVERED |
| cmd/collector/main.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/app/runner/runner.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/apperrors/errors.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/armis/api.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/collector/collector.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/collector/alert_collector.go | YES | partial | YES | YES | partial | YES | COVERED (no tests noted) |
| internal/collector/activity_collector.go | YES | partial | YES | YES | partial | YES | COVERED (no tests noted) |
| internal/collector/audit_collector.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/collector/risk_factor_collector.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/collector/connection_collector.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/collector/device_collector.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/collector/vulnerability_collector.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/config/config.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/config/utils.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/health/server.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/profiling/pprof.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/sink/sink.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/sink/http_sender.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/state/store.go | YES | YES | YES | YES | YES | YES | COVERED |
| internal/state/file_store.go | YES | YES | YES | YES | YES | YES | COVERED |
| tools/tools.go | YES | n/a | n/a | n/a | n/a | n/a | COVERED (inventory only, SKIP priority) |

### Go Test Files

| Test File | Pass 0 | Pass 3 | Pass 5 | Verdict |
|-----------|--------|--------|--------|---------|
| internal/collector/collector_test.go | YES | YES | YES | COVERED |
| internal/collector/device_collector_test.go | YES | YES | YES | COVERED |
| internal/collector/connection_collector_test.go | YES | YES | YES | COVERED |
| internal/collector/vulnerability_collector_test.go | YES | YES | YES | COVERED |
| internal/collector/audit_collector_test.go | YES | YES | YES | COVERED |
| internal/collector/risk_factor_collector_test.go | YES | YES | YES | COVERED |
| internal/state/file_store_test.go | YES | YES | YES | COVERED |
| internal/state/store_test.go | YES | YES | YES | COVERED |
| internal/config/config_test.go | YES | YES | YES | COVERED |
| internal/health/server_test.go | YES | YES | YES | COVERED |
| internal/profiling/pprof_test.go | YES | YES | YES | COVERED |

### Infrastructure / Config Files

| File | Pass 0 | Pass 1 | Pass 4 | Pass 5 | Verdict |
|------|--------|--------|--------|--------|---------|
| go.mod / go.sum | YES | YES | n/a | n/a | COVERED |
| tools/go.mod / tools/go.sum | YES | n/a | n/a | n/a | COVERED |
| Makefile | YES | YES | YES | n/a | COVERED |
| Dockerfile | YES | YES | YES | n/a | COVERED |
| .golangci.yml | YES | n/a | n/a | YES | COVERED |
| .editorconfig | YES | n/a | n/a | n/a | COVERED |
| .pre-commit-config.yaml | YES | n/a | n/a | YES | COVERED |
| renovate.json | YES | n/a | n/a | n/a | COVERED |
| vector.yaml | YES | n/a | n/a | n/a | COVERED |
| scripts/setup.sh | YES | n/a | n/a | n/a | COVERED |
| scripts/pprof-harness.sh | YES | n/a | YES | n/a | COVERED |
| .go-version | YES | n/a | n/a | n/a | COVERED |
| .python-version | YES | n/a | n/a | n/a | COVERED |
| Brewfile | YES | n/a | n/a | n/a | COVERED |
| .gitmodules | YES | n/a | n/a | n/a | COVERED |
| .gitignore | YES | n/a | n/a | n/a | COVERED |
| .dockerignore | YES | n/a | n/a | n/a | COVERED |

### CI/CD Workflows

| Workflow | Pass 0 | Pass 1 | Pass 4 | Verdict |
|----------|--------|--------|--------|---------|
| .github/workflows/build.yml | YES | YES | YES | COVERED |
| .github/workflows/go-test.yml | YES | YES | YES | COVERED |
| .github/workflows/helm-release.yml | YES | YES | n/a | COVERED |
| .github/workflows/lint-test.yml | YES | YES | n/a | COVERED |
| .github/workflows/security-scan.yml | YES | YES | YES | COVERED |
| .github/workflows/pr-agent.yaml | YES | n/a | n/a | COVERED |
| .github/workflows/validate-codeowners.yml | YES | n/a | n/a | COVERED |

### Helm Chart

| File | Pass 0 | Pass 1 | Pass 4 | Verdict |
|------|--------|--------|--------|---------|
| deploy/helm/poller-coaster/Chart.yaml | YES | YES | n/a | COVERED |
| deploy/helm/poller-coaster/values.yaml | YES | YES | YES | COVERED |
| deploy/helm/poller-coaster/templates/_helpers.tpl | YES | YES | YES | COVERED |
| deploy/helm/poller-coaster/templates/deployment.yaml | YES | YES | YES | COVERED |
| deploy/helm/poller-coaster/templates/pvc.yaml | YES | YES | YES | COVERED |
| deploy/helm/poller-coaster/templates/rbac.yaml | YES | YES | YES | COVERED |
| deploy/helm/poller-coaster/templates/secret.yaml | YES | YES | n/a | COVERED |
| deploy/helm/poller-coaster/templates/service.yaml | YES | YES | n/a | COVERED |
| deploy/helm/poller-coaster/templates/serviceaccount.yaml | YES | YES | n/a | COVERED |
| deploy/helm/poller-coaster/ci/test-values.yaml | NO | NO | NO | **BLIND SPOT** |

### Documentation / Meta Files

| File | Referenced? | Verdict |
|------|-------------|---------|
| README.md | YES (Pass 0) | COVERED (inventory only) |
| CLAUDE.md | YES (Pass 0, Pass 4) | COVERED |
| SECURITY.md | YES (Pass 4) | PARTIAL |
| LICENSE | YES (Pass 0) | COVERED (inventory only) |
| INGESTION.md | NO (it IS the broad sweep) | N/A (duplicate) |
| docs/Deployment.md | NO | **BLIND SPOT** |
| docs/Development.md | NO | **BLIND SPOT** |
| docs/PROFILING.md | NO | **BLIND SPOT** |
| docs/img/pollercoaster.png | NO | N/A (image asset) |
| .github/PULL_REQUEST_TEMPLATE.md | NO | **BLIND SPOT** |
| .github/CODEOWNERS | NO | **BLIND SPOT** |

---

## Blind Spot Analysis

### 1. deploy/helm/poller-coaster/ci/test-values.yaml -- LOW IMPACT

**Content:** Minimal CI test overrides for chart-testing: test API key/URL, test sink credentials, debug logging, probes disabled.

**What analysis missed:** Nothing substantive. The file confirms findings already documented:
- Probes disabled for testing (noted in Pass 1 R1)
- Test credentials (no behavioral contracts affected)
- collector.interval: 30s (matches defaults)

**Impact on model:** None. The file is a CI fixture with no behavioral implications beyond what is already documented.

### 2. docs/Deployment.md -- LOW IMPACT

**Content:** Deployment guide describing Helm values customization, K8s secret creation, chart installation, post-deployment verification, rollback procedures.

**What analysis missed:** Specific deployment workflow (create namespace -> create secrets -> helm install -> verify -> rollback). Documents `values-prod.yaml` pattern (separate file for production overrides).

**New information for the model:**
- Rollback procedure via `helm rollback` is documented but not analyzed
- `helm dependency update` step suggests chart might have sub-chart dependencies (but Chart.yaml shows no dependencies section, so this is a no-op)

**Impact on model:** None. Operational runbook, not behavioral.

### 3. docs/Development.md -- LOW IMPACT

**Content:** Developer guide: prerequisites, repository layout, local environment setup, build/run/test commands, troubleshooting tips.

**What analysis missed:** Nothing substantive. All commands and layout information are already captured in Pass 0 inventory and Pass 5 conventions. The troubleshooting section confirms: delete state file to reset cursors, STATE_STORE_TYPE=memory for dev.

**Impact on model:** None.

### 4. docs/PROFILING.md -- LOW IMPACT

**Content:** Comprehensive profiling guide: enable/disable, available endpoints, pprof-harness script usage, analysis techniques (web UI, CLI, comparison), security notes.

**What analysis missed:** Detailed pprof analysis workflow (comparing profiles, live profiling commands). Also mentions "what to look for" hints:
- JSON marshaling overhead in sink
- TLS handshake costs for Armis API
- Rate limiter map growth (already noted as memory leak in Pass 5)
- Response body buffering via io.ReadAll

**Impact on model:** MINIMAL. The `io.ReadAll` in the API client is a potential performance concern not documented in the NFR catalog. However, this is in the SDK (not the application code), so it is outside the analysis scope.

### 5. .github/PULL_REQUEST_TEMPLATE.md -- NO IMPACT

**Content:** Standard PR template with Why/What/Testing/References sections. Mentions `go test -v ./...` and `go run .` as testing validation steps.

**Impact on model:** None. Process documentation, not behavioral.

### 6. .github/CODEOWNERS -- NO IMPACT

**Content:** `* @1898andCo/application-admins @1898andCo/iac-admins` -- all files owned by two teams.

**Impact on model:** None. Organizational, not behavioral.

### 7. SECURITY.md -- PARTIAL COVERAGE

**Content:** Security policy, vulnerability reporting process, security best practices. Referenced once in Pass 4 but not fully analyzed.

**What analysis missed:**
- **Trivy scanning:** SECURITY.md documents 4-layer scanning (Trivy container, gosec code, govulncheck deps, Trivy repo). The analysis files mention gosec/govulncheck/staticcheck (from security-scan.yml) but do NOT mention Trivy. However, Trivy is NOT in the actual CI workflows -- it appears to be aspirational documentation or done externally.
- **Credential rotation policy:** "Rotate credentials regularly (minimum every 90 days)" -- operational policy not captured in NFR catalog.
- **State management note:** SECURITY.md says "Currently uses in-memory state storage" and "State is not persisted between restarts" -- this is INCORRECT per the actual codebase (FileStore IS the default). This is a documentation bug.
- **Resource limits example:** Production values example includes `cpu: 500m, memory: 512Mi` resource limits -- not in the Helm chart defaults (no resources section in values.yaml).

**New findings for model:**
- BC-AUDIT-001: SECURITY.md documents incorrect state storage behavior (claims in-memory only; actual default is FileStore with persistence)
- NFR-AUDIT-001: Trivy scanning claimed in SECURITY.md but absent from CI workflows (aspirational or external)
- NFR-AUDIT-002: No resource limits in default values.yaml (only in SECURITY.md example)

---

## Gap Entities and Contracts

### BC-AUDIT-001: SECURITY.md contains stale state management documentation

**Preconditions:** SECURITY.md "Known Security Considerations > State Management" section
**Current claim:** "Currently uses in-memory state storage" and "State is not persisted between restarts"
**Actual behavior:** Default `STATE_STORE_TYPE` is "file", using FileStore with atomic JSON persistence. State DOES survive restarts.
**Evidence:** config.go DefaultConfig sets Type: "file"; file_store.go implements durable persistence; TestFileStore_AllSevenStatesSurviveRestart verifies restart survival
**Impact:** Documentation bug. Could mislead operators into thinking state is ephemeral.
**Confidence:** HIGH

### BC-AUDIT-002: Helm test-values.yaml uses dead configuration field

**Preconditions:** ci/test-values.yaml sets `collector.interval: 30s`
**Actual behavior:** Per Pass 4 R2, `collector.interval` in values.yaml is dead configuration -- no template references it. The actual interval is controlled only via COLLECTOR_INTERVAL env var (or its default).
**Evidence:** Pass 1 R2 and Pass 4 R2 document this. test-values.yaml perpetuates the dead field.
**Impact:** Misleading to CI test authors who think they are configuring the poll interval.
**Confidence:** HIGH

### NFR-AUDIT-001: Trivy scanning claimed but not in CI

**Preconditions:** SECURITY.md documents 4-layer scanning including "Container Scanning (Trivy)" and "Repository Scanning (Trivy)"
**Actual CI:** security-scan.yml runs gosec, govulncheck, staticcheck only. build.yml builds Docker but does not scan with Trivy.
**Impact:** Security documentation overstates CI scanning capability. Trivy may be run externally or planned but not implemented.
**Confidence:** HIGH

### NFR-AUDIT-002: No default resource limits in Helm chart

**Preconditions:** SECURITY.md recommends `resources.limits: {cpu: 500m, memory: 512Mi}`
**Actual Helm chart:** values.yaml has no `resources` section at all. No defaults for CPU/memory requests or limits.
**Impact:** Default deployments run without resource constraints, which could allow unbounded memory growth (relevant given rate limiter memory leak in health server).
**Confidence:** HIGH

---

## Coverage Statistics

| Category | Total Files | Covered | Partial | Blind Spot | N/A |
|----------|------------|---------|---------|------------|-----|
| Go source files | 21 | 21 | 0 | 0 | 0 |
| Go test files | 11 | 11 | 0 | 0 | 0 |
| Infrastructure/config | 17 | 17 | 0 | 0 | 0 |
| CI/CD workflows | 7 | 7 | 0 | 0 | 0 |
| Helm chart | 10 | 9 | 0 | 1 | 0 |
| Documentation/meta | 11 | 3 | 1 | 5 | 2 |
| **Total** | **77** | **68** | **1** | **6** | **2** |

**Coverage rate:** 68/77 = 88.3% fully covered, 69/77 = 89.6% including partial.

All 6 blind spots are documentation/meta files with LOW or NO behavioral impact. The single partial-coverage file (SECURITY.md) yielded 2 substantive findings (stale documentation, missing Trivy).

---

## Cross-Pass Consistency Check

| Check | Result |
|-------|--------|
| Pass 3 behavioral contracts align with Pass 2 domain model? | YES -- all 78 contracts reference entities from Pass 2 |
| Pass 4 NFRs match Pass 1 architecture decisions? | YES -- single-instance constraint, sequential collection, per-record delivery all aligned |
| Pass 5 conventions consistently applied per Pass 1? | YES with documented exceptions (8 partially-consistent patterns) |
| Orphaned modules with no behavioral contracts? | YES -- runner.go has no contracts (orchestration-only, no testable behavior beyond wiring) |
| Domain entities with no tests? | YES -- AlertCollector and ActivityCollector have 0 dedicated tests (noted in Pass 3 R1 and Pass 5 R1) |
| Any source file NOT read by any analysis pass? | NO -- all 33 Go files referenced in at least one pass |

---

## Final Assessment

**PASS** -- The coverage audit reveals no substantive gaps in the source code analysis. All 33 Go files (22 source + 11 test), all 7 CI workflows, and 9/10 Helm templates are fully covered across the analysis passes.

The 6 blind spots are all documentation/meta files (docs/*.md, CODEOWNERS, PR template, ci/test-values.yaml) with no behavioral impact. The partial coverage of SECURITY.md yielded 2 genuine findings (stale state management docs, missing Trivy scanning) that have been captured as BC-AUDIT and NFR-AUDIT contracts above.

The prior analysis is comprehensive and accurate. No additional deepening rounds are needed.

---

## State Checkpoint

```yaml
phase: B.5
status: PASS
total_files: 77
covered: 68
partial: 1
blind_spots: 6
blind_spot_impact: all LOW or NO behavioral impact
new_contracts: 2 (BC-AUDIT-001, BC-AUDIT-002)
new_nfrs: 2 (NFR-AUDIT-001, NFR-AUDIT-002)
timestamp: 2026-04-13T00:00:00Z
```
