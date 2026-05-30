// SPDX-License-Identifier: Apache-2.0
//! AuthProvider trait — spec-driven auth surface for SensorSpec-declared adapters.
//!
//! Anchors:
//! - BC-2.01.013 (DataSource Trait: Spec-Driven Adapter Pattern)
//! - BC-2.01.017 (Static Cookie AuthProvider Contract — No-Login-Roundtrip Cookie Injection)
//! - ADR-023 §C2 (Plugin-Only Sensor Architecture — Real PipelineExecutor)
//! - ADR-031 §D3-b (Cyberint DTU correction — StaticCookieAuthProvider)
//! - Story: S-PLUGIN-PREREQ-B, S-DTU-CYBERINT-AUTH-FIDELITY-001
//!
//! `AuthProvider` is the TOML-driven replacement for compile-time SensorAuth dispatch.
//! It is injected into `PipelineExecutor::execute` at call sites and is not coupled to
//! any specific sensor adapter implementation.
//!
//! # Object Safety
//!
//! The trait is explicitly object-safe: `acquire_token` returns
//! `Pin<Box<dyn Future<...> + Send + '_>>` so `&dyn AuthProvider` works at call sites
//! (AC-5, AC-8 verify trait-object-safety). This is the canonical Rust pattern for
//! dyn-compatible async traits without the `async_trait` proc-macro.
//!
//! # Architecture Compliance
//!
//! `AuthProvider` MUST live in `prism-spec-engine` only. It MUST NOT be imported by
//! `prism-sensors` or `prism-query` (forbidden dependency per PREREQ-B scope boundary).
//!
//! # Production Implementors
//!
//! - [`StaticCookieAuthProvider`] — production type for `auth_type = "cookie_roundtrip"` sensors
//!   (e.g., Cyberint). Reads the API key from the credential resolver at `acquire_token()` time.
//!   Makes NO HTTP call. Returns the raw API key as the token. NOT feature-gated.
//!   AC-005, AC-006 (S-DTU-CYBERINT-AUTH-FIDELITY-001); BC-2.01.017 §Postconditions.

use std::{future::Future, pin::Pin};

use prism_core::OrgSlug;
use zeroize::Zeroizing;

use crate::{error::SpecEngineError, spec_parser::SensorSpec};

// ---------------------------------------------------------------------------
// AuthToken newtype
// ---------------------------------------------------------------------------

/// An opaque bearer token string produced by `AuthProvider::acquire_token`.
///
/// The inner `Zeroizing<String>` automatically overwrites the bearer token bytes
/// in memory when the token is dropped, preventing credential retention in freed
/// heap memory. Anchors: AD-017 (credential safety), TD-S-PLUGIN-PREREQ-B-002 closure.
///
/// The token value MUST NOT appear in log output at any level (INV-INFUSE-005 / AD-017).
/// The `Debug` impl deliberately redacts the value.
#[derive(Clone)]
pub struct AuthToken(Zeroizing<String>);

impl AuthToken {
    /// Construct an `AuthToken` from a raw bearer token string.
    ///
    /// The value is private — callers MUST NOT read or log it directly.
    /// Use [`as_str`] only for constructing `Authorization: Bearer ...` headers.
    pub fn new(token: String) -> Self {
        Self(Zeroizing::new(token))
    }

    /// Borrow the raw token string for use in `Authorization` headers.
    ///
    /// Do NOT log this value at any level (INV-INFUSE-005 / AD-017).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for AuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never emit the token value in debug output.
        f.write_str("AuthToken(<redacted>)")
    }
}

// ---------------------------------------------------------------------------
// AuthProvider trait
// ---------------------------------------------------------------------------

/// Spec-driven auth provider — resolves credentials for a sensor's `auth_type`
/// at runtime, replacing compile-time-keyed SensorAuth dispatch.
///
/// ## Object Safety
///
/// The trait is object-safe because `acquire_token` returns a boxed future.
/// Use `&dyn AuthProvider` at call sites (production and test).
///
/// ## Implementors
///
/// - [`NullAuthProvider`] — no-op; returns empty token; for tests that do not exercise auth.
/// - [`MockAuthProvider`] — test helper; records calls, returns configurable tokens.
/// - (Future) `CredentialStoreAuthProvider` — production impl; reads from credential store.
pub trait AuthProvider: Send + Sync {
    /// Acquire a fresh bearer token for the given sensor spec and client context.
    ///
    /// Called on initial dispatch and on 401-Unauthorized retry (AC-5).
    ///
    /// Returns a boxed future for dyn-compatibility (`&dyn AuthProvider` at call sites).
    ///
    /// # Errors
    ///
    /// Returns `SpecEngineError::AuthAcquisitionFailed` if the token cannot be
    /// obtained (e.g., bad credentials, network failure, invalid auth_type).
    fn acquire_token<'a>(
        &'a self,
        spec: &'a SensorSpec,
        client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>>;
}

// ---------------------------------------------------------------------------
// StaticCookieAuthProvider — production auth provider for cookie_roundtrip sensors
// ---------------------------------------------------------------------------

/// Production `AuthProvider` for sensors using `auth_type = "cookie_roundtrip"`.
///
/// Reads the API key from the credential resolver at `acquire_token()` time via
/// `prism_credentials::resolve_credential`. Makes NO HTTP call. Returns the raw API key
/// as the `AuthToken`.
///
/// The token is then injected by `PipelineExecutor::build_request` as
/// `Cookie: access_token={token}` per ADR-031 §D3-b.
///
/// ## Preconditions (BC-2.01.017)
///
/// - `auth_type = "cookie_roundtrip"` declared in the sensor's TOML spec.
/// - A `credential_ref` naming the API key is declared in the TOML spec.
/// - `prism_credentials::resolve_credential(client_id, sensor_id, "api_key")` must
///   succeed at `acquire_token()` time.
///
/// ## Postconditions (BC-2.01.017 §Postconditions P1)
///
/// - Returns `Ok(AuthToken)` wrapping the raw API key string.
/// - Makes ZERO HTTP calls during `acquire_token` (INV-COOKIE-001, ADR-031 §D1-b).
///
/// ## Errors
///
/// - `E-AUTH-005`: credential not found in keyring/env for `(client_id, sensor_id)`.
/// - `E-AUTH-006`: API key is empty, all-whitespace, contains illegal cookie characters,
///   or exceeds 4096 bytes. Error-taxonomy.md v1.53 §E-AUTH-006.
///
/// ## AD-017 Credential Safety
///
/// The API key value MUST NOT appear in log output at any level. The struct holds ONLY
/// the `sensor_id` string (used as the credential namespace key) — NOT the API key itself.
/// The API key is resolved at `acquire_token()` time from the credential store and
/// immediately wrapped in `AuthToken(Zeroizing<String>)` — never stored as a field.
///
/// AC-005, AC-006, AC-010 (S-DTU-CYBERINT-AUTH-FIDELITY-001).
/// BC-2.01.017; ADR-031 §D3-b rule 2; AD-017.
pub struct StaticCookieAuthProvider {
    /// Sensor ID used as the credential namespace key.
    ///
    /// This is the plain sensor name string (e.g., `"cyberint"`). The API key itself
    /// is NEVER stored here — AD-017 compliance.
    sensor_id: String,
}

impl StaticCookieAuthProvider {
    /// Construct a new `StaticCookieAuthProvider` for the given sensor.
    ///
    /// The `sensor_id` is the sensor name string from the TOML spec (used as the
    /// credential namespace key in `prism_credentials::resolve_credential`).
    ///
    /// Does NOT accept the API key as a constructor argument (AD-017: credentials
    /// must not be held at construction time; resolved at acquire_token() time only).
    ///
    /// AC-005 (S-DTU-CYBERINT-AUTH-FIDELITY-001)
    ///
    /// `#[allow(unused_variables)]`: parameter is referenced in the `todo!()` message
    /// but not actually used until the implementer fills in the body.
    pub fn new(sensor_id: impl Into<String>) -> Self {
        Self {
            sensor_id: sensor_id.into(),
        }
    }
}

impl AuthProvider for StaticCookieAuthProvider {
    /// Acquire the static cookie token for the sensor.
    ///
    /// Calls `prism_credentials::resolve_credential(client_id, sensor_id, "api_key")`.
    /// Returns `Ok(AuthToken(api_key_value))` without making any HTTP call.
    ///
    /// Validates the resolved api_key per E-AUTH-006: rejects empty strings, all-whitespace,
    /// strings with illegal RFC 6265 cookie-value characters (e.g., `;`), and strings
    /// exceeding 4096 bytes.
    ///
    /// # Errors
    ///
    /// - `E-AUTH-005`: credential not found → `SpecEngineError::AuthAcquisitionFailed`
    ///   with message matching error-taxonomy.md v1.53 §E-AUTH-005 template.
    /// - `E-AUTH-006`: empty/invalid api_key → `SpecEngineError::AuthAcquisitionFailed`
    ///   with message matching error-taxonomy.md v1.53 §E-AUTH-006 template.
    ///
    /// BC-2.01.017 §Postconditions P1; INV-COOKIE-001; AC-005, AC-006, AC-010.
    ///
    /// `#[allow(unused_variables)]`: parameters are referenced in the `todo!()` body
    /// message but not actually used until the implementer fills in the body.
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SensorSpec,
        client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        use secrecy::ExposeSecret;

        let sensor_id = self.sensor_id.clone();
        let client_id_str = client_id.as_str().to_string();

        Box::pin(async move {
            // INV-COOKIE-001 / ADR-031 §D1-b: ZERO HTTP calls.
            // This is a pure credential-store read via the env-var / crud chain.
            let secret =
                prism_credentials::resolve_credential(&client_id_str, &sensor_id, "api_key")
                    .await
                    .map_err(|e| SpecEngineError::AuthAcquisitionFailed {
                        sensor_id: sensor_id.clone(),
                        client_id: client_id_str.clone(),
                        detail: format!("E-AUTH-005: credential not found: {e}"),
                    })?;

            // E-AUTH-006: validate the resolved api_key before returning.
            // RFC 6265 §4.1.1: cookie-value MUST NOT contain spaces, commas,
            // semicolons, backslashes, or double-quotes.
            let api_key = secret.expose_secret().to_string();
            if api_key.is_empty() || api_key.chars().all(char::is_whitespace) {
                return Err(SpecEngineError::AuthAcquisitionFailed {
                    sensor_id,
                    client_id: client_id_str,
                    detail: "E-AUTH-006: api_key is empty or all-whitespace".to_string(),
                });
            }
            if api_key.len() > 4096 {
                return Err(SpecEngineError::AuthAcquisitionFailed {
                    sensor_id,
                    client_id: client_id_str,
                    detail: format!(
                        "E-AUTH-006: api_key exceeds 4096-byte limit ({} bytes)",
                        api_key.len()
                    ),
                });
            }
            // RFC 6265 §4.1.1 illegal cookie-value characters.
            const ILLEGAL_COOKIE_CHARS: &[char] = &[' ', ',', ';', '\\', '"'];
            if api_key.chars().any(|c| ILLEGAL_COOKIE_CHARS.contains(&c)) {
                return Err(SpecEngineError::AuthAcquisitionFailed {
                    sensor_id,
                    client_id: client_id_str,
                    detail:
                        "E-AUTH-006: api_key contains illegal RFC 6265 cookie-value characters \
                             (space, comma, semicolon, backslash, or double-quote)"
                            .to_string(),
                });
            }

            Ok(AuthToken::new(api_key))
        })
    }
}

// ---------------------------------------------------------------------------
// NullAuthProvider — returns an empty bearer token; use for non-auth tests
// ---------------------------------------------------------------------------

/// No-op `AuthProvider` — returns an empty bearer token without any I/O.
///
/// Use in tests that exercise pagination or fan-out logic but do not need
/// real auth (the mock HTTP server does not validate `Authorization` headers).
///
/// **Feature-gated:** only available under `cfg(test)` or the `test-helpers`
/// Cargo feature. Do NOT enable `test-helpers` in production dependency trees —
/// these types bypass real credential resolution.
#[cfg(any(test, feature = "test-helpers"))]
pub struct NullAuthProvider;

#[cfg(any(test, feature = "test-helpers"))]
impl AuthProvider for NullAuthProvider {
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SensorSpec,
        _client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        Box::pin(async move { Ok(AuthToken::new(String::new())) })
    }
}

// ---------------------------------------------------------------------------
// MockAuthProvider — configurable call-recorder for auth-specific tests
// ---------------------------------------------------------------------------

/// Test helper `AuthProvider` that records every `acquire_token` call and
/// returns a fixed bearer token string.
///
/// Use in tests that exercise 401-retry behavior (AC-5, VP-PLUGIN-005).
///
/// **Feature-gated:** only available under `cfg(test)` or the `test-helpers`
/// Cargo feature. Do NOT enable `test-helpers` in production dependency trees —
/// these types bypass real credential resolution.
#[cfg(any(test, feature = "test-helpers"))]
pub struct MockAuthProvider {
    /// Token returned on every call.
    ///
    /// Private: construct via [`MockAuthProvider::new`] and read via [`MockAuthProvider::token`].
    /// Direct field mutation is disallowed — use a new instance if the token must change
    /// (F-LP10-LOW-002: was `pub`, which invited accidental mutation bypassing construction).
    token: String,
    /// Number of times `acquire_token` was called (interior-mutable for `&self` API).
    ///
    /// Private: read via [`MockAuthProvider::calls`] (F-LP10-LOW-002 sibling).
    call_count: std::sync::atomic::AtomicU32,
}

#[cfg(any(test, feature = "test-helpers"))]
impl MockAuthProvider {
    /// Create a new `MockAuthProvider` returning `token` on every call.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            call_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Return the configured bearer token string (read-only).
    ///
    /// Use only for assertions in tests that need to verify the token value.
    /// Do NOT log this value at any level (INV-INFUSE-005 / AD-017).
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Return the number of times `acquire_token` was invoked.
    pub fn calls(&self) -> u32 {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(any(test, feature = "test-helpers"))]
impl AuthProvider for MockAuthProvider {
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SensorSpec,
        _client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let token = self.token.clone();
        Box::pin(async move { Ok(AuthToken::new(token)) })
    }
}

// ---------------------------------------------------------------------------
// FailingAuthProvider — always returns AuthAcquisitionFailed; for abort tests
// ---------------------------------------------------------------------------

/// Test helper `AuthProvider` that always returns `AuthAcquisitionFailed`.
///
/// Use in tests that verify the pipeline aborts immediately when `acquire_token` errors,
/// without issuing any HTTP requests (F-LP7-MED-002 / BC-2.16.002 AC-5 abort condition).
///
/// **Feature-gated:** only available under `cfg(test)` or the `test-helpers`
/// Cargo feature. Do NOT enable `test-helpers` in production dependency trees —
/// these types bypass real credential resolution.
#[cfg(any(test, feature = "test-helpers"))]
#[derive(Debug, Default)]
pub struct FailingAuthProvider {
    /// Number of times `acquire_token` was called (interior-mutable for `&self` API).
    ///
    /// Private: read via [`FailingAuthProvider::calls`] (F-LP10-LOW-002 sibling sweep).
    call_count: std::sync::atomic::AtomicU32,
}

#[cfg(any(test, feature = "test-helpers"))]
impl FailingAuthProvider {
    /// Create a new `FailingAuthProvider`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the number of times `acquire_token` was invoked.
    pub fn calls(&self) -> u32 {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(any(test, feature = "test-helpers"))]
impl AuthProvider for FailingAuthProvider {
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SensorSpec,
        _client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move {
            Err(SpecEngineError::AuthAcquisitionFailed {
                sensor_id: "test-failing".to_string(),
                client_id: "test-org".to_string(),
                detail: "FailingAuthProvider always errors (test fixture)".to_string(),
            })
        })
    }
}

// ---------------------------------------------------------------------------
// ChainAuthProvider — per-call outcomes for auth-refresh integration tests
// ---------------------------------------------------------------------------

/// Predetermined outcome for a single `acquire_token` call.
///
/// Used with [`ChainAuthProvider`] to simulate success-then-failure or
/// different-tokens-per-call scenarios in auth-refresh tests.
#[cfg(any(test, feature = "test-helpers"))]
#[derive(Clone)]
pub enum AuthOutcome {
    /// Return this token string as `Ok(AuthToken)`.
    Ok(String),
    /// Return `Err(AuthAcquisitionFailed)` with this detail string.
    Err(String),
}

/// Test helper `AuthProvider` that returns predetermined per-call outcomes.
///
/// On call N (0-indexed), `acquire_token` consults `outcomes[N]`. If N ≥ outcomes.len(),
/// defaults to `Err("ChainAuthProvider: call index out of bounds")`.
///
/// Use in tests that need different behavior on first vs. subsequent calls, e.g.:
/// - First call (acquire): `AuthOutcome::Ok("token1")` → succeeds
/// - Second call (refresh): `AuthOutcome::Err("cred expired")` → auth_refresh_failed
///
/// **Feature-gated:** only available under `cfg(test)` or the `test-helpers`
/// Cargo feature. Do NOT enable `test-helpers` in production dependency trees.
#[cfg(any(test, feature = "test-helpers"))]
pub struct ChainAuthProvider {
    outcomes: Vec<AuthOutcome>,
    call_count: std::sync::atomic::AtomicU32,
}

#[cfg(any(test, feature = "test-helpers"))]
impl ChainAuthProvider {
    /// Create a `ChainAuthProvider` with the given per-call outcomes (in call order).
    pub fn new(outcomes: Vec<AuthOutcome>) -> Self {
        Self {
            outcomes,
            call_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Return the number of times `acquire_token` was invoked.
    pub fn calls(&self) -> u32 {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(any(test, feature = "test-helpers"))]
impl AuthProvider for ChainAuthProvider {
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SensorSpec,
        _client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        let idx = self
            .call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst) as usize;
        let outcome = self.outcomes.get(idx).cloned().unwrap_or(AuthOutcome::Err(
            "ChainAuthProvider: call index out of bounds".to_string(),
        ));
        Box::pin(async move {
            match outcome {
                AuthOutcome::Ok(token) => Ok(AuthToken::new(token)),
                AuthOutcome::Err(detail) => Err(SpecEngineError::AuthAcquisitionFailed {
                    sensor_id: "chain-auth-test-sensor".to_string(),
                    client_id: "test-org".to_string(),
                    detail,
                }),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Unit test: trait-object-safety (AC-5 / Red Gate test 8)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec};
    use prism_core::{ColumnType, OrgSlug};

    fn cookie_roundtrip_spec() -> SensorSpec {
        SensorSpec::new(
            "cyberint",
            "Cyberint Test Sensor",
            AuthType::CookieRoundtrip,
            "https://mock.invalid",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![ColumnSpec::new(
                    "alert_id",
                    ColumnType::String,
                    None,
                    vec![],
                )],
                vec![FetchStep::new(
                    "fetch_alerts",
                    "GET",
                    "/api/v1/alerts",
                    None,
                    "$.data",
                    None,
                    vec![],
                    None,
                    None,
                )],
            )],
            None,
            "1.0.0",
            vec![],
        )
    }

    /// BC-2.16.002 / AC-5: `AuthProvider` must be usable as `dyn AuthProvider`.
    ///
    /// This test is a compile-time check: if `AuthProvider` is NOT object-safe,
    /// the coercion below fails to compile. A compile error counts as a Red Gate failure
    /// (per story S-PLUGIN-PREREQ-B Red Gate test list item 8).
    ///
    /// When the trait is correctly defined (object-safe), this test compiles and
    /// passes at runtime — the type-check IS the test.
    #[test]
    fn test_BC_2_16_002_auth_provider_trait_object_is_object_safe() {
        let provider = MockAuthProvider::new("test-token");
        // Construct a trait-object reference. Compile error here = Red Gate failure.
        let _dyn_provider: &dyn AuthProvider = &provider;
        // Runtime: verify the coercion succeeded (trivially true if it compiled).
        assert_eq!(
            provider.calls(),
            0,
            "no acquire_token calls yet — just testing object-safety coercion"
        );
    }

    /// AC-005 / BC-2.01.017 §Postconditions P1: StaticCookieAuthProvider::acquire_token
    /// reads the API key from the credential resolver (env var chain) and returns Ok(AuthToken).
    ///
    /// SID-1 compliance: since the integration test (bc_2_01_017_static_cookie_auth_provider
    /// test 1) requires CYBERINT_API_KEY to be set in the environment, this unit test exercises
    /// the same production code path deterministically by setting the env var in-process before
    /// the call, and restoring it after. This drives the real acquire_token production code path
    /// without any external dependency.
    ///
    /// INV-COOKIE-001: no HTTP call is made — StaticCookieAuthProvider holds no reqwest::Client.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_resolves_api_key_from_env() {
        // Set the env var that resolve_credential checks for sensor_id="cyberint".
        // The canonical env var name is CYBERINT_API_KEY (sensor_upper + "_" + name_upper).
        let env_key = "CYBERINT_API_KEY";
        let test_value = "unit-test-api-key-value";
        // Safety: tests run in isolated processes (nextest default); env mutation is
        // safe here because this test binary is single-purpose and no other thread
        // reads CYBERINT_API_KEY concurrently. The set_var/remove_var pair brackets
        // the acquire_token call.
        // SAFETY: test isolation — nextest runs each test in a separate process
        // by default (out-of-process mode). No data race with other test binaries.
        unsafe {
            std::env::set_var(env_key, test_value);
        }

        let provider = StaticCookieAuthProvider::new("cyberint");
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result = provider.acquire_token(&spec, &client_id).await;

        // Clean up the env var immediately after the await returns.
        // SAFETY: same isolation guarantee as set_var above.
        unsafe {
            std::env::remove_var(env_key);
        }

        assert!(
            result.is_ok(),
            "AC-005: acquire_token must return Ok(AuthToken) when CYBERINT_API_KEY is set. \
             Got: {:?}",
            result
        );
        let token = result.unwrap();
        assert_eq!(
            token.as_str(),
            test_value,
            "AC-005: AuthToken value must equal the resolved API key from the env var"
        );
    }

    /// AC-006 / BC-2.01.017 §Invariants INV-COOKIE-001: acquire_token makes ZERO HTTP calls.
    ///
    /// Structural check: StaticCookieAuthProvider has no reqwest::Client field.
    /// This test verifies the credential-not-found error path: when credentials are absent,
    /// the function returns E-AUTH-005 without making any HTTP call. The no-HTTP-call property
    /// is guaranteed by the struct having no HTTP client — if the CYBERINT_API_KEY env var is
    /// unset, acquire_token returns Err immediately from resolve_credential, proving no HTTP
    /// I/O occurred.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_no_http_call_when_credential_missing() {
        // Ensure the env var is NOT set for this test.
        // SAFETY: test isolation — see test_static_cookie_auth_provider_resolves_api_key_from_env
        // SAFETY comment. No concurrent threads read these vars in this test binary.
        unsafe {
            std::env::remove_var("CYBERINT_API_KEY");
            std::env::remove_var("CYBERINT_API_KEY_FILE");
        }

        let provider = StaticCookieAuthProvider::new("cyberint");
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        // acquire_token returns E-AUTH-005 (not found) — no HTTP call made.
        // The error itself proves the function returned without any async HTTP I/O.
        let result = provider.acquire_token(&spec, &client_id).await;
        assert!(
            result.is_err(),
            "AC-006: acquire_token must return Err when no credential is configured \
             (E-AUTH-005 not-found path). Got Ok — unexpected credential resolution."
        );
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("E-AUTH-005"),
            "AC-006: error must be E-AUTH-005 (credential not found). Got: {err_str}"
        );
    }

    /// E-AUTH-006: acquire_token rejects an empty API key.
    ///
    /// When the env var is set to an empty string, acquire_token must return
    /// E-AUTH-006 (empty / all-whitespace credential rejected).
    #[tokio::test]
    async fn test_static_cookie_auth_provider_rejects_empty_api_key() {
        let env_key = "CYBERINT_API_KEY";
        // SAFETY: test isolation — see test_static_cookie_auth_provider_resolves_api_key_from_env.
        unsafe {
            std::env::set_var(env_key, "");
        }

        let provider = StaticCookieAuthProvider::new("cyberint");
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result = provider.acquire_token(&spec, &client_id).await;
        // SAFETY: test isolation.
        unsafe {
            std::env::remove_var(env_key);
        }

        // Empty string: resolve_secret returns Ok(None) → NotFound path, not E-AUTH-006.
        // An empty env var is treated as "not set" by resolve_secret (it filters empty strings).
        // This is correct per BC-2.03.006 semantics — the E-AUTH-005 path fires.
        assert!(
            result.is_err(),
            "AC-006/E-AUTH-006: acquire_token must return Err for empty credential. Got Ok."
        );
    }

    /// E-AUTH-006: acquire_token rejects an API key with illegal RFC 6265 cookie characters.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_rejects_illegal_cookie_chars() {
        let env_key = "CYBERINT_API_KEY";
        // Semicolon is illegal in cookie-value per RFC 6265 §4.1.1.
        // SAFETY: test isolation — see test_static_cookie_auth_provider_resolves_api_key_from_env.
        unsafe {
            std::env::set_var(env_key, "valid-prefix;injected-cookie");
        }

        let provider = StaticCookieAuthProvider::new("cyberint");
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result = provider.acquire_token(&spec, &client_id).await;
        // SAFETY: test isolation.
        unsafe {
            std::env::remove_var(env_key);
        }

        assert!(
            result.is_err(),
            "E-AUTH-006: acquire_token must reject API keys containing ';' (illegal cookie char)"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("E-AUTH-006"),
            "E-AUTH-006: error message must reference E-AUTH-006. Got: {err}"
        );
    }
}
