---
document_type: story
story_id: "S-MAINT-POL29-HOOK-001"
title: "Implement validate-cite-pin-completeness.sh — POL-29 step-8 mechanization"
wave: tbd
epic_id: maintenance
priority: P2
status: planned
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-05-22"
modified: "2026-05-22"
input-hash: "[live-state]"
inputs: []
traces_to: ""
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: ".factory/hooks"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
depends_on: []
blocks: []
points: tbd
estimated_days: tbd
risk: LOW
acceptance_criteria_count: 0
red_gate_tests: 0
estimated_passes: "tbd"
holdout_scenarios: []
assumption_validations: []
---

# S-MAINT-POL29-HOOK-001: Implement validate-cite-pin-completeness.sh — POL-29 step-8 mechanization

## §Origin

This story originates from `.factory/cycles/wave-0-plugin-prereqs/session-review-2026-05-22.md` Group E (D-777 codification of PLUGIN-MIGRATION-001-D Option B exit). During the PLUGIN-MIGRATION-001-D LOCAL cascade, POL-29 v1.28 (cite-pin propagation policy) produced 7 distinct axis recurrences despite having 9 step-8 substeps and ~4,500 words of procedural text. The session-reviewer classified this as a growth-complexity asymptote: adding more policy text is over-codification because agents cannot reliably apply natural-language fixed-point grep procedures.

The strategic fix (Option B) is to mechanize the POL-29 step-8 cite-pin sweep as a deterministic lint hook.

## §Problem Statement

POL-29 (cite-pin propagation) reached v1.28 through iterative accretion — each cascade recurrence produced a new substep. Despite this growth, the PLUGIN-MIGRATION-001-D cascade produced 7 distinct axis recurrences:

1. POL-29 §3a registry artifact bumps not propagated
2. Same-line dual-format cite-pin (file-version + section-version) missed
3. crates/ scope not covered (§8f extension, added as v1.29)
4. Pipe-table-cell variant form not matched by grep
5. Transitive cite-pin chain (A→B→C) — intermediate link not updated
6. 2nd-order side-effect bump (B gains §Changelog row → B's downstream stale)
7. Self-2nd-order propagation (body header cites stale frontmatter version)

Root cause: natural-language grep procedures cannot be enforced reliably across agents with varying interpretation. The policy text describes what to do, but provides no enforcement mechanism when an agent misses a step. Each coverage gap becomes a new recurrence class, not a resolved one.

## §Proposed Solution

Implement `.factory/hooks/validate-cite-pin-completeness.sh` (or equivalent) as a deterministic lint hook that:

1. Executes the POL-29 step-8 grep procedure deterministically across `.factory/`, `crates/`, `tests/`, `docs/`
2. Detects stale cite-pins to old versions of BCs, ADRs, error-taxonomy, story IDs, and policy IDs
3. Returns exit-code-1 with specific stale cite-pin locations on violation
4. Returns exit-code-0 when clean
5. Runs in <5 seconds on a cold workspace
6. Integrates with the existing `lefthook.yml` pre-commit chain OR the factory-dispatcher hook chain

After this hook ships, POL-29 steps 8a-8j become **descriptive documentation** of hook behavior rather than **executable agent instructions**. The recurrence class is eliminated because the hook fails deterministically on stale cite-pins rather than relying on agent diligence.

## §Success Criteria (High-Level — Sprint-Ready Story Will Refine)

- Hook script exists at `.factory/hooks/validate-cite-pin-completeness.sh` or equivalent
- Hook is wired into the pre-commit chain (lefthook + factory-dispatcher)
- Hook detects ALL 7 axis classes documented in `lessons.md` entries 14-37+38:
  1. POL-29 §3a registry artifact bumps
  2. Same-line dual-format cite-pin (file-version + section-version)
  3. crates/ scope (§8f extension)
  4. Pipe-table-cell variant form
  5. Transitive cite-pin chain (A→B→C)
  6. 2nd-order side-effect bump (B gains §Changelog row → B's downstream stale)
  7. Self-2nd-order propagation (body header cites stale frontmatter version)
- Test fixtures: a sample dirty workspace with each axis class triggers exit-1; a clean workspace passes exit-0
- Performance: <5s cold; <2s warm
- TD-VSDD-091 exempt classes excluded from grep (§Changelog rows, task-body attestation cites, immutable historical snapshots)
- Documentation: README at `.factory/hooks/README.md` updated with hook coverage matrix

## §Tasks

To be materialized in a dedicated maintenance burst.

## §Previous Story Intelligence

N/A — first story in the maintenance/hooks epic.

## §Architecture Compliance Rules

To be materialized when the architectural decisions in §Open Questions are resolved.

## §Library & Framework Requirements

To be determined — pending language choice (see §Open Questions).

## §File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.factory/hooks/validate-cite-pin-completeness.sh` | Create | Primary hook script |
| `.factory/hooks/README.md` | Create or update | Hook coverage matrix documentation |
| `lefthook.yml` | Modify | Wire hook into pre-commit chain |
| `.factory/policies.yaml` | Modify (POL-29) | Demote steps 8a-8j to descriptive; add hook reference |

## §Token Budget Estimate

To be materialized. Estimated at sprint-ready time.

## §Open Questions for Refinement

- **Language choice:** bash (matches existing `.factory/hooks/*.sh`) vs Rust (typed, heavier dep) vs Python (readable, but pyyaml dependency disliked per recent burst issues)
- **Run scope:** pre-commit only, or also pre-push?
- **False-positive handling:** TD-VSDD-091 exempt classes must be catalogued and excluded before the hook can run clean on the current workspace
- **Hook source location:** project-local (`.factory/hooks/`) vs upstream vsdd-factory plugin
- **Ownership decision:** Platform Engineering devops-engineer, or spec-tooling specialist?

## §Defer Rationale (STUB Status)

This story is STUB-level (not yet sprint-ready) because:

1. The architectural decisions above (language, scope, source location) require deliberation before task authorship
2. The critical path for production deployment is PLUGIN-MIGRATION-001-A → 001-B → 001-C → live MCP+DTU+OCSF demo
3. This hook is a developer-productivity investment, not a feature-blocker; P2 priority is correct
4. Materializing to sprint-ready level should happen in a dedicated maintenance burst after 001-A/B/C land or in the natural next maintenance sweep

## §Source Citations

- `.factory/cycles/wave-0-plugin-prereqs/session-review-2026-05-22.md` — D-777 (Group E, Option B exit)
- `.factory/cycles/wave-0-plugin-prereqs/lessons.md` — entries 14-37+38 (7 POL-29 axis recurrences during PLUGIN-MIGRATION-001-D LOCAL cascade)
- `.factory/policies.yaml` — POL-29 (current state post-A1 amendment)
- `CLAUDE.md` §Operational Discipline TDs — TD-VSDD-053, TD-VSDD-060, TD-VSDD-091
