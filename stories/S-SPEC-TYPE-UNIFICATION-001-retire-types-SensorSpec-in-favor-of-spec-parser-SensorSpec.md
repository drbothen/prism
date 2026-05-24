---
document_type: story
story_id: S-SPEC-TYPE-UNIFICATION-001
title: "Retire types::SensorSpec — Unify on spec_parser::SensorSpec as Canonical"
wave: 4
epic_id: wave-4-operations
priority: P1
status: draft
version: "v0.1"
level: "L4"
producer: architect
timestamp: "2026-05-23T00:00:00Z"
created: "2026-05-23"
modified: "2026-05-23"
tdd_mode: strict
subsystems: [SS-06, SS-16]
# Subsystem anchor justifications:
#   SS-06 (Client Configuration): owns ConfigSnapshot and the ArcSwap hot-reload
#   infrastructure in prism-core. Changing ConfigSnapshot::sensor_specs field type
#   is an SS-06 change.
#   SS-16 (Sensor Adapter Engine): owns spec_parser::SensorSpec (canonical type)
#   and parse_spec_directory. Retiring types::SensorSpec from this module is SS-16.
crates_touched: [prism-core, prism-spec-engine, prism-bin]
target_module: prism-core
behavioral_contracts:
  - BC-2.16.001  # Sensor Spec File Loading — ConfigSnapshot::sensor_specs is the
                 # output of parse_spec_directory; changing its value type touches
                 # the loading contract.
  - BC-2.06.012  # Per-Tenant Overlay Loading — OverlayLoader::load_overlays expects
                 # spec_parser::SensorSpec; after unification build_type_spec_map_for_overlay
                 # is deleted and replaced by reuse of ConfigSnapshot::sensor_specs.
verification_properties: []
# Expected VPs: 1 regression VP — ConfigSnapshot::sensor_specs contains no
# SensorSpec with stringly-typed auth_type field after unification.
depends_on:
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001  # Introduces build_type_spec_map_for_overlay
                                        # (the duplicate parse helper) and documents
                                        # the type mismatch at boot.rs:707-715.
                                        # Must be merged before this story ships.
blocks: []
# No current story blocks on this; unification is a structural cleanup that
# unlocks cleaner boot-path reasoning but does not gate any Wave 4 feature.
estimated_days: 4
points: 6
# Estimate rationale:
#   Day 1: ADR — design dep-cycle resolution approach (SensorSpecCore in prism-core
#           vs. move spec_parser types to prism-core vs. new intermediate crate).
#   Day 2-3: Implement migration across prism-core + prism-spec-engine + prism-bin
#           (ConfigSnapshot field type change, parse_spec_directory return type, all
#           callsites of types::SensorSpec).
#   Day 4: Delete build_type_spec_map_for_overlay; verify single-parse boot;
#           confirm 3-CLEAN adversarial cascade.
traces_to:
  - F-LP2-LOW-001  # Adversary finding from S-CONFIG-MULTI-TENANT-OVERRIDE-001
                   # cascade pass-2: per-boot duplicate parsing exposure;
                   # type proliferation root cause.
  - plugin-system-audit-2026-05-08.md:115  # Original finding: "Unify SensorSpec types
                                           # — pick spec_parser::SensorSpec as canonical,
                                           # retire types::SensorSpec" (item 4 of ordered
                                           # remediation list; predates current cascade).
  - adversary-pass-2-S-PLUGIN-PREREQ-C:115  # First adversarial flagging; same recommendation.
---

# S-SPEC-TYPE-UNIFICATION-001 — Retire `types::SensorSpec`; Unify on `spec_parser::SensorSpec`

## Narrative

As a prism maintainer, I want a single `SensorSpec` type in the spec-engine codebase so
that the boot path parses each `.sensor.toml` file exactly once, the `ConfigSnapshot` carries
the structured `AuthType` enum rather than a stringly-typed `auth_type: String`, and future
adversarial cascades do not re-flag this type proliferation on every story that touches boot.rs.

## Background and Origin

This story exists because two parallel `SensorSpec` types accumulated during the spec-engine
evolution:

- `prism_spec_engine::types::SensorSpec` — legacy type with `auth_type: String`. Lives in
  `types.rs`. Stored in `ConfigSnapshot::sensor_specs` (ArcSwap hot-reload path).
- `prism_spec_engine::spec_parser::SensorSpec` — canonical type with `auth_type: AuthType`
  enum. Produced by `SpecLoader::parse`. Used by `OverlayLoader::load_overlays`.

Because `ConfigSnapshot` lives in `prism-core` (to break the `prism-core` ↔ `prism-spec-engine`
dependency cycle) and `spec_parser::SensorSpec` lives in `prism-spec-engine`, the two types
cannot be unified without an explicit architectural decision about how `prism-core::ConfigSnapshot`
references a parsed-output type without creating a dep cycle. That decision requires an ADR —
which is why S-CONFIG-MULTI-TENANT-OVERRIDE-001 correctly deferred this to a dedicated story
rather than silently expanding scope mid-cascade (Canonical Principle Rule 3; Architect Decision
F-LP2-LOW-001 deferral rationale 2026-05-23).

The concrete runtime symptom: `boot.rs::build_type_spec_map_for_overlay` (lines 717-774)
performs a SECOND parse of every `.sensor.toml` file because `OverlayLoader` needs
`spec_parser::SensorSpec` but `ConfigSnapshot::sensor_specs` only contains `types::SensorSpec`.
With 4+ production sensor TOMLs (armis, claroty, crowdstrike, cyberint), every boot incurs
8+ TOML parses instead of 4. This is a SOUL.md §4 violation: data that exists is re-derived.

## Required ADR

Before implementation begins, the architect must produce an ADR (suggested: ADR-030) that
selects one of the following dep-cycle resolution approaches:

### Approach A — `SensorSpecCore` in `prism-core`
Extract a minimal `SensorSpecCore` type (fields: `sensor_id`, `name`, `version`, `auth_type`
as `AuthTypeCore` enum, `base_url`, `file_hash`, `source_path`, `mode`, `credential_refs`)
into `prism-core`. `ConfigSnapshot::sensor_specs` stores `HashMap<String, SensorSpecCore>`.
`spec_parser::SensorSpec` adds `tables`, `rate_limit_hints`, and the richer field set.
`SpecLoader::parse` returns `spec_parser::SensorSpec`; the boot path converts
`spec_parser::SensorSpec → SensorSpecCore` to populate `ConfigSnapshot`.

- Pro: dep cycle preserved; `prism-core` stays thin.
- Con: two related types still exist, just at different levels (core vs. parser output).
  Conversion code is new surface area.

### Approach B — Move `spec_parser` module output to `prism-core`
Move `AuthType` enum and a unified `SensorSpec` into `prism-core`. `prism-spec-engine`
depends on `prism-core::SensorSpec` as both its parse output and its `ConfigSnapshot` value.

- Pro: single type everywhere; `build_type_spec_map_for_overlay` deleted cleanly.
- Con: `prism-core` grows; dep direction must be audited carefully.

### Approach C — Introduce `prism-sensor-types` crate
New zero-dep crate `prism-sensor-types` holds `SensorSpec`, `AuthType`, `TableSpec`, etc.
Both `prism-core` and `prism-spec-engine` depend on `prism-sensor-types`.

- Pro: clean separation; no dep cycle; single type.
- Con: new crate adds workspace overhead; Cargo.toml changes across all consumers.

The ADR must select one approach and document the dep-cycle analysis. Implementation does
not begin until the ADR is ACCEPTED.

## Acceptance Criteria

### AC-001 — Zero remaining usages of `types::SensorSpec`
After this story merges, `grep -rn "types::SensorSpec" crates/` returns zero results
(excluding the definition file itself, which is deleted or converted to a re-export stub
if needed for semver compatibility).

### AC-002 — `build_type_spec_map_for_overlay` deleted
`boot.rs::build_type_spec_map_for_overlay` function (currently lines 717-774) is removed.
`step4_load_sensor_specs_with_overlays` reuses `ConfigSnapshot::sensor_specs` to populate
the type_specs map passed to `OverlayLoader::load_overlays` — no second directory scan.

### AC-003 — Single-parse boot verified by test
A new integration test (or extension of an existing boot integration test) asserts that
`SpecLoader::parse` is called exactly N times (where N = number of `.sensor.toml` files
in the test fixture directory) across a full boot sequence — not 2N. The test uses a
mock or counter-instrumented `SpecLoader` to count parse invocations.

### AC-004 — `ConfigSnapshot::sensor_specs` carries structured `auth_type`
After unification, accessing `snapshot.sensor_specs[sensor_id].auth_type` returns the
structured enum variant (e.g., `AuthType::ApiKey`), not a raw `String`. Existing callsites
that pattern-match or `.as_str()` on `auth_type` are updated accordingly.

### AC-005 — No regression in 3-CLEAN adversarial cascade
The story's local adversarial cascade achieves 3-CLEAN (strict) convergence per
BC-5.39.001. Specific focus: SAP-1 (tracing emission catalog completeness) applies
to any new tracing sites added during the boot-path refactor.

## Implementation Notes

### Dep cycle analysis (pre-ADR)
Current dep graph: `prism-bin` → `prism-spec-engine` → `prism-core`. `prism-core` has no
dep on `prism-spec-engine`. `ConfigSnapshot` is in `prism-core` specifically to allow
`prism-bin` to pass `ConfigSnapshot` to `prism-spec-engine` functions without a cycle.
Any solution that moves `spec_parser::SensorSpec` into `prism-core` must verify no reverse
dep is introduced.

### File list (expected changes)
| File | Change |
|------|--------|
| `crates/prism-core/src/config.rs` | `sensor_specs` field type change |
| `crates/prism-core/src/lib.rs` | Re-export new canonical `SensorSpec` |
| `crates/prism-spec-engine/src/types.rs` | Delete `SensorSpec` struct (or convert to type alias pointing at canonical) |
| `crates/prism-spec-engine/src/spec_parser.rs` | Confirm `SensorSpec` remains canonical TOML parse output |
| `crates/prism-bin/src/boot.rs` | Delete `build_type_spec_map_for_overlay`; refactor `step4_load_sensor_specs_with_overlays` |
| `crates/prism-spec-engine/tests/hot_reload_tests.rs` | Update `ConfigSnapshot` construction helpers to use canonical type |
| `crates/prism-spec-engine/tests/bc_3_2_005_reload_mode_rejection.rs` | Same |
| `crates/prism-bin/tests/bc_2_03_013_credential_init.rs` | Update auth_type access pattern |
| `.factory/specs/architecture/adr/ADR-030-*.md` | New ADR for dep-cycle resolution approach |

### Non-exhaustive discipline
`types::SensorSpec` carries `#[non_exhaustive]`. If any external tests construct
`types::SensorSpec` directly, they must be updated to use the canonical type's
`..Default::default()` pattern. The compile-fail gate at `tests/external/non-exhaustive-violation/`
EXPECTED count must be updated if the deleted type was counted there.

## Traceability

| Source | Reference |
|--------|-----------|
| Origin finding | F-LP2-LOW-001 (adversary-pass-2 of S-CONFIG-MULTI-TENANT-OVERRIDE-001; 2026-05-23) |
| Pre-existing audit flag | `cycles/wave-4-operations/plugin-system-audit-2026-05-08.md:115` |
| Deferral decision | Architect adjudication 2026-05-23 — Option B selected; rationale: dep-cycle resolution requires dedicated ADR outside S-CONFIG scope |
| ADR to produce | ADR-030 (dep-cycle resolution approach selection) |
| Deferral authorized by | Human dispatch of architect adjudication task (per Canonical Principle Rule 3) |
| Blocking story | S-CONFIG-MULTI-TENANT-OVERRIDE-001 (must merge first; introduces the duplicate-parse helper this story will delete) |
