//! PrismError — canonical error taxonomy for the entire Prism platform.
//!
//! Every variant's Display output MUST begin with its structured error code token,
//! e.g. `"E-STORE-001: ..."`. Callers rely on the prefix for structured logging
//! and metric tagging.
//!
//! `PluginError` carries E-PLUGIN-* error codes from the WASM plugin runtime (S-1.15).

use thiserror::Error;

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

    /// E-SENSOR-010: Unknown sensor type.
    #[error("E-SENSOR-010: unknown sensor type: {name}")]
    UnknownSensorType { name: String },

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
    /// Returned by `ClarotyMapper` and `ArmisMapper` when the record_type is not one
    /// of their declared supported types. (BC-2.02.005, BC-2.02.006, S-1.05 Edge Cases)
    #[error("E-OCSF-023: unknown record type for sensor={sensor}: record_type={record_type}")]
    OcsfUnknownRecordType { sensor: String, record_type: String },

    /// E-OCSF-024: Timestamp field could not be parsed using any supported format.
    ///
    /// Returned by `CyberintMapper` when `created_date` fails all four parse attempts.
    /// (BC-2.02.004, AC-4, S-1.05 Edge Cases)
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
    #[error("E-CRED-003: credential access denied for {name} — credential values never transit AI context")]
    CredentialAccessDenied { name: String },

    /// E-CRED-004: Backend-level credential store failure (S-1.06).
    #[error("E-CRED-004: credential store error (backend={backend}): {reason}")]
    CredentialStoreError { backend: String, reason: String },

    /// E-CRED-005: Credential encryption or decryption failure (S-1.06).
    #[error("E-CRED-005: credential encryption error: {reason}")]
    CredentialEncryptionError { reason: String },

    /// E-CRED-006: Encryption passphrase not configured (S-1.06).
    #[error("E-CRED-006: encryption key not configured: {reason}")]
    EncryptionKeyMissing { reason: String },

    /// E-CRED-010: Keyring backend error.
    #[error("E-CRED-010: keyring error: {detail}")]
    KeyringError { detail: String },

    // -------------------------------------------------------------------------
    // E-IO — I/O errors
    // -------------------------------------------------------------------------
    /// E-IO-001: I/O error (S-1.06). String-ified so PrismError remains PartialEq+Eq.
    #[error("E-IO-001: I/O error: {0}")]
    Io(String),

    // -------------------------------------------------------------------------
    // E-FLAG — Feature flag / capability errors (BC-2.04.015, E-FLAG-001)
    // -------------------------------------------------------------------------
    /// E-FLAG-001 (CAPABILITY_DENIED): Write capability is denied — structured
    /// error for BC-2.04.015.  The `resolution_trace` is a BTreeMap-derived
    /// ordered list of path→effect pairs showing how the denial was reached.
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

    /// E-FLAG-002: Feature flag disabled — write operation blocked.
    #[error("E-FLAG-002: feature flag {flag} is disabled; write operations are locked")]
    FeatureFlagDisabled { flag: String },

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
    /// E-CFG-001: Config file not found.
    #[error("E-CFG-001: config file not found: {path}")]
    ConfigNotFound { path: String },

    /// E-CFG-002: Config parse error.
    #[error("E-CFG-002: config parse error: {detail}")]
    ConfigParseFailed { detail: String },

    /// E-CFG-003: Config validation error.
    #[error("E-CFG-003: config validation failed: {detail}")]
    ConfigValidationFailed { detail: String },

    /// E-CFG-010: Config snapshot stale.
    #[error("E-CFG-010: config snapshot stale: version {current} < required {required}")]
    ConfigSnapshotStale { current: u64, required: u64 },

    /// E-CFG-020: Capability path validation failed.
    ///
    /// Returned by `CapabilityPath::new()` when the input string violates any
    /// of the format rules: empty string, empty segment, invalid characters,
    /// more than 8 segments, or total length > 256 characters.
    #[error("E-CFG-020: invalid capability path: {reason}")]
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
    #[error("E-QUERY-001: query parse error at offset {offset}: {detail}")]
    QueryParseFailed { offset: usize, detail: String },

    /// E-QUERY-002: Query planning failed.
    #[error("E-QUERY-002: query planning failed: {detail}")]
    QueryPlanFailed { detail: String },

    /// E-QUERY-003: Query execution error.
    #[error("E-QUERY-003: query execution error: {detail}")]
    QueryExecutionFailed { detail: String },

    /// E-QUERY-004: Memory budget exceeded.
    #[error("E-QUERY-004: query memory budget exceeded: limit {limit_mb}MB, used {used_mb}MB")]
    QueryMemoryBudgetExceeded { limit_mb: u64, used_mb: u64 },

    /// E-QUERY-005: Query timeout.
    #[error("E-QUERY-005: query timed out after {elapsed_ms}ms")]
    QueryTimeout { elapsed_ms: u64 },

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

    /// E-QUERY-006: Requested limit exceeds the maximum allowed value (BC-2.11.001).
    ///
    /// Returned when `QueryOptions.limit > 1000`. Semantically distinct from
    /// `QueryExecutionFailed` (E-QUERY-003) — this is a pre-execution parameter
    /// validation error, not a runtime execution error.
    #[error("E-QUERY-001: limit {requested} exceeds maximum of {max} (BC-2.11.001)")]
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
    /// Distinct from E-QUERY-004 (query memory budget exceeded) and E-QUERY-005
    /// (query execution timeout) — this error specifically signals that a previously
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

    /// E-WATCHDOG-001 (query kill): Watchdog killed the running query because process RSS
    /// exceeded the Kill threshold (95% of 512 MB budget) on two consecutive checks
    /// (BC-2.15.007, VP-058).
    #[error(
        "E-WATCHDOG-001: watchdog killed query — process RSS exceeded kill threshold \
         ({budget_bytes} bytes budget); query token cancelled"
    )]
    WatchdogKilled {
        /// Configured memory budget in bytes (default 512 MiB).
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
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InfusionError {
    /// E-INFUSE-001: Unknown infusion name referenced in a query or pipe stage.
    #[error(
        "E-INFUSE-001: Unknown infusion '{name}'. Run list_infusions to see available enrichments."
    )]
    UnknownInfusion { name: String },

    /// E-INFUSE-002: Duplicate UDF name across multiple infusion specs.
    #[error("E-INFUSE-002: Duplicate UDF name '{udf_name}' in '{path2}' — already registered from '{path1}'.")]
    DuplicateUdfName {
        udf_name: String,
        path1: String,
        path2: String,
    },

    /// E-INFUSE-003: Missing required field in infusion spec.
    #[error("E-INFUSE-003: Missing required field '{field}' in infusion spec '{spec_path}'.")]
    MissingRequiredField { field: String, spec_path: String },

    /// E-INFUSE-004: Unknown source type in infusion spec.
    #[error("E-INFUSE-004: Unknown source type '{type_name}'. Valid types: maxmind_mmdb, csv, json_lookup, plugin.")]
    UnknownSourceType { type_name: String },

    /// E-INFUSE-005: Credential cannot be resolved.
    /// NOTE: The message MUST NOT include the credential value — only the field name,
    /// infusion_id, and env_var_name are safe to log (BC-2.19.005).
    #[error("E-INFUSE-005: Credential '{field_name}' for infusion '{infusion_id}' could not be resolved. Ensure '{env_var_name}' is set.")]
    CredentialUnresolved {
        field_name: String,
        infusion_id: String,
        env_var_name: String,
    },

    /// E-RULE-012: Detection rule filter references an API-backed infusion UDF.
    #[error("E-RULE-012: Detection rule filter references API-backed infusion UDF '{udf_name}' (from infusion '{infusion_id}', type 'plugin'). API-backed infusions cannot be used in detection rules — use a local_lookup infusion instead.")]
    ApiBackedUdfInDetectionRule {
        udf_name: String,
        infusion_id: String,
    },
}

// ---------------------------------------------------------------------------
// E-PLUGIN — WASM Plugin Runtime error types (S-1.15)
// ---------------------------------------------------------------------------

/// E-PLUGIN-* error codes from BC-2.17.001 through BC-2.17.006 (S-1.15).
///
/// These variants are returned at the `instance.call_*` boundary in `prism-spec-engine`
/// and MUST NOT propagate as panics into the host tokio runtime. All `PluginError`
/// variants correspond to sandbox isolation, resource enforcement, or contract
/// validation failures — the host process continues executing normally after any
/// `PluginError` is returned.
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
}
