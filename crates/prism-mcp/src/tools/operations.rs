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
//!
//! MED-1 fix: the `not_yet_available_msg` function that returned a plain `String` was
//! dead code (no callers in this crate). The canonical `not_yet_available_msg` that
//! returns `rmcp::model::ErrorData` lives in `crate::server` — that is the one used
//! by all tool handlers. This module is retained as a doc anchor for the operations
//! tool category.
