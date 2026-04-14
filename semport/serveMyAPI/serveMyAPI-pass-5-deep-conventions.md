# Pass 5 Deep: Conventions & Patterns -- serveMyAPI (Round 1)

## Preamble

This deepening round catalogs coding conventions, design patterns, and anti-patterns with line-level precision. It cross-references Phase A findings and Pass 0/1 discoveries to assess consistency across the entire codebase.

---

## Coding Conventions (Refined)

### CONV-001: Indentation and Formatting

- **2-space indentation**: CONSISTENT across all TS/JS files
- **Trailing newline**: Present in all source files
- **Semicolons**: Always used (no ASI reliance)
- **String literals**: Backtick template literals for interpolation, single quotes for static strings
- **Brace style**: K&R (opening brace on same line)
- **No eslint config file in repo**: Despite `npm run lint` script referencing ESLint, there is no `.eslintrc`, `eslint.config.js`, or similar. The lint command would use ESLint defaults + TypeScript plugin.

### CONV-002: Naming Conventions

| Context | Convention | Examples | Consistency |
|---------|-----------|----------|-------------|
| Local variables | camelCase | `keychainService`, `storeValue`, `lastTransportId` | CONSISTENT |
| Function names | camelCase | `storeKey`, `checkPermissionMarker`, `printUsage` | CONSISTENT |
| Class names | PascalCase | `KeychainService` | CONSISTENT (only 1 class) |
| Constants (module-level) | UPPER_SNAKE | `SERVICE_NAME`, `PERMISSION_MARKER`, `STORAGE_DIR`, `IS_DOCKER` | CONSISTENT |
| MCP tool names | kebab-case | `store-api-key`, `get-api-key` | CONSISTENT |
| CLI commands | lowercase single word | `list`, `get`, `store`, `delete` | CONSISTENT within each CLI, but aliases differ across CLIs |
| File names | camelCase for TS, camelCase for JS | `keychain.ts`, `cli.ts`, `cli.js` | CONSISTENT |
| Directory names | lowercase | `src`, `services`, `examples`, `dist` | CONSISTENT |

### CONV-003: Import Organization

All files follow the same import order (not enforced by linter):

1. Third-party packages first
2. Local modules second
3. No blank lines between import groups (inconsistent with best practices)

Example from index.ts:
```typescript
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import keychainService from "./services/keychain.js";
import keytar from 'keytar';  // UNUSED
```

Note: Import paths use `.js` extension even for TypeScript files (required by NodeNext module resolution).

### CONV-004: JSDoc Documentation

| Location | JSDoc Present | Quality |
|----------|--------------|---------|
| KeychainService public methods | Yes (all 4 CRUD methods) | Good -- includes @param and @returns |
| KeychainService private methods | Yes (ensureStorageDirectory, checkPermissionMarker, all file methods) | Good |
| KeychainService class | Yes (class-level comment) | Brief but adequate |
| Tool handlers (index.ts/server.ts) | No (inline comments only) | Inline `// Tool to store/retrieve/delete/list` comments |
| CLI functions (cli.ts) | No | None |
| CLI (cli.js) | Yes (top-level usage comment) | Brief |
| Module-level constants | No | None |

**Assessment:** JSDoc is CONSISTENT within the service layer but ABSENT in the transport layer and CLI.

### CONV-005: Error Handling Pattern

Two distinct patterns are used:

**Pattern A: Tool handler error formatting (transport layer)**
```typescript
try {
  // service call
  return { content: [{ type: "text", text: successMsg }] };
} catch (error) {
  return { content: [{ type: "text", text: `Error: ${(error as Error).message}` }], isError: true };
}
```
Applied in: index.ts (4 instances), server.ts (4 instances) -- 8 total, all identical.

**Pattern B: Service layer error handling**
```typescript
try {
  // operation
} catch (error) {
  console.error('Error description:', error);
  // varies: throw error | return null | return false | return []
}
```
Applied in: keychain.ts (7 instances) -- with inconsistent recovery behavior per Phase A OBS-3.03.

**Type assertion risk:** `(error as Error).message` is unsafe -- if a non-Error value is thrown (e.g., a string), `.message` will be `undefined`. The CLI files handle this better: `error instanceof Error ? error.message : String(error)`.

### CONV-006: Async/Await Usage

- All async operations use async/await (no raw Promise chains)
- EXCEPTION: Constructor calls `this.checkPermissionMarker()` without await (fire-and-forget)
- File backend methods are declared `async` but use synchronous `fs.*Sync` operations -- the `async` keyword is vestigial (methods could be synchronous)

### CONV-007: Module Export Patterns

| File | Export Style | Pattern |
|------|-------------|---------|
| keychain.ts | Named class + default instance | `export class KeychainService { ... } export default new KeychainService();` |
| server.ts | Named export at bottom | `export { server };` |
| index.ts | No exports | Module side-effects only |
| cli.ts | No exports | Module side-effects only |
| cli.js | No exports | Module side-effects only |

The singleton pattern (`export default new KeychainService()`) is the only reuse mechanism.

---

## Design Patterns Catalog

### PAT-001: Singleton Service

**Location:** keychain.ts:198
**Implementation:** `export default new KeychainService();`
**Consumers:** index.ts:4, server.ts:5, cli.ts:3

The singleton is created at module load time. Since ES modules are cached by the runtime, all consumers share the same instance. This guarantees shared state (the `hasStoredPermissionMarker` flag).

**Quality:** Adequate for this use case. The singleton is simple and the shared state is intentional.

### PAT-002: Conditional Strategy (not proper Strategy Pattern)

**Location:** keychain.ts -- every public method
**Implementation:**
```typescript
async storeKey(name: string, key: string): Promise<void> {
  if (IS_DOCKER) {
    return this.storeKeyFile(name, key);
  }
  // ... keytar path
}
```

Every CRUD method has the same `if (IS_DOCKER) return this.xxxFile(...)` guard at the top. This is an **inline conditional**, not a Strategy pattern. A proper Strategy would use a common interface:

```typescript
interface StorageBackend {
  store(name: string, key: string): Promise<void>;
  get(name: string): Promise<string | null>;
  delete(name: string): Promise<boolean>;
  list(): Promise<string[]>;
}
```

**Consistency:** CONSISTENT -- all 4 methods use the same pattern.

### PAT-003: MCP Tool Registration (Template Method variant)

**Location:** index.ts:14-149, server.ts:14-149
**Implementation:**
```typescript
server.tool(
  "tool-name",                          // 1. Name
  { param: z.string().min(1).describe("...") },  // 2. Schema
  async ({ param }) => {                // 3. Handler
    try {
      // 4. Service call
      // 5. Success response
    } catch (error) {
      // 6. Error response
    }
  }
);
```

All 8 tool registrations (4 per file) follow this exact 6-step template. The only variations are:
- Tool name and parameter schema
- Service method called
- Success/error message text
- get-api-key and delete-api-key have an intermediate "not found" check

**Quality:** The pattern is clear and consistent but duplicated. The lack of a shared tool module is the codebase's most significant maintenance debt.

### PAT-004: Guard Clause with Belt-and-Suspenders

**Location:** keychain.ts:75-77, 105-107, 138-140, 170-172
**Implementation:**
```typescript
if (!this.hasStoredPermissionMarker) {
  await this.checkPermissionMarker();
}
```

This guard appears at the top of every keytar-backend CRUD method (4 instances). It ensures the permission marker check has completed even if the constructor's fire-and-forget call has not resolved. This is a defensive programming pattern that compensates for the async constructor limitation.

**Consistency:** CONSISTENT -- all 4 keytar methods have this guard. File backend methods do NOT have this guard (correctly, since permission markers are keytar-only).

### PAT-005: CLI Argument Parsing (Manual)

Both CLIs use manual `process.argv` parsing:

| CLI | Parsing Method | Default Command | Help Flags |
|-----|---------------|-----------------|------------|
| cli.ts | `process.argv.slice(2)` + switch/case | None (falls to default/error) | `help`, `--help`, `-h` (in switch cases) |
| cli.js | `process.argv.slice(2)` + switch/case | `list` (via `|| 'list'`) | `--help`, `-h` (checked before main()) |

No argument parsing library (yargs, commander, etc.) is used. This keeps dependencies minimal but limits extensibility.

### PAT-006: Module-Level Constant Configuration

**Location:** keychain.ts:5-8
```typescript
const SERVICE_NAME = 'serveMyAPI';
const PERMISSION_MARKER = '_permission_granted';
const STORAGE_DIR = process.env.STORAGE_DIR || '/app/data';
const IS_DOCKER = process.env.DOCKER_ENV === 'true';
```

Configuration is a mix of hardcoded constants and environment variable reads, all evaluated once at module load. There is no config file, no validation of environment variable values, and no documentation of available env vars beyond inline code.

**Environment variables used:**

| Variable | Default | Used In | Documented? |
|----------|---------|---------|-------------|
| `DOCKER_ENV` | (falsy = native) | keychain.ts:8 | Dockerfile, README (implicitly) |
| `STORAGE_DIR` | `/app/data` | keychain.ts:7 | Dockerfile |
| `PORT` | `3000` | server.ts:154 | README |
| `NODE_ENV` | (none) | smithery.yaml:17 | Smithery config only |

---

## Anti-Pattern Catalog (Refined)

### ANTI-001: Triple Tool Definition Duplication

Tool definitions exist in THREE locations:
1. `index.ts` lines 14-149 (TypeScript + Zod schemas)
2. `server.ts` lines 14-149 (identical copy)
3. `smithery.yaml` lines 23-74 (JSON Schema format, manually maintained)

Any tool change requires updating all three. If schemas drift, clients may send invalid parameters or miss new parameters.

### ANTI-002: Mixed Sync/Async in Service Layer

The file backend methods are declared `async` (to match the keytar backend signature) but use synchronous `fs.*Sync` calls internally:
- `storeKeyFile`: `fs.writeFileSync` (keychain.ts:87)
- `getKeyFile`: `fs.existsSync` + `fs.readFileSync` (keychain.ts:117-118) -- note this method is NOT declared async, it returns `string | null` directly
- `deleteKeyFile`: `fs.existsSync` + `fs.unlinkSync` (keychain.ts:150-151) -- also NOT async, returns `boolean`
- `listKeyFiles`: `fs.readdirSync` (keychain.ts:187) -- NOT async, returns `string[]`

Actually, on closer inspection: `storeKeyFile` IS declared `async` (keychain.ts:84), but `getKeyFile`, `deleteKeyFile`, and `listKeyFiles` are NOT async -- they return synchronous values. The public methods (`getKey`, `deleteKey`, `listKeys`) are async and call these synchronous private methods, which works because returning a non-Promise from an async function wraps it in `Promise.resolve()`.

This is not technically wrong but is **misleading** -- the method signatures suggest async I/O but the implementations block the event loop.

### ANTI-003: Unsafe Error Type Assertion

Tool handlers use `(error as Error).message` (8 instances across index.ts and server.ts). If a non-Error is thrown, this produces `undefined`, resulting in messages like "Error storing API key: undefined".

The CLI files correctly handle this:
```typescript
error instanceof Error ? error.message : String(error)  // cli.ts:89, cli.ts:116
```

**Inconsistency:** Two different error handling approaches in the same codebase.

### ANTI-004: `.js` Source File in TypeScript Project

`src/cli.js` is a plain JavaScript file in a TypeScript project with `strict: true`. It:
- Does not benefit from type checking
- Has a filename collision with the compiled output of `cli.ts`
- Is unreachable after `tsc` build (TypeScript does not copy JS files without `allowJs`)
- Uses different import syntax (`import { Client } from '@modelcontextprotocol/sdk/client'` -- note no `.js` extension, which may not resolve under NodeNext)

### ANTI-005: README Documents Non-Existent npm link Workflow

README.md line 69-70:
```
npm run build
npm link
```

But `package.json` has no `prepare` or `postinstall` hook, and `npm link` with `"bin": { "api-key": "dist/cli.js" }` would expose the direct CLI (compiled from cli.ts). This is correct but the README does not explain this -- a user might expect the MCP client CLI behavior.

---

## Consistency Assessment

| Convention/Pattern | Scope | Consistency |
|-------------------|-------|-------------|
| 2-space indent | All files | FULL |
| camelCase variables | All files | FULL |
| UPPER_SNAKE constants | keychain.ts | FULL (but only 4 constants) |
| JSDoc on methods | Service layer | FULL; transport layer: NONE |
| Error handling (try/catch) | All async code | FULL structure; INCONSISTENT recovery |
| Tool registration pattern | index.ts, server.ts | FULL (identical copies) |
| Guard clause for permission | keytar CRUD methods | FULL |
| Async/await | All async code | FULL (except constructor) |
| ES module imports | All files | FULL |
| `.js` extension in imports | TS files | FULL; cli.js: INCONSISTENT |
| Error type handling | Transport vs service | INCONSISTENT (`as Error` vs `instanceof Error`) |
| CLI argument parsing | Both CLIs | STRUCTURALLY consistent, BEHAVIORALLY inconsistent (different defaults, aliases, error messages) |

---

## Pattern Adoption Matrix

| Pattern | Files Using | Files That Should | Adoption |
|---------|-----------|-------------------|----------|
| Singleton service | keychain.ts | keychain.ts | 100% |
| Conditional strategy | keychain.ts | keychain.ts | 100% (but should be proper Strategy) |
| Tool registration template | index.ts, server.ts | Should be shared module | 100% of instances, 0% factored out |
| Belt-and-suspenders guard | keychain.ts (4 methods) | keychain.ts (4 methods) | 100% |
| Manual argv parsing | cli.ts, cli.js | Both | 100% |

---

## Delta Summary
- New items added: CONV-005 (error handling pattern detail with type assertion risk), CONV-006 (async/await exceptions), CONV-007 (export patterns), PAT-004 (belt-and-suspenders), PAT-005 (CLI parsing comparison), PAT-006 (env var configuration), ANTI-003 (unsafe error type assertion), ANTI-004 (.js in TS project), ANTI-005 (README documents non-existent workflow), consistency assessment matrix, pattern adoption matrix
- Existing items refined: CONV-001 (no eslint config), CONV-002 (expanded to all naming contexts), CONV-003 (import order detail), CONV-004 (JSDoc coverage per file), ANTI-001 (expanded to triple duplication including smithery.yaml), ANTI-002 (corrected -- not all file methods are async)
- Remaining gaps: Whether the eslint TypeScript plugin config is embedded in package.json (it is not -- checked), icon.svg/icon.png conventions (asset management)

## Novelty Assessment
Novelty: SUBSTANTIVE
Multiple findings change the convention model: (1) The unsafe error type assertion pattern (`as Error` vs `instanceof Error`) is a concrete inconsistency with correctness implications. (2) The triple (not double) tool definition duplication including smithery.yaml increases the maintenance burden assessment. (3) The sync-in-async file backend correction (only `storeKeyFile` is declared async; others are synchronous) refines the understanding of the I/O blocking risk. (4) The environment variable inventory reveals undocumented configuration surface. (5) The no-eslint-config finding means the lint script is effectively useless without additional setup.

## Convergence Declaration
Another round needed -- want to audit the smithery.yaml tool schemas against the Zod schemas for any drift, and check if there are any patterns in the PDF vision document that should inform the convention catalog.

## State Checkpoint
```yaml
pass: 5
round: 1
status: complete
timestamp: 2026-04-13T23:45:00Z
novelty: SUBSTANTIVE
```
