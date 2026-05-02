# Demo Evidence Report — W3-FIX-SEC-002

**Story:** W3-FIX-SEC-002: DTU clones — gate POST /dtu/reset with X-Admin-Token on Claroty/CrowdStrike/Armis/Slack
**Branch:** `feature/W3-FIX-SEC-002`
**HEAD SHA:** `c119029533c1d0f31f2df4c547c637557f31587b`
**Recorded:** 2026-05-01
**Product type:** CLI (Rust) — VHS terminal recordings

---

## Security Context

| CWE | OWASP | Description |
|-----|-------|-------------|
| CWE-306 | A07 | Missing Authentication for Critical Function — `POST /dtu/reset` accepted requests with no credential |

**Finding:** SEC-NEW-001 (HIGH). `POST /dtu/reset` on four DTU clones was fully unauthenticated. Any client reaching a clone's loopback port could erase all org-keyed state without supplying any credential, bypassing the per-clone composite-key isolation guaranteed by BC-3.2.001.

**Fix:** Applied the identical `X-Admin-Token` gate already present on `POST /dtu/configure` to the reset handler on each of the four affected clones (`prism-dtu-claroty`, `prism-dtu-crowdstrike`, `prism-dtu-armis`, `prism-dtu-slack`). The admin token is a UUID v4 generated at clone startup; it is per-instance and is not shared across clones. Missing or wrong token yields HTTP 401 `{"error": "missing or invalid admin token"}`. The check-then-act order is preserved: state is only cleared after the token check passes.

**Precedent:** Wave 2 fix WGS-W2-003 applied the same pattern to `prism-dtu-pagerduty` and `prism-dtu-jira`. This story closes the remaining four.

---

## Acceptance Criteria Coverage

| AC | Description | Behavioral Contract | Clone Coverage | Test Count | Result | Recording |
|----|-------------|-------------------|----------------|------------|--------|-----------|
| AC-001 | Reset without admin token returns 401 | BC-3.2.001 invariant 1 | 4/4 (Claroty, CrowdStrike, Armis, Slack) | 4 tests | PASS | [AC-001.gif](AC-001-reset-without-token-returns-401.gif) |
| AC-002 | Reset with correct admin token returns 200 | BC-3.5.001 postcondition 3 | 4/4 | 4 tests | PASS | [AC-002.gif](AC-002-reset-with-correct-token-returns-200.gif) |
| AC-003 | Cross-clone token returns 401 | BC-3.5.002 precondition 3 | 4/4 | 4 tests | PASS | [AC-003.gif](AC-003-cross-clone-token-returns-401.gif) |
| AC-004 | All four clones covered | BC-3.2.001 invariant 1 | 4/4 | grep confirmed | PASS | see AC-001/AC-002/AC-003 |
| AC-005 | No regression on configure endpoint | BC-3.5.001 postcondition 3 | 4/4 | full suites pass | PASS | see full suite run below |

**Summary: 5/5 ACs demonstrated. 12/12 dtu_reset_auth tests pass (3 per clone × 4 clones).**

---

## Per-Clone Test Results

### prism-dtu-claroty

```
running 3 tests
test test_AC_002_dtu_reset_with_admin_token_returns_200 ... ok
test test_AC_001_dtu_reset_without_admin_token_returns_401 ... ok
test test_AC_003_cross_clone_admin_token_returns_401 ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

### prism-dtu-crowdstrike

```
running 3 tests
test test_AC_001_dtu_reset_without_admin_token_returns_401 ... ok
test test_AC_002_dtu_reset_with_admin_token_returns_200 ... ok
test test_AC_003_cross_clone_admin_token_returns_401 ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

### prism-dtu-armis

```
running 3 tests
test test_AC_001_dtu_reset_without_admin_token_returns_401 ... ok
test test_AC_002_dtu_reset_with_admin_token_returns_200 ... ok
test test_AC_003_cross_clone_admin_token_returns_401 ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

### prism-dtu-slack

```
running 3 tests
test test_AC_001_dtu_reset_without_admin_token_returns_401 ... ok
test test_AC_002_dtu_reset_with_admin_token_returns_200 ... ok
test test_AC_003_cross_clone_admin_token_returns_401 ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

---

## Backwards-Compatible Test File Updates

7 existing test files updated to supply `X-Admin-Token` where they call `POST /dtu/reset` (legacy callers now pass the admin token header):

| File | Change |
|------|--------|
| `crates/prism-dtu-claroty/tests/ac_8_reset.rs` | Added `X-Admin-Token` header to reset call |
| `crates/prism-dtu-claroty/tests/fidelity_validator.rs` | Added `X-Admin-Token` header to reset call |
| `crates/prism-dtu-armis/tests/ac_5_missing_bearer_403.rs` | Added `X-Admin-Token` header to reset call |
| `crates/prism-dtu-armis/tests/fidelity_validator.rs` | Added `X-Admin-Token` header to reset call |
| `crates/prism-dtu-armis/tests/reset_state_invariants.rs` | Added `X-Admin-Token` header to reset call |
| `crates/prism-dtu-crowdstrike/tests/fidelity_validator.rs` | Added `X-Admin-Token` header to reset call |
| `crates/prism-dtu-slack/tests/ac_tests.rs` | Added `X-Admin-Token` header to reset call |

---

## Demo Recordings Index

### AC-001 — Reset without admin token returns 401 (error path)

- **Tape:** [AC-001-reset-without-token-returns-401.tape](AC-001-reset-without-token-returns-401.tape)
- **GIF:** [AC-001-reset-without-token-returns-401.gif](AC-001-reset-without-token-returns-401.gif)
- **WebM:** [AC-001-reset-without-token-returns-401.webm](AC-001-reset-without-token-returns-401.webm)
- **Path demonstrated:** `test_AC_001_dtu_reset_without_admin_token_returns_401` across all 4 clones
- **BC trace:** BC-3.2.001 invariant 1

### AC-002 — Reset with correct admin token returns 200 (success path)

- **Tape:** [AC-002-reset-with-correct-token-returns-200.tape](AC-002-reset-with-correct-token-returns-200.tape)
- **GIF:** [AC-002-reset-with-correct-token-returns-200.gif](AC-002-reset-with-correct-token-returns-200.gif)
- **WebM:** [AC-002-reset-with-correct-token-returns-200.webm](AC-002-reset-with-correct-token-returns-200.webm)
- **Path demonstrated:** `test_AC_002_dtu_reset_with_admin_token_returns_200` across all 4 clones
- **BC trace:** BC-3.5.001 postcondition 3

### AC-003 — Cross-clone token returns 401 (isolation path)

- **Tape:** [AC-003-cross-clone-token-returns-401.tape](AC-003-cross-clone-token-returns-401.tape)
- **GIF:** [AC-003-cross-clone-token-returns-401.gif](AC-003-cross-clone-token-returns-401.gif)
- **WebM:** [AC-003-cross-clone-token-returns-401.webm](AC-003-cross-clone-token-returns-401.webm)
- **Path demonstrated:** `test_AC_003_cross_clone_admin_token_returns_401` across all 4 clones — two independent clone instances per test; clone B's token is presented to clone A and rejected
- **BC trace:** BC-3.5.002 precondition 3

---

## Files in This Directory

```
docs/demo-evidence/W3-FIX-SEC-002/
  evidence-report.md                                      (this file)
  AC-001-reset-without-token-returns-401.tape
  AC-001-reset-without-token-returns-401.gif
  AC-001-reset-without-token-returns-401.webm
  AC-002-reset-with-correct-token-returns-200.tape
  AC-002-reset-with-correct-token-returns-200.gif
  AC-002-reset-with-correct-token-returns-200.webm
  AC-003-cross-clone-token-returns-401.tape
  AC-003-cross-clone-token-returns-401.gif
  AC-003-cross-clone-token-returns-401.webm
```

---

## Verification Properties Demonstrated

| VP | Description | Demo |
|----|-------------|------|
| VP-124 | Admin token check rejects missing/mismatched header on reset endpoint | AC-001, AC-003 |
| VP-125 | Correct admin token allows reset to proceed | AC-002 |
