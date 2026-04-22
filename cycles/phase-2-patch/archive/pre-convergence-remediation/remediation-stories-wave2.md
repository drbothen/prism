---
document_type: remediation-manifest
cycle: phase-2-patch
scope: Wave 2 stories (S-1.06 through S-1.15)
date: "2026-04-20"
author: story-writer
status: complete
---

# Remediation Manifest — Wave 2 Stories (pre-build sweep)

Template-compliance sweep applied to all 10 Wave 2 stories before build phase.
Pattern: add missing frontmatter fields, ## Edge Cases section, rename ## Notes → ## Dev Notes,
rename ## Library and Framework Requirements → ## Library & Framework Requirements, append Changelog row.

---

## Manifest Table

| File | Old Version | New Version | level | points | anchor_bcs (count) | anchor_capabilities (count) | anchor_subsystem | blocks (array) | Edge Cases added? | Dev Notes rename? | Library heading renamed? | Notes |
|------|------------|-------------|-------|--------|-------------------|----------------------------|------------------|----------------|-------------------|-------------------|--------------------------|-------|
| S-1.06-credential-store.md | 1.1 | 1.1 | L4 | 5 | 7 | 1 (CAP-004) | ["SS-03"] | [S-1.07, S-2.06, S-5.05, S-6.04] | yes (synthesized from BC-2.03 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5 |
| S-1.07-credential-crud.md | 1.3 | 1.3 | L4 | 3 | 5 | 1 (CAP-004) | ["SS-03"] | [S-6.04] | yes (stub filled with BC-2.03.005/006/007/009/010 cases) | yes | no (already `&`) | estimated_days=2 → points=3; version already ≥1.1 |
| S-1.08-feature-flags.md | 1.1 | 1.1 | L4 | 5 | 8 | 1 (CAP-005) | ["SS-04"] | [S-1.09, S-3.04, S-3.07, S-4.03, S-5.01] | yes (synthesized from BC-2.04.001/003/013/015 boundary cases) | yes | no (already `&`) | estimated_days=3 → points=5 |
| S-1.09-confirmation-tokens.md | 1.1 | 1.1 | L4 | 3 | 6 | 1 (CAP-005) | ["SS-04"] | [S-3.04, S-3.07] | yes (synthesized from VP-007/008/009/010 boundary cases) | yes | no (already `&`) | estimated_days=2 → points=3; EC synthesis notably derived from Kani VP proofs |
| S-1.10-prompt-injection-defense.md | 1.1 | 1.1 | L4 | 3 | 8 | 1 (CAP-010) | ["SS-09"] | [] | yes (synthesized from BC-2.09.003/004/006 boundary cases) | yes | no (already `&`) | estimated_days=2 → points=3; blocks=[] confirmed by reverse scan |
| S-1.11-spec-loading.md | 1.1 | 1.1 | L4 | 5 | 5 | 1 (CAP-029) | ["SS-16"] | [S-1.12, S-1.13, S-1.14, S-1.15, S-2.06, S-2.08] | yes (synthesized from BC-2.16.001/002/004/009 + VP-023 boundary cases) | yes | no (already `&`) | estimated_days=3 → points=5; most dependents of any Wave 2 story (6) |
| S-1.12-hot-reload.md | 1.1 | 1.1 | L4 | 3 | 5 | 1 (CAP-029) | ["SS-16"] | [S-3.13, S-5.07] | yes (synthesized from BC-2.16.005/007/008/010 + CI-002 invariant cases) | yes | no (already `&`) | estimated_days=2 → points=3 |
| S-1.13-sensor-write-specs.md | 1.1 | 1.1 | L4 | 3 | 2 | 1 (CAP-029) | ["SS-16"] | [S-3.06] | yes (synthesized from BC-2.16.009 validation rules) | yes | yes (`and` → `&`) | estimated_days=2 → points=3 |
| S-1.14-infusion-specs.md | 1.1 | 1.1 | L4 | 5 | 5 | 1 (CAP-031) | ["SS-16", "SS-19"] | [S-5.06] | yes (synthesized from BC-2.19.001/002/003/004/005 boundary cases) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; Edge Cases section absent entirely — added before ## Previous Story Intelligence |
| S-1.15-wasm-runtime.md | 1.2 | 1.2 | L4 | 5 | 6 | 1 (CAP-032) | ["SS-16", "SS-17"] | [S-4.08, S-5.09] | yes (synthesized from BC-2.17.001/002/003/004/005/006 boundary cases + KV scoping) | yes | yes (`and` → `&`) | estimated_days=3 → points=5; Edge Cases section absent entirely — added before ## Previous Story Intelligence |

---

## Derived Values Reference

### Points derivation (estimated_days → points)

| Story | estimated_days | points |
|-------|---------------|--------|
| S-1.06 | 3 | 5 |
| S-1.07 | 2 | 3 |
| S-1.08 | 3 | 5 |
| S-1.09 | 2 | 3 |
| S-1.10 | 2 | 3 |
| S-1.11 | 3 | 5 |
| S-1.12 | 2 | 3 |
| S-1.13 | 2 | 3 |
| S-1.14 | 3 | 5 |
| S-1.15 | 3 | 5 |

### Blocks derivation (reverse depends_on scan)

| Story | Blocks (stories with this ID in their depends_on) |
|-------|--------------------------------------------------|
| S-1.06 | S-1.07 (direct successor), S-2.06 (datasource-trait), S-5.05 (config-loading), S-6.04 (credential-cli) |
| S-1.07 | S-6.04 (credential-cli) |
| S-1.08 | S-1.09 (confirmation-tokens), S-3.04 (alias-system), S-3.07 (write-execution), S-4.03 (detection-rules), S-5.01 (mcp-bootstrap) |
| S-1.09 | S-3.04 (alias-system), S-3.07 (write-execution) |
| S-1.10 | (none — no stories declare depends_on: [S-1.10]) |
| S-1.11 | S-1.12, S-1.13, S-1.14, S-1.15 (direct successors), S-2.06 (datasource-trait), S-2.08 (event-tables) |
| S-1.12 | S-3.13 (dynamic-table-availability), S-5.07 (multi-repo-git-config) |
| S-1.13 | S-3.06 (prismql-write-parser) |
| S-1.14 | S-5.06 (action-infusion-tools) |
| S-1.15 | S-4.08 (action-delivery), S-5.09 (external-log-forwarding) |

### anchor_bcs / anchor_capabilities / anchor_subsystem derivation

| Story | BC group | anchor_bcs count | anchor_capabilities | anchor_subsystem |
|-------|---------|-----------------|--------------------|-----------------| 
| S-1.06 | BC-2.03.* | 7 | [CAP-004] | ["SS-03"] |
| S-1.07 | BC-2.03.* | 5 | [CAP-004] | ["SS-03"] |
| S-1.08 | BC-2.04.* | 8 | [CAP-005] | ["SS-04"] |
| S-1.09 | BC-2.04.* | 6 | [CAP-005] | ["SS-04"] |
| S-1.10 | BC-2.09.* | 8 | [CAP-010] | ["SS-09"] |
| S-1.11 | BC-2.16.* | 5 | [CAP-029] | ["SS-16"] |
| S-1.12 | BC-2.16.* | 5 | [CAP-029] | ["SS-16"] |
| S-1.13 | BC-2.16.* | 2 | [CAP-029] | ["SS-16"] |
| S-1.14 | BC-2.19.* | 5 | [CAP-031] | ["SS-16", "SS-19"] |
| S-1.15 | BC-2.17.* | 6 | [CAP-032] | ["SS-16", "SS-17"] |

---

## Surprises / Notes

1. **Library heading alias — 5 renames performed:** S-1.06, S-1.13, S-1.14, S-1.15 had `## Library and Framework Requirements` ("and" not "&"). S-1.07, S-1.08, S-1.09, S-1.10, S-1.11, S-1.12 already had the correct ampersand form. The hook enforces the exact heading — all 5 were renamed to unblock.

2. **Edge Cases sections absent entirely in S-1.14 and S-1.15:** These stories had no `## Edge Cases` section at all (unlike S-1.07/08/09/10/11/12 which had stub `[TODO]` rows). Sections were inserted before `## Previous Story Intelligence` in both cases.

3. **S-1.10 blocks is empty:** No story in the full 75-story set declares `depends_on: [S-1.10]`. Prompt injection defense (SS-09) is consumed by prism-mcp inline, not via a hard story dependency. Correct.

4. **S-1.09 Edge Cases synthesized from VP proofs:** The four Kani VPs (VP-007 through VP-010) each represent a boundary condition that directly maps to a test-able edge case. The Edge Cases table for S-1.09 was synthesized directly from those VP postconditions — the most mechanically clean EC synthesis of this wave.

5. **S-1.11 most-connected story in wave:** S-1.11 blocks 6 stories (S-1.12, S-1.13, S-1.14, S-1.15, S-2.06, S-2.08). It is the highest-fanout dependency in Wave 2 and the primary critical-path risk.

6. **Version handling:** S-1.14 was at 1.1 and S-1.15 at 1.2 before this sweep. Per instructions "Do NOT bump version if already ≥1.1" — both retained their version numbers; Changelog rows appended only.

7. **input-hash:** Left as null throughout. Step 4 (compute-input-hash) handles population.
