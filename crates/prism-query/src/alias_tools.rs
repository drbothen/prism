//! MCP tool handlers for the alias system.
//!
//! Implements the four alias management tools:
//! - `create_alias`  (BC-2.11.008)
//! - `list_aliases`  (BC-2.11.013)
//! - `delete_alias`  (BC-2.11.014)
//! - `explain_alias` (BC-2.11.015)
//!
//! All write operations (`create_alias`, `delete_alias`) require the
//! `alias.write` capability and issue/consume `ConfirmationToken`s.
//! Read operations (`list_aliases`, `explain_alias`) do not require the
//! capability and do not issue tokens.
//!
//! Each operation emits an audit entry (DI-004).
//!
//! Story: S-3.04 — prism-query: Alias System (P1)
//! BCs:   BC-2.11.008, BC-2.11.013, BC-2.11.014, BC-2.11.015

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use prism_core::error::PrismError;

use crate::alias_store::AliasStore;
use crate::alias_types::AliasEntry;

// ─────────────────────────────────────────────────────────────────────────────
// Input / Output types
// ─────────────────────────────────────────────────────────────────────────────

/// Input parameters for the `create_alias` MCP tool (BC-2.11.008).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAliasInput {
    /// Alias name — must match `[a-zA-Z_][a-zA-Z0-9_]*`, 1–64 chars.
    pub name: String,
    /// Scope string: `"global"` or `"client:<client_id>"`.
    pub scope: String,
    /// The PrismQL expression or template string.
    pub query: String,
    /// Optional parameter map: parameter name → default value string.
    pub parameters: Option<HashMap<String, String>>,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Successful response from `create_alias` when a new alias is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAliasSuccess {
    /// The created alias definition.
    pub alias: AliasEntryView,
    /// Fully expanded form of the alias (at depth 0, no `@` references).
    pub expanded: String,
}

/// Input parameters for the `list_aliases` MCP tool (BC-2.11.013).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAliasesInput {
    /// Optional scope filter: `"global"`, `"client:<id>"`, or `null` for all.
    pub scope: Option<String>,
}

/// Input parameters for the `delete_alias` MCP tool (BC-2.11.014).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAliasInput {
    /// Alias name to delete.
    pub name: String,
    /// Scope: `"global"` or `"client:<client_id>"`.
    pub scope: String,
    /// When `true`, cascade-delete all dependent aliases.
    #[serde(default)]
    pub force: bool,
    /// Confirmation token from a prior call (when completing a two-step delete).
    pub token_id: Option<String>,
}

/// Successful response from `delete_alias`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAliasSuccess {
    /// Name of the primary alias that was deleted.
    pub deleted: String,
    /// Additional aliases cascade-deleted when `force: true`.
    pub cascade_deleted: Vec<String>,
}

/// Input parameters for the `explain_alias` MCP tool (BC-2.11.015).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainAliasInput {
    /// Alias name to explain.
    pub name: String,
    /// Optional scope; if absent, per-client overrides global.
    pub scope: Option<String>,
}

/// Response from `explain_alias` (BC-2.11.015).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainAliasResponse {
    /// Alias name.
    pub name: String,
    /// Resolved scope string.
    pub scope: String,
    /// Raw template string.
    pub query: String,
    /// Fully expanded query after recursive alias resolution.
    pub expanded: String,
    /// Parameter map with defaults (if parameterized).
    pub parameters: Option<HashMap<String, String>>,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Ordered list of alias names expanded during resolution.
    pub composition_chain: Vec<String>,
    /// Integer depth of the composition chain.
    pub composition_depth: usize,
}

/// JSON-serializable view of an alias entry (for list/create responses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasEntryView {
    pub name: String,
    pub scope: String,
    pub query: String,
    pub parameters: Option<HashMap<String, String>>,
    pub description: Option<String>,
}

impl AliasEntryView {
    /// Convert an `AliasEntry` to its view type for serialization.
    pub fn from_entry(_entry: &AliasEntry) -> Self {
        todo!()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool handler functions
// ─────────────────────────────────────────────────────────────────────────────

/// Handle the `create_alias` MCP tool invocation (BC-2.11.008).
///
/// # Steps
/// 1. Parse `input.scope` → `AliasScope`.
/// 2. Check `alias.write` capability via `alias_capability::check_alias_write`.
/// 3. Validate alias name format (`[a-zA-Z_][a-zA-Z0-9_]*`, 1–64 chars).
/// 4. Check keyword/OCSF collision → `E-ALIAS-006`.
/// 5. Validate parameter defaults as PrismQL atomic literals → `E-ALIAS-004`.
/// 6. Run cycle detection → `E-ALIAS-002`.
/// 7. Check composition depth → `E-ALIAS-003`.
/// 8. Validate parser accepts the template (placeholders treated as valid tokens).
/// 9. Call `AliasStore::create_or_update(entry, None)`.
/// 10. If `CreateResult::ConfirmationRequired`: return token response.
///     If `CreateResult::Created`: return success with expanded form.
///
/// Returns `Err` on any validation or I/O failure.
pub fn create_alias(
    _input: CreateAliasInput,
    _store: &mut AliasStore,
    _ocsf_reserved: &std::collections::HashSet<String>,
) -> Result<serde_json::Value, PrismError> {
    todo!()
}

/// Handle the `list_aliases` MCP tool invocation (BC-2.11.013).
///
/// # Steps
/// 1. Parse optional scope filter.
/// 2. If `scope` references a non-existent client: return `E-CFG-001`.
/// 3. Call `AliasStore::list(scope_filter)`.
/// 4. Sort alphabetically by name within each scope group.
/// 5. Emit audit entry (DI-004).
///
/// Returns `Ok(serde_json::Value)` — JSON array of `AliasEntryView` objects.
pub fn list_aliases(
    _input: ListAliasesInput,
    _store: &AliasStore,
    _valid_client_ids: &[String],
) -> Result<serde_json::Value, PrismError> {
    todo!()
}

/// Handle the `delete_alias` MCP tool invocation (BC-2.11.014).
///
/// Deletion ALWAYS requires a confirmation token (not only forced cascade).
///
/// # Steps
/// 1. Parse `input.scope` → `AliasScope`.
/// 2. Check `alias.write` capability.
/// 3. Check alias exists at scope; return `E-ALIAS-001` if absent.
/// 4. Validate `scope` client ID; return `E-CFG-001` if unknown.
/// 5. Resolve current dependents.
/// 6. Issue a `ConfirmationToken` with `action_summary` and optional
///    `dependent_aliases` warning field.
/// 7. On `confirm_action` path (when `input.token_id` is `Some`):
///    - Re-resolve dependents at confirmation time.
///    - Call `AliasStore::delete(name, scope, force, token)`.
///    - If dependents exist and `force: false`: return `E-ALIAS-005`.
/// 8. Emit audit entry (DI-004).
///
/// Returns confirmation token JSON on first call, success JSON after confirmation.
pub fn delete_alias(
    _input: DeleteAliasInput,
    _store: &mut AliasStore,
    _token_store: &prism_security::ConfirmationTokenStore,
    _valid_client_ids: &[String],
) -> Result<serde_json::Value, PrismError> {
    todo!()
}

/// Handle the `explain_alias` MCP tool invocation (BC-2.11.015).
///
/// # Steps
/// 1. Parse optional `input.scope`; if absent, use per-client-overrides-global
///    precedence.
/// 2. Look up alias; return `E-ALIAS-001` if absent.
/// 3. Run `AliasResolver::expand()` to produce the expanded form and composition
///    chain.
/// 4. Run parse validation on the expanded query.
/// 5. Emit audit entry (DI-004).
///
/// Returns `Ok(ExplainAliasResponse)` serialized as JSON.
pub fn explain_alias(
    _input: ExplainAliasInput,
    _store: &AliasStore,
    _client_scope: Option<&prism_core::types::ClientId>,
) -> Result<ExplainAliasResponse, PrismError> {
    todo!()
}

// ─────────────────────────────────────────────────────────────────────────────
// Keyword / OCSF collision validation
// ─────────────────────────────────────────────────────────────────────────────

/// PrismQL reserved keywords that must not be used as alias names.
///
/// Case-insensitive comparison at validation time (BC-2.11.008 invariants).
pub const PRISMQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "AND", "OR", "NOT", "LIMIT", "ORDER", "BY", "AS", "IN", "LIKE",
    "IS", "NULL", "TRUE", "FALSE", "BETWEEN", "GROUP", "HAVING", "JOIN", "ON", "DISTINCT", "UNION",
    "ALL", "WITH", "CASE", "WHEN", "THEN", "ELSE", "END",
];

/// Validate that `name` does not conflict with a PrismQL keyword or OCSF field.
///
/// Returns `Err(E-ALIAS-006)` with structured conflict information when a
/// collision is detected. Comparison is case-insensitive.
pub fn validate_no_keyword_collision(
    _name: &str,
    _ocsf_reserved: &std::collections::HashSet<String>,
) -> Result<(), PrismError> {
    todo!()
}

/// Validate that the alias name matches the required pattern.
///
/// Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`, 1–64 characters.
/// Returns `Err(PrismError::McpParameterInvalid)` on violation.
pub fn validate_alias_name(_name: &str) -> Result<(), PrismError> {
    todo!()
}
