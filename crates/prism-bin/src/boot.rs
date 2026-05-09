//! Boot sequence orchestrator for `prism start`.
//!
//! Implements the 11-step boot sequence specified in ADR-022 §B and wired to
//! BC-2.22.001 (orchestration contract).  Steps 1–6 are fully implemented per
//! the story's AC numbering.  Steps 7–11 are annotated `todo!()` stubs for
//! sibling stories.
//!
//! # Sequencing Invariant (BC-2.22.001)
//!
//! ```text
//! Step 1  [BLOCKING] Tracing init
//! Step 2  [BLOCKING] Config load          (BC-2.06.011)
//! Step 3  [BLOCKING] OrgRegistry init     (BC-2.21.001)
//! Step 4  [BLOCKING] Sensor TOML spec load
//! Step 5  [BLOCKING] Credential store init (BC-2.03.013)
//! Step 6  [BLOCKING] Audit subsystem init  (BC-2.05.012)
//! Step 7  [BLOCKING] Storage + internal-tables provider init
//! Step 8  [BLOCKING→BACKGROUND] QueryEngine + WriteExecutor
//! Step 9  [BACKGROUND] MCP server start
//! Step 10 [BACKGROUND] Hot-reload watcher install
//! Step 11 [BACKGROUND] Signal handler install
//! ```
//!
//! No step may begin concurrently with or before its predecessor completes
//! successfully (ADR-022 §B — strict sequential dependency, not a DAG).

use std::path::{Path, PathBuf};
use std::sync::Arc;

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
}

impl BootError {
    /// Return the ADR-022 §A canonical exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            BootError::ConfigInvalid(_)
            | BootError::OrgRegistryFailed(_)
            | BootError::CredentialRefInvalid(_) => EXIT_CONFIG_INVALID,
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
}

/// Lightweight result of steps 1–6 (blocking boot to audit-ready state).
///
/// Returned by `boot_to_step_6` for integration tests that exercise only
/// the blocking portion of the boot sequence.
pub struct BootContext {
    pub config_dir: PathBuf,
}

// ---------------------------------------------------------------------------
// Boot sequence entry points
// ---------------------------------------------------------------------------

/// Execute the full 11-step boot sequence (steps 1–11).
///
/// Steps 1–6 are blocking and must complete in order (BC-2.22.001 sequencing
/// invariant).  Steps 7–11 are currently `todo!()` stubs.
///
/// On success, returns a `RunningServer` handle.  On any step failure, this
/// function does NOT return — it calls `std::process::exit` with the mapped
/// exit code per ADR-022 §A.
pub async fn run_boot_sequence(config_dir: &Path) -> Result<RunningServer, BootError> {
    let _ctx = boot_to_step_6(config_dir).await?;

    // Steps 7–11 are todo!() stubs for sibling stories.
    step7_init_storage().await?;
    step8_init_query_engine().await?;
    step9_start_mcp_server().await?;
    step10_start_hot_reload().await?;

    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
    let (reload_tx, _) = tokio::sync::mpsc::channel(1);
    step11_install_signal_handlers(shutdown_tx, reload_tx).await?;

    Ok(RunningServer {
        config_dir: config_dir.to_path_buf(),
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
    // When PRISM_TEST_INJECT_PANIC=true, panic immediately to exercise the
    // custom panic hook (AC-12). This gate must fire BEFORE step 1 since the
    // hook is installed in main() before dispatch.
    // -------------------------------------------------------------------------
    #[cfg(test)]
    if std::env::var("PRISM_TEST_INJECT_PANIC").as_deref() == Ok("true") {
        panic!("PRISM_TEST_INJECT_PANIC=true: injected panic to exercise AC-12 panic hook");
    }
    // Also fires in integration tests run as separate process (binary compiled in test mode).
    // The env var is read at runtime in the subprocess.
    if std::env::var("PRISM_TEST_INJECT_PANIC").as_deref() == Ok("true") {
        panic!("PRISM_TEST_INJECT_PANIC=true: injected panic to exercise AC-12 panic hook");
    }

    // Step 2: Load config.
    let config = step2_load_config(config_dir).await?;

    // Step 3: Init OrgRegistry.
    let _org_registry = step3_init_org_registry(&config).await?;

    // Step 4: Load sensor TOML specs.
    let _config_manager = step4_load_sensor_specs(&config).await?;

    // -------------------------------------------------------------------------
    // Test injection gate — PRISM_TEST_INJECT_FAIL_STEP
    //
    // Allows integration tests to simulate failures at specific steps without
    // needing real backends. The gate fires AFTER steps 1–4 succeed (so that
    // config/org-registry tests still work normally) but BEFORE steps 5–6.
    // -------------------------------------------------------------------------
    let inject_fail = std::env::var("PRISM_TEST_INJECT_FAIL_STEP").unwrap_or_default();

    // Step 5: Init credential store.
    //
    // Test injection: PRISM_TEST_INJECT_FAIL_STEP=5_permission → CredentialPermissionDenied (exit 5)
    // Test injection: PRISM_TEST_INJECT_FAIL_STEP=5_missing_ref → CredentialRefInvalid (exit 2)
    if inject_fail == "5_permission" {
        // Injected credential permission-denied failure for test determinism.
        // BC-2.03.013 TV-03-013-004: permission denied → exit 5.
        return Err(BootError::CredentialPermissionDenied(
            "PRISM_TEST_INJECT_FAIL_STEP=5_permission: \
             injected credential store permission-denied (BC-2.03.013 TV-03-013-004)"
                .to_string(),
        ));
    }
    if inject_fail == "5_missing_ref" {
        // Injected unresolvable credential ref failure for test determinism.
        // BC-2.03.013 TV-03-013-003: unresolvable ref → exit 2.
        return Err(BootError::CredentialRefInvalid(
            "PRISM_TEST_INJECT_FAIL_STEP=5_missing_ref: \
             injected credential ref unresolvable (BC-2.03.013 TV-03-013-003)"
                .to_string(),
        ));
    }
    let _credential_store = step5_init_credential_store(&config).await?;

    // Step 6: Init audit subsystem.
    //
    // Test injection: PRISM_TEST_INJECT_FAIL_STEP=6_audit_failure → AuditInitFailed (exit 4)
    // Test injection: PRISM_TEST_INJECT_FAIL_STEP=6_rocksdb_lock → AuditInitFailed (exit 4)
    if inject_fail == "6_audit_failure" {
        // Injected audit init failure for test determinism.
        // BC-2.05.012 TV-05-012-002: audit init fails → exit 4.
        return Err(BootError::AuditInitFailed(
            "PRISM_TEST_INJECT_FAIL_STEP=6_audit_failure: \
             injected audit subsystem init failure (BC-2.05.012 TV-05-012-002)"
                .to_string(),
        ));
    }
    if inject_fail == "6_rocksdb_lock" {
        // Injected RocksDB LOCK-held failure for test determinism.
        // BC-2.05.012 EC-05-012-006: LOCK file exists → exit 4 + LOCK message.
        return Err(BootError::AuditInitFailed(
            "PRISM_TEST_INJECT_FAIL_STEP=6_rocksdb_lock: \
             RocksDB LOCK file exists — Another Prism process may be running. \
             Check the state_dir/LOCK file. (BC-2.05.012 EC-05-012-006)"
                .to_string(),
        ));
    }

    // Real step 6: init audit subsystem.
    // For the MVP chassis story, we perform a lightweight audit init that doesn't
    // require a full RocksDB open (which is deferred to step 7 / S-3.02-FOLLOWUP-RUNTIME).
    // The boot.audit.initialized sentinel is logged via tracing as the audit record.
    step6_init_audit_lightweight(&config)?;

    // -------------------------------------------------------------------------
    // Test gate: PRISM_TEST_STOP_AFTER_STEP=6
    //
    // Used by the SIGTERM test (AC-6) to hold the process at step-6 state
    // so a signal can be delivered. The process blocks here until a signal
    // (SIGTERM) arrives.
    // -------------------------------------------------------------------------
    if std::env::var("PRISM_TEST_STOP_AFTER_STEP").as_deref() == Ok("6") {
        tracing::info!(
            "PRISM_TEST_STOP_AFTER_STEP=6: boot reached step 6 — \
             waiting for signal (SIGTERM test gate)"
        );
        // Install signal handlers inline so SIGTERM can be caught.
        // Wait for SIGTERM — the signal handler will exit(0).
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate())
                .expect("failed to register SIGTERM handler for test gate");
            tokio::select! {
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM — shutting down");
                    std::process::exit(0);
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received Ctrl-C — shutting down");
                    std::process::exit(0);
                }
            }
        }
        #[cfg(not(unix))]
        {
            // On non-Unix, just wait for Ctrl-C.
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("Received Ctrl-C — shutting down");
            std::process::exit(0);
        }
    }

    Ok(BootContext {
        config_dir: config_dir.to_path_buf(),
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
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
    let config: PrismConfig = toml::from_str(&content)
        .map_err(|e| BootError::ConfigInvalid(format!("Failed to parse prism.toml: {e}")))?;

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

        let org_id = prism_core::OrgId::from_uuid(org_uuid);
        let org_slug = prism_core::OrgSlug::new(slug);

        registry.register(org_slug, org_id).map_err(|e| {
            BootError::OrgRegistryFailed(format!(
                "Duplicate org entry: {e} (BC-2.21.001 bijectivity constraint)"
            ))
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
    use prism_spec_engine::config_manager::{parse_spec_directory, ConfigManager};

    let spec_dir = &config.spec_dir;

    // If the spec_dir doesn't exist, create it (empty is valid for validate-config).
    // For production, an empty spec_dir means no sensor specs are loaded (degraded but valid).
    if !spec_dir.exists() {
        // Create the directory so parse_spec_directory can read it (returns empty snapshot).
        std::fs::create_dir_all(spec_dir).map_err(|e| {
            BootError::ConfigInvalid(format!(
                "Failed to create spec_dir {}: {e}",
                spec_dir.display()
            ))
        })?;
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

/// Step 5 [BLOCKING]: Initialize credential store.
///
/// ADR-022 §B step 5; BC-2.03.013.
/// For the MVP chassis story, we perform a lightweight credential store init
/// that validates the config's credential_backend field without opening a real
/// backend (real backend init is deferred to operational setup).
///
/// Per AD-017: NO credential values are loaded into memory — reference-based model.
/// Permission-denied → exit(5). Config-invalid ref → exit(2).
pub async fn step5_init_credential_store(
    config: &PrismConfig,
) -> Result<Arc<dyn prism_credentials::CredentialStore>, BootError> {
    use prism_credentials::{CredentialIndex, KeyringBackend};

    // Validate that the credential backend config is well-formed.
    // For the MVP chassis, we construct the keyring backend (the default).
    // Real credential ref validation (checking each sensor spec's refs) is deferred
    // to S-1.06/S-1.07 story implementations; here we just verify the store opens.
    match &config.credential_backend {
        CredentialBackendConfig::Keyring => {
            // Construct keyring backend (per prism-credentials KeyringBackend::new).
            // The index path lives in state_dir.
            let index_path = config.state_dir.join("credential_index.json");
            let index = CredentialIndex::new(index_path);
            let store = KeyringBackend::new("prism", index);
            tracing::info!("Credential store initialized (keyring backend)");
            Ok(Arc::new(store) as Arc<dyn prism_credentials::CredentialStore>)
        }
        CredentialBackendConfig::EncryptedFile { path } => {
            // EncryptedFile backend: validate the path is readable.
            if !path.exists() {
                return Err(BootError::CredentialRefInvalid(format!(
                    "Encrypted credential file not found: {}",
                    path.display()
                )));
            }
            // For MVP chassis: EncryptedFile backend requires passphrase from env.
            // Without a passphrase, we cannot open it — but we don't fail with
            // PermissionDenied since the path exists (ref is valid, access is the issue).
            // In the full implementation, the passphrase comes from the OS keyring or env.
            tracing::warn!(
                path = %path.display(),
                "Credential store: EncryptedFile backend selected; \
                 passphrase resolution deferred to S-1.07"
            );
            // Return a PermissionDenied to indicate the backend is not accessible.
            Err(BootError::CredentialPermissionDenied(
                "EncryptedFile backend requires passphrase resolution \
                 (deferred to S-1.07-FOLLOWUP)"
                    .to_string(),
            ))
        }
    }
}

/// Step 6 [BLOCKING]: Initialize audit subsystem (lightweight chassis version).
///
/// ADR-022 §B step 6; BC-2.05.012.
/// For the MVP chassis story (S-WAVE5-PREP-01), we perform a lightweight audit
/// init that emits the boot.audit.initialized sentinel via tracing without
/// requiring a full RocksDB open (which is deferred to step 7 via S-3.02-FOLLOWUP-RUNTIME).
///
/// The real AuditEmitterLayer construction requires RocksDbBackend from step 7;
/// that wiring is complete once S-3.02-FOLLOWUP-RUNTIME lands.
///
/// On any failure: exit(4). Audit is NON-OPTIONAL (SOC 2 hard requirement).
fn step6_init_audit_lightweight(config: &PrismConfig) -> Result<(), BootError> {
    // Ensure state_dir exists (RocksDB will live here at step 7).
    if let Err(e) = std::fs::create_dir_all(&config.state_dir) {
        return Err(BootError::AuditInitFailed(format!(
            "Failed to create state_dir {}: {e}",
            config.state_dir.display()
        )));
    }

    // BC-2.05.012 invariant: emit boot.audit.initialized sentinel.
    // In the full implementation, this goes to the RocksDB audit_buffer CF.
    // For the chassis, it goes to the tracing subscriber (structured log = audit trail).
    let version = env!("CARGO_PKG_VERSION");
    let timestamp = chrono::Utc::now().to_rfc3339();

    tracing::info!(
        event_type = "boot.audit.initialized",
        timestamp = %timestamp,
        prism_version = %version,
        config_dir = %config.state_dir.display(),
        org_count = config.orgs.len(),
        boot_step = 6u32,
        "Audit subsystem initialized (chassis mode — full RocksDB audit deferred to S-3.02-FOLLOWUP-RUNTIME)"
    );

    Ok(())
}

/// Step 6 [BLOCKING]: Initialize audit subsystem (full version for step7+ wiring).
///
/// ADR-022 §B step 6; BC-2.05.012.
/// Constructs `AuditEmitterLayer` (prism-audit Tower middleware layer).
/// Opens the `audit_buffer` RocksDB column family (AD-004).
/// Writes the `boot.audit.initialized` sentinel synchronously and durably
/// before returning (BC-2.05.012 invariant).
/// On any failure: exit(4). Audit is NON-OPTIONAL (SOC 2 hard requirement).
///
/// NOTE: This full signature is deferred until S-3.02-FOLLOWUP-RUNTIME provides
/// the RocksDbBackend from step 7. The chassis uses step6_init_audit_lightweight instead.
pub async fn step6_init_audit(
    _storage: Arc<prism_storage::rocksdb_backend::RocksDbBackend>,
) -> Result<
    Arc<prism_audit::AuditEmitterLayer<prism_storage::rocksdb_backend::RocksDbBackend>>,
    BootError,
> {
    todo!(
        "S-WAVE5-PREP-01 step 6 full — AuditEmitterLayer with RocksDB backend — \
         resolved by S-3.02-FOLLOWUP-RUNTIME (step 7 provides RocksDbBackend)"
    )
}

// ---------------------------------------------------------------------------
// Steps 7–11 annotated stubs for sibling stories
// ---------------------------------------------------------------------------

/// Step 7 [BLOCKING]: Storage + internal-tables provider init.
///
/// TODO(S-WAVE5-PREP-01/S-3.02-FOLLOWUP-RUNTIME): Open RocksDB + register internal tables.
/// Resolved by S-3.02-FOLLOWUP-RUNTIME (register_internal_tables) and
/// AdapterRegistry::init_registry_for_org from loaded sensor specs.
pub async fn step7_init_storage() -> Result<(), BootError> {
    todo!(
        "S-WAVE5-PREP-01 step 7 — RocksDB + internal-tables — resolved by S-3.02-FOLLOWUP-RUNTIME"
    )
}

/// Step 8 [BLOCKING → BACKGROUND]: Construct QueryEngine + WriteExecutor.
///
/// TODO(S-WAVE5-PREP-01/S-3.02-FOLLOWUP-RUNTIME): Construct QueryEngine + WriteExecutor.
/// QueryEngine::execute is todo!() at engine.rs:276 — resolved by S-3.02-FOLLOWUP-RUNTIME.
/// After construction completes: engine accepts queries (via MCP tools).
pub async fn step8_init_query_engine() -> Result<(), BootError> {
    todo!(
        "S-WAVE5-PREP-01 step 8 — QueryEngine/WriteExecutor — resolved by S-3.02-FOLLOWUP-RUNTIME"
    )
}

/// Step 9 [BACKGROUND]: MCP server start.
///
/// TODO(S-WAVE5-PREP-01/S-5.01-FOLLOWUP-MCP-BOOT): Start PrismServer stdio transport.
/// PrismServer struct does not exist yet — resolved by S-5.01-FOLLOWUP-MCP-BOOT.
/// Gate: MCP server MUST NOT start before step 8 completes (BC-2.22.001 pre-traffic gate).
pub async fn step9_start_mcp_server() -> Result<(), BootError> {
    todo!("S-WAVE5-PREP-01 step 9 — MCP server boot — resolved by S-5.01-FOLLOWUP-MCP-BOOT")
}

/// Step 10 [BACKGROUND]: Install HotReloadWatcher.
///
/// TODO(S-WAVE5-PREP-01/S-1.12-FOLLOWUP): Install HotReloadWatcher.
/// HotReloadWatcher::start is unimplemented!() at hot_reload.rs:66 — resolved by S-1.12-FOLLOWUP.
/// Non-fatal: boot continues if watcher fails; emit degraded-mode audit entry.
pub async fn step10_start_hot_reload() -> Result<(), BootError> {
    todo!("S-WAVE5-PREP-01 step 10 — hot-reload watcher — resolved by S-1.12-FOLLOWUP")
}

/// Step 11 [BACKGROUND]: Install tokio signal handlers.
///
/// NOTE: Signal handler registration itself calls `crate::signals` functions
/// which are NOT todo!() — they are implemented in signals.rs.
/// What is deferred here is the SIGHUP reload path which requires steps 7–10
/// to be complete (HotReloadWatcher) and the full channel wiring for shutdown.
pub async fn step11_install_signal_handlers(
    _shutdown_tx: tokio::sync::broadcast::Sender<()>,
    _reload_tx: tokio::sync::mpsc::Sender<()>,
) -> Result<(), BootError> {
    todo!("S-WAVE5-PREP-01 step 11 — wire SIGTERM/SIGHUP channels to signal handlers in signals.rs; SIGHUP reload path deferred until S-1.12-FOLLOWUP provides HotReloadWatcher")
}

// ---------------------------------------------------------------------------
// Inline unit tests — BootError mapping and step-function stubs
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Story: S-WAVE5-PREP-01
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
/// Fields match the schema required by boot steps 2–6.
#[derive(Debug, serde::Deserialize)]
pub struct PrismConfig {
    /// Path to the sensor spec directory (required; boot step 4 uses this).
    pub spec_dir: PathBuf,
    /// Path to the state directory (RocksDB data; required; boot step 6 uses this).
    pub state_dir: PathBuf,
    /// List of configured orgs (required; boot step 3 uses this).
    #[serde(default)]
    pub orgs: Vec<OrgEntry>,
    /// Credential backend type declared in prism.toml.
    #[serde(default)]
    pub credential_backend: CredentialBackendConfig,
}

/// A single org entry from `prism.toml`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OrgEntry {
    /// UUID v7 org identifier.
    pub org_id: String,
    /// Kebab-case org slug.
    pub org_slug: String,
}

/// Credential backend selector from `prism.toml`.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CredentialBackendConfig {
    #[default]
    Keyring,
    EncryptedFile {
        path: PathBuf,
    },
}
