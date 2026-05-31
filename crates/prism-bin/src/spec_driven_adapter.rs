// SPDX-License-Identifier: Apache-2.0
//! SpecDrivenSensorAdapter — bridges PipelineExecutor to AdapterRegistry (S-DEMO-001).
//!
//! Closes GAP-002-A: wires sensor TOML specs loaded at boot into the `AdapterRegistry`
//! so that `fan_out()` in prism-query can resolve `(org_id, sensor_id) → Arc<dyn SensorAdapter>`
//! and return live Arrow data from the DTU clones.
//!
//! # Architecture Compliance
//!
//! - MUST live in `prism-bin` (NOT `prism-sensors`) per ADR-023 §D3 Forbidden Dependencies.
//!   `prism-sensors` MUST NOT import `prism-spec-engine`; only `prism-bin` imports both.
//! - `BearerStaticAuthProvider` lives here (NOT in `prism-spec-engine`) because it bridges
//!   `SensorAuth` (prism-sensors) ↔ `AuthProvider` (prism-spec-engine). Only prism-bin imports both.
//! - `StaticCookieAuthProvider` lives in `prism-spec-engine/src/auth_provider.rs`.
//!
//! # Auth strategies (OQ-1 Resolution)
//!
//! `AdapterAuthStrategy` is held at construction time:
//! - `Plugin(Arc<dyn AuthProvider>)` — CrowdStrike: held PluginAuthProvider, ignores SensorAuth arg.
//! - `BearerStatic` — Armis/Claroty: token extracted from SensorAuth arg at fetch() call time.
//! - `StaticCookie(Arc<dyn AuthProvider>)` — Cyberint: held StaticCookieAuthProvider (NO HTTP calls
//!   at acquire_token; reads api_key from credential store). Renamed from CookieLogin (v1.1/v1.2)
//!   per ADR-031 §D3 / DEMO-001 v1.3. NOT `CookieLogin` — that required a login step.
//!
//! # reqwest::Client timeout
//!
//! `SpecDrivenSensorAdapter::new` constructs the HTTP client with `.timeout(Duration::from_secs(30))`
//! per CLAUDE.md conventions (TD-S-PLUGIN-PREREQ-B-005 closure requirement).
//!
//! BCs: BC-2.01.013, BC-2.11.005, BC-2.06.014, BC-2.22.001
//! Story: S-DEMO-001 v1.3

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, time::Duration};

use serde_json::Value as JsonValue;

use arrow::{
    array::{Array, BooleanArray, Float64Array, Int32Array, Int64Array, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use async_trait::async_trait;
use prism_core::{ColumnType, OrgId, SensorId};
use prism_ocsf::EventClassSelector;
use prism_sensors::{
    BearerStaticSensorAuth, SensorAdapter,
    adapter::{QueryParams, SensorError, SensorSpec},
    auth::SensorAuth,
};
use prism_spec_engine::{
    AuthProvider, AuthToken, PluginAuthProvider, ResolvedSensorSpec, ResolvedSpecKey,
    error::SpecEngineError,
    pipeline::{FetchContext, PipelineExecutor, PipelineResult},
    spec_parser::{AuthType, SensorSpec as SpecEngineSensorSpec, TableSpec},
};

// ---------------------------------------------------------------------------
// AdapterAuthStrategy — auth path selected at construction time (OQ-1)
// ---------------------------------------------------------------------------

/// Auth strategy held by a `SpecDrivenSensorAdapter` instance.
///
/// Selected once at boot step 9A construction time based on the sensor spec's `auth_type`:
/// - `Plugin` — CrowdStrike: held `PluginAuthProvider` from step 7.5b; ignores `SensorAuth` arg.
/// - `BearerStatic` — Armis/Claroty: token extracted from `SensorAuth::BearerStatic` arg per-fetch.
/// - `StaticCookie` — Cyberint: held `StaticCookieAuthProvider` (no HTTP calls at acquire_token).
///   NOTE: v1.1/v1.2 called this `CookieLogin`. Renamed `StaticCookie` per ADR-031 §D3 (S-DEMO-001 v1.3).
///   The old `CookieLogin` variant made HTTP calls to `POST /login` — WRONG under ADR-031 DTU=true-DTU.
///
/// BC-2.01.013 postcondition 4; OQ-1 Resolution (S-DEMO-001 §OQ-1).
#[derive(Clone)]
pub enum AdapterAuthStrategy {
    /// CrowdStrike: WASM plugin auth via held `Arc<dyn AuthProvider>` (`PluginAuthProvider`).
    /// The `SensorAuth` argument to `fetch()` is ignored for plugin-authed sensors (ADR-028 §D10).
    Plugin(Arc<dyn AuthProvider>),
    /// Armis / Claroty: `bearer_static` auth type. Token is extracted from the `SensorAuth`
    /// argument at each `fetch()` call (per-fetch `BearerStaticAuthProvider` construction).
    /// No held auth state — the token lives in the credential store and arrives at fetch time.
    BearerStatic,
    /// Cyberint: `cookie_roundtrip` auth type. Held `StaticCookieAuthProvider` reads the API
    /// key from the credential store at `acquire_token()` time with NO HTTP call (ADR-031 §D1-b).
    /// The token is injected by `PipelineExecutor::build_request` as `Cookie: access_token={token}`
    /// (NOT `cyberint_session` — that was the pre-ADR-031 DTU model; permanently superseded).
    ///
    /// ADR-031 §D3-b; S-DEMO-001 v1.3.
    StaticCookie(Arc<dyn AuthProvider>),
}

// ---------------------------------------------------------------------------
// BearerStaticAuthProvider — thin AuthProvider wrapper for bearer_static sensors
// ---------------------------------------------------------------------------

/// Thin `AuthProvider` wrapper for `bearer_static` sensors (Armis, Claroty).
///
/// Lives in `prism-bin` (NOT `prism-spec-engine`) because it bridges `SensorAuth` from
/// `prism-sensors` ↔ `AuthProvider` from `prism-spec-engine`. Only `prism-bin` imports both
/// crates (ADR-023 §Permitted Patterns).
///
/// Constructed **per-fetch** from the `SensorAuth::BearerStatic { token }` argument at
/// `SpecDrivenSensorAdapter::fetch()` call time (OQ-1 Resolution — Option A per-fetch).
///
/// The token is not held at construction time of `SpecDrivenSensorAdapter` — it arrives
/// at query time via the `SensorAuth` argument (ADR-022 §C wiring; AD-017 credential safety).
///
/// BC-2.01.013 postcondition 4; ADR-023 §Permitted Patterns; OQ-1.
pub struct BearerStaticAuthProvider {
    /// Bearer token string.
    ///
    /// AD-017: this field holds the bearer token for the duration of a single fetch() call.
    /// The token is NOT stored at SpecDrivenSensorAdapter construction time.
    token: String,
}

impl BearerStaticAuthProvider {
    /// Construct a `BearerStaticAuthProvider` from a bearer token string.
    ///
    /// Called per-fetch from `SpecDrivenSensorAdapter::fetch()` when `auth_strategy` is
    /// `AdapterAuthStrategy::BearerStatic`. Token comes from the `SensorAuth` argument.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl AuthProvider for BearerStaticAuthProvider {
    /// Return the bearer token as an `AuthToken`.
    ///
    /// Zero I/O. No branching. Returns the held token wrapped in `AuthToken::new`.
    /// `PipelineExecutor::build_request` injects it as `Authorization: Bearer {token}`
    /// for `BearerStatic` sensors.
    ///
    /// WIRING-EXEMPT: this is a pure delegation to `AuthToken::new(token)` — no logic,
    /// no I/O, no branching. The box-pin wrapper is required for dyn-compatibility per
    /// `AuthProvider` trait contract (BC-2.16.002 object-safety).
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SpecEngineSensorSpec,
        _client_id: &'a prism_core::OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        let token = self.token.clone();
        Box::pin(async move { Ok(AuthToken::new(token)) })
    }
}

// ---------------------------------------------------------------------------
// SpecDrivenSensorAdapter
// ---------------------------------------------------------------------------

/// Spec-driven `SensorAdapter` that delegates to `PipelineExecutor::execute()`.
///
/// Registered in `AdapterRegistry` at boot step 9A for every `(OrgId, SensorId)` pair
/// present in the resolved spec catalog. Closes GAP-002-A — enables `fan_out()` to
/// return live Arrow RecordBatches from the DTU clones instead of `AdapterNotFound`.
///
/// ## Auth dispatch (BC-2.01.013 postcondition 4)
///
/// - `Plugin` strategy: passes the held `Arc<PluginAuthProvider>`; ignores `SensorAuth` arg
///   (ADR-028 §D10 — plugin-authed sensors provide their own auth at the plugin level).
/// - `BearerStatic` strategy: constructs `BearerStaticAuthProvider` per-call from the
///   `SensorAuth` argument's token field (OQ-1 Resolution — per-fetch construction).
/// - `StaticCookie` strategy: uses the held `StaticCookieAuthProvider`; ignores `SensorAuth` arg.
///   `acquire_token()` reads the API key from the credential store with NO HTTP call (ADR-031 §D1-b).
///
/// ## reqwest::Client
///
/// Constructed via `reqwest::Client::builder().timeout(Duration::from_secs(30)).build()`
/// per CLAUDE.md conventions (TD-S-PLUGIN-PREREQ-B-005).
///
/// ## OCSF normalization
///
/// The `PipelineExecutor` does NOT return Arrow `RecordBatch` — it returns `PipelineResult`
/// (raw JSON records). `SpecDrivenSensorAdapter::fetch()` maps those records through
/// `ColumnMapper` + `OcsfNormalizer` to produce `Vec<RecordBatch>` (BC-2.11.005 postcondition).
///
/// BCs: BC-2.01.013, BC-2.06.014, BC-2.11.005; Story: S-DEMO-001 v1.3.
pub struct SpecDrivenSensorAdapter {
    /// The resolved sensor spec (with per-org overlay applied).
    ///
    /// Contains the TYPE spec merged with per-org scalar overrides (base_url, timeout).
    /// Immutable after construction — shared via Arc for O(1) dispatch (INV-FANOUT-002).
    sensor_spec: Arc<ResolvedSensorSpec>,

    /// Auth strategy for this adapter (held at construction time, not per-fetch).
    ///
    /// OQ-1 Resolution: BearerStatic extracts the token from the SensorAuth arg at fetch time;
    /// Plugin and StaticCookie hold their providers here.
    auth_strategy: AdapterAuthStrategy,

    /// HTTP client with 30-second timeout (TD-S-PLUGIN-PREREQ-B-005).
    ///
    /// Constructed via `reqwest::Client::builder().timeout(Duration::from_secs(30)).build()`
    /// in `SpecDrivenSensorAdapter::new()`. MUST NOT be a global singleton (ADR-022 §C).
    http_client: reqwest::Client,
}

impl SpecDrivenSensorAdapter {
    /// Construct a `SpecDrivenSensorAdapter`.
    ///
    /// # Parameters
    ///
    /// - `sensor_spec` — The resolved sensor spec (TYPE + per-org overlay). Shared via Arc.
    /// - `auth_strategy` — Auth strategy selected based on `sensor_spec.spec.auth_type`.
    /// - `http_client` — `reqwest::Client` with 30s timeout (TD-S-PLUGIN-PREREQ-B-005).
    ///   MUST be constructed with `.timeout(Duration::from_secs(30))`.
    ///
    /// Production callers pass a client constructed by `build_http_client_with_timeout()`.
    /// Tests inject a client directed at a mock HTTP server.
    ///
    /// BC-2.01.013; ADR-022 §C wiring; OQ-1 Resolution.
    pub fn new(
        sensor_spec: Arc<ResolvedSensorSpec>,
        auth_strategy: AdapterAuthStrategy,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            sensor_spec,
            auth_strategy,
            http_client,
        }
    }
}

#[async_trait]
impl SensorAdapter for SpecDrivenSensorAdapter {
    /// Returns the sensor ID for this adapter.
    ///
    /// Used by `AdapterRegistry::register()` to key the adapter.
    ///
    /// GREEN-BY-DESIGN: zero branching, no I/O, no non-trivial helpers, 1 line.
    /// Criteria: (1) no `if`/`match`/`?`/`unwrap`, (2) no I/O, (3) only type constructor,
    /// (4) 1 line body. BC-5.38.002 — this test passes trivially and that is correct-by-construction.
    fn sensor_type(&self) -> SensorId {
        SensorId::from(self.sensor_spec.spec.sensor_id.as_str())
    }

    /// Returns a human-readable sensor name for tracing spans and error messages.
    ///
    /// Returns `"spec_driven"` as a static discriminator for this adapter type.
    /// Per-sensor names are available via `sensor_type()` (returns the SensorId).
    ///
    /// GREEN-BY-DESIGN: zero branching, no I/O, no non-trivial helpers, 1 line.
    fn sensor_name(&self) -> &'static str {
        "spec_driven"
    }

    /// Fetch sensor data by delegating to `PipelineExecutor::execute()`.
    ///
    /// Dispatches authentication by `auth_strategy`:
    /// - `Plugin`: passes the held `Arc<PluginAuthProvider>`; ignores `auth` arg (ADR-028 §D10).
    /// - `BearerStatic`: extracts bearer token from `auth: &dyn SensorAuth` via downcast to
    ///   `BearerStaticSensorAuth`, constructs `BearerStaticAuthProvider` per-call (OQ-1 Resolution).
    /// - `StaticCookie`: uses the held `StaticCookieAuthProvider` (NO HTTP calls at
    ///   acquire_token; `build_request` injects `Cookie: access_token={token}` per ADR-031 §D3-b).
    ///
    /// Maps `PipelineResult` (raw JSON) → `Vec<RecordBatch>` via OCSF normalization (BC-2.11.005).
    /// Each table in the sensor spec is executed sequentially; results are concatenated.
    ///
    /// On double-401: propagates `SpecEngineError::AuthRefreshFailed` → `SensorError::Internal`
    /// (BC-2.01.013 error case; AC-012).
    ///
    /// BC-2.01.013 postcondition 4; OQ-1 Resolution; ADR-028 §D10; ADR-031 §D3-b.
    async fn fetch(
        &self,
        _spec: &SensorSpec,
        params: &QueryParams,
        auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // Select the auth provider based on the held auth_strategy (OQ-1 Resolution).
        // ADR-028 §D10: Plugin and StaticCookie ignore the SensorAuth arg.
        // BearerStatic extracts the token from the SensorAuth arg via downcast.
        let auth_provider: Arc<dyn AuthProvider> = match &self.auth_strategy {
            AdapterAuthStrategy::Plugin(provider) => {
                // CrowdStrike: use the held PluginAuthProvider; ignore SensorAuth arg (ADR-028 §D10).
                Arc::clone(provider)
            }
            AdapterAuthStrategy::BearerStatic => {
                // Armis/Claroty: extract bearer token from SensorAuth arg via downcast (OQ-1).
                // The production path expects &BearerStaticSensorAuth from prism-sensors.
                // If downcast fails (e.g., test-local stub type), return Internal error.
                let bearer_auth = auth
                    .as_any()
                    .downcast_ref::<BearerStaticSensorAuth>()
                    .ok_or_else(|| SensorError::Internal {
                        detail: format!(
                            "E-SPEC-012: BearerStatic auth strategy requires BearerStaticSensorAuth; \
                             got auth_type_name='{}'. Ensure the caller passes a BearerStaticSensorAuth \
                             instance for bearer_static sensors. S-DEMO-001 OQ-1.",
                            auth.auth_type_name()
                        ),
                    })?;
                Arc::new(BearerStaticAuthProvider::new(bearer_auth.token.clone()))
            }
            AdapterAuthStrategy::StaticCookie(provider) => {
                // Cyberint: use the held StaticCookieAuthProvider; ignore SensorAuth arg (ADR-028 §D10).
                Arc::clone(provider)
            }
        };

        // Build the FetchContext from the resolved spec's org_slug.
        // Thread params.filters into FetchContext.query_filters (F-003).
        // FilterMap = HashMap<String, serde_json::Value>; FetchContext.query_filters
        // = HashMap<String, String>. Convert by serializing Value → String.
        // The client_id for the pipeline is the org_slug (human-readable org identifier).
        let query_filters: std::collections::HashMap<String, String> = params
            .filters
            .iter()
            .map(|(k, v)| {
                let s = match v {
                    JsonValue::String(s) => s.clone(),
                    other => other.to_string(),
                };
                (k.clone(), s)
            })
            .collect();
        let context = FetchContext::new(self.sensor_spec.org_slug.clone(), query_filters);

        // Delegate to PipelineExecutor::execute() for each table in the sensor spec.
        // Collect all RecordBatches by normalizing JSON records → Arrow (BC-2.11.005).
        let mut all_batches: Vec<RecordBatch> = Vec::new();

        for table in &self.sensor_spec.spec.tables {
            let result = PipelineExecutor::execute(
                &self.sensor_spec.spec,
                table,
                &context,
                &self.http_client,
                auth_provider.as_ref(),
            )
            .await
            .map_err(|e| {
                map_spec_engine_error_to_sensor_error(
                    e,
                    &self.sensor_spec.spec.sensor_id,
                    &table.table_name,
                )
            })?;

            // Convert PipelineResult.records (raw JSON) → Arrow RecordBatch
            // with OCSF envelope columns (category_uid, class_uid, _sensor) and
            // spec-defined data columns (BC-2.11.005 / AC-010).
            // BC-2.01.013 v1.8 OCSF Conformance: pass `table` so that:
            //   - spec-declared columns survive into the Arrow schema (item 1)
            //   - class_uid/category_uid are derived from ocsf_class (item 2)
            //   - _sensor is injected as canonical sensor_id (item 3)
            if !result.records.is_empty() {
                let batch = pipeline_result_to_record_batch(
                    result,
                    table,
                    &self.sensor_spec.spec.sensor_id,
                );
                match batch {
                    Ok(b) => all_batches.push(b),
                    Err(e) => {
                        return Err(SensorError::Internal {
                            detail: format!(
                                "SpecDrivenSensorAdapter: OCSF→Arrow normalization failed for \
                                 sensor='{}' table='{}': {e}",
                                self.sensor_spec.spec.sensor_id, table.table_name
                            ),
                        });
                    }
                }
            }
        }

        Ok(all_batches)
    }
}

// ---------------------------------------------------------------------------
// map_spec_engine_error_to_sensor_error — error taxonomy mapping (AC-012 / F-004)
// ---------------------------------------------------------------------------

/// Map `SpecEngineError` → `SensorError`, preserving the E-AUTH-002 taxonomy code.
///
/// AC-012 requirement: double-401 (`AuthRefreshFailed`) must map to `SensorError::Internal`
/// with `detail` containing "E-AUTH-002" so callers can distinguish auth failures from
/// other sensor errors (error taxonomy BC-2.01.013 §Error Cases).
///
/// All `SpecEngineError` variants map to `SensorError::Internal` with a structured
/// `detail` message that includes the sensor id, table name, and the original error's
/// `Display` representation. `AuthRefreshFailed` always includes "E-AUTH-002" in `detail`
/// because that is its `#[error(...)]` prefix in `SpecEngineError`.
fn map_spec_engine_error_to_sensor_error(
    e: SpecEngineError,
    sensor_id: &str,
    table_name: &str,
) -> SensorError {
    // SpecEngineError::Display for AuthRefreshFailed starts with "E-AUTH-002: auth refresh failed..."
    // No special-casing needed: the Display impl preserves the taxonomy code.
    SensorError::Internal {
        detail: format!(
            "SpecDrivenSensorAdapter: PipelineExecutor::execute failed for \
             sensor='{sensor_id}' table='{table_name}': {e}",
        ),
    }
}

// ---------------------------------------------------------------------------
// pipeline_result_to_record_batch — OCSF→Arrow normalization (AC-010 / BC-2.11.005)
// ---------------------------------------------------------------------------

/// Convert `PipelineResult.records` (raw JSON) to an Arrow `RecordBatch`.
///
/// Produces a RecordBatch with (BC-2.01.013 v1.8 OCSF Conformance Clause):
///
/// **Spec-declared data columns (item 1):**
/// Every column declared in `table.columns` is included in the schema, extracted
/// from the raw record by name, and typed per `ColumnSpec::column_type`.
/// Columns absent from a record become null.
///
/// **Derived OCSF envelope columns (item 2):**
/// - `class_uid` (Int32): derived via `EventClassSelector::select(sensor_id, ocsf_class)`.
///   Falls back to 0 (BASE_EVENT) if no mapping exists for this sensor/class combination.
/// - `category_uid` (Int32): `class_uid / 1000` per the OCSF standard category encoding.
///
/// **Canonical _sensor virtual column (item 3):**
/// - `_sensor` (Utf8): always injected as the canonical `sensor_id` from the spec.
///   The raw record's `_sensor` field (if any) is NEVER used — this field can be
///   tampered by the sensor vendor. The spec `sensor_id` is the authoritative value.
///
/// # Column ordering in schema
///
/// Spec-declared data columns appear first (in spec order), followed by the three
/// OCSF envelope columns: `category_uid`, `class_uid`, `_sensor`.
///
/// # Design Rationale (BC-2.11.005)
///
/// `PipelineExecutor::execute()` returns raw JSON; Arrow conversion happens here
/// (not in the query engine's materialization layer). This is the boundary where
/// raw sensor data becomes typed Arrow data with enforced OCSF provenance.
///
/// # Errors
///
/// Returns `arrow::error::ArrowError` if schema/column construction fails.
fn pipeline_result_to_record_batch(
    result: PipelineResult,
    table: &TableSpec,
    sensor_id: &str,
) -> Result<RecordBatch, arrow::error::ArrowError> {
    let n = result.records.len();

    // BC-2.01.013 v1.8 item 2: derive class_uid from spec ocsf_class via EventClassSelector.
    // Falls back to 0 (BASE_EVENT) if no mapping — never reads class_uid from raw record.
    let derived_class_uid: i32 =
        EventClassSelector::select(sensor_id, &table.ocsf_class).unwrap_or(0) as i32;
    // OCSF standard encoding: category_uid = class_uid / 1000
    // e.g. class_uid=2004 → category_uid=2 (Findings), class_uid=3001 → category_uid=3 (IAM)
    let derived_category_uid: i32 = derived_class_uid / 1000;

    // Build schema: spec-declared data columns first, then OCSF envelope.
    let mut fields: Vec<Field> = table
        .columns
        .iter()
        .map(|col| Field::new(&col.name, column_type_to_arrow(&col.column_type), true))
        .collect();
    fields.push(Field::new("category_uid", DataType::Int32, true));
    fields.push(Field::new("class_uid", DataType::Int32, true));
    fields.push(Field::new("_sensor", DataType::Utf8, true));
    let schema = Arc::new(Schema::new(fields));

    if n == 0 {
        // Caller should not call this with empty records; return empty batch.
        let mut arrays: Vec<Arc<dyn Array>> = table
            .columns
            .iter()
            .map(|col| empty_arrow_array(&col.column_type))
            .collect();
        arrays.push(Arc::new(Int32Array::from(Vec::<Option<i32>>::new())) as Arc<dyn Array>);
        arrays.push(Arc::new(Int32Array::from(Vec::<Option<i32>>::new())) as Arc<dyn Array>);
        arrays.push(Arc::new(StringArray::from(Vec::<Option<&str>>::new())) as Arc<dyn Array>);
        return RecordBatch::try_new(schema, arrays);
    }

    // Build per-column value vectors for spec-declared data columns.
    // Each column is extracted from the raw record by name; absent values → None (null).
    let mut col_arrays: Vec<Arc<dyn Array>> = Vec::with_capacity(table.columns.len() + 3);

    for col_spec in &table.columns {
        let array = build_column_array(&result.records, &col_spec.name, &col_spec.column_type);
        col_arrays.push(array);
    }

    // BC-2.01.013 v1.8 item 2: OCSF envelope — class_uid/category_uid derived, not raw-copied.
    // All rows in this batch share the same derived class_uid/category_uid (table-level, not row-level).
    let category_uid_vals: Vec<Option<i32>> = vec![Some(derived_category_uid); n];
    let class_uid_vals: Vec<Option<i32>> = vec![Some(derived_class_uid); n];
    col_arrays.push(Arc::new(Int32Array::from(category_uid_vals)) as Arc<dyn Array>);
    col_arrays.push(Arc::new(Int32Array::from(class_uid_vals)) as Arc<dyn Array>);

    // BC-2.01.013 v1.8 item 3: _sensor is ALWAYS the canonical sensor_id from the spec.
    // Never reads from raw record — the raw record's _sensor field is untrusted vendor data.
    let sensor_vals: Vec<Option<&str>> = vec![Some(sensor_id); n];
    col_arrays.push(Arc::new(StringArray::from(sensor_vals)) as Arc<dyn Array>);

    RecordBatch::try_new(schema, col_arrays)
}

/// Map a `ColumnType` to the corresponding Arrow `DataType`.
///
/// Used to build the RecordBatch schema from the sensor spec's declared columns.
/// `prism_core::column::ColumnType` canonical variants per ADR-024:
///   String / Integer / Float / Boolean / Datetime / Json
fn column_type_to_arrow(col_type: &ColumnType) -> DataType {
    match col_type {
        ColumnType::String => DataType::Utf8,
        ColumnType::Integer => DataType::Int64,
        ColumnType::Float => DataType::Float64,
        ColumnType::Boolean => DataType::Boolean,
        // Datetime → Utf8 for the spec-driven adapter layer.
        // Full timestamp typing is done at the DataFusion materialization layer (S-3.02).
        ColumnType::Datetime => DataType::Utf8,
        // Json → Utf8 (serialized JSON string in Arrow column).
        ColumnType::Json => DataType::Utf8,
        // Non-exhaustive guard: future variants default to Utf8.
        _ => DataType::Utf8,
    }
}

/// Construct an empty Arrow array (zero rows) for the given `ColumnType`.
fn empty_arrow_array(col_type: &ColumnType) -> Arc<dyn Array> {
    match col_type {
        ColumnType::Integer => Arc::new(Int64Array::from(Vec::<Option<i64>>::new())),
        ColumnType::Float => Arc::new(Float64Array::from(Vec::<Option<f64>>::new())),
        ColumnType::Boolean => Arc::new(BooleanArray::from(Vec::<Option<bool>>::new())),
        // String / Datetime / Json / future variants → Utf8
        _ => Arc::new(StringArray::from(Vec::<Option<&str>>::new())),
    }
}

/// Build an Arrow array for a single named column across all records.
///
/// Extracts the value at `col_name` from each raw JSON record.
/// Records where the field is absent or null produce a null entry in the array.
fn build_column_array(
    records: &[serde_json::Value],
    col_name: &str,
    col_type: &ColumnType,
) -> Arc<dyn Array> {
    match col_type {
        ColumnType::Integer => {
            let vals: Vec<Option<i64>> = records
                .iter()
                .map(|r| r.get(col_name).and_then(|v| v.as_i64()))
                .collect();
            Arc::new(Int64Array::from(vals))
        }
        ColumnType::Float => {
            let vals: Vec<Option<f64>> = records
                .iter()
                .map(|r| r.get(col_name).and_then(|v| v.as_f64()))
                .collect();
            Arc::new(Float64Array::from(vals))
        }
        ColumnType::Boolean => {
            let vals: Vec<Option<bool>> = records
                .iter()
                .map(|r| r.get(col_name).and_then(|v| v.as_bool()))
                .collect();
            Arc::new(BooleanArray::from(vals))
        }
        // String / Datetime / Json / future variants → Utf8
        // Json values are serialized as their compact string representation.
        _ => {
            let vals: Vec<Option<String>> = records
                .iter()
                .map(|r| {
                    r.get(col_name).and_then(|v| {
                        if v.is_null() {
                            None
                        } else if let serde_json::Value::String(s) = v {
                            Some(s.clone())
                        } else {
                            Some(v.to_string())
                        }
                    })
                })
                .collect();
            Arc::new(StringArray::from(
                vals.iter().map(|s| s.as_deref()).collect::<Vec<_>>(),
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// build_http_client_with_timeout — production reqwest::Client factory
// ---------------------------------------------------------------------------

/// Construct a `reqwest::Client` with a 30-second timeout.
///
/// MUST be used by `step9a_populate_adapter_registry` when constructing `SpecDrivenSensorAdapter`
/// instances. Using `reqwest::Client::new()` without a timeout is a P2 finding per
/// CLAUDE.md conventions (TD-S-PLUGIN-PREREQ-B-005).
///
/// Returns `Err(String)` if the client builder fails (should not happen in production;
/// failure mode is malformed TLS configuration).
pub fn build_http_client_with_timeout() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("failed to build reqwest::Client with 30s timeout: {e}"))
}

// ---------------------------------------------------------------------------
// step9a_populate_adapter_registry — boot step 9A
// ---------------------------------------------------------------------------

/// Boot step 9A: iterate `resolved_spec_map` and register one `SpecDrivenSensorAdapter`
/// per `(OrgId, SensorId)` pair.
///
/// Positioned between step 7.5b and step 9 in the ADR-022 §B sequencing table.
/// Emits `event_type = "boot.step9a.adapter_registry_populated"` with `sensor_count`
/// and `org_count` fields per BC-2.16.002 catalog row (SAP-1 obligation).
///
/// # Auth strategy selection
///
/// For each `(org_slug, sensor_id, resolved_spec)` triple:
/// - `CustomViaPlugin`: looks up `plugin_auth_providers.get(&sensor_id)` → `AdapterAuthStrategy::Plugin`.
///   If not found (auth_plugin declared but provider not constructed at step 7.5b), logs E-SPEC-012
///   and skips the sensor.
/// - `BearerStatic`: → `AdapterAuthStrategy::BearerStatic` (no held provider).
/// - `CookieRoundtrip`: constructs `StaticCookieAuthProvider::new(sensor_id)` →
///   `AdapterAuthStrategy::StaticCookie(Arc::new(...))`.
/// - Other `auth_type` values: logs E-SPEC-012 (auth type mismatch for S-DEMO-001 scope) and skips.
///
/// # OrgSlug → OrgId translation
///
/// `resolved_spec_map` is keyed by `(OrgSlug, SensorId)`. `AdapterRegistry` is keyed by
/// `(OrgId, SensorId)`. This function calls `org_registry.id_for_slug(slug)` to translate.
/// If a slug has no matching OrgId (should not happen after step 3 cross-validation), the
/// sensor is skipped with a warning log.
///
/// # AC-006: empty spec_catalog
///
/// If `resolved_spec_map` is empty, the function returns `Ok(0)` — no error, no adapters.
/// Boot continues to step 9 (MCP server start).
///
/// # Errors
///
/// Returns `Err(BootError::InternalError)` if the HTTP client cannot be constructed
/// (reqwest builder failure — should never occur in production).
///
/// BC-2.22.001 postcondition; BC-2.01.013; BC-2.06.014; AC-004; AC-006.
pub async fn step9a_populate_adapter_registry(
    resolved_spec_map: &HashMap<ResolvedSpecKey, ResolvedSensorSpec>,
    org_registry: &prism_core::OrgRegistry,
    plugin_auth_providers: &HashMap<String, Arc<PluginAuthProvider>>,
    adapter_registry: &mut prism_sensors::AdapterRegistry,
) -> Result<usize, crate::boot::BootError> {
    // AC-006: empty spec_catalog → 0 registrations, no error.
    if resolved_spec_map.is_empty() {
        tracing::info!(
            event_type = "boot.step9a.adapter_registry_populated",
            sensor_count = 0u64,
            org_count = 0u64,
            "boot step 9A: spec catalog is empty — 0 adapters registered",
        );
        return Ok(0);
    }

    // Build HTTP client with 30s timeout (TD-S-PLUGIN-PREREQ-B-005).
    // Shared across all adapters constructed in this boot step.
    let http_client = build_http_client_with_timeout().map_err(|e| {
        crate::boot::BootError::InternalError(format!(
            "boot step 9A: failed to build HTTP client: {e}"
        ))
    })?;

    let mut registered_count: usize = 0;
    // Track unique orgs that had at least one adapter registered (for org_count metric).
    let mut orgs_with_adapters: std::collections::HashSet<OrgId> = std::collections::HashSet::new();

    for ((org_slug, _sensor_id_key), resolved_spec) in resolved_spec_map {
        // OQ-2 Resolution: translate OrgSlug → OrgId via OrgRegistry::resolve().
        // Note: story spec uses id_for_slug(), but the existing method is resolve().
        // These are functionally equivalent; use resolve() per KNOWN IN-SCOPE WORK note.
        let org_id = match org_registry.resolve(org_slug) {
            Some(id) => id,
            None => {
                // Slug has no matching OrgId — skip with warning (OQ-2 Resolution §skip behavior).
                // No event_type= field: this is an internal diagnostic, not an auditable event.
                // SAP-1: event_type= requires a BC-2.16.002 catalog row.
                tracing::warn!(
                    org_slug = %org_slug.as_str(),
                    sensor_id = %resolved_spec.spec.sensor_id,
                    "boot step 9A: OrgSlug has no matching OrgId in OrgRegistry — \
                     adapter NOT registered; boot continues. Ensure step 3 cross-validation \
                     ran before step 9A. OQ-2 Resolution.",
                );
                continue;
            }
        };

        // Select auth strategy based on sensor spec's auth_type.
        let auth_strategy = match &resolved_spec.spec.auth_type {
            AuthType::CustomViaPlugin => {
                // CrowdStrike: look up the PluginAuthProvider constructed at step 7.5b.
                // If not found (auth_plugin declared but provider not constructed), skip with warning (EC-004).
                let sensor_id_str = resolved_spec.spec.sensor_id.as_str();
                match plugin_auth_providers.get(sensor_id_str) {
                    Some(provider) => {
                        AdapterAuthStrategy::Plugin(Arc::clone(provider) as Arc<dyn AuthProvider>)
                    }
                    None => {
                        // No event_type= field: internal diagnostic, not an auditable event.
                        // SAP-1: event_type= requires a BC-2.16.002 catalog row.
                        tracing::warn!(
                            sensor_id = %sensor_id_str,
                            org_slug = %org_slug.as_str(),
                            "boot step 9A: CustomViaPlugin sensor has no PluginAuthProvider \
                             (not constructed at step 7.5b). Adapter NOT registered. \
                             EC-004: step 7.5b failure skips step 9A for this sensor.",
                        );
                        continue;
                    }
                }
            }
            AuthType::BearerStatic => {
                // Armis/Claroty: token extracted from SensorAuth arg at fetch() call time.
                AdapterAuthStrategy::BearerStatic
            }
            AuthType::CookieRoundtrip => {
                // Cyberint: StaticCookieAuthProvider reads API key from credential store
                // at acquire_token() time with NO HTTP call (ADR-031 §D1-b).
                let provider = prism_spec_engine::StaticCookieAuthProvider::new(
                    resolved_spec.spec.sensor_id.as_str(),
                );
                AdapterAuthStrategy::StaticCookie(Arc::new(provider) as Arc<dyn AuthProvider>)
            }
            other => {
                // EC-007: unsupported auth_type — log E-SPEC-012 and skip.
                // No event_type= field: internal diagnostic, not an auditable event.
                // SAP-1: event_type= requires a BC-2.16.002 catalog row.
                tracing::warn!(
                    sensor_id = %resolved_spec.spec.sensor_id,
                    org_slug = %org_slug.as_str(),
                    auth_type = ?other,
                    "boot step 9A: E-SPEC-012 — unsupported auth_type for S-DEMO-001 scope. \
                     Adapter NOT registered; boot continues. \
                     Supported types: CustomViaPlugin, BearerStatic, CookieRoundtrip. \
                     EC-007: S-DEMO-001 scope boundary.",
                );
                continue;
            }
        };

        // Construct the SpecDrivenSensorAdapter.
        let adapter = SpecDrivenSensorAdapter::new(
            Arc::new(resolved_spec.clone()),
            auth_strategy,
            http_client.clone(),
        );

        // Register in AdapterRegistry keyed by (OrgId, SensorId) — SensorId from adapter.sensor_type().
        adapter_registry.register(org_id, Arc::new(adapter));
        orgs_with_adapters.insert(org_id);
        registered_count += 1;
    }

    // SAP-1 obligation: emit structured event with sensor_count and org_count fields.
    // BC-2.16.002 catalog row: boot.step9a.adapter_registry_populated.
    let org_count = orgs_with_adapters.len();
    tracing::info!(
        event_type = "boot.step9a.adapter_registry_populated",
        sensor_count = registered_count as u64,
        org_count = org_count as u64,
        "boot step 9A complete: adapter registry populated with spec-driven adapters",
    );

    Ok(registered_count)
}

// boot_step9_build_adapter_registry DELETED (F-002-R):
// This duplicate helper was removed per adversary pass-2 finding F-002-R.
// The production wiring is tested by test_BC_2_22_001_production_boot_path_wiring_guard
// which reads boot.rs source directly to verify step9_start_mcp_server calls
// step9a_populate_adapter_registry. That structural guard is stronger than testing
// a parallel helper function that bypasses the real production wiring.
// BC-2.22.001; F-002-R; S-DEMO-001 v1.5.

// ---------------------------------------------------------------------------
// Unit tests placeholder
// ---------------------------------------------------------------------------
// Tests are added by the test-writer (next pipeline step). This module
// contains only the stubs. No test code here.
