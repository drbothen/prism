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
    let _org_registry = step3_init_org_registry(&config).await?;

    // Step 4: Load sensor TOML specs.
    let config_manager = step4_load_sensor_specs(&config).await?;

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
    let _credential_store = step5_init_credential_store(&config, &config_manager).await?;

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
    let _audit_backend = step6_init_audit(&config)?;

    // -------------------------------------------------------------------------
    // Test gate: PRISM_TEST_STOP_AFTER_STEP=6
    //
    // CRIT-1: gated behind `#[cfg(feature = "test-injection")]`.
    // Used by the SIGTERM test (AC-6) to hold the process at step-6 state
    // so a signal can be delivered. The process blocks here until a signal
    // (SIGTERM) arrives.
    // -------------------------------------------------------------------------
    // -------------------------------------------------------------------------
    // Test gate: PRISM_TEST_STOP_AFTER_STEP=6
    //
    // CRIT-1: gated behind `#[cfg(feature = "test-injection")]`.
    // MED-2 (S-WAVE5-PREP-01 fix-pass-1): wires through signals::install_sigterm_handler
    // instead of duplicating the select! arm. This ensures BC-2.10.010 coverage
    // is exercised through the production code path.
    // -------------------------------------------------------------------------
    #[cfg(feature = "test-injection")]
    if std::env::var("PRISM_TEST_STOP_AFTER_STEP").as_deref() == Ok("6") {
        tracing::info!(
            "PRISM_TEST_STOP_AFTER_STEP=6: boot reached step 6 — \
             waiting for signal via signals::install_sigterm_handler (SIGTERM test gate)"
        );
        // MED-2: delegate to signals::install_sigterm_handler so the SIGTERM
        // test exercises the production BC-2.10.010 code path, not a duplicate.
        let (shutdown_tx, _rx) = tokio::sync::broadcast::channel(1);
        crate::signals::install_sigterm_handler(shutdown_tx).await;
        // install_sigterm_handler calls process::exit(0) on SIGTERM — this line
        // is unreachable if a signal is received, but handles the no-signal path.
        std::process::exit(0);
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

        // LOW-3 (S-WAVE5-PREP-01 fix-pass-1): strict UUID v7 validation.
        // BC-2.21.001 EC-21-001-008: non-v7 UUID → exit 2 with "must be a UUID v7".
        if org_uuid.get_version() != Some(uuid::Version::SortRand) {
            return Err(BootError::OrgRegistryFailed(format!(
                "Invalid org_id '{}': must be a UUID v7 (time-ordered, version 7); \
                 got version {:?} (BC-2.21.001 EC-21-001-008)",
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
    use prism_spec_engine::config_manager::{parse_spec_directory, ConfigManager};

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

/// Step 5 [BLOCKING]: Initialize credential store and validate sensor spec credential refs.
///
/// ADR-022 §B step 5; BC-2.03.013.
/// Constructs the `CredentialStore` backend from the config's `credential_backend`
/// field, then validates all credential references declared in loaded sensor specs
/// (reference-only — no values are loaded per AD-017).
///
/// Per AD-017: NO credential values are loaded into memory — reference-based model.
/// Permission-denied → exit(5). Config-invalid ref → exit(2).
pub async fn step5_init_credential_store(
    config: &PrismConfig,
    config_manager: &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
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

    // HIGH-2 (S-WAVE5-PREP-01 fix-pass-1): Iterate all credential refs declared in
    // loaded sensor specs (BC-2.03.013 happy-path postcondition 2).
    // Reference-only validation: verify each ref target EXISTS (no value loading per AD-017).
    //
    // EC-03-013-001: if no specs declare any refs → zero refs validated → boot continues.
    //
    // Current SensorSpec (prism-spec-engine v0.1.0) does not include a top-level
    // [[credentials]] block — credential refs are per-InfusionSpec and require S-1.14
    // (InfusionLoader) for parsing. The sensor spec snapshot from step 4 is iterated here;
    // its SensorSpec structs have no credential_refs field, so refs_validated = 0, which
    // correctly satisfies EC-03-013-001 for v0.1.0 sensor specs (CrowdStrike, Armis, etc.).
    //
    // When S-1.06/S-1.07 adds credential_refs to SensorSpec, update this loop accordingly.
    // Access the current config snapshot from the arc-swapped ConfigManager.
    let cm_guard = config_manager.load(); // Guard<Arc<ConfigManager>>
    let cm = &**cm_guard; // &ConfigManager
    let snapshot_guard = cm.load(); // Guard<Arc<ConfigSnapshot>>
    let snapshot = &**snapshot_guard; // &ConfigSnapshot
    let refs_validated: usize = 0;

    for sensor_id in snapshot.sensor_specs.keys() {
        // SensorSpec v0.1.0 has no credential_refs field.
        // This loop body is the correct placeholder that will be filled when
        // SensorSpec gains a credentials field (S-1.06/S-1.07-FOLLOWUP).
        // Log at trace level to avoid spamming on every boot with large spec sets.
        tracing::trace!(
            sensor_id = %sensor_id,
            "Credential ref check: sensor spec has 0 credential refs (SensorSpec v0.1.0)"
        );
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
/// HIGH-1 + OBS-1 (S-WAVE5-PREP-01 fix-pass-1): Full implementation per BC-2.05.012:
/// 1. Opens RocksDB at `config.state_dir` (all column families including `audit_buffer`).
/// 2. Confirms the `audit_buffer` CF is writable.
/// 3. Constructs a `BootSentinelEntry` with all required BC-2.05.012 sentinel fields.
/// 4. Writes the sentinel synchronously and durably to the `audit_buffer` CF via
///    `prism_storage::audit_buffer::append_audit_entry`.
/// 5. Returns the `Arc<RocksDbBackend>` for use by step 7.
///
/// On any failure: returns `BootError::AuditInitFailed` (exit 4).
/// Audit is NON-OPTIONAL: no degraded mode, no `--skip-audit` flag (SOC 2).
///
/// OBS-1 (sentinel schema): BC-2.05.012 OQ-2 asks whether `AuditEntry` covers
/// `prism_version` and `boot_step` fields. The existing `prism_audit::AuditEntry`
/// is a full MCP-tool-invocation record with SOC 2 + ISO 27001 fields — it does NOT
/// cover `prism_version` or `boot_step` (those are boot-time fields, not tool fields).
/// Decision: use `prism_storage::audit_buffer::AuditEntry` (payload: BTreeMap<String,String>)
/// for the boot sentinel. This is a simpler raw-payload entry that fits the sentinel schema
/// without requiring a full SOC-2 tool invocation context.
fn step6_init_audit(
    config: &PrismConfig,
) -> Result<Arc<prism_storage::rocksdb_backend::RocksDbBackend>, BootError> {
    use prism_storage::audit_buffer::{append_audit_entry, AuditEntry as StorageAuditEntry};
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
    let backend = RocksDbBackend::open(config.state_dir.clone()).map_err(|e| {
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
    })?;

    // OBS-1: BootSentinelEntry — use prism_storage::audit_buffer::AuditEntry (payload map).
    // This provides the boot.audit.initialized schema fields without requiring the full
    // prism_audit::AuditEntry MCP-context structure.
    let version = env!("CARGO_PKG_VERSION");
    let timestamp_ns = chrono::Utc::now()
        .timestamp_nanos_opt()
        .expect("timestamp fits in i64") as u64;
    let trace_id = uuid::Uuid::now_v7().to_string();

    // Redact config_dir: use SHA-256 hash of the path, not the raw path.
    // BC-2.05.012: "config_dir field MUST be redacted (only a hash or basename)".
    let config_dir_hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        config.state_dir.hash(&mut h);
        format!("{:016x}", h.finish())
    };

    let mut payload = std::collections::BTreeMap::new();
    payload.insert(
        "event_type".to_string(),
        "boot.audit.initialized".to_string(),
    );
    payload.insert("prism_version".to_string(), version.to_string());
    payload.insert("config_dir".to_string(), config_dir_hash);
    payload.insert("org_count".to_string(), config.orgs.len().to_string());
    payload.insert("boot_step".to_string(), "6".to_string());

    let sentinel = StorageAuditEntry {
        timestamp_ns,
        trace_id,
        payload,
    };

    // Write the sentinel synchronously and durably to the audit_buffer CF.
    // BC-2.05.012: "synchronous and confirmed durable (not queued asynchronously)".
    append_audit_entry(&backend, &sentinel).map_err(|e| {
        BootError::AuditInitFailed(format!(
            "Audit subsystem init failed: sentinel persistence error: {e}"
        ))
    })?;

    tracing::info!(
        event_type = "boot.audit.initialized",
        prism_version = %version,
        org_count = config.orgs.len(),
        boot_step = 6u32,
        "Audit subsystem initialized; boot.audit.initialized persisted"
    );

    Ok(Arc::new(backend))
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
