# Pass 3 Deep: Behavioral Contracts -- Round 1

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 1

---

## Contract Numbering Convention

Format: `BC-S.SS.NNN` where S = section (1=collection, 2=state, 3=sink, 4=health, 5=config, 6=ocsf, 7=transport), SS = subsection, NNN = sequence.

---

## Section 1: Collection Contracts

### BC-1.01.001: Alert Collection Happy Path

**Preconditions:** Claroty client returns non-empty AlertsBatch; sink is non-nil; alert state is initialized.
**Postconditions:**
- All alerts in batch are sent to sink individually, in order
- Cursor advances to `batch.Last` (Timestamp + AlertID)
- State version increments by 1
- BatchReceipt is persisted with count, first/last IDs, fetch timestamp
- Returns `hasMore=true` when `len(batch.Alerts) >= limit`, `false` otherwise
**Error Cases:** None (happy path)
**Evidence:** `TestCollectAlertsIntegration` (collector_test.go:195-258)
**Confidence:** HIGH (directly from test assertions on cursor, version, sink count, hasMore)

### BC-1.01.002: Alert Collection -- Claroty Client Error Prevents Sink Delivery and State Save

**Preconditions:** Claroty client returns error on FetchAlerts.
**Postconditions:**
- No alerts sent to sink (sink count = 0)
- State is NOT saved (load returns ErrStateNotFound)
- Error is propagated to caller
**Evidence:** `TestCollectAlerts_ClarotyClientError` (collector_test.go:606-640)
**Confidence:** HIGH

### BC-1.01.003: Alert Collection -- Sink Error Prevents State Save

**Preconditions:** Claroty returns valid batch; sink returns error on SendAlert.
**Postconditions:**
- State is NOT saved (cursor does not advance)
- Error wraps ErrSinkDelivery
**Evidence:** `TestCollectAlerts_SinkError` (collector_test.go:642-690)
**Confidence:** HIGH

### BC-1.01.004: Alert Collection -- Empty Batch Returns hasMore=false Without Side Effects

**Preconditions:** Claroty returns batch with empty Alerts slice.
**Postconditions:**
- No alerts sent to sink
- No state change
- Returns (false, nil)
**Evidence:** `TestCollectAlerts_EmptyBatch` (collector_test.go:692-729)
**Confidence:** HIGH

### BC-1.01.005: Alert Collection -- Pagination Indicator (hasMore)

**Preconditions:** Batch size equals configured limit.
**Postconditions:** Returns `hasMore=true`, signaling immediate re-collection without waiting for ticker.
**Evidence:** `TestCollectAlerts_PaginationIndicator` (collector_test.go:731-778)
**Confidence:** HIGH

### BC-1.01.006: Alert Collection -- Context Cancellation

**Preconditions:** Context is cancelled before or during collection.
**Postconditions:** No panic occurs; behavior is graceful.
**Evidence:** `TestCollectAlerts_ContextCancellation` (collector_test.go:984-1027)
**Confidence:** MEDIUM (test only verifies no panic, not specific error behavior)

### BC-1.02.001: Event Collection Happy Path

**Preconditions:** Same pattern as alerts but with EventsBatch/EventsCursor.
**Postconditions:** All events forwarded; cursor advances to `batch.Last` (Timestamp + EventID); version increments.
**Evidence:** `TestCollectEventsIntegration` (collector_test.go:260-323)
**Confidence:** HIGH

### BC-1.02.002: Event Collection -- Client Error

**Preconditions:** FetchEvents returns error.
**Postconditions:** No events to sink; error propagated.
**Evidence:** `TestCollectEvents_ClarotyClientError` (collector_test.go:780-807)
**Confidence:** HIGH

### BC-1.02.003: Event Collection -- Sink Error

**Preconditions:** FetchEvents succeeds; SendActivityEvent fails.
**Postconditions:** Error propagated; state not saved.
**Evidence:** `TestCollectEvents_SinkError` (collector_test.go:809-850)
**Confidence:** HIGH

### BC-1.03.001: AuditLog Collection Happy Path

**Preconditions:** FetchAuditLogs returns non-empty batch.
**Postconditions:**
- All audit logs forwarded
- Cursor Offset = `batch.Last.Offset + 1` (NOT batch.Last.Offset -- unique among all 9 sources)
- Cursor Timestamp and AuditLogID = batch.Last values
- Version increments
**Evidence:** `TestCollectAuditLogsIntegration` (collector_test.go:325-390) -- line 383 asserts `saved.Cursor.Offset != batch.Last.Offset+1`
**Confidence:** HIGH

### BC-1.03.002: AuditLog Collection -- Client Error

**Preconditions:** FetchAuditLogs returns error.
**Postconditions:** No logs to sink; error propagated.
**Evidence:** `TestCollectAuditLogs_ClarotyClientError` (collector_test.go:852-879)
**Confidence:** HIGH

### BC-1.03.003: AuditLog Collection -- Sink Error

**Preconditions:** Fetch succeeds; SendAuditLog fails.
**Postconditions:** Error propagated; state not saved.
**Evidence:** `TestCollectAuditLogs_SinkError` (collector_test.go:881-924)
**Confidence:** HIGH

### BC-1.04.001: DeviceAlertRelation Collection Happy Path

**Preconditions:** FetchDeviceAlertRelations returns non-empty batch.
**Postconditions:**
- All relations forwarded
- Cursor advances to batch.Last (Timestamp, AlertID, DeviceUID)
- Version increments
**Evidence:** `TestCollectDeviceAlertRelations` (collector_test.go:392-463)
**Confidence:** HIGH

### BC-1.04.002: DeviceAlertRelation Collection -- Client Error

**Preconditions:** FetchDeviceAlertRelations returns error.
**Postconditions:** No relations to sink; error propagated.
**Evidence:** `TestCollectDeviceAlertRelations_ClarotyClientError` (collector_test.go:926-953)
**Confidence:** HIGH

### BC-1.05.001: DeviceVulnerabilityRelation Collection Happy Path

**Preconditions:** FetchDeviceVulnerabilityRelations returns non-empty batch.
**Postconditions:**
- All relations forwarded
- Cursor advances to batch.Last (DetectionTime, DeviceUID, VulnerabilityID)
- Version increments
**Evidence:** `TestCollectDeviceVulnerabilityRelationsIntegration` (collector_test.go:465-538)
**Confidence:** HIGH

### BC-1.05.002: DeviceVulnerabilityRelation Collection -- Client Error

**Preconditions:** FetchDeviceVulnerabilityRelations returns error.
**Postconditions:** No relations to sink; error propagated.
**Evidence:** `TestCollectDeviceVulnerabilityRelations_ClarotyClientError` (collector_test.go:955-982)
**Confidence:** HIGH

### BC-1.06.001: Vulnerability Collection Happy Path

**Preconditions:** FetchVulnerabilities returns non-empty batch.
**Postconditions:**
- All vulnerabilities forwarded
- Cursor advances (Offset, VulnerabilityID)
- Version increments
**Evidence:** `TestCollectVulnerabilitiesIntegration` (collector_test.go:1029-1092)
**Confidence:** HIGH

### BC-1.06.002: Vulnerability Collection -- Client Error

**Preconditions:** FetchVulnerabilities returns error.
**Postconditions:** No vulnerabilities to sink; state not saved.
**Evidence:** `TestCollectVulnerabilities_ClarotyClientError` (collector_test.go:1094-1128)
**Confidence:** HIGH

### BC-1.06.003: Vulnerability Collection -- Sink Error

**Preconditions:** Fetch succeeds; SendVulnerability fails.
**Postconditions:** State not saved; error propagated.
**Evidence:** `TestCollectVulnerabilities_SinkError` (collector_test.go:1130-1177)
**Confidence:** HIGH

### BC-1.06.004: Vulnerability Collection -- Empty Batch

**Preconditions:** FetchVulnerabilities returns empty batch.
**Postconditions:** hasMore=false; no side effects.
**Evidence:** `TestCollectVulnerabilities_EmptyBatch` (collector_test.go:1179-1216)
**Confidence:** HIGH

### BC-1.06.005: Vulnerability Collection -- Pagination Indicator

**Preconditions:** Batch size equals limit.
**Postconditions:** hasMore=true.
**Evidence:** `TestCollectVulnerabilities_PaginationIndicator` (collector_test.go:1218-1265)
**Confidence:** HIGH

### BC-1.07.001: Device Collection Happy Path

**Preconditions:** FetchDevices returns non-empty batch.
**Postconditions:**
- All devices forwarded in order (device-1 then device-2)
- Cursor advances to batch.Last (Offset=2, DeviceUID="device-2")
- Version increments
**Evidence:** `TestCollectDevices_Success` (collector_test.go:1334-1412)
**Confidence:** HIGH

### BC-1.07.002: Device Collection -- Empty Batch

**Preconditions:** FetchDevices returns empty batch.
**Postconditions:** hasMore=false; no devices sent.
**Evidence:** `TestCollectDevices_EmptyBatch` (collector_test.go:1414-1447)
**Confidence:** HIGH

### BC-1.07.003: Device Collection -- Fetch Error

**Preconditions:** FetchDevices returns error.
**Postconditions:** Error propagated.
**Evidence:** `TestCollectDevices_FetchError` (collector_test.go:1449-1471)
**Confidence:** HIGH

### BC-1.07.004: Device Collection -- Sink Error

**Preconditions:** Fetch succeeds; SendDevice fails.
**Postconditions:** Error propagated.
**Evidence:** `TestCollectDevices_SinkError` (collector_test.go:1473-1513)
**Confidence:** HIGH

### BC-1.07.005: Device Collection -- hasMore=true When Batch Equals Limit

**Preconditions:** Batch of exactly 100 devices with limit=100.
**Postconditions:** hasMore=true; all 100 devices sent to sink.
**Evidence:** `TestCollectDevices_HasMore` (collector_test.go:1515-1567)
**Confidence:** HIGH

---

## Section 1.1: Forward Progress Contracts

### BC-1.10.001: DeviceAlertRelation Forward Progress -- 3-Tuple Ordering

**Preconditions:** Previous and next DeviceAlertRelationCursor provided.
**Postconditions:**
- Timestamp advances -> no error
- Same timestamp, AlertID advances -> no error
- Same timestamp + AlertID, DeviceUID advances -> no error
- None advance -> error
**Evidence:** `TestEnsureDeviceAlertRelationForwardProgress` (collector_test.go:540-602) -- 4 table-driven cases
**Confidence:** HIGH

### BC-1.10.002: Vulnerability Forward Progress -- Offset + ID Ordering

**Preconditions:** Previous and next VulnerabilityCursor provided.
**Postconditions:**
- Offset advances -> no error
- Same offset, VulnerabilityID advances -> no error
- Same cursor -> error
- ID backward -> error
- Offset backward (even with higher ID) -> error
**Evidence:** `TestEnsureVulnerabilityForwardProgress` (collector_test.go:1267-1332) -- 5 table-driven cases
**Confidence:** HIGH

### BC-1.10.003: Device Forward Progress -- Offset + UID Ordering

**Preconditions:** Previous and next DeviceCursor provided.
**Postconditions:**
- Offset advances -> no error
- Same offset, UID advances -> no error
- Same cursor -> error
- Backwards offset -> error
- Same offset, UID backward -> error
**Evidence:** `TestEnsureDeviceForwardProgress` (collector_test.go:1569-1636) -- 5 table-driven cases
**Confidence:** HIGH

---

## Section 1.2: Collection Orchestration

### BC-1.20.001: collectOnce Executes All 9 Sources Sequentially

**Preconditions:** All source states initialized.
**Postconditions:**
- Sources executed in order: alerts, events, auditLogs, deviceAlertRelations, deviceVulnerabilityRelations, servers, sites, devices, vulnerabilities
- Returns `hasMore = any(source.hasMore)` (logical OR)
- On first error from any source, immediately returns error (subsequent sources skipped)
**Evidence:** `collectOnce()` implementation (collector.go:804-851)
**Confidence:** HIGH (from code; no dedicated test but behavior is deterministic)

### BC-1.20.002: Run Loop -- Exponential Backoff

**Preconditions:** collectOnce() returns error.
**Postconditions:**
- retryCount increments
- If retryCount > maxRetries (and maxRetries > 0): return ErrCollectorRetriesExceeded
- Wait retryDelay (starts at baseDelay=2s, doubles up to maxDelay=30s)
- On next success: reset retryCount=0 and retryDelay=baseDelay
**Evidence:** `Run()` implementation (collector.go:585-653); referenced in broad sweep BC-006
**Confidence:** HIGH (from code)

### BC-1.20.003: Run Loop -- hasMore Skips Ticker Wait

**Preconditions:** collectOnce() returns (true, nil).
**Postconditions:** Loop continues immediately without waiting for ticker interval.
**Evidence:** `Run()` lines 642-645: `if hasMore { continue }`
**Confidence:** HIGH (from code)

### BC-1.20.004: State Initialization Order

**Preconditions:** Run() is called.
**Postconditions:** All 9 source states initialized in order: alerts, events, auditLogs, deviceAlertRelations, deviceVulnerabilityRelations, servers, sites, devices, vulnerabilities. First failure is fatal (returns error, no partial init).
**Evidence:** `initializeState()` (collector.go:655-694)
**Confidence:** HIGH (from code)

---

## Section 2: State Persistence Contracts

### BC-2.01.001: MemoryStore -- Load Before Save Returns ErrStateNotFound

**Preconditions:** MemoryStore is freshly created; no Save called.
**Postconditions:** Load() returns `(zero, ErrStateNotFound)` for all 9 source types.
**Evidence:** `TestMemoryStore_LoadBeforeSave` (store_test.go:130-139), `TestMemoryStore_EventState` (249-260), `TestMemoryStore_AuditLogState` (293-303), `TestMemoryStore_DeviceAlertRelationState` (344-354), `TestMemoryStore_DeviceVulnerabilityRelationState` (395-405), `TestMemoryStore_ServerState` (486-496), `TestMemoryStore_SiteState` (535-545)
**Confidence:** HIGH

### BC-2.01.002: MemoryStore -- Save Then Load Round-Trips State

**Preconditions:** Save(state, receipt) called.
**Postconditions:** Load() returns identical cursor, version, query fingerprint. Multiple saves overwrite (latest wins).
**Evidence:** `TestMemoryStore_SaveAndLoad` (store_test.go:142-194), `TestMemoryStore_MultipleAlertSaves` (196-247)
**Confidence:** HIGH

### BC-2.01.003: MemoryStore -- State Types Are Independent

**Preconditions:** AlertState and EventState saved separately.
**Postconditions:** Loading one does not affect the other. AuditLog (not saved) still returns ErrStateNotFound.
**Evidence:** `TestMemoryStore_IndependentStateTypes` (store_test.go:681-728)
**Confidence:** HIGH

### BC-2.01.004: MemoryStore -- Receipts Accumulate

**Preconditions:** Multiple Save() calls.
**Postconditions:** Receipts() returns all receipts in order of insertion.
**Evidence:** `TestMemoryStore_Receipts` (store_test.go:446-484)
**Confidence:** HIGH

### BC-2.02.001: QueryFingerprint -- Field Order Invariance

**Preconditions:** Same fields in different orders + same limit.
**Postconditions:** Hash is identical regardless of field order.
**Evidence:** `TestNewQueryFingerprint_FieldOrderingDoesNotAffectHash` (store_test.go:37-51)
**Confidence:** HIGH

### BC-2.02.002: QueryFingerprint -- Original Field Order Preserved

**Preconditions:** Fields passed in specific order.
**Postconditions:** `fp.Fields` preserves original order (not sorted).
**Evidence:** `TestNewQueryFingerprint_PreservesOriginalFieldOrder` (store_test.go:53-67)
**Confidence:** HIGH

### BC-2.02.003: QueryFingerprint -- Different Limits Produce Different Hashes

**Preconditions:** Same fields, different limits.
**Postconditions:** Hashes differ.
**Evidence:** `TestNewQueryFingerprint_DifferentLimitsProduceDifferentHashes` (store_test.go:69-79)
**Confidence:** HIGH

### BC-2.02.004: QueryFingerprint -- Different Fields Produce Different Hashes

**Preconditions:** Different field sets, same limit.
**Postconditions:** Hashes differ.
**Evidence:** `TestNewQueryFingerprint_DifferentFieldsProduceDifferentHashes` (store_test.go:81-90)
**Confidence:** HIGH

### BC-2.02.005: QueryFingerprint -- Negative Limit Normalized to Zero

**Preconditions:** Limit is negative.
**Postconditions:** `fp.Limit == 0`.
**Evidence:** `TestNewQueryFingerprint_NegativeLimitBecomesZero` (store_test.go:92-100)
**Confidence:** HIGH

### BC-2.02.006: QueryFingerprint -- Stable Across Calls

**Preconditions:** Same inputs.
**Postconditions:** Hash is identical across multiple calls.
**Evidence:** `TestNewQueryFingerprint_StableAcrossMultipleCalls` (store_test.go:116-128)
**Confidence:** HIGH

### BC-2.03.001: FileStore -- Atomic Write (Write-Sync-Rename)

**Preconditions:** State needs to be persisted.
**Postconditions:**
1. Temp file created in same directory (`.poller-state-*.tmp`)
2. JSON written to temp file
3. `Sync()` called (fsync)
4. `Rename()` atomically replaces state file
5. On any failure: temp file cleaned up
**Evidence:** `persist()` implementation (file_store.go:102-138)
**Confidence:** HIGH (from code; also tested via file_store_test.go)

### BC-2.03.002: FileStore -- Receipt Trimming

**Preconditions:** More than maxReceipts (default 100) receipts accumulated.
**Postconditions:** Only the most recent `maxReceipts` receipts retained (oldest trimmed).
**Evidence:** `trimReceipts()` generic function (file_store.go:141-146); applied in every Save method
**Confidence:** HIGH (from code)

### BC-2.03.003: FileStore -- Thread Safety via RWMutex

**Preconditions:** Concurrent Load/Save calls.
**Postconditions:** Load uses RLock; Save uses Lock. No data races.
**Evidence:** All FileStore methods use `fs.mu.RLock()/fs.mu.Lock()` (file_store.go)
**Confidence:** HIGH (from code)

---

## Section 3: Sink Contracts

### BC-3.01.001: HTTPSender Construction -- Validates Endpoint

**Preconditions:** Empty or invalid endpoint.
**Postconditions:** Returns ErrSinkConfigMissing (empty) or ErrSinkRequestBuild (invalid URL).
**Evidence:** `TestNewHTTPSender_EmptyEndpoint` (http_sender_test.go:56-69), `TestNewHTTPSender_InvalidEndpoint` (114-127), `TestNewHTTPSender_RelativeEndpoint` (129-142)
**Confidence:** HIGH

### BC-3.01.002: HTTPSender Construction -- Validates Credentials

**Preconditions:** Missing username, password, or both.
**Postconditions:** Returns ErrSinkConfigMissing.
**Evidence:** `TestNewHTTPSender_MissingCredentials` (http_sender_test.go:71-112) -- 3 sub-cases
**Confidence:** HIGH

### BC-3.01.003: HTTPSender Construction -- Default Timeout

**Preconditions:** Timeout is 0 or negative.
**Postconditions:** Timeout defaults to 15 seconds.
**Evidence:** `TestNewHTTPSender_DefaultTimeout` (http_sender_test.go:144-162)
**Confidence:** HIGH

### BC-3.01.004: HTTPSender Construction -- Trims Whitespace

**Preconditions:** Endpoint, username, password have leading/trailing whitespace.
**Postconditions:** All trimmed.
**Evidence:** `TestNewHTTPSender_TrimsWhitespace` (http_sender_test.go:164-185)
**Confidence:** HIGH

### BC-3.02.001: Send -- POST with Basic Auth and JSON Content-Type

**Preconditions:** Valid sender; alert payload.
**Postconditions:** Request uses POST method, Authorization header set (Basic auth), Content-Type = application/json.
**Evidence:** `TestSendAlert_Success` (http_sender_test.go:187-231)
**Confidence:** HIGH

### BC-3.02.002: Send -- HTTP Error (status >= 400) Returns ErrSinkDelivery

**Preconditions:** Server returns 400+.
**Postconditions:** Error wraps ErrSinkDelivery; includes status code and body preview (up to 2048 bytes).
**Evidence:** `TestSendAlert_HTTPError` (http_sender_test.go:233-263)
**Confidence:** HIGH

### BC-3.02.003: Send -- Context Cancellation

**Preconditions:** Context cancelled before request completes.
**Postconditions:** Error returned (no hang).
**Evidence:** `TestSendAlert_ContextCancellation` (http_sender_test.go:385-414)
**Confidence:** HIGH

### BC-3.03.001: Enrichment -- Record Type Tag

**Preconditions:** Any record sent.
**Postconditions:** Wrapped in EnrichedPayload with correct `record_type` string.
**Evidence:** `TestSendAlert_RecordTypeEnrichment` (http_sender_test.go:416-479)
**Confidence:** HIGH

### BC-3.03.002: Enrichment -- All 9 Record Types Correctly Tagged

**Preconditions:** Each of the 9 record types sent.
**Postconditions:**
- alert -> "alert"
- activity_event -> "activity_event"
- audit_log -> "audit_log"
- device_alert_relation -> "device_alert_relation"
- device_vulnerability_relation -> "device_vulnerability_relation"
- server -> "server"
- site -> "site"
- device -> "device"
- vulnerability -> "vulnerability"
**Evidence:** `TestEnrichPayload_MultipleRecordTypes` (http_sender_test.go:679-899) -- 9 table-driven sub-tests
**Confidence:** HIGH

### BC-3.03.003: Enrichment -- XMP Metadata Included

**Preconditions:** XMPConfig has site, cluster, node values.
**Postconditions:** EnrichedPayload.XMP populated with all three fields.
**Evidence:** `TestEnrichPayload_WithXMPMetadata` (http_sender_test.go:538-617)
**Confidence:** HIGH

### BC-3.03.004: Enrichment -- XMP Empty When Not Configured

**Preconditions:** XMPConfig is zero-valued.
**Postconditions:** XMP fields are empty strings (not omitted).
**Evidence:** `TestEnrichPayload_WithoutXMPMetadata` (http_sender_test.go:619-677)
**Confidence:** HIGH

### BC-3.03.005: Enrichment -- Original Data Preserved in `data` Field

**Preconditions:** Alert with specific fields sent.
**Postconditions:** EnrichedPayload.Data contains JSON-serialized original record, deserializable back to original type.
**Evidence:** `TestSendAlert_RecordTypeEnrichment` lines 464-478, `TestEnrichPayload_WithXMPMetadata` lines 602-616
**Confidence:** HIGH

### BC-3.04.001: OCSF Mapping -- Only for Alerts When Enabled

**Preconditions:** OCSF enabled; alert record sent.
**Postconditions:** Currently returns nil (TODO stub). Non-alert records always get nil OCSF.
**Evidence:** `mapOCSF()` implementation (http_sender.go:226-251) -- explicit `recordType != "alert"` guard + TODO comment
**Confidence:** HIGH (from code)

### BC-3.04.002: OCSF Mapping -- Panic Recovery

**Preconditions:** mapOCSF panics during execution.
**Postconditions:** Panic recovered; result is nil; error logged.
**Evidence:** `mapOCSF()` defer/recover block (http_sender.go:227-235)
**Confidence:** HIGH (from code)

---

## Section 4: Health Contracts

### BC-4.01.001: Server Starts Not Ready

**Preconditions:** NewServer() called.
**Postconditions:** `alive=true`, `ready=false`.
**Evidence:** `TestNewServer` (server_test.go:11-27)
**Confidence:** HIGH

### BC-4.01.002: Default Health Address

**Preconditions:** Empty address string.
**Postconditions:** Address defaults to `:7321`.
**Evidence:** `TestNewServer_DefaultAddr` (server_test.go:29-37)
**Confidence:** HIGH

### BC-4.02.001: Liveness -- Returns 200 When Alive

**Preconditions:** Server is alive (default).
**Postconditions:** /health returns 200 "ok".
**Evidence:** `TestHandleLiveness_Healthy` (server_test.go:39-56)
**Confidence:** HIGH

### BC-4.02.002: Liveness -- Returns 503 When Not Alive

**Preconditions:** `alive` set to false (via Shutdown).
**Postconditions:** /health returns 503 "unhealthy".
**Evidence:** `TestHandleLiveness_Unhealthy` (server_test.go:58-76)
**Confidence:** HIGH

### BC-4.02.003: Readiness -- Returns 200 When Ready

**Preconditions:** SetReady() called.
**Postconditions:** /ready returns 200 "ready".
**Evidence:** `TestHandleReadiness_Ready` (server_test.go:78-96)
**Confidence:** HIGH

### BC-4.02.004: Readiness -- Returns 503 When Not Ready

**Preconditions:** SetReady() not called.
**Postconditions:** /ready returns 503 "not ready".
**Evidence:** `TestHandleReadiness_NotReady` (server_test.go:98-116)
**Confidence:** HIGH

### BC-4.02.005: Readiness -- Returns 503 When Not Alive (Even If Ready)

**Preconditions:** SetReady() called, then alive=false.
**Postconditions:** /ready returns 503.
**Evidence:** `TestHandleReadiness_NotAlive` (server_test.go:118-133)
**Confidence:** HIGH

### BC-4.03.001: Shutdown Sets Alive=false

**Preconditions:** Server is running.
**Postconditions:** After Shutdown(), alive=false.
**Evidence:** `TestShutdown` (server_test.go:168-192)
**Confidence:** HIGH

### BC-4.03.002: /live Is Alias for /health

**Preconditions:** Server alive.
**Postconditions:** /live returns same response as /health (200 "ok").
**Evidence:** `TestHealthEndpoints_Integration` (server_test.go:194-248) -- tests /health, /live, /ready in sequence
**Confidence:** HIGH

---

## Section 6: OCSF Contracts

### BC-6.01.001: Severity Mapping -- Known Values

**Preconditions:** LoadConfig() succeeds; vendor severity string provided.
**Postconditions:**
- "Critical" -> 5
- "High" -> 4
- "Medium" -> 3
- "Low" -> 2
- "Info" -> 1
**Evidence:** `TestNormalizeSeverity` (severity_test.go:7-40) -- 5 named cases
**Confidence:** HIGH

### BC-6.01.002: Severity Mapping -- Unknown Falls Back to 0

**Preconditions:** Unknown severity string.
**Postconditions:** Returns 0 (FallbackSeverityID).
**Evidence:** `TestNormalizeSeverity` case "unknown value falls back to 0" and "empty string falls back to 0"
**Confidence:** HIGH

### BC-6.01.003: Severity Mapping -- Case Sensitive

**Preconditions:** Lowercase "critical" (not title case).
**Postconditions:** Returns 0 (not matched; falls back).
**Evidence:** `TestNormalizeSeverity_CaseSensitive` (severity_test.go:43-53)
**Confidence:** HIGH

### BC-6.01.004: Severity Mapping -- Empty Adjustments Are Passthrough

**Preconditions:** Adjustments list is empty (not nil).
**Postconditions:** Stage 3 is no-op; "High" still maps to 4.
**Evidence:** `TestNormalizeSeverity_EmptyAdjustmentsPassthrough` (severity_test.go:56-76)
**Confidence:** HIGH

---

## Section 5: Configuration Contracts

### BC-5.01.001: Config Loading -- API Key Required

**Preconditions:** Neither CLAROTY_API_KEY nor CLAROTY_API_KEY_FILE set.
**Postconditions:** LoadFromEnvironment returns error.
**Evidence:** Broad sweep BC-001; validation in config.go
**Confidence:** HIGH (from code inspection)

### BC-5.01.002: Config Loading -- File Variant Takes Precedence

**Preconditions:** Both CLAROTY_API_KEY and CLAROTY_API_KEY_FILE set.
**Postconditions:** Value from file is used.
**Evidence:** Config loading pattern (config.go); documented in broad sweep
**Confidence:** HIGH (from code inspection)

---

## Section 7: Transport Contracts

### BC-7.01.001: Transport Config Validation -- Non-negative Timeouts

**Preconditions:** Any timeout value is negative.
**Postconditions:** Validate() returns error with descriptive message.
**Evidence:** `transport.Config.Validate()` implementation (http.go:41-67) -- 8 validation checks
**Confidence:** HIGH (from code)

### BC-7.01.002: Transport Defaults -- Production-Ready Values

**Preconditions:** DefaultConfig() called.
**Postconditions:**
- MaxIdleConns=100, MaxIdleConnsPerHost=10, MaxConnsPerHost=20
- DialTimeout=10s, TLSHandshakeTimeout=10s, ResponseHeaderTimeout=30s
- MinTLSVersion=TLS 1.2
- HTTP/2 enabled (ForceAttemptHTTP2=true)
**Evidence:** `DefaultConfig()` (http.go:94-106)
**Confidence:** HIGH (from code)

---

## Coverage Gaps (Behaviors Without Test Coverage)

| Gap | Severity | Description |
|-----|----------|-------------|
| Server collection | HIGH | No `TestCollectServers*` tests found |
| Site collection | HIGH | No `TestCollectSites*` tests found |
| AuditLog forward progress | MEDIUM | No `TestEnsureAuditLogForwardProgress` test |
| Alert forward progress | MEDIUM | No `TestEnsureAlertForwardProgress` test |
| Event forward progress | MEDIUM | No `TestEnsureEventForwardProgress` test |
| DeviceVulnRelation forward progress | MEDIUM | No `TestEnsureDeviceVulnerabilityRelationForwardProgress` test |
| Run() retry loop | MEDIUM | No test for exponential backoff or maxRetries exceeded |
| initializeState fingerprint mismatch | MEDIUM | No test for ErrQueryFingerprintMismatch path |
| FileStore concurrent access | LOW | No concurrent test for FileStore RWMutex behavior |
| HTTP client request construction | LOW | HTTP client tests exist but focused on decode, not request construction |
| OCSF golden file tests | LOW | Tests exist (golden_file_test.go) but not analyzed in this pass |

---

## Delta Summary
- New items added: 55 behavioral contracts across 7 sections
- Existing items refined: Broad sweep BC-001 through BC-010 decomposed into granular per-source contracts with test evidence
- Remaining gaps: Server/site collection untested, 4 forward progress functions untested, Run() retry loop untested

## Novelty Assessment
Novelty: **SUBSTANTIVE**
This round extracted 55 specific behavioral contracts with test evidence, identified 11 coverage gaps (including 2 high-severity gaps where entire data sources lack test coverage), and documented the unique AuditLog offset+1 behavior. The per-source decomposition reveals that servers and sites have zero test coverage for their collection paths, which changes the confidence model.

## Convergence Declaration
Another round needed -- should verify server/site collection test coverage gap is real (not in a file I missed), investigate OCSF golden file tests, check http_client_test.go for API-level contracts, and audit for any forward progress test coverage I missed.

## State Checkpoint
```yaml
pass: 3
round: 1
status: complete
files_scanned: 17
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
```
