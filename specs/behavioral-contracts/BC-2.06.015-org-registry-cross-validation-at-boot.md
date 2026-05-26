---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.015"
version: "1.1"
status: active
lifecycle_status: active
producer: product-owner
timestamp: 2026-05-23T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
introduced: "2026-05-23"
modified: "2026-05-24"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-CONFIG-MULTI-TENANT-OVERRIDE-001]
verifying_vps: []
crates: [prism-spec-engine, prism-bin, prism-core]
inputs:
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/research/multi-tenant-sensor-endpoint-overrides-2026-05-23.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: ""
traces_to:
  - "CAP-009"
  - "ADR-029"
extracted_from: null
---

# BC-2.06.015: OrgRegistry Cross-Validation at Boot — Unknown Overlay Directory Triggers E-SPEC-022

## Description

During boot step 4 (`step4_load_sensor_specs`), after `OrgRegistry` has been populated (boot
step 3), the overlay validator cross-checks every `customers/<org_slug>/` directory entry
against the OrgRegistry. If a directory's `<org_slug>` does not correspond to a slug registered
in `OrgRegistry`, boot fails immediately with `E-SPEC-022`. This prevents config drift where
a stale customer directory persists after an org is removed from `prism.toml`, and prevents
typos in directory names from routing queries to incorrect endpoints without operator awareness.
The validation fires even if the directory contains no overlay files — the mere presence of an
unregistered `customers/<slug>/` directory is an error.

## Preconditions

- Boot step 3 has completed: `OrgRegistry` is populated from `customers/*.toml` config files
  (or equivalent org registration source per ADR-010).
- Boot step 4 has begun: the `customers/` subdirectory walk is in progress.
- Each subdirectory entry in `customers/` is being processed.

## Postconditions

### Success path — all directory slugs are registered

- Every `customers/<slug>/` directory has `<slug>` resolvable in `OrgRegistry` via
  `OrgRegistry::slug_exists(slug)` returning `true`.
- Validation passes; overlay loading for all directories proceeds normally.
- No error is emitted for this check.

### Failure path — unregistered slug directory found

- `OrgRegistry::slug_exists(slug)` returns `false` for a `customers/<slug>/` directory.
- `E-SPEC-022` is emitted with: directory path, unrecognized `org_slug`, and message:
  `"Per-org overlay directory 'customers/{slug}/' references org slug '{slug}' which is not registered in OrgRegistry. Check for typos or register the org in prism.toml [[orgs]]."`.
- Boot exits with code 2 (`BootError::ConfigInvalid`).
- No queries can be served until the stale directory is removed or the org is registered.

## Invariants

- INV-COMPAT-001: `customers/` directory entries and `OrgRegistry` entries are in bijective
  correspondence. An overlay directory with no registry entry, or a registry entry with no
  overlay directory, are both valid — but an overlay directory whose slug is not in the
  registry is always an error.
- INV-COMPAT-002: Org registration (boot step 3) always precedes overlay loading (boot step 4)
  per ADR-022 §B boot sequence. `OrgRegistry` is fully populated before this validation runs.
- INV-COMPAT-003: An empty `customers/<slug>/` directory (no `.sensor.toml` files) is still
  validated against `OrgRegistry`. The error fires on the directory's existence, not on the
  presence of overlay files within it.
- INV-COMPAT-004: `customers/.gitkeep` (and other non-directory entries in `customers/`) are
  not treated as org slug directories. Only subdirectory entries trigger the slug lookup.

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-022` | `customers/<slug>/` directory has no matching OrgRegistry entry | Boot exits 2; message includes the unrecognized slug and guidance to register or remove |
| Multiple unregistered directories | Two or more `customers/<slug>/` dirs are unregistered | All `E-SPEC-022` errors are collected and reported together (multi-error boot report, same pattern as BC-2.06.005) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-015-001 | Org removed from `prism.toml` but `customers/<slug>/` directory not cleaned up | `E-SPEC-022` at boot; operator must remove stale directory or re-add org |
| EC-015-002 | Typo in directory name (`customers/ame/` instead of `customers/acme/`) | `E-SPEC-022`; error message suggests the unrecognized slug; operator corrects the directory name |
| EC-015-003 | New org registered in `prism.toml` but no overlay directory created | Valid; no `customers/<slug>/` directory → no overlay → SaaS fallback or single-endpoint behavior. This is the expected initial state for a new org. |
| EC-015-004 | `customers/` is entirely absent (no directory) | Zero directory entries to validate; no `E-SPEC-022` possible; boot succeeds |
| EC-015-005 | `customers/<slug>/` exists but is empty (no `.sensor.toml` files) | `E-SPEC-022` check still fires; slug must be registered even for empty overlay directories |
| EC-015-006 | Both unregistered directory AND a valid overlay file for a different org exist | `E-SPEC-022` for the unregistered directory; boot fails; the valid overlay for the registered org is NOT loaded (fail-fast at first error per INV-SCALAR-003 pattern) |
| EC-015-007 | Slug in `customers/<slug>/` has valid registration but `prism.toml` org definition is subsequently removed without restart | Not detected until next boot or `reload_config`; the running process uses the boot-time `OrgRegistry` snapshot |

## Canonical Test Vectors

| Scenario | `customers/` dirs | `OrgRegistry` slugs | Expected Result |
|----------|-------------------|---------------------|-----------------|
| All registered | `acme/`, `contoso/` | `acme`, `contoso` | Validation passes; both overlays loaded |
| Stale dir | `acme/`, `stale-corp/` | `acme` | `E-SPEC-022` for `stale-corp`; boot fails |
| No overlay dirs | `.gitkeep` only | `acme`, `contoso` | Zero slugs to validate; boot succeeds |
| Typo in dir name | `armcorp/` | `acme` | `E-SPEC-022` for `armcorp` |
| Empty registered dir | `acme/` (no .sensor.toml) | `acme` | Slug is registered; validation passes; zero overlays from this dir |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none yet) | Red Gate test: `test_BC_2_06_015_unregistered_slug_dir_fails_boot` — creates `customers/unknown-org/armis.sensor.toml` with OrgRegistry containing only `acme`; expects `E-SPEC-022` and exit 2 |

## Related BCs

- BC-2.06.012 — Per-Tenant Overlay Loading and Merge Semantics (this validation precedes the merge step)
- BC-2.06.013 — Scalar-Only Overlay Enforcement (peer boot-time validation)
- BC-2.06.016 — Error Taxonomy for Override Violations (defines E-SPEC-022 canonical message)
- BC-2.21.001 — OrgRegistry Initialization (boot step 3; OrgRegistry must be populated before this BC runs)
- BC-2.06.011 — ConfigManager Initialization (boot step ordering context)

## Architecture Anchors

- ADR-029 §Boot-Time Validation: step 3c "Verify the directory name matches a registered `OrgRegistry` slug (E-SPEC-021)" (note: ADR-029 originally proposed E-SPEC-021 for this error; we allocate E-SPEC-022 per error-taxonomy collision resolution — E-SPEC-021 reserved for `[[tables]]` override)
- ADR-029 §Decision Drivers: "OrgRegistry coherence — per-org overlay directories must reference org slugs registered in OrgRegistry (ADR-010)"
- ADR-010: Customer config schema — `OrgRegistry` population source
- ADR-022 §B: boot step 3 (OrgRegistry init) precedes step 4 (sensor spec loading)

## Story Anchor

S-CONFIG-MULTI-TENANT-OVERRIDE-001 (to-be-created)

## VP Anchors

(None yet — VP to be authored by test-writer alongside S-CONFIG-MULTI-TENANT-OVERRIDE-001)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC specifies a boot-time configuration validation rule: the cross-check between `customers/` overlay directories and `OrgRegistry` is part of multi-error config validation ("Configuration Validation Reports All Errors in One Pass" — BC-2.06.005 pattern) applied to the per-org config loading lifecycle. |
| L2 Invariants | DI-008 (client data separation — org slug cross-validation prevents unregistered org directories from acting as routing destinations) |
| L2 Entities | OrgRegistry, OrgSlug, BootError |
| Priority | P0 |
| ADR | ADR-029 (Multi-Tenant Sensor Endpoint Overrides) |
| Source-of-Truth Precedence | ADR-029 §Decision Drivers (OrgRegistry coherence) and §Boot-Time Validation step 3c are authoritative. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | D-803 burst-3 | 2026-05-23 | product-owner | Initial draft per ADR-029 Burst 3 handoff. Note: E-SPEC-022 allocated for this error (unknown org slug) instead of ADR-029's proposed E-SPEC-021, because E-SPEC-021 is allocated to `[[tables]]` override and E-SPEC-022 is the next available code. Full error code allocation documented in BC-2.06.016. |
| 1.1 | F-LP4-MED-003 | 2026-05-24 | product-owner | POL-25/POL-29 canonical-template byte-match restoration. F-LP4-MED-003: E-SPEC-022 message at line 69 — replaced paraphrase ("Register the org in prism.toml [[orgs]] or remove the stale directory") with canonical ("Check for typos or register the org in prism.toml [[orgs]]."). POL-29 sweep result: no additional variant forms of E-SPEC-022 message found in .factory/ beyond the fixed site. 4-way alignment confirmed: BC-2.06.015 v1.1 ↔ BC-2.06.016 line 135 ↔ error-taxonomy.md line 394 ↔ overlay.rs:650-653. |
