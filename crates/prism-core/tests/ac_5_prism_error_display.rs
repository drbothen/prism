//! AC-5: Every PrismError variant Display begins with its structured error code token.

use prism_core::PrismError;

fn assert_error_code_prefix(err: &PrismError, expected_prefix: &str) {
    let msg = format!("{err}");
    assert!(
        msg.starts_with(expected_prefix),
        "PrismError Display must start with '{expected_prefix}', got: {msg:?}"
    );
}

/// AC-5: E-AUTH category prefix present in all auth variants.
#[test]
fn test_ac5_prism_error_display_e_auth_invalid_tenant_id() {
    let err = PrismError::InvalidOrgSlug {
        reason: "too long".to_string(),
    };
    assert_error_code_prefix(&err, "E-AUTH-001");
}

#[test]
fn test_ac5_prism_error_display_e_auth_invalid_analyst_id() {
    let err = PrismError::InvalidAnalystId {
        reason: "bad char".to_string(),
    };
    assert_error_code_prefix(&err, "E-AUTH-002");
}

#[test]
fn test_ac5_prism_error_display_e_auth_unauthorized() {
    let err = PrismError::Unauthorized {
        action: "write:cases".to_string(),
    };
    assert_error_code_prefix(&err, "E-AUTH-020");
}

/// AC-5: E-STORE category.
#[test]
fn test_ac5_prism_error_display_e_store_001() {
    let err = PrismError::StorageOpenFailed {
        detail: "path not found".to_string(),
    };
    assert_error_code_prefix(&err, "E-STORE-001");
}

#[test]
fn test_ac5_prism_error_display_e_store_cursor_cap() {
    let err = PrismError::CursorCapExceeded;
    assert_error_code_prefix(&err, "E-STORE-020");
}

#[test]
fn test_ac5_prism_error_display_e_store_domain_not_found() {
    let err = PrismError::StorageDomainNotFound {
        domain: "unknown".to_string(),
    };
    assert_error_code_prefix(&err, "E-STORE-004");
}

/// AC-5: E-SENSOR category.
#[test]
fn test_ac5_prism_error_display_e_sensor_001() {
    let err = PrismError::SensorHttpError {
        sensor: "crowdstrike".to_string(),
        status: 429,
        body: "rate limited".to_string(),
    };
    assert_error_code_prefix(&err, "E-SENSOR-001");
}

/// AC-5: E-QUERY category.
#[test]
fn test_ac5_prism_error_display_e_query_001() {
    let err = PrismError::QueryParseFailed {
        offset: 42,
        detail: "unexpected token".to_string(),
        query: String::new(),
    };
    assert_error_code_prefix(&err, "E-QUERY-001");
}

/// AC-5: E-CRED category.
#[test]
fn test_ac5_prism_error_display_e_cred_001() {
    let err = PrismError::InvalidCredentialName {
        name: "bad cred".to_string(),
        reason: "test reason".to_string(),
    };
    assert_error_code_prefix(&err, "E-CRED-001");
}

/// AC-5: E-FLAG category.
///
/// P2-03(c) (2026-06-10 review pass-2): previously exercised the
/// `FeatureFlagDisabled` variant, which was removed (zero spec backing, zero
/// production emitters; E-FLAG-002 is the compile-tier `CapabilityDenied`
/// denial per BC-2.04.015 v1.2). E-FLAG category Display coverage is preserved
/// via `FeatureFlagEvalError` (E-FLAG-010).
#[test]
fn test_ac5_prism_error_display_e_flag_010() {
    let err = PrismError::FeatureFlagEvalError {
        flag: "sensor.write".to_string(),
        detail: "evaluation failed".to_string(),
    };
    assert_error_code_prefix(&err, "E-FLAG-010");
}

/// AC-5: E-OCSF category.
#[test]
fn test_ac5_prism_error_display_e_ocsf_001() {
    let err = PrismError::OcsfFieldMissing {
        field: "severity_id".to_string(),
    };
    assert_error_code_prefix(&err, "E-OCSF-001");
}

/// AC-5: E-CFG category — ConfigNotFound renumbered to E-CFG-103 (ADR-038 D2).
/// The prefix assertion also guards the tombstoned pre-v1.8 number from
/// reappearing (ADR-038 D5): a display starting with "E-CFG-103" cannot
/// carry any retired E-CFG-0NN prefix.
#[test]
fn test_ac5_prism_error_display_e_cfg_103() {
    let err = PrismError::ConfigNotFound {
        path: "/etc/prism.toml".to_string(),
    };
    assert_error_code_prefix(&err, "E-CFG-103");
}

/// AC-5: E-CFG-100 — NEW `ClientNotFound` variant (ADR-038 D3 variant split).
///
/// Canonical display (ADR-038 D1 / error-taxonomy §E-CFG-100):
///   "E-CFG-100: client '{client_id}' not found in configuration"
#[test]
fn test_ac5_prism_error_display_e_cfg_100() {
    let err = PrismError::ClientNotFound {
        client_id: "acme".to_string(),
    };
    assert_error_code_prefix(&err, "E-CFG-100");
    let msg = format!("{err}");
    assert_eq!(
        msg, "E-CFG-100: client 'acme' not found in configuration",
        "ClientNotFound Display must match the canonical taxonomy v1.66 format"
    );
}

/// AC-5: E-MCP category.
#[test]
fn test_ac5_prism_error_display_e_mcp_001() {
    let err = PrismError::McpToolNotFound {
        tool: "prism.query".to_string(),
    };
    assert_error_code_prefix(&err, "E-MCP-001");
}

/// AC-5: E-SAFETY category.
#[test]
fn test_ac5_prism_error_display_e_safety_001() {
    let err = PrismError::SafetyContextContamination {
        detail: "credential in payload".to_string(),
    };
    assert_error_code_prefix(&err, "E-SAFETY-001");
}

/// AC-5: E-SCHED category.
#[test]
fn test_ac5_prism_error_display_e_sched_001() {
    let err = PrismError::ScheduleNotFound {
        id: "sched-123".to_string(),
    };
    assert_error_code_prefix(&err, "E-SCHED-001");
}

/// AC-5: E-DET category.
#[test]
fn test_ac5_prism_error_display_e_det_001() {
    let err = PrismError::DetectionRuleParseFailed {
        rule_id: "rule-001".to_string(),
        detail: "syntax error".to_string(),
    };
    assert_error_code_prefix(&err, "E-DET-001");
}

/// AC-5: E-CASE category.
#[test]
fn test_ac5_prism_error_display_e_case_001() {
    let err = PrismError::CaseNotFound {
        case_id: "case-001".to_string(),
    };
    assert_error_code_prefix(&err, "E-CASE-001");
}

/// AC-5: E-WATCH category.
#[test]
fn test_ac5_prism_error_display_e_watch_001() {
    let err = PrismError::WatchdogHeartbeatMissed {
        component: "query-engine".to_string(),
        elapsed_ms: 5000,
    };
    assert_error_code_prefix(&err, "E-WATCH-001");
}

/// AC-5: E-SPEC category.
#[test]
fn test_ac5_prism_error_display_e_spec_001() {
    let err = PrismError::SpecNotFound {
        path: "/etc/sensors/crowdstrike.toml".to_string(),
    };
    assert_error_code_prefix(&err, "E-SPEC-001");
}

/// AC-5: E-IOC category.
#[test]
fn test_ac5_prism_error_display_e_ioc_001() {
    let err = PrismError::IocFeedParseFailed {
        feed: "nvd".to_string(),
        detail: "unexpected format".to_string(),
    };
    assert_error_code_prefix(&err, "E-IOC-001");
}

/// AC-5: E-INT catch-all.
#[test]
fn test_ac5_prism_error_display_e_int_001() {
    let err = PrismError::Internal {
        detail: "unreachable branch hit".to_string(),
    };
    assert_error_code_prefix(&err, "E-INT-001");
}

// ---------------------------------------------------------------------------
// RG-ECRED-001: CredentialEncryptionError Display must start with E-CRED-006
// (ADR-035 §D2; AC-001 of S-MAINT-ECRED-TAXONOMY-SYNC-001)
//
// RED GATE: Currently fails because #[error] reads "E-CRED-005: ..."
// PASSES AFTER: implementer renumbers #[error] to "E-CRED-006: ..."
// ---------------------------------------------------------------------------

/// RG-ECRED-001: CredentialEncryptionError Display starts with "E-CRED-006:".
///
/// Canonical Display string (ADR-035 §Exact-Display-String-Changes):
///   "E-CRED-006: credential encryption error: {reason}"
///
/// Currently fails: `#[error]` still reads `"E-CRED-005: credential encryption error: {reason}"`.
/// Passes after: renumbered to `"E-CRED-006: ..."`.
#[test]
fn test_ac5_prism_error_display_e_cred_006_encryption() {
    let err = PrismError::CredentialEncryptionError {
        reason: "test reason".to_string(),
    };
    let msg = format!("{err}");
    assert!(
        msg.starts_with("E-CRED-006:"),
        "CredentialEncryptionError Display must start with 'E-CRED-006:', got: {msg:?}"
    );
    assert!(
        msg.contains("credential encryption error:"),
        "Display must contain canonical phrase 'credential encryption error:', got: {msg:?}"
    );
    assert!(
        msg.contains("test reason"),
        "Display must include the reason field, got: {msg:?}"
    );
    // Guard: old code must not appear
    assert!(
        !msg.starts_with("E-CRED-005:"),
        "Display must NOT start with 'E-CRED-005:' after renumber, got: {msg:?}"
    );
}

// ---------------------------------------------------------------------------
// RG-ECRED-002: EncryptionKeyMissing Display must start with E-CRED-007
// (ADR-035 §D2; AC-002 of S-MAINT-ECRED-TAXONOMY-SYNC-001)
//
// RED GATE: Currently fails because #[error] reads "E-CRED-006: ..."
// PASSES AFTER: implementer renumbers #[error] to "E-CRED-007: ..."
// ---------------------------------------------------------------------------

/// RG-ECRED-002: EncryptionKeyMissing Display starts with "E-CRED-007:".
///
/// Canonical Display string (ADR-035 §Exact-Display-String-Changes):
///   "E-CRED-007: encryption key not configured: {reason}"
///
/// Currently fails: `#[error]` still reads `"E-CRED-006: encryption key not configured: {reason}"`.
/// Passes after: renumbered to `"E-CRED-007: ..."`.
#[test]
fn test_ac5_prism_error_display_e_cred_007_key_missing() {
    let err = PrismError::EncryptionKeyMissing {
        reason: "not set".to_string(),
    };
    let msg = format!("{err}");
    assert!(
        msg.starts_with("E-CRED-007:"),
        "EncryptionKeyMissing Display must start with 'E-CRED-007:', got: {msg:?}"
    );
    assert!(
        msg.contains("encryption key not configured:"),
        "Display must contain canonical phrase 'encryption key not configured:', got: {msg:?}"
    );
    assert!(
        msg.contains("not set"),
        "Display must include the reason field, got: {msg:?}"
    );
    // Guard: old code must not appear
    assert!(
        !msg.starts_with("E-CRED-006:"),
        "Display must NOT start with 'E-CRED-006:' after renumber, got: {msg:?}"
    );
}
