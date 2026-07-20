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

// ---------------------------------------------------------------------------
// F-P10-CRIT-001 Regression Guard — keyring real-backend compile-time assertion.
//
// keyring 3.x silently falls back to the in-process MOCK keystore when no real
// backend feature is enabled. The mock is cross-process–invisible: `prism credential set`
// (process A) and `prism start` (process B) each get a separate in-memory store,
// so Tier-3 get_by_org always returns Ok(None) → all 4 demo sensors fail auth.
//
// These compile_error! guards prevent silent reversion to the mock keystore.
// They check prism-credentials' own pass-through features (keyring-apple-native,
// keyring-windows-native, keyring-linux-native-sync-persistent, keyring-linux-native)
// which are declared in [features] and enable the corresponding keyring backend features.
//
// Two-place update invariant (linux-gnu): the linux-native-sync-persistent backend is
// activated via [target.'cfg(all(target_os = "linux", not(target_env = "musl")))'.dependencies],
// NOT via the [features] default. Removing that target-cfg block will NOT trip this guard
// (because keyring-linux-native remains in default). However it will silently downgrade
// linux-gnu to keyutils-only (no persistent Secret Service). Removing keyring-linux-native
// from [features] default WILL trip the Linux guard on all Linux targets.
// musl: keyring-linux-native in [features] default is the sole backend; no dbus.
//
// guard: macOS — keyring-apple-native (security-framework, Keychain) required.
#[cfg(all(target_os = "macos", not(feature = "keyring-apple-native")))]
compile_error!(
    "prism-credentials: `keyring-apple-native` feature MUST be enabled on macOS. \
     Removing it reverts to the in-process mock keystore (F-P10-CRIT-001). \
     Add `keyring-apple-native` to prism-credentials features and \
     `apple-native` to the keyring dependency features in Cargo.toml."
);
// guard: iOS — keyring-apple-native (security-framework, iOS Keychain) required.
#[cfg(all(target_os = "ios", not(feature = "keyring-apple-native")))]
compile_error!(
    "prism-credentials: `keyring-apple-native` feature MUST be enabled on iOS. \
     Removing it reverts to the in-process mock keystore (F-P10-CRIT-001)."
);
// guard: Windows — keyring-windows-native (windows-sys, Credential Store) required.
#[cfg(all(target_os = "windows", not(feature = "keyring-windows-native")))]
compile_error!(
    "prism-credentials: `keyring-windows-native` feature MUST be enabled on Windows. \
     Removing it reverts to the in-process mock keystore (F-P10-CRIT-001). \
     Add `keyring-windows-native` to prism-credentials features in Cargo.toml."
);
// guard: Linux — at least one real backend pass-through feature required.
// keyring-linux-native-sync-persistent: kernel keyutils + dbus-secret-service (persistent).
// keyring-linux-native: kernel keyutils only (musl, no dbus).
// Either satisfies the guard — musl uses keyring-linux-native alone.
#[cfg(all(
    target_os = "linux",
    not(any(
        feature = "keyring-linux-native-sync-persistent",
        feature = "keyring-linux-native",
    ))
))]
compile_error!(
    "prism-credentials: `keyring-linux-native-sync-persistent` or `keyring-linux-native` \
     MUST be enabled on Linux. Removing all backend features reverts to the in-process \
     mock keystore (F-P10-CRIT-001). \
     For gnu targets: enable `keyring-linux-native-sync-persistent`. \
     For musl targets: enable `keyring-linux-native`."
);
// ---------------------------------------------------------------------------

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
