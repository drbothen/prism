// SPDX-License-Identifier: Apache-2.0
//! AuthProvider trait — spec-driven auth surface for SensorSpec-declared adapters.
//!
//! Anchors:
//! - BC-2.01.013 (DataSource Trait: Spec-Driven Adapter Pattern)
//! - BC-2.01.017 (StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection)
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

use std::{future::Future, pin::Pin, sync::Arc};

use prism_core::OrgSlug;
use prism_credentials::CredentialResolutionError;
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
// CredentialResolver trait — injectable credential resolution for DI / testability
// ---------------------------------------------------------------------------

/// Injectable credential resolver used by [`StaticCookieAuthProvider`].
///
/// Abstracts over `prism_credentials::resolve_credential` so that tests can inject
/// a mock resolver without touching the real credential store (ADR-022 §C wiring; AC-005).
///
/// # Object Safety
///
/// The trait is object-safe: `resolve` returns a boxed future.
/// Use `Arc<dyn CredentialResolver>` at construction sites.
///
/// # Production Implementor
///
/// [`PrismCredentialResolver`] wraps `prism_credentials::resolve_credential`.
/// Constructed via `Arc::new(PrismCredentialResolver::new(org_registry, keyring))` in `StaticCookieAuthProvider::new`.
pub trait CredentialResolver: Send + Sync {
    /// Resolve the named credential for the given `(client_id, sensor_id, credential_name)` tuple.
    ///
    /// Returns the raw credential value as a `SecretString`, or a typed `CredentialResolutionError`.
    ///
    /// # Errors
    ///
    /// - `CredentialResolutionError::NotFound` — credential name is not registered for this
    ///   `(client_id, sensor_id)` tuple. Callers should map this to `E-AUTH-005`.
    /// - `CredentialResolutionError::BackendUnavailable` — credential is configured but the
    ///   backing store (file, env, keyring) is inaccessible at resolution time. Callers should
    ///   map this to `E-AUTH-007` (BC-2.01.017 EC-017-010).
    fn resolve<'a>(
        &'a self,
        client_id: &'a str,
        sensor_id: &'a str,
        credential_name: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<secrecy::SecretString, CredentialResolutionError>>
                + Send
                + 'a,
        >,
    >;
}

// ---------------------------------------------------------------------------
// PrismCredentialResolver — production impl wrapping prism_credentials
// ---------------------------------------------------------------------------

/// Production [`CredentialResolver`] that delegates to `prism_credentials::resolve_credential`.
///
/// Holds the `OrgRegistry` (for slug→OrgId resolution) and the `CredentialStoreOrgId`-capable
/// keyring backend (for Tier-3 resolution). Constructed via `new(org_registry, keyring)`.
///
/// At `acquire_token()` time it:
///   1. Resolves the org slug → `OrgId` via `OrgRegistry::resolve` (in `prism-spec-engine`,
///      which CAN import `OrgRegistry`; `prism-credentials` MUST NOT — ADR-034 §D1 compliance).
///   2. Calls `prism_credentials::resolve_credential` with the pre-resolved `OrgId` and `keyring`.
///
/// AD-017: no credential value is stored in this struct.
///
/// ADR-034 §D1: this is the canonical production implementation. Unit struct form removed.
pub struct PrismCredentialResolver {
    /// OrgRegistry for slug → OrgId resolution before calling `resolve_credential`.
    ///
    /// `prism-credentials` MUST NOT import `OrgRegistry` (architecture compliance rule
    /// in `trait_.rs:84–85`). Resolution happens here, in `prism-spec-engine`.
    org_registry: Arc<prism_core::OrgRegistry>,
    /// OrgId-keyed keyring backend for Tier-3 resolution (ADR-034 §D2).
    keyring: Arc<dyn prism_credentials::CredentialStoreOrgId>,
}

impl PrismCredentialResolver {
    /// Construct a `PrismCredentialResolver` with OrgRegistry and keyring DI.
    ///
    /// ADR-034 §D1: the only production constructor. Unit struct form removed.
    pub fn new(
        org_registry: Arc<prism_core::OrgRegistry>,
        keyring: Arc<dyn prism_credentials::CredentialStoreOrgId>,
    ) -> Self {
        Self {
            org_registry,
            keyring,
        }
    }
}

impl CredentialResolver for PrismCredentialResolver {
    fn resolve<'a>(
        &'a self,
        client_id: &'a str,
        sensor_id: &'a str,
        credential_name: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<secrecy::SecretString, CredentialResolutionError>>
                + Send
                + 'a,
        >,
    > {
        let client_id = client_id.to_string();
        let sensor_id = sensor_id.to_string();
        let credential_name = credential_name.to_string();
        let org_registry = Arc::clone(&self.org_registry);
        let keyring = Arc::clone(&self.keyring);
        Box::pin(async move {
            // ADR-034 §D1: slug → OrgId resolution happens here (in prism-spec-engine,
            // not in prism-credentials). The resolved OrgId is passed to resolve_credential.
            // OrgSlug::new infallibly constructs from any &str (validated by OrgRegistry at
            // registration time; resolve() returns None if slug is not registered).
            let org_slug = prism_core::OrgSlug::new(&client_id);
            let org_id = org_registry.resolve(&org_slug);

            // Propagate typed CredentialResolutionError directly — no string erasure.
            // NotFound → caller maps to E-AUTH-005; BackendUnavailable → E-AUTH-007
            // (BC-2.01.017 EC-017-010 / E-AUTH-007 in error-taxonomy.md).
            prism_credentials::resolve_credential(
                &client_id,
                &sensor_id,
                &credential_name,
                org_id.as_ref(),
                Some(&keyring),
            )
            .await
        })
    }
}

// ---------------------------------------------------------------------------
// MockCredentialResolver — test helper (cfg(test) / test-helpers feature)
// ---------------------------------------------------------------------------

/// Test helper [`CredentialResolver`] that returns a fixed credential value.
///
/// Inject via `StaticCookieAuthProvider::new_with_resolver(sensor_id, Arc::new(MockCredentialResolver::new("key-value")))`.
///
/// **Feature-gated:** only available under `cfg(test)` or the `test-helpers` Cargo feature.
#[cfg(any(test, feature = "test-helpers"))]
pub struct MockCredentialResolver {
    value: String,
}

#[cfg(any(test, feature = "test-helpers"))]
impl MockCredentialResolver {
    /// Create a `MockCredentialResolver` that always returns `value` as the resolved credential.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[cfg(any(test, feature = "test-helpers"))]
impl CredentialResolver for MockCredentialResolver {
    fn resolve<'a>(
        &'a self,
        _client_id: &'a str,
        _sensor_id: &'a str,
        _credential_name: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<secrecy::SecretString, CredentialResolutionError>>
                + Send
                + 'a,
        >,
    > {
        let value = self.value.clone();
        Box::pin(async move { Ok(secrecy::SecretString::new(value)) })
    }
}

/// Test helper [`CredentialResolver`] that always returns `CredentialResolutionError::NotFound`.
///
/// Use this to test the E-AUTH-005 (credential not found) path without relying on env
/// vars or the real credential store. This is the correct injection mechanism for
/// tests that need to verify "no credential configured" behavior.
///
/// **Feature-gated:** only available under `cfg(test)` or the `test-helpers` Cargo feature.
#[cfg(any(test, feature = "test-helpers"))]
pub struct NotFoundCredentialResolver;

#[cfg(any(test, feature = "test-helpers"))]
impl CredentialResolver for NotFoundCredentialResolver {
    fn resolve<'a>(
        &'a self,
        client_id: &'a str,
        sensor_id: &'a str,
        credential_name: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<secrecy::SecretString, CredentialResolutionError>>
                + Send
                + 'a,
        >,
    > {
        let err = CredentialResolutionError::NotFound {
            client_id: client_id.to_string(),
            sensor_id: sensor_id.to_string(),
            credential_name: credential_name.to_string(),
            suggestion: "NotFoundCredentialResolver: no credential configured (test fixture)"
                .to_string(),
        };
        Box::pin(async move { Err(err) })
    }
}

/// Test helper [`CredentialResolver`] that always returns `CredentialResolutionError::BackendUnavailable`.
///
/// Use this to test the E-AUTH-007 (backend unavailable) path without relying on env
/// vars or the real credential store. This is the correct injection mechanism for
/// tests that need to verify "backend unreachable" behavior.
///
/// BC-2.01.017 EC-017-010 / TV-BC-2.01.017-009 / E-AUTH-007 in error-taxonomy.md.
///
/// **Feature-gated:** only available under `cfg(test)` or the `test-helpers` Cargo feature.
/// AD-017: no credential value is stored in this struct.
#[cfg(any(test, feature = "test-helpers"))]
pub struct BackendUnavailableCredentialResolver;

#[cfg(any(test, feature = "test-helpers"))]
impl CredentialResolver for BackendUnavailableCredentialResolver {
    fn resolve<'a>(
        &'a self,
        client_id: &'a str,
        sensor_id: &'a str,
        credential_name: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<secrecy::SecretString, CredentialResolutionError>>
                + Send
                + 'a,
        >,
    > {
        let err = CredentialResolutionError::BackendUnavailable {
            client_id: client_id.to_string(),
            sensor_id: sensor_id.to_string(),
            credential_name: credential_name.to_string(),
            detail: "BackendUnavailableCredentialResolver: backend unreachable (test fixture)"
                .to_string(),
        };
        Box::pin(async move { Err(err) })
    }
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
/// - `prism_credentials::resolve_credential(client_id, sensor_id, "api_key", org_id, keyring)`
///   (5-arg signature per ADR-034 §D1) must succeed at `acquire_token()` time.
///
/// ## Postconditions (BC-2.01.017 §Postconditions P1)
///
/// - Returns `Ok(AuthToken)` wrapping the raw API key string.
/// - Makes ZERO HTTP calls during `acquire_token` (INV-COOKIE-001, ADR-031 §D1-b).
///
/// ## Errors
///
/// - `E-AUTH-005`: credential not found for `(client_id, sensor_id)`.
///   Resolver returned `CredentialResolutionError::NotFound`.
/// - `E-AUTH-006`: API key is empty, all-whitespace, contains illegal cookie characters,
///   or exceeds 4096 bytes. See E-AUTH-006 in error-taxonomy.md.
/// - `E-AUTH-007`: credential backend unavailable (backend configured but inaccessible).
///   Resolver returned `CredentialResolutionError::BackendUnavailable`.
///   BC-2.01.017 EC-017-010 / E-AUTH-007 in error-taxonomy.md.
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
    /// Injected credential resolver (ADR-022 §C wiring; AC-005 injectable design).
    ///
    /// Production code passes `Arc::new(PrismCredentialResolver::new(org_registry, keyring))` (via `new()`).
    /// Tests pass `Arc::new(MockCredentialResolver::new("key-value"))` (via `new_with_resolver()`).
    resolver: Arc<dyn CredentialResolver>,
}

impl StaticCookieAuthProvider {
    /// Construct a new `StaticCookieAuthProvider` for the given sensor.
    ///
    /// Uses the production [`PrismCredentialResolver`] with injected `OrgRegistry` and
    /// `CredentialStoreOrgId` keyring backend for Tier-3 resolution (ADR-034 §D1).
    ///
    /// The `sensor_id` is the sensor name string from the TOML spec (used as the
    /// credential namespace key in the credential resolver).
    ///
    /// Does NOT accept the API key as a constructor argument (AD-017: credentials
    /// must not be held at construction time; resolved at acquire_token() time only).
    ///
    /// AC-005 (S-DTU-CYBERINT-AUTH-FIDELITY-001); ADR-034 §D1.
    pub fn new(
        sensor_id: impl Into<String>,
        org_registry: Arc<prism_core::OrgRegistry>,
        keyring: Arc<dyn prism_credentials::CredentialStoreOrgId>,
    ) -> Self {
        Self {
            sensor_id: sensor_id.into(),
            resolver: Arc::new(PrismCredentialResolver::new(org_registry, keyring)),
        }
    }

    /// Construct a `StaticCookieAuthProvider` with an injectable credential resolver.
    ///
    /// Used by tests to inject a [`MockCredentialResolver`] without relying on the real
    /// credential store (env vars, keyring). Follows ADR-022 §C wiring contract.
    ///
    /// # Example (test code)
    ///
    /// ```ignore
    /// let provider = StaticCookieAuthProvider::new_with_resolver(
    ///     "cyberint",
    ///     Arc::new(MockCredentialResolver::new("test-api-key-abc123")),
    /// );
    /// ```
    ///
    /// AC-005 (S-DTU-CYBERINT-AUTH-FIDELITY-001); ADR-022 §C; INV-COOKIE-001.
    pub fn new_with_resolver(
        sensor_id: impl Into<String>,
        resolver: Arc<dyn CredentialResolver>,
    ) -> Self {
        Self {
            sensor_id: sensor_id.into(),
            resolver,
        }
    }
}

impl AuthProvider for StaticCookieAuthProvider {
    /// Acquire the static cookie token for the sensor.
    ///
    /// Calls `prism_credentials::resolve_credential(client_id, sensor_id, "api_key",
    /// org_id, keyring)` (5-arg signature per ADR-034 §D1).
    /// Returns `Ok(AuthToken(api_key_value))` without making any HTTP call.
    ///
    /// Validates the resolved api_key per E-AUTH-006: rejects empty strings, all-whitespace,
    /// strings with illegal RFC 6265 cookie-value characters (e.g., `;`), and strings
    /// exceeding 4096 bytes.
    ///
    /// # Errors
    ///
    /// - `E-AUTH-005`: credential not found → `SpecEngineError::AuthAcquisitionFailed`
    ///   with message matching the E-AUTH-005 template in error-taxonomy.md.
    ///   Resolver returned `CredentialResolutionError::NotFound`.
    /// - `E-AUTH-006`: empty/invalid api_key → `SpecEngineError::AuthAcquisitionFailed`
    ///   with message matching the E-AUTH-006 template in error-taxonomy.md.
    /// - `E-AUTH-007`: credential backend unavailable → `SpecEngineError::AuthAcquisitionFailed`
    ///   with message matching the E-AUTH-007 template in error-taxonomy.md.
    ///   Resolver returned `CredentialResolutionError::BackendUnavailable`.
    ///   BC-2.01.017 EC-017-010.
    ///
    /// BC-2.01.017 §Postconditions P1; INV-COOKIE-001; AC-005, AC-006, AC-010.
    ///
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SensorSpec,
        client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        use secrecy::ExposeSecret;

        let sensor_id = self.sensor_id.clone();
        let client_id_str = client_id.as_str().to_string();
        let resolver = Arc::clone(&self.resolver);

        Box::pin(async move {
            // INV-COOKIE-001 / ADR-031 §D1-b: ZERO HTTP calls.
            // Delegates to the injected CredentialResolver — production uses
            // PrismCredentialResolver (wraps prism_credentials::resolve_credential);
            // tests inject MockCredentialResolver without touching the real store.
            //
            // Variant dispatch (BC-2.01.017 EC-017-010 / error-taxonomy.md):
            //   NotFound           → E-AUTH-005 (credential not configured for this tuple)
            //   BackendUnavailable → E-AUTH-007 (backend unreachable at resolution time)
            let secret = match resolver
                .resolve(&client_id_str, &sensor_id, "api_key")
                .await
            {
                Ok(s) => s,
                Err(CredentialResolutionError::NotFound { .. }) => {
                    return Err(SpecEngineError::AuthAcquisitionFailed {
                        sensor_id: sensor_id.clone(),
                        client_id: client_id_str.clone(),
                        detail:
                            "E-AUTH-005: credential not found — no api_key configured for this \
                                 (client_id, sensor_id) tuple. Run `configure_credential_source` \
                                 or set the appropriate env var."
                                .to_string(),
                    });
                }
                Err(CredentialResolutionError::BackendUnavailable { detail, .. }) => {
                    return Err(SpecEngineError::AuthAcquisitionFailed {
                        sensor_id: sensor_id.clone(),
                        client_id: client_id_str.clone(),
                        detail: format!(
                            "E-AUTH-007: credential backend unavailable — {detail}. \
                             Check that the configured backend (env file, keyring, vault) \
                             is accessible in the current execution environment. \
                             BC-2.01.017 EC-017-010."
                        ),
                    });
                }
            };

            // E-AUTH-006: validate the resolved api_key before returning.
            // RFC 6265 §4.1.1: cookie-value MUST NOT contain spaces, commas,
            // semicolons, backslashes, or double-quotes.
            //
            // Validation order (F-LP3-LOW-001 fix — Option a: length check FIRST):
            //   1. Length > 4096 — catches oversized keys (including all-whitespace ones)
            //      with accurate "exceeds 4096-byte limit" detail.
            //   2. Empty or all-whitespace — catches the short empty/whitespace case.
            //   3. RFC 6265 illegal characters.
            let api_key = secret.expose_secret().to_string();
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
            if api_key.is_empty() || api_key.chars().all(char::is_whitespace) {
                return Err(SpecEngineError::AuthAcquisitionFailed {
                    sensor_id,
                    client_id: client_id_str,
                    detail: "E-AUTH-006: api_key is empty or all-whitespace".to_string(),
                });
            }
            // RFC 6265 §4.1.1 illegal cookie-value characters.
            //
            // cookie-octet grammar forbids:
            //   - ASCII control characters (0x00-0x1F incl. CR 0x0D / LF 0x0A) and DEL (0x7F)
            //   - Space (0x20), double-quote (0x22), comma (0x2C), semicolon (0x3B), backslash (0x5C)
            //
            // Defense-in-depth (SEC-001 / CWE-93 / CWE-113): CTL chars (especially CRLF)
            // are rejected here rather than at reqwest send time, producing an actionable
            // E-AUTH-006 instead of a cryptic InvalidHeaderValue / HttpRequestFailed.
            const ILLEGAL_COOKIE_CHARS: &[char] = &[' ', ',', ';', '\\', '"'];
            let has_ctl = api_key.bytes().any(|b| b < 0x21 || b == 0x7F);
            if has_ctl || api_key.chars().any(|c| ILLEGAL_COOKIE_CHARS.contains(&c)) {
                return Err(SpecEngineError::AuthAcquisitionFailed {
                    sensor_id,
                    client_id: client_id_str,
                    detail:
                        "E-AUTH-006: api_key contains illegal RFC 6265 cookie-value characters \
                             (CTL chars, space, comma, semicolon, backslash, or double-quote)"
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
    use prism_core::{ColumnType, OrgSlug};

    use super::*;
    use crate::spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec};

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
    /// injects the resolved API key as the AuthToken value.
    ///
    /// Verifies the StaticCookieAuthProvider's contract: given a resolved credential value,
    /// acquire_token returns Ok(AuthToken) containing that value, with ZERO HTTP calls
    /// (INV-COOKIE-001). Uses MockDI injection (MockCredentialResolver via new_with_resolver)
    /// to decouple StaticCookieAuthProvider's contract from the credential backend
    /// (BC-2.03.006 env-var resolver chain is tested in prism-credentials, not here).
    ///
    /// ADR-022 §C wiring; INV-COOKIE-001; BC-2.01.017 §Postconditions P1.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_injects_resolved_token_from_credentials() {
        let test_value = "unit-test-api-key-value";
        // Inject the resolved credential value via MockCredentialResolver — no env var
        // mutation required. This tests that StaticCookieAuthProvider correctly wraps the
        // resolved value in AuthToken and returns Ok. BC-2.03.006 env-var chain coverage
        // lives in prism-credentials unit tests (where the resolver backend lives).
        let provider = StaticCookieAuthProvider::new_with_resolver(
            "cyberint",
            Arc::new(MockCredentialResolver::new(test_value)),
        );
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result = provider.acquire_token(&spec, &client_id).await;

        assert!(
            result.is_ok(),
            "AC-005: acquire_token must return Ok(AuthToken) when credential is resolved. \
             Got: {:?}",
            result
        );
        let token = result.unwrap();
        assert_eq!(
            token.as_str(),
            test_value,
            "AC-005: AuthToken value must equal the injected resolved credential value. \
             BC-2.01.017 §Postconditions P1."
        );
    }

    /// AC-006 / BC-2.01.017 §Invariants INV-COOKIE-001: acquire_token makes ZERO HTTP calls.
    ///
    /// Structural check: StaticCookieAuthProvider has no reqwest::Client field.
    /// This test verifies the credential-not-found error path: when credentials are absent,
    /// the function returns E-AUTH-005 without making any HTTP call. The no-HTTP-call property
    /// is guaranteed by the struct having no HTTP client — the injected NotFoundCredentialResolver
    /// returns Err immediately, and acquire_token propagates it as E-AUTH-005 with no I/O.
    ///
    /// Uses MockDI injection (NotFoundCredentialResolver via new_with_resolver) — no env var
    /// mutation required. ADR-022 §C; INV-COOKIE-001; BC-2.01.017.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_no_http_call_when_credential_missing() {
        let provider = StaticCookieAuthProvider::new_with_resolver(
            "cyberint",
            Arc::new(NotFoundCredentialResolver),
        );
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

    /// E-AUTH-005: acquire_token returns E-AUTH-005 when the credential resolver returns Err.
    ///
    /// Verifies BC-2.01.017 EC-017-003 (credential not found path):
    /// when resolve_credential returns Err (not-found), acquire_token wraps the error
    /// as E-AUTH-005 and returns immediately without any HTTP call.
    ///
    /// Uses MockDI injection (NotFoundCredentialResolver via new_with_resolver) — no env var
    /// mutation required. TD-VSDD-059 load-bearing assertion: checks E-AUTH-005 error code.
    ///
    /// BC-2.01.017 EC-017-003; ADR-022 §C.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_missing_credential_returns_e_auth_005() {
        let provider = StaticCookieAuthProvider::new_with_resolver(
            "cyberint",
            Arc::new(NotFoundCredentialResolver),
        );
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result = provider.acquire_token(&spec, &client_id).await;

        assert!(
            result.is_err(),
            "BC-2.01.017 EC-017-003: acquire_token must return Err when credential \
             resolver returns Err (not-found path). Got Ok."
        );
        let err_str = result.unwrap_err().to_string();
        assert!(
            err_str.contains("E-AUTH-005"),
            "missing credential MUST yield E-AUTH-005. \
             Got: {err_str}. \
             BC-2.01.017 EC-017-003 (resolver Err path)."
        );
    }

    /// E-AUTH-006: acquire_token returns E-AUTH-006 when the resolved credential is empty.
    ///
    /// Verifies BC-2.01.017 EC-017-005 (empty/whitespace value path):
    /// when resolve_credential returns Ok(SecretString("")) — the case where the credential
    /// name is known but the value is empty (e.g., direct-env branch with empty value per
    /// resolve_secret.rs EC-017-005 semantics) — acquire_token's api_key.is_empty() guard
    /// returns E-AUTH-006, not E-AUTH-005.
    ///
    /// Uses MockDI injection (MockCredentialResolver::new("") via new_with_resolver) to
    /// inject an empty SecretString directly without env var mutation. MockCredentialResolver
    /// accepts empty string because test-helpers deliberately allow injection of invalid values
    /// to exercise validation gates in the production code under test.
    ///
    /// TD-VSDD-059 load-bearing assertion: checks E-AUTH-006 error code.
    /// BC-2.01.017 EC-017-005; ADR-022 §C.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_empty_value_returns_e_auth_006() {
        // Inject an empty resolved credential value via MockCredentialResolver.
        // MockCredentialResolver::new("") returns Ok(SecretString("")) — simulating the case
        // where the credential store resolves the name but the value is empty.
        // This exercises the acquire_token api_key.is_empty() guard (E-AUTH-006 path).
        let provider = StaticCookieAuthProvider::new_with_resolver(
            "cyberint",
            Arc::new(MockCredentialResolver::new("")),
        );
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result = provider.acquire_token(&spec, &client_id).await;

        assert!(
            result.is_err(),
            "BC-2.01.017 EC-017-005: acquire_token must return Err when resolved \
             credential value is empty. Got Ok."
        );
        let err_str = result.unwrap_err().to_string();
        assert!(
            err_str.contains("E-AUTH-006"),
            "empty resolved value MUST yield E-AUTH-006. \
             Got: {err_str}. \
             BC-2.01.017 EC-017-005 (resolver Ok(empty) path)."
        );
    }

    /// E-AUTH-006: acquire_token rejects an API key with illegal RFC 6265 cookie characters.
    ///
    /// Uses MockDI injection (MockCredentialResolver via new_with_resolver) — no env var
    /// mutation required. The injected resolver returns the tainted key value directly,
    /// bypassing the real credential store. ADR-022 §C; BC-2.01.017 EC-017-006.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_rejects_illegal_cookie_chars() {
        // Semicolon is illegal in cookie-value per RFC 6265 §4.1.1.
        // Inject the tainted value directly via MockCredentialResolver — no env var mutation.
        let provider = StaticCookieAuthProvider::new_with_resolver(
            "cyberint",
            Arc::new(MockCredentialResolver::new("valid-prefix;injected-cookie")),
        );
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result = provider.acquire_token(&spec, &client_id).await;

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

    /// E-AUTH-007: acquire_token returns E-AUTH-007 when the credential resolver returns
    /// `CredentialResolutionError::BackendUnavailable`.
    ///
    /// Verifies BC-2.01.017 EC-017-010 (backend unavailable path) /
    /// TV-BC-2.01.017-009: when the resolver returns `BackendUnavailable`, acquire_token
    /// wraps the error as E-AUTH-007 (NOT E-AUTH-005). The distinction is critical:
    /// E-AUTH-005 = credential not configured; E-AUTH-007 = configured but backend down.
    ///
    /// Uses MockDI injection (BackendUnavailableCredentialResolver via new_with_resolver) —
    /// no env var mutation required. ADR-022 §C; BC-2.01.017 EC-017-010.
    ///
    /// TD-VSDD-059 load-bearing assertion: checks E-AUTH-007 code AND absence of E-AUTH-005.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_backend_unavailable_returns_e_auth_007() {
        let provider = StaticCookieAuthProvider::new_with_resolver(
            "cyberint",
            Arc::new(BackendUnavailableCredentialResolver),
        );
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result = provider.acquire_token(&spec, &client_id).await;

        assert!(
            result.is_err(),
            "BC-2.01.017 EC-017-010: acquire_token must return Err when credential \
             resolver returns BackendUnavailable. Got Ok."
        );
        let err_str = result.unwrap_err().to_string();
        assert!(
            err_str.contains("E-AUTH-007"),
            "backend unavailable MUST yield E-AUTH-007. \
             Got: {err_str}. \
             BC-2.01.017 EC-017-010 / TV-BC-2.01.017-009."
        );
        assert!(
            !err_str.contains("E-AUTH-005"),
            "backend unavailable MUST NOT produce E-AUTH-005 (that code is for missing credential). \
             Got: {err_str}. \
             BC-2.01.017 EC-017-010."
        );
    }

    /// E-AUTH-006: acquire_token rejects an oversized whitespace-only API key with
    /// "exceeds 4096-byte limit" detail, NOT "empty or all-whitespace".
    ///
    /// Regression test for F-LP3-LOW-001 (validation order fix — length check before
    /// whitespace check). A 5000-byte whitespace string previously triggered the
    /// whitespace branch first, producing misleading detail. After the reorder,
    /// the length check fires first with accurate detail.
    ///
    /// Uses MockDI injection (MockCredentialResolver with 5000-space value).
    /// ADR-022 §C; BC-2.01.017 §E-AUTH-006.
    #[tokio::test]
    async fn test_static_cookie_auth_provider_rejects_oversized_whitespace_with_length_detail() {
        // 5000 spaces: longer than 4096-byte limit, all-whitespace.
        // Before the fix: whitespace branch fires → "empty or all-whitespace" detail.
        // After the fix: length branch fires first → "exceeds 4096-byte limit" detail.
        let oversized_whitespace = " ".repeat(5000);
        let provider = StaticCookieAuthProvider::new_with_resolver(
            "cyberint",
            Arc::new(MockCredentialResolver::new(oversized_whitespace)),
        );
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result = provider.acquire_token(&spec, &client_id).await;

        assert!(
            result.is_err(),
            "E-AUTH-006: acquire_token must reject a 5000-byte whitespace api_key. Got Ok."
        );
        let err_str = result.unwrap_err().to_string();
        assert!(
            err_str.contains("E-AUTH-006"),
            "E-AUTH-006: error must reference E-AUTH-006. Got: {err_str}"
        );
        assert!(
            err_str.contains("exceeds") || err_str.contains("4096"),
            "F-LP3-LOW-001 regression: oversized whitespace key must fire the length check \
             (detail contains 'exceeds' or '4096'), NOT the whitespace check. \
             Got detail: {err_str}"
        );
        assert!(
            !err_str.contains("empty or all-whitespace"),
            "F-LP3-LOW-001 regression: detail must NOT say 'empty or all-whitespace' for a \
             5000-byte whitespace key — the length check must fire first. Got: {err_str}"
        );
    }

    /// SEC-001 / CWE-93 / CWE-113: acquire_token must reject API keys containing ASCII
    /// control characters (CTL chars: 0x00-0x1F incl. CR 0x0D / LF 0x0A, and DEL 0x7F).
    ///
    /// RFC 6265 §4.1.1 cookie-octet grammar explicitly forbids CTL characters in cookie
    /// values. A credential with an embedded CRLF (copy-paste artifact) would otherwise
    /// slip past E-AUTH-006 and produce a cryptic `InvalidHeaderValue`/`HttpRequestFailed`
    /// at request time instead of a clear, actionable E-AUTH-006 error.
    ///
    /// Tests two distinct CTL patterns:
    /// 1. CRLF injection: `"valid-prefix\r\nInjected: header"` — HTTP header injection vector
    /// 2. Bare CTL byte:  `"abc\x01def"` — non-printable control char
    ///
    /// Both must be rejected with E-AUTH-006, NOT HttpRequestFailed or any other code.
    ///
    /// Uses MockDI injection — no env var mutation required.
    /// ADR-022 §C; RFC 6265 §4.1.1; SEC-001 (S-DTU-CYBERINT-AUTH-FIDELITY-001).
    #[tokio::test]
    async fn test_BC_2_01_017_acquire_token_rejects_crlf_in_api_key() {
        // ---- Pattern 1: CRLF injection (HTTP header injection / CWE-113) ----
        let crlf_key = "valid-prefix\r\nInjected: header";
        let provider_crlf = StaticCookieAuthProvider::new_with_resolver(
            "cyberint",
            Arc::new(MockCredentialResolver::new(crlf_key)),
        );
        let spec = cookie_roundtrip_spec();
        let client_id = OrgSlug::new("test-org-unit");

        let result_crlf = provider_crlf.acquire_token(&spec, &client_id).await;

        assert!(
            result_crlf.is_err(),
            "SEC-001 / CWE-113: acquire_token must reject API keys containing CRLF \
             (RFC 6265 §4.1.1 CTL chars forbidden in cookie-value). Got Ok."
        );
        let err_crlf = result_crlf.unwrap_err().to_string();
        assert!(
            err_crlf.contains("E-AUTH-006"),
            "SEC-001: CRLF rejection must produce E-AUTH-006, NOT HttpRequestFailed or other. \
             Got: {err_crlf}"
        );
        assert!(
            !err_crlf.contains("E-AUTH-007"),
            "SEC-001: CRLF rejection must NOT produce E-AUTH-007. Got: {err_crlf}"
        );

        // ---- Pattern 2: Bare CTL byte (CWE-93) ----
        let ctl_key = "abc\x01def";
        let provider_ctl = StaticCookieAuthProvider::new_with_resolver(
            "cyberint",
            Arc::new(MockCredentialResolver::new(ctl_key)),
        );

        let result_ctl = provider_ctl.acquire_token(&spec, &client_id).await;

        assert!(
            result_ctl.is_err(),
            "SEC-001 / CWE-93: acquire_token must reject API keys containing bare CTL bytes \
             (RFC 6265 §4.1.1). Got Ok."
        );
        let err_ctl = result_ctl.unwrap_err().to_string();
        assert!(
            err_ctl.contains("E-AUTH-006"),
            "SEC-001: CTL byte rejection must produce E-AUTH-006. Got: {err_ctl}"
        );
    }
}
