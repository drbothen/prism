# Pass 1 Deep: Architecture -- serveMyAPI (Round 1)

## Preamble

This deepening round re-examines the architecture with precision, informed by Phase A findings (domain model, behavioral contracts) and new Pass 0 discoveries (ecosystem role, dead code, vision document). The broad sweep correctly identified the 2-layer architecture and transport variants. This round looks for missed architectural patterns, deployment topology details, and cross-cutting concerns.

---

## Architectural Reassessment

### 1. System Role in Ecosystem (New Discovery)

The broad sweep characterized serveMyAPI as a standalone credential management tool. Pass 0 deepening revealed from CLAUDE.md that it is actually the **foundational credential provider** for an ecosystem of 20+ MCP servers. Multiple MCP server configurations reference serveMyAPI's keys via patterns like:

```
"BRAVE_API_KEY": "${await serveMyAPI.getKey('brave_search')}"
"GOOGLE_API_KEY": "${await serveMyAPI.getKey('google_search')}"
"LEONARDO_API_KEY": "${await serveMyAPI.getKey('cMax Leonardo API')}"
```

This means serveMyAPI must start **before** any dependent MCP server, and its availability is a **system-wide dependency**. If serveMyAPI fails, no other MCP server that requires API keys can function.

**Architectural implication for Prism**: The credential service is not optional infrastructure -- it is a critical path dependency. Prism must guarantee high availability and fast startup for the credential subsystem.

### 2. Revised Component Catalog

```
+========================================================================+
|                         ServeMyAPI System                               |
|                    (Credential Infrastructure Service)                   |
+========================================================================+
|                                                                          |
|  TRANSPORT LAYER (4 entry points, 1 dead)                               |
|  +------------------+  +------------------+  +-----------------+        |
|  | index.ts         |  | server.ts        |  | cli.ts          |        |
|  | Stdio MCP Server |  | HTTP/SSE MCP Svr |  | Direct CLI      |        |
|  | [FUNCTIONAL]     |  | [PARTIAL-broken  |  | [FUNCTIONAL]    |        |
|  |                  |  |  SSE sessions]   |  |                 |        |
|  +--------+---------+  +--------+---------+  +--------+--------+        |
|           |                      |                     |                 |
|           |   Tool handlers      |                     |                 |
|           |   (copy-pasted x2)   |                     |                 |
|           +----------+-----------+---------------------+                 |
|                      |                                                   |
|  SERVICE LAYER       v                                                   |
|           +----------+----------+                                        |
|           | KeychainService     |                                        |
|           | (singleton default) |                                        |
|           +----------+----------+                                        |
|                      |                                                   |
|  STORAGE LAYER       |                                                   |
|           +----------+----------+                                        |
|           |   Storage Backend   |                                        |
|           | (selected by env)   |                                        |
|           +-----+----------+---+                                        |
|                 |          |                                              |
|           +-----+--+  +---+--------+                                    |
|           | keytar  |  | File-based |                                    |
|           | (native)|  | (Docker)   |                                    |
|           +--------+  +------------+                                    |
|                                                                          |
|  DEAD CODE                                                               |
|  +------------------+                                                    |
|  | cli.js           |                                                    |
|  | MCP Client CLI   |                                                    |
|  | [NON-FUNCTIONAL] |                                                    |
|  | [UNREACHABLE     |                                                    |
|  |  after build]    |                                                    |
|  +------------------+                                                    |
+========================================================================+
```

### 3. Layer Analysis Refinement

The broad sweep identified 2 layers. More precisely, there are **3 logical layers** even though the code does not enforce boundaries:

| Layer | Responsibility | Files | Boundary Enforcement |
|-------|---------------|-------|---------------------|
| Transport | Protocol handling, schema validation, error formatting | index.ts, server.ts, cli.ts | None -- tool handlers inline |
| Service | Business logic (CRUD + permission marker) | services/keychain.ts | Module boundary (import) |
| Storage | OS keyring / filesystem operations | services/keychain.ts (inline) | None -- storage logic is in the service |

The Storage layer is **embedded within** the Service layer. There is no abstraction boundary -- `KeychainService` directly calls both `keytar` and `fs` methods inline, selected by `IS_DOCKER` conditionals. This violates the Dependency Inversion Principle.

### 4. Deployment Topology (Refined)

The system has **four** deployment modes, not three as implied by the broad sweep:

| Mode | Entry Point | Transport | Storage | Status |
|------|-------------|-----------|---------|--------|
| MCP stdio (native) | `dist/index.js` | Stdio | keytar (OS keyring) | Primary, functional |
| MCP HTTP/SSE | `dist/server.js` | Express SSE | keytar (OS keyring) | Secondary, broken for concurrent users |
| Docker container | `dist/index.js` | Stdio | File-based (plaintext) | Tertiary, but HEALTHCHECK is misconfigured |
| Smithery hosted | `dist/index.js` | Stdio (via Smithery runtime) | Unknown (likely file-based) | Quaternary, hosted service |
| macOS DMG app | `Resources/main.js` (broken) | Stdio | keytar (OS keyring) | **Non-functional** (wrong file reference) |

**Key finding**: Three of five deployment modes have known bugs:
1. HTTP/SSE: SSE session management broken for concurrent users
2. Docker: HEALTHCHECK contradicts CMD (stdio vs HTTP)
3. DMG: Launcher script references non-existent `main.js`

Only the stdio MCP and direct CLI modes are confirmed functional.

### 5. Cross-Cutting Concerns (Refined)

| Concern | Implementation | Quality | Phase A Findings |
|---------|---------------|---------|-----------------|
| Error handling | try/catch per tool + per service method | Adequate but **asymmetric** -- keytar propagates errors, file backend swallows them (OBS-3.03) | File backend errors indistinguishable from "not found" |
| Logging | `console.error` only | MINIMAL | No structured logging, no log levels |
| Auth/Security | OS keyring ACL only | OS-level only | HTTP/SSE has zero auth; plaintext key return by design |
| Input validation | Zod `z.string().min(1)` at transport | Basic | CLI bypasses Zod entirely (OBS-3.02); no path sanitization for Docker file names |
| Graceful shutdown | None | MISSING | No SIGTERM/SIGINT handlers in any entry point |
| Health check | Docker HEALTHCHECK only | BROKEN | Checks HTTP on stdio server |
| Process lifecycle | Fire-and-forget async in constructor | FRAGILE | Permission marker race window (BC-2.03.001) |
| Code organization | Tool definitions duplicated across files | POOR | 3 copies: index.ts, server.ts, smithery.yaml |
| Module system | ES modules throughout | CONSISTENT | `"type": "module"` in package.json |

### 6. Data Flow Diagrams

#### Stdio MCP Flow (Primary)
```
MCP Client (Claude Desktop)
    |
    | stdin: JSON-RPC tool call
    v
index.ts: StdioServerTransport
    |
    | Zod schema validation
    v
index.ts: Tool handler (try/catch)
    |
    | Direct method call
    v
KeychainService (singleton)
    |
    | IS_DOCKER check
    +------[false]------+------[true]------+
    |                                      |
    v                                      v
keytar.setPassword()              fs.writeFileSync()
keytar.getPassword()              fs.readFileSync()
keytar.deletePassword()           fs.unlinkSync()
keytar.findCredentials()          fs.readdirSync()
    |                                      |
    +--------------------------------------+
    |
    v
Return to tool handler
    |
    | Format MCP response
    v
StdioServerTransport
    |
    | stdout: JSON-RPC response
    v
MCP Client
```

#### HTTP/SSE Flow (Secondary)
```
Browser / HTTP Client
    |
    | GET /sse
    v
Express: SSE endpoint
    |
    | Create SSEServerTransport
    | Store in activeTransports map
    v
server.ts: McpServer.connect(transport)
    |
    | ... client sends tool call ...
    |
    | POST /messages
    v
Express: Messages endpoint
    |
    | Get LAST transport from map (BUG: no session correlation)
    |
    v
SSEServerTransport.handlePostMessage()
    |
    | Same tool handlers as stdio
    v
KeychainService -> keytar/fs -> response via SSE
```

### 7. Architecture vs. Vision Document Gap Analysis

The PDF "How to improve serveMyAPI" proposes a 2.0 architecture that would fundamentally change the system:

| Current (1.0) | Proposed (2.0) | Gap |
|---------------|----------------|-----|
| Raw key return via `get-api-key` | Pre-signed URL generation (`generateSignedUrl`) | Major -- new operation type |
| No app layer | Native app with UI + request router | Major -- new component |
| No browser integration | Cross-browser extension with pattern detection | Major -- new subsystem |
| Single service namespace | Multi-service key vault (`api-key-${service}`) | Moderate -- namespace change |
| 2 layers | 3 layers (OS Keychain -> App -> Client integrations) | Moderate -- new layer |
| keytar direct access | `SecureStorage` abstraction with platform detection | Moderate -- proper abstraction |

**Relevance to Prism**: The 2.0 vision aligns well with Prism's goals. The `CredentialStore` trait recommended in the broad sweep maps directly to the `SecureStorage` concept. The pre-signed URL pattern is worth evaluating for Prism's MCP tool design.

### 8. Architectural Anti-Patterns Catalog

| # | Anti-Pattern | Location | Impact | Severity |
|---|-------------|----------|--------|----------|
| AP-1 | Copy-paste tool definitions | index.ts, server.ts, smithery.yaml | Triple maintenance burden | HIGH |
| AP-2 | Inline strategy selection | keychain.ts (IS_DOCKER checks) | No testability, no extensibility | MEDIUM |
| AP-3 | Singleton with async constructor side effect | keychain.ts:18-25 | Race condition window | LOW (benign) |
| AP-4 | Dead code in source tree | src/cli.js | Confusion, maintenance cost | LOW |
| AP-5 | Dependencies in wrong category | package.json | Bloated production installs | LOW |
| AP-6 | No shared module for tool definitions | (missing) | Forces duplication | HIGH |
| AP-7 | Session management via Date.now() | server.ts:160 | Concurrent user collision | HIGH (for SSE mode) |
| AP-8 | Healthcheck protocol mismatch | Dockerfile | Container always unhealthy | MEDIUM |
| AP-9 | Unused import | index.ts:5 (keytar) | Dead code | LOW |
| AP-10 | No graceful shutdown | All entry points | Resource leaks, data loss risk | MEDIUM |

---

## Delta Summary
- New items added: Ecosystem role analysis (serveMyAPI as credential backbone), 5th deployment mode (Smithery), data flow diagrams (2), architecture vs. 2.0 vision gap analysis, 10 cataloged anti-patterns with severity ratings
- Existing items refined: 3-layer (not 2-layer) architecture with embedded storage layer, deployment topology expanded from 3 to 5 modes with 3 broken, cross-cutting concerns updated with Phase A findings, component catalog updated with dead code annotation
- Remaining gaps: Smithery runtime behavior (how it wraps stdio servers), whether Express 5 routing differences affect the SSE transport, exact `mcp-server-template.md` architectural patterns

## Novelty Assessment
Novelty: SUBSTANTIVE
The ecosystem role discovery (serveMyAPI as credential backbone for 20+ MCP servers) fundamentally changes the architectural significance of this service -- it is not a standalone utility but critical infrastructure. The 2.0 vision document gap analysis surfaces design directions Prism should consider. The deployment mode audit revealing 3/5 modes are broken is a significant architectural quality finding. The anti-pattern catalog with severity ratings provides actionable input for Prism's design.

## Convergence Declaration
Another round needed -- want to verify Smithery deployment behavior and check whether the `mcp-server-template.md` contains architectural patterns the codebase should have followed.

## State Checkpoint
```yaml
pass: 1
round: 1
status: complete
timestamp: 2026-04-13T23:35:00Z
novelty: SUBSTANTIVE
```
