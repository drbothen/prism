# Pass 5 Deep: Conventions & Pattern Catalog -- mcp-claroty-xdome (Round 1)

## Overview

This deepening round extends the broad sweep's convention catalog with detailed analysis of code patterns, consistency assessment, test conventions, import patterns, documentation conventions, and AI-assisted development conventions discovered in the `.windsurf/` configuration.

---

## 1. Design Patterns (Refined and Expanded)

### DP-001: Template Method (BaseToolHandler)
- **Location:** `src/core/base-tool-handler.ts`
- **Implementation:** Abstract class with 4 abstract readonly properties (`name`, `title`, `description`, `inputSchema`) and 1 abstract method (`handle`)
- **Generic constraint:** `BaseToolHandler<T extends z.ZodType>` -- handler is generic over its Zod schema type
- **Consistency:** 5/5 tool handlers implement this consistently
- **Variation:** None -- all handlers follow identical structure

### DP-002: Dependency Injection (tsyringe)
- **Decorators used:** `@injectable()`, `@singleton()` (via `container.registerSingleton()`), `@inject(token)`
- **Token types:** Class references (auto-resolved), string tokens ("ExpressApp", "Logger", "SessionManager", "CacheManager"), value tokens
- **Registration modes:** `useValue` (primitives), `useClass` (creates new instance per resolution), `useToken` (alias to existing registration), `registerSingleton` (resolved once, cached)
- **Critical pattern:** `CacheManager` uses `useClass` for per-injection isolation; `SessionManager` uses `useToken` for alias
- **Consistency:** All domain services and tool handlers are injectable singletons

### DP-003: Factory Pattern
- **Location:** `src/server/factory.ts` -- `createAndInitializeServer()`
- **Type:** Pure function (not a class), returns `CoreMcpServer`
- **Responsibilities:** Environment validation, DI container setup, transport registration, tool registration, server initialization
- **Note:** This is a factory function, not a Factory class -- there is no abstract factory or factory method pattern

### DP-004: Registry Pattern
- **Location:** `src/core/tool-registry.ts`
- **Implementation:** `Map<string, AnyToolHandler>` with register/get/getAll/unregisterAll
- **Key behavior:** Duplicate name overwrites with console.warn (not throw)
- **Type narrowing:** `AnyToolHandler = BaseToolHandler<z.ZodObject<ZodRawShape>>`

### DP-005: Strategy Pattern (Transports)
- **Interface:** `SelfDescribingTransport extends Transport`
- **Implementations:** ReusableExpressTransport, SseTransport, StreamableHttpTransport
- **Selection:** Environment variable `MCP_TRANSPORT_TYPE` (comma-separated)
- **Default:** All 3 transports enabled
- **Self-describing:** Each transport declares its own routes via `getRegistrationDetails()`

### DP-006: Facade/Orchestrator Pattern (SessionOrchestrator)
- **Location:** `src/core/session-orchestrator.ts`
- **Bridges:** SessionManager (persistent data) and ConnectionManager (live connections)
- **Purpose:** Single entry point for transport middleware to resolve/create sessions

### DP-007: Connection Pattern (NEW)
- **Interface:** `StatefulConnection` with `send()` and `close()`
- **Implementations:** HttpConnection (ephemeral), SseConnection (persistent), StreamableHttpConnection (hybrid)
- **State machine:** StreamableHttpConnection transitions from HTTP mode to SSE mode after first response

### DP-008: Proxy Pattern (appRouter) (NEW)
- **Location:** `src/core/app-router.ts`
- **Implementation:** `express.Router()` singleton export
- **Purpose:** Decouples route registration from Express app instance, preventing middleware ordering conflicts
- **Used by:** TransportManager registers routes on appRouter; CoreMcpServer mounts appRouter on Express app

---

## 2. Naming Conventions (Comprehensive)

### File Naming

| Convention | Pattern | Examples | Consistency |
|------------|---------|----------|-------------|
| Source files | kebab-case | `alert-service.ts`, `get-alerts-handler.ts` | 100% in src/ |
| Test files | `<source-name>.test.ts` | `alert-service.test.ts` | 100% |
| Schema files | `get-<entity>-schema.ts` | `get-alerts-schema.ts` | 100% |
| Type files | singular noun, kebab-case | `claroty.ts`, `mcp.ts` | 100% |
| Generated files | kebab-case | `version.ts` (in src/generated/) | 100% |
| Config files | dot-case or kebab-case | `.audit-ci.jsonc`, `tsconfig.json` | 100% |

### Class Naming

| Convention | Pattern | Examples | Consistency |
|------------|---------|----------|-------------|
| Services | `<Entity>Service` | `AlertService`, `DeviceService` | 100% |
| Tool handlers | `Get<Entity>ToolHandler` | `GetAlertsToolHandler` | 100% |
| Transports | `<Type>Transport` | `SseTransport`, `StreamableHttpTransport` | 100% |
| Connections | `<Type>Connection` | `HttpConnection`, `SseConnection` | 100% |
| Managers | `<Noun>Manager` | `ConnectionManager`, `TransportManager` | 100% |
| Errors | `<Noun>Error` | `ValidationError`, `IntegrationError` | 100% |

### Tool Naming (MCP tool names)

| Convention | Pattern | Examples |
|------------|---------|----------|
| Tool names | snake_case with get_ prefix | `get_alerts`, `get_devices`, `get_vulnerability_devices` |
| All read-only | get_ prefix only | No set_, create_, update_, delete_ tools |

### Variable/Function Naming

| Convention | Pattern | Examples |
|------------|---------|----------|
| Local variables | camelCase | `baseUrl`, `apiToken`, `healthData` |
| Constants | SCREAMING_SNAKE_CASE | `SECONDS_IN_AN_HOUR`, `BYTES_TO_MB`, `APP_VERSION` |
| Private fields | No prefix convention | `this.cache`, `this.logger`, `this.isRunning` |
| Boolean fields | `is` prefix | `isRunning`, `is_resolved`, `is_known_exploited` |

---

## 3. Import Conventions

### Import Ordering (observed, not enforced by linter)
```typescript
1. Node.js built-in imports (rare -- only http in verify-server.ts)
2. Third-party library imports (express, axios, zod, tsyringe, winston)
3. Project-relative imports using .js extension (ESM requirement)
4. Type-only imports (when applicable)
```

### ESM Extension Convention
All internal imports use `.js` extension despite being TypeScript files:
```typescript
import { AlertService } from "../domain/alerts/alert-service.js";
```
This is required by Node.js ESM resolution with `moduleResolution: "NodeNext"`.

### Path Alias Convention
- **Defined:** `@/*` -> `src/*` in tsconfig.json
- **Used:** Only in `mcp-server-instance.ts:5` -- `import { createLogger } from "@/utils/logger.js"`
- **Consistency:** VERY LOW -- only 1 file uses the alias; all others use relative paths
- **Assessment:** The path alias exists but was not adopted consistently. This is a **partially adopted convention**.

---

## 4. Error Handling Conventions

### Error Class Hierarchy Convention
Every error class follows this pattern:
```typescript
export class <Name>Error extends <Parent>Error {
  constructor(<specific params>) {
    super(<message>, <McpErrorCodes.XYZ>, <optional data>);
  }
}
```

### Error Code Allocation Convention
- **Standard JSON-RPC:** -32700 to -32603 (5 codes)
- **Application-specific:** -32000 to -32009 (10 codes)
- **Unused code:** -32006 (ValidationError code) is defined but never used -- `ValidationError` class uses `-32602` (JSON-RPC standard InvalidParams)

### Error Construction Patterns

| Error Class | Constructor Signature | Auto-Generated Message |
|------------|----------------------|------------------------|
| ValidationError | `(message, data?)` | Caller provides |
| AuthenticationError | `(message? = "Authentication failed")` | Default provided |
| AuthorizationError | `(message? = "Not authorized...")` | Default provided |
| NotFoundError | `(resource, id?, idName? = "id")` | `"{resource} with {idName} '{id}' not found."` |
| ToolNotFoundError | `(toolName)` | Delegates to NotFoundError("Tool", toolName, "name") |
| ConflictError | `(message, data?)` | Caller provides |
| TooManyRequestsError | `(message? = "Rate limit exceeded")` | Default provided |
| ServerError | `(message, code? = ServerError, data?)` | Caller provides |
| DatabaseError | `(message, data?)` | Caller provides |
| IntegrationError | `(service, data?)` | `"Error communicating with {service}"` |

### Error Propagation Convention
- **Integration layer:** Catches and maps (the ONLY layer that catches)
- **Domain layer:** Transparent passthrough (no try/catch)
- **Tool layer:** Transparent passthrough (no try/catch)
- **MCP SDK:** Catches all errors from tool handlers for JSON-RPC serialization

---

## 5. Test Conventions

### Test File Organization
```
tests/
  setup.ts                    -- Global setup (reflect-metadata + dotenv)
  unit/
    core/                     -- Mirror of src/core/
    domain/alerts/            -- Mirror of src/domain/alerts/
    domain/devices/           -- Mirror of src/domain/devices/
    domain/vulnerabilities/   -- Mirror of src/domain/vulnerabilities/
    integrations/claroty/     -- Mirror of src/integrations/claroty/
    tools/                    -- Mirror of src/tools/
    utils/                    -- Mirror of src/utils/
    main.test.ts              -- Entry point tests
    server/factory.test.ts    -- Factory tests
    server/index.test.ts      -- Server index tests
  e2e/
    *.integration.test.ts     -- Integration tests (with .integration suffix)
    *.e2e.test.ts.disabled    -- Disabled e2e test
```

**Convention:** Test directory mirrors source directory structure 1:1.

### Test Naming Convention
- Unit tests: `<source-name>.test.ts`
- Integration tests: `<feature>.integration.test.ts`
- E2E tests: `<feature>.e2e.test.ts`
- Disabled tests: Append `.disabled` extension

### Mocking Convention
```typescript
// Module-level vi.mock with factory
vi.mock("../../src/path/to/module", () => ({
  ClassName: vi.fn().mockImplementation(() => ({
    methodName: vi.fn(),
  })),
}));

// Per-test mock configuration in beforeEach/test
mockMethod.mockResolvedValue(expectedResult);
mockMethod.mockRejectedValue(new Error("test error"));
```

### Test Structure Convention (AAA)
```typescript
describe("ComponentName", () => {
  // Setup
  let component: ComponentType;
  let mockDep: MockType;

  beforeEach(() => {
    mockDep = { method: vi.fn() };
    component = new Component(mockDep);
  });

  it("should <expected behavior>", async () => {
    // Arrange
    mockDep.method.mockResolvedValue(expected);
    
    // Act
    const result = await component.doSomething(input);
    
    // Assert
    expect(result).toEqual(expected);
    expect(mockDep.method).toHaveBeenCalledWith(expectedArgs);
  });
});
```

### E2E Test Convention
- Set mock env vars at module level: `process.env.CLAROTY_XDOME_BASE_URL = "https://mock..."`
- Use `container.clearInstances()` in `afterEach` for DI cleanup
- Use specific port (3001) to avoid conflicts with dev server (3000)
- Use native `fetch()` for HTTP assertions (not supertest for some tests)

### Test Setup Convention
- `tests/setup.ts` loaded via `vite.config.ts:setupFiles`
- Imports `reflect-metadata` (required for tsyringe decorators in tests)
- Loads dotenv (for env var availability in tests)

---

## 6. Documentation Conventions

### JSDoc Convention
- **Used on:** Class declarations, public methods, interfaces, type aliases
- **Format:** Multi-line `/** ... */` with `@param`, `@returns`, `@see`, `@template`
- **Consistency:** HIGH in core/ and utils/, MEDIUM in domain/ and tools/
- **Example file annotation:** Some files have `@file` JSDoc at top (`types/mcp.ts`, `utils/errors.ts`)

### Inline Comment Convention
- Implementation comments explain "why" not "what"
- Critical comments marked with: "This is critical" or "This re-throw is critical"
- SDK workaround comments explain the necessity and risk
- All eslint-disable comments include reason via adjacent comment

### README/Guide Convention
- README.md: Usage-focused with badges, quick start, configuration
- USAGE.md: Detailed usage examples
- CONTRIBUTING.md: Development workflow, testing, PR guidelines
- COMPLIANCE.md: License and compliance information
- docs/guides/TOOL_REGISTRATION.md: Canonical step-by-step guide for adding new tools

---

## 7. Code Organization Conventions

### Module Structure Convention
```
src/
  core/          -- Framework-level infrastructure (transport-agnostic)
  domain/        -- Business logic organized by aggregate root
    <aggregate>/
      <entity>-service.ts
  integrations/  -- External system clients
    <provider>/
      <provider>-api-client.ts
  schemas/       -- Input validation (one per tool)
  tools/         -- MCP tool handlers (one per tool)
  types/         -- Shared TypeScript interfaces
  utils/         -- Cross-cutting utilities
  server/        -- Application assembly (factory)
  generated/     -- Auto-generated files (gitignored)
```

### Domain Service Convention
Every domain service follows this identical template:
```typescript
@injectable()
export class <Entity>Service {
  constructor(
    @inject(XDomeApiClient) private readonly apiClient: XDomeApiClient,
    @inject("CacheManager") private readonly cacheManager: CacheManager<ResponseType>,
    @inject("Logger") private readonly logger: Logger,
  ) {}

  public async find<Entity>(params: InputType): Promise<ResponseType> {
    const cacheKey = JSON.stringify(params);
    const cached = this.cacheManager.get(cacheKey);
    if (cached) {
      this.logger.debug("Cache hit", { cacheKey });
      return cached;
    }
    const response = await this.apiClient.<method>(params);
    this.cacheManager.set(cacheKey, response);
    return response;
  }
}
```

**Consistency:** 5/5 domain services follow this exact pattern. The only variation is the parameter decomposition in junction-entity services (alerted-device, vulnerability-device) which extract the parent entity ID before calling the API client.

### Tool Handler Convention
Every tool handler follows this identical template:
```typescript
@injectable()
export class Get<Entity>ToolHandler extends BaseToolHandler<typeof InputSchema> {
  readonly name = "get_<entity>";
  readonly title = "Get <Entity>";
  readonly description = "...";
  readonly inputSchema = InputSchema;

  constructor(
    @inject(EntityService) private readonly service: EntityService,
  ) { super(); }

  async handle(args: z.infer<typeof InputSchema>): Promise<ToolResult> {
    const response = await this.service.find<Entity>(args);
    return { content: [{ type: "text", text: JSON.stringify(response) }] };
  }
}
```

**Consistency:** 5/5 tool handlers follow this exact pattern.

---

## 8. Anti-Patterns and Inconsistencies (Expanded)

### AP-001: Duplicate VulnerabilityService Classes
- **Files:** `domain/alerts/vulnerability-service.ts` and `domain/vulnerabilities/vulnerability-service.ts`
- **Import aliasing:** Factory uses `VulnerabilityService as VulnerabilitiesService`
- **Risk:** Confusing for new developers; naming collision requires manual aliasing

### AP-002: Unbounded In-Memory Caches
- **5 independent caches** with no max entry limit
- **Cache cleanup() exists** but is never called automatically -- requires explicit invocation
- **Risk:** Memory leak under diverse query patterns

### AP-003: No Session Expiration
- **InMemorySessionManager** stores sessions in `Map<string, SessionData>` with no eviction
- **`lastAccessTimestamp` is tracked** via `touchSession()` but never used for expiration
- **Risk:** Unbounded session accumulation

### AP-004: CORS Wildcard
- **`origin: "*"`** applied unconditionally
- **Not environment-gated** -- same policy in production and development
- **Mitigated by:** MCP endpoints have no authentication anyway, so CORS is not the primary security concern

### AP-005: Schema Field Duplication
- **~180 device fields** duplicated across 3 schema files (devices, alerted-devices, vulnerability-devices)
- **No shared base enum** extracted
- **Risk:** Field additions require updating 3 files

### AP-006: SDK Internal Access
- **`(this.mcpServer as any).setToolRequestHandlers()`** in mcp-server-instance.ts
- **Documented as workaround** with eslint-disable comment
- **Risk:** SDK upgrade breakage

### AP-007: Path Alias Inconsistency (NEW)
- **`@/*` path alias** defined in tsconfig.json but only used in 1 file out of 37
- **All other files** use relative imports (`../`, `../../`)
- **Assessment:** Convention was defined but not adopted

### AP-008: Nodemon Configuration Unused (NEW)
- **`nodemon.json`** configured with `node --loader ts-node/esm src/server.ts`
- **But dev script** uses `tsx watch src/main.ts` (different tool, different entry point)
- **Assessment:** Vestigial configuration from earlier development phase

### AP-009: tsconfig.jest.json Unused (NEW)
- **`tsconfig.jest.json`** exists but project uses Vitest, not Jest
- **Assessment:** Vestigial from migration to Vitest

### AP-010: Zod Version Pinning (NEW)
- **Zod pinned to exact version:** `"zod": "3.25.67"` (no caret/tilde)
- **All other deps use ranges:** `"express": "^4.19.2"`
- **Assessment:** Intentional pinning, possibly due to breaking changes in Zod minor versions

---

## 9. Git/Development Workflow Conventions

### Branching Model
- **GitFlow:** main, develop, release/*, hotfix/*, bugfix/* branches
- **Automation:** 4 dedicated GitHub Actions workflows for GitFlow automation
- **Source:** `.github/workflows/gitflow-*.yml`

### Commit Convention
- **Conventional Commits** referenced in `.windsurf/rules/36-conventional-commits.md`
- **Observable in git log:** `chore: initialize prism repo` format

### Pre-commit Hooks
- **Husky 9.1.7** listed as dev dependency
- **Purpose:** Likely runs lint/format checks before commit (hooks not inspected)

---

## 10. AI-Assisted Development Convention

The `.windsurf/` directory reveals a structured AI development methodology:

### Rules (coding standards for AI agent):
- Core principles, tool usage guidelines, advanced tooling
- AI workflow with examples
- Documentation review standards
- Governance and quality gates
- Coding standards and security/performance rules
- Implementation guidelines and error handling
- Conventional commits and formatting standards

### Workflows (reusable AI task flows):
- Architecture enhancement, consistency review, fixes
- Testability recommendations and application
- API coverage analysis, missing API discovery
- Documentation generation, deployment

### Prompts (task-specific instructions):
- Implementation planners (phase, task, step level)
- API coverage analyzers and validators
- MCP tool builders and server requirements
- Workflow analyzers and mappers

**Assessment:** This is one of the most thoroughly AI-configured codebases I have encountered. The `.windsurf/` directory contains 68 files of structured AI development guidance, suggesting the codebase was developed primarily through AI-assisted pair programming with explicit quality gates.

---

## Delta Summary
- New items added: 2 new design patterns (Connection, Proxy/appRouter); path alias convention (with inconsistency finding); import ordering convention; ESM extension convention; test file organization mirror convention; test naming convention for unit/integration/e2e/disabled; mocking convention; domain service template convention; tool handler template convention; JSDoc convention; 4 new anti-patterns (AP-007 through AP-010); Git workflow convention; AI-assisted development convention; Zod version pinning observation
- Existing items refined: All 6 original design patterns now have precise consistency ratings; error handling convention now includes complete constructor signature catalog; test patterns now include AAA template and e2e setup conventions
- Remaining gaps: Husky pre-commit hook configuration (not inspected); Prettier configuration (referenced in ESLint but .prettierrc not analyzed); complete analysis of `.windsurf/rules/13-coding-standards.md` for additional convention intent

## Novelty Assessment
Novelty: SUBSTANTIVE
The discovery of the AI-assisted development methodology (68 files of structured AI guidance), the domain service/tool handler template conventions (showing 100% mechanical consistency), the 4 new anti-patterns (path alias inconsistency, vestigial configs), and the detailed import/naming conventions all change how you would spec coding standards for this system. The broad sweep presented high-level patterns; this round reveals the precise template-based development approach and its inconsistencies.

## Convergence Declaration
Another round needed -- the following substantive gaps remain: (1) Prettier configuration, (2) husky hook configuration, (3) `.windsurf/rules/13-coding-standards.md` analysis for additional conventions.

## State Checkpoint
```yaml
pass: 5
round: 1
status: complete
timestamp: 2026-04-14T00:15:00Z
novelty: SUBSTANTIVE
```
