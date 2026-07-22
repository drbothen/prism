---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.018"
version: "1.7"
status: active
lifecycle_status: active
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-036"
introduced: "2026-06-09"
modified: "2026-07-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-DEMO-DTU-LIVE-SCENARIO-001-A]
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
input-hash: "3000000"
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

## Substrate Reality (ADR-036 v2.0 §1.3)

**As of ADR-036 v2.0 substrate correction (D-1078), the seeding postconditions of this BC
are UNIMPLEMENTED until Story A (`S-DEMO-DTU-LIVE-SCENARIO-001-A`) closes.** The verified
code reality:

- The demo-server generator-backed clones (CrowdStrike, Armis, Claroty, Cyberint) do NOT
  currently call `generate()` in their serving paths. `CrowdstrikeClone::new()` creates
  an empty stateful write-target; `ArmisClone::new()` loads static JSON fixtures. No
  `generate()` call exists in `build_clone_pairs()` or any clone constructor called from it.
- `CloneConfig.seed` is declared in `config.rs` (default `42`) but is **never read** in
  `build_clone_pairs()`. This BC's seeding postconditions are real retrofit requirements,
  not config-tweak work.
- `DemoConfig` and `CloneConfig` currently have no `org_id` field. The `OrgId` required
  by `seeded_rng(seed, &OrgId)` has no source in the demo-server today.

**Story A RETROFIT:** A new per-clone `new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId)` constructor
calls `generate(...)` under `#[cfg(feature = "fixture-gen")]`, stores the resulting records
in a new state field (`generated_records: Vec<serde_json::Value>`), and route handlers serve
or stage-filter from `generated_records` when present. CrowdStrike and Claroty clones return
`Self` directly; Armis and Cyberint clones return `anyhow::Result<Self>`. The existing
`new()` static-JSON path is unchanged for backward compatibility.

### Canonical Org Slug and Device ID Format

Per ADR-036 v2.0 §2.2 (authoritative, replaces any earlier placeholder format):

- **Org slug derivation:** `org_slug = hex(org_id.as_bytes()[0..4])` — 8 lowercase hex
  characters derived from the first 4 bytes of the OrgId UUID. Example: OrgId whose bytes
  start `[0xde, 0xad, 0xbe, 0xef, ...]` produces `org_slug = "deadbeef"`.
- **Canonical device ID format:** `"dev-{org_slug}-{seed}-{n}"` where `{org_slug}` is the
  8-hex-char value above, `{seed}` is the u64 seed, and `{n}` is a zero-based record index.
  Example: `"dev-deadbeef-42-0"` for the first device with seed=42 and org above.
- **No `"dev-acme-..."` format exists.** Any test, spec, or example using `"dev-acme-..."` is
  incorrect and must be updated to use the canonical `"dev-{8hex}-{seed}-{n}"` form.
- `seeded_rng(seed, org_id: &OrgId)` takes `&OrgId` (a `[u8;16]`-backed UUID) — NOT `&str`.
  CrowdStrike generator derives `org_slug` internally via `org_slug(org_id: &OrgId) -> String`.
  Armis generator takes `org_slug: &str` as an explicit argument; the harness passes the
  catalog-derived slug.

### New Config Requirement: `CloneConfig.org_id`

`DemoConfig`/`CloneConfig` must gain `org_id: Option<String>` (UUID string → parsed to
`OrgId`) to provide the `OrgId` required by `seeded_rng` and `ScenarioEntityCatalog`
derivation. This field is:
- Required when `scenario.enabled = true` for any clone in the same client config block;
  absence produces **E-DEMO-004** at construction time.
- Required for `new_with_seed` to derive the canonical org slug and disjoint device IDs.
- Optional (may be `None`) when `scenario.enabled = false` and caller uses the backward-compat
  `new()` path; however, seed-based data disjointness then cannot be guaranteed across orgs.
- Must be a valid UUID string; a non-UUID value produces **E-DEMO-005** at construction time.

## Preconditions

1. `DemoConfig` has been parsed from a valid TOML file (BC-2.06.001).
2. Each `CloneConfig` has a `seed: u64` field (default `42`) and a `fixture_set: String`
   field (default `"default"`), both already present in
   `crates/prism-dtu-demo-server/src/config.rs::CloneConfig`.
3. The `fixture-gen` Cargo feature is enabled in `prism-dtu-demo-server`'s feature set
   when generator-backed clones are included in the build.
4. `CloneConfig.org_id: Option<String>` is present in `DemoConfig`/`CloneConfig` (new field
   added by Story A). When `scenario.enabled = true` or when `new_with_seed` is called, this
   field must contain a valid UUID string that is parsed to `OrgId` (`[u8;16]`). Absence
   produces E-DEMO-004; non-UUID value produces E-DEMO-005. The `OrgId` is used to call
   `seeded_rng(seed, &org_id)` and to derive `org_slug = hex(org_id.as_bytes()[0..4])`
   per the canonical formula in the Substrate Reality section above.
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
  (via `prism_dtu_common::generator::seeded_rng(seed, &org_id)` where `org_id: &OrgId`
  is the `[u8;16]`-backed UUID per BC-3.4.001 and ADR-036 v2.0 §2.2).

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
the affected integration tests in the same story as S-DEMO-DTU-LIVE-SCENARIO-001-A. This
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
- Every generated record's primary ID uses the canonical format
  `"dev-{org_slug}-{seed}-{index}"` where `org_slug = hex(org_id.as_bytes()[0..4])` (8 hex
  chars). This formula is identical across all generator-backed clones for the same
  `(seed, org_id)` pair, making IDs structurally disjoint across distinct `(seed, org_id)`
  tuples (ADR-036 v2.0 §2.2, Substrate Reality section above).

The invariant is verified by integration tests in S-DEMO-DTU-LIVE-SCENARIO-001-A that
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

## Scope Boundary — Non-Generator-Backed Tables

This BC's seeding/serving contract covers **only the generator-backed clone table surfaces
that the four DTU generators actually emit** as of Story A (`S-DEMO-DTU-LIVE-SCENARIO-001-A`).
The covered surfaces are:

| Clone | Generator-Backed Table Surface(s) | Route(s) Served |
|-------|------------------------------------|-----------------|
| CrowdStrike | devices, detections | `/devices/queries/devices/v1`, `/devices/entities/devices/v2` (device two-step), `/detects/queries/detects/v1`, `/detects/entities/summaries/GET/v1` (detection two-step), plus `/oauth2/token` and write routes |
| Armis | devices, alerts | `/api/v1/search` (primary AQL endpoint; backs both `devices` and `alerts` tables via `?aql=` path_template), plus `/api/v1/devices` and `/api/v1/alerts` (direct-access compatibility endpoints) |
| Claroty | devices, alerts | `/api/v1/devices` (POST), `/api/v1/alerts` (POST). Note: `/api/v1/audit_log/get` and `/api/v1/vulnerabilities` routes are also registered but backed by static fixtures — NOT generator-backed and NOT covered by INV-DISTINCT-DATA-001 under this BC. |
| Cyberint | alerts only (adapter-consumed) | `/api/v1/alerts` (GET and POST). The Cyberint generator additionally emits `asm_asset`, `cve`, and `ioc` surfaces, but NO routes for those surfaces are registered in `prism-dtu-cyberint`, and `cyberint.sensor.toml` declares no corresponding adapter tables. These surfaces are intentionally generated-but-unserved (see subsection below). |

**INV-DISTINCT-DATA-001 applies only to the generator-backed table surfaces listed above.**
Per-clone seeding produces disjoint ID sets across the listed generator-backed routes for
distinct `(seed, org_id)` pairs. It does NOT apply to the Cyberint `asm_asset`/`cve`/`ioc`
surfaces (generated-but-unserved) or the Claroty `audit_log`/`vulnerabilities` routes
(static-fixture-backed).

### Cyberint Generator-Emitted-but-Unserved Surfaces (asm_asset, cve, ioc)

The Cyberint generator (`crates/prism-dtu-cyberint/src/generator.rs`) emits records for
four surfaces: `alert`, `asm_asset`, `cve`, and `ioc`. However, `CyberintClone::build_router`
registers routes for **only** the `alert` surface (`/api/v1/alerts`) and for threat intel
(`/api/v1/threat-intel`, static fixture). There are no `/api/v1/asm_assets`, `/api/v1/cves`,
or `/api/v1/iocs` routes. `cyberint.sensor.toml` likewise declares only the `alerts` adapter
table (and the documented-gap `incidents` table). Consequently:

1. **The `asm_asset`, `cve`, and `ioc` generator surfaces are generated-but-unserved under
   this BC.** They are produced by the generator but filtered out because no route or adapter
   table consumes them. The `_surface=="alert"` discriminator in the `/api/v1/alerts` handler
   ensures only alert records are served; the remaining three surfaces go unused at runtime.

2. **INV-DISTINCT-DATA-001 does NOT apply to the Cyberint `asm_asset`, `cve`, or `ioc`
   surfaces under this BC.** There are no serving routes for these surfaces and no adapter
   tables that would consume them.

3. **This is a confirmed scope boundary, not a defect.** The three unserved surfaces are a
   consequence of the existing DTU route registration — the generator scope was defined ahead
   of the adapter route scope. Source: Local adversary cascade pass 4, finding `F-P4-MED-003`
   (`S-DEMO-DTU-LIVE-SCENARIO-001-A`).

4. **Adding routes for `asm_asset`, `cve`, and `ioc` surfaces is future feature work.** If the
   demo needs these surfaces served, a follow-up story (separate from
   `S-DEMO-CYBERINT-INCIDENTS-SEEDING-001`) is required to register the routes and add the
   corresponding adapter tables to `cyberint.sensor.toml`. The generator already produces the
   data; the work is on the route-registration and sensor-spec side.

### Cyberint `incidents` Table — Intentionally Non-Generator-Backed

The Cyberint generator emits **no incident records**. The `asm_asset`, `cve`, and `ioc`
surfaces it does emit are generated-but-unserved (see subsection above). Incidents are a
separate gap:

1. **No `/api/v1/incidents` route is registered in `prism-dtu-cyberint`** — the Cyberint DTU
   clone has no incidents route. This is documented in `crates/prism-sensors/specs/cyberint.sensor.toml`
   via edge case `EC-016-013-002` ("DTU has no incidents route").

2. **INV-DISTINCT-DATA-001 does NOT apply to the Cyberint `incidents` table under this BC.**
   There are no incident records to seed, and serving an empty `/api/v1/incidents` endpoint
   would misrepresent the actual Cyberint API surface the generator covers.

3. **This is a confirmed scope boundary, not a gap or defect.** The absence of Cyberint
   incident seeding is a legitimate API coverage limitation of the existing Cyberint generator.
   Source: Local adversary cascade pass 2, finding `F-P2-MED-001`
   (`S-DEMO-DTU-LIVE-SCENARIO-001-A`), confirmed during implementation.

4. **Adding a Cyberint incidents generator surface is future feature work** tracked by follow-up
   story `S-DEMO-CYBERINT-INCIDENTS-SEEDING-001`. That story will extend the Cyberint generator
   to emit incident records and register the `/api/v1/incidents` route in `prism-dtu-cyberint`,
   at which point this scope boundary will be lifted and this subsection updated.

   **Amendment (D-1889 / ADR-053 §Finding-1):** `S-DEMO-CYBERINT-INCIDENTS-SEEDING-001` has
   been RETIRED. ADR-053 §Finding-1 determined the Cyberint API exposes no real incidents
   endpoint (phantom endpoint). The incidents gap is closed by `DEFECT-CYBERINT-SPEC-FIDELITY-001`,
   which deletes the `incidents` table from `cyberint.sensor.toml`. This scope boundary stands
   permanently — no route will be added.

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

E-DEMO-001 is registered in `.factory/specs/prd-supplements/error-taxonomy.md` under the
`## DEMO: Demo-Server Errors` section (registered in the BC-2.06.019/020 authorship burst,
v1.63 of error-taxonomy.md).

Two additional error codes govern the new `org_id` config requirement (see Substrate Reality
section, "New Config Requirement"):

| Field | E-DEMO-004 | E-DEMO-005 |
|-------|-----------|-----------|
| Code | `E-DEMO-004` | `E-DEMO-005` |
| Category | configuration | configuration |
| Severity | broken | broken |
| Exit code | `1` (startup failure) | `1` (startup failure) |
| Message format | `"demo-server: E-DEMO-004: clone '{clone_name}': scenario.enabled requires org_id to be set (UUID string)"` | `"demo-server: E-DEMO-005: clone '{clone_name}': org_id '{value}' is not a valid UUID"` |
| Trigger | `scenario.enabled = true` (or `new_with_seed` called) but `CloneConfig.org_id` is `None` | `CloneConfig.org_id` is present but fails `uuid::Uuid::parse_str()` |
| Recoverable | No — operator must add `org_id = "<uuid>"` to `demo.toml` and restart | No — operator must fix the UUID format and restart |

Both codes are detected by `build_clone_pairs` before any clone constructor is called, and
propagated through `build_clone_pairs -> anyhow::Result<Vec<ClonePair>>`. Registered in
error-taxonomy.md §DEMO (v1.64, same correction burst as BC-2.06.018 v1.1).

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
| Stories | S-DEMO-DTU-LIVE-SCENARIO-001-A |
| Upstream BC | BC-3.4.001 (Generator Determinism — Identical Inputs Produce Byte-Identical FixtureSet) — this BC operates at the wiring layer above BC-3.4.001; generation-layer determinism is delegated to BC-3.4.001 |

## Related BCs

- BC-3.4.001 — referenced by (this BC wires seed/archetype inputs to BC-3.4.001's generation layer; BC-3.4.001's postconditions 3/4/6 are the guarantee mechanism for Postcondition 3 here)
- BC-3.4.003 — referenced by (Archetype baseline counts used in TV-018-005/006 and archetype catalog defined there)
- BC-3.4.004 — referenced by (org-tagged ID prefix format `"dev-{org_slug}-{seed}-{index}"` referenced in INV-DISTINCT-DATA-001; authoritative formula is now ADR-036 v2.0 §2.2: `org_slug = hex(org_id.as_bytes()[0..4])`)
- BC-2.06.017 — sibling (both govern demo-server config-wiring; BC-2.06.017 governs multi-address binding, this BC governs data seeding)
- BC-2.06.001 — depends on (TOML config must load successfully before `build_clone_pairs` runs)

## Architecture Anchors

- `crates/prism-dtu-demo-server/src/harness.rs` — `build_clone_pairs` function; site of the GAP being closed (currently calls `CloneType::new()` ignoring `clone_cfg.seed` and `clone_cfg.fixture_set`)
- `crates/prism-dtu-demo-server/src/config.rs` — `CloneConfig.seed: u64` (default 42) and `CloneConfig.fixture_set: String` (default "default"); existing fields
- `crates/prism-dtu-common/src/generator/archetype.rs` — `Archetype` enum; `all_archetypes()` catalog
- `crates/prism-dtu-common/src/generator/rng.rs` — `seeded_rng(seed: u64, org_id: &OrgId)` — the determinism primitive; takes `&OrgId` ([u8;16]), NOT `&str`
- `crates/prism-dtu-common/src/generator/opts.rs` — `GenOpts` — seed is passed via `GenOpts::seed`

## Story Anchor

S-DEMO-DTU-LIVE-SCENARIO-001-A

## VP Anchors

- VP-018-A through VP-018-E (above) — all verified by integration/unit tests in S-DEMO-DTU-LIVE-SCENARIO-001-A

## Open Questions

None at BC authoring time. The architect decision (this session) resolved:
- **OQ-1 (mechanism):** Config-time seeding via `new_with_seed` is the primary path; `POST /dtu/configure` remains a secondary override (INV-CONFIGURE-ENDPOINT-SECONDARY-001).
- **OQ-2 (static clones):** ThreatIntel and NVD use `fixture_set` for named fixture file selection; serving the same file across clients is acceptable (INV-STATIC-CLONE-FIXTURE-SET-001).
- **OQ-3 (error handling):** Unrecognized `fixture_set` → construction-time E-DEMO-001 (INV-CONSTRUCTION-TIME-FAILURE-001); no silent fallback.

## BC Changelog

| Version | Change |
|---------|--------|
| v1.7 | 2026-07-22 — Annotate §Cyberint `incidents` Table item 4: `S-DEMO-CYBERINT-INCIDENTS-SEEDING-001` RETIRED per D-1889 / ADR-053 §Finding-1 (phantom Cyberint incidents endpoint). Incidents gap closed by `DEFECT-CYBERINT-SPEC-FIDELITY-001` (table deletion from `cyberint.sensor.toml`). Scope boundary stands permanently. Input-hash populated (was empty). |
| v1.6 | 2026-06-10 — POL-14 lifecycle promotion (D-1089). S-DEMO-DTU-LIVE-SCENARIO-001-A (T4-A) squash-merged via PR #181 develop@c287b00d. BC promoted draft→active per POL-14 auto-promotion policy. All Postconditions and Invariants are now implemented in develop. status: draft→active; lifecycle_status: draft→active. |
| v1.5 | 2026-06-09 — Internal contradiction fix (F-P6-HIGH-001, LOCAL adversary pass 6; user decision: implement full 8-archetype support; ADR-036 v2.2 reconciliation). §Story A RETROFIT paragraph (line ~75) carried a **2-arg** `new_with_seed(seed: u64, org_id: OrgId)` form that contradicted the canonical **3-arg** `new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId)` specified by Postcondition 1, INV-FIXTURE-SET-ARCHETYPE-MAP-001, EC-018-003/005, TV-018-005/006, and VP-018-C. The 2-arg form was ADR-036 drift that the implementation followed, hardcoding `CompromisedEndpoint`. Fixed: §Story A RETROFIT now cites the canonical 3-arg signature with return-type note (CrowdStrike/Claroty return `Self`; Armis/Cyberint return `anyhow::Result<Self>`) per ADR-036 v2.2. All other live-narrative `new_with_seed` occurrences (lines 46, 105, 119, 135, 175, 185, 381, 421, 464) were already 3-arg or reference-only and required no change. PC-1, INV-FIXTURE-SET-ARCHETYPE-MAP-001, EC-018-003 (dormant→empty), EC-018-005 (large_scale→10 000), TV-018-005 (compromised→≥3 alerts severity≥4), TV-018-006 (dormant→empty list), and VP-018-C (exhaustive 8-variant mapping) verified internally consistent — no further changes needed. lifecycle_status remains draft. |
| v1.4 | 2026-06-09 — Phantom story-anchor correction (F-P5-HIGH-001, `S-DEMO-DTU-LIVE-SCENARIO-001-A` pass 5). All six live narrative sites referencing the non-existent planning-era story ID `S-DEMO-DTU-DATA-SEEDING-001` replaced with the real implementing story `S-DEMO-DTU-LIVE-SCENARIO-001-A` per STORY-INDEX (D-1077 split). Sites updated: frontmatter `anchored_stories`, Postcondition 4 prose, INV-DISTINCT-DATA-001 prose, §Traceability `Stories` row, §Story Anchor, §VP Anchors. Changelog rows untouched (TD-VSDD-091 exempt). lifecycle_status remains draft. |
| v1.3 | 2026-06-09 — §Scope Boundary table factual correction (F-P4-MED-003, `S-DEMO-DTU-LIVE-SCENARIO-001-A` pass 4). **Cyberint row corrected:** "Route(s) Served" was `/api/v1/alerts, /api/v1/asm_assets, /api/v1/cves, /api/v1/iocs`; corrected to `/api/v1/alerts` only — the three non-alert routes do not exist in `CyberintClone::build_router` and have no entries in `cyberint.sensor.toml`. "Generator-Backed Table Surface(s)" corrected to "alerts only (adapter-consumed)". Added new subsection §Cyberint Generator-Emitted-but-Unserved Surfaces documenting that the Cyberint generator does emit `asm_asset`/`cve`/`ioc` surfaces but they are generated-but-unserved (no routes, no adapter tables); future follow-up story required to serve them. **CrowdStrike row corrected:** route paths corrected from non-existent `/device/api/devices/v2` and `/device/api/alerts/v1` to actual registered routes `/devices/queries/devices/v1`, `/devices/entities/devices/v2`, `/detects/queries/detects/v1`, `/detects/entities/summaries/GET/v1`; surface name corrected from "alerts" to "detections" (matching `crowdstrike.sensor.toml` table name). **Armis row updated:** added `/api/v1/search` as the primary AQL endpoint backing both tables via path_template (per `armis.sensor.toml`); `/api/v1/devices` and `/api/v1/alerts` noted as direct-access compatibility endpoints. **Claroty row updated:** routes corrected to POST method paths per `ClarotyClone::build_router`; added note that `/api/v1/audit_log/get` and `/api/v1/vulnerabilities` routes exist but are static-fixture-backed, not generator-backed, and not covered by INV-DISTINCT-DATA-001 under this BC. lifecycle_status remains draft. |
| v1.2 | 2026-06-09 — Scope boundary documentation (F-P2-MED-001, `S-DEMO-DTU-LIVE-SCENARIO-001-A` pass 2). Added §Scope Boundary — Non-Generator-Backed Tables: enumerates the four generator-backed clone table surfaces covered by INV-DISTINCT-DATA-001 (CrowdStrike: devices/alerts; Armis: devices/alerts; Claroty: devices/alerts; Cyberint: alerts/asm_assets/cves/iocs). Explicitly documents that the Cyberint `incidents` table is intentionally NON-generator-backed for this BC — the Cyberint generator emits no incident records; the DTU clone has no `/api/v1/incidents` route (cross-referenced: `cyberint.sensor.toml` EC-016-013-002); INV-DISTINCT-DATA-001 therefore does not apply to this table surface. Notes that adding a Cyberint incidents generator surface is future work tracked by `S-DEMO-CYBERINT-INCIDENTS-SEEDING-001`. lifecycle_status remains draft. |
| v1.1 | ADR-036 v2.0 / D-1078 substrate-reconciliation corrections. Added §Substrate Reality (ADR-036 v2.0 §1.3): documents that seeding postconditions are UNIMPLEMENTED until Story A (`S-DEMO-DTU-LIVE-SCENARIO-001-A`); clones serve static JSON today with no `generate()` call in `build_clone_pairs()`. Added canonical org_slug derivation formula (`hex(org_id.as_bytes()[0..4])`; 8 hex chars) and canonical device ID format (`"dev-{org_slug}-{seed}-{n}"`) per ADR-036 v2.0 §2.2 — replaces incorrect `"dev-acme-..."` placeholder. Removed `"dev-acme-..."` reference from INV-DISTINCT-DATA-001. Added "New Config Requirement" for `CloneConfig.org_id: Option<String>` (UUID string → OrgId). Corrected `seeded_rng` signature to `seeded_rng(seed: u64, org_id: &OrgId)` (takes `&OrgId`, NOT `&str`) in Postcondition 1, INV-DISTINCT-DATA-001, and Architecture Anchors. Registered E-DEMO-004 (scenario.enabled but org_id absent) and E-DEMO-005 (org_id not valid UUID) in §Error Codes. Updated Error Codes section to note E-DEMO-001 already registered in error-taxonomy.md v1.63. Precondition 4 rewritten to match new `CloneConfig.org_id: Option<String>` field. lifecycle_status remains draft. |
| v1.0 | Initial authoring. Product-owner decision: BC-2.06.018 namespace chosen over BC-3.4.005 — this BC governs demo-server config-wiring layer (how `DemoConfig` fields feed `build_clone_pairs`), not generator internals; BC-2.06 is the established namespace for demo-server config-wiring BCs (BC-2.06.017 precedent). Subsystem: SS-01 per ARCH-INDEX.md (prism-dtu-demo-server is owned by Sensor Adapters / SS-01). Capability: CAP-036 ("Multi-Tenant DTU Test Harness") — the harness orchestration capability, not CAP-039 (generator internals) or CAP-009 (production client configuration). |
