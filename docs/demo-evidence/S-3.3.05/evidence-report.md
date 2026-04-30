# Demo Evidence Report — S-3.3.05

**Story:** prism-dtu-harness: builder ergonomics, per-test overrides, and documentation
**Story ID:** S-3.3.05
**Implementation Commit:** 376a0bf7
**Evidence Recorded:** 2026-04-30
**Test Count:** 19 new (builder ergonomics) + 51 prior = 70/70 passing

---

## Coverage Map

| Recording | AC | Behavioral Contract | Tests Exercised |
|-----------|-----|--------------------|--------------------|
| AC-001-builder-ergonomics-tests-green | AC-001/002/003/004/005 | BC-3.5.001, BC-3.5.002, BC-3.6.001 | 19/19 builder ergonomics tests |
| AC-002-with-customer-overrides-inplace | AC-002 | BC-3.5.001 precondition 2 | `with_customer` deduplication, `with_customer_overrides` scale/seed applied, EC-003 last-write-wins (3 tests) |
| AC-003-with-failure-injection | AC-004 | BC-3.6.001 postcondition 1 | `with_failure` pre-injects AuthReject, timeout, None no-op, None clears, multi-org isolation, fluent chain, doc-example (9 tests) |
| AC-004-unknown-slug-deferred | AC-003 | BC-3.6.001 EC-001 | Unknown slug deferred to build() — no panic at call site, no customers variant (2 tests) |
| AC-005-regression-safe | (regression) | BC-3.5.001, BC-3.5.002 | logical_isolation_test (34) + network_isolation_test (16+1) — 51 prior tests pass |

---

## Recordings

### AC-001 — Builder ergonomics test suite: 19/19 GREEN

Demonstrates the complete `builder_ergonomics_test` suite running under `--features dtu`
with all 19 tests passing. Covers all acceptance criteria introduced in S-3.3.05.

- [AC-001-builder-ergonomics-tests-green.gif](AC-001-builder-ergonomics-tests-green.gif)
- [AC-001-builder-ergonomics-tests-green.webm](AC-001-builder-ergonomics-tests-green.webm)
- [AC-001-builder-ergonomics-tests-green.tape](AC-001-builder-ergonomics-tests-green.tape)

**Traces to:** BC-3.5.001 precondition 2, BC-3.5.002, BC-3.6.001 postcondition 1

---

### AC-002 — with_customer_overrides applies in-place (slug deduplication)

Demonstrates that `.with_customer("alpha").with_customer_overrides("alpha", |c| { ... })`
mutates the existing `CustomerSpec` rather than inserting a duplicate. The endpoint count
(1 expected vs 5 if duplicate) is the observable proxy. Also verifies EC-003 last-write-wins
semantics for multiple override calls on the same slug.

- [AC-002-with-customer-overrides-inplace.gif](AC-002-with-customer-overrides-inplace.gif)
- [AC-002-with-customer-overrides-inplace.webm](AC-002-with-customer-overrides-inplace.webm)
- [AC-002-with-customer-overrides-inplace.tape](AC-002-with-customer-overrides-inplace.tape)

**Traces to:** BC-3.5.001 precondition 2, S-3.3.05 Task 2, story EC-003

---

### AC-003 — with_failure shorthand pre-injects failure mode

Demonstrates that `.with_failure(slug, dtu_type, FailureMode::AuthReject)` injects the
failure at build time, making the first HTTP request to that clone return 401 without any
separate `inject_failure` call. Covers AuthReject, timeout scoping to target org, None
no-op, None clearing a prior injection, fluent chain compile+run, and doc-example chain.

- [AC-003-with-failure-injection.gif](AC-003-with-failure-injection.gif)
- [AC-003-with-failure-injection.webm](AC-003-with-failure-injection.webm)
- [AC-003-with-failure-injection.tape](AC-003-with-failure-injection.tape)

**Traces to:** BC-3.6.001 postcondition 1, S-3.3.05 Task 3, VP-128

---

### AC-004 — Unknown slug deferred to build() — no panic at call site

Demonstrates that `.with_failure("unknown_slug", ...)` for an unregistered slug does NOT
panic at the builder call site. The error is deferred to `build()`, which returns
`Err(HarnessError::UnknownOrg { slug: "unknown_slug" })`. Both variants tested: with one
registered org and with no customers at all.

- [AC-004-unknown-slug-deferred.gif](AC-004-unknown-slug-deferred.gif)
- [AC-004-unknown-slug-deferred.webm](AC-004-unknown-slug-deferred.webm)
- [AC-004-unknown-slug-deferred.tape](AC-004-unknown-slug-deferred.tape)

**Traces to:** BC-3.6.001 EC-001, S-3.3.05 story EC-001

---

### AC-005 — Regression safety: 51 prior tests still pass

Demonstrates that the `logical_isolation_test` (34 tests) and `network_isolation_test`
(16 + 1 timeout = 17 tests) suites introduced in S-3.3.03 and S-3.3.04 continue to pass
after the S-3.3.05 builder ergonomics changes.

- [AC-005-regression-safe.gif](AC-005-regression-safe.gif)
- [AC-005-regression-safe.webm](AC-005-regression-safe.webm)
- [AC-005-regression-safe.tape](AC-005-regression-safe.tape)

**Traces to:** BC-3.5.001, BC-3.5.002 — all prior invariants preserved

---

## Summary

| Total recordings | 5 |
|---|---|
| .tape scripts | 5 |
| .gif outputs | 5 |
| .webm outputs | 5 |
| ACs covered | AC-001 through AC-005 (all demoed) |
| BCs covered | BC-3.5.001, BC-3.5.002, BC-3.6.001 |
| Tests demonstrated | 70/70 (19 new + 51 prior) |
| Evidence status | COMPLETE |
