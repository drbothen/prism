---
document_type: story
story_id: "DEFECT-HEALTH-PER-TABLE-BLINDSPOT-001"
title: "health probe is per-sensor not per-table — missed 80-minute E-SENSOR-030 window during live demo"
severity: HIGH
priority: P1
wave: tbd
epic_id: engine-defects
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
origin_finding: "F6 (D-1889 live-demo triage 2026-07-20)"
origin_cascade: "D-1889 live-demo triage (2026-07-20); decisions resolved at D-1943"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
subsystems: [SS-08]
behavioral_contracts:
  - BC-2.08.001
  - BC-2.08.005
  - BC-2.08.007
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: HIGH
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - .factory/planning/findings-remediation-2026-07-20/findings/prism-pql-deficiencies.md
  - .factory/planning/findings-remediation-2026-07-20/findings/prism-pushdown-audit.md
---

# DEFECT-HEALTH-PER-TABLE-BLINDSPOT-001: health probe is per-sensor not per-table — missed 80-minute E-SENSOR-030 window during live demo

## Problem

The health probe reports sensor status at per-sensor granularity. When a multi-table sensor
experiences table-level failures (for example, E-SENSOR-030 errors on one table while other
tables of the same sensor remain queryable), the health report shows the sensor as degraded
without identifying which table is affected. LLM agents and operators cannot determine which
query surfaces are failing.

During the D-1889 live-demo triage (2026-07-20), this granularity gap caused an 80-minute
window of E-SENSOR-030 errors to go undetected at the health tool surface. The per-sensor
health signal was insufficient to isolate the failing table.

BC-2.08.001, BC-2.08.005, and BC-2.08.007 govern sensor health behavior. Extending health
reporting to per-table granularity likely requires a new BC. Product-owner must determine
whether this is an amendment to one or more of the existing health BCs or a new standalone
behavioral contract, and author it accordingly before story decomposition can proceed.

## Origin

Identified in the D-1889 live-demo triage (2026-07-20). Source documents:
`findings/prism-pql-deficiencies.md` and `findings/prism-pushdown-audit.md` (referenced in
`triage-capture.md §source_readings`). Finding ID: F6, severity HIGH. Coverage NEW. Not
wave-assigned in the triage wave schedule.

This defect was unregistered as a tracking artifact prior to this stub. It existed only in
`triage-capture.md` (status: OPEN at time of triage; decisions resolved at D-1943).

## Authority

- BC-2.08.001 §Postconditions (on-demand connectivity check): governs what the health
  check measures at query time. Status: active.
- BC-2.08.005 §Postconditions (health MCP tool): governs the response shape and content
  of the `prism_health` MCP tool. Status: active.
- BC-2.08.007 §Postconditions (partial health status): governs the partial degraded state
  model when some sensors are healthy and others are not. Status: active.
- A new BC governing per-table health granularity is expected to be authored by product-owner
  as part of the remediation scope. No existing BC captures this requirement.
- No ADR currently governs this defect.

## Routing

product-owner → story-writer → implementer

1. **Product-owner**: determine whether per-table health requires amending the existing
   health BCs (BC-2.08.001 / BC-2.08.005 / BC-2.08.007) or authoring a new BC. Author the
   BC content.
2. **Story-writer**: decompose the resulting spec into implementable stories.
3. **Implementer**: TDD delivery.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test list (`RG-001..RG-NNN`), BC-5.38.001 density check, and
task decomposition are deferred to the routed specialist chain. This file is a registration
artifact — it makes the defect countable, discoverable, and assigned with a named owner.

Next step owner: **product-owner** (adjudicate BC structure: amendment vs new BC; author BC
content for per-table health granularity).
After BC authorship: **story-writer** (story decomposition), then **implementer**.

`tdd_mode` and enumerated Red Gate list MUST be set by product-owner when this story (or
its decomposed successors) reaches `status: ready`, per SAC-1.

## §Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub — defect identified in D-1889 live-demo triage (2026-07-20); unregistered prior to this artifact. |
