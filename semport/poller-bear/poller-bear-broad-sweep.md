# Poller Bear -- Full Codebase Ingestion Analysis

> Sensor: **Claroty xDome**
> Repo: `github.com/1898andCo/poller-bear`
> Language: Go 1.25.7 (Docker builds with 1.26.1)
> Analysis date: 2026-04-13

---

## 1. Executive Summary

Poller Bear is a long-running Go service that continuously polls the Claroty xDome REST API for nine data sources (alerts, OT activity events, audit logs, device-alert relations, device-vulnerability relations, servers, sites, devices, and vulnerabilities). It persists cursor-based pagination state to a JSON file (or memory), enriches each record with xMP metadata and optional OCSF normalization, then forwards individual records over HTTP with basic auth to a downstream sink (typically Vector). The architecture is clean and modular: a `collector` drives the polling loop, a `claroty` client handles API interaction, a `state` store tracks cursors, and a `sink` delivers records.

The codebase was recently rewritten from a Python legacy implementation (preserved in `legacy/`) to idiomatic Go with comprehensive test coverage (~50-60%), structured logging via Charmbracelet, and Kubernetes-native deployment via Helm.

---

## 2. Architecture

### 2.1 Component Diagram

```
main.go / cmd/collector/main.go
    |
    v
runner.Execute(ctx)
    |
    +-- config.LoadFromEnvironment()     -- env vars + secret files
    +-- state.NewFileStore/MemoryStore() -- cursor persistence
    +-- claroty.NewHTTPClient()          -- Claroty API client
    +-- sink.NewHTTPSender()             -- downstream delivery
    +-- health.NewServer()               -- /health, /ready, /live
    +-- profiling.Start()                -- optional pprof
    |
    v
collector.New(cfg, client, store, opts)
    |
    v
collector.Run(ctx)  -- infinite loop: collectOnce() -> sleep(30s)
    |
    +-- collectAlerts()
    +-- collectEvents()
    +-- collectAuditLogs()
    +-- collectDeviceAlertRelations()
    +-- collectDeviceVulnerabilityRelations()
    +-- collectServers()
    +-- collectSites()
    +-- collectDevices()
    +-- collectVulnerabilities()
```

### 2.2 Layer Structure

| Layer | Package | Responsibility |
|-------|---------|---------------|
| Entry | `main.go`, `cmd/collector/main.go` | Process setup, pprof, calls `runner.Execute` |
| Orchestration | `internal/app/runner` | Signal handling, component wiring, graceful shutdown |
| Collection | `internal/collector` | Polling loop, retry logic, pagination orchestration |
| API Client | `internal/claroty` | HTTP requests to Claroty xDome API, response decoding |
| State | `internal/state` | Cursor persistence (file/memory), query fingerprinting |
| Delivery | `internal/sink` | HTTP forwarding with basic auth, xMP enrichment, OCSF mapping |
| Normalization | `internal/ocsf` | OCSF Detection Finding (2004) types, severity mapping |
| Transport | `internal/transport` | Shared HTTP transport with TLS 1.2+, connection pooling |
| Health | `internal/health` | Kubernetes liveness/readiness probes |
| Errors | `internal/apperrors` | Sentinel error values |
| Profiling | `internal/profiling` | Opt-in pprof HTTP server |
| Config | `internal/config` | Environment + secret file loading |

### 2.3 Data Flow

```
Claroty xDome API
    |  (POST with JSON body, Bearer token auth)
    v
claroty.HTTPClient.Fetch*()
    |  (decode JSON -> typed Go structs, handle polymorphic IDs)
    v
collector.collect*()
    |  (ensure forward progress, persist cursor + receipt)
    v
sink.HTTPSender.Send*()
    |  (wrap in EnrichedPayload with xMP metadata + optional OCSF)
    |  (POST with basic auth to Vector endpoint)
    v
Vector / Downstream
```

---

## 3. API Connection and Authentication

### 3.1 Authentication Mechanism

**Bearer Token Authentication.** Every request to the Claroty xDome API includes:

```
Authorization: Bearer <CLAROTY_API_KEY>
Content-Type: application/json
Accept: application/json
```

The token is loaded from `CLAROTY_API_KEY` env var or `CLAROTY_API_KEY_FILE` (Kubernetes secret mount). The config loader validates the key is non-empty at startup; missing key is a fatal error.

### 3.2 Base URL

Default: `https://api.claroty.com`. Override via `CLAROTY_BASE_URL` or `CLAROTY_BASE_URL_FILE`.

### 3.3 HTTP Client Configuration

- **Timeout**: 30 seconds per request (hardcoded in `runner.go` when constructing the client)
- **Transport**: Custom `transport.NewTransport` with:
  - TLS 1.2 minimum
  - HTTP/2 enabled
  - Connection pool: 100 idle, 10 per host, 20 max per host
  - Dial timeout: 10s
  - TLS handshake timeout: 10s
  - Response header timeout: 30s
  - Proxy from environment (HTTP_PROXY, HTTPS_PROXY)
  - Optional `InsecureSkipVerify` (not exposed via config currently)

### 3.4 Request Pattern

**All Claroty xDome endpoints use POST** (even for read-only queries). The request body is always JSON with this general shape:

```json
{
  "offset": 0,
  "limit": 100,
  "fields": ["field1", "field2", ...],
  "sort_by": [{"field": "timestamp", "order": "asc"}],
  "filter_by": { "field": "...", "operation": "greater", "value": "..." }
}
```

### 3.5 API Endpoints

| Data Source | Endpoint | JSON Response Key |
|-------------|----------|-------------------|
| Alerts | `POST /api/v1/alerts/` | `alerts` |
| OT Activity Events | `POST /api/v1/ot_activity_events/` | `ot_activity_events` |
| Audit Logs | `POST /api/v1/audit_log/get` | `audit_log` |
| Device-Alert Relations | `POST /api/v1/device_alert_relations/` | `devices_alerts` |
| Device-Vuln Relations | `POST /api/v1/device_vulnerability_relations/` | `devices_vulnerabilities` |
| Servers | `POST /api/v1/servers/` | `servers` |
| Sites | `POST /api/v1/sites/get` | `sites` |
| Devices | `POST /api/v1/devices/` | `devices` |
| Vulnerabilities | `POST /api/v1/vulnerabilities/` | `vulnerabilities` |

Note the non-standard paths: audit logs use `/audit_log/get`, sites use `/sites/get`.

---

## 4. Pagination Strategies

### 4.1 Timestamp + ID Cursor-Based (5 sources)

Used by: **Alerts, OT Activity Events, Audit Logs, Device-Alert Relations, Device-Vulnerability Relations**

These use composite cursors combining a timestamp field with one or more ID fields for tie-breaking. The filter_by clause uses an OR pattern:

```
(timestamp > cursor.timestamp)
OR
(timestamp == cursor.timestamp AND id > cursor.id)
```

For device-alert relations, the cursor is a 3-tuple: `(detected_time, alert_id, device_uid)`.
For device-vulnerability relations: `(detection_time, device_uid, vulnerability_id)`.

The `sort_by` clause ensures deterministic ordering. After fetching, results are re-sorted client-side for stability.

**Alerts sort**: `updated_time desc, id desc` (filter uses `greater` so this captures newer records)
**Events sort**: `detection_time desc, event_id desc`
**Audit logs**: offset-based with timestamp cursor
**Relations**: multi-field ascending sort

### 4.2 Offset-Based (4 sources)

Used by: **Servers, Sites, Devices, Vulnerabilities**

These use simple offset + secondary sort key:

| Source | Sort Key | Cursor Fields |
|--------|----------|--------------|
| Servers | `server_name` | `(offset, server_name)` |
| Sites | `id` | `(offset, site_id)` |
| Devices | `uid` | `(offset, device_uid)` |
| Vulnerabilities | `id` | `(offset, vulnerability_id)` |

Offset advances by `len(batch)` after each page. For vulnerabilities, a hardcoded filter `affected_devices_count > 0` is always applied.

### 4.3 Forward Progress Enforcement

Every `collect*` function calls an `ensure*ForwardProgress` function that compares the new cursor to the previous one. If the cursor did not advance (based on primary timestamp/offset, then secondary/tertiary ID fields), the function returns an error. This prevents infinite loops where the same page is re-fetched.

---

## 5. Data Models / Schemas

### 5.1 Alert

```go
type Alert struct {
    ID                     string      // Polymorphic: API returns string or number
    Name                   string      // alert_name
    TypeName               string      // alert_type_name
    Class                  string      // alert_class
    Category               string
    DetectedTime           time.Time
    UpdatedTime            time.Time
    DevicesCount           int
    UnresolvedDevicesCount int
    MedicalDevicesCount    int
    IoTDevicesCount        int
    ITDevicesCount         int
    OTDevicesCount         int
    Status                 string
    MitreTechniqueICSIDs   []string
    MitreTechniqueICSNames []string
    MitreTechniqueEntIDs   []string
    MitreTechniqueEntNames []string
    Description            string
    MaliciousIPTags        []string
}
```

**Fields requested** (20 fields): `id, alert_name, alert_type_name, alert_class, category, detected_time, updated_time, devices_count, unresolved_devices_count, medical_devices_count, iot_devices_count, it_devices_count, ot_devices_count, status, mitre_technique_ics_ids, mitre_technique_ics_names, mitre_technique_enterprise_ids, mitre_technique_enterprise_names, description, malicious_ip_tags_list`

### 5.2 Activity Event (OT)

```go
type ActivityEvent struct {
    EventID          string      // Polymorphic ID
    EventType        string
    Description      string
    Mode             string
    RelatedAlertIDs  []string    // Polymorphic array
    DetectionTime    time.Time
    DestAssetID      string
    DestIP           string
    DestDeviceType   string
    DestDeviceName   string
    DestSiteName     string
    DestNetwork      string
    Protocol         string
    DestPort         int         // Polymorphic: can be null/string/number
    SourcePort       int
    SourceAssetID    string
    SourceIP         string
    SourceDeviceType string
    SourceUsername   string
    SourceDeviceName string
    SourceSiteName   string
    SourceNetwork    string
    IPProtocol       string
}
```

**Fields requested** (23 fields): `event_id, event_type, description, mode, related_alert_ids, detection_time, dest_asset_id, dest_ip, dest_device_type, dest_device_name, dest_site_name, dest_network, protocol, dest_port, source_port, source_asset_id, source_ip, source_device_type, source_username, source_device_name, source_site_name, source_network, ip_protocol`

### 5.3 Audit Log

```go
type AuditLog struct {
    Action          string
    Category        string
    Details         string
    ID              string      // Polymorphic ID
    Timestamp       time.Time
    UserDisplayName string
    Username        string
}
```

**Fields requested** (7 fields): `action, category, details, id, timestamp, user_display_name, username`

### 5.4 Device-Alert Relation

```go
type DeviceAlertRelation struct {
    // Alert fields
    AlertID          string
    AlertName        string
    AlertClass       string
    AlertCategory    string
    AlertLabels      []string
    AlertAssignees   []string
    AlertTypeName    string
    AlertStatus      string
    AlertDescription string
    // Device fields (30+ fields)
    DeviceUID                    string
    DeviceName                   string
    DeviceCategory               string
    DeviceSubcategory            string
    DeviceType                   string
    DeviceAssignees              []string
    DeviceLabels                 []string
    DevicePurdueLevel            string
    DeviceSiteName               string
    DeviceSiteGroupName          string
    DeviceManufacturer           string
    DeviceRetired                bool
    DeviceRiskScore              string
    DeviceRiskScorePoints        float64
    DeviceIPList                 []string
    DeviceMacList                []string
    DeviceNetworkList            []string
    DeviceFirstSeen              []time.Time
    DeviceLastSeen               []time.Time
    // MITRE fields
    MitreTechniqueEnterpriseNames []string
    MitreTechniqueEnterpriseIDs   []string
    MitreTechniqueICSNames        []string
    MitreTechniqueICSIDs          []string
    // Timestamps
    DeviceAlertStatus        string
    DeviceAlertDetectedTime  time.Time
    DeviceAlertUpdatedTime   time.Time
    // Risk subscores (strings + float64 points)
    // ... 12 more risk/likelihood/impact fields
}
```

**Fields requested**: 44 fields (the most field-heavy endpoint).

### 5.5 Device-Vulnerability Relation

```go
type DeviceVulnerabilityRelation struct {
    DeviceUID                           string
    DeviceCategory                      string
    DeviceSubcategory                   string
    DeviceType                          string
    DeviceRiskScore                     string
    DeviceAssignees                     []string
    DeviceLabels                        []string
    DeviceSiteName                      string
    VulnerabilityID                     string
    VulnerabilityName                   string
    VulnerabilityType                   string
    VulnerabilityCVEIDs                 []string
    VulnerabilityAdjustedScore          float64     // json.RawMessage -> float
    VulnerabilityEPSSScore              float64     // json.RawMessage -> float
    VulnerabilitySources                []VulnerabilitySource
    VulnerabilityDescription            string
    VulnerabilityAffectedProducts       string
    VulnerabilityRecommendations        string
    VulnerabilityIsKnownExploited       bool
    VulnerabilityPublishedDate          time.Time
    VulnerabilityLabels                 []string
    VulnerabilityAssignees              []string
    VulnerabilityNote                   string
    VulnerabilityLastUpdated            time.Time
    VulnerabilityRelevance              string
    VulnerabilityRelevanceSources       []string
    DeviceVulnerabilityDetectionDate    time.Time
    DeviceVulnerabilityResolutionDate   time.Time
    DeviceVulnerabilityDaysToResolution float64     // json.RawMessage -> float
    PatchInstallDate                    time.Time
}
```

**Fields requested**: 30 fields.

### 5.6 Server

```go
type Server struct {
    ServerName              string
    ServerLocation          string
    ServerStatus            string
    Model                   string
    OSVersion               string
    SerialNumber            string
    NumOfInterfaces         int
    ManagementIP            string
    IdracIP                 string
    ManagementMAC           string
    UptimeDays              float64
    AvgTrafficPastMonthMbps float64
    AvgTrafficPastWeekMbps  float64
    AvgTrafficPastHourMbps  float64
    NumOfOpenIncidents      float64
    Notes                   string
}
```

**Fields requested**: 16 fields.

### 5.7 Site

```go
type Site struct {
    ID                        int
    Name                      string
    Location                  string
    Timezone                  string
    CountryCode               string
    Description               string
    HospitalType              string
    NumberOfBeds              int
    DevicesCount              int
    SiteAttributionRulesCount int
    SiteGroupID               int
    SiteGroupName             string
}
```

**Fields requested**: 12 fields.

### 5.8 Device

```go
type Device struct {
    AssetID           string
    Assignees         []string
    DeviceCategory    string
    DeviceSubcategory string
    DeviceType        string
    DeviceTypeFamily  string
    IPList            []string
    Labels            []string
    MACList           []string
    Model             string
    NetworkList       []string
    OSCategory        string
    Retired           bool
    RiskScore         string
    UID               string
    VLANList          []int
}
```

**Fields requested**: 16 fields (`network_list, device_category, device_subcategory, device_type, uid, detector_name, asset_id, mac_list, ip_list, device_type_family, assignees, labels, model, os_category, retired, risk_score, vlan_list` -- note `detector_name` is requested but not in the struct).

### 5.9 Vulnerability

```go
type Vulnerability struct {
    ID                                      string
    Name                                    string
    VulnerabilityType                       string
    CVEIDs                                  []string
    CVSSV2Score                             *float64    // nullable
    CVSSV2ExploitabilitySubscore            *float64
    CVSSV2VectorString                      string
    CVSSV3Score                             *float64
    CVSSV3ExploitabilitySubscore            *float64
    CVSSV3VectorString                      string
    Sources                                 []VulnerabilitySource
    SourceName                              string
    SourceURL                               string
    Description                             string
    AffectedProducts                        string
    Recommendations                         string
    IsKnownExploited                        bool
    AffectedDevicesCount                    int
    AffectedIoTDevicesCount                 int
    AffectedITDevicesCount                  int
    AffectedOTDevicesCount                  int
    PublishedDate                           time.Time
    AffectedFixedDevicesCount               int
    AffectedConfirmedDevicesCount           int
    AffectedPotentiallyRelevantDevicesCount int
    AffectedIrrelevantDevicesCount          int
    AdjustedVulnerabilityScore              float64
    AdjustedVulnerabilityScoreLevel         string
    ExploitsCount                           string
    VulnerabilityLabels                     []string
    VulnerabilityAssignees                  []string
    VulnerabilityNote                       string
    VulnerabilityPriorityGroup              string
    EPSSScore                               float64
}
```

**Fields requested**: 35 fields. Always filtered to `affected_devices_count > 0`.

---

## 6. Sink / Output Format

### 6.1 Enriched Payload Envelope

Every record sent to the downstream sink is wrapped in an `EnrichedPayload`:

```json
{
  "data": { /* original Claroty record as JSON */ },
  "record_type": "alert|activity_event|audit_log|device_alert_relation|device_vulnerability_relation|server|site|device|vulnerability",
  "xmp": {
    "site": "XMP_SITE value",
    "cluster_name": "XMP_CLUSTER_NAME value",
    "node_name": "XMP_NODE_NAME or hostname"
  },
  "ocsf": { /* optional OCSF Detection Finding, only for alerts when enabled */ }
}
```

### 6.2 Sink Transport

- **Method**: POST to `VECTOR_ENDPOINT` (default `http://localhost:4413`)
- **Auth**: HTTP Basic Auth (`VECTOR_USERNAME`/`VECTOR_PASSWORD`, default `xdome`/`xdome`)
- **Content-Type**: `application/json`
- **Timeout**: `VECTOR_TIMEOUT_SECONDS` (default 15s)
- **Transport**: Same TLS 1.2+ config as the Claroty client
- **Per-record delivery**: Each record is sent individually (no batching at the sink level)
- **Error handling**: Status >= 400 returns error with body preview (up to 2048 bytes)

### 6.3 Record Types (9 total)

`alert`, `activity_event`, `audit_log`, `device_alert_relation`, `device_vulnerability_relation`, `server`, `site`, `device`, `vulnerability`

---

## 7. Error Handling and Retry Patterns

### 7.1 Collection Loop Retry

The main `Run()` loop implements exponential backoff:

```
baseDelay = 2s (configurable)
maxDelay = 30s (configurable)
maxRetries = 5 (configurable, 0 = unlimited)

on error:
  retryCount++
  if retryCount > maxRetries: return ErrCollectorRetriesExceeded
  wait retryDelay (context-aware)
  retryDelay *= 2 (capped at maxDelay)

on success:
  reset retryCount = 0
  reset retryDelay = baseDelay
  if hasMore: continue immediately (no sleep)
  else: wait for ticker (30s interval)
```

### 7.2 Health-Aware Retry

On error, the health reporter is set to `NotReady`, causing the `/ready` endpoint to return 503. On success, it's set back to `Ready`. The `/health` (liveness) endpoint remains healthy unless the server is shutting down.

### 7.3 Sentinel Errors

The `apperrors` package defines 13 sentinel errors for structured error handling:

| Error | Meaning |
|-------|---------|
| `ErrStateNotFound` | No persisted state exists (triggers bootstrap) |
| `ErrQueryFingerprintMismatch` | Config changed since last run (fatal) |
| `ErrCursorRegression` | Cursor failed to advance (fatal for that batch) |
| `ErrCollectorRetriesExceeded` | Retry budget exhausted (fatal) |
| `ErrCollectorStateLoad` | Could not load state from store |
| `ErrCollectorStatePersist` | Could not save state to store |
| `ErrClarotyConfigMissing` | Missing API key or base URL |
| `ErrClarotyRequestBuild` | Could not construct HTTP request |
| `ErrClarotyRequestExec` | HTTP request failed |
| `ErrClarotyUnexpectedStatus` | Non-200 response from Claroty |
| `ErrClarotyDecode` | JSON decode failure |
| `ErrSinkConfigMissing` | Missing sink endpoint or credentials |
| `ErrSinkRequestBuild` | Could not construct sink request |
| `ErrSinkDelivery` | Sink rejected or failed |

### 7.4 Error Wrapping Pattern

All errors use `fmt.Errorf("%w: %v", sentinel, cause)` for wrapping with context. This allows callers to `errors.Is(err, sentinel)` for classification.

### 7.5 Context Cancellation

- `signal.NotifyContext` handles SIGTERM/SIGINT
- Every HTTP request uses `context.WithTimeout` (30s for Claroty client)
- The `waitWithContext` helper ensures retry sleeps are cancellable
- Graceful shutdown: health server gets 5s shutdown window

---

## 8. Configuration and Credential Management

### 8.1 Configuration Loading Pattern

1. `DefaultConfig()` provides hardcoded defaults
2. `LoadFromEnvironment(cfg)` overlays environment variables
3. For every sensitive field, there's a `*_FILE` variant checked first (for Kubernetes secret mounts)
4. The `readSecretFile(path)` helper reads + trims the file, returning empty string for non-existent files

### 8.2 Secret File Support

Every credential supports dual loading:

| Direct Env | File Env | Purpose |
|-----------|----------|---------|
| `CLAROTY_API_KEY` | `CLAROTY_API_KEY_FILE` | Bearer token |
| `CLAROTY_BASE_URL` | `CLAROTY_BASE_URL_FILE` | API base URL |
| `VECTOR_ENDPOINT` | `VECTOR_ENDPOINT_FILE` | Sink URL |
| `VECTOR_USERNAME` | `VECTOR_USERNAME_FILE` | Basic auth user |
| `VECTOR_PASSWORD` | `VECTOR_PASSWORD_FILE` | Basic auth password |
| `OCSF_BASE_URL` | `OCSF_BASE_URL_FILE` | OCSF context URL |
| `OCSF_TENANT_UID` | `OCSF_TENANT_UID_FILE` | OCSF tenant ID |

File variant takes precedence over direct env var.

### 8.3 Key Configuration Values

| Config | Default | Notes |
|--------|---------|-------|
| Polling interval | 30s | `Collector.Interval` |
| Retry base delay | 2s | Exponential backoff start |
| Retry max delay | 30s | Backoff cap |
| Max retries | 5 | 0 = unlimited |
| Page size (all sources) | 100 | Per `*Limit` config |
| Reconcile window | 24h | Lookback for reconciliation |
| State file path | `/var/lib/poller-bear/state.json` | `STATE_STORE_PATH` |
| Max receipts per source | 100 | `STATE_STORE_MAX_RECEIPTS` |
| Health address | `:7321` | `COLLECTOR_HEALTH_ADDR` |

---

## 9. State Persistence

### 9.1 State Structure

The file store persists a single JSON file containing:

- Per-source poll state (cursor + query fingerprint + version + updated_at) for all 9 sources
- Per-source batch receipt history (bounded by `maxReceipts`, default 100)
- Global `last_updated` timestamp

### 9.2 Query Fingerprinting

Before starting, the collector computes a SHA-256 hash of `sorted(fields) + "|" + limit` for each data source. If the stored hash doesn't match the current config, the collector fails with `ErrQueryFingerprintMismatch`. This prevents silently using stale cursors when field lists change.

### 9.3 Atomic File Writes

The `FileStore.persist()` method uses write-to-temp + fsync + rename for crash safety:

1. Create temp file in same directory (`.poller-state-*.tmp`)
2. Write JSON
3. `Sync()` to disk
4. `Rename()` atomically over the state file
5. On any failure, clean up temp file

### 9.4 Batch Receipts

Each successful batch saves a receipt containing: version, request hash, count, first/last IDs, fetch timestamp, and the cursor that was applied. Receipts are trimmed to the most recent N per source (default 100) to prevent unbounded growth.

---

## 10. OCSF Normalization

### 10.1 Status

OCSF mapping is **partially implemented**. The types and severity pipeline are in place, but the actual mapper (`mapOCSF` in `sink/http_sender.go`) is a TODO stub that always returns nil. Only alert records would be mapped.

### 10.2 Detection Finding (Class 2004) Structure

The `ocsf.DetectionFinding` struct maps to ~35 OCSF fields including:
- `activity_id`, `category_uid` (2), `class_uid` (2004), `type_uid` (200401)
- `severity_id` (mapped from vendor severity)
- `finding_info` (uid, title, timestamps)
- `metadata.product` (Claroty xDome)
- `resources` (affected endpoints/devices)
- `attacks` (MITRE ATT&CK tactic/technique)
- `observables`, `risk_score`, `unmapped`

### 10.3 Severity Mapping

Embedded YAML (`severity-map.yaml`) maps Claroty severity strings to OCSF IDs:

| Claroty | OCSF severity_id |
|---------|------------------|
| Critical | 5 |
| High | 4 |
| Medium | 3 |
| Low | 2 |
| Info | 1 |
| (unknown) | 0 (fallback) |

Status mapping: New/Open -> 1, InProgress -> 2, Resolved/Closed -> 4.

A per-alert-type adjustment system exists but is currently empty (no adjustments defined).

### 10.4 Golden File Testing

Test fixtures in `internal/ocsf/testdata/` define input Claroty alerts and expected OCSF output for validation. Three test cases cover high, critical (with MITRE), and low (no endpoints) scenarios.

---

## 11. Known API Quirks and Type Handling

### 11.1 Polymorphic ID Types

The Claroty API inconsistently returns IDs as strings or numbers. The client handles this with a two-phase decode:

```go
// Try string first
var id string
if err := json.Unmarshal(raw, &id); err != nil {
    // Fall back to number
    var number json.Number
    json.Unmarshal(raw, &number)
    id = number.String()
}
```

This pattern is used for: `alert.id`, `audit_log.id`, `event_id`, `related_alert_ids[]`.

### 11.2 Polymorphic Numeric Fields

Some fields (`dest_port`, `source_port`, `vulnerability_adjusted_vulnerability_score`, `epss_score`, `device_vulnerability_days_to_resolution`) use `json.RawMessage` decoding to handle null/string/number variants via `parseClarotyFloat()` and `parseClarotyString()`.

### 11.3 Nullable Fields

Time fields silently default to zero time on parse failure. Float fields silently default to 0. This is a defensive pattern that avoids failing on missing data.

### 11.4 Alert ID in Relations

In `DeviceAlertRelation`, the `alert_id` comes from the API as `int` but is stored as `string` via `strconv.Itoa()`. When building cursor filters, the client checks if the stored string AlertID can be parsed as int and sends the numeric form to the API.

---

## 12. Behavioral Contracts

### BC-001: Startup Requires API Key

**Preconditions**: `CLAROTY_API_KEY` or `CLAROTY_API_KEY_FILE` must resolve to a non-empty string.
**Postconditions**: If missing, `LoadFromEnvironment` returns error; runner exits with code 1.
**Confidence**: HIGH (explicit validation in config.go line 463)

### BC-002: Query Fingerprint Mismatch is Fatal

**Preconditions**: Stored state exists with a hash from a previous config.
**Postconditions**: If any source's stored hash differs from current config hash, initialization fails. User must delete the state file.
**Confidence**: HIGH (checked for all 9 sources in initialize*State methods)

### BC-003: Forward Progress is Enforced

**Preconditions**: A batch of records is fetched and non-empty.
**Postconditions**: The new cursor must be strictly greater than the previous cursor (by timestamp, then ID fields). If not, the batch is rejected with an error.
**Confidence**: HIGH (9 separate ensure*ForwardProgress functions, tested in collector_test.go)

### BC-004: At-Least-Once Delivery

**Preconditions**: Records are fetched successfully.
**Postconditions**: Each record is sent to the sink individually. Only after all records in a batch are sent does the cursor advance. On failure mid-batch, the cursor is not advanced, so the entire batch will be re-fetched on retry.
**Confidence**: HIGH (sink.Send* called in loop before state.Save*)

### BC-005: Graceful Shutdown on SIGTERM/SIGINT

**Preconditions**: Process receives SIGTERM or SIGINT.
**Postconditions**: Context is cancelled, in-flight HTTP requests are cancelled, health server gets 5s shutdown window, pprof server gets 5s shutdown window.
**Confidence**: HIGH (signal.NotifyContext in runner.go, deferred shutdown in main.go)

### BC-006: Exponential Backoff with Bounded Retries

**Preconditions**: collectOnce() returns an error.
**Postconditions**: Wait `retryDelay` (starting at 2s, doubling up to 30s). If `maxRetries` (5) exceeded, return fatal error.
**Confidence**: HIGH (implemented in collector.Run, tested in collector_test.go)

### BC-007: Health Readiness Tracks Collection Status

**Preconditions**: Health server is running.
**Postconditions**: `/ready` returns 200 only when the collector has successfully completed at least one cycle after initialization. Returns 503 during initialization and after errors.
**Confidence**: HIGH (SetReady/SetNotReady calls in collector.Run)

### BC-008: Atomic State File Writes

**Preconditions**: State needs to be persisted.
**Postconditions**: State is written to a temp file, synced, then renamed. No partial writes can corrupt the state file.
**Confidence**: HIGH (FileStore.persist implementation)

### BC-009: Records are Enriched with xMP Metadata

**Preconditions**: Any record is sent to sink.
**Postconditions**: The record is wrapped in `EnrichedPayload` with `data`, `record_type`, and `xmp` fields. OCSF field is present only for alerts when enabled.
**Confidence**: HIGH (enrichPayload called for every sendPayload, tested in sink tests)

### BC-010: Vulnerability Filter

**Preconditions**: FetchVulnerabilities is called.
**Postconditions**: The request always includes `filter_by: {field: "affected_devices_count", operation: "greater", value: 0}`.
**Confidence**: HIGH (hardcoded in FetchVulnerabilities)

---

## 13. Conventions and Patterns

### 13.1 Error Handling

- Sentinel errors defined in `apperrors` package
- Wrapped with `fmt.Errorf("%w: %v", sentinel, cause)`
- Switch-case on `errors.Is()` for state initialization (not found vs. other errors)
- Deferred cleanup with `defer resp.Body.Close()`

### 13.2 Interface-Driven Design

- `claroty.Client` interface (9 Fetch* methods) -- allows faking in tests
- `sink.Sender` interface (9 Send* methods) -- allows faking in tests
- `state.Store` interface (composite of 9 sub-interfaces) -- allows memory/file swap
- `health.Reporter` interface (SetReady/SetNotReady) -- allows nil or mock

### 13.3 Testing Patterns

- Table-driven tests (config_test.go, claroty/http_client_test.go)
- Fake implementations preferred over mocks (`fakeClarotyClient`, `fakeSink` in collector_test.go)
- `go:generate mockgen` available for interfaces but fakes are primary
- `httptest.NewServer` for HTTP client testing
- `t.TempDir()` for file store tests
- Golden file tests for OCSF mapping
- Benchmarks for hot paths (http_client_bench_test.go, file_store_bench_test.go, http_sender_bench_test.go)

### 13.4 Naming Conventions

- Go standard: PascalCase exports, camelCase internal
- Package names: lowercase, short, descriptive
- Test files: `*_test.go` co-located with implementation
- Config env vars: SCREAMING_SNAKE_CASE
- JSON tags: snake_case matching Claroty API field names
- Struct fields mirror the Claroty API naming but in Go style (e.g., `device_uid` -> `DeviceUID`)

### 13.5 Code Generation

- `go:generate` directives for mock generation via `mockgen`
- `go:embed` for OCSF YAML data files

---

## 14. Deployment Topology

### 14.1 Container

- Multi-stage Dockerfile: Go 1.26.1 build, distroless runtime (nonroot user)
- Single static binary: `/app/collector`
- No shell, no package manager in runtime image

### 14.2 Kubernetes (Helm)

- Helm chart at `deploy/helm/poller-bear/`
- Resources: Deployment, Service, ServiceAccount, RBAC, PVC, Secret
- PVC for state file persistence across pod restarts
- Secrets for Claroty API key, Vector credentials
- Health probes on port 7321

### 14.3 Local Development

- `make run`: builds and runs locally
- `make vector`: runs Vector with `vector.yaml` for local sink testing
- Vector config: HTTP server on :4413 with basic auth, console sink for debugging

---

## 15. Legacy Python Implementation

The `legacy/` directory preserves the pre-Go Python implementation. Key differences:

- **Alerts only**: The Python version only polled alerts (the Go version added 8 more data sources)
- **No state persistence**: Used `PollMeta.last_poll_ts` in memory, no durable cursor
- **Threaded**: Used `threading.Thread` + `queue.Queue` for producer-consumer
- **Simpler pagination**: Basic offset-based, no cursor-based pagination
- **Same API pattern**: Bearer token auth, POST with JSON body, same alert fields
- **Same sink pattern**: POST to Vector with basic auth

---

## 16. Key Design Decisions for Prism MCP Server

### What to preserve in Rust:

1. **Bearer token auth pattern** -- simple, well-defined
2. **POST-with-JSON-body for all endpoints** -- unusual but consistent
3. **Composite cursor pagination** -- the dual timestamp+ID cursor with OR filters is the core sophistication
4. **Forward progress enforcement** -- prevents infinite loops
5. **Query fingerprinting** -- detects config drift; could be simplified
6. **Atomic state persistence** -- critical for reliability
7. **Polymorphic ID handling** -- the API sends mixed types, must be handled
8. **Per-record sink delivery with batch-level cursor commit** -- at-least-once semantics

### What to reconsider:

1. **9 separate collect functions** with near-identical structure -- could be generalized with a trait/generic
2. **9 separate state store methods** -- heavy interface surface; could use a generic store
3. **Per-record HTTP POST to sink** -- could batch for efficiency
4. **OCSF mapping is a stub** -- decide if this belongs in the MCP server or downstream
5. **No rate limiting toward Claroty API** -- relies on page size and interval
6. **No HTTP status-code-aware retry** (e.g., 429 handling) -- the retry is only at the collection loop level
7. **30s hardcoded client timeout** -- should be configurable

### API Response Keys (for parsing):

| Endpoint | Response JSON Key |
|----------|------------------|
| `/api/v1/alerts/` | `alerts` |
| `/api/v1/ot_activity_events/` | `ot_activity_events` |
| `/api/v1/audit_log/get` | `audit_log` |
| `/api/v1/device_alert_relations/` | `devices_alerts` |
| `/api/v1/device_vulnerability_relations/` | `devices_vulnerabilities` |
| `/api/v1/servers/` | `servers` |
| `/api/v1/sites/get` | `sites` |
| `/api/v1/devices/` | `devices` |
| `/api/v1/vulnerabilities/` | `vulnerabilities` |

---

## 17. Gaps and Open Questions

1. **No circuit breaker** -- if Claroty API is down, the collector hammers it every 2-30s (with backoff, but no circuit break)
2. **No metrics/telemetry export** -- logs only, no Prometheus/OpenTelemetry
3. **OCSF mapper is a TODO** -- stub returns nil for all records
4. **ReconcileWindow (24h)** is configured but never referenced in collection logic -- unclear if it's implemented
5. **detector_name** is in the device fields list but not in the `Device` struct -- it's requested from the API but silently dropped during decode
6. **No deduplication** -- at-least-once means duplicates are possible on restart
7. **Single-threaded collection** -- all 9 sources are polled sequentially within `collectOnce()`; no parallelism
8. **Vulnerability filter is hardcoded** -- `affected_devices_count > 0` cannot be overridden

---

## Appendix A: File Manifest

| File | Lines | Purpose |
|------|-------------|---------|
| `main.go` | 37 | Process entry, pprof setup |
| `cmd/collector/main.go` | 14 | Alternative entry |
| `internal/app/runner/runner.go` | 150 | Orchestration, signal handling |
| `internal/config/config.go` | 597 | Config structs + env loading |
| `internal/claroty/api.go` | 475 | Domain types + Client interface |
| `internal/claroty/http_client.go` | 1836 | All 9 Fetch methods + decode helpers |
| `internal/collector/collector.go` | 1367 | Polling loop + 9 collect/initialize methods |
| `internal/state/store.go` | 361 | State types + Store interface + fingerprinting |
| `internal/state/file_store.go` | 431 | File-based state persistence |
| `internal/state/memory_store.go` | 302 | In-memory state (testing) |
| `internal/sink/sink.go` | 25 | Sender interface |
| `internal/sink/http_sender.go` | 251 | HTTP delivery + enrichment |
| `internal/ocsf/detection_finding.go` | 89 | OCSF structs |
| `internal/ocsf/severity.go` | 17 | Severity normalization |
| `internal/ocsf/config.go` | 97 | OCSF config from embedded YAML |
| `internal/transport/http.go` | 145 | HTTP transport config |
| `internal/health/server.go` | 72 | Health check server |
| `internal/profiling/pprof.go` | 106 | Optional pprof |
| `internal/apperrors/errors.go` | 54 | Sentinel errors |
