# Wave 6 Story Remediation Manifest
# Phase 2 Pre-Build Sweep

**Burst:** pre-build-sweep
**Date:** 2026-04-20
**Author:** story-writer
**Scope:** All 10 Wave 6 stories (S-5.04 backfilled + S-5.05 through S-6.03)

---

## Summary

Template-compliance sweep applied to all Wave 6 stories. Each story received:

1. **Frontmatter additions:** `level`, `inputs`, `points`, `blocks`, `assumption_validations`, `risk_mitigations`, `anchor_bcs`, `anchor_capabilities`, `anchor_subsystem`
2. **Section normalization:** `## Notes` → `## Dev Notes`; `## Library and Framework Requirements` → `## Library & Framework Requirements`
3. **New section:** `## Edge Cases` scaffolded with story-specific entries
4. **Changelog row** appended at current version

---

## Story Table

| Story | Title | Points | Blocks | Changes Applied |
|-------|-------|--------|--------|-----------------|
| S-5.04 | prism-mcp: Sensor Health Subsystem | 3 | [] | Completed by prior agent (B-pre-build-sweep). Backfilled here for completeness. |
| S-5.05 | prism-mcp: Config Loading and Validation | 5 | [S-5.07, S-6.01] | Frontmatter (level/inputs/points/blocks/anchor_*); ## Edge Cases (6 entries); ## Dev Notes rename; ## Library & Framework Requirements rename |
| S-5.06 | prism-mcp: Action and Infusion MCP Tools | 3 | [] | Frontmatter (level/inputs/points/blocks/anchor_*); ## Edge Cases (6 entries); ## Dev Notes rename; ## Library & Framework Requirements rename |
| S-5.07 | prism-mcp: Multi-Repo Git Config Subscriptions | 8 | [] | Frontmatter (level/inputs/points/blocks/anchor_*); ## Edge Cases (6 entries); ## Dev Notes rename; ## Library & Framework Requirements rename |
| S-5.08 | prism-mcp: Diagnostics — prism logs CLI + get_diagnostics + Trace IDs | 8 | [S-5.09] | Frontmatter (level/inputs/points/blocks/anchor_*); ## Edge Cases (6 entries); ## Dev Notes rename; ## Library & Framework Requirements rename |
| S-5.09 | prism-mcp: External Log Forwarding Subsystem | 8 | [S-5.10] | Frontmatter (level/inputs/points/blocks/anchor_*); ## Edge Cases (6 entries); ## Dev Notes rename; ## Library & Framework Requirements rename |
| S-5.10 | prism-audit: Audit Trail External Forwarding | 5 | [] | Frontmatter (level/inputs/points/blocks/anchor_*); ## Edge Cases (6 entries); ## Dev Notes rename; ## Library & Framework Requirements rename |
| S-6.01 | prism-bin: CLI, Startup, and Initialization | 3 | [S-6.02, S-6.03] | Frontmatter (level/inputs/points/blocks/anchor_*); behavioral_contracts populated from body BC table; ## Edge Cases (6 entries); ## Dev Notes rename; ## Library & Framework Requirements rename |
| S-6.02 | prism-bin: End-to-End Integration Smoke Tests | 3 | [] | Frontmatter (level/inputs/points/blocks/anchor_*); behavioral_contracts populated from body BC table; ## Edge Cases (5 entries); ## Dev Notes rename; ## Library & Framework Requirements rename |
| S-6.03 | prism-bin: Installation and Distribution | 2 | [] | Frontmatter (level/inputs/points/blocks/anchor_*); behavioral_contracts populated from body BC table; ## Edge Cases (5 entries); ## Dev Notes rename; ## Library & Framework Requirements rename |

---

## Points Summary

| Wave | Stories | Total Points |
|------|---------|-------------|
| Wave 6 | 10 | 48 |

Breakdown: S-5.04 (3) + S-5.05 (5) + S-5.06 (3) + S-5.07 (8) + S-5.08 (8) + S-5.09 (8) + S-5.10 (5) + S-6.01 (3) + S-6.02 (3) + S-6.03 (2) = **48 points**

---

## Frontmatter Fields Added Per Story

### anchor_capabilities derivation
| Story | Subsystems | anchor_capabilities |
|-------|-----------|---------------------|
| S-5.04 | SS-08 | [CAP-008] |
| S-5.05 | SS-06 | [CAP-006] |
| S-5.06 | SS-10 | [CAP-010] |
| S-5.07 | SS-06 | [CAP-006] |
| S-5.08 | SS-08, SS-10 | [CAP-008] |
| S-5.09 | SS-20 | [CAP-010] |
| S-5.10 | SS-05, SS-20 | [CAP-005] |
| S-6.01 | SS-06, SS-10 (via BCs) | [CAP-006, CAP-010] |
| S-6.02 | SS-06, SS-08, SS-10 (via BCs) | [CAP-006, CAP-008, CAP-010] |
| S-6.03 | SS-10 (via BCs) | [CAP-010] |

### points derivation (estimated_days → points)
| estimated_days | points rule | Stories |
|---------------|-------------|---------|
| 1d | 2 | S-6.03 |
| 2d | 3 | S-5.04, S-5.06, S-6.01, S-6.02 |
| 3d | 5 | S-5.05, S-5.10 |
| 4d | 8 | S-5.07, S-5.09 |
| 5d | 8 | S-5.08 |

---

## Dependency Notes

- `S-5.08 blocks: [S-5.09]` — S-5.09 (log forwarding) depends_on S-5.08 (log writer + LogEntry type)
- `S-5.09 blocks: [S-5.10]` — S-5.10 (audit forwarding) listed as separate story but S-5.09 is a peer; blocks set conservatively as empty for S-5.10 since it depends_on S-2.04 not S-5.09
- `S-5.05 blocks: [S-5.07, S-6.01]` — S-5.07 depends_on S-5.05; S-6.01 depends_on S-5.05
- `S-6.01 blocks: [S-6.02, S-6.03]` — both depend_on S-6.01

---

## Compliance Status

All 9 remediated stories pass the `validate-template-compliance.sh` hook as of this sweep. Hook checks confirmed:
- `## Edge Cases` present
- `## Library & Framework Requirements` (ampersand form) present
- `## Dev Notes` present (was `## Notes`)

No input-hash values were modified. No commits made (per directive).
