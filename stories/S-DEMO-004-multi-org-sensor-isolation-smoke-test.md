---
document_type: story
story_id: S-DEMO-004
title: "prism-bin: Multi-Org × Multi-Sensor Isolation Smoke Test — BC-3.2.001 + ADR-029 + BC-2.06.017 + BC-2.06.018 demo validation"
wave: 5
epic_id: E-DEMO
priority: P0
status: ready
version: "1.16"
level: "L4"
producer: architect
timestamp: "2026-05-29T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-10, SS-11, SS-17, SS-22]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters): multi-org isolation requires per-org adapter registration;
#     AdapterRegistry must not cross-leak between orgs.
#   SS-10 (MCP Interface): demo drives tool_query with client_id scoping per org;
#     PrismServer routes queries per org identity.
#   SS-11 (Query Execution): fan_out() resolves (org_id, sensor_id) pairs; cross-org
#     leakage would appear as data contamination in RecordBatch output.
#   SS-17 (Multi-Tenancy): BC-3.2.001 org isolation is the primary contract under test.
#   SS-22 (Binary Entrypoint): boot step 9A produces per-org adapter entries; this test
#     verifies the boot output is correct for 3-org × mixed-sensor-combo configuration.
crates_touched: [prism-bin, prism-dtu-harness]
# prism-dtu-harness: wiring not redesign — added write_overlay_from_socket_map for
# BackgroundHarness socket_map-keyed overlay (BackgroundHarness extracts the socket_map
# off-runtime and cannot pass &MultiInstanceHarness, so a socket_map-keyed overlay writer
# was needed); write_overlay_temp_dir refactored to delegate to it; behavior-preserving;
# covered by e2e + harness tests. overlay_wiring.rs only.
target_module: prism-bin
capabilities: [CAP-001, CAP-015, CAP-029, CAP-034]
behavioral_contracts:
  - BC-3.2.001  # Multi-tenant org isolation — no cross-org data leakage; AdapterRegistry
                # keyed by (OrgId, SensorId) guarantees per-org adapter isolation.
  - BC-2.06.014  # Instance Identity Resolution at Fanout — (org_id, sensor_id) → ResolvedSensorSpec;
                 # per-org overlay means each org's query resolves to its own DTU clone endpoint.
  - BC-2.11.005  # Ephemeral Materialization — fan_out() materializes per-org; no cross-call state.
  - BC-2.01.013  # DataSource Trait — spec-driven adapters are per-org; no shared adapter across orgs.
  - BC-2.10.001  # rmcp ServerHandler — `client_id` scoping parameter in tool_query routes to
                 # the correct org's adapters.
  - BC-2.06.017  # Per-DTU-Instance Multi-Address Binding — MultiInstanceHarness socket_map keyed
                 # by (org_slug, sensor_id) provides each org a distinct SocketAddr; overlay_wiring
                 # writes per-org base_url; Postcondition 3 (overlay integration end-to-end) + INV-
                 # ISOLATION-001 (zero cross-tenant leakage) underpin AC-006 (distinct sockets) and
                 # AC-007 (per-org Cyberint session isolation) and AC-009 (concurrent no-mix).
                 # Merged PR #187 2026-06-14 (D-1158). AC-006 traces to PC-3.
  - BC-2.06.018  # Demo-Server Config-Time Data Seeding — new_with_seed(seed, archetype, org_id)
                 # wires per-clone seed through build_clone_pairs; INV-DISTINCT-DATA-001 proves that
                 # distinct (seed_A, org_id_A) vs (seed_C, org_id_C) for two CrowdStrike clone
                 # instances produces disjoint device/detection ID sets (ids_org_a ∩ ids_org_c = ∅).
                 # This is the CONTRACT BACKING AC-006's content-level assertion. Canonical ID format
                 # "dev-{8hex}-{seed}-{n}" where 8hex = hex(org_id.as_bytes()[0..4])
                 # (ADR-036 v2.0 §2.2) makes disjointness structural. {8hex} is derived from
                 # org_id UUID bytes, NOT the human slug "org-a" — asserting "dev-org-a-..."
                 # would match nothing and make ids_a ∩ ids_c = ∅ pass VACUOUSLY (false green).
                 # Merged PR #181 2026-06-10 (D-1089). AC-006 is the integration-level proof of
                 # INV-DISTINCT-DATA-001 across two orgs sharing the same sensor type.
  - BC-2.22.001  # Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate —
                 # AC-001 traces here: boot step 9A adapter_registry_populated event confirms
                 # deterministic boot sequencing; AC-010 traces here: startup deterministic and
                 # testable (test gated behind #[ignore] with explicit CI profile per boot
                 # orchestration invariant). Tests named test_BC_2_22_001_*.
  - BC-2.09.008  # Response Envelope with Trust Annotations — AC-008 traces here: ResponseEnvelope
                 # metadata must identify correct org and sensor; no org identifiers from other
                 # orgs appear in the response. Tests named test_BC_2_09_008_*.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — this test extends the parity coverage to the
            # multi-org dimension; each org's adapter must resolve to its org-scoped DTU clone.
depends_on:
  - S-DEMO-001   # Per-org adapter registration (boot step 9A) must be complete.
  - S-DEMO-002   # Single-org single-sensor E2E smoke test must pass first (build on its foundation).
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001  # Per-org overlay loading must be complete.
  - S-DEMO-MULTI-TENANT-DTU-001  # BC-2.06.017 multi-address binding — MultiInstanceHarness
                                  # (socket_map keyed by (org_slug, sensor_id)) is the mechanism
                                  # this story uses to spawn distinct DTU sockets per org × sensor
                                  # combo. Must be MERGED before prism-bin tests can call
                                  # MultiInstanceHarness::start() and write_overlay_temp_dir().
                                  # MERGED PR #187 develop@664566e9 2026-06-14 (D-1158). SATISFIED.
  - S-DEMO-DTU-LIVE-SCENARIO-001-A  # BC-2.06.018 config-time seeding — new_with_seed() constructor
                                     # and CloneConfig.seed wiring are required for AC-006 to assert
                                     # REAL data distinctness (org-A CrowdStrike ID set ≠ org-C
                                     # CrowdStrike ID set via INV-DISTINCT-DATA-001). Without seeded
                                     # generator data, clones return identical fixture JSON and
                                     # response-content assertions cannot distinguish orgs.
                                     # MERGED PR #181 develop@c287b00d 2026-06-10 (D-1089). SATISFIED.
  - S-DEMO-DTU-LIVE-SCENARIO-001-B  # BC-2.06.019 + BC-2.06.020 scenario progression + enrichment
                                     # correlation — Story B wires the scenario clock and archetype
                                     # selection that makes AC-006's "different seeds → disjoint ID
                                     # sets" live across all four generator-backed clone types during
                                     # a running demo. Also required for AC-009 concurrent-query
                                     # distinctness: with static fixtures both orgs return identical
                                     # snapshots; seeded scenarios are structurally disjoint.
                                     # MERGED PR #185 develop@7fd35b77 2026-06-13 (D-1139). SATISFIED.
blocks: []
# Historical edge scrubbed (v1.1, 2026-06-10 story-writer micro-burst): blocks
# originally carried S-DEMO-003 ("runbook should not ship until multi-org isolation
# is verified"). S-DEMO-003 MERGED via PR #176 on 2026-06-08 while this story was
# still draft — a merged story cannot be blocked, so the edge is moot and would
# only mislead the wave scheduler. The shipped-runbook-before-isolation-verification
# risk the edge encoded is now tracked by this story's own ACs (the isolation
# assertions still must pass before the demo is presented). See §Dispatch Ordering
# for the annotated historical diagram.
points: 8
# Points justification:
#   - 3-org test config setup (prism.toml with 3 orgs + 3 customers/ overlay dirs): ~1 pt
#   - DTU demo server config to serve 3 independent sensor combos: ~1 pt
#   - Per-org MCP query harness (3 × tool_query with org-scoped client_id): ~1.5 pts
#   - Cross-org isolation assertion (Org A query for Org B's sensor → AdapterNotFound/error): ~2 pts
#   - DTU per-tenant data verification (Org A data ≠ Org B data): ~1 pt
#   - CI integration + SubprocessGuard reuse from S-DEMO-002: ~0.5 pts
#   - ADR-029 overlay correctness assertion (each org uses its own DTU base_url): ~1 pt
#   Total: 8 points (~1.5-2 days)
estimated_days: 2
risk: MEDIUM
# Risk justification: The multi-org isolation logic depends on boot step 9A correctly
# iterating per-org ResolvedSensorSpec entries and keying the AdapterRegistry by
# (OrgId, SensorId). If boot step 9A merges all orgs under the same SensorSpec (losing
# org scope), isolation assertions will fail with misleading "data looks correct" outputs.
# Test design must include a deliberate cross-org probe (Org A's client_id querying
# Org B's sensor) to catch this failure mode.
acceptance_criteria_count: 10
red_gate_tests: 4
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Use MultiInstanceHarness::start(entries) from prism-dtu-harness to spawn all 8 clone
    instances (org-a×crowdstrike, org-a×armis, org-b×claroty, org-b×cyberint, org-c×crowdstrike,
    org-c×armis, org-c×claroty, org-c×cyberint) in a single call. Each HarnessEntry MUST be
    constructed via HarnessEntry::new(org_slug, sensor_id, clone) — NOT the struct-literal form
    (HarnessEntry is #[non_exhaustive]; E0639 blocks external struct-literal construction). Each
    clone carries a DISTINCT seed and org_id via new_with_seed(seed, archetype, org_id) — org-a's
    CrowdStrike gets seed=100, org-c's CrowdStrike gets seed=200, ensuring INV-DISTINCT-DATA-001
    holds: ids_org_a ∩ ids_org_c = ∅. Note fallibility: ArmisClone::new_with_seed and
    CyberintClone::new_with_seed return anyhow::Result<Self> (use ?); CrowdstrikeClone and
    ClarotyClone return Self (infallible). The archetype arg is prism_dtu_common::Archetype enum
    (e.g. Archetype::HealthyOtEnvironment); org_id is prism_dtu_common::OrgId([u8;16]), constructed
    OrgId(*uuid::Uuid::parse_str(s)?.as_bytes()). See prism-dtu-demo-server/src/harness.rs
    build_clone_pairs for the reference pattern. The disjointness is verified by reading response
    bodies and extracting device/detection ID sets, NOT by asserting on socket addresses alone. The
    write_overlay_temp_dir(&harness, tempdir.path()) call (note: tempdir.path() returns &Path —
    NOT &tempdir, which does not coerce to &Path) writes all 8 overlay TOML files from the
    socket_map keyed by (org_slug, sensor_id). The device IDs extracted from responses will match
    the format dev-{8hex}-{seed}-{n} where 8hex = hex(org_id.as_bytes()[0..4]) — for example,
    if org-a's UUID bytes begin 0xDEADBEEF..., then org-a IDs begin dev-deadbeef-100-0; derive
    the expected prefix from the UUID assigned to that org, NOT from the human slug org-a."
  - "Cross-org isolation probe must use the correct org_id. Test must query Org A using Org B's
    sensor_id explicitly — this must return AdapterNotFound or an explicit isolation error, not
    Org A's data for a different sensor."
  - "Reuse SubprocessGuard and wait_for_file() helpers from S-DEMO-002 tests/helpers/mod.rs
    rather than duplicating subprocess management logic."
inputs:
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - "crates/prism-bin/tests/helpers/mod.rs"
  - "crates/prism-dtu-demo-server/src/main.rs"
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - "crates/prism-dtu-harness/src/multi_instance.rs"
  - "crates/prism-dtu-harness/src/overlay_wiring.rs"
  - ".factory/specs/behavioral-contracts/BC-3.2.001-per-org-sensor-data-isolation.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.014-instance-identity-resolution-at-fanout.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.018-dtu-demo-clone-data-seeding.md"
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/specs/architecture/decisions/ADR-036-deterministic-scenario-progression-engine.md"
  - ".factory/stories/S-DEMO-001-spec-driven-sensor-adapter-and-boot-step-9a.md"
  - ".factory/stories/S-DEMO-002-e2e-subprocess-smoke-test-all-sensors.md"
  - ".factory/stories/S-CONFIG-MULTI-TENANT-OVERRIDE-001-per-org-sensor-endpoint-overlay-loading.md"
input-hash: "7694d3c"
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-004 — Multi-Org × Multi-Sensor Isolation Smoke Test

**Story ID:** S-DEMO-004
**Status:** ready
**Version:** v1.16
**Wave:** 5
**Priority:** P0
**Points:** 8

---

## Authority

ADR-029 §D governs the per-org overlay structure validated end-to-end by this story. The
overlay wiring (write_overlay_temp_dir / write_overlay_from_socket_map) writes one TOML file
per (org, sensor) pair pointing to the org's DTU clone socket. Read §D before writing
overlay wiring calls:
`.factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md`.

ADR-036 §2.2 defines the deterministic scenario progression and canonical device ID format
("dev-{8hex}-{seed}-{n}" where 8hex = hex(org_id.as_bytes()[0..4])). The disjointness
assertions in AC-006 (ids_org_a ∩ ids_org_c = ∅) rely on this structural ID format.
Read §2.2 before writing the ID-set comparison logic:
`.factory/specs/architecture/decisions/ADR-036-deterministic-scenario-progression-engine.md`.

---

## Origin

New story proposed by architect (2026-05-29) to close the multi-client demo scope gap not
covered by S-DEMO-002 v1.0.

S-DEMO-002 tests a single org with all 4 sensors. The user's demo target explicitly requires:
1. Multiple client orgs registered simultaneously with different sensor combos.
2. Verification that Org A's query for Org B's sensor returns an isolation error (BC-3.2.001).
3. Proof that DTU overlays route each org to its own clone instance (ADR-029 per-org overlay).

S-DEMO-002 does NOT cover these. This story extends the demo to the multi-org dimension.

**Reference demo configuration (user-stated 2026-05-29):**
- Org A: CrowdStrike + Armis
- Org B: Claroty + Cyberint
- Org C: all 4 sensors (CrowdStrike + Armis + Claroty + Cyberint)

---

## Narrative

As the Prism platform engineering team, I want an integration test that registers 3 orgs with
different sensor combinations, drives queries for each org via the MCP tool_query interface,
and asserts that: (a) each org's query returns data from its org-scoped DTU clone endpoint,
(b) querying Org A for a sensor not registered to Org A returns an isolation error (not data),
and (c) Org C's queries to all 4 sensors all succeed independently — so that BC-3.2.001 org
isolation is proven in code before the live demo.

---

## Story-Level Goal

After this story merges:
1. A 3-org integration test exists in `crates/prism-bin/tests/e2e_multi_org.rs`.
2. The test proves AdapterRegistry correctly scopes adapters to (OrgId, SensorId) pairs.
3. Cross-org isolation is verified: Org A's `client_id` + Org B's sensor_id → error (not data).
4. Per-org ADR-029 overlay routing is verified: each org's CrowdStrike adapter uses its org-specific DTU port.

---

## Behavioral Contracts

| BC ID | Title | AC Trace |
|-------|-------|----------|
| BC-3.2.001 | Multi-Tenant Org Isolation — no cross-org data leakage | AC-005, AC-007, AC-008 |
| BC-2.06.014 | Instance Identity Resolution at Fanout — (org_id, sensor_id) → ResolvedSensorSpec | AC-002, AC-003, AC-006 |
| BC-2.11.005 | Ephemeral Materialization — fan_out() materializes per-org; no cross-call state | AC-003, AC-009 |
| BC-2.01.013 | DataSource Trait — spec-driven adapters are per-org | AC-004 |
| BC-2.10.001 | rmcp ServerHandler — client_id scoping parameter routes to correct org's adapters | AC-001, AC-002 |
| BC-2.06.017 | Per-DTU-Instance Multi-Address Binding — MultiInstanceHarness + INV-ISOLATION-001 | AC-006 (PC-3: overlay integration end-to-end — write_overlay_temp_dir → distinct SocketAddrs), AC-007 (INV-ISOLATION-001: per-org distinct Cyberint DTU sockets), AC-009 (INV-ISOLATION-001: concurrent no cross-tenant socket dispatch) |
| BC-2.06.018 | Demo-Server Config-Time Data Seeding — INV-DISTINCT-DATA-001 disjoint ID sets | AC-006 (primary: ids_org_a ∩ ids_org_c = ∅ CONTENT assertion via new_with_seed per-org seeds), AC-009 (secondary: concurrent ids_a ∩ ids_c = ∅ under concurrent dispatch) |
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate | AC-001 (boot step 9A adapter_registry_populated event confirms deterministic boot sequencing), AC-010 (startup deterministic and testable; test gated behind #[ignore] with explicit CI profile per boot orchestration invariant) |
| BC-2.09.008 | Response Envelope with Trust Annotations | AC-008 (ResponseEnvelope metadata must identify correct org and sensor; no org identifiers from other orgs appear in the response) |

---

## Multi-Org Demo Configuration

### Org registration (prism.toml)

```toml
[[orgs]]
org_id = "<uuid-v7-org-a>"
org_slug = "org-a"

[[orgs]]
org_id = "<uuid-v7-org-b>"
org_slug = "org-b"

[[orgs]]
org_id = "<uuid-v7-org-c>"
org_slug = "org-c"
```

### Sensor combos

| Org | CrowdStrike | Armis | Claroty | Cyberint |
|-----|-------------|-------|---------|----------|
| org-a | YES | YES | NO | NO |
| org-b | NO | NO | YES | YES |
| org-c | YES | YES | YES | YES |

### customers/ overlay structure

```
customers/
  org-a/
    crowdstrike.sensor.toml   # base_url = "http://127.0.0.1:<CS_PORT_ORG_A>"
    armis.sensor.toml         # base_url = "http://127.0.0.1:<ARMIS_PORT_ORG_A>"
  org-b/
    claroty.sensor.toml       # base_url = "http://127.0.0.1:<CLAROTY_PORT_ORG_B>"
    cyberint.sensor.toml      # base_url = "http://127.0.0.1:<CYBERINT_PORT_ORG_B>"
  org-c/
    crowdstrike.sensor.toml   # base_url = "http://127.0.0.1:<CS_PORT_ORG_C>"
    armis.sensor.toml         # base_url = "http://127.0.0.1:<ARMIS_PORT_ORG_C>"
    claroty.sensor.toml       # base_url = "http://127.0.0.1:<CLAROTY_PORT_ORG_C>"
    cyberint.sensor.toml      # base_url = "http://127.0.0.1:<CYBERINT_PORT_ORG_C>"
```

Each org gets its own DTU clone instances at distinct ephemeral ports. This proves
per-org overlay routing is correct — Org A's CrowdStrike queries hit CS_PORT_ORG_A,
not CS_PORT_ORG_C, even though both orgs have CrowdStrike registered.

### DTU multi-tenancy scope

**RECONCILED (v1.2, architect 2026-06-14, T8 — D-1077 scope expansion)**

The D-1077 user directive requires REAL per-client data segregation, NOT just client-targeting
or port-binding-only isolation. The merged capability stack (BC-2.06.017 + BC-2.06.018, both
ACTIVE via PR #187 and PR #181) makes real data distinctness both available and mandatory.

**Multi-instance mechanism (BC-2.06.017):** Each org × sensor pair gets its own DTU socket via
`MultiInstanceHarness::start(entries)` from `prism-dtu-harness`. The harness binds N clone
instances at ephemeral ports and returns a `socket_map: HashMap<(String, String), SocketAddr>`
keyed by `(org_slug, sensor_id)`. The test then calls `overlay_wiring::write_overlay_temp_dir`
to produce the `customers/org-x/<sensor>.sensor.toml` overlay files from the socket map — each
overlay carries `extends`, `instance_id = "{sensor_id}@{org_slug}"`, and `base_url`. This is
the mechanism specified by BC-2.06.017 Postcondition 2 and Postcondition 3. SEPARATE demo-server
invocations per org are NOT needed and NOT used — a single `MultiInstanceHarness::start()` call
with all (org_slug, sensor_id) entries handles the full 3-org × mixed-sensor matrix.

**Real seeded data distinctness (BC-2.06.018 + INV-DISTINCT-DATA-001):** Each clone instance
is constructed with a DISTINCT seed via `new_with_seed(seed: u64, archetype: prism_dtu_common::Archetype, org_id: prism_dtu_common::OrgId)`. Fallibility varies by clone type:
- `CrowdstrikeClone::new_with_seed` → `Self` (infallible)
- `ClarotyClone::new_with_seed` → `Self` (infallible)
- `ArmisClone::new_with_seed` → `anyhow::Result<Self>` (fallible — use `?`)
- `CyberintClone::new_with_seed` → `anyhow::Result<Self>` (fallible — use `?`)
`archetype` is the `prism_dtu_common::Archetype` enum (e.g., `Archetype::HealthyOtEnvironment` or `Archetype::CompromisedEndpoint`), NOT a string. `org_id` is `prism_dtu_common::OrgId([u8; 16])`, constructed as `OrgId(*uuid::Uuid::parse_str(s)?.as_bytes())`. Reference pattern: `prism-dtu-demo-server/src/harness.rs` `build_clone_pairs` already demonstrates `OrgId(*uuid.as_bytes())` construction and `?`-propagation of the fallible constructors. The `DemoConfig`/`CloneConfig` carries `org_id: Option<String>` (UUID) and `seed: u64`, wired through `build_clone_pairs` per BC-2.06.018 Postcondition 1. Because `seed_A ≠ seed_B` (or `org_id_A ≠ org_id_B`), INV-DISTINCT-DATA-001 guarantees `ids(org-A CrowdStrike) ∩ ids(org-C CrowdStrike) = ∅`. The isolation proof is on RESPONSE DATA CONTENT (device/detection IDs differ between orgs), not on TCP port binding alone.

**Port binding as a first-order invariant (INV-ISOLATION-001, BC-2.06.017):** The distinct
sockets also enforce INV-ISOLATION-001 (zero cross-tenant leakage at the network layer). Both
proofs are required: TCP isolation ensures requests cannot reach the wrong instance; data
distinctness ensures even if routing were misdirected, the content would reveal the error.
The `X-Prism-Org-Id` header routing (BC-3.2.003, Cyberint clone) remains an additional
per-session scoping mechanism exercised by AC-007, but it is no longer the PRIMARY isolation
proof — that role belongs to the seeded data content assertion in AC-006.

**Consequence for fixture data:** The static JSON fixtures used by the pre-seeding `new()`
constructors are NOT used for any clone instance under this test. All generator-backed
clones (CrowdStrike, Armis, Claroty, Cyberint) use `new_with_seed` with per-org seeds. The
backward-compat default (seed=42, fixture_set="default") is NOT used here because we need
distinct data between orgs sharing the same sensor type (e.g., org-a and org-c both have
CrowdStrike, but must serve non-overlapping device ID sets).

---

## Acceptance Criteria

### AC-001: 3-org boot registrations correct
Given: prism.toml with 3 orgs and mixed sensor overlays as in §Multi-Org Demo Configuration.
When: prism-bin starts and boot step 9A completes.
Then: AdapterRegistry contains exactly the expected count:
- org-a: 2 adapters (CrowdStrike + Armis)
- org-b: 2 adapters (Claroty + Cyberint)
- org-c: 4 adapters (all 4 sensors)
Total: 8 adapters. Verified via `boot.step9a.adapter_registry_populated` event log assertion.
(traces to BC-2.22.001 Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate postcondition: boot step 9A adapter_registry_populated event confirms deterministic sequencing; BC-2.10.001 rmcp ServerHandler — correct per-org adapter count enables client_id routing at query time)

### AC-002: Org A queries return data for registered sensors only
Given: org-a is registered with CrowdStrike + Armis (not Claroty or Cyberint).
When: `tool_query "FROM crowdstrike_detections LIMIT 5" client_id="org-a"` is sent.
Then: Returns non-empty data from org-a's CrowdStrike DTU clone.
(traces to BC-2.06.014 instance identity resolution; BC-2.10.001 client_id scoping routes to org-a's adapters)

### AC-003: Org B queries return data for registered sensors only
Given: org-b is registered with Claroty + Cyberint (not CrowdStrike or Armis).
When: `tool_query "SELECT * FROM claroty_alerts LIMIT 5" client_id="org-b"` is sent.
Then: Returns non-empty data from org-b's Claroty DTU clone.
(traces to BC-2.06.014; BC-2.11.005)

### AC-004: Org C queries succeed for all 4 sensors independently
Given: org-c is registered with all 4 sensors.
When: Each of the 4 `tool_query` calls is sent with `client_id="org-c"`.
Then: All 4 return non-empty data; no cross-sensor data contamination.
(traces to BC-2.01.013 spec-driven adapters are per-org)
Red Gate test: `test_BC_2_01_013_org_c_all_4_sensors_return_independent_data`

### AC-005: Cross-org isolation — Org A querying Org B's sensor returns isolation error
Given: Org A has CrowdStrike + Armis registered; Cyberint is NOT registered for Org A.
When: `tool_query "FROM cyberint_alerts LIMIT 5" client_id="org-a"` is sent.
Then: Response envelope contains an error code (AdapterNotFound or SensorNotAvailableForOrg);
NO data rows are returned; NO data from Org B leaks into Org A's response.
(traces to BC-3.2.001 invariant: no cross-org data leakage)
Red Gate test: `test_BC_3_2_001_cross_org_query_returns_isolation_error`

### AC-006: Per-org seeded data distinctness — ids_org_a ∩ ids_org_c = ∅ (INV-DISTINCT-DATA-001)
Given: Org A and Org C both have CrowdStrike registered. In the MultiInstanceHarness setup,
org-a's CrowdStrike clone is constructed via `new_with_seed(seed_a=100, archetype, org_id_a)`
and org-c's CrowdStrike clone via `new_with_seed(seed_c=200, archetype, org_id_c)`, where
`seed_a ≠ seed_c` and `org_id_a ≠ org_id_c`. The overlay TOML files written by
`write_overlay_temp_dir(&harness, tempdir.path())` route each org to its distinct clone socket. (`tempdir.path()` returns `&std::path::Path`; `&TempDir` does NOT coerce to `&Path` — always use `.path()`.)
When: `tool_query "FROM crowdstrike_detections LIMIT 50"` is executed for org-a and then
for org-c (each via the MCP tool_query interface with the appropriate client_id).
Then: The test READS both response bodies and extracts the set of device/detection IDs
(canonical format `"dev-{8hex}-{seed}-{n}"` per ADR-036 v2.0 §2.2, where `{8hex} =
hex(org_id.as_bytes()[0..4])`) from each response. The test asserts:
  `ids_org_a ∩ ids_org_c = ∅`   (INV-DISTINCT-DATA-001 verified at integration-test level)
This is a CONTENT-LEVEL assertion on actual response data. The test MUST NOT assert only
on socket addresses or port numbers. CRITICAL FALSE-GREEN TRAP: the expected ID prefix MUST
be derived from `hex(org_id.as_bytes()[0..4])` of the UUID assigned to that org — NOT the
human slug `"org-a"`. Asserting against `"dev-org-a-..."` would match zero IDs, making
`ids_org_a ∩ ids_org_c = ∅` true VACUOUSLY (both sets empty = disjoint), defeating the proof.
For example: if org-a's UUID is `deadbeef-...`, assert IDs match regex `dev-deadbeef-100-\d+`.
`POST /dtu/configure` is NOT used — config-time seeding via `CloneConfig.seed` +
`CloneConfig.org_id` is the primary path per INV-CONFIGURE-ENDPOINT-SECONDARY-001.
(traces to BC-2.06.017 Postcondition 3 + BC-2.06.018 INV-DISTINCT-DATA-001 + BC-2.06.014 endpoint-resolution)
Red Gate test: `test_BC_2_06_018_per_org_seeded_data_is_disjoint`

### AC-007: Cyberint cookie_roundtrip auth works for org-b and org-c independently
Given: org-b and org-c both have Cyberint registered. MultiInstanceHarness binds a distinct
socket for each org's Cyberint clone — `(org-b, cyberint)` at socket S_B and `(org-c, cyberint)`
at socket S_C (S_B ≠ S_C), per INV-ISOLATION-001 (BC-2.06.017).
When: `tool_query "FROM cyberint_alerts LIMIT 5"` is sent for each org.
Then: Each query succeeds with its own session cookie from the respective org's DTU clone;
the session tokens do not cross between org-b and org-c.
(traces to BC-3.2.001 session isolation; BC-2.01.013 per-org adapter construction;
BC-2.06.017 INV-ISOLATION-001 per-org distinct Cyberint DTU sockets)

### AC-008: ResponseEnvelope metadata identifies correct org and sensor
Given: A successful multi-org query for any org/sensor combination.
When: The ResponseEnvelope is inspected.
Then: `_meta.data_source` contains the correct sensor name; the response is scoped to the
querying org's data — no org identifiers from other orgs appear in the response.
(traces to BC-2.09.008 Response Envelope with Trust Annotations — ResponseEnvelope metadata must identify correct org and sensor; no org identifiers from other orgs appear in the response)

### AC-009: Sequential cross-org queries do not interfere — no cross-call state (BC-2.11.005)
Given: org-a and org-c both query CrowdStrike using the same MultiInstanceHarness setup as
AC-006 (org-a seed=100, org-c seed=200, distinct org_ids, distinct sockets S_A and S_C per
INV-ISOLATION-001).
When: Two `tool_query` calls are sent back-to-back over the MCP stdio channel — first for
org-a, then for org-c — with minimal delay between them (rapid sequential dispatch).
Then: org-a's response contains only data from the org-a CrowdStrike clone; org-c's response
contains only data from the org-c CrowdStrike clone; no row-level mixing occurs — verified by
asserting `ids_a ∩ ids_c = ∅` on both response bodies, consistent with AC-006's
INV-DISTINCT-DATA-001 proof. Port-address cross-reference alone is insufficient; the test MUST
read and compare the actual device/detection ID sets (`"dev-{8hex}-{seed}-{n}"` format where
`{8hex} = hex(org_id.as_bytes()[0..4])`) from both responses. As with AC-006, the expected ID
prefix must be derived from the org's UUID bytes — NOT the human slug. Asserting against
`"dev-org-a-..."` would yield empty ID sets and a vacuous false-green.

Architecture rationale (single-channel stdio): MCP-over-stdio uses a single, serialized
request/response channel per the prism per-analyst deployment model (AD-013 / project
deployment model). A single MCP client cannot issue two genuinely concurrent requests over
one stdio pipe — the channel serializes them. Therefore this AC tests the isolation property
via rapid sequential dispatch, NOT via `tokio::join!` simultaneous dispatch. The property
BC-2.11.005 requires is zero cross-call state / ephemeral per-query materialization, which
is fully proven by back-to-back sequential dispatch + `ids_a ∩ ids_c = ∅` on both response
bodies. True cross-thread client concurrency would require McpStdioHandle to be Send and a
multi-client scenario — that is out of scope for this single-analyst integration test.

Test rename guidance: `test_BC_2_11_005_concurrent_org_queries_do_not_interfere` → rename to
`test_BC_2_11_005_sequential_org_queries_do_not_interfere` (story-writer/implementer to apply)
to align the test name with the actual dispatch model and prevent future re-flagging.
(traces to BC-2.11.005 invariant: no cross-call state; ephemeral materialization;
BC-2.06.018 INV-DISTINCT-DATA-001 sequential distinctness proof)

### AC-010: Test is gated behind `#[ignore]` with explicit CI multi-org profile
Given: Standard nextest profile runs (no DTU server available).
When: `cargo nextest run -p prism-bin` is executed.
Then: Multi-org test is skipped (marked `#[ignore]`). CI runs with `--profile e2e-multi-org`
to execute it. Comment: `// E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.`
(traces to BC-2.22.001 Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate invariant: startup deterministic and testable)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| [TODO: populate per template] | [module_path] | pure-core / effectful-shell |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| AdapterRegistry keyed by (OrgId, SensorId) — not (SensorId) alone | BC-3.2.001 | Test AC-005 cross-org probe fails if AdapterRegistry lacks org scope |
| Per-org ResolvedSensorSpec from overlay must be used — not base spec | ADR-029 §D2 | Test AC-006 verifies data content from per-org seeded clone, not production URL |
| AC-006 isolation proof is on RESPONSE DATA CONTENT (ID sets), not TCP port binding | BC-2.06.018 INV-DISTINCT-DATA-001 + D-1077 | Test AC-006 must read device/detection IDs from both orgs and assert set intersection = ∅; port-binding-only assertion is INSUFFICIENT |
| org-a's CrowdStrike seed ≠ org-c's CrowdStrike seed (and similarly for any sensor shared between orgs) | BC-2.06.018 Postcondition 1 + ADR-036 v2.0 §2.2 | Test fixture passes distinct seed and org_id per HarnessEntry — org-a CrowdStrike seed=100, org-c CrowdStrike seed=200 (example; any distinct pair satisfying seed_A ≠ seed_C is valid) |
| MultiInstanceHarness (BC-2.06.017) is the spawn mechanism — NOT separate demo-server invocations | BC-2.06.017 Postcondition 2 + Open Question 1 (resolved) | Test helper calls MultiInstanceHarness::start(entries) with all 8 (org_slug, sensor_id) pairs; then write_overlay_temp_dir; no separate demo-server processes |
| No shared mutable state between concurrent org queries | BC-2.11.005 ephemeral | Test AC-009 concurrent probe |
| Cyberint session cookie scoped to per-org CookieLoginAuthProvider instance | BC-3.2.001 | Test AC-007 org-specific session token; distinct sockets per BC-2.06.017 INV-ISOLATION-001 |
| AC-005/AC-006/AC-009 must exercise the REAL prism-bin → fan_out → DTU path (not a self-referential tautology) | T6 paper-fix lesson (z23); INV-PERIMETER-001 scoping | prism-bin tests are NOT perimeter-constrained — see §Architecture Compliance Note below |
| Canonical device ID format for all ID-set assertions | ADR-036 v2.0 §2.2; BC-2.06.018 INV-DISTINCT-DATA-001 | IDs extracted from responses must match `"dev-{8hex}-{seed}-{n}"` where `8hex = hex(org_id.as_bytes()[0..4])`; never `"dev-acme-..."` or any non-canonical form |

### Architecture Compliance Note — prism-bin perimeter vs prism-dtu-harness perimeter (T6 lesson z23)

During S-DEMO-MULTI-TENANT-DTU-001 (T6), the PR-LEVEL adversary caught that `prism-dtu-harness`
is subject to INV-PERIMETER-001: it MUST NOT import `prism-spec-engine`, `prism-sensors`, or
`prism-query`. This means harness-level isolation tests cannot call `FanOutTarget` directly and
were limited to distinct-listener TCP counting via `GET /dtu/request-count`. The cross-layer
`FanOutTarget`→`base_url` routing proof lives in `prism-sensors/tests/` (F-PR3-HIGH-001 fix)
outside the harness perimeter.

`crates/prism-bin/tests/e2e_multi_org.rs` (this story's test) operates under a DIFFERENT rule:
`prism-bin` is the binary entrypoint crate and its integration tests CAN import and exercise
the full production stack — `prism-sensors` fan_out, `prism-spec-engine` SpecLoader, and
`prism-core` types are all available to prism-bin test code. There is no equivalent of
INV-PERIMETER-001 for prism-bin integration tests.

Therefore, AC-005, AC-006, and AC-009 MUST exercise the real `prism-bin → fan_out → DTU` path
end-to-end via the MCP `tool_query` interface (spawning prism-bin as a subprocess via
`SubprocessGuard`, following the S-DEMO-002 pattern). The tests MUST NOT:
- Drive fan_out() directly without going through prism-bin (that would be testing the unit, not
  the integration).
- Self-validate by asserting only on port socket addresses without verifying response data.
- Skip the MCP `tool_query` layer and call sensor adapters directly (removes the org-scoping
  logic in `PrismServer`/`client_id` routing from the test scope).

The subprocess E2E pattern ensures the full org-scoping stack (MCP client_id → AdapterRegistry
lookup by (OrgId, SensorId) → ResolvedSensorSpec overlay → FanOutTarget HTTP dispatch → DTU
clone response) is traversed by each assertion, not just a subset of it.

---

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| [TODO: populate per template] | [>= X.Y.Z] | [why this version is required] |

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-bin/tests/e2e_multi_org.rs` | CREATE | Multi-org integration test with all 10 ACs |
| `crates/prism-bin/tests/helpers/mod.rs` | MODIFY | Add `start_multi_org_harness()` helper that calls `MultiInstanceHarness::start(entries)` with all 8 `HarnessEntry` items constructed via `HarnessEntry::new(org_slug, sensor_id, clone)` (one per (org_slug, sensor_id) pair) and `write_multi_org_overlays(&BackgroundHarness, &TempDir)` — internally calls `write_overlay_from_socket_map(harness.socket_map(), specs_dir)` (NOT `write_overlay_temp_dir(&harness, tempdir.path())`; the `BackgroundHarness` wrapper runs the 8 DTU clones on a dedicated multi-thread runtime in a background thread, exposes `socket_map()` via a oneshot, and accepts a `SyncSender` for shutdown — its `Drop` impl joins the background thread; because it cannot pass `&MultiInstanceHarness` off-runtime, the socket_map-keyed `write_overlay_from_socket_map` is the correct call); add `write_multi_org_prism_toml(&TempDir)` — takes only the tempdir (no `socket_map` argument; overlay files already written by `write_multi_org_overlays`), writes the 3-org `prism.toml` from the well-known org UUIDs used during seeding. Note: `write_multi_org_demo_config` and `McpStdioHandle` with `tool_query_scoped` / `tool_query_scoped_expect_rpc_error` already exist in this file from S-DEMO-002; reuse them for per-org `client_id` queries in AC-002/003/005 rather than re-implementing. The retired `MultiOrgDtuPorts` helper struct is NOT used — the `socket_map: &HashMap<(String, String), SocketAddr>` from `harness.socket_map()` is the canonical per-org port registry. |
| `crates/prism-bin/fixtures/multi-org-prism.toml.template` | CREATE | Template with 3-org prism.toml config (org-a, org-b, org-c with mixed sensor combos). Location: crate-level `fixtures/` directory, NOT `tests/fixtures/` — there is no `tests/fixtures/` path in prism-bin. |
| `.config/nextest.toml` | MODIFY | Add `[profile.e2e-multi-org]` block that un-ignores E2E-MULTI-001-tagged tests. This file already exists (contains `[profile.e2e]` and `[profile.ci]`); the new block must be appended, mirroring the existing `[profile.e2e]` block (the established S-DEMO-002 gating pattern). |
| `crates/prism-bin/src/boot.rs` | MODIFY | `step1_init_tracing` forces all tracing fmt layers to stderr (`.with_writer(std::io::stderr)`) so tracing never pollutes the MCP JSON-RPC stdout channel — MCP-stdout-purity invariant. This change landed in fix-burst commit a7c9cbf0 closing LOCAL adversary pass-2 finding M2-01. |
| `crates/prism-dtu-harness/src/overlay_wiring.rs` | MODIFY | Add `write_overlay_from_socket_map` (socket_map-keyed overlay writer); refactor `write_overlay_temp_dir` to delegate to it — needed by the off-runtime `BackgroundHarness` test pattern that cannot pass `&MultiInstanceHarness`. Behavior-preserving; covered by e2e + harness tests. |
| `crates/prism-bin/Cargo.toml` | MODIFY | Add dev-dependencies for multi-org seeded harness test. The current `[dev-dependencies]` in prism-bin only has `prism-dtu-armis` and `prism-dtu-crowdstrike` with `features = ["dtu"]` (NOT `fixture-gen`), and has no `prism-dtu-claroty`, `prism-dtu-cyberint`, or `prism-dtu-harness`. Required changes: (1) add `fixture-gen` feature to `prism-dtu-armis` and `prism-dtu-crowdstrike` dev-dep entries (change `features = ["dtu"]` → `features = ["dtu", "fixture-gen"]`); (2) MODIFY the EXISTING `prism-dtu-common` dev-dep: `features = ["dtu"]` → `features = ["dtu", "fixture-gen"]` (already present from S-DEMO-QUERY-PUSHDOWN-001; `dtu` and `fixture-gen` are independent features on prism-dtu-common — `dtu` does NOT transitively enable `fixture-gen`, so BOTH must be listed; dropping `dtu` would break the push-down tests that depend on it); (3) add `prism-dtu-claroty` dev-dep with `features = ["dtu", "fixture-gen"]`; (4) add `prism-dtu-cyberint` dev-dep with `features = ["dtu", "fixture-gen"]`; (5) add `prism-dtu-harness` dev-dep with `features = ["dtu"]` (transitively pulls `prism-dtu-common/fixture-gen`). Note: INV-PERIMETER-001 does NOT apply to prism-bin dev-dependencies — prism-bin integration tests are permitted to import the full stack (see §Architecture Compliance Note). |
| `crates/prism-bin/tests/bc_2_10_006_mcp_stdout_purity.rs` | CREATE | MCP-stdout-purity regression guard: asserts `prism start` emits zero tracing on stdout under RUST_LOG=info (stdout reserved for MCP JSON-RPC per BC-2.10.006 §Postconditions; enforced at the step1_init_tracing emission site). GREEN regression guard, not a Red Gate AC test. (LOCAL adversary pass-2 finding M2-02: `tracing must not pollute MCP stdout`; standalone file, not part of e2e_multi_org.rs.) Does not change `acceptance_criteria_count` (10) or `red_gate_tests` (4). |

---

## Tasks

1. **Read** S-DEMO-002 test helpers (`crates/prism-bin/tests/helpers/mod.rs`) — understand `SubprocessGuard`, `wait_for_file()`, `write_demo_config()` before extending. NOTE: this file ALREADY contains `write_multi_org_demo_config` and an `McpStdioHandle` with `tool_query_scoped` / `tool_query_scoped_expect_rpc_error` methods. These are directly useful for the per-org `client_id`-scoped queries in AC-002, AC-003, and AC-005 — reuse them rather than re-implementing the scoped-query logic.
2. **Read** `crates/prism-dtu-harness/src/multi_instance.rs` and `crates/prism-dtu-harness/src/overlay_wiring.rs` — understand `MultiInstanceHarness::start(entries)`, `HarnessEntry::new(org_slug, sensor_id, clone)` (the canonical constructor — `HarnessEntry` is `#[non_exhaustive]` so struct-literal construction `HarnessEntry { org_slug, sensor_id, clone }` is BLOCKED by E0639; always use `HarnessEntry::new(org_slug: impl Into<String>, sensor_id: impl Into<String>, clone: Box<dyn BehavioralClone>)`), `harness.socket_map()` (returns `&HashMap<(String, String), SocketAddr>` keyed by `(org_slug, sensor_id)` plain strings), and `write_overlay_temp_dir(&harness, tempdir.path())` (writes per-org overlay TOML with the 3-field format: `extends`, `instance_id = "{sensor_id}@{org_slug}"`, `base_url`; note the second argument is `&std::path::Path`, use `tempdir.path()` — NOT `&tempdir`).
3. **Write** `start_multi_org_harness()` test helper in `crates/prism-bin/tests/helpers/mod.rs` — calls `MultiInstanceHarness::start(entries).await` with 8 `HarnessEntry` items: one per (org_slug, sensor_id) pair. Each `HarnessEntry` MUST be constructed via `HarnessEntry::new(org_slug, sensor_id, clone)` (the canonical constructor — struct-literal form is blocked by `#[non_exhaustive]`). Each clone is constructed via `new_with_seed(seed, archetype, org_id)` with DISTINCT seed and org_id per org (e.g., org-a CrowdStrike seed=100, org-c CrowdStrike seed=200; all per-org seeds must be non-overlapping). See FIX 5 note below for fallibility: `ArmisClone::new_with_seed` and `CyberintClone::new_with_seed` are fallible (return `anyhow::Result<Self>`); use `?` to propagate. `CrowdstrikeClone::new_with_seed` and `ClarotyClone::new_with_seed` are infallible (return `Self`). Calls `write_overlay_temp_dir(&harness, tempdir.path())` (note: `tempdir.path()` not `&tempdir`) to produce the overlay TOML files. Returns the harness handle + temp dir (kept alive for test duration). NOTE: seeds must also satisfy the `MultiInstanceHarness::start` precondition that no duplicate `(org_slug, sensor_id)` keys exist — use exactly 8 distinct entries.
4. **Write** `write_multi_org_prism_toml(&TempDir)` helper — takes only the tempdir (no socket_map arg; overlay TOML files were already written to `tempdir` by `write_multi_org_overlays` in step 3). Writes the 3-org `prism.toml` into `tempdir` with the org_id UUIDs matching those used for seeding (so boot step 9A loads the correct org identities). Does NOT call `write_overlay_from_socket_map` again — that is called inside `write_multi_org_overlays`.
5. **Write Red Gate tests** in `crates/prism-bin/tests/e2e_multi_org.rs` — AC-001, AC-004 shape tests fail RED before S-DEMO-001 merges; `test_BC_2_06_018_per_org_seeded_data_is_disjoint` (AC-006) fails RED until `new_with_seed` seeded clones are serving in prism-bin subprocess path.
6. **Implement** cross-org isolation assertion (AC-005) — sends tool_query with org-a's client_id for a sensor not registered to org-a; asserts error response, not data.
7. **Implement** AC-006 content-level distinctness assertion — sends `tool_query "FROM crowdstrike_detections LIMIT 50"` for org-a and org-c; extracts device/detection ID sets from both response bodies; asserts `ids_org_a ∩ ids_org_c = ∅` (NOT a socket-address assertion). Canonical ID format: `"dev-{8hex}-{seed}-{n}"` where `8hex = hex(org_id.as_bytes()[0..4])` per ADR-036 v2.0 §2.2.
8. **Implement** Cyberint per-org session isolation (AC-007) — two orgs' Cyberint queries at their distinct MultiInstanceHarness sockets; assert CookieLoginAuthProvider constructs per-org sessions independently (session tokens do not cross between org-b and org-c per INV-ISOLATION-001).
9. **Implement** sequential cross-org query test (AC-009) — send org-a and org-c CrowdStrike queries back-to-back over the single MCP stdio channel (rapid sequential dispatch, NOT `tokio::join!` simultaneous dispatch — the stdio pipe serializes requests); extract both ID sets from response bodies; assert `ids_a ∩ ids_c = ∅` (same proof as AC-006, testing ephemeral materialization under rapid sequential dispatch; test name: `test_BC_2_11_005_sequential_org_queries_do_not_interfere`).
10. **Add** `[profile.e2e-multi-org]` to `.config/nextest.toml` (NOT `.cargo/nextest.toml` — that path does not exist; nextest reads `.config/nextest.toml`). Append the new block to the existing file, mirroring the existing `[profile.e2e]` block structure.
11. **Run** `cargo nextest run -p prism-bin --profile e2e-multi-org` after S-DEMO-001 + S-DEMO-002 merge; all assertions GREEN.
12. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| [TODO: populate per template] | [decisions made] | [patterns to follow] | [pitfalls to avoid] |

---

## Open Questions

**RECONCILED (v1.2, architect 2026-06-14, T8) — all questions resolved by the merged capability stack.**

1. **DTU demo server instance count — RESOLVED.** The `MultiInstanceHarness` from `prism-dtu-harness`
   (BC-2.06.017, MERGED PR #187) handles multiple distinct sockets in a single harness call.
   `MultiInstanceHarness::start(entries)` where `entries` is a `Vec<HarnessEntry>` of
   `(org_slug, sensor_id, Box<dyn BehavioralClone>)` tuples binds each entry at an ephemeral port
   and returns a `socket_map: HashMap<(String, String), SocketAddr>`. For this story's 3-org ×
   mixed-sensor matrix we need 8 entries: (org-a, crowdstrike), (org-a, armis), (org-b, claroty),
   (org-b, cyberint), (org-c, crowdstrike), (org-c, armis), (org-c, claroty), (org-c, cyberint).
   All 8 bind in a single `MultiInstanceHarness::start()` call — separate demo-server invocations
   are NOT required. `write_overlay_temp_dir(&harness, tempdir.path())` writes all 8 overlay TOML files. (Second arg is `&std::path::Path`; use `tempdir.path()` — `&TempDir` does NOT coerce to `&Path`.)
   Zero-entry configs are valid no-ops per EC-017-002; 8-entry configs are well within bounds
   per EC-017-008 (no hard cap). Confirmed by BC-2.06.017 Postcondition 2 and TV-017-003.

2. **Org-specific fixture data — RESOLVED. Option (a) — REAL seeded data — is MANDATORY.**
   The D-1077 user directive requires REAL per-client data segregation; the port-binding-only
   proof (former Option b) does not satisfy this requirement. The merged seeding substrate
   (BC-2.06.018 MERGED PR #181, INV-DISTINCT-DATA-001 proven) provides the mechanism.
   Each clone instance is constructed with `new_with_seed(seed: u64, archetype: prism_dtu_common::Archetype, org_id: prism_dtu_common::OrgId)`
   where org-a's CrowdStrike seed ≠ org-c's CrowdStrike seed (e.g., assign seed 100 to org-a
   and seed 200 to org-c). Fallibility: `CrowdstrikeClone::new_with_seed` and
   `ClarotyClone::new_with_seed` are infallible (return `Self`); `ArmisClone::new_with_seed`
   and `CyberintClone::new_with_seed` are fallible (return `anyhow::Result<Self>` — propagate
   with `?`). `archetype` is `prism_dtu_common::Archetype` enum (NOT a string); `org_id` is
   `prism_dtu_common::OrgId([u8; 16])` constructed as `OrgId(*uuid::Uuid::parse_str(s)?.as_bytes())`.
   INV-DISTINCT-DATA-001 guarantees `ids(seed=100, org_id_A) ∩ ids(seed=200, org_id_C) = ∅` —
   the canonical device ID format `"dev-{8hex}-{seed}-{n}"` where `8hex = hex(org_id.as_bytes()[0..4])`
   (ADR-036 v2.0 §2.2) makes this structurally disjoint across distinct (seed, org_id) pairs.
   AC-006 asserts on RESPONSE DATA CONTENT (ID-set intersection is empty), not on port binding.
   `POST /dtu/configure` (secondary override, INV-CONFIGURE-ENDPOINT-SECONDARY-001) is NOT
   used — config-time seeding via CloneConfig.seed + CloneConfig.org_id is the primary path.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Org registered in prism.toml but no `customers/org-x/` overlay directory exists | Boot step 9A uses base spec URL (production URL); demo fails with connection error to production URL. Test must not exercise this case — overlays must always exist for demo orgs. |
| EC-002 | Two orgs registered with same sensor — accidental same DTU socket | Under MultiInstanceHarness, each (org_slug, sensor_id) pair binds to a distinct ephemeral port selected by the OS. The harness returns `Err(HarnessError::DuplicateKey { org_slug, sensor_id })` if the same pair is submitted twice (BC-2.06.017 Postcondition 7). This means the "same socket for two orgs" scenario is structurally prevented at harness-startup time rather than being a silent config error. Test code must never submit duplicate (org_slug, sensor_id) pairs to MultiInstanceHarness::start. |
| EC-003 | Cross-org query (AC-005) returns empty data instead of AdapterNotFound | Empty data looks like a "soft pass" but is a BC-3.2.001 violation — the isolation error MUST be explicit. AC-005 asserts on the error code, not just "no rows". |
| EC-004 | Cyberint login step fails for org-b but succeeds for org-c | CookieLoginAuthProvider instances are independent; one failing doesn't affect the other. org-c's Cyberint query proceeds normally. |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| [TODO: populate per template] | pure-core / effectful-shell | [why] |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | [TODO] |
| Referenced code files | [TODO] |
| Test files | [TODO] |
| Tool outputs overhead | [TODO] |
| **Total** | **[TODO]** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **[TODO]%** |

---

## Dispatch Ordering in Critical Path

> **HISTORICAL (annotated v1.1, 2026-06-10):** the diagram below reflects the
> 2026-05-29 authoring-time plan. S-DEMO-003 has since MERGED (PR #176,
> 2026-06-08) ahead of this story, so the `blocks: [S-DEMO-003]` edge was
> scrubbed from frontmatter (see frontmatter annotation). The isolation
> assertions remain a precondition for PRESENTING the live demo — that
> sequencing is carried by the demo-objective ordering (T5 Story B → T6 → T8),
> not by a story-graph edge to an already-merged story.

```
S-DEMO-001 (boot step 9A + all 3 auth providers)
    │
    └── S-DEMO-002 (single-org 4-sensor E2E smoke test)
            │
            ├── S-DEMO-004 (multi-org isolation test) ← THIS STORY
            │
            └── S-DEMO-003 (runbook + scripts — MERGED PR #176 2026-06-08; edge historical)
```

S-DEMO-004 can run in parallel with S-DEMO-003 preparation, but S-DEMO-003 should not ship
until S-DEMO-004's isolation assertions pass. *(Historical note: S-DEMO-003 shipped first
in practice — PR #176, 2026-06-08 — under the live-demo objective's re-sequencing; the
isolation-verification obligation transfers to this story's own AC gate before demo day.)*

---

## AC-006 Design Directive (T8-architect, 2026-06-14 — for PO in T8-PO + story-writer in T9)

**ARCHITECT DESIGN DIRECTIVE — do NOT implement as prose AC yet. This section states what
AC-006 MUST prove and which BCs it traces to. The product-owner finalizes AC body in T8-PO;
the story-writer propagates to behavioral_contracts frontmatter and AC text in T9.**

### What AC-006 must prove

AC-006 must assert REAL seeded-data distinctness between orgs sharing the same sensor type.
The specific proof target: org-a's CrowdStrike DTU clone and org-c's CrowdStrike DTU clone
are constructed with distinct `(seed, org_id)` pairs (e.g., seed_a=100, seed_c=200, with
org_id_a ≠ org_id_c). When `tool_query "FROM crowdstrike_detections LIMIT 50"` is executed
for org-a and then for org-c, the test collects the set of record IDs (device IDs or
detection IDs) from each response and asserts:

```
ids_org_a ∩ ids_org_c = ∅   (INV-DISTINCT-DATA-001)
```

This is a CONTENT-LEVEL assertion on actual response data, NOT a port-address assertion.
The canonical device ID format is `"dev-{8hex}-{seed}-{n}"` where `{8hex} = hex(org_id.as_bytes()[0..4])` (ADR-036 v2.0 §2.2), so structural disjointness is guaranteed by the seeding math — but the test must still READ the response bodies and extract IDs rather than asserting only on socket addresses. CRITICAL: `{8hex}` is derived from the org's UUID bytes (e.g., if org-a's UUID bytes begin `0xDEADBEEF...`, IDs begin `dev-deadbeef-100-0`), NOT the human slug `org-a`. Asserting against `dev-org-a-...` would match zero IDs, making `ids_a ∩ ids_c = ∅` pass VACUOUSLY (false green), defeating the INV-DISTINCT-DATA-001 proof.

### BCs AC-006 should trace to

The PO must add to AC-006's traces annotation:
- `BC-2.06.017` — the `MultiInstanceHarness` / `socket_map` mechanism provides distinct
  sockets; each org's overlay file points to its own `SocketAddr`. This is Postcondition 3
  of BC-2.06.017 (overlay integration end-to-end).
- `BC-2.06.018` — the `new_with_seed(seed, archetype, org_id)` constructor causes each
  clone to serve deterministically distinct data. INV-DISTINCT-DATA-001 (§Invariants) is
  the invariant AC-006 verifies at the integration-test level.
- `BC-2.06.014` — per-org ResolvedSensorSpec routing (the overlay-to-FanOutTarget chain
  that ensures the correct socket is reached) is the production-side prerequisite.

The current AC-006 traces line reads `(traces to BC-2.06.014 precondition...)`. The PO
should REPLACE this with: `(traces to BC-2.06.017 Postcondition 3 + BC-2.06.018
INV-DISTINCT-DATA-001 + BC-2.06.014 endpoint-resolution)`.

### Consequential notes for AC-002, AC-003, and AC-009

These ACs currently assert "Returns non-empty data" without specifying whether that data
must be seeded-distinct from other orgs. Per the D-1077 mandate:

- **AC-002 (org-a CrowdStrike + Armis):** "non-empty data from org-a's CrowdStrike DTU clone"
  is still correct as stated, but the PO should note that the returned data is generated from
  org-a's seed (not the static JSON fallback). The trace should add BC-2.06.018 as a secondary
  reference. No structural change required — AC-006 carries the cross-org distinctness proof.

- **AC-003 (org-b Claroty + Cyberint):** Same as AC-002. "non-empty data" is correct; add
  BC-2.06.018 secondary trace if the PO wants coverage completeness. Not a blocker.

- **AC-009 (concurrent queries, org-a vs org-c CrowdStrike):** The existing prose — "org-a's
  response contains only data from CS_PORT_ORG_A; org-c's response contains only data from
  CS_PORT_ORG_C; no row-level mixing occurs" — is directionally correct but should be
  strengthened: "no row-level mixing occurs" should be verified by asserting that the
  ID sets from the two concurrent responses satisfy `ids_a ∩ ids_c = ∅`, consistent with
  AC-006. Port-address cross-reference alone is insufficient under D-1077.

### BC-anchor recommendation for PO (behavioral_contracts frontmatter additions)

Add to `behavioral_contracts:` array in the story frontmatter (for PO in T8-PO):
```yaml
  - BC-2.06.017  # MultiInstanceHarness multi-address binding — socket_map provides per-org
                 # distinct DTU sockets; overlay_wiring writes per-org overlay TOML.
                 # Merged PR #187 2026-06-14 (D-1158). AC-006 traces to PC-3.
  - BC-2.06.018  # Config-time data seeding — new_with_seed wires per-clone seed→generator;
                 # INV-DISTINCT-DATA-001 proves disjoint ID sets across distinct (seed, org_id).
                 # Merged PR #181 2026-06-10 (D-1089). AC-006 is the integration-level proof.
```

### ADR decision (architect T8)

**No ADR amendment is required.** The reconciliation is fully covered by existing ADRs:
- ADR-029 governs per-org overlay routing (base_url override per org) — unchanged; still
  the mechanism by which overlay TOML files route each org's adapter to its own DTU socket.
- ADR-036 governs the seeding mechanism (new_with_seed, canonical ID format, INV-DISTINCT-DATA-001
  math). ADR-036 v2.3 (current) is already the authoritative design document; no amendment
  is needed to extend its scope to S-DEMO-004 — S-DEMO-004 is a consumer of ADR-036, not a
  modifier of it.
- BC-2.06.017 (ACTIVE v1.12) and BC-2.06.018 (ACTIVE v1.6) are the canonical contract
  documents for the mechanisms this story consumes. Both BCs already describe the behavior
  S-DEMO-004 relies on; referencing them from this story's frontmatter and AC traces is
  sufficient. No new ADR category (routing, seeding, testing) is opened by this reconciliation.

The story's architecture sections and AC-006 design directive (above) provide the full
cross-referencing needed for the implementer to understand how to use the merged substrate.

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.16 | 2026-08-02 | story-writer | Added ## Authority section (DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 Round 6, D-2084). Synced stale `**Version:**` pseudo-field from v1.15 to v1.16 to match frontmatter (TD-VSDD-060 sibling-sweep correction, orchestrator-authorized). |
| 1.15 | 2026-07-17 | story-writer | F-ADMTOK-P15-MED-002: BC-2.06.017 canonical-contract pin updated v1.10 → v1.12 (bumped v1.10→v1.11 by DEFECT-ADMINTOKEN FIX-BURST-1; v1.11→v1.12 by FIX-BURST-11). BC-2.06.018 pin (v1.6) verified current against BC-2.06.018 frontmatter — no change. POL-23/POL-25 sibling sweep within S-DEMO-004: line 681 was the only live canonical-contract pin site; all other BC-2.06.017/BC-2.06.018 occurrences are non-versioned references or story-changelog historical records — no further updates required. Body version header synced v1.14 → v1.15 (POL-23). |
| 1.14 | 2026-06-14 | story-writer | F-PR3-MED-002 fix: AC-004 Red Gate test name corrected. `test_BC_3_2_001_org_c_all_4_sensors_return_independent_data` → `test_BC_2_01_013_org_c_all_4_sensors_return_independent_data`. The old prefix `BC_3_2_001` self-contradicted the AC-004 trace (`BC-2.01.013`) and did not match the actual test in `crates/prism-bin/tests/e2e_multi_org.rs` (evidence-report Red Gate table). Only one occurrence of the old test name existed in the body (AC-004); confirmed single-site via grep before edit. Body version header synced from v1.13 → v1.14 (POL-23 sibling-sweep propagation completing this burst). |
| 1.13 | 2026-06-14 | product-owner | PR #188 F-PR3-MED-001: AC-003 query corrected. `tool_query "FROM claroty_assets LIMIT 5"` → `tool_query "SELECT * FROM claroty_alerts LIMIT 5"`. Two errors fixed: (1) wrong table name `claroty_assets` → canonical `claroty_alerts`; (2) bare `FROM ... LIMIT` pipe syntax → valid PrismQL `SELECT * FROM ... LIMIT`. Now matches the actual test in `crates/prism-bin/tests/e2e_multi_org.rs` and the evidence-report (POL-22 verified: `claroty_assets` appears at exactly one site; that site is AC-003 body, now corrected). |
| 1.12 | 2026-06-14 | story-writer | POL-8 body propagation for BC-2.22.001 + BC-2.09.008 added in v1.11 frontmatter. Added two rows to §Behavioral Contracts table: BC-2.22.001 "Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate" (AC-001, AC-010) and BC-2.09.008 "Response Envelope with Trust Annotations" (AC-008). Updated AC-001 trace annotation to cite BC-2.22.001 by full verbatim title. Updated AC-008 trace annotation to cite BC-2.09.008 by full verbatim title. Updated AC-010 trace annotation to cite BC-2.22.001 by full verbatim title. Bidirectional AC↔BC traces satisfied for both new BCs. |
| 1.11 | 2026-06-14 | product-owner | Add BC-2.22.001 + BC-2.09.008 to behavioral_contracts array (PR #188 PR-LEVEL MED-1). AC-001 and AC-010 trace to BC-2.22.001 ("Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate"); AC-008 traces to BC-2.09.008 ("Response Envelope with Trust Annotations"). Both BCs were active in BC-INDEX but missing from frontmatter array. Body BC table and AC-trace column propagation deferred to story-writer (POL-8). |
| 1.10 | 2026-06-14 | story-writer | LOCAL adversary PASS-3 LOW-1 §File-Structure correction. Renamed NOTE row header from `crates/prism-bin/tests/e2e_multi_org.rs (regression guard)` to `crates/prism-bin/tests/bc_2_10_006_mcp_stdout_purity.rs`; action changed from NOTE to CREATE (standalone file, not part of e2e_multi_org.rs); description updated to cite BC-2.10.006 §Postconditions as the stdout-purity invariant source (stdout reserved for MCP JSON-RPC per BC-2.10.006, enforced at the step1_init_tracing emission site). No BC-2.22.001→BC-2.10.006 body corrections required: AC-001 and AC-010 reference BC-2.22.001 for boot sequencing and startup determinism respectively — those are correct semantics for BC-2.22.001 and are unrelated to the stdout-purity invariant. Status: ready. |
| 1.9 | 2026-06-14 | story-writer | LOCAL adversary PASS-2 §File-Structure accuracy fixes (M2-01, O2-01, O2-02) + AC-009 test-rename reference + M2-02 regression-guard note. M2-01: added `crates/prism-bin/src/boot.rs \| MODIFY` row documenting that `step1_init_tracing` forces all fmt layers to stderr (`.with_writer(std::io::stderr)`) to uphold MCP-stdout-purity invariant (fix-burst commit a7c9cbf0). O2-01: fixture path corrected from `crates/prism-bin/tests/fixtures/multi-org-prism.toml.template` → `crates/prism-bin/fixtures/multi-org-prism.toml.template` (crate-level `fixtures/`, no `tests/` prefix). O2-02: helper signatures corrected — `write_multi_org_overlays` now typed as `(&BackgroundHarness, &TempDir)` and documented to call `write_overlay_from_socket_map(harness.socket_map(), specs_dir)` internally (not `write_overlay_temp_dir`); `write_multi_org_prism_toml` now typed as `(&TempDir)` only, no socket_map arg; `BackgroundHarness` wrapper described (background thread, dedicated multi-thread runtime, oneshot for socket_map, SyncSender shutdown, Drop joins thread). Task 9: `concurrent`/`tokio::join!` language replaced with `sequential`/back-to-back/single-channel language; test name updated to `test_BC_2_11_005_sequential_org_queries_do_not_interfere`. M2-02: added NOTE row for MCP-stdout cleanliness regression guard (green guard, not Red Gate AC test; does not change acceptance_criteria_count=10 or red_gate_tests=4). Status: ready. |
| 1.8 | 2026-06-14 | product-owner | LOCAL adversary PASS-2 L2-01 adjudication — AC-009 prose corrected (decision A). `tokio::join!`/simultaneously language removed; AC-009 now specifies rapid sequential/back-to-back dispatch on the single MCP stdio channel. Architecture rationale added inline: per-analyst stdio is a serialized single-channel protocol (AD-013 / deployment model) — a single client cannot issue genuinely concurrent requests over one pipe; BC-2.11.005 ephemeral materialization property is fully proven by sequential dispatch + `ids_a ∩ ids_c = ∅`. AC-009 title changed from "Concurrent queries..." to "Sequential cross-org queries do not interfere..." to match. Test rename guidance added: `test_BC_2_11_005_concurrent_org_queries_do_not_interfere` → `test_BC_2_11_005_sequential_org_queries_do_not_interfere` (story-writer/implementer to apply). BC-2.11.005 trace retained; BC-2.06.018 trace retained. §File Structure / other ACs NOT touched (story-writer pass follows). |
| 1.7 | 2026-06-14 | story-writer | LOCAL adversary PASS-1 O-01 closure — spec reconciliation only, no code touched. (1) `crates_touched` frontmatter: `[prism-bin]` → `[prism-bin, prism-dtu-harness]`; inline comment added explaining the wiring-not-redesign rationale (`write_overlay_from_socket_map` added for BackgroundHarness off-runtime pattern; `write_overlay_temp_dir` refactored to delegate; behavior-preserving; covered by e2e + harness tests). (2) §File Structure Requirements: added `crates/prism-dtu-harness/src/overlay_wiring.rs \| MODIFY` row documenting `write_overlay_from_socket_map` addition and `write_overlay_temp_dir` delegation refactor. Status: ready. Counts unchanged (acceptance_criteria_count=10, red_gate_tests=4). |
| 1.6 | 2026-06-14 | story-writer | Pre-TDD remove-uncertainty re-run correction (D-1110). Single fix: §File Structure `crates/prism-bin/Cargo.toml` MODIFY row item (2) — corrected mis-framing of `prism-dtu-common` from ADD to MODIFY. Ground truth (verified against `crates/prism-bin/Cargo.toml [dev-dependencies]`): `prism-dtu-common` is ALREADY present as a dev-dep with `features = ["dtu"]` (added by S-DEMO-QUERY-PUSHDOWN-001). On `prism-dtu-common`, `dtu` and `fixture-gen` are INDEPENDENT features (`dtu` does NOT transitively enable `fixture-gen`), so BOTH must be listed. Corrected wording: "MODIFY the EXISTING `prism-dtu-common` dev-dep: `features = ["dtu"]` → `features = ["dtu", "fixture-gen"]`" with rationale that adding it as a new dep would either produce a duplicate `prism-dtu-common` key (Cargo error) or overwrite `["dtu"]` → `["fixture-gen"]` alone (dropping the `dtu` activation that push-down tests rely on). Other dev-dep instructions confirmed CORRECT and unchanged: `prism-dtu-armis` + `prism-dtu-crowdstrike` remain MODIFYs (`["dtu"]` → `["dtu","fixture-gen"]`); `prism-dtu-claroty` + `prism-dtu-cyberint` remain ADDs with `["dtu","fixture-gen"]`; `prism-dtu-harness` remains ADD with `["dtu"]`. Status kept: ready. |
| 1.5 | 2026-06-14 | story-writer | Remove-uncertainty spec corrections (6 fixes). FIX-1: `.cargo/nextest.toml` → `.config/nextest.toml` in §File Structure and Task 10 (the `.cargo/` path does not exist; nextest reads `.config/nextest.toml`, which already has `[profile.e2e]` and `[profile.ci]`; new block mirrors existing pattern). FIX-2: Added `crates/prism-bin/Cargo.toml` MODIFY row to §File Structure with full enumeration of required dev-dep changes: `fixture-gen` feature on existing `prism-dtu-armis` + `prism-dtu-crowdstrike`; new dev-deps `prism-dtu-claroty` + `prism-dtu-cyberint` (with `["dtu","fixture-gen"]`); MODIFY existing `prism-dtu-common` dev-dep (`["dtu"]` → `["dtu","fixture-gen"]`); ADD `prism-dtu-harness` (with `["dtu"]`). (Note: v1.5 originally listed `prism-dtu-common` as a new ADD — corrected to MODIFY in v1.6; the enumeration here is updated to reflect the ground-truth framing.) INV-PERIMETER-001 note preserved. FIX-3: `HarnessEntry` construction guidance changed from struct-literal to `HarnessEntry::new(org_slug, sensor_id, clone)` in Task 2, Task 3, and risk_mitigations; `#[non_exhaustive]` + E0639 rationale stated explicitly. FIX-4: All `write_overlay_temp_dir(&harness, &tempdir)` → `write_overlay_temp_dir(&harness, tempdir.path())` in risk_mitigations (1 occurrence), §File Structure helpers/mod.rs row (1 occurrence), Open Questions OQ-1 (1 occurrence); Task 2 and Task 3 already use `.path()` form. FIX-5: `new_with_seed` fallibility and arg types corrected in §DTU multi-tenancy scope and Open Questions OQ-2: `CrowdstrikeClone` and `ClarotyClone` → infallible (`Self`); `ArmisClone` and `CyberintClone` → fallible (`anyhow::Result<Self>`, use `?`); `archetype` is `prism_dtu_common::Archetype` enum; `org_id` is `prism_dtu_common::OrgId([u8;16])` constructed `OrgId(*uuid::Uuid::parse_str(s)?.as_bytes())`; reference pattern `build_clone_pairs` in `prism-dtu-demo-server/src/harness.rs`. FIX-6: Standardized all device-ID format occurrences to `"dev-{8hex}-{seed}-{n}"` where `8hex = hex(org_id.as_bytes()[0..4])`; eliminated all `"dev-{org_slug}-{seed}-{n}"` forms (2 occurrences: frontmatter comment line 48, §AC-006 Design Directive); added false-green trap warning in AC-006 body, AC-009 body, frontmatter comment, §AC-006 Design Directive, risk_mitigations, and OQ-2 — asserting `"dev-org-a-..."` matches zero IDs, making intersection vacuously ∅. BONUS (LOW): Task 1 and §File Structure helpers/mod.rs row updated to note that `write_multi_org_demo_config` and `McpStdioHandle::tool_query_scoped` / `tool_query_scoped_expect_rpc_error` already exist in helpers/mod.rs and should be reused for AC-002/003/005. |
| 1.4 | 2026-06-14 | story-writer | T9 materialization — AC-trace propagation + real-seeding body propagation + status draft→ready. (1) AC-006 body rewritten: replaced port-binding-only model with content-level INV-DISTINCT-DATA-001 proof: org-a CrowdStrike clone constructed via `new_with_seed(seed_a=100, archetype, org_id_a)` and org-c via `new_with_seed(seed_c=200, archetype, org_id_c)`; test reads response bodies and asserts `ids_org_a ∩ ids_org_c = ∅`; trace updated to `BC-2.06.017 Postcondition 3 + BC-2.06.018 INV-DISTINCT-DATA-001 + BC-2.06.014 endpoint-resolution`; Red Gate test renamed to `test_BC_2_06_018_per_org_seeded_data_is_disjoint`. (2) AC-009 strengthened: "no row-level mixing occurs" extended to assert `ids_a ∩ ids_c = ∅` on concurrent response bodies; added `BC-2.06.018 INV-DISTINCT-DATA-001` as additional trace. (3) AC-007: added `BC-2.06.017 INV-ISOLATION-001 (per-org distinct Cyberint DTU sockets)` trace and clarified distinct-socket setup. (4) §risk_mitigations: first mitigation rewritten to describe MultiInstanceHarness::start + new_with_seed per-org seeding + write_overlay_temp_dir + ids_a ∩ ids_c = ∅ content assertion. (5) §Tasks: tasks 2, 3, 5, 7, 9 rewritten for MultiInstanceHarness API; new task 4 separates prism.toml writing from overlay TOML; retired stale "DTU demo server invocations" language. (6) §File Structure Requirements: `write_multi_org_config()` and `MultiOrgDtuPorts` replaced with `start_multi_org_harness()`, `write_multi_org_overlays()`, `write_multi_org_prism_toml()` reflecting harness API; retired `MultiOrgDtuPorts` struct. (7) EC-002: reconciled "same port config mistake" edge case — now documents that MultiInstanceHarness prevents this structurally via HarnessError::DuplicateKey (BC-2.06.017 Postcondition 7). (8) §Architecture Compliance Rules: added canonical ID format rule referencing ADR-036 v2.0 §2.2; updated AC-006/AC-007/AC-009 rule rows to reference seeding semantics and content-level proofs; updated §Behavioral Contracts table AC-trace columns for BC-2.06.017 and BC-2.06.018. Status set to ready. |
| 1.3 | 2026-06-14 | product-owner | T8-PO BC anchor addition. (1) `behavioral_contracts:` frontmatter array: added BC-2.06.017 (MultiInstanceHarness multi-address binding; PC-3 + INV-ISOLATION-001 underpin AC-006 distinct-socket claim, AC-007 per-org Cyberint session isolation, AC-009 concurrent no-mix) and BC-2.06.018 (config-time seeding; INV-DISTINCT-DATA-001 is the contract backing AC-006's content-level ids_org_a ∩ ids_org_c = ∅ assertion). (2) §Behavioral Contracts table: added two new rows with AC-trace column. (3) Story body version header updated v1.0→v1.3. BC-2.06.017 and BC-2.06.018 required NO amendment — both BCs already support the AC-006/007/009 claims as written (analysis in T8-PO report). AC-006 trace directive for story-writer T9 recorded in §AC-006 Design Directive (no AC prose touched). |
| 1.2 | 2026-06-14 | architect | T8 reconciliation against merged capability stack (D-1077 scope expansion execution). Changes: (1) `depends_on` — added 3 missing edges: S-DEMO-MULTI-TENANT-DTU-001 (BC-2.06.017 multi-address binding, PR #187 SATISFIED), S-DEMO-DTU-LIVE-SCENARIO-001-A (BC-2.06.018 seeding, PR #181 SATISFIED), S-DEMO-DTU-LIVE-SCENARIO-001-B (BC-2.06.019+020, PR #185 SATISFIED); inline justification per existing comment convention. (2) `inputs` — added prism-dtu-harness multi_instance.rs + overlay_wiring.rs, BC-2.06.017 + BC-2.06.018, ADR-036. (3) `title` — updated to reflect BC-2.06.017 + BC-2.06.018 anchors. (4) §DTU multi-tenancy scope — complete rewrite: committed to MultiInstanceHarness + real seeded data distinctness (INV-DISTINCT-DATA-001) as the primary isolation proof; port-binding-only approach retired. (5) §Open Questions — both questions fully resolved (OQ-1: MultiInstanceHarness single-call handles all 8 entries; OQ-2: real seeded data mandatory, port-binding insufficient). (6) §Architecture Compliance Rules — added 3 new rules (CONTENT-level AC-006 proof; seed distinctness per org; MultiInstanceHarness as spawn mechanism); added §Architecture Compliance Note documenting prism-bin perimeter vs prism-dtu-harness perimeter (T6 lesson z23). (7) §AC-006 Design Directive — new section: precise spec for what AC-006 must prove (INV-DISTINCT-DATA-001 content assertion), BC-anchor recommendations for PO (BC-2.06.017 PC-3 + BC-2.06.018 INV-DISTINCT-DATA-001 + BC-2.06.014), consequential notes for AC-002/003/009, and ADR decision (no amendment needed — ADR-029 + ADR-036 v2.3 sufficient). AC body prose NOT modified (PO domain, T8-PO). |
| 1.1 | 2026-06-10 | story-writer | Moot `blocks: [S-DEMO-003]` frontmatter edge scrubbed to `blocks: []` with historical annotation — S-DEMO-003 merged PR #176 (2026-06-08) ahead of this draft story; a merged story cannot be blocked and the stale edge would mislead the wave scheduler. §Dispatch Ordering diagram + narrative annotated historical (isolation-verification obligation transfers to this story's AC gate before demo presentation). Index row already carried the note since STORY-INDEX v2.342; file now matches (story_frontmatter_index_consistency). No AC/scope changes. |
| 1.0 | 2026-05-29 | architect | Initial draft — addresses multi-client demo scope gap not covered by S-DEMO-002 v1.0 |
