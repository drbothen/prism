---
document_type: story
story_id: S-DEMO-CYBERINT-INCIDENTS-SEEDING-001
title: "prism-dtu-cyberint + prism-dtu-demo-server: Add Cyberint Incidents Generator Surface + /api/v1/incidents DTU Route [RETIRED — phantom endpoint]"
wave: null
epic_id: E-DEMO
priority: P2
status: retired
version: "v1.1"
level: "L4"
producer: story-writer
timestamp: "2026-06-09T00:00:00Z"
modified: "2026-07-22"
phase: 3
cycle: v1.0.0-brownfield
tdd_mode: strict
subsystems: [SS-01]
target_module: "prism-dtu-cyberint"
behavioral_contracts: []
# BC status: RETIRED — no BCs were authored; story was never materialized
verification_properties: []
depends_on: [S-DEMO-DTU-LIVE-SCENARIO-001-A]
blocks: []
points: 0
crates_touched: []
estimated_days: 0
inputs:
  - planning/findings-remediation-2026-07-20/triage-capture.md
input-hash: "9e688f3"
traces_to: null
assumption_validations: []
risk_mitigations: []
retirement_decision: D-1889
retirement_date: "2026-07-22"
superseded_by: DEFECT-CYBERINT-SPEC-FIDELITY-001
---

# S-DEMO-CYBERINT-INCIDENTS-SEEDING-001: Add Cyberint Incidents Generator Surface [RETIRED]

> **Status: RETIRED (2026-07-22, D-1889)**
>
> This story is retired and will never be dispatched. See §Retirement Note below.

## Authority

This story is `status: retired` — no implementation will be dispatched. No ADR governs the
implementation of this story because no implementation was ever authorized.

The retirement authority is D-1889 (human adjudication 2026-07-20), confirmed in
`triage-capture.md §Wrong-Direction Stories`. The retirement determination relied on ADR-053
§Finding-1 (Wave-A sensor fidelity remediation), which confirmed that the Cyberint API has no
`/api/v1/incidents` endpoint — the incidents table was a phantom endpoint. ADR-053 `status:
accepted`; `superseded_by: null`.

The superseding story is `DEFECT-CYBERINT-SPEC-FIDELITY-001`, which eliminates the phantom
incidents table entirely from the Cyberint spec.

---

## Retirement Note

**Decision:** RETIRE — approved per D-1889 (human adjudication 2026-07-20), confirmed in
`triage-capture.md §Wrong-Direction Stories`.

**Reason:** This story was registered (D-1083, 2026-06-09) as a concrete future anchor for
`BC-2.06.018 §Scope Boundary — Non-Generator-Backed Tables`. It was intended to add a
Cyberint incidents generator surface and a `/api/v1/incidents` DTU route so the incidents
table would become generator-backed.

**Why this is wrong-direction:** The Cyberint API has **no incidents object**. The incidents
table in `cyberint.sensor.toml` was a phantom endpoint — it declared `GET /api/v1/incidents`
which does not exist in the real Cyberint API. Everything is an alert. This was confirmed by
ADR-053 v0.27 §Finding-1 (Wave-A sensor fidelity remediation): "spec declares an `incidents`
table mapping `GET /api/v1/incidents` — an endpoint that does not exist. Real Cyberint has no
incidents object; everything is an alert."

**Superseding direction:** `DEFECT-CYBERINT-SPEC-FIDELITY-001` (Wave A) eliminates the phantom
incidents table entirely from the Cyberint spec. It retables Cyberint alerts correctly to
`POST /alert/api/v1/alerts` with `page/size`-based pagination (not cursor-only GET), and
re-clones the DTU against the correct OpenAPI grounding. The incidents concept for Cyberint
does not exist and has no replacement story.

**Grounding flip:** ADR-053 + ADR-054 (both ACCEPTED as of Wave-A spec evolution burst)
establish OpenAPI-grounded specs as the source of truth, superseding the prior DTU-grounded
approach (ADR-028 §D2). Under the new grounding model, the Cyberint incidents table never
had a legitimate OpenAPI basis.

**BC-2.06.018 §Scope Boundary impact:** The forward pointer in `BC-2.06.018 §Scope Boundary
— Non-Generator-Backed Tables` referencing this story is now stale. State-manager should
annotate BC-2.06.018 to note that the incidents-seeding gap is closed by the incidents-table
deletion in DEFECT-CYBERINT-SPEC-FIDELITY-001, not by this story.

## Narrative

N/A — Story retired before materialization. This section would have described adding a
Cyberint incidents generator surface and DTU route. Retirement renders it moot.

## Acceptance Criteria

N/A — Story retired before any BCs or ACs were authored. No tests will be written.

## Architecture Mapping

N/A — Story retired before materialization.

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| N/A | N/A — retired | N/A |

## Edge Cases

N/A — Story retired before materialization.

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| N/A | N/A — retired | N/A |

## Purity Classification

N/A — Story retired before materialization.

| Module | Classification | Justification |
|--------|---------------|---------------|
| N/A | N/A — retired | N/A |

## Token Budget Estimate (MANDATORY)

N/A — Story retired before materialization. No implementation will be dispatched.

| Context Source | Estimated Tokens |
|---------------|-----------------|
| N/A — retired | 0 |
| **Total** | **0** |
| Agent context window | N/A |
| **Budget usage** | **0%** |

## Tasks (MANDATORY)

N/A — Story retired. No tasks will be executed.

1. [ ] N/A — Story retired per D-1889; DEFECT-CYBERINT-SPEC-FIDELITY-001 supersedes

## Previous Story Intelligence (MANDATORY)

N/A — Story retired before first dispatch; no predecessor intelligence to carry forward.

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in epic | N/A — retired | N/A | N/A |

## Architecture Compliance Rules (MANDATORY)

N/A — Story retired before materialization.

| Rule | Source | Enforcement |
|------|--------|-------------|
| N/A — retired | N/A | N/A |

## Library & Framework Requirements (MANDATORY)

N/A — Story retired before materialization.

| Tool | Version | Purpose |
|------|---------|---------|
| N/A — retired | N/A | N/A |

## File Structure Requirements (MANDATORY)

N/A — Story retired before materialization. No files will be created or modified.

| File | Action | Purpose |
|------|--------|---------|
| N/A — retired | N/A | N/A |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| v1.1 | D-2084-authority | 2026-08-02 | story-writer | Round 6 DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 (D-2084): added §Authority section (retired story — documents retirement authority). |
| v1.0 | D-1889-retirement | 2026-07-22 | story-writer | Initial file creation as retirement stub. Story was previously STORY-INDEX-only (registered 2026-06-09 D-1083; `file: not-yet-authored`). Retired per triage-capture.md §Wrong-Direction Stories + D-1889 human adjudication (incidents=retire+derive). ADR-053 §Finding-1 confirms phantom endpoint. Superseded by DEFECT-CYBERINT-SPEC-FIDELITY-001. |
