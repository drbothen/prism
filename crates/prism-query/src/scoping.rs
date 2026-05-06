//! `scoping` — cross-client query scope resolution.
//!
//! Implements BC-2.11.011 intersection semantics:
//!   - Tool parameters define the outer boundary.
//!   - Query predicates (`_client = "..."`) can only narrow within that boundary.
//!   - Predicates cannot widen scope — out-of-scope clients produce empty results.
//!
//! # BC References
//! - BC-2.11.011 — Cross-Client Query Scoping
//!
//! Story: S-3.02

// S-3.02 stub functions: dead_code suppressed for stub phase (BC-5.38.001).
#![allow(dead_code)]

use prism_core::{OrgSlug, PrismError};

// ---------------------------------------------------------------------------
// ClientRegistry
// ---------------------------------------------------------------------------

/// Registry of all configured client IDs known to this Prism instance.
///
/// Used by `resolve_clients` to expand `clients: None` (meaning all configured
/// clients) to a concrete `Vec<OrgSlug>`.
///
/// Implements BC-2.11.011 outer-boundary resolution.
#[derive(Debug, Default)]
pub struct ClientRegistry {
    /// All configured client IDs.
    client_ids: Vec<OrgSlug>,
}

impl ClientRegistry {
    /// Construct a `ClientRegistry` from a list of configured client slugs.
    pub fn new(client_ids: Vec<OrgSlug>) -> Self {
        Self { client_ids }
    }

    /// Return all configured client IDs.
    pub fn all_clients(&self) -> &[OrgSlug] {
        &self.client_ids
    }

    /// Return true if the registry contains the given client ID.
    pub fn contains(&self, client_id: &OrgSlug) -> bool {
        self.client_ids.contains(client_id)
    }
}

// ---------------------------------------------------------------------------
// resolve_clients
// ---------------------------------------------------------------------------

/// Resolve the effective client scope for a query.
///
/// Intersection semantics (BC-2.11.011):
/// - `clients: None` → all clients in `registry`
/// - `clients: Some(list)` → validate each exists; return list
///
/// Returns `Err(PrismError::InvalidClientId)` if any client ID in the list
/// is not found in the registry. (BC-2.11.001 E-MCP-004 path)
///
/// The returned `Vec<OrgSlug>` is the effective fan-out target list.
/// Each client's sensor data will be a separate RecordBatch contributing
/// to the same MemTable. (BC-2.11.011)
pub fn resolve_clients(
    _clients: Option<Vec<OrgSlug>>,
    _registry: &ClientRegistry,
) -> Result<Vec<OrgSlug>, PrismError> {
    todo!("S-3.02 — resolve_clients")
}

// ---------------------------------------------------------------------------
// intersect_query_client_predicates
// ---------------------------------------------------------------------------

/// Intersect tool-parameter client scope with query-predicate client scope.
///
/// If the query AST contains `_client = "..."` predicates, narrow the
/// `tool_scope` to the intersection. If the intersection is empty, returns
/// an empty `Vec` (not an error — BC-2.11.011 specifies empty results).
///
/// # BC-2.11.011
/// "Query predicates cannot widen" — if a predicate names a client outside
/// the tool-parameter scope, it is silently excluded.
pub(crate) fn intersect_query_client_predicates(
    _tool_scope: Vec<OrgSlug>,
    _query_client_predicates: &[OrgSlug],
) -> Vec<OrgSlug> {
    todo!("S-3.02 — intersect_query_client_predicates")
}
