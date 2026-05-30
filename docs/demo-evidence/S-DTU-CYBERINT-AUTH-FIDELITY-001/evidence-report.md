# Evidence Report: S-DTU-CYBERINT-AUTH-FIDELITY-001

**Story:** S-DTU-CYBERINT-AUTH-FIDELITY-001 — Cyberint DTU Auth Fidelity
**Version:** 1.5
**LOCAL Adversary Cascade:** Converged at Pass 17 (D-881)
**Feature HEAD:** 4f5b5404
**Date:** 2026-05-30

---

## Coverage Summary

| AC | Description | Evidence File | Evidence Type | Status |
|----|-------------|---------------|---------------|--------|
| AC-001 | POST /login route removed from DTU build_router — returns 404 | AC-001-no-login-route.txt | Red Gate integration test (HTTP-layer) | PASS |
| AC-002 | extract_access_token parses access_token cookie; returns None for cyberint_session | AC-002-extract-access-token.txt | Red Gate unit test (5 cookie header cases) | PASS |
| AC-003 | check_auth validates access_token cookie (not cyberint_session); 200/401 routing | AC-003-check-auth-access-token-cookie.txt | Red Gate integration test (3 HTTP sub-cases) | PASS |
| AC-004 | CyberintState static-token allowlist replaces session UUID store | AC-004-state-access-token-allowlist.txt | Red Gate unit test (4 struct-level sub-tests) | PASS |
| AC-005 | StaticCookieAuthProvider::acquire_token returns api_key without HTTP call | AC-005-static-cookie-auth-provider.txt | Red Gate integration test (MockCredentialResolver DI) | PASS |
| AC-006 | acquire_token never issues HTTP request (INV-COOKIE-001) | AC-006-acquire-token-no-http-call.txt | Red Gate integration test (structural + NotFoundCredentialResolver) | PASS |
| AC-007 | build_request injects Cookie: access_token={token} for CookieRoundtrip | AC-007-build-request-cookie-injection.txt | Red Gate integration test (wiremock CookieMatcher) | PASS |
| AC-008 | End-to-end parity: access_token cookie → DTU alerts returns data | AC-008-end-to-end-parity.txt | Parity infrastructure tests + multi_tenant HTTP tests | PASS (full DTU parity gated on S-6.09) |
| AC-009 | Negative parity: cyberint_session cookie returns 401 | AC-009-negative-parity-cyberint-session.txt | AC-003 negative sub-case + multi_tenant HTTP test | PASS |
| AC-010 | E-AUTH-005/E-AUTH-006/E-AUTH-007 surfaced on auth failure | AC-010-error-taxonomy-compliance.txt | Unit tests in auth_provider.rs::tests (5 error path tests) | PASS |
| AC-011 | No event_type emission without BC-2.16.002 catalog row (SAP-1) | AC-011-no-uncatalogued-event-type.txt | Code analysis (grep; zero results) | PASS |

---

## Red Gate Tests (7 required, 7 pass)

| Test Name | AC | Crate | Result |
|-----------|----|-------|--------|
| test_BC_2_16_013_dtu_post_login_route_removed_returns_404 | AC-001 | prism-dtu-cyberint | PASS |
| test_BC_2_01_017_dtu_extract_access_token_parses_cookie_header | AC-002 | prism-dtu-cyberint | PASS |
| test_BC_2_16_013_dtu_check_auth_requires_access_token_cookie_not_session | AC-003 | prism-dtu-cyberint | PASS |
| test_BC_2_16_013_dtu_state_access_token_allowlist_not_session_uuid | AC-004 | prism-dtu-cyberint | PASS |
| test_BC_2_01_017_static_cookie_auth_provider_returns_api_key_without_http_call | AC-005 | prism-spec-engine | PASS |
| test_BC_2_01_017_static_cookie_auth_provider_acquire_token_no_http_call | AC-006 | prism-spec-engine | PASS |
| test_BC_2_01_017_build_request_injects_access_token_cookie_for_cookie_roundtrip | AC-007 | prism-spec-engine | PASS |

---

## Suite Totals

| Crate | Tests Run | Passed | Failed | Skipped |
|-------|-----------|--------|--------|---------|
| prism-dtu-cyberint (--features dtu) | 109 | 109 | 0 | 0 |
| prism-spec-engine | 492 | 492 | 0 | 10 |

---

## POL-10 Compliance

All evidence files are in `docs/demo-evidence/S-DTU-CYBERINT-AUTH-FIDELITY-001/`
(story-scoped subdirectory). No flat `docs/demo-evidence/*.md` files were created.

---

## Architecture Compliance Verified

| Rule | Verification |
|------|-------------|
| DTU cookie name is `access_token` (not `cyberint_session`) | AC-002 / AC-003 Red Gate tests assert exact cookie name |
| POST /login route absent from DTU | AC-001 Red Gate test: 404 asserted |
| acquire_token makes ZERO HTTP calls | AC-006: struct has no reqwest::Client field; NotFoundCredentialResolver test |
| StaticCookieAuthProvider is NOT feature-gated | pub struct, no #[cfg] annotation |
| AD-017: credentials not held at construction | struct fields: sensor_id (String) + resolver (Arc<dyn CredentialResolver>); no api_key field |
| No new event_type emissions uncatalogued | AC-011: zero grep results in story's changed files |
