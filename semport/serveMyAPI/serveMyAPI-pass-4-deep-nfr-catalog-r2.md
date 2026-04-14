# Pass 4 Deep: NFR Catalog -- serveMyAPI (Round 2)

## Preamble

Hallucination audit and gap closure round. Verifies Round 1 NFR claims, examines Linux keyring persistence, and checks Smithery NFR implications.

---

## Hallucination Audit

### Claim: "chmod 777 on Docker storage directory"
**Verified:** Dockerfile line 20: `RUN mkdir -p /app/data && chmod 777 /app/data`. **Confirmed.**

### Claim: "Path traversal vulnerability in Docker mode"
**Verified:** `path.join(STORAGE_DIR, `${name}.key`)` at keychain.ts:86, 116, 149. Node.js `path.join('/app/data', '../../etc/passwd.key')` resolves to `/etc/passwd.key`. No sanitization of `name` parameter exists in the service layer. Zod at the transport layer enforces `min(1)` but no character restrictions. **Confirmed -- path traversal is possible.**

However, refinement: In a Docker container running as root (which `node:20-slim` does by default unless a USER directive is added -- and the Dockerfile has NO `USER` directive), the process runs as root. Combined with `chmod 777`, this means the path traversal could write to system directories. But typical Docker containers have a limited filesystem, so the impact depends on what directories exist and what side effects file creation has.

### Claim: "Synchronous file I/O blocks the event loop"
**Verified:** `writeFileSync` at keychain.ts:87, `readFileSync` at keychain.ts:118, `unlinkSync` at keychain.ts:151, `readdirSync` at keychain.ts:187. `existsSync` at keychain.ts:117, 150. All synchronous. **Confirmed.**

For the stdio server (single client), event loop blocking is irrelevant. For the HTTP/SSE server with concurrent connections, these sync calls would block all connections during file I/O. However, file I/O on local SSD is sub-millisecond, so practical impact is minimal unless the storage directory is on a network filesystem.

### Claim: "Error swallowing asymmetry between keytar and file backends"
**Verified from keychain.ts:**
- `storeKeyFile` (line 88-90): `catch { console.error; throw error }` -- re-throws
- `getKeyFile` (line 121-124): `catch { console.error; return null }` -- swallows
- `deleteKeyFile` (line 155-158): `catch { console.error; return false }` -- swallows
- `listKeyFiles` (line 191-194): `catch { console.error; return [] }` -- swallows

Keytar backend: all errors propagate (no try/catch in the public methods around keytar calls -- the try/catch is in the tool handlers).

**Confirmed -- asymmetry exists.** store re-throws, get/delete/list swallow.

### Claim: "License inconsistency (ISC in package.json, MIT in CONTRIBUTING.md and README.md)"
**Verified:** package.json:20 `"license": "ISC"`, CONTRIBUTING.md:51 "MIT license", README.md:258 "MIT". **Confirmed.**

### Claim: "No SIGTERM/SIGINT handling in any entry point"
**Verified:** Grep for `SIGTERM`, `SIGINT`, `process.on`, `signal` across all source files:

No matches in any source file for signal handling. **Confirmed.**

### Claim: "Docker container runs as root"
**Verified:** Dockerfile has no `USER` directive. `node:20-slim` base image runs as root by default. **Confirmed.** This is a Docker security best-practice violation (should run as non-root user).

---

## Gap Closure: Linux Keyring Persistence

Linux keyring persistence depends on the secret service implementation:

| Implementation | Persistence | Auto-unlock | Notes |
|---------------|-------------|-------------|-------|
| GNOME Keyring | Session or persistent (depends on keyring type) | Login keyring unlocks with PAM | Most common on GNOME desktops |
| KDE Wallet (KWallet) | Persistent | Configured per-wallet | Used on KDE desktops |
| libsecret standalone | Implementation-dependent | Manual | Rare |
| Headless (no DE) | **Not available** | N/A | keytar will throw -- need file fallback |

**Key finding:** On a headless Linux server (no desktop environment), keytar cannot function because no secret service is available. The current codebase only falls back to file-based storage when `DOCKER_ENV=true` is set. A headless Linux deployment without Docker would fail at runtime with a keytar error. This is a gap -- the IS_DOCKER check should be a more general "is native keyring available" check.

## Gap Closure: Smithery NFR Implications

Since Smithery = Docker deployment:
- Same plaintext file storage
- Same chmod 777 permissions
- Same lack of transport security (Smithery platform may add TLS at the proxy level)
- `NODE_ENV=production` is set but unused
- Container may have different resource limits depending on Smithery tier

No additional NFR concerns beyond those already cataloged for Docker.

## Gap Closure: Container Runs as Root

Adding a new NFR:

### SEC-007: Container Process Identity

The Dockerfile does not include a `USER` directive. The Node.js process runs as `root` inside the container. This means:
- Any code execution vulnerability has root-level impact
- Path traversal writes have unrestricted filesystem access
- The `chmod 777` on `/app/data` is redundant (root can access everything)

Docker security best practices recommend running as a non-root user:
```dockerfile
RUN adduser --disabled-password --gecos "" appuser
USER appuser
```

### SEC-008: No Rate Limiting on HTTP/SSE

The Express server has no rate limiting middleware. An attacker could:
- Enumerate all credential names via rapid `list-api-keys` calls
- Brute-force credential names via rapid `get-api-key` calls
- DoS the service via rapid store/delete operations

## Additional NFR: Testability

### TEST-001: Zero Test Infrastructure

- `npm test` exits with error (placeholder script)
- No test framework installed (no jest, mocha, vitest, etc.)
- No test files in the repository
- No mocking infrastructure for keytar (native addon)
- KeychainService is a singleton with no dependency injection -- mocking requires module-level interception
- File backend uses synchronous `fs` calls directly -- no abstraction for test doubles

**Testability assessment:** LOW. The singleton pattern and direct `keytar`/`fs` usage make unit testing difficult without module mocking tools. Integration testing requires a running OS keyring. The file backend is more testable (can use temp directories) but still has no test infrastructure.

---

## Consolidated NFR Registry

### Security (8 NFRs)

| ID | NFR | Status | Severity |
|----|-----|--------|----------|
| SEC-001 | Credential encryption at rest | GOOD (native), POOR (Docker) | HIGH |
| SEC-002 | Transport security | POOR (HTTP has no TLS/auth) | HIGH |
| SEC-003 | Input sanitization | MINIMAL (path traversal vuln) | HIGH |
| SEC-004 | Secret exposure in responses | BY DESIGN | MEDIUM |
| SEC-005 | Permission marker namespace pollution | LOW risk | LOW |
| SEC-006 | Docker storage directory permissions | POOR (chmod 777) | MEDIUM |
| SEC-007 | Container runs as root | POOR | MEDIUM |
| SEC-008 | No rate limiting on HTTP | MISSING | MEDIUM |

### Reliability (5 NFRs)

| ID | NFR | Status | Severity |
|----|-----|--------|----------|
| REL-001 | Error recovery | ASYMMETRIC (keytar propagates, file swallows) | MEDIUM |
| REL-002 | Graceful shutdown | MISSING | MEDIUM |
| REL-003 | Startup reliability | FRAGILE (no port conflict handling) | LOW |
| REL-004 | Data durability | HIGH (native), MEDIUM (Docker) | LOW |
| REL-005 | Idempotency | GOOD (all ops idempotent) | POSITIVE |

### Observability (3 NFRs)

| ID | NFR | Status | Severity |
|----|-----|--------|----------|
| OBS-001 | Logging | MINIMAL (console.error only) | MEDIUM |
| OBS-002 | Audit trail | MISSING | HIGH |
| OBS-003 | Health monitoring | BROKEN (Docker) / MISSING (native) | MEDIUM |

### Performance (3 NFRs)

| ID | NFR | Status | Severity |
|----|-----|--------|----------|
| PERF-001 | Latency | GOOD (sub-50ms operations) | POSITIVE |
| PERF-002 | Concurrency | ADEQUATE (single-user design) | LOW |
| PERF-003 | Resource consumption | LOW footprint | POSITIVE |

### Scalability (2 NFRs)

| ID | NFR | Status | Severity |
|----|-----|--------|----------|
| SCALE-001 | Credential count limits | ADEQUATE for intended use | LOW |
| SCALE-002 | Multi-tenant / multi-namespace | MISSING | LOW |

### Compliance (2 NFRs)

| ID | NFR | Status | Severity |
|----|-----|--------|----------|
| COMP-001 | License consistency | INCONSISTENT (ISC vs MIT) | LOW |
| COMP-002 | Container image labels | GOOD (OCI labels present) | POSITIVE |

### Testability (1 NFR)

| ID | NFR | Status | Severity |
|----|-----|--------|----------|
| TEST-001 | Test infrastructure | MISSING | HIGH |

**Total: 24 NFRs cataloged** (8 security, 5 reliability, 3 observability, 3 performance, 2 scalability, 2 compliance, 1 testability)

---

## Delta Summary
- New items added: SEC-007 (container root), SEC-008 (no rate limiting), TEST-001 (zero test infrastructure), Linux keyring headless failure analysis
- Existing items refined: SEC-003 path traversal impact refined (root user makes it worse), PERF sync I/O impact refined (minimal in practice), all Round 1 claims verified via hallucination audit (6/6 confirmed)
- Remaining gaps: None substantive

## Novelty Assessment
Novelty: NITPICK
SEC-007 (root container) and SEC-008 (no rate limiting) are minor additions that follow naturally from existing analysis. TEST-001 is a restatement of the broad sweep's zero-test finding with more detail on WHY testing is hard (singleton, no DI). The Linux headless keyring failure is worth noting but is an edge case for the intended macOS-primary use case. No finding changes the spec model.

## Convergence Declaration
Pass 4 has converged -- findings are nitpicks, not gaps. The NFR catalog is complete with 24 cataloged NFRs across 7 categories, with verified severity assessments and a consolidated registry.

## State Checkpoint
```yaml
pass: 4
round: 2
status: complete
timestamp: 2026-04-14T00:00:00Z
novelty: NITPICK
```
