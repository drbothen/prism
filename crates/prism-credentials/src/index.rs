//! CredentialIndex — sidecar plaintext JSON index for KeyringBackend list().
//!
//! keyring-rs has no enumeration API, so `KeyringBackend::list()` uses this
//! helper to maintain a `Vec<String>` of namespace keys in a plaintext JSON
//! file. The file contains NO credential values — only namespace keys — so
//! it is safe to store unencrypted.
//!
//! Atomic writes (tmp + rename) prevent partial corruption.
//!
//! Story: S-1.06 | Referenced by: BC-2.03.002 (KeyringBackend list support)

use std::fs;
use std::path::PathBuf;

use prism_core::PrismError;

/// Sidecar credential index for `KeyringBackend`.
pub struct CredentialIndex {
    path: PathBuf,
    /// In-memory cache of the index (loaded lazily on first access).
    keys: Vec<String>,
    /// Whether the in-memory cache has been loaded from disk.
    loaded: bool,
}

impl CredentialIndex {
    /// Create a new `CredentialIndex` backed by `path`.
    ///
    /// Does not read from disk yet — loading is deferred to first access.
    pub fn new(path: PathBuf) -> Self {
        CredentialIndex {
            path,
            keys: Vec::new(),
            loaded: false,
        }
    }

    /// Load from disk if not yet loaded. No-op if already loaded.
    fn ensure_loaded(&mut self) -> Result<(), PrismError> {
        if self.loaded {
            return Ok(());
        }
        if self.path.exists() {
            let content =
                fs::read_to_string(&self.path).map_err(|e| PrismError::Io(e.to_string()))?;
            self.keys =
                serde_json::from_str(&content).map_err(|e| PrismError::CredentialStoreError {
                    backend: "index".to_owned(),
                    reason: format!("failed to parse credential index: {e}"),
                })?;
        }
        self.loaded = true;
        Ok(())
    }

    /// Persist the in-memory keys to disk using atomic tmp+rename.
    fn persist(&self) -> Result<(), PrismError> {
        let json =
            serde_json::to_string(&self.keys).map_err(|e| PrismError::CredentialStoreError {
                backend: "index".to_owned(),
                reason: format!("failed to serialize credential index: {e}"),
            })?;

        // Ensure parent directory exists.
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| PrismError::Io(e.to_string()))?;
        }

        // Atomic write: write to tmp, then rename.
        let tmp_path = self.path.with_extension("tmp");
        fs::write(&tmp_path, &json).map_err(|e| PrismError::Io(e.to_string()))?;
        fs::rename(&tmp_path, &self.path).map_err(|e| PrismError::Io(e.to_string()))?;

        Ok(())
    }

    /// Add a namespace key to the index. No-op if already present.
    pub fn add(&mut self, key: &str) -> Result<(), PrismError> {
        self.ensure_loaded()?;
        if !self.keys.contains(&key.to_owned()) {
            self.keys.push(key.to_owned());
            self.persist()?;
        }
        Ok(())
    }

    /// Remove a namespace key from the index. No-op if not present.
    pub fn remove(&mut self, key: &str) -> Result<(), PrismError> {
        self.ensure_loaded()?;
        let before_len = self.keys.len();
        self.keys.retain(|k| k != key);
        if self.keys.len() != before_len {
            self.persist()?;
        }
        Ok(())
    }

    /// Return all stored namespace keys.
    pub fn list(&mut self) -> Result<Vec<String>, PrismError> {
        self.ensure_loaded()?;
        Ok(self.keys.clone())
    }
}
