# Demo Evidence Report — S-3.2.05

**Story:** S-3.2.05 — prism-dtu-slack: Shared-mode OrgId ingress tagging
**Anchor BCs:** BC-3.2.004, BC-3.2.005
**Implementation commit:** 31108488
**Recorded:** 2026-04-29
**Tool:** VHS 0.10.0

---

## Coverage Summary

| Recording | AC | BC Anchor | Result |
|-----------|-----|-----------|--------|
| AC-001-all-7-org-tagging-tests-green | AC-001 through AC-007 | BC-3.2.004 + BC-3.2.005 | PASS — 7/7 |
| AC-002-concurrent-orgid-tagging | AC-003 | BC-3.2.004 postcondition 4 | PASS — 1/1 |

---

## AC-001 — All 7 org_tagging Tests GREEN

**Traces to:** BC-3.2.004 (postconditions 1, 2, 4, 5) + BC-3.2.005 (postconditions 1, 3, invariant 4)

**Command:**
```
cargo test -p prism-dtu-slack --features dtu --test org_tagging
```

**Terminal output captured (final frame):**
```
running 7 tests
test test_BC_3_2_005_invalid_mode_string_rejected_at_deserialization ... ok
test test_BC_3_2_004_concurrent_sends_distinguished ... ok
test test_BC_3_2_005_dtu_mode_is_shared_at_startup ... ok
test test_BC_3_2_004_org_id_not_in_http_url ... ok
test test_BC_3_2_005_mode_immutable_after_startup ... ok
test test_BC_3_2_004_org_id_in_payload_body ... ok
test test_BC_3_2_004_mode_metadata_absent_from_query_results ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; finished in 5.08s
```

**Recordings:**
- [AC-001-all-7-org-tagging-tests-green.gif](AC-001-all-7-org-tagging-tests-green.gif)
- [AC-001-all-7-org-tagging-tests-green.webm](AC-001-all-7-org-tagging-tests-green.webm)
- [AC-001-all-7-org-tagging-tests-green.tape](AC-001-all-7-org-tagging-tests-green.tape)

---

## AC-002 — Concurrent OrgId Tagging (BC-3.2.004 postcondition 4)

**Traces to:** BC-3.2.004 postcondition 4, EC-001

**Command:**
```
cargo test -p prism-dtu-slack --features dtu --test org_tagging \
  test_BC_3_2_004_concurrent_sends_distinguished -- --nocapture
```

**Terminal output captured (final frame):**
```
running 1 test
test test_BC_3_2_004_concurrent_sends_distinguished ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 5.07s
```

The test spawns two concurrent Tokio tasks (one per org), each POSTing to the shared
Slack DTU webhook with an `X-Prism-Org-Id` header. After `tokio::join!`, the capture
store is asserted to contain exactly 2 entries with distinct OrgId UUIDs matching each
sender — proving Mutex serialization preserves attribution under concurrency.

**Recordings:**
- [AC-002-concurrent-orgid-tagging.gif](AC-002-concurrent-orgid-tagging.gif)
- [AC-002-concurrent-orgid-tagging.webm](AC-002-concurrent-orgid-tagging.webm)
- [AC-002-concurrent-orgid-tagging.tape](AC-002-concurrent-orgid-tagging.tape)

---

## Acceptance Criteria Coverage

| AC | Description | Test Name | Status |
|----|-------------|-----------|--------|
| AC-001 | OrgId UUID present in captured payload body | test_BC_3_2_004_org_id_in_payload_body | PASS |
| AC-002 | OrgId absent from HTTP routing fields (URL/headers) | test_BC_3_2_004_org_id_not_in_http_url | PASS |
| AC-003 | Concurrent sends from distinct orgs produce independent payloads | test_BC_3_2_004_concurrent_sends_distinguished | PASS |
| AC-004 | No mode metadata in OCSF query results | test_BC_3_2_004_mode_metadata_absent_from_query_results | PASS |
| AC-005 | DtuMode::Shared set at startup; dispatch tags OrgId | test_BC_3_2_005_dtu_mode_is_shared_at_startup | PASS |
| AC-006 | Invalid mode string rejected at deserialization with TOML context | test_BC_3_2_005_invalid_mode_string_rejected_at_deserialization | PASS |
| AC-007 | reload_config does not change DtuMode | test_BC_3_2_005_mode_immutable_after_startup | PASS |

**Total: 7/7 PASS — full AC coverage**
