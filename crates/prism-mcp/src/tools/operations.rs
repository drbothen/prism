//! Operations tools: schedule, detection, and case management (BC-2.13.*).
//!
//! These tools depend on `prism-operations` (SS-12/13/14) which is not yet merged.
//! Per POL-12 and AC-10 of S-5.01-FOLLOWUP-MCP-BOOT, these handlers MUST return
//! a structured `McpError::custom(-32003, "Feature not yet available: <feature>")`.
//! They MUST NOT panic or call `todo!()` — structured NotImplemented is the contract.
//!
//! EC-005: "Tool for prism-operations feature invoked before prism-operations merges
//! → Returns McpError::custom(-32003, 'Feature not yet available: schedule management')
//! — NOT panic."
//!
//! These stubs use `todo!()` as placeholders until the implementer replaces them with
//! the real structured NotImplemented response at TDD time. The implementer MUST replace
//! every `todo!()` in this file with the structured error — no `todo!()` may remain
//! before merge (AC-10).

// Schedule management tools

/// Create a recurring PrismQL query schedule.
///
/// Entry point for `create_schedule` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_create_schedule() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_create_schedule — replace todo! with McpError::custom(-32003, \"Feature not yet available: schedule management\")")
}

/// List all PrismQL query schedules for the calling client.
///
/// Entry point for `list_schedules` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_list_schedules() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_list_schedules — replace todo! with McpError::custom(-32003, \"Feature not yet available: schedule management\")")
}

/// Delete a PrismQL query schedule by ID.
///
/// Entry point for `delete_schedule` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_delete_schedule() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_delete_schedule — replace todo! with McpError::custom(-32003, \"Feature not yet available: schedule management\")")
}

/// Retrieve diff results from the most recent schedule run.
///
/// Entry point for `get_diff_results` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_get_diff_results() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_get_diff_results — replace todo! with McpError::custom(-32003, \"Feature not yet available: schedule management\")")
}

// Detection rule tools

/// Create a detection rule from a PrismQL query.
///
/// Entry point for `create_rule` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_create_rule() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_create_rule — replace todo! with McpError::custom(-32003, \"Feature not yet available: detection rules\")")
}

/// List all detection rules for the calling client.
///
/// Entry point for `list_rules` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_list_rules() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_list_rules — replace todo! with McpError::custom(-32003, \"Feature not yet available: detection rules\")")
}

/// Delete a detection rule by ID.
///
/// Entry point for `delete_rule` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_delete_rule() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_delete_rule — replace todo! with McpError::custom(-32003, \"Feature not yet available: detection rules\")")
}

// Case management tools

/// Create a new security case.
///
/// Entry point for `create_case` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_create_case() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_create_case — replace todo! with McpError::custom(-32003, \"Feature not yet available: case management\")")
}

/// List security cases for the calling client.
///
/// Entry point for `list_cases` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_list_cases() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_list_cases — replace todo! with McpError::custom(-32003, \"Feature not yet available: case management\")")
}

/// Get a specific security case by ID.
///
/// Entry point for `get_case` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_get_case() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_get_case — replace todo! with McpError::custom(-32003, \"Feature not yet available: case management\")")
}

/// Update fields on an existing security case.
///
/// Entry point for `update_case` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_update_case() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_update_case — replace todo! with McpError::custom(-32003, \"Feature not yet available: case management\")")
}

/// Retrieve aggregated metrics across security cases.
///
/// Entry point for `case_metrics` MCP tool.
/// Returns structured NotImplemented until prism-operations merges.
pub async fn tool_case_metrics() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_case_metrics — replace todo! with McpError::custom(-32003, \"Feature not yet available: case management\")")
}
