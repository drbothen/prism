//! Host read routes for the CrowdStrike DTU.
//!
//! - `GET /devices/queries/devices/v1` — paginated host ID list (Step 1)
//! - `GET /devices/entities/devices/v2` — batch host detail fetch (Step 2)

use std::sync::Arc;

use axum::{
    extract::{Query, RawQuery, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
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
/// Paginated host ID list.
///
/// Dual-path (ADR-036 §2.3, BC-2.06.018, F-P6-HIGH-001):
/// - When `state.fixture_gen_seeded == true` (clone built via `new_with_seed`):
///   extracts device IDs from the generated records and serves them.
///   A seeded clone with zero generated devices (e.g. `Archetype::DormantTenant`)
///   serves an EMPTY list — it does NOT fall back to the static fixture.
/// - When `state.fixture_gen_seeded == false` (`new()` / non-seeded path):
///   loads IDs from `load_host_ids()` (static embedded JSON).
///
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

    // W3-FIX-SEC-001: validate X-Org-Id header against instance_org_id (AC-001..AC-003).
    //
    // Validation is active only when instance_org_id is non-nil (i.e., the clone was
    // created with a real org identity via `with_admin_token_and_org`). Clones created
    // with `CrowdstrikeClone::new()` have a nil instance_org_id and skip header
    // validation for backward compat with callers that do not supply X-Org-Id.
    if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
        if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body).into_response();
        }
    }

    // Three-way composition (ADR-036 v2.3 §2.4, BC-2.06.019 PC-4, B-P1-01):
    // - Scenario path (fixture_gen_seeded=true && timeline.is_some()): apply StageMask.
    // - Seeded path (fixture_gen_seeded=true && timeline.is_none()): all generated IDs.
    // - Static path (fixture_gen_seeded=false): load_host_ids() from embedded fixture.
    // Use fixture_gen_seeded (not generated_devices.is_empty()) — DormantTenant guard.
    // F-P6-HIGH-001 / ADR-036 v2.2.
    #[cfg(feature = "fixture-gen")]
    let all_ids: Vec<String> = if state.fixture_gen_seeded {
        if let Some(ref timeline) = state.timeline {
            // Scenario path: apply StageMask projection (BC-2.06.019 PC-4).
            use prism_dtu_common::current_stage_index;
            let now = chrono::Utc::now().timestamp();
            let stage_idx = current_stage_index(timeline, now);
            let mask = &timeline.stages[stage_idx].visible_entity_mask;
            let primary_id = &timeline.entities.primary_device_id_cs;
            let lateral_ids: std::collections::HashSet<&str> = timeline
                .entities
                .lateral_device_ids_cs
                .iter()
                .map(|s| s.as_str())
                .collect();

            state
                .generated_devices
                .iter()
                .filter_map(|rec| {
                    rec.get("device_id")
                        .and_then(|v| v.as_str())
                        .and_then(|id| {
                            // Stage 0 (Baseline): primary device not yet visible.
                            // stage_idx > 0 guard: BC-2.06.019 PC-4 / TV-019-007.
                            let visible = if id == primary_id {
                                mask.primary_device && stage_idx > 0
                            } else if lateral_ids.contains(id) {
                                mask.lateral_devices
                            } else {
                                true
                            };
                            if visible {
                                Some(id.to_owned())
                            } else {
                                None
                            }
                        })
                })
                .collect()
        } else {
            // Seeded path (no scenario): serve all generated IDs (Story-A behavior).
            state
                .generated_devices
                .iter()
                .filter_map(|rec| {
                    rec.get("device_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_owned())
                })
                .collect()
        }
    } else {
        load_host_ids()
    };
    #[cfg(not(feature = "fixture-gen"))]
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
/// # W3-FIX-SEC-001 (AC-001..AC-003, BC-3.5.002 postcondition 2)
///
/// Returns `Ok(OrgId)` when the header is present, parseable as UUID, and matches
/// `instance_org_id` byte-for-byte.
///
/// Returns `Err((401, JSON body))` when:
/// - The header is absent (AC-003)
/// - The header value is not a valid UUID (EC-001)
/// - The parsed UUID does not match `instance_org_id` (AC-002)
pub(crate) fn validate_org_id(
    headers: &HeaderMap,
    instance_org_id: OrgId,
) -> Result<OrgId, (StatusCode, Json<serde_json::Value>)> {
    let mismatch_err = || {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "org_id mismatch: request does not match this clone instance"
            })),
        )
    };

    // AC-003: missing header → 401.
    let header_val = headers
        .get("x-org-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(mismatch_err)?;

    // EC-001: non-UUID value → 401.
    let parsed_uuid = uuid::Uuid::parse_str(header_val).map_err(|_| mismatch_err())?;
    let header_org = OrgId::from_uuid(parsed_uuid);

    // AC-002: UUID present but mismatches instance_org_id → 401.
    if header_org != instance_org_id {
        return Err(mismatch_err());
    }

    Ok(header_org)
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

    // W3-FIX-SEC-001 (HIGH-001 security fix): validate X-Org-Id against instance_org_id.
    // Active only when instance_org_id is non-nil (real org identity assigned by harness).
    if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
        if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body).into_response();
        }
    }

    let requested_ids = parse_ids_from_query(raw_query.as_deref());
    host_details_inner(state, headers, requested_ids).await
}

/// Request body for `POST /devices/entities/devices/v2`.
///
/// Mirrors `GetDetectionSummariesBody` in `detections.rs` — same pattern, same
/// empty-ids validation rule.
///
/// `PostHostDetailsBody` is NOT a pub TOML-deserialized type (it is only used as
/// a JSON request body in the DTU handler), so `#[non_exhaustive]` is NOT required —
/// the compile-fail gate at `tests/external/non-exhaustive-violation/` targets
/// TOML-deserialized and pub-API surface types, not internal DTU request body structs.
/// (DEFECT-CSDEVICES-EMPTY-PIPELINE-001 D-1650 ratification §Contract Part 2)
#[derive(Debug, serde::Deserialize)]
pub struct PostHostDetailsBody {
    /// Device IDs to retrieve details for.
    pub ids: Vec<String>,
}

/// `POST /devices/entities/devices/v2`
///
/// Batch host detail fetch by POST body. Body: `{"ids": ["h-001", "h-002", ...]}`.
/// Returns HTTP 400 when `ids` is empty (mirrors detections `get_detection_summaries`).
///
/// All other behavior (auth, org-id guard, session registry, containment merge,
/// three-way composition) is identical to `get_host_details` — both delegates to
/// `host_details_inner`.
///
/// # DEFECT-CSDEVICES-EMPTY-PIPELINE-001 (D-1650 ratification §Contract Part 2)
///
/// The CrowdStrike canonical endpoint for bulk host detail retrieval is
/// `POST /devices/entities/devices/v2` (PostDeviceDetailsV2, FalconPy v1.2.0+).
/// This handler is the DTU implementation, providing the POST route required by
/// the updated `crowdstrike.sensor.toml` `fetch_devices` step.
pub async fn post_host_details(
    State(state): State<Arc<CrowdstrikeState>>,
    headers: HeaderMap,
    Json(body): Json<PostHostDetailsBody>,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers) {
        return *e;
    }
    // W3-FIX-SEC-001: validate X-Org-Id against instance_org_id (same guard as GET).
    if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
        if let Err((status, body_err)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body_err).into_response();
        }
    }
    if body.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "errors": [{"code": 400, "message": "ids array must not be empty"}]
            })),
        )
            .into_response();
    }
    host_details_inner(state, headers, body.ids).await
}

/// Shared host-detail resolution logic — called by both `get_host_details` and
/// `post_host_details` after auth checks and ID extraction.
///
/// Performs three-way composition (scenario/seeded/static fixtures), session-registry
/// filtering, and containment merge for the given `requested_ids`, returning the
/// `{"resources": [...]}` JSON response.
///
/// Extracted to satisfy D-1650 §Contract Part 2: "extract a SHARED helper so GET and
/// POST paths do not duplicate logic (wiring, not copy-paste)."
///
/// # Three-way composition (ADR-036 v2.3 §2.4, BC-2.06.019 PC-4, B-P1-01)
///
/// - Scenario path (fixture_gen_seeded=true && timeline.is_some()): apply StageMask +
///   containment_status override ("normal") for pre-containment stages (stage < 4).
///   AC-008 / TV-019-011: containment_status="contained" only visible at stage 4.
/// - Seeded path (fixture_gen_seeded=true && timeline.is_none()): all generated records.
/// - Static path (fixture_gen_seeded=false): load from embedded fixture.
///
/// Uses fixture_gen_seeded (not generated_devices.is_empty()) — DormantTenant guard
/// (F-P6-HIGH-001 / ADR-036 v2.2).
async fn host_details_inner(
    state: Arc<CrowdstrikeState>,
    headers: HeaderMap,
    requested_ids: Vec<String>,
) -> Response {
    let org_id = extract_org_id(&headers);

    // For the scenario path we also need stage context for containment_status override.
    // Compute once and reuse in both the fixture-build and the resource-assembly steps.
    // Tuple: (stage_idx, primary_id, lateral_ids, mask_primary_device, mask_lateral_devices)
    #[cfg(feature = "fixture-gen")]
    let scenario_stage_ctx: Option<(
        usize,
        String,
        std::collections::HashSet<String>,
        bool,
        bool,
    )> = if state.fixture_gen_seeded {
        if let Some(ref timeline) = state.timeline {
            use prism_dtu_common::current_stage_index;
            let now = chrono::Utc::now().timestamp();
            let stage_idx = current_stage_index(timeline, now);
            let mask = &timeline.stages[stage_idx].visible_entity_mask;
            let mask_primary = mask.primary_device;
            let mask_lateral = mask.lateral_devices;
            let primary_id = timeline.entities.primary_device_id_cs.clone();
            let lateral_ids: std::collections::HashSet<String> = timeline
                .entities
                .lateral_device_ids_cs
                .iter()
                .cloned()
                .collect();
            Some((
                stage_idx,
                primary_id,
                lateral_ids,
                mask_primary,
                mask_lateral,
            ))
        } else {
            None
        }
    } else {
        None
    };

    #[cfg(feature = "fixture-gen")]
    let fixture: std::collections::HashMap<String, serde_json::Value> = if state.fixture_gen_seeded
    {
        if let Some((stage_idx, ref primary_id, ref lateral_ids, mask_primary, mask_lateral)) =
            scenario_stage_ctx
        {
            // Scenario path: filter by StageMask before building the lookup map.
            state
                .generated_devices
                .iter()
                .filter_map(|rec| {
                    let id = rec.get("device_id").and_then(|v| v.as_str())?;
                    let visible = if id == primary_id {
                        mask_primary && stage_idx > 0
                    } else if lateral_ids.contains(id) {
                        mask_lateral
                    } else {
                        true
                    };
                    if visible {
                        Some((id.to_owned(), rec.clone()))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            // Seeded path (no scenario): all generated devices.
            state
                .generated_devices
                .iter()
                .filter_map(|rec| {
                    rec.get("device_id")
                        .and_then(|v| v.as_str())
                        .map(|id| (id.to_owned(), rec.clone()))
                })
                .collect()
        }
    } else {
        load_host_details()
    };
    #[cfg(not(feature = "fixture-gen"))]
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

            // Scenario path: AC-008 / TV-019-011 — containment_status must be "normal"
            // at pre-containment stages (stage < 4). The generator pre-builds the primary
            // device record with containment_status="contained"; without this override,
            // the stage-2 assertion ("must NOT be 'contained'") would fail.
            // At stage 4 (Containment), all mask fields are true and we serve as-is.
            // Stage index 4 = Containment (activates_after_secs = 600).
            //
            // Precedence rule (BC-2.06.019 PC-4, BPRL-P3-OBS-2): in scenario mode
            // (scenario_stage_ctx.is_some()), stage-driven containment projection takes
            // precedence over operator-driven `containment_store` entries for the primary
            // device at stage < 4. This is by design — the demo narrative controls
            // containment visibility through the stage timeline; operator-driven containment
            // actions (PATCH /devices/entities/devices/actions/v2) are visible only at
            // stage 4 ('Containment', activates_after_secs=600) when the mask permits it.
            // Non-primary devices and non-scenario requests are not subject to this override.
            #[cfg(feature = "fixture-gen")]
            if let Some((stage_idx, ref primary_id, _, _, _)) = scenario_stage_ctx {
                if id == *primary_id && stage_idx < 4 {
                    if let Some(obj) = record.as_object_mut() {
                        obj.insert(
                            "containment_status".to_owned(),
                            serde_json::Value::String("normal".to_owned()),
                        );
                    }
                }
            }

            Some(record)
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({ "resources": resources })),
    )
        .into_response()
}
