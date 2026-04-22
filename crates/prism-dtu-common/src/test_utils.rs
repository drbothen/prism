//! Shared assertion helpers and test client builders for DTU integration tests.

/// Assert that a response has the expected HTTP status code.
///
/// Panics with a descriptive message if the status does not match.
pub fn assert_status(response: &reqwest::Response, expected: http::StatusCode) {
    todo!("implement assert_status")
}

/// Assert that `field` is present as a top-level key in `body`.
///
/// Panics with a descriptive message if the field is absent.
pub fn assert_field_present(body: &serde_json::Value, field: &str) {
    todo!("implement assert_field_present")
}

/// Assert that the named `header` is present in the response.
///
/// Panics with a descriptive message if the header is absent.
pub fn assert_header_present(response: &reqwest::Response, header: &str) {
    todo!("implement assert_header_present")
}

/// Build a pre-configured [`reqwest::Client`] suitable for DTU integration tests.
///
/// Uses rustls, disables certificate verification for localhost stubs, and sets
/// a short connection timeout.
pub fn build_test_client() -> reqwest::Client {
    todo!("implement build_test_client")
}
