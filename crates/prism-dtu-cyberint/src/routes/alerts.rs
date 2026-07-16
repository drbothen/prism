//! Alert route handlers for the Cyberint DTU clone.
//!
//! Routes:
//! - `GET /api/v1/alerts` (or `POST /api/v1/alerts`) — alert list with cursor pagination
//! - `GET /api/v1/alerts/{alert_id}` — alert detail
//! - `PATCH /api/v1/alerts/{alert_id}/status` — acknowledge alert
//! - `POST /api/v1/alerts/{alert_id}/close` — close alert (irreversible in session)
//!
//! All routes require cookie auth — validated via `extract_access_token` (ADR-031 §D3-a).
//!
//! # Auth model (ADR-031 §D3-a)
//!
//! Cyberint uses **account-level static cookie auth**: a pre-registered `access_token`
//! cookie value is validated against an `access_token_allowlist` on every request.
//! There is no `POST /login` step — the real Cyberint API requires no login endpoint.
//! Tokens are org-agnostic (no per-session-per-org routing); the `X-Prism-Org-Id`
//! header selects the alert namespace for multi-tenant isolation, but the token
//! itself is not org-scoped.
//!
//! The legacy per-session-per-org routing model described in BC-3.2.003 was
//! superseded by ADR-031 §D3-a. The `validate_org_id` pattern is not used here.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use prism_core::OrgId;
use serde::Deserialize;

use crate::state::{AuthMode, CyberintState};

/// Query parameters for the alert list endpoint.
#[derive(Debug, Deserialize, Default)]
pub struct AlertListParams {
    pub cursor: Option<String>,
}

/// Extract the `access_token` cookie value from the `Cookie` header.
///
/// Parses the raw `cookie` header (case-insensitive header lookup per HTTP/1.1
/// normalization; axum normalises headers to lowercase). Returns the value of the
/// cookie named exactly `access_token`.
///
/// Returns `None` if:
/// - the `cookie` header is absent
/// - the header value is not valid UTF-8
/// - no `access_token` cookie is present (e.g., only a `cyberint_session` cookie)
///
/// Cookie names are case-sensitive per RFC 6265. `Access-Token` is NOT `access_token`.
///
/// ADR-031 §D3-a rule 2 / AC-002 (S-DTU-CYBERINT-AUTH-FIDELITY-001)
pub fn extract_access_token(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(val) = pair.strip_prefix("access_token=") {
            return Some(val.to_owned());
        }
    }
    None
}

/// Extract the `OrgId` for the current request from the `X-Prism-Org-Id` header.
///
/// # BC-3.2.001 / BC-3.2.003
///
/// The `X-Prism-Org-Id` header carries the canonical org identity minted by the
/// Prism query engine (ADR-008 §2.1).  If the header is absent or unparseable,
/// the `fallback` org is returned — callers should pass `state.instance_org_id`
/// to ensure legacy tests that do not send an org header still work against the
/// correct key namespace.
///
/// Production callers MUST supply a real `X-Prism-Org-Id` header.
pub fn extract_org_id(headers: &HeaderMap, fallback: OrgId) -> OrgId {
    headers
        .get("x-prism-org-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(OrgId::from_uuid)
        .unwrap_or(fallback)
}

/// Return HTTP 401 unauthorized response.
fn unauthorized() -> axum::response::Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error": "unauthorized", "code": 401})),
    )
        .into_response()
}

/// Return HTTP 429 rate-limited response.
fn rate_limited() -> axum::response::Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({"error": "rate limit exceeded", "code": 429})),
    )
        .into_response()
}

/// Check auth and rate limits. Returns `Ok(())` if the request may proceed.
///
/// Validates the `access_token` cookie against the static allowlist in `CyberintState`.
/// A missing cookie or unrecognised token returns HTTP 401. The `cyberint_session` cookie
/// name is explicitly not accepted (ADR-031 §D3-a; AC-003).
///
/// Shared across all authenticated route handlers (`alerts`, `threats`) to avoid
/// duplicating the three-step auth check inline. (F-LP1-MED-001)
///
/// # AC-003 (S-DTU-CYBERINT-AUTH-FIDELITY-001)
pub(crate) fn check_auth(
    state: &CyberintState,
    headers: &HeaderMap,
) -> Result<(), Box<axum::response::Response>> {
    // auth_mode=reject: always 401 regardless of cookie.
    if state.auth_mode() == AuthMode::Reject {
        return Err(Box::new(unauthorized()));
    }

    // Validate access_token cookie (ADR-031 §D3-a; AC-003).
    let token = extract_access_token(headers).ok_or_else(|| Box::new(unauthorized()))?;
    if !state.is_valid_access_token(&token) {
        return Err(Box::new(unauthorized()));
    }

    // Rate limit check.
    if state.check_and_increment_rate_limit() {
        return Err(Box::new(rate_limited()));
    }

    Ok(())
}

/// `GET /api/v1/alerts` or `POST /api/v1/alerts`
///
/// Returns a paginated list of alerts. Merges current status from `alert_store`.
///
/// Three-way composition (ADR-036 §2.3, BC-2.06.018, BC-2.06.019, F-P6-HIGH-001, BPRL-P2-01):
///
/// 1. **Scenario path** (`fixture_gen_seeded == true` AND `state.timeline.is_some()`):
///    Computes `current_stage_index(&timeline, Utc::now().timestamp())`, retrieves the
///    `StageMask`, then filters `generated_records` to `_surface == "alert"` records,
///    excluding any records where the real-schema IOC fields (`ioc.value`, `iocs[].value`,
///    `alert_data.ip`, `alert_data.domain`) reference a catalog IOC whose corresponding
///    mask field (`ioc_ips`, `ioc_domains`, or `ioc_hashes`) is `false`.
///    BC-2.06.019 PC-4 alert-surface semantics: `ioc_ips/ioc_domains/ioc_hashes=false`
///    → alert records referencing those catalog IOCs are excluded from the response.
///    NOTE: the synthetic-ioc filter has been REMOVED (BC-2.06.019 §Interim State);
///    IOC field access uses `Ioc.value` deserialized from `#[serde(rename = "type", alias = "ioc_type")]`
///    on `Ioc.ioc_type` and the primary `value` field (AC-003 / S-DEMO-ENRICHMENT-PIVOT-003).
///
/// 2. **Seeded path** (`fixture_gen_seeded == true`, no timeline):
///    Serves alert-surface records (`_surface == "alert"`) from `generated_records`
///    without stage filtering (Story-A behavior, BC-2.06.018).
///    A seeded clone with zero generated alert records (e.g. `Archetype::DormantTenant`)
///    serves an EMPTY list — it does NOT fall back to `alert_fixture` + `alert_store`.
///
/// 3. **Static-fixture path** (`fixture_gen_seeded == false`, `new()` / non-seeded):
///    Merges `alert_fixture` with `alert_store` status (backward-compatible path).
pub async fn get_alerts(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
    Query(params): Query<AlertListParams>,
) -> impl IntoResponse {
    // W3-FIX-SEC-001 (EC-001): if X-Prism-Org-Id header is present but not a valid UUID,
    // reject with 401 "org_id mismatch" (non-UUID headers cannot be routed to any org).
    if let Some(h) = headers.get("x-prism-org-id") {
        if let Ok(s) = h.to_str() {
            if uuid::Uuid::parse_str(s).is_err() {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "org_id mismatch: request does not match this clone instance"
                    })),
                )
                    .into_response();
            }
        }
    }

    if let Err(resp) = check_auth(&state, &headers) {
        return *resp;
    }

    // Dual-path: use fixture_gen_seeded (not generated_records.is_empty()) so DormantTenant
    // (seeded=true, 0 records) serves empty — not the static fixture. F-P6-HIGH-001.
    // Generated records are immutable after construction — no lock needed.
    #[cfg(feature = "fixture-gen")]
    if state.fixture_gen_seeded {
        // Filter to alert surface records only using the authoritative _surface discriminator.
        //
        // F-P3-CRIT-001 fix: the prior discriminator `rec.get("alert_id").is_some()` was
        // incorrect because generate_cves and generate_iocs ALSO emit `alert_id` (their
        // primary key reuses the same ID format). This caused CVE and IOC records to leak
        // into the `/api/v1/alerts` response, corrupting OCSF normalization (20 of 40 records
        // were non-alert garbage in CompromisedEndpoint archetype).
        //
        // Correct discriminator: `_surface == "alert"` — the generator stamps this tag on
        // every surface independently (generate_alerts → "alert", generate_cves → "cve",
        // generate_iocs → "ioc", generate_asm_assets → "asm_asset").

        // Scenario path: apply StageMask projection (BC-2.06.019 PC-4 / BPRL-P2-01).
        // Must nest INSIDE fixture_gen_seeded=true (three-way composition requirement).
        // DormantTenant guard: branching on fixture_gen_seeded, NOT is_empty().
        if let Some(ref timeline) = state.timeline {
            use prism_dtu_common::current_stage_index;
            let now = chrono::Utc::now().timestamp();
            let stage_idx = current_stage_index(timeline, now);
            let mask = &timeline.stages[stage_idx].visible_entity_mask;

            // Pre-compute catalog IOC sets for O(1) membership tests.
            let catalog_ioc_ips: std::collections::HashSet<&str> = timeline
                .entities
                .ioc_ips
                .iter()
                .map(|s| s.as_str())
                .collect();
            let catalog_ioc_domains: std::collections::HashSet<&str> = timeline
                .entities
                .ioc_domains
                .iter()
                .map(|s| s.as_str())
                .collect();
            let catalog_ioc_hashes: std::collections::HashSet<&str> = timeline
                .entities
                .ioc_hashes
                .iter()
                .map(|s| s.as_str())
                .collect();

            // AC-003 / BC-2.06.019 PC-4: real-schema IOC filter.
            //
            // Filter logic (per BC-2.06.019 PC-4):
            // - `ioc_hashes=false`: withhold alert if ioc.value or any iocs[].value
            //   (deserialized via Ioc.ioc_type/value dual-alias) matches catalog_ioc_hashes.
            // - `ioc_ips=false`: withhold if ioc.value, iocs[].value, or alert_data.ip
            //   matches catalog_ioc_ips.
            // - `ioc_domains=false`: withhold if ioc.value, iocs[].value, or alert_data.domain
            //   matches catalog_ioc_domains.
            // Records that do NOT deserialize as Alert or carry no IOC fields always pass through.
            //
            // Deserialization is via `crate::types::Alert` which has dual-alias serde on Ioc
            // so both "type"/"ioc_type" and "value"/"ioc_value" wire forms are tolerated.

            // Filter to alert-surface records first.
            let alert_records: Vec<&serde_json::Value> = state
                .generated_records
                .iter()
                .filter(|rec| rec.get("_surface").and_then(|v| v.as_str()) == Some("alert"))
                .collect();

            // Collect all IOC values (ioc.value + iocs[].value) for a typed Alert record.
            fn ioc_values_for(alert: &crate::types::Alert) -> Vec<String> {
                let mut vals = Vec::new();
                if let Some(ref ioc) = alert.ioc {
                    vals.push(ioc.value.clone());
                }
                for ioc in &alert.iocs {
                    vals.push(ioc.value.clone());
                }
                vals
            }

            let data: Vec<serde_json::Value> = alert_records
                .into_iter()
                .filter(|rec| {
                    // Try to deserialize as typed Alert for IOC access.
                    // BC-2.06.019 PC-4 step 6: fail-closed.
                    // Records that cannot be deserialized as Alert MUST be withheld:
                    // the StageMask IOC filter cannot be correctly applied to untyped data,
                    // so surfacing an undeserializable record would violate the IOC masking
                    // guarantee. (F-PIVOT003-R2-005: changed from pass-through to withhold.)
                    let typed: crate::types::Alert = match serde_json::from_value((*rec).clone()) {
                        Ok(a) => a,
                        Err(_) => return false, // fail-closed: withhold undeserializable records
                    };

                    let ioc_vals = ioc_values_for(&typed);
                    let alert_ip = typed
                        .alert_data
                        .as_ref()
                        .and_then(|ad| ad.ip.as_deref())
                        .unwrap_or("");
                    let alert_domain = typed
                        .alert_data
                        .as_ref()
                        .and_then(|ad| ad.domain.as_deref())
                        .unwrap_or("");

                    // ioc_hashes=false → withhold if any ioc value is in catalog_ioc_hashes.
                    if !mask.ioc_hashes {
                        for v in &ioc_vals {
                            if catalog_ioc_hashes.contains(v.as_str()) {
                                return false;
                            }
                        }
                    }
                    // ioc_ips=false → withhold if any ioc value or alert_data.ip is in catalog_ioc_ips.
                    if !mask.ioc_ips {
                        for v in &ioc_vals {
                            if catalog_ioc_ips.contains(v.as_str()) {
                                return false;
                            }
                        }
                        if !alert_ip.is_empty() && catalog_ioc_ips.contains(alert_ip) {
                            return false;
                        }
                    }
                    // ioc_domains=false → withhold if any ioc value or alert_data.domain is in catalog_ioc_domains.
                    if !mask.ioc_domains {
                        for v in &ioc_vals {
                            if catalog_ioc_domains.contains(v.as_str()) {
                                return false;
                            }
                        }
                        if !alert_domain.is_empty() && catalog_ioc_domains.contains(alert_domain) {
                            return false;
                        }
                    }

                    true
                })
                .cloned()
                .collect();

            return (
                StatusCode::OK,
                Json(serde_json::json!({"data": data, "next_cursor": serde_json::Value::Null})),
            )
                .into_response();
        }

        // Seeded path (no scenario): serve all alert-surface records (Story-A behavior).
        let data: Vec<serde_json::Value> = state
            .generated_records
            .iter()
            .filter(|rec| {
                // Include ONLY records whose _surface tag is exactly "alert".
                rec.get("_surface").and_then(|v| v.as_str()) == Some("alert")
            })
            .cloned()
            .collect();

        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "data": data,
                "next_cursor": serde_json::Value::Null,
            })),
        )
            .into_response();
    }

    // Static-fixture fallback path.
    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let alert_store = state.alert_store.lock().expect("alert_store poisoned");

    // Simple cursor logic: no cursor → page 1; any cursor value == "page2" → page 2.
    let (alerts_to_return, next_cursor) = if params.cursor.as_deref() == Some("page2") {
        (&state.alert_fixture_page2, serde_json::Value::Null)
    } else {
        (&state.alert_fixture, serde_json::json!("page2"))
    };

    let org_id = extract_org_id(&headers, state.instance_org_id);
    let data: Vec<serde_json::Value> = alerts_to_return
        .iter()
        .map(|a| {
            let status = alert_store
                .get(&(org_id, a.alert_id.clone()))
                .map(|s| s.status.clone())
                .unwrap_or_else(|| "open".to_owned());
            serde_json::json!({
                "alert_id": a.alert_id,
                "title": a.title,
                "severity": a.severity,
                "status": status,
                "created_at": a.created_at,
                "source": a.source,
                "type": a.alert_type,
                "affected_assets": a.affected_assets,
            })
        })
        .collect();

    drop(alert_store);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "data": data,
            "next_cursor": next_cursor,
        })),
    )
        .into_response()
}

/// `GET /api/v1/alerts/{alert_id}`
///
/// Returns single alert with current status from `alert_store`.
/// Returns 404 if the alert_id is not found.
pub async fn get_alert_by_id(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state, &headers) {
        return *resp;
    }

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let alert_store = state.alert_store.lock().expect("alert_store poisoned");

    let org_id = extract_org_id(&headers, state.instance_org_id);
    let status_record = match alert_store.get(&(org_id, alert_id.clone())) {
        Some(s) => s.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "alert not found"})),
            )
                .into_response()
        }
    };
    drop(alert_store);

    // Find the fixture for this alert.
    let alert = state
        .alert_fixture
        .iter()
        .chain(state.alert_fixture_page2.iter())
        .find(|a| a.alert_id == alert_id);

    match alert {
        Some(a) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "alert_id": a.alert_id,
                "title": a.title,
                "severity": a.severity,
                "status": status_record.status,
                "created_at": a.created_at,
                "source": a.source,
                "type": a.alert_type,
                "affected_assets": a.affected_assets,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "alert not found"})),
        )
            .into_response(),
    }
}

/// Body for `PATCH /api/v1/alerts/{alert_id}/status`.
#[derive(Debug, Deserialize)]
pub struct PatchStatusBody {
    pub status: String,
}

/// `PATCH /api/v1/alerts/{alert_id}/status`
///
/// Acknowledges an alert. Updates `alert_store[(org_id, alert_id)].status`.
/// Returns 400 if the alert is already closed.
pub async fn patch_alert_status(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
    Path(alert_id): Path<String>,
    Json(body): Json<PatchStatusBody>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state, &headers) {
        return *resp;
    }

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let mut alert_store = state.alert_store.lock().expect("alert_store poisoned");

    let org_id = extract_org_id(&headers, state.instance_org_id);
    match alert_store.get_mut(&(org_id, alert_id.clone())) {
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "alert not found"})),
        )
            .into_response(),
        Some(record) => {
            if record.closed {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "alert already closed"})),
                )
                    .into_response();
            }
            record.status = body.status.clone();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "alert_id": alert_id,
                    "status": body.status,
                })),
            )
                .into_response()
        }
    }
}

/// `POST /api/v1/alerts/{alert_id}/close`
///
/// Closes an alert irreversibly within the session.
/// Only `reset_all()` can restore the alert to "open".
pub async fn post_close_alert(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state, &headers) {
        return *resp;
    }

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let mut alert_store = state.alert_store.lock().expect("alert_store poisoned");

    let org_id = extract_org_id(&headers, state.instance_org_id);
    match alert_store.get_mut(&(org_id, alert_id.clone())) {
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "alert not found"})),
        )
            .into_response(),
        Some(record) => {
            if record.closed {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "alert already closed"})),
                )
                    .into_response();
            }
            record.status = "closed".to_owned();
            record.closed = true;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "alert_id": alert_id,
                    "status": "closed",
                })),
            )
                .into_response()
        }
    }
}
