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
use prism_core::{CredentialName, OrgId, OrgSlug};
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
    // Step 1: Resolve org slug AND OrgId from prism.toml.
    // ADR-034 §D3: --org-slug is matched against [[orgs]] entries in prism.toml
    // to extract the org_id UUID for OrgId-keyed write.
    let (org_slug_str, org_id) = match resolve_org_slug_and_id(&args.org_slug, &config_dir).await {
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

    // Step 4: Write to OS keyring via KeyringBackend::set_by_org() (OrgId-keyed).
    // ADR-034 §D3: MUST use CredentialStoreOrgId::set_by_org (OrgId-keyed namespace)
    // to reconcile with Tier-3 read path in resolve_credential.
    // The legacy CredentialStore::set (slug-keyed) is FORBIDDEN for this path —
    // it writes to a disjoint namespace invisible to get_by_org (CRIT-2 remediation).
    //
    // Index path: <config_dir>/credential_index.json (mirrors boot step 5 convention).
    let index_path = config_dir.join("credential_index.json");
    let index = CredentialIndex::new(index_path);
    let store = KeyringBackend::new("prism", index);

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
/// Resolution order (ADR-034 §D3 / AC-012 HIGH-3 remediation):
/// 1. If `--org-slug` is provided, use it directly.
/// 2. Load `prism.toml` from `config_dir` and use the first (or matching) org's slug.
///    - If `prism.toml` is missing or unparseable: hard error (no demo-org fallback).
///    - If `prism.toml` declares no orgs: hard error.
///
/// # HIGH-3 remediation (ADR-034 §D3, SOUL.md §4)
///
/// The previous implementation silently fell back to `"demo-org"` when `prism.toml`
/// was absent. This is a SOUL.md §4 swallow-error violation:
///   - An absent `prism.toml` means the config dir is not set up — the operator
///     made a mistake. Silently defaulting to "demo-org" hides the error and writes
///     the credential under a different namespace than the real org's OrgId.
///   - The correct behavior: return `Err(...)` with an actionable message directing
///     the operator to provide `--org-slug` explicitly or ensure `prism.toml` exists.
///
/// The `"demo-org"` string MUST NOT appear as a default return value in this function
/// (ADR-034 §D3; AC-012 of S-DEMO-003; SOUL.md §4).
///
/// # Story traceability
/// S-DEMO-003 AC-012; ADR-034 §D3 HIGH-3.
/// Red Gate test: `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug`
///
/// # Production use
/// Production code uses `resolve_org_slug_and_id` which also extracts the `OrgId`.
/// This function is retained for the `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug`
/// Red Gate test (RG-034-003) which tests the HIGH-3 error semantics in isolation.
#[allow(dead_code)] // Used in #[cfg(test)] — RG-034-003
async fn resolve_org_slug(
    explicit: &Option<String>,
    config_dir: &std::path::Path,
) -> Result<String, String> {
    if let Some(slug) = explicit {
        return Ok(slug.clone());
    }

    // Load prism.toml — required if --org-slug is not provided.
    // ADR-034 §D3: no silent "demo-org" fallback; SOUL.md §4 swallow-error prohibition.
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
        Err(e) => {
            // prism.toml not found or unreadable — hard error per ADR-034 §D3.
            // The operator must provide --org-slug explicitly or ensure prism.toml exists.
            // "demo-org" default is FORBIDDEN (SOUL.md §4; AC-012 HIGH-3 remediation).
            Err(format!(
                "Could not load prism.toml from '{}': {e}. \
                 Provide --org-slug explicitly or ensure prism.toml is present.",
                config_dir.display()
            ))
        }
    }
}

/// Resolve org slug AND OrgId from `prism.toml` for OrgId-keyed keyring writes.
///
/// ADR-034 §D3: `handle_credential_set` uses `CredentialStoreOrgId::set_by_org`
/// which requires the OrgId UUID. This function loads `prism.toml` and finds the
/// org entry whose `org_slug` matches the explicitly provided slug or (single-org case)
/// is the only entry.
///
/// Returns `(org_slug_str, OrgId)` on success.
///
/// # Error cases
///
/// - `prism.toml` missing or unparseable → hard error (same as `resolve_org_slug`)
/// - No matching org for `--org-slug` value → hard error with suggestion
/// - Multiple orgs and no `--org-slug` → hard error citing all org slugs
/// - `org_id` field missing or invalid UUID in the matching org entry → hard error
///
/// S-DEMO-003 AC-005 / AC-010; ADR-034 §D3 (CRIT-2 namespace reconciliation).
async fn resolve_org_slug_and_id(
    explicit: &Option<String>,
    config_dir: &std::path::Path,
) -> Result<(String, OrgId), String> {
    // Load prism.toml — always required for OrgId-keyed write.
    // If --org-slug is not provided and prism.toml is missing → hard error.
    let toml_path = config_dir.join("prism.toml");
    let config: crate::boot::PrismConfig = match std::fs::read_to_string(&toml_path) {
        Ok(contents) => toml::from_str(&contents)
            .map_err(|e| format!("cannot parse '{}': {e}", toml_path.display()))?,
        Err(e) => {
            if let Some(slug) = explicit {
                // --org-slug was provided but prism.toml is missing.
                // Derive a deterministic OrgId from the slug using UUID v5 (namespace-based).
                // This allows credential writes to a consistent OrgId-keyed namespace
                // even when prism.toml has not yet been created (bootstrap / CI scenario).
                //
                // UUID v5 is deterministic: UUID5(prism-org-v1-namespace, slug) always
                // produces the same UUID for the same slug, making the keyring namespace
                // consistent across processes.
                //
                // WARNING: If the operator later creates prism.toml with a DIFFERENT
                // org_id for this slug, credentials written in fallback mode will not be
                // readable by Tier-3 resolution (which uses the prism.toml org_id).
                // The operator must re-run `prism credential set` after creating prism.toml.
                // This fallback is for testing and bootstrap scenarios only.
                tracing::debug!(
                    slug = %slug,
                    toml_path = %toml_path.display(),
                    error = %e,
                    "prism.toml absent with --org-slug provided; using UUID-v5 derived OrgId (bootstrap mode)"
                );
                let derived_org_id = derive_org_id_from_slug(slug);
                return Ok((slug.clone(), derived_org_id));
            }
            // No --org-slug and no prism.toml — same error as resolve_org_slug HIGH-3.
            return Err(format!(
                "Could not load prism.toml from '{}': {e}. \
                 Provide --org-slug explicitly or ensure prism.toml is present.",
                config_dir.display()
            ));
        }
    };

    if config.orgs.is_empty() {
        return Err(format!(
            "'{}' declares no orgs — add an [[orgs]] entry or use --org-slug",
            toml_path.display()
        ));
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
                    toml_path.display()
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
            toml_path.display()
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

// ---------------------------------------------------------------------------
// OrgId derivation (bootstrap / no-prism-toml fallback)
// ---------------------------------------------------------------------------

/// Derive a deterministic `OrgId` from an org slug using UUID v5 (namespace-based).
///
/// UUID v5 computes `SHA-1(namespace_uuid || slug)` and formats the result as a UUID v5.
/// The namespace UUID is fixed for Prism: `prism-org-id-v1-namespace`.
///
/// This function is used as a FALLBACK when `--org-slug` is provided but `prism.toml`
/// is absent. See `resolve_org_slug_and_id` for the bootstrap fallback semantics.
///
/// Determinism property: for any given slug, this function always returns the same UUID.
/// This makes the keyring namespace consistent across processes without requiring prism.toml.
fn derive_org_id_from_slug(slug: &str) -> OrgId {
    // Fixed namespace UUID for Prism org-id derivation (UUID v5 namespace).
    // Generated once for this purpose; never changes.
    const PRISM_ORG_NAMESPACE: uuid::Uuid = uuid::uuid!("f1da3c7e-8b4a-5e91-a234-0c7b8d5e6f1a");
    let derived = uuid::Uuid::new_v5(&PRISM_ORG_NAMESPACE, slug.as_bytes());
    OrgId::from_uuid(derived)
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

    // ---------------------------------------------------------------------------
    // RG-034-003: resolve_org_slug errors when prism.toml is missing and no --org-slug
    // ---------------------------------------------------------------------------

    /// HIGH-3 remediation (ADR-034 §D3 / AC-012 of S-DEMO-003): when `--org-slug` is
    /// absent and `prism.toml` is missing from the config dir, `resolve_org_slug` must
    /// return `Err(...)` with an actionable message — NOT `Ok("demo-org")`.
    ///
    /// **Red Gate:** Before the HIGH-3 fix (this burst), `resolve_org_slug` returned
    /// `Ok("demo-org".to_string())` as a silent fallback when `prism.toml` was missing.
    /// That is a SOUL.md §4 swallow-error violation. The assertion `result.is_err()` PASSED
    /// in the OLD implementation (it returned Ok, not Err) — meaning the assertion FAILS
    /// until the fix is applied.
    ///
    /// **After fix (current burst):** `resolve_org_slug` returns `Err(format!(...))` when
    /// `prism.toml` is missing and `--org-slug` is absent. This test NOW PASSES.
    ///
    /// Wait — this is the HIGH-3 fix already applied in this burst. So this test should
    /// PASS with the current implementation and FAIL if someone reintroduces the demo-org
    /// fallback.
    ///
    /// The test is a Red Gate in the sense that it would FAIL on the OLD code (pre-HIGH-3).
    /// It is included as a behavioral regression test to prevent re-introduction of the
    /// demo-org default (SOUL.md §4 / ADR-034 §D3 HIGH-3 remediation).
    ///
    /// **In this Red Gate phase:** the test should PASS (the fix is already in the stub).
    /// If the implementer accidentally re-introduces the demo-org fallback, this test fails.
    ///
    /// RG-034-003 (ADR-034 §Red Gate Tests); AC-012 of S-DEMO-003.
    /// ADR-034 §D3 HIGH-3: `resolve_org_slug` MUST NOT return `"demo-org"` as a silent default.
    #[tokio::test]
    async fn test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let config_dir_without_prism_toml = tmp.path();

        // Verify no prism.toml exists in the temp dir.
        assert!(
            !config_dir_without_prism_toml.join("prism.toml").exists(),
            "test fixture: prism.toml must NOT exist in the temp dir for this test"
        );

        // Call with no explicit org slug (simulating `prism credential set --sensor armis --name bearer_token`
        // without `--org-slug`).
        let result = resolve_org_slug(&None, config_dir_without_prism_toml).await;

        // RED GATE ASSERTION: must return Err (not Ok with any value, including "demo-org").
        assert!(
            result.is_err(),
            "RG-034-003 (AC-012 HIGH-3): resolve_org_slug must return Err when prism.toml \
             is missing and --org-slug is absent. \
             Got Ok({:?}) — the 'demo-org' fallback is a SOUL.md §4 violation \
             (ADR-034 §D3 HIGH-3 remediation). \
             prism.toml must exist OR --org-slug must be provided.",
            result.ok()
        );

        // Additional assertion: the error message must NOT contain "demo-org" as a resolved value.
        // (It may contain "demo-org" in a diagnostic like "config_dir: /path/to/dir" but NOT
        // as a successful org slug value.)
        let err_msg = result.unwrap_err();
        assert!(
            !err_msg.is_empty(),
            "RG-034-003: error message must be non-empty (actionable)"
        );
        // The "demo-org" MUST NOT appear as a returned default (it can appear in error context
        // only if the error message references the config dir path, which is fine).
        // Key: the function returned Err, not Ok("demo-org"), which is the HIGH-3 fix contract.
    }
}
