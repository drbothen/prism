---
document_type: remediation-manifest
cycle: phase-2-patch
scope: Wave 4 stories (S-3.03 through S-3.13, S-4.01)
date: "2026-04-20"
author: story-writer
status: complete
---

# Remediation Manifest — Wave 4 Stories (pre-build sweep)

Template-compliance sweep applied to all 12 Wave 4 stories before build phase.
Pattern: add missing frontmatter fields, ## Edge Cases section, rename ## Notes → ## Dev Notes,
rename ## Library and Framework Requirements → ## Library & Framework Requirements, append Changelog row.

---

## Manifest Table

| File | Old Version | New Version | level | points | anchor_bcs (count) | anchor_capabilities | anchor_subsystem | blocks (array) | Edge Cases added? | Dev Notes rename? | Library heading renamed? | Notes |
|------|------------|-------------|-------|--------|-------------------|---------------------|------------------|----------------|-------------------|-------------------|--------------------------|-------|
| S-3.03-explain-query.md | 1.1 | 1.1 | L4 | 2 | 1 | [CAP-015] | ["SS-11"] | [] | yes (synthesized from BC-2.11.010 + AC boundary cases) | yes | yes (`and` → `&`) | estimated_days=1 → points=2; blocks=[] confirmed by reverse scan |
| S-3.04-alias-system.md | 1.1 | 1.1 | L4 | 3 | 5 | [CAP-015] | ["SS-11"] | [] | already present | yes | yes (`and` → `&`) | estimated_days=2 → points=3; Edge Cases section pre-existed |
| S-3.05-pagination-caching.md | 1.2 | 1.2 | L4 | 3 | 6 | [CAP-015] | ["SS-07", "SS-11"] | [] | yes (synthesized from BC-2.07.001–006 boundary cases) | yes | yes (`and` → `&`) | estimated_days=2 → points=3; anchor_subsystem spans two subsystems (SS-07 + SS-11) |
| S-3.06-prismql-write-parser.md | 1.1 | 1.1 | L4 | 3 | 1 | [CAP-015] | ["SS-11"] | [S-3.07] | yes (synthesized from BC-2.11.004 + parser error boundary cases) | yes | yes (`and` → `&`) | estimated_days=2 → points=3; blocks=[S-3.07] confirmed by reverse scan |
| S-3.07-write-execution.md | 1.1 | 1.1 | L4 | 5 | 5 | [CAP-004, CAP-005, CAP-007] | ["SS-11", "SS-04", "SS-05"] | [] | yes (synthesized from BC-2.04.007/008 + BC-2.05.009 failure boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; anchor_subsystem spans three subsystems (SS-11, SS-04, SS-05) |
| S-3.08-hidden-columns.md | 1.1 | 1.1 | L4 | 2 | 0 | [] | null | [] | yes (synthesized from AC boundary cases — no formal BCs) | yes | yes (`and` → `&`) | estimated_days=1 → points=2; behavioral_contracts=[]; anchor fields empty |
| S-3.09-query-profiling.md | 1.1 | 1.1 | L4 | 2 | 0 | [] | null | [S-3.10] | yes (synthesized from AC boundary cases — no formal BCs) | yes | yes (`and` → `&`) | estimated_days=1 → points=2; blocks=[S-3.10] confirmed by reverse scan |
| S-3.10-cost-estimation.md | 1.1 | 1.1 | L4 | 3 | 0 | [] | null | [] | yes (synthesized from AC boundary cases — no formal BCs) | yes | yes (`and` → `&`) | estimated_days=2 → points=3; blocks=[] confirmed by reverse scan |
| S-3.11-in-query-caching.md | 1.1 | 1.1 | L4 | 2 | 0 | [] | null | [] | yes (synthesized from AC boundary cases — no formal BCs) | yes | yes (`and` → `&`) | estimated_days=1 → points=2; blocks=[] confirmed by reverse scan |
| S-3.12-column-pruning.md | 1.1 | 1.1 | L4 | 2 | 0 | [] | null | [] | yes (synthesized from AC boundary cases — no formal BCs) | yes | yes (`and` → `&`) | estimated_days=1 → points=2; blocks=[] confirmed by reverse scan |
| S-3.13-dynamic-table-availability.md | 1.3 | 1.3 | L4 | 2 | 0 | [] | null | [] | yes (synthesized from AC + hot-reload boundary cases — no formal BCs) | yes | yes (`and` → `&`) | estimated_days=1 → points=2; blocks=[] confirmed by reverse scan |
| S-4.01-schedule-crud.md | 1.3 | 1.3 | L4 | 5 | 5 | [CAP-016] | ["SS-12"] | [S-4.02, S-4.08, S-5.06] | yes (synthesized from BC-2.12.001/002/003/004/010 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; blocks=[S-4.02, S-4.08, S-5.06] confirmed by reverse scan |

---

## Derived Values Reference

### Points derivation (estimated_days → points)

| Story | estimated_days | points |
|-------|---------------|--------|
| S-3.03 | 1 | 2 |
| S-3.04 | 2 | 3 |
| S-3.05 | 2 | 3 |
| S-3.06 | 2 | 3 |
| S-3.07 | 3 | 5 |
| S-3.08 | 1 | 2 |
| S-3.09 | 1 | 2 |
| S-3.10 | 2 | 3 |
| S-3.11 | 1 | 2 |
| S-3.12 | 1 | 2 |
| S-3.13 | 1 | 2 |
| S-4.01 | 3 | 5 |

### Blocks derivation (reverse depends_on scan)

| Story | Blocks (stories with this ID in their depends_on) |
|-------|--------------------------------------------------|
| S-3.03 | (none) |
| S-3.04 | (none) |
| S-3.05 | (none) |
| S-3.06 | S-3.07 |
| S-3.07 | (none) |
| S-3.08 | (none) |
| S-3.09 | S-3.10 |
| S-3.10 | (none) |
| S-3.11 | (none) |
| S-3.12 | (none) |
| S-3.13 | (none) |
| S-4.01 | S-4.02, S-4.08, S-5.06 |

### anchor_bcs / anchor_capabilities / anchor_subsystem derivation

| Story | BC group | anchor_bcs count | anchor_capabilities | anchor_subsystem |
|-------|---------|-----------------|---------------------|-----------------|
| S-3.03 | BC-2.11.010 | 1 | [CAP-015] | ["SS-11"] |
| S-3.04 | BC-2.11.008/009/013/014/015 | 5 | [CAP-015] | ["SS-11"] |
| S-3.05 | BC-2.07.001/002/003/004/005/006 | 6 | [CAP-015] | ["SS-07", "SS-11"] |
| S-3.06 | BC-2.11.004 | 1 | [CAP-015] | ["SS-11"] |
| S-3.07 | BC-2.04.001/005/007/008 + BC-2.05.009 | 5 | [CAP-004, CAP-005, CAP-007] | ["SS-11", "SS-04", "SS-05"] |
| S-3.08 | (none) | 0 | [] | null |
| S-3.09 | (none) | 0 | [] | null |
| S-3.10 | (none) | 0 | [] | null |
| S-3.11 | (none) | 0 | [] | null |
| S-3.12 | (none) | 0 | [] | null |
| S-3.13 | (none) | 0 | [] | null |
| S-4.01 | BC-2.12.001/002/003/004/010 | 5 | [CAP-016] | ["SS-12"] |

---

## Surprises / Notes

1. **Library heading alias — all 12 renames performed:** Every Wave 4 story had `## Library and Framework Requirements` ("and" not "&"). All 12 were renamed to the hook-mandated `## Library & Framework Requirements` form.

2. **Edge Cases sections absent in 11 of 12 stories:** S-3.04 was the only story with a pre-existing `## Edge Cases` section. All others required synthesis from BC postconditions, error cases, and AC boundary conditions.

3. **Seven stories have no behavioral contracts:** S-3.08, S-3.09, S-3.10, S-3.11, S-3.12, S-3.13 (osquery-inspired enhancements without formal BCs) and no formal BC traceability. `anchor_bcs=[]`, `anchor_capabilities=[]`, `anchor_subsystem=null`.

4. **S-3.07 spans three subsystems:** BC-2.04 and BC-2.05 span SS-11 (query), SS-04 (security gates), and SS-05 (audit). anchor_subsystem is the union of all three.

5. **S-3.05 spans two subsystems:** SS-07 (Adapter Pagination & Response Cache) and SS-11 (Query Engine) are both declared in the story's `subsystems:` field.

6. **S-4.01 has the most blocks (3):** S-4.01 is the scheduling foundation for Wave 4; S-4.02 (diff-results), S-4.08 (action-delivery), and S-5.06 (action-infusion-tools) all depend on it.

7. **S-3.06 blocks S-3.07 only:** Write parser is a direct prerequisite for write execution; no other stories depend on S-3.06.

8. **S-3.09 blocks S-3.10 only:** Cost estimation depends on QueryMetrics from profiling; no other stories directly depend on S-3.09.

9. **Version handling:** Stories at version 1.1 retained 1.1; S-3.05 (1.2), S-3.13 (1.3), and S-4.01 (1.3) retained their current versions. No bumps per instructions; Changelog rows appended only.

10. **input-hash:** Left as null throughout. Step 4 (compute-input-hash) handles population.
