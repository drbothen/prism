---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.020"
version: "1.6"
status: draft
lifecycle_status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-036"
introduced: "2026-06-09"
modified: "2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-DEMO-DTU-LIVE-SCENARIO-001, S-DEMO-DTU-LIVE-SCENARIO-001-B]
verifying_vps: []
crates: [prism-dtu-demo-server, prism-dtu-common, prism-dtu-threatintel, prism-dtu-nvd, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty, prism-dtu-cyberint]
inputs:
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - "crates/prism-dtu-threatintel/src/state.rs"
  - "crates/prism-dtu-nvd/src/state.rs"
  - "crates/prism-dtu-armis/src/state.rs"
  - "crates/prism-dtu-crowdstrike/src/state.rs"
  - "crates/prism-dtu-cyberint/src/generator.rs"
  - "crates/prism-dtu-common/src/generator/archetype.rs"
  - "crates/prism-dtu-common/src/scenario/mod.rs"
  - ".factory/specs/behavioral-contracts/BC-2.06.019-demo-server-scenario-progression.md"
  - ".factory/specs/architecture/decisions/ADR-036-deterministic-scenario-progression-engine.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: ""
traces_to:
  - "CAP-036"
  - "ADR-036"
  - "BC-2.06.019"
extracted_from: null
---

# BC-2.06.020: Demo-Server Enrichment Correlation — Scenario IOCs Resolve in ThreatIntel; Scenario CVEs Resolve in NVD; Cyberint Alert CVEs Use Catalog IDs (Collision-Safe in All Modes)

## Description

When `build_clone_pairs` constructs ThreatIntel and NVD clones for a client config that
has `scenario.enabled = true`, it calls overloaded constructors
`ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog)` and
`NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog)` that pre-populate their
respective registries with scenario-correlated entries. This BC governs the **lookup
injection wiring layer**: the mechanism by which the scenario's IOC indicators and CVE
identifiers (produced by BC-2.06.019's `ScenarioEntityCatalog`) become resolvable in the
enrichment DTU clones. It does NOT introduce a new generator for ThreatIntel or NVD;
lookup injection into the existing `fixture_registry` / `cve_registry` structures is the
complete mechanism per ADR-036 §3.2.

This BC also governs the **Cyberint alert CVE namespace contract**: (a) in scenario mode,
every `cve_id` field stamped on Cyberint CVE/threat-alert generator records MUST be drawn
from `catalog.device_cves` so that an analyst pivot `enrich nvd(cve_id)` resolves against
the NVD registry for every CVE visible on the Cyberint surface; (b) in baseline/non-scenario
mode, Cyberint `cve_id` values MUST use the `CVE-9999-` synthetic namespace (never used by
the real NVD; established by SEC-001 for `gen_device_cves`) so that synthetic IDs cannot
collide with real advisories. The real Cyberint API surfaces CVE references on alert records
via the `cve_id` / `cve_name` field in CVE-type alert payloads (confirmed by Cyberint portal
documentation and real-API research 2026-06-12).

## Preconditions

1. BC-2.06.019 §Postcondition 1 has been satisfied: a `ScenarioEntityCatalog` has been
   derived from `(seed, org_id)` for the active client config.
2. `ThreatIntelClone` has an existing `fixture_registry: Mutex<HashMap<String, FixtureKey>>`
   (confirmed by `crates/prism-dtu-threatintel/src/state.rs`) and a `FixtureKey::Malicious`
   variant in the `FixtureKey` enum.
3. `NvdClone` has an existing `cve_registry: HashMap<String, CveRecord>` (confirmed by
   `crates/prism-dtu-nvd/src/state.rs` — this is an **immutable** HashMap built at
   construction time and never mutated after, NOT `Mutex`-wrapped). `CveRecord` carries:
   - `cve_id: String`
   - CVSS v3.1 score accessible via: `CveRecord.metrics.cvss_metric_v31: Option<Vec<CvssMetricV31>>`
     → first element `.cvss_data: CvssData` → `.base_score: f64` and `.base_severity: String`
     (e.g., `"HIGH"` for base_score 7.0–8.9, `"CRITICAL"` for ≥9.0; the field is `base_severity`,
     NOT `severity`).
   - There is NO `NvdClone::lookup()` method. The access method is
     `NvdState::lookup_and_count(&self, cve_id: &str) -> Option<CveRecord>` on the state struct.
     Test vectors and verification code must call `state.lookup_and_count(id)`, not a clone-level
     lookup method.
4. `build_clone_pairs` has access to the `ScenarioEntityCatalog` reference before
   constructing ThreatIntel and NVD clones.
5. For clones with `scenario.enabled = false`, the existing `ThreatIntelClone::new()`
   and `NvdClone::new()` constructors are called unchanged (backward-compat path).
6. `INV-PERIMETER-001` is in effect: `prism-dtu-threatintel` and `prism-dtu-nvd` must
   NOT gain dependencies on `prism-spec-engine`, `prism-sensors`, or `prism-query`.
   The `ScenarioEntityCatalog` type itself lives in `prism-dtu-common` — this is the
   only permitted cross-DTU dependency (it is already within the allowed DTU crate family).

## Postconditions

### Postcondition 1 — ThreatIntel registry is pre-populated with scenario IOCs at construction time

When `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog)` is called:

- For every IP in `entities.ioc_ips`: insert `(ip_string, FixtureKey::Malicious)` into
  `fixture_registry`.
- For every domain in `entities.ioc_domains`: insert `(domain_string, FixtureKey::Malicious)`
  into `fixture_registry`.
- For every hash in `entities.ioc_hashes`: insert `(hash_string, FixtureKey::Malicious)`
  into `fixture_registry`.

The insertion occurs at construction time, before the clone starts serving requests. The
`fixture_registry` is mutable only during construction; after `build_clone_pairs` completes,
the registry is effectively immutable for the lifetime of the process (no runtime mutations
except existing `POST /dtu/configure` secondary-override path, which remains unchanged).

The existing default registry entries (Benign lookups, Unknown lookups) are NOT removed or
overwritten. Scenario injection is purely additive.

### Postcondition 2 — ThreatIntel resolves scenario IOCs as known-malicious

For any lookup request to the ThreatIntel clone where the queried indicator (IP, domain,
or hash) is a member of `entities.ioc_ips`, `entities.ioc_domains`, or `entities.ioc_hashes`
respectively:

- The response includes `threat_is_known_malicious = true`.
- The response includes `threat_score >= 75` (the numerical threat confidence score
  associated with `FixtureKey::Malicious` in the existing fixture evaluation logic).
- The response is structurally identical to a standard ThreatIntel lookup response;
  the only difference from the default registry is the `FixtureKey::Malicious` resolution
  path instead of `FixtureKey::Benign` or `FixtureKey::Unknown`.

### Postcondition 3 — NVD registry is pre-populated with scenario CVEs at construction time

When `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> anyhow::Result<Self>`
is called (FALLIBLE — mirrors `NvdClone::new() -> anyhow::Result<Self>`):

- For every CVE ID in `entities.device_cves`: include a synthetic `CveRecord` in the
  initial `cve_registry: HashMap<String, CveRecord>` built at construction time.
  The synthetic record has the following minimum fields:
  - `cve_id`: the CVE ID string from `entities.device_cves`
  - CVSS v3.1 base score (exact path): `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score: f64 >= 7.0`
    (per ADR-036 v2.0 §2.3; default value `8.1`)
  - CVSS v3.1 severity (exact path): `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_severity: String = "HIGH"`
    (`base_severity`, NOT `severity` — these are different fields in `CvssData`)
  - CVSS v3.1 vector string (for plausibility): a deterministic attack-type vector derived
    from the CVE ID index position in `entities.device_cves` (e.g.,
    `"AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N"` for network-exploitable credential-exfiltration).

The `cve_registry` is an **immutable** `HashMap<String, CveRecord>` — it is built once
during `new_with_scenario` by loading base fixtures from `fixtures/cves.json` (same as
`new()`) and then inserting the synthetic scenario CVE records into the initial HashMap
before construction completes. There is NO post-construction mutation; the registry is
never wrapped in a `Mutex`. Injection is purely additive to the base fixture records.

**Note on `NvdClone::new_with_scenario` fallibility:** This constructor is fallible
(returns `anyhow::Result<Self>`) because it extends `NvdClone::new()` which is itself
fallible (reads `fixtures/cves.json` at construction time). Callers in `build_clone_pairs`
must handle the `Result` with `?` propagation.

### Postcondition 4 — NVD resolves scenario CVEs with realistic HIGH CVSS records

For any CVE lookup where the CVE ID is a member of `entities.device_cves`:

- The NVD state method `NvdState::lookup_and_count(&self, cve_id) -> Option<CveRecord>`
  returns `Some(record)` — NOT `None`. (There is no `NvdClone::lookup()` method; the
  lookup is performed on the state struct via `lookup_and_count`.)
- The returned `CveRecord` satisfies:
  `record.metrics.cvss_metric_v31[0].cvss_data.base_score >= 7.0` (exact path per
  ADR-036 v2.0 §2.3 and `crates/prism-dtu-nvd/src/types.rs`)
- The response does not return 404 / "not found" — the CVE ID resolves to a real record.
- The response is structurally valid per the NVD clone's existing response schema.

### Postcondition 5 — Cross-DTU entity coherence: `primary_device_id` is consistent across Armis, CrowdStrike, Claroty

The `ScenarioEntityCatalog` carries two per-format primary device ID fields (ADR-036 v2.0
§2.2): `primary_device_id_cs` (CrowdStrike format) and `primary_device_id_armis` (Armis
format). Both use the same formula `"dev-{org_slug}-{seed}-0"` where
`org_slug = hex(org_id.as_bytes()[0..4])` (8 hex chars). The same `org_slug` value is
derived by the harness and passed as an explicit `org_slug: &str` argument to the Armis
generator — this ensures the catalog slug and the Armis generator slug are identical.

Because all generator-backed clones share the same generator determinism guarantee
(BC-3.4.001) and the same `(seed, org_id)` inputs:

- Armis `/api/v1/devices` response at stage >= 1 (Recon) CONTAINS a device record with
  ID equal to `catalog.primary_device_id_armis`.
- CrowdStrike device query response at stage >= 1 CONTAINS a device/host record with
  host identifier equal to `catalog.primary_device_id_cs`.
- Claroty device query response at stage >= 1 CONTAINS an asset record with
  ID equal to `catalog.primary_device_id_cs` (Claroty uses the same 8-hex-slug derivation
  as CrowdStrike; the harness injects `&catalog.org_slug` consistently to all clones).

The cross-DTU join `SELECT * FROM armis.devices JOIN crowdstrike.devices ON id` using the
`primary_device_id` value MUST yield a non-empty result when executed at stage >= 1.

This postcondition encodes INV-CROSS-DTU-ENTITY-COHERENCE-001. It is a structural
consequence of (a) all generator-backed clones deriving from the same `(seed, org_id)` pair
and (b) the `ScenarioEntityCatalog` using the same ID derivation formula as the generator
(`"dev-{org_slug}-{seed}-0"`). No additional cross-clone synchronization is required.

### Postcondition 6 — Non-scenario lookups are not affected by scenario injection

For any ThreatIntel lookup where the queried indicator is NOT in `entities.ioc_ips`,
`entities.ioc_domains`, or `entities.ioc_hashes`:

- The response is identical to the response that `ThreatIntelClone::new()` (without scenario)
  would return for the same query. Scenario injection is strictly additive.

For any NVD lookup where the CVE ID is NOT in `entities.device_cves`:

- The response is identical to what `NvdClone::new()` would return (either a default record
  if the ID is in the static registry, or a 404 if it is not). Scenario injection does not
  contaminate the general lookup table.

This postcondition encodes INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001. It ensures that
the demo-server ThreatIntel and NVD clones remain valid for non-scenario lookups
(e.g., an analyst querying a real-world CVE ID that happens to be in a TOML-configured
fixture set).

### Postcondition 7 — Enrichment clones are "always ready" regardless of scenario stage

ThreatIntel and NVD receive the full `ScenarioEntityCatalog` at construction time and
pre-populate their registries immediately. They are NOT gated by stage progression. At
any stage (including stage 0 / Baseline), the scenario IOCs and CVEs resolve correctly
in ThreatIntel and NVD.

The stage-progression gating in BC-2.06.019 applies to the operational DTU clones (Armis,
CrowdStrike, Claroty, Cyberint) which control WHEN the IOC/CVE identifiers are surfaced
in device and alert responses. The enrichment DTUs are the lookup destination — they must
be ready to answer at any point the analyst decides to pivot.

### Postcondition 8 — Cyberint Alert CVE Records Use Catalog CVE IDs in Scenario Mode

When a `CyberintClone` is constructed via the scenario path (i.e., the harness calls
`CyberintClone::new_with_scenario(seed, archetype, org_id, Arc<IncidentTimeline>, time_anchor,
&catalog)` with a non-None `ScenarioEntityCatalog`):

- The `cve_id` field on EVERY Cyberint CVE-surface alert record generated by `generate_cves`
  MUST be drawn from `catalog.device_cves`. No `cve_id` on any record may reference a CVE
  outside `catalog.device_cves`.
- Because `catalog.device_cves` contains exactly 3 entries per `build_scenario_entity_catalog`
  (count=3 in `gen_device_cves`), and generator archetypes may produce more CVE records than
  3 (e.g., `CompromisedEndpoint` baseline = 10 records), the mapping is cyclic: for record
  index `i`, use `catalog.device_cves[i % catalog.device_cves.len()]`. This ensures no record
  ever carries an out-of-catalog CVE ID, even when record count exceeds catalog size.
- The `alert_id` / `id` field on each record remains `"alert-{org_slug}-{seed}-{i}"` (unchanged
  from baseline generation — record identity is not scenario-affected).
- All other CVE record fields (score, dates, `_surface`) are computed as in the non-scenario
  path; only the `cve_id` field changes.
- The NVD registry (governed by Postcondition 3 / INV-NVD-CVE-CORRELATION-001) is pre-populated
  with all `catalog.device_cves` at construction time. Therefore every `cve_id` surfaced by
  Cyberint in scenario mode resolves in NVD: `NvdState::lookup_and_count(cve_id) = Some(record)`
  for every Cyberint alert record.

This postcondition encodes INV-CYBERINT-ALERT-CVE-CORRELATION-001. It is the Cyberint-side
complement to Postcondition 3 (NVD injection) and Postcondition 4 (NVD lookup resolution): the
two postconditions together form the complete end-to-end pivot chain
`Cyberint alert cve_id → NVD lookup → HIGH CVSS record`.

**Wiring change in `harness.rs`:** The `CyberintClone` scenario constructor requires the
`&ScenarioEntityCatalog` to be threaded through. The harness currently calls
`CyberintClone::new_with_seed(seed, archetype, org_id)` and does NOT pass the catalog or
timeline (see `harness.rs` lines ~440–443). The implementer MUST:
1. Change the Cyberint scenario branch in `build_clone_pairs` to call
   `CyberintClone::new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline), time_anchor, &catalog)`
   (analogous to the Armis/CrowdStrike/Claroty `new_with_scenario` pattern).
2. Add a new `generate_cves_from_catalog(catalog: &ScenarioEntityCatalog, ...)` code path in
   `prism-dtu-cyberint/src/generator.rs` (or extend the existing `generate_cves` with an
   optional `catalog_cves: Option<&[String]>` parameter) that substitutes catalog IDs for the
   RNG-generated `CVE-2024-{n}` names when in scenario mode.
3. The `CyberintClone::new_with_scenario` constructor signature and return type follow the
   same pattern as `ArmisClone::new_with_scenario`: returns `anyhow::Result<Self>` (consistent
   with ADR-036 v2.3 §2.4 fallible-clone policy).

### Postcondition 9 — Cyberint CVE Records Use Collision-Safe `CVE-9999-` Namespace in Baseline Mode

When a `CyberintClone` is constructed via the non-scenario path (i.e., `new()`,
`new_with_seed()`, or `new_with_access_token()` — no `ScenarioEntityCatalog` available):

- The `cve_id` field on every Cyberint CVE-surface alert record MUST use the format
  `"CVE-9999-{n:04}"` where `n` is drawn from `rng.gen_range(0..10000)`.
- The `CVE-9999-` prefix uses year 9999, which is never used by the real NVD advisory
  database (SEC-001 sentinel; `gen_device_cves` in `prism-dtu-common/src/scenario/mod.rs`
  uses the same sentinel for the same reason).
- These baseline CVEs are intentionally non-pivotable: no `NvdClone` without an active
  scenario will contain an entry for a `CVE-9999-*` ID in its static fixture registry. An
  analyst pivot from a non-scenario Cyberint CVE record to NVD returns "not found" — this is
  the correct and expected outcome in the absence of a correlated scenario.
- The non-pivot behavior MUST NOT produce an error or exception in the demo flow; a 404/
  "not found" NVD response is valid and expected.

**Implementer directive:** change line 340 of `prism-dtu-cyberint/src/generator.rs`:
```
// BEFORE (collision-unsafe):
let cve_name = format!("CVE-2024-{:04}", rng.gen_range(1000u32..9999));
// AFTER (collision-safe):
let cve_name = format!("CVE-9999-{:04}", rng.gen_range(0u32..10000));
```
This single-line fix applies to ALL non-scenario code paths. It is unconditional — the
`CVE-9999-` namespace is required whether or not a scenario is active, to prevent any
possibility of generating a real-namespace CVE ID.

## Invariants

### INV-THREATINTEL-IOC-CORRELATION-001 — All Scenario IOCs Resolve as Malicious in ThreatIntel

```
∀ ip ∈ ScenarioEntityCatalog.ioc_ips:
    ThreatIntel.lookup(ip).threat_is_known_malicious = true
    ∧ ThreatIntel.lookup(ip).threat_score >= 75

∀ domain ∈ ScenarioEntityCatalog.ioc_domains:
    ThreatIntel.lookup(domain).threat_is_known_malicious = true
    ∧ ThreatIntel.lookup(domain).threat_score >= 75

∀ hash ∈ ScenarioEntityCatalog.ioc_hashes:
    ThreatIntel.lookup(hash).threat_is_known_malicious = true
    ∧ ThreatIntel.lookup(hash).threat_score >= 75
```

This invariant holds from construction time through the entire process lifetime. It does
not depend on the current scenario stage (enrichment clones are always-ready per §PC-7).

### INV-NVD-CVE-CORRELATION-001 — All Scenario CVEs Resolve with HIGH CVSS in NVD

```
∀ cve_id ∈ ScenarioEntityCatalog.device_cves:
    NvdState::lookup_and_count(&state, cve_id) = Some(record)
    ∧ record.metrics.cvss_metric_v31[0].cvss_data.base_score: f64 >= 7.0
    ∧ record.metrics.cvss_metric_v31[0].cvss_data.base_severity: String ∈ {"HIGH", "CRITICAL"}
```

Field path (`base_score` and `base_severity`) sourced from `CvssData` struct in
`crates/prism-dtu-nvd/src/types.rs` per ADR-036 v2.0 §2.3. Note: the lookup method is
`NvdState::lookup_and_count` on the state struct, NOT `NvdClone::lookup()` (which does
not exist). `cve_registry` is an immutable `HashMap` — not `Mutex`-wrapped.

This invariant holds from construction time through the entire process lifetime.

### INV-CYBERINT-ALERT-CVE-CORRELATION-001 — Cyberint Alert CVE IDs Are Catalog Members in Scenario Mode; Collision-Safe in All Modes

**Scenario mode (ScenarioEntityCatalog present):**

```
∀ scenario-mode CyberintClone with catalog C:
    ∀ record ∈ generate_cves_output:
        record.cve_id ∈ C.device_cves
        ∧ NvdState::lookup_and_count(&state, record.cve_id) = Some(r)
        ∧ r.metrics.cvss_metric_v31[0].cvss_data.base_score: f64 >= 7.0
```

Every CVE ID on every Cyberint CVE-surface record is drawn from `catalog.device_cves` (cyclic
assignment when record count > catalog size). Because `catalog.device_cves ⊆ NVD cve_registry`
(INV-NVD-CVE-CORRELATION-001), every Cyberint CVE pivots to a HIGH CVSS NVD record without
exception. There are NO orphan CVE IDs on the Cyberint surface when a scenario is active.

**Baseline/non-scenario mode (no ScenarioEntityCatalog):**

```
∀ baseline-mode CyberintClone:
    ∀ record ∈ generate_cves_output:
        record.cve_id matches "^CVE-9999-\d{4}$"
        ∧ record.cve_id ∉ real_NVD_advisory_namespace
```

Every CVE ID uses the `CVE-9999-` synthetic sentinel (year 9999, never used by the real NVD;
consistent with SEC-001 / `gen_device_cves` in `prism-dtu-common/src/scenario/mod.rs`). These
IDs are intentionally non-pivotable — NVD returns "not found" and no error is raised. This is
the correct observable behavior in the absence of an active scenario.

**Universal collision-safety:**

```
∀ CyberintClone (scenario or baseline):
    ∀ record ∈ generate_cves_output:
        record.cve_id NOT matching "^CVE-(202\d|201\d|200\d|199\d)-"
```

No Cyberint-generated CVE ID may use a real calendar year as the year component. This invariant
is statically guaranteed by the implementation: scenario mode draws from `catalog.device_cves`
(which are `CVE-9999-*` per SEC-001); baseline mode generates `CVE-9999-*` directly.

### INV-CROSS-DTU-ENTITY-COHERENCE-001 — `primary_device_id` Appears Across Armis, CrowdStrike, Claroty at Stage >= 1

```
∀ client_config with scenario.enabled = true:
    let id_cs    = ScenarioEntityCatalog.primary_device_id_cs     // CrowdStrike format
    let id_armis = ScenarioEntityCatalog.primary_device_id_armis  // Armis format
    // Both = "dev-{hex(org_id.as_bytes()[0..4])}-{seed}-0"
    // The harness passes the same org_slug to both generators
    at scenario stage >= 1 (Recon):
        id_armis ∈ Armis.devices response (device.id field)
        ∧ id_cs   ∈ CrowdStrike.devices response (device host identifier field)
        ∧ id_cs   ∈ Claroty.devices response (asset id field)
        // Note: Armis generator receives org_slug as explicit &str argument;
        // catalog.primary_device_id_armis uses the same catalog.org_slug derivation,
        // so the Armis generator output matches the catalog value.
```

This invariant is derived from the generator determinism guarantee (BC-3.4.001): all
clones with the same `(seed, org_id)` produce fixture data where the first device's ID
matches `"dev-{org_slug_from_org_id(org_id)}-{seed}-0"`, which matches the
`primary_device_id_*` derivation formula in `ScenarioEntityCatalog` (ADR-036 v2.0 §2.2).

The harness derives `org_slug = org_slug_from_org_id(&org_id)` once and passes it to
all clone constructors consistently (Armis takes it as an explicit `&str` argument;
CrowdStrike and Claroty derive it internally from the OrgId bytes). This ensures the
catalog's entity IDs match each clone's generated IDs.

At stage 0 (Baseline), the primary device may or may not be visible depending on the
`StageMask`. INV-CROSS-DTU-ENTITY-COHERENCE-001 explicitly applies at stage >= 1 (the
`StageMask.primary_device = true` condition in BC-2.06.019 §PC-2).

### INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 — Scenario Injection is Strictly Additive

```
∀ indicator ∉ (ioc_ips ∪ ioc_domains ∪ ioc_hashes):
    ThreatIntel.new_with_scenario(entities).lookup(indicator) =
    ThreatIntel.new().lookup(indicator)

∀ cve_id ∉ device_cves:
    NVD.new_with_scenario(entities).lookup(cve_id) =
    NVD.new().lookup(cve_id)
```

Scenario injection does not replace or modify any pre-existing registry entries. It only
inserts new entries. This invariant ensures that demo-context enrichment lookups for
non-scenario indicators return the same results as a non-scenario enrichment clone would.

### INV-PERIMETER-COMPLIANCE-001 — Enrichment Constructors Accept Only `prism-dtu-common` Types

`ThreatIntelClone::new_with_scenario` and `NvdClone::new_with_scenario` accept
`&ScenarioEntityCatalog` as their only scenario-related parameter. `ScenarioEntityCatalog`
is defined in `prism-dtu-common` (behind `feature = "fixture-gen"`). The constructors
MUST NOT import any type from `prism-spec-engine`, `prism-sensors`, or `prism-query`.

**`prism-core` is permitted and on the INV-PERIMETER-001 allow-list.** The `fixture-gen`
feature in `prism-dtu-common` transitively enables `prism-core`. This is safe:
INV-PERIMETER-001 prohibits `prism-dtu-*` crates from depending on `prism-spec-engine`,
`prism-sensors`, or `prism-query` — NOT on `prism-core`. Both `prism-dtu-armis` and
`prism-dtu-crowdstrike` already depend on `prism-core` directly. `ScenarioEntityCatalog`
carries no `prism-spec-engine` dependency. The perimeter holds.

Adding `fixture-gen = ["prism-dtu-common/fixture-gen"]` to `prism-dtu-threatintel/Cargo.toml`
and `prism-dtu-nvd/Cargo.toml` is the required Cargo change (per ADR-036 v2.0 §2.3);
neither crate currently has `chrono` or `fixture-gen` declared.

The DTU perimeter is enforced **structurally**: `prism-dtu-threatintel` and `prism-dtu-nvd`
declare no dependency on `prism-spec-engine`, `prism-sensors`, or `prism-query` in their
`Cargo.toml` files, so any forbidden `use` statement is an ordinary E0432 compile error caught
by the standard workspace build. The `tests/external/perimeter-violation/` gate (established
by S-PLUGIN-PREREQ-A, BC-2.11.006) covers the **prism-query pub-API perimeter only** and does
NOT reference the DTU crates; it is irrelevant to this invariant. No separate compile-fail gate
exists or is needed for the DTU perimeter — Cargo dependency declarations ARE the enforcement
mechanism.

### INV-CONSTRUCTION-TIME-INJECTION-001 — Registry Injection Occurs Only at Construction Time

The scenario-correlated entries are inserted into `fixture_registry` / `cve_registry`
exactly once: during the call to `new_with_scenario`. After `build_clone_pairs` returns,
no further registry mutation occurs in the scenario injection path.

This invariant eliminates the race condition that would exist if injection were deferred
to request time: requests arriving between construction and deferred injection would
return non-malicious results for scenario IOCs. Construction-time injection is the only
correct implementation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-020-001 | `scenario.enabled = false` for ThreatIntel | `ThreatIntelClone::new()` called; no scenario injection; default registry only (INV-SCENARIO-DISABLED-COMPAT-001 from BC-2.06.019) |
| EC-020-002 | `scenario.enabled = true` for ThreatIntel but catalog has `ioc_ips = []` (empty) | No entries inserted for IPs; no error; catalog fields that are non-empty still receive injection normally |
| EC-020-003 | Same IOC appears in both `ioc_ips` and (hypothetically) the default registry as Benign | Scenario injection inserts `FixtureKey::Malicious` for that IP; HashMap::insert overwrites the prior value; the scenario-injected `Malicious` entry wins. Benign entries for non-scenario IPs are unchanged. |
| EC-020-004 | NVD lookup for a CVE ID in `device_cves` at scenario stage 0 (Baseline) | CVE resolves correctly with `base_score >= 7.0`; enrichment is always-ready (§PC-7); stage gate does not apply to NVD |
| EC-020-005 | NVD lookup for a CVE ID NOT in `device_cves` | Response is identical to `NvdClone::new()` response for the same query (INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001) |
| EC-020-006 | ThreatIntel lookup for an IP not in `ioc_ips` | Response is identical to non-scenario `ThreatIntelClone::new()` response (INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001) |
| EC-020-007 | Cross-DTU join at stage 0 (Baseline): `SELECT * FROM armis.devices JOIN crowdstrike.devices ON id` using `primary_device_id` | May return empty result (primary device not yet visible at stage 0 per BC-2.06.019 StageMask); this is correct — the enrichment is ready, but the device is not yet surfaced |
| EC-020-008 | Cross-DTU join at stage 1 (Recon): same join query | Returns non-empty result containing `primary_device_id` (INV-CROSS-DTU-ENTITY-COHERENCE-001) |
| EC-020-009 | Process restart with same `demo.toml`; ThreatIntel and NVD reconstructed | Same `ScenarioEntityCatalog` derived (same `seed`, same `org_id`); same entries injected; lookup results byte-identical to pre-restart |
| EC-020-010 | `ThreatIntelClone::new_with_scenario` called with `ioc_ips` containing 50+ IPs | All 50+ IPs inserted at construction time; no cap; `HashMap` handles arbitrary count; construction succeeds |
| EC-020-011 | `scenario.enabled = true` for operational DTUs but `scenario.enabled = false` for ThreatIntel | ThreatIntel uses static default registry; scenario IOCs will NOT resolve as malicious in this config. Operator's configuration choice. No error at `build_clone_pairs` level (partial scenario activation is permitted). |
| EC-020-012 | Cyberint clone in scenario mode; `CompromisedEndpoint` archetype generates 10 CVE records; `catalog.device_cves` has 3 entries | Records at indices 0,1,2 use `device_cves[0]`, `device_cves[1]`, `device_cves[2]`; records at indices 3,4,5 use `device_cves[0]`, `device_cves[1]`, `device_cves[2]` (cyclic); records at indices 6,7,8,9 cycle again. All 10 records' `cve_id` values are members of `catalog.device_cves`. INV-CYBERINT-ALERT-CVE-CORRELATION-001 holds. |
| EC-020-013 | Cyberint clone in scenario mode; `catalog.device_cves` is empty (hypothetical — `gen_device_cves` always generates count=3, so this cannot occur in production, but contract must be safe) | If `catalog.device_cves.is_empty()`, no CVE records are produced (generator returns empty `Vec`). No panic; no orphan CVE emission. Build assertion in `build_scenario_entity_catalog` (`device_cves must be non-empty`) prevents this in the normal path. |
| EC-020-014 | Cyberint clone in baseline mode (no scenario); analyst pivots from a Cyberint CVE record to NVD | NVD returns 404 / "not found". This is correct and expected. No error in the demo flow. `CVE-9999-*` IDs are intentionally not in the NVD static fixture registry. |
| EC-020-015 | Cyberint clone in baseline mode; `generate_cves` produces `cve_id = "CVE-2024-1234"` (old pre-fix behavior) | VIOLATION of INV-CYBERINT-ALERT-CVE-CORRELATION-001. The implementer fix (PC-9 / line 340 of `generator.rs`) prevents this; the regression test TV-020-011 detects any reintroduction. |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-020-001 | ThreatIntel clone constructed with scenario; lookup for `ioc_ips[0]` | `threat_is_known_malicious = true`; `threat_score >= 75` (INV-THREATINTEL-IOC-CORRELATION-001) | happy-path |
| TV-020-002 | ThreatIntel clone constructed with scenario; lookup for `ioc_domains[0]` | `threat_is_known_malicious = true`; `threat_score >= 75` | happy-path |
| TV-020-003 | ThreatIntel clone constructed with scenario; lookup for `ioc_hashes[0]` | `threat_is_known_malicious = true`; `threat_score >= 75` | happy-path |
| TV-020-004 | NVD clone constructed with scenario; lookup for `device_cves[0]` | `base_score >= 7.0`; response is not 404 (INV-NVD-CVE-CORRELATION-001) | happy-path |
| TV-020-005 | ThreatIntel clone constructed with scenario; lookup for a non-scenario IP `"192.0.2.1"` | Response identical to `ThreatIntelClone::new().lookup("192.0.2.1")` (INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001) | passthrough |
| TV-020-006 | NVD clone constructed with scenario; lookup for CVE ID `"CVE-2020-99999"` (not in `device_cves`) | Response identical to `NvdClone::new().lookup("CVE-2020-99999")` (INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001) | passthrough |
| TV-020-007 | Cross-DTU scenario: 3 orgs, `seed=100 / 200 / 300`; for each org, query `primary_device_id` across Armis, CrowdStrike, Claroty at stage 1 | For each org, the same `primary_device_id` appears in all 3 clone responses; cross-org device IDs are disjoint (INV-CROSS-DTU-ENTITY-COHERENCE-001 + BC-2.06.018 INV-DISTINCT-DATA-001) | integration |
| TV-020-008 | ThreatIntel clone constructed WITHOUT scenario (`new()`); lookup for `ioc_ips[0]` (derived from seed=100) | Lookup returns Benign or Unknown (entry was never injected); confirms scenario injection is not active on static-path clone | regression |
| TV-020-009 | NVD clone constructed WITHOUT scenario; lookup for `device_cves[0]` | Returns 404 or default record (not the scenario-injected HIGH-severity record) | regression |
| TV-020-010 | All IOCs from one org's catalog injected into ThreatIntel; verify no IOC from a DIFFERENT org's catalog (seed=200) resolves as Malicious in the seed=100 ThreatIntel clone | Non-scenario org IOCs do not resolve as Malicious in this clone (no cross-org contamination; each clone is a separate instance) | isolation |
| TV-020-011 | Cyberint clone constructed with `new_with_seed()` (baseline, no scenario); inspect all generated CVE records | Every `cve_id` matches `^CVE-9999-\d{4}$`; no `cve_id` matches `^CVE-202\d-` or any real-year pattern (INV-CYBERINT-ALERT-CVE-CORRELATION-001 baseline mode; regression against pre-fix `CVE-2024-*` behavior) | regression |
| TV-020-012 | Cyberint clone constructed with scenario (seed=100, `CompromisedEndpoint`, `catalog.device_cves = ["CVE-9999-00001","CVE-9999-00002","CVE-9999-00003"]`); inspect all generated CVE records | Every `cve_id` is one of `{"CVE-9999-00001","CVE-9999-00002","CVE-9999-00003"}`; no `cve_id` is outside this set (INV-CYBERINT-ALERT-CVE-CORRELATION-001 scenario mode) | happy-path |
| TV-020-013 | Cyberint scenario clone (same catalog as TV-020-012); for each `cve_id` in generated CVE records, call `NvdState::lookup_and_count(cve_id)` | Every call returns `Some(record)` with `base_score >= 7.0`; no NVD "not found" response (end-to-end pivot chain: Cyberint surface → NVD resolution; PC-8 + INV-CYBERINT-ALERT-CVE-CORRELATION-001) | integration |
| TV-020-014 | Cyberint scenario clone with `CompromisedEndpoint` archetype (10 CVE records) and `catalog.device_cves` with 3 entries; collect all `cve_id` values | Set of distinct `cve_id` values equals exactly `catalog.device_cves` (cyclic assignment; no extra IDs introduced); count of records is 10; distribution is cyclic over 3 catalog entries (EC-020-012) | happy-path |
| TV-020-015 | Cyberint baseline clone; analyst pivots from CVE record `cve_id = "CVE-9999-12345"` to NVD; NVD is the non-scenario `NvdClone::new()` | NVD returns 404 / "not found"; no exception or error in demo flow (EC-020-014; intentionally non-pivotable in baseline mode) | expected-miss |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-020-A | `∀ ip ∈ ioc_ips: ThreatIntel.lookup(ip).threat_is_known_malicious = true ∧ threat_score >= 75` | unit test (TV-020-001) |
| VP-020-B | `∀ domain ∈ ioc_domains: ThreatIntel.lookup(domain).threat_is_known_malicious = true` | unit test (TV-020-002) |
| VP-020-C | `∀ hash ∈ ioc_hashes: ThreatIntel.lookup(hash).threat_is_known_malicious = true` | unit test (TV-020-003) |
| VP-020-D | `∀ cve_id ∈ device_cves: NvdState::lookup_and_count(&state, cve_id) = Some(record) ∧ record.metrics.cvss_metric_v31[0].cvss_data.base_score >= 7.0` | unit test (TV-020-004) |
| VP-020-E | Non-scenario lookups are passthrough: `ThreatIntel.new_with_scenario.lookup(non_ioc) = ThreatIntel.new().lookup(non_ioc)` | unit test (TV-020-005, TV-020-006) |
| VP-020-F | Cross-DTU entity coherence: `primary_device_id` in Armis ∩ CrowdStrike ∩ Claroty at stage 1 | integration test (TV-020-007) |
| VP-020-G | Static path (no scenario) does not inject scenario entries | regression test (TV-020-008, TV-020-009) |
| VP-020-H | No cross-org contamination: org-A IOCs do not resolve as Malicious in org-B ThreatIntel clone | isolation test (TV-020-010) |
| VP-020-I | Baseline-mode Cyberint CVE records use `CVE-9999-` namespace; no real-year CVE IDs emitted | regression test (TV-020-011) |
| VP-020-J | Scenario-mode Cyberint CVE records use only `catalog.device_cves` IDs (INV-CYBERINT-ALERT-CVE-CORRELATION-001) | unit test (TV-020-012) |
| VP-020-K | End-to-end pivot: every scenario-mode Cyberint CVE record resolves in NVD with `base_score >= 7.0` | integration test (TV-020-013) |
| VP-020-L | Cyclic catalog assignment: record count > catalog size does not introduce out-of-catalog CVE IDs | unit test (TV-020-014) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-036 ("Multi-Tenant DTU Test Harness (Internal)") per capabilities.md §CAP-036 |
| Capability Anchor Justification | CAP-036 ("Multi-Tenant DTU Test Harness (Internal)") per capabilities.md §CAP-036 — this BC specifies the enrichment-correlation wiring in `build_clone_pairs`: how the `ScenarioEntityCatalog` (produced by the harness's scenario coordination layer in BC-2.06.019) is used to pre-populate ThreatIntel and NVD registries. This is CAP-036 harness orchestration scope — specifically, the harness distributing the entity catalog to enrichment clones via scenario-aware constructors. It is not CAP-039 scope (generator internals); ThreatIntel and NVD have no generator. The enrichment constructors (`new_with_scenario`) are analogous in form to the generator-backed `new_with_seed` constructors governed by BC-2.06.018 — both are harness-layer wiring to specialized clone constructors. |
| L2 Domain Invariants | N/A (demo-server enrichment wiring; no DI-NNN in L2 domain spec maps to this concern) |
| Architecture Module | SS-01 (Sensor Adapters) per ARCH-INDEX.md; `prism-dtu-demo-server` (harness wiring), `prism-dtu-threatintel`, `prism-dtu-nvd`, and `prism-dtu-cyberint` are the primary implementation sites |
| Governing ADR | ADR-036 §2.3 ("Enrichment clones: static lookup injection, not a new generator") — this BC encodes the lookup injection mechanism as testable contracts |
| Stories | S-DEMO-DTU-LIVE-SCENARIO-001, S-DEMO-DTU-LIVE-SCENARIO-001-B |
| Upstream BCs | BC-2.06.019 (Scenario Progression — produces the `ScenarioEntityCatalog` that this BC's constructors consume); BC-2.06.018 (Config-Time Data Seeding — enrichment clone construction follows the same wiring pattern) |

## Related BCs

- BC-2.06.019 — depends on (this BC consumes the `ScenarioEntityCatalog` produced by BC-2.06.019 §Postcondition 1; both BCs are wired in the same `build_clone_pairs` coordination burst)
- BC-2.06.018 — sibling (both govern demo-server config-wiring to specialized clone constructors; BC-2.06.018 governs generator-backed clones via `new_with_seed`, this BC governs enrichment clones via `new_with_scenario`)
- BC-3.4.001 — referenced by (INV-CROSS-DTU-ENTITY-COHERENCE-001 depends on generator determinism: same `(seed, org_id)` produces same `device[0].id = primary_device_id` in all generator-backed clones)

## Architecture Anchors

- `crates/prism-dtu-demo-server/src/harness.rs` — `build_clone_pairs`: passes `&ScenarioEntityCatalog` to `ThreatIntelClone::new_with_scenario` and `NvdClone::new_with_scenario` after constructing the catalog (ADR-036 §2.4 step 4)
- `crates/prism-dtu-threatintel/src/state.rs` — `fixture_registry: Mutex<HashMap<String, FixtureKey>>` — injection target; `FixtureKey::Malicious` is the variant used for scenario IOC injection
- `crates/prism-dtu-nvd/src/state.rs` — `cve_registry: HashMap<String, CveRecord>` (IMMUTABLE, not Mutex-wrapped) — injection target; `CveRecord` with `metrics.cvss_metric_v31[0].cvss_data.base_score: f64 >= 7.0` and `.base_severity: String = "HIGH"` is the injected value type. Lookup via `NvdState::lookup_and_count(&self, cve_id: &str) -> Option<CveRecord>` (not a clone-level method).
- `crates/prism-dtu-common/src/scenario/mod.rs` — `ScenarioEntityCatalog` definition and `gen_device_cves` (ADR-036 §2.2); `catalog.device_cves` uses `CVE-9999-{:05}` format per `gen_device_cves` doc comment (line 429: `"CVE-9999-{seq:05}"`) and SEC-001 test (`gen_device_cves must emit CVE-9999-{{seq:05}} format`); 5-digit suffix; distinct from the Cyberint BASELINE generator (`crates/prism-dtu-cyberint/src/generator.rs:389`) which uses `CVE-9999-{:04}` (4-digit) per PC-9/`^CVE-9999-\d{4}$`/TV-020-011; scenario-mode Cyberint CVEs are drawn FROM this 5-digit catalog (PC-8), so scenario-mode `cve_id` values are 5-digit; baseline-mode Cyberint CVEs are 4-digit; both are collision-safe (`CVE-9999-` namespace); TV-020-012 confirms catalog IDs are 5-digit (`"CVE-9999-00001"` etc.); these are the CVE IDs injected into NVD (PC-3) and drawn by Cyberint in scenario mode (PC-8)
- `crates/prism-dtu-cyberint/src/generator.rs` — `generate_cves`: line ~340 generates `cve_name`; v1.3 amends this to use `CVE-9999-` namespace in baseline mode and `catalog.device_cves` in scenario mode (INV-CYBERINT-ALERT-CVE-CORRELATION-001)
- `tests/external/perimeter-violation/` — compile-fail gate that enforces the **prism-query pub-API perimeter** (BC-2.11.006) ONLY; this crate depends on `prism-query` + `prism-core` and does NOT reference any `prism-dtu-*` crate. The DTU perimeter required by `INV-PERIMETER-001` is enforced structurally: `prism-dtu-threatintel/Cargo.toml` and `prism-dtu-nvd/Cargo.toml` declare no dependency on `prism-spec-engine`, `prism-sensors`, or `prism-query`, making any forbidden `use` an ordinary E0432 compile error in the workspace build.

## Story Anchor

S-DEMO-DTU-LIVE-SCENARIO-001, S-DEMO-DTU-LIVE-SCENARIO-001-B

## VP Anchors

VP-020-A through VP-020-L (above) — verified by integration/unit tests in S-DEMO-DTU-LIVE-SCENARIO-001 (original enrichment-correlation delivery) and S-DEMO-DTU-LIVE-SCENARIO-001-B (AC implementations for all 12 VPs)

## BC Changelog

| Version | Change |
|---------|--------|
| v1.6 | BPRL-P24-01 2026-06-13 — INV-PERIMETER-COMPLIANCE-001 + Architecture Anchors enforcement-mechanism prose corrected. The DTU perimeter (threatintel/nvd must not import prism-spec-engine/prism-sensors/prism-query) is enforced **structurally** via Cargo.toml dependency declarations + ordinary E0432 compile errors in the workspace build. The previous prose incorrectly stated that `tests/external/perimeter-violation/` "continues to hold after these constructors are added"; that gate covers the prism-query pub-API perimeter only (BC-2.11.006) and does not reference the DTU crates. Architecture Anchors bullet for `tests/external/perimeter-violation/` corrected to describe its actual scope (prism-query only) and explicitly state that the DTU perimeter is Cargo-structural. Invariant SEMANTICS (no forbidden imports in threatintel/nvd) are UNCHANGED. User-directed prose-correction; no gate build. |
| v1.5 | BPRL-P22-01 2026-06-13 — exhaustive summary-count sweep (no behavior change). VP Anchors prose: `VP-020-A through VP-020-H` → `VP-020-A through VP-020-L`; `all 8 VPs` → `all 12 VPs`. (Orchestrator-caught: the pass-22 sweep had mis-changed the catalog `{:05}` to `{:04}` by conflating it with the Cyberint baseline generator; reverted — catalog `gen_device_cves` is 5-digit per `gen_device_cves` doc comment line 429 + SEC-001 test + TV-020-012; only the Cyberint BASELINE generator at `generator.rs:389` is 4-digit per PC-9/`^CVE-9999-\d{4}$`/TV-020-011; these are two distinct generators with different digit widths.) All other count/range statements in the document are correct: PC-1..9 (9 postconditions), INVs (7 invariants), TV-020-001..015 (15 test vectors), EC-020-001..015 (15 edge cases), VP-020-A..L (12 VPs) — confirmed by exhaustive grep sweep per POL-32. |
| v1.4 | BPRL-P14-01 2026-06-13 — spec-internal contradiction fix (no behavior change; code was already correct). PC-9 range literal `rng.gen_range(0..100000)` → `rng.gen_range(0..10000)`. Implementer-directive code block range literal `rng.gen_range(0u32..100000)` → `rng.gen_range(0u32..10000)`. Both now consistent with INV-CYBERINT-ALERT-CVE-CORRELATION-001 baseline clause (`^CVE-9999-\d{4}$`), TV-020-011 (`Every cve_id matches ^CVE-9999-\d{4}$`), and the `{:04}` format specifier. The old `100000` upper bound allowed 5-digit suffixes (n ≥ 10000) for ~90% of draws, violating the `\d{4}` invariant. `rng.gen_range(0..10000)` is bounded to [0, 9999] → `{:04}` always produces exactly 4 digits. Shipped code at `generator.rs:389` was already correct (`0u32..10000`). |
| v1.3 | D-1117 2026-06-12 — Cyberint alert CVE ↔ NVD correlation + collision-safety (human-directed, production-grade). Added: Description paragraph scoping Cyberint CVE namespace contract. Postcondition 8 (scenario mode: all Cyberint `cve_id` fields drawn from `catalog.device_cves`; cyclic assignment for record count > catalog size; explicit harness wiring change in `harness.rs` and `generator.rs`). Postcondition 9 (baseline/non-scenario mode: `CVE-9999-` namespace required; intentionally non-pivotable; single-line fix for `generator.rs` line 340). INV-CYBERINT-ALERT-CVE-CORRELATION-001 (scenario-mode CVE membership + NVD resolution + universal collision-safety in all modes). Edge cases EC-020-012 through EC-020-015. Test vectors TV-020-011 through TV-020-015. Verification properties VP-020-I through VP-020-L. Architecture Anchors extended with `prism-dtu-cyberint/src/generator.rs` and `prism-dtu-common/src/scenario/mod.rs`. `crates:` frontmatter extended with `prism-dtu-cyberint`. `inputs:` frontmatter extended with `crates/prism-dtu-cyberint/src/generator.rs` and `crates/prism-dtu-common/src/scenario/mod.rs`. H1 title updated to reflect Cyberint scope. |
| v1.2 | PO micro-burst 2026-06-12 — OBS-2 anchor drift fixed. Stories traceability row, Story Anchor section, and VP Anchors section updated to include S-DEMO-DTU-LIVE-SCENARIO-001-B (frontmatter `anchored_stories` already included -B per D-1090 v6.28 backlink; body sections were stale). INV-CROSS-DTU-ENTITY-COHERENCE-001 now receives downstream enforcement from BC-2.06.019 PRE-6 / E-DEMO-006 guard (org_id equality) — no change to this BC's invariant text required (the invariant remains structurally enforced by generator determinism; the new guard prevents the misconfiguration that would silently violate it). |
| v1.1 | ADR-036 v2.0 / D-1078 substrate-reconciliation corrections. Replaced `NvdClone::lookup()` (which does not exist) with `NvdState::lookup_and_count(&self, cve_id) -> Option<CveRecord>` in Precondition 3, Postcondition 4, INV-NVD-CVE-CORRELATION-001, VP-020-D, and Architecture Anchors. Corrected CVSS access path to `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score: f64` and `.base_severity: String` (NOT `.severity`) per `crates/prism-dtu-nvd/src/types.rs` CvssData struct; updated all occurrences. Noted `NvdState.cve_registry` is IMMUTABLE `HashMap` (not Mutex-wrapped); `new_with_scenario` builds the initial map including scenario CVEs at construction and never mutates after. Clarified `NvdClone::new_with_scenario` is FALLIBLE (`anyhow::Result<Self>`) mirroring `NvdClone::new()`. Noted `ThreatIntelClone::new_with_scenario` is INFALLIBLE (`Self`) mirroring `ThreatIntelClone::new()`. Updated INV-CROSS-DTU-ENTITY-COHERENCE-001 to use split `primary_device_id_cs` / `primary_device_id_armis` fields from `ScenarioEntityCatalog` (ADR-036 v2.0 §2.2); documented Armis's explicit `org_slug: &str` injection pattern. Extended INV-PERIMETER-COMPLIANCE-001 to explicitly confirm `prism-core` is on the INV-PERIMETER-001 allow-list (transitive via `prism-dtu-common/fixture-gen`); noted required Cargo.toml additions for `prism-dtu-threatintel` and `prism-dtu-nvd`. Postcondition 5 updated to reference `primary_device_id_cs` for CrowdStrike/Claroty and `primary_device_id_armis` for Armis. lifecycle_status remains draft. Invariant semantics (threshold values, IOC resolution, additive injection) unchanged. |
| v1.0 | Initial authoring. ADR-036 ACCEPTED 2026-06-09. BC-2.06.020 namespace confirmed (next-available after BC-2.06.019). Subsystem: SS-01. Capability: CAP-036 — enrichment wiring is harness-layer orchestration. EC-020-003 documents HashMap insert semantics for potential IOC collision between scenario injection and prior Benign entries. EC-020-011 explicitly permits partial scenario activation (enrichment disabled while operational DTUs enabled) as a valid operator configuration. INV-CONSTRUCTION-TIME-INJECTION-001 added to prevent deferred-injection race condition that would produce incorrect lookup results during concurrent startup request handling. |
