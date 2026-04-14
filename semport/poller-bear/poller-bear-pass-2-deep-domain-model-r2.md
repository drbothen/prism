# Pass 2 Deep: Domain Model -- Round 2

> Project: poller-bear
> Source: /Users/jmagady/Dev/prism/.references/poller-bear/
> Round: 2

---

## Audit of Round 1 Claims

### Over-extrapolated lists: NONE FOUND
All entity field counts were verified line-by-line against `api.go`. No inflation detected.

### Miscounted enumerations: ONE CORRECTION
- Round 1 claimed "Vulnerability: 34 fields". Re-count of `api.go` lines 340-375: `ID, Name, VulnerabilityType, CVEIDs, CVSSV2Score, CVSSV2ExploitabilitySubscore, CVSSV2VectorString, CVSSV3Score, CVSSV3ExploitabilitySubscore, CVSSV3VectorString, Sources, SourceName, SourceURL, Description, AffectedProducts, Recommendations, IsKnownExploited, AffectedDevicesCount, AffectedIoTDevicesCount, AffectedITDevicesCount, AffectedOTDevicesCount, PublishedDate, AffectedFixedDevicesCount, AffectedConfirmedDevicesCount, AffectedPotentiallyRelevantDevicesCount, AffectedIrrelevantDevicesCount, AdjustedVulnerabilityScore, AdjustedVulnerabilityScoreLevel, ExploitsCount, VulnerabilityLabels, VulnerabilityAssignees, VulnerabilityNote, VulnerabilityPriorityGroup, EPSSScore` = **34 fields confirmed**.

### Pattern conflation: NONE FOUND
Cursor type duplication between `claroty` and `state` packages correctly identified as separate Go types.

### Basename conflation: NONE FOUND
`AlertCursor` in `claroty` package and `AlertCursor` in `state` package are correctly identified as distinct types.

### Dead sentinel verification
Confirmed: `ErrCursorRegression` is defined in `apperrors/errors.go` line 17 but searching for its usage:
- Not referenced in any `fmt.Errorf("%w:` pattern
- Not referenced in any `errors.Is()` check
- All 9 `ensure*ForwardProgress` functions use plain `fmt.Errorf("... cursor did not advance: ...")` without wrapping `ErrCursorRegression`
- This is indeed a **dead sentinel** -- defined for future use but currently unused

---

## New Discoveries: Polymorphic Decode Helpers

The `claroty/http_client.go` contains three decode helper functions that handle the Claroty API's type inconsistencies. These are critical domain translation functions:

### parseClarotyFloat (line 292)

**Input:** `json.RawMessage` (could be number, null, empty)
**Output:** `(float64, error)`
**Behavior:**
- `null` or empty -> 0.0, nil
- Valid number -> parsed float64, nil
- String "not a number" -> 0, error
- Invalid JSON -> 0, error

**Test coverage:** 8 table-driven cases in `TestParseClarotyFloat` (http_client_test.go:109-183)

### parseClarotyString (line 559)

**Input:** `json.RawMessage` (could be string, number, boolean, null, empty)
**Output:** `(string, error)`
**Behavior:**
- Quoted string -> trimmed string, nil
- Number (42, 3.14) -> string representation ("42", "3.14"), nil
- Boolean (true/false) -> "true"/"false", nil
- `null` or empty -> "", nil
- Array or object -> "", error

**Test coverage:** 10 table-driven cases in `TestParseClarotyString` (http_client_test.go:185-269)

### parseClarotyStringList (line 582)

**Input:** `json.RawMessage` (could be array of strings, single string, null, empty)
**Output:** `(string, error)` -- comma-joined
**Behavior:**
- String array -> comma-joined ("foo,bar,baz"), nil
- Single element array -> that element, nil
- Empty array -> "", nil
- Single string -> trimmed string, nil
- `null` or empty -> "", nil
- Object -> "", error

**Test coverage:** 8 table-driven cases in `TestParseClarotyStringList` (http_client_test.go:271-345)

---

## New Discoveries: Decode Functions for Entities

The `http_client_test.go` reveals decode functions that handle the polymorphic ID pattern:

### decodeAlert

**Behavior:**
- String ID ("alert-123") -> stored as-is
- Numeric ID (456) -> converted to string "456"
- Missing ID field -> error
- Malformed JSON -> error
- Timestamps parsed as RFC3339Nano
- Device counts parsed as integers

**Test coverage:** `TestDecodeAlert` (5 cases), `TestDecodeAlert_TimestampParsing`, `TestDecodeAlert_DeviceCounts`

### decodeAuditLog

**Behavior:**
- String ID -> stored as-is
- Numeric ID (789) -> converted to string "789"
- Malformed JSON -> error wrapping ErrClarotyDecode

**Test coverage:** `TestDecodeAuditLog` (3 cases) -- notably verifies `errors.Is(err, apperrors.ErrClarotyDecode)`

### decodeActivityEvent

**Behavior:**
- String EventID -> stored as-is
- Numeric EventID (999) -> converted to string "999"
- `dest_port: null` -> DestPort = 0
- `dest_port: 80` -> DestPort = 80
- Malformed JSON -> error

**Test coverage:** `TestDecodeActivityEvent` (3 cases)

---

## New Discoveries: OCSF Stub Mapper (Test-Only)

The `ocsf/mapper_stub_test.go` contains `stubMapAlert()` -- a test-only implementation of the alert-to-OCSF mapping that will be replaced by real `MapAlert()` in story 3-2.1.

### stubMapAlert Behavior

**Input:** Raw JSON Claroty alert + OCSF Config
**Output:** `*DetectionFinding`
**Mapping rules:**
- `ActivityID` = 1 (Create)
- `CategoryUID` = 2 (Findings)
- `ClassUID` = 2004
- `TypeUID` = 200401 (Detection Finding: Create)
- `SeverityID` = `cfg.NormalizeSeverity(alert.Severity)`
- `StatusID` = `cfg.StatusMap[alert.Status]` or fallback
- `ConfidenceID` = 1 (Low, stub default)
- `Time` = parsed RFC3339 timestamp or fixed fallback epoch
- `FindingInfo.UID` = `"claroty-" + alert.ID`
- `Metadata.Product` = "Claroty xDome" / "Claroty"
- `Metadata.Version` = "1.7.0"
- Resources mapped from `alert.Source` (if present)
- Attacks mapped from `alert.MITRE` array (if present)

### OCSF Test Input Structure (test-only types)

The `clarotyAlert` test type differs from the production `claroty.Alert` struct:
- Test type: `ID, Type, Severity, Message, Timestamp, Status, Source, MITRE`
- Production type: `ID, Name, TypeName, Class, Category, DetectedTime, UpdatedTime, ...`

This means the golden file tests validate the OCSF output structure but use a simplified input schema. The real mapper will need to adapt to the production `claroty.Alert` struct.

### Golden File Tests

`TestMapAlert_GoldenFiles` validates that `stubMapAlert()` output matches expected golden files in `testdata/golden/alert-*.json`.

`TestMapAlert_SchemaValidation` validates golden file output against the OCSF Detection Finding 2004 JSON schema at `ocsf-schema/detection-finding-2004.json`.

---

## New Discoveries: OCSF Integration in Sink

### OCSF Enable/Disable Toggle

From `http_sender_ocsf_test.go`:

| Scenario | OCSF Key in JSON? | Evidence |
|----------|-------------------|----------|
| OCSFConfig.Enabled = false | Absent | `TestEnrichPayload_OCSFDisabled_NoField` |
| OCSFConfig.Enabled = true, mapper returns nil | Absent (omitempty) | `TestEnrichPayload_OCSFEnabled_StubNil_NoField` |
| OCSFConfig.Enabled = true, non-alert record type | Absent | `TestEnrichPayload_OCSFEnabled_NonAlertRecordTypes` (8 sub-tests) |
| mapOCSF receives wrong payload type | Returns nil (no panic) | `TestMapOCSF_WrongPayloadType` |
| mapOCSF panics | Recovery, returns nil | `TestMapOCSF_PanicRecovery` (skipped -- deferred to story 3-2.1) |

### OCSF_ENABLED Parsing

From `TestOCSFConfig_ParseEnabled`: Follows `strconv.ParseBool` semantics:
- "true", "1", "TRUE" -> enabled
- "false", "0" -> disabled
- "" (unset) -> disabled
- "yes", "on" -> disabled with warning

---

## New Discovery: Client Construction Contracts

### HTTPClient Construction

From `http_client_test.go`:
- Missing BaseURL -> `ErrClarotyConfigMissing`
- Invalid BaseURL -> `ErrClarotyRequestBuild`
- Timeout = 0 -> defaults to 30s
- Token is trimmed of whitespace (including newlines)

**Note**: Missing Token is NOT validated at construction time (the `NewHTTPClient` test passes with `Token: "token"` but there's no test for empty token). Token validation may happen at request time or not at all -- this is a gap.

---

## Refined: Complete Type Taxonomy

Total Go types in the domain model (verified count):

| Category | Count | Types |
|----------|-------|-------|
| Data entities | 9 | Alert, ActivityEvent, AuditLog, DeviceAlertRelation, DeviceVulnerabilityRelation, Server, Site, Device, Vulnerability |
| Value objects | 5 | QueryFingerprint, VulnerabilitySource, EnrichedPayload, XMPMetadata, Adjustment |
| Claroty cursors | 9 | AlertCursor, EventsCursor, AuditLogCursor, DeviceAlertRelationsCursor, DeviceVulnerabilityRelationsCursor, ServerCursor, SiteCursor, DeviceCursor, VulnerabilityCursor |
| State cursors | 9 | AlertCursor, EventCursor, AuditLogCursor, DeviceAlertRelationCursor, DeviceVulnerabilityRelationCursor, ServerCursor, SiteCursor, DeviceCursor, VulnerabilityCursor |
| Request types | 9 | AlertsRequest, EventsRequest, AuditLogRequest, DeviceAlertRelationsRequest, DeviceVulnerabilityRelationsRequest, ServersRequest, SitesRequest, DevicesRequest, VulnerabilitiesRequest |
| Batch types | 9 | AlertsBatch, EventsBatch, AuditLogBatch, DeviceAlertRelationsBatch, DeviceVulnerabilityRelationsBatch, ServersBatch, SitesBatch, DevicesBatch, VulnerabilitiesBatch |
| Poll state types | 9 | AlertPollState, EventPollState, AuditLogPollState, DeviceAlertRelationPollState, DeviceVulnerabilityRelationPollState, ServerPollState, SitePollState, DevicePollState, VulnerabilityPollState |
| Receipt types | 9 | AlertBatchReceipt, EventBatchReceipt, AuditLogBatchReceipt, DeviceAlertRelationBatchReceipt, DeviceVulnerabilityRelationBatchReceipt, ServerBatchReceipt, SiteBatchReceipt, DeviceBatchReceipt, VulnerabilityBatchReceipt |
| OCSF types | 11 | DetectionFinding, FindingInfo, Metadata, Product, Evidence, Resource, Endpoint, Attack, AttackComponent, Observable, Config |
| Config types | 9 | Config, ClarotyConfig, CollectorConfig, SinkConfig, LoggingConfig, XMPConfig, StateConfig, OCSFConfig, StoreType |
| Infrastructure | 7 | Collector, HTTPClient (claroty), HTTPSender (sink), FileStore, MemoryStore, Server (health), Options |
| Internal (API) | 3 | sortClause, filterClause, compoundFilter |
| Sentinel errors | 15 | (see Round 1) |
| **Total** | **~113** | |

---

## Delta Summary
- New items added: 3 polymorphic decode helpers documented, 3 entity decode functions documented, OCSF stub mapper behavior mapped, OCSF toggle behavior in sink fully characterized, client construction contracts, parseClarotyStringList function
- Existing items refined: Vulnerability field count re-confirmed at 34, dead sentinel re-confirmed, total type taxonomy compiled (~113 types)
- Remaining gaps: Token validation gap at client construction, Server/Site specific decode functions not analyzed (but follow same patterns)

## Novelty Assessment
Novelty: **NITPICK**
The polymorphic decode helpers, OCSF stub behavior, and client construction contracts are refinements of patterns already identified in Round 1. The decode helpers follow the same pattern documented in Round 1's "Polymorphic Type Handling" section. The OCSF integration details confirm the stub nature already documented. The total type count is useful but confirmatory. None of these discoveries change how the system would be spec'd -- they add implementation detail but no new architectural or behavioral insights.

## Convergence Declaration
Pass 2 has converged -- findings are nitpicks, not gaps. The domain model is comprehensively characterized with 9 data entities, 9x repeated cursor/batch/request/state/receipt patterns, 3 polymorphic decode helpers, and a well-defined OCSF stub awaiting implementation.

## State Checkpoint
```yaml
pass: 2
round: 2
status: complete
files_scanned: 22
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
