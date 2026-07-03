// S-1.12: Error types for prism-spec-engine.
// E-SPEC-002: filesystem write failure (BC-2.16.008)
// E-RELOAD-001..004: reload error conditions (BC-2.16.005)
// S-3.1.05: OrgId-scoped store errors (BC-3.1.001 AC-1, AC-3)

use prism_core::OrgSlug;
use thiserror::Error;

/// Top-level spec engine error.
///
/// Marked `#[non_exhaustive]` to allow adding variants in future stories
/// without a major version bump (workspace convention, see `prism-core::PrismError`).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SpecEngineError {
    /// E-RELOAD-001: Config file read error (file not found, permission denied)
    #[error("E-RELOAD-001: Failed to read config file '{path}': {os_error}")]
    FileReadError { path: String, os_error: String },

    /// E-RELOAD-002: Validation failed for prism.toml or aliases.toml (Tier 1/2)
    #[error("E-RELOAD-002: Config validation failed: {errors:?}")]
    ValidationFailed { errors: Vec<String> },

    /// E-RELOAD-003: Partial reload — some sensor spec files failed validation
    #[error("E-RELOAD-003: Partial reload: {failed_count} spec(s) failed validation")]
    PartialReloadFailure { failed_count: usize },

    /// E-RELOAD-004: No changes detected (all files match previous hash)
    #[error("E-RELOAD-004: No changes detected; reload is a no-op")]
    NoChangesDetected,

    /// E-SPEC-002: Filesystem write failure with path and OS error (BC-2.16.008)
    #[error("E-SPEC-002: Failed to write spec file '{path}': {os_error}")]
    SpecWriteError { path: String, os_error: String },

    /// TOML parse error
    #[error("TOML parse error in '{path}': {detail}")]
    TomlParseError { path: String, detail: String },

    /// Watcher setup error
    #[error("Filesystem watcher setup failed: {detail}")]
    WatcherError { detail: String },

    // -------------------------------------------------------------------------
    // S-3.1.05 — OrgId-scoped spec store errors (BC-3.1.001)
    // -------------------------------------------------------------------------
    /// E-SPEC-ORG-001: Slug not registered in OrgRegistry (BC-3.1.001 AC-1).
    ///
    /// Returned by `OrgScopedSpecStore::get_spec` when
    /// `OrgRegistry::resolve(slug)` returns `None`.
    #[error("E-SPEC-ORG-001: unknown org slug '{slug}' — not registered in OrgRegistry")]
    UnknownOrg { slug: OrgSlug },

    /// E-SPEC-ORG-002: Org exists but has no spec for the requested sensor (BC-3.1.001 AC-1, EC-002).
    ///
    /// Returned by `OrgScopedSpecStore::get_spec` when the org is known but
    /// `(OrgId, sensor_name)` is absent from the internal store.
    #[error("E-SPEC-ORG-002: org '{slug}' has no spec for sensor '{sensor}'")]
    SensorNotFound { slug: OrgSlug, sensor: String },

    /// E-SPEC-ORG-003: OrgRegistry not yet initialized (BC-3.1.001 AC-3).
    ///
    /// In practice unreachable post-startup (BC-3.1.001 invariant 3), but the
    /// API MUST NOT panic — it returns this error instead.
    #[error("E-SPEC-ORG-003: OrgRegistry not initialized; call is too early in startup sequence")]
    RegistryNotInitialized,

    // -------------------------------------------------------------------------
    // S-PLUGIN-PREREQ-B — PipelineExecutor auth errors (BC-2.16.002, BC-2.01.013)
    // -------------------------------------------------------------------------
    /// E-AUTH-001: Auth token acquisition failed for the given sensor / client.
    ///
    /// Returned by `AuthProvider::acquire_token` implementations when credentials
    /// cannot be resolved (bad config, network failure, invalid auth_type).
    #[error(
        "E-AUTH-001: auth token acquisition failed for sensor '{sensor_id}', client '{client_id}': {detail}"
    )]
    AuthAcquisitionFailed {
        sensor_id: String,
        client_id: String,
        detail: String,
    },

    /// E-AUTH-002: Auth refresh failed — double-401 after token re-acquisition.
    ///
    /// Returned by `PipelineExecutor::execute` when a step returns HTTP 401,
    /// `acquire_token` is called to get a fresh token, and the retry ALSO
    /// returns HTTP 401. Pipeline aborts; no further retries (AC-5 abort condition).
    ///
    /// This variant is ONLY returned for refreshable auth types (e.g.,
    /// `Oauth2ClientCredentials`). Static auth types (e.g., `CookieRoundtrip`)
    /// return `CookieAuthFailed` (E-AUTH-004) instead — they have no refresh
    /// mechanism, so retrying is provably futile (BC-2.01.017 EC-017-002).
    #[error(
        "E-AUTH-002: auth refresh failed for sensor '{sensor_id}', client '{client_id}': \
         HTTP 401 persisted after token re-acquisition on step '{step_name}'"
    )]
    AuthRefreshFailed {
        sensor_id: String,
        client_id: String,
        step_name: String,
    },

    /// E-AUTH-004: Cookie authentication failed — HTTP 401 received on a
    /// `CookieRoundtrip` auth sensor; no retry is attempted.
    ///
    /// Returned by `PipelineExecutor::execute` (via `issue_request_with_retry`)
    /// when a step returns HTTP 401 and the sensor's `auth_type` is
    /// `CookieRoundtrip`. Unlike the OAuth2 refresh path, `CookieRoundtrip`
    /// uses a static API key — `acquire_token()` would just re-read the same
    /// key, making retry provably futile (BC-2.01.017 EC-017-002 / TV-BC-2.01.017-006).
    ///
    /// The pipeline aborts with this variant (not `AuthRefreshFailed`) so that
    /// the query fan-out layer can treat this as a per-sensor partial failure
    /// (BC-2.01.010 partial-failure fan-out semantics).
    ///
    /// Error format matches the E-AUTH-004 entry in error-taxonomy.md:
    /// `"Cookie authentication failed for {sensor} on client '{client_id}'"`
    #[error("E-AUTH-004: Cookie authentication failed for {sensor_id} on client '{client_id}'")]
    CookieAuthFailed {
        /// Sensor identity for operator diagnostics.
        sensor_id: String,
        /// Client (org) for which the auth failed.
        client_id: String,
    },

    /// E-HTTP-001: HTTP request failed (non-401 error, e.g., 500, network error).
    ///
    /// Returned by `PipelineExecutor` when a step receives a non-retryable HTTP
    /// error. Pipeline aborts (EC-007).
    #[error(
        "E-HTTP-001: HTTP {status_code} from sensor '{sensor_id}' step '{step_name}': {detail}"
    )]
    HttpRequestFailed {
        sensor_id: String,
        step_name: String,
        status_code: u16,
        detail: String,
    },

    /// E-JSONPATH-001: JSONPath extraction failed — `response_path` did not match
    /// the response structure (E-SPEC-010).
    ///
    /// Step name and path are included for diagnostics (BC-2.16.002 error table).
    /// `detail` carries the descriptive message from `extract_at_path` (e.g.,
    /// "index 99 out of bounds: array has 3 elements in path '$.x[99]'").
    #[error(
        "E-JSONPATH-001 (E-SPEC-010): response_path '{path}' did not match response \
         from sensor '{sensor_id}' step '{step_name}': {detail}"
    )]
    JsonPathExtractionFailed {
        sensor_id: String,
        step_name: String,
        path: String,
        /// Descriptive extraction failure reason from the JSONPath engine.
        detail: String,
    },

    // -------------------------------------------------------------------------
    // S-PLUGIN-PREREQ-D — PipelineExecutor cumulative request cap (AC-16)
    // -------------------------------------------------------------------------
    /// E-PIPELINE-001: Cumulative HTTP request count across all pipeline steps reached
    /// `MAX_REQUESTS_PER_PIPELINE` (10,000); pipeline aborts immediately.
    ///
    /// Non-retryable. Partial results are discarded. Anchors: BC-2.16.002 §Postconditions
    /// (Canonical Structured Event Catalog bullet, v1.12) row `pipeline_max_requests_exceeded`;
    /// TD-S-PLUGIN-PREREQ-B-004 closure.
    #[error(
        "E-PIPELINE-001: Pipeline executor reached MAX_REQUESTS_PER_PIPELINE cap of 10_000 \
         ({total} requests attempted); aborting pipeline execution"
    )]
    TooManyRequests {
        /// Total cumulative HTTP requests attempted (always >= MAX_REQUESTS_PER_PIPELINE).
        total: usize,
    },

    // -------------------------------------------------------------------------
    // S-PLUGIN-PREREQ-E — SensorAuth runtime cross-composition rejection variants
    // (BC-2.01.016 Rule 2 / ADR-023 Rule 2 / ADR-026 D3 / Task 6c)
    // -------------------------------------------------------------------------
    /// E-SPEC-012 (Rule A): `auth_type` field is multi-valued or contains a value
    /// outside the closed enumeration `{oauth2_client_credentials, bearer_static,
    /// cookie_roundtrip, api_key, custom_via_plugin}`.
    ///
    /// Sealed-trait compile-time enforcement is replaced by this runtime validator
    /// (BC-2.01.016 Rule 2 / ADR-023 §Architectural Constraints Rule 2, Rule A).
    ///
    /// `provided_value` carries the raw TOML field value that failed validation.
    /// Implementer note (AD-017): `provided_value` is the TOML `auth_type` string,
    /// NOT a credential secret — `Debug` derive is safe for this field.
    #[error(
        "E-SPEC-012: sensor '{sensor_id}' auth_type value '{provided_value}' is invalid — \
         must be a scalar from the closed enumeration per ADR-026 §D3"
    )]
    AuthTypeCrossComposition {
        /// Sensor identity for diagnostics.
        sensor_id: String,
        /// The raw `auth_type` TOML value that failed validation (NOT a credential secret).
        provided_value: String,
    },

    /// E-SPEC-013 (Rule B): Multiple `credential_refs` declared for a single auth method
    /// block (cardinality must be exactly 1 per BC-2.01.016 §Error Cases / ADR-023 Rule B).
    #[error(
        "E-SPEC-013: sensor '{sensor_id}' declares {credential_count} credential_refs — \
         exactly 1 is required per BC-2.01.016 §Error Cases"
    )]
    MultipleCredentialRefs {
        /// Sensor identity for diagnostics.
        sensor_id: String,
        /// Actual number of `credential_refs` entries declared (always > 1 when this error fires).
        credential_count: usize,
    },

    /// E-SPEC-014 (Rule C): Structural mismatch between the resolved credential's shape and
    /// the declared `auth_type` variant (e.g., `auth_type = "bearer_static"` with an
    /// `oauth2_client_credentials`-shaped credential containing `client_id+client_secret`).
    ///
    /// Implementer note (AD-017): `expected_shape` and `actual_shape` are STRUCTURAL
    /// DESCRIPTOR STRINGS (e.g., `"api_key"`, `"client_id+client_secret"`) — NOT raw
    /// credential values. However, the implementer MUST verify at implementation time
    /// that no credential data propagates into these fields (see Task 6c credential
    /// redaction discipline note). `Debug` derive is used for the stub; the implementer
    /// should confirm or replace with a custom `Debug` impl if shapes ever contain secrets.
    #[error(
        "E-SPEC-014: sensor '{sensor_id}' auth_type/credential shape mismatch — \
         expected shape '{expected_shape}', got '{actual_shape}'"
    )]
    AuthTypeCredentialMismatch {
        /// Sensor identity for diagnostics.
        sensor_id: String,
        /// Structural shape expected by the declared `auth_type`.
        expected_shape: String,
        /// Structural shape of the resolved credential (MUST NOT contain raw secret values).
        actual_shape: String,
    },

    /// E-PLUGIN-012: A second `register_write_tool()` call with the same `tool_name`
    /// was rejected. First-registration always persists; last-writer-wins is explicitly
    /// forbidden (ADR-026 §D7; BC-2.16.012 EC-016-012-004).
    ///
    /// The `tool_name` field carries the duplicate tool name for diagnostics.
    /// It is NOT a credential value — `Debug` derive is safe.
    #[error(
        "E-PLUGIN-012: duplicate write tool registration rejected — tool '{0}' is already registered (ADR-026 §D7)"
    )]
    DuplicateWriteToolRegistration(String),

    /// E-PLUGIN-020: `register_write_tool()` called after query-phase flag was set
    /// (i.e., after boot step 8 has started per ADR-022 §B step 7.5/8 ordering).
    ///
    /// Unit variant — dynamic context is carried by the accompanying structured
    /// tracing event fields (`plugin_name`, `tool_name`, `error = "E-PLUGIN-020"`)
    /// per BC-2.16.002 §Postconditions (Canonical Structured Event Catalog) row 33
    /// and ADR-026 §D7. Story: S-PLUGIN-PREREQ-E AC-9 / Task 7c.
    #[error(
        "E-PLUGIN-020: write tool registration rejected — query phase already started (ADR-026 §D7)"
    )]
    WriteToolRegistrationAfterBoot,

    /// E-PLUGIN-021: `DYNAMIC_WRITE_TOOLS` RwLock is poisoned (a previous write-guard
    /// holder panicked while holding the lock). This is an unrecoverable state indicating
    /// a prior bug in the calling code — the registry is permanently corrupted.
    ///
    /// Distinct from `WriteToolRegistrationAfterBoot` (E-PLUGIN-020): poisoning indicates
    /// a programming error (panic in a guard holder), NOT a lifecycle ordering violation.
    ///
    /// Unit variant — no additional context needed; poisoning is self-evident and
    /// the operator should restart the process.
    ///
    /// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P1-004 | ADR-026 §D7
    #[error(
        "E-PLUGIN-021: write tool registry RwLock is poisoned — \
         a prior panic in a write-guard holder has corrupted the registry; \
         process restart required"
    )]
    WriteToolRegistryPoisoned,

    // -------------------------------------------------------------------------
    // PLUGIN-MIGRATION-001-E — auth_plugin registry-membership validation error
    // -------------------------------------------------------------------------
    /// E-SPEC-012 (extended): `auth_plugin` field references a plugin_id that is not
    /// registered in the `PluginRuntime` registry at spec-load time.
    ///
    /// This is the "unknown auth plugin" variant of E-SPEC-012 (which also covers
    /// `auth_type` outside the closed enumeration — `AuthTypeCrossComposition`).
    /// The error code E-SPEC-012 is shared between the two cases per error-taxonomy.md.
    ///
    /// Example: `auth_plugin = "typo-oauth2"` where only `"crowdstrike-oauth2"` is loaded.
    /// A typo'd `auth_plugin` would silently fail in production after 001-A merges.
    ///
    /// Story: PLUGIN-MIGRATION-001-E / F-LP1-CRIT-003 / HIGH-008
    /// Traces to: BC-2.01.016 §Error Cases; ADR-028 §D2; CRIT-003 closure
    #[error(
        "E-SPEC-012: sensor '{sensor_id}' declares auth_plugin = '{plugin_id}' \
         but that plugin is not registered in PluginRuntime — \
         ensure the plugin .prx file is present in the plugin directory at boot step 7.5"
    )]
    UnknownAuthPlugin {
        /// Sensor ID for operator diagnostics.
        sensor_id: String,
        /// The unrecognized plugin_id from the TOML `auth_plugin` field.
        plugin_id: String,
    },

    // -------------------------------------------------------------------------
    // PLUGIN-MIGRATION-001-E F-LP2-MED-002 — structured plugin auth dispatch error
    // -------------------------------------------------------------------------
    /// Plugin auth dispatch failed with a structured `PluginError` (not a stringified message).
    ///
    /// F-LP2-MED-002 closure: replaces the prior `AuthAcquisitionFailed { detail: plugin_err.to_string() }`
    /// pattern which discarded the structured `PluginError` type by stringifying it.
    /// This variant preserves the structured error for machine-readable diagnosis and
    /// allows callers to inspect the `plugin_error` type (e.g., `PluginError::SandboxViolation`
    /// vs `PluginError::NotLoaded`) without parsing a string.
    ///
    /// Also closes the `client_id = "plugin-auth"` sentinel anti-pattern: the variant carries
    /// `sensor_id` and `plugin_id` explicitly (the former provides the org context formerly
    /// communicated by `client_id`).
    ///
    /// AD-017: `plugin_error` field MUST NOT contain credential values. `PluginError` variants
    /// use sensor_id/plugin_id strings only — never credential handles or raw secrets.
    ///
    /// Story: PLUGIN-MIGRATION-001-E / F-LP2-MED-002
    /// Traces to: BC-2.01.016 §Error Cases; error-taxonomy.md E-AUTH-001 / E-AUTH-002
    #[error(
        "plugin auth dispatch failed for sensor '{sensor_id}' via plugin '{plugin_id}': {plugin_error}"
    )]
    #[non_exhaustive]
    AuthPluginDispatchFailed {
        /// Sensor identity for diagnostics.
        sensor_id: String,
        /// The plugin_id that was dispatched (e.g., "crowdstrike-oauth2").
        plugin_id: String,
        /// Structured plugin error from PluginRuntime dispatch.
        ///
        /// Preserves the full structured type hierarchy for machine-readable diagnosis.
        /// MUST NOT contain credential values (AD-017).
        plugin_error: prism_core::PluginError,
    },

    // -------------------------------------------------------------------------
    // S-SPEC-HTTP-METHOD-VALIDATION-001 — HTTP method whitelist error (BC-2.16.009 §VR7)
    // -------------------------------------------------------------------------
    /// E-SPEC-025: A `FetchStep::method` value (after env-var token resolution) is not in
    /// the allowed HTTP method set: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`.
    ///
    /// Emitted by `validate_step_methods()` in `validation.rs`. Case-sensitive. Multiple
    /// invalid steps produce multiple errors in the same multi-error pass (INV-ERR-003).
    /// Absent `step.method` (defaults to `"GET"` via `FetchStep::default()`) is NOT an error.
    /// Rule 7 skips steps whose `method` field already failed Rule 6 (`E-SPEC-024`).
    ///
    /// The method value IS safe to echo (config text, not a credential per AD-017).
    ///
    /// Error message template (error-taxonomy.md E-SPEC-025, byte-verbatim):
    /// `"Step '<step_name>' in '<sensor_id>.<table_name>' declares method '<method_value>'
    ///   which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"`
    ///
    /// BC-2.16.009 §Validation Rules 7; error-taxonomy.md E-SPEC-025;
    /// S-SPEC-HTTP-METHOD-VALIDATION-001.
    #[error(
        "Step '{step_name}' in '{sensor_id}.{table_name}' declares method '{method_value}' which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"
    )]
    InvalidHttpMethod {
        /// Step name from the TOML `name` field.
        step_name: String,
        /// Sensor ID for diagnostics.
        sensor_id: String,
        /// Table name containing the offending step.
        table_name: String,
        /// The resolved (post-Rule-6) method value that failed the whitelist check.
        ///
        /// Safe to echo — HTTP method is config text, not a credential (AD-017).
        method_value: String,
    },

    // -------------------------------------------------------------------------
    // PLUGIN-MIGRATION-001-D — ADR-028 §D8-B/C timestamp normalization errors
    // -------------------------------------------------------------------------
    // -------------------------------------------------------------------------
    // S-SPEC-ENV-VAR-001 — env var token resolution error (BC-2.16.009 §Validation Rules 6)
    // -------------------------------------------------------------------------
    /// E-SPEC-024: A `${env.VAR_NAME}` token in a sensor spec field could not be resolved
    /// because the named environment variable is absent or empty.
    ///
    /// Emitted by `resolve_env_var_tokens` (post-TOML-parse, pre-URL-format-validation).
    /// Multiple instances are collected in a single multi-error pass (no fail-fast).
    /// The spec is rejected entirely — no degraded-load state (fail-closed).
    ///
    /// ## AD-017 no-value-leak discipline
    /// `var_name` carries the environment variable NAME only. The resolved VALUE is NEVER
    /// stored in this variant — not in `var_name`, not in `toml_path`, not in any field.
    /// `SpecEngineError::EnvVarNotSet` construction sites MUST NOT pass
    /// `std::env::var(&var_name).unwrap_or_default()` or any resolved value into any field.
    ///
    /// BC-2.16.009 §Validation Rules 6 (AC-6); error-taxonomy.md §E-SPEC-024;
    /// S-SPEC-ENV-VAR-001.
    #[error(
        "Sensor spec '{file_path}' field '{toml_path}' references environment \
         variable '{var_name}' which is not set or is empty. \
         Set '{var_name}' before starting prism."
    )]
    EnvVarNotSet {
        /// The environment variable NAME (e.g., "ARMIS_INSTANCE_URL").
        ///
        /// MUST be the variable name string — NEVER the resolved value.
        /// AD-017: credential values must not transit AI context or appear in logs.
        var_name: String,
        /// TOML field path containing the unresolved token (e.g., "sensor.base_url").
        toml_path: String,
        /// Source spec file path for operator diagnostics (e.g., "specs/armis.sensor.toml").
        ///
        /// May be an empty string if the spec was constructed in-memory without a file source.
        file_path: String,
    },

    /// E-SPEC-018: `PipelineExecutor` failed to parse a `ColumnType::Datetime` column value
    /// against any of the declared `timestamp_formats`.
    ///
    /// Emitted during response-to-Arrow materialization when `ColumnSpec::timestamp_formats`
    /// is non-empty and no format successfully parsed the field value.
    ///
    /// Field names match the canonical error-taxonomy.md §E-SPEC-018 Message Format template
    /// byte-for-byte (F-LP2-HIGH-002; POLICY 24).
    ///
    /// Maps to `prism_core::SpecErrorCode::ESpec018`.
    /// BC-2.16.013 §O-001; ADR-028 v1.10 §D8-C; error-taxonomy.md §E-SPEC-018.
    #[error(
        "Failed to parse timestamp for column '{column_name}' in sensor '{sensor_id}': \
         tried formats [{}], value='{value}'",
        attempted_formats.join(", ")
    )]
    TimestampParseFailure {
        /// Sensor ID for operator diagnostics (added F-LP2-HIGH-002; POLICY 24).
        sensor_id: String,
        /// Column name for operator diagnostics (the `ColumnSpec.name`).
        column_name: String,
        /// Format names attempted in declaration order, all of which failed.
        attempted_formats: Vec<String>,
        /// The raw JSON value that failed all formats (for actionable correction).
        value: String,
    },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// F-LP3-MEDIUM-003: E-SPEC-018 Display must match error-taxonomy.md §E-SPEC-018 Message Format template
    /// byte-for-byte (POLICY 24). This test pins the format string to prevent silent drift.
    ///
    /// Template: "Failed to parse timestamp for column '{column_name}' in sensor '{sensor_id}':
    ///             tried formats [{formats}], value='{raw_value}'"
    ///
    /// Any change to the Display impl that breaks this test is a POLICY 24 violation
    /// requiring an error-taxonomy.md version bump and synchronized test update.
    #[test]
    fn test_E_SPEC_018_display_matches_error_taxonomy_template_byte_for_byte() {
        let err = SpecEngineError::TimestampParseFailure {
            sensor_id: "cyberint".to_string(),
            column_name: "created_at".to_string(),
            attempted_formats: vec!["iso8601".to_string(), "unix_epoch_seconds".to_string()],
            value: "not_a_timestamp".to_string(),
        };
        let display = err.to_string();
        let expected = "Failed to parse timestamp for column 'created_at' in sensor 'cyberint': \
                        tried formats [iso8601, unix_epoch_seconds], value='not_a_timestamp'";
        assert_eq!(
            display, expected,
            "E-SPEC-018 Display must match error-taxonomy.md v1.45 template byte-for-byte \
             (POLICY 24). Got:\n  {display:?}\nExpected:\n  {expected:?}"
        );
    }

    /// F-LOCAL-P1-MED-001 / POL-24: E-SPEC-024 Display must match error-taxonomy.md §E-SPEC-024
    /// template byte-for-byte (no code prefix). This test pins the format string to prevent
    /// silent drift and mirrors the pattern established for E-SPEC-018.
    ///
    /// Taxonomy template (error-taxonomy.md §E-SPEC-024 Message Format):
    ///   "Sensor spec '{file}' field '{toml_path}' references environment variable '{var_name}'
    ///    which is not set or is empty. Set '{var_name}' before starting prism."
    ///
    /// Any change to the Display impl that breaks this test is a POL-24 violation requiring
    /// an error-taxonomy.md version bump and synchronized test update.
    #[test]
    fn test_E_SPEC_024_display_matches_error_taxonomy_template_byte_for_byte() {
        let err = SpecEngineError::EnvVarNotSet {
            var_name: "ARMIS_INSTANCE_URL".to_string(),
            toml_path: "sensor.base_url".to_string(),
            file_path: "specs/armis.sensor.toml".to_string(),
        };
        let display = err.to_string();
        // Taxonomy template (error-taxonomy.md §E-SPEC-024 Message Format):
        // No "E-SPEC-024: " prefix — matches E-SPEC-018 precedent where the code
        // is NOT prefixed in the Display output (POL-24 byte-for-byte alignment).
        let expected = "Sensor spec 'specs/armis.sensor.toml' field 'sensor.base_url' references \
                        environment variable 'ARMIS_INSTANCE_URL' which is not set or is empty. \
                        Set 'ARMIS_INSTANCE_URL' before starting prism.";
        assert_eq!(
            display, expected,
            "E-SPEC-024 Display must match error-taxonomy.md v1.56 template byte-for-byte \
             (POL-24, no 'E-SPEC-024: ' prefix). Got:\n  {display:?}\nExpected:\n  {expected:?}"
        );
    }

    /// F-LP2-LOW-001: `AuthAcquisitionFailed` variant is constructible and its
    /// Display output includes sensor_id and client_id for operator diagnostics.
    #[test]
    fn test_auth_acquisition_failed_error_constructs() {
        let err = SpecEngineError::AuthAcquisitionFailed {
            sensor_id: "crowdstrike".to_string(),
            client_id: "test-org".to_string(),
            detail: "credentials store unavailable".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("crowdstrike"),
            "error message must contain sensor_id; got: {msg}"
        );
        assert!(
            msg.contains("test-org"),
            "error message must contain client_id; got: {msg}"
        );
        assert!(
            msg.contains("credentials store unavailable"),
            "error message must contain detail; got: {msg}"
        );
    }
}
