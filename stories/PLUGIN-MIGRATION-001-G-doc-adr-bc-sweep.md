---
document_type: story
story_id: PLUGIN-MIGRATION-001-G
title: ".factory: Doc/ADR/BC Body Sweep — Generalize 8 Sensor-Named BCs + Architecture Doc Amendments"
wave: 2
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: draft
version: "v1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-27T00:00:00Z"
modified: "2026-05-27"
tdd_mode: strict
subsystems: [SS-01, SS-02, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) — BC-2.01.005/006/007/008 are the four sensor-auth BCs,
#   each describing auth behavior now delivered by TOML spec + open SensorAuth trait.
#   The body amendment changes "CrowdStrike adapter" → "TOML spec + SpecDrivenMapper"
#   style language throughout.
#   SS-02 (OCSF Normalization) — BC-2.02.003/004/005/006 are the four field-mapping BCs.
#   Post-001-C, SpecDrivenMapper replaces the four hardcoded mapper modules; BC bodies
#   must be updated to describe the spec-driven mapping path.
#   SS-16 (Spec Engine) — module-decomposition.md and sensor-adapters.md both contain
#   sensor-named references to the deleted Rust adapter modules; these architecture docs
#   are in SS-16 territory (spec engine governs the sensor spec loading contract).
crates_touched: []
# This is a .factory-only story. No Rust crates are modified.
target_module: ".factory/specs"
capabilities: [CAP-029]
behavioral_contracts:
  - BC-2.01.005  # CrowdStrike OAuth2 Auth — full body amendment: replace sensor-specific
                 # Rust adapter language with TOML spec + SpecDrivenMapper + .prx WASM plugin
                 # plugin (where applicable) language per ADR-023 Rule 1 + Rule 2
  - BC-2.01.006  # Cyberint Cookie Auth — same amendment pattern
  - BC-2.01.007  # Claroty Bearer Token Auth — same amendment pattern
  - BC-2.01.008  # Armis Bearer Token Auth — same amendment pattern
  - BC-2.02.003  # CrowdStrike Field Mapping — full body amendment: replace hardcoded mapper
                 # module language with SpecDrivenMapper + ocsf_field TOML annotation language
  - BC-2.02.004  # Cyberint Field Mapping — same amendment pattern
  - BC-2.02.005  # Claroty Field Mapping — same amendment pattern
  - BC-2.02.006  # Armis Field Mapping — same amendment pattern
# BC status: All 8 BCs have amendment_lifecycle: pending — ADR-023 per BC-INDEX v5.53.
# The PENDING AMENDMENT banner in each file explicitly names PLUGIN-MIGRATION-001-G as
# the target story for full body amendment. This is the single story that satisfies it.
# No BC-TBD placeholders.
verification_properties: []
# This story produces only .factory/ artifacts (BC files, architecture docs, BC-INDEX).
# No executable verification properties are authored. The "verification" for this story
# is the consistency-validator gate at the wave-2 integration gate (adversary confirms
# all 8 BC bodies no longer contain sensor-named Rust adapter language and the PENDING
# AMENDMENT banners are removed).
depends_on:
  - PLUGIN-MIGRATION-001-A  # auth module deletion (001-A) must be complete before the BC
                            # auth descriptions can be amended to past-tense ("was delivered
                            # by CrowdStrikeAuth; now delivered by TOML spec")
  - PLUGIN-MIGRATION-001-B  # query dispatch conversion (001-B) ensures no BC body refers to
                            # a dispatch site that still has sensor-named code
  - PLUGIN-MIGRATION-001-C  # SpecDrivenMapper (001-C) replaces the 4 mapper modules; the
                            # BC-2.02.003-006 body amendments describe the SpecDrivenMapper
                            # path — must not amend until 001-C is merged and correct
blocks: []
# 001-H (story supersession) has no dependency on 001-G. Both are Wave 2 cleanup stories
# that can be dispatched in parallel after their respective depends_on are satisfied.
points: 8
# Points justification:
#   - 4 auth BCs (BC-2.01.005–008): ~1 pt each = 4 pts
#     (each BC body: remove PENDING AMENDMENT banner, replace sensor-Rust-adapter language
#     with TOML-spec language, update Invariants/Error Cases/Edge Cases sections)
#   - 4 field-mapping BCs (BC-2.02.003–006): ~1 pt each = 4 pts
#     (same pattern: remove PENDING AMENDMENT banner, describe SpecDrivenMapper + ocsf_field
#     annotation path, update Error Cases for new error codes)
#   - BC-INDEX.md row updates (8 rows): ~0.5 pt
#     (update amendment_lifecycle: pending → null; update status to active; bump version pins)
#   - module-decomposition.md + sensor-adapters.md grep sweep: ~0.5 pt
#     (remove/update sensor-named architecture prose per ADR-023 wave-2 scope)
#   - BC-INDEX version bump propagation: included above
#   Total: 9 pts at upper count; absorb into 8 pts given doc-only work (no compile/test cycle).
#   ADR-023 Wave 2 estimate: 5–8 SP.
estimated_days: 3
risk: LOW
# Risk justification: This is a .factory/-only story. No Rust code changes. The primary
# risk is over-amending a BC (removing behavior that is still active). Each BC body amendment
# must preserve all behavioral semantics (preconditions, postconditions, invariants) —
# only the implementation-mechanism language changes ("CrowdStrikeAdapter" → "TOML spec
# with [auth] type = 'oauth2_client_credentials'"). The adversary catch for this is
# consistency-validator comparing BC bodies against the corresponding TOML spec files.
acceptance_criteria_count: 6
red_gate_tests: 0
# This story has no Red Gate tests (no executable code). The gate is the adversary
# consistency-validator pass confirming all 8 BC bodies are updated correctly.
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Behavioral semantics preservation: each BC body amendment preserves all preconditions,
    postconditions, invariants, and error cases. Only the mechanism language changes.
    Adversary SAP-1 probe applies: grep for 'event_type =' in amended BC files to ensure
    no structured event catalog rows are accidentally removed."
  - "PENDING AMENDMENT banner removal: the PENDING AMENDMENT banner must be FULLY removed
    from each BC file (not just updated). A partial banner removal (e.g., leaving a
    stray '> **PENDING' line) is a format violation caught by the consistency-validator."
  - "BC-INDEX version pin propagation (POL-29): after amending each BC, update the BC-INDEX
    row's status field and version pin. Run the POL-29 step-8 cite-pin grep sweep across
    all STORY-INDEX rows that reference the amended BCs to ensure no stale version pins."
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.006-cyberint-cookie-auth.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.007-claroty-bearer-polymorphic-ids.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.008-armis-bearer-aql.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.003-crowdstrike-field-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.004-cyberint-field-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.005-claroty-field-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.02.006-armis-field-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-INDEX.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/architecture/module-decomposition.md"
  - ".factory/specs/architecture/sensor-adapters.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-C-prism-ocsf-spec-driven-mapper.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-D-author-4-production-toml-sensor-specs.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
---

# PLUGIN-MIGRATION-001-G: .factory — Doc/ADR/BC Body Sweep — Generalize 8 Sensor-Named BCs + Architecture Doc Amendments

**Story ID:** PLUGIN-MIGRATION-001-G
**Status:** draft
**Version:** v1.0
**Wave:** 2 (ordered after PLUGIN-MIGRATION-001-A + 001-B + 001-C all merged)

---

## §Origin

Registered in STORY-INDEX at D-334 (2026-05-10) as Wave 2 of the PLUGIN-MIGRATION saga.

ADR-023 §Migration Plan Wave 2 scope: "Doc sweep — module-decomposition, production-runtime-wiring
decision record (inline note directing readers to ADR-023 and PLUGIN-PREREQ-F), BC catalog
sensor-name grep, full body amendment of the 8 sensor-named BCs (BC-2.01.005 through BC-2.01.008
and BC-2.02.003 through BC-2.02.006), sensor-adapters.md (5–8 SP)."

**Scope clarification (from ADR-023 v1.19):** The ADR-022 v1.2 amendment (adding
`superseded_by_partial: ADR-023` annotation at both sites — ADR-022 line 65 and §G Story 3
line 613) landed in Wave 1/A (PLUGIN-MIGRATION-001-A), NOT Wave 2/G. Wave 2/G scope is the
BC body sweep only. This story does NOT need to amend ADR-022 again.

---

## Story-Level Goal

At merge:

1. All 8 sensor-named BCs have their PENDING AMENDMENT banners removed and their bodies
   updated to describe the plugin-only architecture: TOML spec + `SensorAuth` open trait
   (for auth BCs) and TOML spec `ocsf_field` annotations + `SpecDrivenMapper` (for
   field-mapping BCs).

2. `BC-INDEX.md` rows for all 8 BCs updated: `amendment_lifecycle: pending` removed,
   status updated to `active`, version pins updated.

3. `module-decomposition.md` and `sensor-adapters.md` have any surviving sensor-named
   Rust adapter references (e.g., "CrowdStrikeAdapter is wired at boot") replaced with
   spec-driven language aligned to the as-built state post-Wave-1.

4. BC-INDEX version bumped; all story files that cite the amended BCs get their version
   pins updated (POL-29 step-8 cite-pin sweep).

---

## Narrative

As the Prism platform specification, I want the 8 sensor-named behavioral contracts
amended to describe the plugin-only architecture (TOML specs + `SensorAuth` open trait +
`SpecDrivenMapper`) rather than the deleted hardcoded Rust adapters, so that the
authoritative behavioral specifications accurately reflect the as-built system after
PLUGIN-MIGRATION Wave 1.

---

## §Amendment Pattern for Auth BCs (BC-2.01.005–008)

Each of the 4 auth BCs follows this amendment pattern:

**Before (example BC-2.01.005 excerpt):**
```
## Description
The CrowdStrike adapter authenticates using OAuth2 client credentials grant, then follows
a mandatory two-step fetch pattern...
```

**After:**
```
## Description
> **Amendment — ADR-023 (PLUGIN-MIGRATION-001-G):** This BC previously described a
> hardcoded Rust adapter (`CrowdStrikeAuth`). That implementation was deleted in
> PLUGIN-MIGRATION-001-A. The auth behavior described here is now delivered by the
> CrowdStrike TOML sensor spec (`.prism/specs/sensors/crowdstrike.sensor.toml`)
> with `[auth] type = "oauth2_client_credentials"` and the CrowdStrike `.prx` WASM
> plugin for OAuth2 refresh-on-401 (PLUGIN-MIGRATION-001-E). The behavioral
> contract itself is unchanged — preconditions, postconditions, and invariants
> describe what the system must do, not how. The `SensorAuth` open trait
> (BC-2.01.016) is the runtime interface.

The CrowdStrike sensor authenticates using OAuth2 client credentials grant...
[prose updated to describe the TOML + plugin path]
```

The behavioral semantics (preconditions, postconditions, error cases, invariants) are
preserved exactly — only the mechanism description changes from "CrowdStrike adapter
[Rust code]" to "TOML spec + SensorAuth open trait + .prx WASM plugin [declarative]".

The `DI-012` invariant clause ("Sealed auth trait — CrowdStrike OAuth2 flow cannot be
accidentally composed with other sensor auth mechanisms") must be updated: the sealed
trait was retired in S-PLUGIN-PREREQ-E. The replacement invariant is the runtime
cross-composition prevention in `SpecLoader::validate_cross_composition()` per
BC-2.01.016 §Invariants. The DI-012 reference in BC-2.01.005 Invariants section is
updated to reference the runtime enforcement path.

---

## §Amendment Pattern for Field-Mapping BCs (BC-2.02.003–006)

Each of the 4 field-mapping BCs follows this amendment pattern:

**Before (example BC-2.02.003 excerpt):**
```
## Description
The CrowdStrike normalizer maps CrowdStrike alert fields to their canonical OCSF
Detection Finding (class 2004) equivalents...
```

**After:**
```
## Description
> **Amendment — ADR-023 (PLUGIN-MIGRATION-001-G):** This BC previously described a
> hardcoded Rust mapper module (`prism-ocsf/src/mappers/crowdstrike.rs`). That
> implementation was deleted in PLUGIN-MIGRATION-001-C. The field-mapping behavior
> described here is now delivered by `SpecDrivenMapper` reading `ocsf_field` column
> annotations from the CrowdStrike TOML sensor spec. The behavioral contract itself
> is unchanged — the same OCSF field mappings must be produced; they are now
> data-driven via TOML annotations per ADR-023 Rule 1.

The CrowdStrike sensor maps alert fields to canonical OCSF Detection Finding
(class 2004) equivalents via `SpecDrivenMapper` reading `ocsf_field` column annotations
from `.prism/specs/sensors/crowdstrike.sensor.toml`...
[prose updated; field mapping table preserved]
```

The field mapping tables (which source fields map to which OCSF fields) are PRESERVED
exactly — these describe WHAT the system must do. Only the HOW (hardcoded Rust mapper
→ TOML annotation + SpecDrivenMapper) is updated.

---

## Behavioral Contracts

| BC ID | Version | Title | Subsystem | Role in This Story |
|-------|---------|-------|-----------|-------------------|
| BC-2.01.005 | 1.4 | CrowdStrike OAuth2 Authentication and Two-Step Fetch | SS-01 | **Primary** — body amendment: PENDING AMENDMENT banner removed; OAuth2 auth mechanism updated from deleted Rust adapter to TOML spec `[auth] type = "oauth2_client_credentials"` + .prx WASM plugin |
| BC-2.01.006 | 1.4 | Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing | SS-01 | **Primary** — same amendment pattern for Cyberint cookie auth |
| BC-2.01.007 | 1.4 | Claroty Bearer Token Auth with Polymorphic ID Handling | SS-01 | **Primary** — same amendment pattern for Claroty bearer auth |
| BC-2.01.008 | 1.4 | Armis Bearer Token Auth with AQL Query Forwarding and Timestamp Fallback | SS-01 | **Primary** — same amendment pattern for Armis bearer auth |
| BC-2.02.003 | 1.5 | CrowdStrike Alert Field Mapping to OCSF | SS-02 | **Primary** — body amendment: PENDING AMENDMENT banner removed; mapper mechanism updated from deleted hardcoded mapper to SpecDrivenMapper + ocsf_field annotations |
| BC-2.02.004 | 1.4 | Cyberint Alert Field Mapping to OCSF | SS-02 | **Primary** — same amendment pattern for Cyberint field mapping |
| BC-2.02.005 | 1.4 | Claroty xDome Field Mapping to OCSF (9 Data Sources) | SS-02 | **Primary** — same amendment pattern for Claroty field mapping |
| BC-2.02.006 | 1.4 | Armis Centrix Field Mapping to OCSF (7 Data Sources) | SS-02 | **Primary** — same amendment pattern for Armis field mapping |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~7,000 |
| 8 BC files to amend (~300–500 lines each) | ~25,000 |
| BC-INDEX.md (relevant rows + version tracking) | ~3,000 |
| ADR-023 §Amendment Pattern, §Wave 2 scope | ~4,000 |
| module-decomposition.md (architecture doc) | ~5,000 |
| sensor-adapters.md (architecture doc) | ~4,000 |
| PLUGIN-MIGRATION-001-C spec (SpecDrivenMapper context) | ~4,000 |
| PLUGIN-MIGRATION-001-D spec (TOML spec context) | ~3,000 |
| STORY-INDEX rows for amended BCs (POL-29 sweep) | ~2,000 |
| **Total estimate** | **~57,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~28.5%** |

Near the 30% ceiling for a doc-only story due to the 8 BC files. The implementer should
read each BC sequentially (not all 8 simultaneously) and do POL-29 cite-pin sweeps
incrementally after each BC amendment.

---

## Acceptance Criteria

### AC-001: 4 auth BCs amended — PENDING AMENDMENT banners removed, bodies updated to TOML spec + open-trait language (traces to BC-2.01.005/006/007/008 postconditions — auth behavior is delivered by TOML spec configuration + SensorAuth open trait, not by deleted Rust adapter modules)

Each of the 4 auth BCs (BC-2.01.005, BC-2.01.006, BC-2.01.007, BC-2.01.008):

- PENDING AMENDMENT banner (the `> **PENDING AMENDMENT — ADR-023**: ...` block quote) is removed
- The BC gains an Amendment Note at the top of the Description section per §Amendment Pattern
- All prose references to `{Sensor}Adapter`, `{Sensor}Auth Rust type`, "the adapter authenticates..."
  are updated to "the TOML sensor spec with `[auth] type = ...`" language
- The `DI-012` sealed-trait invariant reference is updated to reference `SpecLoader::validate_cross_composition()` runtime enforcement (BC-2.01.016) — DI-012 itself was retired in S-PLUGIN-PREREQ-E
- Preconditions, postconditions, error cases, and field behavior tables are PRESERVED exactly
- `version:` frontmatter field is bumped +0.1 (e.g., `"1.4"` → `"1.5"`)
- `modified:` frontmatter field is updated to `"2026-05-27"` (or current date)
- `amendment_lifecycle: pending` frontmatter field is removed (or set to `null`)

(traces to BC-2.01.005 postcondition — OAuth2 token obtained via client credentials grant before any API call; mechanism is now TOML spec + .prx WASM plugin; contract unchanged)
(traces to BC-2.01.006 postcondition — Cyberint cookie auth mechanism now TOML spec `[auth] type = "cookie_roundtrip"`; contract unchanged)
(traces to BC-2.01.007 postcondition — Claroty bearer token auth mechanism now TOML spec `[auth] type = "bearer_static"`; contract unchanged)
(traces to BC-2.01.008 postcondition — Armis bearer token auth + AQL forwarding mechanism now TOML spec `[auth] type = "bearer_static"` + query forwarding config; contract unchanged)

### AC-002: 4 field-mapping BCs amended — PENDING AMENDMENT banners removed, bodies updated to SpecDrivenMapper + ocsf_field annotation language (traces to BC-2.02.003/004/005/006 postconditions — field mappings delivered by SpecDrivenMapper reading ocsf_field annotations from TOML sensor specs, not by deleted hardcoded mapper modules)

Each of the 4 field-mapping BCs (BC-2.02.003, BC-2.02.004, BC-2.02.005, BC-2.02.006):

- PENDING AMENDMENT banner removed
- Amendment Note added to Description section per §Amendment Pattern
- All references to `{Sensor}Mapper Rust struct` and `prism-ocsf/src/mappers/{sensor}.rs`
  updated to "SpecDrivenMapper reading `ocsf_field` column annotations from the TOML sensor spec"
- Field mapping tables (source field → OCSF field) PRESERVED exactly
- `raw_extensions` preservation clause preserved (BC-2.02.007 anti-regression guard)
- Error case language updated: "mapper returns PrismError" → "SpecDrivenMapper returns PrismError"
  (error code unchanged)
- `version:` bumped +0.1; `modified:` updated; `amendment_lifecycle: pending` removed

(traces to BC-2.02.003 postcondition — CrowdStrike alert fields are mapped to OCSF Detection Finding class 2004; mechanism is now SpecDrivenMapper + ocsf_field TOML annotations; field mapping table preserved)
(traces to BC-2.02.004 postcondition — Cyberint alert field mapping; same pattern)
(traces to BC-2.02.005 postcondition — Claroty xDome field mapping for 9 data sources; field table preserved)
(traces to BC-2.02.006 postcondition — Armis Centrix field mapping for 7 data sources; field table preserved)

### AC-003: BC-INDEX.md updated — 8 rows reflect amended status (traces to BC-2.01.013 invariant — BC catalog is the canonical source of truth for behavioral contracts; BC-INDEX must reflect current amendment state)

For each of the 8 amended BCs, the BC-INDEX.md row is updated:

| BC ID | Old Status Column | New Status Column |
|-------|------------------|------------------|
| BC-2.01.005 | `draft (amendment_lifecycle: pending — ADR-023)` | `active (amended per ADR-023/PLUGIN-MIGRATION-001-G; v1.5)` |
| BC-2.01.006 | same pattern | `active (amended per ADR-023/PLUGIN-MIGRATION-001-G; v1.5)` |
| BC-2.01.007 | same pattern | `active (amended per ADR-023/PLUGIN-MIGRATION-001-G; v1.5)` |
| BC-2.01.008 | same pattern | `active (amended per ADR-023/PLUGIN-MIGRATION-001-G; v1.5)` |
| BC-2.02.003 | same pattern | `active (amended per ADR-023/PLUGIN-MIGRATION-001-G; v1.6)` |
| BC-2.02.004 | same pattern | `active (amended per ADR-023/PLUGIN-MIGRATION-001-G; v1.5)` |
| BC-2.02.005 | same pattern | `active (amended per ADR-023/PLUGIN-MIGRATION-001-G; v1.5)` |
| BC-2.02.006 | same pattern | `active (amended per ADR-023/PLUGIN-MIGRATION-001-G; v1.5)` |

BC-INDEX.md version is bumped (e.g., `v5.53` → `v5.54`). The `total_contracts`,
`active_contracts`, and `draft_contracts` counts are updated: 8 BCs transition from
`draft (amendment pending)` to `active`, changing active count accordingly.

(traces to BC-2.01.013 invariant — the spec catalog and BC catalog must remain consistent; stale amendment_lifecycle: pending entries represent an inconsistency between spec and implementation)

### AC-004: `module-decomposition.md` sensor-name grep sweep — no surviving sensor-named Rust adapter references (traces to BC-2.16.012 invariant — module boundaries reflect the plugin-only architecture; no hardcoded sensor adapter references in module decomposition)

`rg 'CrowdStrike[Aa]dapter\|ClarotyAdapter\|CyberintAdapter\|ArmisAdapter\|CrowdStrikeAuth\|ClarotyAuth\|CyberintAuth\|ArmisAuth\|SensorType::' .factory/specs/architecture/module-decomposition.md`

All hits (if any) are updated to describe the equivalent spec-driven path. If the module
decomposition doc describes `prism-sensors` as containing "the four CrowdStrike/Claroty/
Cyberint/Armis adapters," that sentence is updated to "prism-sensors provides the open
`SensorAuth` trait (BC-2.01.016) and `AdapterRegistry` keyed by `SensorId(Arc<str>)`;
sensor-specific behavior is delivered by TOML specs in `.prism/specs/sensors/`."

(traces to BC-2.16.012 postcondition — PluginRegistry dispatch is the sole mechanism; no sensor-named Rust code exists in the adapter layer post-Wave-1)

### AC-005: `sensor-adapters.md` sensor-name grep sweep + forward reference to ADR-023 (traces to BC-2.01.013 postcondition — sensor-adapters.md describes the current as-built state: TOML spec + open SensorAuth trait; no hardcoded adapter language)

`rg 'CrowdStrike[Aa]dapter\|ClarotyAdapter\|CyberintAdapter\|ArmisAdapter\|CrowdStrikeAuth\|ClarotyAuth\|CyberintAuth\|ArmisAuth\|SensorType::' .factory/specs/architecture/sensor-adapters.md`

All hits updated to the spec-driven equivalent. The document already contains the ADR-023
amendment note at the top (per v1.1 amendment from 2026-05-15); this story verifies that
no sensor-named Rust adapter references survived in the body prose below that note.

If `sensor-adapters.md` still contains a section like "Two-Tier Model: Tier 1 = TOML
spec; Tier 2 = .prx WASM plugin (replaces CustomAdapter)" — that is CORRECT per ADR-023
Rule 5; do not remove tier language.

(traces to BC-2.01.013 postcondition — the adapter architecture doc matches the as-built plugin-only state)

### AC-006: POL-29 cite-pin sweep — STORY-INDEX and other spec files referencing amended BCs updated (traces to BC-2.01.013 invariant — cross-document consistency; all cite-pins must match the canonical BC version)

After all 8 BC amendments are committed, run the POL-29 step-8 cite-pin grep across:

1. `STORY-INDEX.md` — any row in the BC Traceability Matrix that references `BC-2.01.005/006/007/008` or `BC-2.02.003/004/005/006` must have the correct version pin (matching the newly bumped BC file version)
2. `STORY-INDEX.md` Full Story List — any cell citing a BC version string is updated
3. Any STORY-NNN.md file that contains an inline `BC-2.01.005 v1.4` cite (or similar) in its BC table must be updated to `v1.5` (or the new version)

Use: `rg 'BC-2\.01\.00[5-8]\|BC-2\.02\.00[3-6]' .factory/stories/ .factory/specs/ --type md`

For each hit: verify the version pin matches the post-amendment BC frontmatter version;
update if stale.

(traces to BC-2.01.013 invariant — cite-pins are a consistency invariant per POL-29; stale version pins are a finding class that has generated multiple adversary cascades)

---

## Tasks

- [ ] **Task 1:** Read all 8 BC files (listed in §Inputs) to understand their current body state,
      version, and the PENDING AMENDMENT banner text before writing any changes
- [ ] **Task 2:** Amend BC-2.01.005 (CrowdStrike auth): remove PENDING AMENDMENT banner; add
      Amendment Note; update mechanism prose; update DI-012 invariant reference; bump version;
      remove `amendment_lifecycle: pending`
- [ ] **Task 3:** Amend BC-2.01.006 (Cyberint auth): same pattern as Task 2
- [ ] **Task 4:** Amend BC-2.01.007 (Claroty auth): same pattern as Task 2
- [ ] **Task 5:** Amend BC-2.01.008 (Armis auth): same pattern as Task 2
- [ ] **Task 6:** Amend BC-2.02.003 (CrowdStrike field mapping): remove PENDING AMENDMENT
      banner; add Amendment Note; update mapper mechanism prose; preserve field mapping table;
      bump version; remove `amendment_lifecycle: pending`
- [ ] **Task 7:** Amend BC-2.02.004 (Cyberint field mapping): same pattern as Task 6
- [ ] **Task 8:** Amend BC-2.02.005 (Claroty field mapping): same pattern as Task 6
- [ ] **Task 9:** Amend BC-2.02.006 (Armis field mapping): same pattern as Task 6
- [ ] **Task 10:** Update BC-INDEX.md — 8 rows: remove `amendment_lifecycle: pending`, set
       to `active`, update version pins; bump BC-INDEX version
- [ ] **Task 11:** Run sensor-name grep on module-decomposition.md and sensor-adapters.md;
       update any surviving sensor-named Rust adapter language (AC-004, AC-005)
- [ ] **Task 12:** POL-29 cite-pin sweep (AC-006): `rg 'BC-2\.01\.00[5-8]\|BC-2\.02\.00[3-6]'`
       across .factory/stories/ and .factory/specs/; update stale version pins

---

## Architecture Compliance Rules

1. **Behavioral semantics preservation (ADR-023 §BC Amendment Policy).** BC body amendments
   change the mechanism description only. Preconditions, postconditions, invariants, and
   error case tables are preserved verbatim. If an invariant references a deleted mechanism
   (e.g., `DI-012` sealed trait), replace with the active equivalent (`SpecLoader::
   validate_cross_composition()` per BC-2.01.016) — do NOT silently delete the invariant.

2. **PENDING AMENDMENT banner removal is COMPLETE removal.** The `> **PENDING AMENDMENT**`
   block-quote must be fully removed, not re-labeled as "AMENDMENT COMPLETE." The Amendment
   Note added to the Description section is the permanent record.

3. **ADR-023 Wave 2/G scope boundary.** ADR-022 v1.2 was already amended in Wave 1/A
   (PLUGIN-MIGRATION-001-A). This story does NOT re-amend ADR-022. The scope is BC files +
   architecture prose docs ONLY.

4. **POL-29 fixed-point iteration.** After bumping a BC version, grep for that BC's ID
   across ALL .factory/ markdown files and update any inline version pins. Do not stop
   at the first sweep — do a second sweep to catch transitive references in STORY-INDEX
   BC Traceability Matrix rows.

---

## Library & Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| N/A — doc-only story | — | No Rust code changes |

---

## File Structure Requirements

| Action | File Path | Notes |
|--------|-----------|-------|
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md` | Full body amendment: remove PENDING AMENDMENT, update mechanism, bump version |
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.01.006-cyberint-cookie-auth.md` | Same pattern |
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.01.007-claroty-bearer-polymorphic-ids.md` | Same pattern |
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.01.008-armis-bearer-aql.md` | Same pattern |
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.02.003-crowdstrike-field-mapping.md` | Same pattern |
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.02.004-cyberint-field-mapping.md` | Same pattern |
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.02.005-claroty-field-mapping.md` | Same pattern |
| MODIFY | `.factory/specs/behavioral-contracts/BC-2.02.006-armis-field-mapping.md` | Same pattern |
| MODIFY | `.factory/specs/behavioral-contracts/BC-INDEX.md` | 8 row status updates + version bump |
| MODIFY | `.factory/specs/architecture/module-decomposition.md` | Sensor-name grep sweep + fixes |
| MODIFY | `.factory/specs/architecture/sensor-adapters.md` | Sensor-name grep sweep + fixes |
| MODIFY | `.factory/stories/STORY-INDEX.md` | POL-29 cite-pin sweep for 8 amended BCs |

---

## Previous Story Intelligence

Previous stories in the PLUGIN-MIGRATION-001 saga that define the as-built state:

1. **PLUGIN-PREREQ-F (Wave 0):** First added the PENDING AMENDMENT banners to all 8 BCs
   with the placeholder "Full BC amendment in PLUGIN-MIGRATION-001-G." This story fulfills
   that placeholder.
2. **PLUGIN-MIGRATION-001-A (merged PR #156):** Deleted the auth modules. BC-2.01.005–008
   bodies should now describe the post-deletion state (TOML + .prx plugin).
3. **PLUGIN-MIGRATION-001-C (merged PR #158):** Deleted the hardcoded mapper modules;
   implemented SpecDrivenMapper. BC-2.02.003–006 bodies should now describe SpecDrivenMapper
   + ocsf_field annotations.
4. **PLUGIN-MIGRATION-001-D (merged PR #153):** Authored the 4 production TOML sensor specs.
   The TOML spec field names (e.g., `[auth] type = "oauth2_client_credentials"`) are the
   canonical mechanism references to use in the BC body amendments.

Key lesson from prior PLUGIN-MIGRATION stories: the POL-29 cite-pin sweep is essential.
BC version bumps propagate to STORY-INDEX rows, story file BC tables, and cross-references
in other architecture docs. Do a minimum of 2 grep sweeps (one after all 8 BCs are amended,
one after BC-INDEX is updated) to catch all stale pins.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A BC body contains a sensor-named DI reference (e.g., DI-012) that is retired | Update DI reference to the active equivalent (runtime enforcement via BC-2.01.016); do NOT delete the invariant clause entirely |
| EC-002 | The field mapping table in a BC-2.02.* file is referenced by a story AC | Preserve the table exactly; verify that the story AC trace still holds (the behavioral outcome is unchanged) |
| EC-003 | A `raw_extensions` preservation clause in BC-2.02.* is affected by the amendment | Preserve the raw_extensions clause — BC-2.02.007 anti-regression is a standing invariant |
| EC-004 | BC-INDEX `active_contracts` count is wrong after the 8 status transitions | Recount: 8 BCs change from `draft (amendment pending)` to `active`; update active_contracts accordingly |
| EC-005 | A BC amendment removes a behavioral assertion that is still valid | This is a P1 defect per Canonical Principle Rule 4 — fix before committing. Behavioral assertions are NEVER removed by mechanism amendments. |

---

## Forbidden Dependencies

This story produces only `.factory/` artifacts. No crate dependency changes. No
`Cargo.toml` modifications.

If the implementer is tempted to add any `crates_touched` entry, that is a scope
boundary violation — this story is a doc-only story. Code changes in this story
are a routing failure: code changes belong in the implementation stories (001-A
through 001-C), not in the doc sweep.

---

## Changelog

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| v1.0 | 2026-05-27 | story-writer | Initial draft — 6 ACs + 12 tasks; PLUGIN-MIGRATION-001-G Wave 2 materialization |
