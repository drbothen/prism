//! Internal error types and backend identifier for prism-credentials.

/// Which backend produced an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialBackend {
    Keyring,
    EncryptedFile,
}

impl std::fmt::Display for CredentialBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialBackend::Keyring => write!(f, "keyring"),
            CredentialBackend::EncryptedFile => write!(f, "encrypted-file"),
        }
    }
}

/// Re-export PrismError as the canonical error type for this crate.
pub type CredentialError = prism_core::PrismError;
