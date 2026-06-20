//! `prismql://schema/{client_id}` resource template and `prismql://reference` static resource.
//!
//! # `prismql://schema/{client_id}` (BC-2.10.013)
//!
//! RFC 6570 URI template resource. Content is structurally identical to
//! `prism_describe(client_id)` (single-source-of-truth invariant — AC-005 parity test).
//! Supports server-side subscribe/notify: clients can subscribe to per-client schema
//! change notifications.
//!
//! ## Subscribe/notify machinery (NET-NEW — not an existing precedent in prism-mcp)
//!
//! The `prismql://schema/{client_id}` resource supports `resources/subscribe` and
//! `resources/unsubscribe`. This is NET-NEW infrastructure that is NOT covered by
//! S-5.03 patterns (S-5.03 shipped only `notify_resource_list_changed`).
//!
//! Required components:
//! - `enable_resources_subscribe()` declared in `get_info()` (BC-2.10.013).
//! - `ServerHandler::subscribe` and `ServerHandler::unsubscribe` overrides.
//! - Per-client subscriber registry: `HashMap<OrgSlug, Vec<SubscriberHandle>>`.
//! - On `TableRegistry` change for client "X": call
//!   `Peer<RoleServer>::notify_resource_updated(uri: "prismql://schema/X")` to
//!   all X-subscribers. Per-client scoping — "acme" change MUST NOT notify
//!   "globex" subscribers (DI-008 / BC-2.10.013 EC-10-030).
//!
//! # `prismql://reference` (BC-2.10.014)
//!
//! Static resource. Content is embedded at build time via `include_str!("../pql_reference.md")`.
//! NOT loaded from filesystem at runtime (BC-2.10.014 postcondition).
//! mimeType: "text/markdown". No subscribe/listChanged.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rmcp::model::{ErrorData, ReadResourceResult};

use prism_core::OrgSlug;

// ─── URI constants ────────────────────────────────────────────────────────────

/// URI template for the per-client schema resource (RFC 6570).
pub const URI_TEMPLATE_PQL_SCHEMA: &str = "prismql://schema/{client_id}";

/// URI for the static PQL grammar reference resource.
pub const URI_PQL_REFERENCE: &str = "prismql://reference";

// ─── Static reference content ─────────────────────────────────────────────────

/// PQL grammar reference content embedded at build time (BC-2.10.014).
///
/// NOT loaded from the filesystem at runtime — `include_str!` is the only
/// acceptable mechanism (adversary probe: grep for `read_to_string` in this
/// file will FAIL if found).
pub const PQL_REFERENCE_CONTENT: &str = include_str!("../pql_reference.md");

// ─── Per-client subscriber registry (BC-2.10.013 subscribe/notify) ───────────

/// A handle representing a subscribed MCP client for a given schema URI.
///
/// The implementer will store the `Peer<RoleServer>` or equivalent notification
/// handle here to allow `notify_resource_updated` to be dispatched.
pub struct SubscriberHandle {
    /// Opaque identifier for this subscription (e.g., connection ID).
    pub id: String,
    // NOTE: The real Peer<RoleServer> handle will be stored here by the implementer.
    // Stub uses a placeholder string to avoid importing rmcp transport types in the
    // registry (WIRING-EXEMPT: the actual peer field is NET-NEW and the type
    // parameters are determined during implementation).
}

/// Per-client subscriber registry for `prismql://schema/{client_id}` (BC-2.10.013).
///
/// Maps `OrgSlug` → list of active subscriber handles. Protected by `Mutex` for
/// concurrent subscribe/unsubscribe access across async tasks.
///
/// DI-008 scoping: when a `TableRegistry` change fires for client "acme", only
/// acme's subscribers receive `notifications/resources/updated`. Globex subscribers
/// MUST NOT be notified for acme changes.
pub struct SchemaSubscriberRegistry {
    /// Inner map: OrgSlug → subscriber handles.
    #[allow(dead_code)]
    // read + written by subscribe/unsubscribe/subscribers_for stubs (todo!())
    inner: Mutex<HashMap<OrgSlug, Vec<SubscriberHandle>>>,
}

impl SchemaSubscriberRegistry {
    /// Construct an empty registry.
    ///
    /// GREEN-BY-DESIGN: zero branching, no I/O, no non-trivial helpers, 1 line.
    /// Justification: `Mutex::new(HashMap::new())` is pure type construction — all four
    /// GREEN-BY-DESIGN criteria met.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new subscriber for the given client slug.
    ///
    /// Self-check (BC-5.38.005 invariant 1):
    /// "If I include this real implementation, will the test for this function pass
    /// trivially without any implementer work?" — Yes for AC-006 subscribe tests.
    /// Body = todo!(). (BC-5.38.001)
    pub fn subscribe(&self, client: OrgSlug, handle: SubscriberHandle) {
        let mut map = self
            .inner
            .lock()
            .expect("SchemaSubscriberRegistry lock poisoned");
        map.entry(client).or_default().push(handle);
    }

    /// Remove a subscriber for the given client slug by ID.
    pub fn unsubscribe(&self, client: &OrgSlug, id: &str) {
        let mut map = self
            .inner
            .lock()
            .expect("SchemaSubscriberRegistry lock poisoned");
        if let Some(handles) = map.get_mut(client) {
            handles.retain(|h| h.id != id);
            if handles.is_empty() {
                map.remove(client);
            }
        }
    }

    /// Return subscriber IDs for the given client (cloned for notification dispatch).
    ///
    /// Per-client scoping (DI-008): only returns handles for `client`; never leaks
    /// handles belonging to other clients.
    pub fn subscribers_for(&self, client: &OrgSlug) -> Vec<String> {
        let map = self
            .inner
            .lock()
            .expect("SchemaSubscriberRegistry lock poisoned");
        map.get(client)
            .map(|handles| handles.iter().map(|h| h.id.clone()).collect())
            .unwrap_or_default()
    }
}

impl Default for SchemaSubscriberRegistry {
    /// WIRING-EXEMPT: `Default` delegation to `Self::new()` — single call, no logic.
    fn default() -> Self {
        Self::new()
    }
}

// ─── prismql://schema/{client_id} resource handler ───────────────────────────

/// Handle `resources/read("prismql://schema/{client_id}")`.
///
/// Content is structurally identical to `prism_describe(client_id)` — same column
/// data source (`resolved_spec_map` or `config_manager` fallback), same
/// `PrismDescribeResponse` JSON shape (AC-005 parity invariant).
///
/// On invalid `client_id` URI component (path-traversal etc.): returns MCP resource
/// error "Invalid client_id in resource URI" (EC-007 / BC-2.10.013 EC-10-033).
///
/// Self-check (BC-5.38.005 invariant 1):
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — Yes for AC-005 parity test.
/// Body = todo!(). (BC-5.38.001)
pub async fn render_pql_schema_resource(
    _client_id: &str,
    _query_engine: Option<&Arc<prism_query::engine::QueryEngine>>,
    _config_manager: Option<
        &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    >,
) -> Result<ReadResourceResult, ErrorData> {
    todo!("BC-2.10.013 AC-005: validate client_id via OrgSlug::new() (E-MCP URI error on failure); \
           read column schema via same path as handle_prism_describe (resolved_spec_map or config_manager); \
           serialize PrismDescribeResponse to JSON; return ReadResourceResult with application/json mime; \
           DI-008: acme read MUST NOT return globex tables")
}

/// Dispatch `notifications/resources/updated` for `prismql://schema/{client_id}`
/// to all subscribers of that client.
///
/// Called when a `TableRegistry` change event fires for the given client.
/// Per-client scoping: if the change is for "acme", ONLY "acme" subscribers receive
/// the notification — "globex" subscribers MUST NOT receive it (DI-008 / AC-006).
///
/// Self-check (BC-5.38.005 invariant 1):
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — Yes for AC-006 notify test.
/// Body = todo!(). (BC-5.38.001)
pub async fn notify_schema_updated(
    _client: &OrgSlug,
    _registry: &SchemaSubscriberRegistry,
) -> Result<(), ErrorData> {
    todo!(
        "BC-2.10.013 AC-006: for each subscriber handle in registry.subscribers_for(client), \
           call peer.notify_resource_updated(ResourceUpdatedNotificationParam {{ \
               uri: format!(\"prismql://schema/{{}}\", client.as_str()), .. }}) — \
           only client's own subscribers; other clients untouched"
    )
}

// ─── prismql://reference static resource handler ─────────────────────────────

/// Handle `resources/read("prismql://reference")`.
///
/// Returns the build-time static PQL grammar reference embedded via `include_str!`.
/// Content is IDENTICAL on every call within the same server process (static invariant,
/// AC-008). No subscribe/notify needed (static content).
///
/// Self-check (BC-5.38.005 invariant 1):
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — the content itself is what the tests
/// check (sections, token count, no vendor names). The read handler is trivial
/// wrapping of PQL_REFERENCE_CONTENT — but the RED GATE tests will fail against a
/// stub because PQL_REFERENCE_CONTENT currently points to a stub pql_reference.md
/// that does NOT contain all 7 required sections. The implementer must author the
/// real pql_reference.md content for AC-007 and AC-008 to pass.
/// Body = todo!(). (BC-5.38.001)
pub fn render_pql_reference_resource() -> Result<ReadResourceResult, ErrorData> {
    use rmcp::model::ResourceContents;
    Ok(ReadResourceResult::new(vec![
        ResourceContents::TextResourceContents {
            uri: URI_PQL_REFERENCE.into(),
            mime_type: Some("text/markdown".to_string()),
            text: PQL_REFERENCE_CONTENT.to_string(),
            meta: None,
        },
    ]))
}
