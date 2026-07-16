//! Spec-prose fidelity tests for `claroty.sensor.toml` audit_logs table comments.
//!
//! These tests enforce the TOML comment corrections defined by
//! S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 (wave-5-e-demo-fidelity).
//!
//! Red Gate layout:
//!
//! | Test | Red Gate? | Reason for initial state |
//! |------|-----------|--------------------------|
//! | `test_BC_2_16_013_AC001_audit_logs_no_stale_dtu_gap_comments` | GREEN pre-impl | Stale comments already removed by the `docs(S-DEMO-CLAROTY-AUDIT-DTU-001)` commit (9e4e17bf); AC-001 is now a no-regression guard verifying they stay absent. |
//! | `test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present` | RED pre-impl | `"Gap-CL-006 CLOSED"` comment line not yet written; this is the genuine Red Gate assertion. |
//! | `test_BC_2_16_013_AC004_audit_logs_functional_fields_unchanged` | GREEN pre-impl | No-regression guard — asserts functional TOML content is intact before and after. |
//!
//! Traces to: BC-2.16.013 §Postconditions §1 (audit_logs clause).
//!
//! Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
#![allow(clippy::expect_used, clippy::unwrap_used)]

use prism_spec_engine::spec_parser::SpecLoader;

// ---------------------------------------------------------------------------
// Helper: load and scope the audit_logs block from the TOML file.
// ---------------------------------------------------------------------------

/// Extract the text content of the `audit_logs` table block from `claroty.sensor.toml`.
///
/// Strategy: scan from the `[[tables]]` line that immediately precedes
/// `table_name = "audit_logs"` through to the next `[[tables]]` separator (exclusive)
/// or end-of-file, whichever comes first.
///
/// This scoping prevents false-positive or false-negative matches from lines in
/// the `alerts` or `devices` table blocks.
fn audit_logs_block() -> String {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let full_content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    // Find the `[[tables]]` line that owns `table_name = "audit_logs"`.
    // We walk line-by-line and locate the boundary of the block.
    let lines: Vec<&str> = full_content.lines().collect();

    // Find the line index of `table_name = "audit_logs"`
    let audit_logs_name_idx = lines
        .iter()
        .position(|l| l.trim() == r#"table_name = "audit_logs""#)
        .expect("claroty.sensor.toml must contain 'table_name = \"audit_logs\"'");

    // Walk backwards from that index to find the opening `[[tables]]` line.
    let block_start = (0..=audit_logs_name_idx)
        .rev()
        .find(|&i| lines[i].trim() == "[[tables]]")
        .expect("audit_logs table must be preceded by a [[tables]] header");

    // Walk forwards from `block_start + 1` to find the next `[[tables]]` or end-of-file.
    let block_end = lines
        .iter()
        .enumerate()
        .skip(block_start + 1)
        .find(|(_, l)| l.trim() == "[[tables]]")
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    lines[block_start..block_end].join("\n")
}

// ---------------------------------------------------------------------------
// AC-001 — No stale "DTU gap" comments in the audit_logs block
//
// GREEN-BY-DESIGN (pre-implementation): the stale "DTU gap" / "no route" /
// "404 until DTU route lands" comments were removed by the earlier
// `docs(S-DEMO-CLAROTY-AUDIT-DTU-001)` commit (9e4e17bf) — before this story
// was actioned. This test is therefore a no-regression guard: it asserts those
// stale strings remain absent after the implementer touches the comment lines.
//
// If any stale comment string re-appears in the audit_logs block (e.g., from an
// incorrect revert or incorrect patch), this test becomes a red gate.
//
// Traces to: BC-2.16.013 §Postconditions §1 — audit_logs clause.
// ---------------------------------------------------------------------------

/// GREEN-BY-DESIGN (no-regression guard): audit_logs block must not contain
/// `"DTU gap"`, `"no /api/v1/audit_log/get route"`, or `"404 until DTU route lands"`.
///
/// These comment substrings described the pre-S-DEMO-CLAROTY-AUDIT-DTU-001 state
/// of the Claroty audit_log route gap. After Gap-CL-006 was closed by
/// S-DEMO-CLAROTY-AUDIT-DTU-001, these comments became stale misinformation
/// (F-P2-DEFER-001). They were removed in the `docs(S-DEMO-CLAROTY-AUDIT-DTU-001)`
/// commit and must remain absent.
///
/// Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 AC-001
/// BC: BC-2.16.013 §Postconditions §1
#[test]
fn test_BC_2_16_013_AC001_audit_logs_no_stale_dtu_gap_comments() {
    let block = audit_logs_block();

    assert!(
        !block.contains("DTU gap"),
        "audit_logs block must not contain stale 'DTU gap' comment; found it in:\n{block}"
    );
    assert!(
        !block.contains("no /api/v1/audit_log/get route"),
        "audit_logs block must not contain stale 'no /api/v1/audit_log/get route' comment; found it in:\n{block}"
    );
    assert!(
        !block.contains("404 until DTU route lands"),
        "audit_logs block must not contain stale '404 until DTU route lands' comment; found it in:\n{block}"
    );
}

// ---------------------------------------------------------------------------
// AC-002 — Gap-CL-006 closure comment MUST be present in the audit_logs block
//
// RED gate (pre-implementation): the `# Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.`
// comment line has not yet been written into the audit_logs block.
// This test FAILS before the implementer adds the closure comment.
// It PASSES only after the implementer adds a line containing both
// "Gap-CL-006 CLOSED" and "S-DEMO-CLAROTY-AUDIT-DTU-001" to the block.
//
// Traces to: BC-2.16.013 §Postconditions §1 — prose accurately reflects
// closed gap (Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001).
// ---------------------------------------------------------------------------

/// RED gate: audit_logs block must contain a comment line with `"Gap-CL-006 CLOSED"`.
///
/// S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 §Change 1 requires replacing all stale "DTU gap"
/// comments with:
///
///   # Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.
///   # POST /api/v1/audit_log/get route registered in prism-dtu-claroty.
///
/// Until the implementer adds that comment, this assertion fails.
///
/// Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 AC-002
/// BC: BC-2.16.013 §Postconditions §1
#[test]
fn test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present() {
    let block = audit_logs_block();

    // Both "Gap-CL-006 CLOSED" and "S-DEMO-CLAROTY-AUDIT-DTU-001" must appear on the
    // same comment line (e.g. `# Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.`).
    // Two independent contains() checks would false-pass if the strings landed on
    // unrelated lines — co-occurrence on one line is the correct assertion.
    assert!(
        block.lines().any(|line| line.contains("Gap-CL-006 CLOSED")
            && line.contains("S-DEMO-CLAROTY-AUDIT-DTU-001")),
        "audit_logs block must contain a single comment line with both \
         'Gap-CL-006 CLOSED' and 'S-DEMO-CLAROTY-AUDIT-DTU-001'; not found in:\n{block}"
    );
}

// ---------------------------------------------------------------------------
// AC-004 — No functional TOML content changed (no-regression guard)
//
// GREEN-BY-DESIGN (both before and after implementation): the implementer only
// modifies comment lines. The functional TOML fields (path_template, method,
// response_path, columns) must remain identical before and after this story.
//
// This test parses the full claroty.sensor.toml via the canonical SpecLoader::parse
// API, finds the audit_logs TableSpec, and asserts every functional field matches
// the expected values grounded from BC-2.16.013 §Postconditions §1 and
// the Gap-CL-002 fix (path corrected to /api/v1/audit_log/get in 72baf413).
//
// Traces to: BC-2.16.013 §Postconditions §1 — TOML functional content
// already correct per earlier Gap-CL-002 fix; only comment lines are in scope.
// ---------------------------------------------------------------------------

/// GREEN-BY-DESIGN (no-regression guard): audit_logs functional fields must be intact.
///
/// Parses `claroty.sensor.toml` via `SpecLoader::parse` (canonical TOML loader)
/// and asserts:
/// - `path_template` == `"/api/v1/audit_log/get/"` (with trailing slash per S-DEMO-CLAROTY-TRAILING-SLASH-001)
/// - `method` == `"POST"` (POST-for-read pattern per BC-2.16.013 §Postconditions §1)
/// - `response_path` == `"$.audit_log"` (per DTU GetAuditLogResponse struct)
/// - Expected columns are present: `id`, `action`, `actor`, `timestamp`, `resource`
///
/// This guards that the comment-only edit in AC-001/AC-002 does not accidentally alter
/// any functional TOML key.
///
/// Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 AC-004
/// BC: BC-2.16.013 §Postconditions §1
#[test]
fn test_BC_2_16_013_AC004_audit_logs_functional_fields_unchanged() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec = SpecLoader::parse(&content)
        .unwrap_or_else(|e| panic!("SpecLoader::parse failed for claroty.sensor.toml: {e:?}"));

    // Locate the audit_logs TableSpec.
    let audit_logs = spec
        .tables
        .iter()
        .find(|t| t.table_name == "audit_logs")
        .expect("claroty.sensor.toml must declare a table named 'audit_logs'");

    // --- Step-level assertions ---
    assert_eq!(
        audit_logs.steps.len(),
        1,
        "audit_logs table must have exactly 1 fetch step"
    );
    let step = &audit_logs.steps[0];

    assert_eq!(
        step.method, "POST",
        "audit_logs fetch step must use POST (POST-for-read pattern per BC-2.16.013 §PC §1)"
    );

    assert_eq!(
        step.path_template, "/api/v1/audit_log/get/",
        "audit_logs fetch step path_template must be '/api/v1/audit_log/get/' \
         (Gap-CL-002 fix + trailing slash per S-DEMO-CLAROTY-TRAILING-SLASH-001)"
    );

    assert_eq!(
        step.response_path, "$.audit_log",
        "audit_logs fetch step response_path must be '$.audit_log' \
         (DTU GetAuditLogResponse struct field name)"
    );

    // --- Column-set assertions ---
    let column_names: Vec<&str> = audit_logs.columns.iter().map(|c| c.name.as_str()).collect();

    for expected_col in &["id", "action", "actor", "timestamp", "resource"] {
        assert!(
            column_names.contains(expected_col),
            "audit_logs columns must include '{expected_col}'; got: {column_names:?}"
        );
    }

    assert_eq!(
        column_names.len(),
        5,
        "audit_logs must have exactly 5 columns (id, action, actor, timestamp, resource); \
         got {}: {column_names:?}",
        column_names.len()
    );
}
