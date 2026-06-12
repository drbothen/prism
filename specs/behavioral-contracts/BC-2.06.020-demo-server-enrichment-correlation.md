---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.020"
version: "1.2"
status: draft
lifecycle_status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-036"
introduced: "2026-06-09"
modified: "2026-06-09"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-DEMO-DTU-LIVE-SCENARIO-001, S-DEMO-DTU-LIVE-SCENARIO-001-B]
verifying_vps: []
crates: [prism-dtu-demo-server, prism-dtu-common, prism-dtu-threatintel, prism-dtu-nvd, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty]
inputs:
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - "crates/prism-dtu-threatintel/src/state.rs"
  - "crates/prism-dtu-nvd/src/state.rs"
  - "crates/prism-dtu-armis/src/state.rs"
  - "crates/prism-dtu-crowdstrike/src/state.rs"
  - "crates/prism-dtu-common/src/generator/archetype.rs"
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

# BC-2.06.020: Demo-Server Enrichment Correlation — Scenario IOCs Resolve in ThreatIntel; Scenario CVEs Resolve in NVD

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

The `tests/external/perimeter-violation/` compile-fail gate (established by S-PLUGIN-PREREQ-A)
continues to hold after these constructors are added. No new perimeter-violation exclusions
are required.

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

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-036 ("Multi-Tenant DTU Test Harness (Internal)") per capabilities.md §CAP-036 |
| Capability Anchor Justification | CAP-036 ("Multi-Tenant DTU Test Harness (Internal)") per capabilities.md §CAP-036 — this BC specifies the enrichment-correlation wiring in `build_clone_pairs`: how the `ScenarioEntityCatalog` (produced by the harness's scenario coordination layer in BC-2.06.019) is used to pre-populate ThreatIntel and NVD registries. This is CAP-036 harness orchestration scope — specifically, the harness distributing the entity catalog to enrichment clones via scenario-aware constructors. It is not CAP-039 scope (generator internals); ThreatIntel and NVD have no generator. The enrichment constructors (`new_with_scenario`) are analogous in form to the generator-backed `new_with_seed` constructors governed by BC-2.06.018 — both are harness-layer wiring to specialized clone constructors. |
| L2 Domain Invariants | N/A (demo-server enrichment wiring; no DI-NNN in L2 domain spec maps to this concern) |
| Architecture Module | SS-01 (Sensor Adapters) per ARCH-INDEX.md; `prism-dtu-demo-server` (harness wiring), `prism-dtu-threatintel`, and `prism-dtu-nvd` are the primary implementation sites |
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
- `crates/prism-dtu-common/src/scenario/` — `ScenarioEntityCatalog` definition (ADR-036 §2.2); the entity catalog is the data contract between BC-2.06.019 (producer) and this BC (consumer)
- `tests/external/perimeter-violation/` — compile-fail gate enforcing `INV-PERIMETER-001`; `new_with_scenario` constructors must not introduce forbidden dependencies

## Story Anchor

S-DEMO-DTU-LIVE-SCENARIO-001, S-DEMO-DTU-LIVE-SCENARIO-001-B

## VP Anchors

VP-020-A through VP-020-H (above) — verified by integration/unit tests in S-DEMO-DTU-LIVE-SCENARIO-001 (original enrichment-correlation delivery) and S-DEMO-DTU-LIVE-SCENARIO-001-B (AC implementations for all 8 VPs)

## BC Changelog

| Version | Change |
|---------|--------|
| v1.2 | PO micro-burst 2026-06-12 — OBS-2 anchor drift fixed. Stories traceability row, Story Anchor section, and VP Anchors section updated to include S-DEMO-DTU-LIVE-SCENARIO-001-B (frontmatter `anchored_stories` already included -B per D-1090 v6.28 backlink; body sections were stale). INV-CROSS-DTU-ENTITY-COHERENCE-001 now receives downstream enforcement from BC-2.06.019 PRE-6 / E-DEMO-006 guard (org_id equality) — no change to this BC's invariant text required (the invariant remains structurally enforced by generator determinism; the new guard prevents the misconfiguration that would silently violate it). |
| v1.1 | ADR-036 v2.0 / D-1078 substrate-reconciliation corrections. Replaced `NvdClone::lookup()` (which does not exist) with `NvdState::lookup_and_count(&self, cve_id) -> Option<CveRecord>` in Precondition 3, Postcondition 4, INV-NVD-CVE-CORRELATION-001, VP-020-D, and Architecture Anchors. Corrected CVSS access path to `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score: f64` and `.base_severity: String` (NOT `.severity`) per `crates/prism-dtu-nvd/src/types.rs` CvssData struct; updated all occurrences. Noted `NvdState.cve_registry` is IMMUTABLE `HashMap` (not Mutex-wrapped); `new_with_scenario` builds the initial map including scenario CVEs at construction and never mutates after. Clarified `NvdClone::new_with_scenario` is FALLIBLE (`anyhow::Result<Self>`) mirroring `NvdClone::new()`. Noted `ThreatIntelClone::new_with_scenario` is INFALLIBLE (`Self`) mirroring `ThreatIntelClone::new()`. Updated INV-CROSS-DTU-ENTITY-COHERENCE-001 to use split `primary_device_id_cs` / `primary_device_id_armis` fields from `ScenarioEntityCatalog` (ADR-036 v2.0 §2.2); documented Armis's explicit `org_slug: &str` injection pattern. Extended INV-PERIMETER-COMPLIANCE-001 to explicitly confirm `prism-core` is on the INV-PERIMETER-001 allow-list (transitive via `prism-dtu-common/fixture-gen`); noted required Cargo.toml additions for `prism-dtu-threatintel` and `prism-dtu-nvd`. Postcondition 5 updated to reference `primary_device_id_cs` for CrowdStrike/Claroty and `primary_device_id_armis` for Armis. lifecycle_status remains draft. Invariant semantics (threshold values, IOC resolution, additive injection) unchanged. |
| v1.0 | Initial authoring. ADR-036 ACCEPTED 2026-06-09. BC-2.06.020 namespace confirmed (next-available after BC-2.06.019). Subsystem: SS-01. Capability: CAP-036 — enrichment wiring is harness-layer orchestration. EC-020-003 documents HashMap insert semantics for potential IOC collision between scenario injection and prior Benign entries. EC-020-011 explicitly permits partial scenario activation (enrichment disabled while operational DTUs enabled) as a valid operator configuration. INV-CONSTRUCTION-TIME-INJECTION-001 added to prevent deferred-injection race condition that would produce incorrect lookup results during concurrent startup request handling. |
