//! BackendSelector — credential backend selection and fallback — BC-2.03.012
//!
//! Selects exactly one backend per session:
//!
//! | config.backend | keyring probe | result |
//! |---------------|--------------|--------|
//! | "keyring"     | passes       | KeyringBackend |
//! | "keyring"     | fails        | Err (hard error — no silent downgrade) |
//! | "auto"        | passes       | KeyringBackend |
//! | "auto"        | fails        | EncryptedFileBackend + WARN log |
//! | "file"        | (skipped)    | EncryptedFileBackend |
//!
//! `passphrase_env` stores the NAME of an env var, not the passphrase itself.
//! `BackendSelector` calls `std::env::var(passphrase_env)` at runtime.
//!
//! Story: S-1.06 | BC: BC-2.03.012

use std::path::PathBuf;

use prism_core::PrismError;
use secrecy::SecretString;
use tracing::warn;

use crate::{
    file::EncryptedFileBackend,
    index::CredentialIndex,
    keyring::KeyringBackend,
    probe::{probe_keyring, KeyringStatus},
    trait_::CredentialStore,
};

/// Configuration for credential backend selection.
#[derive(Debug, Clone)]
pub struct CredentialConfig {
    /// Backend preference: `"keyring"`, `"auto"`, or `"file"`.
    pub backend: String,
    /// Base directory for `EncryptedFileBackend` credential files.
    pub file_path: Option<PathBuf>,
    /// Name of the environment variable that holds the file backend passphrase.
    /// NOT the passphrase itself — the env var name only.
    pub passphrase_env: Option<String>,
}

/// Selects the appropriate credential backend based on configuration and
/// runtime probe results.
pub struct BackendSelector;

impl BackendSelector {
    /// Select and return the active credential store backend.
    ///
    /// Returns `Box<dyn CredentialStore>` on success or `Err(PrismError)` if
    /// the requested backend is unavailable and no fallback is permitted.
    pub async fn select_backend(
        config: &CredentialConfig,
    ) -> Result<Box<dyn CredentialStore>, PrismError> {
        match config.backend.as_str() {
            "keyring" => {
                // Explicit keyring: probe must succeed or we return a hard error.
                let status = probe_keyring("prism").await;
                match status {
                    KeyringStatus::Available => {
                        Ok(Box::new(KeyringBackend::new("prism", Self::default_index(config))))
                    }
                    KeyringStatus::Unavailable(reason) => {
                        Err(PrismError::CredentialStoreError {
                            backend: "keyring".to_owned(),
                            reason: format!(
                                "explicit keyring backend requested but keyring is unavailable: {reason}"
                            ),
                        })
                    }
                }
            }

            "auto" => {
                // Auto: try keyring, fall back to encrypted file with WARN.
                let status = probe_keyring("prism").await;
                match status {
                    KeyringStatus::Available => Ok(Box::new(KeyringBackend::new(
                        "prism",
                        Self::default_index(config),
                    ))),
                    KeyringStatus::Unavailable(reason) => {
                        warn!(
                            reason = %reason,
                            "BackendSelector: keyring unavailable in auto mode — \
                             falling back to encrypted file backend"
                        );
                        Ok(Box::new(Self::build_file_backend(config)?))
                    }
                }
            }

            "file" => {
                // Explicit file: skip probe, build encrypted file backend directly.
                Ok(Box::new(Self::build_file_backend(config)?))
            }

            other => Err(PrismError::CredentialStoreError {
                backend: other.to_owned(),
                reason: format!(
                    "unknown backend '{other}'; valid values: 'keyring', 'auto', 'file'"
                ),
            }),
        }
    }

    /// Build an `EncryptedFileBackend` from config.
    ///
    /// Reads passphrase from the env var named in `config.passphrase_env`.
    fn build_file_backend(config: &CredentialConfig) -> Result<EncryptedFileBackend, PrismError> {
        let base_dir =
            config
                .file_path
                .clone()
                .ok_or_else(|| PrismError::CredentialStoreError {
                    backend: "encrypted-file".to_owned(),
                    reason: "file backend requires 'file_path' to be set".to_owned(),
                })?;

        let passphrase_env =
            config
                .passphrase_env
                .as_deref()
                .ok_or_else(|| PrismError::EncryptionKeyMissing {
                    reason: "file backend requires 'passphrase_env' to be set".to_owned(),
                })?;

        let passphrase_str =
            std::env::var(passphrase_env).map_err(|_| PrismError::EncryptionKeyMissing {
                reason: format!(
                    "env var '{passphrase_env}' is not set; \
                     set it to the master passphrase for the encrypted file backend"
                ),
            })?;

        if passphrase_str.is_empty() {
            return Err(PrismError::EncryptionKeyMissing {
                reason: format!(
                    "env var '{passphrase_env}' is empty; passphrase must not be empty"
                ),
            });
        }

        Ok(EncryptedFileBackend::new(
            base_dir,
            SecretString::new(passphrase_str),
        ))
    }

    /// Build a `CredentialIndex` for `KeyringBackend`.
    ///
    /// Stored alongside the file backend directory (or in a OS-appropriate dir).
    fn default_index(config: &CredentialConfig) -> CredentialIndex {
        let index_path = if let Some(dir) = &config.file_path {
            dir.join(".prism-credential-index.json")
        } else {
            // No file_path configured — store index in home dir as fallback.
            // In production, file_path should always be set.
            let base = std::env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."));
            base.join(".prism").join("credential-index.json")
        };
        CredentialIndex::new(index_path)
    }
}
