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

/// Maximum total elements returned by a single JSONPath extraction (HIGH-007).
///
/// Guards against nested-wildcard memory amplification: a path like
/// `$.a[*].b[*].c[*]` against hostile JSON with large arrays produces O(|a|*|b|*|c|)
/// elements. This cap aborts extraction before OOM occurs.
const MAX_JSONPATH_RESULT_SIZE: usize = 100_000;

/// Maximum recursion depth for JSONPath traversal (HIGH-007).
///
/// Prevents stack overflow on deeply nested `[*]` wildcards (e.g., 32+ levels).
const MAX_JSONPATH_DEPTH: usize = 32;

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
                    // HIGH-001 (S-PLUGIN-PREREQ-C): emit structured tracing event before
                    // mapping to SpecEngineError so operators have observability even when
                    // the error is swallowed by a caller (BC-2.16.002 Structured Event Catalog).
                    let page_records =
                        extract_at_path(&body, &step.response_path).map_err(|e| {
                            tracing::warn!(
                                event_type = "jsonpath_extraction_failed",
                                sensor_id = %spec.sensor_id,
                                step_name = %step.name,
                                path = %step.response_path,
                                detail = %e,
                                "JSONPath extraction failed for response_path",
                            );
                            SpecEngineError::JsonPathExtractionFailed {
                                sensor_id: spec.sensor_id.clone(),
                                step_name: step.name.clone(),
                                path: step.response_path.clone(),
                                detail: e,
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
                            ..
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

        let extracted = extract_at_path(&body, &step.response_path).map_err(|e| {
            tracing::warn!(
                event_type = "jsonpath_extraction_failed",
                sensor_id = %spec.sensor_id,
                step_name = %step.name,
                path = %step.response_path,
                detail = %e,
                "JSONPath extraction failed for response_path in execute_step",
            );
            SpecEngineError::JsonPathExtractionFailed {
                sensor_id: spec.sensor_id.clone(),
                step_name: step.name.clone(),
                path: step.response_path.clone(),
                detail: e,
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
///
/// AC-1 (S-PLUGIN-PREREQ-C): When `PaginationConfig::CursorToken { page_size: Some(n), .. }`,
/// appends `page_size=n` to BOTH first-call and cursor-continuation URLs.
/// When `page_size: None`, the parameter is omitted (backward-compatible).
fn build_paged_url(
    base_url: &str,
    step: &FetchStep,
    cursor: &Option<String>,
    offset: u32,
) -> String {
    build_paged_url_impl(base_url, step, cursor, offset)
}

/// Public test-helper wrapper for `build_paged_url` (exposed under test-helpers feature).
///
/// Allows integration tests in `crates/prism-spec-engine/tests/` to call the private
/// URL-construction function directly rather than driving it through a full pipeline execution.
///
/// AC-1 (S-PLUGIN-PREREQ-C): Integration tests in `ac_1_cursor_page_size_test.rs` use this
/// to verify `page_size` threading without spinning up a wiremock server.
#[cfg(any(test, feature = "test-helpers"))]
pub fn build_paged_url_for_test(
    base_url: &str,
    step: &FetchStep,
    cursor: &Option<String>,
    offset: u32,
) -> String {
    build_paged_url_impl(base_url, step, cursor, offset)
}

fn build_paged_url_impl(
    base_url: &str,
    step: &FetchStep,
    cursor: &Option<String>,
    offset: u32,
) -> String {
    match &step.pagination {
        Some(PaginationConfig::CursorToken {
            page_size: ps_opt, ..
        }) => {
            let mut url = if let Some(c) = cursor {
                let sep = if base_url.contains('?') { '&' } else { '?' };
                format!("{base_url}{sep}cursor={c}")
            } else {
                base_url.to_string()
            };
            // AC-1: append page_size parameter when declared.
            if let Some(n) = ps_opt {
                let sep = if url.contains('?') { '&' } else { '?' };
                url = format!("{url}{sep}page_size={n}");
            }
            url
        }
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            let sep = if base_url.contains('?') { '&' } else { '?' };
            format!("{base_url}{sep}offset={offset}&limit={page_size}")
        }
        Some(PaginationConfig::None) | None => base_url.to_string(),
    }
}

/// Extract the value at a JSONPath expression.
///
/// ## Supported syntax (AC-2, S-PLUGIN-PREREQ-C)
///
/// Extended beyond dot-notation to support bracket notation and wildcards:
///
/// - `$.field` — dot-notation key lookup
/// - `$.a.b.c` — nested dot-notation
/// - `$.array[0]` — bracket index (0-based)
/// - `$.array[*].field` — wildcard enumeration (returns JSON array of all matches)
/// - Mixed: `$.data[0].items[*].name`
///
/// ## F-LP2-LOW-002: RFC 6901 escaping
///
/// Dot-separated key segments apply RFC 6901 escaping before lookup:
/// - `~` → `~0` (applied before `/` escape to avoid double-escape)
/// - `/` → `~1`
///
/// ## Out-of-bounds behavior (AC-2d)
///
/// `$.x[99]` on a 3-element array returns `Err` with a descriptive message mentioning
/// the index and bound; never panics.
///
/// ## Error variants
///
/// Returns `Err(String)` with a descriptive message for:
/// - Malformed path (missing `$.` prefix)
/// - Key not found at any step
/// - Bracket index out of bounds
/// - Wildcard on non-array value (EC-002)
fn extract_at_path(body: &serde_json::Value, path: &str) -> Result<serde_json::Value, String> {
    let stripped = path
        .strip_prefix("$.")
        .ok_or_else(|| format!("path must start with '$.' : {path}"))?;
    // F-LP5-LOW-001: reject "$." with no key segment.
    if stripped.is_empty() {
        return Err(format!(
            "response_path '{path}' must contain at least one key segment after '$.'",
        ));
    }

    // Tokenize the path into segments supporting both dot-notation and bracket notation.
    let tokens = tokenize_jsonpath(stripped);

    // Traverse the JSON value following the token sequence.
    // When a wildcard `[*]` is encountered, switch to multi-value mode.
    // HIGH-007: thread extraction context to enforce size and depth caps.
    let mut ctx = ExtractionContext::new();
    extract_with_tokens(body, &tokens, path, &mut ctx)
}

/// Accumulator threaded through extract_with_tokens to enforce resource caps.
///
/// HIGH-007 defense: prevents nested-wildcard O(N^k) memory amplification and
/// stack overflow from deeply nested paths.
struct ExtractionContext {
    /// Current recursion depth (incremented on wildcard recursion).
    depth: usize,
    /// Total elements produced so far (incremented on each wildcard result push).
    size: usize,
}

impl ExtractionContext {
    fn new() -> Self {
        Self { depth: 0, size: 0 }
    }
}

/// A single path token in a tokenized JSONPath expression.
#[derive(Debug, Clone)]
enum PathToken {
    /// A dot-notation key segment (e.g., `field`).
    Key(String),
    /// A bracket index (e.g., `[0]`).
    Index(usize),
    /// A wildcard selector (e.g., `[*]`).
    Wildcard,
}

/// Tokenize a JSONPath expression (after stripping the `$.` prefix) into tokens.
///
/// Handles:
/// - `field` → `Key("field")`
/// - `field[0]` → `Key("field")`, `Index(0)`
/// - `field[*]` → `Key("field")`, `Wildcard`
/// - `a.b[0].c[*]` → `Key("a")`, `Key("b")`, `Index(0)`, `Key("c")`, `Wildcard`
fn tokenize_jsonpath(path: &str) -> Vec<PathToken> {
    let mut tokens = Vec::new();
    // Split on `.` first to get dot-segments; each may contain bracket suffixes.
    for dot_segment in path.split('.') {
        if dot_segment.is_empty() {
            continue;
        }
        // Check if this segment contains a `[` bracket.
        if let Some(bracket_start) = dot_segment.find('[') {
            let key_part = &dot_segment[..bracket_start];
            if !key_part.is_empty() {
                // Apply RFC 6901 escaping for the key part.
                tokens.push(PathToken::Key(
                    key_part.replace('~', "~0").replace('/', "~1"),
                ));
            }
            // Parse bracket suffixes (there may be multiple: `field[0][*]`).
            let mut rest = &dot_segment[bracket_start..];
            while let Some(stripped) = rest.strip_prefix('[') {
                if let Some(end) = stripped.find(']') {
                    let inner = &stripped[..end];
                    if inner == "*" {
                        tokens.push(PathToken::Wildcard);
                    } else if let Ok(idx) = inner.parse::<usize>() {
                        tokens.push(PathToken::Index(idx));
                    }
                    rest = &stripped[end + 1..]; // advance past `]`
                } else {
                    break;
                }
            }
        } else {
            // No brackets — plain key segment with RFC 6901 escaping.
            tokens.push(PathToken::Key(
                dot_segment.replace('~', "~0").replace('/', "~1"),
            ));
        }
    }
    tokens
}

/// Traverse a JSON value following a sequence of path tokens.
///
/// Returns `Ok(Value)` for a single-value path (no wildcards), or
/// `Ok(Value::Array([...]))` for wildcard paths.
/// Returns `Err(String)` for missing keys, out-of-bounds indexes, type mismatches,
/// or when size/depth caps are exceeded (HIGH-007).
fn extract_with_tokens(
    current: &serde_json::Value,
    tokens: &[PathToken],
    original_path: &str,
    ctx: &mut ExtractionContext,
) -> Result<serde_json::Value, String> {
    // HIGH-007: depth cap — prevents stack overflow on deeply nested wildcards.
    if ctx.depth > MAX_JSONPATH_DEPTH {
        return Err(format!(
            "JSONPath depth exceeded {MAX_JSONPATH_DEPTH} levels in path '{original_path}'"
        ));
    }

    if tokens.is_empty() {
        return Ok(current.clone());
    }

    let (head, tail) = tokens.split_first().expect("tokens non-empty");

    match head {
        PathToken::Key(k) => {
            // RFC 6901 pointer step for key lookup.
            let pointer = format!("/{k}");
            let next = current
                .pointer(&pointer)
                .ok_or_else(|| format!("path not found: {original_path}"))?;
            extract_with_tokens(next, tail, original_path, ctx)
        }
        PathToken::Index(idx) => {
            let arr = current.as_array().ok_or_else(|| {
                format!("expected array at bracket index step in path '{original_path}'")
            })?;
            let elem = arr.get(*idx).ok_or_else(|| {
                format!(
                    "index {idx} out of bounds: array has {} elements in path '{original_path}'",
                    arr.len()
                )
            })?;
            extract_with_tokens(elem, tail, original_path, ctx)
        }
        PathToken::Wildcard => {
            // Wildcard: enumerate all elements of the array; apply remaining tokens to each.
            let arr = current.as_array().ok_or_else(|| {
                format!(
                    "wildcard [*] applied to non-array value in path '{original_path}' \
                     (EC-002: wildcard on object)"
                )
            })?;
            let mut results = Vec::with_capacity(arr.len().min(MAX_JSONPATH_RESULT_SIZE));
            ctx.depth += 1;
            for elem in arr {
                // HIGH-007: size cap — abort if total result elements exceed limit.
                if ctx.size >= MAX_JSONPATH_RESULT_SIZE {
                    tracing::warn!(
                        event_type = "jsonpath_size_cap_exceeded",
                        path = %original_path,
                        max_size = MAX_JSONPATH_RESULT_SIZE,
                        "JSONPath result size cap exceeded — truncating extraction",
                    );
                    return Err(format!(
                        "JSONPath result exceeded {MAX_JSONPATH_RESULT_SIZE} elements in path '{original_path}'"
                    ));
                }
                let val = extract_with_tokens(elem, tail, original_path, ctx)?;
                ctx.size += 1;
                results.push(val);
            }
            ctx.depth -= 1;
            Ok(serde_json::Value::Array(results))
        }
    }
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

    use crate::auth_provider::{
        AuthOutcome, ChainAuthProvider, FailingAuthProvider, MockAuthProvider, NullAuthProvider,
    };
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

    // ---------------------------------------------------------------------------
    // auth_refresh tests — BC v1.8 catalog rows 3, 7, 8, 9, 10
    // All invoke PipelineExecutor::execute (NOT execute_step) because the
    // auth_refresh_* events fire from issue_request_with_retry, called from execute.
    // ---------------------------------------------------------------------------

    fn make_execute_spec(base_url: &str) -> SensorSpec {
        SensorSpec {
            sensor_id: "auth-refresh-test-sensor".to_string(),
            name: "Auth Refresh Test Sensor".to_string(),
            auth_type: AuthType::BearerStatic,
            base_url: base_url.to_string(),
            tables: vec![TableSpec::new_point_in_time(
                "items",
                "security_finding",
                vec![ColumnSpec {
                    name: "id".to_string(),
                    column_type: prism_core::ColumnType::String,
                    ocsf_field: None,
                    options: vec![],
                }],
                vec![FetchStep {
                    name: "fetch_items".to_string(),
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
    // Test: BC row 3 — execute / auth_initial_failed / fields: sensor_id, client_id, detail
    // F-LP13-MED-001 closure
    // ---------------------------------------------------------------------------

    /// BC-2.16.002 Structured Event Catalog row 3:
    /// `PipelineExecutor::execute` when `acquire_token` returns `Err` at pipeline start
    /// emits `event_type = "auth_initial_failed"` at ERROR level with fields
    /// `sensor_id`, `client_id`, and `detail`. No HTTP request is issued.
    ///
    /// Distinct from row 6 (execute_step / auth_initial_failed): this test uses execute()
    /// which omits `step_name` from the emission (pipeline-level call site).
    ///
    /// RED GATE (F-LP13-MED-001): Before this test, the execute() auth_initial_failed path
    /// had only call-count + error-variant assertions (pipeline_oauth_retry.rs:284), with
    /// ZERO buffer assertions on the event_type string. A refactor removing `detail` from
    /// the error arm at pipeline.rs:165-173 would NOT have failed any prior test.
    #[tokio::test]
    async fn test_BC_2_16_002_execute_auth_initial_failed_emits_event_with_detail() {
        let mock_server = MockServer::start().await;
        // FailingAuthProvider aborts before any HTTP — expect 0 wiremock hits.
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(0)
            .mount(&mock_server)
            .await;

        let (log_buf, _guard) = setup_log_capture();

        let spec = make_execute_spec(&mock_server.uri());
        let table = spec.tables[0].clone();
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
        let http_client = reqwest::Client::new();
        let auth_provider = FailingAuthProvider::new();

        let result =
            PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

        assert!(
            result.is_err(),
            "row 3: execute must fail when FailingAuthProvider errors; got Ok"
        );
        // Negative assertion: auth_initial_acquired must NOT fire.
        let captured = log_buf.lock().unwrap().clone();
        assert!(
            !captured.contains("auth_initial_acquired"),
            "row 3: auth_initial_failed path must NOT emit 'auth_initial_acquired'; captured: {captured}",
        );
        // BC row 3: event_type field.
        assert!(
            captured.contains("auth_initial_failed"),
            "row 3: log must contain 'auth_initial_failed'; captured: {captured}",
        );
        // BC row 3: detail field must be present.
        assert!(
            captured.contains("detail"),
            "row 3: log must contain 'detail' field; captured: {captured}",
        );
    }

    // ---------------------------------------------------------------------------
    // Test: BC row 7 — issue_request_with_retry / auth_refresh_triggered / step_name
    // F-LP13-MED-001 closure
    // ---------------------------------------------------------------------------

    /// BC-2.16.002 Structured Event Catalog row 7:
    /// When HTTP 401 is received on first request, `issue_request_with_retry` emits
    /// `event_type = "auth_refresh_triggered"` at WARN level with `step_name`.
    ///
    /// Setup: MockAuthProvider (returns Ok on both calls); wiremock returns 401 then 200.
    ///
    /// RED GATE (F-LP13-MED-001): ZERO prior buffer assertions on "auth_refresh_triggered".
    /// `grep -rn 'contains.*auth_refresh_triggered'` in crates/prism-spec-engine → 0 matches.
    /// A refactor removing `step_name` from the auth_refresh_triggered tracing macro at
    /// pipeline.rs:629-635 would NOT have failed any prior test.
    #[tokio::test]
    async fn test_BC_2_16_002_auth_refresh_triggered_emits_event_with_step_name() {
        let mock_server = MockServer::start().await;
        // First request: 401 (triggers auth refresh).
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(ResponseTemplate::new(401))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;
        // Retry after refresh: 200 with data.
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"items": []})),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let (log_buf, _guard) = setup_log_capture();

        let spec = make_execute_spec(&mock_server.uri());
        let table = spec.tables[0].clone();
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
        let http_client = reqwest::Client::new();
        // MockAuthProvider: Ok on every call (both initial acquire and refresh).
        let auth_provider = MockAuthProvider::new("token1");

        let result =
            PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

        assert!(
            result.is_ok(),
            "row 7: 401→200 with refresh must succeed; got {:?}",
            result.err()
        );

        let captured = log_buf.lock().unwrap().clone();
        // BC row 7: event_type field.
        assert!(
            captured.contains("auth_refresh_triggered"),
            "row 7: log must contain 'auth_refresh_triggered'; captured: {captured}",
        );
        // BC row 7: step_name field must be present.
        assert!(
            captured.contains("fetch_items"),
            "row 7: log must contain step_name='fetch_items'; captured: {captured}",
        );
    }

    // ---------------------------------------------------------------------------
    // Test: BC row 8 — issue_request_with_retry / auth_refresh_succeeded / step_name
    // F-LP13-MED-001 closure
    // ---------------------------------------------------------------------------

    /// BC-2.16.002 Structured Event Catalog row 8:
    /// After auth_refresh_triggered, when `acquire_token` on the refresh path returns Ok,
    /// `issue_request_with_retry` emits `event_type = "auth_refresh_succeeded"` at INFO
    /// level with `step_name`.
    ///
    /// Distinct from row 7 by event_type literal ("auth_refresh_succeeded" vs "auth_refresh_triggered").
    /// Same wiremock setup as row 7 (401 then 200); same MockAuthProvider.
    ///
    /// RED GATE (F-LP13-MED-001): ZERO prior buffer assertions on "auth_refresh_succeeded".
    #[tokio::test]
    async fn test_BC_2_16_002_auth_refresh_succeeded_emits_event_with_step_name() {
        let mock_server = MockServer::start().await;
        // First request: 401 (triggers auth refresh).
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(ResponseTemplate::new(401))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;
        // Retry after refresh: 200 with data.
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"items": []})),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let (log_buf, _guard) = setup_log_capture();

        let spec = make_execute_spec(&mock_server.uri());
        let table = spec.tables[0].clone();
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
        let http_client = reqwest::Client::new();
        let auth_provider = MockAuthProvider::new("token1");

        let result =
            PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

        assert!(
            result.is_ok(),
            "row 8: 401→200 with refresh must succeed; got {:?}",
            result.err()
        );

        let captured = log_buf.lock().unwrap().clone();
        // BC row 8: event_type field (distinct from row 7).
        assert!(
            captured.contains("auth_refresh_succeeded"),
            "row 8: log must contain 'auth_refresh_succeeded'; captured: {captured}",
        );
        // BC row 8: step_name field must be present.
        assert!(
            captured.contains("fetch_items"),
            "row 8: log must contain step_name='fetch_items'; captured: {captured}",
        );
    }

    // ---------------------------------------------------------------------------
    // Test: BC row 9 — issue_request_with_retry / auth_refresh_failed / step_name + detail
    // F-LP13-MED-001 closure
    // ---------------------------------------------------------------------------

    /// BC-2.16.002 Structured Event Catalog row 9:
    /// When HTTP 401 is received and `acquire_token` on the refresh path returns Err,
    /// `issue_request_with_retry` emits `event_type = "auth_refresh_failed"` at ERROR
    /// level with `step_name` and `detail`. Pipeline aborts.
    ///
    /// Uses ChainAuthProvider: call 0 (initial acquire) → Ok("token1");
    ///                         call 1 (refresh)         → Err("cred store unavailable").
    /// Wiremock: first request returns 401 (triggering refresh). No retry request because
    /// refresh itself fails before the retry is issued.
    ///
    /// RED GATE (F-LP13-MED-001): ZERO prior buffer assertions on "auth_refresh_failed".
    #[tokio::test]
    async fn test_BC_2_16_002_auth_refresh_failed_emits_event_with_detail() {
        let mock_server = MockServer::start().await;
        // Only the initial request fires; refresh fails before retry → expect exactly 1 hit.
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&mock_server)
            .await;

        let (log_buf, _guard) = setup_log_capture();

        let spec = make_execute_spec(&mock_server.uri());
        let table = spec.tables[0].clone();
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
        let http_client = reqwest::Client::new();
        // Call 0: initial acquire → Ok. Call 1: refresh → Err.
        let auth_provider = ChainAuthProvider::new(vec![
            AuthOutcome::Ok("token1".to_string()),
            AuthOutcome::Err("cred store unavailable".to_string()),
        ]);

        let result =
            PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

        assert!(
            result.is_err(),
            "row 9: auth_refresh_failed path must return Err; got Ok"
        );

        let captured = log_buf.lock().unwrap().clone();
        // BC row 9: event_type field.
        assert!(
            captured.contains("auth_refresh_failed"),
            "row 9: log must contain 'auth_refresh_failed'; captured: {captured}",
        );
        // BC row 9: step_name field must be present.
        assert!(
            captured.contains("fetch_items"),
            "row 9: log must contain step_name='fetch_items'; captured: {captured}",
        );
        // BC row 9: detail field must be present.
        assert!(
            captured.contains("detail"),
            "row 9: log must contain 'detail' field; captured: {captured}",
        );
    }

    // ---------------------------------------------------------------------------
    // Test: BC row 10 — issue_request_with_retry / auth_refresh_double_401
    // F-LP13-MED-001 closure
    // ---------------------------------------------------------------------------

    /// BC-2.16.002 Structured Event Catalog row 10:
    /// When HTTP 401 is received on first request AND the retry after token refresh
    /// ALSO returns 401, `issue_request_with_retry` emits
    /// `event_type = "auth_refresh_double_401"` at ERROR level with `step_name`.
    /// Pipeline aborts with `SpecEngineError::AuthRefreshFailed`.
    ///
    /// Uses MockAuthProvider (succeeds on both acquire and refresh calls).
    /// Wiremock: both the initial request AND the retry return 401.
    ///
    /// RED GATE (F-LP13-MED-001): ZERO prior buffer assertions on "auth_refresh_double_401".
    /// A refactor removing `step_name` from the double-401 tracing macro at pipeline.rs:682-688
    /// would NOT have failed any prior test.
    #[tokio::test]
    async fn test_BC_2_16_002_auth_refresh_double_401_emits_event() {
        let mock_server = MockServer::start().await;
        // All requests return 401 — both initial and retry.
        WmMock::given(wm_method("GET"))
            .and(wm_path("/items"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let (log_buf, _guard) = setup_log_capture();

        let spec = make_execute_spec(&mock_server.uri());
        let table = spec.tables[0].clone();
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
        let http_client = reqwest::Client::new();
        // MockAuthProvider: Ok on both calls (acquire + refresh succeed; double-401 is the
        // server side, not the auth provider side).
        let auth_provider = MockAuthProvider::new("token-that-wont-work");

        let result =
            PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

        assert!(
            result.is_err(),
            "row 10: double-401 must return Err; got Ok"
        );

        let captured = log_buf.lock().unwrap().clone();
        // BC row 10: event_type field.
        assert!(
            captured.contains("auth_refresh_double_401"),
            "row 10: log must contain 'auth_refresh_double_401'; captured: {captured}",
        );
        // BC row 10: step_name field must be present.
        assert!(
            captured.contains("fetch_items"),
            "row 10: log must contain step_name='fetch_items'; captured: {captured}",
        );
    }
}

// ---------------------------------------------------------------------------
// S-PLUGIN-PREREQ-C: AC-1 Red Gate — page_size on CursorToken first-call and continuation
//
// BC-2.16.002 postcondition: pagination follows the sensor spec's declared config.
// AC-1 extends `PaginationConfig::CursorToken` with `page_size: Option<u32>`.
// When `Some(n)`, `page_size=n` MUST appear in first-call and continuation URLs.
// When `None`, no `page_size` parameter may appear.
//
// RED GATE MECHANISM: `build_paged_url` currently ignores the `page_size` field
// (see TD-S-PLUGIN-PREREQ-B-001 comment at pipeline.rs build_paged_url). These tests
// assert the EXPECTED postcondition; they fail until `build_paged_url` reads and
// threads `page_size` into the URL.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod cursor_page_size_tests {
    use super::{PaginationConfig, build_paged_url};
    use crate::spec_parser::FetchStep;

    fn cursor_step(page_size: Option<u32>) -> FetchStep {
        FetchStep {
            name: "fetch".to_string(),
            method: "GET".to_string(),
            path_template: "/api/devices".to_string(),
            body_template: None,
            response_path: "$.resources".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: Some(PaginationConfig::CursorToken {
                cursor_response_path: "$.next_cursor".to_string(),
                page_size,
            }),
        }
    }

    /// AC-1(a): `page_size: Some(50)` on a first call (no cursor) → URL contains `page_size=50`.
    ///
    /// RED GATE: `build_paged_url` does not yet thread `page_size` into the URL.
    /// This test MUST FAIL until AC-1 implementation is complete.
    #[test]
    fn test_BC_2_16_002_cursor_pagination_first_call_includes_page_size() {
        let step = cursor_step(Some(50));
        let base = "https://api.crowdstrike.com/devices/queries/devices/v1";
        let url = build_paged_url(base, &step, &None, 0);
        assert!(
            url.contains("page_size=50"),
            "AC-1 RED GATE: first-call URL must contain 'page_size=50' when page_size=Some(50); \
             got: {url}\n\
             IMPLEMENTATION NEEDED: update build_paged_url to read \
             PaginationConfig::CursorToken {{ page_size }} and append page_size=N to the URL."
        );
    }

    /// AC-1(b): `page_size: Some(50)` on a continuation call (cursor present) → URL contains
    /// both `page_size=50` and the cursor parameter.
    ///
    /// RED GATE: `build_paged_url` does not yet append `page_size` on continuation calls.
    /// This test MUST FAIL until AC-1 implementation is complete.
    #[test]
    fn test_BC_2_16_002_cursor_pagination_continuation_includes_page_size() {
        let step = cursor_step(Some(50));
        let base = "https://api.crowdstrike.com/devices/queries/devices/v1";
        let cursor = Some("cursor_xyz_abc".to_string());
        let url = build_paged_url(base, &step, &cursor, 0);
        assert!(
            url.contains("page_size=50"),
            "AC-1 RED GATE: continuation URL must contain 'page_size=50' when page_size=Some(50); \
             got: {url}"
        );
        assert!(
            url.contains("cursor_xyz_abc"),
            "continuation URL must also contain the cursor value; got: {url}"
        );
    }

    /// AC-1(c): `page_size: None` → URL does NOT contain `page_size` parameter (backward compat).
    ///
    /// This assertion is expected to PASS already (existing behavior omits page_size).
    /// Included to document the backward-compat invariant.
    #[test]
    fn test_BC_2_16_002_cursor_pagination_page_size_none_omitted() {
        let step = cursor_step(None);
        let base = "https://api.crowdstrike.com/devices/queries/devices/v1";
        // First call: no cursor, page_size None
        let url_first = build_paged_url(base, &step, &None, 0);
        assert!(
            !url_first.contains("page_size="),
            "when page_size=None, first-call URL must not contain 'page_size='; got: {url_first}"
        );
        // Continuation call: cursor present, page_size None
        let cursor = Some("some_cursor".to_string());
        let url_cont = build_paged_url(base, &step, &cursor, 0);
        assert!(
            !url_cont.contains("page_size="),
            "when page_size=None, continuation URL must not contain 'page_size='; got: {url_cont}"
        );
    }
}

// ---------------------------------------------------------------------------
// S-PLUGIN-PREREQ-C: AC-2 Red Gate — JSONPath bracket notation + wildcard support
//
// BC-2.16.002 postcondition: `extract_at_path` supports dot-notation paths.
// AC-2 extends this to bracket indexing (`$.x[0]`) and wildcard (`$.x[*]`).
//
// RED GATE MECHANISM: `extract_at_path` currently only supports dot-notation paths
// (see TD-S-PLUGIN-PREREQ-B-003 comment). Bracket notation returns Err.
// These tests assert the EXPECTED postcondition; they fail until AC-2 is implemented.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod jsonpath_bracket_tests {
    use super::extract_at_path;
    use serde_json::json;

    /// AC-2(a): `$.devices[0].id` on an array-valued JSON object extracts the first element.
    ///
    /// RED GATE: `extract_at_path` splits on `.` only; `[0]` is not recognized as an
    /// array index, so this path fails to match. Test MUST FAIL until AC-2 is complete.
    #[test]
    fn test_BC_2_16_002_extract_bracket_index() {
        let body = json!({
            "devices": [
                {"id": "device-A", "hostname": "host1"},
                {"id": "device-B", "hostname": "host2"}
            ]
        });
        let result = extract_at_path(&body, "$.devices[0].id");
        assert!(
            result.is_ok(),
            "AC-2 RED GATE: $.devices[0].id must succeed; got Err: {:?}\n\
             IMPLEMENTATION NEEDED: extend extract_at_path to parse bracket index notation.",
            result.err()
        );
        assert_eq!(
            result.unwrap(),
            json!("device-A"),
            "$.devices[0].id must return 'device-A'"
        );
    }

    /// AC-2(b): `$.devices[*].id` on an array-valued JSON object returns all matching values.
    ///
    /// RED GATE: wildcard `[*]` is not supported by the current dot-split path traversal.
    /// Test MUST FAIL until AC-2 is complete.
    #[test]
    fn test_BC_2_16_002_extract_wildcard_enumeration() {
        let body = json!({
            "devices": [
                {"id": "device-A"},
                {"id": "device-B"}
            ]
        });
        let result = extract_at_path(&body, "$.devices[*].id");
        assert!(
            result.is_ok(),
            "AC-2 RED GATE: $.devices[*].id must succeed; got Err: {:?}\n\
             IMPLEMENTATION NEEDED: extend extract_at_path to support wildcard [*] enumeration \
             returning a JSON array of matched values.",
            result.err()
        );
        let values = result.unwrap();
        let expected = json!(["device-A", "device-B"]);
        assert_eq!(
            values, expected,
            "$.devices[*].id must return [\"device-A\", \"device-B\"]; got: {values}"
        );
    }

    /// AC-2(c): Backward compat — `$.resources` on an object still resolves to the array.
    ///
    /// This test verifies the existing dot-notation behavior is unchanged after AC-2 impl.
    /// Expected to PASS before AC-2 (existing behavior). Included as a regression anchor.
    #[test]
    fn test_BC_2_16_002_extract_backward_compat_dot_path() {
        let body = json!({
            "resources": [{"id": 1}, {"id": 2}]
        });
        let result = extract_at_path(&body, "$.resources");
        assert!(
            result.is_ok(),
            "backward compat: $.resources must still resolve; got Err: {:?}",
            result.err()
        );
        assert_eq!(
            result.unwrap(),
            json!([{"id": 1}, {"id": 2}]),
            "$.resources must return the full array"
        );
    }

    /// AC-2(d): `$.x[99]` on a 3-element array returns a structured error (not panic, not None).
    ///
    /// RED GATE: current `extract_at_path` returns `Err(String)` for any bracket path.
    /// After AC-2, it must return `Err` specifically for out-of-bounds (not panic).
    /// This test will fail at the first assertion because `$.x[99]` syntax is not
    /// parsed — after AC-2 it should return Err due to out-of-bounds (not due to
    /// unrecognized syntax). The behavior changes but the no-panic invariant is the goal.
    #[test]
    fn test_BC_2_16_002_extract_bracket_out_of_bounds_structured_error() {
        let body = json!({
            "x": [1, 2, 3]
        });
        let result = extract_at_path(&body, "$.x[99]");
        // Post-AC-2: must return Err (structured, not panic). Currently returns Err for a
        // different reason (unrecognized bracket syntax). The invariant: MUST NOT panic.
        assert!(
            result.is_err(),
            "AC-2: $.x[99] on a 3-element array must return Err (out-of-bounds); \
             after AC-2 impl this Err should have a descriptive message, not just 'path not found'"
        );
        // Post-AC-2 refinement: the error message should indicate out-of-bounds.
        // Before AC-2 this message says "path must start with '$.'..." or "path not found".
        // After AC-2 this message should say something like "index 99 out of bounds".
        // This assertion documents the EXPECTED post-AC-2 error message and FAILS before AC-2.
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("out of bounds")
                || err_msg.contains("index")
                || err_msg.contains("99"),
            "AC-2 RED GATE: out-of-bounds error message must reference the index or 'out of bounds'; \
             current message before AC-2 is: '{err_msg}'"
        );
    }

    /// HIGH-007: JSONPath result size cap fires when nested wildcards produce > 100_000 elements.
    ///
    /// `$.a[*].b[*]` on a 201x500 = 100_500 element nested array must return Err.
    #[test]
    fn test_BC_2_16_002_jsonpath_wildcard_size_cap_fires() {
        // Build 201 items each containing 500 b-values = 100_500 > MAX_JSONPATH_RESULT_SIZE
        let inner: Vec<serde_json::Value> = (0..500).map(|i| json!(i)).collect();
        let outer: Vec<serde_json::Value> = (0..201).map(|_| json!({"b": inner.clone()})).collect();
        let body = json!({"a": outer});
        let result = extract_at_path(&body, "$.a[*].b[*]");
        assert!(
            result.is_err(),
            "HIGH-007: nested wildcard producing >100_000 elements must return Err; got Ok"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("exceeded") || err.contains("100000") || err.contains("100_000"),
            "HIGH-007: size cap error must mention 'exceeded' or the cap value; got: {err}"
        );
    }

    /// HIGH-007: Depth cap fires at 32+ nested wildcard levels.
    ///
    /// A path with 33 nested wildcards must return Err before stack overflow.
    #[test]
    fn test_BC_2_16_002_jsonpath_depth_cap_fires() {
        // Build a deeply nested array: [[[[...]]]] 33 levels deep with single element each.
        let mut deep: serde_json::Value = json!([1]);
        for _ in 0..33 {
            deep = json!([deep]);
        }
        let body = json!({"a": deep});
        // Path with 33 wildcards
        let path = "$.a[*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*][*]";
        let result = extract_at_path(&body, path);
        assert!(
            result.is_err(),
            "HIGH-007: 33-level deep wildcard path must return Err (depth cap); got Ok"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("depth") || err.contains("exceeded"),
            "HIGH-007: depth cap error must mention 'depth' or 'exceeded'; got: {err}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-3: proptest for `extract_at_path` totality
//
// BC-2.16.002 postcondition: extract_at_path returns Ok(_) or Err(_) for any
// (Value, &str) input — never panics, never produces an unwrap() failure.
//
// HIGH-002 (S-PLUGIN-PREREQ-C): proptest body was previously fixed (hardcoded JSON);
// AC-3(c) required "ANY JSON string" as input. The body is now an arbitrary
// serde_json::Value generated via a depth-limited recursive strategy. The path
// regex is also expanded to include `~` characters (RFC 6901 tilde escapes).
//
// Placed in-module (not in tests/proptest_AC_3.rs) because `extract_at_path`
// is a private function. The tests/proptest_AC_3.rs sentinel test delegates
// to the canonical test in this module once the proptest is wired here.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_extract_at_path {
    use super::extract_at_path;

    use proptest::prelude::*;
    use serde_json::Value;

    /// Generate an arbitrary JSON leaf value.
    fn json_leaf() -> impl Strategy<Value = Value> {
        prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(|b| Value::Bool(b)),
            any::<i64>().prop_map(|n| Value::Number(n.into())),
            ".*".prop_map(|s: String| Value::String(s)),
        ]
    }

    /// Generate an arbitrary JSON value with depth-bounded recursion.
    ///
    /// Depth 4 and branching factor 8 produce bodies up to ~4096 nodes —
    /// realistic for API responses without being too slow for proptest.
    fn arbitrary_json() -> impl Strategy<Value = Value> {
        json_leaf().prop_recursive(4, 64, 8, |inner| {
            prop_oneof![
                // JSON array: 0..8 elements of arbitrary type
                prop::collection::vec(inner.clone(), 0..8).prop_map(|v| Value::Array(v)),
                // JSON object: 0..8 key-value pairs
                prop::collection::hash_map(".*", inner, 0..8)
                    .prop_map(|m| { Value::Object(m.into_iter().collect()) }),
            ]
        })
    }

    proptest! {
        /// AC-3(c): `extract_at_path` totality — for ANY JSON value and path string,
        /// the function returns Ok(_) or Err(_) without panic.
        ///
        /// HIGH-002: body strategy is now arbitrary JSON (not a fixed literal).
        /// Path regex includes `~` for RFC 6901 tilde escape coverage.
        ///
        /// Traces to BC-2.16.002 postcondition: JSONPath extraction returns Ok or Err.
        #[test]
        fn proptest_extract_at_path_totality(
            body in arbitrary_json(),
            path in "\\$\\.[a-zA-Z0-9_\\.\\[\\]\\*~]{1,30}"
        ) {
            // The invariant: MUST NOT panic. Return type is always Ok(_) or Err(_).
            let _ = extract_at_path(&body, &path);
        }
    }
}
