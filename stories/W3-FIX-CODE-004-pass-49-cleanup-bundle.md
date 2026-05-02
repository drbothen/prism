---
story_id: W3-FIX-CODE-004
title: "prism-dtu-harness/sensors/config: pass-49 hygiene bundle — CR-010..015, SEC-P2-002/006, BC-3.5.002 timing"
wave: 3.2
level: "L4"
target_module: prism-dtu-harness
subsystems: [SS-01]
priority: P0
depends_on: []
blocks: []
estimated_days: 3
points: 5
status: draft
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-02T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/adversarial-reviews/pass-48.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review-pass2.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review-pass2.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-f-holdout-evaluation.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.5.002-harness-network-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md
  - .factory/specs/behavioral-contracts/BC-3.3.004-customer-config-startup-validation.md
  - .factory/specs/behavioral-contracts/BC-3.1.002-audit-entry-org-fields.md
  - .factory/specs/behavioral-contracts/BC-3.2.001-per-org-sensor-data-isolation.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.5.001
  - BC-3.5.002
  - BC-3.6.001
  - BC-3.3.004
  - BC-3.2.001
verification_properties: [VP-124, VP-125, VP-128, VP-129, VP-130]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.3.004, BC-3.2.001]
anchor_capabilities: [CAP-036]
anchor_subsystem: ["SS-01"]
tdd_mode: strict
parent_finding: "CR-010, CR-011, CR-012/SEC-P2-001, CR-013, CR-014, CR-015, SEC-P2-002, SEC-P2-006, BC-3.5.002 timing fragility"
# BC status: anchored — all BCs fully authored
---

# W3-FIX-CODE-004: pass-49 hygiene bundle — CR-010..015, SEC-P2-002/006, BC-3.5.002 timing

## Narrative

As a Prism maintainer, I want the pass-49 MEDIUM/LOW hygiene findings in
`prism-dtu-harness`, `prism-sensors`, `prism-customer-config`, and `prism-dtu-armis`
remediated in a single bundled story, so that the Wave 3 integration gate proceeds to a
clean convergence verdict without residual stale docs, spurious HTTP calls, auth model
inconsistencies, or CI-fragile timing assertions.

## Objective

Gate Step C pass-2 (`gate-step-c-code-review-pass2.md`) and Gate Step D pass-2
(`gate-step-d-security-review-pass2.md`) surfaced six new MEDIUM/LOW code-review findings
(CR-010 through CR-015) and two security findings (SEC-P2-002, SEC-P2-006) that were not
covered by the Wave 3.1 fix stories. The holdout evaluator additionally identified
BC-3.5.002 timing test fragility (three hard FAILs in `network_isolation_test.rs` under
nextest parallelism) that mirrors the already-handled BC-3.5.001 fragility (#[ignore] in
PR #113).

All nine items are MEDIUM or LOW severity. None is a blocking regression. They are bundled
here rather than filed as individual stories to avoid combinatorial story overhead for
small one-to-five-line fixes.

**Items in scope:**

| ID | Severity | Crate | One-line description |
|----|----------|-------|----------------------|
| CR-010 | MEDIUM | prism-dtu-harness | Module doc at `harness.rs:18-20` still says "calls `handle.abort()`" after CR-002 fix |
| CR-011 | MEDIUM | prism-dtu-harness | `with_failure(FailureMode::None)` inserts sentinel into `HashMap` instead of removing entry |
| CR-012 / SEC-P2-001 | MEDIUM | prism-dtu-armis | Armis `validate_org_id` guard is header-presence-based; inconsistent with Claroty/CrowdStrike instance-identity model |
| CR-013 | MEDIUM | prism-sensors | `fan_out()` has no assertion that `target.org_id == target.spec.org_id` before `adapter.fetch()` |
| CR-014 | LOW | prism-customer-config | `validate_spec_path` is `pub` — should be `pub(crate)` |
| CR-015 | LOW | prism-dtu-cyberint | `validate_org_id` is `#[allow(dead_code)]` without architectural explanation |
| SEC-P2-002 | MEDIUM | prism-customer-config | Pre-join `..` / absolute path checks bypassed when traversal target does not exist |
| SEC-P2-006 | LOW | prism-sensors | No `deny(deprecated)` lint gate on `init_registry` — migration to `init_registry_for_org` not enforced at compile time |
| BC-3.5.002 timing | N/A (fragility) | prism-dtu-harness | 3 network isolation startup-budget tests hard-fail under nextest due to CI wall-clock variance |

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.5.001 | Harness Logical Isolation Invariants | EC-004 (Drop semantics — doc comment must be accurate after CR-002 fix) |
| BC-3.5.002 | Harness Network Isolation Invariants | Postcondition 5 (5s total startup budget); EC-003/EC-004 (startup failure behavior) |
| BC-3.6.001 | Per-Org Failure Injection | Invariant 4 (`FailureMode::None` clears failure — no sentinel entry); Postcondition 2 (other clones unaffected) |
| BC-3.3.004 | Customer Config Startup Validation | CWE-22 path traversal prevention; pre-join checks fire before existence test |
| BC-3.2.001 | Per-Org Sensor Data Isolation via Composite HashMap Key | Precondition 4 (`fan_out` must not dispatch to mismatched OrgId adapter) |

## Acceptance Criteria

### AC-001: CR-010 — Module doc at `harness.rs:18-20` updated (traces to BC-3.5.001 EC-004)
The module-level doc comment at `crates/prism-dtu-harness/src/harness.rs:18-20` no longer
references `handle.abort()`. It accurately describes the post-CR-002 Drop behavior: sends
shutdown signal and drops `JoinHandle`s without abort, relying on axum's
`with_graceful_shutdown` drain. Exact wording per the CR-010 proposed fix:
```
//! `impl Drop for Harness` sends shutdown signals to all non-crashed clones
//! and drops their `JoinHandle`s without abort, honoring the spec-promised
//! graceful-exit window via axum's `with_graceful_shutdown` drain
//! (BC-3.5.001 EC-004; W3-FIX-CODE-001 AC-002/AC-004).
```

### AC-002: CR-011 — `with_failure(FailureMode::None)` removes entry instead of inserting (traces to BC-3.6.001 invariant 4)
In `crates/prism-dtu-harness/src/builder.rs`, when `mode` is `FailureMode::None`:
- The immediate-resolution path calls `existing.initial_failure.remove(&dtu_type)` (not
  `.insert(dtu_type, FailureMode::None)`).
- The deferred `pending` drain in `build()` applies the same remove-on-None logic.
After this fix, `with_failure("org", DtuType::Claroty, FailureMode::None)` on an
already-registered org results in `spec.initial_failure.is_empty() == true` for that org
(assuming no other failure was set), so no HTTP configure call is issued during build.
A new test `test_with_failure_none_removes_entry` in `tests/builder_test.rs` (or
equivalent) verifies: set a failure, then call `with_failure(FailureMode::None)`, then
build — assert no spurious configure call was made and clone returns HTTP 200.

### AC-003: CR-012/SEC-P2-001 — Armis `validate_org_id` guard uses instance-identity model (traces to BC-3.5.002 precondition 3)
In `crates/prism-dtu-armis/src/routes/devices.rs`, the header-presence conditional
`if headers.get("x-org-id").is_some()` is replaced with the instance-identity guard
`if state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID`. This is applied
to all affected handlers (`get_or_post_devices`, `post_devices`, and tag endpoints). After
this change, a real-org Armis clone (constructed with a customer OrgId) rejects requests
that omit the `X-Org-Id` header with HTTP 401, matching the Claroty/CrowdStrike behavior.
The auth model table in the crate's `README` or module doc is updated to reflect the
chosen model (instance-identity, not header-presence).

### AC-004: CR-013 — `fan_out()` asserts `target.org_id == target.spec.org_id` (traces to BC-3.2.001 precondition 4)
In `crates/prism-sensors/src/fanout.rs`, immediately before `adapter.fetch(&target.spec, ...)`
a `debug_assert_eq!` is added:
```rust
debug_assert_eq!(
    target.org_id, target.spec.org_id,
    "fan_out precondition violation: target.org_id ({}) != target.spec.org_id ({}) — \
     callers must set spec.org_id = target.org_id (BC-3.2.001 precondition 4)",
    target.org_id, target.spec.org_id
);
```
`debug_assert_eq!` fires in debug builds (CI) and is a no-op in release builds.
No production performance impact. The assertion message includes both UUIDs for
diagnostic clarity.

### AC-005: CR-014 — `validate_spec_path` narrowed to `pub(crate)` (traces to BC-3.3.004 invariant)
`crates/prism-customer-config/src/validator.rs:625` changes from
`pub fn validate_spec_path` to `pub(crate) fn validate_spec_path`. No external crate
uses this function directly (confirmed by `git grep` showing only internal callers).
The change does not break any test.

### AC-006: CR-015 — Cyberint `validate_org_id` either removed or documented (traces to BC-3.5.002 precondition 3)
Option A (preferred): Remove `validate_org_id` from
`crates/prism-dtu-cyberint/src/routes/alerts.rs`. Add a module-level comment explaining
that Cyberint uses session-based multi-org routing rather than instance-identity
enforcement, and that this is the intentional deviation from the other three DTUs.
Remove the `#[allow(dead_code)]` attribute along with the function.
Option B (acceptable if Option A creates test friction): Retain the function, remove
`#[allow(dead_code)]`, add `// TODO(S-3.x): wire into single-tenant mode path` comment.

### AC-007: SEC-P2-002 — Pre-join traversal checks fire before `resolved.exists()` (traces to BC-3.3.004 CWE-22 invariant)
In `crates/prism-customer-config/src/validator.rs`, the I/O-free pre-join checks
(`..` component scan and absolute path rejection) are moved OUTSIDE the
`if resolved.exists()` gate in `validate_dtu_block`. The ordering after the fix:
1. Pre-join: reject `..` components (no filesystem I/O).
2. Pre-join: reject absolute paths (no filesystem I/O).
3. Post-join: `resolved.exists()` check.
4. Post-join: `canonicalize()` + prefix comparison.
A traversal attempt targeting a non-existent path (e.g., `"../../../../etc/nonexistent"`)
now emits `E-CFG-018` (`SpecPathTraversal`) rather than `E-CFG-015` (`SpecFileNotFound`),
and the attempt is captured in the audit log. A new test
`test_traversal_nonexistent_target_still_logs_E_CFG_018` in `tests/path_traversal.rs`
verifies this.

### AC-008: SEC-P2-006 — `#[deny(deprecated)]` lint added to `prism-sensors` (traces to BC-3.2.001 invariant 1)
A crate-level `#![deny(deprecated)]` attribute (or `#![deny(deprecated_in_future)]` if
the former breaks the `#[allow(deprecated)]` test site) is added to
`crates/prism-sensors/src/lib.rs`. The one existing call site that uses `init_registry`
under `#[allow(deprecated)]` in `tests/test_armis.rs` must also carry its own
`#[allow(deprecated)]` annotation to suppress the deny-level lint at that scope —
this is already present per the security review's positive observation. CI must now
fail on any new caller that uses `init_registry` without `#[allow(deprecated)]`.

### AC-009: BC-3.5.002 timing — three startup-budget tests in `network_isolation_test.rs` marked `#[ignore]` (traces to BC-3.5.002 postcondition 5)
The three test functions in
`crates/prism-dtu-harness/tests/network_isolation_test.rs` that assert the 5-second
startup wall-clock budget (`test_BC_3_5_002_*`) are annotated with `#[ignore]` and a
`// TD-W3-TIMING-001: wall-clock startup budget; see also BC-3.5.001 #[ignore] in PR #113`
comment. This matches the existing treatment of the analogous BC-3.5.001 timing tests.
The tests remain in the file so that they can be run explicitly on dedicated hardware
(`cargo nextest run ... --ignored`); they are simply suppressed from default CI runs.

## Tasks

### Part A: Doc and builder fixes (CR-010, CR-011)

1. Read `crates/prism-dtu-harness/src/harness.rs` lines 15-25 — locate the stale module
   doc comment.
2. Replace lines 18-20 with the accurate post-CR-002 description (AC-001 exact wording).
3. Read `crates/prism-dtu-harness/src/builder.rs` lines 233-246 (`with_failure`) and
   the `build()` pending-drain section.
4. Apply the `remove`-on-`FailureMode::None` fix to both the immediate-resolution path
   and the deferred drain path (AC-002).
5. Write `test_with_failure_none_removes_entry` to verify no spurious configure call.

### Part B: Armis auth model alignment (CR-012 / SEC-P2-001)

6. Read `crates/prism-dtu-armis/src/routes/devices.rs` lines 60-105 to find all
   header-presence guards.
7. Read `crates/prism-dtu-armis/src/state.rs` to confirm `DTU_DEFAULT_INSTANCE_ORG_ID`
   constant name and value.
8. Replace `if headers.get("x-org-id").is_some()` guards with
   `if state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID` across all
   Armis route handlers.
9. Update the crate module doc or auth model table to document the change.
10. Run `cargo test -p prism-dtu-armis` — all existing tests pass; the multi-tenant
    test for missing-header-returns-401 now passes for real-org clones.

### Part C: fan_out assertion (CR-013)

11. Read `crates/prism-sensors/src/fanout.rs` lines 338-380.
12. Insert `debug_assert_eq!` immediately before `adapter.fetch(...)` (AC-004).
13. Run `cargo test -p prism-sensors` — all tests pass, assert not triggered.

### Part D: Visibility and dead-code cleanup (CR-014, CR-015)

14. Change `pub fn validate_spec_path` → `pub(crate) fn validate_spec_path` in
    `crates/prism-customer-config/src/validator.rs` (AC-005).
15. Apply Option A for CR-015: remove `validate_org_id` from
    `crates/prism-dtu-cyberint/src/routes/alerts.rs`, add module doc comment explaining
    session-routing model (AC-006).

### Part E: Path traversal pre-join gate (SEC-P2-002)

16. Read `crates/prism-customer-config/src/validator.rs` lines 540-580 (the
    `validate_dtu_block` function).
17. Move `..` component rejection and absolute-path rejection BEFORE the
    `if resolved.exists()` block (AC-007).
18. Write `test_traversal_nonexistent_target_still_logs_E_CFG_018` in
    `tests/path_traversal.rs` (AC-007).

### Part F: Deprecation lint + timing fragility (SEC-P2-006, BC-3.5.002)

19. Add `#![deny(deprecated)]` to `crates/prism-sensors/src/lib.rs` (AC-008).
    Verify the existing `#[allow(deprecated)]` in `tests/test_armis.rs` suppresses it
    correctly; fix any new compile errors (there should be none per the pass-2 assessment).
20. Read `crates/prism-dtu-harness/tests/network_isolation_test.rs` — locate the three
    `test_BC_3_5_002_*` startup-budget test functions.
21. Annotate each with `#[ignore]` + TD-W3-TIMING-001 comment (AC-009).

### Part G: Integration

22. Run `cargo test -p prism-dtu-harness -p prism-sensors -p prism-customer-config
    -p prism-dtu-armis -p prism-dtu-cyberint --all-features` — all tests pass.
23. Run `cargo clippy --workspace -- -D warnings` — no new clippy warnings.
24. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| Harness module doc | prism-dtu-harness | `crates/prism-dtu-harness/src/harness.rs:18-20` | Pure (comment-only) |
| `HarnessBuilder::with_failure` | prism-dtu-harness | `crates/prism-dtu-harness/src/builder.rs` | Pure (builder method) |
| `validate_org_id` guard — Armis | prism-dtu-armis | `crates/prism-dtu-armis/src/routes/devices.rs` | Pure (header validation) |
| `fan_out` precondition assertion | prism-sensors | `crates/prism-sensors/src/fanout.rs` | Pure (debug_assert) |
| `validate_spec_path` visibility | prism-customer-config | `crates/prism-customer-config/src/validator.rs` | Pure (visibility change) |
| Cyberint `validate_org_id` removal | prism-dtu-cyberint | `crates/prism-dtu-cyberint/src/routes/alerts.rs` | Pure (dead code removal) |
| `validate_dtu_block` pre-join order | prism-customer-config | `crates/prism-customer-config/src/validator.rs` | Pure (reordering I/O-free checks) |
| `#![deny(deprecated)]` lint | prism-sensors | `crates/prism-sensors/src/lib.rs` | Pure (compile-time gate) |
| BC-3.5.002 timing `#[ignore]` | prism-dtu-harness | `crates/prism-dtu-harness/tests/network_isolation_test.rs` | Pure (test attribute) |

**Subsystem anchor justification:** SS-01 (Sensor Adapters) owns this story's scope
because the majority of changes are in `prism-dtu-harness`, `prism-dtu-armis`,
`prism-dtu-cyberint`, and `prism-sensors` — all of which are Sensor Adapter subsystem
crates per the ARCH-INDEX Subsystem Registry. The two `prism-customer-config` changes
(CR-014, SEC-P2-002) are minor and co-ride this story for bundling efficiency.

**Dependency anchor justification:** `depends_on: []` — all nine items are
self-contained; none requires another W3.2 story to land first. Specifically: CR-011
depends on the CR-001 `HashMap<DtuType, FailureMode>` change (W3-FIX-CODE-001 PR #116,
already merged), not on W3-FIX-CREDS-001. `blocks: []` — no downstream story requires
any of these fixes as a precondition.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `harness.rs:18-20` doc comment | pure-core | Comment text change only; no runtime effect |
| `builder.rs with_failure` None path | pure-core | Builder method; no I/O; `HashMap::remove` is a pure data mutation |
| `devices.rs` Armis guard | pure-core | Conditional expression change; validation returns `Result`; no I/O |
| `fanout.rs debug_assert_eq!` | pure-core | `debug_assert_eq!` is a no-op in release; no I/O in debug |
| `validator.rs pub(crate)` change | pure-core | Visibility annotation; no runtime behavior change |
| `alerts.rs` dead-code removal | pure-core | Removing unreachable function; no behavior change |
| `validator.rs` pre-join reordering | pure-core | Reorders I/O-free checks before I/O-bearing check; no new I/O |
| `lib.rs #![deny(deprecated)]` | pure-core | Compile-time attribute; no runtime effect |
| `network_isolation_test.rs #[ignore]` | effectful-shell | Tests were effectful (spawn HTTP); `#[ignore]` suppresses them in CI |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `with_failure(None)` on org with no prior failure set | `HashMap::remove` on absent key is a no-op; `is_empty()` remains `true`; no configure call |
| EC-002 | `with_failure(None)` on deferred (not yet registered) slug | `pending_failures` entry should be dropped or marked None; resolution in `build()` does no insert |
| EC-003 | Armis clone constructed with `DTU_DEFAULT_INSTANCE_ORG_ID` (legacy single-tenant) | Guard condition `instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID` is `false`; request proceeds as before (backward compat) |
| EC-004 | `fan_out` called with `target.spec.org_id = Uuid::nil()` (serde default) while `target.org_id` is real | `debug_assert_eq!` fires in debug builds with diagnostic message; release builds proceed to `adapter.fetch()` where `OrgIdMismatch` guard fires |
| EC-005 | `validate_spec_path` called from a test that used it as `pub` | Will become a compile error after the `pub(crate)` change; fix any test that directly calls it (expected: none per `git grep`) |
| EC-006 | Non-existent traversal path `"../../../../etc/nonexistent"` | `SpecPathTraversal` (E-CFG-018) emitted BEFORE `SpecFileNotFound` (E-CFG-015); audit trail captures the attempt |
| EC-007 | `init_registry` called without `#[allow(deprecated)]` after lint addition | Compile error (E0658 / deprecated warning promoted to error); the developer must migrate to `init_registry_for_org` |
| EC-008 | BC-3.5.002 timing tests run explicitly with `--ignored` | Tests execute and may pass on dedicated low-load hardware; failures are acceptable in this mode |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~5 000 |
| BC files (5 BCs) | ~7 000 |
| `harness.rs` (lines 15-25 + Drop impl ~40 lines) | ~500 |
| `builder.rs` (`with_failure` + pending drain, ~80 lines) | ~700 |
| `prism-dtu-armis/src/routes/devices.rs` (lines 55-115) | ~700 |
| `prism-dtu-armis/src/state.rs` (DTU_DEFAULT_INSTANCE_ORG_ID) | ~300 |
| `prism-sensors/src/fanout.rs` (lines 330-385) | ~600 |
| `prism-customer-config/src/validator.rs` (lines 530-640) | ~1 200 |
| `prism-dtu-cyberint/src/routes/alerts.rs` (lines 80-110) | ~400 |
| `prism-sensors/src/lib.rs` (top of file for lint attr) | ~300 |
| `tests/network_isolation_test.rs` (three test fn signatures) | ~400 |
| `tests/path_traversal.rs` (existing 7 tests + new test) | ~1 200 |
| `cargo test` + `cargo clippy` output | ~1 000 |
| **Total** | **~19 300** |

Fits in a single agent context window. If the validator file is large, load only the
`validate_dtu_block` function (SEC-P2-002 section) rather than the entire file.

## Previous Story Intelligence

- **W3-FIX-CODE-001** (PR #116): fixed CR-001/CR-002 (HashMap failure scoping +
  Drop abort removal). CR-010 is a doc residue from that same fix — the `fn drop` doc
  was updated but the module-level doc at line 18-20 was missed. CR-011 is the corollary:
  now that `initial_failure` is a `HashMap`, inserting `FailureMode::None` should be
  a `remove`, not an `insert`.
- **W3-FIX-SEC-001** (PR #113): wired `validate_org_id` into all four DTU clones.
  CR-012/SEC-P2-001 is the Armis inconsistency that was accepted as backward-compat at
  the time but has since been identified as weaker defense-in-depth. CR-015 is the
  Cyberint `#[allow(dead_code)]` residue from the same PR.
- **W3-FIX-SEC-003** (PR #114): implemented `validate_spec_path`. SEC-P2-002 is a
  known gap that was explicitly acknowledged in the PR but deferred — the pre-join
  checks were gated behind `exists()`. CR-014 (`validate_spec_path` visibility) is a
  separate residue from the same PR.
- **S-3.1.06-ImplPhase** (PR #117): added `AdapterRegistry` + OrgId-keyed dispatch.
  CR-013 (`fan_out` dual-identity gap) and SEC-P2-006 (`init_registry` deprecation
  enforcement) both arise from this PR.
- **PR #113 BC-3.5.001 timing**: the `#[ignore]` pattern for wall-clock startup tests
  was already established; BC-3.5.002 timing gets the same treatment here.
- **Lesson:** Each fix wave should include a one-pass sweep for: (1) module docs that
  reference removed behavior, (2) `HashMap` insert/remove semantic mismatches,
  (3) `#[allow(dead_code)]` without explanatory comments, (4) `pub` visibility on
  helpers that are only used internally, and (5) `exists()` guards that bypass
  I/O-free validation steps.

## Architecture Compliance Rules

- The `debug_assert_eq!` in `fan_out` MUST use `debug_assert_eq!` (not `assert_eq!`).
  A production fanout path must not panic on mismatched OrgIds — the adapter's own
  `OrgIdMismatch` guard is the fail-safe. The assertion is diagnostic only.
- The Armis instance-identity guard MUST compare against
  `crate::state::DTU_DEFAULT_INSTANCE_ORG_ID`, NOT against `OrgId::from_uuid(Uuid::nil())`.
  Armis uses a non-nil default sentinel (`0...AA`) — using nil would break legacy
  single-tenant clones.
- The pre-join path traversal checks in `validate_dtu_block` MUST remain I/O-free after
  the reorder. Do NOT introduce any filesystem call before `resolved.exists()`.
- `validate_spec_path` visibility change MUST NOT break any integration test. Run
  `git grep "validate_spec_path"` to confirm no external test references it directly
  before committing.
- The `#![deny(deprecated)]` attribute in `prism-sensors/src/lib.rs` MUST be placed at
  the top of the file (crate-level inner attribute). Do NOT use `#[deny(deprecated)]`
  (outer attribute form) which would apply only to the next item.
- Cyberint session-routing doc comment MUST explain WHY the crate deviates from the
  three-DTU instance-identity model. A comment that merely says "uses session routing"
  without explaining the implication (missing header falls through to instance session,
  not 401) leaves the next developer confused.

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| `axum` | workspace pin | Handler extractors (Armis guard change) |
| `std::collections::HashMap` | std | `HashMap::remove` in `with_failure` fix |
| `tokio` | workspace pin | Async harness tests |

No new Cargo dependencies introduced by this story.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-dtu-harness/src/harness.rs` | Modify | Lines 18-20: update module doc (CR-010) |
| `crates/prism-dtu-harness/src/builder.rs` | Modify | `with_failure` None path: `remove` not `insert`; deferred drain path same fix (CR-011) |
| `crates/prism-dtu-harness/tests/builder_test.rs` | Modify or Create | New test: `test_with_failure_none_removes_entry` (AC-002) |
| `crates/prism-dtu-armis/src/routes/devices.rs` | Modify | Replace header-presence guards with instance-identity guards (CR-012 / AC-003) |
| `crates/prism-sensors/src/fanout.rs` | Modify | Add `debug_assert_eq!` before `adapter.fetch()` (CR-013 / AC-004) |
| `crates/prism-customer-config/src/validator.rs` | Modify | `pub(crate)` visibility (CR-014 / AC-005); reorder pre-join checks (SEC-P2-002 / AC-007) |
| `crates/prism-customer-config/tests/path_traversal.rs` | Modify | Add `test_traversal_nonexistent_target_still_logs_E_CFG_018` (AC-007) |
| `crates/prism-dtu-cyberint/src/routes/alerts.rs` | Modify | Remove `validate_org_id` + add session-routing doc (CR-015 / AC-006) |
| `crates/prism-sensors/src/lib.rs` | Modify | Add `#![deny(deprecated)]` crate attribute (SEC-P2-006 / AC-008) |
| `crates/prism-dtu-harness/tests/network_isolation_test.rs` | Modify | Add `#[ignore]` + TD-W3-TIMING-001 comment to 3 test fns (AC-009) |

## Forbidden Dependencies

- Do NOT add any new external crate dependencies to any crate modified by this story.
- Do NOT change the public API of `HarnessBuilder::with_failure` — argument types
  `(slug: &str, dtu_type: DtuType, mode: FailureMode)` are unchanged.
- Do NOT remove the `validate_spec_path` function — only change its visibility to
  `pub(crate)`.
- Do NOT replace the `debug_assert_eq!` with a runtime `Result`-returning check —
  that would change `fan_out`'s function signature and is a bigger refactor than this
  story's scope.
