//! [`FidelityValidator`] — Runs behavioral fidelity checks against a stub server.

/// Describes a single fidelity check to execute against a running DTU stub.
///
/// # ADR-003 Amendment #3 (TD-WV1-01)
///
/// The `headers` field allows fidelity checks to probe auth-required endpoints
/// by injecting request headers (e.g. `Authorization: Bearer <token>`).
/// `Vec<(String, String)>` is used instead of `HashMap` to allow duplicate header
/// names, which HTTP permits (RFC 7230 §3.2.2).
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
    ///
    /// Accepts two forms:
    /// - **Flat top-level key**: e.g., `"status"` — checked via `serde_json::Value::get`.
    /// - **JSON pointer (RFC 6901)**: e.g., `"/response/data/id"` — checked via `serde_json::Value::pointer`.
    ///
    /// Implementations detect the form by the leading `/`.
    pub required_fields: Vec<String>,
    /// Additional HTTP headers to inject into the request.
    ///
    /// Allows probing auth-required endpoints with bearer tokens or API keys.
    /// Uses `Vec<(String, String)>` to permit duplicate header names (RFC 7230 §3.2.2).
    pub headers: Vec<(String, String)>,
}

impl Default for FidelityCheck {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: Vec::new(),
            headers: Vec::new(),
        }
    }
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
        // SAFETY: Client::builder() with only a timeout cannot fail unless system
        // TLS is broken — treating this as an infallible invariant is correct.
        #[allow(clippy::expect_used)]
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build reqwest client");

        let mut checks_passed = 0usize;
        let mut checks_failed = 0usize;
        let mut failures: Vec<FidelityFailure> = Vec::new();

        for check in checks {
            let url = format!("{base_url}{}", check.endpoint);
            // SAFETY: check.method comes from http::Method which only contains valid method bytes.
            #[allow(clippy::expect_used)]
            let mut req = client.request(
                reqwest::Method::from_bytes(check.method.as_str().as_bytes())
                    .expect("valid HTTP method"),
                &url,
            );
            if let Some(body) = &check.body {
                req = req.json(body);
            }
            // Inject additional headers (ADR-003 Amendment #3 — TD-WV1-01).
            for (name, value) in &check.headers {
                req = req.header(name.as_str(), value.as_str());
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
                                    let found = if field.starts_with('/') {
                                        body.pointer(field).is_some()
                                    } else {
                                        body.get(field).is_some()
                                    };
                                    if !found {
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use serde_json::json;

    /// Verifies that flat top-level key lookup (no leading `/`) works via `get`.
    #[test]
    fn test_required_fields_flat_key() {
        let body = json!({"status": "ok", "count": 42});

        // Flat key "status" present.
        let found = if "status".starts_with('/') {
            body.pointer("status").is_some()
        } else {
            body.get("status").is_some()
        };
        assert!(found, "flat key 'status' should be found");

        // Flat key "missing" absent.
        let found = if "missing".starts_with('/') {
            body.pointer("missing").is_some()
        } else {
            body.get("missing").is_some()
        };
        assert!(!found, "flat key 'missing' should not be found");
    }

    /// Verifies that JSON pointer (RFC 6901) lookup (leading `/`) works via `pointer`.
    #[test]
    fn test_required_fields_json_pointer() {
        let body = json!({"response": {"data": {"id": "abc-123"}}});

        // JSON pointer "/response/data/id" present.
        let found = if "/response/data/id".starts_with('/') {
            body.pointer("/response/data/id").is_some()
        } else {
            body.get("/response/data/id").is_some()
        };
        assert!(found, "JSON pointer '/response/data/id' should be found");

        // JSON pointer "/response/data/missing" absent.
        let found = if "/response/data/missing".starts_with('/') {
            body.pointer("/response/data/missing").is_some()
        } else {
            body.get("/response/data/missing").is_some()
        };
        assert!(
            !found,
            "JSON pointer '/response/data/missing' should not be found"
        );
    }
}
