---
document_type: architect-audit
title: "Cross-Poller DTU Fidelity Audit — All 4 Sensors (2026-05-29)"
author: architect
date: "2026-05-29"
status: FINAL
version: "1.1"
revision_notes: |
  v1.1 (2026-05-29): Re-evaluated ALL gap rows under the DTU=true-DTU principle (ADR-031).
  Key changes from v1.0:
  - Gap-CY-001: FLIPPED from "DTU demo path consistent, document for live" to
    "CRITICAL pre-demo — DTU must be corrected to emit access_token cookie".
    Story reclassified: S-DEMO-CYBERINT-LIVE-AUTH-001 (P2-post-demo) →
    S-DTU-CYBERINT-AUTH-FIDELITY-001 (P0-pre-demo-BLOCKING).
  - All sensor tables: added "DTU change required: Y/N" and "Real-API reference" columns.
  - Per-sensor "READY for live demo" table updated: Cyberint is now N.
  - Cross-sensor DTU-shortcut audit added (§7).
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
  - .factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md
governing_principle: ADR-031 DTU=true-DTU (2026-05-29)
---

# Cross-Poller DTU Fidelity Audit — All 4 Sensors (v1.1)

## Purpose

Verify that prism's TOML sensor specs, DTU clones, and S-DEMO-001/002 ACs represent
true-to-life production behavior for a LIVE demo. Source of truth for each dimension:
the production poller repo semport docs (poller reality). Where TOML or DTU diverge
from the poller, that is a fidelity gap.

**Governing principle for v1.1:** ADR-031 DTU=true-DTU. Every gap row is re-evaluated
under this rule. DTU "simplifications" and "acceptable divergences" that were approved
in v1.0 are re-examined. If the DTU deviates from the real API in a way that affects
demo correctness, that deviation is NOT acceptable regardless of demo-path consistency.

## Severity Classification

- **CRITICAL** — runtime defect that makes the live demo fail or silently return wrong data
- **HIGH** — fidelity gap the live demo audience would notice
- **MEDIUM** — fidelity gap that could cause follow-up questions
- **LOW** — cosmetic or completeness gap not visible during demo queries

---

## 1. CrowdStrike Falcon (poller-cobra)

### 1.1 Fidelity Gap Audit Table

| Dimension | Poller Reality | Prism TOML | DTU Clone | Gap? | Severity | DTU change required | Real-API reference | Action |
|-----------|---------------|------------|-----------|------|----------|--------------------|--------------------|--------|
| **Auth mechanism** | OAuth2 client credentials via gofalcon SDK. Calls `/oauth2/token` with `client_id` + `client_secret`. Returns `access_token` Bearer. | `auth_type = "oauth2_client_credentials"`, `auth_plugin = "crowdstrike-oauth2"` | DTU `POST /oauth2/token` returns `{"access_token": "dtu-fake-cs-token", "token_type": "bearer", "expires_in": 3600}` | **NO** — fully aligned. | — | **N** | poller-cobra-broad-sweep §2.1 gofalcon SDK | None |
| **API base URL** | `https://api.crowdstrike.com` (us-1); region selectable via `Cloud` param | `base_url = "https://api.crowdstrike.com"` (us-1 hardcoded) | DTU listens on ephemeral 127.0.0.1:PORT | **MEDIUM** — Multi-region routing not in TOML. Demo environment is us-1. | MEDIUM | **N** | poller-cobra Cloud const | Follow-up S-DEMO-CROWDSTRIKE-MULTIREGION-001 (P3). Not a demo blocker. |
| **Detection field: primary key name** | `composite_id` (poller-cobra `alertToMap`) | `detection_id` (fixed develop@72baf413) | DTU fixture keyed by `detection_id` | **NO** — FIXED. Both TOML and DTU use `detection_id` after Gap-CS-001 fix. | — | **N** — DTU already uses `detection_id` | poller-cobra alertToMap | None; fix already applied. |
| **Pagination** | QueryV2 + PostEntities two-step; no explicit page loop | TOML: two-step ID-then-batch | DTU: returns IDs from fixture; accepts `{"ids": [...]}` | **NO** — aligned. | — | **N** | poller-cobra QueryV2 | None |

### 1.2 Remaining Gaps

| Gap ID | Dimension | Severity | DTU change required | Disposition |
|--------|-----------|----------|---------------------|-------------|
| Gap-CS-002 | Poller polls `/alerts` API; prism spec exposes `/detects` API | MEDIUM | **N** — demo queries detections, not alerts | Document in demo script. Demo queries `crowdstrike_detections`. |
| Gap-CS-003 | Base URL hardcoded for us-1 only | MEDIUM | **N** | Follow-up story S-DEMO-CROWDSTRIKE-MULTIREGION-001 (P3). |

### 1.3 READY for live demo: **YES** (post develop@72baf413 fixes)

---

## 2. Claroty xDome (poller-bear)

### 2.1 Fidelity Gap Audit Table

| Dimension | Poller Reality | Prism TOML | DTU Clone | Gap? | Severity | DTU change required | Real-API reference | Action |
|-----------|---------------|------------|-----------|------|----------|--------------------|--------------------|--------|
| **Auth mechanism** | Bearer token (`Authorization: Bearer <CLAROTY_API_KEY>`). Static key. No session/refresh. | `auth_type = "bearer_static"` (D-747 LOCKED) | `check_bearer_auth()` enforces `Authorization: Bearer {non-empty}`. | **NO** — fully aligned. | — | **N** | poller-bear-broad-sweep §2.1 | None |
| **Endpoint pattern** | ALL endpoints use POST, even read-only. Trailing slash on most endpoints. | TOML: POST on alerts, audit_logs. No trailing slash. | DTU: POST routes. Axum matches with/without trailing slash. | **MEDIUM** — trailing slash matters for live Claroty, not for DTU. | MEDIUM | **N** (DTU matches via axum normalization) | poller-bear-broad-sweep API table | Follow-up S-DEMO-CLAROTY-TRAILING-SLASH-001 (P2). |
| **Audit log endpoint** | `POST /api/v1/audit_log/get` (with `/get` suffix). | TOML `audit_logs`: `path_template = "/api/v1/audit_log/get"` (FIXED develop@72baf413). | DTU: no `/api/v1/audit_log/get` route. | **HIGH** — DTU route gap. Demo queries against `claroty_audit_logs` will fail (404). | HIGH | **YES** — add `/api/v1/audit_log/get` DTU route | poller-bear §4.3 audit_log path | S-DEMO-CLAROTY-AUDIT-DTU-001 (P2). Pre-demo if audit_logs queries are in demo script. |
| **Devices table** | `POST /api/v1/devices/` (trailing slash). Response key: `devices`. | TOML `devices` table ADDED (develop@72baf413). Columns: uid, device_category, device_type, ip_list, mac_list, risk_score, retired, asset_id. | DTU `POST /api/v1/devices` route EXISTS. Response: `{"devices": [...], "total": N}`. | **NO** — FIXED. TOML now has devices table aligned to DTU. | — | **N** | poller-bear §4.2 devices | None; fix already applied. |
| **Pagination** | offset + limit in POST body. | TOML: `type = "offset_limit"` (OffsetLimit engine appends `?offset=N&limit=M` to URL). | DTU: accepts offset/limit in POST body, NOT URL params. | **HIGH** — CRITICAL mismatch: pipeline sends URL params; DTU and real API expect body params. After page 1, pagination broken. | HIGH | **N** (DTU is correct; pipeline needs fixing) | poller-bear POST body schema | S-DEMO-CLAROTY-PAGINATION-001 (P1). Must fix pipeline before multi-page queries work. |
| **Alert column names** | `alert_type_name`, `detected_time`, `updated_time` (no `severity`, no `device_id`). | TOML: `alert_type_name`, `detected_time`, `updated_time` — FIXED (develop@72baf413). `severity` removed, `device_id` removed. | DTU `ClarotyAlert` struct: `alert_type_name`, `detected_time`, `updated_time` — match. | **NO** — FIXED. | — | **N** | poller-bear §3 Alert struct | None; fix already applied. |

### 2.2 Remaining Gaps

| Gap ID | Dimension | Severity | DTU change required | Disposition |
|--------|-----------|----------|---------------------|-------------|
| Gap-CL-001 | Trailing slash on endpoint paths | MEDIUM | **N** (axum handles it) | S-DEMO-CLAROTY-TRAILING-SLASH-001 (P2). |
| Gap-CL-004 | Offset pagination sent as URL params; Claroty API expects body params | HIGH | **N** (DTU correct; pipeline wrong) | S-DEMO-CLAROTY-PAGINATION-001 (P1). Limits demo to first-page results until fixed. |
| Gap-CL-006 (new) | DTU has no `/api/v1/audit_log/get` route | HIGH | **YES** — add route per real-API path | S-DEMO-CLAROTY-AUDIT-DTU-001 (P2). |

### 2.3 READY for live demo: **PARTIAL**

- `claroty_alerts` (page 1 only; pagination blocked by Gap-CL-004): READY
- `claroty_devices` (page 1 only): READY
- `claroty_audit_logs`: NOT READY (DTU route gap Gap-CL-006)

---

## 3. Armis Centrix (poller-coaster)

### 3.1 Fidelity Gap Audit Table

| Dimension | Poller Reality | Prism TOML | DTU Clone | Gap? | Severity | DTU change required | Real-API reference | Action |
|-----------|---------------|------------|-----------|------|----------|--------------------|--------------------|--------|
| **Auth mechanism** | Bearer token via Armis SDK (`Authorization: Bearer` on every request). `ARMIS_API_KEY` env var. No OAuth2, no session. | `auth_type = "bearer_static"` (D-747 LOCKED). | DTU `Authorization: Bearer {non-empty}` required; 403 on missing/empty token. | **NO** — fully aligned. | — | **N** | poller-coaster-broad-sweep §2.1 centrix.NewClient | None |
| **Sole API operation: AQL vs direct** | poller-coaster uses ONE endpoint: `GET /api/v1/search?aql=<query>` for all 7 data sources. | TOML: `GET /api/v1/devices` and `GET /api/v1/alerts` (two different endpoints). | DTU: `GET /api/v1/devices`, `GET /api/v1/alerts` (separate routes). | **MEDIUM** — Production uses AQL; DTU+prism use direct endpoints. For DTU demo: works fine. | MEDIUM | **N** (under ADR-031 D2 — permitted divergence with justification in D6) | poller-coaster GetSearch call | DTU-EXT-003/004 documented. Follow-up S-DEMO-ARMIS-AQL-001 (P2). |
| **Pagination** | poller-coaster does NOT use API pagination for AQL queries — fetches full set. | TOML: `type = "offset_limit"`, `page_size = 25`. | DTU: `page`/`size` URL params. DTU supports offset pagination. | **MEDIUM** — Acceptable for DTU path: DTU returns correct data; prism paginates. | MEDIUM | **N** | poller-coaster cursor comparison pattern | Document in runbook. Not a demo blocker for DTU path. |
| **Device response fields** | SDK fields: `id, name, ipAddress, macAddress, manufacturer, model, operatingSystem, riskLevel, type, category, purdueLevel, firstSeen, lastSeen, tags, visibility` | TOML `devices`: `device_id, name, type, manufacturer, last_seen, first_seen, ip_address, mac_address, os_name, risk_score` (10 columns) | DTU `DeviceRecord`: `device_id, name, ip_address, mac_address, type, manufacturer, os_name, os_version, risk_score, risk_factors, last_seen, first_seen, network_id, site, tags` | **MEDIUM** — TOML+DTU use `device_id`; real poller uses `id`. DTU field names match TOML. | MEDIUM | **N** | poller-coaster SDK fields | DTU column names match TOML. TOML→DTU alignment verified. |
| **Alert response fields** | `title, status, severity, time, lastAlertUpdateTime, policyTitle, type, classification` | TOML `alerts`: `alert_id, name, severity, status, policy_name, device_id, created_at, updated_at` | DTU `AlertRecord`: matches TOML exactly | **MEDIUM** — `policyTitle` → `policy_name` (rename OK). `type`, `classification` not in TOML. `device_id` not a direct field in Armis alerts (device association via AQL). | MEDIUM | **N** | poller-coaster SearchResult alert fields | Non-blocking for demo. Follow-up to add `alert_type` and `classification`. |

### 3.2 AQL Divergence Assessment Under ADR-031

Under ADR-031 D2, the AQL vs direct endpoint divergence is assessed as a **permitted divergence**
with explicit justification:

- The real Armis API DOES expose both direct REST endpoints (`/api/v1/devices`, `/api/v1/alerts`)
  and the AQL search endpoint (`/api/v1/search`). Prism uses the direct endpoints; the poller
  uses AQL. Both are valid call patterns for the real API.
- This is NOT a case where the DTU accepts a cookie name that the real API does not use — the
  real Armis API genuinely accepts `GET /api/v1/devices` with a Bearer token.
- The DTU-EXT-003/004 gap is documented. S-DEMO-ARMIS-AQL-001 (P2) adds AQL support as a
  follow-up when it is needed for live-Armis demos.

Gap ID: Gap-AR-001 (documented permitted divergence per ADR-031 D2).

### 3.3 Remaining Gaps

| Gap ID | Dimension | Severity | DTU change required | Disposition |
|--------|-----------|----------|---------------------|-------------|
| Gap-AR-001 | AQL vs direct endpoint | MEDIUM | **N** (permitted per ADR-031 D2) | S-DEMO-ARMIS-AQL-001 (P2-post-demo acceptable). |
| Gap-AR-002 | `alert_type`, `classification` alert fields absent | LOW | **N** | Follow-up column additions. |
| Gap-AR-003 | `category`, `purdueLevel` device fields absent | LOW | **N** | Follow-up column additions. |

### 3.4 READY for live demo: **YES**

---

## 4. Cyberint Argos (poller-express)

### 4.1 Fidelity Gap Audit Table

| Dimension | Poller Reality | Prism TOML | DTU Clone | Gap? | Severity | DTU change required | Real-API reference | Action |
|-----------|---------------|------------|-----------|------|----------|--------------------|--------------------|--------|
| **Auth mechanism** | `cookieTransport` injects `Cookie: access_token={apiKey}` on every request. NO login step. API key IS the session. | `auth_type = "cookie_roundtrip"` (D-747 LOCKED). Implies login step + session cookie in v1.0 design. | DTU: `POST /login` returns `Set-Cookie: cyberint_session={uuid}`. Routes extract `cyberint_session` cookie. | **CRITICAL** — DTU uses wrong cookie name (`cyberint_session` vs real `access_token`). DTU requires login step; real API requires no login step. This is a DTU fidelity violation under ADR-031 D1-a and D1-b. | CRITICAL | **YES — BLOCKING** | poller-express-broad-sweep §2.1 cookieTransport: `Name: "access_token"` | S-DTU-CYBERINT-AUTH-FIDELITY-001 (P0-pre-demo-BLOCKING). See §4.2. |
| **Alert endpoint** | `POST {baseURL}/alert/api/v1/alerts` (POST method, `/alert/` prefix). | TOML: `GET /api/v1/alerts` (GET method, no `/alert/` prefix). | DTU: `GET /api/v1/alerts` (matches TOML). | **HIGH** (live) / **NO** (DTU demo) — TOML+DTU self-consistent for demo; mismatches real API for live query. | HIGH (live) | **N** (DTU matches TOML; both wrong vs real but self-consistent; live-API correction is separate story) | poller-express-broad-sweep §4.2 endpoint table | Document in demo script. DTU demo path: self-consistent. Live-Cyberint: needs correcting in future story. |
| **Alert pagination** | Page-based: `{"page": 1, "size": 100}`. | TOML: cursor_token pagination (`next_cursor`). | DTU: cursor-based (`?cursor=...` returns `{"data": [...], "next_cursor": "..."}`). | **MEDIUM** (live) / **NO** (DTU) — TOML+DTU cursor-token consistent. Real API uses page-number. | MEDIUM (live) | **N** (DTU cursor matches TOML) | poller-express pagination logic | Document. DTU demo: works. Live-Cyberint: page pagination needed in future story. |
| **Alert field: primary ID** | `RefId` (string, OpenAPI model). | TOML: `alert_id`. | DTU `Alert` struct: `alert_id`. | **MEDIUM** (live) / **NO** (DTU) — TOML+DTU use `alert_id`; real API uses `ref_id`. DTU demo path works. Live: `alert_id` would be NULL. | MEDIUM (live) | **N** (DTU+TOML consistent; `ref_id` mapping needed for live) | poller-express OpenAPI model `Alert.RefId` | Document. S-DTU-CYBERINT-AUTH-FIDELITY-001 scope should also add `ref_id` alias. |
| **Alert field: type** | `Type` in OpenAPI model. | TOML: `type` column. | DTU response: `"type": a.alert_type` (serialized from `alert_type` field). | **MEDIUM** (naming) — DTU emits `type` in JSON response (correct); internal struct field is `alert_type`. TOML column `type` matches DTU JSON key. | MEDIUM | **N** | poller-express Alert.Type | No change needed for demo; DTU JSON key `type` matches TOML column. |
| **Timestamp: `created_at` format** | `CyberintTime` handles RFC3339, no-tz RFC3339, unix microseconds, null/"" → zero. | TOML: `timestamp_formats = ["iso8601", "unix_epoch_seconds"]`. | DTU `Alert.created_at: serde_json::Value` — flexible. | **NO** — TOML handles multi-format timestamps per ADR-028 §D8-A. | — | **N** | poller-express CyberintTime | None |

### 4.2 Gap-CY-001 Detailed Analysis Under ADR-031

**Gap ID:** Gap-CY-001
**Severity (v1.1 under DTU=true-DTU):** CRITICAL — pre-demo BLOCKING

**v1.0 disposition (WRONG):** "Demo against DTU is CONSISTENT and will work. Document for
live-Cyberint scenarios. Build CookieLoginAuthProvider to match DTU's `/login → cyberint_session`
flow, NOT the real Cyberint `access_token` direct injection."

**v1.1 disposition (CORRECT per ADR-031):** The DTU must be CORRECTED to emit `access_token`
cookie. Prism must be implemented to inject `Cookie: access_token={api_key}` without a login
step. A demo that "works" against a wrong DTU proves nothing about Cyberint compatibility.

**Real API behavior (poller-express-broad-sweep §2.1):**
```go
type cookieTransport struct { apiKey string }
func (t *cookieTransport) RoundTrip(req *http.Request) (*http.Response, error) {
    req.AddCookie(&http.Cookie{Name: "access_token", Value: t.apiKey})
    return http.DefaultTransport.RoundTrip(req)
}
```

The `access_token` value equals the API key loaded from `CYBERINT_API_KEY` env var. There is
no login step. There is no session. The API key is the credential; the cookie is the
transport mechanism for that credential on every request.

**Required DTU changes (S-DTU-CYBERINT-AUTH-FIDELITY-001):**

1. Remove `POST /login` route (or convert to no-op stub). No login step in real API.
2. Replace `extract_session_token()` with `extract_access_token()` — extracts `access_token`
   cookie value instead of `cyberint_session` cookie value.
3. `check_auth()` uses `extract_access_token()` and validates against an `access_token`
   allowlist (demo server registers demo token on startup).
4. All existing DTU tests that send `cyberint_session` cookie must be updated to send
   `access_token` cookie.

**Required prism changes (S-DTU-CYBERINT-AUTH-FIDELITY-001 or in-scope of S-DEMO-001):**

1. Auth provider for `CookieRoundtrip` Cyberint: `StaticCookieAuthProvider` — reads API key
   from credential store at acquire-token time; returns API key as "token" value; NO HTTP call
   during acquire_token.
2. `PipelineExecutor::build_request` dispatch: `CookieRoundtrip → Cookie: access_token={token}`.
3. `cyberint.sensor.toml` `auth_type = "cookie_roundtrip"` is preserved (D-747 LOCKED); the
   BEHAVIOR change is in the auth provider and header injection, not the TOML label.

### 4.3 Remaining Gaps Under DTU=True-DTU

| Gap ID | Dimension | Severity | DTU change required | Pre-demo blocking | Disposition |
|--------|-----------|----------|---------------------|-------------------|-------------|
| Gap-CY-001 | Cookie name `access_token` vs `cyberint_session`; login step required by DTU but not real API | CRITICAL | **YES** | **YES** | S-DTU-CYBERINT-AUTH-FIDELITY-001 (P0-pre-demo-BLOCKING). |
| Gap-CY-002 | Real API path `POST /alert/api/v1/alerts` vs TOML/DTU `GET /api/v1/alerts` | HIGH (live) | **N** (TOML+DTU self-consistent; live-API correction separate) | **NO** (DTU demo unaffected) | Document in demo runbook. Future live-API story needed. |
| Gap-CY-003 | `alert_id` vs real `ref_id` field name | MEDIUM (live) | **N** (TOML+DTU consistent; live-API fix in future story) | **NO** (DTU demo unaffected) | Document in demo runbook. Future story to add `ref_id` alias. |

### 4.4 READY for live demo: **NO** (until S-DTU-CYBERINT-AUTH-FIDELITY-001 merges)

---

## 5. Cross-Sensor Summary: Per-Sensor Gap List (v1.1)

### CrowdStrike

| Gap ID | Dimension | Severity | DTU change required | Pre-demo blocking |
|--------|-----------|----------|---------------------|-------------------|
| Gap-CS-002 | Poller polls alerts API; prism exposes detections API | MEDIUM | **N** | **NO** — document in demo script |
| Gap-CS-003 | Base URL hardcoded for us-1 only | MEDIUM | **N** | **NO** |

### Claroty

| Gap ID | Dimension | Severity | DTU change required | Pre-demo blocking |
|--------|-----------|----------|---------------------|-------------------|
| Gap-CL-001 | Trailing slash on endpoint paths | MEDIUM | **N** | **NO** |
| Gap-CL-004 | Offset pagination sent as URL params; Claroty API expects body params | HIGH | **N** (pipeline wrong, not DTU) | **YES** — limits demo to first-page results; `claroty_alerts`/`claroty_devices` page-1 only |
| Gap-CL-006 | DTU has no `/api/v1/audit_log/get` route | HIGH | **YES** | **CONDITIONAL** — only blocking if demo script includes `claroty_audit_logs` queries |

### Armis

| Gap ID | Dimension | Severity | DTU change required | Pre-demo blocking |
|--------|-----------|----------|---------------------|-------------------|
| Gap-AR-001 | AQL vs direct endpoint (permitted divergence per ADR-031 D2) | MEDIUM | **N** | **NO** |
| Gap-AR-002 | `alert_type`, `classification` alert fields absent | LOW | **N** | **NO** |
| Gap-AR-003 | `category`, `purdueLevel` device fields absent | LOW | **N** | **NO** |

### Cyberint

| Gap ID | Dimension | Severity | DTU change required | Pre-demo blocking |
|--------|-----------|----------|---------------------|-------------------|
| Gap-CY-001 | Cookie name `access_token` vs `cyberint_session`; no-login vs login-step | CRITICAL | **YES** | **YES** — BLOCKING |
| Gap-CY-002 | Real API path/method mismatch vs TOML/DTU (live-API scenario only) | HIGH (live) | **N** | **NO** (DTU demo unaffected) |
| Gap-CY-003 | `alert_id` vs real `ref_id` field name (live-API scenario only) | MEDIUM (live) | **N** | **NO** (DTU demo unaffected) |

---

## 6. Demo-Readiness Summary (v1.1)

| Sensor | READY for DTU Demo? | Gating Items | Notes |
|--------|---------------------|--------------|-------|
| **CrowdStrike** | **YES** | None — Gap-CS-001 fix applied at develop@72baf413. | Queries `crowdstrike_detections` → `detection_id` column correct. |
| **Claroty** | **PARTIAL** | Gap-CL-004 (pagination): first-page only for `claroty_alerts` and `claroty_devices`. Gap-CL-006 (audit_log DTU route): `claroty_audit_logs` returns 404 until S-DEMO-CLAROTY-AUDIT-DTU-001 ships. | Demo can show alerts (page 1) + devices (page 1). Demo script must avoid `claroty_audit_logs` until DTU route exists. |
| **Armis** | **YES** | None blocking. Gap-AR-001 documented permitted divergence. | Queries `armis_devices` and `armis_alerts`. |
| **Cyberint** | **NO** | Gap-CY-001: DTU emits wrong cookie name (`cyberint_session`), requires login step, violates ADR-031 D1. S-DTU-CYBERINT-AUTH-FIDELITY-001 MUST ship before demo. | After fix: queries `cyberint_alerts` with `access_token` cookie. `alert_id` column works for DTU path. |

---

## 7. Story Reclassification Table (v1.1 Under DTU=True-DTU)

| Old stub ID | Old priority | Old name | Revised ID | Revised name | Revised priority | DTU work in scope | Pre-demo blocking |
|------------|--------------|----------|-----------|--------------|-----------------|-------------------|-------------------|
| S-DEMO-CYBERINT-LIVE-AUTH-001 | P2-post-demo | StaticCookieAuthProvider for real Cyberint API | **S-DTU-CYBERINT-AUTH-FIDELITY-001** | prism-dtu-cyberint + prism-spec-engine: Cyberint auth fidelity — `access_token` cookie, static injection, no login step | **P0-pre-demo-BLOCKING** | **YES** — DTU `post_login` removed; `extract_session_token` → `extract_access_token`; all DTU tests updated | **YES** |
| S-DEMO-CLAROTY-PAGINATION-001 | P1 | OffsetLimit POST-body pagination | S-DEMO-CLAROTY-PAGINATION-001 (unchanged) | prism-spec-engine: OffsetLimit pagination sends params in POST body for POST-method steps | P1 | **NO** — pipeline fix only; DTU already accepts body params | **YES** — limits Claroty to page-1 until fixed |
| S-DEMO-CLAROTY-AUDIT-DTU-001 | P2 | Add audit_log route to prism-dtu-claroty | S-DEMO-CLAROTY-AUDIT-DTU-001 (unchanged) | prism-dtu-claroty: Add `/api/v1/audit_log/get` route (real-API path per poller-bear) | P2 | **YES** — DTU route addition | **CONDITIONAL** — blocking only if demo queries `claroty_audit_logs` |
| S-DEMO-CLAROTY-TRAILING-SLASH-001 | P2 | Trailing slash on Claroty paths | S-DEMO-CLAROTY-TRAILING-SLASH-001 (unchanged) | prism-sensors, prism-spec-engine: trailing slash conformance | P2 | **YES** (verify DTU handles it; axum likely normalizes) | **NO** |
| S-DEMO-ARMIS-AQL-001 | P2 | AQL search endpoint support | S-DEMO-ARMIS-AQL-001 (unchanged) | prism-dtu-armis + prism-spec-engine: AQL search endpoint (ADR-031 D2 documented divergence) | P2 | **YES** — new DTU route + TOML table + pipeline support | **NO** — direct endpoints work for DTU demo |
| S-DEMO-CROWDSTRIKE-MULTIREGION-001 | P3 | Multi-region base_url | S-DEMO-CROWDSTRIKE-MULTIREGION-001 (unchanged) | prism-sensors: Multi-region base_url for CrowdStrike | P3 | **YES** — verify DTU exposes equivalent multi-region config | **NO** |

### Recommended dispatch order for pre-demo stories:

1. **S-DTU-CYBERINT-AUTH-FIDELITY-001** (P0) — must ship before or alongside S-DEMO-001.
   Dispatch sequence: worktree on `feature/S-DTU-CYBERINT-AUTH-FIDELITY-001`. Deliverables:
   DTU `prism-dtu-cyberint` code changes + `prism-spec-engine` `StaticCookieAuthProvider` +
   `build_request` dispatch correction + updated parity tests.

2. **S-DEMO-001** (P0) — blocked on S-DTU-CYBERINT-AUTH-FIDELITY-001 for the Cyberint auth
   path. With the fidelity story merged, S-DEMO-001 implements `StaticCookieAuthProvider`
   (not `CookieLoginAuthProvider`) and injects `Cookie: access_token={token}`.

3. **S-DEMO-CLAROTY-PAGINATION-001** (P1) — enables multi-page Claroty results in the demo.
   Not strictly blocking if the demo stays under 100-row thresholds.

4. **S-DEMO-CLAROTY-AUDIT-DTU-001** (P2) — only needed if demo script includes
   `claroty_audit_logs` queries.

### Option A vs Option B: split vs coordinated DTU stories

**Recommendation: Option A (split), with caveat.**

S-DTU-CYBERINT-AUTH-FIDELITY-001 is the only P0 DTU story. The Claroty DTU stories
(S-DEMO-CLAROTY-AUDIT-DTU-001) and the Armis AQL story (S-DEMO-ARMIS-AQL-001) are P2 and
can be dispatched independently in parallel via separate worktrees.

Rationale:
- Parallelization benefit is real — these are independent codebases (different `prism-dtu-*`
  crates) with no cross-story dependencies.
- Fragmented PR review concern is mitigated by the existing per-story adversary cascade
  protocol, which catches regressions per story.
- A single coordinated DTU story (Option B) would create an unusually large PR touching
  4 different crates plus test fixtures, increasing adversary cascade complexity.

**Dispatch sequence recommendation:**
1. S-DTU-CYBERINT-AUTH-FIDELITY-001 (parallel with S-DEMO-001 preparation, must land first)
2. S-DEMO-001 (after S-DTU-CYBERINT-AUTH-FIDELITY-001)
3. S-DEMO-CLAROTY-PAGINATION-001 (parallel with S-DEMO-002)
4. S-DEMO-CLAROTY-AUDIT-DTU-001 + S-DEMO-ARMIS-AQL-001 (parallel, post-demo if not in demo script)

---

## 8. S-DEMO-001 / S-DEMO-002 AC Amendments Required (v1.1)

### S-DEMO-001 Amendments (v1.2 → v1.3)

**AC-003:** Change from `cyberint_session` to `access_token`. Current AC-003 says:
"The adapter uses its held `CookieLoginAuthProvider`, which (a) calls `POST {base_url}/login`,
(b) parses the `Set-Cookie` response header, (c) extracts the value of the cookie named
`cyberint_session`..."

Revised AC-003 must say: "The adapter uses its held `StaticCookieAuthProvider`, which reads
the Cyberint API key from the credential store at acquire-token time and returns it as the
token. The pipeline injects it as `Cookie: access_token={api_key}`. No HTTP call is made
during `acquire_token`. No login step."

**AC-009:** Change from `cyberint_session` cookie parse to `access_token` static injection.
Current AC-009 says: "`CookieLoginAuthProvider` parses the cookie name `cyberint_session` from
the `Set-Cookie` header."

Revised AC-009 must say: "`StaticCookieAuthProvider` reads the API key from the credential
store. The HTTP request to `GET /api/v1/alerts` carries `Cookie: access_token={api_key}`,
NOT `Authorization: Bearer`. No login step occurs. The `POST /login` route in the DTU is
NOT called."

**§Inputs:** Add `crates/prism-dtu-cyberint/src/routes/auth.rs` and
`crates/prism-dtu-cyberint/src/routes/alerts.rs` (DTU will be modified by
S-DTU-CYBERINT-AUTH-FIDELITY-001 before S-DEMO-001 is implemented).

**§depends_on:** Add `S-DTU-CYBERINT-AUTH-FIDELITY-001` as an explicit dependency.

**§Origin:** Update the cookie name reconciliation section — remove the statement that "for
the demo, the DTU model governs." Replace with "per ADR-031, the real API governs; the DTU
is corrected to match the real API."

### S-DEMO-002 AC Amendments

No additional amendments required beyond those in v1.2 for Cyberint — after the fidelity
fix, the demo queries `cyberint_alerts` with the correct auth path and the `alert_id` column
is populated (DTU path).

---

## 9. Auth Mechanism Summary (All 4 Sensors, v1.1)

| Sensor | Production (Poller) Auth | Prism TOML auth_type | DTU Auth Enforcement | DTU change required | Demo Path (post-fix) |
|--------|--------------------------|---------------------|---------------------|--------------------|-----------------------|
| CrowdStrike | OAuth2 client-credentials via gofalcon SDK → `Authorization: Bearer` | `oauth2_client_credentials` + `auth_plugin = "crowdstrike-oauth2"` | `POST /oauth2/token` returns Bearer token; downstream routes require `Authorization: Bearer` | **N** | YES — correct |
| Claroty xDome | `Authorization: Bearer <CLAROTY_API_KEY>` static | `bearer_static` | `check_bearer_auth()` enforces `Authorization: Bearer {non-empty}` | **N** | YES — correct |
| Armis Centrix | `Authorization: Bearer <ARMIS_API_KEY>` via SDK | `bearer_static` | `Authorization: Bearer {non-empty}` required → 403 on missing | **N** | YES — correct |
| Cyberint Argos | `Cookie: access_token={API_KEY}` (no login step, static cookie) | `cookie_roundtrip` | **PRE-FIX:** `POST /login` → `cyberint_session` cookie (WRONG). **POST-FIX:** static `access_token` cookie validation (no login step, CORRECT) | **YES** — S-DTU-CYBERINT-AUTH-FIDELITY-001 | YES — after fidelity story merges |
