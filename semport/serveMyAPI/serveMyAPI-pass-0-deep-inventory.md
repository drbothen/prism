# Pass 0 Deep: Inventory -- serveMyAPI (Round 1)

## Preamble

This deepening round re-examines the file manifest, dependency graph, tech stack, and line counts with precision. It cross-references against findings from Phase A (Passes 2/3) to catch inventory items those passes surfaced but Pass 0 missed.

---

## Corrections to Broad Sweep Inventory

### 1. Actual Line Counts (verified via wc -l)

| Path | Broad Sweep | Actual | Delta |
|------|-------------|--------|-------|
| `src/index.ts` | 158 | 157 | -1 |
| `src/server.ts` | 230 | 229 | -1 |
| `src/services/keychain.ts` | 198 | 197 | -1 |
| `src/cli.ts` | 118 | 117 | -1 (broad sweep was off by 1) |
| `src/cli.js` | 164 | 163 | -1 (broad sweep was off by 1) |
| `package.json` | 38 | 37 | -1 |
| `tsconfig.json` | 16 | 15 | -1 (broad sweep counted 16 but file has 15 code lines) |
| `Dockerfile` | 35 | 34 | -1 |
| `smithery.yaml` | 81 | 81 | 0 |
| `build_dmg.sh` | 109 | 108 | -1 |

**Total TS/JS/JSON code**: 915 lines (verified). The broad sweep stated "~870 lines of TypeScript" which undercounts -- 915 includes JSON config and the JS CLI, while the actual TypeScript is 157 + 229 + 197 + 117 = 700 lines.

### 2. Missing Files from Broad Sweep Manifest

The broad sweep file manifest omitted several files:

| Path | Type | Lines | Purpose |
|------|------|-------|---------|
| `.gitignore` | Config | 39 | Standard Node.js gitignore + CLAUDE.md exclusion |
| `CLAUDE.md` | Documentation | ~300+ | Project guide, author's personal AI memory, MCP config templates |
| `CONTRIBUTING.md` | Documentation | 50 | Contribution guidelines, future development ideas |
| `README.md` | Documentation | 258 | Full project documentation with installation, usage, roadmap |
| `How to improve serveMyAPI.pdf` | Design doc | 5 pages | **serveMyAPI 2.0 vision**: pre-signed URLs, browser extensions, pattern detection |
| `INGESTION.md` | Generated doc | ~660 | Copy of broad sweep output (self-referential) |
| `mcp-server-template.md` | Reference | ~large | MCP server implementation template/guide |
| `MCP-TypeScript-Readme.md` | Reference | ~300 | MCP TypeScript SDK readme |
| `icon.png` | Asset | binary | Application icon (99KB PNG) |
| `icon.svg` | Asset | markup | Application icon (SVG source) |

### 3. CLAUDE.md Contains Significant Context

The CLAUDE.md file (gitignored, but present in the reference copy) contains:
- **Author identity**: James William Peter King (James King), CEO of New Model Venture Capital
- **Personal technical preferences**: Svelte 5, Windsurf IDE, Vercel, Neon, Mac hardware
- **Complete MCP server registry**: 20+ MCP servers with config JSON for each
- **serveMyAPI's role in ecosystem**: It is the **credential provider** for all other MCP servers -- `brave-search`, `google-search`, `neon`, `leonardoAI`, `agentql` all reference `${await serveMyAPI.getKey('...')}` for their API keys
- **Security instruction contradiction**: CLAUDE.md says "Never log or expose API keys in plaintext" but the `get-api-key` tool returns keys as plaintext by design

This file reveals serveMyAPI's position as a **foundational infrastructure service** -- it is not just a standalone tool but the credential backbone for an entire ecosystem of MCP servers.

### 4. PDF Vision Document (Not in Broad Sweep)

"How to improve serveMyAPI.pdf" describes a **serveMyAPI 2.0** architecture:
- **Pre-signed URL pattern**: Instead of returning raw API keys, generate time-limited signed URLs that embed credentials
- **Browser extension system**: Cross-browser extensions (Chrome, Firefox, Safari, Edge) with pattern detection
- **Pattern recognition**: `$SECURE_API:service/endpoint?params` syntax detected in LLM conversations
- **Three-layer architecture**: OS Keychain (KeyVault) -> ServeMyAPI App (ExtensionBridge) -> Client Integrations
- **Key never leaves the vault**: The `_getKey()` method is private; only `generateSignedUrl()` is exposed

This document represents the **author's intended direction** and is relevant to Prism's design decisions.

### 5. Dependency Classification Refinement

The broad sweep listed dependencies but did not distinguish runtime from dev or misplaced:

**Runtime dependencies (in `dependencies` -- some misplaced):**

| Package | Version | Classification | Notes |
|---------|---------|---------------|-------|
| `@modelcontextprotocol/sdk` | ^1.7.0 | Runtime | Core MCP SDK |
| `express` | ^5.0.1 | Runtime | HTTP/SSE transport only |
| `keytar` | ^7.9.0 | Runtime + Native | C++ addon, requires node-gyp build |
| `zod` | ^3.24.2 | Runtime | Schema validation |
| `@types/express` | ^5.0.0 | **Misplaced** | Should be devDependency |
| `@types/node` | ^22.13.10 | **Misplaced** | Should be devDependency |
| `ts-node` | ^10.9.2 | **Misplaced** | Should be devDependency (dev runner) |
| `typescript` | ^5.8.2 | **Misplaced** | Should be devDependency (compiler) |

**Actual devDependencies:**

| Package | Version | Purpose |
|---------|---------|---------|
| `@typescript-eslint/eslint-plugin` | ^8.26.1 | Linting |
| `@typescript-eslint/parser` | ^8.26.1 | Linting |
| `eslint` | ^9.22.0 | Linting |
| `nodemon` | ^3.1.9 | Dev hot-reload |

**Impact**: Four packages (`@types/express`, `@types/node`, `ts-node`, `typescript`) are in `dependencies` instead of `devDependencies`. This means production `npm install` installs the TypeScript compiler and type definitions unnecessarily. In Docker, this adds build-time-only packages to the runtime image.

### 6. Entry Point Clarification

The broad sweep listed 4 entry points. Refined assessment with Phase A findings:

| # | File | Transport | Functional? | Notes |
|---|------|-----------|-------------|-------|
| 1 | `src/index.ts` | Stdio MCP | **Yes** | Primary entry point. Works correctly. |
| 2 | `src/server.ts` | HTTP/SSE MCP | **Partially** | SSE session management is broken for concurrent users (Pass 3 OBS-3.05) |
| 3 | `src/cli.ts` | Direct calls | **Yes** | Works but bypasses Zod validation |
| 4 | `src/cli.js` | MCP client | **No** | Spawns `server.js` (HTTP) instead of `index.js` (stdio) -- non-functional (Pass 3 OBS-3.05) |

### 7. npm Scripts Inventory

| Script | Command | Purpose | Notes |
|--------|---------|---------|-------|
| `build` | `tsc` | Compile TS to JS | Outputs to `dist/` |
| `start` | `node dist/index.js` | Run stdio server | Production entry |
| `dev` | `nodemon --watch 'src/**/*.ts' --exec 'ts-node' src/index.ts` | Dev mode | Hot reload |
| `lint` | `eslint 'src/**/*.ts'` | Lint | No lint config file in repo |
| `test` | `echo "Error: no test specified" && exit 1` | Placeholder | Zero test infrastructure |
| `cli` | `node dist/cli.js` | Run CLI | This runs the **broken** MCP client CLI, not the working direct CLI |

**Note**: `npm run cli` runs `dist/cli.js` (the broken MCP client CLI per Pass 3 findings), NOT `dist/cli.ts` (the working direct CLI). The `bin` field correctly points to `dist/cli.js` as the `api-key` command, but the compiled output of `cli.ts` would be `dist/cli.js` -- wait, there is a **name collision**: both `src/cli.ts` and `src/cli.js` would compile/copy to `dist/cli.js`. The TypeScript compiler would overwrite the JS file, or the JS file would overwrite the TS output. This is a latent build conflict.

**Build conflict analysis**: `tsconfig.json` sets `rootDir: "src"` and `outDir: "dist"`. The `include` pattern is `src/**/*`. Since `src/cli.js` is a plain JS file (not TS), `tsc` may or may not copy it. TypeScript by default does NOT copy `.js` files from `rootDir` to `outDir` unless `allowJs` is true (it is not in this config). So `tsc` will compile `src/cli.ts` to `dist/cli.js`, and `src/cli.js` (the MCP client CLI) will NOT be in `dist/`. The `bin.api-key` field pointing to `dist/cli.js` will get the TypeScript-compiled direct CLI, not the MCP client CLI. The `npm run cli` script also runs `dist/cli.js` which will be the direct CLI. So **the MCP client CLI (`src/cli.js`) is dead code** -- it cannot be reached via any npm script or bin entry after build.

### 8. Smithery Deployment Configuration

The `smithery.yaml` file (81 lines) defines:
- Stdio transport type (not HTTP)
- Empty config schema (no user-configurable options)
- Tool schemas matching the Zod definitions in source code
- Dockerfile build reference
- Explicit note: "macOS-only service"

The Smithery tool schemas use JSON Schema draft-07 format, which differs from the Zod format in the source code. The schemas are **manually duplicated** -- a third copy of tool definitions (alongside index.ts and server.ts).

---

## Revised Dependency Graph

```
src/index.ts -----> src/services/keychain.ts -----> keytar (native)
    |                        |                          |
    |                        +-----> fs (Node builtin)  +---> macOS Keychain
    |                        +-----> path (Node builtin)+---> Win Credential Vault
    |                        |                          +---> Linux libsecret
    |                        +-----> process.env
    |
    +-----> @modelcontextprotocol/sdk (server/mcp.js, server/stdio.js)
    +-----> zod
    +-----> keytar (UNUSED IMPORT -- dead code at line 5)

src/server.ts ----> src/services/keychain.ts (same singleton)
    +-----> express
    +-----> @modelcontextprotocol/sdk (server/mcp.js, server/sse.js)
    +-----> zod

src/cli.ts -------> src/services/keychain.ts (same singleton)
    (no other deps)

src/cli.js -------> @modelcontextprotocol/sdk (client, client/stdio.js)
    +-----> child_process (spawn -- but actually handled by StdioClientTransport)
    +-----> path, url (Node builtins)
    (DOES NOT import keychain.ts -- communicates via MCP protocol)
    (DEAD CODE -- unreachable after tsc build)

smithery.yaml ----> Dockerfile (build reference)
                    Tool schemas (manual copy #3)

package.json -----> bin: api-key -> dist/cli.js (compiled from cli.ts, NOT cli.js)
                    main: dist/index.js
                    scripts: 6 defined, test is placeholder
```

---

## Revised Tech Stack

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Language | TypeScript | 5.8.2 | strict mode, ES2022 target, NodeNext module |
| Runtime | Node.js | 20+ | Dockerfile uses node:20-slim |
| MCP SDK | @modelcontextprotocol/sdk | ^1.7.0 | Server + Client APIs |
| OS Keyring | keytar | ^7.9.0 | Native C++ addon via node-gyp |
| HTTP Framework | Express | ^5.0.1 | Express 5 (major version, breaking changes from 4) |
| Schema Validation | Zod | ^3.24.2 | Tool parameter schemas only |
| Build | tsc | (bundled with typescript 5.8.2) | No bundler, no minification |
| Test Framework | **None** | N/A | Placeholder npm test script |
| Lint | ESLint | ^9.22.0 | With TypeScript parser; no config file in repo |
| Dev Runner | ts-node | ^10.9.2 | Misplaced in dependencies |
| Dev Reload | nodemon | ^3.1.9 | Watches src/**/*.ts |
| Package Manager | npm | (system) | package-lock.json gitignored |
| Deployment | Docker, Smithery, DMG | N/A | Three deployment targets |
| Module System | ES Modules | `"type": "module"` | All imports use ESM syntax |

---

## Delta Summary
- New items added: 10 missing files in manifest (including critical CLAUDE.md, PDF vision doc, CONTRIBUTING.md, README.md, icon assets, reference docs), dependency misclassification analysis (4 packages), npm scripts inventory with `cli` script analysis, build conflict discovery (cli.ts vs cli.js name collision), Smithery as third tool schema copy, serveMyAPI ecosystem role from CLAUDE.md
- Existing items refined: Line counts corrected for 4 files, total TS LOC corrected (700 not ~870), entry point functionality status updated per Phase A, dependency graph updated with dead code annotations
- Remaining gaps: Whether `eslint` has a config file elsewhere (not in repo root), exact Smithery hosting behavior, `mcp-server-template.md` contents (large reference doc not analyzed)

## Novelty Assessment
Novelty: SUBSTANTIVE
Multiple model-changing discoveries: (1) serveMyAPI is the **credential backbone** for 20+ MCP servers in the author's ecosystem (from CLAUDE.md), not just a standalone tool -- this fundamentally changes how Prism should think about the service's criticality. (2) The PDF vision document reveals a planned 2.0 architecture with pre-signed URLs and browser extensions that Prism should be aware of. (3) The `src/cli.js` is confirmed dead code due to the tsc build conflict. (4) Four dependencies are misclassified. (5) The `npm run cli` script runs the wrong CLI variant.

## Convergence Declaration
Another round needed -- want to examine `mcp-server-template.md` for any architectural patterns, verify ESLint config absence, and audit the README roadmap items against actual implementation.

## State Checkpoint
```yaml
pass: 0
round: 1
status: complete
files_scanned: 22
timestamp: 2026-04-13T23:30:00Z
novelty: SUBSTANTIVE
```
