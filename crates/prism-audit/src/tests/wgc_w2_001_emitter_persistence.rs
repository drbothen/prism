//! RED tests for WGC-W2-001: specialized emitters must call append_audit_entry.
//!
//! All 5 emitters currently ignore persistence — they log via tracing only.
//! These tests assert that after calling each emitter with a backend, at least
//! one audit entry appears in the backend's audit_buffer CF.
//!
//! # RED gate
//! These tests will FAIL until each emitter is updated to:
//!   1. Accept a `backend: &dyn RocksStorageBackend` parameter
//!   2. Construct and call `append_audit_entry(backend, …)`
//!
//! # Test IDs
//! - WGC-W2-001-cred   — emit_credential_event persists to backend
//! - WGC-W2-001-flag   — emit_flag_eval persists to backend
//! - WGC-W2-001-gen    — emit_token_generated persists to backend
//! - WGC-W2-001-con    — emit_token_consumed persists to backend
//! - WGC-W2-001-exp    — emit_token_expired persists to backend

use chrono::Utc;

use crate::credential_events::{
    emit_credential_event, CredentialAccessResult, CredentialAccessType, RequestingContext,
};
use crate::flag_events::{emit_flag_eval, FlagEvalContext, FlagEvalDetail, FlagResolutionStep};
use crate::tests::helpers::{count_audit_entries, MemBackend};
use crate::token_events::{
    emit_token_consumed, emit_token_expired, emit_token_generated, TokenEventContext,
};

// ── WGC-W2-001-cred ───────────────────────────────────────────────────────────

/// WGC-W2-001: emit_credential_event must call append_audit_entry exactly once.
///
/// RED: current implementation ignores the backend parameter (it doesn't
/// even accept one). This test will fail until the emitter is updated.
#[test]
fn test_WGC_W2_001_emit_credential_event_persists_to_backend() {
    let backend = MemBackend::new();
    let ctx = RequestingContext {
        tool_name: "crowdstrike_get_detections".to_owned(),
        client_id: "acme".to_owned(),
        trace_id: "01900000-0000-7000-0000-000000000010".to_owned(),
    };

    let before = count_audit_entries(&backend);

    emit_credential_event(
        &backend,
        "crowdstrike_api_key",
        "crowdstrike",
        CredentialAccessType::Read,
        CredentialAccessResult::Success,
        &ctx,
    )
    .expect("emit_credential_event must not fail");

    let after = count_audit_entries(&backend);
    assert_eq!(
        after,
        before + 1,
        "WGC-W2-001: emit_credential_event must persist exactly 1 audit entry to the backend \
         (before={before}, after={after}). Current code only calls tracing::info! and never \
         calls append_audit_entry — no row lands in RocksDB."
    );
}

// ── WGC-W2-001-flag ───────────────────────────────────────────────────────────

/// WGC-W2-001: emit_flag_eval must call append_audit_entry exactly once.
///
/// RED: current implementation ignores the backend parameter.
#[test]
fn test_WGC_W2_001_emit_flag_eval_persists_to_backend() {
    let backend = MemBackend::new();
    let ctx = FlagEvalContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        trace_id: "01900000-0000-7000-0000-000000000011".to_owned(),
    };
    let detail = FlagEvalDetail {
        client_id: "acme".to_owned(),
        capability_path: "sensors.crowdstrike.write".to_owned(),
        evaluation_result: true,
        resolution_trace: vec![FlagResolutionStep {
            rule_id: "R-001".to_owned(),
            matched: true,
            reason: "client_id matched allowlist".to_owned(),
        }],
    };

    let before = count_audit_entries(&backend);

    emit_flag_eval(&backend, detail, &ctx).expect("emit_flag_eval must not fail");

    let after = count_audit_entries(&backend);
    assert_eq!(
        after,
        before + 1,
        "WGC-W2-001: emit_flag_eval must persist exactly 1 audit entry to the backend \
         (before={before}, after={after}). Current code only calls tracing::info! and never \
         calls append_audit_entry."
    );
}

// ── WGC-W2-001-gen ────────────────────────────────────────────────────────────

/// WGC-W2-001: emit_token_generated must call append_audit_entry exactly once.
///
/// RED: current implementation ignores the backend parameter.
#[test]
fn test_WGC_W2_001_emit_token_generated_persists_to_backend() {
    let backend = MemBackend::new();
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };
    let expiry = Utc::now() + chrono::Duration::seconds(300);

    let before = count_audit_entries(&backend);

    emit_token_generated(
        &backend,
        "tok-wgc-001",
        "isolate host acme-ws-01",
        expiry,
        &ctx,
    )
    .expect("emit_token_generated must not fail");

    let after = count_audit_entries(&backend);
    assert_eq!(
        after,
        before + 1,
        "WGC-W2-001: emit_token_generated must persist exactly 1 audit entry to the backend \
         (before={before}, after={after}). Current code only calls tracing::info!."
    );
}

// ── WGC-W2-001-con ────────────────────────────────────────────────────────────

/// WGC-W2-001: emit_token_consumed must call append_audit_entry exactly once.
///
/// RED: current implementation ignores the backend parameter.
#[test]
fn test_WGC_W2_001_emit_token_consumed_persists_to_backend() {
    let backend = MemBackend::new();
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };

    let before = count_audit_entries(&backend);

    emit_token_consumed(&backend, "tok-wgc-002", "isolate host acme-ws-01", &ctx)
        .expect("emit_token_consumed must not fail");

    let after = count_audit_entries(&backend);
    assert_eq!(
        after,
        before + 1,
        "WGC-W2-001: emit_token_consumed must persist exactly 1 audit entry to the backend \
         (before={before}, after={after}). Current code only calls tracing::info!."
    );
}

// ── WGC-W2-001-exp ────────────────────────────────────────────────────────────

/// WGC-W2-001: emit_token_expired must call append_audit_entry exactly once.
///
/// RED: current implementation ignores the backend parameter.
#[test]
fn test_WGC_W2_001_emit_token_expired_persists_to_backend() {
    let backend = MemBackend::new();
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };
    let expiry = Utc::now() - chrono::Duration::seconds(1); // already expired

    let before = count_audit_entries(&backend);

    emit_token_expired(
        &backend,
        "tok-wgc-003",
        "isolate host acme-ws-02",
        expiry,
        &ctx,
    )
    .expect("emit_token_expired must not fail");

    let after = count_audit_entries(&backend);
    assert_eq!(
        after,
        before + 1,
        "WGC-W2-001: emit_token_expired must persist exactly 1 audit entry to the backend \
         (before={before}, after={after}). Current code only calls tracing::info!."
    );
}
