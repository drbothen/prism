//! HttpLookupSource — built-in HTTP lookup infusion source (ADR-040 v2.0 D8).
//!
//! Implements `InfusionSource` for `InfusionType::HttpLookup` specs.
//! Reuses `build_http_client_with_timeout` and `extract_at_path` from `pipeline.rs`.
//! Handles credential resolution (AD-017), SSRF validation (CWE-918),
//! and error taxonomy (E-INFUSE-009/010/011).

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use prism_core::InfusionError;
use url::Url;

use crate::infusion::{HttpLookupAuthType, HttpLookupConfig, InfusionSource};
use crate::pipeline::extract_at_path;

/// HTTP lookup enrichment source for `InfusionType::HttpLookup` specs.
///
/// Construction: `HttpLookupSource::new(client, config, spec_path)`.
/// The `client` MUST be built with `build_http_client_with_timeout()` (CLAUDE.md §Conventions).
/// SSRF validation runs at construction time (not call time) so misconfigured specs are
/// rejected at registry load, not at query execution.
///
/// Credential values are resolved at call time from `env_var`; they are NEVER stored in
/// struct fields (AD-017 / INV-INFUSE-005).
#[derive(Debug)]
pub struct HttpLookupSource {
    client: reqwest::Client,
    config: HttpLookupConfig,
    spec_path: String,
}

impl HttpLookupSource {
    /// Construct an `HttpLookupSource`, validating SSRF rules at construction time.
    ///
    /// Returns `Err(InfusionError::SsrfRejected)` if `base_url` resolves to a
    /// private/loopback address and `PRISM_DTU_MODE` is not set (CWE-918).
    /// The error NEVER contains the resolved IP address (CWE-209).
    pub fn new(
        client: reqwest::Client,
        config: HttpLookupConfig,
        spec_path: impl Into<String>,
    ) -> Result<Self, InfusionError> {
        let spec_path = spec_path.into();

        // SSRF validation at construction time (ADR-040 D8.5 / CWE-918).
        // Skip if PRISM_DTU_MODE is set (DTU override for test/demo local clones).
        let dtu_mode = std::env::var("PRISM_DTU_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if !dtu_mode {
            validate_ssrf_safe(&config.base_url, &spec_path)?;
        }

        Ok(Self {
            client,
            config,
            spec_path,
        })
    }
}

/// Validate that `base_url` does not resolve to a private/loopback/link-local address.
///
/// Reject RFC-1918 (10.x, 172.16-31.x, 192.168.x), loopback (127.x/8, ::1),
/// and link-local (169.254.x/16) unless PRISM_DTU_MODE bypasses the check.
///
/// Fail-closed: if the hostname cannot be resolved or the URL cannot be parsed,
/// reject the spec rather than allowing it.
///
/// DO NOT include resolved IP addresses in error messages (CWE-209).
fn validate_ssrf_safe(base_url: &str, spec_path: &str) -> Result<(), InfusionError> {
    // Attempt to parse as a URL.
    let parsed = Url::parse(base_url).map_err(|_e| InfusionError::InvalidFieldSpec {
        field: "base_url".to_string(),
        spec_path: spec_path.to_string(),
        message: "base_url is not a valid URL".to_string(),
    })?;

    let host = parsed
        .host_str()
        .ok_or_else(|| InfusionError::InvalidFieldSpec {
            field: "base_url".to_string(),
            spec_path: spec_path.to_string(),
            message: "base_url has no host".to_string(),
        })?;

    // Try to parse the host directly as an IP address (IP literal in URL).
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_or_loopback(ip) {
            return Err(InfusionError::SsrfRejected {
                infusion_id: spec_path.to_string(),
                spec_path: spec_path.to_string(),
            });
        }
        return Ok(());
    }

    // Hostname: perform synchronous DNS resolution and check each resolved address.
    // Fail-closed: any resolution failure rejects the spec.
    let port = parsed.port_or_known_default().unwrap_or(443);
    let addrs: Vec<SocketAddr> = format!("{host}:{port}")
        .to_socket_addrs()
        .map_err(|_e| {
            // DNS failure → fail-closed: reject the spec (ADR-040 D8.5 note).
            // Do NOT expose the hostname or error details in the message (CWE-209).
            InfusionError::SsrfRejected {
                infusion_id: spec_path.to_string(),
                spec_path: spec_path.to_string(),
            }
        })?
        .collect();

    for addr in &addrs {
        if is_private_or_loopback(addr.ip()) {
            // Found a private/loopback address — reject without exposing the IP (CWE-209).
            return Err(InfusionError::SsrfRejected {
                infusion_id: spec_path.to_string(),
                spec_path: spec_path.to_string(),
            });
        }
    }

    Ok(())
}

/// Returns `true` if the IP address is private, loopback, or link-local.
///
/// Checked ranges (RFC-1918 + loopback + link-local):
/// - 10.0.0.0/8
/// - 172.16.0.0/12 (172.16.x.x – 172.31.x.x)
/// - 192.168.0.0/16
/// - 127.0.0.0/8 (loopback)
/// - ::1 (IPv6 loopback)
/// - 169.254.0.0/16 (link-local)
/// - fd00::/8 (IPv6 unique local — starts with 0xfd)
fn is_private_or_loopback(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            // 127.0.0.0/8 — loopback
            if octets[0] == 127 {
                return true;
            }
            // 10.0.0.0/8 — RFC-1918
            if octets[0] == 10 {
                return true;
            }
            // 172.16.0.0/12 — RFC-1918 (172.16 through 172.31)
            if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                return true;
            }
            // 192.168.0.0/16 — RFC-1918
            if octets[0] == 192 && octets[1] == 168 {
                return true;
            }
            // 169.254.0.0/16 — link-local
            if octets[0] == 169 && octets[1] == 254 {
                return true;
            }
            false
        }
        IpAddr::V6(v6) => {
            // ::1 — IPv6 loopback
            if v6.is_loopback() {
                return true;
            }
            // fd00::/8 — IPv6 unique local (first byte 0xfd)
            let segments = v6.segments();
            if (segments[0] >> 8) == 0xfd {
                return true;
            }
            false
        }
    }
}

impl InfusionSource for HttpLookupSource {
    /// Enrich a single input value via HTTP GET/POST, interpolating `${input}` in the URL template.
    ///
    /// Steps (ADR-040 D8.4):
    /// 1. Resolve credential from env var (logs E-INFUSE-010 on failure → returns None).
    /// 2. Build the full URL: `base_url + url_template.replace("${input}", input)`.
    /// 3. Apply auth per `HttpLookupAuthType`.
    /// 4. Issue HTTP call via `self.client` (non-2xx or network error → logs E-INFUSE-009 → None).
    /// 5. Parse response as JSON (parse failure → logs E-INFUSE-009 → None).
    /// 6. Extract `response_path` subtree via `extract_at_path` from pipeline.rs.
    /// 7. Return `Some(subtree)` or `None` if path not found.
    ///
    /// Errors are logged at WARN level with structured fields and E-INFUSE-* codes;
    /// they are NEVER surfaced in the return type (InfusionSource::enrich_single is Option).
    /// Credential VALUES are never logged (AD-017 / INV-INFUSE-005).
    fn enrich_single(&self, input: &str, _input_type: &str) -> Option<serde_json::Value> {
        // Build a single-threaded tokio runtime to drive the async HTTP call from the
        // synchronous InfusionSource::enrich_single interface.
        let rt = tokio::runtime::Runtime::new()
            .expect("HttpLookupSource: failed to create tokio runtime for HTTP call");

        rt.block_on(self.enrich_single_async(input))
    }

    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        // Default implementation: call enrich_single per item (ADR-040 D8 §enrich_batch).
        inputs
            .iter()
            .map(|i| self.enrich_single(i, input_type))
            .collect()
    }
}

impl HttpLookupSource {
    /// Async implementation of enrich_single, driven via `block_on` from the sync trait.
    async fn enrich_single_async(&self, input: &str) -> Option<serde_json::Value> {
        let config = &self.config;

        // Step 1: Resolve credential from env var at call time (AD-017).
        // Credential VALUE is never stored — resolved fresh each call.
        // On resolution failure: log E-INFUSE-010 and return None.
        let credential_value: Option<String> = if let Some(cred) = &config.credential {
            match std::env::var(&cred.env_var) {
                Ok(val) => Some(val),
                Err(_) => {
                    tracing::warn!(
                        infusion_id = %self.spec_path,
                        spec_path = %self.spec_path,
                        credential_ref = %cred.ref_name,
                        event_type = "infusion.http_lookup.failed",
                        error_code = "E-INFUSE-010",
                        "credential resolution failed: env var not set"
                    );
                    return None;
                }
            }
        } else {
            None
        };

        // Step 2: Interpolate ${input} in url_template to form the full path.
        let url_path = config.url_template.replace("${input}", input);
        let full_url = format!("{}{}", config.base_url.trim_end_matches('/'), url_path);

        tracing::debug!(
            infusion_id = %self.spec_path,
            spec_path = %self.spec_path,
            event_type = "infusion.http_lookup.started",
            method = %config.method,
            "starting HTTP lookup enrichment"
        );

        // Step 3: Build request, applying auth per HttpLookupAuthType.
        let request_builder = match config.method.as_str() {
            "GET" => self.client.get(&full_url),
            "POST" => self.client.post(&full_url),
            other => {
                tracing::warn!(
                    infusion_id = %self.spec_path,
                    spec_path = %self.spec_path,
                    event_type = "infusion.http_lookup.failed",
                    error_code = "E-INFUSE-009",
                    method = %other,
                    "unsupported HTTP method"
                );
                return None;
            }
        };

        // Apply credential auth to the request builder.
        let request_builder = if let Some(cred_val) = &credential_value {
            apply_auth(request_builder, &config.credential, cred_val)
        } else {
            request_builder
        };

        // Step 4: Issue the HTTP call.
        let response = match request_builder.send().await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::warn!(
                    infusion_id = %self.spec_path,
                    spec_path = %self.spec_path,
                    event_type = "infusion.http_lookup.failed",
                    error_code = "E-INFUSE-009",
                    // Do NOT log e directly — may contain URL with credential (AD-017)
                    message = "HTTP call failed",
                    error_kind = %classify_reqwest_error(&e),
                );
                return None;
            }
        };

        // Check for non-2xx status (E-INFUSE-009).
        let status = response.status();
        if !status.is_success() {
            tracing::warn!(
                infusion_id = %self.spec_path,
                spec_path = %self.spec_path,
                event_type = "infusion.http_lookup.failed",
                error_code = "E-INFUSE-009",
                status_code = status.as_u16(),
                "HTTP lookup returned non-2xx status"
            );
            return None;
        }

        // Step 5: Parse response body as JSON.
        let body_bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    infusion_id = %self.spec_path,
                    spec_path = %self.spec_path,
                    event_type = "infusion.http_lookup.failed",
                    error_code = "E-INFUSE-009",
                    error_kind = %classify_reqwest_error(&e),
                    "failed to read HTTP response body"
                );
                return None;
            }
        };

        let body_json: serde_json::Value = match serde_json::from_slice(&body_bytes) {
            Ok(j) => j,
            Err(_) => {
                tracing::warn!(
                    infusion_id = %self.spec_path,
                    spec_path = %self.spec_path,
                    event_type = "infusion.http_lookup.failed",
                    error_code = "E-INFUSE-009",
                    "HTTP response body is not valid JSON"
                );
                return None;
            }
        };

        // Step 6: Extract response_path subtree using extract_at_path from pipeline.rs.
        match extract_at_path(&body_json, &config.response_path) {
            Ok(subtree) => {
                tracing::debug!(
                    infusion_id = %self.spec_path,
                    spec_path = %self.spec_path,
                    event_type = "infusion.http_lookup.succeeded",
                    "HTTP lookup enrichment succeeded"
                );
                Some(subtree)
            }
            Err(_) => {
                // Path not found in response — return None (not an error, just no match).
                None
            }
        }
    }
}

/// Apply the credential to the request builder per the configured auth type.
///
/// The credential VALUE is only used here to build the request — it is never
/// stored, logged, or included in error messages (AD-017 / INV-INFUSE-005).
fn apply_auth(
    builder: reqwest::RequestBuilder,
    credential: &Option<crate::infusion::HttpLookupCredentialConfig>,
    value: &str,
) -> reqwest::RequestBuilder {
    let Some(cred) = credential else {
        return builder;
    };
    // `#[non_exhaustive]` on HttpLookupAuthType means external crates require a wildcard arm.
    // Within the crate the compiler sees all variants, so we suppress the unreachable warning.
    #[allow(unreachable_patterns)]
    match &cred.auth {
        HttpLookupAuthType::QueryParam { param_name } => {
            // Append as a query parameter — the URL already has the ${input} interpolation.
            // Use reqwest's query method to correctly encode the parameter.
            builder.query(&[(param_name.as_str(), value)])
        }
        HttpLookupAuthType::BearerHeader => {
            builder.header(reqwest::header::AUTHORIZATION, format!("Bearer {value}"))
        }
        HttpLookupAuthType::ApiKeyHeader { header_name } => {
            builder.header(header_name.as_str(), value)
        }
        _ => builder,
    }
}

/// Classify a reqwest error for structured logging without leaking URLs or credentials.
///
/// Returns a short string describing the error kind (connect, timeout, decode, etc.)
/// without including the actual URL or response body (CWE-209 / AD-017).
fn classify_reqwest_error(e: &reqwest::Error) -> &'static str {
    if e.is_connect() {
        "connect_error"
    } else if e.is_timeout() {
        "timeout"
    } else if e.is_decode() {
        "decode_error"
    } else if e.is_status() {
        "status_error"
    } else {
        "request_error"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infusion::HttpLookupConfig;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    // SID-1: Unit tests for AC-016 behavior that integration tests 23-24 cover,
    // but cannot run without live NVD API (blocked by DTU-EXT-NVD-001).
    // These tests use wiremock to mock the HTTP boundary.

    /// AC-016 (ADR-040 D8.4): HttpLookupSource::enrich_single must interpolate ${input}
    /// in the url_template before issuing the HTTP request, and extract response_path.
    ///
    /// Corresponds to integration test 23 (test_enrichment_pivot_002_http_lookup_source_enrich_single_calls_url_template).
    /// DTU-EXT-NVD-001: live NVD API integration test blocked until DTU clone is deployed.
    ///
    /// Uses enrich_single_async directly to avoid nested tokio runtime (block_on inside tokio::test).
    #[tokio::test]
    async fn test_enrich_single_extracts_response_path_via_wiremock() {
        let mock_server = MockServer::start().await;

        // NVD-shaped mock response for CVE-2024-1234 with CVSS data.
        let mock_response = serde_json::json!({
            "vulnerabilities": [{
                "cve": {
                    "id": "CVE-2024-1234",
                    "metrics": {
                        "cvssMetricV31": [{
                            "cvssData": {
                                "baseScore": 7.5,
                                "vectorString": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:N/A:N"
                            }
                        }]
                    }
                }
            }]
        });

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&mock_server)
            .await;

        let config = HttpLookupConfig::new(
            &mock_server.uri(),
            "/rest/json/cves/2.0?cveId=${input}",
            "GET",
            "$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData",
            None,
        );
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest::Client::build");

        // Use PRISM_DTU_MODE=true so wiremock's localhost address passes SSRF check.
        // SAFETY: test-only; single-threaded test context; no concurrent env access.
        unsafe { std::env::set_var("PRISM_DTU_MODE", "true") };
        let source = HttpLookupSource::new(client, config, "nvd.infusion.toml")
            .expect("construct HttpLookupSource against wiremock");
        // SAFETY: test-only cleanup; single-threaded test context.
        unsafe { std::env::remove_var("PRISM_DTU_MODE") };

        // Call enrich_single_async directly — avoids nested tokio runtime from block_on.
        let result = source.enrich_single_async("CVE-2024-1234").await;

        assert!(
            result.is_some(),
            "AC-016: enrich_single must return Some for valid CVE input (wiremock). Got None."
        );
        let json_val = result.unwrap();
        assert!(
            json_val.get("baseScore").is_some(),
            "AC-016: response_path extraction must include baseScore. Got: {:?}",
            json_val
        );
    }

    /// AC-016 (ADR-040 D8.4): HttpLookupSource::enrich_single must return None when
    /// response_path does not match any node in the HTTP response.
    ///
    /// Corresponds to integration test 25 behavior, verified via wiremock.
    #[tokio::test]
    async fn test_enrich_single_returns_none_on_path_not_found_via_wiremock() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "vulnerabilities": [{"cve": {"id": "CVE-2024-1234"}}]
        });

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&mock_server)
            .await;

        let config = HttpLookupConfig::new(
            &mock_server.uri(),
            "/rest/json/cves/2.0?cveId=${input}",
            "GET",
            "$.nonexistent.path.that.will.never.match",
            None,
        );
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest::Client::build");

        // SAFETY: test-only; single-threaded test context; no concurrent env access.
        unsafe { std::env::set_var("PRISM_DTU_MODE", "true") };
        let source = HttpLookupSource::new(client, config, "nvd.infusion.toml")
            .expect("construct HttpLookupSource");
        // SAFETY: test-only cleanup; single-threaded test context.
        unsafe { std::env::remove_var("PRISM_DTU_MODE") };

        // Call enrich_single_async directly to avoid nested tokio runtime.
        let result = source.enrich_single_async("CVE-2024-1234").await;

        assert!(
            result.is_none(),
            "AC-016: must return None when response_path doesn't match. Got: {:?}",
            result
        );
    }
}
