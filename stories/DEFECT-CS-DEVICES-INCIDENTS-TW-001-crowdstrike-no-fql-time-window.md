---
document_type: story
story_id: "DEFECT-CS-DEVICES-INCIDENTS-TW-001"
title: "CrowdStrike devices and incidents tables have no FQL time-window slot"
wave: tbd
epic_id: engine-defects
priority: P2
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - findings/prism-pushdown-audit.md
severity: MED
behavioral_contracts:
  - BC-2.01.013
# BC status: BC-2.01.013 carries status: active, lifecycle_status: active.
# Per S-7.01 gate: behavioral_contracts is non-empty, so draft status is valid.
# BC authorship of new clauses (if required) must be confirmed by product-owner
# before status=ready.
# BC array propagation: AC↔BC bidirectional traces are deferred to specification time.
verification_properties: []
depends_on: []
blocks: []
origin_finding: "DEFECT-CS-DEVICES-INCIDENTS-TW-001 (G8)"
origin_cascade: "D-1889 live-demo triage 2026-07-20"
cycle: "v1.0.0-greenfield"
phase: 3
# tdd_mode: NOT SET — tdd_mode and Red Gate enumeration (RG-001..RG-NNN) are
# deferred to specification time per SAC-1. Setting tdd_mode: strict on a
# registration stub with no acceptance criteria would create an immediate SAC-1
# violation (no enumerated RG list, no BC-5.38.001 density check).
track: "Platform Engineering"
subsystems: [SS-01, SS-11]
---

# DEFECT-CS-DEVICES-INCIDENTS-TW-001: CrowdStrike devices and incidents tables have no FQL time-window slot

## Problem

The CrowdStrike sensor spec declares `devices` and `incidents` tables but neither table
spec includes an FQL (Falcon Query Language) parameter slot that accepts a time-window
predicate. A query with a `WHERE timestamp BETWEEN ...` or similar time-range filter
against either table cannot push the time constraint down to the CrowdStrike API; the
full upstream dataset is retrieved and filtered client-side.

**Critical sequencing dependency — read before specifying this story:**

The `incidents` table half of this defect may be moot before specification begins.
D-1949 (2026-07-22) recorded the retirement of `S-DTU-CROWDSTRIKE-INCIDENTS-ROUTE-001`
because the CrowdStrike Incidents API was removed by the vendor around 2026-03.
A separate story, `S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001`, exists in the triage
as the designated vehicle for retiring the `incidents` table from
`crowdstrike.sensor.toml` entirely.

However, verification confirms that **no file named `S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001`
exists** anywhere under `.factory/stories/`. The retirement story was designated in the
D-1889 triage but was never authored as a tracked artifact (same class of
unregistered-promise finding as `S-DEMO-CLAROTY-TIME-001`).

The routing chain must determine whether:
(a) `S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001` is authored and executed first, which would
    make the `incidents` half of this stub moot and reduce the scope to `devices` only,
(b) both tables are addressed here together (appropriate if the retirement story is further
    deferred), or
(c) this story is split: one story for `devices` (no dependency), one story for
    `incidents` (blocked on or superseded by the retirement story).

The routed implementer and product-owner must resolve this ordering before specifying
acceptance criteria. The stub does NOT pre-judge the outcome.

## Origin

D-1889 live-demo triage 2026-07-20 — Bucket B Engine Defects, finding G8 (MED).
Source readings: `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
and `findings/prism-pushdown-audit.md`.

Triage entry: "CrowdStrike devices and incidents tables have no FQL time-window slot;
BC anchor: BC-2.01.013."

## Authority

**Behavioral contracts:**

| BC | Title | Frontmatter status | Governing clause |
|----|-------|-------------------|-----------------|
| BC-2.01.013 | Datasource Trait Adapter Pattern | status: active; lifecycle_status: active | Governs `FetchStep` grammar and per-table time-window pushdown parameter slots. The absence of an FQL time-window slot in the CrowdStrike devices (and potentially incidents) table specs is a gap in the sensor-spec content that should be governed by this BC's pushdown obligations. |

No ADR governs CrowdStrike FQL parameter slot authorship. If the product-owner
determines that BC-2.01.013 requires an amendment or new postcondition to specify
FQL time-window slot requirements, that authorship occurs during the routing chain
below.

## Routing

Route: `implementer` + `product-owner`

The product-owner must:
(a) confirm whether the `incidents` table should be retired (via
    `S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001`) before this story is specified, or
    whether this story covers `devices` only (with `incidents` handled separately),
(b) amend `crowdstrike.sensor.toml` to add FQL time-window parameter slots for the
    in-scope tables, and confirm the FQL parameter name(s) from the CrowdStrike API
    documentation.

The implementer proceeds under TDD after product-owner scope confirmation.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, architecture mapping, edge cases, and task breakdown are deferred to specification
time. This stub exists solely to register the defect in the tracking system and document
the incidents-retirement sequencing dependency.

`tdd_mode` and Red Gate enumeration will be set by the spec-authoring agent (per SAC-1)
when acceptance criteria are authored, after the incidents-retirement ordering is
resolved.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage G8; documents incidents-retirement sequencing dependency; notes S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001 is also untracked |
