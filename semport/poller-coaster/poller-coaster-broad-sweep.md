# Poller Coaster -- Codebase Ingestion Analysis

**Sensor**: Armis Centrix  
**Org**: 1898 & Co (MSSP)  
**Language**: Go 1.25.7  
**Key dependency**: `github.com/1898andCo/armis-sdk-go/v2`  
**Analysis date**: 2026-04-13

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture](#2-architecture)
3. [API Connection & Authentication](#3-api-connection--authentication)
4. [Data Sources & Schemas](#4-data-sources--schemas)
5. [Polling & Cursor Mechanics](#5-polling--cursor-mechanics)
6. [Error Handling & Retry Patterns](#6-error-handling--retry-patterns)
7. [Configuration & Credential Management](#7-configuration--credential-management)
8. [State Persistence](#8-state-persistence)
9. [Sink / Downstream Delivery](#9-sink--downstream-delivery)
10. [Domain Model](#10-domain-model)
11. [Behavioral Contracts](#11-behavioral-contracts)
12. [NFR Catalog](#12-nfr-catalog)
13. [Conventions & Patterns](#13-conventions--patterns)
14. [Gaps, Risks & Translation Notes](#14-gaps-risks--translation-notes)

---

## 1. Executive Summary

Poller Coaster is a single-binary Go service that continuously polls the **Armis Centrix Search API** for seven data sources (alerts, activities, audit logs, risk factors, connections, devices, vulnerabilities) using AQL (Armis Query Language). It maintains durable cursor-based state so restarts resume without reprocessing, enriches each record with xMP metadata (site/cluster/node), and forwards enriched JSON payloads to a downstream HTTP sink (typically Vector) using basic auth.

The architecture is clean and well-structured: a runner wires dependencies, seven parallel-pattern collectors each manage their own cursor, a state store provides crash-safe persistence, and a health server exposes Kubernetes readiness/liveness probes.

**Key numbers**:
- ~32 Go source files, ~5,500 LOC (estimated)
- 7 data source collectors with identical structural patterns
- 13 test files with thorough table-driven and mock-based coverage
- 1 external API dependency (Armis SDK v2)
- Single-instance only (no distributed locking)

---

## 2. Architecture

### Component Diagram

```
main.go / cmd/collector/main.go
    |
    v
runner.Execute(ctx)
    |
    +---> config.DefaultConfig() + LoadFromEnvironment() + Validate()
    +---> state.NewFileStore() or state.NewMemoryStore()
    +---> armis.NewHTTPClient(cfg)
    +---> sink.NewHTTPSender(cfg)
    +---> health.NewServer(addr)
    +---> collector.New(cfg, armisClient, store, opts)
    |         |
    |         +---> AlertCollector
    |         +---> ActivityCollector
    |         +---> AuditLogCollector
    |         +---> RiskFactorCollector
    |         +---> ConnectionCollector
    |         +---> DeviceCollector
    |         +---> VulnerabilityCollector
    |
    +---> collector.Run(ctx) -- infinite poll loop
    +---> healthServer.ListenAndServe() -- goroutine
    +---> profiling.Start() -- optional pprof server
```

### Layer Structure

| Layer | Packages | Direction |
|-------|----------|-----------|
| Entrypoint | `main.go`, `cmd/collector/` | Calls runner |
| Orchestration | `internal/app/runner/` | Wires all components |
| Collection | `internal/collector/` | Calls armis client, sink, state |
| API Client | `internal/armis/` | Wraps SDK, calls Armis API |
| Delivery | `internal/sink/` | HTTP POST to downstream |
| Persistence | `internal/state/` | FileStore / MemoryStore |
| Config | `internal/config/` | Env vars + secret files |
| Health | `internal/health/` | HTTP readiness/liveness |
| Profiling | `internal/profiling/` | Opt-in pprof |
| Errors | `internal/apperrors/` | Sentinel error values |

### Data Flow

```
Armis Search API
    |  (AQL query via SDK GetSearch)
    v
centrix.SearchData{Results: []SearchResult}
    |  (sort by timestamp + ID)
    |  (filter against cursor)
    |  (cap at limit, track hasMore)
    v
For each record:
    sink.SendXxx(ctx, record)
        |  (marshal to JSON)
        |  (wrap with EnrichedPayload{data, record_type, xmp})
        |  (HTTP POST with basic auth)
        v
    Vector / downstream
    
After batch:
    state.Save(ctx, newPollState, batchReceipt)
        |  (atomic JSON write via temp file + rename)
        v
    /var/lib/poller-coaster/state.json
```

---

## 3. API Connection & Authentication

### Armis SDK Integration

**File**: `internal/armis/api.go`

The Armis client wraps `github.com/1898andCo/armis-sdk-go/v2/armis` (aliased as `centrix`).

```go
// Construction
inner, err := centrix.NewClient(cfg.APIKey, cfg.BaseURL, options...)

// Options applied:
centrix.WithAPIURL(baseURL)
centrix.WithHTTPClient(&http.Client{Timeout: timeout})
```

**Authentication**: Bearer token via API key. The SDK handles injecting the bearer header. The API key is passed directly to `centrix.NewClient()`.

**Single method used**: `GetSearch(ctx, aql, includeSample=false, includeTotal=false)` returns `centrix.SearchData` containing `[]centrix.SearchResult`.

### Auth Flow Summary

1. API key loaded from `ARMIS_API_KEY` env var or `ARMIS_API_KEY_FILE` (secret file mount)
2. Base URL from `ARMIS_API_URL` env var or file variant (default: `https://lab-1898andco.armis.com`)
3. SDK client constructed with key + URL + custom HTTP client (with configurable timeout, default 30s)
4. SDK handles bearer auth header injection on every request
5. No token refresh, no OAuth flow -- simple static API key

### Client Interface

```go
type Client interface {
    GetSearch(ctx context.Context, aql string, includeSample, includeTotal bool) (centrix.SearchData, error)
}
```

This is the **only** API operation the poller uses. All seven data sources go through `GetSearch` with different AQL queries.

---

## 4. Data Sources & Schemas

### Seven Data Sources

Each data source has: an AQL query, a per-request limit, a field list, a cursor type, and a dedicated collector.

#### 4.1 Alerts

- **Default AQL**: `in:alerts status:Open`
- **Default limit**: 100
- **Fields**: `title, status, severity, time, lastAlertUpdateTime, policyTitle, type, classification`
- **Record type**: `armis_alert`
- **Cursor timestamp source**: `lastAlertUpdateTime` (primary), `time` (fallback)
- **Cursor ID source**: `AlertID` (int, from SDK), `PolicyID`, `Title`, or timestamp nano

#### 4.2 Activities

- **Default AQL**: `in:activity`
- **Default limit**: 100
- **Fields**: `activityUUIDs, title, type, time`
- **Record type**: `armis_activity`
- **Cursor timestamp source**: `time`
- **Cursor ID source**: `PolicyID`, `ActivityUUIDs[0]`, `Title`, or timestamp nano

#### 4.3 Audit Logs

- **Default AQL**: `in:auditLog`
- **Default limit**: 100
- **Fields**: `id, time, action, user, trigger, userIp, additionalInfo`
- **Record type**: `armis_audit_log`
- **Cursor timestamp source**: `time`
- **Cursor ID source**: `PolicyID`, `Title`, or timestamp nano

#### 4.4 Risk Factors

- **Default AQL**: `in:riskFactors`
- **Default limit**: 100
- **Fields**: `id, name, description, severity, category, time, deviceId`
- **Record type**: `armis_risk_factor`
- **Cursor timestamp source**: `LastSeen` (primary), `FirstSeen` (fallback)
- **Cursor ID source**: `PolicyID`, `Title`, or timestamp nano

#### 4.5 Connections

- **Default AQL**: `in:connections`
- **Default limit**: 100
- **Fields**: `id, title, protocol, risk, startTimestamp, endTimestamp, sourceId, targetId, traffic, inboundTraffic, outboundTraffic, duration`
- **Record type**: `armis_connection`
- **Cursor timestamp source**: `StartTimestamp` (primary), `EndTimestamp` (fallback)
- **Cursor ID source**: `ID` (from SDK), `Title`, or timestamp nano

#### 4.6 Devices

- **Default AQL**: `in:devices`
- **Default limit**: 100
- **Fields**: `id, name, ipAddress, macAddress, manufacturer, model, operatingSystem, riskLevel, type, category, purdueLevel, firstSeen, lastSeen, tags, visibility`
- **Record type**: `armis_device`
- **Cursor timestamp source**: `LastSeen` (primary), `FirstSeen` (fallback) -- "reflects most recent device activity"
- **Cursor ID source**: `ID` (from SDK), `Title`, or timestamp nano

#### 4.7 Vulnerabilities

- **Default AQL**: `in:vulnerabilities`
- **Default limit**: 100
- **Fields**: `id, cveUid, description, cvssScore, cvssScoreV4, status, affectedDevicesCount, lastDetected, hasRemediationInfo, firstDetected, type, publishedDate, severity, attackVector, attackComplexity, userInteraction, privilegesRequired, scope, exploitabilityScore, impactScore, confidentialityImpact, integrityImpact, score, availabilityImpact`
- **Record type**: `armis_vulnerability`
- **Cursor timestamp source**: `LastDetected` (primary), `FirstDetected` (fallback), `PublishedDate` (tertiary)
- **Cursor ID source**: `ID` (from SDK), `Title`, or timestamp nano

### SDK SearchResult Type

The `centrix.SearchResult` is a flat struct from the Armis SDK. The poller does NOT define its own domain types for the API response -- it uses the SDK struct directly and passes it through to the sink. Key fields used across collectors:

- `AlertID` (int), `PolicyID` (string), `Title` (string), `Time` (string)
- `LastAlertUpdateTime` (string), `ActivityUUIDs` ([]string)
- `LastSeen`, `FirstSeen`, `StartTimestamp`, `EndTimestamp` (strings, RFC3339)
- `LastDetected`, `FirstDetected`, `PublishedDate` (strings, RFC3339)
- `ID` (custom type from SDK), `Protocol`, `Risk`

### Sink Payload Format

Every record is wrapped before delivery:

```json
{
  "data": { <raw SearchResult JSON> },
  "record_type": "armis_alert",
  "xmp": {
    "site": "site-id",
    "cluster_name": "prod-cluster",
    "node_name": "node-01"
  }
}
```

The `EnrichedPayload` struct:

```go
type EnrichedPayload struct {
    Data       json.RawMessage `json:"data"`
    RecordType string          `json:"record_type"`
    XMP        XMPMetadata     `json:"xmp"`
}
```

---

## 5. Polling & Cursor Mechanics

### Poll Loop (`collector.Run`)

1. **Initialize state** for all 7 data sources (load from store, or bootstrap with zero cursor)
2. **Set health to ready**
3. **Enter infinite loop**:
   - Call `collectOnce(ctx)` which sequentially collects all 7 sources
   - If error: increment retry counter, exponential backoff, set health to not-ready
   - If success with `hasMore=true`: immediately loop again (no ticker wait)
   - If success with `hasMore=false`: wait on ticker (default 30s) or context cancellation

### Cursor Advancement Pattern (identical across all 7 collectors)

Each `XxxCollector.Collect()` follows this exact pattern:

1. **Fetch**: Call `client.GetSearch(ctx, query, false, false)`
2. **Empty check**: If no results, return current state unchanged
3. **Copy + sort**: Copy results slice, sort by (timestamp, ID) ascending
4. **Filter**: Remove results at or before the current cursor position
5. **Limit**: If results exceed `fingerprint.Limit`, truncate and set `hasMore=true`
6. **Deliver**: For each result, call `sink.SendXxx()` (if sink is non-nil)
7. **Forward progress check**: Verify the last cursor is strictly ahead of the previous
8. **Build new state**: Increment version, set new cursor to last record's position
9. **Build receipt**: Record batch metadata (count, first/last IDs, timestamp)
10. **Return**: (newState, receipt, hasMore, nil)

### Cursor Comparison (2-field composite)

Every cursor type uses `(Timestamp, TypeID)` with lexicographic string comparison for tie-breaking:

```go
func isCursorAhead(previous, next XxxCursor) bool {
    if next.Timestamp.After(previous.Timestamp) { return true }
    if next.Timestamp.Equal(previous.Timestamp) && next.ID > previous.ID { return true }
    return false
}
```

### Forward Progress Guarantee

The `ensureXxxForwardProgress()` function prevents cursor regression. If the new cursor is not strictly ahead of the previous, it returns an error (wrapping `ErrCursorRegression` for some types, or a plain format error for others).

### Query Fingerprinting

State includes a SHA-256 fingerprint of `(sorted fields | limit)`. On startup, if the stored fingerprint mismatches the current config, the collector refuses to start with `ErrQueryFingerprintMismatch`. This prevents silently resuming with incompatible query parameters.

---

## 6. Error Handling & Retry Patterns

### Sentinel Errors (`internal/apperrors/`)

| Error | Meaning |
|-------|---------|
| `ErrStateNotFound` | No persisted state yet (triggers bootstrap) |
| `ErrQueryFingerprintMismatch` | Config changed since last run -- fatal |
| `ErrCursorRegression` | Cursor moved backward -- error |
| `ErrCollectorRetriesExceeded` | Max retries exhausted -- fatal |
| `ErrCollectorStateLoad` | State store read failed |
| `ErrCollectorStatePersist` | State store write failed |
| `ErrArmisRequestExec` | Armis API call failed |
| `ErrSinkDelivery` | Downstream rejected/failed |
| `ErrSinkConfigMissing` | Sink not configured properly |
| `ErrSinkRequestBuild` | HTTP request construction failed |

### Retry with Exponential Backoff

**Location**: `collector.Run()` main loop.

```
Base delay: 2s (configurable via COLLECTOR_RETRY_BASE_DELAY)
Max delay:  30s (configurable via COLLECTOR_RETRY_MAX_DELAY)
Max retries: 10 (configurable via COLLECTOR_MAX_RETRIES, 0 = unlimited)
Strategy: delay *= 2 after each failure, capped at maxDelay
Reset: retry counter and delay reset to base on ANY successful collection
```

**Tested behaviors** (from `collector_test.go`):
- BC-001: Retries exhaust after configured max attempts
- BC-002: Retry counter resets after a successful collection
- BC-003: When MaxRetries=0, retries are unlimited (context timeout only)
- BC-004: Error message includes attempt count

### Error Wrapping

All errors are wrapped with `fmt.Errorf("context: %w", err)` using sentinel errors from `apperrors`. This enables `errors.Is()` checks upstream.

### Sink Error Handling

- HTTP status >= 400: reads up to 2048 bytes of response body, logs it, returns `ErrSinkDelivery` with status code and body
- Network error: wraps with `ErrSinkDelivery`
- Response body close error: logged as warning, not propagated

---

## 7. Configuration & Credential Management

### Loading Order

1. `config.DefaultConfig()` -- hardcoded sane defaults
2. `config.LoadFromEnvironment(cfg)` -- env var overrides
3. `cfg.Validate()` -- comprehensive validation

### Secret File Support

For every sensitive value, there is a `*_FILE` variant that reads from a file path (for Kubernetes secret mounts):

| Direct var | File var | Purpose |
|------------|----------|---------|
| `ARMIS_API_URL` | `ARMIS_API_URL_FILE` | Armis base URL |
| `ARMIS_API_KEY` | `ARMIS_API_KEY_FILE` | Armis API token |
| `VECTOR_ENDPOINT` | `VECTOR_ENDPOINT_FILE` | Sink URL |
| `VECTOR_USERNAME` | `VECTOR_USERNAME_FILE` | Sink basic auth user |
| `VECTOR_PASSWORD` | `VECTOR_PASSWORD_FILE` | Sink basic auth pass |

File variant takes priority. Empty file or non-existent file silently falls back to env var.

### Duration Parsing

Timeout values accept either Go duration strings (`"30s"`, `"2m"`) or plain integers (interpreted as seconds).

### Config Struct Hierarchy

```go
Config {
    Armis:     ArmisConfig     // URL, key, timeout, 7x AQL+limit+fields
    Collector: CollectorConfig // interval, retry params, healthAddr
    Sink:      SinkConfig      // endpoint, username, password, timeout
    Logging:   LoggingConfig   // level
    XMP:       XMPConfig       // site, clusterName, nodeName
    State:     StateConfig     // type (file|memory), path, maxReceipts
}
```

### Validation Rules

- `Armis.BaseURL`: required, must parse as URL
- `Armis.APIKey`: required, non-empty
- `Armis.Timeout`: >= 1s
- All 7 AQL queries: required, non-empty
- All 7 limits: >= 1
- `Collector.Interval`: >= 1s
- `Collector.RetryBaseDelay`: >= 1s
- `Collector.RetryMaxDelay`: >= 1s, >= RetryBaseDelay
- `Collector.MaxRetries`: >= 0
- `Collector.HealthAddr`: required
- `Sink.Endpoint`: optional, but if set must be valid URL
- `Sink.Timeout`: >= 1s
- `State.Path`: required when Type is "file"
- `State.MaxReceipts`: >= 1
- `Logging.Level`: one of DEBUG, INFO, WARN, ERROR, FATAL

Validation errors are aggregated using `errors.Join()` and returned together.

---

## 8. State Persistence

### Store Interface

```go
type Store interface {
    AlertStore       // Load/Save for alerts
    ActivityStore    // LoadActivity/SaveActivity
    AuditLogStore    // LoadAuditLog/SaveAuditLog
    RiskFactorStore  // LoadRiskFactor/SaveRiskFactor
    ConnectionStore  // LoadConnection/SaveConnection
    DeviceStore      // LoadDevice/SaveDevice
    VulnerabilityStore // LoadVulnerability/SaveVulnerability
}
```

### FileStore (default, production)

**Path**: `/var/lib/poller-coaster/state.json` (configurable)

**Atomic write pattern**:
1. Marshal state to indented JSON
2. Create temp file in same directory (`.poller-state-*.tmp`)
3. Write data to temp file
4. `Sync()` the temp file (fsync)
5. `Close()` the temp file
6. `Rename()` temp to target path (atomic on POSIX)
7. On any error: cleanup temp file, return error

**Concurrency**: `sync.RWMutex` protects all reads and writes.

**Receipt trimming**: Each save appends a `BatchReceipt` to the receipts array for that data source, then trims to `maxReceipts` (default 100) by keeping the most recent entries.

**State JSON structure** (`fileState`):
```json
{
  "alert": { "Cursor": {...}, "Query": {...}, "UpdatedAt": "...", "Version": N },
  "activity": { ... },
  "audit_log": { ... },
  "risk_factor": { ... },
  "connection": { ... },
  "device": { ... },
  "vulnerability": { ... },
  "alert_receipts": [ ... ],
  "activity_receipts": [ ... ],
  "audit_log_receipts": [ ... ],
  "risk_factor_receipts": [ ... ],
  "connection_receipts": [ ... ],
  "device_receipts": [ ... ],
  "vulnerability_receipts": [ ... ],
  "last_updated": "2026-04-13T..."
}
```

### MemoryStore (testing, ephemeral)

In-memory with `sync.RWMutex`. State lost on restart. Returns `ErrStateNotFound` before first save.

### Poll State Structure (per data source)

```go
type XxxPollState struct {
    Cursor    XxxCursor        // (Timestamp, XxxID)
    Query     QueryFingerprint // (Hash, Fields, Limit)
    UpdatedAt time.Time
    Version   uint64           // monotonically increasing
}
```

### Batch Receipt Structure (per data source)

```go
type XxxBatchReceipt struct {
    Version       uint64
    RequestHash   string
    Count         int
    FirstXxxID    string
    LastXxxID     string
    FetchedAt     time.Time
    CursorApplied XxxCursor
}
```

---

## 9. Sink / Downstream Delivery

### HTTPSender

**File**: `internal/sink/http_sender.go`

**Protocol**: HTTP POST with JSON body and basic auth.

**Construction requirements**:
- Endpoint must be non-empty, parse as absolute URL
- Username AND password must both be provided
- Timeout defaults to 15s

**Per-record delivery**: Each record is sent individually (no batching). The `sendPayload()` method:

1. Enriches with xMP metadata (wraps original JSON in `EnrichedPayload`)
2. Creates `POST` request to endpoint
3. Sets `Content-Type: application/json`
4. Sets basic auth header
5. Sends request
6. Checks response: status >= 400 is an error
7. Logs success or failure with structured fields

**Sink is optional**: If `VECTOR_ENDPOINT` is not set, no sink is created and records are still collected/cursor-advanced but not forwarded. The sink field is nil-checked in every collector.

### Sender Interface

```go
type Sender interface {
    SendAlert(ctx context.Context, alert centrix.SearchResult) error
    SendActivity(ctx context.Context, activity centrix.SearchResult) error
    SendAuditLog(ctx context.Context, auditLog centrix.SearchResult) error
    SendRiskFactor(ctx context.Context, riskFactor centrix.SearchResult) error
    SendConnection(ctx context.Context, connection centrix.SearchResult) error
    SendDevice(ctx context.Context, device centrix.SearchResult) error
    SendVulnerability(ctx context.Context, vulnerability centrix.SearchResult) error
}
```

---

## 10. Domain Model

### Entity Catalog

| Entity | Identity | Timestamp Fields | Domain |
|--------|----------|-----------------|--------|
| Alert | AlertID (int), PolicyID, Title | time, lastAlertUpdateTime | Security events |
| Activity | ActivityUUIDs, PolicyID, Title | time | User/system activity |
| AuditLog | PolicyID, Title | time | Platform audit trail |
| RiskFactor | PolicyID, Title | LastSeen, FirstSeen | Risk assessment |
| Connection | ID, Title | StartTimestamp, EndTimestamp | Network connections |
| Device | ID, Title | LastSeen, FirstSeen | Asset inventory |
| Vulnerability | ID, Title | LastDetected, FirstDetected, PublishedDate | Vulnerability management |

### Relationships (implicit)

- `RiskFactor.deviceId` -> `Device.id` (risk factors associated with devices)
- `Connection.sourceId` / `Connection.targetId` -> `Device.id` (network connections between devices)
- `Vulnerability.affectedDevicesCount` -> count of associated devices

### Value Objects

- **AlertCursor** / **ActivityCursor** / etc: `(Timestamp, ID)` composite cursor
- **QueryFingerprint**: `(Hash, Fields, Limit)` -- detects config drift
- **XMPMetadata**: `(Site, ClusterName, NodeName)` -- enrichment metadata
- **EnrichedPayload**: `(Data, RecordType, XMP)` -- wire format
- **BatchReceipt**: audit trail for each poll batch

### Ubiquitous Language

| Term | Meaning |
|------|---------|
| AQL | Armis Query Language -- the search syntax |
| Cursor | (timestamp, ID) position marking how far polling has progressed |
| Forward progress | Invariant that cursors must only advance, never regress |
| Fingerprint | SHA-256 hash of query parameters to detect config changes |
| Receipt | Audit record of a completed poll batch |
| Sink | Downstream HTTP endpoint receiving enriched records |
| xMP | Metadata enrichment (site, cluster, node) for attribution |
| Vector | The typical downstream sink (a log/event aggregation tool) |

---

## 11. Behavioral Contracts

### BC-001: Retry exhaustion terminates collector

**Preconditions**: API returns errors continuously  
**Postconditions**: After `MaxRetries` consecutive failures, `Run()` returns `ErrCollectorRetriesExceeded`  
**Error cases**: Each failure doubles the backoff delay up to `RetryMaxDelay`  
**Evidence**: `TestCollector_MaxRetries_ExhaustsAfterConfiguredAttempts`  
**Confidence**: HIGH

### BC-002: Retry counter resets on success

**Preconditions**: API fails N times then succeeds  
**Postconditions**: Retry counter resets to 0, health set to ready  
**Evidence**: `TestCollector_MaxRetries_ResetsAfterSuccess`  
**Confidence**: HIGH

### BC-003: Unlimited retries when MaxRetries=0

**Preconditions**: `MaxRetries=0`, API fails continuously  
**Postconditions**: Never returns `ErrCollectorRetriesExceeded`, only stops on context cancellation  
**Evidence**: `TestCollector_MaxRetries_UnlimitedWhenZero`  
**Confidence**: HIGH

### BC-004: Cursor filters already-processed records

**Preconditions**: API returns mix of old and new records  
**Postconditions**: Only records beyond cursor position are delivered to sink  
**Evidence**: `TestDeviceCollector_Collect_FiltersBeyondCursor`, `TestConnectionCollector_Collect_FiltersBeyondCursor`, etc.  
**Confidence**: HIGH

### BC-005: hasMore=true when results exceed limit

**Preconditions**: API returns more results than configured limit  
**Postconditions**: Results truncated to limit, `hasMore=true` triggers immediate re-poll  
**Evidence**: `TestDeviceCollector_Collect_HasMore`, `TestVulnerabilityCollector_Collect_HasMore`  
**Confidence**: HIGH

### BC-006: Nil sink does not block collection

**Preconditions**: Sink endpoint not configured  
**Postconditions**: Collection proceeds, state advances, no delivery errors  
**Evidence**: `TestDeviceCollector_Collect_NilSink`, `TestConnectionCollector_Collect_NilSink`  
**Confidence**: HIGH

### BC-007: Invalid timestamps are skipped with warning

**Preconditions**: API returns records with unparseable timestamps  
**Postconditions**: Record filtered out, warning logged, remaining records processed  
**Evidence**: `TestAuditLogCollector_Collect_InvalidTimestamp`, `TestRiskFactorCollector_Collect_InvalidTimestamp`  
**Confidence**: HIGH

### BC-008: Timestamp field fallback chains

**Preconditions**: Primary timestamp field is empty  
**Postconditions**: Falls back to secondary (then tertiary for vulnerabilities)  
**Evidence**: `TestDeviceCollector_Collect_UsesFirstSeenFallback`, `TestVulnerabilityCollector_Collect_UsesFirstDetectedFallback`, `TestVulnerabilityCollector_Collect_UsesPublishedDateFallback`, `TestConnectionCollector_Collect_UsesEndTimestampFallback`  
**Confidence**: HIGH

### BC-009: Query fingerprint mismatch prevents startup

**Preconditions**: Stored state has different query fingerprint than current config  
**Postconditions**: `initializeXxxState()` returns `ErrQueryFingerprintMismatch`  
**Evidence**: Code path in `collector.go:initializeAlertState()` etc.  
**Confidence**: MEDIUM (no dedicated test, but clear code path)

### BC-010: Atomic state file persistence

**Preconditions**: State needs to be saved  
**Postconditions**: Write to temp file, fsync, atomic rename -- crash-safe  
**Evidence**: `file_store.go:persistState()`  
**Confidence**: HIGH (from code analysis)

### BC-011: Results sorted before cursor comparison

**Preconditions**: API returns results in arbitrary order  
**Postconditions**: Results sorted by (timestamp, ID) ascending before filtering  
**Evidence**: `TestAuditLogCollector_Collect_SortsResults`, `TestRiskFactorCollector_Collect_SortsResults`  
**Confidence**: HIGH

### BC-012: ID fallback chains for cursor identity

**Preconditions**: Primary ID field is empty/zero  
**Postconditions**: Falls back to Title, then timestamp nano  
**Evidence**: `TestDeviceCollector_Collect_UsesTitleFallbackForID`, `TestVulnerabilityCollector_Collect_UsesTitleFallbackForID`  
**Confidence**: HIGH

---

## 12. NFR Catalog

### Performance

| Pattern | Location | Details |
|---------|----------|---------|
| Configurable poll interval | `COLLECTOR_INTERVAL` | Default 30s between cycles |
| Per-request result limit | `ARMIS_xxx_LIMIT` | Default 100 per data source |
| Immediate re-poll on hasMore | `collector.Run()` | Skips ticker when more data available |
| HTTP client timeout | `ARMIS_API_TIMEOUT` | Default 30s per API request |
| Sink timeout | `VECTOR_TIMEOUT_SECONDS` | Default 15s per delivery |

### Security

| Pattern | Location | Details |
|---------|----------|---------|
| Secret file support | `config.LoadFromEnvironment()` | `*_FILE` variants for K8s secret mounts |
| Bearer auth for Armis | `armis.NewHTTPClient()` | API key injected via SDK |
| Basic auth for sink | `HTTPSender.sendPayload()` | `req.SetBasicAuth()` |
| Secret redaction in logs | `config/utils.go:redactSecret()` | Shows first 2 + last 2 chars |
| Non-root container | `Dockerfile` | distroless:nonroot, UID 65532 |
| Read-only root filesystem | `values.yaml` | `readOnlyRootFilesystem: true` |
| Drop all capabilities | `values.yaml` | `capabilities.drop: [ALL]` |
| Seccomp profile | `values.yaml` | `RuntimeDefault` |
| pprof cmdline blocked | `profiling/pprof.go` | `/debug/pprof/cmdline` returns 404 |
| pprof loopback warning | `profiling/pprof.go` | Warns if non-loopback address |
| No secrets in logs | CLAUDE.md convention | "never log API keys or secrets" |

### Observability

| Pattern | Location | Details |
|---------|----------|---------|
| Structured JSON logging | `charmbracelet/log` with JSONFormatter | Throughout |
| Configurable log levels | `POLLER_COASTER_LOG_LEVEL` | DEBUG/INFO/WARN/ERROR/FATAL |
| Health/readiness endpoints | `:7322` /health, /ready, /live | K8s probes |
| Batch processing logs | Every `collectXxx()` | Count, timestamp, ID, version |
| Sink delivery logs | `HTTPSender.sendPayload()` | Type, endpoint, ID, size_bytes |
| Opt-in pprof | `ENABLE_PPROF=1` on `localhost:3030` | CPU, memory, goroutine profiling |

### Reliability

| Pattern | Location | Details |
|---------|----------|---------|
| Exponential backoff | `collector.Run()` | 2s base, 30s max, configurable |
| Configurable max retries | `COLLECTOR_MAX_RETRIES` | Default 10, 0=unlimited |
| Atomic file writes | `FileStore.persistState()` | temp+fsync+rename pattern |
| Forward progress invariant | `ensureXxxForwardProgress()` | Prevents cursor regression |
| Query fingerprint validation | `initializeXxxState()` | Detects config drift |
| Graceful shutdown | `runner.Execute()` | Context cancellation, 5s shutdown timeout |
| Receipt-based auditing | `BatchReceipt` types | Tracks what was fetched and when |
| Health state transitions | `SetReady()`/`SetNotReady()` | Toggles on error/success |
| Rate-limited health endpoint | `health.Server` | 100 req/s per IP with burst of 20 |

### Missing NFRs

- No rate limiting for Armis API calls (relies on SDK/API-side limits)
- No circuit breaker pattern
- No metrics/Prometheus endpoint
- No distributed locking for multi-instance
- No sink retry (failure bubbles up to collector retry loop)
- No mTLS or OAuth for sink auth

---

## 13. Conventions & Patterns

### Code Organization

- **Standard Go project layout**: `cmd/`, `internal/`, single `go.mod`
- **Package per concern**: armis, collector, config, health, profiling, sink, state
- **Two entrypoints**: `main.go` and `cmd/collector/main.go` (identical, both call `runner.Execute`)

### Error Handling

- Sentinel errors in `internal/apperrors/` with `errors.New()`
- Wrapping via `fmt.Errorf("%w: %v", sentinelErr, err)` -- note: uses `%v` not `%w` for the inner error in most places, which means only the sentinel is matchable with `errors.Is()`
- Aggregated validation errors via `errors.Join()`

### Naming Conventions

- **Files**: `snake_case.go` with `_test.go` co-located
- **Types**: PascalCase, descriptive (`AlertCollector`, `DevicePollState`, `EnrichedPayload`)
- **Env vars**: SCREAMING_SNAKE_CASE
- **Config fields**: PascalCase structs matching env var semantics
- **Record types**: `armis_xxx` (snake_case for JSON wire format)

### Testing Patterns

- **Table-driven tests**: Used in `store_test.go`
- **Mock interfaces**: Each test file defines mock implementations of `SearchClient` and `Sender`
- **Parallel tests**: All tests use `t.Parallel()`
- **Test coverage areas**: no results, with results, API errors, sink errors, hasMore, cursor filtering, nil sink, timestamp fallbacks, ID fallbacks, sorting, cursor advancement

### Design Patterns

- **Strategy pattern**: `Store` interface with FileStore/MemoryStore implementations
- **Functional options**: `FileStoreOption` for configuring FileStore
- **Interface segregation**: Separate store interfaces per data source, composed into `Store`
- **Null object**: Nil sink is handled gracefully throughout
- **Composite cursor**: (timestamp, ID) with deterministic tie-breaking

### Repetition / Code Smell

The 7 collectors are **nearly identical** in structure. Each has:
- An `XxxCollector` struct
- `Collect()` method with the same algorithm
- `filterNewXxx()`, `xxxResultCursor()`, `parseXxxTimestamp()`, `isXxxCursorAhead()`, `ensureXxxForwardProgress()`
- Corresponding `XxxPollState`, `XxxCursor`, `XxxBatchReceipt` types
- Per-source `Load`/`Save` methods on the store

This is a prime candidate for generification in the Rust port. The behavioral algorithm is identical; only the cursor field selection and record type string vary.

---

## 14. Gaps, Risks & Translation Notes

### For Rust MCP Server Port

1. **Generic collector pattern**: The 7 collectors share identical logic. In Rust, use a generic `Collector<C: CursorType, R: Receipt>` with trait-based cursor extraction. This eliminates ~75% of the collector code duplication.

2. **Armis SDK dependency**: The Go code wraps `armis-sdk-go/v2`. In Rust, you will need to either:
   - Port the SDK's `GetSearch` HTTP call directly (it is a single REST endpoint)
   - Or write a thin Armis API client with `reqwest`

3. **SearchResult type**: The SDK provides a flat struct. For Rust, consider defining separate typed structs per data source (AlertRecord, DeviceRecord, etc.) rather than a single mega-struct.

4. **Cursor extraction varies by data source**: Each collector has its own `parseXxxTimestamp()` function choosing different timestamp fields. This is the primary variation point -- model it as a trait:
   ```rust
   trait CursorExtractor {
       fn extract_timestamp(result: &SearchResult) -> Option<DateTime<Utc>>;
       fn extract_id(result: &SearchResult) -> String;
       fn record_type() -> &'static str;
   }
   ```

5. **State store is single-file JSON**: Works for single-instance. For MCP server, consider whether state needs to be external (database) for multi-instance.

6. **No pagination in AQL queries**: The code does not append `after:timestamp` to the AQL. It relies on the Armis API returning results in a way that cursor-based filtering works. This may mean the API always returns the latest N results and the client filters. Verify this assumption when porting.

7. **Sink delivery is per-record, not batched**: Each record gets its own HTTP POST. Consider batching in the Rust port for efficiency.

8. **Basic auth only for sink**: Consider supporting bearer token auth in the unified server.

9. **Duration parsing quirk**: Accepts both Go duration strings and plain integers (as seconds). Rust port should standardize on one format.

10. **Forward progress error inconsistency**: Some collectors wrap with `ErrCursorRegression` sentinel, others use plain `fmt.Errorf()`. Minor inconsistency to clean up in port.

### Known Limitations (from README)

- Single-instance state (no distributed locking)
- Sink only supports HTTP basic auth
- Heap profiles can capture in-memory secrets
- `FileStore.Receipts()` only returns alert receipts (TODO in code)

### Cross-Reference with Other Pollers

When comparing with other poller repos, look for:
- Same `runner.Execute` -> config -> collector pattern?
- Same cursor-based state persistence approach?
- Same sink/xMP enrichment format?
- Different Armis API operations (not just GetSearch)?
- Different auth mechanisms (OAuth, mTLS)?
