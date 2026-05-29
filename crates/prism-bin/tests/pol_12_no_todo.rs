//! POL-12 (AC-10 extension): no `todo!()` or `unimplemented!()` in prism-bin production source
//! files unless the call cites a specific story ID (S-NNN pattern).
//!
//! MED-005 fix: extends the AC-10 gate (originally prism-mcp–only) to cover prism-bin/src/.
//!
//! Allowed exceptions: `todo!()` calls that include a story ID citation of the form `S-`
//! followed by alphanumeric characters (e.g., `todo!("S-WAVE5-PREP-01 step 10 — …")`).
//! These are intentional, traceable deferrals to named follow-up stories.
//!
//! Disallowed: bare `todo!()`, `todo!("placeholder")`, or any `todo!` without a story citation.
//! This prevents accidental shipping of unnamed stubs.

#[test]
fn test_pol_12_no_untraced_todo_in_prism_bin_production_code() {
    use std::path::Path;

    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations: Vec<String> = Vec::new();

    // SUG-7(a): use regex-style pattern to match any S-<word> story citation.
    // Previous approach used a prefix list (S-1., S-2., …, S-WAVE) which would miss future
    // S-6. / S-7. stories and required manual maintenance.
    // The canonical story-ID format is: S- followed by one or more alphanumeric/hyphen chars.
    // We use a lightweight manual scan (no regex dep) that accepts any "S-" followed by
    // at least one [A-Z0-9a-z] character.
    fn has_story_id(window: &str) -> bool {
        let bytes = window.as_bytes();
        let mut i = 0;
        while i + 2 <= bytes.len() {
            if bytes[i] == b'S' && bytes[i + 1] == b'-' {
                // Must have at least one alphanumeric/digit after "S-"
                if i + 2 < bytes.len() && bytes[i + 2].is_ascii_alphanumeric() {
                    return true;
                }
            }
            i += 1;
        }
        false
    }

    fn scan_dir(dir: &Path, violations: &mut Vec<String>) {
        // SUG-7(b): propagate I/O errors as test failures instead of silently skipping.
        let entries = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("POL-12: failed to read directory {}: {e}", dir.display()));
        for entry in entries {
            let entry =
                entry.unwrap_or_else(|e| panic!("POL-12: failed to read directory entry: {e}"));
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, violations);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                // SUG-7(b): propagate file read errors instead of silently skipping.
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("POL-12: failed to read {}: {e}", path.display()));
                for (line_no, line) in content.lines().enumerate() {
                    let has_todo = line.contains("todo!(") || line.contains("unimplemented!(");
                    if !has_todo {
                        continue;
                    }

                    // Allow lines that are purely comments (grep the non-comment portion).
                    // Simple heuristic: skip if line starts with whitespace + "//"
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Allow if the 5-line window starting at this line contains a story-ID citation.
                    // Look-ahead covers multi-line todo!() strings.
                    // SUG-7(a): use has_story_id() instead of prefix list.
                    // SUG-7(c): removed dead `let _ = start_byte` computation.
                    let window: String = content
                        .lines()
                        .skip(line_no)
                        .take(5)
                        .collect::<Vec<_>>()
                        .join("\n");
                    let has_citation = has_story_id(&window);

                    if !has_citation {
                        violations.push(format!(
                            "{}:{}: {}",
                            path.display(),
                            line_no + 1,
                            line.trim()
                        ));
                    }
                }
            }
        }
    }

    scan_dir(&src_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "POL-12 (AC-10 extension): found {} untraced todo!()/unimplemented!() in \
         prism-bin production source files (story-ID citations required for intentional deferrals):\n{}",
        violations.len(),
        violations.join("\n")
    );
}
