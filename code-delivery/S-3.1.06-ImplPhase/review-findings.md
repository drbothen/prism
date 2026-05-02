# PR Review Findings — S-3.1.06-ImplPhase

**PR:** #117 — prism-sensors: complete adapter OrgId binding (F-48-H-001 closure)
**Reviewer:** vsdd-factory:pr-review-triage (fresh-context)
**Date:** 2026-05-01
**Cycle:** 1

## Verdict: APPROVE

**Finding count:** BLOCKING=0 | MAJOR=0 | MINOR=1 | SUGGESTION=1

---

## AC Verification Matrix

| AC | Description | Result |
|----|-------------|--------|
| AC-001 | `init_registry_for_org` uses `org_id: OrgId` (no `_` prefix); all 4 adapters accept `org_id` as first param | PASS |
| AC-002 | `AdapterRegistry` keyed by `(OrgId, SensorType)` composite; two orgs yield distinct Arc pointers | PASS |
| AC-003 | `SensorError::OrgIdMismatch` returned before any I/O when `spec.org_id != adapter.org_id` | PASS |
| AC-004 (story) | `#[deprecated]` on `init_registry` fires compile warning; legacy path compiles | PASS |
| AC-005 (story) | 6 downstream test files migrated; `cargo test -p prism-sensors` — 0 E0061, all pass | PASS |
| AC-006 (story) | OrgId sentinel from inlined bytes is idempotent; integration test callers use correct construction idiom | PASS |

## Architecture Compliance

| Rule | Result |
|------|--------|
| `grep -rn "HashMap<String," crates/prism-sensors/src/` — 0 hits for mutable state stores | PASS (only FilterMap type alias and comment refs) |
| `grep -rn "OrgRegistry" crates/prism-sensors/src/` — 0 hits (only doc comment) | PASS |
| `grep -rn "_org_id" crates/prism-sensors/src/` in feature branch — 0 hits | PASS |
| OrgIdMismatch guard before I/O in all 4 adapters | PASS (verified line positions: before `acquire_http_permit()` and `login()`) |
| E-SENSOR-060 in error-taxonomy.md (line 431, taxonomy v1.12) | PASS |
| `adapter.org_id` fields are `pub(crate)` in all 4 adapter structs | PASS |
| `DEFAULT_ORG_ID_BYTES` remains `#[cfg(test)]` gated | PASS |
| No `OrgRegistry` import in production `prism-sensors/src/` | PASS |

---

## Findings

### MINOR-001 — Stale `// TODO impl-phase` comments in `bc_2_01_013.rs`

**File:** `crates/prism-sensors/src/tests/bc_2_01_013.rs`
**Lines:** 77, 95, 126, 142, 187
**Severity:** MINOR

**Description:** Five occurrences of `// TODO impl-phase: use real OrgId` remain in the internal
BC-2.01.013 registry tests. These comments were added during the Red Gate phase to mark that
tests used `OrgId::new()` as a placeholder. Since the implementation phase (this story) is now
complete, the comments are stale. The tests still function correctly — any `OrgId` works for
BC-2.01.013 tests since they only exercise get/register round-trips within the same org — but
the comments imply outstanding work where there is none.

**Recommendation:** Remove the `// TODO impl-phase: use real OrgId` suffix or replace with
`// BC-2.01.013 tests operate on a single org; OrgId value is arbitrary` to clarify intent.
Not blocking — tests pass and behavior is correct.

---

### SUGGESTION-001 — Internal bc_2_01_013 tests could use sentinel bytes for consistency

**File:** `crates/prism-sensors/src/tests/bc_2_01_013.rs`
**Severity:** SUGGESTION

**Description:** The external integration tests and `org_id_binding.rs` all use the canonical
`DEFAULT_ORG_ID_BYTES` sentinel for test OrgId construction. The internal `bc_2_01_013.rs` tests
use `OrgId::new()` (generates a fresh random UUID per call). This is functionally correct but
creates an inconsistency in test patterns across the codebase. Using the sentinel bytes in
`bc_2_01_013.rs` would make the test corpus consistent and allow `DEFAULT_ORG_ID_BYTES` (which
is `#[cfg(test)]`-accessible from within the crate) to be used directly.

**Recommendation:** Not blocking. Consider addressing in a follow-up cleanup story.

---

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1     | 2        | 0        | 0     | 2 (MINOR+SUGGESTION, non-blocking) |

**Verdict: APPROVE** — All 6 ACs fully satisfied, all 8 architecture compliance checks pass,
0 blocking findings. The MINOR and SUGGESTION findings are cosmetic (stale comments) and
do not affect correctness, security, or behavior. F-48-H-001 is closed.
