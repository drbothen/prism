//! Alert / policy violation list route handler.
//!
//! Endpoint:
//! - `GET /api/v1/alerts` — returns paginated alert list
//!
//! Auth: requires `Authorization: Bearer {non-empty}` header.
//! Missing/empty token → HTTP 403 `{"error": "...", "code": 403}`.
//!
//! `X-Org-Id` uses the same **dual-mode** policy as `devices.rs`:
//! - Real-org clones (`instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`): absent header → 401.
//! - Default-instance clones: absent header → 200 (backward compat).
//!
//! (CR-017 / M-50-001; BC-3.5.002 precondition 3; BC-3.2.001 precondition 4)
//!
//! # Dual-path (F-P15-MED-001 / ADR-036 §2.3 / BC-2.06.018 / F-P6-HIGH-001)
//!
//! When the clone was built via `new_with_seed` (`fixture_gen_seeded == true`), this
//! handler serves generated alert records from `state.generated_records` — NOT the
//! static `state.alert_fixture`.  `DormantTenant` (seeded=true, 0 generated alert records)
//! serves an EMPTY response.  The sentinel is `fixture_gen_seeded` (NOT `.is_empty()`)
//! so that DormantTenant never falls back to the static fixture (EC-018-003 / F-P6-HIGH-001).

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{
    routes::devices::validate_org_id,
    state::ArmisState,
    types::{AlertRecord, AlertsData, AlertsResponse, ArmisError},
};

/// Query parameters for `GET /api/v1/alerts`.
#[derive(Debug, Deserialize, Default)]
pub struct AlertQueryParams {
    pub page: Option<u32>,
    pub size: Option<u32>,
}

/// `GET /api/v1/alerts`
///
/// Returns a paginated list of alert / policy violation records.
/// Pagination: `page` (1-based, default 1), `size` (default 25).
///
/// # Dual-path (F-P15-MED-001 / ADR-036 §2.3 / BC-2.06.018 / F-P6-HIGH-001)
///
/// When `state.fixture_gen_seeded == true` (clone built via `new_with_seed`):
///   serves generated alert records as raw `serde_json::Value` (Claroty/search.rs pattern).
///   A seeded clone with zero generated alert records (e.g. `Archetype::DormantTenant`)
///   serves EMPTY — it does NOT fall back to the static `alert_fixture`.
/// When `state.fixture_gen_seeded == false` (clone built via `new()`):
///   serves from `state.alert_fixture` (static fixture, backward-compatible path).
///
/// Response envelope: `{"data": {"alerts": [...], "total": N}}` — matching the existing
/// `AlertsResponse` shape consumed by the Armis adapter (response_path `$.data.alerts`).
pub async fn get_alerts(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Query(params): Query<AlertQueryParams>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // CR-017 / M-50-001: dual-mode X-Org-Id policy.
    // Real-org clones (instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID):
    //   auth model A — absent header → 401, mismatch → 401.
    // Default-instance clones (instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID):
    //   validate-on-presence — absent header → skip (backward compat).
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, body_err)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body_err).into_response();
        }
    }

    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(25).max(1) as usize;
    let offset = ((page - 1) as usize) * size;

    // F-P15-MED-001: dual-path — use fixture_gen_seeded (NOT .is_empty()) as sentinel.
    // DormantTenant (seeded=true, 0 records) must serve EMPTY, not the static fixture.
    // Mirrors the pattern used in search.rs (lines ~198-222) and Claroty/Cyberint alert routes.
    #[cfg(feature = "fixture-gen")]
    if state.fixture_gen_seeded {
        // Serve generated alert records partitioned by "alert_id" presence.
        // Generated records use Armis-native shapes; raw serde_json::Value is correct
        // (adapter reads by $.data.alerts response_path and accepts any JSON object).
        let generated_alerts: Vec<&serde_json::Value> = state
            .generated_records
            .iter()
            .filter(|rec| rec.get("alert_id").is_some())
            .collect();
        let total = generated_alerts.len() as u32;
        let page_alerts: Vec<serde_json::Value> = if offset >= generated_alerts.len() {
            vec![]
        } else {
            generated_alerts
                .iter()
                .skip(offset)
                .take(size)
                .map(|v| (*v).clone())
                .collect()
        };
        // Return as hand-assembled JSON to match the AlertsResponse envelope shape
        // (`{"data": {"alerts": [...], "total": N}}`).
        let body = serde_json::json!({
            "data": {
                "alerts": page_alerts,
                "total": total,
            }
        });
        return (StatusCode::OK, Json(body)).into_response();
    }

    // Static-fixture path (backward-compatible: clone built via new()).
    let all_alerts = &state.alert_fixture;
    let total = all_alerts.len() as u32;

    let page_alerts: Vec<AlertRecord> = if offset >= all_alerts.len() {
        vec![]
    } else {
        all_alerts.iter().skip(offset).take(size).cloned().collect()
    };

    let body = AlertsResponse {
        data: AlertsData {
            alerts: page_alerts,
            total,
        },
    };

    (StatusCode::OK, Json(body)).into_response()
}

fn check_bearer_auth(headers: &HeaderMap) -> Option<axum::response::Response> {
    let valid = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && v.len() > "Bearer ".len())
        .unwrap_or(false);

    if valid {
        None
    } else {
        let body = ArmisError {
            error: "invalid or missing bearer token".to_owned(),
            code: 403,
        };
        Some((StatusCode::FORBIDDEN, Json(body)).into_response())
    }
}
