# Pass 2 Deep: Domain Model -- serveMyAPI (Round 1)

## Preamble

This deepening round reads every source file in the repo and extracts domain entities, value objects, relationships, enums, state machines, and domain events with more precision than the broad sweep. The broad sweep correctly identified the domain as "extremely thin" -- this round verifies that claim and looks for anything missed.

---

## Entity Catalog

### E-2.01: Credential (Implicit)

The codebase has **no explicit Credential type**. Credentials are represented as implicit `(name: string, key: string)` tuples. The closest thing to a structural definition is the keytar return type from `findCredentials`, which returns `{account: string, password: string}[]`, but this type is never surfaced to consumers -- `listKeys()` maps it to `string[]` (names only).

**Properties:**

| Property | Type | Source Location | Semantics |
|----------|------|-----------------|-----------|
| name | string | Tool parameter / keytar "account" | Unique identifier within the service namespace. No constraints beyond non-empty (Zod `z.string().min(1)`). |
| key | string | Tool parameter / keytar "password" | The secret value. Opaque to the system -- no format validation, no length limits beyond non-empty. |

**Identity:** Name is the sole identity key. Two credentials with the same name are the same credential (store overwrites silently).

**Lifecycle:** Create (store) -> Exists -> Destroy (delete). No update operation -- store-to-existing-name is an implicit overwrite.

**Missing properties that Prism should consider:**
- `created_at` / `updated_at` timestamps
- `description` (human-readable purpose)
- `source` (which tool/client stored it)
- `last_accessed_at` (for rotation auditing)

### E-2.02: PermissionMarker (Sentinel Entity)

A special-purpose credential stored in keytar with fixed identity:

| Property | Value | Notes |
|----------|-------|-------|
| name | `_permission_granted` | Hardcoded constant `PERMISSION_MARKER` |
| key | `'true'` | Literal string, value is irrelevant -- existence is the signal |

**Purpose:** Triggers the macOS Keychain permission dialog at startup rather than on first real operation. This is a **sentinel record** pattern -- a domain entity that exists solely to trigger a side effect.

**Relationship to Credential:** Same storage mechanism, same namespace, but explicitly filtered out of `listKeys()` results. It is invisible to consumers.

### E-2.03: McpServer Instance

Each transport variant creates an `McpServer` instance with identity:

| Property | Value | Source |
|----------|-------|--------|
| name | `"ServeMyAPI"` | `index.ts:9`, `server.ts:9` |
| version | `"1.0.0"` | `index.ts:10`, `server.ts:10` |

This is not a domain entity per se, but it defines the MCP server identity that clients see. It is duplicated across both entry points with identical values.

### E-2.04: McpClient Instance (cli.js only)

The MCP client CLI creates a client with identity:

| Property | Value | Source |
|----------|-------|--------|
| name | `"ServeMyAPIClient"` | `cli.js:32` |
| version | `"1.0.0"` | `cli.js:32` |

---

## Value Objects

### VO-2.01: SERVICE_NAME

```typescript
const SERVICE_NAME = 'serveMyAPI';  // keychain.ts:5
```

The keytar service namespace. All credentials are grouped under this single identifier. This is the logical equivalent of a "vault" or "keyring" name. It is hardcoded -- there is no multi-tenant or multi-namespace support.

### VO-2.02: PERMISSION_MARKER

```typescript
const PERMISSION_MARKER = '_permission_granted';  // keychain.ts:6
```

The sentinel credential name. Used as both an existence check and a filter predicate.

### VO-2.03: STORAGE_DIR

```typescript
const STORAGE_DIR = process.env.STORAGE_DIR || '/app/data';  // keychain.ts:7
```

Docker file-based storage root directory. Configurable via environment variable. Default is container-internal `/app/data`.

### VO-2.04: IS_DOCKER

```typescript
const IS_DOCKER = process.env.DOCKER_ENV === 'true';  // keychain.ts:8
```

Boolean flag derived from environment. This is the **storage backend discriminator** -- the single value that selects between keytar and file-based storage strategies. It is evaluated once at module load time and never changes.

### VO-2.05: MCP Tool Response Shape

All tool handlers return one of two shapes:

**Success:**
```typescript
{ content: [{ type: "text", text: string }] }
```

**Error:**
```typescript
{ content: [{ type: "text", text: string }], isError: true }
```

This is a de facto value object -- a standardized MCP response envelope. The `isError` field is only present on errors (not set to `false` on success).

### VO-2.06: Port Configuration

```typescript
const port = process.env.PORT || 3000;  // server.ts:154
```

HTTP server port, configurable via environment. Only relevant for the SSE transport variant.

### VO-2.07: SSE Session ID

```typescript
const id = Date.now().toString();  // server.ts:160
```

Ephemeral session identifier for SSE connections. This is a value object with serious quality issues -- `Date.now()` can collide for simultaneous connections.

---

## Aggregates

There are no aggregates. Each credential is an independent root entity with no child entities or value objects. The SERVICE_NAME groups them logically but does not form an aggregate boundary -- there is no transactional consistency across credentials.

---

## Domain Events

There are **no domain events**. The system is purely synchronous request-response. No event bus, no message queue, no pub/sub, no webhooks. The only "event-like" behavior is:

1. **SSE client disconnect** -- `req.on('close', ...)` in `server.ts:171` removes the transport from the active map. This is infrastructure, not domain.
2. **Console error logging** -- errors are logged but not emitted as events.

---

## Relationships

### R-2.01: Credential belongs-to ServiceNamespace

Every credential is stored under `SERVICE_NAME = 'serveMyAPI'`. This is a 1:N relationship (one namespace, many credentials). The namespace is implicit and hardcoded -- there is no Namespace entity.

### R-2.02: KeychainService owns StorageBackend (conditional)

```
KeychainService --[IS_DOCKER=false]--> keytar (native OS keyring)
KeychainService --[IS_DOCKER=true]---> filesystem (/app/data/*.key)
```

This is a compile-time-fixed strategy selection. Both backends have the same behavioral interface (store, get, delete, list) but are not behind a shared interface or abstract class.

### R-2.03: Transport depends-on KeychainService (singleton)

All three TypeScript entry points import the same singleton:
```
index.ts   --import--> keychainService (default export)
server.ts  --import--> keychainService (default export)
cli.ts     --import--> keychain (default export, aliased)
```

The singleton means all transports share state (permission marker flag).

### R-2.04: cli.js spawns index.ts as subprocess

The MCP client CLI (`cli.js`) does not import `keychainService` directly. It spawns a server process and communicates via MCP stdio transport. This is a process-level dependency, not a module dependency.

---

## State Machines

### SM-2.01: Credential Lifecycle

```
                    store(name, key)
    [NonExistent] ==================> [Exists]
         ^                               |
         |       delete(name)            |
         +===============================+
                                         |
                    store(name, key2)     |
                  [Exists] <============-+
                  (overwrite, same state)
```

States: `{NonExistent, Exists}`. Transitions: `{store, delete}`. Store is idempotent on existence (always moves to Exists). Delete from NonExistent returns false but does not error.

### SM-2.02: Permission Marker Lifecycle

```
    [Unchecked] --constructor()--> [Checking]
                                       |
                    +--[marker exists]--+--[marker absent]--+
                    |                                       |
                    v                                       v
              [Authorized]                         [Creating Marker]
                                                        |
                                                        v
                                                  [Authorized]
```

The `hasStoredPermissionMarker` boolean flag tracks this. Once true, it never reverts. The "belt-and-suspenders" pattern means each CRUD method re-checks if the flag is still false:

```typescript
if (!this.hasStoredPermissionMarker) {
    await this.checkPermissionMarker();
}
```

### SM-2.03: SSE Transport Session (server.ts only)

```
    [No Connection] --GET /sse--> [Connected]
                                      |
                   --client close---> [Disconnected]
                                      (removed from activeTransports)
```

No reconnection logic. No session persistence.

---

## Ubiquitous Language Glossary

| Term | Meaning in Codebase | Notes |
|------|---------------------|-------|
| API key | A secret string (token, key, password) stored by name | Despite the name, any secret string can be stored |
| Key name | The unique identifier for a stored credential | Used as keytar "account" and as Docker filename |
| Keychain | The OS-level secure credential store (macOS Keychain, etc.) | Used loosely -- also refers to file-based fallback |
| Service | The keytar namespace grouping (`serveMyAPI`) | Not an application service -- a keyring grouping |
| Permission marker | Sentinel credential that triggers Keychain authorization | Invisible to users, filtered from list results |
| Store | Create or overwrite a credential | No distinction between create and update |
| Docker mode | File-based storage fallback for containerized environments | Activated by `DOCKER_ENV=true` |

---

## Bounded Context Map

```
+-----------------------------------------------------------+
|                  Credential Management                     |
|                  (sole bounded context)                    |
|                                                            |
|  Entities: Credential (implicit), PermissionMarker         |
|  Operations: store, get, delete, list                      |
|  Storage: keytar OR filesystem                             |
|                                                            |
|  No upstream/downstream contexts                           |
|  No domain events                                          |
|  No integration with external systems                      |
+-----------------------------------------------------------+
```

The MCP protocol is a **transport concern**, not a bounded context. The CLI is an alternative interface to the same context.

---

## Findings Beyond Broad Sweep

1. **VO-2.05 (MCP Response Shape)** was not identified as a value object in the broad sweep. It is a consistent pattern across all 8 tool handler instances (4 in index.ts, 4 in server.ts) and defines the domain-level response contract.

2. **VO-2.07 (SSE Session ID)** was not cataloged. Its quality issues were noted in the broad sweep architecture section but not as a domain concept.

3. **R-2.04 (cli.js spawns server subprocess)** was mentioned but not explicitly modeled as a relationship. This is a distinct integration pattern from the direct-import pattern used by the other three entry points.

4. **SM-2.02 (Permission Marker Lifecycle)** was partially described in the broad sweep but not drawn as a state machine. The "belt-and-suspenders" re-check pattern in every CRUD method is a notable defensive coding choice.

5. **E-2.03/E-2.04 (Server/Client identity)** -- the MCP server name and version are duplicated across files with no shared constant. The client has a different name.

6. **The `add` and `remove` CLI aliases** (cli.ts lines 42, 59) are not reflected in the MCP tool names. `store` in CLI maps to `store-api-key` in MCP. `add` is a CLI-only alias. `remove` is a CLI-only alias for `delete`. This vocabulary mismatch between interfaces was not noted.

7. **The Docker Dockerfile uses stdio entry point** (`CMD ["node", "dist/index.js"]`) **but the HEALTHCHECK uses HTTP** (`curl -f http://localhost:3000/`). This is contradictory -- the stdio server does not listen on port 3000. The healthcheck will always fail unless the container is actually running `server.ts`.

---

## Delta Summary
- New items added: 3 value objects (VO-2.05, VO-2.06, VO-2.07), 2 entity refinements (E-2.03, E-2.04), 1 relationship (R-2.04 formalized), 1 state machine (SM-2.02 fully drawn), 1 Dockerfile contradiction discovered
- Existing items refined: E-2.01 (added missing property recommendations), SM-2.01 (added idempotency note), Glossary (6 new terms)
- Remaining gaps: Whether keytar `findCredentials` returns credentials in any guaranteed order (affects list output determinism)

## Novelty Assessment
Novelty: SUBSTANTIVE
The Dockerfile healthcheck contradiction (Finding 7), the CLI alias vocabulary mismatch (Finding 6), the MCP response shape as a value object (Finding 1), and the full permission marker state machine (Finding 4) all change how you would spec the system. The Dockerfile issue is a functional bug that must be addressed in any port.

## Convergence Declaration
Another round needed -- want to verify the keytar ordering behavior and check whether the `build_dmg.sh` reveals any additional domain concepts around app bundling identity.

## State Checkpoint
```yaml
pass: 2
round: 1
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
```
