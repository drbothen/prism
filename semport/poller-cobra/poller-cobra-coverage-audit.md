# Coverage Audit -- poller-cobra

> Phase B.5: Grep-driven coverage audit of all prior analysis against the full source tree.
> Executed 2026-04-13.

---

## Source Tree Inventory

### Directories (25 total, excluding .git)

| Directory | File Count | Type |
|-----------|-----------|------|
| `/` (root) | 14 | Config, entry, docs |
| `.claude/shared-rules/` | 0 | Empty submodule |
| `.github/` | 1 | CODEOWNERS |
| `.github/workflows/` | 7 | CI workflows |
| `deploy/helm/poller-cobra/` | 3 | Chart.yaml, values.yaml |
| `deploy/helm/poller-cobra/ci/` | 1 | test-values.yaml |
| `deploy/helm/poller-cobra/templates/` | 6 | K8s templates |
| `docs/` | 2 | PROFILING.md, PROFILING_FINDINGS.md |
| `internal/app/runner/` | 1 | runner.go |
| `internal/apperrors/` | 1 | errors.go |
| `internal/collector/` | 2 | collector.go, alert_collector.go |
| `internal/config/` | 2 | config.go, utils.go |
| `internal/crowdstrike/` | 4 | api.go, source.go, api_test.go, README.md |
| `internal/health/` | 2 | server.go, server_test.go |
| `internal/profiling/` | 2 | pprof.go, pprof_test.go |
| `internal/sink/` | 2 | sink.go, http_sender.go |
| `internal/state/` | 1 | store.go |
| `scripts/` | 2 | setup.sh, pprof-harness.sh |
| `tools/` | 3 | go.mod, go.sum, tools.go |

### Total Files: ~57 (excluding .git, go.sum)

---

## Coverage Matrix

Legend:
- **Y** = Substantively covered (entities, contracts, or patterns extracted)
- **P** = Partially covered (mentioned but not deeply analyzed)
- **N** = Not covered (blind spot)

| Package/Dir | Pass 0 (Inv) | Pass 1 (Arch) | Pass 2 (Domain) | Pass 3 (BC) | Pass 4 (NFR) | Pass 5 (Conv) | Status |
|-------------|:---:|:---:|:---:|:---:|:---:|:---:|--------|
| `main.go` | Y | Y | Y | Y | P | Y | COVERED |
| `internal/app/runner/` | Y | Y | Y | Y | Y | Y | COVERED |
| `internal/apperrors/` | Y | Y | Y | Y | P | Y | COVERED |
| `internal/collector/` | Y | Y | Y | Y | Y | Y | COVERED |
| `internal/config/` | Y | Y | Y | Y | Y | Y | COVERED |
| `internal/crowdstrike/api.go` | Y | Y | Y | Y | Y | Y | COVERED |
| `internal/crowdstrike/source.go` | Y | Y | Y | Y | P | Y | COVERED |
| `internal/health/` | Y | Y | Y | Y | Y | Y | COVERED |
| `internal/profiling/` | Y | Y | P | Y | Y | Y | COVERED |
| `internal/sink/` | Y | Y | Y | Y | Y | Y | COVERED |
| `internal/state/` | Y | Y | Y | Y | Y | Y | COVERED |
| `deploy/helm/` | Y | Y | Y | Y | Y | Y | COVERED |
| `.github/workflows/` | Y | P | N | P | Y | Y | COVERED |
| `docs/PROFILING.md` | P | P | N | N | P | N | PARTIAL |
| `docs/PROFILING_FINDINGS.md` | Y | Y | P | Y | Y | P | COVERED |
| `Dockerfile` | Y | Y | P | P | Y | P | COVERED |
| `Makefile` | Y | P | Y | P | P | Y | COVERED |
| `vector.yaml` | P | Y | P | N | P | N | COVERED |
| `.golangci.yml` | Y | P | N | N | P | Y | COVERED |
| `scripts/setup.sh` | P | N | P | N | N | N | PARTIAL |
| `scripts/pprof-harness.sh` | P | N | N | N | P | N | PARTIAL |
| `tools/` | Y | P | N | N | P | Y | COVERED |
| `renovate.json` | Y | N | N | N | N | N | PARTIAL |
| `INGESTION.md` | P | N | N | N | N | N | PARTIAL |
| `CLAUDE.md` | P | N | N | N | N | N | PARTIAL |
| `SECURITY.md` | P | N | N | N | N | N | PARTIAL |
| `Brewfile` | Y | N | N | N | N | N | PARTIAL |
| `.editorconfig` | P | N | N | N | N | N | PARTIAL |
| `.pre-commit-config.yaml` | Y | N | N | N | N | Y | COVERED |
| `CODEOWNERS` | Y | N | N | N | N | N | PARTIAL |
| `crowdstrike/README.md` | Y | Y | N | N | N | Y | COVERED |

---

## Blind Spot Analysis

### Go Source Packages: NONE MISSED

All 9 Go packages (plus main.go) are substantively covered across all relevant passes:
- `main`, `runner`, `apperrors`, `collector`, `config`, `crowdstrike`, `health`, `profiling`, `sink`, `state`

Every entity, interface, function, and behavioral contract has been extracted. No Go source code blind spots exist.

### Infrastructure Files: MINIMAL GAPS

The following files have only inventory-level (Pass 0) mentions with no deeper analysis. Assessment of whether this matters:

| File | Gap Assessment | Action Needed |
|------|---------------|---------------|
| `docs/PROFILING.md` | Usage guide for pprof -- operational docs, not domain knowledge | NO |
| `scripts/setup.sh` | 3 lines: exports VECTOR_USERNAME, VECTOR_PASSWORD, VECTOR_ENDPOINT for local dev | NO |
| `scripts/pprof-harness.sh` | Shell script for collecting profiles -- tooling, not application behavior | NO |
| `renovate.json` | Dependency automation rules -- DevOps concern, not application domain | NO |
| `INGESTION.md` | Identical content to broad-sweep.md (auto-generated analysis doc placed in repo) | NO |
| `CLAUDE.md` | Claude Code instructions -- meta-documentation for AI tooling | NO |
| `SECURITY.md` | Security policy (vulnerability reporting process) -- organizational policy | NO |
| `Brewfile` | 3 lines: go, gofumpt, vector Homebrew deps | NO |
| `.editorconfig` | Editor formatting rules (indent size, charset) | NO |
| `CODEOWNERS` | 1 line: `* @1898andCo/application-admins @1898andCo/iac-admins` | NO |

**Verdict:** None of these gaps contain domain knowledge, behavioral contracts, architectural patterns, or NFR information that would affect the spec or Rust rewrite. They are operational/tooling artifacts fully covered at the inventory level.

### Test Files: FULLY COVERED

All 3 test files were analyzed in Pass 3 (behavioral contracts) with test-by-test inventory:
- `crowdstrike/api_test.go` -- 4 top-level test functions (10+ subtests)
- `health/server_test.go` -- 12 test functions
- `profiling/pprof_test.go` -- 9 test functions (7+ subtests)

### Helm Chart: FULLY COVERED

All 10 Helm chart files analyzed across Pass 0 (inventory), Pass 2 R2 (domain model), Pass 3 R2 (behavioral contracts), Pass 4 (NFR), and Pass 5 R2 (conventions):
- Migration guards, credential resolution, probe defaults, security context, PVC, RBAC, service
- Dead `collector.interval` value documented
- Three-tier secret resolution pattern documented

### CI Workflows: FULLY COVERED

All 7 workflows analyzed in Pass 0 (inventory), Pass 4 R2 (NFR), and Pass 5 R2 (conventions):
- build.yml, collector-tests.yml, helm-release.yml, lint-test.yml, security-scan.yml, validate-codeowners.yml, version-check.yml
- Runner hardening, pinned SHAs, self-hosted runners, path-scoped triggers documented

---

## Cross-Reference Consistency Check

### Entity Counts Across Passes

| Metric | Pass 0 | Pass 2 | Pass 3 | Consistent? |
|--------|--------|--------|--------|-------------|
| Go files (production) | 14 | 14 | 14 | YES |
| Go files (test) | 3 | 3 | 3 | YES |
| Total Go LOC (production) | 2,245 | -- | -- | N/A (only measured in Pass 0) |
| Total Go LOC (test) | 681 | -- | -- | N/A |
| External deps | 3 | -- | -- | N/A |
| Sentinel errors | -- | -- | 17 | YES (corrected in R2) |
| alertToMap fields | -- | 32 | -- | YES (corrected in R2) |
| Domain entities | -- | 35 | -- | N/A |
| Behavioral contracts | -- | -- | 79 | N/A (68 R1 + 11 R2) |
| NFR catalog items | -- | -- | -- | 48 (Pass 4) |
| Test functions | -- | -- | 25 top-level | YES (corrected in Pass 5 R2) |
| Anti-patterns | -- | -- | -- | 9 (Pass 5) |
| Interfaces | -- | 6 | -- | N/A |

### Known Inconsistencies (Already Documented)

1. **INGESTION.md == broad-sweep.md** -- The repo's INGESTION.md is identical to our broad-sweep output. This is an auto-generated artifact, not a separate document.

2. **docs/PROFILING_FINDINGS.md Finding #6** -- Claims Ping is nil-check-only (stale). Current code has real API call. Documented in Pass 0 R2.

3. **crowdstrike/README.md** -- References outdated API surface (NewClient vs NewHTTPClient, SourceConfig vs CrowdStrikeConfig). Documented in Pass 0 R2 and Pass 5 R2.

4. **Helm values.yaml `collector.interval`** -- Dead configuration (never wired to env var). Documented in Pass 2 R2.

---

## Final Verdict

### Coverage Status: PASS

All Go source packages are substantively covered across all 6 passes plus convergence deepening. No blind spots exist in:
- Application source code (14 production files, 3 test files)
- Domain model (35 entities, 6 interfaces)
- Behavioral contracts (79 contracts)
- NFR catalog (48 items)
- Convention patterns (9 anti-patterns, 7 design patterns)
- Helm chart (10 templates, deployment constraints)
- CI pipeline (7 workflows)

The only files with inventory-only coverage are operational/tooling artifacts (SECURITY.md, Brewfile, .editorconfig, scripts, CODEOWNERS) that contain no domain knowledge relevant to the spec or Rust rewrite.

### No Additional Rounds Needed

The coverage audit confirms that the deepening rounds achieved full coverage of all substantive source material. No blind spots require filling.

---

## State Checkpoint
```yaml
phase: B.5
status: PASS
files_audited: 57
packages_covered: 9/9 Go packages + main.go
blind_spots_found: 0 (substantive)
blind_spots_found_trivial: 10 (operational/tooling files, inventory-only)
action_required: none
timestamp: 2026-04-13T00:00:00Z
```
