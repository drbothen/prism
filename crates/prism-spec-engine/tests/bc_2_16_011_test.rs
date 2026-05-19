#![allow(non_snake_case)]
//! Red Gate tests for BC-2.16.011 — CustomAdapter Rust Trait Retirement.
//!
//! Tests 6 and 7 of the S-PLUGIN-PREREQ-E Red Gate set (prism-spec-engine crate).
//!
//! Both tests are grep-gate tests: they read source files and assert that
//! certain patterns are absent from production code. They fail RED because
//! `CustomAdapter` still exists in `src/custom_adapter.rs` pre-deletion.
//!
//! | Test | Name                                                        | AC   | Red Gate failure mode                  |
//! |------|-------------------------------------------------------------|------|----------------------------------------|
//! | 6    | test_BC_2_16_011_001_custom_adapter_absent_post_deletion    | AC-4 | `custom_adapter.rs` exists → grep hits |
//! | 7    | test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code  | AC-5 | `ESpec008` in `custom_adapter.rs`      |
//!
//! Story: S-PLUGIN-PREREQ-E | BC: BC-2.16.011 | ADR-023 §C5 Rule 5

use std::path::PathBuf;

/// Returns the crate root for `prism-spec-engine` by walking up from
/// `CARGO_MANIFEST_DIR`.
fn spec_engine_crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Returns the `src/` directory of `prism-spec-engine`.
fn src_dir() -> PathBuf {
    spec_engine_crate_root().join("src")
}

// ---------------------------------------------------------------------------
// Test 6 — test_BC_2_16_011_001_custom_adapter_absent_post_deletion
// AC-4: `CustomAdapter` / `CustomAdapterRegistry` / `CustomAuth` must not
//       appear in any `src/` path after deletion.
//
// Pre-implementation failure mode: `src/custom_adapter.rs` exists and contains
// `CustomAdapter`, `CustomAdapterRegistry`. Grep returns hits — assertion fails.
// ---------------------------------------------------------------------------

/// BC-2.16.011 AC-4 / INV-ADAPTER-RETIRE-001: After PREREQ-E, `CustomAdapter`,
/// `CustomAdapterRegistry`, and `CustomAuth` must not appear in any
/// production `src/` file in `prism-spec-engine`.
///
/// Grep-gate test: reads all `.rs` files under `crates/prism-spec-engine/src/`
/// and asserts zero occurrences of the retired identifiers.
///
/// Red Gate failure mode: `src/custom_adapter.rs` still exists pre-deletion —
/// grep finds `CustomAdapter`, `CustomAdapterRegistry`, `CustomAuth`, and the
/// assertion fails.
///
/// Story: S-PLUGIN-PREREQ-E AC-4 | BC: BC-2.16.011 | ADR-023 §C5 Rule 5
#[test]
fn test_BC_2_16_011_001_custom_adapter_absent_post_deletion() {
    let src = src_dir();
    let forbidden = ["CustomAdapter", "CustomAdapterRegistry", "CustomAuth"];

    let mut hits: Vec<String> = Vec::new();

    for entry in walkdir_rs_files(&src) {
        let content = std::fs::read_to_string(&entry)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", entry.display()));

        for line in content.lines() {
            // Skip comment lines and doc-string lines — only scan executable code.
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!")
            {
                continue;
            }
            for &name in &forbidden {
                if line.contains(name) {
                    hits.push(format!(
                        "{}:{}",
                        entry.strip_prefix(&src).unwrap_or(&entry).display(),
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        hits.is_empty(),
        "BC-2.16.011 AC-4 / INV-ADAPTER-RETIRE-001: found {} retired CustomAdapter \
         identifiers in prism-spec-engine src/. Task 2/3/4/5 must delete \
         custom_adapter.rs and clean all call sites before this test passes.\n\
         Hits:\n{}",
        hits.len(),
        hits.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Test 7 — test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code
// AC-5: `E-SPEC-008` / `ESpec008` must not be CONSTRUCTED in any non-test
//       src/ path after CustomAdapter deletion.
//
// Pre-implementation failure mode: `src/custom_adapter.rs::safe_override_fetch`
// constructs `ESpec008` via the catch_unwind handler — grep hits.
// ---------------------------------------------------------------------------

/// BC-2.16.011 AC-5 / BC-2.16.011 §Error Cases (E-SPEC-008 retired):
/// After CustomAdapter deletion, no production `src/` code path constructs
/// or returns the `E-SPEC-008` error code / `ESpec008` variant.
///
/// The E-SPEC-008 error taxonomy entry is PRESERVED (retired annotation only)
/// per append_only_numbering (DF-030). Only the construction sites are removed.
///
/// Grep-gate test: scans `crates/prism-spec-engine/src/` for lines that
/// reference `ESpec008` (the enum variant) or `E-SPEC-008` in non-comment
/// executable code, and asserts zero hits.
///
/// Red Gate failure mode: `src/custom_adapter.rs` contains `ESpec008` in its
/// `safe_override_fetch` catch_unwind handler — grep finds it and fails.
///
/// Story: S-PLUGIN-PREREQ-E AC-5 | BC: BC-2.16.011 §Error Cases
#[test]
fn test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code() {
    let src = src_dir();
    let patterns = ["ESpec008", "E-SPEC-008"];

    let mut hits: Vec<String> = Vec::new();

    for entry in walkdir_rs_files(&src) {
        let content = std::fs::read_to_string(&entry)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", entry.display()));

        for line in content.lines() {
            let trimmed = line.trim();
            // Skip comment and doc lines.
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!")
            {
                continue;
            }
            for &pat in &patterns {
                if line.contains(pat) {
                    hits.push(format!(
                        "{}:{}",
                        entry.strip_prefix(&src).unwrap_or(&entry).display(),
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        hits.is_empty(),
        "BC-2.16.011 AC-5: found {} E-SPEC-008 / ESpec008 construction sites in \
         prism-spec-engine src/. Task 2 must delete custom_adapter.rs (which constructs \
         ESpec008 in safe_override_fetch) before this test passes.\n\
         Hits:\n{}",
        hits.len(),
        hits.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Helper: walk a directory and return all *.rs file paths.
// ---------------------------------------------------------------------------

fn walkdir_rs_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_rs_files(dir, &mut files);
    files
}

fn collect_rs_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read_dir {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("read_dir entry must be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            files.push(path);
        }
    }
}
