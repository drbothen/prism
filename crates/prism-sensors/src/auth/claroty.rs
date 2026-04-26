//! Claroty xDome API authentication credentials and adapter.
//!
//! # Auth credential (S-2.06)
//! [`ClarotyAuth`] — username/password; sealed via `SensorAuth`.
// Stubs: fields and methods are intentionally unused until implementation.
#![allow(dead_code)]
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

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::types::SensorType;
use reqwest::Client;
use secrecy::SecretString;
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
/// (`"550e8400-e29b-41d4-a716-446655440000"`).  This enum normalizes both
/// representations so that downstream cursor comparison and deduplication
/// treat them deterministically (BC-2.01.007).
///
/// # Serialization
/// Serialized back to a canonical string for downstream field usage:
/// - `Int(n)` → `"12345"`
/// - `Uuid(u)` → `"550e8400-..."` (hyphenated lowercase)
///
/// # GREEN-BY-DESIGN
/// The `Display` impl is a trivial `match` — implemented here because a test
/// that asserts `ClarotyId::Int(12345).to_string() == "12345"` would be
/// tautological if the body were `todo!()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum ClarotyId {
    /// Integer ID (JSON number, e.g., `12345`).
    Int(i64),
    /// UUID ID (JSON string, e.g., `"550e8400-e29b-41d4-a716-446655440000"`).
    Uuid(Uuid),
}

impl std::fmt::Display for ClarotyId {
    /// GREEN-BY-DESIGN: trivial format dispatch; no business logic.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClarotyId::Int(n) => write!(f, "{n}"),
            ClarotyId::Uuid(u) => write!(f, "{u}"),
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
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!(
            "AC-4 / BC-2.01.007: visit_i64 → ClarotyId::Int; \
             visit_str → Uuid::parse_str → ClarotyId::Uuid; \
             use serde::de::Visitor pattern"
        )
    }
}

// ---------------------------------------------------------------------------
// ClarotyAdapter — SensorAdapter implementation
// ---------------------------------------------------------------------------

/// Claroty xDome adapter implementing bearer token auth and offset pagination.
///
/// # Bearer token
/// The bearer token is retrieved from the credential store once at construction
/// and included as `Authorization: Bearer {token}` on every request.
///
/// # Pagination
/// Uses `crate::pagination::paginate_claroty()` for `audit_logs` endpoint;
/// other endpoints use standard single-page or cursor-based fetches per the
/// source type.
///
/// # POST-for-read
/// Claroty uses POST for read operations — `fetch()` issues POST requests even
/// for data reads (BC-2.01.007 postcondition).
///
/// Story: S-2.07 | BC: BC-2.01.004, BC-2.01.007
pub struct ClarotyAdapter {
    /// xDome instance base URL (e.g., `"https://acme.claroty.com"`).
    pub(crate) instance_url: String,
    /// Shared HTTP client.
    pub(crate) http: Client,
    /// Static bearer token (retrieved from credential store at construction).
    ///
    /// Stored as plain `String` after secure retrieval; never logged.
    pub(crate) bearer_token: String,
}

impl std::fmt::Debug for ClarotyAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClarotyAdapter")
            .field("instance_url", &self.instance_url)
            .field("bearer_token", &"Bearer(***)")
            .finish()
    }
}

impl ClarotyAdapter {
    /// Constructs a new adapter, retrieving the bearer token from the
    /// credential store.
    ///
    /// BC: BC-2.01.007 (precondition: valid bearer token in credential store)
    pub fn new(_auth: &ClarotyAuth, _bearer_token: String) -> Self {
        todo!(
            "BC-2.01.007: build reqwest::Client; store instance_url and \
             bearer_token; no cookie store needed (bearer auth)"
        )
    }

    /// Issues a POST-for-read request to `endpoint` with `body` as JSON.
    ///
    /// Includes `Authorization: Bearer {self.bearer_token}` header.
    ///
    /// BC: BC-2.01.007 postcondition (POST-for-read pattern)
    pub(crate) async fn post_read(
        &self,
        _endpoint: &str,
        _body: &serde_json::Value,
    ) -> Result<serde_json::Value, SensorError> {
        todo!(
            "BC-2.01.007: POST endpoint with bearer header and JSON body; \
             parse response; handle 401 → category: authentication"
        )
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
    /// For other sources: uses `post_read()` with offset-based or standard pagination.
    ///
    /// All `id` fields are deserialized as `ClarotyId` to handle polymorphic
    /// integer/UUID representations (BC-2.01.007).
    ///
    /// BC: BC-2.01.004, BC-2.01.007
    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        todo!(
            "BC-2.01.004 / BC-2.01.007: acquire_http_permit(); \
             dispatch to paginate_claroty() for audit_logs or post_read() \
             for other sources; deserialize ClarotyId fields; \
             return Vec<RecordBatch>"
        )
    }
}
