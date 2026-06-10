# Demo Evidence Report — S-DEMO-DTU-LIVE-SCENARIO-001-A

**Story:** Baseline Seeding Retrofit — Wire Seeded Generators into Demo-Server Clones for Per-Client Distinct Data
**Branch:** `feature/S-DEMO-DTU-LIVE-SCENARIO-001-A`
**HEAD at recording:** 3e5179a2
**`just check` result:** GREEN (4141+ tests)
**Recorded:** 2026-06-10

## Modality

**CLI product — VHS terminal recordings + captured test output.**

Three VHS recordings were produced (`.gif` + `.webm` per AC group) alongside this evidence report.
For each acceptance criterion the corresponding nextest assertion and observed output are shown verbatim.

## Evidence Files

| File | Description |
|------|-------------|
| `AC-001-005-seeding-disjoint-ids.gif` | VHS recording: demo_seeding binary — disjoint ID side-by-side |
| `AC-001-005-seeding-disjoint-ids.webm` | VHS recording (archival) |
| `AC-001-005-seeding-disjoint-ids.tape` | VHS script source |
| `AC-007-015-017-archetype-differential.gif` | VHS recording: Red Gate tests 15/16/17 running green |
| `AC-007-015-017-archetype-differential.webm` | VHS recording (archival) |
| `AC-007-015-017-archetype-differential.tape` | VHS script source |
| `AC-008-009-010-error-codes.gif` | VHS recording: E-DEMO-001/004/005 construction-time failures |
| `AC-008-009-010-error-codes.webm` | VHS recording (archival) |
| `AC-008-009-010-error-codes.tape` | VHS script source |
| `evidence-report.md` | This file |

Demo binary source: `crates/prism-dtu-crowdstrike/examples/demo_seeding.rs`

---

## AC Coverage Map

### AC-001 — ScenarioEntityCatalog constructed from (seed, org_id) via secondary RNG stream

**Red Gate:** `test_BC_2_06_018_scenario_catalog_secondary_rng_and_canonical_ids`
**Crate:** `prism-dtu-common`
**Command:**
```
cargo nextest run -p prism-dtu-common --features fixture-gen -E 'test(BC_2_06_018)'
```
**Observed output:**
```
Starting 2 tests across 2 binaries (45 tests skipped)
    PASS [   0.012s] (1/2) prism-dtu-common scenario::tests::test_BC_2_06_018_scenario_catalog_secondary_rng_and_canonical_ids
    PASS [   0.012s] (2/2) prism-dtu-common scenario::tests::test_BC_2_06_018_org_slug_from_org_id_canonical_format
 Summary [   0.013s] 2 tests run: 2 passed, 45 skipped
```
**Assertion:** `ScenarioEntityCatalog` has `org_slug = "deadbeef"` for OrgId bytes `[0xde, 0xad, 0xbe, 0xef, ...]`, and `primary_device_id_cs = "dev-deadbeef-42-0"`. `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves` all non-empty from secondary RNG stream `gen_seeded_rng(seed.wrapping_add(1), &org_id)`.

---

### AC-002 — org_slug_from_org_id produces canonical 8-hex-char slug

**Red Gate:** `test_BC_2_06_018_org_slug_from_org_id_canonical_format`
**Crate:** `prism-dtu-common`
**Command:** (same run as AC-001 above)
**Observed output:** PASS (see AC-001 block)
**Assertion:** `org_slug_from_org_id(&OrgId([0xde, 0xad, 0xbe, 0xef, ...]))` returns `"deadbeef"` (exactly 8 lowercase hex chars). Formula verified: `hex(org_id.as_bytes()[0..4])`.

---

### AC-003 — new_with_seed forwarded to CrowdStrike clone

**Red Gate:** `test_BC_2_06_018_crowdstrike_new_with_seed_forwarded`
**Crate:** `prism-dtu-crowdstrike`
**Command:**
```
cargo nextest run -p prism-dtu-crowdstrike --features fixture-gen -E 'test(BC_2_06_018)'
```
**Observed output:**
```
Starting 2 tests across 5 binaries (45 tests skipped)
    PASS [   0.010s] (1/2) prism-dtu-crowdstrike::bc_2_06_018_new_with_seed test_BC_2_06_018_crowdstrike_new_with_seed_forwarded
    PASS [   0.011s] (2/2) prism-dtu-crowdstrike::bc_2_06_018_new_with_seed test_BC_2_06_018_crowdstrike_new_with_seed_disjoint_id_prefixes
 Summary [   0.011s] 2 tests run: 2 passed, 45 skipped
```
**Assertion:** `CrowdstrikeClone::new_with_seed(100, Archetype::CompromisedEndpoint, org_a)` populates `state.generated_devices` (non-empty) and `state.generated_detections` (non-empty). All device IDs start with `"dev-deadbeef-100-"`.

---

### AC-004 — new_with_seed forwarded to Armis clone; org_slug derived internally (fallible)

**Red Gate:** `test_BC_2_06_018_armis_new_with_seed_canonical_3arg`
**Crate:** `prism-dtu-armis`
**Command:**
```
cargo nextest run -p prism-dtu-armis --features 'fixture-gen dtu' -E 'test(BC_2_06_018)'
```
**Observed output:**
```
Starting 3 tests across 24 binaries (184 tests skipped)
    PASS [   0.011s] (1/3) prism-dtu-armis::bc_2_06_018_new_with_seed test_BC_2_06_018_armis_new_with_seed_canonical_3arg
    PASS [   0.012s] (2/3) prism-dtu-armis::bc_2_06_018_new_with_seed test_BC_2_06_018_armis_new_with_seed_fallibility_consistent_with_new
    PASS [   0.012s] (3/3) prism-dtu-armis::bc_2_06_018_new_with_seed test_BC_2_06_018_armis_new_with_seed_disjoint_ids
 Summary [   0.012s] 3 tests run: 3 passed, 184 skipped
```
**Assertion:** `ArmisClone::new_with_seed(100, Archetype::CompromisedEndpoint, org_id) -> anyhow::Result<Self>` succeeds; `state.generated_records` non-empty; asset_ids contain `"dev-deadbeef-100-"`. `new_with_seed` type matches `new() -> anyhow::Result<Self>` (fallibility consistent).

---

### AC-005 — INV-DISTINCT-DATA-001: disjoint ID sets for distinct seeds

**Red Gate:** `test_BC_2_06_018_distinct_seeds_disjoint_ids`
**Crate:** `prism-dtu-demo-server`
**Recording:** `AC-001-005-seeding-disjoint-ids.gif` / `.webm`
**Demo binary output:**

```
--- DEMO 1: INV-DISTINCT-DATA-001 (Disjoint IDs Across Clients) ---
Client A: seed=100, org_slug=deadbeef  (first 4 org bytes: de ad be ef)
Client B: seed=200, org_slug=cafebabe  (first 4 org bytes: ca fe ba be)

Client A device IDs (first 5 of 50):
  dev-deadbeef-100-0
  dev-deadbeef-100-1
  dev-deadbeef-100-10
  dev-deadbeef-100-11
  dev-deadbeef-100-12

Client B device IDs (first 5 of 50):
  dev-cafebabe-200-0
  dev-cafebabe-200-1
  dev-cafebabe-200-10
  dev-cafebabe-200-11
  dev-cafebabe-200-12

PASS  IDs are pairwise-disjoint (ids_A ∩ ids_B = empty set)
      INV-DISTINCT-DATA-001 holds: 50 Client A IDs, 50 Client B IDs, 0 shared
```

**Assertion:** `ids_A ∩ ids_B = ∅`. Clients with distinct `(seed=100, org_slug=deadbeef)` and `(seed=200, org_slug=cafebabe)` produce zero overlapping device IDs. The canonical format `"dev-{8hex}-{seed}-{n}"` guarantees structural disjointness.

**nextest gate:**
```
PASS [   0.021s] prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_distinct_seeds_disjoint_ids
```

---

### AC-006 — Backward compat: new() static-JSON path unchanged

**Red Gate:** `test_BC_2_06_018_backward_compat_seed42_default`
**Crate:** `prism-dtu-demo-server`
**Command:** (see full 9-test run below)
**Assertion:** `build_clone_pairs(&DemoConfig::default())` (seed=42, no org_id) succeeds, returns 6 clone pairs (crowdstrike, armis, claroty, cyberint, threatintel, nvd). Static-JSON path byte-identical to pre-Story-A.

```
PASS [   0.032s] prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_backward_compat_seed42_default
```

---

### AC-007 — INV-FIXTURE-SET-ARCHETYPE-MAP-001: all 8 fixture_set values map correctly and archetype drives output

**Red Gate:** `test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error`
**Additional Red Gates:** 15, 16, 17 (dormant empty, large_scale count, differential)
**Recording:** `AC-007-015-017-archetype-differential.gif` / `.webm`
**Full nextest run:**

```
Starting 9 tests across 17 binaries (34 tests skipped)
    PASS [   0.014s] (1/9) prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction
    PASS [   0.014s] (2/9) prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_e_demo_004_absent_org_id_at_construction
    PASS [   0.014s] (3/9) prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_e_demo_001_at_construction_not_request_time
    PASS [   0.021s] (4/9) prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_distinct_seeds_disjoint_ids
    PASS [   0.032s] (5/9) prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_backward_compat_seed42_default
    PASS [   0.038s] (6/9) prism-dtu-demo-server::bc_2_06_018_archetype_differential test_BC_2_06_018_dormant_archetype_empty_served_response
    PASS [   0.040s] (7/9) prism-dtu-demo-server::bc_2_06_018_archetype_differential test_BC_2_06_018_archetype_drives_served_output_differential
    PASS [   0.101s] (8/9) prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error
    PASS [   0.503s] (9/9) prism-dtu-demo-server::bc_2_06_018_archetype_differential test_BC_2_06_018_large_scale_archetype_record_count
 Summary [   0.504s] 9 tests run: 9 passed, 34 skipped
```

**Assertions verified:**
- All 8 valid fixture_set strings (`default`, `compromised`, `auth_outage`, `large_scale`, `pagination_edges`, `schema_drift`, `high_churn`, `dormant`) return `Ok(...)` from `build_clone_pairs`.
- `fixture_set="xyzzy_unknown"` returns `Err` containing `"E-DEMO-001"`.

---

### AC-008 — E-DEMO-001 at construction, not request time

**Red Gate:** `test_BC_2_06_018_e_demo_001_at_construction_not_request_time`
**Recording:** `AC-008-009-010-error-codes.gif` / `.webm`
**Observed:**
```
PASS [   0.010s] prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_e_demo_001_at_construction_not_request_time
```
**Assertion:** `build_clone_pairs` with `fixture_set="totally_invalid_value"` returns `Err(e)` where `e.to_string()` contains `"E-DEMO-001"`, `"crowdstrike"`, and `"totally_invalid_value"`.

**Error path summary:**
```
build_clone_pairs(fixture_set="totally_invalid_value")
  -> Err("E-DEMO-001: crowdstrike: unknown fixture_set 'totally_invalid_value'")
```

---

### AC-009 — E-DEMO-004: missing org_id fails at construction

**Red Gate:** `test_BC_2_06_018_e_demo_004_absent_org_id_at_construction`
**Observed:**
```
PASS [   0.010s] prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_e_demo_004_absent_org_id_at_construction
```
**Assertion:** `build_clone_pairs(fixture_set="compromised", org_id=None)` returns `Err(e)` containing `"E-DEMO-004"` and `"crowdstrike"`. Error surfaces before any clone constructor is called.

**Error path summary:**
```
build_clone_pairs(fixture_set="compromised", org_id=None)
  -> Err("E-DEMO-004: crowdstrike: org_id required for non-default fixture_set")
```

---

### AC-010 — E-DEMO-005: invalid UUID in org_id fails at construction

**Red Gate:** `test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction`
**Observed:**
```
PASS [   0.010s] prism-dtu-demo-server::bc_2_06_018_seeding test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction
```
**Assertion:** `build_clone_pairs(org_id=Some("not-a-valid-uuid"), fixture_set="compromised")` returns `Err(e)` containing `"E-DEMO-005"`, `"crowdstrike"`, and `"not-a-valid-uuid"`. No clone constructor is called.

**Error path summary:**
```
build_clone_pairs(org_id="not-a-valid-uuid")
  -> Err("E-DEMO-005: crowdstrike: invalid org_id UUID 'not-a-valid-uuid'")
```

---

### AC-011 — new_with_seed forwarded to Claroty clone (infallible)

**Red Gate:** `test_BC_2_06_018_claroty_new_with_seed_forwarded`
**Crate:** `prism-dtu-claroty`
**Command:**
```
cargo nextest run -p prism-dtu-claroty --features 'fixture-gen dtu' -E 'test(BC_2_06_018)'
```
**Observed output:**
```
Starting 3 tests across 23 binaries (205 tests skipped)
    PASS [   0.010s] (1/3) prism-dtu-claroty::bc_2_06_018_new_with_seed test_BC_2_06_018_claroty_new_with_seed_forwarded
    PASS [   0.011s] (2/3) prism-dtu-claroty::bc_2_06_018_new_with_seed test_BC_2_06_018_claroty_new_with_seed_disjoint_ids
    PASS [   0.011s] (3/3) prism-dtu-claroty::bc_2_06_018_new_with_seed test_BC_2_06_018_claroty_new_with_seed_deterministic
 Summary [   0.011s] 3 tests run: 3 passed, 205 skipped
```
**Assertion:** `ClarotyClone::new_with_seed(seed, archetype, org_id) -> Self` (infallible) constructs successfully; route handlers serve from `generated_records`.

---

### AC-012 — new_with_seed forwarded to Cyberint clone (fallible)

**Red Gate:** `test_BC_2_06_018_cyberint_new_with_seed_forwarded_fallible`
**Crate:** `prism-dtu-cyberint`
**Command:**
```
cargo nextest run -p prism-dtu-cyberint --features 'fixture-gen dtu' -E 'test(BC_2_06_018)'
```
**Observed output:**
```
Starting 3 tests across 22 binaries (158 tests skipped)
    PASS [   0.011s] (1/3) prism-dtu-cyberint::bc_2_06_018_new_with_seed test_BC_2_06_018_cyberint_new_with_seed_forwarded_fallible
    PASS [   0.012s] (2/3) prism-dtu-cyberint::bc_2_06_018_new_with_seed test_BC_2_06_018_cyberint_new_with_seed_fallibility_consistent_with_new
    PASS [   0.012s] (3/3) prism-dtu-cyberint::bc_2_06_018_new_with_seed test_BC_2_06_018_cyberint_new_with_seed_disjoint_ids
 Summary [   0.012s] 3 tests run: 3 passed, 158 skipped
```
**Assertion:** `CyberintClone::new_with_seed(seed, archetype, org_id) -> anyhow::Result<Self>` (fallible, mirrors `new()`). Construction succeeds; error propagates consistently.

---

### AC-013 — fixture-gen feature additions compile cleanly for threatintel + nvd

**Red Gate:** compile-time gate (INV-PERIMETER-001)
**Commands:**
```
cargo build -p prism-dtu-threatintel --features fixture-gen -q  # exit 0
cargo build -p prism-dtu-nvd --features fixture-gen -q          # exit 0
cd tests/external/perimeter-violation && cargo test             # 33 E0432/E0603 errors (expected)
```
**Observed:**
- `prism-dtu-threatintel --features fixture-gen`: exit code 0 (compiles cleanly, no new violations)
- `prism-dtu-nvd --features fixture-gen`: exit code 0 (compiles cleanly, no new violations)
- Perimeter violation gate: 33 E0432/E0603 compile errors (these ARE the gate — the errors prove the perimeter is intact; the gate script exits 0 because error count matches expected)

**Assertion:** Adding `fixture-gen = ["prism-dtu-common/fixture-gen"]` to both crates introduces zero new perimeter violations (no `prism-spec-engine`/`prism-sensors`/`prism-query` deps introduced).

---

### AC-014 — ci.yml EXPECTED bumped atomically for new #[non_exhaustive] types

**Gate:** compile-fail gate (`tests/external/non-exhaustive-violation/`)
**Command:**
```
cargo test -p non-exhaustive-violation --manifest-path tests/external/non-exhaustive-violation/Cargo.toml
```
**Observed:**
```
error: could not compile `non-exhaustive-violation` (bin "non-exhaustive-violation" test)
       due to 50 previous errors; 5 warnings emitted
```
**ci.yml value:** `EXPECTED=50` (bumped from 49 for `ScenarioEntityCatalog`)

**Assertion:** `ScenarioEntityCatalog` is `#[non_exhaustive]` in `prism-dtu-common/src/scenario/`. The non-exhaustive violation count is 50, matching `EXPECTED=50` in `ci.yml`. Gate exits 0 (count >= EXPECTED).

---

## Red Gate test 15 (dormant archetype empty served response)

**Test:** `test_BC_2_06_018_dormant_archetype_empty_served_response`
**Crate:** `prism-dtu-demo-server` (route-level, via Claroty live axum server)
**Recording:** `AC-007-015-017-archetype-differential.gif`

**What it proves (load-bearing):** A `DormantTenant` archetype clone serves `$.total = 0` on both `POST /api/v1/devices` and `POST /api/v1/alerts`. Red-proof was verified: when `ClarotyClone::new_with_seed` was temporarily patched to hardcode `CompromisedEndpoint`, the assertion `device_total == 0` failed (got 50). Fix restored. Test PASSES.

```
PASS [   0.038s] prism-dtu-demo-server::bc_2_06_018_archetype_differential test_BC_2_06_018_dormant_archetype_empty_served_response
```

**Demo binary evidence (dormant path):**
```
--- DEMO 2: Dormant Archetype -> Empty Response (BC-2.06.018 EC-018-003) ---
Client C: seed=42, org=deadbeef, fixture_set=dormant (DormantTenant)
PASS  DormantTenant: generated_devices=0, generated_detections=0
      Route handlers will serve 0 devices and 0 detections
```

---

## Red Gate test 16 (large_scale archetype record count)

**Test:** `test_BC_2_06_018_large_scale_archetype_record_count`
**Crate:** `prism-dtu-demo-server` (route-level)
**What it proves:** `LargeScale` archetype produces exactly 10,000 device records (baseline read directly from `crates/prism-dtu-claroty/src/generator.rs` `gen_large_scale` branch). If hardcoded to `CompromisedEndpoint`, total would be 50, not 10,000.

```
PASS [   0.503s] prism-dtu-demo-server::bc_2_06_018_archetype_differential test_BC_2_06_018_large_scale_archetype_record_count
```

---

## Red Gate test 17 (archetype drives served output differential)

**Test:** `test_BC_2_06_018_archetype_drives_served_output_differential`
**Crate:** `prism-dtu-demo-server` (route-level)
**What it proves:** Same `(seed=42, org=deadbeef)` but `Archetype::CompromisedEndpoint` vs `Archetype::DormantTenant` → different `$.total` values (50 vs 0). The `device_total_compromised != device_total_dormant` assertion would fail if archetype was hardcoded. Red-proof confirmed.

```
PASS [   0.040s] prism-dtu-demo-server::bc_2_06_018_archetype_differential test_BC_2_06_018_archetype_drives_served_output_differential
```

---

## Complete Test Run Summary

All 17 Red Gate tests (plus additional sub-tests) GREEN:

| Test # | Name | Crate | Result |
|--------|------|-------|--------|
| 1 | `test_BC_2_06_018_scenario_catalog_secondary_rng_and_canonical_ids` | prism-dtu-common | PASS |
| 2 | `test_BC_2_06_018_org_slug_from_org_id_canonical_format` | prism-dtu-common | PASS |
| 3 | `test_BC_2_06_018_crowdstrike_new_with_seed_forwarded` | prism-dtu-crowdstrike | PASS |
| 3b | `test_BC_2_06_018_crowdstrike_new_with_seed_disjoint_id_prefixes` | prism-dtu-crowdstrike | PASS |
| 4 | `test_BC_2_06_018_armis_new_with_seed_canonical_3arg` | prism-dtu-armis | PASS |
| 4b | `test_BC_2_06_018_armis_new_with_seed_fallibility_consistent_with_new` | prism-dtu-armis | PASS |
| 4c | `test_BC_2_06_018_armis_new_with_seed_disjoint_ids` | prism-dtu-armis | PASS |
| 5 | `test_BC_2_06_018_distinct_seeds_disjoint_ids` | prism-dtu-demo-server | PASS |
| 6 | `test_BC_2_06_018_backward_compat_seed42_default` | prism-dtu-demo-server | PASS |
| 7 | `test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error` | prism-dtu-demo-server | PASS |
| 8 | `test_BC_2_06_018_e_demo_001_at_construction_not_request_time` | prism-dtu-demo-server | PASS |
| 9 | `test_BC_2_06_018_e_demo_004_absent_org_id_at_construction` | prism-dtu-demo-server | PASS |
| 10 | `test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction` | prism-dtu-demo-server | PASS |
| 11 | `test_BC_2_06_018_claroty_new_with_seed_forwarded` | prism-dtu-claroty | PASS |
| 11b | `test_BC_2_06_018_claroty_new_with_seed_disjoint_ids` | prism-dtu-claroty | PASS |
| 11c | `test_BC_2_06_018_claroty_new_with_seed_deterministic` | prism-dtu-claroty | PASS |
| 12 | `test_BC_2_06_018_cyberint_new_with_seed_forwarded_fallible` | prism-dtu-cyberint | PASS |
| 12b | `test_BC_2_06_018_cyberint_new_with_seed_fallibility_consistent_with_new` | prism-dtu-cyberint | PASS |
| 12c | `test_BC_2_06_018_cyberint_new_with_seed_disjoint_ids` | prism-dtu-cyberint | PASS |
| 13 | perimeter-violation compile-fail gate | tests/external/perimeter-violation | PASS |
| 14 | non-exhaustive-violation gate (EXPECTED=50) | tests/external/non-exhaustive-violation | PASS |
| 15 | `test_BC_2_06_018_dormant_archetype_empty_served_response` | prism-dtu-demo-server | PASS |
| 16 | `test_BC_2_06_018_large_scale_archetype_record_count` | prism-dtu-demo-server | PASS |
| 17 | `test_BC_2_06_018_archetype_drives_served_output_differential` | prism-dtu-demo-server | PASS |

**Total: 24 tests, 24 PASS, 0 FAIL**

---

## Success and Error Path Coverage

| Behavior | Path | Evidence |
|----------|------|----------|
| Disjoint IDs (INV-DISTINCT-DATA-001) | Success | AC-005 demo binary + Red Gate 5 |
| org_slug canonical format | Success | AC-002 unit test |
| DormantTenant = empty response | Success | Red Gate 15 + Demo 2 output |
| LargeScale = 10,000 devices | Success | Red Gate 16 |
| Archetype differential (compromised vs dormant) | Success | Red Gate 17 |
| Backward compat (no org_id, seed=42, default) | Success | Red Gate 6 |
| E-DEMO-001 (bad fixture_set) | Error | Red Gate 8 |
| E-DEMO-004 (missing org_id) | Error | Red Gate 9 |
| E-DEMO-005 (invalid UUID) | Error | Red Gate 10 |
| Perimeter intact after fixture-gen additions | Success | AC-013 compile gate |
| EXPECTED=50 after ScenarioEntityCatalog | Success | AC-014 compile gate |
