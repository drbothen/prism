---
document_type: story
story_id: S-DEMO-MULTI-TENANT-DTU-001
title: "prism-dtu-demo-server + prism-dtu-harness: Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing"
wave: 5
epic_id: E-DEMO
priority: P2
status: ready
version: "1.2"
level: "L4"
producer: story-writer
timestamp: "2026-06-03T00:00:00Z"
created: "2026-06-03"
modified: "2026-06-09T12:00:00Z"
tdd_mode: strict
subsystems: [SS-01]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns all prism-dtu-* crates including prism-dtu-demo-server
#   and prism-dtu-harness per ARCH-INDEX Subsystem Registry. DTU clones are test-infrastructure
#   siblings of the sensor adapters — both live under SS-01. Multi-address binding is a
#   DTU clone infrastructure concern: the demo server and harness each act as a host for
#   multiple in-process DTU clone instances, each bound to a distinct socket address.
#   This is the same SS-01 anchor used by sibling story S-DEMO-HARNESS-CLONE-PARITY-001
#   after its D-1068 F-P3-HIGH-001 POL-6 fix. SS-17 (WASM Plugin Runtime) does NOT own
#   this story — it is not a WASM plugin concern. Decision: D-1075.
crates_touched: [prism-dtu-demo-server, prism-dtu-harness]
target_module: prism-dtu-demo-server
behavioral_contracts: [BC-2.06.017]
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
  - "crates/prism-dtu-demo-server/src/lib.rs"
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - "crates/prism-dtu-demo-server/src/config.rs"
  - "crates/prism-dtu-harness/src/lib.rs"
  - "crates/prism-dtu-harness/src/clones/armis.rs"
  - "crates/prism-dtu-harness/src/clones/claroty.rs"
  - "crates/prism-dtu-harness/src/error.rs"
  - "crates/prism-dtu-common/src/lib.rs"
  - ".factory/stories/S-CONFIG-MULTI-TENANT-OVERRIDE-001-per-org-sensor-endpoint-overlay-loading.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.014-instance-identity-resolution-at-fanout.md"
input-hash: null
traces_to:
  - "S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-009"
  - "D-849"
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-MULTI-TENANT-DTU-001 v1.2 — Per-DTU-Instance Multi-Address Binding

**Story ID:** S-DEMO-MULTI-TENANT-DTU-001
**Status:** ready
**Version:** v1.2
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

2. `prism-dtu-harness` exposes a `MultiInstanceHarness` fixture builder that:
   - Starts N named DTU clone instances at ephemeral ports.
   - Returns a `socket_map: &HashMap<(String, String), SocketAddr>` keyed by plain
     `(org_slug, sensor_id)` strings (NOT newtypes — U-004 lightweight test-infra key)
     suitable for passing to `overlay_wiring::write_overlay_temp_dir`.
   - Integrates with the `ResolvedSensorSpec` map from S-CONFIG-MULTI-TENANT-OVERRIDE-001
     via the overlay wiring helper (not via direct spec-engine import).

3. At least one multi-tenant routing integration test (Red Gate level) drives:
   - org A → armis instance A (distinct address).
   - org B → armis instance B (distinct address).
   - Asserts instance A received ONLY org A's requests; instance B received ONLY org B's.
   - Zero cross-tenant URL leakage (AC-006-class paper-fix resistance).

4. All existing single-instance DTU parity tests continue to pass (backward compatibility).

---

## Behavioral Contracts

| BC | Title | Role |
|----|-------|------|
| BC-2.06.017 v1.1 | Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing | Anchoring BC — governs the multi-address bind semantics, the per-instance SocketAddr registry API, overlay TOML integration, INV-ISOLATION-001 no-cross-tenant-leakage invariant, INV-COMPAT-001 backward-compat invariant, INV-ERR-003-COMPAT multi-error aggregation, and duplicate-key error semantics (Postconditions 1–7). v1.1 amendments: corrected start_on signature + disambiguated error type names (DemoBindError vs BindError). |
| BC-2.06.014 | Instance Identity Resolution at Fanout | Referenced BC — governs (org_id, sensor_id) → ResolvedSensorSpec lookup and overlay base_url routing at the production fanout layer; this story's integration tests exercise that contract end-to-end against real DTU sockets. BC-2.06.014 is NOT anchored here per PO Flag 2 decision (D-1074): BC-2.06.017 is the test-infrastructure contract; BC-2.06.014 is the production behavior contract. See §BC Flag 2 Resolution. |

### BC Flag 2 Resolution

**Status: RESOLVED (D-1074)**

Flag 2 from the original story draft asked: "Does BC-2.06.014 need a new postcondition
for the 'multi-DTU routing verified end-to-end' scenario?"

**Decision: BC-2.06.014 is NOT amended.** BC-2.06.014 specifies WHAT must happen
(HTTP dispatch uses overlay `base_url`). BC-2.06.017 specifies the TEST APPARATUS that
PROVES it happens (real sockets, counted requests, INV-ISOLATION-001 assertion). Adding
end-to-end verification to BC-2.06.014 would conflate the production behavior contract
with the test infrastructure contract. BC-2.06.017 §Flag 2 Decision Notes carries the
full rationale.

---

## Acceptance Criteria

### AC-001: prism-dtu-demo-server accepts multi-instance bind configuration
`prism-dtu-demo-server` accepts a `MultiInstanceConfig` value (defined in the new file
`crates/prism-dtu-demo-server/src/multi_instance.rs`, re-exported from `lib.rs`) that
specifies N named instances of a sensor DTU type, each with an independent `bind` address.
On startup, each instance binds to its configured address (or an ephemeral port if address
is `127.0.0.1:0`) and the bound `SocketAddr` is exposed to callers via the returned
`HashMap<String, SocketAddr>`.
(traces to BC-2.06.017 postcondition 1 — multi-instance bind configuration accepted;
each `InstanceEntry` starts exactly one clone instance via `BehavioralClone::start_on`)

Red Gate test: `test_demo_server_multi_instance_bind_config_accepted`

### AC-002: Two armis instances start on distinct ephemeral ports
Given `MultiInstanceConfig { instances: [InstanceEntry { name: "armis-acme", bind: "127.0.0.1:0" }, InstanceEntry { name: "armis-contoso", bind: "127.0.0.1:0" }] }`,
when the multi-instance bind function runs, two separate `ArmisClone`
instances start on distinct ephemeral ports. The returned `Ok(HashMap<String, SocketAddr>)`
has two entries with distinct addresses (`map["armis-acme"] != map["armis-contoso"]`).
(traces to BC-2.06.017 postcondition 1 — OS assigns ephemeral ports; all N instances
returned in one map with no entries silently dropped)

Red Gate test: `test_demo_server_two_armis_instances_bind_distinct_ports`

### AC-003: Two claroty instances start on distinct ephemeral ports
Same as AC-002 but for `ClarotyClone`. Two separate Claroty instances start on distinct
ephemeral ports. Response to `POST /api/v1/alerts/` from each instance returns that
instance's fixture data independently.
(traces to BC-2.06.017 postcondition 1 — each instance addressable independently;
request to instance A's SocketAddr is served by instance A's clone)

Red Gate test: `test_demo_server_two_claroty_instances_bind_distinct_ports`

### AC-004: prism-dtu-harness MultiInstanceHarness builds per-org SocketAddr map
`prism-dtu-harness` exposes a `MultiInstanceHarness` (defined in the new file
`crates/prism-dtu-harness/src/multi_instance.rs`, re-exported from `lib.rs`) that:
- Accepts `Vec<HarnessEntry>` where `HarnessEntry { org_slug: String, sensor_id: String, clone: Box<dyn BehavioralClone> }`.
- Starts each clone via `entry.clone.start_on(bind_addr, Some(shutdown_tx.subscribe()), None).await?`
  (no-tls path; `&mut self` receiver requires iterating `entries.iter_mut()` — U-001).
- Returns `Ok(MultiInstanceHarness)` where `harness.socket_map()` returns
  `&HashMap<(String, String), SocketAddr>` — key is plain `(org_slug, sensor_id)` Strings,
  NOT `(OrgSlug, SensorId)` newtypes (U-004: lightweight test-infra key, distinct from
  production `OrgKey = (OrgId, DtuType)`).
This API is usable from integration tests without requiring `prism-dtu-demo-server`.
(traces to BC-2.06.017 v1.1 postcondition 2 — `start(entries).await` starts each entry;
`socket_map()` returns the per-(org, sensor) address map)

Red Gate test: `test_harness_multi_instance_builds_per_org_socket_map`

### AC-005: Per-org base_url overlay integrates with MultiInstanceHarness output
Given two `ArmisClone` instances at distinct sockets (from `MultiInstanceHarness`), and
per-org overlays constructed via `overlay_wiring::write_overlay_temp_dir(&harness, &tempdir)`:
```
customers/acme/armis.sensor.toml → base_url = "http://127.0.0.1:{acme_port}"
customers/contoso/armis.sensor.toml → base_url = "http://127.0.0.1:{contoso_port}"
```
when `SpecLoader::load_all` processes these overlays (S-CONFIG-MULTI-TENANT-OVERRIDE-001),
the `ResolvedSensorSpec` map entries for `(acme, armis)` and `(contoso, armis)` carry
distinct `base_url` values corresponding to the two instances. The overlay wiring helper
writes raw TOML strings only — it does NOT import `prism-spec-engine` types (INV-PERIMETER-001).
(traces to BC-2.06.017 postcondition 3 — after `write_overlay_temp_dir` + `SpecLoader::load_all`,
`ResolvedSensorSpec` for `(acme, armis)` has `base_url = "http://S_A"` and for `(contoso, armis)`
has `base_url = "http://S_B"` with S_A ≠ S_B)

Red Gate test: `test_multi_instance_overlay_loads_distinct_base_urls`

### AC-006: Org A's requests reach instance A exclusively; org B's reach instance B
Given the two-instance setup from AC-005, when `FanOutTarget` dispatches for `(acme, armis)`,
ALL requests go to the acme instance socket (S_A). When `FanOutTarget` dispatches for
`(contoso, armis)`, ALL requests go to the contoso instance socket (S_B). Zero requests
from org A reach org B's instance socket; zero requests from org B reach org A's instance
socket. Verification uses an in-clone request counter (`Arc<AtomicUsize>` injected at
start time or axum extension state) to count actual HTTP requests received by each clone's router.

This is the core multi-tenant routing isolation proof. It is the DTU-grounded equivalent of
S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-008 (paper-fix resistance) — proving that per-org
`base_url` is consumed by the live HTTP dispatch layer, not merely stored in the spec map.
(traces to BC-2.06.017 INV-ISOLATION-001 — requests_received_by_instance(S_A, query_for_org=O_B) = 0
and requests_received_by_instance(S_B, query_for_org=O_A) = 0 for any two distinct org slugs)

Red Gate test: `test_multi_tenant_routing_zero_cross_tenant_leakage`

### AC-007: Single-instance path unchanged (backward compatibility)
Existing single-instance callers of `ArmisClone::start_on(bind, ...)`,
`ClarotyClone::start_on(bind, ...)`, etc. are NOT modified and continue to work. All
existing parity tests in S-6.07–6.10 and S-DEMO-002 integration tests pass without
modification.

The real (unchanged) signature per BC-2.06.017 v1.1 Postcondition 5 and INV-COMPAT-001
(no-tls path, which this story targets):
```
async fn start_on(
    &mut self,
    bind: SocketAddr,
    shutdown: Option<broadcast::Receiver<()>>,
    tls: Option<()>,  // cfg(not(tls)) path
) -> anyhow::Result<SocketAddr>
```
This story does NOT modify `start_on` — the signature statement is here to confirm the
real signature for the implementer (U-001 correction; BC-2.06.017 v1.1 Amendment 1).

The `&mut self` receiver means `HarnessEntry.clone` must be held mutably in the bind
loop. Correct bind-loop call form (no-tls path):
```rust
let bound = entry.clone.start_on(bind_addr, Some(shutdown_tx.subscribe()), None).await?;
```
Iterate as `&mut entries` (index access or `iter_mut()`) — do NOT hold a shared `&self`
reference simultaneously with the `&mut self` call.

The multi-instance binding API is ADDITIVE — expressed through new structs and functions
that call `start_on` internally; `start_on` itself is never modified.
(traces to BC-2.06.017 v1.1 INV-COMPAT-001 — single-instance backward compatibility;
compile-time enforced by existing S-6.07–6.10 tests compiling unchanged)

Red Gate test: `test_single_instance_path_unaffected_by_multi_instance_addition`

### AC-008: Module-doc and pub API documentation updated
Both `prism-dtu-demo-server` and `prism-dtu-harness` module-doc (or `README.md` equivalents)
are updated to document the multi-instance binding API. No undocumented public types.
The `MultiInstanceHarness` type and `MultiInstanceConfig` type have doc comments on all public methods.
(traces to BC-2.06.017 INV-NONEXHAUSTIVE-001 — all new public types carry `#[non_exhaustive]`;
documentation completeness is a correlate of proper pub-API discipline)

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
| `test_harness_multi_instance_builds_per_org_socket_map` | AC-004 | prism-dtu-harness | `MultiInstanceHarness` returns `HashMap<(String,String),SocketAddr>` with correct `(org_slug, sensor_id)` string keys (U-004); `ArmisClone`/`ClarotyClone` from `[dev-dependencies]` |
| `test_harness_distinct_org_slots_different_sockets` | AC-004 | prism-dtu-harness | Two orgs for the same sensor type → two distinct `SocketAddr` values; key type is plain `(String, String)` |
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
| `MultiInstanceConfig` (new struct) | `crates/prism-dtu-demo-server/src/multi_instance.rs` (re-exported from `lib.rs`) | Pure (config struct) |
| `InstanceEntry` (new struct) | `crates/prism-dtu-demo-server/src/multi_instance.rs` (re-exported from `lib.rs`) | Pure (config struct) |
| Multi-instance bind logic | `crates/prism-dtu-demo-server/src/multi_instance.rs` | Effectful (binds sockets, spawns tokio tasks) |
| `MultiInstanceBindError` (new enum, demo-server) | `crates/prism-dtu-demo-server/src/multi_instance.rs` | N/A (error type; `#[non_exhaustive]`) |
| `DemoBindError` (new struct, demo-server) | `crates/prism-dtu-demo-server/src/multi_instance.rs` | N/A (error inner type; `#[non_exhaustive]`) |
| `MultiInstanceHarness` (new struct) | `crates/prism-dtu-harness/src/multi_instance.rs` (re-exported from `lib.rs`) | Effectful (binds sockets, wires overlay map) |
| `HarnessEntry` (new struct) | `crates/prism-dtu-harness/src/multi_instance.rs` (re-exported from `lib.rs`) | Pure (entry struct; holds `Box<dyn BehavioralClone>`) |
| `BindError` (new struct, harness) | `crates/prism-dtu-harness/src/error.rs` | N/A (error inner type; `#[non_exhaustive]`) |
| Overlay wiring helper | `crates/prism-dtu-harness/src/overlay_wiring.rs` (new file) | Pure (writes TOML strings to caller-supplied `&Path`; no fs side-effects in src/) |
| `HarnessError` extensions | `crates/prism-dtu-harness/src/error.rs` (existing; extend with `DuplicateKey` + `BindFailure` variants) | N/A (error type; ALREADY `#[non_exhaustive]` — new variants add 0 to compile-fail gate count) |
| BehavioralClone trait | `crates/prism-dtu-common/src/lib.rs` | Effectful (existing; NOT modified) |

**Critical file placement notes (D-1075):**
- `prism-dtu-demo-server` does NOT have a `server.rs`. It uses `lib.rs` + `harness.rs` + `config.rs`.
  `MultiInstanceConfig`, `InstanceEntry`, `MultiInstanceBindError`, and `DemoBindError` live in
  the NEW file `multi_instance.rs`, re-exported from `lib.rs`.
- `prism-dtu-common` does NOT gain any new types from this story (architect override D-1075:
  avoid coupling all downstream clone crates to orchestration types). `MultiInstanceConfig` and
  `InstanceEntry` are demo-server-local; `MultiInstanceHarness` and `HarnessEntry` are harness-local.
- `socket_map` key is `(String, String)` — plain `(org_slug, sensor_id)` strings, NOT `(OrgSlug, SensorId)` newtypes (U-004). This is an intentional lightweight test-infra key, distinct from the production `OrgKey = (OrgId, DtuType)`. Do not conflate; do not pull in the `OrgSlug`/`SensorId` newtype constructors for the `socket_map` key.
- `prism-dtu-common` is already a dep of `prism-dtu-harness`; `Box<dyn BehavioralClone>` requires no new Cargo.toml dep change.

**Locked API — D-1075 reconciliation (U-001..U-008, 2026-06-09):**

The following canonical signatures are architecture-locked. The implementer MUST use these
exact shapes; any deviation is a HIGH finding in adversarial review.

```rust
// ---- prism-dtu-demo-server/src/multi_instance.rs ----

#[non_exhaustive]
pub struct MultiInstanceConfig {
    pub instances: Vec<InstanceEntry>,
}

#[non_exhaustive]
pub struct InstanceEntry {
    pub name: String,     // e.g., "armis-acme"
    pub bind: SocketAddr, // typically 127.0.0.1:0 for ephemeral
}

// Returns Ok(HashMap<String, SocketAddr>) or Err(MultiInstanceBindError).
// Multi-error: all bind ops attempted before returning any error (INV-ERR-003-COMPAT).
// Duplicate name: returns Err(MultiInstanceBindError::DuplicateName { name }) before any binds.
pub async fn start_instances(
    cfg: MultiInstanceConfig,
    clone_factory: impl Fn(&InstanceEntry) -> Box<dyn BehavioralClone>,
) -> Result<HashMap<String, SocketAddr>, MultiInstanceBindError>;

#[non_exhaustive]
pub enum MultiInstanceBindError {
    DuplicateName { name: String },
    BindFailure(Vec<DemoBindError>),
}

#[non_exhaustive]
pub struct DemoBindError {
    pub instance_name: String,
    pub source: std::io::Error,
}

// ---- prism-dtu-harness/src/multi_instance.rs ----

#[non_exhaustive]
pub struct HarnessEntry {
    pub org_slug: String,
    pub sensor_id: String,
    pub clone: Box<dyn BehavioralClone>, // &mut self consumed at start_on call
}

// Field layout (D-1075 architect-locked):
//   socket_map: HashMap<(String, String), SocketAddr>  -- key is (org_slug, sensor_id)
//   shutdown_tx: broadcast::Sender<()>                 -- shared sender; per-instance .subscribe() at bind time
//   task_handles: Vec<JoinHandle<()>>                  -- one per started instance
//
// Key note: (org_slug: String, sensor_id: String) is a NEW lightweight test-infra key,
// intentionally DISTINCT from the production Harness OrgKey = (OrgId, DtuType). Do not
// conflate the two. The production OrgKey is not used here.
//
// Shutdown pattern: matches existing DemoHarness. impl Drop sends shutdown_tx.send(())
// then drops task_handles WITHOUT abort — axum with_graceful_shutdown handles the 5s
// drain on the clone side.
//
// Admin-token map is OMITTED from MultiInstanceHarness for this story — routing isolation
// is verified via request counts (Arc<AtomicUsize>), not configure calls.
#[non_exhaustive]
pub struct MultiInstanceHarness {
    socket_map: HashMap<(String, String), SocketAddr>,
    shutdown_tx: broadcast::Sender<()>,
    task_handles: Vec<JoinHandle<()>>,
}

impl MultiInstanceHarness {
    pub async fn start(entries: Vec<HarnessEntry>) -> Result<Self, HarnessError>;
    pub fn socket_map(&self) -> &HashMap<(String, String), SocketAddr>;
}

impl Drop for MultiInstanceHarness {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
        // task_handles dropped here; no explicit abort — graceful shutdown via axum
    }
}

// Bind-loop call form (no-tls path, &mut self receiver — U-001):
//   let bound = entry.clone.start_on(bind_addr, Some(shutdown_tx.subscribe()), None).await?;
// Iterate as: for entry in entries.iter_mut()  (NOT &entries)

// ---- prism-dtu-harness/src/overlay_wiring.rs ----

// Writes dir/customers/{org_slug}/{sensor_id}.sensor.toml with:
//   base_url = "http://{socket_addr}"
// `tempfile` is a TEST-ONLY [dev-dependency] — the caller owns the TempDir and
// passes dir.path(). No tempfile import in src/.
pub fn write_overlay_temp_dir(
    harness: &MultiInstanceHarness,
    dir: &std::path::Path,
) -> std::io::Result<()>;

// ---- prism-dtu-harness/src/error.rs additions ----
// Add to EXISTING HarnessError (already #[non_exhaustive] — new variants add 0 to compile-fail count):
//   DuplicateKey { org_slug: String, sensor_id: String }
//   BindFailure(Vec<BindError>)
//
// NEW #[non_exhaustive] struct (adds 1 E0639 to compile-fail gate):
#[non_exhaustive]
pub struct BindError {
    pub org_slug: String,
    pub sensor_id: String,
    pub source: std::io::Error,
}
```

**Clone reachability in tests (U-002):**
`prism-dtu-harness` `src/` code is self-contained: the multi-instance API takes
`Box<dyn BehavioralClone>` (trait in `prism-dtu-common`, already a runtime dep) and
NEVER names `ArmisClone` or `ClarotyClone` directly. `ArmisClone`/`ClarotyClone` are
only referenced in `prism-dtu-harness/tests/` via `[dev-dependencies]`. This does NOT
breach INV-PERIMETER-001 — dev-deps are test-only; the forbidden list is
`prism-spec-engine` / `prism-sensors` / `prism-query`; clone crates are not on it.

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (DTU clone infrastructure)
- `architecture/dependency-graph.md` §Wave-5 DTU fidelity stories

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec (v1.2 — larger due to locked API section) | ~8,500 |
| `crates/prism-dtu-demo-server/src/main.rs` (read) | ~2,000 |
| `crates/prism-dtu-demo-server/src/lib.rs` + `harness.rs` + `config.rs` (read) | ~4,000 |
| `crates/prism-dtu-harness/src/lib.rs` + `clones/` + `error.rs` (read) | ~6,000 |
| `crates/prism-dtu-common/src/lib.rs` — BehavioralClone trait (read) | ~2,000 |
| `tests/external/non-exhaustive-violation/` source (read — for U-006 wiring) | ~1,500 |
| `.github/workflows/ci.yml` (read — EXPECTED bump) | ~1,000 |
| S-CONFIG-MULTI-TENANT-OVERRIDE-001 story (reference) | ~4,000 |
| BC-2.06.017 v1.1 (anchoring BC — full read) | ~4,000 |
| BC-2.06.014 (reference for overlay base_url contract) | ~2,000 |
| BC files (2 BCs read at dispatch) | ~6,000 |
| Test file output (cargo nextest) | ~1,500 |
| **Total estimate** | **~42,500 tokens (~17% of 256K context)** |

Well within 20-30% context budget.

---

## Tasks

1. **Read source files first** — `crates/prism-dtu-demo-server/src/main.rs`,
   `crates/prism-dtu-demo-server/src/lib.rs`, `crates/prism-dtu-demo-server/src/harness.rs`,
   `crates/prism-dtu-demo-server/src/config.rs` (current startup API — NOTE: there is NO `server.rs`),
   `crates/prism-dtu-harness/src/lib.rs` + `clones/armis.rs` + `clones/claroty.rs` +
   `crates/prism-dtu-harness/src/error.rs` (`HarnessError` existing variants),
   `crates/prism-dtu-common/src/lib.rs` (`BehavioralClone` trait signature). Establish exact
   function signatures and startup patterns before writing any new code.

2. **Write Red Gate tests (stub phase — ALL must FAIL before implementation):**
   - All 15 tests listed in §Red Gate Test Plan.
   - Tests that require multi-instance binding must fail with `unimplemented!()` or
     compile error if the multi-instance API does not yet exist — that is the correct
     Red Gate failure mode.
   - Tests that verify single-instance backward compatibility must PASS in the pre-change
     state (these are regression guards, not forward-failing tests).

3. **Create `crates/prism-dtu-demo-server/src/multi_instance.rs`** — add `MultiInstanceConfig`
   + `InstanceEntry` structs + multi-instance bind function + `MultiInstanceBindError` enum
   + `DemoBindError` struct. Re-export from `lib.rs`. Use the canonical shapes from
   §Locked API (D-1075 reconciliation). All four types are `#[non_exhaustive]`.
   Verify `axum = "0.7"` and `tokio = { version = "1", features = ["full"] }` are the
   literal pins in `Cargo.toml` (U-008 — no workspace entry exists for these). Do NOT
   modify `BehavioralClone::start_on` signature.

4. **Extend `crates/prism-dtu-harness/src/error.rs`** with two changes:
   - (a) ADD new `#[non_exhaustive]` struct `BindError { pub org_slug: String, pub sensor_id: String, pub source: std::io::Error }`. This is a NEW type (adds 1 E0639 to compile-fail gate count).
   - (b) ADD two variants to the EXISTING `HarnessError` enum: `DuplicateKey { org_slug: String, sensor_id: String }` and `BindFailure(Vec<BindError>)`. `HarnessError` is ALREADY `#[non_exhaustive]` — new variants add 0 to the compile-fail count by themselves.
   Do NOT create a new top-level error enum. Do NOT replace `HarnessError`.

5. **Create `crates/prism-dtu-harness/src/multi_instance.rs`** — add `MultiInstanceHarness`
   + `HarnessEntry` structs. Re-export from `lib.rs`. Use the canonical field layout from
   §Locked API (D-1075 reconciliation) exactly:
   ```rust
   // Field layout (architect-locked — U-004):
   //   socket_map: HashMap<(String, String), SocketAddr>   -- (org_slug, sensor_id) as plain Strings
   //   shutdown_tx: broadcast::Sender<()>
   //   task_handles: Vec<JoinHandle<()>>
   //
   // Key: (org_slug: String, sensor_id: String) — lightweight test-infra key.
   // DISTINCT from production OrgKey = (OrgId, DtuType). Do not conflate.
   //
   // Bind-loop call form (no-tls, &mut self receiver — U-001):
   //   for entry in entries.iter_mut() {
   //       let bound = entry.clone.start_on(bind_addr, Some(shutdown_tx.subscribe()), None).await?;
   //   }
   //
   // impl Drop: send shutdown_tx.send(()); drop task_handles WITHOUT abort.
   // Matches existing DemoHarness drop pattern. Axum with_graceful_shutdown drains 5s.
   // Admin-token map: OMITTED for this story.
   ```
   New public types require `#[non_exhaustive]`. Update `EXPECTED` accordingly.

6. **Create `crates/prism-dtu-harness/src/overlay_wiring.rs`** — implement:
   ```rust
   pub fn write_overlay_temp_dir(
       harness: &MultiInstanceHarness,
       dir: &std::path::Path,
   ) -> std::io::Result<()>
   ```
   Writes `dir/customers/{org_slug}/{sensor_id}.sensor.toml` with `base_url = "http://{socket_addr}"`.
   Writes raw TOML strings only — NO import of `prism-spec-engine` types (INV-PERIMETER-001).
   NO `tempfile` import in `src/`; the caller (test code) owns the `TempDir` and passes `dir.path()`.
   `tempfile = "3"` (literal pin) is a `[dev-dependency]` of `prism-dtu-harness` only.
   The test that invokes `SpecLoader::load_all` calls `write_overlay_temp_dir` directly.

6a. **Wire non-exhaustive-violation compile-fail gate (U-006):** The
    `tests/external/non-exhaustive-violation/` crate currently does NOT import any
    `prism-dtu-*` crate. The implementer MUST:
    - (a) Add `prism-dtu-demo-server` and `prism-dtu-harness` as dependencies of the
      `non-exhaustive-violation` crate in its `Cargo.toml`.
    - (b) Add 6 struct-literal violation arms in `struct_violations.rs`:
      `MultiInstanceConfig`, `InstanceEntry`, `DemoBindError` (from demo-server);
      `MultiInstanceHarness`, `HarnessEntry`, `BindError` (from harness).
      Each arm must cause an E0639 compile error, proving `#[non_exhaustive]` is present.
    - (c) Add 1 match-arm violation in `enum_violations.rs` for `MultiInstanceBindError`
      (from demo-server), causing E0004.
    - (d) Bump `EXPECTED=49` → `EXPECTED=56` in `.github/workflows/ci.yml`.
    - Verify `just check` green after the bump (the gate counts must match exactly).

7. **Implement the multi-tenant routing isolation test** (`test_multi_tenant_routing_zero_cross_tenant_leakage`):
   - Start two `ArmisClone` instances via `MultiInstanceHarness`.
   - Construct per-org overlays via `overlay_wiring::write_overlay_temp_dir`.
   - Run `FanOutTarget` dispatches for both orgs.
   - Assert cross-tenant leakage count == 0 for each instance via `Arc<AtomicUsize>` request counters.
   - This test is the primary value of this story (AC-006 / INV-ISOLATION-001).

8. **SAP-1 sweep** — `rg 'event_type\s*=' crates/prism-dtu-demo-server crates/prism-dtu-harness --type rust` —
   any new `event_type = "..."` emissions must have BC-2.16.002 catalog rows.

9. **Pre-push gate** — `just check` GREEN workspace-wide. Verify `EXPECTED` count in
   `ci.yml` is incremented for all new `#[non_exhaustive]` public types. No `--no-verify`.

---

## Previous Story Intelligence

This is the first story in E-DEMO explicitly targeting `prism-dtu-demo-server` multi-instance
binding. Key lessons from predecessor stories:

- **S-CONFIG-MULTI-TENANT-OVERRIDE-001 (merged PR #155):** Delivered the overlay mechanism,
  `SensorInstanceOverlay`, `ResolvedSensorSpec`, and `FanOutTarget` per-org routing. Read its
  §File Structure Requirements and §Tasks carefully — especially the customers/ directory layout
  and SpecLoader::load_all overlay walk semantics — before writing overlay wiring code in Task 6.
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

- **prism-dtu-demo-server has no server.rs (D-1075):** The crate uses `lib.rs` + `harness.rs` +
  `config.rs`. Any story or previous code sketch referencing `server.rs` is incorrect.
  New logic goes in `multi_instance.rs` (new file) and re-exported from `lib.rs`.

- **Multi-error aggregation pattern:** If multi-instance bind fails for one instance, collect
  all bind errors before returning (consistent with INV-ERR-003 pattern from
  S-CONFIG-MULTI-TENANT-OVERRIDE-001 BC-2.06.016). Do not short-circuit at the first error.

- **Duplicate key is an error (D-1074):** `HarnessError::DuplicateKey` is returned immediately
  on duplicate `(org_slug, sensor_id)` entries — NOT silent last-wins. Same pattern for
  `MultiInstanceConfig`: duplicate `InstanceEntry::name` returns `MultiInstanceBindError::DuplicateName`.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `MultiInstanceConfig` + `InstanceEntry` live in `multi_instance.rs`, NOT `server.rs` | D-1075 (architect OQ-1 resolution) | Read source files first; there is no server.rs in prism-dtu-demo-server |
| `MultiInstanceHarness` + `HarnessEntry` live in `prism-dtu-harness/src/multi_instance.rs` | D-1075 (architect OQ-1 resolution) | File structure table |
| Nothing new added to `prism-dtu-common` | D-1075 (architect override) | Build fails if prism-dtu-common gains new orchestration types |
| `HarnessError` is EXTENDED, not replaced — extend the existing enum in `error.rs` | D-1075 (architect OQ-2 resolution) | Additive variants only: `DuplicateKey` + `BindFailure` |
| `BehavioralClone::start_on` signature MUST NOT change | BC-2.06.017 INV-COMPAT-001 | Existing S-6.07–6.10 parity tests compile unchanged |
| Multi-instance API is ADDITIVE only | BC-2.06.017 INV-COMPAT-001 | No removal of existing single-instance test helpers |
| New public structs require `#[non_exhaustive]` | CLAUDE.md + BC-2.06.017 INV-NONEXHAUSTIVE-001 | `ci.yml EXPECTED` bumped 49→56 (U-006); `non-exhaustive-violation` crate Cargo.toml gains `prism-dtu-demo-server` + `prism-dtu-harness` deps; 6 E0639 struct arms + 1 E0004 enum arm added |
| `non-exhaustive-violation` crate must import `prism-dtu-demo-server` + `prism-dtu-harness` | U-006 (D-1075 reconciliation) | These crates are NOT currently imported by the gate crate; implementer must add them and wire violation arms before declaring done |
| `axum = "0.7"` and `tokio = { version = "1", features = ["full"] }` literal per-crate pins | U-008 (D-1075 reconciliation) | No `[workspace.dependencies]` entry exists; do NOT write "workspace version"; check sibling crates for current pins |
| `tempfile = "3"` in `[dev-dependencies]` only; no `tempfile` import in `src/` | U-005 (D-1075 reconciliation) | Caller owns `TempDir`; passes `dir.path()` to `write_overlay_temp_dir` |
| `prism-dtu-armis` + `prism-dtu-claroty` in `[dev-dependencies]` only; never `[dependencies]` | U-002 (D-1075 reconciliation) | `src/` code never names clone types; only `tests/` uses them |
| No `println!` in production code | CLAUDE.md | `rg 'println!' crates/prism-dtu-demo-server crates/prism-dtu-harness --type rust` must return 0 results |
| New `event_type` emissions require BC-2.16.002 catalog row | SAP-1 + PG-LP11-001 | Adversary SAP-1 sweep on every pass |
| reqwest::Client instances must use `.timeout(Duration::from_secs(30))` | CLAUDE.md conventions | Any new HTTP client in this story must have 30s timeout |
| `prism-dtu-harness` MUST NOT depend on `prism-spec-engine`, `prism-sensors`, or `prism-query` | BC-2.06.017 INV-PERIMETER-001 | Build MUST fail if these deps appear in Cargo.toml |
| Overlay wiring helper operates on temp directories only; writes raw TOML strings | BC-2.06.017 INV-PERIMETER-001 | No `SensorInstanceOverlay` / `ResolvedSensorSpec` imports in harness |
| Duplicate key semantics: error immediately, NOT last-wins | BC-2.06.017 postcondition 7 + D-1074 | `HarnessError::DuplicateKey` returned before any clone instances started |

### Forbidden Dependencies

`prism-dtu-demo-server` and `prism-dtu-harness` MUST NOT gain dependencies on:
- `prism-spec-engine` (the spec engine must not be imported by DTU crates — build MUST fail)
- `prism-sensors` (same perimeter rule)
- `prism-query` (same perimeter rule)

The overlay wiring helper (Task 6) writes raw TOML strings. The test crate (which MAY
depend on `prism-spec-engine`) invokes `SpecLoader::load_all` directly — not via the harness.
This preserves the perimeter boundary (BC-2.06.017 INV-PERIMETER-001).

---

## Library & Framework Requirements

| Library | Version | Dep type | Purpose |
|---------|---------|----------|---------|
| `axum` | `"0.7"` (literal per-crate pin; no workspace entry) | `[dependencies]` | Existing DTU router; unchanged for multi-instance |
| `tokio` | `{ version = "1", features = ["full"] }` (literal per-crate pin; no workspace entry) | `[dependencies]` | Async multi-instance task spawning |
| `prism-dtu-common` | workspace path | `[dependencies]` | `BehavioralClone` trait; NOT modified; already a dep of prism-dtu-harness |
| `prism-core` | workspace path | `[dependencies]` | Already a dep; may be needed for other utilities. NOTE (U-004): `socket_map` key is plain `(String, String)`, NOT `(OrgSlug, SensorId)` newtypes — do not introduce newtype constructors for the key just because this dep exists. |
| `prism-dtu-armis` | workspace path | `[dev-dependencies]` (test-only) | `ArmisClone` used only in `prism-dtu-harness/tests/`; `src/` code never names it. Does NOT breach INV-PERIMETER-001 (dev-deps are test-only; forbidden list = prism-spec-engine/prism-sensors/prism-query; clone crates are not on it). |
| `prism-dtu-claroty` | workspace path | `[dev-dependencies]` (test-only) | `ClarotyClone` used only in `prism-dtu-harness/tests/`; same perimeter note as above. |
| `tempfile` | `"3"` (literal pin; matches siblings; no workspace pin exists) | `[dev-dependencies]` (test-only) | Caller owns `TempDir` and passes `dir.path()` to `write_overlay_temp_dir`; no `tempfile` import in `src/`. |

Version source for `axum`/`tokio`: use literal per-crate pins matching sibling crates
(U-008 — no `[workspace.dependencies]` entry exists for them). Do NOT write "workspace
version" for these two. For all other crates, verify workspace path/version before adding.

---

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| CREATE | `crates/prism-dtu-demo-server/src/multi_instance.rs` | `MultiInstanceConfig` + `InstanceEntry` structs + multi-instance bind function + `MultiInstanceBindError` enum + `DemoBindError` struct (all `#[non_exhaustive]`). NOT in server.rs (which does not exist in this crate). |
| MODIFY | `crates/prism-dtu-demo-server/src/lib.rs` | Re-export `multi_instance` module and its public types. |
| MODIFY | `crates/prism-dtu-demo-server/Cargo.toml` | Verify `axum = "0.7"` and `tokio = { version = "1", features = ["full"] }` pins match siblings (U-008). |
| CREATE | `crates/prism-dtu-harness/src/multi_instance.rs` | `MultiInstanceHarness` + `HarnessEntry` structs (both `#[non_exhaustive]`) + `start()` + `socket_map()` + `impl Drop`. |
| CREATE | `crates/prism-dtu-harness/src/overlay_wiring.rs` | `pub fn write_overlay_temp_dir(harness: &MultiInstanceHarness, dir: &std::path::Path) -> std::io::Result<()>` — writes `dir/customers/{org_slug}/{sensor_id}.sensor.toml` with `base_url = "http://{socket_addr}"`. Writes raw TOML strings only; no spec-engine import; no tempfile import in `src/`. |
| MODIFY | `crates/prism-dtu-harness/src/lib.rs` | Export `multi_instance` + `overlay_wiring` modules. |
| MODIFY | `crates/prism-dtu-harness/src/error.rs` | (a) Add `BindError` struct (`#[non_exhaustive]`; new: adds 1 E0639 to compile-fail count). (b) Extend EXISTING `HarnessError` enum with `DuplicateKey { org_slug: String, sensor_id: String }` + `BindFailure(Vec<BindError>)` variants (HarnessError is ALREADY `#[non_exhaustive]` — new variants add 0 to compile-fail count). |
| MODIFY | `crates/prism-dtu-harness/Cargo.toml` | Add `prism-dtu-armis` + `prism-dtu-claroty` as `[dev-dependencies]` (test-only); add `tempfile = "3"` as `[dev-dependency]`. Verify `axum`/`tokio` literal pins (U-008). |
| MODIFY | `tests/external/non-exhaustive-violation/Cargo.toml` | **ADD** `prism-dtu-demo-server` + `prism-dtu-harness` as dependencies of this crate (U-006: these crates are NOT currently imported; the implementer must add them). |
| MODIFY | `tests/external/non-exhaustive-violation/src/struct_violations.rs` | Add 6 struct-literal violation arms for: `MultiInstanceConfig`, `InstanceEntry`, `DemoBindError` (demo-server) + `MultiInstanceHarness`, `HarnessEntry`, `BindError` (harness). Each arm causes E0639 compile failure proving `#[non_exhaustive]` is present. |
| MODIFY | `tests/external/non-exhaustive-violation/src/enum_violations.rs` | Add 1 match-arm violation for `MultiInstanceBindError` (demo-server). Causes E0004 compile failure. Note: `HarnessError` match arm ALREADY EXISTS — do NOT add a duplicate; only check that existing arm still compiles (new variants are covered by existing wildcard or by adding new arms if the existing match is exhaustive). |
| MODIFY | `.github/workflows/ci.yml` | Bump `EXPECTED=49` → `EXPECTED=56` (7 new gate errors: 6 E0639 struct-literal arms + 1 E0004 match arm; detail: 5 structs from demo-server/harness + BindError struct = 6 structs = 6 E0639; MultiInstanceBindError enum = 1 E0004). |
| CREATE | `crates/prism-dtu-demo-server/tests/multi_instance_tests.rs` | Red Gate tests for demo-server multi-instance (per §Red Gate Test Plan). |
| CREATE | `crates/prism-dtu-harness/tests/multi_instance_harness_tests.rs` | Red Gate tests for harness multi-instance (per §Red Gate Test Plan). Uses `ArmisClone`/`ClarotyClone` from `[dev-dependencies]`. |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Requested bind address is already in use (EADDRINUSE) | Multi-error aggregation (INV-ERR-003-COMPAT): all bind operations attempted; all failures collected; successful partial binds shut down. Demo-server returns `Err(MultiInstanceBindError::BindFailure(Vec<DemoBindError>))` where each `DemoBindError { instance_name: String, source: std::io::Error }`. Harness returns `Err(HarnessError::BindFailure(Vec<BindError>))` where each `BindError { org_slug: String, sensor_id: String, source: std::io::Error }`. Does NOT short-circuit at first error. (BC-2.06.017 v1.1 EC-017-001 + Postcondition 6 + INV-ERR-003-COMPAT) |
| EC-002 | `MultiInstanceConfig` with zero instances | Returns `Ok(HashMap::new())` — empty map, no error, no spawned tasks (BC-2.06.017 EC-017-002) |
| EC-003 | Same `(org_slug, sensor_id)` pair appears in two `HarnessEntry` items | `Err(HarnessError::DuplicateKey { org_slug, sensor_id })` returned immediately before any clone instances started. Same instance `name` duplication in demo-server returns `Err(MultiInstanceBindError::DuplicateName { name })` before any bind attempts. Silent last-wins behavior is FORBIDDEN per BC-2.06.017 v1.1 postcondition 7 + D-1074. Error message must name the conflicting pair verbatim. |
| EC-004 | Overlay TOML file for org that is NOT in `OrgRegistry` | `SpecLoader::load_all` emits `E-SPEC-022` (S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-004 / BC-2.06.015); multi-instance harness test must register test orgs in OrgRegistry before calling load_all. |
| EC-005 | `MultiInstanceHarness` dropped before test asserts isolation | `impl Drop` for `MultiInstanceHarness` calls `self.shutdown_tx.send(())` (best-effort; error ignored) then drops `task_handles` WITHOUT explicit abort. Axum `with_graceful_shutdown` drains in-flight requests within 5s on the clone side (matching existing `DemoHarness::drop` pattern). Test code must ensure assertions happen before the harness goes out of scope. (BC-2.06.017 EC-017-005; U-004 architect-locked drop pattern) |
| EC-006 | DTU clone instance crashes mid-test | Test receives connection-refused error; not a silent cross-tenant leakage; test fails with clear error (BC-2.06.017 EC-017-006) |
| EC-007 | org A overlay points to instance B socket (misconfiguration in test) | Requests go to instance B; leakage test fails (correct behavior — misconfiguration is detectable; BC-2.06.017 EC-017-007) |
| EC-008 | Per-org address map has 10+ entries (larger multi-tenant scenario) | All instances bind successfully; no hard cap; test time increases linearly; memory use increases proportionally (BC-2.06.017 EC-017-008) |
| EC-009 | Two `InstanceEntry` items with the same `name` string | `Err(MultiInstanceBindError::DuplicateName { name: String })` returned before any bind attempt (BC-2.06.017 v1.1 EC-017-009 / postcondition 7). Inner field is `name: String` matching the `DuplicateName { name: String }` variant shape. |

---

## Gating / Open Questions at Dispatch

| OQ | Question | Owner | Resolution |
|----|----------|-------|-----------|
| OQ-1 | Which crate should own `MultiInstanceConfig`? Options: `prism-dtu-demo-server` (narrower) or `prism-dtu-common` (shared). | Architect | **RESOLVED (D-1075):** `MultiInstanceConfig` + `InstanceEntry` live in `prism-dtu-demo-server/src/multi_instance.rs`. NOT in `prism-dtu-common` — rationale: avoid coupling all downstream clone crates to orchestration types. No ADR — local crate-ownership choice per D-1075. |
| OQ-2 | Should `MultiInstanceHarness` use the `BehavioralClone` trait directly or take `Box<dyn BehavioralClone>`? Does it require a `prism-dtu-common` dep? | Implementer + architect | **RESOLVED (D-1075):** `HarnessEntry { org_slug: String, sensor_id: String, clone: Box<dyn BehavioralClone> }`. `prism-dtu-common` is already a dep of `prism-dtu-harness` — no Cargo.toml change needed. No ADR — local API design per D-1075. |
| OQ-3 | Has the PO authored the new multi-instance-binding BC and assigned canonical BC IDs? | Product Owner | **RESOLVED (D-1074 + BC-2.06.017):** BC-2.06.017 authored and anchored by PO on 2026-06-09. `behavioral_contracts: [BC-2.06.017]` is now populated. S-7.01 Spec-First Gate is cleared. |

---

## References

- S-CONFIG-MULTI-TENANT-OVERRIDE-001 AC-009 — DTU multi-tenant emulation gap (this story's origin)
- D-849 — state decision registering this story as a planned stub
- D-1074 — PO authorship of BC-2.06.017; EC-003 duplicate-key semantics decision (error, not last-wins)
- D-1075 — Architect adjudication: OQ-1 (`MultiInstanceConfig` in demo-server, not dtu-common), OQ-2 (`Box<dyn BehavioralClone>` API shape + extend HarnessError), file structure decisions
- ADR-031 §D5 — DTU=true-DTU fidelity principle: parity tests should exercise real DTU paths
- ADR-029 — Multi-tenant sensor endpoint overlay mechanism (overlay base_url plumbing)
- BC-2.06.017 — Per-DTU-Instance Multi-Address Binding (anchoring BC for this story)
- BC-2.06.014 — Instance identity resolution at fanout (per-org base_url routing; referenced, not anchored)
- S-DEMO-002 PR #171 — E2E multi-tenant isolation test (single-DTU baseline this story extends)

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-06-03 | story-writer | Initial materialization from [planned stub] per D-849 + AC-009 of S-CONFIG-MULTI-TENANT-OVERRIDE-001. Scope: prism-dtu-demo-server multi-address binding + prism-dtu-harness MultiInstanceHarness API + per-org overlay integration + 15 Red Gate tests. 9 ACs, 8 pts, P2. Status draft — BCs pending PO authorship per S-7.01 gate. OQ-1/OQ-2/OQ-3 flagged for dispatch-time resolution. |
| 1.1 | 2026-06-09 | story-writer | Finalized for S-7.01 gate clearance. Changes: (1) behavioral_contracts: [] → [BC-2.06.017] (D-1074 PO authorship); (2) subsystems: [SS-17] → [SS-01] (D-1075 architect POL-6 correction — SS-01 Sensor Adapters owns prism-dtu-* crates, not SS-17 WASM Plugin Runtime); (3) file placement corrected per D-1075 architect OQ-1 — MultiInstanceConfig/InstanceEntry in prism-dtu-demo-server/src/multi_instance.rs (NOT server.rs, NOT prism-dtu-common); (4) harness API corrected per D-1075 architect OQ-2 — HarnessEntry uses Box<dyn BehavioralClone>; canonical start/socket_map signatures applied; HarnessError extended with DuplicateKey + BindFailure (NOT new error type); (5) EC-003 updated — explicit HarnessError::DuplicateKey error instead of last-wins (D-1074 PO decision); (6) AC traces updated — BC-TBD placeholders replaced with canonical BC-2.06.017 postcondition/invariant names; (7) OQ-1/OQ-2/OQ-3 marked RESOLVED with D-1074/D-1075 citations; (8) §New-BC Flags section replaced with §BC Flag 2 Resolution (resolved); (9) status: draft → ready; inputs updated to reflect actual prism-dtu-demo-server files (lib.rs/harness.rs/config.rs — no server.rs). |
| 1.2 | 2026-06-09 | story-writer | D-1075 reconciliation — architect remove-uncertainty scan U-001..U-008 applied. BC reference updated to BC-2.06.017 v1.1 throughout. **U-001:** AC-007 and Task-5 corrected to real `start_on` signature (`&mut self`, `Option<broadcast::Receiver<()>>`, `Option<()>` tls); bind-loop call form `entry.clone.start_on(bind_addr, Some(shutdown_tx.subscribe()), None).await?` added verbatim; `iter_mut()` requirement stated. **U-002:** Library table: `prism-dtu-armis`/`prism-dtu-claroty` reclassified from `[dependencies]` to `[dev-dependencies]` (test-only); `src/` code never names clone types; INV-PERIMETER-001 non-breach note added. **U-003+U-007:** Error type disambiguation — `DemoBindError { instance_name, source }` (demo-server) vs `BindError { org_slug, sensor_id, source }` (harness); both new `#[non_exhaustive]` structs added to Architecture Mapping + File Structure + EC-001/EC-003/EC-009. `MultiInstanceBindError` enum added to Architecture Mapping. **U-004:** `MultiInstanceHarness` locked field layout (`socket_map: HashMap<(String,String),SocketAddr>`, `shutdown_tx: broadcast::Sender<()>`, `task_handles: Vec<JoinHandle<()>>`); key is plain `(String,String)` test-infra key (not production OrgKey); `impl Drop` graceful pattern; admin-token map omission noted; AC-004/Story-Level-Goal/Red-Gate-Plan updated; EC-005 aligned. **U-005:** `overlay_wiring` function signature locked (`&MultiInstanceHarness, &std::path::Path`) -> `std::io::Result<()>`; `tempfile` = `[dev-dependencies]` `"3"` literal pin; no tempfile in `src/`. **U-006:** EXPECTED 49→56 (7 new gate errors: 6 E0639 + 1 E0004); explicit Task-6a for non-exhaustive-violation crate wiring (Cargo.toml deps + struct_violations.rs arms + enum_violations.rs arm + ci.yml bump). **U-008:** `axum = "0.7"` and `tokio = { version = "1", features = ["full"] }` literal per-crate pins replacing "workspace version" guidance throughout Library table and Task-3. **Also:** §Locked API (D-1075 reconciliation) subsection added to Architecture Mapping with verbatim canonical Rust signatures for all new types. Token budget updated to ~42,500 (~17% of 256K). Status remains `ready`. |
