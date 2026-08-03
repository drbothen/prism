---
document_type: story
story_id: "DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001"
title: "Time-window pushdown structurally impossible for Claroty xDome and Cyberint sensors"
wave: tbd
epic_id: engine-defects
priority: P1
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - findings/prism-pushdown-audit.md
severity: HIGH
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
origin_finding: "DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001 (G2)"
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

# DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001: Time-window pushdown structurally impossible for Claroty xDome and Cyberint sensors

## Problem

Time-window predicate pushdown cannot be implemented for the Claroty xDome and Cyberint
sensor adapters because neither API exposes a query parameter that accepts a time-window
filter at the `FetchStep` grammar level. The APIs' filtering surfaces do not include a
field or parameter slot that the spec-driven adapter can populate with a `WHERE timestamp
BETWEEN ...` or equivalent pushdown. Any time-window filter applied to queries over these
sensors is therefore evaluated post-fetch (client-side), after the full dataset has been
retrieved — defeating the purpose of pushdown and creating potential for large upstream
API payloads.

The defect is structural: it is not a missing implementation, but a mismatch between the
capabilities declared (or implied) in the sensor spec grammar and the actual API surface.
Resolution requires an architectural decision about how to represent this structural
limitation in the spec and whether to surface it as a query error, a warning, or a
silently-accepted (but non-pushed-down) constraint.

**Coverage note from triage:** The triage records coverage as PARTIAL because the Claroty
half was previously tracked as an unauthored stub named `S-DEMO-CLAROTY-TIME-001`.
Verification confirms that **no file named `S-DEMO-CLAROTY-TIME-001` exists** anywhere
under `.factory/stories/`. The stub exists only as a name in the triage table — it was
never authored. This is an unregistered-promise finding: a story was designated to hold
work that never materialized as a tracked artifact. The current DEFECT stub subsumes that
outstanding intent.

## Origin

D-1889 live-demo triage 2026-07-20 — Bucket B Engine Defects, finding G2 (HIGH).
Source readings: `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
and `findings/prism-pushdown-audit.md`.

Triage entry: "time-window pushdown is structurally impossible for claroty and cyberint;
coverage PARTIAL — the claroty half is an unauthored stub, S-DEMO-CLAROTY-TIME-001."

## Authority

**Behavioral contracts:**

| BC | Title | Frontmatter status | Governing clause |
|----|-------|-------------------|-----------------|
| BC-2.01.013 | Datasource Trait Adapter Pattern | status: active; lifecycle_status: active | Governs the `FetchStep` grammar and the spec-driven adapter contract. Time-window pushdown requires a `FetchStep` slot that receives the time predicate — the absence of such a slot in these sensors' specs is the structural root cause. |

**FetchStep grammar** (defined in `prism-spec-engine::spec_parser`) governs what
parameters the spec-driven adapter can populate per fetch. No ADR currently formalizes
what behavior is expected when a sensor's API has no time-window slot.

No ADR governs this defect's resolution path at present. The architect adjudication step
(see §Routing) will determine whether an ADR amendment to BC-2.01.013 or a new ADR is
required.

## Routing

**Architect adjudication precedes story decomposition.**

Route: `architect` + `product-owner` → `story-writer`

The architect must adjudicate whether:
(a) the spec grammar should expose a "no-time-window-pushdown" marker that disables
    time-window pushdown silently and documents the limitation, or
(b) the engine should surface a query warning when a time-window predicate cannot be
    pushed down, or
(c) an error should be returned when a time-window pushdown is requested against a sensor
    that has no slot for it.

The product-owner must decide whether to amend BC-2.01.013 to codify the chosen behavior
as a postcondition, and whether a separate story (or expansion of this stub) covers the
Cyberint-specific path.

After architect and PO adjudication, the story-writer produces acceptance criteria.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, architecture mapping, edge cases, and task breakdown are deferred to specification
time. This stub exists solely to register the defect and consolidate the previously
untracked `S-DEMO-CLAROTY-TIME-001` promise into a real artifact.

`tdd_mode` and Red Gate enumeration will be set by the spec-authoring agent (per SAC-1)
when acceptance criteria are authored.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage G2; subsumes untracked S-DEMO-CLAROTY-TIME-001 promise |
