---
story_id: W3-FIX-CODE-006
title: "Armis activity/risk endpoint org-id guard test coverage (CR-023 closure)"
wave: 3.4
level: "L4"
target_module: prism-dtu-armis
subsystems: [SS-01]
priority: P3
depends_on: []
blocks: []
estimated_days: 0.5
points: 2
status: planned
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-02T21:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review-pass4.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.5.001
verification_properties: [VP-124]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.5.001]
anchor_capabilities: [CAP-036]
anchor_subsystem: ["SS-01"]
tdd_mode: strict
parent_finding: "CR-023 (L) — gate-step-c-code-review-pass4.md"
# BC status: anchored — all BCs fully authored
---

# W3-FIX-CODE-006: Armis activity/risk endpoint org-id guard test coverage (CR-023 closure)

## Narrative

As a Prism maintainer, I want explicit test coverage for the dual-mode org-id guard on
Armis `get_device_activity` and `get_device_risk`, so that future changes to these
handlers cannot silently remove the guard without a failing test to catch the regression.

## Objective

Pass 4 of Gate Step C (`gate-step-c-code-review-pass4.md`) identified CR-023 (LOW):
W3-FIX-CODE-005 correctly extended the `is_real_org` dual-mode guard to
`get_device_activity` and `get_device_risk` (lines 205-209 and 241-245 in `devices.rs`),
but the new test file `cr017_tag_alert_org_id_guard.rs` covers only `post_device_tag`,
`delete_device_tag`, and `get_alerts`. No tests exercise the guard on the two activity/risk
endpoints. This story closes that gap.

**Scope:** Add test cases covering the dual-mode guard on:
- `GET /api/v1/devices/:device_id/activity` — handler: `get_device_activity`
- `GET /api/v1/devices/:device_id/risk` — handler: `get_device_risk`

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.5.001 | Harness Logical Isolation Invariants | Invariant 3: failure injection state scoped to target clone. The dual-mode `is_real_org` guard enforces the per-clone `instance_org_id` boundary that supports clone isolation. Test coverage for this guard prevents regressions that would silently bypass isolation. |

## Acceptance Criteria

### AC-001: Real-org mode test — `get_device_activity` rejects absent X-Org-Id with 401 (traces to BC-3.5.001 invariant 3)

A test function `test_get_device_activity_real_org_absent_header_returns_401` verifies:
- Given: a real-org Armis clone (instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID)
- When: `GET /api/v1/devices/:device_id/activity` is called without `X-Org-Id` header
- Then: HTTP 401 is returned

### AC-002: Real-org mode test — `get_device_activity` accepts correct X-Org-Id with 200 (traces to BC-3.5.001 postcondition 1)

A test function `test_get_device_activity_real_org_correct_header_returns_200` verifies:
- Given: a real-org Armis clone
- When: `GET /api/v1/devices/:device_id/activity` is called with the correct `X-Org-Id` header
- Then: HTTP 200 is returned

### AC-003: Default-instance mode test — `get_device_activity` passes without X-Org-Id (traces to BC-3.5.001 postcondition 2)

A test function `test_get_device_activity_default_instance_absent_header_returns_200` verifies:
- Given: a default-instance Armis clone (instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID)
- When: `GET /api/v1/devices/:device_id/activity` is called without `X-Org-Id` header
- Then: HTTP 200 is returned (backward compatibility — guard is a no-op for default instance)

### AC-004: Real-org mode test — `get_device_risk` rejects absent X-Org-Id with 401 (traces to BC-3.5.001 invariant 3)

A test function `test_get_device_risk_real_org_absent_header_returns_401` mirrors AC-001
for `GET /api/v1/devices/:device_id/risk`.

### AC-005: Real-org mode test — `get_device_risk` accepts correct X-Org-Id with 200 (traces to BC-3.5.001 postcondition 1)

A test function `test_get_device_risk_real_org_correct_header_returns_200` mirrors AC-002
for `GET /api/v1/devices/:device_id/risk`.

### AC-006: Default-instance mode test — `get_device_risk` passes without X-Org-Id (traces to BC-3.5.001 postcondition 2)

A test function `test_get_device_risk_default_instance_absent_header_returns_200` mirrors
AC-003 for `GET /api/v1/devices/:device_id/risk`.

### AC-007: All new tests pass under cargo test -p prism-dtu-armis (traces to BC-3.5.001 postcondition 5)

`cargo test -p prism-dtu-armis --all-features` passes with zero failures after the
new test file is added. No existing test is modified or removed.

## Tasks

### Part A: Orient in existing test corpus

1. Read `crates/prism-dtu-armis/tests/cr017_tag_alert_org_id_guard.rs` — capture the
   full test structure: how the real-org clone is constructed, how the default-instance
   clone is constructed, how HTTP requests are sent, and the exact assertion pattern.
2. Read `crates/prism-dtu-armis/src/routes/devices.rs:195-265` — confirm the guard
   at lines 205-209 (`get_device_activity`) and 241-245 (`get_device_risk`); confirm
   the route paths (`:device_id` parameter format) and the response body schema for
   the 200 and 401 cases.
3. Check whether device IDs required for the route parameter exist in the default
   clone state, or whether the test must construct a device ID programmatically.
   Mirror the approach used by existing `get_alerts` tests in `cr017_tag_alert_org_id_guard.rs`.

### Part B: Choose test file location

4. **Decision point:** Extend `cr017_tag_alert_org_id_guard.rs` vs create
   `cr023_activity_risk_org_id_guard.rs`. Use the following heuristic:
   - If `cr017_tag_alert_org_id_guard.rs` is under 200 lines, extend it.
   - If over 200 lines, create `cr023_activity_risk_org_id_guard.rs` to keep files manageable.
   Document the choice in the PR description.

### Part C: Write 6 test functions

5. Write `test_get_device_activity_real_org_absent_header_returns_401` (AC-001).
6. Write `test_get_device_activity_real_org_correct_header_returns_200` (AC-002).
7. Write `test_get_device_activity_default_instance_absent_header_returns_200` (AC-003).
8. Write `test_get_device_risk_real_org_absent_header_returns_401` (AC-004).
9. Write `test_get_device_risk_real_org_correct_header_returns_200` (AC-005).
10. Write `test_get_device_risk_default_instance_absent_header_returns_200` (AC-006).

### Part D: Integration

11. Run `cargo test -p prism-dtu-armis --all-features` — all tests pass including
    the 6 new functions (AC-007).
12. Run `cargo clippy -p prism-dtu-armis -- -D warnings` — zero new warnings.
13. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| `get_device_activity` guard (already implemented by W3-FIX-CODE-005) | prism-dtu-armis | `crates/prism-dtu-armis/src/routes/devices.rs:205-209` | Pure (existing guard; no change) |
| `get_device_risk` guard (already implemented by W3-FIX-CODE-005) | prism-dtu-armis | `crates/prism-dtu-armis/src/routes/devices.rs:241-245` | Pure (existing guard; no change) |
| New test functions (6) | prism-dtu-armis | `tests/cr017_tag_alert_org_id_guard.rs` OR `tests/cr023_activity_risk_org_id_guard.rs` | Effectful (HTTP test server) |

**Subsystem anchor justification:** SS-01 (Sensor Adapters) owns this story's scope
because `prism-dtu-armis` is a Sensor Adapter subsystem crate per the ARCH-INDEX
Subsystem Registry. The dual-mode org-id guard is an access-control mechanism within
the DTU HTTP layer of the sensor adapter stack.

**Dependency anchor justification:** `depends_on: []` — the guard implementations in
`devices.rs` already exist (landed in W3-FIX-CODE-005 PR #123); this story adds tests
for existing production code, requiring no other story to land first. `blocks: []` —
no downstream story is gated on this test coverage.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `devices.rs:205-209` / `241-245` guards | pure-core | Already implemented; this story adds no production code changes |
| New test functions | effectful-shell | Spin up Armis DTU HTTP test server; send HTTP requests; assert response codes |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `get_device_activity` called with correct `X-Org-Id` but device ID does not exist in clone state | HTTP 404 (or the existing handler's "not found" response) — the guard passed (HTTP 200 range vs 401); test should assert HTTP != 401 if exact status is implementation-dependent |
| EC-002 | Default-instance clone with `X-Org-Id` header present | Dual-mode guard: `is_real_org` is false, so `if is_real_org \|\| headers.get("x-org-id").is_some()` is true only because the header is present; `validate_org_id` fires and may return 401 if the header value doesn't match nil instance ID. Mirror the `test_get_alerts_*` pattern for this case |
| EC-003 | `get_device_risk` route path format differs from `get_device_activity` | Read `devices.rs` route registration to confirm both use `:device_id` path parameter in the same format |
| EC-004 | Running 6 new tests introduces concurrency with other Armis tests | All DTU test crates use per-test server instances; no shared state. No concern. |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3 500 |
| BC file (1 BC: BC-3.5.001) | ~2 500 |
| `crates/prism-dtu-armis/tests/cr017_tag_alert_org_id_guard.rs` (existing 8 tests, ~120 lines) | ~1 200 |
| `crates/prism-dtu-armis/src/routes/devices.rs:185-265` (guard locations + route paths) | ~800 |
| New test file (~80 lines for 6 test functions) | ~800 |
| `cargo test` output | ~500 |
| **Total** | **~9 300** |

Well within single-agent context window. This is a small, targeted test-coverage story.

## Previous Story Intelligence

- **W3-FIX-CODE-005** (PR #123 e4be29ae): extended the `is_real_org` dual-mode guard to
  `get_device_activity` and `get_device_risk` in `devices.rs` as part of the CR-017
  closure. This story adds the missing test coverage for those two endpoints — the
  production code is already correct; the gap is test-only.
- **Pattern lesson (from CR-023):** When a fix story extends scope beyond its original
  AC targets (W3-FIX-CODE-005 applied the guard to activity/risk in addition to the
  specified tags/alerts), the author must also extend the test scope to match. The
  absence of this discipline was the root cause of CR-023. Future fix stories should
  include a "sibling test scan" step: verify that every production code location touched
  has at least one test path covering it.
- **Previous Story Intelligence:** N/A for this story specifically — this is the first
  story in the "CR-023 closure" sub-thread.

## Architecture Compliance Rules

- The test functions MUST use the naming convention
  `test_<handler>_<mode>_<condition>_<expected>` consistent with
  `cr017_tag_alert_org_id_guard.rs` (e.g., `test_get_device_activity_real_org_absent_header_returns_401`).
- The Armis guard MUST be tested against `DTU_DEFAULT_INSTANCE_ORG_ID` (the Armis-specific
  sentinel), NOT `OrgId::from_uuid(Uuid::nil())`. Do not use the CrowdStrike sentinel.
- Production code (`devices.rs`) MUST NOT be modified by this story — it is already
  correct. All changes are test-only.
- If extending `cr017_tag_alert_org_id_guard.rs`, do NOT remove or modify any of the
  existing 8 test functions (3 for post_device_tag, 2 for delete_device_tag, 3 for
  get_alerts). All additions are strictly additive.

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| `axum` | workspace pin | Route handler; test server construction |
| `tokio` | workspace pin | `#[tokio::test]` async test runtime |
| `reqwest` or `axum` test client | workspace pin | HTTP request construction in tests |

No new Cargo dependencies introduced by this story.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-dtu-armis/tests/cr017_tag_alert_org_id_guard.rs` | Extend (if ≤200 lines) | Add 6 new test functions for activity/risk guard (AC-001..006) |
| OR `crates/prism-dtu-armis/tests/cr023_activity_risk_org_id_guard.rs` | Create (if >200 lines) | New file with 6 test functions; mirror structure of cr017 file |

**Only one of the above files is modified/created** — the implementer chooses based on
the file-size heuristic in Task 4. Document the choice in the PR description.

## Forbidden Dependencies

- Do NOT add any new external crate dependencies to `prism-dtu-armis`.
- Do NOT modify `crates/prism-dtu-armis/src/routes/devices.rs` — production code is
  already correct; this story is test-only.
- Do NOT use `OrgId::from_uuid(Uuid::nil())` in Armis tests. Armis uses
  `DTU_DEFAULT_INSTANCE_ORG_ID` as the default sentinel (a non-nil UUID) per the
  architecture compliance rule established in W3-FIX-CODE-005.
- Do NOT remove, rename, or alter any of the 8 existing test functions in
  `cr017_tag_alert_org_id_guard.rs`.
