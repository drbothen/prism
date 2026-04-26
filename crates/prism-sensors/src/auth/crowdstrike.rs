//! CrowdStrike Falcon API authentication credentials and adapter.
//!
//! # Auth credential (S-2.06)
//! [`CrowdStrikeAuth`] — OAuth2 client credentials; sealed via `SensorAuth`.
//!
//! # Adapter (S-2.07)
//! [`CrowdStrikeAdapter`] — implements [`SensorAdapter`] with:
//! - OAuth2 token acquisition + token cache (`Arc<RwLock<Option<CachedToken>>>`)
//! - Two-step fetch: QueryV2 (IDs) → batched PostEntities (full records)
//! - Token refresh on 401 (transparent, no caller awareness)
//! - Batch size capped at `CROWDSTRIKE_BATCH_SIZE` (100) per API limit
//!
//! Story: S-2.06 (credentials) / S-2.07 (adapter) | BC: BC-2.01.005, BC-2.01.013

use std::sync::Arc;
use std::time::{Duration, Instant};

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::types::SensorType;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use tokio::sync::RwLock;

use super::{private::Sealed, SensorAuth};
use crate::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};

// ---------------------------------------------------------------------------
// CrowdStrikeAuth — credential struct (S-2.06, unchanged)
// ---------------------------------------------------------------------------

/// CrowdStrike Falcon API OAuth2 client credentials.
///
/// `Debug` omits the `client_secret` value — credentials MUST NOT transit
/// AI context (AI-opaque credential model).
pub struct CrowdStrikeAuth {
    /// OAuth2 client ID (non-secret; safe to log).
    pub client_id: String,
    /// OAuth2 client secret — MUST NOT appear in any log output.
    pub client_secret: SecretString,
    /// CrowdStrike cloud region (e.g., `"us-1"`, `"eu-1"`).
    pub cloud_region: String,
}

impl std::fmt::Debug for CrowdStrikeAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrowdStrikeAuth")
            .field("client_id", &self.client_id)
            .field("client_secret", &"Secret(***)")
            .field("cloud_region", &self.cloud_region)
            .finish()
    }
}

impl Sealed for CrowdStrikeAuth {}
impl SensorAuth for CrowdStrikeAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// CachedToken — internal token cache entry
// ---------------------------------------------------------------------------

/// A cached OAuth2 access token with its expiry timestamp.
#[derive(Debug)]
pub(crate) struct CachedToken {
    /// The raw bearer token string.  MUST NOT be logged.
    pub token: String,
    /// Monotonic instant after which the token is considered expired.
    pub expires_at: Instant,
}

impl CachedToken {
    /// Returns true if the cached token is still valid.
    pub fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of IDs per PostEntities batch request.
///
/// CrowdStrike enforces a hard limit; exceeding it causes API errors.
/// BC-2.01.005 §EC-01-008.
pub const CROWDSTRIKE_BATCH_SIZE: usize = 100;

// ---------------------------------------------------------------------------
// CrowdStrikeAdapter — SensorAdapter implementation
// ---------------------------------------------------------------------------

/// CrowdStrike Falcon adapter implementing the two-step fetch pattern.
pub struct CrowdStrikeAdapter {
    /// Base URL derived from the cloud region (e.g., `"https://api.crowdstrike.com"`).
    pub(crate) base_url: String,
    /// Shared HTTP client.  Configured once at construction.
    pub(crate) http: Client,
    /// Cached OAuth2 token.  `None` until first acquisition.
    pub(crate) token_cache: Arc<RwLock<Option<CachedToken>>>,
}

impl std::fmt::Debug for CrowdStrikeAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrowdStrikeAdapter")
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl CrowdStrikeAdapter {
    /// Constructs a new adapter for the given CrowdStrike cloud region.
    pub fn new(auth: &CrowdStrikeAuth) -> Self {
        // Tests pass a raw URL (e.g. "http://localhost:PORT") as cloud_region.
        // Production uses cloud_region like "us-1" → "https://api.crowdstrike.com".
        // We detect by checking if the value starts with "http".
        let base_url = if auth.cloud_region.starts_with("http") {
            auth.cloud_region.clone()
        } else {
            format!("https://api.{}.crowdstrike.com", auth.cloud_region)
        };

        let http = Client::builder()
            .cookie_store(false)
            .build()
            .unwrap_or_default();

        Self {
            base_url,
            http,
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Acquires (or refreshes) the OAuth2 access token.
    ///
    /// BC: BC-2.01.005
    pub(crate) async fn acquire_token(
        &self,
        auth: &CrowdStrikeAuth,
    ) -> Result<String, SensorError> {
        let url = format!("{}/oauth2/token", self.base_url);
        let params = [
            ("client_id", auth.client_id.as_str()),
            ("client_secret", auth.client_secret.expose_secret()),
        ];

        let resp = self
            .http
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| SensorError::Internal {
                detail: format!("OAuth2 token request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SensorError::HttpError {
                sensor: "crowdstrike".to_string(),
                status: code,
                body,
            });
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| SensorError::ResponseParse {
            sensor: "crowdstrike".to_string(),
            detail: format!("token response parse error: {e}"),
        })?;

        let token = json
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SensorError::ResponseParse {
                sensor: "crowdstrike".to_string(),
                detail: "missing access_token in OAuth2 response".to_string(),
            })?
            .to_string();

        let expires_in = json
            .get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(1799);

        // Store new token in cache under write lock.
        let cached = CachedToken {
            token: token.clone(),
            expires_at: Instant::now() + Duration::from_secs(expires_in.saturating_sub(30)),
        };
        {
            let mut guard = self.token_cache.write().await;
            *guard = Some(cached);
        }

        Ok(token)
    }

    /// Returns a valid bearer token, acquiring/refreshing as needed.
    async fn get_valid_token(&self, auth: &CrowdStrikeAuth) -> Result<String, SensorError> {
        // Fast path: check under read lock first.
        {
            let guard = self.token_cache.read().await;
            if let Some(cached) = guard.as_ref() {
                if cached.is_valid() {
                    return Ok(cached.token.clone());
                }
            }
        }
        // Slow path: acquire/refresh under write lock.
        self.acquire_token(auth).await
    }

    /// Step 1: queries the resource ID list via `GET /queries/{resource_type}`.
    pub(crate) async fn query_resource_ids(
        &self,
        token: &str,
        resource_type: &str,
        _params: &QueryParams,
    ) -> Result<Vec<String>, SensorError> {
        let url = format!("{}/queries/{}", self.base_url, resource_type);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| SensorError::Internal {
                detail: format!("QueryV2 request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SensorError::HttpError {
                sensor: "crowdstrike".to_string(),
                status: code,
                body,
            });
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| SensorError::ResponseParse {
            sensor: "crowdstrike".to_string(),
            detail: format!("QueryV2 response parse error: {e}"),
        })?;

        let ids = json
            .get("resources")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ids)
    }

    /// Step 2: fetches full entity records via batched `POST /entities/{resource_type}/GET`.
    pub(crate) async fn fetch_entities(
        &self,
        token: &str,
        resource_type: &str,
        ids: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, SensorError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let url = format!("{}/entities/{}/GET", self.base_url, resource_type);
        let mut all_records = Vec::new();

        for chunk in ids.chunks(CROWDSTRIKE_BATCH_SIZE) {
            let body = serde_json::json!({ "ids": chunk });
            let resp = self
                .http
                .post(&url)
                .bearer_auth(token)
                .json(&body)
                .send()
                .await
                .map_err(|e| SensorError::Internal {
                    detail: format!("PostEntities request failed: {e}"),
                })?;

            let status = resp.status();
            if status.as_u16() == 401 {
                // Signal 401 so caller can refresh token and retry.
                let body_text = resp.text().await.unwrap_or_default();
                return Err(SensorError::HttpError {
                    sensor: "crowdstrike".to_string(),
                    status: 401,
                    body: body_text,
                });
            }
            if !status.is_success() {
                let code = status.as_u16();
                let body_text = resp.text().await.unwrap_or_default();
                return Err(SensorError::HttpError {
                    sensor: "crowdstrike".to_string(),
                    status: code,
                    body: body_text,
                });
            }

            let json: serde_json::Value =
                resp.json().await.map_err(|e| SensorError::ResponseParse {
                    sensor: "crowdstrike".to_string(),
                    detail: format!("PostEntities response parse error: {e}"),
                })?;

            if let Some(resources) = json.get("resources").and_then(|v| v.as_array()) {
                all_records.extend(resources.iter().cloned());
            }
        }

        Ok(all_records)
    }

    /// Derives the resource type (plural) from the source table name.
    ///
    /// Strips the `"crowdstrike_"` prefix and appends `"s"` to pluralize
    /// (e.g. `"crowdstrike_alert"` → `"alerts"`). Falls back to `"alerts"`.
    fn resource_type_from_spec(spec: &SensorSpec) -> String {
        let singular = spec
            .source_table
            .strip_prefix("crowdstrike_")
            .unwrap_or("alert");
        format!("{singular}s")
    }
}

#[async_trait]
impl SensorAdapter for CrowdStrikeAdapter {
    fn sensor_type(&self) -> SensorType {
        SensorType::CrowdStrike
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike"
    }

    /// Executes the CrowdStrike two-step fetch.
    ///
    /// BC: BC-2.01.005 (AC-1, AC-2)
    async fn fetch(
        &self,
        spec: &SensorSpec,
        params: &QueryParams,
        auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // Acquire HTTP semaphore permit.
        let _permit = crate::http::acquire_http_permit().await?;

        // Downcast auth to &CrowdStrikeAuth.
        let cs_auth = auth
            .as_any()
            .downcast_ref::<CrowdStrikeAuth>()
            .ok_or_else(|| SensorError::Internal {
                detail: "auth downcast to CrowdStrikeAuth failed".to_string(),
            })?;

        // Step 1: acquire valid token.
        let token = self.get_valid_token(cs_auth).await?;
        let resource_type = Self::resource_type_from_spec(spec);

        // Step 2: query IDs.
        let ids = self.query_resource_ids(&token, &resource_type, params).await?;

        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Step 3: fetch entities, with transparent 401 refresh.
        let records = match self.fetch_entities(&token, &resource_type, ids.clone()).await {
            Ok(r) => r,
            Err(SensorError::HttpError { status: 401, .. }) => {
                // Token expired mid-fetch — refresh and retry once.
                let new_token = self.acquire_token(cs_auth).await?;
                self.fetch_entities(&new_token, &resource_type, ids).await?
            }
            Err(e) => return Err(e),
        };

        // Convert records to a minimal RecordBatch.
        let batch = json_values_to_record_batch(records)?;
        Ok(vec![batch])
    }
}

/// Converts a `Vec<serde_json::Value>` to a single-column `RecordBatch`.
///
/// Stores each JSON value as a string in an Arrow `StringArray`.
fn json_values_to_record_batch(
    records: Vec<serde_json::Value>,
) -> Result<RecordBatch, SensorError> {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "data",
        DataType::Utf8,
        true,
    )]));
    let strings: Vec<Option<String>> = records
        .iter()
        .map(|v| Some(v.to_string()))
        .collect();
    let array = Arc::new(StringArray::from(strings));
    RecordBatch::try_new(schema, vec![array]).map_err(|e| SensorError::Internal {
        detail: format!("RecordBatch construction failed: {e}"),
    })
}

