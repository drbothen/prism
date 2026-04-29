---
story: S-3.1.03
phase: Red Gate (Failing Tests)
date: 2026-04-29
agent: test-writer
---

# Red Gate Log — S-3.1.03 OrgRegistry

## Summary

**Result: RED GATE PASSED** — all 35 new tests fail before implementation.

| Metric | Value |
|--------|-------|
| Test file | `crates/prism-core/tests/bc_3_1_003_org_registry.rs` |
| Tests added | 35 |
| Tests passing | 0 |
| Tests failing | 35 |
| Failure cause | `todo!("S-3.1.03: implement OrgRegistry::new")` in all stub methods |
| `cargo test --no-run` | PASS (clean compile, 2 cosmetic doc-comment warnings on `proptest!` blocks) |

## BC Coverage

| BC ID | Clauses Covered | Test Functions |
|-------|----------------|----------------|
| BC-3.1.001 | Postconditions 1-4, Invariant 2, EC-001–005, TV-01–05 | 11 tests |
| BC-3.1.003 | Postconditions 1,3, Invariants 1,3, EC-001–005, TV-01–03, VP-3.1.003-01 | 8 tests + 1 proptest |
| BC-3.1.004 | Postconditions 2-4, Invariants 1-3, EC-001–005, TV-01–03, VP-3.1.004-01,04 | 12 tests + 2 proptests |

## AC Coverage

| AC | Test Function |
|----|--------------|
| AC-1 | `test_BC_3_1_001_tv_05_round_trip_consistency` |
| AC-2 | `test_BC_3_1_001_ac2_resolve_unknown_no_side_effect` |
| AC-4 (no I/O) | `test_BC_3_1_001_ec005_concurrent_reads_are_safe` (thread-based read test) |
| AC-5 | `test_BC_3_1_003_proptest_bijection_size_invariant` |
| AC-6 | `test_BC_3_1_004_tv_01_slug_conflict_error_fields` |
| AC-7 | `test_BC_3_1_004_tv_02_id_conflict_error_fields` |
| AC-8 | `test_BC_3_1_004_ac8_exact_duplicate_is_idempotent` |

## Failure Output (excerpt)

All 35 tests panic at:
```
panicked at crates/prism-core/src/org_registry.rs:94:9:
not yet implemented: S-3.1.03: implement OrgRegistry::new
```

```
test result: FAILED. 0 passed; 35 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Notes

- Two proptest tests (`test_BC_3_1_003_proptest_bijection_size_invariant`,
  `test_BC_3_1_004_proptest_size_unchanged_on_error`,
  `test_BC_3_1_004_proptest_successful_resolve_after_rejection`) use
  `ProptestConfig::with_cases(1000)` as required by the BC verification properties.
- Concurrency test uses `std::thread::spawn` (not tokio) because `OrgRegistry`
  wraps `std::sync::RwLock` — no async runtime needed.
- `non_snake_case = "allow"` was already set in `prism-core/Cargo.toml` lints;
  the `test_BC_3_1_NNN_*` naming convention compiles without warnings.
- `proptest = "1"` and `uuid = { version = "1", features = ["v7"] }` added to
  `[dev-dependencies]` in `prism-core/Cargo.toml`.

## Hand-off to Implementer

All tests are RED. Implementer must make them GREEN by implementing:
1. `OrgRegistry::new()` — construct `RwLock<BiMap<OrgSlug, OrgId>>`
2. `OrgRegistry::resolve(&self, slug)` — read lock, BiMap forward lookup
3. `OrgRegistry::slug_for(&self, id)` — read lock, BiMap reverse lookup
4. `OrgRegistry::register(&self, slug, id)` — write lock, check conflicts, insert
5. `OrgRegistry::len(&self)` — read lock, `bimap.len()`
6. `OrgRegistry::is_empty(&self)` — `self.len() == 0`

Key decisions:
- Idempotent exact-duplicate re-registration returns `Ok(())` (D-050 / AC-8)
- `SlugConflict` when slug bound to different id; `IdConflict` when id bound to different slug
- Both error variants must carry enough fields for operator-actionable Display messages
