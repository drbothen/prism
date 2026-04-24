//! TD-WV0-04: `/dtu/configure` rejects unknown fields (deny_unknown_fields).
//!
//! - Known field `auth_mode` → 200 OK.
//! - Unknown field `bogus` → 400 Bad Request (not silent accept).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::BehavioralClone;
use prism_dtu_nvd::NvdClone;

async fn start_clone() -> (NvdClone, String, String) {
    let mut clone = NvdClone::new().expect("NvdClone::new must succeed");
    clone
        .start()
        .await
        .expect("TD-WV0-04 nvd: start() must succeed");
    let base_url = clone.base_url();
    let token = clone.admin_token().to_string();
    (clone, base_url, token)
}

/// Known field → 200 OK (with valid admin token per ADR-003 Amendment #5).
#[tokio::test]
async fn configure_known_field_returns_200() {
    let (_clone, base_url, token) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &token)
        .json(&serde_json::json!({"auth_mode": "reject"}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(resp.status(), 200, "known field must return 200");
}

/// Unknown field → 400 Bad Request (with valid admin token per ADR-003 Amendment #5).
#[tokio::test]
async fn configure_unknown_field_returns_400() {
    let (_clone, base_url, token) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &token)
        .json(&serde_json::json!({"bogus": "val"}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(
        resp.status(),
        400,
        "unknown field must return 400 Bad Request, not silently accept"
    );
}
