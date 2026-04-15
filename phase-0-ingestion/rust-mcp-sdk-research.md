---
name: Rust MCP SDK Research
type: research
date: 2026-04-15
phase: pre-architecture
---

# Rust MCP SDK Research

## Recommendation: Use `rmcp` (official Anthropic SDK)

**Crate:** `rmcp` v1.4.0 (released 2026-04-10)
**Repo:** https://github.com/modelcontextprotocol/rust-sdk (3,304 stars)
**License:** Apache-2.0
**Spec version:** MCP 2025-11-25

## Why rmcp

`rmcp` is the **official** Rust SDK for the Model Context Protocol, maintained under the `modelcontextprotocol` GitHub org. It is mature (v1.4.0), actively maintained, and covers all our needs:

### Feature Coverage for Prism

| Prism Requirement | rmcp Support |
|---|---|
| Stdio transport | `transport-io` feature (tokio stdin/stdout) |
| ~35 tool registration | `#[tool]` + `#[tool_router]` macros with auto JSON Schema via `schemars` |
| JSON Schema input/output | `schemars` v1.0 integration for auto-generation |
| Server notifications | `context.peer.notify_resource_list_changed()`, `notify_resource_updated()` |
| `notifications/tools/list_changed` | Built-in via `notify_tools_list_changed()` |
| Async/tokio | Native tokio runtime |
| Structured content | First-class `Content` types |

### Key API Patterns

**Server with tool macros:**
```rust
use rmcp::{tool, tool_router, ServiceExt, transport::stdio};

#[derive(Clone)]
struct PrismServer { /* ... */ }

#[tool_router(server_handler)]
impl PrismServer {
    #[tool(description = "Execute an AxiQL query")]
    async fn query(&self, #[tool(param)] query: String, #[tool(param)] clients: Option<Vec<String>>) -> String {
        // ...
    }
}

#[tokio::main]
async fn main() {
    let service = PrismServer::new();
    let server = service.serve(stdio()).await.unwrap();
    server.waiting().await.unwrap();
}
```

**Notifications:**
```rust
context.peer.notify_resource_list_changed().await?;
context.peer.notify_resource_updated(ResourceUpdatedNotificationParam {
    uri: "prism://alerts".into(),
}).await?;
```

### Features We Need
```toml
[dependencies]
rmcp = { version = "1.4", features = ["server", "transport-io", "macros", "schemars"] }
```

- `server` — server-side handler infrastructure
- `transport-io` — stdio transport (stdin/stdout)
- `macros` — `#[tool]`, `#[tool_router]` proc macros
- `schemars` — auto JSON Schema generation for tool parameters

### Dependencies rmcp Brings
- tokio (already needed)
- serde + serde_json (already needed)
- schemars v1.0 (for JSON Schema generation)
- async-trait
- tracing (already planned)

### Alternatives Considered

| Option | Verdict |
|---|---|
| `rmcp` (official) | **USE THIS** — v1.4.0, 3.3K stars, proc macros, full transport support |
| Derek-X-Wang/mcp-rust-sdk | 132 stars, community fork — superseded by official SDK |
| Roll our own | Unnecessary — rmcp is mature and covers all our needs |

## Architecture Impact

- `prism-mcp` crate depends on `rmcp` with server + stdio features
- Tool registration uses `#[tool_router]` macro — each tool is a method
- Notifications use the peer context pattern from rmcp
- No need for custom JSON-RPC implementation
- `schemars` derive macros generate tool input schemas automatically
