//! RG-001 + RG-002: S-REL-AGENT-VERSION-001 — Agent-Facing Version Identity (prism-mcp)
//!
//! These tests gate Surface A: `PrismServer::get_info()` must return the injected
//! `product_version`, not the stale hardcoded `"0.1.0"` string.
//!
//! Authority: ADR-064 v1.9 D4 §Surface A; wire-shape discipline (CLAUDE.md §Conventions
//! §Wire-shape assertion discipline).
//!
//! # Architecture compliance
//!
//! prism-mcp MUST NOT have a `build.rs` and MUST NOT contain `env!("PRISM_VERSION")` in
//! any source file (ADR-064 D4 §Architecture Compliance Rules: "No CI env-var resolution
//! logic in library crates"). The product version is threaded from prism-bin boot step 9
//! via `PrismServer::with_deps(... product_version: env!("PRISM_VERSION"))`. Tests here
//! therefore assert against `"0.0.0-test"` — the fixed default that `PrismServer::new()`
//! will inject after implementation (ADR-064 D4 §Surface A: "`PrismServer::new` uses
//! `product_version: \"0.0.0-test\"` default — distinguishable from any release, indicates
//! test context").
//!
//! # Red Gate state (pre-implementation)
//!
//! Both tests fail with ASSERTION FAILURES (not compile errors) because:
//! - `PrismServer::new()` compiles and runs successfully (no product_version field yet)
//! - `get_info()` returns `Implementation::new("prism", "0.1.0")` — the stale hardcoded string
//! - All assertions expecting `"0.0.0-test"` therefore fail
//!
//! # Green Gate state (post-implementation)
//!
//! Both tests pass after the implementer completes Tasks 7–10 (Surface A):
//! - `PrismServer` struct gains `product_version: &'static str` field
//! - `PrismServer::new()` sets `product_version: "0.0.0-test"`
//! - `get_info()` returns `Implementation::new("prism", self.product_version)` = `"0.0.0-test"`
//!
//! # Test naming
//!
//! Follows BC-based traceability convention: `test_<story_id>_<descriptor>` since this
//! story has no BC (build-infrastructure change, conforming per S-REL-002 precedent).

use prism_mcp::PrismServer;
use rmcp::ServerHandler;

/// RG-001: `PrismServer::get_info()` Rust struct `server_info.version` equals the injected
/// `product_version` (`"0.0.0-test"` via `PrismServer::new()`), NOT the stale hardcoded `"0.1.0"`.
///
/// Traces to: ADR-064 v1.9 D4 §Surface A — `PrismServer::get_info` row:
///   Before: `Implementation::new("prism", "0.1.0")`
///   After:  `Implementation::new("prism", self.product_version)`
///
/// RED state: `get_info()` returns `"0.1.0"` → `assert_eq!` fails.
/// GREEN state: `get_info()` returns `"0.0.0-test"` → passes.
#[test]
fn test_server_info_version_is_product_version() {
    let server = PrismServer::new();
    let info = server.get_info();

    // After implementation: PrismServer::new() sets product_version: "0.0.0-test" and
    // get_info() returns Implementation::new("prism", self.product_version) = "0.0.0-test".
    // Before implementation: get_info() returns "0.1.0" — this assertion FAILS (RED gate).
    assert_eq!(
        info.server_info.version, "0.0.0-test",
        "serverInfo.version must equal the injected product_version \
         (\"0.0.0-test\" via PrismServer::new() default per ADR-064 D4 §Surface A); \
         got {:?}. Fails RED because current get_info returns the stale hardcoded \"0.1.0\".",
        info.server_info.version
    );
}

/// RG-002: Wire-shape assertion — the serialized MCP `initialize` response JSON
/// `serverInfo.version` field equals `"0.0.0-test"` (the injected `product_version`)
/// and does NOT equal the stale hardcoded `"0.1.0"`.
///
/// Wire-shape discipline (CLAUDE.md §Conventions §Wire-shape assertion discipline):
/// Any test covering an MCP-visible surface MUST assert on the serialized JSON bytes
/// the LLM agent consumes, not only the Rust struct (origin: live-audit [C3]/[H20]).
///
/// The MCP `initialize` response structure (rmcp `InitializeResult`, alias `ServerInfo`):
/// ```json
/// {
///   "protocolVersion": "...",
///   "capabilities": { ... },
///   "serverInfo": { "name": "prism", "version": "..." }
/// }
/// ```
/// (`#[serde(rename_all = "camelCase")]` on `InitializeResult` → `serverInfo` key)
///
/// Traces to: ADR-064 v1.9 D4 §Surface A; S-REL-AGENT-VERSION-001 §Red Gate Test List RG-002.
///
/// RED state: `serverInfo.version` on wire = `"0.1.0"` → first `assert_ne!` fails.
/// GREEN state: `serverInfo.version` on wire = `"0.0.0-test"` → both assertions pass.
#[test]
fn test_server_info_version_mcp_initialize_wire_shape() {
    let server = PrismServer::new();
    let info = server.get_info();

    // Serialize the MCP initialize response to JSON — this is the wire format the LLM agent
    // reads. `ServerInfo` = `InitializeResult` which derives `Serialize` (rmcp model.rs).
    let json = serde_json::to_value(&info)
        .expect("ServerInfo (InitializeResult) must serialize to JSON; it derives Serialize");

    // Navigate the JSON structure to `serverInfo.version`.
    // `InitializeResult` uses `#[serde(rename_all = "camelCase")]` so `server_info` → `serverInfo`.
    let wire_version = json["serverInfo"]["version"]
        .as_str()
        .expect("serverInfo.version must be a non-null string in the MCP initialize response JSON");

    // PRIMARY RED assertion: the wire output must NOT contain the stale hardcoded version.
    // Before implementation: get_info() returns "0.1.0" → this fails (RED gate).
    assert_ne!(
        wire_version, "0.1.0",
        "Wire output serverInfo.version must NOT be the stale hardcoded \"0.1.0\"; \
         current code returns this value, which is the RED state this test detects. \
         After Surface A implementation, get_info() returns self.product_version."
    );

    // GREEN assertion: the wire output must contain the injected product_version.
    // After implementation: PrismServer::new() sets product_version: "0.0.0-test".
    assert_eq!(
        wire_version, "0.0.0-test",
        "Wire output serverInfo.version must equal the injected product_version \
         (\"0.0.0-test\" via PrismServer::new() per ADR-064 D4 §Surface A); \
         got {:?}",
        wire_version
    );
}
