---
document_type: remediation-manifest
wave: 7
title: "Wave 7 Story Remediation — Pre-Build Sweep"
date: "2026-04-20"
author: story-writer
stories_covered: [S-6.04, S-6.05, S-6.06, S-6.07, S-6.08, S-6.09, S-6.10, S-6.11, S-6.12, S-6.13]
---

# Wave 7 Story Remediation — Pre-Build Sweep

Template-compliance remediation for Wave 7 stories (S-6.04 – S-6.13).
Wave 7 contains the credential CLI, storage migration CLI, and DTU (Digital Twin Universe)
clone infrastructure stories.

---

## Summary

| Story | Title | Prior Agent Sweep | This Sweep | Status |
|-------|-------|-------------------|------------|--------|
| S-6.04 | prism-bin: prism credential CLI Subcommand Group | Yes (B-pre-build) | Backfill only | Complete |
| S-6.05 | prism-bin: prism migrate-storage CLI Command | Yes (B-pre-build) | Backfill only | Complete |
| S-6.06 | prism-dtu-common: DTU Common Infrastructure | Yes (B-pre-build) | Backfill only | Complete |
| S-6.07 | prism-dtu-crowdstrike: DTU for CrowdStrike Falcon API | Yes (B-pre-build) | Backfill only | Complete |
| S-6.08 | prism-dtu-claroty: DTU for Claroty xDome API | No | Full sweep | Complete |
| S-6.09 | prism-dtu-cyberint: DTU for Cyberint API | No | Full sweep | Complete |
| S-6.10 | prism-dtu-armis: DTU for Armis Centrix API | No | Full sweep | Complete |
| S-6.11 | prism-dtu-slack: DTU for Slack Webhook API | No | Full sweep | Complete |
| S-6.12 | prism-dtu-pagerduty: DTU for PagerDuty Events API v2 | No | Full sweep | Complete |
| S-6.13 | prism-dtu-jira: DTU for Jira REST API v3 | No | Full sweep | Complete |

---

## Remediation Detail

### S-6.04 — prism-bin: prism credential CLI Subcommand Group

**Remediated by:** prior agent (B-pre-build sweep, 2026-04-19)

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array (prd.md + BC paths)
- Added `points` frontmatter field (derived from `estimated_days: 3` → 5 points)
- Added `blocks` frontmatter field (reverse-scan result)
- Added `assumption_validations: []` frontmatter field
- Added `risk_mitigations: []` frontmatter field
- Added `anchor_bcs`, `anchor_capabilities`, `anchor_subsystem` frontmatter fields
- Added `## Edge Cases` section
- Added `## Architecture Compliance Rules` section
- Normalized `## Dev Notes` heading (was `## Notes`)
- Normalized `## Library & Framework Requirements` heading

**Verified current state:** `level: "L4"`, `inputs` array present, `points: 5` set.

---

### S-6.05 — prism-bin: prism migrate-storage CLI Command

**Remediated by:** prior agent (B-pre-build sweep, 2026-04-19)

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array (prd.md + BC paths)
- Added `points` frontmatter field (derived from `estimated_days: 2` → 3 points)
- Added `blocks` frontmatter field
- Added `assumption_validations: []`, `risk_mitigations: []` frontmatter fields
- Added `anchor_bcs`, `anchor_capabilities`, `anchor_subsystem` frontmatter fields
- Added `## Edge Cases` section
- Added `## Architecture Compliance Rules` section
- Normalized section headings

**Verified current state:** `level: "L4"`, `points: 3` set.

---

### S-6.06 — prism-dtu-common: DTU Common Infrastructure

**Remediated by:** prior agent (B-pre-build sweep, 2026-04-19)

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array (prd.md + dtu-assessment.md + dtu-strategy.md)
- Added `points: 8` frontmatter field (from `estimated_days: 4`)
- Added `blocks` frontmatter field (all downstream DTU stories: S-6.07 through S-6.19)
- Added `assumption_validations: []`, `risk_mitigations: []` frontmatter fields
- Added `anchor_bcs: []`, `anchor_capabilities: []`, `anchor_subsystem: null`
- Added `## Edge Cases` section
- Added `## Architecture Compliance Rules` section (cross-cutting DTU rules)
- Normalized section headings

**Verified current state:** `level: "L4"`, `points: 8` set.

---

### S-6.07 — prism-dtu-crowdstrike: DTU for CrowdStrike Falcon API

**Remediated by:** prior agent (B-pre-build sweep, 2026-04-19) — the exemplar for Wave 7 pattern.

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array (prd.md + dtu-assessment.md + dtu-strategy.md)
- Added `points: 8` frontmatter field (from `estimated_days: 5`)
- Added `blocks: [S-3.06, S-3.07]` frontmatter field
- Added `assumption_validations: []`, `risk_mitigations: [R-DTU-003]` frontmatter fields
- Added `anchor_bcs: []`, `anchor_capabilities: []`, `anchor_subsystem: SS-01`
- `## Edge Cases` section was already present (6 edge cases)
- `## Architecture Compliance Rules` was already present (CrowdStrike-specific)
- `## Dev Notes` heading already normalized
- `## Library & Framework Requirements` heading already normalized

**Verified current state:** All template-compliance fields present. Serves as the Wave 7 exemplar.

---

### S-6.08 — prism-dtu-claroty: DTU for Claroty xDome API

**Remediated by:** this sweep (2026-04-20)

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array: prd.md, dtu-assessment.md, dtu-strategy.md
- Added `points: 8` frontmatter field (from `estimated_days: 4`)
- `blocks: [S-3.02]` was already present — verified correct
- Added `assumption_validations: []` frontmatter field
- `risk_mitigations: [R-DTU-004]` was already present
- Added `anchor_bcs: []`, `anchor_capabilities: []`, `anchor_subsystem: null`
- `## Edge Cases` was already present (5 cases) — added EC-006 (network timeout simulation)
- Added `## Architecture Compliance Rules` section (Claroty-specific)
- Normalized `## Library and Framework Requirements` → `## Library & Framework Requirements`
- Added `## Dev Notes` section (was absent)
- Added BC file count row to Token Budget table

**Edge cases added:** EC-006 (network timeout via LatencyLayer)

---

### S-6.09 — prism-dtu-cyberint: DTU for Cyberint API

**Remediated by:** this sweep (2026-04-20)

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array: prd.md, dtu-assessment.md, dtu-strategy.md
- Added `points: 5` frontmatter field (from `estimated_days: 3`)
- `blocks: [S-3.02]` was already present — verified correct
- Added `assumption_validations: []` frontmatter field
- `risk_mitigations: [R-DTU-001]` was already present
- Added `anchor_bcs: []`, `anchor_capabilities: []`, `anchor_subsystem: null`
- `## Edge Cases` was already present (5 cases) — added EC-006 (auth failure simulation)
- Added `## Architecture Compliance Rules` section (Cyberint-specific, including CookieRoundtrip rule)
- Normalized `## Library and Framework Requirements` → `## Library & Framework Requirements`
- Added `## Dev Notes` section
- Added BC file count row to Token Budget table

**Edge cases added:** EC-006 (auth failure via `POST /dtu/configure`)

---

### S-6.10 — prism-dtu-armis: DTU for Armis Centrix API

**Remediated by:** this sweep (2026-04-20)

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array: prd.md, dtu-assessment.md, dtu-strategy.md
- Added `points: 5` frontmatter field (from `estimated_days: 3`)
- `blocks: [S-3.02]` was already present — verified correct
- Added `assumption_validations: []` frontmatter field
- `risk_mitigations: [R-DTU-002]` was already present
- Added `anchor_bcs: []`, `anchor_capabilities: []`, `anchor_subsystem: null`
- `## Edge Cases` was already present (5 cases) — added EC-006 (malformed mock response)
- Added `## Architecture Compliance Rules` section (Armis-specific, including 403-not-401 rule and AQL log rule)
- Normalized `## Library and Framework Requirements` → `## Library & Framework Requirements`
- Added `## Dev Notes` section
- Added BC file count row to Token Budget table

**Edge cases added:** EC-006 (malformed response via FailureLayer::MalformedResponse)
**Architecture rule added:** Armis MUST return 403 (not 401) for missing bearer token

---

### S-6.11 — prism-dtu-slack: DTU for Slack Webhook API

**Remediated by:** this sweep (2026-04-20)

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array: prd.md, dtu-assessment.md, dtu-strategy.md
- Added `points: 3` frontmatter field (from `estimated_days: 2`)
- `blocks: [S-4.08, S-5.06]` was already present — verified correct
- Added `assumption_validations: []` frontmatter field
- `risk_mitigations: [R-DTU-006]` was already present
- Added `anchor_bcs: []`, `anchor_capabilities: []`, `anchor_subsystem: null`
- `## Edge Cases` was already present (4 cases) — added EC-005 (reset during capture)
- Added `## Architecture Compliance Rules` section (Slack-specific, including deterministic message_ts rule)
- `## Library and Framework Requirements` heading was already correct (no `and`)
- Added `## Dev Notes` section
- Added BC file count row to Token Budget table

**Edge cases added:** EC-005 (POST /dtu/reset during active payload capture)
**Architecture rule added:** message_ts MUST be stable `"1234567890.123456"`, not random

---

### S-6.12 — prism-dtu-pagerduty: DTU for PagerDuty Events API v2

**Remediated by:** this sweep (2026-04-20)

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array: prd.md, dtu-assessment.md, dtu-strategy.md
- Added `points: 8` frontmatter field (from `estimated_days: 4`)
- `blocks: [S-4.08, S-5.06]` was already present — verified correct
- Added `assumption_validations: []` frontmatter field
- `risk_mitigations: [R-DTU-007]` was already present
- Added `anchor_bcs: []`, `anchor_capabilities: []`, `anchor_subsystem: null`
- `## Edge Cases` was already present (4 cases) — added EC-005 (auth failure simulation)
- Added `## Architecture Compliance Rules` section (PagerDuty-specific, including severity case-sensitivity rule and dedup idempotency rule)
- `## Library and Framework Requirements` heading was already correct
- Added `## Dev Notes` section
- Added BC file count row to Token Budget table

**Edge cases added:** EC-005 (auth failure via `POST /dtu/configure {"auth_mode": "reject"}`)
**Architecture rule added:** severity validation MUST be case-sensitive (no `"CRITICAL"`)

---

### S-6.13 — prism-dtu-jira: DTU for Jira REST API v3

**Remediated by:** this sweep (2026-04-20)

**Changes applied:**
- Added `level: "L4"` frontmatter field
- Added `inputs` frontmatter array: prd.md, dtu-assessment.md, dtu-strategy.md
- Added `points: 8` frontmatter field (from `estimated_days: 5`)
- `blocks: [S-4.08, S-5.06]` was already present — verified correct
- Added `assumption_validations: []` frontmatter field
- `risk_mitigations: [R-DTU-008]` was already present
- Added `anchor_bcs: []`, `anchor_capabilities: []`, `anchor_subsystem: null`
- `## Edge Cases` was already present (5 cases) — added EC-006 (rate limit during issue creation)
- Added `## Architecture Compliance Rules` section (Jira-specific, including status machine enforcement rule and counter reset rule)
- `## Library and Framework Requirements` heading was already correct
- Added `## Dev Notes` section
- Added BC file count row to Token Budget table
- `## Risk Mitigations` heading preserved (non-standard but retained from v1.2; contains full prose risk block)

**Edge cases added:** EC-006 (rate limit during issue creation — state not persisted)
**Architecture rule added:** issue number counter MUST reset to 1000 on `reset()` for predictable test keys

---

## Points Summary

| Story | Title | Points | Epic |
|-------|-------|--------|------|
| S-6.04 | prism credential CLI | 5 | E-6 |
| S-6.05 | prism migrate-storage CLI | 3 | E-6 |
| S-6.06 | prism-dtu-common | 8 | E-6 |
| S-6.07 | prism-dtu-crowdstrike | 8 | E-6 |
| S-6.08 | prism-dtu-claroty | 8 | E-6 |
| S-6.09 | prism-dtu-cyberint | 5 | E-6 |
| S-6.10 | prism-dtu-armis | 5 | E-6 |
| S-6.11 | prism-dtu-slack | 3 | E-6 |
| S-6.12 | prism-dtu-pagerduty | 8 | E-6 |
| S-6.13 | prism-dtu-jira | 8 | E-6 |
| **Wave 7 Total** | | **61** | |

---

## Template-Compliance Checklist (per story)

| Check | S-6.04 | S-6.05 | S-6.06 | S-6.07 | S-6.08 | S-6.09 | S-6.10 | S-6.11 | S-6.12 | S-6.13 |
|-------|--------|--------|--------|--------|--------|--------|--------|--------|--------|--------|
| `level` field | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `inputs` array | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `points` field | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `blocks` field | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `assumption_validations` | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `risk_mitigations` | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `anchor_bcs` | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `anchor_capabilities` | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `anchor_subsystem` | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `## Edge Cases` | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `## Architecture Compliance Rules` | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `## Dev Notes` (not `## Notes`) | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| `## Library & Framework Requirements` | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| Changelog row for this sweep | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |

All 10 Wave 7 stories: COMPLIANT.

---

## Notes

- DTU stories (S-6.06 through S-6.13) carry `anchor_bcs: []`, `anchor_capabilities: []`,
  `anchor_subsystem: null` — correct because DTU stories have no product-level BCs.
- S-6.09 (Cyberint) `level: "L4"` is set in frontmatter even though the story describes
  L2 fidelity. This is intentional: `level` in frontmatter refers to template compliance
  level (story quality), not DTU fidelity level. The DTU fidelity level (L2) is described
  in the title and narrative.
- S-6.13 (Jira) retains `## Risk Mitigations` as a non-standard section heading (prose
  block with R-DTU-008 detail). This was introduced in v1.0 and is preserved for
  traceability; `risk_mitigations: [R-DTU-008]` in frontmatter provides the machine-readable
  reference.
