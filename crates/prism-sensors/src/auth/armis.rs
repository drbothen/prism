//! Armis Centrix API authentication credentials and adapter.
//!
//! # Auth credential (S-2.06)
//! [`ArmisAuth`] — static API secret key (bearer token); sealed via `SensorAuth`.
// Stubs: fields and methods are intentionally unused until implementation.
#![allow(dead_code)]
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
impl SensorAuth for ArmisAuth {}

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
///
/// # Bearer token
/// The API secret key is exchanged for an access token via Armis's access-token
/// endpoint at construction, then included as `Authorization: Bearer {token}`
/// on all GetSearch API calls.
///
/// # AQL forwarding (BC-2.01.008)
/// If `spec.sensor_config["aql_query"]` is present, it is forwarded verbatim
/// as the `aql` parameter — NO modification, sanitization, or injection
/// prevention occurs here (Architecture Compliance Rule).
///
/// # Timestamp fallback chain (BC-2.01.008)
/// For each record:
/// 1. Try `firstSeen` field → `parse_timestamp()`.
/// 2. If absent/null/unparseable → try `lastSeen` field → `parse_timestamp()`.
/// 3. If both absent/null/unparseable → use `Utc::now()` and emit
///    `tracing::warn!` with sensor/table/client context (AC-6, EC-005).
///
/// Story: S-2.07 | BC: BC-2.01.008
pub struct ArmisAdapter {
    /// Armis tenant base URL.
    pub(crate) instance_url: String,
    /// Shared HTTP client.
    pub(crate) http: Client,
    /// Bearer access token obtained from the Armis access-token endpoint.
    ///
    /// Never logged.
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
    ///
    /// Exchanges the Armis `secret_key` for a bearer access token at
    /// construction time.
    ///
    /// BC: BC-2.01.008 (precondition: valid API key)
    pub fn new(_auth: &ArmisAuth, _bearer_token: String) -> Self {
        todo!(
            "BC-2.01.008: store instance_url and bearer_token; \
             build reqwest::Client with bearer auth header"
        )
    }

    /// Constructs the AQL query string for a fetch.
    ///
    /// If `spec.sensor_config["aql_query"]` is a non-null string, returns it
    /// verbatim.  Otherwise derives a default AQL from `spec.source_table`
    /// using `DEFAULT_AQL_TEMPLATE`.
    ///
    /// BC: BC-2.01.008 (AQL verbatim forwarding rule)
    pub(crate) fn build_aql(&self, _spec: &SensorSpec, _params: &QueryParams) -> String {
        todo!(
            "BC-2.01.008: extract spec.sensor_config[\"aql_query\"] as Option<&str>; \
             if Some return verbatim; else substitute table name into DEFAULT_AQL_TEMPLATE"
        )
    }

    /// Resolves the timestamp for an Armis asset record using the fallback chain.
    ///
    /// Tries `firstSeen`, then `lastSeen`, then `Utc::now()`.
    /// Emits `tracing::warn!` with `sensor`, `table`, and `client` context
    /// when the `now()` fallback is used (AC-6, EC-005, BC-2.01.008).
    ///
    /// BC: BC-2.01.008
    pub(crate) fn resolve_timestamp(
        &self,
        _record: &serde_json::Value,
        _spec: &SensorSpec,
    ) -> DateTime<Utc> {
        todo!(
            "AC-6 / BC-2.01.008: try record[\"firstSeen\"] → parse_timestamp(); \
             on None/err try record[\"lastSeen\"] → parse_timestamp(); \
             on None/err use Utc::now() and tracing::warn! with context"
        )
    }

    /// Issues a GetSearch API call with the given AQL query.
    ///
    /// Includes `Authorization: Bearer {self.bearer_token}` header.
    ///
    /// BC: BC-2.01.008
    pub(crate) async fn get_search(
        &self,
        _aql: &str,
        _params: &QueryParams,
    ) -> Result<Vec<serde_json::Value>, SensorError> {
        todo!(
            "BC-2.01.008: GET /api/v1/search?aql={{aql}}&...pagination params; \
             bearer header; parse response data array; handle 401 → authentication, \
             400 → api_contract with AQL text"
        )
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
    /// 1. Acquires an HTTP semaphore permit.
    /// 2. Builds the AQL query via `build_aql()`.
    /// 3. Calls `get_search()`.
    /// 4. Resolves timestamps via `resolve_timestamp()` for each record.
    /// 5. Returns a `RecordBatch`.
    ///
    /// BC: BC-2.01.008 (AC-6)
    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        todo!(
            "BC-2.01.008: acquire_http_permit(); downcast auth to &ArmisAuth; \
             build_aql(); get_search(); resolve_timestamp() per record; \
             convert to RecordBatch Vec"
        )
    }
}
