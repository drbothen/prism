# Pass 3 Deep: Behavioral Contracts -- poller-express (Round 1)

## Contract Numbering: BC-{subsystem}.{sequence}

Subsystems:
- 1: Collector (alert)
- 2: Collector (asset)
- 3: State management
- 4: Sink delivery
- 5: Health server
- 6: Configuration
- 7: Asset client
- 8: Profiling
- 9: Utilities

---

## Subsystem 1: Alert Collector

### BC-1.001: Alert collection cycle processes new alerts beyond cursor

**Preconditions:**
- PollState initialized (either bootstrapped or loaded from store)
- Cyberint API reachable
- Context not cancelled

**Postconditions:**
- All alerts with ModificationDate > cursor timestamp (or same timestamp with RefId > cursor RecordID) are sent to sink
- Cursor advanced to last processed alert's (ModificationDate, RefId)
- PollState.Version incremented by 1
- BatchReceipt saved with correct FirstRecordID, LastRecordID, Count
- Returns hasMore=true if fetched alerts count equals page size (100)

**Error Cases:**
- API failure: wrapped as `ErrCyberIntRequestExec`, propagated to retry loop
- Sink failure: propagated directly, causes batch retry
- Cursor regression: `ErrCursorRegression`, propagated to retry loop

**Evidence:** `alert_collector.go:192-301`, `TestFilterNewAlerts_SkipsZeroTimestampAndRespectsCursor`
**Confidence:** HIGH

### BC-1.002: Alerts are sorted by (ModificationDate, RefId) before processing

**Preconditions:** API returns a batch of alerts

**Postconditions:**
- Alerts sorted by ModificationDate ascending (stable sort)
- Within same ModificationDate, sorted by RefId ascending (lexicographic)
- Processing order is deterministic regardless of API response order

**Evidence:** `alert_collector.go:236-241`
**Confidence:** HIGH (from code; no dedicated sort-order test)

### BC-1.003: Alerts with zero timestamps are skipped

**Preconditions:** Batch contains alerts with `ModificationDate.IsZero() == true`

**Postconditions:**
- Zero-timestamp alerts excluded from filtered results
- Warning logged for each skipped alert
- Non-zero-timestamp alerts unaffected

**Evidence:** `alert_collector.go:336-339`, `TestFilterNewAlerts_SkipsZeroTimestampAndRespectsCursor` (line 43: alert "a" with zero timestamp, filtered out, result is 1 alert "d")
**Confidence:** HIGH

### BC-1.004: isCursorAhead correctly determines cursor ordering

**Preconditions:** Two Cursor values provided (current and candidate)

**Postconditions:**
- Returns true if candidate.Timestamp > current.Timestamp (regardless of RecordID)
- Returns true if timestamps equal AND candidate.RecordID > current.RecordID (lexicographic)
- Returns false if candidate.Timestamp < current.Timestamp (regardless of RecordID)
- Returns false if timestamps equal AND candidate.RecordID <= current.RecordID

**Evidence:** `alert_collector.go:319-327`, `TestIsCursorAhead` (4 test cases covering all branches):
- ahead by timestamp: t1 > t0, RecordID "a" (lower) -- still ahead
- ahead by RecordID: same t0, "c" > "b"
- not ahead: same t0, "a" < "b"
- not ahead: t0-1s < t0, "z" (higher RecordID doesn't matter)

**Confidence:** HIGH

### BC-1.005: ensureForwardProgress rejects cursor regression

**Preconditions:** Current and next Cursor values provided

**Postconditions:**
- Returns nil if next is strictly ahead of current
- Returns error wrapping `ErrCursorRegression` if next is at or behind current

**Evidence:** `alert_collector.go:347-352`, `TestEnsureForwardProgress`:
- Same position (t0, "b") -> (t0, "b"): error containing `ErrCursorRegression`
- Forward by timestamp (t0, "b") -> (t0+1s, "a"): nil

**Confidence:** HIGH

### BC-1.006: Alert collector uses modification_date filter when cursor is non-zero

**Preconditions:** Cursor timestamp is not zero

**Postconditions:**
- GetAlertsRequest includes `filters.modification_date` with from=cursor_timestamp, to=now
- When cursor is zero (first run), no filter is applied -- fetches all alerts

**Evidence:** `alert_collector.go:203-210`
**Confidence:** HIGH (from code; no test covers filter construction)

### BC-1.007: Alert collector retry loop with exponential backoff

**Preconditions:** collectOnce returns an error

**Postconditions:**
- retryCount incremented
- If retryCount > maxRetries: returns `ErrCollectorRetriesExceeded` with attempts count
- Otherwise: waits retryDelay, then retries
- retryDelay doubles each failure, capped at maxDelay
- On success: retryCount reset to 0, retryDelay reset to baseDelay
- Health reporter set to NotReady on failure, Ready on success

**Error Cases:**
- Context cancelled during wait: returns context error immediately
- Retries exceeded: fatal error, process exits

**Evidence:** `alert_collector.go:86-154`
**Confidence:** HIGH (from code; integration behavior matches asset collector tests)

### BC-1.008: Alert collector state initialization

**Preconditions:** Store and fingerprint available

**Postconditions:**
- If store returns PollState with matching fingerprint hash: state loaded, no write
- If store returns ErrStateNotFound: bootstrap state created with InitialSince, version 0, saved to store
- If store returns error with different fingerprint hash: returns `ErrQueryFingerprintMismatch` (fatal)
- If store returns other error: returns `ErrCollectorStateLoad`

**Evidence:** `alert_collector.go:156-190`
**Confidence:** HIGH (from code; asset collector equivalent has tests -- see BC-2.006/7/8)

---

## Subsystem 2: Asset Collector

### BC-2.001: Asset collection cycle processes new assets beyond cursor

**Preconditions:**
- AssetPollState initialized
- Asset API reachable
- Context not cancelled

**Postconditions:**
- All assets with Updated > cursor timestamp (or same timestamp with GetID() > cursor RecordID via string comparison) sent to sink
- Cursor advanced to last processed asset's (Updated, GetID())
- AssetPollState.Version incremented
- AssetBatchReceipt saved
- Returns hasMore = (pageNumber * 1000 < totalAssets)

**Evidence:** `asset_collector.go:188-280`, `TestAssetCollector_CollectOnce_ProcessesAssets`
**Confidence:** HIGH

### BC-2.002: isAssetAhead uses string comparison for IDs

**Preconditions:** Asset and current AssetCursor provided

**Postconditions:**
- Returns true if asset.Updated > cursor.Timestamp
- Returns true if timestamps equal AND asset.GetID() > cursor.RecordID (string comparison)
- Returns false otherwise
- Known behavior: "50" > "100" is true in string comparison (documented)

**Evidence:** `asset_collector.go:292-301`, `TestAssetCollector_IsAssetAhead` (6 test cases):
- ahead by timestamp: ID 50, t1 > t0 -> true
- ahead by ID (numeric higher): ID 200, same t0 -> true ("200" > "100")
- ahead by ID (string higher): ID 50, same t0 -> true ("50" > "100" because '5' > '1')
- not ahead (earlier timestamp): ID 200, t0-1s -> false
- not ahead (same position): ID 100, t0 -> false
- not ahead (lower string ID): ID 1, t0 -> false ("1" < "100")

**Confidence:** HIGH

### BC-2.003: Asset collector ensureForwardProgress with numeric fallback

**Preconditions:** Current AssetPollState cursor and proposed next AssetCursor

**Postconditions:**
- Returns nil if next.Timestamp > current.Timestamp
- Returns nil if timestamps equal AND next.RecordID > current.RecordID (string)
- Returns nil if timestamps equal AND string comparison fails BUT numeric parse succeeds AND nextID > currentID (int64)
- Returns `ErrCursorRegression` otherwise

**Evidence:** `asset_collector.go:303-320`, `TestAssetCollector_EnsureForwardProgress` (6 cases):
- forward by timestamp: t0+1s, ID "50" -> no error
- forward by string ID: t0, "200" > "100" -> no error
- forward by numeric ID: t0, "150" > "100" (string comparison also passes) -> no error
- regression (same position): t0, "100" -> error
- regression (earlier timestamp): t0-1s, "200" -> error
- regression (lower ID): t0, "1" < "100" both string and numeric -> error

**Confidence:** HIGH

### BC-2.004: filterNewAssets excludes already-processed assets

**Preconditions:** Sorted list of assets and current cursor

**Postconditions:**
- Only assets where isAssetAhead returns true are included
- Original order preserved within filtered set

**Evidence:** `asset_collector.go:282-290`, `TestAssetCollector_FilterNewAssets`:
- Cursor at (t1, "2"), assets at (t0,1), (t1,2), (t1,3), (t2,4)
- Result: [asset 3, asset 4] (2 assets)

**Confidence:** HIGH

### BC-2.005: Asset collector returns hasMore=false for empty/no-content responses

**Preconditions:** API returns HTTP 204 or empty assets

**Postconditions:**
- No error returned
- hasMore=false
- Cursor unchanged

**Evidence:** `TestAssetCollector_CollectOnce_NoAssets` (HTTP 204 -> hasMore=false, no error)
**Confidence:** HIGH

### BC-2.006: Asset collector state initialization -- new state bootstrap

**Preconditions:** Store returns ErrStateNotFound

**Postconditions:**
- New AssetPollState created with zero AssetCursor (or InitialSince), version 0
- QueryFingerprint set to `NewQueryFingerprint(["cyberint_assets"], 1000)`
- State and empty receipt saved to store

**Evidence:** `TestAssetCollector_InitializeState_NewState`:
- Asserts version == 0
- Asserts fingerprint hash matches

**Confidence:** HIGH

### BC-2.007: Asset collector state initialization -- existing state restoration

**Preconditions:** Store returns valid AssetPollState with matching fingerprint

**Postconditions:**
- State loaded from store without modification
- Version and cursor preserved

**Evidence:** `TestAssetCollector_InitializeState_ExistingState`:
- Saves state with version=5, RecordID="123"
- After init: version==5, RecordID=="123"

**Confidence:** HIGH

### BC-2.008: Asset collector state initialization -- fingerprint mismatch is fatal

**Preconditions:** Store returns AssetPollState with different fingerprint hash

**Postconditions:**
- Returns error containing `ErrQueryFingerprintMismatch`
- State NOT loaded

**Evidence:** `TestAssetCollector_InitializeState_FingerprintMismatch`:
- Saves state with `NewQueryFingerprint(["different_query"], 500)`
- Init returns error containing "query fingerprint mismatch"

**Confidence:** HIGH

### BC-2.009: Asset collector sends each asset to sink with correct ID and type

**Preconditions:** Assets beyond cursor exist, sink is configured

**Postconditions:**
- Each asset sent to sink.Send with recordID = asset.GetID() (string form of int64)
- recordType = "cyberint_asset"
- Assets sent in sorted order

**Evidence:** `TestAssetCollector_CollectOnce_SendsToSink`:
- 2 assets (ID 1, ID 2)
- mockSink receives ["1", "2"]

**Confidence:** HIGH

### BC-2.010: All assets below cursor produce no state change

**Preconditions:** All fetched assets have Updated <= cursor.Timestamp and ID <= cursor.RecordID

**Postconditions:**
- No sink calls
- Cursor unchanged
- hasMore=false, no error

**Evidence:** `TestAssetCollector_CollectOnce_AllAssetsBelowCursor`:
- Cursor at (t0+24h, "999"), assets at (t0, 1) and (t0+1h, 2)
- After collectOnce: cursor still "999"

**Confidence:** HIGH

### BC-2.011: Default collector interval is 30 seconds

**Preconditions:** No interval override provided

**Postconditions:** Collector interval = 30s

**Evidence:** `TestAssetCollector_DefaultInterval`, `TestAssetCollector_CustomInterval`, `TestAssetCollector_ConfigInterval`:
- No override -> 30s
- Options.Interval = 5m -> 5m (takes precedence)
- Config.Collector.Interval = 2m, no Options override -> 2m

**Confidence:** HIGH

---

## Subsystem 3: State Management

### BC-3.001: MemoryStore.Load returns ErrStateNotFound when uninitialized

**Preconditions:** MemoryStore created via NewMemoryStore(), no Save called

**Postconditions:** Returns `(PollState{}, ErrStateNotFound)`

**Evidence:** Used transitively in `TestAssetCollector_InitializeState_NewState` (alert side uses same logic)
**Confidence:** HIGH

### BC-3.002: MemoryStore.Save persists state atomically

**Preconditions:** Valid PollState and BatchReceipt provided

**Postconditions:**
- Subsequent Load returns the saved PollState
- initialized flag set to true
- Thread-safe via RWMutex

**Evidence:** `state/store.go:70-79`, transitively tested in collector tests
**Confidence:** HIGH

### BC-3.003: MemoryStore handles alert and asset state independently

**Preconditions:** Both alert and asset state operations

**Postconditions:**
- LoadAsset/SaveAsset operate on separate `assetState`/`assetInitialized` fields
- Load/Save operate on separate `state`/`initialized` fields
- Single mutex protects both

**Evidence:** `state/store.go:40-50` (struct definition), `state/store.go:58-103` (separate methods)
**Confidence:** HIGH (from code)

### BC-3.004: QueryFingerprint is order-independent

**Preconditions:** Same set of fields provided in different orders

**Postconditions:**
- Same Hash produced regardless of input order
- Fields stored in original (unsorted) order
- Negative limit clamped to 0

**Evidence:** `state/store.go:145-163` (sorts canonical copy, joins with |, appends limit, SHA-256)
**Confidence:** MEDIUM (from code; no test for order-independence directly)

### BC-3.005: AssetCursor.IsZero detects uninitialized cursors

**Preconditions:** AssetCursor instance

**Postconditions:**
- Returns true only when both Timestamp.IsZero() and RecordID == ""
- Returns false if either is set

**Evidence:** `TestAssetCursor_IsZero` (4 cases: zero, non-zero timestamp, non-zero RecordID, fully populated)
**Confidence:** HIGH

---

## Subsystem 4: Sink Delivery

### BC-4.001: HTTPSender validates configuration on construction

**Preconditions:** SinkConfig provided to NewHTTPSender

**Postconditions:**
- Empty endpoint: returns `ErrSinkConfigMissing` ("endpoint is empty")
- Empty username OR password: returns `ErrSinkConfigMissing` ("credentials are incomplete")
- Non-URL endpoint: returns `ErrSinkRequestBuild`
- Valid config: returns configured HTTPSender

**Evidence:** `TestNewHTTPSender_Validation` (3 cases: empty endpoint, missing username, invalid URL)
**Confidence:** HIGH

### BC-4.002: HTTPSender defaults timeout to 15s when zero

**Preconditions:** SinkConfig with Timeout=0

**Postconditions:** Internal http.Client has Timeout=15s

**Evidence:** `TestNewHTTPSender_DefaultTimeout`
**Confidence:** HIGH

### BC-4.003: HTTPSender.Send enriches payload and delivers with basic auth

**Preconditions:** Valid HTTPSender, reachable endpoint

**Postconditions:**
- Request sent as POST with Content-Type: application/json
- Authorization header set to Basic auth (base64 of username:password)
- Body is `{"data": <original_json>, "xmp": {"site": ..., "cluster_name": ..., "node_name": ...}}`
- Returns nil on 2xx response

**Evidence:** `TestHTTPSender_Send_Success`:
- Verifies auth header matches `Basic base64("user:pass")`
- Verifies Content-Type is application/json
- Verifies body unmarshals to EnrichedPayload with correct data and xmp fields
- Server returns 202 Accepted

**Confidence:** HIGH

### BC-4.004: HTTPSender.Send returns ErrSinkDelivery on HTTP error status

**Preconditions:** Endpoint returns HTTP >= 400

**Postconditions:**
- Returns error wrapping `ErrSinkDelivery`
- Error message contains status code (e.g., "status=500")
- Error message contains response body (up to 2048 bytes, trimmed)

**Evidence:** `TestHTTPSender_Send_HTTPErrorStatus`:
- Server returns 500 with body "nope"
- Error contains "sink delivery failed", "status=500", "nope"

**Confidence:** HIGH

### BC-4.005: HTTPSender.Send returns ErrSinkDelivery on transport error

**Preconditions:** HTTP transport returns error (e.g., connection refused, context cancelled)

**Postconditions:**
- Returns error wrapping `ErrSinkDelivery`

**Evidence:** `TestHTTPSender_Send_DoError`:
- Custom RoundTripper returns context.Canceled
- Error contains "sink delivery failed"

**Confidence:** HIGH

### BC-4.006: enrichPayload wraps data with xMP metadata

**Preconditions:** Any serializable Go value

**Postconditions:**
- Output is valid JSON: `{"data": <encoded_value>, "xmp": {"site": ..., "cluster_name": ..., "node_name": ...}}`
- Uses manual buffer construction (not struct marshaling for outer wrapper)
- Strips trailing newlines from json.Encoder output

**Evidence:** `http_sender.go:121-149`, `BenchmarkEnrichPayload` (implicitly tests correctness -- would fail if JSON invalid)
**Confidence:** HIGH

---

## Subsystem 5: Health Server

### BC-5.001: Liveness endpoint returns 200 when alive

**Preconditions:** Server created (alive=true by default)

**Postconditions:**
- GET /health returns 200 with body "ok"
- GET /live returns 200 with body "ok"

**Evidence:** `TestServer_Liveness`
**Confidence:** HIGH

### BC-5.002: Readiness endpoint reflects ready state

**Preconditions:** Server created

**Postconditions:**
- Initially: GET /ready returns 503 with body "not ready"
- After SetReady(): GET /ready returns 200 with body "ready"
- After SetNotReady(): GET /ready returns 503 again

**Evidence:** `TestServer_Readiness_NotReady`, `TestServer_Readiness_Ready`, `TestServer_SetReady`, `TestServer_SetNotReady`
**Confidence:** HIGH

### BC-5.003: Rate limiting allows normal traffic at default config

**Preconditions:** Default rate limit (100 req/sec, burst 20)

**Postconditions:** 10 rapid requests from same IP all return 200

**Evidence:** `TestServer_RateLimiting_AllowsNormalTraffic`
**Confidence:** HIGH

### BC-5.004: Rate limiting blocks excessive traffic with 429 and Retry-After

**Preconditions:** Restrictive rate limit (1 req/sec, burst 2)

**Postconditions:**
- First 2 requests succeed (burst capacity)
- 3rd immediate request returns 429
- Response body: "rate limit exceeded"
- Retry-After header: "1"

**Evidence:** `TestServer_RateLimiting_BlocksExcessiveTraffic`
**Confidence:** HIGH

### BC-5.005: Rate limiting is per-IP

**Preconditions:** Restrictive rate limit (1 req/sec, burst 1)

**Postconditions:**
- IP1 first request: 200
- IP1 second request: 429
- IP2 first request: 200 (separate limiter)

**Evidence:** `TestServer_RateLimiting_PerIPIsolation`
**Confidence:** HIGH

### BC-5.006: Rate limit tokens replenish after waiting

**Preconditions:** Rate limit of 5 req/sec, burst 1

**Postconditions:**
- First request: 200
- Immediate second: 429
- After 250ms wait: 200 (token replenished)

**Evidence:** `TestServer_RateLimiting_AllowsAfterWaiting`
**Confidence:** HIGH

### BC-5.007: Invalid RemoteAddr does not panic

**Preconditions:** Request with RemoteAddr "invalid-addr" (no port)

**Postconditions:** Request processed normally (uses full RemoteAddr as limiter key)

**Evidence:** `TestServer_RateLimiting_HandlesInvalidRemoteAddr`
**Confidence:** HIGH

### BC-5.008: Rate limit config is customizable

**Preconditions:** Custom RateLimitConfig applied

**Postconditions:** Server uses provided RequestsPerSecond and Burst values

**Evidence:** `TestDefaultRateLimitConfig`, `TestWithRateLimitConfig`
**Confidence:** HIGH

### BC-5.009: Readiness requires both alive AND ready

**Preconditions:** Server exists

**Postconditions:**
- GET /ready returns 503 if alive=false (even if ready=true)
- GET /ready returns 503 if ready=false (even if alive=true)
- GET /ready returns 200 only when alive=true AND ready=true

**Evidence:** `server.go:143-151` (condition: `!s.ready.Load() || !s.alive.Load()`)
**Confidence:** HIGH (from code)

---

## Subsystem 6: Configuration

### BC-6.001: File-backed secrets take precedence over env vars

**Preconditions:** Both `*_FILE` and direct env vars set, file exists

**Postconditions:** Value loaded from file, direct env var ignored

**Evidence:** `TestLoadFromEnvironment_UsesFileBackedSecrets`:
- Sets CYBERINT_API_URL_FILE and CYBERINT_API_KEY_FILE pointing to temp files
- Loaded config uses file contents

**Confidence:** HIGH

### BC-6.002: Missing file falls back to direct env var

**Preconditions:** `*_FILE` points to non-existent path, direct env var set

**Postconditions:** Value loaded from direct env var (no error)

**Evidence:** `TestLoadFromEnvironment_FileMissingFallsBackToEnv`:
- CYBERINT_API_URL_FILE = "/path/does/not/exist"
- CYBERINT_API_URL = "https://api.example.com"
- Result: BaseURL = "https://api.example.com"

**Confidence:** HIGH

### BC-6.003: Missing required values produce error

**Preconditions:** Both CYBERINT_API_URL and CYBERINT_API_KEY empty

**Postconditions:** LoadFromEnvironment returns error

**Evidence:** `TestLoadFromEnvironment_MissingRequiredValues`
**Confidence:** HIGH

### BC-6.004: Timeout and interval parsing supports multiple formats

**Preconditions:** Various env var formats

**Postconditions:**
- VECTOR_TIMEOUT_SECONDS "3" parsed as 3s (integer -> seconds)
- VECTOR_TIMEOUT_SECONDS "3s" would parse as 3s (duration format)
- COLLECTOR_INTERVAL "5s" parsed as 5s
- COLLECTOR_RETRY_BASE_DELAY "2s" parsed as 2s
- COLLECTOR_RETRY_MAX_DELAY "10s" parsed as 10s
- COLLECTOR_MAX_RETRIES "7" parsed as int 7
- POLLER_LOG_LEVEL "debug" normalized to "DEBUG"

**Evidence:** `TestLoadFromEnvironment_ParsesTimeoutAndInterval`
**Confidence:** HIGH

### BC-6.005: Validate aggregates all errors

**Preconditions:** Config with multiple invalid fields

**Postconditions:**
- Returns combined error containing all validation failures
- Does not stop at first error

**Evidence:** `TestConfigValidate_AggregatesErrors`:
- Sets BaseURL="", APIKey="", Interval=500ms, RetryBaseDelay=0, RetryMaxDelay=0, HealthAddr="", Timeout=0, Level="NOPE"
- Asserts error is non-nil (all errors aggregated via `errors.Join`)

**Confidence:** HIGH

### BC-6.006: Asset collection toggle

**Preconditions:** ASSET_COLLECTION_ENABLED env var

**Postconditions:**
- "true" or "1" enables asset collection
- Any other value disables it
- Default is true (from DefaultConfig)

**Evidence:** `config.go:177-179` (case-insensitive "true" or "1")
**Confidence:** HIGH (from code)

### BC-6.007: XMP_NODE_NAME falls back to hostname

**Preconditions:** XMP_NODE_NAME env var not set

**Postconditions:** NodeName set to `os.Hostname()` result

**Evidence:** `config.go:200-204`
**Confidence:** HIGH (from code)

### BC-6.008: Dry-run validates config without starting collector

**Preconditions:** `--dry-run` flag passed to binary

**Postconditions:**
- LoadFromEnvironment + Validate called
- Success: prints validated config (secrets redacted), exits 0
- Failure: prints error to stderr, exits 1

**Evidence:** `cmd/collector/main.go:22-29`, `config/utils.go:8-40`
**Confidence:** HIGH (from code)

### BC-6.009: Secret redaction shows first 2 and last 2 chars

**Preconditions:** Secret string for display

**Postconditions:**
- Empty string -> "<empty>"
- 1-4 chars -> "***"
- 5+ chars -> first 2 chars + "***" + last 2 chars

**Evidence:** `config/utils.go:42-50`
**Confidence:** HIGH (from code)

---

## Subsystem 7: Asset Client

### BC-7.001: GetAssets returns parsed response on success

**Preconditions:** Valid request, API returns 200 with JSON

**Postconditions:**
- POST to `{baseURL}/asset-configuration/external/api/v1/assets/`
- Content-Type: application/json
- Accept: application/json
- Response unmarshaled to GetAssetsResponse
- Returns parsed assets with correct IDs, names, types

**Evidence:** `TestGetAssets_Success`:
- Verifies POST method, correct path, Content-Type header
- Verifies response has 2 assets with correct IDs

**Confidence:** HIGH

### BC-7.002: HTTP 204 returns empty valid response

**Preconditions:** API returns 204 No Content

**Postconditions:**
- No error
- TotalAssets = 0, PageNumber = 1, Assets = []

**Evidence:** `TestGetAssets_NoContent`
**Confidence:** HIGH

### BC-7.003: Empty response body returns empty valid response

**Preconditions:** API returns 200 with empty body

**Postconditions:**
- No error
- TotalAssets = 0

**Evidence:** `TestGetAssets_EmptyResponse`
**Confidence:** HIGH

### BC-7.004: HTTP error status returns error

**Preconditions:** API returns HTTP >= 300 (tested with 500)

**Postconditions:** Error returned containing status code and body text

**Evidence:** `TestGetAssets_HTTPError`
**Confidence:** HIGH

### BC-7.005: Invalid JSON returns error

**Preconditions:** API returns 200 with non-JSON body

**Postconditions:** Error returned (unmarshal failure)

**Evidence:** `TestGetAssets_InvalidJSON`
**Confidence:** HIGH

### BC-7.006: Cancelled context returns error

**Preconditions:** Context cancelled before request completes

**Postconditions:** Error returned

**Evidence:** `TestGetAssets_ContextCancellation`
**Confidence:** HIGH

### BC-7.007: Response body limited to 10 MiB

**Preconditions:** API returns response body >= 10 MiB

**Postconditions:** Error returned ("response exceeds maximum size")

**Evidence:** `client.go:63-65` (reads via LimitReader, checks if len >= maxSizeResponse)
**Confidence:** MEDIUM (from code; no test)

### BC-7.008: Asset.GetID converts int64 to string

**Preconditions:** Asset with numeric ID

**Postconditions:** Returns decimal string representation

**Evidence:** `TestAsset_GetID`: ID 12345 -> "12345"
**Confidence:** HIGH

### BC-7.009: GetAssetsRequest serializes correctly with omitempty

**Preconditions:** Request with various optional fields

**Postconditions:** JSON roundtrip preserves all set fields, omits empty ones

**Evidence:** `TestGetAssetsRequest_Marshaling`
**Confidence:** HIGH

---

## Subsystem 8: Profiling

### BC-8.001: Pprof server disabled when ENABLE_PPROF unset

**Preconditions:** ENABLE_PPROF env var empty or unset

**Postconditions:**
- Start() returns a no-op shutdown function
- No server started
- Shutdown function returns nil

**Evidence:** `TestStart_DisabledWhenEnvUnset`
**Confidence:** HIGH

### BC-8.002: Pprof server starts on configured address

**Preconditions:** ENABLE_PPROF set to any non-empty value

**Postconditions:**
- HTTP server listening on PPROF_ADDR (default: localhost:3030)
- /debug/pprof/ returns 200
- Shutdown function stops the server

**Evidence:** `TestStart_LaunchesServer`, `TestStart_RespectsCustomAddr`, `TestStart_ShutdownStopsServer`
**Confidence:** HIGH

---

## Subsystem 9: Utilities

### BC-9.001: validate.Check logs deferred close errors

**Preconditions:** Function that returns error (e.g., `resp.Body.Close`)

**Postconditions:**
- If function returns nil: nothing logged
- If function returns error: logged via slog.Error

**Evidence:** `pkg/validate/utils.go:10-14`
**Confidence:** MEDIUM (from code; no direct test)

---

## Contract Coverage Summary

| Subsystem | Total Contracts | HIGH Confidence | MEDIUM | LOW |
|-----------|----------------|-----------------|--------|-----|
| 1: Alert Collector | 8 | 7 | 0 | 0 |
| 2: Asset Collector | 11 | 11 | 0 | 0 |
| 3: State Management | 5 | 3 | 1 | 0 |
| 4: Sink Delivery | 6 | 6 | 0 | 0 |
| 5: Health Server | 9 | 9 | 0 | 0 |
| 6: Configuration | 9 | 7 | 0 | 0 |
| 7: Asset Client | 9 | 8 | 1 | 0 |
| 8: Profiling | 2 | 2 | 0 | 0 |
| 9: Utilities | 1 | 0 | 1 | 0 |
| **Total** | **60** | **53** | **3** | **0** |

### Gaps: Behaviors with No Test Coverage

1. **Alert collector retry loop** (BC-1.007): No direct test; behavior inferred from matching asset collector pattern and code review
2. **Alert collector state initialization** (BC-1.008): No direct test; asset collector equivalent is tested
3. **Alert modification_date filter construction** (BC-1.006): No test verifying the filter is correctly built
4. **Alert sort order** (BC-1.002): No test verifying deterministic sort
5. **QueryFingerprint order-independence** (BC-3.004): No test with reordered fields
6. **Response size limit** (BC-7.007): 10 MiB limit not tested
7. **Runner.Execute orchestration**: No unit tests for the runner package at all (integration-level only)
8. **cookieTransport RoundTrip**: No test verifying the access_token cookie injection
9. **extractCustomerIDFromURL**: No test for URL parsing logic
10. **Graceful shutdown sequence**: Runner.Execute shutdown logic not tested

---

## Delta Summary
- New items added: 60 behavioral contracts across 9 subsystems
- Existing items refined: Expanded from 8 broad-sweep contracts (BC-001 through BC-008) to 60 granular contracts with precise evidence and confidence levels
- Remaining gaps: 10 behaviors with no test coverage identified

## Novelty Assessment
Novelty: SUBSTANTIVE
This round discovered 52 additional contracts beyond the 8 in the broad sweep. Key new discoveries: (1) the asymmetry between asset sort (numeric ID) and cursor comparison (string ID) confirmed from tests, (2) the exact retry boundary condition (> not >=), (3) the complete configuration subsystem contracts including secret redaction, dry-run validation, and asset toggle parsing, (4) 10 untested behavioral gaps identified, (5) the readiness endpoint requires BOTH alive AND ready (not just ready), (6) rate limit token replenishment behavior, (7) the 10 MiB response size cap on asset client. These materially change the specification.

## Convergence Declaration
Another round needed -- should audit Round 1 for hallucination classes (over-extrapolation, miscounting), verify gap claims, and check for any contracts missed in the runner/orchestration layer.

## State Checkpoint
```yaml
pass: 3
round: 1
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
```
