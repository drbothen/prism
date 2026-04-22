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
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build reqwest client");

        let mut checks_passed = 0usize;
        let mut checks_failed = 0usize;
        let mut failures: Vec<FidelityFailure> = Vec::new();

        for check in checks {
            let url = format!("{base_url}{}", check.endpoint);
            let mut req = client.request(
                reqwest::Method::from_bytes(check.method.as_str().as_bytes())
                    .expect("valid HTTP method"),
                &url,
            );
            if let Some(body) = &check.body {
                req = req.json(body);
            }

            let result = req.send().await;

            match result {
                Err(e) => {
                    checks_failed += 1;
                    failures.push(FidelityFailure {
                        endpoint: check.endpoint.clone(),
                        reason: format!("request failed: {e}"),
                    });
                }
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if status != check.expected_status {
                        checks_failed += 1;
                        failures.push(FidelityFailure {
                            endpoint: check.endpoint.clone(),
                            reason: format!(
                                "expected status {}, got {}",
                                check.expected_status, status
                            ),
                        });
                        continue;
                    }

                    // Check required fields in response body.
                    if check.required_fields.is_empty() {
                        checks_passed += 1;
                    } else {
                        match resp.json::<serde_json::Value>().await {
                            Err(e) => {
                                checks_failed += 1;
                                failures.push(FidelityFailure {
                                    endpoint: check.endpoint.clone(),
                                    reason: format!("failed to parse response body as JSON: {e}"),
                                });
                            }
                            Ok(body) => {
                                let mut field_failures: Vec<String> = Vec::new();
                                for field in &check.required_fields {
                                    if body.get(field).is_none() {
                                        field_failures.push(field.clone());
                                    }
                                }
                                if field_failures.is_empty() {
                                    checks_passed += 1;
                                } else {
                                    checks_failed += 1;
                                    failures.push(FidelityFailure {
                                        endpoint: check.endpoint.clone(),
                                        reason: format!(
                                            "missing required fields: {}",
                                            field_failures.join(", ")
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        FidelityReport {
            checks_passed,
            checks_failed,
            failures,
        }
    }
}
