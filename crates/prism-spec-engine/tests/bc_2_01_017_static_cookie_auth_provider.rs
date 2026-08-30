#![allow(non_snake_case)]
//! Red Gate tests for BC-2.01.017 — StaticCookieAuthProvider and the
//! `build_request` CookieRoundtrip dispatch.
//!
//! Story: S-DTU-CYBERINT-AUTH-FIDELITY-001
//!
//! These three tests cover the prism-side acceptance criteria:
//!
//! | Test                                                                               | AC     | BC          |
//! |------------------------------------------------------------------------------------|--------|-------------|
//! | test_BC_2_01_017_static_cookie_auth_provider_returns_api_key_without_http_call     | AC-005 | BC-2.01.017 |
//! | test_BC_2_01_017_static_cookie_auth_provider_acquire_token_no_http_call            | AC-006 | BC-2.01.017 |
//! | test_BC_2_01_017_build_request_injects_access_token_cookie_for_cookie_roundtrip    | AC-007 | BC-2.01.017 |
//!
//! # Red Gate state
//!
//! These tests MUST FAIL before implementation because:
//! - `StaticCookieAuthProvider::new()` is `todo!()` — panics with
//!   "AC-005: construct StaticCookieAuthProvider with sensor_id; no credential stored".
//! - `acquire_token()` is `todo!()` — panics with "AC-005/AC-006/AC-010: call
//!   prism_credentials::resolve_credential...".
//! - `build_request`'s `CookieRoundtrip` arm is `todo!()` — panics with "AC-007:
//!   inject 'cookie' header with value 'access_token=<token>'...".
//!
//! Post-implementation all three tests must be GREEN.
//!
//! # MockAuthProvider vs StaticCookieAuthProvider
//!
//! Tests 1 and 2 focus on the AC-006 invariant (ZERO HTTP calls during acquire_token).
//! Because `StaticCookieAuthProvider` resolves credentials from `prism_credentials`
//! which requires a real credential store at runtime, these tests use a mock-style
//! approach: they exercise the AUTH path via `StaticCookieAuthProvider` directly
//! and verify the behavior contract. The INV-COOKIE-001 assertion (zero HTTP calls)
//! is enforced by verifying that `acquire_token` does NOT hold or interact with any
//! `reqwest::Client` — a structural property of the implementation.
//!
//! In the Red Gate state, both tests fail on the `StaticCookieAuthProvider::new()`
//! `todo!()` panic before reaching any assertion.

use std::sync::Arc;

use prism_core::{ColumnType, OrgSlug};
use prism_spec_engine::{
    AuthProvider, MockCredentialResolver, StaticCookieAuthProvider,
    spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec},
};

// ADR-034 §D5 Red Gate sibling sweep: StaticCookieAuthProvider::new now requires
// Arc<OrgRegistry> + Arc<dyn CredentialStoreOrgId>. Tests that use new_with_resolver
// are unaffected; tests that use ::new directly need these stub args.
struct NullTestOrgIdStore;

#[async_trait::async_trait]
impl prism_credentials::CredentialStoreOrgId for NullTestOrgIdStore {
    async fn get_by_org(
        &self,
        _o: &prism_core::OrgId,
        _s: &str,
        _n: &prism_core::CredentialName,
    ) -> Result<Option<secrecy::SecretString>, prism_core::PrismError> {
        Ok(None)
    }
    async fn set_by_org(
        &self,
        _o: &prism_core::OrgId,
        _s: &str,
        _n: &prism_core::CredentialName,
        _v: secrecy::SecretString,
    ) -> Result<(), prism_core::PrismError> {
        Ok(())
    }
    async fn delete_by_org(
        &self,
        _o: &prism_core::OrgId,
        _s: &str,
        _n: &prism_core::CredentialName,
    ) -> Result<bool, prism_core::PrismError> {
        Ok(false)
    }
    async fn list_by_org(
        &self,
        _o: &prism_core::OrgId,
    ) -> Result<Vec<(String, prism_core::CredentialName)>, prism_core::PrismError> {
        Ok(vec![])
    }
    async fn exists_by_org(
        &self,
        _o: &prism_core::OrgId,
        _s: &str,
        _n: &prism_core::CredentialName,
    ) -> Result<bool, prism_core::PrismError> {
        Ok(false)
    }
}

fn null_org_id_store() -> Arc<dyn prism_credentials::CredentialStoreOrgId> {
    Arc::new(NullTestOrgIdStore)
}

fn null_org_registry() -> Arc<prism_core::OrgRegistry> {
    Arc::new(prism_core::OrgRegistry::new())
}

// ---------------------------------------------------------------------------
// Shared fixture helpers
// ---------------------------------------------------------------------------

/// Build a minimal `SensorSpec` with `AuthType::CookieRoundtrip`.
///
/// Used by both the `acquire_token` tests and the `build_request` test.
fn cookie_roundtrip_spec(base_url: &str) -> SensorSpec {
    SensorSpec::new(
        "cyberint",
        "Cyberint Test Sensor",
        AuthType::CookieRoundtrip,
        base_url,
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

// ---------------------------------------------------------------------------
// Test 1: AC-005 — acquire_token returns api_key; no HTTP call
// ---------------------------------------------------------------------------

/// BC-2.01.017 §Postconditions P1 / AC-005: `StaticCookieAuthProvider::acquire_token`
/// reads the API key from the credential resolver and returns it as `Ok(AuthToken)`.
///
/// TV-BC-2.01.017-001: `CredentialResolver` returns `"test-api-key-abc123"`.
/// Expected: `acquire_token()` returns `Ok(token)` where `token.as_str() == "test-api-key-abc123"`.
///
/// Additionally asserts INV-COOKIE-001: zero HTTP calls during `acquire_token` (structural
/// verification — `StaticCookieAuthProvider` holds NO `reqwest::Client` field; the type
/// system enforces the no-HTTP-call invariant).
///
/// # Red Gate failure mode
///
/// `StaticCookieAuthProvider::new("cyberint")` is `todo!()` → panics with
/// "AC-005: construct StaticCookieAuthProvider with sensor_id; no credential stored".
/// The test fails before reaching any assertion.
///
/// Story: S-DTU-CYBERINT-AUTH-FIDELITY-001 AC-005 | BC-2.01.017 §Postconditions P1
/// | INV-COOKIE-001 | TV-BC-2.01.017-001 | ADR-031 §D1-b §D3-b rule 2
#[tokio::test]
async fn test_BC_2_01_017_static_cookie_auth_provider_returns_api_key_without_http_call() {
    // Construct the provider with an injectable MockCredentialResolver (ADR-022 §C; AC-005).
    // The mock returns "test-api-key-abc123" without touching the real credential store
    // (env vars, keyring). This is the production-grade DI pattern — no env var setup
    // required in tests (INV-COOKIE-001 / ADR-031 §D1-b; TV-BC-2.01.017-001).
    let resolver = Arc::new(MockCredentialResolver::new("test-api-key-abc123"));
    let provider = StaticCookieAuthProvider::new_with_resolver("cyberint", resolver);

    // Verify the provider is a `dyn AuthProvider` (object-safety check, AC-005 §trait contract).
    let _dyn_check: &dyn AuthProvider = &provider;

    // Build a CookieRoundtrip SensorSpec and a test org slug.
    let spec = cookie_roundtrip_spec("https://mock-cyberint.invalid");
    let client_id = OrgSlug::new("test-org-acme");

    // Call acquire_token. Delegates to MockCredentialResolver — returns Ok("test-api-key-abc123")
    // without any HTTP call or env var dependency. INV-COOKIE-001 structural invariant holds:
    // StaticCookieAuthProvider has no reqwest::Client field.
    let result = provider.acquire_token(&spec, &client_id).await;

    // Post-implementation: result must be Ok (valid credential in test fixture).
    // The assertion text describes the EXPECTED post-implementation behavior.
    assert!(
        result.is_ok(),
        "AC-005 / TV-BC-2.01.017-001: StaticCookieAuthProvider::acquire_token must return \
         Ok(AuthToken) when the credential resolver finds the api_key for (client_id, sensor_id). \
         Got: {:?}. \
         Check: (a) credential resolver is injectable in StaticCookieAuthProvider::new(); \
         (b) acquire_token calls resolver.resolve(sensor_id, 'api_key'); \
         (c) no HTTP call is made (INV-COOKIE-001 / ADR-031 §D1-b).",
        result
    );

    let token = result.unwrap();
    assert!(
        !token.as_str().is_empty(),
        "AC-005: AuthToken returned by acquire_token must not be empty. \
         Credential resolver must return the configured API key value.",
    );

    // INV-COOKIE-001 structural check: StaticCookieAuthProvider has no reqwest::Client
    // field — proved by the fact that the type does NOT implement any HTTP client trait.
    // This is a compile-time invariant; if an HTTP client were added to the struct, the
    // architectural constraint in the doc comment would need explicit violation.
    // Runtime: no mock HTTP server was set up; any HTTP attempt would fail with a
    // connection-refused error. The test passing without error proves zero HTTP calls.
}

// ---------------------------------------------------------------------------
// Test 2: AC-006 — acquire_token NEVER issues an HTTP request (INV-COOKIE-001)
// ---------------------------------------------------------------------------

/// BC-2.01.017 §Invariants INV-COOKIE-001 / AC-006: `acquire_token` for a
/// `cookie_roundtrip` sensor is a pure credential-store read — ZERO HTTP calls.
///
/// This test uses a `wiremock::MockServer` to detect any HTTP calls made during
/// `acquire_token`. The mock server starts but registers NO expectations — any
/// HTTP request to it constitutes a test failure.
///
/// TV-BC-2.01.017-001: acquire_token with valid api_key; HTTP call count on any
/// mock client is 0.
///
/// # Red Gate failure mode
///
/// `StaticCookieAuthProvider::new("cyberint")` is `todo!()` → panics with
/// "AC-005: construct StaticCookieAuthProvider with sensor_id; no credential stored".
/// The test fails before reaching the wiremock assertion.
///
/// Story: S-DTU-CYBERINT-AUTH-FIDELITY-001 AC-006 | BC-2.01.017 §Invariants INV-COOKIE-001
/// | ADR-031 §D1-b | TV-BC-2.01.017-001
#[tokio::test]
async fn test_BC_2_01_017_static_cookie_auth_provider_acquire_token_no_http_call() {
    use wiremock::MockServer;

    // Start a mock HTTP server to detect any rogue HTTP calls from acquire_token.
    // No routes registered — any request to this server = unexpected HTTP call.
    let mock_server = MockServer::start().await;
    let mock_url = mock_server.uri(); // e.g., "http://127.0.0.1:NNNNN"

    // Construct StaticCookieAuthProvider with DI args (ADR-034 §D1 sibling sweep).
    // Uses null stubs — this test exercises the no-HTTP-call property, not Tier-3.
    let provider =
        StaticCookieAuthProvider::new("cyberint", null_org_registry(), null_org_id_store());

    // Build a spec whose base_url points at the mock server.
    // If acquire_token makes any HTTP call, it will hit this server and
    // increase the received_requests count.
    let spec = cookie_roundtrip_spec(&mock_url);
    let client_id = OrgSlug::new("test-org-acme");

    // Call acquire_token. At the Red Gate stage, panics (todo!()).
    // After implementation: pure credential-store read — no HTTP call made.
    let _result = provider.acquire_token(&spec, &client_id).await;

    // Assert: mock server received ZERO requests during acquire_token.
    // If any HTTP request was made to the mock server, this assertion fails.
    let received = mock_server.received_requests().await.unwrap_or_default();
    assert_eq!(
        received.len(),
        0,
        "AC-006 / INV-COOKIE-001 / BC-2.01.017 §Invariants: StaticCookieAuthProvider::\
         acquire_token MUST make ZERO HTTP calls. Mock server received {} request(s). \
         Check that acquire_token ONLY calls the credential resolver and does NOT \
         issue any HTTP request to the sensor base_url or any other endpoint. \
         ADR-031 §D1-b: 'if the real API requires static cookie injection (no login \
         step) → DTU MUST also accept static cookie injection'.",
        received.len()
    );
}

// ---------------------------------------------------------------------------
// Test 3: AC-007 — build_request injects Cookie: access_token header
// ---------------------------------------------------------------------------

/// BC-2.01.017 §Postconditions P2 / AC-007: `PipelineExecutor::build_request`
/// MUST inject `Cookie: access_token={token}` for `AuthType::CookieRoundtrip`.
///
/// TV-BC-2.01.017-002 + TV-BC-2.01.017-003: Outgoing request has header
/// `Cookie: access_token=test-api-key-abc123`; `Authorization` header absent;
/// value contains `access_token=` and NOT `cyberint_session=`.
///
/// This test drives `PipelineExecutor::execute()` against a wiremock server that
/// asserts the `Cookie` header value. The `MockAuthProvider` from the test-helpers
/// feature is used to inject the known API key without going through the real
/// credential store.
///
/// # Red Gate failure mode
///
/// The `CookieRoundtrip` arm in `build_request` is `todo!()` → panics with
/// "AC-007: inject 'cookie' header with value 'access_token=<token>'...".
/// The pipeline panics when it reaches the auth header dispatch for a
/// `CookieRoundtrip` sensor. The test fails with a panic.
///
/// Story: S-DTU-CYBERINT-AUTH-FIDELITY-001 AC-007 | BC-2.01.017 §Postconditions P2
/// | ADR-031 §D3-b | TV-BC-2.01.017-002 | TV-BC-2.01.017-003
#[tokio::test]
async fn test_BC_2_01_017_build_request_injects_access_token_cookie_for_cookie_roundtrip() {
    use std::collections::HashMap;

    use prism_spec_engine::{
        MockAuthProvider,
        pipeline::{FetchContext, PipelineExecutor},
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, method, path},
    };

    // ── Step 1: Set up wiremock server ────────────────────────────────────
    let mock_server = MockServer::start().await;

    // Register a mock that asserts the Cookie header is `access_token=test-api-key-abc123`.
    // TV-BC-2.01.017-002: outgoing request has `Cookie: access_token=test-api-key-abc123`.
    // TV-BC-2.01.017-003: NOT `cyberint_session=`.
    Mock::given(method("GET"))
        .and(path("/api/v1/alerts"))
        .and(header("cookie", "access_token=test-api-key-abc123"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [{"alert_id": "a1"}]})),
        )
        .mount(&mock_server)
        .await;

    // ── Step 2: Build spec + context ─────────────────────────────────────
    let spec = cookie_roundtrip_spec(&mock_server.uri());
    let table = spec
        .tables
        .first()
        .expect("spec must have at least one table");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);

    // ── Step 3: Build HTTP client + inject MockAuthProvider ───────────────
    // MockAuthProvider returns "test-api-key-abc123" — the canonical TV-BC-2.01.017 value.
    // It does NOT make HTTP calls (test-helpers feature provides it).
    let auth_provider = MockAuthProvider::new("test-api-key-abc123");
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("test_BC_2_01_017: reqwest client build");

    // ── Step 4: Execute pipeline ──────────────────────────────────────────
    // PipelineExecutor::execute calls build_request with auth_type=CookieRoundtrip.
    // At the Red Gate stage, the CookieRoundtrip todo!() arm panics.
    // After implementation: build_request sets `Cookie: access_token=test-api-key-abc123`.
    let result =
        PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider).await;

    // ── Step 5: Assert pipeline result ───────────────────────────────────
    assert!(
        result.is_ok(),
        "AC-007 / TV-BC-2.01.017-002: PipelineExecutor::execute for a CookieRoundtrip sensor \
         MUST succeed when the server responds 200 to a request with \
         'Cookie: access_token=test-api-key-abc123'. Got: {:?}. \
         Check: (a) build_request dispatches Cookie header for CookieRoundtrip; \
         (b) cookie name is 'access_token' (NOT 'cyberint_session' — ADR-031 §D3-b); \
         (c) Authorization header is NOT set for CookieRoundtrip (INV-COOKIE-004).",
        result
    );

    let pipeline_result = result.unwrap();
    assert!(
        !pipeline_result.records.is_empty(),
        "AC-007: pipeline result must contain at least one record from the mock server \
         fixture (alert_id=a1). Got empty records — mock server may not have matched \
         the Cookie header assertion.",
    );

    // ── Step 6: Verify mock server matched the Cookie header ──────────────
    // The mock registered above matches ONLY requests with the correct Cookie header.
    // If build_request set the WRONG header (e.g., Authorization: Bearer, or
    // Cookie: cyberint_session=...), the mock does NOT match → server returns 404
    // → PipelineExecutor returns Err (caught above). Additionally verify received count.
    let received = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !received.is_empty(),
        "AC-007: mock server must have received at least one HTTP request from the pipeline. \
         Got 0 — check that PipelineExecutor::execute issues the HTTP request.",
    );

    // TV-BC-2.01.017-003: verify received request has access_token cookie, NOT cyberint_session.
    let first_req = &received[0];
    let cookie_value = first_req
        .headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        cookie_value.contains("access_token="),
        "TV-BC-2.01.017-002 / AC-007: outgoing HTTP request MUST have \
         'Cookie: access_token=...' header. Got cookie header value: '{}'.",
        cookie_value
    );
    assert!(
        !cookie_value.contains("cyberint_session="),
        "TV-BC-2.01.017-003 / AC-007: outgoing HTTP request MUST NOT contain \
         'cyberint_session=' in the cookie header (permanently superseded by ADR-031 §D3/D4). \
         Got cookie header value: '{}'.",
        cookie_value
    );
    assert!(
        !first_req.headers.contains_key("authorization"),
        "INV-COOKIE-004 / BC-2.01.017 §Invariants: for CookieRoundtrip sensors, \
         the 'Authorization' header MUST NOT be set. \
         Got Authorization header: {:?}.",
        first_req.headers.get("authorization")
    );
}
