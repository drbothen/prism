//! Route handlers for IP, domain, and hash lookup endpoints.
//!
//! All handlers increment the request counter and enforce rate-limit and auth checks.
//! Fixture dispatch is keyed by the lookup value string.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::state::ThreatIntelState;
use crate::types::FixtureKey;

/// Query parameters for lookup endpoints (API key via `?key=`).
#[derive(Debug, Deserialize)]
pub struct LookupParams {
    pub key: Option<String>,
}

/// Check API key from query param or Authorization header.
/// Returns `Ok(())` if a key is present, `Err(401 response)` otherwise.
fn check_auth(params: &LookupParams, headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
    let has_query_key = params
        .key
        .as_deref()
        .map(|k| !k.is_empty())
        .unwrap_or(false);
    let has_bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && v.len() > 7)
        .unwrap_or(false);

    if has_query_key || has_bearer {
        Ok(())
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing API key", "code": 401})),
        ))
    }
}

/// Build the benign-default response for an unknown lookup value.
fn benign_default(lookup_value: &str) -> Value {
    json!({
        "lookup_value": lookup_value,
        "threat_score": 0,
        "threat_is_known_malicious": false,
        "threat_sources": []
    })
}

/// Resolve a FixtureKey to its fixture response JSON, substituting the lookup value.
fn fixture_response(key: &FixtureKey, lookup_value: &str) -> Value {
    match key {
        FixtureKey::Malicious => json!({
            "lookup_value": lookup_value,
            "threat_score": 85,
            "threat_is_known_malicious": true,
            "threat_sources": ["greynoise", "abuseipdb"]
        }),
        FixtureKey::Benign => json!({
            "lookup_value": lookup_value,
            "threat_score": 5,
            "threat_is_known_malicious": false,
            "threat_sources": ["greynoise"]
        }),
        FixtureKey::Unknown => json!({
            "lookup_value": lookup_value,
            "threat_score": 0,
            "threat_is_known_malicious": false,
            "threat_sources": []
        }),
    }
}

/// Apply rate-limit check after incrementing the counter.
/// Returns `Err(429)` if the threshold is exceeded.
fn check_rate_limit(state: &ThreatIntelState) -> Result<u32, (StatusCode, Json<Value>)> {
    let count = state.increment_counter();
    if state.is_rate_limited(count) {
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": "rate limit exceeded", "code": 429})),
        ))
    } else {
        Ok(count)
    }
}

/// `GET /v3/ip/{ip}` — IP address threat lookup.
pub async fn ip_lookup(
    State(state): State<Arc<ThreatIntelState>>,
    Query(params): Query<LookupParams>,
    headers: HeaderMap,
    Path(ip): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&params, &headers) {
        return resp.into_response();
    }
    if let Err(resp) = check_rate_limit(&state) {
        let mut r = resp.into_response();
        r.headers_mut()
            .insert("retry-after", "30".parse().expect("static header value"));
        return r;
    }
    let body = state
        .lookup_fixture(&ip)
        .as_ref()
        .map(|k| fixture_response(k, &ip))
        .unwrap_or_else(|| benign_default(&ip));
    (StatusCode::OK, Json(body)).into_response()
}

/// `GET /v3/domain/{domain}` — Domain threat lookup (same fixture dispatch as IP).
pub async fn domain_lookup(
    State(state): State<Arc<ThreatIntelState>>,
    Query(params): Query<LookupParams>,
    headers: HeaderMap,
    Path(domain): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&params, &headers) {
        return resp.into_response();
    }
    if let Err(resp) = check_rate_limit(&state) {
        let mut r = resp.into_response();
        r.headers_mut()
            .insert("retry-after", "30".parse().expect("static header value"));
        return r;
    }
    let body = state
        .lookup_fixture(&domain)
        .as_ref()
        .map(|k| fixture_response(k, &domain))
        .unwrap_or_else(|| benign_default(&domain));
    (StatusCode::OK, Json(body)).into_response()
}

/// `GET /v3/hash/{hash}` — File hash threat lookup (VirusTotal-style shape).
pub async fn hash_lookup(
    State(state): State<Arc<ThreatIntelState>>,
    Query(params): Query<LookupParams>,
    headers: HeaderMap,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&params, &headers) {
        return resp.into_response();
    }
    if let Err(resp) = check_rate_limit(&state) {
        let mut r = resp.into_response();
        r.headers_mut()
            .insert("retry-after", "30".parse().expect("static header value"));
        return r;
    }
    let body = match state.lookup_fixture(&hash) {
        Some(FixtureKey::Malicious) => json!({
            "hash": hash,
            "threat_score": 95,
            "threat_is_known_malicious": true,
            "threat_sources": ["virustotal"]
        }),
        Some(FixtureKey::Benign) => json!({
            "hash": hash,
            "threat_score": 2,
            "threat_is_known_malicious": false,
            "threat_sources": ["virustotal"]
        }),
        _ => json!({
            "hash": hash,
            "threat_score": 0,
            "threat_is_known_malicious": false,
            "threat_sources": []
        }),
    };
    (StatusCode::OK, Json(body)).into_response()
}

/// `POST /dtu/configure` — Runtime reconfiguration endpoint.
pub async fn configure(
    State(state): State<Arc<ThreatIntelState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    // Handle rate_limit_after field.
    if let Some(n) = body.get("rate_limit_after").and_then(|v| v.as_u64()) {
        let mut threshold = state
            .rate_limit_after
            .lock()
            .expect("rate_limit_after poisoned");
        *threshold = Some(n as u32);
    }

    // Handle ip + fixture mapping.
    if let (Some(ip), Some(fixture_str)) = (
        body.get("ip").and_then(|v| v.as_str()),
        body.get("fixture").and_then(|v| v.as_str()),
    ) {
        let fixture_key = match fixture_str {
            "malicious" => FixtureKey::Malicious,
            "benign" => FixtureKey::Benign,
            "unknown" => FixtureKey::Unknown,
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "unknown fixture key"})),
                )
                    .into_response();
            }
        };
        let mut registry = state
            .fixture_registry
            .lock()
            .expect("fixture_registry poisoned");
        registry.insert(ip.to_string(), fixture_key);
    }

    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}
