# Demo Evidence Report — S-3.2.08

**Story:** S-3.2.08 — prism-query CrowdStrike session ID org-scoping
**Branch:** feature/S-3.2.08
**Implementation commit:** c734b589
**BC Anchors:** BC-3.2.003 D-048
**Recorded:** 2026-04-30
**Recorder:** demo-recorder agent

---

## Coverage Summary

| Recording | AC | BC Anchor | Path | Status |
|-----------|-----|-----------|------|--------|
| AC-001-bc-3-2-003-tests-green | AC-001 | BC-3.2.003 | success | PASS |
| AC-002-cross-org-collision-impossibility | AC-002 | BC-3.2.003 cross-org structural separation | success | PASS |
| AC-003-xor-involution | AC-003 | BC-3.2.003 XOR involution property | success | PASS |

All 15 BC-3.2.003 tests pass. No failures recorded.

---

## AC-001 — All 15 BC-3.2.003 tests GREEN

**Acceptance criterion:** All `prism-query` integration tests in `crowdstrike_session_isolation`
compile and pass under:

```
cargo test -p prism-query --test crowdstrike_session_isolation -- --nocapture 2>&1 | tail -25
```

**Traces to:** BC-3.2.003 D-048 (all postconditions: generate, extract, xor_org_into_session_bytes)

**Re-run command:**
```bash
cargo test -p prism-query --test crowdstrike_session_isolation -- --nocapture
```

**Recordings:**
- [AC-001-bc-3-2-003-tests-green.gif](AC-001-bc-3-2-003-tests-green.gif)
- [AC-001-bc-3-2-003-tests-green.webm](AC-001-bc-3-2-003-tests-green.webm)
- [AC-001-bc-3-2-003-tests-green.tape](AC-001-bc-3-2-003-tests-green.tape) (VHS script source)

**Observed output (15 tests):**
```
test test_BC_3_2_003_generate_embeds_org_a_roundtrip ... ok
test test_BC_3_2_003_generate_embeds_org_b_roundtrip ... ok
test test_BC_3_2_003_generate_org_a_never_returns_org_b ... ok
test test_BC_3_2_003_xor_org_involutive_table_driven ... ok
test test_BC_3_2_003_xor_same_base_different_orgs_differ_in_bytes_8_to_15 ... ok
test test_BC_3_2_003_xor_nil_org_is_identity ... ok
test test_BC_3_2_003_generate_produces_valid_uuid_v7 ... ok
test test_BC_3_2_003_generate_uuid_time_bits_non_zero ... ok
test test_BC_3_2_003_extract_rejects_uuid_v4 ... ok
test test_BC_3_2_003_extract_rejects_malformed_string ... ok
test test_BC_3_2_003_extract_rejects_empty_string ... ok
test test_BC_3_2_003_cross_org_collision_impossibility_1000_each ... ok
test test_BC_3_2_003_intra_org_uniqueness_1000_sessions ... ok
test test_BC_3_2_003_session_registry_lookup_org_b_misses_org_a_entry ... ok
test test_BC_3_2_003_xor_preserves_bytes_0_to_7_modifies_8_to_15 ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---

## AC-002 — Cross-org structural separation (collision impossibility)

**Acceptance criterion:** Session IDs generated for two distinct orgs never collide
across 1,000 sessions each. The test generates 1,000 sessions under org A and 1,000
under org B, asserts the two sets are fully disjoint (no shared session ID), and
confirms that extracting a session from org B's set under org A's key never returns
a valid org-A session.

**Test:** `test_BC_3_2_003_cross_org_collision_impossibility_1000_each`

**Traces to:** BC-3.2.003 D-048, cross-org structural separation postcondition

**Re-run command:**
```bash
cargo test -p prism-query --test crowdstrike_session_isolation \
  test_BC_3_2_003_cross_org_collision_impossibility_1000_each -- --nocapture
```

**Recordings:**
- [AC-002-cross-org-collision-impossibility.gif](AC-002-cross-org-collision-impossibility.gif)
- [AC-002-cross-org-collision-impossibility.webm](AC-002-cross-org-collision-impossibility.webm)
- [AC-002-cross-org-collision-impossibility.tape](AC-002-cross-org-collision-impossibility.tape) (VHS script source)

**Path demonstrated:** success (1,000 sessions per org, zero collisions confirmed)

---

## AC-003 — XOR involution property

**Acceptance criterion:** `xor_org_into_session_bytes` is its own inverse: applying
it twice with the same org ID restores the original bytes. Table-driven test covers
multiple (org_id, session_bytes) pairs.

**Test:** `test_BC_3_2_003_xor_org_involutive_table_driven`

**Traces to:** BC-3.2.003 D-048, XOR involution property of session ID org-scoping

**Re-run command:**
```bash
cargo test -p prism-query --test crowdstrike_session_isolation \
  test_BC_3_2_003_xor_org_involutive_table_driven -- --nocapture
```

**Recordings:**
- [AC-003-xor-involution.gif](AC-003-xor-involution.gif)
- [AC-003-xor-involution.webm](AC-003-xor-involution.webm)
- [AC-003-xor-involution.tape](AC-003-xor-involution.tape) (VHS script source)

**Path demonstrated:** success (table-driven involution holds across all test vectors)

---

## Artifact Inventory

| File | Size | Purpose |
|------|------|---------|
| AC-001-bc-3-2-003-tests-green.gif | 172 KB | PR embed — full 15-test suite green |
| AC-001-bc-3-2-003-tests-green.webm | 352 KB | Archival — full 15-test suite green |
| AC-001-bc-3-2-003-tests-green.tape | 930 B | VHS script source |
| AC-002-cross-org-collision-impossibility.gif | 91 KB | PR embed — cross-org collision impossibility |
| AC-002-cross-org-collision-impossibility.webm | 176 KB | Archival — cross-org collision impossibility |
| AC-002-cross-org-collision-impossibility.tape | 1.1 KB | VHS script source |
| AC-003-xor-involution.gif | 87 KB | PR embed — XOR involution table-driven test |
| AC-003-xor-involution.webm | 168 KB | Archival — XOR involution table-driven test |
| AC-003-xor-involution.tape | 1.0 KB | VHS script source |

---

## Notes

- VHS 0.10.0 on macOS does not support the `Wait+Line` command reliably; `Sleep` used
  instead. Compilation is pre-warmed in the hidden setup block so the visible recording
  shows only the test run output, not the build phase.
- Error-path demo: the test suite itself provides error-path coverage. Tests
  `test_BC_3_2_003_extract_rejects_uuid_v4`, `test_BC_3_2_003_extract_rejects_malformed_string`,
  `test_BC_3_2_003_extract_rejects_empty_string`, and
  `test_BC_3_2_003_generate_org_a_never_returns_org_b` assert on rejection and isolation
  variants. These execute and pass in AC-001. No separate error-path tape is required
  because the error paths are exercised as the primary assertion of those tests.
- Both `.gif` (PR embed) and `.webm` (archival) produced for every AC per protocol.
