---
document_type: adr
adr_id: ADR-036
title: "Deterministic Scenario-Progression Engine — IncidentTimeline, Per-DTU Projection, and Cross-DTU Entity Coherence for Live Demo"
status: ACCEPTED
date: 2026-06-09
wave: 5
phase: 3.demo
version: "2.2"
authors: [architect]
related_decisions: [D-1077]
related_adrs: [ADR-009, ADR-011, ADR-002, ADR-031, ADR-028]
related_bcs_planned: [BC-2.06.018, BC-2.06.019, BC-2.06.020]
anchored_capabilities: [CAP-036, CAP-039]
subsystems_affected: [SS-01]
supersedes: null
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
inputs:
  - crates/prism-dtu-common/src/generator/archetype.rs
  - crates/prism-dtu-common/src/generator/rng.rs
  - crates/prism-dtu-common/src/generator/opts.rs
  - crates/prism-dtu-common/src/generator/fixture.rs
  - crates/prism-dtu-armis/src/state.rs
  - crates/prism-dtu-armis/src/clone.rs
  - crates/prism-dtu-armis/src/generator.rs
  - crates/prism-dtu-crowdstrike/src/state.rs
  - crates/prism-dtu-crowdstrike/src/clone.rs
  - crates/prism-dtu-crowdstrike/src/generator.rs
  - crates/prism-dtu-threatintel/src/state.rs
  - crates/prism-dtu-threatintel/src/clone.rs
  - crates/prism-dtu-nvd/src/state.rs
  - crates/prism-dtu-nvd/src/clone.rs
  - crates/prism-dtu-nvd/src/types.rs
  - crates/prism-dtu-demo-server/src/harness.rs
  - crates/prism-dtu-demo-server/src/config.rs
  - crates/prism-dtu-demo-server/Cargo.toml
  - crates/prism-dtu-armis/Cargo.toml
  - crates/prism-dtu-crowdstrike/Cargo.toml
  - crates/prism-dtu-threatintel/Cargo.toml
  - crates/prism-dtu-nvd/Cargo.toml
  - crates/prism-dtu-common/Cargo.toml
  - .factory/specs/behavioral-contracts/BC-2.06.018-dtu-demo-clone-data-seeding.md
  - .factory/objectives/multi-client-soc-demo-tasks.md
wiring_deferred_to: S-DEMO-DTU-LIVE-SCENARIO-001-A
---

# ADR-036 v2.2: Deterministic Scenario-Progression Engine — Substrate-Corrected Design

## Status

ACCEPTED 2026-06-09 (v1.0). Substrate-corrected 2026-06-09 (v2.0) per remove-uncertainty scan findings U-01 through U-09 (D-1077 follow-up). Symbol/path/syntax corrections 2026-06-09 (v2.1) per Story-A re-validation scan U-A-01, U-A-04, U-A-07. Archetype-forwarding reconciliation 2026-06-09 (v2.2) per local adversary pass 6 finding F-P6-HIGH-001 and user decision 2026-06-09: the canonical Story-A constructor is the 3-arg `new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId)` for all four generator-backed clones; the `fixture_set→Archetype` map drives the generator (NOT a hardcoded `CompromisedEndpoint`). Governs the live-scenario story split (S-DEMO-DTU-LIVE-SCENARIO-001-A baseline retrofit and S-DEMO-DTU-LIVE-SCENARIO-001-B scenario progression). Extends ADR-009 (multi-tenant deterministic generator) with a temporal scenario-progression layer.

---

## 1. Context

### 1.1 The Baseline (ADR-009 + BC-2.06.018)

ADR-009 established the multi-tenant deterministic generator for DTU behavioral
clones. It is a pure function: `generate(org_id, sensor, archetype, GenOpts) ->
FixtureSet`. The `CompromisedEndpoint` and `HighChurn` archetypes model a
compromised or churning environment, but they produce a **static snapshot** — all
records are generated at clone construction time and served unchanged for the
lifetime of the process. There is no notion of time passing.

BC-2.06.018 governs config-time seeding: `CloneConfig.seed` is forwarded from
`demo.toml` through `build_clone_pairs` to each clone constructor, causing per-client
data differentiation. This BC is the baseline that ADR-036 extends.

### 1.2 The Gap (D-1077)

The SOC-analyst live demo (multi-client-soc-demo-tasks.md) requires an incident that
*unfolds* over time. An analyst initiating a demo investigation should see: early
reconnaissance indicators in Armis and Claroty, then lateral movement detections in
CrowdStrike, then exfiltration IOCs resolving in ThreatIntel, then patched CVEs
resolving in NVD. This temporal narrative is the core of a believable SOC demo.

### 1.3 Substrate Reality (v2.0 Correction)

The v1.0 design contained a false substrate assumption that changes the implementation
scope substantially. The verified code reality is:

**The demo-server generator-backed clones (CrowdStrike, Armis, Claroty, Cyberint)
do NOT currently use their generators in their serving paths.** The facts:

- `CrowdstrikeClone::new()` (`crates/prism-dtu-crowdstrike/src/clone.rs:new()`)
  creates a `CrowdstrikeState` with empty `containment_store`, `detection_status_store`,
  and `session_registry`. The generator in `generator.rs` is never called. The routes
  serve directly from the state stores (stateful write targets), NOT from a FixtureSet.
  The CrowdStrike clone is a **stateful write-target clone**, not a fixture-serving clone.

- `ArmisClone::new()` (`crates/prism-dtu-armis/src/clone.rs`) loads `fixtures/devices.json`,
  `fixtures/device-activity.json`, and `fixtures/alerts.json` into `ArmisState` as
  immutable `Vec<DeviceRecord>` / `Vec<AlertRecord>`. The generator in `generator.rs`
  is never called from the serving path. `ArmisState.devices_ordered` is the static
  JSON fixture, not a generated FixtureSet.

- The generators ARE consumed — but only by `prism-dtu-harness` tests, NOT by
  the demo-server clones. The demo-server's `build_clone_pairs()` calls
  `CrowdstrikeClone::new()`, `ArmisClone::new()`, `CyberintClone::new()`, and
  `ClarotyClone::new()` — no `generate()` call anywhere in that path.

- `CloneConfig.seed` is declared in config.rs (`default = 42`) but is **never read**
  in `build_clone_pairs()`. BC-2.06.018's seeding postcondition is thus **unimplemented**
  — it is a real retrofit requirement, not a config-tweak.

- `DemoConfig` and `CloneConfig` have no `org_id` field. There is no `OrgId`
  construction in the demo-server today. This means `seeded_rng(seed, &OrgId)` has
  no OrgId source in the demo-server path.

**The v1.0 §2.3 "filter the pre-generated FixtureSet by stage mask" design has no
substrate in the target clones as they stand.** Obtaining seeded/distinct/scenario-evolving
data in the serving path requires **retrofitting each relevant clone** with a constructor
variant that calls `generate(...)` and stores the resulting records in state, while leaving
the existing `new()` static-JSON path unchanged for backward compatibility.

**Additional substrate corrections (U-03, U-05, U-06, U-07/U-08, U-09):**

- `seeded_rng(seed, org_id: &OrgId)` takes `&OrgId` (a `[u8;16]`-backed UUID), NOT `&str`.
- CrowdStrike `org_slug` is derived from org UUID bytes as 8 hex chars: `hex(org_id.as_bytes()[0..4])`. Armis generator takes `org_slug: &str` as explicit argument. There is no `"dev-acme-100-0"` format — the real format is `"dev-{8hex_from_uuid_bytes}-{seed}-{n}"` for CrowdStrike, `"dev-{injected_slug}-{seed}-{n}"` for Armis.
- NVD: `NvdClone::new()` returns `anyhow::Result<Self>` (fallible). `NvdState.cve_registry` is an immutable `HashMap<String, CveRecord>` (NOT `Mutex`-wrapped). There is no `lookup()` method — the method is `NvdState::lookup_and_count(&self, cve_id: &str) -> Option<CveRecord>`. `new_with_scenario` must return `anyhow::Result<Self>`. CVSS score path is `CveRecord.metrics.cvss_metric_v31: Option<Vec<CvssMetricV31>>` → first element `.cvss_data.base_score: f64` and `.base_severity: String`.
- ThreatIntel: `ThreatIntelClone::new()` is infallible. `ThreatIntelState.fixture_registry` is `Mutex<HashMap<String, FixtureKey>>` — it IS mutable at construction time. `new_with_scenario` can be infallible.
- Neither `prism-dtu-threatintel` nor `prism-dtu-nvd` have `chrono` as a dependency today. Neither has `fixture-gen` feature declared. Adding `ScenarioEntityCatalog` usage requires adding `fixture-gen = ["prism-dtu-common/fixture-gen"]` feature to both Cargo.toml files. `prism-dtu-common/fixture-gen` transitively enables `prism-core`. This is permitted: `INV-PERIMETER-001` prohibits `prism-dtu-*` from depending on `prism-spec-engine`, `prism-sensors`, or `prism-query` — NOT on `prism-core`. Both `prism-dtu-armis` and `prism-dtu-crowdstrike` already depend on `prism-core` directly. `ScenarioEntityCatalog` carries no `prism-spec-engine` dependency. Perimeter holds.
- `demo.toml` `stage_duration_secs` array has **4 entries**, corresponding to the cumulative `activates_after_secs` thresholds for stages 1-4. Stage 0 (Baseline) always activates at elapsed=0 and needs no entry. `stage_duration_secs = [60, 180, 360, 600]` means: stage 1 activates at >=60s, stage 2 at >=180s, stage 3 at >=360s, stage 4 at >=600s. The 5-stage timeline is thus specified by a 4-element array.

### 1.4 Constraints from the Existing Architecture

The generator (ADR-009) is a pure function behind `feature = "fixture-gen"` in
`prism-dtu-common`. `INV-PERIMETER-001` (enforced by compile-fail gate in
`tests/external/perimeter-violation/`) prohibits `prism-dtu-*` crates from depending
on `prism-spec-engine`, `prism-sensors`, or `prism-query`. Dependency on `prism-core`
is permitted and already present in Armis, CrowdStrike, and demo-server.

The demo-server harness (`DemoHarness`, `build_clone_pairs`) constructs all clones
sequentially and holds them in a `Vec<ClonePair>`. There is no existing mechanism to
share state across clone instances at construction time (this must be added).

---

## 2. Decision

### 2.1 Time/Clock Model: Pure Function of Wall-Clock Elapsed Seconds

**Decision:** (unchanged from v1.0) The progression is computed as a pure function of
`(seed, scenario_start_epoch_secs, now_secs)` on every inbound HTTP request. There is
no background mutator task. There are no mutable stage counters. The route handler calls
`current_stage_index(scenario_start, Utc::now(), &stages) -> usize` and projects the
full scene for that stage.

```text
elapsed_secs = now_unix_secs - scenario_start_unix_secs
stage_index  = max stage S such that elapsed_secs >= stages[S].activates_after_secs
```

**`scenario_start` source.** `CloneConfig` gains `scenario_start_secs: Option<i64>`
(unix epoch seconds). When `None`, the scenario starts at construction time (stage 0
immediately). The operator sets an explicit `scenario_start_secs` in `demo.toml` to
synchronize all DTUs to a shared timeline.

**Stage duration representation.** `CloneConfig` gains
`stage_duration_secs: Vec<u64>` with 4 entries for the 5-stage default timeline.
Entry `[i]` is the cumulative `activates_after_secs` for stage `i+1`. Stage 0
(Baseline) always activates at 0 and needs no entry. When empty, archetype defaults
are used.

**`demo.toml` canonical example:**
```toml
[clones.crowdstrike.scenario]
enabled = true
archetype = "compromised_endpoint"
scenario_start_secs = 1749456000
stage_duration_secs = [60, 180, 360, 600]  # 4 entries for stages 1-4 activation thresholds
```

### 2.2 The `IncidentTimeline` and `ScenarioEntityCatalog`: Shared Abstractions in `prism-dtu-common`

A new module `crates/prism-dtu-common/src/scenario/` (behind `feature = "fixture-gen"`)
houses two types:

#### `ScenarioEntityCatalog`

The catalog is the **shared entity namespace** that all DTU projections draw from. It
is a pure-data struct computed from `(seed, org_id)` at construction time. It contains:

```rust
/// Shared entity catalog for one client's incident scenario.
/// Produced once at harness construction time from (seed, org_id).
/// All DTU projections for this client derive their entity IDs from this catalog.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct ScenarioEntityCatalog {
    /// The primary compromised device ID (used by Armis, CrowdStrike, Claroty, Cyberint).
    /// Format for CrowdStrike: "dev-{8hex}-{seed}-0"
    /// where 8hex = hex(org_id.as_bytes()[0..4]).
    /// Format for Armis: "dev-{org_slug}-{seed}-0"
    /// where org_slug is injected as an explicit arg to the Armis generator.
    /// The harness derives org_slug from org_id bytes consistently.
    pub primary_device_id_cs: String,    // CrowdStrike format
    pub primary_device_id_armis: String, // Armis format (same derivation, explicit slug arg)
    /// Hostname for the compromised device (consistent across DTUs).
    pub primary_hostname: String,
    /// Secondary device IDs involved in lateral movement.
    pub lateral_device_ids_cs: Vec<String>,
    pub lateral_device_ids_armis: Vec<String>,
    /// IOC IPv4 addresses introduced during Exfil stage.
    /// These MUST resolve as malicious in ThreatIntel.
    pub ioc_ips: Vec<String>,
    /// IOC domain names introduced during Exfil stage.
    pub ioc_domains: Vec<String>,
    /// IOC SHA256 file hashes introduced during LateralMovement stage.
    pub ioc_hashes: Vec<String>,
    /// CVE IDs assigned to the primary device.
    /// These MUST resolve in NVD (cvss_metric_v31[0].cvss_data.base_score >= 7.0).
    pub device_cves: Vec<String>,
    /// Canonical org_slug derived from org_id bytes (hex of first 4 bytes).
    /// Used by both CrowdStrike and Armis generators for consistent ID derivation.
    pub org_slug: String,
}
```

**Canonical org_slug derivation (authoritative, fixes U-03/U-06):**

```rust
/// Derive the canonical org_slug from OrgId bytes.
/// Formula: hex(org_id.as_bytes()[0..4]) — 8 hex characters.
/// This matches CrowdStrike generator's internal `org_slug()` function exactly.
/// Armis generator receives this value as the `org_slug: &str` argument.
pub fn org_slug_from_org_id(org_id: &OrgId) -> String {
    let b = org_id.as_bytes();
    format!("{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3])
}
```

**Device ID format (authoritative, fixes U-03):**

- CrowdStrike: `"dev-{org_slug}-{seed}-{n}"` where `org_slug = org_slug_from_org_id(org_id)`.
  Example: org_id whose bytes start `[0xde, 0xad, 0xbe, 0xef, ...]` → slug `"deadbeef"` →
  `primary_device_id_cs = "dev-deadbeef-42-0"` (seed=42).
- Armis: `"dev-{org_slug}-{seed}-{n}"` — same formula, same slug, same result.
  Armis generator receives `org_slug` as `&str` argument; the catalog passes `&catalog.org_slug`.

**Catalog derivation from `(seed, org_id)`:**

- IOC IPs, domains, hashes, and CVE IDs are derived from `gen_seeded_rng(seed.wrapping_add(1), &org_id)`
  (secondary RNG stream, independent of primary generator stream; `gen_seeded_rng` is the
  two-arg re-export in `prism-dtu-common::lib` — distinct from the one-arg legacy `seeded_rng`).
- CVE IDs formatted as `"CVE-{year}-{n}"` where year and n are derived from the secondary RNG.

#### `IncidentTimeline`

```rust
/// Timeline for a staged incident scenario.
/// Carries the entity catalog plus stage definitions.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct IncidentTimeline {
    pub entities: ScenarioEntityCatalog,
    pub stages: Vec<IncidentStage>,
    pub scenario_start_epoch_secs: i64,
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct IncidentStage {
    pub name: &'static str,
    /// Cumulative elapsed seconds from scenario_start before this stage activates.
    /// Stage 0 (Baseline) always has activates_after_secs = 0.
    pub activates_after_secs: u64,
    pub visible_entity_mask: StageMask,
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct StageMask {
    pub primary_device: bool,
    pub lateral_devices: bool,
    pub ioc_ips: bool,
    pub ioc_domains: bool,
    pub ioc_hashes: bool,
    pub device_cves: bool,
}
```

**Default `CompromisedEndpoint` stage definitions (5 stages, 4-entry `stage_duration_secs`):**

| Stage | Name | activates_after_secs | stage_duration_secs index | Visible |
|-------|------|----------------------|--------------------------|---------|
| 0 | `Baseline` | 0s | (none — always first) | primary device only; no alerts |
| 1 | `Recon` | 60s | [0] | primary device + low-severity alerts |
| 2 | `LateralMovement` | 180s | [1] | primary + lateral devices; IOC hashes emerge |
| 3 | `Exfil` | 360s | [2] | all devices; IOC IPs + domains emerge |
| 4 | `Containment` | 600s | [3] | primary device contained; all IOCs; CVEs visible in NVD |

`stage_duration_secs = [60, 180, 360, 600]` — 4 entries, one per non-zero stage activation
threshold. `stages[0].activates_after_secs = 0` always; it is not represented in the array.

**`current_stage_index` (pure function):**

```rust
pub fn current_stage_index(timeline: &IncidentTimeline, now_epoch_secs: i64) -> usize {
    let elapsed = (now_epoch_secs - timeline.scenario_start_epoch_secs).max(0) as u64;
    let mut stage = 0;
    for (i, s) in timeline.stages.iter().enumerate() {
        if elapsed >= s.activates_after_secs {
            stage = i;
        }
    }
    stage
}
```

### 2.3 Per-DTU Projection — The Retrofit Design (corrects v1.0 §2.3)

**The critical design correction from v1.0:** Because the demo-server clones currently
have NO `FixtureSet` in their serving path (U-01), the stage-mask approach requires a
two-phase retrofit:

**Phase A (Story A — BC-2.06.018 baseline retrofit):** Wire the generators into the
demo-server clone serving path. Each generator-backed clone gains a `new_with_seed`
constructor that:
1. Calls `generate(org_id, [org_slug,] archetype, &GenOpts { seed, ..GenOpts::default() })` to produce
   a `FixtureSet`. The `archetype` argument is the `Archetype` variant mapped from
   `CloneConfig.fixture_set` by `build_clone_pairs` via INV-FIXTURE-SET-ARCHETYPE-MAP-001.
   It is NEVER hardcoded — any of the 8 archetypes may be requested.
2. Stores the generated records in a new field `generated_records: Vec<serde_json::Value>`
   in the state struct (alongside the existing static-JSON fixture fields).
3. Route handlers serve from `generated_records` when this field is populated; fall
   back to `devices_ordered` / static fixtures when empty.
4. `new()` constructors are unchanged (static JSON path, backward-compatible).

This is the "baseline seeding" requirement of BC-2.06.018 — per-client distinct data
from `CloneConfig.seed` — and it is entirely absent today. Without it, BC-2.06.018's
postconditions 1-3 are unimplemented.

**Phase B (Story B — BC-2.06.019 + BC-2.06.020 scenario progression):** Add the
`IncidentTimeline` layer on top of the Phase A substrate. Each generator-backed clone's
state gains `timeline: Option<Arc<IncidentTimeline>>`. Route handlers that already serve
from `generated_records` add the stage-mask filter. New constructors
`new_with_scenario(seed, archetype, org_id, timeline)` combine Phase A generation with
Phase B timeline attachment.

#### Generator-backed clones: per-clone retrofit scope

**CrowdStrike** (`crates/prism-dtu-crowdstrike/src/`):
- `CrowdstrikeState` gains `generated_devices: Vec<serde_json::Value>` and
  `generated_detections: Vec<serde_json::Value>` (from FixtureSet, filtered by
  `_record_type`).
- New constructor `CrowdstrikeClone::new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId)` calls
  `generate(org_id, archetype, GenOpts { seed, ..GenOpts::default() })` under
  `#[cfg(feature = "fixture-gen")]`. The `archetype` parameter is supplied by `build_clone_pairs`
  via the `fixture_set→Archetype` mapping (INV-FIXTURE-SET-ARCHETYPE-MAP-001); it is NOT hardcoded
  to `Archetype::CompromisedEndpoint`.
- Route `routes/hosts.rs` and `routes/detections.rs` serve `generated_devices` /
  `generated_detections` when non-empty; fall back to the static device-read path
  (`load_host_ids()` / `load_host_details()` in `routes/hosts.rs`) when empty.
  (The `containment_store` / `detection_status_store` are write-target overlays for
  containment/detection mutations — they are NOT the device-read fallback.)
- `fixture-gen` feature already declared in `Cargo.toml` (`fixture-gen = ["prism-dtu-common/fixture-gen"]`).
- `chrono` already a direct dependency.
- Org_slug for ID generation: `org_slug_from_org_id(&org_id)` (8 hex chars from UUID bytes).

**Armis** (`crates/prism-dtu-armis/src/`):
- `ArmisState` gains `generated_records: Vec<serde_json::Value>` alongside the
  existing `devices_ordered: Vec<DeviceRecord>` and `alert_fixture: Vec<AlertRecord>`.
- New constructor `ArmisClone::new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId)`.
  Internally derives `org_slug = org_slug_from_org_id(&org_id)` (8-hex formula, same as
  `ScenarioEntityCatalog`). Under `#[cfg(feature = "fixture-gen")]`, calls
  `generate(org_id, &org_slug, archetype, &GenOpts { seed, ..GenOpts::default() })`.
  The `archetype` parameter is supplied by `build_clone_pairs` via INV-FIXTURE-SET-ARCHETYPE-MAP-001;
  it is NOT hardcoded to `Archetype::CompromisedEndpoint`.
  Stores FixtureSet records in `generated_records`.
- Route `routes/devices.rs` `paginate_devices` serves from `generated_records` when
  non-empty, deserialized as `DeviceRecord`; falls back to `devices_ordered`.
- `fixture-gen` feature already declared in `Cargo.toml`.
- `chrono` already a direct dependency.

**Claroty** (`crates/prism-dtu-claroty/src/`): same 3-arg pattern —
`ClarotyClone::new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId)`.
Internally calls `generate(&org_id, archetype, &GenOpts { seed, ..GenOpts::default() })`
(Claroty generator signature: `generate(org_id: &OrgId, archetype: Archetype, opts: &GenOpts)`).
The `archetype` is supplied by `build_clone_pairs` via INV-FIXTURE-SET-ARCHETYPE-MAP-001;
NOT hardcoded. Generated records stored in state; route handler fallback logic as per
CrowdStrike/Armis pattern.

**Cyberint** (`crates/prism-dtu-cyberint/src/`): same 3-arg pattern —
`CyberintClone::new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId) -> anyhow::Result<Self>`.
Internally calls `generate(&org_id, archetype, &GenOpts { seed, ..GenOpts::default() })`
(Cyberint generator signature: `generate(org_id: &OrgId, archetype: Archetype, opts: &GenOpts)`).
The `archetype` is supplied by `build_clone_pairs` via INV-FIXTURE-SET-ARCHETYPE-MAP-001;
NOT hardcoded. Cyberint constructor is currently fallible (`CyberintClone::new() -> anyhow::Result<Self>`);
`new_with_seed` is also fallible for the same reason.

#### Enrichment clones (ThreatIntel, NVD)

**Decision: static lookup injection at construction time** (unchanged from v1.0 §2.3).
These are pure key-value lookup stores; no generator needed.

**ThreatIntel** (`crates/prism-dtu-threatintel/src/`):
- New constructor: `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> Self`.
  (Infallible — mirrors `ThreatIntelClone::new()` which is infallible.)
- Implementation: calls `with_admin_token`, then pre-populates `fixture_registry`
  (which is `Mutex<HashMap<String, FixtureKey>>`) by inserting all `entities.ioc_ips`,
  `entities.ioc_domains`, and `entities.ioc_hashes` with `FixtureKey::Malicious`.
- Feature requirement: add `fixture-gen = ["prism-dtu-common/fixture-gen"]` to
  `crates/prism-dtu-threatintel/Cargo.toml`. (No `chrono` needed for this constructor.)
- INV-PERIMETER-001: `ScenarioEntityCatalog` is in `prism-dtu-common`. No
  `prism-spec-engine`/`prism-sensors`/`prism-query` dependency introduced.

**NVD** (`crates/prism-dtu-nvd/src/`):
- New constructor: `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> anyhow::Result<Self>`.
  (Fallible — mirrors `NvdClone::new()` return type.)
- `NvdState.cve_registry` is an **immutable** `HashMap<String, CveRecord>` built at
  construction time and never mutated after. Injection happens at construction by
  including `entities.device_cves` entries in the initial `HashMap`.
- `NvdClone::new_with_scenario` builds the registry by: loading base fixtures from
  `fixtures/cves.json` (same as `new()`), then inserting synthetic `CveRecord` entries
  for each CVE ID in `entities.device_cves` with:
  - `metrics.cvss_metric_v31 = Some(vec![CvssMetricV31 { cvss_data: CvssData { base_score: 8.1, base_severity: "HIGH".to_string(), .. }, .. }])`
  - (Base path: `CveRecord.metrics: CveMetrics` → `.cvss_metric_v31: Option<Vec<CvssMetricV31>>` → `[0].cvss_data: CvssData` → `.base_score: f64`, `.base_severity: String`)
- Feature requirement: add `fixture-gen = ["prism-dtu-common/fixture-gen"]` to
  `crates/prism-dtu-nvd/Cargo.toml`. (No `chrono` needed for this constructor — CVE
  timestamp fields are static strings.)
- INV-PERIMETER-001: as with ThreatIntel, only `prism-dtu-common` is added. Perimeter holds.

#### Stage-gate enrichment

ThreatIntel IOCs and NVD CVEs resolve as soon as the clone starts. The analyst discovers
IOCs/CVEs at the appropriate operational DTU stage, then pivots to enrichment. Enrichment
DTUs are always "ready" — stage progression is driven by the operational DTUs.

### 2.4 Cross-Clone Coordination: `ScenarioConfig` in `demo.toml`

The shared `ScenarioEntityCatalog` is constructed once in `build_clone_pairs` and
distributed to each clone constructor. This is the coordination point.

**`CloneConfig` extension (adds `org_id`, `scenario_start_secs`, `stage_duration_secs`):**

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ScenarioConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_scenario_archetype")]
    pub archetype: String,  // "compromised_endpoint" (only supported value in v1)
    #[serde(default)]
    pub scenario_start_secs: Option<i64>,
    /// 4-entry array: activates_after_secs thresholds for stages 1..=4.
    /// Empty = use archetype defaults ([60, 180, 360, 600]).
    #[serde(default)]
    pub stage_duration_secs: Vec<u64>,
}
```

**`DemoConfig`/`CloneConfig` must also gain `org_id`** (to construct `OrgId` for
`seeded_rng` and catalog derivation). This is a new required field for scenario mode:

```rust
// In CloneConfig (or in a new top-level [scenario] section)
/// Org UUID (hyphenated) for this demo client.
/// Required when scenario.enabled = true for any clone.
/// Used to construct prism_core::OrgId for seeded_rng and catalog derivation.
pub org_id: Option<String>,  // parsed as uuid::Uuid → OrgId
```

**`build_clone_pairs` coordination logic (complete, corrects v1.0 §2.4):**

1. If any clone has `scenario.enabled = true`: read `config.clones.*.org_id` (required
   when scenario enabled; return `E-DEMO-004` if absent).
2. Parse `org_id` string as `uuid::Uuid` → `OrgId`. All scenario-enabled clones MUST
   share the same `seed` — if they differ, return `E-DEMO-002`.
3. Derive `org_slug = org_slug_from_org_id(&org_id)` (8 hex chars from UUID bytes).
4. Build `ScenarioEntityCatalog` from `(seed, org_id, org_slug)`.
5. Build `IncidentTimeline` from catalog + stage definitions.
6. Construct generator-backed clones with `new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline))`.
7. Construct ThreatIntel with `ThreatIntelClone::new_with_scenario(&catalog)`.
8. Construct NVD with `NvdClone::new_with_scenario(&catalog)?`.
9. When `CloneConfig.fixture_set != "default"` (any non-default archetype) and
   `scenario.enabled = false` (or absent): call `new_with_seed(seed, archetype, org_id)`
   where `archetype` is the INV-FIXTURE-SET-ARCHETYPE-MAP-001 mapping of `fixture_set`.
   This is the Story A baseline seeding path. The `org_id` is from `CloneConfig.org_id`
   (parsed to `OrgId`; absence returns E-DEMO-004 for the generator-backed clone).
   When `CloneConfig.fixture_set = "default"` AND no `org_id` is provided: construct
   with existing `new()` / `new_with_access_token()` path (backward-compatible with
   BC-2.06.018 postcondition 4).

### 2.5 Backward Compatibility and Perimeter

**Backward-compatible path.** All existing constructors (`CrowdstrikeClone::new()`,
`ArmisClone::new()`, etc.) are unchanged. Scenario progression is opt-in via the
`[clones.*.scenario]` block. When absent, behavior is byte-identical to pre-ADR-036
behavior (BC-2.06.018 postcondition 4).

**VP-018-B backward-compat case (canonical).** The 3-arg `new_with_seed` preserves full
backward compatibility because `"default"` maps to `Archetype::HealthyOtEnvironment` via
INV-FIXTURE-SET-ARCHETYPE-MAP-001. The BC-2.06.018 Postcondition 4 backward-compat call is:

```text
new_with_seed(42, Archetype::HealthyOtEnvironment, <default_org_id>)
```

where `<default_org_id>` is a well-known test UUID (implementer to define once and use
consistently across all backward-compat regression tests, RG-A-005). This triple must
produce data semantically equivalent to the pre-seeding `new()` behavior. The
`HealthyOtEnvironment` archetype MUST be the generator's natural default behavior — if the
pre-seeding `new()` used a different internal archetype baseline, the implementer MUST either
align `HealthyOtEnvironment` output to match it or update the regression tests accordingly
(BC-2.06.018 Postcondition 4 note).

**INV-PERIMETER-001.** `ScenarioEntityCatalog` and `IncidentTimeline` live in
`prism-dtu-common` (behind `feature = "fixture-gen"`). They carry no dependency on
`prism-spec-engine`, `prism-sensors`, or `prism-query`. The compile-fail gate in
`tests/external/perimeter-violation/` continues to hold.

**Non-exhaustive gate (ci.yml EXPECTED=49).** Adding new constructors (`new_with_seed`,
`new_with_scenario`) to public types does NOT change the EXPECTED count — EXPECTED
counts `#[non_exhaustive]` types and struct/enum violation rows, not constructors. The
`ScenarioEntityCatalog` and `IncidentTimeline` types ARE `#[non_exhaustive]`, so they
each need a violation row and the EXPECTED count MUST increase (2 new public
`#[non_exhaustive]` types in `prism-dtu-common` → EXPECTED increases from 49 to 51,
pending exact count by Story A/B implementer). The implementer MUST update both the
`tests/external/non-exhaustive-violation/` crate and `ci.yml EXPECTED=` atomically.

**`fixture-gen` feature propagation.** Adding `fixture-gen = ["prism-dtu-common/fixture-gen"]`
to `prism-dtu-threatintel/Cargo.toml` and `prism-dtu-nvd/Cargo.toml` is safe. The
`ScenarioEntityCatalog` type is gated `#[cfg(feature = "fixture-gen")]`; production
binaries that do not activate this feature are unaffected.

---

## 3. Rationale

### 3.1 Pure-function-of-time over background-mutator

(unchanged from v1.0 §3.1 — the reasoning is correct and unaffected by U-01/U-02)

The primary alternative is a background tokio task that mutates shared mutable state.
This is rejected for three reasons:
1. Non-reproducibility: exact stage depends on process start time, not a deterministic function.
2. Cross-clone synchronization hazard: independent background tasks can diverge by one poll interval.
3. Concurrency complexity: `Arc<Mutex<StageIndex>>` inside axum violates `await_holding_lock = "deny"` (ADR-002 §H1).

### 3.2 Retrofit-generate over per-request regeneration

The v1.0 §2 "filter the pre-generated FixtureSet" approach is still the correct
production-grade design — generate once at construction, filter per-request. The v2.0
correction is that "generate once at construction" requires a new constructor path since
the current `new()` constructors do NOT call `generate()`. The per-request regeneration
alternative (Option B from §5) is still rejected for the same reasons.

### 3.3 Lookup injection over new ThreatIntel/NVD generator

(unchanged from v1.0 §3.2 — ThreatIntel and NVD serve pure key-value lookups; lookup
injection is simpler, sufficient, and directly testable)

### 3.4 Shared `ScenarioEntityCatalog` over per-DTU entity derivation

(unchanged from v1.0 §3.3 — the shared catalog is the authoritative entity namespace;
per-DTU independent derivation risks RNG state consumption divergence)

### 3.5 CrowdStrike device ID format correction

The v1.0 design cited `"dev-acme-100-0"` as an example cross-DTU device ID. This format
is incorrect. The real CrowdStrike generator `org_slug()` function derives the slug from
UUID bytes (`hex(org_id.as_bytes()[0..4])`), producing an 8-character hex string.
`"dev-acme-..."` was a placeholder that does not match any real generator output.
The canonical format is `"dev-{8hex}-{seed}-{n}"`. All spec documents, tests, and
example configurations MUST use the canonical format with an actual org UUID.

---

## 4. Consequences

### Positive

- The baseline seeding retrofit (Story A) unblocks BC-2.06.018 implementation and
  provides the substrate that BC-2.06.019 / BC-2.06.020 require.
- A SOC-analyst demo from any stage is reproducible (same `scenario_start_secs` + same
  `seed` → same timeline every time).
- Cross-DTU join is coherent: same `org_slug`, same `seed` → same device ID prefix
  across all operational DTUs via the shared `ScenarioEntityCatalog`.
- ThreatIntel and NVD enrichment are scenario-correlated without a new generator.
- No background tasks, no shared mutable progression state.

### Negative / Trade-offs

- Story A (baseline retrofit) is non-trivial: 4 generator-backed clones each need a
  new `new_with_seed` constructor and route-handler dual-path logic. This is ~2-3 days
  of implementation per the story split estimate in Section 8.
- `DemoConfig` gains `org_id` as a new required field for scenario mode — operators
  must supply a UUID in `demo.toml`. A validation error at startup (`E-DEMO-004`)
  surfaces this immediately.
- `ScenarioEntityCatalog` and `IncidentTimeline` are `#[non_exhaustive]`, so the
  ci.yml `EXPECTED` counter must be incremented by the implementer (from 49 to 51,
  exact count to be verified by the implementer reading the non-exhaustive gate).

---

## 5. Alternatives Considered

- **Option A — Background mutation task.** Rejected: non-reproducible, cross-clone
  synchronization hazard, `await_holding_lock` violation (see §3.1).

- **Option B — Per-request re-generation.** Rejected: generation latency per-request;
  concurrent requests in the same stage can produce different field values.

- **Option C — Poll-count-based staging.** Rejected: ties progression to prism polling
  frequency; operators need wall-clock "6 minutes in" semantics.

- **Option D — ThreatIntel/NVD generator.** Rejected: enrichment DTUs serve pure
  key-value lookups; a full generator adds specification burden without demo value.

- **Option E — Per-DTU independent entity derivation.** Rejected: RNG state consumption
  divergence risk produces incoherent cross-DTU device IDs.

- **Option F (v2.0 new) — Harness-delegated generation (use prism-dtu-harness
  generators in demo-server).** The harness already calls `generate()` for its test
  clones. Option F would have demo-server import `prism-dtu-harness` and reuse its
  `CloneState`-based generation. Rejected: `prism-dtu-harness` is a test-only crate
  (`publish = false`, only activated under `#[cfg(any(test, feature = "dtu"))]`);
  making demo-server depend on it would bring test infrastructure into the production
  binary path and tightly couple demo-server to harness internals. The retrofit
  approach (new constructors per clone) is cleaner and maintains the crate boundary.

---

## 6. New Error Codes

Four new `E-DEMO-NNN` codes required (to be registered in `.factory/specs/prd-supplements/error-taxonomy.md`):

| Code | Category | Trigger | Message |
|------|----------|---------|---------|
| `E-DEMO-002` | configuration | `scenario.enabled = true` for multiple clones with different `seed` values | `"demo-server: E-DEMO-002: scenario clones {A} (seed={X}) and {B} (seed={Y}) have different seeds; cross-DTU coherence requires all scenario-enabled clones to share the same seed"` |
| `E-DEMO-003` | configuration | `scenario.archetype` is an unrecognized string | `"demo-server: E-DEMO-003: clone '{name}': unrecognized scenario archetype '{value}'; valid values: compromised_endpoint"` |
| `E-DEMO-004` | configuration | `scenario.enabled = true` but no `org_id` supplied | `"demo-server: E-DEMO-004: clone '{name}': scenario.enabled requires org_id to be set (UUID string)"` |
| `E-DEMO-005` | configuration | `org_id` present but not a valid UUID | `"demo-server: E-DEMO-005: clone '{name}': org_id '{value}' is not a valid UUID"` |

---

## 7. BCs Flagged for PO Authorship

### BC-2.06.018 Substrate Corrections (flag to PO)

The PO must update BC-2.06.018 to reflect the substrate reality:
- Postconditions claiming seeded data differentiation (per-client distinct device IDs)
  are currently UNIMPLEMENTED. The BC's "implemented" status must remain `draft`
  until Story A closes.
- Add a note: "`CloneConfig.seed` is declared but not forwarded to clone constructors
  until S-DEMO-DTU-LIVE-SCENARIO-001-A implements `new_with_seed` constructors."

### BC-2.06.019 Corrections (flag to PO)

The PO must update the `stage_duration_secs` description: the array has **4 entries**
(for stages 1-4 activation thresholds). Stage 0 always activates at 0. AC-009 must
reflect `activates_after_secs` terminology, not `stage_duration_secs` (which is the
config key name). The BC must specify the 4-entry convention explicitly.

### BC-2.06.020 Corrections (flag to PO)

The PO must update BC-2.06.020 to use the correct NVD API paths:
- `INV-NVD-CVE-CORRELATION-001` must cite the CVSS access path:
  `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score >= 7.0` (type: `f64`)
  and `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_severity` (type: `String`).
- Remove any reference to `NvdClone.lookup()` — the method is `NvdState::lookup_and_count()`.
- `NvdClone::new_with_scenario()` returns `anyhow::Result<Self>` (fallible, like `new()`).
  The BC postcondition test must handle the `Result`.

---

## 8. Story Split (User-Authorized, corrects v1.0 §8)

The v1.0 single-story recommendation is superseded. The remove-uncertainty scan
confirmed that BC-2.06.018 baseline seeding is entirely unimplemented (U-01/U-02),
making a single-story delivery unrealistic at production-grade. The user has authorized
splitting into:

### Story A: `S-DEMO-DTU-LIVE-SCENARIO-001-A` — Baseline Seeding Retrofit

**BC:** BC-2.06.018

**Scope:**
1. `prism-dtu-common/src/scenario/` module stub: `ScenarioEntityCatalog`, `org_slug_from_org_id()`,
   secondary RNG stream catalog derivation (IOC IPs, domains, hashes, CVE IDs).
   Gated `#[cfg(feature = "fixture-gen")]`. `#[non_exhaustive]` on all public types.
2. `DemoConfig`/`CloneConfig` extension: `org_id: Option<String>`, `ScenarioConfig` struct
   (enabled, archetype, scenario_start_secs, stage_duration_secs).
3. Per-clone `new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId)` constructors:
   CrowdStrike, Armis (also derives `org_slug` internally), Claroty, Cyberint (fallible).
   Each calls `generate(org_id, [org_slug,] archetype, &GenOpts { seed, ..GenOpts::default() })`
   under `fixture-gen` feature; the `archetype` argument is the INV-FIXTURE-SET-ARCHETYPE-MAP-001
   mapping of `CloneConfig.fixture_set` forwarded by `build_clone_pairs`. NOT hardcoded to
   `CompromisedEndpoint`. Stores records in state. Route handlers add dual-path (generated vs
   static-JSON fallback).
4. `build_clone_pairs` seed-and-archetype forwarding: read `CloneConfig.seed` and
   `CloneConfig.fixture_set`; map `fixture_set→Archetype` via INV-FIXTURE-SET-ARCHETYPE-MAP-001;
   forward both `seed` and the resolved `Archetype` to `new_with_seed(seed, archetype, org_id)`.
   The mapped `Archetype` drives the generator for all 8 fixture_set values — NO internal
   default to `CompromisedEndpoint` anywhere in this call path.
   Add `E-DEMO-001` (unrecognized fixture_set), `E-DEMO-002` (seed mismatch when
   scenario.enabled on multiple clones), `E-DEMO-004` (missing org_id when new_with_seed
   called with non-default archetype), `E-DEMO-005` (invalid UUID).
5. Cargo.toml additions: `fixture-gen` feature to `prism-dtu-threatintel` and `prism-dtu-nvd`.
6. Update ci.yml `EXPECTED` to account for new `#[non_exhaustive]` types (exact count by implementer).
7. Integration test: start demo-server with seed=1 and seed=2 for two "clients"; assert
   device IDs are pairwise-disjoint (cross-client isolation).
8. Backward-compat regression: seed=42 default path produces same results as pre-Story-A.

**Point estimate: 8 points.** This is entirely new constructor + serving-path wiring across
4 generator-backed clones — the largest single unit of work in the ADR-036 scope.

**Disposition of existing `S-DEMO-DTU-LIVE-SCENARIO-001`:** Story-writer supersedes this
file with `S-DEMO-DTU-LIVE-SCENARIO-001-A` (Story A scope) and creates
`S-DEMO-DTU-LIVE-SCENARIO-001-B` (Story B scope). The original file should be marked
superseded in its frontmatter.

### Story B: `S-DEMO-DTU-LIVE-SCENARIO-001-B` — Live Scenario Progression + Enrichment

**BCs:** BC-2.06.019 (scenario progression), BC-2.06.020 (enrichment correlation)

**Depends on:** S-DEMO-DTU-LIVE-SCENARIO-001-A (must merge first)

**Scope:**
1. `IncidentTimeline`, `IncidentStage`, `StageMask`, `current_stage_index()` in
   `prism-dtu-common/src/scenario/`. `#[non_exhaustive]` on all public types.
2. Per-clone `new_with_scenario(seed, archetype, org_id, timeline)` constructors:
   CrowdStrike, Armis, Claroty, Cyberint. Adds `timeline: Option<Arc<IncidentTimeline>>`
   to state; route handlers add stage-mask filter on top of Story A's generated records.
3. `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> Self`.
4. `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> anyhow::Result<Self>`.
   Builds `cve_registry` including scenario CVEs with CVSS records
   (`base_score = 8.1`, `base_severity = "HIGH"`, per `CvssData` struct fields).
5. `build_clone_pairs` scenario coordination: catalog derivation, `IncidentTimeline`
   construction, `Arc<IncidentTimeline>` threading to 4 operational clones, catalog
   injection to ThreatIntel + NVD.
6. Integration test: multi-org CompromisedEndpoint scenario; assert cross-DTU entity
   coherence at stage 2 (LateralMovement) and IOC resolution in ThreatIntel. Assert
   NVD CVE resolution with `base_score >= 7.0`.

**Point estimate: 7 points.**

---

## 9. Source / Origin

- **Decision D-1077** (2026-06-09) — user-directed scope expansion.
- **ADR-009** — the generator architecture this extends.
- **BC-2.06.018** — the baseline seeding BC.
- **Remove-uncertainty scan U-01..U-09** (2026-06-09) — corrected substrate reality
  for this v2.0 revision.
- **Code as-built — serving path verification:**
  `crates/prism-dtu-crowdstrike/src/clone.rs:CrowdstrikeClone::new()` — confirms no
  `generate()` call; state has empty stores only.
  `crates/prism-dtu-armis/src/state.rs:ArmisState::new()` — confirms fixture loading
  from `Vec<DeviceRecord>` (static JSON), not `generate()`.
  `crates/prism-dtu-demo-server/src/harness.rs:build_clone_pairs()` — confirms no seed
  forwarding; `CloneConfig.seed` field unused.
- **Code as-built — generator signatures:**
  `crates/prism-dtu-crowdstrike/src/generator.rs:generate(org_id: OrgId, ...)` and
  `org_slug(org_id: &OrgId) -> String` (8 hex chars from UUID bytes).
  `crates/prism-dtu-armis/src/generator.rs:generate(org_id: OrgId, org_slug: &str, ...)`.
  `crates/prism-dtu-common/src/generator/rng.rs:seeded_rng(seed: u64, org_id: &OrgId)`.
- **Code as-built — NVD types:**
  `crates/prism-dtu-nvd/src/types.rs:CveRecord.metrics.cvss_metric_v31: Option<Vec<CvssMetricV31>>`
  `CvssMetricV31.cvss_data: CvssData`, `CvssData.base_score: f64`, `CvssData.base_severity: String`.
  `crates/prism-dtu-nvd/src/state.rs:NvdState.cve_registry: HashMap<String, CveRecord>` (NOT Mutex).
  `crates/prism-dtu-nvd/src/clone.rs:NvdClone::new() -> anyhow::Result<Self>`.
- **Code as-built — ThreatIntel:**
  `crates/prism-dtu-threatintel/src/state.rs:ThreatIntelState.fixture_registry: Mutex<HashMap<String, FixtureKey>>`.
  `crates/prism-dtu-threatintel/src/clone.rs:ThreatIntelClone::new() -> Self` (infallible).
- **Code as-built — Cargo features:**
  `crates/prism-dtu-threatintel/Cargo.toml` — no `chrono`, no `fixture-gen` feature.
  `crates/prism-dtu-nvd/Cargo.toml` — no `chrono`, no `fixture-gen` feature.
  `crates/prism-dtu-common/Cargo.toml` — `fixture-gen` feature enables `chrono` + `prism-core`.
  `.github/workflows/ci.yml` — `EXPECTED=49` (non-exhaustive violation count).

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-09 | architect | Initial authoring. Decision D-1077. |
| 2.0 | 2026-06-09 | architect | Substrate-corrected. U-01: clones serve static JSON, generators not in serving path. U-02: CloneConfig.seed never forwarded; baseline seeding unimplemented. U-03: org_slug is 8-hex-from-UUID not arbitrary string; corrected ID format. U-05: NvdClone::new() is fallible; cve_registry is immutable HashMap; no lookup() method; corrected CVSS path. U-06: DemoConfig has no org_id field; added required new field. U-07/U-08: threatintel/nvd have no fixture-gen feature; added Cargo requirements; perimeter confirmed safe. U-09: pinned stage_duration_secs as 4-entry array for 5-stage timeline. Story split authorized and specified. |
| 2.1 | 2026-06-09 | architect | Symbol/path/syntax corrections from Story-A re-validation scan. U-A-01: catalog derivation call site corrected from `seeded_rng` to `gen_seeded_rng` (two-arg re-export alias in prism-dtu-common::lib; avoids collision with one-arg legacy seeded_rng). U-A-04: CrowdStrike device-read fallback corrected from "stateful-write-target path (containment_store/detection_status_store)" to `load_host_ids()/load_host_details()` (static JSON helpers in routes/hosts.rs); containment_store/detection_status_store are write-target overlays only. U-A-07: GenOpts struct-literal corrected from bare `..` to `..GenOpts::default()` in all three Phase A call sites (§2.3 Phase A description and per-clone CrowdStrike + Armis constructor notes). No design change. |
| 2.2 | 2026-06-09 | architect | Archetype-forwarding reconciliation. F-P6-HIGH-001 + user decision 2026-06-09: the canonical Story-A constructor is the **3-arg** `new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId)` for all four generator-backed clones. The v2.1 §2.3 per-clone constructor notes had a substrate-reconciliation drift error: CrowdStrike was `new_with_seed(seed, org_id)` (2-arg, implicitly hardcoding `CompromisedEndpoint`); Armis was `new_with_seed(seed, org_id, org_slug)` (no `archetype`). Both are now corrected to the 3-arg form with explicit `archetype` parameter. Armis derives `org_slug` internally from `org_id` rather than taking it as a constructor argument (keeps the constructor signature symmetric with the other three clones). Claroty and Cyberint prose updated to state the 3-arg form explicitly (previously said "same pattern" which was ambiguous). §2.3 Phase A description updated: `archetype` note added to the generate() call. §2.4 step 9 expanded to document the Story A non-scenario path explicitly (previously only documented the Story B scenario path). §8 scope item 4 updated to state that `build_clone_pairs` maps `fixture_set→Archetype` and forwards the result to `new_with_seed`. §2.5 VP-018-B backward-compat canonical call documented: `new_with_seed(42, Archetype::HealthyOtEnvironment, <default_org_id>)`. Root cause of drift: v2.0/v2.1 focused on substrate corrections (empty state stores, no generate() call in serving path) and chose the 2-arg form as a "simpler retrofit"; this was incorrect because BC-2.06.018 Postcondition 1, INV-FIXTURE-SET-ARCHETYPE-MAP-001, EC-018-003, EC-018-005, TV-018-005, TV-018-006, and VP-018-C all specify full 8-archetype support driven by fixture_set. The 2-arg form was drift, not a deliberate scope decision. |
