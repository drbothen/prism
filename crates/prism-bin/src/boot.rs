//! Boot sequence orchestrator for `prism start`.
//!
//! Implements the 11-step boot sequence specified in ADR-022 §B and wired to
//! BC-2.22.001 (orchestration contract).  Steps 1–6 are stubs here (Red Gate);
//! the implementer replaces each `todo!()` with real logic per the story's AC
//! numbering.  Steps 7–11 are annotated `todo!()` stubs for sibling stories.
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
/// Fields are placeholders — the implementer replaces `()` with real types.
pub struct RunningServer {
    /// Resolved config directory used during boot.
    pub config_dir: PathBuf,
    // Additional fields are added by the implementer as real types become
    // available from steps 7–11 sibling stories.
}

/// Lightweight result of steps 1–6 (blocking boot to audit-ready state).
///
/// Returned by `boot_to_step_6` for integration tests that exercise only
/// the blocking portion of the boot sequence.
pub struct BootContext {
    pub config_dir: PathBuf,
    // Implementer populates: PrismConfig, OrgRegistry, CredentialStore,
    // AuditEmitterLayer, ConfigManager, Arc<ArcSwap<ConfigManager>> once
    // sibling types stabilize their APIs.
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
/// The caller in `main.rs` should NOT handle errors from this function with `?`
/// — exit-code mapping is done internally to preserve the invariant that every
/// exit code is exactly the canonical code for its failure class.
pub async fn run_boot_sequence(_config_dir: &Path) -> Result<RunningServer, BootError> {
    todo!("S-WAVE5-PREP-01: execute 11-step boot sequence per ADR-022 §B; satisfy BC-2.22.001 sequencing invariant — steps 1-6 then steps 7-11 stubs")
}

/// Execute boot steps 1–6 only (blocking, audit-ready state).
///
/// Used by `validate-config` subcommand and integration tests that verify
/// the blocking boot path without entering the MCP serving loop.
///
/// BC-2.22.001: Steps 1–6 must complete in order before this returns.
/// The sequencing invariant is enforced by sequential await; no step is
/// started concurrently with its predecessor.
pub async fn boot_to_step_6(_config_dir: &Path) -> Result<BootContext, BootError> {
    todo!("S-WAVE5-PREP-01: execute boot steps 1-6 blocking per ADR-022 §B; returns BootContext for validate-config and integration tests")
}

// ---------------------------------------------------------------------------
// Individual boot steps (stubs — implementer fills each one)
// ---------------------------------------------------------------------------

/// Step 1 [BLOCKING]: Initialize tracing subscriber.
///
/// ADR-022 §B step 1.  Must run FIRST, before any other boot step or log
/// output.  On failure, emits to stderr and calls `std::process::exit(4)`.
///
/// Format: JSON if `PRISM_LOG_FORMAT=json`; pretty otherwise.
/// First log line: `tracing::info!("Prism v{}", env!("CARGO_PKG_VERSION"))`.
pub fn step1_init_tracing(_log_format: &crate::cli::LogFormat) {
    todo!("S-WAVE5-PREP-01 step 1 — init tracing subscriber with EnvFilter + JSON/pretty per PRISM_LOG_FORMAT; emit Prism version as first log line")
}

/// Step 2 [BLOCKING]: Load `prism.toml` from config directory.
///
/// ADR-022 §B step 2; BC-2.06.011.
/// Reads `prism.toml`, deserializes via serde/toml, validates schema.
/// `$PRISM_CONFIG_DIR` always overrides the default; does NOT fall back to
/// default if the env var points to a non-existent directory (BC-2.06.011 invariant).
///
/// Returns a `PrismConfig` placeholder (real type added by implementer).
/// On failure, calls `std::process::exit(2)`.
pub async fn step2_load_config(_config_dir: &Path) -> Result<PrismConfig, BootError> {
    todo!("S-WAVE5-PREP-01 step 2 — load prism.toml from config_dir; deserialize + validate schema per BC-2.06.011; exit(2) on any failure; PRISM_CONFIG_DIR must not fall back to default")
}

/// Step 3 [BLOCKING]: Construct `OrgRegistry` from config.
///
/// ADR-022 §B step 3; BC-2.21.001.
/// Builds a bijective (org_id, org_slug) registry and verifies uniqueness.
/// Empty org list, duplicate org_id, duplicate org_slug, malformed slug →
/// exit(2) with a descriptive message.
///
/// "Config must declare at least one org" is the AC-9 required message for
/// the empty-list case.
pub async fn step3_init_org_registry(
    _config: &PrismConfig,
) -> Result<Arc<prism_core::OrgRegistry>, BootError> {
    todo!("S-WAVE5-PREP-01 step 3 — construct OrgRegistry from config org list; verify bijectivity per BC-2.21.001; exit(2) on empty list / duplicate / malformed slug")
}

/// Step 4 [BLOCKING]: Load sensor TOML specs.
///
/// ADR-022 §B step 4; ADR-022 §C ConfigManager wiring contract.
/// Calls `parse_spec_directory(config.spec_dir)` → `ConfigSnapshot`.
/// Wraps in `Arc<ArcSwap<ConfigManager>>` for hot-reload support (AD-007).
/// This is the FIRST production call site for `parse_spec_directory`.
/// On failure: exit(2).
pub async fn step4_load_sensor_specs(
    _config: &PrismConfig,
) -> Result<Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>, BootError> {
    todo!("S-WAVE5-PREP-01 step 4 — call parse_spec_directory(config.spec_dir); wrap in Arc<ArcSwap<ConfigManager>>; first production call site per ADR-022 §C; exit(2) on failure")
}

/// Step 5 [BLOCKING]: Initialize credential store.
///
/// ADR-022 §B step 5; BC-2.03.013.
/// Constructs `CredentialStore` using the backend declared in `prism.toml`.
/// Validates all credential refs declared in sensor specs (reference-only;
/// NO credential values are loaded into memory — AI-opacity invariant AD-017).
/// Permission-denied → exit(5). Config-invalid ref → exit(2).
pub async fn step5_init_credential_store(
    _config: &PrismConfig,
) -> Result<Arc<dyn prism_credentials::CredentialStore>, BootError> {
    todo!("S-WAVE5-PREP-01 step 5 — construct CredentialStore per BC-2.03.013; validate all refs from sensor specs (reference-only, no values per AD-017); permission-denied → exit(5); invalid ref → exit(2)")
}

/// Step 6 [BLOCKING]: Initialize audit subsystem.
///
/// ADR-022 §B step 6; BC-2.05.012.
/// Constructs `AuditEmitterLayer` (prism-audit Tower middleware layer).
/// Opens the `audit_buffer` RocksDB column family (AD-004).
/// Writes the `boot.audit.initialized` sentinel synchronously and durably
/// before returning (BC-2.05.012 invariant).
/// On any failure: exit(4). Audit is NON-OPTIONAL (SOC 2 hard requirement).
pub async fn step6_init_audit(
    _storage: Arc<prism_storage::rocksdb_backend::RocksDbBackend>,
) -> Result<
    Arc<prism_audit::AuditEmitterLayer<prism_storage::rocksdb_backend::RocksDbBackend>>,
    BootError,
> {
    todo!("S-WAVE5-PREP-01 step 6 — construct AuditEmitterLayer; open audit_buffer CF per AD-004; write boot.audit.initialized sentinel synchronously before returning per BC-2.05.012; exit(4) on any failure; SOC 2 non-optional")
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
    /// BC: BC-2.22.001 §Sequencing Invariant — run_boot_sequence is a todo!()
    /// at Red Gate; calling it panics.
    ///
    /// RED GATE: This test verifies that run_boot_sequence is not yet implemented.
    /// It will fail once the implementer fills the stub.
    // Note: We cannot directly test async todo!() in a sync test without spawning
    // a tokio runtime. The subprocess tests in bc_2_22_001_boot_orchestration.rs
    // cover the sequencing contract end-to-end.

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

/// Placeholder for the deserialized `prism.toml` config struct.
///
/// The implementer replaces this with the real serde-derived struct that
/// matches `config-schema.md`.  Fields here are the minimum needed to
/// compile the boot step stubs against correct types.
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
