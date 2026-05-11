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

// ---------------------------------------------------------------------------
// AuthToken newtype
// ---------------------------------------------------------------------------

/// An opaque bearer token string produced by `AuthProvider::acquire_token`.
///
/// The inner `String` is the raw bearer token value — do NOT log it.
/// Credentials MUST NOT appear in log output at any level (INV-INFUSE-005 / AD-017).
///
/// TD-S-PLUGIN-PREREQ-B-002 P3: `AuthToken` does not implement `zeroize::Zeroize`
/// on `Drop`, meaning the token string may linger in heap memory after the token is
/// discarded. PREREQ-D credential-store integration scope: gate `zeroize` behind a
/// `zeroize-memory` Cargo feature (off by default) to avoid pulling `zeroize` into
/// the dependency graph before it is needed.
#[derive(Clone)]
pub struct AuthToken(pub String);

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
pub struct NullAuthProvider;

impl AuthProvider for NullAuthProvider {
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SensorSpec,
        _client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        Box::pin(async move { Ok(AuthToken(String::new())) })
    }
}

// ---------------------------------------------------------------------------
// MockAuthProvider — configurable call-recorder for auth-specific tests
// ---------------------------------------------------------------------------

/// Test helper `AuthProvider` that records every `acquire_token` call and
/// returns a fixed bearer token string.
///
/// Use in tests that exercise 401-retry behavior (AC-5, VP-PLUGIN-005).
pub struct MockAuthProvider {
    /// Token returned on every call.
    pub token: String,
    /// Number of times `acquire_token` was called (interior-mutable for `&self` API).
    pub call_count: std::sync::atomic::AtomicU32,
}

impl MockAuthProvider {
    /// Create a new `MockAuthProvider` returning `token` on every call.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            call_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Return the number of times `acquire_token` was invoked.
    pub fn calls(&self) -> u32 {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl AuthProvider for MockAuthProvider {
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SensorSpec,
        _client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let token = self.token.clone();
        Box::pin(async move { Ok(AuthToken(token)) })
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
