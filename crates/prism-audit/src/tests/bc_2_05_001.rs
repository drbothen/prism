//! Tests for BC-2.05.001 — Every MCP Tool Invocation Produces Exactly One
//! Audit Entry (Fail-Closed for Writes).
//!
//! Postconditions tested:
//!   - Exactly one `AuditEntry` is written to `audit_buffer` per tool invocation.
//!   - Write tools: if `emit()` fails, the write is aborted and `E-AUDIT-001` returned.
//!   - Read tools: if `emit()` fails, the read proceeds (fail-open).
//!
//! AC-1: read tool, success → exactly one entry in audit_buffer.
//! AC-2: write tool, emit failure → write aborted, `E-AUDIT-001` returned.
//! EC-001: fail-closed — inner handler NOT called when audit fails for write tool.
//! EC-002: read tool, audit failure → operation proceeds (result returned).

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use prism_core::PrismError;
use tower::{Layer, Service};

use crate::audit_emitter::{
    AuditEmitterLayer, AuditedRequest, AuditedResponse, ToolClass, ToolClassificationRegistry,
};
use crate::tests::helpers::{
    count_audit_entries, make_request, AlwaysSucceedService, FailingBackend, MemBackend,
};

// ── Helper: invoke a service for tests (poll_ready + call) ───────────────────

async fn invoke<S>(svc: &mut S, req: AuditedRequest) -> Result<AuditedResponse, PrismError>
where
    S: Service<AuditedRequest, Response = AuditedResponse, Error = PrismError>,
    S::Future: Send,
{
    std::future::poll_fn(|cx| svc.poll_ready(cx))
        .await
        .expect("poll_ready failed");
    svc.call(req).await
}

// ── AC-1: read tool, successful invocation → exactly one audit entry ──────────

/// AC-1 (BC-2.05.001): A successful read tool invocation produces exactly one
/// audit entry in `audit_buffer` after the call completes.
#[tokio::test]
async fn test_BC_2_05_001_read_tool_success_produces_exactly_one_entry() {
    let backend = MemBackend::new();
    let mut registry: ToolClassificationRegistry = HashMap::new();
    registry.insert("query_crowdstrike_alerts", ToolClass::ReadTool);

    let layer = AuditEmitterLayer::new(Arc::new(backend.clone()), Arc::new(registry));
    let mut svc = layer.layer(AlwaysSucceedService);

    let result = invoke(&mut svc, make_request("query_crowdstrike_alerts")).await;

    assert!(
        result.is_ok(),
        "read tool invocation should succeed: {result:?}"
    );

    let entry_count = count_audit_entries(&backend);
    assert_eq!(
        entry_count, 1,
        "expected exactly 1 audit entry after a successful read tool call, found {entry_count}"
    );
}

// ── AC-2: write tool, emit failure → write aborted, E-AUDIT-001 ──────────────

/// AC-2 (BC-2.05.001): When `emit()` fails for a write tool (storage unavailable),
/// the write is aborted before execution and `PrismError::AuditPersistenceFailed`
/// (E-AUDIT-001) is returned to the caller.
#[tokio::test]
async fn test_BC_2_05_001_write_tool_emit_failure_aborts_and_returns_E_AUDIT_001() {
    let backend = FailingBackend::new();
    let mut registry: ToolClassificationRegistry = HashMap::new();
    registry.insert("crowdstrike_contain_host", ToolClass::WriteTool);

    let layer = AuditEmitterLayer::new(Arc::new(backend), Arc::new(registry));
    let mut svc = layer.layer(AlwaysSucceedService);

    let result = invoke(&mut svc, make_request("crowdstrike_contain_host")).await;

    assert!(
        result.is_err(),
        "write tool with failing audit should return Err, got Ok"
    );
    let err = result.unwrap_err();
    assert_eq!(
        err,
        PrismError::AuditPersistenceFailed,
        "expected PrismError::AuditPersistenceFailed (E-AUDIT-001), got: {err:?}"
    );

    // BC-2.05.001 error case: Display must contain the structured error code.
    let display = err.to_string();
    assert!(
        display.contains("E-AUDIT-001"),
        "PrismError::AuditPersistenceFailed Display must contain 'E-AUDIT-001', got: {display}"
    );
}

// ── EC-001: fail-closed — inner handler never called when audit fails ─────────

/// EC-001 (BC-2.05.001): When audit emission fails for a write tool, the inner
/// handler is NEVER called — no write side effect occurs.
///
/// We use a spy service that panics if called.
#[tokio::test]
async fn test_BC_2_05_001_write_tool_emit_failure_inner_handler_never_called() {
    #[derive(Clone)]
    struct PanicOnCallService;

    impl Service<AuditedRequest> for PanicOnCallService {
        type Response = AuditedResponse;
        type Error = PrismError;
        type Future = Pin<Box<dyn Future<Output = Result<AuditedResponse, PrismError>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: AuditedRequest) -> Self::Future {
            panic!(
                "inner handler must NOT be called when audit emission fails for a write tool \
                 (EC-001 / BC-2.05.001 fail-closed contract)"
            )
        }
    }

    let backend = FailingBackend::new();
    let mut registry: ToolClassificationRegistry = HashMap::new();
    registry.insert("crowdstrike_contain_host", ToolClass::WriteTool);

    let layer = AuditEmitterLayer::new(Arc::new(backend), Arc::new(registry));
    let mut svc = layer.layer(PanicOnCallService);

    let result = invoke(&mut svc, make_request("crowdstrike_contain_host")).await;
    assert!(result.is_err(), "should return Err(AuditPersistenceFailed)");
}

// ── EC-002: read tool, audit failure → operation proceeds ────────────────────

/// EC-002 (BC-2.05.001): When audit emission fails for a read tool, the read
/// still proceeds and returns a successful result.
#[tokio::test]
async fn test_BC_2_05_001_read_tool_emit_failure_operation_proceeds() {
    let backend = FailingBackend::new();
    let mut registry: ToolClassificationRegistry = HashMap::new();
    registry.insert("query_crowdstrike_alerts", ToolClass::ReadTool);

    let layer = AuditEmitterLayer::new(Arc::new(backend), Arc::new(registry));
    let mut svc = layer.layer(AlwaysSucceedService);

    let result = invoke(&mut svc, make_request("query_crowdstrike_alerts")).await;

    assert!(
        result.is_ok(),
        "read tool should proceed even when audit emission fails, got: {result:?}"
    );
}

// ── Invariant: unclassified tool defaults to ReadTool ────────────────────────

/// BC-2.05.001 invariant: tools not in the classification registry default to
/// `ReadTool` (fail-open for unclassified tools) per `AuditEmitter::call()`.
#[tokio::test]
async fn test_BC_2_05_001_unclassified_tool_defaults_to_read_tool_fail_open() {
    let backend = MemBackend::new();
    // Empty registry — no tool is classified.
    let registry: ToolClassificationRegistry = HashMap::new();

    let layer = AuditEmitterLayer::new(Arc::new(backend.clone()), Arc::new(registry));
    let mut svc = layer.layer(AlwaysSucceedService);

    let result = invoke(&mut svc, make_request("unknown_tool_xyz")).await;

    assert!(
        result.is_ok(),
        "unclassified tool should default to ReadTool behavior (fail-open): {result:?}"
    );
}

// ── Invariant: write tool success → at least one audit entry ─────────────────

/// BC-2.05.001 postcondition: write tool success → at least one pre-invocation
/// entry is written to audit_buffer before inner handler executes.
#[tokio::test]
async fn test_BC_2_05_001_write_tool_success_has_audit_entry() {
    let backend = MemBackend::new();
    let mut registry: ToolClassificationRegistry = HashMap::new();
    registry.insert("crowdstrike_contain_host", ToolClass::WriteTool);

    let layer = AuditEmitterLayer::new(Arc::new(backend.clone()), Arc::new(registry));
    let mut svc = layer.layer(AlwaysSucceedService);

    let result = invoke(&mut svc, make_request("crowdstrike_contain_host")).await;

    assert!(result.is_ok(), "write tool should succeed: {result:?}");

    let entry_count = count_audit_entries(&backend);
    assert!(
        entry_count >= 1,
        "expected at least 1 audit entry for a successful write tool call, found {entry_count}"
    );
}
