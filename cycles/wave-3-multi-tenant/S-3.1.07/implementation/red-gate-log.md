---
story: S-3.1.07
phase: red-gate
timestamp: 2026-04-29
status: RED_GATE_PASS
---

# Red Gate Log — S-3.1.07

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|--------------|-----------------|------|
| S-3.1.07 | 25 (11 org-field + 14 aql_hash) | yes — 25/25 FAILED via `todo!()` panic or incorrect golden vector | PASS — Red Gate confirmed |

## Test Files

- `crates/prism-audit/tests/bc_3_1_001_org_fields.rs` — 11 org-field tests
- `crates/prism-audit/tests/bc_3_1_002_aql_hash.rs` — 14 aql_hash tests

## Tests Written

### Org-Field Tests (bc_3_1_001_org_fields.rs)

| Test Name | BC Clause | AC |
|-----------|-----------|-----|
| `test_BC_3_1_002_org_id_present_in_serialized_record` | BC-3.1.002 pc-1 | AC-1 |
| `test_BC_3_1_002_org_slug_present_in_serialized_record` | BC-3.1.002 pc-2 | AC-2 |
| `test_BC_3_1_002_org_id_is_uuid_string` | BC-3.1.002 pc-1 | AC-1 |
| `test_BC_3_1_002_org_slug_matches_supplied_value` | BC-3.1.002 pc-2 | AC-2 |
| `test_BC_3_1_002_proptest_org_fields_non_empty` | BC-3.1.002 pc-3 | AC-3 |
| `test_BC_3_1_002_rename_forensic_trail` | BC-3.1.002 pc-5 | AC-5 |
| `test_BC_3_1_002_uuid_stable_across_rename` | BC-3.1.002 pc-5 | AC-5 |
| `test_BC_3_1_002_pre_rename_slug_unchanged` | BC-3.1.002 pc-5 | AC-5 |
| `test_BC_3_1_002_two_orgs_no_commingling` | BC-3.1.002 EC-005 | — |
| `test_BC_3_1_002_org_id_filters_both_pre_and_post_rename` | BC-3.1.002 EC-002 | — |
| `test_BC_3_1_002_json_shape_top_level_fields` | BC-3.1.002 pc-1,2,3 | AC-1,2,3 |

### AQL Hash Tests (bc_3_1_002_aql_hash.rs)

| Test Name | BC Clause | AC |
|-----------|-----------|-----|
| `test_BC_3_1_002_aql_hash_canonical_crowdstrike_query` | BC-3.1.002 inv-1, TD-ADR005-002 | AC-6 |
| `test_BC_3_1_002_aql_hash_is_64_char_lowercase_hex` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_empty_string` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_deterministic_same_call` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_deterministic_repeated_calls` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_distinct_inputs_produce_distinct_hashes` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_single_byte_change_produces_different_hash` | BC-3.1.002 EC-006 | AC-6 |
| `test_BC_3_1_002_aql_hash_unicode_query` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_very_long_query` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_all_lowercase_hex_chars` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_not_recoverable` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_no_leading_zeros_stripped` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_different_sensor_queries` | BC-3.1.002 inv-1 | AC-6 |
| `test_BC_3_1_002_aql_hash_consistent_with_sha2_crate` | BC-3.1.002 inv-1 | AC-6 |

## Stubs Created

- `crates/prism-audit/src/audit_entry.rs` — `org_id: OrgId` and `org_slug: OrgSlug` fields added as `todo!()` stubs in constructor; `compute_aql_hash` stub returns `todo!()`
- `crates/prism-audit/src/audit_emitter.rs` — `emit_*` signatures updated to accept `org_id` and `org_slug` parameters (stubs)
- `sha2 = "0.10"` added to `[dependencies]` in `crates/prism-audit/Cargo.toml`

## Red Gate Verification

All 25 tests FAIL. Org-field tests fail with `todo!()` panic; aql_hash tests fail because
`compute_aql_hash` is a stub. Test-writer's golden hash vector for
`test_BC_3_1_002_aql_hash_canonical_crowdstrike_query` was incorrect (caught and corrected
by implementer in same commit — correct value:
`207d7ded4cfaf669a5db096fb025086b1e9964e8b0fcc2f924a24481b2accac8`).

```
test result: FAILED. 0 passed; 25 failed; 0 ignored; 0 measured; 0 filtered out
```

## Regression Check

| Existing Tests | Status |
|---------------|--------|
| 113 existing prism-audit tests | all pass — 0 regressions (org fields additive; DefaultHasher still in place until impl) |

## Implementer Handoff

1. Add `org_id: OrgId` and `org_slug: OrgSlug` as required non-Option fields to `AuditEntry`.
2. Update all `AuditEntry::new()` and `emit_*` call sites in `audit_emitter.rs`.
3. Replace `DefaultHasher` with `sha2::Sha256` in `compute_aql_hash` — output 64-char lowercase hex.
4. Correct golden vector in aql_hash test to verified SHA-256 output.
5. Verify `cargo test -p prism-audit` → 138/138 PASS (113 existing + 25 new).

Key constraints: `org_id` and `org_slug` are unconditional (BC-3.1.002 invariant 3). No Option wrappers.
