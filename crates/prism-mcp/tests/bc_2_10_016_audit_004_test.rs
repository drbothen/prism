//! Red Gate test for S-DEMO-FIDELITY-REMEDIATION-001 AC-AUDIT-004 — BC-2.10.016 v1.2.
//!
//! Finding AUDIT-004: All five `render_*` prompt functions in `prism-mcp/src/prompts.rs`
//! contain SQL examples that use dot-notation FROM references (`FROM crowdstrike.alerts`,
//! `FROM claroty.assets`, etc.). These must be replaced with underscore-qualified names
//! (`FROM crowdstrike_alerts`, `FROM claroty_assets`, etc.) — matching the valid PrismQL
//! FROM syntax.
//!
//! # Dot-notation instances found in current code (source of this test's assertions)
//!
//! - `render_triage_alerts`: `FROM crowdstrike.alerts`, `FROM claroty.alerts`, `FROM armis.alerts`
//! - `render_investigate_host`: `FROM crowdstrike.devices`, `FROM claroty.assets`, `FROM armis.devices`
//! - `render_client_overview`: `FROM crowdstrike.alerts`, `FROM claroty.alerts`
//! - `render_cross_client_status`: `FROM crowdstrike.alerts`
//! - `render_query_tutorial`: no dot-notation FROM (CLEAN — this test guards it stays clean)
//!
//! # Test strategy
//!
//! Call each `render_*` function with valid inputs, then scan the returned body text for
//! `FROM sensor.table` dot-notation patterns. Uses a manual token scan (no regex dep):
//! split by whitespace, find "FROM" tokens, check the next token for a `.` that is NOT
//! a URL scheme (i.e., not `://`). Assert zero violations.
//!
//! RED GATE: Current code has 7 dot-notation FROM instances across 4 functions.
//! The test counts them and asserts zero. It fails RED listing all occurrences found.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_10_016_audit_004_no_dot_notation_in_prompts | AUDIT-004 | BC-2.10.016 v1.2 |

use prism_mcp::prompts::{
    render_client_overview, render_cross_client_status, render_investigate_host,
    render_query_tutorial, render_triage_alerts,
};

// ── Tests ─────────────────────────────────────────────────────────────────────

/// BC-2.10.016 v1.2 AUDIT-004 — Red Gate test.
///
/// Every `render_*` prompt function must use underscore-qualified FROM targets
/// (e.g., `FROM crowdstrike_alerts`) NOT dot-notation (e.g., `FROM crowdstrike.alerts`).
///
/// Dot-notation FROM references cause AI agents to emit invalid PrismQL queries
/// that produce silent E-SENSOR-030 partial failures (0 rows, no error surface).
///
/// The fix: replace every `FROM sensor.table` literal in `prompts.rs` with
/// `FROM sensor_table`.
///
/// # Red Gate failure
///
/// Current code has 7 dot-notation FROM instances:
/// - render_triage_alerts (3): crowdstrike.alerts, claroty.alerts, armis.alerts
/// - render_investigate_host (3): crowdstrike.devices, claroty.assets, armis.devices
/// - render_client_overview (2): crowdstrike.alerts, claroty.alerts
/// - render_cross_client_status (1): crowdstrike.alerts
///
/// This test asserts zero occurrences and FAILS RED because > 0 are found.
#[test]
fn test_bc_2_10_016_audit_004_no_dot_notation_in_prompts() {
    // ── Collect all prompt outputs ────────────────────────────────────────────

    // render_triage_alerts("acme")
    let triage = render_triage_alerts("acme")
        .expect("render_triage_alerts must not fail for valid client_id 'acme'");
    let triage_text = extract_text(&triage);

    // render_investigate_host("acme", "10.0.0.1")
    let investigate = render_investigate_host("acme", "10.0.0.1")
        .expect("render_investigate_host must not fail for valid inputs");
    let investigate_text = extract_text(&investigate);

    // render_client_overview("acme")
    let overview = render_client_overview("acme")
        .expect("render_client_overview must not fail for valid client_id 'acme'");
    let overview_text = extract_text(&overview);

    // render_cross_client_status(None)
    let cross = render_cross_client_status(None)
        .expect("render_cross_client_status must not fail with no time_range");
    let cross_text = extract_text(&cross);

    // render_query_tutorial("acme", None)
    let tutorial = render_query_tutorial("acme", None)
        .expect("render_query_tutorial must not fail for valid inputs");
    let tutorial_text = extract_text(&tutorial);

    // ── Scan each function's output for dot-notation FROM references ──────────

    let mut all_violations: Vec<String> = Vec::new();

    scan_for_violations(&triage_text, "render_triage_alerts", &mut all_violations);
    scan_for_violations(
        &investigate_text,
        "render_investigate_host",
        &mut all_violations,
    );
    scan_for_violations(
        &overview_text,
        "render_client_overview",
        &mut all_violations,
    );
    scan_for_violations(
        &cross_text,
        "render_cross_client_status",
        &mut all_violations,
    );
    scan_for_violations(&tutorial_text, "render_query_tutorial", &mut all_violations);

    // ── Assert zero violations ────────────────────────────────────────────────

    assert!(
        all_violations.is_empty(),
        "BC-2.10.016 AUDIT-004 RED GATE: found {} dot-notation FROM reference(s) \
         across render_* prompt functions. All must be replaced with underscore-qualified \
         names (e.g., FROM crowdstrike_alerts). \
         Violations found:\n{}",
        all_violations.len(),
        all_violations.join("\n")
    );
}

// ── Helper functions ──────────────────────────────────────────────────────────

/// Extract the text content from a `GetPromptResult`.
///
/// Concatenates text from all `PromptMessage` entries in the result.
fn extract_text(result: &rmcp::model::GetPromptResult) -> String {
    result
        .messages
        .iter()
        .filter_map(|msg| match &msg.content {
            rmcp::model::PromptMessageContent::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Scan `text` for `FROM sensor.table` dot-notation patterns without the `regex` crate.
///
/// Algorithm: split the text into whitespace-delimited tokens, find "FROM" tokens (case
/// sensitive — PrismQL uses uppercase FROM in examples), then check the next token.
/// If the next token contains a `.` and the `.` is NOT part of a `://` URL scheme
/// (e.g., `prismql://schema/...` is NOT a FROM target), record as a violation.
///
/// The sensor names in prompts are always lowercase (`crowdstrike`, `claroty`, etc.),
/// so we check that the token looks like `word.word` (no slash after the dot).
fn scan_for_violations(text: &str, fn_name: &str, violations: &mut Vec<String>) {
    // Tokenize by splitting on ASCII whitespace (spaces, newlines, tabs).
    let tokens: Vec<&str> = text.split_ascii_whitespace().collect();

    for (i, token) in tokens.iter().enumerate() {
        // Look for "FROM" keyword (SQL convention: uppercase in prompt examples).
        if *token == "FROM" {
            if let Some(next) = tokens.get(i + 1) {
                // Strip trailing punctuation (comma, semicolon, newline) from the token.
                let clean = next.trim_end_matches(|c: char| c == ',' || c == ';' || c == ')');

                // Check for dot-notation: must contain '.' and NOT contain '://'
                // (to exclude URL references like prismql://reference).
                if clean.contains('.') && !clean.contains("://") {
                    violations.push(format!(
                        "  [{fn_name}] 'FROM {clean}' — must be 'FROM {}'",
                        clean.replacen('.', "_", 1)
                    ));
                }
            }
        }
    }
}
