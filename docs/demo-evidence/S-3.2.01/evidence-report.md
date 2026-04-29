# Demo Evidence Report — S-3.2.01

**Story:** S-3.2.01 — prism-dtu-claroty: Multi-tenant state segregation
**Branch:** feature/S-3.2.01
**Implementation commit:** 80d91208
**BC Anchors:** BC-3.2.001, BC-3.2.003
**Recorded:** 2026-04-29
**Recorder:** demo-recorder agent

---

## Coverage Summary

| Recording | AC | BC Anchor | Path | Status |
|-----------|-----|-----------|------|--------|
| AC-001-all-tests-green | AC-001..AC-007 | BC-3.2.001 | success | PASS |
| AC-002-cross-org-isolation | AC-001 | BC-3.2.001 postcondition 1 | success | PASS |

All 16 multi-tenant tests pass. No failures recorded.

---

## AC-001 — All multi-tenant tests GREEN

**Acceptance criterion:** All multi-tenant state segregation tests compile and pass under
`cargo test -p prism-dtu-claroty --features dtu --test multi_tenant`.

**Traces to:** BC-3.2.001 (all postconditions and invariants), BC-3.2.003 (token isolation gate)

**Recordings:**
- [AC-001-all-tests-green.gif](AC-001-all-tests-green.gif)
- [AC-001-all-tests-green.webm](AC-001-all-tests-green.webm)
- [AC-001-all-tests-green.tape](AC-001-all-tests-green.tape) (VHS script source)

**Observed output (16 tests):**
```
test test_bc_3_2_001_reset_all_clears_all_orgs ... ok
test test_bc_3_2_001_missing_org_returns_default ... ok
test test_bc_3_2_001_cross_org_lookup_returns_empty ... ok
test test_bc_3_2_001_reset_for_is_selective ... ok
test test_bc_3_2_001_write_isolation ... ok
test test_bc_3_2_001_reset_for_clears_all_devices_for_org ... ok
test test_bc_3_2_001_same_org_lookup_returns_stored_tag ... ok
test test_bc_3_2_001_independent_per_org_state ... ok
test test_bc_3_2_001_prop_cross_org_lookup_returns_default ... ok
test test_bc_3_2_001_prop_reset_for_selectivity ... ok
test test_bc_3_2_001_prop_write_isolation ... ok
test test_bc_3_2_001_http_reset_for_returns_200 ... ok
test test_bc_3_2_001_http_reset_for_invalid_org_id_returns_400 ... ok
test test_bc_3_2_001_http_cross_org_tag_not_visible_to_other_org ... ok
test test_bc_3_2_001_http_independent_per_org_tag_state ... ok
test test_bc_3_2_001_http_reset_for_clears_org_a_preserves_org_b ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

---

## AC-002 — Cross-org isolation (HTTP path)

**Acceptance criterion:** Tag written under `X-Org-Id: ORG_A` is NOT visible when the
same device is listed under `X-Org-Id: ORG_B`. Traces directly to BC-3.2.001
postcondition 1 and story AC-001 (cross-org lookup returns empty).

**Test:** `test_bc_3_2_001_http_cross_org_tag_not_visible_to_other_org`

**Traces to:** BC-3.2.001 postcondition 1, S-3.2.01 AC-001

**Recordings:**
- [AC-002-cross-org-isolation.gif](AC-002-cross-org-isolation.gif)
- [AC-002-cross-org-isolation.webm](AC-002-cross-org-isolation.webm)
- [AC-002-cross-org-isolation.tape](AC-002-cross-org-isolation.tape) (VHS script source)

**Path demonstrated:** success (test passes — assertion confirms ORG_B device list has
empty tags for a device tagged only under ORG_A)

---

## Artifact Inventory

| File | Size | Purpose |
|------|------|---------|
| AC-001-all-tests-green.gif | 175 KB | PR embed — full test suite green |
| AC-001-all-tests-green.webm | 376 KB | Archival — full test suite green |
| AC-001-all-tests-green.tape | 838 B | VHS script source |
| AC-002-cross-org-isolation.gif | 109 KB | PR embed — HTTP cross-org isolation |
| AC-002-cross-org-isolation.webm | 182 KB | Archival — HTTP cross-org isolation |
| AC-002-cross-org-isolation.tape | 1.0 KB | VHS script source |

---

## Notes

- VHS 0.10.0 on macOS does not support the `Wait+Line` command reliably; `Sleep` used
  instead. Compilation is pre-warmed in the hidden setup block so the visible recording
  shows only the test run output, not the build phase.
- Error-path demo: the test suite itself contains the error-path coverage via proptest
  (`prop_cross_org_lookup_returns_default`, `prop_write_isolation`,
  `prop_reset_for_selectivity`) — these are executed and pass in AC-001. No separate
  error-path tape is required because the property tests exercise adversarial inputs
  (arbitrary `OrgId` pairs with `org_a != org_b`) as their primary assertion.
- BC-3.2.003 (session token isolation) is covered by the `DEFAULT_ORG_ID` compile-gate
  invariant; no runtime demo is required as the enforcement is at the type/compile level.
