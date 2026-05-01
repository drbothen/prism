# Demo Evidence Report — S-3.4.01

**Story:** prism-dtu-claroty: harness migration (ClarotyClone into prism-dtu-harness)
**Story ID:** S-3.4.01
**Implementation Commit:** 2840474a
**Evidence Recorded:** 2026-04-30
**Test Count:** 56 harness_tests + 60 legacy claroty tests = 116 prism-dtu-claroty tests; 70 prism-dtu-harness tests

---

## Coverage Map

| Recording | AC | Behavioral Contract | Tests Exercised |
|-----------|-----|--------------------|--------------------|
| AC-001-claroty-harness-tests-green | AC-001 (harness_tests) | BC-3.5.001, BC-3.5.002 | 56/56 harness_tests (all migrated tests) |
| AC-002-multi-org-logical | AC-002 (multi-org logical isolation) | BC-3.5.001 postcondition 2 | `ac_multi_org_logical_isolation` — pairwise-disjoint device ID sets |
| AC-003-network-cross-creds-401 | AC-003 (network cross-creds 401) | BC-3.5.002 postcondition 2 | `ac_network_cross_creds_401` — cross-org token rejected HTTP 401 |
| AC-004-harness-regression-safe | AC-004 (harness regression safety) | BC-3.5.001, BC-3.5.002 | 70/70 prism-dtu-harness tests still pass |
| AC-005-legacy-tests-still-pass | AC-005 (legacy claroty tests) | BC-3.5.001 | 60/60 legacy prism-dtu-claroty tests still pass |

---

## Recordings

### AC-001 — Claroty harness migration tests: 56/56 GREEN

Demonstrates the full `harness_tests` suite for `prism-dtu-claroty` running under `--features dtu`
with all 56 migrated tests passing. Covers all ACs originally introduced in S-6.08 (AC-001 through
AC-008), edge cases (EC-001 through EC-006), fidelity validator, configure variants, and the two
new isolation tests (`ac_multi_org_logical_isolation`, `ac_network_cross_creds_401`).

- [AC-001-claroty-harness-tests-green.gif](AC-001-claroty-harness-tests-green.gif)
- [AC-001-claroty-harness-tests-green.webm](AC-001-claroty-harness-tests-green.webm)
- [AC-001-claroty-harness-tests-green.tape](AC-001-claroty-harness-tests-green.tape)

**Traces to:** BC-3.5.001, BC-3.5.002 — all migrated Claroty harness tests green

---

### AC-002 — Multi-org logical isolation: pairwise-disjoint device ID sets

Demonstrates `ac_multi_org_logical_isolation`: a 2-org `IsolationMode::Logical` harness
built with `test-tenant` (seed=1) and `other-tenant` (seed=2) returns device ID sets that
are pairwise disjoint. No `asset_id` appears in both orgs.

- [AC-002-multi-org-logical.gif](AC-002-multi-org-logical.gif)
- [AC-002-multi-org-logical.webm](AC-002-multi-org-logical.webm)
- [AC-002-multi-org-logical.tape](AC-002-multi-org-logical.tape)

**Traces to:** BC-3.5.001 postcondition 2 — query scoped to registered org returns only that org's records

---

### AC-003 — Network isolation cross-creds 401: cross-org token rejected

Demonstrates `ac_network_cross_creds_401`: a 2-org `IsolationMode::Network` harness
returns HTTP 401 when `test-tenant`'s admin token is sent to `other-tenant`'s endpoint.
Network mode issues per-org distinct credentials; cross-org token reuse must be rejected.

- [AC-003-network-cross-creds-401.gif](AC-003-network-cross-creds-401.gif)
- [AC-003-network-cross-creds-401.webm](AC-003-network-cross-creds-401.webm)
- [AC-003-network-cross-creds-401.tape](AC-003-network-cross-creds-401.tape)

**Traces to:** BC-3.5.002 postcondition 2 — cross-org credential mismatch returns HTTP 401

---

### AC-004 — Harness regression safety: 70/70 prism-dtu-harness still pass

Demonstrates that the full `prism-dtu-harness` test suite (logical isolation + network isolation +
builder ergonomics = 70 tests) continues to pass after the Claroty wiring in S-3.4.01.
No harness invariant was broken by adding Claroty as a registered DTU type.

- [AC-004-harness-regression-safe.gif](AC-004-harness-regression-safe.gif)
- [AC-004-harness-regression-safe.webm](AC-004-harness-regression-safe.webm)
- [AC-004-harness-regression-safe.tape](AC-004-harness-regression-safe.tape)

**Traces to:** BC-3.5.001, BC-3.5.002 — harness isolation invariants preserved

---

### AC-005 — Legacy Claroty tests still pass: 60/60

Demonstrates that the 60 pre-existing `prism-dtu-claroty` tests (ac_1_devices_list through
ac_8_reset, edge_cases, fidelity_validator, multi_tenant, td_wv0_04_configure_deny_unknown,
td_wv0_07_configure_requires_admin_token, bc_3_4_claroty_generator) continue to pass after
the harness migration. No regressions introduced.

- [AC-005-legacy-tests-still-pass.gif](AC-005-legacy-tests-still-pass.gif)
- [AC-005-legacy-tests-still-pass.webm](AC-005-legacy-tests-still-pass.webm)
- [AC-005-legacy-tests-still-pass.tape](AC-005-legacy-tests-still-pass.tape)

**Traces to:** BC-3.5.001 — existing AC coverage preserved after harness migration

---

## Summary

| Total recordings | 5 |
|---|---|
| .tape scripts | 5 |
| .gif outputs | 5 |
| .webm outputs | 5 |
| ACs covered | AC-001 through AC-005 (all demoed) |
| BCs covered | BC-3.5.001, BC-3.5.002 |
| Tests demonstrated | 116 prism-dtu-claroty (56 harness + 60 legacy) + 70 prism-dtu-harness |
| Evidence status | COMPLETE |
