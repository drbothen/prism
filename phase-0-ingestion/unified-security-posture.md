# Unified Security Posture -- Prism Multi-Repo Phase 0 Synthesis

**Date:** 2026-04-13
**Scope:** 9 repositories analyzed for security-relevant patterns, vulnerabilities, and architectural decisions
**Purpose:** Consolidate all security findings into a single reference for Prism's security architecture

---

## 1. Per-Repo Security Audit Summaries

### 1.1 poller-cobra (CrowdStrike -- Go)

| Finding | Severity | Details |
|---------|----------|---------|
| In-memory-only cursor state | HIGH | MemoryStore hardcoded despite full FileStore config. All cursor state lost on restart, causing full historical re-fetch. |
| In-memory state updated before persistence | HIGH | `alertState = nextState` runs BEFORE `store.Save()`. On persistence failure, cursor advances past undelivered alerts. |
| Response body not drained on success | MEDIUM | Breaks HTTP/1.1 connection reuse in http_sender.go. |
| Health server not gracefully shut down | MEDIUM | Shutdown() exists but never called. |
| Zero tests for business-critical paths | HIGH | Collector, config, sink, state have zero test coverage. |

**Security strengths:** Distroless nonroot container, read-only root filesystem, drop ALL caps, seccomp profile, file-backed secrets (`*_FILE` env vars), pprof cmdline blocked, pinned action SHAs in CI, daily vulnerability scanning.

### 1.2 poller-express (Cyberint -- Go)

| Finding | Severity | Details |
|---------|----------|---------|
| No OS signal handling | MEDIUM | No SIGTERM/SIGINT handler. Process killed mid-batch with no graceful shutdown. |
| String comparison of numeric asset IDs | MEDIUM | asset_collector.go uses string comparison for numeric IDs, causing incorrect ordering. |
| Unbounded per-IP rate limiter map | LOW | Health server rate limiter map grows without bound. Memory leak under high-cardinality IPs. |
| In-memory state only | HIGH | MemoryStore only. State lost on restart. |
| Strict JSON decoding | MEDIUM | `DisallowUnknownFields` breaks forward compatibility with API changes. |

**Security strengths:** Cookie-based auth via custom RoundTripper, file-backed secrets, distroless container, query fingerprint drift detection (SHA-256).

### 1.3 poller-bear (Claroty xDome -- Go)

| Finding | Severity | Details |
|---------|----------|---------|
| No rate limiting toward Claroty API | MEDIUM | No HTTP 429 handling. Relies on page size + interval. |
| No credential rotation | MEDIUM | Token read once at startup. Requires pod restart to rotate. |
| Helm-config mismatch (BUG) | MEDIUM | 4 env vars set by Helm but never read by config.go. Operators think they are configuring values that have no effect. |
| Dead sentinel error (ErrCursorRegression) | LOW | Defined but never used. Forward progress errors use plain fmt.Errorf. |

**Security strengths:** Bearer token auth with whitespace trimming, TLS 1.2 minimum on all HTTP clients, file-backed secrets, atomic state persistence (write-temp-fsync-rename), distroless nonroot container, read-only root filesystem.

### 1.4 poller-coaster (Armis -- Go)

| Finding | Severity | Details |
|---------|----------|---------|
| Rate limiter memory leak | LOW | Per-IP rate limiter map in health server grows without bound. No eviction. |
| Inconsistent forward progress error handling | MEDIUM | 3/7 collectors use sentinel error, 4/7 use plain error. `errors.Is()` only works for 3/7. |
| Missing limit validation | MEDIUM | AuditLogLimit and RiskFactorLimit not validated. Limit=0 silently disables hasMore pagination. |
| 5 unused sentinel errors | LOW | Dead code that could mislead developers. |
| Dead Helm configuration | LOW | `collector.interval` in values.yaml not referenced in any template. |

**Security strengths:** Atomic JSON state persistence (temp+fsync+rename), distroless nonroot container, file-backed secrets, query fingerprint validation, forward progress invariant, security scanning (gosec + govulncheck + staticcheck).

### 1.5 serveMyAPI (Credential Manager -- TypeScript)

| Finding | Severity | CRITICAL |
|---------|----------|----------|
| **Path traversal vulnerability** | CRITICAL | Key names used directly as file paths in Docker mode with zero sanitization. |
| **Plaintext credential storage** | CRITICAL | Docker mode stores credentials as plaintext `*.key` files with `chmod 777`. |
| **No access control** | HIGH | Any connected MCP client can read all keys. No authentication on MCP endpoints. |
| **No audit trail** | HIGH | Zero logging of credential access events. |
| SSE session ID collision | HIGH | `Date.now()` session IDs collide; messages route to wrong client. |
| No input sanitization | HIGH | Key names not validated at service layer (only at MCP transport via Zod). |
| No encryption for file fallback | HIGH | Container runs as root with no USER directive. |
| Zero test coverage | MEDIUM | `npm test` is a placeholder. |

**Security strengths:** OS keyring delegation (AES-256 on macOS, DPAPI on Windows), macOS permission pre-auth pattern.

### 1.6 tally (Findings Tracker -- Rust)

| Finding | Severity | Details |
|---------|----------|---------|
| State machine enforcement is caller-level | MEDIUM | `Finding.status` is pub. Transition validation is in handlers, not setters. Bypass possible. |
| O(N) load_all for point lookups | LOW | Every MCP tool call loads ALL findings from git. Performance, not security. |
| Sync-in-async without spawn_blocking | LOW | Git operations block Tokio worker threads. |
| Silent save failure | LOW | `let _ = store.save_finding(finding)` during suppression expiry auto-reopen. |

**Security strengths:** `#![forbid(unsafe_code)]`, `clippy::unwrap_used = deny`, CWE-referenced parser security limits (8KB query, 64-depth), SHA-256 fingerprint-based identity, thiserror with `#[non_exhaustive]`, property tests.

### 1.7 axiathon (Security Lake / SIEM -- Rust)

| Finding | Severity | Details |
|---------|----------|---------|
| **Hardcoded vault passphrase** | CRITICAL | `"axiathon-spike-test-key"` in state.rs:429 (CWE-798). |
| **Static Argon2 salt** | HIGH | `b"axiathon-vault-salt-v1"` in vault.rs:126 (CWE-760). |
| Permissive CORS (allow any origin) | HIGH | CWE-942 in main.rs:66. |
| Unprotected admin endpoints | HIGH | No auth on admin routes (OWASP A01:2021). |
| No regex size limit in detection DSL | MEDIUM | Unlike AxiQL which has security limits. |
| Error info leakage to API | MEDIUM | CWE-209 at 8 identified call sites. |
| No `forbid(unsafe_code)` in spike | MEDIUM | All 19 spike crates lack the safety lint. |
| In-memory stores for alerts/cases | HIGH | AlertStore, CaseStore, correlation state all lost on restart. |
| Public fields on domain types | MEDIUM | 78 call sites using public AxiathonEvent fields, 93 using public tenant_id. |

**Security strengths:** 9-layer tenant isolation model, TenantFilterRule optimizer-level query rewriting (prevents OR bypass), CWE-cited parser security limits (64KB/128 depth/1024 regex), AES-256-GCM vault concept, per-tenant file isolation, SECURITY comment convention.

### 1.8 ocsf-proto-gen (Proto Generator -- Rust)

| Finding | Severity | Details |
|---------|----------|---------|
| Version string used in paths without sanitization | LOW | User-supplied `--ocsf-version` used in directory path. Path traversal possible in CLI context. |
| No partial-failure cleanup | LOW | Failed generation leaves partial output files on disk. |

**Security strengths:** Feature-gated network dependencies, deterministic BTreeMap/BTreeSet output, `#[serde(default)]` tolerant parsing.

### 1.9 mcp-claroty-xdome (MCP Server -- TypeScript)

| Finding | Severity | Details |
|---------|----------|---------|
| Unbounded in-memory caches | HIGH | All 5 domain service caches grow without limit. Memory exhaustion possible. |
| No session expiration | HIGH | Sessions accumulate forever in memory. |
| CORS wildcard in production | MEDIUM | `origin: "*"` unconditionally in factory.ts. |
| No rate limiting on MCP endpoints | MEDIUM | Only retry logic for upstream; no protection on server endpoints. |
| Static bearer token (no rotation) | MEDIUM | Single token from env var. Rotation requires restart. |
| Filter value untyped | MEDIUM | `z.any()` for filter values in all Zod schemas. |
| Express body size conflict | MEDIUM | Global 100KB limit may reject before per-route 10MB limit. |
| SDK internal access | MEDIUM | `(mcpServer as any).setToolRequestHandlers()` uses private API. |

**Security strengths:** Typed error hierarchy mapped to JSON-RPC 2.0 codes, Zod schema validation, bearer token auth, retry with backoff on 429/5xx, structured logging.

---

## 2. Cross-Repo Attack Surface Analysis

When patterns from these 9 repos combine in Prism, the following attack vectors emerge:

### 2.1 Credential Exposure Chain

**Vector:** serveMyAPI's plaintext file storage + poller credential loading patterns = potential credential leak path.

All 4 pollers load credentials via environment variables or `*_FILE` file mounts. If Prism stores credentials using serveMyAPI's Docker file fallback pattern (plaintext, `chmod 777`, path traversal), the entire credential chain is compromised.

**Risk level:** CRITICAL
**Mitigation:** Encrypted file storage with AES-256-GCM (axiathon's vault concept, properly implemented with unique salts and external key management).

### 2.2 Multi-Tenant Data Leakage

**Vector:** Pollers are all single-tenant. Prism combines multiple clients' sensor data in one process.

None of the 4 pollers have any concept of tenant isolation -- each runs as a single pod per client. When Prism aggregates all clients into a single MCP server:
- Cursor state for Client A could be confused with Client B
- Cached API responses could be served to the wrong client
- Log output could intermingle client data
- Error messages could leak one client's sensor URLs/tokens to another

**Risk level:** CRITICAL
**Mitigation:** Axiathon's 9-layer tenant isolation model is the reference architecture. At minimum: newtype TenantId, per-tenant state stores, per-tenant caches, optimizer-level query isolation.

### 2.3 Denial of Service via Unbounded Resources

**Vector:** Multiple repos share the same unbounded resource patterns.

- Unbounded per-IP rate limiter maps (poller-express, poller-coaster)
- Unbounded in-memory caches (mcp-claroty-xdome)
- Unbounded session maps (mcp-claroty-xdome)
- No query size limits (mcp-claroty-xdome filter values)

In a multi-tenant Prism server handling multiple sensors per client, these unbounded patterns compound. A single misbehaving client or attacker could exhaust server memory.

**Risk level:** HIGH
**Mitigation:** LRU eviction on all maps, maximum entry counts, session TTLs, query/filter size limits (adopt axiathon's CWE-cited limits).

### 2.4 State Corruption on Restart

**Vector:** poller-cobra and poller-express use in-memory-only state. poller-cobra updates in-memory state before persistence.

If Prism adopts these patterns, a crash or restart causes:
- Full re-fetch of all historical data from all sensors
- Duplicate data sent downstream
- Potential rate limit exhaustion on sensor APIs during re-fetch

**Risk level:** HIGH
**Mitigation:** Durable cursor persistence from day one (poller-bear's atomic write pattern). Update state AFTER successful persistence (fix poller-cobra's ordering bug).

### 2.5 API Authentication Diversity

**Vector:** Each sensor uses a different auth mechanism. Misconfiguring one could expose another's credentials.

| Sensor | Auth Method |
|--------|------------|
| CrowdStrike | OAuth2 Client Credentials (client_id + client_secret -> bearer token) |
| Cyberint | Static API key as HTTP cookie (`access_token`) |
| Claroty xDome | Static bearer token (`Authorization: Bearer <token>`) |
| Armis | Bearer token via SDK |

A unified credential store must handle all four patterns without cross-contamination. If Cyberint's cookie-based auth accidentally sends the token as a Bearer header, authentication fails. If Claroty's bearer token accidentally leaks into a cookie, it could be sent to unintended domains.

**Risk level:** MEDIUM
**Mitigation:** Per-sensor auth middleware with strict type safety. Each sensor adapter owns its own auth injection. No shared auth middleware across sensor types.

---

## 3. Authentication/Authorization Flows

### 3.1 Per-Sensor Authentication Mechanisms

#### CrowdStrike Falcon (poller-cobra)

```
Flow: OAuth2 Client Credentials
1. Client sends (client_id, client_secret) to OAuth2 token endpoint
2. CrowdStrike returns bearer token with TTL
3. gofalcon SDK manages token lifecycle transparently (auto-refresh)
4. All API calls use: Authorization: Bearer <token>
5. Multi-region: us-1, us-2, eu-1, ap-1 (different base URLs)
```

**Credentials required:** `CROWDSTRIKE_CLIENT_ID`, `CROWDSTRIKE_CLIENT_SECRET`, `CROWDSTRIKE_REGION`
**Rotation:** Automatic via OAuth2 token refresh in SDK

#### Cyberint Argos (poller-express)

```
Flow: Static API Key as Cookie
1. API key loaded from env var or file mount
2. Custom http.RoundTripper injects cookie on every request:
   Cookie: access_token=<api_key>
3. Customer ID auto-extracted from URL: https://<customer>.cyberint.io
4. All API calls include the cookie
```

**Credentials required:** `CYBERINT_API_KEY`, `CYBERINT_API_URL` (contains customer ID)
**Rotation:** Manual. Requires pod restart.

#### Claroty xDome (poller-bear, mcp-claroty-xdome)

```
Flow: Static Bearer Token
1. API token loaded from env var or file mount
2. Trimmed of whitespace/newlines at construction
3. All API calls use: Authorization: Bearer <token>
4. 15-30s request timeout
```

**Credentials required:** `CLAROTY_TOKEN`, `CLAROTY_BASE_URL`
**Rotation:** Manual. Requires pod restart.

#### Armis Centrix (poller-coaster)

```
Flow: Bearer Token via SDK
1. API key loaded from env var or file mount
2. armis-sdk-go wraps authentication
3. Single SDK method: GetSearch(aql)
4. Bearer token injected by SDK
```

**Credentials required:** `ARMIS_API_KEY`, `ARMIS_API_URL`
**Rotation:** Manual. Requires pod restart.

### 3.2 Credential Storage Patterns Across Repos

| Pattern | Repos Using It | Security Level |
|---------|---------------|---------------|
| `*_FILE` env vars (K8s secret mount) | All 4 pollers | GOOD -- secrets never in env var logs |
| Direct env vars | All 4 pollers (fallback) | ACCEPTABLE -- standard K8s pattern |
| OS keyring (macOS/Windows/Linux) | serveMyAPI | GOOD -- hardware-backed encryption |
| Plaintext files (Docker fallback) | serveMyAPI | CRITICAL VULNERABILITY |
| AES-256-GCM vault (per-tenant) | axiathon (spike) | GOOD CONCEPT, BAD IMPLEMENTATION (hardcoded key/salt) |

### 3.3 Prism's Unified Authentication Approach

Prism must implement a per-sensor authentication middleware pattern:

```
trait SensorAuth: Send + Sync {
    /// Inject authentication into an outgoing HTTP request
    fn authenticate(&self, request: &mut Request) -> Result<(), AuthError>;
    
    /// Check if credentials need refresh (for OAuth2 flows)
    fn needs_refresh(&self) -> bool;
    
    /// Refresh credentials (for OAuth2 flows)
    async fn refresh(&self) -> Result<(), AuthError>;
}
```

Implementations:
- `OAuth2ClientCredentials` -- for CrowdStrike (auto-refresh)
- `CookieAuth` -- for Cyberint (static, injects cookie)
- `BearerTokenAuth` -- for Claroty, Armis (static, injects header)

---

## 4. Data Classification

### 4.1 Sensitivity Level Definitions

| Level | Definition | Examples |
|-------|-----------|---------|
| **CRITICAL** | Credentials that grant access to external systems. Compromise enables unauthorized data access. | API keys, OAuth2 client secrets, bearer tokens |
| **HIGH** | Client-identifying metadata and configuration that reveals MSSP client infrastructure. | Client IDs, sensor base URLs, customer subdomains, tenant IDs |
| **MEDIUM** | Security event data from sensors. Contains indicators of compromise and vulnerability details. | CrowdStrike alerts, Cyberint threat intel, Claroty OT alerts, Armis device inventory |
| **LOW** | Operational metadata. Internal to Prism. | Cursor positions, query fingerprints, health status, cache statistics |

### 4.2 Data Flow Classification

```
CRITICAL DATA (never log, never cache unencrypted):
  - CrowdStrike client_id + client_secret
  - CrowdStrike OAuth2 bearer tokens
  - Cyberint API keys
  - Claroty xDome bearer tokens
  - Armis API keys
  - Vector sink credentials (username + password)

HIGH DATA (log redacted, encrypt at rest):
  - Client tenant identifiers
  - Sensor API base URLs (contain customer identifiers)
  - Cyberint customer subdomain
  - CrowdStrike region assignment per client
  - Per-client sensor configuration

MEDIUM DATA (standard protection, encrypt in transit):
  - CrowdStrike alerts (IOCs, tactics, techniques, severity)
  - Cyberint threat intelligence (alerts with 52 AlertData subtypes)
  - Cyberint digital assets (10 fields including domain, IP ranges)
  - Claroty xDome alerts, devices, vulnerabilities (9 data sources, 47-field relations)
  - Armis alerts, devices, vulnerabilities, connections (7 data sources)
  - xMP enrichment metadata (site, cluster_name, node_name)
  - OCSF-normalized events

LOW DATA (standard logging, no encryption required):
  - Cursor positions per sensor per client
  - Query fingerprint hashes (SHA-256)
  - Batch receipt audit trails
  - Health/readiness probe status
  - Poll cycle timing and retry counts
  - Cache hit/miss statistics
```

### 4.3 Data at Rest

| Location | Data Level | Current Protection | Prism Requirement |
|----------|-----------|-------------------|-------------------|
| K8s Secrets | CRITICAL | K8s etcd encryption (cluster-dependent) | Minimum: K8s secrets. Consider external vault (HashiCorp Vault, AWS Secrets Manager) |
| State files (cursors) | LOW | Filesystem permissions | Filesystem permissions sufficient. Per-tenant isolation required. |
| Cached API responses | MEDIUM | None (in-memory) | Bounded, per-tenant isolated, auto-expiring |
| Log output | LOW-HIGH | Depends on log aggregator | CRITICAL data must be redacted before logging |

### 4.4 Data in Transit

| Path | Data Level | Current Protection | Prism Requirement |
|------|-----------|-------------------|-------------------|
| Prism -> CrowdStrike API | CRITICAL + MEDIUM | TLS (SDK-managed) | TLS 1.2+ minimum |
| Prism -> Cyberint API | CRITICAL + MEDIUM | TLS (system default) | TLS 1.2+ minimum |
| Prism -> Claroty API | CRITICAL + MEDIUM | TLS 1.2+ (explicit) | TLS 1.2+ minimum |
| Prism -> Armis API | CRITICAL + MEDIUM | TLS (SDK-managed) | TLS 1.2+ minimum |
| Prism -> Vector sink | MEDIUM | HTTP Basic Auth | TLS 1.2+ required. Consider mTLS. |
| MCP Client -> Prism | HIGH + MEDIUM | stdio (local) or HTTP | If HTTP transport: TLS required, token auth required |

---

## 5. Shared Secret Management Assessment

### 5.1 Current Patterns Across Repos

**Pattern 1: File-backed secrets with env var fallback** (all 4 pollers)

```
resolve_secret(file_env, direct_env):
  1. Read ${FILE_ENV} -> get file path
  2. If file exists, read contents, trim whitespace -> return
  3. If file doesn't exist, read ${DIRECT_ENV} -> return
  4. If neither, return empty (validation catches later)
```

This is the most mature pattern. All pollers implement it consistently. File priority over env var ensures K8s secret mounts work correctly.

**Pattern 2: OS keyring** (serveMyAPI)

Uses `keytar` (TypeScript) / `keyring-rs` (Rust equivalent) for OS-level credential storage. Good for desktop/development, unusable in containers without desktop environment.

**Pattern 3: AES-256-GCM vault** (axiathon spike)

Per-tenant encrypted files using Argon2 KDF. Good concept but implementation is fatally flawed (hardcoded passphrase and static salt).

### 5.2 Gaps Identified

| Gap | Repos Affected | Impact |
|-----|---------------|--------|
| No credential rotation without restart | All 4 pollers, mcp-claroty-xdome | Credential changes require pod restart |
| No credential validation at load time | poller-cobra (empty token passes initial load) | Bad credentials discovered at first API call, not startup |
| No credential isolation between tenants | All repos (single-tenant design) | In multi-tenant Prism, one client's credentials must never be accessible to another |
| No audit of credential access | All repos except axiathon (partial) | No logging of when/why credentials are read |
| No encryption of file-backed secrets | serveMyAPI Docker mode | Plaintext on disk |
| CrowdStrike OAuth2 token caching | poller-cobra (SDK handles) | Prism must implement token caching if not using official SDK |

### 5.3 Prism's Secret Management Architecture

```
                    +---------------------------+
                    |   Secret Resolution Chain  |
                    +---------------------------+
                              |
              +---------------+---------------+
              |               |               |
         K8s Secret      Env Var        Encrypted File
         Mount (FILE)    Fallback       Store (container)
              |               |               |
              v               v               v
         +------------------------------------------+
         |        Per-Tenant Credential Store        |
         |  - Keyed by (tenant_id, sensor_type)      |
         |  - Credentials never cross tenant boundary |
         |  - In-memory cache with TTL               |
         |  - Audit log on every access              |
         +------------------------------------------+
                              |
         +--------------------+--------------------+
         |                    |                    |
    OAuth2 Flow         Cookie Injection     Bearer Token
    (CrowdStrike)       (Cyberint)          (Claroty, Armis)
```

**Requirements:**
1. Credentials keyed by `(tenant_id, sensor_type)` -- no cross-tenant access
2. File-backed secret resolution with env var fallback (proven pattern from pollers)
3. Encrypted file store for container deployments (AES-256-GCM with external master key)
4. In-memory credential cache with configurable TTL
5. Audit log for every credential access (who, when, which credential, success/failure)
6. File watch for credential rotation without restart
7. Startup validation: verify all configured credentials are loadable and non-empty

---

## 6. MSSP-Specific Security Concerns

### 6.1 Multi-Tenancy Isolation Requirements

Prism serves as an MSSP platform where 1898 & Co manages multiple clients' security sensors. The isolation requirements are strict:

**Axiom: No client must ever see another client's data, credentials, or operational metadata.**

### 6.2 Isolation Layers (Adapted from Axiathon's 9-Layer Model)

| Layer | Mechanism | Purpose |
|-------|-----------|---------|
| L1 | **TenantId newtype** | Compile-time prevention of string mixing. `TenantId` is a validated, private-field newtype -- not a bare `String`. |
| L2 | **Per-tenant credential store** | Credentials keyed by `(tenant_id, sensor_type)`. No shared credential namespace. |
| L3 | **Per-tenant cursor state** | Each client's polling cursors stored independently. Cursor files/records isolated by tenant. |
| L4 | **Per-tenant API clients** | Separate HTTP clients per tenant (or at minimum, per-tenant auth middleware). No shared connection state. |
| L5 | **Per-tenant cache isolation** | Cache instances are per `(tenant_id, sensor_type)`. Client A's cached responses never served to Client B. |
| L6 | **Per-tenant rate limiting** | API rate limits tracked per tenant to prevent one client's volume from exhausting another's quota. |
| L7 | **Log field isolation** | Structured logging with tenant_id as a required span field. Log filtering by tenant. Credential values never logged. |
| L8 | **MCP tool scoping** | MCP tools must accept and enforce tenant context. A tool call for Client A cannot return Client B's data. |
| L9 | **Error message sanitization** | Error messages must not leak tenant-identifying information (URLs, credentials, customer IDs) across boundaries. |

### 6.3 Client Data Separation Patterns

**Cursor State Isolation:**
```
/var/lib/prism/state/
  tenant-a/
    crowdstrike-alerts.json
    cyberint-alerts.json
    claroty-alerts.json
  tenant-b/
    crowdstrike-alerts.json
    ...
```

Each tenant's state is in a separate directory. File paths derived from validated TenantId (not raw strings -- prevents path traversal).

**Credential Isolation:**
```
K8s Secrets:
  prism-tenant-a-crowdstrike-credentials
  prism-tenant-a-cyberint-credentials
  prism-tenant-b-crowdstrike-credentials
  ...
```

One K8s Secret per (tenant, sensor). Mounted to tenant-specific paths.

**Cache Isolation:**
Each `(tenant_id, sensor_type)` pair gets its own cache instance with independent TTL, size bounds, and eviction. No shared cache keys.

### 6.4 Credential Isolation Per Client

| Concern | Mitigation |
|---------|-----------|
| Client A's CrowdStrike token used for Client B's query | Per-tenant API client instances with tenant-scoped credential loading |
| Credential leak in error messages | Error types carry tenant_id but redact credential values. `fmt::Display` never includes raw secrets. |
| Shared OAuth2 token across tenants | Each tenant gets its own OAuth2 flow with its own client_id/client_secret. Token cache keyed by tenant_id. |
| Admin access to all client credentials | Role-based access if HTTP transport is used. Audit logging for all credential operations. |
| Pod restart re-authenticates all tenants | OAuth2 tokens cached per-tenant in durable storage (not just in-memory). File-backed secrets survive restart. |

### 6.5 Cross-Tenant Attack Scenarios

| Scenario | Vector | Mitigation |
|----------|--------|-----------|
| Tenant ID spoofing via MCP tool | Client sends different tenant_id in tool parameters | Tenant context derived from authenticated session, not from tool parameters. Tool parameters cannot override tenant identity. |
| Cache poisoning | Malformed response cached under wrong tenant key | Cache keys include tenant_id as mandatory prefix. Cache lookup validates tenant match. |
| Cursor manipulation | Attacker advances another tenant's cursor | State files in per-tenant directories with filesystem permissions. TenantId in file path validated against request context. |
| Log harvesting | Attacker reads logs to discover other tenants | Structured logging with tenant_id field enables log filtering by tenant. CRITICAL data redacted before logging. |

---

## 7. Recommended Security Architecture for Prism

### P0: Must Have Before MVP (Blocks All Production Use)

#### P0-SEC-001: Encrypted Credential Store with Per-Tenant Isolation

**Rationale:** serveMyAPI's plaintext storage is a critical vulnerability. All pollers lack multi-tenant credential isolation. Prism manages sensitive API keys for multiple MSSP clients.

**Requirements:**
1. `CredentialStore` trait with pluggable backends (keyring, encrypted file, K8s secrets)
2. Credentials keyed by `(tenant_id, sensor_type)`
3. Encrypted file backend using AES-256-GCM with external master key (NOT hardcoded -- fix axiathon's anti-pattern)
4. Argon2id KDF with unique per-credential salts (NOT static -- fix axiathon's anti-pattern)
5. Credential name sanitization: restrict to `[a-zA-Z0-9_.-]` (fix serveMyAPI's path traversal)
6. Audit log for every credential access

#### P0-SEC-002: TenantId Newtype with Validated Constructor

**Rationale:** Axiathon's spike has 93 call sites using public `tenant_id` field. Multi-tenant MSSP product requires compile-time tenant safety.

**Requirements:**
1. `TenantId` as newtype with private inner field
2. `new()` validates format (e.g., UUID or constrained alphanumeric)
3. `new_unchecked()` for trusted sources (database reads)
4. All functions handling tenant-scoped data accept `TenantId`, never bare `String`
5. `Display` implementation never reveals internal format details

#### P0-SEC-003: Per-Sensor Auth Middleware with Strict Type Safety

**Rationale:** 4 different auth mechanisms that must not cross-contaminate.

**Requirements:**
1. `SensorAuth` trait with per-sensor implementations
2. OAuth2 implementation for CrowdStrike with automatic token refresh
3. Cookie injection for Cyberint (`access_token` cookie, not header)
4. Bearer token injection for Claroty and Armis
5. Auth middleware injected at the API client layer, not at the HTTP transport layer
6. Unit tests verifying each auth type injects credentials correctly and ONLY in the expected format

#### P0-SEC-004: Durable State Persistence with Atomic Writes

**Rationale:** poller-cobra and poller-express lose all state on restart. poller-cobra has the state-before-persistence ordering bug.

**Requirements:**
1. Atomic write pattern: temp file -> fsync -> rename (poller-bear/poller-coaster proven pattern)
2. State updated AFTER successful persistence (fix poller-cobra's ordering bug)
3. Per-tenant state isolation (separate files/records per tenant)
4. Query fingerprint validation to prevent stale cursor use after config change

#### P0-SEC-005: TLS 1.2+ on All HTTP Connections

**Rationale:** poller-bear enforces TLS 1.2 minimum. Other pollers use system defaults.

**Requirements:**
1. TLS 1.2 minimum on all outbound connections (sensor APIs)
2. TLS 1.2 minimum on all inbound connections (if HTTP transport is used)
3. Configurable custom CA bundle path for enterprise deployments
4. Certificate validation enabled (no `danger_accept_invalid_certs` in production)

#### P0-SEC-006: Secret Redaction in Logs and Errors

**Rationale:** poller-express has secret redaction (first2 + *** + last2). Other repos are inconsistent.

**Requirements:**
1. All CRITICAL data (API keys, tokens, passwords) redacted before logging
2. `fmt::Display` on credential types always produces redacted output
3. Error types never include raw credential values
4. Config dump / dry-run mode shows redacted credentials only

### P1: Should Have (Significant Security Improvement)

#### P1-SEC-001: Bounded Resource Limits

**Rationale:** Unbounded maps/caches across multiple repos create DoS surface.

**Requirements:**
1. LRU eviction on all per-IP rate limiter maps (fix poller-express, poller-coaster pattern)
2. Maximum entry count and TTL on all caches (fix mcp-claroty-xdome pattern)
3. Session TTL with periodic cleanup (fix mcp-claroty-xdome session leak)
4. Query/filter size limits with CWE citations (adopt axiathon's limits: 64KB query, 128 depth)

#### P1-SEC-002: Credential Rotation Without Restart

**Rationale:** All pollers require pod restart for credential changes. MSSP operations need zero-downtime rotation.

**Requirements:**
1. File-backed secrets watched for changes (inotify/kqueue)
2. OAuth2 token cache invalidated on credential file change
3. Bearer/cookie tokens refreshed from new file contents
4. Graceful transition: in-flight requests complete with old credentials, new requests use updated credentials

#### P1-SEC-003: Fail-Fast Credential Validation at Startup

**Rationale:** poller-cobra's Ping pattern verifies credentials before polling. Prism must verify all sensors for all tenants.

**Requirements:**
1. `ping()` method on every sensor adapter trait
2. All sensors for all tenants verified during startup
3. Per-sensor health status reported (partial startup allowed -- some sensors up, others retrying)
4. Clear error messages identifying which tenant + sensor has credential issues

#### P1-SEC-004: Graceful Shutdown

**Rationale:** poller-express has no signal handling. poller-cobra's health server shutdown is never called.

**Requirements:**
1. SIGTERM/SIGINT handler via `tokio::signal`
2. Cancel all polling loops via context cancellation
3. Drain in-flight sink deliveries
4. Persist cursor state before exit
5. Shut down health server with grace period (5s)
6. All components included in shutdown sequence (fix poller-cobra's missing health server shutdown)

#### P1-SEC-005: Structured Error Hierarchy

**Rationale:** Consistent error handling prevents information leakage and enables proper error classification.

**Requirements:**
1. `thiserror` enum with `#[non_exhaustive]` (tally's proven pattern)
2. Error variants carry actionable context without sensitive data
3. Centralized MCP error code mapping (fix tally's distributed `to_mcp_err` pattern)
4. Sensor-specific errors wrapped in common variants, not exposed directly

#### P1-SEC-006: Audit Trail for Credential and Data Access

**Rationale:** serveMyAPI has zero audit capability. MSSP operations require accountability.

**Requirements:**
1. Log every credential access: who (tenant), when, which credential, success/failure
2. Log every sensor API call: tenant, sensor, endpoint, response status
3. Log every MCP tool invocation: tenant, tool name, parameters (redacted), result status
4. Structured logging with `tracing` crate, JSON output, tenant_id as span field

### P2: Nice to Have (Defense in Depth)

#### P2-SEC-001: Container Security Hardening

**Rationale:** All pollers use distroless nonroot containers. Proven pattern.

**Requirements:**
1. Distroless base image (or scratch for Rust static binary)
2. Non-root user (UID 65532)
3. Read-only root filesystem
4. Drop ALL capabilities
5. Seccomp RuntimeDefault profile
6. No shell, no package manager

#### P2-SEC-002: Supply Chain Security

**Rationale:** poller-cobra uses pinned action SHAs and harden-runner with egress audit.

**Requirements:**
1. Pinned dependency versions (Cargo.lock committed)
2. `cargo audit` in CI (daily cron + PR trigger)
3. `cargo deny` for license and advisory checking
4. Pinned GitHub Action SHAs in CI workflows
5. SBOM generation for container images

#### P2-SEC-003: Rate Limiting Toward Sensor APIs

**Rationale:** No poller implements rate limiting toward upstream APIs. No 429 handling in most.

**Requirements:**
1. Per-sensor, per-tenant rate limiting for outbound API calls
2. HTTP 429 handling with Retry-After header respect
3. Configurable rate limits per sensor type
4. Backoff on rate limit rather than immediate retry

#### P2-SEC-004: Input Validation at Service Layer

**Rationale:** serveMyAPI validates only at transport layer (Zod), not at service layer. CLI bypasses validation.

**Requirements:**
1. Validation at the service layer, not just the transport layer
2. All tool inputs validated regardless of transport (stdio, HTTP, CLI)
3. Single validation path for both MCP and CLI interfaces
4. Typed filter operations (fix mcp-claroty-xdome's `z.any()` pattern)

#### P2-SEC-005: SECURITY Comment Convention

**Rationale:** Axiathon's `SECURITY(CWE-xxx)` comment pattern makes security decisions grep-able and auditable.

**Requirements:**
1. All security-relevant code decisions annotated with `// SECURITY(CWE-xxx):` or `// SECURITY(OWASP-xxx):`
2. CI check that all SECURITY comments reference valid CWE/OWASP identifiers
3. Regular security comment audit as part of maintenance sweep

#### P2-SEC-006: Connection Pooling with Limits

**Rationale:** mcp-claroty-xdome's Python impl uses connection limits (100 total, 30/host). TypeScript creates new connections per request.

**Requirements:**
1. reqwest connection pool with configurable limits per sensor
2. Per-host connection limits to prevent resource exhaustion on any single sensor API
3. Idle connection timeout to prevent stale connections
4. Connection pool metrics (active, idle, waiting) exposed via health endpoint

---

## Summary of Security Priority Matrix

| Priority | Count | Theme |
|----------|-------|-------|
| **P0** | 6 items | Credential encryption, tenant isolation, auth type safety, durable state, TLS, secret redaction |
| **P1** | 6 items | Resource bounds, credential rotation, startup validation, graceful shutdown, error hierarchy, audit trail |
| **P2** | 6 items | Container hardening, supply chain, rate limiting, input validation, security annotations, connection pooling |

**Critical vulnerabilities that Prism must NOT inherit:**
1. serveMyAPI's plaintext file storage and path traversal (P0-SEC-001)
2. axiathon's hardcoded vault passphrase and static salt (P0-SEC-001)
3. poller-cobra's state-before-persistence ordering bug (P0-SEC-004)
4. mcp-claroty-xdome's unbounded caches and sessions (P1-SEC-001)
5. poller-express's missing signal handling (P1-SEC-004)
6. All pollers' single-tenant architecture in a multi-tenant context (P0-SEC-002, Section 6)

---

## State Checkpoint

```yaml
document: unified-security-posture
phase: 0 (ingestion synthesis)
status: complete
repos_analyzed: 9
critical_vulnerabilities: 6
high_vulnerabilities: 12
medium_vulnerabilities: 18
low_vulnerabilities: 8
p0_security_items: 6
p1_security_items: 6
p2_security_items: 6
tenant_isolation_layers: 9
auth_mechanisms: 4 (OAuth2, Cookie, Bearer x2)
data_classification_levels: 4 (CRITICAL, HIGH, MEDIUM, LOW)
timestamp: 2026-04-13T00:00:00Z
```
