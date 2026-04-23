// S-1.09: BC-2.04.012 — Token Content Hash Verification Prevents Action Tampering
//
// Tests verify:
//   - Same params → same hash (deterministic, canonical JSON with sorted keys).
//   - Different params → different hash (collision-resistance).
//   - Key-order variation → same hash (sorted keys normalize ordering).
//   - Tampered host ID → hash mismatch → E-FLAG-005.
//   - Modified client_id → hash mismatch → rejection.
//   - compute_action_hash is deterministic: same inputs always produce same output.
//   - EC-04-025: key order variation produces same hash.
//   - EC-04-026: token for host A cannot be used to execute on host B.
//   - AC-4: token for contain_host(device_id: "A"), confirm with device_id: "B" → rejected.
//
// Naming: test_BC_2_04_012_<assertion>
#![allow(non_snake_case)]

use serde_json::json;

use prism_core::error::PrismError;
use prism_security::confirmation_token::ConfirmationTokenStore;
use prism_security::content_hash::compute_action_hash;

fn make_store() -> ConfirmationTokenStore {
    ConfirmationTokenStore::new()
}

// ─────────────────────────────────────────────────────────────
// compute_action_hash: determinism and canonicalization
// ─────────────────────────────────────────────────────────────

/// Same inputs always produce the same hash (deterministic).
#[test]
fn test_BC_2_04_012_same_params_produce_same_hash() {
    let params = json!({"device_id": "host-A", "reason": "isolation"});
    let h1 = compute_action_hash("acme", "crowdstrike_contain_host", &params);
    let h2 = compute_action_hash("acme", "crowdstrike_contain_host", &params);
    assert_eq!(
        h1, h2,
        "BC-2.04.012: same inputs must always produce the same hash (deterministic)"
    );
}

/// Different params produce different hashes (collision resistance).
///
/// Canonical test vector: "Host ID tampered" → hash mismatch.
/// EC-04-026: host A vs host B → different hashes.
#[test]
fn test_BC_2_04_012_different_params_produce_different_hashes() {
    let params_a = json!({"device_id": "host-A"});
    let params_b = json!({"device_id": "host-B"});
    let h_a = compute_action_hash("acme", "crowdstrike_contain_host", &params_a);
    let h_b = compute_action_hash("acme", "crowdstrike_contain_host", &params_b);
    assert_ne!(
        h_a, h_b,
        "BC-2.04.012: different action params must produce different hashes"
    );
}

/// EC-04-025 / Canonical test vector "Key order variation": params with different
/// JSON key ordering produce the same hash (sorted keys normalize order).
#[test]
fn test_BC_2_04_012_ec_key_order_variation_same_hash() {
    // {b: 2, a: 1} vs {a: 1, b: 2} — semantically identical.
    let params_v1 = json!({"b": 2, "a": 1});
    let params_v2 = json!({"a": 1, "b": 2});
    let h1 = compute_action_hash("acme", "crowdstrike_contain_host", &params_v1);
    let h2 = compute_action_hash("acme", "crowdstrike_contain_host", &params_v2);
    assert_eq!(
        h1, h2,
        "BC-2.04.012 EC-04-025: key order variation must produce the same hash (sorted keys)"
    );
}

/// Different client_id changes the hash.
#[test]
fn test_BC_2_04_012_different_client_id_changes_hash() {
    let params = json!({"device_id": "host-A"});
    let h_acme = compute_action_hash("acme", "crowdstrike_contain_host", &params);
    let h_beta = compute_action_hash("beta", "crowdstrike_contain_host", &params);
    assert_ne!(
        h_acme, h_beta,
        "BC-2.04.012: different client_id must produce different hash (prevents cross-client replay)"
    );
}

/// Different tool_name changes the hash.
#[test]
fn test_BC_2_04_012_different_tool_name_changes_hash() {
    let params = json!({"item": "malware.exe"});
    let h1 = compute_action_hash("acme", "quarantine_file", &params);
    let h2 = compute_action_hash("acme", "delete_file", &params);
    assert_ne!(
        h1, h2,
        "BC-2.04.012: different tool_name must produce different hash"
    );
}

/// Hash output is 64 lowercase hex characters (SHA-256).
#[test]
fn test_BC_2_04_012_hash_is_sha256_hex_64_chars() {
    let params = json!({"device_id": "host-A"});
    let hash = compute_action_hash("acme", "crowdstrike_contain_host", &params);
    assert_eq!(
        hash.len(),
        64,
        "BC-2.04.012: hash must be 64 chars (SHA-256 hex)"
    );
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "BC-2.04.012: hash must be lowercase hex"
    );
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.012 Postcondition: consume() rejects tampered params
// ─────────────────────────────────────────────────────────────

/// Postcondition: tampered action params at consume() time → E-FLAG-005.
///
/// Canonical test vector: "Host ID tampered".
/// AC-4: contain_host(device_id: "A"), confirm with device_id: "B" → rejected.
/// EC-04-026: token for host A used for host B → mismatch.
#[test]
fn test_BC_2_04_012_tampered_params_rejected_with_e_flag_005() {
    let store = make_store();
    let original_params = json!({"device_id": "host-A"});
    let token = store
        .generate(
            "acme",
            "crowdstrike_contain_host",
            original_params.clone(),
            "Isolate host-A",
        )
        .expect("generate must succeed");

    // AC-4: tampered device_id.
    let tampered_params = json!({"device_id": "host-B"});
    let result = store.consume(&token.token_id, "acme", &tampered_params);

    assert!(
        matches!(result, Err(PrismError::TokenContentHashMismatch { .. })),
        "BC-2.04.012 AC-4 EC-04-026: tampered params must return TokenContentHashMismatch (E-FLAG-005), got {result:?}"
    );
}

/// E-FLAG-005 error is non-retryable.
#[test]
fn test_BC_2_04_012_content_hash_mismatch_error_is_not_retryable() {
    let err = PrismError::TokenContentHashMismatch {
        token_id: "abc".to_string(),
        retryable: false,
    };
    match err {
        PrismError::TokenContentHashMismatch { retryable, .. } => {
            assert!(!retryable, "BC-2.04.012: E-FLAG-005 must be non-retryable");
        }
        _ => panic!("unexpected variant"),
    }
}

/// E-FLAG-005 display contains the error code.
#[test]
fn test_BC_2_04_012_content_hash_mismatch_display_contains_e_flag_005() {
    let err = PrismError::TokenContentHashMismatch {
        token_id: "abc".to_string(),
        retryable: false,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("E-FLAG-005"),
        "BC-2.04.012: TokenContentHashMismatch display must contain 'E-FLAG-005', got: {msg}"
    );
}

/// Adding an extra field to params changes the hash → rejection.
#[test]
fn test_BC_2_04_012_extra_field_in_params_changes_hash() {
    let store = make_store();
    let original = json!({"device_id": "host-A"});
    let token = store
        .generate("acme", "crowdstrike_contain_host", original, "s")
        .unwrap();

    // Add extra field — structural change → hash mismatch.
    let augmented = json!({"device_id": "host-A", "force": true});
    let result = store.consume(&token.token_id, "acme", &augmented);

    assert!(
        matches!(result, Err(PrismError::TokenContentHashMismatch { .. })),
        "BC-2.04.012: extra field in params must produce hash mismatch and be rejected"
    );
}

/// Canonical test vector "Match": original params → hash matches → execution proceeds.
#[test]
fn test_BC_2_04_012_matching_params_succeed() {
    let store = make_store();
    let params = json!({"device_id": "host-A"});
    let token = store
        .generate(
            "acme",
            "crowdstrike_contain_host",
            params.clone(),
            "Isolate host-A",
        )
        .unwrap();

    let result = store.consume(&token.token_id, "acme", &params);
    assert!(
        result.is_ok(),
        "BC-2.04.012 canonical vector 'Match': same params must succeed, got {result:?}"
    );
}
