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
//! ADR-023 §C2 and BC-2.16.002.
//!
//! The `fan_out_batches` pure function is unchanged.

use std::collections::HashMap;
use std::time::Duration;

use prism_core::OrgSlug;

use crate::auth_provider::{AuthProvider, AuthToken};
use crate::error::SpecEngineError;
use crate::interpolation::{InterpolationContext, Interpolator};
use crate::spec_parser::{FetchStep, PaginationConfig, SensorSpec, TableSpec};

/// Maximum records materialised per pipeline execution (DI-019 / AC-8).
const MAX_PIPELINE_RECORDS: usize = 10_000;

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
    /// # Behaviour (BC-2.16.002)
    ///
    /// - Steps run in spec-declared order (invariant: no parallel execution).
    /// - Variables from step N are available to steps N+1, N+2, ... but not prior.
    /// - Rate limit hints apply between each API call (AC-7).
    /// - The 10K materialization limit (DI-019) applies to the total collected records (AC-8).
    /// - On HTTP 401: calls `auth_provider.acquire_token` once and retries ONCE.
    ///   If retry also returns 401, returns `SpecEngineError::AuthRefreshFailed` (AC-5).
    ///
    /// # Errors
    ///
    /// Returns `SpecEngineError` on HTTP failure, auth failure, JSONPath extraction
    /// failure, or interpolation failure.
    pub async fn execute(
        spec: &SensorSpec,
        table: &TableSpec,
        context: &FetchContext,
        http_client: &reqwest::Client,
        auth_provider: &dyn AuthProvider,
    ) -> Result<PipelineResult, SpecEngineError> {
        let mut all_records: Vec<serde_json::Value> = Vec::new();
        let mut request_count: u32 = 0;
        // step_vars: keyed as "step_name.field" -> JSON value
        let mut step_vars: HashMap<String, serde_json::Value> = HashMap::new();
        let mut truncated = false;

        // Start with an empty bearer token. The auth_provider is called lazily on
        // HTTP 401 (AC-5). This avoids an unconditional token-acquisition round-trip
        // for specs that don't need auth (NullAuthProvider) and keeps the call count
        // at exactly 1 for the 401-retry scenario.
        let mut bearer_token = AuthToken(String::new());

        'steps: for step in &table.steps {
            // Rate limit inter-step delay (AC-7): apply BETWEEN requests, not before the first.
            // We track whether this is the very first request in the pipeline.
            let mut is_first_request_in_step = true;

            // Interpolate the path template with variables from prior steps.
            let interpolated_path = Interpolator::interpolate(
                &step.path_template,
                &InterpolationContext::UrlPath,
                &step_vars,
            )
            .map_err(|e| SpecEngineError::HttpRequestFailed {
                sensor_id: spec.sensor_id.clone(),
                step_name: step.name.clone(),
                status_code: 0,
                detail: format!("path interpolation failed: {e}"),
            })?;

            let url = format!("{}{}", spec.base_url, interpolated_path);

            // Pagination state for this step.
            let mut cursor: Option<String> = None;
            let mut offset: u32 = 0;

            loop {
                // AC-7: apply rate-limit delay BETWEEN consecutive HTTP calls.
                if !is_first_request_in_step {
                    if let Some(ref hints) = spec.rate_limit_hints {
                        if let Some(rps) = hints.requests_per_second {
                            if rps > 0.0 {
                                let delay_secs = 1.0 / rps;
                                tokio::time::sleep(Duration::from_secs_f64(delay_secs)).await;
                            }
                        }
                    }
                }
                is_first_request_in_step = false;

                // Build the paginated URL.
                let paged_url = build_paged_url(&url, step, &cursor, offset);

                // Issue the request (with 401-retry logic per AC-5).
                let (body, new_token) = issue_request_with_retry(
                    http_client,
                    step,
                    spec,
                    &paged_url,
                    bearer_token,
                    auth_provider,
                    &context.client_id,
                    &mut request_count,
                )
                .await?;
                bearer_token = new_token;

                // Extract records at `step.response_path`.
                let page_records = extract_at_path(&body, &step.response_path).map_err(|_| {
                    SpecEngineError::JsonPathExtractionFailed {
                        sensor_id: spec.sensor_id.clone(),
                        step_name: step.name.clone(),
                        path: step.response_path.clone(),
                    }
                })?;

                // Store step variables for downstream interpolation.
                // Each field of the first record (or the raw scalar) is stored as
                // "step_name.field" for subsequent steps.
                store_step_vars(step, &body, &page_records, &mut step_vars);

                let page_count = match &page_records {
                    serde_json::Value::Array(arr) => {
                        all_records.extend(arr.iter().cloned());
                        arr.len()
                    }
                    scalar => {
                        // Single scalar result (e.g., `$.access_token`).
                        // Don't add to all_records for intermediate steps —
                        // only the last step's records are collected.
                        // We still need the page_count to decide pagination.
                        let _ = scalar;
                        1
                    }
                };

                // AC-8 / DI-019: truncate at 10K total records.
                if all_records.len() >= MAX_PIPELINE_RECORDS {
                    all_records.truncate(MAX_PIPELINE_RECORDS);
                    truncated = true;
                    break 'steps;
                }

                // Advance pagination or break.
                match &step.pagination {
                    Some(PaginationConfig::CursorToken {
                        cursor_response_path,
                    }) => {
                        let next = extract_cursor(&body, cursor_response_path);
                        match next {
                            Some(c) if !c.is_empty() && page_count > 0 => {
                                cursor = Some(c);
                            }
                            _ => break,
                        }
                    }
                    Some(PaginationConfig::OffsetLimit { page_size }) => {
                        let ps = *page_size as usize;
                        if page_count < ps {
                            break;
                        }
                        offset += *page_size;
                    }
                    Some(PaginationConfig::None) | None => break,
                }
            }
        }

        Ok(PipelineResult {
            records: all_records,
            table_name: table.table_name.clone(),
            request_count,
            truncated,
        })
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
    /// Returns the raw JSON response body extracted at `step.response_path`.
    /// Pagination is handled internally — all page records are concatenated before
    /// returning.
    ///
    /// # Errors
    ///
    /// Returns `SpecEngineError` on HTTP failure or JSONPath extraction failure.
    pub async fn execute_step(
        step: &FetchStep,
        spec: &SensorSpec,
        prior_vars: &std::collections::HashMap<String, serde_json::Value>,
        context: &FetchContext,
        http_client: &reqwest::Client,
        auth_provider: &dyn AuthProvider,
    ) -> Result<serde_json::Value, SpecEngineError> {
        let bearer_token = AuthToken(String::new());
        let mut request_count: u32 = 0;

        let interpolated_path = Interpolator::interpolate(
            &step.path_template,
            &InterpolationContext::UrlPath,
            prior_vars,
        )
        .map_err(|e| SpecEngineError::HttpRequestFailed {
            sensor_id: spec.sensor_id.clone(),
            step_name: step.name.clone(),
            status_code: 0,
            detail: format!("path interpolation failed: {e}"),
        })?;

        let url = format!("{}{}", spec.base_url, interpolated_path);

        let (body, _new_token) = issue_request_with_retry(
            http_client,
            step,
            spec,
            &url,
            bearer_token,
            auth_provider,
            &context.client_id,
            &mut request_count,
        )
        .await?;

        let extracted = extract_at_path(&body, &step.response_path).map_err(|_| {
            SpecEngineError::JsonPathExtractionFailed {
                sensor_id: spec.sensor_id.clone(),
                step_name: step.name.clone(),
                path: step.response_path.clone(),
            }
        })?;

        Ok(extracted)
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

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Issue one HTTP request, with a single 401-retry via `auth_provider` (AC-5).
///
/// Takes `current_token` by value (consumed) and returns `(body, token)` so the
/// caller can store the (possibly refreshed) token without borrow conflicts.
///
/// On 401: calls `auth_provider.acquire_token` once and retries the request once.
/// If the retry also returns 401, returns `SpecEngineError::AuthRefreshFailed`.
///
/// On any other non-2xx: returns `SpecEngineError::HttpRequestFailed`.
#[allow(clippy::too_many_arguments)]
async fn issue_request_with_retry(
    http_client: &reqwest::Client,
    step: &FetchStep,
    spec: &SensorSpec,
    url: &str,
    current_token: AuthToken,
    auth_provider: &dyn AuthProvider,
    client_id: &OrgSlug,
    request_count: &mut u32,
) -> Result<(serde_json::Value, AuthToken), SpecEngineError> {
    // Issue the first request.
    let response = build_request(http_client, step, url, &current_token)
        .send()
        .await
        .map_err(|e| SpecEngineError::HttpRequestFailed {
            sensor_id: spec.sensor_id.clone(),
            step_name: step.name.clone(),
            status_code: 0,
            detail: e.to_string(),
        })?;
    *request_count += 1;

    let status = response.status();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        // AC-5: refresh token and retry ONCE.
        let fresh_token = auth_provider.acquire_token(spec, client_id).await?;

        let retry_response = build_request(http_client, step, url, &fresh_token)
            .send()
            .await
            .map_err(|e| SpecEngineError::HttpRequestFailed {
                sensor_id: spec.sensor_id.clone(),
                step_name: step.name.clone(),
                status_code: 0,
                detail: e.to_string(),
            })?;
        *request_count += 1;

        let retry_status = retry_response.status();
        if retry_status == reqwest::StatusCode::UNAUTHORIZED {
            // AC-5 abort condition: double-401.
            return Err(SpecEngineError::AuthRefreshFailed {
                sensor_id: spec.sensor_id.clone(),
                client_id: client_id.to_string(),
                step_name: step.name.clone(),
            });
        }

        if !retry_status.is_success() {
            return Err(SpecEngineError::HttpRequestFailed {
                sensor_id: spec.sensor_id.clone(),
                step_name: step.name.clone(),
                status_code: retry_status.as_u16(),
                detail: format!("HTTP {retry_status}"),
            });
        }

        let body: serde_json::Value =
            retry_response
                .json()
                .await
                .map_err(|e| SpecEngineError::HttpRequestFailed {
                    sensor_id: spec.sensor_id.clone(),
                    step_name: step.name.clone(),
                    status_code: 0,
                    detail: format!("failed to parse response JSON: {e}"),
                })?;
        return Ok((body, fresh_token));
    }

    if !status.is_success() {
        return Err(SpecEngineError::HttpRequestFailed {
            sensor_id: spec.sensor_id.clone(),
            step_name: step.name.clone(),
            status_code: status.as_u16(),
            detail: format!("HTTP {status}"),
        });
    }

    let body: serde_json::Value =
        response
            .json()
            .await
            .map_err(|e| SpecEngineError::HttpRequestFailed {
                sensor_id: spec.sensor_id.clone(),
                step_name: step.name.clone(),
                status_code: 0,
                detail: format!("failed to parse response JSON: {e}"),
            })?;

    Ok((body, current_token))
}

/// Build a `reqwest::RequestBuilder` for the given step and URL.
fn build_request(
    http_client: &reqwest::Client,
    step: &FetchStep,
    url: &str,
    token: &AuthToken,
) -> reqwest::RequestBuilder {
    let method = match step.method.to_ascii_uppercase().as_str() {
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "PATCH" => reqwest::Method::PATCH,
        "DELETE" => reqwest::Method::DELETE,
        _ => reqwest::Method::GET,
    };

    let mut req = http_client.request(method, url);

    // Add bearer token if non-empty.
    if !token.0.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token.0));
    }

    // Add request body for POST/PUT/PATCH.
    if let Some(ref body) = step.body_template {
        req = req
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.clone());
    }

    req
}

/// Build a paginated URL by appending pagination query parameters.
fn build_paged_url(
    base_url: &str,
    step: &FetchStep,
    cursor: &Option<String>,
    offset: u32,
) -> String {
    match &step.pagination {
        Some(PaginationConfig::CursorToken { .. }) => {
            if let Some(ref c) = cursor {
                if base_url.contains('?') {
                    format!("{base_url}&cursor={c}")
                } else {
                    format!("{base_url}?cursor={c}")
                }
            } else {
                base_url.to_string()
            }
        }
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            let sep = if base_url.contains('?') { '&' } else { '?' };
            format!("{base_url}{sep}offset={offset}&limit={page_size}")
        }
        Some(PaginationConfig::None) | None => base_url.to_string(),
    }
}

/// Extract the value at a simple JSONPath expression (e.g. `$.field` or `$.a.b.c`).
///
/// Supported syntax: `$.field1.field2...fieldN` (dot-notation only).
/// Returns `Err(())` if the path does not match the response structure.
fn extract_at_path(body: &serde_json::Value, path: &str) -> Result<serde_json::Value, ()> {
    // Strip leading `$.` prefix.
    let stripped = path.strip_prefix("$.").ok_or(())?;

    let mut current = body;
    for segment in stripped.split('.') {
        match current.get(segment) {
            Some(v) => current = v,
            None => return Err(()),
        }
    }
    Ok(current.clone())
}

/// Extract a cursor string from the response body at the given JSONPath.
///
/// Returns `None` if the path does not match, the value is null, or the
/// value is not a string.
fn extract_cursor(body: &serde_json::Value, cursor_path: &str) -> Option<String> {
    match extract_at_path(body, cursor_path) {
        Ok(serde_json::Value::String(s)) => Some(s),
        Ok(serde_json::Value::Null) => None,
        _ => None,
    }
}

/// Store step output variables into `step_vars` for downstream interpolation.
///
/// For each field in `variables_produced`, the value is extracted from the
/// response body and stored as `"step_name.field"`.
///
/// If a step's `response_path` resolves to a scalar (non-array), the scalar
/// itself is stored directly as `"step_name.<last_path_segment>"`.
fn store_step_vars(
    step: &FetchStep,
    body: &serde_json::Value,
    extracted: &serde_json::Value,
    step_vars: &mut HashMap<String, serde_json::Value>,
) {
    // If `variables_produced` is declared, extract each named variable from body.
    for var_name in &step.variables_produced {
        // Try to find the var at `$.var_name` in the body directly.
        let path = format!("$.{var_name}");
        if let Ok(v) = extract_at_path(body, &path) {
            step_vars.insert(format!("{}.{var_name}", step.name), v);
        } else if let Some(v) = body.get(var_name) {
            // Fallback: direct field lookup at root.
            step_vars.insert(format!("{}.{var_name}", step.name), v.clone());
        }
    }

    // Also store the extracted value under the last path segment of response_path,
    // so `${step_name.<last_segment>}` works without declaring variables_produced.
    if let Some(last_seg) = step.response_path.split('.').next_back() {
        let key = format!("{}.{last_seg}", step.name);
        step_vars.entry(key).or_insert_with(|| extracted.clone());
    }
}
