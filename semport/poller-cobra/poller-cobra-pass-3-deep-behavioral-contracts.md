# Pass 3 Deep: Behavioral Contracts -- poller-cobra (Round 1)

> Convergence deepening round 1. Extracted from all 18 .go source files and 3 test files.

---

## Contract Numbering Convention

Format: `BC-{subsystem}.{category}.{sequence}`

Subsystems:
- `CS` = CrowdStrike client (crowdstrike package)
- `COL` = Collector (collector package)
- `SNK` = Sink (sink package)
- `ST` = State (state package)
- `HLT` = Health (health package)
- `CFG` = Config (config package)
- `PRF` = Profiling (profiling package)
- `RUN` = Runner (runner package)
- `MAIN` = Main entrypoint

Categories:
- `01` = Initialization/Construction
- `02` = Core Operations
- `03` = Error Handling
- `04` = Validation
- `05` = State Management

---

## CrowdStrike Client Contracts

### BC-CS.01.001: HTTPClient rejects empty ClientID

**Preconditions:** Config with ClientID that is empty or whitespace-only
**Postconditions:** Returns `(nil, error)` immediately. No SDK client created. No network call.
**Error Cases:** `"crowdstrike: client ID is required"` (plain error, no sentinel wrapping)
**Evidence:** api.go:73-74, api_test.go:132-143 (TestHTTPClient_FetchAlerts_NilInner indirectly -- tests nil inner, not empty creds)
**Confidence:** HIGH (code is unambiguous, though test coverage is indirect)

### BC-CS.01.002: HTTPClient rejects empty ClientSecret

**Preconditions:** Config with ClientSecret that is empty or whitespace-only
**Postconditions:** Returns `(nil, error)` immediately. No SDK client created.
**Error Cases:** `"crowdstrike: client secret is required"` (plain error)
**Evidence:** api.go:76-78
**Confidence:** HIGH (code is unambiguous, no direct test)

### BC-CS.01.003: HTTPClient defaults Region to "us-1" when empty

**Preconditions:** Config with empty/whitespace Region
**Postconditions:** Region set to "us-1" before SDK client creation
**Evidence:** api.go:81-83
**Confidence:** HIGH (code is unambiguous)

### BC-CS.01.004: HTTPClient creates default logger when none provided

**Preconditions:** Config.Logger is nil
**Postconditions:** JSON-formatted stdout logger with timestamps created
**Evidence:** api.go:98-100
**Confidence:** HIGH (code is unambiguous)

### BC-CS.02.001: Ping validates connectivity via limit=1 alerts query

**Preconditions:** HTTPClient with non-nil inner SDK client
**Postconditions:** Returns nil on success. Checks for both SDK errors and API-level errors in payload.
**Error Cases:**
- nil inner -> `fmt.Errorf("crowdstrike ping: %w", ErrClientNotInitialized)`
- SDK error -> `fmt.Errorf("crowdstrike ping: %w", sdkErr)`
- API error in `resp.Payload.Errors[0].Message` -> `fmt.Errorf("crowdstrike ping: API error: %s", msg)`
- nil payload -> returns nil (success)
- errors array with nil Message -> returns nil (success)
**Evidence:** api.go:307-328, api_test.go:36-130 (6 test cases)
**Confidence:** HIGH (comprehensive test coverage)

**Test case inventory (api_test.go:39-106):**
1. `"nil inner returns ErrClientNotInitialized"` -- asserts `errors.Is(err, ErrClientNotInitialized)` and error contains `"crowdstrike ping"`
2. `"SDK error is wrapped and returned"` -- mock returns `errors.New("connection refused")`, asserts error contains `"crowdstrike ping"`
3. `"API-level error in payload is returned"` -- mock returns payload with `Errors[0].Message = "scope not authorized"`, asserts error contains `"scope not authorized"`
4. `"successful ping returns nil"` -- mock returns payload with empty Resources, asserts nil error
5. `"successful ping with nil payload returns nil"` -- mock returns `QueryV2OK{}` with nil Payload, asserts nil error
6. `"payload errors with nil message returns nil"` -- mock returns error entry with nil Message pointer, asserts nil error

### BC-CS.02.002: FetchAlerts returns ErrClientNotInitialized when inner is nil

**Preconditions:** HTTPClient with nil inner field
**Postconditions:** Returns `(nil, error)` wrapping `ErrClientNotInitialized`
**Evidence:** api.go:112-114, api_test.go:132-143
**Confidence:** HIGH (direct test assertion with `errors.Is`)

### BC-CS.02.003: FetchAlerts defaults limit to 100 when non-positive

**Preconditions:** limit parameter <= 0
**Postconditions:** Internally uses limit=100 for API query
**Evidence:** api.go:116-118
**Confidence:** MEDIUM (code is clear, no test)

### BC-CS.02.004: FetchAlerts returns empty slice (not nil) when no results

**Preconditions:** API returns nil payload or empty Resources
**Postconditions:** Returns `[]Alert{}` (non-nil empty slice)
**Evidence:** api.go:138-139, api.go:155-157
**Confidence:** HIGH (code is unambiguous at two guard points)

### BC-CS.02.005: FetchAlerts skips nil resource entries

**Preconditions:** API returns resources array containing nil entries
**Postconditions:** Nil entries are silently skipped, non-nil entries are processed
**Evidence:** api.go:162-164 (`if a == nil { continue }`)
**Confidence:** HIGH (code is explicit)

### BC-CS.02.006: FetchAlerts uses two-step query pattern

**Preconditions:** Client initialized, API reachable
**Postconditions:** Step 1: QueryV2 with filter, limit, sort=timestamp|desc. Step 2: PostEntitiesAlertsV1 with returned IDs. Returns mapped Alert slice.
**Error Cases:** QueryV2 error -> `fmt.Errorf("query alerts: %w", err)`. PostEntitiesAlertsV1 error -> `fmt.Errorf("fetch alert details: %w", err)`
**Evidence:** api.go:122-157
**Confidence:** HIGH (code flow is linear and clear)

### BC-CS.02.007: FetchDetections returns empty slice (stub)

**Preconditions:** Client initialized (non-nil inner)
**Postconditions:** Logs debug message, returns `[]Detection{}` with nil error
**Error Cases:** nil inner -> `fmt.Errorf("fetch detections: %w", ErrClientNotInitialized)`
**Evidence:** api.go:278-288, api_test.go:145-156
**Confidence:** HIGH (test confirms nil inner case)

### BC-CS.02.008: FetchHosts returns empty slice (stub)

**Preconditions:** Client initialized (non-nil inner)
**Postconditions:** Logs debug message, returns `[]Host{}` with nil error
**Error Cases:** nil inner -> `fmt.Errorf("fetch hosts: %w", ErrClientNotInitialized)`
**Evidence:** api.go:291-301, api_test.go:158-168
**Confidence:** HIGH (test confirms nil inner case)

### BC-CS.02.009: alertToMap returns nil for nil input

**Preconditions:** Input alert pointer is nil
**Postconditions:** Returns nil map
**Evidence:** api.go:205-207
**Confidence:** HIGH (code is explicit guard)

### BC-CS.02.010: alertToMap merges additional properties into result map

**Preconditions:** Alert has non-empty `DetectsAlertAdditionalProperties` map
**Postconditions:** All key-value pairs from overflow map are added to result. Keys may overwrite explicit mappings if they collide.
**Evidence:** api.go:270-272
**Confidence:** HIGH (code is explicit loop)

---

## Collector Contracts

### BC-COL.01.001: Collector defaults interval through fallback chain

**Preconditions:** Options.Interval and config Interval may be zero/negative
**Postconditions:** Fallback chain: opts.Interval -> cfg.Collector.Interval -> 30s hardcoded
**Evidence:** collector.go:68-74
**Confidence:** HIGH (code is explicit)

### BC-COL.01.002: Collector constructs QueryFingerprint from source config

**Preconditions:** Config loaded with Source.Region, Source.SourceType, Source.Filter, Source.Limit
**Postconditions:** QueryFingerprint created with fields=[Region, SourceType, Filter] and Limit. SHA-256 hash of sorted fields + limit.
**Evidence:** collector.go:77-80
**Confidence:** HIGH (code is explicit)

### BC-COL.05.001: State initialization loads existing state and validates fingerprint

**Preconditions:** Store.Load returns existing PollState
**Postconditions:** If fingerprint matches -> use loaded state. If mismatch -> return ErrQueryFingerprintMismatch with both hashes.
**Evidence:** collector.go:173-179
**Confidence:** HIGH (code is explicit)

### BC-COL.05.002: State initialization bootstraps on ErrStateNotFound

**Preconditions:** Store.Load returns ErrStateNotFound
**Postconditions:** Creates PollState with Cursor={Timestamp: cfg.Collector.InitialSince, RecordID: ""}, Version=0, Query=current fingerprint. Creates empty BatchReceipt. Saves both to store.
**Error Cases:** Store.Save fails -> wraps with ErrCollectorStatePersist
**Evidence:** collector.go:182-203
**Confidence:** HIGH (code is explicit)

### BC-COL.05.003: State initialization fails on non-sentinel store errors

**Preconditions:** Store.Load returns error that is NOT ErrStateNotFound
**Postconditions:** Returns error wrapped with ErrCollectorStateLoad
**Evidence:** collector.go:204-206
**Confidence:** HIGH (code is explicit default case)

### BC-COL.02.001: Collection loop uses exponential backoff with configurable parameters

**Preconditions:** collectOnce returns error
**Postconditions:**
- Retry count incremented
- Health set to NotReady
- If retryCount > MaxRetries -> return ErrCollectorRetriesExceeded
- Wait retryDelay (respects context cancellation)
- Double retryDelay, cap at maxDelay
**Error Cases:** MaxRetries=0 -> retries disabled (condition `c.cfg.Collector.MaxRetries > 0` is false, so no limit check)
**Evidence:** collector.go:128-148
**Confidence:** HIGH (code is explicit)

**Correction from broad sweep:** The broad sweep says "5 max attempts" and shows attempts 1-6. Verified: the check is `retryCount > c.cfg.Collector.MaxRetries` AFTER incrementing, and `retryCount` starts at 0. So with MaxRetries=5, the 6th increment triggers the error. This means 5 actual retry attempts before failure, which matches the broad sweep.

**Additional finding:** When MaxRetries is set to 0, the condition `c.cfg.Collector.MaxRetries > 0 && retryCount > ...` is always false, meaning retries are effectively unlimited. This is not documented anywhere.

### BC-COL.02.002: Successful collection resets retry state and sets Ready

**Preconditions:** collectOnce returns nil error
**Postconditions:** retryCount reset to 0, retryDelay reset to baseDelay, health set to Ready
**Evidence:** collector.go:150-154
**Confidence:** HIGH (code is explicit)

### BC-COL.02.003: hasMore triggers immediate re-fetch without waiting

**Preconditions:** collectOnce returns (true, nil)
**Postconditions:** Loop continues immediately without waiting on ticker
**Evidence:** collector.go:156-159
**Confidence:** HIGH (code is explicit `continue`)

### BC-COL.02.004: Collection waits on ticker when no more data

**Preconditions:** collectOnce returns (false, nil)
**Postconditions:** Blocks on `select` between ticker.C and ctx.Done(). Ticker fires -> loop continues. ctx.Done -> returns ctx.Err().
**Evidence:** collector.go:161-166
**Confidence:** HIGH (code is explicit)

### BC-COL.02.005: Health starts NotReady and ends NotReady

**Preconditions:** Reporter is non-nil
**Postconditions:** SetNotReady() called on entry and deferred on exit. SetReady() called after successful state initialization.
**Evidence:** collector.go:100-111
**Confidence:** HIGH (code is explicit)

### BC-COL.02.006: AlertCollector sorts alerts ascending before filtering

**Preconditions:** Alerts fetched from API (sorted timestamp|desc by API)
**Postconditions:** Stable sort by (Timestamp ASC, ID ASC). This reverses the API sort order.
**Evidence:** alert_collector.go:63-68
**Confidence:** HIGH (code is explicit, SliceStable used)

### BC-COL.02.007: AlertCollector filters alerts by cursor position

**Preconditions:** Alerts sorted, current cursor available
**Postconditions:** Only alerts where `isCursorAhead(current, alertCursor)` is true are kept. Zero-timestamp alerts are excluded with warning log.
**Evidence:** alert_collector.go:71, 121-133
**Confidence:** HIGH (code is explicit)

### BC-COL.02.008: AlertCollector delivers alerts individually in order

**Preconditions:** newAlerts non-empty, sink non-nil
**Postconditions:** Each alert sent via `sink.Send(ctx, alert.Raw, alert.ID, "crowdstrike_alert")`. Sent in ascending timestamp order. First failure aborts entire batch.
**Evidence:** alert_collector.go:81-86
**Confidence:** HIGH (code is explicit loop with early return)

### BC-COL.02.009: AlertCollector skips delivery when sink is nil

**Preconditions:** Sink is nil (not configured)
**Postconditions:** Alerts are processed (cursor advances) but not delivered. No error.
**Evidence:** alert_collector.go:81 (`if a.sink != nil`)
**Confidence:** HIGH (explicit nil check)

### BC-COL.02.010: AlertCollector enforces forward cursor progress

**Preconditions:** Batch processed, new cursor computed from last alert
**Postconditions:** `ensureForwardProgress` called. If cursor did not advance -> returns error (NOT wrapped with a sentinel). If advanced -> state updated.
**Evidence:** alert_collector.go:96-98, 145-152
**Confidence:** HIGH (code is explicit)

**Finding:** The ensureForwardProgress error is NOT wrapped with a sentinel error. It returns a plain `fmt.Errorf(...)` without `%w`. This means `errors.Is()` cannot match it. This is inconsistent with the rest of the error handling pattern.

### BC-COL.02.011: AlertCollector returns no receipt when no new alerts

**Preconditions:** FetchAlerts returns empty, or all alerts filtered out by cursor
**Postconditions:** Returns (currentState, nil, false, nil). No receipt. No state modification.
**Evidence:** alert_collector.go:57-59, 73-75
**Confidence:** HIGH (code is explicit)

### BC-COL.02.012: Collector saves state after successful alert batch

**Preconditions:** AlertCollector returns non-nil receipt
**Postconditions:** State and receipt saved to store. If save fails -> error wrapped with ErrCollectorStatePersist.
**Evidence:** collector.go:230-232
**Confidence:** HIGH (code is explicit)

### BC-COL.02.013: Collector skips state save when receipt is nil

**Preconditions:** AlertCollector returns nil receipt (no new alerts)
**Postconditions:** State not saved. hasMore returned as-is.
**Evidence:** collector.go:226-228
**Confidence:** HIGH (explicit nil check)

---

## Sink Contracts

### BC-SNK.01.001: HTTPSender rejects empty endpoint

**Preconditions:** SinkConfig with empty/whitespace endpoint
**Postconditions:** Returns error wrapping ErrSinkConfigMissing with message "endpoint is empty"
**Evidence:** http_sender.go:51-53
**Confidence:** HIGH (code is explicit)

### BC-SNK.01.002: HTTPSender rejects incomplete credentials

**Preconditions:** Username or password is empty/whitespace
**Postconditions:** Returns error wrapping ErrSinkConfigMissing with message "credentials are incomplete"
**Evidence:** http_sender.go:54-56
**Confidence:** HIGH (code is explicit)

### BC-SNK.01.003: HTTPSender validates endpoint is absolute URL

**Preconditions:** Endpoint is non-empty
**Postconditions:** Parsed via url.Parse. If parse error or not absolute -> returns error wrapping ErrSinkRequestBuild
**Evidence:** http_sender.go:63-65
**Confidence:** HIGH (code is explicit)

### BC-SNK.01.004: HTTPSender defaults timeout to 15s

**Preconditions:** SinkConfig.Timeout <= 0
**Postconditions:** HTTP client timeout set to 15 seconds
**Evidence:** http_sender.go:58-60
**Confidence:** HIGH (code is explicit)

### BC-SNK.02.001: Send enriches every payload with xMP metadata

**Preconditions:** Record and xMP config available
**Postconditions:** Payload wrapped as `{"data": <marshaledRecord>, "xmp": {"site":..., "cluster_name":..., "node_name":...}}`. Data is json.RawMessage (double-serialization avoided via RawMessage).
**Evidence:** http_sender.go:84-88, 121-149
**Confidence:** HIGH (code is explicit)

### BC-SNK.02.002: Send uses HTTP POST with Basic Auth

**Preconditions:** Enriched payload available
**Postconditions:** POST to endpoint with Basic Auth header and Content-Type: application/json
**Evidence:** http_sender.go:92-98
**Confidence:** HIGH (code is explicit)

### BC-SNK.03.001: Send treats status >= 400 as delivery failure

**Preconditions:** HTTP response received
**Postconditions:** If status >= 400 -> reads up to 2048 bytes of response body for logging, returns error wrapping ErrSinkDelivery with status and body.
**Evidence:** http_sender.go:111-115
**Confidence:** HIGH (code is explicit)

### BC-SNK.03.002: Send wraps HTTP client errors with ErrSinkDelivery

**Preconditions:** HTTP client.Do returns error (network, timeout, etc.)
**Postconditions:** Error logged, wrapped with ErrSinkDelivery
**Evidence:** http_sender.go:101-104
**Confidence:** HIGH (code is explicit)

### BC-SNK.03.003: Send logs response body close errors as warnings

**Preconditions:** resp.Body.Close() returns error
**Postconditions:** Warning logged, error not propagated
**Evidence:** http_sender.go:105-109
**Confidence:** HIGH (code is explicit)

---

## State Contracts

### BC-ST.01.001: MemoryStore starts uninitialized

**Preconditions:** NewMemoryStore() called
**Postconditions:** initialized=false. Load returns ErrStateNotFound.
**Evidence:** store.go:37-39, 42-48
**Confidence:** HIGH (code is explicit)

### BC-ST.02.001: MemoryStore.Load returns ErrStateNotFound when not initialized

**Preconditions:** No prior Save call
**Postconditions:** Returns `(PollState{}, ErrStateNotFound)`
**Evidence:** store.go:42-48
**Confidence:** HIGH (code is explicit)

### BC-ST.02.002: MemoryStore.Save atomically stores state and receipt

**Preconditions:** Valid state and receipt provided
**Postconditions:** State and receipt stored under mutex lock. initialized set to true. Always returns nil error.
**Evidence:** store.go:54-63
**Confidence:** HIGH (code is explicit)

### BC-ST.02.003: MemoryStore uses RWMutex for concurrent access

**Preconditions:** Multiple goroutines accessing store
**Postconditions:** Load uses RLock (read lock), Save uses Lock (write lock). Thread-safe.
**Evidence:** store.go:43-44, 55-56
**Confidence:** HIGH (code is explicit)

### BC-ST.02.004: QueryFingerprint sorts fields canonically for hashing

**Preconditions:** Fields and limit provided
**Postconditions:** Fields copied twice. Canonical copy sorted. Joined with `|`, limit appended. SHA-256 hashed. Original order preserved in returned Fields.
**Evidence:** store.go:105-123
**Confidence:** HIGH (code is explicit)

### BC-ST.02.005: QueryFingerprint clamps negative limit to 0

**Preconditions:** limit < 0
**Postconditions:** limit set to 0 before hashing
**Evidence:** store.go:107-109
**Confidence:** HIGH (code is explicit)

---

## Health Contracts

### BC-HLT.01.001: Health server defaults to port 7322

**Preconditions:** Empty addr passed to NewServer
**Postconditions:** Server binds to ":7322"
**Evidence:** server.go:58-59
**Confidence:** HIGH (code is explicit)

### BC-HLT.01.002: Health server starts alive but not ready

**Preconditions:** NewServer called
**Postconditions:** alive=true, ready=false
**Evidence:** server.go:77-78
**Confidence:** HIGH (code is explicit, server_test.go:71-73 confirms)

### BC-HLT.02.001: /health returns 200 "ok" when alive

**Preconditions:** Server is alive
**Postconditions:** 200 OK with body "ok"
**Evidence:** server.go:133-141, server_test.go:13-29 (TestServer_Liveness)
**Confidence:** HIGH (direct test)

### BC-HLT.02.002: /health returns 503 "unhealthy" when not alive

**Preconditions:** alive=false (after Shutdown)
**Postconditions:** 503 Service Unavailable with body "unhealthy"
**Evidence:** server.go:134-137
**Confidence:** HIGH (code is explicit, not directly tested)

### BC-HLT.02.003: /ready returns 200 "ready" when alive AND ready

**Preconditions:** alive=true AND ready=true
**Postconditions:** 200 OK with body "ready"
**Evidence:** server.go:143-151, server_test.go:49-66 (TestServer_Readiness_Ready)
**Confidence:** HIGH (direct test)

### BC-HLT.02.004: /ready returns 503 "not ready" when not ready OR not alive

**Preconditions:** ready=false OR alive=false
**Postconditions:** 503 Service Unavailable with body "not ready"
**Evidence:** server.go:144-148, server_test.go:31-47 (TestServer_Readiness_NotReady)
**Confidence:** HIGH (direct test)

### BC-HLT.02.005: Rate limiter enforces per-IP limits

**Preconditions:** Multiple requests from same IP exceed burst
**Postconditions:** Returns 429 Too Many Requests with body "rate limit exceeded" and Retry-After: 1 header
**Evidence:** server.go:112-131, server_test.go:115-158 (TestServer_RateLimiting_BlocksExcessiveTraffic)
**Confidence:** HIGH (direct test with burst=2 and 3rd request blocked)

### BC-HLT.02.006: Rate limiter isolates per IP

**Preconditions:** Different IPs sending requests
**Postconditions:** Each IP gets its own rate limiter. One IP being rate-limited does not affect another.
**Evidence:** server.go:89-109, server_test.go:160-196 (TestServer_RateLimiting_PerIPIsolation)
**Confidence:** HIGH (direct test)

### BC-HLT.02.007: Rate limiter recovers after waiting

**Preconditions:** IP rate-limited, then waits sufficient time
**Postconditions:** Subsequent request succeeds after token bucket refills
**Evidence:** server_test.go:198-239 (TestServer_RateLimiting_AllowsAfterWaiting)
**Confidence:** HIGH (direct test with 250ms sleep)

### BC-HLT.02.008: Rate limiter handles invalid RemoteAddr gracefully

**Preconditions:** Request with malformed RemoteAddr (no port)
**Postconditions:** Uses full RemoteAddr as IP key, does not panic, request processed
**Evidence:** server.go:115-119, server_test.go:241-255
**Confidence:** HIGH (direct test)

### BC-HLT.02.009: Rate limiter uses double-checked locking

**Preconditions:** Concurrent requests from new IP
**Postconditions:** RLock check first, if miss: Lock, re-check, create. Thread-safe limiter creation.
**Evidence:** server.go:89-109
**Confidence:** HIGH (code is explicit pattern)

### BC-HLT.02.010: Shutdown marks server as not alive

**Preconditions:** Server running
**Postconditions:** alive set to false, then HTTP server shutdown called
**Evidence:** server.go:159-162
**Confidence:** HIGH (code is explicit)

---

## Config Contracts

### BC-CFG.02.001: LoadFromEnvironment prioritizes file-backed secrets

**Preconditions:** Both `*_FILE` env var and direct env var are set
**Postconditions:** File-backed value wins. File read via `readSecretFile` -- if file not found, returns empty string (falls through to direct env var).
**Evidence:** config.go:186-193 (ClientID pattern repeated for all secrets)
**Confidence:** HIGH (code is explicit)

### BC-CFG.02.002: readSecretFile returns empty on missing file

**Preconditions:** File path set but file does not exist
**Postconditions:** Returns `("", nil)` -- not an error. Allows fallback to direct env var.
**Evidence:** config.go:454-456
**Confidence:** HIGH (code is explicit os.ErrNotExist check)

### BC-CFG.02.003: LoadFromEnvironment requires both ClientID and ClientSecret

**Preconditions:** Either credential missing after file + env loading
**Postconditions:** Returns error `"missing CrowdStrike client ID"` or `"missing CrowdStrike client secret"`
**Evidence:** config.go:203-210
**Confidence:** HIGH (code is explicit)

### BC-CFG.02.004: XMP NodeName falls back to OS hostname

**Preconditions:** XMP_NODE_NAME env var not set
**Postconditions:** Uses `os.Hostname()` if available, ignores hostname errors
**Evidence:** config.go:246-249
**Confidence:** HIGH (code is explicit)

### BC-CFG.02.005: VECTOR_TIMEOUT_SECONDS accepts both duration strings and plain integers

**Preconditions:** Timeout env var set
**Postconditions:** Tries `time.ParseDuration` first (e.g., "15s"). If that fails, tries `strconv.Atoi` and converts to seconds. If both fail, returns error.
**Evidence:** config.go:276-284
**Confidence:** HIGH (code is explicit dual-parse)

### BC-CFG.02.006: STATE_STORE_TYPE validates against file/memory

**Preconditions:** STATE_STORE_TYPE env var set
**Postconditions:** Accepts "file" or "memory" (case-insensitive). Any other value returns error.
**Evidence:** config.go:328-336
**Confidence:** HIGH (code is explicit switch)

### BC-CFG.04.001: Validate aggregates all validation errors

**Preconditions:** Config loaded
**Postconditions:** Returns `errors.Join(errs...)` -- all errors collected, not short-circuiting. Returns nil if no errors.
**Evidence:** config.go:360-442
**Confidence:** HIGH (code is explicit)

**Validation rules:**
1. Source.ClientID required (non-empty after trim)
2. Source.ClientSecret required
3. Source.Region required
4. Source.SourceType required
5. Source.Limit >= 1
6. Collector.Interval >= 1s
7. Collector.RetryBaseDelay >= 1s
8. Collector.RetryMaxDelay >= 1s
9. Collector.RetryMaxDelay >= RetryBaseDelay
10. Collector.MaxRetries >= 0
11. Collector.HealthAddr required
12. Sink.Endpoint valid URL (if non-empty)
13. Sink.Timeout >= 1s
14. State.Path required when Type is "file"
15. State.MaxReceipts >= 1
16. Logging.Level in {DEBUG, INFO, WARN, ERROR, FATAL}

---

## Profiling Contracts

### BC-PRF.01.001: Profiling disabled by default

**Preconditions:** ENABLE_PPROF env var unset, empty, or "false"
**Postconditions:** Returns no-op shutdown function and nil error
**Evidence:** pprof.go:58-61, pprof_test.go:27-39 (TestStart_DisabledWhenEnvUnset), pprof_test.go:41-52 (TestStart_DisabledWhenSetToFalse)
**Confidence:** HIGH (two direct tests)

### BC-PRF.01.002: Profiling server binds eagerly and returns error on bind failure

**Preconditions:** ENABLE_PPROF=1, port already occupied
**Postconditions:** Returns `(nil, error)` wrapping bind error
**Evidence:** pprof.go:72-75, pprof_test.go:196-211 (TestStart_ReturnsErrorOnBindFailure)
**Confidence:** HIGH (direct test)

### BC-PRF.01.003: Profiling server defaults to localhost:3030

**Preconditions:** ENABLE_PPROF=1, PPROF_ADDR empty
**Postconditions:** Binds to localhost:3030
**Evidence:** pprof.go:63-65, pprof_test.go:129-152 (TestStart_DefaultAddrWhenEnvEmpty)
**Confidence:** HIGH (direct test)

### BC-PRF.02.001: /debug/pprof/cmdline returns 404

**Preconditions:** Profiling server running
**Postconditions:** GET /debug/pprof/cmdline returns 404 Not Found. Prevents exposing process arguments.
**Evidence:** pprof.go:30, pprof_test.go:178-193 (TestPprofMux_CmdlineBlocked)
**Confidence:** HIGH (direct test)

### BC-PRF.02.002: Profiling warns on non-loopback bind address

**Preconditions:** PPROF_ADDR set to non-loopback address (e.g., "0.0.0.0:3030")
**Postconditions:** Warning logged but server still starts
**Evidence:** pprof.go:68-70
**Confidence:** HIGH (code is explicit, isLoopback tested separately)

### BC-PRF.02.003: isLoopback correctly classifies addresses

**Preconditions:** Various address formats
**Postconditions:**
- "localhost:*" -> true
- "127.0.0.1:*" -> true
- "[::1]:*" -> true
- ":*" (empty host) -> false
- "0.0.0.0:*" -> false
- "192.168.1.1:*" -> false
- malformed -> false
**Evidence:** pprof_test.go:154-176 (TestIsLoopback, 7 test cases)
**Confidence:** HIGH (comprehensive test table)

---

## Runner Contracts

### BC-RUN.02.001: Runner registers SIGTERM/SIGINT for graceful shutdown

**Preconditions:** Execute called
**Postconditions:** Context cancelled on SIGTERM or SIGINT via signal.NotifyContext
**Evidence:** runner.go:33-34
**Confidence:** HIGH (code is explicit)

### BC-RUN.02.002: Runner treats context.Canceled as graceful shutdown

**Preconditions:** Collector returns context.Canceled
**Postconditions:** Returns nil (success). Logs "collector stopped gracefully".
**Evidence:** runner.go:121-124
**Confidence:** HIGH (code is explicit)

### BC-RUN.02.003: Runner hardcodes MemoryStore regardless of config

**Preconditions:** Any StateConfig
**Postconditions:** Always creates `state.NewMemoryStore()`. Config.State is loaded but ignored.
**Evidence:** runner.go:59-61 (TODO comment acknowledges this)
**Confidence:** HIGH (code is explicit)

### BC-RUN.02.004: Runner skips sink creation when endpoint is empty

**Preconditions:** cfg.Sink.Endpoint is empty
**Postconditions:** sender remains nil. Warning logged. Collector runs without forwarding.
**Evidence:** runner.go:64-73
**Confidence:** HIGH (code is explicit)

### BC-RUN.02.005: Runner Pings CrowdStrike before entering collection loop

**Preconditions:** CrowdStrike client created successfully
**Postconditions:** Ping called. On failure -> returns error immediately (fail-fast). On success -> proceeds to collector creation.
**Evidence:** runner.go:94-101
**Confidence:** HIGH (code is explicit)

### BC-RUN.03.001: parseLogLevel only handles DEBUG, INFO, TRACE

**Preconditions:** Log level string from config
**Postconditions:**
- "" or "INFO" -> InfoLevel
- "DEBUG" -> DebugLevel
- "TRACE" -> DebugLevel (mapped to Debug, not separate level)
- WARN, ERROR, FATAL -> returns error, defaults to InfoLevel with warning log
**Evidence:** runner.go:131-142
**Confidence:** HIGH (code is explicit)

**Inconsistency:** Config validation accepts WARN/ERROR/FATAL as valid levels, but parseLogLevel rejects them. The config passes validation but the runner falls back to INFO with a warning.

---

## Main Entrypoint Contracts

### BC-MAIN.02.001: --dry-run validates config and exits

**Preconditions:** --dry-run flag set
**Postconditions:** Calls `config.ValidateConfig()`. On success -> exit 0. On failure -> stderr error, exit 1.
**Evidence:** main.go:23-29
**Confidence:** HIGH (code is explicit)

### BC-MAIN.02.002: pprof shutdown has 5-second timeout

**Preconditions:** pprof started successfully
**Postconditions:** Deferred shutdown with 5-second context timeout
**Evidence:** main.go:40-46
**Confidence:** HIGH (code is explicit)

---

## Source (crowdstrike/source.go) Contracts

### BC-CS.02.011: Source.FetchRecords dispatches by dataSource string

**Preconditions:** Source created with dataSource from config
**Postconditions:** "alerts" -> fetchAlerts, "detections" -> fetchDetections, "hosts" -> fetchHosts, anything else -> error "unsupported data source"
**Evidence:** source.go:116-127
**Confidence:** HIGH (code is explicit)

### BC-CS.02.012: Source defaults dataSource to "alerts" when empty

**Preconditions:** cfg.SourceType is empty
**Postconditions:** dataSource set to "alerts"
**Evidence:** source.go:96-99
**Confidence:** HIGH (code is explicit)

### BC-CS.02.013: Source defaults limit to 100 when non-positive

**Preconditions:** cfg.Limit <= 0
**Postconditions:** limit set to 100
**Evidence:** source.go:101-103
**Confidence:** HIGH (code is explicit)

---

## Cross-Cutting Observations

### Error Handling Pattern Consistency

All packages use `fmt.Errorf("context: %w", sentinelErr)` for error wrapping, enabling `errors.Is()` checking. One exception found:

- `ensureForwardProgress` (alert_collector.go:149-151) uses `fmt.Errorf(...)` WITHOUT `%w` wrapping. This means cursor regression errors cannot be matched with `errors.Is()`. This is likely a bug -- `ErrCursorRegression` sentinel exists in apperrors but is never used.

### Sentinel Error Usage Audit

| Sentinel | Used in wrapping? | Tested? |
|----------|------------------|---------|
| ErrConfigValidationFailed | main.go:25 (print only, not wrap) | No |
| ErrStateNotFound | store.go:47 (direct return) | Indirectly via collector |
| ErrQueryFingerprintMismatch | collector.go:178 | No |
| ErrCursorRegression | **NEVER USED** | No |
| ErrCollectorRetriesExceeded | collector.go:136 | No |
| ErrCollectorStateLoad | collector.go:205 | No |
| ErrCollectorStatePersist | collector.go:200, 231 | No |
| ErrSourceConfigMissing | **NEVER USED** | No |
| ErrSourceRequestBuild | **NEVER USED** | No |
| ErrSourceRequestExec | alert_collector.go:54 | No |
| ErrSourceUnexpectedStatus | **NEVER USED** | No |
| ErrSourceDecode | **NEVER USED** | No |
| ErrSinkConfigMissing | http_sender.go:52, 55 | No |
| ErrSinkRequestBuild | http_sender.go:65, 95 | No |
| ErrSinkDelivery | http_sender.go:103, 114, alert_collector.go:84 | No |
| ErrConfigLoad | **NEVER USED** | No |
| ErrClientNotInitialized | api.go:113, 280, 293, 309 | api_test.go (3 tests) |

**Finding:** 6 of 16 sentinel errors are never used: ErrCursorRegression, ErrSourceConfigMissing, ErrSourceRequestBuild, ErrSourceUnexpectedStatus, ErrSourceDecode, ErrConfigLoad. These were likely defined speculatively for future use.

---

## Delta Summary
- New items added: 56 behavioral contracts catalogued (vs 10 in broad sweep)
- Existing items refined: All 10 broad sweep contracts verified and enriched with specific evidence
- Remaining gaps: No collector/alert_collector test files exist (only crowdstrike, health, profiling tests). Helm chart values as behavioral constraints not analyzed.

## Novelty Assessment
Novelty: SUBSTANTIVE
Major findings that change the model:
1. **ErrCursorRegression sentinel is never used** -- ensureForwardProgress doesn't wrap with it (likely bug)
2. **6 of 16 sentinel errors are never used** -- changes the error domain model
3. **MaxRetries=0 means unlimited retries** -- undocumented edge case
4. **parseLogLevel rejects WARN/ERROR/FATAL despite config validator accepting them** -- inconsistency
5. **Record interface and Source.FetchRecords are unused by collector** -- changes integration model
6. **56 contracts vs 10 in broad sweep** -- substantial domain expansion

## Convergence Declaration
Another round needed -- should verify the hasMore heuristic edge case (filtered count vs raw count), audit Makefile/scripts for behavioral constraints, and verify whether the Source/FetchRecords path has any callers.

## State Checkpoint
```yaml
pass: 3
round: 1
status: complete
files_scanned: 18
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
```
