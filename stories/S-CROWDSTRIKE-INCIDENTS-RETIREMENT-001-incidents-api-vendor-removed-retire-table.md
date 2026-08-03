---
document_type: story
story_id: "S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001"
title: "CrowdStrike Incidents API vendor-removed — retire incidents table and re-adjudicate DTU-EXT-001"
wave: "A"
epic_id: wave-a-sensor-fidelity
priority: P0
status: draft
version: "0.1"
severity: CRIT
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - .factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md
origin_finding: "S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001 (D-1889 triage 2026-07-20, sensor CRIT bucket)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts: []
# BC status: pending PO authorship
# S-7.01 gate: behavioral_contracts: [] — status MUST remain draft until a product-owner
# authors and anchors a BC with canonical BC-S.SS.NNN ID for this story.
# Product decision is required before BC authorship — see ## Gate.
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: CRIT
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001: CrowdStrike Incidents API vendor-removed — retire incidents table and re-adjudicate DTU-EXT-001

## Problem

`crates/prism-sensors/specs/crowdstrike.sensor.toml` still declares a table with
`table_name = "incidents"`. The CrowdStrike Incidents API was removed by the vendor
approximately 2026-03. Live tenants return errors on the incidents endpoint; the incidents
table is dead weight that surfaces as health-probe noise and misleads LLM agents about
available sensor tables.

The Prism DTU for CrowdStrike (`crates/prism-dtu-crowdstrike/`) contains incidents routes
that correspond to the now-removed API. These routes must be removed to keep the DTU
consistent with the production spec per BC-2.16.013.

## D-1949 Scope Clarification — What It Did and Did Not Cover

D-1949 (2026-07-22, state-manager burst, Wave-A spec-evolution burst 4) covered:

**What D-1949 DID:**
- Retired story `S-DTU-CROWDSTRIKE-INCIDENTS-ROUTE-001` (DTU route build story), marking it
  superseded by this story (S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001)
- Retired DTU-EXT-001 and DTU-EXT-005 in BC-2.16.013 (removed external validation blockers
  that referenced the now-removed incidents API)
- Added STORY-INDEX rows for the successor story IDs
- Severed the story-dependency edge from the retired Cyberint seeding story to the demo scenario

**What D-1949 did NOT cover:**
- Removing `table_name = "incidents"` from `crates/prism-sensors/specs/crowdstrike.sensor.toml`
- Removing incidents DTU routes from `crates/prism-dtu-crowdstrike/`
- Authoring acceptance criteria, implementation guidance, or Red Gate tests for the retirement
- Adjudicating whether the incidents capability is permanently retired or migrated to an
  alternative CrowdStrike API surface

The production `crowdstrike.sensor.toml` incidents table entry and the DTU incidents routes
remain present on `develop` as of this registration. The STORY-INDEX row exists; the
implementation stub file does not — this file is that stub.

## Cross-Reference: DEFECT-CS-DEVICES-INCIDENTS-TW-001 Sequencing Dependency

`DEFECT-CS-DEVICES-INCIDENTS-TW-001-crowdstrike-no-fql-time-window.md` covers FQL
time-window push-down for both CrowdStrike devices AND incidents tables. If this retirement
story lands before or concurrently with DEFECT-CS-DEVICES-INCIDENTS-TW-001, the incidents
half of that defect becomes moot — there will be no incidents table to apply FQL time-windows
to. The product-owner must adjudicate whether:

- **(a)** DEFECT-CS-DEVICES-INCIDENTS-TW-001 should be scoped down to devices-only once this
  retirement lands, or
- **(b)** DEFECT-CS-DEVICES-INCIDENTS-TW-001 should be sequenced after this retirement to
  avoid authoring FQL time-window tests for an endpoint that will be removed

This sequencing dependency is unresolved at registration time. The product-owner must resolve
it as part of the gate decision for this story before `depends_on` can be finalized for either
story.

## Origin

**Triage date:** 2026-07-20
**Source:** D-1889 live-demo triage, sensor CRIT bucket
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
**D-1949 context (2026-07-22):** Story retirement and BC-2.16.013 DTU-EXT blocker retirement
only; production spec incidents table and DTU incidents routes remain on develop.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| ADR-053 (Wave-A Sensor Fidelity Remediation) | `status: accepted` | §D1 — OpenAPI grounding: removed vendor API endpoints must not appear in TOML sensor specs |
| BC-2.16.013 (Bundled sensor spec DTU parity) | `status: active` | DTU-EXT-001 retired (D-1949); DTU-EXT-005 retired (D-1949); implementation retirement of the incidents table entry and its DTU routes is required to close the remaining DTU parity gap |
| BC-2.02.003 (CrowdStrike field mapping) | `status: active` | Incidents field mapping rows may require removal or archival as part of retirement; product-owner determines scope during BC authorship |

**FINDING-A (product-owner-routed, MEDIUM):** No governing BC exists specifying the
retirement mechanism (incidents table entry removal, DTU route removal, post-retirement
health-probe behavior). Product decision and BC authorship required before implementation.

## Gate

**Product decision required.** This retirement requires product-owner adjudication of:

1. Whether the CrowdStrike incidents capability is permanently retired (no replacement) or
   migrated to an alternative CrowdStrike API surface (e.g., incidents semantics mapped onto
   the Alerts-v2 surface landing in S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001)
2. Whether DEFECT-CS-DEVICES-INCIDENTS-TW-001 should be scoped down (devices-only) or
   sequenced after this retirement (see §Cross-Reference above) — this sequencing decision
   must be recorded before either story's `depends_on` can be finalized
3. Whether BC-2.02.003 incidents field mapping rows should be deleted or archived
4. BC authorship for the retirement state

After product decision and BC authorship, story-writer populates ACs, RG list, density check,
and `tdd_mode`.

## Routing

1. **Product-owner** — adjudicate retirement vs. migration decision; resolve
   DEFECT-CS-DEVICES-INCIDENTS-TW-001 sequencing; author BCs covering the post-retirement
   state; determine BC-2.02.003 incidents row disposition
2. **Story-writer** — populate ACs, RG-001..RG-NNN list, BC-5.38.001 density check from
   adjudicated BCs; set `tdd_mode`; update `depends_on` per sequencing adjudication
3. **Implementer** — remove `incidents` table entry from `crowdstrike.sensor.toml`; remove
   incidents DTU routes from `crates/prism-dtu-crowdstrike/`; update BC-2.16.013 parity
   annotations as directed by product-owner

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density check,
`tdd_mode` declaration, task decomposition, story-point estimate, and token budget are deferred
to product-owner (retirement vs. migration decision, BC authorship) and story-writer (AC/RG
decomposition). The DEFECT-CS-DEVICES-INCIDENTS-TW-001 sequencing dependency is explicitly
unresolved and must be adjudicated before this story's `depends_on` can be finalized.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (sensor CRIT); records D-1949 scope (story retirement + DTU-EXT-001/005 retirement only; production TOML incidents table and DTU incidents routes remain on develop); captures DEFECT-CS-DEVICES-INCIDENTS-TW-001 sequencing dependency; product decision gate required before BC authorship |
