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

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use prism_core::OrgSlug;

use crate::auth_provider::{AuthProvider, AuthToken};
use crate::error::SpecEngineError;
use crate::interpolation::{InterpolationContext, Interpolator};
use crate::spec_parser::{FetchStep, PaginationConfig, SensorSpec, TableSpec};

/// Maximum records materialised per pipeline execution (DI-019 / AC-8).
const MAX_PIPELINE_RECORDS: usize = 10_000;

/// Maximum pages fetched per step to guard against infinite pagination loops
/// caused by APIs that fail to advance cursors or that emit perpetual data.
///
/// F-LP2-HIGH-002 defense: if a step exceeds this page count, the pipeline
/// aborts with `SpecEngineError::HttpRequestFailed` (detail includes step name
/// and page limit). Full resource bound (MAX_REQUESTS_PER_PIPELINE) deferred to
/// TD-S-PLUGIN-PREREQ-B-004 P3.
const MAX_PAGES_PER_STEP: usize = 1_000;

/// Context provided to each pipeline execution.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FetchContext {
    /// The client/tenant this query is executing for.
    pub client_id: OrgSlug,
    /// Push-down filter values from the query planner (${query.filter.*}).
    pub query_filters: std::collections::HashMap<String, String>,
}

impl FetchContext {
    /// Construct a `FetchContext`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct-literal construction
    /// outside the crate. External callers (tests, integration code) MUST use this.
    pub fn new(
        client_id: OrgSlug,
        query_filters: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            client_id,
            query_filters,
        }
    }
}

/// The output of a successful pipeline execution.
///
/// Contains the raw JSON records from the final step. OCSF mapping (BC-2.16.003)
/// is applied by `ColumnMapper` separately.
#[non_exhaustive]
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
    ///   **TD-S-PLUGIN-PREREQ-B-005 P2:** Production callers (boot.rs / chassis) MUST
    ///   construct this client with a configurable timeout (default 30s) using
    ///   `reqwest::Client::builder().timeout(Duration::from_secs(30)).build()`.
    ///   Test fixtures already use this pattern (F-LP4-MED-001).
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

        // AC-7 (F-LP1-HIGH-002): rate-limit flag is pipeline-scoped, not step-scoped.
        // Hoisted OUTSIDE the steps loop so the delay applies between ALL API calls
        // across step boundaries, not just within a single step.
        let mut is_first_pipeline_request = true;

        // Eager token acquisition: acquire_token is called BEFORE the steps loop
        // (F-LP5-LOW-003 closure). AuthType has no Null variant — all 4 variants
        // (Oauth2ClientCredentials, BearerStatic, CookieRoundtrip, ApiKey) require auth.
        // NullAuthProvider (test-only) returns an empty token without I/O.
        //
        // TD-S-PLUGIN-PREREQ-B-010 CLOSED: lazy-token-on-401 design replaced by eager
        // acquisition. The auth_refresh_triggered event now fires ONLY on legitimate
        // token-expiry mid-pipeline (not on every first request). Orchestrator authorized
        // Option A (eager unconditional) on 2026-05-11.
        let mut bearer_token = match auth_provider.acquire_token(spec, &context.client_id).await {
            Ok(tok) if !tok.as_str().is_empty() => {
                tracing::info!(
                    event_type = "auth_initial_acquired",
                    sensor_id = %spec.sensor_id,
                    client_id = %context.client_id,
                    "auth token acquired (eager)",
                );
                tok
            }
            Ok(tok) => {
                // Empty token — typically NullAuthProvider (test-only) or buggy production provider.
                // Emit debug log rather than info to keep production audit signal clean.
                tracing::debug!(
                    event_type = "auth_initial_acquired_empty",
                    sensor_id = %spec.sensor_id,
                    client_id = %context.client_id,
                    "auth_provider returned empty token (NullAuth test path or provider bug)",
                );
                tok
            }
            Err(e) => {
                tracing::error!(
                    event_type = "auth_initial_failed",
                    sensor_id = %spec.sensor_id,
                    client_id = %context.client_id,
                    detail = %e,
                    "auth token acquisition failed at pipeline start",
                );
                return Err(e);
            }
        };

        // F-LP1-HIGH-004: seed step_vars with query context so ${query.filter.*}
        // and ${query.client_id} are available for interpolation in all steps.
        step_vars.insert(
            "query.client_id".to_string(),
            serde_json::Value::String(context.client_id.to_string()),
        );
        for (k, v) in &context.query_filters {
            step_vars.insert(
                format!("query.filter.{k}"),
                serde_json::Value::String(v.clone()),
            );
        }

        let step_count = table.steps.len();

        'steps: for (step_idx, step) in table.steps.iter().enumerate() {
            let is_final_step = step_idx == step_count - 1;

            // AC-6 (F-LP1-HIGH-001): fan-out — if any variable in the step's
            // path_template or body_template resolves to an array from a prior step,
            // execute the step once per batch.
            //
            // F-LP2-HIGH-001 fix: `find_fan_out_array` now returns (key, value) so
            // the fan-out loop can override the source key with each batch slice.
            // Previously, only `{step.name}.batch` was inserted, but the template
            // still referenced `${step1.ids}` (the full 250-element array), causing
            // every fan-out iteration to send the same payload — a paper-fix regression
            // introduced in fix-burst-1.
            let fan_out = find_fan_out_array(step, &step_vars);
            let batch_size = step.fan_out_batch_size.map(|s| s as usize).unwrap_or(100); // AC-6 default per spec

            // Build batches (or a single pass if no fan-out)
            let batches: Vec<Option<(String, Vec<serde_json::Value>)>> =
                if let Some((source_key, ref arr)) = fan_out {
                    // Fan-out: one batch per chunk; carry the source key so we can
                    // override step_vars[source_key] with the current batch slice.
                    Self::fan_out_batches(arr, batch_size)
                        .into_iter()
                        .map(|b| Some((source_key.clone(), b)))
                        .collect()
                } else {
                    // No fan-out: single pass
                    vec![None]
                };

            for batch in batches {
                // Build per-batch step_vars: override the source array key with the
                // current batch slice so that template interpolation receives only the
                // batch items, not the full prior-step array.
                let mut batch_step_vars = step_vars.clone();
                if let Some((ref source_key, ref batch_items)) = batch {
                    // Override the source key (e.g. "step1.ids") with the current
                    // batch slice.  This ensures ${step1.ids} in the template resolves
                    // to this batch's items, not the full 250-element array.
                    batch_step_vars.insert(
                        source_key.clone(),
                        serde_json::Value::Array(batch_items.clone()),
                    );
                    // Also inject under the synthetic {this_step}.batch key for
                    // templates that prefer the explicit batch reference.
                    batch_step_vars.insert(
                        format!("{}.batch", step.name),
                        serde_json::Value::Array(batch_items.clone()),
                    );
                }

                // Interpolate the path template with variables from prior steps.
                //
                // TD-S-PLUGIN-PREREQ-B-007 P3: HttpRequestFailed.status_code = 0 is overloaded
                // across 11 distinct origins (interpolation, network, JSON parse, page-cap,
                // cursor non-advance). Future error-classification refactor should add an origin
                // discriminator field to SpecEngineError. Per F-LP5-LOW-004.
                let interpolated_path = Interpolator::interpolate(
                    &step.path_template,
                    &InterpolationContext::UrlPath,
                    &batch_step_vars,
                )
                .map_err(|e| SpecEngineError::HttpRequestFailed {
                    sensor_id: spec.sensor_id.clone(),
                    step_name: step.name.clone(),
                    status_code: 0,
                    detail: format!("path interpolation failed: {e}"),
                })?;

                let url = format!("{}{}", spec.base_url, interpolated_path);

                // Pagination state for this step/batch.
                let mut cursor: Option<String> = None;
                let mut prev_cursor: Option<String> = None; // F-LP2-HIGH-002: cursor non-advance guard
                let mut offset: u32 = 0;
                let mut page_count: usize = 0; // F-LP2-HIGH-002: MAX_PAGES_PER_STEP guard

                loop {
                    // F-LP2-HIGH-002: abort if step has exceeded the page cap.
                    if page_count >= MAX_PAGES_PER_STEP {
                        return Err(SpecEngineError::HttpRequestFailed {
                            sensor_id: spec.sensor_id.clone(),
                            step_name: step.name.clone(),
                            status_code: 0,
                            detail: format!(
                                "step '{}' exceeded {MAX_PAGES_PER_STEP} pages — \
                                 likely API misbehavior or cursor non-advancement",
                                step.name
                            ),
                        });
                    }
                    page_count += 1;
                    // AC-7: apply rate-limit delay BETWEEN consecutive HTTP calls.
                    // is_first_pipeline_request is pipeline-scoped (F-LP1-HIGH-002 fix).
                    if !is_first_pipeline_request {
                        let rps_opt = spec
                            .rate_limit_hints
                            .as_ref()
                            .and_then(|h| h.requests_per_second)
                            .filter(|&r| r > 0.0);
                        if let Some(rps) = rps_opt {
                            // Cap at 1 hour (3600s) to prevent Duration overflow when
                            // rps is pathologically small (F-LP4-LOW-003 overflow guard).
                            let delay_secs = (1.0 / rps).min(3600.0);
                            tokio::time::sleep(Duration::from_secs_f64(delay_secs)).await;
                        }
                    }
                    is_first_pipeline_request = false;

                    // F-LP1-CRIT-002: cursor must be percent-encoded before appending to URL.
                    // F-LP2-LOW-003: `percent_encoding` imports are hoisted to module top.
                    let encoded_cursor = cursor
                        .as_deref()
                        .map(|c| utf8_percent_encode(c, NON_ALPHANUMERIC).to_string());

                    // Build the paginated URL with encoded cursor.
                    let paged_url = build_paged_url(&url, step, &encoded_cursor, offset);

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
                        &batch_step_vars,
                    )
                    .await?;
                    bearer_token = new_token;

                    // Extract records at `step.response_path`.
                    let page_records =
                        extract_at_path(&body, &step.response_path).map_err(|_| {
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

                    // F-LP1-CRIT-003: only accumulate records for the FINAL step.
                    // Intermediate step records (e.g., OAuth tokens) must not appear
                    // in the pipeline result.
                    //
                    // `page_record_count` is the number of records returned in this
                    // single page response; used by the pagination-advance logic below.
                    let page_record_count = match &page_records {
                        serde_json::Value::Array(arr) => {
                            if is_final_step {
                                all_records.extend(arr.iter().cloned());
                            }
                            arr.len()
                        }
                        scalar => {
                            // Single scalar result (e.g., `$.access_token`).
                            // Never added to all_records regardless of step position.
                            let _ = scalar;
                            1
                        }
                    };

                    // AC-8 / DI-019: truncate at 10K total records.
                    if all_records.len() >= MAX_PIPELINE_RECORDS {
                        tracing::warn!(
                            event_type = "pipeline_truncated",
                            sensor_id = %spec.sensor_id,
                            client_id = %context.client_id,
                            step_name = %step.name,
                            max_records = MAX_PIPELINE_RECORDS,
                            accumulated = all_records.len(),
                            "DI-019 cap reached — records truncated to 10K",
                        );
                        all_records.truncate(MAX_PIPELINE_RECORDS);
                        truncated = true;
                        break 'steps;
                    }

                    // Advance pagination or break.
                    // Cursor read from raw body (before encoding); stored raw for
                    // next iteration where it will be encoded by build_paged_url.
                    match &step.pagination {
                        Some(PaginationConfig::CursorToken {
                            cursor_response_path,
                        }) => {
                            let next = extract_cursor(&body, cursor_response_path);
                            match next {
                                Some(c) if !c.is_empty() && page_record_count > 0 => {
                                    // F-LP2-HIGH-002: cursor non-advance guard.
                                    // If the API returns the same cursor AND non-empty data,
                                    // the pagination loop would run forever.
                                    if prev_cursor.as_deref() == Some(c.as_str()) {
                                        return Err(SpecEngineError::HttpRequestFailed {
                                            sensor_id: spec.sensor_id.clone(),
                                            step_name: step.name.clone(),
                                            status_code: 0,
                                            detail: "pagination cursor did not advance".to_string(),
                                        });
                                    }
                                    prev_cursor = Some(c.clone());
                                    cursor = Some(c);
                                }
                                _ => break,
                            }
                        }
                        Some(PaginationConfig::OffsetLimit { page_size }) => {
                            let ps = *page_size as usize;
                            if page_record_count < ps {
                                break;
                            }
                            offset += *page_size;
                        }
                        Some(PaginationConfig::None) | None => break,
                    }
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

    /// Execute a single fetch step against the resolved variables — issues ONE HTTP
    /// request without pagination, fan-out, rate-limit, or truncation.
    ///
    /// This helper is intended for plugin-runtime contexts that have pre-resolved a
    /// step's state and want to delegate the single HTTP issue to the shared executor.
    ///
    /// **Pagination, fan-out, 10K truncation (DI-019), rate-limit delays, and auth
    /// refresh are NOT performed here.** Use [`PipelineExecutor::execute`] for those
    /// semantics (BC-2.16.002 full pipeline).
    ///
    /// ## Testing
    ///
    /// This helper is intentionally untested at the PREREQ-B integration test layer
    /// because it has no PREREQ-B callers (per story §94-96 deferral to Wave 1).
    /// PREREQ-D wiring will add the test vehicle. See TD-S-PLUGIN-PREREQ-B-012 P3.
    ///
    /// # Parameters
    ///
    /// - `step` — The fetch step to execute (method, path_template, etc.).
    /// - `spec` — Full sensor spec for base URL, auth type.
    /// - `prior_vars` — Resolved variables from all previous steps
    ///   (keyed `"step_name.field"` per BC-2.16.002 interpolation semantics).
    /// - `context` — Runtime context: client ID and query push-down filters.
    /// - `http_client` — Injected HTTP client (same instance as `execute`).
    /// - `auth_provider` — Injected auth provider (same instance as `execute`).
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
        // Eager token acquisition: symmetric with PipelineExecutor::execute (BC-2.16.002 — see Structured Event Catalog).
        // Ensures consistent audit signal when plugin-runtime calls execute_step directly
        // (PREREQ-D wiring scope). On acquisition failure the call is aborted immediately,
        // matching the execute() contract. If the step's HTTP request returns 401, the
        // issue_request_with_retry helper calls acquire_token again as a refresh.
        let bearer_token = match auth_provider.acquire_token(spec, &context.client_id).await {
            Ok(tok) if !tok.as_str().is_empty() => {
                tracing::info!(
                    event_type = "auth_initial_acquired",
                    sensor_id = %spec.sensor_id,
                    client_id = %context.client_id,
                    step_name = %step.name,
                    "execute_step: auth token acquired (eager)",
                );
                tok
            }
            Ok(tok) => {
                // Empty token — typically NullAuthProvider (test-only) or buggy production provider.
                // Emit debug log rather than info to keep production audit signal clean.
                tracing::debug!(
                    event_type = "auth_initial_acquired_empty",
                    sensor_id = %spec.sensor_id,
                    client_id = %context.client_id,
                    step_name = %step.name,
                    "execute_step: auth_provider returned empty token (NullAuth test path or provider bug)",
                );
                tok
            }
            Err(e) => {
                tracing::error!(
                    event_type = "auth_initial_failed",
                    sensor_id = %spec.sensor_id,
                    client_id = %context.client_id,
                    step_name = %step.name,
                    detail = %e,
                    "execute_step: auth token acquisition failed",
                );
                return Err(e);
            }
        };
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
            prior_vars,
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
    /// TD-S-PLUGIN-PREREQ-B-006 P2: pure functions (fan_out_batches, extract_at_path,
    /// Interpolator::interpolate) lack proptest coverage. PREREQ-A established
    /// cross-crate validator-parity proptest precedent. PREREQ-C scope.
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
        // Defense-in-depth: clamp to 1 so chunks(0) can never panic even if the
        // caller bypasses validation. Callers SHOULD validate before reaching here
        // (F-LP4-HIGH-001 validation in validation.rs is the primary guard).
        let batch_size = batch_size.max(1);
        match values {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    return Vec::new();
                }
                arr.chunks(batch_size).map(|chunk| chunk.to_vec()).collect()
            }
            scalar => {
                // Non-array: single batch of one item.
                // TD-S-PLUGIN-PREREQ-B-009 P3: this scalar arm is unreachable from
                // production callers (find_fan_out_array filters on .is_array()). Either
                // delete with unreachable!() or add regression test documenting the
                // external-caller contract. Per F-LP5-OBS-001.
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
/// On 401: calls `auth_provider.acquire_token` once, logs the event (AC-5 audit),
/// and retries the request once.
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
    step_vars: &HashMap<String, serde_json::Value>,
) -> Result<(serde_json::Value, AuthToken), SpecEngineError> {
    // Issue the first request.
    let response = build_request(http_client, step, url, &current_token, step_vars)
        .map_err(|e| SpecEngineError::HttpRequestFailed {
            sensor_id: spec.sensor_id.clone(),
            step_name: step.name.clone(),
            status_code: 0,
            detail: format!("body interpolation failed: {e}"),
        })?
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
        // F-LP1-HIGH-003 (AC-5 audit): log auth refresh event. Token value is NEVER logged.
        tracing::warn!(
            event_type = "auth_refresh_triggered",
            sensor_id = %spec.sensor_id,
            client_id = %client_id,
            step_name = %step.name,
            "auth refresh triggered by 401 response"
        );

        // AC-5: refresh token and retry ONCE.
        let fresh_token = match auth_provider.acquire_token(spec, client_id).await {
            Ok(tok) => {
                tracing::info!(
                    event_type = "auth_refresh_succeeded",
                    sensor_id = %spec.sensor_id,
                    client_id = %client_id,
                    step_name = %step.name,
                    "auth refresh acquired fresh token",
                );
                tok
            }
            Err(e) => {
                tracing::error!(
                    event_type = "auth_refresh_failed",
                    sensor_id = %spec.sensor_id,
                    client_id = %client_id,
                    step_name = %step.name,
                    detail = %e,
                    "auth refresh acquire_token failed",
                );
                return Err(e);
            }
        };

        let retry_response = build_request(http_client, step, url, &fresh_token, step_vars)
            .map_err(|e| SpecEngineError::HttpRequestFailed {
                sensor_id: spec.sensor_id.clone(),
                step_name: step.name.clone(),
                status_code: 0,
                detail: format!("body interpolation failed on retry: {e}"),
            })?
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
            tracing::error!(
                event_type = "auth_refresh_double_401",
                sensor_id = %spec.sensor_id,
                client_id = %client_id,
                step_name = %step.name,
                "auth refresh resulted in second 401 — aborting pipeline",
            );
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
///
/// F-LP1-CRIT-001: body_template is interpolated against `step_vars` before sending.
/// Content-Type is derived from body shape:
///   - JSON object (`{...}`) → `application/json`
///   - Otherwise → `application/x-www-form-urlencoded`
fn build_request(
    http_client: &reqwest::Client,
    step: &FetchStep,
    url: &str,
    token: &AuthToken,
    step_vars: &HashMap<String, serde_json::Value>,
) -> Result<reqwest::RequestBuilder, String> {
    let method = match step.method.to_ascii_uppercase().as_str() {
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "PATCH" => reqwest::Method::PATCH,
        "DELETE" => reqwest::Method::DELETE,
        _ => reqwest::Method::GET,
    };

    let mut req = http_client.request(method, url);

    // Add bearer token if non-empty.
    if !token.as_str().is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token.as_str()));
    }

    // F-LP1-CRIT-001: Add request body for POST/PUT/PATCH.
    // Interpolate body_template against step_vars and derive Content-Type from shape.
    //
    // TD-S-PLUGIN-PREREQ-B-008 P3: Interpolator grammar has no escape mechanism for
    // literal ${...}. Spec authors cannot send documentation strings containing template
    // syntax. PREREQ-C scope: add $${...} or \${...} escape convention. Per F-LP5-LOW-005.
    if let Some(ref body_tpl) = step.body_template {
        let interpolated_body =
            Interpolator::interpolate(body_tpl, &InterpolationContext::JsonBody, step_vars)
                .map_err(|e| format!("body template interpolation failed: {e}"))?;

        // Derive Content-Type: JSON if body starts with '{' or '[', else form-urlencoded.
        // F-LP2-MED-002: JSON arrays (starting with '[') are also application/json.
        let trimmed = interpolated_body.trim_start();
        let content_type = if trimmed.starts_with('{') || trimmed.starts_with('[') {
            "application/json"
        } else {
            "application/x-www-form-urlencoded"
        };

        req = req
            .header("Content-Type", content_type)
            .body(interpolated_body);
    }

    Ok(req)
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
            // TD-S-PLUGIN-PREREQ-B-001 P2: cursor pagination first-call does not include page_size param.
            // Real-world APIs (CrowdStrike GraphQL cursor) require first: N on every request. PREREQ-C scope:
            // add `page_size: Option<u32>` field to PaginationConfig::CursorToken.
            if let Some(c) = cursor {
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
/// Internally converts to a JSON Pointer (RFC 6901) and delegates to
/// `serde_json::Value::pointer` so nested key lookup is unambiguous.
///
/// F-LP2-LOW-002: RFC 6901 key-content escaping is applied to each dot-separated
/// segment before joining with '/'. The escape rules are:
///
/// - `~` → `~0`  (must be applied BEFORE the `/` escape to avoid double-escape)
/// - `/` → `~1`
///
/// This handles JSON keys that contain literal `~` or `/` characters.
///
/// TD-S-PLUGIN-PREREQ-B-003 P3: JSON Pointer dot-notation only. Bracket notation
/// ($.x[0]) and wildcards ($.x[*]) deferred to PREREQ-C scope (per fix-burst-1 OBS).
///
/// Returns `Err(String)` with a descriptive message if the path does not match.
fn extract_at_path(body: &serde_json::Value, path: &str) -> Result<serde_json::Value, String> {
    let stripped = path
        .strip_prefix("$.")
        .ok_or_else(|| format!("path must start with '$.' : {path}"))?;
    // F-LP5-LOW-001: reject "$." with no key segment — this is a malformed path
    // that would produce an empty JSON Pointer "/" matching the root, not a key.
    if stripped.is_empty() {
        return Err(format!(
            "response_path '{path}' must contain at least one key segment after '$.'",
        ));
    }
    // Each dot-separated segment is a JSON key; escape ~ and / per RFC 6901
    // before joining with '/' as the JSON Pointer segment separator.
    let segments: Vec<String> = stripped
        .split('.')
        .map(|seg| seg.replace('~', "~0").replace('/', "~1"))
        .collect();
    let pointer_path = format!("/{}", segments.join("/"));
    body.pointer(&pointer_path)
        .cloned()
        .ok_or_else(|| format!("path not found: {path}"))
}

/// Extract a cursor string from the response body at the given JSONPath.
///
/// F-LP2-MED-003: Numeric cursors are coerced to their string representation
/// so that APIs returning `{"cursor": 42}` correctly advance pagination.
/// Object/Array/Bool cursor values are treated as terminal and logged as a
/// diagnostic warning. Empty strings are terminal (no next page).
fn extract_cursor(body: &serde_json::Value, cursor_path: &str) -> Option<String> {
    match extract_at_path(body, cursor_path).ok()? {
        serde_json::Value::String(s) if !s.is_empty() => Some(s),
        serde_json::Value::String(_) => None, // empty string = terminal
        serde_json::Value::Number(n) => Some(n.to_string()), // numeric cursor → string
        serde_json::Value::Null => None,
        other => {
            // Object/Array/Bool: treat as terminal but emit structured diagnostic.
            // F-LP8-MED-003: include event_type for SIEM/SOC alerting pipelines.
            // The bare-warn without event_type was inconsistent with the project's
            // audit-signal discipline (compare pipeline_truncated at pipeline.rs:362-370).
            let actual_type = match &other {
                serde_json::Value::Array(_) => "Array",
                serde_json::Value::Object(_) => "Object",
                serde_json::Value::Bool(_) => "Bool",
                _ => "Unknown",
            };
            // OBS-LP9-003: use char_indices for char-boundary-safe truncation.
            // `s.len()` is BYTES; byte-index slicing panics on multi-byte UTF-8.
            // `char_indices().nth(100)` gives the byte index of the 100th codepoint.
            let cursor_preview = {
                let s = other.to_string();
                match s.char_indices().nth(100) {
                    Some((idx, _)) => format!("{}...", &s[..idx]),
                    None => s,
                }
            };
            tracing::warn!(
                event_type = "pagination_cursor_unsupported_type",
                cursor_path = %cursor_path,
                actual_type = %actual_type,
                cursor_preview = %cursor_preview,
                "Cursor pagination terminated: cursor resolved to unsupported type \
                 (only String, Number, Null are supported)"
            );
            None
        }
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

/// Detect whether any variable referenced in `step`'s templates resolves to an
/// array in `step_vars`. Returns `(source_key, array_value)` if fan-out applies.
///
/// Fan-out is triggered when a step variable reference (${step_name.field}) resolves
/// to a JSON array. The first such array found is used as the fan-out source.
/// Non-array variables are not considered for fan-out.
///
/// F-LP2-HIGH-001: The source key is returned alongside the array so the caller
/// can override `step_vars[source_key]` with each batch slice during iteration,
/// ensuring the template receives the batch items rather than the full array.
///
/// AC-6: `fan_out_batch_size` field on `FetchStep` controls batch size (default 100).
fn find_fan_out_array(
    step: &FetchStep,
    step_vars: &HashMap<String, serde_json::Value>,
) -> Option<(String, serde_json::Value)> {
    // Collect ALL array-valued variables referenced in path_template and body_template.
    // F-LP9-MED-002: must iterate all templates to detect multi-array ambiguity.
    let templates: Vec<&str> = std::iter::once(step.path_template.as_str())
        .chain(step.body_template.as_deref())
        .collect();

    let mut array_vars: Vec<(String, serde_json::Value)> = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();

    for template in &templates {
        let refs = crate::interpolation::Interpolator::extract_references(template);
        for (step_name, field_path) in refs {
            let key = format!("{step_name}.{field_path}");
            if seen_keys.contains(&key) {
                continue; // dedup: same var referenced in multiple templates
            }
            if let Some(val) = step_vars.get(&key).filter(|v| v.is_array()) {
                seen_keys.insert(key.clone());
                array_vars.push((key, val.clone()));
            }
        }
    }

    // F-LP10-MED-002: After collecting array vars, check for Object-typed variables that
    // are referenced in templates but were NOT classified as fan-out source (they are
    // Objects, not Arrays). Object values passed through `value_to_string` are silently
    // stringified as JSON into the URL/body — likely a spec bug. Emit a structured warn
    // per offending reference so operators can detect this ambiguity.
    for template in &templates {
        let refs = crate::interpolation::Interpolator::extract_references(template);
        for (ref_step_name, ref_field_path) in refs {
            let key = format!("{ref_step_name}.{ref_field_path}");
            if let Some(value) = step_vars.get(&key)
                && value.is_object()
            {
                tracing::warn!(
                    event_type = "fanout_invalid_source_type",
                    step_name = %step.name,
                    var_name = %key,
                    actual_type = "Object",
                    "Step references an Object-valued variable in a template; will be \
                     stringified into URL/body. This is likely a spec bug — consider \
                     referencing a scalar field (${{var.field}}) instead."
                );
            }
        }
    }

    match array_vars.len() {
        0 => None, // no fan-out
        1 => {
            // Exactly one array — normal fan-out, no ambiguity.
            array_vars.into_iter().next()
        }
        _ => {
            // F-LP9-MED-002: multiple array-valued variables → ambiguous fan-out semantics.
            // Emit structured warn so operators/SIEM can detect this case.
            // Future PREREQ-C/D may define cartesian or zipped fan-out.
            let first_var_name = array_vars[0].0.clone();
            let other_var_names: Vec<&str> =
                array_vars[1..].iter().map(|(k, _)| k.as_str()).collect();
            tracing::warn!(
                event_type = "fanout_ambiguous_multi_array",
                step_name = %step.name,
                array_vars_count = array_vars.len(),
                first_var = %first_var_name,
                other_vars = ?other_var_names,
                "Step references multiple array-valued variables; fan-out semantics ambiguous \
                 (only first array drives batching). Future PREREQ-C/D may define cartesian \
                 or zipped fan-out."
            );
            // Preserve current behavior: first array drives fan-out.
            array_vars.into_iter().next()
        }
    }
}

// ---------------------------------------------------------------------------
// F-LP12-MED-001: execute_step unit tests — pre-emptive anchoring for BC v1.8 rows 4/5/6
//
// BC-2.16.002 Structured Event Catalog rows 4/5/6 document three events emitted by
// PipelineExecutor::execute_step during eager auth token acquisition. These tests
// anchor the field-schema for those rows so that any future refactor that removes or
// renames `step_name` (or other fields) from the tracing macros causes a test failure.
//
// RED GATE verified against HEAD 6e436d65: these tests did not exist, so they could not
// pass. Adding them with field-schema assertions converts the 3 contract-only rows into
// 3 test-anchored rows.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod execute_step_tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use prism_core::OrgSlug;
    use tracing_subscriber::util::SubscriberInitExt;
    use wiremock::matchers::{method as wm_method, path as wm_path};
    use wiremock::{Mock as WmMock, MockServer, ResponseTemplate};

    use crate::auth_provider::{FailingAuthProvider, MockAuthProvider, NullAuthProvider};
    use crate::pipeline::{FetchContext, PipelineExecutor};
    use crate::spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec};
    use prism_core::ColumnType;

    // ---------------------------------------------------------------------------
    // Log-capture helper — returns the buffer + a DefaultGuard that installs
    // a tracing subscriber for the current thread.
    // Matches the pattern used in pipeline_http_integration.rs tests.
    // ---------------------------------------------------------------------------

    fn setup_log_capture() -> (Arc<Mutex<String>>, tracing::dispatcher::DefaultGuard) {
        let buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let buf_clone = buf.clone();
        let writer = tracing_subscriber::fmt::writer::BoxMakeWriter::new(move || {
            struct BufWriter(Arc<Mutex<String>>);
            impl std::io::Write for BufWriter {
                fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                    self.0.lock().unwrap().push_str(&String::from_utf8_lossy(b));
                    Ok(b.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }
            BufWriter(buf_clone.clone())
        });
        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer)
            .with_max_level(tracing::Level::TRACE)
            .finish();
        let guard = subscriber.set_default();
        (buf, guard)
    }

    fn make_single_step_spec(base_url: &str, step_name: &str) -> SensorSpec {
        SensorSpec {
            sensor_id: "execute-step-test-sensor".to_string(),
            name: "Execute Step Test Sensor".to_string(),
            auth_type: AuthType::BearerStatic,
            base_url: base_url.to_string(),
            tables: vec![TableSpec::new_point_in_time(
                "items",
                "security_finding",
                vec![ColumnSpec {
                    name: "id".to_string(),
                    column_type: ColumnType::String,
                    ocsf_field: None,
                    options: vec![],
                }],
                vec![FetchStep {
                    name: step_name.to_string(),
                    method: "GET".to_string(),
                    path_template: "/items".to_string(),
                    body_template: None,
                    response_path: "$.items".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                }],
            )],
            rate_limit_hints: None,
            version: "1.0.0".to_string(),
            credential_refs: Vec::new(),
        }
    }

    // ---------------------------------------------------------------------------
    // Test 1: BC row 4 — execute_step / auth_initial_acquired / fields: sensor_id,
    // client_id, step_name
    // ---------------------------------------------------------------------------

    /// BC-2.16.002 Structured Event Catalog row 4:
    /// `execute_step` with a non-empty token emits `event_type = "auth_initial_acquired"`
    /// at INFO level with fields `sensor_id`, `client_id`, and `step_name`.
    ///
    /// RED GATE: Before this test existed there were ZERO test or production callers of
    /// execute_step. A future refactor that removes `step_name` from the tracing macro at
    /// pipeline.rs:470-474 would cause this test to FAIL on the `step_name` assertion.
    #[tokio::test]
    async fn test_BC_2_16_002_execute_step_emits_auth_initial_acquired_with_step_name_field() {
        let mock_server = MockServer::start().await;
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"items": [{"id": 1}]})),
            )
            .mount(&mock_server)
            .await;

        let (log_buf, _guard) = setup_log_capture();

        let step_name = "fetch_items_step";
        let spec = make_single_step_spec(&mock_server.uri(), step_name);
        let step = spec.tables[0].steps[0].clone();
        let prior_vars = HashMap::new();
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
        let http_client = reqwest::Client::new();
        let auth_provider = MockAuthProvider::new("real-token");

        let result = PipelineExecutor::execute_step(
            &step,
            &spec,
            &prior_vars,
            &context,
            &http_client,
            &auth_provider,
        )
        .await;

        assert!(
            result.is_ok(),
            "execute_step must succeed with MockAuthProvider; got {:?}",
            result.err()
        );

        let captured = log_buf.lock().unwrap().clone();
        // BC row 4: event_type field
        assert!(
            captured.contains("auth_initial_acquired"),
            "BC row 4: log must contain 'auth_initial_acquired'; captured: {captured}",
        );
        // Must NOT emit the empty variant (token is non-empty)
        assert!(
            !captured.contains("auth_initial_acquired_empty"),
            "BC row 4: non-empty token must NOT emit 'auth_initial_acquired_empty'; captured: {captured}",
        );
        // BC row 4: step_name field must be present
        assert!(
            captured.contains(step_name),
            "BC row 4: log must contain step_name='{step_name}'; captured: {captured}",
        );
    }

    // ---------------------------------------------------------------------------
    // Test 2: BC row 5 — execute_step / auth_initial_acquired_empty / fields: sensor_id,
    // client_id, step_name
    // ---------------------------------------------------------------------------

    /// BC-2.16.002 Structured Event Catalog row 5:
    /// `execute_step` with an empty token (NullAuthProvider) emits
    /// `event_type = "auth_initial_acquired_empty"` at DEBUG level with fields
    /// `sensor_id`, `client_id`, and `step_name`.
    ///
    /// RED GATE: A future refactor merging the empty/non-empty Ok arms into a single
    /// emit, or removing `step_name` from the empty-token arm, would cause this test
    /// to FAIL on the `step_name` or `auth_initial_acquired_empty` assertion.
    #[tokio::test]
    async fn test_BC_2_16_002_execute_step_emits_auth_initial_acquired_empty_with_step_name_field()
    {
        let mock_server = MockServer::start().await;
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"items": [{"id": 1}]})),
            )
            .mount(&mock_server)
            .await;

        let (log_buf, _guard) = setup_log_capture();

        let step_name = "fetch_items_step";
        let spec = make_single_step_spec(&mock_server.uri(), step_name);
        let step = spec.tables[0].steps[0].clone();
        let prior_vars = HashMap::new();
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
        let http_client = reqwest::Client::new();
        let auth_provider = NullAuthProvider;

        let result = PipelineExecutor::execute_step(
            &step,
            &spec,
            &prior_vars,
            &context,
            &http_client,
            &auth_provider,
        )
        .await;

        assert!(
            result.is_ok(),
            "execute_step must succeed with NullAuthProvider (empty token path); got {:?}",
            result.err()
        );

        let captured = log_buf.lock().unwrap().clone();
        // BC row 5: event_type field
        assert!(
            captured.contains("auth_initial_acquired_empty"),
            "BC row 5: log must contain 'auth_initial_acquired_empty'; captured: {captured}",
        );
        // BC row 5: step_name field must be present
        assert!(
            captured.contains(step_name),
            "BC row 5: log must contain step_name='{step_name}'; captured: {captured}",
        );
    }

    // ---------------------------------------------------------------------------
    // Test 3: BC row 6 — execute_step / auth_initial_failed / fields: sensor_id,
    // client_id, step_name, detail
    // ---------------------------------------------------------------------------

    /// BC-2.16.002 Structured Event Catalog row 6:
    /// `execute_step` when `acquire_token` returns `Err` emits
    /// `event_type = "auth_initial_failed"` at ERROR level with fields
    /// `sensor_id`, `client_id`, `step_name`, and `detail`.
    ///
    /// FailingAuthProvider always errors without making any HTTP request.
    /// The wiremock server expects 0 calls (verifying the auth-abort path fires before HTTP).
    ///
    /// RED GATE: A future refactor that removes `step_name` or `detail` from the error
    /// arm's tracing macro at pipeline.rs:490-497 would cause this test to FAIL.
    #[tokio::test]
    async fn test_BC_2_16_002_execute_step_emits_auth_initial_failed_with_step_name_field() {
        let mock_server = MockServer::start().await;
        // FailingAuthProvider aborts before any HTTP — expect 0 wiremock hits.
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(0)
            .mount(&mock_server)
            .await;

        let (log_buf, _guard) = setup_log_capture();

        let step_name = "fetch_items_step";
        let spec = make_single_step_spec(&mock_server.uri(), step_name);
        let step = spec.tables[0].steps[0].clone();
        let prior_vars = HashMap::new();
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
        let http_client = reqwest::Client::new();
        let auth_provider = FailingAuthProvider::new();

        let result = PipelineExecutor::execute_step(
            &step,
            &spec,
            &prior_vars,
            &context,
            &http_client,
            &auth_provider,
        )
        .await;

        assert!(
            result.is_err(),
            "execute_step must fail when FailingAuthProvider errors; got Ok"
        );
        // HTTP must not have been called (auth abort fires before fetch).
        assert_eq!(
            auth_provider.calls(),
            1,
            "FailingAuthProvider must be called exactly once"
        );

        let captured = log_buf.lock().unwrap().clone();
        // BC row 6: event_type field
        assert!(
            captured.contains("auth_initial_failed"),
            "BC row 6: log must contain 'auth_initial_failed'; captured: {captured}",
        );
        // BC row 6: step_name field must be present
        assert!(
            captured.contains(step_name),
            "BC row 6: log must contain step_name='{step_name}'; captured: {captured}",
        );
        // BC row 6: detail field must be present (FailingAuthProvider includes error detail)
        assert!(
            captured.contains("detail"),
            "BC row 6: log must contain 'detail' field; captured: {captured}",
        );
    }
}
