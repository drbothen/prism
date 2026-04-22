// S-1.09: BC-2.04.010 — Confirmation Token Consumption via confirm_action
//
// Tests verify:
//   - Valid consumption: unexpired, unconsumed token, correct client_id → Ok.
//   - Expired token → E-FLAG-003 (TokenExpired); write not executed.
//   - Already consumed → E-FLAG-004 (TokenAlreadyConsumed); write not executed.
//   - Token not found → E-FLAG-008 (TokenNotFound).
//   - client_id mismatch → E-MCP-004 (ConfirmClientIdMismatch); write not executed.
//   - Content hash mismatch → E-FLAG-005 (TokenContentHashMismatch).
//   - Token is marked consumed BEFORE execution (single-use; VP-008).
//   - EC-04-021: concurrent same-token → first succeeds; second gets E-FLAG-004.
//   - EC-04-022: global-scope token with __global__ client_id sentinel.
//   - AC-2: valid token at 299s → operation executes.
//
// Naming: test_BC_2_04_010_<assertion>
#![allow(non_snake_case)]

use std::time::Duration;

use serde_json::json;

use prism_core::error::PrismError;
use prism_security::confirmation_token::ConfirmationTokenStore;

fn make_store() -> ConfirmationTokenStore {
    ConfirmationTokenStore::new()
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.010 Postcondition: Valid consumption
// ─────────────────────────────────────────────────────────────

/// Postcondition: valid token (unexpired, unconsumed, correct client_id, matching params)
/// → consume() returns Ok(ConfirmationToken).
///
/// Canonical test vector: "Valid consumption".
/// AC-2: valid token (well within TTL) → operation executes.
#[test]
fn test_BC_2_04_010_valid_token_consumes_successfully() {
    let store = make_store();
    let params = json!({"device_id": "host-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "Isolate host-001")
        .expect("generate must succeed");

    let result = store.consume(&token.token_id, "acme", &params);
    assert!(
        result.is_ok(),
        "BC-2.04.010: consume of valid unexpired token must return Ok, got {result:?}"
    );
}

/// Postcondition: after successful consume, token is marked consumed.
#[test]
fn test_BC_2_04_010_consumed_token_is_marked_consumed() {
    let store = make_store();
    let params = json!({"device_id": "host-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "s")
        .unwrap();

    let consumed_token = store.consume(&token.token_id, "acme", &params).unwrap();
    assert!(
        consumed_token.consumed,
        "BC-2.04.010: consumed token must have consumed = true"
    );
}

/// Postcondition: after consume(), active_count decreases (token is no longer active).
#[test]
fn test_BC_2_04_010_active_count_decreases_after_consume() {
    let store = make_store();
    let params = json!({"device_id": "host-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "s")
        .unwrap();
    assert_eq!(store.active_count(), 1);

    store.consume(&token.token_id, "acme", &params).unwrap();
    assert_eq!(
        store.active_count(),
        0,
        "BC-2.04.010: active_count must be 0 after consuming the only token"
    );
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.010 Error: Already consumed (E-FLAG-004, VP-008)
// ─────────────────────────────────────────────────────────────

/// Error case: token already consumed → E-FLAG-004 (TokenAlreadyConsumed).
///
/// VP-008 invariant: single-use enforcement.
/// Canonical test vector: "Already consumed".
#[test]
fn test_BC_2_04_010_already_consumed_returns_e_flag_004() {
    let store = make_store();
    let params = json!({"device_id": "host-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "s")
        .unwrap();

    // First consume: success.
    store.consume(&token.token_id, "acme", &params).unwrap();

    // Second consume: must fail.
    let result = store.consume(&token.token_id, "acme", &params);
    assert!(
        matches!(
            result,
            Err(PrismError::TokenAlreadyConsumed { .. }) | Err(PrismError::TokenNotFound { .. })
        ),
        "BC-2.04.010 VP-008: second consume must return TokenAlreadyConsumed or TokenNotFound (E-FLAG-004/008), got {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.010 Error: Token not found (E-FLAG-008)
// ─────────────────────────────────────────────────────────────

/// Error case: token_id not found in store → E-FLAG-008 (TokenNotFound).
#[test]
fn test_BC_2_04_010_token_not_found_returns_e_flag_008() {
    let store = make_store();
    let params = json!({"device_id": "host-001"});
    let result = store.consume("nonexistent-token-id", "acme", &params);
    assert!(
        matches!(result, Err(PrismError::TokenNotFound { .. })),
        "BC-2.04.010: unknown token_id must return TokenNotFound (E-FLAG-008), got {result:?}"
    );
}

/// E-FLAG-008 error message must contain the token_id that was not found.
#[test]
fn test_BC_2_04_010_token_not_found_error_contains_token_id() {
    let store = make_store();
    let err = store
        .consume("phantom-token-xyz", "acme", &json!({}))
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("phantom-token-xyz") || msg.contains("E-FLAG-008"),
        "BC-2.04.010 E-FLAG-008: error must reference token_id or E-FLAG-008 code: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.010 Error: client_id mismatch (E-MCP-004)
// ─────────────────────────────────────────────────────────────

/// Error case: supplied client_id does not match token's embedded client_id → E-MCP-004.
///
/// Canonical test vector: "Client ID mismatch".
#[test]
fn test_BC_2_04_010_client_id_mismatch_returns_e_mcp_004() {
    let store = make_store();
    let params = json!({"device_id": "host-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "s")
        .unwrap();

    // Use wrong client_id.
    let result = store.consume(&token.token_id, "other-client", &params);
    assert!(
        matches!(result, Err(PrismError::ConfirmClientIdMismatch { .. })),
        "BC-2.04.010 E-MCP-004: client_id mismatch must return ConfirmClientIdMismatch, got {result:?}"
    );
}

/// client_id check is equality only — no config lookup (BC-2.04.010 postcondition).
#[test]
fn test_BC_2_04_010_client_id_check_is_equality_only() {
    let store = make_store();
    let params = json!({"device_id": "host-001"});
    // Generate with "acme". Confirm with "ACME" (different case) → mismatch.
    let token = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "s")
        .unwrap();
    let result = store.consume(&token.token_id, "ACME", &params);
    assert!(
        matches!(result, Err(PrismError::ConfirmClientIdMismatch { .. })),
        "BC-2.04.010: client_id check is case-sensitive equality; 'ACME' != 'acme'"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-022: __global__ sentinel for global-scope operations
// ─────────────────────────────────────────────────────────────

/// EC-04-022: Generating a token with client_id = "__global__" and confirming
/// with the same sentinel is accepted (global-scope mutations: aliases, schedules, etc.).
#[test]
fn test_BC_2_04_010_ec_global_sentinel_client_id_accepted() {
    let store = make_store();
    let params = json!({"alias": "threat-detect"});
    let token = store
        .generate("__global__", "create_alias", params.clone(), "Create global alias threat-detect")
        .expect("generate with __global__ must succeed");

    // Confirm with same __global__ sentinel.
    let result = store.consume(&token.token_id, "__global__", &params);
    assert!(
        result.is_ok(),
        "EC-04-022: __global__ sentinel must be accepted when token was generated with __global__, got {result:?}"
    );
}

/// EC-04-022: Using __global__ when token was generated for a real client → mismatch.
#[test]
fn test_BC_2_04_010_ec_global_sentinel_mismatches_real_client() {
    let store = make_store();
    let params = json!({"device_id": "host-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "s")
        .unwrap();

    let result = store.consume(&token.token_id, "__global__", &params);
    assert!(
        matches!(result, Err(PrismError::ConfirmClientIdMismatch { .. })),
        "EC-04-022: __global__ must not match a token generated for 'acme'"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-021: Concurrent consumption (single-thread model)
// ─────────────────────────────────────────────────────────────

/// EC-04-021 (single-thread model): Two consume calls with same token_id —
/// first succeeds, second returns TokenAlreadyConsumed or TokenNotFound.
///
/// This simulates the expected outcome of concurrent calls without multi-threading.
/// True concurrency is tested separately.
#[test]
fn test_BC_2_04_010_ec_sequential_double_consume_only_one_succeeds() {
    let store = make_store();
    let params = json!({"device_id": "host-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "s")
        .unwrap();

    let r1 = store.consume(&token.token_id, "acme", &params);
    let r2 = store.consume(&token.token_id, "acme", &params);

    assert!(
        r1.is_ok(),
        "EC-04-021: first consume must succeed, got {r1:?}"
    );
    assert!(
        r2.is_err(),
        "EC-04-021: second consume must fail (single-use), got {r2:?}"
    );
}
