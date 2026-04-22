# Red Gate Log — S-6.07 prism-dtu-crowdstrike

**Step:** 2 (failing tests written)
**Date:** 2026-04-22
**Author:** test-writer agent
**Command:** `cargo test -p prism-dtu-crowdstrike --features dtu --no-fail-fast`
**Worktree:** `.worktrees/S-6.07-crowdstrike` @ branch `feature/S-6.07-dtu-crowdstrike`

## Summary

| Metric | Count |
|--------|-------|
| Test binaries compiled | 12 / 12 (all compile clean) |
| Tests run (non-ignored) | 38 |
| Passed | 0 |
| Failed | 38 |
| Ignored | 2 |
| Build errors | 0 |

Red Gate status: **VERIFIED** — all non-ignored tests fail at runtime due to
`CrowdstrikeClone::start()` being `unimplemented!()`.

## Failure Cause

Every test calls `clone.start().await` and immediately panics:

```
thread panicked at crates/prism-dtu-crowdstrike/src/clone.rs:57:9:
not implemented: CrowdstrikeClone::start — not yet implemented
```

This is the correct Red Gate failure mode: tests fail because stubs are
`unimplemented!()`, not because test code is broken.

## Test Files and Coverage Map

| Test File | Tests | AC / EC Covered |
|-----------|-------|-----------------|
| `tests/ac_1_happy_path.rs` | 3 | AC-1 |
| `tests/ac_2_two_step_pagination.rs` | 3 | AC-2 |
| `tests/ac_3_contain_write.rs` | 3 | AC-3 |
| `tests/ac_4_rate_limit.rs` | 3 | AC-4 |
| `tests/ac_5_oauth.rs` | 3 | AC-5 |
| `tests/ac_6_determinism.rs` | 3 | AC-6 |
| `tests/ac_7_auth.rs` | 7 | AC-7 (6 auth-required endpoints + bare bearer) |
| `tests/ac_8_reset.rs` | 3 | AC-8 |
| `tests/edge_cases.rs` | 8 | EC-001 (×2), EC-002, EC-003 (×2), EC-004, EC-005, EC-006 |
| `tests/fidelity.rs` | 1 | AC-1 through AC-8 (all 8 endpoints, FidelityValidator) |
| `tests/integration_vp033.rs` | 1 fail + 1 ignored | AC-9 / VP-033 |
| `tests/integration_vp036.rs` | 1 fail + 1 ignored | AC-10 / VP-036 |

**Total non-ignored failing:** 38  
**Total ignored (needs-prism-audit):** 2

## Ignored Tests

| Test | Reason | Blocking Story |
|------|--------|----------------|
| `crowdstrike_vp033_write_intent_before_dtu_arrival` | needs prism-audit InMemoryBackend | S-3.07 |
| `crowdstrike_vp036_session_context_drops_before_error` | needs prism-sensors SessionContext | S-3.06 |

## Spec Underspecification Notes

1. **AC-6 seed behavior on static fixtures:** The spec says "seed=42 → same response
   twice" (AC-6) but the fixture files are static JSON arrays. If the implementation
   uses pure static fixtures (not seed-shuffled), then `ac_6_different_seeds_produce_different_responses`
   will also pass trivially once implemented (seed does not affect output). The test
   includes an inline note about this. The implementer should confirm whether seed
   influences fixture ordering/selection or only error injection.

2. **Fidelity: unauthenticated probes.** The `FidelityValidator` does not add an
   `Authorization` header. The story spec says all data endpoints require a bearer
   token (AC-7). If the fidelity probes hit 401 instead of 200, the fidelity test
   will fail even after implementation. The implementer should either:
   a. Have `FidelityValidator` send a default bearer (preferred), or
   b. Add a DTU "fidelity-bypass" mode that skips auth for probe requests.
   This is noted as an underspecification in the fidelity test body.

3. **PATCH dispatch:** The story spec says `PATCH /detects/entities/detects/v2`
   dispatches to `update_status` or `assign` based on presence of `assigned_to_uid`.
   The stub has two separate handler stubs (`update_detection_status`, `assign_detection`)
   but a single route. The router must dispatch based on body content, not method.
   The tests cover both paths; the implementer must wire the dispatch logic.

## Handoff to Implementer

All 38 tests fail for the right reason. Implement in this order to pass tests
progressively:

1. `CrowdstrikeClone::start()` — bind port, spawn axum router
2. `build_router()` — wire all 8 endpoints
3. `oauth.rs::token()` — simplest handler (no auth, no state)
4. `detections.rs::list_detection_ids()` + `get_detection_summaries()`
5. `hosts.rs::list_host_ids()` + `get_host_details()`
6. `writes.rs::contain_device()` + `lift_containment()` + status/assign
7. Populate `fixtures/*.json` (50 detection IDs + detail, 30 host IDs + detail)
8. Wire `FailureLayer` from `prism-dtu-common`
9. Un-ignore VP-033/VP-036 once prism-audit lands (S-3.06, S-3.07)
