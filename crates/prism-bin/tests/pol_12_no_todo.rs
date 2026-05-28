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

    fn scan_dir(dir: &Path, violations: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, violations);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
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

                    // Allow if the line (or the immediately following context captured by reading
                    // the full file) contains a story-ID citation (S- prefix followed by
                    // alphanumeric/hyphen chars). We use a simple substring scan here.
                    //
                    // Pattern: "S-" followed by one or more uppercase letters or digits or hyphens.
                    // Examples: S-WAVE5-PREP-01, S-1.12-FOLLOWUP, S-5.01-FOLLOWUP-MCP-BOOT.
                    let has_story_citation = {
                        // Look ahead 5 lines in content for multi-line todo strings.
                        let start_byte = content
                            .lines()
                            .take(line_no + 5)
                            .collect::<Vec<_>>()
                            .join("\n")
                            .len();
                        let window: String = content
                            .lines()
                            .skip(line_no)
                            .take(5)
                            .collect::<Vec<_>>()
                            .join("\n");
                        let _ = start_byte;
                        window.contains("S-WAVE")
                            || window.contains("S-1.")
                            || window.contains("S-2.")
                            || window.contains("S-3.")
                            || window.contains("S-4.")
                            || window.contains("S-5.")
                    };

                    if !has_story_citation {
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
