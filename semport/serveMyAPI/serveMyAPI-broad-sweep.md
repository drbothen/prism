# ServeMyAPI -- Full Codebase Ingestion

> Ingested 2026-04-13 for Prism reference. This document captures the complete
> behavioral intent, architecture, domain model, and design decisions of the
> ServeMyAPI MCP server so that Prism can replicate and improve upon its
> credential-management patterns in Rust.

---

## Pass 0: Inventory

### Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Language | TypeScript | 5.8.2 |
| Runtime | Node.js | 20+ (Dockerfile uses node:20-slim) |
| MCP SDK | @modelcontextprotocol/sdk | ^1.7.0 |
| OS Keyring | keytar | ^7.9.0 |
| HTTP Framework | Express | ^5.0.1 |
| Schema Validation | Zod | ^3.24.2 |
| Build | tsc (vanilla TypeScript compiler) |
| Test Framework | **None** -- `npm test` exits with error |

### Key Dependencies

- **keytar ^7.9.0** -- Native Node.js addon that provides cross-platform credential
  storage. On macOS it uses Keychain, on Windows it uses Credential Vault, on
  Linux it uses libsecret/GNOME Keyring. This is the single most important
  dependency for Prism to understand.
- **@modelcontextprotocol/sdk ^1.7.0** -- Official MCP TypeScript SDK providing
  `McpServer`, `StdioServerTransport`, `SSEServerTransport`, and client classes.
- **express ^5.0.1** -- Used only for the HTTP/SSE transport variant.
- **zod ^3.24.2** -- Schema validation for MCP tool parameter definitions.

### File Manifest

| Path | Type | Lines | Purpose |
|------|------|-------|---------|
| `src/index.ts` | Entry point (stdio) | 158 | Main MCP server with stdio transport; registers all 4 tools |
| `src/server.ts` | Entry point (HTTP) | 230 | Identical tool registration + Express SSE transport |
| `src/services/keychain.ts` | Core service | 198 | Credential CRUD via keytar with Docker file-based fallback |
| `src/cli.ts` | CLI | 118 | Direct CLI that calls keychainService methods |
| `src/cli.js` | CLI (alt) | 164 | MCP-client-based CLI that spawns the server and calls tools |
| `package.json` | Config | 38 | Project metadata, deps, scripts |
| `tsconfig.json` | Config | 16 | TS compiler config (ES2022, NodeNext) |
| `Dockerfile` | Deploy | 35 | Docker build with file-based storage fallback |
| `smithery.yaml` | Deploy | 81 | Smithery hosting config with tool schemas |
| `build_dmg.sh` | Deploy | 109 | macOS .app bundle / DMG packaging script |
| `examples/claude_desktop_config.json` | Config example | 10 | Claude Desktop MCP config |
| `examples/windsurf_config.json` | Config example | 10 | Windsurf MCP config |

### Entry Points

1. **`src/index.ts`** -- Primary. Stdio MCP server. This is what Claude Desktop and
   other MCP clients launch via `node dist/index.js`.
2. **`src/server.ts`** -- Secondary. HTTP/SSE MCP server on Express. Used for
   browser-based or network-accessible deployments.
3. **`src/cli.ts`** -- Tertiary. Direct CLI for terminal-based key management
   (compiled to `dist/cli.js`, registered as `api-key` bin).
4. **`src/cli.js`** -- Quaternary. Alternative CLI that acts as an MCP client,
   spawning the server process and calling tools over stdio.

---

## Pass 1: Architecture

### Component Catalog

```
+------------------------------------------------------------------+
|                        ServeMyAPI System                          |
+------------------------------------------------------------------+
|                                                                    |
|  +-----------------+    +-----------------+    +--------------+   |
|  | index.ts        |    | server.ts       |    | cli.ts       |   |
|  | (Stdio MCP)     |    | (HTTP/SSE MCP)  |    | (Direct CLI) |   |
|  +--------+--------+    +--------+--------+    +------+-------+   |
|           |                      |                     |          |
|           |   All three call     |                     |          |
|           +----------+-----------+---------------------+          |
|                      |                                            |
|                      v                                            |
|           +----------+----------+                                 |
|           | KeychainService     |                                 |
|           | (services/keychain) |                                 |
|           +----------+----------+                                 |
|                      |                                            |
|           +----------+----------+                                 |
|           |   Storage Backend   |                                 |
|           +-----+----------+---+                                 |
|                 |          |                                      |
|           +-----+--+  +---+--------+                             |
|           | keytar  |  | File-based |                             |
|           | (native)|  | (Docker)   |                             |
|           +--------+  +------------+                             |
+------------------------------------------------------------------+
```

### Layer Structure

The architecture is a simple **2-layer design**:

1. **Transport Layer** (index.ts, server.ts, cli.ts, cli.js)
   - Handles MCP protocol communication (stdio or SSE)
   - Registers tools with Zod schemas
   - Maps tool calls to KeychainService methods
   - Handles error formatting into MCP response format

2. **Service Layer** (services/keychain.ts)
   - Single service class: `KeychainService`
   - Abstracts storage backend (keytar vs file-based)
   - Owns the credential lifecycle logic
   - Exported as a singleton default instance

There is **no domain layer** separate from the service layer -- the domain is thin
enough that entities (credentials) are represented as plain string key-value pairs
rather than typed objects.

### Transport Variants

| Transport | File | Protocol | Use Case |
|-----------|------|----------|----------|
| Stdio | `index.ts` | MCP over stdin/stdout | Claude Desktop, MCP clients |
| HTTP/SSE | `server.ts` | MCP over Server-Sent Events | Browser, network access |
| Direct CLI | `cli.ts` | None (direct function calls) | Terminal management |
| MCP Client CLI | `cli.js` | MCP over stdio (as client) | Alternative terminal access |

### Critical Architecture Observations for Prism

1. **Code duplication**: The 4 MCP tools are defined identically in both `index.ts`
   and `server.ts`. There is no shared tool-definition module. Prism should define
   tools once and share them across transports.

2. **Singleton service**: `KeychainService` is instantiated once at module load and
   exported as default. The permission marker check runs in the constructor.

3. **No middleware or auth**: The HTTP/SSE server has zero authentication. Anyone
   who can reach port 3000 can read all stored API keys.

4. **SSE session management is fragile**: The `activeTransports` map uses
   `Date.now().toString()` as IDs and the POST `/messages` handler grabs the
   *last* transport in the map, which breaks with concurrent connections.

### Cross-Cutting Concerns

| Concern | Implementation | Quality |
|---------|---------------|---------|
| Error handling | try/catch in every tool handler, returns `isError: true` | Adequate but repetitive |
| Logging | `console.error` only | Minimal |
| Auth/Security | macOS Keychain ACL (via keytar); no app-level auth | OS-level only |
| Input validation | Zod `z.string().min(1)` on tool parameters | Basic |
| Health check | Dockerfile HEALTHCHECK via curl | Docker only |

---

## Pass 2: Domain Model

### Entities

The domain is extremely thin. There is effectively one entity:

#### Credential (API Key)

| Property | Type | Source | Notes |
|----------|------|--------|-------|
| name | string | Tool parameter | Acts as the unique identifier / account name |
| key | string | Tool parameter | The secret value (API key, token, etc.) |

Credentials are **not modeled as a type** -- they are plain string pairs passed
directly through to keytar. There is no Credential class, interface, or type alias.

### Value Objects

- **SERVICE_NAME** = `'serveMyAPI'` -- The keytar service identifier. All credentials
  are stored under this single service namespace. This is analogous to a "keyring
  name" or "credential group."

- **PERMISSION_MARKER** = `'_permission_granted'` -- A sentinel credential stored
  in keytar to pre-authorize keychain access and consolidate macOS permission
  prompts into a single initial request.

### Domain Operations (CRUD)

| Operation | Method | keytar Call | Docker Fallback |
|-----------|--------|-------------|-----------------|
| Store | `storeKey(name, key)` | `setPassword(SERVICE_NAME, name, key)` | Write `{name}.key` file |
| Retrieve | `getKey(name)` | `getPassword(SERVICE_NAME, name)` | Read `{name}.key` file |
| Delete | `deleteKey(name)` | `deletePassword(SERVICE_NAME, name)` | Unlink `{name}.key` file |
| List | `listKeys()` | `findCredentials(SERVICE_NAME)` | List `*.key` files in dir |

### State Machine

Credentials have no state transitions -- they exist or they don't. The lifecycle is:

```
[Not Exists] --store--> [Exists] --delete--> [Not Exists]
                          |   ^
                          +---+  (store overwrites silently)
```

There is no update operation -- storing with an existing name silently overwrites
(this is keytar's native behavior with `setPassword`).

### Bounded Context Map

There is a single bounded context: **Credential Management**. It has no
relationships to other contexts, no events, no aggregates beyond the individual
credential.

---

## Pass 3: Behavioral Contracts

**Note: This codebase has zero tests.** (`npm test` = `echo "Error: no test specified" && exit 1`).
All contracts are inferred from code. Confidence is MEDIUM at best.

### BC-001: store-api-key stores a credential in the OS keyring

**Preconditions:**
- `name` is a non-empty string (Zod: `z.string().min(1)`)
- `key` is a non-empty string (Zod: `z.string().min(1)`)
- On macOS: user has granted Keychain access (permission marker handles this)
- In Docker: STORAGE_DIR exists and is writable

**Postconditions:**
- Credential is persisted in keytar under service `serveMyAPI`, account = `name`
- If credential already exists with that name, it is silently overwritten
- Returns MCP text response: `"Successfully stored API key with name: {name}"`

**Error Cases:**
- keytar throws (e.g., Keychain locked, permission denied) -> returns `isError: true`
  with error message
- In Docker: filesystem write fails -> throws (propagated to tool handler)

**Confidence:** MEDIUM (from code, no tests)

### BC-002: get-api-key retrieves a credential by name

**Preconditions:**
- `name` is a non-empty string

**Postconditions:**
- If found: returns the raw key value as plain text in MCP response
- If not found: returns `isError: true` with `"No API key found with name: {name}"`

**Error Cases:**
- keytar throws -> `isError: true` with error message

**Security Note:** The key value is returned as **plain text** in the MCP response.
There is no redaction, masking, or access control beyond the OS keyring's native
protections. Once the MCP transport delivers the response, the secret is in the
clear.

**Confidence:** MEDIUM (from code, no tests)

### BC-003: delete-api-key removes a credential by name

**Preconditions:**
- `name` is a non-empty string

**Postconditions:**
- If credential existed: deleted from keytar, returns success message
- If credential did not exist: keytar returns `false`, tool returns `isError: true`
  with `"No API key found with name: {name}"`

**Error Cases:**
- keytar throws -> `isError: true` with error message

**Confidence:** MEDIUM (from code, no tests)

### BC-004: list-api-keys enumerates all stored credential names

**Preconditions:**
- None (no parameters)

**Postconditions:**
- Returns newline-separated list of credential names (account names from keytar)
- The permission marker `_permission_granted` is **filtered out** of results
- If no credentials exist: returns `"No API keys found"` (not an error)

**Error Cases:**
- keytar throws -> `isError: true` with error message

**Confidence:** MEDIUM (from code, no tests)

### BC-005: Permission marker consolidates Keychain prompts

**Preconditions:**
- Running on native macOS (not Docker)
- First access to the Keychain

**Postconditions:**
- On KeychainService construction, checks for `_permission_granted` marker
- If absent: creates it via `setPassword`, which triggers the macOS Keychain
  permission dialog once
- Subsequent operations skip the check (instance flag `hasStoredPermissionMarker`)
- Every CRUD method re-checks the marker if the flag is still false (belt-and-suspenders)

**Design Intent:** macOS Keychain prompts the user for permission per-app. By writing
a marker credential at startup, the permission dialog appears once at initialization
rather than on the first actual credential operation.

**Confidence:** MEDIUM (from code, no tests)

### BC-006: Docker fallback uses file-based storage

**Preconditions:**
- `DOCKER_ENV=true` environment variable is set
- `STORAGE_DIR` env var points to writable directory (default: `/app/data`)

**Postconditions:**
- All CRUD operations use filesystem instead of keytar
- Keys stored as `{STORAGE_DIR}/{name}.key` files, plaintext UTF-8
- List operation reads directory and strips `.key` suffix
- No permission marker logic in Docker mode

**Security Note:** In Docker mode, credentials are stored as **plaintext files** on
the filesystem. There is no encryption. Security depends entirely on filesystem
permissions and container isolation.

**Confidence:** MEDIUM (from code, no tests)

---

## Pass 4: NFR Catalog

### Security

| NFR | Implementation | Assessment |
|-----|---------------|------------|
| Credential encryption at rest | Delegated to OS keyring (macOS Keychain, Windows Credential Vault, Linux libsecret) | GOOD -- leverages OS-level encryption |
| Credential encryption in Docker | None -- plaintext files | POOR -- no encryption layer |
| Transport security (stdio) | Inherent -- stdin/stdout within same process | N/A |
| Transport security (HTTP) | None -- no TLS, no auth | POOR -- any network client can read all keys |
| Input sanitization | Zod `min(1)` only -- no character restrictions on names | MINIMAL |
| Access control | OS keyring ACL only | OS-level only, no app-level RBAC |
| Secrets in logs | Keys not logged; only names appear in success messages | GOOD |
| Secrets in responses | `get-api-key` returns raw key as plaintext | BY DESIGN but notable |

### Reliability

| NFR | Implementation | Assessment |
|-----|---------------|------------|
| Error recovery | try/catch per operation, returns error to caller | Basic |
| Graceful shutdown | None | MISSING |
| Retry logic | None | MISSING |
| Health check | Docker HEALTHCHECK only | Docker-only |

### Observability

| NFR | Implementation | Assessment |
|-----|---------------|------------|
| Logging | `console.error` for startup message and errors | MINIMAL |
| Structured logging | None | MISSING |
| Metrics | None | MISSING |
| Tracing | None | MISSING |

### Performance

| NFR | Implementation | Assessment |
|-----|---------------|------------|
| Connection pooling | N/A (keytar uses native calls, no pool needed) | N/A |
| Caching | `hasStoredPermissionMarker` instance flag caches permission check | Minimal |
| Concurrent access | No locking; keytar operations are serialized by OS | Implicit |

---

## Pass 5: Convention and Pattern Catalog

### Coding Conventions

| Convention | Example | Consistency |
|------------|---------|-------------|
| 2-space indentation | All files | CONSISTENT |
| camelCase for variables/functions | `storeKey`, `keychainService` | CONSISTENT |
| PascalCase for classes | `KeychainService` | CONSISTENT |
| JSDoc on public methods | All KeychainService methods | CONSISTENT in service, ABSENT in tools |
| ES module imports | `import x from 'y'` | CONSISTENT |
| Async/await (no raw Promises) | All async code | CONSISTENT |

### Design Patterns

#### Singleton Service Pattern
`KeychainService` is exported both as a class and as a pre-instantiated default
export. Consumers import the singleton instance:
```typescript
export class KeychainService { ... }
export default new KeychainService();
```

#### Strategy Pattern (implicit)
The `IS_DOCKER` flag selects between two storage strategies (keytar vs file) at
the method level. Each CRUD method has an `if (IS_DOCKER) return this.xxxFile()`
guard. This is **not** a proper Strategy pattern -- it's inline conditional logic.
A proper implementation would use an interface with two implementations.

#### MCP Tool Registration Pattern
Tools are registered with `server.tool(name, zodSchema, handler)`:
```typescript
server.tool(
  "tool-name",
  { param: z.string().min(1).describe("description") },
  async ({ param }) => {
    try {
      // call service
      return { content: [{ type: "text", text: "result" }] };
    } catch (error) {
      return { content: [{ type: "text", text: `Error: ${msg}` }], isError: true };
    }
  }
);
```
Every tool handler follows this exact pattern: try/catch wrapping a service call,
returning `{ content: [{ type: "text", text }] }` on success and
`{ content: [...], isError: true }` on failure.

### Anti-Patterns and Code Smells

1. **Duplicated tool definitions** -- The 4 tools are copy-pasted identically between
   `index.ts` and `server.ts`. Any change requires updating both files. This is the
   single biggest maintenance risk.

2. **No shared tool module** -- Tools should be defined once and registered on any
   McpServer instance. The codebase does not extract this.

3. **Inline storage strategy selection** -- Every CRUD method checks `IS_DOCKER`.
   This should be a pluggable backend behind an interface.

4. **Permission marker as a credential** -- Storing `_permission_granted` as an
   actual keytar credential means it shows up in `findCredentials` and must be
   filtered. This is a leaky abstraction.

5. **No input validation beyond Zod min(1)** -- Key names can contain any characters
   including path separators, which in Docker mode could be a **path traversal
   vulnerability** (`name = "../../etc/passwd"` would write to
   `/app/data/../../etc/passwd.key`).

6. **SSE transport session management** -- Using `Date.now()` as session ID and
   grabbing the last transport for POST messages is fundamentally broken for
   concurrent users.

7. **Unused import in index.ts** -- `keytar` is imported directly but never used
   (all access goes through `keychainService`).

8. **Two CLI implementations** -- `cli.ts` (direct) and `cli.js` (MCP client) serve
   the same purpose but use different approaches. Only `cli.ts` is the "real" one
   (registered as the `api-key` bin).

---

## Pass 6: Synthesis

### Executive Summary

ServeMyAPI is a small (~870 lines of TypeScript) MCP server that provides CRUD
operations for API key storage using the OS keyring via the `keytar` npm package.
It exposes 4 tools (store, get, delete, list) over both stdio and HTTP/SSE MCP
transports, with a CLI for direct terminal access. The architecture is simple and
functional but has significant code duplication, no tests, no input sanitization
beyond basic non-empty checks, and a Docker fallback that stores credentials in
plaintext files. The core value is in its integration pattern: using `keytar` to
abstract OS-level credential storage behind a service that MCP clients can call.

### Key Findings

1. **keytar is the cross-platform abstraction** -- It wraps macOS Keychain,
   Windows Credential Vault, and Linux libsecret/GNOME Keyring behind a single
   async API: `setPassword`, `getPassword`, `deletePassword`, `findCredentials`.
   For Prism (Rust), the equivalent crate is likely `keyring-rs` or direct
   platform API calls via `security-framework` (macOS), `windows-credentials`
   (Windows), and `secret-service`/`libsecret` bindings (Linux).

2. **The service namespace pattern** -- All credentials are stored under a single
   service name (`serveMyAPI`). This groups them in the OS keyring. The "account"
   field is the user-chosen key name. Prism should use a similar namespace
   (`prism` or configurable) to avoid collisions with other apps.

3. **Permission marker trick** -- On macOS, the first keytar access triggers a
   permission dialog. ServeMyAPI pre-writes a marker credential at startup to
   surface this dialog immediately rather than on the first real operation. Prism
   may need a similar strategy, or it can handle this at the Rust level by
   attempting a probe read at initialization.

4. **MCP tool registration is formulaic** -- Every tool follows the same pattern:
   name + Zod schema + async handler with try/catch. Prism should define a macro
   or trait-based registration that eliminates this boilerplate.

5. **No test coverage** -- All behavioral contracts are inferred from code alone.
   Prism must not replicate this gap.

### Confidence Assessment

| Area | Confidence | Basis |
|------|-----------|-------|
| Architecture | HIGH | Complete source read, simple structure |
| Domain Model | HIGH | Trivially simple domain (string key-value pairs) |
| Behavioral Contracts | MEDIUM | No tests; all inferred from code |
| NFRs | HIGH | Straightforward to assess from code |
| Conventions | HIGH | Small codebase, patterns are obvious |

### Gaps and Risks

1. **Path traversal in Docker mode** -- Key names are used directly in file paths
   with no sanitization. `name = "../../../etc/cron.d/evil"` would write outside
   the storage directory.

2. **No update semantics** -- Store silently overwrites. There is no way to know
   if you are creating vs updating. Prism may want explicit create/update
   semantics.

3. **No key metadata** -- No timestamps, no description, no tags, no TTL. Just
   name and value. Prism should consider richer metadata.

4. **No access control** -- Any MCP client connected to the server can read all
   keys. There is no per-key ACL, no client authentication.

5. **HTTP/SSE transport is insecure** -- No TLS, no auth, broken session management.

6. **No graceful shutdown** -- Server processes do not handle SIGTERM/SIGINT.

### Recommendations for Prism

1. **Use `keyring-rs`** (or equivalent Rust crate) as the cross-platform keyring
   abstraction. It provides the same macOS Keychain / Windows Credential Manager /
   Linux Secret Service coverage that keytar provides in Node.js.

2. **Define a `CredentialStore` trait** with `store`, `get`, `delete`, `list`
   methods. Implement it for:
   - `KeyringStore` (native OS keyring)
   - `FileStore` (encrypted file-based fallback for containers)
   - `MemoryStore` (for testing)

3. **Define tools once** in a shared module and register them on any MCP server
   transport. Avoid the duplication between stdio and HTTP entry points.

4. **Sanitize key names** -- restrict to alphanumeric + limited punctuation, or
   hash/encode names before using them as file paths or keyring accounts.

5. **Add credential metadata** -- at minimum: created_at timestamp, optional
   description. Consider storing as JSON in the keyring value field with the
   actual secret as a nested field.

6. **Encrypt the file-based fallback** -- if Prism supports a container/headless
   mode, use an encryption key derived from an environment variable or mounted
   secret, not plaintext files.

7. **Add structured error types** -- instead of string error messages, define an
   enum of credential operation errors (NotFound, AlreadyExists, PermissionDenied,
   StorageError, InvalidName).

8. **Write tests from the behavioral contracts above** -- BC-001 through BC-006
   map directly to test cases.

---

## Appendix A: keytar API Reference (for Prism mapping)

The `keytar` npm package provides these methods, all async:

| keytar Method | Signature | OS Mechanism |
|---------------|-----------|-------------|
| `setPassword(service, account, password)` | `(string, string, string) -> Promise<void>` | macOS: `SecItemAdd`/`SecItemUpdate`; Win: `CredWrite`; Linux: `secret_password_store` |
| `getPassword(service, account)` | `(string, string) -> Promise<string \| null>` | macOS: `SecItemCopyMatching`; Win: `CredRead`; Linux: `secret_password_lookup` |
| `deletePassword(service, account)` | `(string, string) -> Promise<boolean>` | macOS: `SecItemDelete`; Win: `CredDelete`; Linux: `secret_password_clear` |
| `findCredentials(service)` | `(string) -> Promise<{account, password}[]>` | Enumerates all credentials for a service |

For Prism's Rust implementation, the mapping is:

| keytar | Rust Equivalent (keyring-rs) | Notes |
|--------|------------------------------|-------|
| `setPassword` | `Entry::new(service, account).set_password(password)` | |
| `getPassword` | `Entry::new(service, account).get_password()` | Returns `Result<String>`, not `Option` |
| `deletePassword` | `Entry::new(service, account).delete_credential()` | |
| `findCredentials` | Not directly supported | Must maintain an index or use platform-specific enumeration |

**Important:** `keyring-rs` does not have a `findCredentials` equivalent. Prism will
need to maintain its own index of stored key names (e.g., store a JSON list of names
as a special credential, or use a local metadata file) to support the `list` operation.

## Appendix B: MCP Tool Schemas (for Prism tool registration)

### store-api-key
```json
{
  "name": "store-api-key",
  "parameters": {
    "name": { "type": "string", "minLength": 1, "description": "The name/identifier for the API key" },
    "key": { "type": "string", "minLength": 1, "description": "The API key to store" }
  },
  "required": ["name", "key"]
}
```

### get-api-key
```json
{
  "name": "get-api-key",
  "parameters": {
    "name": { "type": "string", "minLength": 1, "description": "The name/identifier of the API key to retrieve" }
  },
  "required": ["name"]
}
```

### delete-api-key
```json
{
  "name": "delete-api-key",
  "parameters": {
    "name": { "type": "string", "minLength": 1, "description": "The name/identifier of the API key to delete" }
  },
  "required": ["name"]
}
```

### list-api-keys
```json
{
  "name": "list-api-keys",
  "parameters": {},
  "required": []
}
```

## Appendix C: Docker Deployment Model

The Dockerfile reveals the container deployment strategy:

1. **Environment detection**: `DOCKER_ENV=true` environment variable switches the
   KeychainService to file-based storage.
2. **Storage volume**: `/app/data` (configurable via `STORAGE_DIR`) stores `.key` files.
3. **Health check**: HTTP GET to `/` every 30s (implies the Docker deployment uses
   the HTTP/SSE server variant, not stdio).
4. **No keyring in containers**: This is the fundamental limitation -- Docker
   containers lack OS keyring access. Prism should consider:
   - FUSE-mounted keyring forwarding
   - Encrypted file store with key from environment
   - Remote keyring proxy (key management service)

## Appendix D: Source File Dependency Graph

```
index.ts ------> services/keychain.ts ------> keytar (native)
    |                    |                         |
    |                    +------> fs (Node.js)     +---> macOS Keychain
    |                    +------> path (Node.js)   +---> Windows Credential Vault
    |                                              +---> Linux libsecret
    +-----------> @modelcontextprotocol/sdk
    +-----------> zod

server.ts -----> services/keychain.ts (same)
    +-----------> express
    +-----------> @modelcontextprotocol/sdk (SSE transport)
    +-----------> zod

cli.ts --------> services/keychain.ts (same)

cli.js --------> @modelcontextprotocol/sdk (client)
```
