---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.019"
version: "1.18"
status: active
lifecycle_status: active
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-036"
introduced: "2026-06-09"
modified: "2026-07-10T00:00:00Z"
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
input-hash: "d02f16a"
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
  6. **Fail-closed projection-integrity (MED-005):** Any alert-surface record that cannot be
     deserialized as the expected `Alert` type MUST be withheld from the response — not passed
     through. A deserialization failure means the StageMask cannot be correctly applied (IOC field
     values cannot be extracted for comparison against `catalog.ioc_*`), so surfacing an
     undeserializable record would violate the IOC masking guarantee. The real-schema filter that
     replaces the `_ioc_value` synthetic filter in S-DEMO-ENRICHMENT-PIVOT-003 MUST implement
     `Err(_) => { /* withhold */ }`, not `Err(_) => pass_through`.

#### General Filtering Semantics (Non-IOC Entity Types)

- `primary_device=false`: the device with ID `catalog.primary_device_id` is excluded from
  `/api/v1/devices` (Armis), `/devices/v2` (CrowdStrike), and equivalent device endpoint responses.
- `lateral_devices=false`: devices with IDs in `catalog.lateral_devices` are excluded.
- `device_cves=false`: CVE-related enrichment fields on device records are omitted or set to `[]`.
  **The `device_cves` array field is NEVER stamped on generated records and is NOT declared as a
  TOML column (Ruling 1b, U17 from remove-uncertainty pass 2026-06-12).** Only the scalar
  projection `device_cves_first` (String — first CVE ID from `catalog.device_cves`) is surfaced
  on device records. The canonical NVD pivot query MUST use `has device_cves_first` as its
  existence filter, never `has device_cves`:
  ```
  from armis.devices
  | where has device_cves_first
  | enrich cvss_base_score(device_cves_first)
  | where cvss_base_score >= 7.0
  | sort cvss_base_score desc
  ```
  The CVSS filter MUST use `cvss_base_score >= 7.0` (canonical column name per
  `infusions.md` §NVD infusion spec + ADR-040 §3.2; NOT `nvd_cvss_score`, NOT `>` with strict
  inequality — the canonical form is `>=` with `cvss_base_score`). The enrichment UDF name
  MUST be `cvss_base_score` (registered per `nvd.infusion.toml` `[[infusion.fields]]` name field;
  NOT `nvd(...)` which is the infusion_id, not the per-field UDF name). Any reference to
  `has device_cves` as a PrismQL filter, `nvd_cvss_score` as a CVSS column name, or
  `| enrich nvd(...)` as the enrichment operator form, is stale and MUST be treated as a
  P1 finding in adversarial review.
  This query returns non-empty results only at stage 4 (Containment), when `device_cves=true`
  in the StageMask causes `device_cves_first` to be present on device records.
  **Authoritative guarded route for `armis.devices`:** the sensor-spec `path_template` for the
  `armis.devices` table targets `GET /api/v1/search?aql=...` (`src/routes/search.rs`, device
  branch). The `device_cves` masking guard applies on whichever route a table's `path_template`
  targets — for `armis.devices` that is `/api/v1/search`. The `paginate_devices` path
  (`GET /api/v1/devices`, `src/routes/devices.rs`) is a secondary route that also carries the
  guard (both are guarded); it is NOT the authoritative route for the `armis.devices` table.
  See Route Coverage Table Rows 8 (devices.rs) and 11 (search.rs).
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
| Cyberint alerts | YES (deferred) | **ENRICH-1 clean column names (S-DEMO-ENRICH-1, supersedes PIVOT-003 bracket-in-name convention):** **Canonical pivot target:** `iocs_value` (TOML column name; `source_path = "$.iocs[*].value"` — wildcard yields JSON-list string per Design Decision 2). `iocs_type` (TOML: `source_path = "$.iocs[*].type"`). `alert_data_url` (TOML: `source_path = "$.alert_data.url"` — CONFIRMED); `alert_data_ip` (TOML: `source_path = "$.alert_data.ip"` — UNCONFIRMED-plausible); `alert_data_domain` (TOML: `source_path = "$.alert_data.domain"` — UNCONFIRMED-plausible). Singleton columns: `ioc_type` (TOML: `source_path = "$.ioc.type"` — PENDING-LIVE-VALIDATION); `ioc_value_singleton` (TOML: `source_path = "$.ioc.value"` — PENDING-LIVE-VALIDATION; name `ioc_value_singleton` avoids collision with `iocs_value`). Singleton top-level `ioc` field has NO public-documentation basis; NOT stamped by scenario generator; NOT targeted by canonical pivot query; still flagged for removal (F-PIVOT003-R2-004). Wire-key note: `Ioc.ioc_type` serializes with `#[serde(rename = "type")]`; `source_path` uses `$.iocs[*].type` (wire key), not `$.iocs[*].ioc_type` (Rust field name). See cyberint.sensor.toml inline comments for full serde alias rationale. | ip, domain, url, hash | S-DEMO-ENRICHMENT-PIVOT-003 (IOC surface); S-DEMO-ENRICH-1 (clean column names + source_path) | `iocs` array and `alert_data.url` confirmed via Check Point sk182975 + FortiSOAR connector (2026-06-19). Inner IOC key names for `iocs[]` elements are **INCONCLUSIVE-pending-live-validation**: the real Cyberint alerts API does not expose inner-element structure in any public doc. DTU implementation uses serde dual-alias (`#[serde(rename = "type", alias = "ioc_type")]` and `#[serde(alias = "ioc_value")]`) to tolerate both wire forms. ENRICH-1 source_path uses `$.iocs[*].type` (wire key "type") per the serde rename annotation — not `$.ioc_type`. Singleton `Alert.ioc` field remains flagged for removal; `generate_with_scenario_iocs` stamps only `iocs[]`. Source: uncertainty-pivot003-s504-2026-06-19.md §Item 2; F-PIVOT003-R2-004 ruling 2026-06-19; S-DEMO-ENRICH-1 cyberint.sensor.toml. |
| CrowdStrike detections | YES (deferred) | **ENRICH-1 clean column names (S-DEMO-ENRICH-1, supersedes PIVOT-003 bracket-in-name convention):** `behaviors_ioc_value` (TOML column name; `source_path = "$.behaviors[*].ioc_value"` — canonical pivot column); `behaviors_ioc_type` (TOML: `source_path = "$.behaviors[*].ioc_type"`); `behaviors_ioc_source` (TOML: `source_path = "$.behaviors[*].ioc_source"`); `behaviors_ioc_description` (TOML: `source_path = "$.behaviors[*].ioc_description"`). All on detection records, NOT on host/device records. Wire keys confirmed: "ioc_type", "ioc_value", "ioc_source", "ioc_description" from `generator.rs` `make_detection()` (untyped `serde_json::Value` path — no Ioc struct serde convention; wire keys are set directly, no rename needed). | `hash_sha256`, `hash_md5`, `domain`, `filename`, `registry_key` — NOT `hash` (bare, incorrect), NOT `registry` (bare, incorrect), NOT `cmdline` (NOT an ioc_type; it is a separate sibling behavior field; NOT ipv4/ipv6 (IPs only appear on separate custom-IOC / device-query surfaces, not on detection `behaviors[].ioc_type`) | S-DEMO-ENRICHMENT-PIVOT-003 (IOC surface); S-DEMO-ENRICH-1 (clean column names + source_path) | Real FalconPy SDK / CrowdStrike Detect API `behaviors[]` array carries `ioc_type`/`ioc_value`/`ioc_source`/`ioc_description` per real-API research. Corrected ioc_type token values per 2026-06-19 research (uncertainty-pivot003-s504-2026-06-19.md §Item 1): ThreatQ CrowdStrike Insight EDR CDF + XSOAR CrowdStrike Falcon integration confirm algorithm-qualified tokens (`hash_sha256`, `hash_md5`) and `_key`-suffixed registry token (`registry_key`); `cmdline` confirmed as separate sibling field, never an ioc_type value. **Tolerant-unknown-type policy:** CrowdStrike publishes no normative exhaustive enum for `behaviors[].ioc_type`; DTU parser MUST treat unknown tokens as non-fatal (log + preserve raw string) rather than rejecting. Source: S-DEMO-ENRICH-1 crowdstrike.sensor.toml; generator.rs `make_detection()` wire key confirmation. |
| CrowdStrike devices (hosts) | NO | — | — | — | Host/device records do not carry IOC fields; IOCs live on detection records only |
| Armis alerts | NO (permanent) | — | — | — | Armis alert payloads are reference-only (deviceIds, activityUUIDs, endpoints). No structured IOC fields in the real Armis API. Fabricating IOC fields would violate the DTU=True-DTU fidelity principle (ADR-031). This exclusion is permanent — not a deferral. |
| Claroty xDome alerts | NO (permanent) | — | — | — | Claroty alerts carry IP addresses only as free text in `alert_name`; no structured IOC schema in the real API. Fabricating structured IOC fields would violate ADR-031. This exclusion is permanent — not a deferral. |

**IOC filtering semantics for sensors WITH IOC surface (Cyberint, CrowdStrike detections):**

- `ioc_hashes=false`: detection records where the wire-level `behaviors[].ioc_value` field value (surfaced as TOML column `behaviors_ioc_value`, `source_path = "$.behaviors[*].ioc_value"`) matches a value in `catalog.ioc_hashes` AND `behaviors[].ioc_type` is one of `hash_sha256` or `hash_md5` (CrowdStrike), or alert records where the wire-level `iocs[].value` field (dual-alias: `value` or `ioc_value`; surfaced as TOML column `iocs_value`, `source_path = "$.iocs[*].value"`) matches a hash-type IOC in `catalog.ioc_hashes` (Cyberint), are withheld from the response. NOTE: the DTU route-handler filter operates on the raw wire JSON (not the TOML column name) because the DTU never reads the TOML spec; the TOML column name is the prism-side surface name, while the DTU filter checks native struct fields.
- `ioc_ips=false`: Cyberint alert records where the wire-level `iocs[].value` field (dual-alias; TOML column `iocs_value`), or `alert_data.ip` field value (TOML column `alert_data_ip`, `source_path = "$.alert_data.ip"`) matches a value in `catalog.ioc_ips` are withheld. CrowdStrike detections: not applicable (CrowdStrike `behaviors[]` does not carry `ipv4`/`ipv6` IOC types on detection records).
- `ioc_domains=false`: Cyberint alert records where the wire-level `iocs[].value` field (dual-alias; TOML column `iocs_value`), or `alert_data.domain` field value (TOML column `alert_data_domain`, `source_path = "$.alert_data.domain"`) matches a value in `catalog.ioc_domains` are withheld. CrowdStrike detections: `behaviors[].ioc_value` (TOML column `behaviors_ioc_value`) with `ioc_type = "domain"` matching `catalog.ioc_domains` are withheld.

**Sensors WITHOUT IOC surface (Armis, Claroty): IOC masking does not apply.** The `ioc_*`
StageMask fields are ignored for these sensors at all stages; they have no IOC-bearing records
to filter.

#### Post-S-DEMO-ENRICHMENT-PIVOT-003 State (ACTIVE as of v1.11)

S-DEMO-ENRICHMENT-PIVOT-003 has shipped. All route guards in the Route Coverage Table are now ACTIVE:

- **Cyberint alerts:** The synthetic `_ioc_value` / `_ioc_type` forward-provision stub has been
  removed. The real-schema filter — checking wire-level `iocs[].value` (dual-alias: `value` or `ioc_value`; prism TOML column `iocs_value` with `source_path = "$.iocs[*].value"`),
  `alert_data.ip` / `alert_data.domain` (prism TOML columns `alert_data_ip` / `alert_data_domain`), AND defensively the singleton `ioc.value` (prism TOML column `ioc_value_singleton`) — is now
  active. The DTU route handler operates on wire-level struct fields; the prism TOML column names are the prism-side surface (surfaced by ENRICH-1 source_path design). The singleton `ioc.value` check is inert (the scenario generator never stamps the
  singleton `ioc` field; only `iocs[]` is populated); the defensive inclusion is harmless per
  story AC-003 and CLAUDE.md §Source-of-Truth Precedence. The singleton `Alert.ioc` field itself
  remains flagged for removal (no public-documentation basis; see F-PIVOT003-R2-004).
- **CrowdStrike detections (list + summaries):** The `stage_idx > 0` guard (from commit `bc0f36c5`)
  has been supplemented by the real-schema `ioc_hashes` filter: detection records are withheld
  (filter_map returns None) when any `behaviors[].ioc_value` ∈ `catalog.ioc_hashes` and
  `!mask.ioc_hashes`. Both the list-IDs route and the get-summaries route enforce this guard.
  See Rows 9–10 in the Route Coverage Table.
- **Armis devices:** The `device_cves_first` per-record field is now omitted
  (`obj.remove("device_cves_first")`) when `!mask.device_cves`. **Two guarded routes:** (1) the
  `paginate_devices` scenario path in `src/routes/devices.rs` (`GET /api/v1/devices`) — Row 8 in
  the Route Coverage Table; (2) the scenario path in `src/routes/search.rs` (device branch;
  `GET /api/v1/search?aql=...`) — Row 11, added v1.12 (F-PIVOT003-R8C-001). The `search.rs` route
  is the path that the `armis.devices` sensor-spec `path_template` targets, and is therefore the
  canonical query path for `from armis.devices` PrismQL statements. Both routes carry the guard;
  `search.rs` is the authoritative route for the `armis.devices` table. There is no `device_cves`
  array field (Ruling 1b — see §PC-4 General Filtering Semantics).

The §Interim State prose above (pre-v1.11 versions of this BC) is superseded by this section. The
`_ioc_value` synthetic filter is absent from the codebase; the real-schema filter is load-bearing.

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
| `primary_device`, `lateral_devices` | prism-dtu-crowdstrike | `routes/hosts.rs` | `GET /devices/queries/devices/v1` and `GET /devices/entities/devices/v2` (existing `get_host_details`) + `POST /devices/entities/devices/v2` (new `post_host_details`; mirrors `get_host_details` StageMask/scenario/session-registry logic exactly per DEFECT-CSDEVICES-EMPTY-PIPELINE-001 ratification 2026-07-10) | `stage_idx > 0` for primary; `mask.lateral_devices` for lateral | ACTIVE (GET pre-bc0f36c5, B-P1-01; POST delivered by DEFECT-CSDEVICES-EMPTY-PIPELINE-001 fix lane (post_host_details; shared host_details_inner helper with GET; identical StageMask/session-registry semantics)) |
| `ioc_hashes`, `ioc_ips`, `ioc_domains` | prism-dtu-cyberint | `routes/alerts.rs` | `GET /api/v1/alerts` (also registered for POST via same handler — confirmed routes/alerts.rs) | Real-schema filter (S-DEMO-ENRICHMENT-PIVOT-003): checks wire-level `iocs[].value` (dual-alias: `value` or `ioc_value`; prism TOML column `iocs_value`, `source_path = "$.iocs[*].value"` per ENRICH-1), `alert_data.ip`/`alert_data.domain` (prism TOML columns `alert_data_ip`/`alert_data_domain`), AND defensively the singleton `ioc.value` (prism TOML column `ioc_value_singleton`) (per `ioc_values_for` in `routes/alerts.rs`, consistent with story AC-003); synthetic `_ioc_value` filter removed atomically in same commit; singleton `ioc.value` check is inert — generator never stamps the singleton field, so the defensive inclusion never matches in practice but is harmless (F-PIVOT003-R10A-002 coherence fix v1.13). DTU checks wire-level struct fields; prism TOML column names are the prism-side surface via ENRICH-1 source_path. | ACTIVE (real-schema filter, S-DEMO-ENRICHMENT-PIVOT-003) |
| `ioc_hashes` | prism-dtu-crowdstrike | `routes/detections.rs` | `GET /detects/queries/detects/v1` (list IDs) and `POST /detects/entities/summaries/GET/v1` (batch summaries — confirmed routes/mod.rs) | `stage_idx > 0` guard on both list and summary routes (added commit bc0f36c5); real-schema `ioc_hashes` filter (filter_map withhold when `behaviors[].ioc_value` ∈ `catalog.ioc_hashes` and `!mask.ioc_hashes`) added S-DEMO-ENRICHMENT-PIVOT-003 — see Rows 9–10 | ACTIVE (stage-guard + real-schema ioc_hashes filter) |
| `primary_device`, `lateral_devices` | prism-dtu-claroty | `routes/devices.rs` | `POST /api/v1/devices` (registered in `ClarotyClone` router via `build_router` in `clone.rs`) | `mask.primary_device && stage_idx > 0` for primary (in `list_devices` scenario-path block); `mask.lateral_devices` for lateral (in `list_devices` scenario-path block); single handler `list_devices` covers full device list (no separate detail handler in this file) | ACTIVE (added BPRL-P6-01) |
| (no IOC surface) | prism-dtu-claroty | `routes/alerts.rs` | `POST /api/v1/alerts` (confirmed clone.rs) | EXEMPT — no structured IOC fields in real Claroty API; device_id not emitted on alert records; relation via separate endpoint | PERMANENT EXEMPT |
| `primary_device`, `lateral_devices` | prism-dtu-claroty | `routes/device_alert_relations.rs` | `POST /api/v1/device_alert_relations/` (NormalizePathLayer strips trailing slash; `list_device_alert_relations` handler; registered without trailing slash in `build_router` in `clone.rs`) | `mask.primary_device && stage_idx > 0` for primary device relations (keyed on `_device_id` matching `timeline.entities.primary_device_id_cs`); `mask.lateral_devices` for lateral device relations (keyed on `_device_id` in `timeline.entities.lateral_device_ids_cs`); non-catalog relations pass unconditionally; mirrors `list_devices` guard exactly per BC-2.06.019 PC-4; enforces INV-CROSS-DTU-ENTITY-COHERENCE-001 cross-DTU join coherence | ACTIVE (added S-DEMO-CLAROTY-DAR-001, F-CLARO-RG-P1-HIGH-001) |
| `device_cves` | prism-dtu-armis | `src/routes/devices.rs` | `GET /api/v1/devices` | Per-record `device_cves_first` field omitted (`obj.remove("device_cves_first")`) when `!mask.device_cves`; applied in the `paginate_devices` scenario path | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003) |
| `device_cves`, `primary_device`, `lateral_devices` | prism-dtu-armis | `src/routes/search.rs` | `GET /api/v1/search?aql=...` (device branch, scenario path; this is the route `armis.devices` sensor-spec `path_template` targets — the canonical path for `from armis.devices` PrismQL queries) | Per-record `device_cves_first` omitted (`obj.remove("device_cves_first")`) when `!mask.device_cves` (device branch); entity-visibility guards: `primary_device` and `lateral_devices` applied on device-branch records per-stage | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003, F-PIVOT003-R8C-001) |
| `ioc_hashes` | prism-dtu-crowdstrike | `src/routes/detections.rs` | `GET /detects/queries/detects/v1` | Detection withheld (filter_map returns None) when any `behaviors[].ioc_value` ∈ `catalog.ioc_hashes` AND `!mask.ioc_hashes`; list-ID scenario path | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003) |
| `ioc_hashes` | prism-dtu-crowdstrike | `src/routes/detections.rs` | `POST /detects/entities/summaries/GET/v1` | Detection withheld (filter_map returns None) when any `behaviors[].ioc_value` ∈ `catalog.ioc_hashes` AND `!mask.ioc_hashes`; get-summaries scenario path | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003) |

**Inventory verification (v1.18) — exhaustive StageMask handler scan:**

Union of files returned by `rg -l 'mask\.primary_device|mask\.lateral_devices|mask\.ioc_|mask\.device_cves|stage_idx' crates/prism-dtu-*/src/routes/` and `rg -l 'scenario_stage_ctx|with_stage_mask_projection|StageMask' crates/prism-dtu-*/src/routes/`:

| File | Handler(s) | Registered Route | Table Row Status |
|------|-----------|-----------------|-----------------|
| `prism-dtu-armis/src/routes/devices.rs` | `list_devices` | `GET /api/v1/devices` | Row 1 (pre-bc0f36c5); `device_cves` guard added S-DEMO-ENRICHMENT-PIVOT-003 (Row 8) |
| `prism-dtu-armis/src/routes/search.rs` | search handler (device branch) | `GET /api/v1/search?aql=...` | Row 2 (v1.5 added — primary/lateral guards); `device_cves` guard added S-DEMO-ENRICHMENT-PIVOT-003 Row 11 (v1.12, F-PIVOT003-R8C-001). This is the canonical `armis.devices` query path per sensor-spec `path_template`. |
| `prism-dtu-armis/src/routes/alerts.rs` | `list_alerts` | `GET /api/v1/alerts` | Row 3 (v1.5 added) |
| `prism-dtu-crowdstrike/src/routes/hosts.rs` | hosts list + entities | `GET /devices/queries/devices/v1`, `GET /devices/entities/devices/v2` (existing), and `POST /devices/entities/devices/v2` (new `post_host_details`; same StageMask guards as `get_host_details` per DEFECT-CSDEVICES-EMPTY-PIPELINE-001 ratification 2026-07-10) | Row 4 (pre-bc0f36c5); POST handler delivered by DEFECT-CSDEVICES-EMPTY-PIPELINE-001 fix lane (post_host_details; shared host_details_inner helper with GET; identical StageMask/session-registry semantics) |
| `prism-dtu-crowdstrike/src/routes/detections.rs` | list IDs + batch summaries | `GET /detects/queries/detects/v1` and `POST /detects/entities/summaries/GET/v1` | Row 5 (v1.4 added); `ioc_hashes` real-schema guards added S-DEMO-ENRICHMENT-PIVOT-003 (Rows 9–10) |
| `prism-dtu-cyberint/src/routes/alerts.rs` | `list_alerts` | `GET /api/v1/alerts` (also POST) | Row 6 (v1.4 added) |
| `prism-dtu-claroty/src/routes/devices.rs` | `list_devices` | `POST /api/v1/devices` | **Row 7 — added v1.6 (BPRL-P6-01)** |
| `prism-dtu-claroty/src/routes/device_alert_relations.rs` | `list_device_alert_relations` | `POST /api/v1/device_alert_relations/` | **Row added v1.18 (F-CLARO-RG-P1-HIGH-001)** — added S-DEMO-CLAROTY-DAR-001 (Claroty live-API fidelity branch); was the un-enumerated 8th union-grep file whose omission triggered this finding |

All 8 files in the union are now enumerated. Rows 8–10 (added v1.11) cover additional StageMask fields (`device_cves`, `ioc_hashes`) on files already present in the inventory. Row 11 (added v1.12, F-PIVOT003-R8C-001) covers the `device_cves` guard on `search.rs` — also a file already present in the inventory. The 8th file (`prism-dtu-claroty/src/routes/device_alert_relations.rs`) was added by the Claroty live-API fidelity branch (S-DEMO-CLAROTY-DAR-001); it is the newly-added StageMask handler whose omission triggered F-CLARO-RG-P1-HIGH-001 and is now enumerated in the Route Coverage Table row for `list_device_alert_relations` (v1.18). The `prism-dtu-claroty/src/routes/alerts.rs` file does NOT appear in either grep set; it is PERMANENT EXEMPT solely on real-API grounds: the real Claroty xDome API emits no structured IOC fields on alert records and no `device_id` field, so no StageMask projection is applicable there. The Claroty alerts EXEMPT row remains correct and complete.

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
| v1.18 | F-CLARO-RG-P1-HIGH-001 POL-33 process-gap closure 2026-08-12. **Root cause:** The Claroty live-API fidelity branch (S-DEMO-CLAROTY-DAR-001) added a StageMask-guarded scenario path to `list_device_alert_relations` in `routes/device_alert_relations.rs` (`mask.primary_device && stage_idx > 0` for primary device relations keyed on `_device_id` matching `timeline.entities.primary_device_id_cs`; `mask.lateral_devices` for lateral device relations keyed on `_device_id` in `timeline.entities.lateral_device_ids_cs`; enforces INV-CROSS-DTU-ENTITY-COHERENCE-001; mirrors `list_devices` guard exactly per PC-4). The POL-33 union grep now returns 8 files, but the Route Coverage Table and Inventory verification table still asserted "All 7 files" — both were FALSE. **Changes:** (1) ADDED Route Coverage Table row: `primary_device, lateral_devices / prism-dtu-claroty / routes/device_alert_relations.rs / POST /api/v1/device_alert_relations/ (list_device_alert_relations) / mask.primary_device && stage_idx > 0 for primary (keyed on _device_id matching primary_device_id_cs); mask.lateral_devices for lateral (keyed on _device_id in lateral_device_ids_cs); mirrors list_devices; INV-CROSS-DTU-ENTITY-COHERENCE-001 / ACTIVE (S-DEMO-CLAROTY-DAR-001)`. (2) ADDED Inventory verification table row for `device_alert_relations.rs` as the 8th union-grep file (handler `list_device_alert_relations`, route `POST /api/v1/device_alert_relations/`). (3) UPDATED inventory prose: "All 7 files" → "All 8 files"; stale "no new files enter the StageMask handler set" claim replaced with accurate description of the 8th file (S-DEMO-CLAROTY-DAR-001 addition). (4) Updated inventory version marker "(v1.12)" → "(v1.18)". (5) FIX-IN-SCOPE TD-031 violations in pre-existing content: Route Coverage Table Row 7 and Changelog v1.6 both contained volatile `*.<ext>:NNN` line cites (`build_router` route confirmation and `list_devices` guard verification now cited by symbol/function name instead of line number). TD-VSDD-097 dimension 1 sibling sweep: all 7 pre-existing union files verified to have accurate rows — no additional missing row found beyond `device_alert_relations.rs`. BC v1.17 → v1.18. |
| v1.17 | F-CSD-P1-006 time-brittle annotation removal 2026-07-10. **Root cause:** Route Coverage Table Row 4 Status and Inventory verification table Row 4 "Table Row Status" both contained "POST pending DEFECT-CSDEVICES-EMPTY-PIPELINE-001 fix lane" — temporal/branch-reference language that becomes stale once the fix lane merges to develop (violates TD-VSDD-091 behavioral-anchors-only). **Changes:** (1) Route Coverage Table Row 4 Status: "POST pending DEFECT-CSDEVICES-EMPTY-PIPELINE-001 fix lane" → "POST delivered by DEFECT-CSDEVICES-EMPTY-PIPELINE-001 fix lane (post_host_details; shared host_details_inner helper with GET; identical StageMask/session-registry semantics)". (2) Inventory verification table Row 4 Table Row Status: identical replacement. No semantic change to guard logic or handler behavior. Closes adversary finding F-CSD-P1-006 (MED). BC v1.16 → v1.17. |
| v1.16 | DEFECT-CSDEVICES-EMPTY-PIPELINE-001 route coverage update 2026-07-10. **Root cause:** Route Coverage Table Row 4 and Inventory verification table Row 4 both enumerated only `GET /devices/entities/devices/v2` for `prism-dtu-crowdstrike/src/routes/hosts.rs`. Per architect ratification of DEFECT-CSDEVICES-EMPTY-PIPELINE-001 (D-1650 §Architect Ratification, research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md), the route `/devices/entities/devices/v2` gains a POST handler `post_host_details` that mirrors `get_host_details` StageMask/scenario/session-registry logic exactly (same auth check, same org-id guard, same three-way composition, same containment merge, same response shape). **Changes:** (1) Route Coverage Table Row 4: added `POST /devices/entities/devices/v2` (`post_host_details`) alongside existing `GET /devices/entities/devices/v2` (`get_host_details`); Status updated to note POST handler is pending the DEFECT-CSDEVICES-EMPTY-PIPELINE-001 fix lane. (2) Inventory verification table Row 4: updated `hosts.rs` route column to enumerate GET, GET (existing), and POST (new, same StageMask guards). No change to guard semantics — both handlers share identical StageMask behavior per the ratification contract. BC v1.15 → v1.16. |
| v1.15 | GAP-1 enrichment UDF name correction 2026-06-23. **Root cause:** §PC-4 canonical NVD pivot query used `\| enrich nvd(device_cves_first)` — citing the `infusion_id` ("nvd") rather than the per-field registered UDF name. The infusion_id is NOT the registered DataFusion function name. The actual registered UDF name is derived from `[[infusion.fields]].name` in `nvd.infusion.toml`, which is `cvss_base_score`. **Change:** `\| enrich nvd(device_cves_first)` → `\| enrich cvss_base_score(device_cves_first)`. Added adversary-probe note: `\| enrich nvd(...)` is stale and MUST be treated as a P1 finding. No semantic change to filtering logic, stage-mask behavior, or route guards. Source: `specs/infusions/nvd.infusion.toml` `[[infusion.fields]]` name field; T13 capstone demo runbook GAP-1 audit 2026-06-23. |
| v1.14 | ENRICH-1 clean-column-name amendment 2026-06-23 (S-DEMO-ENRICH-1). **Root cause:** PIVOT-003 (v1.8–v1.13) introduced IOC columns using the bracket-in-name convention (`iocs[].value`, `behaviors[].ioc_value`, `alert_data.ip`, etc.) in the Per-Sensor IOC-Surface Matrix and IOC filtering semantics. ENRICH-1 supersedes this convention: all bracket-in-name columns are renamed to clean SQL identifiers with `source_path` carrying the JSONPath (sensor-column-source-path-design.md §DD-3). **Changes:** (1) Per-Sensor IOC-Surface Matrix Cyberint row: canonical pivot target updated from `iocs[].value` to `iocs_value` (TOML column name; `source_path = "$.iocs[*].value"`); all bracket-in-name column references replaced with clean names + source_path annotations (`iocs_type`, `alert_data_url`, `alert_data_ip`, `alert_data_domain`, `ioc_type`, `ioc_value_singleton`). Wire-key note added for serde rename (`$.iocs[*].type` = wire key "type" per `#[serde(rename = "type")]` on `Ioc.ioc_type`). (2) CrowdStrike row: `behaviors[].ioc_value` → `behaviors_ioc_value` (TOML; `source_path = "$.behaviors[*].ioc_value"`); all four behaviors IOC columns updated. Wire key confirmation added from generator.rs `make_detection()` direct `serde_json::Value` insertion. (3) IOC filtering semantics bullets: `iocs[].value` → wire-level `iocs[].value` (dual-alias; TOML column `iocs_value`) with DTU-vs-prism distinction clarified. (4) §Post-S-DEMO-ENRICHMENT-PIVOT-003 State Cyberint bullet and Route Coverage Table Row 6: added prism TOML column name parentheticals alongside wire-level descriptions. DTU route handlers check wire-level struct fields; TOML column names are the prism-side surface surfaced by ENRICH-1 source_path. No semantic change to guard logic. This amendment records the ENRICH-1-implemented renaming; spec supersedes PIVOT-003 bracket convention per CLAUDE.md §Source-of-Truth Precedence (story spec scope). Cite: S-DEMO-ENRICH-1 cyberint.sensor.toml + crowdstrike.sensor.toml; sensor-column-source-path-design.md §DD-3. |
| v1.13 | PO prose-vs-code coherence fix 2026-06-19 (F-PIVOT003-R10A-002 OBS). **Root cause:** Route Coverage Table Row 6 (Cyberint alerts) and §Post-S-DEMO-ENRICHMENT-PIVOT-003 State prose both stated the real-schema IOC filter "does NOT check singleton `ioc.value`." The implemented `ioc_values_for` function in `crates/prism-dtu-cyberint/src/routes/alerts.rs` defensively includes `alert.ioc.value` (the singleton) alongside `iocs[].value` and `alert_data.ip`/`alert_data.domain`, consistent with story AC-003. Per CLAUDE.md §Source-of-Truth Precedence, the story AC governs implementation scope; the BC prose was the stale artifact. The singleton check is functionally inert — the scenario generator never stamps the singleton `ioc` field (`iocs[]` only), so the check never matches in practice. **Changes:** (1) Row 6 Guard Mechanism updated to accurately describe the implemented filter as checking `iocs[].value` (dual-alias), `alert_data.ip`/`alert_data.domain`, AND defensively `ioc.value`; inert-but-harmless rationale added inline. (2) §Post-S-DEMO-ENRICHMENT-PIVOT-003 State Cyberint bullet updated to match: "does NOT check singleton" claim replaced with accurate description of defensive singleton inclusion, with inert-harmless qualification. The singleton `Alert.ioc` field-removal flag (F-PIVOT003-R2-004) is preserved — field removal remains the right long-term action; this fix only corrects the filter description. **Story-writer note:** no structural change to acceptance criteria; the stale "does NOT check singleton" language may appear in S-DEMO-ENRICHMENT-PIVOT-003 AC prose — story-writer should propagate the corrected description there. |
| v1.12 | POL-33 route coverage correction 2026-06-19 (F-PIVOT003-R8C-001 HIGH). **Root cause:** v1.11 Route Coverage Table Row 8 claimed `paginate_devices` (`GET /api/v1/devices`, `src/routes/devices.rs`) was the `device_cves` guard for Armis devices, and §Post-S-DEMO-ENRICHMENT-PIVOT-003 State prose stated "this is the only `device_cves` guard." This was incorrect: the `armis.devices` sensor-spec `path_template` targets `GET /api/v1/search?aql=...` (`src/routes/search.rs`, device branch), not `GET /api/v1/devices`. The `device_cves_first` leakage at stages 0–3 was occurring on the canonical analyst query path (`from armis.devices` → search.rs) because the `search.rs` guard was missing. The implementer added the `device_cves_first` masking guard (and the previously-missing `primary_device`/`lateral_devices` entity-visibility guards) to `crates/prism-dtu-armis/src/routes/search.rs` (device branch, scenario path) as part of S-DEMO-ENRICHMENT-PIVOT-003. **Changes:** (1) ADDED Row 11 to Route Coverage Table: `device_cves, primary_device, lateral_devices` × prism-dtu-armis × `src/routes/search.rs` × `GET /api/v1/search?aql=...` (device branch, scenario path; canonical `armis.devices` query path) × per-record `device_cves_first` omitted when `!mask.device_cves`; entity-visibility guards for primary/lateral devices × ACTIVE (F-PIVOT003-R8C-001). (2) RETRACTED "only `device_cves` guard" claim in §Post-S-DEMO-ENRICHMENT-PIVOT-003 State — replaced with accurate two-route description: `devices.rs` (Row 8) and `search.rs` (Row 11) both carry the guard; `search.rs` is the authoritative route for `armis.devices`. (3) ADDED authoritative-route note in §PC-4 `device_cves` bullet clarifying that the canonical `armis.devices` query path is `/api/v1/search`, `/api/v1/devices` is secondary, and masking applies on whichever route a table's `path_template` targets. (4) UPDATED Inventory verification note to v1.12 with `search.rs` table entry updated to cite Row 11. **Story-writer must mirror:** story-writer must propagate the new Route Coverage Row 11 into S-DEMO-ENRICHMENT-PIVOT-003 AC-009 Route Coverage Table, and update any AC prose that references `/api/v1/devices` as the sole `device_cves` guard. |
| v1.11 | POL-33 route coverage update 2026-06-19 (F-PIVOT003-R7A-001/R7A-002 HIGH). S-DEMO-ENRICHMENT-PIVOT-003 shipped 3 new StageMask-relevant route guards that were not reflected in the Route Coverage Table. **Three rows added:** (1) `device_cves` × prism-dtu-armis × `src/routes/devices.rs` × `GET /api/v1/devices` — per-record `device_cves_first` omitted (`obj.remove("device_cves_first")`) when `!mask.device_cves` in the paginate_devices scenario path (Row 8). (2) `ioc_hashes` × prism-dtu-crowdstrike × `src/routes/detections.rs` × `GET /detects/queries/detects/v1` — detection withheld (filter_map None) when `behaviors[].ioc_value` ∈ catalog.ioc_hashes and `!mask.ioc_hashes` (Row 9). (3) Same guard on `POST /detects/entities/summaries/GET/v1` (Row 10). **PC-4 §Interim State reconciled:** the "Until S-DEMO-ENRICHMENT-PIVOT-003" block replaced with "Post-S-DEMO-ENRICHMENT-PIVOT-003 State (ACTIVE)" prose confirming all three previously-deferred guards are now live. The CrowdStrike detections Row 5 status updated from "STAGE-GUARD ACTIVE, IOC-STAMP DEFERRED" to "ACTIVE (stage-guard + real-schema ioc_hashes filter)". The Cyberint alerts Row 6 status updated from "INTERIM" to "ACTIVE (real-schema filter, S-DEMO-ENRICHMENT-PIVOT-003)". Inventory verification note updated to v1.11 with explanatory note that Rows 8–10 cover additional StageMask fields on files already in the 7-file inventory set. **Story-writer must mirror:** AC-009 Route Coverage Table in S-DEMO-ENRICHMENT-PIVOT-003 story spec needs these 3 rows propagated. |
| v1.10 | PO spec-coherence reconciliation 2026-06-19 (F-PIVOT003-R5B-001 MED + F-PIVOT003-R5B-002 OBS). **Root cause:** AC-008 of S-DEMO-ENRICHMENT-PIVOT-003 referenced `where has device_cves` as the NVD pivot query existence filter, but the `device_cves` array field is NEVER stamped on generated records and is NOT declared as a TOML column (Ruling 1b). The filter would match zero records, making the canonical NVD pivot query return 0 rows against the demo server. The scalar projection `device_cves_first` IS surfaced on device records and IS the correct pivot field. **Changes:** (1) Extended `device_cves=false` bullet in §PC-4 General Filtering Semantics to explicitly state that `device_cves` array is never stamped on generated records (Ruling 1b), that only the scalar `device_cves_first` is surfaced, and to define the canonical NVD pivot query with the corrected filter `where has device_cves_first` and the canonical CVSS filter form `where cvss_base_score >= 7.0`. (2) Added adversary-probe note: any reference to `has device_cves` (array form) as a PrismQL filter, or `nvd_cvss_score` as a CVSS column name, is stale and must be treated as a P1 finding. **Story-writer propagates:** the corrected canonical NVD pivot query to S-DEMO-ENRICHMENT-PIVOT-003 AC-008 query text, EC-008 explanation, and all related prose referencing `has device_cves` or `nvd_cvss_score`. |
| v1.9 | PO coherence fix 2026-06-19 (F-PIVOT003-R2-004 HIGH) — Canonical ThreatIntel pivot query field reconciliation. **Root cause:** the canonical pivot query (AC-007 of S-DEMO-ENRICHMENT-PIVOT-003) targeted `ioc.value` (singleton), but `generate_with_scenario_iocs` stamps only the plural `iocs[]` array, never the singleton `Alert.ioc`. The singleton has no public-documentation basis and was already flagged for likely removal in v1.8. Resolving with `iocs[].value` as the canonical pivot target. **Changes:** (1) Per-Sensor IOC-Surface Matrix Cyberint row updated: `iocs[].value` explicitly marked as "canonical pivot target"; singleton `ioc.value` explicitly marked "NOT stamped by scenario generator, NOT targeted by canonical pivot query — case for removal strengthens now that no pivot query depends on it." (2) Route Coverage Table INTERIM row: future-state guard mechanism description now explicitly states the replacement filter checks `iocs[].value` (dual-alias) and `alert_data.ip`/`domain`, and does NOT check singleton `ioc.value`. (3) IOC filtering semantics (PC-4) already correctly used `iocs[].value` — no change required there. **PC-4 fail-closed ruling (for implementer):** PC-4's projection-integrity postcondition requires fail-closed behavior; alert-surface records that do not deserialize as `Alert` MUST be withheld (not passed through). See explicit §PC-4 note below. Story-writer propagates the `ioc.value` → `iocs[].value` change to story AC-007 canonical query text. |
| v1.8 | PO fidelity correction 2026-06-19 — True-DTU ioc_type enum correction (ADR-031) for CrowdStrike and Cyberint rows in Per-Sensor IOC-Surface Matrix. **CrowdStrike:** `behaviors[].ioc_type` value set corrected from `{hash, domain, filename, registry, cmdline}` to `{hash_sha256, hash_md5, domain, filename, registry_key}`. Bare `hash` → split into algorithm-qualified `hash_sha256` + `hash_md5`. Bare `registry` → `registry_key`. `cmdline` removed from the ioc_type value set — it is a SEPARATE sibling behavior field (`behaviors[].cmdline`), never an `ioc_type` value. `domain`/`filename` unchanged. ipv4/ipv6 exclusion retained (confirmed correct). Added tolerant-unknown-type policy: CrowdStrike publishes no normative exhaustive enum; parser must treat unknown `ioc_type` tokens as non-fatal. **Cyberint:** Inner IOC key names for `iocs[]` elements marked INCONCLUSIVE-pending-live-validation. DTU implementation MUST use serde dual-alias (`type`/`value` AND `ioc_type`/`ioc_value`) rather than a single hard-coded guess. Singleton top-level `ioc` field flagged for likely removal (no public-documentation basis found). `iocs[]` array and `alert_data.url` confirmed; `alert_data.ip`/`domain` remain plausible-unconfirmed. IOC filtering semantics updated to use dual-alias reference for Cyberint `iocs[].value`. Source: uncertainty-pivot003-s504-2026-06-19.md §Item 1 (HIGH confidence) + §Item 2 (INCONCLUSIVE). |
| v1.7 | PO micro-fix 2026-06-12 (BPRL-P7-01) — Inventory verification note prose corrected. Replaced fabricated claim that `prism-dtu-claroty/src/routes/alerts.rs` "appears in both grep sets due to `scenario_stage_ctx` or similar context references" with the accurate statement: the file does NOT appear in either grep set; its PERMANENT EXEMPT table row stands solely on real-API grounds (no structured IOC fields in the real Claroty xDome API per 2026-06-12 research; no `device_id` emitted on alert records). No table change, no semantic change. |
| v1.6 | PO fix-burst 2026-06-12 (BPRL-P6-01) — Route Coverage Table corrected: (1) ADDED missing Claroty devices row: `primary_device, lateral_devices / prism-dtu-claroty / routes/devices.rs / POST /api/v1/devices (registered via build_router in clone.rs) / mask.primary_device && stage_idx > 0 for primary; mask.lateral_devices for lateral / ACTIVE`. Guard verified in `list_devices` scenario-path block (`routes/devices.rs`); route confirmed via `ClarotyClone` `build_router` registration in `clone.rs`. [TD-031 volatile cites corrected at v1.18.] (2) ADDED exhaustive inventory verification note under table: all 7 StageMask-consulting route files in the `prism-dtu-*` workspace enumerated with handler↔route↔row mapping, providing mechanical re-verification baseline for future passes. No further omissions found beyond the Claroty devices row. |
| v1.5 | PO fix-burst 2026-06-12 (BPRL-P5-01) — Route Coverage Table corrected to match commit-verified router ground truth. (1) DELETED phantom row `prism-dtu-crowdstrike / routes/alerts_search.rs / GET /alerts/queries/alerts/v2 (in:alerts branch)` — no such file or route exists in prism-dtu-crowdstrike (routes/: mod.rs, oauth.rs, writes.rs, hosts.rs, detections.rs); "in:alerts" is Armis AQL terminology mis-attributed to CrowdStrike. (2) CORRECTED crowdstrike detections summary row: route corrected from `GET /detects/entities/summaries/v1` (wrong method + path) to `POST /detects/entities/summaries/GET/v1` (confirmed routes/mod.rs). (3) ADDED missing Armis search row: `routes/search.rs / GET /api/v1/search / stage_idx > 0 + mask.lateral_devices guard` — StageMask-relevant route guarded in commit bc0f36c5 but never enumerated (exact POL-33 omission class). (4) TIGHTENED lateral wording: all rows now state `mask.lateral_devices` (the actual guard mechanism) instead of `stage_idx >= 2` (behaviorally equivalent but imprecise). (5) Corrected CrowdStrike device rows to accurate route paths: `GET /devices/queries/devices/v1` and `GET /devices/entities/devices/v2` (confirmed routes/mod.rs + hosts.rs). (6) Corrected Claroty alerts path from `GET /xdome/api/v1/alerts` to `POST /api/v1/alerts` (confirmed clone.rs). (7) PC-4 prose updated: 4-arg `new_with_scenario(seed, archetype, org_id, Arc<IncidentTimeline>)` → 5-arg form `(seed, archetype, org_id, Arc<IncidentTimeline>, time_anchor: DateTime<Utc>)` per ADR-036 v2.3; per-clone return type split noted (CrowdStrike `-> Self`; Armis/Cyberint/NVD `-> anyhow::Result<Self>`) per code-verified signatures in clone.rs. |
| v1.4 | PO burst 2026-06-12 (D-1109, WO-D1109) — PC-4 redesigned from a single blanket IOC-on-alert clause to a full per-sensor IOC-surface matrix. Root cause: BPRL-P4-01 (MED) — BC-2.06.019 v1.3 PC-4's IOC filter was production-inert because no DTU generator stamped IOC fields using real API schema field names; the Cyberint `_ioc_value` filter matched only a synthetic injected field. Resolution: (1) Replaced blanket clause with the Per-Sensor IOC-Surface Matrix (research-agent 2026-06-12: falconpy SDK, Cyberint portal docs, ThreatQ/XSOAR/Elastic field maps); Cyberint alerts (real fields: `ioc.value`, `iocs[].value`, `alert_data.ip/domain/url`) and CrowdStrike detections (`behaviors[].ioc_value`; hash/domain/filename/registry/cmdline; NOT ipv4/ipv6) carry IOC data per real API; Armis and Claroty permanently excluded on ADR-031 fidelity grounds. (2) Added Interim State clause: `_ioc_value` synthetic filter acknowledged as forward-provision stub; removed atomically in S-DEMO-ENRICHMENT-PIVOT-003 when real-schema fields land. (3) Added Route Coverage Table (BPRL-P4-02 process-gap codification, per WO-D1109 §Question 4): enumerates every StageMask-relevant DTU route × guard mechanism × status; standing rule added requiring table update in same commit as any future StageMask-relevant route addition. BPRL-P4-02 fix (commit bc0f36c5) codified: stage-guards on crowdstrike detections list/summary routes and armis alerts/search in:alerts branch now appear in the table. |
| v1.3 | PO micro-burst 2026-06-12 — B-P5-01 precondition renumbering correction. The v1.2 insertion of PRE-6 (org_id equality guard) displaced the archetype and build-before-start preconditions but the body labels were incorrectly written as PRE-8 and PRE-9, skipping PRE-7. Corrected: archetype precondition → PRE-7 (was mislabeled 8), build-before-start → PRE-8 (was mislabeled 9). The v1.2 changelog claim "Renumbered former PRE-6→PRE-8 … PRE-7→PRE-9" is annotated `[corrected at v1.3]` below. No semantic change to any precondition. B-P5-02 (taxonomy half): error-taxonomy.md E-DEMO-003 row updated from "§Precondition 6" to "§Precondition 7" (archetype is PRE-7 post-renumber). |
| v1.2 | PO micro-burst 2026-06-12 — OBS-1 org_id-equality gap closed. Added PRE-6 (org_id equality guard across scenario-enabled clones; E-DEMO-006; detection before E-DEMO-003; rationale: silent INV-CROSS-DTU-ENTITY-COHERENCE-001 incoherence is SOUL.md §4 class). Renumbered former PRE-6→PRE-8 (archetype) and PRE-7→PRE-9 (build_clone_pairs before start_all) [corrected at v1.3: body labels were written as 8 and 9, skipping 7; correct labels are PRE-7 (archetype) and PRE-8 (build-before-start)]. Added E-DEMO-006 error code section with full format table. Added EC-019-013 (org_id mismatch edge case). Added TV-019-015 (E-DEMO-006 test vector). Added VP-019-I (E-DEMO-006 unit test). OBS-2 anchor drift fixed: Stories traceability row, Story Anchor section, and VP Anchors section updated to include S-DEMO-DTU-LIVE-SCENARIO-001-B. Guard order in PRE-6: E-DEMO-002 → E-DEMO-006 → E-DEMO-003 → E-DEMO-004. |
| v1.1 | ADR-036 v2.0 / D-1078 substrate-reconciliation corrections. Pinned `stage_duration_secs` 4-entry convention (ADR-036 v2.0 §1.3, §2.1): added explicit 4-entry table showing array-index-to-stage mapping in §Postcondition 2; stage 0 (Baseline) always activates at 0s and is NOT represented in the array. Corrected EC-019-006 which incorrectly described 4 entries as an E-DEMO-003 error — 4 entries IS the correct count for `CompromisedEndpoint`; added EC-019-006b (3 entries → error) and EC-019-006c (5 entries → error) to document the actual error cases. Confirmed `activates_after_secs: u64` as the authoritative `IncidentStage` field name (NOT "duration") per ADR-036 v2.0 §2.2. All stage threshold values (60/180/360/600) unchanged. lifecycle_status remains draft. Invariant semantics unchanged. |
| v1.0 | Initial authoring. ADR-036 ACCEPTED 2026-06-09. BC-2.06.019 namespace confirmed (next-available after BC-2.06.018). Subsystem: SS-01 (prism-dtu-demo-server, prism-dtu-common). Capability: CAP-036 — harness wiring layer (temporal staging extends the harness orchestration that BC-2.06.018 began). Error codes E-DEMO-002 and E-DEMO-003 registered here (to be reflected in error-taxonomy.md by error-taxonomy owner in same burst per BC-2.06.018 precedent). Invariant naming follows ADR-036 §7 candidate invariant IDs verbatim with BC-house-style expansion. |
