# Pass 2 Deep: Domain Model -- Round 1

**Project:** poller-coaster
**Date:** 2026-04-13
**Basis:** Full source code read of all 33 Go files + broad sweep verification

---

## Sub-pass 2a: Structural Extraction

### Entity Catalog (verified from source)

The domain has **no custom entity types for Armis records**. All 7 data sources flow through the SDK's `centrix.SearchResult` flat struct. The domain's own types are entirely about **polling infrastructure**: cursors, state, receipts, and delivery wrappers.

#### Infrastructure Entities

| Entity | Package | Identity Fields | Key Properties |
|--------|---------|----------------|----------------|
| Collector | collector | singleton (one per process) | cfg, client, store, sink, reporter, 7x sub-collectors, 7x fingerprints, 7x states, interval |
| AlertCollector | collector | (query, fingerprint) | client, sink, logger, query string, fingerprint |
| ActivityCollector | collector | (query, fingerprint) | client, sink, logger, query string, fingerprint |
| AuditLogCollector | collector | (query, fingerprint) | client, sink, logger, query string, fingerprint |
| RiskFactorCollector | collector | (query, fingerprint) | client, sink, logger, query string, fingerprint |
| ConnectionCollector | collector | (query, fingerprint) | client, sink, logger, query string, fingerprint |
| DeviceCollector | collector | (query, fingerprint) | client, sink, logger, query string, fingerprint |
| VulnerabilityCollector | collector | (query, fingerprint) | client, sink, logger, query string, fingerprint |
| HTTPSender | sink | (endpoint, username, password) | client, logger, xmpConfig |
| HTTPClient (Armis) | armis | (baseURL, apiKey) | inner centrix.Client, logger |
| FileStore | state | (path) | mu, state fileState, maxReceipts, logger |
| MemoryStore | state | singleton | mu, 7x (state, receipt, initialized) triples |
| Server (Health) | health | (addr) | ready, alive atomics, httpServer, rateLimitConfig, limiters map |
| Config | config | n/a (value) | Armis, Collector, Sink, Logging, XMP, State sub-configs |

#### Value Objects (verified field-by-field from `state/store.go`)

| Value Object | Fields | Used By | Notes |
|-------------|--------|---------|-------|
| AlertCursor | Timestamp (time.Time), AlertID (string) | AlertPollState, AlertBatchReceipt | AlertID is string despite SDK's int AlertID |
| ActivityCursor | Timestamp, ActivityID (string) | ActivityPollState, ActivityBatchReceipt | |
| AuditLogCursor | Timestamp, AuditID (string) | AuditLogPollState, AuditLogBatchReceipt | Named "AuditID" not "AuditLogID" |
| RiskFactorCursor | Timestamp, RiskFactorID (string) | RiskFactorPollState, RiskFactorBatchReceipt | |
| ConnectionCursor | Timestamp, ConnectionID (string) | ConnectionPollState, ConnectionBatchReceipt | |
| DeviceCursor | Timestamp, DeviceID (string) | DevicePollState, DeviceBatchReceipt | |
| VulnerabilityCursor | Timestamp, VulnerabilityID (string) | VulnerabilityPollState, VulnerabilityBatchReceipt | |
| QueryFingerprint | Hash (string), Fields ([]string), Limit (int) | All PollState types | SHA-256 of sorted fields + limit |
| XMPMetadata | Site, ClusterName, NodeName (all string, omitempty JSON) | EnrichedPayload | |
| EnrichedPayload | Data (json.RawMessage), RecordType (string), XMP (XMPMetadata) | HTTPSender | Wire format for sink |
| RateLimitConfig | RequestsPerSecond (int), Burst (int) | health.Server | Default: 100 req/s, burst 20 |

#### Aggregate: PollState + BatchReceipt (7 copies)

Each data source has an identical aggregate structure (verified from source):

| Type | Fields |
|------|--------|
| XxxPollState | Cursor (XxxCursor), Query (QueryFingerprint), UpdatedAt (time.Time), Version (uint64) |
| XxxBatchReceipt | Version (uint64), RequestHash (string), Count (int), FirstXxxID (string), LastXxxID (string), FetchedAt (time.Time), CursorApplied (XxxCursor) |

The Xxx placeholder expands to: Alert, Activity, AuditLog, RiskFactor, Connection, Device, Vulnerability.

**CORRECTION from broad sweep:** The broad sweep listed `AlertBatchReceipt.FirstAlertID` field names accurately. Verified all 7 receipt types follow the same pattern with type-specific ID field names.

#### Persistence Aggregate: fileState

The `fileState` struct (file_store.go) holds all 7 PollState pointers (nullable via `omitempty`) and 7 receipt slices, plus a `LastUpdated` timestamp. This is the **root aggregate** for file-based persistence.

### Enums / Constants

| Type | Values | Location |
|------|--------|----------|
| StoreType | "file", "memory" | config.go |
| Log levels | DEBUG, INFO, WARN, ERROR, FATAL | config.go (validation), runner.go (parsing) |
| Record types | armis_alert, armis_activity, armis_audit_log, armis_risk_factor, armis_connection, armis_device, armis_vulnerability | http_sender.go (hardcoded strings) |

### Interfaces (verified from source)

| Interface | Package | Methods | Implementations |
|-----------|---------|---------|----------------|
| Store | state | AlertStore + ActivityStore + AuditLogStore + RiskFactorStore + ConnectionStore + DeviceStore + VulnerabilityStore (14 methods total) | FileStore, MemoryStore |
| Sender | sink | SendAlert, SendActivity, SendAuditLog, SendRiskFactor, SendConnection, SendDevice, SendVulnerability (7 methods) | HTTPSender, mock senders in tests |
| Client (armis) | armis | GetSearch(ctx, aql, includeSample, includeTotal) | HTTPClient |
| SearchClient | collector | GetSearch(ctx, aql, includeSample, includeTotal) | (mirrors armis.Client) |
| Reporter | health | SetReady(), SetNotReady() | health.Server |
| AlertStore | state | Load(ctx), Save(ctx, state, receipt) | FileStore, MemoryStore |
| (6 more per-source stores) | state | LoadXxx(ctx), SaveXxx(ctx, state, receipt) | FileStore, MemoryStore |

**Key finding:** There are TWO identical `Client`/`SearchClient` interfaces defined in different packages:
- `armis.Client` in `internal/armis/api.go`
- `collector.SearchClient` in `internal/collector/collector.go`

Both have the same single method signature. The collector package defines its own to avoid a direct import dependency on the armis package.

### Relationships

```
Collector --owns--> 7x XxxCollector
Collector --uses--> SearchClient (armis.Client)
Collector --uses--> state.Store
Collector --uses--> sink.Sender (optional, nil-safe)
Collector --uses--> health.Reporter (optional, nil-safe)

XxxCollector --uses--> SearchClient
XxxCollector --uses--> sink.Sender (optional)

runner.Execute --creates--> Config, Store, armis.HTTPClient, sink.HTTPSender, health.Server, Collector
runner.Execute --starts--> health.Server (goroutine), Collector.Run (blocking)

FileStore --persists--> fileState (7x PollState + 7x []BatchReceipt + LastUpdated)
HTTPSender --wraps--> centrix.SearchResult into EnrichedPayload
```

### Dependency Graph (verified from imports)

```
main.go --> runner.Execute
runner --> config, armis, collector, sink, state, health
collector --> config, armis(sdk), apperrors, sink, state, health
sink --> config, armis(sdk), apperrors
state --> apperrors
armis --> armis-sdk-go/v2
config --> (stdlib only)
health --> golang.org/x/time/rate
apperrors --> (stdlib only)
```

---

## Sub-pass 2b: Behavioral Extraction

### Cursor Extraction Variation Table (verified line-by-line from source)

This is the **primary variation point** across the 7 collectors:

| Collector | Timestamp Fields (priority order) | ID Extraction Chain | Forward Progress Error |
|-----------|----------------------------------|--------------------|-----------------------|
| Alert | LastAlertUpdateTime, Time | AlertID (int->string), PolicyID, Title, timestamp-nano | Plain fmt.Errorf (NO sentinel) |
| Activity | Time (single field) | PolicyID, ActivityUUIDs[0], Title, timestamp-nano | Plain fmt.Errorf (NO sentinel) |
| AuditLog | Time (single field) | PolicyID, Title, timestamp-nano | Plain fmt.Errorf (NO sentinel) |
| RiskFactor | LastSeen, FirstSeen | PolicyID, Title, timestamp-nano | Plain fmt.Errorf (NO sentinel) |
| Connection | StartTimestamp, EndTimestamp | ID (sdk type->string), Title, timestamp-nano | ErrCursorRegression sentinel |
| Device | LastSeen, FirstSeen | ID (sdk type->string), Title, timestamp-nano | ErrCursorRegression sentinel |
| Vulnerability | LastDetected, FirstDetected, PublishedDate | ID (sdk type->string), Title, timestamp-nano | ErrCursorRegression sentinel |

**CORRECTION from broad sweep:** The broad sweep said "Some collectors wrap with `ErrCursorRegression` sentinel, others use plain `fmt.Errorf()`." This is now precisely documented. The split is:
- **WITH sentinel:** Connection, Device, Vulnerability (all use `result.ID` from SDK)
- **WITHOUT sentinel:** Alert, Activity, AuditLog, RiskFactor (all use PolicyID-based ID chains)

This is a real inconsistency that matters for error handling behavior upstream.

### ID Type Asymmetry (new finding)

The SDK provides different ID field types per data source:
- `result.AlertID` -- `int` (converted to string via `fmt.Sprintf("%d", ...)`)
- `result.ID` -- custom SDK type (converted via `string(result.ID)`) -- used by Connection, Device, Vulnerability
- `result.PolicyID` -- `string` -- used by Activity, AuditLog, RiskFactor
- `result.ActivityUUIDs` -- `[]string` -- secondary for Activity

The alert collector has a unique check: `result.AlertID != 0` (int comparison), whereas Connection/Device/Vulnerability check `id == "" || id == "0"` (string comparison after conversion). RiskFactor/AuditLog/Activity check `strings.TrimSpace(result.PolicyID) != ""`.

### Timestamp Parsing (shared pattern, verified)

All collectors use the same dual-format parse attempt:
1. `time.Parse(time.RFC3339Nano, value)`
2. `time.Parse(time.RFC3339, value)`

If both fail, the record gets `time.Time{}` (zero) which causes it to be **skipped with a warning** during filtering.

### State Machine: Collector Lifecycle

```
[Not Started]
    |
    v  initializeState()
[Initializing] -- for each of 7 sources:
    |                load from store
    |                if found: check fingerprint match
    |                if not found: bootstrap with zero cursor + save
    |                if fingerprint mismatch: FATAL
    |                if store error: FATAL
    v
[Ready] -- reporter.SetReady()
    |
    v  enter loop
[Polling]
    |
    +---> collectOnce() succeeds, hasMore=false --> wait on ticker --> [Polling]
    +---> collectOnce() succeeds, hasMore=true  --> immediate loop --> [Polling]
    +---> collectOnce() fails:
              reporter.SetNotReady()
              retryCount++
              if retryCount > MaxRetries (and MaxRetries > 0): FATAL with ErrCollectorRetriesExceeded
              else: wait with exponential backoff --> [Polling]
    +---> collectOnce() succeeds after failures:
              retryCount = 0, retryDelay = baseDelay
              reporter.SetReady()
    +---> ctx.Done() --> return ctx.Err()
```

### State Machine: Health Server

```
[Constructed] -- alive=true, ready=false
    |
    v  ListenAndServe() (goroutine)
[Serving]
    |
    +-- GET /health, /live: 200 "ok" (if alive) or 503 "unhealthy"
    +-- GET /ready: 200 "ready" (if ready AND alive) or 503 "not ready"
    +-- SetReady(): ready=true
    +-- SetNotReady(): ready=false
    +-- Shutdown(): alive=false, httpServer.Shutdown()
```

### Sequential Collection Order (verified from collectOnce)

Collections execute **sequentially, not in parallel**:
1. Alerts
2. Activities
3. Audit Logs
4. Risk Factors
5. Connections
6. Devices
7. Vulnerabilities

If ANY source fails, the entire `collectOnce()` returns an error immediately. Later sources are not attempted. The `hasMore` flag is OR'd across all 7 sources.

### Bootstrap Behavior (verified from initializeXxxState)

When a data source has no prior state (`ErrStateNotFound`):
- Cursor is set to `{Timestamp: cfg.Collector.InitialSince, XxxID: ""}` where `InitialSince` defaults to `time.Time{}` (zero time)
- A bootstrap receipt with count=0 is persisted immediately
- Version starts at 0

This means on first run, the zero-time cursor causes ALL records from the API to pass the "is ahead" check, since any valid timestamp is after zero time.

### Fingerprint Construction (verified from collector.New)

Fingerprints are computed from `(AQL query string, Limit)` -- NOT from the `Fields` list despite `QueryFingerprint` having a `Fields` field. The `NewQueryFingerprint` call passes `[]string{cfg.Armis.AlertAQL}` as fields, meaning the fingerprint hash incorporates the AQL string and limit, treating the AQL string AS a field.

**CORRECTION from broad sweep:** The broad sweep described fingerprint as `SHA-256 of (sorted fields | limit)`. More precisely, it is SHA-256 of `"<AQL query>|<limit>"` since only the AQL string is passed as the single "field" element.

### Sink Delivery: ID Extraction Differences Between Sink and Collector

The `HTTPSender.SendXxx()` methods have their own ID extraction logic that is **separate from** the collector's `xxxResultCursor()` logic:

| Data Source | Collector ID Chain | Sink ID Chain |
|-------------|-------------------|---------------|
| Alert | AlertID(int), PolicyID, Title, nano | AlertID(int, "0"->Title) |
| Activity | PolicyID, ActivityUUIDs[0], Title, nano | ActivityUUIDs[0], Title, nano(now) |
| AuditLog | PolicyID, Title, nano | PolicyID, Title, nano(now) |
| RiskFactor | PolicyID, Title, nano | PolicyID, Title, nano(now) |
| Connection | ID(sdk), Title, nano | ID(sdk), Title, nano(now) |
| Device | ID(sdk), Title, nano | ID(sdk), Title, nano(now) |
| Vulnerability | ID(sdk), Title, nano | ID(sdk), Title, nano(now) |

**Key difference:** The sink uses `time.Now().UnixNano()` for fallback IDs (current time), while collectors use the record's parsed timestamp nano (which could be zero). This means the same record could have different "IDs" in the cursor vs. the delivery log.

### Ubiquitous Language (refined from broad sweep)

| Term | Precise Meaning |
|------|-----------------|
| Cursor | `(Timestamp, TypeSpecificID)` -- the composite position marker, always string-typed ID |
| Forward Progress | Invariant: new cursor must have (later timestamp) OR (same timestamp AND lexicographically greater ID) |
| Fingerprint | SHA-256 of `"<AQL query string>|<limit>"` -- detects query/limit changes between runs |
| Receipt | Immutable audit record of a completed batch: version, count, first/last IDs, timestamp, cursor applied |
| hasMore | Boolean signal: results exceeded the configured limit, immediate re-poll needed |
| Bootstrap | First-run initialization: zero cursor + empty receipt persisted, version=0 |
| collectOnce | Single sequential pass through all 7 data sources |
| Enrichment | Wrapping raw SearchResult JSON with record_type + xMP metadata before sink delivery |

---

## Delta Summary

- New items added: 3 subsystems fully documented (cursor extraction variation table, ID type asymmetry, sink vs collector ID divergence)
- Existing items refined: 8 (fingerprint construction corrected, forward progress error inconsistency precisely mapped, sequential collection order documented, bootstrap behavior detailed)
- Remaining gaps: SDK `centrix.SearchResult` struct fields not inspectable (external dependency), profiling subsystem domain model not analyzed (low priority)

## Novelty Assessment

Novelty: SUBSTANTIVE

The cursor extraction variation table, the forward progress sentinel inconsistency mapping, the fingerprint construction correction, and the sink-vs-collector ID divergence are all new findings that change how you would spec the system. The variation table alone defines the trait abstraction boundary for a Rust port.

## Convergence Declaration

Another round needed -- should verify: (1) whether `AuditLogLimit` and `RiskFactorLimit` are validated in config.Validate() (broad sweep lists them but code may differ), (2) the exact `collectOnce` error propagation pattern for per-source failures, (3) any additional domain model elements in health/profiling tests.

## State Checkpoint

```yaml
pass: 2
round: 1
status: complete
files_scanned: 32
timestamp: 2026-04-13T00:00:00Z
novelty: SUBSTANTIVE
next_action: round 2 -- audit hallucination classes, verify config validation completeness, check for missed types
```
