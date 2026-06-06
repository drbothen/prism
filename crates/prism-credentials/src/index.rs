//! CredentialIndex — sidecar plaintext JSON index for KeyringBackend list().
//!
//! keyring-rs has no enumeration API, so `KeyringBackend::list()` uses this
//! helper to maintain a `Vec<String>` of namespace keys in a plaintext JSON
//! file. The file contains NO credential values — only namespace keys — so
//! it is safe to store unencrypted.
//!
//! ## Portable Credential Cache (cross-process fallback)
//!
//! On macOS, the OS keychain ACL may prevent cross-process reads when both
//! processes use different code-signing identities (e.g., unsigned test binaries).
//! `KeyringBackend` writes to a portable AES-256-GCM credential cache alongside
//! the index file as a cross-process fallback. The cache is encrypted with a key
//! derived from the `app_name` via SHA-256, so any process with the same app
//! name can decrypt it.
//!
//! Security model: the OS keychain is the PRIMARY secure store. The file cache is a
//! SECONDARY fallback for environments where the OS keychain ACL prevents cross-process
//! reads (macOS test binaries, Linux CI without libsecret, Docker). The cache file
//! is encrypted; it is not plaintext on disk.
//!
//! Cache file path: `<index_path_stem>_cache.json` alongside the index file.
//!
//! Atomic writes (tmp + rename) prevent partial corruption.
//!
//! Story: S-1.06 | Referenced by: BC-2.03.002 (KeyringBackend list support)
//! ADR-034: cross-process fallback for environments where OS keychain ACL blocks reads.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use aes_gcm::{
    aead::{Aead, Payload},
    Aes256Gcm, KeyInit, Nonce,
};
use prism_core::PrismError;
use secrecy::{ExposeSecret, SecretString};
use tracing::debug;

/// Cache file version tag — embedded in the JSON for future format evolution.
const CACHE_VERSION: u32 = 1;

/// Derive a 32-byte AES-256 encryption key from an `app_name` using SHA-256.
///
/// Key material: `SHA-256(app_name || "\x00prism-cred-cache-v1")`.
/// This is a deterministic per-app-name key — any process with the same
/// app_name can decrypt the cache file. Security model: provides AES-GCM
/// confidentiality for the file at rest; the key is predictable from the
/// app_name (not a high-entropy secret). The OS keychain remains the
/// authoritative secure store; this is a cross-process portability layer.
fn derive_cache_key(app_name: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(app_name.as_bytes());
    hasher.update(b"\x00prism-cred-cache-v1");
    hasher.finalize().into()
}

/// On-disk JSON structure for the credential cache file.
#[derive(serde::Serialize, serde::Deserialize)]
struct CacheFile {
    version: u32,
    /// Map from namespace_key → base64-encoded `nonce (12 bytes) || ciphertext`.
    entries: HashMap<String, String>,
}

impl CacheFile {
    fn new() -> Self {
        CacheFile {
            version: CACHE_VERSION,
            entries: HashMap::new(),
        }
    }
}

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

    /// Compute the companion cache file path: `<stem>_cache.json`.
    pub(crate) fn cache_path(&self) -> PathBuf {
        let stem = self
            .path
            .file_stem()
            .unwrap_or(std::ffi::OsStr::new("credential_index"))
            .to_os_string();
        let mut cache_stem = stem;
        cache_stem.push("_cache");
        let mut cache_path = self.path.clone();
        cache_path.set_file_name(cache_stem);
        cache_path.set_extension("json");
        cache_path
    }

    /// Load from disk if not yet loaded. No-op if already loaded.
    fn ensure_loaded(&mut self) -> Result<(), PrismError> {
        if self.loaded {
            return Ok(());
        }
        if self.path.exists() {
            let content =
                fs::read_to_string(&self.path).map_err(|e| PrismError::Io(e.to_string()))?;
            // SEC-005: log the raw serde error at DEBUG only; return a sanitized
            // message so credential index structure is not leaked in error logs.
            self.keys = serde_json::from_str(&content).map_err(|e| {
                debug!(error = %e, path = %self.path.display(), "credential index parse error");
                PrismError::CredentialStoreError {
                    backend: "index".to_owned(),
                    reason: "credential index is corrupt or unreadable".to_owned(),
                }
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

    // ---------------------------------------------------------------------------
    // Portable credential cache — cross-process fallback (ADR-034)
    // ---------------------------------------------------------------------------

    /// Store a credential value in the cross-process file cache.
    ///
    /// The value is encrypted with AES-256-GCM using a key derived from `app_name`.
    /// The encrypted blob is base64-encoded and stored in the companion cache file
    /// (`<index_path_stem>_cache.json`).
    ///
    /// This is called by `KeyringBackend::set_by_org` after a successful OS keychain
    /// write. The cache enables cross-process reads in environments where the OS
    /// keychain ACL prevents different binaries from reading each other's entries.
    ///
    /// AD-017: the plaintext value is never written to disk; only the AES-GCM
    /// ciphertext is stored. The encryption key is deterministic from `app_name`.
    pub(crate) fn store_value_in_cache(
        &self,
        namespace_key: &str,
        value: &SecretString,
        app_name: &str,
    ) -> Result<(), PrismError> {
        let cache_path = self.cache_path();

        // Load existing cache or create new.
        let mut cache_file = load_cache_file(&cache_path)?;

        // Derive the encryption key from app_name.
        let key_bytes = derive_cache_key(app_name);
        let cipher = Aes256Gcm::new(&key_bytes.into());

        // Generate a random 12-byte nonce.
        let mut nonce_bytes = [0u8; 12];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        // Encrypt the value.
        let plaintext = value.expose_secret().as_bytes();
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: namespace_key.as_bytes(),
                },
            )
            .map_err(|e| PrismError::CredentialStoreError {
                backend: "cache".to_owned(),
                reason: format!("AES-GCM encrypt failed: {e}"),
            })?;

        // Encode nonce + ciphertext as base64.
        let mut blob = nonce_bytes.to_vec();
        blob.extend_from_slice(&ciphertext);
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &blob);

        cache_file.entries.insert(namespace_key.to_owned(), encoded);
        persist_cache_file(&cache_path, &cache_file)?;

        Ok(())
    }

    /// Load a credential value from the cross-process file cache.
    ///
    /// Returns `Ok(Some(secret))` if the key is found and decryption succeeds,
    /// `Ok(None)` if the key is not in the cache, or an error if decryption fails.
    ///
    /// Called by `KeyringBackend::get_by_org` as a fallback when the OS keychain
    /// returns `NoEntry`.
    pub(crate) fn load_value_from_cache(
        &self,
        namespace_key: &str,
        app_name: &str,
    ) -> Result<Option<SecretString>, PrismError> {
        let cache_path = self.cache_path();
        if !cache_path.exists() {
            return Ok(None);
        }

        let cache_file = load_cache_file(&cache_path)?;
        let Some(encoded) = cache_file.entries.get(namespace_key) else {
            return Ok(None);
        };

        // Decode base64.
        let blob = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .map_err(|e| PrismError::CredentialStoreError {
                backend: "cache".to_owned(),
                reason: format!("base64 decode failed: {e}"),
            })?;

        if blob.len() < 12 {
            return Err(PrismError::CredentialStoreError {
                backend: "cache".to_owned(),
                reason: format!("cache blob too short ({} bytes, need ≥12)", blob.len()),
            });
        }

        let (nonce_bytes, ciphertext) = blob.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Derive the decryption key from app_name.
        let key_bytes = derive_cache_key(app_name);
        let cipher = Aes256Gcm::new(&key_bytes.into());

        let plaintext = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext,
                    aad: namespace_key.as_bytes(),
                },
            )
            .map_err(|e| PrismError::CredentialStoreError {
                backend: "cache".to_owned(),
                reason: format!("AES-GCM decrypt failed: {e}"),
            })?;

        let secret = SecretString::new(String::from_utf8(plaintext).map_err(|e| {
            PrismError::CredentialStoreError {
                backend: "cache".to_owned(),
                reason: format!("UTF-8 decode failed: {e}"),
            }
        })?);

        Ok(Some(secret))
    }

    /// Remove a credential value from the cross-process file cache.
    ///
    /// Called by `KeyringBackend::delete_by_org` to keep the cache consistent.
    /// No-op if the key is not in the cache.
    pub(crate) fn remove_value_from_cache(&self, namespace_key: &str) -> Result<(), PrismError> {
        let cache_path = self.cache_path();
        if !cache_path.exists() {
            return Ok(());
        }

        let mut cache_file = load_cache_file(&cache_path)?;
        if cache_file.entries.remove(namespace_key).is_some() {
            persist_cache_file(&cache_path, &cache_file)?;
        }

        Ok(())
    }
}

/// Load the cache file from disk, or return an empty `CacheFile` if not present.
fn load_cache_file(path: &Path) -> Result<CacheFile, PrismError> {
    if !path.exists() {
        return Ok(CacheFile::new());
    }
    let content = fs::read_to_string(path).map_err(|e| PrismError::Io(e.to_string()))?;
    serde_json::from_str(&content).map_err(|e| {
        debug!(error = %e, path = %path.display(), "credential cache parse error");
        PrismError::CredentialStoreError {
            backend: "cache".to_owned(),
            reason: "credential cache is corrupt or unreadable".to_owned(),
        }
    })
}

/// Persist the cache file to disk using atomic tmp+rename.
fn persist_cache_file(path: &Path, cache: &CacheFile) -> Result<(), PrismError> {
    let json = serde_json::to_string(cache).map_err(|e| PrismError::CredentialStoreError {
        backend: "cache".to_owned(),
        reason: format!("failed to serialize credential cache: {e}"),
    })?;

    // Ensure parent directory exists.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PrismError::Io(e.to_string()))?;
    }

    // Atomic write: write to tmp, then rename.
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, &json).map_err(|e| PrismError::Io(e.to_string()))?;
    fs::rename(&tmp_path, path).map_err(|e| PrismError::Io(e.to_string()))?;

    Ok(())
}
