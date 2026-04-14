# Pass 3 Deep: Behavioral Contracts -- mcp-claroty-xdome (Round 1)

## Overview

This deepening round extracts behavioral contracts from test files (Vitest), Zod schemas, error types, and service implementations. The broad sweep identified 13 contracts (BC-001 through BC-013). This round provides a systematic, exhaustive extraction using the BC-S.SS.NNN format organized by subsystem.

---

## Subsystem Key

| Code | Subsystem |
|------|-----------|
| 1 | Integration Layer (XDomeApiClient) |
| 2 | Domain Services (caching + delegation) |
| 3 | Tool Handlers (MCP tool interface) |
| 4 | Schema Validation (Zod contracts) |
| 5 | Session Management |
| 6 | Cache Infrastructure |
| 7 | Error Hierarchy |
| 8 | Transport Layer |
| 9 | Server Lifecycle |

---

## Subsystem 1: Integration Layer (XDomeApiClient)

### BC-1.01.001: Environment variable validation on construction

**Preconditions:** XDomeApiClient is instantiated
**Postconditions:** If `CLAROTY_XDOME_BASE_URL` or `CLAROTY_XDOME_API_TOKEN` are missing, throws `McpError` with code `-32009` (InvalidConfiguration)
**Error Cases:** Both variables missing -> single error; either missing -> same error
**Evidence:** `xdome-api-client.ts:34-42`
**Test Evidence:** Implicit in `xdome-api-client.test.ts:53-59` -- test sets both env vars in beforeEach, constructor succeeds
**Confidence:** HIGH (from code, constructor guard)

### BC-1.01.002: Bearer token authentication on all requests

**Preconditions:** XDomeApiClient has been constructed successfully
**Postconditions:** Every HTTP request includes header `Authorization: Bearer <CLAROTY_XDOME_API_TOKEN>`
**Evidence:** `xdome-api-client.ts:52-58` (axios.create with Authorization header)
**Confidence:** HIGH (from code)

### BC-1.01.003: Request timeout of 15 seconds

**Preconditions:** Any API request is made
**Postconditions:** Request will timeout after 15,000ms
**Evidence:** `xdome-api-client.ts:59` (`timeout: 15000`)
**Confidence:** HIGH (from code, no specific test)

### BC-1.02.001: Retry on 429 and 5xx with linear backoff

**Preconditions:** API request fails with HTTP 429 or status >= 500
**Postconditions:** Request retried up to 3 times with delays of 1s, 2s, 3s (linear: retryCount * 1000)
**Non-retryable:** 4xx errors (except 429) are NOT retried
**Evidence:** `xdome-api-client.ts:64-79`
**Confidence:** HIGH (from code)

### BC-1.03.001: HTTP 401/403 mapped to AuthenticationError

**Preconditions:** xDome API returns HTTP 401 or 403
**Postconditions:** Throws `AuthenticationError` (code -32001) with default message "Authentication failed"
**Evidence:** `xdome-api-client.ts:215-217`
**Test Evidence:** `xdome-api-client.test.ts:102-109` (401), `xdome-api-client-alerted-devices.test.ts:104-111` (401)
**Confidence:** HIGH (from tests)

### BC-1.03.002: HTTP 404 mapped to NotFoundError with endpoint URL

**Preconditions:** xDome API returns HTTP 404
**Postconditions:** Throws `NotFoundError` with message `"xDome API endpoint with id '<url>' not found."` and data `{resource: "xDome API endpoint", id: url}`
**Evidence:** `xdome-api-client.ts:218-219`
**Test Evidence:** `xdome-api-client.test.ts:111-128` (asserts message format), `xdome-api-client-alerted-devices.test.ts:113-130`
**Confidence:** HIGH (from tests)

### BC-1.03.003: HTTP 422 mapped to ValidationError

**Preconditions:** xDome API returns HTTP 422
**Postconditions:** Throws `ValidationError` (code -32602) with message "Invalid parameters for xDome API request." and `{cause: error}` data
**Evidence:** `xdome-api-client.ts:220-226`
**Test Evidence:** `xdome-api-client.test.ts:180-190` (devices), `xdome-api-client.test.ts:240-250` (vulnerabilities), `xdome-api-client-alerted-devices.test.ts:132-142`
**Confidence:** HIGH (from tests)

### BC-1.03.004: Other HTTP errors mapped to IntegrationError

**Preconditions:** xDome API returns any status code not in {401, 403, 404, 422}
**Postconditions:** Throws `IntegrationError` (code -32007) with message "Error communicating with Claroty xDome API" and `{cause: error}` data
**Evidence:** `xdome-api-client.ts:227-228`
**Test Evidence:** `xdome-api-client.test.ts:130-137` (500), `xdome-api-client-alerted-devices.test.ts:144-151` (500)
**Confidence:** HIGH (from tests)

### BC-1.03.005: Non-Axios errors mapped to IntegrationError

**Preconditions:** Non-Axios error thrown during API call (e.g., network error, TypeError)
**Postconditions:** Throws `IntegrationError` with same message and `{cause: error}` data
**Evidence:** `xdome-api-client.ts:231-233`
**Test Evidence:** `xdome-api-client.test.ts:94-100` ("Network Error" test)
**Confidence:** HIGH (from tests)

### BC-1.04.001: getAlerts POSTs to /api/v1/alerts

**Preconditions:** Called with GetAlertsParameters
**Postconditions:** Performs `POST /api/v1/alerts` with parameters as JSON body; returns `GetAlertsResponse`
**Evidence:** `xdome-api-client.ts:87-88`
**Test Evidence:** `xdome-api-client.test.ts:70-92`
**Confidence:** HIGH (from tests)

### BC-1.04.002: getDevices POSTs to /api/v1/devices with group_by field adjustment

**Preconditions:** Called with GetDevicesInput
**Postconditions:** 
- If `group_by` is specified and non-empty: `fields` in the request payload is REPLACED with the `group_by` array values
- If `group_by` is absent/empty: `fields` is passed through unchanged
- Performs `POST /api/v1/devices`
**Side effect protection:** Creates a shallow copy of params (`{...params}`) before mutation to avoid modifying the caller's object
**Evidence:** `xdome-api-client.ts:106-121`
**Test Evidence:** `get-devices-handler.test.ts:88-100` (group_by test, but tests handler not client directly)
**Confidence:** MEDIUM (group_by behavior from code only, no direct unit test on API client for field replacement)

### BC-1.04.003: getAlertedDevices POSTs to /api/v1/alerts/{alertId}/devices

**Preconditions:** Called with alertId (string) and GetAlertedDevicesParameters
**Postconditions:** Performs `POST /api/v1/alerts/${alertId}/devices` with parameters as JSON body
**Evidence:** `xdome-api-client.ts:142-144`
**Test Evidence:** `xdome-api-client-alerted-devices.test.ts:86-93` (asserts exact URL with alertId interpolation)
**Confidence:** HIGH (from tests)

### BC-1.04.004: getVulnerabilities POSTs to /api/v1/vulnerabilities

**Preconditions:** Called with GetVulnerabilitiesParameters
**Postconditions:** Performs `POST /api/v1/vulnerabilities` with parameters as JSON body
**Evidence:** `xdome-api-client.ts:163-164`
**Test Evidence:** `xdome-api-client.test.ts:202-220`
**Confidence:** HIGH (from tests)

### BC-1.04.005: getVulnerabilityDevices POSTs to /api/v1/vulnerabilities/{vulnerabilityId}/devices

**Preconditions:** Called with vulnerabilityId (string) and params (Omit<GetVulnerabilityDevicesInput, "vulnerability_id">)
**Postconditions:** Performs `POST /api/v1/vulnerabilities/${vulnerabilityId}/devices`
**Evidence:** `xdome-api-client.ts:187-189`
**Confidence:** HIGH (from code, consistent with alerted devices pattern)

### BC-1.04.006: Complex compound filters are passed through to API

**Preconditions:** getAlertedDevices called with compound filter (nested and/or operands)
**Postconditions:** The compound filter object is passed as-is in the POST body
**Evidence:** `xdome-api-client-alerted-devices.test.ts:153-179`
**Confidence:** HIGH (from tests)

---

## Subsystem 2: Domain Services (Caching + Delegation)

### BC-2.01.001: AlertService cache key is JSON.stringify(params)

**Preconditions:** `findAlerts(params)` is called
**Postconditions:** Cache lookup uses `JSON.stringify(params)` as key
**Evidence:** `alert-service.ts:45`
**Test Evidence:** `alert-service.test.ts:138-139` (asserts exact cache key)
**Confidence:** HIGH (from tests)

### BC-2.01.002: AlertService returns cached response without API call

**Preconditions:** `findAlerts(params)` called and cache.get returns non-null
**Postconditions:** Returns cached response; `apiClient.getAlerts` is NOT called
**Evidence:** `alert-service.ts:47-54`
**Test Evidence:** `alert-service.test.ts:126-142` (asserts `not.toHaveBeenCalled`)
**Confidence:** HIGH (from tests)

### BC-2.01.003: AlertService fetches from API on cache miss and stores with 300000ms TTL

**Preconditions:** `findAlerts(params)` called and cache.get returns null
**Postconditions:** Calls `apiClient.getAlerts(params)`, stores response via `cache.set(key, response, 300000)`, returns response
**Evidence:** `alert-service.ts:57-59`
**Test Evidence:** `alert-service.test.ts:144-165` (asserts cache.set called with 300000)
**Confidence:** HIGH (from tests)

### BC-2.01.004: AlertService does not cache errors

**Preconditions:** `findAlerts(params)` called, cache miss, API throws error
**Postconditions:** Error propagates to caller; `cache.set` is NOT called
**Evidence:** `alert-service.ts:57` (await without try/catch)
**Test Evidence:** `alert-service.test.ts:207-226` (asserts `set.not.toHaveBeenCalled`)
**Confidence:** HIGH (from tests)

### BC-2.01.005: AlertService uses distinct cache keys for distinct params

**Preconditions:** `findAlerts` called with two different parameter objects
**Postconditions:** Two different cache keys are generated and stored
**Evidence:** `alert-service.test.ts:168-204`
**Confidence:** HIGH (from tests)

### BC-2.02.001: AlertedDeviceService cache key includes alertId

**Preconditions:** `findAlertedDevices(alertId, params)` called
**Postconditions:** Cache key is `JSON.stringify({alertId, ...params})` -- alertId is part of the key
**Evidence:** `alerted-device-service.ts:48`
**Test Evidence:** `alerted-device-service.test.ts:163-164` (asserts key includes alertId)
**Confidence:** HIGH (from tests)

### BC-2.02.002: AlertedDeviceService uses distinct keys for different alertIds

**Preconditions:** Same params but different alertIds
**Postconditions:** Different cache keys, different cache.set calls
**Evidence:** `alerted-device-service.test.ts:204-239`
**Confidence:** HIGH (from tests)

### BC-2.03.001: DeviceService delegates to apiClient.getDevices unchanged

**Preconditions:** `findDevices(filters)` called, cache miss
**Postconditions:** Calls `apiClient.getDevices(filters)` with exact same filters object
**Evidence:** `device-service.ts:57`
**Test Evidence:** `device-service.test.ts:44-63`
**Confidence:** HIGH (from tests)

### BC-2.04.001: VulnerabilityService (vulnerabilities/) delegates to apiClient.getVulnerabilities

**Preconditions:** `findVulnerabilities(params)` called, cache miss
**Postconditions:** Calls `apiClient.getVulnerabilities(params)` with exact params
**Evidence:** `domain/vulnerabilities/vulnerability-service.ts:57`
**Test Evidence:** `domain/vulnerabilities/vulnerability-service.test.ts:41-49`
**Confidence:** HIGH (from tests)

### BC-2.05.001: VulnerabilityService (alerts/) decomposes vulnerability_id from params

**Preconditions:** `findVulnerabilityDevices(params)` called where params includes vulnerability_id
**Postconditions:** Destructures `{vulnerability_id, ...apiParams} = params`, calls `apiClient.getVulnerabilityDevices(vulnerability_id, apiParams)`
**Evidence:** `domain/alerts/vulnerability-service.ts:41-43`
**Test Evidence:** `domain/alerts/vulnerability-service.test.ts:41-65` (asserts separate args)
**Confidence:** HIGH (from tests)

### BC-2.05.002: VulnerabilityService (alerts/) cache key is JSON.stringify of FULL params including vulnerability_id

**Preconditions:** `findVulnerabilityDevices(params)` called
**Postconditions:** Cache key = `JSON.stringify(params)` where params INCLUDES vulnerability_id
**Evidence:** `domain/alerts/vulnerability-service.ts:27`
**Test Evidence:** `domain/alerts/vulnerability-service.test.ts:176-201`
**Confidence:** HIGH (from tests)

### BC-2.06.001: All domain services propagate API errors transparently

**Preconditions:** Any domain service method called, API throws error
**Postconditions:** Error propagates unmodified to caller (no wrapping, no catching, no transformation)
**Evidence:** All 5 service files -- none have try/catch blocks
**Test Evidence:** `alert-service.test.ts:89-99`, `device-service.test.ts:107-122`, `alerted-device-service.test.ts:106-116`, `vulnerability-service.test.ts (both):112-129` and `77-87`
**Confidence:** HIGH (from tests, consistent across all services)

### BC-2.06.002: All domain services use identical 300000ms cache TTL

**Preconditions:** Any domain service caches a response
**Postconditions:** TTL is exactly 300000ms (5 minutes) -- hardcoded in each service
**Evidence:** All service files: `cache.set(key, response, 300000)`
**Test Evidence:** All service test files assert `300000` as third argument to `cache.set`
**Confidence:** HIGH (from tests)

---

## Subsystem 3: Tool Handlers

### BC-3.01.001: GetAlertsToolHandler tool name is "get_alerts"

**Preconditions:** Handler instantiated
**Postconditions:** `handler.name === "get_alerts"`
**Test Evidence:** `get-alerts-handler.test.ts:22-25`
**Confidence:** HIGH (from tests)

### BC-3.01.002: GetAlertsToolHandler passes args directly to AlertService.findAlerts

**Preconditions:** `handle(args)` called with validated input
**Postconditions:** Calls `alertService.findAlerts(args)` with exact args object
**Test Evidence:** `get-alerts-handler.test.ts:27-41`
**Confidence:** HIGH (from tests)

### BC-3.01.003: GetAlertsToolHandler returns JSON-stringified response as text content

**Preconditions:** AlertService returns response
**Postconditions:** Returns `{content: [{type: "text", text: JSON.stringify(response)}]}`
**Test Evidence:** `get-alerts-handler.test.ts:43-82` (asserts exact structure)
**Confidence:** HIGH (from tests)

### BC-3.01.004: GetAlertsToolHandler propagates service errors unmodified

**Preconditions:** AlertService.findAlerts throws McpError
**Postconditions:** Same error is thrown from handle()
**Test Evidence:** `get-alerts-handler.test.ts:107-122`
**Confidence:** HIGH (from tests)

### BC-3.02.001: GetAlertedDevicesToolHandler decomposes alert_id from args

**Preconditions:** `handle(args)` called where args contains `alert_id`
**Postconditions:** Destructures `{alert_id, ...params} = args`, calls `alertedDeviceService.findAlertedDevices(alert_id, params)`
**Evidence:** `get-alerted-devices-handler.ts:32-34`
**Test Evidence:** `get-alerted-devices-handler.test.ts:29-53` (asserts separate arguments)
**Confidence:** HIGH (from tests)

### BC-3.02.002: GetAlertedDevicesToolHandler handles multi-field sort_by

**Preconditions:** args contains sort_by array with multiple entries
**Postconditions:** Entire sort_by array passed through to service
**Test Evidence:** `get-alerted-devices-handler.test.ts:163-195`
**Confidence:** HIGH (from tests)

### BC-3.02.003: GetAlertedDevicesToolHandler handles pagination

**Preconditions:** args contains limit=25, offset=50, include_count=true
**Postconditions:** All pagination params passed through; response text contains correct count
**Test Evidence:** `get-alerted-devices-handler.test.ts:197-223`
**Confidence:** HIGH (from tests)

### BC-3.03.001: GetDevicesToolHandler passes group_by through to service

**Preconditions:** args contains group_by field
**Postconditions:** Full args including group_by passed to `deviceService.findDevices(args)`
**Test Evidence:** `get-devices-handler.test.ts:88-100`
**Confidence:** HIGH (from tests)

### BC-3.04.001: GetVulnerabilityDevicesToolHandler passes full args to service (no decomposition)

**Preconditions:** `handle(args)` called
**Postconditions:** Calls `vulnerabilityService.findVulnerabilityDevices(args)` with full args INCLUDING vulnerability_id
**Evidence:** `get-vulnerability-devices-handler.ts:32-33`
**Test Evidence:** `get-vulnerability-devices-handler.test.ts:28-46`
**Confidence:** HIGH (from tests)

### BC-3.04.002: GetVulnerabilityDevicesToolHandler handles vulnerability-specific fields

**Preconditions:** args.fields contains vulnerability_relevance, vulnerability_source, vulnerability_last_updated
**Postconditions:** These fields are accepted by the Zod schema and passed through
**Test Evidence:** `get-vulnerability-devices-handler.test.ts:128-149`
**Confidence:** HIGH (from tests)

### BC-3.05.001: All tool handlers follow identical response format

**Preconditions:** Any tool handler's handle() returns
**Postconditions:** Returns `{content: [{type: "text", text: JSON.stringify(serviceResult)}]}`
**Evidence:** All 5 handler files follow the same pattern
**Test Evidence:** Each handler test has a "should return the result" test case asserting this format
**Confidence:** HIGH (from tests, 5/5 consistent)

### BC-3.05.002: All tool handlers propagate errors transparently

**Preconditions:** Domain service throws any error
**Postconditions:** Error propagates unmodified from handle()
**Evidence:** No try/catch in any handler
**Test Evidence:** Each handler test has an "error propagation" test case
**Confidence:** HIGH (from tests, 5/5 consistent)

---

## Subsystem 4: Schema Validation (Zod Contracts)

### BC-4.01.001: fields array requires at least 1 element

**Preconditions:** Any tool input schema validation
**Postconditions:** `fields` must be `.min(1)` -- empty array rejected
**Evidence:** All 5 schemas: `z.array(...).min(1)`
**Confidence:** HIGH (from schema definition)

### BC-4.01.002: limit defaults to 100, constrained 0-5000

**Preconditions:** Tool input validation
**Postconditions:** `limit` defaults to 100 if not specified; valid range is `.min(0).max(5000)`
**Evidence:** All 5 schemas
**Confidence:** HIGH (from schema definition)

### BC-4.01.003: offset defaults to 0

**Preconditions:** Tool input validation
**Postconditions:** `offset` defaults to 0 if not specified
**Evidence:** All 5 schemas: `.default(0)`
**Confidence:** HIGH (from schema definition)

### BC-4.01.004: include_count defaults to false

**Preconditions:** Tool input validation
**Postconditions:** `include_count` defaults to false if not specified
**Evidence:** All 5 schemas: `.default(false)`
**Confidence:** HIGH (from schema definition)

### BC-4.02.001: Alerts sort_by defaults to [{field: "id", order: "asc"}]

**Preconditions:** get_alerts called without sort_by
**Postconditions:** Default sort is ascending by alert id
**Evidence:** `get-alerts-schema.ts:88-89`
**Confidence:** HIGH (from schema)

### BC-4.02.002: Vulnerabilities sort_by defaults to [{field: "published_date", order: "desc"}]

**Preconditions:** get_vulnerabilities called without sort_by
**Postconditions:** Default sort is descending by published_date (newest first)
**Evidence:** `get-vulnerabilities-schema.ts:137-138`
**Confidence:** HIGH (from schema)

### BC-4.02.003: Devices, AlertedDevices, VulnerabilityDevices have no sort_by default

**Preconditions:** Tool called without sort_by
**Postconditions:** sort_by is optional with no default -- undefined is passed to API
**Evidence:** `get-devices-schema.ts:242-246`, `get-alerted-devices-schema.ts:420-425`, `get-vulnerability-devices-schema.ts:228-233`
**Confidence:** HIGH (from schema)

### BC-4.03.001: Alerts schema rejects unknown properties

**Preconditions:** get_alerts input contains properties not in the schema
**Postconditions:** Zod `.strict()` mode rejects with validation error
**Evidence:** `get-alerts-schema.ts:115`
**Confidence:** HIGH (from schema)

### BC-4.03.002: Devices and VulnerabilityDevices schemas accept unknown properties

**Preconditions:** get_devices or get_vulnerability_devices input contains extra properties
**Postconditions:** Extra properties are silently accepted (no `.strict()`)
**Evidence:** `get-devices-schema.ts:228`, `get-vulnerability-devices-schema.ts:217`
**Confidence:** HIGH (from schema)

### BC-4.04.001: filter_by supports recursive compound filters

**Preconditions:** filter_by contains nested `{operation: "and"|"or", operands: [...]}` structure
**Postconditions:** Recursion is handled via `z.lazy()` -- arbitrary depth supported
**Evidence:** All 5 schemas define CompoundQueryFilter with z.lazy()
**Test Evidence:** Multiple handler tests verify compound filters pass through (e.g., `get-alerts-handler.test.ts:84-105`)
**Confidence:** HIGH (from schema + tests)

### BC-4.05.001: get_alerted_devices requires alert_id string parameter

**Preconditions:** get_alerted_devices tool input
**Postconditions:** `alert_id: z.string()` is required (not optional)
**Evidence:** `get-alerted-devices-schema.ts:413`
**Confidence:** HIGH (from schema)

### BC-4.05.002: get_vulnerability_devices requires vulnerability_id string parameter

**Preconditions:** get_vulnerability_devices tool input
**Postconditions:** `vulnerability_id: z.string()` is required (not optional)
**Evidence:** `get-vulnerability-devices-schema.ts:218`
**Confidence:** HIGH (from schema)

### BC-4.06.001: get_devices supports group_by parameter

**Preconditions:** get_devices input includes group_by
**Postconditions:** `group_by` must be array of DeviceFieldsEnum values, `.min(1)` when present
**Evidence:** `get-devices-schema.ts:223-226`
**Confidence:** HIGH (from schema)

---

## Subsystem 5: Session Management

### BC-5.01.001: Sessions are created with UUID and matching creation/access timestamps

**Preconditions:** `createSession()` called
**Postconditions:** Returns `SessionData` with `sessionId` = UUID, `creationTimestamp` = `lastAccessTimestamp` = `Date.now()`
**Evidence:** `in-memory-session-manager.ts:15-22`
**Test Evidence:** `in-memory-session-manager.test.ts:10-17`
**Confidence:** HIGH (from tests)

### BC-5.01.002: getSession returns undefined for unknown IDs

**Preconditions:** `getSession("non-existent-id")` called
**Postconditions:** Returns undefined (not error)
**Test Evidence:** `in-memory-session-manager.test.ts:27-29`
**Confidence:** HIGH (from tests)

### BC-5.01.003: touchSession updates lastAccessTimestamp

**Preconditions:** `touchSession(sessionId)` called for existing session
**Postconditions:** `lastAccessTimestamp` is updated to current time; `creationTimestamp` unchanged
**Test Evidence:** `in-memory-session-manager.test.ts:31-46`
**Confidence:** HIGH (from tests)

### BC-5.01.004: touchSession is no-op for non-existent sessions

**Preconditions:** `touchSession("non-existent-id")` called
**Postconditions:** Does not throw, no side effects
**Test Evidence:** `in-memory-session-manager.test.ts:48-51`
**Confidence:** HIGH (from tests)

### BC-5.01.005: deleteSession removes session; no-op for non-existent

**Preconditions:** `deleteSession(sessionId)` called
**Postconditions:** Session no longer retrievable via getSession; does not throw for unknown IDs
**Test Evidence:** `in-memory-session-manager.test.ts:53-63`, `65-67`
**Confidence:** HIGH (from tests)

### BC-5.02.001: SessionOrchestrator creates new session when no sessionId provided

**Preconditions:** `getSessionConnection(undefined, factory)` called
**Postconditions:** Creates new session, creates new connection via factory, registers connection
**Test Evidence:** `session-orchestrator.test.ts:33-54`
**Confidence:** HIGH (from tests)

### BC-5.02.002: SessionOrchestrator reuses existing connection for known session

**Preconditions:** `getSessionConnection(sessionId, factory)` where session exists AND connection exists
**Postconditions:** Returns existing connection; factory is NOT called; addConnection is NOT called; session is touched
**Test Evidence:** `session-orchestrator.test.ts:85-109`
**Confidence:** HIGH (from tests)

### BC-5.02.003: SessionOrchestrator creates new connection for existing session without connection

**Preconditions:** `getSessionConnection(sessionId, factory)` where session exists but connection does not
**Postconditions:** Session touched; new connection created via factory; connection registered
**Test Evidence:** `session-orchestrator.test.ts:56-83`
**Confidence:** HIGH (from tests)

### BC-5.02.004: SessionOrchestrator creates new session for invalid sessionId

**Preconditions:** `getSessionConnection("invalid-id", factory)` where session does not exist
**Postconditions:** getSession returns undefined -> creates new session, new connection
**Test Evidence:** `session-orchestrator.test.ts:111-134`
**Confidence:** HIGH (from tests)

---

## Subsystem 6: Cache Infrastructure

### BC-6.01.001: Cache returns null for non-existent keys

**Preconditions:** `get(key)` where key was never set
**Postconditions:** Returns null
**Test Evidence:** `cache.test.ts:32-35`
**Confidence:** HIGH (from tests)

### BC-6.01.002: Cache returns null for expired entries and evicts them

**Preconditions:** `get(key)` where entry TTL has elapsed
**Postconditions:** Returns null; entry is deleted from the underlying Map
**Test Evidence:** `cache.test.ts:37-48`
**Confidence:** HIGH (from tests)

### BC-6.01.003: Cache default TTL is 300000ms (5 minutes)

**Preconditions:** `set(key, data)` called without explicit TTL
**Postconditions:** Entry stored with TTL = 300000ms
**Evidence:** `cache.ts:45` (`ttlMs: number = 300000`)
**Test Evidence:** `cache.test.ts:11-18` (default TTL, data retrievable)
**Confidence:** HIGH (from code + tests)

### BC-6.01.004: Zero or negative TTL causes immediate expiration

**Preconditions:** `set(key, data, 0)` or `set(key, data, -1000)`
**Postconditions:** Immediate `get` returns null (entry treated as expired)
**Evidence:** `cache.ts:72` (`entry.ttl <= 0`)
**Test Evidence:** `cache.test.ts:261-279` (zero TTL), `cache.test.ts:274-279` (negative TTL)
**Confidence:** HIGH (from tests)

### BC-6.01.005: Cache set overwrites existing entries

**Preconditions:** `set(key, data1)` then `set(key, data2)`
**Postconditions:** `get(key)` returns data2
**Test Evidence:** `cache.test.ts:67-77`
**Confidence:** HIGH (from tests)

### BC-6.01.006: Cache has() returns false for expired entries and evicts them

**Preconditions:** `has(key)` where entry TTL has elapsed
**Postconditions:** Returns false; entry deleted from Map; size decremented
**Test Evidence:** `cache.test.ts:96-121`
**Confidence:** HIGH (from tests)

### BC-6.01.007: Cache size() includes expired entries until accessed

**Preconditions:** Expired entries exist but have not been accessed via get/has/cleanup
**Postconditions:** `size()` returns count including expired entries (lazy eviction)
**Test Evidence:** `cache.test.ts:137-151`
**Confidence:** HIGH (from tests)

### BC-6.01.008: Cache cleanup() removes only expired entries and returns count

**Preconditions:** Mix of expired and valid entries
**Postconditions:** Expired entries removed; valid entries retained; returns count of removed entries
**Test Evidence:** `cache.test.ts:191-213`
**Confidence:** HIGH (from tests)

### BC-6.01.009: Cache handles 10,000+ entries without error

**Preconditions:** 10,000 entries stored
**Postconditions:** All entries retrievable; no performance degradation within test timeout
**Test Evidence:** `cache.test.ts:340-352`
**Confidence:** HIGH (from tests)

---

## Subsystem 7: Error Hierarchy

### BC-7.01.001: McpError captures stack trace excluding constructor

**Preconditions:** Any McpError subclass instantiated
**Postconditions:** Error has `code`, `data`, `message`, `name` (= constructor name), stack trace excluding the constructor frame
**Evidence:** `errors.ts:38-46` (`Error.captureStackTrace(this, this.constructor)`)
**Test Evidence:** `errors.test.ts:18-26`
**Confidence:** HIGH (from tests)

### BC-7.01.002: NotFoundError formats message with resource name and optional id

**Preconditions:** `new NotFoundError("User", "123")`
**Postconditions:** `message = "User with id '123' not found."`, `data = {resource: "User", id: "123"}`
**Preconditions (no id):** `new NotFoundError("Resource")`
**Postconditions:** `message = "Resource not found."`
**Test Evidence:** `errors.test.ts:39-52`
**Confidence:** HIGH (from tests)

### BC-7.01.003: ToolNotFoundError extends NotFoundError with custom name

**Preconditions:** `new ToolNotFoundError("get_devices")`
**Postconditions:** `message = "Tool with name 'get_devices' not found."`, `name = "ToolNotFoundError"`, `code = -32003`, `data = {resource: "Tool", id: "get_devices"}`
**Test Evidence:** `errors.test.ts:104-113`
**Confidence:** HIGH (from tests)

### BC-7.01.004: IntegrationError formats message with service name

**Preconditions:** `new IntegrationError("Claroty API", {detail: "Connection refused"})`
**Postconditions:** `message = "Error communicating with Claroty API"`, `code = -32007`
**Test Evidence:** `errors.test.ts:64-73`
**Confidence:** HIGH (from tests)

### BC-7.01.005: DatabaseError inherits from ServerError

**Preconditions:** `new DatabaseError(...)`
**Postconditions:** `instanceof ServerError === true`, `code = -32008`
**Test Evidence:** `errors.test.ts:140-150`
**Confidence:** HIGH (from tests)

---

## Subsystem 8: Transport Layer

### BC-8.01.001: ToolRegistry throws ToolNotFoundError for unknown tool names

**Preconditions:** `registry.get("non_existent_tool")`
**Postconditions:** Throws `ToolNotFoundError` with message "Tool with name 'non_existent_tool' not found."
**Test Evidence:** `tool-registry.test.ts:45-50`
**Confidence:** HIGH (from tests)

### BC-8.01.002: ToolRegistry warns and overwrites on duplicate registration

**Preconditions:** `register(handler1)` then `register(handler2)` with same name
**Postconditions:** `console.warn` called; second handler replaces first
**Test Evidence:** `tool-registry.test.ts:52-68`
**Confidence:** HIGH (from tests)

### BC-8.01.003: ToolRegistry.getAll() returns all registered handlers

**Preconditions:** Multiple handlers registered
**Postconditions:** Returns IterableIterator containing all handlers
**Test Evidence:** `tool-registry.test.ts:70-79`
**Confidence:** HIGH (from tests)

### BC-8.01.004: ToolRegistry.unregisterAll() clears all handlers

**Preconditions:** Handlers registered
**Postconditions:** All handlers removed; getAll returns empty; get throws ToolNotFoundError
**Test Evidence:** `tool-registry.test.ts:81-90`
**Confidence:** HIGH (from tests)

---

## Subsystem 9: Server Lifecycle

### BC-9.01.001: Server refuses to start twice

**Preconditions:** `server.start(port)` called when `isRunning === true`
**Postconditions:** Returns immediately (no-op), logs warning
**Evidence:** `mcp-server-instance.ts:113-116`
**Confidence:** MEDIUM (from code, no direct test for double-start)

### BC-9.01.002: Health endpoint returns structured diagnostics

**Preconditions:** `GET /health` after server started
**Postconditions:** Returns JSON with `{status: "ok", timestamp, version, uptime: {seconds, human}, memory: {used, total, external, unit: "MB"}, environment: {nodeVersion, platform, arch}, dependencies: {claroty_api: "configured"|"not_configured"}}`
**Evidence:** `mcp-server-instance.ts:127-161`
**Test Evidence:** `server-startup.integration.test.ts:30-45`
**Confidence:** HIGH (from e2e test)

### BC-9.01.003: Server registers SIGINT and SIGTERM handlers for graceful shutdown

**Preconditions:** `main()` completes successfully
**Postconditions:** Signal handlers registered that call `server.stop()` then `process.exit(0)`
**Evidence:** `main.ts:29-38`
**Confidence:** HIGH (from code)

### BC-9.01.004: Graceful shutdown closes transports, then MCP server

**Preconditions:** `server.stop()` called
**Postconditions:** Sequence: HTTP server close -> transportManager.closeAll() -> mcpServer.close() -> isRunning = false
**Evidence:** `mcp-server-instance.ts:174-216`
**Confidence:** HIGH (from code)

### BC-9.01.005: Uncaught exceptions trigger fatal error handler

**Preconditions:** Uncaught exception occurs
**Postconditions:** `handleFatalError` called with Error object, which logs at fatal level and calls `process.exit(1)`
**Evidence:** `main.ts:7-11`
**Confidence:** HIGH (from code)

### BC-9.01.006: Factory validates environment variables before DI setup

**Preconditions:** `createAndInitializeServer()` called
**Postconditions:** If CLAROTY_XDOME_BASE_URL or CLAROTY_XDOME_API_TOKEN missing, throws `McpError(-32009)` BEFORE any DI registration
**Evidence:** `factory.ts:35-46` (validateEnvVariables called first)
**Test Evidence:** Implicit in `server-startup.integration.test.ts` (sets env vars in process.env)
**Confidence:** HIGH (from code)

### BC-9.01.007: Factory enables CORS with wildcard origin and Mcp-Session-Id exposure

**Preconditions:** Server created
**Postconditions:** `cors({origin: "*", exposedHeaders: ["Mcp-Session-Id"]})` applied to Express app
**Evidence:** `factory.ts:145-150`
**Confidence:** HIGH (from code)

### BC-9.01.008: Factory configures transports from MCP_TRANSPORT_TYPE environment variable

**Preconditions:** Server created
**Postconditions:** If MCP_TRANSPORT_TYPE set, only those transports registered (comma-separated); if unset, all three (sse, http, streamable-http) registered; unknown transport names logged as warning
**Evidence:** `factory.ts:122-142`
**Confidence:** HIGH (from code)

### BC-9.01.009: Tool registration with MCP SDK uses schema shape and annotations

**Preconditions:** `coreMcpServer.initialize(toolRegistry)` called
**Postconditions:** For each tool handler, calls `mcpServer.registerTool(name, {description, inputSchema: handler.inputSchema.shape, annotations: {title, parameters}}, handler.handle.bind(handler))`
**Evidence:** `mcp-server-instance.ts:73-84`
**Confidence:** HIGH (from code)

### BC-9.01.010: Tools with non-ZodObject schemas are skipped with warning

**Preconditions:** Tool handler has inputSchema that is not instanceof z.ZodObject
**Postconditions:** Tool is not registered; warning logged
**Evidence:** `mcp-server-instance.ts:56-60`
**Confidence:** MEDIUM (from code, no test for this edge case)

---

## Contract Coverage Summary

| Subsystem | Contract Count | HIGH Confidence | MEDIUM Confidence | LOW Confidence |
|-----------|---------------|-----------------|-------------------|----------------|
| 1. Integration | 14 | 12 | 2 | 0 |
| 2. Domain Services | 12 | 12 | 0 | 0 |
| 3. Tool Handlers | 10 | 10 | 0 | 0 |
| 4. Schema Validation | 10 | 10 | 0 | 0 |
| 5. Session Management | 9 | 9 | 0 | 0 |
| 6. Cache Infrastructure | 9 | 9 | 0 | 0 |
| 7. Error Hierarchy | 5 | 5 | 0 | 0 |
| 8. Transport (Registry) | 4 | 4 | 0 | 0 |
| 9. Server Lifecycle | 10 | 7 | 3 | 0 |
| **TOTAL** | **83** | **78** | **5** | **0** |

---

## Gaps: Behaviors Without Test Coverage

1. **group_by field replacement** (BC-1.04.002): The XDomeApiClient's logic to replace `fields` with `group_by` values has no direct unit test on the API client layer. Only tested indirectly via handler test.
2. **Double-start prevention** (BC-9.01.001): No test verifies that `start()` is a no-op when already running.
3. **Non-ZodObject schema skip** (BC-9.01.010): No test for edge case where a tool has a non-ZodObject schema.
4. **Transport protocol behavior**: HTTP, SSE, and Streamable HTTP transport tests exist but were not fully analyzed in this round -- see convergence declaration.
5. **ConnectionManager.removeConnection**: Calls `connection.close()` then deletes -- no test found for the close behavior in the connection manager test.
6. **Retry backoff timing**: No test verifies the linear backoff delay values (1s, 2s, 3s).

---

## Delta Summary
- New items added: 83 behavioral contracts (vs 13 in broad sweep); systematic subsystem numbering; contract coverage matrix; 6 gap identifications
- Existing items refined: BC-001 through BC-013 decomposed into granular contracts with precise test evidence citations
- Remaining gaps: Transport layer protocols (HTTP/SSE/Streamable HTTP connection tests), ConnectionManager close behavior, e2e test analysis for get-alerts/get-devices/get-vulnerability-devices integration tests

## Novelty Assessment
Novelty: SUBSTANTIVE
The broad sweep had 13 contracts. This round extracts 83 with precise test evidence, subsystem classification, confidence levels, and gap analysis. The schema validation contracts (default values, strictness differences, filter operation divergence), the cache infrastructure contracts (lazy eviction, zero-TTL behavior), and the domain service caching key composition patterns all change how you would spec this system.

## Convergence Declaration
Another round needed -- the following gaps remain: (1) transport-layer protocol contracts from express-transport.test.ts, sse-transport.test.ts, streamable-http-connection.test.ts; (2) connection manager behavior contracts; (3) e2e integration test contracts; (4) main.test.ts and factory.test.ts contracts.

## State Checkpoint
```yaml
pass: 3
round: 1
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
```
