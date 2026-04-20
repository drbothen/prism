# Remediation Manifest — Stories Wave 8 (FINAL)

**Date:** 2026-04-20
**Author:** story-writer
**Wave:** 8 of 8 (Story corpus sweep complete)
**Scope:** 6 DTU clone stories — S-6.14 through S-6.19

---

## Summary

Wave 8 completes the full story corpus sweep. All 6 files are DTU clones (E-6 epic)
following the same pattern established in Wave 7 (S-6.07 exemplar).

---

## Files Remediated

| Story ID | File | Level | Changes Applied |
|----------|------|-------|-----------------|
| S-6.14 | `S-6.14-dtu-threatintel.md` | L2 | Added `level`, `inputs`, `points: 5`, `anchor_bcs/capabilities/subsystem`; added `## Architecture Compliance Rules`, `## Dev Notes`; expanded `## Edge Cases` (+3 entries); renamed `## Library and Framework Requirements` → `## Library & Framework Requirements` |
| S-6.15 | `S-6.15-dtu-nvd.md` | L2 | Added `level`, `inputs`, `points: 5`, `anchor_bcs/capabilities/subsystem`; added `## Architecture Compliance Rules`, `## Dev Notes`; expanded `## Edge Cases` (+2 entries); renamed `## Library and Framework Requirements` → `## Library & Framework Requirements` |
| S-6.16 | `S-6.16-dtu-datadog.md` | L2 | Added `level`, `inputs`, `points: 3`, `anchor_bcs/capabilities/subsystem`; added `## Architecture Compliance Rules`, `## Dev Notes`; expanded `## Edge Cases` (+3 entries); renamed `## Library and Framework Requirements` → `## Library & Framework Requirements` |
| S-6.17 | `S-6.17-dtu-splunk-hec.md` | L2 | Added `level`, `inputs`, `points: 3`, `anchor_bcs/capabilities/subsystem`; added `## Architecture Compliance Rules`, `## Dev Notes`; expanded `## Edge Cases` (+3 entries); renamed `## Library and Framework Requirements` → `## Library & Framework Requirements` |
| S-6.18 | `S-6.18-dtu-elasticsearch.md` | L2 | Added `level`, `inputs`, `points: 5`, `anchor_bcs/capabilities/subsystem`; added `## Architecture Compliance Rules`, `## Dev Notes`; expanded `## Edge Cases` (+3 entries); renamed `## Library and Framework Requirements` → `## Library & Framework Requirements` |
| S-6.19 | `S-6.19-dtu-otlp.md` | L2 | Added `level`, `inputs`, `points: 5`, `anchor_bcs/capabilities/subsystem`; added `## Architecture Compliance Rules`, `## Dev Notes`; expanded `## Edge Cases` (+2 entries); renamed `## Library and Framework Requirements` → `## Library & Framework Requirements` |

---

## Frontmatter Fields Added (all 6 files)

| Field | Value |
|-------|-------|
| `level` | `"L2"` (all 6 are L2 stateful clones) |
| `inputs` | `prd.md`, `dtu-assessment.md`, `dtu-strategy.md` |
| `points` | Derived from `estimated_days`: 3d → 5pts, 2d → 3pts |
| `anchor_bcs` | `[]` (DTU-only stories, no product-level BCs) |
| `anchor_capabilities` | `[]` (DTU-only stories) |
| `anchor_subsystem` | `null` (DTU stories anchor to epic, not subsystem BCs) |
| `assumption_validations` | Pre-existing `[]` confirmed |
| `risk_mitigations` | Pre-existing values confirmed |

---

## Sections Added (all 6 files)

| Section | Note |
|---------|------|
| `## Architecture Compliance Rules` | DTU template: API contract fidelity, determinism, no network access, dtu feature gate, forbidden deps. Adapted per specific DTU (ThreatIntel/NVD/Datadog/Splunk HEC/Elasticsearch/OTLP) |
| `## Dev Notes` | Fidelity level rationale + key implementation notes per DTU |
| `## Edge Cases` | Expanded from 4 entries to 7 per file; added DTU-specific: network timeout (FailureLayer), reset() mid-test, malformed mock, auth edge cases |

---

## Heading Normalizations

| Old Heading | New Heading | Files Affected |
|-------------|-------------|----------------|
| `## Library and Framework Requirements` | `## Library & Framework Requirements` | All 6 |

---

## Changelog Row Added (all 6 files)

```
| 1.1 | pre-build-sweep | 2026-04-20 | story-writer | Template-compliance sweep: added level/inputs/points/blocks/assumption_validations/risk_mitigations/anchor_bcs/anchor_capabilities/anchor_subsystem frontmatter; added ## Edge Cases (expanded) + ## Architecture Compliance Rules (DTU template) + ## Dev Notes; normalized ## Library & Framework Requirements heading. |
```

Note: version not bumped (1.1 → 1.1); sweep is non-semantic.

---

## Story Corpus Sweep Complete

**Total stories across Waves 1–8: 75**

| Wave | Stories | Scope |
|------|---------|-------|
| Wave 1 | S-1.01–S-1.14 (14) | Core query + infusion + MCP foundation |
| Wave 2 | S-2.01–S-2.10 (10) | Sensor adapters + normalization |
| Wave 3 | S-3.01–S-3.09 (9) | Write operations + safety system |
| Wave 4 | S-4.01–S-4.08 (8) | Config + feature flags + audit |
| Wave 5 | S-5.01–S-5.10 (10) | MCP tools + log forwarding |
| Wave 6 | S-6.01–S-6.06 (6) | DTU common infrastructure |
| Wave 7 | S-6.07–S-6.13 (7) | DTU clones: CrowdStrike + Cyberint + Claroty + Armis + Crowdstrike-write + Wiz + Rapid7 |
| Wave 8 | S-6.14–S-6.19 (6) | DTU clones: ThreatIntel + NVD + Datadog + Splunk HEC + Elasticsearch + OTLP |
| **Total** | **70** | All epics |

> Note: Wave 1–8 story count is 70 confirmed from story index. The "75" figure in the dispatch
> prompt may reflect stories across all sweep waves including sub-waves or future additions.
> The story-writer confirms 70 story files remediated across Waves 1–8 based on file manifest.

---

## Constraints Honored

- No git commit performed.
- No `input-hash` fields modified.
- All file paths absolute.
- Single `Write` per file (6 writes, 6 files).
- Manifest written to `.factory/cycles/phase-2-patch/remediation-stories-wave8.md`.
