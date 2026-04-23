//! EncryptedFileBackend — AES-256-GCM file-based credential store — BC-2.03.003
//!
//! Storage layout: `{base_dir}/{tenant}/{sensor}/{name}.enc`
//!
//! File format (binary, raw concatenation):
//!   `[SALT_LEN-byte salt][NONCE_LEN-byte nonce][ciphertext + 16-byte GCM tag]`
//!
//! Key derivation: Argon2id from master passphrase + per-credential salt.
//! Production params: m=65536 (64MB), t=3, p=1 (BC-2.03.003 v1.4).
//! Test params (proptest): m=256, t=1, p=1 (VP-034/VP-035 speed requirement).
//!
//! Atomic writes: write to `{name}.enc.tmp`, rename to `{name}.enc`.
//!
//! BC-2.03.003 file format:
//!   salt   = 16 bytes  (random, per-credential, Argon2id input)
//!   nonce  = 12 bytes  (random, per-encryption-operation, AES-GCM IV)
//!   rest   = ciphertext + 16-byte GCM authentication tag
//!   Minimum file size: 16 + 12 + 0 + 16 = 44 bytes
//!
//! Story: S-1.06 | BC: BC-2.03.003, BC-2.03.004

use std::path::PathBuf;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use async_trait::async_trait;
use prism_core::{PrismError, TenantId};
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use std::fs;

use crate::{namespace::{validate_sensor, CredentialName}, trait_::CredentialStore};

/// Salt length in bytes (BC-2.03.003 v1.4 — Argon2id 16-byte salt).
pub const SALT_LEN: usize = 16;

/// Nonce length in bytes (AES-256-GCM, 96-bit).
pub const NONCE_LEN: usize = 12;

/// AES-256 key length in bytes.
pub const KEY_LEN: usize = 32;

/// Minimum valid file size: salt(16) + nonce(12) + GCM tag(16) = 44 bytes.
pub const MIN_FILE_BYTES: usize = SALT_LEN + NONCE_LEN + 16;

/// Argon2id production parameters (BC-2.03.003 v1.4).
/// Only used in non-test builds; suppress dead_code warning in test mode.
#[allow(dead_code)]
const ARGON2_M_COST: u32 = 65536; // 64 MB
#[allow(dead_code)]
const ARGON2_T_COST: u32 = 3; // iterations
#[allow(dead_code)]
const ARGON2_P_COST: u32 = 1; // parallelism

/// Argon2id test parameters for proptest speed (VP-034, VP-035).
/// Activated when the `PRISM_ARGON2_TEST_PARAMS` env var is set, or in
/// proptest contexts that explicitly pass scaled-down params to `derive_key_with_params`.
const ARGON2_TEST_M_COST: u32 = 256;
const ARGON2_TEST_T_COST: u32 = 1;
const ARGON2_TEST_P_COST: u32 = 1;

/// Encrypted-file credential backend.
///
/// Stores credentials as AES-256-GCM encrypted files. One file per credential.
/// The passphrase is held in memory as `SecretString` (zeroized on drop).
pub struct EncryptedFileBackend {
    base_dir: PathBuf,
    /// Master key material — never written to disk.
    /// `SecretString` ensures zeroization on drop.
    passphrase: SecretString,
    /// Whether to use test-speed Argon2 params (for proptest VP-034/VP-035).
    use_test_params: bool,
}

impl EncryptedFileBackend {
    /// Create a new `EncryptedFileBackend`.
    ///
    /// `base_dir` is the root directory for credential files.
    /// `passphrase` is the master key material (from env var, never from prism.toml).
    ///
    /// In `cfg(test)` mode (including proptests), automatically uses scaled-down
    /// Argon2id parameters (m=256, t=1, p=1) for test speed (VP-034/VP-035).
    /// Production builds always use full parameters (m=65536, t=3, p=1).
    pub fn new(base_dir: PathBuf, passphrase: SecretString) -> Self {
        // SEC-004: test-speed Argon2 params are ONLY active in cfg(test) builds.
        // The PRISM_ARGON2_TEST_PARAMS env var escape hatch is removed to prevent
        // production degradation via env injection.
        #[cfg(test)]
        let use_test_params = true;
        #[cfg(not(test))]
        let use_test_params = false;

        EncryptedFileBackend {
            base_dir,
            passphrase,
            use_test_params,
        }
    }

    /// Create a new `EncryptedFileBackend` with scaled-down Argon2id parameters.
    ///
    /// For use in proptest / unit tests only. Uses m=256, t=1, p=1.
    #[cfg(test)]
    pub fn new_for_test(base_dir: PathBuf, passphrase: SecretString) -> Self {
        EncryptedFileBackend {
            base_dir,
            passphrase,
            use_test_params: true,
        }
    }

    /// Encrypt plaintext bytes using AES-256-GCM with a freshly derived key.
    ///
    /// Returns `[salt(SALT_LEN) || nonce(NONCE_LEN) || ciphertext+tag]` as a `Vec<u8>`.
    ///
    /// Called by VP-034 proptest to test the round-trip property.
    pub fn encrypt_bytes(&self, plaintext: &[u8]) -> Result<Vec<u8>, PrismError> {
        // Generate fresh random salt and nonce.
        let mut salt = [0u8; SALT_LEN];
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        // Derive key from passphrase + salt.
        let passphrase_bytes = self.passphrase.expose_secret().as_bytes();
        let key = if self.use_test_params {
            derive_key_with_params(
                passphrase_bytes,
                &salt,
                ARGON2_TEST_M_COST,
                ARGON2_TEST_T_COST,
                ARGON2_TEST_P_COST,
            )?
        } else {
            derive_key(passphrase_bytes, &salt)?
        };

        // Encrypt with AES-256-GCM.
        let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(&key));
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|e| {
            PrismError::CredentialEncryptionError {
                reason: format!("AES-GCM encryption failed: {e}"),
            }
        })?;

        // Assemble output: salt || nonce || ciphertext+tag
        let mut output = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    /// Decrypt bytes produced by `encrypt_bytes`.
    ///
    /// Input format: `[salt(SALT_LEN) || nonce(NONCE_LEN) || ciphertext+tag]`.
    /// Returns the original plaintext on success.
    pub fn decrypt_bytes(&self, ciphertext_blob: &[u8]) -> Result<Vec<u8>, PrismError> {
        if ciphertext_blob.len() < MIN_FILE_BYTES {
            return Err(PrismError::CredentialEncryptionError {
                reason: format!(
                    "credential file too short: {} bytes (minimum {})",
                    ciphertext_blob.len(),
                    MIN_FILE_BYTES
                ),
            });
        }

        let salt = &ciphertext_blob[..SALT_LEN];
        let nonce_bytes = &ciphertext_blob[SALT_LEN..SALT_LEN + NONCE_LEN];
        let ciphertext = &ciphertext_blob[SALT_LEN + NONCE_LEN..];

        // Derive key from passphrase + salt.
        let passphrase_bytes = self.passphrase.expose_secret().as_bytes();
        let key = if self.use_test_params {
            derive_key_with_params(
                passphrase_bytes,
                salt,
                ARGON2_TEST_M_COST,
                ARGON2_TEST_T_COST,
                ARGON2_TEST_P_COST,
            )?
        } else {
            derive_key(passphrase_bytes, salt)?
        };

        // Decrypt with AES-256-GCM.
        let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(&key));
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
            PrismError::CredentialEncryptionError {
                reason: format!("AES-GCM decryption failed (wrong key or corrupt data): {e}"),
            }
        })?;

        Ok(plaintext)
    }

    /// Compute the file path for a credential.
    fn credential_path(&self, tenant: &TenantId, sensor: &str, name: &CredentialName) -> PathBuf {
        // Layout: base_dir/{tenant}/{sensor}/{name}.enc
        self.base_dir
            .join(tenant.as_str())
            .join(sensor)
            .join(format!("{}.enc", name.as_str()))
    }
}

/// Derive a 256-bit AES key from `passphrase_bytes` and `salt` using Argon2id.
///
/// In `cfg(test)` mode: uses scaled-down parameters (m=256, t=1, p=1) for
/// VP-034/VP-035 proptest speed requirement (must complete in <1 min).
/// In production: uses m=65536, t=3, p=1 per BC-2.03.003 v1.4.
///
/// Returns `Err(CredentialStoreError)` if passphrase is empty (EC-005).
///
/// Called by VP-035 proptest to test determinism.
pub fn derive_key(passphrase_bytes: &[u8], salt: &[u8]) -> Result<[u8; KEY_LEN], PrismError> {
    #[cfg(test)]
    return derive_key_with_params(
        passphrase_bytes,
        salt,
        ARGON2_TEST_M_COST,
        ARGON2_TEST_T_COST,
        ARGON2_TEST_P_COST,
    );
    #[cfg(not(test))]
    derive_key_with_params(
        passphrase_bytes,
        salt,
        ARGON2_M_COST,
        ARGON2_T_COST,
        ARGON2_P_COST,
    )
}

/// Derive a 256-bit AES key with explicit Argon2id parameters.
///
/// Used internally for both production (m=65536) and test-speed (m=256) derivation.
pub fn derive_key_with_params(
    passphrase_bytes: &[u8],
    salt: &[u8],
    m_cost: u32,
    t_cost: u32,
    p_cost: u32,
) -> Result<[u8; KEY_LEN], PrismError> {
    if passphrase_bytes.is_empty() {
        return Err(PrismError::CredentialStoreError {
            backend: "encrypted-file".to_owned(),
            reason: "passphrase must not be empty (EC-005)".to_owned(),
        });
    }

    let params = Params::new(m_cost, t_cost, p_cost, Some(KEY_LEN)).map_err(|e| {
        PrismError::CredentialEncryptionError {
            reason: format!("Argon2id parameter error: {e}"),
        }
    })?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output_key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(passphrase_bytes, salt, &mut output_key)
        .map_err(|e| PrismError::CredentialEncryptionError {
            reason: format!("Argon2id key derivation failed: {e}"),
        })?;

    Ok(output_key)
}

#[async_trait]
impl CredentialStore for EncryptedFileBackend {
    /// Read, decode, and decrypt the credential file.
    ///
    /// - Returns `Ok(None)` if file does not exist.
    /// - Returns `Err(CredentialEncryptionError)` for corrupt/truncated files.
    async fn get(
        &self,
        tenant: &TenantId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<Option<SecretString>, PrismError> {
        // SEC-001: validate sensor before any path construction (BC-2.03.004).
        validate_sensor(sensor)?;
        let path = self.credential_path(tenant, sensor, name);

        if !path.exists() {
            return Ok(None);
        }

        let raw = fs::read(&path).map_err(|e| PrismError::Io(e.to_string()))?;
        let plaintext_bytes = self.decrypt_bytes(&raw)?;
        let plaintext = String::from_utf8(plaintext_bytes).map_err(|e| {
            PrismError::CredentialEncryptionError {
                reason: format!("credential value is not valid UTF-8: {e}"),
            }
        })?;

        Ok(Some(SecretString::new(plaintext)))
    }

    /// Encrypt and atomically write the credential file.
    ///
    /// Write to `{name}.enc.tmp`, then rename to `{name}.enc`.
    async fn set(
        &self,
        tenant: &TenantId,
        sensor: &str,
        name: &CredentialName,
        value: SecretString,
    ) -> Result<(), PrismError> {
        // SEC-001: validate sensor before any path construction (BC-2.03.004).
        validate_sensor(sensor)?;
        let path = self.credential_path(tenant, sensor, name);

        // Ensure parent directory exists.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PrismError::Io(e.to_string()))?;
        }

        let plaintext_bytes = value.expose_secret().as_bytes();
        let encrypted = self.encrypt_bytes(plaintext_bytes)?;

        // Atomic write: tmp + rename.
        let tmp_path = path.with_extension("enc.tmp");
        fs::write(&tmp_path, &encrypted).map_err(|e| PrismError::Io(e.to_string()))?;
        fs::rename(&tmp_path, &path).map_err(|e| PrismError::Io(e.to_string()))?;

        Ok(())
    }

    /// Remove the credential file. Returns `true` if deleted, `false` if
    /// file did not exist.
    async fn delete(
        &self,
        tenant: &TenantId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        // SEC-001: validate sensor before any path construction (BC-2.03.004).
        validate_sensor(sensor)?;
        let path = self.credential_path(tenant, sensor, name);

        if !path.exists() {
            return Ok(false);
        }

        fs::remove_file(&path).map_err(|e| PrismError::Io(e.to_string()))?;
        Ok(true)
    }

    /// Scan `{base_dir}/{tenant}/` for `.enc` files and return (sensor, name) pairs.
    ///
    /// Layout: `base_dir/{tenant}/{sensor}/{name}.enc`
    async fn list(&self, tenant: &TenantId) -> Result<Vec<(String, CredentialName)>, PrismError> {
        let tenant_dir = self.base_dir.join(tenant.as_str());

        if !tenant_dir.exists() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        // Iterate sensor subdirectories.
        for sensor_entry in fs::read_dir(&tenant_dir).map_err(|e| PrismError::Io(e.to_string()))? {
            let sensor_entry = sensor_entry.map_err(|e| PrismError::Io(e.to_string()))?;
            let sensor_path = sensor_entry.path();

            if !sensor_path.is_dir() {
                continue;
            }

            let sensor_name = sensor_entry.file_name().into_string().unwrap_or_default();

            // SEC-001: skip sensor directories whose names fail validation.
            // Protects against adversarially-named directories in the credential tree.
            if validate_sensor(&sensor_name).is_err() {
                tracing::warn!(
                    sensor = %sensor_name,
                    "list(): skipping sensor directory with invalid name (SEC-001)"
                );
                continue;
            }

            // Iterate credential files in this sensor dir.
            for cred_entry in
                fs::read_dir(&sensor_path).map_err(|e| PrismError::Io(e.to_string()))?
            {
                let cred_entry = cred_entry.map_err(|e| PrismError::Io(e.to_string()))?;
                let cred_path = cred_entry.path();

                if cred_path.extension().and_then(|e| e.to_str()) != Some("enc") {
                    continue;
                }

                // Strip `.enc` extension to get the credential name.
                // SEC-003: validate the filesystem-sourced stem through CredentialName::new()
                // rather than new_unchecked — prevents injection from adversarially-named files.
                if let Some(stem) = cred_path.file_stem().and_then(|s| s.to_str()) {
                    match CredentialName::new(stem) {
                        Ok(cred_name) => results.push((sensor_name.clone(), cred_name)),
                        Err(_) => {
                            tracing::warn!(
                                stem = %stem,
                                "list(): skipping credential file with invalid name (SEC-003)"
                            );
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Check whether a credential file exists.
    async fn exists(
        &self,
        tenant: &TenantId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        // SEC-001: validate sensor before any path construction (BC-2.03.004).
        validate_sensor(sensor)?;
        let path = self.credential_path(tenant, sensor, name);
        Ok(path.exists())
    }
}
