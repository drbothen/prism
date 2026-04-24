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
/// Returns `Ok(())` if a valid key is present, `Err(401 response)` otherwise.
///
/// Valid key: non-empty `key` query param, OR `Authorization: Bearer <token>`
/// where `<token>` is non-empty (i.e., the header must have chars after "Bearer ").
fn check_auth(params: &LookupParams, headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
    let has_query_key = params
        .key
        .as_deref()
        .map(|k| !k.is_empty())
        .unwrap_or(false);

    // Bearer token: require non-empty token after the "Bearer " prefix (7 chars).
    let has_bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && !v[7..].trim().is_empty())
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

/// Resolve a FixtureKey to its fixture response JSON for an IP/domain lookup.
fn ip_fixture_response(key: &FixtureKey, ip: &str) -> Value {
    match key {
        FixtureKey::Malicious => json!({
            "ip": ip,
            "threat_score": 85,
            "threat_is_known_malicious": true,
            "threat_sources": ["greynoise", "abuseipdb"],
            "greynoise_classification": "malicious",
            "abuseipdb_confidence_score": 92,
            "virustotal_detections": 14,
            "virustotal_first_seen": "2024-01-15T08:00:00Z"
        }),
        FixtureKey::Benign => json!({
            "ip": ip,
            "threat_score": 5,
            "threat_is_known_malicious": false,
            "threat_sources": ["greynoise"],
            "greynoise_classification": "benign",
            "abuseipdb_confidence_score": 0,
            "virustotal_detections": 0,
            "virustotal_first_seen": null
        }),
        FixtureKey::Unknown => json!({
            "ip": ip,
            "threat_score": 0,
            "threat_is_known_malicious": false,
            "threat_sources": [],
            "greynoise_classification": "unknown",
            "abuseipdb_confidence_score": 0,
            "virustotal_detections": 0,
            "virustotal_first_seen": null
        }),
    }
}

/// Resolve a FixtureKey to its fixture response JSON for a domain lookup.
fn domain_fixture_response(key: &FixtureKey, domain: &str) -> Value {
    match key {
        FixtureKey::Malicious => json!({
            "domain": domain,
            "threat_score": 85,
            "threat_is_known_malicious": true,
            "threat_sources": ["greynoise", "abuseipdb"],
            "greynoise_classification": "malicious",
            "abuseipdb_confidence_score": 90,
            "virustotal_detections": 10,
            "virustotal_first_seen": "2024-02-01T12:00:00Z"
        }),
        FixtureKey::Benign => json!({
            "domain": domain,
            "threat_score": 5,
            "threat_is_known_malicious": false,
            "threat_sources": ["greynoise"],
            "greynoise_classification": "benign",
            "abuseipdb_confidence_score": 0,
            "virustotal_detections": 0,
            "virustotal_first_seen": null
        }),
        FixtureKey::Unknown => json!({
            "domain": domain,
            "threat_score": 0,
            "threat_is_known_malicious": false,
            "threat_sources": [],
            "greynoise_classification": "unknown",
            "abuseipdb_confidence_score": 0,
            "virustotal_detections": 0,
            "virustotal_first_seen": null
        }),
    }
}

/// Build the benign-default response for an unknown IP address.
fn ip_benign_default(ip: &str) -> Value {
    json!({
        "ip": ip,
        "threat_score": 0,
        "threat_is_known_malicious": false,
        "threat_sources": []
    })
}

/// Build the benign-default response for an unknown domain.
fn domain_benign_default(domain: &str) -> Value {
    json!({
        "domain": domain,
        "threat_score": 0,
        "threat_is_known_malicious": false,
        "threat_sources": []
    })
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

/// `GET /v3/ip/:ip` — IP address threat lookup.
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
        .map(|k| ip_fixture_response(k, &ip))
        .unwrap_or_else(|| ip_benign_default(&ip));
    (StatusCode::OK, Json(body)).into_response()
}

/// `GET /v3/domain/:domain` — Domain threat lookup.
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
        .map(|k| domain_fixture_response(k, &domain))
        .unwrap_or_else(|| domain_benign_default(&domain));
    (StatusCode::OK, Json(body)).into_response()
}

/// `GET /v3/hash/:hash` — File hash threat lookup (VirusTotal-style shape).
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
            "threat_sources": ["virustotal"],
            "greynoise_classification": "malicious",
            "abuseipdb_confidence_score": 98,
            "virustotal_detections": 58,
            "virustotal_first_seen": "2023-11-20T00:00:00Z"
        }),
        Some(FixtureKey::Benign) => json!({
            "hash": hash,
            "threat_score": 2,
            "threat_is_known_malicious": false,
            "threat_sources": ["virustotal"],
            "greynoise_classification": "benign",
            "abuseipdb_confidence_score": 0,
            "virustotal_detections": 0,
            "virustotal_first_seen": null
        }),
        _ => json!({
            "hash": hash,
            "threat_score": 0,
            "threat_is_known_malicious": false,
            "threat_sources": [],
            "greynoise_classification": "unknown",
            "abuseipdb_confidence_score": 0,
            "virustotal_detections": 0,
            "virustotal_first_seen": null
        }),
    };
    (StatusCode::OK, Json(body)).into_response()
}

/// Validated configuration payload for `POST /dtu/configure` (TD-WV0-04).
///
/// Unknown fields are rejected by serde to prevent silent misconfiguration.
#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ConfigPayload {
    /// Set rate-limit threshold (requests before 429).
    #[serde(default)]
    rate_limit_after: Option<u32>,
    /// IP address to add to the fixture registry.
    #[serde(default)]
    ip: Option<String>,
    /// Domain to add to the fixture registry.
    #[serde(default)]
    domain: Option<String>,
    /// File hash to add to the fixture registry.
    #[serde(default)]
    hash: Option<String>,
    /// Fixture key to assign (`"malicious"`, `"benign"`, or `"unknown"`).
    #[serde(default)]
    fixture: Option<FixtureKey>,
}

/// `POST /dtu/configure` — Runtime reconfiguration endpoint.
///
/// Accepts:
/// - `{"rate_limit_after": N}` — sets rate-limit threshold
/// - `{"ip": "x.x.x.x", "fixture": "malicious"|"benign"|"unknown"}` — adds IP to registry
/// - `{"hash": "<sha256>", "fixture": "malicious"|"benign"|"unknown"}` — adds hash to registry
/// - `{"domain": "<domain>", "fixture": "malicious"|"benign"|"unknown"}` — adds domain to registry
///
/// Unknown fields are rejected with 400 Bad Request (TD-WV0-04).
pub async fn configure(
    State(state): State<Arc<ThreatIntelState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    // Deserialize with deny_unknown_fields to reject typos / unknown keys (TD-WV0-04).
    let payload: ConfigPayload = match serde_json::from_value(body) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("invalid /dtu/configure payload: {e}")})),
            )
                .into_response();
        }
    };

    // Handle rate_limit_after field.
    if let Some(n) = payload.rate_limit_after {
        let mut threshold = state
            .rate_limit_after
            .lock()
            .expect("rate_limit_after poisoned");
        *threshold = Some(n);
    }

    // Handle lookup_value + fixture mapping (ip or hash or domain).
    let lookup_value = payload
        .ip
        .as_deref()
        .or(payload.domain.as_deref())
        .or(payload.hash.as_deref());

    if let (Some(value), Some(fixture_key)) = (lookup_value, payload.fixture) {
        let mut registry = state
            .fixture_registry
            .lock()
            .expect("fixture_registry poisoned");
        registry.insert(value.to_string(), fixture_key);
    }

    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}
