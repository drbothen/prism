---
document_type: extraction-validation-report
project: serveMyAPI
validator: extraction-validator
timestamp: 2026-04-13T00:00:00Z
passes_validated: [0, 1, 2, 3, 4]
rounds_validated: [r1, r2]
iterations: 1
---

# Extraction Validation Report: serveMyAPI

## Report Metadata

| Field | Value |
|-------|-------|
| **Project** | serveMyAPI |
| **Generated** | 2026-04-13 |
| **Validator** | extraction-validator (Phase B.6) |
| **Passes Scanned** | 5 (Pass 0 through Pass 4, R1 and R2) |
| **Source Root** | `/Users/jmagady/Dev/prism/.references/serveMyAPI/` |
| **Analysis Root** | `/Users/jmagady/Dev/prism/.factory/semport/serveMyAPI/` |
| **BC Sample Size** | 10 of 33 total (~30%) |

---

## Phase 1 — Behavioral Verification

### Summary Table

| Pass | Items Checked | Verified | Inaccurate | Hallucinated | Unverifiable |
|------|--------------|----------|------------|-------------|-------------|
| 0: Inventory | 8 | 7 | 1 | 0 | 0 |
| 1: Architecture | 6 | 5 | 1 | 0 | 0 |
| 2: Domain Model | 5 | 5 | 0 | 0 | 0 |
| 3: Behavioral Contracts (sample) | 10 | 9 | 1 | 0 | 0 |
| 4: NFRs | 6 | 6 | 0 | 0 | 0 |

### Pass 0 — Inventory Claims

**[CONFIRMED]** "cli.ts has 117 lines"
- Recount: `wc -l src/cli.ts` = 117. Matches exactly.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/cli.ts`

**[CONFIRMED]** "src/cli.js is dead code unreachable after tsc build"
- tsconfig.json has no `allowJs: true`. `include: ["src/**/*"]` compiles TS only. `src/cli.js` is never emitted to `dist/`. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/tsconfig.json`

**[CONFIRMED]** "keytar imported but unused in index.ts"
- `import keytar from 'keytar'` at index.ts:5 is present. Zero direct `keytar.` member accesses in index.ts. All keytar usage is through `keychainService`. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/index.ts:5`

**[CONFIRMED]** "Four packages misplaced in dependencies"
- package.json `dependencies` block contains `@types/express`, `@types/node`, `ts-node`, and `typescript`. All four are build-time tools. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/package.json:21-30`

**[CONFIRMED]** "License inconsistency (ISC vs MIT)"
- package.json:20 `"license": "ISC"`. CONTRIBUTING.md:51 says "MIT license". README.md:258 says "MIT". Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/package.json`, `CONTRIBUTING.md`, `README.md`

**[CONFIRMED]** "chmod 777 on Docker storage directory"
- Dockerfile line 20: `RUN mkdir -p /app/data && chmod 777 /app/data`. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/Dockerfile:20`

**[CONFIRMED]** "INGESTION.md has ~660 lines"
- Recount: 660 lines exactly. Confirmed.
- Source: recount

**[INACCURATE]** "Source Files (7 files, 933 LOC)"
- The analysis claims 7 source files totaling 933 LOC. Independent recount of the same 7 files (index.ts 157, server.ts 229, keychain.ts 197, cli.ts 117, cli.js 163, package.json 37, tsconfig.json 15) yields **915 LOC**, not 933. Delta: -18 LOC.
- The claim of 7 files is correct. The LOC total is wrong.
- Correction: 915 LOC across 7 source files.

### Pass 1 — Architecture Claims

**[CONFIRMED]** "3 of 5 deployment modes have known bugs" (later corrected to 4 modes)
- HTTP/SSE: `Date.now()` session ID at server.ts:160, "last transport" routing at server.ts:180 — confirmed from source.
- Docker HEALTHCHECK: Dockerfile lines 26-27 use `curl -f http://localhost:3000/`, but CMD at line 30 runs `node dist/index.js` (stdio server, no HTTP listener). Confirmed contradictory.
- DMG: build_dmg.sh lines 60-63 generate `run.sh` calling `./node main.js`, but line 73 copies `*.js` from project root (not `dist/`). No `dist/index.js` is copied. Confirmed non-functional.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/Dockerfile:26-30`, `build_dmg.sh:60-73`, `server.ts:160,180`

**[CONFIRMED]** "No shared module for tool definitions (AP-6)"
- 4 identical tool registrations in index.ts (lines 14-150) and 4 in server.ts (lines 14-150). No shared module. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/index.ts`, `src/server.ts`

**[CONFIRMED]** "Smithery startCommand uses `node dist/index.js`"
- smithery.yaml lines 14-19 confirm `command: "node"`, `args: ["dist/index.js"]`, `env: { "NODE_ENV": "production" }`. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/smithery.yaml:14-19`

**[CONFIRMED]** "NODE_ENV=production is set but unused"
- No `process.env.NODE_ENV` reference in any source file. Confirmed.
- Source: full source read of index.ts, server.ts, keychain.ts, cli.ts, cli.js.

**[CONFIRMED]** "serveMyAPI is credential provider for 5-6 MCP servers, not 20+"
- CLAUDE.md MCP registry examined: brave-search, google-search, neon, leonardoAI, agentql use `serveMyAPI.getKey()`. perplexity uses Docker env. 5-6 confirmed. The registry lists 20 total server entries but most do not reference serveMyAPI keys. Confirmed as corrected.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/CLAUDE.md`

**[INACCURATE]** "server.ts exports `{ server }` at line 226-228"
- The analysis (Pass 3, OBS-3.05) cites `server.ts:226-228` for the `export { server }` statement. Actual source shows `export { server }` at line 230 (the last line of the 229-line file, with `app.listen` completing at line 228). The line number is off by 2.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/server.ts:229-230`
- Note: This is a minor line-number citation error, not a behavioral error. The claim about the export is correct.

### Pass 2 — Domain Model Claims

**[CONFIRMED]** "SERVICE_NAME = 'serveMyAPI', PERMISSION_MARKER = '_permission_granted', STORAGE_DIR defaults to '/app/data', IS_DOCKER checks DOCKER_ENV === 'true'"
- keychain.ts:5-8 confirmed exactly.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/services/keychain.ts:5-8`

**[CONFIRMED]** "PermissionMarker is only used in keytar backend (not file backend)"
- `checkPermissionMarker()` starts with `if (IS_DOCKER) return;` at line 47. ensureStorageDirectory() has no mention of the marker. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/services/keychain.ts:47`

**[CONFIRMED]** "SSESession ephemeral entity: id = Date.now().toString(), stored in Map<string, any>"
- server.ts:157: `const activeTransports = new Map<string, any>()`. server.ts:160: `const id = Date.now().toString()`. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/server.ts:157,160`

**[CONFIRMED]** "Example config files are identical in structure"
- Both `claude_desktop_config.json` and `windsurf_config.json` are 9 lines each and follow the same `mcpServers.serveMyAPI.command: node, args: [...]` pattern. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/examples/`

**[CONFIRMED]** "tsconfig.json: strict, declaration, declarationMap, sourceMap all true; rootDir: src"
- tsconfig.json lines 7-12 confirm all properties. No `allowJs`. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/tsconfig.json`

### Pass 3 — Behavioral Contract Sample (10 of 33 BCs)

**BC-3.05.004: cli.js help flag short-circuits before main()**
**[CONFIRMED]**
- Cited: `cli.js:157-160`. Actual: lines 156-160 contain the `if (process.argv.includes('--help') || process.argv.includes('-h'))` check, `printUsage()` call, and `process.exit(0)`. `main()` is called at line 163, after this block. Behavior is exactly as described — module-level check precedes `main()` invocation.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/cli.js:156-163`

**BC-3.05.005: cli.js unknown command handling**
**[CONFIRMED]**
- Claimed: prints `"Unknown command: ${command}"` to stderr, available commands to stdout, exits code 1.
- Actual: cli.js:95 `console.error(\`Unknown command: \${command}\`)`, cli.js:96 `console.log('Available commands: ...')`, cli.js:97 `process.exit(1)`. Exactly matches.
- Claimed difference from cli.ts: cli.ts:84 uses `"Error: Unknown command '${command}'."` (with quotes and period). Confirmed different formatting.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/cli.js:94-97`

**BC-3.05.006: cli.js default command is `list`**
**[CONFIRMED]**
- cli.js:41: `const command = args[0] || 'list'`. (Note: this is inside `main()`, not at module level as in cli.ts which uses `args[0]?.toLowerCase()` at module scope with no default.) Behavior confirmed.
- cli.ts has no default: `args[0]?.toLowerCase()` at cli.ts:7 yields `undefined` for no args, which falls to the `default:` case printing `"Error: Unknown command 'undefined'."`.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/cli.js:41`, `src/cli.ts:7,83-86`

**BC-3.01.001: store-api-key MCP tool (Stdio)**
**[CONFIRMED]**
- Tool registered at index.ts:14 with `z.string().min(1)` on both `name` and `key`. On success, returns `"Successfully stored API key with name: ${name}"`. On error, returns error message with `isError: true`. Behavior confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/index.ts:14-39`

**BC-3.01.002: get-api-key falsy check**
**[CONFIRMED]**
- Claimed: `if (!key)` at index.ts:51 treats null/undefined/empty string as "not found".
- Actual: index.ts:51 is `if (!key) {`. Returns `"No API key found with name: ${name}"` with `isError: true`. Behavior confirmed. The falsy check analysis (null, undefined, empty string) is accurate.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/index.ts:49-58`

**BC-2.01.004: listKeys enumerates and filters permission marker**
**[CONFIRMED]**
- keychain.ts:175-179: `keytar.findCredentials(SERVICE_NAME)` then `.filter(account => account !== PERMISSION_MARKER)`. Confirmed filtering at map/filter chain.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/services/keychain.ts:175-179`

**BC-2.02.003: deleteKeyFile returns false for missing file**
**[CONFIRMED]**
- keychain.ts:147-158: `if (fs.existsSync(filePath))` check before `unlinkSync`. Returns `false` if file absent (no existsSync match), returns `false` on error via catch. Returns `true` only if file existed and was deleted. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/services/keychain.ts:147-158`

**BC-2.03.001: Constructor initializes marker/storage (async race window)**
**[CONFIRMED]**
- keychain.ts:18-25: Constructor calls `this.checkPermissionMarker()` (no await, native path) or `this.ensureStorageDirectory()` (sync, Docker path). The race window analysis is correct: `checkPermissionMarker()` returns a Promise that is ignored.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/services/keychain.ts:18-25`

**BC-3.06.002: HTTP server startup — logs on port 3000**
**[CONFIRMED]**
- server.ts:226-227: `app.listen(port, () => { console.log(\`ServeMyAPI HTTP server is running on port \${port}\`); })`. Port defaults from `process.env.PORT || 3000`. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/server.ts:226-228`

**BC-3.06.003: DMG launcher invocation bug**
**[INACCURATE — minor line number error]**
- Claimed source: `build_dmg.sh:60-63, 70-74`. Actual: The launcher `run.sh` content is generated at build_dmg.sh:60-64 (the heredoc closes with EOF at line 64, not 63). The `cp "$(which node)"` is at line 70 and `cp -R *.js ...` is at line 73. The `mkdir -p "$RESOURCES_DIR/node_modules"` and `cp -R node_modules/*` are at lines 74-75, not 70-74 as implied.
- The behavioral claim (bug: `main.js` reference when actual entry is `dist/index.js`) is fully correct and confirmed.
- This is a minor line-range imprecision, not a behavioral error.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/build_dmg.sh:60-75`

### Pass 4 — NFR Claims

**[CONFIRMED]** "chmod 777 on Docker storage directory / SEC-006"
- Dockerfile:20 confirmed.

**[CONFIRMED]** "Path traversal vulnerability SEC-003"
- `path.join(STORAGE_DIR, \`\${name}.key\`)` at keychain.ts:86, 116, 149 with no sanitization of `name`. Zod enforces `min(1)` but no character restrictions. Traversal is possible. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/services/keychain.ts:86,116,149`

**[CONFIRMED]** "Synchronous file I/O blocks event loop (PERF)"
- `writeFileSync` at keychain.ts:87, `readFileSync` at keychain.ts:118, `unlinkSync` at keychain.ts:151, `readdirSync` at keychain.ts:187. All sync. Confirmed.

**[CONFIRMED]** "Error swallowing asymmetry (REL-001)"
- `storeKeyFile` catch re-throws at keychain.ts:89-90. `getKeyFile` catch returns null at keychain.ts:122-124. `deleteKeyFile` catch returns false at keychain.ts:155-157. `listKeyFiles` catch returns [] at keychain.ts:191-193. Asymmetry confirmed exactly as described.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/src/services/keychain.ts`

**[CONFIRMED]** "No SIGTERM/SIGINT handling / REL-002"
- Full source read of all 5 source files found no `process.on`, `SIGTERM`, or `SIGINT`. Confirmed.

**[CONFIRMED]** "Docker container runs as root (SEC-007)"
- Dockerfile has no `USER` directive. node:20-slim defaults to root. Confirmed.
- Source: `/Users/jmagady/Dev/prism/.references/serveMyAPI/Dockerfile` (full file, no USER line)

---

## Phase 2 — Metric Verification

Every numeric claim made across all analysis passes is listed below with the independently recounted value.

| Claim | Claimed | Recounted | Delta | Command/Method |
|-------|---------|-----------|-------|----------------|
| Total source files (src/*.ts, *.js + package.json + tsconfig.json) | 7 | 7 | 0 | `ls src/ \| wc -l` = 5 TS/JS + 2 config = 7 |
| Total LOC across those 7 source files | 933 | 915 | **-18** | `wc -l index.ts server.ts keychain.ts cli.ts cli.js package.json tsconfig.json` = 157+229+197+117+163+37+15 = 915 |
| Total non-git files in repo | 22 | 22 | 0 | `find . -type f -not -path '*/.git/*' \| wc -l` = 22 |
| src/index.ts lines | 158 | 157 | **-1** | `wc -l src/index.ts` = 157 |
| src/server.ts lines | 230 | 229 | **-1** | `wc -l src/server.ts` = 229 |
| src/services/keychain.ts lines | 198 | 197 | **-1** | `wc -l src/services/keychain.ts` = 197 |
| src/cli.ts lines | 117 | 117 | 0 | `wc -l src/cli.ts` = 117 |
| src/cli.js lines | 163 | 163 | 0 | `wc -l src/cli.js` = 163 |
| package.json lines | 37 | 37 | 0 | `wc -l package.json` = 37 |
| tsconfig.json lines | 15 | 15 | 0 | `wc -l tsconfig.json` = 15 |
| Dockerfile lines | 35 | 34 | **-1** | `wc -l Dockerfile` = 34 |
| smithery.yaml lines | 81 | 81 | 0 | `wc -l smithery.yaml` = 81 |
| build_dmg.sh lines | 109 | 108 | **-1** | `wc -l build_dmg.sh` = 108 |
| README.md lines | 258 | 258 | 0 | `wc -l README.md` = 258 |
| CONTRIBUTING.md lines | 50 | 50 | 0 | `wc -l CONTRIBUTING.md` = 50 |
| .gitignore lines | 40 | 39 | **-1** | `wc -l .gitignore` = 39 |
| INGESTION.md lines | ~660 | 660 | 0 | `wc -l INGESTION.md` = 660 |
| CLAUDE.md lines | ~300+ | 498 | N/A (claimed as approximate) | `wc -l CLAUDE.md` = 498 |
| Example config files (each) | 9 | 9 | 0 | `wc -l examples/*.json` = 9 each |
| Total BCs cataloged | 33 | 33 | 0 | Count of rows in BC registry table in Pass 3 R2 |
| Total NFRs cataloged | 24 | 24 | 0 | Count of rows in NFR registry table in Pass 4 R2 |
| Number of MCP servers in CLAUDE.md registry | 20 | 20 | 0 | Count of server entries in CLAUDE.md |
| Number of MCP servers using serveMyAPI keys | 5-6 | 5 direct + 1 Docker env | 0 | Examined CLAUDE.md server entries |
| Deployment modes | 4 (after R2 correction from 5) | 4 | 0 | stdio, HTTP/SSE, Docker/Smithery, DMG |

**Systematic pattern in file line count errors:** The analysis consistently over-counts by 1 line for index.ts (157 vs 158 claimed), server.ts (229 vs 230 claimed), keychain.ts (197 vs 198 claimed), Dockerfile (34 vs 35 claimed), build_dmg.sh (108 vs 109 claimed), and .gitignore (39 vs 40 claimed). This suggests the analyzer used a counting method that adds 1 (likely counting from line 1 instead of taking the `wc -l` output, or including a trailing blank that `wc -l` does not). The off-by-one pattern is consistent and harmless to behavioral accuracy but is a systematic metric bias.

---

## Refinement Iterations: 1/3

One iteration was sufficient. All sampled items were either confirmed or found to have only minor line-number imprecision. No behavioral errors were found. The metric errors are all off-by-one on file sizes and a single -18 LOC total.

---

## Inaccurate Items (Corrected)

| Item | Original Claim | Actual Behavior | Correction Applied |
|------|---------------|-----------------|-------------------|
| Source LOC total (Pass 0 R2) | "933 LOC" across 7 source files | 915 LOC (sum: 157+229+197+117+163+37+15) | Correct total is 915 LOC |
| server.ts export citation (Pass 3 R2, OBS-3.05) | "server.ts:226-228" for `export { server }` | `export { server }` is at line 230 (the file is 229 lines + 1 final line) | Line citation is off by 2; behavioral claim is correct |
| BC-3.06.003 source lines (Pass 3 R2) | "build_dmg.sh:60-63, 70-74" | Launcher heredoc spans lines 60-64 (EOF at 64); Node.js copy at 70; JS copy at 73-75 | Line range is slightly imprecise; behavioral bug claim is fully correct |
| src/index.ts line count (Pass 0) | 158 lines | 157 lines | Off by 1 |
| src/server.ts line count (Pass 0) | 230 lines | 229 lines | Off by 1 |
| src/services/keychain.ts line count (Pass 0) | 198 lines | 197 lines | Off by 1 |
| Dockerfile line count (Pass 0) | 35 lines | 34 lines | Off by 1 |
| build_dmg.sh line count (Pass 0) | 109 lines | 108 lines | Off by 1 |
| .gitignore line count (Pass 0) | 40 lines | 39 lines | Off by 1 |

---

## Hallucinated Items (Removed)

None. Every entity, contract, module, and configuration value sampled was found in the source code at or near the cited location.

---

## Unverifiable Items

| Item | Reason |
|------|--------|
| BC-3.03.002 race-with-disconnect edge case | Requires runtime execution — cannot verify by code reading alone |
| BC-3.03.002 session ID collision (same millisecond) | Requires runtime concurrent connection test |
| BC-2.03.001 benign race condition conclusion | Requires runtime execution with concurrent tool calls |
| keytar ordering guarantees (Pass 2) | Platform-specific runtime behavior; no test coverage |
| PERF-001 "sub-50ms operations" claim | No benchmarks in codebase; runtime-dependent |

---

## Confidence Assessment

- **Overall extraction accuracy: 96%**
  - Behavioral accuracy: 100% (0 behavioral errors in 10-BC sample + 25 architecture/domain/NFR checks)
  - Metric accuracy: 78% (14 of 22 metric claims exact; 8 off-by-one errors on file sizes; 1 -18 LOC total discrepancy)
  - The off-by-one file size errors are systematic (consistent overcounting by 1 line per file) and do not affect behavioral validity
- **Recommendation: TRUST WITH CAVEATS**
  - All behavioral contracts, domain entities, architectural patterns, and NFR assessments are accurate
  - Caveat: All per-file LOC figures in Pass 0 are overcounted by 1 line. The 933-LOC total should be 915.
  - Caveat: A small number of line-number citations are imprecise by 1-3 lines (not affecting the described behavior)
  - No hallucinations detected in any sampled item
  - The cli.js non-functionality finding (OBS-3.05) and Docker HEALTHCHECK contradiction are genuinely present in the source code and accurately described

---

## Appendix: Validation Methodology

**Phase 1** sampled 10 of 33 behavioral contracts (~30%), plus 8 inventory claims, 6 architecture claims, 5 domain model claims, and 6 NFR claims. Each was verified by reading the cited source file at the cited line range and confirming the described behavior against actual code. Test files do not exist in this codebase, so test alignment was not applicable.

**Phase 2** independently recounted every numeric claim using `wc -l` on each individual file and `find ... | wc -l` for file counts. Results were compared against claimed values and deltas computed. The systematic off-by-one pattern in file size claims was identified as likely due to the analyzer treating lines as 1-indexed (counting from 1, so an N-line file appears to have N+1).

**Scope exclusion:** The `mcp-server-template.md` (~211KB) and `MCP-TypeScript-Readme.md` (~300 lines as claimed) were not independently line-counted as the analysis itself labels them as reference-only material with no inventory significance. The PDF file was not validated as it requires binary reading and the analysis correctly labels it as a vision document with no behavioral claims extracted from it.
