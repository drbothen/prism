---
story_id: W3-FIX-SEC-005
title: "5-DTU admin-token uniformity — constant-time comparison + post_reset gate (cyberint/jira/nvd/pagerduty/threatintel)"
wave: 3.4
level: "L4"
target_module: prism-dtu-cyberint
subsystems: [SS-01]
priority: P1
depends_on: []
blocks: []
estimated_days: 1
points: 5
status: planned
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-02T21:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review-pass4.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.5.002-harness-network-isolation.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.5.001
  - BC-3.5.002
verification_properties: [VP-124, VP-126]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.5.001, BC-3.5.002]
anchor_capabilities: [CAP-036]
anchor_subsystem: ["SS-01"]
tdd_mode: strict
parent_finding: "CR-021 (M), CR-022 (L) — gate-step-c-code-review-pass4.md"
# BC status: anchored — all BCs fully authored
---

# W3-FIX-SEC-005: 5-DTU admin-token uniformity — constant-time comparison + post_reset gate (cyberint/jira/nvd/pagerduty/threatintel)

## Narrative

As a Prism security reviewer, I want the admin-token enforcement in 5 DTU clones
(cyberint, jira, nvd, pagerduty, threatintel) brought into uniformity with the 4
already-converted DTUs (armis, claroty, crowdstrike, slack) so that the ADR-003
Amendment #5 invariant holds across the entire DTU surface and the test-isolation
threat model is uniformly enforced.

## Objective

Pass 4 of Gate Step C (`gate-step-c-code-review-pass4.md`) identified CR-021 (MEDIUM)
and CR-022 (LOW) — together exposing that W3-FIX-SEC-002 (PR #119) and W3-FIX-SEC-004
(PR #122) brought armis, claroty, crowdstrike, and slack to full admin-token parity but
left 5 DTU clones entirely unaddressed. The 5 remaining DTUs have two distinct defects:

| ID | Severity | CWE | One-line description |
|----|----------|-----|----------------------|
| CR-021 | MEDIUM | CWE-863 | `post_reset` / `dtu_reset` has NO admin-token gate in all 5 DTUs; any caller can reset state |
| CR-022 | LOW | CWE-208 | `post_configure` still uses short-circuit `!=` comparison instead of `subtle::ConstantTimeEq` |

**10 fix sites total: 5 DTUs × 2 endpoints (post_configure + post_reset/dtu_reset)**

Sites to fix (verified by grep on develop@e4be29ae):

| DTU | post_configure (CWE-208) | post_reset (CWE-863 — NO GATE) |
|-----|--------------------------|--------------------------------|
| cyberint | `crates/prism-dtu-cyberint/src/routes/dtu.rs:38` | `crates/prism-dtu-cyberint/src/routes/dtu.rs:61` |
| jira | `crates/prism-dtu-jira/src/routes/dtu.rs:36` | `crates/prism-dtu-jira/src/routes/dtu.rs:58` |
| nvd | `crates/prism-dtu-nvd/src/routes/dtu.rs:69` | `crates/prism-dtu-nvd/src/routes/dtu.rs:89` |
| pagerduty | `crates/prism-dtu-pagerduty/src/routes/dtu.rs:66` | `crates/prism-dtu-pagerduty/src/routes/dtu.rs:89` |
| threatintel | `crates/prism-dtu-threatintel/src/routes/dtu.rs:308` | `crates/prism-dtu-threatintel/src/routes/dtu.rs:27` (`dtu_reset`) |

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.5.001 | Harness Logical Isolation Invariants | Invariant 3: failure injection state scoped to target clone. The `admin_token` gate is the enforcement point that ensures only authorized callers can reset per-clone state. CR-021 leaves 5 DTUs' reset endpoints completely unauthenticated — any caller can disrupt test isolation. |
| BC-3.5.002 | Harness Network Isolation Invariants | Precondition 6: each clone's authentication middleware is initialized with that clone's own `admin_token`. CR-021/CR-022 violate this precondition for all 5 unguarded DTUs. |

## Acceptance Criteria

### AC-001: CR-022 — All 5 DTUs' post_configure use constant-time comparison (traces to BC-3.5.002 precondition 6)

In all 5 affected DTU crates (`prism-dtu-cyberint`, `prism-dtu-jira`, `prism-dtu-nvd`,
`prism-dtu-pagerduty`, `prism-dtu-threatintel`), the `post_configure` handler replaces
the short-circuit `!=` comparison:

```rust
// BEFORE (CWE-208: timing oracle):
if provided != Some(state.admin_token.as_str()) { ... }

// AFTER (constant-time, matching armis/claroty/crowdstrike/slack pattern):
use subtle::ConstantTimeEq;
let provided_bytes = provided.unwrap_or("").as_bytes();
let expected_bytes = state.admin_token.as_bytes();
let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
if !valid { ... }
```

After this fix:
- The 5 DTU `post_configure` handlers perform constant-time byte comparison for
  `X-Admin-Token`; branch timing does not depend on the position of the first differing byte.
- Observable HTTP behavior is unchanged: mismatched/absent token → 401; correct token → next handler step.
- `subtle` is added to each crate's `Cargo.toml` if not already present (workspace dep pattern).

### AC-002: CR-021 — All 5 DTUs' post_reset/dtu_reset gate on X-Admin-Token with constant-time comparison (traces to BC-3.5.002 precondition 6)

In all 5 affected DTU crates, the `post_reset` / `dtu_reset` handler receives an
`X-Admin-Token` gate matching the Armis reference implementation at
`crates/prism-dtu-armis/src/routes/dtu.rs:77-95`:

```rust
pub async fn post_reset(
    State(state): State<Arc<DtuState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok()).unwrap_or("");
    let provided_bytes = provided.as_bytes();
    let expected_bytes = state.admin_token.as_bytes();
    let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
    if !valid {
        return (StatusCode::UNAUTHORIZED,
                Json(json!({"error": "missing or invalid admin token"})))
            .into_response();
    }
    // ... existing reset logic follows unchanged
}
```

The admin-token gate MUST be the first check in the handler body — before any org-id
branch logic (e.g., Cyberint's `X-Prism-Org-Id` scoped-reset path). This matches the
Armis/Slack pattern where the gate is a prerequisite check, not an alternative branch.

After this fix:
- A call to `POST /dtu/reset` (or the crate-specific equivalent) without `X-Admin-Token`
  → HTTP 401 for all 5 DTUs.
- A call with an incorrect token → HTTP 401.
- A call with the correct token → HTTP 200 + existing reset semantics unchanged.

### AC-003: Positive test — post_reset accepts correct admin token for all 5 DTUs (traces to BC-3.5.001 invariant 3)

For each of the 5 DTUs, a test confirms that `POST /dtu/reset` with the correct
`X-Admin-Token` header returns HTTP 200 (positive case). This ensures the gate does not
regress valid reset flows.

### AC-004: New regression test files for all 5 DTUs (traces to BC-3.5.002 precondition 6)

**Per-crate test files to create:**

| Crate | Test File | Description |
|-------|-----------|-------------|
| prism-dtu-jira | `tests/td_wv0_07_configure_requires_admin_token.rs` | Mirror of existing cyberint pattern — verify post_configure gate |
| prism-dtu-nvd | `tests/td_wv0_07_configure_requires_admin_token.rs` | Same |
| prism-dtu-pagerduty | `tests/td_wv0_07_configure_requires_admin_token.rs` | Same |
| prism-dtu-threatintel | `tests/td_wv0_07_configure_requires_admin_token.rs` | Same |
| prism-dtu-cyberint | `tests/td_wv0_08_reset_requires_admin_token.rs` | New — verify post_reset gate |
| prism-dtu-jira | `tests/td_wv0_08_reset_requires_admin_token.rs` | New |
| prism-dtu-nvd | `tests/td_wv0_08_reset_requires_admin_token.rs` | New |
| prism-dtu-pagerduty | `tests/td_wv0_08_reset_requires_admin_token.rs` | New |
| prism-dtu-threatintel | `tests/td_wv0_08_reset_requires_admin_token.rs` | New |

**Note:** `crates/prism-dtu-cyberint/tests/td_wv0_07_configure_requires_admin_token.rs`
already exists and covers the cyberint configure gate — do NOT create a duplicate.

Each test file must contain at minimum:
- `test_reset_requires_admin_token_missing_returns_401` — POST /dtu/reset without header → 401
- `test_reset_requires_admin_token_wrong_returns_401` — POST /dtu/reset with wrong token → 401
- `test_reset_correct_admin_token_returns_200` — POST /dtu/reset with correct token → 200

Mirror the structure from `crates/prism-dtu-cyberint/tests/td_wv0_07_configure_requires_admin_token.rs`.

### AC-005: cargo test --workspace --features dtu passes (traces to BC-3.5.001 postcondition 5)

`cargo test --workspace --features dtu` (or equivalent nextest command) passes with
zero failures after all 10 fix sites and all 9 new test files are in place.

### AC-006: subtle dependency present in each affected crate's Cargo.toml (traces to BC-3.5.002 precondition 6)

`subtle = { workspace = true }` is present in the `[dependencies]` section of:
- `crates/prism-dtu-cyberint/Cargo.toml`
- `crates/prism-dtu-jira/Cargo.toml`
- `crates/prism-dtu-nvd/Cargo.toml`
- `crates/prism-dtu-pagerduty/Cargo.toml`
- `crates/prism-dtu-threatintel/Cargo.toml`

The workspace root `Cargo.toml` already has `subtle = "2"` from W3-FIX-SEC-004; add
the per-crate entries only. If a crate already has `subtle` for another reason, confirm
the workspace-dep form is used (not a duplicate direct pin).

## Tasks

### Part A: Read reference implementations

1. Read `crates/prism-dtu-armis/src/routes/dtu.rs:77-95` — capture the exact
   `post_reset` gate pattern (the canonical reference for AC-002).
2. Read `crates/prism-dtu-armis/src/routes/dtu.rs:40-55` — capture the exact
   `post_configure` ct_eq pattern (the canonical reference for AC-001).
3. Read `crates/prism-dtu-cyberint/tests/td_wv0_07_configure_requires_admin_token.rs`
   — capture the test structure to mirror for the 4 new configure test files and all
   5 reset test files.

### Part B: Fix post_configure in 5 crates (CR-022)

4. Read `crates/prism-dtu-cyberint/src/routes/dtu.rs:35-45` — locate the `!=` comparison.
   Replace with `subtle::ConstantTimeEq::ct_eq` pattern (AC-001).
5. Read `crates/prism-dtu-jira/src/routes/dtu.rs:32-45` — same fix.
6. Read `crates/prism-dtu-nvd/src/routes/dtu.rs:65-80` — same fix.
7. Read `crates/prism-dtu-pagerduty/src/routes/dtu.rs:62-80` — same fix.
8. Read `crates/prism-dtu-threatintel/src/routes/dtu.rs:305-315` — same fix.

### Part C: Add post_reset gate in 5 crates (CR-021)

9. Read `crates/prism-dtu-cyberint/src/routes/dtu.rs:58-80` — locate `post_reset`.
   Add admin-token gate as the FIRST check before org-id branch (AC-002).
   Update `ac_8_reset_semantics.rs` or equivalent test to supply the admin token
   so existing tests continue to pass.
10. Read `crates/prism-dtu-jira/src/routes/dtu.rs:54-70` — same treatment.
11. Read `crates/prism-dtu-nvd/src/routes/dtu.rs:85-100` — same treatment.
12. Read `crates/prism-dtu-pagerduty/src/routes/dtu.rs:85-100` — same treatment.
13. Read `crates/prism-dtu-threatintel/src/routes/dtu.rs:24-40` (`dtu_reset`) — same treatment.

### Part D: Update Cargo.toml for 5 crates

14. For each of the 5 crates, add `subtle = { workspace = true }` to `[dependencies]`
    in their `Cargo.toml` (AC-006).
15. Confirm `subtle = "2"` is present in workspace root `Cargo.toml` from
    W3-FIX-SEC-004 (do NOT add again if already present; check first).

### Part E: Write new test files

16. For each of the 4 crates (jira, nvd, pagerduty, threatintel), create
    `tests/td_wv0_07_configure_requires_admin_token.rs` mirroring cyberint's existing
    test structure (AC-004).
17. For each of the 5 crates, create
    `tests/td_wv0_08_reset_requires_admin_token.rs` with 3 test functions (AC-003, AC-004).
18. Verify existing reset tests in the 5 crates (e.g., `ac_8_reset_semantics.rs`) still
    pass — update them to supply the correct `X-Admin-Token` header if they previously
    called the reset endpoint without authentication.

### Part F: Integration

19. Run `cargo test -p prism-dtu-cyberint -p prism-dtu-jira -p prism-dtu-nvd
    -p prism-dtu-pagerduty -p prism-dtu-threatintel --all-features` — all tests pass.
20. Run `grep -r 'provided != Some' crates/prism-dtu-cyberint crates/prism-dtu-jira
    crates/prism-dtu-nvd crates/prism-dtu-pagerduty crates/prism-dtu-threatintel` —
    zero matches (confirms no residual `!=` comparisons against admin_token).
21. Run `cargo clippy --workspace -- -D warnings` — zero new warnings.
22. Run `cargo test --workspace --features dtu` — all tests pass (AC-005).
23. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| Cyberint `post_configure` ct_eq | prism-dtu-cyberint | `crates/prism-dtu-cyberint/src/routes/dtu.rs:38` | Pure (comparison change; no I/O) |
| Cyberint `post_reset` gate | prism-dtu-cyberint | `crates/prism-dtu-cyberint/src/routes/dtu.rs:61` | Pure (gate addition; no I/O) |
| Jira `post_configure` ct_eq | prism-dtu-jira | `crates/prism-dtu-jira/src/routes/dtu.rs:36` | Pure |
| Jira `post_reset` gate | prism-dtu-jira | `crates/prism-dtu-jira/src/routes/dtu.rs:58` | Pure |
| NVD `post_configure` ct_eq | prism-dtu-nvd | `crates/prism-dtu-nvd/src/routes/dtu.rs:69` | Pure |
| NVD `post_reset` gate | prism-dtu-nvd | `crates/prism-dtu-nvd/src/routes/dtu.rs:89` | Pure |
| PagerDuty `post_configure` ct_eq | prism-dtu-pagerduty | `crates/prism-dtu-pagerduty/src/routes/dtu.rs:66` | Pure |
| PagerDuty `post_reset` gate | prism-dtu-pagerduty | `crates/prism-dtu-pagerduty/src/routes/dtu.rs:89` | Pure |
| ThreatIntel `post_configure` ct_eq | prism-dtu-threatintel | `crates/prism-dtu-threatintel/src/routes/dtu.rs:308` | Pure |
| ThreatIntel `dtu_reset` gate | prism-dtu-threatintel | `crates/prism-dtu-threatintel/src/routes/dtu.rs:27` | Pure |
| New test files (9 total) | per-crate test directories | `tests/td_wv0_07_*.rs`, `tests/td_wv0_08_*.rs` | Effectful (HTTP test servers) |

**Subsystem anchor justification:** SS-01 (Sensor Adapters) owns this story's scope
because all 5 affected crates (`prism-dtu-cyberint`, `prism-dtu-jira`, `prism-dtu-nvd`,
`prism-dtu-pagerduty`, `prism-dtu-threatintel`) are Sensor Adapter subsystem crates per
the ARCH-INDEX Subsystem Registry. The admin-token enforcement is a security boundary
within the DTU test infrastructure, which is definitively SS-01 scope.

**Dependency anchor justification:** `depends_on: []` — this story is self-contained;
W3-FIX-SEC-002 and W3-FIX-SEC-004 have already landed and established the workspace
`subtle` dependency and the armis/claroty/crowdstrike/slack pattern. This story simply
applies the same pattern to the 5 remaining DTUs. `blocks: []` — no downstream story
depends on these 5 DTU clones being security-hardened before proceeding.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| 5× `post_configure` ct_eq change | pure-core | Replaces one comparison operator with another; no I/O; no state mutation |
| 5× `post_reset` / `dtu_reset` gate addition | pure-core | Adds early-return guard; no I/O; no new state; existing post-gate logic unchanged |
| New test functions (9 files × 3 tests) | effectful-shell | Spawn HTTP test servers; marked `#[tokio::test]` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `POST /dtu/reset` to any of the 5 DTUs with no `X-Admin-Token` header | HTTP 401 returned before any reset logic executes |
| EC-002 | `POST /dtu/reset` with wrong token (correct length, wrong bytes) | CT comparison returns `false` in constant time; HTTP 401 |
| EC-003 | `POST /dtu/reset` with correct token + valid `X-Prism-Org-Id` (Cyberint scoped reset) | Gate passes (CT compare true); org-scoped reset executes normally — gate is added BEFORE the org-id branch, not inside it |
| EC-004 | `POST /dtu/reset` with correct token but no `X-Prism-Org-Id` (Cyberint global reset) | Gate passes; full state reset executes normally |
| EC-005 | Existing tests that call `POST /dtu/reset` without supplying `X-Admin-Token` | Will fail with 401 — these tests MUST be updated to supply the token (see Task 18) |
| EC-006 | `subtle` already present in a crate's Cargo.toml from another dep | Check first; add workspace dep form only if missing; do not create a duplicate pin |
| EC-007 | Token length mismatch: provided="" (empty string, unwrap_or default) | CT compare on different-length slices returns false without timing leak; HTTP 401 |
| EC-008 | ThreatIntel `dtu_reset` has different handler signature than the 4-arg pattern | Read the actual signature in Task 13; apply the same gate logic adapted to the actual parameter set |
| EC-009 | A DTU crate already has admin token checks in post_configure (unexpected) | Verify with grep before modifying; if ct_eq already present, skip that site and document in PR description |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~5 500 |
| BC files (2 BCs: BC-3.5.001, BC-3.5.002) | ~4 500 |
| Reference: armis `dtu.rs:40-95` (post_configure + post_reset patterns) | ~700 |
| Reference: cyberint `td_wv0_07_configure_requires_admin_token.rs` (test template) | ~500 |
| 5× DTU `dtu.rs` slices (35-100 lines each, post_configure + post_reset) | ~3 500 |
| 5× DTU `Cargo.toml` files (~30 lines each) | ~750 |
| Workspace root `Cargo.toml` (workspace deps section ~30 lines) | ~300 |
| 5× existing reset test files to audit/update | ~2 000 |
| `cargo test` + `cargo clippy` + `grep` verification output | ~1 500 |
| **Total** | **~19 250** |

Fits in a single agent context window. Load only the specific line ranges listed in
Tasks; do not load entire crate source files.

## Previous Story Intelligence

- **W3-FIX-SEC-002** (PR #119 / SEC-NEW-001 closure): added `X-Admin-Token` gate to
  armis, claroty, crowdstrike, and slack `post_reset` / `dtu_reset` handlers. This
  story applies the same fix to the 5 remaining DTUs that were out of scope at the time.
- **W3-FIX-SEC-004** (PR #122 / SEC-P3-003 closure): migrated the `post_configure`
  comparison to `subtle::ConstantTimeEq` in armis, claroty, crowdstrike, and slack.
  This story applies the same migration to the 5 remaining DTUs' `post_configure`.
- **Lesson (sibling gap pattern):** Both W3-FIX-SEC-002 and W3-FIX-SEC-004 applied
  fixes to exactly 4 of the 9 DTUs with admin-token enforcement. CR-021/CR-022 are
  the expected sibling-propagation gap findings. The corrective action from D-148 /
  W3-FIX-CODE-005 applies here: after applying a fix pattern, always run
  `grep -r '<old-pattern>' crates/` to confirm zero residues before declaring done.
- **Cyberint special case:** Cyberint's `post_reset` has an org-id branch
  (`X-Prism-Org-Id` → scoped reset vs global reset). The admin-token gate must be
  inserted BEFORE this branch, not inside either branch. The gate is orthogonal to
  the org-scoping logic.

## Architecture Compliance Rules

- The constant-time comparison MUST use `subtle::ConstantTimeEq::ct_eq` on byte slices —
  not `==`/`!=` on strings or `Option<&str>`. This is the workspace-established pattern
  from W3-FIX-SEC-004.
- The admin-token gate MUST be the first check in `post_reset`/`dtu_reset` — before any
  routing or org-id logic. The threat model is: an unauthenticated caller should not
  reach any reset logic, regardless of which reset variant (scoped or global) they target.
- `subtle = { workspace = true }` is the required form. Do NOT pin a version directly in
  the crate `Cargo.toml` — use the workspace dep that W3-FIX-SEC-004 added to the root.
- Do NOT change the existing reset behavior (semantics, state cleared, response body)
  after the gate passes. The gate adds a 401 short-circuit; it does not alter the
  happy path.
- ThreatIntel uses `dtu_reset` (not `post_reset` as a function name) — apply the same
  logic adapted to the actual function signature and state type.
- All new test names MUST follow `test_<endpoint>_<condition>_<expected_status>` convention
  (e.g., `test_reset_requires_admin_token_missing_returns_401`) consistent with the
  existing cyberint `td_wv0_07_*` test naming.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `subtle` | `{ workspace = true }` (pinned `"2"` in workspace root from W3-FIX-SEC-004) | `ConstantTimeEq` trait for admin token byte comparison |
| `axum` | workspace pin | Handler extractors in DTU admin routes |
| `serde_json` | workspace pin | `json!()` macro in 401 response bodies |
| `tokio` | workspace pin | `#[tokio::test]` in new test files |

No NEW Cargo dependencies. All libraries are already workspace-pinned.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-dtu-cyberint/src/routes/dtu.rs` | Modify | `post_configure` ct_eq (line ~38) + `post_reset` gate (line ~61) |
| `crates/prism-dtu-jira/src/routes/dtu.rs` | Modify | `post_configure` ct_eq (line ~36) + `post_reset` gate (line ~58) |
| `crates/prism-dtu-nvd/src/routes/dtu.rs` | Modify | `post_configure` ct_eq (line ~69) + `post_reset` gate (line ~89) |
| `crates/prism-dtu-pagerduty/src/routes/dtu.rs` | Modify | `post_configure` ct_eq (line ~66) + `post_reset` gate (line ~89) |
| `crates/prism-dtu-threatintel/src/routes/dtu.rs` | Modify | `post_configure` ct_eq (line ~308) + `dtu_reset` gate (line ~27) |
| `crates/prism-dtu-cyberint/Cargo.toml` | Modify | Add `subtle = { workspace = true }` to `[dependencies]` |
| `crates/prism-dtu-jira/Cargo.toml` | Modify | Same |
| `crates/prism-dtu-nvd/Cargo.toml` | Modify | Same |
| `crates/prism-dtu-pagerduty/Cargo.toml` | Modify | Same |
| `crates/prism-dtu-threatintel/Cargo.toml` | Modify | Same |
| `crates/prism-dtu-jira/tests/td_wv0_07_configure_requires_admin_token.rs` | Create | 3+ tests for configure gate (AC-004) |
| `crates/prism-dtu-nvd/tests/td_wv0_07_configure_requires_admin_token.rs` | Create | Same |
| `crates/prism-dtu-pagerduty/tests/td_wv0_07_configure_requires_admin_token.rs` | Create | Same |
| `crates/prism-dtu-threatintel/tests/td_wv0_07_configure_requires_admin_token.rs` | Create | Same |
| `crates/prism-dtu-cyberint/tests/td_wv0_08_reset_requires_admin_token.rs` | Create | 3+ tests for reset gate (AC-003, AC-004) |
| `crates/prism-dtu-jira/tests/td_wv0_08_reset_requires_admin_token.rs` | Create | Same |
| `crates/prism-dtu-nvd/tests/td_wv0_08_reset_requires_admin_token.rs` | Create | Same |
| `crates/prism-dtu-pagerduty/tests/td_wv0_08_reset_requires_admin_token.rs` | Create | Same |
| `crates/prism-dtu-threatintel/tests/td_wv0_08_reset_requires_admin_token.rs` | Create | Same |

## Forbidden Dependencies

- Do NOT add a direct `subtle = "2"` pin in any crate `Cargo.toml` — always use
  `subtle = { workspace = true }`. The workspace root already has the pin.
- Do NOT use `ring`, `openssl`, or `secrecy` for the constant-time comparison — `subtle`
  is the correct minimal dependency per the workspace-established pattern.
- Do NOT modify the post-gate reset logic (the state clearing or response body that
  occurs when the token is valid) — only add the gate before it.
- Do NOT remove or weaken the existing tests that already exist in these 5 crate
  test directories. All additions must be in new test files or appended as new
  functions to existing test files.
- Do NOT apply changes to armis, claroty, crowdstrike, or slack — those 4 DTUs are
  already compliant after W3-FIX-SEC-002 + W3-FIX-SEC-004. Touching them is out of scope.
