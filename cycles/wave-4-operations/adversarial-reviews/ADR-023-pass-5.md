---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T22:00:00Z
phase: 5
pass: 5
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
review_id: ADR-023-pass-5
date: 2026-05-10
reviewer: adversary
target_artifact: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
target_artifact_sha_at_review: "0911b336"
target_artifact_version: "v1.4"
findings_total: 3
findings_by_tier:
  CRIT: 0
  HIGH: 1
  MED: 1
  LOW: 1
  OBS: 0
process_gap_findings: 1
previous_review: "ADR-023-pass-4.md"
convergence_status: NOT_CLEAN
fix_burst_required: true
residuals_from_previous_pass: 1
new_findings_this_pass: 2
streak_status: "0/3 (HIGH residual blocks CLEAN; close to convergence)"
trajectory: "26 → 16 → 12 → 14 → 3 (strong decrease)"
related_tasks: [94, 95]
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-4.md"
  - ".factory/cycles/wave-4-operations/td-from-adr-023-pass-1.md"
  - ".factory/specs/verification-properties/VP-INDEX.md"
  - ".factory/specs/architecture/verification-architecture.md"
  - ".factory/specs/architecture/verification-coverage-matrix.md"
  - ".factory/specs/domain-spec/invariants.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
input-hash: "[live-state]"
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 5)

## Finding ID Convention

Finding IDs use the pass-5-scoped format:

- `F-PASS5-{HIGH,MED,LOW}-NNN` — net-new or residual finding in pass-5

Pass-4 findings that were substantively closed in v1.4 are tracked in the residual
verification table (Part A). F-PASS4-HIGH-002 is marked incomplete-propagation and
carries forward as F-PASS5-HIGH-001 (the residual).

This is pass 5 of the ADR-023 adversarial review cycle. Target: 3 consecutive CLEAN
passes (current streak: 0/3 — this pass NOT_CLEAN due to 1 HIGH residual).

---

## Summary

Fresh-context review of ADR-023 v1.4 at SHA `0911b336`. Pass-5 surfaces **3 findings**
(0 CRIT / 1 HIGH / 1 MED / 1 LOW / 0 OBS). The trajectory shows strong overall
decrease (26→16→12→14→3). The streak remains at 0/3 because a HIGH residual is
present.

The 14 pass-4 findings are **14/14 substantively closed** — 13 fully resolved, 1 with
incomplete propagation (F-PASS4-HIGH-002 status-block fix was applied to the frontmatter
version but not to the body Status block at §A L80).

After fix-burst-5 (3–4 small mechanical fixes), pass-6 has high probability of being
CLEAN, opening streak 1/3.

**Finding counts:**

| Tier | Count | IDs |
|------|-------|-----|
| CRIT | 0 | — |
| HIGH | 1 | F-PASS5-HIGH-001 (residual from F-PASS4-HIGH-002) |
| MED  | 1 | F-PASS5-MED-001 |
| LOW  | 1 | F-PASS5-LOW-001 [process-gap] |
| OBS  | 0 | — |
| **Total** | **3** | |

---

## Part A — Pass-4 Residual Verification

The adversary verified each of the 14 pass-4 findings against the v1.4 artifact at
SHA `0911b336`. The table below records the disposition of every finding. 13 are fully
closed. 1 (F-PASS4-HIGH-002) is marked incomplete-propagation and carries forward as
F-PASS5-HIGH-001.

| Pass-4 Finding | Tier | Pass-4 Description | Pass-5 Verdict | Notes |
|----------------|------|--------------------|----------------|-------|
| F-PASS4-CRIT-001 | CRIT | Story count 12 vs 13 contradicts at 5+ sites | CLOSED | All 5 sites corrected to 12. Verified §C Status, §D Negative Consequences, §D Migration Plan heading, §E Wave 1 header, §F Source/Origin. |
| F-PASS4-CRIT-002 | CRIT | Wave 1 SP arithmetic 30-47 claimed vs actual 32-52 | CLOSED | Arithmetic corrected. Wave 1 SP now correctly stated for 4 stories. Three-wave totals corrected. |
| F-PASS4-CRIT-003 | CRIT | Python open/write bypass — left "COMMITTED v1.2" in v1.3 changelog | CLOSED (partial) | TD-FACTORY-HOOK-BYPASS-001 codified. Fix-burst-4 used Edit/Write only. The stale "COMMITTED v1.2" stamp was corrected in the frontmatter version field. However, the Status block body at §A L80 still reads "COMMITTED v1.3" despite frontmatter being v1.4 — this is F-PASS5-HIGH-001 (incomplete propagation). |
| F-PASS4-HIGH-001 | HIGH | §D Positive Consequences cite Wave 1/E (removed story) | CLOSED | Wave 1/E citations removed from §D Positive Consequences. Verified. |
| F-PASS4-HIGH-002 | HIGH | "COMMITTED v1.2" stamp in v1.3 Status block — stale | CLOSED (incomplete) | Frontmatter version bumped to v1.4. BUT: body Status block at §A L80 still reads "COMMITTED v1.3" while frontmatter is v1.4. Carries forward as F-PASS5-HIGH-001. |
| F-PASS4-HIGH-003 | HIGH | Wave 1 story ordering in table contradicts D→E→A→B→C mandate | CLOSED | Wave 1 table reordered to D→A→B→C (E removed). Verified table row order matches mandate. |
| F-PASS4-HIGH-004 | HIGH | §E Wave 1 header story count stale | CLOSED | §E Wave 1 header updated. Verified. |
| F-PASS4-HIGH-005 | HIGH | §F Source/Origin story count stale | CLOSED | §F Source/Origin corrected. Verified. |
| F-PASS4-MED-001 | MED | §C Status block — implementation_status field missing for v1.3 rescope | CLOSED | implementation_status field present and coherent in v1.4. Verified. |
| F-PASS4-MED-002 | MED | §D Migration Plan table column headers inconsistent | CLOSED | Column headers corrected. Verified. |
| F-PASS4-MED-003 | MED | Changelog entry for v1.3 used wrong category label | CLOSED | Changelog v1.3 entry uses correct category. Verified. |
| F-PASS4-MED-004 | MED | VP-PLUGIN-006 registration table row missing required columns | CLOSED (related) | VP-PLUGIN-006 row corrected in the PREREQ-F instructions. However, see F-PASS5-MED-001 for a related schema gap in the same PREREQ-F block. |
| F-PASS4-LOW-001 | LOW | §E Wave 1 SP column — three entries had wrong per-story SP | CLOSED | SP values corrected. Verified. |
| F-PASS4-LOW-002 | LOW | Changelog v1.3 row — date field format inconsistent | CLOSED | Date format corrected. Verified. |
| F-PASS4-LOW-003 | LOW | §D Negative Consequences — stale story name "PLUGIN-MIGRATION-001-E" | CLOSED | Stale reference removed. Verified. |

**Pass-4 residual summary:** 14/14 substantively closed. 1 incomplete propagation
(F-PASS4-HIGH-002 → F-PASS5-HIGH-001).

---

## Part B — New Findings (Pass 5)

### HIGH

#### F-PASS5-HIGH-001: Status block "COMMITTED v1.3" contradicts frontmatter v1.4 [RESIDUAL from F-PASS4-HIGH-002]

- **Severity:** HIGH
- **Category:** Version-consistency violation (S-7.01 partial-fix regression)
- **Location:** §A Status block, approximately L80
- **Description:** The frontmatter declares `version: "v1.4"` but the §A Status block body reads `**COMMITTED v1.3**`. Fix-burst-4 correctly updated the frontmatter `version:` field but did not propagate the version bump to the body Status block. This is an incomplete-propagation residual — the same class of error that produced F-PASS4-CRIT-003.
- **Evidence:** Frontmatter: `version: "v1.4"`. Body §A L80: `**COMMITTED v1.3** — Plugin-only sensor architecture. Wave 0 (5 prereq stories) → Wave 1 (4 stories: D→A→B→C) → Wave 2 (3 cleanup stories). Total: 12 stories.`
- **Proposed Fix:** Edit the Status block at §A L80. Change the single occurrence of `COMMITTED v1.3` to `COMMITTED v1.4`. Use Edit tool (not Write) to ensure the hook chain fires on the targeted mutation. After fix, grep for all occurrences of `v1.3` and `COMMITTED` in the document to verify no sister sites remain.

### MEDIUM

#### F-PASS5-MED-001: PREREQ-F instructions cite non-existent VP-INDEX `phase` column and abbreviated module name

- **Severity:** MED
- **Category:** Schema drift — VP-INDEX registration instructions reference non-existent column
- **Location:** L204 (phase: migration column reference); L499–500 (module: spec-engine abbreviated name)
- **Description:** The PREREQ-F sub-section contains VP-INDEX registration instructions with a template row that cites a `phase` column. The VP-INDEX schema has no `phase` column. Additionally, the module field is abbreviated as `spec-engine` rather than the canonical `prism-spec-engine`.
- **Evidence:** VP-INDEX column schema (from `.factory/specs/verification-properties/VP-INDEX.md`): `| ID | Description | Category | Module | Status | Story | Kani | Fuzz |` — no `phase` column exists. ADR-023 PREREQ-F registration template at L204 includes `| migration |` in a position that does not correspond to any VP-INDEX column. At L499–500, module is written as `spec-engine` rather than the ARCH-INDEX canonical `prism-spec-engine`.
- **Proposed Fix:** (1) Remove the `phase` column reference from the registration template at L204 so the template row matches the actual VP-INDEX schema. (2) Change `spec-engine` to `prism-spec-engine` at L499–500. Both fixes are in the same PREREQ-F block and should be applied together in fix-burst-5.

### LOW

#### F-PASS5-LOW-001 [process-gap]: `input-hash: "[live-state]"` bracketed-placeholder anti-pattern recurs

- **Severity:** LOW
- **Category:** Process gap — bracketed placeholder in machine-readable field
- **Location:** Frontmatter `input-hash` field
- **Description:** The frontmatter carries `input-hash: "[live-state]"`, the same bracketed-sentinel anti-pattern class as ADR-022's `[md5]` placeholder. The `input-hash` field is machine-readable; a bracketed string disables drift detection. This is a process-gap finding — no logical defect in the ADR's content, but a disabled process control.
- **Evidence:** Frontmatter line: `input-hash: "[live-state]"`. ADR-022 carried `input-hash: "[md5]"` — the same pattern class. No validator currently rejects bracketed sentinels in this field.
- **Proposed Fix:** Compute the actual input hash using `compute-input-hash --update` and replace the placeholder. Systemic recommendation: extend an existing hook (e.g., `validate-changelog-monotonicity`) to reject any `input-hash` value matching `^\[.*\]$`, catching both `[live-state]` and `[md5]` variants at write-time.

---

## Source-of-Truth Verifications

The adversary performed 23 source-of-truth verifications during this pass.

| # | Claim Verified | Source | Result |
|---|----------------|--------|--------|
| 1 | Frontmatter `version: "v1.4"` | frontmatter | PASS |
| 2 | Status block version stamp | body §A L80 | FAIL — "COMMITTED v1.3" contradicts frontmatter v1.4 (F-PASS5-HIGH-001) |
| 3 | Wave 0 story count = 5 | §C, §E body | PASS |
| 4 | Wave 1 story count = 4 (D→A→B→C) | §E table | PASS |
| 5 | Wave 2 story count = 3 (F+G+H) | §E table | PASS |
| 6 | Total story count = 12 | §C, §D, §F | PASS — all sites consistent |
| 7 | Wave 1 SP range (32–52) | §E table arithmetic | PASS |
| 8 | Three-wave total SP | §E summary | PASS |
| 9 | Wave 1 ordering D→A→B→C | §E table row order | PASS |
| 10 | PLUGIN-MIGRATION-001-E removed from Wave 1 scope | §C, §D, §E | PASS — no remaining references |
| 11 | VP-INDEX column schema | `.factory/specs/verification-properties/VP-INDEX.md` | PASS — verified no `phase` column exists |
| 12 | PREREQ-F VP registration template `phase` column | §D PREREQ-F L204 | FAIL — phantom column cited (F-PASS5-MED-001) |
| 13 | Module name `prism-spec-engine` canonical form | ARCH-INDEX module registry | PASS — canonical name is `prism-spec-engine` |
| 14 | PREREQ-F L499–500 module name | body L499–500 | FAIL — abbreviated `spec-engine` (F-PASS5-MED-001) |
| 15 | `input-hash` field validity | frontmatter | FAIL — `[live-state]` is a bracketed sentinel (F-PASS5-LOW-001) |
| 16 | BC-2.16.004 Rust escape-hatch BC body coherence with §B Rule 5 | BC file | PASS |
| 17 | BC-2.01.013 DataSource trait adapter pattern coherence with §B Rule 1 | BC file | PASS |
| 18 | DI-012 back-reference — present in ADR-023 body | §F | PASS — back-reference present and correct |
| 19 | VP-PLUGIN-006 registration row in PREREQ-F instructions | §D | PASS — row present; schema mismatch noted as F-PASS5-MED-001 |
| 20 | Changelog v1.4 entry — date, category, description | frontmatter changelog | PASS |
| 21 | Changelog v1.3 entry — "COMMITTED v1.2" stamp corrected | frontmatter changelog v1.3 row | PASS — v1.3 row no longer says "COMMITTED v1.2"; closed by fix-burst-4 |
| 22 | §D Positive Consequences — no stale Wave 1/E references | §D body | PASS |
| 23 | §D Negative Consequences — no stale "PLUGIN-MIGRATION-001-E" reference | §D body | PASS |

**Verification summary:** 20 PASS / 3 FAIL. The 3 FAILs map exactly to the 3 findings
in this pass.

---

## Top 3 Most-Critical Findings

### Finding 1: F-PASS5-HIGH-001 — Residual Status Block Version Contradiction

The pass-4 fix-burst correctly advanced the frontmatter from v1.3 to v1.4, and
correctly updated the changelog with a v1.4 entry. However, the §A Status block —
the first human-readable declaration of the document's state — still reads
`**COMMITTED v1.3**`. A reader opening this ADR sees the body say v1.3 while the
YAML header says v1.4. This is a version-consistency violation of the same class that
produced the original F-PASS4-CRIT-003 finding (S-7.01 partial-fix regression: the
fix touched some version sites but not all).

The fix is mechanical: one occurrence of `v1.3` in the Status block body must become
`v1.4`. The risk is that fix-burst-5 makes the same error — updating only the Status
block text and missing any sister occurrences. Therefore, the fix-burst-5 architect
must run a post-edit grep for all `v1.3` occurrences in the file before declaring done.

### Finding 2: F-PASS5-MED-001 — PREREQ-F VP-INDEX Registration Template Cites Non-Existent Column

The PREREQ-F instructions are the authoritative blueprint that Wave 0/F story
implementation will follow when registering new verification properties in VP-INDEX.
Those instructions currently contain a registration template that includes a `phase`
column. The VP-INDEX table has no such column. If the Wave 0/F implementer follows
these instructions as written, they will produce a VP-INDEX row with an extra cell
that has no column header — either causing the table renderer to misparse all
subsequent cells (if strict), or silently dropping the extra cell (if lenient, leaving
the module value in the wrong column).

The secondary defect — `spec-engine` vs `prism-spec-engine` — would produce an
unresolvable module reference in ARCH-INDEX lookups. Both defects are in the same
PREREQ-F block and should be fixed together in fix-burst-5.

### Finding 3: F-PASS5-LOW-001 — Bracketed Placeholder in `input-hash` Field

The `input-hash: "[live-state]"` pattern is the same anti-pattern class as
ADR-022's `input-hash: "[md5]"`, which was surfaced by the adversary in an earlier
pass and noted as a process gap. That the same pattern appears in ADR-023 without a
validator catching it confirms that the process-gap recommendation from ADR-022's
review was not acted upon: no validator exists that rejects bracketed sentinels in
machine-readable frontmatter fields. The recommended systemic fix is a hook rule that
rejects any `input-hash` value matching `^\[.*\]$`, which would catch both `[live-state]`
and `[md5]` variants at write-time.

---

## Convergence Assessment

**Verdict:** NOT_CLEAN — RESIDUAL

**Justification:** The HIGH residual (F-PASS5-HIGH-001) blocks a CLEAN declaration
regardless of the low finding count. The 3-CLEAN streak requirement means any HIGH
finding resets the clock. The streak remains at 0/3.

**Fix-burst-5 scope (3–4 mechanical fixes):**
1. §A Status block L80: `COMMITTED v1.3` → `COMMITTED v1.4` (single-line Edit)
2. PREREQ-F L204: remove phantom `phase` column from VP-INDEX registration template (one-line Edit)
3. PREREQ-F L499–500: `spec-engine` → `prism-spec-engine` (one or two-line Edit)
4. Frontmatter `input-hash`: replace `[live-state]` with computed hash (one-line Edit after running compute-input-hash)

After fix-burst-5, pass-6 is expected CLEAN (streak 1/3) assuming no new defects are
introduced by the fixes. With streak 1/3 after pass-6, passes 7 and 8 (idempotency
checks) would close the 3-CLEAN window and declare ADR-023 converged.

**Readiness:** requires revision

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 |
| **New findings** | 2 (F-PASS5-MED-001, F-PASS5-LOW-001) |
| **Duplicate/variant findings** | 1 (F-PASS5-HIGH-001 is residual from F-PASS4-HIGH-002) |
| **Novelty score** | 0.667 (2/3 new) |
| **Median severity** | MED |
| **Trajectory** | 26→16→12→14→3 |
| **Verdict** | FINDINGS_REMAIN |

The strong decrease from 14 to 3 findings indicates that fix-burst-4 was largely
effective. The remaining 3 findings are all mechanical in nature — no new architectural
defects, no new process-gap classes beyond the already-known bracketed-placeholder
pattern. The trajectory reversal at pass-4 (12→14) was caused by a specific root
cause (Python bypass + cascade defects from Wave 1/E rescope) that has been fully
addressed. The trajectory is now back on a strong downward path.

Pass-6 is the projected CLEAN pass that opens streak 1/3.

---

## Operational Notes

Pass-5 was performed from a fresh-context, read-only adversary profile. No code
execution. All source-of-truth verifications were performed against files read in this
session. The 23-verification table above represents the full scope of factual checks
performed.

The near-convergence position (3 findings, all mechanical) indicates that fix-burst-5
should be straightforward. The architect must use Edit/Write tool path only
(TD-FACTORY-HOOK-BYPASS-001) and run a post-edit sister-site sweep before declaring
fix-burst-5 complete.
