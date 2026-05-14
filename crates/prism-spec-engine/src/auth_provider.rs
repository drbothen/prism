// SPDX-License-Identifier: Apache-2.0
//! AuthProvider trait — spec-driven auth surface for SensorSpec-declared adapters.
//!
//! Anchors:
//! - BC-2.01.013 (DataSource Trait: Spec-Driven Adapter Pattern)
//! - ADR-023 §C2 (Plugin-Only Sensor Architecture — Real PipelineExecutor)
//! - Story: S-PLUGIN-PREREQ-B
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

use crate::error::SpecEngineError;
use crate::spec_parser::SensorSpec;
use prism_core::OrgSlug;
use std::future::Future;
use std::pin::Pin;
use zeroize::Zeroizing;

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
}
