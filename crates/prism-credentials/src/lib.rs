//! prism-credentials — Credential Store Trait and Backends
//!
//! Story: S-1.06
//! BCs: BC-2.03.001, BC-2.03.002, BC-2.03.003, BC-2.03.004,
//!      BC-2.03.008, BC-2.03.011, BC-2.03.012
//! VPs: VP-034 (AES-256-GCM round-trip), VP-035 (key derivation deterministic)

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

// Public re-exports
pub use error::{CredentialBackend, CredentialError};
pub use file::EncryptedFileBackend;
pub use index::CredentialIndex;
pub use keyring::KeyringBackend;
pub use namespace::namespace_key;
pub use probe::{probe_keyring, KeyringStatus};
pub use selector::{BackendSelector, CredentialConfig};
pub use trait_::CredentialStore;
