# Demo Evidence Report — S-3.1.06

**Story:** S-3.1.06 — prism-sensors OrgId-keyed adapter dispatch
**Branch:** feature/S-3.1.06
**Implementation commit:** c15cc504
**BC Anchors:** BC-3.2.001
**Recorded:** 2026-04-29
**Recorder:** demo-recorder agent

---

## Coverage Summary

| Recording | AC | BC Anchor | Path | Status |
|-----------|-----|-----------|------|--------|
| AC-001-bc-3-2-001-tests-green | AC-001 | BC-3.2.001 | success | PASS |
| AC-002-cross-org-dispatch | AC-002 | BC-3.2.001 OrgId isolation postcondition | success | PASS |

All 15 BC-3.2.001 tests pass. No failures recorded.

---

## AC-001 — All BC-3.2.001 tests GREEN

**Acceptance criterion:** All `prism-sensors` tests matching `bc_3_2_001` compile
and pass under `cargo test -p prism-sensors --all-features bc_3_2_001 2>&1 | tail -25`.

**Traces to:** BC-3.2.001 (all postconditions and invariants for OrgId-keyed adapter dispatch)

**Recordings:**
- [AC-001-bc-3-2-001-tests-green.gif](AC-001-bc-3-2-001-tests-green.gif)
- [AC-001-bc-3-2-001-tests-green.webm](AC-001-bc-3-2-001-tests-green.webm)
- [AC-001-bc-3-2-001-tests-green.tape](AC-001-bc-3-2-001-tests-green.tape) (VHS script source)

**Observed output (15 tests):**
```
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_default_org_id_bytes_accessible_in_test_context ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_org_id_mismatch_is_fatal_dispatch_error ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_fan_out_target_org_id_field_is_org_id_type ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_no_bare_string_hashmap_in_adapter_rs_post_migration ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_lookup_unknown_org_returns_default_not_error ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_sensor_spec_distinct_org_ids_are_not_equal ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_reset_for_org_a_does_not_affect_org_b ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_cross_org_lookup_returns_empty_not_other_org_data ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_sensor_spec_org_id_field_is_org_id_type ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_event_buffer_key_prefix_must_be_uuid_format ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_write_under_org_a_does_not_affect_org_b_entry ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_init_registry_for_org_accepts_org_id_parameter ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_proptest_reset_for_org_a_selectivity ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_proptest_cross_org_lookup_always_returns_empty ... ok
test tests::bc_3_2_001_org_id_dispatch::test_BC_3_2_001_proptest_write_org_a_does_not_modify_org_b ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 102 filtered out; finished in 0.15s
```

---

## AC-002 — Cross-org dispatch isolation

**Acceptance criterion:** A sensor dispatch keyed under `OrgId` A returns empty (not
data from `OrgId` B) when looked up under `OrgId` B. The property-based test
`test_BC_3_2_001_proptest_cross_org_lookup_always_returns_empty` verifies this
invariant across random `OrgId` pairs.

**Test:** `test_BC_3_2_001_proptest_cross_org_lookup_always_returns_empty`

**Traces to:** BC-3.2.001 OrgId isolation postcondition, S-3.1.06 AC-002

**Recordings:**
- [AC-002-cross-org-dispatch.gif](AC-002-cross-org-dispatch.gif)
- [AC-002-cross-org-dispatch.webm](AC-002-cross-org-dispatch.webm)
- [AC-002-cross-org-dispatch.tape](AC-002-cross-org-dispatch.tape) (VHS script source)

**Path demonstrated:** success (proptest passes — for any two distinct OrgIds A and B,
a lookup under B after a write under A returns empty, never leaking A's data)

---

## Artifact Inventory

| File | Size | Purpose |
|------|------|---------|
| AC-001-bc-3-2-001-tests-green.gif | 128 KB | PR embed — full 15-test suite green |
| AC-001-bc-3-2-001-tests-green.webm | 265 KB | Archival — full 15-test suite green |
| AC-001-bc-3-2-001-tests-green.tape | 864 B | VHS script source |
| AC-002-cross-org-dispatch.gif | 736 KB | PR embed — cross-org proptest isolation |
| AC-002-cross-org-dispatch.webm | 449 KB | Archival — cross-org proptest isolation |
| AC-002-cross-org-dispatch.tape | 1.0 KB | VHS script source |

---

## Notes

- VHS 0.10.0 on macOS does not support the `Wait+Line` command reliably; `Sleep` used
  instead. Compilation is pre-warmed in the hidden setup block so the visible recording
  shows only the test run output, not the build phase.
- Error-path demo: the test suite itself provides error-path coverage — tests
  `test_BC_3_2_001_org_id_mismatch_is_fatal_dispatch_error` and
  `test_BC_3_2_001_cross_org_lookup_returns_empty_not_other_org_data` assert on
  error/isolation variants. These execute and pass in AC-001. No separate error-path
  tape is required because the error paths are exercised as the primary assertion of
  those tests.
- Both `.gif` (PR embed) and `.webm` (archival) produced for every AC per protocol.
