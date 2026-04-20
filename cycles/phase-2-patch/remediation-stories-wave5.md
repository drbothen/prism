---
document_type: remediation-manifest
cycle: phase-2-patch
scope: Wave 5 stories (S-4.02 through S-4.08, S-5.01 through S-5.03)
date: "2026-04-20"
author: story-writer
status: complete
---

# Remediation Manifest — Wave 5 Stories (pre-build sweep)

Template-compliance sweep applied to all 10 Wave 5 stories before build phase.
Pattern: add missing frontmatter fields, ## Edge Cases section, rename ## Notes → ## Dev Notes,
rename ## Library and Framework Requirements → ## Library & Framework Requirements, append Changelog row.

---

## Manifest Table

| File | Old Version | New Version | level | points | anchor_bcs (count) | anchor_capabilities | anchor_subsystem | blocks (array) | Edge Cases added? | Dev Notes rename? | Library heading renamed? | Notes |
|------|------------|-------------|-------|--------|-------------------|---------------------|------------------|----------------|-------------------|-------------------|--------------------------|-------|
| S-4.02-diff-results-packs.md | 1.1 | 1.1 | L4 | 3 | 5 | [CAP-016] | ["SS-12"] | [] | yes (synthesized from BC-2.12.005/006/007/008/009 boundary cases) | yes | yes (`and` → `&`) | estimated_days=2 → points=3; blocks=[] confirmed by reverse scan |
| S-4.03-detection-rules.md | 1.2 | 1.2 | L4 | 5 | 8 | [CAP-017] | ["SS-13"] | [S-4.04] | yes (synthesized from BC-2.13.001/006/008/010/011/014 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; blocks=[S-4.04] confirmed by reverse scan |
| S-4.04-detection-evaluation.md | 1.1 | 1.1 | L4 | 5 | 5 | [CAP-017] | ["SS-13"] | [S-4.05] | yes (synthesized from BC-2.13.002/003/004/012/013 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; blocks=[S-4.05] confirmed by reverse scan |
| S-4.05-alert-generation.md | 1.1 | 1.1 | L4 | 2 | 1 | [CAP-017] | ["SS-13"] | [S-4.06, S-4.08] | yes (synthesized from BC-2.13.005 boundary cases) | yes | yes (`and` → `&`) | estimated_days=1 → points=2; blocks=[S-4.06, S-4.08] confirmed by reverse scan |
| S-4.06-case-management.md | 1.1 | 1.1 | L4 | 5 | 9 | [CAP-018] | ["SS-14"] | [S-4.07, S-4.08] | yes (synthesized from BC-2.14.001/002/003/006/007/013 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; blocks=[S-4.07, S-4.08] confirmed by reverse scan |
| S-4.07-case-metrics.md | 1.1 | 1.1 | L4 | 3 | 3 | [CAP-018] | ["SS-14"] | [] | yes (synthesized from BC-2.14.008/010/012 boundary cases) | yes | yes (`and` → `&`) | estimated_days=2 → points=3; blocks=[] confirmed by reverse scan |
| S-4.08-action-delivery.md | 1.1 | 1.1 | L4 | 5 | 9 | [CAP-018] | ["SS-18"] | [S-5.06] | yes (synthesized from BC-2.18.001–009 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; blocks=[S-5.06] confirmed by reverse scan |
| S-5.01-mcp-bootstrap.md | 1.4 | 1.4 | L4 | 5 | 7 | [CAP-010] | ["SS-10"] | [S-5.02, S-5.05, S-5.06, S-5.08] | yes (synthesized from BC-2.10.001/002/003/005/006/010 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; blocks=[S-5.02, S-5.05, S-5.06, S-5.08] confirmed by reverse scan |
| S-5.02-tool-routing.md | 1.1 | 1.1 | L4 | 3 | 3 | [CAP-010] | ["SS-10"] | [S-5.03, S-5.08] | yes (synthesized from BC-2.10.004/007/011 boundary cases) | yes | yes (`and` → `&`) | estimated_days=2 → points=3; blocks=[S-5.03, S-5.08] confirmed by reverse scan |
| S-5.03-resources-prompts.md | 1.4 | 1.4 | L4 | 3 | 4 | [CAP-010] | ["SS-10", "SS-08"] | [S-5.04, S-5.08] | yes (synthesized from BC-2.10.008/009/BC-2.08.005/006 boundary cases) | yes | yes (`and` → `&`) | estimated_days=2 → points=3; blocks=[S-5.04, S-5.08] confirmed by reverse scan; anchor_subsystem spans SS-10 + SS-08 |

---

## Derived Values Reference

### Points derivation (estimated_days → points)

| Story | estimated_days | points |
|-------|---------------|--------|
| S-4.02 | 2 | 3 |
| S-4.03 | 3 | 5 |
| S-4.04 | 3 | 5 |
| S-4.05 | 1 | 2 |
| S-4.06 | 3 | 5 |
| S-4.07 | 2 | 3 |
| S-4.08 | 3 | 5 |
| S-5.01 | 3 | 5 |
| S-5.02 | 2 | 3 |
| S-5.03 | 2 | 3 |

### Blocks derivation (reverse depends_on scan)

| Story | Blocks (stories with this ID in their depends_on) |
|-------|--------------------------------------------------|
| S-4.02 | (none) |
| S-4.03 | S-4.04 |
| S-4.04 | S-4.05 |
| S-4.05 | S-4.06, S-4.08 |
| S-4.06 | S-4.07, S-4.08 |
| S-4.07 | (none) |
| S-4.08 | S-5.06 |
| S-5.01 | S-5.02, S-5.05, S-5.06, S-5.08 |
| S-5.02 | S-5.03, S-5.08 |
| S-5.03 | S-5.04, S-5.08 |

### anchor_bcs / anchor_capabilities / anchor_subsystem derivation

| Story | BC group | anchor_bcs count | anchor_capabilities | anchor_subsystem |
|-------|---------|-----------------|---------------------|-----------------|
| S-4.02 | BC-2.12.005/006/007/008/009 | 5 | [CAP-016] | ["SS-12"] |
| S-4.03 | BC-2.13.001/006/007/008/009/010/011/014 | 8 | [CAP-017] | ["SS-13"] |
| S-4.04 | BC-2.13.002/003/004/012/013 | 5 | [CAP-017] | ["SS-13"] |
| S-4.05 | BC-2.13.005 | 1 | [CAP-017] | ["SS-13"] |
| S-4.06 | BC-2.14.001/002/003/004/005/006/007/009/013 | 9 | [CAP-018] | ["SS-14"] |
| S-4.07 | BC-2.14.008/010/012 | 3 | [CAP-018] | ["SS-14"] |
| S-4.08 | BC-2.18.001/002/003/004/005/006/007/008/009 | 9 | [CAP-018] | ["SS-18"] |
| S-5.01 | BC-2.04.014/BC-2.10.001/002/003/005/006/010 | 7 | [CAP-010] | ["SS-10"] |
| S-5.02 | BC-2.10.004/007/011 | 3 | [CAP-010] | ["SS-10"] |
| S-5.03 | BC-2.10.008/009/BC-2.08.005/006 | 4 | [CAP-010] | ["SS-10", "SS-08"] |

---

## Surprises / Notes

1. **Library heading alias — all 10 renames performed:** Every Wave 5 story had `## Library and Framework Requirements` ("and" not "&"). All 10 were renamed to the hook-mandated `## Library & Framework Requirements` form.

2. **Edge Cases sections absent in all 10 stories:** None of the Wave 5 stories had a pre-existing `## Edge Cases` section. All required synthesis from BC postconditions, error cases, and AC boundary conditions.

3. **S-4.08 anchor_subsystem set to SS-18:** S-4.08 spans SS-12, SS-13, SS-14, and SS-18 in its subsystems field. SS-18 (Action Delivery Engine) is the primary owning subsystem per BC-2.18.* grouping; `anchor_subsystem` reflects the primary owner only.

4. **S-5.03 spans two subsystems:** SS-10 (MCP Interface) and SS-08 (Sensor Health) are both declared in the story's `subsystems:` field; `anchor_subsystem` is the union of both.

5. **S-5.01 has the most blocks (4):** S-5.01 is the MCP bootstrap foundation for Wave 5; S-5.02, S-5.05, S-5.06, and S-5.08 all depend on it.

6. **S-4.05 blocks two stories:** S-4.06 (Case Management) and S-4.08 (Action Delivery) both depend on S-4.05's alert broadcast channel and `Alert` type.

7. **S-4.06 blocks two stories:** S-4.07 (Case Metrics) and S-4.08 (Action Delivery) both depend on S-4.06's case management foundation.

8. **Version handling:** Stories retained their existing versions (no bumps per instructions). Changelog rows appended only. S-4.08 had an out-of-order Changelog (1.1 before 1.0); new row appended after the existing entries.

9. **input-hash:** Left as null throughout. Step 4 (compute-input-hash) handles population.

10. **Hook fires on every edit:** The validate-template-compliance hook fires after each individual Edit tool call, not just at the end. This required a minimum of 3 edits per story (frontmatter, Library heading, Edge Cases+Dev Notes) to satisfy the hook incrementally.
