# Pass 0 Deep: Inventory -- mcp-claroty-xdome (Round 1)

## Overview

This deepening round corrects multiple inventory gaps and errors from the broad sweep. The most significant discovery is that `src2/` is NOT an abandoned alternate TypeScript implementation -- it is a **complete parallel Python implementation** of the same MCP server using fastmcp/Pydantic. The broad sweep also under-counted files, missed the entire CI/CD pipeline, did not document the `.archive/` directory of planned tool definitions, and omitted the `.windsurf/` AI-assisted development configuration.

---

## 1. Critical Correction: src2/ is a Python Implementation

The broad sweep stated: "src2/ directory: Contains an alternate implementation that appears abandoned (19 files in a parallel structure). Not analyzed."

**This is wrong.** `src2/` is a **Python 3.11+ implementation** of the same MCP server:

| Aspect | src/ (TypeScript) | src2/ (Python) |
|--------|-------------------|----------------|
| Language | TypeScript 5.4+ | Python 3.11+ |
| MCP SDK | @modelcontextprotocol/sdk ^1.12.3 | fastmcp >=2.3.0, mcp >=1.12.2 |
| HTTP Client | axios + axios-retry | aiohttp + aiohttp-retry |
| Validation | Zod | Pydantic >=2.0.0 |
| Logging | Winston | structlog >=23.1.0 |
| Build System | tsc | hatchling |
| Test Framework | Vitest | pytest + pytest-asyncio |
| Linting | ESLint + Prettier | ruff |
| Type Checking | TypeScript strict | mypy |
| Package Manager | npm | uv (pyproject.toml) |

**File count:** 41 Python files (17 source + 4 root-level tests + 14 tests/directory tests + 1 __init__.py + 5 config/support files)

**Structural mirror:**
```
src2/
  main.py                          -- Entry point (asyncio-based)
  domain/
    alerts/alert_service.py
    alerts/alerted_device_service.py
    alerts/vulnerability_service.py
    devices/device_service.py
    vulnerabilities/vulnerability_service.py
  integrations/claroty/xdome_api_client.py
  schemas/
    get_alerts_schema.py
    get_alerted_devices_schema.py
    get_devices_schema.py
    get_vulnerabilities_schema.py
    get_vulnerability_devices_schema.py
  server/
    mcp_server.py
    middleware.py
    server_composer.py
    transport_config.py
  tools/
    __init__.py
    alerts_provider.py
    devices_provider.py
    vulnerabilities_provider.py
  resources/
    server_resources.py
  utils/
    cache.py
    errors.py
    logger.py
  tests/ (14 test files)
  test_*.py (4 root-level test files)
  pyproject.toml
  requirements.txt
  .env.example
  .gitignore
  docs/
  README.md
```

**Key architectural difference:** Python version uses `fastmcp` (a higher-level wrapper) instead of the raw `@modelcontextprotocol/sdk`, and has a `transport_config.py` module for SSE vs Streamable HTTP transport selection via config. It also has a `middleware.py` and `server_composer.py` that suggest a different composition pattern than the TypeScript DI approach.

---

## 2. Corrected File Counts

| Directory | Broad Sweep | Actual | Delta |
|-----------|-------------|--------|-------|
| src/*.ts | 37 files | 37 files | Correct |
| tests/*.ts | 34 files | 35 files (including 1 .disabled) | +1 |
| src2/*.py | "19 files" | 41 files | +22 |
| scripts/ | Not counted | 36 files | New |
| .github/ | Not counted | 53 files (20 actions, 33 workflows+configs) | New |
| docs/ | Not counted | ~100+ files (API refs, schemas, guides) | New |
| .archive/ | Not counted | ~63 files (tool definitions, specs, schemas) | New |
| .windsurf/ | Not counted | ~52 files (AI prompts, rules, workflows) | New |
| Config files (root) | Partially counted | 15 files | Corrected |

**Total codebase files:** ~440+ (vs broad sweep's implicit ~71)

---

## 3. Dependency Corrections and Additions

### Production Dependencies (from package.json)

| Dependency | Version | Purpose | Broad Sweep |
|-----------|---------|---------|-------------|
| @modelcontextprotocol/sdk | ^1.12.3 | MCP protocol SDK | Listed |
| axios | ^1.11.0 | HTTP client | Listed |
| axios-retry | ^4.5.0 | Retry middleware | Listed |
| cors | 2.8.5 | CORS middleware | NOT listed in broad sweep tech stack |
| express | ^4.19.2 | HTTP framework | Listed |
| reflect-metadata | ^0.2.2 | Decorator metadata for tsyringe | NOT listed |
| tsyringe | ^4.10.0 | DI container | Listed |
| uuid | ^11.1.0 | UUID generation | NOT listed |
| winston | 3.17.0 | Logging | Listed |
| zod | 3.25.67 | Validation | Listed |

### Dev Dependencies (complete catalog, not in broad sweep)

| Dependency | Version | Purpose |
|-----------|---------|---------|
| @eslint/js | ^9.29.0 | ESLint base rules |
| @modelcontextprotocol/inspector | ^0.15.0 | MCP debugging tool |
| @types/cors | 2.8.19 | CORS type definitions |
| @types/express | ^4.17.21 | Express type definitions |
| @types/node | ^20.12.12 | Node.js type definitions |
| @types/supertest | ^6.0.3 | Supertest type definitions |
| @types/uuid | ^10.0.0 | UUID type definitions |
| @typescript-eslint/eslint-plugin | 8.34.1 | TS ESLint rules |
| @typescript-eslint/parser | 8.34.1 | TS ESLint parser |
| concurrently | ^8.2.2 | Parallel script runner (dev mode) |
| dotenv | ^17.1.0 | .env file loader |
| eslint | 9.29.0 | Linting |
| eslint-config-prettier | 10.1.5 | Prettier ESLint integration |
| eslint-plugin-prettier | 5.5.0 | Prettier ESLint plugin |
| husky | 9.1.7 | Git hooks |
| prettier | 3.5.3 | Code formatting |
| supertest | ^7.1.3 | HTTP assertion testing |
| ts-node | ^10.9.2 | TypeScript execution |
| tsx | ^4.16.2 | TypeScript execution (fast, for dev) |
| typescript | ^5.4.5 | TypeScript compiler |
| typescript-eslint | ^8.34.1 | TS ESLint monorepo |
| vitest | ^3.2.4 | Test framework |
| @vitest/ui | ^3.2.4 | Test UI |
| @vitest/coverage-v8 | ^3.2.4 | V8 coverage provider |
| globals | ^15.0.0 | Global variable definitions |
| vite | ^7.0.5 | Build tool (for Vitest) |

---

## 4. Build and Development Scripts (from package.json)

| Script | Command | Purpose |
|--------|---------|---------|
| `start` | `node dist/server.js` | Production start |
| `prebuild` | `node scripts/generate-version.js` | Auto-generates version.ts from package.json |
| `build` | `tsc` | TypeScript compilation |
| `dev` | `tsx watch src/main.ts` | Dev mode with hot reload |
| `dev:inspector` | concurrently dev + MCP inspector | Dev with protocol inspector |
| `dev:kill` | lsof/xargs kill | Kill dev processes on ports 3000/6274/6277 |
| `test` | `vitest run` | Run all tests |
| `test:unit` | `LOG_LEVEL=fatal vitest run tests/unit/` | Unit tests (silent logging) |
| `test:e2e` | `LOG_LEVEL=fatal vitest run tests/e2e/` | Integration tests |
| `test:watch` | `vitest` | Interactive test mode |
| `test:ui` | `vitest --ui` | Visual test UI |
| `lint` | `eslint . --ext .ts,.js,.cjs,.json` | Linting |
| `lint:fix` | Same with `--fix` | Auto-fix lint issues |
| `verify` | `npm run build && node dist/scripts/verify-server.js` | Smoke test server build |

---

## 5. CI/CD Pipeline (Not in Broad Sweep)

### GitHub Actions Workflows (17 workflows)

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| ci.yml | push/PR to main/develop | Node.js CI matrix (18, 20, 22) + Docker build |
| code-quality.yml | push/PR | Lint, format check, type check, dependency check |
| security.yml | push/PR + weekly Monday 6AM | npm audit via audit-ci |
| hadolint.yml | Dockerfile changes | Dockerfile linting |
| gitflow-automation.yml | Various | GitFlow branch management |
| gitflow-bugfix-automation.yml | bugfix branches | Bugfix workflow |
| gitflow-hotfix-automation.yml | hotfix branches | Hotfix workflow |
| gitflow-release-automation.yml | release branches | Release workflow |
| release-build.yml | Release tags | Production build |
| release-detection.yml | Various | Detect when to release |
| release-docker.yml | Release tags | Docker image publish |
| release-orchestrator.yml | Various | Coordinate release process |
| scheduled-release.yml | Scheduled | Periodic releases |
| auto-merge-back.yml | Post-release | Merge release back to develop |
| enhanced-auto-merge.yml | Various | Smart auto-merge |
| auto-release-check.yml | Various | Release readiness checks |
| validate-codeowners.yml | CODEOWNERS changes | Validate CODEOWNERS file |

### Reusable Workflows (12 reusable)

`_reusable-node-ci.yml`, `_reusable-docker-build.yml`, `_reusable-security-scan.yml`, `_reusable-quality-check.yml`, `_reusable-version-management.yml`, `_reusable-environment-deploy.yml`, `_reusable-performance-test.yml`, `_reusable-performance-optimizer.yml`, `_reusable-pr-management.yml`, `_reusable-retry-handler.yml`, `_reusable-dependency-manager.yml`, `_reusable-workflow-monitor.yml`

### Custom GitHub Actions (12 composite actions)

`setup-node-env`, `setup-git`, `branch-operations`, `check-skip-release`, `check-workflow-status`, `create-gitflow-branch`, `create-gitflow-pr`, `detect-change-type`, `detect-version-type`, `determine-version-bump`, `extract-pr-number`, `get-pr-from-workflow-run`, `manage-auto-merge`, `pr-creation`, `release-notes-generator`, `should-release`, `version-management`, `workflow-monitor`, `workflow-orchestrator`

### CI NFRs
- **Node.js test matrix:** 18, 20, 22
- **Coverage threshold:** 70% branches/functions/lines/statements (vite.config.ts)
- **Coverage reporting:** Codecov + lcov PR comments
- **Security scanning:** audit-ci with explicit vulnerability allowlist (`.audit-ci.jsonc`)
- **Container scanning:** Trivy with ignore list (`.trivyignore`)
- **Dockerfile linting:** Hadolint with SARIF output
- **Runner hardening:** step-security/harden-runner with egress auditing
- **All action versions pinned by SHA** (supply chain security)
- **Self-hosted runners** for security scanning (ubuntu-latest for CI)

---

## 6. Scripts Directory (36 files)

| Category | Files | Purpose |
|----------|-------|---------|
| Version generation | generate-version.js | Extracts version from package.json into src/generated/version.ts |
| Server verification | verify-server.ts | Smoke-test: creates minimal MCP server with mock tool |
| Documentation | generate_docs.py, generate_schemas.py, doc_check.py, stale_doc_check.py, stale_schema_check.py | OpenAPI-to-docs pipeline |
| Development | dev-utils.sh, setup-dev.sh | Developer environment setup |
| Integration testing | get_alerts_test.sh, get_alert_events_test.sh | Manual curl-based API tests |
| Phase validation | validate-phase-{1,2,3}-task-{1,2,3,4}-step-{1-7}.sh (25 files) | Phased implementation validation scripts |

The phase validation scripts reveal a **structured implementation methodology** -- the project was built in 3 phases with multiple tasks and steps per phase, each validated by a script. This is not documented anywhere in the broad sweep.

---

## 7. Documentation Directory (~100+ files)

| Category | Content |
|----------|---------|
| API References (48 files) | Complete xDome REST API endpoint documentation (OpenAPI-derived) |
| Schema References (~50 files) | Schema markdown for every xDome request/response type |
| Guides (2 files) | TOOL_REGISTRATION.md, parameter-descriptions.md |
| Index | docs/index.md |
| Logging | docs/logging.md |
| Link checker | docs/link-checker.sh |

**Key finding:** The xDome API has **48 documented endpoints** but the MCP server only implements **5 of them**. The docs reveal the full xDome API surface including write operations (CMMS assets, custom attributes, device alert status, vulnerability relevance, sites, site groups, edge management, purdue levels, user actions).

---

## 8. .archive/ Directory (Planned Tool Coverage)

The `.archive/tools/definitions/` directory contains tool definition documents for **51 planned tools** across 10 API categories:

| Category | Tools | Status |
|----------|-------|--------|
| Alerts | get-alerts, get-alerted-devices, get-alert-devices, get-device-alert-relations, set-device-alert-status | 2/5 implemented |
| Devices | get-devices, get-device-alert-relations, get-device-vulnerability-relations, set-device-alert-status, set-device-vulnerability-relevance | 1/5 implemented |
| Vulnerabilities | get-vulnerabilities, get-vulnerability-devices, get-vulnerable-devices | 2/3 implemented |
| CMMS | get-assets, add-assets, delete-assets, match-assets, get-matched-assets, get-match-status | 0/6 implemented |
| Custom Attributes | set-attributes, replace-attributes | 0/2 implemented |
| Edge Management | get-locations, add-location, update-location, upload-results | 0/4 implemented |
| OT Activity | get-events | 0/1 implemented |
| Purdue Level | set-level | 0/1 implemented |
| Site Attribution Rules | get-rules, add-rule, patch-rule, delete-rule, update-priorities, are-changes-pending | 0/6 implemented |
| Site Groups | get-groups, add-group, patch-group, delete-group | 0/4 implemented |
| Sites | get-sites, add-site, edit-site, patch-site, delete-site | 0/5 implemented |
| User Actions | set-assignees, replace-assignees, set-labels, replace-labels, set-notes | 0/5 implemented |

**Total planned: ~51 tools; implemented: 5/51 (9.8%)**

This represents significant planned expansion including **write operations**, which the broad sweep noted as absent.

---

## 9. .windsurf/ Directory (AI Development Configuration)

Contains **52 files** of AI-assisted development configuration:
- **Prompts (19):** Implementation planners, API analyzers, tool builders
- **Rules (22):** Core principles, coding standards, security/performance, governance, Git workflow, Jira/Confluence integration, conventional commits
- **Workflows (11):** Architecture enhancement, modularity, testability, deployment, API coverage

This reveals the project was developed with **Windsurf AI** assistance, with structured rules governing code quality, implementation patterns, and project management integration.

---

## 10. TypeScript Configuration Details

| Config File | Purpose | Key Settings |
|-------------|---------|-------------|
| tsconfig.json | Main compilation | target: es2020, module: NodeNext, strict: true, experimentalDecorators, emitDecoratorMetadata, paths: @/* -> src/* |
| tsconfig.eslint.json | ESLint type-checking | Extends main, includes src/ and test/ |
| tsconfig.test.json | Test compilation | Extends main, jsx: react (for Vitest), types: vitest/globals + node |
| tsconfig.jest.json | Legacy (unused) | Extends main, duplicate path aliases |

**Key insight:** `experimentalDecorators` and `emitDecoratorMetadata` are required for tsyringe DI decorators (`@injectable()`, `@singleton()`, `@inject()`). The `reflect-metadata` import in `main.ts` and `tests/setup.ts` enables runtime decorator metadata.

---

## 11. Security Configuration Files

| File | Purpose |
|------|---------|
| `.audit-ci.jsonc` | npm audit vulnerability allowlist (15 known CVEs with comments explaining each) |
| `.trivyignore` | Container scan vulnerability ignorelist (7 CVEs in SDK, axios, ajv, body-parser, qs) |
| `.github/CODEOWNERS` | Code review: @drbothen @Zious11 @arcaven own all files |

---

## 12. Corrected Test File Inventory

| Directory | File Count | Type |
|-----------|-----------|------|
| tests/unit/core/ | 10 | Unit (transport, session, tool registry, connection, health) |
| tests/unit/domain/alerts/ | 3 | Unit (alert, alerted-device, vulnerability services) |
| tests/unit/domain/devices/ | 1 | Unit (device service) |
| tests/unit/domain/vulnerabilities/ | 1 | Unit (vulnerability service) |
| tests/unit/integrations/claroty/ | 2 | Unit (API client, API client alerted-devices) |
| tests/unit/tools/ | 5 | Unit (one per tool handler) |
| tests/unit/utils/ | 3 | Unit (cache, errors, logger) |
| tests/unit/main.test.ts | 1 | Unit (main entry point) |
| tests/unit/server/factory.test.ts | 1 | Unit (factory) |
| tests/unit/server/index.test.ts | 1 | Unit (server index) |
| tests/e2e/ | 4 | Integration (alerts, devices, vulnerability-devices, server-startup) |
| tests/e2e/ | 1 | **Disabled** (sse-transport.e2e.test.ts.disabled) |
| tests/setup.ts | 1 | Setup (reflect-metadata + dotenv) |
| **Total** | **34 active + 1 disabled + 1 setup = 36** | |

The broad sweep stated "34 test files" which undercounts by 2.

---

## 13. Environment Variables (Complete Catalog)

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| CLAROTY_XDOME_BASE_URL | Yes | - | xDome API base URL |
| CLAROTY_XDOME_API_TOKEN | Yes | - | Bearer token for xDome |
| MCP_TRANSPORT_TYPE | No | "sse,http,streamable-http" (all) | Comma-separated transport list |
| PORT | No | 3000 | Express server port |
| LOG_LEVEL | No | "info" | Winston log level |
| NODE_ENV | No | - | Environment (production/development/test) |

---

## Delta Summary
- New items added: Python implementation discovery (src2/ -- 41 files); CI/CD pipeline (17 workflows, 12 reusable workflows, 18+ composite actions); scripts inventory (36 files); documentation inventory (~100+ files); .archive/ planned tools (51 tool definitions, 10 API categories); .windsurf/ AI config (52 files); 3 missing production dependencies (cors, reflect-metadata, uuid); 25 dev dependencies; 12 npm scripts; 6 environment variables; security config files; test file count correction
- Existing items refined: src2/ reclassified from "abandoned TypeScript" to "active Python implementation"; test count corrected from 34 to 36; dependency list now complete
- Remaining gaps: LOC counts could not be computed (Bash sandbox restriction); src2/ Python files not deeply analyzed (would require a separate pass)

## Novelty Assessment
Novelty: SUBSTANTIVE
The Python implementation in src2/, the CI/CD pipeline, the .archive/ planned tool roadmap (showing 5/51 implementation progress), and the full dependency catalog all change how you would spec this system. The broad sweep presented this as a 37-file TypeScript server; the actual codebase is ~440+ files across two languages with an extensive CI/CD pipeline and a documented expansion roadmap.

## Convergence Declaration
Another round needed -- the following substantive gaps remain: (1) LOC counts for src/ and tests/, (2) analysis of src2/ Python implementation architecture differences, (3) verification of which scripts are actively used vs. vestigial, (4) the `docs/references/claroty-xdome/` xDome API coverage gap analysis needs completion.

## State Checkpoint
```yaml
pass: 0
round: 1
status: complete
timestamp: 2026-04-13T23:30:00Z
novelty: SUBSTANTIVE
```
