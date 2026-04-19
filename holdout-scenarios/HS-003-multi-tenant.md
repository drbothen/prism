---
document_type: holdout-scenario
level: L3
id: "HS-003"
category: "multi-tenant"
must_pass: true
priority: P1
epic_id: "E-0"
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T00:00:00
phase: 1b
inputs: []
input-hash: null
traces_to: prd.md
behavioral_contracts: []
lifecycle_status: active
introduced: cycle-1
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "Grouped scenario file — contains multiple sub-scenarios HS-NNN-MM. Split into individual files deferred."
---

# HS-003: Multi-Tenant Scenarios

**Group:** Ensuring Client A cannot see Client B's data
**Date:** 2026-04-13
**Priority:** P0 (MSSP requirement -- isolation is non-negotiable)

---

## HS-003-01: Tenant Data Isolation Under Normal Operation

**Title:** Two tenants querying the same sensor type receive only their own data

**Preconditions:**
- Tenant A and Tenant B both have CrowdStrike configured
- Different OAuth2 credentials per tenant (different client_id/client_secret)
- Both tenants' CrowdStrike instances contain distinct alert data

**Steps:**
1. MCP client acting as Tenant A calls `query_alerts` with `{ "tenant_id": "tenant-a", "sensor": "crowdstrike" }`
2. MCP client acting as Tenant B calls `query_alerts` with `{ "tenant_id": "tenant-b", "sensor": "crowdstrike" }`
3. Compare the two result sets

**Expected Outcome:**
- Tenant A receives only alerts from Tenant A's CrowdStrike instance
- Tenant B receives only alerts from Tenant B's CrowdStrike instance
- Zero overlap in alert IDs between the two result sets
- Each tenant's query uses its own OAuth2 token (separate token per tenant)
- Cursor state stored in isolated paths: `state/tenant-a/crowdstrike-alerts.json` vs `state/tenant-b/crowdstrike-alerts.json`
- Cache entries keyed by `(tenant_id, sensor_type)` -- Tenant A's cached results never served to Tenant B

**Repos Tested:** axiathon (9-layer tenant isolation model, TenantFilterRule concept), all pollers (single-tenant architecture being unified into multi-tenant)

---

## HS-003-02: Tenant ID Spoofing Prevention

**Title:** MCP tool parameters cannot override authenticated tenant identity

**Preconditions:**
- Tenant A authenticated via MCP session
- Tenant B's data exists in the system

**Steps:**
1. Tenant A's MCP client calls `query_alerts` with `{ "tenant_id": "tenant-b", "sensor": "crowdstrike" }`
2. Observe whether Prism returns Tenant B's data or rejects the request

**Expected Outcome:**
- Request REJECTED -- tenant context derived from authenticated session, not from tool parameters
- Error response: `"Permission denied: tenant context mismatch"` (or equivalent)
- No data from Tenant B returned
- Audit log entry: `{ "event": "tenant_spoofing_attempt", "authenticated_tenant": "tenant-a", "requested_tenant": "tenant-b", "action": "rejected" }`
- The `tenant_id` parameter in tool input is either ignored (overridden by session) or validated against session identity

**Repos Tested:** axiathon (TenantFilterRule -- optimizer-level isolation preventing OR bypass), mcp-claroty-xdome (session management pattern)

---

## HS-003-03: Cache Isolation Between Tenants

**Title:** Cached API responses are never served across tenant boundaries

**Preconditions:**
- Tenant A queries Claroty devices (response cached)
- Tenant B queries Claroty devices 1 second later (within cache TTL)

**Steps:**
1. Tenant A calls `query_devices` with `{ "tenant_id": "tenant-a", "sensor": "claroty", "limit": 50 }`
2. Response cached under key including tenant-a
3. Tenant B calls `query_devices` with `{ "tenant_id": "tenant-b", "sensor": "claroty", "limit": 50 }` within cache TTL
4. Verify Tenant B's request goes to Claroty API (cache miss, not cache hit)

**Expected Outcome:**
- Tenant B's request results in a Claroty API call (fresh fetch, not cached)
- Cache keys structured as `(tenant_id, sensor_type, query_hash)` -- never shared across tenants
- Each tenant has independent cache instances with independent TTL, size bounds, and eviction
- Cache size bounded per tenant (LRU eviction, fixing mcp-claroty-xdome's unbounded cache)
- Cache statistics (hit/miss) tracked per tenant

**Repos Tested:** mcp-claroty-xdome (per-service cache isolation pattern -- must be extended to per-tenant), axiathon (per-tenant file isolation)

---

## HS-003-04: Cursor State Isolation Between Tenants

**Title:** One tenant's cursor advancement does not affect another tenant's polling position

**Preconditions:**
- Tenant A and Tenant B both polling CrowdStrike alerts
- Both start from clean state (no prior cursor)

**Steps:**
1. Tenant A polls and receives 100 alerts, cursor advances to timestamp T100
2. Tenant B polls and receives 50 alerts, cursor advances to timestamp T50
3. Tenant A polls again -- should resume from T100
4. Tenant B polls again -- should resume from T50

**Expected Outcome:**
- Cursor files stored in per-tenant directories: `state/tenant-a/crowdstrike-alerts.json` (cursor at T100), `state/tenant-b/crowdstrike-alerts.json` (cursor at T50)
- TenantId validated in file path construction (no path traversal with malicious tenant names)
- Query fingerprint validated per tenant: Tenant A's fingerprint does not affect Tenant B
- Atomic file persistence per tenant (temp -> fsync -> rename) -- Tenant A's write never corrupts Tenant B's file
- Forward progress invariant enforced independently per tenant

**Repos Tested:** poller-bear (FileStore atomic write, per-source cursor isolation), poller-coaster (FileStore, query fingerprint)

---

## HS-003-05: Error Message Tenant Isolation

**Title:** Error messages for one tenant do not leak another tenant's information

**Preconditions:**
- Tenant A's CrowdStrike credentials are invalid (expired)
- Tenant B's CrowdStrike credentials are valid

**Steps:**
1. Tenant A queries CrowdStrike -- receives auth error
2. Tenant B queries CrowdStrike -- succeeds
3. Examine error message returned to Tenant A
4. Examine log output

**Expected Outcome:**
- Tenant A's error message says "Authentication failed for sensor: crowdstrike" (no URL, no credential, no client_id)
- Error does NOT contain: Tenant B's client_id, Tenant B's base URL, Tenant B's region
- Structured logs include `tenant_id: "tenant-a"` in the error span
- CRITICAL data (credentials) redacted in all log output: `"cs***et"` format (first 2 + *** + last 2)
- Error type carries `tenant_id` but redacts credential values in `Display` impl
- No error propagation path that could leak cross-tenant data

**Repos Tested:** poller-express (secret redaction: first2 + *** + last2), axiathon (error info leakage finding -- CWE-209), tally (thiserror with actionable messages)

---

## HS-003-06: Per-Tenant Rate Limiting Toward Sensor APIs

**Title:** One tenant's high volume does not exhaust another tenant's API quota

**Preconditions:**
- Tenant A configured with high-frequency polling (1s interval)
- Tenant B configured with standard polling (60s interval)
- Both share the same Armis sensor type (but different Armis instances)

**Steps:**
1. Tenant A starts rapid polling -- high API call volume
2. Rate limiter engages for Tenant A at per-tenant limit
3. Tenant B starts polling at normal rate
4. Verify Tenant B is not affected by Tenant A's rate limiting

**Expected Outcome:**
- Tenant A's API calls rate-limited independently
- Tenant B's API calls unaffected by Tenant A's rate state
- Rate limit tracking keyed by `(tenant_id, sensor_type)`, not globally
- When rate-limited, Tenant A receives backoff (not dropped data)
- HTTP 429 from sensor API handled with Retry-After header respect per tenant

**Repos Tested:** mcp-claroty-xdome (Python impl's 50 req/sec per client), all pollers (no upstream rate limiting -- Prism adds this)

---

## HS-003-07: Log Field Isolation and Filtering

**Title:** Structured logs enable per-tenant filtering without data leakage

**Preconditions:**
- Both Tenant A and Tenant B actively polling
- Structured logging with `tracing` crate, JSON output

**Steps:**
1. Both tenants generate log output simultaneously
2. Filter logs by `tenant_id = "tenant-a"`
3. Examine filtered output for any Tenant B data

**Expected Outcome:**
- Every log line includes `tenant_id` as a structured field (tracing span)
- Filtering by `tenant_id` yields only that tenant's log entries
- No log entry for Tenant A contains Tenant B's: URLs, credential fragments, alert data, cursor positions
- CRITICAL data never appears in any log entry regardless of tenant
- Log format is JSON (consistent with pollers' charmbracelet/log JSON formatter, but using tracing-subscriber in Rust)

**Repos Tested:** axiathon (per-tenant log isolation concept), tally (tracing with structured fields), all pollers (charmbracelet/log -- pattern migrated to tracing)

---

## State Checkpoint

```yaml
scenario_group: HS-003
title: Multi-Tenant
scenarios: 7
priority: P0
repos_covered: [poller-cobra, poller-express, poller-bear, poller-coaster, tally, axiathon, mcp-claroty-xdome]
status: defined
```
