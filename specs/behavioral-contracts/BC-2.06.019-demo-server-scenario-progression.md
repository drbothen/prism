---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.019"
version: "1.8"
status: active
lifecycle_status: active
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-036"
introduced: "2026-06-09"
modified: "2026-06-19"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-DEMO-DTU-LIVE-SCENARIO-001, S-DEMO-DTU-LIVE-SCENARIO-001-B]
verifying_vps: []
crates: [prism-dtu-demo-server, prism-dtu-common, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty, prism-dtu-cyberint]
inputs:
  - "crates/prism-dtu-demo-server/src/config.rs"
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - "crates/prism-dtu-common/src/generator/archetype.rs"
  - "crates/prism-dtu-common/src/generator/rng.rs"
  - "crates/prism-dtu-armis/src/state.rs"
  - "crates/prism-dtu-crowdstrike/src/state.rs"
  - ".factory/specs/behavioral-contracts/BC-2.06.018-dtu-demo-clone-data-seeding.md"
  - ".factory/specs/behavioral-contracts/BC-3.4.001.md"
  - ".factory/specs/architecture/decisions/ADR-036-deterministic-scenario-progression-engine.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: ""
traces_to:
  - "CAP-036"
  - "ADR-036"
  - "BC-2.06.018"
  - "BC-3.4.001"
extracted_from: null
---

# BC-2.06.019: Demo-Server Scenario Progression — Pure-Function Temporal Stage Advancement with Reproducibility Guarantee

## Description

`build_clone_pairs` in `prism-dtu-demo-server` extends the config-time seeding layer
(BC-2.06.018) with an optional `[clones.*.scenario]` block in `demo.toml`. When
`scenario.enabled = true`, each generator-backed clone's state struct holds an
`Option<Arc<IncidentTimeline>>`. Route handlers compute `current_stage_index(timeline,
Utc::now().timestamp())` — a pure function of elapsed wall-clock seconds — and apply
the resulting `StageMask` to the pre-built `FixtureSet` to determine which entities
are visible in the response. This BC governs the **wiring layer** from `ScenarioConfig`
through `IncidentTimeline` construction to per-request stage projection; the underlying
generator determinism guarantee remains owned by BC-3.4.001.

## Preconditions

1. `DemoConfig` has been parsed from a valid TOML file (BC-2.06.001), including any
   `[clones.*.scenario]` blocks.
2. The `fixture-gen` Cargo feature is enabled in `prism-dtu-demo-server`'s feature set
   (prerequisite for generator-backed clone construction).
3. `prism-dtu-common/src/scenario/` module exists (behind `feature = "fixture-gen"`),
   providing `ScenarioEntityCatalog`, `IncidentTimeline`, `IncidentStage`, `StageMask`,
   and `current_stage_index(timeline: &IncidentTimeline, now_epoch_secs: i64) -> usize`.
4. For each clone with `scenario.enabled = true`, `CloneConfig.seed` and `CloneConfig.org_id`
   are available at construction time (same precondition as BC-2.06.018 §Precondition 4).
5. When multiple clones have `scenario.enabled = true`, all such clones in the same
   client config block share the same `seed` value. If any two scenario-enabled clones
   have different seeds, `build_clone_pairs` must return `E-DEMO-002` before any clone
   is constructed.
6. When multiple clones have `scenario.enabled = true`, all such clones in the same
   client config block share the same `org_id` value. If any two scenario-enabled clones
   have different `org_id` values, `build_clone_pairs` must return `E-DEMO-006` before
   any clone is constructed. Rationale: the `ScenarioEntityCatalog` is derived from the
   first scenario-enabled clone's `(seed, org_id)` via
   `seeded_rng(seed.wrapping_add(1), org_id)` (§Postcondition 1). A second clone with a
   different `org_id` generates device IDs using `dev-{slug_B}-{seed}-0` (its own
   org_slug), which cannot equal `catalog.primary_device_id` derived from slug_A. The
   cross-DTU join in INV-CROSS-DTU-ENTITY-COHERENCE-001 (BC-2.06.020) returns empty with
   no diagnostic — a SOUL.md §4 silent partial-failure. The guard prevents this silent
   incoherence at construction time.
   Guard order within `build_clone_pairs` pre-construction validation:
   `E-DEMO-002 (seed mismatch) → E-DEMO-006 (org_id mismatch) → E-DEMO-003 (bad archetype) → E-DEMO-004 (missing org_id)`
7. `scenario.archetype` for each scenario-enabled clone is one of the recognized strings
   (`"compromised_endpoint"`, `"healthy"`, etc.); unrecognized values cause `E-DEMO-003`.
8. `build_clone_pairs` is called before `DemoHarness::start_all`.

## Postconditions

### Postcondition 1 — `ScenarioEntityCatalog` is constructed once per client config from `(seed, org_id)`

When at least one clone in a client config block has `scenario.enabled = true`:

- `build_clone_pairs` derives a single `ScenarioEntityCatalog` using `(seed, org_id)`
  from the first scenario-enabled clone's config.
- The catalog is derived via a secondary RNG stream `seeded_rng(seed.wrapping_add(1), org_id)`
  — independent of the primary generator RNG stream `seeded_rng(seed, org_id)` used by
  BC-3.4.001. This separation ensures catalog derivation does not consume generator RNG
  state and produce shifted fixture data.
- The catalog fields populated are:
  - `primary_device_id`: `"dev-{org_slug}-{seed}-0"` (first device ID of `CompromisedEndpoint`
    generator output for this `(seed, org_id)` pair)
  - `primary_hostname`: consistent hostname derived from the same secondary RNG stream
  - `lateral_devices`: `Vec<String>` of additional device IDs for lateral movement stage
  - `ioc_ips`, `ioc_domains`, `ioc_hashes`: IOC indicators for Exfil stage, formatted
    with org-slug prefix (e.g., `"ioc-{org_slug}-{seed}-0.evil.example.com"`) to maintain
    cross-client disjointness
  - `device_cves`: `Vec<String>` of CVE IDs assigned to the primary device

### Postcondition 2 — `IncidentTimeline` wraps the catalog with ordered stage definitions

An `IncidentTimeline` is constructed from the `ScenarioEntityCatalog` and the
`CompromisedEndpoint` stage definitions (operator-overridable via `stage_duration_secs`
in `demo.toml`):

| Stage Index | Stage Name | `activates_after_secs` (default) | Entities Visible (`StageMask`) |
|-------------|------------|-----------------------------------|--------------------------------|
| 0 | `Baseline` | 0 | `primary_device=true`; all IOC/lateral/CVE fields `false` |
| 1 | `Recon` | 60 | `primary_device=true`; `lateral_devices=false`; IOC/CVE `false` |
| 2 | `LateralMovement` | 180 | `primary_device=true`; `lateral_devices=true`; `ioc_hashes=true`; IP/domain/CVE `false` |
| 3 | `Exfil` | 360 | All devices `true`; `ioc_ips=true`; `ioc_domains=true`; `ioc_hashes=true`; `device_cves=false` |
| 4 | `Containment` | 600 | All fields `true`; primary device shows `containment_status="contained"` |

The `StageMask` MUST be explicit for every `ScenarioEntityCatalog` field at every stage.
No entity type is implicitly visible — absent fields default to `false` per
INV-STAGE-MASK-COMPLETENESS-001.

**`stage_duration_secs` 4-entry convention (ADR-036 v2.0 §1.3, §2.1):**
The 5-stage timeline is specified by a **4-entry** array — NOT 5 entries. Stage 0 (Baseline)
always activates at elapsed=0 seconds and is not configurable; it has no entry in the array.
Entry `[i]` is the cumulative `activates_after_secs` threshold for stage `i+1`:

| Array index | Stage activated | Default `activates_after_secs` |
|-------------|----------------|-------------------------------|
| [0] | Stage 1 (Recon) | 60 |
| [1] | Stage 2 (LateralMovement) | 180 |
| [2] | Stage 3 (Exfil) | 360 |
| [3] | Stage 4 (Containment) | 600 |

Example: `stage_duration_secs = [60, 180, 360, 600]` (the default) means stage 1 activates
at elapsed ≥ 60s, stage 2 at ≥ 180s, stage 3 at ≥ 360s, stage 4 at ≥ 600s.

When `stage_duration_secs` is provided in `demo.toml`, it MUST have exactly **4 entries**
for the `CompromisedEndpoint` archetype. Providing any count other than 4 is an operator
error and produces `E-DEMO-003` (with the `stage_duration_secs has {provided} entries but
archetype '{archetype}' requires exactly {expected}` variant).

`scenario_start_epoch_secs` in the timeline is set from `CloneConfig.scenario_start_secs`
when present, or from `Utc::now().timestamp()` at `build_clone_pairs` call time when
`None` (i.e., the scenario starts at harness construction time, beginning at stage 0).

### Postcondition 3 — Stage index computation is a pure function of `(timeline, now_epoch_secs)`

The `current_stage_index` function:

```
elapsed_secs = max(0, now_epoch_secs - timeline.scenario_start_epoch_secs)
stage_index  = max stage S such that elapsed_secs >= stages[S].activates_after_secs
```

This function:
- Has no side effects (no writes, no shared state mutation, no spawned tasks).
- Is called once per inbound HTTP request in each route handler.
- Returns a `usize` in `[0, stages.len() - 1]`.
- Returns `0` (Baseline) when `elapsed_secs < stages[1].activates_after_secs`.

There is NO background mutator task, NO `Arc<AtomicU64>` stage counter, NO `Mutex<StageIndex>`
in any clone state struct. Each request independently computes the current stage.

### Postcondition 4 — Per-Sensor IOC-Surface Masking and Route-Level Stage Guards

Each generator-backed clone (Armis, CrowdStrike, Claroty, Cyberint) that is constructed
via the scenario path gains:

- An `Option<Arc<IncidentTimeline>>` field in its state struct (e.g., `ArmisState.timeline`,
  `CrowdstrikeState.timeline`). Constructed via the 5-arg form
  `<CloneType>::new_with_scenario(seed, archetype, org_id, Arc<IncidentTimeline>, time_anchor: DateTime<Utc>)`.
  Return types differ per clone: CrowdStrike and Claroty return `-> Self`; Armis, Cyberint, and NVD
  return `-> anyhow::Result<Self>` (ADR-036 v2.3 §2.4). The `time_anchor` is derived ONCE in
  `build_clone_pairs` (typically from `Utc::now()` or config-provided epoch) so all clones in the
  same client config share the same timestamp anchor for era-coherent generated fixtures.
- Route handlers implementing the scenario path:
  1. Acquire the `Arc<IncidentTimeline>` from state.
  2. Call `current_stage_index(&timeline, Utc::now().timestamp())` → `stage_idx`.
  3. Retrieve the `StageMask` for `stage_idx` from `timeline.stages[stage_idx].visible_entity_mask`.
  4. Filter the pre-built `FixtureSet` records using the mask: entities marked `false` in
     the mask are withheld from the response for the current request.
  5. The `FixtureSet` itself is generated ONCE at construction time (same as BC-2.06.018 §PC-1).
     No re-generation occurs per request. The stage mask is a filter over immutable data.

#### General Filtering Semantics (Non-IOC Entity Types)

- `primary_device=false`: the device with ID `catalog.primary_device_id` is excluded from
  `/api/v1/devices` (Armis), `/devices/v2` (CrowdStrike), and equivalent device endpoint responses.
- `lateral_devices=false`: devices with IDs in `catalog.lateral_devices` are excluded.
- `device_cves=false`: CVE-related enrichment fields on device records are omitted or set to `[]`.
- `primary_device=true` at `Containment` stage: `containment_status` field for the primary device
  is `"contained"` (the pre-built `FixtureSet` record already carries this value per the
  `CompromisedEndpoint` generator; the Containment stage makes it visible).

#### Per-Sensor IOC-Surface Matrix

The `StageMask.ioc_*` fields govern IOC visibility only for sensors whose real API surfaces
native IOC data. Which sensors carry IOC fields is determined by the per-sensor IOC-surface
matrix below, which is the authoritative enumeration for this BC and for the story spec of
S-DEMO-ENRICHMENT-PIVOT-003.

| Sensor | IOC-Surface | Real-Schema Field Path(s) | IOC Types Supported | Implementation Story | Fidelity Basis |
|--------|-------------|---------------------------|---------------------|---------------------|----------------|
| Cyberint alerts | YES (deferred) | `iocs[].value` (list — CONFIRMED); `alert_data.url` (CONFIRMED); `alert_data.ip`, `alert_data.domain` (UNCONFIRMED-plausible); `ioc.value` (singleton top-level — UNCONFIRMED, flagged for likely removal) | ip, domain, url, hash | S-DEMO-ENRICHMENT-PIVOT-003 | `iocs` array and `alert_data.url` confirmed via Check Point sk182975 + FortiSOAR connector (2026-06-19). Inner IOC key names for `iocs[]` elements are **INCONCLUSIVE-pending-live-validation**: the real Cyberint alerts API does not expose inner-element structure in any public doc. Two equally-plausible forms: `type`/`value` (short, prism's current bet) OR `ioc_type`/`ioc_value` (matching Cyberint feed convention per XSOAR Check Point EM Feed docs). DTU implementation MUST use serde dual-alias (`#[serde(rename = "type", alias = "ioc_type")]` and `#[serde(alias = "ioc_value")]`) to tolerate both forms until validated against a live tenant. Singleton top-level `ioc` field (prism's `Alert.ioc: Option<Ioc>`) has NO public-documentation basis; flagged for removal — confirm via live tenant whether it ever appears, remove if absent. Source: uncertainty-pivot003-s504-2026-06-19.md §Item 2. |
| CrowdStrike detections | YES (deferred) | `behaviors[].ioc_value` (on detection records, NOT on host/device records) | `hash_sha256`, `hash_md5`, `domain`, `filename`, `registry_key` — NOT `hash` (bare, incorrect), NOT `registry` (bare, incorrect), NOT `cmdline` (NOT an ioc_type; it is a separate sibling behavior field `behaviors[].cmdline`), NOT ipv4/ipv6 (IPs only appear on separate custom-IOC / device-query surfaces, not on detection `behaviors[].ioc_type`) | S-DEMO-ENRICHMENT-PIVOT-003 | Real FalconPy SDK / CrowdStrike Detect API `behaviors[]` array carries `ioc_type`/`ioc_value`/`ioc_source`/`ioc_description` per real-API research. Corrected ioc_type token values per 2026-06-19 research (uncertainty-pivot003-s504-2026-06-19.md §Item 1): ThreatQ CrowdStrike Insight EDR CDF + XSOAR CrowdStrike Falcon integration confirm algorithm-qualified tokens (`hash_sha256`, `hash_md5`) and `_key`-suffixed registry token (`registry_key`); `cmdline` confirmed as separate sibling field, never an ioc_type value. **Tolerant-unknown-type policy:** CrowdStrike publishes no normative exhaustive enum for `behaviors[].ioc_type`; DTU parser MUST treat unknown tokens as non-fatal (log + preserve raw string) rather than rejecting, to avoid breakage if undocumented or licence-gated types (e.g., URL/email) are encountered. |
| CrowdStrike devices (hosts) | NO | — | — | — | Host/device records do not carry IOC fields; IOCs live on detection records only |
| Armis alerts | NO (permanent) | — | — | — | Armis alert payloads are reference-only (deviceIds, activityUUIDs, endpoints). No structured IOC fields in the real Armis API. Fabricating IOC fields would violate the DTU=True-DTU fidelity principle (ADR-031). This exclusion is permanent — not a deferral. |
| Claroty xDome alerts | NO (permanent) | — | — | — | Claroty alerts carry IP addresses only as free text in `alert_name`; no structured IOC schema in the real API. Fabricating structured IOC fields would violate ADR-031. This exclusion is permanent — not a deferral. |

**IOC filtering semantics for sensors WITH IOC surface (Cyberint, CrowdStrike detections):**

- `ioc_hashes=false`: detection records where `behaviors[].ioc_value` matches a value in
  `catalog.ioc_hashes` AND `behaviors[].ioc_type` is one of `hash_sha256` or `hash_md5`
  (CrowdStrike), or alert records where `iocs[].value` (dual-alias: `value` or `ioc_value`)
  matches a hash-type IOC in `catalog.ioc_hashes` (Cyberint), are withheld from the response.
- `ioc_ips=false`: Cyberint alert records where `iocs[].value` (dual-alias), or `alert_data.ip`
  matches a value in `catalog.ioc_ips` are withheld. CrowdStrike detections: not applicable
  (CrowdStrike `behaviors[]` does not carry `ipv4`/`ipv6` IOC types on detection records).
- `ioc_domains=false`: Cyberint alert records where `iocs[].value` (dual-alias), or
  `alert_data.domain` matches a value in `catalog.ioc_domains` are withheld. CrowdStrike
  detections: `behaviors[].ioc_value` with `ioc_type = "domain"` matching `catalog.ioc_domains`
  are withheld.

**Sensors WITHOUT IOC surface (Armis, Claroty): IOC masking does not apply.** The `ioc_*`
StageMask fields are ignored for these sensors at all stages; they have no IOC-bearing records
to filter.

#### Interim State (Until S-DEMO-ENRICHMENT-PIVOT-003)

Until S-DEMO-ENRICHMENT-PIVOT-003 ships:

- The Cyberint alerts route (`crates/prism-dtu-cyberint/src/routes/alerts.rs`) contains a
  synthetic `_ioc_value` / `_ioc_type` filter that matches a non-real field injected only by
  injection tests. This filter is a forward-provision stub: it is exercised only by those
  injection tests and has no effect on real-schema alert records (which carry no `_ioc_value`
  field). It does NOT represent the real Cyberint IOC schema.
- CrowdStrike detections route (added in commit `bc0f36c5`) carries the `stage_idx > 0` guard
  (see Route Coverage Table below) but does NOT yet stamp `behaviors[].ioc_*` fields on
  fixture records. The IOC-field stamping is deferred to S-DEMO-ENRICHMENT-PIVOT-003.
- S-DEMO-ENRICHMENT-PIVOT-003 removes the `_ioc_value` synthetic filter atomically when it
  adds the real-schema `ioc` / `iocs[]` / `alert_data.*` fields to the Cyberint `Alert` struct.
  The synthetic filter and the real-schema filter MUST NOT coexist — the story removes the
  synthetic one in the same commit that adds the real one.

#### Route Coverage Table

Every DTU clone route that is governed by a `StageMask` field is enumerated here. This table is
the authoritative cross-reference between StageMask fields and their implementation sites.
**Standing rule:** any future story adding or modifying a StageMask-relevant route MUST extend
or update this table in the same commit. Failure to do so is a process-gap finding of severity
HIGH (consistent with BPRL-P4-01 root-cause: the detection route was not enumerated, causing
the production-inert filter to be undetected for one full review cycle).

| StageMask Field | DTU Clone | Route File | Route Path | Guard Mechanism | Status |
|-----------------|-----------|------------|------------|-----------------|--------|
| `primary_device`, `lateral_devices` | prism-dtu-armis | `routes/devices.rs` | `GET /api/v1/devices` | `stage_idx > 0` for primary; `mask.lateral_devices` for lateral | ACTIVE (pre-bc0f36c5, B-P1-01) |
| `primary_device`, `lateral_devices` | prism-dtu-armis | `routes/search.rs` | `GET /api/v1/search` | `stage_idx > 0` for primary; `mask.lateral_devices` for lateral (added commit bc0f36c5) | ACTIVE |
| `primary_device`, `lateral_devices` | prism-dtu-armis | `routes/alerts.rs` | `GET /api/v1/alerts` | `stage_idx > 0` for primary; `mask.lateral_devices` for lateral (added commit bc0f36c5) | ACTIVE |
| `primary_device`, `lateral_devices` | prism-dtu-crowdstrike | `routes/hosts.rs` | `GET /devices/queries/devices/v1` and `GET /devices/entities/devices/v2` | `stage_idx > 0` for primary; `mask.lateral_devices` for lateral | ACTIVE (pre-bc0f36c5, B-P1-01) |
| `ioc_hashes`, `ioc_ips`, `ioc_domains` | prism-dtu-cyberint | `routes/alerts.rs` | `GET /api/v1/alerts` (also registered for POST via same handler — confirmed routes/alerts.rs) | Synthetic `_ioc_value` filter (interim) → replaced by real-schema filter in S-DEMO-ENRICHMENT-PIVOT-003 | INTERIM — see §Interim State |
| `ioc_hashes` | prism-dtu-crowdstrike | `routes/detections.rs` | `GET /detects/queries/detects/v1` (list IDs) and `POST /detects/entities/summaries/GET/v1` (batch summaries — confirmed routes/mod.rs) | `stage_idx > 0` guard on both list and summary routes (added commit bc0f36c5); IOC-field stamping deferred to S-DEMO-ENRICHMENT-PIVOT-003 | STAGE-GUARD ACTIVE, IOC-STAMP DEFERRED |
| `primary_device`, `lateral_devices` | prism-dtu-claroty | `routes/devices.rs` | `POST /api/v1/devices` (confirmed clone.rs:246) | `mask.primary_device && stage_idx > 0` for primary (devices.rs ~line 291); `mask.lateral_devices` for lateral (devices.rs ~line 293); single handler `list_devices` covers full device list (no separate detail handler in this file) | ACTIVE (added this PR) |
| (no IOC surface) | prism-dtu-claroty | `routes/alerts.rs` | `POST /api/v1/alerts` (confirmed clone.rs) | EXEMPT — no structured IOC fields in real Claroty API; device_id not emitted on alert records; relation via separate endpoint | PERMANENT EXEMPT |

**Inventory verification (v1.6) — exhaustive StageMask handler scan:**

Union of files returned by `rg -l 'mask\.primary_device|mask\.lateral_devices|mask\.ioc_|mask\.device_cves|stage_idx' crates/prism-dtu-*/src/routes/` and `rg -l 'scenario_stage_ctx|with_stage_mask_projection|StageMask' crates/prism-dtu-*/src/routes/`:

| File | Handler(s) | Registered Route | Table Row Status |
|------|-----------|-----------------|-----------------|
| `prism-dtu-armis/src/routes/devices.rs` | `list_devices` | `GET /api/v1/devices` | Row 1 (pre-bc0f36c5) |
| `prism-dtu-armis/src/routes/search.rs` | search handler | `GET /api/v1/search` | Row 2 (v1.5 added) |
| `prism-dtu-armis/src/routes/alerts.rs` | `list_alerts` | `GET /api/v1/alerts` | Row 3 (v1.5 added) |
| `prism-dtu-crowdstrike/src/routes/hosts.rs` | hosts list + entities | `GET /devices/queries/devices/v1` and `GET /devices/entities/devices/v2` | Row 4 (pre-bc0f36c5) |
| `prism-dtu-crowdstrike/src/routes/detections.rs` | list IDs + batch summaries | `GET /detects/queries/detects/v1` and `POST /detects/entities/summaries/GET/v1` | Row 5 (v1.4 added) |
| `prism-dtu-cyberint/src/routes/alerts.rs` | `list_alerts` | `GET /api/v1/alerts` (also POST) | Row 6 (v1.4 added) |
| `prism-dtu-claroty/src/routes/devices.rs` | `list_devices` | `POST /api/v1/devices` | **Row 7 — added v1.6 (BPRL-P6-01)** |

All 7 files in the union are now enumerated. No other StageMask-consulting route files exist in the codebase at the time of this scan. The `prism-dtu-claroty/src/routes/alerts.rs` file does NOT appear in either grep set; it is PERMANENT EXEMPT solely on real-API grounds: the real Claroty xDome API emits no structured IOC fields on alert records and no `device_id` field, so no StageMask projection is applicable there. The Claroty alerts EXEMPT row remains correct and complete.

### Postcondition 5 — Operator `scenario_start_secs` synchronizes cross-DTU timelines

When the operator sets the same `scenario_start_secs` value in all clone blocks:

- Every clone in the client config computes `current_stage_index` using the same
  `scenario_start_epoch_secs` value. For any fixed `now`, all clones return the same
  stage index.
- Cross-DTU snapshot coherence: at any given real-world instant, all clones are
  in the same stage and present a consistent incident narrative.
- This is structural coherence — it follows from each clone independently calling the same
  pure function with the same inputs. No inter-clone communication or lock is required.

### Postcondition 6 — `scenario.enabled = false` path is byte-identical to BC-2.06.018

When `scenario.enabled = false` (or the `[clones.*.scenario]` block is absent) for a
given clone:

- That clone is constructed via the static seeding path (BC-2.06.018 §Postcondition 1/2).
- The clone's `timeline: Option<Arc<IncidentTimeline>>` field is `None`.
- All route handlers take the static snapshot code path: no stage computation, no mask
  filtering. The full `FixtureSet` is served unchanged.
- The behavior is byte-identical to BC-2.06.018 postcondition 4 (seed=42 default or any
  explicit seed with `scenario.enabled=false`).

## Invariants

### INV-PROGRESSION-REPRODUCIBILITY-001 — Stage Index is a Pure Function of `(seed, scenario_start_epoch_secs, now_epoch_secs)`

For any fixed triple `(seed, scenario_start_epoch_secs, now_epoch_secs)`:

```
current_stage_index(IncidentTimeline built from (seed, org_id, scenario_start), now) 
  = same usize value across:
    - independent process restarts
    - independent harness re-instantiations
    - parallel route handler invocations with the same now value
```

This invariant holds because `current_stage_index` is a pure function with no reads from
shared mutable state, no system entropy, and no side effects. It is the `scenario_start_epoch_secs`
field (captured at construction time from config or `Utc::now()`) that determines the stage
progression anchor — not any runtime-mutated counter.

References BC-3.4.001 (Generator Determinism): the `FixtureSet` records being filtered
are themselves reproducible, so the combination of reproducible records + reproducible stage
index → reproducible route handler responses.

### INV-STAGE-MONOTONICITY-001 — Stage Index Never Decreases Within a Process Lifetime

Within a single process lifetime, the wall-clock time returned by `Utc::now()` is
monotonically non-decreasing (system clock does not go backwards under normal operating
conditions). Because `current_stage_index` is a non-decreasing function of elapsed time:

```
∀ t1 ≤ t2: current_stage_index(timeline, t1) ≤ current_stage_index(timeline, t2)
```

This invariant means once a stage has become visible, it remains visible for all future
requests in the same process. Stages do not "un-advance."

**Note:** System clock skew (NTP correction, DST boundary) is not protected against.
The operator is responsible for a stable system clock during a live demo. This invariant
is documented, not enforced by a guard; the pure-function model is correct under
monotonic clocks.

### INV-STAGE-MASK-COMPLETENESS-001 — Every Stage Has an Explicit Mask for Every Entity Type

For each `IncidentStage` in `IncidentTimeline.stages`, the `StageMask` MUST explicitly
set every boolean field corresponding to a field in `ScenarioEntityCatalog`:

- `primary_device`
- `lateral_devices`
- `ioc_ips`
- `ioc_domains`
- `ioc_hashes`
- `device_cves`

No entity type is implicitly visible. A field that is not yet surfaced at a given stage is
`false`, not `None` or "missing." This invariant prevents accidental entity leakage through
uninitialized mask fields.

Verified at construction time: `build_clone_pairs` (or the `IncidentTimeline` constructor)
MUST assert that all six fields are set for each stage. If a new field is added to
`ScenarioEntityCatalog`, a corresponding field MUST be added to `StageMask` and explicitly
set in the default stage definitions before the code compiles (enforced by the non-exhaustive
struct pattern — `StageMask` does NOT carry `#[non_exhaustive]` since it is internal to
`prism-dtu-common` and must be exhaustively constructible within the crate).

### INV-SCENARIO-DISABLED-COMPAT-001 — `scenario.enabled = false` is Byte-Identical to BC-2.06.018 Static Path

When `scenario.enabled = false` or the `[clones.*.scenario]` block is absent:

- The clone is constructed with the identical code path as pre-ADR-036.
- No `IncidentTimeline` is constructed or passed to the clone constructor.
- The clone state struct has `timeline: None`.
- All HTTP response bodies for fixed request paths with fixed `(seed, fixture_set, org_id)`
  are byte-identical to the pre-ADR-036 responses for the same inputs.

This invariant is validated by the backward-compatibility regression test (TV-019-007).

### INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 — Catalog Derivation Uses Distinct RNG Stream

The `ScenarioEntityCatalog` is derived via `seeded_rng(seed.wrapping_add(1), org_id)`.
This is an independent `ChaCha20Rng` instance from the primary stream `seeded_rng(seed, org_id)`
used by the `CompromisedEndpoint` generator. The two streams consume independent seed
state and never share RNG output.

This invariant ensures that adding scenario-catalog derivation to a harness construction
does not shift the RNG output of the primary generator, which would invalidate
BC-2.06.018 §Postcondition 4 (backward compat at seed=42).

## Error Codes

### E-DEMO-002 — Scenario-Enabled Clones Have Mismatched Seeds

When `build_clone_pairs` detects that two or more scenario-enabled clones in the same client
config block have different `seed` values:

| Field | Value |
|-------|-------|
| Code | `E-DEMO-002` |
| Category | configuration |
| Severity | broken |
| Exit code | `1` (startup failure) |
| Message format | `"demo-server: E-DEMO-002: scenario clones '{clone_a}' (seed={seed_a}) and '{clone_b}' (seed={seed_b}) have different seeds; cross-DTU coherence requires all scenario-enabled clones to share the same seed"` |
| Recoverable | No — operator must fix `demo.toml` to use the same `seed` for all scenario-enabled clones under the same client config block |

`{clone_a}` and `{clone_b}` are the clone type names (e.g., `"crowdstrike"`, `"armis"`) of
the first mismatched pair found. `{seed_a}` and `{seed_b}` are the conflicting seed values.

Detection occurs before any clone constructor is called. If `E-DEMO-002` is triggered,
`build_clone_pairs` returns `Err(...)` immediately and no clones are constructed for that
client. Cross-DTU coherence requires all scenario-enabled clones to produce the same
`ScenarioEntityCatalog` (same entity IDs), which requires a single shared seed.

### E-DEMO-006 — Scenario-Enabled Clones Have Mismatched org_ids

When `build_clone_pairs` detects that two or more scenario-enabled clones in the same client
config block have different `org_id` values:

| Field | Value |
|-------|-------|
| Code | `E-DEMO-006` |
| Category | configuration |
| Severity | broken |
| Exit code | `1` (startup failure) |
| Message format | `"demo-server: E-DEMO-006: scenario clones '{clone_a}' (org_id={org_id_a}) and '{clone_b}' (org_id={org_id_b}) have different org_ids; cross-DTU coherence requires all scenario-enabled clones to share the same org_id"` |
| Recoverable | No — operator must fix `demo.toml` to use the same `org_id` for all scenario-enabled clones under the same client config block |

`{clone_a}` and `{clone_b}` are the clone type names of the first mismatched pair found.
`{org_id_a}` and `{org_id_b}` are the conflicting org_id UUID strings from `demo.toml`
(safe to echo: configuration values, not credentials per AD-017).

Detection occurs after the E-DEMO-002 seed-mismatch check and before the E-DEMO-003
archetype check. If `E-DEMO-006` is triggered, `build_clone_pairs` returns `Err(...)`
immediately and no clones are constructed for that client.

Rationale: the `ScenarioEntityCatalog` is derived from the first scenario-enabled clone's
`(seed, org_id)` pair. If a second clone has a different `org_id`, its generator produces
device IDs with `org_slug_B = hex(org_id_B.as_bytes()[0..4])` while the catalog's
`primary_device_id` uses `org_slug_A`. The cross-DTU join defined in
INV-CROSS-DTU-ENTITY-COHERENCE-001 (BC-2.06.020) returns empty with no diagnostic —
a silent partial-failure (SOUL.md §4). This guard surfaces the misconfiguration at
construction time instead.

### E-DEMO-003 — Unrecognized Scenario Archetype

When `scenario.archetype` is a string not in the recognized set:

| Field | Value |
|-------|-------|
| Code | `E-DEMO-003` |
| Category | configuration |
| Severity | broken |
| Exit code | `1` (startup failure) |
| Message format | `"demo-server: E-DEMO-003: clone '{clone_name}': unrecognized scenario archetype '{value}'; valid values: compromised_endpoint, healthy"` |
| Recoverable | No — operator must fix `demo.toml` |

Also applies when `stage_duration_secs` has a different length than the number of stages
for the named archetype. In that case the message variant is:
`"demo-server: E-DEMO-003: clone '{clone_name}': stage_duration_secs has {provided} entries but archetype '{archetype}' requires exactly {expected}"`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-019-001 | `scenario.enabled = false` for all clones | BC-2.06.018 behavior preserved byte-for-byte; no `IncidentTimeline` constructed (INV-SCENARIO-DISABLED-COMPAT-001) |
| EC-019-002 | `scenario.enabled = true` for CrowdStrike; `scenario.enabled = false` for Armis | CrowdStrike uses scenario projection; Armis uses static snapshot; both serve from independent `FixtureSet`s; no cross-clone interference |
| EC-019-003 | `now_epoch_secs < scenario_start_epoch_secs` (clock skew or start in the future) | `elapsed = max(0, ...)` → `elapsed = 0` → stage 0 (Baseline); no negative-elapsed panic |
| EC-019-004 | `now_epoch_secs` far past last stage threshold (e.g., elapsed >> 600s) | Stage index saturates at `stages.len() - 1` (Containment); all entities visible; no index-out-of-bounds |
| EC-019-005 | Two scenario-enabled clones with `seed_A ≠ seed_B` in same client config | `build_clone_pairs` returns `E-DEMO-002`; no clone constructed |
| EC-019-006 | `stage_duration_secs = [30, 60, 120, 240]` (operator override; 4 entries for 5-stage timeline) | VALID — 4 entries is the correct count for `CompromisedEndpoint` (stage 0 always at 0s needs no entry; entries [0..3] specify thresholds for stages 1-4). Construction succeeds with stage 1 activating at ≥30s, stage 2 at ≥60s, stage 3 at ≥120s, stage 4 at ≥240s. |
| EC-019-006b | `stage_duration_secs = [30, 60, 120]` (only 3 entries for `compromised_endpoint` archetype that requires 4) | `E-DEMO-003` variant: `"stage_duration_secs has 3 entries but archetype 'compromised_endpoint' requires exactly 4"` |
| EC-019-006c | `stage_duration_secs = [30, 60, 120, 240, 480]` (5 entries for `compromised_endpoint` archetype that requires 4) | `E-DEMO-003` variant: `"stage_duration_secs has 5 entries but archetype 'compromised_endpoint' requires exactly 4"` |
| EC-019-007 | `scenario_start_secs = None` in config | `scenario_start_epoch_secs` set to `Utc::now().timestamp()` at `build_clone_pairs` call time; demo begins at stage 0 immediately |
| EC-019-008 | `scenario_start_secs` set to same value for CrowdStrike and Armis in demo.toml | Both clones compute identical `stage_index` for any fixed `now`; cross-DTU snapshot coherent |
| EC-019-009 | `scenario_start_secs` set to a past epoch (demo "already in progress" at startup) | Correct behavior: elapsed already positive at startup; stage index may start at Recon or LateralMovement; operator uses this to start a demo mid-scenario |
| EC-019-010 | Process restart with same `demo.toml` and same `scenario_start_secs` | INV-PROGRESSION-REPRODUCIBILITY-001: same `now` produces same stage; responses byte-identical across restarts |
| EC-019-011 | Route handler called concurrently from multiple async tasks | `current_stage_index` is a pure function with no shared mutable state; concurrent calls are safe with no locking required |
| EC-019-012 | `scenario.enabled = true` with `fixture_set = "dormant"` (DormantTenant archetype) | Archetype mismatch: the default 5-stage timeline is for `CompromisedEndpoint`; `E-DEMO-003` if archetype doesn't support scenario progression; or if archetype IS `compromised_endpoint` but `fixture_set` maps to `DormantTenant`, `build_clone_pairs` detects the contradiction and returns `E-DEMO-003` |
| EC-019-013 | Two scenario-enabled clones with same `seed` but different `org_ids` (e.g., CrowdStrike `org_id="<uuid-A>"`, Armis `org_id="<uuid-B>"`) | `build_clone_pairs` returns `E-DEMO-006` before any clone is constructed. Silent path (no guard): catalog uses slug_A; Armis generator produces `dev-{slug_B}-{seed}-0`; cross-DTU join returns empty. The guard prevents this silent incoherence (PRE-6, INV-CROSS-DTU-ENTITY-COHERENCE-001 in BC-2.06.020). |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-019-001 | `scenario_start_secs = T`, `now = T + 30s` | `current_stage_index = 0` (Baseline; elapsed 30s < Recon threshold 60s) | happy-path |
| TV-019-002 | `scenario_start_secs = T`, `now = T + 90s` | `current_stage_index = 1` (Recon; elapsed 90s ≥ 60s, < 180s) | happy-path |
| TV-019-003 | `scenario_start_secs = T`, `now = T + 200s` | `current_stage_index = 2` (LateralMovement; elapsed 200s ≥ 180s, < 360s) | happy-path |
| TV-019-004 | `scenario_start_secs = T`, `now = T + 400s` | `current_stage_index = 3` (Exfil; elapsed 400s ≥ 360s, < 600s) | happy-path |
| TV-019-005 | `scenario_start_secs = T`, `now = T + 700s` | `current_stage_index = 4` (Containment; elapsed ≥ 600s) | happy-path |
| TV-019-006 | `scenario_start_secs = T`, `now = T - 30s` (clock skew) | `current_stage_index = 0` (elapsed clamped to 0; no panic) | edge-case |
| TV-019-007 | Clone constructed with `scenario.enabled = false`, `seed = 42` | Response body byte-identical to pre-ADR-036 `new_with_seed(42, HealthyOtEnvironment, default_org)` response (INV-SCENARIO-DISABLED-COMPAT-001) | regression |
| TV-019-008 | Two harness instantiations with same `(seed, org_id, scenario_start_secs)`, queried at `now = T + 200s` | Both return identical response bodies (INV-PROGRESSION-REPRODUCIBILITY-001) | reproducibility |
| TV-019-009 | Stage 0 (Baseline): GET `/api/v1/devices` from Armis | Response does NOT contain `catalog.primary_device_id` (`primary_device=false` at stage 0); response may be empty or contain non-scenario devices | happy-path |
| TV-019-010 | Stage 1 (Recon): GET `/api/v1/devices` from Armis | Response CONTAINS `catalog.primary_device_id` (`primary_device=true` at stage 1); lateral devices NOT present | happy-path |
| TV-019-011 | Stage 4 (Containment): GET CrowdStrike device detail for `primary_device_id` | Device record has `containment_status = "contained"` (StageMask exposes the pre-built containment value) | happy-path |
| TV-019-012 | `seed_A = 100` for crowdstrike, `seed_B = 200` for armis, both `scenario.enabled = true` | `build_clone_pairs` returns `Err` containing `E-DEMO-002` | error-path |
| TV-019-013 | `scenario.archetype = "unknown_value"` | `build_clone_pairs` returns `Err` containing `E-DEMO-003` | error-path |
| TV-019-014 | 3 concurrent HTTP requests to same route at same `now = T + 200s` | All 3 responses byte-identical (pure function, no lock contention) | concurrency |
| TV-019-015 | `seed_A = 100` for crowdstrike (`org_id="<uuid-A>"`), `seed_A = 100` for armis (`org_id="<uuid-B>"` where uuid-B ≠ uuid-A), both `scenario.enabled = true` | `build_clone_pairs` returns `Err` containing `E-DEMO-006` and both clone names and org_id values; no clone constructed (PRE-6, EC-019-013) | error-path |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-019-A | `current_stage_index` is a pure function: same `(timeline, now)` → same result, across restarts | integration test (TV-019-008) |
| VP-019-B | Stage index is monotonically non-decreasing over increasing `now_epoch_secs` | unit test: assert `∀ t1 ≤ t2: stage(t1) ≤ stage(t2)` over all threshold boundaries |
| VP-019-C | All 6 `StageMask` fields are explicitly set for every stage in default `CompromisedEndpoint` timeline | compile-time assertion or unit test of stage construction (INV-STAGE-MASK-COMPLETENESS-001) |
| VP-019-D | `scenario.enabled = false` produces byte-identical responses to BC-2.06.018 §PC-4 | regression test (TV-019-007) |
| VP-019-E | E-DEMO-002 fires when two scenario-enabled clones have mismatched seeds | unit test (TV-019-012) |
| VP-019-F | Concurrent requests at same `now` produce identical responses (no lock contention, no shared mutable state) | concurrency test (TV-019-014) |
| VP-019-G | Entity visibility at stage boundaries: primary device not present at stage 0, present at stage 1+ | integration test (TV-019-009, TV-019-010) |
| VP-019-H | Containment status visible at stage 4 only | integration test (TV-019-011) |
| VP-019-I | E-DEMO-006 fires when two scenario-enabled clones have same seed but different org_ids; no clone constructed | unit test (TV-019-015) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-036 ("Multi-Tenant DTU Test Harness (Internal)") per capabilities.md §CAP-036 |
| Capability Anchor Justification | CAP-036 ("Multi-Tenant DTU Test Harness (Internal)") per capabilities.md §CAP-036 — this BC specifies the scenario-progression wiring in `build_clone_pairs` and the per-DTU projection contract, which are demo-server harness-layer behaviors for orchestrating per-customer clone instances. The temporal staging layer (IncidentTimeline, StageMask, current_stage_index) is harness infrastructure that extends BC-2.06.018's per-clone seeding into temporal projection. This is CAP-036 scope (harness orchestration and per-org fixture data) and not CAP-039 scope (generator internals: the generator runs once at construction time; this BC governs how its output is filtered per-request by the harness wiring layer). |
| L2 Domain Invariants | N/A (demo-server scenario wiring; no DI-NNN in L2 domain spec maps to this concern) |
| Architecture Module | SS-01 (Sensor Adapters) per ARCH-INDEX.md; `prism-dtu-demo-server` and `prism-dtu-common` are the primary implementation sites |
| Governing ADR | ADR-036 ("Deterministic Scenario-Progression Engine") — this BC encodes ADR-036 §2.1, §2.2, §2.3, §2.5 as testable contracts |
| Stories | S-DEMO-DTU-LIVE-SCENARIO-001, S-DEMO-DTU-LIVE-SCENARIO-001-B |
| Upstream BCs | BC-2.06.018 (Config-Time Data Seeding — baseline this BC extends); BC-3.4.001 (Generator Determinism — the FixtureSet being filtered is guaranteed reproducible by BC-3.4.001) |

## Related BCs

- BC-2.06.018 — extends (this BC adds temporal staging to BC-2.06.018's config-time seeding; BC-2.06.018 §Postcondition 4 is the backward-compat path referenced by INV-SCENARIO-DISABLED-COMPAT-001)
- BC-2.06.020 — composes with (BC-2.06.020 specifies ThreatIntel/NVD enrichment correlation; the `ScenarioEntityCatalog` produced by this BC's Postcondition 1 is the input to BC-2.06.020's lookup injection)
- BC-3.4.001 — depends on (generator determinism: the `FixtureSet` filtered by stage mask is guaranteed reproducible by BC-3.4.001)
- BC-3.4.003 — referenced by (archetype catalog and baseline entity counts referenced in test vectors)

## Architecture Anchors

- `crates/prism-dtu-common/src/scenario/` — new module housing `ScenarioEntityCatalog`, `IncidentTimeline`, `IncidentStage`, `StageMask`, `current_stage_index` (ADR-036 §2.2)
- `crates/prism-dtu-demo-server/src/harness.rs` — `build_clone_pairs`: site of scenario coordination logic (catalog derivation, E-DEMO-002 guard, `Arc<IncidentTimeline>` threading to 4 generator-backed clones)
- `crates/prism-dtu-demo-server/src/config.rs` — `ScenarioConfig` struct + `CloneConfig.scenario: Option<ScenarioConfig>` extension (ADR-036 §2.4)
- `crates/prism-dtu-armis/src/state.rs` — `ArmisState.timeline: Option<Arc<IncidentTimeline>>`
- `crates/prism-dtu-crowdstrike/src/state.rs` — `CrowdstrikeState.timeline: Option<Arc<IncidentTimeline>>`
- `crates/prism-dtu-common/src/generator/rng.rs` — `seeded_rng(seed, org_id)` — primary stream; `seeded_rng(seed.wrapping_add(1), org_id)` — secondary stream for catalog derivation

## Story Anchor

S-DEMO-DTU-LIVE-SCENARIO-001, S-DEMO-DTU-LIVE-SCENARIO-001-B

## VP Anchors

VP-019-A through VP-019-I (above) — verified by integration/unit tests in S-DEMO-DTU-LIVE-SCENARIO-001 (original scenario-progression delivery) and S-DEMO-DTU-LIVE-SCENARIO-001-B (org_id-equality guard VP-019-I + all AC implementations)

## BC Changelog

| Version | Change |
|---------|--------|
| v1.8 | PO fidelity correction 2026-06-19 — True-DTU ioc_type enum correction (ADR-031) for CrowdStrike and Cyberint rows in Per-Sensor IOC-Surface Matrix. **CrowdStrike:** `behaviors[].ioc_type` value set corrected from `{hash, domain, filename, registry, cmdline}` to `{hash_sha256, hash_md5, domain, filename, registry_key}`. Bare `hash` → split into algorithm-qualified `hash_sha256` + `hash_md5`. Bare `registry` → `registry_key`. `cmdline` removed from the ioc_type value set — it is a SEPARATE sibling behavior field (`behaviors[].cmdline`), never an `ioc_type` value. `domain`/`filename` unchanged. ipv4/ipv6 exclusion retained (confirmed correct). Added tolerant-unknown-type policy: CrowdStrike publishes no normative exhaustive enum; parser must treat unknown `ioc_type` tokens as non-fatal. **Cyberint:** Inner IOC key names for `iocs[]` elements marked INCONCLUSIVE-pending-live-validation. DTU implementation MUST use serde dual-alias (`type`/`value` AND `ioc_type`/`ioc_value`) rather than a single hard-coded guess. Singleton top-level `ioc` field flagged for likely removal (no public-documentation basis found). `iocs[]` array and `alert_data.url` confirmed; `alert_data.ip`/`domain` remain plausible-unconfirmed. IOC filtering semantics updated to use dual-alias reference for Cyberint `iocs[].value`. Source: uncertainty-pivot003-s504-2026-06-19.md §Item 1 (HIGH confidence) + §Item 2 (INCONCLUSIVE). |
| v1.7 | PO micro-fix 2026-06-12 (BPRL-P7-01) — Inventory verification note prose corrected. Replaced fabricated claim that `prism-dtu-claroty/src/routes/alerts.rs` "appears in both grep sets due to `scenario_stage_ctx` or similar context references" with the accurate statement: the file does NOT appear in either grep set; its PERMANENT EXEMPT table row stands solely on real-API grounds (no structured IOC fields in the real Claroty xDome API per 2026-06-12 research; no `device_id` emitted on alert records). No table change, no semantic change. |
| v1.6 | PO fix-burst 2026-06-12 (BPRL-P6-01) — Route Coverage Table corrected: (1) ADDED missing Claroty devices row: `primary_device, lateral_devices / prism-dtu-claroty / routes/devices.rs / POST /api/v1/devices (clone.rs:246) / mask.primary_device && stage_idx > 0 for primary; mask.lateral_devices for lateral / ACTIVE (added this PR)`. Guard verified at devices.rs ~lines 291/293 (`list_devices` handler); route confirmed at clone.rs:246. (2) ADDED exhaustive inventory verification note under table: all 7 StageMask-consulting route files in the `prism-dtu-*` workspace enumerated with handler↔route↔row mapping, providing mechanical re-verification baseline for future passes. No further omissions found beyond the Claroty devices row. |
| v1.5 | PO fix-burst 2026-06-12 (BPRL-P5-01) — Route Coverage Table corrected to match commit-verified router ground truth. (1) DELETED phantom row `prism-dtu-crowdstrike / routes/alerts_search.rs / GET /alerts/queries/alerts/v2 (in:alerts branch)` — no such file or route exists in prism-dtu-crowdstrike (routes/: mod.rs, oauth.rs, writes.rs, hosts.rs, detections.rs); "in:alerts" is Armis AQL terminology mis-attributed to CrowdStrike. (2) CORRECTED crowdstrike detections summary row: route corrected from `GET /detects/entities/summaries/v1` (wrong method + path) to `POST /detects/entities/summaries/GET/v1` (confirmed routes/mod.rs). (3) ADDED missing Armis search row: `routes/search.rs / GET /api/v1/search / stage_idx > 0 + mask.lateral_devices guard` — StageMask-relevant route guarded in commit bc0f36c5 but never enumerated (exact POL-33 omission class). (4) TIGHTENED lateral wording: all rows now state `mask.lateral_devices` (the actual guard mechanism) instead of `stage_idx >= 2` (behaviorally equivalent but imprecise). (5) Corrected CrowdStrike device rows to accurate route paths: `GET /devices/queries/devices/v1` and `GET /devices/entities/devices/v2` (confirmed routes/mod.rs + hosts.rs). (6) Corrected Claroty alerts path from `GET /xdome/api/v1/alerts` to `POST /api/v1/alerts` (confirmed clone.rs). (7) PC-4 prose updated: 4-arg `new_with_scenario(seed, archetype, org_id, Arc<IncidentTimeline>)` → 5-arg form `(seed, archetype, org_id, Arc<IncidentTimeline>, time_anchor: DateTime<Utc>)` per ADR-036 v2.3; per-clone return type split noted (CrowdStrike `-> Self`; Armis/Cyberint/NVD `-> anyhow::Result<Self>`) per code-verified signatures in clone.rs. |
| v1.4 | PO burst 2026-06-12 (D-1109, WO-D1109) — PC-4 redesigned from a single blanket IOC-on-alert clause to a full per-sensor IOC-surface matrix. Root cause: BPRL-P4-01 (MED) — BC-2.06.019 v1.3 PC-4's IOC filter was production-inert because no DTU generator stamped IOC fields using real API schema field names; the Cyberint `_ioc_value` filter matched only a synthetic injected field. Resolution: (1) Replaced blanket clause with the Per-Sensor IOC-Surface Matrix (research-agent 2026-06-12: falconpy SDK, Cyberint portal docs, ThreatQ/XSOAR/Elastic field maps); Cyberint alerts (real fields: `ioc.value`, `iocs[].value`, `alert_data.ip/domain/url`) and CrowdStrike detections (`behaviors[].ioc_value`; hash/domain/filename/registry/cmdline; NOT ipv4/ipv6) carry IOC data per real API; Armis and Claroty permanently excluded on ADR-031 fidelity grounds. (2) Added Interim State clause: `_ioc_value` synthetic filter acknowledged as forward-provision stub; removed atomically in S-DEMO-ENRICHMENT-PIVOT-003 when real-schema fields land. (3) Added Route Coverage Table (BPRL-P4-02 process-gap codification, per WO-D1109 §Question 4): enumerates every StageMask-relevant DTU route × guard mechanism × status; standing rule added requiring table update in same commit as any future StageMask-relevant route addition. BPRL-P4-02 fix (commit bc0f36c5) codified: stage-guards on crowdstrike detections list/summary routes and armis alerts/search in:alerts branch now appear in the table. |
| v1.3 | PO micro-burst 2026-06-12 — B-P5-01 precondition renumbering correction. The v1.2 insertion of PRE-6 (org_id equality guard) displaced the archetype and build-before-start preconditions but the body labels were incorrectly written as PRE-8 and PRE-9, skipping PRE-7. Corrected: archetype precondition → PRE-7 (was mislabeled 8), build-before-start → PRE-8 (was mislabeled 9). The v1.2 changelog claim "Renumbered former PRE-6→PRE-8 … PRE-7→PRE-9" is annotated `[corrected at v1.3]` below. No semantic change to any precondition. B-P5-02 (taxonomy half): error-taxonomy.md E-DEMO-003 row updated from "§Precondition 6" to "§Precondition 7" (archetype is PRE-7 post-renumber). |
| v1.2 | PO micro-burst 2026-06-12 — OBS-1 org_id-equality gap closed. Added PRE-6 (org_id equality guard across scenario-enabled clones; E-DEMO-006; detection before E-DEMO-003; rationale: silent INV-CROSS-DTU-ENTITY-COHERENCE-001 incoherence is SOUL.md §4 class). Renumbered former PRE-6→PRE-8 (archetype) and PRE-7→PRE-9 (build_clone_pairs before start_all) [corrected at v1.3: body labels were written as 8 and 9, skipping 7; correct labels are PRE-7 (archetype) and PRE-8 (build-before-start)]. Added E-DEMO-006 error code section with full format table. Added EC-019-013 (org_id mismatch edge case). Added TV-019-015 (E-DEMO-006 test vector). Added VP-019-I (E-DEMO-006 unit test). OBS-2 anchor drift fixed: Stories traceability row, Story Anchor section, and VP Anchors section updated to include S-DEMO-DTU-LIVE-SCENARIO-001-B. Guard order in PRE-6: E-DEMO-002 → E-DEMO-006 → E-DEMO-003 → E-DEMO-004. |
| v1.1 | ADR-036 v2.0 / D-1078 substrate-reconciliation corrections. Pinned `stage_duration_secs` 4-entry convention (ADR-036 v2.0 §1.3, §2.1): added explicit 4-entry table showing array-index-to-stage mapping in §Postcondition 2; stage 0 (Baseline) always activates at 0s and is NOT represented in the array. Corrected EC-019-006 which incorrectly described 4 entries as an E-DEMO-003 error — 4 entries IS the correct count for `CompromisedEndpoint`; added EC-019-006b (3 entries → error) and EC-019-006c (5 entries → error) to document the actual error cases. Confirmed `activates_after_secs: u64` as the authoritative `IncidentStage` field name (NOT "duration") per ADR-036 v2.0 §2.2. All stage threshold values (60/180/360/600) unchanged. lifecycle_status remains draft. Invariant semantics unchanged. |
| v1.0 | Initial authoring. ADR-036 ACCEPTED 2026-06-09. BC-2.06.019 namespace confirmed (next-available after BC-2.06.018). Subsystem: SS-01 (prism-dtu-demo-server, prism-dtu-common). Capability: CAP-036 — harness wiring layer (temporal staging extends the harness orchestration that BC-2.06.018 began). Error codes E-DEMO-002 and E-DEMO-003 registered here (to be reflected in error-taxonomy.md by error-taxonomy owner in same burst per BC-2.06.018 precedent). Invariant naming follows ADR-036 §7 candidate invariant IDs verbatim with BC-house-style expansion. |
