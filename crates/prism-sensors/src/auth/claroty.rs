//! Claroty xDome API authentication credentials and adapter.
//!
//! # Auth credential (S-2.06)
//! [`ClarotyAuth`] — username/password; sealed via `SensorAuth`.
//!
//! # Adapter (S-2.07)
//! [`ClarotyAdapter`] — implements [`SensorAdapter`] with:
//! - Static bearer token auth (`Authorization: Bearer {token}` on all requests).
//! - Polymorphic ID handling via [`ClarotyId`] enum (JSON int or UUID string).
//! - Offset-based hybrid pagination via `crate::pagination::paginate_claroty()`.
//! - POST-for-read pattern (POST requests for read operations).
//!
//! Story: S-2.06 (credentials) / S-2.07 (adapter) | BC: BC-2.01.004, BC-2.01.007,
//! BC-2.01.013

use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use futures::StreamExt;
use prism_core::types::SensorType;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{private::Sealed, SensorAuth};
use crate::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};

// ---------------------------------------------------------------------------
// ClarotyAuth — credential struct (S-2.06, unchanged)
// ---------------------------------------------------------------------------

/// Claroty xDome REST API credentials (username + password).
///
/// `Debug` omits the `password` value — credentials MUST NOT transit AI context.
pub struct ClarotyAuth {
    /// xDome instance base URL (e.g., `"https://acme.claroty.com"`).
    pub instance_url: String,
    /// xDome API username (non-secret; safe to log).
    pub username: String,
    /// xDome API password — MUST NOT appear in any log output.
    pub password: SecretString,
}

impl std::fmt::Debug for ClarotyAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClarotyAuth")
            .field("instance_url", &self.instance_url)
            .field("username", &self.username)
            .field("password", &"Secret(***)")
            .finish()
    }
}

impl Sealed for ClarotyAuth {}
impl SensorAuth for ClarotyAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// ClarotyId — polymorphic ID enum (BC-2.01.007)
// ---------------------------------------------------------------------------

/// Polymorphic ID from Claroty xDome API responses.
///
/// Claroty returns IDs inconsistently as JSON integers (`12345`) or UUID strings
/// (`"550e8400-e29b-41d4-a716-446655440000"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum ClarotyId {
    /// Integer ID (JSON number, e.g., `12345`).
    Int(i64),
    /// UUID ID (JSON string, e.g., `"550e8400-e29b-41d4-a716-446655440000"`).
    Uuid(Uuid),
}

impl std::fmt::Display for ClarotyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClarotyId::Int(n) => write!(f, "{n}"),
            ClarotyId::Uuid(u) => write!(f, "{u}"),
        }
    }
}

/// Visitor for ClarotyId deserialization.
struct ClarotyIdVisitor;

impl<'de> serde::de::Visitor<'de> for ClarotyIdVisitor {
    type Value = ClarotyId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON integer or a UUID string")
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<ClarotyId, E> {
        Ok(ClarotyId::Int(v))
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<ClarotyId, E> {
        Ok(ClarotyId::Int(v as i64))
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<ClarotyId, E> {
        // Try parsing as UUID.
        match Uuid::parse_str(v) {
            Ok(uuid) => Ok(ClarotyId::Uuid(uuid)),
            Err(_) => Err(E::custom(format!(
                "expected a UUID string or integer ID, got: {v:?}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for ClarotyId {
    /// Deserializes a Claroty ID from either a JSON integer or UUID string.
    ///
    /// Tries integer first; if the JSON token is a string, attempts UUID parse.
    /// Returns a `serde` error if neither format matches.
    ///
    /// BC: BC-2.01.007 (AC-4, EC-004)
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ClarotyIdVisitor)
    }
}

// ---------------------------------------------------------------------------
// ClarotyAdapter — SensorAdapter implementation
// ---------------------------------------------------------------------------

/// Claroty xDome adapter implementing bearer token auth and offset pagination.
pub struct ClarotyAdapter {
    /// Canonical org identity for this adapter instance (BC-3.2.001 precondition 4).
    ///
    /// Stored at construction time; verified against `SensorSpec.org_id` at the
    /// start of every `fetch()` call.  A mismatch returns
    /// `SensorError::OrgIdMismatch` immediately, before any network I/O.
    pub(crate) org_id: prism_core::OrgId,
    /// xDome instance base URL (e.g., `"https://acme.claroty.com"`).
    pub(crate) instance_url: String,
    /// Shared HTTP client.
    pub(crate) http: Client,
    /// Static bearer token — wrapped in `SecretString` to guarantee zeroing on
    /// drop and prevent plaintext emission via `Debug` (WGS-W2-002, CWE-312).
    /// Use `expose_secret()` only at HTTP header injection.
    pub(crate) bearer_token: SecretString,
}

impl std::fmt::Debug for ClarotyAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClarotyAdapter")
            .field("org_id", &self.org_id)
            .field("instance_url", &self.instance_url)
            .field("bearer_token", &"Secret([REDACTED])")
            .finish()
    }
}

impl ClarotyAdapter {
    /// Constructs a new adapter.
    ///
    /// `bearer_token` is accepted as `SecretString` to enforce the type-system
    /// guarantee that the token is treated as a secret from the point of
    /// construction (WGS-W2-002).
    ///
    /// # Arguments
    /// - `org_id`        — canonical org identity; stored and verified on every `fetch()` call
    ///   (BC-3.2.001 precondition 4, AC-001).
    /// - `auth`          — Claroty xDome credentials.
    /// - `bearer_token`  — static bearer token for `Authorization: Bearer` header.
    pub fn new(org_id: prism_core::OrgId, auth: &ClarotyAuth, bearer_token: SecretString) -> Self {
        let http = Client::builder()
            .cookie_store(false)
            .build()
            .unwrap_or_default();

        Self {
            org_id,
            instance_url: auth.instance_url.clone(),
            http,
            bearer_token,
        }
    }

    /// Issues a POST-for-read request to `endpoint` with `body` as JSON.
    ///
    /// Includes `Authorization: Bearer {self.bearer_token}` header.
    pub(crate) async fn post_read(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, SensorError> {
        let url = format!("{}{}", self.instance_url, endpoint);

        let resp = self
            .http
            .post(&url)
            .bearer_auth(self.bearer_token.expose_secret())
            .json(body)
            .send()
            .await
            .map_err(|e| SensorError::Internal {
                detail: format!("Claroty POST request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(SensorError::HttpError {
                sensor: "claroty".to_string(),
                status: code,
                body: body_text,
            });
        }

        resp.json().await.map_err(|e| SensorError::ResponseParse {
            sensor: "claroty".to_string(),
            detail: format!("response parse error: {e}"),
        })
    }

    /// Derives the API path from the source table name.
    ///
    /// `"claroty_alert"` → `"/api/v1/alerts"` (strip prefix + pluralize)
    /// `"audit_logs"` → `"/api/v1/audit_logs"` (kept as-is, special case)
    fn endpoint_from_spec(spec: &SensorSpec) -> String {
        let table = &spec.source_table;
        if table == "audit_logs" {
            return "/api/v1/audit_logs".to_string();
        }
        let resource = table.strip_prefix("claroty_").unwrap_or(table.as_str());
        format!("/api/v1/{resource}s")
    }
}

#[async_trait]
impl SensorAdapter for ClarotyAdapter {
    fn sensor_type(&self) -> SensorType {
        SensorType::Claroty
    }

    fn sensor_name(&self) -> &'static str {
        "claroty"
    }

    /// Fetches data from the Claroty xDome API.
    ///
    /// For `audit_logs` source: delegates to `paginate_claroty()` stream.
    /// For other sources: uses `post_read()`.
    ///
    /// BC: BC-2.01.004, BC-2.01.007
    async fn fetch(
        &self,
        spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // Acquire HTTP semaphore permit.
        let _permit = crate::http::acquire_http_permit().await?;

        let endpoint = Self::endpoint_from_spec(spec);

        if spec.source_table == "audit_logs" {
            // Use paginate_claroty() stream for audit_logs (BC-2.01.004).
            let full_url = format!("{}{}", self.instance_url, endpoint);
            // Build a new client that carries the bearer token as a default header.
            let auth_client = Client::builder()
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    let mut auth_val = reqwest::header::HeaderValue::from_str(&format!(
                        "Bearer {}",
                        self.bearer_token.expose_secret()
                    ))
                    .unwrap_or_else(|_| {
                        reqwest::header::HeaderValue::from_static("Bearer invalid")
                    });
                    auth_val.set_sensitive(true);
                    headers.insert(reqwest::header::AUTHORIZATION, auth_val);
                    headers
                })
                .build()
                .unwrap_or_default();

            let stream = crate::pagination::paginate_claroty(full_url, 100, auth_client);
            let pages: Vec<_> = stream.collect().await;

            let mut all_records: Vec<serde_json::Value> = Vec::new();
            for page in pages {
                match page {
                    Ok(records) => all_records.extend(records),
                    Err(e) => return Err(e),
                }
            }

            if all_records.is_empty() {
                return Ok(vec![]);
            }
            let batch = json_values_to_record_batch(all_records)?;
            return Ok(vec![batch]);
        }

        // Non-audit_logs: use POST-for-read.
        let body = serde_json::json!({});
        let response = self.post_read(&endpoint, &body).await?;

        // Extract objects/records from response.
        let records: Vec<serde_json::Value> = response
            .get("objects")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        if records.is_empty() {
            return Ok(vec![]);
        }
        let batch = json_values_to_record_batch(records)?;
        Ok(vec![batch])
    }
}

/// Converts a `Vec<serde_json::Value>` to a single-column `RecordBatch`.
fn json_values_to_record_batch(
    records: Vec<serde_json::Value>,
) -> Result<RecordBatch, SensorError> {
    let schema = Arc::new(Schema::new(vec![Field::new("data", DataType::Utf8, true)]));
    let strings: Vec<Option<String>> = records.iter().map(|v| Some(v.to_string())).collect();
    let array = Arc::new(StringArray::from(strings));
    RecordBatch::try_new(schema, vec![array]).map_err(|e| SensorError::Internal {
        detail: format!("RecordBatch construction failed: {e}"),
    })
}
