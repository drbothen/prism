---
document_type: holdout-scenario
level: L3
id: "HS-001"
category: "happy-path"
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

# HS-001: Happy Path Scenarios

**Group:** Happy Path -- MCP client queries sensors, gets normalized OCSF responses
**Date:** 2026-04-13
**Priority:** P0 (must pass before any other scenario group)

---

## HS-001-01: Single Sensor Alert Query via MCP

**Title:** MCP client queries CrowdStrike alerts and receives OCSF-normalized response

**Preconditions:**
- Prism MCP server running with stdio transport
- CrowdStrike sensor adapter configured for Tenant A with valid OAuth2 credentials
- CrowdStrike Falcon API reachable and returning alerts
- OCSF proto definitions compiled at build time via ocsf-proto-gen

**Steps:**
1. MCP client connects via stdio JSON-RPC
2. Client calls `query_alerts` tool with parameters: `{ "tenant_id": "tenant-a", "sensor": "crowdstrike", "limit": 10 }`
3. Prism authenticates to CrowdStrike using OAuth2 Client Credentials flow
4. Prism calls CrowdStrike QueryV2 for alert IDs, then PostEntitiesAlertsV1 for details (two-step fetch)
5. Prism normalizes each alert to OCSF Detection Finding (class 2004) using DynamicMessage pattern
6. Prism wraps response as MCP CallToolResult with JSON content

**Expected Outcome:**
- MCP response contains up to 10 OCSF-normalized alert objects
- Each alert has OCSF mandatory fields: `class_uid`, `category_uid`, `severity_id`, `activity_id`, `type_uid`, `time`, `metadata`
- Response time under 5 seconds for 10 alerts
- No raw CrowdStrike field names leak into OCSF output
- Cursor state persisted durably for next poll

**Repos Tested:** poller-cobra (API contract, OAuth2, two-step fetch), tally (rmcp 0.8 tool dispatch, Parameters<T>), ocsf-proto-gen (type mapping), axiathon (DynamicMessage pattern)

---

## HS-001-02: Cyberint Alert Query with Cookie Authentication

**Title:** MCP client queries Cyberint alerts using cookie-based auth

**Preconditions:**
- Cyberint sensor adapter configured for Tenant B with valid API key
- Cyberint Argos API reachable

**Steps:**
1. MCP client calls `query_alerts` tool with `{ "tenant_id": "tenant-b", "sensor": "cyberint", "limit": 50 }`
2. Prism constructs HTTP request with `Cookie: access_token=<api_key>` header (NOT Authorization header)
3. Prism calls POST `{base}/alert/api/v1/alerts` with page/size/filters
4. Prism parses CyberintTime fields using multi-format parser (RFC3339, no-timezone, microseconds, null/empty)
5. Prism normalizes alerts to OCSF and wraps in xMP envelope

**Expected Outcome:**
- Authentication uses cookie injection, not bearer header
- CyberintTime fields parsed without error across all 4 formats
- Customer ID correctly extracted from URL subdomain
- OCSF-normalized response returned via MCP
- xMP envelope contains `record_type: "cyberint_alert"`

**Repos Tested:** poller-express (Cyberint API contract, cookie auth, CyberintTime parsing, xMP enrichment), tally (MCP transport)

---

## HS-001-03: Claroty xDome Multi-Source Query

**Title:** MCP client queries Claroty devices and alerts in a single session

**Preconditions:**
- Claroty sensor adapter configured for Tenant C with valid bearer token
- Claroty xDome API reachable with 9 endpoints available

**Steps:**
1. MCP client calls `query_devices` tool with `{ "tenant_id": "tenant-c", "sensor": "claroty", "limit": 100 }`
2. Prism calls POST `/api/v1/devices/` (POST-for-read pattern) with bearer auth
3. Prism handles polymorphic JSON IDs (string OR number) without error
4. Client then calls `query_alerts` tool with `{ "tenant_id": "tenant-c", "sensor": "claroty" }`
5. Prism calls POST `/api/v1/alerts/` with (UpdatedTime, AlertID) cursor

**Expected Outcome:**
- Both queries return OCSF-normalized results
- Polymorphic ID parsing handles both string "123" and number 123
- Device query uses offset-based pagination; alert query uses timestamp-based cursor
- Bearer token trimmed of whitespace at construction time
- Each query updates its own independent cursor state

**Repos Tested:** poller-bear (Claroty API contract, 9 endpoints, polymorphic JSON, POST-for-read), mcp-claroty-xdome (sensor MCP wrapper pattern, cache isolation)

---

## HS-001-04: Armis AQL Query Forwarding

**Title:** MCP client queries Armis vulnerabilities using AQL

**Preconditions:**
- Armis sensor adapter configured for Tenant D with valid API key
- Armis Centrix API reachable

**Steps:**
1. MCP client calls `query_vulnerabilities` tool with `{ "tenant_id": "tenant-d", "sensor": "armis", "limit": 25 }`
2. Prism constructs AQL query and calls GetSearch via Armis SDK wrapper
3. Prism applies timestamp fallback chain (tries 1-3 timestamp fields in priority order)
4. Prism applies ID fallback chain (tries 2-4 ID fields in priority order)
5. Prism normalizes results to OCSF Vulnerability Finding

**Expected Outcome:**
- AQL query forwarded correctly to Armis API
- Timestamp field resolved from priority chain without error
- hasMore pagination triggers immediate re-fetch when results exceed limit
- Query fingerprint computed as SHA-256(AQL query + limit) and validated against stored fingerprint
- OCSF-normalized results returned via MCP

**Repos Tested:** poller-coaster (Armis API contract, AQL forwarding, timestamp fallback chains, hasMore pagination), ocsf-proto-gen (OCSF type mapping)

---

## HS-001-05: xMP Envelope Backward Compatibility

**Title:** Prism output maintains xMP envelope format compatible with existing Vector pipeline

**Preconditions:**
- Prism configured with Vector sink endpoint
- Vector pipeline consuming xMP-format JSON

**Steps:**
1. Prism polls CrowdStrike alerts for Tenant A
2. Prism wraps each alert in xMP envelope
3. Prism delivers enriched payload to Vector via HTTP POST with Basic Auth

**Expected Outcome:**
- Envelope matches exact format: `{"data": <record>, "record_type": "crowdstrike_alert", "xmp": {"site": "...", "cluster_name": "...", "node_name": "..."}, "ocsf": <normalized_event>}`
- `record_type` follows standardized `<sensor>_<entity>` pattern
- `ocsf` field present (optional OCSF normalization output, standardized across all sensors)
- Vector accepts payload without modification to existing pipeline configuration

**Repos Tested:** poller-cobra (xMP envelope format), poller-express (xMP format), poller-bear (xMP + ocsf field), poller-coaster (xMP format)

---

## HS-001-06: Health Probes and Readiness Tracking

**Title:** Prism health endpoints report correct readiness based on collection state

**Preconditions:**
- Prism running with health server enabled (default: enabled, unlike pollers where disabled by default)
- At least one sensor adapter configured and polling

**Steps:**
1. HTTP GET `/live` on health port
2. HTTP GET `/ready` on health port before first successful collection
3. Wait for first successful sensor poll cycle
4. HTTP GET `/ready` again after successful collection
5. Simulate sensor failure, then GET `/ready`

**Expected Outcome:**
- `/live` returns 200 immediately (process is alive)
- `/ready` returns 503 before first successful collection
- `/ready` returns 200 after first successful collection (readiness tracks collection success)
- `/ready` returns 503 after sensor failure
- `/health` returns 200 with aggregate status
- Health port configurable (single port, resolving poller inconsistency: cobra/express 7322, bear 7321)
- Per-IP rate limiting active: 100 req/s burst 20, returns 429 with Retry-After header
- Rate limiter map uses LRU eviction (fixing unbounded map from pollers)

**Repos Tested:** poller-cobra (health probes), poller-express (rate limiting), poller-bear (readiness tracking), poller-coaster (rate limiting)

---

## State Checkpoint

```yaml
scenario_group: HS-001
title: Happy Path
scenarios: 6
priority: P0
repos_covered: [poller-cobra, poller-express, poller-bear, poller-coaster, tally, ocsf-proto-gen, axiathon, mcp-claroty-xdome]
status: defined
```
