# Red Gate Log — S-3.1.06-ImplPhase

**Story:** S-3.1.06-ImplPhase — prism-sensors: complete adapter OrgId binding
**Phase:** Red Gate (Phase 1: Stubs)
**Date:** 2026-05-01
**Agent:** Stub Architect / Test Writer

---

## Summary

All stub changes have been applied. `cargo check --workspace` exits 0.
All test binaries compile (`cargo test -p prism-sensors --no-run` exits 0).
All 5 new `org_id_binding` tests fail at runtime — Red Gate verified.

---

## Cargo Check Results

```
cargo check --workspace: exit 0
  - Zero errors
  - Warnings only (unused variables in todo!() stubs — expected)
```

---

## Test Binary Build Results

```
cargo test -p prism-sensors --no-run: exit 0
  - All test binaries compile (prism-sensors lib test, integration tests)
  - Zero E0061 (wrong argument count) errors
```

---

## Red Gate: org_id_binding Tests (5/5 FAIL)

```
test result: FAILED. 0 passed; 5 failed; 0 ignored
```

| Test | Panic Site | Panic Message |
|------|-----------|---------------|
| `test_AC_001_init_registry_for_org_uses_org_id_in_signature` | `lib.rs:153` | `not yet implemented: AC-001: propagate org_id through adapter constructors` |
| `test_AC_002_adapter_registry_keyed_by_org_id_and_sensor_type` | `registry.rs:64` | `not yet implemented: AC-002: store adapter under (org_id, sensor_type) composite key` |
| `test_AC_003_org_id_mismatch_returns_typed_error` | `org_id_binding.rs:252` | `AC-003: error must be OrgIdMismatch with correct org IDs; got: Internal { .. }` (network attempt; guard not yet installed) |
| `test_AC_004_legacy_init_registry_deprecated_warning` | `lib.rs:121` | `not yet implemented: AC-005: legacy init_registry has no OrgId` |
| `test_AC_005_downstream_callers_migrate_to_init_registry_for_org` | `lib.rs:153` | `not yet implemented: AC-001: propagate org_id through adapter constructors` |

---

## Stub Changes Applied

### New stubs (test file)
- `crates/prism-sensors/tests/org_id_binding.rs` — CREATED

### Modified source files
- `crates/prism-sensors/src/lib.rs` — `_org_id` prefix removed; body replaced with `todo!()`
- `crates/prism-sensors/src/registry.rs` — HashMap rekeyed to `(OrgId, SensorType)`; `register`/`get` signatures updated; bodies `todo!()`
- `crates/prism-sensors/src/adapter.rs` — `SensorError::OrgIdMismatch` variant added; `is_transient()` updated
- `crates/prism-sensors/src/auth/crowdstrike.rs` — `org_id: OrgId` field + param added; constructor body includes field
- `crates/prism-sensors/src/auth/cyberint.rs` — `org_id: OrgId` field + param added
- `crates/prism-sensors/src/auth/claroty.rs` — `org_id: OrgId` field + param added
- `crates/prism-sensors/src/auth/armis.rs` — `org_id: OrgId` field + param added
- `crates/prism-sensors/src/fanout.rs` — `registry.get(target.org_id, target.sensor_type)` updated

### Modified test files (AC-006 migration)
- `crates/prism-sensors/tests/test_crowdstrike.rs` — `CrowdStrikeAdapter::new` + `test_org_id()` helper
- `crates/prism-sensors/tests/test_cyberint.rs` — `CyberintAdapter::new` + `test_org_id()` helper
- `crates/prism-sensors/tests/test_claroty.rs` — `ClarotyAdapter::new` + `test_org_id()` helper
- `crates/prism-sensors/tests/test_armis.rs` — `ArmisAdapter::new` + `test_org_id()` helper + registry.get 2-arg
- `crates/prism-sensors/tests/test_wgs_w2_001_aql_validator.rs` — `ArmisAdapter::new` updated
- `crates/prism-sensors/tests/test_wgs_w2_002_secretstring.rs` — `ArmisAdapter::new` + `ClarotyAdapter::new` + `CrowdStrikeAdapter::new` updated

### Modified internal test files
- `crates/prism-sensors/src/tests/bc_2_01_013.rs` — `register(org_id, adapter)` + `get(org_id, sensor_type)`
- `crates/prism-sensors/src/tests/bc_2_01_002.rs` — `register(org_id, adapter)` placeholder
- `crates/prism-sensors/src/tests/bc_2_01_010.rs` — `register(org_id, adapter)` placeholder

---

## Workarounds for Stub-Phase Compilability

1. **`init_registry` deprecated body**: The function now `todo!()`s because adapter constructors require `OrgId` and the legacy function has none. This makes `test_BC_2_01_008_init_registry_registers_armis_adapter` a RED test (runtime panic). The `#[deprecated]` attribute is confirmed present.

2. **`DEFAULT_ORG_ID_BYTES` inaccessible from integration tests**: The constant is `#[cfg(test)]` gated in lib.rs, which means external integration tests in `tests/` cannot access it. All `test_org_id()` helpers in integration test files inline the same byte values directly.

3. **`bc_2_01_013.rs` / `bc_2_01_002.rs` / `bc_2_01_010.rs`**: These use `OrgId::new()` as a placeholder first argument to `register()`. These tests will `todo!()` at runtime (registry stubs). Annotated with `// TODO impl-phase`.

4. **`test_AC_003` failure path**: The OrgId mismatch guard is not yet in `fetch()` (that's the implementation, not the stub). The test currently fails via a network error (the adapter attempts the real HTTP call before the guard would fire). This is the correct RED behavior — the guard doesn't exist yet.

---

## Flag for Implementation

- **E-SENSOR-060** must be added to `.factory/specs/prd-supplements/error-taxonomy.md` in the same PR. The variant is coded in `adapter.rs` but the error taxonomy doc has not been updated.
