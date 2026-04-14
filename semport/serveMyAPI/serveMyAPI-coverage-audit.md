# Coverage Audit -- serveMyAPI

> Phase B.5 coverage audit performed 2026-04-13. Grep-driven, not judgment-driven.
> All 22 source files inventoried. All 13 prior analysis files cross-referenced.

---

## 1. Complete File Inventory (22 files)

### Source Files (7)

| # | Path | Lines | Type |
|---|------|-------|------|
| 1 | `src/index.ts` | 158 | Entry point (stdio MCP) |
| 2 | `src/server.ts` | 230 | Entry point (HTTP/SSE MCP) |
| 3 | `src/services/keychain.ts` | 198 | Core service |
| 4 | `src/cli.ts` | 117 | Direct CLI |
| 5 | `src/cli.js` | 163 | MCP client CLI (dead code) |
| 6 | `package.json` | 37 | Project config |
| 7 | `tsconfig.json` | 15 | TS compiler config |

### Configuration & Deployment (5)

| # | Path | Lines | Type |
|---|------|-------|------|
| 8 | `Dockerfile` | 35 | Container image |
| 9 | `smithery.yaml` | 81 | Smithery hosting config |
| 10 | `build_dmg.sh` | 109 | macOS DMG packaging |
| 11 | `examples/claude_desktop_config.json` | 9 | MCP client config example |
| 12 | `examples/windsurf_config.json` | 9 | MCP client config example |

### Documentation (6)

| # | Path | Type |
|---|------|------|
| 13 | `CLAUDE.md` | AI context / MCP registry |
| 14 | `CONTRIBUTING.md` | Dev guidelines |
| 15 | `README.md` | Primary documentation |
| 16 | `INGESTION.md` | Generated (copy of broad sweep) |
| 17 | `mcp-server-template.md` | Reference material |
| 18 | `MCP-TypeScript-Readme.md` | Reference material |

### Assets & Meta (4)

| # | Path | Type |
|---|------|------|
| 19 | `icon.png` | App icon (99KB) |
| 20 | `icon.svg` | App icon source |
| 21 | `How to improve serveMyAPI.pdf` | 2.0 vision document |
| 22 | `.gitignore` | Git config |

---

## 2. Coverage Matrix

Reference counts are total mentions across all 13 analysis files (broad sweep + 12 deepening files).

### Source Files

| File | P0 Inv | P1 Arch | P2 Domain | P3 Contracts | P4 NFR | P5 Conv | Coverage |
|------|--------|---------|-----------|--------------|--------|---------|----------|
| `src/index.ts` | YES (14) | YES (11) | YES (5) | YES (35) | YES (ref) | YES (15) | **FULL** |
| `src/server.ts` | YES (6) | YES (11) | YES (15) | YES (30) | YES (ref) | YES (14) | **FULL** |
| `src/services/keychain.ts` | YES (6) | YES (5) | YES (11) | YES (39) | YES (4) | YES (24) | **FULL** |
| `src/cli.ts` | YES (11) | YES (3) | YES (6) | YES (36) | YES (ref) | YES (11) | **FULL** |
| `src/cli.js` | YES (12) | YES (3) | YES (9) | YES (48) | YES (ref) | YES (12) | **FULL** |
| `package.json` | YES (6) | YES (4) | YES (2) | NO | YES (4) | YES (3) | **FULL** (no behavioral contracts expected) |
| `tsconfig.json` | YES (4) | NO | YES (1) | NO | NO | YES (8) | **FULL** (config file -- conv + inventory sufficient) |

### Configuration & Deployment Files

| File | P0 Inv | P1 Arch | P2 Domain | P3 Contracts | P4 NFR | P5 Conv | Coverage |
|------|--------|---------|-----------|--------------|--------|---------|----------|
| `Dockerfile` | YES (6) | YES (8) | YES (3) | YES (2) | YES (7) | YES (2) | **FULL** |
| `smithery.yaml` | YES (4) | YES (6) | YES (1) | YES (3) | YES (1) | YES (8) | **FULL** |
| `build_dmg.sh` | YES (3) | YES (2) | YES (13) | YES (3) | NO | NO | **FULL** (packaging -- domain + contracts + arch cover it) |
| `examples/claude_desktop_config.json` | YES (2) | NO | YES (1) | NO | NO | NO | **FULL** (trivial config example, fully characterized in P0 + P2) |
| `examples/windsurf_config.json` | YES (2) | NO | YES (1) | NO | NO | NO | **FULL** (identical to claude config, noted in P2-R2) |

### Documentation Files

| File | P0 Inv | P1 Arch | P2 Domain | P3 Contracts | P4 NFR | P5 Conv | Coverage |
|------|--------|---------|-----------|--------------|--------|---------|----------|
| `CLAUDE.md` | YES (8) | YES (2) | NO | NO | NO | NO | **FULL** (ecosystem role extracted in P0; content mined in P1) |
| `CONTRIBUTING.md` | YES (4) | NO | NO | NO | YES (3) | YES (1) | **FULL** (license inconsistency found, contribution guidelines noted) |
| `README.md` | YES (3) | NO | NO | NO | YES (1) | YES (1) | **FULL** (roadmap gap analysis in P0-R2, license inconsistency in P4) |
| `INGESTION.md` | YES (4) | NO | NO | NO | NO | NO | **FULL** (identified as copy of broad sweep; no novel content) |
| `mcp-server-template.md` | YES (3) | YES (4) | NO | NO | NO | NO | **FULL** (characterized as generic reference material in P0-R2, P1-R2) |
| `MCP-TypeScript-Readme.md` | YES (2) | NO | NO | NO | NO | NO | **FULL** (characterized as SDK readme reference in P0-R2) |

### Assets & Meta

| File | P0 Inv | P1 Arch | P2 Domain | P3 Contracts | P4 NFR | P5 Conv | Coverage |
|------|--------|---------|-----------|--------------|--------|---------|----------|
| `icon.png` | YES (3) | NO | NO | NO | NO | NO | **FULL** (binary asset, identified as orphaned in P0-R2) |
| `icon.svg` | YES (3) | NO | NO | NO | NO | NO | **FULL** (markup asset, identified as orphaned in P0-R2) |
| `How to improve serveMyAPI.pdf` | YES (3) | YES (1) | NO | NO | NO | NO | **FULL** (2.0 vision doc analyzed in P0-R1 and P1-R1) |
| `.gitignore` | YES (2) | NO | NO | NO | NO | NO | **FULL** (config meta, characterized in P0-R1) |

---

## 3. Coverage Summary

| Category | Files | Full | Partial | None |
|----------|-------|------|---------|------|
| Source | 7 | 7 | 0 | 0 |
| Config/Deploy | 5 | 5 | 0 | 0 |
| Documentation | 6 | 6 | 0 | 0 |
| Assets/Meta | 4 | 4 | 0 | 0 |
| **Total** | **22** | **22** | **0** | **0** |

**Result: 22/22 files fully covered (100%).**

---

## 4. Behavioral Contract Coverage

All functions/methods in source files verified against BC catalog:

### src/services/keychain.ts (14 methods/functions)

| Method | Contract | Status |
|--------|----------|--------|
| `constructor()` | BC-2.03.001 | COVERED |
| `ensureStorageDirectory()` | BC-2.03.003 | COVERED |
| `checkPermissionMarker()` | BC-2.03.002 | COVERED |
| `storeKey()` | BC-2.01.001 | COVERED |
| `storeKeyFile()` | BC-2.02.001 | COVERED |
| `getKey()` | BC-2.01.002 | COVERED |
| `getKeyFile()` | BC-2.02.002 | COVERED |
| `deleteKey()` | BC-2.01.003 | COVERED |
| `deleteKeyFile()` | BC-2.02.003 | COVERED |
| `listKeys()` | BC-2.01.004 | COVERED |
| `listKeyFiles()` | BC-2.02.004 | COVERED |
| Module constants (4) | VO-2.01 to VO-2.04 | COVERED |
| Singleton export | PAT-001 | COVERED |
| IS_DOCKER conditional | PAT-002 | COVERED |

### src/index.ts (5 registration blocks + 1 startup)

| Block | Contract | Status |
|-------|----------|--------|
| `store-api-key` tool | BC-3.01.001 | COVERED |
| `get-api-key` tool | BC-3.01.002 | COVERED |
| `delete-api-key` tool | BC-3.01.003 | COVERED |
| `list-api-keys` tool | BC-3.01.004 | COVERED |
| Server startup (connect + log) | BC-3.06.001 | COVERED |
| Unused keytar import | OBS-3.04 | COVERED |

### src/server.ts (5 tool blocks + 4 Express routes + 1 startup)

| Block | Contract | Status |
|-------|----------|--------|
| `store-api-key` tool | BC-3.02.001 (= BC-3.01.001) | COVERED |
| `get-api-key` tool | BC-3.02.002 (= BC-3.01.002) | COVERED |
| `delete-api-key` tool | BC-3.02.003 (= BC-3.01.003) | COVERED |
| `list-api-keys` tool | BC-3.02.004 (= BC-3.01.004) | COVERED |
| `GET /sse` | BC-3.03.001 | COVERED |
| `POST /messages` | BC-3.03.002 | COVERED |
| `GET /` | BC-3.03.003 | COVERED |
| `app.listen()` | BC-3.06.002 | COVERED |
| `export { server }` | CONV-007 | COVERED |
| `activeTransports` Map | E-2.05, VO-2.07 | COVERED |

### src/cli.ts (6 commands + printUsage + main catch)

| Block | Contract | Status |
|-------|----------|--------|
| `list` case | BC-3.04.001 | COVERED |
| `get` case | BC-3.04.002 | COVERED |
| `store`/`add` case | BC-3.04.003 | COVERED |
| `delete`/`remove` case | BC-3.04.004 | COVERED |
| `help`/`--help`/`-h` case | BC-3.04.005 | COVERED |
| `default` case | BC-3.04.006 | COVERED |
| `printUsage()` | Referenced in P3, P5 | COVERED |
| `main().catch()` | Referenced in CONV-005 | COVERED |

### src/cli.js (6 commands + printUsage + help check + main)

| Block | Contract | Status |
|-------|----------|--------|
| `main()` setup + connect | BC-3.05.001 | COVERED |
| `list` case | BC-3.05.001 | COVERED |
| `get` case | BC-3.05.001 | COVERED |
| `store` case | BC-3.05.001 | COVERED |
| `delete` case | BC-3.05.001 | COVERED |
| `default` case | BC-3.05.005 | COVERED |
| Result parsing | BC-3.05.002 | COVERED |
| `finally` disconnect | BC-3.05.003 | COVERED |
| `--help`/`-h` check | BC-3.05.004 | COVERED |
| Default command = `list` | BC-3.05.006 | COVERED |
| Spawns wrong server | OBS-3.05 | COVERED |

---

## 5. Entity/Value Object Coverage

| ID | Entity/VO | First Identified | Deepened | Status |
|----|-----------|------------------|----------|--------|
| E-2.01 | Credential (implicit) | Broad sweep | P2-R1, P2-R2 | COVERED |
| E-2.02 | PermissionMarker | Broad sweep | P2-R1 | COVERED |
| E-2.03 | McpServer instance | P2-R1 | P2-R2 | COVERED |
| E-2.04 | McpClient instance | P2-R1 | P2-R2 | COVERED |
| E-2.05 | SSESession (ephemeral) | P2-R2 | -- | COVERED |
| VO-2.01 | SERVICE_NAME | P2-R1 | -- | COVERED |
| VO-2.02 | PERMISSION_MARKER | P2-R1 | -- | COVERED |
| VO-2.03 | STORAGE_DIR | P2-R1 | -- | COVERED |
| VO-2.04 | IS_DOCKER | P2-R1 | -- | COVERED |
| VO-2.05 | MCP Tool Response Shape | P2-R1 | -- | COVERED |
| VO-2.06 | Port Configuration | P2-R1 | -- | COVERED |
| VO-2.07 | SSE Session ID | P2-R1 | P2-R2 | COVERED |
| VO-2.08 | App Bundle Identity | P2-R2 | -- | COVERED |

---

## 6. NFR Coverage

| ID | NFR | Identified In | Status |
|----|-----|---------------|--------|
| SEC-001 | Encryption at rest | Broad sweep, P4-R1 | COVERED |
| SEC-002 | Transport security | Broad sweep, P4-R1 | COVERED |
| SEC-003 | Input sanitization / path traversal | Broad sweep, P4-R1, P4-R2 | COVERED |
| SEC-004 | Secret exposure in responses | Broad sweep, P4-R1 | COVERED |
| SEC-005 | Permission marker namespace pollution | P4-R1 | COVERED |
| SEC-006 | Docker storage chmod 777 | P4-R1, P4-R2 | COVERED |
| SEC-007 | Container runs as root | P4-R2 | COVERED |
| SEC-008 | No rate limiting on HTTP | P4-R2 | COVERED |
| REL-001 | Error recovery asymmetry | P3-R1 (OBS-3.03), P4-R1 | COVERED |
| REL-002 | Graceful shutdown missing | Broad sweep, P4-R1 | COVERED |
| REL-003 | Startup reliability | P4-R1 | COVERED |
| REL-004 | Data durability | P4-R1 | COVERED |
| REL-005 | Idempotency | P4-R1 | COVERED |
| OBS-001 | Logging (console.error only) | Broad sweep, P4-R1 | COVERED |
| OBS-002 | Audit trail missing | P4-R1 | COVERED |
| OBS-003 | Health monitoring broken | P4-R1 | COVERED |
| PERF-001 | Latency profile | P4-R1 | COVERED |
| PERF-002 | Concurrency model | P4-R1 | COVERED |
| PERF-003 | Resource consumption | P4-R1 | COVERED |
| SCALE-001 | Credential count limits | P4-R1 | COVERED |
| SCALE-002 | Multi-tenant missing | P4-R1 | COVERED |
| COMP-001 | License inconsistency (ISC vs MIT) | P0-R2, P4-R1 | COVERED |
| COMP-002 | Container image OCI labels | P4-R1 | COVERED |
| TEST-001 | Zero test infrastructure | Broad sweep, P4-R2 | COVERED |

---

## 7. Anti-Pattern / Bug Coverage

| ID | Issue | Identified In | Status |
|----|-------|---------------|--------|
| AP-1 / ANTI-001 | Triple tool definition duplication | Broad sweep, P1-R1, P5-R1 | COVERED |
| AP-2 / PAT-002 | Inline strategy selection (IS_DOCKER) | Broad sweep, P5-R1 | COVERED |
| AP-3 | Async constructor side effect | Broad sweep, P3-R1, P3-R2 | COVERED |
| AP-4 / ANTI-004 | Dead code (src/cli.js) | P0-R1, P0-R2, P3-R2 | COVERED |
| AP-5 | Dependencies in wrong category | P0-R1, P0-R2 | COVERED |
| AP-6 | No shared tool module | Broad sweep, P1-R1 | COVERED |
| AP-7 | SSE session Date.now() | Broad sweep, P3-R2 | COVERED |
| AP-8 | HEALTHCHECK protocol mismatch | P0-R1, P1-R1, P2-R1, P4-R1 | COVERED |
| AP-9 | Unused keytar import in index.ts | Broad sweep, P0-R1, P3-R1 | COVERED |
| AP-10 | No graceful shutdown | P1-R1, P4-R1 | COVERED |
| ANTI-002 | Sync I/O in async methods | P5-R1, P5-R2, P4-R1 | COVERED |
| ANTI-003 | Unsafe error type assertion | P5-R1, P5-R2 | COVERED |
| ANTI-005 | README documents non-functional workflow | P5-R1 | COVERED |
| OBS-3.05 | cli.js spawns wrong server (server.js not index.js) | P3-R2 | COVERED |
| DMG bug | Launcher references main.js (nonexistent) | P0-R1, P2-R2, P3-R2 | COVERED |
| License | ISC vs MIT inconsistency | P0-R2, P4-R1, P4-R2 | COVERED |
| ESLint | No config file; lint script non-functional | P0-R2, P5-R1 | COVERED |
| cli.js name collision | tsc output overwrites cli.js path | P0-R1 | COVERED |

---

## 8. Blind Spot Scan

Systematic check for uncovered items:

### Checked: Are there any functions/methods not in the BC catalog?

Re-reading all source files against the BC catalog:

- `src/cli.js:printUsage()` (line 137) -- referenced in BC-3.05.004 as called before `process.exit(0)`. Not a standalone BC but covered as part of the help flow. **No gap.**
- `src/cli.ts:printUsage()` (line 94) -- referenced in BC-3.04.005 and BC-3.04.006. **No gap.**
- `src/server.ts` line 230: `export { server }` -- covered in CONV-007. **No gap.**
- `src/cli.js` line 163: bare `main()` call (no `.catch`) -- the cli.js file has `main()` at line 163 with no `.catch()`, unlike cli.ts which has `main().catch(...)` at line 115. The try/catch inside main handles errors, but unhandled promise rejections from the finally block's `client.disconnect()` would be uncaught. **Minor gap -- documenting below.**

### Checked: Are there any environment variables not cataloged?

| Variable | Documented In | Status |
|----------|---------------|--------|
| `DOCKER_ENV` | P0-R1, P2-R1, P5-R1 | COVERED |
| `STORAGE_DIR` | P0-R1, P2-R1, P5-R1 | COVERED |
| `PORT` | P2-R1, P5-R1 | COVERED |
| `NODE_ENV` | P1-R2, P5-R2 | COVERED (set by Smithery, never read) |

**No gaps.**

### Checked: Are there import paths not analyzed?

All imports in all 5 TS/JS source files have been traced:
- `@modelcontextprotocol/sdk/server/mcp.js` -- covered
- `@modelcontextprotocol/sdk/server/stdio.js` -- covered
- `@modelcontextprotocol/sdk/server/sse.js` -- covered
- `@modelcontextprotocol/sdk/client` -- covered (cli.js)
- `@modelcontextprotocol/sdk/client/stdio.js` -- covered (cli.js)
- `express` -- covered
- `zod` -- covered
- `keytar` -- covered
- `fs`, `path`, `url`, `child_process` -- covered as Node builtins
- `./services/keychain.js` -- covered

**No gaps.**

### Checked: Are there any code paths in keychain.ts not contracted?

Line-by-line verification:
- Lines 1-8: Imports + constants -- VO-2.01 to VO-2.04. COVERED.
- Lines 10-14: Class JSDoc comment -- CONV-004. COVERED.
- Lines 15-25: Class + constructor -- BC-2.03.001. COVERED.
- Lines 27-39: ensureStorageDirectory -- BC-2.03.003. COVERED.
- Lines 42-61: checkPermissionMarker -- BC-2.03.002. COVERED.
- Lines 63-79: storeKey -- BC-2.01.001. COVERED.
- Lines 81-92: storeKeyFile -- BC-2.02.001. COVERED.
- Lines 94-109: getKey -- BC-2.01.002. COVERED.
- Lines 111-125: getKeyFile -- BC-2.02.002. COVERED.
- Lines 127-142: deleteKey -- BC-2.01.003. COVERED.
- Lines 144-159: deleteKeyFile -- BC-2.02.003. COVERED.
- Lines 161-180: listKeys -- BC-2.01.004. COVERED.
- Lines 182-195: listKeyFiles -- BC-2.02.004. COVERED.
- Lines 196-198: Class close + singleton export -- PAT-001. COVERED.

**All 198 lines accounted for.**

---

## 9. Micro-Gap: cli.js Unhandled Promise in main() Call

**Finding:** `src/cli.js` line 163 calls `main()` without `.catch()`:
```javascript
main();
```

Compare to `src/cli.ts` line 115 which has:
```typescript
main().catch(err => {
  console.error('Error:', err instanceof Error ? err.message : String(err));
  process.exit(1);
});
```

In cli.js, the `finally` block calls `await client.disconnect()` (line 131). If `disconnect()` throws, the rejection propagates to the `main()` promise. Without a `.catch()` at the call site, this would be an unhandled promise rejection. Node.js 20 treats unhandled promise rejections as fatal by default (`--unhandled-rejections=throw`), so this would crash the process with a generic error instead of a formatted message.

**Severity:** LOW -- cli.js is dead code (unreachable after build), but this is a correctness gap not previously noted.

**Contract:** BC-AUDIT-001

**BC-AUDIT-001: cli.js main() call lacks error handler for finally-block rejections**

- **Preconditions:** cli.js is executed (hypothetically, since it is dead code)
- **Postconditions:** If `client.disconnect()` in the `finally` block throws, the rejection is unhandled. Node.js 20 will crash with an `UnhandledPromiseRejection` error.
- **Evidence:** cli.js line 163 vs cli.ts line 115-117
- **Confidence:** HIGH (trivially verifiable from code)
- **Impact:** NONE (dead code)

---

## 10. Verdict

| Check | Result |
|-------|--------|
| All 22 files inventoried | PASS |
| All files referenced in at least one analysis pass | PASS |
| All source code functions have behavioral contracts | PASS (33 BCs + 1 audit BC) |
| All entities and value objects cataloged | PASS (5 entities, 8 value objects) |
| All NFRs cataloged | PASS (24 NFRs across 7 categories) |
| All anti-patterns/bugs cataloged | PASS (18 issues) |
| All environment variables documented | PASS (4 variables) |
| All import paths traced | PASS |
| All code paths in core service contracted | PASS (198/198 lines) |
| Blind spot scan found substantive gaps | NO -- 1 micro-gap in dead code |

**COVERAGE AUDIT: PASS**

The prior analysis (broad sweep + 12 deepening rounds across 6 passes) achieves complete coverage of the serveMyAPI codebase. Every source file, every function, every code path, every entity, and every environmental configuration has been analyzed, contracted, and cross-referenced. The single micro-gap found (BC-AUDIT-001) is in dead code and has zero impact.

---

## State Checkpoint

```yaml
phase: B.5
task: coverage-audit
status: PASS
files_inventoried: 22
files_covered: 22
behavioral_contracts: 34
entities: 5
value_objects: 8
nfrs: 24
anti_patterns: 18
blind_spots_found: 1 (micro-gap in dead code, zero impact)
timestamp: 2026-04-13T23:59:00Z
```
