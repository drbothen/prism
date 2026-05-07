//! Core data types for the alias system.
//!
//! Defines `AliasScope`, `AliasEntry`, and `ParamDefault` — the shared vocabulary
//! used by `alias_store`, `alias_resolver`, `alias_capability`, and `alias_tools`.
//!
//! Story: S-3.04 — prism-query: Alias System (P1)
//! BCs:   BC-2.11.008, BC-2.11.009

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use prism_core::types::ClientId;

// ─────────────────────────────────────────────────────────────────────────────
// AliasScope
// ─────────────────────────────────────────────────────────────────────────────

/// The scope at which an alias is defined.
///
/// `Global` aliases are available to every client context.
/// `Client(id)` aliases override a global alias of the same name for a specific
/// client and are only accessible when the query scope matches that client.
///
/// Scope is mandatory on all write operations (BC-2.11.008, BC-2.11.014).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AliasScope {
    /// Available across all client contexts.
    Global,
    /// Scoped to a single client; overrides a global alias of the same name.
    Client(ClientId),
}

impl AliasScope {
    /// Parse a scope string as used in MCP tool inputs.
    ///
    /// Accepted formats:
    /// - `"global"` — produces `AliasScope::Global`
    /// - `"client:<client_id>"` — produces `AliasScope::Client(ClientId::new(client_id))`
    ///
    /// Any other format returns `Err(PrismError::McpParameterInvalid)`.
    pub fn parse(s: &str) -> Result<Self, prism_core::error::PrismError> {
        if s == "global" {
            return Ok(AliasScope::Global);
        }
        if let Some(client_id_str) = s.strip_prefix("client:") {
            use prism_core::tenant::OrgSlug;
            let slug = OrgSlug::new(client_id_str);
            return Ok(AliasScope::Client(ClientId(slug)));
        }
        Err(prism_core::error::PrismError::McpParameterInvalid {
            tool: "alias".to_string(),
            detail: format!(
                "invalid scope format '{}'; expected 'global' or 'client:<client_id>'",
                s
            ),
        })
    }

    /// Return the `client_id` sentinel used in `ConfirmationToken` generation.
    ///
    /// Per BC-2.11.008 postcondition:
    /// - `Global` → `"__global__"`
    /// - `Client(id)` → `id.as_str()`
    pub fn token_client_id(&self) -> &str {
        match self {
            AliasScope::Global => "__global__",
            AliasScope::Client(id) => id.0.as_str(),
        }
    }

    /// Return a human-readable display string for error messages.
    ///
    /// - `Global` → `"global"`
    /// - `Client(id)` → `"client:<id>"`
    pub fn display_string(&self) -> String {
        match self {
            AliasScope::Global => "global".to_string(),
            AliasScope::Client(id) => format!("client:{}", id.0.as_str()),
        }
    }
}

impl std::fmt::Display for AliasScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ParamDefault
// ─────────────────────────────────────────────────────────────────────────────

/// A single parameter default value for a parameterized alias template.
///
/// The stored value must be a PrismQL atomic literal — one of:
/// `StringLiteral`, `IntegerLiteral`, `FloatLiteral`, `BooleanLiteral`,
/// `DurationLiteral`, or `Identifier`. Compound expressions are rejected at
/// both creation time and substitution time (BC-2.11.009 injection guard).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamDefault {
    /// The default value stored as a raw string.
    ///
    /// Re-validated as a PrismQL atomic literal token at substitution time.
    pub value: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// AliasEntry
// ─────────────────────────────────────────────────────────────────────────────

/// A single stored alias definition.
///
/// Persisted as an entry in `aliases.toml`. The `(name, scope)` pair is the
/// unique key — alias names alone are not globally unique (BC-2.11.008).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasEntry {
    /// Alias name — must match `[a-zA-Z_][a-zA-Z0-9_]*`, 1–64 chars.
    pub name: String,
    /// Scope at which this alias is defined.
    pub scope: AliasScope,
    /// Raw PrismQL template string (may contain `{{param}}` placeholders).
    pub query: String,
    /// Optional parameter map: parameter name → default value.
    ///
    /// When present all parameters MUST have defaults.
    pub parameters: Option<HashMap<String, ParamDefault>>,
    /// Optional human-readable description shown in `list_aliases` responses.
    pub description: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// CreateResult / DeleteResult — operation outcome types
// ─────────────────────────────────────────────────────────────────────────────

/// Outcome of a `create_alias` / `create_or_update` operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateResult {
    /// New alias created and persisted immediately.
    Created(AliasEntry),
    /// Alias already existed; a confirmation token is required to update it.
    ConfirmationRequired {
        /// The `client_id` to use when calling `confirm_action`.
        token_client_id: String,
        /// Serialized `ConfirmationToken` for the pending update.
        token_json: String,
    },
}

/// Outcome of a `delete_alias` operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteResult {
    /// Alias (and any cascade-deleted dependents) were removed.
    Deleted {
        /// The primary alias that was deleted.
        name: String,
        /// Scope the deletion targeted.
        scope: AliasScope,
        /// Additional aliases cascade-deleted when `force: true`.
        cascade_deleted: Vec<String>,
    },
}
