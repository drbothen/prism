//! HttpLookupSource — built-in HTTP lookup infusion source (ADR-040 v2.0 D8).
//!
//! Implements `InfusionSource` for `InfusionType::HttpLookup` specs.
//! Reuses `Interpolator`, `extract_at_path`, and `build_http_client_with_timeout`
//! from `pipeline.rs`. Handles credential resolution (AD-017), SSRF validation (CWE-918),
//! and error taxonomy (E-INFUSE-009/010/011).

use prism_core::InfusionError;

use crate::infusion::{HttpLookupConfig, InfusionSource};

/// HTTP lookup enrichment source for `InfusionType::HttpLookup` specs.
///
/// Construction: `HttpLookupSource::new(client, config, spec_path)`.
/// The `client` MUST be built with `build_http_client_with_timeout(30)` (CLAUDE.md §Conventions).
/// SSRF validation runs at construction time (not call time) so misconfigured specs are
/// rejected at registry load, not at query execution.
///
/// Credential values are resolved at call time from `env_var`; they are NEVER stored in
/// struct fields (AD-017 / INV-INFUSE-005).
// Dead-code allow: fields are declared for the stub shape; they are used in enrich_single
// and new() once implemented (todo!() bodies). Removing them would break the type contract.
#[allow(dead_code)]
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
    /// private/loopback address and `PRISM_DTU_MODE` is not set.
    /// Returns `Err(InfusionError::InvalidFieldSpec)` if the path does not exist
    /// when used with path-based resolution.
    pub fn new(
        _client: reqwest::Client,
        _config: HttpLookupConfig,
        _spec_path: impl Into<String>,
    ) -> Result<Self, InfusionError> {
        todo!()
    }
}

impl InfusionSource for HttpLookupSource {
    /// Enrich a single input value via HTTP GET, interpolating `${input}` in the URL template.
    ///
    /// Steps (ADR-040 D8.4):
    /// 1. Resolve credential from env var (E-INFUSE-010 on failure).
    /// 2. Interpolate URL template via `Interpolator::interpolate` from pipeline.rs.
    /// 3. Apply auth per `HttpLookupAuthType`.
    /// 4. Issue HTTP call via `self.client` (non-2xx or network error → E-INFUSE-009).
    /// 5. Parse response as JSON (parse failure → E-INFUSE-009).
    /// 6. Extract `response_path` subtree via `extract_at_path` from pipeline.rs.
    /// 7. Return `Ok(Some(subtree))` or `Ok(None)` if path not found.
    ///
    /// NOTE: This function currently returns `Option<serde_json::Value>` not `Result`.
    /// Error cases panic via todo!() — implementer must update the InfusionSource trait
    /// to return Result or handle errors internally.
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        todo!()
    }

    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        // Default implementation: call enrich_single per item.
        inputs
            .iter()
            .map(|i| self.enrich_single(i, input_type))
            .collect()
    }
}
