# Demo Evidence Report — S-3.1.04

**Story:** prism-credentials OrgId-keyed namespace
**Story ID:** S-3.1.04
**Impl SHA:** 033ad83e
**Recorded:** 2026-04-29
**Product type:** CLI (Rust) — VHS recordings

---

## Coverage Map

| AC | Description | Success Path | Error Path | Recording |
|----|-------------|:---:|:---:|-----------|
| AC-001 | All 24 credentials tests GREEN | ✅ | N/A | `AC-001-all-24-tests-green.{gif,webm}` |
| AC-002 | Cross-org isolation | ✅ | N/A | `AC-002-cross-org-isolation.{gif,webm}` |

---

## AC-001 — All 24 Credentials Tests GREEN

**Command:** `cargo test -p prism-credentials --test bc_3_2_002_org_id_namespace 2>&1 | tail -30`

**What it demonstrates:** The full BC-3.2.002 test suite (24 tests) passes including
preconditions, postconditions, invariants, edge cases, and the proptest property.

**Test inventory:**
- `test_BC_3_2_002_namespace_key_format_uses_org_id_uuid` — Precondition 1: key uses UUID
- `test_BC_3_2_002_invariant_no_slug_keyed_fallback_in_namespace_key` — Precondition 4: no slug fallback
- `test_BC_3_2_002_get_by_org_returns_credential_for_correct_org` — Postcondition 1
- `test_BC_3_2_002_cross_org_get_returns_not_found` — Postcondition 2
- `test_BC_3_2_002_rename_stable_lookup` — Postcondition 3
- `test_BC_3_2_002_credential_value_not_in_error_message` — Postcondition 4
- `test_BC_3_2_002_invariant_namespace_key_always_from_org_id` — Invariant 1
- `test_BC_3_2_002_invariant_physical_isolation_by_namespace_prefix` — Invariant 3
- `test_BC_3_2_002_invariant_exists_by_org_keyed_by_org_id` — Invariant 4
- `test_BC_3_2_002_tv_01_same_org_round_trip` — TV-01 happy path
- `test_BC_3_2_002_tv_02_cross_org_isolation` — TV-02 cross-org isolation
- `test_BC_3_2_002_tv_03_per_sensor_isolation` — TV-03 per-sensor isolation
- `test_BC_3_2_002_tv_04_rename_stability` — TV-04 rename stability
- `test_BC_3_2_002_ec_001_org_with_credentials` — EC-001
- `test_BC_3_2_002_ec_002_org_without_credentials` — EC-002
- `test_BC_3_2_002_ec_003_per_sensor_not_found` — EC-003
- `test_BC_3_2_002_ec_004_rename_slug_org_id_stable` — EC-004
- `test_BC_3_2_002_ec_005_sequential_slug_reuse_no_collision` — EC-005
- `test_BC_3_2_002_list_by_org_scoped_to_org`
- `test_BC_3_2_002_delete_by_org_removes_only_target`
- `test_BC_3_2_002_delete_by_org_idempotent_returns_false`
- `test_BC_3_2_002_exists_by_org_after_set`
- `test_BC_3_2_002_distinct_org_ids_produce_distinct_keys`
- `proptest_BC_3_2_002_vp_01_cross_org_isolation`

**Recordings:**
- `AC-001-all-24-tests-green.gif`
- `AC-001-all-24-tests-green.webm`
- `AC-001-all-24-tests-green.tape` (VHS script source)

---

## AC-002 — Cross-Org Isolation

**Command:** `cargo test -p prism-credentials --test bc_3_2_002_org_id_namespace test_BC_3_2_002_tv_02_cross_org_isolation -- --nocapture`

**What it demonstrates:** Credentials stored under org A are not visible to org B.
The test stores a credential under `org_a`, then attempts retrieval using `org_b`'s
OrgId — which must return `CredentialError::NotFound`. This validates that the
`namespace_key_by_org_id` UUID prefix provides physical isolation, and that no
slug-based confusion can bridge org boundaries.

**Recordings:**
- `AC-002-cross-org-isolation.gif`
- `AC-002-cross-org-isolation.webm`
- `AC-002-cross-org-isolation.tape` (VHS script source)

---

## VHS Tape Sources

| File | AC | Purpose |
|------|----|---------|
| `AC-001-all-24-tests-green.tape` | AC-001 | Full test suite run |
| `AC-002-cross-org-isolation.tape` | AC-002 | Targeted cross-org isolation test with --nocapture |
