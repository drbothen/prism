# Pass 2 Deep: Domain Model -- serveMyAPI (Round 2)

## Preamble

This round targets the remaining gaps from Round 1: keytar ordering guarantees, build_dmg.sh domain implications, example config files, and any missed domain concepts. All source files have been re-read.

---

## Gap Closure: keytar findCredentials Ordering

The `findCredentials` call at keychain.ts:175 returns an array of `{account, password}` objects. The keytar documentation does not guarantee any particular order. On macOS, the order depends on the Keychain's internal storage (typically insertion order but not contractually guaranteed). On Linux (libsecret), order is undefined. On Windows (Credential Manager), order is alphabetical by target name.

**Impact on Prism:** The `list-api-keys` tool output order is platform-dependent and non-deterministic. If Prism wants consistent ordering, it should sort explicitly (e.g., alphabetical by name). The current codebase does not sort.

Similarly, the Docker file backend at keychain.ts:187 (`fs.readdirSync`) returns entries in filesystem order, which is also platform-dependent (on macOS HFS+ it is alphabetical, on ext4 it is inode order).

**New value object:** None needed -- this is a behavioral refinement, not a new entity.

---

## Gap Closure: build_dmg.sh Domain Implications

The build script (build_dmg.sh) reveals:

### VO-2.08: Application Identity (macOS Bundle)

| Property | Value | Source |
|----------|-------|--------|
| CFBundleIdentifier | `com.newmodel.serveMyAPI` | build_dmg.sh:36 |
| CFBundleName | `serveMyAPI` | build_dmg.sh:40 |
| CFBundlePackageType | `APPL` | build_dmg.sh:42 |
| LSApplicationCategoryType | `public.app-category.utilities` | build_dmg.sh:46 |
| LSMinimumSystemVersion | `12.0` (macOS Monterey+) | build_dmg.sh:48 |
| LSUIElement | `true` (no Dock icon -- runs as background/menu bar app) | build_dmg.sh:50 |
| Copyright | `Copyright (c) 2025 James King` | build_dmg.sh:52 |

The `LSUIElement = true` is a notable domain decision: the app is intended to run as an **invisible background service** with no Dock icon or visible window. This aligns with its MCP server role.

The bundle identifier `com.newmodel.serveMyAPI` reveals the organization domain (`newmodel` -- matching the author's company New Model Venture Capital).

### R-2.05: DMG Launcher delegates to Node.js

The launcher script (`run.sh`, generated at build_dmg.sh:60-63) does:
```bash
cd "$(dirname "$0")/../Resources"
./node main.js
```

It copies the system's Node.js binary into the app bundle (`cp "$(which node)" "$RESOURCES_DIR/"` at line 70) and runs `main.js` -- but the actual main file is `dist/index.js`, suggesting either the script is outdated or there is a missing build step to rename/copy the entry point.

**Bug:** The DMG launcher references `main.js` but the built output is `dist/index.js`. The `cp -R *.js package.json package-lock.json "$RESOURCES_DIR/"` at line 73 copies all JS files from the project root, which does not include `dist/` contents. The DMG build process appears to be **non-functional as written**.

---

## Gap Closure: Example Config Files

The two example config files (`claude_desktop_config.json` and `windsurf_config.json`) are **identical** in structure and content:

```json
{
  "mcpServers": {
    "serveMyAPI": {
      "command": "node",
      "args": ["/ABSOLUTE/PATH/TO/servemyapi/dist/index.js"]
    }
  }
}
```

These confirm:
- The standard deployment model is stdio transport via `dist/index.js`
- No environment variables, no arguments, no authentication tokens needed for stdio mode
- The server name in MCP client configs is `"serveMyAPI"` (matching the McpServer name)
- Both Claude Desktop and Windsurf use the same config format (standard MCP client config)

No new domain entities here -- just deployment confirmation.

---

## Gap Closure: TypeScript Configuration Domain Impact

The `tsconfig.json` reveals:
- `strict: true` -- enables all strict type checking (noImplicitAny, strictNullChecks, etc.)
- `declaration: true` + `declarationMap: true` -- generates .d.ts files, indicating the package is intended to be importable as a library (not just executable)
- `sourceMap: true` -- debugging support

The declaration generation suggests the author intended KeychainService to be consumable as a library dependency, not just via the CLI/MCP transport. However, no package publishing config exists (no `files` field in package.json, no `.npmignore`).

---

## Refined Entity Catalog Additions

### E-2.05: SSESession (Ephemeral, server.ts only)

From Round 1 analysis, the SSE transport maintains a map of active sessions. This was noted as VO-2.07 (Session ID) but is more properly modeled as an ephemeral entity:

| Property | Type | Source |
|----------|------|--------|
| id | string (Date.now().toString()) | server.ts:160 |
| transport | SSEServerTransport | server.ts:161 |
| lifetime | Tied to HTTP connection | server.ts:171-173 |

**Storage:** In-memory `Map<string, any>` (server.ts:157). No persistence. Lost on process restart.

**Lifecycle:**
```
[Created on GET /sse] --> [Active] --> [Destroyed on client disconnect]
```

---

## Refined Relationship Map (Final)

```
                  +---------------------------+
                  |    Credential (implicit)   |
                  |    name: string            |
                  |    key: string             |
                  +---------------------------+
                            |
                  belongs-to (N:1)
                            |
                  +---------------------------+
                  |  SERVICE_NAME = 'serveMyAPI' |
                  |  (hardcoded namespace)      |
                  +---------------------------+
                            |
                   stored-via (1:1, conditional)
                     /                 \
            +-------+-------+   +-------+-------+
            | keytar backend |   | file backend  |
            | (IS_DOCKER=F)  |   | (IS_DOCKER=T) |
            +---------------+   +---------------+
                                       |
                                 uses (1:1)
                                       |
                                +------+-------+
                                | STORAGE_DIR  |
                                | /app/data    |
                                +--------------+

  PermissionMarker (sentinel):
    Same as Credential but name='_permission_granted', key='true'
    Filtered from listKeys() output
    Only exists in keytar backend (not file backend)
```

---

## Complete Ubiquitous Language Glossary (Merged)

| Term | Meaning | Source |
|------|---------|--------|
| API key | Any secret string stored by user-chosen name | All source files |
| Key name / name | Unique identifier for a credential | Tool parameters, CLI args |
| Keychain | OS-level secure credential store | keychain.ts, docs |
| Service / service name | keytar namespace grouping (`serveMyAPI`) | keychain.ts:5 |
| Permission marker | Sentinel credential `_permission_granted` | keychain.ts:6 |
| Store | Create or overwrite a credential (no create/update distinction) | All interfaces |
| Docker mode | File-based fallback (`DOCKER_ENV=true`) | keychain.ts:8 |
| Storage directory | Filesystem root for Docker credentials | keychain.ts:7 |
| MCP tool | A registered function callable by MCP clients | index.ts, server.ts |
| Transport | Communication channel (stdio, SSE, direct call) | Architecture |
| Menu bar app | macOS deployment with no Dock icon | build_dmg.sh (LSUIElement) |
| `add` | CLI alias for `store` (not in MCP) | cli.ts:42 |
| `remove` | CLI alias for `delete` (not in MCP) | cli.ts:59 |
| `set` | cli.js alias for `store` (not in MCP or cli.ts) | cli.js:67 |

**Note on vocabulary inconsistency:** cli.js uses `set` as the argument value but maps it to `store-api-key` tool call. cli.ts uses `store` and `add`. The MCP tools use `store-api-key`. Three interfaces, three slightly different vocabularies.

---

## Domain Model Completeness Assessment

The domain is now fully characterized. The entity catalog contains:

| ID | Entity | Status |
|----|--------|--------|
| E-2.01 | Credential (implicit) | Complete |
| E-2.02 | PermissionMarker (sentinel) | Complete |
| E-2.03 | McpServer instance | Complete |
| E-2.04 | McpClient instance | Complete |
| E-2.05 | SSESession (ephemeral) | Complete (new this round) |

Value objects: VO-2.01 through VO-2.08 (SERVICE_NAME, PERMISSION_MARKER, STORAGE_DIR, IS_DOCKER, MCP Response Shape, Port, Session ID, App Bundle Identity).

Relationships: R-2.01 through R-2.05 fully mapped.

State machines: SM-2.01 through SM-2.03 fully drawn.

---

## Delta Summary
- New items added: 1 entity (E-2.05 SSESession), 1 value object (VO-2.08 App Bundle Identity), 1 relationship (R-2.05 DMG launcher), 1 bug (DMG launcher references wrong file), 1 vocabulary inconsistency (`set` in cli.js)
- Existing items refined: Ordering behavior for listKeys (platform-dependent), declaration generation intent, example config confirmation
- Remaining gaps: None substantive -- all source files have been read and cross-referenced

## Novelty Assessment
Novelty: NITPICK
The SSESession entity formalization, the DMG launcher bug, and the `set` vocabulary variant are refinements to the model, not changes to it. None of these findings would change how you spec the core credential management system. The DMG script bug is notable but is a packaging concern, not a domain concern.

## Convergence Declaration
Pass 2 has converged -- findings are nitpicks, not gaps. The domain model is fully characterized across 5 entities, 8 value objects, 5 relationships, and 3 state machines.

## State Checkpoint
```yaml
pass: 2
round: 2
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
