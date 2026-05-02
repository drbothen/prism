# Review Findings — W3-FIX-CODE-001

| Field | Value |
|-------|-------|
| Story | W3-FIX-CODE-001 |
| PR | #116 |
| Status | CONVERGED — APPROVE (cycle 1) |
| Reviewer | pr-review-triage (step 5, cycle 1) |
| Reviewed at | 2026-05-01 |
| Verdict | APPROVE |

## Convergence Tracking

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 | 2 | 0 | 0 | 0 (both suggestions, non-blocking) |

**CONVERGED in 1 cycle — 0 blocking findings.**

## AC Satisfaction Matrix

| AC | Title | Status | Evidence |
|----|-------|--------|----------|
| AC-001 | with_failure honors DtuType — injection scoped to specified clone | PASS | HashMap insert in both immediate + deferred paths; Phase 4 loop iterates map |
| AC-002 | DtuType key stored in CustomerSpec failure map | PASS | `initial_failure: HashMap<DtuType, FailureMode>` in types.rs; initialized as `HashMap::new()` |
| AC-003 | Deferred failure path honors DtuType | PASS | `std::mem::take` drain + per-entry insert in build() |
| AC-004 | Drop implements 5-second graceful window | PASS | handle.abort() removed; axum with_graceful_shutdown wired; JoinHandles dropped (tasks detach) |
| AC-005 | No regression in existing failure injection tests | PASS | 19/19 builder_ergonomics_test pass |
| AC-006 | No regression in existing drop/teardown tests | PASS | 34/34 logical_isolation_test pass including test_BC_3_5_001_drop_releases_ports |

## Findings

### S-001 (SUGGESTION): Drop doc comment "waits up to 5s" is still imprecise

- **Severity:** SUGGESTION (non-blocking)
- **Location:** `crates/prism-dtu-harness/src/harness.rs:349-387`
- **Description:** The doc comment says the Drop impl "honors the 5-second grace window" and references "waits up to 5s for graceful exit." The implementation sends the shutdown signal then drops the JoinHandles — the tasks continue running detached. The Drop itself returns immediately (synchronous context). The 5-second behavioral contract is upheld by axum's graceful drain, but the Drop does not actively wait for 5 seconds. A future maintainer reading "waits up to 5s" may expect a blocking wait that does not occur.
- **Proposed fix (non-blocking):** Update the doc comment to "Sends shutdown signal to all clones and drops their JoinHandles. Tasks continue running detached and drain gracefully via axum::with_graceful_shutdown. No hard abort is issued; Drop returns immediately."
- **Impact:** None on correctness. AC-004 behavioral contract is satisfied.

### S-002 (SUGGESTION): `FailureMode::None` inserted into HashMap rather than removed

- **Severity:** SUGGESTION (non-blocking)
- **Location:** `crates/prism-dtu-harness/src/builder.rs:240` (with_failure immediate path)
- **Description:** The doc comment says "Passing FailureMode::None clears any previously set initial_failure." The implementation does `existing.initial_failure.insert(dtu_type, mode)` unconditionally — inserting `FailureMode::None` rather than calling `initial_failure.remove(&dtu_type)`. The behavior is functionally correct (inject_failure with FailureMode::None sends `{"clear": true}` to the admin endpoint, equivalent to clear_failure). However, storing FailureMode::None in the map is semantically inconsistent: if a caller checks `spec.initial_failure.contains_key(&DtuType::Claroty)` after calling `with_failure("slug", DtuType::Claroty, FailureMode::None)`, they will find an entry, which contradicts "cleared."
- **Proposed fix (non-blocking):** In with_failure, check `if matches!(mode, FailureMode::None) { existing.initial_failure.remove(&dtu_type); } else { existing.initial_failure.insert(dtu_type, mode); }`. Same in the deferred resolution path.
- **Impact:** None on runtime correctness for current consumers. All current tests pass. Could cause confusion for future test authors inspecting the spec directly.

## Test Evidence

```
running 19 tests (builder_ergonomics_test)
test result: ok. 19 passed; 0 failed

running 2 tests (failure_scope_test)
test result: ok. 2 passed; 0 failed

running 34 tests (logical_isolation_test)
test result: ok. 34 passed; 0 failed

running 16 tests (network_isolation_test)
test result: ok. 16 passed; 0 failed

running 1 test (network_isolation_timeout_test)
test result: ok. 1 passed; 0 failed

Total: 72 passed, 0 failed
```

## Verdict

**APPROVE** — All 6 ACs satisfied. 72/72 tests pass. 0 blocking findings. 2 non-blocking suggestions for future cleanup (S-001 doc precision, S-002 FailureMode::None map semantics). Ready to merge.
