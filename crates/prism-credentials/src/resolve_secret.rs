//! resolve_secret() — {NAME}_FILE env var → {NAME} env var → None.
//!
//! # Contract: BC-2.03.009
//! Priority order (matches K8s secret mount convention):
//!   1. `{file_env}` — env var holding a file path; read file and strip trailing newline
//!   2. `{direct_env}` — env var holding the value directly
//!   3. Neither set → return `None`

use secrecy::SecretString;
use std::path::Path;

/// Resolve a secret using the _FILE env var pattern.
///
/// # Arguments
/// - `file_env`: name of the env var that may hold a file path (e.g. `"CROWDSTRIKE_API_KEY_FILE"`)
/// - `direct_env`: name of the env var that may hold the value directly (e.g. `"CROWDSTRIKE_API_KEY"`)
///
/// # Returns
/// - `Ok(Some(SecretString))` — resolved from file or direct env var
/// - `Ok(None)` — neither env var is set
/// - `Err(PrismError)` — `file_env` points to a non-existent or unreadable file
///
/// # Contract: BC-2.03.009
/// - File contents have trailing newline stripped.
/// - File takes precedence if both are set.
/// - File must be a regular file (not a directory).
pub fn resolve_secret(
    file_env: &str,
    direct_env: &str,
) -> Result<Option<SecretString>, prism_core::PrismError> {
    // Priority 1: check {file_env} env var
    if let Ok(file_path) = std::env::var(file_env) {
        let path = Path::new(&file_path);

        // Verify it exists
        if !path.exists() {
            return Err(prism_core::PrismError::InvalidCredentialName {
                name: file_path.clone(),
                reason: format!(
                    "E-CRED-009: credential file does not exist at path '{}' (env var '{}')",
                    file_path, file_env
                ),
            });
        }

        // Verify it is a regular file, not a directory
        if path.is_dir() {
            return Err(prism_core::PrismError::InvalidCredentialName {
                name: file_path.clone(),
                reason: format!(
                    "E-CRED-009: path '{}' points to a directory, not a regular file — credential must be a regular file",
                    file_path
                ),
            });
        }

        // Read the file content
        let content = std::fs::read_to_string(path).map_err(|e| {
            prism_core::PrismError::InvalidCredentialName {
                name: file_path.clone(),
                reason: format!(
                    "E-CRED-009: failed to read credential file '{}': {}",
                    file_path, e
                ),
            }
        })?;

        // Strip trailing newline per BC-2.03.009
        let trimmed = content
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string();

        return Ok(Some(SecretString::new(trimmed.into())));
    }

    // Priority 2: check {direct_env} env var
    if let Ok(value) = std::env::var(direct_env) {
        return Ok(Some(SecretString::new(value.into())));
    }

    // Priority 3: neither set
    Ok(None)
}
