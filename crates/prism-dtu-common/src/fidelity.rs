//! [`FidelityValidator`] — Runs behavioral fidelity checks against a stub server.

/// Describes a single fidelity check to execute against a running DTU stub.
#[derive(Debug, Clone)]
pub struct FidelityCheck {
    /// Endpoint path (e.g. `"/api/v1/detects"`).
    pub endpoint: String,
    /// HTTP method for the request.
    pub method: http::Method,
    /// Optional request body.
    pub body: Option<serde_json::Value>,
    /// HTTP status code the stub must return.
    pub expected_status: u16,
    /// JSON field paths that must be present in the response body.
    pub required_fields: Vec<String>,
}

/// Describes a single fidelity check that did not pass.
#[derive(Debug, Clone)]
pub struct FidelityFailure {
    pub endpoint: String,
    pub reason: String,
}

/// Summary of a fidelity validation run.
#[derive(Debug)]
pub struct FidelityReport {
    pub checks_passed: usize,
    pub checks_failed: usize,
    pub failures: Vec<FidelityFailure>,
}

/// Executes a suite of [`FidelityCheck`]s against a running stub server.
pub struct FidelityValidator;

impl FidelityValidator {
    /// Run all `checks` against `base_url` and return a [`FidelityReport`].
    pub async fn run(base_url: &str, checks: Vec<FidelityCheck>) -> FidelityReport {
        todo!("implement FidelityValidator::run per AC-9")
    }
}
