# rmcp Resource Subscribe/Notify — Authoritative API Reference (prism-mcp)

> **Research spike** to de-risk wiring MCP `resources/subscribe` + `notifications/resources/updated`
> for the `prismql://schema/{client_id}` resource (BC-2.10.013). Produced 2026-06-20.
>
> **Pinned rmcp version: `1.7.0`** — `Cargo.lock` line 5915 (`rmcp = { workspace = true }` in
> `crates/prism-mcp/Cargo.toml`, resolves to `1.7.0`, checksum `0810a9f7…f4058e`). All API claims
> below are verified against rmcp 1.7.0 docs.rs **and** cross-checked against the EXISTING prism-mcp
> source (the load-bearing facts are confirmed in-tree, not just from docs).

---

## TL;DR — the key unblocker

The implementer does NOT need to invent a notification sink. The EXISTING hot-reload path
(`PrismServer::reload_config`) already shows the canonical pattern:

```rust
// crates/prism-mcp/src/server.rs — reload_config tool, ~line 3220
pub async fn reload_config(
    &self,
    peer: rmcp::Peer<rmcp::RoleServer>,   // ← rmcp INJECTS the peer as a tool-method param
) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
    ...
    // ~line 3296 — dispatch list_changed using that peer:
    resources::dispatch_hot_reload_notifications(old_tables, new_tables, &peer).await
}
```

And `dispatch_hot_reload_notifications` (resources.rs ~line 1114) calls
`peer.notify_resource_list_changed().await` / `peer.notify_tool_list_changed().await`.

**For `notifications/resources/updated` the implementer uses the SAME `Peer<RoleServer>`**, just a
different method: `peer.notify_resource_updated(ResourceUpdatedNotificationParam::new(uri)).await`.

The difference for subscribe/notify vs list_changed: the `updated` notification fires from a
*different site* (a per-client schema change) than the request that established the subscription
(`subscribe`). So the `Peer` must be **captured at subscribe-time and stored** in the existing
`SchemaSubscriberRegistry` so it survives until a later change event. `Peer<RoleServer>` is
`Clone + Send + Sync` (confirmed below), so storing a clone in the registry is sound. The registry
scaffolding (`SchemaSubscriberRegistry`, `SchemaChangeNotifier` trait, `notify_schema_updated`
dispatcher) **already exists** in `resources/schema.rs` — only the production `Peer` wrapper and the
two `ServerHandler` overrides are missing.

---

## 1. `ServerHandler::subscribe` / `ServerHandler::unsubscribe` — exact signatures

Source: `https://docs.rs/rmcp/1.7.0/rmcp/handler/server/trait.ServerHandler.html` (verified).

```rust
fn subscribe(
    &self,
    request: SubscribeRequestParams,        // NOTE: plural "...Params"
    context: RequestContext<RoleServer>,
) -> impl Future<Output = Result<(), McpError>> + MaybeSendFuture + '_

fn unsubscribe(
    &self,
    request: UnsubscribeRequestParams,      // NOTE: plural "...Params"
    context: RequestContext<RoleServer>,
) -> impl Future<Output = Result<(), McpError>> + MaybeSendFuture + '_
```

- **Async?** Yes — return `impl Future<…>` (`async fn` in the override desugars to this; rmcp uses
  RPITIT, so write the override as `async fn subscribe(&self, …) -> Result<(), ErrorData>`).
- **Return type:** `Result<(), McpError>`. `McpError` is the rmcp alias for `rmcp::model::ErrorData`
  (the prism overrides already use `ErrorData` as the error type — same type, consistent).
- **Default behavior:** rmcp provides default impls returning an empty/no-op future
  (`Ok(())`-equivalent). Because the defaults do nothing, prism MUST override BOTH to register/remove
  handles in the subscriber registry. (Matches the in-tree comment at server.rs ~line 5409: "Resources
  are served by overriding these … methods directly. … no `#[resource_handler]` macro exists in rmcp 1.7".)

### Parameter type — `SubscribeRequestParams`

Source: `https://docs.rs/rmcp/1.7.0/rmcp/model/struct.SubscribeRequestParams.html` (verified).

```rust
#[non_exhaustive]
pub struct SubscribeRequestParams {
    pub meta: Option<Meta>,
    pub uri: String,
}
impl SubscribeRequestParams { pub fn new(uri: impl Into<String>) -> Self { … } }
```

`UnsubscribeRequestParams` is the symmetric type (`meta: Option<Meta>`, `uri: String`,
`#[non_exhaustive]`).

> **Naming caveat (do not guess):** rmcp also exports a *type alias* `SubscribeRequestParam`
> (singular). The trait method takes the **struct `SubscribeRequestParams` (plural)** — this matches
> the plural forms the prism code already uses for `ReadResourceRequestParams` and
> `PaginatedRequestParams`. Use the plural struct names in the override signatures.

---

## 2. Obtaining `Peer<RoleServer>` inside the `subscribe` override

Source: `https://docs.rs/rmcp/1.7.0/rmcp/service/struct.RequestContext.html` (verified).

```rust
pub struct RequestContext<R: ServiceRole> {   // #[non_exhaustive]
    pub ct: CancellationToken,
    pub id: RequestId,
    pub meta: Meta,
    pub extensions: Extensions,
    pub peer: Peer<R>,                          // ← THIS is the notification handle
}
```

- **Does `RequestContext<RoleServer>` expose `.peer`?** YES. `context.peer` is the
  `Peer<RoleServer>` for the connection that issued the `subscribe` request — i.e., the exact client
  that must later receive `notifications/resources/updated`.
- **Exact type:** `Peer<RoleServer>` (rmcp aliases this as `ClientSink` for the server role).
- **Is it `Clone`?** YES. `impl<R: Clone + ServiceRole> Clone for Peer<R>`. `RoleServer` is a unit
  ZST role type and is `Clone`, so `Peer<RoleServer>: Clone`. Also `Send + Sync + Unpin`.
- **Can it be stored in a registry for later dispatch?** YES — `Clone + Send + Sync` makes it safe to
  clone into `SchemaSubscriberRegistry` and hold it across async boundaries until a change event
  fires. This is exactly the requirement subscribe/notify imposes (the notify site differs from the
  subscribe site).

> Compare to the `reload_config` path: there the peer is needed only *within the same tool call*, so
> rmcp's tool-method peer injection (`peer: rmcp::Peer<rmcp::RoleServer>` as a `#[tool]` fn param)
> suffices and nothing is stored. For subscribe/notify the peer must outlive the request, so capture
> `context.peer.clone()` and store it.

---

## 3. `Peer<RoleServer>::notify_resource_updated` — exact signature + param

Source: `https://docs.rs/rmcp/1.7.0/rmcp/service/type.ClientSink.html` (the `Peer<RoleServer>` impl
block; verified).

```rust
pub async fn notify_resource_updated(
    &self,
    params: ResourceUpdatedNotificationParam,
) -> Result<(), ServiceError>
```

- This sends the JSON-RPC `notifications/resources/updated` message to the connected client.
- **Return type:** `Result<(), ServiceError>` (note: `ServiceError`, not `ErrorData` — same as the
  existing `notify_resource_list_changed()` used in `dispatch_hot_reload_notifications`). Map it to
  `ErrorData` at the boundary if you need to, exactly as the existing code does with `.map_err(…)`.

### Param type — `ResourceUpdatedNotificationParam`

Source: `https://docs.rs/rmcp/1.7.0/rmcp/model/struct.ResourceUpdatedNotificationParam.html` (verified).

```rust
pub struct ResourceUpdatedNotificationParam {
    pub uri: String,                  // "The URI of the resource that was updated"
}
impl ResourceUpdatedNotificationParam { pub fn new(uri: impl Into<String>) -> Self { … } }
```

- Single field `uri: String`. Construct via `ResourceUpdatedNotificationParam::new(uri)`.
- **NOT `#[non_exhaustive]`** in 1.7.0 (struct-literal construction is allowed, but prefer `::new`).
- Derives `Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema`.

For prism the `uri` value is `format!("prismql://schema/{client_id}")` — already what
`notify_schema_updated` builds in `resources/schema.rs` (~line 290).

---

## 4. `enable_resources_subscribe()` on the capabilities builder — CONFIRMED present and in use

Source: rmcp 1.7.0 `ServerCapabilities::builder()`; cross-confirmed in-tree at
`crates/prism-mcp/src/server.rs` `get_info()` (~line 5403):

```rust
ServerCapabilities::builder()
    .enable_tools()
    .enable_prompts()
    .enable_resources()
    .enable_resources_subscribe()   // ← already called (BC-2.10.013 AC-006)
    .build()
```

- It advertises the `resources.subscribe = true` capability in the server's `initialize` response,
  telling MCP clients they may issue `resources/subscribe`. **Already wired** — no change needed here.
- (`enable_resources_subscribe()` advertises `subscribe`; it does NOT by itself advertise
  `listChanged`. The existing `.enable_resources()` covers the resource capability set; list_changed
  notifications already flow via `notify_resource_list_changed`.)

---

## 5. EXISTING prism-mcp notification mechanism — how the send handle is obtained (THE pattern to mirror)

This is the load-bearing answer the implementer must mirror. Two existing facts:

### 5a. list_changed path (S-5.03, shipped) — peer comes from the tool-method param

`crates/prism-mcp/src/server.rs`, `reload_config` `#[tool]` method (~line 3220):

```rust
pub async fn reload_config(
    &self,
    peer: rmcp::Peer<rmcp::RoleServer>,   // rmcp injects the connection's peer here
) -> Result<CallToolResult, ErrorData> {
    let old_tables = …;
    let result = self.reload_config_core().await?;     // does the ArcSwap store() swap
    let new_tables = …;
    // dispatch AFTER the swap, using the injected peer (swap-before-notify ordering):
    resources::dispatch_hot_reload_notifications(old_tables, new_tables, &peer).await;
    Ok(result)
}
```

`dispatch_hot_reload_notifications` (`resources.rs` ~line 1114) — the actual send:

```rust
pub async fn dispatch_hot_reload_notifications(
    old_tables: Vec<String>,
    new_tables: Vec<String>,
    peer: &rmcp::service::Peer<rmcp::RoleServer>,   // ← borrowed peer is the send sink
) -> Result<(), ErrorData> {
    if old_set == new_set { return Ok(()); }
    peer.notify_resource_list_changed().await.map_err(…)?;
    peer.notify_tool_list_changed().await.map_err(…)?;
    Ok(())
}
```

**Mechanism summary:** the send handle is the `Peer<RoleServer>` that rmcp injects directly into the
`reload_config` tool method. It is used immediately, within the same call, and never stored.
`notify_resource_updated` uses the **identical** `Peer<RoleServer>` handle and the identical
`.await.map_err(…)` pattern — only the method name and param type differ.

### 5b. subscribe/notify scaffolding (BC-2.10.013, NET-NEW, partially present)

`crates/prism-mcp/src/resources/schema.rs` already defines:

- `trait SchemaChangeNotifier { async fn notify_resource_updated(&self, uri: &str) -> Result<(), ErrorData>; }`
  — the indirection that lets production wrap a real `Peer` and tests inject a mock.
- `struct SubscriberHandle { pub id: String, pub notifier: Arc<dyn SchemaChangeNotifier> }`.
- `struct SchemaSubscriberRegistry { inner: Mutex<HashMap<OrgSlug, Vec<SubscriberHandle>>> }` with
  `subscribe(client, handle)`, `unsubscribe(client, id)`, and
  `subscriber_notifiers_for(client) -> Vec<(String, Arc<dyn SchemaChangeNotifier>)>`
  (clones the `Arc`s under the lock, then releases — async calls happen lock-free).
- `async fn notify_schema_updated(client: &OrgSlug, registry: &SchemaSubscriberRegistry)` — iterates
  subscribers for that client only (DI-008 per-client scoping), calls `notify_resource_updated(uri)`
  on each, WARN-and-continues on error (DI-004 fail-open).

**What is MISSING (the implementer's job):**
1. A production `SchemaChangeNotifier` impl that wraps a stored `Peer<RoleServer>`.
2. `ServerHandler::subscribe` / `unsubscribe` overrides that build that wrapper from `context.peer`
   and register/deregister it in a `SchemaSubscriberRegistry` owned by `PrismServer`.
3. Owning the `SchemaSubscriberRegistry` on `PrismServer` (an `Arc<SchemaSubscriberRegistry>` field).
4. Calling `notify_schema_updated(&client, &registry)` from the hot-reload site (see §6).

---

## 6. Integration point — where the per-client `updated` notification fires

Config hot-reload happens in `PrismServer::reload_config` → `reload_config_core`
(`crates/prism-mcp/src/server.rs` ~line 3147), which calls
`prism_spec_engine::reload_config::reload_config(...)` to perform the **ArcSwap `store()` swap**, then
returns. The `reload_config` wrapper dispatches list_changed AFTER the swap (swap-before-notify
ordering — the same ordering the `updated` dispatch must follow).

`reload_config` already has the `Peer<RoleServer>` in scope (the injected `peer` param) AND can
compute the set of clients whose schema changed (it already computes `old_tables`/`new_tables` for the
list_changed set-comparison). To add per-client `resources/updated`:

- After `reload_config_core()` (post-swap), determine which client(s) had a schema/table-set change.
- For each changed `client`, call
  `resources::schema::notify_schema_updated(&client_slug, &self.schema_subscriber_registry).await`.
- `notify_schema_updated` fans out to that client's stored notifiers, each of which calls
  `peer.notify_resource_updated(ResourceUpdatedNotificationParam::new("prismql://schema/{client}"))`.

> The `Peer` used for `updated` is **NOT** the `reload_config` injected peer — it is the peer captured
> at each subscriber's `subscribe` time and stored in the registry (a subscriber may be a different
> connection than the one triggering the reload). This is the crucial structural difference from the
> list_changed path and the reason the `Peer` must be stored, not borrowed-and-dropped.

---

## Minimal code sketch (tailored to `PrismServer`)

```rust
// resources/schema.rs — production notifier wrapping a stored Peer<RoleServer>.
use rmcp::{service::Peer, RoleServer, model::ResourceUpdatedNotificationParam, model::ErrorData};

pub struct PeerSchemaNotifier {
    peer: Peer<RoleServer>,   // Clone + Send + Sync — captured from RequestContext.peer
}

#[async_trait::async_trait]
impl SchemaChangeNotifier for PeerSchemaNotifier {
    async fn notify_resource_updated(&self, uri: &str) -> Result<(), ErrorData> {
        self.peer
            .notify_resource_updated(ResourceUpdatedNotificationParam::new(uri.to_string()))
            .await
            // ServiceError -> ErrorData at the boundary (mirror dispatch_hot_reload_notifications):
            .map_err(|e| ErrorData::new(
                rmcp::model::ErrorCode(-32000),
                format!("notify_resource_updated failed: {e}"),
                None,
            ))
    }
}
```

```rust
// server.rs — add to `impl ServerHandler for PrismServer` (alongside list_resources etc.)
// PrismServer must own: schema_subscriber_registry: Arc<resources::schema::SchemaSubscriberRegistry>

async fn subscribe(
    &self,
    request: SubscribeRequestParams,           // plural struct
    context: RequestContext<RoleServer>,
) -> Result<(), ErrorData> {
    // Only handle prismql://schema/{client_id}; ignore other URIs gracefully.
    if let Some(client_id) = request.uri.strip_prefix("prismql://schema/") {
        // Reuse the existing validation guard (path-traversal etc.).
        let slug = prism_core::OrgSlug::new(client_id)
            .map_err(|_| ErrorData::invalid_params("E-MCP-001: invalid client_id in subscribe URI", None))?;
        let handle = resources::schema::SubscriberHandle {
            // Use a stable per-subscription id. context.id is the request id; a connection-stable
            // id is preferable. (See OPEN QUESTION below on subscription identity.)
            id: context.id.to_string(),
            notifier: std::sync::Arc::new(resources::schema::PeerSchemaNotifier {
                peer: context.peer.clone(),     // store the Clone — survives past this request
            }),
        };
        self.schema_subscriber_registry.subscribe(slug, handle);
    }
    Ok(())
}

async fn unsubscribe(
    &self,
    request: UnsubscribeRequestParams,         // plural struct
    context: RequestContext<RoleServer>,
) -> Result<(), ErrorData> {
    if let Some(client_id) = request.uri.strip_prefix("prismql://schema/") {
        if let Ok(slug) = prism_core::OrgSlug::new(client_id) {
            self.schema_subscriber_registry.unsubscribe(&slug, &context.id.to_string());
        }
    }
    Ok(())
}
```

```rust
// server.rs — at the hot-reload site (inside reload_config, AFTER reload_config_core swap):
for changed_client in changed_client_slugs {   // derived from old/new table-set diff per client
    resources::schema::notify_schema_updated(&changed_client, &self.schema_subscriber_registry)
        .await
        .ok();   // DI-004 fail-open is handled inside notify_schema_updated; .ok() at the call edge
}
```

---

## API uncertainty — flagged explicitly (do NOT guess)

1. **Subscription identity / id stability (DESIGN, not API).** `context.id` is the *request* id of the
   `subscribe` call. For `unsubscribe` to remove the right handle, the id used at subscribe MUST match
   what unsubscribe can reconstruct. The MCP `unsubscribe` request carries only `uri`, NOT the
   original request id — so keying the registry handle by `context.id` will NOT let `unsubscribe` find
   it (the unsubscribe call has a *different* request id). **The correct key is per-connection, not
   per-request.** rmcp does not expose a stable connection id on `RequestContext` directly in 1.7.0
   (fields are `ct, id, meta, extensions, peer`). Options to resolve in implementation:
   (a) key the registry by `(OrgSlug)` and on `unsubscribe` remove ALL handles for that client whose
   `peer` matches — but `Peer` is not `PartialEq`; or (b) derive a connection-stable token from
   `context.extensions` if the transport injects one; or (c) treat unsubscribe as "remove all
   subscriptions for this client from this peer" by storing the `Peer` and comparing via a
   monotonic per-connection counter assigned at subscribe. **This is a real design decision the
   implementer/architect must settle — the current `SubscriberHandle.id: String` field exists but the
   source of a stable id is unspecified.** Recommend deciding before TDD: simplest correct option is a
   `PrismServer`-owned `AtomicU64` issuing a subscription id stored in `context.extensions` is NOT
   possible (extensions are per-request), so a connection-scoped registry keyed off the transport
   session is the production-grade path. Verify whether the stdio transport exposes a session id.

2. **`McpError` vs `ErrorData`.** rmcp aliases `McpError = ErrorData`; docs render the trait return as
   `Result<(), McpError>`. The prism overrides already use `ErrorData` — they are the same type, so
   the override signature `-> Result<(), ErrorData>` is correct. (Confirmed by the existing
   `read_resource` override returning `Result<ReadResourceResult, ErrorData>`.)

3. **`notify_resource_updated` error type is `ServiceError`, not `ErrorData`.** The `Peer` method
   returns `Result<(), ServiceError>` — map it at the wrapper boundary (sketch above does this). This
   matches the existing `dispatch_hot_reload_notifications` which `.map_err(…)`s the `Peer` notify
   results into `ErrorData`.

4. **Default subscribe/unsubscribe impls return `Ok(())` (no-op).** Verified that defaults exist and
   do nothing; not verified line-by-line whether they additionally validate the URI. prism overrides
   both fully, so the default body is irrelevant to the implementation — flagged only for completeness.

Everything in §§1–6 except item 1 (a design decision, not an API fact) is confirmed against rmcp
1.7.0 docs.rs and the in-tree prism-mcp source.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Context7 query-docs | 2 | rmcp 1.7 `ServerHandler` subscribe/unsubscribe, `Peer<RoleServer>` notify methods, `SubscribeRequestParams` |
| Context7 resolve-library-id | 1 | resolve rmcp → `/websites/rs_rmcp` |
| WebFetch (docs.rs rmcp 1.7.0) | 4 | exact signatures: `ServerHandler` trait, `RequestContext`, `ResourceUpdatedNotificationParam`, `Peer` Clone/Send/Sync |
| Read (in-tree source) | 4 | `Cargo.toml`/`Cargo.lock` version pin, `resources.rs`, `resources/schema.rs`, `server.rs` reload_config + get_info + overrides |
| Grep (in-tree source) | 3 | locate `reload_config` peer injection, capabilities builder, `RequestContext`/`RoleServer` imports |
| Training data | 0 areas | none relied upon — all API facts sourced from docs.rs 1.7.0 + in-tree code |

**Total MCP tool calls:** 3 (1 Context7 resolve + 2 Context7 query-docs).
**Training data reliance:** low — every signature is cited to docs.rs rmcp 1.7.0 and cross-checked
against the existing prism-mcp implementation; the only non-cited item (subscription identity) is
explicitly flagged as an open design decision, not a guessed API fact.
