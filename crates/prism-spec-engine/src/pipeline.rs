//! Multi-step fetch pipeline executor (BC-2.16.002).
//!
//! Steps execute sequentially in spec-declared order. Variables from each step
//! are available to subsequent steps via `${step_name.field}` interpolation.
//! Fan-out: when a variable resolves to an array, the step is batched.
//! Rate limit hints from SensorSpec apply between API calls.
//!
//! ## S-PLUGIN-PREREQ-B
//!
//! `execute` and `execute_step` accept `http_client: &reqwest::Client` and
//! `auth_provider: &dyn AuthProvider` as dependency-injected parameters per
//! ADR-023 §C2 and BC-2.16.002. The production bodies are `todo!()` stubs —
//! implemented by the implementer in S-PLUGIN-PREREQ-B.
//!
//! The `fan_out_batches` pure function is unchanged.

use prism_core::{OrgSlug, PrismError};

use crate::auth_provider::AuthProvider;
use crate::spec_parser::{FetchStep, SensorSpec, TableSpec};

/// Context provided to each pipeline execution.
#[derive(Debug, Clone)]
pub struct FetchContext {
    /// The client/tenant this query is executing for.
    pub client_id: OrgSlug,
    /// Push-down filter values from the query planner (${query.filter.*}).
    pub query_filters: std::collections::HashMap<String, String>,
}

/// The output of a successful pipeline execution.
///
/// Contains the raw JSON records from the final step. OCSF mapping (BC-2.16.003)
/// is applied by `ColumnMapper` separately.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Raw records from the final step, as JSON values.
    pub records: Vec<serde_json::Value>,
    /// Name of the table spec that was fetched.
    pub table_name: String,
    /// Total number of API requests made (for rate limit tracking).
    pub request_count: u32,
    /// True if `records` was truncated at the 10K DI-019 limit (AC-8).
    pub truncated: bool,
}

/// Executes a multi-step fetch pipeline for a sensor table (BC-2.16.002).
pub struct PipelineExecutor;

impl PipelineExecutor {
    /// Execute all steps of a table's fetch pipeline sequentially.
    ///
    /// # Parameters
    ///
    /// - `spec` — The full sensor spec (base URL, auth_type, rate limits).
    /// - `table` — The table to fetch (its `steps` are executed in order).
    /// - `context` — Runtime context: client ID and query push-down filters.
    /// - `http_client` — Injected `reqwest::Client`; MUST NOT be a global singleton.
    ///   Tests inject a client whose traffic is directed at a wiremock mock server.
    /// - `auth_provider` — Injected `&dyn AuthProvider`; called to acquire/refresh
    ///   bearer tokens. Tests inject `MockAuthProvider`; production injects a
    ///   `CredentialStoreAuthProvider` (or `NullAuthProvider` placeholder).
    ///
    /// # Behaviour (S-PLUGIN-PREREQ-B implementation — body is todo!())
    ///
    /// - Steps run in spec-declared order (invariant: no parallel execution).
    /// - Variables from step N are available to steps N+1, N+2, ... but not prior.
    /// - Fan-out: if a variable resolves to an array, the step batches requests
    ///   (default batch_size = 100 per FetchStep configuration).
    /// - Rate limit hints apply between each API call.
    /// - The 10K materialization limit (DI-019) applies to the final collected records.
    /// - On HTTP 401: calls `auth_provider.acquire_token` once and retries ONCE.
    ///   If retry also returns 401, returns `SpecEngineError::AuthRefreshFailed`.
    ///
    /// # Errors
    ///
    /// Returns `PrismError` on HTTP failure, auth failure, JSONPath extraction
    /// failure, or interpolation failure.
    pub async fn execute(
        _spec: &SensorSpec,
        _table: &TableSpec,
        _context: &FetchContext,
        _http_client: &reqwest::Client,
        _auth_provider: &dyn AuthProvider,
    ) -> Result<PipelineResult, PrismError> {
        todo!("S-PLUGIN-PREREQ-B: HTTP execution per AC-1..AC-8 (BC-2.16.002)")
    }

    /// Execute a single fetch step, given resolved variables from prior steps.
    ///
    /// # Parameters
    ///
    /// - `step` — The fetch step to execute (method, path_template, pagination, etc.).
    /// - `spec` — Full sensor spec for base URL, auth type, rate limit hints.
    /// - `prior_vars` — Resolved variables from all previous steps
    ///   (keyed `"step_name.field"` per BC-2.16.002 interpolation semantics).
    /// - `context` — Runtime context: client ID and query push-down filters.
    /// - `http_client` — Injected HTTP client (same instance as `execute`).
    /// - `auth_provider` — Injected auth provider (same instance as `execute`).
    ///
    /// Returns the raw JSON response records extracted at `step.response_path`.
    /// Pagination is handled internally — all page records are concatenated before
    /// returning.
    ///
    /// # Errors
    ///
    /// Returns `PrismError` on HTTP failure or JSONPath extraction failure.
    pub async fn execute_step(
        _step: &FetchStep,
        _spec: &SensorSpec,
        _prior_vars: &std::collections::HashMap<String, serde_json::Value>,
        _context: &FetchContext,
        _http_client: &reqwest::Client,
        _auth_provider: &dyn AuthProvider,
    ) -> Result<serde_json::Value, PrismError> {
        todo!("S-PLUGIN-PREREQ-B: HTTP step execution per BC-2.16.002")
    }

    /// Resolve and expand fan-out: if a variable resolves to an array, return
    /// batches of `batch_size` items each (BC-2.16.002 Fan-Out Behavior).
    ///
    /// - Array input: batches of up to `batch_size` elements each.
    /// - Scalar input: single batch containing that one value.
    /// - Empty array: zero batches.
    ///
    /// This function is pure (no I/O) and unchanged from the prior stub.
    /// AC-6 mandates it is NOT duplicated in the HTTP execution path.
    pub fn fan_out_batches(
        values: &serde_json::Value,
        batch_size: usize,
    ) -> Vec<Vec<serde_json::Value>> {
        match values {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    return Vec::new();
                }
                arr.chunks(batch_size).map(|chunk| chunk.to_vec()).collect()
            }
            scalar => {
                // Non-array: single batch of one item.
                vec![vec![scalar.clone()]]
            }
        }
    }
}
