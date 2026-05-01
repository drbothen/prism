//! Host read routes for the CrowdStrike DTU.
//!
//! - `GET /devices/queries/devices/v1` — paginated host ID list (Step 1)
//! - `GET /devices/entities/devices/v2` — batch host detail fetch (Step 2)

use std::sync::Arc;

use axum::extract::{Query, RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use prism_core::OrgId;
use serde::Deserialize;

use crate::state::{CrowdstrikeState, SessionData};

/// Query params for host ID list.
#[derive(Debug, Deserialize, Default)]
pub struct HostListParams {
    /// FQL filter string — accepted but not parsed.
    pub filter: Option<String>,
    /// Maximum results to return (default 100).
    pub limit: Option<usize>,
    /// Offset into the result set.
    pub offset: Option<usize>,
}

/// Parse repeated `?ids=val` parameters from raw query string.
fn parse_ids_from_query(raw_query: Option<&str>) -> Vec<String> {
    let qs = raw_query.unwrap_or("");
    qs.split('&')
        .filter_map(|part| {
            let (key, val) = part.split_once('=')?;
            if key == "ids" && !val.is_empty() {
                // URL decode simple percent-encoding
                Some(url_decode(val))
            } else {
                None
            }
        })
        .collect()
}

/// Minimal URL percent-decoding for query param values.
fn url_decode(s: &str) -> String {
    // Replace '+' with space and handle %XX sequences.
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '+' {
            result.push(' ');
        } else if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            let hex = format!("{h1}{h2}");
            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                result.push(b as char);
            } else {
                result.push('%');
                result.push(h1);
                result.push(h2);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Load host IDs from the embedded fixture.
fn load_host_ids() -> Vec<String> {
    let raw = include_str!("../../fixtures/hosts-ids.json");
    // SAFETY: fixture is compiled in via include_str!; parse failure means a corrupt
    // build artifact — panicking at startup is correct behaviour.
    #[allow(clippy::expect_used)]
    serde_json::from_str::<Vec<String>>(raw)
        .expect("hosts-ids.json must be a JSON array of strings")
}

/// Load host detail objects from the embedded fixture, keyed by device_id.
fn load_host_details() -> std::collections::HashMap<String, serde_json::Value> {
    let raw = include_str!("../../fixtures/hosts-detail.json");
    // SAFETY: fixture is compiled in via include_str!; parse failure means a corrupt
    // build artifact — panicking at startup is correct behaviour.
    #[allow(clippy::expect_used)]
    let records: Vec<serde_json::Value> =
        serde_json::from_str(raw).expect("hosts-detail.json must be a JSON array");
    let mut map = std::collections::HashMap::new();
    for record in records {
        if let Some(id) = record.get("device_id").and_then(|v| v.as_str()) {
            map.insert(id.to_owned(), record);
        }
    }
    map
}

/// Validate the `Authorization` header.
fn check_auth(headers: &HeaderMap) -> Result<(), Box<axum::response::Response>> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("").trim();
    if token.is_empty() {
        return Err(Box::new(
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "errors": [{"code": 401, "message": "access denied, authorization required"}]
                })),
            )
                .into_response(),
        ));
    }
    Ok(())
}

/// Shuffle IDs deterministically using the given seed.
fn shuffle_ids_by_seed(ids: &[String], seed: u64) -> Vec<String> {
    use rand::seq::SliceRandom;
    let mut rng = prism_dtu_common::seeded_rng(seed);
    let mut shuffled = ids.to_vec();
    shuffled.shuffle(&mut rng);
    shuffled
}

/// `GET /devices/queries/devices/v1`
///
/// Paginated host ID list. Loads IDs from `fixtures/hosts-ids.json`.
/// Registers returned IDs in session registry under `X-DTU-Session-Id`.
/// Supports `filter` (FQL string, accepted but not parsed), `limit`, `offset` query params.
pub async fn list_host_ids(
    State(state): State<Arc<CrowdstrikeState>>,
    Query(params): Query<HostListParams>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers) {
        return *e;
    }

    let all_ids = load_host_ids();

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let seed = state
        .runtime_config
        .lock()
        .expect("runtime_config poisoned")
        .seed;
    let ordered_ids = shuffle_ids_by_seed(&all_ids, seed);

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100).min(all_ids.len());
    let total = ordered_ids.len();

    let page: Vec<String> = ordered_ids.into_iter().skip(offset).take(limit).collect();

    // Register IDs in session registry if X-DTU-Session-Id header is present.
    if let Some(session_id) = headers
        .get("x-dtu-session-id")
        .and_then(|v| v.to_str().ok())
    {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");
        let entry = registry.get_or_insert_mut(session_id.to_owned(), || SessionData {
            detection_ids: Vec::new(),
            host_ids: Vec::new(),
        });
        for id in &page {
            if !entry.host_ids.contains(id) {
                entry.host_ids.push(id.clone());
            }
        }
    }

    let next_token = if offset + limit < total {
        Some(format!("offset={}", offset + limit))
    } else {
        None
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "resources": page,
            "meta": {
                "pagination": {
                    "offset": offset,
                    "limit": limit,
                    "total": total
                }
            },
            "next_token": next_token
        })),
    )
        .into_response()
}

/// Extract `OrgId` from the `X-Org-Id` request header.
///
/// If the header is absent or unparseable as a UUID, falls back to a fixed
/// default `OrgId` (nil UUID). This keeps backward compatibility with existing
/// tests (e.g. `ac_3_contain_write`) that do not supply an org header.
fn extract_org_id(headers: &HeaderMap) -> OrgId {
    headers
        .get("x-org-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(OrgId::from_uuid)
        .unwrap_or_else(|| OrgId::from_uuid(uuid::Uuid::nil()))
}

/// Validate the `X-Org-Id` header against `instance_org_id`.
///
/// # W3-FIX-SEC-001 (AC-001..AC-003, BC-3.5.002 precondition 3)
///
/// Returns `Ok(OrgId)` when the header is present, parseable as UUID, and matches
/// `instance_org_id` byte-for-byte.
///
/// Returns `Err((401, JSON body))` when:
/// - The header is absent (AC-003)
/// - The header value is not a valid UUID (EC-001)
/// - The parsed UUID does not match `instance_org_id` (AC-002)
#[allow(dead_code)]
pub(crate) fn validate_org_id(
    _headers: &HeaderMap,
    _instance_org_id: OrgId,
) -> Result<OrgId, (StatusCode, Json<serde_json::Value>)> {
    todo!("AC-001/AC-002/AC-003: validate X-Org-Id header against instance_org_id; return 401 on mismatch or absence")
}

/// `GET /devices/entities/devices/v2`
///
/// Batch host detail fetch. Query param: `ids` (repeated, e.g., `?ids=h-001&ids=h-002`).
/// Loads base records from `fixtures/hosts-detail.json` and merges `containment_status`
/// from the `containment_store` for each device.
///
/// # Session registry behavior
///
/// If `X-DTU-Session-Id` is present:
/// - Session found in registry: only return IDs that are both requested AND registered
/// - Session not in registry: return empty (EC-003)
///
/// If `X-DTU-Session-Id` is absent: look up directly from fixture (fidelity probe path).
pub async fn get_host_details(
    State(state): State<Arc<CrowdstrikeState>>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers) {
        return *e;
    }

    let requested_ids = parse_ids_from_query(raw_query.as_deref());

    let org_id = extract_org_id(&headers);

    let fixture = load_host_details();
    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let containment = state
        .containment_store
        .lock()
        .expect("containment_store poisoned")
        .clone();

    // Determine which IDs to look up.
    let ids_to_lookup: Vec<String> = if let Some(session_id) = headers
        .get("x-dtu-session-id")
        .and_then(|v| v.to_str().ok())
    {
        // Session-filtered path.
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");
        if let Some(session_data) = registry.peek(session_id) {
            let registered: std::collections::HashSet<&str> =
                session_data.host_ids.iter().map(|s| s.as_str()).collect();
            requested_ids
                .iter()
                .filter(|id| registered.contains(id.as_str()))
                .cloned()
                .collect()
        } else {
            // Session not in registry → empty (EC-003).
            Vec::new()
        }
    } else {
        // No session header (e.g., fidelity probes) — direct fixture lookup.
        requested_ids.clone()
    };

    let resources: Vec<serde_json::Value> = ids_to_lookup
        .into_iter()
        .filter_map(|id| {
            // Look up base record from fixture.
            let mut record = fixture.get(&id).cloned()?;

            // Merge containment status: store overrides fixture.
            // Key is (org_id, device_id) per BC-3.2.001 — S-3.2.03.
            if let Some(status) = containment.get(&(org_id, id.clone())) {
                if let Some(obj) = record.as_object_mut() {
                    obj.insert(
                        "containment_status".to_owned(),
                        serde_json::Value::String(status.status.clone()),
                    );
                }
            }
            // If not in containment_store: fixture's own containment_status remains.

            Some(record)
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({ "resources": resources })),
    )
        .into_response()
}
