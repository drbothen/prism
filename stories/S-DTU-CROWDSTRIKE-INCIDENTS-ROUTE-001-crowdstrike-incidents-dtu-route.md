---
document_type: story
story_id: S-DTU-CROWDSTRIKE-INCIDENTS-ROUTE-001
title: "prism-dtu-crowdstrike + prism-sensors: Add DTU Route Handler for CrowdStrike Incidents Table [RETIRED — API removed]"
wave: null
epic_id: E-DTU
priority: P2
status: retired
version: "v1.0"
level: "L4"
producer: story-writer
timestamp: "2026-06-12T00:00:00Z"
modified: "2026-07-22"
phase: 3
cycle: v1.0.0-brownfield
tdd_mode: strict
subsystems: [SS-01]
target_module: "prism-dtu-crowdstrike"
behavioral_contracts: []
# BC status: RETIRED — no BCs were authored; story was never materialized (PRODUCT-DECISION-PENDING at registration)
verification_properties: []
depends_on: []
blocks: []
points: 0
crates_touched: [prism-dtu-crowdstrike, prism-sensors]
estimated_days: 0
inputs:
  - planning/findings-remediation-2026-07-20/triage-capture.md
input-hash: "9e688f3"
traces_to: null
assumption_validations: []
risk_mitigations: []
retirement_decision: D-1889
retirement_date: "2026-07-22"
superseded_by: S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001
---

# S-DTU-CROWDSTRIKE-INCIDENTS-ROUTE-001: Add CrowdStrike Incidents DTU Route [RETIRED]

> **Status: RETIRED (2026-07-22, D-1889)**
>
> This story is retired and will never be dispatched. See §Retirement Note below.

## Retirement Note

**Decision:** RETIRE — approved per D-1889 (human adjudication 2026-07-20), confirmed in
`triage-capture.md §Wrong-Direction Stories`. Open Decision #4 (incidents retire-vs-build
conflict) resolved as "incidents=retire+derive".

**Reason:** This story was registered (D-1103, 2026-06-12) as a PRODUCT-DECISION-PENDING
stub to add a DTU route handler for the CrowdStrike `incidents` table.
`crowdstrike.sensor.toml` declared a `[[tables]]` block with `table_name = "incidents"` and
a 2-step fetch pipeline (`/incidents/queries/incidents/v1` →
`/incidents/entities/incidents/GET/v1`), but `prism-dtu-crowdstrike` had no corresponding
route handler (gap labeled `# DTU-EXT-001` in the TOML). This story was registered to close
that DTU-spec parity gap.

**Why this is wrong-direction:** The CrowdStrike Incidents API was **removed approximately
2026-03**. There is no live API endpoint to clone. Building a DTU route for a removed API
would create a behavioral clone of a non-existent service, which has no value for integration
testing. This was confirmed by `triage-capture.md §S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001`
(CRIT finding): "Incidents API removed ~2026-03; retire table; DTU-EXT-001 re-adjudicate."

**DTU-EXT-001 disposition:** The `BC-2.16.013 §Known Gaps` entry `DTU-EXT-001` (which this
story was meant to close) is being RETIRED separately. See state-manager dispatch to annotate
BC-2.16.013 and tech-debt-register.md.

**Superseding direction:** `S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001` (Wave A, CRIT) is the
correct replacement. It:
1. Retires the `incidents` table from `crowdstrike.sensor.toml` entirely
2. Derives incidents from the Alerts table via `aggregate_id` field (incidents are a
   conceptual grouping of alerts, not a first-class API object)
3. Updates the DTU to reflect the removal (no incidents route needed)

**ADR grounding:** ADR-053 + ADR-054 (both ACCEPTED as of Wave-A spec evolution burst)
establish OpenAPI-grounded specs as the source of truth. Under OpenAPI grounding, the
CrowdStrike incidents table has no valid OpenAPI anchor post-2026-03, confirming retirement.

## Narrative

N/A — Story retired before materialization. This section would have described adding a
two-step DTU route handler (`/incidents/queries/incidents/v1` +
`/incidents/entities/incidents/GET/v1`) to `prism-dtu-crowdstrike`. Retirement renders it moot.

## Acceptance Criteria

N/A — Story retired before any BCs or ACs were authored (was PRODUCT-DECISION-PENDING at
registration; S-7.01 gate prevented BC authorship). No tests will be written.

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

1. [ ] N/A — Story retired per D-1889; S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001 supersedes

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
| v1.0 | D-1889-retirement | 2026-07-22 | story-writer | Initial file creation as retirement stub. Story was previously STORY-INDEX-only (registered 2026-06-12 D-1103; `file: not-yet-authored`; PRODUCT-DECISION-PENDING). Retired per triage-capture.md §Wrong-Direction Stories + D-1889 human adjudication (incidents=retire+derive). CrowdStrike Incidents API removed ~2026-03. DTU-EXT-001 RETIRED separately in BC-2.16.013. Superseded by S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001. |
