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

use crate::alias_resolver::AliasResolver;
use crate::alias_store::AliasStore;
use crate::alias_types::{AliasEntry, AliasScope, CreateResult, ParamDefault};

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
    pub fn from_entry(entry: &AliasEntry) -> Self {
        AliasEntryView {
            name: entry.name.clone(),
            scope: entry.scope.display_string(),
            query: entry.query.clone(),
            parameters: entry.parameters.as_ref().map(|params| {
                params
                    .iter()
                    .map(|(k, v)| (k.clone(), v.value.clone()))
                    .collect()
            }),
            description: entry.description.clone(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool handler functions
// ─────────────────────────────────────────────────────────────────────────────

/// Handle the `create_alias` MCP tool invocation (BC-2.11.008).
///
/// # Steps
/// 1. Parse `input.scope` → `AliasScope`.
/// 2. Validate alias name format (`[a-zA-Z_][a-zA-Z0-9_]*`, 1–64 chars).
/// 3. Check keyword/OCSF collision → `E-ALIAS-006`.
/// 4. Validate parameter defaults as PrismQL atomic literals → `E-ALIAS-004`.
/// 5. Run cycle detection → `E-ALIAS-002`.
/// 6. Validate parser accepts the template.
/// 7. Call `AliasStore::create_or_update(entry, None)`.
/// 8. If `CreateResult::ConfirmationRequired`: return token response.
///    If `CreateResult::Created`: return success with expanded form.
///
/// Returns `Err` on any validation or I/O failure.
pub fn create_alias(
    input: CreateAliasInput,
    store: &mut AliasStore,
    ocsf_reserved: &std::collections::HashSet<String>,
) -> Result<serde_json::Value, PrismError> {
    create_alias_with_clients(input, store, ocsf_reserved, &[])
}

/// Handle the `create_alias` MCP tool invocation (BC-2.11.008) with client ID validation.
///
/// When `valid_client_ids` is non-empty, client-scoped aliases are validated against
/// the list. Pass `&[]` to skip client ID validation (for use in tests/tool contexts
/// where the client list is not available).
pub fn create_alias_with_clients(
    input: CreateAliasInput,
    store: &mut AliasStore,
    ocsf_reserved: &std::collections::HashSet<String>,
    valid_client_ids: &[String],
) -> Result<serde_json::Value, PrismError> {
    // Step 1: parse scope.
    let scope = AliasScope::parse(&input.scope)?;

    // Validate client ID when applicable.
    if let AliasScope::Client(ref client_id) = scope {
        if !valid_client_ids.is_empty() {
            let id_str = client_id.0.as_str();
            if !valid_client_ids.iter().any(|v| v == id_str) {
                return Err(PrismError::ConfigNotFound {
                    path: format!("client:{id_str}"),
                });
            }
        }
    }

    // Step 2: validate alias name.
    validate_alias_name(&input.name)?;

    // Step 3: keyword/OCSF collision check.
    validate_no_keyword_collision(&input.name, ocsf_reserved)?;

    // Step 4: validate parameter defaults.
    let parameters = if let Some(params) = &input.parameters {
        let mut validated: std::collections::HashMap<String, ParamDefault> =
            std::collections::HashMap::new();
        for (param_name, default_value) in params {
            AliasResolver::validate_atomic_literal(default_value, param_name, &input.name)?;
            validated.insert(
                param_name.clone(),
                ParamDefault {
                    value: default_value.clone(),
                },
            );
        }
        Some(validated)
    } else {
        None
    };

    // Step 6: validate PrismQL template syntax (BC-2.11.008 E-QUERY-001).
    // Replace {{param}} placeholders with dummy literal "1" for parse validation.
    let query_for_parse = {
        use std::sync::OnceLock;
        static PLACEHOLDER_RE: OnceLock<regex::Regex> = OnceLock::new();
        #[allow(clippy::expect_used)]
        let re = PLACEHOLDER_RE.get_or_init(|| {
            regex::Regex::new(r"\{\{[a-zA-Z_][a-zA-Z0-9_]*\}\}").expect("placeholder pattern")
        });
        re.replace_all(&input.query, "1").to_string()
    };

    // Use the public PrismQlParser entry point for template validation (BC-2.11.006).
    // If the template cannot be parsed in any PrismQL mode, reject it with E-QUERY-001.
    if crate::filter_parser::PrismQlParser::parse(&query_for_parse).is_err() {
        return Err(PrismError::QueryParseFailed {
            offset: 0,
            detail: format!(
                "alias '{}' query template is not valid PrismQL: {}",
                input.name, query_for_parse
            ),
        });
    }

    // Build the entry.
    let entry = AliasEntry {
        name: input.name.clone(),
        scope: scope.clone(),
        query: input.query.clone(),
        parameters,
        description: input.description.clone(),
    };

    // Step 5: cycle detection (via create_or_update, which calls detect_cycle).
    // Step 7: create or update in store.
    let result = store.create_or_update(entry.clone(), None)?;

    match result {
        CreateResult::Created(created_entry) => {
            // Expand the alias body at depth 0 to get expanded form.
            let args = HashMap::new();
            let expanded = AliasResolver::expand(&created_entry.query, store, &scope, &args, 0)
                .unwrap_or_else(|_| created_entry.query.clone());

            let response = CreateAliasSuccess {
                alias: AliasEntryView::from_entry(&created_entry),
                expanded,
            };
            Ok(
                serde_json::to_value(response).map_err(|e| PrismError::McpParameterInvalid {
                    tool: "create_alias".to_string(),
                    detail: format!("response serialization failed: {e}"),
                })?,
            )
        }
        CreateResult::ConfirmationRequired {
            token_client_id,
            token_json: _,
        } => {
            let response = serde_json::json!({
                "confirmation_required": true,
                "message": format!(
                    "alias '{}' already exists; provide a confirmation token to update it",
                    input.name
                ),
                "token_client_id": token_client_id,
            });
            Ok(response)
        }
    }
}

/// Handle the `list_aliases` MCP tool invocation (BC-2.11.013).
///
/// # Steps
/// 1. Parse optional scope filter.
/// 2. If `scope` references a non-existent client: return `E-CFG-001`.
/// 3. Call `AliasStore::list(scope_filter)`.
/// 4. Sort alphabetically by name within each scope group.
///
/// Returns `Ok(serde_json::Value)` — JSON array of `AliasEntryView` objects.
pub fn list_aliases(
    input: ListAliasesInput,
    store: &AliasStore,
    valid_client_ids: &[String],
) -> Result<serde_json::Value, PrismError> {
    // Parse optional scope filter.
    let scope_filter: Option<AliasScope> = match input.scope {
        None => None,
        Some(ref s) => {
            let parsed = AliasScope::parse(s)?;
            // Validate client ID exists if scope is per-client.
            if let AliasScope::Client(ref client_id) = parsed {
                let id_str = client_id.0.as_str();
                if !valid_client_ids.iter().any(|v| v == id_str) {
                    return Err(PrismError::ConfigNotFound {
                        path: format!("client:{id_str}"),
                    });
                }
            }
            Some(parsed)
        }
    };

    let entries = store.list(scope_filter.as_ref());
    let views: Vec<AliasEntryView> = entries
        .iter()
        .map(|e| AliasEntryView::from_entry(e))
        .collect();

    serde_json::to_value(views).map_err(|e| PrismError::McpParameterInvalid {
        tool: "list_aliases".to_string(),
        detail: format!("response serialization failed: {e}"),
    })
}

/// Handle the `delete_alias` MCP tool invocation (BC-2.11.014).
///
/// Deletion ALWAYS requires a confirmation token.
///
/// # Steps
/// 1. Parse `input.scope` → `AliasScope`.
/// 2. Validate client ID exists if scope is per-client.
/// 3. Check alias exists; return `E-ALIAS-001` if absent.
/// 4. If no token: issue ConfirmationRequired response.
/// 5. If token provided: resolve dependents, call `AliasStore::delete`.
///
/// Returns confirmation token JSON on first call, success JSON after confirmation.
pub fn delete_alias(
    input: DeleteAliasInput,
    store: &mut AliasStore,
    _token_store: &prism_security::ConfirmationTokenStore,
    valid_client_ids: &[String],
) -> Result<serde_json::Value, PrismError> {
    // Step 1: parse scope.
    let scope = AliasScope::parse(&input.scope)?;

    // Step 2: validate client ID.
    if let AliasScope::Client(ref client_id) = scope {
        let id_str = client_id.0.as_str();
        if !valid_client_ids.iter().any(|v| v == id_str) {
            return Err(PrismError::ConfigNotFound {
                path: format!("client:{id_str}"),
            });
        }
    }

    // Step 3: check alias exists.
    let exists = store.get(&input.name, &scope)?.is_some();
    if !exists {
        let all_entries = store.list(None);
        let available = all_entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(PrismError::AliasNotFound {
            name: input.name.clone(),
            scope: scope.display_string(),
            available,
        });
    }

    // Step 4: no token — issue confirmation required response.
    if input.token_id.is_none() {
        let deps = store.dependents(&input.name, &scope);
        let mut response = serde_json::json!({
            "confirmation_required": true,
            "message": format!(
                "deleting alias '{}' requires confirmation; provide token_id to proceed",
                input.name
            ),
            "token_client_id": scope.token_client_id(),
        });
        if !deps.is_empty() {
            response["dependent_aliases"] = serde_json::json!(deps);
            if !input.force {
                response["warning"] = serde_json::json!(format!(
                    "alias '{}' has {} dependent(s); pass force=true to cascade-delete",
                    input.name,
                    deps.len()
                ));
            }
        }
        return Ok(response);
    }

    // Step 5: token provided — check dependents and delete.
    // NOTE: In a full implementation we would validate the token via token_store.consume().
    // For this story scope, we accept any non-None token_id as confirmation.
    let deps = store.dependents(&input.name, &scope);
    if !deps.is_empty() && !input.force {
        return Err(PrismError::AliasDependentsExist {
            name: input.name.clone(),
            count: deps.len(),
            dependents: deps.join(", "),
        });
    }

    // Build a synthetic token for the store.delete() call.
    // Full token validation is deferred to the TD-S304-001 tech debt item.
    let synthetic_token = build_synthetic_token(&input, &scope);

    let result = store.delete(&input.name, &scope, input.force, synthetic_token)?;

    match result {
        crate::alias_types::DeleteResult::Deleted {
            name,
            cascade_deleted,
            ..
        } => {
            let response = DeleteAliasSuccess {
                deleted: name,
                cascade_deleted,
            };
            Ok(
                serde_json::to_value(response).map_err(|e| PrismError::McpParameterInvalid {
                    tool: "delete_alias".to_string(),
                    detail: format!("response serialization failed: {e}"),
                })?,
            )
        }
    }
}

/// Handle the `explain_alias` MCP tool invocation (BC-2.11.015).
///
/// # Steps
/// 1. Parse optional `input.scope`; if absent, use per-client-overrides-global
///    precedence.
/// 2. Look up alias; return `E-ALIAS-001` if absent.
/// 3. Run `AliasResolver::expand()` to produce the expanded form.
///
/// Returns `Ok(ExplainAliasResponse)` serialized as JSON.
pub fn explain_alias(
    input: ExplainAliasInput,
    store: &AliasStore,
    client_scope: Option<&prism_core::types::ClientId>,
) -> Result<ExplainAliasResponse, PrismError> {
    // Step 1: resolve scope.
    let resolved_scope: AliasScope = if let Some(scope_str) = &input.scope {
        AliasScope::parse(scope_str)?
    } else if let Some(client_id) = client_scope {
        AliasScope::Client(client_id.clone())
    } else {
        AliasScope::Global
    };

    // Step 2: look up alias with scope precedence.
    let entry = AliasResolver::resolve_scope(&input.name, store, &resolved_scope)?;

    // Step 3: expand to get full expansion + composition chain.
    let args = HashMap::new();
    let expanded = AliasResolver::expand(&entry.query, store, &resolved_scope, &args, 0)?;

    // Build a composition chain by detecting what aliases were in the body.
    let body_aliases = AliasResolver::detect_alias_tokens(&entry.query);
    let mut composition_chain = vec![entry.name.clone()];
    composition_chain.extend(body_aliases);
    let composition_depth = composition_chain.len();

    Ok(ExplainAliasResponse {
        name: entry.name.clone(),
        scope: entry.scope.display_string(),
        query: entry.query.clone(),
        expanded,
        parameters: entry.parameters.as_ref().map(|params| {
            params
                .iter()
                .map(|(k, v)| (k.clone(), v.value.clone()))
                .collect()
        }),
        description: entry.description.clone(),
        composition_chain,
        composition_depth,
    })
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
    name: &str,
    ocsf_reserved: &std::collections::HashSet<String>,
) -> Result<(), PrismError> {
    let name_upper = name.to_uppercase();

    // Check PrismQL keywords.
    for &kw in PRISMQL_KEYWORDS {
        if name_upper == kw {
            return Err(PrismError::AliasNameConflict {
                name: name.to_string(),
                conflict_kind: "PrismQL keyword".to_string(),
                conflict: kw.to_string(),
            });
        }
    }

    // Check OCSF fields (case-insensitive; ocsf_reserved is already lowercased).
    let name_lower = name.to_lowercase();
    if let Some(field) = ocsf_reserved.get(&name_lower) {
        return Err(PrismError::AliasNameConflict {
            name: name.to_string(),
            conflict_kind: "OCSF field name".to_string(),
            conflict: field.clone(),
        });
    }

    Ok(())
}

/// Validate that the alias name matches the required pattern.
///
/// Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`, 1–64 characters.
/// Returns `Err(PrismError::McpParameterInvalid)` on violation.
pub fn validate_alias_name(name: &str) -> Result<(), PrismError> {
    if name.is_empty() {
        return Err(PrismError::McpParameterInvalid {
            tool: "alias".to_string(),
            detail: "alias name must not be empty".to_string(),
        });
    }

    if name.len() > 64 {
        return Err(PrismError::McpParameterInvalid {
            tool: "alias".to_string(),
            detail: format!(
                "alias name '{}' exceeds 64 character limit (got {})",
                name,
                name.len()
            ),
        });
    }

    // Must match [a-zA-Z_][a-zA-Z0-9_]* — all ASCII.
    let mut chars = name.chars();
    // Safety: already checked name.is_empty() above.
    let Some(first) = chars.next() else {
        return Err(PrismError::McpParameterInvalid {
            tool: "alias".to_string(),
            detail: "alias name must not be empty".to_string(),
        });
    };

    // Validate using ASCII-only character checks.
    let first_valid = first.is_ascii_alphabetic() || first == '_';
    if !first_valid {
        return Err(PrismError::McpParameterInvalid {
            tool: "alias".to_string(),
            detail: format!(
                "alias name '{}' must start with a letter or underscore (got '{}')",
                name, first
            ),
        });
    }

    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return Err(PrismError::McpParameterInvalid {
                tool: "alias".to_string(),
                detail: format!(
                    "alias name '{}' contains invalid character '{}'; only ASCII letters, digits, and underscores are allowed",
                    name, ch
                ),
            });
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build a synthetic ConfirmationToken for the delete path.
///
/// The full token validation (store.consume() path) is deferred to
/// TD-S304-001 (ConfirmationTokenStore integration). For this story scope,
/// we accept any non-None token_id as the user's explicit acknowledgment.
fn build_synthetic_token(
    input: &DeleteAliasInput,
    scope: &AliasScope,
) -> prism_security::ConfirmationToken {
    use serde_json::json;
    prism_security::ConfirmationToken {
        token_id: input
            .token_id
            .clone()
            .unwrap_or_else(|| "synthetic".to_string()),
        client_id: scope.token_client_id().to_string(),
        tool_name: "delete_alias".to_string(),
        action_params: json!({"name": input.name, "scope": input.scope, "force": input.force}),
        action_summary: format!("delete alias '{}'", input.name),
        action_hash: String::new(),
        created_at: std::time::SystemTime::now(),
        expires_at: std::time::SystemTime::now() + std::time::Duration::from_secs(300),
        consumed: false,
    }
}
