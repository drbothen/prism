# Pass 3 Deep: Behavioral Contracts -- mcp-claroty-xdome (Round 2)

## Overview

This round closes the gaps from Round 1: transport-layer protocol contracts, connection manager behavior, e2e integration test contracts, and main/factory lifecycle contracts. All test files have now been read and analyzed.

---

## Subsystem 8 (Continued): Transport Layer

### BC-8.02.001: ReusableExpressTransport processes valid JSON-RPC requests via onmessage

**Preconditions:** POST request with JSON-RPC body and `mcp-session-id` header
**Postconditions:** SessionOrchestrator resolves session; `onmessage` callback invoked with request body; `Mcp-Session-Id` header set on response
**Test Evidence:** `express-transport.test.ts:56-81`
**Confidence:** HIGH (from tests)

### BC-8.02.002: ReusableExpressTransport returns 500 with -32603 on processing errors

**Preconditions:** SessionOrchestrator throws during middleware processing
**Postconditions:** Response status 500, JSON-RPC error with code -32603 (InternalError)
**Test Evidence:** `express-transport.test.ts:83-99`
**Confidence:** HIGH (from tests)

### BC-8.02.003: ReusableExpressTransport send() routes response by rpcId to session connection

**Preconditions:** `send(responseMessage)` called where response.id maps to a known sessionId
**Postconditions:** Finds connection via ConnectionManager.getConnection(sessionId); calls connection.send(response); removes connection via removeConnection; cleans up rpcId mapping
**Test Evidence:** `express-transport.test.ts:102-130`
**Confidence:** HIGH (from tests)

### BC-8.02.004: ReusableExpressTransport send() logs error if session not found for response

**Preconditions:** `send(responseMessage)` where response.id has no rpcId-to-sessionId mapping
**Postconditions:** Logs error "Could not find session for response" with rpcId
**Test Evidence:** `express-transport.test.ts:132-146`
**Confidence:** HIGH (from tests)

### BC-8.02.005: ReusableExpressTransport close() closes all active connections

**Preconditions:** `close()` called
**Postconditions:** Gets all connections from ConnectionManager; calls close() on each
**Test Evidence:** `express-transport.test.ts:148-167`
**Confidence:** HIGH (from tests)

### BC-8.03.001: SseTransport registers GET /sse and POST /sse/message routes

**Preconditions:** Transport instantiated
**Postconditions:** `getRegistrationDetails()` returns `[{path: "/sse", method: "GET"}, {path: "/sse/message", method: "POST"}]`
**Test Evidence:** `sse-transport.test.ts:38-44`
**Confidence:** HIGH (from tests)

### BC-8.03.002: SseTransport GET /sse establishes SSE connection with correct headers

**Preconditions:** GET request to /sse
**Postconditions:** Response headers set: `Content-Type: text/event-stream`, `Cache-Control: no-cache`, `Connection: keep-alive`; status 200
**Test Evidence:** `sse-transport.test.ts:47-70`
**Confidence:** HIGH (from tests)

### BC-8.03.003: SseTransport POST /sse/message processes messages for valid sessions

**Preconditions:** POST to /sse/message with sessionId query parameter and JSON-RPC body
**Postconditions:** `onmessage` invoked with body; response status 202 (Accepted)
**Test Evidence:** `sse-transport.test.ts:73-88`
**Confidence:** HIGH (from tests)

### BC-8.03.004: SseTransport POST /sse/message rejects requests without sessionId

**Preconditions:** POST to /sse/message without sessionId query parameter
**Postconditions:** Response status 400, JSON body `{error: "sessionId query parameter is required."}`
**Test Evidence:** `sse-transport.test.ts:90-105`
**Confidence:** HIGH (from tests)

### BC-8.03.005: SseTransport send() routes response to correct session connection

**Preconditions:** `send(response)` after a request mapped the rpcId to a sessionId
**Postconditions:** Connection retrieved via ConnectionManager; connection.send(response) called
**Test Evidence:** `sse-transport.test.ts:107-134`
**Confidence:** HIGH (from tests)

### BC-8.04.001: StreamableHttpConnection sends first message as HTTP JSON response

**Preconditions:** First `send()` call on a new StreamableHttpConnection
**Postconditions:** Calls `initialResponse.status(200).json(message)`; SSE stream NOT used
**Test Evidence:** `streamable-http-connection.test.ts:44-57`
**Confidence:** HIGH (from tests)

### BC-8.04.002: StreamableHttpConnection uses initial HTTP response exactly once

**Preconditions:** Two `send()` calls, with `setSseResponse()` between them
**Postconditions:** First uses HTTP response; second uses SSE stream; HTTP response json() called exactly once
**Test Evidence:** `streamable-http-connection.test.ts:59-86`
**Confidence:** HIGH (from tests)

### BC-8.04.003: StreamableHttpConnection setSseResponse() configures SSE headers

**Preconditions:** `setSseResponse(res)` called
**Postconditions:** Sets headers: Content-Type: text/event-stream, Cache-Control: no-cache, Connection: keep-alive; flushes headers; registers 'close' event listener
**Test Evidence:** `streamable-http-connection.test.ts:88-108`
**Confidence:** HIGH (from tests)

### BC-8.04.004: StreamableHttpConnection sends SSE messages in event-stream format

**Preconditions:** SSE mode active, `send(message)` called
**Postconditions:** Writes `"event: message\ndata: ${JSON.stringify(message)}\n\n"` to response
**Test Evidence:** `streamable-http-connection.test.ts:110-133`
**Confidence:** HIGH (from tests)

### BC-8.04.005: StreamableHttpConnection triggers onDisconnect on client close

**Preconditions:** Client closes SSE connection ('close' event fires)
**Postconditions:** `onDisconnect` callback invoked
**Test Evidence:** `streamable-http-connection.test.ts:135-149`
**Confidence:** HIGH (from tests)

### BC-8.04.006: StreamableHttpConnection close() ends SSE stream and triggers onDisconnect

**Preconditions:** `close()` called with active SSE stream
**Postconditions:** `sseResponse.end()` called; `onDisconnect` called
**Test Evidence:** `streamable-http-connection.test.ts:151-157`
**Confidence:** HIGH (from tests)

### BC-8.04.007: StreamableHttpConnection close() without SSE stream triggers onDisconnect only

**Preconditions:** `close()` called before `setSseResponse()`
**Postconditions:** `sseResponse.end()` NOT called; `onDisconnect` IS called
**Test Evidence:** `streamable-http-connection.test.ts:159-164`
**Confidence:** HIGH (from tests)

### BC-8.05.001: ConnectionManager stores and retrieves connections by sessionId

**Preconditions:** `addConnection(sessionId, connection)` called
**Postconditions:** `getConnection(sessionId)` returns the same connection object
**Test Evidence:** `connection-manager.test.ts:18-25`
**Confidence:** HIGH (from tests)

### BC-8.05.002: ConnectionManager returns undefined for unknown sessionId

**Preconditions:** `getConnection("non-existent-id")`
**Postconditions:** Returns undefined
**Test Evidence:** `connection-manager.test.ts:27-30`
**Confidence:** HIGH (from tests)

### BC-8.05.003: ConnectionManager removeConnection closes and deletes connection

**Preconditions:** `removeConnection(sessionId)` for existing connection
**Postconditions:** `connection.close()` called exactly once; subsequent `getConnection` returns undefined
**Test Evidence:** `connection-manager.test.ts:32-39`, `42-49`
**Confidence:** HIGH (from tests)

### BC-8.05.004: ConnectionManager removeConnection is no-op for unknown sessionId

**Preconditions:** `removeConnection("non-existent-id")`
**Postconditions:** Does not throw
**Test Evidence:** `connection-manager.test.ts:51-55`
**Confidence:** HIGH (from tests)

### BC-8.05.005: ConnectionManager getAllConnections returns all active connections

**Preconditions:** Multiple connections added
**Postconditions:** Returns Map with all session-connection pairs
**Test Evidence:** `connection-manager.test.ts:57-67`
**Confidence:** HIGH (from tests)

### BC-8.06.001: TransportManager registers POST routes with JSON body parser middleware

**Preconditions:** Transport with POST route registered
**Postconditions:** `appRouter.post(path, jsonBodyParser, transport.middleware)` called
**Test Evidence:** `transport-manager.test.ts:47-62`
**Confidence:** HIGH (from tests)

### BC-8.06.002: TransportManager registers GET routes without body parser

**Preconditions:** Transport with GET route registered
**Postconditions:** `appRouter.get(path, transport.middleware)` called (no body parser)
**Test Evidence:** `transport-manager.test.ts:64-82`
**Confidence:** HIGH (from tests)

### BC-8.06.003: TransportManager startAll() calls start on all registered transports

**Preconditions:** Transports registered
**Postconditions:** Each transport's `start()` called
**Test Evidence:** `transport-manager.test.ts:84-93`
**Confidence:** HIGH (from tests)

### BC-8.06.004: TransportManager startAll() does not throw on transport start failure

**Preconditions:** A transport's start() rejects
**Postconditions:** No exception propagates; other transports unaffected
**Test Evidence:** `transport-manager.test.ts:106-115`
**Confidence:** HIGH (from tests)

### BC-8.06.005: TransportManager closeAll() does not throw on transport close failure

**Preconditions:** A transport's close() rejects
**Postconditions:** No exception propagates
**Test Evidence:** `transport-manager.test.ts:117-126`
**Confidence:** HIGH (from tests)

---

## Subsystem 9 (Continued): Server Lifecycle

### BC-9.02.001: CoreMcpServer skips registration of non-ZodObject schema tools

**Preconditions:** Tool with `z.string()` inputSchema (not `z.object()`)
**Postconditions:** Warning logged "Tool does not have a ZodObject schema. Skipping."; `mcpServer.registerTool` NOT called
**Test Evidence:** `mcp-server-instance.test.ts:81-91`
**Confidence:** HIGH (from tests -- this was MEDIUM in Round 1, now upgraded)

### BC-9.02.002: CoreMcpServer warns on double-start

**Preconditions:** `start(port)` called when server is already running
**Postconditions:** Logs warning "Server is already running"; returns immediately
**Test Evidence:** `mcp-server-instance.test.ts:109-123`
**Confidence:** HIGH (from tests -- was MEDIUM in Round 1, now upgraded)

### BC-9.02.003: CoreMcpServer stop rejects if HTTP server close fails

**Preconditions:** `stop()` called, `httpServer.close()` callback receives error
**Postconditions:** Rejects with the HTTP close error; logs error message
**Test Evidence:** `mcp-server-instance.test.ts:127-139`
**Confidence:** HIGH (from tests)

### BC-9.02.004: CoreMcpServer stop rejects if MCP server close fails

**Preconditions:** `stop()` called, `mcpServer.close()` rejects
**Postconditions:** Rejects with the MCP close error
**Test Evidence:** `mcp-server-instance.test.ts:141-156`
**Confidence:** HIGH (from tests)

### BC-9.02.005: CoreMcpServer stop calls transportManager.closeAll

**Preconditions:** `stop()` called
**Postconditions:** `transportManager.closeAll()` called exactly once
**Test Evidence:** `mcp-server-instance.test.ts:158-161`
**Confidence:** HIGH (from tests)

### BC-9.02.006: CoreMcpServer stop resolves gracefully when not running

**Preconditions:** `stop()` called on server that was never started
**Postconditions:** Resolves to undefined; `mcpServer.close()` still called
**Test Evidence:** `mcp-server-instance.test.ts:163-167`
**Confidence:** HIGH (from tests)

### BC-9.03.001: main() starts server on default port 3000

**Preconditions:** PORT env var not set
**Postconditions:** `server.start(3000)` called; logs "Server listening" with port 3000
**Test Evidence:** `main.test.ts:57-65`
**Confidence:** HIGH (from tests)

### BC-9.03.002: main() uses PORT environment variable

**Preconditions:** `PORT=8080` in environment
**Postconditions:** `server.start(8080)` called
**Test Evidence:** `main.test.ts:67-77`
**Confidence:** HIGH (from tests)

### BC-9.03.003: main() calls handleFatalError on creation failure

**Preconditions:** `createAndInitializeServer()` throws
**Postconditions:** `handleFatalError(error)` called
**Test Evidence:** `main.test.ts:79-87`
**Confidence:** HIGH (from tests)

### BC-9.03.004: main() calls handleFatalError on start failure

**Preconditions:** `server.start()` throws
**Postconditions:** `handleFatalError(error)` called
**Test Evidence:** `main.test.ts:89-96`
**Confidence:** HIGH (from tests)

### BC-9.04.001: Factory registers all 3 transports by default

**Preconditions:** `MCP_TRANSPORT_TYPE` env var not set
**Postconditions:** TransportManager.register called 3 times (SSE, HTTP, Streamable HTTP)
**Test Evidence:** `factory.test.ts:62-75`
**Confidence:** HIGH (from tests)

### BC-9.04.002: Factory respects MCP_TRANSPORT_TYPE=http for single transport

**Preconditions:** `MCP_TRANSPORT_TYPE=http`
**Postconditions:** TransportManager.register called exactly once
**Test Evidence:** `factory.test.ts:77-81`
**Confidence:** HIGH (from tests)

### BC-9.04.003: Factory does not throw for unknown transport type

**Preconditions:** `MCP_TRANSPORT_TYPE=invalid-transport`
**Postconditions:** No exception; register NOT called (no valid transports)
**Test Evidence:** `factory.test.ts:83-91`
**Confidence:** HIGH (from tests)

---

## Subsystem 10: Health Endpoint (Dedicated Subsystem)

### BC-10.01.001: Health endpoint returns 200 with complete structure

**Preconditions:** GET /health
**Postconditions:** Status 200; JSON body contains: status, timestamp, version, uptime, memory, environment, dependencies
**Test Evidence:** `health-endpoint.test.ts:54-57`, `155-172`
**Confidence:** HIGH (from tests)

### BC-10.01.002: Health endpoint version matches semver pattern

**Preconditions:** GET /health
**Postconditions:** `version` matches `/^\d+\.\d+\.\d+/`
**Test Evidence:** `health-endpoint.test.ts:69-76`
**Confidence:** HIGH (from tests)

### BC-10.01.003: Health endpoint uptime.human matches "Xh Xm Xs" format

**Preconditions:** GET /health
**Postconditions:** `uptime.human` matches `/^\d+h \d+m \d+s$/`
**Test Evidence:** `health-endpoint.test.ts:78-89`
**Confidence:** HIGH (from tests)

### BC-10.01.004: Health endpoint memory.unit is always "MB"

**Preconditions:** GET /health
**Postconditions:** `memory.unit === "MB"`, `memory.used > 0`, `memory.total > 0`
**Test Evidence:** `health-endpoint.test.ts:91-110`
**Confidence:** HIGH (from tests)

### BC-10.01.005: Health endpoint reports Claroty API as "configured" when env var set

**Preconditions:** `CLAROTY_XDOME_BASE_URL` is set
**Postconditions:** `dependencies.claroty_api === "configured"`
**Test Evidence:** `health-endpoint.test.ts:128-136`
**Confidence:** HIGH (from tests)

### BC-10.01.006: Health endpoint reports Claroty API as "not_configured" when env var missing

**Preconditions:** `CLAROTY_XDOME_BASE_URL` is not set
**Postconditions:** `dependencies.claroty_api === "not_configured"`
**Test Evidence:** `health-endpoint.test.ts:138-146`
**Confidence:** HIGH (from tests)

### BC-10.01.007: Health endpoint returns application/json content type

**Preconditions:** GET /health
**Postconditions:** Content-Type header matches `application/json`; body is valid JSON
**Test Evidence:** `health-endpoint.test.ts:148-153`
**Confidence:** HIGH (from tests)

---

## Updated Contract Coverage Summary

| Subsystem | R1 Count | R2 New | Total | HIGH | MEDIUM |
|-----------|----------|--------|-------|------|--------|
| 1. Integration | 14 | 0 | 14 | 12 | 2 |
| 2. Domain Services | 12 | 0 | 12 | 12 | 0 |
| 3. Tool Handlers | 10 | 0 | 10 | 10 | 0 |
| 4. Schema Validation | 10 | 0 | 10 | 10 | 0 |
| 5. Session Management | 9 | 0 | 9 | 9 | 0 |
| 6. Cache Infrastructure | 9 | 0 | 9 | 9 | 0 |
| 7. Error Hierarchy | 5 | 0 | 5 | 5 | 0 |
| 8. Transport Layer | 4 | 22 | 26 | 26 | 0 |
| 9. Server Lifecycle | 10 | 10 | 20 | 20 | 0 |
| 10. Health Endpoint | 0 | 7 | 7 | 7 | 0 |
| **TOTAL** | **83** | **39** | **122** | **120** | **2** |

### Confidence Upgrades from Round 1:
- BC-9.01.001 (double-start) upgraded from MEDIUM to HIGH (test found in `mcp-server-instance.test.ts:109-123`)
- BC-9.01.010 (non-ZodObject skip) upgraded from MEDIUM to HIGH (test found in `mcp-server-instance.test.ts:81-91`)

### Remaining MEDIUM Confidence Contracts:
1. **BC-1.01.003** (15s timeout): Configuration constant, no specific timeout test
2. **BC-1.04.002** (group_by field replacement): No direct API client unit test for field overwrite behavior

---

## Updated Gaps Analysis

**Closed gaps from Round 1:**
1. Transport protocol behavior -- CLOSED: 22 new contracts from express-transport, sse-transport, streamable-http-connection, connection-manager, transport-manager tests
2. ConnectionManager close behavior -- CLOSED: BC-8.05.003
3. Main/factory lifecycle -- CLOSED: 10 new contracts from main.test.ts and factory.test.ts
4. Double-start prevention -- CLOSED: BC-9.02.002 (test found)
5. Non-ZodObject schema skip -- CLOSED: BC-9.02.001 (test found)

**Remaining gaps (not closable from test evidence):**
1. **Retry backoff timing verification**: No test verifies the 1s/2s/3s delays -- only code inspection
2. **group_by field replacement at API client layer**: Only handler-level test, not client-level test
3. **include_count constraint when offset > 0**: No test or validation for this documented xDome constraint
4. **Session expiration/cleanup**: No mechanism exists -- sessions live forever (identified as missing NFR)
5. **Cache size bounds**: No maximum -- cache can grow unbounded (identified as missing NFR)

---

## Delta Summary
- New items added: 39 behavioral contracts across transport layer (22), server lifecycle (10), and health endpoint (7 -- new subsystem 10)
- Existing items refined: 2 contracts upgraded from MEDIUM to HIGH confidence
- Remaining gaps: 5 gaps remain, none closable from existing test files (they represent missing tests or missing functionality)

## Novelty Assessment
Novelty: NITPICK
The 39 new contracts are all in the transport/infrastructure layer. They document how HTTP, SSE, and Streamable HTTP transports work, and how the server lifecycle operates. These are important for completeness but do not change how you would spec the core domain behavior (querying alerts, devices, vulnerabilities). The domain-level contracts (subsystems 1-7) are unchanged. The transport contracts are infrastructure plumbing that would be reimplemented differently in a Prism context anyway.

## Convergence Declaration
Pass 3 has converged -- findings are infrastructure-layer completionism, not domain-behavioral gaps. The 122 total contracts cover all test files and all significant code paths. The 5 remaining gaps are either missing tests or missing functionality (NFRs), not undiscovered behavior.

## State Checkpoint
```yaml
pass: 3
round: 2
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
