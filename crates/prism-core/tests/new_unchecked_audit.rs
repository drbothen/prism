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
//! ## HIGH-005 fix (S-PLUGIN-PREREQ-C):
//!
//! The allowlist is now SYMBOL-keyed via `(file_suffix, type_name)` tuples rather
//! than file-suffix-only. A file-suffix-keyed allowlist would silently allowlist
//! any future `fn new_unchecked` added to the same file for a DIFFERENT type.
//!
//! The test walks back from each `fn new_unchecked` declaration to find the
//! enclosing `impl <Type>` block and extracts the type name. Both the file suffix
//! AND the type name must match the allowlist for the site to be accepted.

use std::path::Path;

// ---------------------------------------------------------------------------
// Symbol-keyed allowlist: `new_unchecked` sites that have been audited and accepted.
//
// Format: (file_suffix, type_name)
//   - file_suffix: relative suffix of the source file within crates/prism-core/src/
//   - type_name: the name of the type implementing `fn new_unchecked` (extracted
//     from the `impl <TypeName>` block above the declaration)
//
// HIGH-005 fix: both parts must match for the site to be allowlisted.
// A future `OtherType::new_unchecked` in tenant.rs would NOT be allowlisted
// even though `OrgSlug::new_unchecked` is accepted.
//
// AC-6 audit result (S-PLUGIN-PREREQ-C):
//   - OrgSlug::new_unchecked in tenant.rs is allowlisted with documented justification.
//     HIGH-006 note: the production caller in prism-query/src/materialization.rs has
//     been updated to use OrgSlug::new() (validated constructor) instead of new_unchecked.
//     OrgSlug::new_unchecked remains pub for any future test fixtures that need it; the
//     allowlist entry ensures intentional audit when future new_unchecked sites are added.
// ---------------------------------------------------------------------------
const GATED_OR_ALLOWLISTED_UNCHECKED: &[(&str, &str)] = &[
    // AC-6 audit result (S-PLUGIN-PREREQ-C): OrgSlug::new_unchecked in tenant.rs.
    //
    // The doc-comment on OrgSlug::new_unchecked documents the precondition and production
    // use context (BC-2.03.013 synthetic slug derivation from UUID prefix).
    // Feature-gating is NOT applied because the function may be needed in test fixtures.
    // Symbol-keyed: only OrgSlug in tenant.rs is allowlisted; a future OtherType::new_unchecked
    // in the same file would require explicit allowlist addition.
    ("tenant.rs", "OrgSlug"),
];

/// AC-6 Red Gate: asserts no ungated `fn new_unchecked` exists in `crates/prism-core/src/`
/// beyond the known symbol-keyed allowlist.
///
/// HIGH-005 fix: allowlist is symbol-keyed (file_suffix, type_name) — a future
/// `Tenant::new_unchecked` or `OrgRef::new_unchecked` in tenant.rs would NOT be
/// silently allowlisted; it would require an explicit allowlist addition.
///
/// Traces to BC-2.01.013 postcondition: validation-bypass constructors are inventoried
/// and justified.
#[test]
fn test_BC_2_01_013_new_unchecked_inventory_baseline() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    let src_dir = Path::new(&manifest_dir).join("src");

    assert!(
        src_dir.exists(),
        "prism-core/src/ must exist at {:?}",
        src_dir
    );

    let mut violations: Vec<String> = Vec::new();

    walk_rs_files(&src_dir, &mut |file_path: &Path, content: &str| {
        let relative = file_path
            .strip_prefix(&src_dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let lines: Vec<&str> = content.lines().collect();

        // Find every `fn new_unchecked` declaration.
        for (line_idx, line) in lines.iter().enumerate() {
            if !line.contains("fn new_unchecked") {
                continue;
            }

            // Extract the type name from the enclosing `impl <Type>` block.
            // Walk backwards from this line to find the nearest `impl` declaration.
            let type_name = extract_impl_type_name(&lines, line_idx);

            // Check: is this (file, type) pair in the symbol-keyed allowlist?
            let in_allowlist =
                GATED_OR_ALLOWLISTED_UNCHECKED
                    .iter()
                    .any(|(allowed_file, allowed_type)| {
                        relative.ends_with(allowed_file)
                            && type_name.as_deref() == Some(allowed_type)
                    });

            if in_allowlist {
                // Allowlisted for this specific (file, type) pair.
                continue;
            }

            // Check: is this line preceded by a cfg(test) or cfg(feature="test-helpers")
            // attribute within the previous 10 lines?
            let window_start = line_idx.saturating_sub(10);
            let preceding = lines[window_start..line_idx].join("\n");

            let is_gated = preceding.contains("#[cfg(test)]")
                || preceding.contains("cfg(test)")
                || preceding.contains("feature = \"test-helpers\"")
                || preceding.contains("feature=\"test-helpers\"")
                || preceding.contains("cfg(any(test")
                || preceding.contains("cfg(feature");

            if !is_gated {
                let type_context = type_name.as_deref().unwrap_or("<unknown type>");
                violations.push(format!(
                    "UNGATED new_unchecked in {relative} (approx line {}) for type '{type_context}': `{}`\n  \
                     Fix: add #[cfg(any(test, feature=\"test-helpers\"))] before the fn,\n  \
                     OR add (\"{relative}\", \"{type_context}\") to GATED_OR_ALLOWLISTED_UNCHECKED with justification.",
                    line_idx + 1,
                    line.trim()
                ));
            }
        }
    });

    assert!(
        violations.is_empty(),
        "AC-6: found ungated `fn new_unchecked` in prism-core/src/ not in symbol-keyed allowlist:\n\n{}\n\n\
         To add a new site: append (\"<file_suffix>\", \"<TypeName>\") to GATED_OR_ALLOWLISTED_UNCHECKED\n\
         with a justification comment explaining the production use case.",
        violations.join("\n\n")
    );
}

/// Extract the type name from the nearest enclosing `impl <Type>` block
/// above the given line index.
///
/// Walks backwards through `lines` from `from_line_idx` to find a line
/// matching `impl\s+<TypeName>` (with optional generic params and braces).
/// Returns `Some("TypeName")` if found, `None` if no impl block is found.
fn extract_impl_type_name(lines: &[&str], from_line_idx: usize) -> Option<String> {
    // Search backwards up to 50 lines to find the `impl` block.
    let search_start = from_line_idx.saturating_sub(50);
    for idx in (search_start..from_line_idx).rev() {
        let line = lines[idx].trim();
        // Match `impl TypeName`, `impl<T> TypeName`, `impl TypeName<T>`, etc.
        // Simple heuristic: line starts with `impl` (possibly with generics), followed by a type name.
        if let Some(after_impl) = line.strip_prefix("impl") {
            // Skip generic params on `impl` itself: `impl<T>` or `impl <T>`
            let after_impl = after_impl.trim_start();
            let after_generics = if after_impl.starts_with('<') {
                // Find matching `>`
                let depth_iter = after_impl.char_indices().scan(0i32, |depth, (i, c)| {
                    if c == '<' {
                        *depth += 1;
                    } else if c == '>' {
                        *depth -= 1;
                    }
                    Some((i, c, *depth))
                });
                let end = depth_iter
                    .skip_while(|&(_, _, d)| d > 0)
                    .next()
                    .map(|(i, _, _)| i + 1)
                    .unwrap_or(after_impl.len());
                after_impl[end..].trim_start()
            } else {
                after_impl
            };

            // Extract the type name (alphanumeric + underscores).
            let type_name: String = after_generics
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();

            if !type_name.is_empty() {
                return Some(type_name);
            }
        }
    }
    None
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
