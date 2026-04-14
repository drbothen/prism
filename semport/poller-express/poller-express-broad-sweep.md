# Poller-Express: Brownfield Codebase Ingestion

## Executive Summary

poller-express is a Go service (Go 1.25.8) that continuously polls the **Cyberint Argos** threat intelligence API for two data types -- **security alerts** and **digital assets** -- and forwards them as enriched JSON payloads to an HTTP endpoint (Vector) for downstream processing. It uses cursor-based state management with modification timestamps for incremental collection, provides at-least-once delivery guarantees, and runs as a single-replica Kubernetes deployment with Helm-based configuration.

The codebase is clean, well-structured (~1,500 LOC of hand-written Go), with a large auto-generated OpenAPI client (~100+ model files). The hand-written code is production-quality with comprehensive test coverage, clear error handling, and sensible defaults.

---

## 1. Architecture

### 1.1 Component Topology

```
cmd/collector/main.go
    |
    v
internal/app/runner/runner.Execute()
    |
    +-- config.DefaultConfig() + LoadFromEnvironment()
    +-- state.NewMemoryStore()
    +-- http.Client (shared transport with cookieTransport)
    |       |
    |       +-- cyberint.NewAPIClient()     --> Alert API (OpenAPI-generated)
    |       +-- asset.NewClient()           --> Asset API (hand-written)
    |
    +-- sink.NewHTTPSender()                --> Vector endpoint
    +-- health.NewServer()                  --> :7322 liveness/readiness
    +-- profiling.Start()                   --> pprof (opt-in via ENABLE_PPROF)
    |
    +-- [goroutine] collector.New().Run()           --> Alert polling loop
    +-- [goroutine] collector.NewAssetCollector().Run() --> Asset polling loop
```

### 1.2 Data Flow

```
Cyberint Argos API
    |
    | POST /alert/api/v1/alerts (alerts)
    | POST /asset-configuration/external/api/v1/assets/ (assets)
    |
    v
Collector (filters by cursor, sorts by modification_date + record_id)
    |
    | JSON with xMP enrichment wrapper: {"data": <original>, "xmp": {...}}
    |
    v
Vector HTTP endpoint (basic auth, POST, application/json)
    |
    v
Downstream (logging, SIEM, etc.)
```

### 1.3 Module Boundaries

| Package | Responsibility | LOC (approx) |
|---------|---------------|--------------|
| `cmd/collector` | Entry point, dry-run validation, pprof lifecycle | 48 |
| `internal/app/runner` | Orchestration: wires all components, manages goroutine lifecycle | 240 |
| `internal/collector` | Alert + asset polling loops, cursor management, retry logic | 650 |
| `internal/config` | Env var loading, secret file reading, validation | 370 |
| `internal/sink` | HTTP delivery to Vector with xMP enrichment | 150 |
| `internal/state` | Cursor, PollState, QueryFingerprint, MemoryStore | 194 |
| `internal/health` | Liveness/readiness HTTP server with per-IP rate limiting | 173 |
| `internal/asset` | Hand-written client for Cyberint Asset Configuration API | 138 |
| `internal/apperrors` | Sentinel errors for all failure modes | 55 |
| `internal/profiling` | Optional pprof server | 53 |
| `pkg/cyberint` | OpenAPI-generated client (DO NOT EDIT) | ~10,000+ |
| `pkg/validate` | Deferred error-check utility | 16 |

### 1.4 Deployment Topology

- **Single-replica Kubernetes Deployment** via Helm chart
- **Container**: Distroless static-debian12 (nonroot user 65532)
- **No persistent storage**: State is in-memory only (MemoryStore). On restart, the cursor resets to zero and re-fetches everything.
- **Single service**: One binary runs both alert and asset collectors concurrently as goroutines
- **Sink target**: Vector HTTP server on port 4416 with basic auth

---

## 2. Authentication & API Connection

### 2.1 Cyberint API Authentication

**Mechanism**: Cookie-based authentication via a custom `http.RoundTripper`.

```go
// runner.go:196-207
type cookieTransport struct {
    apiKey string
    base   http.RoundTripper
}

func (t *cookieTransport) RoundTrip(req *http.Request) (*http.Response, error) {
    req.AddCookie(&http.Cookie{
        Name:  "access_token",
        Value: t.apiKey,
    })
    return t.base.RoundTrip(req)
}
```

Key facts:
- The API key is injected as a cookie named `access_token` on every request
- A **single shared `http.Client`** (30s timeout) is used by both alert and asset collectors, reusing TCP connections and TLS sessions
- The API key is loaded from either `CYBERINT_API_KEY_FILE` (Kubernetes secret mount) or `CYBERINT_API_KEY` (env var)
- File-backed secrets take precedence over direct env vars
- The base URL format is `https://<customer_id>.cyberint.io` -- the customer ID can be auto-extracted from the subdomain

### 2.2 Alert API Connection

Uses the **OpenAPI-generated client** (`pkg/cyberint`).

- **Endpoint**: `POST {baseURL}/alert/api/v1/alerts` (note: the generated client's server URL is set to `{baseURL}/alert`)
- **Request body**: `GetAlertsRequest` with page, size, and optional filters (modification_date range)
- **Response**: `GetAlertsResponse` containing `Total` count and `Alerts` array

### 2.3 Asset API Connection

Uses a **hand-written client** (`internal/asset`).

- **Endpoint**: `POST {baseURL}/asset-configuration/external/api/v1/assets/`
- **Request body**: `GetAssetsRequest` with customer_id, page_number, optional type/status filters
- **Response**: `GetAssetsResponse` with total_assets, page_number, and assets array
- **Response size limit**: 10 MiB hard cap
- **HTTP 204**: Treated as valid "no matching assets"

### 2.4 Sink (Vector) Authentication

- **Mechanism**: HTTP Basic Auth (`req.SetBasicAuth(username, password)`)
- **Content-Type**: `application/json`
- **Credentials source**: `VECTOR_USERNAME` / `VECTOR_PASSWORD` (or `*_FILE` variants)

---

## 3. Data Models

### 3.1 Alert Model (from OpenAPI spec)

The `Alert` struct is the primary data entity. Key fields:

| Field | Type | Description |
|-------|------|-------------|
| `Id` | `int32` | Internal numeric ID |
| `RefId` | `string` | Unique alert ID (used as cursor record ID) |
| `Environment` | `string` | Environment the alert belongs to |
| `Confidence` | `int32` | Confidence level that this is a real threat |
| `Status` | `AlertStatus` | `"open"`, `"acknowledged"`, `"closed"` |
| `Severity` | `AlertSeverity` | `"low"`, `"medium"`, `"high"`, `"very_high"` |
| `Category` | `AlertCategory` | General alert category |
| `Type` | `AlertType` | Specific threat type |
| `SourceCategory` | `AlertSourceCategory` | Where the event was detected |
| `Source` | `NullableString` | Specific source |
| `TargetedVectors` | `[]AlertTargetedVector` | Attack vectors targeted |
| `TargetedBrands` | `[]string` | Brands targeted |
| `RelatedEntities` | `[]string` | Related customer entities |
| `Impacts` | `[]AlertImpact` | Potential impact |
| `Title` | `Title` | Alert title |
| `Description` | `string` | Detailed description |
| `Recommendation` | `string` | Cyberint's recommended action |
| `AlertData` | `*AlertData` | Type-specific alert payload (polymorphic -- dozens of subtypes) |
| `Iocs` | `[]IOC` | Indicators of compromise |
| `Indicators` | `[]ResponseIndicator` | Response indicators |
| `Tags` | `[]string` | Alert tags |
| `Attachments` | `[]Attachment` | File attachments |
| `Mitre` | `[]string` | MITRE ATT&CK technique IDs |
| `RelatedAssets` | `[]Asset` | Related digital assets |
| `CreatedDate` | `CyberintTime` | Creation timestamp |
| `ModificationDate` | `CyberintTime` | Last modification (used as cursor timestamp) |
| `UpdateDate` | `CyberintTime` | Last content/status update |
| `ClosureDate` | `NullableTime` | When closed |
| `ClosureReason` | `NullableAlertClosureReason` | Why closed |
| `AcknowledgedDate` | `NullableTime` | When acknowledged |
| `ThreatActor` | `NullableString` | Attributed threat actor |
| `TicketId` | `NullableString` | External ticket reference |
| `AnalysisReport` | `NullableAnalysisReport` | Cyberint analysis report |
| `AssignedTo` | `NullableUser` | Assigned analyst |
| `CreatedBy` | `User` | Creator |
| `ClosedBy` | `NullableUser` | Closer |
| `AcknowledgedBy` | `NullableUser` | Acknowledger |

### 3.2 Alert Data Subtypes

The `AlertData` field is polymorphic with **50+ concrete types** covering different threat categories. Examples from the generated models:

- `ActivePhishingWebsiteTargetingCompanyAlertData`
- `CompanyEmployeeCorporateCredentialsExposedAlertData`
- `CompanyCustomerPaymentCardsExposedAlertData`
- `LookalikeDomainPotentiallyTargetingCompanyAlertData`
- `ExploitablePortOnCompanyServerDetectedAlertData`
- `RansomwareAttackTargetingThirdPartyVendor`
- `VulnerableTechnologyDetectedOnExposedCompanyAsset`
- `DDOSIncidentAlertData`
- `SSLCertificateExpirationAlertData`
- `MisconfiguredCompanyDomainDMARCRecordsDetectedAlertData`
- `SocialMediaAccountImpersonatingCompanyAlertData`
- And many more...

### 3.3 Asset Model (hand-written)

```go
type Asset struct {
    ID                 int64     `json:"id"`
    Name               *string   `json:"name,omitempty"`
    Type               *string   `json:"type,omitempty"`       // "domain", "ip", etc.
    Status             *string   `json:"status,omitempty"`     // "active", etc.
    AssetGroup         *string   `json:"asset_group,omitempty"`
    Created            time.Time `json:"created"`
    Updated            time.Time `json:"updated"`
    ParentAssetValue   *string   `json:"parent_asset_value,omitempty"`
    DiscoveryPrecision *int      `json:"discovery_precision,omitempty"`
    DiscoveryReason    *string   `json:"discovery_reason,omitempty"`
}
```

### 3.4 CyberintTime (Custom Time Handling)

Cyberint's API returns timestamps in multiple formats. The `CyberintTime` wrapper handles:

1. RFC3339 (`2006-01-02T15:04:05Z07:00`) -- standard
2. Without timezone (`2006-01-02T15:04:05`) -- assumed UTC
3. With microseconds (`2006-01-02T15:04:05.999999`) -- assumed UTC
4. `"null"` or empty string -- zero time

### 3.5 Enriched Payload (Sink Output Format)

Every record sent to Vector is wrapped:

```json
{
  "data": { /* original Alert or Asset JSON */ },
  "xmp": {
    "site": "...",
    "cluster_name": "...",
    "node_name": "..."
  }
}
```

The `xmp` fields identify the physical/logical site, Kubernetes cluster, and node where the collector runs. These are optional enrichment fields set via `XMP_SITE`, `XMP_CLUSTER_NAME`, `XMP_NODE_NAME` env vars.

### 3.6 Request Models

**GetAlertsRequest:**
```json
{
  "page": 1,
  "size": 100,
  "filters": {
    "modification_date": { "from": "...", "to": "..." }
  }
}
```

**GetAssetsRequest:**
```json
{
  "customer_id": "...",
  "page_number": 1,
  "type": ["domain", "ip"],
  "status": ["active"]
}
```

### 3.7 Key Enumerations

**AlertStatus**: `open`, `acknowledged`, `closed`

**AlertSeverity**: `low`, `medium`, `high`, `very_high`

**Filters** (for alert queries): `created_date`, `modification_date`, `update_date`, `environments`, `status`, `severity`, `type`, `targeted_brands`, `assigned_to`, `is_assigned`

---

## 4. Cursor-Based State Management

### 4.1 Cursor Strategy

Both collectors use a `(Timestamp, RecordID)` pair as their cursor position.

**Alert cursor**: `state.Cursor{Timestamp: CyberintTime, RecordID: string}` -- uses `ModificationDate` and `RefId`

**Asset cursor**: `state.AssetCursor{Timestamp: time.Time, RecordID: string}` -- uses `Updated` and `ID` (int64 converted to string)

### 4.2 Ordering and Filtering

1. Records are fetched from the API
2. Sorted by timestamp, then by record ID (stable sort) for deterministic ordering
3. Filtered against the current cursor: only records "ahead" of the cursor are processed
4. "Ahead" means: timestamp is later, OR timestamp is equal AND record ID is lexicographically greater

### 4.3 Forward Progress Guarantee

`ensureForwardProgress()` validates that the new cursor is strictly ahead of the current one. If not, a `ErrCursorRegression` error is returned, which prevents the cursor from going backward.

**Known issue in asset collector**: Record IDs are compared as strings first, with a numeric fallback. String comparison of numeric IDs can produce surprising results (e.g., `"50" > "100"` is true because `'5' > '1'`). The tests acknowledge this behavior explicitly.

### 4.4 Query Fingerprint

A SHA-256 hash of the sorted field names and limit is stored alongside the cursor. On restart, if the fingerprint doesn't match the current configuration, the collector returns `ErrQueryFingerprintMismatch` rather than using a stale cursor with changed query semantics.

Alert fingerprint: `NewQueryFingerprint(["cyberint_alerts"], 100)`
Asset fingerprint: `NewQueryFingerprint(["cyberint_assets"], 1000)`

### 4.5 State Persistence

Currently **in-memory only** (`MemoryStore`). The `Store` interface is designed for pluggable backends, but no persistent implementation exists. **On pod restart, all state is lost and the collector starts from scratch.**

### 4.6 Pagination

- **Alerts**: If a full page (100 records) is returned, the collector assumes more pages exist and immediately fetches the next batch without waiting for the ticker
- **Assets**: If `page_number * 1000 < total_assets`, more pages exist. Note: assets are fetched without cursor-based modification_date filtering -- they fetch all assets and filter client-side

---

## 5. Error Handling & Retry Patterns

### 5.1 Sentinel Errors

All error types are defined in `internal/apperrors`:

| Error | Meaning |
|-------|---------|
| `ErrStateNotFound` | No persisted state exists (triggers bootstrap) |
| `ErrQueryFingerprintMismatch` | Configuration drift detected -- **fatal** |
| `ErrCursorRegression` | Cursor would go backward -- **fatal for the batch** |
| `ErrCollectorRetriesExceeded` | Max retries exhausted -- **fatal** |
| `ErrCollectorStateLoad` | State store read failure |
| `ErrCollectorStatePersist` | State store write failure |
| `ErrCyberIntRequestExec` | API call failed |
| `ErrSinkConfigMissing` | Sink not properly configured |
| `ErrSinkRequestBuild` | Could not construct sink HTTP request |
| `ErrSinkDelivery` | Sink rejected or failed to accept data |

### 5.2 Retry Logic

Both collectors use identical exponential backoff:

```
Initial delay:  COLLECTOR_RETRY_BASE_DELAY  (default: 2s)
Max delay:      COLLECTOR_RETRY_MAX_DELAY   (default: 30s)
Max attempts:   COLLECTOR_MAX_RETRIES       (default: 5)
Backoff:        delay *= 2 each failure, capped at max_delay
Reset:          on any successful collection cycle
```

On retry exhaustion, the collector returns `ErrCollectorRetriesExceeded` and the process exits (runner.Execute returns an error, main returns exit code 1).

### 5.3 Error Handling Pattern

All errors are wrapped with `fmt.Errorf("context: %w", err)` using the sentinel errors. This allows callers to use `errors.Is()` for matching while preserving the full error chain.

### 5.4 Sink Error Handling

- HTTP status >= 400: Error with status code and up to 2048 bytes of response body in the error message
- HTTP transport errors: Wrapped as `ErrSinkDelivery`
- **No retry at the sink level** -- retry is handled by the collector's main loop, which means a sink failure retries the entire batch

### 5.5 Health State Transitions

The health reporter tracks collector state:
- `SetNotReady()` on startup and on any collection failure
- `SetReady()` after successful state initialization and after each successful collection cycle
- Liveness (`/health`, `/live`): Always returns 200 while the process is running
- Readiness (`/ready`): Returns 200 only when the collector has successfully completed at least one cycle

---

## 6. Configuration & Credential Management

### 6.1 Configuration Loading Order

1. `DefaultConfig()` provides baseline defaults
2. `LoadFromEnvironment()` overrides with env vars
3. For each secret: check `*_FILE` env var first (read file contents), fall back to direct env var
4. `Validate()` aggregates all validation errors

### 6.2 Secret File Pattern

Designed for Kubernetes secret volume mounts:

```go
func readSecretFile(path string) (string, error) {
    // Empty path -> return empty (not an error)
    // File not found -> return empty (not an error, allows fallback to env var)
    // File exists -> return trimmed contents
    // Other read error -> return error
}
```

Supported file-backed secrets:
- `CYBERINT_API_URL_FILE` / `CYBERINT_API_KEY_FILE`
- `VECTOR_ENDPOINT_FILE` / `VECTOR_USERNAME_FILE` / `VECTOR_PASSWORD_FILE`
- `ASSET_CUSTOMER_ID_FILE`

### 6.3 All Configuration Parameters

#### Required
| Variable | Description |
|----------|-------------|
| `CYBERINT_API_URL` | Base URL (e.g., `https://customer.cyberint.io`) |
| `CYBERINT_API_KEY` | API key (set as `access_token` cookie) |

#### Optional
| Variable | Default | Description |
|----------|---------|-------------|
| `VECTOR_ENDPOINT` | *(none)* | Sink URL. If unset, collector logs but doesn't forward |
| `VECTOR_USERNAME` | *(none)* | Basic auth username |
| `VECTOR_PASSWORD` | *(none)* | Basic auth password |
| `VECTOR_TIMEOUT_SECONDS` | `15s` | HTTP timeout (accepts `15s` or `15`) |
| `COLLECTOR_INTERVAL` | `30s` | Polling interval (>= 1s) |
| `COLLECTOR_MAX_RETRIES` | `5` | Max retry attempts before fatal exit |
| `COLLECTOR_RETRY_BASE_DELAY` | `2s` | Initial backoff delay |
| `COLLECTOR_RETRY_MAX_DELAY` | `30s` | Maximum backoff delay |
| `HEALTH_ADDR` | `:7322` | Health endpoint listen address |
| `ASSET_COLLECTION_ENABLED` | `true` | Toggle asset collection |
| `ASSET_CUSTOMER_ID` | *(auto from URL)* | Customer ID for asset API |
| `POLLER_LOG_LEVEL` | `INFO` | `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL` |
| `XMP_SITE` | *(none)* | Site identifier for payload enrichment |
| `XMP_CLUSTER_NAME` | *(none)* | Cluster identifier |
| `XMP_NODE_NAME` | *(hostname)* | Node identifier (falls back to `os.Hostname()`) |
| `ENABLE_PPROF` | *(unset)* | Enable pprof profiling server |
| `PPROF_ADDR` | `localhost:3030` | Pprof listen address |

### 6.4 Validation Rules

- `cyberint.baseURL`: required, must be valid URL
- `cyberint.apiKey`: required
- `collector.interval`: >= 1s
- `collector.retryBaseDelay`: >= 1s
- `collector.retryMaxDelay`: >= 1s, >= retryBaseDelay
- `collector.maxRetries`: >= 0
- `collector.healthAddr`: required
- `sink.endpoint`: if provided, must be valid URL
- `sink.timeout`: >= 1s
- `logging.level`: must be one of `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`

---

## 7. Behavioral Contracts

### BC-001: Alert Collection Cycle

**Preconditions**: Valid configuration loaded, Cyberint API reachable, state initialized
**Postconditions**: All new alerts (beyond cursor) forwarded to sink, cursor advanced to last processed alert, state persisted
**Error Cases**: 
- API failure -> retry with exponential backoff
- Sink rejection -> retry entire batch
- Cursor regression -> error, retry
- Max retries exceeded -> fatal exit
**Evidence**: `alert_collector.go:192-301`, tests in `alert_collector_test.go`
**Confidence**: HIGH

### BC-002: Asset Collection Cycle

**Preconditions**: Asset collection enabled, valid customer ID, Cyberint API reachable
**Postconditions**: All new assets (beyond cursor) forwarded to sink, cursor advanced
**Error Cases**: Same retry pattern as alerts. HTTP 204 = valid empty response
**Evidence**: `asset_collector.go:188-280`, tests in `asset_collector_test.go`
**Confidence**: HIGH

### BC-003: Cursor Forward Progress

**Preconditions**: Batch of records processed
**Postconditions**: New cursor timestamp >= old cursor timestamp; if equal, new record ID > old record ID
**Error Cases**: `ErrCursorRegression` if cursor would go backward
**Evidence**: `ensureForwardProgress()` in both collectors, `TestEnsureForwardProgress`
**Confidence**: HIGH

### BC-004: Authentication Cookie Injection

**Preconditions**: `http.Client` configured with `cookieTransport`
**Postconditions**: Every HTTP request to Cyberint includes `access_token={apiKey}` cookie
**Evidence**: `runner.go:196-207`
**Confidence**: HIGH

### BC-005: Payload Enrichment

**Preconditions**: Sink configured with xMP settings
**Postconditions**: Every payload wrapped as `{"data": <original>, "xmp": {site, cluster_name, node_name}}`
**Evidence**: `http_sender.go:121-149`, `TestHTTPSender_Send_Success`
**Confidence**: HIGH

### BC-006: Config File-Based Secrets

**Preconditions**: `*_FILE` env var points to a file
**Postconditions**: File contents used as the config value, trimmed of whitespace
**Error Cases**: File not found -> silent fallback to direct env var. Read error -> fatal
**Evidence**: `config.go:286-300`, `TestLoadFromEnvironment_UsesFileBackedSecrets`, `TestLoadFromEnvironment_FileMissingFallsBackToEnv`
**Confidence**: HIGH

### BC-007: Health Readiness State Machine

**Preconditions**: Health server running
**State machine**: `NotReady` -> (successful init) -> `Ready` -> (collection failure) -> `NotReady` -> (successful retry) -> `Ready`
**Evidence**: `health/server.go`, `alert_collector.go:86-98`, tests in `server_test.go`
**Confidence**: HIGH

### BC-008: Rate Limiting on Health Endpoints

**Preconditions**: Health server running
**Postconditions**: Per-IP rate limiting at 100 req/sec with burst of 20. Exceeding returns HTTP 429 with `Retry-After: 1` header
**Evidence**: `server.go:89-131`, comprehensive tests including per-IP isolation
**Confidence**: HIGH

---

## 8. Conventions & Patterns

### 8.1 Error Handling Pattern
- Sentinel errors in `internal/apperrors` for all failure categories
- All errors wrapped with `fmt.Errorf("context: %w", err)` preserving the chain
- No panics anywhere in the codebase

### 8.2 Configuration Pattern
- Defaults -> environment overlay -> validation
- Dual-source: direct env var + file-backed variant for Kubernetes secrets
- Aggregated validation errors (returns all problems, not just the first)

### 8.3 Logging
- Primary: `github.com/charmbracelet/log` with JSON formatter and timestamps
- Utility code: `log/slog` (only in `pkg/validate`)
- Structured fields: `type`, `endpoint`, `id`, `error`, `count`, etc.
- Log levels: DEBUG for cycle details, INFO for batch summaries, WARN for retries, ERROR for failures

### 8.4 Testing Pattern
- Table-driven tests with subtests (`t.Run`)
- `httptest.NewServer` for HTTP integration tests
- Mock implementations inline in test files (e.g., `mockSink`)
- Tests verify both happy path and error conditions
- Benchmark tests for performance-sensitive code (`BenchmarkEnrichPayload`)

### 8.5 HTTP Client Pattern
- Single shared `http.Client` for connection reuse
- Custom `RoundTripper` for authentication injection
- Context-aware requests throughout

### 8.6 Deferred Error Checking
- `defer validate.Check(resp.Body.Close)` pattern for cleaner cleanup
- The `Check` function logs errors from deferred closers rather than silently ignoring them

---

## 9. Gaps & Risks for Unified Rust MCP Server

### 9.1 State Persistence Gap
The current `MemoryStore` loses cursor state on restart. A Rust implementation should use durable storage (e.g., SQLite, file-backed) to avoid re-fetching the entire alert history on every restart.

### 9.2 Asset Cursor String Comparison Bug
Asset IDs (int64) are compared as strings for cursor ordering, which produces incorrect results for numeric IDs of different digit lengths. The Rust implementation should use numeric comparison.

### 9.3 No Pagination Loop for Alerts
The alert collector only fetches page 1 each cycle. If there are more than 100 alerts since the last cursor, it relies on the "hasMore" flag to immediately re-enter `collectOnce`, which then refetches with the updated cursor. This works but is inefficient -- a proper pagination loop within a single collection cycle would be cleaner.

### 9.4 Asset API Has No Server-Side Time Filtering
Unlike the alert API (which accepts `modification_date` filters), the asset API fetches all assets and filters client-side. For large asset inventories, this is wasteful. The Rust implementation should check if the Cyberint asset API supports any server-side filtering.

### 9.5 No TLS Certificate Verification Configuration
The shared HTTP client uses `http.DefaultTransport` with no customization for TLS. This is fine for production (it uses system CAs) but there's no option to configure custom CA bundles or skip verification for testing.

### 9.6 Sink Has No Retry Logic
Sink delivery failures cause the entire batch to be retried at the collector level. Individual record delivery failures are not retried independently. For the Rust implementation, consider per-record retry or batch-level retry with partial progress tracking.

### 9.7 No Graceful Shutdown Signal Handling
The collector relies on context cancellation but `main.go` doesn't set up OS signal handling (SIGTERM/SIGINT). In Kubernetes, the container runtime sends SIGTERM, which would kill the process without graceful shutdown of in-flight requests.

### 9.8 Single-Tenant Design
Each poller-express instance polls one Cyberint customer. For a multi-tenant MSSP deployment, the Rust MCP server should support polling multiple customers from a single process.

---

## 10. Key Decisions for Rust Port

### 10.1 What to Preserve
1. Cookie-based auth via `access_token` cookie on all Cyberint API requests
2. Cursor-based incremental polling with (timestamp, record_id) pairs
3. Forward progress validation preventing cursor regression
4. Query fingerprint for configuration drift detection
5. xMP enrichment wrapper format: `{"data": ..., "xmp": {...}}`
6. Exponential backoff retry: base_delay * 2^attempt, capped at max_delay
7. Health/readiness state machine: not_ready -> ready -> not_ready on failure -> ready on recovery
8. File-based secret loading for Kubernetes compatibility

### 10.2 What to Improve
1. Persistent cursor storage (not in-memory)
2. Numeric asset ID comparison
3. Proper pagination loops within single collection cycles
4. Individual record retry for sink delivery
5. OS signal handling for graceful shutdown
6. Multi-tenant support
7. Connection pooling with configurable TLS options

### 10.3 API Endpoints to Implement

| API | Method | Path | Auth | Body |
|-----|--------|------|------|------|
| Get Alerts | POST | `{base}/alert/api/v1/alerts` | Cookie: `access_token` | `GetAlertsRequest` with page, size, filters |
| Get Assets | POST | `{base}/asset-configuration/external/api/v1/assets/` | Cookie: `access_token` | `GetAssetsRequest` with customer_id, page_number |

### 10.4 Cyberint Date Format Handling

The Rust implementation needs a custom deserializer that handles:
- RFC3339: `2006-01-02T15:04:05Z07:00`
- No timezone: `2006-01-02T15:04:05` (assume UTC)
- Microseconds: `2006-01-02T15:04:05.999999` (assume UTC)
- Null/empty: zero time
