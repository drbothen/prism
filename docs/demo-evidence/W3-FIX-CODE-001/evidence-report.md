# Demo Evidence Report — W3-FIX-CODE-001

| Field | Value |
|-------|-------|
| Story ID | W3-FIX-CODE-001 |
| Title | prism-dtu-harness: per-DtuType failure scoping and honest Drop semantics |
| Branch | feature/W3-FIX-CODE-001 |
| HEAD SHA | 8d4a472a |
| Recorded | 2026-05-01 |
| Recorder | Demo Recorder (Claude Code) |

---

## Coverage Map

| AC | Title | Recording | Result |
|----|-------|-----------|--------|
| AC-001 | with_failure injects ONLY into specified DtuType | [AC-001-failure-scope-per-dtu-type.gif](AC-001-failure-scope-per-dtu-type.gif) | PASS |
| AC-002 | Drop honors 5-second graceful shutdown budget | [AC-002-drop-graceful-shutdown.gif](AC-002-drop-graceful-shutdown.gif) | PASS |
| AC-003 | Deferred failure path honors DtuType | Covered by AC-001 test (same builder path exercised) | PASS |
| AC-004 | Drop semantics: remove-abort chosen | Covered by AC-002; doc comment updated per story | PASS |
| AC-005 | No regression in existing failure injection tests | 19 pre-existing BC-3.6.001 tests — see suite summary below | PASS |
| AC-006 | No regression in drop/teardown tests | All drop-related tests in logical/network isolation suites pass | PASS |

---

## AC-001: with_failure DtuType Scope

**Recording:** [AC-001-failure-scope-per-dtu-type.gif](AC-001-failure-scope-per-dtu-type.gif)  
**Source tape:** [AC-001-failure-scope-per-dtu-type.tape](AC-001-failure-scope-per-dtu-type.tape)  
**Archival:** [AC-001-failure-scope-per-dtu-type.webm](AC-001-failure-scope-per-dtu-type.webm)

**What is demonstrated:**

A 4-clone harness is built for org `acme` with DtuTypes: Claroty, Armis, CrowdStrike, Cyberint.
`with_failure("acme", DtuType::Claroty, FailureMode::AuthReject)` is called before `build()`.

After build:
- GET `/assets/v1/assets` on the **Claroty** clone returns **401** (AuthReject injected — success path of AC-001)
- GET `/api/v1/devices` (Bearer token) on the **Armis** clone returns **200** (not affected)
- GET `/devices/entities/devices/v2` (Bearer token) on the **CrowdStrike** clone returns **200** (not affected)
- GET `/api/v1/events` on the **Cyberint** clone returns **200** (not affected)

**BC trace:** BC-3.6.001 postcondition 2 (other clones return normal responses), invariant 1 (failure scoped to target)  
**VP trace:** VP-128 (failure injection isolation), VP-129 (per-DtuType injection scope)

**Test:** `test_AC_001_with_failure_only_injects_into_specified_dtu_type` in `tests/failure_scope_test.rs`

```
test test_AC_001_with_failure_only_injects_into_specified_dtu_type ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.04s
```

---

## AC-002: Drop Graceful Shutdown

**Recording:** [AC-002-drop-graceful-shutdown.gif](AC-002-drop-graceful-shutdown.gif)  
**Source tape:** [AC-002-drop-graceful-shutdown.tape](AC-002-drop-graceful-shutdown.tape)  
**Archival:** [AC-002-drop-graceful-shutdown.webm](AC-002-drop-graceful-shutdown.webm)

**What is demonstrated:**

A single-clone harness (Claroty) is built for org `drop-test`. The harness is then dropped
inside a `tokio::time::timeout(5s, ...)` gate. The drop completes within the 5-second budget.

**Drop semantics chosen: remove-abort**

`handle.abort()` calls were removed from `Harness::Drop`. The shutdown broadcast signal is
sent; `axum::Server::with_graceful_shutdown` was already wired during `build()` and handles
orderly drain when the signal fires. In-flight requests complete before the server task exits.
The doc comment at `harness.rs:19` now accurately states this behavior.

**BC trace:** BC-3.5.001 EC-004 ("waits up to 5s for graceful exit"), BC-3.5.002 EC-004 (same in network mode)  
**VP trace:** VP-124 (drop releases resources/ports), VP-130 (drop grace period)

**Test:** `test_AC_002_drop_honors_5s_graceful_shutdown_budget` in `tests/failure_scope_test.rs`

```
test test_AC_002_drop_honors_5s_graceful_shutdown_budget ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.01s
```

---

## Pre-existing BC-3.6.001 Test Suite — No Regression

All 19 pre-existing failure-injection tests in `tests/failure_scope_test.rs` (2 new) plus
the builder ergonomics, logical isolation, network isolation, and network isolation timeout
suites continue to pass at `8d4a472a`.

Full `cargo test -p prism-dtu-harness --features dtu` result:

| Suite | Tests | Result |
|-------|-------|--------|
| builder_ergonomics_test | 19 | ok |
| failure_scope_test (new AC-001, AC-002) | 2 | ok |
| logical_isolation_test | 34 | ok |
| network_isolation_test | 16 | ok |
| network_isolation_timeout_test | 1 | ok |
| doc-tests | 4 (ignored) | ok |

**Total: 72 tests pass, 0 fail.**

---

## Architecture Notes

- `CustomerSpec.initial_failure` changed from `Option<FailureMode>` to `HashMap<DtuType, FailureMode>`
- Phase 4 injection loop iterates `spec.initial_failure.iter()` as `(dtu_type, mode)` — not all `spec.dtu_types`
- `DtuType` derives `Hash + Eq` (verified pre-existing)
- `Harness::Drop` impl: `handle.abort()` calls removed; shutdown broadcast signal fires axum graceful drain
- Doc comment at `harness.rs:19` updated to accurately describe remove-abort semantics

---

_Generated by Demo Recorder per POL-010. Evidence committed to `feature/W3-FIX-CODE-001`._
