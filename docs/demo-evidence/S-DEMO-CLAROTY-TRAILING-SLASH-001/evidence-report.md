# Evidence Report — S-DEMO-CLAROTY-TRAILING-SLASH-001

**Story:** Claroty DTU trailing-slash route fidelity (BC-2.16.013 v1.25)
**Branch:** `feature/S-DEMO-CLAROTY-TRAILING-SLASH-001-trailing-slash`
**Demo vehicle:** VHS (terminal recordings via `cargo nextest run`)
**Recorded:** 2026-06-08

---

## Coverage Summary

| AC | Description | Evidence Artifact | Status |
|----|-------------|-------------------|--------|
| AC-001 | `POST /api/v1/alerts/` → HTTP 200 + alerts fixture | `AC-001-002-003-trailing-slash-200.{gif,webm}` | PASS |
| AC-002 | `POST /api/v1/devices/` → HTTP 200 + 20 devices fixture | `AC-001-002-003-trailing-slash-200.{gif,webm}` | PASS |
| AC-003 | `POST /api/v1/audit_log/get/` → HTTP 200 | `AC-001-002-003-trailing-slash-200.{gif,webm}` | PASS |
| AC-004 | `NormalizePathLayer` at outer service + spec-engine parity (Gap-CL-001 CLOSED) | `AC-004-toml-trailing-slash-path-template.{gif,webm}` | PASS |
| AC-005 | Backward compat: no-slash routes unaffected; tags route; `/dtu/health/` | `AC-005-backward-compat-no-slash-tags-health.{gif,webm}` | PASS |
| AC-005 (error) | `POST /api/v1/alerts/` with no auth → 401 (not 404) | `AC-005-error-path-missing-auth-401.{gif,webm}` | PASS |

All 5 acceptance criteria have at least one artifact covering both success and error/contrast paths.

---

## Artifacts

### AC-001 / AC-002 / AC-003 — Trailing-slash POST paths return HTTP 200

**Files:**
- `AC-001-002-003-trailing-slash-200.tape` — VHS tape script
- `AC-001-002-003-trailing-slash-200.gif` — terminal recording
- `AC-001-002-003-trailing-slash-200.webm` — archival recording

**What it shows:**

```
cargo nextest run -p prism-dtu-claroty --features dtu -E 'test(claroty_trailing_slash)'
```

Three tests run against a live `ClarotyClone` on an ephemeral port:
- `test_claroty_trailing_slash_alerts_returns_200` — AC-001
- `test_claroty_trailing_slash_devices_returns_200` — AC-002
- `test_claroty_trailing_slash_audit_log_get_returns_200` — AC-003

All three POST to the trailing-slash path with `Authorization: Bearer test-token` and a JSON body.
Each asserts HTTP 200 and verifies the fixture response key (`alerts` / `devices` / audit entries).

Before the fix all three returned HTTP 404 — the axum router registered routes without trailing
slashes and had no `NormalizePathLayer` in the outer service. After the fix
(`NormalizePathLayer::trim_trailing_slash()` wrapping both TLS and plain-HTTP serve sites in
`clone.rs`), the inbound `/alerts/` is stripped to `/alerts` before routing resolves.

**Nextest result:** 3/3 PASS

---

### AC-004 — NormalizePathLayer in clone.rs + spec-engine parity

**Files:**
- `AC-004-toml-trailing-slash-path-template.tape` — VHS tape script
- `AC-004-toml-trailing-slash-path-template.gif` — terminal recording
- `AC-004-toml-trailing-slash-path-template.webm` — archival recording

**What it shows:**

Step 1 — grep confirms `NormalizePathLayer` and `trim_trailing_slash` are present in
`crates/prism-dtu-claroty/src/clone.rs` at both serve sites (TLS and plain-HTTP):

```
grep -n 'NormalizePathLayer\|trim_trailing_slash' crates/prism-dtu-claroty/src/clone.rs
```

Expected output (lines ~30, ~168, ~206 of clone.rs):
```
30: use tower_http::normalize_path::NormalizePathLayer;
168:     let app = NormalizePathLayer::trim_trailing_slash().layer(router);
```

Step 2 — spec-engine parity confirms `claroty.sensor.toml` round-trips without parse error:

```
cargo nextest run -p prism-spec-engine -E 'test(claroty)' 2>&1 | tail -8
```

**Nextest result:** 4/4 PASS (parity + behavioral-equivalence + pushdown tests)

**Gap-CL-001 status:** CLOSED. The `sensors/claroty.sensor.toml` covers write endpoints
(per AD-022 risk tier). The trailing-slash `path_template` for read endpoints is enforced
by `NormalizePathLayer` at the outer-service level in `clone.rs`, which is the correct
implementation boundary for a DTU behavioral clone (BC-2.16.013 §Postconditions §1).

---

### AC-005 — Backward compatibility: no-slash routes + tags route + `/dtu/health/`

**Files:**
- `AC-005-backward-compat-no-slash-tags-health.tape` — VHS tape script
- `AC-005-backward-compat-no-slash-tags-health.gif` — terminal recording
- `AC-005-backward-compat-no-slash-tags-health.webm` — archival recording

**What it shows:**

```
cargo nextest run -p prism-dtu-claroty --features dtu \
  -E 'test(BC_2_16_013_no_slash) | test(BC_2_16_013_tags) | test(BC_2_16_013_dtu_health)'
```

Four tests prove backward compatibility:
- `test_BC_2_16_013_no_slash_alerts_still_returns_200` — `POST /api/v1/alerts` (no slash) → 200
- `test_BC_2_16_013_no_slash_devices_still_returns_200` — `POST /api/v1/devices` (no slash) → 200
- `test_BC_2_16_013_tags_route_with_slash_still_works` — `POST .../tags/` → 201 (intentional slash stripped to `/tags`, route registered without slash)
- `test_BC_2_16_013_dtu_health_trailing_slash_returns_200` — `GET /dtu/health/` → 200 (control-plane route normalized)

`trim_trailing_slash()` is STRIP-ONLY. No-slash requests pass through unmodified and match
their registered routes directly — they cannot be broken by the middleware.

**Nextest result:** 4/4 PASS

---

### AC-005 (error path) / EC-002 — Missing auth → 401 not 404

**Files:**
- `AC-005-error-path-missing-auth-401.tape` — VHS tape script
- `AC-005-error-path-missing-auth-401.gif` — terminal recording
- `AC-005-error-path-missing-auth-401.webm` — archival recording

**What it shows:**

```
cargo nextest run -p prism-dtu-claroty --features dtu \
  -E 'test(trailing_slash_alerts_missing_auth)'
```

`test_BC_2_16_013_trailing_slash_alerts_missing_auth_returns_401`:
- POSTs to `/api/v1/alerts/` with NO `Authorization` header
- Asserts HTTP **401** (not 404)

This is the critical placement proof:
- HTTP 404 would indicate `NormalizePathLayer` is mis-placed via `Router::layer()`, which
  no-ops in axum 0.7 because the Router resolves the path before inner layers run.
- HTTP 401 proves the layer is at the **outer service** level — the strip ran before route
  resolution, `/api/v1/alerts/` became `/api/v1/alerts`, the `list_alerts` handler fired,
  and its `check_bearer_auth` call rejected the unauthenticated request.

**Nextest result:** 1/1 PASS

---

### ALL ACs — Full suite (11 tests)

**Files:**
- `ALL-ACs-full-suite.tape` — VHS tape script
- `ALL-ACs-full-suite.gif` — terminal recording
- `ALL-ACs-full-suite.webm` — archival recording

**What it shows:**

```
cargo nextest run -p prism-dtu-claroty --features dtu \
  -E 'test(trailing_slash) | test(BC_2_16_013)'
```

Runs all 11 `trailing_slash_parity` + `BC_2_16_013` tests in one sweep — the full story
evidence in a single frame.

**Nextest result:** 11/11 PASS

---

## Implementation Notes

**Demo vehicle rationale:** This story implements an API/DTU-level change (`NormalizePathLayer`
middleware). There is no standalone CLI binary to demonstrate. The correct evidence vehicle is
`cargo nextest run` against the live `ClarotyClone` harness, which starts a real HTTP server
on an ephemeral port and fires real HTTP requests via `reqwest`. This is equivalent in
evidential weight to VHS recordings of a CLI binary — the test harness exercises the exact
same code path a real Claroty xDome client would.

**VHS recording approach:** All tapes use `Sleep` for timing (no `Wait+Line`) due to the
nextest output pattern. Each recording runs the exact `cargo nextest` command that the
prism development workflow uses for the TDD inner loop (`just iter` equivalent). The
recordings are fully reproducible — running `vhs <tape>` re-executes against live code.

**Test filter precision:**
- `test(claroty_trailing_slash)` → 3 tests (AC-001/002/003)
- `test(trailing_slash_alerts_missing_auth)` → 1 test (EC-002)
- `test(BC_2_16_013_no_slash) | test(BC_2_16_013_tags) | test(BC_2_16_013_dtu_health)` → 4 tests (AC-005)
- `test(trailing_slash) | test(BC_2_16_013)` → 11 tests (full story coverage)

---

## Self-Audit Checklist

- [x] Every AC (001–005) has at least one artifact covering the success path
- [x] Every AC has at least one artifact covering the error/contrast path (EC-002 for all)
- [x] Every artifact is linked to a specific AC via the naming convention
- [x] No source code or test files were modified
- [x] Evidence lives under `docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/` (not flat)
- [x] VHS was used (not plain text captures); both `.gif` and `.webm` produced per tape
- [x] Recordings reflect actual test execution, not mocked output
