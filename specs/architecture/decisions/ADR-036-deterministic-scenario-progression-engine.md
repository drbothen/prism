---
document_type: adr
adr_id: ADR-036
title: "Deterministic Scenario-Progression Engine — IncidentTimeline, Per-DTU Projection, and Cross-DTU Entity Coherence for Live Demo"
status: ACCEPTED
date: 2026-06-09
wave: 5
phase: 3.demo
version: "1.0"
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
  - crates/prism-dtu-crowdstrike/src/state.rs
  - crates/prism-dtu-crowdstrike/src/generator.rs
  - crates/prism-dtu-threatintel/src/state.rs
  - crates/prism-dtu-nvd/src/state.rs
  - crates/prism-dtu-demo-server/src/harness.rs
  - crates/prism-dtu-demo-server/src/config.rs
  - .factory/specs/behavioral-contracts/BC-2.06.018-dtu-demo-clone-data-seeding.md
  - .factory/objectives/multi-client-soc-demo-tasks.md
wiring_deferred_to: S-DEMO-DTU-LIVE-SCENARIO-001
---

# ADR-036: Deterministic Scenario-Progression Engine — IncidentTimeline, Per-DTU Projection, and Cross-DTU Entity Coherence for Live Demo

## Status

ACCEPTED 2026-06-09. Decision D-1077. Governs the live-scenario story (S-DEMO-DTU-LIVE-SCENARIO-001, T4/T5 in multi-client-soc-demo-tasks.md). Extends ADR-009 (multi-tenant deterministic generator) with a temporal scenario-progression layer.

---

## 1. Context

### 1.1 The Baseline (ADR-009 + BC-2.06.018)

ADR-009 established the multi-tenant deterministic generator for DTU behavioral
clones. It is a pure function: `generate(org_id, sensor, archetype, GenOpts) ->
FixtureSet`. The `CompromisedEndpoint` and `HighChurn` archetypes model a
compromised or churning environment, but they produce a **static snapshot** — all
records are generated at clone construction time and served unchanged for the
lifetime of the process. There is no notion of time passing.

BC-2.06.018 governs config-time seeding: `CloneConfig.seed` + `CloneConfig.fixture_set`
are forwarded from `demo.toml` through `build_clone_pairs` to each clone constructor,
causing per-client data differentiation. This BC is the baseline that ADR-036 extends.

### 1.2 The Gap (D-1077)

The SOC-analyst live demo (multi-client-soc-demo-tasks.md) requires an incident that
*unfolds* over time. An analyst initiating a demo investigation should see: early
reconnaissance indicators in Armis and Claroty, then lateral movement detections in
CrowdStrike, then exfiltration IOCs resolving in ThreatIntel, then patched CVEs
resolving in NVD. This temporal narrative is the core of a believable SOC demo.

Three specific requirements, recorded as D-1077:

1. **Scenario progression (temporal staging).** The `CompromisedEndpoint` archetype
   must evolve through named stages (e.g., `Recon → LateralMovement → Exfil →
   Containment`), with different sets of devices, alerts, and events becoming visible
   at each stage. MUST be reproducible: same seed + same clock-offset produces the
   same timeline every time.

2. **Cross-DTU entity coherence.** The same compromised device identifier must
   appear in Armis (device inventory), CrowdStrike (detection hit), Claroty (OT asset),
   and Cyberint (intelligence alert). The IOC hashes from the attack must resolve in
   ThreatIntel. The CVEs on the affected device must resolve in NVD. An analyst
   joining data across DTUs must produce a coherent incident picture, not four
   independent inventories.

3. **Enrichment DTU correlation.** ThreatIntel and NVD currently serve static lookup
   tables that are unrelated to the active scenario. They must be seeded at
   construction time with scenario-correlated entries: the incident's IOC hashes/IPs
   resolve as malicious; the affected device's CVE IDs resolve with realistic CVSS
   records.

### 1.3 Constraints from the Existing Architecture

The generator (ADR-009) is a pure function behind `feature = "fixture-gen"` in
`prism-dtu-common`. The `INV-PERIMETER-001` rule (enforced by compile-fail gate in
`tests/external/perimeter-violation/`) prohibits `prism-dtu-*` crates from depending
on `prism-spec-engine`, `prism-sensors`, or `prism-query`. All new types must stay
within `prism-dtu-common` or the individual DTU clone crates.

The current clone state model (Armis: `ArmisState`, CrowdStrike: `CrowdstrikeState`)
holds immutable fixture registries loaded once at construction time. `ArmisState`
has no temporal notion. `GenOpts` has a `time_anchor: DateTime<Utc>` field but it
is used only as a timestamp anchor for record field values, not for stage gating.

The demo-server harness (`DemoHarness`, `build_clone_pairs`) constructs all clones
sequentially and holds them in a `Vec<ClonePair>` with no shared coordinator object.
There is no existing mechanism to share state across clone instances.

---

## 2. Decision

### 2.1 Time/Clock Model: Pure Function of Wall-Clock Elapsed Seconds

**Decision:** The progression is computed as a pure function of `(seed,
scenario_start_epoch_secs, now_secs)` on every inbound HTTP request. There is no
background mutator task. There are no mutable stage counters. The route handler
calls `current_stage(scenario_start, Utc::now(), stage_durations) -> StageIndex`
and projects the full scene for that stage.

```text
elapsed_secs = now_unix_secs - scenario_start_unix_secs
stage_index  = max stage S such that elapsed_secs >= stage_durations[0..S].sum()
```

This model has three properties that are critical for the demo:

- **Reproducibility.** Same `scenario_start_epoch_secs` + same `seed` + same
  `elapsed_secs` → identical response bodies. A recording made at T+120s is
  byte-identical to a live serve at T+120s. This is the only model compatible with
  BC-3.4.001 determinism and reproducible demo recordings.

- **No background task.** No tokio `spawn`, no `Arc<AtomicU64>`, no shared mutable
  progression state. Each request is stateless with respect to progression. This is
  the correct production-grade default: a background task that mutates shared state
  across clone boundaries is a concurrency hazard with no benefit (the pure function
  model is strictly simpler and equally "continuous" from the analyst's perspective).

- **Demo "continuous" feel.** Stage durations are configurable per-scenario in
  `demo.toml` (e.g., `stage_duration_secs = [60, 120, 180, 300]` for 4 stages at
  1/2/3/5-minute marks). Between stage transitions the data is stable. During a live
  demo, the operator can let the demo progress naturally or fast-forward by adjusting
  the `scenario_start` timestamp. Both are reproducible.

**`scenario_start` source.** `CloneConfig` gains an optional
`scenario_start_secs: Option<i64>` field (unix epoch seconds). When `None`, the
clone uses a default of `now - 0` (i.e., the scenario starts at construction time,
so the demo begins at stage 0). The operator sets an explicit `scenario_start_secs`
in `demo.toml` to synchronize all DTUs to a shared timeline.

**Stage duration defaults.** The `IncidentTimeline` (Section 2.2) defines default
stage durations. `demo.toml` can override them per-client via a `[clones.*.scenario]`
block (see Section 2.4).

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
#[derive(Clone, Debug)]
pub struct ScenarioEntityCatalog {
    /// The primary compromised device ID (used by Armis, CrowdStrike, Claroty, Cyberint).
    /// Format: "dev-{org_slug}-{seed}-0"  (first device in CompromisedEndpoint generator).
    pub primary_device_id: String,
    /// Hostname for the compromised device (consistent across DTUs).
    pub primary_hostname: String,
    /// Secondary device IDs involved in lateral movement.
    pub lateral_devices: Vec<String>,
    /// IOC IPv4 addresses introduced during Exfil stage.
    /// These MUST resolve as malicious in ThreatIntel.
    pub ioc_ips: Vec<String>,
    /// IOC domain names introduced during Exfil stage.
    pub ioc_domains: Vec<String>,
    /// IOC SHA256 file hashes introduced during LateralMovement stage.
    pub ioc_hashes: Vec<String>,
    /// CVE IDs assigned to the primary device.
    /// These MUST resolve in NVD.
    pub device_cves: Vec<String>,
}
```

The catalog is derived deterministically from `(seed, org_id)`:
- `primary_device_id` matches the first device ID that the `CompromisedEndpoint`
  generator produces for this `(seed, org_id)` pair (i.e., `"dev-{org_slug}-{seed}-0"`).
  This is NOT a fresh invention — it reuses the existing ID format from ADR-009 §2.5
  and `gen_compromised_endpoint` in `crates/prism-dtu-crowdstrike/src/generator.rs`.
- IOC IPs, domains, hashes, and CVE IDs are derived from `seeded_rng(seed + 1, org_id)`
  (a secondary RNG stream so the catalog derivation does not consume generator RNG state).
  They are formatted with org-slug prefixes where applicable (e.g.,
  `"ioc-{org_slug}-{seed}-0.evil.example.com"`) to maintain cross-client disjointness.

#### `IncidentTimeline`

```rust
/// Timeline for a staged incident scenario.
/// Carries the entity catalog plus stage definitions.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct IncidentTimeline {
    /// The entity catalog shared by all DTU projections.
    pub entities: ScenarioEntityCatalog,
    /// Ordered stage definitions. Stage 0 is always the baseline.
    pub stages: Vec<IncidentStage>,
    /// Unix epoch seconds at which the scenario started.
    pub scenario_start_epoch_secs: i64,
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct IncidentStage {
    /// Human name for demo narration (e.g., "Recon", "LateralMovement").
    pub name: &'static str,
    /// Cumulative elapsed seconds from scenario_start before this stage activates.
    pub activates_after_secs: u64,
    /// Which entity subsets are visible at this stage (bit-flag or Vec<EntityRef>).
    pub visible_entity_mask: StageMask,
}

/// Which entities from ScenarioEntityCatalog are visible at a given stage.
/// Controls per-DTU projection logic (Section 2.3).
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

**Default `CompromisedEndpoint` stage definitions:**

| Stage | Name | Activates after | Visible |
|-------|------|-----------------|---------|
| 0 | `Baseline` | 0s | primary device only; no alerts |
| 1 | `Recon` | 60s | primary device + low-severity alerts |
| 2 | `LateralMovement` | 180s | primary + lateral devices; IOC hashes emerge |
| 3 | `Exfil` | 360s | all devices; IOC IPs + domains emerge |
| 4 | `Containment` | 600s | primary device contained; all IOCs; CVEs visible in NVD |

Stage durations are overridable via `[clones.*.scenario.stage_duration_secs]` in
`demo.toml`. The stage activation logic is a pure function:

```rust
/// Compute the current stage index for this timeline.
/// Pure function of current time: no mutations, no side effects.
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

### 2.3 Per-DTU Projection

Each generator-backed clone projects the `IncidentTimeline` into its own API response
shape. The projection function is per-DTU and co-located with that DTU's generator
module.

#### Generator-backed clones (Armis, CrowdStrike, Claroty, Cyberint)

Each clone's state struct gains an optional `timeline: Option<Arc<IncidentTimeline>>`.
When `None`, the clone serves a static snapshot (backward-compatible path — Section 2.5).
When `Some(timeline)`, the route handlers:

1. Call `current_stage_index(&timeline, Utc::now().timestamp())` to determine the
   active stage and its `StageMask`.
2. Filter the pre-generated `FixtureSet` records using the stage mask:
   - Devices/assets not yet visible in the current stage are withheld from the response.
   - Alert records use the stage to determine severity progression (recon: LOW;
     lateral movement: MEDIUM+; exfil: CRITICAL).
3. The underlying `FixtureSet` is generated once at construction time (same as today),
   and the stage mask is applied as a per-request filter — NO re-generation on each
   request.

This design is critical: the generator runs once (deterministic, at construction time),
and the stage mask selects a subset of those records per request. The records
themselves never change; only which records are visible changes over time.

**CrowdStrike specifics.** The `CompromisedEndpoint` generator already marks
`device[0]` as `containment_status = "contained"` and generates 5 severity_id ≥ 4
detections. At `Containment` stage, the `contained` status becomes visible; at earlier
stages, `normal` is served for that device. This is implemented by a stage-aware
projection in the detections and devices routes.

**Armis specifics.** Armis device inventory shows the primary device from stage
`Recon` onward. At `Containment` stage, the device's `risk_level` is elevated.

#### Enrichment clones (ThreatIntel, NVD)

ThreatIntel and NVD have no generator today. They serve a static registry
(`fixture_registry: Mutex<HashMap<String, FixtureKey>>`). The mechanism for
scenario-correlation is **scenario-keyed registry injection at construction time**
— NOT a new generator.

**Decision: static lookup injection, not a new generator.** The rationale: ThreatIntel
and NVD serve pure lookup semantics (key → value with no pagination, no session state,
no archetype-specific shapes). A lookup table injected at construction time is
architecturally simpler, faster to test, and sufficient for the demo requirement. A
full generator for ThreatIntel/NVD adds complexity without behavioral value (there is
no SOC demo narrative benefit to generating thousands of random IOCs — the analyst
investigates the *specific* IOCs from the scenario).

**Implementation:** When `build_clone_pairs` constructs ThreatIntel and NVD with
an active scenario, it calls an overloaded constructor `ThreatIntelClone::new_with_scenario`
and `NvdClone::new_with_scenario` that pre-populates the registry:

- `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog)`: inserts
  `entities.ioc_ips`, `entities.ioc_domains`, and `entities.ioc_hashes` into
  `fixture_registry` with `FixtureKey::Malicious`. All other lookups return the default
  registry behavior (Benign or Unknown).

- `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog)`: inserts
  `entities.device_cves` into `cve_registry` with realistic synthetic CVSS records
  (severity HIGH, CVSS 8.1, vector string that implies the attack type). CVE IDs
  not in `device_cves` continue to resolve normally (or return 404 for unknown IDs).

Both constructors satisfy `INV-PERIMETER-001`: they accept a `ScenarioEntityCatalog`
(from `prism-dtu-common`) and do not depend on `prism-spec-engine` or `prism-sensors`.

**Stage-gate enrichment.** ThreatIntel IOCs resolve as Malicious from the moment
the clone starts (the analyst can look them up at any stage). NVD CVEs also resolve
immediately. This is intentional: the analyst *discovers* the IOCs and CVEs by
running queries against Armis/CrowdStrike/Cyberint that surface them at the
appropriate stage, then pivots to ThreatIntel/NVD to enrich. The enrichment DTUs
are always "ready" — the stage progression is driven by the operational DTUs, not
the enrichment DTUs.

### 2.4 Cross-Clone Coordination: `ScenarioConfig` in `demo.toml`

The shared `ScenarioEntityCatalog` is constructed once in `build_clone_pairs` and
distributed to each clone constructor. This is the coordination point.

**New `demo.toml` fields:**

```toml
[clones.crowdstrike.scenario]
enabled = true
archetype = "compromised_endpoint"  # maps to IncidentTimeline variant
scenario_start_secs = 1749456000    # unix epoch; omit for "now"
stage_duration_secs = [60, 180, 360, 600]  # cumulative activation seconds

[clones.armis.scenario]
enabled = true
archetype = "compromised_endpoint"
scenario_start_secs = 1749456000    # SAME value as crowdstrike — cross-DTU sync

# ThreatIntel and NVD: no stage config needed — they receive entity catalog at construction
[clones.threatintel.scenario]
enabled = true

[clones.nvd.scenario]
enabled = true
```

**`CloneConfig` extension:**

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ScenarioConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_scenario_archetype")]
    pub archetype: String,  // "compromised_endpoint" | "healthy" | ... (extensible)
    #[serde(default)]
    pub scenario_start_secs: Option<i64>,
    #[serde(default)]
    pub stage_duration_secs: Vec<u64>,  // empty = use archetype defaults
}
```

**`build_clone_pairs` coordination logic:**

1. Parse `ScenarioConfig` from `DemoConfig`. If any clone has `scenario.enabled = true`,
   derive the shared `ScenarioEntityCatalog` from `(seed, org_id)` using the first
   enabled clone's seed (all enabled clones in a client config MUST have the same seed
   — if they differ, `build_clone_pairs` returns `E-DEMO-002`).
2. Construct the `IncidentTimeline` from the catalog + stage definitions.
3. Pass `Arc<IncidentTimeline>` to each generator-backed clone constructor as an optional
   parameter: `CrowdstrikeClone::new_with_scenario(seed, archetype, org_id, timeline)`.
4. Pass `&ScenarioEntityCatalog` to `ThreatIntelClone::new_with_scenario` and
   `NvdClone::new_with_scenario`.
5. When `scenario.enabled = false` for a clone (or the `[clones.*.scenario]` block is
   absent), the clone is constructed with the static path (backward-compatible with
   BC-2.06.018 postcondition 4).

### 2.5 Backward Compatibility and Perimeter

**Backward-compatible path.** All existing constructors (`CrowdstrikeClone::new()`,
`ArmisClone::new()`, etc.) are unchanged. Scenario progression is opt-in via the
`[clones.*.scenario]` block in `demo.toml`. When absent, behavior is byte-identical
to pre-ADR-036 behavior (BC-2.06.018 postcondition 4).

**INV-PERIMETER-001.** `ScenarioEntityCatalog` and `IncidentTimeline` live in
`prism-dtu-common` (behind `feature = "fixture-gen"`). They carry no dependency on
`prism-spec-engine`, `prism-sensors`, or `prism-query`. The compile-fail gate in
`tests/external/perimeter-violation/` continues to hold.

**Static-fixture path unaffected.** ThreatIntel and NVD `new()` constructors load
the default registry unchanged. Only `new_with_scenario` injects scenario-correlated
entries.

**Single-instance regression safety.** Existing integration tests that instantiate
clones without a `ScenarioConfig` are unaffected. The `Option<Arc<IncidentTimeline>>`
field in state structs defaults to `None`, and `None` → static snapshot path.

---

## 3. Rationale

### 3.1 Pure-function-of-time over background-mutator

The primary alternative for "continuous" progression is a background tokio task that
periodically updates shared mutable state (e.g., an `AtomicUsize` stage counter
or a `Mutex<StageIndex>` in each clone). This is rejected for three reasons:

1. **Non-reproducibility.** A background task advances the stage counter at wall-clock
   intervals. The exact stage at demo time T depends on when the process started, not
   on a deterministic function of inputs. Demo recordings taken with `demo-recorder`
   would capture a stage that cannot be reproduced exactly in a unit test without
   mocking `SystemTime`. The pure-function model makes every request a deterministic
   function of `(seed, scenario_start_secs, now_secs)` — testable without mocking
   shared state.

2. **Cross-clone synchronization hazard.** If each clone runs its own background task,
   stage transitions between Armis and CrowdStrike can diverge by one polling interval,
   producing incoherent cross-DTU snapshots (Armis says stage 2, CrowdStrike says
   stage 1). The pure-function model uses a single shared `scenario_start_epoch_secs`
   field — all clones compute the same stage from the same inputs, guaranteed.

3. **Concurrency complexity.** A background task mutating `Arc<Mutex<StageIndex>>`
   inside a running axum server creates a shared mutable state pattern that
   `await_holding_lock = "deny"` (ADR-002 §H1) makes hazardous. The pure function
   requires no locks on the progression state.

### 3.2 Lookup injection over new ThreatIntel/NVD generator

Generating a full IOC/CVE dataset for ThreatIntel and NVD would require specifying:
record count, threat score distributions, CVSS vector generation logic, CWE weighting,
KEV subset logic — a substantial specification and test burden for purely cosmetic
coverage. The lookup injection approach covers the only behavioral invariant that
matters: the scenario's IOCs resolve as Malicious and the scenario's CVEs resolve with
data. Everything else in those registries retains its default behavior.

### 3.3 Shared `ScenarioEntityCatalog` over per-DTU entity derivation

An alternative approach is to derive entity IDs independently in each DTU generator
from the same `(seed, org_id)` tuple. This is rejected because it requires every
DTU generator to implement the same derivation logic and keep it synchronized. A
subtle divergence in derivation — one DTU calls `seeded_rng(seed, org_id)` and
generates 5 records before deriving the IOC IPs; another starts from a fresh RNG
stream — produces different IOC IPs in different DTUs, breaking cross-DTU coherence.
The shared catalog is the authoritative source of truth; DTU projections read from it
rather than re-derive entity identifiers.

### 3.4 `prism-dtu-common` as the home for scenario types

The scenario types (`ScenarioEntityCatalog`, `IncidentTimeline`, `IncidentStage`,
`StageMask`) follow the same placement decision as ADR-009 §2.8: they belong in
`prism-dtu-common` behind `feature = "fixture-gen"` because they are shared test
infrastructure consumed by all DTU crates. Creating a separate `prism-dtu-scenario`
crate would split the generator and scenario modules across two crates with a
unidirectional dependency (scenario → generator) that provides no architectural
benefit for the current scope.

---

## 4. Consequences

### Positive

- A SOC-analyst demo from any stage is reproducible: set `scenario_start_secs` in
  `demo.toml`, and the same demo runs at the same stage every time — no "get there
  before the stage expires" fragility.
- Cross-DTU join is coherent: the same `primary_device_id` appears in Armis, CrowdStrike,
  and Claroty because all projections draw from `ScenarioEntityCatalog`. The analyst's
  PrismQL JOIN across sensor tables surfaces a believable incident.
- ThreatIntel and NVD enrichment are scenario-correlated without a new generator:
  lookup injection is simple, testable, and sufficient.
- Backward compatibility is structural: `Option<Arc<IncidentTimeline>>` = `None` is the
  static path; existing tests are unaffected.
- No background tasks, no shared mutable progression state: the progression layer is
  zero-overhead when `scenario.enabled = false`, and pure-function-overhead (two
  timestamp comparisons per request) when `true`.

### Negative / Trade-offs

- `CloneConfig` gains a `ScenarioConfig` field. `build_clone_pairs` gains scenario
  coordination logic (approximately 40-60 lines). This increases demo-server complexity.
- The stage mask applied per-request means that even though the FixtureSet is generated
  once, the route handlers must implement filtering logic — two code paths (static path,
  scenario path) per route. Each DTU's routes need the additional match arm.
- `scenario_start_secs` must be kept synchronized across all DTUs in `demo.toml`.
  If the operator sets different values for different clones, `build_clone_pairs`
  returns `E-DEMO-002` (construction-time error, not a silent divergence).
- The `ScenarioEntityCatalog` derivation uses a secondary RNG stream (`seed + 1`).
  This is a convention that must be documented and must not conflict with the primary
  generator's RNG consumption. The secondary stream is safe because the CrowdStrike
  and Armis generators call `seeded_rng(seed, org_id)` (primary stream); the catalog
  derivation calls `seeded_rng(seed.wrapping_add(1), org_id)` (secondary stream) — these
  are independent `ChaCha20Rng` instances with different seeds, not sequential
  consumers of the same stream.

### Status as of v1.0 (2026-06-09)

ACCEPTED — not yet implemented. Governs S-DEMO-DTU-LIVE-SCENARIO-001 (T5 in
multi-client-soc-demo-tasks.md). BCs BC-2.06.019 and BC-2.06.020 are pending PO
authorship (T4 obligations). Implementation begins after PO authors both BCs and
story-writer assembles the single larger live-scenario story.

---

## 5. Alternatives Considered

- **Option A — Background mutation task.** Each clone runs a tokio task that advances
  a `Mutex<StageIndex>` on a timer. Rejected: non-reproducible across restarts,
  cross-clone synchronization hazard, concurrency complexity (see Rationale §3.1).

- **Option B — Per-request re-generation.** Call `generate(...)` on every inbound
  request with a time-parameterized `GenOpts` to produce a stage-appropriate fixture.
  Rejected: generation is not free (it builds `Vec<Value>`); calling it per-request
  under load would create latency spikes. More importantly, re-generation on every
  request conflicts with the `FixtureSet` being the stable reference for stage-mask
  filtering — if fields like `containment_status` are re-generated each request, they
  could differ between two simultaneous requests from the same analyst in the same
  stage (race on non-deterministic timestamp-based RNG seeding).

- **Option C — Poll-count-based staging.** Stage advances on the N-th poll, not on
  wall-clock time. Rejected: poll-count staging ties the progression to prism's polling
  frequency, which is an operational detail not visible to the demo operator. The
  demo operator needs to know "the exfil stage starts 6 minutes in" — wall-clock
  progression is the right abstraction for demo planning.

- **Option D — ThreatIntel/NVD generator.** Build a full generator for ThreatIntel
  and NVD analogous to the CrowdStrike generator. Rejected: the enrichment DTUs serve
  pure key-value lookups. A generator that produces thousands of random IOCs and CVEs
  adds specification and maintenance burden without improving demo believability. The
  demo analyst enriches the *specific* scenario IOCs, not a random sample. Lookup
  injection is sufficient.

- **Option E — Per-DTU independent entity derivation.** Each DTU generator derives
  entity IDs from the same `(seed, org_id)` independently. Rejected: divergence risk
  in RNG state consumption produces incoherent cross-DTU IDs (see Rationale §3.3).

---

## 6. New Error Codes

Two new `E-DEMO-NNN` codes required (to be registered in `.factory/specs/prd-supplements/error-taxonomy.md` by error-taxonomy owner):

| Code | Category | Trigger | Message |
|------|----------|---------|---------|
| `E-DEMO-002` | configuration | `scenario.enabled = true` for multiple clones with different `seed` values | `"demo-server: E-DEMO-002: scenario clones {A} (seed={X}) and {B} (seed={Y}) have different seeds; cross-DTU coherence requires all scenario-enabled clones to share the same seed"` |
| `E-DEMO-003` | configuration | `scenario.archetype` is an unrecognized string | `"demo-server: E-DEMO-003: clone '{name}': unrecognized scenario archetype '{value}'; valid values: compromised_endpoint, healthy"` |

---

## 7. BCs Flagged for PO Authorship (Not Authored Here)

These BCs are required before story-writer can assemble the single larger story.
The architect provides candidate titles and key invariants — PO authors the full BCs.

### Candidate BC-2.06.019 — Deterministic Scenario Progression (Live Demo)

**Candidate title:** "Demo-Server Scenario Progression — Pure-Function Temporal Stage Advancement with Reproducibility Guarantee"

**Key invariants the PO must specify:**
- `INV-PROGRESSION-REPRODUCIBILITY-001`: For any `(seed, scenario_start_epoch_secs, now_epoch_secs)` triple, `current_stage_index(timeline, now)` returns the same value across process restarts and independent invocations.
- `INV-STAGE-MONOTONICITY-001`: Stage index never decreases within a continuous process lifetime (elapsed time is monotonically non-decreasing; `Utc::now()` is only called once per request).
- `INV-STAGE-MASK-COMPLETENESS-001`: For each stage index, the `StageMask` covers all entity types in `ScenarioEntityCatalog`; no entity type is implicitly visible (explicit `false` for entities not yet surfaced).
- `INV-SCENARIO-DISABLED-COMPAT-001`: When `scenario.enabled = false` (or absent) for a clone, the clone's behavior is byte-identical to BC-2.06.018 postcondition 4 (static snapshot, seed=42 default).

### Candidate BC-2.06.020 — Cross-DTU Enrichment Correlation

**Candidate title:** "Demo-Server Enrichment Correlation — Scenario IOCs Resolve in ThreatIntel; Scenario CVEs Resolve in NVD"

**Key invariants the PO must specify:**
- `INV-THREATINTEL-IOC-CORRELATION-001`: For every `ip` in `ScenarioEntityCatalog.ioc_ips` and every `domain` in `ioc_domains` and every `hash` in `ioc_hashes`, `ThreatIntelClone` lookup returns `threat_is_known_malicious = true` and `threat_score >= 75`.
- `INV-NVD-CVE-CORRELATION-001`: For every `cve_id` in `ScenarioEntityCatalog.device_cves`, `NvdClone` lookup returns a `CveRecord` with `cvss_metric_v31[0].cvss_data.base_score >= 7.0`.
- `INV-CROSS-DTU-ENTITY-COHERENCE-001`: `ScenarioEntityCatalog.primary_device_id` appears as a device/host ID in at least one record returned by each of: Armis `/api/v1/devices`, CrowdStrike device query, Claroty device query — when the active stage is `Recon` or later.
- `INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001`: IOC lookups for values NOT in `ioc_ips / ioc_domains / ioc_hashes` return the default registry result (Benign, Unknown, or 404) — the scenario injection does not contaminate the general lookup table.

---

## 8. Story Scope Recommendation (for Story-Writer)

**Recommended story name:** `S-DEMO-DTU-LIVE-SCENARIO-001`

**Recommended scope (single story, one delivery gate):**
- BC-2.06.018 (baseline seeding — already authored, must be implemented)
- BC-2.06.019 (scenario progression — pending PO authorship)
- BC-2.06.020 (enrichment correlation — pending PO authorship)

**Point estimate reasoning:**

The story contains:
1. `prism-dtu-common/src/scenario/` module: `ScenarioEntityCatalog`, `IncidentTimeline`,
   `IncidentStage`, `StageMask`, `current_stage_index` pure function — estimated 150-200
   lines of pure-core Rust with unit tests.
2. `CloneConfig` extension: `ScenarioConfig` struct, TOML deserialization, defaults.
3. `build_clone_pairs` coordination: catalog derivation, `E-DEMO-002` / `E-DEMO-003`
   error paths, `Arc<IncidentTimeline>` threading to 4 generator-backed clones + entity
   catalog to 2 enrichment clones.
4. Per-DTU state extension: `Option<Arc<IncidentTimeline>>` in `ArmisState`,
   `CrowdstrikeState`, `ClarotyState`, `CyberintState`.
5. Per-DTU route projection: stage-mask filtering in devices + alerts + detections
   routes for all 4 generator-backed clones (8 routes modified).
6. ThreatIntel `new_with_scenario` constructor: 20-30 lines.
7. NVD `new_with_scenario` constructor + synthetic CVE record builder: 30-50 lines.
8. Integration test: 3 org x CompromisedEndpoint scenario, assert cross-DTU entity
   coherence at stage 2 (LateralMovement), and that IOCs resolve in ThreatIntel.
9. Backward-compat regression: seed=42 default path byte-identical (BC-2.06.018 PC-4).

This is substantive but tractable as a single story. Estimated complexity: **13 points**.
The user's directive is one story ("fold baseline + progression + enrichment correlation");
the architect does not recommend splitting. The entire scenario layer is tightly coupled
(entity catalog feeds 6 clones; splitting would require a sub-story that delivers an
incomplete catalog with no test path). If the story-writer independently assesses
this exceeds 13 points, the only defensible split is:

- Sub-story A: Baseline seeding (BC-2.06.018 only) — closes the existing gap in
  `build_clone_pairs`; 5 points.
- Sub-story B: Progression + enrichment (BC-2.06.019 + BC-2.06.020) — depends on A;
  8-10 points.

Surface this tradeoff to the user; the one-story preference is recorded.

---

## 9. Source / Origin

- **Decision D-1077** (2026-06-09) — user-directed scope expansion of multi-client SOC
  demo: scenario progression + enrichment correlation; recorded in
  `.factory/objectives/multi-client-soc-demo-tasks.md §Scope Expansion`.
- **ADR-009** — the generator architecture this extends; pure-function determinism and
  org-tagged ID conventions are inherited directly.
- **BC-2.06.018** — the baseline seeding BC this builds on.
- **Code as-built — generator statics model:**
  `crates/prism-dtu-crowdstrike/src/generator.rs:gen_compromised_endpoint` — confirms
  `device[0]` as always-contained and `alert[0..4]` as always-severity-4+. The stage
  mask approach filters these deterministic records rather than re-generating.
- **Code as-built — state struct pattern:**
  `crates/prism-dtu-armis/src/state.rs:ArmisState` — confirms the immutable fixture
  registry + mutable tag store pattern; `Option<Arc<IncidentTimeline>>` adds a third
  field without breaking existing constructors.
- **Code as-built — enrichment clone state:**
  `crates/prism-dtu-threatintel/src/state.rs:fixture_registry` — confirms a
  `Mutex<HashMap<String, FixtureKey>>` that can be pre-populated at construction time
  via `with_admin_token`; the scenario constructor takes the same approach.
- **Code as-built — demo-server harness:**
  `crates/prism-dtu-demo-server/src/harness.rs:build_clone_pairs` — confirms the
  sequential construction pattern and the `anyhow::Result<Vec<ClonePair>>` error
  propagation contract; scenario coordination logic adds ~60 lines before the first
  `if config.clones.crowdstrike.enabled` block.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-09 | architect | Initial authoring. Decision D-1077. |
