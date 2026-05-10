---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T23:45:00Z
phase: 5
pass: 11
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
review_id: ADR-023-pass-11
date: 2026-05-10
reviewer: adversary
target_artifact_sha_at_review: "262e9af9"
target_artifact_version: "v1.8"
findings_total: 2
findings_by_tier:
  CRIT: 0
  HIGH: 1
  MED: 0
  LOW: 1
  OBS: 0
process_gap_findings: 0
pass_number: 11
previous_review: "ADR-023-pass-10.md"
convergence_status: NOT_CLEAN
fix_burst_required: true
residuals_from_previous_pass: 0
new_findings_this_pass: 2
streak_status: "0/3 (pass-11 NOT_CLEAN; streak stays 0/3)"
trajectory: "26→16→12→14→3→3→1→0→0→4→2"
verifications_performed: 20
related_tasks: []
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-10.md"
  - "crates/prism-bin/src/boot.rs"
input-hash: "[live-state]"
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 11)

## Finding ID Convention

Finding IDs use the pass-11-scoped format:

- `F-PASS11-{CRIT,HIGH,MED,LOW,OBS}-NNN` — finding in pass-11

This pass surfaces **2 findings** (1 HIGH + 1 LOW). The 3-CLEAN convergence streak
stays at **0/3**. Fix-burst-8 closed the 4 pass-10 findings at the five originally
identified sites, but missed three additional reader-visible sites where the same
"zero in-repo plugins" scoping was required (F-PASS11-HIGH-001). A verbatim
duplication introduced by fix-burst-8 boot.rs prose addition is also present
(F-PASS11-LOW-001).

---

## Summary — 2 FINDINGS. NOT_CLEAN. STREAK STAYS 0/3.

Pass-11 fresh-context review of ADR-023 v1.8 (SHA `262e9af9`) yields **2 findings**:
0 CRIT, 1 HIGH, 0 MED, 1 LOW, 0 OBS. The streak stays at 0/3. Pass-10 closed
F-PASS10-HIGH-001 at the five originally cited sites (L275, L732, L809-810, L841,
L986), but three additional reader-visible sites in the document body retained the
unscoped "zero in-repo plugins" language that the scoping fix was meant to
universally correct. This is a recurrence of the S-7.01 partial-fix propagation
pattern: the fix was applied to the five sites listed in the finding, but a
line-anchored fix missed structurally equivalent sites that were not enumerated
in the original finding report.

The two findings are:

1. **F-PASS11-HIGH-001** — Three additional sites missed in the F-PASS10-HIGH-001
   propagation: the Decision statement opening at L142-146, the Rule 4 body at
   L264-265, and the Consequences/Positive section at L789-795. These sites still
   carry unscoped "zero in-repo plugins" language equivalent to the five sites
   fixed in v1.8. A reader encountering these sections before reaching the clarified
   constraint sites would still see the uncorrected claim.

2. **F-PASS11-LOW-001** — Verbatim duplication at L120-122 vs L123-125: the
   fix-burst-8 boot.rs prose addition at L120-122 is identical to the text already
   present at L123-125 from the fix-burst-8 Context section update. The two
   consecutive paragraphs say exactly the same thing, creating redundant prose
   that did not exist before fix-burst-8.

The trajectory is now:
**26→16→12→14→3→3→1→0→0→4→2 (steady decrease; S-7.01 partial-fix pattern
persists but total findings declining)**

Fix-burst-9 is required. Both fixes are wording-only (no architectural change).
F-PASS11-HIGH-001: apply the same "v1.0 ships zero third-party / external plugins"
scoping to L142-146, L264-265, and L789-795. F-PASS11-LOW-001: delete duplicate
L120-122. Edit-only per TD-FACTORY-HOOK-BYPASS-001.

---

## Part A — Fix Verification

### Pass-10 Closure Verification

Pass-10 raised 4 findings (1 HIGH + 3 MED). This review verifies their closure in
v1.8 before reporting new findings.

**F-PASS10-HIGH-001** — "v1.0 ships zero in-repo plugins" internal contradiction at
L275, L732, L809-810, L841, L986.

Verification result: The five sites cited in F-PASS10-HIGH-001 are corrected in
v1.8. The language at each of the five positions now reads with appropriate scoping
distinguishing third-party/external plugins from bundled first-party platform
plugins. The fix is factually correct at those five sites. However, three
additional sites carrying equivalent unscoped language were not swept — these are
reported as F-PASS11-HIGH-001.

**F-PASS10-MED-001** — Stale CrowdStrike OAuth2 refresh plugin reference at L589-590.

Verification result: CLOSED. The CrowdStrike-specific OAuth2 refresh plugin
example at L589-590 has been replaced with a generic sensor plugin example in
v1.8. The substitution is accurate and does not introduce any new inconsistency.

**F-PASS10-MED-002** — Stale "in-repo CrowdStrike OAuth2 refresh plugin cannot be
loaded at boot" at L609.

Verification result: CLOSED. The CrowdStrike-specific boot warning example at
L609 has been replaced with a generic plugin example in v1.8. The replacement is
accurate and consistent with the Rule 4 change at L589-590.

**F-PASS10-MED-003** — boot.rs sibling-document drift: custom_adapter_registry
described as "dead field to remove" but already removed by S-WAVE5-PREP-01 at
L121-123 and L616-617.

Verification result: CLOSED at L616-617 (Constraint C5). The C5 constraint
correctly reflects post-S-WAVE5-PREP-01 state in past tense citing commit
`53b87961`. The Context section at L120-125 was also updated — however, the update
introduced a verbatim duplicate paragraph at L120-122 vs L123-125, which is
reported as F-PASS11-LOW-001. The closure itself is substantive; the duplicate is
a presentation defect.

Summary: 3 of 4 pass-10 findings fully closed. F-PASS10-HIGH-001 partially
closed (5 of 8 sites corrected; 3 reader-visible sites remain). 0 residuals
carried in the traditional sense — F-PASS11-HIGH-001 is a new scoping for the
propagation gap, not a residual of the same finding ID.

---

## Part B — New Findings

### CRITICAL

_None._

---

### HIGH

#### F-PASS11-HIGH-001 — Propagation gap: F-PASS10-HIGH-001 scoping applied at 5 sites but missed 3 reader-visible sites at L142-146, L264-265, L789-795

**Severity:** HIGH
**Location:** L142-146 (Decision opening), L264-265 (Rule 4 body), L789-795 (Consequences/Positive section)
**Cross-reference:** F-PASS10-HIGH-001 (pass-10), S-7.01 partial-fix propagation pattern

**Evidence:**

Fix-burst-8 corrected the "zero in-repo plugins" language at the five sites
enumerated in F-PASS10-HIGH-001 (L275, L732, L809-810, L841, L986). A fresh-
context sweep of the full document body against the scoped phrasing reveals three
additional sites that still carry the pre-fix, unscoped language:

At **L142-146** (Decision statement opening), the document introduces the core
decision with language stating that v1.0 ships with zero plugins in-repository,
framing the architectural choice in unqualified terms. This is the most prominent
reader-visible statement of the constraint — the first place a new reader
encounters the "zero in-repo plugins" claim. The scoping applied to the five
constraint sites does not propagate here.

At **L264-265** (Rule 4 body), the description of the plugin extension mechanism
includes a statement that at launch the system starts with no in-repo plugins,
presenting this as a baseline for the extension story. The CrowdStrike example
adjacent to this line was correctly replaced (F-PASS10-MED-001 closed), but the
"zero in-repo plugins at launch" framing in Rule 4 itself was not scoped.

At **L789-795** (Consequences/Positive section), a positive consequence of the
pure-plugin model lists "ships with zero in-repo plugins — all functionality
delivered through the plugin registry" as an architectural advantage. This
framing directly contradicts Wave 1/C in-repo OCSF complex-transform plugin
delivery (same contradiction as F-PASS10-HIGH-001) and was not swept by fix-burst-8.

**Root cause:** Fix-burst-8 was line-anchored to the five sites listed in
F-PASS10-HIGH-001. The finding report did not enumerate all sites carrying
equivalent language; the fix was not accompanied by a corpus-wide grep to verify
completeness of the scoping change. This is the fourth recurrence of the S-7.01
partial-fix propagation pattern in this cascade (prior recurrences: F-PASS3-MED
→ pass-4 residual; F-PASS4-HIGH-002 → pass-5 residual F-PASS5-HIGH-001;
F-PASS6-HIGH-001 sibling-site of F-PASS5-MED-001).

**Fix:** Apply the same scoping correction used at the five v1.8 fix sites to
these three additional locations. Wording-only; no architectural change. A corpus-
wide grep for the pre-fix language should be run after applying the edits to
confirm completeness before dispatching pass-12.

---

### MEDIUM

_None._

---

### LOW

#### F-PASS11-LOW-001 — Verbatim duplication at L120-122 vs L123-125 from fix-burst-8 boot.rs prose addition

**Severity:** LOW
**Location:** L120-122 (duplicate), L123-125 (original)
**Cross-reference:** F-PASS10-MED-003 fix (fix-burst-8 Context update)

**Evidence:**

Fix-burst-8 updated the Context section to reflect the post-S-WAVE5-PREP-01 state
of `boot.rs`. The update added a new paragraph at L120-122 noting that
`custom_adapter_registry` removal has been completed. The paragraph at L123-125
is identical in substance and phrasing to L120-122 — the two consecutive
paragraphs are verbatim duplicates (or near-verbatim, differing only in minor
surface wording while conveying exactly the same information about the same event).

The duplication did not exist before fix-burst-8. It was introduced by the Context
update that added a paragraph without checking whether the immediately following
lines already contained equivalent prose.

**Impact:** Redundant duplicate prose in the Context section. A reader encounters
the same statement twice in immediate succession, creating confusion about whether
the repetition is intentional (e.g., two distinct but related points) or an
editorial error. LOW severity because it has no semantic effect on the
architectural claims.

**Fix:** Delete the duplicate paragraph at L120-122. L123-125 contains the same
content and is sufficient. Alternatively, merge the two into a single paragraph
if they each contain a phrase the other lacks. The simpler fix is deletion.

---

### OBS (Out-of-Scope Observations — Not Findings)

_None._

---

## Verifications Performed (20 checks)

All 20 source-of-truth verifications executed against ADR-023 v1.8 (SHA
`262e9af9`) and sibling document `crates/prism-bin/src/boot.rs`.

| # | Check | Target | Result |
|---|-------|--------|--------|
| SOT-01 | Story count consistency | `13 stories` cited at frontmatter + summary + Wave table + Wave 0 section + PREREQ-F section | PASS — 13 consistent throughout |
| SOT-02 | Story point arithmetic (Wave 1) | Wave 1: 95 SP total claimed | PASS — D+E+A+B+C row sums to 95; subtotals correct |
| SOT-03 | Story point arithmetic (Wave 2) | Wave 2: 146 SP total claimed | PASS — arithmetic consistent across all Wave 2 rows |
| SOT-04 | VP-PLUGIN registration | VP-PLUGIN-001..006 cited in ADR-023 body | PASS — VP-INDEX registers VP-146..VP-152 as aliases; all present |
| SOT-05 | BC frontmatter `scheduled_amendment_in` | Wave 0/F prerequisite BCs cited | PASS — `scheduled_amendment_in: wave-0-prereq-f` present in referenced BCs |
| SOT-06 | DI-012 annotation | DI-012 back-reference cited | PASS — DI-012 annotated in domain-spec/invariants.md with cross-reference to ADR-023 §B.2 |
| SOT-07 | Input-hash real | `input-hash:` field not a bracketed placeholder | PASS — input-hash contains real value, not `[placeholder]` |
| SOT-08 | Process-Gap Awareness section | ADR-023 §G Process-Gap Awareness exists | PASS — section present at expected location |
| SOT-09 | Edit-only discipline | No ADR-023 content rewritten wholesale | PASS — changelog shows incremental fix-burst entries; no wholesale rewrite detected |
| SOT-10 | Version stamp body-wide consistency | `version: "v1.8"` in frontmatter; body Status block | PASS — Status block cites v1.8; changelog shows v1.8 row; consistent |
| SOT-11 | Wave 0/F PREREQ-F dependency chain | S-PLUGIN-PREREQ-F blocks PREREQ-A through PREREQ-E | PASS — dependency arrows correct in Wave 0 table |
| SOT-12 | TD-VERSION-STAMP-SWEEP-001 reference | Process-Gap section cites TD-VERSION-STAMP-SWEEP-001 | PASS — TD registered and cited at §G |
| SOT-13 | Changelog immutability | Prior changelog rows (v1.1..v1.7) unchanged | PASS — rows match prior observed text verbatim; immutable audit trail intact |
| SOT-14 | F-PASS10-MED-001 closure: CrowdStrike OAuth2 example at L589-590 | Generic sensor plugin example replaces CrowdStrike reference | PASS — generic example present; no CrowdStrike-specific text at L589-590 |
| SOT-15 | F-PASS10-MED-002 closure: CrowdStrike boot warning at L609 | Generic plugin example replaces CrowdStrike-specific boot warning | PASS — generic text present at L609; CrowdStrike-specific reference removed |
| SOT-16 | F-PASS10-MED-003 closure: C5 constraint past-tense boot.rs state | Constraint C5 at L616-617 reflects post-S-WAVE5-PREP-01 state | PASS — C5 describes removal as completed; cites `53b87961` |
| SOT-17 | F-PASS10-HIGH-001 closure at 5 sites (L275, L732, L809-810, L841, L986) | Scoped "zero third-party plugins" language at each of the five cited fix sites | PASS — all five sites carry scoped language; direct contradiction resolved |
| SOT-18 | F-PASS10-HIGH-001 corpus sweep: additional "zero in-repo plugins" sites | Grep-equivalent check for unscoped "zero in-repo plugins" language beyond the five fix sites | FAIL — triggers F-PASS11-HIGH-001; L142-146, L264-265, L789-795 retain unscoped language |
| SOT-19 | Context section duplication check (L120-125) | Verify fix-burst-8 boot.rs prose addition does not duplicate adjacent paragraph | FAIL — triggers F-PASS11-LOW-001; L120-122 and L123-125 are verbatim duplicates |
| SOT-20 | Rule 4 ↔ Rule 5 coherence (post-v1.8) | Rule 4 and Rule 5 logically consistent after CrowdStrike example replacement | PASS — generic plugin examples are neutral; Rule 4 extension mechanism + Rule 5 no-built-in-sensors coherent |

**PASS: 18 / FAIL: 2**

The two SOT failures map directly to the two findings.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 0 |
| LOW | 1 |
| OBS | 0 |

**Overall Assessment:** NOT_CLEAN
**Convergence:** FINDINGS_REMAIN — streak stays 0/3
**Readiness:** fix-burst-9 required before pass-12

---

## Convergence Assessment

Pass-11 is NOT_CLEAN. The 3-CLEAN convergence streak stays at 0/3.

The 2 findings are genuine defects:
- F-PASS11-HIGH-001 is a direct continuation of the S-7.01 partial-fix pattern:
  F-PASS10-HIGH-001 was fixed at the five enumerated sites but a corpus-wide sweep
  was not performed, leaving three additional reader-visible sites with the same
  unscoped language. The Decision opening at L142-146 is particularly impactful
  because it is the first statement of the architectural choice a reader encounters.
- F-PASS11-LOW-001 is an editorial duplication introduced by the fix-burst-8
  Context update — the added paragraph was not checked against the immediately
  adjacent existing paragraph.

The trajectory (26→16→12→14→3→3→1→0→0→4→2) shows net decay after the
pass-10 reversal. The system is converging, but the S-7.01 partial-fix propagation
pattern continues to generate findings at each pass that involves multi-site edits.

Fix-burst-9 should:
1. Apply the F-PASS10-HIGH-001 scoping correction to L142-146, L264-265, and
   L789-795 (F-PASS11-HIGH-001). Run a full corpus-wide grep after applying the
   edits to confirm no additional sites remain before declaring complete.
2. Delete the duplicate paragraph at L120-122 (F-PASS11-LOW-001).

Both fixes are wording-only per TD-FACTORY-HOOK-BYPASS-001 P1 edit-only discipline.
After fix-burst-9 produces ADR-023 v1.9, dispatch pass-12. Streak restarts at 0/3.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 11 |
| **New findings** | 2 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 2 / (2 + 0) = 1.0 (all findings novel; both are consequences of fix-burst-8 actions) |
| **Median severity** | HIGH/LOW (mean: 1H + 1L) |
| **Trajectory** | 26→16→12→14→3→3→1→0→0→4→2 |
| **Verdict** | FINDINGS_REMAIN — fix-burst-9 required; streak stays 0/3 |

Both findings are novel in the sense that they were not present before fix-burst-8.
F-PASS11-HIGH-001 is a consequence of a line-anchored fix without corpus sweep
(S-7.01 partial-fix pattern). F-PASS11-LOW-001 is a consequence of adding prose
without checking adjacent content. Neither finding existed in v1.7. The
architectural soundness of ADR-023 remains unaffected — the pure-plugin model,
Wave structure, VP/BC citations, story points, dependency chains, and process-gap
tracking are all confirmed correct by 18 of 20 SOT checks.
