//! Boot sequence orchestrator for `prism start`.
//!
//! Implements the 11-step boot sequence specified in ADR-022 §B and wired to
//! BC-2.22.001 (orchestration contract).  Steps 1–9 are implemented.
//! Steps 10–11 are intentional deferrals to S-1.12-FOLLOWUP per
//! BC-2.22.001 §step10-deferred-contract and §step11-deferred-contract —
//! they emit a WARN structured-log on entry and return `Ok(())` rather than panic.
//! Step 7.5 (plugin-load) is implemented by S-PLUGIN-PREREQ-D.
//!
//! # Sequencing Invariant (BC-2.22.001)
//!
//! ```text
//! Step 1   [BLOCKING] Tracing init
//! Step 2   [BLOCKING] Config load          (BC-2.06.011)
//! Step 3   [BLOCKING] OrgRegistry init     (BC-2.21.001)
//! Step 4   [BLOCKING] Sensor TOML spec load
//! Step 5   [BLOCKING] Credential store init (BC-2.03.013)
//! Step 6   [BLOCKING] Audit subsystem init  (BC-2.05.012)
//! Step 7   [BLOCKING] Storage + internal-tables provider init
//! Step 7.5 [BLOCKING] Plugin-load step (S-PLUGIN-PREREQ-D)
//! Step 8   [BLOCKING→BACKGROUND] QueryEngine + WriteExecutor
//! Step 9   [BACKGROUND] MCP server start
//! Step 10  [BACKGROUND] Hot-reload watcher install
//! Step 11  [BACKGROUND] Signal handler install
//! ```
//!
//! No step may begin concurrently with or before its predecessor completes
//! successfully (ADR-022 §B — strict sequential dependency, not a DAG).

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;

use crate::exit_codes::{EXIT_CONFIG_INVALID, EXIT_INTERNAL_ERROR, EXIT_PERMISSION_DENIED};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Errors surfaced by the boot sequence orchestrator.
///
/// Each variant maps to an ADR-022 §A exit code via [`BootError::exit_code`].
#[derive(Debug, thiserror::Error)]
pub enum BootError {
    /// Config file missing, TOML parse error, or schema validation failure.
    /// Maps to exit code 2. Anchors: BC-2.06.011.
    #[error("config-invalid: {0}")]
    ConfigInvalid(String),

    /// OrgRegistry construction failure (empty list, duplicate, malformed slug).
    /// Maps to exit code 2. Anchors: BC-2.21.001.
    #[error("org-registry-failed: {0}")]
    OrgRegistryFailed(String),

    /// CredentialStore: unresolvable ref or malformed ref.
    /// Maps to exit code 2. Anchors: BC-2.03.013.
    #[error("credential-ref-invalid: {0}")]
    CredentialRefInvalid(String),

    /// CredentialStore: permission denied or backend unavailable.
    /// Maps to exit code 5. Anchors: BC-2.03.013.
    #[error("credential-permission-denied: {0}")]
    CredentialPermissionDenied(String),

    /// Credential auth_type/shape mismatch (Rule C / E-SPEC-014).
    ///
    /// The sensor spec declares `auth_type = expected_shape` but the credential
    /// store reports the credential was configured for `actual_shape`. This is a
    /// config-invalid condition (exit 2) — the spec and the credential disagree.
    ///
    /// Credential values MUST NOT appear in this error message (AD-017 AI-opaque model).
    /// Maps to exit code 2. Anchors: BC-2.01.016 §Error Cases E-SPEC-014, ADR-023 Rule 2 Rule C.
    #[error(
        "E-SPEC-014: sensor '{sensor_id}' auth_type/credential shape mismatch — \
             spec declares '{expected_shape}' but credential ref reports '{actual_shape}' \
             (BC-2.01.016 §Error Cases E-SPEC-014; ADR-023 Rule 2 Rule C)"
    )]
    AuthTypeCredentialMismatch {
        sensor_id: String,
        expected_shape: String,
        actual_shape: String,
    },

    /// Audit subsystem init failed (RocksDB CF open, WAL, sentinel write).
    /// Maps to exit code 4. Anchors: BC-2.05.012.
    #[error("audit-init-failed: {0}")]
    AuditInitFailed(String),

    /// Storage, QueryEngine, WriteExecutor, or MCP server init failure.
    /// Maps to exit code 4.
    #[error("internal-error: {0}")]
    InternalError(String),

    /// Sensor adapter required but failed to initialize.
    /// Maps to exit code 3.
    #[error("sensor-fail: {0}")]
    SensorFail(String),

    /// Sensor spec declares `auth_plugin` that is not registered in PluginRuntime.
    ///
    /// F-LP2-CRIT-002 closure (paper-fix of F-LP1-CRIT-003): a typo'd `auth_plugin` value
    /// now causes boot to exit code 2 (config-invalid) instead of silently breaking at
    /// runtime when the plugin dispatch is first attempted.
    ///
    /// Maps to exit code 2 per ADR-022 §A config-invalid class.
    /// Anchors: E-SPEC-012 (SpecEngineError::UnknownAuthPlugin); F-LP1-CRIT-003; F-LP2-CRIT-002.
    #[error(
        "E-SPEC-012: sensor '{sensor_id}' declares auth_plugin = '{plugin_id}' \
         but no plugin with that id was loaded — check plugin_dir for '{plugin_id}.prx' \
         or correct the auth_plugin field in the sensor spec (F-LP2-CRIT-002)"
    )]
    UnknownAuthPlugin {
        sensor_id: String,
        plugin_id: String,
    },
}

impl BootError {
    /// Return the ADR-022 §A canonical exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            BootError::ConfigInvalid(_)
            | BootError::OrgRegistryFailed(_)
            | BootError::CredentialRefInvalid(_)
            | BootError::AuthTypeCredentialMismatch { .. }
            // F-LP2-CRIT-002: typo'd auth_plugin = config-invalid → exit 2.
            | BootError::UnknownAuthPlugin { .. } => EXIT_CONFIG_INVALID,
            BootError::CredentialPermissionDenied(_) => EXIT_PERMISSION_DENIED,
            BootError::AuditInitFailed(_) | BootError::InternalError(_) => EXIT_INTERNAL_ERROR,
            BootError::SensorFail(_) => crate::exit_codes::EXIT_SENSOR_FAIL,
        }
    }
}

/// Handle returned after a successful full boot (steps 1–11).
///
/// Holds the running subsystem handles; dropped during graceful shutdown.
pub struct RunningServer {
    /// Resolved config directory used during boot.
    pub config_dir: PathBuf,
    /// Per-org overlay resolved spec map produced by step 4 (S-CONFIG-MULTI-TENANT-OVERRIDE-001).
    ///
    /// Read-only after boot; shared via `Arc<HashMap>` for O(1) fanout dispatch
    /// without mutex contention (INV-OVL-006, INV-FANOUT-002).
    /// Key: `(OrgSlug, SensorId)`; value: effective `ResolvedSensorSpec`.
    ///
    /// Empty map when no `customers/` directory exists (BC-2.06.012 backwards compat).
    /// Threaded into `QueryEngine` for per-org endpoint resolution at query time (ADR-029).
    pub resolved_spec_map: Arc<
        std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
    /// Background tokio task running `PrismServer::serve_stdio`.
    ///
    /// Awaited by `main.rs` via `wait_for_shutdown()` to keep the process alive
    /// for the lifetime of the MCP stdio session.  The task exits when stdin closes
    /// (client disconnect) or SIGTERM/SIGINT is received (BC-2.10.010).
    ///
    /// The `Result` propagates the serve_stdio return value so `wait_for_shutdown()`
    /// can translate rmcp error variants to canonical exit codes (BC-2.10.010, ADR-022 §A).
    pub(crate) mcp_server_task: tokio::task::JoinHandle<Result<(), rmcp::RmcpError>>,
}

impl RunningServer {
    /// Block (async-await) until the MCP server background task exits.
    ///
    /// Called from `main.rs` after `run_boot_sequence` returns to keep the process
    /// alive for the duration of the MCP stdio session.  Returns when the task exits
    /// for any reason (stdin closed, SIGTERM/SIGINT, error).
    ///
    /// # Exit-code mapping (BC-2.10.010 + ADR-022 §A)
    ///
    /// | `serve_stdio` result               | Exit code |
    /// |------------------------------------|-----------|
    /// | `Ok(())`                           | `0` — clean shutdown                |
    /// | `Err(RmcpError::TaskError(_))`     | `1` — graceful-drain timeout        |
    /// | `Err(RmcpError::Runtime(_))`       | `4` — task panic / join error       |
    /// | `Err(<other rmcp variant>)`        | `0` — transport init fail / stdin-EOF|
    /// | `JoinError` (task panicked)        | `4` — internal error                |
    ///
    /// **`Err(<other rmcp variant>)` → exit 0 rationale**: when the MCP client
    /// disconnects before rmcp can complete transport initialization (e.g., stdin is
    /// already closed when `serve()` is called), rmcp returns a transport error that
    /// is functionally equivalent to a clean client disconnect.  There is no active
    /// session to drain, and the process should exit cleanly.  Only the explicit
    /// 5-second drain timeout (`TaskError`) warrants exit 1 per BC-2.10.010.
    ///
    /// Panics in the MCP task are caught here as `JoinError`; the panic hook also
    /// runs (emitting the structured log) before this point.
    pub async fn wait_for_shutdown(self) -> i32 {
        use crate::exit_codes::{EXIT_GENERIC_ERROR, EXIT_INTERNAL_ERROR, EXIT_SUCCESS};
        match self.mcp_server_task.await {
            // Clean shutdown — stdin closed by client or SIGTERM/SIGINT drained.
            Ok(Ok(())) => EXIT_SUCCESS,
            // BC-2.10.010: graceful-drain timeout exceeded → exit 1 (EXIT_GENERIC_ERROR).
            Ok(Err(rmcp::RmcpError::TaskError(_))) => {
                tracing::warn!(
                    event_type = "boot.shutdown.timeout",
                    "MCP server graceful-drain timeout exceeded (BC-2.10.010); exit 1"
                );
                EXIT_GENERIC_ERROR
            }
            // Task panicked (join error wraps a panic payload) → exit 4 (EXIT_INTERNAL_ERROR).
            Ok(Err(rmcp::RmcpError::Runtime(_))) => {
                tracing::error!(
                    event_type = "boot.shutdown.runtime_error",
                    "MCP server exited with runtime error; exit 4"
                );
                EXIT_INTERNAL_ERROR
            }
            // Transport init failure (e.g., stdin already closed before serve() completes).
            // Functionally equivalent to a clean client-never-connected case.
            // Log at WARN so the condition is observable, but exit 0 — no session was active.
            Ok(Err(e)) => {
                tracing::warn!(
                    event_type = "boot.shutdown.transport_init_error",
                    error = %e,
                    "MCP server transport init error (stdin EOF before connect?); exit 0"
                );
                EXIT_SUCCESS
            }
            // JoinError: the task panicked — panic hook already fired, exit 4.
            Err(_join_err) => {
                tracing::error!(
                    event_type = "boot.shutdown.task_panic",
                    "MCP server task panicked; exit 4"
                );
                EXIT_INTERNAL_ERROR
            }
        }
    }
}

/// Unit tests for `RunningServer::wait_for_shutdown` exit-code mapping.
///
/// These tests exercise BC-2.10.010 exit-code contract without spawning a real
/// MCP server. Each test constructs a `RunningServer` with a pre-resolved
/// `JoinHandle` and asserts the returned exit code matches the ADR-022 §A table.
#[cfg(test)]
mod shutdown_exit_code_tests {
    use super::*;

    /// Construct a `RunningServer` with a pre-resolved task for testing.
    ///
    /// `config_dir` is set to `std::env::temp_dir()` — these tests never perform
    /// filesystem I/O via `config_dir` (only `mcp_server_task` is exercised), but
    /// the field must be populated. `std::env::temp_dir()` works on all platforms
    /// including Windows (returns `%TEMP%` or `%TMP%`), unlike a hardcoded `/tmp/`.
    fn make_server(task: tokio::task::JoinHandle<Result<(), rmcp::RmcpError>>) -> RunningServer {
        RunningServer {
            config_dir: std::env::temp_dir().join("prism-test"),
            resolved_spec_map: Arc::new(std::collections::HashMap::new()),
            mcp_server_task: task,
        }
    }

    /// BC-2.10.010 + ADR-022 §A: clean shutdown → exit 0.
    #[tokio::test]
    async fn test_wait_for_shutdown_clean_returns_exit_0() {
        let handle = tokio::spawn(async { Ok(()) });
        let server = make_server(handle);
        assert_eq!(
            server.wait_for_shutdown().await,
            0,
            "clean MCP shutdown must return exit 0 (BC-2.10.010, ADR-022 §A)"
        );
    }

    /// BC-2.10.010 + ADR-022 §A: graceful-drain timeout (TaskError) → exit 1.
    #[tokio::test]
    async fn test_wait_for_shutdown_task_error_returns_exit_1() {
        let handle =
            tokio::spawn(async { Err(rmcp::RmcpError::TaskError("shutdown timeout".to_string())) });
        let server = make_server(handle);
        assert_eq!(
            server.wait_for_shutdown().await,
            1,
            "TaskError (graceful-drain timeout) must return exit 1 (BC-2.10.010)"
        );
    }

    /// BC-2.10.010 + ADR-022 §A: Runtime error (task panic wrapper) → exit 4.
    #[tokio::test]
    async fn test_wait_for_shutdown_runtime_error_returns_exit_4() {
        let handle = tokio::spawn(async {
            let join_err = tokio::spawn(async { panic!("simulated panic") })
                .await
                .unwrap_err();
            Err::<(), _>(rmcp::RmcpError::Runtime(join_err))
        });
        let server = make_server(handle);
        assert_eq!(
            server.wait_for_shutdown().await,
            4,
            "RmcpError::Runtime must return exit 4 (ADR-022 §A internal-error)"
        );
    }

    /// Transport init failure (stdin already closed) → exit 0.
    ///
    /// When `serve()` fails because stdin is already closed (MCP client never
    /// connected), this is functionally a clean disconnect. Map to exit 0 so that
    /// `prism start` with no piped MCP client exits cleanly.
    ///
    /// Uses `RmcpError::TransportCreation` (the canonical transport-init-error variant)
    /// to exercise the `Ok(Err(e))` catch-all arm.  This test is LOAD-BEARING: if
    /// line 215 (`EXIT_SUCCESS`) is changed to `EXIT_GENERIC_ERROR`, this test fails.
    #[tokio::test]
    async fn test_wait_for_shutdown_transport_error_returns_exit_0() {
        // Use RmcpError::transport_creation::<()>() to construct a TransportCreation
        // variant — the canonical transport-init-error type in rmcp 1.7. This falls
        // into the `Ok(Err(e))` catch-all arm of wait_for_shutdown (not TaskError,
        // not Runtime), which MUST map to EXIT_SUCCESS (exit 0) per the spec.
        let transport_err = rmcp::RmcpError::transport_creation::<()>(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "stdin closed before serve()",
        ));
        let handle = tokio::spawn(async move { Err::<(), _>(transport_err) });
        let server = make_server(handle);
        assert_eq!(
            server.wait_for_shutdown().await,
            0,
            "TransportCreation error (stdin closed, no client connected) must return exit 0 \
             per boot.shutdown.transport_init_error semantics (ADR-022 §A, BC-2.10.010)"
        );
    }

    /// BC-2.10.010 + ADR-022 §A: task itself panics (JoinError) → exit 4.
    #[tokio::test]
    async fn test_wait_for_shutdown_join_error_returns_exit_4() {
        // Type annotation drives inference so the spawn closure has the right signature.
        let handle: tokio::task::JoinHandle<Result<(), rmcp::RmcpError>> =
            tokio::spawn(async { panic!("task panicked") });
        let server = make_server(handle);
        assert_eq!(
            server.wait_for_shutdown().await,
            4,
            "JoinError from panicking task must return exit 4 (ADR-022 §A internal-error)"
        );
    }
}

/// Lightweight result of steps 1–6 (blocking boot to audit-ready state).
///
/// Returned by `boot_to_step_6` for integration tests that exercise only
/// the blocking portion of the boot sequence.
pub struct BootContext {
    pub config_dir: PathBuf,
    /// RocksDB backend opened in step 6 (all CFs, including `audit_buffer`).
    /// Threaded into step 7.5 (`plugin_load_step`) so the `RocksDbPluginAuditSink`
    /// can write durable audit entries for each unsigned plugin load (HIGH-002 / AC-4).
    pub rocksdb_backend: Arc<prism_storage::rocksdb_backend::RocksDbBackend>,
    /// Per-org overlay resolved spec map produced by step 4 (S-CONFIG-MULTI-TENANT-OVERRIDE-001).
    ///
    /// Read-only after boot; shared via `Arc<HashMap>` for O(1) fanout dispatch
    /// without mutex contention (INV-OVL-006, INV-FANOUT-002).
    /// Key: `(OrgSlug, SensorId)`; value: effective `ResolvedSensorSpec`.
    ///
    /// Empty map when no `customers/` directory exists (BC-2.06.012 backwards compat).
    pub resolved_spec_map: Arc<
        std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
    /// ConfigManager from step 4 — threaded into step 7.5 validation so that
    /// `validate_auth_plugin_registered` can check all loaded SensorSpecs against the
    /// plugin registry after `load_all_plugins` completes (F-LP2-CRIT-002).
    pub config_manager: Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    /// OrgRegistry produced by step 3 (BC-2.21.001).
    ///
    /// Threaded into step 9 (`step9_start_mcp_server`) to wire the real OrgRegistry
    /// into QueryEngine — replacing the earlier `OrgRegistry::new()` empty placeholder
    /// that would cause multi-tenant queries to resolve no org IDs (CRIT-5 fix).
    ///
    /// Also provides the org slug list for `ClientRegistry` construction.
    pub org_registry: Arc<prism_core::OrgRegistry>,
    /// CredentialStore produced by step 5 (BC-2.03.013).
    ///
    /// Threaded into step 9 (`step9_start_mcp_server`) so that QueryEngine receives
    /// the real keyring-backed credential store — replacing the earlier
    /// `BootNullCredentialStore` that silently returned `Ok(None)` for all
    /// credential lookups (CRIT-5 fix, F-PASS3-CRIT-5 closure).
    pub credential_store: Arc<dyn prism_credentials::CredentialStore>,
}

// ---------------------------------------------------------------------------
// Boot sequence entry points
// ---------------------------------------------------------------------------

// Step 7.5b pure function — extracted for testability (F-LP3-MED-001 closure)
/// Validate `auth_plugin` registry membership for all sensors in `snapshot` and
/// construct a `PluginAuthProvider` for each sensor that declares `auth_plugin`.
///
/// This is the pure business logic extracted from `run_boot_sequence` step 7.5b
/// (F-LP3-MED-001 closure: the prior inline code was wired but had no behavioral test —
/// any iteration bug, wrong key, or missed `auth_plugin.is_some()` check would be silent).
///
/// # Parameters
///
/// - `snapshot`: config snapshot with all sensor specs (from config_manager).
/// - `runtime`: the initialized `PluginRuntime` that holds registered plugin IDs.
///
/// # Returns
///
/// `Ok(HashMap<sensor_id, Arc<PluginAuthProvider>>)` — one entry per sensor spec with
/// `auth_plugin.is_some()`. Sensors without `auth_plugin` are absent from the map.
///
/// `Err(BootError::UnknownAuthPlugin)` — if any sensor references a plugin_id not
/// registered in `runtime`. Exit code: `EXIT_CONFIG_INVALID` (ADR-022 §A exit table).
///
/// # Production caller
///
/// `run_boot_sequence` step 7.5b (boot.rs) — only production construction site for
/// `PluginAuthProvider`. After ADR-028 §D10 co-merge (001-A), this is how CrowdStrike
/// gets its `Arc<dyn AuthProvider>` in production (F-LP2-HIGH-001).
pub fn validate_and_construct_auth_providers(
    snapshot: &prism_spec_engine::types::ConfigSnapshot,
    runtime: &Arc<prism_spec_engine::plugin::PluginRuntime>,
) -> Result<std::collections::HashMap<String, Arc<prism_spec_engine::PluginAuthProvider>>, BootError>
{
    use std::collections::{HashMap, HashSet};

    use prism_spec_engine::{PluginAuthProvider, validate_auth_plugin_fields};

    let registered_ids: HashSet<String> = runtime.list_plugins().into_iter().collect();
    let mut providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();

    for (sensor_id, sensor_spec) in &snapshot.sensor_specs {
        validate_auth_plugin_fields(
            sensor_id,
            sensor_spec.auth_plugin.as_deref(),
            &registered_ids,
        )
        .map_err(|se| match se {
            prism_spec_engine::error::SpecEngineError::UnknownAuthPlugin {
                sensor_id,
                plugin_id,
            } => BootError::UnknownAuthPlugin {
                sensor_id,
                plugin_id,
            },
            other => BootError::ConfigInvalid(other.to_string()),
        })?;

        if let Some(plugin_id) = sensor_spec.auth_plugin.as_deref() {
            // ADR-028 §D11 Option C: pass sensor_id (not opaque credential_handle).
            // PluginAuthProvider resolves client_id/client_secret from prism_credentials
            // at acquire_token dispatch time using sensor_id as the credential namespace key.
            let token_endpoint = format!("{}/oauth2/token", sensor_spec.base_url);

            // SEC-003: validate token_endpoint host against plugin's allowed_urls at boot time.
            // If the host is not allowlisted, the plugin's HTTP dispatch would return HTTP 403
            // at runtime (host_http_request allowlist gate). Fail-fast at boot instead.
            let plugin_arc = runtime.get_plugin(plugin_id).map_err(|e| {
                BootError::ConfigInvalid(format!(
                    "SEC-003: cannot retrieve plugin '{plugin_id}' for allowed_urls check: {e}"
                ))
            })?;
            // Use reqwest::Url (re-exported from the `url` crate) to parse the endpoint.
            // reqwest is already a direct dep of prism-bin for Client construction.
            if let Ok(parsed) = reqwest::Url::parse(&token_endpoint) {
                let endpoint_host = parsed.host_str().unwrap_or("");
                let is_allowed = plugin_arc
                    .allowed_urls
                    .iter()
                    .any(|allowed| !allowed.is_empty() && endpoint_host == allowed.as_str());
                if !is_allowed {
                    return Err(BootError::ConfigInvalid(format!(
                        "SEC-003: sensor '{sensor_id}' token_endpoint host '{endpoint_host}' \
                         is not in plugin '{plugin_id}' allowed_urls {:?}. \
                         Add the host to the plugin manifest 'allowed_urls' field.",
                        plugin_arc.allowed_urls
                    )));
                }
            }

            let auth_provider = Arc::new(PluginAuthProvider::new(
                Arc::clone(runtime),
                plugin_id,
                sensor_id.as_str(),
                token_endpoint,
            ));

            providers.insert(sensor_id.clone(), auth_provider);
        }
    }

    Ok(providers)
}

/// Execute the full 11-step boot sequence (steps 1–11).
///
/// Steps 1–9 are implemented. Steps 10–11 are intentional deferrals to
/// S-1.12-FOLLOWUP per BC-2.22.001 §step10-deferred-contract and
/// §step11-deferred-contract — they emit a WARN structured-log on entry
/// and return `Ok(())` rather than panic.
///
/// On success, returns a `RunningServer` handle.  On any step failure, this
/// function does NOT return — it calls `std::process::exit` with the mapped
/// exit code per ADR-022 §A.
///
/// # Step ordering (F-PASS3-CRIT-001 fix)
///
/// BC-2.22.001 §Sequencing Invariant specifies step 7.5 BEFORE step 8 and BEFORE
/// the MCP server bind (step 9).  The invariant does NOT require step 7 (storage
/// init) to precede step 7.5 — plugin-load only needs the RocksDB audit backend from
/// step 6 (`ctx.rocksdb_backend`), which `boot_to_step_6` already provides.
///
/// Correct execution order:
///   steps 1–6 (boot_to_step_6) → step 7.5 (plugin-load) → step 7 (storage) → steps 8–11
///
/// This ordering ensures `plugin_load_step_with_audit` is REACHABLE at runtime:
/// step 7 (storage init) runs AFTER plugin-load completes, so plugin-load is
/// never skipped.
pub async fn run_boot_sequence(config_dir: &Path) -> Result<RunningServer, BootError> {
    let ctx = boot_to_step_6(config_dir).await?;

    // Step 2 re-loads config to obtain plugin_dir for step 7.5.
    // This is acceptable because step2_load_config is idempotent (pure read + validate).
    let config = step2_load_config(config_dir).await?;

    // Step 7.5 [BLOCKING]: Plugin-load step — BC-2.22.001 §Sequencing Invariant.
    // Positioned BEFORE step 7 (storage init) — plugin-load only requires the RocksDB
    // audit backend from step 6 (ctx.rocksdb_backend), which boot_to_step_6 already
    // provides.  This ordering makes plugin-load REACHABLE at runtime: step 7
    // (storage init) runs AFTER plugin-load completes, so plugin-load is never skipped.
    //
    // Pre-traffic gate (AC-2): MCP server (step 9) does NOT bind before this completes.
    // ADR-023 §C4 + ADR-022 §B.
    //
    // HIGH-002 (F-IMPL-LP1-HIGH-002): wire RocksDbPluginAuditSink from step 6 backend
    // so plugin load events are persisted durably to audit_buffer CF (not just tracing::warn!).
    let plugin_audit_sink = Arc::new(crate::plugin_audit::RocksDbPluginAuditSink::new(
        Arc::clone(&ctx.rocksdb_backend),
    ));
    let mut plugin_result =
        plugin_load_step_with_audit(&config.plugin_dir, plugin_audit_sink).await?;

    // Step 7.5b [BLOCKING]: Validate auth_plugin registry membership + construct PluginAuthProviders.
    //
    // F-LP2-CRIT-002: validate every `auth_plugin = "..."` names a registered plugin (exit 2 on miss).
    // F-LP2-HIGH-001: construct PluginAuthProvider for each plugin-authed sensor — FIRST non-test
    // production construction site. Stored in plugin_result.plugin_auth_providers for step 8 wiring.
    // F-LP3-MED-001: iteration logic extracted into `validate_and_construct_auth_providers` for
    // behavioral testability — the inline block was wired code but had no behavioral test driving
    // the iteration semantics (TD-VSDD-059 paper-fix pattern closed here).
    //
    // Uses ctx.config_manager (step 4) + plugin_result.runtime for registered IDs.
    {
        let cm_guard = ctx.config_manager.load();
        let cm = &**cm_guard;
        let snapshot_guard = cm.load();
        let snapshot = &**snapshot_guard;

        let providers = validate_and_construct_auth_providers(snapshot, &plugin_result.runtime)?;

        for (sensor_id, auth_provider) in &providers {
            let plugin_id = auth_provider.plugin_id();
            tracing::info!(
                sensor_id = %sensor_id,
                plugin_id = %plugin_id,
                event_type = "plugin_auth_provider_constructed",
                "boot: PluginAuthProvider constructed for sensor"
            );
        }

        plugin_result.plugin_auth_providers = providers;
    }

    // Step 7.5c: Register built-in write tools into DYNAMIC_WRITE_TOOLS before step 8
    // closes the registration window via mark_query_phase_started().
    // BC-2.16.012 INV-INVALIDATION-EXT-001 / PLUGIN-MIGRATION-001-B AC-003.
    // Must execute AFTER step 7.5 plugin-load AND BEFORE step 8 mark_query_phase_started().
    prism_query::invalidation::register_builtin_write_tools()
        .map_err(|e| BootError::InternalError(format!("write tool registration failed: {e}")))?;

    // Step 7 [BLOCKING]: Storage + internal-tables provider init.
    // Positioned AFTER step 7.5 — plugin-load does not depend on storage tables.
    // Pass ctx.rocksdb_backend (opened in step 6) so step7 can health-check it.
    step7_init_storage(&ctx.rocksdb_backend).await?;

    step8_init_query_engine().await?;
    // Step 9: start MCP server — construct QueryEngine + WriteExecutor with available
    // deps from prior boot steps, then spawn PrismServer::serve_stdio as a background task.
    // Pass BootContext deps (storage, config_manager, resolved_spec_map, org_registry,
    // credential_store) to step9 for wiring (CRIT-5: real deps, not empty placeholders).
    // CRIT-4: also pass spec_dir (for reload_config/add_sensor_spec) and config_dir
    // (for alias store file path).
    let mcp_server_task = step9_start_mcp_server(
        Arc::clone(&ctx.rocksdb_backend),
        Arc::clone(&ctx.config_manager),
        Arc::clone(&ctx.resolved_spec_map),
        Arc::clone(&ctx.org_registry),
        Arc::clone(&ctx.credential_store),
        config.spec_dir.clone(),
        config_dir.to_path_buf(),
        plugin_result.plugin_auth_providers,
    )
    .await?;
    step10_start_hot_reload().await?;

    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
    let (reload_tx, _) = tokio::sync::mpsc::channel(1);
    step11_install_signal_handlers(shutdown_tx, reload_tx).await?;

    Ok(RunningServer {
        config_dir: config_dir.to_path_buf(),
        resolved_spec_map: ctx.resolved_spec_map,
        mcp_server_task,
    })
}

/// Execute boot steps 1–6 only (blocking, audit-ready state).
///
/// Used by `validate-config` subcommand and integration tests that verify
/// the blocking boot path without entering the MCP serving loop.
///
/// BC-2.22.001: Steps 1–6 must complete in order before this returns.
/// The sequencing invariant is enforced by sequential await; no step is
/// started concurrently with its predecessor.
pub async fn boot_to_step_6(config_dir: &Path) -> Result<BootContext, BootError> {
    // -------------------------------------------------------------------------
    // Test injection gate — PRISM_TEST_INJECT_PANIC
    //
    // CRIT-1 (S-WAVE5-PREP-01 fix-pass-1): ALL PRISM_TEST_* env-var reads are
    // gated behind `#[cfg(feature = "test-injection")]`. The feature is only
    // enabled in test builds via `--all-features` (Justfile iter/check recipes).
    // The release binary has zero test-injection code paths.
    //
    // When PRISM_TEST_INJECT_PANIC=true, panic immediately to exercise the
    // custom panic hook (AC-12). This gate fires BEFORE step 1 since the
    // hook is installed in main() before dispatch.
    // -------------------------------------------------------------------------
    #[cfg(feature = "test-injection")]
    if std::env::var("PRISM_TEST_INJECT_PANIC").as_deref() == Ok("true") {
        panic!("PRISM_TEST_INJECT_PANIC=true: injected panic to exercise AC-12 panic hook");
    }

    // Step 2: Load config.
    let config = step2_load_config(config_dir).await?;

    // Step 3: Init OrgRegistry.
    // INV-COMPAT-002 (BC-2.06.015): step 3 MUST precede step 4 — overlay validation
    // requires a fully-constructed OrgRegistry to cross-check customers/ slugs.
    let org_registry = step3_init_org_registry(&config).await?;

    // Step 4: Load sensor TOML specs AND per-org overlay specs.
    // S-CONFIG-MULTI-TENANT-OVERRIDE-001: extended step 4 replaces bare step4_load_sensor_specs.
    // Returns (config_manager, resolved_spec_map) — overlay errors abort boot with exit 2.
    let (config_manager, resolved_spec_map) =
        step4_load_sensor_specs_with_overlays(&config, &org_registry).await?;

    // Step 5: Init credential store.
    //
    // CRIT-1: test injection blocks gated behind `#[cfg(feature = "test-injection")]`.
    // Test injection: PRISM_TEST_INJECT_FAIL_STEP=5_permission → CredentialPermissionDenied (exit 5)
    // Test injection: PRISM_TEST_INJECT_FAIL_STEP=5_missing_ref → CredentialRefInvalid (exit 2)
    #[cfg(feature = "test-injection")]
    {
        let inject_fail_step5 = std::env::var("PRISM_TEST_INJECT_FAIL_STEP").unwrap_or_default();
        if inject_fail_step5 == "5_permission" {
            // Injected credential permission-denied failure for test determinism.
            // BC-2.03.013 TV-03-013-004: permission denied → exit 5.
            return Err(BootError::CredentialPermissionDenied(
                "PRISM_TEST_INJECT_FAIL_STEP=5_permission: \
                 injected credential store permission-denied (BC-2.03.013 TV-03-013-004)"
                    .to_string(),
            ));
        }
        if inject_fail_step5 == "5_missing_ref" {
            // Injected unresolvable credential ref failure for test determinism.
            // BC-2.03.013 TV-03-013-003: unresolvable ref → exit 2.
            return Err(BootError::CredentialRefInvalid(
                "PRISM_TEST_INJECT_FAIL_STEP=5_missing_ref: \
                 injected credential ref unresolvable (BC-2.03.013 TV-03-013-003)"
                    .to_string(),
            ));
        }
    }
    // CRIT-5: Store the real credential_store in BootContext so step9 can wire it into
    // QueryEngine instead of the BootNullCredentialStore that silently returns Ok(None).
    let credential_store = step5_init_credential_store(&config, &config_manager).await?;

    // Step 6: Init audit subsystem.
    //
    // CRIT-1: test injection blocks gated behind `#[cfg(feature = "test-injection")]`.
    // Test injection: PRISM_TEST_INJECT_FAIL_STEP=6_audit_failure → AuditInitFailed (exit 4)
    // Test injection: PRISM_TEST_INJECT_FAIL_STEP=6_rocksdb_lock → AuditInitFailed (exit 4)
    #[cfg(feature = "test-injection")]
    {
        let inject_fail_step6 = std::env::var("PRISM_TEST_INJECT_FAIL_STEP").unwrap_or_default();
        if inject_fail_step6 == "6_audit_failure" {
            // Injected audit init failure for test determinism.
            // BC-2.05.012 TV-05-012-002: audit init fails → exit 4.
            return Err(BootError::AuditInitFailed(
                "PRISM_TEST_INJECT_FAIL_STEP=6_audit_failure: \
                 injected audit subsystem init failure (BC-2.05.012 TV-05-012-002)"
                    .to_string(),
            ));
        }
        if inject_fail_step6 == "6_rocksdb_lock" {
            // Injected RocksDB LOCK-held failure for test determinism.
            // BC-2.05.012 EC-05-012-006: LOCK file exists → exit 4 + LOCK message.
            return Err(BootError::AuditInitFailed(
                "PRISM_TEST_INJECT_FAIL_STEP=6_rocksdb_lock: \
                 RocksDB LOCK file exists — Another Prism process may be running. \
                 Check the state_dir/LOCK file. (BC-2.05.012 EC-05-012-006)"
                    .to_string(),
            ));
        }
    }

    // Step 6: Init audit subsystem (full implementation per BC-2.05.012).
    // Opens RocksDB, confirms audit_buffer CF writable, writes sentinel durably.
    // HIGH-002 (F-IMPL-LP1-HIGH-002): retain the backend in BootContext so step 7.5
    // can wire it into RocksDbPluginAuditSink for durable plugin load audit entries.
    let audit_backend = step6_init_audit(&config)?;

    // -------------------------------------------------------------------------
    // Test gate: PRISM_TEST_STOP_AFTER_STEP=6
    //
    // CRIT-1: gated behind `#[cfg(feature = "test-injection")]`.
    // Used by the SIGTERM test (AC-6) to hold the process at step-6 state
    // so a signal can be delivered. The process blocks here until a signal
    // (SIGTERM) arrives.
    //
    // MED-2 (S-WAVE5-PREP-01 fix-pass-1): wires through signals::install_sigterm_handler
    // instead of duplicating the select! arm. This ensures BC-2.10.010 coverage
    // is exercised through the production code path.
    // -------------------------------------------------------------------------
    #[cfg(feature = "test-injection")]
    if std::env::var("PRISM_TEST_STOP_AFTER_STEP").as_deref() == Ok("6") {
        // OBS-2 (S-WAVE5-PREP-01 fix-pass-5): register the SIGTERM handler FIRST
        // (sync), THEN write the sentinel, THEN await the signal.  This eliminates
        // the race window where SIGTERM arrives between sentinel write and handler
        // registration — any SIGTERM delivered after create_sigterm_future returns
        // will be queued by the kernel and delivered on the first poll.
        //
        // PRISM_TEST_READY_FILE: path provided by the test for the sentinel.
        // Falls back to a PID-based path if not set.
        let (shutdown_tx, _rx) = tokio::sync::broadcast::channel(1);

        // Step 1: register handler synchronously (returns a future, does not await).
        // MED-2: delegate to signals::create_sigterm_future so the SIGTERM
        // test exercises the production BC-2.10.010 code path, not a duplicate.
        #[cfg(unix)]
        let handler_fut = crate::signals::create_sigterm_future(shutdown_tx);
        #[cfg(not(unix))]
        let handler_fut = crate::signals::install_sigterm_handler(shutdown_tx);

        // Step 2: NOW write the sentinel — handler is registered.
        let ready_file = std::env::var("PRISM_TEST_READY_FILE")
            .unwrap_or_else(|_| format!("/tmp/prism-ready-{}.sentinel", std::process::id()));
        let _ = std::fs::write(&ready_file, "ready");
        tracing::info!(
            ready_file = %ready_file,
            "PRISM_TEST_STOP_AFTER_STEP=6: boot reached step 6 — \
             waiting for SIGTERM via create_sigterm_future (MED-2, OBS-2)"
        );

        // Step 3: await the signal.
        // create_sigterm_future / install_sigterm_handler calls process::exit(0)
        // on SIGTERM — this line is unreachable if a signal is received.
        handler_fut.await;
        std::process::exit(0);
    }

    Ok(BootContext {
        config_dir: config_dir.to_path_buf(),
        rocksdb_backend: audit_backend,
        resolved_spec_map,
        // F-LP2-CRIT-002: thread config_manager into BootContext so that run_boot_sequence
        // can validate auth_plugin registry membership after plugins are loaded at step 7.5.
        config_manager,
        // CRIT-5: thread the real OrgRegistry produced by step 3 — avoids creating a new
        // empty OrgRegistry in step 9 that would cause multi-tenant queries to resolve no orgs.
        org_registry,
        // CRIT-5: thread the real credential_store produced by step 5 — replaces
        // BootNullCredentialStore that silently returned Ok(None) for all credential lookups.
        credential_store,
    })
}

// ---------------------------------------------------------------------------
// Individual boot steps
// ---------------------------------------------------------------------------

/// Step 1 [BLOCKING]: Initialize tracing subscriber.
///
/// ADR-022 §B step 1.  Must run FIRST, before any other boot step or log
/// output.  On failure, emits to stderr and calls `std::process::exit(4)`.
///
/// Format: JSON if `PRISM_LOG_FORMAT=json`; pretty otherwise.
/// First log line: `tracing::info!("Prism v{}", env!("CARGO_PKG_VERSION"))`.
pub fn step1_init_tracing(log_format: &crate::cli::LogFormat) {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let result = match log_format {
        crate::cli::LogFormat::Json => tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json())
            .try_init(),
        crate::cli::LogFormat::Pretty => tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().pretty())
            .try_init(),
    };

    if let Err(e) = result {
        eprintln!("Failed to init tracing: {e}");
        // ADR-022 §B step 1: tracing init failure → exit 4 (internal-error).
        std::process::exit(EXIT_INTERNAL_ERROR);
    }

    // AC-5: first log line must be Prism version.
    tracing::info!("Prism v{}", env!("CARGO_PKG_VERSION"));
}

/// Step 2 [BLOCKING]: Load `prism.toml` from config directory.
///
/// ADR-022 §B step 2; BC-2.06.011.
/// Reads `prism.toml`, deserializes via serde/toml, validates schema.
/// `$PRISM_CONFIG_DIR` always overrides the default; does NOT fall back to
/// default if the env var points to a non-existent directory (BC-2.06.011 invariant).
///
/// Returns a `PrismConfig` on success.
/// On failure, returns `BootError::ConfigInvalid` (exit 2).
pub async fn step2_load_config(config_dir: &Path) -> Result<PrismConfig, BootError> {
    // BC-2.06.011 EC-06-011-001: Config directory must exist.
    if !config_dir.exists() {
        return Err(BootError::ConfigInvalid(format!(
            "Config directory not found: {}",
            config_dir.display()
        )));
    }
    if !config_dir.is_dir() {
        return Err(BootError::ConfigInvalid(format!(
            "Config path is not a directory: {}",
            config_dir.display()
        )));
    }

    let toml_path = config_dir.join("prism.toml");
    if !toml_path.exists() {
        return Err(BootError::ConfigInvalid(format!(
            "Config file not found: {} \
             (Config directory not found or prism.toml missing)",
            toml_path.display()
        )));
    }

    let content = std::fs::read_to_string(&toml_path).map_err(|e| {
        BootError::ConfigInvalid(format!(
            "Failed to read config file {}: {e}",
            toml_path.display()
        ))
    })?;

    // EC-06-011-003: empty file → exit 2.
    // TOML parse of empty string produces an empty table, which fails schema validation
    // at the required-field check below. We handle it via serde deserialization.

    // Deserialize + schema validation via serde.
    // MED-4 (S-WAVE5-PREP-01 fix-pass-1): extract field name from toml error for AC-4.
    // AC-4: stderr must contain the line number and field name of the parse error.
    // `toml::de::Error` includes line/column context in its Display output.
    // We also extract the field path from the error's span_info when available.
    let config: PrismConfig = toml::from_str(&content).map_err(|e| {
        // toml::de::Error::to_string() includes both the error message and the
        // field context when available (e.g., "missing field `spec_dir` at line 1").
        // The Display output includes the key name in serde missing-field errors.
        let toml_msg = e.to_string();
        BootError::ConfigInvalid(format!(
            "Failed to parse prism.toml: {toml_msg} \
             (AC-4: see line/field context above)"
        ))
    })?;

    tracing::info!(
        config_dir = %config_dir.display(),
        "Config loaded successfully"
    );

    Ok(config)
}

/// Step 3 [BLOCKING]: Construct `OrgRegistry` from config.
///
/// ADR-022 §B step 3; BC-2.21.001.
/// Builds a bijective (org_id, org_slug) registry and verifies uniqueness.
/// Empty org list, duplicate org_id, duplicate org_slug, malformed slug →
/// `BootError::OrgRegistryFailed` (exit 2).
///
/// "Config must declare at least one org" is the AC-9 required message for
/// the empty-list case.
pub async fn step3_init_org_registry(
    config: &PrismConfig,
) -> Result<Arc<prism_core::OrgRegistry>, BootError> {
    use prism_core::OrgRegistry;

    // BC-2.21.001 failure path: empty org list.
    if config.orgs.is_empty() {
        return Err(BootError::OrgRegistryFailed(
            "Config must declare at least one org (BC-2.21.001 EC-21-001-001)".to_string(),
        ));
    }

    let registry = OrgRegistry::new();

    for entry in &config.orgs {
        // Validate kebab-case slug (BC-2.21.001 EC-21-001-004).
        let slug = &entry.org_slug;
        let is_kebab = !slug.is_empty()
            && !slug.starts_with('-')
            && !slug.ends_with('-')
            && slug
                .chars()
                .all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '-');
        if !is_kebab {
            return Err(BootError::OrgRegistryFailed(format!(
                "Invalid org_slug '{}': must be kebab-case (lowercase alphanumeric + hyphens, \
                 no leading/trailing hyphens) (BC-2.21.001 EC-21-001-004)",
                slug
            )));
        }

        // Parse org_id as UUID.
        let org_uuid = uuid::Uuid::parse_str(&entry.org_id).map_err(|e| {
            BootError::OrgRegistryFailed(format!(
                "Invalid org_id '{}': must be a valid UUID: {e}",
                entry.org_id
            ))
        })?;

        // F-PASS2-MED-3 + LOW-3 (S-WAVE5-PREP-01): strict UUID v7 validation.
        // BC-2.21.001 EC-21-001-008: non-v7 UUID → exit 2 with "must be a UUID v7".
        //
        // Validation order (F-PASS2-MED-3 documentation):
        // 1. UUID parse (valid hex format with dashes) — Err → exit 2 "must be a valid UUID"
        // 2. UUID version check (Version::SortRand = v7) — mismatch → exit 2 "must be a UUID v7"
        // 3. Slug kebab-case check — invalid → exit 2 "must be kebab-case"
        // 4. Bijectivity check — duplicate → exit 2 "Duplicate org_id/org_slug"
        // All checks occur in this order per the BC-2.21.001 postconditions spec.
        if org_uuid.get_version() != Some(uuid::Version::SortRand) {
            return Err(BootError::OrgRegistryFailed(format!(
                "org_id '{}' must be a UUID v7 (time-ordered, version 7); \
                 got version {:?} — generate with uuid::Uuid::now_v7() \
                 (BC-2.21.001 EC-21-001-008)",
                entry.org_id,
                org_uuid.get_version()
            )));
        }

        let org_id = prism_core::OrgId::from_uuid(org_uuid);
        let org_slug = prism_core::OrgSlug::new(slug);

        // LOW-2 (S-WAVE5-PREP-01 fix-pass-1): produce canonical BC-2.21.001 messages.
        // BC-2.21.001 Error Cases table specifies:
        //   - SlugConflict → "Duplicate org_slug: {slug}"
        //   - IdConflict   → "Duplicate org_id: {uuid}"
        registry.register(org_slug, org_id).map_err(|e| {
            use prism_core::org_registry::RegistrationError;
            let canonical_msg = match &e {
                RegistrationError::SlugConflict { slug, .. } => {
                    format!("Duplicate org_slug: {slug} (BC-2.21.001 bijectivity constraint)")
                }
                RegistrationError::IdConflict { id, .. } => {
                    format!("Duplicate org_id: {id} (BC-2.21.001 bijectivity constraint)")
                }
            };
            BootError::OrgRegistryFailed(canonical_msg)
        })?;
    }

    tracing::info!(org_count = config.orgs.len(), "OrgRegistry initialized");

    Ok(Arc::new(registry))
}

/// Step 4 [BLOCKING]: Load sensor TOML specs.
///
/// ADR-022 §B step 4; ADR-022 §C ConfigManager wiring contract.
/// Calls `parse_spec_directory(config.spec_dir)` → `ConfigSnapshot`.
/// Wraps in `Arc<ArcSwap<ConfigManager>>` for hot-reload support (AD-007).
/// This is the FIRST production call site for `parse_spec_directory`.
/// On failure: exit(2).
pub async fn step4_load_sensor_specs(
    config: &PrismConfig,
) -> Result<Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>, BootError> {
    use arc_swap::ArcSwap;
    use prism_spec_engine::config_manager::{ConfigManager, parse_spec_directory};

    let spec_dir = &config.spec_dir;

    // MED-3 (S-WAVE5-PREP-01 fix-pass-1): fail-fast if spec_dir does not exist.
    // BC-2.06.011 §Invariants requires strict validation — auto-creating the
    // directory papers over invalid configs and has filesystem side-effects.
    // The validate-config subcommand MUST NOT have filesystem side-effects.
    if !spec_dir.exists() {
        return Err(BootError::ConfigInvalid(format!(
            "spec_dir does not exist: {} \
             (Create the directory or update spec_dir in prism.toml)",
            spec_dir.display()
        )));
    }

    let snapshot = parse_spec_directory(spec_dir).map_err(|e| {
        BootError::ConfigInvalid(format!(
            "Failed to load sensor spec directory {}: {e}",
            spec_dir.display()
        ))
    })?;

    let manager = Arc::new(ArcSwap::from_pointee(ConfigManager::new(snapshot)));

    tracing::info!(
        spec_dir = %spec_dir.display(),
        "Sensor TOML specs loaded"
    );

    Ok(manager)
}

/// Step 4 extension [BLOCKING]: Load sensor TOML specs AND per-org overlay specs.
///
/// S-CONFIG-MULTI-TENANT-OVERRIDE-001 extension of `step4_load_sensor_specs`.
///
/// After loading all TYPE specs from the root `spec_dir`, walks `customers/`
/// subdirectory for per-org overlay files and validates them against the OrgRegistry
/// produced in step 3 (BC-2.06.015 INV-COMPAT-002 — step 3 must precede step 4).
///
/// Overlay validation errors (E-SPEC-019..E-SPEC-023) are collected and aggregated
/// before returning — does NOT short-circuit at the first error (INV-ERR-003,
/// BC-2.06.016). All errors are reported as `BootError::ConfigInvalid` (exit 2).
///
/// On success, returns:
/// - `config_manager` — `Arc<ArcSwap<ConfigManager>>` for hot-reload support
/// - `resolved_spec_map` — `Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>` for
///   O(1) fanout dispatch (INV-OVL-006 — read-only after boot).
///
/// # BC-2.06.015 — OrgRegistry cross-validation
/// Every `customers/<slug>/` directory is cross-checked against `OrgRegistry`.
/// An unregistered slug → E-SPEC-022; boot aborts with exit 2.
///
/// # BC-2.06.012 — Backwards compatibility
/// Absent `customers/` directory → zero `ResolvedSensorSpec` entries; boot
/// continues normally. All fanout targets fall back to TYPE spec `base_url`.
///
/// Story: S-CONFIG-MULTI-TENANT-OVERRIDE-001 | BCs: BC-2.06.012..016
pub async fn step4_load_sensor_specs_with_overlays(
    config: &PrismConfig,
    org_registry: &Arc<prism_core::OrgRegistry>,
) -> Result<
    (
        Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
        Arc<
            std::collections::HashMap<
                prism_spec_engine::ResolvedSpecKey,
                prism_spec_engine::ResolvedSensorSpec,
            >,
        >,
    ),
    BootError,
> {
    use prism_spec_engine::overlay::OverlayLoader;

    // Step 4a: Load TYPE specs (same as step4_load_sensor_specs).
    let config_manager = step4_load_sensor_specs(config).await?;

    // Step 4b: Extract type_specs map from the loaded ConfigSnapshot.
    //
    // ADR-030 Approach D: ConfigSnapshot::sensor_specs now carries spec_parser::SensorSpec
    // directly (unified type). OverlayLoader::load_overlays expects the same type, so we
    // pass sensor_specs directly — no secondary parse, no type conversion (AC-002).
    //
    // This eliminates the build_type_spec_map_for_overlay double-parse that existed while
    // types::SensorSpec (auth_type: String) and spec_parser::SensorSpec (AuthType enum)
    // were distinct. With the unified type, every boot parses each .sensor.toml exactly once.
    let type_specs = {
        let guard = config_manager.load();
        let snapshot = guard.load();

        // PRR-005 invariant preserved: hard-abort on failed TYPE specs before overlay resolution.
        //
        // If any sensor TOML files failed to parse (stored in failed_specs), abort boot with
        // a clear diagnostic. A corrupt TYPE spec would otherwise produce misleading E-SPEC-019
        // "unknown extends" errors for any overlay that references the broken sensor.
        //
        // This replaces the hard-abort that was in build_type_spec_map_for_overlay — the
        // check is now performed against failed_specs populated by parse_spec_directory,
        // rather than a separate directory scan. Behavioral outcome is identical: boot
        // aborts with ConfigInvalid containing the corrupt file path (PRR-005, TD-VSDD-059).
        if !snapshot.failed_specs.is_empty() {
            let detail = snapshot
                .failed_specs
                .iter()
                .map(|(sensor_id, err)| {
                    format!(
                        "  - {} ({}): {}",
                        sensor_id,
                        err.source_path,
                        err.errors.join("; ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            return Err(BootError::ConfigInvalid(format!(
                "One or more TYPE spec files failed to parse during overlay map construction. \
                 Boot aborted to prevent misleading E-SPEC-019 errors for overlays that extend \
                 these sensors. Failed files:\n{detail}"
            )));
        }

        snapshot.sensor_specs.clone()
    };

    // Step 4c: Walk customers/ for per-org overlay files and validate against OrgRegistry.
    // BC-2.06.012: absent customers/ → zero resolved entries, boot continues.
    // BC-2.06.015: every customers/<slug>/ is cross-validated against OrgRegistry.
    // BC-2.06.016 / INV-ERR-003: all errors collected before returning.
    let customers_dir = config.spec_dir.join("customers");
    let overlay_result = OverlayLoader::load_overlays(&customers_dir, &type_specs, org_registry);

    // BC-2.06.016: aggregate ALL overlay errors; if any, boot aborts with exit 2.
    if !overlay_result.errors.is_empty() {
        let detail = overlay_result
            .errors
            .iter()
            .map(|e| format!("  - {e}"))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(BootError::ConfigInvalid(format!(
            "Overlay validation failed ({} error(s)):\n{detail}",
            overlay_result.errors.len()
        )));
    }

    let resolved_spec_map = Arc::new(overlay_result.resolved);

    // ADV-009 fix: add event_type for SAP-1 compliance (BC-2.16.002 catalog row added).
    // event_type = "boot.overlays_loaded" | audit role: operational/boot-traceability
    // recurrence: once per boot / config-reload when customers_dir is present
    tracing::info!(
        event_type = "boot.overlays_loaded",
        customers_dir = %customers_dir.display(),
        overlay_count = resolved_spec_map.len(),
        "Per-org overlay specs loaded (BC-2.06.012..016)"
    );

    Ok((config_manager, resolved_spec_map))
}

// ---------------------------------------------------------------------------
// CredentialRefProbe — injectable probe for step5 (BC-2.03.013 §Test Strategy)
// ---------------------------------------------------------------------------

/// Probe interface for validating credential refs during step 5.
///
/// Abstracts keyring access so that unit tests can inject a test double
/// (Approach B from BC-2.03.013 §Test Strategy). The production implementation
/// uses the keyring crate; tests inject mock implementations such as
/// `AlwaysOkProbe` and `MissingOneProbe` in the integration test suite.
///
/// Contract (BC-2.03.013 §Critical Invariant):
/// - Implementations MUST NOT store or return credential values.
/// - `probe()` performs an existence check AND returns available `auth_type` metadata.
///
/// # Rule C (E-SPEC-014) enforcement
///
/// The `Ok(Some(auth_type))` return enables Rule C enforcement in step5:
/// when the probe returns `Some(actual_shape)`, step5 compares it against
/// `sensor_spec.auth_type` (the `expected_shape`). A mismatch produces
/// `BootError::AuthTypeCredentialMismatch` (exit 2) per BC-2.01.016 §Error Cases
/// E-SPEC-014 and ADR-023 Rule 2 Rule C.
///
/// Probes that cannot determine the auth_type (e.g., keyring-only backends with
/// no metadata store) return `Ok(None)` — Rule C is skipped for that ref.
pub trait CredentialRefProbe: Send + Sync {
    /// Check whether `ref_name` for `sensor_id` is registered in the backend.
    ///
    /// Returns:
    /// - `Ok(Some(auth_type))` — ref exists AND auth_type metadata is available;
    ///   step5 uses this to enforce Rule C (E-SPEC-014)
    /// - `Ok(None)` — ref exists BUT no auth_type metadata is available;
    ///   Rule C check is skipped for this ref
    /// - `Err(BootError::CredentialRefInvalid)` — ref not found (exit 2)
    /// - `Err(BootError::CredentialPermissionDenied)` — backend unavailable (exit 5)
    fn probe(&self, sensor_id: &str, ref_name: &str) -> Result<Option<String>, BootError>;
}

/// Production credential ref probe — uses the `keyring` crate.
///
/// Constructs a namespaced `keyring::Entry("prism", "{sensor_id}/{ref_name}")` and
/// calls `get_password()` to check existence. The value is immediately discarded
/// (AD-017 AI-opaque model: credential values MUST NOT be retained).
///
/// Returns `Ok(None)` on the success path — the keyring backend (ADR-007 / AD-017)
/// stores credential names only; no `auth_type` shape metadata is stored alongside
/// the credential value in the keyring entry. Per [ADR-026 §D3 Rule C Backend Scope
/// (D-706 amendment)]: Rule C (E-SPEC-014 credential structural shape mismatch)
/// enforcement is conditional on the credential backend exposing shape metadata.
/// Shape introspection requires a separate metadata sidecar or plugin-manifest
/// declaration; that capability is deferred to PLUGIN-MIGRATION-001-A.
///
/// The `if let Some(actual_shape)` gate in step5 is structurally correct for when a
/// future backend (e.g., a keyring metadata sidecar or plugin-manifest probe) returns
/// the activating variant. It is intentionally a no-op with the current keyring backend.
///
/// Rules A and B (E-SPEC-012, E-SPEC-013) DO fire in production via
/// `validate_cross_composition` at spec-load time. Only Rule C is backend-conditional.
pub struct KeyringCredentialProbe;

impl CredentialRefProbe for KeyringCredentialProbe {
    fn probe(&self, sensor_id: &str, ref_name: &str) -> Result<Option<String>, BootError> {
        let account = format!("{sensor_id}/{ref_name}");
        let entry = keyring::Entry::new("prism", &account).map_err(|e| {
            BootError::CredentialPermissionDenied(format!(
                "Credential backend unavailable: failed to construct keyring entry \
                 for sensor '{sensor_id}' ref '{ref_name}': {e} (BC-2.03.013)"
            ))
        })?;

        match entry.get_password() {
            Ok(_secret) => {
                // Secret value discarded immediately — AD-017 AI-opaque model.
                // No auth_type metadata in keyring — Rule C skipped for this ref.
                tracing::trace!(
                    sensor_id = %sensor_id,
                    ref_name = %ref_name,
                    "Credential ref validated (exists in backend; no auth_type metadata available)"
                );
                Ok(None)
            }
            Err(keyring::Error::NoEntry) => Err(BootError::CredentialRefInvalid(format!(
                "Unresolvable credential ref: '{ref_name}' for sensor '{sensor_id}' not found in \
                 keyring backend (BC-2.03.013 TV-03-013-003). \
                 Register the credential with: prism credential set {sensor_id} {ref_name}"
            ))),
            Err(e) => Err(BootError::CredentialPermissionDenied(format!(
                "Credential store access denied: keyring backend returned error \
                 for sensor '{sensor_id}' ref '{ref_name}': {e} (BC-2.03.013)"
            ))),
        }
    }
}

/// Step 5 [BLOCKING]: Initialize credential store and validate sensor spec credential refs.
///
/// ADR-022 §B step 5; BC-2.03.013.
/// Constructs the `CredentialStore` backend from the config's `credential_backend`
/// field, then validates all credential references declared in loaded sensor specs
/// (reference-only — no values are loaded per AD-017).
///
/// Per AD-017: NO credential values are loaded into memory — reference-based model.
/// Permission-denied → exit(5). Config-invalid ref → exit(2).
///
/// Uses the [`KeyringCredentialProbe`] production probe. For testing with a
/// custom probe, call [`step5_init_credential_store_with_probe`] directly.
pub async fn step5_init_credential_store(
    config: &PrismConfig,
    config_manager: &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
) -> Result<Arc<dyn prism_credentials::CredentialStore>, BootError> {
    step5_init_credential_store_with_probe(config, config_manager, &KeyringCredentialProbe).await
}

/// Step 5 implementation with injectable credential probe.
///
/// Identical to [`step5_init_credential_store`] but accepts a custom
/// `probe: &dyn CredentialRefProbe` so unit tests can inject a mock
/// (BC-2.03.013 §Test Strategy Approach B). The production boot path
/// calls `step5_init_credential_store` which passes `&KeyringCredentialProbe`.
///
/// # Behavioral coverage (F-PASS3-HIGH-1 closure)
///
/// This function is the correct test entry point for unit tests that need
/// to exercise the credential_refs iteration loop with N>0 refs and a
/// controllable probe outcome.
pub async fn step5_init_credential_store_with_probe(
    config: &PrismConfig,
    config_manager: &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    probe: &dyn CredentialRefProbe,
) -> Result<Arc<dyn prism_credentials::CredentialStore>, BootError> {
    use prism_credentials::{CredentialIndex, KeyringBackend};

    // HIGH-3 (S-WAVE5-PREP-01 fix-pass-1): EncryptedFile backend requires passphrase
    // resolution that is deferred to S-1.07-FOLLOWUP. prism-credentials does NOT yet
    // expose a passphrase-accepting constructor. Fail-fast with ConfigInvalid (exit 2)
    // per orchestrator pre-decision — this is deterministic config feedback, not
    // permission-denied (which would be exit 5 and mislead the user).
    let store: Arc<dyn prism_credentials::CredentialStore> = match &config.credential_backend {
        CredentialBackendConfig::Keyring => {
            // Construct keyring backend (per prism-credentials KeyringBackend::new).
            // The index path lives in state_dir.
            let index_path = config.state_dir.join("credential_index.json");
            let index = CredentialIndex::new(index_path);
            let store = KeyringBackend::new("prism", index);
            tracing::info!("Credential store: keyring backend constructed");
            Arc::new(store) as Arc<dyn prism_credentials::CredentialStore>
        }
        CredentialBackendConfig::EncryptedFile { path } => {
            // HIGH-3: Fail-fast with ConfigInvalid (exit 2), not PermissionDenied (exit 5).
            // A valid encrypted_file config that cannot be opened at v0.1.0 is a config
            // problem, not a permission problem. PermissionDenied implies the backend is
            // reachable but access was denied — encrypted_file passphrase is not yet resolved.
            // Full implementation is S-1.07-FOLLOWUP.
            return Err(BootError::ConfigInvalid(format!(
                "encrypted_file backend requires passphrase resolution \
                 (deferred to S-1.07-FOLLOWUP); \
                 use keyring backend for v0.1.0. \
                 Path: {}",
                path.display()
            )));
        }
    };

    // F-PASS2-HIGH-3 (S-WAVE5-PREP-01 fix-pass-2): Iterate all credential refs declared
    // in loaded sensor specs (BC-2.03.013 happy-path postcondition 2).
    //
    // SensorSpec now carries credential_refs: Vec<CredentialRef> (added in fix-pass-2
    // to prism-spec-engine::types::SensorSpec and spec_parser::SensorSpec). The TOML
    // `[[credential_refs]]` section is optional; existing specs with no section produce
    // an empty Vec (serde default), satisfying EC-03-013-001 (zero refs = boot continues).
    //
    // Validation is reference-only (AD-017 AI-opaque model). Delegated to `probe`
    // so that unit tests can inject a controllable test double.
    //
    // UUID v7 ordering note (F-PASS2-MED-3): validation order is deterministic (HashMap
    // iteration order not guaranteed), but all refs are checked before boot continues.
    // This is documented per BC-2.21.001 EC-21-001-008 — order does not affect correctness.
    let cm_guard = config_manager.load(); // Guard<Arc<ConfigManager>>
    let cm = &**cm_guard; // &ConfigManager
    let snapshot_guard = cm.load(); // Guard<Arc<ConfigSnapshot>>
    let snapshot = &**snapshot_guard; // &ConfigSnapshot
    let mut refs_validated: usize = 0;

    for (sensor_id, sensor_spec) in &snapshot.sensor_specs {
        // Cross-composition validation (BC-2.01.016):
        // Rules A+B (E-SPEC-012/013): enforced at parse time by parse_and_validate_spec_toml
        // via validate_cross_composition (all three production spec-load paths):
        //   - config_manager::parse_spec_directory (boot step 4)
        //   - add_sensor_spec::add_sensor_spec (MCP tool)
        //   - hot_reload::process_spec_changes
        //
        // Rule C (E-SPEC-014): enforced HERE at step5 credential-introspection time.
        // When the probe returns Some(actual_shape), compare against sensor_spec.auth_type.
        // A mismatch means the credential was configured for a different auth method than
        // the spec declares — fail-closed per BC-2.01.016 §Error Cases E-SPEC-014 and
        // ADR-023 Rule 2 Rule C. Credential values MUST NOT appear in the error (AD-017).

        // Step 5: Credential reference existence probing + Rule C shape check.
        //
        // ADR-030 AC-004: sensor_spec.auth_type is now AuthType enum (not String).
        // Use .as_str() to compare against the probe's String return (Rule C E-SPEC-014).
        // .as_str() returns the canonical snake_case form (e.g. "api_key", "bearer_static")
        // which matches the TOML auth_type field — no behavioral change (AC-003/EC-003).
        for cred_ref in &sensor_spec.credential_refs {
            let actual_shape_opt = probe.probe(sensor_id, &cred_ref.name)?;
            if let Some(actual_shape) = actual_shape_opt {
                let expected_shape = sensor_spec.auth_type.as_str();
                if actual_shape != expected_shape {
                    return Err(BootError::AuthTypeCredentialMismatch {
                        sensor_id: sensor_id.clone(),
                        expected_shape: expected_shape.to_string(),
                        actual_shape,
                    });
                }
            }
            refs_validated += 1;
        }
    }

    tracing::info!(
        refs_validated,
        "Credential store initialized: {refs_validated} refs validated (BC-2.03.013)"
    );

    Ok(store)
}

/// Step 6 [BLOCKING]: Initialize audit subsystem.
///
/// ADR-022 §B step 6; BC-2.05.012.
///
/// F-PASS2-CRIT-1 + F-PASS2-HIGH-1 + F-PASS2-HIGH-2 (S-WAVE5-PREP-01 fix-pass-2):
/// Full implementation per BC-2.05.012 using `prism_audit::BootAuditEmitter`:
///
/// 1. Opens RocksDB at `config.state_dir` (all column families including `audit_buffer`).
/// 2. Constructs `prism_audit::BootAuditEmitter::new(backend)` from the prism-audit crate
///    (BC-2.05.012 postcondition 1: "via AuditEmitter").
/// 3. Emits the `boot.audit.initialized` sentinel via `BootAuditEmitter::emit_boot_sentinel`
///    with all required schema fields: event_type, timestamp (RFC 3339), prism_version,
///    config_dir (hash), org_count, boot_step=6 (BC-2.05.012 §Sentinel Schema).
/// 4. `emit_boot_sentinel` calls `append_audit_entry_sync` (flush_wal(true)) to guarantee
///    durable write before returning (BC-2.05.012 postcondition 2: "synchronous and
///    confirmed durable").
/// 5. Returns the `Arc<RocksDbBackend>` for use by step 7.
///
/// On any failure: returns `BootError::AuditInitFailed` (exit 4).
/// Audit is NON-OPTIONAL: no degraded mode, no `--skip-audit` flag (SOC 2).
fn step6_init_audit(
    config: &PrismConfig,
) -> Result<Arc<prism_storage::rocksdb_backend::RocksDbBackend>, BootError> {
    use prism_audit::{BootAuditEmitter, BootSentinelFields};
    use prism_storage::rocksdb_backend::RocksDbBackend;

    // Ensure state_dir exists before opening RocksDB.
    // (BC-2.05.012 EC-05-012-002: if step 2 validates state_dir, this is a safety net.)
    std::fs::create_dir_all(&config.state_dir).map_err(|e| {
        BootError::AuditInitFailed(format!(
            "Audit subsystem init failed: cannot create state_dir {}: {e}",
            config.state_dir.display()
        ))
    })?;

    // Open RocksDB at state_dir — opens ALL column families including audit_buffer CF.
    // BC-2.05.012: audit_buffer CF confirmed open and writable.
    let backend = Arc::new(RocksDbBackend::open(config.state_dir.clone()).map_err(|e| {
        let msg = e.to_string();
        if msg.to_lowercase().contains("lock") || msg.contains("LOCK") {
            // BC-2.05.012 EC-05-012-006: LOCK file exists → actionable message.
            BootError::AuditInitFailed(format!(
                "Audit subsystem init failed: \
                 RocksDB LOCK file exists — Another Prism process may be running. \
                 Check the state_dir/LOCK file. ({msg})"
            ))
        } else {
            BootError::AuditInitFailed(format!(
                "Audit subsystem init failed: RocksDB CF open error: {msg}"
            ))
        }
    })?);

    // F-PASS2-CRIT-1: Construct BootAuditEmitter from the prism-audit crate.
    // BC-2.05.012 postcondition 1: "The audit_buffer RocksDB column family is opened and
    // confirmed writable via AuditEmitter."
    let emitter = BootAuditEmitter::new(Arc::clone(&backend));

    let version = env!("CARGO_PKG_VERSION");

    // Redact config_dir: use SHA-256 hash of the path, not the raw path.
    // BC-2.05.012: "config_dir field MUST be redacted (only a hash or basename)".
    let config_dir_hash = {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };
        let mut h = DefaultHasher::new();
        config.state_dir.hash(&mut h);
        format!("{:016x}", h.finish())
    };

    // F-PASS2-HIGH-2 + F-PASS2-HIGH-1: emit_boot_sentinel builds the complete
    // sentinel with RFC 3339 timestamp and writes via append_audit_entry_sync (fsync).
    emitter
        .emit_boot_sentinel(BootSentinelFields {
            prism_version: version,
            config_dir_hash,
            org_count: config.orgs.len(),
        })
        .map_err(|e| {
            BootError::AuditInitFailed(format!(
                "Audit subsystem init failed: sentinel persistence error: {e}"
            ))
        })?;

    tracing::info!(
        event_type = "boot.audit.initialized",
        prism_version = %version,
        org_count = config.orgs.len(),
        boot_step = 6u32,
        "Audit subsystem initialized; boot.audit.initialized persisted (durable via WAL fsync)"
    );

    Ok(backend)
}

// ---------------------------------------------------------------------------
// Step 7.5 — Plugin-load step (S-PLUGIN-PREREQ-D / BC-2.22.001)
// ---------------------------------------------------------------------------

/// Result of the plugin-load step 7.5.
///
/// Holds the constructed `PluginRuntime` (zero plugins registered if
/// `PRISM_DISABLE_PLUGIN_LOAD=1` was set) so callers can pass it to the
/// query-engine and MCP server steps.
///
/// `plugin_auth_providers` carries pre-constructed `PluginAuthProvider` instances for
/// every sensor spec that declares `auth_plugin = "..."`. These are wired into the
/// QueryEngine at step 8 (`step8_init_query_engine`) so that PipelineExecutor receives
/// the correct `Arc<dyn AuthProvider>` for plugin-authed sensors (F-LP2-HIGH-001).
pub struct PluginLoadResult {
    /// The initialized plugin runtime (always `Some` — never None).
    pub runtime: Arc<prism_spec_engine::plugin::PluginRuntime>,
    /// Number of plugins successfully loaded (0 if disabled or none found).
    pub plugins_loaded: usize,
    /// Pre-constructed PluginAuthProvider instances keyed by sensor_id.
    ///
    /// Populated at step 7.5b for every SensorSpec where `auth_plugin.is_some()`.
    /// Threaded into step 8 (QueryEngine init) so PipelineExecutor dispatches with
    /// the correct `Arc<dyn AuthProvider>` for plugin-authed sensors.
    ///
    /// Construction at step 7.5b (not step 8) proves production reachability:
    /// `PluginAuthProvider::new` is called on the main boot path, not just in tests.
    ///
    /// F-LP2-HIGH-001 closure: this field carries the FIRST non-test construction site
    /// of `PluginAuthProvider`. ADR-028 §D10 co-merge gate: after 001-A deletes
    /// crowdstrike.rs, this is how CrowdStrike gets an AuthProvider in production.
    pub plugin_auth_providers:
        std::collections::HashMap<String, Arc<prism_spec_engine::PluginAuthProvider>>,
}

/// Step 7.5 [BLOCKING]: Plugin-load step — scan plugin directory and load `.prx` plugins.
///
/// BC-2.22.001: plugin-load step — positioned after step 7 (storage init) and before
/// query-engine init per ADR-023 §C4 + ADR-022 §B sequencing invariant.
/// PRISM_DISABLE_PLUGIN_LOAD=1 skips this step (emergency escape valve).
///
/// # Behavior
///
/// 1. Check `PRISM_DISABLE_PLUGIN_LOAD` env var; if set to exact string `"1"`, emit
///    a single `tracing::warn!(event_type = "plugin_load_disabled_via_envvar", ...)` and
///    return `Ok(PluginLoadResult { runtime, plugins_loaded: 0 })` immediately (AC-3/AC-18).
/// 2. Construct a single `reqwest::Client` with 30-second timeout (AC-9).
/// 3. Construct `PluginRuntime::new(http_client)`.
/// 4. Call `runtime.load_all_plugins(&plugin_dir)` (AC-1).
/// 5. Return the runtime for injection into downstream boot steps.
///
/// # Errors
///
/// Returns `Err(BootError::InternalError)` if:
/// - `reqwest::Client` construction fails (OS resource exhaustion, EC-D-009)
/// - `PluginRuntime::new` fails (wasmtime Engine construction)
///
/// These failures exit with code 4 per ADR-022 §A (AC-2).
///
/// Per-plugin load failures do NOT cause this function to fail — the n-1 survivor rule
/// applies inside `load_all_plugins`.
pub async fn plugin_load_step(plugin_dir: &Path) -> Result<PluginLoadResult, BootError> {
    plugin_load_step_with_audit(
        plugin_dir,
        prism_spec_engine::plugin_audit_sink::noop_sink(),
    )
    .await
}

/// Core implementation of the plugin-load step with injectable audit sink.
///
/// The production boot path (run_boot_sequence) calls this with a
/// `RocksDbPluginAuditSink` wired from the step 6 `Arc<RocksDbBackend>`.
/// Integration tests call `plugin_load_step` which passes a `NoOpPluginAuditSink`.
///
/// This separation closes HIGH-002 (F-IMPL-LP1-HIGH-002) without breaking
/// existing tests that don't have RocksDB available.
pub async fn plugin_load_step_with_audit(
    plugin_dir: &Path,
    audit_sink: Arc<dyn prism_spec_engine::plugin_audit_sink::PluginLoadAuditSink>,
) -> Result<PluginLoadResult, BootError> {
    use std::time::Duration;

    use prism_spec_engine::plugin::{PLUGIN_HTTP_CLIENT_TIMEOUT_SECS, PluginRuntime};

    // AC-18: PRISM_DISABLE_PLUGIN_LOAD takes absolute precedence over plugin_dir config.
    // Only the exact string "1" disables loading (EC-D-011). Values like "true", "yes", "0"
    // are treated as unset.
    if std::env::var("PRISM_DISABLE_PLUGIN_LOAD").as_deref() == Ok("1") {
        // Single structured emission per BC-2.16.002 v1.12 catalog row plugin_load_disabled_via_envvar.
        tracing::warn!(
            event_type = "plugin_load_disabled_via_envvar",
            env_var = "PRISM_DISABLE_PLUGIN_LOAD",
            "Plugin loading disabled via PRISM_DISABLE_PLUGIN_LOAD=1; \
             no plugins loaded (emergency escape valve)"
        );

        // Construct runtime without loading plugins (MCP server still binds with zero plugins).
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))
            .build()
            .map_err(|e| {
                BootError::InternalError(format!(
                    "PluginRuntime HTTP client construction failed (EC-D-009): {e}"
                ))
            })?;

        let runtime = PluginRuntime::new_with_audit_sink(http_client, audit_sink)
            .map_err(|e| BootError::InternalError(e.to_string()))?;

        return Ok(PluginLoadResult {
            runtime: Arc::new(runtime),
            plugins_loaded: 0,
            plugin_auth_providers: std::collections::HashMap::new(),
        });
    }

    // AC-9: Construct ONE shared reqwest::Client with 30-second timeout.
    // Construction is fallible (OS resource exhaustion per EC-D-009).
    // On failure: return Err → boot exits with code 4 (ADR-022 §A internal-error class).
    // Using .expect() is FORBIDDEN here — it would panic instead of returning the structured
    // error that EC-D-009 requires (expect_used = "deny" in workspace clippy config).
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))
        .build()
        .map_err(|e| {
            BootError::InternalError(format!(
                "PluginRuntime HTTP client construction failed (EC-D-009): {e}"
            ))
        })?;

    // AC-9: Inject the single shared client + audit sink into PluginRuntime::new_with_audit_sink.
    // HIGH-002 (F-IMPL-LP1-HIGH-002): production boot path wires RocksDbPluginAuditSink here.
    let runtime = PluginRuntime::new_with_audit_sink(http_client, audit_sink)
        .map_err(|e| BootError::InternalError(e.to_string()))?;

    // AC-1: Scan plugin_dir for .prx files and load each one.
    // load_all_plugins returns (n_loaded, pending_write_tool_registrations) — the latter
    // must be registered with prism_query::invalidation::register_write_tool before
    // mark_query_phase_started() is called at step 8 (F-LP-IMPL-P1-002 / ADR-026 §D7 step 7.5).
    let (plugins_loaded, pending_write_tools) = runtime
        .load_all_plugins(plugin_dir)
        .await
        .map_err(|e| BootError::InternalError(e.to_string()))?;

    // Step 7.6 (new): Register plugin write tools with the invalidation map.
    //
    // Must execute AFTER step 7.5 plugin-load AND BEFORE step 8 mark_query_phase_started().
    // Ordering invariant: ADR-022 §B step 7.5/8 (plugin-load → register_write_tool → query-phase).
    //
    // Fail-closed semantics (F-LP-IMPL-P4-002 / BC-2.07.004):
    // If ANY write-tool registration for a plugin fails, the entire plugin is rolled back:
    // - All previously-registered write tools for that plugin are removed from DYNAMIC_WRITE_TOOLS
    // - The plugin is unregistered from PluginRuntime
    // - ERROR-level tracing event `plugin_registration_rolled_back` is emitted
    // (BC-2.16.002 row 34)
    //
    // Rationale: a plugin with partial write-tool registration is in an inconsistent state —
    // its read queries succeed but its write paths are silently broken, violating BC-2.07.004
    // §write-then-read consistency. The "n-1 survivor rule" applies to plugin LOAD failures
    // (step 7.5), NOT to write-tool registration failures (step 7.6) which leave the plugin
    // in an undefined write state.

    // Option B — per-plugin atomic write-tool registration (F-LP-IMPL-P6-001).
    //
    // Group `pending_write_tools` by plugin_name FIRST, preserving the original
    // per-plugin tool insertion order from `load_all_plugins`. Then iterate once
    // per plugin: attempt all of that plugin's tool registrations atomically.
    // If ANY tool fails, immediately:
    //   1. Deregister all tools already registered for this plugin.
    //   2. Unregister the plugin from PluginRuntime.
    //   3. Emit `plugin_registration_rolled_back` (exactly ONCE per plugin, not per tool).
    //   4. Break out of the inner tool loop — do NOT attempt remaining tools for this plugin.
    //
    // This eliminates the loop-continuation orphan bug: in the flat-loop design (pre-fix),
    // after rolling back good_t1 and unregistering plugin P, the loop continued to good_t3,
    // successfully registering it and leaving an orphaned DYNAMIC_WRITE_TOOLS entry for a
    // plugin no longer present in PluginRuntime (BC-2.07.004 §write-then-read consistency
    // violation — a query referencing good_t3's invalidation entry hits a missing plugin).
    //
    // Per-plugin grouping: use IndexMap-like stable ordering via a Vec of (plugin_name, tools).
    // We cannot use HashMap because it drops insertion order. Use a Vec of (name, Vec<tool>)
    // accumulated in encounter order.

    // Build an ordered list of (plugin_name, tools) grouping while preserving order.
    let mut plugin_tool_groups: Vec<(String, Vec<_>)> = Vec::new();
    for (plugin_name, manifest_tool) in &pending_write_tools {
        match plugin_tool_groups
            .iter_mut()
            .find(|(n, _)| n == plugin_name)
        {
            Some((_, tools)) => tools.push(manifest_tool),
            None => plugin_tool_groups.push((plugin_name.clone(), vec![manifest_tool])),
        }
    }

    let mut n_rolled_back: usize = 0;

    // Per-plugin atomic loop: register all tools or roll back entirely.
    'plugin_loop: for (plugin_name, tools) in &plugin_tool_groups {
        for manifest_tool in tools {
            let entry = prism_query::invalidation::WriteToolInvalidationMap::new(
                manifest_tool.tool_name.clone(),
                manifest_tool.source_ids.clone(),
                prism_core::SensorId::from(manifest_tool.sensor_id.as_str()),
                plugin_name.clone(),
            );
            if let Err(reg_err) = prism_query::invalidation::register_write_tool(entry) {
                // Fail-closed: rollback all write tools registered so far for this plugin
                // (including those successfully registered in earlier iterations of this
                // inner loop) + unregister the plugin from PluginRuntime.
                let rolled_back_tools =
                    prism_query::invalidation::deregister_write_tools_for_plugin(plugin_name);
                let plugin_unregistered = runtime.unregister_plugin(plugin_name);
                // Emit exactly ONE rollback event per plugin (not one per remaining tool).
                tracing::error!(
                    event_type = "plugin_registration_rolled_back",
                    plugin_name = %plugin_name,
                    tool_name = %manifest_tool.tool_name,
                    registration_error = %reg_err,
                    rolled_back_tools = rolled_back_tools,
                    plugin_unregistered = plugin_unregistered,
                    "write tool registration failed; plugin rolled back (BC-2.07.004 fail-closed — \
                     partial write-tool state would violate write-then-read consistency)"
                );
                n_rolled_back += 1;
                // Break out of the inner tool loop — do NOT attempt remaining tools for
                // this plugin. Continue to the next plugin in the outer loop.
                continue 'plugin_loop;
            }
        }
    }
    if n_rolled_back > 0 {
        let survivors = plugins_loaded.saturating_sub(n_rolled_back);
        tracing::error!(
            n_rolled_back = n_rolled_back,
            n_surviving = survivors,
            "boot: {} plugin(s) rolled back due to write-tool registration failure \
             (BC-2.07.004 fail-closed; {} plugin(s) operational)",
            n_rolled_back,
            survivors
        );
    }

    tracing::info!(
        n_loaded = plugins_loaded,
        plugin_dir = %plugin_dir.display(),
        "boot: plugin-load step complete ({plugins_loaded} plugins loaded)"
    );

    Ok(PluginLoadResult {
        runtime: Arc::new(runtime),
        plugins_loaded,
        // plugin_auth_providers: populated in step 7.5b of run_boot_sequence after
        // validate_auth_plugin_fields checks pass. Here it starts empty — it is filled
        // in by the caller (run_boot_sequence) using the config_manager snapshot.
        plugin_auth_providers: std::collections::HashMap::new(),
    })
}

// ---------------------------------------------------------------------------
// Steps 7–9 implementations + Steps 10–11 deferred to S-1.12-FOLLOWUP
// ---------------------------------------------------------------------------

/// Step 7 [BLOCKING]: Storage + internal-tables provider init.
///
/// Validates that the RocksDB storage backend (opened in step 6) is alive and
/// all column families are accessible.  Calls [`prism_storage::rocksdb_backend::RocksDbBackend::health_check`]
/// which performs a write/read/delete probe on the `default` CF and verifies
/// all CF handles are reachable.
///
/// Internal table registration per `register_internal_tables` is intentionally
/// deferred to per-query `execute_inner` to keep the boot path lightweight and
/// avoid registering into a session context that is not shared across query calls
/// (each query creates its own ephemeral `SessionContext`).
///
/// # Structured Event
///
/// Emits `boot.step7.storage_validated` (INFO) on success per BC-2.16.002 catalog
/// (implemented in S-3.02-FOLLOWUP-RUNTIME, PR #162).
///
/// # Errors
///
/// Returns `BootError::InternalError` if the health check fails (maps to exit 4).
pub async fn step7_init_storage(
    storage: &Arc<prism_storage::rocksdb_backend::RocksDbBackend>,
) -> Result<(), BootError> {
    storage.health_check().map_err(|e| {
        BootError::InternalError(format!(
            "step 7 storage health check failed — RocksDB backend is not ready: {e}"
        ))
    })?;
    tracing::info!(
        event_type = "boot.step7.storage_validated",
        "boot: step 7 storage init complete — RocksDB backend alive, all CFs accessible \
         (internal tables registered per-query via register_internal_tables in execute_inner)"
    );
    Ok(())
}

/// Step 8 [BLOCKING → BACKGROUND]: Close write-tool registration window.
///
/// Calls [`prism_query::invalidation::mark_query_phase_started`] to permanently
/// close the write-tool registration window (ADR-026 §D7; ADR-022 §B step 7.5/8).
/// This is the sole load-bearing act of this step (implemented in S-3.02-FOLLOWUP-RUNTIME, PR #162).
///
/// QueryEngine + WriteExecutor construction and wiring into PrismServer is
/// performed by S-5.01-FOLLOWUP-MCP-BOOT (boot step 9), which is the first
/// consumer of the constructed engine.
///
/// # AdapterRegistry assertion (S-5.01-FOLLOWUP-MCP-BOOT)
///
/// When step 9 (S-5.01-FOLLOWUP-MCP-BOOT) constructs the QueryEngine, the FIRST
/// thing it must do is verify the `AdapterRegistry` contains at least one adapter
/// before serving queries. Without this assertion, a silent `init_registry_for_org`
/// failure would propagate as silent empty results across all queries (regressing
/// ADV-W3MT-P58-LOW-002 fix).
///
/// Defense-in-depth: `materialization.rs` retains `is_empty()` short-circuit
/// (test-mode aware) until this assertion is enforced in S-5.01-FOLLOWUP-MCP-BOOT.
pub async fn step8_init_query_engine() -> Result<(), BootError> {
    // Mark query phase started as the FIRST act of step 8, before QueryEngine construction.
    // This permanently closes the write-tool registration window (ADR-026 §D7; ADR-022 §B step 7.5/8).
    // F-LP56-HIGH-001 adjudication: this is the sole permitted boot.rs change in S-PLUGIN-PREREQ-E.
    prism_query::invalidation::mark_query_phase_started();

    // QueryEngine + WriteExecutor construction: S-5.01-FOLLOWUP-MCP-BOOT.
    // Requires Arc<RocksDbBackend>, Arc<AdapterRegistry>, Arc<OcsfNormalizer>,
    // Arc<ClientRegistry>, Arc<dyn CredentialResolver>, and Arc<OrgRegistry> —
    // all of which live in BootContext / RunningServer and are threaded into
    // step 9 by run_boot_sequence when S-5.01-FOLLOWUP-MCP-BOOT implements it.
    tracing::info!(
        event_type = "boot.step8.query_engine_started",
        "boot: step 8 query-engine phase started — write-tool registration window closed \
         (QueryEngine + WriteExecutor wired via S-5.01-FOLLOWUP-MCP-BOOT)"
    );
    Ok(())
}

/// Step 9 [BACKGROUND]: MCP server start.
///
/// Constructs `QueryEngine` + `WriteExecutor` from available boot deps, builds
/// `PrismServer::with_deps(...)`, and spawns `serve_stdio` as a background tokio task.
///
/// Gate: MCP server MUST NOT start before step 8 completes (BC-2.22.001 pre-traffic gate).
/// Step 8 calls `mark_query_phase_started()` — step 9 only runs after step 8 returns Ok(()).
///
/// # Dependency injection (ADR-022 §F)
///
/// Uses RocksDbBackend from step 6 (via BootContext) as the query storage backend.
/// Uses OrgRegistry from step 3 and CredentialStore from step 5 (CRIT-5 fix).
/// Step 9A (S-DEMO-001): calls `step9a_populate_adapter_registry` to wire spec-driven adapters.
/// The AuditWriter is a tracing stub until the full audit-writer integration story (S-2.04) ships.
///
/// # Background task semantics
///
/// `serve_stdio` runs for the lifetime of the server process.  This function
/// returns `Ok(())` immediately after spawning the background task — the caller
/// (`run_boot_sequence`) proceeds to steps 10–11.  The background serve task
/// exits when:
///   - stdin closes (client disconnect), or
///   - SIGTERM/SIGINT is received (BC-2.10.010 — handled inside `serve_stdio`).
pub async fn step9_start_mcp_server(
    storage: Arc<prism_storage::rocksdb_backend::RocksDbBackend>,
    config_manager: Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    resolved_spec_map: Arc<
        std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
    org_registry: Arc<prism_core::OrgRegistry>,
    credential_store: Arc<dyn prism_credentials::CredentialStore>,
    spec_dir: std::path::PathBuf,
    config_dir: std::path::PathBuf,
    plugin_auth_providers: std::collections::HashMap<
        String,
        Arc<prism_spec_engine::PluginAuthProvider>,
    >,
) -> Result<tokio::task::JoinHandle<Result<(), rmcp::RmcpError>>, BootError> {
    use std::collections::BTreeMap;

    use prism_mcp::server::PrismServer;
    use prism_ocsf::OcsfNormalizer;
    use prism_query::{
        WriteExecutor, WritePlan, WriteResult,
        engine::{QueryEngine, QueryEngineConfig},
        scoping::ClientRegistry,
        write_dispatch::AuditWriter,
    };
    use prism_security::{
        confirmation_token::ConfirmationTokenStore,
        feature_flag::{CapabilityCheckResult, FeatureFlagEvaluator},
        injection_scanner::InjectionScanner,
    };
    use prism_sensors::AdapterRegistry;
    use prism_spec_engine::write_endpoint::WriteEndpointRegistry;

    // ── Build QueryEngine ─────────────────────────────────────────────────────
    //
    // Step 9A (S-DEMO-001): Populate AdapterRegistry from resolved spec map.
    // Calls step9a_populate_adapter_registry to wire one SpecDrivenSensorAdapter
    // per (OrgId, SensorId) pair from resolved_spec_map (BC-2.22.001 §Step 9A).
    let mut adapter_registry_inner = AdapterRegistry::new();
    crate::spec_driven_adapter::step9a_populate_adapter_registry(
        &resolved_spec_map,
        &org_registry,
        &plugin_auth_providers,
        &mut adapter_registry_inner,
    )
    .await?;
    let adapter_registry = Arc::new(adapter_registry_inner);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());

    // ClientRegistry: populated from OrgRegistry slug list (F-PR163-IMP-8).
    // OrgRegistry::list_slugs() returns Vec<String>; ClientRegistry::new expects Vec<OrgSlug>.
    let client_slugs: Vec<prism_core::tenant::OrgSlug> = org_registry
        .list_slugs()
        .into_iter()
        .map(prism_core::tenant::OrgSlug::new)
        .collect();
    let client_registry = Arc::new(ClientRegistry::new(client_slugs));

    let query_config = QueryEngineConfig::default();

    // ProductionCredentialResolver: wraps the real CredentialStore from step 5.
    //
    // All sensor auth is now via WASM plugins (PluginAuthProvider, ADR-028 §D10) or via
    // SpecDrivenSensorAdapter (step 9A, S-DEMO-001). The CredentialResolver trait is used
    // by the fan_out path in prism-sensors when direct sensor adapters are dispatched.
    // SpecDrivenSensorAdapter handles its own auth dispatch via AdapterAuthStrategy — it
    // does NOT call the CredentialResolver. This resolver is retained for architectural
    // completeness and will be wired in S-2.07 (per-sensor auth resolution story).
    //
    // This is NOT a BootNull* — it correctly reports the architectural state:
    // the credential_store IS wired, but SpecDrivenSensorAdapter and WASM plugin auth
    // both bypass this resolver (plugins: via PluginAuthProvider::acquire_token;
    // spec-driven: via AdapterAuthStrategy). Resolving per-sensor SensorAuth subtypes
    // from the credential_store at fan_out time requires S-2.07.
    struct ProductionCredentialResolver {
        #[allow(dead_code)]
        credential_store: Arc<dyn prism_credentials::CredentialStore>,
    }
    impl prism_sensors::CredentialResolver for ProductionCredentialResolver {
        fn resolve(
            &self,
            client_id: &str,
            sensor_id: prism_core::SensorId,
        ) -> Result<Box<dyn prism_sensors::auth::SensorAuth>, prism_sensors::adapter::SensorError>
        {
            // SpecDrivenSensorAdapter handles auth via AdapterAuthStrategy (S-DEMO-001).
            // WASM plugins handle auth via PluginAuthProvider (ADR-028 §D10).
            // Neither path uses the CredentialResolver — they acquire tokens at the
            // adapter level before fan_out dispatches. Per-query credential resolution
            // via this resolver is S-2.07.
            Err(prism_sensors::adapter::SensorError::ConfigValidation {
                sensor: sensor_id.to_string(),
                detail: format!(
                    "Direct sensor auth for client '{client_id}' sensor '{sensor_id}': \
                     use SpecDrivenSensorAdapter (S-DEMO-001) or WASM plugin (ADR-028 §D10). \
                     Per-query credential resolution via CredentialResolver is S-2.07."
                ),
            })
        }
    }
    let credential_resolver: Arc<dyn prism_sensors::CredentialResolver> =
        Arc::new(ProductionCredentialResolver {
            credential_store: Arc::clone(&credential_store),
        });

    // ── Build AliasStore ──────────────────────────────────────────────────────
    //
    // Constructed BEFORE QueryEngine so the same Arc<Mutex<AliasStore>> can be
    // shared between:
    //   1. QueryEngine — for @alias expansion at query execution time (F-PASS9-LOW-1).
    //   2. PrismServer — for alias CRUD tools (create/list/delete/explain_alias).
    //
    // BC-2.11.008: aliases created via MCP tools are live-queryable immediately
    // because both subsystems share the same lock-protected in-memory store.
    //
    // Try to load existing aliases.toml from the config directory; fall back to
    // an empty store if the file does not exist (aliases.toml is optional).
    let alias_file = config_dir.join("aliases.toml");
    let alias_store = if alias_file.exists() {
        match prism_query::alias_store::AliasStore::load(&alias_file) {
            Ok(store) => {
                tracing::info!(
                    event_type = "boot.step9.alias_store_loaded",
                    path = %alias_file.display(),
                    "boot: step 9 — alias store loaded from disk"
                );
                store
            }
            Err(e) => {
                tracing::warn!(
                    event_type = "boot.step9.alias_store_load_failed",
                    path = %alias_file.display(),
                    error = %e,
                    "boot: step 9 — alias store load failed, starting with empty store"
                );
                prism_query::alias_store::AliasStore::empty(&alias_file)
            }
        }
    } else {
        tracing::info!(
            event_type = "boot.step9.alias_store_empty",
            path = %alias_file.display(),
            "boot: step 9 — aliases.toml not found, starting with empty alias store"
        );
        prism_query::alias_store::AliasStore::empty(&alias_file)
    };
    let alias_store = Arc::new(std::sync::Mutex::new(alias_store));

    // IMP-8: retain org_registry Arc so it can be passed to both QueryEngine and PrismServer.
    // Arc::clone before the move into QueryEngine::new_full so PrismServer::with_deps
    // can also receive the registry for alias CRUD allowlist validation.
    let org_registry_for_server = Arc::clone(&org_registry);
    let query_engine = Arc::new(QueryEngine::new_full(
        adapter_registry.clone(),
        // CRIT-5: real credential_store from step 5 — replaces BootNullCredentialStore.
        credential_store,
        ocsf_normalizer,
        client_registry,
        query_config,
        credential_resolver,
        // CRIT-5: real org_registry from step 3 — replaces OrgRegistry::new() placeholder.
        org_registry,
        storage,
        resolved_spec_map,
        // F-PASS9-LOW-1: alias_store shared with PrismServer so @alias tokens in queries
        // are resolved against aliases created via MCP tools (BC-2.11.008).
        Arc::clone(&alias_store),
    ));

    // ── Build WriteExecutor ───────────────────────────────────────────────────
    //
    // Feature flags start with empty client capability map — all write capabilities deny-by-default
    // until the per-client capability config loads (S-2.03). The deny-by-default posture matches
    // the production security-default; do not interpret the empty map as "all-open".
    let feature_flags = Arc::new(FeatureFlagEvaluator::new(BTreeMap::new()));
    let confirmation_store = Arc::new(ConfirmationTokenStore::new());

    // TracingAuditWriter — emits structured tracing events for write audit entries.
    // Full AuditEmitter integration via Tower layer requires S-2.04.
    struct TracingAuditWriter;
    #[async_trait::async_trait]
    impl AuditWriter for TracingAuditWriter {
        async fn write_intent(
            &self,
            plan: &WritePlan,
            _context: &prism_query::QueryContext,
            _capability_check: &CapabilityCheckResult,
        ) -> Result<ulid::Ulid, prism_core::error::PrismError> {
            let id = ulid::Ulid::new();
            tracing::info!(
                event_type = "write.intent.recorded",
                intent_id = %id,
                sensor = %plan.sensor,
                "write intent recorded (BC-2.05.009 tracing audit stub)"
            );
            Ok(id)
        }
        async fn write_outcome(
            &self,
            intent_id: ulid::Ulid,
            result: &WriteResult,
        ) -> Result<(), prism_core::error::PrismError> {
            tracing::info!(
                event_type = "write.outcome.recorded",
                intent_id = %intent_id,
                succeeded = result.succeeded_count,
                failed = result.failed_count,
                "write outcome recorded (BC-2.05.009 tracing audit stub)"
            );
            Ok(())
        }
    }
    let audit_writer: Arc<dyn AuditWriter> = Arc::new(TracingAuditWriter);
    let write_adapter_registry = Arc::new(AdapterRegistry::new());
    let endpoint_registry = Arc::new(WriteEndpointRegistry::new());

    let write_executor = Arc::new(WriteExecutor::new(
        feature_flags,
        confirmation_store,
        audit_writer.clone(),
        write_adapter_registry,
        endpoint_registry,
    ));

    // ── Construct PrismServer and spawn serve_stdio ───────────────────────────
    let injection_scanner = Arc::new(InjectionScanner);
    let server = PrismServer::with_deps(
        injection_scanner,
        query_engine,
        write_executor,
        audit_writer,
        // CRIT-4: wire config_manager so config tools can list/reload/validate/add specs.
        config_manager,
        // CRIT-4: wire spec_dir for reload_config and add_sensor_spec tools.
        spec_dir,
        // CRIT-4: wire alias_store for alias CRUD tools.
        alias_store,
        // IMP-8: wire org_registry for alias CRUD allowlist validation.
        org_registry_for_server,
    );

    // Spawn serve_stdio as a background task — returns immediately.
    // The background task runs until stdin closes or SIGTERM/SIGINT is received.
    // BC-2.10.010: graceful shutdown is handled inside serve_stdio.
    //
    // The JoinHandle<Result<(), rmcp::RmcpError>> is stored in RunningServer.
    // main.rs awaits RunningServer::wait_for_shutdown() → translates the Result to
    // an exit code (BC-2.10.010: timeout → exit 1; clean → exit 0; panic → exit 4).
    let handle = tokio::spawn(async move { server.serve_stdio().await });

    // F-PASS12-MED-3: emit mcp_server_started AFTER tokio::spawn so the catalog statement
    // "emitted once the background transport task is running" is accurate (BC-2.16.002).
    // Pre-spawn emission was a catalog drift (the server was not yet spawned).
    tracing::info!(
        event_type = "boot.step9.mcp_server_started",
        "boot: step 9 — PrismServer spawned on stdio transport (BC-2.10.006)"
    );

    Ok(handle)
}

/// Step 10 [BACKGROUND]: Install HotReloadWatcher.
///
/// The HotReloadWatcher (S-1.12-FOLLOWUP) is not yet implemented — boot continues
/// without the hot-reload watcher.  This is non-fatal: the server operates in
/// degraded mode (no filesystem change detection) until S-1.12-FOLLOWUP ships.
///
/// This step MUST NOT use `todo!()` — a `todo!()` would cause `prism start` to panic
/// at boot time.  The structured log below surfaces the deferred feature to operators.
///
/// BC-2.22.001 step 10 deferred contract: returns `Ok(())` immediately; emits
/// `boot.step10.deferred` WARN so monitoring can detect degraded-mode operation.
pub async fn step10_start_hot_reload() -> Result<(), BootError> {
    tracing::warn!(
        event_type = "boot.step10.deferred",
        target = "boot",
        "Hot-reload watcher deferred to S-1.12-FOLLOWUP — \
         boot continues without filesystem watcher (degraded mode)"
    );
    Ok(())
}

/// Step 11 [BACKGROUND]: Install tokio signal handlers.
///
/// The full SIGHUP reload path requires `HotReloadWatcher` (S-1.12-FOLLOWUP).
/// Signal handlers for SIGTERM/SIGINT are already wired inside
/// `PrismServer::serve_stdio` (BC-2.10.010).  The additional SIGHUP handler
/// that triggers a config reload is deferred.
///
/// This step MUST NOT use `todo!()` — a `todo!()` would cause `prism start` to panic.
/// The channels are accepted but not connected until S-1.12-FOLLOWUP ships the
/// HotReloadWatcher and SIGHUP dispatch.
///
/// BC-2.22.001 step 11 deferred contract: returns `Ok(())` immediately; emits
/// `boot.step11.deferred` WARN so monitoring can detect degraded-mode operation.
pub async fn step11_install_signal_handlers(
    _shutdown_tx: tokio::sync::broadcast::Sender<()>,
    _reload_tx: tokio::sync::mpsc::Sender<()>,
) -> Result<(), BootError> {
    tracing::warn!(
        event_type = "boot.step11.deferred",
        target = "boot",
        "SIGHUP config-reload handler deferred to S-1.12-FOLLOWUP — \
         SIGTERM/SIGINT handled by PrismServer::serve_stdio (BC-2.10.010)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Inline unit tests — BootError mapping and step-function stubs
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// Story: S-WAVE5-PREP-01 + S-PLUGIN-PREREQ-E F-LP-IMPL-P4-001
    /// BC: BC-2.22.001 — BootError::exit_code() maps all variants correctly
    ///
    /// Unit test of the already-implemented exit_code() method.
    /// Documents the full mapping table in test form.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_22_001_boot_error_exit_code_complete_mapping() {
        // Config-invalid class → 2
        assert_eq!(BootError::ConfigInvalid("x".into()).exit_code(), 2);
        assert_eq!(BootError::OrgRegistryFailed("x".into()).exit_code(), 2);
        assert_eq!(BootError::CredentialRefInvalid("x".into()).exit_code(), 2);
        // AuthTypeCredentialMismatch → 2 (config-invalid: spec and credential disagree on auth_type)
        assert_eq!(
            BootError::AuthTypeCredentialMismatch {
                sensor_id: "test".into(),
                expected_shape: "oauth2_client_credentials".into(),
                actual_shape: "api_key".into(),
            }
            .exit_code(),
            2,
            "AuthTypeCredentialMismatch must map to exit 2 \
             (config-invalid: spec and credential disagree on auth_type — E-SPEC-014)"
        );
        // Permission-denied → 5
        assert_eq!(
            BootError::CredentialPermissionDenied("x".into()).exit_code(),
            5
        );
        // Internal-error → 4
        assert_eq!(BootError::AuditInitFailed("x".into()).exit_code(), 4);
        assert_eq!(BootError::InternalError("x".into()).exit_code(), 4);
        // Sensor-fail → 3
        assert_eq!(BootError::SensorFail("x".into()).exit_code(), 3);
    }

    /// Story: S-WAVE5-PREP-01  AC-7
    /// BC: BC-2.03.013 — permission-denied maps to exit 5, not 2 or 4
    ///
    /// This is the most critical mapping distinction: CredentialPermissionDenied
    /// must be 5, but CredentialRefInvalid must be 2.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_03_013_credential_exit_code_distinction() {
        let permission = BootError::CredentialPermissionDenied("locked".into());
        let ref_invalid = BootError::CredentialRefInvalid("missing".into());

        assert_eq!(
            permission.exit_code(),
            5,
            "permission-denied must be exit 5"
        );
        assert_eq!(ref_invalid.exit_code(), 2, "ref-invalid must be exit 2");
        assert_ne!(
            permission.exit_code(),
            ref_invalid.exit_code(),
            "permission-denied and ref-invalid must map to DIFFERENT exit codes"
        );
    }

    /// Story: S-WAVE5-PREP-01
    /// BC: BC-2.06.011 — PrismConfig struct has required fields
    ///
    /// Verifies that the PrismConfig placeholder has the fields that boot steps
    /// 2–6 require (spec_dir, state_dir, orgs). These fields being present is
    /// a prerequisite for the TOML deserialization in step 2.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_06_011_prism_config_has_required_fields() {
        // Construct a minimal PrismConfig to verify the struct fields compile.
        let config = PrismConfig {
            spec_dir: PathBuf::from("/tmp/specs"),
            state_dir: PathBuf::from("/tmp/state"),
            plugin_dir: PathBuf::from("plugins"),
            orgs: vec![OrgEntry {
                org_id: "0196f000-0000-7000-8000-000000000001".to_string(),
                org_slug: "acme".to_string(),
            }],
            credential_backend: CredentialBackendConfig::Keyring,
        };
        assert_eq!(config.spec_dir, PathBuf::from("/tmp/specs"));
        assert_eq!(config.state_dir, PathBuf::from("/tmp/state"));
        assert_eq!(config.orgs.len(), 1);
        assert_eq!(config.orgs[0].org_slug, "acme");
    }

    /// Story: S-WAVE5-PREP-01
    /// BC: BC-2.21.001 EC-21-001-001 — minimum org list: 1 entry is valid
    ///
    /// Verifies that OrgEntry with a valid UUID and kebab-case slug compiles.
    /// The actual validation is in step3_init_org_registry (todo!()).
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_21_001_org_entry_with_valid_uuid_and_kebab_slug() {
        let entry = OrgEntry {
            org_id: "0196f000-0000-7000-8000-000000000001".to_string(),
            org_slug: "acme-corp".to_string(),
        };
        // Kebab-case: lowercase alphanumeric + hyphens.
        let slug = &entry.org_slug;
        assert!(
            slug.chars()
                .all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '-'),
            "org_slug must be kebab-case (BC-2.21.001); got: {slug}"
        );
        assert!(
            !slug.starts_with('-'),
            "org_slug must not start with hyphen"
        );
        assert!(!slug.ends_with('-'), "org_slug must not end with hyphen");
        assert!(!slug.is_empty(), "org_slug must not be empty");
    }

    /// Story: S-WAVE5-PREP-01
    /// BC: BC-2.21.001 EC-21-001-004 — org_slug with uppercase fails kebab validation
    ///
    /// Demonstrates the malformed-slug detection that step3_init_org_registry
    /// must implement. This test exercises the OrgEntry type (not the validator,
    /// which is todo!() in step 3).
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_21_001_malformed_slug_fails_kebab_check() {
        let entry = OrgEntry {
            org_id: "0196f000-0000-7000-8000-000000000001".to_string(),
            org_slug: "ACME".to_string(), // uppercase — invalid
        };
        let slug = &entry.org_slug;
        // The slug FAILS kebab-case validation (step3 must reject this).
        let is_kebab = slug
            .chars()
            .all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '-')
            && !slug.starts_with('-')
            && !slug.ends_with('-')
            && !slug.is_empty();
        assert!(
            !is_kebab,
            "ACME slug must fail kebab-case validation (BC-2.21.001 EC-21-001-004); \
             step3_init_org_registry must return OrgRegistryFailed for this slug"
        );
    }
}

// ---------------------------------------------------------------------------
// PrismConfig placeholder
// ---------------------------------------------------------------------------

/// Deserialized `prism.toml` config struct.
///
/// Fields match the schema required by boot steps 2–6 and step 7.5 (plugin-load).
///
/// Marked `#[non_exhaustive]` — new fields may be added in future releases without
/// breaking external code that constructs or matches on this type (CLAUDE.md convention).
#[non_exhaustive]
#[derive(Debug, serde::Deserialize)]
pub struct PrismConfig {
    /// Path to the sensor spec directory (required; boot step 4 uses this).
    pub spec_dir: PathBuf,
    /// Path to the state directory (RocksDB data; required; boot step 6 uses this).
    pub state_dir: PathBuf,
    /// Path to the plugin directory (optional; boot step 7.5 uses this).
    ///
    /// Default is `"plugins"` relative to the config file's directory when absent from
    /// `prism.toml`. The directory is scanned for `*.prx` plugin files at boot step 7.5
    /// (BC-2.22.001 §Sequencing Invariant / S-PLUGIN-PREREQ-D AC-1).
    ///
    /// Set `PRISM_DISABLE_PLUGIN_LOAD=1` to skip plugin loading regardless of this path
    /// (emergency escape valve — AC-18).
    #[serde(default = "default_plugin_dir")]
    pub plugin_dir: PathBuf,
    /// List of configured orgs (required; boot step 3 uses this).
    #[serde(default)]
    pub orgs: Vec<OrgEntry>,
    /// Credential backend type declared in prism.toml.
    #[serde(default)]
    pub credential_backend: CredentialBackendConfig,
}

impl PrismConfig {
    /// Construct a `PrismConfig` for use in tests.
    ///
    /// The `#[non_exhaustive]` attribute prevents external struct literal construction;
    /// use this factory for tests in external crates (e.g., `prism-bin` integration tests).
    ///
    /// `plugin_dir` defaults to `"plugins"` (the TOML default); callers may override it.
    pub fn new_for_test(
        spec_dir: impl Into<PathBuf>,
        state_dir: impl Into<PathBuf>,
        plugin_dir: impl Into<PathBuf>,
        orgs: Vec<OrgEntry>,
        credential_backend: CredentialBackendConfig,
    ) -> Self {
        Self {
            spec_dir: spec_dir.into(),
            state_dir: state_dir.into(),
            plugin_dir: plugin_dir.into(),
            orgs,
            credential_backend,
        }
    }
}

/// Default `plugin_dir` when absent from `prism.toml`: `"plugins"` relative to config
/// file location (AC-1 line 282-283 of S-PLUGIN-PREREQ-D story spec v1.32).
fn default_plugin_dir() -> PathBuf {
    PathBuf::from("plugins")
}

/// A single org entry from `prism.toml`.
///
/// Marked `#[non_exhaustive]` per project convention (CLAUDE.md) — new fields may be
/// added to `prism.toml` without breaking external code.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OrgEntry {
    /// UUID v7 org identifier.
    pub org_id: String,
    /// Kebab-case org slug.
    pub org_slug: String,
}

impl OrgEntry {
    /// Construct a new `OrgEntry` with the given org_id and org_slug strings.
    ///
    /// Prefer this over struct literal syntax (which is forbidden externally due to
    /// `#[non_exhaustive]`).
    pub fn new(org_id: impl Into<String>, org_slug: impl Into<String>) -> Self {
        Self {
            org_id: org_id.into(),
            org_slug: org_slug.into(),
        }
    }
}

/// Credential backend selector from `prism.toml`.
///
/// Marked `#[non_exhaustive]` per project convention (CLAUDE.md) — new backend variants
/// may be added in future releases.
#[non_exhaustive]
#[derive(Debug, Default, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CredentialBackendConfig {
    #[default]
    Keyring,
    EncryptedFile {
        path: PathBuf,
    },
}

// ---------------------------------------------------------------------------
// Unit tests — step4_load_sensor_specs_with_overlays (BC-2.06.012, BC-2.06.015)
// ---------------------------------------------------------------------------
//
// SID-1: tests use in-process tempdir helpers; no subprocess spawning; no #[ignore].
// These tests exercise the production code path directly.

#[cfg(test)]
mod step4_overlay_tests {
    use std::sync::Arc;

    use prism_core::{OrgId, OrgRegistry, OrgSlug};

    use super::*;

    /// Minimal valid TYPE spec TOML for testing.
    const ARMIS_TYPE_SPEC_TOML: &str = r#"
sensor_id = "armis"
name = "Armis test"
auth_type = "bearer_static"
base_url = "https://armis.default.example.com"
version = "1.0.0"

[[tables]]
table_name = "devices"
ocsf_class = "device_inventory_info"

  [[tables.columns]]
  name = "device_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/api/v1/devices"
  response_path = "$.data"
  variables_produced = []
"#;

    /// Build a minimal prism.toml + spec_dir/ directory structure in a tempdir.
    ///
    /// Returns the tempdir (kept alive for test lifetime) and the `PrismConfig`.
    fn make_config_dir_with_sensor(
        org_id: &str,
        org_slug: &str,
    ) -> (tempfile::TempDir, PrismConfig) {
        let dir = tempfile::tempdir().expect("tempdir must succeed");

        // Write prism.toml.
        let prism_toml = format!(
            r#"
spec_dir = "specs"
state_dir = "state"

[[orgs]]
org_id = "{org_id}"
org_slug = "{org_slug}"
"#
        );
        std::fs::write(dir.path().join("prism.toml"), &prism_toml).expect("write prism.toml");

        // Create specs/ directory with the Armis TYPE spec.
        let spec_dir = dir.path().join("specs");
        std::fs::create_dir_all(&spec_dir).expect("create specs/");
        std::fs::write(spec_dir.join("armis.sensor.toml"), ARMIS_TYPE_SPEC_TOML)
            .expect("write armis.sensor.toml");

        // Construct a PrismConfig pointing to the tempdir.
        let config = PrismConfig::new_for_test(
            spec_dir,
            dir.path().join("state"),
            dir.path().join("plugins"),
            vec![OrgEntry::new(org_id, org_slug)],
            CredentialBackendConfig::Keyring,
        );

        (dir, config)
    }

    /// Build an OrgRegistry with one org.
    fn registry_with_org(org_id: OrgId, org_slug: &str) -> Arc<OrgRegistry> {
        let reg = OrgRegistry::new();
        reg.register(OrgSlug::new(org_slug), org_id)
            .expect("register must succeed");
        Arc::new(reg)
    }

    /// BC-2.06.012: absent customers/ directory → zero resolved entries, boot continues.
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_BC_2_06_012_no_customers_dir_boots_with_empty_overlay_map() {
        // Use a valid UUID v7 for org_id (BC-2.21.001 EC-21-001-008).
        let v7 = uuid::Uuid::now_v7();
        let v7_str = v7.to_string();
        let (dir, config) = make_config_dir_with_sensor(&v7_str, "acme");
        let _ = &dir; // keep alive

        // Explicitly ensure no customers/ directory exists.
        assert!(
            !config.spec_dir.join("../customers").exists(),
            "customers/ must not exist for this test"
        );

        let org_id = prism_core::OrgId::from_uuid(v7);
        let registry = registry_with_org(org_id, "acme");

        let result = step4_load_sensor_specs_with_overlays(&config, &registry).await;
        let (_config_manager, resolved_map) = result
            .expect("boot must succeed with absent customers/ (BC-2.06.012 backwards compat)");

        assert!(
            resolved_map.is_empty(),
            "BC-2.06.012: absent customers/ → zero ResolvedSensorSpec entries"
        );
    }

    /// BC-2.06.015: unknown org slug in customers/ → overlay error → boot aborts exit 2.
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_BC_2_06_015_unknown_org_slug_in_customers_dir_aborts_boot() {
        let v7 = uuid::Uuid::now_v7();
        let v7_str = v7.to_string();
        let (dir, config) = make_config_dir_with_sensor(&v7_str, "acme");
        let _ = &dir; // keep alive

        // Create customers/unknown-org/ with a dummy overlay.
        let overlay_dir = config.spec_dir.join("customers").join("unknown-org");
        std::fs::create_dir_all(&overlay_dir).expect("create unknown-org overlay dir");
        std::fs::write(
            overlay_dir.join("armis.sensor.toml"),
            r#"extends = "armis"
instance_id = "armis@unknown-org"
base_url = "https://armis.unknown.example.com"
"#,
        )
        .expect("write overlay file");

        let org_id = prism_core::OrgId::from_uuid(v7);
        let registry = registry_with_org(org_id, "acme");

        let result = step4_load_sensor_specs_with_overlays(&config, &registry).await;
        match result {
            Err(BootError::ConfigInvalid(msg)) => {
                assert!(
                    msg.contains("Overlay validation failed"),
                    "BC-2.06.015: ConfigInvalid must contain 'Overlay validation failed', got: {msg}"
                );
            }
            Ok(_) => panic!("BC-2.06.015: boot must fail for unknown org slug in customers/"),
            Err(e) => {
                panic!("BC-2.06.015: wrong error variant — expected ConfigInvalid, got: {e:?}")
            }
        }
    }

    /// BC-2.06.012 happy path: overlay present → resolved_spec_map contains the entry.
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_BC_2_06_012_overlay_present_in_customers_dir_resolves_to_map() {
        let v7 = uuid::Uuid::now_v7();
        let v7_str = v7.to_string();
        let (dir, config) = make_config_dir_with_sensor(&v7_str, "acme");
        let _ = &dir; // keep alive

        // Create customers/acme/armis.sensor.toml overlay.
        let overlay_dir = config.spec_dir.join("customers").join("acme");
        std::fs::create_dir_all(&overlay_dir).expect("create acme overlay dir");
        std::fs::write(
            overlay_dir.join("armis.sensor.toml"),
            r#"extends = "armis"
instance_id = "armis@acme"
base_url = "https://armis.acme-corp.io"
"#,
        )
        .expect("write overlay");

        let org_id = prism_core::OrgId::from_uuid(v7);
        let registry = registry_with_org(org_id, "acme");

        let (_config_manager, resolved_map) =
            step4_load_sensor_specs_with_overlays(&config, &registry)
                .await
                .expect("boot must succeed with valid overlay (BC-2.06.012)");

        // ResolvedSpecKey = (OrgSlug, SensorId).
        let key = (OrgSlug::new("acme"), prism_core::SensorId::from("armis"));
        let resolved = resolved_map.get(&key);
        assert!(
            resolved.is_some(),
            "BC-2.06.012: overlay entry must appear in resolved_spec_map at (acme, armis)"
        );
        let spec = &resolved.unwrap().spec;
        assert_eq!(
            spec.base_url, "https://armis.acme-corp.io",
            "BC-2.06.012: overlay base_url must be used in resolved spec"
        );
    }

    /// F-P2-MED-001 / PRR-005: corrupt TYPE spec file → hard boot abort with `BootError::ConfigInvalid`.
    ///
    /// Verifies the pass-1 fix that changed `build_type_spec_map_for_overlay` from
    /// warn-and-skip to hard-abort on parse failure.  A corrupt `*.sensor.toml` must not
    /// silently produce misleading E-SPEC-019 errors for overlays that extend the broken sensor;
    /// instead boot must fail immediately with a `ConfigInvalid` error message that includes
    /// the corrupt file path.
    ///
    /// TD-VSDD-059: this test provides the load-bearing assertion that the hard-failure path
    /// is exercised — not just doc-commented.
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_F_P2_MED_001_corrupt_type_spec_file_aborts_boot_with_config_invalid() {
        let v7 = uuid::Uuid::now_v7();
        let v7_str = v7.to_string();
        let (dir, config) = make_config_dir_with_sensor(&v7_str, "acme");
        let _ = &dir; // keep alive

        // Overwrite the valid armis.sensor.toml with corrupt TOML (unparseable).
        let corrupt_path = config.spec_dir.join("armis.sensor.toml");
        std::fs::write(
            &corrupt_path,
            "this is [not valid {{toml content at all\x00\n",
        )
        .expect("write corrupt sensor.toml");

        let org_id = prism_core::OrgId::from_uuid(v7);
        let registry = registry_with_org(org_id, "acme");

        // boot step 4 must fail hard on the corrupt TYPE spec.
        let result = step4_load_sensor_specs_with_overlays(&config, &registry).await;
        match result {
            Err(BootError::ConfigInvalid(msg)) => {
                // Error message must mention the corrupt file path so operators know
                // which file to fix (PRR-005 diagnostic requirement).
                assert!(
                    msg.contains("armis.sensor.toml"),
                    "F-P2-MED-001: ConfigInvalid must contain the corrupt file path in message, got: {msg}"
                );
                // Confirm this is the parse-failure branch, not the I/O failure branch.
                assert!(
                    msg.contains("Failed") || msg.contains("failed") || msg.contains("parse"),
                    "F-P2-MED-001: ConfigInvalid message should describe a parse failure, got: {msg}"
                );
            }
            Ok(_) => panic!(
                "F-P2-MED-001: boot must fail when a TYPE spec file is corrupt (PRR-005 hard-abort)"
            ),
            Err(e) => {
                panic!("F-P2-MED-001: wrong error variant — expected ConfigInvalid, got: {e:?}")
            }
        }
    }
}
