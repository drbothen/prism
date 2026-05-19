#![allow(non_snake_case)]
//! Red Gate test 14 — BC-2.16.011 E-SPEC-008 Retirement Annotation.
//!
//! | Test | Name                                        | AC    | Red Gate failure mode                               |
//! |------|---------------------------------------------|-------|-----------------------------------------------------|
//! | 14   | test_BC_2_16_011_e_spec_008_retired_annotation | AC-11 | E-SPEC-008 construction site found in crates/*/src/ |
//!
//! Story: S-PLUGIN-PREREQ-E | BC: BC-2.16.011 §Error Cases | AC-11
//!
//! NOTE: The spec-governance annotation invariant (E-SPEC-008 row in
//! error-taxonomy.md must carry "RETIRED in S-PLUGIN-PREREQ-E" + "ADR-027")
//! is now enforced by `.factory/hooks/validate-error-taxonomy-retirement-annotations.sh`
//! (FB-PR-1 architect adjudication; see
//! `.factory/cycles/wave-0-plugin-prereqs/architect-adjudications/FB-PR-1-error-taxonomy-test-relocation.md`).

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Test 14 — test_BC_2_16_011_e_spec_008_retired_annotation
// AC-11: no `src/` path in any workspace crate constructs E-SPEC-008 (ESpec008)
//        post-deletion of custom_adapter.rs.
//
// Pre-implementation failure mode:
//   custom_adapter.rs constructs ESpec008 in safe_override_fetch.
//   Deleting custom_adapter.rs (Task 2) satisfies this assertion.
// ---------------------------------------------------------------------------

/// Returns the workspace root by walking up from CARGO_MANIFEST_DIR.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .expect("crates/ parent must exist")
        .parent() // workspace root
        .expect("workspace root must exist")
        .to_path_buf()
}

/// BC-2.16.011 AC-11: No `src/` path in any workspace crate must construct
/// or return `E-SPEC-008` / `ESpec008` after `custom_adapter.rs` is deleted.
///
/// Reads all `.rs` files under `crates/*/src/` (workspace-wide scan,
/// F-LP-IMPL-P1-008) and asserts zero lines reference `ESpec008` or
/// `E-SPEC-008` in non-comment code. The variant DECLARATION in
/// `prism-core/src/error.rs` is exempt (POL-1 append-only numbering;
/// the variant must stay as a tombstone).
///
/// Red Gate failure mode: fails if `custom_adapter.rs` still exists and
/// constructs `ESpec008` in `safe_override_fetch`. Deleting `custom_adapter.rs`
/// (Task 2) satisfies this assertion.
///
/// Story: S-PLUGIN-PREREQ-E AC-11 | BC: BC-2.16.011 §Error Cases | Task 9
#[test]
fn test_BC_2_16_011_e_spec_008_retired_annotation() {
    let workspace = workspace_root();

    // -----------------------------------------------------------------------
    // No E-SPEC-008 construction sites in ANY crate's src/.
    //
    // F-LP-IMPL-P1-008: Expanded to workspace-wide scan (all crates/*/src/).
    // The variant DECLARATION in prism-core/src/error.rs is excluded (POL-1
    // append-only numbering; the variant must stay). Construction sites in
    // crate src/ are what we gate against.
    // -----------------------------------------------------------------------
    let crates_dir = workspace.join("crates");
    let mut construction_hits: Vec<String> = Vec::new();

    // Walk all crates/*/src/ paths.
    if let Ok(crate_entries) = std::fs::read_dir(&crates_dir) {
        for crate_entry in crate_entries.flatten() {
            let crate_path = crate_entry.path();
            if !crate_path.is_dir() {
                continue;
            }
            let src_dir = crate_path.join("src");
            if src_dir.exists() {
                collect_e_spec_008_hits(&src_dir, &mut construction_hits);
            }
        }
    }

    // Remove the variant DECLARATION from prism-core/src/error.rs (POL-1 exemption).
    // The hit format is "{relative_path_from_src}:{trimmed_line_content}".
    // The enum variant declaration line is just `ESpec008,` (trimmed).
    // It is only exempt if the hit also contains "error.rs" (the canonical declaration file).
    construction_hits.retain(|hit| {
        // Keep hits that are NOT the variant declaration in error.rs.
        // Pattern: the hit path contains "error.rs" AND the content is just the variant name.
        let is_declaration = hit.contains("error.rs") && hit.ends_with("ESpec008,");
        !is_declaration
    });

    assert!(
        construction_hits.is_empty(),
        "BC-2.16.011 AC-11 (F-LP-IMPL-P1-008 workspace-wide): found {} E-SPEC-008 / \
         ESpec008 construction sites in workspace crates/*/src/. \
         The variant DECLARATION in prism-core/src/error.rs is exempt (POL-1). \
         All other construction sites must be removed.\n\
         Hits:\n{}",
        construction_hits.len(),
        construction_hits.join("\n")
    );
}

/// Recursively scan `dir` for `.rs` files and collect lines containing
/// `ESpec008` or `E-SPEC-008` in non-comment executable code.
fn collect_e_spec_008_hits(dir: &std::path::Path, hits: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_e_spec_008_hits(&path, hits);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("//")
                    || trimmed.starts_with("///")
                    || trimmed.starts_with("//!")
                {
                    continue;
                }
                if line.contains("ESpec008") || line.contains("E-SPEC-008") {
                    hits.push(format!(
                        "{}:{}",
                        path.strip_prefix(dir).unwrap_or(&path).display(),
                        line.trim()
                    ));
                }
            }
        }
    }
}
