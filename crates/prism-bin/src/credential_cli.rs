//! `prism credential set` CLI subcommand — AD-017 compliant keyring write.
//!
//! # Contract (BC-2.03.007 + AC-005 of S-DEMO-003)
//!
//! The `prism credential set` subcommand writes a credential value to the OS
//! keyring under an OrgId-UUID-keyed namespace `{org_id_uuid}/{sensor}/{name}` via
//! `CredentialStoreOrgId::set_by_org` (ADR-034 §D3 / namespace_key_by_org_id).
//!
//! ## AD-017 compliance — stdin-only value input
//!
//! The credential VALUE must NEVER appear as a CLI argument:
//! - CLI arg values are visible in `ps aux` output and shell history.
//! - AD-017 (AI-opaque credential model) forbids any path where the value
//!   transits a visible channel.
//!
//! The value is read from stdin using `rpassword::prompt_password` (or equivalent)
//! which disables terminal echo. The `--value` flag is explicitly ABSENT from
//! `CredentialSetArgs` by design (EC-005 of S-DEMO-003: clap must reject it).
//!
//! ## Keyring write API (OrgId-keyed — ADR-034 §D3)
//!
//! The subcommand calls `CredentialStoreOrgId::set_by_org` via the `KeyringBackend`
//! from `prism-credentials`:
//!
//! ```text
//! KeyringBackend::set_by_org(org_id: &OrgId, sensor: &str, name: &CredentialName, value: SecretString)
//! ```
//!
//! Namespace key produced: `"{org_id_uuid}/{sensor}/{name}"` (namespace_key_by_org_id).
//! The legacy slug-keyed path `CredentialStore::set` (`"{slug}/{sensor}/{name}"`) is
//! FORBIDDEN here — its namespace is disjoint from `get_by_org`'s OrgId-keyed namespace
//! (CRIT-2 remediation; ADR-034 §D3).
//!
//! ## rpassword dependency
//!
//! `rpassword = "7"` added to `[dependencies]` in `crates/prism-bin/Cargo.toml`
//! as a prism-bin-only dep (not in workspace [dependencies]) — binary-surface only.
//!
//! ## Testable seam — `generate_demo_prism_toml`
//!
//! The function `generate_demo_prism_toml` is the Red Gate seam for
//! `test_BC_2_06_001_demo_setup_generates_valid_prism_toml`. The implementer MUST
//! implement this function to return a valid `prism.toml` string that deserializes
//! via `PrismConfig` (BC-2.06.001). The demo-setup.sh script uses this function's
//! output (or matches its logic) when writing `~/.config/prism-demo/prism.toml`.
//!
//! Story: S-DEMO-003
//! BCs: BC-2.03.007, BC-2.06.001

use std::sync::Arc;

use clap::{Args, Subcommand};
use prism_core::{CredentialName, OrgId};
use prism_credentials::{CredentialIndex, CredentialStoreOrgId, KeyringBackend};
use secrecy::SecretString;

use crate::exit_codes::{EXIT_CONFIG_INVALID, EXIT_GENERIC_ERROR, EXIT_SUCCESS};

// ---------------------------------------------------------------------------
// Clap argument types
// ---------------------------------------------------------------------------

/// Subcommands under `prism credential`.
#[derive(Debug, Subcommand)]
pub enum CredentialCommand {
    /// Store a credential in the OS keyring (AD-017 compliant: value read from stdin).
    ///
    /// Usage: prism credential set --sensor <SENSOR_ID> --name <CREDENTIAL_NAME>
    ///
    /// The subcommand prompts "Enter value for prism/<sensor>/<name>: " on stderr
    /// with terminal echo disabled (rpassword), then writes to the OS keyring.
    ///
    /// The `--value` flag is explicitly ABSENT (AD-017 / EC-005 S-DEMO-003).
    Set(CredentialSetArgs),

    /// Delete a credential from the OS keyring.
    ///
    /// Usage: prism credential delete --sensor <SENSOR_ID> --name <CREDENTIAL_NAME>
    ///
    /// Deletes the credential stored under the OrgId-keyed namespace
    /// `{org_id_uuid}/{sensor}/{name}` (namespace_key_by_org_id, ADR-034 §D3).
    ///
    /// Exit codes: 0 deleted or already absent, 1 keyring error, 2 config-invalid.
    ///
    /// F-P10-HIGH-001: used by demo-teardown.sh on ALL platforms to guarantee the
    /// EXACT same namespace + backend attribute schema as the write path (`set_by_org`).
    /// This replaces the platform-CLI approach (`security delete-generic-password` /
    /// `secret-tool clear`) which was fragile (wrong attribute name on Linux caused
    /// 5 orphaned keyring entries).
    ///
    /// AC-007 / BC-2.03.005.
    Delete(CredentialDeleteArgs),
}

/// Arguments for `prism credential set`.
///
/// AD-017 contract: `--value` flag is FORBIDDEN. Value MUST be read from stdin.
#[derive(Debug, Args)]
pub struct CredentialSetArgs {
    /// Sensor ID (e.g., crowdstrike, armis, claroty, cyberint).
    #[arg(long, value_name = "SENSOR_ID")]
    pub sensor: String,

    /// Credential name (e.g., client_id, client_secret, api_token).
    #[arg(long, value_name = "CREDENTIAL_NAME")]
    pub name: String,

    /// Org slug (default: first org in prism.toml; required if multiple orgs configured).
    ///
    /// The org slug is matched against `[[orgs]]` entries in `prism.toml` to extract
    /// the `org_id` UUID. The credential is then written via `set_by_org` under the
    /// OrgId-keyed namespace `{org_id_uuid}/{sensor}/{name}` (namespace_key_by_org_id,
    /// ADR-034 §D3). The legacy slug-keyed namespace `{slug}/{sensor}/{name}` is
    /// FORBIDDEN for writes — it is disjoint from `get_by_org`'s read namespace (CRIT-2).
    #[arg(long, value_name = "ORG_SLUG")]
    pub org_slug: Option<String>,
}

/// Arguments for `prism credential delete`.
///
/// F-P10-HIGH-001 fix: delete MUST use `delete_by_org` (OrgId-keyed namespace)
/// to match the write path. Platform-CLI tools (`security delete-generic-password` /
/// `secret-tool clear account`) use different attribute names and produce orphaned
/// entries on Linux (wrong attribute key in libsecret). Using the Rust keyring backend
/// guarantees namespace parity on all platforms.
#[derive(Debug, Args)]
pub struct CredentialDeleteArgs {
    /// Sensor ID (e.g., crowdstrike, armis, claroty, cyberint).
    #[arg(long, value_name = "SENSOR_ID")]
    pub sensor: String,

    /// Credential name (e.g., client_id, client_secret, api_token).
    #[arg(long, value_name = "CREDENTIAL_NAME")]
    pub name: String,

    /// Org slug (default: first org in prism.toml; required if multiple orgs configured).
    #[arg(long, value_name = "ORG_SLUG")]
    pub org_slug: Option<String>,
}

/// Top-level args wrapper for `prism credential <subcommand>`.
#[derive(Debug, Args)]
pub struct CredentialArgs {
    #[command(subcommand)]
    pub command: CredentialCommand,
}

// ---------------------------------------------------------------------------
// Handler implementation
// ---------------------------------------------------------------------------

/// Handle `prism credential set` — AD-017 compliant keyring write.
///
/// Production entry point: loads `prism.toml` ONCE via `load_prism_config_for_cli`,
/// constructs a `KeyringBackend` with the state_dir-based index, then delegates to
/// `handle_credential_set_with_store`.
///
/// # Single config load (F-P8-OBS-001)
///
/// `prism.toml` is parsed exactly ONCE per invocation via `load_prism_config_for_cli`.
/// The parsed `PrismConfig` is passed to `handle_credential_set_with_store` which uses
/// it for org resolution via `resolve_org_slug_and_id`. No second parse occurs.
///
/// # AC-012 message ownership
///
/// `load_prism_config_for_cli` emits the EXACT AC-012-documented error message on
/// missing/unparseable prism.toml. No `boot::step2_load_config` precheck exists here
/// that would shadow the AC-012 message (F-P8-MED-001 fix).
///
/// # AD-017 invariant
///
/// The credential value MUST NOT appear in:
/// - stdout (checked by `test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout`)
/// - stderr (same test)
/// - logs (`tracing::*!` calls)
/// - any `PrismError` message (BC-2.03.007 postcondition 3)
///
/// # OrgId-keyed write (ADR-034 §D3)
///
/// Uses `CredentialStoreOrgId::set_by_org` with the OrgId UUID from `prism.toml`.
/// This reconciles the write namespace with the Tier-3 read namespace in
/// `resolve_credential`. The legacy `CredentialStore::set` (slug-keyed) is NOT used
/// because its namespace (`"{slug}/{sensor}/{name}"`) is disjoint from the OrgId-keyed
/// namespace (`"{org_id_uuid}/{sensor}/{name}"`) used by `get_by_org` — CRIT-2 fix.
///
/// # Exit codes
///
/// 0 — success (credential stored)
/// 1 — keyring unavailable or write failure (with actionable stderr message)
/// 2 — config-invalid (cannot load prism.toml or resolve org)
pub async fn handle_credential_set(args: CredentialSetArgs, config_dir: std::path::PathBuf) -> i32 {
    // Load PrismConfig ONCE — single parse, AC-012 message on failure (F-P8-MED-001 / F-P8-OBS-001).
    // The credential index MUST live in state_dir to match the boot read-path
    // (boot step 5: config.state_dir.join("credential_index.json")).
    let prism_config = match load_prism_config_for_cli(&config_dir) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{e}");
            return EXIT_CONFIG_INVALID;
        }
    };
    // Ensure state_dir exists before writing the index (mirrors boot step 6 mkdir).
    if let Err(e) = std::fs::create_dir_all(&prism_config.state_dir) {
        eprintln!(
            "prism credential set: cannot create state_dir '{}': {e}",
            prism_config.state_dir.display()
        );
        return EXIT_GENERIC_ERROR;
    }
    // Construct the production KeyringBackend.
    // Index path: <state_dir>/credential_index.json — aligned with boot step 5.
    let index_path = prism_config.state_dir.join("credential_index.json");
    let index = CredentialIndex::new(index_path);
    let store: Arc<dyn CredentialStoreOrgId> = Arc::new(KeyringBackend::new("prism", index));

    // Production secret reader strategy:
    //
    // 1. Try rpassword::read_password() — opens /dev/tty to disable echo on a TTY.
    //    If the operator is typing interactively, this path suppresses echo (AD-017
    //    interactive-TTY requirement; BC-2.03.007 postcondition "echo disabled").
    //
    // 2. If rpassword fails with ENXIO (os error 6 — /dev/tty unavailable, e.g. piped
    //    stdin from demo-setup.sh or `printf '%s\n' | prism credential set`), fall back
    //    to reading from stdin directly via BufRead::read_line. Echo is not possible on
    //    a pipe (AD-017 satisfied by the channel property).
    //
    // In both cases, the value is placed in a Cursor so handle_credential_set_with_store
    // can read it via its injectable `secret_reader: &mut dyn BufRead` parameter.
    // This keeps the production TTY rpassword path AND the test seam intact.
    let prompt = format!("Enter value for prism/{}/{}: ", args.sensor, args.name);
    eprint!("{prompt}"); // prompt on stderr before rpassword opens /dev/tty

    // NOTE: rpassword::read_password() internally emits its own prompt via /dev/tty;
    // suppress the double-prompt in the pipe fallback by detecting ENXIO here.
    let secret_bytes: Vec<u8> = match rpassword::read_password() {
        Ok(v) => {
            // TTY path: echo was disabled by rpassword. The value arrived without echo.
            // Encode as line for Cursor (read_secret_value_from strips the trailing \n).
            let mut bytes = v.into_bytes();
            bytes.push(b'\n');
            bytes
        }
        Err(e) if e.raw_os_error() == Some(6) => {
            // ENXIO — piped stdin, no /dev/tty. Read from stdin directly.
            let stdin = std::io::stdin();
            let mut locked = stdin.lock();
            let mut line = String::new();
            if let Err(read_err) = std::io::BufRead::read_line(&mut locked, &mut line) {
                eprintln!("\nprism credential set: failed to read credential value: {read_err}");
                return EXIT_GENERIC_ERROR;
            }
            line.into_bytes()
        }
        Err(e) => {
            eprintln!("\nprism credential set: failed to read credential value: {e}");
            return EXIT_GENERIC_ERROR;
        }
    };

    let mut cursor = std::io::Cursor::new(secret_bytes);
    let prism_toml_path = config_dir.join("prism.toml");
    handle_credential_set_with_store(args, &prism_config, &prism_toml_path, store, &mut cursor)
        .await
}

/// Inner implementation of `handle_credential_set` with injectable `CredentialStoreOrgId`
/// and injectable `BufRead` for the credential value.
///
/// Separated from `handle_credential_set` to enable unit testing without:
/// - a real OS keyring (inject `InMemoryCredentialStore` via `store`)
/// - real stdin (inject a `std::io::Cursor<&[u8]>` via `secret_reader`)
///
/// # Testability seams (RG-034-004)
///
/// `bc_2_03_007_credential_set_org_id_keyed.rs` calls this function with:
/// - A `PrismConfig` loaded from a fixture `prism.toml` in a temp dir.
/// - `InMemoryCredentialStore` to assert the namespace key is OrgId-keyed.
/// - A `Cursor` wrapping the test secret value to avoid reading from real stdin.
///
/// This provides load-bearing in-process coverage of the full production code path:
/// `PrismConfig` → OrgId extraction → `CredentialName` validation → `set_by_org` dispatch.
/// The `prism.toml` is parsed exactly ONCE (in the caller) — no second parse here.
///
/// # AD-017 compliance
///
/// The credential value must never appear in stdout, stderr, or logs.
/// `secret_reader` provides the value without it transiting any visible channel.
/// The `store` concrete type (KeyringBackend or InMemoryCredentialStore) does not affect
/// the AD-017 guarantee; this function never echoes `raw_value` after reading it.
pub async fn handle_credential_set_with_store(
    args: CredentialSetArgs,
    prism_config: &crate::boot::PrismConfig,
    prism_toml_path: &std::path::Path,
    store: Arc<dyn CredentialStoreOrgId>,
    secret_reader: &mut dyn std::io::BufRead,
) -> i32 {
    // Step 1: Resolve org slug AND OrgId from the already-loaded PrismConfig.
    // ADR-034 §D3: --org-slug is matched against [[orgs]] entries in prism.toml
    // to extract the org_id UUID for OrgId-keyed write.
    // Single parse: prism_config was loaded ONCE by handle_credential_set (or test caller).
    let (org_slug_str, org_id) =
        match resolve_org_slug_and_id(&args.org_slug, prism_config, prism_toml_path) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("prism credential set: config error: {e}");
                return EXIT_CONFIG_INVALID;
            }
        };
    // org_slug_str and org_id are now valid and consistent.
    let _ = org_slug_str; // retained for potential future use in audit messages

    // Validate credential name.
    let cred_name = match CredentialName::new(&args.name) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("prism credential set: invalid credential name: {e}");
            return EXIT_CONFIG_INVALID;
        }
    };

    // Step 3: Read the credential value from the injected reader — AD-017.
    //
    // The reader is provided by the caller:
    //   - Production (`handle_credential_set`): a Cursor containing the value already
    //     read by rpassword (TTY path, echo disabled) or stdin (pipe fallback). The
    //     prompt was emitted by `handle_credential_set` before calling this function.
    //   - Tests (RG-034-004): a Cursor wrapping the injected secret bytes. No real stdin
    //     access occurs. No prompt emission needed in tests.
    //
    // This function does NOT emit the prompt — that is the caller's responsibility.
    // AD-017: `raw_value` must be wrapped in SecretString immediately (see below).
    let raw_value = match read_secret_value_from(secret_reader) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("\nprism credential set: failed to read credential value: {e}");
            return EXIT_GENERIC_ERROR;
        }
    };

    // INVARIANT: the raw_value MUST NOT appear in any log or stdout output below.
    // Wrap in SecretString immediately so accidental Display/Debug shows "***".
    let value = SecretString::new(raw_value);

    // Step 4: Write to the injected store via set_by_org() (OrgId-keyed).
    // ADR-034 §D3: MUST use CredentialStoreOrgId::set_by_org (OrgId-keyed namespace)
    // to reconcile with Tier-3 read path in resolve_credential.
    // The legacy CredentialStore::set (slug-keyed) is FORBIDDEN for this path —
    // it writes to a disjoint namespace invisible to get_by_org (CRIT-2 remediation).
    match store
        .set_by_org(&org_id, &args.sensor, &cred_name, value)
        .await
    {
        Ok(()) => {
            // Step 5: Print success to stdout — only the success message, never the value.
            println!("Credential stored successfully.");
            EXIT_SUCCESS
        }
        Err(e) => {
            // Check for "Keyring unavailable" pattern (EC-001 of S-DEMO-003).
            // E-CRED-004 (write-path): keyring-rs surfaces platform unavailability as
            // NoStorageAccess or similar. This is distinct from E-CRED-008 (read-path),
            // which is surfaced by resolution.rs Tier-3 at query time.
            let err_str = e.to_string();
            if err_str.contains("NoStorageAccess")
                || err_str.contains("NoKeyringService")
                || err_str.contains("Unavailable")
                || err_str.contains("DBus")
                || err_str.contains("keychain")
                || err_str.contains("spawn_blocking panicked")
            {
                eprintln!(
                    "Keyring unavailable: {e}. Set the credential via the \
                     PRISM_CLIENTS_<ORG>_SENSORS_<SENSOR>_<REF> environment variable instead \
                     (see DEMO-RUNBOOK.md §6 Troubleshooting (a))."
                );
            } else {
                eprintln!("prism credential set: keyring write failed: {e}");
            }
            EXIT_GENERIC_ERROR
        }
    }
}

// ---------------------------------------------------------------------------
// `prism credential delete` handler — F-P10-HIGH-001 fix
// ---------------------------------------------------------------------------

/// Handle `prism credential delete` — OrgId-keyed keyring delete via `delete_by_org`.
///
/// F-P10-HIGH-001: the teardown script previously used `secret-tool clear service "prism"
/// account "${key}"` on Linux. keyring-rs 3.x dbus-secret-service stores the user under
/// the `username` attribute (NOT `account`), so the clear matched nothing — 5 orphaned
/// entries remained in the Linux keyring after every demo teardown.
///
/// Using `prism credential delete` guarantees the EXACT same namespace + backend attribute
/// schema as the write path (`set_by_org`). This is correct on ALL platforms (macOS,
/// Linux, Windows) because keyring-rs handles the attribute mapping transparently.
///
/// # Single config load (F-P8-OBS-001)
///
/// `prism.toml` is parsed exactly ONCE via `load_prism_config_for_cli`. The parsed
/// `PrismConfig` is passed to `handle_credential_delete_with_store` for org resolution.
/// AC-012 message is emitted by `load_prism_config_for_cli` on missing/invalid toml.
///
/// # Contract (BC-2.03.005 delete path / AC-007)
///
/// Returns:
/// - 0 — credential deleted (was present) OR already absent (idempotent)
/// - 1 — keyring unavailable or delete error (backend error surfaced to stderr)
/// - 2 — config-invalid (cannot load prism.toml or resolve org)
pub async fn handle_credential_delete(
    args: CredentialDeleteArgs,
    config_dir: std::path::PathBuf,
) -> i32 {
    // Load PrismConfig ONCE — single parse, AC-012 message on failure (F-P8-MED-001 / F-P8-OBS-001).
    // The credential index MUST live in state_dir to match the boot read-path
    // (boot step 5: config.state_dir.join("credential_index.json")).
    let prism_config = match load_prism_config_for_cli(&config_dir) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{e}");
            return EXIT_CONFIG_INVALID;
        }
    };
    // Index path: <state_dir>/credential_index.json — aligned with boot step 5.
    let index_path = prism_config.state_dir.join("credential_index.json");
    let index = CredentialIndex::new(index_path);
    let store: Arc<dyn CredentialStoreOrgId> = Arc::new(KeyringBackend::new("prism", index));
    let prism_toml_path = config_dir.join("prism.toml");
    handle_credential_delete_with_store(args, &prism_config, &prism_toml_path, store).await
}

/// Inner implementation of `handle_credential_delete` with injectable `CredentialStoreOrgId`.
///
/// Separated for testability: inject `InMemoryCredentialStore` to assert the namespace key
/// is OrgId-keyed (mirrors the `handle_credential_set_with_store` test pattern).
///
/// Accepts an already-parsed `&PrismConfig` — no second prism.toml parse occurs here
/// (F-P8-OBS-001 single-parse invariant).
pub async fn handle_credential_delete_with_store(
    args: CredentialDeleteArgs,
    prism_config: &crate::boot::PrismConfig,
    prism_toml_path: &std::path::Path,
    store: Arc<dyn CredentialStoreOrgId>,
) -> i32 {
    // Step 1: Resolve OrgId from the already-loaded PrismConfig (same as set path — ADR-034 §D3).
    // Single parse: prism_config was loaded ONCE by handle_credential_delete (or test caller).
    let (_org_slug_str, org_id) =
        match resolve_org_slug_and_id(&args.org_slug, prism_config, prism_toml_path) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("prism credential delete: config error: {e}");
                return EXIT_CONFIG_INVALID;
            }
        };

    // Step 2: Validate credential name.
    let cred_name = match CredentialName::new(&args.name) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("prism credential delete: invalid credential name: {e}");
            return EXIT_CONFIG_INVALID;
        }
    };

    // Step 3: Delete via delete_by_org (OrgId-keyed namespace — matches write path exactly).
    // This guarantees namespace parity regardless of platform attribute schema.
    match store.delete_by_org(&org_id, &args.sensor, &cred_name).await {
        Ok(true) => {
            println!("Credential deleted.");
            EXIT_SUCCESS
        }
        Ok(false) => {
            // Not found — idempotent (teardown scripts call delete even when entries may
            // already be absent from a previous run). Return 0 so teardown continues.
            println!("Credential not found (already absent).");
            EXIT_SUCCESS
        }
        Err(e) => {
            eprintln!("prism credential delete: keyring delete failed: {e}");
            EXIT_GENERIC_ERROR
        }
    }
}

/// Read one line from an injectable `BufRead` source, stripping the trailing newline.
///
/// # Testability seam (F-HIGH-002 / RG-034-004)
///
/// This function accepts `&mut dyn BufRead` (via the generic `R: BufRead + ?Sized` bound)
/// so `handle_credential_set_with_store` can be tested in-process without touching real
/// stdin. Tests inject a `std::io::Cursor<Vec<u8>>` wrapping the secret value; production
/// callers inject a `std::io::Cursor<Vec<u8>>` containing the value already read by
/// `rpassword::read_password()` (TTY path, echo disabled) or from stdin (pipe fallback).
///
/// # AD-017 compliance
///
/// - **TTY path:** `handle_credential_set` calls `rpassword::read_password()` BEFORE this
///   function, which opens /dev/tty to suppress echo. The returned value is placed in a
///   `Cursor` and passed here. Echo disable happens before this function is reached.
/// - **Pipe path:** no TTY, echo impossible. `handle_credential_set` reads from stdin
///   lock and places the value in a `Cursor`. No echo is possible on a pipe.
/// - **Test path:** the `Cursor` holds the injected value. No stdin or TTY access occurs.
///
/// AD-017: the caller (`handle_credential_set_with_store`) wraps the returned `String`
/// in `SecretString` immediately so any accidental `Debug`/`Display` output is redacted.
fn read_secret_value_from<R: std::io::BufRead + ?Sized>(
    reader: &mut R,
) -> Result<String, std::io::Error> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    // Strip the trailing newline that `writeln!` / `printf '%s\n'` appends.
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(line)
}

/// Load and parse `prism.toml` for CLI subcommands — single canonical entry point.
///
/// This is the ONLY place where `prism.toml` is read for CLI handlers. All credential
/// subcommands (`credential set`, `credential delete`) call this function ONCE and pass
/// the returned `PrismConfig` to their inner functions (F-P8-OBS-001 single-parse invariant).
///
/// # AC-012 — canonical error message (F-P8-MED-001)
///
/// On missing or unparseable `prism.toml`, this function returns `Err` with EXACTLY the
/// AC-012-documented message:
///
/// > "Could not load prism.toml from '<config_dir>': <reason>. Ensure prism.toml exists
/// > (run demo-setup.sh or create it manually) before running `prism credential set`."
///
/// No `boot::step2_load_config` precheck shadowing occurs — this is the FIRST and ONLY
/// config load in the CLI handler path. RG-034-003 remains load-bearing because
/// `resolve_org_slug_and_id` delegates to this function for the file-read path
/// (unit test `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug`).
///
/// S-DEMO-003 AC-012 / ADR-034 §D3.
pub fn load_prism_config_for_cli(
    config_dir: &std::path::Path,
) -> Result<crate::boot::PrismConfig, String> {
    let toml_path = config_dir.join("prism.toml");
    match std::fs::read_to_string(&toml_path) {
        Ok(contents) => toml::from_str::<crate::boot::PrismConfig>(&contents)
            .map_err(|e| format!("cannot parse '{}': {e}", toml_path.display()))
            .map(|mut config| {
                // BLOCKING-3 (S-REL-005 AC-009): resolve relative path fields against
                // config_dir so that `prism credential set` (CLI) resolves state_dir to
                // the same absolute path as `prism start` (boot), regardless of CWD.
                // Without this call a relative state_dir = "./state" in prism.toml causes
                // CLI and server to write/read credential indexes from different directories
                // — silent credential loss.
                crate::boot::resolve_config_paths(&mut config, config_dir);
                config
            }),
        Err(e) => {
            // prism.toml missing or unreadable — AC-012 canonical error message.
            //
            // ADR-034 §D3: --org-slug maps to OrgId by reading the matching [[orgs]]
            // entry's org_id UUID from prism.toml. If prism.toml is missing or
            // unparseable → hard error (exit 2).
            //
            // The UUID-v5 derivation fallback is REMOVED because:
            //   1. It creates a namespace-mismatch risk: if the operator later creates
            //      prism.toml with a different org_id for the same slug, credentials
            //      written in fallback mode are silently lost (SOUL.md §4).
            //   2. ADR-034 §D3 explicitly requires the org_id to come from prism.toml.
            //
            // The correct operator workflow is:
            //   1. Create prism.toml (via demo-setup.sh or manual setup).
            //   2. Then run `prism credential set`.
            Err(format!(
                "Could not load prism.toml from '{}': {e}. \
                 Ensure prism.toml exists (run demo-setup.sh or create it manually) \
                 before running `prism credential set`.",
                config_dir.display()
            ))
        }
    }
}

/// Resolve org slug AND OrgId from an already-loaded `PrismConfig`.
///
/// ADR-034 §D3: `handle_credential_set` uses `CredentialStoreOrgId::set_by_org`
/// which requires the OrgId UUID. This function finds the org entry whose `org_slug`
/// matches the explicitly provided slug or (single-org case) is the only entry.
///
/// Accepts an already-parsed `PrismConfig` — no file I/O occurs here. The config was
/// loaded once by `load_prism_config_for_cli` (F-P8-OBS-001 single-parse invariant).
///
/// Returns `(org_slug_str, OrgId)` on success.
///
/// # Error cases
///
/// - No matching org for `--org-slug` value → hard error with suggestion
/// - Multiple orgs and no `--org-slug` → hard error citing all org slugs
/// - `org_id` field missing or invalid UUID in the matching org entry → hard error
///
/// S-DEMO-003 AC-005 / AC-010; ADR-034 §D3 (CRIT-2 namespace reconciliation).
///
/// # Unit test (RG-034-003)
///
/// `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug` tests the
/// missing-toml error path by calling the path-based wrapper function below, which
/// calls `load_prism_config_for_cli` and delegates here. RG-034-003 remains load-bearing
/// because the AC-012 message originates in `load_prism_config_for_cli`.
fn resolve_org_slug_and_id(
    explicit: &Option<String>,
    config: &crate::boot::PrismConfig,
    prism_toml_path: &std::path::Path,
) -> Result<(String, OrgId), String> {
    if config.orgs.is_empty() {
        return Err(
            "prism.toml declares no orgs — add an [[orgs]] entry or use --org-slug".to_string(),
        );
    }

    // Find the matching org entry.
    let org_entry = if let Some(slug) = explicit {
        // --org-slug provided: find the exact match.
        config
            .orgs
            .iter()
            .find(|o| o.org_slug == *slug)
            .ok_or_else(|| {
                let all_slugs: Vec<&str> =
                    config.orgs.iter().map(|o| o.org_slug.as_str()).collect();
                format!(
                    "--org-slug '{slug}' not found in prism.toml '{}'. \
                     Configured orgs: {all_slugs:?}",
                    prism_toml_path.display()
                )
            })?
    } else if config.orgs.len() == 1 {
        // Single org: use it automatically.
        &config.orgs[0]
    } else {
        // Multiple orgs and no --org-slug: require explicit selection.
        let all_slugs: Vec<&str> = config.orgs.iter().map(|o| o.org_slug.as_str()).collect();
        return Err(format!(
            "Multiple orgs configured in '{}' — use --org-slug <slug> to select one. \
             Configured orgs: {all_slugs:?}",
            prism_toml_path.display()
        ));
    };

    // Parse the OrgId UUID from the org entry.
    let uuid = uuid::Uuid::parse_str(&org_entry.org_id).map_err(|e| {
        format!(
            "org '{}' in prism.toml has invalid org_id '{}': {e}. \
             Expected a valid UUID v7 string.",
            org_entry.org_slug, org_entry.org_id
        )
    })?;
    let org_id = OrgId::from_uuid(uuid);

    Ok((org_entry.org_slug.clone(), org_id))
}

/// Path-based wrapper for `resolve_org_slug_and_id` — loads `prism.toml` from disk
/// and delegates to the config-taking variant.
///
/// Used by the unit test `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug`
/// (RG-034-003) to test the missing-toml error path directly. Production code uses
/// `load_prism_config_for_cli` + `resolve_org_slug_and_id` in sequence.
///
/// S-DEMO-003 AC-012 / RG-034-003.
#[cfg(test)]
async fn resolve_org_slug_and_id_by_path(
    explicit: &Option<String>,
    config_dir: &std::path::Path,
) -> Result<(String, OrgId), String> {
    let config = load_prism_config_for_cli(config_dir)?;
    let prism_toml_path = config_dir.join("prism.toml");
    resolve_org_slug_and_id(explicit, &config, &prism_toml_path)
}

// ---------------------------------------------------------------------------
// Testable seam — generate_demo_prism_toml
// ---------------------------------------------------------------------------

/// Generate the demo `prism.toml` content as a `String`.
///
/// This function is the Red Gate seam for
/// `test_BC_2_06_001_demo_setup_generates_valid_prism_toml` (AC-001 of S-DEMO-003).
///
/// # Contract (BC-2.06.001)
///
/// The returned string:
/// - Is valid TOML that deserializes via `toml::from_str::<PrismConfig>(&content)` without error.
/// - Contains `spec_dir`, `state_dir`, and one `[[orgs]]` entry with a real UUID v7 `org_id`
///   and kebab-case `org_slug = "demo-org"`.
/// - The `org_id` is a real UUID v7 (time-ordered, `get_version() == Some(uuid::Version::SortRand)`).
///
/// # Implementation note — fixed demo UUID
///
/// A fixed (hardcoded) UUID v7 value is used so the demo prism.toml is stable across
/// re-runs of `demo-setup.sh` and so the credential index keys are predictable.
/// This is intentional for demo purposes: the UUID must be v7 but need not be unique
/// across runs. Production orgs use `uuid::Uuid::now_v7()` for uniqueness.
///
/// Story: S-DEMO-003 | BC: BC-2.06.001
pub fn generate_demo_prism_toml() -> String {
    // Fixed UUID v7 for the demo org — stable across setup script re-runs.
    // Generated with uuid::Uuid::now_v7() at S-DEMO-003 implementation time (2026-06-06).
    // Version 7 verified: time-ordered (SortRand), not v4.
    //
    // Format: TOML with literal tilde paths (e.g. "~/.config/prism-demo/specs").
    // NOTE: This function is a test seam only (no production caller). It emits literal
    // tilde strings for schema-validity testing. demo-setup.sh does NOT mirror this
    // function's content — it writes shell-expanded absolute paths (${DEMO_SPECS_DIR}
    // etc.) directly into prism.toml. Prism does not tilde-expand filesystem paths.
    let demo_org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0b1c";
    let demo_org_slug = "demo-org";
    let spec_dir = "~/.config/prism-demo/specs";
    let state_dir = "~/.config/prism-demo/state";
    let plugin_dir = "~/.config/prism-demo/plugins";

    format!(
        r#"spec_dir = "{spec_dir}"
state_dir = "{state_dir}"
plugin_dir = "{plugin_dir}"

[[orgs]]
org_id = "{demo_org_id}"
org_slug = "{demo_org_slug}"
"#
    )
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the stub compiles and exposes the correct public surface.
    /// This test passes immediately — it is not a Red Gate test.
    /// The Red Gate tests are in `tests/bc_2_03_007_credential_set_no_echo.rs`
    /// and `tests/bc_2_06_001_demo_setup_toml.rs`.
    #[test]
    fn test_credential_set_args_no_value_flag() {
        // AD-017: verify CredentialSetArgs has no `value` field at the type level.
        // This is a structural assertion — if a `--value` field is ever added,
        // the test should be updated to assert the field is absent or feature-gated.
        //
        // The real enforcement is at clap parse time: if a user passes `--value foo`,
        // clap must reject it as "unexpected argument" (EC-005 of S-DEMO-003).
        // That behavior is tested by the subprocess Red Gate test.
        let args = CredentialSetArgs {
            sensor: "crowdstrike".to_string(),
            name: "client_id".to_string(),
            org_slug: None,
        };
        assert_eq!(args.sensor, "crowdstrike");
        assert_eq!(args.name, "client_id");
        assert!(args.org_slug.is_none());
        // Key assertion: no `value` field exists on CredentialSetArgs.
        // If the field were added (AD-017 violation), this struct literal
        // construction would require it and fail to compile.
    }

    /// Verify generate_demo_prism_toml returns non-empty TOML.
    #[test]
    fn test_generate_demo_prism_toml_non_empty() {
        let toml = generate_demo_prism_toml();
        assert!(!toml.is_empty());
        assert!(toml.contains("spec_dir"));
        assert!(toml.contains("state_dir"));
        assert!(toml.contains("demo-org"));
    }

    // ---------------------------------------------------------------------------
    // RG-034-003: resolve_org_slug_and_id errors when prism.toml is missing
    // ---------------------------------------------------------------------------

    /// HIGH-3 remediation (ADR-034 §D3 / AC-012 of S-DEMO-003): when `--org-slug` is
    /// absent and `prism.toml` is missing from the config dir, the credential CLI path
    /// must return `Err(...)` with the AC-012 actionable message — NOT `Ok(("demo-org", ...))`.
    ///
    /// **Contract:** `resolve_org_slug_and_id_by_path` is the path-based wrapper that
    /// calls `load_prism_config_for_cli` (the AC-012 message source). This test drives
    /// the production missing-toml error path end-to-end:
    ///   `resolve_org_slug_and_id_by_path` → `load_prism_config_for_cli` → `Err(AC-012 message)`
    ///
    /// **Red Gate:** Before the HIGH-3 fix, the function returned a silent `"demo-org"`
    /// fallback. After fix: returns `Err(...)` with an actionable message.
    /// If someone reintroduces the demo-org fallback, this test fails.
    ///
    /// RG-034-003 (ADR-034 §Red Gate Tests); AC-012 of S-DEMO-003.
    /// ADR-034 §D3 HIGH-3: MUST NOT return `"demo-org"` as a silent default when
    /// `prism.toml` is missing and `--org-slug` is absent.
    #[tokio::test]
    async fn test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let config_dir_without_prism_toml = tmp.path();

        // Verify no prism.toml exists in the temp dir.
        assert!(
            !config_dir_without_prism_toml.join("prism.toml").exists(),
            "test fixture: prism.toml must NOT exist in the temp dir for this test"
        );

        // Call the path-based wrapper (which calls load_prism_config_for_cli) with no explicit
        // org slug — simulates `prism credential set --sensor armis --name bearer_token` without
        // `--org-slug` when prism.toml is absent.
        let result = resolve_org_slug_and_id_by_path(&None, config_dir_without_prism_toml).await;

        // RED GATE ASSERTION: must return Err (not Ok with any value, including "demo-org").
        assert!(
            result.is_err(),
            "RG-034-003 (AC-012 HIGH-3): resolve_org_slug_and_id_by_path must return Err when \
             prism.toml is missing and --org-slug is absent. \
             Got Ok({:?}) — the 'demo-org' fallback is a SOUL.md §4 violation \
             (ADR-034 §D3 HIGH-3 remediation). \
             prism.toml must exist OR --org-slug must be provided.",
            result.ok()
        );

        let err_msg = result.unwrap_err();
        assert!(
            !err_msg.is_empty(),
            "RG-034-003: error message must be non-empty (actionable)"
        );
        // Verify the AC-012 canonical message is present (F-P8-MED-001).
        assert!(
            err_msg.contains("Could not load prism.toml from"),
            "RG-034-003 (AC-012): error message must contain 'Could not load prism.toml from'. \
             Got: {err_msg:?}"
        );
        assert!(
            err_msg.contains("Ensure prism.toml exists"),
            "RG-034-003 (AC-012): error message must contain 'Ensure prism.toml exists'. \
             Got: {err_msg:?}"
        );
        // The function returned Err — the HIGH-3 fix is confirmed.
        // "demo-org" must NOT have been returned as a successful default value.
    }

    // ---------------------------------------------------------------------------
    // AC-012 load-bearing test: load_prism_config_for_cli emits AC-012 message
    // ---------------------------------------------------------------------------

    /// AC-012 load-bearing coverage for the production CLI config-load path.
    ///
    /// Drives `load_prism_config_for_cli` directly with a config_dir whose `prism.toml`
    /// is ABSENT and asserts the returned `Err` contains the EXACT AC-012-documented
    /// message fragments.
    ///
    /// This is the in-process production-code-path test required by the task spec:
    /// "ADD a load-bearing test for the production CLI missing-toml path" (F-P8-MED-001 fix).
    ///
    /// Per SID-1: a unit test in this module's `#[cfg(test)]` block drives the behavior
    /// without subprocess overhead. The AC-012 message originates in `load_prism_config_for_cli`
    /// (single canonical source), so testing that function directly gives exact coverage.
    ///
    /// AC-012 / S-DEMO-003 / F-P8-MED-001.
    #[test]
    fn test_load_prism_config_for_cli_missing_toml_emits_ac012_message() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let config_dir = tmp.path();

        // Verify no prism.toml in the temp dir.
        assert!(
            !config_dir.join("prism.toml").exists(),
            "test fixture: prism.toml must NOT exist"
        );

        let result = load_prism_config_for_cli(config_dir);

        // (a) Must return Err — exit code 2 path.
        assert!(
            result.is_err(),
            "AC-012: load_prism_config_for_cli must return Err when prism.toml is absent. \
             Got Ok(_)."
        );

        let err_msg = result.unwrap_err();

        // (b) Must contain the AC-012-documented message fragments (F-P8-MED-001).
        assert!(
            err_msg.contains("Could not load prism.toml from"),
            "AC-012: error must contain 'Could not load prism.toml from'. Got: {err_msg:?}"
        );
        assert!(
            err_msg.contains("Ensure prism.toml exists"),
            "AC-012: error must contain 'Ensure prism.toml exists'. Got: {err_msg:?}"
        );
        assert!(
            err_msg.contains("demo-setup.sh"),
            "AC-012: error must mention 'demo-setup.sh'. Got: {err_msg:?}"
        );
        assert!(
            err_msg.contains("prism credential set"),
            "AC-012: error must mention 'prism credential set'. Got: {err_msg:?}"
        );
        // Must NOT silently return "demo-org" or any default — this is a hard-error path only.
    }

    // ---------------------------------------------------------------------------
    // BLOCKING-3: load_prism_config_for_cli must resolve relative state_dir
    // ---------------------------------------------------------------------------

    /// BLOCKING-3 fix verification — `load_prism_config_for_cli` must resolve a relative
    /// `state_dir` against `config_dir`, identical to the boot path.
    ///
    /// Without this fix, `prism credential set` (CLI path) and `prism start` (boot path)
    /// would each call their respective config loaders and, if `state_dir = "./state"` is
    /// declared in prism.toml, resolve to different directories depending on CWD — causing
    /// silent credential-index divergence and a CWE-22 path-traversal surface.
    ///
    /// This test exercises the production `load_prism_config_for_cli` code path directly
    /// (SID-1 compliance: no subprocess, no `#[ignore]`).
    ///
    /// BLOCKING-3 / S-REL-005 AC-009 (CLI site).
    #[test]
    fn test_load_prism_config_for_cli_resolves_relative_state_dir() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let config_dir = tmp.path();

        // Write prism.toml with a relative state_dir (and relative spec_dir / plugin_dir
        // to mirror the typical operator configuration).
        std::fs::write(
            config_dir.join("prism.toml"),
            "spec_dir = \"./specs\"\nstate_dir = \"./state\"\nplugin_dir = \"./plugins\"\n",
        )
        .expect("write prism.toml");

        let config = load_prism_config_for_cli(config_dir)
            .expect("load_prism_config_for_cli must succeed for syntactically valid prism.toml");

        // BLOCKING-3: state_dir must be an absolute path rooted at config_dir.
        // Before the fix the function returned "./state" unresolved.
        assert_eq!(
            config.state_dir,
            config_dir.join("state"),
            "BLOCKING-3: load_prism_config_for_cli must resolve state_dir './state' to \
             '{}/state'. Got: {:?}",
            config_dir.display(),
            config.state_dir,
        );

        // Guard: spec_dir and plugin_dir must also be resolved (covered by boot tests but
        // verified here to confirm the shared resolve_config_paths helper is called).
        assert_eq!(
            config.spec_dir,
            config_dir.join("specs"),
            "BLOCKING-3 guard: spec_dir must also be resolved at CLI site"
        );
        assert_eq!(
            config.plugin_dir,
            config_dir.join("plugins"),
            "BLOCKING-3 guard: plugin_dir must also be resolved at CLI site"
        );
    }
}
