---
document_type: story
story_id: S-DEMO-MULTI-TENANT-DTU-001
title: "prism-dtu-demo-server + prism-dtu-harness: Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing"
wave: 5
epic_id: E-DEMO
priority: P2
status: draft
# BC status: pending PO authorship
# S-7.01 gate: behavioral_contracts is empty — status MUST remain draft until PO
# authors BCs for per-instance multi-address binding behavior. Candidate BCs flagged
# below for PO (see §New-BC Flags).
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-06-03T00:00:00Z"
created: "2026-06-03"
modified: "2026-06-03"
tdd_mode: strict
subsystems: [SS-17]
# Subsystem anchor justifications:
#   SS-17 (DTU Clones) owns all prism-dtu-* crates including prism-dtu-demo-server and
#   prism-dtu-harness per ARCH-INDEX Subsystem Registry v2.105. Multi-address binding
#   is a DTU clone infrastructure concern — the demo server and harness each act as a
#   host for multiple in-process DTU clone instances, each bound to a distinct socket
#   address. No production sensor or spec-engine logic is touched; this is pure DTU
#   test-infrastructure work.
crates_touched: [prism-dtu-demo-server, prism-dtu-harness]
target_module: prism-dtu-demo-server
behavioral_contracts: []
# BC status: pending PO authorship — see §New-BC Flags for PO
verification_properties: []
depends_on:
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001
  # Dependency anchor: S-CONFIG-MULTI-TENANT-OVERRIDE-001 (merged PR #155) delivers
  # the ResolvedSensorSpec map and per-org overlay base_url plumbing in prism-spec-engine
  # and prism-sensors. This story's value proposition — testing that per-org base_url
  # overlays actually route to separate DTU instances — requires that plumbing to be
  # in place. Without the overlay mechanism, per-instance routing cannot be tested end
  # to end. S-CONFIG-MULTI-TENANT-OVERRIDE-001 is status: merged (develop@3e822522),
  # so the dependency is already satisfied.
blocks: []
points: 8
# Points justification:
#   prism-dtu-demo-server multi-address binding:
#   - Read current DemoServer / ServerConfig API and BehavioralClone trait: 0.5 pts
#   - Design per-instance SocketAddr registry: 0.5 pts
#   - Implement multi-instance bind for prism-dtu-armis and prism-dtu-claroty: 1.5 pts
#   - CLI / config extension (--multi-tenant flag or config file): 1 pt
#   prism-dtu-harness multi-instance wiring:
#   - Extend harness fixture API to start multiple clone instances at distinct addrs: 1 pt
#   - Wire per-org base_url into ResolvedSensorSpec for harness tests: 1 pt
#   Red Gate tests (≥15 per BC-5.38.001):
#   - 15+ test stubs × ~10 min each: 1.5 pts
#   Integration + pre-push gate: 0.5 pts
#   Total: ~7.5 pts → 8 pts
estimated_days: 3
risk: MEDIUM
# Risk justification:
#   The primary risk is that prism-dtu-demo-server currently binds to a single SocketAddr
#   per clone type. Adding multi-instance support requires a non-trivial API extension to
#   BehavioralClone::start_on (or a new binding entry point). The harness wiring introduces
#   a second risk: prism-dtu-harness::clones use in-process routers (not separate process
#   sockets), so "multi-address" in the harness context requires spawning multiple tokio
#   tasks with distinct bind addresses. Cross-crate coordination between prism-dtu-common
#   (shared BehavioralClone trait), prism-dtu-demo-server (orchestration), and
#   prism-dtu-harness (test fixture) must remain backward compatible.
acceptance_criteria_count: 9
red_gate_tests: 15
estimated_passes: "2-4 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Backward compatibility of BehavioralClone::start_on: any extension to the trait must
    use #[cfg(feature = ...)] or an additional optional method to avoid breaking existing
    single-instance callers (S-DEMO-002 integration tests, S-6.07/6.08/6.09/6.10 parity tests)."
  - "In-process harness vs out-of-process demo-server: these are separate execution models.
    prism-dtu-harness spawns router tasks in the same tokio runtime; prism-dtu-demo-server
    spawns separate OS-level processes (or tasks). The multi-address binding implementation
    must be consistent within each crate's model but need not unify the two models."
  - "reqwest::Client timeout: any new HTTP client construction uses .timeout(Duration::from_secs(30))
    per CLAUDE.md conventions. The per-org routing tests that fan out to multiple DTU addresses
    must not use default (infinite) timeouts."
inputs:
  - "crates/prism-dtu-demo-server/src/main.rs"
  - "crates/prism-dtu-demo-server/src/server.rs"
  - "crates/prism-dtu-harness/src/lib.rs"
  - "crates/prism-dtu-harness/src/clones/armis.rs"
  - "crates/prism-dtu-harness/src/clones/claroty.rs"
  - "crates/prism-dtu-common/src/lib.rs"
  - ".factory/stories/S-CONFIG-MULTI-TENANT-OVERRIDE-001-per-org-sensor-endpoint-overlay-loading.md"
  - ".factory/specs/behavioral-contracts/"
input-hash: null
traces_to:
  - "S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-009"
  - "D-849"
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-MULTI-TENANT-DTU-001 v1.0 — Per-DTU-Instance Multi-Address Binding

**Story ID:** S-DEMO-MULTI-TENANT-DTU-001
**Status:** draft (pending PO BC authorship)
**Version:** v1.0
**Wave:** 5
**Priority:** P2
**Points:** 8

---

## Origin

Surfaced by S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-009 (DTU multi-tenant emulation gap,
D-849). AC-009 documents:

> **Case B (gap acknowledged):** The `prism-dtu-demo-server` or individual DTU clones
> (`prism-dtu-armis`, `prism-dtu-claroty`) do NOT currently support binding multiple
> network addresses to simulate different tenant instances. This means the full "org A
> to instance A, org B to instance B" routing cannot be tested against separate DTU
> processes in this story.

This story closes that gap. After merging, each tenant's per-org `base_url` overlay can
point to a distinct DTU clone address, proving that the S-CONFIG overlay mechanism routes
org A's queries exclusively to instance A and org B's queries exclusively to instance B —
satisfying the ADR-031 §D5 DTU=true-DTU validation principle for multi-tenant deployments.

**Non-blocking rationale:** This story is P2 (non-blocking for single-tenant demo). The
single-tenant demo path (one DTU, all orgs pointing to the same address) was verified by
S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-009 Case A and the tests in S-DEMO-002. This story
adds the full per-org-to-per-instance routing test that Case B deferred.

---

## Narrative

As a test infrastructure consumer running multi-tenant end-to-end tests,
I want `prism-dtu-demo-server` to support binding separate DTU clone instances to distinct
network addresses (one per simulated tenant), and `prism-dtu-harness` to expose a
multi-instance fixture API that wires per-org `base_url` overlays to distinct clone sockets,
so that the per-tenant endpoint overlay mechanism (S-CONFIG-MULTI-TENANT-OVERRIDE-001) can
be validated end-to-end against realistic per-org DTU instances — proving that org A's
queries go to instance A and org B's queries go to instance B, with zero cross-tenant
URL leakage.

---

## Story-Level Goal

After this story merges:

1. `prism-dtu-demo-server` can start two named DTU clone instances of the same sensor type
   (e.g., two `prism-dtu-armis` instances, one at `127.0.0.1:0` for org A and one at
   `127.0.0.1:0` for org B) and expose their distinct `SocketAddr`s to the test harness.

2. `prism-dtu-harness` exposes a `MultiInstanceHarness` fixture builder (or equivalent
   per-instance binding API) that:
   - Starts N named DTU clone instances at ephemeral ports.
   - Returns a map of `(org_slug, sensor_id) → SocketAddr` suitable for constructing
     `SensorInstanceOverlay { base_url: addr.to_string() }` entries.
   - Integrates with the `ResolvedSensorSpec` map from S-CONFIG-MULTI-TENANT-OVERRIDE-001.

3. At least one multi-tenant routing integration test (Red Gate level) drives:
   - org A → armis instance A (distinct address).
   - org B → armis instance B (distinct address).
   - Asserts instance A received ONLY org A's requests; instance B received ONLY org B's.
   - Zero cross-tenant URL leakage (AC-008-class paper-fix resistance).

4. All existing single-instance DTU parity tests continue to pass (backward compatibility).

---

## Behavioral Contracts

Pending PO authorship. Candidate BCs:

| BC (candidate) | Title | Why relevant |
|----------------|-------|-------------|
| (new — pending PO) | Per-DTU-Instance Multi-Address Binding for Multi-Tenant Testing | Governs the multi-address bind semantics, the per-instance SocketAddr registry API, and the no-cross-tenant-leakage invariant |
| BC-2.06.014 | Instance Identity Resolution at Fanout | Governs (org_id, sensor_id) → ResolvedSensorSpec lookup and overlay base_url routing; this story's integration tests exercise that contract end-to-end against real DTU sockets |

PO must author canonical BCs and set `behavioral_contracts:` before `status: ready`.

---

## New-BC Flags for Product-Owner

**Flag 1 (REQUIRED — NEW BC):** No existing BC governs multi-address binding in
`prism-dtu-demo-server` or `prism-dtu-harness`. A new BC is needed to specify:
- The `MultiInstanceConfig` (or equivalent) API contract: how the demo server accepts
  N instance addresses per sensor type.
- The `MultiInstanceHarness` fixture API contract: the per-org address map interface.
- The no-cross-tenant-leakage invariant: requests for org A MUST NOT reach instance B.
- Backward-compatibility invariant: single-instance callers MUST NOT be broken.

**Flag 2 (CONFIRM):** Does BC-2.06.014 need a new postcondition for the "multi-DTU
routing verified end-to-end" scenario (as opposed to the mock-HTTP-server test in AC-003
of S-CONFIG-MULTI-TENANT-OVERRIDE-001)? PO should decide whether to amend BC-2.06.014
or rely solely on the new BC for this story.

---

## Acceptance Criteria

### AC-001: prism-dtu-demo-server accepts multi-instance bind configuration
`prism-dtu-demo-server` accepts a configuration (via `ServerConfig` extension, CLI flag,
or TOML config block) that specifies N named instances of a sensor DTU type, each with an
independent `bind` address. On startup, each instance binds to its configured address
(or an ephemeral port if address is `127.0.0.1:0`) and the bound `SocketAddr` is exposed
to callers.
(traces to BC-TBD postcondition — multi-instance bind configuration accepted; pending PO authorship)

Red Gate test: `test_demo_server_multi_instance_bind_config_accepted`

### AC-002: Two armis instances start on distinct ephemeral ports
Given `MultiInstanceConfig { instances: [("armis-acme", "127.0.0.1:0"), ("armis-contoso", "127.0.0.1:0")] }`,
when the demo server starts the armis DTU with multi-instance config, two separate `ArmisClone`
instances start on distinct ephemeral ports. The returned `SocketAddr` map has two entries
with distinct addresses.
(traces to BC-TBD postcondition — two instances at distinct addresses; pending PO authorship)

Red Gate test: `test_demo_server_two_armis_instances_bind_distinct_ports`

### AC-003: Two claroty instances start on distinct ephemeral ports
Same as AC-002 but for `ClarotyClone`. Two separate Claroty instances start on distinct
ephemeral ports. Response to `POST /api/v1/alerts/` from each instance returns that
instance's fixture data independently.
(traces to BC-TBD postcondition; pending PO authorship)

Red Gate test: `test_demo_server_two_claroty_instances_bind_distinct_ports`

### AC-004: prism-dtu-harness MultiInstanceHarness builds per-org SocketAddr map
`prism-dtu-harness` exposes a `MultiInstanceHarness` (or equivalent API) that:
- Accepts a list of `(org_slug: &str, sensor_id: &str, clone: Box<dyn BehavioralClone>)` entries.
- Starts each clone at an ephemeral port.
- Returns `HashMap<(OrgSlug, SensorId), SocketAddr>` for use in overlay construction.
This API is usable from integration tests without requiring prism-dtu-demo-server.
(traces to BC-TBD postcondition; pending PO authorship)

Red Gate test: `test_harness_multi_instance_builds_per_org_socket_map`

### AC-005: Per-org base_url overlay integrates with MultiInstanceHarness output
Given two `ArmisClone` instances at distinct sockets (from `MultiInstanceHarness`), and
per-org overlays constructed as:
```
customers/acme/armis.sensor.toml → base_url = "http://127.0.0.1:{acme_port}"
customers/contoso/armis.sensor.toml → base_url = "http://127.0.0.1:{contoso_port}"
```
when `SpecLoader::load_all` processes these overlays (S-CONFIG-MULTI-TENANT-OVERRIDE-001),
the `ResolvedSensorSpec` map entries for `(acme, armis)` and `(contoso, armis)` carry
distinct `base_url` values corresponding to the two instances.
(traces to BC-2.06.014 postcondition Case A — overlay base_url used at HTTP dispatch;
and BC-TBD per-instance overlay integration)

Red Gate test: `test_multi_instance_overlay_loads_distinct_base_urls`

### AC-006: Org A's requests reach instance A exclusively; org B's reach instance B
Given the two-instance setup from AC-005, when `FanOutTarget` dispatches for `(acme, armis)`,
ALL requests go to the acme instance socket. When `FanOutTarget` dispatches for `(contoso, armis)`,
ALL requests go to the contoso instance socket. Zero requests from org A reach org B's instance
socket; zero requests from org B reach org A's instance socket.

This is the core multi-tenant routing isolation proof. It is the DTU-grounded equivalent of
S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-008 (paper-fix resistance) — proving that per-org
base_url is consumed by the live HTTP dispatch layer, not merely stored in the spec map.
(traces to BC-2.06.014 postcondition Case A — fanout uses overlay base_url at HTTP dispatch;
and BC-TBD no-cross-tenant-leakage invariant)

Red Gate test: `test_multi_tenant_routing_zero_cross_tenant_leakage`

### AC-007: Single-instance path unchanged (backward compatibility)
Existing single-instance callers of `ArmisClone::start_on(bind, ...)`,
`ClarotyClone::start_on(bind, ...)`, etc. are NOT modified and continue to work. All
existing parity tests in S-6.07–6.10 and S-DEMO-002 integration tests pass without
modification.

The multi-instance binding API is ADDITIVE — it does not change the signature of
`BehavioralClone::start_on`.
(traces to BC-TBD backward-compatibility invariant)

Red Gate test: `test_single_instance_path_unaffected_by_multi_instance_addition`

### AC-008: Module-doc and pub API documentation updated
Both `prism-dtu-demo-server` and `prism-dtu-harness` module-doc (or `README.md` equivalents)
are updated to document the multi-instance binding API. No undocumented public types.
The `MultiInstanceHarness` type (or equivalent) has doc comments on all public methods.
(traces to BC-TBD documentation completeness invariant)

### AC-009: SAP-1 sweep — no uncatalogued event_type emissions
Any new `tracing::*!(event_type = ...)` emission sites added by this story have corresponding
rows in BC-2.16.002 Structured Event Catalog (SAP-1 standing adversary probe). If no new
emissions are added, the SAP-1 sweep returns zero new sites.
(traces to SAP-1 + PG-LP11-001)

Red Gate test: `rg 'event_type\s*=' crates/prism-dtu-demo-server crates/prism-dtu-harness --type rust` (manual adversary sweep; zero uncatalogued sites expected)

---

## Red Gate Test Plan

Minimum 15 Red Gate tests per BC-5.38.001. All must FAIL before implementation (Red Gate
discipline — SID-1). Test crate: `prism-dtu-demo-server/tests/` and
`prism-dtu-harness/tests/`.

| Test Name | AC | Crate | Description |
|-----------|-----|-------|-------------|
| `test_demo_server_multi_instance_bind_config_accepted` | AC-001 | prism-dtu-demo-server | `MultiInstanceConfig` accepted without panic/error; returns non-empty instance map |
| `test_demo_server_two_armis_instances_bind_distinct_ports` | AC-002 | prism-dtu-demo-server | Two `ArmisClone` instances start at distinct `SocketAddr`s; neither is the same as the other |
| `test_demo_server_two_claroty_instances_bind_distinct_ports` | AC-003 | prism-dtu-demo-server | Two `ClarotyClone` instances start at distinct sockets; independent `POST /api/v1/alerts/` responses |
| `test_demo_server_instance_a_responds_independently` | AC-002 | prism-dtu-demo-server | Request to instance A socket returns instance A fixture data (not instance B) |
| `test_demo_server_instance_b_responds_independently` | AC-002 | prism-dtu-demo-server | Request to instance B socket returns instance B fixture data (not instance A) |
| `test_demo_server_multi_instance_shutdown_clean` | AC-002 | prism-dtu-demo-server | Both instances shut down cleanly when `shutdown` signal sent; no port leak |
| `test_demo_server_zero_instances_returns_empty_map` | AC-001 | prism-dtu-demo-server | Empty `MultiInstanceConfig` returns empty socket map; no panic |
| `test_harness_multi_instance_builds_per_org_socket_map` | AC-004 | prism-dtu-harness | `MultiInstanceHarness` returns `HashMap` with correct `(org_slug, sensor_id)` keys |
| `test_harness_distinct_org_slots_different_sockets` | AC-004 | prism-dtu-harness | Two orgs for the same sensor type → two distinct `SocketAddr` values |
| `test_multi_instance_overlay_loads_distinct_base_urls` | AC-005 | prism-dtu-harness | `ResolvedSensorSpec` map for `(acme, armis)` and `(contoso, armis)` have distinct `base_url` values |
| `test_multi_tenant_routing_zero_cross_tenant_leakage` | AC-006 | prism-dtu-harness | Org A dispatch → 0 requests at instance B; org B dispatch → 0 requests at instance A |
| `test_multi_tenant_routing_acme_instance_receives_acme_requests` | AC-006 | prism-dtu-harness | All of org A's dispatched requests arrive at instance A (exact count match) |
| `test_multi_tenant_routing_contoso_instance_receives_contoso_requests` | AC-006 | prism-dtu-harness | All of org B's dispatched requests arrive at instance B (exact count match) |
| `test_single_instance_path_unaffected_by_multi_instance_addition` | AC-007 | prism-dtu-harness | Existing single-instance `ArmisClone::start_on` call pattern compiles + works unchanged |
| `test_single_instance_parity_test_still_passes_after_multi_instance_addition` | AC-007 | prism-dtu-demo-server | Existing single-instance S-DEMO-002-style test still green after multi-instance API added |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `MultiInstanceConfig` (new struct) | `crates/prism-dtu-demo-server/src/server.rs` or `multi_instance.rs` | Pure (config struct) |
| Multi-instance bind logic | `crates/prism-dtu-demo-server/src/server.rs` | Effectful (binds sockets, spawns tokio tasks) |
| `MultiInstanceHarness` (new struct) | `crates/prism-dtu-harness/src/multi_instance.rs` | Effectful (binds sockets, wires overlay map) |
| Overlay + `ResolvedSensorSpec` wiring | `crates/prism-dtu-harness/src/multi_instance.rs` | Effectful (calls `SpecLoader::load_all` with overlay dir) |
| BehavioralClone trait | `crates/prism-dtu-common/src/lib.rs` | Effectful (existing; NOT modified) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-17 DTU Clones
- `architecture/dependency-graph.md` §Wave-5 DTU fidelity stories

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~5,000 |
| `crates/prism-dtu-demo-server/src/main.rs` (read) | ~2,000 |
| `crates/prism-dtu-demo-server/src/server.rs` (read) | ~3,000 |
| `crates/prism-dtu-harness/src/lib.rs` + clones/ (read) | ~5,000 |
| `crates/prism-dtu-common/src/lib.rs` — BehavioralClone trait (read) | ~2,000 |
| S-CONFIG-MULTI-TENANT-OVERRIDE-001 story (reference) | ~4,000 |
| BC-2.06.014 (reference for overlay base_url contract) | ~2,000 |
| BC files (pending PO authorship; ~2 new BCs) | ~4,000 |
| Test file output (cargo nextest) | ~1,500 |
| **Total estimate** | **~28,500 tokens (~11% of 256K context)** |

Well within 20-30% context budget.

---

## Tasks

1. **Read source files first** — `crates/prism-dtu-demo-server/src/main.rs`,
   `crates/prism-dtu-demo-server/src/server.rs` (current `ServerConfig` and startup API),
   `crates/prism-dtu-harness/src/lib.rs` + `clones/armis.rs` + `clones/claroty.rs`,
   `crates/prism-dtu-common/src/lib.rs` (`BehavioralClone` trait signature). Establish exact
   function signatures and startup patterns before writing any new code.

2. **Confirm PO-assigned BC IDs** — this story MUST NOT dispatch to `ready` until the PO
   has authored the new BC (§New-BC Flags Flag 1) and assigned canonical `BC-S.SS.NNN`
   identifiers. Use placeholders `BC-TBD` in ACs until PO confirms.

3. **Write Red Gate tests (stub phase — ALL must FAIL before implementation):**
   - All 15 tests listed in §Red Gate Test Plan.
   - Tests that require multi-instance binding must fail with `unimplemented!()` or
     compile error if the multi-instance API does not yet exist — that is the correct
     Red Gate failure mode.
   - Tests that verify single-instance backward compatibility must PASS in the pre-change
     state (these are regression guards, not forward-failing tests).

4. **Design `MultiInstanceConfig` struct** in `prism-dtu-demo-server`:
   ```rust
   // Sketch — implementer reads actual server.rs before finalizing
   #[non_exhaustive]
   pub struct MultiInstanceConfig {
       pub instances: Vec<InstanceEntry>,
   }
   #[non_exhaustive]
   pub struct InstanceEntry {
       pub name: String,       // e.g., "armis-acme"
       pub bind: SocketAddr,   // typically 127.0.0.1:0 for ephemeral
   }
   ```
   New public structs require `#[non_exhaustive]`. Bump `ci.yml` `EXPECTED` count for
   the compile-fail gate. Do NOT modify `BehavioralClone::start_on` signature.

5. **Implement multi-instance bind** in `prism-dtu-demo-server`:
   - Accept `MultiInstanceConfig`; for each `InstanceEntry`, spawn a `BehavioralClone`
     instance via the existing `start_on(bind, shutdown, tls)` API.
   - Collect the returned `SocketAddr`s into a `HashMap<String, SocketAddr>` (instance
     name → bound address).
   - Return the map to the caller.

6. **Design `MultiInstanceHarness`** in `prism-dtu-harness`:
   ```rust
   // Sketch — implementer reads harness lib.rs before finalizing
   #[non_exhaustive]
   pub struct MultiInstanceHarness {
       instances: HashMap<(OrgSlug, SensorId), SocketAddr>,
       shutdown: broadcast::Sender<()>,
   }
   impl MultiInstanceHarness {
       pub async fn start(entries: Vec<HarnessEntry>) -> anyhow::Result<Self>;
       pub fn socket_map(&self) -> &HashMap<(OrgSlug, SensorId), SocketAddr>;
   }
   ```
   New public types require `#[non_exhaustive]`. Bump `EXPECTED` accordingly.

7. **Wire overlay construction helper** — implement a helper that converts
   `MultiInstanceHarness::socket_map()` output into a temp directory of
   `customers/<org_slug>/<sensor_id>.sensor.toml` overlay files with
   `base_url = "http://{socket_addr}"`. This enables `SpecLoader::load_all` to pick
   up the per-instance addresses as overlays. The helper lives in
   `prism-dtu-harness/src/overlay_wiring.rs` (new file).

8. **Implement the multi-tenant routing isolation test** (`test_multi_tenant_routing_zero_cross_tenant_leakage`):
   - Start two `ArmisClone` instances via `MultiInstanceHarness`.
   - Construct per-org overlays pointing to distinct sockets.
   - Run `FanOutTarget` dispatches for both orgs.
   - Assert cross-tenant leakage count == 0 for each instance.
   - This test is the primary value of this story (AC-006).

9. **SAP-1 sweep** — `rg 'event_type\s*=' crates/prism-dtu-demo-server crates/prism-dtu-harness --type rust` —
   any new `event_type = "..."` emissions must have BC-2.16.002 catalog rows.

10. **Pre-push gate** — `just check` GREEN workspace-wide. Verify `EXPECTED` count in
    `ci.yml` is incremented for all new `#[non_exhaustive]` public types. No `--no-verify`.

---

## Previous Story Intelligence

This is the first story in E-DEMO explicitly targeting `prism-dtu-demo-server` multi-instance
binding. Key lessons from predecessor stories:

- **S-CONFIG-MULTI-TENANT-OVERRIDE-001 (merged PR #155):** Delivered the overlay mechanism,
  `SensorInstanceOverlay`, `ResolvedSensorSpec`, and `FanOutTarget` per-org routing. Read its
  §File Structure Requirements and §Tasks carefully — especially the customers/ directory layout
  and SpecLoader::load_all overlay walk semantics — before writing overlay wiring code in Task 7.
  The AC-009 DTU gap note in that story is the direct source for this story's scope.

- **S-DEMO-002 (merged PR #171):** Delivered E2E subprocess smoke test with all 4 sensors.
  The test harness pattern in S-DEMO-002 (single DTU instance per sensor) is the baseline
  that this story extends. Read S-DEMO-002's test fixture setup (`prism-bin/fixtures/e2e-demo/`)
  before designing the multi-instance harness to ensure the two patterns are consistent.

- **S-DEMO-ARMIS-AQL-001 + S-DEMO-CLAROTY-AUDIT-DTU-001 (both merged):** Both standalone
  DTU clones are now route-complete. The multi-instance binding in this story will exercise
  the full route surface of both clones (including `GET /api/v1/search` for Armis and
  `POST /api/v1/audit_log/get` for Claroty) via per-org overlays.

- **#[non_exhaustive] discipline (CLAUDE.md):** Every new public struct added to
  `prism-dtu-demo-server` or `prism-dtu-harness` needs `#[non_exhaustive]`. The
  compile-fail gate at `tests/external/non-exhaustive-violation/` enforces this; the
  `EXPECTED` count in `ci.yml` MUST be incremented by the number of new non-exhaustive types.

- **Multi-error aggregation pattern:** If multi-instance bind fails for one instance, collect
  all bind errors before returning (consistent with INV-ERR-003 pattern from
  S-CONFIG-MULTI-TENANT-OVERRIDE-001 BC-2.06.016). Do not short-circuit at the first error.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `BehavioralClone::start_on` signature MUST NOT change | AC-007 backward compat | Existing S-6.07–6.10 parity tests compile unchanged |
| Multi-instance API is ADDITIVE only | AC-007 backward compat | No removal of existing single-instance test helpers |
| New public structs require `#[non_exhaustive]` | CLAUDE.md + compile-fail gate | `ci.yml EXPECTED` bumped; compile-fail gate test updated |
| No `println!` in production code | CLAUDE.md | `rg 'println!' crates/prism-dtu-demo-server crates/prism-dtu-harness --type rust` must return 0 results |
| New `event_type` emissions require BC-2.16.002 catalog row | SAP-1 + PG-LP11-001 | Adversary SAP-1 sweep on every pass |
| reqwest::Client instances must use `.timeout(Duration::from_secs(30))` | CLAUDE.md conventions | Any new HTTP client in this story must have 30s timeout |
| `prism-dtu-harness` MUST NOT depend on `prism-spec-engine`, `prism-sensors`, or `prism-query` | Forbidden Dependencies | Build MUST fail if these deps appear |
| Overlay wiring helper operates on temp directories only; no production config mutation | Story scope | Unit tests use `tempdir()` crates |

### Forbidden Dependencies

`prism-dtu-demo-server` and `prism-dtu-harness` MUST NOT gain dependencies on:
- `prism-spec-engine` (the spec engine must not be imported by DTU crates — build MUST fail)
- `prism-sensors` (same perimeter rule)
- `prism-query` (same perimeter rule)

The overlay wiring helper (Task 7) uses `SensorInstanceOverlay` / `ResolvedSensorSpec` types.
If those types live in `prism-spec-engine`, the harness CANNOT import them directly — it must
either:
1. Re-express the overlay TOML file structure without importing spec-engine types, OR
2. Accept raw `base_url: String` values and produce raw TOML strings (the spec engine
   is then invoked by the test itself via `SpecLoader::load_all`, not by the harness).

Option 2 is the production-grade approach: the harness writes TOML overlay files to a temp
directory; the test drives `SpecLoader::load_all` on that temp directory; the spec engine
reads them. This preserves the perimeter boundary.

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `axum` | workspace version | Existing DTU router; unchanged for multi-instance |
| `tokio` | workspace version | Async multi-instance task spawning |
| `prism-dtu-common` | workspace path | `BehavioralClone` trait; NOT modified |
| `prism-dtu-armis` | workspace path | `ArmisClone` for multi-instance tests |
| `prism-dtu-claroty` | workspace path | `ClarotyClone` for multi-instance tests |
| `tempfile` | workspace version | Temp directory for overlay TOML files in tests |

Version source: workspace `Cargo.toml`. Do not pin independently. Do NOT add new crate
dependencies without verifying workspace version pin first.

---

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-dtu-demo-server/src/server.rs` | Add `MultiInstanceConfig` + `InstanceEntry` structs + multi-instance bind function |
| CREATE (optional) | `crates/prism-dtu-demo-server/src/multi_instance.rs` | Isolate multi-instance logic if server.rs becomes large |
| CREATE | `crates/prism-dtu-harness/src/multi_instance.rs` | `MultiInstanceHarness` struct + `start()` + `socket_map()` |
| CREATE | `crates/prism-dtu-harness/src/overlay_wiring.rs` | Helper: convert socket map to temp-dir overlay TOML files |
| MODIFY | `crates/prism-dtu-harness/src/lib.rs` | Export `MultiInstanceHarness` + `overlay_wiring` module |
| MODIFY | `tests/external/non-exhaustive-violation/src/lib.rs` | Assert new public structs are non_exhaustive; bump `EXPECTED` in `ci.yml` |
| MODIFY or CREATE | `crates/prism-dtu-demo-server/tests/multi_instance_tests.rs` | 15 Red Gate tests (per §Red Gate Test Plan) |
| MODIFY or CREATE | `crates/prism-dtu-harness/tests/multi_instance_harness_tests.rs` | Additional harness-specific Red Gate tests |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Requested bind address is already in use (EADDRINUSE) | Multi-instance startup returns `Err` with bind failure details; does not start partially-bound set silently |
| EC-002 | `MultiInstanceConfig` with zero instances | Returns empty `HashMap<String, SocketAddr>`; no error; no spawned tasks |
| EC-003 | Same sensor type, same org slug, two entries | Second entry overwrites first in the map (last-wins); implementer may choose to return an error instead — PO decides; document the chosen semantics |
| EC-004 | Overlay TOML file for org that is NOT in `OrgRegistry` | `SpecLoader::load_all` emits `E-SPEC-022` (S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-004); multi-instance harness test must register test orgs in OrgRegistry before calling load_all |
| EC-005 | `MultiInstanceHarness` dropped before test asserts isolation | Async drop races with request in flight; shutdown signal must drain in-flight requests; use `shutdown_timeout` pattern |
| EC-006 | DTU clone instance crashes mid-test | Test receives connection-refused error; not a silent cross-tenant leakage; test fails with clear error |
| EC-007 | org A overlay points to instance B socket (misconfiguration in test) | Requests go to instance B; leakage test fails (correct behavior — misconfiguration is detectable) |
| EC-008 | Per-org address map has 10+ entries (larger multi-tenant scenario) | All instances bind successfully; no hard cap; test time increases linearly; memory use increases proportionally |

---

## Gating / Open Questions at Dispatch

| OQ | Question | Owner | Resolution |
|----|----------|-------|-----------|
| OQ-1 | Which crate should own `MultiInstanceConfig`? Options: `prism-dtu-demo-server` (narrower) or `prism-dtu-common` (shared). If `prism-dtu-common`, both demo-server and harness can use it without a shared dep issue. | Architect | Must decide before dispatch; no default assumed |
| OQ-2 | Should `MultiInstanceHarness` use the `BehavioralClone` trait directly or take `Box<dyn BehavioralClone>`? The latter requires `prism-dtu-common` as a harness dep (currently unknown if it is). | Implementer + architect | Read Cargo.toml before dispatch; the answer drives the API design |
| OQ-3 | Has the PO authored the new multi-instance-binding BC and assigned canonical BC IDs? | Product Owner | Required before `status: ready` (Spec-First Gate S-7.01) |

---

## References

- S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-009 — DTU multi-tenant emulation gap (this story's origin)
- D-849 — state decision registering this story as a planned stub
- ADR-031 §D5 — DTU=true-DTU fidelity principle: parity tests should exercise real DTU paths
- ADR-029 — Multi-tenant sensor endpoint overlay mechanism (overlay base_url plumbing)
- BC-2.06.014 — Instance identity resolution at fanout (per-org base_url routing)
- S-DEMO-002 PR #171 — E2E multi-tenant isolation test (single-DTU baseline this story extends)

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-06-03 | story-writer | Initial materialization from [planned stub] per D-849 + AC-009 of S-CONFIG-MULTI-TENANT-OVERRIDE-001. Scope: prism-dtu-demo-server multi-address binding + prism-dtu-harness MultiInstanceHarness API + per-org overlay integration + 15 Red Gate tests. 9 ACs, 8 pts, P2. Status draft — BCs pending PO authorship per S-7.01 gate. OQ-1/OQ-2/OQ-3 flagged for dispatch-time resolution. |
