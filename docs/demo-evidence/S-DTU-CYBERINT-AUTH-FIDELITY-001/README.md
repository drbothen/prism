# Demo Evidence: S-DTU-CYBERINT-AUTH-FIDELITY-001

Story: Cyberint DTU Auth Fidelity — Remove POST /login DTU Route; Implement
StaticCookieAuthProvider; Inject Cookie: access_token={api_key}; No Session UUID

LOCAL adversary cascade converged at Pass 17 (D-881). PR #164 / story v1.8 (branch feature/S-DTU-CYBERINT-AUTH-FIDELITY-001).

---

## Evidence Index

| File | AC | Description |
|------|----|-------------|
| AC-001-no-login-route.txt | AC-001 | POST /login returns 404; route absent from build_router |
| AC-002-extract-access-token.txt | AC-002 | extract_access_token parses access_token; None for cyberint_session |
| AC-003-check-auth-access-token-cookie.txt | AC-003 | check_auth: access_token → 200; cyberint_session → 401; no cookie → 401 |
| AC-004-state-access-token-allowlist.txt | AC-004 | CyberintState allowlist model; no session UUID store |
| AC-005-static-cookie-auth-provider.txt | AC-005 | StaticCookieAuthProvider::acquire_token returns api_key; no HTTP call |
| AC-006-acquire-token-no-http-call.txt | AC-006 | Zero HTTP calls during acquire_token (INV-COOKIE-001) |
| AC-007-build-request-cookie-injection.txt | AC-007 | build_request injects Cookie: access_token={token} for CookieRoundtrip |
| AC-008-end-to-end-parity.txt | AC-008 | access_token cookie → DTU alerts returns data (parity infrastructure) |
| AC-009-negative-parity-cyberint-session.txt | AC-009 | cyberint_session cookie → 401 from corrected DTU |
| AC-010-error-taxonomy-compliance.txt | AC-010 | E-AUTH-005/E-AUTH-006/E-AUTH-007 on auth failure paths |
| AC-011-no-uncatalogued-event-type.txt | AC-011 | SAP-1: one new emission (cookie_auth_401) introduced and catalogued in BC-2.16.002 v1.60; no uncatalogued event_type |
| evidence-report.md | All | Full coverage table + Red Gate test summary + POL-10 compliance |

---

## Test Totals at PR #164 / story v1.8

- prism-dtu-cyberint (--features dtu): 109/109 passed
- prism-spec-engine: 492/492 passed (10 skipped, pre-existing)
- All 7 Red Gate tests: PASS

---

## POL-10 Note

All files are in this story-scoped subdirectory per POL-10.
No flat `docs/demo-evidence/*.md` files exist.
