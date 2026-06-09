---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.018"
version: "1.0"
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
anchored_stories: [S-DEMO-DTU-DATA-SEEDING-001]
verifying_vps: []
crates: [prism-dtu-demo-server, prism-dtu-common, prism-dtu-claroty, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-cyberint, prism-dtu-threatintel, prism-dtu-nvd]
inputs:
  - "crates/prism-dtu-demo-server/src/config.rs"
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - "crates/prism-dtu-common/src/generator/archetype.rs"
  - "crates/prism-dtu-common/src/generator/opts.rs"
  - "crates/prism-dtu-common/src/generator/rng.rs"
  - ".factory/specs/behavioral-contracts/BC-3.4.001.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: ""
traces_to:
  - "CAP-036"
  - "BC-3.4.001"
extracted_from: null
---

# BC-2.06.018: Demo-Server Config-Time Data Seeding — Per-Clone seed + fixture_set Wire-Up

## Description

`build_clone_pairs` in `prism-dtu-demo-server` must read `CloneConfig.seed` and
`CloneConfig.fixture_set` for each enabled clone and pass them to a per-clone
`new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId)` constructor, causing
each clone instance to serve deterministically distinct fixture data at startup. This BC
governs the **demo-server wiring layer** — the path from TOML config fields through
`build_clone_pairs` to the per-clone constructor. The underlying generation determinism
guarantee is specified by BC-3.4.001 (one layer below); this BC governs how the
demo-server supplies the seed and archetype inputs to that layer.

Generator-backed clones (Claroty, Armis, CrowdStrike, Cyberint) invoke the `fixture-gen`
generator at construction time. Static-file clones (ThreatIntel, NVD) use
`fixture_set` to select a named fixture file from the clone's embedded fixture catalog.
The existing `POST /dtu/configure` secondary override endpoint is unchanged and remains
a runtime override path; it is NOT the primary seeding path governed by this BC.

## Preconditions

1. `DemoConfig` has been parsed from a valid TOML file (BC-2.06.001).
2. Each `CloneConfig` has a `seed: u64` field (default `42`) and a `fixture_set: String`
   field (default `"default"`), both already present in
   `crates/prism-dtu-demo-server/src/config.rs::CloneConfig`.
3. The `fixture-gen` Cargo feature is enabled in `prism-dtu-demo-server`'s feature set
   when generator-backed clones are included in the build.
4. `CloneConfig.org_id: OrgId` is available at harness construction time — either passed
   in by the caller or derived from a canonical demo org slug per the story design
   (story-writer resolves the exact parameter plumbing in S-DEMO-DTU-DATA-SEEDING-001).
5. For static-file clones (ThreatIntel, NVD), the named fixture file corresponding to
   `CloneConfig.fixture_set` exists in the clone crate's embedded fixture catalog.
6. `build_clone_pairs` is called before `DemoHarness::start_all`.

## Postconditions

### Postcondition 1 — Seed is forwarded to generator-backed clones at construction time

For each enabled generator-backed clone (Claroty, Armis, CrowdStrike, Cyberint), when
`build_clone_pairs` constructs the clone:

- It calls `<CloneType>::new_with_seed(seed, archetype, org_id)` instead of the
  argument-free `<CloneType>::new()`.
- `seed` is sourced from `clone_cfg.seed` (e.g., `config.clones.claroty.seed`).
- `archetype` is the `Archetype` enum variant mapped from `clone_cfg.fixture_set`
  (see INV-FIXTURE-SET-ARCHETYPE-MAP-001 below).
- `org_id` is the `OrgId` for this demo instance (plumbing resolved by story-writer per
  Precondition 4).
- The clone generates its fixture data at construction time using the seeded generator
  (via `prism_dtu_common::generator::seeded_rng(seed, org_id)` per BC-3.4.001).

### Postcondition 2 — fixture_set selects named static fixture for static clones

For each enabled static-file clone (ThreatIntel, NVD), when `build_clone_pairs`
constructs the clone:

- It calls `<CloneType>::new_with_fixture_set(fixture_set: &str)` (or equivalent).
- `fixture_set` is sourced from `clone_cfg.fixture_set`.
- The clone loads the named embedded fixture file at construction time.
- An unrecognized `fixture_set` name that has no matching embedded fixture causes a
  construction-time error (E-DEMO-001; see Error Codes section). The process does not
  start if a non-`continue_on_error` clone fails construction.
- `fixture_set = "default"` is always valid and resolves to the backward-compatible
  default embedded fixture for that clone.

### Postcondition 3 — Seed-to-data determinism (references BC-3.4.001)

For any generator-backed clone instance constructed with `(seed, fixture_set, org_id)`:

- All subsequent HTTP responses to fixed request paths are byte-identical across process
  restarts and harness re-runs, given identical `(seed, fixture_set, org_id)` inputs.
- This postcondition is enforced by BC-3.4.001 (Generator Determinism). This BC's
  contribution is ensuring `build_clone_pairs` supplies those inputs consistently from
  `CloneConfig`.

### Postcondition 4 — Backward compatibility when seed=42 and fixture_set="default"

When `CloneConfig.seed = 42` (the default) and `CloneConfig.fixture_set = "default"`
(the default) for all clones, all existing integration tests that were passing against
the pre-seeding `CloneType::new()` constructor continue to pass. The
`new_with_seed(42, Archetype::HealthyOtEnvironment, <default_org_id>)` call produces
fixture data semantically equivalent to the pre-seeding `new()` behavior:

- For generator-backed clones: the default seed and default archetype reproduce the
  same fixture set that `new()` produced (the old `new()` used `seed=42` and the
  equivalent default archetype internally, or the generator is initialized equivalently).
- For static-file clones: `fixture_set = "default"` loads the same embedded default
  fixture file that `new()` loaded.

**Note:** If the pre-seeding `new()` constructors used different internal defaults, the
implementer MUST either preserve those defaults in `new_with_seed(42, ...)` or update
the affected integration tests in the same story as S-DEMO-DTU-DATA-SEEDING-001. This
postcondition prohibits silent data-shape changes that cause existing tests to fail
without the implementer noticing.

### Postcondition 5 — `POST /dtu/configure` secondary override remains functional

The existing `POST /dtu/configure` endpoint on each clone is unchanged and continues
to accept runtime override payloads. It is a secondary mechanism that overrides
config-time seeding for a specific request sequence. It does NOT replace config-time
seeding as the primary determinism path.

## Invariants

### INV-DISTINCT-DATA-001 — Distinct Seeds Produce Disjoint ID Sets

For two generator-backed clone instances A and B constructed with `seed_A ≠ seed_B`
(or different `org_id_A ≠ org_id_B`), the ID sets present in their responses are
disjoint: no record ID that appears in responses from instance A appears in responses
from instance B.

This invariant holds because:
- `seeded_rng(seed, org_id)` produces a unique RNG stream per `(seed, org_id)` pair
  (BC-3.4.001 postconditions 3 and 4).
- Every generated record's primary ID carries an org-tagged prefix
  `dev-{org_slug}-{seed}-{index}` (BC-3.4.004), making IDs structurally disjoint across
  distinct `(seed, org_id)` tuples.

The invariant is verified by integration tests in S-DEMO-DTU-DATA-SEEDING-001 that
start two clone instances with `seed_A = 100` and `seed_B = 200` and assert that
`responses_A.ids ∩ responses_B.ids = ∅`.

### INV-FIXTURE-SET-ARCHETYPE-MAP-001 — Canonical fixture_set → Archetype Mapping

The `fixture_set: String` from `CloneConfig` maps to an `Archetype` enum variant for
generator-backed clones according to this canonical table:

| `fixture_set` string   | `Archetype` variant         |
|------------------------|-----------------------------|
| `"default"`            | `Archetype::HealthyOtEnvironment` |
| `"compromised"`        | `Archetype::CompromisedEndpoint`  |
| `"auth_outage"`        | `Archetype::AuthOutage`           |
| `"large_scale"`        | `Archetype::LargeScale`           |
| `"pagination_edges"`   | `Archetype::PaginationEdgeCases`  |
| `"schema_drift"`       | `Archetype::SchemaDrift`          |
| `"high_churn"`         | `Archetype::HighChurn`            |
| `"dormant"`            | `Archetype::DormantTenant`        |
| any other string       | construction-time error: E-DEMO-001 |

The mapping function lives in `build_clone_pairs` (or a helper it calls) in
`crates/prism-dtu-demo-server/src/harness.rs`. Unrecognized `fixture_set` values cause
a construction-time `Err(...)` that propagates through `build_clone_pairs`'s
`anyhow::Result<Vec<ClonePair>>` return type and aborts the harness startup.

### INV-STATIC-CLONE-FIXTURE-SET-001 — Static Clones Accept fixture_set; "default" Always Valid

ThreatIntel and NVD clones do NOT have a generator-backed fixture path. For these clones,
`fixture_set` selects a named embedded fixture file:

- The set of valid `fixture_set` values for static clones is defined by the embedded
  fixture catalog in each clone crate. The "default" value is always valid (backward
  compatibility guarantee).
- Serving the same fixture file for all demo clients from a given static clone is
  **acceptable** — ThreatIntel and NVD serve lookup/enrichment data (CVE records,
  threat intel indicators) that are MSSP-wide, not per-client. Data distinctness per
  client is NOT a requirement for these two clone types.
- An unrecognized `fixture_set` for a static clone also causes E-DEMO-001 at
  construction time.

### INV-CONSTRUCTION-TIME-FAILURE-001 — Unrecognized fixture_set Fails at Construction, Not at Request Time

An invalid `fixture_set` value MUST produce a construction-time error propagated through
`build_clone_pairs -> anyhow::Result<Vec<ClonePair>>`. It MUST NOT:

- Silently fall back to `"default"`.
- Panic at request-handling time.
- Return a partially-constructed clone that subsequently panics or serves incorrect data.

The construction-time failure path follows the existing abort/continue-on-error semantics
already present in `DemoHarness::start_all`: if `build_clone_pairs` returns `Err`,
`start_all` propagates the error and the harness does not start.

### INV-CONFIGURE-ENDPOINT-SECONDARY-001 — `POST /dtu/configure` Is a Secondary Override, Not Primary Path

Config-time seeding via `CloneConfig.seed` + `CloneConfig.fixture_set` is the **primary**
path. `POST /dtu/configure` is a **secondary runtime override** that may change the
clone's behavior after startup. The two mechanisms are independent:

- Config-time seeding runs in `build_clone_pairs` before the server binds.
- Runtime `POST /dtu/configure` runs after the server is bound and serving.
- A `POST /dtu/configure` that changes fixture data does not retroactively alter the
  config-time seeding. The clone's initial state is always determined by the config.

## Error Codes

### E-DEMO-001 — Unrecognized fixture_set Name

A new error code is required in the `prism-dtu-demo-server` error domain:

| Field | Value |
|-------|-------|
| Code | `E-DEMO-001` |
| Category | configuration |
| Severity | broken |
| Exit code | `1` (startup failure) |
| Message format | `"demo-server: E-DEMO-001: clone '{clone_name}': unrecognized fixture_set '{value}'; valid values: default, compromised, auth_outage, large_scale, pagination_edges, schema_drift, high_churn, dormant"` |
| Recoverable | No — operator must fix `demo.toml` and restart |

**Flag for error-taxonomy owner:** E-DEMO-001 must be registered in
`.factory/specs/prd-supplements/error-taxonomy.md` under a new `E-DEMO-NNN` namespace.
The demo-server is test/demo infrastructure and currently has no E-DEMO-NNN entries.
Error-taxonomy owner (product-owner) must add the `E-DEMO` subsystem prefix and this
entry in the same story or a coordinated burst.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-018-001 | `seed = 42` (default) + `fixture_set = "default"` for all clones | Backward-compatible: data identical to pre-seeding `new()` behavior; all existing integration tests pass |
| EC-018-002 | Two clone configs with `seed_A = 100` + `seed_B = 200`, same `fixture_set = "default"` | INV-DISTINCT-DATA-001 holds: response ID sets are disjoint |
| EC-018-003 | `fixture_set = "dormant"` for a generator-backed clone | `Archetype::DormantTenant` constructed; clone returns empty device/alert responses; no construction-time error |
| EC-018-004 | `fixture_set = "xyzzy_unknown"` for any clone | Construction-time E-DEMO-001; `build_clone_pairs` returns `Err`; harness aborts (unless `continue_on_error = true` for that clone) |
| EC-018-005 | `fixture_set = "large_scale"` for CrowdStrike clone | `Archetype::LargeScale` constructed; 10 000 device records generated at startup (scale=1.0); startup time within acceptable bounds (no assertion on exact time, but not unbounded) |
| EC-018-006 | `fixture_set = "default"` for ThreatIntel (static clone) | Loads default embedded fixture file; no generator invoked; response is byte-identical to pre-seeding behavior |
| EC-018-007 | `fixture_set = "compromised"` for ThreatIntel (static clone — no such fixture) | Behavior depends on ThreatIntel embedded catalog: if no "compromised" fixture exists, E-DEMO-001 at construction time; if a "compromised" fixture was added, it is loaded. The fixture catalog is authoritative. |
| EC-018-008 | `seed = u64::MAX` for a generator-backed clone | Valid; deterministic (BC-3.4.001 EC-3.4.001-04); construction succeeds |
| EC-018-009 | Process restart with same `demo.toml` | All clone instances serve byte-identical responses to the same request paths as before the restart (INV-DISTINCT-DATA-001 + BC-3.4.001 postcondition 6) |
| EC-018-010 | `continue_on_error = true` for a clone with invalid `fixture_set` | `build_clone_pairs` returns the E-DEMO-001 error; `start_all` logs WARN and skips that clone per existing continue-on-error semantics |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-018-001 | Two Claroty clones: clone_A `seed=100 fixture_set="default"`, clone_B `seed=200 fixture_set="default"` | `ids_A ∩ ids_B = ∅` (INV-DISTINCT-DATA-001) | happy-path |
| TV-018-002 | Single clone `seed=42 fixture_set="default"` built twice (two harness runs) | Both runs return byte-identical `/devices` response bodies (Postcondition 3 + BC-3.4.001) | happy-path |
| TV-018-003 | `fixture_set="default"` for all clones; compare to pre-seeding baseline | All existing integration tests pass (Postcondition 4, backward compat) | regression |
| TV-018-004 | `fixture_set="xyzzy_unknown"` for Claroty clone | `build_clone_pairs` returns `Err` containing E-DEMO-001 message | error-path |
| TV-018-005 | `fixture_set="compromised"` for Claroty clone | Clone constructed with `Archetype::CompromisedEndpoint`; `/alerts` response contains ≥3 alerts with `severity_id >= 4` (per BC-3.4.003 archetype baseline) | happy-path |
| TV-018-006 | `fixture_set="dormant"` for Armis clone | Clone constructed with `Archetype::DormantTenant`; `/devices` response returns empty list | happy-path |
| TV-018-007 | ThreatIntel clone `fixture_set="default"` | Clone constructs successfully; data identical to pre-seeding `ThreatIntelClone::new()` | regression |
| TV-018-008 | Four generator-backed clones with seeds 1, 2, 3, 4 | All six pairwise ID-set intersections are empty | happy-path |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-018-A | `∀ clone ∈ {claroty, armis, crowdstrike, cyberint}: seed_A ≠ seed_B ⇒ ids(A) ∩ ids(B) = ∅` | integration test (TV-018-001, TV-018-008) |
| VP-018-B | Backward compat: `new_with_seed(42, HealthyOtEnvironment, default_org)` produces same data as pre-seeding `new()` for all generator-backed clones | regression test (TV-018-003) |
| VP-018-C | `fixture_set → Archetype` mapping is exhaustive: all 8 valid strings map to correct variants; any other string returns E-DEMO-001 | unit test of mapping function (INV-FIXTURE-SET-ARCHETYPE-MAP-001 table) |
| VP-018-D | Construction-time error on invalid `fixture_set` propagates through `build_clone_pairs` without panic | unit test (TV-018-004) |
| VP-018-E | Process-restart determinism: same seed + fixture_set + org_id ⇒ byte-identical responses | integration test (TV-018-002) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-036 ("Multi-Tenant DTU Test Harness (Internal)") per capabilities.md §CAP-036 |
| Capability Anchor Justification | CAP-036 ("Multi-Tenant DTU Test Harness (Internal)") per capabilities.md §CAP-036 — this BC specifies the per-clone seeding wiring in `build_clone_pairs`, which is the demo-server's harness-layer mechanism for constructing per-customer DTU clone instances with distinct fixture data. CAP-036 defines the multi-tenant DTU test harness as "orchestrating per-customer DTU clone instances" with "per-org fixture data via the deterministic generator (ADR-009)". Seeding wiring is a core orchestration behavior of the harness; it is not generator-internal behavior (CAP-039) or client configuration loading (CAP-009). |
| L2 Domain Invariants | N/A (demo-server seeding wiring; no DI-NNN in L2 domain spec maps to this concern) |
| Architecture Module | SS-01 (Sensor Adapters) per ARCH-INDEX.md; `prism-dtu-demo-server` and `prism-dtu-common` are the implementation sites |
| Stories | S-DEMO-DTU-DATA-SEEDING-001 |
| Upstream BC | BC-3.4.001 (Generator Determinism — Identical Inputs Produce Byte-Identical FixtureSet) — this BC operates at the wiring layer above BC-3.4.001; generation-layer determinism is delegated to BC-3.4.001 |

## Related BCs

- BC-3.4.001 — referenced by (this BC wires seed/archetype inputs to BC-3.4.001's generation layer; BC-3.4.001's postconditions 3/4/6 are the guarantee mechanism for Postcondition 3 here)
- BC-3.4.003 — referenced by (Archetype baseline counts used in TV-018-005/006 and archetype catalog defined there)
- BC-3.4.004 — referenced by (org-tagged ID prefix format `dev-{org_slug}-{seed}-{index}` used in INV-DISTINCT-DATA-001 justification)
- BC-2.06.017 — sibling (both govern demo-server config-wiring; BC-2.06.017 governs multi-address binding, this BC governs data seeding)
- BC-2.06.001 — depends on (TOML config must load successfully before `build_clone_pairs` runs)

## Architecture Anchors

- `crates/prism-dtu-demo-server/src/harness.rs` — `build_clone_pairs` function; site of the GAP being closed (currently calls `CloneType::new()` ignoring `clone_cfg.seed` and `clone_cfg.fixture_set`)
- `crates/prism-dtu-demo-server/src/config.rs` — `CloneConfig.seed: u64` (default 42) and `CloneConfig.fixture_set: String` (default "default"); existing fields
- `crates/prism-dtu-common/src/generator/archetype.rs` — `Archetype` enum; `all_archetypes()` catalog
- `crates/prism-dtu-common/src/generator/rng.rs` — `seeded_rng(seed, org_id)` — the determinism primitive
- `crates/prism-dtu-common/src/generator/opts.rs` — `GenOpts` — seed is passed via `GenOpts::seed`

## Story Anchor

S-DEMO-DTU-DATA-SEEDING-001

## VP Anchors

- VP-018-A through VP-018-E (above) — all verified by integration/unit tests in S-DEMO-DTU-DATA-SEEDING-001

## Open Questions

None at BC authoring time. The architect decision (this session) resolved:
- **OQ-1 (mechanism):** Config-time seeding via `new_with_seed` is the primary path; `POST /dtu/configure` remains a secondary override (INV-CONFIGURE-ENDPOINT-SECONDARY-001).
- **OQ-2 (static clones):** ThreatIntel and NVD use `fixture_set` for named fixture file selection; serving the same file across clients is acceptable (INV-STATIC-CLONE-FIXTURE-SET-001).
- **OQ-3 (error handling):** Unrecognized `fixture_set` → construction-time E-DEMO-001 (INV-CONSTRUCTION-TIME-FAILURE-001); no silent fallback.

## BC Changelog

| Version | Change |
|---------|--------|
| v1.0 | Initial authoring. Product-owner decision: BC-2.06.018 namespace chosen over BC-3.4.005 — this BC governs demo-server config-wiring layer (how `DemoConfig` fields feed `build_clone_pairs`), not generator internals; BC-2.06 is the established namespace for demo-server config-wiring BCs (BC-2.06.017 precedent). Subsystem: SS-01 per ARCH-INDEX.md (prism-dtu-demo-server is owned by Sensor Adapters / SS-01). Capability: CAP-036 ("Multi-Tenant DTU Test Harness") — the harness orchestration capability, not CAP-039 (generator internals) or CAP-009 (production client configuration). |
