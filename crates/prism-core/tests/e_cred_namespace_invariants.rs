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

/// Walk `<workspace>/crates/*/src/` recursively and collect all `*.rs` files
/// whose content contains the literal `needle`.
///
/// Uses `git grep -r -l` when git is available in PATH (fast, hermetic) and
/// post-filters results to the `src/` subtree only — `tests/`, `proofs/`, and
/// `examples/` subdirectories are excluded so that Red Gate test files containing
/// the literal in comments do not appear as false positives.
///
/// Falls back to an `std::fs` recursive walk if git is unavailable.
fn src_files_containing(workspace_root: &std::path::Path, needle: &str) -> Vec<std::path::PathBuf> {
    // Attempt git grep first — it is fast and respects .gitignore.
    // We scan all of `crates/` and then post-filter to `/src/` paths only
    // (git grep does not expand shell globs in pathspec arguments, so
    // `crates/*/src/` would be treated as a literal path, not a pattern).
    let git_result = std::process::Command::new("git")
        .current_dir(workspace_root)
        .args(["grep", "-r", "-l", needle, "--", "crates/"])
        .output();

    if let Ok(output) = git_result {
        // exit 0 = matches found; exit 1 = no matches; both are clean runs.
        // Any other non-zero (e.g., 128 = not a git repo) falls through to fs-walk.
        if output.status.success() || output.status.code() == Some(1) {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout
                .lines()
                .filter(|l| !l.is_empty())
                // Post-filter: keep only paths whose components include a `src` segment
                // immediately under a crate root (crates/<name>/src/...).
                .filter(|rel_path| {
                    // rel_path looks like "crates/prism-foo/src/bar.rs"
                    // We accept it iff it contains "/src/" anywhere after crates/<name>.
                    rel_path.contains("/src/")
                })
                .map(|rel| workspace_root.join(rel))
                .collect();
        }
    }

    // Fallback: std::fs recursive walk over crates/*/src/**/*.rs
    let mut results = Vec::new();
    let crates_dir = workspace_root.join("crates");

    let crates_entries = match std::fs::read_dir(&crates_dir) {
        Ok(e) => e,
        Err(err) => panic!(
            "Failed to read crates/ directory at {}: {err}",
            crates_dir.display()
        ),
    };

    for crate_entry in crates_entries.flatten() {
        let src_dir = crate_entry.path().join("src");
        if src_dir.is_dir() {
            collect_rs_files_containing(&src_dir, needle, &mut results);
        }
    }
    results
}

/// Recursively collect `*.rs` files under `dir` whose content contains `needle`.
fn collect_rs_files_containing(
    dir: &std::path::Path,
    needle: &str,
    out: &mut Vec<std::path::PathBuf>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_containing(&path, needle, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if content.contains(needle) {
                    out.push(path);
                }
            }
        }
    }
}

/// RG-ECRED-005: Workspace-wide no-collision guard — E-CRED-005 must appear in
/// EXACTLY ONE `crates/*/src/**/*.rs` file: `crates/prism-credentials/src/resolve_secret.rs`.
///
/// No-collision invariant (ADR-035 §D2): E-CRED-005 is assigned exclusively to
/// the Tier-1 file-I/O condition. No other source file under `crates/*/src/` may
/// contain the literal "E-CRED-005" — not error.rs, not file.rs, not a new crate.
///
/// Scanning strategy:
/// - Uses `git grep -r -l "E-CRED-005" -- crates/*/src/` from the workspace root
///   (fast, hermetic) with std::fs recursive walk as fallback if git is unavailable.
/// - Scoped to `src/` only — `tests/`, `proofs/`, and `examples/` are excluded so
///   that the Red Gate test file itself (which contains this literal in comments)
///   does not produce a false positive.
/// - The workspace root is located via `CARGO_MANIFEST_DIR` walking up to the
///   directory that contains `crates/` — manifest-relative, not CWD-relative.
///
/// Assertions:
/// COLLISION: the set of matching src files equals exactly
///            `{crates/prism-credentials/src/resolve_secret.rs}`.
///            Any unexpected file produces a clear panic listing the offender.
/// POSITIVE COVERAGE: resolve_secret.rs contains all three file-I/O sub-case
///            strings (does-not-exist, is-directory, read-failed) so that a
///            targeted deletion of one sub-case fails this guard.
///
/// Before migration: FAILS (error.rs has `"E-CRED-005: credential encryption error:"`)
/// After migration:  PASSES (CredentialEncryptionError → E-CRED-006, error.rs clean;
///                           resolve_secret.rs retains all three E-CRED-005 sub-cases)
#[test]
fn test_e_cred_005_emitted_only_by_file_io_path() {
    let root = workspace_root();
    let resolve_secret_rs = root.join("crates/prism-credentials/src/resolve_secret.rs");

    // --- WORKSPACE SCAN: find every crates/*/src/**/*.rs containing "E-CRED-005" ---
    let matching_files = src_files_containing(&root, "E-CRED-005");

    // Normalise to paths relative to the workspace root for deterministic comparison.
    let relative_matches: Vec<String> = {
        let mut v: Vec<String> = matching_files
            .iter()
            .map(|p| {
                p.strip_prefix(&root)
                    .map(|rel| rel.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| p.to_string_lossy().replace('\\', "/"))
            })
            .collect();
        v.sort();
        v
    };

    let expected = "crates/prism-credentials/src/resolve_secret.rs";

    // POSITIVE: resolve_secret.rs must be present (guard against accidental deletion).
    assert!(
        relative_matches.iter().any(|p| p == expected),
        "E-CRED-005 must be present in {expected} \
         (ADR-035 §D1; RG-ECRED-005 positive coverage). \
         No src file under crates/ contains 'E-CRED-005'. \
         Workspace root: {}",
        root.display()
    );

    // NO-COLLISION: only resolve_secret.rs may contain E-CRED-005.
    let unexpected: Vec<&str> = relative_matches
        .iter()
        .map(String::as_str)
        .filter(|p| *p != expected)
        .collect();

    assert!(
        unexpected.is_empty(),
        "E-CRED-005 no-collision invariant violated (ADR-035 §D2; AC-009). \
         The following src files contain 'E-CRED-005' but must not:\n  {}\n\
         Only '{}' is the authorised file-I/O emitter. \
         Rename the colliding condition to the next available E-CRED-NNN code.",
        unexpected.join("\n  "),
        expected
    );

    // POSITIVE SUB-CASES: all three file-I/O conditions must be present in resolve_secret.rs
    // so a targeted deletion of one sub-case fails here rather than silently.
    let resolve_source = std::fs::read_to_string(&resolve_secret_rs)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", resolve_secret_rs.display()));

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
