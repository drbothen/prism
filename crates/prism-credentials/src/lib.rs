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

// test-helpers feature: in-memory CredentialStoreOrgId trait double for integration tests.
// AD-017: InMemoryCredentialStore stores values in heap memory only (SecretString, zeroized on drop).
// Never enabled in production builds — feature is for test binaries only.
#[cfg(feature = "test-helpers")]
pub mod in_memory_store;

// S-1.07 modules
pub mod audit;
pub mod crud;
pub mod resolution;
pub mod resolve_secret;
pub mod secret;

// S-1.06 re-exports
// S-1.07 re-exports
pub use audit::{AuditEvent, AuditOperation, AuditOutcome};
pub use crud::{
    configure_credential_source, credential_status, delete_credential, list_credentials,
    ConfigureCredentialRequest, ConfigureCredentialResponse, ConfirmationRequired,
    CredentialMetadata, CredentialRef, CredentialRefKind, CredentialStatusResponse,
};
pub use error::{CredentialBackend, CredentialError};
pub use file::EncryptedFileBackend;
pub use index::CredentialIndex;
pub use keyring::KeyringBackend;
// S-3.1.04 / BC-3.2.002 re-exports — OrgId-keyed namespace (BC-3.2.002)
pub use namespace::namespace_key_by_org_id;
pub use probe::{probe_keyring, KeyringStatus};
pub use resolution::{resolve_credential, CredentialResolutionError};
pub use resolve_secret::resolve_secret;
pub use secret::Secret;
pub use selector::{BackendSelector, CredentialConfig};
pub use trait_::{CredentialStore, CredentialStoreOrgId};

#[cfg(feature = "test-helpers")]
pub use in_memory_store::InMemoryCredentialStore;
