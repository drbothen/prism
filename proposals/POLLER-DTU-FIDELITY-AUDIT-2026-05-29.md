---
document_type: architect-audit
title: "Cross-Poller DTU Fidelity Audit — All 4 Sensors (2026-05-29)"
author: architect
date: "2026-05-29"
status: FINAL
version: "1.0"
scope: CrowdStrike, Claroty xDome, Armis Centrix, Cyberint Argos
sources_read:
  - .factory/semport/poller-cobra/{broad-sweep, pass-1-r2, pass-2-r2, pass-3-r2}
  - .factory/semport/poller-bear/{broad-sweep, pass-1-r3, pass-2-r2, pass-3-r2}
  - .factory/semport/poller-coaster/{broad-sweep, pass-1-r2, pass-2-r2, pass-3-r2}
  - .factory/semport/poller-express/{broad-sweep, pass-1-r2, pass-2-r2, pass-3-r2}
  - crates/prism-sensors/specs/{crowdstrike,claroty,armis,cyberint}.sensor.toml
  - crates/prism-dtu-{crowdstrike,claroty,armis,cyberint}/src/{clone,types}.rs
  - crates/prism-dtu-crowdstrike/src/routes/{mod,oauth,detections}.rs
  - crates/prism-dtu-claroty/src/routes/alerts.rs
  - crates/prism-dtu-armis/src/routes/devices.rs
  - crates/prism-dtu-cyberint/src/routes/{alerts,auth}.rs
  - .factory/proposals/E2E-DEMO-WIRING-PLAN.md
  - .factory/stories/{S-DEMO-001,S-DEMO-002}
---

# Cross-Poller DTU Fidelity Audit — All 4 Sensors

## Purpose

Verify that prism's TOML sensor specs, DTU clones, and S-DEMO-001/002 ACs represent
true-to-life production behavior for a LIVE demo. Source of truth for each dimension:
the production poller repo semport docs (poller reality). Where TOML or DTU diverge
from the poller, that is a fidelity gap.

## Severity Classification

- **CRITICAL** — runtime defect that makes the live demo fail or silently return wrong data
- **HIGH** — fidelity gap the live demo audience would notice
- **MEDIUM** — fidelity gap that could cause follow-up questions
- **LOW** — cosmetic or completeness gap not visible during demo queries

---

## 1. CrowdStrike Falcon (poller-cobra)

### 1.1 Fidelity Gap Audit Table

| Dimension | Poller Reality | Prism TOML | DTU Clone | Gap? | Severity | Action |
|-----------|---------------|------------|-----------|------|----------|--------|
| **Auth mechanism** | OAuth2 client credentials via gofalcon SDK. Calls `/oauth2/token` with `client_id` + `client_secret` (form body implied by gofalcon). Returns `access_token` Bearer. | `auth_type = "oauth2_client_credentials"`, `auth_plugin = "crowdstrike-oauth2"` | DTU `POST /oauth2/token` returns `{"access_token": "dtu-fake-cs-token", "token_type": "bearer", "expires_in": 3600}` (routes/oauth.rs) | **NO** — fully aligned. Plugin handles OAuth2 flow; DTU emulates the token endpoint. | — | None |
| **API base URL** | `https://api.crowdstrike.com` (SDK default for us-1); region selectable: us-1, us-2, eu-1, ap-1 via `Cloud` param | `base_url = "https://api.crowdstrike.com"` (us-1 hardcoded) | DTU listens on ephemeral 127.0.0.1:PORT | **MEDIUM** — Multi-region routing not in TOML. Demo environment is likely us-1 so no runtime breakage. | MEDIUM | Follow-up story: add `base_url` env override for non-us-1 regions. Not a demo blocker. |
| **Alerts two-step: poller endpoint** | `QueryV2` (GET) + `PostEntitiesAlertsV1` (POST). Poller actually uses **alerts** API, NOT detections API. | TOML `detections` table uses `GET /detects/queries/detects/v1` + `POST /detects/entities/summaries/GET/v1` | DTU routes `GET /detects/queries/detects/v1` + `POST /detects/entities/summaries/GET/v1` | **MEDIUM** — Poller polls **alerts** (`/alerts` API), prism spec exposes **detections** (`/detects` API). These are different CrowdStrike entities. For the demo, prism returns detections; live CrowdStrike has both. Not a runtime defect since DTU serves detections correctly; demo just exposes a different table than the poller uses. | MEDIUM | Document in demo script. Demo should query `crowdstrike_detections`, not "alerts". |
| **Pagination** | poller-cobra uses `sort: "timestamp|desc"` + `limit` in QueryV2, no explicit pagination loop — relies on `hasMore` burst refetch | TOML: no explicit `PaginationConfig` on step 2 (PostEntities fan_out). Step 1 returns IDs. | DTU: `GET /detects/queries/detects/v1` returns list of IDs from fixture; `POST /detects/entities/summaries/GET/v1` accepts `{"ids": [...]}` | **NO** — two-step ID-then-batch is correctly modeled in both TOML and DTU. | — | None |
| **Auth credentials env vars** | `CROWDSTRIKE_CLIENT_ID` + `CROWDSTRIKE_CLIENT_SECRET` (plus `_FILE` variants) | TOML auth_plugin = "crowdstrike-oauth2" (plugin handles cred extraction) | DTU: no credential requirement (static fake token returned) | **NO** — prod flow: plugin reads creds, calls DTU `/oauth2/token`, gets Bearer token | — | None |
| **Detection response fields** | poller-cobra `alertToMap`: `id, composite_id, aggregate_id, cid, timestamp, created_timestamp, updated_timestamp, status, severity, severity_name, confidence, name, display_name, description, type, product, platform, tactic, tactic_id, technique, technique_id, objective, agent_id, cmdline, filename, filepath, sha256, md5, assigned_to_name, assigned_to_uuid, resolution, tags` (32 named fields) | TOML `detections` columns: `id, created_timestamp, status, severity, device_id, tactic, technique` (7 columns) | DTU `detections-detail.json` fixture: uses `detection_id` as the key field | **CRITICAL** — DTU fixture keys records by `detection_id`, but TOML step 2 body template uses `${query_detection_ids}` from step 1's `resources` array. Step 1 returns IDs; step 2 sends those IDs. TOML column list has `id` (not `detection_id`). If the DTU fixture returns records with `detection_id` key but TOML column is `id`, the JSONPath extraction `$.resources` may not find a field named `id`. Need to verify fixture field names match TOML column names. | CRITICAL | See Gap-CS-001 below |
| **OCSF mapping** | Poller sends raw alert JSON to Vector sink. No OCSF in poller-cobra. | TOML `ocsf_class = "security_finding"` + per-column `ocsf_field` mappings | DTU serves raw fixture; OCSF is applied by prism pipeline, not DTU | **NO** — OCSF is prism's responsibility, not the poller's. Architecture is correct. | — | None |
| **Rate limiting** | No explicit rate-limit handling in poller-cobra (SDK handles internally) | `[rate_limit_hints] requests_per_second = 10.0` | DTU: FailureMode::RateLimit injectable via configure endpoint | **NO** — rate limit hints exist. Demo path unlikely to hit limits. | — | None |
| **Edge cases: 401 token refresh** | gofalcon SDK handles 401 token refresh transparently | Plugin: PluginAuthProvider handles token lifecycle | DTU: `auth_mode = "reject"` injectable | **NO** — plugin handles refresh per its contract | — | None |

### 1.2 Gap Detail: Gap-CS-001 (CRITICAL)

**Finding:** The DTU fixture files for CrowdStrike use `detection_id` as the primary key field
(`routes/detections.rs`: `if let Some(id) = record.get("detection_id") ...`), but the TOML
`detections` table declares a column named `id` with `ocsf_field = "finding.uid"`.

**Impact:** When the pipeline executes step 2 (`POST /detects/entities/summaries/GET/v1`)
and applies JSONPath `$.resources` to extract detection records, those records will have
`detection_id` as the ID field. The TOML `id` column (mapped to `finding.uid`) will be
`NULL` for every row in the resulting Arrow batch. This means the live demo will produce
a table with an empty `id` column, which is a visible defect during query output.

**Root cause:** The prism TOML was authored using `id` (generic name) while the DTU fixture
uses `detection_id` (vendor-specific field name from the CrowdStrike API). The actual
CrowdStrike Falcon alerts/detections API returns records with a `composite_id` field as the
primary identifier (per poller-cobra `alertToMap`: `id`, `composite_id`).

**Disposition:** Fix-in-scope. Rename TOML column from `id` to `detection_id` to match both
the DTU fixture and the real CrowdStrike API field. Update `ocsf_field = "finding.uid"` mapping
to target the correct field.

**Note:** This requires a TOML spec edit (architect domain). No code changes needed in DTU.

---

## 2. Claroty xDome (poller-bear)

### 2.1 Fidelity Gap Audit Table

| Dimension | Poller Reality | Prism TOML | DTU Clone | Gap? | Severity | Action |
|-----------|---------------|------------|-----------|------|----------|--------|
| **Auth mechanism** | Bearer token (`Authorization: Bearer <CLAROTY_API_KEY>`). Static key loaded from `CLAROTY_API_KEY` env var. No session/refresh. All 9 endpoints use the same Bearer pattern. | `auth_type = "bearer_static"` — D-747 LOCKED. NOTE: legacy `ClarotyAuth::auth_type_name()` returned "cookie_roundtrip" (deleted by PLUGIN-MIGRATION-001-A). | `check_bearer_auth()` in `routes/devices.rs` enforces `Authorization: Bearer {non-empty}`. Same check in `routes/alerts.rs` (calls `check_bearer_auth`). | **NO** — fully aligned. `bearer_static` matches poller reality. The legacy "cookie_roundtrip" label in deleted code was a bug that is now gone. | — | None |
| **API base URL** | `https://api.claroty.com` (default), overridable via `CLAROTY_BASE_URL`. | `base_url = "${env.CLAROTY_INSTANCE_URL}"` — env-var driven. | DTU: ephemeral 127.0.0.1:PORT | **NO** — prism uses env var, poller uses env var. Same pattern. | — | None |
| **Endpoint pattern** | **ALL endpoints use POST** even for read-only queries. JSON body with `offset`, `limit`, `fields`, `sort_by`, `filter_by`. Base paths: `/api/v1/alerts/`, `/api/v1/devices/`, `/api/v1/audit_log/get`, `/api/v1/sites/get`, etc. | TOML `alerts`: `method = "POST"`, `path_template = "/api/v1/alerts"`. TOML `audit_logs`: `method = "POST"`, `path_template = "/api/v1/audit_logs"`. | DTU: `POST /api/v1/alerts`, `POST /api/v1/devices`, `POST /api/v1/alerts/:alert_id/devices`, `POST /api/v1/vulnerabilities`. | **HIGH** — Poller uses trailing slash on most endpoints (e.g. `/api/v1/alerts/`, not `/api/v1/alerts`). Prism TOML uses no trailing slash. Real Claroty API may be slash-sensitive. For the DTU demo this doesn't matter (DTU matches TOML). For a live-API demo, trailing slash mismatch could produce 404 or 301 redirects. | HIGH | See Gap-CL-001 below. |
| **Audit log endpoint** | Poller uses `POST /api/v1/audit_log/get` (note: `/get` suffix, not `/audit_logs`). | TOML `audit_logs` table: `path_template = "/api/v1/audit_logs"` | DTU: no `/api/v1/audit_logs` route registered | **HIGH** — Two mismatches: (1) poller path is `/api/v1/audit_log/get` not `/api/v1/audit_logs`, (2) DTU has no audit_logs route at all (DTU-audit-gap noted in TOML comment). Demo queries against `claroty_audit_logs` will fail against DTU (404) and would fail against a real Claroty xDome instance (wrong path). | HIGH | See Gap-CL-002 below. |
| **Device endpoint** | Poller: `POST /api/v1/devices/` (trailing slash). Response key: `devices`. | TOML (devices table): **NOT PRESENT** in TOML — only `alerts` and `audit_logs` tables exist. | DTU: `POST /api/v1/devices` route exists. Response: `{"devices": [...], "total": N, "page": N}`. | **CRITICAL** — The `claroty.sensor.toml` has NO `devices` table. Prism cannot query Claroty devices via the spec-driven adapter. The DTU has a working devices route, the poller has a working devices API, but the TOML bridge is missing. S-DEMO-002 ACs must not include `claroty_devices` queries. | CRITICAL | See Gap-CL-003 below. |
| **Pagination** | Poller: `offset + limit` in POST body (`{"offset": N, "limit": 100, "fields": [...], "sort_by": [...], "filter_by": {...}}`). No URL-level pagination params. | TOML `alerts` step: `type = "offset_limit"`, `page_size = 100`. Comment: "OffsetLimit engine appends `?offset=N&limit=M` to URL". | DTU `GetAlertsBody`: accepts `offset: Option<u32>` + `limit: Option<u32>` in POST body. No URL query params for pagination. | **HIGH** — Critical mismatch: poller sends offset/limit in POST body; TOML/pipeline appends `?offset=N&limit=M` as URL query params. Real Claroty API and DTU accept body pagination; the pipeline sends URL params. Against DTU this means paginated requests always get page 1 (DTU ignores URL params). Against real Claroty, same issue — URL params ignored, returns first page repeatedly. | HIGH | See Gap-CL-004 below. |
| **Auth credentials** | `CLAROTY_API_KEY` env var (or `_FILE` variant). Simple static Bearer. | `auth_type = "bearer_static"` — `BearerStaticAuthProvider` reads cred from keychain/env. | DTU: `check_bearer_auth` validates any non-empty Bearer token. | **NO** — auth credential flow is correct. | — | None |
| **Alert response fields** | Poller Alert struct: 20 fields including `id` (polymorphic int-or-string), `alert_name`, `alert_type_name`, `alert_class`, `category`, `detected_time`, `updated_time`, `devices_count`, `status`, `mitre_technique_ics_ids/names`, `mitre_technique_enterprise_ids/names`, `description`, `malicious_ip_tags_list`. | TOML `alerts` columns: `id, type, severity, status, created_at, updated_at, device_id, description` (8 columns). | DTU `ClarotyAlert` struct: `alert_type_name, category, description, detected_time, devices_count, id, iot_devices_count, it_devices_count, medical_devices_count, mitre_technique_enterprise_ids/names, mitre_technique_ics_ids/names, status, unresolved_devices_count, updated_time`. | **MEDIUM** — TOML column `type` → poller field is `alert_type_name`. DTU has `alert_type_name`. TOML column `severity` has NO equivalent in either poller or DTU — the real Claroty alerts API does NOT return a `severity` field per poller-bear's `Alert` struct. `created_at` in TOML → poller uses `detected_time`. These are mapping name mismatches. | MEDIUM | See Gap-CL-005 below. |
| **OCSF mapping** | Poller sends enriched JSON to Vector. OCSF is a stub (mapper returns nil). | TOML `ocsf_class = "security_finding"` per-column ocsf_field mappings. | DTU: raw fixture data, no OCSF. Prism pipeline applies OCSF. | **NO** — same pattern as CrowdStrike. Prism owns OCSF. | — | None |

### 2.2 Gap Details

**Gap-CL-001 (HIGH): Trailing slash on API endpoints**

Poller-bear uses trailing slashes: `/api/v1/alerts/`, `/api/v1/devices/`, etc.
TOML and DTU use no trailing slash. Against the DTU this is harmless (axum routes
match both). Against the real Claroty xDome API, the trailing slash is significant —
Claroty may return 301 redirects or 404s for path mismatches.

Disposition: MEDIUM severity for DTU demo (harmless); HIGH severity for live Claroty.
Document in demo runbook. Add trailing slash to TOML paths as follow-up story.

**Gap-CL-002 (HIGH): Audit log path mismatch**

Poller path: `POST /api/v1/audit_log/get` (with `/get` suffix).
TOML path: `POST /api/v1/audit_logs` (no `/get` suffix, different name).
DTU: no audit_logs route at all.

The TOML spec notes "DTU-audit-gap: Claroty DTU does not yet expose /api/v1/audit_logs."
But even the TOML path is wrong — the real Claroty path is `/api/v1/audit_log/get`.

Disposition: Fix TOML `audit_logs` table `path_template` to `/api/v1/audit_log/get`. Add DTU
route as a follow-up story. Demo should avoid querying `claroty_audit_logs` until fixed.

**Gap-CL-003 (CRITICAL): Missing `devices` table in TOML**

The Claroty sensor is one of the richest device-inventory data sources in the MSSP stack.
The DTU has a fully working `POST /api/v1/devices` endpoint. The poller collects 16-field
device records. But `claroty.sensor.toml` has only `alerts` and `audit_logs` tables — no
`devices` table.

Disposition: Author a `devices` table in `claroty.sensor.toml`. This requires:
- `table_name = "devices"`, `ocsf_class = "device"`
- `method = "POST"`, `path_template = "/api/v1/devices"`
- Columns: `uid` (REQUIRED), `device_category`, `device_type`, `ip_list`, `mac_list`,
  `risk_score`, `retired`, `asset_id`
- Pagination: `type = "offset_limit"` — BUT see Gap-CL-004 about offset-in-body

This is a TOML spec addition (architect domain). Target: fix in this burst.

**Gap-CL-004 (HIGH): Pagination via POST body, not URL query params**

The OffsetLimit engine in the pipeline appends `?offset=N&limit=M` to the URL.
Claroty's API (and DTU) expect `{"offset": N, "limit": M}` in the POST body.
Result: every paginated fetch after page 1 returns the first page again (offset ignored).

Disposition: This requires a pipeline change (`PipelineExecutor` / `OffsetLimit` logic).
Scope: `prism-spec-engine`. Surface as new story with explicit dependency on current
pagination implementation. The DTU already accepts body-based pagination, so a pipeline
fix would be testable immediately.

**Gap-CL-005 (MEDIUM): Alert field name mismatches**

| TOML column | Real Claroty field | DTU field | Match? |
|-------------|-------------------|-----------|--------|
| `id` | `id` (polymorphic int/string) | `id: u32` | YES |
| `type` | `alert_type_name` | `alert_type_name` | NO — TOML uses `type`, real field is `alert_type_name` |
| `severity` | NOT in poller Alert struct | NOT in DTU ClarotyAlert | NO — severity column doesn't exist in the real Claroty alerts API |
| `status` | `status` | `status` | YES |
| `created_at` | `detected_time` | `detected_time` | NO — TOML uses `created_at`, real field is `detected_time` |
| `updated_at` | `updated_time` | `updated_time` | NO — TOML uses `updated_at`, real field is `updated_time` |
| `device_id` | NOT a direct field — poller uses `devices_count` and relations | NOT in ClarotyAlert | NO — `device_id` doesn't exist in the alerts endpoint |
| `description` | `description` | `description` | YES |

Summary of mismatches: `type` → `alert_type_name`; `severity` → absent; `created_at` → `detected_time`; `updated_at` → `updated_time`; `device_id` → absent.

Disposition: Fix TOML column names. The `severity` and `device_id` columns should be removed or replaced with accurate fields. `type` → rename to `alert_type_name`. Timestamps to match real field names.

---

## 3. Armis Centrix (poller-coaster)

### 3.1 Fidelity Gap Audit Table

| Dimension | Poller Reality | Prism TOML | DTU Clone | Gap? | Severity | Action |
|-----------|---------------|------------|-----------|------|----------|--------|
| **Auth mechanism** | Bearer token via Armis SDK (`centrix.NewClient(apiKey, baseURL, ...)`). SDK injects `Authorization: Bearer` on every request. No OAuth2, no session, no token refresh. `ARMIS_API_KEY` env var. | `auth_type = "bearer_static"` — D-747 LOCKED. NOTE: legacy `ArmisAuth::auth_type_name()` returned "api_key" (deleted by PLUGIN-MIGRATION-001-A). | DTU `routes/devices.rs`: `Authorization: Bearer {non-empty}` required; 403 on missing/empty token. | **NO** — `bearer_static` matches poller reality. The legacy "api_key" label was a label bug (the actual header was always Bearer, not an API-Key header). | — | None |
| **API base URL** | Default: `https://lab-1898andco.armis.com`. Override via `ARMIS_API_URL`. | `base_url = "${env.ARMIS_INSTANCE_URL}"` | DTU: ephemeral 127.0.0.1:PORT | **NO** — pattern matches. | — | None |
| **Sole API operation** | poller-coaster uses ONE endpoint: `GetSearch(ctx, aql, false, false)` via the armis-sdk-go. Internally this calls `GET /api/v1/search?aql=<query>` (or equivalent SDK call). All 7 data sources go through this single AQL endpoint. | TOML: `GET /api/v1/devices` and `GET /api/v1/alerts` (TWO different endpoints). | DTU: `GET /api/v1/devices`, `GET /api/v1/alerts` (separate routes, not AQL search). | **MEDIUM** — Production Armis uses AQL search endpoint for all queries. Prism uses direct resource endpoints (DTU-grounded per ADR-028 §D5 notes DTU-EXT-003/004). The real Armis API does have both patterns. For demo purposes the DTU endpoints work; for a live Armis demo, AQL would be the more authentic query path. Not a runtime defect for DTU. | MEDIUM | Documented gap DTU-EXT-003/004. Follow-up story for AQL endpoint support. |
| **Pagination** | poller-coaster does NOT use pagination in AQL queries — it fetches a full result set and filters client-side using cursor comparison. No offset/page params sent to Armis API. | TOML: `type = "offset_limit"`, `page_size = 25` on both `devices` and `alerts` steps. | DTU `routes/devices.rs`: `paginate_devices` function uses `page`/`size` URL params: `size.unwrap_or(25)`. DTU supports offset pagination. | **MEDIUM** — Poller doesn't paginate at the API level; DTU and TOML use offset pagination. This is an acceptable divergence for the DTU path: the DTU implements pagination, prism uses it, the demo gets correct results. Against a real Armis instance, the AQL endpoint behavior may differ from offset pagination. | MEDIUM | Acceptable for demo; document in runbook. |
| **Auth credentials** | `ARMIS_API_KEY` env var. Injected as Bearer by SDK. | `auth_type = "bearer_static"` — BearerStaticAuthProvider reads token from credential store. | DTU: any non-empty Bearer token accepted. | **NO** — correctly aligned. | — | None |
| **Device response fields** | poller-coaster SDK `SearchResult` fields used for cursor: `LastSeen, FirstSeen, ID, Title`. Fields requested: `id, name, ipAddress, macAddress, manufacturer, model, operatingSystem, riskLevel, type, category, purdueLevel, firstSeen, lastSeen, tags, visibility`. | TOML `devices` columns: `device_id, name, type, manufacturer, last_seen, first_seen, ip_address, mac_address, os_name, risk_score` (10 columns). | DTU `DeviceRecord`: `device_id, name, ip_address, mac_address, type, manufacturer, os_name, os_version, risk_score, risk_factors, last_seen, first_seen, network_id, site, tags`. | **MEDIUM** — TOML `risk_score` = `integer`, DTU `risk_score: Option<u32>` — MATCH. TOML `os_name` → poller field is `operatingSystem`; DTU field is `os_name` — OK (DTU field name matches TOML). TOML `type` → poller field is `type`; DTU field is `device_type` (serialized as `type` via `#[serde(rename = "type")]`) — OK. Overall field alignment is GOOD. One low concern: poller's `category` (Armis device category) and `purdueLevel` are not in TOML. | LOW | Non-blocking. Add `category` and `purdue_level` columns in follow-up. |
| **Alert response fields** | poller-coaster alert fields: `title, status, severity, time, lastAlertUpdateTime, policyTitle, type, classification` (8 fields). Record type: `armis_alert`. Cursor: `lastAlertUpdateTime` primary, `time` fallback. | TOML `alerts` columns: `alert_id, name, severity, status, policy_name, device_id, created_at, updated_at` (8 columns). | DTU `AlertRecord`: `alert_id, name, severity, status, policy_name, device_id, created_at, updated_at`. | **MEDIUM** — Poller `policyTitle` → TOML `policy_name` → DTU `policy_name` — OK (rename is acceptable). Poller `title` → TOML `name` → DTU `name` — OK. BUT: poller `type` (alert type classification) and `classification` are NOT in TOML. Also, `device_id` in TOML → poller has no direct `device_id` field in its SearchResult alert data (device association is via separate AQL). | MEDIUM | Non-blocking for demo. Follow-up story to add `alert_type` and `classification` columns. |
| **Timestamp format** | Poller uses `lastAlertUpdateTime` and `time` fields (both RFC3339 strings from Armis). SDK `SearchResult` fields are strings. | TOML `created_at: datetime`, `updated_at: datetime`. No explicit `timestamp_formats`. | DTU `AlertRecord.created_at: String`, `updated_at: String` — RFC3339. | **NO** — both use RFC3339 strings. Prism datetime normalization handles RFC3339 as default. | — | None |
| **OCSF mapping** | poller-coaster sends raw data to Vector. No OCSF. | TOML `ocsf_class = "device"/"security_finding"` per table. | DTU serves raw fixture. Prism applies OCSF. | **NO** — correct architecture. | — | None |
| **Rate limiting** | No explicit rate limiting in poller-coaster. Relies on SDK/API-side limits. | No `[rate_limit_hints]` block. | DTU: FailureMode::RateLimit injectable. | **LOW** — no rate limit hints configured. Unlikely to matter for demo. | LOW | Add rate_limit_hints as low-priority follow-up. |

### 3.2 Gap Details

**Gap-AR-001 (MEDIUM): AQL vs direct endpoint**

This is an architectural difference: the real Armis API is query-oriented (AQL), but prism
uses direct REST endpoints (as does the DTU). For a demo against the DTU this is fine — the
DTU returns correct data. For a future live-Armis demo, AQL support would be required.

Disposition: Documented gap DTU-EXT-003/004. Not a demo blocker.

---

## 4. Cyberint Argos (poller-express)

### 4.1 Fidelity Gap Audit Table

| Dimension | Poller Reality | Prism TOML | DTU Clone | Gap? | Severity | Action |
|-----------|---------------|------------|-----------|------|----------|--------|
| **Auth mechanism** | Cookie-based authentication. `cookieTransport` injects `access_token={apiKey}` cookie on every request. API key is NOT sent as Bearer. Pattern: inject cookie on every call — NO login step required. | `auth_type = "cookie_roundtrip"` — D-747 LOCKED. `CookieLoginAuthProvider` expected to: POST `/login` → capture `cyberint_session` Set-Cookie. | DTU: `POST /login` returns `Set-Cookie: cyberint_session={uuid}`. Routes extract `cyberint_session` cookie. | **CRITICAL** — The poller uses cookie name `access_token` (injected directly, no login step). The DTU uses cookie name `cyberint_session` (issued by `/login`). The TOML `auth_type = "cookie_roundtrip"` maps to `CookieLoginAuthProvider` which calls `/login` then uses `cyberint_session`. This is a fundamental mismatch: (1) wrong cookie name (`access_token` vs `cyberint_session`), (2) wrong flow (direct injection vs login step). | CRITICAL | See Gap-CY-001 (already identified in original mandate). |
| **Alert endpoint** | `POST {baseURL}/alert/api/v1/alerts` (note: the alert subdomain path includes `/alert/` prefix before `/api/v1`). The OpenAPI client's server URL is `{baseURL}/alert`. | TOML: `GET /api/v1/alerts` (no `/alert/` prefix). | DTU: `GET /api/v1/alerts` (no prefix, matching TOML). | **HIGH** — Real Cyberint API path is `POST /alert/api/v1/alerts` but both TOML and DTU use `GET /api/v1/alerts`. The DTU demo path is self-consistent (both use same path), so demo against DTU works fine. Against live Cyberint: (a) wrong method (GET vs POST), (b) wrong base path (missing `/alert/` prefix). | HIGH | See Gap-CY-002. Demo-against-DTU: NO IMPACT. Demo against live Cyberint: CRITICAL. |
| **Asset endpoint** | `POST {baseURL}/asset-configuration/external/api/v1/assets/` (hand-written client). Accepts `customer_id`, `page_number`, `type`, `status`. | No `assets` table in TOML. | No asset route in DTU. | **MEDIUM** — Assets are not in scope for the demo (no TOML table, no DTU route). This is a known gap. No demo impact. | MEDIUM | Follow-up story for asset table. Not a demo blocker. |
| **Alert pagination** | Poller: page-based, `{"page": 1, "size": 100, ...}`. If page is full (`len == pageSize`), assumes more and re-fetches with updated cursor. | TOML: `type = "cursor_token"`, `cursor_response_path = "$.next_cursor"`. | DTU: `GET /api/v1/alerts?cursor=...` — returns `{"data": [...], "next_cursor": "token_or_null"}`. | **MEDIUM** — Poller uses page-number pagination; TOML uses cursor-token pagination; DTU uses cursor-token. For the DTU demo, cursor-token works correctly (DTU emits `next_cursor`). Against real Cyberint: the real API uses page-number (`page`, `size`), so cursor-token won't work. But demo-against-DTU is fine. | MEDIUM | Document discrepancy. Live-Cyberint demo requires page-number pagination implementation. |
| **Alert fields** | Poller Alert (OpenAPI-generated): `Id (int32), RefId (string), Environment, Confidence, Status, Severity, Category, Type, SourceCategory, Source, TargetedVectors, TargetedBrands, RelatedEntities, Impacts, Title, Description, Recommendation, AlertData, Iocs, Indicators, Tags, Attachments, Mitre, RelatedAssets, CreatedDate, ModificationDate, UpdateDate, ClosureDate, ClosureReason, AcknowledgedDate, ThreatActor, TicketId, AnalysisReport, AssignedTo, CreatedBy, ClosedBy, AcknowledgedBy`. Cursor uses `RefId` + `ModificationDate`. | TOML `alerts` columns: `alert_id, title, type, severity, status, created_at, source` (7 columns). | DTU `Alert` struct: `alert_id, title, severity, status, created_at (serde_json::Value), source, alert_type, affected_assets`. | **MEDIUM** — TOML `alert_id` → DTU `alert_id` → real poller `RefId` (string unique ID). This is a name mismatch: real API field is `RefId`, DTU uses `alert_id`, TOML uses `alert_id`. Against DTU: works fine. Against live Cyberint: `alert_id` column would be NULL (real API has no `alert_id` field, uses `ref_id` or `id`). TOML `type` → DTU `alert_type` — name mismatch. | MEDIUM | See Gap-CY-003. Demo-against-DTU: NO IMPACT. Live-Cyberint: field names wrong. |
| **Timestamp format** | `CyberintTime` handles RFC3339, no-tz RFC3339, microsecond variant, and "null"/empty → zero. | TOML `created_at` with `timestamp_formats = ["iso8601", "unix_epoch_seconds"]`. | DTU `Alert.created_at: serde_json::Value` — flexible format. | **NO** — TOML timestamp_formats handles the multi-format nature. | — | None |
| **Auth credentials** | `CYBERINT_API_KEY` env var. Injected as `access_token` cookie. | `auth_type = "cookie_roundtrip"` — CookieLoginAuthProvider expected to do login step. | DTU: issues `cyberint_session` cookie on POST `/login`. | **CRITICAL** — Same as auth mechanism gap: cookie name mismatch `access_token` vs `cyberint_session`. | CRITICAL | Gap-CY-001. |

### 4.2 Gap Details

**Gap-CY-001 (CRITICAL): Cookie name mismatch — `access_token` vs `cyberint_session`**

This gap was identified in the original mandate. Full analysis:

- **Poller reality** (poller-express broad-sweep §2.1): The `cookieTransport` injects
  `Cookie: access_token={apiKey}` on every request. This is a static API-key-as-cookie
  pattern. NO login step. The API key itself is the session identifier.

- **DTU behavior** (routes/auth.rs `post_login`): `POST /login` issues
  `Set-Cookie: cyberint_session={uuid_token}`. Subsequent routes extract `cyberint_session`
  cookie via `extract_session_token()`.

- **TOML spec** (`auth_type = "cookie_roundtrip"`): Maps to `CookieLoginAuthProvider` which
  performs a login step (POST `/login`), captures the `Set-Cookie` response, then uses that
  cookie on subsequent requests.

The DTU implements `cyberint_session` (with login step). The TOML implements `cookie_roundtrip`
(with login step, emitting `cyberint_session` via DTU). For the **demo-against-DTU path**, this
is self-consistent: TOML → CookieLoginAuthProvider → POST /login → receives `cyberint_session`
→ uses on subsequent calls → DTU validates `cyberint_session`.

For a **live-Cyberint demo**, this would fail: real Cyberint API expects `access_token` cookie
injected directly, not a login step.

**Disposition for DTU demo:** Demo against DTU is CONSISTENT and will work. The
cookie-name mismatch is only relevant for live-Cyberint scenarios. Document this in S-DEMO-001
ACs so the implementer builds `CookieLoginAuthProvider` to match the DTU's `/login` → `cyberint_session` flow, NOT the real Cyberint `access_token` direct injection.

**Gap-CY-002 (HIGH): Real Cyberint alert endpoint path**

Real path: `POST {baseURL}/alert/api/v1/alerts` (POST method, `/alert/` prefix).
TOML + DTU: `GET /api/v1/alerts` (GET method, no prefix).

For DTU demo: self-consistent, works fine.
For live-Cyberint demo: (a) HTTP method must be POST, (b) base URL for alert calls is `{base}/alert/`.

Disposition: Note in S-DEMO-001 story and demo runbook. This is HIGH severity only if
live-Cyberint is in scope for the demo. If demo-against-DTU only: LOW impact.

**Gap-CY-003 (MEDIUM): `alert_id` vs `ref_id` field naming**

Real Cyberint API primary identifier: `RefId` (string, per OpenAPI model).
TOML + DTU primary identifier: `alert_id` (string).

For DTU demo: TOML `alert_id` → DTU `Alert.alert_id` — works.
For live-Cyberint: would produce NULL `alert_id` (real field is `ref_id` per OpenAPI spec).

Disposition: Document in demo runbook. Not a blocker for DTU demo.

---

## 5. Cross-Sensor Summary: Per-Sensor Gap List

### CrowdStrike

| Gap ID | Dimension | Severity | Disposition |
|--------|-----------|----------|-------------|
| Gap-CS-001 | DTU fixture uses `detection_id`; TOML column is `id` → NULL id column in query output | CRITICAL | Fix TOML: rename column `id` → `detection_id` |
| Gap-CS-002 | Poller polls alerts API (`/alerts`); prism spec exposes detections API (`/detects`) | MEDIUM | Document in demo script. Demo queries detections table. |
| Gap-CS-003 | Base URL hardcoded for us-1 only | MEDIUM | Follow-up story for multi-region |

### Claroty

| Gap ID | Dimension | Severity | Disposition |
|--------|-----------|----------|-------------|
| Gap-CL-001 | Trailing slash on endpoint paths (poller uses `/api/v1/alerts/`) | HIGH | Follow-up story to add trailing slash to TOML paths |
| Gap-CL-002 | Audit log path wrong: TOML `/api/v1/audit_logs` vs real `/api/v1/audit_log/get` | HIGH | Fix TOML path. Add DTU route as follow-up. |
| Gap-CL-003 | `devices` table missing from TOML entirely | CRITICAL | Author `devices` table in TOML (fix in this burst) |
| Gap-CL-004 | Offset pagination sent as URL params; Claroty API expects body params | HIGH | Surface as pipeline story (prism-spec-engine) |
| Gap-CL-005 | Alert column names don't match real Claroty fields (`type` vs `alert_type_name`, `severity` absent, `created_at` vs `detected_time`) | MEDIUM | Fix TOML column names |

### Armis

| Gap ID | Dimension | Severity | Disposition |
|--------|-----------|----------|-------------|
| Gap-AR-001 | Poller uses AQL search endpoint; prism uses direct resource endpoints | MEDIUM | Documented gap DTU-EXT-003/004. Not demo blocker. |
| Gap-AR-002 | Poller's `type`, `classification` alert fields absent from TOML | LOW | Follow-up story |
| Gap-AR-003 | `category`, `purdueLevel` device fields absent from TOML | LOW | Follow-up story |

### Cyberint

| Gap ID | Dimension | Severity | Disposition |
|--------|-----------|----------|-------------|
| Gap-CY-001 | Cookie name mismatch (`access_token` vs `cyberint_session`) — self-consistent for DTU demo | CRITICAL (live) / NONE (DTU) | Document in S-DEMO-001 ACs. DTU demo path is consistent. |
| Gap-CY-002 | Real API path `POST /alert/api/v1/alerts` vs TOML/DTU `GET /api/v1/alerts` | HIGH (live) / NONE (DTU) | Document in runbook. DTU demo path is consistent. |
| Gap-CY-003 | `alert_id` vs real `RefId` field name | MEDIUM (live) / NONE (DTU) | Document in runbook. DTU demo path works. |

---

## 6. Demo-Readiness Summary

| Sensor | READY for DTU Demo? | Gating Items | Notes |
|--------|---------------------|--------------|-------|
| **CrowdStrike** | CONDITIONAL | Gap-CS-001 MUST fix: rename TOML column `id` → `detection_id` | After fix: queries against `crowdstrike_detections` will return correct id column. Demo script should query this table. |
| **Claroty** | NO | Gap-CL-003 (CRITICAL): missing `devices` table; Gap-CL-004 (HIGH): pagination broken — paginated alerts return page 1 only | Immediate TOML fixes: (1) add `devices` table, (2) fix alert column names. Pagination issue requires pipeline story and may limit demo to small (<100 row) result sets. Demo can show alerts (page 1 only) and devices (once table is added). |
| **Armis** | YES (limited) | None blocking. Gap-AR-001 is documented/acceptable. | Demo can query `armis_devices` and `armis_alerts`. AQL is not required for DTU demo. |
| **Cyberint** | YES (DTU path) | None for DTU demo. Cookie name + endpoint path mismatches only affect live-Cyberint. | `CookieLoginAuthProvider` must implement login step → `cyberint_session` cookie (NOT direct `access_token` injection). This is the S-DEMO-001 implementation contract. Demo queries `cyberint_alerts`. |

---

## 7. Fixes Required Before Demo (TOML-Only — No Code Changes)

The following TOML changes are within architect scope and should be committed in this burst:

### Fix 1: crowdstrike.sensor.toml — rename `id` column in `detections` table

Change `name = "id"` to `name = "detection_id"` in the `detections` table's primary
key column. Keep `ocsf_field = "finding.uid"`.

This aligns the TOML column name with the DTU fixture key (`detection_id`) and the real
CrowdStrike API detection record primary identifier.

### Fix 2: claroty.sensor.toml — add `devices` table

Add a new `[[tables]]` block for Claroty devices:
- `table_name = "devices"`, `ocsf_class = "device"`
- Columns: `uid` (string, REQUIRED), `device_category`, `device_type`, `ip_list`, `mac_list`, `risk_score`, `retired`, `asset_id`
- Step: `POST /api/v1/devices`, response_path `$.devices`, offset_limit pagination
- NOTE: pagination type = "offset_limit" with body params is currently broken (Gap-CL-004) —
  include the pagination config so it is correct when the pipeline is fixed.

### Fix 3: claroty.sensor.toml — fix alert column names

Rename columns to match DTU field names:
- `type` → `alert_type_name` (matches DTU `ClarotyAlert.alert_type_name`)
- Remove `severity` (no such field in Claroty alerts API)
- `created_at` → `detected_time` (matches DTU `ClarotyAlert.detected_time`)
- `updated_at` → `updated_time` (matches DTU `ClarotyAlert.updated_time`)
- Remove `device_id` (not a direct field on the alerts endpoint; comes from relations)

### Fix 4: claroty.sensor.toml — fix audit_log path

Change `path_template = "/api/v1/audit_logs"` to `path_template = "/api/v1/audit_log/get"`
to match the real Claroty xDome API endpoint (per poller-bear).

---

## 8. Stories to Surface (Not In-Scope for This Burst)

These require product-owner/story-writer dispatch, NOT architect TOML edits:

| Story ID (proposed) | Scope | Priority |
|--------------------|-------|----------|
| S-DEMO-CLAROTY-PAGINATION-001 | Fix OffsetLimit pipeline to send offset/limit in POST body for POST-method steps | P1 — blocks multi-page Claroty queries |
| S-DEMO-CLAROTY-AUDIT-DTU-001 | Add `/api/v1/audit_log/get` route to prism-dtu-claroty | P2 |
| S-DEMO-CLAROTY-TRAILING-SLASH-001 | Add trailing slash to Claroty TOML paths; verify real API behavior | P2 |
| S-DEMO-CROWDSTRIKE-MULTIREGION-001 | Add region-based base_url override to CrowdStrike sensor spec | P3 |
| S-DEMO-ARMIS-AQL-001 | Add AQL search endpoint support to Armis DTU and TOML | P2 |
| S-DEMO-CYBERINT-LIVE-AUTH-001 | Implement direct `access_token` cookie injection for live-Cyberint | P2 (post-demo) |

---

## 9. S-DEMO-001 / S-DEMO-002 AC Amendments Required

### S-DEMO-001 Amendments

The existing S-DEMO-001 v1.1 already captures the Cyberint auth model correction. No
additional AC amendments are required for S-DEMO-001 based on this audit, EXCEPT:

- **AC for CookieLoginAuthProvider**: Must specify it uses `cyberint_session` cookie name
  (matching DTU), NOT `access_token` (matching real Cyberint). The DTU demo path requires
  `cyberint_session`. Ensure S-DEMO-001 ACs explicitly state this.

### S-DEMO-002 Amendments

S-DEMO-002 AC table (smoke test) should be updated:

1. **CrowdStrike**: Query `crowdstrike_detections`, expect `detection_id` column (not `id`)
   — after Gap-CS-001 TOML fix lands.

2. **Claroty**: Query `claroty_alerts` AND `claroty_devices` (once Gap-CL-003 devices table fix
   lands). Expect column names to match fixed TOML: `alert_type_name`, `detected_time`, `updated_time`.

3. **Armis**: Query `armis_devices` and `armis_alerts`. No AC changes needed.

4. **Cyberint**: Query `cyberint_alerts`. Expect `alert_id` column to be populated (DTU path).

---

## 10. Appendix: Auth Mechanism Summary (All 4 Sensors)

| Sensor | Production (Poller) Auth | Prism TOML auth_type | DTU Auth Enforcement | Demo Path Consistent? |
|--------|--------------------------|---------------------|---------------------|----------------------|
| CrowdStrike | OAuth2 client-credentials via gofalcon SDK → `Authorization: Bearer` | `oauth2_client_credentials` + `auth_plugin = "crowdstrike-oauth2"` | `POST /oauth2/token` returns Bearer token; downstream routes require `Authorization: Bearer` | YES |
| Claroty xDome | `Authorization: Bearer <CLAROTY_API_KEY>` static | `bearer_static` | `check_bearer_auth()` enforces `Authorization: Bearer {non-empty}` → 401 on empty | YES |
| Armis Centrix | `Authorization: Bearer <ARMIS_API_KEY>` via SDK | `bearer_static` | `Authorization: Bearer {non-empty}` required → 403 on missing | YES |
| Cyberint Argos | `Cookie: access_token={API_KEY}` (no login step) | `cookie_roundtrip` | `POST /login` → `Set-Cookie: cyberint_session={uuid}` → routes require `cyberint_session` cookie | YES (DTU-path only — login step matches DTU but differs from real API) |
