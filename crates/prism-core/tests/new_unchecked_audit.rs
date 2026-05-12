#![allow(non_snake_case)]
//! AC-6 Red Gate test: cross-newtype `*::new_unchecked` validation-bypass audit.
//!
//! BC-2.01.013 postcondition: the spec-driven adapter surface enforces
//! validation-on-construct for sensor identifiers and org identifiers.
//! `new_unchecked` constructors bypass that enforcement and MUST be explicitly
//! inventoried, justified, and either feature-gated or doc-commented with a
//! clear precondition statement.
//!
//! AC-6 resolves TD-S-PLUGIN-PREREQ-A-006 P3.
//!
//! RED GATE MECHANISM: `OrgSlug::new_unchecked` in `crates/prism-core/src/tenant.rs`
//! is currently `pub` with a doc-comment saying "MUST NOT be called from production code"
//! but WITHOUT a `#[cfg(any(test, feature = "test-helpers"))]` gate. The audit test below
//! asserts either:
//!   (a) the function is NOT present in any `*.rs` file outside a `#[cfg(...)]` block, OR
//!   (b) it IS present but gated by the test-helpers feature or cfg(test).
//!
//! The test reads actual source files and fails if any `fn new_unchecked` is found that
//! is NOT in the known allowlist AND NOT feature-gated.
//!
//! KNOWN ALLOWLIST (baseline at Red Gate phase):
//!   - `OrgSlug::new_unchecked` in `crates/prism-core/src/tenant.rs`
//!     Status: UNGATED (pub, no cfg). RED GATE: this item is NOT in the allowlist below.
//!     The test fails because the ungated `new_unchecked` is found but not allowed.
//!
//! After AC-6 implementation, the implementer must either:
//!   1. Gate `OrgSlug::new_unchecked` with `#[cfg(any(test, feature = "test-helpers"))]`
//!      (if no production callers exist), OR
//!   2. Add it to the allowlist with a doc-comment explaining the production use case,
//!      AND update this allowlist to acknowledge the item.
//!
//! The test is a workspace-grep regression: any future `fn new_unchecked` added to
//! `crates/prism-core/src/` MUST appear in the allowlist below or be feature-gated.

use std::path::Path;

// ---------------------------------------------------------------------------
// Known allowlist: `new_unchecked` sites that have been audited and accepted.
//
// Format: (file_suffix, context_requirement)
//   - file_suffix: relative suffix of the source file within crates/prism-core/src/
//   - context_requirement: what must surround the `fn new_unchecked` declaration
//     for it to be considered gated. "gated" = has cfg(test) or cfg(feature="test-helpers")
//     within 10 lines before the declaration.
//
// INITIALLY EMPTY: the audit finds `OrgSlug::new_unchecked` in tenant.rs and
// determines it is NOT in this allowlist AND is NOT feature-gated.
// The test FAILS = RED GATE.
//
// After AC-6 implementation (implementer's obligation):
//   - If gated: add nothing to this allowlist; the gate is sufficient.
//   - If ungated with documented production justification: add to this list with
//     the file path and a comment explaining the production use case.
// ---------------------------------------------------------------------------
const GATED_OR_ALLOWLISTED_UNCHECKED: &[&str] = &[
    // Empty allowlist at Red Gate phase.
    // The implementer adds entries here ONLY for new_unchecked sites that have a
    // legitimate ungated production use case (documented with a justification comment
    // in the source). Each entry is the relative file path (from prism-core/src/).
    //
    // Example (post-AC-6 if ungated production use is justified):
    //   "tenant.rs",  // OrgSlug::new_unchecked: called from boot.rs step-3 after slug validation
];

/// AC-6 Red Gate: asserts no ungated `fn new_unchecked` exists in `crates/prism-core/src/`
/// beyond the known allowlist.
///
/// RED GATE: `OrgSlug::new_unchecked` in tenant.rs is ungated (`pub` with no cfg attribute).
/// It is NOT in `GATED_OR_ALLOWLISTED_UNCHECKED`. The test FAILS.
///
/// Traces to BC-2.01.013 postcondition: validation-bypass constructors are inventoried
/// and justified.
#[test]
fn test_BC_2_01_013_new_unchecked_inventory_baseline() {
    // Locate the prism-core src directory. Integration tests run from the workspace root
    // or from the crate directory. We use `CARGO_MANIFEST_DIR` (set by Cargo for test
    // binaries) to find the source reliably.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    let src_dir = Path::new(&manifest_dir).join("src");

    assert!(
        src_dir.exists(),
        "prism-core/src/ must exist at {:?}",
        src_dir
    );

    let mut violations: Vec<String> = Vec::new();

    // Walk all .rs files in prism-core/src/ recursively.
    walk_rs_files(&src_dir, &mut |file_path: &Path, content: &str| {
        let relative = file_path
            .strip_prefix(&src_dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        // Find every `fn new_unchecked` declaration.
        for (line_idx, line) in content.lines().enumerate() {
            if !line.contains("fn new_unchecked") {
                continue;
            }

            // Check: is this in the allowlist?
            let in_allowlist = GATED_OR_ALLOWLISTED_UNCHECKED
                .iter()
                .any(|allowed| relative.ends_with(allowed));

            if in_allowlist {
                // Allowlisted: no check needed (the AC-6 audit has accepted this site).
                continue;
            }

            // Check: is this line preceded by a cfg(test) or cfg(feature="test-helpers")
            // attribute within the previous 10 lines?
            let window_start = line_idx.saturating_sub(10);
            let preceding = content
                .lines()
                .skip(window_start)
                .take(line_idx - window_start)
                .collect::<Vec<_>>()
                .join("\n");

            let is_gated = preceding.contains("#[cfg(test)]")
                || preceding.contains("cfg(test)")
                || preceding.contains("feature = \"test-helpers\"")
                || preceding.contains("feature=\"test-helpers\"")
                || preceding.contains("cfg(any(test")
                || preceding.contains("cfg(feature");

            if !is_gated {
                violations.push(format!(
                    "UNGATED new_unchecked in {relative} (approx line {}): `{}`\n  \
                     Fix: add #[cfg(any(test, feature=\"test-helpers\"))] before the fn,\n  \
                     OR add the file path to GATED_OR_ALLOWLISTED_UNCHECKED with a justification.",
                    line_idx + 1,
                    line.trim()
                ));
            }
        }
    });

    assert!(
        violations.is_empty(),
        "AC-6 RED GATE: found ungated `fn new_unchecked` in prism-core/src/ that is not \
         in the allowlist:\n\n{}\n\n\
         KNOWN UNGATED SITE (TD-S-PLUGIN-PREREQ-A-006): OrgSlug::new_unchecked in tenant.rs.\n\
         IMPLEMENTATION NEEDED (AC-6): either gate with #[cfg(any(test, feature=\"test-helpers\"))],\n\
         or add to GATED_OR_ALLOWLISTED_UNCHECKED with production use justification.",
        violations.join("\n\n")
    );
}

/// Recursive .rs file walker. Calls `callback` for each file with its path and content.
fn walk_rs_files(dir: &Path, callback: &mut impl FnMut(&Path, &str)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, callback);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                callback(&path, &content);
            }
        }
    }
}
