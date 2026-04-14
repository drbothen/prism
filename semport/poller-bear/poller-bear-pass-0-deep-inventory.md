# Pass 0 Deep: Inventory -- Round 1

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 1

---

## Audit of Broad Sweep Claims

### Go Version Discrepancy
- Broad sweep title says "Go 1.25.7 (Docker builds with 1.26.1)" -- **confirmed correct**.
  - `go.mod` declares `go 1.25.7`
  - `Dockerfile` ARG `GO_VERSION=1.26.1`
  - `.devcontainer/devcontainer.json` also references `1.26.1`
  - The broad sweep correctly identified this discrepancy.

### File Manifest LOC Corrections
The broad sweep file manifest had estimated LOC. Actual line counts from file reads:

| File | Broad Sweep Est. | Actual | Delta |
|------|------------------|--------|-------|
| `main.go` | 37 | 37 | exact |
| `cmd/collector/main.go` | ~30 | 14 | -16 (significantly smaller) |
| `internal/app/runner/runner.go` | 151 | 150 | -1 |
| `internal/config/config.go` | 597 | 597 | exact |
| `internal/transport/http.go` | 146 | 145 | -1 |
| `internal/health/server.go` | 73 | 72 | -1 |
| `internal/profiling/pprof.go` | 107 | 106 | -1 |
| `internal/apperrors/errors.go` | 55 | 54 | -1 |
| `internal/sink/sink.go` | 26 | 25 | -1 |
| `internal/sink/http_sender.go` | 252 | 251 | -1 |
| `internal/ocsf/detection_finding.go` | 90 | 89 | -1 |
| `internal/ocsf/severity.go` | 17 | 17 | exact |
| `internal/ocsf/config.go` | 98 | 97 | -1 |
| `internal/state/store.go` | 362 | 361 | -1 |

**Correction**: `cmd/collector/main.go` is only 14 lines, not ~30. It is a minimal entry point that calls `runner.Execute` and exits on error.

### Missing Files from Broad Sweep Manifest

The broad sweep listed 19 Go source files but the actual codebase has **37 Go files** (including tests and tools). The missing files break down as:

**Test files not in manifest (16 files):**
- `internal/claroty/http_client_test.go`
- `internal/claroty/http_client_bench_test.go`
- `internal/collector/collector_test.go`
- `internal/config/config_test.go`
- `internal/health/server_test.go`
- `internal/ocsf/config_test.go`
- `internal/ocsf/golden_file_test.go`
- `internal/ocsf/mapper_stub_test.go`
- `internal/ocsf/severity_test.go`
- `internal/profiling/pprof_test.go`
- `internal/sink/http_sender_test.go`
- `internal/sink/http_sender_bench_test.go`
- `internal/sink/http_sender_ocsf_test.go`
- `internal/state/file_store_test.go`
- `internal/state/file_store_bench_test.go`
- `internal/state/store_test.go`
- `internal/transport/http_test.go`

**Tool files not in manifest (1 file):**
- `tools/tools.go` -- build-tagged tool dependency pinning

**Generated mock files referenced by go:generate but not in manifest:**
- `internal/sink/mock_sender.go` (generated from `sink.go`)
- `internal/state/mock_store.go` (generated from `store.go`)

### Sentinel Error Count Correction
- Broad sweep said 13 sentinel errors. Actual count: **14** (`apperrors/errors.go` defines: `ErrStateNotFound`, `ErrQueryFingerprintMismatch`, `ErrCursorRegression`, `ErrCollectorRetriesExceeded`, `ErrCollectorStateLoad`, `ErrCollectorStatePersist`, `ErrClarotyConfigMissing`, `ErrClarotyRequestBuild`, `ErrClarotyRequestExec`, `ErrClarotyUnexpectedStatus`, `ErrClarotyDecode`, `ErrSinkConfigMissing`, `ErrSinkRequestBuild`, `ErrSinkDelivery`, `ErrConfigLoad`). That is **15** errors.
- Recount: The file has exactly 15 `var` entries. Broad sweep claimed 13, which is **2 short**. Missing: `ErrCollectorStatePersist` and `ErrConfigLoad`.

---

## New Discoveries: Non-Go File Inventory

### Configuration/Infrastructure Files

| Path | Type | Purpose |
|------|------|---------|
| `.editorconfig` | Config | Indent style: tabs for Makefiles, spaces for Markdown/Terraform |
| `.pre-commit-config.yaml` | Config | Pre-commit hooks: go-fumpt, go-build-mod, go-mod-tidy, trailing-whitespace, end-of-file-fixer, check-added-large-files (500KB) |
| `.golangci.yml` | Config | Linter config: v2 format, 13 linters enabled (errcheck, goconst, gocritic, gosec, govet, ineffassign, misspell, nolintlint, revive, staticcheck, unconvert, unused, whitespace), gofumpt+goimports formatters |
| `.python-version` | Config | Python version pin (for chart-testing CI) |
| `Brewfile` | Config | Homebrew dependencies for local dev |
| `renovate.json` | Config | Dependency update automation |
| `.gitmodules` | Config | Git submodule reference |
| `.dockerignore` | Config | Docker build context exclusions |

### CI/CD Workflows (6 total)

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `build.yml` | push(main), PR, release | Build Docker image, push to Cloudsmith |
| `collector-tests.yml` | push(main), PR | Build + run `go test -v ./...` |
| `lint-test.yml` | PR (helm paths only) | Helm chart lint + kind cluster install test |
| `release.yml` | (not read, exists) | Release automation |
| `security-scan.yml` | push(main), PR, daily cron | gosec, govulncheck, staticcheck |
| `validate-codeowners.yml` | (exists) | CODEOWNERS validation |

### Helm Chart (version 1.18.0, appVersion 1.18.0)

7 template files: `_helpers.tpl`, `deployment.yaml`, `pvc.yaml`, `rbac.yaml`, `secret.yaml`, `service.yaml`, `serviceaccount.yaml`

### Documentation Files

| Path | Purpose |
|------|---------|
| `docs/Deployment.md` | Deployment guide |
| `docs/Development.md` | Development guide |
| `docs/PROFILING.md` | pprof profiling reference |
| `docs/specs.json` | Claroty API specification |
| `ocsf-schema/detection-finding-2004.json` | OCSF schema reference |
| `ocsf-schema/README.md` | OCSF schema documentation |

### OCSF Test Data (6 files)

- `internal/ocsf/testdata/input/alert-basic-high.json`
- `internal/ocsf/testdata/input/alert-critical-mitre.json`
- `internal/ocsf/testdata/input/alert-low-no-endpoints.json`
- `internal/ocsf/testdata/golden/alert-basic-high.json`
- `internal/ocsf/testdata/golden/alert-critical-mitre.json`
- `internal/ocsf/testdata/golden/alert-low-no-endpoints.json`

### OCSF Embedded Data Files

- `internal/ocsf/data/severity-map.yaml`
- `internal/ocsf/data/severity-adjustments.yaml`

### Legacy Python Codebase (preserved)

| Path | Purpose |
|------|---------|
| `legacy/pyproject.toml` | Python project config |
| `legacy/python/main.py` | Python entry point |
| `legacy/python/poller_bear/classes.py` | Domain types |
| `legacy/python/poller_bear/config.py` | Config module |
| `legacy/python/poller_bear/xdome.py` | xDome API client |
| `legacy/python/Dockerfile` | Python Docker image |
| `legacy/python/docker-compose.yaml` | Local dev compose |
| `legacy/python/Makefile.legacy` | Python build |
| `legacy/python/chart/` | Legacy Helm chart |
| `legacy/requirements/` | pip requirements |
| `legacy/sources.yaml` | Source definitions |

### Scripts

| Path | Purpose |
|------|---------|
| `scripts/setup.sh` | Environment variable setup for local dev |
| `scripts/pprof-harness.sh` | pprof profiling helper |

---

## Dependency Graph (Updated)

### Direct Dependencies (from `go.mod`)

| Dependency | Version | Purpose |
|------------|---------|---------|
| `github.com/charmbracelet/log` | v0.4.2 | Structured JSON logging |
| `github.com/santhosh-tekuri/jsonschema/v6` | v6.0.2 | JSON schema validation (OCSF) |
| `github.com/stretchr/testify` | v1.11.1 | Test assertions |
| `gopkg.in/yaml.v3` | v3.0.1 | YAML parsing (OCSF config) |

### Tool Dependencies (from `tools/go.mod`)

| Tool | Purpose |
|------|---------|
| `github.com/golangci/golangci-lint/v2` | Linting |
| `go.uber.org/mock/mockgen` | Mock generation |
| `golang.org/x/tools/cmd/stringer` | Enum string generation |
| `golang.org/x/vuln/cmd/govulncheck` | Vulnerability scanning |

Note: `jsonschema/v6` was not mentioned in the broad sweep at all. It is used for OCSF schema validation.

---

## Complete File Count Summary

| Category | Count |
|----------|-------|
| Go source files (non-test) | 17 |
| Go test files | 17 |
| Go benchmark test files | 3 (embedded in test files) |
| Go tool files | 1 |
| Helm templates | 7 |
| CI workflows | 6 |
| Config/meta files | ~12 |
| Documentation | 6 |
| OCSF data/test fixtures | 8 |
| Legacy Python files | ~15 |
| Scripts | 2 |
| **Total non-legacy Go** | **35** |

---

## Delta Summary
- New items added: 15 sentinel errors (corrected from 13), 6 CI workflows cataloged, ~30 non-Go files inventoried, 1 missing dependency (jsonschema/v6), 16 test files explicitly cataloged
- Existing items refined: cmd/collector/main.go LOC corrected (30 -> 14), sentinel error count corrected (13 -> 15)
- Remaining gaps: Exact LOC for collector.go, http_client.go, collector_test.go, file_store.go, memory_store.go (large files not counted due to Bash denial)

## Novelty Assessment
Novelty: SUBSTANTIVE
The inventory was missing 15+ files from the manifest, had an incorrect sentinel error count (off by 2), had an incorrect LOC for cmd/collector/main.go, and was completely missing the jsonschema/v6 dependency. These are model-changing corrections.

## Convergence Declaration
Another round needed -- need to verify LOC for the 5 largest files and confirm completeness of dependency graph.

## State Checkpoint
```yaml
pass: 0
round: 1
status: complete
files_scanned: 70+
timestamp: 2026-04-13T23:30:00Z
novelty: SUBSTANTIVE
next_pass: 0-r2
```
