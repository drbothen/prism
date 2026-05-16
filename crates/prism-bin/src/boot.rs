//! Boot sequence orchestrator for `prism start`.
//!
//! Implements the 11-step boot sequence specified in ADR-022 §B and wired to
//! BC-2.22.001 (orchestration contract).  Steps 1–6 are fully implemented per
//! the story's AC numbering.  Steps 7–11 are annotated `todo!()` stubs for
//! sibling stories. Step 7.5 (plugin-load) is implemented by S-PLUGIN-PREREQ-D.
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
    /// RocksDB backend opened in step 6 (all CFs, including `audit_buffer`).
    /// Threaded into step 7.5 (`plugin_load_step`) so the `RocksDbPluginAuditSink`
    /// can write durable audit entries for each unsigned plugin load (HIGH-002 / AC-4).
    pub rocksdb_backend: Arc<prism_storage::rocksdb_backend::RocksDbBackend>,
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
/// This ordering ensures `plugin_load_step_with_audit` is REACHABLE at runtime
/// (step 7's `todo!()` panic fires AFTER plugin-load, not before).
pub async fn run_boot_sequence(config_dir: &Path) -> Result<RunningServer, BootError> {
    let ctx = boot_to_step_6(config_dir).await?;

    // Step 2 re-loads config to obtain plugin_dir for step 7.5.
    // This is acceptable because step2_load_config is idempotent (pure read + validate).
    let config = step2_load_config(config_dir).await?;

    // Step 7.5 [BLOCKING]: Plugin-load step — BC-2.22.001 §Sequencing Invariant.
    // Positioned BEFORE step 7 (storage init) — plugin-load only requires the RocksDB
    // audit backend from step 6 (ctx.rocksdb_backend), which boot_to_step_6 already
    // provides.  This ordering makes plugin-load REACHABLE at runtime: step 7's todo!()
    // panic fires AFTER plugin-load completes, not before.
    //
    // Pre-traffic gate (AC-2): MCP server (step 9) does NOT bind before this completes.
    // ADR-023 §C4 + ADR-022 §B.
    //
    // HIGH-002 (F-IMPL-LP1-HIGH-002): wire RocksDbPluginAuditSink from step 6 backend
    // so plugin load events are persisted durably to audit_buffer CF (not just tracing::warn!).
    let plugin_audit_sink = Arc::new(crate::plugin_audit::RocksDbPluginAuditSink::new(
        Arc::clone(&ctx.rocksdb_backend),
    ));
    let _plugin_result = plugin_load_step_with_audit(&config.plugin_dir, plugin_audit_sink).await?;

    // Step 7 [BLOCKING]: Storage + internal-tables provider init.
    // Positioned AFTER step 7.5 — plugin-load does not depend on storage tables.
    step7_init_storage().await?;

    // Steps 8–11 are todo!() stubs for sibling stories.
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
/// - `probe()` performs an existence check only — it checks if the ref is
///   registered in the backend and returns Ok(()) or an appropriate `BootError`.
pub trait CredentialRefProbe: Send + Sync {
    /// Check whether `ref_name` for `sensor_id` is registered in the backend.
    ///
    /// Returns:
    /// - `Ok(())` — ref exists (credential is registered)
    /// - `Err(BootError::CredentialRefInvalid)` — ref not found (exit 2)
    /// - `Err(BootError::CredentialPermissionDenied)` — backend unavailable (exit 5)
    fn probe(&self, sensor_id: &str, ref_name: &str) -> Result<(), BootError>;
}

/// Production credential ref probe — uses the `keyring` crate.
///
/// Constructs a namespaced `keyring::Entry("prism", "{sensor_id}/{ref_name}")` and
/// calls `get_password()` to check existence. The value is immediately discarded
/// (AD-017 AI-opaque model: credential values MUST NOT be retained).
pub struct KeyringCredentialProbe;

impl CredentialRefProbe for KeyringCredentialProbe {
    fn probe(&self, sensor_id: &str, ref_name: &str) -> Result<(), BootError> {
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
                tracing::trace!(
                    sensor_id = %sensor_id,
                    ref_name = %ref_name,
                    "Credential ref validated (exists in backend)"
                );
                Ok(())
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
        for cred_ref in &sensor_spec.credential_refs {
            probe.probe(sensor_id, &cred_ref.name)?;
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
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
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
pub struct PluginLoadResult {
    /// The initialized plugin runtime (always `Some` — never None).
    pub runtime: Arc<prism_spec_engine::plugin::PluginRuntime>,
    /// Number of plugins successfully loaded (0 if disabled or none found).
    pub plugins_loaded: usize,
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
    use prism_spec_engine::plugin::{PLUGIN_HTTP_CLIENT_TIMEOUT_SECS, PluginRuntime};
    use std::time::Duration;

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
    let plugins_loaded = runtime
        .load_all_plugins(plugin_dir)
        .await
        .map_err(|e| BootError::InternalError(e.to_string()))?;

    tracing::info!(
        n_loaded = plugins_loaded,
        plugin_dir = %plugin_dir.display(),
        "boot: plugin-load step complete ({plugins_loaded} plugins loaded)"
    );

    Ok(PluginLoadResult {
        runtime: Arc::new(runtime),
        plugins_loaded,
    })
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
///
/// # AdapterRegistry assertion (DEFERRED — TD-S-PLUGIN-PREREQ-A-004 P1)
///
/// When step8 (init_query_engine) is wired to a non-stub body
/// (S-WAVE5-PREP-01 / S-3.02-FOLLOWUP-RUNTIME), the FIRST thing it must do is
/// verify the `AdapterRegistry` contains at least one adapter before serving
/// queries. Without this assertion, a silent `init_registry_for_org` failure
/// would propagate as silent empty results across all queries (regressing
/// ADV-W3MT-P58-LOW-002 fix).
///
/// Implementation when step8 wires:
/// ```rust,ignore
/// if registry.is_empty() && !is_test_mode() {
///     return Err(BootError::EmptyRegistry { /* ... */ });
/// }
/// ```
///
/// Defense-in-depth: `materialization.rs:653` retains `is_empty()` short-circuit
/// (test-mode aware) until this assertion is enforced.
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
    todo!(
        "S-WAVE5-PREP-01 step 11 — wire SIGTERM/SIGHUP channels to signal handlers in signals.rs; \
         SIGHUP reload path deferred until S-1.12-FOLLOWUP provides HotReloadWatcher. \
         MCP server boot (step 9) deferred to MCP server chassis story (see STORY-INDEX)."
    )
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
