# HS-004: Credential Lifecycle Scenarios

**Group:** Store, retrieve, rotate, expire credentials per client per sensor
**Date:** 2026-04-13
**Priority:** P0

---

## HS-004-01: Credential Storage and Retrieval Per Tenant Per Sensor

**Title:** Store and retrieve credentials keyed by (tenant_id, sensor_type)

**Preconditions:**
- Prism credential store initialized with encrypted file backend
- No pre-existing credentials

**Steps:**
1. Store CrowdStrike credentials for Tenant A: `(client_id, client_secret)` via CredentialStore trait
2. Store Cyberint credentials for Tenant A: `(api_key)` via CredentialStore trait
3. Store CrowdStrike credentials for Tenant B: `(client_id, client_secret)` with different values
4. Retrieve Tenant A's CrowdStrike credentials
5. Retrieve Tenant B's CrowdStrike credentials
6. Attempt to retrieve non-existent Tenant C's credentials

**Expected Outcome:**
- Tenant A's CrowdStrike credentials returned correctly (not Tenant B's)
- Tenant B's CrowdStrike credentials returned correctly (not Tenant A's)
- Tenant C lookup returns `None` / typed error (not crash, not empty string)
- Credentials stored encrypted at rest using AES-256-GCM with external master key (NOT hardcoded -- avoiding axiathon's anti-pattern)
- Argon2id KDF used with unique per-credential salts (NOT static -- avoiding axiathon's anti-pattern)
- Credential names restricted to `[a-zA-Z0-9_.-]` (preventing serveMyAPI's path traversal vulnerability)
- Audit log entry for every store and retrieve operation

**Repos Tested:** serveMyAPI (credential CRUD domain model, keyring abstraction), axiathon (AES-256-GCM vault concept -- fixed implementation)

---

## HS-004-02: File-Backed Secret Resolution with Env Var Fallback

**Title:** Credentials loaded from K8s secret file mounts with env var fallback

**Preconditions:**
- K8s secret mounted at `/etc/prism/secrets/tenant-a-crowdstrike-client-id`
- Env var `PRISM_CROWDSTRIKE_CLIENT_ID` also set (should be ignored when file exists)

**Steps:**
1. Prism loads CrowdStrike client_id for Tenant A
2. Secret resolution checks `PRISM_CROWDSTRIKE_CLIENT_ID_FILE` env var for file path
3. File exists -- reads content, trims whitespace
4. Returns file content (env var `PRISM_CROWDSTRIKE_CLIENT_ID` ignored)
5. Remove the file mount, restart
6. Prism falls back to `PRISM_CROWDSTRIKE_CLIENT_ID` env var

**Expected Outcome:**
- File takes priority over direct env var (proven pattern from all 4 pollers)
- File content trimmed of whitespace/newlines (poller-bear's bearer token trimming pattern)
- When file unavailable, env var used as fallback
- When neither available, validation reports clear error identifying which tenant + sensor is misconfigured
- Config prefix standardized: `PRISM_<SENSOR>_*` for all (resolving poller inconsistency: cobra uses `CROWDSTRIKE_*`, bear uses `CLAROTY_*/POLLER_BEAR_*`, coaster has 5 schemes)

**Repos Tested:** all 4 pollers (file-backed secrets pattern), poller-bear (whitespace trimming)

---

## HS-004-03: OAuth2 Token Lifecycle for CrowdStrike

**Title:** OAuth2 Client Credentials flow with automatic token refresh

**Preconditions:**
- Tenant A's CrowdStrike credentials stored: client_id + client_secret
- CrowdStrike OAuth2 endpoint reachable

**Steps:**
1. Prism initiates OAuth2 Client Credentials flow for Tenant A
2. Receives bearer token with TTL (e.g., 30 minutes)
3. Token cached in per-tenant token store
4. First API call uses cached token successfully
5. Token expires (TTL elapsed)
6. Next API call triggers automatic token refresh
7. New token cached, API call succeeds

**Expected Outcome:**
- OAuth2 flow implemented with reqwest + oauth2 crate (not gofalcon SDK -- Rust equivalent)
- Token cached per tenant -- Tenant A's token never used for Tenant B
- Automatic refresh before expiry (or on 401 response)
- Multi-region support: us-1, us-2, eu-1, ap-1 (different base URLs per region)
- Token refresh does not block concurrent API calls (old token valid until replaced)
- Audit log: `{ "event": "oauth2_token_refresh", "tenant": "tenant-a", "sensor": "crowdstrike" }`

**Repos Tested:** poller-cobra (CrowdStrike OAuth2 flow, multi-region, gofalcon SDK behavior)

---

## HS-004-04: Credential Rotation Without Restart

**Title:** Rotating a sensor's API key without restarting Prism

**Preconditions:**
- Tenant A's Claroty bearer token stored in K8s secret file mount
- Prism actively polling Claroty for Tenant A
- File watch (inotify/kqueue) configured on secret mount directory

**Steps:**
1. Prism polling Claroty with current token successfully
2. Operator updates K8s secret (new Claroty token written to file)
3. File watch detects change
4. Prism loads new token from file
5. In-flight requests complete with old token
6. New requests use updated token
7. Next poll cycle uses new token successfully

**Expected Outcome:**
- Zero downtime during rotation -- no restart required (fixing all pollers' restart-required pattern)
- Graceful transition: in-flight requests complete, new requests use new credentials
- Old token invalidated in credential cache after transition
- Audit log: `{ "event": "credential_rotated", "tenant": "tenant-a", "sensor": "claroty", "method": "file_watch" }`
- OAuth2 credentials (CrowdStrike): rotation triggers new OAuth2 flow, old bearer token discarded
- Cookie credentials (Cyberint): new API key injected into cookie on next request

**Repos Tested:** all pollers (currently require restart -- this is the improvement), serveMyAPI (credential update pattern)

---

## HS-004-05: Startup Credential Validation (Fail-Fast)

**Title:** Prism validates all credentials at startup before entering poll loop

**Preconditions:**
- Tenant A has valid CrowdStrike credentials
- Tenant B has expired Cyberint API key
- Tenant C has valid Claroty token

**Steps:**
1. Prism starts up
2. Prism calls `ping()` on each sensor adapter for each tenant
3. CrowdStrike (Tenant A): ping succeeds (limit=1 query, like poller-cobra's connectivity verification)
4. Cyberint (Tenant B): ping fails (401 Unauthorized)
5. Claroty (Tenant C): ping succeeds

**Expected Outcome:**
- Startup reports per-tenant, per-sensor health: Tenant A CrowdStrike OK, Tenant B Cyberint FAILED, Tenant C Claroty OK
- Partial startup allowed: healthy sensors begin polling, unhealthy sensors retry in background
- Clear error message: `"Tenant tenant-b sensor cyberint: authentication failed (HTTP 401). Verify PRISM_CYBERINT_API_KEY or secret file mount."`
- Prism does not crash on partial credential failure -- continues with healthy sensors
- Empty credentials caught at load time (fixing poller-cobra's pattern where empty token passes initial load)

**Repos Tested:** poller-cobra (ping via limit=1 query), poller-express (runner orchestration), all pollers (config validation)

---

## HS-004-06: Credential Audit Trail

**Title:** Every credential access event is logged with structured context

**Preconditions:**
- Prism running with structured logging (tracing crate, JSON output)
- Multiple tenants and sensors active

**Steps:**
1. Prism loads Tenant A's CrowdStrike credentials at startup
2. Prism uses credentials for OAuth2 token exchange
3. Prism refreshes OAuth2 token
4. Operator rotates Tenant A's Claroty token
5. MCP client queries data (triggering credential use)

**Expected Outcome:**
- Audit log entries for each event:
  - `{ "event": "credential_loaded", "tenant": "tenant-a", "sensor": "crowdstrike", "source": "file_mount" }`
  - `{ "event": "oauth2_token_exchanged", "tenant": "tenant-a", "sensor": "crowdstrike" }`
  - `{ "event": "oauth2_token_refreshed", "tenant": "tenant-a", "sensor": "crowdstrike" }`
  - `{ "event": "credential_rotated", "tenant": "tenant-a", "sensor": "claroty" }`
  - `{ "event": "credential_accessed", "tenant": "tenant-a", "sensor": "crowdstrike", "caller": "mcp_tool:query_alerts" }`
- No credential values appear in any log entry (redacted as `"cs***et"`)
- Audit trail enables answering: "When was Tenant A's CrowdStrike token last used?"

**Repos Tested:** serveMyAPI (zero audit trail -- this is the gap being filled), axiathon (partial audit concept), tally (tracing patterns)

---

## State Checkpoint

```yaml
scenario_group: HS-004
title: Credential Lifecycle
scenarios: 6
priority: P0
repos_covered: [poller-cobra, poller-express, poller-bear, poller-coaster, serveMyAPI, axiathon, tally]
status: defined
```
