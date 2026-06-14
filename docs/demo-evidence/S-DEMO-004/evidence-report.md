# Demo Evidence Report — S-DEMO-004

**Story:** S-DEMO-004 — Multi-Org × Multi-Sensor Isolation Smoke Test
**Version:** v1.10
**Branch:** feature/S-DEMO-004
**HEAD:** 10769aac
**Recorded:** 2026-06-14
**Recorder:** vsdd-factory:demo-recorder

---

## Headline Result

10/10 e2e-multi-org tests PASS.

```
cargo nextest run -p prism-bin --profile e2e-multi-org --run-ignored ignored-only

────────────
 Nextest run ID 16d0c3b1-49a4-423e-86dc-746b8145dc07 with nextest profile: e2e-multi-org
    Starting 10 tests across 1 binary (18 binaries skipped via profile.e2e-multi-org.default-filter)
        PASS [   0.021s] ( 1/10) prism-bin::e2e_multi_org test_BC_2_22_001_multi_org_tests_ignored_under_standard_profile
        PASS [   0.527s] ( 2/10) prism-bin::e2e_multi_org test_BC_3_2_001_cross_org_query_returns_isolation_error
        PASS [   0.572s] ( 3/10) prism-bin::e2e_multi_org test_BC_2_22_001_multi_org_boot_registers_8_adapters
        PASS [   0.580s] ( 4/10) prism-bin::e2e_multi_org test_BC_2_06_014_org_b_claroty_query_returns_data
        PASS [   0.587s] ( 5/10) prism-bin::e2e_multi_org test_BC_3_2_001_cyberint_session_isolation_org_b_and_org_c
        PASS [   0.638s] ( 6/10) prism-bin::e2e_multi_org test_BC_2_09_008_response_envelope_identifies_correct_org_and_sensor
        PASS [   0.638s] ( 7/10) prism-bin::e2e_multi_org test_BC_2_06_014_org_a_crowdstrike_query_returns_data
        PASS [   0.672s] ( 8/10) prism-bin::e2e_multi_org test_BC_2_01_013_org_c_all_4_sensors_return_independent_data
        PASS [   0.749s] ( 9/10) prism-bin::e2e_multi_org test_BC_2_11_005_sequential_org_queries_do_not_interfere
        PASS [  0.751s] (10/10) prism-bin::e2e_multi_org test_BC_2_06_018_per_org_seeded_data_is_disjoint
────────────
     Summary [   0.751s] 10 tests run: 10 passed, 0 skipped
```

---

## Org × Sensor Matrix Under Test

| Org | CrowdStrike | Armis | Claroty | Cyberint | Seeds |
|-----|-------------|-------|---------|----------|-------|
| org-a | YES (seed=100) | YES (seed=110) | NO | NO | — |
| org-b | NO | NO | YES (seed=120) | YES (seed=130) | — |
| org-c | YES (seed=200) | YES (seed=210) | YES (seed=220) | YES (seed=230) | — |

8 DTU clone instances total (BC-2.06.017 Postcondition 2), spawned via `MultiInstanceHarness::start(entries)`.

---

## AC Coverage Table

| AC | Title | Test | Result | Key Assertion |
|----|-------|------|--------|---------------|
| AC-001 | 3-org boot registers 8 adapters | `test_BC_2_22_001_multi_org_boot_registers_8_adapters` | PASS | AdapterRegistry count: org-a=2, org-b=2, org-c=4; total=8; verified via `boot.step9a.adapter_registry_populated` log event |
| AC-002 | Org A queries CrowdStrike — returns data | `test_BC_2_06_014_org_a_crowdstrike_query_returns_data` | PASS | `tool_query "FROM crowdstrike_detections LIMIT 5" client_id="org-a"` → non-empty data from org-a's DTU clone |
| AC-003 | Org B queries Claroty — returns data | `test_BC_2_06_014_org_b_claroty_query_returns_data` | PASS | `tool_query "FROM claroty_assets LIMIT 5" client_id="org-b"` → non-empty data from org-b's Claroty clone |
| AC-004 | Org C queries all 4 sensors independently | `test_BC_2_01_013_org_c_all_4_sensors_return_independent_data` | PASS | 4 independent queries for org-c all return non-empty data; no cross-sensor contamination (Red Gate) |
| AC-005 | Cross-org isolation: Org A + Cyberint → error | `test_BC_3_2_001_cross_org_query_returns_isolation_error` | PASS | `tool_query "FROM cyberint_alerts LIMIT 5" client_id="org-a"` → AdapterNotFound/isolation error, zero data rows (Red Gate) |
| AC-006 | Per-org seeded data disjoint: ids_org_a ∩ ids_org_c = ∅ | `test_BC_2_06_018_per_org_seeded_data_is_disjoint` | PASS | Response bodies for org-a (seed=100) and org-c (seed=200) CrowdStrike clones extracted; ID sets are disjoint — INV-DISTINCT-DATA-001 proven at integration level (Red Gate) |
| AC-007 | Cyberint per-org session isolation (org-b and org-c) | `test_BC_3_2_001_cyberint_session_isolation_org_b_and_org_c` | PASS | Distinct Cyberint sockets S_B ≠ S_C per INV-ISOLATION-001; each org receives its own session cookie; tokens do not cross |
| AC-008 | ResponseEnvelope metadata correct per org/sensor | `test_BC_2_09_008_response_envelope_identifies_correct_org_and_sensor` | PASS | `_meta.data_source` matches queried sensor; no foreign org identifiers appear in response |
| AC-009 | Sequential cross-org queries do not interfere | `test_BC_2_11_005_sequential_org_queries_do_not_interfere` | PASS | Rapid sequential org-a → org-c CrowdStrike queries; `ids_a ∩ ids_c = ∅` on both response bodies; BC-2.11.005 ephemeral materialization proven |
| AC-010 | Test gated by `#[ignore]` + e2e-multi-org profile | `test_BC_2_22_001_multi_org_tests_ignored_under_standard_profile` | PASS | `cargo nextest run -p prism-bin` (no profile) skips all 10 tests; `--profile e2e-multi-org --run-ignored ignored-only` un-gates them |

---

## Red Gate Tests

4 Red Gate tests designated in story frontmatter (`red_gate_tests: 4`), all PASS:

| Red Gate Test | AC | Result |
|---|---|---|
| `test_BC_2_22_001_multi_org_boot_registers_8_adapters` | AC-001 | PASS |
| `test_BC_2_01_013_org_c_all_4_sensors_return_independent_data` | AC-004 | PASS |
| `test_BC_3_2_001_cross_org_query_returns_isolation_error` | AC-005 | PASS |
| `test_BC_2_06_018_per_org_seeded_data_is_disjoint` | AC-006 | PASS |

---

## Headline Isolation Proofs

### INV-DISTINCT-DATA-001 — `ids_org_a ∩ ids_org_c = ∅`

AC-006 (`test_BC_2_06_018_per_org_seeded_data_is_disjoint`) is the primary proof.

- org-a CrowdStrike clone: `new_with_seed(seed=100, archetype, org_id_a)` — canonical IDs carry `hex(org_id_a.as_bytes()[0..4])`
- org-c CrowdStrike clone: `new_with_seed(seed=200, archetype, org_id_c)` — canonical IDs carry `hex(org_id_c.as_bytes()[0..4])`
- Both orgs share the sensor type; their ID sets are structurally disjoint by ADR-036 v2.0 §2.2 canonical format `dev-{8hex}-{seed}-{n}`
- The test reads actual response bodies and asserts the intersection is empty — NOT a port-binding-only assertion

### BC-3.2.001 — Cross-Org Isolation Error (AC-005)

org-a querying Cyberint (registered only to org-b/org-c) returns an isolation error — no data rows, no cross-org leakage. Proven by `test_BC_3_2_001_cross_org_query_returns_isolation_error`.

### INV-ISOLATION-001 — Per-Org Distinct DTU Sockets (AC-007)

`MultiInstanceHarness::start(entries)` binds 8 distinct ephemeral sockets. org-b Cyberint at S_B ≠ org-c Cyberint at S_C. Session cookies are org-scoped to each socket's clone instance. Proven by `test_BC_3_2_001_cyberint_session_isolation_org_b_and_org_c`.

---

## Recordings Index

```
docs/demo-evidence/S-DEMO-004/
  AC-001-010-e2e-multi-org-full-run.gif    (701KB) — AC-001..AC-010, full suite 10/10
  AC-001-010-e2e-multi-org-full-run.webm   (2.0MB) — AC-001..AC-010, full suite 10/10
  AC-001-010-e2e-multi-org-full-run.tape   — VHS source (no absolute paths)
  evidence-report.md                       — this file
```

---

## Gating Rationale

All 10 ACs are covered by the e2e-multi-org test suite. Unlike S-DEMO-003 which had
Keychain ACL constraints requiring some tests to be backed only by in-process mocks, this
story's ACs are all proven by the full subprocess E2E path:

- `prism-bin` is spawned as a subprocess via `SubprocessGuard` (S-DEMO-002 pattern)
- `MultiInstanceHarness` starts 8 real DTU clone instances at ephemeral ports
- `write_overlay_from_socket_map` writes per-org overlay TOML files to a `TempDir`
- `McpStdioHandle` drives `tool_query` over the real MCP stdio channel per org
- Response bodies are read and parsed; assertions are on actual data content

The MCP-over-stdio single-channel architecture (AD-013) serializes requests per analyst.
AC-009 tests "sequential" isolation (back-to-back, not `tokio::join!`) — this is the
correct model for the per-analyst deployment, per the AC-009 architecture rationale in
the story spec.

---

## Attestation

All 10 acceptance criteria are covered by the `e2e-multi-org` test suite. All 10 tests
PASS on feature/S-DEMO-004 HEAD 10769aac. The VHS recording captures the full nextest
run output showing 10/10 PASS. No AC is silently skipped. The 4 designated Red Gate
tests all pass. No absolute paths appear in any `.tape` file.
