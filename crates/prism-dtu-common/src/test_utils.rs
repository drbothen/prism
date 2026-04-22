//! Shared assertion helpers and test client builders for DTU integration tests.

/// Assert that a response has the expected HTTP status code.
///
/// Panics with a descriptive message if the status does not match.
pub fn assert_status(response: &reqwest::Response, expected: http::StatusCode) {
    let actual = response.status();
    assert_eq!(
        actual.as_u16(),
        expected.as_u16(),
        "expected HTTP status {expected}, got {actual}"
    );
}

/// Assert that `field` is present as a top-level key in `body`.
///
/// Panics with a descriptive message if the field is absent.
pub fn assert_field_present(body: &serde_json::Value, field: &str) {
    assert!(
        body.get(field).is_some(),
        "expected field '{field}' to be present in response body, but it was absent"
    );
}

/// Assert that the named `header` is present in the response.
///
/// Panics with a descriptive message if the header is absent.
pub fn assert_header_present(response: &reqwest::Response, header: &str) {
    assert!(
        response.headers().contains_key(header),
        "expected header '{header}' to be present in response, but it was absent"
    );
}

/// Build a pre-configured [`reqwest::Client`] suitable for DTU integration tests.
///
/// Uses rustls, disables redirect following, and sets a 5s timeout.
pub fn build_test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build DTU test reqwest client")
}
