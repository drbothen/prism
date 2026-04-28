---
document_type: adr
adr_id: ADR-009
title: "Multi-Tenant Data Generator — Hybrid Archetype Catalog + Deterministic Generator"
status: PROPOSED
date: 2026-04-27
wave: 3
phase: 3.A
version: "0.9"
authors: [architect]
related_decisions: [D-043, D-045, D-054, D-055, D-056, D-059]
related_adrs: [ADR-006, ADR-010]
related_bcs_planned: [BC-3.4.001, BC-3.4.002, BC-3.4.003, BC-3.4.004]
anchored_capabilities: [CAP-039]
subsystems_affected: [SS-01, SS-05, SS-06]
supersedes: null
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
inputs:
  - crates/prism-dtu-common/src/seed.rs
  - crates/prism-dtu-common/src/config.rs
  - crates/prism-dtu-common/src/fixture.rs
  - crates/prism-dtu-claroty/src/state.rs
  - crates/prism-dtu-crowdstrike/src/state.rs
  - .references/poller-bear/docs/specs.json
  - .references/poller-express/docs/specs/
  - .factory/STATE.md (D-043, D-045)
---

# ADR-009: Multi-Tenant Data Generator — Hybrid Archetype Catalog + Deterministic Generator

## Status

PROPOSED — decision D-043 recorded. BCs authored at v0.3+ during Phase 3.A;
see BC-INDEX. Implementation BLOCKED until Phase 3.A converges (D-045).

---

## 1. Context

### 1.1 Multi-Tenant Test Data Problem

Wave 3 introduces `OrgId`-keyed multi-tenancy (ADR-006). The existing DTU behavioral
clones — Claroty, CrowdStrike, Armis, Cyberint — each ship with static JSON fixture
files loaded from `crates/prism-dtu-*/fixtures/`. Under single-tenant DTU testing,
one fixture set per sensor type was sufficient. Under multi-tenant testing, the
harness must generate independent, org-tagged fixture sets for N simultaneous customers,
each with a distinct `OrgId`.

Three problems with the current static-fixture model at multi-tenant scale:

1. **Cross-tenant leakage is invisible.** Static fixtures share device IDs across
   orgs (e.g., `claroty-device-001` appears in every clone's fixture). A keying bug
   in `ClarotyState::tag_store` (currently `HashMap<String, HashSet<String>>` at
   `crates/prism-dtu-claroty/src/state.rs:24`) that merges two orgs' data would
   not be caught because both orgs present identical device IDs. Leakage detection
   requires that each org has a distinct, traceable device ID namespace.

2. **Scenario coverage is rigid.** Static fixtures capture one "happy path" snapshot.
   Testing pagination edge cases, schema drift, auth outages, or large-scale enumeration
   requires hand-crafting separate fixture files for each scenario per sensor type —
   an N×M maintenance burden (N sensors × M scenarios).

3. **Determinism is implicit, not guaranteed.** The existing `seeded_rng` function
   (`crates/prism-dtu-common/src/seed.rs:9`) provides a `ChaCha20Rng` seeded from
   `StubConfig::seed`. However, fixture content itself is not generated — it is loaded
   from disk (`crates/prism-dtu-common/src/fixture.rs`). There is no contract that
   a given `(seed, org_id, scenario)` triple always produces the same fixture set,
   because the fixture is not derived from those inputs.

Decision D-043 resolves these three problems by adopting a hybrid model: a named
archetype catalog covers scenario semantics, and a deterministic generator produces
org-tagged fixture data from the archetype parameters.

### 1.2 Schema Sources

The Wave 3 generator must produce fixtures conformant to the actual vendor API schemas.
These schemas are vendored from 1898's own repositories — no external attribution is
required:

- **Claroty xDome:** `.references/poller-bear/docs/specs.json` — OpenAPI spec,
  approximately 12,700 lines. This is the authoritative schema for all Claroty
  device, tag, and alert response shapes.
- **Cyberint:** `.references/poller-express/docs/specs/` — four separate OpenAPI
  specs (`alert_api_specs.json`, `asm_assets_api_specs.json`, `cve_api_specs.json`,
  `ioc_api_specs.json`) covering the four Cyberint API surfaces in scope.
- **Armis:** Derived from type definitions in `.references/poller-coaster/internal/`
  or translated from `armis-sdk-go/v2` Go types. Translation to Rust struct shapes
  is required; spec-writer will produce the Rust type shapes as part of BC-3.4
  authoring.
- **CrowdStrike:** Derived from `gofalcon` SDK types in `.references/poller-cobra/`
  or from static test data in `crates/prism-dtu-crowdstrike/fixtures/`. The existing
  CrowdStrike state model (`crates/prism-dtu-crowdstrike/src/state.rs`) with
  `containment_store` and `detection_status_store` provides the stateful schema.

### 1.3 Relationship to Existing Infrastructure

The `prism-dtu-common` crate already provides:
- Seeded determinism: `seeded_rng(seed: u64) -> ChaCha20Rng` (`seed.rs:9`)
- Configuration: `StubConfig { seed: u64, latency_ms: u64, failure_mode, bind }` (`config.rs`)
- Fixture loading: `load_fixture`, `load_fixture_as` (`fixture.rs`)

ADR-009 adds a generator layer that sits above `prism-dtu-common` and replaces
static fixture loading for multi-tenant test scenarios. Static fixture loading
(`load_fixture`) remains available for canonical test vector (TV) tests that assert
on exact bytes (see Section 2.5).

---

## 2. Decision

### 2.1 Hybrid Option C — Archetype Catalog + Deterministic Generator

The generator is structured as two complementary components:

1. **Archetype Catalog** — a named enum of realistic deployment scenarios. Each archetype
   defines the semantic shape of data: what is present, what is absent, what is anomalous.
   Archetypes are fixed at spec time and extended by adding new enum variants.

2. **Deterministic Generator** — a pure function from `(org_id, sensor_type, archetype,
   GenOpts)` to a `FixtureSet`. Given identical inputs, it always returns byte-identical
   output. Uses `ChaCha20Rng` seeded from `GenOpts::seed` combined with the `org_id`
   bytes to ensure org-namespace separation without requiring separate seed management.

Neither component requires I/O. The generator is a pure Rust function — no disk reads,
no network calls, no global state. This makes it suitable for in-process use in both
logical and network harness modes (see ADR-011).

### 2.2 Archetype Catalog (Initial Set)

```rust
#[non_exhaustive]
pub enum Archetype {
    /// A stable OT network with all expected sensors online, no active alerts,
    /// and devices in their baseline configuration state. Standard happy-path data.
    HealthyOtEnvironment,

    /// An environment where one or more devices show indicators of compromise:
    /// elevated alert counts, unusual lateral-movement patterns, containment
    /// state changes (Armis/CrowdStrike), or anomalous tag mutations (Claroty).
    CompromisedEndpoint,

    /// An environment where the authentication or token-refresh path is
    /// degraded: expired credentials, intermittent 401s, partial responses.
    /// Tests the retry and auth-refresh logic in the sensor adapter layer.
    AuthOutage,

    /// A large-scale environment (~10,000 devices per sensor) designed to
    /// exercise pagination logic, memory budget constraints (512MB process /
    /// 200MB per-query per memory: `project_memory_budget.md`), and
    /// DataFusion query plan behavior under cardinality pressure.
    LargeScale,

    /// An environment tuned to hit cursor-boundary and off-by-one conditions:
    /// total device counts that are exact multiples of page size, single-page
    /// responses, empty final pages, and cursor values at their length limits.
    PaginationEdgeCases,

    /// An environment where the vendor API response deviates from the schema:
    /// nullable fields returned as absent, extra unknown fields present,
    /// integer fields returned as strings. Tests OCSF normalization resilience.
    SchemaDrift,

    /// An environment with rapid device churn: devices appearing and
    /// disappearing between polling cycles. Tests the EventBuffer's handling
    /// of tombstone records and ephemeral device IDs.
    HighChurn,

    /// An environment where the org's sensors are present in config but have
    /// returned no data for an extended period — simulating a dormant or
    /// recently onboarded tenant with no historical events.
    DormantTenant,
}
```

The `#[non_exhaustive]` attribute ensures that adding new archetypes in future waves
is a backward-compatible change for any `match` expressions on `Archetype` outside
the generator crate.

### 2.3 Generator API

```rust
/// Options controlling a single generator invocation.
pub struct GenOpts {
    /// Seed for the deterministic ChaCha20Rng.
    /// Combined with org_id bytes to namespace per-org randomness.
    pub seed: u64,
    /// Scale multiplier relative to the archetype's default device count.
    /// 1.0 = archetype default; 0.1 = minimal; 10.0 = stress-test scale.
    pub scale: f64,
    /// Time anchor for all generated timestamps. Relative offsets are
    /// computed from this anchor to produce coherent event timelines.
    pub time_anchor: DateTime<Utc>,
    /// JSON patch overrides applied to the generated FixtureSet after
    /// generation. Use for test-specific field manipulation without
    /// forking the archetype.
    pub overrides: serde_json::Value,
}

impl Default for GenOpts {
    fn default() -> Self {
        Self {
            seed: 42,
            scale: 1.0,
            time_anchor: DateTime::UNIX_EPOCH, // tests use fixed anchor
            overrides: serde_json::Value::Null,
        }
    }
}

/// A fully-generated fixture dataset for one (org, sensor, archetype) triple.
pub struct FixtureSet {
    /// Org-tagged records. Every device/alert/asset ID is derived from org_id.
    pub records: Vec<serde_json::Value>,
    /// Pagination cursors for multi-page scenarios (archetype-dependent).
    pub cursors: Vec<String>,
    /// Generator provenance: (org_id, sensor_type, archetype, seed, scale).
    /// Stored in FixtureSet for debugging; not part of the canonical output.
    pub provenance: FixtureProvenance,
}

/// Top-level generator function — pure, no I/O, deterministic.
pub fn generate(
    org_id: OrgId,
    sensor_type: SensorType,
    archetype: Archetype,
    opts: GenOpts,
) -> FixtureSet;
```

### 2.4 Determinism Contract (BC-3.4.001)

The generator's primary invariant:

> For all `(org_id, seed, archetype, scale)`, calling `generate(org_id, sensor_type,
> archetype, GenOpts { seed, scale, .. })` twice on the same binary produces
> byte-identical `FixtureSet::records`.

Implementation mechanism: the RNG is initialized as
`ChaCha20Rng::seed_from_u64(seed ^ org_id_hash)` where `org_id_hash` is the
first 8 bytes of the `OrgId` UUID bytes interpreted as `u64` (little-endian).
This XOR ensures:
- Same seed, different org → different RNG stream → different records
- Same org, different seed → different RNG stream → different records
- Same org, same seed → identical RNG stream → identical records

The XOR with org bytes is the canonical org-namespace separation. No global state,
no `thread_rng()`, no timestamp-seeded entropy. The `seeded_rng` convention already
established in `prism-dtu-common/src/seed.rs:9` is extended, not replaced.

### 2.5 Org-Tagged Record IDs (BC-3.4.004)

Every generated record's primary identifier is derived from the `org_id`:

```
device_id format:  "dev-{org_slug}-{seed}-{index}"
alert_id format:   "alert-{org_slug}-{seed}-{index}"
asset_id format:   "asset-{org_slug}-{seed}-{index}"
```

`org_slug` is resolved at generation time via `OrgRegistry::slug_for(org_id)`.
If slug resolution fails (org not registered), the generator returns an error —
`GeneratorError::UnregisteredOrg(org_id)`. There is no UUID-prefix fallback. A
missing slug at generation time is a test misconfiguration; failing loudly is the
correct behavior (per spec-reviewer S-2 recommendation and M-007 convergence).

The prefix scheme uses the org **slug** (not UUID namespace prefix). This slug-based
prefix is binding: `"dev-acme-corp-42-0"` not `"dev-<uuid-namespace>-42-0"`. This
aligns with BC-3.4.004 which specifies that the ID contains the org slug as an
inspectable substring for leakage detection purposes.

This tagging makes cross-tenant data leakage detectable by inspection: a
test assertion that inspects any device ID from an `ACME` org query and finds
a `globex`-prefixed ID is a visible leakage indicator without requiring a
cryptographic proof.

### 2.6 Integration with Customer TOML Config

Per the customer TOML schema established in ADR-006, each `[[dtu]]` block optionally
declares a `[dtu.data]` subsection:

```toml
# customers/acme-corp.toml
[[dtu]]
type = "claroty"
mode = "client"

[dtu.data]
archetype = "HealthyOtEnvironment"
scale = 1.0
seed = 12345

[[dtu]]
type = "crowdstrike"
mode = "client"

[dtu.data]
archetype = "CompromisedEndpoint"
scale = 0.5
seed = 99
static = "fixtures/acme-crowdstrike-canonical.json"  # override: use static TV
```

The `static` override in `[dtu.data]` instructs the harness to load the fixture
file verbatim rather than invoking the generator. This path is reserved for
canonical test vector (TV) assertions where the test checks exact bytes. The
default (no `static` key) uses the generator.

### 2.7 Harness Override API

For programmatic override within test code:

```rust
let harness = Harness::builder()
    .with_customer("acme-corp", |c| {
        c.dtu("claroty")
            .set_archetype(Archetype::AuthOutage)
            .set_seed(9999)
            .set_scale(2.0)
    })
    .build()
    .await?;
```

This API is provided by the new `prism-dtu-harness` crate (see ADR-011 for harness
structure). The override chain calls into the generator for the modified
`(org_id, sensor_type, archetype, GenOpts)` triple at harness build time.

### 2.8 Crate Placement

The generator is implemented as a new module within `crates/prism-dtu-common/src/generator/`,
behind a `#[cfg(feature = "fixture-gen")]` feature gate. It is NOT a separate crate.
Rationale:

- `prism-dtu-common` already owns the `seeded_rng` convention, `StubConfig`, and
  `load_fixture`. The generator is a natural extension of the existing "shared DTU
  infrastructure" scope.
- Avoiding a new crate keeps the workspace crate count stable for Wave 3 housekeeping
  (see ADR-012).
- The `fixture-gen` feature gate ensures the generator never links into production
  binaries; it is activated only in test and harness builds.

If the generator module grows beyond 1,500 lines in a future wave, extracting it to
`crates/prism-dtu-fixture-gen` is the natural next step. That extraction is deferred
(open question — see Section 8).

---

## Rationale

The three components of Option C are jointly necessary; no subset is sufficient.

**Named archetypes are required for BC traceability.** A test that invokes
`generate(org, claroty, seed=42)` with no scenario semantics cannot be traced to a
behavioral contract. A test that invokes `generate(org, claroty, Archetype::AuthOutage,
opts)` is explicitly linked to the auth-outage failure mode, traceable to BC-3.4.003,
and self-documenting in CI failure output. Scenario names are load-bearing spec
artifacts, not aesthetic choices.

**Deterministic generation is required for formal verification support.** The
verification properties in BC-3.4.001 and BC-3.5.001 (ADR-011) are property-based
tests run under Kani and proptest. Property-based harnesses must be able to reproduce
a failing input exactly. A generator that produces different output on each run cannot
be embedded in a Kani harness — it would produce a non-deterministic proof obligation.
The XOR-seed construction (Section 2.4) provides org-namespace separation within a
single deterministic function.

**Org-tagged IDs are required to make isolation failures visible.** The core threat
model (Section 3.1, BC-3.5.002 in ADR-011) is that a `HashMap<String, V>` keying
bug merges two orgs' data silently. When all orgs share the same device IDs (the
current static fixture state), that bug produces a test that passes — both orgs see
"their" device, which happens to be the same record. Org-tagged IDs make the bug
produce a visible assertion failure: an `ACME` query returning a `globex`-prefixed ID
is an unambiguous isolation violation.

**Module placement in `prism-dtu-common` is correct for Wave 3.** The generator is
test-only infrastructure, gated behind `feature = "fixture-gen"`. Adding
it as a module within the existing crate avoids a new workspace crate entry, which
aligns with the ADR-012 housekeeping principle of stable crate count for Wave 3.
The extraction threshold (1,500 lines) is defined so that the decision to create
`prism-dtu-fixture-gen` is driven by evidence, not speculation.

**The `static` override path is required to preserve canonical TV integrity.** Some
BCs specify postconditions over exact byte sequences (e.g., "the audit record for
this event contains exactly these fields"). Those tests cannot use a generator because
the generator introduces variance in non-asserted fields. The `static` override path
allows those tests to pin to a known fixture while all other tests use the generator.
This is not a compromise — it is the correct separation of concerns between
behavioral coverage (generator) and exact-format verification (static TV).

---

## 3. Threat Model

### 3.1 Cross-Tenant Leakage via Identical Device IDs

**Threat:** Org A and Org B share the same generated device ID, causing a keying bug
to look like correct isolation when it is not.

**Mitigation:** BC-3.4.004 mandates org-tagged IDs. The `prism-dtu-harness` cross-
tenant fidelity test (BC-3.5.002 per ADR-011) asserts that no device ID returned
in an Org A query appears in an Org B query result.

### 3.2 Non-Determinism Contamination

**Threat:** A developer calls `rand::thread_rng()` inside the generator, breaking
the determinism contract and causing flaky tests.

**Mitigation:** `prism-dtu-common` already enforces the `seeded_rng` convention by
convention (no `thread_rng()` per `seed.rs` module doc). BC-3.4.001 formalizes
this as a verifiable postcondition. The generator module doc will carry a
`#[deny(clippy::thread_rng_use)]` annotation (pending clippy lint availability)
and a code comment ban.

### 3.3 Schema Conformance Regression

**Threat:** A generated fixture does not match the actual vendor API response schema,
causing a test to pass against the generator but fail against the live API.

**Mitigation:** Generator output is validated against the vendored OpenAPI schemas
(`.references/poller-bear/docs/specs.json`, etc.) using `jsonschema` crate validation
in the `generate()` function under `#[cfg(test)]`. BC-3.4.002 specifies this
conformance postcondition.

---

## 4. Migration Strategy

The migration from static fixtures to the generator is additive and per-crate:

**Step 1 — Add generator module to `prism-dtu-common`.**
Create `crates/prism-dtu-common/src/generator/` with `mod.rs`, `archetype.rs`,
`opts.rs`, `sensor_generators/claroty.rs`, `sensor_generators/crowdstrike.rs`,
`sensor_generators/armis.rs`, `sensor_generators/cyberint.rs`. Gate behind
`#[cfg(feature = "fixture-gen")]`. Export `generate`, `Archetype`, `GenOpts`,
`FixtureSet` from `prism_dtu_common` lib under that feature. Gate: `cargo build -p prism-dtu-common` clean.

**Step 2 — Add harness integration (`prism-dtu-harness`).**
Per ADR-011, the harness crate calls `generate()` when building per-org DTU state.
This step depends on ADR-011 implementation. Gate: `cargo test -p prism-dtu-harness`
green with at least `HealthyOtEnvironment` and `CompromisedEndpoint` archetypes.

**Step 3 — Migrate per-DTU tests to generator-backed fixtures.**
Each `prism-dtu-*` crate's integration tests that currently call `load_fixture()`
for multi-tenant scenarios are updated to call `generate()`. Single-tenant canonical
TV tests retain `load_fixture()`. Gate: per-crate tests green.

**Step 4 — Add TOML config parsing.**
`customers/*.toml` `[dtu.data]` block parsing added to the config loading path
(ADR-010 scope). Generator invoked at startup when `static` override is absent.
Gate: `cargo test -p prism-spec-engine` green.

Existing static fixtures in `crates/prism-dtu-*/fixtures/` are retained as
canonical TV fixtures. They are not deleted; they are reclassified as
`static` override fixtures for BC-level exact-byte assertions.

---

## 5. Alternatives Considered

| Option | Description | Decision |
|--------|-------------|----------|
| **Option A — Pure static fixtures** | Keep existing `load_fixture` approach; hand-craft one fixture file per (org, scenario) | Rejected: O(N×M) maintenance burden. Cross-tenant leakage is invisible when all orgs share the same device IDs. |
| **Option B — Pure generator, no archetypes** | Generate all data from `(org_id, seed)` with no scenario semantics | Rejected: tests would not be able to assert "this test exercises the AuthOutage scenario." Scenario semantics must be first-class for test documentation and BC traceability. |
| **Option C — Hybrid (SELECTED)** | Named archetype catalog + deterministic generator; static fallback for canonical TVs | Selected: scenarios are named and traceable (archetype → BC); data is org-tagged and deterministic; canonical TV assertions are preserved via `static` override. |
| **Separate `prism-dtu-fixture-gen` crate** | New crate rather than module in `prism-dtu-common` | Rejected for Wave 3: module extension behind `feature = "fixture-gen"` is simpler. Extraction to separate crate deferred if module exceeds 1,500 lines. |
| **`serde_json::Value` patch for all scenario variation** | Single "default" generator with JSON patch overrides for all scenario differences | Rejected: a patch that says `"alerts": [...]` does not document why that structure is semantically significant. Named archetypes self-document intent. |

---

## 6. Consequences

### Positive

- Cross-tenant leakage is detectable by inspection: org-tagged IDs make isolation
  failures visible without a formal proof.
- Scenario coverage is extensible without new fixture files: adding an archetype
  variant costs one enum arm and one generator branch.
- Determinism is contractual (BC-3.4.001), not incidental: the XOR-seed construction
  guarantees org-namespace separation.
- Static canonical TVs are preserved: exact-byte BC assertions continue to use
  `load_fixture` via the `static` override path.
- Generator is pure (no I/O): usable in both logical and network harness modes (ADR-011)
  without any harness-level I/O coordination.

### Negative

- Generator code must track schema evolution: when the Claroty API adds a new
  required field, the generator's Claroty branch must be updated in the same story.
  This is a maintenance coupling that static fixtures share, but it is now explicit.
- `serde_json::Value` as the generator's output type sacrifices type safety in the
  generator internals. This is acceptable for test-only code; production normalization
  uses typed OCSF structs.
- `#[non_exhaustive]` on `Archetype` means downstream `match` expressions must have
  a wildcard arm. Tests that need to be exhaustive over all archetypes must use an
  `all_archetypes()` helper function rather than exhaustive `match`.

### Unchanged

- `prism-dtu-common` crate gate: `#[cfg(any(test, feature = "dtu"))]`. The generator
  is additionally gated behind `feature = "fixture-gen"` and never links into
  production binaries.
- `seeded_rng` convention: unchanged. The generator builds on the existing
  `ChaCha20Rng::seed_from_u64` pattern.
- Static fixture files in `crates/prism-dtu-*/fixtures/`: retained as canonical TV
  fixtures. No deletions in Wave 3.

---

## 7. Behavioral Contracts Scoped by This ADR

The following BCs were authored during Phase 3.A; see BC-INDEX for canonical metadata.

| BC ID | Title | Postcondition summary |
|-------|-------|-----------------------|
| BC-3.4.001 | Generator Determinism — Identical Inputs Produce Byte-Identical FixtureSet | `generate(org_id, sensor, archetype, opts { seed, scale })` called twice on the same binary returns byte-identical `FixtureSet::records`. |
| BC-3.4.002 | Generator Output Schema-Validates Against Canonical Vendor API Spec | Every record in a generated `FixtureSet` validates against the vendored OpenAPI schema for the given `sensor_type`. |
| BC-3.4.003 | Archetype Catalog Enumeration — 8 Archetypes with Defined Baselines | For each `Archetype` variant, the generated `FixtureSet` satisfies the archetype's semantic specification (e.g., `AuthOutage` produces at least one 401-class response fixture; `LargeScale` produces at least 10,000 device records at `scale=1.0`). |
| BC-3.4.004 | Org-Tagged Record IDs — Every Generated Record Carries an Org-Derived ID Prefix | Every record in a `FixtureSet` generated for `org_id(A)` has a primary identifier that contains an `org_id(A)`-derived prefix. No record generated for `org_id(A)` has a primary identifier derived from any other `org_id`. |

---

## 8. Open Questions for Next Dispatch

1. **Crate extraction threshold.** At what point does `prism-dtu-common/src/generator/`
   warrant extraction to a separate `crates/prism-dtu-fixture-gen` crate? Current
   heuristic: 1,500 lines of generator code or two waves of generator growth. Spec-writer
   should note the module size after Wave 3 authoring and flag if extraction is warranted.

2. **Armis and CrowdStrike schema derivation.** The Armis generator branch derives from
   Go types in `.references/poller-coaster/internal/` or `armis-sdk-go/v2`. The
   translation from Go struct tags to Rust `serde` attributes requires a manual
   derivation step that spec-writer must perform. Should this derivation be a separate
   story (with output checked into `.references/`) or inline in the BC-3.4 authoring?
   Recommend a separate story to keep BC-3.4 authoring unblocked.

3. **`scale` semantics per archetype.** `LargeScale` at `scale=1.0` produces 10,000
   devices. What does `HealthyOtEnvironment` at `scale=1.0` produce? Each archetype
   needs a defined default device count. Spec-writer must enumerate per-archetype
   baseline counts in BC-3.4.003. Recommend: `HealthyOtEnvironment=50`,
   `CompromisedEndpoint=50`, `AuthOutage=20`, `LargeScale=10_000`,
   `PaginationEdgeCases=default_page_size() × 3`, `SchemaDrift=30`, `HighChurn=200`,
   `DormantTenant=0`.

4. **JSON patch `overrides` application order.** The `GenOpts::overrides: serde_json::Value`
   patch is applied after generation. Should the patch be a JSON Merge Patch (RFC 7396)
   or a JSON Patch (RFC 6902)? Recommendation: JSON Merge Patch for Wave 3 (simpler
   to implement and sufficient for field-level overrides). RFC 6902 operations deferred
   if needed.

5. **Static TV fixture naming convention.** The `static = "fixtures/path.json"` path in
   `[dtu.data]` is relative to what root? Recommend: crate root of the referencing DTU
   crate (i.e., `crates/prism-dtu-*/`). ADR-012's fixture convention (see Section 2)
   establishes `fixtures/` as the canonical top-level directory; static paths should
   reference files within that directory.

---

## 9. ADR Chain — Related Documents

This ADR specifies the data generator infrastructure consumed by the multi-tenant
harness. Related documents:

- **ADR-006:** OrgId/OrgSlug identity. The generator derives org-tagged record IDs
  from `OrgId`; `OrgRegistry::slug_for` provides the slug component.
- **ADR-010** (to be drafted): Customer TOML config schema. Specifies the `[dtu.data]`
  block structure that feeds `GenOpts` at harness build time.
- **ADR-011:** DTU harness isolation modes. The harness calls `generate()` during
  per-org DTU state construction in both logical and network modes.
- **ADR-012:** Workspace convention normalization. The generator module lives in
  `prism-dtu-common/src/generator/` under the `src/` convention.

---

## Source / Origin

- **PO decision:** D-043 (multi-tenant data generator hybrid model) — recorded in
  `.factory/STATE.md`, Wave 3 kickoff 2026-04-27.
- **Code as-built — seeded RNG:** `crates/prism-dtu-common/src/seed.rs:9` —
  `seeded_rng(seed: u64) -> ChaCha20Rng`; generator extends this convention.
- **Code as-built — stub config:** `crates/prism-dtu-common/src/config.rs:5-31` —
  `StubConfig { seed, latency_ms, failure_mode, bind }`; `GenOpts` parallels this
  structure at the generator level.
- **Code as-built — state stores (migration targets):**
  `crates/prism-dtu-claroty/src/state.rs:24` (`tag_store: Mutex<HashMap<String, HashSet<String>>>`),
  `crates/prism-dtu-crowdstrike/src/state.rs` (`containment_store`, `detection_status_store`) —
  these stores will be populated from generator output under multi-tenant test runs.
- **Schema sources (vendored, 1898 IP):**
  `.references/poller-bear/docs/specs.json` (Claroty OpenAPI);
  `.references/poller-express/docs/specs/` (four Cyberint API specs).
- **Behavioral contracts:** BC-3.4.001 through BC-3.4.004 — scoped by this ADR; to
  be authored by spec-writer in Phase 3.A.

---

## Decision Refinements (2026-04-27)

The following questions surfaced during BC authoring (Phase 3.A) and were resolved by the orchestrator on 2026-04-27. Each refinement is recorded here for historical traceability and is binding for Wave 3 implementation.

### D-054 — Armis and CrowdStrike schema derivation is a pre-story under E-3.7

**Question:** The Armis generator branch requires translation of Go SDK types (`armis-sdk-go/v2`) to Rust `serde` structs. The CrowdStrike branch requires derivation from `gofalcon` SDK types. Should this derivation happen inline in BC-3.4 authoring, or as a separate pre-story?

**Resolution:** Armis and CrowdStrike schema derivation (Go SDK → Rust types) is a separate pre-story under E-3.7, designated S-3.7.00 (schema-derive). This pre-story blocks generator implementation for those two sensors. Claroty and Cyberint generators may proceed without it (they have vendored OpenAPI specs). S-3.7.00 must complete before the Armis and CrowdStrike generator branches are authored.

**Rationale:** Inlining Go-to-Rust type derivation into BC-3.4 authoring would block spec authoring on a technical translation task that belongs in an implementation story. Separating it as S-3.7.00 keeps BC-3.4 authoring unblocked for Claroty and Cyberint, and gives the implementation team a clear pre-condition story with a defined deliverable (Rust type definitions checked into `.references/`). The "separate story" pattern is established by the OSS reference spec-to-impl pipeline in the factory; this follows that pattern.

**Affected BCs:** BC-3.4.002, BC-3.4.003 (Armis and CrowdStrike sensor branches)

### D-055 — PaginationEdgeCases archetype baseline count uses `default_page_size()`

**Question:** Open Question 3 in Section 8 noted that `PaginationEdgeCases` baseline count is `page_size × 3`. What is `page_size`? Is it a global constant or per-sensor?

**Resolution:** `PaginationEdgeCases` archetype baseline count is `default_page_size() × 3` where `default_page_size()` is a per-sensor function defined in each generator module (e.g., `claroty_generator::default_page_size() -> usize`). There is no single global page size; each sensor's API has its own default pagination behavior.

**Rationale:** Using a per-sensor `default_page_size()` function co-locates the page size knowledge with the sensor generator that knows the sensor's actual API behavior. A global constant would require it to be artificially reconciled across sensors with different real-world page sizes (Claroty's default differs from CrowdStrike's). BC-3.4.003's `PaginationEdgeCases` baseline count specification (`page_size × 3`) is therefore instantiated per sensor type, not as a single global number. Each sensor generator module exposes `pub fn default_page_size() -> usize` so the archetype baseline table in BC-3.4.003 can reference it generically.

**Affected BCs:** BC-3.4.003

### D-056 — Archetype catalog code in `prism-dtu-common` behind `feature = "fixture-gen"`

**Question:** Earlier guidance suggested a separate `prism-dtu-fixture-gen` crate. This ADR (Section 2.8) placed the generator in `prism-dtu-common/src/generator/`. Which is correct?

**Resolution:** The archetype catalog code lives in `crates/prism-dtu-common/src/generator/` behind `#[cfg(feature = "fixture-gen")]`. There is NO separate `prism-dtu-fixture-gen` crate for Wave 3. This supersedes any earlier guidance suggesting a separate crate.

**Rationale:** The Wave 3 decision (D-043, ADR-009 Section 2.8) was made with full context: keeping the generator as a module within `prism-dtu-common` avoids adding a crate to the workspace (consistent with ADR-012 housekeeping), keeps the seeded-RNG convention co-located with the generator that extends it, and uses a feature gate (`fixture-gen`) to ensure zero production binary impact. BC alignment (BC-3.4.001 through BC-3.4.004) references `prism-dtu-common` as the module home. The "separate crate" guidance was an earlier exploration, not a final decision. This refinement closes that ambiguity.

**Affected BCs:** BC-3.4.001, BC-3.4.002, BC-3.4.003, BC-3.4.004

### D-059 — Generated record IDs use slug-based prefix (not UUID-namespace prefix)

**Question:** Section 2.5 specifies `"dev-{org_slug}-{seed}-{index}"` but earlier discussion mentioned a UUID-namespace prefix variant. Which format is canonical?

**Resolution:** Generated record IDs use slug-based prefix — `"dev-{org_slug}-{seed}-{index}"` — not UUID-namespace prefix. This is the binding format for Wave 3. An example: org `acme-corp` with seed `42`, first device → `"dev-acme-corp-42-0"`. The UUID-namespace variant is not implemented.

**Rationale:** Slug-based prefixes are human-inspectable at a glance: `"dev-acme-corp-42-0"` immediately tells a developer which org generated the record and with what seed. UUID-namespace prefixes (`"dev-01975e4e-42-0"`) require looking up the UUID to identify the org. The primary purpose of org-tagged IDs is to make cross-tenant leakage visible by inspection during test debugging — slug-based prefixes maximize that signal. BC-3.4.004 specifies that the ID "contains the org slug as a substring," which is only satisfied by the slug-based format. The ADR Section 2.5 text is already correct; this refinement resolves any ambiguity from earlier discussion.

**Affected BCs:** BC-3.4.004

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.9 | 2026-04-27 | product-owner | M-003 (pass-13-remediation): Status block updated — "BCs to be authored in subsequent Phase 3.A spec-writer dispatch" → "BCs authored at v0.3+ during Phase 3.A; see BC-INDEX." §7 preamble updated to match. |
| 0.8 | 2026-04-27 | product-owner | m-001/m-002 (pass-10-remediation): §7 BC table titles updated to Title Case matching BC-INDEX H1 source-of-truth: "Generator determinism"→"Generator Determinism — Identical Inputs Produce Byte-Identical FixtureSet"; "Generator schema conformance"→"Generator Output Schema-Validates Against Canonical Vendor API Spec"; "Archetype behavioral coverage"→"Archetype Catalog Enumeration — 8 Archetypes with Defined Baselines"; "Org-tagged record IDs"→"Org-Tagged Record IDs — Every Generated Record Carries an Org-Derived ID Prefix". |
| 0.7 | 2026-04-27 | product-owner | M-003 (pass-6-remediation): Frontmatter `title:` corrected to Title Case to match H1 heading (POL 7 H1 source-of-truth). |
| 0.6 | 2026-04-27 | product-owner | M-003 (pass-4-remediation): SS-01 added to subsystems_affected (generator code lives in prism-dtu-common which is SS-01; archetype catalog, seed.rs, config.rs, and fixture.rs all reside in prism-dtu-common). |
| 0.5 | 2026-04-27 | product-owner | m-003 (Pass 3): `S-3.7.0` → `S-3.7.00` in D-054 Resolution, Rationale, and 0.2 changelog row (canonical 2-digit suffix per STORY-INDEX). |
| 0.4 | 2026-04-27 | product-owner | M-007 fix: §2.5 fallback behaviour corrected — UUID-prefix fallback (`dev-{org_id_prefix}-...`) removed. Slug resolution failure now returns `GeneratorError::UnregisteredOrg(org_id)` (fail-loud on test misconfiguration, per spec-reviewer S-2 and D-059 canonical format). No UUID-namespace variant implemented. |
| 0.3 | 2026-04-27 | product-owner | C-5 capability anchoring: `anchored_capabilities: [CAP-039]` added to frontmatter. CAP-039 (Multi-Tenant Fixture Generation) anchors BC-3.4.001–004. |
| 0.2 | 2026-04-27 | architect | Decision Refinements: D-054 (Armis/CrowdStrike schema derivation as pre-story S-3.7.00), D-055 (PaginationEdgeCases baseline = default_page_size() × 3 per-sensor), D-056 (generator in prism-dtu-common behind fixture-gen feature, not separate crate), D-059 (slug-based ID prefix not UUID-namespace prefix) |
| 0.1 | 2026-04-27 | architect | Initial draft — scopes D-043, D-045; status PROPOSED |
