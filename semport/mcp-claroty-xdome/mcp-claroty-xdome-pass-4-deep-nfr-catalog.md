# Pass 4 Deep: NFR Catalog -- mcp-claroty-xdome (Round 1)

## Overview

This deepening round extends the broad sweep's NFR catalog with discoveries from the CI/CD pipeline, security scanning configuration, test infrastructure, and detailed code inspection. The broad sweep listed NFRs in a table format with limited detail; this round provides precise values, configurations, and identifies NFRs embedded in CI/CD that were completely missed.

---

## 1. Performance NFRs (Refined)

### P-001: API Request Timeout
- **Value:** 15,000ms (15 seconds)
- **Source:** `xdome-api-client.ts:58` -- `timeout: 15000` on axios instance
- **Scope:** All outbound xDome API requests
- **Test Coverage:** No test verifies this specific value (MEDIUM confidence from code)

### P-002: Response Caching TTL
- **Value:** 300,000ms (5 minutes) default
- **Source:** `cache.ts:45` -- `ttlMs: number = 300000` parameter default
- **Scope:** All domain service method results
- **Architecture:** Per-service isolated caches (not shared). Each of the 5 domain services gets its own InMemoryCacheManager instance via tsyringe `useClass` registration.
- **Cache Key:** `JSON.stringify(params)` -- exact parameter match required for cache hit
- **Test Coverage:** HIGH (comprehensive cache behavior tests in `alert-service.test.ts`)

### P-003: Pagination Limits
- **Maximum:** 5,000 items per request (`limit` max in all Zod schemas)
- **Default:** 100 items per request (`limit` default in all Zod schemas)
- **Minimum:** 0 items per request (`limit` min in all Zod schemas)
- **Offset:** Integer, defaults to 0
- **include_count constraint:** Only available when offset is 0 (documented in Zod description, not enforced by validation)
- **Source:** All 5 schema files

### P-004: Request Body Size Limit
- **Value:** 10MB for transport routes
- **Source:** `transport-manager.ts` -- `express.json({ limit: "10mb" })` per-route middleware for POST routes
- **Note:** Global `express.json()` in CoreMcpServer uses default 100KB limit, but transport-specific middleware takes precedence for MCP routes
- **Scope:** POST /mcp, POST /sse/message, POST /mcp-stream

### P-005: Test Timeout
- **Value:** 25,000ms (25 seconds)
- **Source:** `vite.config.ts:10` -- `testTimeout: 25000`
- **Scope:** All Vitest tests (unit and e2e)
- **Integration test override:** 10,000ms explicit timeout on some e2e tests (`server-startup.integration.test.ts:45`)

---

## 2. Reliability NFRs (Refined)

### R-001: API Retry Strategy
- **Retries:** 3 attempts maximum
- **Backoff:** Linear -- `retryCount * 1000` (1s, 2s, 3s)
- **Retry conditions:** HTTP 429 (rate limited) OR HTTP 500+ (server errors)
- **Non-retried:** All 4xx except 429 (client errors)
- **Source:** `xdome-api-client.ts:64-79`
- **Logging:** Each retry attempt logs a warning with attempt number
- **Test Coverage:** MEDIUM (retry configuration is in code, no specific retry test)

### R-002: Graceful Shutdown
- **Signals:** SIGINT and SIGTERM both trigger shutdown
- **Sequence:** 
  1. `server.stop()` called
  2. HTTP server closed (waits for callback)
  3. TransportManager.closeAll() closes all transports
  4. McpServer.close() called
  5. `isRunning` set to false
  6. `process.exit(0)` called
- **Source:** `main.ts:29-38`, `mcp-server-instance.ts:174-216`
- **Test Coverage:** HIGH (from main.test.ts and mcp-server-instance.test.ts)

### R-003: Uncaught Exception Handler
- **Location:** `main.ts:7-11` -- global `process.on('uncaughtException')`
- **Behavior:** Wraps error with stack + origin into new Error, calls `handleFatalError()`
- **Fatal error handling:** Logs at fatal level, waits 500ms for log flush, then `process.exit(1)`
- **Source:** `logger.ts:83-89`
- **Test Coverage:** HIGH (from main.test.ts)

### R-004: Connection Premature Close
- **SSE:** `res.on('close')` handler on SSE response
- **Streamable HTTP:** `setSseResponse()` registers 'close' event, triggers `onDisconnect` callback
- **HTTP:** Short-lived -- connection removed after response sent
- **Source:** `sse-connection.ts`, `streamable-http-connection.ts`
- **Test Coverage:** HIGH (from streamable-http-connection.test.ts)

### R-005: Double-Start Prevention
- **Behavior:** If `start()` called on already-running server, logs warning and returns immediately
- **Source:** `mcp-server-instance.ts:113-116`
- **Test Coverage:** HIGH (from mcp-server-instance.test.ts)

### R-006: Transport Start Failure Isolation
- **Behavior:** If one transport's `start()` rejects, other transports are unaffected; no exception propagates
- **Source:** `transport-manager.ts` -- TransportManager.startAll()
- **Test Coverage:** HIGH (from transport-manager.test.ts)

### R-007: Transport Close Failure Isolation
- **Behavior:** If one transport's `close()` rejects, no exception propagates; other transports still closed
- **Source:** `transport-manager.ts` -- TransportManager.closeAll()
- **Test Coverage:** HIGH (from transport-manager.test.ts)

---

## 3. Security NFRs (Refined and Expanded)

### S-001: API Authentication
- **Method:** Static Bearer token in Authorization header
- **Source:** Environment variable `CLAROTY_XDOME_API_TOKEN`
- **Validation:** Both `CLAROTY_XDOME_BASE_URL` and `CLAROTY_XDOME_API_TOKEN` must be non-empty at startup
- **Token protection:** Token length logged but not value (`apiToken: Present (N chars)`)
- **Source:** `xdome-api-client.ts:34-49`
- **Test Coverage:** HIGH (constructor validation tests)

### S-002: CORS Policy
- **Configuration:** `origin: "*"` (all origins), `exposedHeaders: ["Mcp-Session-Id"]`
- **Applied:** Unconditionally (not environment-gated)
- **Risk:** Allows any domain to make requests and read the session ID header
- **Source:** `factory.ts:145-150`

### S-003: Docker Container Security
- **Base image:** `node:18-alpine` with **pinned digest** (SHA256)
- **Multi-stage build:** deps -> builder -> runner (minimal production image)
- **Non-root execution:** `addgroup nodejs (GID 1001)`, `adduser nodejs (UID 1001)`, `USER nodejs`
- **Writable directories:** `/app/logs` created and chown'd to nodejs user
- **Production mode:** `NODE_ENV=production`
- **Source:** `Dockerfile`

### S-004: Supply Chain Security (NEW -- not in broad sweep)
- **GitHub Action SHA pinning:** Every `uses:` reference pins to full commit SHA, not version tag
- **Runner hardening:** `step-security/harden-runner` with `egress-policy: audit` on all workflows
- **Dependency auditing:** `audit-ci` with explicit allowlist of 15 known CVEs (`.audit-ci.jsonc`)
- **Container scanning:** Trivy with 7 CVE ignore entries (`.trivyignore`)
- **Dockerfile linting:** Hadolint with SARIF output and PR comments
- **CODEOWNERS:** All files require review from @drbothen, @Zious11, or @arcaven

### S-005: Input Validation
- **Framework:** Zod schemas with strict validation
- **Strictness:** Some schemas use `.strict()` (alerts, alerted-devices, vulnerabilities), some do not (devices, vulnerability-devices) -- inconsistency documented in Pass 2
- **Fields:** Enum-constrained in most schemas (except alerts fields which use `z.string()`)
- **Pagination:** Type-safe with int/min/max constraints
- **Filter values:** `z.any()` -- NO type validation on filter values (potential injection risk)

### S-006: Secret Management
- **Storage:** `.env` file (gitignored)
- **Loading:** `dotenv.config({ quiet: true })` at both factory and test setup
- **Exposure:** Token value never appears in logs (only length)
- **Example:** `.env.example` provided with placeholder values

---

## 4. Observability NFRs (Refined and Expanded)

### O-001: Structured Logging
- **Framework:** Winston 3.17.0
- **Production format:** JSON (timestamp + structured fields)
- **Development format:** Colorized `TIMESTAMP [MODULE] [LEVEL]: MESSAGE {metadata}`
- **Source:** `logger.ts:33-50`

### O-002: Log Level Hierarchy
- **Custom levels:** fatal(0), error(1), warn(2), info(3), http(4), verbose(5), debug(6), silly(7)
- **Default level:** `info` (from `LOG_LEVEL` env var or default)
- **Test level:** `fatal` (set by `LOG_LEVEL=fatal` in test scripts to suppress output)
- **Colors:** fatal=red, error=red, warn=yellow, info=green, http=magenta, verbose=cyan, debug=blue, silly=grey

### O-003: Module-Scoped Logging
- **Function:** `createLogger(moduleName: string): AppLogger`
- **Implementation:** `baseLogger.child({ module: moduleName })`
- **Modules using scoped loggers:** CoreMcpServer, InMemoryCacheManager, XDomeApiClient, SessionOrchestrator, TransportManager, and all domain services
- **Modules using global logger:** Factory, main entry point

### O-004: Health Endpoint
- **Path:** GET /health
- **Response fields:**
  - `status`: Always "ok"
  - `timestamp`: ISO8601 datetime
  - `version`: Semantic version from generated version.ts
  - `uptime.seconds`: Integer floor of process uptime
  - `uptime.human`: "Xh Xm Xs" format
  - `memory.used`: Heap used in MB (rounded)
  - `memory.total`: Heap total in MB (rounded)
  - `memory.external`: External memory in MB (rounded)
  - `memory.unit`: Always "MB"
  - `environment.nodeVersion`: process.version
  - `environment.platform`: process.platform
  - `environment.arch`: process.arch
  - `dependencies.claroty_api`: "configured" or "not_configured" based on env var presence
- **Test Coverage:** HIGH (7 behavioral contracts from health-endpoint.test.ts)

### O-005: API Request Logging
- **Debug level:** Request parameters logged before each API call
- **Verbose level:** Response count logged after successful API call
- **Error level:** Failed requests logged with status, response data, request path
- **Retry warnings:** Each retry attempt logged with attempt number
- **Source:** Throughout `xdome-api-client.ts`

### O-006: Cache Operation Logging
- **Debug level:** Cache set (key + TTL + cache size), cache hit (key + age + TTL), cache miss (key + reason)
- **Verbose level:** Cache cleanup (removed count + remaining + initial size)
- **Info level:** Cache clear (previous size)
- **Source:** `cache.ts`

---

## 5. Quality NFRs (NEW -- from CI/CD pipeline)

### Q-001: Code Coverage Thresholds
- **Minimum:** 70% for branches, functions, lines, and statements
- **Provider:** @vitest/coverage-v8
- **Reporters:** json, lcov, text, clover
- **Exclusions:** `src/**/*.d.ts`, `src/types/**/*.{js,ts}`
- **Source:** `vite.config.ts:14-22`

### Q-002: Multi-Version Node.js Compatibility
- **Tested versions:** Node.js 18, 20, 22 (matrix strategy in CI)
- **Minimum required:** Node.js >=18.0.0 (from package.json engines)
- **Source:** `ci.yml`, `package.json`

### Q-003: Linting Standards
- **ESLint:** Flat config (eslint.config.js) with typescript-eslint recommended
- **Prettier:** Integration via eslint-plugin-prettier
- **Key rules:**
  - `@typescript-eslint/no-unused-vars`: warn (with _ prefix ignore pattern)
  - `@typescript-eslint/no-explicit-any`: warn in src/, off in tests
  - Separate parser configs for src/ (tsconfig.eslint.json) and tests/ (tsconfig.test.json)
- **Source:** `eslint.config.js`

### Q-004: Type Safety
- **TypeScript strict mode:** Enabled in tsconfig.json
- **Type checking:** Separate `tsc --noEmit` in code-quality workflow
- **Decorators:** `experimentalDecorators` + `emitDecoratorMetadata` for tsyringe DI
- **Source map:** Enabled

### Q-005: Dependency Hygiene
- **Dependency checking:** depcheck in code-quality workflow
- **Ignored:** @types/*, @vitest/*, tsx, ts-node, typescript, @modelcontextprotocol/inspector, @typescript-eslint/*, husky
- **Source:** `code-quality.yml`

### Q-006: Dockerfile Quality
- **Linting:** Hadolint with SARIF output
- **Triggered on:** Dockerfile changes only (path filter)
- **Reporting:** PR comments with line-by-line issues
- **Source:** `hadolint.yml`

---

## 6. Development Experience NFRs (NEW)

### D-001: Hot Reload
- **Command:** `npm run dev` -> `tsx watch src/main.ts`
- **Alternative:** nodemon.json configured for `.ts` files in `src/`, executing via ts-node ESM loader
- **Inspector:** `npm run dev:inspector` runs server + MCP protocol inspector concurrently

### D-002: Test UI
- **Command:** `npm run test:ui` -> `vitest --ui` (visual test interface via @vitest/ui)
- **Watch mode:** `npm run test:watch` -> `vitest` (interactive)

### D-003: Process Cleanup
- **Command:** `npm run dev:kill` kills processes on ports 3000, 6274, 6277
- **Ports:** 3000 (server), 6274 and 6277 (MCP inspector)

### D-004: Smoke Verification
- **Command:** `npm run verify` -> build + run verify-server.js
- **Purpose:** Creates minimal MCP server with mock tool and prints curl command for manual testing

---

## 7. Updated Missing NFRs (from broad sweep, re-evaluated)

| Missing NFR | Severity | Status |
|------------|----------|--------|
| Rate limiting on MCP endpoints | Medium | Still missing -- only upstream rate limit retry |
| Session expiration/cleanup | High | Still missing -- sessions live forever in memory |
| Metrics/telemetry (Prometheus/OTEL) | Medium | Still missing -- only Winston logging |
| Distributed tracing | Low | Still missing -- no trace IDs/spans |
| Cache size limits | High | Still missing -- unbounded growth per cache |
| MCP endpoint authentication | High | Still missing -- /mcp, /sse, /mcp-stream accept unauthenticated requests |
| Connection limits | Medium | Still missing -- no max connections per transport |
| Request validation on filter values | Medium | Still missing -- `z.any()` for filter value field |
| Graceful degradation on xDome unavailability | Medium | Partially addressed: retry on 5xx/429, but no circuit breaker |
| Cache invalidation on write | N/A | Not needed yet (read-only), but will be needed for planned write tools |

---

## Delta Summary
- New items added: 6 quality NFRs (Q-001 through Q-006); 4 development experience NFRs (D-001 through D-004); supply chain security NFR (S-004); API request logging NFR (O-005); cache operation logging NFR (O-006); double-start prevention (R-005); transport failure isolation (R-006, R-007); 10 missing NFR re-evaluation
- Existing items refined: All performance NFRs now have exact source locations and test coverage status; reliability NFRs now include shutdown sequence detail; security NFRs expanded with Docker digest pinning, supply chain detail; observability NFRs now have complete health endpoint field catalog
- Remaining gaps: Performance benchmarks (no load testing infrastructure found); memory profiling under sustained query diversity

## Novelty Assessment
Novelty: SUBSTANTIVE
The CI/CD quality NFRs (coverage thresholds, multi-version testing, dependency checking), supply chain security measures (SHA pinning, runner hardening, audit-ci), development experience NFRs, and the precise per-service cache isolation architecture all change how you would spec the system's non-functional requirements. The broad sweep presented a minimal NFR table; the actual system has a mature quality and security pipeline.

## Convergence Declaration
Another round needed -- the following substantive gaps remain: (1) verify whether any performance/load testing exists in the CI pipeline or scripts, (2) analyze the `.windsurf/rules/14-security-performance.md` for additional NFR intent documentation.

## State Checkpoint
```yaml
pass: 4
round: 1
status: complete
timestamp: 2026-04-14T00:00:00Z
novelty: SUBSTANTIVE
```
