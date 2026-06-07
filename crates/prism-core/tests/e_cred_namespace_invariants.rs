//! E-CRED namespace invariant tests (S-MAINT-ECRED-TAXONOMY-SYNC-001, ADR-035).
//!
//! These tests guard the structural invariants of the canonical E-CRED-001..010
//! namespace defined in ADR-035:
//!
//! - RG-ECRED-004: `PrismError::KeyringError` variant is absent from error.rs
//! - RG-ECRED-005: E-CRED-005 is emitted only from the file-I/O path (resolve_secret.rs)
//!
//! Both tests use source-scan via std::fs::read_to_string rather than runtime
//! variant construction, so they remain valid after the variant is removed
//! (a compile-reference to a removed variant would prevent the crate from building,
//! preventing ALL tests from running — the source-scan approach yields an assertion
//! failure on the pre-migration codebase, which is the correct Red Gate signal).

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Locate the workspace root by walking up from the current test binary's
/// CARGO_MANIFEST_DIR until we find a `Cargo.toml` with `[workspace]`.
///
/// Returns the path to `crates/prism-core/src/error.rs` relative to the
/// workspace root that contains the manifest-level `[workspace]` section.
fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is set by cargo during test compilation to the
    // directory of the crate being tested (prism-core).
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo during test compilation");
    let manifest_path = PathBuf::from(&manifest_dir);

    // prism-core's Cargo.toml is at <workspace>/crates/prism-core/Cargo.toml
    // Walk up two levels to reach the workspace root.
    manifest_path
        .parent() // crates/
        .expect("prism-core manifest should have a parent (crates/)")
        .parent() // workspace root
        .expect("crates/ directory should have a parent (workspace root)")
        .to_path_buf()
}

// ---------------------------------------------------------------------------
// RG-ECRED-004: KeyringError variant is retired and absent from error.rs
// (ADR-035 §D4; AC-004 of S-MAINT-ECRED-TAXONOMY-SYNC-001)
//
// RED GATE: Currently FAILS because error.rs still contains the KeyringError
//           variant declaration (the source scan finds "KeyringError").
// PASSES AFTER: implementer deletes the KeyringError variant from PrismError.
// ---------------------------------------------------------------------------

/// RG-ECRED-004: `PrismError::KeyringError` variant is absent from `crates/prism-core/src/error.rs`.
///
/// Uses source-scan rather than a compile-time reference to the variant.
/// A compile-time reference to a deleted variant would prevent compilation,
/// preventing ALL tests from running and making the Red Gate unobservable.
/// Source-scan yields an assertion failure (RED) on pre-migration code and
/// passes (GREEN) once the variant is deleted.
///
/// The specific pattern scanned for is the variant DECLARATION — i.e., the line
/// `KeyringError { detail: String }` inside the `enum PrismError` block.
/// Match arms in error_mapping.rs are covered by AC-005 (separate story task).
#[test]
fn test_e_cred_keyring_error_variant_retired() {
    let root = workspace_root();
    let error_rs = root.join("crates/prism-core/src/error.rs");

    let source = std::fs::read_to_string(&error_rs)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", error_rs.display()));

    // The variant declaration looks like:
    //   KeyringError { detail: String },
    // The #[error(...)] attribute above it also contains "KeyringError".
    // We check that NO line containing "KeyringError" appears in the file
    // after the migration — the entire variant block (doc comment, #[error],
    // variant body) must be gone.
    //
    // Before migration: this assertion FAILS (variant declaration found).
    // After migration: this assertion PASSES (variant fully removed).
    assert!(
        !source.contains("KeyringError"),
        "PrismError::KeyringError variant must be retired and removed from error.rs \
         (ADR-035 §D4; S-MAINT-ECRED-TAXONOMY-SYNC-001 AC-004). \
         Found 'KeyringError' in {}",
        error_rs.display()
    );
}

// ---------------------------------------------------------------------------
// RG-ECRED-005: E-CRED-005 is emitted ONLY from the file-I/O path
// (ADR-035 §D2 no-collision invariant; AC-009 of S-MAINT-ECRED-TAXONOMY-SYNC-001)
//
// RED GATE: Currently FAILS because prism-core/src/error.rs still contains
//           the #[error("E-CRED-005: credential encryption error: ...")] string,
//           meaning E-CRED-005 collides with the encryption code.
// PASSES AFTER: CredentialEncryptionError is renumbered to E-CRED-006,
//               so E-CRED-005 appears ONLY in resolve_secret.rs.
// ---------------------------------------------------------------------------

/// RG-ECRED-005: E-CRED-005 must not appear in `prism-core/src/error.rs` after renumber,
/// AND must be present in `prism-credentials/src/resolve_secret.rs` (the sole file-I/O emitter).
///
/// No-collision invariant (ADR-035 §D2): E-CRED-005 is assigned exclusively to
/// the Tier-1 file-I/O condition. After migration, the ONLY source file under
/// `crates/` that contains the literal "E-CRED-005" is `resolve_secret.rs`.
///
/// This test covers BOTH halves of the invariant:
///
/// NEGATIVE: `prism-core/src/error.rs` must NOT contain "E-CRED-005" (collision resolved).
/// POSITIVE: `prism-credentials/src/resolve_secret.rs` MUST contain "E-CRED-005" and
///           all three file-I/O sub-case strings (missing-file, is-directory, read-failed).
///           This guards against E-CRED-005 being accidentally deleted from the sole emitter.
///
/// Scope note: S-DEMO-003 worktree files are out of scope (they still contain
/// E-CRED-005 until S-DEMO-003 re-baseline). This test only scans develop-branch
/// files which are in scope for this story.
///
/// Before migration: FAILS (error.rs has `"E-CRED-005: credential encryption error:"`)
/// After migration: PASSES (CredentialEncryptionError → E-CRED-006, error.rs clean;
///                          resolve_secret.rs retains all three E-CRED-005 sub-cases)
#[test]
fn test_e_cred_005_emitted_only_by_file_io_path() {
    let root = workspace_root();
    let error_rs = root.join("crates/prism-core/src/error.rs");
    let resolve_secret_rs = root.join("crates/prism-credentials/src/resolve_secret.rs");

    // --- NEGATIVE half: E-CRED-005 absent from prism-core/src/error.rs ---
    let error_source = std::fs::read_to_string(&error_rs)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", error_rs.display()));

    // Before migration: error.rs contains `"E-CRED-005: credential encryption error: {reason}"`
    // as the #[error(...)] attribute of CredentialEncryptionError. This assertion FAILS.
    //
    // After migration: CredentialEncryptionError uses E-CRED-006; error.rs no longer
    // contains any "E-CRED-005" string. This assertion PASSES.
    assert!(
        !error_source.contains("E-CRED-005"),
        "E-CRED-005 must not appear in prism-core/src/error.rs after renumber \
         (ADR-035 §D2 no-collision invariant; S-MAINT-ECRED-TAXONOMY-SYNC-001 AC-009). \
         Found 'E-CRED-005' in {}. \
         After migration, CredentialEncryptionError must use 'E-CRED-006:'.",
        error_rs.display()
    );

    // --- POSITIVE half: E-CRED-005 present in resolve_secret.rs (sole file-I/O emitter) ---
    let resolve_source = std::fs::read_to_string(&resolve_secret_rs)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", resolve_secret_rs.display()));

    // Guard that E-CRED-005 has not been accidentally deleted from the sole emitter.
    assert!(
        resolve_source.contains("E-CRED-005"),
        "E-CRED-005 must be present in crates/prism-credentials/src/resolve_secret.rs \
         (ADR-035 §D1; S-MAINT-ECRED-TAXONOMY-SYNC-001 RG-ECRED-005 positive coverage). \
         Found no 'E-CRED-005' in {}.",
        resolve_secret_rs.display()
    );

    // Guard all three file-I/O sub-case error strings individually so a targeted
    // deletion of one sub-case would fail this test.
    assert!(
        resolve_source.contains("does not exist"),
        "resolve_secret.rs missing-file sub-case must contain 'does not exist' \
         (RG-ECRED-005 positive coverage, missing-file path). \
         File: {}",
        resolve_secret_rs.display()
    );
    assert!(
        resolve_source.contains("is a directory")
            || resolve_source.contains("directory, not a regular file"),
        "resolve_secret.rs directory sub-case must contain a directory-path error string \
         (RG-ECRED-005 positive coverage, is-directory path). \
         File: {}",
        resolve_secret_rs.display()
    );
    assert!(
        resolve_source.contains("read failed"),
        "resolve_secret.rs read-failed sub-case must contain 'read failed' \
         (RG-ECRED-005 positive coverage, read-failed path). \
         File: {}",
        resolve_secret_rs.display()
    );
}
