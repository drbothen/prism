//! resolve_secret() — {NAME}_FILE env var → {NAME} env var → None.
//!
//! # Contract: BC-2.03.009
//! Priority order (matches K8s secret mount convention from all 4 Go pollers):
//!   1. `{file_env}` — env var holding a file path; read file and strip trailing newline
//!   2. `{direct_env}` — env var holding the value directly
//!   3. Neither set → return `None`
//!
//! The function signature accepts `file_env` and `direct_env` as separate parameters
//! matching the BC contract: `resolve_secret(file_env, direct_env)`.

use secrecy::SecretString;

/// Resolve a secret using the _FILE env var pattern.
///
/// # Arguments
/// - `file_env`: name of the env var that may hold a file path (e.g. `"CROWDSTRIKE_API_KEY_FILE"`)
/// - `direct_env`: name of the env var that may hold the value directly (e.g. `"CROWDSTRIKE_API_KEY"`)
///
/// # Returns
/// - `Ok(Some(SecretString))` — resolved from file or direct env var
/// - `Ok(None)` — neither env var is set
/// - `Err(PrismError::Credential)` — `file_env` points to a non-existent or unreadable file
///
/// # Contract: BC-2.03.009
/// - File contents have trailing newline stripped.
/// - File takes precedence if both are set.
/// - File must be a regular file (not a directory).
pub fn resolve_secret(
    file_env: &str,
    direct_env: &str,
) -> Result<Option<SecretString>, prism_core::PrismError> {
    todo!("S-1.07: implement resolve_secret")
}
