# Demo Evidence Report — S-3.1.05

**Story:** S-3.1.05 — prism-spec-engine OrgId-scoped sensor specs
**Branch:** feature/S-3.1.05
**Implementation commit:** 2e566685
**BC Anchors:** BC-3.1.001
**Recorded:** 2026-04-29
**Recorder:** demo-recorder agent

---

## Coverage Summary

| Recording | AC | BC Anchor | Path | Status |
|-----------|-----|-----------|------|--------|
| AC-001-all-7-tests-green | AC-001 | BC-3.1.001 | success | PASS |
| AC-002-cross-org-spec-isolation | AC-002 | BC-3.1.001 postcondition (OrgId isolation) | success | PASS |

All 7 spec-engine tests pass. No failures recorded.

---

## AC-001 — All 7 spec-engine tests GREEN

**Acceptance criterion:** All 7 `prism-spec-engine` `bc_3_1_001_test` tests compile
and pass under `cargo test -p prism-spec-engine --test bc_3_1_001_test 2>&1 | tail -20`.

**Traces to:** BC-3.1.001 (all postconditions and invariants for OrgId-scoped spec store)

**Recordings:**
- [AC-001-all-7-tests-green.gif](AC-001-all-7-tests-green.gif)
- [AC-001-all-7-tests-green.webm](AC-001-all-7-tests-green.webm)
- [AC-001-all-7-tests-green.tape](AC-001-all-7-tests-green.tape) (VHS script source)

**Observed output (7 tests):**
```
test test_BC_3_1_001_get_spec_unknown_org_returns_error ... ok
test test_BC_3_1_001_empty_registry_returns_err_not_panic ... ok
test test_BC_3_1_001_cross_org_spec_isolation ... ok
test test_BC_3_1_001_two_orgs_same_sensor_name_no_collision ... ok
test test_BC_3_1_001_get_spec_resolves_slug_to_org_id ... ok
test test_BC_3_1_001_known_org_missing_sensor_returns_sensor_not_found ... ok
test test_BC_3_1_001_org_rename_preserves_spec_access ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## AC-002 — Cross-org spec isolation

**Acceptance criterion:** A spec registered under `OrgId` A is NOT visible when
looked up under `OrgId` B. The test `test_BC_3_1_001_cross_org_spec_isolation`
verifies this invariant directly.

**Test:** `test_BC_3_1_001_cross_org_spec_isolation`

**Traces to:** BC-3.1.001 postcondition (OrgId isolation), S-3.1.05 AC-002

**Recordings:**
- [AC-002-cross-org-spec-isolation.gif](AC-002-cross-org-spec-isolation.gif)
- [AC-002-cross-org-spec-isolation.webm](AC-002-cross-org-spec-isolation.webm)
- [AC-002-cross-org-spec-isolation.tape](AC-002-cross-org-spec-isolation.tape) (VHS script source)

**Path demonstrated:** success (test passes — assertion confirms spec registered under
`OrgId` A returns `SensorNotFound` when queried under `OrgId` B)

---

## Artifact Inventory

| File | Size | Purpose |
|------|------|---------|
| AC-001-all-7-tests-green.gif | 125 KB | PR embed — full test suite green |
| AC-001-all-7-tests-green.webm | 256 KB | Archival — full test suite green |
| AC-001-all-7-tests-green.tape | 829 B | VHS script source |
| AC-002-cross-org-spec-isolation.gif | 97 KB | PR embed — cross-org spec isolation |
| AC-002-cross-org-spec-isolation.webm | 162 KB | Archival — cross-org spec isolation |
| AC-002-cross-org-spec-isolation.tape | 964 B | VHS script source |

---

## Notes

- VHS 0.10.0 on macOS does not support the `Wait+Line` command reliably; `Sleep` used
  instead. Compilation is pre-warmed in the hidden setup block so the visible recording
  shows only the test run output, not the build phase.
- Error-path demo: the test suite itself provides error-path coverage — tests
  `test_BC_3_1_001_get_spec_unknown_org_returns_error`,
  `test_BC_3_1_001_empty_registry_returns_err_not_panic`, and
  `test_BC_3_1_001_known_org_missing_sensor_returns_sensor_not_found` all assert on
  `Err` variants. These execute and pass in AC-001. No separate error-path tape is
  required because the error paths are exercised as the primary assertion of those tests.
- Both `.gif` (PR embed) and `.webm` (archival) produced for every AC per protocol.
