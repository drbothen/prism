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
    let err = PrismError::InvalidTenantId {
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
    let err = PrismError::CursorCapExceeded {
        max: 1000,
        count: 2000,
    };
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
    };
    assert_error_code_prefix(&err, "E-QUERY-001");
}

/// AC-5: E-CRED category.
#[test]
fn test_ac5_prism_error_display_e_cred_001() {
    let err = PrismError::InvalidCredentialName {
        name: "bad cred".to_string(),
    };
    assert_error_code_prefix(&err, "E-CRED-001");
}

/// AC-5: E-FLAG category.
#[test]
fn test_ac5_prism_error_display_e_flag_002() {
    let err = PrismError::FeatureFlagDisabled {
        flag: "sensor.write".to_string(),
    };
    assert_error_code_prefix(&err, "E-FLAG-002");
}

/// AC-5: E-OCSF category.
#[test]
fn test_ac5_prism_error_display_e_ocsf_001() {
    let err = PrismError::OcsfFieldMissing {
        field: "severity_id".to_string(),
    };
    assert_error_code_prefix(&err, "E-OCSF-001");
}

/// AC-5: E-CFG category.
#[test]
fn test_ac5_prism_error_display_e_cfg_001() {
    let err = PrismError::ConfigNotFound {
        path: "/etc/prism.toml".to_string(),
    };
    assert_error_code_prefix(&err, "E-CFG-001");
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
