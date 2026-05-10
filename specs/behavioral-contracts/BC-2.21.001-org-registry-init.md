---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-08T00:00:00Z
phase: 3
origin: greenfield
subsystem: "SS-21"
capability: "CAP-038"
lifecycle: active
anchored_stories: [S-WAVE5-PREP-01]
verifying_vps: []
crates: [prism-bin, prism-core]
inputs:
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/architecture/module-decomposition.md
input-hash: "[md5]"
traces_to: ["CAP-038"]
---

# BC-2.21.001: OrgRegistry Initialization — Bijective Resolution Verified at Process Start

## Description

This BC is the Identity & Core Types subsystem's (SS-21) startup-time contract. It specifies
how `prism-bin` constructs and verifies the `OrgRegistry` at boot step 3 (per ADR-022 §B).
The orchestration of this and the other 3 subsystem init contracts in §B order is specified
separately in BC-2.22.001.

`prism-bin` constructs the `OrgRegistry` from the `PrismConfig` produced in step 2. The registry
loads all configured `(org_id, org_slug)` pairs (per ADR-006), verifies that the bijectivity
invariant holds (no two slugs share an org_id, no two org_ids share a slug), and makes the
registry available to all subsequent boot steps. This step is BLOCKING: step 4 (sensor TOML spec
load) does not begin until OrgRegistry construction and verification complete. On any failure
(empty org list, duplicate org_id, duplicate org_slug, malformed org_slug, registry construction
failure), the process exits with code 2.

No `todo!()`, `unimplemented!()`, or `panic!("stub...")` may appear in the production code path
for this step at or after story S-WAVE5-PREP-01 merges (POL-12 enforcement).

## Preconditions

- BC-2.06.011 is satisfied: `PrismConfig` handle is valid and available
- Boot step 2 (config load) has completed without error
- The config declares at least one `[[org]]` entry (validated in step 2 schema check)

## Postconditions

**Happy path:**
- `OrgRegistry` is constructed from all `(org_id, org_slug)` pairs in the config
- Bijectivity is verified at construction time (not lazily at first lookup):
  - Each `org_id` is unique across all entries
  - Each `org_slug` is unique across all entries
  - `OrgRegistry::resolve_slug(slug)` returns exactly the corresponding `org_id` with O(1)
  - `OrgRegistry::resolve_id(org_id)` returns exactly the corresponding `org_slug` with O(1)
- The `OrgRegistry` handle (wrapped in `Arc`) is available to all subsequent boot steps
- Boot continues to step 4 (sensor TOML spec load) per ADR-022 §B ordering

**Failure path — empty org list:**
- The process emits a `tracing::error!("Config must declare at least one org")`
- The process exits with code **2** (config-invalid) per ADR-022 §A
- Step 4 never begins

**Failure path — duplicate org_id:**
- The process emits a `tracing::error!` identifying the duplicated org_id value
- The process exits with code **2**
- Step 4 never begins

**Failure path — duplicate org_slug:**
- The process emits a `tracing::error!` identifying the duplicated slug value
- The process exits with code **2**
- Step 4 never begins

**Failure path — malformed org_slug:**
- Org slug must be kebab-case (lowercase alphanumeric and hyphens, non-empty, no leading/trailing
  hyphens per ADR-006 slug format)
- The process emits a `tracing::error!` identifying the malformed slug and format requirement
- The process exits with code **2**
- Step 4 never begins

## Invariants

- Boot step 3 is blocking: no concurrent execution with step 4 or later (ADR-022 §B "Traffic gate")
- Exit code on any OrgRegistry failure is exactly 2, never 1, 4, or 5 (ADR-022 §A)
- Bijectivity is enforced atomically at construction time, not lazily at lookup time (per ADR-006
  and CAP-038: "enforced atomically at registration time, not lazily at lookup time")
- `OrgRegistry::register` returns `Ok` for idempotent exact-duplicate re-registration and a
  `RegistrationError` for true bijectivity violations (per CAP-038 / D-050)
- The registry is read-only after construction; no runtime API can mutate it except a full
  config reload (which is handled by a separate boot-after-reload path, not this boot BC)

## Error Cases

| Error Code | Condition | Behavior |
|------------|-----------|----------|
| Exit 2 | No `[[org]]` entries in config | "Config must declare at least one org"; exit 2 |
| Exit 2 | Two orgs share the same `org_id` | "Duplicate org_id: {uuid}"; exit 2 |
| Exit 2 | Two orgs share the same `org_slug` | "Duplicate org_slug: {slug}"; exit 2 |
| Exit 2 | `org_slug` fails kebab-case validation | "Invalid org_slug '{slug}': must be lowercase kebab-case"; exit 2 |
| Exit 2 | `org_id` fails UUID v7 format validation | "Invalid org_id '{value}': must be a UUID v7"; exit 2 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-21-001-001 | Exactly one org entry (minimum valid config) | Registry constructed with 1 entry; boot continues |
| EC-21-001-002 | `org_slug` is empty string | Malformed; exit 2 |
| EC-21-001-003 | `org_slug` has leading hyphen (`-acme`) | Malformed; exit 2 |
| EC-21-001-004 | `org_slug` has uppercase characters (`ACME`) | Malformed; exit 2 |
| EC-21-001-005 | Two orgs: same `org_id`, different `org_slug` | Bijectivity violated; exit 2 with "Duplicate org_id" |
| EC-21-001-006 | Two orgs: same `org_slug`, different `org_id` | Bijectivity violated; exit 2 with "Duplicate org_slug" |
| EC-21-001-007 | 100 valid org entries | Registry constructs successfully; all 100 orgs resolvable; boot continues |
| EC-21-001-008 | `org_id` is not a valid UUID (random string) | "Invalid org_id '{value}': must be a UUID v7"; exit 2 |

## Canonical Test Vectors

| ID | Scenario | Config Input | Expected Exit Code | Expected Log Output |
|----|----------|-------------|-------------------|---------------------|
| TV-21-001-001 | Single valid org | `org_id = "0196f..." (uuid-v7)`, `org_slug = "acme"` | Boot continues | `tracing::info!("OrgRegistry initialized: 1 org(s)")` |
| TV-21-001-002 | Empty org list | Config has `orgs = []` | 2 | "Config must declare at least one org" |
| TV-21-001-003 | Duplicate org_id | Two entries with same UUID | 2 | "Duplicate org_id: 0196f..." |
| TV-21-001-004 | Duplicate org_slug | Two entries with slug `"acme"` | 2 | "Duplicate org_slug: acme" |
| TV-21-001-005 | Malformed slug | `org_slug = "ACME"` | 2 | "Invalid org_slug 'ACME': must be lowercase kebab-case" |
| TV-21-001-006 | Large registry (100 orgs) | 100 unique (org_id, org_slug) pairs | Boot continues | `tracing::info!("OrgRegistry initialized: 100 org(s)")` |

## Test Strategy

Integration tests in `crates/prism-bin/tests/boot_tests.rs` invoke `prism` as a subprocess.
Tests for this BC:

- `test_BC_2_21_001_single_org` — assert boot step 3 completes
- `test_BC_2_21_001_empty_orgs` — assert exit code 2 + error message "at least one org"
- `test_BC_2_21_001_duplicate_org_id` — assert exit code 2 + "Duplicate org_id"
- `test_BC_2_21_001_duplicate_slug` — assert exit code 2 + "Duplicate org_slug"
- `test_BC_2_21_001_malformed_slug` — assert exit code 2 + "lowercase kebab-case"

Bijectivity resolution can also be tested via unit test on `OrgRegistry::new()` in
`crates/prism-core/tests/org_registry_tests.rs` — this is the pure-core logic that
the boot step calls.

## Verification Properties

No formal VP is proposed. Bijectivity of the in-memory BiMap is a deterministic, pure function
of the input — the construction either succeeds or returns an error. The `OrgRegistry` lives in
`prism-core` (pure, per purity-boundary-map.md) and is a candidate for proptest-based property
testing (generate random (org_id, org_slug) pairs; assert no-duplicate input → Ok,
duplicate input → Err). Proposed as a future VP if the property test pattern is codified.

## Related BCs

- BC-2.22.001 — Boot Orchestration (orchestrates: this BC is one of 4 subsystem init contracts
  whose ordering and exit-code mapping are specified in BC-2.22.001)
- BC-2.06.011 — Config load (depends on: this BC requires BC-2.06.011 to have succeeded)
- BC-2.03.013 — Credential store init (depends on: step 5 requires this BC + BC-2.06.011)
- BC-2.05.012 — Audit subsystem init (depends on: step 6 requires all preceding BCs)
- BC-3.1.001 — OrgRegistry BiMap Construction (composes with: BC-3.1.001 specifies the core
  OrgRegistry behavioral contract; this BC specifies the boot-sequence invocation and exit-code
  contract around it)
- BC-3.1.003 — Bijective Resolution (enforced by: this BC's bijectivity verification step maps
  to the invariant specified in BC-3.1.003)

## Architecture Anchors

- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §B step 3 (boot step spec)
- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §A exit-code contract
- `specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md` (OrgId/OrgSlug identity model)
- `specs/architecture/module-decomposition.md` COMP-001 `prism-bin` (SS-22), COMP-012 `prism-core` (SS-21)
- `specs/domain-spec/capabilities.md` CAP-038 (Multi-Tenant Identity Model — defines bijectivity invariant)

## Story Anchor

S-WAVE5-PREP-01 — prism-bin: Binary Chassis, CLI, and Boot Sequence

## VP Anchors

None (see Verification Properties)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-038 |
| Capability Anchor Justification | CAP-038 ("Multi-Tenant Identity Model (Internal)") per capabilities.md §CAP-038 — this BC specifies the startup-time construction and bijectivity verification of the `OrgRegistry`, which is exactly what CAP-038 defines: "The registry is populated at startup from `customers/*.toml` files" with the bijectivity invariant "enforced atomically at registration time, not lazily at lookup time." |
| L2 Invariants | CAP-038 bijectivity invariant (enforced at construction time, not lazily). No standalone DI covers OrgRegistry boot ordering; the bijectivity property is specified in CAP-038 and ADR-006. |
| ADR Source | ADR-022 §B step 3, §A exit-code table; ADR-006 (org identity model) |
| Priority | P0 |
| POL-12 Note | The production code path satisfying this BC MUST contain no `todo!()`, `unimplemented!()`, or `panic!("stub...")` before S-WAVE5-PREP-01 transitions to `merged`. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | bundle-B-phase-B-1b-ss22-bcs-2026-05-08 | 2026-05-08 | product-owner | Initial authorship — Bundle B Phase B-1b SS-22 boot-sequence BCs |
| 1.0 | redirect-option-d-2026-05-08 | 2026-05-08 | product-owner | Relocated from BC-2.22.002 (SS-22) to BC-2.21.001 (SS-21 Identity & Core Types) per Option (d) decomposition. Capability anchor updated CAP-034 → CAP-038. EC/TV IDs renumbered to EC-21-001-NNN / TV-21-001-NNN. This is the FIRST BC under SS-21. |
| 1.1 | D-319-post-merge-state-burst | 2026-05-10 | state-manager | lifecycle draft → active per ADR-021 POL-14 (S-WAVE5-PREP-01 merged at develop@53b87961 PR #138 2026-05-10T00:55:49Z). First active BC under SS-21. |
