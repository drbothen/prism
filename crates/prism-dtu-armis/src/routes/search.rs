//! AQL search route handler.
//!
//! Endpoint:
//! - `GET /api/v1/search` — AQL-forwarded unified search (devices or alerts)
//!
//! This is the primary data query path in the real Armis Centrix API (ADR-031 §D8-a).
//! The production poller (poller-coaster) uses `centrix.Search()` with an AQL query
//! string for all 7 data sources. This route closes Gap-AR-001 / DTU-EXT-003 / DTU-EXT-004.
//!
//! # Auth model
//!
//! Requires `Authorization: Bearer {non-empty}` header.
//! Missing/empty token → HTTP 403 `{"error": "...", "code": 403}` (AC-001 EC-004).
//!
//! `X-Org-Id` uses the same **dual-mode** policy as `devices.rs` (CR-012/SEC-P2-001):
//!
//! - Real-org clones (`instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`): absent header → 401.
//! - Default-instance clones: absent header → 200 (backward compat).
//!
//! (BC-3.5.002 precondition 3; TD-VSDD-060 sibling-sweep of devices.rs + alerts.rs guard)
//!
//! # AQL treatment
//!
//! The `aql` query parameter is accepted verbatim, captured via `state.capture_aql()`,
//! and NOT parsed or validated (R-DTU-002 / ADR-005 §D1 opaque AQL model).
//! Simple string pattern matching determines whether to return devices or alerts using the
//! real Armis Centrix `in:<entity>` discriminator syntax (research artifact 2026-06-01):
//! - Contains `in:alerts` → AlertRecord results
//! - Contains `in:devices` (or absent AQL per EC-001) → DeviceRecord results
//! - Both present → devices take precedence (EC-002)
//!
//! Real Armis AQL examples (from 1898 production poller + Armis community):
//! - `in:devices` — all devices
//! - `in:devices type:(switch)` — devices filtered to switch type
//! - `in:alerts status:Open` — open alerts
//!
//! # Response envelope
//!
//! Real Armis `/api/v1/search` returns `{"data": {"results": [...], "total": N}}`.
//! Note `results` (not `devices` or `alerts`) — matches `$.data.results` in armis.sensor.toml.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    routes::devices::validate_org_id,
    state::{ArmisState, DTU_ROUTE_ORG_ID},
    types::ArmisError,
};

/// Query parameters accepted by `GET /api/v1/search`.
///
/// Per AC-001 / AC-002 / AC-003: `aql` carries the AQL filter string.
/// `page` and `size` mirror the real Armis pagination params.
/// `offset` and `limit` are the prism-spec-engine OffsetLimit pagination params
/// (sent by PipelineExecutor via `build_paged_url` — see pipeline.rs `OffsetLimit` arm).
/// Both styles are accepted; when `offset`/`limit` are present they take precedence
/// over `page`/`size`. This allows the DTU to respond correctly to the pipeline's
/// OffsetLimit pagination without requiring the real Armis API's param names.
#[derive(Debug, Deserialize, Default)]
pub struct SearchQueryParams {
    /// AQL filter string — accepted verbatim, stored in AQL log (R-DTU-002).
    pub aql: Option<String>,
    /// Page number (1-based). Defaults to 1. Ignored when `offset` is present.
    pub page: Option<u32>,
    /// Page size. Defaults to 25. Ignored when `limit` is present.
    pub size: Option<u32>,
    /// Zero-based record offset (prism OffsetLimit pagination convention).
    /// When present, `page` is derived as `offset / effective_size + 1`.
    pub offset: Option<u32>,
    /// Maximum records to return (prism OffsetLimit pagination convention).
    /// When present, overrides `size`.
    pub limit: Option<u32>,
}

/// Top-level search response wrapper.
///
/// Real Armis `/api/v1/search` envelope: `{"data": {"results": [...], "total": N}}`.
/// Used by `GET /api/v1/search` (the primary path per ADR-031 §D8-a).
/// `armis.sensor.toml` response_path = `"$.data.results"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub data: SearchData,
}

/// Inner data envelope for the search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchData {
    /// Unified results array — may contain `DeviceRecord` or `AlertRecord` entries
    /// depending on the AQL filter, serialized as JSON objects.
    pub results: Vec<serde_json::Value>,
    /// Total count of matching records (before pagination).
    pub total: u32,
}

/// `GET /api/v1/search` — AQL-forwarded unified search handler.
///
/// Accepts an `aql` query parameter, captures it via `state.capture_aql()`,
/// inspects the AQL string to determine the response type (devices or alerts),
/// and returns the appropriate paginated result in the search envelope format.
///
/// AC-001: registered in `build_router()` → returns 200 with valid Bearer, 403 without.
/// AC-002: `in:devices` AQL → returns DeviceRecord results.
/// AC-003: `in:alerts` AQL → returns AlertRecord results.
/// EC-001: absent AQL → defaults to devices.
/// EC-002: if both `in:devices` and `in:alerts` are present, devices take precedence;
///   route to alerts only when `in:alerts` is present and `in:devices` is absent.
pub async fn get_search(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Query(params): Query<SearchQueryParams>,
) -> impl IntoResponse {
    // AC-001 EC-004: 403 (not 401) for missing/empty Bearer — matches Armis auth model.
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // CR-012/SEC-P2-001: dual-mode X-Org-Id policy (sibling-sweep TD-VSDD-060).
    // Real-org clones (instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID):
    //   auth model A — absent header → 401, mismatch → 401.
    // Default-instance clones (instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID):
    //   validate-on-presence — absent header → skip (backward compat with existing tests).
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body).into_response();
        }
    }

    // R-DTU-002 / ADR-005 §D1: capture AQL verbatim, no parsing or validation.
    // The verbatim capture MUST happen first — before any time-clause parsing.
    // This preserves R-DTU-002: `capture_aql()` stores the raw AQL string unchanged.
    if let Some(ref aql) = params.aql {
        state.capture_aql(aql);
    }

    // §8.3 (pushdown-redesign.md): After verbatim capture, parse AQL for time bounds.
    // This is NOT a violation of R-DTU-002: we do not modify or reject the AQL string.
    // We only use the parsed bounds to filter the fixture dataset (DTU behavioral fidelity).
    let (aql_after_bound, aql_before_bound) = params
        .aql
        .as_deref()
        .map(parse_aql_time_bounds)
        .unwrap_or((None, None));

    // Resolve pagination parameters.
    // Prefer `offset`/`limit` (prism OffsetLimit convention) over `page`/`size` (real Armis API).
    // The prism PipelineExecutor sends `?offset=N&limit=M` via build_paged_url for OffsetLimit steps.
    // The real Armis API uses `?page=N&size=M`. Both conventions are accepted for DTU fidelity.
    let (size, start_offset): (usize, usize) = if let Some(lim) = params.limit {
        // prism OffsetLimit path: `?offset=N&limit=M`
        let effective_size = (lim as usize).max(1);
        let effective_offset = params.offset.unwrap_or(0) as usize;
        (effective_size, effective_offset)
    } else {
        // Real Armis API path: `?page=N&size=M`
        let effective_size = params.size.unwrap_or(25).max(1) as usize;
        let page = params.page.unwrap_or(1).max(1) as usize;
        let effective_offset = (page - 1) * effective_size;
        (effective_size, effective_offset)
    };

    // Determine whether to return alerts or devices based on AQL pattern matching.
    // R-DTU-002: AQL is opaque — only simple string pattern matching is permitted.
    // Real Armis `in:<entity>` discriminator syntax (research-artifact 2026-06-01,
    // grounded from 1898 production poller + 3 independent external connectors):
    //   - `in:alerts` → alert entity selection
    //   - `in:devices` → device entity selection
    // EC-002: if both `in:devices` and `in:alerts` are present, devices take precedence.
    // EC-001: absent AQL → default to devices (safe fallback).
    let return_alerts = params
        .aql
        .as_deref()
        .map(|s| s.contains("in:alerts") && !s.contains("in:devices"))
        .unwrap_or(false);

    if return_alerts {
        // AC-003: alert AQL → paginated AlertRecord results.
        // §8.3: apply AQL time filtering BEFORE pagination (pushdown-redesign.md).
        //
        // Three-way composition (ADR-036 v2.3 §2.4, BC-2.06.019 PC-4, BPRL-P4-02 sibling sweep):
        // - Scenario path (fixture_gen_seeded=true && timeline.is_some()): apply StageMask.
        //   Alert records whose `device_id` equals the primary Armis device are withheld at
        //   stage 0 — mirrors devices.rs `stage_idx > 0` guard.
        //   Detections route added to BC-2.06.019 PC-4 coverage matrix per D-1109.
        // - Seeded path (fixture_gen_seeded=true && timeline.is_none()): all generated alerts.
        //   DormantTenant (seeded=true, 0 records) serves empty — not the static fixture.
        // - Static path (fixture_gen_seeded=false): state.alert_fixture.
        //
        // F-P2-CRIT-001: serve generated alert records as raw serde_json::Value.
        // Use fixture_gen_seeded (not generated_records.is_empty()) — DormantTenant guard.
        // F-P6-HIGH-001.
        #[cfg(feature = "fixture-gen")]
        if state.fixture_gen_seeded {
            // P2-02 (review 2026-06-10 cascade pass-2): apply AQL time-window filtering.
            // Filters on the flat `created_at` key (P2-01 mirror of `time`).
            let stage_filtered: Vec<&serde_json::Value> = if let Some(ref timeline) = state.timeline
            {
                // Scenario path: apply StageMask projection (BC-2.06.019 PC-4 / BPRL-P4-02).
                // Mirror devices.rs paginate_devices scenario logic exactly.
                use prism_dtu_common::current_stage_index;
                let now = chrono::Utc::now().timestamp();
                let stage_idx = current_stage_index(timeline, now);
                let mask = &timeline.stages[stage_idx].visible_entity_mask;
                let primary_id = &timeline.entities.primary_device_id_armis;
                let lateral_ids: std::collections::HashSet<&str> = timeline
                    .entities
                    .lateral_device_ids_armis
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                state
                    .generated_records
                    .iter()
                    .filter(|rec| rec.get("alert_id").is_some())
                    .filter(|rec| {
                        let dev_id = rec.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                        if dev_id.is_empty() {
                            return true;
                        }
                        if dev_id == primary_id {
                            // stage_idx > 0 guard: BC-2.06.019 PC-4 / BPRL-P4-02 sibling sweep.
                            mask.primary_device && stage_idx > 0
                        } else if lateral_ids.contains(dev_id) {
                            mask.lateral_devices
                        } else {
                            true
                        }
                    })
                    .collect()
            } else {
                // Seeded path (no scenario): all generated alert records.
                state
                    .generated_records
                    .iter()
                    .filter(|rec| rec.get("alert_id").is_some())
                    .collect()
            };
            let generated_alerts: Vec<&serde_json::Value> = stage_filtered
                .into_iter()
                .filter(|rec| {
                    generated_in_time_window(
                        rec,
                        "created_at",
                        None,
                        aql_after_bound,
                        aql_before_bound,
                    )
                })
                .collect();
            let total = generated_alerts.len() as u32;
            let page_alerts: Vec<serde_json::Value> = if start_offset >= generated_alerts.len() {
                vec![]
            } else {
                generated_alerts
                    .iter()
                    .skip(start_offset)
                    .take(size)
                    .map(|v| (*v).clone())
                    .collect()
            };
            let body = SearchResponse {
                data: SearchData {
                    results: page_alerts,
                    total,
                },
            };
            return (StatusCode::OK, Json(body)).into_response();
        }

        // Filter by created_at using after:/before: bounds from AQL string.
        let time_filtered_alerts: Vec<&crate::types::AlertRecord> = state
            .alert_fixture
            .iter()
            .filter(|a| alert_in_time_window(a, aql_after_bound, aql_before_bound))
            .collect();
        let total = time_filtered_alerts.len() as u32;

        let page_alerts: Vec<serde_json::Value> = if start_offset >= time_filtered_alerts.len() {
            vec![]
        } else {
            time_filtered_alerts
                .iter()
                .skip(start_offset)
                .take(size)
                .filter_map(|a| serde_json::to_value(*a).ok())
                .collect()
        };

        let body = SearchResponse {
            data: SearchData {
                results: page_alerts,
                total,
            },
        };
        (StatusCode::OK, Json(body)).into_response()
    } else {
        // AC-002: device AQL (or absent AQL per EC-001) → paginated DeviceRecord results.
        // §8.3: apply AQL time filtering BEFORE pagination.
        //
        // F-P2-CRIT-001: dual-path — when generated_records is non-empty, serve
        // generated device records as raw serde_json::Value.
        // Generated records use camelCase Armis-native shapes ("asset_id", "lastSeen", etc.);
        // the adapter reads by response_path "$.data.results" so raw Value is correct.
        // Partition by "asset_id" presence (generator.rs::build_asset always emits "asset_id").
        // Use fixture_gen_seeded (not generated_records.is_empty()) so DormantTenant
        // (seeded=true, 0 records) serves empty — not the static fixture. F-P6-HIGH-001.
        #[cfg(feature = "fixture-gen")]
        if state.fixture_gen_seeded {
            // P2-02: apply the SAME AQL after:/before: window filtering as the
            // static branch (device_in_time_window) — filters on the flat
            // `last_seen` key with `first_seen` fallback (P2-01 mirrors of
            // `lastSeen` / `firstSeen`), matching the §8.3 fallback chain.

            // Scenario path: apply StageMask projection (BC-2.06.019 PC-4 /
            // F-PIVOT003-R8C-001). Mirrors devices.rs::paginate_devices scenario logic
            // exactly. This is the CANONICAL path for `from armis.devices` queries:
            // the `devices` table path_template is `/api/v1/search?aql=${query.filter.aql}`
            // (armis.sensor.toml), so GET /api/v1/search?aql=in:devices is the actual HTTP
            // route that PipelineExecutor calls — NOT GET /api/v1/devices.
            // R7 guard (device_cves_first strip) was previously applied only to devices.rs;
            // it MUST also be applied here on the canonical query path (F-PIVOT003-R8C-001).
            if let Some(ref timeline) = state.timeline {
                use prism_dtu_common::current_stage_index;
                let now = chrono::Utc::now().timestamp();
                let stage_idx = current_stage_index(timeline, now);
                let mask = &timeline.stages[stage_idx].visible_entity_mask;
                let primary_id = &timeline.entities.primary_device_id_armis;
                let lateral_ids: std::collections::HashSet<&str> = timeline
                    .entities
                    .lateral_device_ids_armis
                    .iter()
                    .map(|s| s.as_str())
                    .collect();

                // Stage-filter + time-window filter (same order as devices.rs).
                // stage_idx > 0 guard mirrors devices.rs::paginate_devices exactly:
                // primary device is enrolled in the scenario but not surfaced at Baseline.
                let stage_filtered: Vec<&serde_json::Value> = state
                    .generated_records
                    .iter()
                    .filter(|rec| rec.get("asset_id").is_some() && rec.get("alert_id").is_none())
                    .filter(|rec| {
                        let asset_id = rec.get("asset_id").and_then(|a| a.as_str()).unwrap_or("");
                        if asset_id == primary_id {
                            mask.primary_device && stage_idx > 0
                        } else if lateral_ids.contains(asset_id) {
                            mask.lateral_devices
                        } else {
                            true
                        }
                    })
                    .filter(|rec| {
                        generated_in_time_window(
                            rec,
                            "last_seen",
                            Some("first_seen"),
                            aql_after_bound,
                            aql_before_bound,
                        )
                    })
                    .collect();

                let total = stage_filtered.len() as u32;
                let page_devices: Vec<serde_json::Value> = if start_offset >= stage_filtered.len() {
                    vec![]
                } else {
                    stage_filtered
                        .iter()
                        .skip(start_offset)
                        .take(size)
                        .map(|v| {
                            // BC-2.06.019 PC-4 / F-PIVOT003-R8C-001:
                            // device_cves=false → strip device_cves_first.
                            // Applied on the CANONICAL search path (in:devices AQL →
                            // /api/v1/search) — the path PipelineExecutor actually uses.
                            // Mirrors devices.rs paginate_devices scenario path exactly.
                            if !mask.device_cves {
                                let mut owned = (*v).clone();
                                if let Some(obj) = owned.as_object_mut() {
                                    obj.remove("device_cves_first");
                                }
                                owned
                            } else {
                                (*v).clone()
                            }
                        })
                        .collect()
                };

                let body = SearchResponse {
                    data: SearchData {
                        results: page_devices,
                        total,
                    },
                };
                return (StatusCode::OK, Json(body)).into_response();
            }

            // Seeded path (no scenario): serve all generated device records (Story-A behavior).
            // Same DormantTenant guard: branching on fixture_gen_seeded, not is_empty().
            let generated_devices: Vec<&serde_json::Value> = state
                .generated_records
                .iter()
                .filter(|rec| rec.get("asset_id").is_some() && rec.get("alert_id").is_none())
                .filter(|rec| {
                    generated_in_time_window(
                        rec,
                        "last_seen",
                        Some("first_seen"),
                        aql_after_bound,
                        aql_before_bound,
                    )
                })
                .collect();
            let total = generated_devices.len() as u32;
            let page_devices: Vec<serde_json::Value> = if start_offset >= generated_devices.len() {
                vec![]
            } else {
                generated_devices
                    .iter()
                    .skip(start_offset)
                    .take(size)
                    .map(|v| (*v).clone())
                    .collect()
            };
            let body = SearchResponse {
                data: SearchData {
                    results: page_devices,
                    total,
                },
            };
            return (StatusCode::OK, Json(body)).into_response();
        }

        // Filter by last_seen (with first_seen fallback) using after:/before: bounds.
        let time_filtered_devices: Vec<&crate::types::DeviceRecord> = state
            .devices_ordered
            .iter()
            .filter(|d| device_in_time_window(d, aql_after_bound, aql_before_bound))
            .collect();
        let total = time_filtered_devices.len() as u32;

        let page_devices: Vec<serde_json::Value> = if start_offset >= time_filtered_devices.len() {
            vec![]
        } else {
            time_filtered_devices
                .iter()
                .skip(start_offset)
                .take(size)
                .filter_map(|d| {
                    // BC-3.2.001: merge per-org tag_store entries with fixture tags.
                    let merged_tags = state.tags_for(DTU_ROUTE_ORG_ID, &d.device_id, &d.tags);
                    let merged = crate::types::DeviceRecord {
                        tags: merged_tags,
                        ..(*d).clone()
                    };
                    serde_json::to_value(&merged).ok()
                })
                .collect()
        };

        let body = SearchResponse {
            data: SearchData {
                results: page_devices,
                total,
            },
        };
        (StatusCode::OK, Json(body)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Time-window fixture filtering helpers (§8.3 — pushdown-redesign.md)
// ---------------------------------------------------------------------------

/// Check whether a `DeviceRecord` falls within the given AQL time window.
///
/// Implements pushdown-redesign.md §8.3 fallback chain:
/// 1. Try `last_seen` (primary timestamp).
/// 2. If `last_seen` is null, try `first_seen` (fallback).
/// 3. If both are null, EXCLUDE the record (conservative: cannot confirm in-window).
///
/// When `after_bound` and `before_bound` are both `None` (no time clause in AQL),
/// all records pass the filter (no filtering → verbatim passthrough).
///
/// Boundary semantics: inclusive — records with ts == bound are KEPT here so the
/// push-down result ⊇ the exact DataFusion result (BC-2.11.007 result-equivalence,
/// ADV-P08-MED-001). For a strict `>` PrismQL predicate the AQL emits `after:T` but
/// the boundary record is over-fetched; DataFusion's exact post-filter removes it.
/// For an inclusive `>=` predicate the boundary record must survive both push-down
/// and DataFusion → result-equivalence holds.
fn device_in_time_window(
    device: &crate::types::DeviceRecord,
    after_bound: Option<DateTime<Utc>>,
    before_bound: Option<DateTime<Utc>>,
) -> bool {
    // No time bounds → include all records (no filtering).
    if after_bound.is_none() && before_bound.is_none() {
        return true;
    }
    // Resolve device timestamp: last_seen → first_seen fallback → exclude if both null.
    let ts_str = match device.last_seen.as_deref().or(device.first_seen.as_deref()) {
        Some(s) => s,
        None => return false, // Both null → exclude (conservative per §8.3).
    };
    let ts = match parse_iso8601_utc(ts_str) {
        Some(t) => t,
        None => return false, // Unparseable timestamp → exclude (conservative).
    };
    // Apply bounds with inclusive boundary (keep ts == bound; ADV-P08-MED-001).
    if let Some(after) = after_bound {
        if ts < after {
            return false;
        }
    }
    if let Some(before) = before_bound {
        if ts > before {
            return false;
        }
    }
    true
}

/// Check whether an `AlertRecord` falls within the given AQL time window.
///
/// Filters by `created_at` (the primary alert timestamp, §8.3).
/// Same inclusive-boundary semantics as `device_in_time_window` (ADV-P08-MED-001).
fn alert_in_time_window(
    alert: &crate::types::AlertRecord,
    after_bound: Option<DateTime<Utc>>,
    before_bound: Option<DateTime<Utc>>,
) -> bool {
    if after_bound.is_none() && before_bound.is_none() {
        return true;
    }
    let ts = match parse_iso8601_utc(&alert.created_at) {
        Some(t) => t,
        None => return false,
    };
    // Apply bounds with inclusive boundary (keep ts == bound; ADV-P08-MED-001).
    if let Some(after) = after_bound {
        if ts < after {
            return false;
        }
    }
    if let Some(before) = before_bound {
        if ts > before {
            return false;
        }
    }
    true
}

/// Check whether a SEEDED (generator-emitted) record falls within the given
/// AQL time window (P2-02, review 2026-06-10 cascade pass-2).
///
/// Generic over the flat timestamp key because seeded records are raw
/// `serde_json::Value` (not typed `DeviceRecord` / `AlertRecord`):
/// - devices: `primary = "last_seen"`, `fallback = Some("first_seen")` —
///   the same §8.3 fallback chain as `device_in_time_window`.
/// - alerts: `primary = "created_at"`, no fallback — same as
///   `alert_in_time_window`.
///
/// Semantics are identical to the static-branch helpers: no bounds → include
/// all; missing/unparseable timestamp under active bounds → EXCLUDE
/// (conservative per §8.3); inclusive boundaries (ADV-P08-MED-001).
#[cfg(feature = "fixture-gen")]
fn generated_in_time_window(
    rec: &serde_json::Value,
    primary: &str,
    fallback: Option<&str>,
    after_bound: Option<DateTime<Utc>>,
    before_bound: Option<DateTime<Utc>>,
) -> bool {
    if after_bound.is_none() && before_bound.is_none() {
        return true;
    }
    let ts_str = rec
        .get(primary)
        .and_then(|v| v.as_str())
        .or_else(|| fallback.and_then(|f| rec.get(f).and_then(|v| v.as_str())));
    let Some(ts_str) = ts_str else {
        return false; // No timestamp → exclude (conservative per §8.3).
    };
    let Some(ts) = parse_iso8601_utc(ts_str) else {
        return false; // Unparseable timestamp → exclude (conservative).
    };
    if let Some(after) = after_bound {
        if ts < after {
            return false;
        }
    }
    if let Some(before) = before_bound {
        if ts > before {
            return false;
        }
    }
    true
}

/// Parse an ISO8601 datetime string (with or without `Z`/`+00:00` suffix) to `DateTime<Utc>`.
///
/// Tries multiple formats:
/// - RFC3339 / ISO8601 with timezone (`+00:00`, `Z`).
/// - Timezone-naive `YYYY-MM-DDTHH:MM:SSZ` (trailing Z treated as UTC).
/// - Timezone-naive `YYYY-MM-DDTHH:MM:SS`.
fn parse_iso8601_utc(s: &str) -> Option<DateTime<Utc>> {
    // Try full RFC3339 parsing first (handles Z and +00:00 suffixes).
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Some(dt);
    }
    // Try naive datetime (e.g., "2024-06-11T14:22:00" from fixture without Z).
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(naive.and_utc());
    }
    None
}

// ---------------------------------------------------------------------------
// AQL time-clause parsing (§8.3 — pushdown-redesign.md)
// ---------------------------------------------------------------------------

/// Parse `after:` and `before:` absolute time clauses from an AQL string.
///
/// Implements pushdown-redesign.md §8.3. Called AFTER `capture_aql()` in `get_search`
/// so R-DTU-002 (opaque AQL capture) is preserved. This function only extracts
/// time bounds for fixture dataset filtering — it does NOT modify or reject the AQL.
///
/// Supported formats (research-confirmed, §2.2 / §3 of armis-aql-time-window-syntax-2026-06.md):
/// - `after:YYYY-MM-DDTHH:MM:SS` — bare, unquoted, timezone-naive ISO8601 datetime.
/// - `before:YYYY-MM-DDTHH:MM:SS` — same.
/// - `after:YYYY-MM-DD` — date-only form (BlinkOps source).
/// - `before:YYYY-MM-DD` — same.
///
/// Returns `(after_bound, before_bound)` as UTC datetimes with time defaulting to
/// midnight UTC for date-only forms.
///
/// # Story: S-DEMO-QUERY-PUSHDOWN-001 v2.1
/// Implemented: parses `after:`/`before:` time clauses from AQL.
/// AC-ARMIS-TW-002 LOAD-BEARING test verifies `filtered_count < unfiltered_count`.
pub(crate) fn parse_aql_time_bounds(aql: &str) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
    let after_bound = extract_aql_keyword_bound(aql, "after:");
    let before_bound = extract_aql_keyword_bound(aql, "before:");
    (after_bound, before_bound)
}

/// Extract a `DateTime<Utc>` bound for the given keyword (e.g., `"after:"`) from an AQL string.
///
/// Parses the token immediately following the keyword up to the next whitespace or end-of-string.
/// Accepts both `YYYY-MM-DDTHH:MM:SS` (datetime) and `YYYY-MM-DD` (date-only) forms.
/// Returns `None` if the keyword is absent or the token cannot be parsed.
fn extract_aql_keyword_bound(aql: &str, keyword: &str) -> Option<DateTime<Utc>> {
    // Find the keyword position.
    let pos = aql.find(keyword)?;
    let rest = &aql[pos + keyword.len()..];
    // The timestamp value is everything up to the next whitespace (or end-of-string).
    let token = rest.split_whitespace().next()?;
    if token.is_empty() {
        return None;
    }
    // SEC-004 defense-in-depth: a valid Armis AQL datetime token is at most ~19 chars
    // (e.g., "YYYY-MM-DDTHH:MM:SS"); 32 gives margin for date-only and any valid form.
    // Guard prevents wasted parse work on absurdly long input.
    if token.len() > 32 {
        return None;
    }
    // Try parsing as a full datetime (YYYY-MM-DDTHH:MM:SS — timezone-naive).
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(token, "%Y-%m-%dT%H:%M:%S") {
        return Some(naive.and_utc());
    }
    // Try parsing as date-only (YYYY-MM-DD).
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(token, "%Y-%m-%d") {
        let naive_dt = naive_date.and_hms_opt(0, 0, 0)?;
        return Some(naive_dt.and_utc());
    }
    None
}

/// Validate `Authorization: Bearer {non-empty}` header.
///
/// Returns `Some(response)` on auth failure (HTTP 403) or `None` when valid.
/// Per AC-001 EC-004 / AC-5 (`dtu-assessment.md §3.4`): Armis returns 403, NOT 401.
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

// ---------------------------------------------------------------------------
// S-DEMO-QUERY-PUSHDOWN-001 v2.1 — DTU Red Gate tests for parse_aql_time_bounds
// ---------------------------------------------------------------------------

#[cfg(test)]
mod pushdown_dtu_red_gate_tests {
    //! Red Gate tests for S-DEMO-QUERY-PUSHDOWN-001 v2.1 — DTU side.
    //!
    //! Tests `parse_aql_time_bounds` (the stub function in this module).
    //! ALL tests in this module MUST FAIL before the implementer wires the real parser.
    //!
    //! # SID-1 Compliance
    //! These are in-process unit tests that exercise `parse_aql_time_bounds` directly
    //! without any external DTU dependency. They provide the non-#[ignore] coverage
    //! required by SID-1 §2.
    //!
    //! # Load-bearing assertion
    //! `test_ac_armis_tw_002_dtu_parse_aql_after_clause_yields_bound`:
    //! asserts `after:YYYY-MM-DDTHH:MM:SS` → `Some(DateTime<Utc>)`.
    //! The stub returns (None, None) → test FAILS (Red Gate correctly RED).

    use chrono::Datelike;

    use super::parse_aql_time_bounds;

    /// Unit test for parse_aql_time_bounds: `after:` clause → start bound.
    ///
    /// Red Gate: stub returns (None, None). Test asserts Some(start_bound) → FAILS.
    ///
    /// Canonical Armis AQL absolute time syntax (research-doc §2.2, HIGH confidence):
    /// `after:YYYY-MM-DDTHH:MM:SS` — bare, unquoted, timezone-naive ISO8601.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_002_dtu_parse_aql_after_clause_yields_bound() {
        let (after_bound, before_bound) =
            parse_aql_time_bounds("in:devices after:2024-06-11T12:00:00");

        assert!(
            after_bound.is_some(),
            "parse_aql_time_bounds must parse 'after:2024-06-11T12:00:00' and return \
             Some(start_bound); got None. \
             Red Gate: stub returns (None, None). Implementer must wire the parser. \
             Canonical syntax: bare, unquoted, timezone-naive (research-doc §2.2)."
        );
        assert!(
            before_bound.is_none(),
            "parse_aql_time_bounds: no 'before:' clause → before_bound must be None; \
             got: {:?}",
            before_bound
        );

        // Validate the parsed bound's year/month to guard against off-by-one.
        if let Some(bound) = after_bound {
            assert_eq!(bound.year(), 2024, "parsed start year must be 2024");
            assert_eq!(bound.month(), 6, "parsed start month must be 6 (June)");
            assert_eq!(bound.day(), 11, "parsed start day must be 11");
        }
    }

    /// Unit test: `before:` clause → end bound.
    ///
    /// Red Gate: stub returns (None, None). Test asserts Some(end_bound) → FAILS.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_002_dtu_parse_aql_before_clause_yields_bound() {
        let (after_bound, before_bound) =
            parse_aql_time_bounds("in:devices before:2024-12-31T00:00:00");

        assert!(
            before_bound.is_some(),
            "parse_aql_time_bounds must parse 'before:2024-12-31T00:00:00' and return \
             Some(end_bound); got None. Red Gate: stub returns (None, None)."
        );
        assert!(
            after_bound.is_none(),
            "parse_aql_time_bounds: no 'after:' clause → after_bound must be None; \
             got: {:?}",
            after_bound
        );
    }

    /// Unit test: bounded range `after:T1 before:T2` → both bounds.
    ///
    /// Red Gate: stub returns (None, None). Test asserts (Some, Some) → FAILS.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_002_dtu_parse_aql_bounded_range_yields_both_bounds() {
        let (after_bound, before_bound) = parse_aql_time_bounds(
            "in:devices after:2024-01-01T00:00:00 before:2024-12-31T23:59:59",
        );

        assert!(
            after_bound.is_some(),
            "parse_aql_time_bounds bounded range: must parse 'after:' → Some(start); got None."
        );
        assert!(
            before_bound.is_some(),
            "parse_aql_time_bounds bounded range: must parse 'before:' → Some(end); got None."
        );
    }

    /// Unit test: AQL with no time clause → (None, None).
    ///
    /// PASSES with stub (returns (None, None) — correct behavior).
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_002_dtu_parse_aql_no_time_clause_returns_none() {
        let (after_bound, before_bound) = parse_aql_time_bounds("in:devices");

        assert!(
            after_bound.is_none(),
            "parse_aql_time_bounds: no time clause → after_bound must be None; got: {:?}",
            after_bound
        );
        assert!(
            before_bound.is_none(),
            "parse_aql_time_bounds: no time clause → before_bound must be None; got: {:?}",
            before_bound
        );
    }

    /// SEC-004 defense-in-depth: an over-length token (>32 chars) must return `None`
    /// rather than wasting parse work. A valid 19-char datetime token must still parse.
    #[test]
    fn test_sec_004_aql_over_length_token_returns_none() {
        // 33-char token: exceeds the 32-char cap → must return None.
        let long_token = "2026-01-01T00:00:00XXXXXXXXXXXXXX"; // 33 chars
        assert!(long_token.len() > 32, "test fixture must be >32 chars");
        let aql = format!("in:devices after:{long_token}");
        let (after_bound, _) = parse_aql_time_bounds(&aql);
        assert!(
            after_bound.is_none(),
            "SEC-004: over-length token ({} chars) must return None; got: {:?}",
            long_token.len(),
            after_bound
        );

        // Valid 19-char timezone-naive datetime "YYYY-MM-DDTHH:MM:SS" must still parse.
        let valid_token = "2026-01-01T00:00:00"; // 19 chars
        assert!(valid_token.len() <= 32, "test fixture must be ≤32 chars");
        let aql_valid = format!("in:devices after:{valid_token}");
        let (after_bound_valid, _) = parse_aql_time_bounds(&aql_valid);
        assert!(
            after_bound_valid.is_some(),
            "SEC-004: valid 19-char datetime token must still parse; got None. token='{valid_token}'"
        );
    }
}
