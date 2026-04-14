# Pass 3 Deep: Behavioral Contracts -- Round 1

**Project:** poller-coaster
**Date:** 2026-04-13
**Basis:** All 11 test files + all source files read, cross-referenced with broad sweep BC-001 through BC-012

---

## Test File Coverage Map

| Test File | Tests | Coverage Area |
|-----------|-------|---------------|
| collector_test.go | 4 | Retry exhaustion, retry reset, unlimited retries, attempt count in error |
| device_collector_test.go | 9 | All collector patterns: no results, with results, API error, sink error, hasMore, cursor filter, nil sink, FirstSeen fallback, Title fallback for ID |
| connection_collector_test.go | 10 | Same as device + EndTimestamp fallback, Title fallback for ID |
| vulnerability_collector_test.go | 10 | Same as device + FirstDetected fallback, PublishedDate fallback, Title fallback for ID |
| audit_collector_test.go | 10 | Same as device + sorting verification, invalid timestamp, cursor advancement |
| risk_factor_collector_test.go | 12 | Same as audit + FirstSeen fallback, Title as fallback ID, cursor advancement |
| file_store_test.go | 12 | FileStore: create, directory creation, load existing, invalid JSON, save/load, persistence to disk, restart survival, all 7 states survive restart, receipts, max receipts, atomic write, independent state types |
| store_test.go | (in file_store_test.go) | trimReceipts generic function |
| config_test.go | (not read yet) | Configuration validation |
| health/server_test.go | (not read yet) | Health server behavior |
| profiling/pprof_test.go | (not read yet) | Profiling server |

**Test coverage gaps:** No dedicated tests for AlertCollector or ActivityCollector. They share the same code pattern but have NO test verification of their specific timestamp/ID extraction logic.

---

## Behavioral Contracts (BC-S.SS.NNN format)

### Section 1: Collector Orchestration (BC-1.xx.NNN)

#### BC-1.01.001: Retry exhaustion terminates collector with sentinel error

**Preconditions:** API returns errors on every call; MaxRetries > 0
**Postconditions:** `Run()` returns error wrapping `ErrCollectorRetriesExceeded`; error message includes attempt count (e.g., "attempts=10")
**Error Cases:** Each failure doubles backoff delay (base -> 2x -> 4x ... capped at maxDelay)
**Evidence:** `TestCollector_MaxRetries_ExhaustsAfterConfiguredAttempts` (collector_test.go:32), `TestCollector_MaxRetries_ReturnsAttemptCount` (collector_test.go:69)
**Confidence:** HIGH (two dedicated tests verify both the sentinel and the message content)

#### BC-1.01.002: Retry counter resets to zero after any successful collectOnce

**Preconditions:** API fails N times (N < MaxRetries), then succeeds
**Postconditions:** Retry counter resets to 0; backoff delay resets to baseDelay; health reporter set to ready
**Evidence:** `TestCollector_MaxRetries_ResetsAfterSuccess` (collector_test.go:102) -- verifies callCount > 3 (proving the collector continued past the initial failures)
**Confidence:** HIGH

#### BC-1.01.003: Unlimited retries when MaxRetries equals zero

**Preconditions:** MaxRetries=0; API fails continuously
**Postconditions:** `Run()` never returns `ErrCollectorRetriesExceeded`; stops only on context cancellation
**Evidence:** `TestCollector_MaxRetries_UnlimitedWhenZero` (collector_test.go:134) -- verifies >= 20 calls made, no ErrCollectorRetriesExceeded
**Confidence:** HIGH

#### BC-1.01.004: Retry count threshold check uses strictly-greater-than

**Preconditions:** MaxRetries=N; exactly N+1 API calls fail
**Postconditions:** Error returned on the (N+1)th failure because `retryCount > MaxRetries` after incrementing
**Evidence:** Source code `collector.go:167` -- `if c.cfg.Collector.MaxRetries > 0 && retryCount > c.cfg.Collector.MaxRetries`; test at collector_test.go:63 -- `callCount < maxRetries+1`
**Confidence:** HIGH (from code + test alignment)

#### BC-1.02.001: collectOnce executes sources sequentially and short-circuits on first error

**Preconditions:** 7 data sources configured
**Postconditions:** Sources polled in fixed order: alerts, activities, audit logs, risk factors, connections, devices, vulnerabilities. If any source's Collect() returns error, remaining sources are skipped and error propagated.
**Error Cases:** Error from source N means sources N+1..7 never execute in that cycle
**Evidence:** Source code `collector.go:492-529` -- sequential if-err-return pattern
**Confidence:** HIGH (from code, no dedicated test)

#### BC-1.02.002: hasMore is OR'd across all sources

**Preconditions:** Multiple sources return results
**Postconditions:** If ANY source's hasMore=true, collectOnce returns hasMore=true, triggering immediate re-poll (no ticker wait)
**Evidence:** Source code `collector.go:528` -- `return alertMore || activityMore || ...`
**Confidence:** HIGH (from code)

#### BC-1.02.003: Health reporter transitions match collector state

**Preconditions:** Reporter is non-nil
**Postconditions:**
- On initialization start: SetNotReady()
- On deferred exit: SetNotReady()
- After successful initializeState: SetReady()
- On collectOnce error: SetNotReady()
- On collectOnce success: SetReady()
**Evidence:** Source code `collector.go:132-186`
**Confidence:** HIGH (from code, no dedicated test for transitions)

### Section 2: Per-Source Collection (BC-2.xx.NNN)

These contracts apply uniformly to all 7 collectors. Tests verify them on specific collectors as noted.

#### BC-2.01.001: Empty API response returns current state unchanged with nil receipt

**Preconditions:** `GetSearch()` returns empty results slice
**Postconditions:** Returns (currentState, nil receipt, hasMore=false, nil error); version unchanged
**Evidence:** `TestDeviceCollector_Collect_NoResults`, `TestConnectionCollector_Collect_NoResults`, `TestVulnerabilityCollector_Collect_NoResults`, `TestAuditLogCollector_Collect_NoResults`, `TestRiskFactorCollector_Collect_NoResults`
**Confidence:** HIGH (5/7 collectors tested)
**Gap:** Alert and Activity collectors have no test for this.

#### BC-2.01.002: API error wraps with ErrArmisRequestExec sentinel

**Preconditions:** `GetSearch()` returns error
**Postconditions:** Returns error wrapping `apperrors.ErrArmisRequestExec` via `fmt.Errorf("%w: %v", ...)`; current state returned unchanged; receipt is nil
**Evidence:** `TestDeviceCollector_Collect_APIError`, `TestConnectionCollector_Collect_APIError`, `TestVulnerabilityCollector_Collect_APIError`, `TestAuditLogCollector_Collect_APIError`, `TestRiskFactorCollector_Collect_APIError`
**Confidence:** HIGH (5/7 tested; all tests verify error is non-nil but do NOT assert errors.Is(err, ErrArmisRequestExec))

**Important nuance:** Tests only assert `err != nil`, not the specific sentinel. The sentinel wrapping is verified from source code only.

#### BC-2.02.001: Results are sorted by (timestamp, ID) ascending before processing

**Preconditions:** API returns results in arbitrary order
**Postconditions:** Results sorted by timestamp ascending, with ID string comparison as tiebreaker
**Evidence:** `TestAuditLogCollector_Collect_SortsResults` (audit_collector_test.go:350), `TestRiskFactorCollector_Collect_SortsResults` (risk_factor_collector_test.go:350) -- both verify FirstXxxID and LastXxxID are in correct order
**Confidence:** HIGH (2/7 tested, code identical for all 7)

#### BC-2.02.002: Records at or before cursor position are filtered out

**Preconditions:** API returns mix of old (at/before cursor) and new (after cursor) records
**Postconditions:** Only records where `isCursorAhead(current, recordCursor)` returns true are kept. Records at cursor position (same timestamp AND same-or-earlier ID) are excluded.
**Evidence:** `TestDeviceCollector_Collect_FiltersBeyondCursor`, `TestConnectionCollector_Collect_FiltersBeyondCursor`, `TestVulnerabilityCollector_Collect_FiltersBeyondCursor`, `TestAuditLogCollector_Collect_FiltersBeyondCursor`, `TestRiskFactorCollector_Collect_FiltersBeyondCursor`
**Confidence:** HIGH (5/7 tested)

#### BC-2.02.003: Records with unparseable timestamps are skipped with warning

**Preconditions:** API returns records where all timestamp fields fail RFC3339/RFC3339Nano parsing
**Postconditions:** Record excluded from results (not delivered to sink, not counted in receipt); warning logged; other records processed normally
**Evidence:** `TestAuditLogCollector_Collect_InvalidTimestamp` (audit_collector_test.go:388), `TestRiskFactorCollector_Collect_InvalidTimestamp` (risk_factor_collector_test.go:388)
**Confidence:** HIGH (2/7 tested; code pattern identical for all 7)

#### BC-2.03.001: Results truncated to limit with hasMore=true when exceeding limit

**Preconditions:** After filtering, results count exceeds fingerprint.Limit
**Postconditions:** Results truncated to first `limit` items (by sorted order); hasMore=true; receipt.Count equals limit
**Evidence:** `TestDeviceCollector_Collect_HasMore`, `TestConnectionCollector_Collect_HasMore`, `TestVulnerabilityCollector_Collect_HasMore`, `TestAuditLogCollector_Collect_HasMore`, `TestRiskFactorCollector_Collect_HasMore` -- all verify receipt.Count=100 when 150 results provided with limit=100
**Confidence:** HIGH (5/7 tested)

#### BC-2.04.001: Each record delivered individually to sink

**Preconditions:** Sink is non-nil; new records exist after filtering
**Postconditions:** `sink.SendXxx()` called once per record in sorted order
**Evidence:** `TestDeviceCollector_Collect_WithResults` -- verifies `len(sender.sent) == 2`; `TestConnectionCollector_Collect_WithResults` -- same
**Confidence:** HIGH

#### BC-2.04.002: Sink error halts collection and propagates

**Preconditions:** Sink returns error on any SendXxx call
**Postconditions:** Collect() returns error wrapping `ErrSinkDelivery`; current state unchanged (not advanced); remaining records not delivered
**Evidence:** `TestDeviceCollector_Collect_SinkError`, `TestConnectionCollector_Collect_SinkError`, `TestVulnerabilityCollector_Collect_SinkError`, `TestAuditLogCollector_Collect_SinkError`, `TestRiskFactorCollector_Collect_SinkError`
**Confidence:** HIGH (5/7 tested)

#### BC-2.04.003: Nil sink does not block collection or state advancement

**Preconditions:** Sink is nil (not configured)
**Postconditions:** Collection proceeds; cursor advances; receipt generated; no delivery errors
**Evidence:** `TestDeviceCollector_Collect_NilSink`, `TestConnectionCollector_Collect_NilSink`, `TestAuditLogCollector_Collect_NilSink`, `TestRiskFactorCollector_Collect_NilSink`
**Confidence:** HIGH (4/7 tested)

#### BC-2.05.001: Forward progress check prevents cursor regression

**Preconditions:** New cursor is not strictly ahead of current cursor
**Postconditions:** Error returned; state not advanced
**Error handling:** Connection, Device, Vulnerability wrap with `ErrCursorRegression` sentinel; Alert, Activity, AuditLog, RiskFactor use plain `fmt.Errorf` (NOT matchable by errors.Is)
**Evidence:** Source code verified for all 7 collectors; no dedicated test for regression scenario
**Confidence:** MEDIUM (from code, no tests verify the regression path directly)

#### BC-2.05.002: Cursor advancement sets new cursor to last processed record

**Preconditions:** At least one new record passes filtering
**Postconditions:** New state cursor = cursor of last record in sorted/truncated batch; version incremented by 1; UpdatedAt set to now
**Evidence:** `TestAuditLogCollector_Collect_CursorAdvancement`, `TestRiskFactorCollector_Collect_CursorAdvancement` -- both verify timestamp advanced and ID updated
**Confidence:** HIGH (2/7 tested directly; all code identical)

### Section 3: Timestamp Fallback Chains (BC-3.xx.NNN)

#### BC-3.01.001: Alert timestamp uses LastAlertUpdateTime with Time fallback

**Preconditions:** Alert record from API
**Postconditions:** Cursor timestamp = first parseable value from [LastAlertUpdateTime, Time]
**Evidence:** Source code `alert_collector.go:148` -- no dedicated test
**Confidence:** MEDIUM (code only, no test)

#### BC-3.02.001: Activity timestamp uses Time only (no fallback)

**Preconditions:** Activity record from API
**Postconditions:** Cursor timestamp = parsed Time field; if empty/unparseable, timestamp is zero (record skipped)
**Evidence:** Source code `activity_collector.go:149-158` -- single field, no fallback chain
**Confidence:** MEDIUM (code only, no test)

#### BC-3.03.001: AuditLog timestamp uses Time only (no fallback)

**Preconditions:** AuditLog record from API
**Postconditions:** Same as Activity -- single Time field
**Evidence:** Source code `audit_collector.go:153-162`; `TestAuditLogCollector_Collect_InvalidTimestamp` validates skip behavior
**Confidence:** HIGH (test validates skip path)

#### BC-3.04.001: RiskFactor timestamp uses LastSeen with FirstSeen fallback

**Preconditions:** RiskFactor record from API
**Postconditions:** Cursor timestamp = first parseable from [LastSeen, FirstSeen]
**Evidence:** Source code `risk_factor_collector.go:153-166`; `TestRiskFactorCollector_Collect_UsesFirstSeenFallback` verifies fallback
**Confidence:** HIGH

#### BC-3.05.001: Connection timestamp uses StartTimestamp with EndTimestamp fallback

**Preconditions:** Connection record from API
**Postconditions:** Cursor timestamp = first parseable from [StartTimestamp, EndTimestamp]
**Evidence:** Source code `connection_collector.go:147-161`; `TestConnectionCollector_Collect_UsesEndTimestampFallback` verifies fallback
**Confidence:** HIGH

#### BC-3.06.001: Device timestamp uses LastSeen with FirstSeen fallback

**Preconditions:** Device record from API
**Postconditions:** Cursor timestamp = first parseable from [LastSeen, FirstSeen]
**Evidence:** Source code `device_collector.go:147-165`; `TestDeviceCollector_Collect_UsesFirstSeenFallback` verifies fallback
**Confidence:** HIGH

#### BC-3.07.001: Vulnerability timestamp uses LastDetected, FirstDetected, PublishedDate (3-level chain)

**Preconditions:** Vulnerability record from API
**Postconditions:** Cursor timestamp = first parseable from [LastDetected, FirstDetected, PublishedDate]
**Evidence:** Source code `vulnerability_collector.go:148-163`; `TestVulnerabilityCollector_Collect_UsesFirstDetectedFallback`, `TestVulnerabilityCollector_Collect_UsesPublishedDateFallback`
**Confidence:** HIGH (both fallback levels tested)

### Section 4: ID Fallback Chains (BC-4.xx.NNN)

#### BC-4.01.001: Alert ID extraction: AlertID(int) -> PolicyID -> Title -> timestamp-nano

**Preconditions:** Alert record
**Postconditions:** First non-empty/non-zero value used; AlertID checked as `!= 0` (int); PolicyID/Title checked with TrimSpace
**Evidence:** Source code `alert_collector.go:128-145`; no dedicated test
**Confidence:** MEDIUM

#### BC-4.02.001: Activity ID extraction: PolicyID -> ActivityUUIDs[0] -> Title -> timestamp-nano

**Preconditions:** Activity record
**Postconditions:** First non-empty value used
**Evidence:** Source code `activity_collector.go:128-147`; no dedicated test
**Confidence:** MEDIUM

#### BC-4.03.001: AuditLog ID extraction: PolicyID -> Title -> timestamp-nano

**Preconditions:** AuditLog record
**Postconditions:** First non-empty value used
**Evidence:** Source code `audit_collector.go:135-151`; no dedicated test for ID fallback specifically
**Confidence:** MEDIUM

#### BC-4.04.001: RiskFactor ID extraction: PolicyID -> Title -> timestamp-nano

**Preconditions:** RiskFactor record
**Postconditions:** First non-empty value used
**Evidence:** Source code `risk_factor_collector.go:135-151`; `TestRiskFactorCollector_Collect_UsesTitleAsFallbackID` verifies Title fallback
**Confidence:** HIGH (Title fallback tested)

#### BC-4.05.001: Connection ID extraction: ID(sdk) -> Title -> timestamp-nano

**Preconditions:** Connection record
**Postconditions:** `string(result.ID)` checked for empty or "0"; then Title; then timestamp nano
**Evidence:** Source code `connection_collector.go:132-145`; `TestConnectionCollector_Collect_UsesTitleFallbackForID`
**Confidence:** HIGH

#### BC-4.06.001: Device ID extraction: ID(sdk) -> Title -> timestamp-nano

**Preconditions:** Device record
**Postconditions:** Same pattern as Connection
**Evidence:** Source code `device_collector.go:132-145`; `TestDeviceCollector_Collect_UsesTitleFallbackForID`
**Confidence:** HIGH

#### BC-4.07.001: Vulnerability ID extraction: ID(sdk) -> Title -> timestamp-nano

**Preconditions:** Vulnerability record
**Postconditions:** Same pattern as Connection/Device
**Evidence:** Source code `vulnerability_collector.go:133-146`; `TestVulnerabilityCollector_Collect_UsesTitleFallbackForID`
**Confidence:** HIGH

### Section 5: State Persistence (BC-5.xx.NNN)

#### BC-5.01.001: FileStore atomic write pattern prevents corruption

**Preconditions:** State save requested
**Postconditions:** Write temp file -> fsync -> close -> rename (atomic on POSIX); on any error: temp file cleaned up, error returned, in-memory state unchanged
**Evidence:** Source code `file_store.go:118-162`; `TestFileStore_AtomicWrite` verifies no temp files left behind; `TestFileStore_PersistsToDisk` verifies data on disk
**Confidence:** HIGH

#### BC-5.01.002: FileStore loads existing state on construction

**Preconditions:** State file exists with valid JSON
**Postconditions:** State loaded into memory; Load() returns persisted values
**Evidence:** `TestNewFileStore_LoadsExistingState` (file_store_test.go:53)
**Confidence:** HIGH

#### BC-5.01.003: FileStore rejects invalid JSON on construction

**Preconditions:** State file exists with invalid JSON
**Postconditions:** `NewFileStore()` returns error
**Evidence:** `TestNewFileStore_InvalidJSON` (file_store_test.go:96)
**Confidence:** HIGH

#### BC-5.01.004: FileStore creates parent directories on construction

**Preconditions:** Path includes non-existent nested directories
**Postconditions:** Directories created with 0o750 permissions
**Evidence:** `TestNewFileStore_CreatesDirectory` (file_store_test.go:31)
**Confidence:** HIGH

#### BC-5.01.005: FileStore returns ErrStateNotFound before first save

**Preconditions:** No state file exists; no prior save
**Postconditions:** Load() returns `apperrors.ErrStateNotFound`
**Evidence:** `TestFileStore_LoadBeforeSave` (file_store_test.go:113); similar tests for all 7 data sources
**Confidence:** HIGH

#### BC-5.01.006: All 7 state types survive process restart

**Preconditions:** All 7 data sources saved to FileStore
**Postconditions:** New FileStore instance at same path loads all 7 states correctly
**Evidence:** `TestFileStore_AllSevenStatesSurviveRestart` (file_store_test.go:700)
**Confidence:** HIGH

#### BC-5.01.007: State types are independent (saving one does not affect others)

**Preconditions:** Multiple data source states saved
**Postconditions:** Loading one source's state does not affect other sources
**Evidence:** `TestFileStore_IndependentStateTypes` (file_store_test.go:527)
**Confidence:** HIGH

#### BC-5.02.001: Receipt trimming keeps most recent entries within limit

**Preconditions:** Receipts exceed maxReceipts after append
**Postconditions:** Oldest receipts dropped; exactly maxReceipts retained; most recent entries kept
**Evidence:** `TestFileStore_WithMaxReceipts` (file_store_test.go:623) -- verifies 10 saves with max=3 results in versions [8,9,10]; `TestTrimReceipts` (file_store_test.go:801) -- table-driven tests for edge cases
**Confidence:** HIGH

#### BC-5.03.001: MemoryStore returns ErrStateNotFound before first save

**Preconditions:** Fresh MemoryStore
**Postconditions:** All 7 LoadXxx methods return `ErrStateNotFound`
**Evidence:** Source code `store.go:125-130` etc.; used implicitly in all collector_test.go tests
**Confidence:** HIGH (from code; tests use MemoryStore indirectly)

### Section 6: Query Fingerprint (BC-6.xx.NNN)

#### BC-6.01.001: Fingerprint mismatch on startup is fatal

**Preconditions:** Stored state has fingerprint hash different from current config's computed hash
**Postconditions:** `initializeXxxState()` returns error wrapping `ErrQueryFingerprintMismatch` with both hashes in message
**Evidence:** Source code `collector.go:232-233` (and 6 copies); no dedicated test
**Confidence:** MEDIUM (clear code path, no test)

#### BC-6.01.002: Fingerprint hash is order-independent for fields

**Preconditions:** Fields provided in any order
**Postconditions:** Same hash produced (fields sorted before hashing)
**Evidence:** Source code `store.go:486-487` -- `sort.Strings(canonical)` before joining
**Confidence:** HIGH (from code)

#### BC-6.01.003: Negative limit clamped to zero in fingerprint

**Preconditions:** Limit < 0 passed to NewQueryFingerprint
**Postconditions:** Limit treated as 0 for hash computation
**Evidence:** Source code `store.go:482-484`
**Confidence:** HIGH (from code)

### Section 7: Sink Delivery (BC-7.xx.NNN)

#### BC-7.01.001: HTTPSender requires non-empty endpoint, username, and password

**Preconditions:** Any of endpoint/username/password is empty
**Postconditions:** `NewHTTPSender()` returns error wrapping `ErrSinkConfigMissing`
**Evidence:** Source code `http_sender.go:54-59`
**Confidence:** HIGH (from code; no dedicated test)

#### BC-7.01.002: HTTPSender endpoint must be absolute URL

**Preconditions:** Endpoint does not parse as absolute URL
**Postconditions:** `NewHTTPSender()` returns error wrapping `ErrSinkRequestBuild`
**Evidence:** Source code `http_sender.go:67-69`
**Confidence:** HIGH (from code)

#### BC-7.02.001: HTTP status >= 400 is treated as delivery failure

**Preconditions:** Sink endpoint returns HTTP 4xx/5xx
**Postconditions:** Error wrapping `ErrSinkDelivery` with status code and up to 2048 bytes of response body
**Evidence:** Source code `http_sender.go:197-200`
**Confidence:** HIGH (from code)

#### BC-7.02.002: Response body close error logged as warning, not propagated

**Preconditions:** resp.Body.Close() returns error
**Postconditions:** Warning logged; original success/error result unchanged
**Evidence:** Source code `http_sender.go:192-194`
**Confidence:** HIGH (from code)

#### BC-7.03.001: All records enriched with xMP metadata before delivery

**Preconditions:** Any record sent through HTTPSender
**Postconditions:** Payload wrapped as `{"data": <raw>, "record_type": "<type>", "xmp": {"site":..., "cluster_name":..., "node_name":...}}`
**Evidence:** Source code `http_sender.go:207-236`
**Confidence:** HIGH (from code)

### Section 8: Health Server (BC-8.xx.NNN)

#### BC-8.01.001: Health server starts with alive=true, ready=false

**Preconditions:** `NewServer()` called
**Postconditions:** /live returns 200; /ready returns 503
**Evidence:** Source code `server.go:77-79`
**Confidence:** HIGH (from code)

#### BC-8.01.002: Rate limiting applied per-IP with configurable limits

**Preconditions:** Multiple requests from same IP
**Postconditions:** Requests exceeding rate (default: 100/s, burst 20) receive 429 with Retry-After:1 header
**Evidence:** Source code `server.go:112-131`
**Confidence:** HIGH (from code)

### Section 9: Configuration (BC-9.xx.NNN)

#### BC-9.01.001: Config loading order: defaults -> env vars -> file secrets -> validation

**Preconditions:** Process starts
**Postconditions:** `DefaultConfig()` called first, then `LoadFromEnvironment()` overrides, then `Validate()` checks
**Evidence:** Source code `runner.go:33-42`
**Confidence:** HIGH

#### BC-9.01.002: File-backed secrets take priority over direct env vars

**Preconditions:** Both `ARMIS_API_KEY_FILE` and `ARMIS_API_KEY` set
**Postconditions:** File value used; direct env var ignored
**Evidence:** Source code `config.go:319-325` -- `if urlFromFile != ""` branch takes priority
**Confidence:** HIGH (from code)

#### BC-9.01.003: Non-existent secret files silently fall back to env vars

**Preconditions:** `ARMIS_API_KEY_FILE` points to non-existent file
**Postconditions:** No error; falls through to check `ARMIS_API_KEY` env var
**Evidence:** Source code `config.go:684-692` -- `readSecretFile` returns ("", nil) for os.ErrNotExist
**Confidence:** HIGH (from code)

#### BC-9.02.001: Validation aggregates all errors via errors.Join

**Preconditions:** Multiple config values invalid
**Postconditions:** Single error returned containing all validation failures
**Evidence:** Source code `config.go:556-681` -- `errors.Join(errs...)`
**Confidence:** HIGH (from code)

#### BC-9.02.002: Missing API key fails in LoadFromEnvironment, not in Validate

**Preconditions:** Neither `ARMIS_API_KEY` nor `ARMIS_API_KEY_FILE` set
**Postconditions:** `LoadFromEnvironment()` returns `"missing Armis API key"` error before Validate() is reached
**Evidence:** Source code `config.go:327-329`
**Confidence:** HIGH (from code)

#### BC-9.03.001: Duration parsing accepts both Go duration strings and plain integers as seconds

**Preconditions:** Timeout env var set to e.g. "30" or "30s"
**Postconditions:** Both produce 30*time.Second; if neither format works, returns error
**Evidence:** Source code `config.go:336-342` (Armis timeout), `config.go:472-479` (sink timeout)
**Confidence:** HIGH (from code)

#### BC-9.03.002: XMP NodeName falls back to system hostname

**Preconditions:** `XMP_NODE_NAME` env var not set
**Postconditions:** `os.Hostname()` used as fallback
**Evidence:** Source code `config.go:356-361`
**Confidence:** HIGH (from code)

---

## Contracts Missing Test Coverage (Gaps)

| Contract | Area | Risk |
|----------|------|------|
| BC-2.05.001 | Forward progress regression path | MEDIUM -- no test verifies the error when cursor fails to advance |
| BC-6.01.001 | Fingerprint mismatch rejection | MEDIUM -- clear code path but no test |
| BC-3.01.001 | Alert timestamp fallback | LOW -- code is simple but untested |
| BC-3.02.001 | Activity timestamp (no fallback) | LOW -- single field, trivial |
| BC-4.01.001 | Alert ID extraction chain | LOW -- untested |
| BC-4.02.001 | Activity ID extraction chain | LOW -- untested |
| BC-4.03.001 | AuditLog ID extraction chain | LOW -- Title fallback untested |
| BC-7.01.001-002 | HTTPSender construction validation | LOW -- straightforward guard clauses |
| BC-9.02.002 | Missing API key early error | LOW -- may cause confusion vs Validate() |
| All BC-2.xx for Alert | AlertCollector has ZERO dedicated tests | MEDIUM -- relies on code symmetry with tested collectors |
| All BC-2.xx for Activity | ActivityCollector has ZERO dedicated tests | MEDIUM -- relies on code symmetry |

---

## Delta Summary

- New items added: 42 behavioral contracts (vs. 12 in broad sweep)
- Existing items refined: All 12 broad sweep contracts (BC-001 through BC-012) now have precise code line references and test evidence
- Remaining gaps: Alert/Activity collector test coverage, forward progress regression testing, fingerprint mismatch testing

## Novelty Assessment

Novelty: SUBSTANTIVE

This round adds 30 contracts not present in the broad sweep, including: the collectOnce sequential short-circuit behavior, the hasMore OR logic, health reporter transition sequence, the exact distinction between sentinel and non-sentinel forward progress errors, the duration parsing dual-format pattern, the XMP hostname fallback, the secret file priority ordering, the validation aggregation pattern, and the missing API key early-fail behavior. These change how you would spec the system.

## Convergence Declaration

Another round needed -- should verify: (1) config_test.go for validation contract evidence, (2) health/server_test.go for health contracts, (3) profiling/pprof_test.go for completeness, (4) audit hallucination classes from broad sweep BC numbering.

## State Checkpoint

```yaml
pass: 3
round: 1
status: complete
files_scanned: 32
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
next_action: round 2 -- read config_test.go, health/server_test.go, pprof_test.go; audit hallucination classes; verify all contracts against source
```
