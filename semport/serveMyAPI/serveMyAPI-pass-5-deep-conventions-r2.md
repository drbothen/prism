# Pass 5 Deep: Conventions & Patterns -- serveMyAPI (Round 2)

## Preamble

Hallucination audit and gap closure round. Verifies Round 1 convention claims, checks smithery.yaml schema drift against Zod, and audits PDF vision document for convention patterns.

---

## Hallucination Audit

### Claim: "Unsafe error type assertion `(error as Error).message` in 8 instances"
**Verified:** index.ts lines 33, 71, 109, 144 (4 instances) and server.ts lines 33, 71, 109, 144 (4 instances). Total: 8 instances. **Confirmed.**

### Claim: "CLI files use `instanceof Error` check instead"
**Verified:** cli.ts lines 89, 116: `error instanceof Error ? error.message : String(error)`. cli.js line 127: `error.message` (no instanceof check -- direct property access, which will throw if error is not an object). **Partial correction:** cli.ts has the safe pattern (2 instances). cli.js does NOT -- it uses bare `error.message` at line 127. Only cli.ts is safe; cli.js is unsafe in a different way (no `as Error` cast, but also no `instanceof` guard).

### Claim: "storeKeyFile IS declared async, others are NOT"
**Verified from keychain.ts:**
- Line 84: `private async storeKeyFile(name: string, key: string): Promise<void>` -- ASYNC
- Line 114: `private getKeyFile(name: string): string | null` -- NOT ASYNC (returns sync)
- Line 147: `private deleteKeyFile(name: string): boolean` -- NOT ASYNC (returns sync)
- Line 185: `private listKeyFiles(): string[]` -- NOT ASYNC (returns sync)

**Confirmed.** Only `storeKeyFile` is async; the others return synchronous values.

### Claim: "No eslint config file in repo"
**Verified:** Glob for `*eslint*`, `*.eslintrc*` found no files. `package.json` has no `eslintConfig` key. **Confirmed.**

### Claim: "Triple tool definition duplication (index.ts, server.ts, smithery.yaml)"
**Verified:** All three files define the same 4 tools. Now checking for schema drift:

**Smithery vs Zod parameter descriptions comparison:**

| Tool | Parameter | Zod (.describe()) | Smithery (description) | Match? |
|------|-----------|-------------------|----------------------|--------|
| store-api-key | name | "The name/identifier for the API key" | "The name/identifier for the API key" | EXACT |
| store-api-key | key | "The API key to store" | "The API key to store" | EXACT |
| get-api-key | name | "The name/identifier of the API key to retrieve" | "The name/identifier of the API key to retrieve" | EXACT |
| delete-api-key | name | "The name/identifier of the API key to delete" | "The name/identifier of the API key to delete" | EXACT |

**Parameter schemas match exactly.** No drift at the parameter level.

**Smithery has additional fields not in Zod:**
- `additionalProperties: false` on each parameter schema (5 instances)
- `$schema: "http://json-schema.org/draft-07/schema#"` on each parameter schema
- Tool-level `name` and `description` fields (e.g., `description: "Store an API key securely in the keychain"`)

The Zod schemas in the source code do not generate tool-level descriptions. The MCP SDK's `server.tool()` method takes `(name, inputSchema, handler)` -- there is no tool description parameter. So the Smithery tool descriptions exist only in smithery.yaml and are not part of the MCP server's runtime metadata.

**Minor drift:** Smithery tool descriptions reference "keychain" consistently, while the actual MCP tool behavior depends on the deployment (keytar on native, file on Docker). This is a documentation inaccuracy for Docker/Smithery deployments.

### Claim: "Smithery `commandFunction` adds `NODE_ENV: production` but no code reads it"
**Verified:** Grep for `NODE_ENV` across all source files: zero matches. **Confirmed.** The environment variable is set but never read.

---

## Gap Closure: PDF Vision Document Convention Patterns

The PDF proposes code patterns that differ from current conventions:

| Current Convention | Proposed (PDF) | Implication |
|-------------------|---------------|-------------|
| Direct `keytar` calls | `SecureStorage` abstraction | Proper dependency inversion |
| Raw key return | `generateSignedUrl()` pattern | Key never leaves vault |
| No pattern detection | `APIPatternDetector` class with regex | New subsystem |
| No browser integration | `ExtensionBridge` class with Chrome messaging | New subsystem |
| camelCase methods | camelCase methods (same) | Consistent |
| Class-based architecture | Class-based architecture (same) | Consistent |

The vision document's code samples follow the SAME naming conventions (camelCase, PascalCase classes) as the current codebase. This suggests the author intends to maintain current conventions in the 2.0 architecture.

## Gap Closure: Comment Style Conventions

| Comment Type | Convention | Examples | Consistency |
|-------------|-----------|----------|-------------|
| Single-line | `// Comment` | index.ts:7 `// Create an MCP server` | CONSISTENT |
| JSDoc | `/** ... */` with `@param`, `@returns` | keychain.ts:64-68 | CONSISTENT in service |
| Block comment | `/* ... */` | Not used | N/A |
| TODO/FIXME | Not used anywhere | No instances found | N/A |
| Section headers | `// Tool to store/retrieve/delete/list an API key` | index.ts:13, 41, 79, 117 | CONSISTENT |

**Absence of TODOs:** Despite known issues (SSE session management, Docker security, no tests), there are zero TODO or FIXME comments in the codebase. This suggests either the author is unaware of these issues or tracks them elsewhere.

## Gap Closure: TypeScript-Specific Conventions

| Convention | Setting | Source |
|-----------|---------|--------|
| Target | ES2022 | tsconfig.json:3 |
| Module | NodeNext | tsconfig.json:4 |
| Module resolution | NodeNext | tsconfig.json:5 |
| Strict mode | true | tsconfig.json:7 |
| Source maps | true | tsconfig.json:10 |
| Declarations | true + declarationMap | tsconfig.json:11-12 |
| ES interop | esModuleInterop: true | tsconfig.json:6 |

**ES2022 target implications:** Enables top-level await, class fields, `Array.at()`, `Object.hasOwn()`, error cause. The codebase does not use top-level await (the server startup uses `.then()` chains instead). No ES2022-specific features are used.

**NodeNext module resolution:** Requires `.js` extension on relative imports even for TypeScript files. All imports comply: `./services/keychain.js`, `./server.js`. This is a strict convention enforced by the compiler.

---

## Consolidated Convention & Pattern Catalog

### Conventions (7 categories)

| ID | Convention | Scope | Consistency | Notes |
|----|-----------|-------|-------------|-------|
| CONV-001 | 2-space indent, semicolons, K&R braces | All files | FULL | Standard TS formatting |
| CONV-002 | camelCase vars/functions, PascalCase classes, UPPER_SNAKE constants | All files | FULL | Standard TS naming |
| CONV-003 | Import order: third-party then local, `.js` extensions | All TS files | FULL | NodeNext requirement |
| CONV-004 | JSDoc on service methods, inline comments on transport | Service vs transport | SPLIT | Service: full JSDoc; transport: none |
| CONV-005 | Error handling: try/catch with formatted responses | All async code | STRUCTURALLY consistent | Recovery behavior varies (ANTI-003) |
| CONV-006 | Async/await for all async code | All files | FULL except constructor | Constructor fire-and-forget is intentional |
| CONV-007 | ES module exports (named + default) | All modules | CONSISTENT | Singleton pattern for service |

### Patterns (6 cataloged)

| ID | Pattern | Location | Quality |
|----|---------|----------|---------|
| PAT-001 | Singleton service | keychain.ts:198 | Adequate |
| PAT-002 | Conditional strategy (inline) | keychain.ts (all CRUD) | Should be proper Strategy |
| PAT-003 | MCP tool registration template | index.ts, server.ts | Consistent but duplicated |
| PAT-004 | Belt-and-suspenders guard | keychain.ts (4 methods) | Defensive, effective |
| PAT-005 | Manual argv parsing | cli.ts, cli.js | Simple but limited |
| PAT-006 | Module-level env config | keychain.ts:5-8, server.ts:154 | 4 env vars, undocumented |

### Anti-Patterns (5 cataloged)

| ID | Anti-Pattern | Impact | Severity |
|----|-------------|--------|----------|
| ANTI-001 | Triple tool definition duplication | Maintenance burden, drift risk | HIGH |
| ANTI-002 | Sync I/O in async context | Event loop blocking (minimal) | LOW |
| ANTI-003 | Unsafe `(error as Error).message` cast | Undefined in error messages | MEDIUM |
| ANTI-004 | Plain JS file in strict TS project | Dead code, name collision | LOW |
| ANTI-005 | README documents non-functional workflows | User confusion | LOW |

---

## Delta Summary
- New items added: Smithery schema drift analysis (parameter-level: no drift; tool-level descriptions: Smithery-only), comment style conventions, TypeScript-specific conventions (ES2022 target, NodeNext modules), absence of TODO/FIXME comments
- Existing items refined: cli.js error handling corrected (bare `error.message`, not `instanceof`), smithery schema verified as matching Zod exactly at parameter level
- Remaining gaps: None substantive

## Novelty Assessment
Novelty: NITPICK
The Smithery schema comparison confirmed no parameter drift (positive finding but not model-changing). The cli.js error handling correction is a minor refinement. The comment style and TypeScript convention cataloging are completeness additions. The ES2022 target analysis (features available but unused) is informational only. None of these findings change how you would spec the system.

## Convergence Declaration
Pass 5 has converged -- findings are nitpicks, not gaps. The convention catalog is complete with 7 conventions, 6 patterns, and 5 anti-patterns, all with consistency assessments and line-level references.

## State Checkpoint
```yaml
pass: 5
round: 2
status: complete
timestamp: 2026-04-14T00:05:00Z
novelty: NITPICK
```
