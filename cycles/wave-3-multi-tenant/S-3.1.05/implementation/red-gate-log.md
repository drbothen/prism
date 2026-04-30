---
story: S-3.1.05
phase: red-gate
timestamp: 2026-04-29
status: RED_GATE_PASS
---

# Red Gate Log — S-3.1.05

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|--------------|-----------------|------|
| S-3.1.05 | 7 (bc_3_1_001_test.rs) | yes — 7/7 FAILED via `todo!()` panic | PASS — Red Gate confirmed |

## Test File

`crates/prism-spec-engine/tests/bc_3_1_001_test.rs`

## Tests Written

| Test Name | BC Clause | AC |
|-----------|-----------|-----|
| `test_BC_3_1_001_get_spec_resolves_slug_to_org_id` | BC-3.1.001 postcond-1, EC-001 | AC-1 |
| `test_BC_3_1_001_get_spec_unknown_org_returns_error` | BC-3.1.001 postcond-1, EC-002 | AC-1 |
| `test_BC_3_1_001_cross_org_spec_isolation` | BC-3.1.001 postcond-3, inv-2 | AC-4 |
| `test_BC_3_1_001_two_orgs_same_sensor_name_no_collision` | BC-3.1.001 inv-1 | AC-2 |
| `test_BC_3_1_001_empty_registry_returns_err_not_panic` | BC-3.1.001 inv-3 | AC-3 |
| `test_BC_3_1_001_known_org_missing_sensor_returns_sensor_not_found` | BC-3.1.001 EC-002 | AC-1 |
| `test_BC_3_1_001_org_rename_preserves_spec_access` | BC-3.1.001 EC-003 | AC-4 |

## Stubs Created

- `crates/prism-spec-engine/src/org_scoped_store.rs` — `OrgScopedSpecStore` struct with `get_spec(slug, sensor)` stub; body is `todo!("S-3.1.05: implement OrgScopedSpecStore::get_spec")`
- `crates/prism-spec-engine/src/error.rs` — extended with `UnknownOrg`, `SensorNotFound`, `RegistryNotInitialized` variants; `SpecEngineError` marked `#[non_exhaustive]`
- `crates/prism-spec-engine/src/lib.rs` — re-exported `OrgScopedSpecStore`

## Red Gate Verification

All 7 tests FAIL with:
```
panicked at crates/prism-spec-engine/src/org_scoped_store.rs:
not yet implemented: S-3.1.05: implement OrgScopedSpecStore::get_spec
```

Failure is the correct `todo!()` panic — no spurious passes. All assertions are load-bearing.

```
test result: FAILED. 0 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out
```

## Regression Check

| Existing Tests | Status |
|---------------|--------|
| Prior prism-spec-engine tests | all pass — 0 regressions |

## Implementer Handoff

Implement `OrgScopedSpecStore::get_spec(&self, slug: &OrgSlug, sensor: &str)` to:
1. Resolve `slug` → `OrgId` via `self.registry.resolve(slug)` — return `Err(UnknownOrg)` on miss.
2. Look up `(org_id, sensor.to_string())` in internal `HashMap<(OrgId, String), SensorSpec>`.
3. Return `Ok(spec.clone())` on hit; `Err(SensorNotFound)` on miss.
4. Return `Err(RegistryNotInitialized)` if registry `Arc` is unset.

Semver: bump `prism-spec-engine` 0.2.0 → 0.3.0 (new public error variants + `#[non_exhaustive]`).
