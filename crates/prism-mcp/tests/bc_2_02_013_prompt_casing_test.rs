//! Red Gate test for S-PRISMQL-CASE-INSENSITIVE-001 F-P9-MED-3.
//!
//! Finding F-P9-MED-3: the `render_triage_alerts` prompt (and any other rendered
//! prompt) must not contain case-sensitive equality fragments using ALL-CAPS OCSF
//! enum literals (e.g., `severity IN ('HIGH', 'CRITICAL')`).
//!
//! After OCSF Title-case normalization (BC-2.02.013), sensor adapters convert
//! `'HIGH'` → `'High'` before DataFusion materialization.  A prompt that teaches
//! agents to write `severity IN ('HIGH', 'CRITICAL')` will silently produce 0 rows
//! against live (or DTU) data.
//!
//! # UNHANDLED status adjudication
//!
//! The armis prompt leg contains `status = 'UNHANDLED'`.  Investigation of
//! `crates/prism-ocsf/src/enum_map.rs` confirms that "UNHANDLED" is NOT a
//! recognized OCSF status_id caption — OCSF status values are:
//!   Unknown / New / In Progress / Suppressed / Resolved / Archived / Deleted /
//!   Success / Failure / Other.
//!
//! The armis DTU generator (`crates/prism-dtu-armis/src/generator.rs` line ~789)
//! emits `"status": "UNHANDLED"` as a vendor-specific literal.  Because it does
//! NOT match any OCSF enum caption, the normalizer returns `None` and the raw
//! vendor value passes through unchanged.  Therefore `status = 'UNHANDLED'` IS
//! correct for armis data and must NOT be flagged by this test.
//!
//! Only the severity leg (`IN ('HIGH', 'CRITICAL')`) is stale.
//!
//! # Test → BC mapping
//!
//! | Test | BC | Finding |
//! |------|----|---------|
//! | test_BC_2_02_013_triage_alerts_prompt_no_stale_vendor_casing | BC-2.02.013 | F-P9-MED-3 |

use prism_mcp::prompts::{
    render_client_overview, render_cross_client_status, render_investigate_host,
    render_query_tutorial, render_triage_alerts,
};

/// Extract all text content from a `GetPromptResult` into a single string.
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

/// F-P9-MED-3 / BC-2.02.013
///
/// Every `render_*` prompt function must use OCSF Title-case enum literals
/// (e.g., `severity IN ('High', 'Critical')`) or case-insensitive operators,
/// NOT vendor-cased ALL-CAPS literals (e.g., `IN ('HIGH', 'CRITICAL')`).
///
/// # UNHANDLED status exception
///
/// `status = 'UNHANDLED'` is a vendor-specific armis value that is NOT in the
/// OCSF status_id enum map.  The normalizer passes it through unchanged.
/// Therefore `status = 'UNHANDLED'` is CORRECT and is NOT flagged here.
/// See module-level doc for the full adjudication.
///
/// # Red Gate
///
/// At HEAD 0b2c0983, `render_triage_alerts` armis leg reads:
///   `severity IN ('HIGH', 'CRITICAL') AND status = 'UNHANDLED'`
///
/// This test fails because `IN ('HIGH'` is found in the rendered text.
///
/// # Fix target
///
/// Change the armis severity leg from `IN ('HIGH', 'CRITICAL')` to
/// `IN ('High', 'Critical')` (matching the already-correct crowdstrike leg).
#[test]
fn test_BC_2_02_013_triage_alerts_prompt_no_stale_vendor_casing() {
    // Render all prompt functions so we catch any stale pattern across the whole
    // prompts module, not just the one known-bad function.
    let triage_text = extract_text(
        &render_triage_alerts("acme")
            .expect("render_triage_alerts must not fail for valid client_id 'acme'"),
    );
    let investigate_text = extract_text(
        &render_investigate_host("acme", "10.0.0.1")
            .expect("render_investigate_host must not fail for valid inputs"),
    );
    let overview_text = extract_text(
        &render_client_overview("acme")
            .expect("render_client_overview must not fail for valid client_id 'acme'"),
    );
    let cross_text = extract_text(
        &render_cross_client_status(None)
            .expect("render_cross_client_status must not fail with no time_range"),
    );
    let tutorial_text = extract_text(
        &render_query_tutorial("acme", None)
            .expect("render_query_tutorial must not fail for valid inputs"),
    );

    // Forbidden patterns: ALL-CAPS or all-lowercase OCSF-enum-label literals in a
    // case-sensitive equality / IN-list context.  These silently produce 0 rows
    // after OCSF Title-case normalization (BC-2.02.013).
    //
    // Intentionally excluded: `= 'UNHANDLED'`
    //   — `UNHANDLED` is a vendor-specific armis status value, not an OCSF enum
    //     caption; the normalizer returns None and the raw value passes through.
    //     Querying `status = 'UNHANDLED'` against armis data is therefore correct.
    let forbidden: &[&str] = &[
        "= 'HIGH'",
        "= 'high'",
        "IN ('HIGH'",
        "IN ('high'",
        "= 'CRITICAL'",
        "= 'critical'",
    ];

    let all_prompts: &[(&str, &str)] = &[
        ("render_triage_alerts", &triage_text),
        ("render_investigate_host", &investigate_text),
        ("render_client_overview", &overview_text),
        ("render_cross_client_status", &cross_text),
        ("render_query_tutorial", &tutorial_text),
    ];

    let mut violations: Vec<String> = Vec::new();
    for (fn_name, text) in all_prompts {
        for pattern in forbidden {
            if text.contains(pattern) {
                violations.push(format!("{fn_name}: found forbidden pattern {pattern:?}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "BC-2.02.013 F-P9-MED-3: prompt function(s) contain stale vendor-cased \
         enum equality patterns that silently produce 0 rows after OCSF Title-case \
         normalization. Violations found:\n{}\n\n\
         Replace with Title-case forms (`= 'High'`, `IN ('High', 'Critical')`) or \
         case-insensitive operators (`IEQ`, `IIN`).\n\
         Note: `status = 'UNHANDLED'` is exempt (vendor-specific armis value, \
         not in OCSF status_id enum map, passes through un-normalized).",
        violations.join("\n"),
    );
}
