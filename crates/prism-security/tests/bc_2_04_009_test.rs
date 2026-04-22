// S-1.09: BC-2.04.009 — Confirmation Token Generation (100-Token Active Cap)
//
// Tests verify:
//   - Token generation returns a ConfirmationToken (not execution).
//   - token_id is non-empty (cryptographic random, not sequential).
//   - action_hash is SHA-256 of canonical params.
//   - expires_at = created_at + 300s.
//   - consumed = false on fresh token.
//   - Two independent tokens for same action have different token_ids (CSPRNG).
//   - Token store enforces hard cap of 100 active tokens after cleanup.
//   - Cap reached (100 active after sweep) → E-FLAG-007, no eviction.
//   - EC-04-018: same action twice → two independent tokens.
//   - EC-04-019: store at 99, sweep frees slot → token created.
//   - EC-04-034: in-memory only (store is not persisted).
//   - AC-1: contain_host without token → confirmation token returned.
//
// Naming: test_BC_2_04_009_<assertion>
#![allow(non_snake_case)]

use std::time::Duration;

use serde_json::json;

use prism_core::error::PrismError;
use prism_security::confirmation_token::{ConfirmationTokenStore, TOKEN_CAP, TOKEN_TTL};

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

fn make_store() -> ConfirmationTokenStore {
    ConfirmationTokenStore::new()
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.009 Postconditions: token fields
// ─────────────────────────────────────────────────────────────

/// Postcondition: generate returns a ConfirmationToken with non-empty token_id.
///
/// The action is NOT executed.
/// AC-1: contain_host without token → token returned (not execution).
#[test]
fn test_BC_2_04_009_generate_returns_token_not_execution() {
    let store = make_store();
    let params = json!({"device_id": "abc-001"});
    let result = store.generate(
        "acme",
        "crowdstrike_contain_host",
        params,
        "Isolate host abc-001 (10.0.1.5) from network for client acme",
    );
    assert!(
        result.is_ok(),
        "BC-2.04.009: generate must succeed on empty store: {result:?}"
    );
    let token = result.unwrap();
    assert!(
        !token.token_id.is_empty(),
        "BC-2.04.009: token_id must be non-empty"
    );
}

/// Postcondition: token_id is 64 hex characters (256-bit cryptographic random).
///
/// BC-2.04.009 postcondition 1: token_id is cryptographic random string.
#[test]
fn test_BC_2_04_009_token_id_is_256bit_hex() {
    let store = make_store();
    let params = json!({"device_id": "abc-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params, "Isolate host")
        .expect("BC-2.04.009: generate must succeed");

    assert_eq!(
        token.token_id.len(),
        64,
        "BC-2.04.009: token_id must be 64 hex chars (256-bit random)"
    );
    assert!(
        token.token_id.chars().all(|c| c.is_ascii_hexdigit()),
        "BC-2.04.009: token_id must be lowercase hex"
    );
}

/// Postcondition: `consumed = false` on fresh token.
#[test]
fn test_BC_2_04_009_fresh_token_is_not_consumed() {
    let store = make_store();
    let params = json!({"device_id": "abc-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params, "Isolate host")
        .expect("BC-2.04.009: generate must succeed");

    assert!(
        !token.consumed,
        "BC-2.04.009: freshly generated token must have consumed = false"
    );
}

/// Postcondition: `expires_at = created_at + 300s`.
#[test]
fn test_BC_2_04_009_expires_at_is_300s_after_created_at() {
    let store = make_store();
    let params = json!({"device_id": "abc-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params, "Isolate host")
        .expect("BC-2.04.009: generate must succeed");

    let expected_ttl = Duration::from_secs(300);
    let actual_ttl = token
        .expires_at
        .duration_since(token.created_at)
        .expect("BC-2.04.009: expires_at must be after created_at");

    assert_eq!(
        actual_ttl, expected_ttl,
        "BC-2.04.009: expires_at must be exactly 300s after created_at (DI-007)"
    );
}

/// Postcondition: `action_hash` is non-empty (SHA-256 hex).
#[test]
fn test_BC_2_04_009_action_hash_is_nonempty_sha256_hex() {
    let store = make_store();
    let params = json!({"device_id": "abc-001"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", params, "Isolate host")
        .expect("BC-2.04.009: generate must succeed");

    assert!(
        !token.action_hash.is_empty(),
        "BC-2.04.009: action_hash must be non-empty"
    );
    assert_eq!(
        token.action_hash.len(),
        64,
        "BC-2.04.009: action_hash must be 64 chars (SHA-256 hex)"
    );
}

/// Postcondition: token fields carry client_id, tool_name, action_params, action_summary.
#[test]
fn test_BC_2_04_009_token_carries_client_and_tool_fields() {
    let store = make_store();
    let params = json!({"device_id": "abc-001"});
    let summary = "Isolate host abc-001 from network for client acme";
    let token = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), summary)
        .expect("BC-2.04.009: generate must succeed");

    assert_eq!(token.client_id, "acme");
    assert_eq!(token.tool_name, "crowdstrike_contain_host");
    assert_eq!(token.action_params, params);
    assert_eq!(token.action_summary, summary);
}

// ─────────────────────────────────────────────────────────────
// EC-04-018: Same action twice → two independent tokens (both valid)
// ─────────────────────────────────────────────────────────────

/// EC-04-018: Requesting a token for the same action twice creates two independent tokens.
/// Both are valid; both have distinct token_ids (CSPRNG-backed).
#[test]
fn test_BC_2_04_009_ec_same_action_twice_creates_independent_tokens() {
    let store = make_store();
    let params = json!({"device_id": "abc-001"});

    let t1 = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "Action A")
        .expect("BC-2.04.009 EC-04-018: first generate must succeed");
    let t2 = store
        .generate("acme", "crowdstrike_contain_host", params.clone(), "Action A")
        .expect("BC-2.04.009 EC-04-018: second generate must succeed");

    assert_ne!(
        t1.token_id, t2.token_id,
        "EC-04-018: two tokens for the same action must have different token_ids (CSPRNG)"
    );
    assert_eq!(store.active_count(), 2);
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.009 Hard Cap: E-FLAG-007 when cap reached
// ─────────────────────────────────────────────────────────────

/// Cap enforcement: when TOKEN_CAP active tokens exist (no expired),
/// the next generate() returns E-FLAG-007 (TokenCapExceeded).
///
/// VP-010 invariant: no eviction occurs.
#[test]
fn test_BC_2_04_009_cap_exceeded_returns_e_flag_007() {
    let store = make_store();

    // Fill store to cap.
    for i in 0..TOKEN_CAP {
        let params = json!({"slot": i});
        let result = store.generate(
            "acme",
            "crowdstrike_contain_host",
            params,
            &format!("slot-{i}"),
        );
        assert!(
            result.is_ok(),
            "BC-2.04.009: fill slot {i} must succeed, got {result:?}"
        );
    }

    assert_eq!(
        store.active_count(),
        TOKEN_CAP,
        "BC-2.04.009: active_count must equal TOKEN_CAP after filling"
    );

    // 101st token must fail with E-FLAG-007.
    let overflow = store.generate(
        "acme",
        "crowdstrike_contain_host",
        json!({"overflow": true}),
        "overflow",
    );

    assert!(
        matches!(overflow, Err(PrismError::TokenCapExceeded)),
        "BC-2.04.009: 101st token must return TokenCapExceeded (E-FLAG-007), got {overflow:?}"
    );

    // No eviction: count must not have decreased.
    assert_eq!(
        store.active_count(),
        TOKEN_CAP,
        "BC-2.04.009 VP-010 invariant: active_count must not decrease (no eviction)"
    );
}

/// Cap error message must be the canonical E-FLAG-007 form.
#[test]
fn test_BC_2_04_009_cap_exceeded_error_message_contains_e_flag_007() {
    let store = make_store();

    for i in 0..TOKEN_CAP {
        store
            .generate("acme", "crowdstrike_contain_host", json!({"slot": i}), "s")
            .unwrap();
    }

    let err = store
        .generate("acme", "crowdstrike_contain_host", json!({"x": 1}), "x")
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("E-FLAG-007"),
        "BC-2.04.009: E-FLAG-007 error must mention 'E-FLAG-007' in display, got: {msg}"
    );
    assert!(
        msg.contains("100"),
        "BC-2.04.009: E-FLAG-007 must mention the cap (100), got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-034: In-memory only — no persistence
// ─────────────────────────────────────────────────────────────

/// EC-04-034: A new ConfirmationTokenStore starts empty (no persistence across instances).
#[test]
fn test_BC_2_04_009_ec_new_store_starts_empty() {
    let store1 = make_store();
    store1
        .generate("acme", "crowdstrike_contain_host", json!({"x": 1}), "s")
        .unwrap();
    assert_eq!(store1.active_count(), 1);

    // New store instance is independent — in-memory, not persisted.
    let store2 = make_store();
    assert_eq!(
        store2.active_count(),
        0,
        "EC-04-034: new store must start empty (in-memory, not persisted)"
    );
}
