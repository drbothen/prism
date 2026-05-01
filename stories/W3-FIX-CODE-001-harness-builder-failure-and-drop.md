---
story_id: W3-FIX-CODE-001
title: "prism-dtu-harness: per-DtuType failure scoping and honest Drop semantics"
wave: 3.1
level: "L4"
target_module: prism-dtu-harness
subsystems: [SS-01]
priority: P0
depends_on: []
blocks: []
estimated_days: 2
points: 5
status: draft
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-01T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.5.002-harness-network-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md
  - .factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.5.001
  - BC-3.5.002
  - BC-3.6.001
verification_properties: [VP-124, VP-128, VP-129, VP-130]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.5.001, BC-3.5.002, BC-3.6.001]
anchor_capabilities: [CAP-036]
anchor_subsystem: ["SS-01"]
tdd_mode: strict
---

# W3-FIX-CODE-001: prism-dtu-harness — per-DtuType failure scoping and honest Drop semantics

## Narrative

As a Prism test-infrastructure maintainer, I want `HarnessBuilder::with_failure` to
honor its `DtuType` argument (injecting failure only into the specified sensor type
for the specified org), and I want `Harness::Drop` to either implement the documented
5-second graceful shutdown or tell the truth about immediate abort, so that test authors
can rely on the harness API matching its specification.

## Objective

Gate Step C identified two HIGH findings in `prism-dtu-harness`:

**CR-001:** `with_failure(slug, DtuType, mode)` stores the failure in
`CustomerSpec.initial_failure: Option<FailureMode>` — a single field covering ALL
DtuTypes for the org. The `DtuType` argument is silently discarded on the
immediate-resolution path; on the deferred path it is stored in `pending_failures`
but not propagated. In `build()` Phase 4, the failure is injected for every `dtu_type
in &spec.dtu_types`. This violates BC-3.6.001 postcondition 2 (other clones must
return normal responses).

**CR-002:** `Harness::Drop` sends shutdown signals then immediately calls
`handle.abort()`. The doc comment at `harness.rs:19` claims "waits up to 5s for
graceful exit" — this is false. In an async runtime the abort is immediate; no grace
period is given to in-flight requests.

Both defects have observable consequences in multi-DtuType harness tests and in tests
that verify request completion under drop.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.6.001 | Per-Org Failure Injection | Postcondition 1 (injected failure applies to target clone), Postcondition 2 (other clones return normal responses), Invariant 1 (failure state scoped strictly to target) |
| BC-3.5.001 | Harness Logical Isolation Invariants | EC-004: "waits up to 5s for graceful exit, then calls handle.abort()" |
| BC-3.5.002 | Harness Network Isolation Invariants | EC-004: same graceful-exit contract applies in network mode |

## Acceptance Criteria

### AC-001: with_failure honors DtuType — injection scoped to specified clone (traces to BC-3.6.001 postcondition 2)
`with_failure("acme", DtuType::Claroty, FailureMode::AuthReject)` injects failure ONLY
into the Claroty clone for org `acme`. The Armis, CrowdStrike, and Cyberint clones for
`acme` return normal HTTP 200 responses. Verified by `test_failure_scope_per_dtu_type`
in `tests/failure_scope_test.rs`.

### AC-002: DtuType key is stored in CustomerSpec failure map (traces to BC-3.6.001 invariant 1)
`CustomerSpec.initial_failure` is changed from `Option<FailureMode>` to
`HashMap<DtuType, FailureMode>`. `with_failure` inserts only the specified `(DtuType, FailureMode)`.
Build Phase 4 iterates the map and calls `inject_failure` only for entries present in
the map — not for all `spec.dtu_types`.

### AC-003: Deferred failure path also honors DtuType (traces to BC-3.6.001 postcondition 1)
When `with_failure` is called before `with_customer` registers the org (deferred path),
the `pending_failures` entry is resolved with both `slug` and `DtuType`. When the org
is registered and `build()` runs, only the specified `DtuType` is injected.

### AC-004: Drop implements 5-second graceful window (traces to BC-3.5.001 EC-004)
**Preferred resolution:** The `Drop` impl sends the shutdown signal and does NOT
immediately call `handle.abort()`. `axum::Server::with_graceful_shutdown` already
handles orderly drain when the shutdown future fires. Removing the abort calls is
sufficient; in-flight requests complete within Tokio's cooperative scheduling window.
The doc comment "waits up to 5s" becomes accurate with this approach.

**Alternative (if preferred resolution is infeasible):** If immediate abort is kept,
the doc comment MUST be changed to accurately state:
"Sends shutdown signal to all clones, then immediately aborts their Tokio tasks.
In-flight requests are not given a grace period. This is a known limitation in
synchronous Drop contexts where `.await` is unavailable (BC-3.5.001 EC-004)."
The test at `tests/logical_isolation_test.rs:890` that gates drop with a 5s timeout
must be updated to match whichever semantics are chosen.

### AC-005: No regression in existing failure injection tests (traces to BC-3.6.001 postcondition 3)
All existing tests that use `inject_failure` and `clear_failure` on a single DtuType
continue to pass after the `HashMap` refactor.

### AC-006: No regression in existing drop/teardown tests (traces to BC-3.5.001 postcondition 4)
`test_BC_3_5_001_drop_releases_ports` and all other drop-related tests pass after the
Drop semantics change.

## Tasks

### Part A: Fix CR-001 (with_failure DtuType scoping)

1. Read `crates/prism-dtu-harness/src/builder.rs` lines 233-251 (`with_failure`) and
   lines 483-488 (Phase 4 failure injection loop) in full context.
2. Read `crates/prism-dtu-harness/src/types.rs` lines 188-206 (`CustomerSpec`) to find
   the `initial_failure` field.
3. Change `CustomerSpec.initial_failure: Option<FailureMode>` to
   `initial_failure: HashMap<DtuType, FailureMode>` (initialize as empty `HashMap::new()`).
4. Update `with_failure` (immediate-resolution path): instead of
   `existing.initial_failure = Some(mode)`, insert
   `existing.initial_failure.insert(dtu_type, mode)` (or remove if `FailureMode::None`).
5. Update `with_failure` (deferred path at `pending_failures`): verify the `DtuType`
   is preserved through the resolution loop at line ~296.
6. Update the Phase 4 failure injection loop in `build()`: iterate
   `spec.initial_failure.iter()` as `(dtu_type, mode)` rather than iterating
   `spec.dtu_types` with a single `initial_failure`.
7. Fix any other places in `builder.rs` that read `spec.initial_failure` as an
   `Option` (compile errors will surface these).

### Part B: Fix CR-002 (Drop graceful shutdown)

8. Read `crates/prism-dtu-harness/src/harness.rs` lines 348-370 (`Drop` impl) and
   line 19 (doc comment).
9. Determine which resolution to apply (see AC-004). If the preferred resolution is
   chosen: remove the `handle.abort()` calls from `Drop`. Verify that
   `axum::Server::with_graceful_shutdown` was wired during `build()` — if not, wire it.
10. Update the doc comment at `harness.rs:19` to accurately describe the chosen semantics.
11. Update `tests/logical_isolation_test.rs:890` to match the chosen semantics.

### Part C: Tests

12. Create `crates/prism-dtu-harness/tests/failure_scope_test.rs` with
    `test_failure_scope_per_dtu_type` (AC-001): build a two-DtuType harness for one org,
    inject failure into one type, assert the other returns 200.
13. Run `cargo test -p prism-dtu-harness --features dtu` — all tests pass.
14. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| `CustomerSpec.initial_failure` | prism-dtu-harness | `crates/prism-dtu-harness/src/types.rs` | Pure (data type) |
| `HarnessBuilder::with_failure` | prism-dtu-harness | `crates/prism-dtu-harness/src/builder.rs` | Pure (builder method, no I/O) |
| `build()` Phase 4 injection loop | prism-dtu-harness | `crates/prism-dtu-harness/src/builder.rs` | Effectful (HTTP to clone admin endpoint) |
| `Harness::Drop` | prism-dtu-harness | `crates/prism-dtu-harness/src/harness.rs` | Effectful (sends to tokio task handles) |
| `failure_scope_test.rs` | prism-dtu-harness | `crates/prism-dtu-harness/tests/failure_scope_test.rs` | Effectful (spawns HTTP clones) |

**Subsystem anchor justification:** SS-01 (Sensor Adapters) owns this story's scope
because `prism-dtu-harness` is the DTU test harness for sensor adapter clones, per the
ARCH-INDEX Subsystem Registry definition of SS-01 which includes test infrastructure
for the sensor adapter subsystem.

**Dependency anchor justification:** `depends_on: []` — `builder.rs` and `harness.rs`
do not depend on the SEC-001/SEC-002 route-level fixes. `blocks: []` — no other
W3-FIX story requires per-DtuType failure scoping to be complete first.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `with_failure` called twice on same (slug, DtuType) with different modes | Second call overwrites first; `HashMap::insert` semantics — last write wins |
| EC-002 | `with_failure(slug, DtuType::Claroty, FailureMode::None)` explicitly clears | `HashMap::remove(DtuType::Claroty)` or insert `None`-equivalent; resulting in no failure for Claroty; same as `clear_failure` semantics (BC-3.6.001 invariant 4) |
| EC-003 | Org has three DtuTypes; failure injected for two of them | Only the two specified DtuTypes have failures; third returns normal 200 |
| EC-004 | Drop called while a 200ms slow-response injection is active | With preferred AC-004 resolution (no immediate abort): request completes. With alternative (immediate abort): request is aborted mid-flight. Either behavior must be documented in the doc comment. |
| EC-005 | `HashMap<DtuType, FailureMode>` for a new org starts empty | `CustomerSpec::new()` (or wherever it is constructed) must initialize `initial_failure` as `HashMap::new()`, not `None` |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `CustomerSpec.initial_failure` field | pure-core | Data structure change; no I/O |
| `HarnessBuilder::with_failure` | pure-core | Builder pattern; no I/O; returns `Self` |
| Build Phase 4 injection loop | effectful-shell | Makes HTTP POST to clone admin endpoint |
| `Harness::Drop` | effectful-shell | Sends to `broadcast::Sender`; calls `JoinHandle::abort` (or removes it) |
| `failure_scope_test.rs` | effectful-shell | Spawns real HTTP clones via harness |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3 500 |
| BC files (3 BCs) | ~5 000 |
| `builder.rs` (full file, ~550 lines) | ~4 000 |
| `types.rs` (full file, ~250 lines) | ~1 800 |
| `harness.rs` (relevant sections, ~80 lines) | ~600 |
| `logical_isolation_test.rs` (relevant sections, ~100 lines) | ~800 |
| New `failure_scope_test.rs` | ~600 |
| Cargo output + clippy | ~1 000 |
| **Total** | **~17 300** |

Fits in a single agent context window. If `builder.rs` is larger than estimated, load
only the `with_failure`, `build` Phase 4, and pending-failures resolution sections.

## Previous Story Intelligence

- **S-3.3.05** (harness builder ergonomics) introduced `with_failure` and the
  `CustomerSpec.initial_failure: Option<FailureMode>` design. The `Option<FailureMode>`
  design was a first-pass shortcut that assumed a single failure per org; the correct
  design is `HashMap<DtuType, FailureMode>`.
- **S-3.6.01, S-3.6.02** consumed `with_failure` in tests — review those test files
  to understand what existing call sites look like; the refactored API must not break
  them (AC-005).
- **Lesson:** When a public API takes a type argument but does not use it in the stored
  state type, that is a strong signal that the state type is wrong. `with_failure(slug,
  DtuType, mode)` → `initial_failure: Option<FailureMode>` was a type-level mismatch
  from the beginning.

## Architecture Compliance Rules

- `HashMap<DtuType, FailureMode>` is the replacement for `Option<FailureMode>`. Do NOT
  use `Vec<(DtuType, FailureMode)>` — `HashMap` provides O(1) lookup and natural dedup
  semantics.
- `DtuType` must implement `Hash + Eq` for use as a `HashMap` key — verify this is
  already derived on the enum; if not, add `#[derive(Hash, Eq)]`.
- The Phase 4 injection loop MUST iterate only the entries in `initial_failure` map,
  NOT all of `spec.dtu_types`. This is the core behavioral fix.
- Doc comment at `harness.rs:19` MUST accurately describe actual drop behavior after
  the fix. False doc comments are a contract violation.
- The 5-second grace period assertion at `tests/logical_isolation_test.rs:890` MUST
  be consistent with whichever drop semantics are implemented.

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| std::collections::HashMap | std | `initial_failure` field type |
| tokio (workspace) | workspace pin | `JoinHandle` await in Drop (if preferred resolution) |
| axum (workspace) | workspace pin | `with_graceful_shutdown` (if preferred resolution) |

No new external Cargo dependencies.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-dtu-harness/src/types.rs` | Modify | Change `initial_failure: Option<FailureMode>` → `HashMap<DtuType, FailureMode>` |
| `crates/prism-dtu-harness/src/builder.rs` | Modify | Fix `with_failure` and Phase 4 injection loop |
| `crates/prism-dtu-harness/src/harness.rs` | Modify | Fix Drop + update doc comment |
| `crates/prism-dtu-harness/tests/logical_isolation_test.rs` | Modify | Update drop semantics test (line ~890) if needed |
| `crates/prism-dtu-harness/tests/failure_scope_test.rs` | Create | New: per-DtuType failure scope regression test |

## Forbidden Dependencies

- Do NOT add any new external crate dependencies to `prism-dtu-harness`.
- Do NOT change the public signature of `HarnessBuilder::with_failure` — the argument
  types `(slug: &str, dtu_type: DtuType, mode: FailureMode)` remain the same; only
  the internal storage changes.
