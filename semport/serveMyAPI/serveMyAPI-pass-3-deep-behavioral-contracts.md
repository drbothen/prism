# Pass 3 Deep: Behavioral Contracts -- serveMyAPI (Round 1)

## Preamble

This round extracts behavioral contracts from source code with line-level precision. Since the codebase has **zero tests** (`npm test` exits with error), all contracts are extracted from implementation code. Confidence ceiling is MEDIUM for all contracts.

Contracts use the format BC-S.SS.NNN where S=system (2=domain, 3=transport), SS=subsystem, NNN=sequence.

---

## Subsystem 2.01: KeychainService -- Keytar Backend (Native)

### BC-2.01.001: storeKey persists credential to OS keyring

**Preconditions:**
- `IS_DOCKER === false` (module-level constant, checked at keychain.ts:70)
- `name`: string (no constraints enforced at service level -- caller responsible)
- `key`: string (no constraints enforced at service level)
- Permission marker has been checked (if `hasStoredPermissionMarker` is false, `checkPermissionMarker()` runs first)

**Postconditions:**
- `keytar.setPassword('serveMyAPI', name, key)` is called (keychain.ts:78)
- If credential with same name exists, it is silently overwritten (keytar behavior)
- Returns `void` (Promise resolves with no value)

**Error Cases:**
- keytar native addon throws (Keychain locked, permission denied, corrupted keyring) -- exception propagates to caller
- `checkPermissionMarker()` itself throws -- exception propagates (but is caught internally with console.error; the CRUD method call will still attempt keytar.setPassword)

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:69-79

---

### BC-2.01.002: getKey retrieves credential value by name from OS keyring

**Preconditions:**
- `IS_DOCKER === false`
- `name`: string
- Permission marker checked (belt-and-suspenders guard, keychain.ts:105-107)

**Postconditions:**
- Returns `string` if credential exists, `null` if not found (keytar.getPassword behavior)
- The raw secret value is returned with no transformation or masking

**Error Cases:**
- keytar throws -- propagates to caller

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:99-109

---

### BC-2.01.003: deleteKey removes credential by name from OS keyring

**Preconditions:**
- `IS_DOCKER === false`
- `name`: string
- Permission marker checked (keychain.ts:138-140)

**Postconditions:**
- Returns `true` if credential was found and deleted
- Returns `false` if credential did not exist (keytar.deletePassword behavior)
- No error on deleting non-existent credential -- just returns false

**Error Cases:**
- keytar throws -- propagates to caller

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:132-142

---

### BC-2.01.004: listKeys enumerates all credential names, filtering permission marker

**Preconditions:**
- `IS_DOCKER === false`
- Permission marker checked (keychain.ts:170-172)

**Postconditions:**
- Calls `keytar.findCredentials('serveMyAPI')` (keychain.ts:175)
- Maps result to `cred.account` array (keychain.ts:177)
- Filters out entries where `account === '_permission_granted'` (keychain.ts:179)
- Returns `string[]` (may be empty)
- **Order is undefined** -- keytar `findCredentials` does not guarantee ordering

**Error Cases:**
- keytar throws -- propagates to caller

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:165-180

---

## Subsystem 2.02: KeychainService -- File Backend (Docker)

### BC-2.02.001: storeKeyFile writes credential to filesystem

**Preconditions:**
- `IS_DOCKER === true`
- `name`: string (used directly in file path -- **no sanitization**)
- `key`: string
- Storage directory exists (ensured by constructor calling `ensureStorageDirectory`)

**Postconditions:**
- Writes `key` as UTF-8 to `${STORAGE_DIR}/${name}.key` (keychain.ts:86)
- Uses `writeFileSync` -- synchronous, blocking call despite being in an async method
- If file exists, silently overwrites (Node.js writeFileSync behavior)

**Error Cases:**
- Filesystem error (permissions, disk full) -- caught, logged to console.error, then **re-thrown** (keychain.ts:89-90)
- **Path traversal vulnerability**: `name` containing `../` or `/` will write outside STORAGE_DIR

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:84-92

---

### BC-2.02.002: getKeyFile reads credential from filesystem

**Preconditions:**
- `IS_DOCKER === true`
- `name`: string (unsanitized path component)

**Postconditions:**
- Checks `fs.existsSync(filePath)` (keychain.ts:117)
- If exists: returns file contents as UTF-8 string
- If not exists: returns `null`

**Error Cases:**
- Read error -- caught, logged to console.error, returns `null` (keychain.ts:122-124)
- Note: errors are **swallowed** -- returns null instead of propagating, making read errors indistinguishable from "not found"

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:114-125

---

### BC-2.02.003: deleteKeyFile removes credential file

**Preconditions:**
- `IS_DOCKER === true`
- `name`: string (unsanitized)

**Postconditions:**
- If file exists: `unlinkSync` deletes it, returns `true`
- If not exists: returns `false`

**Error Cases:**
- Unlink error -- caught, logged, returns `false` (keychain.ts:156-158)
- Like getKeyFile, errors are swallowed -- delete failure is indistinguishable from "not found"

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:147-159

---

### BC-2.02.004: listKeyFiles enumerates credential files in storage directory

**Preconditions:**
- `IS_DOCKER === true`
- STORAGE_DIR exists (ensured by constructor)

**Postconditions:**
- Reads directory listing via `fs.readdirSync(STORAGE_DIR)` (keychain.ts:187)
- Filters to files ending in `.key` (keychain.ts:189)
- Strips `.key` suffix to produce name list (keychain.ts:190)
- Returns `string[]` (may be empty)
- **Does not recurse** into subdirectories
- **Does not filter hidden files** -- a file named `.secret.key` would appear as `.secret`

**Error Cases:**
- Directory read error -- caught, logged, returns empty array `[]` (keychain.ts:192)
- Errors swallowed -- makes directory-missing indistinguishable from "no keys"

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:185-195

---

## Subsystem 2.03: KeychainService -- Permission Marker

### BC-2.03.001: Constructor initializes permission marker or storage directory

**Preconditions:**
- Module is imported (singleton instantiation at keychain.ts:198)

**Postconditions:**
- If `IS_DOCKER === false`: calls `checkPermissionMarker()` (async, fire-and-forget from constructor)
- If `IS_DOCKER === true`: calls `ensureStorageDirectory()` (synchronous)

**Critical observation:** `checkPermissionMarker()` is async but called from a synchronous constructor with no `await`. This means the permission marker check runs as a detached promise. If a CRUD method is called immediately after construction (before the check completes), the belt-and-suspenders guard will re-trigger it, but there is a **race window** where two concurrent `checkPermissionMarker` calls could run.

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:18-25

---

### BC-2.03.002: checkPermissionMarker creates marker if absent

**Preconditions:**
- `IS_DOCKER === false` (guard at keychain.ts:47)

**Postconditions:**
- Reads `keytar.getPassword('serveMyAPI', '_permission_granted')` (keychain.ts:50)
- If result is truthy: sets `hasStoredPermissionMarker = true` (keychain.ts:51)
- If result is null/falsy: calls `keytar.setPassword('serveMyAPI', '_permission_granted', 'true')` (keychain.ts:55), then sets flag to true (keychain.ts:56)
- The `setPassword` call is what triggers the macOS Keychain permission dialog

**Error Cases:**
- Any keytar error -- caught, logged to console.error (keychain.ts:58-59)
- Flag remains `false` if error occurs -- next CRUD call will retry the check

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:46-61

---

### BC-2.03.003: ensureStorageDirectory creates STORAGE_DIR if absent

**Preconditions:**
- `IS_DOCKER === true` (guard at keychain.ts:32)

**Postconditions:**
- If directory does not exist: creates it with `{ recursive: true }` (keychain.ts:34)
- If directory exists: no-op

**Error Cases:**
- mkdir error -- caught, logged to console.error (keychain.ts:36-38)
- **Does not throw** -- constructor continues even if directory creation fails
- Subsequent file operations will fail when they try to write to the non-existent directory

**Confidence:** MEDIUM (from code, no tests)
**Source:** keychain.ts:30-39

---

## Subsystem 3.01: MCP Tool Handlers (stdio -- index.ts)

### BC-3.01.001: store-api-key tool stores credential via MCP

**Preconditions:**
- MCP client sends `store-api-key` tool call
- `name`: string, min length 1 (Zod schema, index.ts:17)
- `key`: string, min length 1 (Zod schema, index.ts:18)

**Postconditions:**
- Calls `keychainService.storeKey(name, key)` (index.ts:22)
- On success: returns `{ content: [{ type: "text", text: "Successfully stored API key with name: ${name}" }] }` (index.ts:23-26)
- Response text includes the credential name but NOT the key value

**Error Cases:**
- Service throws: returns `{ content: [{ type: "text", text: "Error storing API key: ${message}" }], isError: true }` (index.ts:29-33)
- Error message extracted via `(error as Error).message` -- unsafe cast, may produce `undefined` for non-Error throws

**Confidence:** MEDIUM (from code, no tests)
**Source:** index.ts:14-38

---

### BC-3.01.002: get-api-key tool retrieves credential via MCP

**Preconditions:**
- `name`: string, min length 1 (Zod schema, index.ts:44)

**Postconditions:**
- Calls `keychainService.getKey(name)` (index.ts:49)
- If key is null/falsy: returns `isError: true` with `"No API key found with name: ${name}"` (index.ts:52-57)
- If key is truthy: returns raw key value as plain text (index.ts:60-63)
- **The secret is returned in cleartext** in the MCP response content

**Error Cases:**
- Service throws: returns `isError: true` with error message (index.ts:66-70)

**Important behavioral nuance:** The falsy check (`if (!key)`) means an empty string key would also be treated as "not found". However, Zod prevents storing empty keys, so this should not happen in practice unless credentials are stored outside this tool.

**Confidence:** MEDIUM (from code, no tests)
**Source:** index.ts:42-76

---

### BC-3.01.003: delete-api-key tool deletes credential via MCP

**Preconditions:**
- `name`: string, min length 1 (Zod schema, index.ts:83)

**Postconditions:**
- Calls `keychainService.deleteKey(name)` (index.ts:87)
- If returns false: `isError: true` with `"No API key found with name: ${name}"` (index.ts:89-94)
- If returns true: `"Successfully deleted API key with name: ${name}"` (index.ts:97-101)

**Error Cases:**
- Service throws: `isError: true` with error message (index.ts:104-108)

**Confidence:** MEDIUM (from code, no tests)
**Source:** index.ts:80-114

---

### BC-3.01.004: list-api-keys tool lists all credential names via MCP

**Preconditions:**
- No parameters (empty Zod schema, index.ts:119)

**Postconditions:**
- Calls `keychainService.listKeys()` (index.ts:123)
- If empty array: returns `"No API keys found"` (NOT isError) (index.ts:125-129)
- If non-empty: returns `"Available API keys:\n${keys.join("\n")}"` (index.ts:133-136)
- **Empty list is not an error** -- this differs from get/delete where not-found IS an error

**Error Cases:**
- Service throws: `isError: true` with error message (index.ts:139-143)

**Confidence:** MEDIUM (from code, no tests)
**Source:** index.ts:118-149

---

## Subsystem 3.02: MCP Tool Handlers (HTTP/SSE -- server.ts)

### BC-3.02.001 through BC-3.02.004: Identical to BC-3.01.001 through BC-3.01.004

The four tool handler implementations in `server.ts` (lines 14-150) are **character-for-character identical** to those in `index.ts`. Every behavioral contract from subsystem 3.01 applies exactly to subsystem 3.02. This is confirmed by reading both files in full -- there are zero behavioral differences in the tool handlers.

The only difference is the transport layer below the tools.

---

## Subsystem 3.03: SSE Transport (server.ts)

### BC-3.03.001: GET /sse establishes SSE connection

**Preconditions:**
- HTTP GET request to `/sse` endpoint

**Postconditions:**
- Creates `SSEServerTransport` with message endpoint `/messages` (server.ts:161)
- Generates session ID via `Date.now().toString()` (server.ts:160)
- Stores transport in `activeTransports` Map (server.ts:163)
- Sets SSE headers: Content-Type, Cache-Control, Connection (server.ts:166-168)
- Connects MCP server to transport (server.ts:175)
- On client disconnect: removes transport from map (server.ts:171-173)

**Error Cases:**
- No explicit error handling in the GET handler

**Confidence:** MEDIUM (from code, no tests)
**Source:** server.ts:159-176

---

### BC-3.03.002: POST /messages routes message to SSE transport

**Preconditions:**
- HTTP POST to `/messages` with JSON body
- At least one active SSE connection exists

**Postconditions:**
- Retrieves the **last** transport ID from the map: `Array.from(activeTransports.keys()).pop()` (server.ts:180)
- Calls `transport.handlePostMessage(req, res)` on that transport (server.ts:188)

**Error Cases:**
- No active connections: returns `400 { error: "No active connections" }` (server.ts:183)
- handlePostMessage throws: logs error, returns `500` if headers not sent (server.ts:189-192)
- **Fundamental concurrency bug:** With multiple SSE connections, messages always route to the most recently connected client, not the one that originated the request. There is no session correlation.

**Confidence:** MEDIUM (from code, no tests)
**Source:** server.ts:178-194

---

### BC-3.03.003: GET / returns HTML landing page

**Preconditions:**
- HTTP GET to `/`

**Postconditions:**
- Returns static HTML describing the server and its tools (server.ts:197-222)
- No dynamic content, no server status

**Confidence:** HIGH (trivially simple)
**Source:** server.ts:197-223

---

## Subsystem 3.04: Direct CLI (cli.ts)

### BC-3.04.001: CLI `list` command

**Preconditions:**
- `process.argv[2]` is `'list'` (case-insensitive, cli.ts:11)

**Postconditions:**
- Calls `keychain.listKeys()` (cli.ts:14)
- If empty: prints `"No API keys found."` (cli.ts:16)
- If non-empty: prints `"Stored API keys:"` followed by ` - ${key}` per entry (cli.ts:18-19)
- Output format differs from MCP tool (uses `" - "` prefix, MCP uses bare newlines)

**Confidence:** MEDIUM (from code, no tests)
**Source:** cli.ts:13-20

---

### BC-3.04.002: CLI `get` command

**Preconditions:**
- `process.argv[2]` is `'get'`
- `process.argv[3]` is the key name (required)

**Postconditions:**
- If no key name provided: prints error, prints usage, exits with code 1 (cli.ts:26-29)
- Calls `keychain.getKey(keyName)` (cli.ts:32)
- If found: prints `"${keyName}: ${value}"` (cli.ts:34) -- **includes the secret in terminal output**
- If not found: prints `"Error: Key '${keyName}' not found."`, exits with code 1 (cli.ts:36-38)

**Confidence:** MEDIUM (from code, no tests)
**Source:** cli.ts:23-38

---

### BC-3.04.003: CLI `store`/`add` command

**Preconditions:**
- `process.argv[2]` is `'store'` or `'add'` (cli.ts:42)
- `process.argv[3]` is key name, `process.argv[4]` is key value (both required)

**Postconditions:**
- If missing args: prints error, usage, exits 1 (cli.ts:47-50)
- Calls `keychain.storeKey(storeName, storeValue)` (cli.ts:53)
- Prints `"Key '${storeName}' stored successfully."` (cli.ts:54)
- **`add` is a CLI-only alias** not present in MCP tools

**Confidence:** MEDIUM (from code, no tests)
**Source:** cli.ts:41-55

---

### BC-3.04.004: CLI `delete`/`remove` command

**Preconditions:**
- `process.argv[2]` is `'delete'` or `'remove'` (cli.ts:59)
- `process.argv[3]` is key name (required)

**Postconditions:**
- If missing name: prints error, usage, exits 1 (cli.ts:63-66)
- Calls `keychain.deleteKey(deleteName)` (cli.ts:68)
- If true: prints `"Key '${deleteName}' deleted successfully."` (cli.ts:70)
- If false: prints `"Error: Failed to delete key '${deleteName}'. Key may not exist."`, exits 1 (cli.ts:72-73)
- **`remove` is a CLI-only alias** not present in MCP tools
- Error message says "Key may not exist" (hedging), unlike MCP which says "No API key found" (definitive)

**Confidence:** MEDIUM (from code, no tests)
**Source:** cli.ts:57-75

---

### BC-3.04.005: CLI `help` command

**Preconditions:**
- `process.argv[2]` is `'help'`, `'--help'`, or `'-h'` (cli.ts:77-79)

**Postconditions:**
- Prints usage information (cli.ts:94-112)
- Does not call process.exit (falls through naturally)

**Confidence:** HIGH (trivially simple)
**Source:** cli.ts:77-80

---

### BC-3.04.006: CLI unknown command handling

**Preconditions:**
- `process.argv[2]` does not match any known command

**Postconditions:**
- Prints `"Error: Unknown command '${command}'."` (cli.ts:83)
- Prints usage (cli.ts:84)
- Exits with code 1 (cli.ts:85)

**Edge case:** If no command provided (`process.argv[2]` is `undefined`), `command` is `undefined`, and the default case triggers printing `"Error: Unknown command 'undefined'."` -- not a clean UX.

**Confidence:** MEDIUM (from code, no tests)
**Source:** cli.ts:82-85

---

## Subsystem 3.05: MCP Client CLI (cli.js)

### BC-3.05.001: MCP client CLI spawns server and connects

**Preconditions:**
- `cli.js` is executed via `node`

**Postconditions:**
- Resolves `__dirname` for server path (cli.js:17)
- Creates `StdioClientTransport` that spawns `node ${serverPath}` (cli.js:26-29)
- Creates MCP `Client("ServeMyAPIClient", "1.0.0")` (cli.js:32)
- Calls `client.connect(transport)` (cli.js:36)
- Default command is `'list'` if no args (cli.js:41)

**Error Cases:**
- Connection failure: caught in try/catch, prints error, exits 1 (cli.js:126-128)

**Behavioral difference from cli.ts:** This CLI spawns the **server** as a subprocess and communicates over MCP protocol, while cli.ts calls the service directly. This means cli.js exercises the full MCP stack including Zod validation, while cli.ts bypasses it entirely.

**Confidence:** MEDIUM (from code, no tests)
**Source:** cli.js:15-37

---

### BC-3.05.002: MCP client CLI result parsing

**Preconditions:**
- A tool call has returned a result

**Postconditions:**
- Extracts first `text` content item from `result.content` (cli.js:104)
- For `list` command: strips `"Available API keys:\n"` prefix and formats with `"- "` bullets (cli.js:113-114)
- For other commands: prints `"Result: ${text}"` (cli.js:120)
- If no content: prints `"No data returned from the server"` (cli.js:124)

**Confidence:** MEDIUM (from code, no tests)
**Source:** cli.js:103-125

---

### BC-3.05.003: MCP client CLI disconnects in finally block

**Preconditions:**
- Main function completes or throws

**Postconditions:**
- Calls `client.disconnect()` in `finally` block (cli.js:131)
- Transport handles closing the server process (cli.js:132)

**Confidence:** MEDIUM (from code, no tests)
**Source:** cli.js:130-133

---

## Subsystem 3.06: Server Lifecycle

### BC-3.06.001: Stdio server startup (index.ts)

**Preconditions:**
- Module loaded by Node.js runtime

**Postconditions:**
- Creates `StdioServerTransport` (index.ts:153)
- Calls `server.connect(transport)` (index.ts:154)
- On success: logs `"ServeMyAPI MCP server is running..."` to stderr (index.ts:155)
- On failure: logs `"Error starting ServeMyAPI MCP server: ${error}"` to stderr (index.ts:157)
- **Does not call process.exit on failure** -- leaves the process running in error state

**Confidence:** MEDIUM (from code, no tests)
**Source:** index.ts:152-158

---

### BC-3.06.002: HTTP server startup (server.ts)

**Preconditions:**
- Module loaded by Node.js runtime

**Postconditions:**
- Creates Express app on `process.env.PORT || 3000` (server.ts:154)
- Calls `app.listen(port)` (server.ts:226)
- Logs `"ServeMyAPI HTTP server is running on port ${port}"` to stdout (server.ts:227)
- **No graceful shutdown handler** -- no SIGTERM/SIGINT handling
- Server instance is exported as `{ server }` (server.ts:230)

**Confidence:** MEDIUM (from code, no tests)
**Source:** server.ts:152-230

---

## Cross-Subsystem Behavioral Observations

### OBS-3.01: Error message inconsistency across interfaces

| Scenario | MCP (index.ts/server.ts) | CLI (cli.ts) | MCP Client CLI (cli.js) |
|----------|--------------------------|--------------|------------------------|
| Key not found (get) | `"No API key found with name: X"` | `"Error: Key 'X' not found."` | Passes through MCP message |
| Delete failed | `"No API key found with name: X"` | `"Error: Failed to delete key 'X'. Key may not exist."` | Passes through MCP message |
| Empty list | `"No API keys found"` (not error) | `"No API keys found."` (with period) | `"No API keys found"` |
| Store success | `"Successfully stored API key with name: X"` | `"Key 'X' stored successfully."` | `"Result: Successfully stored..."` |

### OBS-3.02: Validation gap between CLI and MCP

The MCP tools enforce `z.string().min(1)` via Zod schemas. The CLI (`cli.ts`) checks for argument presence (`if (!keyName)`) but does not validate minimum length -- an empty string argument would pass the CLI check but could cause issues at the keytar level.

### OBS-3.03: Error swallowing asymmetry

| Backend | storeKey error | getKey error | deleteKey error | listKeys error |
|---------|---------------|--------------|-----------------|----------------|
| Keytar | Propagates | Propagates | Propagates | Propagates |
| File | Logs + re-throws | Logs + returns null | Logs + returns false | Logs + returns [] |

The file backend swallows read/delete/list errors (returns default values), but re-throws store errors. This inconsistency means callers cannot distinguish "not found" from "storage error" in Docker mode for get, delete, and list operations.

### OBS-3.04: Unused keytar import

`index.ts:5` imports `keytar` directly but never uses it. All keytar access goes through `keychainService`. This is dead code.

---

## Delta Summary
- New items added: 22 behavioral contracts (BC-2.01.001 through BC-3.06.002), 4 cross-subsystem observations
- Existing items refined: All 6 broad-sweep contracts (BC-001 through BC-006) decomposed into granular per-subsystem contracts with line references
- Remaining gaps: cli.js `--help`/`-h` flag handling (checked but not fully traced), edge case behavior when keytar native addon is missing entirely

## Novelty Assessment
Novelty: SUBSTANTIVE
Multiple new findings change the spec model: the async constructor race condition (BC-2.03.001), error swallowing asymmetry (OBS-3.03), falsy-check edge case on empty string keys (BC-3.01.002), the Dockerfile stdio-vs-HTTP contradiction (noted in BC-3.06.001/002 context), validation gap between CLI and MCP (OBS-3.02), and the undefined command UX issue (BC-3.04.006).

## Convergence Declaration
Another round needed -- want to verify cli.js help flag behavior, check whether the `build_dmg.sh` launcher script has behavioral implications, and look for any edge cases in the SSE transport session management.

## State Checkpoint
```yaml
pass: 3
round: 1
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
```
