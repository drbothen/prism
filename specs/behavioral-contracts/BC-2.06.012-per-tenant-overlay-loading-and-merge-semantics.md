---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.012"
version: "1.0"
status: draft
lifecycle_status: draft
producer: product-owner
timestamp: 2026-05-23T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
introduced: "2026-05-23"
modified: "2026-05-23"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-CONFIG-MULTI-TENANT-OVERRIDE-001]
verifying_vps: []
crates: [prism-spec-engine, prism-bin]
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

# BC-2.06.012: Per-Tenant Overlay Loading and Merge Semantics

## Description

At boot step 4 (`step4_load_sensor_specs`), after loading all TYPE specs from the root
`sensor_specs_dir`, Prism walks the `customers/` subdirectory and discovers per-org overlay
files matching `customers/<org_slug>/<sensor_id>.sensor.toml`. Each overlay file is parsed
into a `SensorInstanceOverlay` struct and merged onto the corresponding TYPE spec's scalar
fields to produce a `ResolvedSensorSpec` cached in memory for each `(org_slug, sensor_id)`
pair. Tables, `auth_type`, `version`, and `sensor_id` are never overridden — they are always
inherited from the TYPE spec unchanged. Overlay files are only scanned recursively within the
`customers/` subdirectory; the root spec dir scan remains flat (non-recursive).

## Preconditions

- Boot step 4 (`step4_load_sensor_specs`) has been invoked.
- All TYPE specs have been loaded from `<sensor_specs_dir>/*.sensor.toml` (root-level, flat
  scan, preceding the overlay walk in the same step).
- A `customers/` subdirectory exists within `sensor_specs_dir` (the directory may be empty
  or absent — both are valid; an absent `customers/` dir is treated as empty).
- `OrgRegistry` is populated (boot step 3 completes before step 4; OrgRegistry is
  available for slug cross-validation per ADR-022 §B boot sequence).

## Postconditions

- For each `customers/<org_slug>/<sensor_id>.sensor.toml` file discovered:
  - The file is parsed into a `SensorInstanceOverlay` with fields: `extends: String`,
    `instance_id: String`, `base_url: Option<String>`, and
    `rate_limit_hints: Option<RateLimitHints>`.
  - A `ResolvedSensorSpec` is produced by copying the TYPE spec identified by `extends`
    and overlaying only the scalar fields present in the overlay:
    - `base_url` from the overlay (if present) replaces the TYPE spec default.
    - `rate_limit_hints.requests_per_second` from the overlay (if present) replaces the
      TYPE spec default.
    - `rate_limit_hints.burst_size` from the overlay (if present) replaces the TYPE spec
      default.
  - The following fields are NEVER overridden by an overlay and are always taken from the
    TYPE spec: `[[tables]]` (schema), `auth_type`, `version`, `sensor_id`.
  - The `ResolvedSensorSpec` carries provenance metadata indicating which fields originated
    from the TYPE spec and which from the overlay.
  - The resolved spec is indexed by `(org_slug, sensor_id)` for O(1) fanout lookup.
- A `ResolvedSensorSpec` is stored for each valid `(org_slug, sensor_id)` pair.
- Sensors with no overlay file for a given org (SaaS sensors such as CrowdStrike and
  Cyberint) are served by the TYPE spec as-is; no `ResolvedSensorSpec` entry is required
  for those pairs — the fanout engine falls back to the TYPE spec directly when no
  resolved entry exists (see BC-2.06.014).
- Boot continues normally after the overlay walk completes with zero errors.
- If the `customers/` directory is absent or empty, the overlay walk produces zero
  `ResolvedSensorSpec` entries and boot continues normally.
- Overlay loading is logged at `info` level per file: `event_type = "overlay.loaded"`,
  `org_slug`, `sensor_id`, `instance_id`.

## Invariants

- INV-OVL-001: `[[tables]]` schema is immutable per overlay. No overlay may expand or
  contract the column list of the TYPE spec. All tenants sharing a sensor TYPE observe
  identical schema.
- INV-OVL-002: `auth_type` is immutable per overlay. Authentication mechanism is declared
  once at the TYPE level and cannot be changed per org.
- INV-OVL-003: `ResolvedSensorSpec` provenance metadata is always present. Each resolved
  spec records which scalar fields came from the overlay vs the TYPE spec.
- INV-OVL-004: The root `sensor_specs_dir` scan remains non-recursive (flat). Only the
  `customers/` sub-path is walked recursively.
- INV-OVL-005: Overlay walk occurs AFTER all TYPE specs are loaded. A TYPE spec referenced
  by `extends` in an overlay is always present in memory when the overlay is validated.
- INV-OVL-006: Per-org `ResolvedSensorSpec` entries are read-only after boot. They are
  regenerated only on config reload (not mutated in place).

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-019` | Overlay `extends` references a sensor_id not present in loaded TYPE specs | Boot fails (exit 2); see BC-2.06.016 for full error contract |
| `E-SPEC-020` | Overlay `instance_id` does not match `{sensor_id}@{org_slug}` convention | Boot fails (exit 2); see BC-2.06.016 |
| `E-SPEC-021` | Overlay contains `[[tables]]` blocks (schema override forbidden) | Boot fails (exit 2); see BC-2.06.016 |
| `E-SPEC-022` | Overlay directory `customers/<slug>/` references unknown org slug | Boot fails (exit 2); see BC-2.06.016 |
| `E-SPEC-023` | Overlay file contains unrecognized scalar field | Boot fails (exit 2); see BC-2.06.016 |
| `E-SPEC-001` | Overlay TOML fails to parse (syntax error) | Boot fails (exit 2); structured error with file path and line number |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-012-001 | `customers/` directory absent | Treated as empty; zero overlays loaded; boot continues normally |
| EC-012-002 | `customers/` directory present but empty (only `.gitkeep`) | Zero overlays loaded; boot continues normally |
| EC-012-003 | SaaS sensor (CrowdStrike) has no per-org overlay | No `ResolvedSensorSpec` entry created; fanout falls back to TYPE spec |
| EC-012-004 | Overlay sets `base_url` only; rate_limit_hints absent | Merged spec uses TYPE spec rate_limit_hints unchanged; base_url from overlay |
| EC-012-005 | Overlay sets `rate_limit_hints.requests_per_second` only; base_url absent | Merged spec uses TYPE spec base_url unchanged; rate override applied |
| EC-012-006 | Two orgs have overlays for the same sensor (e.g., `acme/armis.sensor.toml` and `contoso/armis.sensor.toml`) | Two independent `ResolvedSensorSpec` entries produced; no interference |
| EC-012-007 | Config reload (`reload_config`) after initial boot | Overlay walk is re-executed; `ResolvedSensorSpec` map is rebuilt atomically from the new config snapshot |
| EC-012-008 | `customers/.gitkeep` file present alongside subdirectories | `.gitkeep` is not a subdirectory; it is ignored; org subdirectories are walked normally |

## Canonical Test Vectors

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — single overlay | `customers/acme/armis.sensor.toml` with `extends="armis"`, `instance_id="armis@acme"`, `base_url="https://armis.acme-corp.io"` | `ResolvedSensorSpec` for `(acme, armis)` with overlay `base_url`; TYPE spec tables unchanged |
| Absent customers dir | `sensor_specs_dir` contains no `customers/` subdirectory | Zero overlays; boot succeeds |
| Empty customers dir | `customers/.gitkeep` only | Zero overlays; boot succeeds |
| Rate-limit-only override | overlay sets `rate_limit_hints.requests_per_second = 5.0`; no `base_url` | `ResolvedSensorSpec` uses TYPE `base_url`; `requests_per_second` overridden to 5.0 |
| Two-org same-sensor | `customers/acme/armis.sensor.toml` + `customers/contoso/armis.sensor.toml` | Two distinct `ResolvedSensorSpec` entries; schemas identical (from TYPE) |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none yet) | Integration test: `test_BC_2_06_012_overlay_merge_scalars_only` — verifies that a valid overlay file produces a `ResolvedSensorSpec` with overlay `base_url` and TYPE-spec tables intact |

## Related BCs

- BC-2.06.013 — Scalar-Only Overlay Enforcement (boot-time rejection of schema fields)
- BC-2.06.014 — Instance Identity Resolution at Fanout (consumes `ResolvedSensorSpec` produced here)
- BC-2.06.015 — OrgRegistry Cross-Validation at Boot (precondition: OrgRegistry must be populated)
- BC-2.06.016 — Error Taxonomy for Override Violations (error codes emitted here)
- BC-2.16.001 — Sensor Spec File Loading (TYPE spec loading that precedes this BC)
- BC-2.21.001 — OrgRegistry Initialization (boot step 3; must complete before this BC executes)

## Architecture Anchors

- ADR-029 §Decision: Hybrid Sensor Instance with Per-Org Composition Directory
- ADR-029 §Merge Semantics: scalar-only overlay fields (`base_url`, `rate_limit_hints`)
- ADR-022 §B: boot step 4 (`step4_load_sensor_specs`) context
- `prism-spec-engine/src/spec_parser.rs` — `SpecLoader::load_all()` / overlay walk extension

## Story Anchor

S-CONFIG-MULTI-TENANT-OVERRIDE-001 (to-be-created)

## VP Anchors

(None yet — VP to be authored by test-writer alongside S-CONFIG-MULTI-TENANT-OVERRIDE-001)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC specifies how per-tenant overlay TOML files are loaded and merged into resolved sensor specs, which is a client configuration loading behavior: "Load and validate per-client sensor mappings, credential references, and capability overrides from TOML configuration." |
| L2 Invariants | DI-008 (client data separation — overlays are scoped per org_slug, preventing cross-org leakage) |
| L2 Entities | SensorSpec (TYPE), SensorInstanceOverlay, ResolvedSensorSpec |
| Priority | P0 |
| ADR | ADR-029 (Multi-Tenant Sensor Endpoint Overrides) |
| Source-of-Truth Precedence | This BC supersedes any earlier ambiguity about per-tenant spec resolution. ADR-029 §Decision is authoritative for merge semantics. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | D-803 burst-3 | 2026-05-23 | product-owner | Initial draft per ADR-029 Burst 3 handoff |
