---
document_type: remediation-manifest
cycle: phase-2-patch
scope: Wave 3 stories (S-2.01 through S-2.08, S-3.01, S-3.02)
date: "2026-04-20"
author: story-writer
status: complete
---

# Remediation Manifest — Wave 3 Stories (pre-build sweep)

Template-compliance sweep applied to all 10 Wave 3 stories before build phase.
Pattern: add missing frontmatter fields, ## Edge Cases section, rename ## Notes → ## Dev Notes,
rename ## Library and Framework Requirements → ## Library & Framework Requirements, append Changelog row.

---

## Manifest Table

| File | Old Version | New Version | level | points | anchor_bcs (count) | anchor_capabilities | anchor_subsystem | blocks (array) | Edge Cases added? | Dev Notes rename? | Library heading renamed? | Notes |
|------|------------|-------------|-------|--------|-------------------|---------------------|------------------|----------------|-------------------|-------------------|--------------------------|-------|
| S-2.01-rocksdb-init.md | 1.1 | 1.1 | L4 | 5 | 3 | [CAP-019, CAP-024] | ["SS-15"] | [S-2.02, S-2.03, S-2.04, S-2.08, S-3.02, S-4.01, S-4.03, S-4.06, S-6.05] | yes (synthesized from BC-2.15.001/002/005 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; most-connected story in wave (9 blocks) |
| S-2.02-audit-buffer-watchdog.md | 1.1 | 1.1 | L4 | 3 | 5 | [CAP-024, CAP-025] | ["SS-15"] | [S-2.04] | yes (synthesized from BC-2.15.003/004/006/007/008 boundary cases) | yes | yes (`and` → `&`) | estimated_days=2 → points=3 |
| S-2.03-decorators-internal-tables.md | 1.1 | 1.1 | L4 | 3 | 3 | [CAP-026, CAP-028] | ["SS-15"] | [S-3.02] | yes (synthesized from BC-2.15.009/010/011 + EC-15-039 boundary cases) | yes | yes (`and` → `&`) | estimated_days=2 → points=3 |
| S-2.04-audit-construction.md | 1.1 | 1.1 | L4 | 5 | 6 | [CAP-007] | ["SS-05"] | [S-2.05, S-3.07] | yes (synthesized from BC-2.05.001/003/006 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5 |
| S-2.05-audit-events.md | 1.1 | 1.1 | L4 | 2 | 4 | [CAP-007] | ["SS-05"] | [] | yes (synthesized from BC-2.05.005/007/009/010 boundary cases) | yes | yes (`and` → `&`) | estimated_days=1 → points=2; blocks=[] confirmed by reverse scan |
| S-2.06-datasource-trait.md | 1.1 | 1.1 | L4 | 5 | 4 | [CAP-001, CAP-002] | ["SS-01"] | [S-2.07, S-2.08, S-3.02] | yes (synthesized from BC-2.01.002/010/013/014 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; anchor_capabilities spans two CAPs due to BC-2.01.010 covering [CAP-001, CAP-002] |
| S-2.07-per-sensor-auth.md | 1.1 | 1.1 | L4 | 5 | 5 | [CAP-001] | ["SS-01"] | [S-5.04] | yes (synthesized from BC-2.01.004/005/006/007/008 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5 |
| S-2.08-event-tables.md | 1.1 | 1.1 | L4 | 5 | 0 | [] | null | [] | yes (synthesized from AC boundary cases — no formal BCs assigned) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; behavioral_contracts=[]; anchor fields empty; blocks=[] confirmed by reverse scan |
| S-3.01-prismql-parser.md | 1.1 | 1.1 | L4 | 5 | 4 | [CAP-015] | ["SS-11"] | [S-3.02, S-3.06] | yes (synthesized from BC-2.11.002/003/004/006 + VP-014/015/021 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5 |
| S-3.02-query-materialization.md | 1.1 | 1.1 | L4 | 5 | 6 | [CAP-015] | ["SS-11"] | [S-3.03, S-3.04, S-3.05, S-3.07, S-3.08, S-3.09, S-4.01, S-4.03, S-5.01] | yes (synthesized from BC-2.11.001/005/006/007/011/012 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; most-connected story in wave (9 blocks) |

---

## Derived Values Reference

### Points derivation (estimated_days → points)

| Story | estimated_days | points |
|-------|---------------|--------|
| S-2.01 | 3 | 5 |
| S-2.02 | 2 | 3 |
| S-2.03 | 2 | 3 |
| S-2.04 | 3 | 5 |
| S-2.05 | 1 | 2 |
| S-2.06 | 3 | 5 |
| S-2.07 | 3 | 5 |
| S-2.08 | 3 | 5 |
| S-3.01 | 3 | 5 |
| S-3.02 | 3 | 5 |

### Blocks derivation (reverse depends_on scan)

| Story | Blocks (stories with this ID in their depends_on) |
|-------|--------------------------------------------------|
| S-2.01 | S-2.02, S-2.03, S-2.04, S-2.08, S-3.02, S-4.01, S-4.03, S-4.06, S-6.05 |
| S-2.02 | S-2.04 |
| S-2.03 | S-3.02 |
| S-2.04 | S-2.05, S-3.07 |
| S-2.05 | (none) |
| S-2.06 | S-2.07, S-2.08, S-3.02 |
| S-2.07 | S-5.04 |
| S-2.08 | (none) |
| S-3.01 | S-3.02, S-3.06 |
| S-3.02 | S-3.03, S-3.04, S-3.05, S-3.07, S-3.08, S-3.09, S-4.01, S-4.03, S-5.01 |

### anchor_bcs / anchor_capabilities / anchor_subsystem derivation

| Story | BC group | anchor_bcs count | anchor_capabilities | anchor_subsystem |
|-------|---------|-----------------|---------------------|-----------------|
| S-2.01 | BC-2.15.001/002/005 | 3 | [CAP-019, CAP-024] | ["SS-15"] |
| S-2.02 | BC-2.15.003/004/006/007/008 | 5 | [CAP-024, CAP-025] | ["SS-15"] |
| S-2.03 | BC-2.15.009/010/011 | 3 | [CAP-026, CAP-028] | ["SS-15"] |
| S-2.04 | BC-2.05.001/002/003/004/006/008 | 6 | [CAP-007] | ["SS-05"] |
| S-2.05 | BC-2.05.005/007/009/010 | 4 | [CAP-007] | ["SS-05"] |
| S-2.06 | BC-2.01.002/010/013/014 | 4 | [CAP-001, CAP-002] | ["SS-01"] |
| S-2.07 | BC-2.01.004/005/006/007/008 | 5 | [CAP-001] | ["SS-01"] |
| S-2.08 | (none) | 0 | [] | null |
| S-3.01 | BC-2.11.002/003/004/006 | 4 | [CAP-015] | ["SS-11"] |
| S-3.02 | BC-2.11.001/005/006/007/011/012 | 6 | [CAP-015] | ["SS-11"] |

---

## Surprises / Notes

1. **Library heading alias — all 10 renames performed:** Every Wave 3 story had `## Library and Framework Requirements` ("and" not "&"). All 10 were renamed to the hook-mandated `## Library & Framework Requirements` form.

2. **Edge Cases sections absent in all 10 stories:** Unlike some Wave 2 stories that had stub `[TODO]` rows, none of the Wave 3 stories had any `## Edge Cases` section at all. Sections were synthesized from BC postconditions, error cases, and AC boundary conditions, then inserted before `## Previous Story Intelligence` in all cases.

3. **S-2.08 has no behavioral contracts:** This story was explicitly noted as "osquery-inspired enhancement without formal BCs at this stage." anchor_bcs=[], anchor_capabilities=[], anchor_subsystem=null. Edge Cases were synthesized from ACs directly.

4. **S-2.01 and S-3.02 have the most blocks (9 each):** S-2.01 is the storage foundation blocking 9 downstream stories. S-3.02 is the query materialization engine blocking 9 downstream stories. Both are critical-path nodes.

5. **S-2.05 blocks is empty:** No story in the full set declares `depends_on: [S-2.05]`. Specialized audit events (S-2.05) are a leaf in the dependency graph. Correct.

6. **S-2.08 blocks is empty:** No story declares `depends_on: [S-2.08]`. Event table abstraction is consumed implicitly (via runtime behavior of S-2.06 adapters) rather than via a hard dependency. Correct.

7. **S-2.06 anchor_capabilities spans two CAPs:** BC-2.01.010 (partial failure handling) lists `capability: [CAP-001, CAP-002]` in its frontmatter. The union across all 4 BCs for S-2.06 is [CAP-001, CAP-002].

8. **Version handling:** All 10 stories were at version 1.1 before this sweep. Per instructions "Do NOT bump version" — all retained 1.1; Changelog rows appended only.

9. **input-hash:** Left as null throughout. Step 4 (compute-input-hash) handles population.
