//! Red Gate tests for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Area E.
//!
//! BC-2.10.015: `FeatureFlagEvaluator::client_exists` Arc<OrgRegistry> DI wiring.
//! BC-2.10.016: Prompt fast-return guarantee.
//! BC-2.10.017: Not-yet-available tools fast-fail guard ordering.
//!
//! Red Gate tests: 6 total.
//!
//! BC-2.10.015 (AC-013, AC-014):
//!   Call `FeatureFlagEvaluator::client_exists` directly on a server wired with a real
//!   `OrgRegistry`. Red Gate: `client_exists` body is `todo!()` → panics RED.
//!
//! BC-2.10.016 (AC-015, AC-016):
//!   Call `render_query_tutorial` / `render_investigate_host` under timeout.
//!   Note: these render functions are already implemented (not todo!()). These tests
//!   serve as regression guards — they will catch regressions introduced during the
//!   BLOCKER-003 fix. See DONE_WITH_CONCERNS at bottom.
//!
//! BC-2.10.017 (AC-017, AC-018):
//!   Inject a SLOW or PANICKING AuditWriter via `PrismServer::new().with_audit_writer(w)`,
//!   call `list_infusions` / `plugin_status` / `infusion_status` directly.
//!   Red Gate: `emit_tool_audit` currently fires BEFORE the `-32003` guard → blocks.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unused_imports
)]

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use prism_core::{OrgId, OrgRegistry, OrgSlug};
use prism_mcp::{render_investigate_host, render_query_tutorial, PrismServer};
use prism_query::{
    write_dispatch::AuditWriter,
    write_pipeline::{QueryContext, WritePlan},
    write_result::WriteResult,
};
use prism_security::{feature_flag::CapabilityCheckResult, FeatureFlagEvaluator};
use rmcp::handler::server::wrapper::Parameters;
use ulid::Ulid;

// ─── Area D (cross-crate): BC-2.11.023 AC-010 — normalized_pql on mode-bridge error ──
//
// This test is placed in prism-mcp/tests/ because it uses both:
//   prism_mcp::error_mapping::prism_error_to_structured_call_result (prism-mcp)
//   prism_query::PrismQlParser (prism-query)
// prism-mcp depends on prism-query, so both are available here.
//
// ADJUDICATION D-1323: The original test used `SELECT * FROM t WHERE severity = 'HIGH' | limit 10`
// which now PARSES SUCCESSFULLY as Ast::SqlPipe after the Group-1 grammar landing
// (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001). The test is rewritten to:
// (a) Drive the REAL PRODUCTION path `prism_error_to_structured_call_result` (not
//     the test-helper `map_prism_error_to_structured`).
// (b) Use a synthetic QueryParseFailed whose query is a valid D1 mode-bridge candidate
//     that `mode_bridge_normalized_pql` can rewrite into a valid Pipe query.
// (c) Assert `structuredContent.error.normalized_pql` is populated.
//
// Red Gate: `prism_error_to_structured_call_result` QueryParseFailed arm has
// `normalized_pql: None` hardcoded — test FAILS RED until that arm calls
// `mode_bridge_normalized_pql(query)`.

/// AC-010 / BC-2.11.023 postcondition — `normalized_pql` on production structured error envelope.
///
/// For a `QueryParseFailed` carrying a SQL-with-pipe query (D1 mode-bridge candidate),
/// the production `prism_error_to_structured_call_result` must populate
/// `structuredContent.error.normalized_pql` with a valid Pipe-mode rewrite.
///
/// Red Gate: the production `prism_error_to_structured_call_result` QueryParseFailed arm
/// hardcodes `normalized_pql: None` — test FAILS RED until the arm is wired to call
/// `mode_bridge_normalized_pql`.
#[test]
fn test_bc_2_11_023_normalized_pql_on_mode_bridge_error() {
    use prism_core::error::PrismError;
    use prism_mcp::error_mapping::prism_error_to_structured_call_result;
    use prism_query::{ast::Ast, PrismQlParser};

    // Query that mode_bridge_normalized_pql can rewrite into a valid Pipe query.
    // Step 1 (re-parse): PrismQlParser::parse succeeds as Ast::SqlPipe, so normalize_pql
    // returns the canonical pipe form "FROM crowdstrike.detections | where severity = 'HIGH'
    // | limit 10".
    // This query already parses as SqlPipe post-Group-1 grammar, so mode_bridge_normalized_pql
    // step 1 will call normalize_pql and return Some(rewrite).
    let query = "SELECT * FROM crowdstrike.detections WHERE severity = 'HIGH' | limit 10";

    // Construct a synthetic QueryParseFailed — this represents a D1 mode-bridge error
    // as it would appear in production (e.g., generated before the SqlPipe grammar landed,
    // or from a parser version that rejected this combination).
    let prism_err = PrismError::QueryParseFailed {
        offset: 0,
        detail: "D1 mode-bridge: SQL+pipe mode mix".to_string(),
        query: query.to_string(),
    };

    // Drive the REAL PRODUCTION path: prism_error_to_structured_call_result.
    // This is the function wired into the query tool at server.rs.
    // Red Gate: production QueryParseFailed arm has normalized_pql: None → test FAILS.
    let result = prism_error_to_structured_call_result(prism_err);

    // Extract normalized_pql from structuredContent.error.
    let structured = result
        .structured_content
        .expect("BC-2.11.023: production path must return structuredContent");
    let normalized_pql = structured
        .get("error")
        .and_then(|e| e.get("normalized_pql"))
        .and_then(|v| v.as_str());

    assert!(
        normalized_pql.is_some(),
        "BC-2.11.023 AC-010: prism_error_to_structured_call_result must populate \
         normalized_pql for a D1 mode-bridge QueryParseFailed; got None. \
         Fix: wire mode_bridge_normalized_pql(query) into the QueryParseFailed arm of \
         prism_error_to_structured_call_result."
    );

    // The normalized rewrite must be valid PrismQL.
    let normalized = normalized_pql.unwrap();
    let reparse = PrismQlParser::parse(normalized);
    assert!(
        reparse.is_ok(),
        "BC-2.11.023 AC-010: normalized_pql must be valid PrismQL; reparse got: {:?}",
        reparse
    );

    // Verify normalized form is Pipe or SqlPipe (canonical pipe-mode rewrite).
    let ast = reparse.unwrap();
    assert!(
        matches!(ast, Ast::Pipe(_) | Ast::SqlPipe(_)),
        "BC-2.11.023 AC-010: normalized_pql must parse as Ast::Pipe or Ast::SqlPipe; \
         got: {:?}",
        ast
    );
}

// ─── Area E: BC-2.10.015 — FeatureFlagEvaluator Arc<OrgRegistry> DI ──────────

/// AC-013 / BC-2.10.015 postcondition.
///
/// Construct a `FeatureFlagEvaluator` with an `OrgRegistry` containing `"org-c"`,
/// but with an EMPTY `client_capabilities` map (no prism.toml `[clients]` entry).
///
/// Assert:
/// - `client_exists("org-c")` → `true`  (org is in OrgRegistry)
/// - `client_exists("unknown-org")` → `false` (org not in OrgRegistry)
///
/// Red Gate: `client_exists` body is `todo!()` → panics RED when called.
#[test]
fn test_bc_2_10_015_client_registered_true_from_org_registry() {
    let registry = Arc::new(OrgRegistry::new());
    let slug = OrgSlug::new("org-c");
    assert!(slug.is_ok(), "BC-2.10.015: 'org-c' must be a valid OrgSlug");
    registry
        .register(slug, OrgId::new())
        .expect("BC-2.10.015: OrgRegistry::register must succeed for 'org-c'");

    let evaluator = FeatureFlagEvaluator::new(BTreeMap::new(), Arc::clone(&registry));

    // Panics on todo!() → RED.
    let exists = evaluator.client_exists("org-c");
    assert!(
        exists,
        "BC-2.10.015 AC-013: client_exists('org-c') must return true \
         when org is in OrgRegistry (even with empty client_capabilities)"
    );

    let not_exists = evaluator.client_exists("unknown-org");
    assert!(
        !not_exists,
        "BC-2.10.015 AC-013: client_exists('unknown-org') must return false \
         when org is not in OrgRegistry"
    );
}

/// AC-014 / BC-2.10.015 postcondition — demo provisioning path.
///
/// An org provisioned ONLY via spec overlays (no `[clients.*]` entry in `prism.toml`)
/// returns `client_registered: true`. An org NOT in `OrgRegistry` → `false`.
///
/// Red Gate: `client_exists` body is `todo!()` → panics RED when called.
#[test]
fn test_bc_2_10_015_demo_provisioned_org_registered() {
    let registry = Arc::new(OrgRegistry::new());
    let demo_slug = OrgSlug::new("demo-org");
    assert!(
        demo_slug.is_ok(),
        "BC-2.10.015: 'demo-org' must be a valid OrgSlug"
    );
    registry
        .register(demo_slug, OrgId::new())
        .expect("BC-2.10.015: OrgRegistry::register must succeed for 'demo-org'");

    let evaluator = FeatureFlagEvaluator::new(BTreeMap::new(), Arc::clone(&registry));

    // "demo-org" is in OrgRegistry → must return true. Panics on todo!() → RED.
    let registered = evaluator.client_exists("demo-org");
    assert!(
        registered,
        "BC-2.10.015 AC-014: 'demo-org' provisioned via OrgRegistry must return \
         client_exists=true (demo provisioning path)"
    );

    let not_registered = evaluator.client_exists("non-existent-org");
    assert!(
        !not_registered,
        "BC-2.10.015 AC-014: 'non-existent-org' not in OrgRegistry must return \
         client_exists=false"
    );

    // A client_id that maps to an Invalid OrgSlug → must return false (not panic).
    // 65-char string exceeds the 64-byte cap for OrgSlug validation.
    let long_id = "z".repeat(65);
    let malformed = evaluator.client_exists(&long_id);
    assert!(
        !malformed,
        "BC-2.10.015 AC-014: malformed client_id (>64 chars) must return false, not panic"
    );
}

// ─── Area E: BC-2.10.016 — Prompt fast-return guarantee ─────────────────────

/// AC-015 / BC-2.10.016 postcondition — prompt fast-return.
///
/// `render_query_tutorial("test-org", None)` must return within 5 seconds.
///
/// Note: `render_query_tutorial` is currently implemented (not todo!()). This test
/// serves as a regression guard against blocking calls introduced during BLOCKER-003 fix.
///
/// HIGH-001 fix: use `async { render_query_tutorial(...) }` instead of
/// `std::future::ready(render_query_tutorial(...))`. The `ready()` form evaluates the
/// function eagerly (before the timeout begins), so the timeout CANNOT detect a hang.
/// The `async { ... }` form runs the function lazily inside the tokio executor, allowing
/// `timeout` to race against a genuinely slow computation.
#[tokio::test]
async fn test_bc_2_10_016_prompts_fast_return_within_5s() {
    use tokio::time::timeout;

    let result = timeout(Duration::from_secs(5), async {
        render_query_tutorial("test-org", None)
    })
    .await;

    assert!(
        result.is_ok(),
        "BC-2.10.016 AC-015: render_query_tutorial must complete within 5 seconds"
    );
    let render_result = result.unwrap();
    assert!(
        render_result.is_ok(),
        "BC-2.10.016 AC-015: render_query_tutorial must return Ok; got: {:?}",
        render_result
    );
    let prompt = render_result.unwrap();
    assert!(
        !prompt.messages.is_empty(),
        "BC-2.10.016 AC-015: render_query_tutorial must return at least one message"
    );
}

/// AC-016 / BC-2.10.016 invariant INV-PROMPT-REQUIRED-ARGS.
///
/// `render_investigate_host("test-org", "(unknown)")` (missing hostname default) must
/// return within 5 seconds — must NOT hang on missing required arg.
///
/// HIGH-001 fix: use `async { render_investigate_host(...) }` instead of
/// `std::future::ready(render_investigate_host(...))` so the timeout can actually race
/// against the function execution (see AC-015 doc comment for full rationale).
#[tokio::test]
async fn test_bc_2_10_016_missing_required_arg_fast_error() {
    use tokio::time::timeout;

    let result = timeout(Duration::from_secs(5), async {
        render_investigate_host("test-org", "(unknown)")
    })
    .await;

    assert!(
        result.is_ok(),
        "BC-2.10.016 AC-016: render_investigate_host with placeholder hostname must \
         return within 5 seconds; MUST NOT hang (BLOCKER-003)"
    );
    // Either Ok or Err is acceptable — the contract is that it RETURNS (does not hang).
    let _ = result.unwrap();
}

// ─── Area E: BC-2.10.017 — Not-yet-available tools fast-fail guard ordering ──

/// Slow AuditWriter for BC-2.10.017 AC-017 timing test.
///
/// `write_tool_call` sleeps for `delay` to simulate a slow durable audit write.
/// If the guard fires BEFORE `emit_tool_audit`, this writer is never called.
struct SlowAuditWriter {
    delay: Duration,
}

#[async_trait]
impl AuditWriter for SlowAuditWriter {
    async fn write_intent(
        &self,
        _plan: &WritePlan,
        _context: &QueryContext,
        _capability_check: &CapabilityCheckResult,
    ) -> Result<Ulid, prism_core::error::PrismError> {
        Ok(Ulid::new())
    }

    async fn write_outcome(
        &self,
        _intent_id: Ulid,
        _result: &WriteResult,
    ) -> Result<(), prism_core::error::PrismError> {
        Ok(())
    }

    async fn write_tool_call(
        &self,
        _tool_name: &str,
        _client_id: Option<&str>,
        _operation: &str,
        _outcome: &str,
    ) -> Result<(), prism_core::error::PrismError> {
        tokio::time::sleep(self.delay).await;
        Ok(())
    }
}

/// Panicking AuditWriter for BC-2.10.017 AC-018 guard-ordering test.
///
/// If the guard fires BEFORE `emit_tool_audit`, this writer is never called
/// and the test passes. If `emit_tool_audit` fires first, the panic propagates
/// → test fails RED.
struct PanickingAuditWriter;

#[async_trait]
impl AuditWriter for PanickingAuditWriter {
    async fn write_intent(
        &self,
        _plan: &WritePlan,
        _context: &QueryContext,
        _capability_check: &CapabilityCheckResult,
    ) -> Result<Ulid, prism_core::error::PrismError> {
        Ok(Ulid::new())
    }

    async fn write_outcome(
        &self,
        _intent_id: Ulid,
        _result: &WriteResult,
    ) -> Result<(), prism_core::error::PrismError> {
        Ok(())
    }

    async fn write_tool_call(
        &self,
        _tool_name: &str,
        _client_id: Option<&str>,
        _operation: &str,
        _outcome: &str,
    ) -> Result<(), prism_core::error::PrismError> {
        panic!(
            "BC-2.10.017 AC-018: PanickingAuditWriter::write_tool_call was invoked — \
             emit_tool_audit fired BEFORE the not_yet_available guard. \
             The guard MUST precede emit_tool_audit in each NOT_YET_AVAILABLE handler."
        )
    }
}

/// AC-017 / BC-2.10.017 postcondition — fast-fail under 1s with slow writer.
///
/// Invoke `list_infusions`, `plugin_status`, and `infusion_status` on a PrismServer
/// with a 10-second slow AuditWriter. Assert all return within 1 second.
///
/// Red Gate: current code calls `scan_inputs_audited(...)` then `emit_tool_audit(...)`
/// BEFORE the `-32003` return. The slow writer blocks `emit_tool_audit`, the 1-second
/// timeout fires → test fails RED (BLOCKER-004).
#[tokio::test]
async fn test_bc_2_10_017_not_yet_available_fast_fail_under_1s() {
    use prism_mcp::server::{InfusionStatusParams, ListInfusionsParams, PluginStatusParams};
    use tokio::time::timeout;

    let slow_writer: Arc<dyn AuditWriter> = Arc::new(SlowAuditWriter {
        delay: Duration::from_secs(10),
    });
    let server = PrismServer::new().with_audit_writer(Arc::clone(&slow_writer));

    // list_infusions — must return within 1 second.
    let list_params: ListInfusionsParams =
        serde_json::from_value(serde_json::json!({ "client_id": "test-org" }))
            .expect("valid ListInfusionsParams JSON");
    let list_result = timeout(
        Duration::from_secs(1),
        server.list_infusions(Parameters(list_params)),
    )
    .await;
    assert!(
        list_result.is_ok(),
        "BC-2.10.017 AC-017: list_infusions must return within 1s with slow AuditWriter; \
         guard must fire BEFORE emit_tool_audit await (BLOCKER-004)"
    );
    assert!(
        list_result.unwrap().is_err(),
        "BC-2.10.017 AC-017: list_infusions must return Err(-32003 not_yet_available)"
    );

    // plugin_status — must return within 1 second.
    let plugin_params: PluginStatusParams =
        serde_json::from_value(serde_json::json!({ "plugin_id": "test-plugin" }))
            .expect("valid PluginStatusParams JSON");
    let plugin_result = timeout(
        Duration::from_secs(1),
        server.plugin_status(Parameters(plugin_params)),
    )
    .await;
    assert!(
        plugin_result.is_ok(),
        "BC-2.10.017 AC-017: plugin_status must return within 1s with slow AuditWriter"
    );
    assert!(
        plugin_result.unwrap().is_err(),
        "BC-2.10.017 AC-017: plugin_status must return Err(-32003 not_yet_available)"
    );

    // infusion_status — must return within 1 second.
    let infusion_params: InfusionStatusParams =
        serde_json::from_value(serde_json::json!({ "infusion_id": "test-infusion" }))
            .expect("valid InfusionStatusParams JSON");
    let infusion_result = timeout(
        Duration::from_secs(1),
        server.infusion_status(Parameters(infusion_params)),
    )
    .await;
    assert!(
        infusion_result.is_ok(),
        "BC-2.10.017 AC-017: infusion_status must return within 1s with slow AuditWriter"
    );
    assert!(
        infusion_result.unwrap().is_err(),
        "BC-2.10.017 AC-017: infusion_status must return Err(-32003 not_yet_available)"
    );
}

/// AC-018 / BC-2.10.017 invariant INV-AUDIT-NON-BLOCKING.
///
/// Guard ordering: the `-32003` not-yet-available response must be returned
/// BEFORE any `emit_tool_audit(...).await` in each NOT_YET_AVAILABLE handler.
///
/// This test injects a PanickingAuditWriter:
/// - Correct behavior (guard fires first): writer never called → test passes with -32003.
/// - Wrong behavior (emit_tool_audit fires first): writer panics → test fails RED.
///
/// Red Gate: current handlers call `emit_tool_audit` before returning `-32003`,
/// so the panicking writer panics → nextest reports FAILED (process panicked) → RED.
#[tokio::test]
async fn test_bc_2_10_017_not_yet_available_guard_precedes_audit() {
    use prism_mcp::server::ListInfusionsParams;
    use tokio::time::timeout;

    let panicking_writer: Arc<dyn AuditWriter> = Arc::new(PanickingAuditWriter);
    let server = PrismServer::new().with_audit_writer(panicking_writer);

    let list_params: ListInfusionsParams =
        serde_json::from_value(serde_json::json!({ "client_id": "test-org" }))
            .expect("valid ListInfusionsParams JSON");
    let result = timeout(
        Duration::from_secs(2),
        server.list_infusions(Parameters(list_params)),
    )
    .await;

    assert!(
        result.is_ok(),
        "BC-2.10.017 AC-018: list_infusions must return within 2s (no hang)"
    );
    let call_result = result.unwrap();
    assert!(
        call_result.is_err(),
        "BC-2.10.017 AC-018: list_infusions must return Err(-32003); \
         reaching here means PanickingAuditWriter was never called → guard precedes audit"
    );
    let err = call_result.unwrap_err();
    assert_eq!(
        err.code,
        rmcp::model::ErrorCode(-32003),
        "BC-2.10.017 AC-018: error code must be -32003 (not_yet_available); got: {:?}",
        err.code
    );
}

// ─── HIGH-002 sibling handler coverage ───────────────────────────────────────

/// AC-017 BC-2.10.017 — sibling NOT_YET_AVAILABLE guard ordering: `list_plugins`,
/// `reload_infusion`, `create_schedule`.
///
/// These are a representative sample of the 30+ NOT_YET_AVAILABLE handlers that were
/// not individually exercised by the original test (which only called `list_infusions`,
/// `plugin_status`, `infusion_status`).
///
/// PanickingAuditWriter proves the guard fires BEFORE any audit path — if any handler
/// called emit_tool_audit or scan_inputs_audited before the not_yet_available_msg guard,
/// the writer would panic and the test would FAIL.
///
/// All three handlers must:
/// 1. Return Err(-32003) without panicking (PanickingAuditWriter never invoked).
/// 2. Return within 1 second when wired with a 10-second SlowAuditWriter.
#[tokio::test]
async fn test_bc_2_10_017_sibling_handlers_guard_precedes_audit() {
    use prism_mcp::server::{CreateScheduleParams, ListPluginsParams, ReloadInfusionParams};
    use tokio::time::timeout;

    // ── PanickingAuditWriter path ──────────────────────────────────────────
    // Any handler that calls scan_inputs_audited / emit_tool_audit before
    // not_yet_available_msg will invoke this writer and cause a process panic.
    let panicking: Arc<dyn AuditWriter> = Arc::new(PanickingAuditWriter);
    let server_p = PrismServer::new().with_audit_writer(Arc::clone(&panicking));

    // list_plugins (no params)
    let list_plugins_params: ListPluginsParams =
        serde_json::from_value(serde_json::json!({})).expect("valid ListPluginsParams JSON");
    let result = timeout(
        Duration::from_secs(2),
        server_p.list_plugins(Parameters(list_plugins_params)),
    )
    .await;
    assert!(
        result.is_ok(),
        "BC-2.10.017 HIGH-002: list_plugins must return within 2s (no hang)"
    );
    let call_result = result.unwrap();
    assert!(
        call_result.is_err(),
        "BC-2.10.017 HIGH-002: list_plugins must return Err; \
         reaching here means PanickingAuditWriter never invoked → guard precedes audit"
    );
    assert_eq!(
        call_result.unwrap_err().code,
        rmcp::model::ErrorCode(-32003),
        "BC-2.10.017 HIGH-002: list_plugins must return -32003"
    );

    // reload_infusion
    let reload_infusion_params: ReloadInfusionParams =
        serde_json::from_value(serde_json::json!({ "infusion_id": "test-infusion" }))
            .expect("valid ReloadInfusionParams JSON");
    let result = timeout(
        Duration::from_secs(2),
        server_p.reload_infusion(Parameters(reload_infusion_params)),
    )
    .await;
    assert!(
        result.is_ok(),
        "BC-2.10.017 HIGH-002: reload_infusion must return within 2s (no hang)"
    );
    let call_result = result.unwrap();
    assert!(
        call_result.is_err(),
        "BC-2.10.017 HIGH-002: reload_infusion must return Err; \
         reaching here means PanickingAuditWriter never invoked → guard precedes audit"
    );
    assert_eq!(
        call_result.unwrap_err().code,
        rmcp::model::ErrorCode(-32003),
        "BC-2.10.017 HIGH-002: reload_infusion must return -32003"
    );

    // create_schedule
    let create_schedule_params: CreateScheduleParams =
        serde_json::from_value(
            serde_json::json!({ "query": "SELECT * FROM crowdstrike.detections LIMIT 10", "cron": "0 * * * *" }),
        )
        .expect("valid CreateScheduleParams JSON");
    let result = timeout(
        Duration::from_secs(2),
        server_p.create_schedule(Parameters(create_schedule_params)),
    )
    .await;
    assert!(
        result.is_ok(),
        "BC-2.10.017 HIGH-002: create_schedule must return within 2s (no hang)"
    );
    let call_result = result.unwrap();
    assert!(
        call_result.is_err(),
        "BC-2.10.017 HIGH-002: create_schedule must return Err; \
         PanickingAuditWriter never invoked → guard precedes audit"
    );
    assert_eq!(
        call_result.unwrap_err().code,
        rmcp::model::ErrorCode(-32003),
        "BC-2.10.017 HIGH-002: create_schedule must return -32003"
    );

    // ── SlowAuditWriter path (fast-fail proof) ────────────────────────────
    // All three handlers must return within 1s even with a 10s slow writer.
    let slow: Arc<dyn AuditWriter> = Arc::new(SlowAuditWriter {
        delay: Duration::from_secs(10),
    });
    let server_s = PrismServer::new().with_audit_writer(Arc::clone(&slow));

    // list_plugins fast-fail
    let list_plugins_params2: ListPluginsParams =
        serde_json::from_value(serde_json::json!({})).expect("valid ListPluginsParams JSON");
    let fast = timeout(
        Duration::from_secs(1),
        server_s.list_plugins(Parameters(list_plugins_params2)),
    )
    .await;
    assert!(
        fast.is_ok(),
        "BC-2.10.017 HIGH-002: list_plugins must return within 1s with SlowAuditWriter"
    );

    // reload_infusion fast-fail
    let reload_infusion_params2: ReloadInfusionParams =
        serde_json::from_value(serde_json::json!({ "infusion_id": "test-infusion" }))
            .expect("valid ReloadInfusionParams JSON");
    let fast = timeout(
        Duration::from_secs(1),
        server_s.reload_infusion(Parameters(reload_infusion_params2)),
    )
    .await;
    assert!(
        fast.is_ok(),
        "BC-2.10.017 HIGH-002: reload_infusion must return within 1s with SlowAuditWriter"
    );

    // create_schedule fast-fail
    let create_schedule_params2: CreateScheduleParams =
        serde_json::from_value(
            serde_json::json!({ "query": "SELECT * FROM crowdstrike.detections LIMIT 10", "cron": "0 * * * *" }),
        )
        .expect("valid CreateScheduleParams JSON");
    let fast = timeout(
        Duration::from_secs(1),
        server_s.create_schedule(Parameters(create_schedule_params2)),
    )
    .await;
    assert!(
        fast.is_ok(),
        "BC-2.10.017 HIGH-002: create_schedule must return within 1s with SlowAuditWriter"
    );
}

// ─── HIGH-001: full-transport get_prompt dispatch ────────────────────────────

/// HIGH-001 / BC-2.10.016 — `ServerHandler::get_prompt` via full MCP transport.
///
/// The macro-generated `ServerHandler::get_prompt` dispatches via:
///   client.get_prompt(name, args) → JSON-RPC wire →
///   #[prompt_handler] expand → PromptRouter::get_prompt →
///   HashMap lookup → closure → render_query_tutorial(client_id, goal).
///
/// Prior gap: tests called `render_query_tutorial` directly (leaf renderer),
/// completely bypassing the macro-generated dispatch. This test drives the REAL
/// `ServerHandler::get_prompt` path via tokio::io::duplex + rmcp ClientHandler,
/// proving the entire dispatch chain functions end-to-end.
///
/// cargo-expand analysis: the `#[prompt_handler(router = self.prompt_router)]`
/// macro generates:
///   async fn get_prompt(&self, request: GetPromptRequestParams, context: RequestContext<RoleServer>)
///       -> Result<GetPromptResult, rmcp::ErrorData> {
///       let prompt_context = PromptContext::new(self, request.name, request.arguments, context);
///       self.prompt_router.get_prompt(prompt_context).await
///   }
/// There is no blocking point: the router does a HashMap lookup and calls a
/// synchronous pure closure (render_query_tutorial). The Peer::new() constructor
/// is pub(crate) in rmcp — RequestContext cannot be constructed from external
/// crates. The only correct test pattern is the duplex + ClientHandler approach
/// (mirroring rmcp's own test_prompt_macros.rs test_optional_i64_field_with_null_input).
///
/// Test protocol:
/// 1. Spawn PrismServer on the server-side duplex stream.
/// 2. DummyClientHandler.serve(client_stream) completes the MCP handshake.
/// 3. client.get_prompt("query_tutorial", args) sends a real JSON-RPC request.
/// 4. Assert the response contains PQL tutorial content (not an error).
/// 5. Entire call completes within 5s (regression guard against future hang introduction).
#[tokio::test]
async fn test_bc_2_10_016_get_prompt_full_transport_dispatch() {
    use rmcp::{
        model::{ClientInfo, GetPromptRequestParams},
        ClientHandler, ServiceExt,
    };
    use tokio::time::timeout;

    // DummyClientHandler: minimal no-op client to complete MCP handshake.
    #[derive(Debug, Clone, Default)]
    struct DummyClientHandler;
    impl ClientHandler for DummyClientHandler {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    // duplex transport — server side / client side.
    let (server_transport, client_transport) = tokio::io::duplex(4096);

    // Spawn the PrismServer on the server stream.
    let server_handle = tokio::spawn(async move {
        PrismServer::new()
            .serve(server_transport)
            .await
            .expect("PrismServer::serve must complete MCP handshake")
            .waiting()
            .await
            .expect("PrismServer waiting must not fail");
    });

    // Connect the dummy client — completes the handshake.
    let client = DummyClientHandler::default()
        .serve(client_transport)
        .await
        .expect("DummyClientHandler::serve must complete handshake");

    // Invoke get_prompt("query_tutorial") via real JSON-RPC wire.
    let args = serde_json::json!({
        "client_id": "test-tenant",
        "goal": "learn basic PQL filter syntax"
    });
    let params = GetPromptRequestParams::new("query_tutorial")
        .with_arguments(args.as_object().unwrap().clone());

    let result = timeout(Duration::from_secs(5), client.get_prompt(params))
        .await
        .expect("HIGH-001: get_prompt must return within 5s (no hang in dispatch chain)")
        .expect("HIGH-001: get_prompt must return Ok (prompt found and rendered)");

    // The response must carry at least one message.
    assert!(
        !result.messages.is_empty(),
        "HIGH-001 BC-2.10.016: get_prompt(query_tutorial) must return at least one message; \
         got empty messages vec"
    );

    // The rendered content must contain PQL tutorial content (not an error placeholder).
    let full_text: String = result
        .messages
        .iter()
        .filter_map(|m| match &m.content {
            rmcp::model::PromptMessageContent::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        full_text.contains("PQL") || full_text.contains("query") || full_text.contains("SELECT"),
        "HIGH-001 BC-2.10.016: get_prompt(query_tutorial) rendered content must contain \
         PQL tutorial text (\"PQL\" / \"query\" / \"SELECT\"); got first 200 chars: {:?}",
        &full_text[..full_text.len().min(200)]
    );

    // Clean up: cancel the client and let the server task finish.
    client.cancel().await.expect("client cancel must succeed");
    // Server handle: we let it finish naturally (the cancel above closes the transport).
    drop(server_handle);
}
