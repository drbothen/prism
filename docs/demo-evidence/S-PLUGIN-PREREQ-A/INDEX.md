# Demo Evidence Index — S-PLUGIN-PREREQ-A

**Story:** SensorId(Arc<str>) open newtype replaces SensorType closed enum
**Branch:** `feature/S-PLUGIN-PREREQ-A`
**HEAD SHA:** `8b949bba9008b4e236e87320ddb51228e74911e0`
**Convergence:** 3/3 CLEAN through 12 adversarial passes
**Evidence captured:** 2026-05-11

---

## Story Status

| Field | Value |
|---|---|
| Convergence | 3/3 CLEAN (pass-10, pass-11, pass-12) |
| HEAD SHA | `8b949bba` |
| Branch | `feature/S-PLUGIN-PREREQ-A` → targets `develop` |
| Workspace build | CLEAN (`Finished dev profile in 19.48s`) |
| Red Gate tests | 6/6 PASS |

---

## AC Satisfaction Table

| AC# | Description | Status | Evidence File |
|---|---|---|---|
| AC-1 | `SensorId(Arc<str>)` newtype with full impl set | SATISFIED | [AC-1-evidence.md](AC-1-evidence.md) |
| AC-2 | `pub enum SensorType` deleted from prism-core | SATISFIED | [AC-2-evidence.md](AC-2-evidence.md) |
| AC-3 | `SensorAdapter::sensor_type` returns `SensorId` | SATISFIED | [AC-3-evidence.md](AC-3-evidence.md) |
| AC-4 | `AdapterRegistry` keyed by `(OrgId, SensorId)` | SATISFIED | [AC-4-evidence.md](AC-4-evidence.md) |
| AC-5 | All 7 dispatch sites use open dispatch | SATISFIED | [AC-5-evidence.md](AC-5-evidence.md) |
| AC-6 | Perimeter compile-fail test catches reintroduction | SATISFIED | [AC-6-evidence.md](AC-6-evidence.md) |
| AC-7 | No CustomAdapter calls depend on closed-enum dispatch | SATISFIED | [AC-7-evidence.md](AC-7-evidence.md) |
| AC-8 | Workspace build + crate test suites pass at HEAD | SATISFIED | [AC-8-evidence.md](AC-8-evidence.md) |
| AC-9 | prism-core unit tests: equality/hash/Display + Borrow<str> | SATISFIED | [AC-9-evidence.md](AC-9-evidence.md) |
| AC-10 | prism-sensors integration test: AdapterRegistry insert+lookup | SATISFIED | [AC-10-evidence.md](AC-10-evidence.md) |
| AC-11 | CustomAdapter NOT removed; registry uses `&str` (no SensorType) | SATISFIED | [AC-11-evidence.md](AC-11-evidence.md) |

**11/11 ACs SATISFIED**

---

## Red Gate Tests: 6/6 Passing

| Test Name | File:Line | Crate | Status |
|---|---|---|---|
| `test_BC_2_01_013_001_sensorid_from_str_roundtrip` | `crates/prism-core/src/sensor_id.rs:327` | prism-core | PASS |
| `test_BC_2_01_013_003_sensorid_hash_eq_invariant` | `crates/prism-core/src/sensor_id.rs:372` | prism-core | PASS |
| `test_BC_2_01_013_004_sensor_id_borrow_str_lookup` | `crates/prism-core/src/sensor_id.rs:396` | prism-core | PASS |
| `test_BC_2_01_013_004_adapter_registry_sensorid_insert_lookup` | `crates/prism-sensors/src/tests/bc_2_01_013_sensorid.rs:74` | prism-sensors | PASS |
| Perimeter `use prism_core::SensorType` → `E0432` | `tests/external/perimeter-violation/src/main.rs:69` | perimeter-violation | FAIL (expected) |
| `test_BC_2_01_013_005_sensorid_lookup_at_virtual_fields_dispatch` | `crates/prism-query/tests/sensorid_dispatch_redgate.rs:37` | prism-query | PASS |

Note: The perimeter-violation test is a compile-fail assertion. "FAIL (expected)"
means the crate fails to compile with `E0432` — that is the passing state. If it
compiled successfully, `SensorType` would have been reintroduced (regression).

---

## Compilation Status

```
$ cargo build --workspace --all-features --color=never
Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.48s
```

No errors. Deprecation warnings exist for unrelated legacy call sites
(`init_registry` deprecated in favor of `init_registry_for_org`).

---

## Test Suite Summary

| Crate | Tests Run | Passed | Skipped | Failed |
|---|---|---|---|---|
| prism-core | 235 | 235 | 0 | 0 |
| prism-sensors | 267 | 267 | 0 | 0 |
| prism-query | 896 | 896 | 6 | 0 |
| **Total** | **1,398** | **1,398** | **6** | **0** |

6 skipped tests are pre-existing integration tests requiring live sensor endpoints.

---

## Evidence Directory Contents

```
docs/demo-evidence/S-PLUGIN-PREREQ-A/
├── INDEX.md                (this file)
├── AC-1-evidence.md        SensorId newtype impl set
├── AC-2-evidence.md        SensorType enum deletion
├── AC-3-evidence.md        SensorAdapter::sensor_type → SensorId
├── AC-4-evidence.md        AdapterRegistry (OrgId, SensorId) key
├── AC-5-evidence.md        7 dispatch sites open dispatch
├── AC-6-evidence.md        Perimeter compile-fail E0432
├── AC-7-evidence.md        CustomAdapter no closed-enum dispatch
├── AC-8-evidence.md        Workspace build + test pass
├── AC-9-evidence.md        prism-core unit tests (3 Red Gate)
├── AC-10-evidence.md       prism-sensors integration test
└── AC-11-evidence.md       CustomAdapter preserved, &str key
```
