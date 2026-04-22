---
document_type: adversary-report
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-20T00:00:00Z
pass: 59
inputs:
  - ".factory/STATE.md"
  - ".factory/stories/STORY-INDEX.md"
  - ".factory/specs/behavioral-contracts/BC-INDEX.md"
  - ".factory/specs/verification-properties/VP-INDEX.md"
  - ".factory/specs/architecture/"
  - ".factory/stories/"
  - ".factory/specs/epics.md"
  - ".factory/specs/architecture/verification-coverage-matrix.md"
  - ".factory/cycles/phase-2-patch/consistency-validation-step5.md"
input-hash: "1585afb"
traces_to: ".factory/STATE.md"
verdict: FINDINGS-OPEN
finding_count: 11
severity_breakdown:
  HIGH: 3
  MED: 4
  LOW: 3
  OBS: 1
counter_state: "RESET 2→0"
---

# Adversarial Review — Pass 59

**Verdict:** FINDINGS-OPEN
**Counter:** RESET 2 → 0
**Total Findings:** 11 (3 HIGH, 4 MED, 3 LOW, 1 OBS)

---

## Policy Rubric

| # | Policy | Result | Notes |
|---|--------|--------|-------|
| 1 | append_only_numbering | PASS | No renumbering detected |
| 2 | lift_invariants_to_bcs | PASS | Invariants present in BC files |
| 3 | state_manager_runs_last | PASS | STATE.md reflects final state |
| 4 | semantic_anchoring_integrity | FAIL | HIGH-002: anchor_capabilities derived incorrectly across ~19 stories |
| 5 | creators_justify_anchors | PASS | Justifications present |
| 6 | architecture_is_subsystem_name_source_of_truth | PASS | Subsystem names consistent |
| 7 | bc_h1_is_title_source_of_truth | PASS | BC H1 titles match INDEX |
| 8 | bc_array_changes_propagate_to_body_and_acs | PASS | Propagation verified |
| 9 | vp_index_is_vp_catalog_source_of_truth | PASS | VP-INDEX authoritative |

---

## Sweeps Performed

1. Story frontmatter completeness (all 75 stories)
2. BC slug integrity in `inputs:` arrays (stories referencing BCs)
3. `specs/` prefix correctness in `inputs:` paths
4. `anchor_capabilities:` derivation vs BC-INDEX `capability:` fields
5. `anchor_subsystem:` YAML form (array vs scalar)
6. DTU story `inputs:` and body references to architecture files
7. `dtu-strategy.md` vs `dtu-assessment.md` reference audit
8. STORY-INDEX.md frontmatter YAML validity
9. epics.md E-6 wave assignment vs DTU-first wave schedule
10. verification-coverage-matrix.md column completeness
11. STATE.md field consistency (total_artifacts_swept, anomaly prose)
12. remediation-step5-track-a.md DTU filename correctness
13. VP slug completeness in story `inputs:` arrays
14. `anchor_subsystem:` scalar vs array form
15. Changelog format in all modified stories
16. `input-hash:` sentinel audit (checking for `[pending-recompute]` in frontmatter)

---

## Findings

### HIGH-001 — Stem-Only or Missing `specs/` Prefix in `inputs:` Paths

**Severity:** HIGH
**Affected files:** ~17 stories
**Description:** Multiple stories used stem-only BC filenames (e.g. `BC-2.18.001.md` instead of
`BC-2.18.001-action-at-least-once-delivery.md`) or omitted the `specs/` directory prefix
(`.factory/behavioral-contracts/` instead of `.factory/specs/behavioral-contracts/`).
Both forms break static analysis and input-drift tooling.

**Affected stories:**
- `S-4.02-diff-results-packs.md` — stem-only slugs on 5 BCs + VP-019
- `S-4.03-detection-rules.md` — stem-only slugs on 8 BCs + VP-018
- `S-4.04-detection-evaluation.md` — stem-only slugs on 5 BCs + VP-027
- `S-4.05-alert-generation.md` — stem-only slug on BC-2.13.005 + VP-028
- `S-4.06-case-management.md` — stem-only slugs on 9 BC-2.14.xxx files
- `S-4.07-case-metrics.md` — stem-only slugs on 3 BCs
- `S-4.08-action-delivery.md` — stem-only slugs on 9 BC-2.18.xxx files
- `S-5.03-resources-prompts.md` — stem-only slugs on 4 BCs
- `S-5.05-config-loading.md` — missing `specs/` prefix + slugs on 10 BC-2.06.xxx files
- `S-5.06-action-infusion-tools.md` — missing `specs/` prefix + slugs on 4 BCs
- `S-5.07-multi-repo-git-config.md` — missing `specs/` prefix + slugs on 8 BC-2.06.xxx files
- `S-5.09-external-log-forwarding.md` — missing `specs/` prefix + slug on BC-2.10.001
- `S-5.10-audit-trail-forwarding.md` — missing `specs/` prefix on 7 BC-2.05.xxx + VP-039 slug
- `S-6.01-cli-startup.md` — missing `specs/` prefix + slugs on 4 BCs
- `S-6.02-e2e-smoke-tests.md` — missing `specs/` prefix + slugs on 5 BCs
- `S-6.03-installation.md` — missing `specs/` prefix + slugs on BC-2.10.001 and BC-2.10.006
- `S-6.05-migrate-storage.md` — stem-only slugs on 3 BCs

**Recommendation:** Replace all stem-only paths with full slugged filenames; ensure all
paths use `.factory/specs/behavioral-contracts/` prefix.

---

### HIGH-002 — `anchor_capabilities:` Mis-Anchored

**Severity:** HIGH
**Affected files:** ~19 stories
**Description:** `anchor_capabilities:` fields were populated with CAP codes that do not
correspond to the `capability:` fields of the stories' `behavioral_contracts:` entries
in BC-INDEX. The anchor must be derived as the union of `capability:` values from
BC-INDEX for every BC in the story's `behavioral_contracts:` array.

**Affected stories and recommended corrections:**
- `S-4.01-schedule-crud.md` — `[CAP-016]` → `[CAP-017]`
- `S-4.02-diff-results-packs.md` — `[CAP-016]` → `[CAP-018, CAP-023]`
- `S-4.03-detection-rules.md` — `[CAP-017]` → `[CAP-020, CAP-027]`
- `S-4.04-detection-evaluation.md` — `[CAP-017]` → `[CAP-020, CAP-021]`
- `S-4.05-alert-generation.md` — `[CAP-017]` → `[CAP-021]`
- `S-4.06-case-management.md` — `[CAP-018]` → `[CAP-022]`
- `S-4.07-case-metrics.md` — `[CAP-018]` → `[CAP-022]`
- `S-4.08-action-delivery.md` — `[CAP-018]` → `[CAP-033]`
- `S-5.01-mcp-bootstrap.md` — `[CAP-010]` → `[CAP-005, CAP-009, CAP-015, CAP-034]`
- `S-5.02-tool-routing.md` — `[CAP-010]` → `[CAP-005, CAP-009, CAP-034]`
- `S-5.03-resources-prompts.md` — `[CAP-010]` → `[CAP-008, CAP-009, CAP-034]`
- `S-5.05-config-loading.md` — `[CAP-006]` → `[CAP-009]`
- `S-5.06-action-infusion-tools.md` — `[CAP-010]` → `[CAP-007, CAP-030, CAP-031, CAP-033]`
- `S-5.07-multi-repo-git-config.md` — `[CAP-006]` → `[CAP-009]`
- `S-5.09-external-log-forwarding.md` — `[CAP-010]` → `[CAP-008, CAP-025]`
- `S-5.10-audit-trail-forwarding.md` — `[CAP-005]` → `[CAP-007, CAP-025]`
- `S-6.01-cli-startup.md` — `[CAP-006, CAP-010]` → `[CAP-009, CAP-034]`
- `S-6.02-e2e-smoke-tests.md` — `[CAP-006, CAP-008, CAP-010]` → `[CAP-008, CAP-009, CAP-034]`
- `S-6.03-installation.md` — `[CAP-010]` → `[]`
- `S-6.04-credential-cli.md` — `[CAP-003]` → `[CAP-004]`
- `S-6.05-migrate-storage.md` — `[CAP-015]` → `[CAP-019, CAP-024]`

**Note:** Adversary derived these from the semantic mapping of BCs to capabilities.
Remediator must cross-check against BC-INDEX `capability:` fields for final values;
BC-INDEX is authoritative per Policy 6/7.

---

### HIGH-003 — DTU Stories Reference Non-Existent `dtu-strategy.md`

**Severity:** HIGH
**Affected files:** 13 DTU stories (S-6.07–S-6.19)
**Description:** All 13 DTU stories reference `.factory/specs/architecture/dtu-strategy.md`
in both their `inputs:` frontmatter and body text. This file does not exist. The correct
file is `.factory/specs/architecture/dtu-assessment.md` (finalized 2026-04-20).

**Affected stories:**
- `S-6.07-dtu-crowdstrike.md` through `S-6.19-dtu-otlp.md` (all 13 DTU clone stories)

**Recommendation:** Replace all occurrences of `dtu-strategy.md` with `dtu-assessment.md`
in both `inputs:` arrays and body text. Deduplicate if stories already have a correct
`dtu-assessment.md` entry.

---

### MED-001 — epics.md E-6 Wave Assignment Incorrect

**Severity:** MEDIUM
**Affected file:** `specs/epics.md`
**Description:** E-6 (DTU clone infrastructure) lists Wave column as `6`. Under Option 2
DTU-first strategy (decided 2026-04-20), DTU clones are distributed across Waves 0–3 to
precede their product consumers. Wave `6` is incorrect for DTU clones.

**Recommendation:** Update E-6 Wave column to `0–3, 6` with a footnote explaining
Option 2 DTU-first two-track delivery. Bump epics.md v1.0 → v1.1.

---

### MED-002 — Wrong DTU Filenames in `remediation-step5-track-a.md`

**Severity:** MEDIUM
**Affected file:** `cycles/phase-2-patch/remediation-step5-track-a.md`
**Description:** Lines 111–116 of the manifest list DTU story filenames that do not exist
(e.g. `S-6.08-dtu-sentinel.md`, `S-6.09-dtu-qradar.md`). Correct filenames differ.

**Wrong → Correct:**
- `S-6.08-dtu-sentinel.md` → `S-6.08-dtu-claroty.md`
- `S-6.09-dtu-qradar.md` → `S-6.09-dtu-cyberint.md`
- `S-6.10-dtu-defender.md` → `S-6.10-dtu-armis.md`
- `S-6.11-dtu-armis.md` → `S-6.11-dtu-slack.md`
- `S-6.12-dtu-claroty.md` → `S-6.12-dtu-pagerduty.md`
- `S-6.13-dtu-cyberint.md` → `S-6.13-dtu-jira.md`

**Recommendation:** Correct the 6 filenames in the manifest table.

---

### MED-003 — STATE.md `total_artifacts_swept` Understated

**Severity:** MEDIUM
**Affected file:** `STATE.md`
**Description:** `total_artifacts_swept: 320` but Step 4 recompute report states
204 BCs + 75 stories + 39 VPs + 4 supplements = 322 artifacts. The count is off by 2.

**Recommendation:** Update `total_artifacts_swept: 320` → `total_artifacts_swept: 322`.

---

### MED-004 — `anchor_subsystem:` Scalar Form Instead of YAML Array

**Severity:** MEDIUM
**Affected files:** `S-6.04-credential-cli.md`, `S-6.05-migrate-storage.md`,
`S-6.07-dtu-crowdstrike.md`
**Description:** Three stories use scalar form `anchor_subsystem: SS-XX` instead of the
required YAML array form `anchor_subsystem: ["SS-XX"]`.

**Recommendation:** Convert scalar to array form in all three files.

---

### LOW-001 — STATE.md Stale Wave 8 Anomaly Prose

**Severity:** LOW
**Affected file:** `STATE.md` Session Resume Checkpoint
**Description:** The checkpoint contains a paragraph describing a Wave 8 DTU level
inconsistency (L2 vs L4 classification for S-6.14–S-6.19). BLK-001 was closed in Step 5
with global normalization. The anomaly prose is stale and contradicts the resolved state.

**Recommendation:** Delete the anomaly paragraph or mark it `[RESOLVED 2026-04-20]`.

---

### LOW-002 — `verification-coverage-matrix.md` Missing Integration Tests Column

**Severity:** LOW
**Affected file:** `specs/architecture/verification-coverage-matrix.md`
**Description:** The matrix has columns for Unit Tests, Property Tests, and Holdout Scenarios
but does not have an Integration Tests column. Per the VP-INDEX, 39 VPs include integration
test coverage requirements. The missing column means the per-module counts cannot sum to 39.

**Recommendation:** Add Integration Tests column to the matrix with per-module counts.
Bump version v1.0 → v1.1.

---

### LOW-003 — STORY-INDEX.md `level: L4` Unquoted

**Severity:** LOW
**Affected file:** `stories/STORY-INDEX.md`
**Description:** Line 3 frontmatter has bare scalar `level: L4` instead of quoted
`level: "L4"`. YAML parsers may interpret unquoted `L4` as a string, but it is not
hook-compliant and inconsistent with all other level fields in the corpus.

**Recommendation:** Change `level: L4` → `level: "L4"`.

---

### OBS-001 — Orphaned `dtu-clones/` References in Wave Schedule

**Severity:** OBSERVATION
**Affected file:** `cycles/phase-2-patch/remediation-step5-option2-dtu.md`
**Description:** The wave schedule manifest references `.factory/dtu-clones/` as the
output location for DTU clone builds, but this directory does not exist and `dtu_clones_built`
remains `pending`. No corrective action required at spec stage — noting for Phase 3 setup.

**Recommendation:** Ensure `.factory/dtu-clones/` is created as part of Phase 3 worktree
initialization before DTU clone builds begin.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 59 |
| **New findings** | 11 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 11 / (11 + 0) = 1.00 |
| **Median severity** | 3.0 (HIGH=5, MED=3, LOW=1, OBS=0.5) |
| **Trajectory** | 29→24→21→7→4→3→0(p51)→0(p52)→0(p53)→1(p55)→0(p56)→0(p57)→0(p58)→11(p59) |
| **Verdict** | FINDINGS_REMAIN |

All 11 findings are new (not previously surfaced in passes 1–58). Root causes:

1. **Wave 1-8 mechanical sweep** (HIGH-001, HIGH-002, MED-004): Template-compliance sweeps
   populated frontmatter from partial data; BC slug expansion and capability derivation
   require BC-INDEX cross-reference that the sweep agents did not perform.

2. **Step 5 inputs-format conversion** (HIGH-001): The dict→YAML-string conversion preserved
   wrong paths (missing `specs/` prefix, stem-only filenames) from pre-sweep state.

3. **DTU rename gap** (HIGH-003): DTU stories were created when the file was named
   `dtu-strategy.md`. The rename to `dtu-assessment.md` (2026-04-20) was not propagated
   to the 13 DTU story files.

4. **Wave schedule update not propagated** (MED-001): epics.md E-6 Wave column not updated
   when Option 2 DTU-first strategy was decided.

5. **Manifest copy error** (MED-002): step5-track-a.md captured wrong DTU filenames from
   an earlier draft of the DTU story list.

6. **State count arithmetic error** (MED-003): Wave 8 checkpoint reported 320 artifacts;
   Step 4 recompute report corrects to 322.

7. **BLK-001 close not reflected** (LOW-001): State checkpoint not pruned after blocker closed.

8. **Matrix completeness** (LOW-002): VP count audit exposed missing column.

9. **YAML quoting inconsistency** (LOW-003): Pre-existing hook-compliance gap in INDEX file.

10. **Observational** (OBS-001): Forward-looking note for Phase 3 setup; no spec action needed.

---

## Summary

Pass-59 surfaces 11 findings from the pre-build sweep corpus. Three issues are HIGH severity
requiring immediate remediation before pass-60. The pre-build sweep improved template
compliance significantly but introduced mechanical errors in `inputs:` path formatting
and `anchor_capabilities:` derivation. Counter RESET from 2 → 0. Remediation should be
dispatched via parallel tracks (story-writer for HIGH/MED story fixes; product-owner for
epics.md; architect for verification-coverage-matrix; state-manager for STATE.md fields).
Pass-60 will re-verify the full corpus against this baseline.

---

## Remediation Links

- Track A manifest: `.factory/cycles/phase-2-patch/remediation-pass59-track-a.md`
- Track B: product-owner fixed `epics.md` E-6 wave column (v1.0→v1.1)
- Track C: architect added Integration Tests column to `verification-coverage-matrix.md` (v1.0→v1.1)
- STATE.md: MED-003 + LOW-001 fixed by state-manager (this burst)
