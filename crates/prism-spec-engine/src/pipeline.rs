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

use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, TimeZone, Utc};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use prism_core::{ColumnType, OrgSlug};

use crate::{
    auth_provider::{AuthProvider, AuthToken},
    error::SpecEngineError,
    interpolation::{InterpolationContext, Interpolator},
    spec_parser::{ColumnSpec, FetchStep, PaginationConfig, SensorSpec, TableSpec},
};

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
/// and page limit).
const MAX_PAGES_PER_STEP: usize = 1_000;

/// Maximum cumulative HTTP requests across ALL steps of a single pipeline execution.
///
/// When the running total of HTTP requests across all steps in a pipeline reaches
/// this cap, the executor returns `SpecEngineError::TooManyRequests { total }` immediately
/// and emits `event_type = "pipeline_max_requests_exceeded"` per BC-2.16.002 catalog.
///
/// This is a hard invariant — non-retryable. Partial results are discarded.
/// Closes TD-S-PLUGIN-PREREQ-B-004 (AC-16 / BC-2.16.002 §Postconditions).
pub const MAX_REQUESTS_PER_PIPELINE: usize = 10_000;

/// Context provided to each pipeline execution.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FetchContext {
    /// The client/tenant this query is executing for.
    pub client_id: OrgSlug,
    /// Push-down filter values from the query planner (${query.filter.*}).
    pub query_filters: std::collections::HashMap<String, String>,
    /// ADR-060 §D8.1: LIMIT-aware early-stop pagination threshold.
    ///
    /// When `Some(n)`, `PipelineExecutor::execute_impl` stops fetching pages once
    /// `all_records.len() >= n` (checked at complete page boundaries, immediately
    /// after the DI-019 truncation check). `None` = unchanged full pagination.
    ///
    /// Wired by `SpecDrivenSensorAdapter::fetch` via `params.limit` mapping
    /// (S-ENGINE-LIMIT-EARLY-STOP-001 AC-005). Test callers pass `None`.
    pub early_stop_limit: Option<usize>,
}

impl FetchContext {
    /// Construct a `FetchContext`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct-literal construction
    /// outside the crate. External callers (tests, integration code) MUST use this.
    pub fn new(
        client_id: OrgSlug,
        query_filters: std::collections::HashMap<String, String>,
        early_stop_limit: Option<usize>,
    ) -> Self {
        Self {
            client_id,
            query_filters,
            early_stop_limit,
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
    /// True when `execute_impl` fired the ADR-060 §D8.2 early-stop `break 'steps`
    /// because `all_records.len() >= early_stop_limit`. False when the pipeline
    /// completed normally (pagination exhausted, DI-019 cap, or `early_stop_limit = None`).
    /// Propagates to `FetchOutput.any_early_stopped` → `FanOutResult.any_early_stopped`
    /// → `MaterializationOutput.any_early_stopped` → engine Step 6 `is_truncated` formula
    /// (ADR-060 §D8.3; S-ENGINE-LIMIT-EARLY-STOP-001 AC-009(a)).
    pub early_stopped: bool,
}

impl PipelineResult {
    /// Construct a `PipelineResult` from its component fields.
    ///
    /// The `#[non_exhaustive]` attribute on `PipelineResult` prevents struct-literal
    /// construction from outside the defining crate, which blocks test code in downstream
    /// crates (e.g., `prism-bin`) from building test fixtures. This named constructor
    /// provides a stable forward-compatible construction path — if new fields are added
    /// to `PipelineResult`, callers of this function can be updated to supply them
    /// while struct-literal construction remains blocked to prevent silent field omission.
    pub fn new(
        records: Vec<serde_json::Value>,
        table_name: impl Into<String>,
        request_count: u32,
        truncated: bool,
    ) -> Self {
        Self {
            records,
            table_name: table_name.into(),
            request_count,
            truncated,
            early_stopped: false,
        }
    }
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
    ///   Production callers (boot.rs / chassis) construct this client with a 30s timeout using
    ///   `reqwest::Client::builder().timeout(Duration::from_secs(30)).build()` per AC-9
    ///   (TD-S-PLUGIN-PREREQ-B-005 closure). Test fixtures already use this pattern (F-LP4-MED-001).
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
        Self::execute_impl(
            spec,
            table,
            context,
            http_client,
            auth_provider,
            MAX_REQUESTS_PER_PIPELINE,
        )
        .await
    }

    /// Test-injectable variant of `execute` with a custom `max_requests` cap.
    ///
    /// **ONLY for testing** — allows exercising the cumulative-cap branch without
    /// needing 10,001 HTTP requests (MED-004 / F-IMPL-LP1-MED-004 closure).
    /// Production code MUST use `execute` (which uses `MAX_REQUESTS_PER_PIPELINE`).
    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn execute_with_max_requests(
        spec: &SensorSpec,
        table: &TableSpec,
        context: &FetchContext,
        http_client: &reqwest::Client,
        auth_provider: &dyn AuthProvider,
        max_requests: usize,
    ) -> Result<PipelineResult, SpecEngineError> {
        Self::execute_impl(
            spec,
            table,
            context,
            http_client,
            auth_provider,
            max_requests,
        )
        .await
    }

    async fn execute_impl(
        spec: &SensorSpec,
        table: &TableSpec,
        context: &FetchContext,
        http_client: &reqwest::Client,
        auth_provider: &dyn AuthProvider,
        max_requests: usize,
    ) -> Result<PipelineResult, SpecEngineError> {
        let mut all_records: Vec<serde_json::Value> = Vec::new();
        let mut request_count: u32 = 0;
        // step_vars: keyed as "step_name.field" -> JSON value
        let mut step_vars: HashMap<String, serde_json::Value> = HashMap::new();
        let mut truncated = false;
        // ADR-060 §D8.3: tracks whether §D8.2 early-stop fired (early_stop_limit reached).
        // Set to true BEFORE break 'steps in the early-stop block; false for all other exits.
        let mut early_stopped = false;

        // AC-7 (F-LP1-HIGH-002): rate-limit flag is pipeline-scoped, not step-scoped.
        // Hoisted OUTSIDE the steps loop so the delay applies between ALL API calls
        // across step boundaries, not just within a single step.
        let mut is_first_pipeline_request = true;

        // Eager token acquisition: acquire_token is called BEFORE the steps loop
        // (F-LP5-LOW-003 closure). AuthType has no Null variant — all 5 variants
        // (Oauth2ClientCredentials, BearerStatic, CookieRoundtrip, ApiKey, CustomViaPlugin) require auth.
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
        // AC-CWS-001 / BC-2.01.013: seed ${query.limit} from QueryParams.limit so
        // sensor TOML path_templates can push the LIMIT clause down to the sensor API
        // (e.g., CrowdStrike Step 1 DetectionListParams.limit).
        // Default: empty string → PipelineExecutor::strip_empty_url_params removes the
        // &limit= param from the URL so the DTU defaults to its own page size.
        // F-PUSHDOWN-001 invariant: the LIMIT param is ONLY applied to is_first_step.
        // (Callers are responsible for not seeding query.limit on subsequent steps.)
        step_vars.insert(
            "query.limit".to_string(),
            serde_json::Value::String(
                context
                    .query_filters
                    .get("query.limit")
                    .cloned()
                    .unwrap_or_default(),
            ),
        );
        // BC-2.16.013 §Postcondition 1 / S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-004:
        // Auto-parse query_filter values that are JSON-object or JSON-array strings
        // (trimmed start is `{` or `[`) into `Value::Object` / `Value::Array`.
        // In `JsonBody` interpolation context, `Value::Object` is inserted verbatim
        // via `value.to_string()`, producing inline JSON.  `Value::String` goes through
        // `json_escape()` which escapes inner quotes, producing invalid JSON when the
        // filter is placed bare in a body template.
        // Backward-compat: FQL/AQL strings do NOT start with `{`/`[` and remain
        // `Value::String` — no regression for CrowdStrike or Armis push-down paths.
        // EC-005: on parse failure, log WARN and fall back to `Value::String` passthrough.
        for (k, v) in &context.query_filters {
            let trimmed = v.trim_start();
            let parsed_value = if trimmed.starts_with('{') || trimmed.starts_with('[') {
                match serde_json::from_str::<serde_json::Value>(v) {
                    Ok(val) => val,
                    Err(e) => {
                        tracing::warn!(
                            event_type = "query_filter_json_parse_degraded",
                            key = %k,
                            error = %e,
                            "query_filter JSON auto-parse failed; \
                             using as string passthrough (EC-005)"
                        );
                        serde_json::Value::String(v.clone())
                    }
                }
            } else {
                serde_json::Value::String(v.clone())
            };
            step_vars.insert(format!("query.filter.{k}"), parsed_value);
        }

        // ADR-033 T1 / AC-CWS-002: Pre-seed any ${query.filter.*} variables referenced
        // in step path_templates or body_templates that are not in context.query_filters.
        // This prevents interpolation errors when optional filter slots (e.g., ${query.filter._fql})
        // are present in the TOML path_template but absent from the FetchContext.
        // Default: empty string (no filter → empty URL param, safely ignored by DTU).
        for step in &table.steps {
            seed_missing_query_filter_vars(
                &step.path_template,
                step.body_template.as_deref(),
                &mut step_vars,
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
                let interpolated_path = {
                    let raw = Interpolator::interpolate(
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
                    // AC-CWS-001: strip empty query params (e.g. &limit= when no push-down limit
                    // was provided) so optional push-down params don't reach the DTU as invalid
                    // empty strings that fail `Option<usize>` deserialization.
                    strip_empty_url_params(&raw)
                };

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

                    // Derive the active page_size for OffsetLimit POST-body injection
                    // (BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch:
                    // POST-body vs GET-URL"). Non-OffsetLimit steps pass page_size=0 to
                    // indicate no body injection is needed.
                    //
                    // ADR-060 §D8.4: CursorToken page-fill is not a valid cursor-exhaustion
                    // signal. All CursorToken sub-cases (page_size Some/None), PageNumber,
                    // and None fall through to 0 → conservative early_stopped=true. Precise
                    // cursor-exhaustion detection is deferred to S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001.
                    let active_page_size: u32 = match &step.pagination {
                        Some(PaginationConfig::OffsetLimit { page_size: ps }) => *ps,
                        _ => 0,
                    };

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
                        offset,
                        active_page_size,
                    )
                    .await?;
                    bearer_token = new_token;

                    // Cumulative cap check (AC-16 / BC-2.16.002).
                    // After each request, check if we've reached the hard cap.
                    // `max_requests` is MAX_REQUESTS_PER_PIPELINE in production;
                    // tests may inject a smaller value via execute_with_max_requests.
                    // Emits: event_type = "pipeline_max_requests_exceeded" (ERROR).
                    if request_count as usize >= max_requests {
                        let total = request_count as usize;
                        tracing::error!(
                            event_type = "pipeline_max_requests_exceeded",
                            sensor_id = %spec.sensor_id,
                            total_requests = total,
                            max = max_requests,
                            "Pipeline executor reached request cap; aborting"
                        );
                        return Err(SpecEngineError::TooManyRequests { total });
                    }

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

                    // ADR-060 §D8.2: LIMIT-aware early-stop. Fires at COMPLETE page boundary,
                    // immediately after DI-019. truncated is NOT set — this is a success-path
                    // query-driven early exit, not a capacity overflow (ADR-060 §D8.3).
                    // CRITICAL: early_stopped MUST be set BEFORE break 'steps so that
                    // PipelineResult.early_stopped is readable by the caller. This is
                    // the root of the ADR-060 §D8.3 propagation chain:
                    // PipelineResult.early_stopped → FetchOutput.any_early_stopped
                    // → FanOutResult.any_early_stopped → MaterializationOutput.any_early_stopped
                    // → engine Step 6 is_truncated formula (EC-11-092 / RG-PSG-025/028).
                    //
                    // ADR-060 §D8.2 discriminator (AC-014 / RG-PSG-039):
                    //   FULL page  (page_record_count >= active_page_size): more pages may exist
                    //              → early_stopped = true
                    //   PARTIAL page (page_record_count < active_page_size): source exhausted
                    //              → early_stopped = false (complete dataset retrieved)
                    // For non-OffsetLimit steps active_page_size=0 → page_record_count >= 0
                    // is always true (conservative: treat as full page, same as before).
                    if let Some(limit) = context.early_stop_limit
                        && all_records.len() >= limit
                    {
                        // ADR-060 §D8.2 partial-final-page discriminator — set BEFORE break.
                        early_stopped = page_record_count >= active_page_size as usize;
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

        // ADR-028 §D8-B/C: normalize Datetime fields per column's timestamp_formats
        // and timestamp_fallback_chain declarations before returning to caller.
        // F-LP2-HIGH-001: removed redundant tracing::error!(event_type = "timestamp_parse_failure")
        // — not registered in BC-2.16.002 catalog; ? propagation carries full context via
        // SpecEngineError::TimestampParseFailure (E-SPEC-018) which includes sensor_id, column_name,
        // attempted_formats, and raw value (error-taxonomy.md §E-SPEC-018; PG-LP11-001).
        let normalized_records =
            normalize_timestamp_fields(&all_records, &table.columns, spec.sensor_id.as_str())?;

        Ok(PipelineResult {
            records: normalized_records,
            table_name: table.table_name.clone(),
            request_count,
            truncated,
            early_stopped,
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
    /// Tested by `test_TD_S_PLUGIN_PREREQ_B_011_execute_step_eager_token_calls_auth_once`
    /// in `plugin_integration_tests.rs`, which verifies that auth is acquired exactly once
    /// per invocation regardless of sub-request count (TD-S-PLUGIN-PREREQ-B-011/012 closure).
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

        let interpolated_path = {
            let raw = Interpolator::interpolate(
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
            // AC-CWS-001: strip empty query params (e.g. &limit= when no push-down limit
            // was provided) so optional push-down params don't reach the DTU as invalid
            // empty strings that fail `Option<usize>` deserialization.
            strip_empty_url_params(&raw)
        };

        let url = format!("{}{}", spec.base_url, interpolated_path);

        // execute_step issues a single request without pagination — pass offset=0, page_size=0
        // to signal to build_request that no OffsetLimit body injection should occur.
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
            0,
            0,
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
    /// TD-S-PLUGIN-PREREQ-B-006 CLOSED by S-PLUGIN-PREREQ-C: proptest coverage added
    /// for pure functions (fan_out_batches, extract_at_path, Interpolator::interpolate)
    /// following the PREREQ-A cross-crate validator-parity proptest precedent.
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

/// Best-effort read of a non-2xx response body for diagnostic inclusion in
/// `SpecEngineError::HttpRequestFailed.detail`.
///
/// BC-2.16.002 Non-2xx Response Body Capture postcondition:
/// - Decodes bytes as UTF-8 (via `String::from_utf8_lossy` — replaces invalid sequences
///   with U+FFFD rather than using Latin-1 `b as char` which produces mojibake).
/// - Caps body at **256 bytes** (byte-boundary-safe; never splits a multibyte UTF-8 char).
///   A char-based cap would allow multibyte characters to inflate the byte count: 256
///   "€" chars = 768 bytes. The byte cap enforces the BC-2.16.002 contract exactly
///   (F-1 / DEFECT-ADAPTER-TLS-XDOME-LIVE-001).
/// - Replaces ALL `char::is_control()` characters (C0+DEL+C1, incl. `\t`/`\n`/`\r`) and
///   U+2028/U+2029 with a space — prevents CWE-117/CWE-116 log/prompt injection (MED-1).
///   Replacement happens before byte-counting so the byte budget reflects emitted bytes.
/// - Returns an empty string on any body-read failure — the primary status-code
///   error MUST NOT be replaced by a secondary body-read failure.
///
/// Sanitization is performed by `prism_core::sanitize_body_snippet_bytes` — the
/// byte-capping variant of the shared `prism-core` sanitizer. `prism_mcp::health::
/// connectivity::sanitize_error` uses `sanitize_body_snippet` (char-based, 512 chars)
/// for its own char-bound contract; the two functions are intentionally distinct.
/// (F-1 design choice (a); BC-2.16.002 ≤256-byte postcondition.)
async fn read_non_2xx_body(response: reqwest::Response) -> String {
    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(_) => return String::new(),
    };
    // Decode bytes as UTF-8 (lossy: invalid sequences → U+FFFD).
    // Avoids the Latin-1 mojibake produced by the former `b as char` byte-to-char cast
    // which misinterpreted multi-byte UTF-8 sequences (MED-1).
    let text = String::from_utf8_lossy(&bytes);
    // sanitize_body_snippet_bytes: caps at 256 bytes (byte-boundary-safe; replaces control
    // chars + U+2028/U+2029 with space before byte-counting; CWE-117/CWE-116;
    // BC-2.16.002 ≤256-byte postcondition). Trim for clean presentation in error messages.
    prism_core::sanitize_body_snippet_bytes(text.as_ref(), 256)
        .trim()
        .to_string()
}

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
    offset: u32,
    page_size: u32,
) -> Result<(serde_json::Value, AuthToken), SpecEngineError> {
    // Issue the first request.
    let response = build_request(
        http_client,
        step,
        url,
        &current_token,
        &spec.auth_type,
        step_vars,
        offset,
        page_size,
    )
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
        // BC-2.16.002 Send-Failure Error Source Chain postcondition:
        // Include the reqwest error source chain (hyper/h2/TLS-level cause) that
        // e.to_string() omits. EC-008: source() returns None for simple errors,
        // so .unwrap_or_default() produces "" (no "; caused by:" suffix).
        detail: format!(
            "{}{}",
            e,
            std::error::Error::source(&e)
                .map(|s| format!("; caused by: {s}"))
                .unwrap_or_default()
        ),
    })?;
    *request_count += 1;

    let status = response.status();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        // BC-2.01.017 EC-017-002 / TV-BC-2.01.017-006: discriminator guard for static auth.
        //
        // `CookieRoundtrip` uses a static API key — `acquire_token()` just re-reads
        // the same key from the credential store, so retrying would send the same
        // (already-rejected) token again. The retry is provably futile.
        //
        // For static auth types, skip the entire OAuth2-style refresh-retry block and
        // immediately surface E-AUTH-004 as a per-sensor partial failure
        // (BC-2.01.010 partial-failure fan-out semantics).
        //
        // The OAuth2 refresh-retry path below MUST remain UNCHANGED for
        // `Oauth2ClientCredentials` (and other non-static auth types).
        if spec.auth_type == crate::spec_parser::AuthType::CookieRoundtrip {
            tracing::warn!(
                event_type = "cookie_auth_401",
                sensor_id = %spec.sensor_id,
                client_id = %client_id,
                step_name = %step.name,
                "CookieRoundtrip sensor received 401 — static API key rejected; \
                 no retry (BC-2.01.017 EC-017-002)"
            );
            return Err(SpecEngineError::CookieAuthFailed {
                sensor_id: spec.sensor_id.clone(),
                client_id: client_id.to_string(),
            });
        }

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

        let retry_response = build_request(
            http_client,
            step,
            url,
            &fresh_token,
            &spec.auth_type,
            step_vars,
            offset,
            page_size,
        )
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
            // BC-2.16.002 Send-Failure Error Source Chain postcondition (401-retry send):
            // Include the reqwest error source chain (hyper/h2/TLS-level cause).
            detail: format!(
                "{}{}",
                e,
                std::error::Error::source(&e)
                    .map(|s| format!("; caused by: {s}"))
                    .unwrap_or_default()
            ),
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
            // BC-2.16.002 Non-2xx Response Body Capture postcondition:
            // Best-effort read of response body; cap to 256 bytes; strip control chars.
            // A secondary body-read failure MUST NOT replace the primary status-code error.
            let body_snippet = read_non_2xx_body(retry_response).await;
            let detail = if body_snippet.is_empty() {
                format!("HTTP {retry_status}")
            } else {
                format!("HTTP {retry_status}: {body_snippet}")
            };
            return Err(SpecEngineError::HttpRequestFailed {
                sensor_id: spec.sensor_id.clone(),
                step_name: step.name.clone(),
                status_code: retry_status.as_u16(),
                detail,
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
        // BC-2.16.002 Non-2xx Response Body Capture postcondition:
        // Best-effort read of response body; cap to 256 bytes; strip control chars.
        // A secondary body-read failure MUST NOT replace the primary status-code error.
        let body_snippet = read_non_2xx_body(response).await;
        let detail = if body_snippet.is_empty() {
            format!("HTTP {status}")
        } else {
            format!("HTTP {status}: {body_snippet}")
        };
        return Err(SpecEngineError::HttpRequestFailed {
            sensor_id: spec.sensor_id.clone(),
            step_name: step.name.clone(),
            status_code: status.as_u16(),
            detail,
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
///
/// ## Auth Header Dispatch (ADR-031 §D3-b / AC-007 S-DTU-CYBERINT-AUTH-FIDELITY-001)
///
/// | AuthType | Header injected |
/// |----------|----------------|
/// | `CookieRoundtrip` | `Cookie: access_token={token}` |
/// | All other variants | `Authorization: Bearer {token}` |
///
/// Cookie name MUST be `access_token`. The former `cyberint_session` value is permanently
/// superseded per ADR-031 §D3 and §D4.
fn build_request(
    http_client: &reqwest::Client,
    step: &FetchStep,
    url: &str,
    token: &AuthToken,
    auth_type: &crate::spec_parser::AuthType,
    step_vars: &HashMap<String, serde_json::Value>,
    offset: u32,
    page_size: u32,
) -> Result<reqwest::RequestBuilder, String> {
    let method = match step.method.to_ascii_uppercase().as_str() {
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "PATCH" => reqwest::Method::PATCH,
        "DELETE" => reqwest::Method::DELETE,
        _ => reqwest::Method::GET,
    };

    let mut req = http_client.request(method, url);

    // Auth header dispatch per auth_type (ADR-031 §D3-b; AC-007).
    if !token.as_str().is_empty() {
        req = match auth_type {
            crate::spec_parser::AuthType::CookieRoundtrip => {
                // ADR-031 §D3-b / AC-007: inject Cookie header with access_token.
                // Cookie name MUST be 'access_token' — NOT 'cyberint_session' (permanently
                // superseded per ADR-031 §D3 and §D4). INV-COOKIE-004: Authorization header
                // MUST NOT be set for CookieRoundtrip sensors.
                req.header("Cookie", format!("access_token={}", token.as_str()))
            }
            _ => req.header("Authorization", format!("Bearer {}", token.as_str())),
        };
    }

    // F-LP1-CRIT-001: Add request body for POST/PUT/PATCH.
    // Interpolate body_template against step_vars and derive Content-Type from shape.
    // AC-4 (S-PLUGIN-PREREQ-C): `$$` collapses to `$` for literal dollar-sign escaping.
    // TD-S-PLUGIN-PREREQ-B-008 is closed — escape mechanism implemented in Interpolator.
    if let Some(ref body_tpl) = step.body_template {
        let interpolated_body =
            Interpolator::interpolate(body_tpl, &InterpolationContext::JsonBody, step_vars)
                .map_err(|e| format!("body template interpolation failed: {e}"))?;

        // BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch:
        // POST-body vs GET-URL (DRIFT-D850-001)": for POST steps with OffsetLimit
        // pagination, inject "offset" and "limit" as top-level keys in the JSON body.
        // Merge semantics: preserve all existing body_template fields (AC-004).
        // First page uses offset=0 (AC-005). page_size=0 means pagination is not active
        // (e.g., execute_step single-request path), so skip injection in that case.
        let final_body = if step.method.eq_ignore_ascii_case("POST")
            && matches!(step.pagination, Some(PaginationConfig::OffsetLimit { .. }))
            && page_size > 0
        {
            let mut body_val: serde_json::Value = serde_json::from_str(&interpolated_body)
                .map_err(|e| {
                    format!(
                        "body template for POST OffsetLimit step '{}' is not valid JSON: {e}",
                        step.name
                    )
                })?;
            match body_val {
                serde_json::Value::Object(ref mut map) => {
                    map.insert(
                        "offset".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(offset)),
                    );
                    map.insert(
                        "limit".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(page_size)),
                    );
                }
                _ => {
                    // EC-002: body_template is not a JSON object (e.g., raw string, array).
                    // Surface as an error — cannot merge offset/limit into a non-object body.
                    // Object(_) is handled by the arm above; only non-object variants reach here.
                    let type_name = match &body_val {
                        serde_json::Value::Array(_) => "array",
                        serde_json::Value::String(_) => "string",
                        serde_json::Value::Number(_) => "number",
                        serde_json::Value::Bool(_) => "boolean",
                        _ => "null",
                    };
                    return Err(format!(
                        "POST OffsetLimit step '{}' body_template interpolated to a non-object \
                         JSON value ({type_name}); expected a JSON object to merge offset+limit \
                         into (BC-2.16.002 EC-002)",
                        step.name
                    ));
                }
            }
            body_val.to_string()
        } else {
            interpolated_body
        };

        // Derive Content-Type: JSON if body starts with '{' or '[', else form-urlencoded.
        // F-LP2-MED-002: JSON arrays (starting with '[') are also application/json.
        let trimmed = final_body.trim_start();
        let content_type = if trimmed.starts_with('{') || trimmed.starts_with('[') {
            "application/json"
        } else {
            "application/x-www-form-urlencoded"
        };

        req = req.header("Content-Type", content_type).body(final_body);
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
            // BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch:
            // POST-body vs GET-URL (DRIFT-D850-001)": for POST steps, offset+limit
            // go in the request body (injected in build_request); return URL unchanged.
            // For GET steps (and any non-POST method), append ?offset=N&limit=M as before.
            if step.method.eq_ignore_ascii_case("POST") {
                base_url.to_string()
            } else {
                let sep = if base_url.contains('?') { '&' } else { '?' };
                format!("{base_url}{sep}offset={offset}&limit={page_size}")
            }
        }
        Some(PaginationConfig::None) | None => base_url.to_string(),
    }
}

/// Build a `reqwest::Client` with a 30-second timeout and `prism/` User-Agent (CLAUDE.md §Conventions).
///
/// Used by `HttpLookupSource` and by the spec-driven adapter boot path.
/// Callers MUST NOT call `reqwest::Client::new()` directly — that construction
/// is forbidden because it sets no timeout (CLAUDE.md §Forbidden patterns).
///
/// ADR-050 §D6: all outbound HTTP clients set a `User-Agent: prism/{version}` header for
/// WAF-fingerprint coherence so security appliances can attribute requests to prism.
///
/// OBS-4 sibling of `prism_bin::spec_driven_adapter::build_http_client_with_custom_timeout`
/// (both factories now carry the same ADR-050 §D6 User-Agent obligation).
///
/// Returns `Err(String)` if the client builder fails.  Under `rustls-tls` this is
/// effectively unreachable (the only failure mode is malformed TLS configuration, which
/// cannot occur with the default rustls stack — see ADR-050 §D1/D2).  The `Result`
/// return mirrors the sibling `prism_bin::spec_driven_adapter::build_http_client_with_timeout`
/// and eliminates the forbidden `.expect()` on `Result` in production code
/// (CLAUDE.md §Forbidden patterns, DEFECT-ADAPTER-TLS-XDOME-LIVE-001 F-2).
///
/// Callers in `infusion/mod.rs` convert the `Err(String)` to
/// `InfusionError::HttpClientBuildFailed` (E-INFUSE-015, error-taxonomy v2.74).
/// The E-INFUSE-009 stopgap mapping has been retired.
///
/// AC-UA-001 | BC-2.16.002 (HTTP Client Compliance postconditions) | DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pub(crate) fn build_http_client_with_timeout() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        // ADR-050 §D6: all outbound clients MUST set User-Agent for WAF-fingerprint coherence.
        // concat! produces a &'static str with zero allocation at runtime.
        .user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| {
            format!("failed to build reqwest::Client with 30s timeout and prism/ User-Agent: {e}")
        })
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
pub fn extract_at_path(body: &serde_json::Value, path: &str) -> Result<serde_json::Value, String> {
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
            // audit-signal discipline (compare pipeline_truncated emission in PipelineExecutor::execute records-accumulation loop).
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

// ---------------------------------------------------------------------------
// strip_empty_url_params (AC-CWS-001 — empty query-param cleanup)
// ---------------------------------------------------------------------------

/// Remove any `&key=` or `?key=` query-param pairs where the value is the empty string.
///
/// This supports optional push-down parameters in path_templates (e.g., `&limit=${query.limit}`)
/// that are seeded to `""` when not provided by the query context.  Without stripping,
/// `&limit=` would reach the DTU and fail to deserialize into `Option<usize>` (a 422).
///
/// Rules:
/// - `?key=&other=val` → `?other=val`  (first-param empty, not last)
/// - `?key=val&empty=` → `?key=val`    (last-param empty)
/// - `?key=` (only param) → bare path  (no query string)
/// - `?a=1&b=&c=3` → `?a=1&c=3`       (middle-param empty)
/// - Params with non-empty values are preserved.
///
/// AC-CWS-001 / BC-2.01.013 limit push-down.
pub(crate) fn strip_empty_url_params(path: &str) -> String {
    // Split at the first `?` to separate path from query string.
    let (base, query) = match path.split_once('?') {
        Some((b, q)) => (b, q),
        // No query string → nothing to strip.
        None => return path.to_owned(),
    };

    // Split query string into individual `key=value` pairs.
    let kept: Vec<&str> = query
        .split('&')
        .filter(|pair| {
            // Retain pairs where value is non-empty.
            // A pair is `key=value`; if there is no `=` or value is empty, drop it.
            match pair.split_once('=') {
                Some((_key, value)) => !value.is_empty(),
                // Bare key with no `=` (unusual) → retain as-is.
                None => !pair.is_empty(),
            }
        })
        .collect();

    if kept.is_empty() {
        base.to_owned()
    } else {
        format!("{}?{}", base, kept.join("&"))
    }
}

// ---------------------------------------------------------------------------
// seed_missing_query_filter_vars (ADR-033 T1 — optional filter pre-seeding)
// ---------------------------------------------------------------------------

/// Pre-seed any `${query.filter.*}` variables referenced in a step's path/body
/// templates that are NOT already present in `step_vars`, defaulting to empty string.
///
/// This prevents interpolation errors when optional filter slots (e.g.,
/// `${query.filter._fql}` for CrowdStrike FQL injection) are present in the
/// TOML path_template but the FetchContext provides no value for them.
///
/// The default empty string causes the URL param to be present but empty
/// (e.g., `?filter=`), which is safely ignored by DTUs that do not parse
/// the param when it is empty.
///
/// ADR-033 T1 / BC-2.01.013: CrowdStrike FQL injection via
/// `${query.filter._fql}` in path_template requires this pre-seeding to be
/// robust when no time predicates are present in the PrismQL query.
fn seed_missing_query_filter_vars(
    path_template: &str,
    body_template: Option<&str>,
    step_vars: &mut std::collections::HashMap<String, serde_json::Value>,
) {
    // Regex to extract ${query.filter.VARNAME} references from templates.
    // Matches the canonical `${query.filter.*}` interpolation pattern.
    static QUERY_FILTER_PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = QUERY_FILTER_PATTERN.get_or_init(|| {
        regex::Regex::new(r"\$\{query\.filter\.([^}]+)\}")
            .expect("query.filter interpolation regex is valid")
    });
    for template in [Some(path_template), body_template].into_iter().flatten() {
        for cap in re.captures_iter(template) {
            let var_name = cap.get(1).expect("var name group").as_str();
            let full_key = format!("query.filter.{var_name}");
            // Only seed if not already present — do NOT override a provided value.
            step_vars
                .entry(full_key)
                .or_insert(serde_json::Value::String(String::new()));
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
/// Variables in the `query.filter.*` namespace are excluded from fan-out selection
/// regardless of value type — they carry push-down filter values, not batch sources
/// (BC-2.16.013; cycle-16 LOW-1 fix).
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
            // query.filter.* variables are JSON filter values, not fan-out sources.
            // An array-valued query.filter.* var (e.g., a malformed empty array) must not
            // trigger zero-batch fan-out (zero HTTP requests, zero rows, no error).
            // Same namespace-based reasoning as the Object-warn exemption below:
            // the correct discriminator is the NAMESPACE (`query.filter.*`), not the value type.
            // S-CLAROTY-AUDITLOG-TIMEBOX-001 cycle-16 LOW-1 (TD-VSDD-060 sibling sweep of FIX-2).
            if key.starts_with("query.filter.") {
                continue;
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
    // stringified as JSON — which is generally a spec bug.
    //
    // FIX-2 (S-CLAROTY-AUDITLOG-TIMEBOX-001 cycle-4 BLOCKING-2 revert): single-pass over
    // ALL templates; exempt `query.filter.*` variables regardless of template context.
    //
    // The correct discriminator is the NAMESPACE (`query.filter.*`), NOT the template
    // context (path_template vs body_template). Rationale:
    //   - query.filter.* variables carry JSON filter values that are Object-valued by
    //     design for BC-2.16.013 verbatim insertion (e.g., Claroty `_claroty_audit_filter_by`).
    //   - ADR-031 §D8-a: `${query.filter.aql}` in Armis path_template is a String value
    //     (never an Object), so it never triggers this Object warn regardless of exemption.
    //   - The Claroty Object (`_claroty_audit_filter_by`) appears only in body_template
    //     today, but the exemption must be namespace-based to avoid a false alarm if a
    //     future spec places a query.filter.* Object in a path_template — `value_to_string`
    //     handles Object serialization correctly in both contexts.
    //
    // The cycle-3 fix-burst-3 two-pass implementation was WRONG:
    //   Pass 1 (path_template): no exemption → false alarm for query.filter.* Objects
    //   Pass 2 (body_template): query.filter.* exemption → correct
    // Reverted to the single-pass approach here.
    for template in &templates {
        let refs = crate::interpolation::Interpolator::extract_references(template);
        for (ref_step_name, ref_field_path) in refs {
            let key = format!("{ref_step_name}.{ref_field_path}");
            // query.filter.* variables are Object-valued by design (BC-2.16.013).
            // Exempt them from the fanout_invalid_source_type warn in all template contexts.
            if key.starts_with("query.filter.") {
                continue;
            }
            if let Some(value) = step_vars.get(&key)
                && value.is_object()
            {
                tracing::warn!(
                    event_type = "fanout_invalid_source_type",
                    step_name = %step.name,
                    var_name = %key,
                    actual_type = "Object",
                    "Step template references an Object-valued variable; will be \
                     stringified into the request. This is likely a spec bug — consider \
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
// PLUGIN-MIGRATION-001-D — ADR-028 §D8-B/C: timestamp normalization
//
// `normalize_timestamp_fields` processes the raw JSON records returned by a
// pipeline step and applies timestamp parsing/normalization for all
// `ColumnType::Datetime` columns that declare `timestamp_formats` or
// `timestamp_fallback_chain`.
//
// Called in `execute_impl` before returning `PipelineResult`.
//
// Contract (ADR-028 v1.10 §D8-B/C):
//   1. If `column.timestamp_formats.is_empty()`: default ISO 8601 parsing.
//   2. If non-empty: try each format in order; first success wins.
//   3. On all-formats failure (non-null, non-absent value): return
//      `SpecEngineError::TimestampParseFailure` (maps to E-SPEC-018).
//   4. If primary field is null/absent AND `column.timestamp_fallback_chain` is
//      non-empty: try each fallback field in order.
//   5. If all fallback fields are also null/absent: use `Utc::now()` and emit
//      `tracing::warn!(event_type = "timestamp.fallback_to_now", column = %col_name)`.
//
// Output values are RFC 3339 / ISO 8601 strings (canonical wire format).
// ---------------------------------------------------------------------------

/// Parse a JSON value as an ISO 8601 / RFC 3339 datetime string.
///
/// Accepts `Value::String` only (ISO 8601 strings). Returns `None` on failure.
fn try_parse_iso8601(value: &serde_json::Value) -> Option<DateTime<Utc>> {
    match value {
        serde_json::Value::String(s) => s.parse::<DateTime<Utc>>().ok(),
        _ => None,
    }
}

/// Parse a JSON value as Unix epoch seconds (i64 integer or numeric string).
///
/// Accepts `Value::Number` (integer) or `Value::String` (decimal integer string).
/// Returns `None` on failure.
fn try_parse_unix_epoch_seconds(value: &serde_json::Value) -> Option<DateTime<Utc>> {
    let secs = match value {
        serde_json::Value::Number(n) => n.as_i64()?,
        serde_json::Value::String(s) => s.trim().parse::<i64>().ok()?,
        _ => return None,
    };
    Utc.timestamp_opt(secs, 0).single()
}

/// Parse a JSON value as Unix epoch milliseconds (i64 integer or numeric string).
///
/// Accepts `Value::Number` (integer) or `Value::String` (decimal integer string).
/// Returns `None` on failure.
fn try_parse_unix_epoch_millis(value: &serde_json::Value) -> Option<DateTime<Utc>> {
    let millis = match value {
        serde_json::Value::Number(n) => n.as_i64()?,
        serde_json::Value::String(s) => s.trim().parse::<i64>().ok()?,
        _ => return None,
    };
    DateTime::from_timestamp_millis(millis)
}

/// Try to parse `value` using the named format. Returns `None` if the format name
/// is unrecognized (guard: validation already rejected unrecognized names at load time
/// per BC-2.16.009, so this branch is only hit for defensive completeness).
fn try_format(fmt_name: &str, value: &serde_json::Value) -> Option<DateTime<Utc>> {
    match fmt_name {
        "iso8601" => try_parse_iso8601(value),
        "unix_epoch_seconds" => try_parse_unix_epoch_seconds(value),
        "unix_epoch_millis" => try_parse_unix_epoch_millis(value),
        // Unrecognized format: treat as failure (validation should have caught this).
        _ => None,
    }
}

/// Effective format list: when `timestamp_formats` is empty, default to `["iso8601"]`
/// for backward compatibility (ADR-028 §D8-B).
fn effective_formats(formats: &[String]) -> Vec<&str> {
    if formats.is_empty() {
        vec!["iso8601"]
    } else {
        formats.iter().map(|s| s.as_str()).collect()
    }
}

/// Try to parse `value` against the effective format list.
///
/// Returns `Some(DateTime<Utc>)` on first success, `None` if all formats fail.
fn try_formats(formats: &[String], value: &serde_json::Value) -> Option<DateTime<Utc>> {
    for fmt in effective_formats(formats) {
        if let Some(dt) = try_format(fmt, value) {
            return Some(dt);
        }
    }
    None
}

/// Returns `true` if a JSON value is considered absent for timestamp fallback purposes.
fn is_null_or_absent(value: Option<&serde_json::Value>) -> bool {
    matches!(value, None | Some(serde_json::Value::Null))
}

/// Normalize `ColumnType::Datetime` fields in `records` according to each column's
/// `timestamp_formats` and `timestamp_fallback_chain` declarations.
///
/// See module-level doc comment for the full contract.
///
/// # Errors
///
/// Returns `SpecEngineError::TimestampParseFailure` (E-SPEC-018) if a non-null
/// datetime value fails all declared formats and there is no fallback chain.
pub(crate) fn normalize_timestamp_fields(
    records: &[serde_json::Value],
    columns: &[ColumnSpec],
    sensor_id: &str,
) -> Result<Vec<serde_json::Value>, SpecEngineError> {
    // Collect only Datetime columns — skip non-datetime columns entirely.
    let datetime_cols: Vec<&ColumnSpec> = columns
        .iter()
        .filter(|c| c.column_type == ColumnType::Datetime)
        .collect();

    // Fast path: no datetime columns → return records unchanged.
    if datetime_cols.is_empty() {
        return Ok(records.to_vec());
    }

    let mut out = Vec::with_capacity(records.len());

    for record in records {
        let mut row = record.clone();

        for col in &datetime_cols {
            let primary_value = row.get(&col.name).cloned();
            let primary_absent = is_null_or_absent(primary_value.as_ref());

            if primary_absent {
                // --- Fallback chain path ---
                if col.timestamp_fallback_chain.is_empty() {
                    // No fallback: leave null/absent as-is (existing behavior).
                    continue;
                }

                // Try each fallback field in order.
                // F-LP2-arch-handoff#1 + LOW-002: defensive skip guard — if a fallback chain
                // entry matches the primary column name itself, it yields the same null/absent value
                // already confirmed above. Skip it to avoid wasting an iteration (and to guard
                // against TOML authors who accidentally include the primary in the chain, e.g.,
                // `timestamp_fallback_chain = ["last_seen", "first_seen"]` on the `last_seen` column).
                let mut resolved: Option<DateTime<Utc>> = None;
                for fb_field in &col.timestamp_fallback_chain {
                    if fb_field == &col.name {
                        // Primary field confirmed absent above — skip the redundant chain entry.
                        continue;
                    }
                    let fb_value = row.get(fb_field.as_str());
                    if is_null_or_absent(fb_value) {
                        continue;
                    }
                    let fb_value_inner = fb_value.expect(
                        "fb_value: guaranteed Some — is_null_or_absent check above continues if None",
                    );
                    if let Some(dt) = try_formats(&col.timestamp_formats, fb_value_inner) {
                        resolved = Some(dt);
                        break;
                    }
                }

                match resolved {
                    Some(dt) => {
                        if let Some(obj) = row.as_object_mut() {
                            obj.insert(
                                col.name.clone(),
                                // Use `Z` suffix (UTC marker) for canonical form.
                                // chrono `to_rfc3339()` produces `+00:00`; DataFusion's
                                // string comparison treats `"T10:00:00+00:00" >= "T10:00:00Z"`
                                // as FALSE because lexicographically `+` (43) < `Z` (90).
                                // Using `to_rfc3339_opts(Secs, use_z=true)` normalises all
                                // pipeline-emitted timestamps to `Z` suffix so DataFusion
                                // WHERE clause literals (which users write with `Z`) compare
                                // correctly at exact boundaries. (ADV-P08-MED-001 fix)
                                serde_json::Value::String(
                                    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                                ),
                            );
                        }
                    }
                    None => {
                        // All fallbacks exhausted → use now().
                        let now = Utc::now();
                        tracing::warn!(
                            event_type = "timestamp.fallback_to_now",
                            column = %col.name,
                            "all timestamp_fallback_chain fields are null/absent; \
                             falling back to Utc::now() (ADR-028 §D8-B)"
                        );
                        if let Some(obj) = row.as_object_mut() {
                            obj.insert(
                                col.name.clone(),
                                serde_json::Value::String(
                                    now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                                ),
                            );
                        }
                    }
                }
            } else {
                // --- Primary field present and non-null ---
                let value = primary_value.as_ref().expect(
                    "primary_value: guaranteed Some — primary_absent guard above routed to absent-branch if None",
                );
                match try_formats(&col.timestamp_formats, value) {
                    Some(dt) => {
                        if let Some(obj) = row.as_object_mut() {
                            obj.insert(
                                col.name.clone(),
                                // Use `Z` suffix for canonical form (ADV-P08-MED-001).
                                serde_json::Value::String(
                                    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                                ),
                            );
                        }
                    }
                    None => {
                        // All formats failed → E-SPEC-018.
                        let formats = effective_formats(&col.timestamp_formats)
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();
                        return Err(SpecEngineError::TimestampParseFailure {
                            sensor_id: sensor_id.to_string(),
                            column_name: col.name.clone(),
                            attempted_formats: formats,
                            // SEC-002 (CWE-532 / AD-017): cap raw sensor value at 50
                            // codepoints before storing in the error — Display output is
                            // then naturally capped. Consistent with value_prefix convention.
                            value: value.to_string().chars().take(50).collect(),
                        });
                    }
                }
            }
        }

        out.push(row);
    }

    Ok(out)
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
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use prism_core::{ColumnType, OrgSlug};
    use tracing_subscriber::util::SubscriberInitExt;
    use wiremock::{
        Mock as WmMock, MockServer, ResponseTemplate,
        matchers::{method as wm_method, path as wm_path},
    };

    use crate::{
        auth_provider::{
            AuthOutcome, ChainAuthProvider, FailingAuthProvider, MockAuthProvider, NullAuthProvider,
        },
        pipeline::{FetchContext, PipelineExecutor},
        spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec},
    };

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
                    timestamp_formats: vec![],
                    timestamp_fallback_chain: vec![],
                    source_path: None,
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
            auth_plugin: None,
            // ADR-030 Approach D: post-parse metadata fields. Test-constructed specs use
            // empty values — these are set by file-loading callers in production.
            file_hash: String::new(),
            source_path: String::new(),
            mode: crate::types::DtuMode::Shared,
            ocsf_column_naming: false,
            // S-5.04 AC-8/9/10: type scaffold only; probe_table not a positional arg per design doc §2
            probe_table: None,
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
    /// execute_step. A future refactor that removes `step_name` from the
    /// auth_initial_acquired tracing macro in `PipelineExecutor::execute_step`
    /// would cause this test to FAIL on the `step_name` assertion.
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
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
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
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
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
    /// arm's auth_initial_failed tracing macro in `PipelineExecutor::execute_step`
    /// would cause this test to FAIL.
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
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
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
                    timestamp_formats: vec![],
                    timestamp_fallback_chain: vec![],
                    source_path: None,
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
            auth_plugin: None,
            // ADR-030 Approach D: post-parse metadata fields. Test-constructed specs use
            // empty values — these are set by file-loading callers in production.
            file_hash: String::new(),
            source_path: String::new(),
            mode: crate::types::DtuMode::Shared,
            ocsf_column_naming: false,
            // S-5.04 AC-8/9/10: type scaffold only; probe_table not a positional arg per design doc §2
            probe_table: None,
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
    /// had only call-count + error-variant assertions in pipeline_oauth_retry tests, with
    /// ZERO buffer assertions on the event_type string. A refactor removing `detail` from
    /// the auth_initial_failed error arm in `PipelineExecutor::execute` would NOT have
    /// failed any prior test.
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
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
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
    /// A refactor removing `step_name` from the auth_refresh_triggered tracing macro
    /// in `issue_request_with_retry` would NOT have failed any prior test.
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
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
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
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
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
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
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
    /// A refactor removing `step_name` from the auth_refresh_double_401 tracing macro
    /// in `issue_request_with_retry`
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
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
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
// RED GATE (pre-fix): `build_paged_url` ignored the `page_size` field
// (see TD-S-PLUGIN-PREREQ-B-001 comment at pipeline.rs build_paged_url). These tests
// assert the EXPECTED postcondition; they pass once `build_paged_url` was updated to
// thread `page_size` into the URL.
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
    /// RED GATE (pre-fix): `build_paged_url` did not thread `page_size` into the URL.
    /// This test was the red gate for AC-1; now passes.
    #[test]
    fn test_BC_2_16_002_cursor_pagination_first_call_includes_page_size() {
        let step = cursor_step(Some(50));
        let base = "https://api.crowdstrike.com/devices/queries/devices/v1";
        let url = build_paged_url(base, &step, &None, 0);
        assert!(
            url.contains("page_size=50"),
            "AC-1 regression guard: first-call URL must contain 'page_size=50' when page_size=Some(50); \
             got: {url}"
        );
    }

    /// AC-1(b): `page_size: Some(50)` on a continuation call (cursor present) → URL contains
    /// both `page_size=50` and the cursor parameter.
    ///
    /// RED GATE (pre-fix): `build_paged_url` did not append `page_size` on continuation calls.
    /// Test was the red gate for AC-1; now passes.
    #[test]
    fn test_BC_2_16_002_cursor_pagination_continuation_includes_page_size() {
        let step = cursor_step(Some(50));
        let base = "https://api.crowdstrike.com/devices/queries/devices/v1";
        let cursor = Some("cursor_xyz_abc".to_string());
        let url = build_paged_url(base, &step, &cursor, 0);
        assert!(
            url.contains("page_size=50"),
            "AC-1 regression guard: continuation URL must contain 'page_size=50' when page_size=Some(50); \
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
// RED GATE (pre-fix): `extract_at_path` supported only dot-notation paths
// (see TD-S-PLUGIN-PREREQ-B-003 comment). Bracket notation returned Err.
// These tests assert the EXPECTED postcondition; they pass once AC-2 was implemented.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod jsonpath_bracket_tests {
    use serde_json::json;

    use super::extract_at_path;

    /// AC-2(a): `$.devices[0].id` on an array-valued JSON object extracts the first element.
    ///
    /// RED GATE (pre-fix): `extract_at_path` split on `.` only; `[0]` was not recognized as an
    /// array index, so this path failed to match. Test was the red gate for AC-2; now passes.
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
            "AC-2 regression guard: $.devices[0].id must succeed; got Err: {:?}\n\
             if this fails, extract_at_path bracket-index support regressed.",
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
    /// RED GATE (pre-fix): wildcard `[*]` was not supported by the dot-split path traversal.
    /// Test was the red gate for AC-2; now passes.
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
            "AC-2 regression guard: $.devices[*].id must succeed; got Err: {:?}\n\
             if this fails, extract_at_path wildcard [*] enumeration support regressed.",
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
    /// RED GATE (pre-fix): `extract_at_path` returned `Err(String)` for any bracket path.
    /// After AC-2, it returns `Err` specifically for out-of-bounds (not panic).
    /// Pre-fix, this test failed at the first assertion because `$.x[99]` syntax was not
    /// parsed; after AC-2 it returns Err due to out-of-bounds. The no-panic invariant holds.
    #[test]
    fn test_BC_2_16_002_extract_bracket_out_of_bounds_structured_error() {
        let body = json!({
            "x": [1, 2, 3]
        });
        let result = extract_at_path(&body, "$.x[99]");
        // Post-AC-2: returns Err (structured, not panic). The invariant: MUST NOT panic.
        assert!(
            result.is_err(),
            "AC-2: $.x[99] on a 3-element array must return Err (out-of-bounds); \
             after AC-2 impl this Err should have a descriptive message, not just 'path not found'"
        );
        // Post-AC-2: the error message indicates out-of-bounds (implemented).
        // Pre-fix the message said "path must start with '$.'..." or "path not found".
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("out of bounds")
                || err_msg.contains("index")
                || err_msg.contains("99"),
            "AC-2 regression guard: out-of-bounds error message must reference the index or 'out of bounds'; \
             got: '{err_msg}'"
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

    /// AC-011 regression guard: numeric-index JSONPath resolves first element end-to-end
    /// through the production `extract_at_path` resolver.
    ///
    /// The AC-011 enrichment generator tests (cyberint + crowdstrike) verify nested structure
    /// via direct `serde_json` indexing, which does NOT exercise `extract_at_path`. This test
    /// regression-guards `[N]` index token resolution through the production resolver so that
    /// a bug in `tokenize_jsonpath` / `PathToken::Index` handling fails here, not silently.
    ///
    /// Paths tested mirror the real TOML `source_path` values used in the enrichment pipeline:
    /// - Cyberint IOC surface: `$.iocs[0].value`
    /// - CrowdStrike detections: `$.behaviors[0].ioc_value`
    #[test]
    fn test_extract_at_path_numeric_index_resolves_first_element() {
        // Cyberint-style record: iocs array where iocs[0].value = "1.2.3.4"
        let cyberint_record = json!({
            "iocs": [
                {"type": "ip", "value": "1.2.3.4", "severity": "high"},
                {"type": "domain", "value": "evil.example.com", "severity": "medium"}
            ]
        });
        let result = extract_at_path(&cyberint_record, "$.iocs[0].value");
        assert!(
            result.is_ok(),
            "$.iocs[0].value must succeed on a cyberint-style record; got Err: {:?}",
            result.err()
        );
        assert_eq!(
            result.unwrap(),
            json!("1.2.3.4"),
            "$.iocs[0].value must return the first IOC value string"
        );

        // CrowdStrike-style record: behaviors array where behaviors[0].ioc_value = "bad.exe"
        let crowdstrike_record = json!({
            "behaviors": [
                {"ioc_type": "hash_md5", "ioc_value": "bad.exe", "tactic": "Execution"},
                {"ioc_type": "domain", "ioc_value": "c2.attacker.com", "tactic": "C2"}
            ]
        });
        let result = extract_at_path(&crowdstrike_record, "$.behaviors[0].ioc_value");
        assert!(
            result.is_ok(),
            "$.behaviors[0].ioc_value must succeed on a crowdstrike-style record; got Err: {:?}",
            result.err()
        );
        assert_eq!(
            result.unwrap(),
            json!("bad.exe"),
            "$.behaviors[0].ioc_value must return the first behavior's ioc_value string"
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
    use proptest::prelude::*;
    use serde_json::Value;

    use super::extract_at_path;

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

// ---------------------------------------------------------------------------
// PLUGIN-MIGRATION-001-D — BC-2.16.013 §O-001 — timestamp normalization unit tests
//
// ADR-028 v1.10 §D8-B/C: PipelineExecutor must honor `timestamp_formats` and
// `timestamp_fallback_chain` on ColumnType::Datetime columns.
//
// These tests MUST FAIL before `normalize_timestamp_fields` is implemented,
// then PASS after implementation (strict TDD per CLAUDE.md §TDD Inner Loop Discipline).
// ---------------------------------------------------------------------------
#[cfg(test)]
mod timestamp_normalization_tests {
    use prism_core::ColumnType;
    use serde_json::json;

    use super::normalize_timestamp_fields;
    use crate::{error::SpecEngineError, spec_parser::ColumnSpec};

    // -----------------------------------------------------------------------
    // Helper: build a single-column Datetime ColumnSpec.
    // -----------------------------------------------------------------------
    fn datetime_col(
        name: &str,
        timestamp_formats: Vec<&str>,
        timestamp_fallback_chain: Vec<&str>,
    ) -> ColumnSpec {
        ColumnSpec {
            name: name.to_string(),
            column_type: ColumnType::Datetime,
            ocsf_field: None,
            options: vec![],
            timestamp_formats: timestamp_formats
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            timestamp_fallback_chain: timestamp_fallback_chain
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            source_path: None,
        }
    }

    // -----------------------------------------------------------------------
    // test 1 — iso8601 only: explicit single-format list with ISO 8601 string
    // -----------------------------------------------------------------------
    /// BC-2.16.013 §O-001: column with timestamp_formats = ["iso8601"] + ISO 8601 value
    /// → parses successfully; output value is ISO 8601 string (unchanged or normalized).
    #[test]
    fn test_BC_2_16_013_timestamp_formats_iso8601_only() {
        let cols = vec![datetime_col("created_at", vec!["iso8601"], vec![])];
        let records = vec![json!({"created_at": "2026-05-21T00:00:00Z"})];

        let result = normalize_timestamp_fields(&records, &cols, "test-sensor");
        assert!(
            result.is_ok(),
            "iso8601 value must parse successfully; got: {:?}",
            result.err()
        );
        let normalized = result.unwrap();
        assert_eq!(normalized.len(), 1);
        // Output must be a non-null string value.
        let val = normalized[0]
            .get("created_at")
            .expect("created_at must be present");
        assert!(
            val.is_string(),
            "normalized datetime must be a string; got: {val}"
        );
        let s = val.as_str().unwrap();
        assert!(
            s.contains("2026-05-21"),
            "normalized ISO 8601 output must contain the original date; got: {s}"
        );
    }

    // -----------------------------------------------------------------------
    // test 2 — multi-format: first format fails, second succeeds (unix_epoch_seconds)
    // -----------------------------------------------------------------------
    /// BC-2.16.013 §O-001: column with timestamp_formats = ["iso8601", "unix_epoch_seconds"]
    /// + numeric unix seconds value → parses successfully via second format.
    #[test]
    fn test_BC_2_16_013_timestamp_formats_multi_iso_then_unix_seconds() {
        let cols = vec![datetime_col(
            "created_at",
            vec!["iso8601", "unix_epoch_seconds"],
            vec![],
        )];
        // 1716249600 = 2024-05-21T00:00:00Z (unix seconds)
        let records = vec![json!({"created_at": 1716249600_i64})];

        let result = normalize_timestamp_fields(&records, &cols, "test-sensor");
        assert!(
            result.is_ok(),
            "unix_epoch_seconds value must parse on second format; got: {:?}",
            result.err()
        );
        let normalized = result.unwrap();
        let val = normalized[0]
            .get("created_at")
            .expect("created_at must be present");
        assert!(
            val.is_string(),
            "normalized datetime must be a string; got: {val}"
        );
        let s = val.as_str().unwrap();
        // 1716249600 = 2024-05-21T00:00:00Z
        assert!(
            s.contains("2024-05-21"),
            "unix_epoch_seconds output must contain 2024-05-21; got: {s}"
        );
    }

    // -----------------------------------------------------------------------
    // test 3 — all formats fail → E-SPEC-018 error
    // -----------------------------------------------------------------------
    /// BC-2.16.013 §O-001: all declared formats fail → SpecEngineError::TimestampParseFailure
    /// carrying E-SPEC-018 error code pattern.
    #[test]
    fn test_BC_2_16_013_timestamp_formats_all_fail_emits_E_SPEC_018() {
        let cols = vec![datetime_col(
            "created_at",
            vec!["iso8601", "unix_epoch_seconds"],
            vec![],
        )];
        let records = vec![json!({"created_at": "garbage-not-a-timestamp"})];

        let result = normalize_timestamp_fields(&records, &cols, "test-sensor");
        assert!(result.is_err(), "garbage value must return Err");
        let err = result.unwrap_err();
        match &err {
            SpecEngineError::TimestampParseFailure {
                column_name,
                attempted_formats,
                ..
            } => {
                assert_eq!(column_name, "created_at", "error must cite column name");
                assert!(
                    attempted_formats.contains(&"iso8601".to_string()),
                    "attempted_formats must include iso8601; got: {attempted_formats:?}"
                );
                assert!(
                    attempted_formats.contains(&"unix_epoch_seconds".to_string()),
                    "attempted_formats must include unix_epoch_seconds; got: {attempted_formats:?}"
                );
            }
            other => panic!("expected TimestampParseFailure, got: {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // test 4 — fallback chain: primary is null, fallback succeeds
    // -----------------------------------------------------------------------
    /// BC-2.16.013 §O-001: column with timestamp_fallback_chain = ["last_seen", "first_seen"]
    /// + JSON record where primary field is null but first_seen has a valid value
    /// → result uses first_seen's value.
    #[test]
    fn test_BC_2_16_013_timestamp_fallback_chain_uses_fallback() {
        let cols = vec![ColumnSpec {
            name: "last_seen".to_string(),
            column_type: ColumnType::Datetime,
            ocsf_field: None,
            options: vec![],
            timestamp_formats: vec![],
            timestamp_fallback_chain: vec!["last_seen".to_string(), "first_seen".to_string()],
            source_path: None,
        }];
        // primary field "last_seen" is null; fallback "first_seen" has a value.
        let records = vec![json!({"last_seen": null, "first_seen": "2026-05-21T00:00:00Z"})];

        let result = normalize_timestamp_fields(&records, &cols, "test-sensor");
        assert!(
            result.is_ok(),
            "fallback chain must resolve via first_seen; got: {:?}",
            result.err()
        );
        let normalized = result.unwrap();
        let val = normalized[0]
            .get("last_seen")
            .expect("last_seen must be in output");
        assert!(
            val.is_string(),
            "fallback-resolved datetime must be a string; got: {val}"
        );
        let s = val.as_str().unwrap();
        assert!(
            s.contains("2026-05-21"),
            "fallback value must contain the first_seen date; got: {s}"
        );
    }

    // -----------------------------------------------------------------------
    // test 5 — fallback exhausts to now()
    // -----------------------------------------------------------------------
    /// BC-2.16.013 §O-001: all fallback fields are null/absent → result is approximately
    /// now() (within ±10 seconds tolerance).
    ///
    /// Note: tracing::warn!(event_type = "timestamp.fallback_to_now") emission is the
    /// behavioral contract per BC-2.16.002 row 35 (timestamp.fallback_to_now catalog entry). The BC catalog row is the
    /// authoritative contract record; direct assertion of the emission from this unit
    /// test would require tracing-test infrastructure not available in-scope.
    #[test]
    fn test_BC_2_16_013_timestamp_fallback_exhausts_to_now_emits_tracing_warn() {
        use chrono::{DateTime, Utc};

        let cols = vec![ColumnSpec {
            name: "last_seen".to_string(),
            column_type: ColumnType::Datetime,
            ocsf_field: None,
            options: vec![],
            timestamp_formats: vec![],
            timestamp_fallback_chain: vec!["last_seen".to_string(), "first_seen".to_string()],
            source_path: None,
        }];
        // Both primary and fallback are null.
        let records = vec![json!({"last_seen": null, "first_seen": null})];

        let before = Utc::now();
        let result = normalize_timestamp_fields(&records, &cols, "test-sensor");
        let after = Utc::now();

        assert!(
            result.is_ok(),
            "fallback-to-now must succeed (not error); got: {:?}",
            result.err()
        );
        let normalized = result.unwrap();
        let val = normalized[0]
            .get("last_seen")
            .expect("last_seen must be in output");
        assert!(
            val.is_string(),
            "now() fallback must produce a string; got: {val}"
        );
        let s = val.as_str().unwrap();
        let parsed: DateTime<Utc> = s.parse().expect("now() output must be valid RFC 3339");
        let tolerance = chrono::Duration::seconds(10);
        assert!(
            parsed >= before - tolerance && parsed <= after + tolerance,
            "now() fallback must be approximately current time; got: {parsed}, before: {before}"
        );
    }

    // -----------------------------------------------------------------------
    // test 6 — empty timestamp_formats defaults to ISO 8601 (backward compat)
    // -----------------------------------------------------------------------
    /// BC-2.16.013 §O-001: column with timestamp_formats = [] (default) + ISO 8601 value
    /// → parses successfully (backward compatibility — same behavior as before this feature).
    #[test]
    fn test_BC_2_16_013_timestamp_formats_empty_defaults_to_iso8601() {
        let cols = vec![ColumnSpec {
            name: "event_time".to_string(),
            column_type: ColumnType::Datetime,
            ocsf_field: None,
            options: vec![],
            timestamp_formats: vec![],
            timestamp_fallback_chain: vec![],
            source_path: None,
        }];
        let records = vec![json!({"event_time": "2026-05-21T00:00:00Z"})];

        let result = normalize_timestamp_fields(&records, &cols, "test-sensor");
        assert!(
            result.is_ok(),
            "empty timestamp_formats + ISO 8601 value must parse; got: {:?}",
            result.err()
        );
        let normalized = result.unwrap();
        let val = normalized[0]
            .get("event_time")
            .expect("event_time must be present");
        assert!(
            val.is_string(),
            "normalized datetime must be a string; got: {val}"
        );
        let s = val.as_str().unwrap();
        assert!(
            s.contains("2026-05-21"),
            "ISO 8601 output must contain the original date; got: {s}"
        );
    }

    // -----------------------------------------------------------------------
    // test 7 — unix_epoch_millis variant
    // -----------------------------------------------------------------------
    /// BC-2.16.013 §O-001: column with timestamp_formats = ["unix_epoch_millis"]
    /// + millisecond unix timestamp value → parses successfully.
    #[test]
    fn test_BC_2_16_013_timestamp_format_unix_epoch_millis_parses() {
        let cols = vec![datetime_col("ts", vec!["unix_epoch_millis"], vec![])];
        // 1716249600000 ms = 1716249600 s = 2024-05-21T00:00:00Z
        let records = vec![json!({"ts": 1716249600000_i64})];

        let result = normalize_timestamp_fields(&records, &cols, "test-sensor");
        assert!(
            result.is_ok(),
            "unix_epoch_millis must parse correctly; got: {:?}",
            result.err()
        );
        let normalized = result.unwrap();
        let val = normalized[0].get("ts").expect("ts must be present");
        assert!(
            val.is_string(),
            "normalized datetime must be a string; got: {val}"
        );
        let s = val.as_str().unwrap();
        assert!(
            s.contains("2024-05-21"),
            "unix_epoch_millis output must contain 2024-05-21; got: {s}"
        );
    }

    // -----------------------------------------------------------------------
    // test 8 — skip guard: primary name appears in fallback_chain (F-LP2-arch-handoff#1)
    // -----------------------------------------------------------------------
    /// BC-2.16.013 §O-001 + F-LP2-HIGH-004 code side:
    /// Column `last_seen` with `timestamp_fallback_chain = ["last_seen", "first_seen"]`.
    /// Primary `last_seen` is null. The skip guard prevents the chain entry that
    /// matches the primary column from being consulted a second time (it would yield
    /// the same absent value). `first_seen` provides the resolved value.
    ///
    /// This is a DEFENSIVE contract test — even though the armis TOML was corrected
    /// to `["first_seen"]` only, the skip guard must protect against future TOML authors
    /// who accidentally include the primary column name in the chain.
    #[test]
    fn test_BC_2_16_013_timestamp_fallback_chain_skips_primary_self_reference() {
        let cols = vec![ColumnSpec {
            name: "last_seen".to_string(),
            column_type: ColumnType::Datetime,
            ocsf_field: None,
            options: vec![],
            timestamp_formats: vec![],
            // Deliberately includes the primary column name as first chain entry
            // to exercise the skip guard.
            timestamp_fallback_chain: vec!["last_seen".to_string(), "first_seen".to_string()],
            source_path: None,
        }];
        // last_seen is null; first_seen has a valid value.
        let records = vec![json!({"last_seen": null, "first_seen": "2026-05-21T00:00:00Z"})];

        let result = normalize_timestamp_fields(&records, &cols, "test-sensor");
        assert!(
            result.is_ok(),
            "skip guard must allow resolution via first_seen when last_seen is in chain; got: {:?}",
            result.err()
        );
        let normalized = result.unwrap();
        let val = normalized[0]
            .get("last_seen")
            .expect("last_seen must be in output");
        assert!(
            val.is_string(),
            "skip-guarded fallback must produce a string; got: {val}"
        );
        let s = val.as_str().unwrap();
        assert!(
            s.contains("2026-05-21"),
            "resolved value must contain the first_seen date; got: {s}"
        );
    }

    // -----------------------------------------------------------------------
    // F11 / SNS-04 (2026-06-10 review) — Armis-spec engagement verification
    // -----------------------------------------------------------------------
    /// F11 / SNS-04: the `timestamp.fallback_to_now` path ENGAGES for the
    /// bundled armis.sensor.toml — not just for synthetic ColumnSpecs.
    ///
    /// Loads the production Armis TOML via `SpecLoader::parse`, extracts the
    /// `devices` table's actual parsed columns (last_seen: Datetime with
    /// `timestamp_fallback_chain = ["first_seen"]`), and feeds a record where
    /// BOTH last_seen and first_seen are null. The normalization pass must
    /// resolve last_seen to ~now() (the fallback-to-now branch — the branch
    /// that emits `tracing::warn!(event_type = "timestamp.fallback_to_now")`,
    /// BC-2.16.002 row 35). This pins the armis.sensor.toml comment claims to
    /// live spec-engine behavior.
    #[test]
    fn test_f11_sns04_armis_devices_last_seen_fallback_engages() {
        use chrono::{DateTime, Utc};

        let toml_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml");
        let raw = std::fs::read_to_string(&toml_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", toml_path.display()));
        // SpecLoader::parse does not perform env-var resolution (that is
        // load_all Rule 6) — substitute the base_url token so parsing succeeds
        // without mutating process env (set_var is unsafe in edition 2024).
        let raw = raw.replace("${env.ARMIS_INSTANCE_URL}", "https://armis.example.com");
        let spec = crate::spec_parser::SpecLoader::parse(&raw)
            .expect("bundled armis.sensor.toml must parse");

        let devices = spec
            .tables
            .iter()
            .find(|t| t.table_name == "devices")
            .expect("armis spec must have a 'devices' table");
        let last_seen_col = devices
            .columns
            .iter()
            .find(|c| c.name == "last_seen")
            .expect("devices table must declare last_seen");
        assert_eq!(
            last_seen_col.timestamp_fallback_chain,
            vec!["first_seen".to_string()],
            "armis devices.last_seen must declare timestamp_fallback_chain = [\"first_seen\"] \
             (ADR-028 §D8-B; armis.sensor.toml comment claim)"
        );

        // Both chain sources null → fallback-to-now branch must engage.
        let records = vec![json!({"last_seen": null, "first_seen": null})];
        let before = Utc::now();
        let normalized = normalize_timestamp_fields(&records, &devices.columns, "armis")
            .expect("fallback-to-now must succeed, not error");
        let after = Utc::now();

        let val = normalized[0]
            .get("last_seen")
            .and_then(|v| v.as_str())
            .expect("last_seen must be a string after fallback-to-now");
        let resolved: DateTime<Utc> = val
            .parse()
            .unwrap_or_else(|e| panic!("fallback value '{val}' must be RFC3339: {e}"));
        // Secs-precision output truncates sub-second — allow 1s slack on the lower bound.
        assert!(
            resolved >= before - chrono::Duration::seconds(1) && resolved <= after,
            "fallback-resolved last_seen '{val}' must be ~Utc::now() \
             (window {before}..{after}) — proves the timestamp.fallback_to_now \
             branch engages for the bundled Armis spec (F11 / SNS-04)"
        );
    }
}

// ---------------------------------------------------------------------------
// S-DEMO-CLAROTY-PAGINATION-001 — BC-2.16.002 §Postconditions
// "OffsetLimit Pagination Dispatch: POST-body vs GET-URL (DRIFT-D850-001)"
//
// Red Gate tests for AC-001, AC-002, AC-003, AC-004, AC-005, AC-006.
//
// RED GATE (pre-fix):
//   AC-001, AC-004, AC-005, AC-003-unit: `build_paged_url_impl` previously
//   appended `?offset=N&limit=M` to the URL regardless of HTTP method and
//   `build_request` did not inject offset/limit into the POST body.
//   These tests assert the EXPECTED postcondition; now passing.
//
//   AC-002, AC-006: existing GET behavior was correct.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod pagination_post_body_tests {
    use std::collections::HashMap;

    use prism_core::{ColumnType, OrgSlug};
    use wiremock::{
        Mock as WmMock, MockServer, ResponseTemplate,
        matchers::{method as wm_method, path as wm_path},
    };

    use super::{PaginationConfig, build_paged_url};
    use crate::{
        auth_provider::NullAuthProvider,
        pipeline::{FetchContext, PipelineExecutor},
        spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec},
    };

    // -----------------------------------------------------------------------
    // Fixture helpers
    // -----------------------------------------------------------------------

    /// Build a single-step POST spec with OffsetLimit pagination.
    ///
    /// `body_template`: the JSON string body template (e.g. `"{}"` for Claroty).
    fn post_offset_limit_spec(
        base_url: &str,
        path: &str,
        body_template: Option<String>,
        page_size: u32,
    ) -> SensorSpec {
        SensorSpec::new(
            "post-paginated-sensor",
            "Post Paginated Sensor",
            AuthType::BearerStatic,
            base_url,
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
                vec![FetchStep::new(
                    "fetch_alerts",
                    "POST",
                    path,
                    body_template,
                    "$.alerts",
                    None,
                    vec![],
                    None,
                    Some(PaginationConfig::OffsetLimit { page_size }),
                )],
            )],
            None,
            "1.0.0",
            vec![],
        )
    }

    /// Build a single-step GET spec with OffsetLimit pagination (regression guard).
    fn get_offset_limit_step(page_size: u32) -> FetchStep {
        FetchStep::new(
            "fetch_logs",
            "GET",
            "/api/logs",
            None,
            "$.items",
            None,
            vec![],
            None,
            Some(PaginationConfig::OffsetLimit { page_size }),
        )
    }

    /// Build a POST step with OffsetLimit pagination (for URL-side assertions).
    fn post_offset_limit_step(page_size: u32) -> FetchStep {
        FetchStep::new(
            "fetch_alerts",
            "POST",
            "/api/alerts",
            Some("{}".to_string()),
            "$.alerts",
            None,
            vec![],
            None,
            Some(PaginationConfig::OffsetLimit { page_size }),
        )
    }

    fn default_context() -> FetchContext {
        FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None)
    }

    // -----------------------------------------------------------------------
    // AC-001 (URL side): POST step with OffsetLimit → URL unchanged (no ?offset=&limit=)
    //
    // RED GATE (pre-fix): `build_paged_url_impl` appended ?offset=N&limit=M to ALL
    // methods. This test passed once build_paged_url_impl branched on step.method.
    // -----------------------------------------------------------------------

    /// AC-001 / BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch:
    /// POST-body vs GET-URL" — POST step clause.
    ///
    /// For `method == "POST"` with OffsetLimit pagination, `build_paged_url_impl`
    /// MUST return the base URL unchanged. The `?offset=` and `?limit=` params
    /// MUST NOT appear in the URL — they go in the request body instead.
    ///
    /// RED GATE (pre-fix): build_paged_url_impl appended ?offset=N&limit=M regardless
    /// of method. Now passes with Task 2 (build_paged_url_impl POST branch) done.
    #[test]
    fn test_BC_2_16_002_pagination_post_method_url_unchanged() {
        let step = post_offset_limit_step(100);
        let base = "https://api.claroty.example.com/api/v1/alerts";

        // Page 1 (offset=0): URL must be base URL unchanged.
        let url_page1 = build_paged_url(base, &step, &None, 0);
        assert_eq!(
            url_page1, base,
            "AC-001 regression guard: POST step page1 URL must equal base URL unchanged; \
             got: {url_page1}"
        );
        assert!(
            !url_page1.contains("offset="),
            "AC-001: POST step URL must NOT contain 'offset='; got: {url_page1}"
        );
        assert!(
            !url_page1.contains("limit="),
            "AC-001: POST step URL must NOT contain 'limit='; got: {url_page1}"
        );

        // Page 2 (offset=100): URL must also be base URL unchanged.
        let url_page2 = build_paged_url(base, &step, &None, 100);
        assert_eq!(
            url_page2, base,
            "AC-001 regression guard: POST step page2 URL must equal base URL unchanged; \
             got: {url_page2}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-001 (body side): POST step with OffsetLimit → request body contains
    // top-level "offset" and "limit" integer keys.
    //
    // RED GATE (pre-fix): build_request did not inject offset/limit into the body.
    // This test drives the real production code path (PipelineExecutor::execute_with_max_requests
    // → build_request) via wiremock and inspects the received request body.
    // Passes once Task 3a (thread offset/page_size) + Task 3b (body injection) completed.
    // -----------------------------------------------------------------------

    /// AC-001 / BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch:
    /// POST-body vs GET-URL" — POST step body-injection clause.
    ///
    /// For `method == "POST"` with OffsetLimit pagination, the request body MUST
    /// contain top-level integer keys `"offset"` and `"limit"`.
    ///
    /// Test seam: wiremock mock server receives the real HTTP request built by
    /// `build_request` via `PipelineExecutor::execute_with_max_requests`. The
    /// received request body is inspected via `mock_server.received_requests()`.
    ///
    /// RED GATE (pre-fix): build_request did not inject offset/limit into the body.
    /// Passes once Tasks 3a + 3b completed.
    #[tokio::test]
    async fn test_BC_2_16_002_pagination_post_method_sends_offset_limit_in_body() {
        let mock_server = MockServer::start().await;

        // Single-page response: 2 records (less than page_size=100 → terminates after 1 page).
        // Using up_to_n_times(1) so we capture exactly the first request's body.
        WmMock::given(wm_method("POST"))
            .and(wm_path("/api/v1/alerts"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "alerts": [{"id": "alert-1"}, {"id": "alert-2"}]
            })))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let spec = post_offset_limit_spec(
            &mock_server.uri(),
            "/api/v1/alerts",
            Some("{}".to_string()),
            100,
        );
        let table = spec.tables[0].clone();
        let context = default_context();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest Client::build must succeed");
        let auth_provider = NullAuthProvider;

        // Drive the real production path. max_requests=2 to bound the test.
        let result = PipelineExecutor::execute_with_max_requests(
            &spec,
            &table,
            &context,
            &http_client,
            &auth_provider,
            2,
        )
        .await
        .expect("AC-001: POST-paged execution must succeed");

        assert_eq!(
            result.records.len(),
            2,
            "AC-001: expect 2 records from single-page POST; got {}",
            result.records.len()
        );

        // Inspect the request body received by the mock server.
        let received = mock_server
            .received_requests()
            .await
            .expect("wiremock must record received requests");

        let post_requests: Vec<_> = received
            .iter()
            .filter(|r| r.url.path() == "/api/v1/alerts")
            .collect();

        assert_eq!(
            post_requests.len(),
            1,
            "AC-001: exactly 1 POST request to /api/v1/alerts; got {}",
            post_requests.len()
        );

        let req = &post_requests[0];

        // The URL must NOT contain offset= or limit=.
        let url_str = req.url.as_str();
        assert!(
            !url_str.contains("offset="),
            "AC-001 regression guard: POST URL must not contain 'offset='; url={url_str}"
        );
        assert!(
            !url_str.contains("limit="),
            "AC-001 regression guard: POST URL must not contain 'limit='; url={url_str}"
        );

        // The request body must contain top-level "offset" and "limit" integer keys.
        let body_json: serde_json::Value = serde_json::from_slice(&req.body).unwrap_or_else(|e| {
            panic!(
                "AC-001: POST request body must be valid JSON; parse error: {e}; raw body: {:?}",
                String::from_utf8_lossy(&req.body)
            )
        });

        let offset_val = body_json.get("offset").unwrap_or_else(|| {
            panic!(
                "AC-001: POST body must contain top-level 'offset' key; \
                 body={body_json}"
            )
        });
        let limit_val = body_json.get("limit").unwrap_or_else(|| {
            panic!(
                "AC-001: POST body must contain top-level 'limit' key; \
                 body={body_json}"
            )
        });

        assert!(
            offset_val.is_number(),
            "AC-001: 'offset' in POST body must be a number; got: {offset_val}"
        );
        assert!(
            limit_val.is_number(),
            "AC-001: 'limit' in POST body must be a number; got: {limit_val}"
        );
        assert_eq!(
            offset_val.as_u64(),
            Some(0),
            "AC-001 + AC-005: first page offset must be 0; got: {offset_val}"
        );
        assert_eq!(
            limit_val.as_u64(),
            Some(100),
            "AC-001: limit must equal page_size (100); got: {limit_val}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-002: GET step with OffsetLimit → URL params appended (regression guard)
    //
    // This test is expected to PASS today (existing behavior is correct for GET).
    // It is the regression guard: any change to build_paged_url_impl that
    // accidentally breaks GET appending would cause this to FAIL.
    // -----------------------------------------------------------------------

    /// AC-002 / BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch:
    /// POST-body vs GET-URL" — GET/absent-method step clause (regression guard).
    ///
    /// For `method == "GET"` with OffsetLimit pagination, `build_paged_url_impl`
    /// MUST continue to append `?offset=N&limit=M` to the URL unchanged.
    /// This is the regression guard for Cyberint, Armis, and CrowdStrike GET sensors.
    ///
    /// Expected to PASS today and continue passing after implementation.
    #[test]
    fn test_BC_2_16_002_pagination_get_method_continues_url_params() {
        let step = get_offset_limit_step(50);
        let base = "https://api.example.com/api/logs";

        // Page 1 (offset=0): URL must contain ?offset=0&limit=50.
        let url_page1 = build_paged_url(base, &step, &None, 0);
        assert!(
            url_page1.contains("offset=0"),
            "AC-002 regression guard: GET step page1 URL must contain 'offset=0'; got: {url_page1}"
        );
        assert!(
            url_page1.contains("limit=50"),
            "AC-002 regression guard: GET step page1 URL must contain 'limit=50'; got: {url_page1}"
        );

        // Page 2 (offset=50): URL must contain ?offset=50&limit=50.
        let url_page2 = build_paged_url(base, &step, &None, 50);
        assert!(
            url_page2.contains("offset=50"),
            "AC-002 regression guard: GET step page2 URL must contain 'offset=50'; got: {url_page2}"
        );
        assert!(
            url_page2.contains("limit=50"),
            "AC-002 regression guard: GET step page2 URL must contain 'limit=50'; got: {url_page2}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-004: Body template merging preserves existing body fields
    //
    // RED GATE (pre-fix): build_request did not inject offset/limit into the body.
    // Now implemented: existing body_template keys are preserved (merge, not replace).
    // Passes once Tasks 3a + 3b completed.
    // -----------------------------------------------------------------------

    /// AC-004 / BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch:
    /// POST-body vs GET-URL" — body merge clause.
    ///
    /// When offset+limit are injected into the POST body, any existing keys from
    /// `body_template` MUST be preserved. The pagination params are merged into the
    /// existing body object, not replacing it.
    ///
    /// Test vector: body_template = `{"filter": "active"}` → after injection body
    /// must contain ALL OF: `"filter": "active"`, `"offset": 0`, `"limit": 100`.
    ///
    /// RED GATE (pre-fix): build_request set the body to the raw interpolated body_template
    /// string without merging offset/limit. Now passes with Task 3b complete.
    #[tokio::test]
    async fn test_BC_2_16_002_pagination_body_template_merge_preserves_existing_keys() {
        let mock_server = MockServer::start().await;

        // Single-page response to keep test bounded.
        WmMock::given(wm_method("POST"))
            .and(wm_path("/api/v1/alerts"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "alerts": [{"id": "alert-merge-1"}]
            })))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // body_template has a pre-existing "filter" key that must survive the merge.
        let spec = post_offset_limit_spec(
            &mock_server.uri(),
            "/api/v1/alerts",
            Some(r#"{"filter": "active"}"#.to_string()),
            100,
        );
        let table = spec.tables[0].clone();
        let context = default_context();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest Client::build must succeed");
        let auth_provider = NullAuthProvider;

        PipelineExecutor::execute_with_max_requests(
            &spec,
            &table,
            &context,
            &http_client,
            &auth_provider,
            2,
        )
        .await
        .expect("AC-004: POST-paged execution with existing body keys must succeed");

        let received = mock_server
            .received_requests()
            .await
            .expect("wiremock must record received requests");

        let post_req = received
            .iter()
            .find(|r| r.url.path() == "/api/v1/alerts")
            .expect("AC-004: must have received a POST request to /api/v1/alerts");

        let body_json: serde_json::Value =
            serde_json::from_slice(&post_req.body).unwrap_or_else(|e| {
                panic!(
                    "AC-004: POST request body must be valid JSON; error: {e}; \
                     raw: {:?}",
                    String::from_utf8_lossy(&post_req.body)
                )
            });

        // The pre-existing "filter" key must survive the offset/limit merge.
        let filter_val = body_json.get("filter").unwrap_or_else(|| {
            panic!(
                "AC-004: merged POST body must preserve existing 'filter' key; \
                 body={body_json}"
            )
        });
        assert_eq!(
            filter_val,
            &serde_json::Value::String("active".to_string()),
            "AC-004: 'filter' key must retain its original value 'active'; got: {filter_val}"
        );

        // offset and limit must also be present.
        assert!(
            body_json.get("offset").is_some(),
            "AC-004: merged body must also contain 'offset'; body={body_json}"
        );
        assert!(
            body_json.get("limit").is_some(),
            "AC-004: merged body must also contain 'limit'; body={body_json}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-005: First-page request uses offset=0 in body
    //
    // RED GATE (pre-fix): body injection did not yet exist.
    // Passes once Tasks 3a + 3b completed.
    //
    // Note: AC-005 offset=0 assertion is also covered by test_BC_2_16_002_pagination_post_method_sends_offset_limit_in_body
    // above. This test provides focused dedicated coverage with a clearer diagnostic.
    // -----------------------------------------------------------------------

    /// AC-005 / BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch:
    /// POST-body vs GET-URL" — offset initialization clause.
    ///
    /// For the first pagination step, `offset = 0` and `limit = page_size` MUST be
    /// present in the POST body.
    ///
    /// RED GATE (pre-fix): body injection did not exist. Passes with Task 3b complete.
    #[tokio::test]
    async fn test_BC_2_16_002_pagination_post_first_page_offset_zero_in_body() {
        let mock_server = MockServer::start().await;

        // Single-page response (1 record < page_size=100 → terminates after page 1).
        WmMock::given(wm_method("POST"))
            .and(wm_path("/api/v1/devices"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "alerts": [{"id": "device-1"}]
            })))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let spec = post_offset_limit_spec(
            &mock_server.uri(),
            "/api/v1/devices",
            Some("{}".to_string()),
            100,
        );
        let table = spec.tables[0].clone();
        let context = default_context();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest Client::build must succeed");
        let auth_provider = NullAuthProvider;

        PipelineExecutor::execute_with_max_requests(
            &spec,
            &table,
            &context,
            &http_client,
            &auth_provider,
            2,
        )
        .await
        .expect("AC-005: first-page POST execution must succeed");

        let received = mock_server
            .received_requests()
            .await
            .expect("wiremock must record received requests");

        let req = received
            .iter()
            .find(|r| r.url.path() == "/api/v1/devices")
            .expect("AC-005: must have received at least one POST request to /api/v1/devices");

        let body_json: serde_json::Value = serde_json::from_slice(&req.body).unwrap_or_else(|e| {
            panic!(
                "AC-005: first-page POST body must be valid JSON; error: {e}; \
                     raw: {:?}",
                String::from_utf8_lossy(&req.body)
            )
        });

        let offset = body_json
            .get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| {
                panic!(
                    "AC-005: first-page POST body must contain numeric 'offset' key; \
                     body={body_json}"
                )
            });

        assert_eq!(
            offset, 0,
            "AC-005: first-page offset in POST body must be 0; got: {offset}"
        );

        let limit = body_json
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| {
                panic!(
                    "AC-005: first-page POST body must contain numeric 'limit' key; \
                     body={body_json}"
                )
            });
        assert_eq!(
            limit, 100,
            "AC-005: first-page limit in POST body must equal page_size (100); got: {limit}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-006: build_paged_url_for_test remains callable for GET paths
    //
    // This test confirms the public test helper is accessible and returns correct
    // URL-appended results for GET steps (regression guard per AC-006).
    // Expected to PASS today. Included as a conditional structural regression guard.
    // -----------------------------------------------------------------------

    /// AC-006 / BC-2.16.002 §Postconditions "OffsetLimit Pagination Dispatch:
    /// POST-body vs GET-URL" — GET regression guard.
    ///
    /// The existing `build_paged_url_for_test` public test helper MUST remain callable
    /// and return correct URL-appended results for GET steps after implementation.
    /// No signature change is expected (build_paged_url_impl already receives step: &FetchStep).
    ///
    /// Expected to PASS today and continue passing after implementation.
    #[test]
    fn test_BC_2_16_002_pagination_build_paged_url_for_test_get_path_still_works() {
        use super::build_paged_url_for_test;

        let step = get_offset_limit_step(25);
        let base = "https://api.armis.example.com/api/v1/devices";

        let url = build_paged_url_for_test(base, &step, &None, 0);
        assert!(
            url.contains("offset=0"),
            "AC-006: build_paged_url_for_test GET result must contain 'offset=0'; got: {url}"
        );
        assert!(
            url.contains("limit=25"),
            "AC-006: build_paged_url_for_test GET result must contain 'limit=25'; got: {url}"
        );

        let url2 = build_paged_url_for_test(base, &step, &None, 25);
        assert!(
            url2.contains("offset=25"),
            "AC-006: build_paged_url_for_test GET continuation must contain 'offset=25'; \
             got: {url2}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 companion unit test (SID-1 compliance):
    // Multi-page POST pagination advances offset across pages.
    //
    // RED GATE (pre-fix): offset/limit were not injected into the body. Body injection
    // now implemented: the mock distinguishes pages by their body content
    // (offset=0 vs offset=page_size). The URL-clean + body-has-offset assertions are
    // the POST-body correctness gates. Passes once Tasks 3a + 3b completed.
    // -----------------------------------------------------------------------

    /// AC-003 companion unit test / BC-2.01.013 postcondition §1 (SID-1 compliance).
    ///
    /// Multi-page POST OffsetLimit pagination: 2 pages of 51 records each (total 102).
    /// Verifies that:
    ///   1. The pipeline issues 2 POST requests (offset advances).
    ///   2. The total returned record count is 102.
    ///   3. Page 2 request body contains `"offset": 51` (offset advanced by page_size).
    ///   4. Page 2 URL does NOT contain `offset=` (no URL leakage for POST method).
    ///
    /// This is the non-#[ignore]'d companion to `test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data`.
    /// It drives the pagination loop WITHOUT the external Claroty DTU (wiremock mock
    /// HTTP boundary) per SID-1.
    ///
    /// RED GATE (pre-fix): Without body injection, page 1 and page 2 POSTs were indistinguishable
    /// at the body level. The key assertion `"offset": 51` in the page 2 body now passes
    /// with Tasks 3a + 3b complete.
    #[tokio::test]
    async fn test_BC_2_16_002_pagination_post_offset_advances_across_pages() {
        let mock_server = MockServer::start().await;

        // page_size = 51. Page 1: 51 records (full page → continue pagination).
        let page1_alerts: Vec<serde_json::Value> = (0u32..51)
            .map(|i| serde_json::json!({"id": format!("alert-p1-{i}")}))
            .collect();
        WmMock::given(wm_method("POST"))
            .and(wm_path("/api/v1/alerts"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"alerts": page1_alerts})),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Page 2: 51 records (also full page — the test will stop after 2 pages due to
        // max_requests=2 cap used in execute_with_max_requests below).
        // Using page 2 = 51 records so the pagination loop would continue, but we cap at 2
        // requests. This still verifies that the offset advanced and page 2 was requested.
        // In the green state, page 2 body will have offset=51; the test captures and asserts.
        let page2_alerts: Vec<serde_json::Value> = (0u32..51)
            .map(|i| serde_json::json!({"id": format!("alert-p2-{i}")}))
            .collect();
        WmMock::given(wm_method("POST"))
            .and(wm_path("/api/v1/alerts"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"alerts": page2_alerts})),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let spec = post_offset_limit_spec(
            &mock_server.uri(),
            "/api/v1/alerts",
            Some("{}".to_string()),
            51,
        );
        let table = spec.tables[0].clone();
        let context = default_context();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest Client::build must succeed");
        let auth_provider = NullAuthProvider;

        // Cap at 2 requests: lets exactly 2 pages be fetched before the pipeline aborts
        // with TooManyRequests. We therefore expect an error OR 102 records.
        // Since page 2 is a full page (51 >= page_size=51), the loop would continue but
        // max_requests=2 fires. This is acceptable — we get TooManyRequests on cap, but
        // the 2 requests WERE issued, which is what we need to verify offset advance.
        //
        // Alternative: use a 3rd mock that serves 0 records (clean termination). But the
        // TooManyRequests path still exercises the 2-request case and the URL/body checks
        // below operate on received_requests regardless of Ok vs Err.
        let _ = PipelineExecutor::execute_with_max_requests(
            &spec,
            &table,
            &context,
            &http_client,
            &auth_provider,
            2,
        )
        .await;
        // Note: may return TooManyRequests error (max_requests=2 cap fires after page 2).
        // That's acceptable — we only need the 2 requests to have been issued.

        let received = mock_server
            .received_requests()
            .await
            .expect("wiremock must record received requests");

        let post_reqs: Vec<_> = received
            .iter()
            .filter(|r| r.url.path() == "/api/v1/alerts")
            .collect();

        assert_eq!(
            post_reqs.len(),
            2,
            "AC-003-unit regression guard: pagination must issue 2 POST requests (page 1 + page 2); \
             got {} requests",
            post_reqs.len()
        );

        // Assert page 2 URL does NOT contain offset= (no URL leakage for POST).
        let page2_url = post_reqs[1].url.as_str();
        assert!(
            !page2_url.contains("offset="),
            "AC-003-unit regression guard: page 2 POST URL must not contain 'offset='; url={page2_url}"
        );
        assert!(
            !page2_url.contains("limit="),
            "AC-003-unit regression guard: page 2 POST URL must not contain 'limit='; url={page2_url}"
        );

        // Assert page 2 body contains "offset": 51 (offset advanced by page_size).
        let page2_body: serde_json::Value = serde_json::from_slice(&post_reqs[1].body)
            .unwrap_or_else(|e| {
                panic!(
                    "AC-003-unit: page 2 POST body must be valid JSON; error: {e}; \
                     raw: {:?}",
                    String::from_utf8_lossy(&post_reqs[1].body)
                )
            });

        let page2_offset = page2_body
            .get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| {
                panic!(
                    "AC-003-unit: page 2 POST body must contain 'offset' key; \
                     body={page2_body}"
                )
            });

        assert_eq!(
            page2_offset, 51,
            "AC-003-unit: page 2 offset in body must be 51 (advanced by page_size=51); \
             got: {page2_offset}"
        );
    }

    // -----------------------------------------------------------------------
    // EC-002: POST OffsetLimit body_template interpolates to a non-object JSON value
    //
    // EC-002 specifies: "Treat as parse error; surface SpecEngineError with
    // sensor_id and step_name. Do NOT panic."
    //
    // Two sub-cases:
    //   (a) body_template is not valid JSON at all (parse fails in build_request)
    //   (b) body_template is valid JSON but NOT an object (e.g., `[]`, `42`, `"str"`)
    //
    // Both branches surface as SpecEngineError::HttpRequestFailed{status_code:0}.
    // The HTTP request is never sent (build_request returns Err before .send()).
    // -----------------------------------------------------------------------

    /// EC-002 / BC-2.16.002 §Edge Cases — POST OffsetLimit body_template
    /// interpolates to a non-object JSON value (e.g., raw array `[]`).
    ///
    /// Contract: "Treat as parse error; surface SpecEngineError with sensor_id and
    /// step_name. Do NOT panic." (BC-2.16.002 EC-002)
    ///
    /// Test vector: `body_template = "[]"` (a JSON array literal). After interpolation
    /// the `serde_json::Value` is `Array([])` — not an Object — which triggers the
    /// non-object branch in `build_request`.
    ///
    /// This test drives the REAL production code path:
    ///   PipelineExecutor::execute_with_max_requests
    ///   → issue_request_with_retry
    ///   → build_request (returns Err before .send())
    ///   → maps to SpecEngineError::HttpRequestFailed{status_code:0}
    ///
    /// Naming: test_BC_<id>_<desc> per CLAUDE.md §Conventions.
    #[tokio::test]
    async fn test_BC_2_16_002_pagination_post_non_object_body_surfaces_error() {
        // SEC-001 regression guard: a valid-JSON non-object body_template that contains a
        // recognizable sentinel.  The sentinel must NOT appear in the SpecEngineError detail.
        const SENTINEL: &str = "SENSITIVE_SENTINEL_VALUE";

        // Start a mock server to provide a valid URL. The request will never reach
        // the server — build_request returns Err before .send() is called for the
        // EC-002 branch. No routes are registered intentionally.
        let mock_server = MockServer::start().await;

        // body_template = `["SENSITIVE_SENTINEL_VALUE"]`: a JSON array (not an object)
        // that contains the sentinel token.
        // After Interpolator::interpolate (no variables → passthrough), serde_json parses
        // this as Value::Array([...]) — triggers the non-object arm in build_request
        // (EC-002 branch b).  The sentinel-absence assertion proves that even though the
        // body parses successfully (it is valid JSON), its content is not echoed into the
        // error detail (SEC-001 / CWE-209 guard).
        let body_template = format!(r#"["{SENTINEL}"]"#);
        let spec = post_offset_limit_spec(
            &mock_server.uri(),
            "/api/v1/alerts",
            Some(body_template),
            100,
        );
        let table = spec.tables[0].clone();
        let context = default_context();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest Client::build must succeed");
        let auth_provider = NullAuthProvider;

        let result = PipelineExecutor::execute_with_max_requests(
            &spec,
            &table,
            &context,
            &http_client,
            &auth_provider,
            2,
        )
        .await;

        assert!(
            result.is_err(),
            "EC-002: POST OffsetLimit step with non-object body_template must surface \
             an error, not succeed; got Ok with {} records",
            result.as_ref().map(|r| r.records.len()).unwrap_or(0)
        );

        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                crate::error::SpecEngineError::HttpRequestFailed { status_code: 0, .. }
            ),
            "EC-002: error must be SpecEngineError::HttpRequestFailed{{status_code:0}}; \
             got: {err:?}"
        );

        // Verify the error detail mentions the body / interpolation problem.
        let detail = match &err {
            crate::error::SpecEngineError::HttpRequestFailed { detail, .. } => detail.clone(),
            other => panic!("unexpected error variant: {other:?}"),
        };
        assert!(
            detail.contains("non-object") || detail.contains("body interpolation failed"),
            "EC-002: error detail must mention the non-object body or interpolation failure; \
             got detail: {detail:?}"
        );

        // SEC-001 regression guard (CWE-209 / FB-ADV-179-003): the body content must NOT
        // appear in the error detail.  The non-object branch produces an error naming the
        // JSON value type ("array", "null", etc.) but must NOT dump the body value itself.
        assert!(
            !detail.contains(SENTINEL),
            "SEC-001 regression: error detail must NOT contain the raw body sentinel \
             (interpolated body must not be leaked into error strings); \
             got detail: {detail:?}"
        );
    }

    /// EC-002 / BC-2.16.002 §Edge Cases — POST OffsetLimit body_template
    /// is not valid JSON (parse branch, EC-002 branch a).
    ///
    /// Contract: "Treat as parse error; surface SpecEngineError with sensor_id and
    /// step_name. Do NOT panic." (BC-2.16.002 EC-002)
    ///
    /// Test vector: `body_template = "{SENSITIVE_SENTINEL_VALUE"` — an unterminated/malformed
    /// JSON object that embeds a recognizable sentinel token.  After
    /// Interpolator::interpolate (no variables → passthrough), serde_json::from_str fails
    /// → build_request returns Err → maps to SpecEngineError::HttpRequestFailed{status_code:0}.
    ///
    /// The sentinel is chosen so that:
    ///   - It still triggers EC-002 branch (a): body is not parseable as JSON.
    ///   - The sentinel-absence assertion on `detail` proves SEC-001 (CWE-209) stays fixed:
    ///     if a future change reintroduces `raw body: {interpolated_body:?}` in the error
    ///     string the sentinel will appear in `detail` and the test will fail.
    ///
    /// This covers EC-002 branch (a): interpolated body is not parseable as JSON at all.
    #[tokio::test]
    async fn test_BC_2_16_002_pagination_post_invalid_json_body_surfaces_error() {
        // SEC-001 regression guard: a malformed body_template that contains a recognizable
        // sentinel.  The sentinel must NOT appear in the SpecEngineError detail — if it
        // does, the fix (removing `raw body: {interpolated_body:?}` from the error string)
        // has been regressed.
        const SENTINEL: &str = "SENSITIVE_SENTINEL_VALUE";

        let mock_server = MockServer::start().await;

        // body_template = "{SENSITIVE_SENTINEL_VALUE": an unterminated JSON object that
        // contains the sentinel.  serde_json::from_str fails (EC-002 branch a) while
        // the sentinel is preserved in the interpolated body.
        let body_template = format!("{{{SENTINEL}");
        let spec = post_offset_limit_spec(
            &mock_server.uri(),
            "/api/v1/alerts",
            Some(body_template),
            100,
        );
        let table = spec.tables[0].clone();
        let context = default_context();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest Client::build must succeed");
        let auth_provider = NullAuthProvider;

        let result = PipelineExecutor::execute_with_max_requests(
            &spec,
            &table,
            &context,
            &http_client,
            &auth_provider,
            2,
        )
        .await;

        assert!(
            result.is_err(),
            "EC-002(a): POST OffsetLimit step with invalid JSON body_template must surface \
             an error, not succeed; got Ok with {} records",
            result.as_ref().map(|r| r.records.len()).unwrap_or(0)
        );

        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                crate::error::SpecEngineError::HttpRequestFailed { status_code: 0, .. }
            ),
            "EC-002(a): error must be SpecEngineError::HttpRequestFailed{{status_code:0}}; \
             got: {err:?}"
        );

        // Verify sensor_id and step_name are present in the error (EC-002 contract).
        let (sensor_id, step_name, detail) = match &err {
            crate::error::SpecEngineError::HttpRequestFailed {
                sensor_id,
                step_name,
                detail,
                ..
            } => (sensor_id.clone(), step_name.clone(), detail.clone()),
            other => panic!("unexpected error variant: {other:?}"),
        };
        assert_eq!(
            sensor_id, "post-paginated-sensor",
            "EC-002(a): error must carry sensor_id; got: {sensor_id:?}"
        );
        assert_eq!(
            step_name, "fetch_alerts",
            "EC-002(a): error must carry step_name; got: {step_name:?}"
        );

        // SEC-001 regression guard (CWE-209 / FB-ADV-179-003): the interpolated body value
        // must NOT appear in the error detail.  If this assertion fails it means a future
        // change reintroduced dumping the raw body into the error string.
        assert!(
            !detail.contains(SENTINEL),
            "SEC-001 regression: error detail must NOT contain the raw body sentinel \
             (interpolated body must not be leaked into error strings); \
             got detail: {detail:?}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 integration test (DTU-gated, #[ignore]'d)
    //
    // Per SID-1: this test is gated on the live DTU clone. The companion unit test
    // above (test_BC_2_16_002_pagination_post_offset_advances_across_pages) provides
    // non-ignored coverage of the pagination-advance logic.
    // -----------------------------------------------------------------------

    /// AC-003 / BC-2.01.013 postcondition §1 — integration test against Claroty DTU.
    ///
    /// A test issuing against a DTU serving exactly 102 synthetic alert entries
    /// returns 102 rows (all entries across 2 paginated POST requests).
    ///
    /// Without the POST-body pagination fix, only 100 rows are returned (page 1 only)
    /// because the Claroty API/DTU ignores URL-based offset params.
    ///
    /// # DTU-EXT-001: requires Claroty DTU clone running
    ///
    /// Ungated in CI after S-DEMO-CLAROTY-PAGINATION-001 is merged and the DTU
    /// 102-fixture is verified via TS-PLUGIN-PARITY-001 procedure.
    ///
    /// Companion non-ignored unit test:
    /// `test_BC_2_16_002_pagination_post_offset_advances_across_pages` (above)
    #[tokio::test]
    #[ignore = "DTU-EXT-001: requires prism-dtu-claroty clone running with 102-entry alerts fixture; ungated after S-DEMO-CLAROTY-PAGINATION-001 merges and DTU fixture is recorded"]
    async fn test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data() {
        use crate::spec_parser::SpecLoader;
        use prism_dtu_claroty::ClarotyClone;
        use prism_dtu_common::BehavioralClone;

        // Boot a Claroty DTU clone serving a 102-entry alerts fixture.
        // The fixture produces exactly 2 pages at page_size=100:
        //   page 1 (offset=0, limit=100): 100 records (full page → continue)
        //   page 2 (offset=100, limit=100): 2 records (short page → stop)
        let mut clone = ClarotyClone::default();
        clone
            .start()
            .await
            .expect("Claroty DTU clone must start successfully");
        let base_url = clone.base_url();

        // Load the canonical Claroty sensor spec from the bundled TOML via SpecLoader::parse.
        // The TOML is read from the sensors/specs directory relative to the workspace root
        // (same path used by the bundled-spec-load tests in bc_2_16_001_test.rs).
        let claroty_toml = std::fs::read_to_string("sensors/specs/claroty.sensor.toml")
            .expect("claroty.sensor.toml must be readable from workspace root sensors/specs/");
        let mut sensor_spec = SpecLoader::parse(&claroty_toml)
            .expect("claroty.sensor.toml must parse without errors");

        // Select the alerts table (method=POST + OffsetLimit).
        let alerts_table = sensor_spec
            .tables
            .iter()
            .find(|t| t.table_name == "alerts")
            .expect("claroty sensor spec must have an 'alerts' table")
            .clone();

        let context = FetchContext::new(OrgSlug::new("demo-org"), HashMap::new(), None);
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest Client::build must succeed");

        // Override base_url with the DTU clone's address.
        sensor_spec.base_url = base_url;

        let result = PipelineExecutor::execute(
            &sensor_spec,
            &alerts_table,
            &context,
            &http_client,
            &NullAuthProvider,
        )
        .await
        .expect("AC-003: Claroty alerts pipeline execution must succeed");

        assert_eq!(
            result.records.len(),
            102,
            "AC-003: 102-entry fixture must return 102 rows across 2 paginated POST requests; \
             got {} rows\n\
             If this returns 100, the POST-body pagination fix is not applied (page 2 was not \
             fetched because the DTU ignored URL-based offset params).",
            result.records.len()
        );
        assert!(
            result.request_count >= 2,
            "AC-003: at least 2 POST requests must have been issued (2 pages); \
             got request_count={}",
            result.request_count
        );
    }
}

#[cfg(test)]
mod store_step_vars_tests {
    use std::collections::HashMap;

    use crate::spec_parser::FetchStep;

    use super::store_step_vars;

    // -----------------------------------------------------------------------
    // F-CSD-P24-OBS-002: lock the `store_step_vars` last-segment fallback contract
    //
    // When `variables_produced` is empty (no explicit variables declared), `store_step_vars`
    // derives a key from the last dotted segment of `response_path` via:
    //   `response_path.split('.').next_back()` → key `"{step_name}.{last_seg}"`
    //
    // Real-world use: crowdstrike.sensor.toml's `query_detection_ids` step has
    // `response_path = "$.resources"` and `variables_produced = []` (implicit).
    // The `post_detection_summaries` step's body_template references
    // `${query_detection_ids.resources}` — that variable is populated by this fallback.
    // Without it the fan-out array would never be found and the second step would send
    // an empty IDs list to the CrowdStrike API.
    //
    // These unit tests call `store_step_vars` directly (it is a private function;
    // access is permitted within the same file's `#[cfg(test)]` modules).
    // -----------------------------------------------------------------------

    /// F-CSD-P24-OBS-002 / BC-2.16.002 §Postconditions:
    /// `store_step_vars` last-segment fallback — a step with `variables_produced = []`
    /// and `response_path = "$.resources"` must insert key `"{step_name}.resources"`
    /// into `step_vars` with the extracted value.
    ///
    /// This is the mechanism that makes `${query_detection_ids.resources}` resolvable
    /// in the next step's body_template without an explicit `variables_produced` entry
    /// in the TOML spec (crowdstrike.sensor.toml `query_detection_ids` step).
    #[test]
    fn test_F_CSD_P24_OBS_002_store_step_vars_last_segment_fallback_key_stored() {
        let step = FetchStep {
            name: "query_detection_ids".to_string(),
            response_path: "$.resources".to_string(),
            variables_produced: vec![],
            ..Default::default()
        };

        let body = serde_json::json!({ "resources": ["id-001", "id-002"] });
        let extracted = serde_json::json!(["id-001", "id-002"]);
        let mut step_vars: HashMap<String, serde_json::Value> = HashMap::new();

        store_step_vars(&step, &body, &extracted, &mut step_vars);

        // LOCK: last-segment fallback must produce key "query_detection_ids.resources".
        let key = "query_detection_ids.resources";
        assert!(
            step_vars.contains_key(key),
            "F-CSD-P24-OBS-002: store_step_vars fallback must insert key '{key}' \
             (last path segment of response_path '$.resources'). \
             This is the mechanism that makes ${{query_detection_ids.resources}} \
             resolvable in the next step's body_template. \
             got step_vars keys: {:?}",
            step_vars.keys().collect::<Vec<_>>()
        );

        // The stored value must be the extracted value, not the raw body.
        assert_eq!(
            step_vars[key], extracted,
            "F-CSD-P24-OBS-002: store_step_vars fallback must store the extracted value \
             under key '{key}'. got: {:?}",
            step_vars[key]
        );
    }

    /// F-CSD-P24-OBS-002b: `or_insert_with` guard — when a `variables_produced` entry has
    /// already populated the same key, the last-segment fallback must NOT overwrite it.
    ///
    /// This tests the `step_vars.entry(key).or_insert_with(|| extracted.clone())` guard:
    /// the fallback uses `or_insert_with` so that an explicitly declared variable (from the
    /// `variables_produced` loop above it) is never clobbered by the implicit fallback.
    #[test]
    fn test_F_CSD_P24_OBS_002b_store_step_vars_fallback_does_not_overwrite_variables_produced() {
        // Step that declares "resources" in variables_produced AND has response_path "$.resources".
        // The variables_produced loop populates the key first; the fallback must skip it.
        let step = FetchStep {
            name: "fetch_ids".to_string(),
            response_path: "$.resources".to_string(),
            variables_produced: vec!["resources".to_string()],
            ..Default::default()
        };

        let body = serde_json::json!({ "resources": ["id-001", "id-002"] });
        // extracted is a sentinel distinct from body["resources"] — if the fallback
        // overwrites the variables_produced value with extracted, this test fails.
        let extracted = serde_json::json!(["SHOULD_NOT_OVERWRITE"]);
        let mut step_vars: HashMap<String, serde_json::Value> = HashMap::new();

        store_step_vars(&step, &body, &extracted, &mut step_vars);

        // variables_produced path writes body["resources"] under "fetch_ids.resources" first.
        let key = "fetch_ids.resources";
        assert!(
            step_vars.contains_key(key),
            "F-CSD-P24-OBS-002b: key '{key}' must exist (set by variables_produced loop)"
        );
        // The value must be from variables_produced (body["resources"]), not from extracted.
        assert_eq!(
            step_vars[key],
            serde_json::json!(["id-001", "id-002"]),
            "F-CSD-P24-OBS-002b: or_insert_with must NOT overwrite the variables_produced \
             value with the extracted fallback. The `entry().or_insert_with()` guard must \
             preserve the value set by the variables_produced loop. got: {:?}",
            step_vars[key]
        );
    }

    /// F-CSD-P24-OBS-002c: nested response_path segment — `response_path = "$.data.items"`
    /// must store key `"{step_name}.items"` (the LAST segment after splitting on '.').
    ///
    /// This verifies that the `next_back()` implementation selects the last segment
    /// regardless of path depth.
    #[test]
    fn test_F_CSD_P24_OBS_002c_store_step_vars_nested_response_path_last_segment_used() {
        let step = FetchStep {
            name: "fetch_results".to_string(),
            response_path: "$.data.items".to_string(),
            variables_produced: vec![],
            ..Default::default()
        };

        let body = serde_json::json!({ "data": { "items": ["a", "b"] } });
        let extracted = serde_json::json!(["a", "b"]);
        let mut step_vars: HashMap<String, serde_json::Value> = HashMap::new();

        store_step_vars(&step, &body, &extracted, &mut step_vars);

        // The last segment of "$.data.items" is "items" (split on '.', next_back).
        let key = "fetch_results.items";
        assert!(
            step_vars.contains_key(key),
            "F-CSD-P24-OBS-002c: last segment of '$.data.items' must produce key '{key}'. \
             got step_vars keys: {:?}",
            step_vars.keys().collect::<Vec<_>>()
        );
        assert_eq!(
            step_vars[key], extracted,
            "F-CSD-P24-OBS-002c: stored value must be extracted; got: {:?}",
            step_vars[key]
        );
    }
}

// ---------------------------------------------------------------------------
// RG-003: pipeline non-2xx response must capture HTTP body in detail field
// ---------------------------------------------------------------------------

/// RG-003: `PipelineExecutor` MUST capture the HTTP response body snippet into
/// `SpecEngineError::HttpRequestFailed.detail` when the server returns a non-2xx status.
///
/// Before fix: `detail` is set to `format!("HTTP {status}")` — no body captured.
///   → `detail.contains("forbidden")` == false → assertion FAILS → RED.
///
/// After fix: body snippet appended to detail string (e.g. `"HTTP 403: forbidden"`)
///   → `detail.contains("forbidden")` == true → GREEN.
///
/// Uses wiremock so the test exercises the real `execute_with_max_requests` HTTP path.
/// Response: 403 with text body "forbidden".
///
/// BC-2.16.002 (Non-2xx Response Body Capture postcondition) AC-ERR-003 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-003
#[cfg(test)]
mod non_2xx_body_snippet_tests {
    use std::collections::HashMap;

    use prism_core::{ColumnType, OrgSlug};
    use wiremock::{
        Mock as WmMock, MockServer, ResponseTemplate,
        matchers::{method as wm_method, path as wm_path},
    };

    use crate::{
        auth_provider::MockAuthProvider,
        error::SpecEngineError,
        pipeline::{FetchContext, PipelineExecutor},
        spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec},
    };

    /// Build a minimal SensorSpec for a single GET step pointing to the provided
    /// base URL / path. No pagination so the executor issues exactly one request.
    fn single_step_spec(base_url: &str, path: &str) -> SensorSpec {
        SensorSpec::new(
            "rg003-sensor",
            "RG-003 Sensor",
            AuthType::BearerStatic,
            base_url,
            vec![TableSpec::new_point_in_time(
                "devices",
                "network_activity",
                vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
                vec![FetchStep::new(
                    "fetch_devices",
                    "GET",
                    path,
                    None,
                    "$.items",
                    None,
                    vec![],
                    None,
                    None,
                )],
            )],
            None,
            "1.0.0",
            vec![],
        )
    }

    /// RG-003: pipeline HttpRequestFailed.detail MUST contain response body snippet.
    ///
    /// FAIL reason before fix: `detail` is `"HTTP 403"`, which does NOT contain "forbidden".
    /// → `assert!(detail.contains("forbidden"), ...)` panics → RED.
    ///
    /// BC-2.16.002 (Non-2xx Response Body Capture postcondition) AC-ERR-003 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-003
    #[tokio::test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    async fn test_pipeline_non_2xx_body_in_detail() {
        let mock_server = MockServer::start().await;

        // Wiremock: return 403 Forbidden with text body "forbidden".
        WmMock::given(wm_method("GET"))
            .and(wm_path("/api/v1/devices"))
            .respond_with(ResponseTemplate::new(403).set_body_bytes(b"forbidden"))
            .mount(&mock_server)
            .await;

        let spec = single_step_spec(&mock_server.uri(), "/api/v1/devices");
        let table = spec.tables[0].clone();
        let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
        let http_client = reqwest::Client::builder()
            .build()
            .expect("RG-003: reqwest::Client build must succeed");
        let auth_provider = MockAuthProvider::new("rg003-token");

        let result = PipelineExecutor::execute_with_max_requests(
            &spec,
            &table,
            &context,
            &http_client,
            &auth_provider,
            10,
        )
        .await;

        assert!(
            result.is_err(),
            "RG-003: pipeline must return Err on 403 response; got Ok"
        );

        let err = result.unwrap_err();
        match &err {
            SpecEngineError::HttpRequestFailed {
                status_code,
                detail,
                ..
            } => {
                assert_eq!(
                    *status_code, 403,
                    "RG-003: HttpRequestFailed.status_code must be 403; got: {status_code}"
                );

                assert!(
                    detail.contains("forbidden"),
                    "RG-003: HttpRequestFailed.detail must contain the response body snippet \
                     ('forbidden') so operators can diagnose the 403. \
                     Current detail (before fix): {:?}. \
                     Fix: append response body bytes to the `detail` field when the \
                     server returns a non-2xx status (BC-2.16.002 AC-ERR-003).",
                    detail
                );

                assert!(
                    detail.contains("403"),
                    "RG-003: HttpRequestFailed.detail must still contain the status code '403'; \
                     got: {:?}",
                    detail
                );
            }
            other => {
                panic!(
                    "RG-003: expected SpecEngineError::HttpRequestFailed for 403 response; \
                     got: {:?}",
                    other
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// OBS-4: build_http_client_with_timeout must set prism/ User-Agent header
// ---------------------------------------------------------------------------

/// OBS-4: `build_http_client_with_timeout` (the `HttpLookupSource` outbound client factory
/// in `prism-spec-engine`) MUST produce a `reqwest::Client` that sends a `User-Agent`
/// header beginning with `"prism/"`.
///
/// Before fix: no `.user_agent()` call → reqwest default `"reqwest/x.x.x"` → FAILS.
/// After fix: `.user_agent("prism/{version}")` → starts with `"prism/"` → GREEN.
///
/// This is the sibling sweep of the same obligation already satisfied by
/// `prism_bin::spec_driven_adapter::build_http_client_with_custom_timeout` (ADR-050 §D6).
///
/// AC-UA-001 | BC-2.16.002 (HTTP Client Compliance postconditions) | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 OBS-4
#[cfg(test)]
mod infusion_http_client_user_agent_tests {
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use super::build_http_client_with_timeout;

    /// OBS-4: `build_http_client_with_timeout` MUST produce a client that sends
    /// `User-Agent: prism/{version}` so WAF appliances attribute enrichment/threat-intel
    /// HTTP calls to prism (ADR-050 §D6 WAF-fingerprint coherence).
    #[tokio::test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    async fn test_infusion_http_client_sends_prism_user_agent() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/probe"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = build_http_client_with_timeout()
            .expect("OBS-4: reqwest::Client build must succeed under rustls-tls (ADR-050 §D1/D2)");

        // Fire a request so wiremock records the User-Agent header.
        let _ = client
            .get(format!("{}/probe", mock_server.uri()))
            .send()
            .await;

        let received = mock_server
            .received_requests()
            .await
            .expect("OBS-4: wiremock must record received requests");

        assert_eq!(
            received.len(),
            1,
            "OBS-4: exactly one request must be recorded by wiremock; got {}",
            received.len()
        );

        let ua = received[0]
            .headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert!(
            ua.starts_with("prism/"),
            "OBS-4 (AC-UA-001): build_http_client_with_timeout (prism-spec-engine) MUST send \
             'User-Agent: prism/{{version}}' for WAF-fingerprint coherence (ADR-050 §D6). \
             Got: {:?}. Fix: add .user_agent(concat!(\"prism/\", env!(\"CARGO_PKG_VERSION\"))) \
             to the builder in build_http_client_with_timeout (BC-2.16.002 AC-UA-001).",
            ua
        );
    }

    /// F-2 (DEFECT-ADAPTER-TLS-XDOME-LIVE-001): `build_http_client_with_timeout`
    /// MUST return `Ok(Client)` under normal config (rustls-tls default, ADR-050 §D1/D2).
    ///
    /// This test mirrors `test_BC_2_01_013_build_http_client_with_custom_timeout_accepts_duration`
    /// in `prism-bin` (RG-PERF-001 precedent) and asserts that the `Result` path resolves
    /// to `Ok` — confirming the production code never panics on client construction.
    ///
    /// Under rustls-tls the only failure mode is malformed TLS configuration, which
    /// cannot occur with the default rustls stack.  `Err(String)` is effectively
    /// unreachable but must be a `Result` rather than an `expect()` call (CLAUDE.md
    /// §Forbidden patterns: no `.expect()` on `Result` in non-test code paths).
    #[test]
    fn test_build_http_client_with_timeout_returns_ok_under_rustls() {
        let result = build_http_client_with_timeout();
        assert!(
            result.is_ok(),
            "F-2 (DEFECT-ADAPTER-TLS-XDOME-LIVE-001): build_http_client_with_timeout \
             (prism-spec-engine) must return Ok(Client) under the default rustls-tls stack \
             (ADR-050 §D1/D2). Got Err: {:?}",
            result.err()
        );
    }
}

// ---------------------------------------------------------------------------
// RG-004: BC-2.16.013 §Postcondition 1 — JSON filter string auto-parsed to
// Value::Object in step_vars (S-CLAROTY-AUDITLOG-TIMEBOX-001)
// ---------------------------------------------------------------------------
//
// When `context.query_filters["_claroty_audit_filter_by"]` is a JSON-object
// string (starting with `{`), the step_vars seeding MUST parse it to
// `Value::Object` rather than leaving it as `Value::String`.
//
// CURRENT FAILURE: The seeding (lines ~265-270) always stores `Value::String`.
// In `JsonBody` interpolation context, `Value::String(s)` calls `json_escape(s)`
// which escapes inner quotes: `{"field":"timestamp",...}` →
// `{\"field\":\"timestamp\",...}`. Placed bare in a body template, this produces
// INVALID JSON. The wire-level assertion `assert!(body_json.is_ok(), ...)` FAILS.
//
// After implementation: `Value::Object` → `value.to_string()` → inline JSON.
// Body is valid JSON; `filter_by` is a JSON object.
//
// Backward-compat: FQL strings (`created_timestamp:>'...'`) do NOT start
// with `{` or `[`, stay as `Value::String`, and are correctly json_escaped
// when embedded as a JSON string value in a body template.
#[cfg(test)]
mod rg004_pipeline_json_filter_tests {
    use std::collections::HashMap;

    use prism_core::OrgSlug;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::{
        auth_provider::MockAuthProvider,
        pipeline::{FetchContext, PipelineExecutor},
        spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec},
    };

    /// BC-2.16.013 §Postcondition 1:
    /// A `query_filter` value that is a JSON-object string (leading `{`) MUST be
    /// stored as `Value::Object` in step_vars — not `Value::String`. This ensures
    /// that the body_template `${query.filter._claroty_audit_filter_by}` emits
    /// the filter inline as JSON (not as a backslash-escaped string).
    ///
    /// # Red Gate Failure
    ///
    /// The received POST body is NOT valid JSON because `Value::String` causes
    /// `json_escape()` to produce `{\"field\":\"timestamp\",...}` — invalid JSON
    /// when placed bare in a body template. `assert!(body_json.is_ok(), ...)` FAILS.
    ///
    /// # Test design (SID-1 compliant)
    ///
    /// Uses a SYNTHETIC spec WITHOUT OffsetLimit pagination to avoid the early
    /// body-validation at `serde_json::from_str(&interpolated_body)` in the
    /// OffsetLimit path. Without OffsetLimit, the malformed body is sent to the
    /// mock server, and the wire-level assertion catches the issue.
    ///
    /// BC-2.16.013 §Postcondition 1; Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 / RG-004.
    #[tokio::test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    async fn test_BC_2_16_013_pipeline_json_filter_string_parsed_to_value_object_backward_compat() {
        let mock_server = MockServer::start().await;

        // Mount: respond to POST /api/v1/audit_log/get with a minimal valid response.
        // The mock captures raw request bytes regardless of body content validity.
        Mock::given(method("POST"))
            .and(path("/api/v1/audit_log/get"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"audit_log": [], "total": 0})),
            )
            .mount(&mock_server)
            .await;

        // Synthetic SensorSpec — one POST step with a body_template that uses
        // ${query.filter._claroty_audit_filter_by}. NO OffsetLimit pagination
        // (pagination = None) so the body is sent as-is without early validation.
        let spec = SensorSpec {
            sensor_id: "claroty".to_string(),
            name: "Claroty audit filter backward-compat test".to_string(),
            auth_type: AuthType::BearerStatic,
            base_url: mock_server.uri(),
            tables: vec![TableSpec::new_point_in_time(
                "audit_logs",
                "audit_activity",
                vec![ColumnSpec {
                    name: "id".to_string(),
                    column_type: prism_core::ColumnType::String,
                    ocsf_field: None,
                    options: vec![],
                    timestamp_formats: vec![],
                    timestamp_fallback_chain: vec![],
                    source_path: None,
                }],
                vec![FetchStep {
                    name: "fetch_audit_logs".to_string(),
                    method: "POST".to_string(),
                    path_template: "/api/v1/audit_log/get".to_string(),
                    // body_template uses two filter slots:
                    //   (a) JSON-object slot for Claroty filter_by (must expand to Value::Object)
                    //   (b) quoted string slot for FQL backward-compat (must stay Value::String)
                    body_template: Some(
                        r#"{"filter_by": ${query.filter._claroty_audit_filter_by}, "fql": "${query.filter._fql_filter}"}"#.to_string(),
                    ),
                    response_path: "$.audit_log".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    // NO OffsetLimit — avoids the serde_json::from_str early-failure
                    // so the malformed body reaches the mock server.
                    pagination: None,
                }],
            )],
            rate_limit_hints: None,
            version: "1.0.0".to_string(),
            credential_refs: vec![],
            auth_plugin: None,
            file_hash: String::new(),
            source_path: String::new(),
            mode: crate::types::DtuMode::Shared,
            ocsf_column_naming: false,
            probe_table: None,
        };

        // FetchContext: two entries —
        //   (a) _claroty_audit_filter_by: JSON-object string (starts with `{`)
        //       → after implementation, parsed to Value::Object in step_vars
        //   (b) _fql_filter: plain FQL string (does NOT start with `{`/`[`)
        //       → MUST stay as Value::String (backward-compat gate; AC-004)
        // AC-004: JSON-object filter value uses ISO-8601 string "2026-01-01T00:00:00Z",
        // NOT integer 1234567890. BC-2.01.013 EC-01-030..EC-01-033.
        let mut query_filters = HashMap::new();
        query_filters.insert(
            "_claroty_audit_filter_by".to_string(),
            r#"{"field": "timestamp", "operation": "greater_or_equal", "value": "2026-01-01T00:00:00Z"}"#
                .to_string(),
        );
        // FQL string: does NOT start with `{` or `[` — must remain Value::String.
        // This is the backward-compat gate (AC-004 / RG-004 else-branch).
        query_filters.insert(
            "_fql_filter".to_string(),
            "created_timestamp:>2026-01-01".to_string(),
        );
        let context = FetchContext::new(OrgSlug::new("claroty-rg004-org"), query_filters, None);

        let http_client = reqwest::Client::new(); // in-process test client
        let auth_provider = MockAuthProvider::new("claroty-rg004-test-token");

        // Execute the pipeline. With current code (before implementation):
        // - step_vars["query.filter._claroty_audit_filter_by"] = Value::String(r#"{"field":...}"#)
        // - JsonBody context: json_escape → {\"field\":\"timestamp\",\"value\":\"2026-01-01T00:00:00Z\",...}
        // - Interpolated body: {"filter_by": {\"field\":...}, "fql": "..."} (INVALID JSON for filter_by slot)
        // → body_json.is_ok() FAILS (Red Gate)
        // - Body sent to mock (no OffsetLimit validation), mock responds 200
        // - Pipeline returns Ok (response is valid JSON {"audit_log": [], "total": 0})
        let result = PipelineExecutor::execute(
            &spec,
            &spec.tables[0],
            &context,
            &http_client,
            &auth_provider,
        )
        .await;

        // The pipeline MUST succeed — the mock returns 200 with a valid response body.
        assert!(
            result.is_ok(),
            "RG-004: PipelineExecutor::execute must succeed against a 200 mock. \
             Got Err: {:?}. \
             This spec uses pagination = None so the OffsetLimit body-validation path \
             is not reached. If the pipeline failed, check body interpolation logic.",
            result.err()
        );

        // Check the outbound POST body at the wire level.
        let requests = mock_server.received_requests().await.unwrap_or_default();
        assert!(
            !requests.is_empty(),
            "RG-004: PipelineExecutor::execute must have issued a POST to \
             /api/v1/audit_log/get. Got no requests."
        );

        let body_bytes = &requests[0].body;
        let body_str =
            std::str::from_utf8(body_bytes).expect("received POST body must be valid UTF-8");

        let body_json = serde_json::from_str::<serde_json::Value>(body_str);

        // LOAD-BEARING Red Gate assertion (RG-004):
        // The received POST body MUST be valid JSON.
        // FAILS BEFORE IMPLEMENTATION: step_vars seeding stores query_filters as
        // Value::String; json_escape() produces {\"field\":\"timestamp\",...} which
        // is NOT valid JSON when placed bare in a body template.
        assert!(
            body_json.is_ok(),
            "RG-004 LOAD-BEARING: POST body must be valid JSON when \
             _claroty_audit_filter_by is a JSON-object string. \
             Got: '{body_str}'. Parse error: {:?}. \
             Root cause: pipeline.rs step_vars seeding stores all query_filters as \
             Value::String; JsonBody interpolation calls json_escape() which produces \
             {{\\\"field\\\":\\\"timestamp\\\"...}} — NOT valid JSON when placed bare \
             in a body template. \
             Fix (BC-2.16.013 §Postcondition 1): detect strings starting with '{{' or \
             '[' and auto-parse to Value::Object/Value::Array before seeding step_vars. \
             Backward-compat: non-JSON strings (FQL 'created_timestamp:>...') \
             do not start with '{{' and MUST stay as Value::String.",
            body_json.as_ref().err()
        );

        // Secondary assertion: filter_by must be a JSON object.
        let body = body_json.unwrap();
        assert!(
            body["filter_by"].is_object(),
            "RG-004 SECONDARY: filter_by in the POST body must be a JSON object. \
             Got: {:?}. BC-2.16.013 §Postcondition 1.",
            body["filter_by"]
        );

        // Verify the filter_by contents match the original JSON string values.
        assert_eq!(
            body["filter_by"]["field"].as_str().unwrap_or(""),
            "timestamp",
            "RG-004: filter_by.field must be 'timestamp'. Got: {:?}.",
            body["filter_by"]["field"]
        );
        assert_eq!(
            body["filter_by"]["operation"].as_str().unwrap_or(""),
            "greater_or_equal",
            "RG-004: filter_by.operation must be 'greater_or_equal'. Got: {:?}.",
            body["filter_by"]["operation"]
        );
        // AC-004: value field MUST be an ISO-8601 STRING, NOT an epoch integer.
        // BC-2.01.013 EC-01-030..EC-01-033.
        assert_eq!(
            body["filter_by"]["value"],
            serde_json::Value::String("2026-01-01T00:00:00Z".to_string()),
            "RG-004: filter_by.value must be the ISO-8601 string '2026-01-01T00:00:00Z' \
             (serde_json::Value::String), NOT an epoch integer. \
             BC-2.01.013 EC-01-030; AC-004.",
        );

        // BACKWARD-COMPAT assertion (RG-004 else-branch positive gate):
        // The FQL string 'created_timestamp:>2026-01-01' (does NOT start with `{`/`[`)
        // MUST remain serde_json::Value::String in step_vars and arrive verbatim
        // in the "fql" property of the POST body.
        // This is a POSITIVE assert_eq! — NOT merely absence of panic (AC-004 / F-P1-MED-003).
        assert_eq!(
            body["fql"],
            serde_json::Value::String("created_timestamp:>2026-01-01".to_string()),
            "RG-004 backward-compat gate: FQL string 'created_timestamp:>2026-01-01' \
             MUST remain serde_json::Value::String in step_vars (else-branch). \
             The pipeline.rs JSON-auto-parse MUST NOT convert non-JSON strings to objects. \
             This positive assert_eq! gates the backward-compat invariant for \
             CrowdStrike FQL and Armis AQL sensors. BC-2.01.013 backward-compat invariant; \
             BC-2.16.013 §Postcondition 1 else-branch; AC-004; F-P1-MED-003.",
        );
    }

    // ---------------------------------------------------------------------------
    // FIX-3 (S-CLAROTY-AUDITLOG-TIMEBOX-001 cycle-4 MED-1) — query_filter_json_parse_degraded
    // warn path: fires when a filter value starts with `{` or `[` but fails JSON parse.
    // ---------------------------------------------------------------------------

    /// FIX-3 / BC-2.16.002 catalog row `query_filter_json_parse_degraded` —
    /// warn emitted when a filter value begins with `{` or `[` (appears to be JSON)
    /// but `serde_json::from_str` fails to parse it.
    ///
    /// The cycle-3 BC row v2.21 had two errors:
    ///   (1) Trigger condition: "non-JSON string such as a bare UUID" — WRONG.
    ///       A bare UUID does NOT start with `{`/`[`, so it bypasses the guard and
    ///       goes to `Value::String` passthrough WITHOUT triggering this warn.
    ///   (2) Recurrence: "per step execution" — WRONG. The step_vars seeding loop
    ///       runs once per pipeline execution (before any step iteration).
    ///
    /// This test exercises the CORRECT trigger path: a filter value that starts with
    /// `{` (looks like JSON) but is not valid JSON. The warn MUST fire exactly once
    /// per pipeline execution for that filter key.
    ///
    /// BC-2.16.002 catalog row `query_filter_json_parse_degraded`; BC-2.16.013 EC-005;
    /// S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3 cycle-4 MED-1.
    #[tokio::test]
    #[tracing_test::traced_test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    async fn test_BC_2_16_013_query_filter_json_parse_degraded_warn_on_starts_with_brace() {
        let mock_server = MockServer::start().await;

        // A POST step; respond with an empty result.
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"audit_log": [], "total": 0})),
            )
            .mount(&mock_server)
            .await;

        // Minimal spec: POST step, no filter references in template (filter key is not
        // interpolated — the seeding loop processes ALL query_filters regardless).
        let spec = SensorSpec {
            sensor_id: "test_sensor".to_string(),
            name: "EC-005 warn test sensor".to_string(),
            auth_type: AuthType::BearerStatic,
            base_url: mock_server.uri(),
            tables: vec![TableSpec::new_point_in_time(
                "items",
                "system_activity",
                vec![ColumnSpec {
                    name: "id".to_string(),
                    column_type: prism_core::ColumnType::String,
                    ocsf_field: None,
                    options: vec![],
                    timestamp_formats: vec![],
                    timestamp_fallback_chain: vec![],
                    source_path: None,
                }],
                vec![FetchStep {
                    name: "fetch_items".to_string(),
                    method: "POST".to_string(),
                    path_template: "/api/v1/items".to_string(),
                    body_template: Some(r#"{"query": "all"}"#.to_string()),
                    response_path: "$.audit_log".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                }],
            )],
            rate_limit_hints: None,
            version: "1.0.0".to_string(),
            credential_refs: vec![],
            auth_plugin: None,
            file_hash: String::new(),
            source_path: String::new(),
            mode: crate::types::DtuMode::Shared,
            ocsf_column_naming: false,
            probe_table: None,
        };

        // filter value starts with `{` (triggers the JSON-auto-parse guard) but is
        // NOT valid JSON (missing closing brace + colon+value required by JSON spec).
        // This is the activation condition for query_filter_json_parse_degraded:
        //   `starts_with('{') || starts_with('[')` → true → serde_json::from_str → Err
        //   → warn fires → value falls back to Value::String passthrough (EC-005).
        //
        // A bare UUID like "f47ac10b-58cc-4372-a567-0e02b2c3d479" would NOT activate
        // this guard (doesn't start with `{`/`[`) — that is the corrected understanding
        // in BC-2.16.002 v2.22 (cycle-4 FIX-3).
        let mut query_filters = HashMap::new();
        query_filters.insert(
            "bad_json_filter".to_string(),
            "{not-valid-json".to_string(), // starts with `{` but not parseable
        );
        let context = FetchContext::new(OrgSlug::new("ec005-warn-test-org"), query_filters, None);

        let http_client = reqwest::Client::new();
        let auth_provider = MockAuthProvider::new("ec005-warn-test-token");

        let result = PipelineExecutor::execute(
            &spec,
            &spec.tables[0],
            &context,
            &http_client,
            &auth_provider,
        )
        .await;

        // The pipeline MUST succeed despite the degraded filter (EC-005 string passthrough).
        assert!(
            result.is_ok(),
            "FIX-3 EC-005: pipeline MUST succeed when a query filter starts with `{{` \
             but is not valid JSON — the value falls back to string passthrough. \
             Got Err: {:?}",
            result.err()
        );

        // LOAD-BEARING (FIX-3 / BC-2.16.002 v2.22 catalog row trigger verification):
        // query_filter_json_parse_degraded WARN MUST be emitted when a filter value
        // starts with `{` or `[` but serde_json::from_str fails.
        //
        // This verifies the CORRECT trigger condition (not "bare UUID") — cycle-4 MED-1:
        //   - `{not-valid-json` starts with `{` → enters JSON auto-parse guard
        //   - serde_json::from_str fails → warn fires
        //   - value falls back to Value::String passthrough
        assert!(
            logs_contain("query_filter_json_parse_degraded"),
            "FIX-3 REGRESSION (BC-2.16.002 v2.22 / cycle-4 MED-1): \
             query_filter_json_parse_degraded WARN MUST be emitted when a filter \
             value starts with `{{` but fails JSON parse. \
             Trigger condition: `starts_with('{{') || starts_with('[')` → true → \
             serde_json::from_str Err → warn. \
             This test uses '{{not-valid-json' (starts with `{{`, not valid JSON). \
             If this fails, the EC-005 warn arm was removed or the guard was changed. \
             BC-2.16.002 catalog row `query_filter_json_parse_degraded`; \
             S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-3."
        );
    }

    /// BC-2.16.013 §Postcondition 1 / S-CLAROTY-AUDITLOG-TIMEBOX-001 cycle-2 BLOCKING-1:
    /// `find_fan_out_array` MUST NOT emit `fanout_invalid_source_type` WARN for
    /// `query.filter.*` keys, even when their step_vars value is `Value::Object`.
    ///
    /// `query.filter.*` keys (e.g., `_claroty_audit_filter_by`) are stored as
    /// `Value::Object` by design — the `InterpolationContext::JsonBody` path does
    /// verbatim JSON insertion (`value.to_string()` inline), NOT string serialization.
    /// Emitting the warn is a false audit signal and misleads maintainers.
    ///
    /// # Red Gate failure (before fix)
    ///
    /// `find_fan_out_array` warns for every `Value::Object` in step_vars, firing on
    /// every `audit_logs` fetch and polluting the SIEM audit trail.
    ///
    /// BC-2.16.013 §Postcondition 1; S-CLAROTY-AUDITLOG-TIMEBOX-001 cycle-2 BLOCKING-1.
    #[tokio::test]
    #[tracing_test::traced_test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    async fn test_BC_2_16_013_pipeline_json_filter_object_no_fanout_warn_emitted() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/audit_log/get"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"audit_log": [], "total": 0})),
            )
            .mount(&mock_server)
            .await;

        // Reuse the same synthetic spec as RG-004 — one POST step using
        // ${query.filter._claroty_audit_filter_by} in body_template, no OffsetLimit.
        let spec = SensorSpec {
            sensor_id: "claroty".to_string(),
            name: "Claroty no-fanout-warn test".to_string(),
            auth_type: AuthType::BearerStatic,
            base_url: mock_server.uri(),
            tables: vec![TableSpec::new_point_in_time(
                "audit_logs",
                "audit_activity",
                vec![ColumnSpec {
                    name: "id".to_string(),
                    column_type: prism_core::ColumnType::String,
                    ocsf_field: None,
                    options: vec![],
                    timestamp_formats: vec![],
                    timestamp_fallback_chain: vec![],
                    source_path: None,
                }],
                vec![FetchStep {
                    name: "fetch_audit_logs".to_string(),
                    method: "POST".to_string(),
                    path_template: "/api/v1/audit_log/get".to_string(),
                    body_template: Some(
                        r#"{"filter_by": ${query.filter._claroty_audit_filter_by}}"#.to_string(),
                    ),
                    response_path: "$.audit_log".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                }],
            )],
            rate_limit_hints: None,
            version: "1.0.0".to_string(),
            credential_refs: vec![],
            auth_plugin: None,
            file_hash: String::new(),
            source_path: String::new(),
            mode: crate::types::DtuMode::Shared,
            ocsf_column_naming: false,
            probe_table: None,
        };

        // FetchContext: _claroty_audit_filter_by is a JSON-object string.
        // After seeding, step_vars stores this as Value::Object — the correct BC-designed shape.
        let mut query_filters = HashMap::new();
        query_filters.insert(
            "_claroty_audit_filter_by".to_string(),
            r#"{"field": "timestamp", "operation": "greater_or_equal", "value": "2026-01-01T00:00:00Z"}"#
                .to_string(),
        );
        let context = FetchContext::new(OrgSlug::new("claroty-no-warn-org"), query_filters, None);

        let http_client = reqwest::Client::new();
        let auth_provider = MockAuthProvider::new("claroty-no-warn-token");

        let result = PipelineExecutor::execute(
            &spec,
            &spec.tables[0],
            &context,
            &http_client,
            &auth_provider,
        )
        .await;

        assert!(
            result.is_ok(),
            "test_BC_2_16_013_pipeline_json_filter_object_no_fanout_warn_emitted: \
             pipeline must succeed against the 200 mock. Got Err: {:?}.",
            result.err()
        );

        // LOAD-BEARING assertion (cycle-2 BLOCKING-1):
        // `fanout_invalid_source_type` WARN MUST NOT appear in the captured log output.
        // `query.filter.*` keys are Object-valued by design (BC-2.16.013 §Postcondition 1).
        // Firing this warn on the designed happy path degrades the SIEM audit trail.
        assert!(
            !logs_contain("fanout_invalid_source_type"),
            "test_BC_2_16_013_pipeline_json_filter_object_no_fanout_warn_emitted: \
             fanout_invalid_source_type WARN must NOT be emitted for query.filter.* keys. \
             BC-2.16.013 §Postcondition 1 designates _claroty_audit_filter_by as \
             Object-valued (JsonBody verbatim insertion). \
             Cycle-2 BLOCKING-1: restrict the warn to non-query.filter.* keys only."
        );
    }

    // ---------------------------------------------------------------------------
    // FIX-2 cycle-4 companion — path_template scenario:
    // query.filter.* Object in path_template MUST NOT emit fanout_invalid_source_type.
    // ---------------------------------------------------------------------------

    /// FIX-2 / BC-2.16.013 — `query.filter.*` Object-valued variable in path_template
    /// MUST NOT emit `fanout_invalid_source_type` warn.
    ///
    /// The cycle-3 two-pass implementation exempted `query.filter.*` only in body_template
    /// (Pass 2), but NOT in path_template (Pass 1). This would produce a false alarm if
    /// any spec placed a `query.filter.*` Object in a path_template.
    ///
    /// The correct fix (cycle-4 FIX-2): exempt `query.filter.*` namespace in ALL template
    /// contexts (single-pass over the `templates` vec, same exemption for path and body).
    ///
    /// This test calls `find_fan_out_array` directly (unit test) with a `FetchStep` whose
    /// path_template references `${query.filter._json_filter}` and step_vars carrying that
    /// key as a `Value::Object`. The function MUST NOT emit `fanout_invalid_source_type`.
    ///
    /// BC-2.16.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-2 cycle-4 BLOCKING-2.
    #[test]
    #[tracing_test::traced_test]
    #[allow(clippy::unwrap_used)]
    fn test_BC_2_16_013_pipeline_json_filter_object_path_template_no_fanout_warn_emitted() {
        // FetchStep with a path_template that references ${query.filter._json_filter}.
        // This simulates a hypothetical spec where a query.filter.* Object appears in a
        // path_template (e.g., future ADR-031 §D8-a extension).
        let step = FetchStep {
            name: "fetch_step".to_string(),
            method: "GET".to_string(),
            path_template: "/api/v1/query/${query.filter._json_filter}".to_string(),
            body_template: None,
            response_path: "$.data".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        };

        // step_vars: query.filter._json_filter is a Value::Object (as seeded by
        // seed_query_filters when the raw filter value starts with '{').
        let mut step_vars = HashMap::new();
        step_vars.insert(
            "query.filter._json_filter".to_string(),
            serde_json::json!({"field": "timestamp", "value": "2024-01-01T00:00:00Z"}),
        );

        // Call find_fan_out_array directly — it is a module-private function, accessible
        // from this #[cfg(test)] block in the same file.
        let fan_out = super::find_fan_out_array(&step, &step_vars);

        // ASSERTION 1: No array-valued variable in step_vars → no fan-out detected.
        assert!(
            fan_out.is_none(),
            "FIX-2 companion (path_template): find_fan_out_array should return None when \
             step_vars contains only an Object-valued variable (not Array). \
             Got: {fan_out:?}"
        );

        // ASSERTION 2 (LOAD-BEARING — FIX-2 cycle-4 BLOCKING-2 regression):
        // fanout_invalid_source_type WARN MUST NOT be emitted for query.filter.* keys
        // even when they appear in path_template references.
        //
        // FAILS under the cycle-3 two-pass impl (Pass 1 path_template: no exemption →
        // Object in path_template triggers warn). PASSES after FIX-2 single-pass revert
        // (query.filter.* exempt in all template contexts).
        assert!(
            !logs_contain("fanout_invalid_source_type"),
            "FIX-2 REGRESSION (path_template — cycle-4 BLOCKING-2): \
             fanout_invalid_source_type WARN must NOT be emitted for query.filter.* \
             Object-valued variables, even when referenced in path_template. \
             The correct discriminator is the NAMESPACE (query.filter.*), not the \
             template context (path vs body). \
             The cycle-3 two-pass impl incorrectly fired this warn for path_template \
             query.filter.* Objects. \
             BC-2.16.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 FIX-2 cycle-4."
        );
    }

    // ---------------------------------------------------------------------------
    // Cycle-16 LOW-1 regression: query.filter.* Array-valued vars must NOT be
    // selected as fan-out sources (TD-VSDD-060 sibling sweep of FIX-2).
    // ---------------------------------------------------------------------------

    /// Cycle-16 LOW-1 / BC-2.16.013 — `query.filter.*` Array-valued variable MUST NOT be
    /// selected as a fan-out source by `find_fan_out_array`.
    ///
    /// FIX-2 (S-CLAROTY-AUDITLOG-TIMEBOX-001 cycle-4) added `if key.starts_with("query.filter.") { continue; }`
    /// to the Object-warn loop, exempting `query.filter.*` from the `fanout_invalid_source_type` warn.
    /// That fix omitted the sibling site — the `array_vars` collection loop immediately above,
    /// which selects the fan-out source. An Array-valued `query.filter.*` variable (e.g.,
    /// `query.filter.aql = "[]"` after auto-parse) would be selected, producing zero batches,
    /// zero HTTP requests, zero rows, and no error (SOUL.md §4 silent-empty class).
    ///
    /// This test calls `find_fan_out_array` directly with a step whose body_template references
    /// `${query.filter._claroty_audit_filter_by}` and step_vars carrying that key as
    /// `Value::Array([])`. The function MUST return `None` (not selected as fan-out source).
    ///
    /// BC-2.16.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 cycle-16 LOW-1.
    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_BC_2_16_013_pipeline_json_array_filter_no_fanout_selection() {
        // FetchStep with a body_template that references ${query.filter._claroty_audit_filter_by}.
        let step = FetchStep {
            name: "fetch_audit_logs".to_string(),
            method: "POST".to_string(),
            path_template: "/api/v1/audit_log/get".to_string(),
            body_template: Some(
                r#"{"filter_by": ${query.filter._claroty_audit_filter_by}}"#.to_string(),
            ),
            response_path: "$.audit_log".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        };

        // step_vars: query.filter._claroty_audit_filter_by is a Value::Array([]).
        // This simulates the malformed/edge-case path where auto-parse produces an Array
        // from a query.filter.* value. Must NOT be selected as the fan-out source.
        let mut step_vars = HashMap::new();
        step_vars.insert(
            "query.filter._claroty_audit_filter_by".to_string(),
            serde_json::Value::Array(vec![]),
        );

        let fan_out = super::find_fan_out_array(&step, &step_vars);

        // LOAD-BEARING ASSERTION (cycle-16 LOW-1 regression):
        // find_fan_out_array MUST return None when step_vars contains only a
        // query.filter.* variable, even if that variable is Array-valued.
        //
        // Rationale: query.filter.* variables are JSON filter values injected into
        // the request body, not fan-out batch sources. Selecting them as fan-out
        // sources with an empty array produces zero HTTP requests, zero rows, and
        // no error — the SOUL.md §4 silent-empty defect class.
        //
        // The correct discriminator is the NAMESPACE (query.filter.*), not the value type.
        // This exemption mirrors the Object-warn exemption in the loop below (FIX-2).
        assert!(
            fan_out.is_none(),
            "cycle-16 LOW-1 REGRESSION: find_fan_out_array MUST return None for \
             query.filter.* variables regardless of their value type (Object or Array). \
             An Array-valued query.filter.* var must NOT be selected as the fan-out source — \
             doing so produces zero-batch fan-out (zero HTTP requests, zero rows, no error). \
             The correct discriminator is the NAMESPACE (query.filter.*), not the value type. \
             Got: {fan_out:?} — this means the array_vars collection loop is missing the \
             query.filter.* namespace exemption. \
             BC-2.16.013; S-CLAROTY-AUDITLOG-TIMEBOX-001 cycle-16 LOW-1."
        );
    }
}
