//! TD-WV0-04: `/dtu/configure` rejects unknown fields (deny_unknown_fields).
//!
//! - Known field `auth_mode` → 200 OK.
//! - Unknown field `bogus` → 400 Bad Request (not silent accept).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::BehavioralClone;
use prism_dtu_cyberint::CyberintClone;

async fn start_clone() -> (CyberintClone, String) {
    let mut clone = CyberintClone::new().expect("CyberintClone::new must succeed");
    clone.start().await.expect("TD-WV0-04 cyberint: start() must succeed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// Known field → 200 OK.
#[tokio::test]
async fn configure_known_field_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .json(&serde_json::json!({"auth_mode": "reject"}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(resp.status(), 200, "known field must return 200");
}

/// Unknown field → 400 Bad Request.
#[tokio::test]
async fn configure_unknown_field_returns_400() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
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
