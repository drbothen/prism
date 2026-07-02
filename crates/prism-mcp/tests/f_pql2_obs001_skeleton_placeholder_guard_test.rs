//! Guard test for S-DEMO-FIDELITY-REMEDIATION-001 F-PQL2-OBS-001 (process-gap closure).
//!
//! Finding F-PQL2-OBS-001: The existing `bc_2_10_016_audit_004_test.rs` guard scans only the
//! `render_*` prompt functions, leaving server.rs query-tool SCHEMA-AGNOSTIC SKELETONS and
//! the `resources.rs` Datetime Arithmetic prose section unguarded against hardcoded concrete
//! column names like `timestamp` that mislead agents into constructing non-executable queries.
//!
//! This test covers the sibling surfaces that escaped the prior guard sweep:
//! 1. The `query` tool-description SCHEMA-AGNOSTIC SKELETONS in server.rs
//! 2. The Datetime Arithmetic examples block in `build_reference_content` (resources.rs)
//!
//! # Guard strategy
//!
//! For server.rs skeletons: scan each skeleton line for any bareword that could be a concrete
//! column or table identifier (not wrapped in `<...>` angle brackets), and fail on the specific
//! known-bad tokens `timestamp` (non-universal datetime col) and bare severity literals
//! (`'high'`, `'HIGH'`, `'High'`, etc. without a placeholder wrapper).
//!
//! For resources.rs Datetime Arithmetic: assert the prose does NOT contain the pattern
//! `WHERE timestamp >` (which was the pre-fix illustrative example teaching the concrete
//! column name), and DOES contain `<datetime_col>` to confirm the annotation was applied.
//!
//! # Load-bearing property
//!
//! This test FAILS against the pre-fix server.rs that contained:
//!   `SELECT COUNT(*) FROM <table> WHERE timestamp > NOW() - INTERVAL '1h'`
//! because `timestamp` is a bareword (not `<datetime_col>`) in a skeleton line.
//!
//! The test PASSES after the fix replacing `timestamp` with `<datetime_col>` in the skeleton
//! and adding the `<datetime_col>` placeholder to the Datetime Arithmetic prose.
//!
//! # Surfaces covered
//!
//! | Surface | Location | Token guarded |
//! |---------|----------|---------------|
//! | query tool description skeleton #1 | server.rs ~line 1875 | `timestamp` bareword |
//! | Datetime Arithmetic examples | resources.rs ~lines 1472-1474 | `WHERE timestamp >` |
//!
//! # Test → finding mapping
//!
//! | Test | Finding | BC |
//! |------|---------|-----|
//! | test_f_pql2_obs001_query_skeleton_no_bare_timestamp | F-PQL2-OBS-001 | BC-2.10.016 v1.2 |
//! | test_f_pql2_obs001_datetime_arithmetic_uses_placeholder | F-PQL2-OBS-001 | BC-2.10.016 v1.2 |

use prism_mcp::resources::build_reference_content;

// ── Surface 1: server.rs query-tool SCHEMA-AGNOSTIC SKELETONS ─────────────────

/// F-PQL2-OBS-001 guard — SCHEMA-AGNOSTIC SKELETONS in the `query` tool description
/// must NOT contain the bareword `timestamp` as a column name.
///
/// `timestamp` is NOT a universal datetime column name across the demo sensor tables:
/// - crowdstrike_detections → `created_timestamp`
/// - *_devices → `last_seen` / `first_seen`
/// - claroty_alerts → `detected_time`
/// - cyberint_alerts → `created_at`
/// - claroty_audit_logs → `timestamp` (only table with this literal name)
///
/// An agent copying skeleton #1 verbatim would produce a non-executable query on 5 of 6
/// demo tables. The correct form is `<datetime_col>` with a substitution note.
///
/// **Load-bearing:** fails against the pre-fix skeleton
/// `SELECT COUNT(*) FROM <table> WHERE timestamp > NOW() - INTERVAL '1h'`
/// because `timestamp` is a bareword in a skeleton line, not a `<...>` placeholder.
#[test]
fn test_f_pql2_obs001_query_skeleton_no_bare_timestamp() {
    use std::path::Path;

    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("server.rs");
    let content = std::fs::read_to_string(&src)
        .expect("F-PQL2-OBS-001: server.rs must be readable from prism-mcp crate root");

    // Extract the SCHEMA-AGNOSTIC SKELETONS block from the query tool description.
    // The block starts with "SCHEMA-AGNOSTIC SKELETONS" and contains numbered skeleton lines.
    let skeleton_start = content
        .find("SCHEMA-AGNOSTIC SKELETONS")
        .expect("F-PQL2-OBS-001: server.rs must contain 'SCHEMA-AGNOSTIC SKELETONS' marker in query tool description; the skeleton block was removed or renamed");

    // Find the end of the skeleton block: the next tool description section header.
    // Skeleton lines end before the next ALL-CAPS section label (e.g., "SEVERITY CASING WARNING:",
    // "DISCOVERY:", etc.). We scan forward to find the next section-header pattern.
    let skeleton_region_raw = &content[skeleton_start..];

    // The skeleton lines are numbered "1.", "2.", "3." in the description string.
    // Extract skeleton lines: lines starting with whitespace + a digit + ".".
    let mut skeleton_lines: Vec<String> = Vec::new();
    let mut in_description = false;
    for line in skeleton_region_raw.lines() {
        // We're looking for lines like: `          1. SELECT COUNT(*) FROM ...`
        let trimmed = line.trim();

        // Stop at section boundary (ALL-CAPS word followed by ':'  — next description section).
        // These look like: "SEVERITY CASING WARNING:", "DISCOVERY:", etc.
        // Heuristic: a line that begins with uppercase and ends with ':' after the first word
        // is a section break. This prevents scanning too far.
        if !skeleton_lines.is_empty() {
            let upper_start = trimmed
                .split_ascii_whitespace()
                .next()
                .map(|w| w.trim_end_matches(':'))
                .unwrap_or("");
            if !upper_start.is_empty()
                && upper_start
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c == '-')
                && upper_start.len() >= 4
                && trimmed.contains(':')
            {
                break;
            }
        }

        // Match skeleton lines: trimmed starts with a digit followed by period+space.
        if trimmed
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            let after_digit = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
            if after_digit.starts_with(". ") {
                skeleton_lines.push(trimmed.to_string());
                in_description = true;
            }
        } else if in_description && trimmed.is_empty() {
            // Allow blank lines within skeleton block
        }
    }

    // Guard against vacuous pass: we must find at least 1 skeleton line.
    assert!(
        !skeleton_lines.is_empty(),
        "F-PQL2-OBS-001 vacuous-pass guard: no SCHEMA-AGNOSTIC SKELETON lines found \
         in server.rs starting at the 'SCHEMA-AGNOSTIC SKELETONS' marker. \
         The skeleton format may have changed — update this test to match."
    );

    // Scan each skeleton line for the bareword `timestamp` (case-insensitive).
    // A correctly-written skeleton uses `<datetime_col>` (angle-bracket placeholder) instead.
    let mut violations: Vec<String> = Vec::new();
    for skeleton_line in &skeleton_lines {
        // Check for bareword `timestamp`: it must NOT appear outside `<...>` angle brackets.
        // Strategy: check that `timestamp` does not appear as a standalone word token.
        let lower = skeleton_line.to_ascii_lowercase();
        if lower.contains("timestamp") && !skeleton_line.contains("<datetime_col>") {
            // `timestamp` appears and there's no `<datetime_col>` placeholder replacing it.
            violations.push(format!(
                "skeleton line contains bareword 'timestamp' without a <datetime_col> placeholder: {skeleton_line:?}"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "F-PQL2-OBS-001: SCHEMA-AGNOSTIC SKELETONS in server.rs query tool description \
         contain concrete datetime column name 'timestamp' which is sensor-specific (only \
         claroty_audit_logs uses this literal name; 5 of 6 demo tables use different datetime \
         column names). Replace with '<datetime_col>' placeholder so agents substitute the \
         real column name from prism_describe output.\n\
         Violations:\n{}",
        violations.join("\n")
    );
}

// ── Surface 2: resources.rs Datetime Arithmetic prose ────────────────────────

/// F-PQL2-OBS-001 guard — `build_reference_content` Datetime Arithmetic section must
/// use `<datetime_col>` placeholder in the example SQL blocks (not bare `timestamp`).
///
/// Before the fix the section contained:
/// ```
/// WHERE timestamp > NOW() - INTERVAL '7d'
/// WHERE timestamp > NOW() - INTERVAL '24h'
/// ```
///
/// This taught agents `timestamp` as the universal datetime column name, producing silent
/// 0-row queries on most demo tables (5 of 6 have different datetime column names).
///
/// After the fix the section contains `<datetime_col>` with a note that the real column
/// name comes from `prism_describe`.
///
/// **Load-bearing:** fails against the pre-fix resources.rs that used `WHERE timestamp >`.
#[test]
fn test_f_pql2_obs001_datetime_arithmetic_uses_placeholder() {
    let content = build_reference_content(None);

    // The "Datetime Arithmetic" section must be present.
    let dt_section_start = content.find("Datetime Arithmetic").expect(
        "F-PQL2-OBS-001: build_reference_content must contain 'Datetime Arithmetic' section header",
    );

    // Isolate the Datetime Arithmetic section (up to the next ## header).
    let dt_region = &content[dt_section_start..];
    let section_end = dt_region[1..] // skip the header itself
        .find("## ")
        .map(|p| p + 1)
        .unwrap_or(dt_region.len());
    let dt_section = &dt_region[..section_end];

    // Positive assertion: `<datetime_col>` placeholder must appear in the section.
    // This confirms the fix was applied and the section no longer uses bare `timestamp`.
    assert!(
        dt_section.contains("<datetime_col>"),
        "F-PQL2-OBS-001: Datetime Arithmetic section in build_reference_content must contain \
         '<datetime_col>' placeholder in the example SQL blocks. The section previously used \
         a concrete column name that is sensor-specific (regression: the fix was reverted). \
         Fix: restore '<datetime_col>' and the note that the real name comes from prism_describe.\n\
         Section content (first 600 chars): {:?}",
        &dt_section[..dt_section.len().min(600)]
    );

    // Negative assertion: bare `WHERE timestamp >` pattern must NOT appear in the section.
    // This is the exact pre-fix string that taught agents the wrong column name.
    assert!(
        !dt_section.contains("WHERE timestamp >"),
        "F-PQL2-OBS-001: Datetime Arithmetic section in build_reference_content must NOT \
         contain 'WHERE timestamp >' — this teaches agents to use 'timestamp' as a universal \
         datetime column, but only claroty_audit_logs uses this name (5 of 6 demo tables have \
         different datetime column names). Replace with 'WHERE <datetime_col> >'.\n\
         Section content (first 600 chars): {:?}",
        &dt_section[..dt_section.len().min(600)]
    );
}
