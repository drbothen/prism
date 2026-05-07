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
use tracing;

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
/// **Client-scope restriction:** This variant does not receive a valid client
/// list, so `Client(_)` scopes are rejected with `E-CFG-001`. Pass
/// `create_alias_with_clients` with an explicit `valid_client_ids` list to
/// permit per-client alias creation (CR-007).
///
/// Returns `Err` on any validation or I/O failure.
pub fn create_alias(
    input: CreateAliasInput,
    store: &mut AliasStore,
    ocsf_reserved: &std::collections::HashSet<String>,
) -> Result<serde_json::Value, PrismError> {
    // Reject Client-scoped creates when no valid client list is available (CR-007).
    let scope_check = AliasScope::parse(&input.scope)?;
    if let AliasScope::Client(ref client_id) = scope_check {
        let id_str = client_id.0.as_str();
        return Err(PrismError::ConfigNotFound {
            path: format!("client:{id_str}"),
        });
    }
    create_alias_with_clients(input, store, ocsf_reserved, &[])
}

/// Handle the `create_alias` MCP tool invocation (BC-2.11.008) with client ID validation.
///
/// When `valid_client_ids` is non-empty, client-scoped aliases are validated against
/// the list. Pass `&[]` to skip client ID validation (for use in tests/tool contexts
/// where the client list is not available).
///
/// `capability_gate` is an optional `(FeatureFlagEvaluator, CompileTimeGate)` pair.
/// When `Some(...)`, the `alias.write` capability is checked before any mutation
/// (SEC-005 / BC-2.11.008 precondition). When `None`, the capability check is skipped
/// (backward-compatible for test harnesses that don't supply a capability context).
pub fn create_alias_with_clients(
    input: CreateAliasInput,
    store: &mut AliasStore,
    ocsf_reserved: &std::collections::HashSet<String>,
    valid_client_ids: &[String],
) -> Result<serde_json::Value, PrismError> {
    create_alias_with_clients_gated(input, store, ocsf_reserved, valid_client_ids, None)
}

/// Full-capability-gated variant of `create_alias_with_clients` (SEC-005).
///
/// Callers that have a `FeatureFlagEvaluator` should call this directly so that the
/// `alias.write` capability gate is enforced before any mutation.
pub fn create_alias_with_clients_gated(
    input: CreateAliasInput,
    store: &mut AliasStore,
    ocsf_reserved: &std::collections::HashSet<String>,
    valid_client_ids: &[String],
    capability_gate: Option<(
        &prism_security::feature_flag::FeatureFlagEvaluator,
        prism_security::feature_flag::CompileTimeGate,
    )>,
) -> Result<serde_json::Value, PrismError> {
    // Step 1: parse scope.
    let scope = AliasScope::parse(&input.scope)?;

    // SEC-005 / BC-2.11.008 precondition: check alias.write capability gate when provided.
    if let Some((evaluator, compile_gate)) = capability_gate {
        crate::alias_capability::check_alias_write(&scope, evaluator, compile_gate)?;
    }

    // Validate client ID when scope is Client(_). ALWAYS enforced — an empty
    // valid_client_ids list means no client is valid, so any client-scoped op
    // is rejected with E-CFG-001. (CR-007 fix)
    if let AliasScope::Client(ref client_id) = scope {
        let id_str = client_id.0.as_str();
        if !valid_client_ids.iter().any(|v| v == id_str) {
            return Err(PrismError::ConfigNotFound {
                path: format!("client:{id_str}"),
            });
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

            // DI-004 audit span (CR-023): emit structured log for alias create.
            // TODO TD-S304-AUDIT-001: replace with prism_audit::emit_audit when available.
            tracing::info!(
                operation = "alias.create",
                alias_name = %created_entry.name,
                alias_scope = %created_entry.scope,
                outcome = "created",
            );

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
            // DI-004 audit span (CR-023): emit for confirmation-required path.
            tracing::info!(
                operation = "alias.create",
                alias_name = %input.name,
                alias_scope = %input.scope,
                outcome = "confirmation_required",
            );
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

    // DI-004 audit span (CR-023): emit for list operation.
    // TODO TD-S304-AUDIT-001: replace with prism_audit::emit_audit when available.
    tracing::info!(
        operation = "alias.list",
        scope = ?input.scope,
        result_count = views.len(),
        outcome = "ok",
    );

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
///
/// # Two-step flow (CR-008 — full token lifecycle implemented)
///
/// **First call** (`token_id: None`): the alias exists check passes, the
/// function generates a `ConfirmationToken` via `token_store.generate()` bound
/// to `(name, scope, force)`, and returns `confirmation_required: true` with the
/// `token_id` in the response.
///
/// **Second call** (`token_id: Some(id)`): the function consumes the token via
/// `token_store.consume(token_id, client_id, action_params)`. If the consume
/// fails (unknown token, expired, hash mismatch), returns `E-FLAG-008` or
/// the relevant error. On success, proceeds with the delete.
pub fn delete_alias(
    input: DeleteAliasInput,
    store: &mut AliasStore,
    token_store: &prism_security::ConfirmationTokenStore,
    valid_client_ids: &[String],
) -> Result<serde_json::Value, PrismError> {
    delete_alias_gated(input, store, token_store, valid_client_ids, None)
}

/// Full-capability-gated variant of `delete_alias` (SEC-005).
///
/// Callers that have a `FeatureFlagEvaluator` should call this directly so that the
/// `alias.write` capability gate is enforced before any mutation.
pub fn delete_alias_gated(
    input: DeleteAliasInput,
    store: &mut AliasStore,
    token_store: &prism_security::ConfirmationTokenStore,
    valid_client_ids: &[String],
    capability_gate: Option<(
        &prism_security::feature_flag::FeatureFlagEvaluator,
        prism_security::feature_flag::CompileTimeGate,
    )>,
) -> Result<serde_json::Value, PrismError> {
    // Step 1: parse scope.
    let scope = AliasScope::parse(&input.scope)?;

    // SEC-005 / BC-2.11.008 precondition: check alias.write capability gate when provided.
    if let Some((evaluator, compile_gate)) = capability_gate {
        crate::alias_capability::check_alias_write(&scope, evaluator, compile_gate)?;
    }

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
        // Restrict error disclosure to scope-visible aliases only (SEC-003).
        let visible_entries: Vec<&AliasEntry> = match &scope {
            AliasScope::Client(_) => {
                let mut v = store.list(Some(&scope));
                v.extend(store.list(Some(&AliasScope::Global)));
                v
            }
            AliasScope::Global => store.list(Some(&AliasScope::Global)),
        };
        let available = visible_entries
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

    // Step 4: no token — generate a real ConfirmationToken and return it.
    if input.token_id.is_none() {
        let deps = store.dependents(&input.name, &scope);

        // Generate a real token bound to (name, scope, force) (CR-008).
        let action_params = serde_json::json!({
            "name": input.name,
            "scope": input.scope,
            "force": input.force
        });
        let token = token_store.generate(
            scope.token_client_id(),
            "delete_alias",
            action_params,
            &format!("delete alias '{}'", input.name),
        )?;

        let mut response = serde_json::json!({
            "confirmation_required": true,
            "message": format!(
                "deleting alias '{}' requires confirmation; provide token_id to proceed",
                input.name
            ),
            "token_client_id": scope.token_client_id(),
            "token_id": token.token_id,
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
        // DI-004 audit span (CR-023): emit for delete confirmation-required.
        // TODO TD-S304-AUDIT-001: replace with prism_audit::emit_audit when available.
        tracing::info!(
            operation = "alias.delete",
            alias_name = %input.name,
            alias_scope = %input.scope,
            force = input.force,
            outcome = "confirmation_required",
        );
        return Ok(response);
    }

    // Step 5: token provided — validate via token_store.consume() (CR-008).
    // Safety: we checked is_none() above; if None we returned Ok early.
    let Some(ref token_id_str) = input.token_id else {
        // Structurally unreachable — is_none() check above returns Ok(response).
        return Err(PrismError::McpParameterInvalid {
            tool: "delete_alias".to_string(),
            detail: "internal: token_id unexpectedly None after is_none check".to_string(),
        });
    };
    let token_id = token_id_str.as_str();
    let action_params = serde_json::json!({
        "name": input.name,
        "scope": input.scope,
        "force": input.force
    });
    let consumed_token = token_store.consume(token_id, scope.token_client_id(), &action_params)?;

    // Check dependents and delete.
    let deps = store.dependents(&input.name, &scope);
    if !deps.is_empty() && !input.force {
        return Err(PrismError::AliasDependentsExist {
            name: input.name.clone(),
            count: deps.len(),
            dependents: deps.join(", "),
        });
    }

    let result = store.delete(&input.name, &scope, input.force, consumed_token)?;

    match result {
        crate::alias_types::DeleteResult::Deleted {
            name,
            cascade_deleted,
            ..
        } => {
            // DI-004 audit span (CR-023): emit for successful delete.
            tracing::info!(
                operation = "alias.delete",
                alias_name = %name,
                alias_scope = %input.scope,
                force = input.force,
                cascade_deleted = ?cascade_deleted,
                outcome = "deleted",
            );
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

    // CR-014: validate the expanded query via the PrismQL parser.
    // If the expansion is not valid PrismQL, return E-QUERY-001.
    if crate::filter_parser::PrismQlParser::parse(&expanded).is_err() {
        return Err(PrismError::QueryParseFailed {
            offset: 0,
            detail: format!(
                "alias '{}' expanded form is not valid PrismQL: {}",
                input.name, expanded
            ),
        });
    }

    // Build composition chain by walking the expansion tree recursively (DFS).
    // CR-006: one-level body_aliases detection was non-recursive and missed depth>1 chains.
    let mut composition_chain = vec![entry.name.clone()];
    build_composition_chain(
        &entry.query,
        store,
        &resolved_scope,
        0,
        &mut composition_chain,
    );
    let composition_depth = composition_chain.len();

    // DI-004 audit span (CR-023): emit for explain operation.
    // TODO TD-S304-AUDIT-001: replace with prism_audit::emit_audit when available.
    tracing::info!(
        operation = "alias.explain",
        alias_name = %input.name,
        alias_scope = %resolved_scope,
        composition_depth = composition_depth,
        outcome = "ok",
    );

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

/// Recursively walk the alias expansion tree (DFS) to build a composition chain.
///
/// Starting from the `query` template, detect all `@alias_name` references,
/// add each to `chain`, then recurse into the body of each discovered alias.
/// Stops at `MAX_ALIAS_DEPTH` to prevent stack overflow. (CR-006 fix)
fn build_composition_chain(
    query: &str,
    store: &AliasStore,
    scope: &AliasScope,
    depth: u32,
    chain: &mut Vec<String>,
) {
    use crate::alias_resolver::MAX_ALIAS_DEPTH;
    if depth >= MAX_ALIAS_DEPTH {
        return;
    }

    let tokens = AliasResolver::detect_alias_tokens(query);
    for token_name in &tokens {
        if chain.contains(token_name) {
            // Already recorded — avoid infinite loop on cycles (cycle detection
            // should prevent cycles in store, but guard defensively).
            continue;
        }
        chain.push(token_name.clone());

        // Resolve the alias body with the same scope-precedence as resolve_scope.
        let entry_query: Option<String> = {
            let client_entry = if let AliasScope::Client(_) = scope {
                store
                    .get(token_name, scope)
                    .ok()
                    .flatten()
                    .map(|e| e.query.clone())
            } else {
                None
            };
            client_entry.or_else(|| {
                store
                    .get(token_name, &AliasScope::Global)
                    .ok()
                    .flatten()
                    .map(|e| e.query.clone())
            })
        };

        if let Some(body) = entry_query {
            build_composition_chain(&body, store, scope, depth + 1, chain);
        }
    }
}
