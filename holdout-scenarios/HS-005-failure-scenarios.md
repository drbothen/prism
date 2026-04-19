---
document_type: holdout-scenario
level: L3
id: "HS-005"
category: "failure-scenarios"
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

# HS-005: Failure Scenarios

**Group:** Sensor API down, auth expired, rate limited, timeout
**Date:** 2026-04-13
**Priority:** P0

---

## HS-005-01: Sensor API Unreachable (Connection Refused)

**Title:** Sensor API completely down -- Prism retries with exponential backoff

**Preconditions:**
- Tenant A's CrowdStrike sensor configured
- CrowdStrike API unreachable (connection refused / DNS failure)
- Exponential backoff configured: 2s base, 30s max, 5 max retries

**Steps:**
1. Prism attempts to poll CrowdStrike for Tenant A
2. Connection refused on first attempt
3. Backoff engages: 2s delay
4. Second attempt fails -- 4s delay
5. Third attempt fails -- 8s delay
6. Fourth attempt fails -- 16s delay
7. Fifth attempt fails -- 30s delay (capped at max)
8. Max retries exceeded

**Expected Outcome:**
- Exponential backoff follows formula: `base_delay * 2^attempt`, capped at `max_delay`
- Delays: 2s, 4s, 8s, 16s, 30s (cap applies on 5th attempt)
- After max retries (5), Prism does NOT crash -- enters longer backoff cycle and retries
- When MaxRetries=0 configured, retries indefinitely (proven pattern from all pollers)
- Health endpoint reports CrowdStrike NOT_READY for Tenant A
- Other tenants and sensors unaffected
- Cursor state unchanged (no forward progress on failure)
- Structured log: `{ "event": "sensor_unreachable", "tenant": "tenant-a", "sensor": "crowdstrike", "attempt": 5, "next_retry_in": "30s" }`

**Repos Tested:** poller-cobra (backoff config: 2s/30s/5), poller-express (MaxRetries=0 = infinite), all pollers (exponential backoff pattern)

---

## HS-005-02: Authentication Expired Mid-Session

**Title:** CrowdStrike OAuth2 token expires during a poll batch

**Preconditions:**
- Tenant A polling CrowdStrike with valid OAuth2 token
- Token expires between step 1 (QueryV2) and step 2 (PostEntitiesAlertsV1) of two-step fetch

**Steps:**
1. Prism calls CrowdStrike QueryV2 for alert IDs -- succeeds (token still valid)
2. Token expires
3. Prism calls PostEntitiesAlertsV1 for alert details -- receives 401 Unauthorized
4. Prism detects auth failure, triggers token refresh
5. Prism retries PostEntitiesAlertsV1 with new token

**Expected Outcome:**
- 401 triggers automatic token refresh (not full backoff cycle)
- Retry with new token succeeds
- The two-step fetch (QueryV2 then PostEntitiesAlertsV1) completes correctly after refresh
- No partial data delivered -- either full batch or nothing
- Cursor not advanced until batch fully delivered
- Audit log: `{ "event": "oauth2_token_expired", "tenant": "tenant-a", "sensor": "crowdstrike", "action": "auto_refresh" }`

**Repos Tested:** poller-cobra (two-step fetch pattern, OAuth2 via gofalcon SDK -- Prism implements equivalent)

---

## HS-005-03: Sensor API Rate Limited (HTTP 429)

**Title:** Sensor API returns 429 Too Many Requests

**Preconditions:**
- Tenant A polling Armis at high frequency
- Armis API enforces rate limits

**Steps:**
1. Prism sends AQL query to Armis
2. Armis returns HTTP 429 with `Retry-After: 30` header
3. Prism respects Retry-After header
4. Prism retries after 30 seconds
5. Retry succeeds

**Expected Outcome:**
- HTTP 429 recognized as rate-limiting (not treated as generic server error)
- `Retry-After` header value respected (30s wait, not exponential backoff default)
- If no `Retry-After` header, fall back to exponential backoff
- Per-tenant rate limit tracking -- Tenant A's 429 does not affect Tenant B's polling schedule
- No data loss -- the request is retried, not dropped
- This fixes the gap in poller-bear (no 429 handling) and poller-coaster

**Repos Tested:** mcp-claroty-xdome (retry with backoff on 429/5xx), all pollers (no 429 handling -- Prism adds this)

---

## HS-005-04: Request Timeout

**Title:** Sensor API request times out after configured duration

**Preconditions:**
- Tenant A querying Claroty xDome
- Claroty API responding slowly (>30s for a single request)
- Request timeout configured at 30s

**Steps:**
1. Prism sends POST to `/api/v1/devices/` for Tenant A
2. Claroty API does not respond within 30s
3. Request times out
4. Prism retries with backoff

**Expected Outcome:**
- Request cancelled after timeout (30s, matching Claroty adapter config)
- Timeout treated as transient failure -- retried with exponential backoff
- Connection pool not poisoned by timed-out connection (proper cleanup)
- Response body drained on timeout to maintain HTTP/1.1 connection reuse (fixing poller-cobra's body-not-drained bug)
- Cursor state unchanged (no forward progress on timeout)
- Different sensors can have different timeout configurations (CrowdStrike: 15s, Claroty: 30s, etc.)

**Repos Tested:** poller-bear (Claroty 15-30s timeout), poller-cobra (response body not drained -- fixed), mcp-claroty-xdome (15s timeout)

---

## HS-005-05: Partial Batch Failure in Sink Delivery

**Title:** Some records in a batch fail to deliver to Vector sink

**Preconditions:**
- Prism has 100 enriched records ready for sink delivery
- Vector endpoint intermittently returning 503

**Steps:**
1. Prism batches 100 records for delivery
2. First sub-batch (records 1-50) delivered successfully
3. Second sub-batch (records 51-100) receives 503 from Vector
4. Prism retries sub-batch delivery
5. Retry succeeds

**Expected Outcome:**
- Records 1-50 not re-delivered (already confirmed)
- Records 51-100 retried until successful
- Per-record error attribution maintained -- Prism knows exactly which records failed
- Cursor advanced only after ALL records in the poll batch are confirmed delivered
- No data loss: failed records eventually delivered or moved to dead letter handling
- Batch delivery significantly more efficient than per-record POST (fixing all pollers' anti-pattern)

**Repos Tested:** poller-cobra (per-record delivery, sink retry), poller-bear (per-record delivery anti-pattern), poller-coaster (per-record delivery)

---

## HS-005-06: Malformed Sensor API Response

**Title:** Sensor API returns unexpected JSON structure

**Preconditions:**
- Tenant A querying Cyberint
- Cyberint API returns response with unknown fields (API version upgrade)

**Steps:**
1. Prism calls Cyberint alert API
2. Response contains all expected fields PLUS new unknown fields
3. Prism deserializes response

**Expected Outcome:**
- Deserialization succeeds -- unknown fields ignored (lenient parsing)
- This fixes poller-express's `DisallowUnknownFields` strict JSON decoding that breaks forward compatibility
- Known fields parsed and normalized to OCSF
- Unknown fields optionally captured in `unmapped` JSON blob (axiathon's vendor extension pattern)
- Polymorphic types handled: IDs as string OR number (poller-bear's pattern)
- CyberintTime multi-format parsing works: RFC3339, no-timezone, microseconds, null/empty

**Repos Tested:** poller-express (strict JSON decoding bug -- fixed), poller-bear (polymorphic JSON handling), axiathon (vendor extension via unmapped)

---

## HS-005-07: MCP Client Disconnection During Long Query

**Title:** MCP client disconnects while Prism is fetching data from sensor

**Preconditions:**
- MCP client connected via stdio
- Client initiates a large query spanning multiple sensor API pages

**Steps:**
1. Client calls `query_alerts` with large limit
2. Prism begins paginating through sensor API
3. Client disconnects (stdin EOF / pipe broken)
4. Prism detects disconnection

**Expected Outcome:**
- In-flight sensor API calls cancelled (context cancellation propagation)
- No orphaned HTTP connections to sensor APIs
- Cursor state NOT advanced (incomplete query -- no partial commit)
- Server remains stable for other clients
- Session cleaned up properly (fixing mcp-claroty-xdome's session accumulation bug)
- No memory leak from abandoned query state

**Repos Tested:** tally (stdio transport, ServerHandler), mcp-claroty-xdome (session management, no expiration bug -- fixed)

---

## State Checkpoint

```yaml
scenario_group: HS-005
title: Failure Scenarios
scenarios: 7
priority: P0
repos_covered: [poller-cobra, poller-express, poller-bear, poller-coaster, tally, axiathon, mcp-claroty-xdome]
status: defined
```
