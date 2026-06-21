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

use async_trait::async_trait;
use rmcp::{
    model::{ErrorData, ReadResourceResult, ResourceUpdatedNotificationParam},
    service::Peer,
    RoleServer,
};

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

// ─── SchemaChangeNotifier trait (BC-2.10.013 subscribe/notify) ───────────────

/// Notification target for `prismql://schema/{client_id}` resource updates.
///
/// Implemented by the production `Peer<RoleServer>` wrapper and by test mocks
/// (injectable via `SubscriberHandle::notifier`). Called by `notify_schema_updated`
/// for each subscriber of the changed client.
///
/// # DI-004 fail-open contract
/// `notify_schema_updated` MUST NOT abort notification of other subscribers if
/// one call returns `Err`. Errors are logged at WARN and iteration continues.
///
/// # Production implementation
/// The production implementor wraps `Peer<RoleServer>` from the rmcp transport
/// layer. It is constructed in the `ServerHandler::subscribe` override, stored in
/// `SubscriberHandle`, and removed in `ServerHandler::unsubscribe`.
#[async_trait]
pub trait SchemaChangeNotifier: Send + Sync + 'static {
    /// Dispatch `notifications/resources/updated` for the given resource URI.
    ///
    /// Called by `notify_schema_updated` for each subscriber of the changed client.
    /// The `uri` format is `"prismql://schema/{client_id}"`.
    async fn notify_resource_updated(&self, uri: &str) -> Result<(), ErrorData>;
}

// ─── Per-client subscriber registry (BC-2.10.013 subscribe/notify) ───────────

/// A handle representing a subscribed MCP client for a given schema URI.
///
/// Carries the notification target (`notifier`) that `notify_schema_updated`
/// calls to dispatch `notifications/resources/updated` to this subscriber.
///
/// # Non-exhaustive note
/// `SubscriberHandle` is NOT marked `#[non_exhaustive]` because integration
/// tests in `prism-mcp/tests/` construct it with struct literal syntax — this
/// requires all fields to be visible. If new fields are added, integration
/// tests using struct literal syntax must be updated accordingly.
pub struct SubscriberHandle {
    /// Opaque identifier for this subscription (e.g., connection ID).
    pub id: String,
    /// Notification target — called when the subscribed schema resource changes.
    ///
    /// In production: wraps a `Peer<RoleServer>` from the rmcp transport layer.
    /// In tests: an injectable mock implementing `SchemaChangeNotifier`.
    pub notifier: Arc<dyn SchemaChangeNotifier>,
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
    /// Thread-safe: acquires the inner `Mutex` lock. Per-client scoping (DI-008):
    /// only the named client's subscriber vec is mutated.
    ///
    /// F-006: poison-tolerant lock — recovers via `into_inner()` on `PoisonError`.
    /// The guarded data is a plain `HashMap` with no broken invariant on poison.
    /// (CLAUDE.md §Conventions: `expect()` on `Result` forbidden in production paths.)
    pub fn subscribe(&self, client: OrgSlug, handle: SubscriberHandle) {
        let mut map = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.entry(client).or_default().push(handle);
    }

    /// Remove a subscriber for the given client slug by ID.
    ///
    /// F-006: poison-tolerant lock.
    pub fn unsubscribe(&self, client: &OrgSlug, id: &str) {
        let mut map = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
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
    ///
    /// F-006: poison-tolerant lock.
    pub fn subscribers_for(&self, client: &OrgSlug) -> Vec<String> {
        let map = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.get(client)
            .map(|handles| handles.iter().map(|h| h.id.clone()).collect())
            .unwrap_or_default()
    }

    /// Return a snapshot of `(id, notifier)` pairs for the given client.
    ///
    /// The `Arc<dyn SchemaChangeNotifier>` is cloned (cheap reference-count bump),
    /// so this method releases the `Mutex` lock before any async notification calls.
    /// Per-client scoping (DI-008): only handles for `client` are returned.
    ///
    /// F-006: poison-tolerant lock.
    pub fn subscriber_notifiers_for(
        &self,
        client: &OrgSlug,
    ) -> Vec<(String, Arc<dyn SchemaChangeNotifier>)> {
        let map = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.get(client)
            .map(|handles| {
                handles
                    .iter()
                    .map(|h| (h.id.clone(), Arc::clone(&h.notifier)))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Return all client slugs that currently have active subscriptions.
    ///
    /// Used by `reload_config` to fan-out `notifications/resources/updated` to
    /// every subscribed client when the global table-set changes (AC-006).
    /// Lock is held only for the snapshot clone, released before any async work.
    ///
    /// F-006: poison-tolerant lock.
    pub fn all_subscribed_clients(&self) -> Vec<OrgSlug> {
        let map = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.keys().cloned().collect()
    }
}

impl Default for SchemaSubscriberRegistry {
    /// WIRING-EXEMPT: `Default` delegation to `Self::new()` — single call, no logic.
    fn default() -> Self {
        Self::new()
    }
}

// ─── Production SchemaChangeNotifier — wraps a live Peer<RoleServer> ─────────

/// Production `SchemaChangeNotifier` wrapping a stored `Peer<RoleServer>`.
///
/// Constructed in `ServerHandler::subscribe` from `context.peer.clone()` and
/// stored in `SubscriberHandle::notifier`. Outlives the subscribe request —
/// the `Peer<RoleServer>` is `Clone + Send + Sync` and can be held across
/// async boundaries until a config change fires.
///
/// # Production path
/// `ServerHandler::subscribe` → constructs `PeerSchemaNotifier { peer }` →
/// wraps in `SubscriberHandle` → stored in `SchemaSubscriberRegistry`.
/// `reload_config` → `notify_schema_updated` → `notify_resource_updated` →
/// `peer.notify_resource_updated(ResourceUpdatedNotificationParam::new(uri))`.
///
/// # Error mapping
/// `Peer::notify_resource_updated` returns `Result<(), ServiceError>`.
/// `ServiceError` is mapped to `ErrorData` at this boundary
/// (mirrors the existing `dispatch_hot_reload_notifications` pattern).
pub struct PeerSchemaNotifier {
    /// Stored connection peer — captured from `RequestContext::peer` at subscribe time.
    pub peer: Peer<RoleServer>,
}

#[async_trait]
impl SchemaChangeNotifier for PeerSchemaNotifier {
    async fn notify_resource_updated(&self, uri: &str) -> Result<(), ErrorData> {
        self.peer
            .notify_resource_updated(ResourceUpdatedNotificationParam::new(uri.to_string()))
            .await
            .map_err(|e| {
                ErrorData::new(
                    rmcp::model::ErrorCode(-32000),
                    format!("notify_resource_updated failed: {e}"),
                    None,
                )
            })
    }
}

// ─── prismql://schema/{client_id} resource handler ───────────────────────────

/// Handle `resources/read("prismql://schema/{client_id}")`.
///
/// Content is structurally identical to `prism_describe(client_id)` — same column
/// data source (`resolved_spec_map` or `config_manager` fallback), same
/// `PrismDescribeResponse` JSON shape (AC-005 parity invariant, BC-2.10.013).
///
/// On invalid `client_id` URI component (path-traversal etc.): returns MCP resource
/// error "Invalid client_id in resource URI" (EC-007 / BC-2.10.013 EC-10-033).
/// DI-008: only returns the named client's tables.
pub async fn render_pql_schema_resource(
    client_id: &str,
    query_engine: Option<&Arc<prism_query::engine::QueryEngine>>,
    config_manager: Option<
        &Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    >,
) -> Result<ReadResourceResult, ErrorData> {
    use crate::tools::prism_describe::handle_prism_describe;
    use rmcp::model::ResourceContents;

    // Validate client_id (EC-10-033: path-traversal guard).
    // DI-006: do not echo the raw value in the error message.
    let slug = OrgSlug::new(client_id);
    if slug.is_err() {
        return Err(ErrorData::invalid_params(
            "E-MCP-001: invalid client_id in resource URI — must match [a-zA-Z0-9_-]{1,64}",
            None,
        ));
    }

    // Delegate to handle_prism_describe for single-source-of-truth parity (AC-005).
    let tool_result = handle_prism_describe(
        client_id.to_string(),
        query_engine,
        config_manager,
        None, // no audit_writer in resource read path
    )
    .await?;

    // Extract the JSON text from the CallToolResult content.
    let json_text: String = tool_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    let uri = format!("prismql://schema/{client_id}");
    Ok(ReadResourceResult::new(vec![
        ResourceContents::TextResourceContents {
            uri,
            mime_type: Some("application/json".to_string()),
            text: json_text,
            meta: None,
        },
    ]))
}

/// Dispatch `notifications/resources/updated` for `prismql://schema/{client_id}`
/// to all subscribers of that client.
///
/// Called when a `TableRegistry` change event fires for the given client.
/// Per-client scoping (DI-008): only `client`'s subscribers receive the notification;
/// other clients' subscribers are untouched (BC-2.10.013 EC-10-030).
///
/// # DI-004 fail-open contract
/// If a subscriber's `notify_resource_updated` call returns `Err`, the error is
/// logged at WARN and iteration continues — a single failed notification MUST NOT
/// abort delivery to remaining subscribers or surface an error to the caller.
///
/// # Mutex release before async calls
/// The registry lock is held only long enough to clone the `(id, Arc<notifier>)`
/// snapshot via `subscriber_notifiers_for`. All async notification calls happen
/// after the lock is released, avoiding async-in-Mutex-hold.
pub async fn notify_schema_updated(
    client: &OrgSlug,
    registry: &SchemaSubscriberRegistry,
) -> Result<(), ErrorData> {
    // Snapshot notifiers under the lock, then release before any async calls.
    let notifiers = registry.subscriber_notifiers_for(client);
    if notifiers.is_empty() {
        tracing::debug!(
            client = %client.as_str(),
            "notify_schema_updated: no subscribers for client, skip dispatch"
        );
        return Ok(());
    }

    let uri = format!("prismql://schema/{}", client.as_str());
    tracing::info!(
        client = %client.as_str(),
        subscriber_count = notifiers.len(),
        uri = %uri,
        "notify_schema_updated: dispatching schema change to subscribers (DI-008 scoped)"
    );

    for (id, notifier) in &notifiers {
        if let Err(e) = notifier.notify_resource_updated(&uri).await {
            // DI-004 fail-open: log at WARN and continue to next subscriber.
            tracing::warn!(
                client = %client.as_str(),
                subscriber_id = %id,
                error = ?e,
                "notify_schema_updated: subscriber notification failed (DI-004 warn-and-continue)"
            );
        }
    }

    Ok(())
}

// ─── prismql://reference static resource handler ─────────────────────────────

/// Handle `resources/read("prismql://reference")`.
///
/// Returns the build-time static PQL grammar reference embedded via `include_str!`.
/// Content is IDENTICAL on every call within the same server process (static invariant,
/// AC-008, BC-2.10.014). No subscribe/notify needed (static content).
///
/// NOTE: content is embedded at build time via `include_str!("../pql_reference.md")` —
/// NOT loaded from the filesystem at runtime.
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
