//! Tests for BC-2.05.003 — Credential Values Are Never Present in Audit Entries.
//!
//! Postconditions tested:
//!   - All credential key patterns are replaced with `"[REDACTED]"` (v1.5 sentinel).
//!   - Credential key names are preserved; only values are replaced.
//!   - Redaction is recursive at any nesting depth.
//!   - Non-credential fields are not modified.
//!
//! AC-4: `{ "secret": "abc123" }` → `entry.parameters["secret"] == "[REDACTED]"`.
//! EC-003: nested `"api_key"` at any depth → redacted.
//!
//! SPEC CORRECTION NOTE (S-2.04 v1.5):
//!   These tests assert the canonical sentinel `"[REDACTED]"` (BC-2.05.003).
//!   The stub used `"***REDACTED***"` — the implementer must update
//!   `REDACTED_SENTINEL` in `redaction.rs` to `"[REDACTED]"` to pass these tests.

use serde_json::{json, Value};

use crate::redaction::{is_credential_key, redact, REDACTED_SENTINEL};

// ── Sentinel value assertion ──────────────────────────────────────────────────

/// BC-2.05.003 (S-2.04 v1.5): The redaction sentinel MUST be `"[REDACTED]"`.
/// The stub used `"***REDACTED***"` — this test will FAIL until the implementer
/// corrects `REDACTED_SENTINEL` in `redaction.rs`.
#[test]
fn test_BC_2_05_003_redacted_sentinel_is_square_bracket_form() {
    assert_eq!(
        REDACTED_SENTINEL, "[REDACTED]",
        "REDACTED_SENTINEL must be '[REDACTED]' per S-2.04 v1.5 spec correction (BC-2.05.003). \
         Stub value '***REDACTED***' is incorrect — implementer must fix redaction.rs."
    );
}

// ── AC-4: top-level secret key → value replaced with "[REDACTED]" ────────────

/// AC-4 (BC-2.05.003): Given `parameters: {{ "secret": "abc123" }}`,
/// the audit entry must have `parameters["secret"] == "[REDACTED]"`.
#[test]
fn test_BC_2_05_003_top_level_secret_key_is_redacted() {
    let params = json!({"secret": "abc123"});
    let redacted = redact(params);

    assert_eq!(
        redacted["secret"],
        Value::String("[REDACTED]".to_owned()),
        "top-level 'secret' key must be replaced with '[REDACTED]', not {:?}",
        redacted["secret"]
    );
}

/// BC-2.05.003 canonical test vector: `{{ api_key: "secret123" }}` → `{{ api_key: "[REDACTED]" }}`.
#[test]
fn test_BC_2_05_003_api_key_is_redacted() {
    let params = json!({"api_key": "secret123"});
    let redacted = redact(params);

    assert_eq!(
        redacted["api_key"],
        Value::String("[REDACTED]".to_owned()),
        "top-level 'api_key' must be '[REDACTED]'"
    );
}

/// BC-2.05.003: `password` key is redacted.
#[test]
fn test_BC_2_05_003_password_key_is_redacted() {
    let params = json!({"password": "hunter2"});
    let redacted = redact(params);
    assert_eq!(
        redacted["password"],
        Value::String("[REDACTED]".to_owned()),
        "'password' key must be '[REDACTED]'"
    );
}

/// BC-2.05.003: `token` key is redacted.
#[test]
fn test_BC_2_05_003_token_key_is_redacted() {
    let params = json!({"token": "Bearer abc123xyz"});
    let redacted = redact(params);
    assert_eq!(
        redacted["token"],
        Value::String("[REDACTED]".to_owned()),
        "'token' key must be '[REDACTED]'"
    );
}

/// BC-2.05.003: `credential` key is redacted.
#[test]
fn test_BC_2_05_003_credential_key_is_redacted() {
    let params = json!({"credential": "my-cred-value"});
    let redacted = redact(params);
    assert_eq!(
        redacted["credential"],
        Value::String("[REDACTED]".to_owned()),
        "'credential' key must be '[REDACTED]'"
    );
}

/// BC-2.05.003: `private_key` key is redacted.
#[test]
fn test_BC_2_05_003_private_key_is_redacted() {
    let params = json!({"private_key": "-----BEGIN RSA PRIVATE KEY-----"});
    let redacted = redact(params);
    assert_eq!(
        redacted["private_key"],
        Value::String("[REDACTED]".to_owned()),
        "'private_key' key must be '[REDACTED]'"
    );
}

/// BC-2.05.003: `passphrase` key is redacted.
#[test]
fn test_BC_2_05_003_passphrase_key_is_redacted() {
    let params = json!({"passphrase": "my-passphrase"});
    let redacted = redact(params);
    assert_eq!(
        redacted["passphrase"],
        Value::String("[REDACTED]".to_owned()),
        "'passphrase' key must be '[REDACTED]'"
    );
}

/// BC-2.05.003: `bearer` key is redacted.
#[test]
fn test_BC_2_05_003_bearer_key_is_redacted() {
    let params = json!({"bearer": "my-bearer-token"});
    let redacted = redact(params);
    assert_eq!(
        redacted["bearer"],
        Value::String("[REDACTED]".to_owned()),
        "'bearer' key must be '[REDACTED]'"
    );
}

// ── Suffix patterns ───────────────────────────────────────────────────────────

/// BC-2.05.003: Keys ending in `_token` (e.g., `access_token`) are redacted.
#[test]
fn test_BC_2_05_003_suffix_token_key_is_redacted() {
    let params = json!({"access_token": "tok_abc"});
    let redacted = redact(params);
    assert_eq!(
        redacted["access_token"],
        Value::String("[REDACTED]".to_owned()),
        "key ending in '_token' must be '[REDACTED]'"
    );
}

/// BC-2.05.003: Keys ending in `_secret` (e.g., `client_secret`) are redacted.
#[test]
fn test_BC_2_05_003_suffix_secret_key_is_redacted() {
    let params = json!({"client_secret": "cs_xyz"});
    let redacted = redact(params);
    assert_eq!(
        redacted["client_secret"],
        Value::String("[REDACTED]".to_owned()),
        "key ending in '_secret' must be '[REDACTED]'"
    );
}

/// BC-2.05.003: Keys ending in `_key` (e.g., `ssh_key`, `encryption_key`) are redacted.
#[test]
fn test_BC_2_05_003_suffix_key_key_is_redacted() {
    let params = json!({"ssh_key": "ssh-rsa AAA..."});
    let redacted = redact(params);
    assert_eq!(
        redacted["ssh_key"],
        Value::String("[REDACTED]".to_owned()),
        "key ending in '_key' must be '[REDACTED]'"
    );
}

/// BC-2.05.003: Keys ending in `_password` (e.g., `db_password`) are redacted.
#[test]
fn test_BC_2_05_003_suffix_password_key_is_redacted() {
    let params = json!({"db_password": "s3cr3t"});
    let redacted = redact(params);
    assert_eq!(
        redacted["db_password"],
        Value::String("[REDACTED]".to_owned()),
        "key ending in '_password' must be '[REDACTED]'"
    );
}

// ── EC-003: nested credential key at any depth → redacted ────────────────────

/// EC-003 / BC-2.05.003: Nested `"api_key"` at any depth must be redacted
/// (canonical test vector: `{{ config: {{ api_key: "secret" }} }}`).
#[test]
fn test_BC_2_05_003_nested_api_key_is_redacted() {
    let params = json!({"config": {"api_key": "secret"}});
    let redacted = redact(params);

    assert_eq!(
        redacted["config"]["api_key"],
        Value::String("[REDACTED]".to_owned()),
        "nested 'api_key' must be '[REDACTED]' (EC-003, BC-2.05.003)"
    );
}

/// BC-2.05.003: Deeply nested credential (3 levels) is redacted.
#[test]
fn test_BC_2_05_003_deeply_nested_credential_is_redacted() {
    let params = json!({
        "sensor": {
            "crowdstrike": {
                "auth": {
                    "client_secret": "cs_deep"
                }
            }
        }
    });
    let redacted = redact(params);

    assert_eq!(
        redacted["sensor"]["crowdstrike"]["auth"]["client_secret"],
        Value::String("[REDACTED]".to_owned()),
        "deeply nested 'client_secret' must be '[REDACTED]'"
    );
}

// ── Key names are preserved, only values replaced ────────────────────────────

/// BC-2.05.003: Credential key names are preserved for traceability — only
/// values are replaced (canonical test vector: `{{ api_key: "secret123" }}`
/// → `{{ api_key: "[REDACTED]" }}`).
#[test]
fn test_BC_2_05_003_credential_key_name_preserved() {
    let params = json!({"api_key": "secret123"});
    let redacted = redact(params);

    let obj = redacted.as_object().unwrap();
    assert!(
        obj.contains_key("api_key"),
        "key name 'api_key' must be preserved after redaction"
    );
}

// ── Non-credential fields are not modified ────────────────────────────────────

/// BC-2.05.003 canonical test vector: non-credential fields pass through unchanged.
/// `{{ sensor: "crowdstrike", api_key: "secret" }}` →
/// `{{ sensor: "crowdstrike", api_key: "[REDACTED]" }}`.
#[test]
fn test_BC_2_05_003_non_credential_field_not_modified() {
    let params = json!({"sensor": "crowdstrike", "api_key": "secret"});
    let redacted = redact(params);

    assert_eq!(
        redacted["sensor"],
        Value::String("crowdstrike".to_owned()),
        "non-credential field 'sensor' must not be modified"
    );
    assert_eq!(
        redacted["api_key"],
        Value::String("[REDACTED]".to_owned()),
        "credential field 'api_key' must be '[REDACTED]'"
    );
}

/// BC-2.05.003 edge case: a value containing a credential pattern substring
/// in its content (not key name) is NOT redacted.
/// `{{ hostname: "api_token_server.example.com" }}` → unchanged.
#[test]
fn test_BC_2_05_003_value_containing_pattern_substring_not_redacted() {
    let params = json!({"hostname": "api_token_server.example.com"});
    let redacted = redact(params);

    assert_eq!(
        redacted["hostname"],
        Value::String("api_token_server.example.com".to_owned()),
        "value containing '_token' as a substring must NOT be redacted (only key names match)"
    );
}

// ── is_credential_key pattern checker ────────────────────────────────────────

/// BC-2.05.003: `is_credential_key` must return true for all listed exact patterns.
#[test]
fn test_BC_2_05_003_is_credential_key_exact_patterns() {
    let patterns = [
        "password",
        "secret",
        "token",
        "api_key",
        "credential",
        "private_key",
        "passphrase",
        "bearer",
    ];
    for pattern in patterns {
        assert!(
            is_credential_key(pattern),
            "is_credential_key('{pattern}') must return true"
        );
    }
}

/// BC-2.05.003: `is_credential_key` must be case-insensitive.
#[test]
fn test_BC_2_05_003_is_credential_key_case_insensitive() {
    assert!(
        is_credential_key("PASSWORD"),
        "is_credential_key must be case-insensitive for 'PASSWORD'"
    );
    assert!(
        is_credential_key("API_KEY"),
        "is_credential_key must be case-insensitive for 'API_KEY'"
    );
    assert!(
        is_credential_key("Client_Secret"),
        "is_credential_key must be case-insensitive for 'Client_Secret'"
    );
}

/// BC-2.05.003: `is_credential_key` must return false for non-credential keys.
#[test]
fn test_BC_2_05_003_is_credential_key_non_credential_keys_return_false() {
    let non_patterns = [
        "sensor",
        "hostname",
        "query",
        "client_id",
        "tool_name",
        "result",
    ];
    for key in non_patterns {
        assert!(
            !is_credential_key(key),
            "is_credential_key('{key}') must return false"
        );
    }
}

// ── Object-typed credential value is replaced with string "[REDACTED]" ────────

/// BC-2.05.003: When a credential key's value is an object (not just a string),
/// the entire value is replaced with the sentinel string.
#[test]
fn test_BC_2_05_003_credential_value_object_replaced_with_sentinel_string() {
    let params = json!({"api_key": {"nested_key": "some_value"}});
    let redacted = redact(params);

    // The value must be the sentinel string, not the original object.
    assert_eq!(
        redacted["api_key"],
        Value::String("[REDACTED]".to_owned()),
        "credential key with object value must be replaced with '[REDACTED]' string"
    );
}

// ── Mixed fields canonical test vector ───────────────────────────────────────

/// BC-2.05.003 canonical test vector: mixed fields —
/// `{{ sensor: "crowdstrike", api_key: "secret" }}` →
/// `{{ sensor: "crowdstrike", api_key: "[REDACTED]" }}`.
#[test]
fn test_BC_2_05_003_mixed_fields_canonical_vector() {
    let params = json!({"sensor": "crowdstrike", "api_key": "secret"});
    let redacted = redact(params);

    assert_eq!(
        redacted["sensor"],
        Value::String("crowdstrike".to_owned()),
        "non-credential 'sensor' must pass through unchanged"
    );
    assert_eq!(
        redacted["api_key"],
        Value::String("[REDACTED]".to_owned()),
        "credential 'api_key' must be '[REDACTED]'"
    );
}
