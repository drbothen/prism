//! Operations tool handlers — implemented as PrismServer methods (BC-2.13.*).
//!
//! All operations tool handlers live on `PrismServer` in `crate::server` via the
//! `#[tool_router(server_handler)]` macro block:
//!
//! - Schedule management: `create_schedule`, `list_schedules`, `delete_schedule`, `get_diff_results`
//! - Detection rules: `create_rule`, `list_rules`, `delete_rule`
//! - Case management: `create_case`, `list_cases`, `get_case`, `update_case`, `case_metrics`
//!
//! These tools depend on `prism-operations` (SS-12/13/14) which is not yet merged.
//! Per POL-12 and AC-10 of S-5.01-FOLLOWUP-MCP-BOOT, these handlers return a
//! structured `Err` with "Feature not yet available: <feature>" (EC-005).
//! They MUST NOT panic — structured error is the contract.
//!
//! EC-005: "Tool for prism-operations feature invoked before prism-operations merges
//! → Returns structured error, 'Feature not yet available: schedule management' — NOT panic."

/// Structured "not yet available" error message for prism-operations tools.
///
/// Returns the standard error message string for tools that depend on
/// `prism-operations` (not yet merged). Tool handlers on `PrismServer` return
/// `Err(not_yet_available_msg("schedule management"))` per EC-005.
///
/// # Naming
///
/// Feature strings per EC-005:
/// - Schedule management: `"schedule management"`
/// - Detection rules: `"detection rules"`
/// - Case management: `"case management"`
pub fn not_yet_available_msg(feature: &str) -> String {
    format!("Feature not yet available: {feature}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// EC-005: `not_yet_available_msg` produces a structured message (not a panic).
    ///
    /// Tests the shared message builder used by every operations tool handler.
    #[test]
    fn test_not_yet_available_produces_structured_error() {
        let msg = not_yet_available_msg("schedule management");
        assert!(
            msg.contains("Feature not yet available"),
            "message must contain 'Feature not yet available'; got: '{msg}'"
        );
        assert!(
            msg.contains("schedule management"),
            "message must name the feature; got: '{msg}'"
        );
    }

    /// Verify every feature category has the correct "not yet available" message.
    #[test]
    fn test_not_yet_available_all_feature_categories() {
        for feature in &["schedule management", "detection rules", "case management"] {
            let msg = not_yet_available_msg(feature);
            assert!(
                msg.contains(feature),
                "message must name feature '{feature}'; got: '{msg}'"
            );
        }
    }
}
