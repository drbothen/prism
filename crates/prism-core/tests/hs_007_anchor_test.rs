//! Red Gate anchor tests for S-3.6.02 — HS-007 Wave 3 BC re-anchor.
//!
//! Verifies that `tests/holdout-scenarios/HS-007-cross-repo-failure.md` has
//! been fully refreshed from Phase 1b stub to a Wave 3 holdout scenario:
//!   - `behavioral_contracts` lists exactly BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002
//!   - `phase` is `3.A`
//!   - All three sub-scenarios (HS-007-01, HS-007-02, HS-007-03) are present
//!   - No TODO / STUB / "(stub:" markers remain in the body
//!   - No legacy Wave 1/2 BC references (BC-1.X.X, BC-2.X.X) appear
//!
//! ALL FIVE TESTS MUST FAIL until the implementer rewrites the HS-007 file
//! (Red Gate discipline per VSDD §TDD).
//!
//! Traces to: BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002
//! Verification properties: VP-131, VP-132, VP-133

use std::path::PathBuf;

/// Resolve the absolute path to `tests/holdout-scenarios/HS-007-cross-repo-failure.md`.
///
/// CARGO_MANIFEST_DIR points to `crates/prism-core/`.
/// The file lives at `<workspace-root>/tests/holdout-scenarios/HS-007-cross-repo-failure.md`.
fn hs007_path() -> PathBuf {
    // crates/prism-core  -> parent -> crates  -> parent -> workspace root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo during test execution");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("prism-core lives inside crates/")
        .parent()
        .expect("crates/ lives inside workspace root")
        .join("tests")
        .join("holdout-scenarios")
        .join("HS-007-cross-repo-failure.md")
}

/// Parse YAML frontmatter from a Markdown file that uses `---` delimiters.
///
/// Returns the raw YAML string between the first and second `---` lines,
/// plus the remainder of the file (body) after the closing `---`.
fn parse_frontmatter(source: &str) -> (String, String) {
    let mut lines = source.lines();

    // Consume the opening `---` (may be preceded by a non-frontmatter first line
    // like a comment / title — handle both orderings robustly).
    // Find the first `---` line.
    let mut fence_count = 0usize;
    let mut yaml_lines: Vec<&str> = Vec::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut in_yaml = false;
    let mut past_yaml = false;

    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed == "---" && !past_yaml {
            fence_count += 1;
            match fence_count {
                1 => {
                    in_yaml = true;
                }
                2 => {
                    in_yaml = false;
                    past_yaml = true;
                }
                _ => {}
            }
            continue;
        }
        if in_yaml {
            yaml_lines.push(line);
        } else if past_yaml {
            body_lines.push(line);
        }
        // Lines before the first `---` are ignored (may be a stub comment).
    }

    (yaml_lines.join("\n"), body_lines.join("\n"))
}

// ---------------------------------------------------------------------------
// test_hs_007_anchored_to_wave_3_bcs
// BC-3.6.001 postcondition 1 / AC-001
// MUST FAIL at Red Gate: behavioral_contracts is [] in the stub.
// ---------------------------------------------------------------------------

/// Asserts that the HS-007 frontmatter `behavioral_contracts` field lists
/// exactly BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002 — no more, no less,
/// no duplicates.
///
/// MUST FAIL at Red Gate: stub file has `behavioral_contracts: []`.
#[test]
fn test_hs_007_anchored_to_wave_3_bcs() {
    let path = hs007_path();
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));

    let (yaml, _body) = parse_frontmatter(&source);

    // Extract the `behavioral_contracts:` line(s).
    // The frontmatter uses an inline YAML sequence: behavioral_contracts: [A, B, C, D]
    // We do a targeted parse rather than pulling in a YAML crate.
    let contracts = extract_yaml_sequence(&yaml, "behavioral_contracts");

    let required: &[&str] = &["BC-3.5.001", "BC-3.5.002", "BC-3.6.001", "BC-3.6.002"];

    // Check no duplicates.
    let mut seen = std::collections::HashSet::new();
    for c in &contracts {
        assert!(
            seen.insert(c.clone()),
            "test_hs_007_anchored_to_wave_3_bcs: duplicate entry '{c}' in behavioral_contracts"
        );
    }

    // Check exact membership (order-independent).
    for req in required {
        assert!(
            contracts.contains(&req.to_string()),
            "test_hs_007_anchored_to_wave_3_bcs: required BC '{req}' missing from \
             behavioral_contracts; got: {contracts:?}"
        );
    }

    // Check no extras.
    for found in &contracts {
        assert!(
            required.contains(&found.as_str()),
            "test_hs_007_anchored_to_wave_3_bcs: unexpected BC '{found}' in \
             behavioral_contracts; allowed: {required:?}"
        );
    }

    // Exact count.
    assert_eq!(
        contracts.len(),
        required.len(),
        "test_hs_007_anchored_to_wave_3_bcs: expected exactly {} behavioral_contracts, \
         got {}: {contracts:?}",
        required.len(),
        contracts.len()
    );
}

// ---------------------------------------------------------------------------
// test_hs_007_phase_is_3a
// MUST FAIL at Red Gate: stub has `phase: "1b"`.
// ---------------------------------------------------------------------------

/// Asserts that the HS-007 frontmatter `phase` field equals `3.A`.
///
/// MUST FAIL at Red Gate: stub file has `phase: "1b"`.
#[test]
fn test_hs_007_phase_is_3a() {
    let path = hs007_path();
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));

    let (yaml, _body) = parse_frontmatter(&source);

    let phase = extract_yaml_scalar(&yaml, "phase").unwrap_or_else(|| {
        panic!("test_hs_007_phase_is_3a: `phase:` key not found in frontmatter")
    });

    assert_eq!(
        phase.as_str(),
        "3.A",
        "test_hs_007_phase_is_3a: expected `phase: 3.A`, got `phase: {phase}`"
    );
}

// ---------------------------------------------------------------------------
// test_hs_007_three_sub_scenarios_present
// MUST FAIL at Red Gate: stub headings are present but marked "(STUB)" —
// the test checks for clean headings without STUB markers.
// ---------------------------------------------------------------------------

/// Asserts that the HS-007 body contains all three required sub-scenario headings:
/// `## HS-007-01`, `## HS-007-02`, `## HS-007-03` — without any STUB suffix.
///
/// MUST FAIL at Red Gate: stub has `## HS-007-01: ... (STUB)` etc.
#[test]
fn test_hs_007_three_sub_scenarios_present() {
    let path = hs007_path();
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));

    let (_yaml, body) = parse_frontmatter(&source);

    let required_headings = &["## HS-007-01", "## HS-007-02", "## HS-007-03"];

    for heading in required_headings {
        // Heading must appear.
        assert!(
            body.contains(heading),
            "test_hs_007_three_sub_scenarios_present: required heading '{heading}' \
             not found in HS-007 body"
        );

        // Heading must NOT be decorated with a STUB suffix on the same line.
        // Find the line containing the heading and check it.
        let stub_on_heading_line = body.lines().filter(|l| l.contains(heading)).any(|l| {
            let upper = l.to_uppercase();
            upper.contains("(STUB)") || upper.contains("STUB)")
        });

        assert!(
            !stub_on_heading_line,
            "test_hs_007_three_sub_scenarios_present: heading '{heading}' still carries \
             a '(STUB)' suffix — implementation must remove the stub marker"
        );
    }
}

// ---------------------------------------------------------------------------
// test_hs_007_no_stub_markers
// MUST FAIL at Red Gate: stub body is full of TODO / STUB / "(stub:" text.
// ---------------------------------------------------------------------------

/// Asserts that no line in the HS-007 file body contains the literal strings
/// `TODO`, `STUB`, or `(stub:` (case-insensitive).
///
/// MUST FAIL at Red Gate: the stub file contains dozens of these markers.
#[test]
fn test_hs_007_no_stub_markers() {
    let path = hs007_path();
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));

    let (_yaml, body) = parse_frontmatter(&source);

    let forbidden = &["TODO", "STUB", "(stub:"];

    let mut violations: Vec<String> = Vec::new();
    for (line_no, line) in body.lines().enumerate() {
        let upper = line.to_uppercase();
        for marker in forbidden {
            if upper.contains(&marker.to_uppercase()) {
                violations.push(format!("  line {}: {}", line_no + 1, line.trim()));
                break;
            }
        }
    }

    assert!(
        violations.is_empty(),
        "test_hs_007_no_stub_markers: found {} line(s) with stub/TODO markers \
         in HS-007 body — implementation must replace all stubs:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// test_hs_007_no_legacy_wave_bc_references
// MUST FAIL at Red Gate: stub has "BC Anchors:" lines with "TODO — [BC-3.6.001...]"
// but more importantly the notes field and STUB comment references must be gone.
// Also guards against any future re-introduction of BC-1.X or BC-2.X patterns.
// ---------------------------------------------------------------------------

/// Asserts that no "BC Anchors:" line in the HS-007 body references a legacy
/// BC from Wave 1 (BC-1.X.X) or Wave 2 (BC-2.X.X).
///
/// Additionally asserts no "BC Anchors:" line contains the literal text `TODO`
/// or an empty bracket pair `[]`, which would indicate unresolved stub anchors.
///
/// MUST FAIL at Red Gate: stub has `**BC Anchors:** TODO — [BC-3.6.001...] (stub: not yet installed)`.
#[test]
fn test_hs_007_no_legacy_wave_bc_references() {
    let path = hs007_path();
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));

    let (_yaml, body) = parse_frontmatter(&source);
    let legacy_pattern =
        regex::Regex::new(r"BC-[12]\.\d+\.\d+").expect("static regex must compile");

    let mut violations: Vec<String> = Vec::new();

    for (line_no, line) in body.lines().enumerate() {
        let lower = line.to_lowercase();
        if !lower.contains("bc anchors") && !lower.contains("bc-") {
            continue;
        }

        // Check for legacy BC-1.X.X or BC-2.X.X patterns anywhere on BC-bearing lines.
        if legacy_pattern.is_match(line) {
            violations.push(format!(
                "  line {}: legacy BC reference — {}",
                line_no + 1,
                line.trim()
            ));
        }

        // Check that "BC Anchors:" lines do not contain TODO or unresolved stub markers.
        if lower.contains("bc anchors") {
            let upper = line.to_uppercase();
            if upper.contains("TODO") {
                violations.push(format!(
                    "  line {}: unresolved TODO in BC Anchors line — {}",
                    line_no + 1,
                    line.trim()
                ));
            }
            // An empty-bracket anchor `[]` means the stub was not replaced.
            if line.contains("[]") {
                violations.push(format!(
                    "  line {}: empty BC Anchors bracket [] — {}",
                    line_no + 1,
                    line.trim()
                ));
            }
            // "(stub:" anywhere on a BC Anchors line.
            if lower.contains("(stub:") {
                violations.push(format!(
                    "  line {}: stub marker on BC Anchors line — {}",
                    line_no + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "test_hs_007_no_legacy_wave_bc_references: found {} violation(s):\n{}",
        violations.len(),
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Helper: extract a YAML inline sequence value for a given key.
//
// Handles both:
//   key: [A, B, C]
//   key: []
//
// Returns a Vec<String> of trimmed, quote-stripped items.
// ---------------------------------------------------------------------------

fn extract_yaml_sequence(yaml: &str, key: &str) -> Vec<String> {
    let prefix = format!("{key}:");
    for line in yaml.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(prefix.as_str()) {
            continue;
        }
        // Everything after "key:" on the same line.
        let rest = trimmed[prefix.len()..].trim();
        if rest.starts_with('[') && rest.ends_with(']') {
            let inner = &rest[1..rest.len() - 1];
            if inner.trim().is_empty() {
                return Vec::new();
            }
            return inner
                .split(',')
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}

// ---------------------------------------------------------------------------
// Helper: extract a YAML scalar value for a given key.
//
// Handles:
//   key: value
//   key: "value"
//   key: 'value'
// ---------------------------------------------------------------------------

fn extract_yaml_scalar(yaml: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(prefix.as_str()) {
            let rest = trimmed[prefix.len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            return Some(rest);
        }
    }
    None
}
