---
document_type: holdout-scenario
level: L3
id: "HS-002"
category: "multi-sensor"
must_pass: true
priority: P1
epic_id: "E-0"
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T00:00:00
phase: 1b
inputs: []
input-hash: null
traces_to: prd.md
behavioral_contracts: []
lifecycle_status: active
introduced: cycle-1
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "Grouped scenario file — contains multiple sub-scenarios HS-NNN-MM. Split into individual files deferred."
---

# HS-002: Multi-Sensor Scenarios

**Group:** Queries that span multiple sensors for a single client
**Date:** 2026-04-13
**Priority:** P0

---

## HS-002-01: Cross-Sensor Alert Correlation for Single Tenant

**Title:** MCP client queries alerts from all 4 sensors for one client in a single session

**Preconditions:**
- Tenant A has all 4 sensors configured: CrowdStrike, Cyberint, Claroty, Armis
- All 4 sensor APIs reachable with valid credentials
- Prism MCP server running

**Steps:**
1. MCP client calls `query_alerts` with `{ "tenant_id": "tenant-a", "sensor": "crowdstrike", "limit": 10 }`
2. MCP client calls `query_alerts` with `{ "tenant_id": "tenant-a", "sensor": "cyberint", "limit": 10 }`
3. MCP client calls `query_alerts` with `{ "tenant_id": "tenant-a", "sensor": "claroty", "limit": 10 }`
4. MCP client calls `query_alerts` with `{ "tenant_id": "tenant-a", "sensor": "armis", "limit": 10 }`

**Expected Outcome:**
- All 4 queries succeed independently
- Each uses its own auth mechanism: OAuth2 (CrowdStrike), Cookie (Cyberint), Bearer (Claroty), Bearer via SDK (Armis)
- All responses are OCSF-normalized to the same schema version
- OCSF `class_uid` and `category_uid` are consistent across sensors for the same alert type
- No auth credential cross-contamination (CrowdStrike OAuth2 token never sent as cookie, Cyberint cookie never sent as Bearer)
- Each sensor's cursor state updated independently
- Total session handles 4 sequential tool calls without state leakage

**Repos Tested:** All 4 pollers (auth contracts), tally (MCP session handling), ocsf-proto-gen (unified OCSF output), axiathon (DynamicMessage enabling cross-sensor schema consistency)

---

## HS-002-02: Mixed Data Source Types Across Sensors

**Title:** MCP client queries different entity types from different sensors in interleaved fashion

**Preconditions:**
- Tenant B configured with Claroty (9 data sources) and Armis (7 data sources)
- Both APIs reachable

**Steps:**
1. Client queries Claroty devices (`POST /api/v1/devices/`)
2. Client queries Armis vulnerabilities (AQL query)
3. Client queries Claroty device-vulnerability relations (`POST /api/v1/device_vulnerability_relations/`)
4. Client queries Armis devices (AQL query)
5. Client queries Claroty alerts (`POST /api/v1/alerts/`)

**Expected Outcome:**
- Each query uses the correct cursor type: Claroty devices use offset-based, Claroty alerts use (UpdatedTime, AlertID), Armis uses (Timestamp, TypeSpecificID)
- Cursor state per (tenant, sensor, data_source) is independent -- querying Claroty devices does not affect Claroty alert cursor
- Claroty's 3-tuple cursor (audit logs) works alongside 2-tuple cursors (other sources)
- Armis query fingerprints validated per data source (SHA-256 of AQL + limit)
- Response normalization handles both Claroty's 47-field device-alert relations and Armis's simpler structures
- No cursor state collision across (sensor, data_source) pairs

**Repos Tested:** poller-bear (9 data sources, variable cursor arity, POST-for-read), poller-coaster (7 data sources, AQL, timestamp fallback chains)

---

## HS-002-03: Concurrent Sensor Polling with Independent Backoff

**Title:** Multiple sensors polling simultaneously with independent retry/backoff state

**Preconditions:**
- Tenant C has CrowdStrike and Cyberint configured
- CrowdStrike API experiencing intermittent 500s
- Cyberint API healthy

**Steps:**
1. Prism starts polling both sensors concurrently
2. CrowdStrike returns HTTP 500 on first attempt
3. CrowdStrike backoff engages: 2s base, exponential increase
4. Cyberint continues polling normally (no backoff)
5. CrowdStrike recovers after 3 retries

**Expected Outcome:**
- CrowdStrike adapter retries with exponential backoff (2s, 4s, 8s) capped at 30s max
- Cyberint adapter unaffected by CrowdStrike failures -- continues on its own schedule
- Health readiness reflects per-sensor status: CrowdStrike NOT_READY, Cyberint READY
- After CrowdStrike recovery, backoff resets on success
- No shared backoff state between sensors
- Structured logs show tenant_id + sensor in every span

**Repos Tested:** poller-cobra (backoff config), poller-express (independent collection), all pollers (exponential backoff pattern: 2s base, 30s max)

---

## HS-002-04: Batch Sink Delivery Across Multiple Sensors

**Title:** Prism batches records from multiple sensors into efficient sink delivery

**Preconditions:**
- Tenant A has CrowdStrike and Claroty configured
- Vector sink endpoint configured
- Both sensors returning data

**Steps:**
1. CrowdStrike poll returns 50 alerts
2. Claroty poll returns 30 devices
3. Prism batches records for sink delivery (improving over pollers' per-record POST anti-pattern)
4. Prism delivers batched payloads to Vector

**Expected Outcome:**
- Records from different sensors batched together (or in sensor-grouped batches)
- Each record in the batch retains its correct `record_type` (`crowdstrike_alert`, `claroty_device`)
- xMP metadata correct per record (sensor-specific enrichment preserved)
- Per-record error attribution maintained despite batching -- if one record fails delivery, others are not lost
- Batch receipts generated per sensor per source with version, count, first_id, last_id, cursor_applied
- Total HTTP round-trips significantly fewer than 80 (50 + 30 individual POSTs)

**Repos Tested:** poller-cobra (xMP enrichment), poller-bear (batch receipts), poller-coaster (per-record delivery anti-pattern being improved)

---

## HS-002-05: OCSF Schema Consistency Across Sensors

**Title:** Alerts from different sensors normalize to compatible OCSF structures

**Preconditions:**
- All 4 sensors returning alert data
- OCSF proto definitions compiled from ocsf-proto-gen v1.7.0

**Steps:**
1. Fetch one CrowdStrike alert (32 named fields + overflow)
2. Fetch one Cyberint alert (52 AlertData subtypes)
3. Fetch one Claroty alert (20 fields)
4. Fetch one Armis alert
5. Normalize all 4 to OCSF Detection Finding (class 2004)
6. Compare normalized structures

**Expected Outcome:**
- All 4 normalized events share the same `class_uid` (2004)
- OCSF mandatory fields present in all: `time` (int64 epoch ms), `severity_id`, `activity_id`, `type_uid`
- Vendor-specific fields preserved in `unmapped` JSON blob (axiathon's vendor extension pattern)
- `timestamp_t` fields are int64 (epoch ms), NOT google.protobuf.Timestamp
- `json_t` fields are string (serialized JSON), NOT google.protobuf.Struct
- `datetime_t` fields are string (RFC 3339)
- Three-tier field alias resolution works: `src_ip` resolves to the same OCSF path regardless of source sensor

**Repos Tested:** ocsf-proto-gen (type mapping table: timestamp_t->int64, json_t->string, datetime_t->string), axiathon (DynamicMessage, three-tier alias, vendor extension via unmapped)

---

## State Checkpoint

```yaml
scenario_group: HS-002
title: Multi-Sensor
scenarios: 5
priority: P0
repos_covered: [poller-cobra, poller-express, poller-bear, poller-coaster, tally, ocsf-proto-gen, axiathon, mcp-claroty-xdome]
status: defined
```
