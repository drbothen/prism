---
story_id: W3-FIX-SEC-002
title: "DTU clones: gate POST /dtu/reset with X-Admin-Token on Claroty/CrowdStrike/Armis/Slack"
wave: 3.1
level: "L4"
target_module: prism-dtu-claroty
subsystems: [SS-01]
priority: P0
depends_on: [W3-FIX-SEC-001]
blocks: []
estimated_days: 1
points: 3
status: merged
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-01T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.5.002-harness-network-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.2.001-per-org-sensor-data-isolation.md
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - .factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.5.001
  - BC-3.5.002
  - BC-3.2.001
verification_properties: [VP-124, VP-125]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.5.001, BC-3.5.002, BC-3.2.001]
anchor_capabilities: [CAP-036]
anchor_subsystem: ["SS-01"]
tdd_mode: strict
---

# W3-FIX-SEC-002: DTU clones — gate POST /dtu/reset with X-Admin-Token on Claroty/CrowdStrike/Armis/Slack

## Narrative

As a Prism security reviewer, I want `POST /dtu/reset` on all four affected DTU clones
to require a valid `X-Admin-Token` header (the same token already used to gate
`POST /dtu/configure`), so that a test client cannot erase another org's harness state
without authentication, preserving BC-3.2.001 isolation at the admin endpoint.

## Objective

Gate Step D identified SEC-002 (HIGH, CWE-306, OWASP A07): `POST /dtu/reset` on
Claroty, CrowdStrike, Armis, and Slack DTU clones is unauthenticated. An attacker who
can reach any clone's loopback port can issue `POST /dtu/reset` and erase all org-keyed
state without supplying any credential.

The admin token is already generated per-clone at startup (UUID v4, stored in clone
state) and is already checked on `POST /dtu/configure`. This story applies the identical
gate to `POST /dtu/reset`. No new token generation logic is needed — only the
existing middleware call is missing from the reset handler.

The Wave 2 SEC finding WGS-W2-003 applied this fix to PagerDuty and Jira. Claroty,
CrowdStrike, Armis, and Slack were not covered by that fix.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.5.001 | Harness Logical Isolation Invariants | Postcondition 3: harness teardown is clean and controlled; no external entity can corrupt shared state |
| BC-3.5.002 | Harness Network Isolation Invariants | Precondition 3: routing errors are observable at the HTTP layer; unauthenticated resets defeat this |
| BC-3.2.001 | Per-Org Sensor Data Isolation via Composite HashMap Key | Invariant 1: composite keying is the exclusive store scheme; unauthenticated reset bypasses the key guard entirely |

## Acceptance Criteria

### AC-001: Reset without admin token returns 401 (traces to BC-3.2.001 invariant 1)
`POST /dtu/reset` on each of the four clones with no `X-Admin-Token` header returns
HTTP 401 with JSON body `{"error": "missing or invalid admin token"}`. No state is
cleared.

### AC-002: Reset with correct admin token returns 200 (traces to BC-3.5.001 postcondition 3)
`POST /dtu/reset` with the clone's own admin token in `X-Admin-Token` returns HTTP 200
`{"status": "ok"}` and clears state. This is the existing happy path — it must continue
to work.

### AC-003: Cross-token attempt returns 401 (traces to BC-3.5.002 precondition 3)
`POST /dtu/reset` on Clone A using Clone B's admin token returns HTTP 401. The admin
token is per-clone and is not shared. Clone A's state is not cleared.

### AC-004: All four clones covered (traces to BC-3.2.001 invariant 1)
The `X-Admin-Token` gate is applied to `POST /dtu/reset` on `prism-dtu-claroty`,
`prism-dtu-crowdstrike`, `prism-dtu-armis`, and `prism-dtu-slack`. Confirmed by grep
for `dtu_reset` / `post_reset` handler signatures in each crate after the change.

### AC-005: No regression on configure endpoint (traces to BC-3.5.001 postcondition 3)
`POST /dtu/configure` (which already requires `X-Admin-Token`) continues to work
correctly on all four clones after the reset fix. No shared code paths are disturbed.

## Tasks

1. Read `crates/prism-dtu-claroty/src/routes/devices.rs` lines 337-340 (`dtu_reset`)
   and the existing `dtu_configure` handler to understand the current admin-token check
   pattern.
2. Extract or confirm the admin-token middleware extractor used in `dtu_configure` — it
   should be a function that reads `X-Admin-Token`, compares to `state.admin_token`,
   and returns HTTP 401 on mismatch.
3. Apply the same extractor/check to `dtu_reset` in `prism-dtu-claroty/src/routes/devices.rs`.
4. Repeat for `prism-dtu-crowdstrike/src/routes/mod.rs:29-37` — note the comment
   "No auth required" at line 29; remove that comment and add the token check.
5. Repeat for `prism-dtu-armis/src/routes/dtu.rs:60-69` (`post_reset`).
6. Repeat for `prism-dtu-slack/src/routes/dtu.rs:55-64` (`post_reset`).
7. Add `test_reset_requires_admin_token` integration tests to each crate's test suite
   covering: (a) missing token → 401, (b) correct token → 200, (c) wrong token → 401.
8. Run `cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-armis
   -p prism-dtu-slack --all-features` — all tests pass.
9. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| `dtu_reset` handler | prism-dtu-claroty | `crates/prism-dtu-claroty/src/routes/devices.rs` | Effectful (HTTP, state mutation) |
| `dtu_reset` handler | prism-dtu-crowdstrike | `crates/prism-dtu-crowdstrike/src/routes/mod.rs` | Effectful |
| `post_reset` handler | prism-dtu-armis | `crates/prism-dtu-armis/src/routes/dtu.rs` | Effectful |
| `post_reset` handler | prism-dtu-slack | `crates/prism-dtu-slack/src/routes/dtu.rs` | Effectful |
| Admin token extractor | all four | inline helper or shared via prism-dtu-common | Pure (returns Result) |
| Integration tests | all four | `tests/dtu_admin_test.rs` or equivalent | Effectful (HTTP) |

**Subsystem anchor justification:** SS-01 (Sensor Adapters) owns this story's scope
because all four DTU clone crates are Security Telemetry adapters and the fix is
entirely within their HTTP route modules per the ARCH-INDEX Subsystem Registry.

**Dependency anchor justification:** `depends_on: [W3-FIX-SEC-001]` — SEC-001 establishes
per-instance OrgId binding. SEC-002's reset-auth is logically cleaner once the org
identity is correctly bound, and the two stories touch overlapping route files; landing
SEC-001 first avoids merge conflicts. `blocks: []` — no other W3-FIX story depends on
reset auth being complete.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `X-Admin-Token` header present but empty string | HTTP 401; treat empty token same as missing |
| EC-002 | `X-Admin-Token` value is the correct token but with leading/trailing whitespace | HTTP 401 — do NOT trim; exact match only, matching the existing configure behavior |
| EC-003 | Two concurrent `POST /dtu/reset` requests with correct token | Both succeed; `Arc<Mutex>` or equivalent on state ensures clean reset |
| EC-004 | `POST /dtu/reset` called immediately after `POST /dtu/configure` with a fresh failure injection | Reset succeeds with admin token; state is cleared including injected failure mode |
| EC-005 | Slack clone — `post_reset` is on a different router mount than Claroty | Apply the same check regardless of router structure; verify the route is actually registered by checking the Slack router setup |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Admin token check (inline helper) | pure-core | Pure function: `&str` vs `&str` comparison, returns `Result<(), (StatusCode, Json)>` |
| `dtu_reset` / `post_reset` handlers | effectful-shell | Axum async handlers; perform HTTP I/O and mutate `Arc<CloneState>` |
| Integration tests | effectful-shell | Spawn real HTTP clones; perform loopback TCP connections |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~2 800 |
| BC files (3 BCs) | ~4 000 |
| Four reset handler files (~50 lines each) | ~1 200 |
| Four configure handler files (reference pattern, ~60 lines each) | ~1 400 |
| Four clone state files (admin_token field) | ~800 |
| Test files (new) | ~1 000 |
| Cargo output | ~500 |
| **Total** | **~11 700** |

Well within a single agent context window.

## Previous Story Intelligence

- **WGS-W2-003 (Wave 2 fix):** Applied the `X-Admin-Token` gate to PagerDuty and Jira
  reset handlers. The implementation pattern from those crates is the model for this fix.
  Before writing any new code, read the PagerDuty/Jira `post_reset` handlers to extract
  the exact middleware pattern used.
- **W3-FIX-SEC-001 (this sprint):** Establishes per-instance OrgId binding. Its changes
  to `clone.rs` state structs may affect which fields are accessible in reset handlers;
  review SEC-001 changes before implementing this story.

## Architecture Compliance Rules

- The admin-token check MUST be the same pattern (same comparison semantics, same 401
  error body format) used by `POST /dtu/configure` on the same clone. Inconsistency
  across endpoints confuses future implementers.
- Do NOT generate a new admin token for the reset endpoint. The token is per-clone-instance,
  not per-endpoint.
- The reset handler MUST clear state AFTER the token check passes, not before. The
  check-then-act order is required.
- The comment `"No auth required"` in `prism-dtu-crowdstrike/src/routes/mod.rs:29` MUST
  be removed when the auth check is added.

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| axum | workspace pin | `TypedHeader` or manual `HeaderMap` extraction for `X-Admin-Token` |
| serde_json / axum Json | workspace pin | HTTP 401 error body |

No new Cargo dependencies.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-dtu-claroty/src/routes/devices.rs` | Modify | Add token check to `dtu_reset` |
| `crates/prism-dtu-crowdstrike/src/routes/mod.rs` | Modify | Add token check to `dtu_reset`; remove "No auth required" comment |
| `crates/prism-dtu-armis/src/routes/dtu.rs` | Modify | Add token check to `post_reset` |
| `crates/prism-dtu-slack/src/routes/dtu.rs` | Modify | Add token check to `post_reset` |
| `crates/prism-dtu-{crate}/tests/dtu_admin_test.rs` | Create (×4) or modify existing | Add three-case token test per crate |

## Forbidden Dependencies

- Do NOT add `prism-dtu-common` as a new dependency of any crate that does not already
  depend on it. If the token-check helper is duplicated across four crates, that is
  acceptable; do not create a shared dependency solely for a 5-line helper.
