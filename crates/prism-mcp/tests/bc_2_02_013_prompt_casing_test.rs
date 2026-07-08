//! Red Gate test for S-PRISMQL-CASE-INSENSITIVE-001 F-MED-02 / F-P9-MED-3.
//!
//! **F-P9-MED-3 (original):** prompt functions must not contain case-sensitive equality
//! fragments using ALL-CAPS OCSF severity literals (e.g., `severity IN ('HIGH', 'CRITICAL')`).
//!
//! **F-MED-02 (pass-19 extension):** forbidden-pattern guards are now derived from the
//! AUTHORITATIVE OCSF v1.7.0 enum caption set covering ALL FOUR OCSF enum-label fields:
//! `severity`, `status`, `activity_name`, `disposition`.  The previous severity-only list
//! left `status = 'new'` (lowercase) uncaught in the crowdstrike prompt legs.
//!
//! After OCSF Title-case normalization (BC-2.02.013), sensor adapters convert
//! `'HIGH'` → `'High'` and `'new'` → `'New'` before DataFusion materialization.
//! A prompt that teaches agents to write `status = 'new'` will silently produce 0 rows
//! against live (or DTU) data where the crowdstrike adapter normalizes status to 'New'.
//!
//! # Pattern generation
//!
//! `forbidden_ocsf_casing_patterns()` derives all forbidden patterns from
//! `prism_ocsf::OcsfEnumMap` captions.  For each Title-case caption C it generates:
//!   - `= '{lower}'`     — all-lowercase equality (wrong casing)
//!   - `= '{UPPER}'`     — all-uppercase equality (wrong casing)
//!   - ` IN ('{lower}'`  — case-sensitive IN first-position, lowercase (leading-space
//!                         guard prevents false-positive on IIN operator)
//!   - ` IN ('{UPPER}'`  — same, uppercase
//!
//! Title-case forms (`= 'High'`, `IN ('New'`) and case-insensitive operators
//! (`IEQ`, `IIN`, `INE`) remain ALLOWED.
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
//! # Test → BC mapping
//!
//! | Test | BC | Finding |
//! |------|----|---------|
//! | test_BC_2_02_013_triage_alerts_prompt_no_stale_vendor_casing | BC-2.02.013 | F-P9-MED-3 / F-MED-02 |

use prism_mcp::prompts::{
    render_client_overview, render_cross_client_status, render_investigate_host,
    render_query_tutorial, render_triage_alerts,
};

/// Generate all forbidden wrong-casing patterns from the authoritative OCSF v1.7.0
/// enum caption set (BC-2.02.013 / F-MED-02).
///
/// For each Title-case caption C registered in `prism_ocsf::OcsfEnumMap::new()`,
/// generates four patterns:
/// - `= '{lower}'`      — all-lowercase equality (wrong casing; silently 0 rows after normalization)
/// - `= '{UPPER}'`      — all-uppercase equality (wrong casing)
/// - ` IN ('{lower}'`   — case-sensitive IN, first-position, lowercase
///                        (leading space guards against `IIN (` false-positive)
/// - ` IN ('{UPPER}'`   — same, uppercase
///
/// Title-case forms (e.g., `= 'High'`, `= 'New'`) and case-insensitive operators
/// (`IEQ`, `IIN`, `INE`) remain ALLOWED and are not generated here.
///
/// Exception: `UNHANDLED` is NOT an OCSF caption (vendor-specific Armis value;
/// normalizer returns None → passes through). It is excluded from generation.
/// Exception: `OPEN` is similarly excluded — not an OCSF caption.
fn forbidden_ocsf_casing_patterns() -> Vec<String> {
    // Authoritative caption set from prism_ocsf::OcsfEnumMap::new().
    // Four OCSF enum-label fields: severity, activity_name, status, disposition.
    // Shared captions ("Unknown", "Other", "Deleted") listed once — duplicates are harmless.
    const CAPTIONS: &[&str] = &[
        // severity_id (severity field)
        "Unknown",
        "Informational",
        "Low",
        "Medium",
        "High",
        "Critical",
        "Other",
        // activity_id (activity_name field — OCSF exception: sibling is activity_id)
        "Create",
        "Read",
        "Update",
        "Delete",
        // status_id — finding-class subset (synthetic keys 1001–1006 in OcsfEnumMap)
        "New",
        "In Progress",
        "Suppressed",
        "Resolved",
        "Archived",
        "Deleted",
        // status_id — generic subset (Unknown / Other already above)
        "Success",
        "Failure",
        // disposition_id (29 values from OCSF v1.7.0 dictionary_attributes)
        "Allowed",
        "Blocked",
        "Quarantined",
        "Isolated",
        "Dropped",
        "Custom Action",
        "Approved",
        "Restored",
        "Exonerated",
        "Corrected",
        "Partially Corrected",
        "Uncorrected",
        "Delayed",
        "Detected",
        "No Action",
        "Logged",
        "Tagged",
        "Alert",
        "Count",
        "Reset",
        "Captcha",
        "Challenge",
        "Access Revoked",
        "Rejected",
        "Unauthorized",
        "Error",
    ];

    let mut patterns = Vec::new();
    for &cap in CAPTIONS {
        let lower = cap.to_lowercase();
        let upper = cap.to_uppercase();
        // Equality patterns (case-sensitive = operator with wrong casing)
        patterns.push(format!("= '{lower}'"));
        patterns.push(format!("= '{upper}'"));
        // IN first-position patterns (leading space distinguishes SQL `IN` from `IIN` operator)
        patterns.push(format!(" IN ('{lower}'"));
        patterns.push(format!(" IN ('{upper}'"));
    }
    patterns
}

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

/// F-MED-02 / F-P9-MED-3 / BC-2.02.013
///
/// Every `render_*` prompt function must use OCSF Title-case enum literals
/// (e.g., `severity IN ('High', 'Critical')`, `status = 'New'`) or
/// case-insensitive operators (`IEQ`, `IIN`, `INE`), NOT wrong-cased literals
/// (e.g., `IN ('HIGH', 'CRITICAL')`, `status = 'new'`).
///
/// Forbidden patterns are derived from the full OCSF v1.7.0 enum caption set
/// via `forbidden_ocsf_casing_patterns()` (F-MED-02 pass-19 extension).
/// Four fields covered: severity, activity_name, status, disposition.
///
/// # UNHANDLED status exception
///
/// `status = 'UNHANDLED'` is a vendor-specific armis value that is NOT in the
/// OCSF status_id enum map.  The normalizer passes it through unchanged.
/// Therefore `status = 'UNHANDLED'` is CORRECT and is NOT flagged here.
/// See module-level doc for the full adjudication.
///
/// # Red Gate (pass-19)
///
/// At HEAD e8b25d67, `render_triage_alerts`, `render_client_overview`, and
/// `render_cross_client_status` all contain `status = 'new'` (lowercase) in their
/// crowdstrike legs.  Post-normalization crowdstrike status is `'New'` (Title-case).
/// This test goes RED because `= 'new'` is now in the generated forbidden list.
///
/// # Fix target (pass-19)
///
/// Change all three crowdstrike prompt legs from `status = 'new'` to `status = 'New'`.
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

    // Forbidden patterns: wrong-cased (all-lowercase or all-uppercase) OCSF enum-label
    // literals in a case-sensitive equality / IN-list context.  These silently produce
    // 0 rows after OCSF Title-case normalization (BC-2.02.013).
    //
    // Derived from all four OCSF enum-label fields via forbidden_ocsf_casing_patterns()
    // (F-MED-02 pass-19 extension).
    //
    // Intentionally excluded: `= 'UNHANDLED'`
    //   — `UNHANDLED` is a vendor-specific armis status value, not an OCSF enum
    //     caption; the normalizer returns None and the raw value passes through.
    //     Querying `status = 'UNHANDLED'` against armis data is therefore correct.
    let forbidden = forbidden_ocsf_casing_patterns();

    let all_prompts: &[(&str, &str)] = &[
        ("render_triage_alerts", &triage_text),
        ("render_investigate_host", &investigate_text),
        ("render_client_overview", &overview_text),
        ("render_cross_client_status", &cross_text),
        ("render_query_tutorial", &tutorial_text),
    ];

    let mut violations: Vec<String> = Vec::new();
    for (fn_name, text) in all_prompts {
        for pattern in &forbidden {
            if text.contains(pattern.as_str()) {
                violations.push(format!("{fn_name}: found forbidden pattern {pattern:?}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "BC-2.02.013 F-MED-02/F-P9-MED-3: prompt function(s) contain wrong-cased \
         OCSF enum-label literals that silently produce 0 rows after BC-2.02.013 \
         Title-case normalization. Violations found:\n{}\n\n\
         Replace with Title-case forms (`= 'High'`, `= 'New'`, `IN ('High', 'Critical')`) \
         or case-insensitive operators (`IEQ`, `IIN`, `INE`).\n\
         Note: `status = 'UNHANDLED'` is exempt (vendor-specific armis value, \
         not in OCSF status_id enum map, passes through un-normalized).",
        violations.join("\n"),
    );
}
