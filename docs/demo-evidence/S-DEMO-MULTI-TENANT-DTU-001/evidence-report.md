---
document_type: demo-evidence-report
product: "prism-dtu-demo-server + prism-dtu-harness (S-DEMO-MULTI-TENANT-DTU-001)"
pipeline_run: "2026-06-14"
demo_type: library
recording_tool: vhs
story_id: S-DEMO-MULTI-TENANT-DTU-001
bc: BC-2.06.017
status: complete
---

# Demo Evidence Report — S-DEMO-MULTI-TENANT-DTU-001

**Story:** Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing
**BC:** BC-2.06.017
**Product type:** Test infrastructure / library (no UI, no standalone CLI)
**Recording tool:** VHS 0.10.0 (terminal capture of `cargo nextest` runs)
**Feature branch:** `feature/S-DEMO-MULTI-TENANT-DTU-001`
**Test count:** 19 tests across 2 binaries — all PASS

---

## Per-AC Demo Recordings

| AC | Description | Demonstrating Test(s) | Observed Behavior | Artifact |
|----|-------------|----------------------|-------------------|----------|
| AC-001 | `MultiInstanceConfig` accepted; `start_instances` returns non-empty socket map | `test_BC_2_06_017_demo_server_multi_instance_bind_config_accepted` | PASS 0.047s — config accepted, socket_map non-empty, no error | [AC-001-004-007 .gif](AC-001-004-007-multi-instance-bind-and-compat.gif) / [.webm](AC-001-004-007-multi-instance-bind-and-compat.webm) |
| AC-002 | Two armis instances start on distinct ephemeral ports; `map["armis-acme"] != map["armis-contoso"]` | `test_BC_2_06_017_demo_server_two_armis_instances_bind_distinct_ports`, `test_BC_2_06_017_demo_server_instance_a_responds_independently`, `test_BC_2_06_017_demo_server_instance_b_responds_independently`, `test_BC_2_06_017_demo_server_multi_instance_shutdown_clean` | PASS — two distinct SocketAddrs, each instance responds to its own address only, shutdown releases ports | [AC-001-004-007 .gif](AC-001-004-007-multi-instance-bind-and-compat.gif) / [.webm](AC-001-004-007-multi-instance-bind-and-compat.webm) |
| AC-003 | Two claroty instances start on distinct ephemeral ports; independent `POST /api/v1/alerts/` responses | `test_BC_2_06_017_demo_server_two_claroty_instances_bind_distinct_ports` | PASS 0.047s — two distinct SocketAddrs, each Claroty instance returns independent fixture data | [AC-001-004-007 .gif](AC-001-004-007-multi-instance-bind-and-compat.gif) / [.webm](AC-001-004-007-multi-instance-bind-and-compat.webm) |
| AC-004 | `MultiInstanceHarness.start(entries)` returns `HashMap<(String,String),SocketAddr>` keyed by plain `(org_slug, sensor_id)` strings | `test_BC_2_06_017_harness_multi_instance_builds_per_org_socket_map`, `test_BC_2_06_017_harness_distinct_org_slots_different_sockets` | PASS 0.050s — socket_map contains correct (org_slug, sensor_id) String key pairs; two orgs yield two distinct SocketAddrs | [AC-001-004-007 .gif](AC-001-004-007-multi-instance-bind-and-compat.gif) / [.webm](AC-001-004-007-multi-instance-bind-and-compat.webm) |
| AC-005 | `write_overlay_temp_dir` produces 3-field TOML overlays; `ResolvedSensorSpec` for `(acme, armis)` and `(contoso, armis)` carry distinct `base_url` values | `test_BC_2_06_017_multi_instance_overlay_loads_distinct_base_urls` | PASS 0.061s — overlays written with `extends`, `instance_id`, `base_url`; `SpecLoader::load_all` returns distinct base_urls for each org; base_url-only overlay would fail OverlayLoader validation | [AC-005-006 .gif](AC-005-006-overlay-and-isolation-proof.gif) / [.webm](AC-005-006-overlay-and-isolation-proof.webm) |
| AC-006 | All acme requests reach instance A; all contoso requests reach instance B; zero cross-tenant leakage (server-side `AtomicU64` counter proof via `/dtu/request-count` endpoint on `prism-dtu-armis`) | `test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage`, `test_BC_2_06_017_multi_tenant_routing_acme_instance_receives_acme_requests`, `test_BC_2_06_017_multi_tenant_routing_contoso_instance_receives_contoso_requests` | PASS 0.036s — server-side request counters confirm instance A received 0 requests from org B; instance B received 0 requests from org A; counts are independent and match dispatch counts exactly | [AC-005-006 .gif](AC-005-006-overlay-and-isolation-proof.gif) / [.webm](AC-005-006-overlay-and-isolation-proof.webm) |
| AC-007 | Existing single-instance `ArmisClone::start_on` path unchanged; all S-DEMO-002-style tests pass | `test_BC_2_06_017_single_instance_path_unaffected_by_multi_instance_addition` (harness), `test_BC_2_06_017_single_instance_parity_test_still_passes_after_multi_instance_addition` (demo-server) | PASS 5.057s / 5.062s — single-instance `start_on(&mut self, bind, shutdown, tls)` signature not modified; existing patterns compile and run unchanged | [AC-001-004-007 .gif](AC-001-004-007-multi-instance-bind-and-compat.gif) / [.webm](AC-001-004-007-multi-instance-bind-and-compat.webm) |
| AC-008 | Module-doc and pub API documentation complete; no undocumented public types; `#[non_exhaustive]` on all new public types | Verified by inspection: `//!` module docs in `multi_instance.rs` (both crates); `///` doc comments on all pub types and methods; `#[non_exhaustive]` on `MultiInstanceConfig`, `InstanceEntry`, `MultiInstanceServers`, `MultiInstanceBindError`, `DemoBindError`, `MultiInstanceHarness`, `HarnessEntry`, `BindError`; `EXPECTED` bumped `52→60` in `ci.yml` | Doc completeness confirmed. All 8 new public types carry `#[non_exhaustive]`. The non-exhaustive compile-fail gate at `tests/external/non-exhaustive-violation/` counts 60 expected violations (7 E0639 struct-literal + 1 E0004 match-arm for this story, added to prior 52). | No runtime test — inspection artifact only |
| AC-009 | SAP-1 sweep: zero uncatalogued `event_type` emissions in `prism-dtu-demo-server`, `prism-dtu-harness` | `rg 'event_type\s*=' crates/prism-dtu-demo-server crates/prism-dtu-harness crates/prism-dtu-armis --type rust` → exit code 1 (no matches) | Zero new `event_type =` emissions added by this story. SAP-1 sweep clean. No BC-2.16.002 catalog rows required. | No runtime test — sweep artifact only |

---

## Error Path Recordings

| Error Case | EC ID | Demonstrating Test(s) | Observed Behavior | Artifact |
|------------|-------|----------------------|-------------------|----------|
| Bind failure aggregated (EADDRINUSE) — all errors collected before returning | EC-001 | `test_BC_2_06_017_demo_server_bind_failure_aggregates_all_errors`, `test_BC_2_06_017_harness_bind_failure_aggregates_all_errors` | PASS — `MultiInstanceBindError::BindFailure(Vec<DemoBindError>)` and `HarnessError::BindFailure(Vec<BindError>)` returned with all failed instances listed; no short-circuit on first error | [AC-ERROR .gif](AC-ERROR-bind-failure-and-duplicate-key.gif) / [.webm](AC-ERROR-bind-failure-and-duplicate-key.webm) |
| Zero instances config → empty socket map, no error | EC-002 | `test_BC_2_06_017_demo_server_zero_instances_returns_empty_map` | PASS — `Ok(MultiInstanceServers)` with empty `socket_map()` returned; no panic | [AC-ERROR .gif](AC-ERROR-bind-failure-and-duplicate-key.gif) / [.webm](AC-ERROR-bind-failure-and-duplicate-key.webm) |
| Duplicate `(org_slug, sensor_id)` in harness → `DuplicateKey` error before any binds | EC-003 | `test_BC_2_06_017_harness_duplicate_key_returns_error` | PASS — `HarnessError::DuplicateKey { org_slug, sensor_id }` returned immediately; no clone instances started | [AC-ERROR .gif](AC-ERROR-bind-failure-and-duplicate-key.gif) / [.webm](AC-ERROR-bind-failure-and-duplicate-key.webm) |
| Duplicate `name` in demo-server config → `DuplicateName` error before any binds | EC-009 | `test_BC_2_06_017_demo_server_duplicate_instance_name_returns_error` | PASS — `MultiInstanceBindError::DuplicateName { name }` returned before any bind attempts | [AC-ERROR .gif](AC-ERROR-bind-failure-and-duplicate-key.gif) / [.webm](AC-ERROR-bind-failure-and-duplicate-key.webm) |

---

## Full Suite Recording

| Demo | Tests | Description | Artifact |
|------|-------|-------------|----------|
| Full 19-test suite | 19 / 19 PASS | All multi-instance tests in `multi_instance_tests` + `multi_instance_harness_tests` binaries | [AC-ALL .gif](AC-ALL-full-suite.gif) / [.webm](AC-ALL-full-suite.webm) |

---

## AC Coverage Summary

| AC | Status | Evidence type |
|----|--------|---------------|
| AC-001 | COVERED | VHS recording — runtime test PASS |
| AC-002 | COVERED | VHS recording — runtime tests PASS (distinct ports, independent responses, clean shutdown) |
| AC-003 | COVERED | VHS recording — runtime test PASS |
| AC-004 | COVERED | VHS recording — runtime tests PASS (per-org socket map, distinct SocketAddrs) |
| AC-005 | COVERED | VHS recording — runtime test PASS (overlay loads distinct base_urls) |
| AC-006 | COVERED | VHS recording — runtime tests PASS (server-side counter proof, zero leakage) |
| AC-007 | COVERED | VHS recording — runtime tests PASS (backward compat, 5s shutdown unchanged) |
| AC-008 | COVERED | Inspection — rustdoc present on all pub types/methods; `#[non_exhaustive]` on 8 types; ci.yml EXPECTED=60 |
| AC-009 | COVERED | Inspection — SAP-1 sweep clean; `rg 'event_type\s*='` returns zero results |

All 9 ACs covered. 19 runtime tests pass. Both success paths and error paths demonstrated.

---

## Toolchain

| Tool | Version | Status |
|------|---------|--------|
| VHS | 0.10.0 | installed (`/opt/homebrew/bin/vhs`) |
| cargo nextest | workspace pin | installed |
| FiraCode Nerd Font Mono | system | installed (`/Users/jmagady/Library/Fonts/FiraCodeNerdFontMono-Medium.ttf`) |

**Note on VHS syntax:** VHS 0.10.0 does not support `Wait+Line`. Recordings use `Sleep` with conservative durations (5–12s) calibrated against actual test runtimes (fast tests: <0.1s; single-instance backward-compat tests: ~5s each).

---

## PR Embedding Snippet

```markdown
## Demo Evidence

Full 19-test suite — all PASS:

![S-DEMO-MULTI-TENANT-DTU-001 full suite](docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-ALL-full-suite.gif)

AC-005 overlay loads + AC-006 zero cross-tenant leakage (isolation proof):

![AC-005/006 isolation proof](docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-005-006-overlay-and-isolation-proof.gif)

Error paths (bind failure aggregation + duplicate key rejection):

![Error paths](docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-ERROR-bind-failure-and-duplicate-key.gif)
```
