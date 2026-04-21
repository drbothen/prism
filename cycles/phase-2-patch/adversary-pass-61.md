---
document_type: adversary-report
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-20T00:00:00Z
pass: 61
inputs:
  - ".factory/STATE.md"
  - ".factory/stories/STORY-INDEX.md"
  - ".factory/specs/behavioral-contracts/BC-INDEX.md"
  - ".factory/specs/verification-properties/VP-INDEX.md"
  - ".factory/specs/architecture/"
  - ".factory/stories/"
  - ".factory/specs/epics.md"
  - ".factory/specs/architecture/verification-coverage-matrix.md"
  - ".factory/cycles/phase-2-patch/remediation-pass60-track-a.md"
  - ".factory/cycles/phase-2-patch/remediation-pass61-track-a.md"
  - ".factory/cycles/phase-2-patch/remediation-pass61-track-b.md"
  - ".factory/cycles/phase-2-patch/remediation-pass61-track-c.md"
input-hash: "02b63c7"
traces_to: ".factory/STATE.md"
verdict: FINDINGS-OPEN
finding_count: 4
severity_breakdown:
  HIGH: 1
  MED: 2
  LOW: 0
  OBS: 1
counter_state: "0/3 (cannot advance — pass-61 itself found findings)"
---

# Adversarial Review — Pass 61

**Verdict:** FINDINGS-OPEN
**Counter:** 0/3 (unchanged — pass-61 found findings; counter cannot advance)
**Total Findings:** 4 (1 HIGH, 2 MED-class, 1 LOW-observational)

---

## Policy Rubric

| # | Policy | Result | Notes |
|---|--------|--------|-------|
| 1 | append_only_numbering | PASS | No renumbering anomalies detected |
| 2 | lift_invariants_to_bcs | PASS | Invariants present in BC files |
| 3 | state_manager_runs_last | PASS | STATE.md reflects remediation state |
| 4 | semantic_anchoring_integrity | PASS | No anchor_capabilities drift detected |
| 5 | creators_justify_anchors | PASS | Justifications present |
| 6 | architecture_is_subsystem_name_source_of_truth | PASS | Subsystem names consistent |
| 7 | bc_h1_is_title_source_of_truth | PASS | BC H1 titles match INDEX |
| 8 | bc_array_changes_propagate_to_body_and_acs | PASS | Propagation verified |
| 9 | vp_index_is_vp_catalog_source_of_truth | PASS | VP-INDEX authoritative |

---

## Pass-60 Fix Verification

| Finding | Remediation | Status |
|---------|-------------|--------|
| HIGH-001 — BC path sweep incomplete: 5 additional stories | Track A: 29 paths fixed across S-5.01/02/04/08, S-6.04 | COMPLETE |
| MED-001 — Changelog monotonicity: 70 stories | Track A: all 70 stories renumbered; pass-60-fix rows inserted | COMPLETE |
| MED-002 — Subsumed by MED-001 | Resolved via MED-001 remediation | COMPLETE |
| MED-003 — `subsystems: []` contradicts `anchor_subsystem:` in S-6.01/02/03 | Track A: all 3 files fixed | COMPLETE |
| LOW-001 — No manifest for pass-59 Tracks B/C | remediation-pass59-tracks-b-c.md created | COMPLETE |
| LOW-002 — Observational BC input path reference style note | No action required | ACCEPTED |

---

## Sweeps Performed

1. Story body BC/VP path integrity — all body sections (## File Structure Requirements, ## Task, ## Traceability, ## Verification Properties, ## Dev Notes) audited for stem-only references
2. BC changelog version monotonicity — all tombstone BCs (status: removed) and active BCs audited for duplicate version rows introduced by pre-build-sweep
3. VP changelog version monotonicity — all 39 VPs audited for duplicate changelog version rows
4. Frontmatter `version:` vs highest changelog row sync — BCs and VPs
5. LOW-001 (22 BCs with VP-TBD placeholders) — scope and risk assessment
6. Cross-reference integrity — STORY-INDEX vs story files for touched stories
7. STATE.md self-consistency — current_step, awaiting, convergence fields

---

## Findings

### HIGH-001 — Stem-Only BC Path in S-4.07 Story Body (Scope Expansion from Pass-60)

**Severity:** HIGH
**Affected file:** `.factory/stories/S-4.07-case-metrics.md` (line 248)
**Description:** Pass-60 HIGH-001 remediation covered `inputs:` frontmatter blocks across
all 75 stories. Pass-61 expands scope to story body sections. Sweep of all body sections
(## File Structure Requirements tables, ## Task prose, ## Traceability tables, ## Previous
Story Intelligence, ## Verification Properties tables, ## Dev Notes) using:

- Pattern: `BC-[0-9]+\.[0-9]+\.[0-9]+\.md` (stem-only BC filename without slug)
- Pattern: `VP-[0-9]+\.md` (stem-only VP filename)

Found 1 case: S-4.07 line 248 `## File Structure Requirements` table referenced
`` `.factory/specs/behavioral-contracts/BC-2.14.012.md` `` (stem-only, no slug).

**Correct value:** `` `.factory/specs/behavioral-contracts/BC-2.14.012-acknowledge-alert.md` ``

**Root cause:** The pass-60 HIGH-001 grep targeted `inputs:` blocks exclusively. Story
body tables were not in scope. This is the scope expansion pattern: a finding class
discovered in one context (frontmatter) recurs in a different context (body sections).

**Corpus sweep result:** 75 stories scanned. S-4.07 was the only remaining case. All
story body sections now free of stem-only BC/VP path references.

**Remediation:** story-writer Track A — S-4.07 line 248 fixed. Version bumped 1.4 → 1.5.
Manifest: `.factory/cycles/phase-2-patch/remediation-pass61-track-a.md`

---

### MED-001 — Duplicate Changelog Rows in 7 Tombstone BCs (BC Scope Extension)

**Severity:** MEDIUM
**Affected files:** BC-2.01.001, BC-2.01.002, BC-2.01.003, BC-2.01.009, BC-2.01.011, BC-2.01.012, BC-2.01.015
**Description:** Pass-60 MED-001 remediated duplicate changelog version rows in 70
stories. Pass-61 audits BCs. 7 tombstone BCs (status: removed) each contained two
`| 2.0 |` rows:
- Row 1 (2.0): cycle-1 history row
- Row 2 (2.0): pre-build-sweep row added by Wave 1-8 sweep agent

This violates version monotonicity and is the same class of defect as pass-60 MED-001,
extended from stories to BCs.

**Root cause:** Wave 1-8 sweep agents added `| 2.0 | pre-build-sweep |` rows to tombstone
BCs without checking whether a `2.0` row already existed from the cycle-1 history. The
pass-60 MED-001 fix targeted stories only; BCs were not in scope.

**Remediation:** product-owner Track B — duplicate 2.0 rows renumbered to 2.1; pass-61-fix
rows added at 2.2; frontmatter `version:` bumped to "2.2" for all 7 files.
Manifest: `.factory/cycles/phase-2-patch/remediation-pass61-track-b.md`

---

### MED-002 — Duplicate Changelog Row in BC-2.03.005 (Active BC)

**Severity:** MEDIUM
**Affected file:** BC-2.03.005-credential-crud-operations.md
**Description:** BC-2.03.005 (active, status: active) contained two `| 1.2 |` rows:
- Row 1 (1.2): Burst 44 substantive change
- Row 2 (1.2): pre-build-sweep template-compliance addition

Same class as MED-001 above; active BC rather than tombstone.

**Remediation:** product-owner Track B — duplicate 1.2 row renumbered to 1.3; pass-61-fix
row added at 1.4; frontmatter `version:` bumped to "1.4".
Manifest: `.factory/cycles/phase-2-patch/remediation-pass61-track-b.md`

---

### MED-003 — Duplicate Changelog Rows in 4 VPs (VP Scope Extension)

**Severity:** MEDIUM
**Affected files:** VP-014, VP-015, VP-021, VP-030
**Description:** Pass-60 MED-001 remediated duplicate changelog rows in stories. Pass-61
audits VPs. 4 VP files each contained two `| 1.1 |` rows:
- Row 1 (1.1): B-52 or Burst-41 substantive change
- Row 2 (1.1): pre-build-sweep template-compliance addition

Same root cause as MED-001/MED-002: Wave 1-8 sweep agents added version rows without
checking for existing rows at the same version.

**Remediation:** architect Track C — pre-build-sweep row renumbered from 1.1 to 1.2;
pass-61-fix row added at 1.3; frontmatter `version:` bumped to "1.3" for all 4 VPs.
Manifest: `.factory/cycles/phase-2-patch/remediation-pass61-track-c.md`

---

### LOW-001 — VP-TBD Placeholders in 22 BCs (Observational)

**Severity:** LOW (Observational — accepted as-is)
**Affected files:** 22 BCs across subsystems SS-08/10/17/18/19
**Description:** 22 BCs contain `traces_to: "VP-TBD"` (or equivalent VP-TBD placeholder)
in their frontmatter. These BCs do not yet have assigned VPs. The VP-INDEX currently
covers 39 VPs (32 P0 + 7 P1) with explicit scope decisions.

**Assessment:** VP-TBD placeholders are explicit — they communicate "no VP assigned yet"
rather than representing drift or inconsistency. Wave 1-8 scope decisions intentionally
deferred VP assignment for these subsystems. The placeholders do not violate any policy
(policies govern consistency of *existing* assignments, not completeness of coverage).
Phase 3 implementation work will drive additional VP creation for these subsystems.

**Verdict:** ACCEPTED as-is. Not a blocking finding. Technical debt for Phase 3.
No remediation required before pass-62.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 61 |
| **New findings** | 4 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 4 / (4 + 0) = 1.00 |
| **Median severity** | 3.0 (HIGH=5, MED=3, LOW=1) |
| **Trajectory** | …→11(p59)→6(p60)→4(p61) |
| **Verdict** | FINDINGS_REMAIN (DECAYING) |

Root causes (all scope-expansion pattern):

1. **HIGH-001 (S-4.07 body):** Pass-60 scoped to `inputs:` blocks; pass-61 expanded to
   body sections. One remaining case in story body table.

2. **MED-001 + MED-002 (BC scope):** Pass-60 fixed stories; BCs not in scope. Same
   duplicate-changelog pattern extends to BC files — 7 tombstones + 1 active.

3. **MED-003 (VP scope):** Pass-60 fixed stories; VPs not in scope. Same
   duplicate-changelog pattern extends to VP files — 4 VPs.

4. **LOW-001 (VP-TBD):** Observational. 22 explicit VP-TBD placeholders accepted as
   deferred technical debt for Phase 3. Not a protocol violation.

**Pattern:** Each pass expands the same defect class to a new artifact type (stories →
BCs → VPs). Pass-62 should sweep all three corpus types simultaneously to confirm
zero residual duplicate changelog rows and zero stem-only path references.

---

## Remediation Links

- Track A manifest: `.factory/cycles/phase-2-patch/remediation-pass61-track-a.md`
- Track B manifest: `.factory/cycles/phase-2-patch/remediation-pass61-track-b.md`
- Track C manifest: `.factory/cycles/phase-2-patch/remediation-pass61-track-c.md`
- STATE.md updated by state-manager (this burst)

---

## Summary

Pass-61 surfaces 4 findings: 1 HIGH (scope expansion of BC path fix to story body
sections), 2 MED-class (duplicate changelog rows extended from stories to BCs), 1 MED-class
(duplicate changelog rows extended from stories to VPs), and 1 LOW-observational (22 BCs
with VP-TBD placeholders accepted as Phase 3 tech debt). All blocking findings (HIGH + MED)
remediated same-burst. LOW-001 accepted. Counter remains 0/3. Trajectory decaying:
11(p59) → 6(p60) → 4(p61). Pass-62 will confirm corpus-complete remediation.
