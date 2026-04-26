//! Armis Centrix API authentication credentials and adapter.
//!
//! # Auth credential (S-2.06)
//! [`ArmisAuth`] — static API secret key (bearer token); sealed via `SensorAuth`.
//!
//! # Adapter (S-2.07)
//! [`ArmisAdapter`] — implements [`SensorAdapter`] with:
//! - Static bearer token auth (`Authorization: Bearer {token}` on all requests).
//! - AQL query forwarding: passes `SensorSpec.aql_query` verbatim to the
//!   Armis GetSearch endpoint (`aql` parameter); constructs a default AQL from
//!   table name if `aql_query` is absent.
//! - Timestamp fallback chain: `firstSeen` → `lastSeen` → `DateTime::now()`
//!   (with `tracing::warn!` on fallback to `now()`).
//!
//! Story: S-2.06 (credentials) / S-2.07 (adapter) | BC: BC-2.01.008, BC-2.01.013

use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use prism_core::types::SensorType;
use reqwest::Client;
use secrecy::SecretString;

use super::{private::Sealed, SensorAuth};
use crate::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};

// ---------------------------------------------------------------------------
// ArmisAuth — credential struct (S-2.06, unchanged)
// ---------------------------------------------------------------------------

/// Armis Centrix REST API key credentials.
///
/// `Debug` omits the `secret_key` value — credentials MUST NOT transit AI context.
pub struct ArmisAuth {
    /// Armis tenant base URL (e.g., `"https://acme.armis.com"`).
    pub instance_url: String,
    /// Armis API secret key — MUST NOT appear in any log output.
    pub secret_key: SecretString,
}

impl std::fmt::Debug for ArmisAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArmisAuth")
            .field("instance_url", &self.instance_url)
            .field("secret_key", &"Secret(***)")
            .finish()
    }
}

impl Sealed for ArmisAuth {}
impl SensorAuth for ArmisAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// AQL constants
// ---------------------------------------------------------------------------

/// Default AQL template used when `SensorSpec.aql_query` is absent.
///
/// `{table}` is substituted at runtime with `spec.source_table`.
/// BC: BC-2.01.008 (AQL forwarding postcondition).
pub const DEFAULT_AQL_TEMPLATE: &str = "in:{table}";

// ---------------------------------------------------------------------------
// ArmisAdapter — SensorAdapter implementation
// ---------------------------------------------------------------------------

/// Armis Centrix adapter implementing AQL forwarding and timestamp fallback.
pub struct ArmisAdapter {
    /// Armis tenant base URL.
    pub(crate) instance_url: String,
    /// Shared HTTP client.
    pub(crate) http: Client,
    /// Bearer access token obtained from the Armis access-token endpoint.
    pub(crate) bearer_token: String,
}

impl std::fmt::Debug for ArmisAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArmisAdapter")
            .field("instance_url", &self.instance_url)
            .field("bearer_token", &"Bearer(***)")
            .finish()
    }
}

impl ArmisAdapter {
    /// Constructs a new adapter.
    pub fn new(auth: &ArmisAuth, bearer_token: String) -> Self {
        let http = Client::builder()
            .cookie_store(false)
            .build()
            .unwrap_or_default();

        Self {
            instance_url: auth.instance_url.clone(),
            http,
            bearer_token,
        }
    }

    /// Constructs the AQL query string for a fetch.
    ///
    /// If `spec.sensor_config["aql_query"]` is a non-null string, returns it
    /// verbatim (NO modification — architecture compliance rule).
    /// Otherwise derives a default AQL from `spec.source_table`.
    ///
    /// BC: BC-2.01.008 (AQL verbatim forwarding rule)
    pub(crate) fn build_aql(&self, spec: &SensorSpec, _params: &QueryParams) -> String {
        // Check for an explicit AQL query in sensor config.
        if let Some(aql) = spec
            .sensor_config
            .get("aql_query")
            .and_then(|v| v.as_str())
        {
            // Return VERBATIM — no modification, sanitization, or injection prevention.
            return aql.to_string();
        }
        // Default: substitute table name into template.
        DEFAULT_AQL_TEMPLATE.replace("{table}", &spec.source_table)
    }

    /// Resolves the timestamp for an Armis asset record using the fallback chain.
    ///
    /// Tries `firstSeen`, then `lastSeen`, then `Utc::now()`.
    /// Emits `tracing::warn!` when the `now()` fallback is used (AC-6, EC-005).
    pub(crate) fn resolve_timestamp(
        &self,
        record: &serde_json::Value,
        spec: &SensorSpec,
    ) -> DateTime<Utc> {
        // Try firstSeen.
        if let Some(ts_str) = record.get("firstSeen").and_then(|v| v.as_str()) {
            if let Ok(dt) = crate::timestamp::parse_timestamp(ts_str) {
                return dt;
            }
        }

        // Try lastSeen.
        if let Some(ts_str) = record.get("lastSeen").and_then(|v| v.as_str()) {
            if let Ok(dt) = crate::timestamp::parse_timestamp(ts_str) {
                return dt;
            }
        }

        // AC-6 / EC-005: both absent/null/unparseable → use Utc::now() and warn.
        tracing::warn!(
            sensor = "armis",
            table = %spec.source_table,
            client = %spec.client_id,
            "AC-6/EC-005: both firstSeen and lastSeen absent or unparseable; \
             using Utc::now() as timestamp fallback"
        );
        Utc::now()
    }

    /// Issues a GetSearch API call with the given AQL query.
    ///
    /// Includes `Authorization: Bearer {self.bearer_token}` header.
    pub(crate) async fn get_search(
        &self,
        aql: &str,
        _params: &QueryParams,
    ) -> Result<Vec<serde_json::Value>, SensorError> {
        let url = format!("{}/api/v1/search", self.instance_url);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.bearer_token)
            .query(&[("aql", aql)])
            .send()
            .await
            .map_err(|e| SensorError::Internal {
                detail: format!("Armis GetSearch request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(SensorError::HttpError {
                sensor: "armis".to_string(),
                status: code,
                body: body_text,
            });
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| SensorError::ResponseParse {
            sensor: "armis".to_string(),
            detail: format!("GetSearch response parse error: {e}"),
        })?;

        // Armis response: `{ "data": { "results": [...], "total": N } }`
        let results = json
            .get("data")
            .and_then(|d| d.get("results"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(results)
    }
}

#[async_trait]
impl SensorAdapter for ArmisAdapter {
    fn sensor_type(&self) -> SensorType {
        SensorType::Armis
    }

    fn sensor_name(&self) -> &'static str {
        "armis"
    }

    /// Fetches data from the Armis Centrix GetSearch API using AQL.
    ///
    /// BC: BC-2.01.008 (AC-6)
    async fn fetch(
        &self,
        spec: &SensorSpec,
        params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // Acquire HTTP semaphore permit.
        let _permit = crate::http::acquire_http_permit().await?;

        // Build AQL query (verbatim forwarding or default template).
        let aql = self.build_aql(spec, params);

        // Fetch records via GetSearch.
        let records = self.get_search(&aql, params).await?;

        if records.is_empty() {
            return Ok(vec![]);
        }

        // Resolve timestamps for each record (AC-6 fallback chain).
        let _timestamps: Vec<DateTime<Utc>> = records
            .iter()
            .map(|r| self.resolve_timestamp(r, spec))
            .collect();

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
