# Pass 1 Deep: Architecture -- serveMyAPI (Round 2)

## Preamble

Hallucination audit and gap closure round. Verifies Round 1 architectural claims, examines Smithery deployment behavior, and checks mcp-server-template.md for architectural patterns the codebase missed.

---

## Hallucination Audit

### Claim: "serveMyAPI is the credential backbone for 20+ MCP servers"
**Verified from CLAUDE.md:** The MCP registry lists 20 server entries. Of these, the following explicitly reference serveMyAPI for credentials:
- `brave-search`: `${await serveMyAPI.getKey('brave_search')}`
- `google-search`: `${await serveMyAPI.getKey('google_search')}` and `${await serveMyAPI.getKey('google_search_engine_id')}`
- `neon`: `${await serveMyAPI.getKey('neon_database')}`
- `leonardoAI`: `${await serveMyAPI.getKey('cMax Leonardo API')}`
- `agentql`: `${await serveMyAPI.getKey('agentql')}`
- `perplexity-ask`: env var `PERPLEXITY_API_KEY` (uses Docker env, separate pattern)

That is **5 servers with direct serveMyAPI key references** (not 20+). The other 15 MCP servers do not reference serveMyAPI keys. The "20+" claim was overstated -- serveMyAPI is the credential provider for **5-6 MCP servers**, not 20+. It is still a foundational dependency for those servers, but the scope is more limited than Round 1 stated.

**Correction applied:** The ecosystem role is significant but affects ~6 servers, not the entire 20-server ecosystem.

### Claim: "3 of 5 deployment modes have known bugs"
**Verified:**
1. HTTP/SSE: SSE `Date.now()` session ID and "last transport" routing -- confirmed from server.ts:160, 180
2. Docker: HEALTHCHECK `curl -f http://localhost:3000/` vs CMD `node dist/index.js` -- confirmed from Dockerfile lines 26-30
3. DMG: `run.sh` references `main.js` but build copies `*.js` from project root, not from `dist/` -- confirmed from build_dmg.sh:63, 73

**All three bug claims confirmed.**

### Claim: "The Smithery deployment is a 5th deployment mode"
**Partial correction:** Smithery is a hosting platform that wraps the stdio server. The `smithery.yaml` specifies `type: stdio` and `command: "node", args: ["dist/index.js"]`. Smithery would build the Docker image and then run the stdio server inside it. So Smithery deployment IS the Docker deployment -- it uses the Dockerfile. This means the Docker HEALTHCHECK bug also affects Smithery. There are **4 deployment modes** (not 5): stdio native, HTTP/SSE, Docker/Smithery, and DMG.

### Claim: "Express 5 breaking changes"
**Verified:** Express 5.0.1 is a major version. However, the server.ts code uses only basic Express features (`app.get()`, `app.post()`, `express.json()`, `app.listen()`, `res.send()`, `res.setHeader()`, `res.status().json()`). None of these have breaking changes from Express 4 to 5. The claim that "the usage is basic enough for any HTTP framework" is correct.

### Claim: "Anti-pattern AP-6: No shared module for tool definitions"
**Verified:** grep for tool registration across all files confirms 4 tool registrations in index.ts and 4 in server.ts with identical handler code. No shared module exists. **Confirmed.**

---

## Gap Closure: Smithery Deployment Behavior

The `smithery.yaml` `build` section references the Dockerfile:
```yaml
build:
  dockerfile: Dockerfile
  dockerBuildPath: "."
```

And the `startCommand` specifies:
```yaml
startCommand:
  type: stdio
  commandFunction: |
    function(config) {
      return {
        command: "node",
        args: ["dist/index.js"],
        env: { "NODE_ENV": "production" }
      };
    }
```

This means Smithery:
1. Builds using the Dockerfile (which sets `DOCKER_ENV=true`)
2. Runs `node dist/index.js` (stdio transport)
3. The `DOCKER_ENV=true` from Dockerfile activates file-based storage

So on Smithery, credentials are stored as **plaintext files** inside the container, with `chmod 777` permissions. This is the same Docker deployment with the same security implications.

**New observation:** The Smithery `commandFunction` adds `NODE_ENV: "production"` to the environment, but no code in the codebase reads `NODE_ENV`. This environment variable has no effect.

## Gap Closure: mcp-server-template.md Architectural Patterns

The template document (reference material) describes recommended MCP server patterns including:
- Shared tool definitions via a tools module
- Proper error handling with typed errors
- Resource and prompt registration (beyond just tools)

The serveMyAPI codebase does NOT follow these recommendations:
- No shared tools module (AP-6)
- No typed errors (string messages only)
- No MCP resources or prompts registered (tools only)

This confirms the architectural anti-patterns identified in Round 1 are deviations from the MCP SDK's own recommended practices.

## Gap Closure: Module Initialization Order

The architecture has an implicit initialization dependency:

```
1. Node.js loads index.ts (or server.ts)
2. ES module system resolves imports
3. keychain.ts module loads:
   a. Constants evaluated (SERVICE_NAME, PERMISSION_MARKER, STORAGE_DIR, IS_DOCKER)
   b. KeychainService class defined
   c. `new KeychainService()` called (default export)
   d. Constructor runs:
      - Native: checkPermissionMarker() (async, fire-and-forget)
      - Docker: ensureStorageDirectory() (sync)
4. index.ts continues:
   a. McpServer created
   b. 4 tools registered (synchronous)
   c. StdioServerTransport created
   d. server.connect(transport) (async)
5. Server ready to accept tool calls
```

**Race window:** Between step 3d (async permission marker) and step 5 (server ready), there is a brief window where the permission marker check may still be in progress. The first tool call within this window triggers the belt-and-suspenders guard. This is already documented in Phase A but is worth noting as an architectural initialization concern.

---

## Revised Deployment Topology (4 modes, not 5)

| Mode | Entry | Transport | Storage | Config Source | Status |
|------|-------|-----------|---------|---------------|--------|
| Native stdio | `dist/index.js` | MCP stdio | keytar (OS keyring) | MCP client config | **Functional** |
| HTTP/SSE | `dist/server.js` | Express SSE | keytar (OS keyring) | Manual launch | **Broken** (concurrent sessions) |
| Docker / Smithery | `dist/index.js` (Dockerfile) | MCP stdio | File-based (plaintext, chmod 777) | Dockerfile + smithery.yaml | **Partially broken** (HEALTHCHECK) |
| macOS DMG | `run.sh -> main.js` | MCP stdio (intended) | keytar (OS keyring) | build_dmg.sh | **Non-functional** (wrong file reference) |

---

## Refined Architecture Diagram (incorporating corrections)

```
+==========================================================================+
|                        ServeMyAPI System v1.0                              |
|               (Credential Provider for 5-6 MCP Servers)                    |
+==========================================================================+
|                                                                            |
|  INITIALIZATION ORDER: Constants -> Class Def -> Singleton -> Tools -> Srv |
|                                                                            |
|  TRANSPORT LAYER                                                           |
|  +------------------+  +------------------+  +-----------------+          |
|  | index.ts         |  | server.ts        |  | cli.ts          |          |
|  | Stdio MCP Server |  | HTTP/SSE MCP Svr |  | Direct CLI      |          |
|  | [WORKING]        |  | [BROKEN-concurr] |  | [WORKING]       |          |
|  |                  |  |                  |  |                 |          |
|  | 4 tools (Zod)    |  | 4 tools (Zod)    |  | 5 commands      |          |
|  | copy #1          |  | copy #2          |  | (no Zod)        |          |
|  +--------+---------+  +--------+---------+  +--------+--------+          |
|           |                      |                     |                   |
|           +----------+-----------+---------------------+                   |
|                      |                                                     |
|  SERVICE LAYER       v                                                     |
|           +----------+----------+                                          |
|           | KeychainService     | (singleton, default export)              |
|           | - hasStoredPerm...  | (instance flag)                          |
|           +----------+----------+                                          |
|                      |                                                     |
|                      | IS_DOCKER env check                                |
|                      |                                                     |
|  STORAGE LAYER       |  (embedded, no interface)                           |
|           +----------+----------+                                          |
|           |     |          |    |                                          |
|     [keytar] [fs.Sync]  (inline conditional, not Strategy)                |
|                                                                            |
|  DEAD CODE: src/cli.js (MCP client CLI, unreachable after build)          |
|                                                                            |
|  EXTERNAL SCHEMA: smithery.yaml tools (copy #3, JSON Schema format)       |
+==========================================================================+
```

---

## Delta Summary
- New items added: Smithery `NODE_ENV` env var has no effect, mcp-server-template.md confirms anti-patterns are deviations from SDK recommendations, module initialization order documented
- Existing items refined: Ecosystem role corrected from "20+" to "5-6" dependent MCP servers, deployment modes corrected from 5 to 4 (Smithery = Docker), Smithery security implications (plaintext + chmod 777)
- Remaining gaps: None substantive -- architecture is fully characterized

## Novelty Assessment
Novelty: NITPICK
The ecosystem role correction (5-6 servers, not 20+) is a precision refinement that does not change the architectural significance finding. The Smithery = Docker consolidation simplifies the deployment topology but does not change the bug analysis. The mcp-server-template.md confirmation is supporting evidence, not new discovery. The initialization order documentation is a refinement of the existing async constructor analysis.

## Convergence Declaration
Pass 1 has converged -- findings are nitpicks, not gaps. The architecture is fully characterized with 4 deployment modes (1 working, 1 partially broken, 1 broken HEALTHCHECK, 1 non-functional), 3 layers (with embedded storage), and corrected ecosystem role assessment.

## State Checkpoint
```yaml
pass: 1
round: 2
status: complete
timestamp: 2026-04-13T23:55:00Z
novelty: NITPICK
```
