---
document_type: architect-audit
title: "Harness-Clone DTU Fidelity Audit — All 4 Sensors (2026-05-30)"
author: architect
date: "2026-05-30"
status: FINAL
version: "1.0"
trigger: F-LP1-OBS-001 [process-gap] — ADR-031 §D1 scope under-specified; missed crates/prism-dtu-harness/src/clones/ path
anchor_story: S-DTU-CYBERINT-AUTH-FIDELITY-001 (Pass 1 cascade)
governing_principle: ADR-031 DTU=true-DTU (2026-05-29)
sources_read:
  - crates/prism-dtu-harness/src/clones/cyberint.rs
  - crates/prism-dtu-harness/src/clones/crowdstrike.rs
  - crates/prism-dtu-harness/src/clones/claroty.rs
  - crates/prism-dtu-harness/src/clones/armis.rs
  - crates/prism-dtu-cyberint/src/routes/auth.rs
  - crates/prism-dtu-cyberint/src/routes/alerts.rs
  - crates/prism-dtu-cyberint/src/clone.rs
  - crates/prism-sensors/specs/cyberint.sensor.toml
  - crates/prism-sensors/specs/crowdstrike.sensor.toml
  - .factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md
  - .factory/proposals/POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md (v1.1)
  - crates/prism-dtu-harness/tests/logical_isolation_test.rs
related_docs:
  - POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md (v1.1 — canonical DTU clone audit; see §9)
  - ADR-031-dtu-equals-true-dtu-fidelity-principle.md (v1.1 — amended by this audit)
---

# Harness-Clone DTU Fidelity Audit — All 4 Sensors (v1.0)

## Purpose

Audit all four sensor harness clones at `crates/prism-dtu-harness/src/clones/{sensor}.rs`
against ADR-031 §D1 fidelity requirements. This audit was triggered by F-LP1-CRIT-001 from
the S-DTU-CYBERINT-AUTH-FIDELITY-001 Pass 1 LOCAL adversary cascade, which found that the
harness Cyberint clone still uses the legacy `cyberint_session` cookie + `POST /login`
model, a violation the prior POLLER-DTU-FIDELITY-AUDIT (v1.1) missed because it only
inspected `crates/prism-dtu-{sensor}/src/` paths.

**Scope of this audit:** `crates/prism-dtu-harness/src/clones/` exclusively.
This supplements, not replaces, POLLER-DTU-FIDELITY-AUDIT-2026-05-29 v1.1.

## Severity Classification

- **CRITICAL** — runtime defect that makes tests or the live demo fail/silently return wrong data
- **HIGH** — fidelity gap that causes test divergence from real-API behavior
- **MEDIUM** — fidelity gap that causes coverage gap (not a runtime failure in current tests)
- **LOW** — cosmetic or completeness gap not affecting current test correctness

---

## 1. Cyberint Argos — Harness Clone

**File:** `crates/prism-dtu-harness/src/clones/cyberint.rs`

### 1.1 Fidelity Gap Audit Table (vs canonical DTU + real API)

| Dimension | Real API (poller-express) | Canonical DTU (`prism-dtu-cyberint/src/routes/auth.rs`) | Harness Clone | Harness state | Severity |
|-----------|--------------------------|----------------------------------------------------------|---------------|---------------|----------|
| **Auth flow** | No login step. Static `Cookie: access_token={api_key}` on every request. | `POST /login` → `Set-Cookie: cyberint_session={uuid}` (also wrong — both canonical and harness must change) | `POST /login` → `Set-Cookie: cyberint_session={token}` (line 770: `cyberint_session={token}`) | **VIOLATES ADR-031 D1-b** — requires login step; real API does not | **CRITICAL** |
| **Cookie name** | `access_token` (poller-express `cookieTransport.Name: "access_token"`) | `cyberint_session` (`routes/auth.rs:39`) — also wrong | `cyberint_session` (`extract_session_token` function, line 710-719) | **VIOLATES ADR-031 D1-a** | **CRITICAL** |
| **Auth validator** | Validates `access_token` cookie value against API key allowlist | Validates `cyberint_session` cookie against session store | `extract_session_token()` extracts `cyberint_session` (line 714); `check_auth()` calls `extract_session_token()` (line 741) | **WRONG** — must extract `access_token` | **CRITICAL** |
| **Session store** | No session state; API key IS the static credential | `session_store: HashSet<String>` keyed on UUID tokens from login | `session_store: Mutex<HashSet<String>>` (line 155) — populated by `register_session` called from `post_login` | **WRONG** — should be `access_token` allowlist, not login-issued UUID store | **CRITICAL** |
| **`POST /login` route** | Does NOT exist in real API | Exists (wrong) at `/login` | Exists (wrong) at `build_cyberint_router` line 1288 `.route("/login", post(post_login))` | **WRONG** — must not require login step | **HIGH** |
| **Alert endpoint** | `POST {baseURL}/alert/api/v1/alerts` | `GET /api/v1/alerts` (also wrong vs real API, but TOML-consistent) | `GET /api/v1/alerts` + `POST /api/v1/alerts` alias | **MEDIUM** (TOML-consistent for demo path; real-API mismatch deferred) | MEDIUM |
| **Alert field names** | `RefId` → alert primary key | `alert_id` (TOML-consistent) | `alert_id` in `HarnessAlert` struct (line 92), serialized as `"alert_id"` in `to_json()` (line 102) | **MEDIUM** (TOML-consistent; real `ref_id` mapping deferred) | MEDIUM |
| **Response shape** | `{"data": [...], "next_cursor": "..."}` (cursor-based) | Cursor-based | `{"data": data, "next_cursor": next_cursor}` (line 872) | **CORRECT** — matches TOML/canonical DTU | — |

### 1.2 Summary

The harness Cyberint clone is a COMPLETE DUPLICATE of the canonical DTU's wrong auth model.
Both the canonical DTU (`prism-dtu-cyberint`) and the harness clone (`prism-dtu-harness/src/clones/cyberint.rs`)
must be corrected to implement `access_token` static-cookie auth with no login step.

**CRITICAL gap count: 4** (auth flow, cookie name, auth validator, session store)
**HIGH gap count: 1** (POST /login route presence)

### 1.3 Harness test impact

`tests/logical_isolation_test.rs` uses `GET /api/v1/events` (unauthenticated legacy alias)
for Cyberint org-isolation tests — NOT `GET /api/v1/alerts` with cookie auth. This means
the isolation tests do NOT currently exercise the `cyberint_session` auth path. The CRITICAL
fidelity violations are real but are dormant in the CURRENT isolation test suite.

**Risk:** Any test that exercises `GET /api/v1/alerts` via the harness will pass against
the wrong `cyberint_session` model, providing false confidence that the pipeline works
correctly for Cyberint.

### 1.4 READY for demo (harness path): **NO** (same blocking status as canonical DTU)

---

## 2. CrowdStrike Falcon — Harness Clone

**File:** `crates/prism-dtu-harness/src/clones/crowdstrike.rs`

### 2.1 Fidelity Gap Audit Table

| Dimension | Real API (poller-cobra) | Canonical DTU (`prism-dtu-crowdstrike/`) | Harness Clone | Harness state | Severity |
|-----------|------------------------|------------------------------------------|---------------|---------------|----------|
| **Auth flow** | OAuth2 client credentials via `POST /oauth2/token`; downstream routes require `Authorization: Bearer` | `POST /oauth2/token` returns `{"access_token": "dtu-fake-cs-token", ...}` | `POST /oauth2/token` returns `{"access_token": "dtu-fake-cs-token", "token_type": "bearer", "expires_in": 3600}` (line 451-458) | **CORRECT** — matches canonical and real API | — |
| **Bearer auth** | `Authorization: Bearer {token}` | `check_bearer_auth()` validates non-empty Bearer | `check_bearer_auth()` validates non-empty Bearer (lines 282-302); returns 401 on empty | **CORRECT** | — |
| **Detection list endpoint** | `GET /detects/queries/detects/v1` | Matches | `GET /detects/queries/detects/v1` (line 1131) | **CORRECT** | — |
| **Detection summaries endpoint** | `POST /detects/entities/summaries/GET/v1` | Matches | `POST /detects/entities/summaries/GET/v1` (line 1132-1134) | **CORRECT** | — |
| **Host list endpoint** | `GET /devices/queries/devices/v1` | Matches | `GET /devices/queries/devices/v1` (line 1137) | **CORRECT** | — |
| **Host details endpoint** | `GET /devices/entities/devices/v2` | Matches | `GET /devices/entities/devices/v2` (line 1138) | **CORRECT** | — |
| **Device actions endpoint** | `POST /devices/entities/devices-actions/v2` | Matches | `POST /devices/entities/devices-actions/v2` (line 1140) | **CORRECT** | — |
| **Detection primary key** | `composite_id` → prism maps to `detection_id` (Gap-CS-001 FIXED develop@72baf413) | `detection_id` | `detection_detail()` returns `"detection_id": detection_id` (line 252) | **CORRECT** — post-fix state | — |
| **Host primary key** | `device_id` in host records | `device_id` | `host_detail()` returns `"device_id": device_id` (line 263) | **CORRECT** | — |
| **Response wrapper** | `{"resources": [...], "meta": {...}}` | `{"resources": [...], "meta": {...}}` | `{"resources": page, "meta": {...}, "next_token": ...}` (line 524-535) | **CORRECT** | — |
| **Rate limit flow** | 429 with `Retry-After` | Supported | `FailureMode::RateLimit` sets `retry-after` header (lines 326-337) | **CORRECT** | — |
| **OAuth AuthReject** | 401 on token endpoint | 401 on `/oauth2/token` | `if state.is_auth_reject()` returns 401 in `oauth_token` (line 441-448) | **CORRECT** | — |

### 2.2 Remaining Gaps

None from ADR-031 D1-a through D1-e. The harness CrowdStrike clone faithfully mirrors
the canonical DTU which was confirmed CORRECT in POLLER-DTU-FIDELITY-AUDIT v1.1.

One structural divergence exists but is intentional per architecture:
- The harness clone uses `CrowdStrikeHarnessState` (standalone) rather than importing from
  `prism-dtu-crowdstrike`. The doc-comment on `claroty.rs` states: "intentionally self-contained
  — does NOT import from `prism-dtu-claroty` to avoid circular dev-dependency chains."
  This is NOT a fidelity violation — it is a correct architectural isolation pattern for
  the harness crate.

### 2.3 READY for demo (harness path): **YES**

---

## 3. Claroty xDome — Harness Clone

**File:** `crates/prism-dtu-harness/src/clones/claroty.rs`

### 3.1 Fidelity Gap Audit Table

| Dimension | Real API (poller-bear) | Canonical DTU (`prism-dtu-claroty/`) | Harness Clone | Harness state | Severity |
|-----------|------------------------|--------------------------------------|---------------|---------------|----------|
| **Auth flow** | `Authorization: Bearer <API_KEY>` static | `check_bearer_auth()` requires non-empty Bearer | `check_bearer_auth()` requires Bearer header starting with `Bearer ` and non-empty value (lines 238-253) | **CORRECT** | — |
| **Device list endpoint** | `POST /api/v1/devices` | `POST /api/v1/devices` | `POST /api/v1/devices` (line 904) | **CORRECT** | — |
| **Alert list endpoint** | `POST /api/v1/alerts` | `POST /api/v1/alerts` | `POST /api/v1/alerts` (line 906) | **CORRECT** | — |
| **Vulnerability endpoint** | `POST /api/v1/vulnerabilities` (not in poller-bear directly, but part of Claroty xDome API) | `POST /api/v1/vulnerabilities` | `POST /api/v1/vulnerabilities` (lines 908-912) | **CORRECT** (extended endpoint) | — |
| **Audit log endpoint** | `POST /api/v1/audit_log/get` (poller-bear §4.3) | NOT present in canonical DTU — Gap-CL-006 documented in POLLER-DTU-FIDELITY-AUDIT v1.1 | NOT present in harness clone — no `/api/v1/audit_log/get` route visible in `router()` or `network_router()` | **HIGH** — DTU harness also lacks the audit log route (same as canonical DTU) | HIGH |
| **Device fixture** | `uid`, `asset_id`, `device_type`, `risk_score` etc. | `DEVICES_FIXTURE` loaded from `prism-dtu-claroty/fixtures/devices.json` | `DEVICES_FIXTURE` = `include_str!("../../../prism-dtu-claroty/fixtures/devices.json")` (line 58) | **CORRECT** — shares canonical fixture | — |
| **Alert fixture** | `alert_type_name`, `detected_time`, `updated_time` (FIXED develop@72baf413) | `ALERTS_FIXTURE` from `prism-dtu-claroty/fixtures/alerts.json` | `ALERTS_FIXTURE` = `include_str!("../../../prism-dtu-claroty/fixtures/alerts.json")` (line 61) | **CORRECT** — shares canonical fixture | — |
| **Tag endpoints** | `POST /api/v1/devices/:id/tags/` + `DELETE /api/v1/devices/:id/tags/:key` | Present | Present in `router()` lines 913-914 | **CORRECT** | — |
| **Pagination** | `offset`/`limit` in POST body (Gap-CL-004 — pipeline sends URL params; both DTU and harness accept body params) | Accepts body params | `list_devices` accepts `page`, `page_size`, `offset`, `limit` from POST body (lines 354-360) | **CORRECT** — harness DTU is correct; pipeline has the bug (Gap-CL-004) | — |

### 3.2 Harness-Clone-Specific Gap

| Gap ID | Dimension | Harness vs Canonical | Severity | Disposition |
|--------|-----------|---------------------|----------|-------------|
| Gap-HARNESS-CL-001 | Missing `/api/v1/audit_log/get` route | SAME gap as canonical DTU (Gap-CL-006) — both lack the route | HIGH | Same fix as canonical: add `/api/v1/audit_log/get` POST route in BOTH `crates/prism-dtu-claroty` AND `crates/prism-dtu-harness/src/clones/claroty.rs`. Pre-demo conditional on whether demo script includes `claroty_audit_logs` queries. |

**Key architectural observation:** The Claroty harness clone uses `include_str!` to embed
canonical fixture files at compile time (`lines 58-73`). This means the harness clone's
fixture data stays in sync with the canonical DTU's fixture changes automatically. This is
the correct pattern; Claroty and Armis harness clones both follow it.

### 3.3 READY for demo (harness path): **PARTIAL** (same status as canonical DTU)

- `claroty_alerts`, `claroty_devices`: READY (page 1)
- `claroty_audit_logs`: NOT READY (Gap-HARNESS-CL-001 = same as Gap-CL-006)

---

## 4. Armis Centrix — Harness Clone

**File:** `crates/prism-dtu-harness/src/clones/armis.rs`

### 4.1 Fidelity Gap Audit Table

| Dimension | Real API (poller-coaster) | Canonical DTU (`prism-dtu-armis/`) | Harness Clone | Harness state | Severity |
|-----------|--------------------------|-------------------------------------|---------------|---------------|----------|
| **Auth flow** | `Authorization: Bearer <ARMIS_API_KEY>` via Armis SDK. No login step. | `check_bearer_auth()` requires Bearer, returns 403 on absent (AC-5 Armis spec) | `check_bearer_auth()` returns 403 on absent Bearer, 401 on wrong token (lines 236-267) | **CORRECT** — 403 for missing auth is per Armis spec | — |
| **Device list endpoints** | Real API uses `GET /api/v1/search?aql=...` (AQL). Permitted divergence per ADR-031 D2 (Gap-AR-001) — direct endpoints also valid. | `GET /api/v1/devices` + `POST /api/v1/devices` | `GET /api/v1/devices` + `POST /api/v1/devices` (lines 950-951) | **CORRECT** — same permitted divergence as canonical DTU | — |
| **Alert list endpoint** | Via AQL or `GET /api/v1/alerts` | `GET /api/v1/alerts` | `GET /api/v1/alerts` (line 957) | **CORRECT** | — |
| **Device fixture** | `id, name, ipAddress, manufacturer, riskLevel, type` (SDK fields) | `DEVICES_FIXTURE` from `prism-dtu-armis/fixtures/devices.json` | `DEVICES_FIXTURE` = `include_str!("../../../prism-dtu-armis/fixtures/devices.json")` (line 79) | **CORRECT** — shares canonical fixture | — |
| **Alert fixture** | `title, status, severity, time, policyTitle` | `ALERTS_FIXTURE` from `prism-dtu-armis/fixtures/alerts.json` | `ALERTS_FIXTURE` = `include_str!("../../../prism-dtu-armis/fixtures/alerts.json")` (line 82) | **CORRECT** — shares canonical fixture | — |
| **Device primary key** | `id` in poller; TOML uses `device_id` (mapping acceptable) | `device_id` | `device_id` in fixture (line 438 `device["device_id"]`) | **CORRECT** (TOML-consistent) | — |
| **Auth error code** | 403 on missing/malformed Bearer | 403 on missing Bearer | harness `check_bearer_auth()` returns 403 on missing Bearer (line 244-251), 401 on wrong token (lines 255-264) | **CORRECT** — 403 vs 401 distinction is per AC-5 and cross-org isolation spec | — |
| **AQL capture log** | N/A | Present | `GET /dtu/aql-log` route (line 963) | **CORRECT** | — |
| **Activity endpoint** | Per-device activity | Present | `GET /api/v1/devices/:device_id/activity` (line 952-954) | **CORRECT** | — |
| **Risk endpoint** | Per-device risk | Present | `GET /api/v1/devices/:device_id/risk` (line 955) | **CORRECT** | — |
| **Tag endpoints** | POST tags / DELETE tags | Present | `POST /api/v1/devices/:device_id/tags/` + `DELETE /api/v1/devices/:device_id/tags/:tag_key` (lines 958-962) | **CORRECT** | — |

### 4.2 Remaining Gaps

None from ADR-031 D1-a through D1-e. The Armis harness clone correctly implements:
- Bearer-static auth with correct 403/401 semantics (matching Armis AC-5 spec)
- Correct endpoint structure (permitted divergence for AQL vs direct-endpoint documented)
- Correct fixture sharing via `include_str!`

### 4.3 READY for demo (harness path): **YES**

---

## 5. Cross-Sensor Harness Fidelity Summary

| Sensor | ADR-031 D1 violations | Severity | Harness change required | Demo-ready (harness) |
|--------|-----------------------|----------|------------------------|---------------------|
| **Cyberint** | 4 CRITICAL (cookie name, auth flow, auth validator, session store model) + 1 HIGH (POST /login route) | CRITICAL | **YES** — must change with canonical DTU in same story | **NO** (same blocking gate as canonical) |
| **CrowdStrike** | None | — | **NO** | **YES** |
| **Claroty** | 1 HIGH — missing `/api/v1/audit_log/get` route (same gap as canonical DTU) | HIGH | **YES** — add audit_log route in same story as canonical DTU (S-DEMO-CLAROTY-AUDIT-DTU-001) | **PARTIAL** |
| **Armis** | None | — | **NO** | **YES** |

---

## 6. Pattern Decision — Recommended Remediation Architecture

### Options evaluated

**Pattern A (delete + delegate):** Delete `prism-dtu-harness/src/clones/{sensor}.rs`; have
`HarnessBuilder::add_dtu(DtuType::Sensor, ...)` construct the canonical `prism_dtu_{sensor}::SensorClone`
directly.

**Pattern B (in-place rewrite):** Rewrite each harness clone to mirror the canonical clone's
behavior. Two DTU implementations per sensor maintained manually.

**Pattern C (delegate via trait):** Refactor harness clones to forward to canonical via a
`DtuClone` trait. Preserves harness abstraction; eliminates logic duplication.

**Pattern D (hybrid):** Some sensors deleted+delegated, some kept.

### Decision: Pattern B (uniform in-place rewrite) for Cyberint; Pattern C (shared fixtures) preserved for Claroty/Armis; Pattern A not recommended

**Rationale:**

1. **Claroty and Armis harness clones already use a de-facto Pattern C variant.** They
   embed the canonical fixture data via `include_str!` (`prism-dtu-claroty/fixtures/*.json`,
   `prism-dtu-armis/fixtures/*.json`). The fixture data is single-source. The route logic
   is duplicated but is self-contained and small. Full Pattern A (delete + delegate) for
   these two would require making `prism-dtu-claroty` and `prism-dtu-armis` non-dev-dependencies
   of `prism-dtu-harness` — the Claroty harness clone's doc-comment explicitly states
   "does NOT import from `prism-dtu-claroty` to avoid circular dev-dependency chains." This
   is a deliberate architecture choice that Pattern A would undo.

2. **CrowdStrike harness clone is already correct.** Pattern A would delete working code for
   no fidelity benefit.

3. **Cyberint harness clone requires rewrite (Pattern B) in-place.** The auth model must
   change from `cyberint_session`+`POST /login` to `access_token` static injection. This is
   the same correction required in the canonical DTU. The canonical DTU crate (`prism-dtu-cyberint`)
   and the harness clone MUST be fixed in the SAME story to avoid a state where one is
   corrected and the other is not.

4. **Pattern A risks are too high.** Import from canonical DTU in harness creates a dev-dependency
   that must be managed for circular-dep prevention across the workspace. The existing
   fixture-sharing pattern (Pattern C partial) is the correct level of sharing — share data,
   not routing logic.

**Uniform remediation pattern (adopted):**
- **Cyberint:** In-place rewrite of `clones/cyberint.rs` to use `access_token` cookie,
  no `POST /login` requirement, `access_token` allowlist in lieu of session store.
  Simultaneous with canonical DTU fix in the same story (`S-DTU-CYBERINT-AUTH-FIDELITY-001`).
- **Claroty:** Add `/api/v1/audit_log/get` route to `clones/claroty.rs` simultaneously
  with the canonical DTU fix in `S-DEMO-CLAROTY-AUDIT-DTU-001`.
- **CrowdStrike, Armis:** No harness changes required.
- **Fixture sharing:** Continue using `include_str!` for Claroty and Armis. Do NOT introduce
  harness→canonical crate runtime dependencies.

---

## 7. Scope Decision

### Decision: Scope-1 (in-current-story) for Cyberint harness; new sub-scope for Claroty

**Rationale:**

**Cyberint (Scope-1):** The harness clone fix is MANDATORY before any test that exercises
the `GET /api/v1/alerts` path via the harness will produce correct behavior. Since
`S-DTU-CYBERINT-AUTH-FIDELITY-001` already targets `prism-dtu-cyberint` and the auth
provider in `prism-spec-engine`, adding the harness clone fix to the same story is the
correct scope expansion. The harness clone and the canonical DTU share the same wrong auth
model — fixing one without the other leaves two conflicting implementations with the wrong
model still present. Adding the harness fix to the story also avoids a separate PR touching
the same `prism-dtu-harness` crate.

**Scope expansion for S-DTU-CYBERINT-AUTH-FIDELITY-001:**
- `crates/prism-dtu-harness/src/clones/cyberint.rs`: rewrite auth model
  (same `access_token` changes as `prism-dtu-cyberint`)
- ALL harness tests that use the Cyberint clone must be verified to not call
  `POST /login` or reference `cyberint_session` after the fix

**Claroty audit_log harness gap (scope stays with S-DEMO-CLAROTY-AUDIT-DTU-001):**
The `crates/prism-dtu-harness/src/clones/claroty.rs` audit_log route gap is identical
to Gap-CL-006 in the canonical DTU. When `S-DEMO-CLAROTY-AUDIT-DTU-001` ships, the
implementer MUST add the `/api/v1/audit_log/get` route to BOTH `prism-dtu-claroty` AND
`prism-dtu-harness/src/clones/claroty.rs` in the same PR. This is codified here as a
two-target implementation requirement, not a separate story.

**CrowdStrike and Armis (no scope change):** No harness changes required.

---

## 8. Scope Expansion Documentation for S-DTU-CYBERINT-AUTH-FIDELITY-001

The following items are added to the current story scope (to be propagated by story-writer):

### Additional deliverables for S-DTU-CYBERINT-AUTH-FIDELITY-001

1. **`crates/prism-dtu-harness/src/clones/cyberint.rs` changes:**
   - Rename `extract_session_token()` → `extract_access_token()` that strips `access_token=`
     prefix instead of `cyberint_session=`
   - Remove `post_login` route handler and unregister `POST /login` from both
     `build_cyberint_router` and `build_cyberint_network_router`
   - Rename `CyberintCloneState.session_store` → `access_token_store` (same `HashSet<String>`
     type; semantics change from UUID-per-login to static-key allowlist)
   - Add `register_access_token(token: String)` called from `start_cyberint_clone` startup
     (the demo token is the `admin_token` value or a fixed test value; no HTTP call required)
   - Update `check_auth()` to call `extract_access_token()` and validate against
     `access_token_store`
   - Update module-level doc-comment (lines 14, 38-41): remove `POST /login` entry and
     `cyberint_session` cookie description; add `access_token` static cookie description
   - `CyberintCloneState.reset()` must clear `access_token_store` AND re-register the
     configured demo token (so the clone is usable again after reset without a login call)

2. **`crates/prism-dtu-harness/tests/` verification:**
   - Run `rg 'cyberint_session\|/login' crates/prism-dtu-harness/` after the fix; zero hits
     required
   - The logical_isolation_test.rs `test_BC_3_6_001_timeout_does_not_block_org_b` test uses
     `GET /api/v1/events` (unauthenticated), not the alert route — it is unaffected by the
     auth change

---

## 9. Cross-Reference: POLLER-DTU-FIDELITY-AUDIT-2026-05-29 v1.1 Gap

This audit reveals that POLLER-DTU-FIDELITY-AUDIT-2026-05-29 v1.1 was INCOMPLETE in scope.
The audit's `sources_read:` listed `crates/prism-dtu-{crowdstrike,claroty,armis,cyberint}/src/{clone,types}.rs`
but did NOT list `crates/prism-dtu-harness/src/clones/{sensor}.rs`. This is the process-gap
codified as F-LP1-OBS-001 and addressed by ADR-031 v1.1 §D7.

A v1.2 addendum note is appended to POLLER-DTU-FIDELITY-AUDIT-2026-05-29 separately.

**Effective fidelity status after combining both audits:**

| Sensor | Canonical DTU ready | Harness clone ready | Combined status |
|--------|--------------------|--------------------|-----------------|
| CrowdStrike | YES | YES | **READY** |
| Claroty | PARTIAL (no audit_log) | PARTIAL (no audit_log) | **PARTIAL** (same gap) |
| Armis | YES | YES | **READY** |
| Cyberint | NO (BLOCKING) | NO (BLOCKING) | **NOT READY** |

---

## 10. Recommended Next Dispatch Order

1. **S-DTU-CYBERINT-AUTH-FIDELITY-001 (expanded scope)** — fix BOTH `prism-dtu-cyberint`
   AND `prism-dtu-harness/src/clones/cyberint.rs` in the same PR. The Pass 1 LOCAL adversary
   cascade should resume at Pass 2 after the implementer delivers this expanded scope.

2. **S-DEMO-001** — blocked until S-DTU-CYBERINT-AUTH-FIDELITY-001 merges.

3. **S-DEMO-CLAROTY-PAGINATION-001 (P1)** — pipeline fix; no harness changes required.

4. **S-DEMO-CLAROTY-AUDIT-DTU-001 (P2)** — implementer must add audit_log route to BOTH
   `prism-dtu-claroty` AND `prism-dtu-harness/src/clones/claroty.rs` in the same PR.
