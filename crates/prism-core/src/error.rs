//! PrismError — canonical error taxonomy for the entire Prism platform.
//!
//! Every variant's Display output MUST begin with its structured error code token,
//! e.g. `"E-STORE-001: ..."`. Callers rely on the prefix for structured logging
//! and metric tagging.
//!
//! `PluginError` carries E-PLUGIN-* error codes from the WASM plugin runtime (S-1.15).

use thiserror::Error;

/// Private Display helper for the `suggested_column: Option<String>` field of
/// `PrismError::QueryTypeMismatch` (error-taxonomy v2.19 §E-QUERY-002 AC-022).
///
/// Renders:
/// - `Some(col)` → `"; for label comparison, use the string column '{col}' with IEQ/IIN/INE instead"`
/// - `None` → `""` (empty — no suffix appended to the Display message)
///
/// Used by the `#[error]` positional arg in `PrismError::QueryTypeMismatch` so the
/// Display output is byte-for-byte identical to the previous `String`-field approach
/// while the struct contract now holds a bare `Option<String>` column name.
///
/// Not public — callers construct `QueryTypeMismatch` with a bare `Option<String>`.
struct SuggestedSuffix<'a>(&'a Option<String>);

impl std::fmt::Display for SuggestedSuffix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(col) = self.0 {
            write!(
                f,
                "; for label comparison, use the string column '{col}' with IEQ/IIN/INE instead"
            )
        } else {
            Ok(())
        }
    }
}

/// Inner fields for `PrismError::EnrichUdfNotFound` (E-QUERY-039).
///
/// Boxed inside the enum variant to keep `PrismError` under the
/// `clippy::result_large_err` 128-byte threshold — two `String` fields
/// plus `Vec<String>` plus `Option<String>` inline would exceed the limit.
///
/// # Construction
/// ```
/// use prism_core::error::{PrismError, EnrichUdfNotFoundDetails};
/// let err = PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails::new(
///     "threat_intel",
///     vec!["threat_score".to_string(), "threat_is_known_malicious".to_string()],
///     Some("threat_score".to_string()),
/// )));
/// ```
///
/// Reference: S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B; BC-2.11.019; error-taxonomy.md E-QUERY-039.
///
/// # `#[non_exhaustive]` note
/// Marked `#[non_exhaustive]` per CLAUDE.md convention for public `prism-core` structs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct EnrichUdfNotFoundDetails {
    /// The enrichment UDF name that was requested but not found (e.g. `"threat_intel"`).
    /// This is typically an infusion_id used as if it were a callable per-field UDF name.
    pub infusion: String,
    /// All registered per-field UDF names available in this deployment.
    /// Empty only when `infusion_registry` has no descriptors loaded.
    pub available_infusions: Vec<String>,
    /// Levenshtein-based suggestion — `Some("threat_score")` when distance ≤ 3, `None` otherwise.
    pub did_you_mean: Option<String>,
}

impl EnrichUdfNotFoundDetails {
    /// Construct an `EnrichUdfNotFoundDetails`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct literal construction
    /// from outside `prism-core`. (CLAUDE.md `#[non_exhaustive]` discipline)
    pub fn new(
        infusion: impl Into<String>,
        available_infusions: Vec<String>,
        did_you_mean: Option<String>,
    ) -> Self {
        Self {
            infusion: infusion.into(),
            available_infusions,
            did_you_mean,
        }
    }
}

impl std::fmt::Display for EnrichUdfNotFoundDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Canonical taxonomy template (error-taxonomy.md §E-QUERY-039 Message Format;
        // BC-2.11.019, PO-reconciled spec S-DEMO-FIDELITY-REMEDIATION-001):
        // "E-QUERY-039: enrichment infusion '{infusion}' is not registered;
        //  available: [{available_infusions}]{did_you_mean}"
        // {available_infusions}: available_infusions joined with ", " wrapped in [ ] brackets,
        //   SORTED lexicographically per BC-2.11.019 §PrismError-variant.
        //   Sort happens here in Display so the contract is self-enforcing; the caller
        //   (check_enrich_udf_availability) also sorts+deduplicates before constructing
        //   the error, which is defensive — both are harmless.
        // Empty Vec → [].
        // {did_you_mean}: " Did you mean: '{x}'?" (leading space) when Some, omitted when None.
        //
        // F-PBL1-LOW-002 fix (Pass-B S-DEMO-FIDELITY-REMEDIATION-001): sort within
        // Display so the contract is self-enforcing. Previously the sort only lived
        // in check_enrich_udf_availability; any direct construction with unsorted
        // available_infusions would produce non-deterministic output.
        let mut sorted = self.available_infusions.clone();
        sorted.sort();
        let available = sorted.join(", ");
        let did_you_mean_suffix = match &self.did_you_mean {
            Some(s) => format!(" Did you mean: '{s}'?"),
            None => String::new(),
        };
        write!(
            f,
            "E-QUERY-039: enrichment infusion '{}' is not registered; available: [{}]{}",
            self.infusion, available, did_you_mean_suffix
        )
    }
}

/// Inner fields for `PrismError::ColumnNotFound` (E-QUERY-038).
///
/// Boxed inside the enum variant to keep `PrismError` under the
/// `clippy::result_large_err` 128-byte threshold — four `String` fields
/// plus `Vec<String>` plus `Option<String>` inline would exceed the limit.
///
/// # Construction
/// ```
/// use prism_core::error::{PrismError, ColumnNotFoundDetails};
/// let err = PrismError::ColumnNotFound(Box::new(ColumnNotFoundDetails::new(
///     "sevrity",
///     "crowdstrike_alerts",
///     "acme",
///     vec!["severity".to_string(), "host_name".to_string()],
///     Some("severity".to_string()),
/// )));
/// ```
///
/// Reference: S-DEMO-PRISMQL-ONBOARDING-001-B; BC-2.11.016; error-taxonomy.md E-QUERY-038.
///
/// # `#[non_exhaustive]` note
/// Marked `#[non_exhaustive]` per CLAUDE.md convention for public `prism-core` structs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ColumnNotFoundDetails {
    /// The column name that was queried (e.g. `"sevrity"`).
    pub column: String,
    /// The table the column was looked up in (e.g. `"crowdstrike_alerts"`).
    pub table: String,
    /// The client ID in whose scope the query was executed.
    pub client_id: String,
    /// All column names available in the table for this client (org-scoped per DI-008).
    /// Empty when `resolved_spec_map` is `None` (single-tenant / test mode).
    pub available_columns: Vec<String>,
    /// Levenshtein-based suggestion — `Some("severity")` when distance ≤ 3, `None` otherwise.
    pub did_you_mean: Option<String>,
}

impl ColumnNotFoundDetails {
    /// Construct a `ColumnNotFoundDetails`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct literal construction
    /// from outside `prism-core`. (CLAUDE.md `#[non_exhaustive]` discipline)
    pub fn new(
        column: impl Into<String>,
        table: impl Into<String>,
        client_id: impl Into<String>,
        available_columns: Vec<String>,
        did_you_mean: Option<String>,
    ) -> Self {
        Self {
            column: column.into(),
            table: table.into(),
            client_id: client_id.into(),
            available_columns,
            did_you_mean,
        }
    }
}

impl std::fmt::Display for ColumnNotFoundDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Taxonomy template (error-taxonomy.md §E-QUERY-038 Message Format, POL-24 byte-verbatim):
        // "E-QUERY-038: column '{column}' not found in table '{table}' for client '{client_id}';
        //  available: [{available_columns}]{did_you_mean}"
        // {available_columns}: comma-separated list (may be empty).
        // {did_you_mean}: " Did you mean: '{best_match}'?" when Some; empty string when None.
        let available = self.available_columns.join(", ");
        let did_you_mean_suffix = match &self.did_you_mean {
            Some(s) => format!(" Did you mean: '{s}'?"),
            None => String::new(),
        };
        write!(
            f,
            "E-QUERY-038: column '{}' not found in table '{}' for client '{}'; available: [{}]{}",
            self.column, self.table, self.client_id, available, did_you_mean_suffix
        )
    }
}

/// Inner fields for `PrismError::TableNotAvailable` (E-QUERY-037).
///
/// Boxed inside the enum variant to keep `PrismError` under the
/// `clippy::result_large_err` 128-byte threshold. Five inline `String` fields
/// would push the variant past the limit; boxing trades a pointer-dereference
/// on the error path (which is not performance-critical) for enum-size compliance.
///
/// Implements `Display` directly so `PrismError::TableNotAvailable` can use
/// `#[error("{0}")]` to delegate formatting without `thiserror`'s field-access syntax
/// (which does not support indexing into `Box<T>` fields).
///
/// # Construction
/// ```
/// use prism_core::error::{PrismError, TableNotAvailableDetails};
/// let err = PrismError::TableNotAvailable(Box::new(TableNotAvailableDetails::new(
///     "crowdstrike_alerts",
///     "crowdstrike",
///     "armis",
///     "armis_alerts",
///     "",
///     "Call prism_describe('acme') to see available tables and columns.",
/// )));
/// ```
///
/// Reference: S-3.13; BC-2.11.001; error-taxonomy.md E-QUERY-037.
///
/// # `#[non_exhaustive]` note
/// Marked `#[non_exhaustive]` per CLAUDE.md convention for public `prism-core` structs.
/// Callers outside this crate must use `TableNotAvailableDetails::new(...)` rather than
/// struct literal construction (E0639 would fire at the cross-crate construction site).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TableNotAvailableDetails {
    /// The table name that was queried (e.g. `"crowdstrike_alerts"`).
    pub table: String,
    /// The sensor that owns the table (e.g. `"crowdstrike"`), derived from the table name prefix.
    pub sensor: String,
    /// Pre-formatted comma-separated list of available sensor IDs (e.g. `"armis, claroty"`).
    pub available_sensors: String,
    /// Pre-formatted comma-separated list of all registered table names.
    pub available_tables: String,
    /// Either `""` (no match within Levenshtein ≤ 3) or `" Did you mean: 'X'?"`.
    pub did_you_mean: String,
    /// Pedagogical suggestion produced by `e_query_037_suggestion()`.
    ///
    /// Contains a "Call prism_describe(…)" hint so users learn how to discover
    /// available tables. Set at the call site in `check_availability_gate` via
    /// `prism_query::engine::e_query_037_suggestion`.
    pub suggestion: String,
}

impl TableNotAvailableDetails {
    /// Construct a `TableNotAvailableDetails`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct literal construction
    /// from outside `prism-core`. (CLAUDE.md `#[non_exhaustive]` discipline)
    pub fn new(
        table: impl Into<String>,
        sensor: impl Into<String>,
        available_sensors: impl Into<String>,
        available_tables: impl Into<String>,
        did_you_mean: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self {
            table: table.into(),
            sensor: sensor.into(),
            available_sensors: available_sensors.into(),
            available_tables: available_tables.into(),
            did_you_mean: did_you_mean.into(),
            suggestion: suggestion.into(),
        }
    }
}

impl std::fmt::Display for TableNotAvailableDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "E-QUERY-037: table '{}' is not available — sensor '{}' is not configured. \
             Available sensors: [{}]. \
             Available tables: [{}].{} {}",
            self.table,
            self.sensor,
            self.available_sensors,
            self.available_tables,
            self.did_you_mean,
            self.suggestion
        )
    }
}

/// Inner fields for `PrismError::UnknownSourceTable` (E-QUERY-036).
///
/// Boxed inside the enum variant to keep `PrismError` under the
/// `clippy::result_large_err` 128-byte threshold. Two `String` fields plus
/// `Vec<String>` plus `Option<String>` inline would push the variant past the limit.
///
/// Implements `Display` directly so `PrismError::UnknownSourceTable` can use
/// `#[error("{0}")]` to delegate formatting without `thiserror`'s field-access syntax.
///
/// # Construction
/// ```
/// use prism_core::error::{PrismError, UnknownSourceTableDetails};
/// let err = PrismError::UnknownSourceTable(Box::new(UnknownSourceTableDetails::new(
///     "ghost_sensor.devices",
///     vec!["crowdstrike".to_string()],
///     Some("crowdstrike".to_string()),
/// )));
/// ```
///
/// Reference: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 AC-021; error-taxonomy.md E-QUERY-036.
///
/// # `#[non_exhaustive]` note
/// Marked `#[non_exhaustive]` per CLAUDE.md convention for public `prism-core` structs.
/// Callers outside this crate must use `UnknownSourceTableDetails::new(...)` rather than
/// struct literal construction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnknownSourceTableDetails {
    /// The source name that was queried (e.g. `"ghost_sensor.devices"`).
    pub source_name: String,
    /// Sorted list of registered sensor IDs available to query (e.g. `["crowdstrike"]`).
    pub available_tables: Vec<String>,
    /// Levenshtein-based suggestion — `Some("crowdstrike")` when distance ≤ 3, `None` otherwise.
    pub did_you_mean: Option<String>,
}

impl UnknownSourceTableDetails {
    /// Construct an `UnknownSourceTableDetails`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct literal construction
    /// from outside `prism-core`. (CLAUDE.md `#[non_exhaustive]` discipline)
    pub fn new(
        source_name: impl Into<String>,
        available_tables: Vec<String>,
        did_you_mean: Option<String>,
    ) -> Self {
        Self {
            source_name: source_name.into(),
            available_tables,
            did_you_mean,
        }
    }
}

impl std::fmt::Display for UnknownSourceTableDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let available = self.available_tables.join(", ");
        let did_you_mean_suffix = match &self.did_you_mean {
            Some(s) => format!(" Did you mean: '{s}'?"),
            None => String::new(),
        };
        write!(
            f,
            "E-QUERY-036: unknown source table '{}': table is not a registered sensor \
             or internal table. Check spelling or register the sensor in prism.toml. \
             Available tables: [{}].{}",
            self.source_name, available, did_you_mean_suffix
        )
    }
}

// ---------------------------------------------------------------------------
// E-QUERY-042 support type: TemporalLiteralPosition
// ---------------------------------------------------------------------------

/// The clause/position where an invalid `RawTemporalLiteral` was found.
///
/// Used by `PrismError::TemporalLiteralInvalidPosition` (E-QUERY-042) to select the
/// correct analyst-facing error message for each structural position.
///
/// `#[non_exhaustive]`: new positions may be added in future ADR-052 revisions.
/// External match arms MUST include a wildcard `_ => {}` arm per CLAUDE.md §Conventions.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalLiteralPosition {
    /// Temporal literal found as a GROUP BY key.
    ///
    /// e.g. `GROUP BY '2026-06-24'` — grouping by a constant is a degenerate no-op.
    GroupBy,
    /// Temporal literal found as an ORDER BY key.
    ///
    /// e.g. `ORDER BY '2026-06-24'` — ordering by a constant is a degenerate no-op.
    OrderBy,
    /// Temporal literal found in a comparison where the LHS is a function call or
    /// compound expression (not a bare `Field` node).
    ///
    /// e.g. `WHERE lower(hostname) = '2026-06-24'` — LHS type cannot be resolved at
    /// plan time; silent coercion would reintroduce RISK-1 for datetime-valued exprs.
    NonColumnLhsComparison,
}

impl TemporalLiteralPosition {
    /// Format the E-QUERY-042 error message for this position variant.
    ///
    /// `value_prefix` is the first ≤50 UTF-8 codepoints of the offending literal
    /// (used in GroupBy and OrderBy messages; not interpolated in NonColumnLhsComparison).
    ///
    /// Called by the `PrismError::TemporalLiteralInvalidPosition` thiserror Display impl.
    /// POL-24: messages MUST match error-taxonomy.md §E-QUERY-042 v2.14 byte-for-byte.
    pub fn as_display_string(&self, value_prefix: &str) -> String {
        match self {
            Self::GroupBy => format!(
                "E-QUERY-042: GROUP BY expects a column reference, not a literal constant. \
                 '{value_prefix}' is a date-shaped literal \u{2014} grouping by a constant has \
                 no effect and is almost certainly a query mistake. Did you mean to reference a \
                 column name, or to add a WHERE filter before grouping?"
            ),
            Self::OrderBy => format!(
                "E-QUERY-042: ORDER BY expects a column reference, not a literal constant. \
                 '{value_prefix}' is a date-shaped literal \u{2014} ordering by a constant has \
                 no effect. Did you mean to reference a column name that contains this value?"
            ),
            Self::NonColumnLhsComparison => {
                "E-QUERY-042: A date-like literal compared against a computed expression cannot be \
                 type-checked at plan time. Compare against a bare datetime column using RFC-3339 \
                 (e.g., '2026-07-03T00:00:00Z'), against a string column using a non-date-shaped \
                 value, or wrap the expression in an explicit CAST."
                    .to_string()
            }
        }
    }
}

/// Canonical error type for the Prism platform.
///
/// Covers all 90+ error codes across every subsystem category. Group variants
/// by category prefix; each category maps to a subsystem.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrismError {
    // -------------------------------------------------------------------------
    // E-AUTH — Authentication / tenant identity
    // -------------------------------------------------------------------------
    /// E-AUTH-001: Org slug failed validation.
    #[error("E-AUTH-001: invalid tenant ID: {reason}")]
    InvalidOrgSlug { reason: String },

    /// E-AUTH-002: Analyst identifier failed validation.
    #[error("E-AUTH-002: invalid analyst ID: {reason}")]
    InvalidAnalystId { reason: String },

    /// E-AUTH-003: Client identifier failed validation.
    #[error("E-AUTH-003: invalid client ID: {reason}")]
    InvalidClientId { reason: String },

    /// E-AUTH-010: Auth token expired.
    #[error("E-AUTH-010: auth token expired")]
    AuthTokenExpired,

    /// E-AUTH-011: Auth token invalid.
    #[error("E-AUTH-011: auth token invalid: {reason}")]
    AuthTokenInvalid { reason: String },

    /// E-AUTH-020: Unauthorized — caller lacks required permission.
    #[error("E-AUTH-020: unauthorized: {action}")]
    Unauthorized { action: String },

    // -------------------------------------------------------------------------
    // E-SENSOR — Sensor adapter errors
    // -------------------------------------------------------------------------
    /// E-SENSOR-001: Sensor adapter returned an unexpected HTTP status.
    #[error("E-SENSOR-001: sensor {sensor} returned HTTP {status}: {body}")]
    SensorHttpError {
        sensor: String,
        status: u16,
        body: String,
    },

    /// E-SENSOR-002: Sensor adapter timed out.
    #[error("E-SENSOR-002: sensor {sensor} timed out after {elapsed_ms}ms")]
    SensorTimeout { sensor: String, elapsed_ms: u64 },

    /// E-SENSOR-003: Sensor adapter returned malformed response.
    #[error("E-SENSOR-003: sensor {sensor} response parse error: {detail}")]
    SensorResponseParse { sensor: String, detail: String },

    /// E-SENSOR-020: Sensor rate limited.
    #[error("E-SENSOR-020: sensor {sensor} rate limited; retry after {retry_after_ms}ms")]
    SensorRateLimited { sensor: String, retry_after_ms: u64 },

    // -------------------------------------------------------------------------
    // E-OCSF — OCSF normalization errors
    // -------------------------------------------------------------------------
    /// E-OCSF-001: Required OCSF field missing from source event.
    #[error("E-OCSF-001: required OCSF field missing: {field}")]
    OcsfFieldMissing { field: String },

    /// E-OCSF-002: OCSF field type mismatch.
    #[error("E-OCSF-002: OCSF field type mismatch on {field}: expected {expected}, got {got}")]
    OcsfFieldTypeMismatch {
        field: String,
        expected: String,
        got: String,
    },

    /// E-OCSF-003: Unknown OCSF class UID.
    #[error("E-OCSF-003: unknown OCSF class_uid: {class_uid}")]
    OcsfUnknownClassUid { class_uid: u32 },

    /// E-OCSF-010: OCSF protobuf encode failure.
    #[error("E-OCSF-010: protobuf encode error: {detail}")]
    OcsfProtobufEncode { detail: String },

    /// E-OCSF-011: OCSF protobuf decode failure.
    #[error("E-OCSF-011: protobuf decode error: {detail}")]
    OcsfProtobufDecode { detail: String },

    /// E-OCSF-020: No OCSF event class mapping for the given sensor + record_type pair.
    ///
    /// Emitted by `EventClassSelector::select()` when the sensor/record_type combination
    /// is not found in the compile-time mapping table. (BC-2.02.012, AC-8)
    #[error(
        "E-OCSF-020: no OCSF event class mapping for sensor={sensor}, record_type={record_type}"
    )]
    OcsfUnknownEventClass { sensor: String, record_type: String },

    /// E-OCSF-021: OCSF normalization failed — `normalize()` could not produce a valid
    /// `DynamicMessage` from the provided raw input.
    ///
    /// This is the catch-all error for BC-2.02.002 / VP-022: normalize() must return
    /// this error rather than panicking on malformed input.
    #[error("E-OCSF-021: OCSF normalization failed for source {source_id}: {reason}")]
    OcsfNormalizationFailed { source_id: String, reason: String },

    /// E-OCSF-022: The OCSF protobuf descriptor pool does not contain a descriptor for
    /// the requested `class_uid`.
    ///
    /// Returned by `OcsfNormalizer::normalize()` when `EventClassSelector::select()`
    /// resolves to a class_uid that is absent from the compiled DescriptorPool.
    /// (BC-2.02.001, AC-2)
    #[error("E-OCSF-022: OCSF descriptor not found for class_uid={class_uid}")]
    OcsfDescriptorNotFound { class_uid: u32 },

    /// E-OCSF-023: Sensor record_type not in the mapper's supported set.
    ///
    /// Returned by `SpecDrivenMapper` when the record_type is not one of the table
    /// names declared in the sensor's spec. (BC-2.02.005, BC-2.02.006, S-1.05 Edge Cases)
    #[error("E-OCSF-023: unknown record type for sensor={sensor}: record_type={record_type}")]
    OcsfUnknownRecordType { sensor: String, record_type: String },

    /// E-OCSF-024: Timestamp field could not be parsed using any supported format.
    ///
    /// Returned by `SpecDrivenMapper` when a datetime column value fails all parse
    /// attempts. (BC-2.02.004, AC-4, S-1.05 Edge Cases)
    #[error("E-OCSF-024: timestamp parse failed for field={field}: raw value={raw}")]
    OcsfTimestampParseError { field: String, raw: String },

    // -------------------------------------------------------------------------
    // E-CRED — Credential management errors
    // -------------------------------------------------------------------------
    /// E-CRED-001: Credential name failed validation (S-1.02 + S-1.06).
    #[error("E-CRED-001: invalid credential name '{name}': {reason}")]
    InvalidCredentialName { name: String, reason: String },

    /// E-CRED-002: Credential not found.
    #[error("E-CRED-002: credential not found: {name}")]
    CredentialNotFound { name: String },

    /// E-CRED-003: Credential access denied (AI-opaque boundary enforced).
    #[error(
        "E-CRED-003: credential access denied for {name} — credential values never transit AI context"
    )]
    CredentialAccessDenied { name: String },

    /// E-CRED-004: Backend-level credential store failure (S-1.06).
    #[error("E-CRED-004: credential store error (backend={backend}): {reason}")]
    CredentialStoreError { backend: String, reason: String },

    /// E-CRED-006: Credential encryption failure on encrypted-file backend (S-1.06).
    #[error("E-CRED-006: credential encryption error: {reason}")]
    CredentialEncryptionError { reason: String },

    /// E-CRED-007: Encryption passphrase not configured (S-1.06).
    #[error("E-CRED-007: encryption key not configured: {reason}")]
    EncryptionKeyMissing { reason: String },

    // -------------------------------------------------------------------------
    // E-IO — I/O errors
    // -------------------------------------------------------------------------
    /// E-IO-001: I/O error (S-1.06). String-ified so PrismError remains PartialEq+Eq.
    #[error("E-IO-001: I/O error: {0}")]
    Io(String),

    // -------------------------------------------------------------------------
    // E-FLAG — Feature flag / capability errors (BC-2.04.015,
    // E-FLAG-001 runtime tier / E-FLAG-002 compile tier)
    // -------------------------------------------------------------------------
    /// CAPABILITY_DENIED: Write capability is denied — structured error for
    /// BC-2.04.015. Carries BOTH denial tiers (P2-03, 2026-06-10 review pass-2):
    ///
    /// - **E-FLAG-001 (runtime tier)** — produced from
    ///   `CapabilityCheckResult::DeniedRuntime`: the capability is not enabled
    ///   in the client's runtime TOML configuration.
    /// - **E-FLAG-002 (compile tier)** — produced from
    ///   `CapabilityCheckResult::DeniedCompileTime`: no `[[write_endpoints]]`
    ///   declaration for the capability in the sensor's TOML spec
    ///   (registry-derived per BC-2.04.001 / BC-2.16.012).
    ///
    /// The `resolution_trace` is a BTreeMap-derived ordered list of path→effect
    /// pairs showing how the denial was reached.
    #[error(
        "CAPABILITY_DENIED: capability '{capability}' denied for client '{client_id}': {reason}"
    )]
    CapabilityDenied {
        /// The capability path that was checked (e.g., `sensor.crowdstrike.containment`).
        capability: String,
        /// The client whose effective capabilities were consulted.
        client_id: String,
        /// Human-readable denial reason.
        reason: String,
        /// Actionable guidance (exact TOML path + restart instruction or rebuild note).
        suggestion: String,
        /// Ordered list of `"path=effect"` pairs showing the resolution walk.
        /// Minimum one entry (the winning tier).
        resolution_trace: Vec<String>,
    },

    /// E-FLAG-006: Cross-client write without client_id.
    #[error(
        "E-FLAG-006: write operation requires client_id — cross-client writes are not supported"
    )]
    WriteRequiresClientId,

    // P2-03(c) (2026-06-10 review pass-2): the `FeatureFlagDisabled` variant
    // (formerly claiming the E-FLAG-002 code here) was REMOVED — it had zero
    // spec backing (no hit in .factory/specs/, incl. BC-2.10.007) and zero
    // production emitters (constructed only in its own pinning tests).
    // E-FLAG-002 is the COMPILE-TIER capability denial carried by
    // `CapabilityDenied` (via `CapabilityCheckResult::DeniedCompileTime`) per
    // error-taxonomy E-FLAG-002 row / BC-2.04.015 / BC-2.04.001.
    /// E-FLAG-010: Feature flag evaluation error.
    #[error("E-FLAG-010: feature flag evaluation error for {flag}: {detail}")]
    FeatureFlagEvalError { flag: String, detail: String },

    // -------------------------------------------------------------------------
    // E-FLAG-003..008 — Confirmation token errors (S-1.09, BC-2.04.009..012)
    // -------------------------------------------------------------------------
    /// E-FLAG-003: Confirmation token expired (BC-2.04.011).
    ///
    /// Returned when `confirm_action` is called with a token whose `expires_at`
    /// is in the past (`now >= expires_at`). The `action_summary` from the
    /// original token is included so the agent can re-request intelligently.
    #[error(
        "E-FLAG-003: confirmation token expired for action '{action_summary}'; \
         call the original write tool to generate a new token"
    )]
    TokenExpired {
        /// The `action_summary` from the expired token.
        action_summary: String,
        /// `retryable: false` — agent must call the original write tool again.
        retryable: bool,
    },

    /// E-FLAG-004: Confirmation token already consumed (BC-2.04.010; VP-008).
    #[error(
        "E-FLAG-004: confirmation token '{token_id}' already consumed; \
         call the original write tool to generate a new token if needed"
    )]
    TokenAlreadyConsumed { token_id: String, retryable: bool },

    /// E-FLAG-005: Confirmation token content hash mismatch (BC-2.04.012; VP-009).
    ///
    /// The action parameters supplied to `confirm_action` do not match the
    /// SHA-256 hash stored in the token — tampering or substitution detected.
    #[error(
        "E-FLAG-005: confirmation token '{token_id}' content hash mismatch; \
         request a new token for the intended action"
    )]
    TokenContentHashMismatch { token_id: String, retryable: bool },

    /// E-FLAG-007: Confirmation token store capacity exceeded (BC-2.04.009; VP-010).
    ///
    /// The store holds 100 active tokens. After sweeping expired tokens the cap
    /// is still reached. No eviction occurs — the caller must wait.
    #[error(
        "E-FLAG-007: token store capacity reached (100 active tokens); \
         wait for existing tokens to expire or confirm/cancel pending actions"
    )]
    TokenCapExceeded,

    /// E-FLAG-008: Confirmation token not found in store (BC-2.04.010).
    #[error(
        "E-FLAG-008: confirmation token not found: '{token_id}'; \
         it may have expired and been cleaned up"
    )]
    TokenNotFound { token_id: String },

    /// E-MCP-004: client_id mismatch on confirm_action (BC-2.04.010).
    ///
    /// The `client_id` passed to `confirm_action` does not match the
    /// `client_id` embedded in the token at generation time.
    #[error(
        "E-MCP-004: client_id mismatch on confirm_action for token '{token_id}'; \
         use the same client_id that was used when the token was generated"
    )]
    ConfirmClientIdMismatch { token_id: String, retryable: bool },

    // -------------------------------------------------------------------------
    // E-STORE — Storage backend errors
    // -------------------------------------------------------------------------
    /// E-STORE-001: RocksDB open failed.
    #[error("E-STORE-001: RocksDB open failed: {detail}")]
    StorageOpenFailed { detail: String },

    /// E-STORE-002: RocksDB write failed.
    #[error("E-STORE-002: RocksDB write failed on domain {domain}: {detail}")]
    StorageWriteFailed { domain: String, detail: String },

    /// E-STORE-003: RocksDB read failed.
    #[error("E-STORE-003: RocksDB read failed on domain {domain}: {detail}")]
    StorageReadFailed { domain: String, detail: String },

    /// E-STORE-004: Storage domain not found / column family missing.
    #[error("E-STORE-004: storage domain not found: {domain}")]
    StorageDomainNotFound { domain: String },

    /// E-STORE-005: Storage key not found.
    #[error("E-STORE-005: key not found in domain {domain}")]
    StorageKeyNotFound { domain: String },

    /// E-STORE-006: RocksDB database LOCK file is held by another process.
    ///
    /// Returned when `RocksDbBackend::open()` finds the exclusive lock held
    /// (E-STORE-005 in BC-2.15.001 terminology; mapped to this variant).
    /// The `path` is the state directory passed to `open()`.
    #[error("E-STORE-006: Another Prism instance is using {path}", path = path.display())]
    StorageLockHeld { path: std::path::PathBuf },

    /// E-STORE-007: RocksDB startup health check failed.
    ///
    /// Returned when the write/read/delete cycle on the `default` CF fails
    /// after successful open. Indicates a non-corrupt but unhealthy database
    /// (e.g., permissions error, disk full, IO fault).
    #[error("E-STORE-007: storage health check failed: {detail}")]
    StorageHealthCheckFailed { detail: String },

    /// E-STORE-008: Schema version mismatch — stored schema version does not
    /// match the current Prism build's expected schema version.
    ///
    /// Returned by `RocksDbBackend::check_schema_version()`. The process
    /// MUST NOT proceed with a mismatched schema (BC-2.15.001, EC-003).
    #[error("E-STORE-008: schema version mismatch: stored={stored}, current={current}")]
    SchemaMismatch { stored: String, current: String },

    /// E-STORE-010: Storage batch write failed.
    #[error("E-STORE-010: storage batch write failed: {detail}")]
    StorageBatchFailed { detail: String },

    /// E-STORE-020: Cursor cap exceeded (S-1.02).
    /// Unit variant: CursorRegistry enforces the cap at the type boundary.
    #[error("E-STORE-020: cursor cap exceeded: cannot allocate more than 200 active cursors")]
    CursorCapExceeded,

    // -------------------------------------------------------------------------
    // E-CFG — Configuration errors
    // -------------------------------------------------------------------------
    /// E-CFG-100: Referenced `client_id` is not configured (tool parameter or
    /// alias scope names an unknown client).
    ///
    /// New variant per ADR-038 D3 (client-not-found variant split): carries the
    /// typed `client_id` for the most caller-visible config error in the MCP
    /// surface. Contract anchors: BC-2.10.004, BC-2.11.001/008/011/013/014,
    /// BC-2.08.008, BC-2.14.010 (all pin E-CFG-100 for this condition).
    /// Maps to JSON-RPC `-32602 INVALID_PARAMS` per ADR-038 D4 — a wrong
    /// `client_id` is caller-resolvable, not an internal failure.
    #[error("E-CFG-100: client '{client_id}' not found in configuration")]
    ClientNotFound { client_id: String },

    /// E-CFG-103: Config file not found (renumbered per ADR-038 D2).
    #[error("E-CFG-103: config file not found: {path}")]
    ConfigNotFound { path: String },

    /// E-CFG-104: Config parse error (renumbered per ADR-038 D2;
    /// AD-007 hot-reload surface — zero emitters today, forward-declared).
    #[error("E-CFG-104: config parse error: {detail}")]
    ConfigParseFailed { detail: String },

    /// E-CFG-102: Config validation error (renumbered per ADR-038 D2;
    /// `{detail}` SHOULD carry the toml_path, expected, and actual values).
    #[error("E-CFG-102: config validation failed: {detail}")]
    ConfigValidationFailed { detail: String },

    /// E-CFG-105: Config snapshot stale (renumbered per ADR-038 D2;
    /// transient/retryable — retry acquires a fresh `ArcSwap::load()` snapshot, AD-007).
    #[error("E-CFG-105: config snapshot stale: version {current} < required {required}")]
    ConfigSnapshotStale { current: u64, required: u64 },

    /// E-CFG-106: Capability path validation failed (renumbered per ADR-038 D2).
    ///
    /// Returned by `CapabilityPath::new()` when the input string violates any
    /// of the format rules: empty string, empty segment, invalid characters,
    /// more than 8 segments, or total length > 256 characters.
    #[error("E-CFG-106: invalid capability path: {reason}")]
    InvalidCapabilityPath {
        /// Human-readable description of the validation failure.
        reason: String,
    },

    // -------------------------------------------------------------------------
    // E-MCP — MCP protocol errors
    // -------------------------------------------------------------------------
    /// E-MCP-001: MCP tool not found.
    #[error("E-MCP-001: MCP tool not found: {tool}")]
    McpToolNotFound { tool: String },

    /// E-MCP-002: MCP parameter validation failed.
    #[error("E-MCP-002: MCP parameter validation failed for tool {tool}: {detail}")]
    McpParameterInvalid { tool: String, detail: String },

    /// E-MCP-003: MCP response serialization error.
    #[error("E-MCP-003: MCP response serialization error: {detail}")]
    McpSerializationError { detail: String },

    /// E-MCP-010: Prompt injection detected (safety boundary).
    #[error("E-MCP-010: prompt injection detected in tool {tool}")]
    McpPromptInjectionDetected { tool: String },

    // -------------------------------------------------------------------------
    // E-SAFETY — Safety boundary violations
    // -------------------------------------------------------------------------
    /// E-SAFETY-001: AI context contamination attempt blocked.
    #[error("E-SAFETY-001: AI context contamination attempt blocked: {detail}")]
    SafetyContextContamination { detail: String },

    /// E-SAFETY-002: Sensitive data exfiltration blocked.
    #[error("E-SAFETY-002: sensitive data exfiltration blocked: {field}")]
    SafetyDataExfiltration { field: String },

    // -------------------------------------------------------------------------
    // E-QUERY — Query engine errors
    // -------------------------------------------------------------------------
    /// E-QUERY-001: Query parse error.
    ///
    /// The `query` field carries the original query string so that enrichment helpers
    /// (e.g. `extract_near_text`) can extract a near-text snippet for the MCP error
    /// envelope (BC-2.11.017 AC-003). DI-006: the snippet is truncated to ≤50 chars
    /// before surfacing to callers.
    #[error("E-QUERY-001: query parse error at offset {offset}: {detail}")]
    QueryParseFailed {
        offset: usize,
        detail: String,
        /// Original query string (pre-expansion). Used for near_text extraction only;
        /// never surfaced verbatim to callers — only a ≤50-char token snippet is exposed.
        ///
        /// # Security boundary (SEC-007 / CWE-209)
        /// This field stores the FULL model query as submitted by the LLM agent. It MUST
        /// NEVER be included verbatim in any MCP response, log message, or user-facing
        /// output. The only permitted consumer is `prism-mcp::error_mapping`'s
        /// `QueryParseFailed` arm, which derives the `near_text` snippet (≤50 chars) via
        /// `prism_query::engine::extract_near_text` — that truncation is the correct,
        /// approved exposure surface (BC-2.11.017 AC-003 / DI-006).
        ///
        /// Visibility is `pub` (not `pub(crate)`) because `prism-mcp` (a separate crate)
        /// must access this field to compute `near_text`. If `prism-mcp` ever moves to
        /// a dedicated error-extraction API (e.g. `PrismError::near_text_snippet()`),
        /// this field should be narrowed to `pub(crate)` at that time.
        query: String,
    },

    /// E-QUERY-002: Query planning failed.
    #[error("E-QUERY-002: query planning failed: {detail}")]
    QueryPlanFailed { detail: String },

    /// E-QUERY-002: Query type mismatch — a column was used with an operator that is
    /// not valid for its `ColumnType`.
    ///
    /// Produced by the plan-time type-compatibility gate in `prism-query` (BC-2.11.017).
    /// Carries the column name, table name, the column's actual `ColumnType`, and the
    /// operator string as used in the query (e.g., `">"`), so the MCP error-mapping layer
    /// can call `valid_operators_for_type(actual_type)` to populate the `valid_operators_for_type`
    /// field in the structured error response with the TYPE-SPECIFIC set.
    ///
    /// Inline (not boxed): `column` + `table` + `operator` (3 × 24 bytes) + `actual_type`
    /// (enum discriminant, ≤ 8 bytes) + alignment ≈ 80 bytes — under the 128-byte
    /// `result_large_err` threshold; no helper struct needed.
    ///
    /// Maps to JSON-RPC `-32602 INVALID_PARAMS` — the caller's query used an incompatible
    /// operator on a typed column; caller-resolvable by switching to a valid operator.
    ///
    /// Reference: S-DEMO-PRISMQL-ONBOARDING-001-B; BC-2.11.017; error-taxonomy.md E-QUERY-002.
    #[error(
        "E-QUERY-002: type mismatch — column '{column}' in table '{table}' has type \
         '{actual_type:?}' which does not support operator '{operator}'{}",
        SuggestedSuffix(suggested_column)
    )]
    QueryTypeMismatch {
        /// The column name used with the incompatible operator.
        column: String,
        /// The table the column belongs to (fully-qualified, e.g. `"crowdstrike_alerts"`).
        table: String,
        /// The column's declared `ColumnType` from the sensor spec.
        actual_type: crate::column::ColumnType,
        /// The operator string as it appears in the query (e.g., `">"`).
        operator: String,
        /// Bare column name of the OCSF string sibling, if known; `None` otherwise.
        ///
        /// `Some("severity")` for `severity_id`, `Some("status")` for `status_id`, etc.
        /// `None` for temporal checks, DML assignments, and other cases without a known
        /// OCSF string sibling.
        ///
        /// The Display impl appends the suggestion suffix (error-taxonomy v2.19 §E-QUERY-002
        /// AC-022; BC-2.11.024) via the private `SuggestedSuffix` Display helper.
        suggested_column: Option<String>,
    },

    /// E-QUERY-003: Query security limit exceeded (security-only variant).
    ///
    /// Per error-taxonomy.md §E-QUERY-003, this variant is reserved for security-limit
    /// violations (query size cap, AST depth cap, regex complexity caps, IN-list
    /// caps, etc.). The `detail` carries the specific limit violation message;
    /// the Display impl supplies the single canonical "E-QUERY-003: " prefix —
    /// callers MUST NOT embed the prefix in `detail`.
    ///
    /// Maps to JSON-RPC `-32602 INVALID_PARAMS` — the caller supplied a query
    /// that violates a pre-execution security limit and can fix it by narrowing
    /// the query. Distinct from E-QUERY-034 (`QueryExecutionFailed`), which is
    /// the generic runtime execution error and maps to `-32000`.
    #[error("E-QUERY-003: {detail}")]
    QuerySecurityLimitExceeded { detail: String },

    /// E-QUERY-034: Query execution error (generic runtime execution failure).
    ///
    /// Renumbered from E-QUERY-003 per error-taxonomy.md §E-QUERY-034 + ADR-038 §P5-02
    /// §P5-02: E-QUERY-003 is now security-only (`QuerySecurityLimitExceeded`);
    /// generic execution failures carry E-QUERY-034 and map to JSON-RPC `-32000`.
    #[error("E-QUERY-034: query execution error: {detail}")]
    QueryExecutionFailed { detail: String },

    /// E-WATCHDOG-001: Memory budget exceeded.
    ///
    /// Per error-taxonomy.md, query memory exhaustion is an E-WATCHDOG code
    /// ("Query memory limit exceeded"), NOT an E-QUERY code. The query's memory
    /// consumption exceeded the watchdog budget and the query was terminated.
    #[error("E-WATCHDOG-001: query memory budget exceeded: limit {limit_mb}MB, used {used_mb}MB")]
    QueryMemoryBudgetExceeded { limit_mb: u64, used_mb: u64 },

    /// E-QUERY-004: Query timeout (retryable with a narrower scope per
    /// error-taxonomy.md).
    #[error("E-QUERY-004: query timed out after {elapsed_ms}ms")]
    QueryTimeout { elapsed_ms: u64 },

    /// E-QUERY-005: Materialization limit exceeded — the streaming record
    /// counter exceeded the 10K cap during sensor fan-out fetch
    /// (BC-2.11.006 EC-003, error-taxonomy.md E-QUERY-005).
    #[error("E-QUERY-005: materialization limit exceeded: fetched {count} records (max {max})")]
    QueryMaterializationLimitExceeded {
        /// Number of records the fetch would have materialized.
        count: usize,
        /// Configured materialization cap (10,000 per BC-2.11.006).
        max: usize,
    },

    /// E-QUERY-010: Virtual field resolution failed.
    #[error("E-QUERY-010: virtual field resolution failed for {field}: {detail}")]
    QueryVirtualFieldFailed { field: String, detail: String },

    /// E-QUERY-020: Write targets a composite source (e.g. EVENTS) — not a single
    /// external sensor source. Composite sources are read-only (BC-2.04.005 §Task 3a).
    #[error(
        "E-QUERY-020: write target '{source_name}' is a composite source (e.g. EVENTS); \
         writes must target a single external sensor source"
    )]
    WriteTargetCompositeSource { source_name: String },

    /// E-QUERY-021: Write batch limit exceeded — too many records would be affected.
    ///
    /// Returned when either:
    /// - The structural LIMIT in the write plan exceeds the resolved batch limit
    ///   (Phase 2 structural check, BC-2.04.008 §Task 3d).
    /// - The post-fetch record count exceeds the resolved batch limit (Phase 3→4
    ///   boundary check, story §Task 10).
    #[error(
        "E-QUERY-021: batch limit exceeded: query would affect {requested} records; \
         limit for '{endpoint}' on client '{client_id}' is {limit}"
    )]
    WriteBatchLimitExceeded {
        requested: usize,
        limit: usize,
        endpoint: String,
        client_id: String,
    },

    /// E-QUERY-022: Unbounded write — no WHERE clause and no LIMIT on the source
    /// fetch (BC-2.04.008 §Task 3c, story §AC-8, EC-04-007).
    ///
    /// Returned before any fetch or sensor API contact.
    #[error(
        "E-QUERY-022: unbounded write rejected — query has no WHERE clause and no LIMIT; \
         add a filter or LIMIT to bound the write operation"
    )]
    WriteUnbounded,

    /// E-QUERY-026: Write to internal table is not permitted via PrismQL.
    ///
    /// Emitted when a write attempt targets an internal `prism_*` table (e.g., `prism_audit`,
    /// `prism_alerts`) reserved for prism-internal accounting. Internal tables are
    /// write-protected at the PrismQL surface; operators needing to mutate internal state
    /// must use the dedicated MCP tool for the specific operation.
    ///
    /// Also caught at parse time by S-3.06; this is the runtime defense-in-depth check
    /// (story §Task 3a, AC-4, EC-04-006).
    ///
    /// Reference: write-operations.md catalog (E-QUERY-026).
    /// Distinguished from:
    ///   - E-QUERY-027 (RESERVED): confirmation token required for irreversible write
    ///   - E-QUERY-029 (RESERVED): adapter declared in spec but not init for client
    ///   - E-QUERY-030: write target table not in WriteEndpointRegistry (different code path —
    ///     internal tables ARE in the registry but flagged as internal)
    #[error(
        "E-QUERY-026: Write to internal table '{table}' is not permitted via PrismQL. \
         Use the dedicated MCP tool for this operation."
    )]
    WriteTargetingInternalTable { table: String },

    /// E-QUERY-023: Write verb is not available for the named source.
    ///
    /// Emitted when a write attempt targets a sensor's spec table but the registered
    /// write endpoint catalog does not contain a verb for that (sensor, table) tuple.
    /// This is a structural / configuration error: typically means the sensor's spec
    /// declared no write capability for that table.
    ///
    /// Reference: write-operations.md:625-640 architecture catalog (E-QUERY-023).
    ///
    /// Note: field is named `sensor_source` (not `source`) to avoid conflict with
    /// thiserror's reserved `source` field name for error chaining.
    #[error("E-QUERY-023: Write verb '{verb}' is not available for source '{sensor_source}'")]
    WriteVerbNotAvailable { verb: String, sensor_source: String },

    /// E-QUERY-036: Query references an unregistered sensor table or an invalid table name prefix.
    ///
    /// Caller-resolvable: check spelling or register the sensor in prism.toml.
    /// The inner `UnknownSourceTableDetails` carries `available_tables` (registered sensor IDs)
    /// and `did_you_mean` (Levenshtein ≤ 3 suggestion) for actionable diagnostics.
    ///
    /// The fields are boxed (`Box<UnknownSourceTableDetails>`) to keep `PrismError`
    /// within the `clippy::result_large_err` 128-byte threshold.
    ///
    /// Construct via `PrismError::UnknownSourceTable(Box::new(UnknownSourceTableDetails::new(...)))`.
    /// Match via `PrismError::UnknownSourceTable(ref d)` or `PrismError::UnknownSourceTable(..)`.
    ///
    /// Reference: error-taxonomy.md §E-QUERY-036; BC-2.11.007 EC-001; P6-02 adjudication 2026-06-11;
    ///            S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 AC-021.
    #[error("{0}")]
    UnknownSourceTable(Box<UnknownSourceTableDetails>),

    /// E-QUERY-038: Column not found in table at plan time (BC-2.11.016).
    ///
    /// Returned by the plan-time column gate in `engine.rs` AFTER the E-QUERY-037
    /// table-existence gate passes.  The gate fires BEFORE fan-out (fail fast).
    ///
    /// Maps to MCP code -32602 (INVALID_PARAMS) — caller-resolvable by correcting
    /// the column name or calling `prism_describe` to enumerate available columns.
    ///
    /// The inner fields are boxed (`Box<ColumnNotFoundDetails>`) to keep `PrismError`
    /// within the `clippy::result_large_err` 128-byte threshold — four `String` fields
    /// plus `Vec<String>` plus `Option<String>` inline would exceed the limit.
    ///
    /// Reference: S-DEMO-PRISMQL-ONBOARDING-001-B; BC-2.11.016; error-taxonomy.md E-QUERY-038.
    #[error("{0}")]
    ColumnNotFound(Box<ColumnNotFoundDetails>),

    /// E-QUERY-039: Enrichment UDF not found at plan time (BC-2.11.019).
    ///
    /// Returned by the plan-time enrichment gate in `engine.rs` BEFORE fan-out when a
    /// query's enrichment stage names a UDF that is not registered in the `InfusionRegistry`.
    /// This fires for BOTH pipe mode (`| enrich udf_name(col)`) and SQL mode
    /// (`SELECT udf_name(col) FROM ...`).
    ///
    /// The most common root cause: the caller used the `infusion_id` (e.g. `threat_intel`)
    /// as the callable name instead of the per-field UDF name (e.g. `threat_score`).
    ///
    /// Maps to MCP code -32602 (INVALID_PARAMS) — caller-resolvable by using a
    /// per-field UDF name from `prism_describe` or the PQL reference resource.
    ///
    /// The inner fields are boxed (`Box<EnrichUdfNotFoundDetails>`) to keep `PrismError`
    /// within the `clippy::result_large_err` 128-byte threshold.
    ///
    /// Construct via `PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails::new(...)))`.
    /// Match via `PrismError::EnrichUdfNotFound(ref d)` or `PrismError::EnrichUdfNotFound(..)`.
    ///
    /// Reference: S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B; BC-2.11.019; error-taxonomy.md E-QUERY-039.
    #[error("{0}")]
    EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>),

    /// E-QUERY-037: Table is not available — the sensor that owns the table is not configured.
    ///
    /// Returned by the plan-time availability gate in `engine.rs` when a query references
    /// a table whose owning sensor is not in the live `TableRegistry`. This fires BEFORE
    /// fan-out (fail fast). Carries pre-formatted String fields; `did_you_mean` is `""`
    /// when no candidate within Levenshtein ≤ 3 exists, or `" Did you mean: 'X'?"` otherwise.
    ///
    /// Maps to MCP code -32602 (INVALID_PARAMS) — caller-resolvable by configuring the
    /// missing sensor or correcting the table name.
    ///
    /// The inner fields are boxed (`Box<TableNotAvailableDetails>`) to keep `PrismError`
    /// within the `clippy::result_large_err` threshold — five `String` fields inline would
    /// push the enum variant past 128 bytes (S-3.13 clippy gate).
    ///
    /// Construct via `PrismError::TableNotAvailable(Box::new(TableNotAvailableDetails { .. }))`.
    /// Match via `PrismError::TableNotAvailable(ref d)` or `PrismError::TableNotAvailable(..)`.
    ///
    /// Reference: S-3.13 AC-2, AC-3, AC-8; BC-2.11.001; error-taxonomy.md E-QUERY-037.
    #[error("{0}")]
    TableNotAvailable(Box<TableNotAvailableDetails>),

    /// E-QUERY-032: Sensor is not registered for the requesting org.
    ///
    /// Raised by `resolve_source_refs` at the query-planning boundary when an
    /// explicitly-scoped client list targets a sensor that is registered for OTHER
    /// orgs but NOT for the requesting org (BC-3.2.001 postcondition 5).
    ///
    /// This is a SURFACED operational error (NOT redacted to "Internal error").
    /// The error message is safe to surface to the MCP caller — it contains only
    /// the sensor name and org slug, never credential values (AD-017).
    ///
    /// Maps to MCP code -32602 (INVALID_PARAMS) — the caller supplied an org
    /// scoping parameter that refers to a sensor it is not entitled to query.
    ///
    /// Reference: error-taxonomy.md §E-QUERY-032;
    ///            ADR-007 §2.2 cross-org isolation;
    ///            BC-3.2.001 postcondition 5.
    #[error("E-QUERY-032: Sensor '{sensor_id}' is not registered for org '{org_slug}'")]
    SensorNotRegisteredForOrg {
        /// The sensor ID that was queried.
        sensor_id: String,
        /// The org slug that was explicitly requested but has no registration.
        org_slug: String,
    },

    // RESERVED error codes not yet implemented:
    //
    // E-QUERY-024 (non-terminal write): declared in architecture catalog
    //   (write-operations.md:625-640) but not yet implemented in code.
    //   Tracked: TD-S307-001 (file via state-manager in next burst).
    //   These error paths are not reachable via current S-3.07 surface; implementation
    //   deferred until S-3.06's pipe-mode-write surface is exercised end-to-end (later
    //   stories likely S-3.10 or S-3.11).
    //
    // E-QUERY-027 (confirmation token required for irreversible write): RESERVED for
    //   the write-confirmation flow path on irreversible writes. Will gain callers in
    //   W3-FIX-S307-001 OR a dedicated story for write-confirmation flow. Distinguished
    //   from E-QUERY-026 (`WriteTargetingInternalTable`) which rejects writes to
    //   prism_* tables regardless of confirmation state.

    // E-QUERY-028: RESERVED for write fan-out rate limit / 429 retry path.
    //   Per architecture catalog write-operations.md:639. Will be implemented when
    //   per-sensor HTTP write() dispatch lands (W3-FIX-S307-001). The variant body
    //   will likely be { sensor: String, retry_after: Duration } per the OCSF
    //   429 mapping convention.

    // E-QUERY-029 RESERVED for per-client adapter init failure path.
    //   No callers in S-3.07 — the from_dml_node site that previously emitted this
    //   variant (with `<unknown>` client_id fallback) was switched to E-QUERY-030
    //   per fix-pass-2-correction (D-285) once the architecturally-correct
    //   distinction was recognized: from_dml_node failure is "table unknown to
    //   registry" (no client involved yet), not "adapter not init for client".
    //   Will gain callers when W3-FIX-S307-002 lands the OrgRegistry lookup.
    /// E-QUERY-029: Write endpoint declared in spec but not found in AdapterRegistry —
    /// the sql_table name is not recognized by the WriteEndpointRegistry for this client.
    ///
    /// Returned when a SQL DML plan's target table IS known to the registry but the
    /// per-client adapter has not been initialized for this specific client. Distinguished
    /// from E-QUERY-030 (`WriteTargetTableUnknown`), which fires when the table itself is
    /// absent from the registry (no client involved yet).
    ///
    /// Reference: write-operations.md:625-640 architecture catalog (E-QUERY-029).
    /// RESERVED until W3-FIX-S307-002 lands the OrgRegistry lookup.
    #[error(
        "E-QUERY-029: Write endpoint declared in spec but not found in AdapterRegistry. \
         Sensor '{sensor}' (table '{table}') may not be configured for client '{client_id}'"
    )]
    WriteAdapterNotConfiguredForClient {
        sensor: String,
        table: String,
        client_id: String,
    },

    /// E-QUERY-030: Write target table not declared in the WriteEndpointRegistry.
    ///
    /// Emitted when a parsed DML query references a target table that does not
    /// appear in the loaded WriteEndpointRegistry. This is a structural /
    /// configuration error at the DML parse → registry lookup boundary, BEFORE
    /// any client identity resolution. Distinguished from:
    ///   - E-QUERY-023 (`WriteVerbNotAvailable`): table IS known, verb is not
    ///   - E-QUERY-026 (`WriteTargetingInternalTable`): table IS in registry as `prism_*`
    ///   - E-QUERY-027 (RESERVED): confirmation token required for irreversible write
    ///   - E-QUERY-029 (`WriteAdapterNotConfiguredForClient`): table IS in registry,
    ///     adapter is per-client and not initialized for this specific client
    ///
    /// Reference: write-operations.md catalog (E-QUERY-030).
    #[error(
        "E-QUERY-030: Write target table '{table}' is not declared in the WriteEndpointRegistry. \
         Either the table name is misspelled, or no write endpoint is configured for it in the \
         loaded sensor specs."
    )]
    WriteTargetTableUnknown { table: String },

    /// E-QUERY-025: Write partial failure — some records succeeded and some failed.
    ///
    /// Returned by WriteCapableTableProvider when failed_count > 0 && succeeded_count > 0.
    /// Carries the full WriteResult for partial-success diagnostics.
    ///
    /// Story: S-3.07 | MED-7
    #[error(
        "E-QUERY-025: partial write failure for sensor '{sensor}' endpoint '{endpoint}': \
         {failed} of {total} records failed"
    )]
    WritePartialFailure {
        sensor: String,
        endpoint: String,
        failed: u32,
        total: u32,
    },

    /// E-QUERY-033: Requested limit exceeds the maximum allowed value (BC-2.11.001).
    ///
    /// Returned when `QueryOptions.limit > 1000`. Semantically distinct from
    /// `QueryExecutionFailed` (E-QUERY-034) — this is a pre-execution parameter
    /// validation error, not a runtime execution error. Moved off E-QUERY-001 to
    /// avoid collision with QueryParseFailed (ADV-W3MT-P58-CRIT-001); assigned
    /// E-QUERY-033 per taxonomy v1.70 P2-01 adjudication (ADR-038 D5 tombstone
    /// permanence — the interim code was a Phase-1 tombstone; full history in
    /// the error-taxonomy.md E-QUERY-033 row).
    #[error("E-QUERY-033: limit {requested} exceeds maximum of {max} (BC-2.11.001)")]
    QueryLimitExceeded {
        /// The limit value supplied by the caller.
        requested: usize,
        /// The configured maximum (1000 per BC-2.11.001).
        max: usize,
    },

    /// E-QUERY-011: Query targets `prism_audit` but caller lacks the `audit.read`
    /// capability (BC-2.15.011, AC-9).
    ///
    /// Display message intentionally contains "audit.read capability" so callers
    /// can detect this specific denial by substring match.
    #[error(
        "E-QUERY-011: Audit table requires audit.read capability. \
         Grant via prism.toml [clients.{{id}}.capabilities]."
    )]
    AuditTableAccessDenied,

    /// E-QUERY-012: Pagination cursor expired — caller must re-execute the query.
    ///
    /// Returned by `QueryCursorRegistry::next_page()` when the cursor's TTL
    /// (60 seconds) has elapsed since creation (BC-2.07.002 §Cursor TTL Expiry).
    ///
    /// Distinct from E-QUERY-004 (query timeout) and E-WATCHDOG-001 (query memory
    /// budget exceeded) — this error specifically signals that a previously
    /// valid cursor has aged out of the registry.
    #[error(
        "E-QUERY-012: pagination cursor expired (>60s); re-execute the query to obtain a fresh cursor"
    )]
    CursorExpired,

    /// E-QUERY-013: Pagination page_size must be greater than 0.
    ///
    /// Returned by `QueryCursorRegistry::create()` when `page_size == 0`,
    /// which would cause an infinite pagination loop (BC-2.07.001 preconditions).
    #[error("E-QUERY-013: page_size must be greater than 0")]
    CursorPageSizeInvalid,

    /// E-QUERY-014: Pagination cursor token not found in registry.
    ///
    /// Returned by `QueryCursorRegistry::next_page()` when the token was never
    /// registered (distinct from `CursorExpired` which is a valid token that
    /// has since timed out). (BC-2.07.002 §Error Cases)
    #[error(
        "E-QUERY-014: pagination cursor token not found; the token was never issued or is from a previous process instance"
    )]
    CursorTokenUnknown,

    // -------------------------------------------------------------------------
    // E-SCHED — Scheduler errors
    // -------------------------------------------------------------------------
    /// E-SCHED-001: Schedule not found.
    #[error("E-SCHED-001: schedule not found: {id}")]
    ScheduleNotFound { id: String },

    /// E-SCHED-002: Schedule conflict — overlapping execution window.
    #[error("E-SCHED-002: schedule conflict for {id}: overlapping window with {conflicting_id}")]
    ScheduleConflict { id: String, conflicting_id: String },

    /// E-SCHED-010: Cron expression parse error.
    #[error("E-SCHED-010: invalid cron expression '{expr}': {detail}")]
    ScheduleCronInvalid { expr: String, detail: String },

    // -------------------------------------------------------------------------
    // E-DET — Detection rule errors
    // -------------------------------------------------------------------------
    /// E-DET-001: Detection rule parse error.
    #[error("E-DET-001: detection rule parse error in {rule_id}: {detail}")]
    DetectionRuleParseFailed { rule_id: String, detail: String },

    /// E-DET-002: Detection rule not found.
    #[error("E-DET-002: detection rule not found: {rule_id}")]
    DetectionRuleNotFound { rule_id: String },

    /// E-DET-010: Detection state corruption.
    #[error("E-DET-010: detection state corrupt for rule {rule_id}: {detail}")]
    DetectionStateCorrupt { rule_id: String, detail: String },

    // -------------------------------------------------------------------------
    // E-CASE — Case management errors
    // -------------------------------------------------------------------------
    /// E-CASE-001: Case not found.
    #[error("E-CASE-001: case not found: {case_id}")]
    CaseNotFound { case_id: String },

    /// E-CASE-002: Case state transition invalid.
    #[error("E-CASE-002: invalid case state transition for {case_id}: {from} -> {to}")]
    CaseStateTransitionInvalid {
        case_id: String,
        from: String,
        to: String,
    },

    // -------------------------------------------------------------------------
    // E-WATCH — Watchdog errors
    // -------------------------------------------------------------------------
    /// E-WATCH-001: Watchdog heartbeat missed.
    #[error("E-WATCH-001: watchdog heartbeat missed for {component} after {elapsed_ms}ms")]
    WatchdogHeartbeatMissed { component: String, elapsed_ms: u64 },

    /// E-WATCH-002: Watchdog restart limit exceeded.
    #[error("E-WATCH-002: watchdog restart limit exceeded for {component}: {count} restarts")]
    WatchdogRestartLimitExceeded { component: String, count: u32 },

    /// E-WATCHDOG-002 (query kill): Watchdog killed the running query because process RSS
    /// exceeded the Kill threshold (95% of 512 MB budget) on two consecutive checks
    /// (BC-2.15.007, VP-058). Distinct from E-WATCHDOG-001 (per-query DataFusion
    /// memory-pool trip, `QueryMemoryBudgetExceeded`) — error-taxonomy.md §E-WATCHDOG-002,
    /// P1-04 adjudication.
    #[error(
        "E-WATCHDOG-002: watchdog killed query — process RSS exceeded kill threshold \
         ({budget_bytes} bytes budget); query token cancelled"
    )]
    WatchdogKilled {
        /// Configured memory budget in bytes — default 512 MB (SI, 512_000_000 bytes)
        /// per BC-2.15.006 / ADR-S2.02-002 ("512 MB" is SI decimal, NOT MiB).
        budget_bytes: usize,
    },

    /// E-QUERY-008 (query denylist): Query is denylisted after N consecutive watchdog
    /// terminations (BC-2.15.008, E-QUERY-008).
    #[error(
        "E-QUERY-008: query denylisted after {failure_count} consecutive failures \
         (reason: {reason}); denylist expires at {expiry_ts}; \
         use force_execute: true to override"
    )]
    QueryDenylisted {
        /// Number of consecutive watchdog-triggered failures.
        failure_count: u32,
        /// Reason for the last termination (timeout / memory / record_limit).
        reason: String,
        /// Unix timestamp (seconds) at which the denylist entry expires.
        expiry_ts: u64,
    },

    // -------------------------------------------------------------------------
    // E-SPEC — Spec engine errors
    // -------------------------------------------------------------------------
    /// E-SPEC structured error (BC-2.16.001, BC-2.16.002, BC-2.16.009).
    /// Carries an E-SPEC-* code, human-readable message, and optional TOML path.
    #[error("E-SPEC: {0}")]
    Spec(#[from] SpecError),

    /// E-SPEC-001: Sensor spec file not found.
    #[error("E-SPEC-001: sensor spec not found: {path}")]
    SpecNotFound { path: String },

    /// E-SPEC-002: Sensor spec validation failed.
    #[error("E-SPEC-002: sensor spec validation failed for {path}: {detail}")]
    SpecValidationFailed { path: String, detail: String },

    /// E-SPEC-010: Spec engine hot-reload failed.
    #[error("E-SPEC-010: spec hot-reload failed: {detail}")]
    SpecHotReloadFailed { detail: String },

    // -------------------------------------------------------------------------
    // E-INFUSE — Infusion enrichment errors (S-1.14)
    // -------------------------------------------------------------------------
    /// Infusion enrichment error (BC-2.19.001 through BC-2.19.005).
    #[error("infusion error: {0}")]
    Infusion(#[from] InfusionError),

    // -------------------------------------------------------------------------
    // E-PLUGIN — WASM Plugin Runtime errors (S-1.15)
    // -------------------------------------------------------------------------
    /// E-PLUGIN-* structured error (BC-2.17.001 through BC-2.17.006).
    /// Carries a structured PluginError variant — all calls that return Plugin errors
    /// are isolated at the `instance.call_*` boundary; the host process continues.
    #[error("E-PLUGIN: {0}")]
    Plugin(#[from] PluginError),

    // -------------------------------------------------------------------------
    // E-IOC — IOC / threat intel errors
    // -------------------------------------------------------------------------
    /// E-IOC-001: IOC feed parse error.
    #[error("E-IOC-001: IOC feed parse error from {feed}: {detail}")]
    IocFeedParseFailed { feed: String, detail: String },

    /// E-IOC-002: IOC lookup failed.
    #[error("E-IOC-002: IOC lookup failed for {indicator}: {detail}")]
    IocLookupFailed { indicator: String, detail: String },

    // -------------------------------------------------------------------------
    // E-AUDIT — Audit layer errors (S-2.04, BC-2.05.001)
    // -------------------------------------------------------------------------
    /// E-AUDIT-001: Audit entry persistence failed for a write operation.
    ///
    /// Returned by `AuditEmitter` when `emit()` fails for a write tool invocation.
    /// The write operation MUST be aborted — no unaudited writes are permitted
    /// (BC-2.05.001 fail-closed contract).
    ///
    /// Structured error fields:
    ///   - `category: "transient"`, `retryable: true`
    ///   - `suggestion: "Retry the operation. If the error persists, check tracing subscriber health."`
    #[error(
        "E-AUDIT-001: Audit emission failed; write operation blocked. \
         Retry the operation. If the error persists, check tracing subscriber health."
    )]
    AuditPersistenceFailed,

    // -------------------------------------------------------------------------
    // E-ALIAS — Query alias system errors (S-3.04, CAP-016, BC-2.11.008..015)
    // -------------------------------------------------------------------------
    /// E-ALIAS-001: Alias does not exist at the specified scope.
    ///
    /// Returned by `AliasResolver::expand()` when a `@name` token is found in a
    /// query but no alias named `name` exists in the current scope or globally.
    /// Also returned by `delete_alias` and `explain_alias` when the target alias
    /// is absent (BC-2.11.014, BC-2.11.015).
    #[error(
        "E-ALIAS-001: alias '{name}' not found in scope '{scope}'; \
         available aliases: {available}"
    )]
    AliasNotFound {
        /// The alias name that was referenced.
        name: String,
        /// Scope that was searched (e.g., "global" or "client:acme").
        scope: String,
        /// Comma-separated list of aliases available in the current scope.
        available: String,
    },

    /// E-ALIAS-002: Alias creation would introduce a cycle.
    ///
    /// Cycle detection runs at creation time (DI-020 invariant). The `cycle_chain`
    /// contains the ordered list of alias names that form the cycle, e.g.
    /// `["A", "B", "A"]` for the mutual cycle A → B → A.
    #[error("E-ALIAS-002: alias '{name}' would create a cycle: {cycle_chain}")]
    AliasCycleDetected {
        /// The alias being created.
        name: String,
        /// Human-readable cycle chain, e.g. "A -> B -> A".
        cycle_chain: String,
    },

    /// E-ALIAS-003: Alias composition depth exceeds the hard limit of 3.
    ///
    /// Returned when alias expansion would require traversing more than 3 nested
    /// alias definitions (VP-012). The `chain` lists the alias names traversed
    /// so far at the point of rejection.
    #[error("E-ALIAS-003: alias composition depth exceeded (max 3); chain: {chain}")]
    AliasDepthExceeded {
        /// Alias expansion chain at the point of depth-limit rejection.
        chain: String,
    },

    /// E-ALIAS-004: Parameter value fails type validation.
    ///
    /// Returned when a caller-supplied parameter value or a stored default value
    /// is not a PrismQL atomic literal (StringLiteral, IntegerLiteral,
    /// FloatLiteral, BooleanLiteral, DurationLiteral, or Identifier).
    /// Compound expressions are rejected to prevent query injection (BC-2.11.009).
    #[error(
        "E-ALIAS-004: parameter '{param}' for alias '{alias}' has an invalid value '{value}': \
         {reason}"
    )]
    AliasParameterInvalid {
        /// The parameter name.
        param: String,
        /// The alias the parameter belongs to.
        alias: String,
        /// The rejected value.
        value: String,
        /// Reason for rejection (e.g. "compound expression rejected; use a single literal token").
        reason: String,
    },

    /// E-ALIAS-005: Alias has dependent aliases and `force` is not `true`.
    ///
    /// Returned by `delete_alias` when the target alias is referenced by other
    /// aliases. Deletion is blocked; the caller must either delete dependents
    /// individually or pass `force: true` for cascade deletion (BC-2.11.014).
    #[error(
        "E-ALIAS-005: alias '{name}' has {count} dependent alias(es) and cannot be deleted \
         without force: true; dependents: {dependents}"
    )]
    AliasDependentsExist {
        /// The alias targeted for deletion.
        name: String,
        /// Number of dependents.
        count: usize,
        /// Comma-separated list of dependent alias names.
        dependents: String,
    },

    /// E-ALIAS-006: Alias name conflicts with a PrismQL keyword or OCSF field name.
    ///
    /// Alias names must not shadow PrismQL reserved words (`SELECT`, `WHERE`, etc.)
    /// or known OCSF field names loaded at startup (BC-2.11.008 invariants).
    #[error(
        "E-ALIAS-006: alias name '{name}' conflicts with a reserved {conflict_kind}: '{conflict}'"
    )]
    AliasNameConflict {
        /// The proposed alias name.
        name: String,
        /// Whether the conflict is a "PrismQL keyword" or "OCSF field name".
        conflict_kind: String,
        /// The specific keyword or field name that conflicts.
        conflict: String,
    },

    // -------------------------------------------------------------------------
    // E-QUERY-040 — SQL→Pipe redundant row-limit (ADR-043)
    // -------------------------------------------------------------------------
    /// E-QUERY-040: Both the SQL SELECT head and a pipe `| limit` or `| tail` stage carry a
    /// row-limit.  Only one may be specified; the other must be removed.
    ///
    /// Raised at planning time by the FORBID-BOTH invariant (ADR-043 §C).
    ///
    /// Message template per error-taxonomy.md §E-QUERY-040 Message Format (POL-24).
    #[error(
        "E-QUERY-040: redundant row limit. This query caps rows in two places: a SQL `LIMIT {sql_limit}` in the head and a row-capping `| limit`/`| tail` pipe stage (cap: {pipe_limit}). PrismQL requires exactly one row cap. Remove the SQL `LIMIT {sql_limit}` and place a single `| limit` at the end of the pipeline (recommended for composed queries), or use `LIMIT` only in pure SQL-mode queries."
    )]
    RedundantRowLimit {
        /// The `LIMIT n` value in the SQL SELECT head.
        sql_limit: u64,
        /// The `| limit m` or `| tail m` value from the row-capping pipe stage.
        pipe_limit: u64,
    },

    // -------------------------------------------------------------------------
    // E-QUERY-041 — Temporal literal pre-validator (ADR-052 D4)
    // -------------------------------------------------------------------------
    /// E-QUERY-041: String literal in datetime comparison failed RFC-3339 pre-validation.
    ///
    /// Returned by `check_temporal_literals` (Prism plan-time AST walker, ADR-052 §D4
    /// Option A) when a `Literal::RawTemporalLiteral` node is found in a comparison against a
    /// `ColumnType::Datetime` column. Date-only (`'2026-06-24'`) and offset-less ISO
    /// (`'2026-06-24T12:00:00'`) forms are rejected; full RFC-3339 with UTC offset is required.
    ///
    /// Maps to MCP code -32602 (INVALID_PARAMS) — caller-resolvable by using RFC-3339 form
    /// or switching to `NOW() - INTERVAL 'Nh'` for relative time filters.
    ///
    /// Reference: ADR-052 §D4; error-taxonomy §E-QUERY-041; BC-2.11.021 / BC-2.11.003 / BC-2.11.004 §Error Cases;
    ///            S-PRISMQL-NATIVE-TEMPORAL-TYPING-001.
    #[error(
        "E-QUERY-041: The value '{value_prefix}' cannot be interpreted as a UTC timestamp. \
         Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). Date-only \
         and offset-less forms are not accepted. For relative time filters, use \
         NOW() - INTERVAL 'Nh' (e.g., WHERE timestamp > NOW() - INTERVAL '24h')."
    )]
    TemporalLiteralUnparseable {
        /// First ≤50 UTF-8 codepoints of the offending literal string.
        ///
        /// Truncated at UTF-8 codepoint boundary per error-taxonomy.md §E-QUERY-041
        /// `value_prefix` field convention (AD-017 / E-INFUSE-014 truncation convention).
        value_prefix: String,
    },

    // -------------------------------------------------------------------------
    // E-QUERY-042 — Temporal literal invalid position (ADR-052 §D4 v1.10)
    // -------------------------------------------------------------------------
    /// E-QUERY-042: Date-like literal in a structurally invalid position or with
    /// an unresolvable LHS expression type.
    ///
    /// Three position variants (GroupBy, OrderBy, NonColumnLhsComparison) with
    /// distinct analyst-facing messages (POL-24 byte-for-byte match required).
    ///
    /// **GroupBy**: `GROUP BY '2026-06-24'` — grouping by a bare literal constant is a
    /// degenerate no-op (every row maps to the same group), almost always an analyst mistake.
    ///
    /// **OrderBy**: `ORDER BY '2026-06-24'` — ordering by a bare literal constant is a
    /// degenerate no-op, almost always an analyst mistake.
    ///
    /// **NonColumnLhsComparison**: `WHERE lower(hostname) = '2026-06-24'` — the LHS is a
    /// function or compound expression; the walker cannot resolve the LHS type at plan time.
    /// Silently coercing to `Literal::String` would reintroduce RISK-1 for datetime-valued
    /// expressions. Prior behavior: `QueryPlanFailed → -32000 INTERNAL_ERROR` (analyst-hostile).
    ///
    /// Maps to MCP code -32602 (INVALID_PARAMS) for all three variants — caller-resolvable.
    ///
    /// Reference: ADR-052 §D4 v1.10; error-taxonomy.md §E-QUERY-042 v2.14;
    ///            BC-2.11.021; BC-2.11.003; BC-2.11.004;
    ///            S-PRISMQL-NATIVE-TEMPORAL-TYPING-001.
    #[error("{}", position.as_display_string(value_prefix))]
    TemporalLiteralInvalidPosition {
        /// The clause/position where the literal was found.
        position: TemporalLiteralPosition,
        /// First ≤50 UTF-8 codepoints of the offending literal string.
        ///
        /// Truncated at UTF-8 codepoint boundary per error-taxonomy.md §E-QUERY-042
        /// `value_prefix` field convention (AD-017 / E-INFUSE-014 truncation convention).
        /// Used in GroupBy and OrderBy messages; not interpolated in NonColumnLhsComparison.
        value_prefix: String,
    },

    // -------------------------------------------------------------------------
    // Catch-all for unexpected internal errors
    // -------------------------------------------------------------------------
    /// E-INT-001: Internal invariant violated — indicates a bug.
    #[error("E-INT-001: internal error: {detail}")]
    Internal { detail: String },
}

// ---------------------------------------------------------------------------
// E-SPEC — Spec engine structured error types (S-1.11)
// ---------------------------------------------------------------------------

/// E-SPEC-* error codes from BC-2.16.001, BC-2.16.002, BC-2.16.009.
///
/// `#[non_exhaustive]`: variants will be added as new spec validation rules are introduced.
/// External match arms MUST include a wildcard `_ => {}` arm per CLAUDE.md §Conventions.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecErrorCode {
    /// E-SPEC-001: TOML parse error or schema/variable-reference validation error.
    ESpec001,
    /// E-SPEC-004: Duplicate table_name within a sensor spec.
    ESpec004,
    /// E-SPEC-008: Custom adapter panic caught via catch_unwind.
    ESpec008,
    /// E-SPEC-009: Duplicate sensor_id across spec files.
    ESpec009,
    /// E-SPEC-010: Variable interpolation failure at runtime.
    ESpec010,
    /// E-SPEC-011: Write endpoint pipe_verb collides with reserved PrismQL keyword (BC-2.16.009, S-1.13).
    ESpec011,
    /// E-SPEC-017: Spec `sensor_id` does not case-sensitively match the filename stem.
    /// E.g., `crowdstrike.sensor.toml` with `sensor_id: "falcon"` → rejected at load time.
    /// Emitted by `SpecLoader::load_all()` only (has filename context); never by `SpecLoader::parse()`.
    ESpec017,
    /// E-SPEC-018: `PipelineExecutor` failed to parse a `ColumnType::Datetime` column value
    /// against any of the declared `timestamp_formats`. The formats list and raw value are
    /// included in the `SpecError::message` for actionable correction.
    /// Emitted during response-to-Arrow materialization when `ColumnSpec::timestamp_formats`
    /// is non-empty and no format successfully parsed the field value.
    /// BC-2.16.013 §O-001 (Option A grammar extension); ADR-028 v1.10 §D8-C;
    /// error-taxonomy.md §E-SPEC-018.
    ESpec018,
    /// E-SPEC-019: Per-org overlay `extends` field references a sensor TYPE spec that does
    /// not exist in the loaded TYPE spec set. Boot hard error (exit 2).
    /// BC-2.06.016 §Error Catalog E-SPEC-019; ADR-029 §Decision.
    /// Example message: "Per-org overlay 'customers/acme/foo.sensor.toml' extends 'foo'
    /// which is not a loaded sensor TYPE spec."
    ESpec019,
    /// E-SPEC-020: Per-org overlay `instance_id` field does not match the canonical
    /// `{sensor_id}@{org_slug}` pattern derived from the filename stem and parent directory.
    /// Boot hard error (exit 2). BC-2.06.016 §Error Catalog E-SPEC-020; ADR-029 §Decision.
    /// Example message: "Per-org overlay 'customers/acme/armis.sensor.toml' instance_id
    /// 'wrong@value' does not match expected 'armis@acme'."
    ESpec020,
    /// E-SPEC-021: Per-org overlay file contains `[[tables]]` blocks, which are forbidden.
    /// Schema overrides are not permitted in overlay files — the TYPE spec defines the
    /// canonical schema for all tenants (INV-OVL-001). Boot hard error (exit 2).
    /// BC-2.06.016 §Error Catalog E-SPEC-021; ADR-029 §Decision Drivers (TOML array-replace).
    /// Example message: "Per-org overlay 'customers/acme/armis.sensor.toml' for instance
    /// 'armis@acme' contains [[tables]] blocks. Schema overrides are forbidden in overlay
    /// files (ADR-029)."
    ESpec021,
    /// E-SPEC-022: The `customers/<slug>/` directory name references an org slug that is
    /// not registered in `OrgRegistry`. Boot hard error (exit 2).
    /// BC-2.06.016 §Error Catalog E-SPEC-022; BC-2.06.015 postcondition failure path.
    /// Example message: "Per-org overlay directory 'customers/unknown-org/' references org
    /// slug 'unknown-org' which is not registered in OrgRegistry. Check for typos or
    /// register the org in prism.toml [[orgs]]."
    ESpec022,
    /// E-SPEC-023: Per-org overlay file contains an unrecognized scalar field. Only the
    /// allowed overlay fields are permitted: `extends`, `instance_id`, `base_url`,
    /// `timeout_secs`, `rate_limit_hints` (and its sub-fields). Boot hard error (exit 2).
    /// BC-2.06.016 §Error Catalog E-SPEC-023; BC-2.06.013 (scalar-only overlay enforcement).
    /// Example message: "Per-org overlay 'customers/acme/armis.sensor.toml' contains
    /// unrecognized field 'auth_type'. Only scalar tunables are permitted in overlay files
    /// (ADR-029)."
    ESpec023,
    /// E-SPEC-024: A `${env.VAR_NAME}` token in a sensor spec string field could not be
    /// resolved because the named environment variable is absent or empty.
    ///
    /// Emitted during spec loading (post-TOML-parse, pre-URL-format-validation pass) when
    /// `resolve_env_var_tokens` encounters an unset or empty env var. Multiple E-SPEC-024
    /// errors are collected in a single multi-error pass (no fail-fast). The spec is rejected
    /// entirely — no degraded-load state (fail-closed). Boot exits with code 2.
    ///
    /// The env var VALUE is NEVER included in the error message (AD-017 / AI-opaque-credentials).
    /// Only the var NAME and TOML field path are reported.
    ///
    /// BC-2.16.009 §Validation Rules 6 (AC-6); S-SPEC-ENV-VAR-001.
    /// error-taxonomy.md §E-SPEC-024.
    ESpec024,
    /// E-SPEC-025: A `FetchStep::method` value (after env-var token resolution) is not in the
    /// allowed HTTP method set: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`.
    ///
    /// Emitted by `validate_step_methods()` in `crates/prism-spec-engine/src/validation.rs`
    /// when a step's `method` field contains a value not in `ALLOWED_HTTP_METHODS`.
    ///
    /// Validation is case-sensitive and upper-case only — `"get"` is invalid, `"GET"` is valid.
    /// Absent `step.method` (defaulting to `"GET"`) is NOT an error. Unsupported methods
    /// (`CONNECT`, `TRACE`) and typos (`"GETT"`) produce this error. Multiple invalid steps
    /// each produce a separate E-SPEC-025 error; all collected in the same multi-error pass
    /// (INV-ERR-003). Rule 7 skips step.method fields that already failed Rule 6 (E-SPEC-024).
    ///
    /// The method VALUE is safe to echo in the message — it is config text, not a credential
    /// per AD-017.
    ///
    /// BC-2.16.009 §Validation Rules 7 (AC-7); S-SPEC-HTTP-METHOD-VALIDATION-001.
    /// error-taxonomy.md E-SPEC-025.
    ESpec025,
    /// E-SPEC-026: `probe_table` names a table not present in the sensor spec's `[[tables]]` blocks.
    ///
    /// Emitted by `SpecLoader::parse()` Rule 8 when `spec.probe_table` is `Some(name)` but no
    /// `[[tables]]` block has `table_name = name`.
    ///
    /// Error message template (probe-table-field-design.md §1, BC-2.16.009 Rule 8):
    ///   "sensor '{sensor_id}' declares probe_table = '{name}' but no [[tables]] block
    ///    has table_name = '{name}'. Declared tables: [{table_list}]. Remove probe_table
    ///    or add a matching [[tables]] block."
    ///
    /// BC-2.08.001 Error Cases E-SPEC-026; BC-2.16.009 Validation Rule 8.
    ///
    ESpec026,
}

/// A structured spec validation or runtime error carrying an E-SPEC-* code,
/// a human-readable message, and an optional TOML path for actionable correction.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("spec error {code:?} at {toml_path:?}: {message}")]
pub struct SpecError {
    pub code: SpecErrorCode,
    pub message: String,
    /// TOML path for user-actionable correction (e.g., `sensor.tables[0].steps[1].path_template`).
    pub toml_path: Option<String>,
    /// Source file path, if known.
    pub file_path: Option<String>,
    /// Line number in the source file, if known.
    pub line_number: Option<u32>,
}

// ---------------------------------------------------------------------------
// E-INFUSE — Infusion enrichment framework errors (S-1.14)
// ---------------------------------------------------------------------------

/// E-INFUSE-* error codes from BC-2.19.001 through BC-2.19.005.
///
/// These errors are produced by `InfusionRegistry` and `InfusionLoader` during
/// spec loading, hot reload, and credential resolution.
///
/// Marked `#[non_exhaustive]` per CLAUDE.md pub-API surface discipline — external
/// match arms must include a wildcard `_ => {}` arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InfusionError {
    /// E-INFUSE-001: Unknown infusion name referenced in a query or pipe stage.
    #[error(
        "E-INFUSE-001: Unknown infusion '{name}'. Run list_infusions to see available enrichments."
    )]
    UnknownInfusion { name: String },

    /// E-INFUSE-002: Duplicate UDF name across multiple infusion specs.
    #[error(
        "E-INFUSE-002: Duplicate UDF name '{udf_name}' in '{path2}' — already registered from '{path1}'."
    )]
    DuplicateUdfName {
        udf_name: String,
        path1: String,
        path2: String,
    },

    /// E-INFUSE-003: Missing required field in infusion spec.
    #[error("E-INFUSE-003: Missing required field '{field}' in infusion spec '{spec_path}'.")]
    MissingRequiredField { field: String, spec_path: String },

    /// E-INFUSE-004: Unknown source type in infusion spec.
    #[error(
        "E-INFUSE-004: Unknown source type '{type_name}'. Valid types: maxmind_mmdb, csv, json_lookup, plugin, http_lookup."
    )]
    UnknownSourceType { type_name: String },

    /// E-INFUSE-005: Credential cannot be resolved.
    /// NOTE: The message MUST NOT include the credential value — only the field name,
    /// infusion_id, and env_var_name are safe to log (BC-2.19.005).
    #[error(
        "E-INFUSE-005: Credential '{field_name}' for infusion '{infusion_id}' could not be resolved. Ensure '{env_var_name}' is set."
    )]
    CredentialUnresolved {
        field_name: String,
        infusion_id: String,
        env_var_name: String,
    },

    /// E-RULE-012: Detection rule filter references an API-backed infusion UDF.
    #[error(
        "E-RULE-012: Detection rule filter references API-backed infusion UDF '{udf_name}' (from infusion '{infusion_id}', type 'plugin'). API-backed infusions cannot be used in detection rules — use a local_lookup infusion instead."
    )]
    ApiBackedUdfInDetectionRule {
        udf_name: String,
        infusion_id: String,
    },

    /// E-INFUSE-013: Invalid field specification in infusion spec.
    ///
    /// E-INFUSE-006 ("Infusion not found") is already the tombstoned code for the
    /// `SpecNotFound` variant (see §BC-2.19.001 error taxonomy). This variant uses
    /// E-INFUSE-013 to avoid the collision (PO allocation 2026-06-19).
    #[error(
        "E-INFUSE-013: invalid field name '{field}' in infusion spec '{spec_path}': {message}"
    )]
    InvalidFieldSpec {
        field: String,
        spec_path: String,
        message: String,
    },

    /// E-INFUSE-008: Plugin infusion call failed at the WASM runtime boundary.
    ///
    /// Returned by `map_plugin_error_to_infusion_error` in `plugin_bridge.rs` when
    /// `PluginRuntime::enrich_single` returns any `PluginError` variant. The `reason`
    /// field carries a human-readable description of the PluginError — credential values
    /// MUST NOT appear in `reason` (INV-INFUSE-005 / AD-017).
    ///
    /// Added in S-1.14-REDO (task from `plugin_bridge.rs` TODO comment).
    ///
    /// `InfusionError` is `#[non_exhaustive]` per CLAUDE.md pub-API surface discipline.
    #[error(
        "E-INFUSE-008: plugin infusion call failed for '{infusion_id}' via plugin '{plugin_id}': {reason}"
    )]
    PluginCallFailed {
        /// The plugin_id as registered in PluginRuntime (e.g., `"threat_intel"`).
        plugin_id: String,
        /// The infusion_id from the InfusionSpec (same as plugin_id in current wiring).
        infusion_id: String,
        /// Human-readable failure reason derived from PluginError display.
        /// Credential values MUST NOT appear here (INV-INFUSE-005 / AD-017).
        reason: String,
    },

    /// E-INFUSE-009: HTTP lookup failed for an http_lookup-type infusion.
    /// `message` MUST NOT contain credential values (AD-017).
    #[error(
        "E-INFUSE-009: HTTP lookup failed for infusion '{infusion_id}' (spec: '{spec_path}'): {message}"
    )]
    HttpLookupFailed {
        infusion_id: String,
        spec_path: String,
        status_code: Option<u16>,
        message: String,
    },

    /// E-INFUSE-010: Credential resolution failed for an http_lookup-type infusion.
    /// The env var name MUST NOT appear in the message (AD-017).
    #[error(
        "E-INFUSE-010: credential resolution failed for infusion '{infusion_id}' (spec: '{spec_path}'): credential '{credential_ref}' not available at call time"
    )]
    CredentialResolutionFailed {
        infusion_id: String,
        spec_path: String,
        credential_ref: String,
    },

    /// E-INFUSE-011: SSRF protection rejected the base_url for an http_lookup-type infusion.
    /// The resolved IP address MUST NOT appear in the message (CWE-209).
    #[error(
        "E-INFUSE-011: SSRF protection rejected infusion '{infusion_id}' (spec: '{spec_path}'): base_url resolves to a private or loopback address; set PRISM_DTU_MODE=true to override for test/demo deployments"
    )]
    SsrfRejected {
        infusion_id: String,
        spec_path: String,
    },

    /// E-INFUSE-012: Infusion source file exceeds `MAX_SOURCE_FILE_BYTES` (100 MiB).
    ///
    /// Detected via `fs::metadata(&path)?.len()` BEFORE any file read, preventing
    /// CWE-400 unbounded-memory OOM. Fires at load time and hot-reload time for
    /// CSV, JSON-lookup, and MMDB sources.
    ///
    /// `{path}` — file path (safe to log, not a credential per AD-017).
    /// `{size}` — actual file byte-length from `fs::metadata`.
    /// `{limit}` — configured limit constant (default `MAX_SOURCE_FILE_BYTES = 104857600`).
    ///
    /// MCP surface: propagates as a spec load error; the infusion is unavailable
    /// until the file is reduced or the limit is raised. Non-retryable without a
    /// file or config change. SEC-001 (CWE-400); BC-2.19.001 §Error Conditions E-INFUSE-012.
    #[error(
        "E-INFUSE-012: infusion source file '{path}' exceeds maximum size ({size} bytes > {limit} bytes); reduce the file or raise MAX_SOURCE_FILE_BYTES"
    )]
    SourceFileTooLarge {
        /// The file path (safe to log — not a credential per AD-017).
        path: String,
        /// Actual file size in bytes from `fs::metadata().len()`.
        size: u64,
        /// The configured limit (default `MAX_SOURCE_FILE_BYTES = 104_857_600`).
        limit: u64,
    },

    /// E-INFUSE-014: Runtime coercion failure for a typed enrichment UDF output field.
    ///
    /// Emitted as `tracing::warn!` (not a query error) when a projected value cannot be
    /// coerced to the declared `output_type`. The output row contains NULL.
    ///
    /// Defined in ADR-051 D2 and BC-2.19.001 §E-INFUSE-014.
    /// A corresponding BC-2.16.002 Canonical Structured Event Catalog row for
    /// `event_type = "infusion.coercion_failed"` MUST be added in the same commit as
    /// the `tracing::warn!` emission in `infusion_udf.rs` (SAP-1 obligation; AC-012).
    ///
    /// `truncated_value`: first 50 chars of the projected string value (AD-017 guard —
    /// enrichment response values are external data, not credentials, but truncated as
    /// a defense-in-depth precaution).
    ///
    /// Variant is `#[non_exhaustive]` per CLAUDE.md §Conventions pub-API discipline.
    /// Adding new fields (e.g., `row_index: usize`) in a follow-up story will not break
    /// existing match arms in external crates.
    ///
    /// Story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 (AC-005).
    #[non_exhaustive]
    #[error(
        "E-INFUSE-014: enrichment field '{field_name}' (infusion '{infusion_id}'): \
         declared output_type is '{declared_type}', but projected value \
         '{truncated_value}' (first 50 chars) cannot be coerced; row produces NULL"
    )]
    TypeCoercionFailed {
        /// The UDF / infusion field name (e.g., `"threat_score"`).
        field_name: String,
        /// The `infusion_id` from the `InfusionSpec` (e.g., `"threat_intel"`).
        infusion_id: String,
        /// The declared `output_type` value (e.g., `"integer"`, `"float"`, `"boolean"`, `"datetime"`).
        declared_type: String,
        /// First 50 characters of the projected string value (AD-017: truncated to prevent
        /// accidental exposure of long external data in structured log lines).
        truncated_value: String,
    },
}

impl InfusionError {
    /// Construct an `E-INFUSE-014: TypeCoercionFailed` error from outside `prism-core`.
    ///
    /// Required because `TypeCoercionFailed` is `#[non_exhaustive]`, which prevents struct
    /// literal construction from outside the defining crate.  Callers in `prism-query`
    /// (infusion_udf.rs `coerce_to_typed`) use this to emit the canonical E-INFUSE-014
    /// Display format via `tracing::warn!("{}", err)`.
    ///
    /// `value` is truncated to 50 codepoints per AD-017 (CWE-532 guard — enrichment response
    /// values are external data; truncated as defense-in-depth, consistent with the analogous
    /// truncation in `parse_datetime_to_micros`).
    ///
    /// SEC-001 (CWE-117, error-taxonomy v2.17): `field_name`, `infusion_id`, and `declared_type`
    /// are stripped of ASCII control characters (0x00–0x1F, 0x7F) before storage to prevent
    /// log injection and LLM prompt injection when rendered via `tracing::warn!("{}", err)`.
    /// `truncated_value` is stripped AFTER the 50-char truncation (truncation removes excess
    /// content first; stripping removes any control chars that survived the 50-char window).
    ///
    /// Story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 LOCAL adversary pass-1 MED-001 fix.
    pub fn new_type_coercion_failed(
        field_name: impl Into<String>,
        infusion_id: impl Into<String>,
        declared_type: impl Into<String>,
        value: &str,
    ) -> Self {
        // SEC-001 (CWE-117): strip control chars from all metadata fields and from
        // truncated_value (AFTER the 50-char truncation — order matters per spec).
        let truncated: String = value.chars().take(50).collect();
        Self::TypeCoercionFailed {
            field_name: sanitize_for_log(&field_name.into()),
            infusion_id: sanitize_for_log(&infusion_id.into()),
            declared_type: sanitize_for_log(&declared_type.into()),
            truncated_value: sanitize_for_log(&truncated),
        }
    }
}

/// Strip ASCII control characters (0x00–0x1F, 0x7F) from `s` before embedding in log or error
/// messages.
///
/// **Contract A — canonical log/error-message sanitizer (CWE-117):**
/// Removes chars where `c.is_ascii_control()` is true; no length cap; no replacement character.
/// Use this function for log and error message value sanitization across all crates.
/// Distinct from `prism_spec_engine::overlay::sanitize_for_display` which uses U+FFFD replacement
/// and a 256-char cap for display-facing overlay error strings.
///
/// Prevents CWE-117 log injection and LLM prompt injection into agent-consumed structured logs
/// (AD-017 extension, error-taxonomy v2.17 SEC-001 Rendering Note).
pub fn sanitize_for_log(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_control()).collect()
}

// ---------------------------------------------------------------------------
// E-PLUGIN — WASM Plugin Runtime error types (S-1.15)
// ---------------------------------------------------------------------------

/// E-PLUGIN-* error codes from BC-2.17.001 through BC-2.17.007 (S-1.15, S-PLUGIN-PREREQ-D).
///
/// These variants are returned at the `instance.call_*` boundary in `prism-spec-engine`
/// and MUST NOT propagate as panics into the host tokio runtime. All `PluginError`
/// variants correspond to sandbox isolation, resource enforcement, or contract
/// validation failures — the host process continues executing normally after any
/// `PluginError` is returned.
///
/// Marked `#[non_exhaustive]` per project convention (CLAUDE.md) — external match arms
/// must include a wildcard `_ => {}` arm. New variants are added additively (POL-1).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PluginError {
    /// E-PLUGIN-004: WASM trap caught at host boundary (BC-2.17.001 / INV-PLUGIN-001).
    /// The plugin executed an `unreachable` instruction, caused a memory fault, or
    /// triggered any other fatal WASM error. Host process is unaffected.
    #[error("plugin '{plugin_id}' trapped: {message}")]
    Trapped { plugin_id: String, message: String },

    /// E-PLUGIN-007: Plugin call exceeded its CPU time limit via epoch interruption
    /// (BC-2.17.004 / INV-PLUGIN-004). Default limit is 5 seconds per call.
    #[error("plugin '{plugin_id}' timed out after {duration_ms}ms")]
    Timeout { plugin_id: String, duration_ms: u64 },

    /// E-PLUGIN-006: Plugin instance attempted to allocate memory beyond its configured
    /// limit (default 64MB) via `wasmtime::StoreLimits` (BC-2.17.003 / INV-PLUGIN-003).
    #[error("plugin '{plugin_id}' exceeded memory limit of {limit_mb}MB")]
    MemoryExceeded { plugin_id: String, limit_mb: u64 },

    /// E-PLUGIN-011: Plugin with the given `plugin_id` is not loaded in the registry
    /// (BC-2.17.005 — deletion path). Callers should call `list_plugins` to enumerate
    /// available plugins.
    #[error("plugin '{plugin_id}' is not loaded")]
    NotLoaded { plugin_id: String },

    /// E-PLUGIN-001: Plugin binary does not implement a recognized Prism WIT interface
    /// (BC-2.17.006 / INV-PLUGIN-006). The `missing_export` field names the first
    /// required export that was absent from the component.
    #[error(
        "plugin '{path}' does not implement a recognized Prism WIT interface. \
         Expected one of: prism:sensor-plugin, prism:infusion-plugin, prism:action-plugin. \
         Missing export: {missing_export}"
    )]
    InvalidInterface {
        path: String,
        missing_export: String,
    },

    /// E-PLUGIN-005: Plugin attempted an HTTP request to a URL not in the configured
    /// allowlist (BC-2.17.002 — URL allowlist enforcement).
    #[error("plugin '{plugin_id}' attempted HTTP to non-allowlisted URL: {url}")]
    SandboxViolation { plugin_id: String, url: String },

    /// E-PLUGIN-008: Plugin binary failed WASM Component Model compilation
    /// (BC-2.17.005 — failed hot reload path; BC-2.17.006).
    #[error("plugin '{path}' failed to compile: {message}")]
    CompilationFailed { path: String, message: String },

    /// E-PLUGIN-010: Plugin's `name()` export returned an empty string; a plugin_id
    /// cannot be empty (BC-2.17.006 post-validation check).
    #[error("plugin '{path}' returned an empty plugin_id from name()")]
    EmptyPluginId { path: String },

    // -------------------------------------------------------------------------
    // S-PLUGIN-PREREQ-D: Manifest schema validation errors (BC-2.17.007)
    // -------------------------------------------------------------------------
    /// E-PLUGIN-013: Plugin manifest missing required `allowed_urls` field (BC-2.17.007 / VP-PLUGIN-007).
    ///
    /// `allowed_urls` must be explicitly present in the manifest; absent or null → rejection.
    /// An explicitly empty list `[]` is accepted (default-deny semantics).
    #[error(
        "Plugin manifest at '{path}' missing required field 'allowed_urls'; \
         field must be an explicit list (use `allowed_urls = []` for no URLs)"
    )]
    MissingAllowedUrls { path: String },

    /// E-PLUGIN-014: Plugin manifest `format_version` exceeds `CURRENT_SUPPORTED_VERSION` (BC-2.17.007).
    ///
    /// Manifest `format_version` must be `<= CURRENT_SUPPORTED_VERSION`; higher versions are rejected.
    #[error(
        "Plugin manifest at '{path}' format_version {actual} exceeds maximum supported version {supported}"
    )]
    FormatVersionExceeded {
        path: String,
        actual: u32,
        supported: u32,
    },

    /// E-PLUGIN-015: Plugin manifest `name` field is absent or an empty string (BC-2.17.007 / EC-D-012).
    ///
    /// The `name` field must be a non-empty UTF-8 string.
    #[error("Plugin manifest at '{path}' missing or empty required field 'name'")]
    ManifestNameMissing { path: String },

    /// E-PLUGIN-016: Plugin manifest `version` field is not valid semver (BC-2.17.007 / EC-D-013).
    ///
    /// The `version` field must parse as a valid semver string (e.g., `"1.0.0"`).
    #[error("Plugin manifest at '{path}' field 'version' is not a valid semver string: '{value}'")]
    ManifestVersionMalformed { path: String, value: String },

    /// E-PLUGIN-017: Plugin manifest TOML is present but fails to parse (BC-2.17.007).
    ///
    /// The companion `.manifest.toml` file exists but is structurally invalid TOML
    /// (syntax error, invalid encoding). This is distinct from E-PLUGIN-015 (name absent)
    /// which applies only when TOML parses correctly but a field is missing.
    #[error("Plugin manifest at '{path}' failed TOML parse: {detail}")]
    ManifestParseError { path: String, detail: String },

    /// E-PLUGIN-018: Plugin manifest file is absent entirely (BC-2.17.007).
    ///
    /// The `.prx` plugin binary was found but no companion `.manifest.toml` exists at
    /// the expected path. A manifest is required for all production plugins.
    #[error(
        "Plugin at '{plugin_path}' has no manifest file at '{expected_manifest_path}'; \
         a companion .manifest.toml is required"
    )]
    ManifestNotFound {
        plugin_path: String,
        expected_manifest_path: String,
    },

    /// E-PLUGIN-019: Plugin manifest `format_version` field is absent (BC-2.17.007 / AC-5).
    ///
    /// `format_version` must be explicitly present; absent means the manifest predates the
    /// versioning scheme or is malformed. Distinct from E-PLUGIN-014 (value exceeds max).
    #[error(
        "Plugin manifest at '{path}' missing required field 'format_version'; \
         must be an integer <= {supported}"
    )]
    FormatVersionMissing { path: String, supported: u32 },

    /// E-PLUGIN-022: Plugin acquire-token dispatch completed but no token was cached in KV store
    /// (BC-2.16.002 row 37 — host-observable symptom of guest `AuthError::ResponseParse` or
    /// missing `kv_set` call).
    ///
    /// This is a **runtime-behavioral failure**, not a compilation failure. The WASM Component
    /// Model binary compiled and was instantiated successfully; the guest's `acquire-token`
    /// function was dispatched successfully (no trap, no timeout); but after dispatch, the KV
    /// store has no "token" entry — indicating the guest either:
    ///   - returned `AuthError::ResponseParse` (invalid JSON, missing `access_token` field), or
    ///   - called `kv_set` with a key other than "token", or
    ///   - returned without calling `kv_set` at all.
    ///
    /// Operators searching for token-parse failures should grep for `E-PLUGIN-022`, NOT
    /// `E-PLUGIN-008` (which is a compilation failure). The structured tracing event
    /// `plugin.auth_token_parse_error` (BC-2.16.002 row 37) is emitted by the host BEFORE
    /// this error is returned. Cross-reference: PLUGIN-MIGRATION-001-E EC-002, EC-003;
    /// F-LP8-MED-002 closure.
    #[error(
        "plugin '{plugin_id}' acquire-token dispatch completed but no token was cached in \
         KV store (guest AuthError::ResponseParse or missing kv_set call): {message}"
    )]
    AuthTokenNotCached { plugin_id: String, message: String },

    /// E-PLUGIN-023: Plugin `enrich-single` call completed but returned an invalid or
    /// unparseable result — the JSON string returned by the guest could not be deserialized,
    /// OR the Val type in `results[0]` was not the expected `Val::Option<Val::String>`.
    #[error("plugin '{plugin_id}' enrich-single call failed: {reason}")]
    EnrichCallFailed { plugin_id: String, reason: String },
}

// ---------------------------------------------------------------------------
// Tests — SpecErrorCode (Task 11, PLUGIN-MIGRATION-001-D)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Task 11 unit test: ESpec017 variant constructor and Display (D-737 Decision 3).
    ///
    /// Verifies the new ESpec017 variant is constructable and displays correctly
    /// via the SpecError Display impl (`#[error("spec error {code:?} at {toml_path:?}: {message}")]`).
    #[test]
    fn test_e_spec_017_variant_constructor_and_display() {
        let err = SpecError {
            code: SpecErrorCode::ESpec017,
            message: "Sensor spec `falcon` does not match filename stem `crowdstrike`".to_string(),
            toml_path: None,
            file_path: Some("crowdstrike.sensor.toml".into()),
            line_number: None,
        };
        assert_eq!(err.code, SpecErrorCode::ESpec017);
        let display = format!("{err}");
        assert!(
            display.contains("ESpec017"),
            "display must mention variant: {display}"
        );
        assert!(
            display.contains("does not match filename stem"),
            "display must include message: {display}"
        );
    }

    /// E-QUERY-003 security-only variant Display (error-taxonomy.md §E-QUERY-003,
    /// ADR-038 v1.3 §P5-02): exactly "E-QUERY-003: {detail}" — the Display
    /// impl supplies the single canonical prefix; `detail` carries no prefix.
    #[test]
    fn test_query_security_limit_exceeded_display_e_query_003() {
        let err = PrismError::QuerySecurityLimitExceeded {
            detail: "query exceeds maximum size of 8192 bytes".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "E-QUERY-003: query exceeds maximum size of 8192 bytes"
        );
    }

    /// E-QUERY-034 generic execution error Display (error-taxonomy.md §E-QUERY-034,
    /// ADR-038 v1.3 §P5-02): `QueryExecutionFailed` renumbered 003 → 034.
    #[test]
    fn test_query_execution_failed_display_e_query_034() {
        let err = PrismError::QueryExecutionFailed {
            detail: "DataFusion plan execution aborted".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "E-QUERY-034: query execution error: DataFusion plan execution aborted"
        );
    }

    /// E-QUERY-036 Display — no `did_you_mean` (error-taxonomy.md canonical format, OBS-1 fix).
    ///
    /// Asserts the verbatim message format byte-for-byte:
    ///   `E-QUERY-036: unknown source table '{source_name}': table is not a registered
    ///    sensor or internal table. Check spelling or register the sensor in prism.toml.
    ///    Available tables: [{available_tables}].`
    ///
    /// The label must be "Available tables:" (not the retired "Available sensors:").
    #[test]
    fn test_unknown_source_table_display_no_did_you_mean() {
        let detail = UnknownSourceTableDetails::new(
            "ghost_sensor.devices",
            vec!["armis".to_string(), "crowdstrike".to_string()],
            None,
        );
        assert_eq!(
            format!("{detail}"),
            "E-QUERY-036: unknown source table 'ghost_sensor.devices': table is not a registered \
             sensor or internal table. Check spelling or register the sensor in prism.toml. \
             Available tables: [armis, crowdstrike]."
        );
    }

    /// E-QUERY-036 Display — with `did_you_mean` (error-taxonomy.md canonical format, OBS-1 fix).
    ///
    /// Asserts the verbatim suffix `" Did you mean: '{candidate}'?"` — leading space, colon
    /// after "mean", single-quoted candidate, trailing question mark — matching
    /// E-QUERY-037 `TableNotAvailableDetails` convention (OBS-2 parity).
    #[test]
    fn test_unknown_source_table_display_with_did_you_mean() {
        let detail = UnknownSourceTableDetails::new(
            "crowdstrik",
            vec!["crowdstrike".to_string()],
            Some("crowdstrike".to_string()),
        );
        assert_eq!(
            format!("{detail}"),
            "E-QUERY-036: unknown source table 'crowdstrik': table is not a registered \
             sensor or internal table. Check spelling or register the sensor in prism.toml. \
             Available tables: [crowdstrike]. Did you mean: 'crowdstrike'?"
        );
    }

    /// POL-24 / TD-VSDD-059: `PrismError::TemporalLiteralUnparseable` Display must match
    /// error-taxonomy.md §E-QUERY-041 message template byte-for-byte.
    ///
    /// Taxonomy template (error-taxonomy.md §E-QUERY-041 Message Format):
    ///   "E-QUERY-041: The value '<value_prefix>' cannot be interpreted as a UTC timestamp.
    ///    Expected RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z').
    ///    Date-only and offset-less forms are not accepted.
    ///    For relative time filters, use NOW() - INTERVAL 'Nh'
    ///    (e.g., WHERE timestamp > NOW() - INTERVAL '24h')."
    ///
    /// Any change to the Display impl that breaks this test is a POLICY 24 violation
    /// requiring an error-taxonomy.md version bump and synchronized test update.
    /// Mirrors the pattern established by `test_E_SPEC_018_display_matches_error_taxonomy_template_byte_for_byte`
    /// in `crates/prism-spec-engine/src/error.rs` (TD-VSDD-060 sibling-site discipline).
    ///
    /// Traces to: error-taxonomy.md §E-QUERY-041; ADR-052 §D4;
    /// S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 LOW-1 (adversary pass-3).
    #[test]
    fn test_E_QUERY_041_display_matches_error_taxonomy_template_byte_for_byte() {
        let err = PrismError::TemporalLiteralUnparseable {
            value_prefix: "2026-06-24".to_string(),
        };
        let display = err.to_string();
        let expected = "E-QUERY-041: The value '2026-06-24' cannot be interpreted as a UTC \
                        timestamp. Expected RFC-3339 format with UTC offset \
                        (e.g., '2026-07-03T00:00:00Z'). Date-only and offset-less forms are \
                        not accepted. For relative time filters, use NOW() - INTERVAL 'Nh' \
                        (e.g., WHERE timestamp > NOW() - INTERVAL '24h').";
        assert_eq!(
            display, expected,
            "E-QUERY-041 Display must match error-taxonomy.md §E-QUERY-041 template \
             byte-for-byte (POLICY 24). Got:\n  {display:?}\nExpected:\n  {expected:?}"
        );
    }
}
