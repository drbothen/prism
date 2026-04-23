//! VP-034 and VP-035 property-based tests using proptest.
//!
//! VP-034: AES-256-GCM encrypt → decrypt round-trip returns original plaintext.
//! VP-035: HKDF-SHA256 key derivation is deterministic for same inputs.
//!
//! Both tests call unimplemented!() stubs and will FAIL (panic) at Red Gate.
//!
//! Story: S-1.06 | BCs: BC-2.03.003 | VPs: VP-034, VP-035

use proptest::prelude::*;
use secrecy::SecretString;
use std::path::PathBuf;

use crate::file::{derive_key, EncryptedFileBackend, SALT_LEN};

// ---------------------------------------------------------------------------
// VP-034: Encryption round-trip
// ---------------------------------------------------------------------------

proptest! {
    /// VP-034: For every plaintext p and correctly derived key k,
    /// `decrypt(encrypt(p, k), k)` == `p` byte-for-byte.
    ///
    /// Tests the `encrypt_bytes` / `decrypt_bytes` internal methods of
    /// `EncryptedFileBackend`.
    ///
    /// Red Gate: calls unimplemented!() — will panic.
    #[test]
    fn test_BC_2_03_003_prop_encrypt_decrypt_round_trip(
        plaintext in proptest::collection::vec(any::<u8>(), 0..=4096),
        passphrase in "[a-zA-Z0-9!@#$%^&*]{8,64}",
    ) {
        // Construct backend with a test passphrase.
        // Both EncryptedFileBackend::new and encrypt_bytes are unimplemented!().
        let backend = EncryptedFileBackend::new(
            PathBuf::from("/tmp/prism-test"),
            SecretString::new(passphrase.into()),
        );
        let ciphertext = backend.encrypt_bytes(&plaintext)
            .expect("encrypt_bytes should succeed");
        let recovered = backend.decrypt_bytes(&ciphertext)
            .expect("decrypt_bytes should succeed");
        prop_assert_eq!(
            plaintext,
            recovered,
            "VP-034: decrypt(encrypt(p)) must equal p"
        );
    }
}

// ---------------------------------------------------------------------------
// VP-035: Key derivation determinism
// ---------------------------------------------------------------------------

proptest! {
    /// VP-035: For any (passphrase, salt), `derive_key` returns the same
    /// 256-bit key on every invocation.
    ///
    /// Red Gate: calls unimplemented!() — will panic.
    #[test]
    fn test_BC_2_03_003_prop_key_derivation_deterministic(
        passphrase in proptest::collection::vec(any::<u8>(), 1..=128),
        salt in proptest::collection::vec(any::<u8>(), SALT_LEN..=SALT_LEN),
    ) {
        let key1 = derive_key(&passphrase, &salt)
            .expect("derive_key call 1 should succeed");
        let key2 = derive_key(&passphrase, &salt)
            .expect("derive_key call 2 should succeed");
        prop_assert_eq!(
            key1,
            key2,
            "VP-035: derive_key must be deterministic for same inputs"
        );
    }
}

// ---------------------------------------------------------------------------
// Additional crypto invariant: different encryptions of same plaintext differ
// (BC-2.03.003 TV-BC-2.03.003-006)
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_03_003_two_encryptions_differ_in_nonce_and_salt() {
    // Red Gate: EncryptedFileBackend::new is unimplemented!() — panics.
    let backend = EncryptedFileBackend::new(
        PathBuf::from("/tmp/prism-test"),
        SecretString::new("test-passphrase-001".to_string()),
    );
    let plaintext = b"same-value";
    let ct1 = backend.encrypt_bytes(plaintext).expect("encrypt 1");
    let ct2 = backend.encrypt_bytes(plaintext).expect("encrypt 2");
    // Different salts and nonces → different ciphertexts
    assert_ne!(
        ct1, ct2,
        "Two encryptions of the same plaintext must differ"
    );
}

// ---------------------------------------------------------------------------
// File format integrity: corrupt/truncated file returns Err (BC-2.03.003 EC-03-008/009)
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_03_003_decrypt_zero_byte_file_returns_err() {
    // Red Gate: EncryptedFileBackend::new is unimplemented!() — panics.
    let backend = EncryptedFileBackend::new(
        PathBuf::from("/tmp/prism-test"),
        SecretString::new("test-passphrase-002".to_string()),
    );
    let result = backend.decrypt_bytes(&[]);
    assert!(
        result.is_err(),
        "EC-03-008: zero-byte file must return Err(CredentialEncryptionError)"
    );
}

#[test]
fn test_BC_2_03_003_decrypt_truncated_file_returns_err() {
    // File shorter than salt+nonce minimum (44 bytes in BC, 60 in impl)
    // Red Gate: unimplemented!().
    let backend = EncryptedFileBackend::new(
        PathBuf::from("/tmp/prism-test"),
        SecretString::new("test-passphrase-003".to_string()),
    );
    // 20 bytes: shorter than SALT_LEN(32) + NONCE_LEN(12) + TAG(16) = 60
    let truncated = vec![0u8; 20];
    let result = backend.decrypt_bytes(&truncated);
    assert!(
        result.is_err(),
        "EC-03-009: truncated file (< 60 bytes) must return Err"
    );
}

#[test]
fn test_BC_2_03_003_decrypt_wrong_key_returns_err() {
    // TV-BC-2.03.003-002: decrypt with wrong passphrase must return Err.
    // Red Gate: unimplemented!().
    let encryptor = EncryptedFileBackend::new(
        PathBuf::from("/tmp/prism-test"),
        SecretString::new("correct-passphrase".to_string()),
    );
    let decryptor = EncryptedFileBackend::new(
        PathBuf::from("/tmp/prism-test"),
        SecretString::new("wrong-passphrase".to_string()),
    );
    let ct = encryptor.encrypt_bytes(b"secret-value").expect("encrypt");
    let result = decryptor.decrypt_bytes(&ct);
    assert!(
        result.is_err(),
        "TV-BC-2.03.003-002: wrong key must cause decryption failure"
    );
}
