# Demo Evidence Report — W3-FIX-SEC-001

**Story:** W3-FIX-SEC-001: DTU clones — bind OrgId to clone instance, reject mismatched X-Org-Id header
**Branch:** `feature/W3-FIX-SEC-001`
**HEAD SHA:** `f8b47ccd32906be21f063e53cadcf464641e5ed8`
**Recorded:** 2026-05-01
**Product type:** CLI (Rust) — VHS terminal recordings

---

## Security Context

| CWE | OWASP | Description |
|-----|-------|-------------|
| CWE-287 | A01 | Improper Authentication — `extract_org_id` accepted caller-supplied header with no validation |
| CWE-639 | A01 | Authorization Bypass Through User-Controlled Key — any client reaching a clone's loopback port could supply an arbitrary UUID and access a different org's state |

**Fix:** Each clone validates the incoming `X-Org-Id` header against its own `state.instance_org_id` (assigned by the harness at startup). Mismatch returns HTTP 401 with a JSON error body.

---

## Auth Model per DTU Clone

| Clone | Auth Model | Header Name | Missing Header | Mismatch Behavior |
|-------|-----------|-------------|----------------|-------------------|
| prism-dtu-claroty | A (single-org strict) | `X-Org-Id` | 401 | 401 `org_id mismatch` |
| prism-dtu-crowdstrike | A (single-org strict) | `X-Org-Id` | 401 | 401 `org_id mismatch` |
| prism-dtu-cyberint | B (multi-org routing hint) | `X-Prism-Org-Id` | 200 (defaults to instance session) | 401 `org_id mismatch` |
| prism-dtu-armis | Validate-on-presence (backcompat) | `X-Org-Id` | 200 (guard skipped) | 401 `org_id mismatch` |

---

## Acceptance Criteria Coverage

| AC | Description | Behavioral Contract | Result | Recording |
|----|-------------|-------------------|--------|-----------|
| AC-001 | Same-org request returns 200 — `X-Org-Id` matches `instance_org_id` | BC-3.2.001 postcondition 1 | PASS (4 crates) | [AC-001.gif](AC-001-same-org-returns-200.gif) |
| AC-002 | Cross-org spoofing returns 401 + JSON error | BC-3.5.002 precondition 3 | PASS (4 crates, 8 tests) | [AC-002.gif](AC-002-cross-org-returns-401.gif) |
| AC-003 | Missing header: Model A (401), Model B (200 default), Armis (200 backcompat) | BC-3.5.001 postcondition 1 | PASS (per-clone semantics) | [AC-003.gif](AC-003-auth-model-per-clone.gif) |
| AC-004 | All four DTU clones covered — 30 tests total | BC-3.2.001 invariant 1 | PASS (30/30) | [AC-004.gif](AC-004-all-four-clones-covered.gif) |
| AC-005 | `test_cross_org_header_rejected` regression in each crate | BC-3.5.002 precondition 3 | PASS (4/4 crates) | [AC-005.gif](AC-005-regression-cross-org-rejected.gif) |
| AC-006 | Pre-existing multi_tenant tests still pass — no regressions | BC-3.5.001 postcondition 1 | PASS | [AC-006.gif](AC-006-positive-paths-pass.gif) |

**Summary: 6/6 ACs demonstrated. 30/30 tests pass.**

---

## Demo Recordings Index

### AC-001 — Same-org request returns 200

- **Tape:** [AC-001-same-org-returns-200.tape](AC-001-same-org-returns-200.tape)
- **GIF:** [AC-001-same-org-returns-200.gif](AC-001-same-org-returns-200.gif)
- **WebM:** [AC-001-same-org-returns-200.webm](AC-001-same-org-returns-200.webm)
- **Path demonstrated:** `test_AC_001_x_org_id_validated_against_bearer_token` across all 4 crates
- **BC trace:** BC-3.2.001 postcondition 1

### AC-002 — Cross-org spoofing returns 401

- **Tape:** [AC-002-cross-org-returns-401.tape](AC-002-cross-org-returns-401.tape)
- **GIF:** [AC-002-cross-org-returns-401.gif](AC-002-cross-org-returns-401.gif)
- **WebM:** [AC-002-cross-org-returns-401.webm](AC-002-cross-org-returns-401.webm)
- **Paths demonstrated:** `test_AC_002_cross_org_credential_returns_401` + `test_AC_002_cross_org_401_body_is_json_error_object` (8 tests total, 2 per crate)
- **BC trace:** BC-3.5.002 precondition 3

### AC-003 — Per-clone auth model semantics

- **Tape:** [AC-003-auth-model-per-clone.tape](AC-003-auth-model-per-clone.tape)
- **GIF:** [AC-003-auth-model-per-clone.gif](AC-003-auth-model-per-clone.gif)
- **WebM:** [AC-003-auth-model-per-clone.webm](AC-003-auth-model-per-clone.webm)
- **Paths demonstrated:**
  - Claroty/CrowdStrike: `test_AC_003_missing_x_org_id_header_returns_401` (model A strict)
  - Cyberint: `test_AC_003_cyberint_missing_header_returns_default_session_or_400` + `test_AC_003_cyberint_mismatched_header_returns_401_session_not_found` (model B)
  - Armis: `test_AC_003_armis_validate_on_presence_missing_header_allowed_for_backcompat`
- **BC trace:** BC-3.5.001 postcondition 1

### AC-004 — All four clones: full x_org_id_auth suite

- **Tape:** [AC-004-all-four-clones-covered.tape](AC-004-all-four-clones-covered.tape)
- **GIF:** [AC-004-all-four-clones-covered.gif](AC-004-all-four-clones-covered.gif)
- **WebM:** [AC-004-all-four-clones-covered.webm](AC-004-all-four-clones-covered.webm)
- **Paths demonstrated:** Full `x_org_id_auth` test suite — 8 tests (claroty) + 7 tests (crowdstrike) + 8 tests (cyberint) + 7 tests (armis) = 30 total
- **BC trace:** BC-3.2.001 invariant 1

### AC-005 — Regression: cross-org header rejected

- **Tape:** [AC-005-regression-cross-org-rejected.tape](AC-005-regression-cross-org-rejected.tape)
- **GIF:** [AC-005-regression-cross-org-rejected.gif](AC-005-regression-cross-org-rejected.gif)
- **WebM:** [AC-005-regression-cross-org-rejected.webm](AC-005-regression-cross-org-rejected.webm)
- **Paths demonstrated:** `test_cross_org_header_rejected` in each of 4 crates (4 tests)
- **BC trace:** BC-3.5.002 precondition 3 (HS-003-02 invariant)

### AC-006 — Pre-existing positive paths pass (no regressions)

- **Tape:** [AC-006-positive-paths-pass.tape](AC-006-positive-paths-pass.tape)
- **GIF:** [AC-006-positive-paths-pass.gif](AC-006-positive-paths-pass.gif)
- **WebM:** [AC-006-positive-paths-pass.webm](AC-006-positive-paths-pass.webm)
- **Paths demonstrated:** Full `multi_tenant` integration test suite across all 4 crates — pre-existing tests unmodified
- **BC trace:** BC-3.5.001 postcondition 1

---

## Files in This Directory

```
docs/demo-evidence/W3-FIX-SEC-001/
  evidence-report.md                        (this file)
  AC-001-same-org-returns-200.tape
  AC-001-same-org-returns-200.gif
  AC-001-same-org-returns-200.webm
  AC-002-cross-org-returns-401.tape
  AC-002-cross-org-returns-401.gif
  AC-002-cross-org-returns-401.webm
  AC-003-auth-model-per-clone.tape
  AC-003-auth-model-per-clone.gif
  AC-003-auth-model-per-clone.webm
  AC-004-all-four-clones-covered.tape
  AC-004-all-four-clones-covered.gif
  AC-004-all-four-clones-covered.webm
  AC-005-regression-cross-org-rejected.tape
  AC-005-regression-cross-org-rejected.gif
  AC-005-regression-cross-org-rejected.webm
  AC-006-positive-paths-pass.tape
  AC-006-positive-paths-pass.gif
  AC-006-positive-paths-pass.webm
```

---

## Verification Properties Demonstrated

| VP | Description | Demo |
|----|-------------|------|
| VP-124 | `validate_org_id` rejects missing/mismatched headers | AC-002, AC-003 |
| VP-125 | Correct `X-Org-Id` allows request to proceed | AC-001 |
| VP-126 | All four DTU crates apply the same pattern | AC-004 |
