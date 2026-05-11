//! Cyberint portal API authentication credentials and adapter.
//!
//! # Auth credential (S-2.06)
//! [`CyberintAuth`] — API key used as cookie credential; sealed via `SensorAuth`.
//!
//! # Adapter (S-2.07)
//! [`CyberintAdapter`] — implements [`SensorAdapter`] with:
//! - Cookie-based auth: `POST /login` → `Set-Cookie` session cookie injected into
//!   subsequent requests via `reqwest`'s built-in cookie store.
//! - 401 re-authentication: on 401 response, re-authenticates and retries once.
//! - Multi-format timestamp parsing via `crate::timestamp::parse_timestamp()`.
//!
//! Story: S-2.06 (credentials) / S-2.07 (adapter) | BC: BC-2.01.006, BC-2.01.013

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::SensorId;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

use super::{private::Sealed, SensorAuth};
use crate::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};

// ---------------------------------------------------------------------------
// CyberintAuth — credential struct (S-2.06, unchanged)
// ---------------------------------------------------------------------------

/// Cyberint portal API key credentials.
///
/// `Debug` omits the `api_key` value — credentials MUST NOT transit AI context.
pub struct CyberintAuth {
    /// Cyberint portal environment (e.g., `"portal"`, `"portal.eu"`).
    pub environment: String,
    /// Cyberint API key — MUST NOT appear in any log output.
    pub api_key: SecretString,
}

impl std::fmt::Debug for CyberintAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CyberintAuth")
            .field("environment", &self.environment)
            .field("api_key", &"Secret(***)")
            .finish()
    }
}

impl Sealed for CyberintAuth {}
impl SensorAuth for CyberintAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// CyberintAdapter — SensorAdapter implementation
// ---------------------------------------------------------------------------

/// Cyberint portal adapter implementing cookie-based authentication.
pub struct CyberintAdapter {
    /// Canonical org identity for this adapter instance (BC-3.2.001 precondition 4).
    ///
    /// Stored at construction time; verified against `SensorSpec.org_id` at the
    /// start of every `fetch()` call.  A mismatch returns
    /// `SensorError::OrgIdMismatch` immediately, before any network I/O.
    pub(crate) org_id: prism_core::OrgId,
    /// Base API URL derived from `auth.environment`
    pub(crate) base_url: String,
    /// Shared HTTP client with cookie store enabled.
    pub(crate) http: Client,
    /// Whether we have performed the initial login.
    pub(crate) logged_in: Arc<AtomicBool>,
}

impl std::fmt::Debug for CyberintAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CyberintAdapter")
            .field("org_id", &self.org_id)
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl CyberintAdapter {
    /// Constructs a new adapter.
    ///
    /// Builds the HTTP client with `cookie_store(true)` for automatic cookie
    /// management (BC-2.01.006 §Dev Notes).
    ///
    /// # Arguments
    /// - `org_id` — canonical org identity; stored and verified on every `fetch()` call
    ///   (BC-3.2.001 precondition 4, AC-001).
    /// - `auth`   — Cyberint portal API credentials.
    pub fn new(org_id: prism_core::OrgId, auth: &CyberintAuth) -> Self {
        // Tests pass a raw URL as environment; production uses a domain.
        let base_url = if auth.environment.starts_with("http") {
            auth.environment.clone()
        } else {
            format!("https://{}.cyberint.io", auth.environment)
        };

        let http = Client::builder()
            .cookie_store(true)
            .build()
            .unwrap_or_default();

        Self {
            org_id,
            base_url,
            http,
            logged_in: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Authenticates with the Cyberint portal via `POST /login`.
    ///
    /// BC: BC-2.01.006 (postcondition: access_token cookie header present)
    pub(crate) async fn login(&self, auth: &CyberintAuth) -> Result<(), SensorError> {
        let url = format!("{}/login", self.base_url);
        let body = serde_json::json!({ "api_key": auth.api_key.expose_secret() });

        let resp =
            self.http
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| SensorError::Internal {
                    detail: format!("Cyberint login request failed: {e}"),
                })?;

        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(SensorError::HttpError {
                sensor: "cyberint".to_string(),
                status: code,
                body: body_text,
            });
        }

        // reqwest cookie store captures Set-Cookie automatically.
        self.logged_in.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Fetches a data page from `endpoint` using the session cookie.
    ///
    /// On a 401 response, calls `login()` to re-authenticate and retries the
    /// request once (BC-2.01.006 §cookie refresh).
    pub(crate) async fn get_page(
        &self,
        auth: &CyberintAuth,
        endpoint: &str,
        _params: &QueryParams,
    ) -> Result<Vec<serde_json::Value>, SensorError> {
        let url = format!("{}{}", self.base_url, endpoint);

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| SensorError::Internal {
                detail: format!("Cyberint GET request failed: {e}"),
            })?;

        let status = resp.status();

        if status.as_u16() == 401 {
            // Re-login and retry once.
            self.login(auth).await?;
            let resp2 = self
                .http
                .get(&url)
                .send()
                .await
                .map_err(|e| SensorError::Internal {
                    detail: format!("Cyberint GET retry failed: {e}"),
                })?;

            let status2 = resp2.status();
            if !status2.is_success() {
                let code = status2.as_u16();
                let body_text = resp2.text().await.unwrap_or_default();
                return Err(SensorError::HttpError {
                    sensor: "cyberint".to_string(),
                    status: code,
                    body: body_text,
                });
            }
            return self.parse_page_response(resp2).await;
        }

        if !status.is_success() {
            let code = status.as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(SensorError::HttpError {
                sensor: "cyberint".to_string(),
                status: code,
                body: body_text,
            });
        }

        self.parse_page_response(resp).await
    }

    async fn parse_page_response(
        &self,
        resp: reqwest::Response,
    ) -> Result<Vec<serde_json::Value>, SensorError> {
        let json: serde_json::Value =
            resp.json().await.map_err(|e| SensorError::ResponseParse {
                sensor: "cyberint".to_string(),
                detail: format!("response parse error: {e}"),
            })?;

        let records = json
            .get("data")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // Parse timestamps in each record via parse_timestamp().
        // We validate timestamps but store the raw JSON values.
        for record in &records {
            if let Some(ts_str) = record.get("created_at").and_then(|v| v.as_str()) {
                // Best-effort parse — log but don't fail on bad timestamps.
                let _ = crate::timestamp::parse_timestamp(ts_str);
            }
        }

        Ok(records)
    }

    /// Derives the data endpoint from the source table name.
    fn endpoint_from_spec(spec: &SensorSpec) -> String {
        // "cyberint_alert" → "/api/alerts", "cyberint_event" → "/api/events", etc.
        let resource = spec
            .source_table
            .strip_prefix("cyberint_")
            .unwrap_or("alert");
        format!("/api/{resource}s")
    }
}

#[async_trait]
impl SensorAdapter for CyberintAdapter {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("cyberint")
    }

    fn sensor_name(&self) -> &'static str {
        "cyberint"
    }

    /// Fetches one page from the Cyberint API.
    ///
    /// BC: BC-2.01.006
    async fn fetch(
        &self,
        spec: &SensorSpec,
        params: &QueryParams,
        auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // OrgId mismatch guard (BC-3.2.001 precondition 4, AC-004).
        // Must fire before any network I/O.
        if spec.org_id != self.org_id {
            return Err(SensorError::OrgIdMismatch {
                adapter_org_id: self.org_id,
                query_org_id: spec.org_id,
            });
        }

        // Acquire HTTP semaphore permit.
        let _permit = crate::http::acquire_http_permit().await?;

        // Downcast auth to &CyberintAuth.
        let cy_auth = auth
            .as_any()
            .downcast_ref::<CyberintAuth>()
            .ok_or_else(|| SensorError::Internal {
                detail: "auth downcast to CyberintAuth failed".to_string(),
            })?;

        // Login if we haven't yet.
        if !self.logged_in.load(Ordering::SeqCst) {
            self.login(cy_auth).await?;
        }

        let endpoint = Self::endpoint_from_spec(spec);
        let records = self.get_page(cy_auth, &endpoint, params).await?;

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
