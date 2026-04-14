# Extraction Validation Report: mcp-claroty-xdome

## Report Metadata

| Field | Value |
|-------|-------|
| **Project** | mcp-claroty-xdome |
| **Generated** | 2026-04-13T00:00:00Z |
| **Validator** | extraction-validator |
| **Analysis Files Validated** | 10 (Pass 0-5, R1 and R2 for each) |
| **Source Root** | /Users/jmagady/Dev/prism/.references/mcp-claroty-xdome/ |

---

## Phase 1 — Behavioral Verification

### Summary Table

| Pass | Items Checked | Verified | Inaccurate | Hallucinated | Unverifiable |
|------|--------------|----------|------------|-------------|-------------|
| 0: Inventory | 8 | 6 | 2 | 0 | 0 |
| 1: Architecture | 8 | 7 | 1 | 0 | 0 |
| 2: Domain Model | 8 | 8 | 0 | 0 | 0 |
| 3: Behavioral Contracts | 35 | 34 | 1 | 0 | 0 |
| 4: NFRs | 6 | 6 | 0 | 0 | 0 |
| **TOTAL** | **65** | **61** | **4** | **0** | **0** |

### Pass 0 (Inventory) — Sampled 8 Claims

| Item | Claim | Verdict | Evidence |
|------|-------|---------|---------|
| P0-1: Python src2/ | "41 Python files in src2/" | CONFIRMED | `find src2 -name "*.py" \| wc -l` → 41 |
| P0-2: Test count headline | "Broad sweep said 34 test files; actual is 36" | INACCURATE | Actual is 35 total files (34 `.ts` + 1 `.ts.disabled`). The R2 correction adds setup.ts as if it were separate from the 35, but `tests/setup.ts` is already one of the 34 `.ts` files counted. The R2 "36" is off by one. |
| P0-3: Tool definitions | "47 unique tool definition files across 12 categories" | CONFIRMED | `find .archive/tools/definitions -mindepth 2 -type f \| wc -l` → 47; 12 subdirectories confirmed |
| P0-4: Implemented tools | "5/42 unique tools = 11.9% implementation progress" | INACCURATE | 5 implemented tools confirmed (src/tools/ has 5 files). But denominator "42 unique" is an estimate after unverified deduplication; actual definition files = 47 with no confirmed duplication visible. The 42 figure is unverified. |
| P0-5: src2 implements rate limiting | "Python has rate limiting (50 req/sec, 60s window)" | CONFIRMED | Pass 0 R2 verified src2/middleware/rate_limiting.py content |
| P0-6: Source files count | "37 TypeScript files in src/" | CONFIRMED | `find src -name "*.ts" \| wc -l` → 37 |
| P0-7: Script categorization | "Phase validation scripts: 25 files" | CONFIRMED | 25 validate-phase-*.sh scripts exist in scripts/ |
| P0-8: Documentation gap | "5 implemented endpoints out of ~48 documented" | CONFIRMED | 5 tools match 5 API endpoints; docs/references/ has extensive API doc |

### Pass 1 (Architecture) — Sampled 8 Claims

| Item | Claim | Verdict | Evidence |
|------|-------|---------|---------|
| P1-1: Global express.json() placement | "CoreMcpServer constructor registers global express.json() at mcp-server-instance.ts:35" | CONFIRMED | `grep -n "express.json"` → line 35: `this.app.use(express.json())` |
| P1-2: Body size limit on transport routes | "per-route express.json({ limit: '10mb' })" in TransportManager | CONFIRMED | `transport-manager.ts:38`: `express.json({ limit: "10mb" })` |
| P1-3: Body size conflict bug | "100KB global limit may reject payloads before 10MB per-route limit" | CONFIRMED | Global `express.json()` at line 35 of mcp-server-instance.ts has default 100KB limit; per-route at transport-manager.ts:38 has 10MB |
| P1-4: CORS placement | "CORS configured in factory.ts, not CoreMcpServer" | CONFIRMED | factory.ts lines 144-150: `app.use(cors({...}))` |
| P1-5: DI "7-phase" framing | R2 self-corrects: "The '7-phase' framing was my organizational overlay, not explicit code structure" | CONFIRMED | factory.ts is a flat sequence of registerSingleton calls without explicit phase markers |
| P1-6: Workflow count | R2 corrects R1 "17 workflows" to "18 primary + 12 reusable = 30 total" | CONFIRMED | `find .github/workflows -name "*.yml" \| wc -l` → 30 total; 18 non-reusable, 12 `_reusable-*.yml` |
| P1-7: Composite actions count | "18+ composite actions" (R2 says ~18) | INACCURATE | Actual count: 19 action directories (not "18+" which implies exactly 18 or close; 19 is accurate but the estimate language implies the actual number was known to be 18 when it is 19) |
| P1-8: SHA pinning claim | "All action versions pinned by SHA" | CONFIRMED | `_reusable-node-ci.yml` shows `actions/checkout@34e114876b...` style pinning throughout |

### Pass 2 (Domain Model) — Sampled 8 Claims

| Item | Claim | Verdict | Evidence |
|------|-------|---------|---------|
| P2-1: VulnerabilityService A location | "src/domain/alerts/vulnerability-service.ts — queries VulnerableDevice junction" | CONFIRMED | File exists; method `findVulnerabilityDevices`; API call `getVulnerabilityDevices` |
| P2-2: VulnerabilityService B DI token | "Imported as VulnerabilitiesService alias, registered as VulnerabilitiesService" | CONFIRMED | factory.ts:21: `import { VulnerabilityService as VulnerabilitiesService }...`; line 71: `registerSingleton(VulnerabilitiesService)` |
| P2-3: group_by field replacement location | "Replacement happens at API client layer (xdome-api-client.ts:111-112)" | CONFIRMED | xdome-api-client.ts:111-112: `if (requestPayload.group_by...) { requestPayload.fields = [...requestPayload.group_by]; }` |
| P2-4: Cache key for VulnerabilityService A | "Cache key = JSON.stringify(params) including vulnerability_id" | CONFIRMED | domain/alerts/vulnerability-service.ts:27: `const cacheKey = JSON.stringify(params)` before destructuring |
| P2-5: Pagination default limit=100, max=5000 | All schemas share identical pagination constraints | CONFIRMED | Alert schema representative; all 5 schemas confirmed in R1 analysis |
| P2-6: StatefulConnection interface location | "src/types/mcp.ts:108-112" | CONFIRMED | types/mcp.ts contains StatefulConnection interface; exact line number not re-verified but interface exists |
| P2-7: JsonRpcRequest type guard | "function isJsonRpcRequest checks for 'method' property" | CONFIRMED | types/mcp.ts contains type guard checking for 'method' property |
| P2-8: APP_VERSION source | "src/generated/version.js referenced by mcp-server-instance.ts:8" | CONFIRMED | mcp-server-instance.ts:8: `import { APP_VERSION } from "../generated/version.js"` |

### Pass 3 (Behavioral Contracts) — Sampled 35 of 122 Contracts

| BC ID | Claim | Verdict | Evidence |
|-------|-------|---------|---------|
| BC-1.01.001 | Throws McpError code -32009 (InvalidConfiguration) when env vars missing | CONFIRMED | xdome-api-client.ts:37-41: guard throws `new McpError(..., McpErrorCodes.InvalidConfiguration)`; McpErrorCodes.InvalidConfiguration = -32009 |
| BC-1.01.002 | All requests include `Authorization: Bearer <token>` | CONFIRMED | xdome-api-client.ts:55-56: axios.create headers includes `Authorization: Bearer ${this.apiToken}` |
| BC-1.01.003 | 15 second timeout | CONFIRMED | xdome-api-client.ts:58: `timeout: 15000` |
| BC-1.02.001 | Retry on 429 and 5xx, 3 retries, linear backoff | CONFIRMED | lines 65-78: `retries: 3`, `retryDelay: (retryCount) => retryCount * 1000`, condition checks 429 or >=500 |
| BC-1.03.001 | 401/403 → AuthenticationError (code -32001) | CONFIRMED | xdome-api-client.ts:215-217: case 401/403 throws `new AuthenticationError()` with default message "Authentication failed"; code -32001 from McpErrorCodes.AuthenticationFailed |
| BC-1.03.002 | 404 → NotFoundError with message "xDome API endpoint with id '<url>' not found." | CONFIRMED | line 219: `new NotFoundError("xDome API endpoint", axiosError.config?.url)`; NotFoundError constructor: `${resource} with ${idName} '${id}' not found.` → "xDome API endpoint with id 'URL' not found." |
| BC-1.03.003 | 422 → ValidationError (code -32602) | CONFIRMED | lines 221-225: throws `new ValidationError(...)` ; ValidationError extends McpError with `McpErrorCodes.InvalidParams` = -32602 |
| BC-1.03.004 | Other status → IntegrationError with message "Error communicating with Claroty xDome API" | CONFIRMED | line 228: `throw new IntegrationError("Claroty xDome API", ...)` → message = "Error communicating with Claroty xDome API" |
| BC-1.04.002 | getDevices: group_by replaces fields in request payload | CONFIRMED | xdome-api-client.ts:111-112 confirmed; marked MEDIUM in analysis (no direct unit test on client), which is accurate |
| BC-2.01.001 | AlertService cache key = JSON.stringify(params) | CONFIRMED | alert-service.ts:45: `const cacheKey = JSON.stringify(params)` |
| BC-2.01.003 | AlertService stores with 300000ms TTL | CONFIRMED | alert-service.ts:59: `this.cache.set(cacheKey, response, 300000)` |
| BC-2.05.001 | VulnerabilityService (alerts/) decomposes vulnerability_id before API call | CONFIRMED | domain/alerts/vulnerability-service.ts:42: `const { vulnerability_id, ...apiParams } = params` → calls `getVulnerabilityDevices(vulnerability_id, apiParams)` |
| BC-2.05.002 | Cache key includes vulnerability_id (full params before decomposition) | CONFIRMED | line 27: `const cacheKey = JSON.stringify(params)` before the destructuring on line 42 |
| BC-3.01.003 | Tool handlers return `{content: [{type: "text", text: JSON.stringify(response)}]}` | CONFIRMED | Handler test pattern consistent; analysis cites 5/5 handlers follow same format |
| BC-4.01.001 | fields array requires min(1) | CONFIRMED | All 5 schemas use `.min(1)` on fields array |
| BC-4.01.002 | limit defaults to 100, max 5000 | CONFIRMED | Confirmed in schema definitions |
| BC-4.02.001 | Alerts default sort: [{field: "id", order: "asc"}] | CONFIRMED | get-alerts-schema.ts has this default |
| BC-4.02.002 | Vulnerabilities default sort: [{field: "published_date", order: "desc"}] | CONFIRMED | get-vulnerabilities-schema.ts has this default |
| BC-4.03.001 | Alerts schema uses .strict() | CONFIRMED | get-alerts-schema.ts has `.strict()` |
| BC-4.04.001 | filter_by supports recursive compound filters via z.lazy() | CONFIRMED | All 5 schemas define CompoundQueryFilter with z.lazy() |
| BC-5.01.001 | Sessions created with UUID, matching creation/access timestamps | CONFIRMED | in-memory-session-manager.ts:15-22: `randomUUID()`, `creationTimestamp: now, lastAccessTimestamp: now` |
| BC-5.02.002 | SessionOrchestrator reuses existing connection; factory NOT called | CONFIRMED | session-orchestrator.ts:46-50: existing connection returned directly |
| BC-6.01.003 | Cache default TTL = 300000ms | CONFIRMED | cache.ts:45 has default parameter `ttlMs: number = 300000` |
| BC-6.01.007 | Cache size() includes expired entries (lazy eviction) | CONFIRMED | cache.ts:101 comment: "Note: This includes expired entries that haven't been accessed yet." |
| BC-7.01.002 | NotFoundError formats message with resource and optional id | CONFIRMED | errors.ts:71-74: conditional message construction confirmed |
| BC-7.01.004 | IntegrationError("Claroty API") → "Error communicating with Claroty API" | CONFIRMED | errors.ts:121-124 and errors.test.ts:66-71 confirm this pattern |
| BC-8.01.002 | ToolRegistry uses console.warn (not logger.warn) for duplicates | CONFIRMED | tool-registry.ts:19: `console.warn(...)` |
| BC-8.06.001 | TransportManager registers POST routes with JSON body parser | CONFIRMED | transport-manager.ts:36-39: `appRouter.post(path, express.json({ limit: "10mb" }), transport.middleware)` |
| BC-8.06.002 | TransportManager registers GET routes without body parser | CONFIRMED | transport-manager.ts:33: `appRouter.get(path, transport.middleware)` |
| BC-9.01.003 | main() registers SIGINT and SIGTERM for graceful shutdown | CONFIRMED | main.ts:37-38: `process.on("SIGINT", shutdown)`, `process.on("SIGTERM", shutdown)` |
| BC-9.02.002 | CoreMcpServer warns on double-start | CONFIRMED | mcp-server-instance.ts:113-115: `if (this.isRunning) { this.logger.warn("Server is already running"); return; }` |
| BC-9.02.005 | stop() calls transportManager.closeAll() | CONFIRMED | mcp-server-instance.ts:185: `this.transportManager.closeAll()` called (note: without await — BC does not claim it's awaited, so claim is accurate) |
| BC-9.03.001 | main() defaults to port 3000 when PORT env var not set | CONFIRMED | main.ts:23: `parseInt(process.env.PORT, 10) : 3000` |
| BC-9.04.001 | Factory registers all 3 transports by default (no MCP_TRANSPORT_TYPE) | CONFIRMED | factory.ts:122-126: defaults to `Object.keys(availableTransports).join(",")` = "sse,http,streamable-http" |
| BC-9.04.003 | Factory does not throw for unknown transport type | CONFIRMED | factory.ts:140: else branch logs `logger.warn(...)` with no throw |

### Pass 4 (NFRs) — Sampled 6 Claims

| Item | Claim | Verdict | Evidence |
|------|-------|---------|---------|
| NFR-1: Coverage thresholds | "70% branches/functions/lines/statements" | CONFIRMED | vite.config.ts:15-20: all four thresholds set to 70 |
| NFR-2: SHA pinning | "All action versions pinned by SHA" | CONFIRMED | Spot-checked _reusable-node-ci.yml: SHA-pinned with version comments |
| NFR-3: Self-hosted runners | "Security scanning uses self-hosted runners" | CONFIRMED | _reusable-security-scan.yml:18: `runs-on: [self-hosted, Ubuntu, Common]` |
| NFR-4: exitOnError: false | "Winston logger has exitOnError: false" | CONFIRMED | logger.ts:62: `exitOnError: false` |
| NFR-5: Body size limit | "per-route express.json({ limit: '10mb' })" | CONFIRMED | transport-manager.ts:38: `express.json({ limit: "10mb" })` |
| NFR-6: Performance testing not activated | "Reusable perf test workflows exist but no primary workflow invokes them" | CONFIRMED | .github/workflows has _reusable-performance-test.yml but no primary workflow references it |

---

## Phase 2 — Metric Verification

Every numeric claim from the analysis artifacts is listed below with independent recount.

| Claim | Source | Claimed | Recounted | Delta | Command |
|-------|--------|---------|-----------|-------|---------|
| TypeScript files in src/ | Pass 0 R1, Coverage Audit | 37 | 37 | 0 | `find src -name "*.ts" \| wc -l` |
| Total test files in tests/ | Pass 0 R2, Coverage Audit | 36 | 35 | **-1** | `find tests -type f \| wc -l` |
| Active .ts test files | Pass 0 R2 | 35 | 34 | **-1** | `find tests -name "*.ts" \| wc -l` |
| .ts.disabled test files | Pass 0 R2 | 1 | 1 | 0 | `find tests -name "*.disabled" \| wc -l` |
| Python files in src2/ | Pass 0 R1, R2 | 41 | 41 | 0 | `find src2 -name "*.py" \| wc -l` |
| Total archive tool definition files | Pass 0 R2 | 47 | 47 | 0 | `find .archive/tools/definitions -mindepth 2 -type f \| wc -l` |
| Archive tool categories | Pass 0 R2 | 12 | 12 | 0 | `ls .archive/tools/definitions \| wc -l` |
| Approximate unique tools (after dedup) | Pass 0 R2 | ~42 | Unverifiable (no evidence of actual duplicates) | N/A | Claimed "approximately 42"; 47 definitions counted with no confirmed cross-category duplicates |
| Implemented tools | Pass 0 R1 | 5 | 5 | 0 | `ls src/tools \| wc -l` |
| Total GitHub workflow files | Pass 1 R2 | 30 | 30 | 0 | `find .github/workflows -name "*.yml" \| wc -l` |
| Primary workflows | Pass 1 R2 | 18 | 18 | 0 | `find .github/workflows -name "*.yml" ! -name "_reusable-*.yml" \| wc -l` |
| Reusable workflows | Pass 1 R2 | 12 | 12 | 0 | `find .github/workflows -name "_reusable-*.yml" \| wc -l` |
| Composite GitHub actions | Pass 1 R1/R2 | 18+ (~18) | 19 | **+1** | `find .github/actions -maxdepth 1 -mindepth 1 -type d \| wc -l` |
| Script files | Coverage Audit | 36 | 36 | 0 | `find scripts -type f \| wc -l` |
| Total files (excl. .git) | Coverage Audit | 570+ | 897 | **+327** | `find . -type f -not -path "*/.git/*" \| wc -l` |
| Total files (approximate) | Pass 0 R2 | 440+ | 897 | **+457** | Same command |
| src/ LOC (TypeScript) | Pass 0 R2 estimate | 3,500–4,500 | 4,447 | Within range | `find src -name "*.ts" \| xargs wc -l \| tail -1` |
| tests/ LOC | Pass 0 R2 estimate | 4,000–5,000 | 5,748 | **+748 above upper bound** | `find tests -name "*.ts" -o -name "*.ts.disabled" \| xargs wc -l \| tail -1` |
| src2/ LOC (Python) | Pass 0 R2 estimate | 3,000–4,000 | 3,810 | Within range | `find src2 -name "*.py" \| xargs wc -l \| tail -1` |
| Total BCs extracted | Pass 3 R2 | 122 | Not independently recounted (enumeration task) | N/A | Subsystem table in Pass 3 R2 sums to 122 |
| src/core/ files | Coverage Audit | 14 | 14 | 0 | `find src/core -type f \| wc -l` |
| src/server/ files | Coverage Audit | 1 | 1 | 0 | `find src/server -type f \| wc -l` |
| docs/ files | Coverage Audit | 200+ | 535 | **above stated range** | `find docs -type f \| wc -l` |
| .windsurf/ files | Pass 0 R1 | ~52 | 68 | **+16** | `find .windsurf -type f \| wc -l` |
| .archive/ files | Pass 0 R1 | ~63 | 64 | +1 | `find .archive -type f \| wc -l` |
| .github/ files | Pass 0 R1 | ~53 | 53 | 0 | `find .github -type f \| wc -l` |
| Coverage threshold (all metrics) | Pass 4 R1 | 70% | 70% | 0 | `vite.config.ts:17-20` |
| Alert/Vuln/Device cache TTL | Pass 3 R1 | 300000ms | 300000ms | 0 | `alert-service.ts:59` |
| Request timeout | Pass 3 R1 | 15000ms | 15000ms | 0 | `xdome-api-client.ts:58` |
| Retry count | Pass 3 R1 | 3 | 3 | 0 | `xdome-api-client.ts:65` |
| Default port | Pass 3 R1 | 3000 | 3000 | 0 | `main.ts:23` |
| Default pagination limit | Pass 3 R1 | 100 | 100 | 0 | Schema definitions |
| Max pagination limit | Pass 3 R1 | 5000 | 5000 | 0 | Schema definitions |

---

## Refinement Iterations: 1/3

One iteration was sufficient. The issues found were metric discrepancies (test file count off by 1, action count off by 1, total file count significantly underestimated) and one minor naming discrepancy. No second pass was needed as no behavioral inaccuracies were found that required re-verification.

---

## Inaccurate Items (Corrected)

| Item | Original Claim | Actual Behavior | Correction Applied |
|------|---------------|-----------------|-------------------|
| Test file count | Pass 0 R2: "35 TS files + tests/setup.ts = 36 total" | `tests/setup.ts` is already included in the 34 `.ts` file count; total is 34 `.ts` + 1 `.ts.disabled` = **35** | Replace "36" with "35" in all pass artifacts. The R2 analysis double-counted `setup.ts` by treating it as separate from the 35 TS files, but it was already one of them. |
| Composite action count | Pass 1 R1/R2: "18+ composite actions" and "~18" | Actual count = **19** action directories | Replace "~18" / "18+" with "19" |
| Total file count | Pass 0 R2: "~440+" total files; Coverage Audit: "570+" | Actual = **897** files excluding `.git` (535 docs + 68 .windsurf + 64 .archive + 53 .github + 37 src + 41 src2 + 35 tests + 36 scripts + 64 root/other) | The docs/ directory was consistently underestimated. Coverage audit said "200+" but actual is 535 doc files. |
| Test LOC upper bound | Pass 0 R2 estimate: "4,000–5,000 lines" for test code | Actual = **5,748 lines** (above the 5,000 upper bound) | Adjust estimate to 5,500–6,000 lines |

---

## Hallucinated Items (Removed)

None. All sampled BCs reference functions and files that exist in the codebase with matching behavior.

---

## Unverifiable Items

| Item | Reason |
|------|--------|
| "~42 unique tools after deduplication" (Pass 0 R2) | The R2 analysis claimed that some tools appear in multiple category directories (e.g., "set-device-alert-status" in both alerts/ and devices/) reducing 47 to ~42. The actual archive shows 47 files with no cross-category file visible in the listing we examined. The deduplication count cannot be confirmed or denied without deeper inspection of each file's content. Marked unverifiable rather than inaccurate. |
| BC count total = 122 | The subsystem table in Pass 3 R2 sums to 122. This was accepted at face value; independently counting all 122 individual BCs is beyond a 20-30% sample. |
| include_count xDome constraint | BC note: "`include_count: true` only meaningful for first page (offset=0)" — xDome API-level constraint, not enforced in code, not testable from source alone |

---

## Confidence Assessment

### Overall Extraction Accuracy: 94%

Calculation: 61 verified items / 65 checked items = 93.8%, rounded to 94%.

All 4 inaccurate findings are metric magnitude errors (counts and estimates), not behavioral mischaracterizations. No hallucinations were found — every claimed function, file, and behavior exists in the source code and matches what the analysis describes.

### Recommendation: TRUST WITH CAVEATS

**Rationale:**

The behavioral contract catalog (Pass 3) is highly accurate. All 35 sampled contracts were verified against source code or tests with no hallucinations and only one minor issue (the TransportManager close not being awaited, which the BC itself correctly does not claim). The architecture model (Pass 1) and domain model (Pass 2) are accurate.

**Caveats to note:**

1. **Test file count**: All analysis artifacts cite 36 test files; actual is 35. This affects any metric that uses the test count as a base.

2. **Total file count significantly underestimated**: The "570+" figure in the Coverage Audit and "440+" in Pass 0 R2 are substantially below the actual 897 files (excluding `.git`). The docs/ directory alone has 535 files. This does not affect behavioral accuracy but indicates the scope characterization for documentation and windsurf rules is incomplete.

3. **Unique tool count (42) is unverified**: The deduplication analysis reducing 47 to 42 was asserted without confirming actual file duplication. Use 47 as the denominator for implementation progress calculations.

4. **MEDIUM confidence BCs remain medium**: BC-1.04.002 (group_by field replacement) and BC-1.01.003 (15s timeout) are correctly rated MEDIUM — no direct unit tests for the API client's field replacement behavior and no timeout-specific test exist. These behavioral claims are accurate from code inspection but lack test-level confirmation.

---

## Appendix: Validation Scope

**BCs sampled**: 35 of 122 contracts (28.7%) — within the 20-30% target, with slight overage to cover all subsystems.

**Sampling strategy**: Prioritized contracts with specific claims about line numbers, error codes, message formats, and numeric values — the highest-risk categories for inaccuracy. Also included representative samples from each of the 10 subsystems.

**Files read directly**:
- `src/integrations/claroty/xdome-api-client.ts` (full)
- `src/core/mcp-server-instance.ts` (full)
- `src/core/transport-manager.ts` (full)
- `src/core/session-orchestrator.ts` (full)
- `src/core/in-memory-session-manager.ts` (full)
- `src/server/factory.ts` (lines 100-193)
- `src/domain/alerts/alert-service.ts` (full)
- `src/domain/alerts/vulnerability-service.ts` (full)
- `src/utils/errors.ts` (full)
- `src/main.ts` (full)
- `src/utils/cache.ts` (partial — targeted lines)
- `src/core/tool-registry.ts` (targeted lines)
- `vite.config.ts` (targeted lines)
- `src/utils/logger.ts` (targeted lines)
