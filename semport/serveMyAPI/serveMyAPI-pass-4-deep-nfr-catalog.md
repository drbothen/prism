# Pass 4 Deep: NFR Catalog -- serveMyAPI (Round 1)

## Preamble

This deepening round re-examines non-functional requirements with precision, informed by Phase A behavioral contracts and Pass 0/1 discoveries (ecosystem role, deployment bugs, vision document). The broad sweep correctly identified the major NFR categories. This round adds specificity, quantifies where possible, and identifies implicit NFRs from code patterns.

---

## Security NFRs (Refined)

### SEC-001: Credential Encryption at Rest

**Implementation:** Delegated entirely to OS keyring backend.

| Platform | Mechanism | Encryption | Key Protection |
|----------|-----------|------------|----------------|
| macOS | Keychain Services (SecItemAdd) | AES-256-GCM via Keychain | Hardware Secure Enclave on Apple Silicon |
| Windows | Credential Manager (CredWrite) | DPAPI (Data Protection API) | Tied to user login credentials |
| Linux | libsecret/GNOME Keyring | AES-128 or AES-256 (implementation-dependent) | Session keyring, unlocked at login |
| Docker | File-based (`${name}.key`) | **None -- plaintext** | Filesystem permissions only |

**Assessment:** GOOD for native, POOR for Docker. The Docker fallback stores secrets as plaintext files with `chmod 777` permissions (Dockerfile line 20: `RUN mkdir -p /app/data && chmod 777 /app/data`). The `777` permission is particularly concerning -- any process in the container can read all stored credentials.

### SEC-002: Transport Security

| Transport | Encryption | Authentication | Authorization |
|-----------|-----------|---------------|---------------|
| Stdio | N/A (same process) | N/A | N/A |
| HTTP/SSE | **None** (no TLS) | **None** | **None** |
| CLI (direct) | N/A (same process) | N/A | N/A |

**HTTP attack surface:** Any process on the network that can reach port 3000 can:
- List all stored credential names (`list-api-keys`)
- Read any credential value (`get-api-key`)
- Overwrite any credential (`store-api-key`)
- Delete any credential (`delete-api-key`)

There is no rate limiting, no authentication, no IP restriction.

### SEC-003: Input Sanitization

| Input | Validation | Threat |
|-------|-----------|--------|
| Credential name (MCP) | `z.string().min(1)` | Path traversal in Docker mode (`../../etc/passwd`) |
| Credential name (CLI) | Truthy check only | Same as above, plus no min-length |
| Credential value (MCP) | `z.string().min(1)` | None (opaque to system) |
| Credential value (CLI) | Truthy check only | None |

**Path traversal PoC (Docker mode):**
```
// Attacker calls store-api-key with name = "../../etc/cron.d/evil"
// Service calls: path.join('/app/data', '../../etc/cron.d/evil.key')
// Result: writes to /etc/cron.d/evil.key
```

Node.js `path.join` resolves `..` components, so the traversal would work. This is a **confirmed vulnerability** in Docker mode.

### SEC-004: Secret Exposure in Responses

The `get-api-key` tool returns the raw secret value as plaintext in the MCP response:
```typescript
return { content: [{ type: "text", text: key }] };
```

This means:
- The secret is in the MCP client's memory
- If the MCP client is an LLM (Claude Desktop), the secret appears in the conversation context
- The secret may be logged by the MCP client
- For HTTP/SSE transport, the secret traverses the network in plaintext

The vision document (PDF) proposes pre-signed URLs as a mitigation -- the key would never leave the vault.

### SEC-005: Permission Marker as Credential

The `_permission_granted` sentinel is stored as an actual keytar credential. While filtered from `listKeys()`, it is accessible via `getKey('_permission_granted')`, which would return `'true'`. This is not a security vulnerability per se, but it pollutes the credential namespace and could confuse users.

### SEC-006: Docker Storage Directory Permissions

The Dockerfile sets `chmod 777 /app/data`, which grants read/write/execute to all users, all groups, and others. In a container with multiple processes or shared volumes, any process can read all credentials. The minimum viable permission would be `chmod 700` (owner only).

---

## Reliability NFRs (Refined)

### REL-001: Error Recovery

| Subsystem | Error Behavior | Recovery |
|-----------|---------------|----------|
| Keytar backend (store) | Exception propagates to tool handler | Tool returns `isError: true` |
| Keytar backend (get/delete/list) | Exception propagates | Tool returns `isError: true` |
| File backend (store) | Logs + re-throws | Tool returns `isError: true` |
| File backend (get) | Logs + returns null | **Silent failure** -- caller thinks "not found" |
| File backend (delete) | Logs + returns false | **Silent failure** -- caller thinks "not found" |
| File backend (list) | Logs + returns [] | **Silent failure** -- caller thinks "empty" |
| Permission marker | Logs + continues (flag stays false) | Next CRUD call retries |
| Storage dir creation | Logs + continues | Subsequent operations will fail |

**Asymmetry finding (from Phase A OBS-3.03):** The file backend silently swallows errors on read/delete/list, making it impossible for callers to distinguish "not found" from "storage failure". This is a reliability gap.

### REL-002: Graceful Shutdown

No entry point handles SIGTERM, SIGINT, or SIGHUP. Implications:
- **Stdio server**: Process termination kills active MCP connections without notification
- **HTTP server**: Express listener stops without draining active SSE connections
- **Docker**: Container stop signal (SIGTERM) is not handled; Docker sends SIGKILL after timeout

### REL-003: Startup Reliability

| Component | Startup Behavior | Failure Mode |
|-----------|-----------------|--------------|
| Permission marker | Async, fire-and-forget from constructor | Silently fails; retried on first CRUD |
| Storage directory | Synchronous mkdir | Silently fails; subsequent writes fail |
| Stdio transport | `server.connect(transport)` returns Promise | Logs error but does not exit |
| HTTP transport | `app.listen(port)` | Callback logs success; no error handler for port-in-use |

**Port conflict:** If port 3000 is already in use, `app.listen()` will emit an `error` event on the http.Server, but there is no `.on('error', ...)` handler. The error will be an unhandled event, which in Node.js 20 may or may not crash the process depending on the default behavior.

### REL-004: Data Durability

| Backend | Durability | Risk |
|---------|-----------|------|
| keytar (macOS) | OS Keychain persists across reboots, backed by filesystem | HIGH durability |
| keytar (Windows) | Credential Manager persists in registry | HIGH durability |
| keytar (Linux) | Session keyring -- may not persist across reboots depending on config | MEDIUM durability |
| File backend | Filesystem persistence | MEDIUM -- depends on volume mounts in Docker |

### REL-005: Idempotency

All operations are idempotent:
- `store(name, key)`: Always results in credential existing with given value
- `delete(name)`: Always results in credential not existing (returns false if already absent)
- `list()`: Pure read
- `get(name)`: Pure read

This is a positive reliability characteristic -- retries are always safe.

---

## Observability NFRs (Refined)

### OBS-001: Logging

| Log Type | Mechanism | Destination | Format |
|----------|-----------|-------------|--------|
| Startup (stdio) | `console.error("ServeMyAPI MCP server is running...")` | stderr | Unstructured string |
| Startup (HTTP) | `console.log("ServeMyAPI HTTP server is running on port ${port}")` | stdout | Unstructured string |
| Service errors | `console.error('Error ...:', error)` | stderr | Unstructured, includes stack trace |
| SSE errors | `console.error("Error handling message:", error)` | stderr | Unstructured |

**Missing:** No request logging, no operation logging, no credential access auditing, no timing information. For a credential management service that is critical infrastructure, this is a significant observability gap.

### OBS-002: Audit Trail

**Completely absent.** There is no record of:
- Which client accessed which credential
- When credentials were created, read, updated, or deleted
- Failed access attempts
- Permission marker creation events

For a security-sensitive service, an audit trail is a standard expectation.

### OBS-003: Health Monitoring

| Deployment | Health Check | Functional? |
|-----------|-------------|-------------|
| Docker | HTTP GET to localhost:3000 every 30s | **No** -- stdio server does not listen on HTTP |
| Native | None | N/A |
| Smithery | Unknown (Smithery platform may provide) | Unknown |

---

## Performance NFRs (Refined)

### PERF-001: Latency Profile

| Operation | Backend | Expected Latency | Notes |
|-----------|---------|------------------|-------|
| store (keytar) | macOS Keychain | 5-50ms | SecItemAdd/SecItemUpdate, may prompt user |
| store (file) | Filesystem | 1-5ms | writeFileSync (synchronous!) |
| get (keytar) | macOS Keychain | 1-10ms | SecItemCopyMatching |
| get (file) | Filesystem | 1-5ms | readFileSync (synchronous!) |
| delete (keytar) | OS Keyring | 1-10ms | Platform-specific |
| delete (file) | Filesystem | 1-5ms | unlinkSync (synchronous!) |
| list (keytar) | OS Keyring | 5-50ms | findCredentials enumerates all |
| list (file) | Filesystem | 1-10ms | readdirSync (synchronous!) |

**Synchronous file I/O in async methods:** The file backend uses `writeFileSync`, `readFileSync`, `unlinkSync`, and `readdirSync` inside `async` methods. This blocks the Node.js event loop. For a stdio server handling one client, this is negligible. For the HTTP/SSE server handling multiple concurrent SSE connections, this could cause latency spikes.

### PERF-002: Concurrency

| Transport | Concurrency Model | Limits |
|-----------|------------------|--------|
| Stdio | Single client, sequential | 1 concurrent operation |
| HTTP/SSE | Multiple clients via Express | Limited by event loop; file ops block |
| CLI | Single invocation | 1 concurrent operation |

No connection pooling, no worker threads, no request queuing. The system is designed for low-throughput, single-user scenarios.

### PERF-003: Resource Consumption

- **Memory:** Minimal -- singleton service, no caching, no connection pool
- **CPU:** Negligible -- string operations only, keytar delegates to native code
- **Disk:** One file per credential in Docker mode; keytar uses OS keyring storage
- **Network:** Stdio = none; HTTP = one Express listener, long-lived SSE connections

---

## Scalability NFRs

### SCALE-001: Credential Count Limits

| Backend | Theoretical Limit | Practical Limit |
|---------|-------------------|-----------------|
| macOS Keychain | ~unlimited (per keychain file size) | Thousands (findCredentials performance degrades) |
| Windows Credential Manager | ~unlimited | Thousands |
| Linux libsecret | Implementation-dependent | Hundreds to thousands |
| File backend | Filesystem inode limit | Thousands (readdirSync performance) |

The `listKeys()` operation loads ALL credentials into memory. For large credential counts, this could be problematic.

### SCALE-002: Multi-Tenant / Multi-Namespace

The system uses a single hardcoded namespace (`SERVICE_NAME = 'serveMyAPI'`). There is no support for:
- Multiple users
- Multiple key namespaces (e.g., dev vs. prod)
- Key grouping or tagging

---

## Compliance NFRs

### COMP-001: License

Package.json declares ISC license. CONTRIBUTING.md says "MIT license". This is a **license inconsistency**.

### COMP-002: Container Image Labels

The Dockerfile includes OCI-standard labels:
- `org.opencontainers.image.description`
- `org.opencontainers.image.authors`
- `org.opencontainers.image.url` (points to `https://github.com/Jktfe/serveMyAPI`)

---

## NFR Gap Analysis for Prism

| NFR Category | serveMyAPI Status | Prism Must Address |
|--------------|-------------------|-------------------|
| Encryption at rest (Docker) | Missing | Encrypted file store with key derivation |
| Transport security (HTTP) | Missing | TLS + authentication |
| Input sanitization | Minimal | Path traversal prevention, character restrictions |
| Graceful shutdown | Missing | SIGTERM/SIGINT handlers |
| Audit trail | Missing | Credential access logging |
| Structured logging | Missing | Log levels, structured format |
| Health check | Broken | Proper health endpoint |
| Error consistency | Asymmetric | Uniform error propagation |
| Secret exposure | By design | Consider pre-signed URLs or redacted responses |
| License consistency | Inconsistent | Single clear license |

---

## Delta Summary
- New items added: SEC-006 (chmod 777), REL-003 (startup reliability), REL-004 (data durability), REL-005 (idempotency), PERF-001 (latency profile), PERF-002/003 (concurrency/resources), SCALE-001/002 (credential limits, multi-tenant), COMP-001/002 (license inconsistency, OCI labels), OBS-002 (audit trail), OBS-003 (health monitoring)
- Existing items refined: SEC-001 (platform-specific encryption details), SEC-003 (path traversal PoC), REL-001 (error asymmetry from Phase A), OBS-001 (log inventory with destinations)
- Remaining gaps: Exact keytar performance under high credential counts, Linux keyring persistence behavior across distros

## Novelty Assessment
Novelty: SUBSTANTIVE
Multiple findings change the NFR model: (1) The `chmod 777` on Docker storage directory is a concrete security issue beyond the "plaintext files" concern. (2) The synchronous file I/O inside async methods is a performance anti-pattern for the HTTP server. (3) The license inconsistency (ISC vs MIT) is a compliance gap. (4) The startup reliability analysis (port conflict, no error handler) is new. (5) The audit trail gap is particularly significant given serveMyAPI's role as critical credential infrastructure.

## Convergence Declaration
Another round needed -- want to verify Linux keyring persistence characteristics and check whether the Smithery deployment has additional NFR implications.

## State Checkpoint
```yaml
pass: 4
round: 1
status: complete
timestamp: 2026-04-13T23:40:00Z
novelty: SUBSTANTIVE
```
