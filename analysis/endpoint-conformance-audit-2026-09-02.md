# Endpoint Conformance Audit — 2026-09-02

**Scope:** All `[[tables.steps]]` entries across `claroty.sensor.toml` (14 tables),
`cyberint.sensor.toml` (2 tables), `crowdstrike.sensor.toml` (3 tables),
`armis.sensor.toml` (2 tables). Static analysis only — no live API calls.

**Contract sources:**
- `xdome_openapi_06.20.2026.json` (Claroty xDome, 4.4 MB OpenAPI 3.1)
- `cyberint_alerts_openapi_06.20.2026.json` (Cyberint Alerts API)
- `cyberint_assets_openapi_06.20.2026.json` (Cyberint Assets API)
- `armis_endpoint_research_07.20.2026.md` (Armis — research doc, no OpenAPI)

**Six checks applied per endpoint:**
- (a) PATH + METHOD match a real contract operation
- (b) REQUIRED body fields present
- (c) `fields` projection is a valid subset of the endpoint's `fields_enum`
- (d) PAGINATION params: `sort_by` presence/absence, stable ordering, `include_count` presence,
  `page_size` within contract max
- (e) `response_path` matches the contract's response data key
- (f) COLUMN→FIELD mappings: each TOML column maps to a real contract field

**Check column notation:**
- `PASS` — check passed
- `FAIL` — defect found (see Defect List for details)
- `N/A` — check not applicable for this step type
- `UNVERIFIED` — contract source lacks detail to confirm or deny
- `WARN` — check passes structurally but carries a caveat

**Severity codes:** CRITICAL | HIGH | MEDIUM | LOW | OBS (observation)

---

## 1. Per-Endpoint Conformance Matrix

### 1.1 Claroty xDome

| # | Table | Step | (a) Path+Method | (b) Required Fields | (c) Fields Subset | (d) Pagination | (e) Response Path | (f) Col→Field | Overall |
|---|-------|------|-----------------|--------------------|--------------------|----------------|-------------------|---------------|---------|
| CL-01 | alerts | fetch_alerts | PASS — POST /api/v1/alerts/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 11/11 in fields_enum | WARN — sort_by absent; default id asc (stable unique key); include_count absent (intentional) | PASS — $.alerts ✓ | PASS — all 11 cols in fields_enum | PASS |
| CL-02 | audit_logs | fetch_audit_logs | PASS — POST /api/v1/audit_log/get in OpenAPI | PASS — no required fields; filter_by injected via Layer-2 | N/A — no fields param on this endpoint | WARN — sort_by absent; default timestamp asc (NON-UNIQUE); offset pagination with non-unique sort risks duplicates | PASS — $.audit_log ✓ | UNVERIFIED — OpenAPI response item schema is empty; columns cannot be validated against contract | WARN |
| CL-03 | devices | fetch_devices | PASS — POST /api/v1/devices/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 20/20 in 201-value fields_enum | WARN — sort_by absent; default uid asc (stable unique key); include_count absent (intentional) | PASS — $.devices ✓ | PASS — all 20 cols in fields_enum | PASS |
| CL-04 | device_alert_relations | fetch_device_alert_relations | PASS — POST /api/v1/device_alert_relations/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 10/10 in 92-value fields_enum | WARN — sort_by absent; default [device_uid asc, alert_id asc] (stable composite unique key) | PASS — $.devices_alerts ✓ | PASS — all 10 cols in fields_enum | PASS |
| CL-05 | vulnerabilities | fetch_vulnerabilities | PASS — POST /api/v1/vulnerabilities/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 18/18 in 32-value fields_enum | FAIL-M — sort_by absent; default published_date desc (NON-UNIQUE sort); offset pagination can produce gaps/duplicates at page boundaries for >1000 records | PASS — $.vulnerabilities ✓ | PASS — all 18 cols in fields_enum | MEDIUM |
| CL-06 | ot_activity_events | fetch_ot_activity_events | PASS — POST /api/v1/ot_activity_events/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 21/21 in 23-value fields_enum | WARN — sort_by absent; default event_id asc (stable unique key) | PASS — $.ot_activity_events ✓ | PASS — all 21 cols in fields_enum | PASS |
| CL-07 | device_vulnerability_relations | fetch_device_vulnerability_relations | PASS — POST /api/v1/device_vulnerability_relations/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 13/13 in 214-value fields_enum | WARN — sort_by absent; default [device_uid asc, vulnerability_id asc] (stable composite unique key) | PASS — $.devices_vulnerabilities ✓ | PASS — all 13 cols in fields_enum | PASS |
| CL-08 | servers | fetch_servers | PASS — POST /api/v1/servers/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 17/17 = full fields_enum coverage | WARN — sort_by absent; default server_name asc (stable unique PK) | PASS — $.servers ✓ | PASS — all 17 cols in fields_enum | PASS |
| CL-09 | server_interfaces | fetch_server_interfaces | PASS — POST /api/v1/server_interfaces/ (separate from /servers/) in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 10/10 = full fields_enum coverage | FAIL-M — sort_by absent; default server_name asc (NON-UNIQUE — multiple interfaces per server); offset pagination can produce gaps/duplicates at page boundaries | PASS — $.server_interfaces ✓ | PASS — all 10 cols in fields_enum | MEDIUM |
| CL-10 | organization_zones | fetch_organization_zones | PASS — POST /api/v1/organization_zones/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 11/11 = full fields_enum coverage | FAIL-M — sort_by absent; default priority asc (NON-UNIQUE — multiple zones can share priority) | PASS — $.organization_zones ✓ | PASS — all 11 cols in fields_enum | MEDIUM |
| CL-11 | organization_zone_policies | fetch_organization_zone_policies | PASS — POST /api/v1/organization_zone_policies/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 13/13 = full fields_enum coverage | FAIL-M — sort_by absent; default matching_devices asc (NON-UNIQUE — many policies can have same count) | PASS — $.organization_zone_policies ✓ | PASS — all 13 cols in fields_enum | MEDIUM |
| CL-12 | organization_firewall_groups | fetch_organization_firewall_groups | PASS — POST /api/v1/organization_fw_groups/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 11/11 = full fields_enum coverage | FAIL-M — sort_by absent; default priority asc (NON-UNIQUE) | PASS — $.organization_firewall_groups ✓ | PASS — all 11 cols in fields_enum | MEDIUM |
| CL-13 | organization_firewall_policies | fetch_organization_firewall_policies | PASS — POST /api/v1/organization_fw_group_policies/ in OpenAPI | PASS — `fields` (minItems:1) present in body | PASS — 13/13 = full fields_enum coverage | FAIL-M — sort_by absent; default matching_devices asc (NON-UNIQUE) | PASS — $.organization_firewall_policies ✓ | PASS — all 13 cols in fields_enum | MEDIUM |
| CL-14 | organization_acl_policies | fetch_organization_acl_policies | PASS — POST /api/v1/organization_acl_policies/ in OpenAPI | PASS — required policy_acl_syntax = "Cisco dACL" present; filter_by present (enumerate-all) | PASS — 11/11 in 11-value fields_enum | N/A — pagination.type = "none"; no offset/limit fields in API schema | PASS — $.organization_acl_policies ✓ | PASS — all 11 cols in fields_enum | PASS |

**Claroty result:** 8 PASS, 6 MEDIUM, 0 HIGH, 0 CRITICAL

---

### 1.2 Cyberint

> **Grounding context:** `cyberint.sensor.toml` is explicitly grounded against the
> **DTU clone** (`crates/prism-dtu-cyberint`) per ADR-053 §D1 (historical). The file
> header states it is "superseded per ADR-053 §D3-a; deletion owner:
> S-WAVE-A-CYBERINT-SPEC-001". Checks (a)–(f) are evaluated against **both** the
> real Cyberint OpenAPI and the DTU. Findings are labeled `[vs-real]` or `[vs-DTU]`.

| # | Table | Step | (a) Path+Method | (b) Required Fields | (c) Fields Subset | (d) Pagination | (e) Response Path | (f) Col→Field | Overall |
|---|-------|------|-----------------|--------------------|--------------------|----------------|-------------------|---------------|---------|
| CY-01 | alerts | fetch_alerts | FAIL-H [vs-real] — TOML uses `GET /api/v1/alerts`; real Cyberint API has only `POST /api/v1/alerts` (no GET alerts path in OpenAPI); PASS [vs-DTU] — DTU registers GET /api/v1/alerts | FAIL-H [vs-real] — real POST requires `page`+`size` in body (max size=100); TOML sends no body; PASS [vs-DTU] | N/A — no fields projection used | FAIL-H [vs-real] — TOML uses cursor_token pagination with $.next_cursor; real API uses page-based (GetAlertsRequest: page/size params, no cursor in response); PASS [vs-DTU] — DTU returns next_cursor | FAIL-H [vs-real] — TOML uses $.data; real API response has {"total": N, "alerts": [...]} key is `alerts` not `data`; PASS [vs-DTU] | UNVERIFIED [vs-real] — alert_id, title, type, severity match Alert schema; source maps to no standard Alert field (real Alert has `source` as part of `targeted_vectors`); created_at field is `created_date` in real Alert schema (mismatch) | HIGH [vs-real] |
| CY-02 | incidents | fetch_incidents | UNVERIFIED [vs-real] — no incidents endpoint in Cyberint Alerts OpenAPI; fidelity defect tracked under DEFECT-CYBERINT-SPEC-FIDELITY-001 | UNVERIFIED — no DTU route for incidents (EC-016-013-002) | N/A | WARN — cursor_token pagination consistent with alerts pattern | UNVERIFIED — $.data matches DTU pattern; no contract verification | UNVERIFIED — no DTU/OpenAPI to validate against | MEDIUM |

**Cyberint result:** 0 PASS, 1 MEDIUM, 1 HIGH (vs real API), 0 CRITICAL

---

### 1.3 CrowdStrike Falcon

> **Grounding context:** No downloaded OpenAPI available for CrowdStrike Falcon.
> Checks are evaluated against CrowdStrike's published Falcon API documentation
> patterns and the `crates/prism-dtu-crowdstrike` reference implementation.
> All six checks are marked UNVERIFIED unless grounded from DTU or well-known
> published CrowdStrike API patterns.

| # | Table | Step | (a) Path+Method | (b) Required Fields | (c) Fields Subset | (d) Pagination | (e) Response Path | (f) Col→Field | Overall |
|---|-------|------|-----------------|--------------------|--------------------|----------------|-------------------|---------------|---------|
| CS-01 | detections | query_detection_ids | UNVERIFIED — GET /detects/queries/detects/v1 matches well-known CrowdStrike QueryV2 pattern; DTU-confirmed | N/A — GET with query params | N/A | UNVERIFIED — offset-limit style via &limit= URL param; CrowdStrike QueryV2 uses limit+offset in query params | PASS [vs-DTU] — $.resources ✓ | N/A — ID-query step produces array of IDs, no data columns | UNVERIFIED |
| CS-02 | detections | fetch_detections | UNVERIFIED — POST /detects/entities/summaries/GET/v1 matches well-known CrowdStrike PostDetections pattern; DTU-confirmed | PASS — ids array required, sent as {"ids": [...]} | N/A | N/A — fan-out step, no pagination | PASS [vs-DTU] — $.resources ✓ | PASS [vs-DTU] — detection_id, status, severity, device_id, tactic, technique all confirmed in DTU generator | UNVERIFIED |
| CS-03 | devices | query_device_ids | UNVERIFIED — GET /devices/queries/devices/v1 matches well-known CrowdStrike QueryV2 pattern | N/A — GET with query params | N/A | UNVERIFIED — GET query step | PASS [vs-DTU] — $.resources ✓ | N/A — ID-query step | UNVERIFIED |
| CS-04 | devices | fetch_devices | UNVERIFIED — POST /devices/entities/devices/v2 confirmed from DTU; CrowdStrike PostDeviceDetailsV2 is the official endpoint supporting up to 5000 IDs | PASS — ids array required, sent as {"ids": [...]} | N/A | N/A — fan-out step | PASS [vs-DTU] — $.resources ✓ | UNVERIFIED [vs-real] — device_id, hostname, platform_name, status, first_seen, last_seen match DTU fixture structure | UNVERIFIED |
| CS-05 | incidents | query_incident_ids | UNVERIFIED — GET /incidents/queries/incidents/v1 matches well-known CrowdStrike pattern; RETIREMENT-PENDING-S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001 | N/A — GET with query params | N/A | UNVERIFIED | PASS [vs-DTU] — $.resources ✓ | N/A — ID-query step | UNVERIFIED |
| CS-06 | incidents | fetch_incidents | UNVERIFIED — POST /incidents/entities/incidents/GET/v1 matches well-known CrowdStrike GetIncidentDetails pattern; RETIREMENT-PENDING | PASS — ids array required | N/A | N/A — fan-out step | PASS [vs-DTU] — $.resources ✓ | UNVERIFIED [vs-real] | UNVERIFIED |

**CrowdStrike result:** 0 confirmed PASS (all 6 UNVERIFIED — no OpenAPI contract available)

---

### 1.4 Armis Centrix

> **Grounding context:** No downloadable OpenAPI available for Armis.
> The authoritative contract is `armis_endpoint_research_07.20.2026.md` (ADR-053 §D1).
> TOML is grounded against DTU per ADR-053 §D2; divergences from the real API are
> tracked under S-WAVE-A-ARMIS-REMEDIATION-001.

| # | Table | Step | (a) Path+Method | (b) Required Fields | (c) Fields Subset | (d) Pagination | (e) Response Path | (f) Col→Field | Overall |
|---|-------|------|-----------------|--------------------|--------------------|----------------|-------------------|---------------|---------|
| AR-01 | devices | fetch_devices | PASS — GET /api/v1/search?aql=... confirmed in research doc: "Armis exposes a single unified search surface (GET /api/v1/search/)" | N/A — GET, aql forwarded via query param | N/A — no fields projection (AQL entity projection) | WARN — offset_limit with page_size=25; research doc notes max ~1000 (Chronicle default, unconfirmed); 25 is conservative but correct; no sort_by (AQL server controls ordering) | PASS [vs-DTU] — $.data.results confirmed by DTU SearchResponse struct | FAIL-M [vs-real] — research doc confirms real API field name divergences: `name`→`title`, `ip_address`→`ipaddress`, `mac_address`→`macAddress`, `os_name`→`operatingSystem`, `risk_score`→`riskLevel`; PASS [vs-DTU] | MEDIUM [vs-real] |
| AR-02 | alerts | fetch_alerts | PASS — GET /api/v1/search?aql=... confirmed (same unified search surface, in:alerts discriminator) | N/A — GET | N/A | WARN — same as devices (page_size=25, conservative) | PASS [vs-DTU] — $.data.results | FAIL-M [vs-real] — research doc: `device_id` should be `deviceIds` (array, not scalar); `policy_name` should be `policyTitle`/`policyId`; `created_at` field is `time` in real alerts; `updated_at` has no real-API counterpart; PASS [vs-DTU] | MEDIUM [vs-real] |

**Armis result:** 0 confirmed PASS, 2 MEDIUM (vs real API), 0 HIGH

---

## 2. Defect List

### D-001 — CL-05: vulnerabilities — non-unique default sort, offset pagination instability
**Endpoint:** POST /api/v1/vulnerabilities/  
**Defect class:** Pagination stability — non-unique default sort_by  
**Severity:** MEDIUM  
**Contract violation:** The xDome OpenAPI default for `sort_by` on the vulnerabilities endpoint
is `[{field: "published_date", order: "desc"}]`. `published_date` is NOT a unique key — multiple
vulnerabilities can share the same published date. With offset pagination at page_size=1000,
when a page boundary falls on a tie group, the relative ordering of tie records is non-deterministic
across request pairs. This can cause records near the boundary to appear in both consecutive pages
(duplicates) or in neither page (gaps). This is latent: it only activates in environments with
>1000 total vulnerabilities.  
**Recommended fix:** Add `"sort_by": [{"field": "name", "order": "asc"}]` to the
`fetch_vulnerabilities` body_template. `name` is the vulnerability's unique primary identifier
(it is the REQUIRED TOML column and maps to `finding_info.title`). This yields a stable,
deterministic sort for offset pagination.

---

### D-002 — CL-02: audit_logs — non-unique default sort, offset pagination instability
**Endpoint:** POST /api/v1/audit_log/get  
**Defect class:** Pagination stability — non-unique default sort_by  
**Severity:** MEDIUM  
**Contract violation:** The xDome OpenAPI default for `sort_by` on audit_log is
`[{field: "timestamp", order: "asc"}]`. `timestamp` is NOT unique — multiple audit events
can occur at the same second. Offset pagination across a timestamp-tie group yields the
same duplication/gap risk as D-001.  
**Mitigating factor:** The `_claroty_audit_filter_by` Layer-2 variable injects a time-window
filter (defaulting to a 7-day window) which limits the result set. In most environments,
a 7-day audit window will stay below 1000 events, making the pagination instability latent.
For high-volume tenants the risk is real.  
**Recommended fix:** Add `"sort_by": [{"field": "timestamp", "order": "asc"}, {"field": "id", "order": "asc"}]`
as a compound sort using `id` as the tiebreaker. Verify `id` is present in the audit_log
sort fields schema before applying (audit_log sort fields are a separate schema from the
data fields; the sort clause validation uses `SortClause` not a `ValidatingSortClause`).

---

### D-003 — CL-09: server_interfaces — non-unique default sort
**Endpoint:** POST /api/v1/server_interfaces/  
**Defect class:** Pagination stability — non-unique default sort_by  
**Severity:** MEDIUM  
**Contract violation:** Default sort is `server_name asc`. Server_name is NOT unique for
this table (one server has multiple interfaces). Offset pagination at page boundaries
involving servers with many interfaces may produce gaps/duplicates. In practice this risk
is low because most deployments have far fewer than 1000 interfaces.  
**Recommended fix:** Add `"sort_by": [{"field": "server_name", "order": "asc"}, {"field": "interface_name", "order": "asc"}]`.
The composite (server_name, interface_name) pair forms the unique key for this table.

---

### D-004 — CL-10: organization_zones — non-unique default sort
**Endpoint:** POST /api/v1/organization_zones/  
**Defect class:** Pagination stability — non-unique default sort_by  
**Severity:** MEDIUM (low blast radius — typical deployments have <100 zones)  
**Contract violation:** Default sort is `priority asc`. Priority is NOT unique.  
**Recommended fix:** Add `"sort_by": [{"field": "zone_name", "order": "asc"}]`. `zone_name`
is the unique PK for organization_zones and is always present.

---

### D-005 — CL-11: organization_zone_policies — non-unique default sort
**Endpoint:** POST /api/v1/organization_zone_policies/  
**Defect class:** Pagination stability — non-unique default sort_by  
**Severity:** MEDIUM (low blast radius)  
**Contract violation:** Default sort is `matching_devices asc`. Not unique.  
**Recommended fix:** Add `"sort_by": [{"field": "policy_name", "order": "asc"}]`.

---

### D-006 — CL-12: organization_firewall_groups — non-unique default sort
**Endpoint:** POST /api/v1/organization_fw_groups/  
**Defect class:** Pagination stability — non-unique default sort_by  
**Severity:** MEDIUM (low blast radius)  
**Contract violation:** Default sort is `priority asc`. Not unique.  
**Recommended fix:** Add `"sort_by": [{"field": "firewall_group_name", "order": "asc"}]`.

---

### D-007 — CL-13: organization_firewall_policies — non-unique default sort
**Endpoint:** POST /api/v1/organization_fw_group_policies/  
**Defect class:** Pagination stability — non-unique default sort_by  
**Severity:** MEDIUM (low blast radius)  
**Contract violation:** Default sort is `matching_devices asc`. Not unique.  
**Recommended fix:** Add `"sort_by": [{"field": "policy_name", "order": "asc"}]`.

---

### D-008 — CY-01: Cyberint alerts — method, pagination type, response_path mismatch vs real API
**Endpoint:** TOML uses GET /api/v1/alerts; real Cyberint API is POST /api/v1/alerts  
**Defect class:** Structural incompatibility vs real API (method + pagination + response envelope)  
**Severity:** HIGH (vs real API) / PASS (vs DTU)  
**Contract violation (vs real Cyberint OpenAPI):**
1. Method: TOML uses GET; real API has POST /api/v1/alerts (`operationId: get_alerts_api_v1_alerts_post`). No GET /api/v1/alerts path exists in the real Cyberint API.
2. Pagination: TOML uses `cursor_token` with `$.next_cursor`; real API uses page-based pagination (`GetAlertsRequest.page` / `.size`, max size=100). No cursor field in real API response.
3. Response path: TOML uses `$.data`; real `GetAlertsResponse` has `{"total": N, "alerts": [...]}` — key is `alerts`, not `data`.
4. Field drift: `created_at` in TOML → `created_date` in real Alert schema; `source` in TOML → no direct field in real Alert (alert has `targeted_vectors`, `source_category`).  
**Status:** Tracked under S-WAVE-A-CYBERINT-SPEC-001 / ADR-053 §D3-a as a pending replacement spec. TOML is explicitly flagged superseded.  
**Action:** No immediate fix required in sensor TOML — addressed by the pending spec replacement story.

---

### D-009 — CY-02: Cyberint incidents — fidelity defect, no DTU route
**Endpoint:** GET /api/v1/incidents  
**Defect class:** Unverified endpoint with no DTU grounding  
**Severity:** MEDIUM  
**Contract violation:** No incidents endpoint exists in the Cyberint Alerts OpenAPI.
No DTU route exists in `crates/prism-dtu-cyberint` (EC-016-013-002). The incidents
table was retained in the TOML pending atomic deletion by S-WAVE-A-CYBERINT-SPEC-001.  
**Status:** Tracked under DEFECT-CYBERINT-SPEC-FIDELITY-001. Story owner: S-WAVE-A-CYBERINT-SPEC-001 AC-007 RG-013.

---

### D-010 — AR-01 / AR-02: Armis devices and alerts — field names diverge from real API
**Endpoint:** GET /api/v1/search (both tables)  
**Defect class:** Column→field mapping drift vs real Armis v1 API  
**Severity:** MEDIUM (vs real API) / PASS (vs DTU)  
**Contract violation (vs `armis_endpoint_research_07.20.2026.md` Confirmed tier):**

For `devices`:
- TOML `ip_address` → real API field is `ipaddress` (no underscore per research doc)
- TOML `mac_address` → real API field is `macAddress` (camelCase)
- TOML `os_name` → real API field is `operatingSystem`
- TOML `risk_score` → real API field is `riskLevel` (string, not integer)
- TOML `name` → present in DTU but real API field is `name` (CONFIRMED per research doc)

For `alerts`:
- TOML `device_id` (scalar String) → real API field is `deviceIds` (array, not scalar)
- TOML `policy_name` → real API field is `policyTitle` or `policyId`
- TOML `created_at` → real API field is `time`
- TOML `updated_at` → no direct counterpart in real API alerts

**Status:** Tracked under S-WAVE-A-ARMIS-REMEDIATION-001. ADR-053 §D2 designates the
DTU as source-of-truth until the remediation story lands; TOML is correct against the DTU.

---

### D-011 — CL-02: audit_logs columns UNVERIFIED vs OpenAPI contract
**Endpoint:** POST /api/v1/audit_log/get  
**Defect class:** Column validation gap (OpenAPI response schema is empty)  
**Severity:** LOW  
**Detail:** The `GetAuditLogResponse` item schema in the xDome OpenAPI has empty `properties: {}`.
This means checks (c) and (f) for audit_log cannot be mechanically verified against the
OpenAPI contract — the TOML columns (id, action, user_display_name, category, timestamp,
details, username, note) cannot be cross-referenced against a declared schema. These columns
are grounded against the live xDome API behavior documented in LIVE-DRIFT-003 and the
prism-dtu-claroty audit_log route (`routes/audit_log.rs`). Any column mismatch with the
real API would be caught only at runtime or via DTU parity tests.  
**Action:** Accept. Column validation deferred to DTU parity test coverage.

---

## 3. Summary

### 3.1 Totals

| Sensor | Total Steps | Conformant | Has Findings | UNVERIFIED |
|--------|-------------|------------|--------------|------------|
| Claroty xDome | 14 | 8 | 6 MEDIUM | 0 |
| Cyberint | 2 | 0 | 2 (1 HIGH + 1 MEDIUM) | 0 |
| CrowdStrike | 6 | 0 | 0 | 6 (no OpenAPI) |
| Armis | 2 | 0 | 2 MEDIUM | 0 |
| **TOTAL** | **24** | **8** | **10** | **6** |

**Conformant (PASS all checks):** 8 of 24 steps  
**Has at least one finding:** 10 of 24 steps  
**UNVERIFIED (no OpenAPI contract available):** 6 of 24 steps (all CrowdStrike)  
**CRITICAL defects:** 0  
**HIGH defects:** 1 (Cyberint alerts vs real API — tracked, acknowledged)  
**MEDIUM defects:** 9 across 4 tables  
**LOW defects:** 1 (audit_log column unverified)

---

### 3.2 Systemic Defect Classes

#### Class 1 — Sort_by absent with non-unique API default (7 Claroty endpoints)

**Blast radius:** Claroty xDome only. All 7 use offset_limit pagination with page_size=1000.

The original audit trigger (vulnerabilities step sending no sort_by) is part of a systemic
pattern. However, the audit finding is materially different from the initial concern:

**Initial concern:** sort_by absent is CRITICAL.  
**Audit finding:** sort_by is OPTIONAL in the OpenAPI schema. All 14 Claroty paginated endpoints
have DOCUMENTED DEFAULT sort orders. The defect class is NOT missing sort_by but rather
that 7 of 13 paginated endpoints have non-unique default sort keys, creating offset pagination
instability for large datasets.

| Endpoint | Default sort | Is sort key unique? | Risk |
|----------|-------------|---------------------|------|
| alerts | id asc | YES (int PK) | Stable — no action |
| devices | uid asc | YES (UUID) | Stable — no action |
| device_alert_relations | device_uid asc + alert_id asc | YES (composite PK) | Stable — no action |
| device_vulnerability_relations | device_uid asc + vulnerability_id asc | YES (composite PK) | Stable — no action |
| ot_activity_events | event_id asc | YES (int PK) | Stable — no action |
| servers | server_name asc | YES (unique PK for this table) | Stable — no action |
| organization_acl_policies | policy_id asc | YES (UUID); also no pagination | Stable — N/A |
| **vulnerabilities** | **published_date desc** | **NO (non-unique date)** | **MEDIUM — D-001** |
| **audit_logs** | **timestamp asc** | **NO (non-unique timestamp)** | **MEDIUM — D-002** |
| **server_interfaces** | **server_name asc** | **NO (multiple rows per server)** | **MEDIUM — D-003** |
| **organization_zones** | **priority asc** | **NO (multiple zones per priority)** | **MEDIUM — D-004** |
| **organization_zone_policies** | **matching_devices asc** | **NO** | **MEDIUM — D-005** |
| **organization_firewall_groups** | **priority asc** | **NO** | **MEDIUM — D-006** |
| **organization_firewall_policies** | **matching_devices asc** | **NO** | **MEDIUM — D-007** |

**`include_count` absent:** NOT a defect. Prism uses the short-page halt
(`page_record_count < page_size`) as the pagination termination signal. `include_count`
is an optional performance hint; its absence is intentional and correct per
BC-2.16.002 §Postconditions OffsetLimit Pagination.

#### Class 2 — Cyberint TOML vs real API structural incompatibility (1 step, HIGH)

The entire `cyberint.sensor.toml` is grounded against the DTU, not the real Cyberint API.
The real Cyberint API differs fundamentally on method (POST vs GET), pagination type (page-based
vs cursor), and response envelope (`alerts` vs `data` key). The fix is pending
S-WAVE-A-CYBERINT-SPEC-001 (new replacement spec authored against the real Cyberint API).
This defect class affects 1 live endpoint (Cyberint alerts) and has no action required on
the current TOML — the remediation is a spec replacement, not a TOML patch.

#### Class 3 — Armis DTU-vs-real field name drift (2 steps, MEDIUM)

Armis device and alert columns are grounded against the DTU, which diverges from the real
Armis v1 API. The `armis_endpoint_research_07.20.2026.md` Confirmed tier establishes the
real field names. This defect class affects 2 endpoints and is tracked under
S-WAVE-A-ARMIS-REMEDIATION-001. No action on the current TOML until that story lands.

#### Class 4 — CrowdStrike endpoints UNVERIFIED (6 steps)

No downloadable OpenAPI for CrowdStrike Falcon exists. All 6 CrowdStrike steps are grounded
against the DTU and well-known CrowdStrike Falcon API patterns. No CRITICAL or HIGH findings
were identified — the patterns are consistent with the published API. Recommend obtaining
official CrowdStrike OpenAPI spec when available for formal verification.

---

### 3.3 Top Priority Defects

| Rank | Defect ID | Endpoint | Severity | Action |
|------|-----------|----------|----------|--------|
| 1 | D-001 | claroty vulnerabilities | MEDIUM | Add stable sort_by to body_template |
| 2 | D-002 | claroty audit_logs | MEDIUM | Add compound sort_by to body_template |
| 3 | D-008 | cyberint alerts | HIGH (vs real API) | No TOML action — owned by S-WAVE-A-CYBERINT-SPEC-001 |
| 4 | D-003 | claroty server_interfaces | MEDIUM | Add composite sort_by |
| 5 | D-004..D-007 | claroty org zones/policies | MEDIUM (×4) | Add zone_name/policy_name sort_by |
| 6 | D-010 | armis devices+alerts | MEDIUM (vs real API) | No TOML action — owned by S-WAVE-A-ARMIS-REMEDIATION-001 |
| 7 | D-009 | cyberint incidents | MEDIUM | No TOML action — owned by S-WAVE-A-CYBERINT-SPEC-001 |
| 8 | D-011 | claroty audit_log columns | LOW | Accept — covered by DTU parity tests |

---

### 3.4 Fields Projection Validation Summary (Claroty — the only sensor with a fields_enum)

**All 12 enumerable Claroty endpoints pass (c) with 100% field validity:**

| Endpoint | TOML field count | fields_enum size | Validation |
|----------|-----------------|------------------|------------|
| alerts | 11 | 20 | ALL VALID |
| devices | 20 | 201 | ALL VALID |
| vulnerabilities | 18 | 32 | ALL VALID |
| device_alert_relations | 10 | 92 | ALL VALID |
| device_vulnerability_relations | 13 | 214 | ALL VALID |
| ot_activity_events | 21 | 23 | ALL VALID |
| servers | 17 | 17 | ALL VALID (100% coverage) |
| server_interfaces | 10 | 10 | ALL VALID (100% coverage) |
| organization_zones | 11 | 11 | ALL VALID (100% coverage) |
| organization_zone_policies | 13 | 13 | ALL VALID (100% coverage) |
| organization_fw_groups | 11 | 11 | ALL VALID (100% coverage) |
| organization_fw_group_policies | 13 | 13 | ALL VALID (100% coverage) |
| organization_acl_policies | 11 | 11 | ALL VALID (100% coverage) |
| audit_log | N/A — no fields param | N/A | N/A |

No CRITICAL field-projection defects found. The original CRITICAL hypothesis
(hallucinated field names failing at runtime) is NOT confirmed for any active Claroty endpoint.

---

## Appendix — Runtime Pipeline Behavior Notes

**sort_by injection:** The `pipeline.rs::build_request` function does NOT inject
`sort_by` automatically. It only injects `offset` and `limit` into POST bodies for
`PaginationConfig::OffsetLimit` steps. Adding `sort_by` to body_templates requires
explicit TOML authoring.

**include_count injection:** Not injected by the pipeline. `include_count` is an
optional API hint; omitting it causes the API to return data without a total count.
Prism's short-page halt mechanism (`page_record_count < page_size` termination) does
not need a total count and is correct per BC-2.16.002.

**Missing query_filter variables:** `seed_missing_query_filter_vars` in `pipeline.rs`
pre-seeds any `${query.filter.VARNAME}` references with an empty string if not provided.
For `claroty` audit_log, `_claroty_audit_filter_by` is always injected by
`spec_driven_adapter.rs` before the pipeline runs, ensuring a valid JSON object is
always available.

**Armis page_size=25:** Conservative but correct. The research doc indicates ~1000 max
(unconfirmed); increasing page_size is a performance optimization not a correctness fix.

---

*Audit performed: 2026-09-02. Static analysis only. Report is analysis-only — no TOML or code changes made.*
