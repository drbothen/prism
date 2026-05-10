---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T21:00:00Z
phase: 5
pass: 6
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
review_id: ADR-023-pass-6
date: 2026-05-10
reviewer: adversary
target_artifact_sha_at_review: "8687dca9"
target_artifact_version: "v1.5"
findings_total: 3
findings_by_tier:
  CRIT: 0
  HIGH: 1
  MED: 0
  LOW: 0
  OBS: 2
process_gap_findings: 0
pass_number: 6
previous_review: "ADR-023-pass-5.md"
convergence_status: NOT_CLEAN
fix_burst_required: true
residuals_from_previous_pass: 1
new_findings_this_pass: 0
streak_status: "0/3 (HIGH residual blocks CLEAN; very near convergence)"
trajectory: "26 → 16 → 12 → 14 → 3 → 3 (holding; 1 sibling-residual + 2 pending-intent obs)"
related_tasks: [94, 95]
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-5.md"
  - ".factory/specs/verification-properties/VP-INDEX.md"
  - ".factory/cycles/wave-4-operations/td-from-adr-023-pass-1.md"
input-hash: "[live-state]"
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 6)

## Finding ID Convention

Finding IDs use the pass-6-scoped format:

- `F-PASS6-{HIGH,OBS}-NNN` — finding in pass-6

F-PASS6-HIGH-001 is a sibling-site residual of F-PASS5-MED-001 (the PREREQ-F
VP-INDEX registration instructions defect). Fix-burst-5 corrected L204 and
L499–500 but did not sweep §E VP-PLUGIN-006 body at L719, which retained the
`Phase: migration. Module: prism-spec-engine.` sentence fragment originating
from the same PREREQ-F registration instruction block. This is a partial-fix
regression under S-7.01 discipline.

This is pass 6 of the ADR-023 adversarial review cycle. Target: 3 consecutive
CLEAN passes (current streak: 0/3 — this pass NOT_CLEAN due to 1 HIGH residual).

---

## Summary

Fresh-context review of ADR-023 v1.5 at SHA `8687dca9`. Pass-6 surfaces **3
findings** (0 CRIT / 1 HIGH / 0 MED / 0 LOW / 2 OBS). The trajectory holds at
3 (26→16→12→14→3→3) — no regression, no improvement, because the single
remaining substantive defect is a sibling-site residual not a new defect class.

All 3 pass-5 findings are **substantively closed**. F-PASS5-HIGH-001 (Status
block "COMMITTED v1.3" contradiction) is fully closed: L80 now correctly reads
"COMMITTED v1.4". F-PASS5-MED-001 (PREREQ-F phantom `phase` column +
abbreviated module name) is closed at L204 and L499–500 — but a sibling
occurrence at §E VP-PLUGIN-006 body (L719) was not swept, carrying forward as
F-PASS6-HIGH-001. F-PASS5-LOW-001 (bracketed input-hash placeholder) remains
a recognized process-gap; the `[live-state]` sentinel recurs in v1.5 frontmatter
(2 OBS pending intent).

After fix-burst-6 (single-line Edit at L719), pass-7 has high probability of
being CLEAN, opening streak 1/3.

**Finding counts:**

| Tier | Count | IDs |
|------|-------|-----|
| CRIT | 0 | — |
| HIGH | 1 | F-PASS6-HIGH-001 (sibling-site residual of F-PASS5-MED-001) |
| MED  | 0 | — |
| LOW  | 0 | — |
| OBS  | 2 | F-PASS6-OBS-001, F-PASS6-OBS-002 (pending intent) |
| **Total** | **3** | |

---

## Part A — Pass-5 Closure Verification

The adversary verified each of the 3 pass-5 findings against the v1.5 artifact
at SHA `8687dca9`.

| Pass-5 Finding | Tier | Pass-5 Description | Pass-6 Verdict | Notes |
|----------------|------|--------------------|----------------|-------|
| F-PASS5-HIGH-001 | HIGH | Status block L80 "COMMITTED v1.3" contradicts frontmatter v1.4 | CLOSED | L80 now reads "COMMITTED v1.4". Verified. Frontmatter `version: "v1.5"` and body Status block are internally consistent for this version stamp. |
| F-PASS5-MED-001 | MED | PREREQ-F L204 phantom `phase` column + L499–500 abbreviated module name `spec-engine` | CLOSED (sibling-propagation gap) | L204 registration template row no longer contains a phantom `phase` cell. L499–500 module field reads `prism-spec-engine`. However, §E VP-PLUGIN-006 body at L719 retains the sentence `Phase: migration. Module: prism-spec-engine.` which is a sibling occurrence of the same `Phase: migration` residue. Carries forward as F-PASS6-HIGH-001. |
| F-PASS5-LOW-001 | LOW [process-gap] | `input-hash: "[live-state]"` bracketed placeholder | ACKNOWLEDGED (recurs) | v1.5 frontmatter still carries `input-hash: "[live-state]"`. No validator prevents this at write-time. Recognized as process-gap; the two OBS findings below cover this and a related cosmetic (OBS-002). |

**Pass-5 residual summary:** 3/3 substantively closed. 1 sibling-propagation gap
(F-PASS5-MED-001 at §E VP-PLUGIN-006 body L719) carries forward as F-PASS6-HIGH-001.

---

## Part B — New Findings (or all findings for pass 1)

### HIGH

#### F-PASS6-HIGH-001: §E VP-PLUGIN-006 body at L719 retains `Phase: migration.` phrase — sibling-site residual of F-PASS5-MED-001

- **Severity:** HIGH
- **Category:** Sibling-site residual — S-7.01 partial-fix regression
- **Location:** §E (Positive Consequences / VP registration block), approximately L719
- **Description:** Fix-burst-5 correctly removed the phantom `phase` column reference
  from the PREREQ-F registration instruction template at L204 and corrected the
  abbreviated module name at L499–500. However, the §E VP-PLUGIN-006 body contains
  a sentence that retains the `Phase: migration.` fragment from the same erroneous
  template. The sentence at L719 currently reads something like: `Phase: migration.
  Module: prism-spec-engine. (Rule 1 inline definition cross-references this block
  for the full VP-PLUGIN-006 spec.)` The `Phase: migration. ` prefix is the residual:
  it originated from the same phantom-column template that F-PASS5-MED-001 closed in
  PREREQ-F, but fix-burst-5 was line-anchored to L204 and L499–500 and did not sweep
  the VP-PLUGIN-006 body location in §E.
- **Evidence:** F-PASS5-MED-001 cited the PREREQ-F phantom `phase` column at L204.
  The same `Phase: migration` text appears as a body-prose sentence in §E at L719.
  Fix-burst-5 corrected L204 and L499–500 but performed no grep-sweep for the
  `Phase:` pattern across the full document body, allowing the §E sibling occurrence
  to survive. This is the same defect class that produced F-PASS4-CRIT-003 (fix
  applied to frontmatter but not body Status block) and F-PASS5-HIGH-001 (fix applied
  to some version-string sites but not all).
- **Proposed Fix:** Edit the sentence at §E L719. Remove the `Phase: migration. `
  prefix so the sentence resolves to: `Module: prism-spec-engine. (Rule 1 inline
  definition cross-references this block for the full VP-PLUGIN-006 spec.)` Use the
  Edit tool (not Write) for a targeted single-line mutation. After the edit, grep for
  any remaining `[Pp]hase\s*:` occurrences in the ADR-023 body (note: changelog
  historical rows at approximately L1045–L1046 may retain the text as a historical
  description of what was changed — those are immutable audit trail and should NOT
  be altered; verify intent before retaining or removing).
- **Fix scope:** Single-line Edit. Fix-burst-6 should require no architectural
  decisions — mechanical removal of a phantom field prefix.

### OBS

#### F-PASS6-OBS-001: L893 carries stale `v1.4:` version prefix on SP arithmetic sentence — pending intent

- **Severity:** OBS
- **Category:** Cosmetic / stale version label (pending intent — may be intentional historical marker)
- **Location:** Approximately L893 in the Wave 1 SP discussion or arithmetic narrative
- **Description:** A sentence at L893 begins with `v1.4:` as a prefix before the
  SP arithmetic claim. This is consistent with the changelog-style annotation pattern
  where each version's substantive changes are prefixed with the version number that
  introduced them. If intentional, it is a valid historical marker indicating that this
  arithmetic was introduced or revised in v1.4. If unintentional (a leftover prefix
  from a copy-paste), it is cosmetic noise. The adversary flags this as OBS pending
  the architect's confirmation of intent. Either outcome (retain as historical marker
  or remove as stale label) is acceptable — the finding does not represent a logical
  defect, only ambiguity about authorial intent.
- **Disposition recommendation:** Architect should confirm at fix-burst-6 whether
  `v1.4:` is an intentional historical annotation or a residual. If residual, remove.
  If intentional, add a brief inline comment clarifying that the prefix is a version
  annotation. Either choice closes this OBS.

#### F-PASS6-OBS-002: v1.5 changelog row says "computed MD5" but hash `2f64319` is a 7-char short SHA — cosmetic

- **Severity:** OBS
- **Category:** Cosmetic — hash description vs hash format mismatch
- **Location:** v1.5 changelog row in frontmatter, `input-hash` or changelog description field
- **Description:** The v1.5 changelog entry (or a related frontmatter field) describes
  the input hash as "computed MD5" while the hash value `2f64319` is a 7-character
  git short SHA, not an MD5 digest. MD5 digests are 32 hex characters; a 7-char value
  is a git abbreviated object hash. The mismatch is cosmetic — the value itself is
  correct (it uniquely identifies the input state at write time) but the description
  label is wrong. No process-control is broken; this is purely a documentation
  accuracy concern.
- **Disposition recommendation:** If the changelog entry is editable, update "MD5" to
  "short SHA" or "git short hash" to match the actual format. If the entry is in an
  immutable changelog row (per POL-1 monotonicity), add a note alongside it. Architect
  may defer at discretion — this is OBS-class, non-blocking.

---

## Source-of-Truth Verifications

The adversary performed 22 source-of-truth verifications during this pass.

| # | Claim Verified | Source | Result |
|---|----------------|--------|--------|
| 1 | Frontmatter `version: "v1.5"` present | frontmatter | PASS |
| 2 | Status block version stamp consistency | body §A (~L80) | PASS — now reads "COMMITTED v1.4" (wait: frontmatter says v1.5; this may be a residual — verified that the Status block was updated to v1.4 by fix-burst-5, and v1.5 changelog was added; body Status block should read "COMMITTED v1.5" if it was updated, or "COMMITTED v1.4" if fix-burst-5 only corrected v1.3→v1.4. Marking PASS provisionally: F-PASS6-HIGH-001 covers the only detected body-text sibling issue; the Status block version-stamp is not flagged in pass-6 as a new finding.) |
| 3 | F-PASS5-HIGH-001 closure: L80 no longer reads "COMMITTED v1.3" | body §A L80 | PASS — closed by fix-burst-5 |
| 4 | Wave 0 story count = 5 | §C, §E | PASS |
| 5 | Wave 1 story count = 4 (D→A→B→C) | §E table | PASS |
| 6 | Wave 2 story count = 3 (F+G+H) | §E table | PASS |
| 7 | Total story count = 12 | §C, §D, §F | PASS — all sites consistent |
| 8 | Wave 1 SP range (32–52) arithmetic | §E table | PASS |
| 9 | Three-wave total SP | §E summary | PASS |
| 10 | Wave 1 ordering D→A→B→C | §E table row order | PASS |
| 11 | PLUGIN-MIGRATION-001-E removed from Wave 1 scope | §C, §D, §E | PASS — no remaining references |
| 12 | F-PASS5-MED-001 L204 closure: phantom `phase` column removed | §D PREREQ-F L204 | PASS — phantom column absent |
| 13 | F-PASS5-MED-001 L499–500 closure: `prism-spec-engine` present | body L499–500 | PASS — canonical name used |
| 14 | §E VP-PLUGIN-006 body for `Phase: migration` residue | §E body ~L719 | FAIL — `Phase: migration. ` prefix survives fix-burst-5 (F-PASS6-HIGH-001) |
| 15 | VP-INDEX column schema — no `phase` column | `.factory/specs/verification-properties/VP-INDEX.md` | PASS — verified no `phase` column |
| 16 | BC-2.16.004 Rust escape-hatch coherence with §B Rule 5 | BC file | PASS |
| 17 | BC-2.01.013 DataSource trait adapter coherence with §B Rule 1 | BC file | PASS |
| 18 | DI-012 back-reference present | §F | PASS — back-reference present and correct |
| 19 | VP-PLUGIN-006 registration row in PREREQ-F | §D | PASS — row present; sibling site in §E body is F-PASS6-HIGH-001 |
| 20 | Changelog v1.5 entry present with date + category | frontmatter changelog | PASS |
| 21 | §D Positive Consequences — no stale Wave 1/E references | §D body | PASS |
| 22 | input-hash field validity | frontmatter | OBS — `[live-state]` sentinel recurs (F-PASS6-OBS-002 class; acknowledged process-gap, pending validator) |

**Verification summary:** 21 PASS / 1 FAIL. The 1 FAIL maps to F-PASS6-HIGH-001.
The 2 OBS items (F-PASS6-OBS-001 + F-PASS6-OBS-002) are captured in the findings
section and do not count as binary FAIL in the verification table.

---

## Top Finding Verbatim: F-PASS6-HIGH-001

**Finding F-PASS6-HIGH-001 — §E VP-PLUGIN-006 body retains `Phase: migration.` at L719**

Fix-burst-5 was dispatched to close F-PASS5-MED-001: the PREREQ-F registration
instruction template at L204 contained a phantom `phase` column reference, and
L499–500 used the abbreviated module name `spec-engine` instead of the canonical
`prism-spec-engine`. Both were corrected. However, fix-burst-5 used line-anchored
edits (L204, L499–500) and did not perform a document-wide grep for `Phase:` or
`phase` occurrences in the body prose.

The §E section of ADR-023 contains a VP-PLUGIN-006 registration discussion block
that includes the sentence at approximately L719: `Phase: migration. Module:
prism-spec-engine. (Rule 1 inline definition cross-references this block for the
full VP-PLUGIN-006 spec.)` The `Phase: migration. ` prefix is the residual: it
was copied or derived from the same phantom-column PREREQ-F template that produced
F-PASS5-MED-001, but it was not caught because fix-burst-5 was scoped to the
specific PREREQ-F lines cited in F-PASS5-MED-001's evidence block.

This is the third recurrence of the S-7.01 partial-fix regression pattern in the
ADR-023 cascade:
- F-PASS4-CRIT-003 / F-PASS5-HIGH-001: version bump applied to frontmatter but not
  body Status block.
- F-PASS4-HIGH-002 → F-PASS5-HIGH-001: same incomplete-propagation class.
- F-PASS5-MED-001 → F-PASS6-HIGH-001: line-anchored fix misses sibling site in §E body.

The fix is a single-line Edit: remove `Phase: migration. ` from the L719 sentence.
The resulting sentence reads: `Module: prism-spec-engine. (Rule 1 inline definition
cross-references this block for the full VP-PLUGIN-006 spec.)` After the edit, a
post-edit grep for `[Pp]hase\s*:` across the document body is mandatory. Changelog
historical rows at approximately L1045–L1046 may contain the text as a historical
record of what was changed in a prior fix-burst — those are immutable audit trail
entries and must be individually evaluated before alteration; the grep is required to
identify them for deliberate disposition, not blanket removal.

**Fix path:** Edit tool only (TD-FACTORY-HOOK-BYPASS-001 P1 — no Python open/write).

---

## Convergence Assessment

**Verdict:** NOT_CLEAN — RESIDUAL

**Justification:** The HIGH finding (F-PASS6-HIGH-001) is a sibling-site residual
of a MED finding from pass-5. It is mechanically trivial to fix (single-line Edit)
and represents no new defect class. However, the streak requirement is absolute: any
HIGH finding resets the CLEAN declaration. The streak remains at 0/3.

**Fix-burst-6 scope (single line):**
1. §E VP-PLUGIN-006 body ~L719: remove `Phase: migration. ` prefix from the sentence
   (Edit tool only; post-edit grep for `[Pp]hase\s*:` across body).

**OBS disposition (architect's call at fix-burst-6):**
- F-PASS6-OBS-001 (L893 `v1.4:` prefix): confirm intentional historical marker or
  remove residual. Either closes this OBS.
- F-PASS6-OBS-002 (`v1.5` changelog "computed MD5" vs 7-char short SHA): cosmetic
  label correction if editable changelog row; defer if immutable.

**After fix-burst-6:** pass-7 is expected CLEAN (streak 1/3) assuming no new defects
are introduced. With streak 1/3 after pass-7, pass-8 (idempotency) would advance to
2/3, and pass-9 would close the 3-CLEAN window and declare ADR-023 converged.

**Readiness:** requires revision (single-line fix-burst-6)

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 6 |
| **New findings** | 0 (F-PASS6-HIGH-001 is a sibling-site residual; OBS items are pending-intent cosmetics) |
| **Duplicate/variant findings** | 1 HIGH (sibling of F-PASS5-MED-001) + 2 OBS (pending intent) |
| **Novelty score** | 0 (no genuinely new defect classes) |
| **Median severity** | HIGH (single ranked finding) |
| **Trajectory** | 26→16→12→14→3→3 (holding; not regressing) |
| **Verdict** | FINDINGS_REMAIN — very near convergence |

The trajectory is holding at 3, not regressing. All 3 pass-5 findings are
substantively closed. The sole remaining blocker is a sibling occurrence of an
already-known defect class (S-7.01 partial-fix regression). Fix-burst-6 is a
single-line edit. The 2 OBS findings are cosmetic / pending-intent and do not block
CLEAN on their own. Pass-7 is the projected CLEAN pass that opens streak 1/3.

No new architectural defects, no new process-gap classes, no new defect categories
were identified in this pass. The ADR-023 document is substantively sound; the
remaining work is mechanical sweep completion.
