---
document_type: red-gate-log
story_id: S-DTU-CYBERINT-AUTH-FIDELITY-001
story_title: "Cyberint DTU Auth Fidelity — access_token Static Cookie Model (ADR-031)"
step: 3 (test-writer Red Gate)
date: 2026-05-30
commit: "75bd39bd — test(S-DTU-CYBERINT-AUTH-FIDELITY-001): add failing Red Gate tests for BC-2.01.017 + BC-2.16.013"
feature_head_at_red_gate: "75bd39bd"
stubs_commit: "6fee66a3 — feat(S-DTU-CYBERINT-AUTH-FIDELITY-001): add module stubs"
---

# Red Gate Log — S-DTU-CYBERINT-AUTH-FIDELITY-001

## Red Gate Outcome

| Metric | Value |
|--------|-------|
| Total Red Gate tests written | 16 |
| Tests failing (AC-anchored todo!() panics) | 12 |
| Tests passing (structural — AC-004/POST-login removal shape) | 4 |
| Red Gate commit | `75bd39bd` |
| Verdict | RED GATE VERIFIED (per S-7.02 Step 3) |

## Test Breakdown

### 12 Failing Tests (todo!() panics — AC coverage)

Tests written to verify behavioral contracts BC-2.01.017 (Static Cookie AuthProvider) and BC-2.16.013. These fail with `todo!()` panics inserted as stubs before TDD implementation begins.

| # | Test Name | AC Anchor | Expected Failure Mode |
|---|-----------|-----------|----------------------|
| 1 | `test_BC_2_01_017_001_static_cookie_injection` | AC-001 | todo!() panic |
| 2 | `test_BC_2_01_017_002_no_login_roundtrip` | AC-002 | todo!() panic |
| 3 | `test_BC_2_01_017_003_access_token_cookie_name` | AC-003 | todo!() panic |
| 4 | `test_BC_2_01_017_004_credential_resolved_from_store` | AC-004 | todo!() panic |
| 5 | `test_BC_2_01_017_005_e_auth_005_empty_resolved` | AC-005 | todo!() panic |
| 6 | `test_BC_2_01_017_006_e_auth_006_resolver_error` | AC-006 | todo!() panic |
| 7 | `test_BC_2_01_017_007_dtu_access_token_allowlist` | AC-007 | todo!() panic |
| 8 | `test_BC_2_01_017_008_dtu_no_post_login_route` | AC-008 | todo!() panic |
| 9 | `test_BC_2_01_017_009_dtu_rejects_cyberint_session_cookie` | AC-009 | todo!() panic |
| 10 | `test_BC_2_01_017_010_injectable_credential_resolver` | AC-010 | todo!() panic |
| 11 | `test_BC_2_16_013_001_auth_attempt_event_emitted` | BC-2.16.013 AC-001 | todo!() panic |
| 12 | `test_BC_2_16_013_002_auth_event_fields_complete` | BC-2.16.013 AC-002 | todo!() panic |

### 4 Passing Tests (structural — POST /login removal)

These 4 tests pass structurally before implementation because they verify the ABSENCE of the POST /login route, which was already removed from module stubs per the story scope.

| # | Test Name | AC Anchor | Pass Reason |
|---|-----------|-----------|------------|
| 13 | `test_BC_2_01_017_no_login_endpoint_in_dtu` | AC-008 structural | Stub has no POST /login route stub |
| 14 | `test_BC_2_01_017_no_cyberint_session_in_dtu_routes` | AC-009 structural | Stub has no cyberint_session references |
| 15 | `test_BC_2_01_017_no_post_login_in_dtu_network_router` | AC-008b structural | Network router stub also clean |
| 16 | `test_no_legacy_login_in_public_api` | AC-008 perimeter | Compile-fail perimeter check |

## Notes

- Red Gate discipline per BC-5.39.001: ALL failing tests use `todo!()` panics with AC-anchored message text (SID-1 compliance — no #[ignore]'d tests used as substitute)
- The 4 structurally-passing tests do NOT advance the TDD Red Gate; they confirm pre-condition removal was done correctly in stubs
- Red Gate verified at 75bd39bd before Step 4 TDD implementation dispatch

## Step 4 Implementation Outcome

After Red Gate verification (Step 3), the implementer TDD pass produced:

| Commit | Subject |
|--------|---------|
| `47ab523c` | `wip(...): DTU test rewrites — access_token cookie model (109 tests pass)` |
| `dba6eb95` | `fix(...): close Step 4 exit conditions — threats.rs 401, fmt, injectable resolver` |

**Step 4 exit conditions verified (at dba6eb95):**
- 3835/3836 tests pass (1 unrelated pre-existing ignore)
- Clippy Cyberint-clean (zero new warnings)
- Format clean
- ZERO `todo!()` in production code paths
- `CredentialResolver` injectable via `Arc<dyn CredentialResolver>` (ADR-022 §C Arc-DI pattern)
