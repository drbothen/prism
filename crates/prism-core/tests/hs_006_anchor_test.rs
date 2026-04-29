//! HS-006 anchor validation tests — Red Gate phase for story S-3.6.01.
//!
//! These tests assert that the refreshed `.factory/holdout-scenarios/HS-006-state-recovery.md`
//! (path: `tests/holdout-scenarios/HS-006-state-recovery.md` from workspace root) satisfies
//! the Wave 3 BC anchoring requirements defined in AC-001 through AC-005 of story S-3.6.01.
//!
//! ALL tests in this file MUST FAIL at Red Gate (stub state) and MUST PASS after the
//! implementer completes the HS-006 refresh in the next phase.
//!
//! Traces to: BC-3.6.001, BC-3.6.002, BC-3.5.001, BC-3.2.001, BC-3.2.003
//! Verification properties: VP-128, VP-129, VP-130
//! Closes: TD-HOLDOUT-W2-002

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helper: locate and read the HS-006 holdout scenario file.
//
// CARGO_MANIFEST_DIR is `crates/prism-core` at test-time.
// Navigate up two levels to reach the workspace root, then descend to the
// holdout file at `tests/holdout-scenarios/HS-006-state-recovery.md`.
// ---------------------------------------------------------------------------

fn hs006_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo during test execution");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("prism-core resides inside crates/")
        .parent()
        .expect("crates/ parent is the workspace root")
        .join("tests")
        .join("holdout-scenarios")
        .join("HS-006-state-recovery.md")
}

fn read_hs006() -> String {
    let path = hs006_path();
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read HS-006 file at {}: {e}", path.display()))
}

/// Extract the YAML frontmatter block (content between the first pair of `---` delimiters).
///
/// The stub file begins with a comment line before the first `---`, so we search
/// for the first occurrence of a `---` line rather than assuming it is on line 1.
fn extract_frontmatter(content: &str) -> &str {
    let mut lines = content.splitn(3, "---\n");
    // Skip any leading content before the first `---`
    let _ = lines.next(); // may be empty or a comment block
    let fm = lines
        .next()
        .expect("HS-006 file must contain a YAML frontmatter block delimited by '---'");
    fm
}

// ---------------------------------------------------------------------------
// AC-001 / BC-3.6.001 + BC-3.6.002 + BC-3.5.001 + BC-3.2.001 + BC-3.2.003
//
// Asserts:
//   - `behavioral_contracts` field is a non-empty list
//   - It contains EXACTLY: BC-3.2.001, BC-3.2.003, BC-3.5.001, BC-3.6.001, BC-3.6.002
//     (in any order, no duplicates, no extras)
//
// MUST FAIL at Red Gate: stub has `behavioral_contracts: []`
// ---------------------------------------------------------------------------

const REQUIRED_BCS: &[&str] = &[
    "BC-3.2.001",
    "BC-3.2.003",
    "BC-3.5.001",
    "BC-3.6.001",
    "BC-3.6.002",
];

/// AC-001 (VP-128): `behavioral_contracts` frontmatter field lists exactly the 5 Wave 3 BCs.
///
/// MUST FAIL at Red Gate: stub has `behavioral_contracts: []`.
#[test]
fn test_hs_006_anchored_to_wave_3_bcs() {
    let content = read_hs006();
    let frontmatter = extract_frontmatter(&content);

    // Find the `behavioral_contracts:` line in the frontmatter.
    let bc_line = frontmatter
        .lines()
        .find(|l| l.trim_start().starts_with("behavioral_contracts:"))
        .expect(
            "AC-001: `behavioral_contracts:` key must be present in HS-006 frontmatter.\n\
             Red Gate: this key is present but the list is empty.",
        );

    // Extract the inline list value (e.g. `[BC-3.6.001, BC-3.6.002, BC-3.5.001]`).
    // We accept either inline YAML list or multi-line list items.
    let declared_bcs: Vec<String> = {
        let after_colon = bc_line
            .splitn(2, ':')
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();

        if after_colon.starts_with('[') {
            // Inline list: `[BC-3.6.001, BC-3.6.002, ...]`
            after_colon
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else if after_colon.is_empty() || after_colon == "|" || after_colon == ">" {
            // Multi-line list: lines starting with `  - BC-X.X.XXX`
            let bc_key_idx = frontmatter
                .lines()
                .position(|l| l.trim_start().starts_with("behavioral_contracts:"))
                .expect("behavioral_contracts key must exist");

            frontmatter
                .lines()
                .skip(bc_key_idx + 1)
                .take_while(|l| l.starts_with("  -") || l.starts_with("- "))
                .map(|l| {
                    l.trim_start_matches(|c: char| c == ' ' || c == '-')
                        .trim()
                        .to_string()
                })
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // Single scalar or unexpected format — treat as a single-element list.
            vec![after_colon]
        }
    };

    // Assert: list must be non-empty.
    assert!(
        !declared_bcs.is_empty(),
        "AC-001: `behavioral_contracts` must be a non-empty list.\n\
         Current value: {bc_line}\n\
         Red Gate failure: stub has `behavioral_contracts: []`."
    );

    // Assert: no duplicates.
    let mut deduped = declared_bcs.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(
        declared_bcs.len(),
        deduped.len(),
        "AC-001: `behavioral_contracts` must not contain duplicates.\n\
         Declared: {declared_bcs:?}"
    );

    // Assert: contains ALL required BCs.
    for required in REQUIRED_BCS {
        assert!(
            declared_bcs.iter().any(|bc| bc == required),
            "AC-001: `behavioral_contracts` must contain '{required}'.\n\
             Current list: {declared_bcs:?}\n\
             Required: {REQUIRED_BCS:?}"
        );
    }

    // Assert: contains NO extras beyond the required set.
    for declared in &declared_bcs {
        assert!(
            REQUIRED_BCS.contains(&declared.as_str()),
            "AC-001: `behavioral_contracts` contains unexpected entry '{declared}'.\n\
             Only the following BCs are permitted: {REQUIRED_BCS:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-001 supplementary / BC-3.6.002 postcondition 2
//
// Asserts:
//   - `phase` field is `3.A`
//   - `closes_td` field contains `TD-HOLDOUT-W2-002`
//
// MUST FAIL at Red Gate: stub has `closes_td: []` (phase is already 3.A, but
// closes_td is empty — so the combined assertion fails).
// ---------------------------------------------------------------------------

/// AC-001 supplementary (VP-129): `phase` is `3.A` and `closes_td` includes `TD-HOLDOUT-W2-002`.
///
/// MUST FAIL at Red Gate: stub has `closes_td: []`.
#[test]
fn test_hs_006_phase_is_3a() {
    let content = read_hs006();
    let frontmatter = extract_frontmatter(&content);

    // Assert: phase == "3.A"
    let phase_line = frontmatter
        .lines()
        .find(|l| l.trim_start().starts_with("phase:"))
        .expect("AC-001: `phase:` key must be present in HS-006 frontmatter");

    let phase_value = phase_line
        .splitn(2, ':')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .to_string();

    assert_eq!(
        phase_value, "3.A",
        "AC-001: `phase` must be '3.A', got '{phase_value}'"
    );

    // Assert: closes_td contains TD-HOLDOUT-W2-002
    let closes_td_line = frontmatter
        .lines()
        .find(|l| l.trim_start().starts_with("closes_td:"))
        .expect("AC-001: `closes_td:` key must be present in HS-006 frontmatter");

    assert!(
        closes_td_line.contains("TD-HOLDOUT-W2-002"),
        "AC-001: `closes_td` must contain 'TD-HOLDOUT-W2-002'.\n\
         Current value: {closes_td_line}\n\
         Red Gate failure: stub has `closes_td: []`."
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.6.002 postcondition 3
//
// Asserts: no sub-scenario body contains the literal text "TODO", "STUB",
//          or "(stub:" — every sub-scenario must be fully written.
//
// MUST FAIL at Red Gate: stub is full of TODO and STUB markers.
// ---------------------------------------------------------------------------

/// AC-002 (VP-129): No sub-scenario may contain "TODO", "STUB", or "(stub:".
///
/// MUST FAIL at Red Gate: stub file is full of TODO/STUB markers.
#[test]
fn test_hs_006_no_stub_markers() {
    let content = read_hs006();

    // Extract the body (everything after the closing `---` of the frontmatter).
    let body = {
        // Find the second `---` delimiter which closes the frontmatter.
        let mut parts = content.splitn(3, "---\n");
        let _ = parts.next(); // pre-frontmatter comment (or empty)
        let _ = parts.next(); // frontmatter YAML body
        parts
            .next()
            .expect("HS-006 file must have content after the closing '---' frontmatter delimiter")
    };

    let forbidden = &["TODO", "STUB", "(stub:"];
    let mut violations: Vec<(usize, &str, &str)> = Vec::new();

    for (line_no, line) in body.lines().enumerate() {
        for marker in forbidden {
            if line.contains(marker) {
                violations.push((line_no + 1, marker, line));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "AC-002: sub-scenario body must not contain stub markers.\n\
         Found {} violation(s):\n{}\n\
         Red Gate failure: stub file contains TODO/STUB placeholders throughout.",
        violations.len(),
        violations
            .iter()
            .take(10) // limit output to first 10 for readability
            .map(|(ln, marker, text)| format!("  line {ln}: [{marker}] {text}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.6.002 postcondition 3
//
// Asserts: the file body contains all 7 sub-scenario headings:
//   HS-006-01, HS-006-02, HS-006-03, HS-006-04, HS-006-05, HS-006-06, HS-006-07
//
// This test PASSES at Red Gate (all 7 headings are present in the stub) and
// remains a guard to ensure no heading is accidentally removed during the refresh.
// ---------------------------------------------------------------------------

/// AC-002 (VP-128): All 7 sub-scenario headings must be present in the file body.
///
/// NOTE: This assertion passes at Red Gate (headings exist in stub) and acts as
/// a regression guard to prevent heading removal during the refresh.
#[test]
fn test_hs_006_seven_sub_scenarios_present() {
    let content = read_hs006();

    let required_headings = &[
        "HS-006-01",
        "HS-006-02",
        "HS-006-03",
        "HS-006-04",
        "HS-006-05",
        "HS-006-06",
        "HS-006-07",
    ];

    let mut missing: Vec<&str> = Vec::new();
    for heading in required_headings {
        if !content.contains(heading) {
            missing.push(heading);
        }
    }

    assert!(
        missing.is_empty(),
        "AC-002: the following required sub-scenario headings are missing from HS-006:\n  {:?}\n\
         All 7 sub-scenarios (HS-006-01 through HS-006-07) must be present.",
        missing
    );

    // Also assert that each sub-scenario heading has a non-empty **Expected Outcome**
    // section — not just a TODO placeholder. This is the assertion that MUST FAIL at
    // Red Gate because all Expected Outcome sections contain TODO in the stub.
    let body = {
        let mut parts = content.splitn(3, "---\n");
        let _ = parts.next();
        let _ = parts.next();
        parts
            .next()
            .expect("HS-006 file must have a body after the frontmatter")
    };

    // Split body into sub-scenario blocks by the `## HS-006-` delimiter.
    // Each block must contain an **Expected Outcome:** section with real content.
    let mut incomplete_scenarios: Vec<String> = Vec::new();

    for (block_idx, heading) in required_headings.iter().enumerate() {
        // Find the sub-scenario block for this heading.
        let block_start = match body.find(heading) {
            Some(pos) => pos,
            None => {
                // Already caught above in `missing` check.
                continue;
            }
        };

        // The block ends at the start of the next `## HS-006-` heading, or end-of-body.
        let next_heading = required_headings.get(block_idx + 1);
        let block_end = match next_heading {
            Some(next) => body[block_start..].find(next).map(|p| block_start + p),
            None => None,
        }
        .unwrap_or(body.len());

        let block = &body[block_start..block_end];

        // The Expected Outcome section must exist and must NOT be purely TODO content.
        let has_expected_outcome =
            block.contains("**Expected Outcome:**") || block.contains("Expected Outcome:");

        if !has_expected_outcome {
            incomplete_scenarios.push(format!("{heading}: missing **Expected Outcome** section"));
            continue;
        }

        // Find the Expected Outcome section and check its content is not all-TODO.
        let outcome_start = block
            .find("**Expected Outcome:**")
            .or_else(|| block.find("Expected Outcome:"))
            .expect("Expected Outcome section must be locatable");

        let outcome_section = &block[outcome_start..];
        // The section ends at the next `**` heading or `---` separator.
        let outcome_end = outcome_section[1..]
            .find("**BC Anchors")
            .or_else(|| outcome_section[1..].find("\n---"))
            .map(|p| p + 1)
            .unwrap_or(outcome_section.len());

        let outcome_text = &outcome_section[..outcome_end];

        // Outcome is considered incomplete if every non-empty line starts with TODO.
        let non_empty_lines: Vec<&str> = outcome_text
            .lines()
            .skip(1) // skip the heading line itself
            .filter(|l| !l.trim().is_empty())
            .collect();

        let all_todo = !non_empty_lines.is_empty()
            && non_empty_lines
                .iter()
                .all(|l| l.contains("TODO") || l.contains("NOT YET WRITTEN"));

        if all_todo {
            incomplete_scenarios.push(format!(
                "{heading}: Expected Outcome section contains only TODO/NOT-YET-WRITTEN placeholders"
            ));
        }
    }

    assert!(
        incomplete_scenarios.is_empty(),
        "AC-002 / VP-128: All 7 sub-scenarios must have complete Expected Outcome sections.\n\
         Incomplete scenarios ({}):\n{}\n\
         Red Gate failure: stub Expected Outcome sections are all-TODO placeholders.",
        incomplete_scenarios.len(),
        incomplete_scenarios
            .iter()
            .map(|s| format!("  - {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.5.001 postcondition 2 + BC-3.2.001 / BC-3.2.003
//
// Asserts: no sub-scenario's "BC Anchors:" line references Wave 1 or Wave 2 BCs
//          (i.e. BC-1.X.XXX or BC-2.X.XXX patterns).
//
// MUST FAIL at Red Gate because the "BC Anchors:" lines in the stub contain
// only TODO markers — they do not contain ANY actual BC refs, including no
// Wave 3 refs. The complementary positive assertion (that each BC Anchors line
// contains at least one Wave 3 BC-3.x.xxx ref) will fail.
// ---------------------------------------------------------------------------

/// AC-002 (VP-130): No "BC Anchors:" line may reference Wave 1 or Wave 2 BCs.
/// Each "BC Anchors:" line must also contain at least one valid Wave 3 BC reference.
///
/// MUST FAIL at Red Gate: stub "BC Anchors:" lines contain only TODO placeholders
/// with no actual BC-3.x.xxx references.
#[test]
fn test_hs_006_no_legacy_wave_bc_references() {
    let content = read_hs006();
    let body = {
        let mut parts = content.splitn(3, "---\n");
        let _ = parts.next();
        let _ = parts.next();
        parts
            .next()
            .expect("HS-006 file must have a body after the frontmatter")
    };

    // Pattern for legacy Wave 1/2 BCs.
    let legacy_pattern =
        regex::Regex::new(r"BC-[12]\.\d+\.\d+").expect("valid regex for Wave 1/2 BC IDs");

    // Pattern for valid Wave 3 BCs.
    let wave3_pattern =
        regex::Regex::new(r"BC-3\.\d+\.\d+").expect("valid regex for Wave 3 BC IDs");

    let mut legacy_violations: Vec<(usize, String)> = Vec::new();
    let mut anchor_lines_without_wave3: Vec<(usize, String)> = Vec::new();

    for (line_no, line) in body.lines().enumerate() {
        let is_bc_anchors_line = line.trim_start().starts_with("**BC Anchors:**")
            || line.trim_start().starts_with("**BC Anchors:**")
            || line.contains("BC Anchors:");

        if !is_bc_anchors_line {
            continue;
        }

        // Check for legacy Wave 1/2 references.
        if legacy_pattern.is_match(line) {
            legacy_violations.push((line_no + 1, line.to_string()));
        }

        // Check that the line contains at least one Wave 3 BC reference.
        if !wave3_pattern.is_match(line) {
            anchor_lines_without_wave3.push((line_no + 1, line.to_string()));
        }
    }

    assert!(
        legacy_violations.is_empty(),
        "AC-002 / VP-130: BC Anchors lines must not reference Wave 1 or Wave 2 BCs.\n\
         Found {} legacy reference(s):\n{}\n",
        legacy_violations.len(),
        legacy_violations
            .iter()
            .map(|(ln, text)| format!("  line {ln}: {text}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    assert!(
        anchor_lines_without_wave3.is_empty(),
        "AC-002 / VP-130: every 'BC Anchors:' line must reference at least one Wave 3 BC \
         (BC-3.x.xxx).\n\
         Found {} anchor line(s) with no Wave 3 BC reference:\n{}\n\
         Red Gate failure: stub 'BC Anchors:' lines contain only TODO placeholders.",
        anchor_lines_without_wave3.len(),
        anchor_lines_without_wave3
            .iter()
            .map(|(ln, text)| format!("  line {ln}: {text}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
