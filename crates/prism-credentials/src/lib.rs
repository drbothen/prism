//! prism-credentials — Credential Store Trait, Backends, CRUD, Resolution, Secret Redaction, and Audit Logging.
//!
//! # S-1.06 Modules (storage layer)
//! - [`error`] — CredentialError / CredentialBackend types
//! - [`file`] — EncryptedFileBackend (AES-256-GCM)
//! - [`index`] — CredentialIndex for keyring enumeration
//! - [`keyring`] — KeyringBackend
//! - [`namespace`] — namespace_key() helper
//! - [`probe`] — probe_keyring() readiness check
//! - [`selector`] — BackendSelector, CredentialConfig
//! - [`trait_`] — CredentialStore async trait
//!
//! # S-1.07 Modules (CRUD, resolution, security)
//! - [`crud`] — configure_credential_source, credential_status, delete_credential, list_credentials
//! - [`resolution`] — query-time credential resolution chain
//! - [`secret`] — Secret<T> wrapper; Display/Debug all output "Secret(***)"
//! - [`resolve_secret`] — resolve_secret() with {NAME}_FILE → {NAME} env var chain
//! - [`audit`] — AuditEvent emission for all credential access

// S-1.06 modules
pub mod error;
pub mod file;
pub mod index;
pub mod keyring;
pub mod namespace;
pub mod probe;
pub mod selector;
pub mod trait_;

#[cfg(test)]
pub mod tests;

// S-1.07 modules
pub mod audit;
pub mod crud;
pub mod resolution;
pub mod resolve_secret;
pub mod secret;

// S-1.06 re-exports
pub use error::{CredentialBackend, CredentialError};
pub use file::EncryptedFileBackend;
pub use index::CredentialIndex;
pub use keyring::KeyringBackend;
pub use probe::{probe_keyring, KeyringStatus};
pub use selector::{BackendSelector, CredentialConfig};
pub use trait_::CredentialStore;

// S-3.1.04 / BC-3.2.002 re-exports — OrgId-keyed namespace (BC-3.2.002)
pub use namespace::namespace_key_by_org_id;
pub use trait_::CredentialStoreOrgId;

// S-1.07 re-exports
pub use audit::{AuditEvent, AuditOperation, AuditOutcome};
pub use crud::{
    configure_credential_source, credential_status, delete_credential, list_credentials,
    ConfigureCredentialRequest, ConfigureCredentialResponse, ConfirmationRequired,
    CredentialMetadata, CredentialRef, CredentialRefKind, CredentialStatusResponse,
};
pub use resolution::{resolve_credential, CredentialResolutionError};
pub use resolve_secret::resolve_secret;
pub use secret::Secret;
