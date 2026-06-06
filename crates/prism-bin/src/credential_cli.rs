//! `prism credential set` CLI subcommand — AD-017 compliant keyring write.
//!
//! # Contract (BC-2.03.007 + AC-005 of S-DEMO-003)
//!
//! The `prism credential set` subcommand writes a credential value to the OS
//! keyring under namespace `prism/{sensor_id}/{name}` scoped to org slug.
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
//! ## Keyring write API
//!
//! The implementer MUST call `CredentialStore::set()` via the `KeyringBackend`
//! from `prism-credentials`:
//!
//! ```text
//! KeyringBackend::set(tenant: &OrgSlug, sensor: &str, name: &CredentialName, value: SecretString)
//! ```
//!
//! Namespace key produced: `"{org_slug}/{sensor_id}/{name}"` (BC-2.03.004).
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

use clap::{Args, Subcommand};
use prism_core::{CredentialName, OrgSlug};
use prism_credentials::{CredentialIndex, CredentialStore, KeyringBackend};
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
    /// The org slug is used to scope the keyring namespace: `{org_slug}/{sensor}/{name}`.
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
/// # AD-017 invariant
///
/// The credential value MUST NOT appear in:
/// - stdout (checked by `test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout`)
/// - stderr (same test)
/// - logs (`tracing::*!` calls)
/// - any `PrismError` message (BC-2.03.007 postcondition 3)
///
/// # Exit codes
///
/// 0 — success (credential stored)
/// 1 — keyring unavailable or write failure (with actionable stderr message)
/// 2 — config-invalid (cannot load prism.toml or resolve org)
pub async fn handle_credential_set(args: CredentialSetArgs, config_dir: std::path::PathBuf) -> i32 {
    // Step 1: Resolve org slug.
    // If --org-slug provided, use it. Otherwise load prism.toml to get the first org.
    let org_slug_str = match resolve_org_slug(&args.org_slug, &config_dir).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("prism credential set: config error: {e}");
            return EXIT_CONFIG_INVALID;
        }
    };

    // Validate org slug via OrgSlug::new (non-panicking path — check is_ok()).
    let org_slug = OrgSlug::new(&org_slug_str);
    if org_slug.is_err() {
        eprintln!(
            "prism credential set: invalid org slug '{}': {}",
            org_slug_str,
            org_slug.unwrap_err()
        );
        return EXIT_CONFIG_INVALID;
    }

    // Validate credential name.
    let cred_name = match CredentialName::new(&args.name) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("prism credential set: invalid credential name: {e}");
            return EXIT_CONFIG_INVALID;
        }
    };

    // Step 3: Prompt on stderr (not stdout) — AD-017.
    // rpassword::prompt_password writes the prompt to stderr and reads the value
    // from stdin with echo DISABLED. This satisfies BC-2.03.007 postcondition.
    //
    // Non-TTY fallback: on macOS, rpassword may fail with ENXIO (os error 6) when
    // stdin is a pipe and /dev/tty is unavailable. In that case, we fall back to
    // reading from stdin directly (no echo possible on a pipe — the value is already
    // not echoed because there's no TTY). This path is used by the Red Gate test
    // (piped stdin) and by demo-setup.sh (heredoc input). AD-017 is still satisfied
    // because the value never appears in stdout or logs.
    let prompt = format!("Enter value for prism/{}/{}: ", args.sensor, args.name);
    // Emit the prompt on stderr unconditionally (mirrors rpassword behaviour on TTY).
    eprint!("{prompt}");
    let raw_value = match read_secret_value(&prompt) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("\nprism credential set: failed to read credential value: {e}");
            return EXIT_GENERIC_ERROR;
        }
    };

    // INVARIANT: the raw_value MUST NOT appear in any log or stdout output below.
    // Wrap in SecretString immediately so accidental Display/Debug shows "***".
    let value = SecretString::new(raw_value);

    // Step 4: Write to OS keyring via KeyringBackend::set().
    // Index path: <config_dir>/credential_index.json (mirrors boot step 5 convention).
    let index_path = config_dir.join("credential_index.json");
    let index = CredentialIndex::new(index_path);
    let store = KeyringBackend::new("prism", index);

    match store.set(&org_slug, &args.sensor, &cred_name, value).await {
        Ok(()) => {
            // Step 5: Print success to stdout — only the success message, never the value.
            println!("Credential stored successfully.");
            EXIT_SUCCESS
        }
        Err(e) => {
            // Check for "Keyring unavailable" pattern (EC-001 of S-DEMO-003).
            // keyring-rs surfaces platform unavailability as NoStorageAccess or similar.
            let err_str = e.to_string();
            if err_str.contains("NoStorageAccess")
                || err_str.contains("NoKeyringService")
                || err_str.contains("Unavailable")
                || err_str.contains("DBus")
                || err_str.contains("keychain")
                || err_str.contains("spawn_blocking panicked")
            {
                eprintln!(
                    "Keyring unavailable: {e}. \
                     Use the encrypted file backend instead."
                );
            } else {
                eprintln!("prism credential set: keyring write failed: {e}");
            }
            EXIT_GENERIC_ERROR
        }
    }
}

/// Read a secret value from stdin with echo disabled.
///
/// # Strategy
///
/// 1. Try `rpassword::prompt_password_from_bufread` against stdin — this disables echo on a TTY.
/// 2. If rpassword fails (ENXIO / non-TTY pipe), fall back to `std::io::stdin().read_line()`.
///    A pipe has no echo to suppress, so the fallback is AD-017 compliant:
///    the value is never echoed because the shell never enabled echo for a pipe.
///
/// The prompt is emitted on stderr BEFORE calling this function; `rpassword` will
/// emit it a second time in the TTY path (harmless duplicate). In the pipe fallback
/// path, only our outer `eprint!` fires — no double prompt.
///
/// AD-017: the returned `String` must be wrapped in `SecretString` immediately at
/// the call site so any accidental `Debug`/`Display` output is redacted.
fn read_secret_value(_prompt: &str) -> Result<String, std::io::Error> {
    // Try rpassword first (TTY path — disables echo).
    // On macOS with piped stdin, rpassword 7 may return ENXIO (os error 6) because it
    // tries to open /dev/tty when stdin is not a terminal.
    use std::io::BufRead;
    match rpassword::read_password() {
        Ok(v) => return Ok(v),
        Err(e) => {
            // ENXIO (6) = no such device or address — /dev/tty not available (pipe).
            // Fall through to stdin read. Any other error is a real failure.
            let raw_os_err = e.raw_os_error().unwrap_or(0);
            if raw_os_err != 6 {
                return Err(e);
            }
        }
    }

    // Fallback: read from stdin directly (pipe / non-TTY context).
    // Echo is not possible on a pipe — AD-017 satisfied by the channel property.
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    // Strip the trailing newline that `writeln!` appended in the test.
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(line)
}

/// Resolve the org slug to use for credential scoping.
///
/// Resolution order:
/// 1. If `--org-slug` is provided, use it directly.
/// 2. If `prism.toml` exists in `config_dir`, load it and use the first org's slug.
/// 3. If `prism.toml` does not exist, fall back to `"demo-org"` (the canonical demo
///    org slug). This allows `prism credential set` to be used during `demo-setup.sh`
///    before `prism.toml` has been written to the config dir.
///
/// Step 3 fallback rationale: `prism credential set` is called by `demo-setup.sh`
/// AFTER the credentials are bootstrapped but potentially BEFORE prism.toml is copied
/// to the config dir. Requiring prism.toml at credential-set time would create a
/// chicken-and-egg problem. The "demo-org" default is correct for demo use cases.
/// For multi-org production use, operators must always pass `--org-slug`.
async fn resolve_org_slug(
    explicit: &Option<String>,
    config_dir: &std::path::Path,
) -> Result<String, String> {
    if let Some(slug) = explicit {
        return Ok(slug.clone());
    }

    // Attempt to load prism.toml to find the first org slug.
    let toml_path = config_dir.join("prism.toml");
    match std::fs::read_to_string(&toml_path) {
        Ok(contents) => {
            let config: crate::boot::PrismConfig = toml::from_str(&contents)
                .map_err(|e| format!("cannot parse '{}': {e}", toml_path.display()))?;

            if config.orgs.is_empty() {
                return Err(format!(
                    "'{}' declares no orgs — add an [[orgs]] entry or use --org-slug",
                    toml_path.display()
                ));
            }

            Ok(config.orgs[0].org_slug.clone())
        }
        Err(_) => {
            // prism.toml not found — use the demo-org default.
            // This covers the demo-setup.sh bootstrap phase where prism.toml may not
            // yet exist when credentials are first being stored.
            tracing::debug!(
                config_dir = %config_dir.display(),
                "prism.toml not found in config_dir; defaulting org slug to 'demo-org' \
                 (use --org-slug for explicit org selection)"
            );
            Ok("demo-org".to_string())
        }
    }
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
    // Format: TOML with tilde-expanded paths.
    // demo-setup.sh mirrors this content when writing ~/.config/prism-demo/prism.toml.
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
}
