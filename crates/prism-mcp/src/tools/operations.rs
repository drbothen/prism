//! Operations tools: schedule, detection, and case management (BC-2.13.*).
//!
//! These tools depend on `prism-operations` (SS-12/13/14) which is not yet merged.
//! Per POL-12 and AC-10 of S-5.01-FOLLOWUP-MCP-BOOT, these handlers return a
//! structured `Err` with code -32003 ("Feature not yet available: <feature>").
//! They MUST NOT panic — structured NotImplemented is the contract (EC-005).
//!
//! EC-005: "Tool for prism-operations feature invoked before prism-operations merges
//! → Returns code -32003, 'Feature not yet available: schedule management' — NOT panic."

use std::io::{Error, ErrorKind};

/// Structured "not yet available" error for prism-operations tools.
///
/// Returns `Err` with the feature description for structured -32003 responses.
/// Callers at the MCP layer map this to `McpError::custom(-32003, msg)`.
fn not_yet_available(feature: &str) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(Error::new(
        ErrorKind::Unsupported,
        format!("Feature not yet available: {feature}"),
    ))
}

// ─── Schedule management tools ────────────────────────────────────────────────

/// Create a recurring PrismQL query schedule.
///
/// Entry point for `create_schedule` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_create_schedule() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("schedule management"))
}

/// List all PrismQL query schedules for the calling client.
///
/// Entry point for `list_schedules` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_list_schedules() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("schedule management"))
}

/// Delete a PrismQL query schedule by ID.
///
/// Entry point for `delete_schedule` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_delete_schedule() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("schedule management"))
}

/// Retrieve diff results from the most recent schedule run.
///
/// Entry point for `get_diff_results` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_get_diff_results() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("schedule management"))
}

// ─── Detection rule tools ─────────────────────────────────────────────────────

/// Create a detection rule from a PrismQL query.
///
/// Entry point for `create_rule` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_create_rule() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("detection rules"))
}

/// List all detection rules for the calling client.
///
/// Entry point for `list_rules` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_list_rules() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("detection rules"))
}

/// Delete a detection rule by ID.
///
/// Entry point for `delete_rule` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_delete_rule() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("detection rules"))
}

// ─── Case management tools ────────────────────────────────────────────────────

/// Create a new security case.
///
/// Entry point for `create_case` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_create_case() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("case management"))
}

/// List security cases for the calling client.
///
/// Entry point for `list_cases` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_list_cases() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("case management"))
}

/// Get a specific security case by ID.
///
/// Entry point for `get_case` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_get_case() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("case management"))
}

/// Update fields on an existing security case.
///
/// Entry point for `update_case` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_update_case() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("case management"))
}

/// Retrieve aggregated metrics across security cases.
///
/// Entry point for `case_metrics` MCP tool.
/// Returns structured NotImplemented until prism-operations merges (EC-005).
pub async fn tool_case_metrics() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_available("case management"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify `not_yet_available` produces a structured error message (not a panic).
    ///
    /// EC-005: tools must return structured error, NOT panic.
    /// Tests the shared error constructor used by every operations tool.
    #[test]
    fn test_not_yet_available_produces_structured_error() {
        let err = not_yet_available("schedule management");
        let msg = err.to_string();
        assert!(
            msg.contains("Feature not yet available"),
            "error must contain 'Feature not yet available'; got: '{msg}'"
        );
        assert!(
            msg.contains("schedule management"),
            "error must name the feature; got: '{msg}'"
        );
    }

    /// Verify every feature category has the correct "not yet available" message.
    #[test]
    fn test_not_yet_available_all_feature_categories() {
        for feature in &["schedule management", "detection rules", "case management"] {
            let err = not_yet_available(feature);
            let msg = err.to_string();
            assert!(
                msg.contains(feature),
                "error must name feature '{feature}'; got: '{msg}'"
            );
        }
    }
}
