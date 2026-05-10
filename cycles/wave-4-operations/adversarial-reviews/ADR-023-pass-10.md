---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T23:30:00Z
phase: 5
pass: 10
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
review_id: ADR-023-pass-10
date: 2026-05-10
reviewer: adversary
target_artifact_sha_at_review: "0502b201"
target_artifact_version: "v1.7"
findings_total: 4
findings_by_tier:
  CRIT: 0
  HIGH: 1
  MED: 3
  LOW: 0
  OBS: 0
process_gap_findings: 0
pass_number: 10
previous_review: "ADR-023-pass-9.md"
convergence_status: NOT_CLEAN
fix_burst_required: true
residuals_from_previous_pass: 0
new_findings_this_pass: 4
streak_status: "0/3 (RESET — pass-10 NOT_CLEAN; pass-9 was second clean but pass-10 surfaces novel defects)"
trajectory: "26 → 16 → 12 → 14 → 3 → 3 → 1 → 0 → 0 → 4 (decay reversed in last pass; fresh-context cross-section reasoning surfaced internal contradiction + sibling-doc drift)"
verifications_performed: 22
related_tasks: [94, 95]
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-9.md"
  - "crates/prism-bin/src/boot.rs"
input-hash: "[live-state]"
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 10)

## Finding ID Convention

Finding IDs use the pass-10-scoped format:

- `F-PASS10-{CRIT,HIGH,MED,LOW,OBS}-NNN` — finding in pass-10

This pass surfaces **4 findings** (1 HIGH + 3 MED). The 3-CLEAN convergence streak
**RESETS** from 2/3 to **0/3**. Fresh-context cross-section reasoning — reading Rule
definitions against Wave scope tables and against the live sibling document
`boot.rs` — surfaced an internal contradiction and sibling-document drift that
pass-9's 20 verifications (which focused on arithmetic, VP/BC citations, and
version stamps) did not cover.

---

## Summary — 4 FINDINGS. NOT_CLEAN. STREAK RESET 2/3 → 0/3.

Pass-10 fresh-context review of ADR-023 v1.7 (SHA `0502b201`, HEAD frozen since
pass-8) yields **4 findings**: 0 CRIT, 1 HIGH, 3 MED, 0 LOW, 0 OBS. The streak
resets from 2/3 to 0/3. This result is surprising given pass-8 and pass-9 were
both CLEAN with 13 and 20 verifications respectively, but the pass-10 reviewer
took a different angle — cross-section reasoning between Rule definitions, Wave
scope tables, and the live sibling document `boot.rs` — that prior passes did not
exercise.

The four findings are:

1. **F-PASS10-HIGH-001** — Internal contradiction: ADR-023 states "v1.0 ships zero
   in-repo plugins" at five body sites (L275, L732, L809-810, L841, L986), yet
   Rule 1 (pure plugin model) and Wave 1/C scope explicitly describe shipping
   in-repo `.prx` WASM complex-transform plugins for OCSF field mapping at Wave
   1/C. These OCSF mapping transforms ARE first-party in-repo plugins. The phrase
   "zero in-repo plugins" is internally contradictory unless scoped specifically
   to third-party extension plugins or future Wave 2+ plugins — which it is not.
2. **F-PASS10-MED-001** — Stale CrowdStrike OAuth refresh plugin reference at
   L589-590: a specific CrowdStrike OAuth2 refresh plugin example remains in the
   Rule 4 description body after Wave 1/E (CrowdStrike) was removed from v1.0
   scope in the v1.3 fix-burst.
3. **F-PASS10-MED-002** — Stale "in-repo CrowdStrike OAuth2 refresh plugin cannot
   be loaded at boot" at L609: same Wave 1/E rescope incompleteness — this
   boot-warning example specifically names CrowdStrike, which is no longer a v1.0
   Wave 1 deliverable.
4. **F-PASS10-MED-003** — boot.rs sibling-document drift: ADR-023 Context section
   at L121-123 and Constraint C5 at L616-617 claim that `custom_adapter_registry`
   exists in `boot.rs` as a "dead field" to be removed by the plugin migration.
   The S-WAVE5-PREP-01 refactor (commit `53b87961`, merged PR #138) already
   removed all `CustomAdapter`/`PluginRuntime` references from `boot.rs`. ADR-023
   describes a dead-field removal that has already happened, creating present-tense
   drift against actual code state.

The trajectory ends at:
**26 → 16 → 12 → 14 → 3 → 3 → 1 → 0 → 0 → 4 (decay reversed; cross-section
reasoning surfaced novel defect classes)**

Fix-burst-8 is required. Fixes are wording-only (no architectural change). The
OCSF complex-transform plugins ARE first-party in-repo plugins; the "zero in-repo
plugins" language needs scoping to distinguish third-party/extension plugins from
bundled first-party platform plugins. The CrowdStrike OAuth refresh plugin
references need replacement with generic examples. The boot.rs context description
needs updating to reflect the post-S-WAVE5-PREP-01 state.

---

## Part A — Fix Verification

### Pass-9 Closure Verification

Pass-9 surfaced zero findings. There are no pass-9 closures to verify. The
pass-9 clean verdict at SHA `0502b201` is confirmed — it was genuinely clean under
the angles pass-9 exercised (arithmetic, VP/BC citations, story counts, version
stamps, DI back-references, changelog immutability, Rule 4/Rule 5 coherence).
Zero residuals are carried into this pass.

---

## Part B — New Findings

### CRITICAL

_None._

---

### HIGH

#### F-PASS10-HIGH-001 — Internal Contradiction: "v1.0 ships zero in-repo plugins" vs Rule 1 + Wave 1/C in-repo OCSF plugin delivery

**Severity:** HIGH
**Location:** L275, L732, L809-810, L841, L986 (five occurrences)
**Cross-reference sites:** Rule 1 (pure plugin model), Wave 1/C scope table, PREREQ-F section

**Evidence:**

At L275, the document states (paraphrased): "v1.0 ships zero in-repo plugins;
all sensor adapters are plugin-delivered." At L732 (Wave 1/C column of the
delivery table), the document describes shipping in-repo `.prx` WASM plugins for
OCSF complex-transform field mapping — these are first-party plugins bundled
in-repository. At L809-810, the phrase "ships with zero bundled in-repo plugins"
appears again in the Boot Sequence section. At L841, a constraint reads "v1.0:
zero in-repo plugins at initial commit." At L986, the same claim recurs in the
summary constraints block.

**Contradiction:** If Wave 1/C ships first-party OCSF complex-transform plugins as
`.prx` WASM artifacts checked into the repository (per Rule 1 + Wave 1/C scope),
then "v1.0 ships zero in-repo plugins" is false at v1.0 delivery time (which
includes Wave 1/C). The sites at L275, L809-810, L841, L986 use unqualified
language that directly contradicts the Wave 1/C delivery commitment.

**Root cause:** The "zero in-repo plugins" language likely originated from the
v1.0+1 signing-deferral context: at initial commit / boot time, there are no
third-party unsigned plugins. But the language was never scoped to distinguish
third-party extension plugins from bundled first-party platform plugins. The
unsigned-plugin boot warning and audit log per TD-PLUGIN-SIGNING-001 v1.0+1
deferral applies to BOTH first-party and third-party plugins — but neither context
makes the language contradictory in the same way.

**Fix:** Scope the "zero in-repo plugins" language at each of the five sites to
clarify the actual constraint — e.g., "v1.0 ships zero third-party plugins" or
"the plugin registry begins empty at boot (unsigned first-party platform plugins
are loaded from bundled `.prx` artifacts per Wave 1/C)." No architectural change.
Wording-only clarification.

---

### MEDIUM

#### F-PASS10-MED-001 — Stale CrowdStrike OAuth2 refresh plugin reference at L589-590

**Severity:** MEDIUM
**Location:** L589-590
**Cross-reference:** Rule 4 (Plugin Extension Mechanism), Wave 1/E (CrowdStrike) rescoped out in v1.3

**Evidence:**

At L589-590, Rule 4 body prose includes a specific CrowdStrike OAuth2 refresh
plugin as an example of the plugin extension mechanism. Wave 1/E (CrowdStrike
sensor) was removed from v1.0 scope during the v1.3 fix-burst (D-337 / pass-4
remediation). The Rule 4 description was updated at a high level in v1.3 to
remove CrowdStrike as a Wave 1/E deliverable, but this specific OAuth refresh
plugin example at L589-590 was not replaced with a generic example.

**Impact:** A reader of Rule 4 sees CrowdStrike as the canonical example of the
extension mechanism, creating false expectation that CrowdStrike OAuth2 refresh is
a v1.0 deliverable. This contradicts the Wave 1/E rescope that occurred in v1.3.

**Fix:** Replace the CrowdStrike OAuth2 refresh plugin example with a generic
sensor plugin example (e.g., "a sensor-specific OAuth2 refresh plugin" or
reference a Wave 2 sensor). Wording-only.

---

#### F-PASS10-MED-002 — Stale "in-repo CrowdStrike OAuth2 refresh plugin cannot be loaded at boot" at L609

**Severity:** MEDIUM
**Location:** L609
**Cross-reference:** Boot warning section, Wave 1/E rescope (v1.3)

**Evidence:**

At L609, the boot sequence warning section states that "the in-repo CrowdStrike
OAuth2 refresh plugin cannot be loaded at boot" as an example of the unsigned-
plugin boot warning behavior. This is the same class as F-PASS10-MED-001: the
v1.3 Rule 4 rescope removed Wave 1/E (CrowdStrike) from v1.0 scope but did not
sweep all CrowdStrike-specific illustrative examples from the body prose.

**Impact:** Two separate prose sites still name CrowdStrike as a v1.0 plugin
example after the rescope. These are residual references from the v1.3 fix-burst
— the rescope was logically complete at the Wave table level but missed the
specific illustrative examples in the Rule 4 and Boot sections.

**Fix:** Replace the CrowdStrike-specific boot warning example with a generic
plugin example. Wording-only.

---

#### F-PASS10-MED-003 — boot.rs sibling-document drift: custom_adapter_registry described as "dead field to remove" but already removed by S-WAVE5-PREP-01

**Severity:** MEDIUM
**Location:** L121-123 (Context section), L616-617 (Constraint C5)
**Cross-reference:** commit `53b87961` (S-WAVE5-PREP-01), PR #138

**Evidence:**

At L121-123 (Context section), ADR-023 describes `custom_adapter_registry` as a
field in `boot.rs` that "currently exists as a dead field" and states that the
plugin migration will remove it. At L616-617 (Constraint C5), the document
similarly describes `CustomAdapter`/`PluginRuntime` as types "to be removed from
boot.rs as part of this migration."

The S-WAVE5-PREP-01 story (commit `53b87961`, merged PR #138, 2026-05-08) already
performed the `prism-bin` chassis refactor. According to the PR title and the
develop branch HEAD lineage, all `CustomAdapter`/`PluginRuntime` references were
removed from `boot.rs` as part of that story's implementation. ADR-023 was not
updated to reflect this — it still reads as if the removal is future work, when
in fact it has already been completed by `53b87961`.

**Impact:** ADR-023's Context section and Constraint C5 describe a code state that
no longer exists. A developer reading the ADR to understand the current migration
status sees an incorrect picture: the dead field removal is presented as pending
work, but it is done. This is sibling-document drift between the ADR and the
implementing commit.

**Fix:** Update L121-123 and L616-617 to reflect the post-S-WAVE5-PREP-01 state.
Change "exists as a dead field" and "to be removed" to past-tense descriptions
noting that the removal was completed in `53b87961` (S-WAVE5-PREP-01). Cite the
commit. Wording-only, no architectural change.

---

### LOW

_None._

---

### OBS (Out-of-Scope Observations — Not Findings)

_None._

---

## Verifications Performed (22 checks)

All 22 source-of-truth verifications executed against ADR-023 v1.7 (SHA
`0502b201`) and sibling document `crates/prism-bin/src/boot.rs`.

| # | Check | Target | Result |
|---|-------|--------|--------|
| SOT-01 | Story count consistency | `13 stories` cited at frontmatter + summary + Wave table + Wave 0 section + PREREQ-F section | PASS — 13 consistent throughout |
| SOT-02 | Story point arithmetic (Wave 1) | Wave 1: 95 SP total claimed | PASS — D+E+A+B+C row sums to 95; Wave 1 subtotals correct |
| SOT-03 | Story point arithmetic (Wave 2) | Wave 2: 146 SP total claimed | PASS — arithmetic consistent across all Wave 2 rows |
| SOT-04 | VP-PLUGIN registration | VP-PLUGIN-001..006 cited in ADR-023 body | PASS — VP-INDEX registers VP-146..VP-152 as aliases; all present |
| SOT-05 | BC frontmatter `scheduled_amendment_in` | Wave 0/F prerequisite BCs cited | PASS — `scheduled_amendment_in: wave-0-prereq-f` present in referenced BCs |
| SOT-06 | DI-012 annotation | DI-012 back-reference cited | PASS — DI-012 annotated in domain-spec/invariants.md with cross-reference to ADR-023 §B.2 |
| SOT-07 | Input-hash real | `input-hash:` field not a bracketed placeholder | PASS — input-hash contains real value, not `[placeholder]` |
| SOT-08 | Process-Gap Awareness section | ADR-023 §G Process-Gap Awareness exists | PASS — section present at expected location |
| SOT-09 | Edit-only discipline | No ADR-023 content rewritten wholesale | PASS — changelog shows incremental fix-burst entries; no wholesale rewrite detected |
| SOT-10 | Version stamp body-wide consistency | `version: "v1.7"` in frontmatter; body Status block | PASS — L80: `COMMITTED v1.7`, L850: `Current version: v1.7`; changelog shows v1.7 row |
| SOT-11 | Wave 0/F PREREQ-F dependency chain | S-PLUGIN-PREREQ-F blocks PREREQ-A through PREREQ-E | PASS — dependency arrows correct in Wave 0 table |
| SOT-12 | TD-VERSION-STAMP-SWEEP-001 reference | Process-Gap section cites TD-VERSION-STAMP-SWEEP-001 | PASS — TD registered and cited at §G |
| SOT-13 | Changelog immutability | Prior changelog rows (v1.1..v1.6) unchanged | PASS — rows match prior observed text verbatim; immutable audit trail intact |
| SOT-14 | Rule 4 ↔ Rule 5 coherence | Rule 4 (plugin extension) and Rule 5 (no built-in sensors) logically consistent at abstract level | PASS — abstract coherence holds; concrete example drift at L589-590, L609 is separate finding F-PASS10-MED-001/002 |
| SOT-15 | VP-INDEX VP-146..VP-152 alias registration | All VP-PLUGIN-NNN aliases map to VP-146..VP-152 | PASS — VP-INDEX v1.29 registers all 7 aliases; counts match ADR-023 body citations |
| SOT-16 | SP arithmetic re-derivation (Wave 1 per-story) | Re-sum D+E+A+B+C from individual story rows | PASS — 95 SP confirmed independently; no rounding or transcription error |
| SOT-17 | SP arithmetic re-derivation (Wave 2 total) | Re-sum Wave 2 from individual story rows | PASS — 146 SP confirmed independently |
| SOT-18 | Rule 1 ↔ Wave delivery table cross-section | Rule 1 (pure plugin model) vs Wave 1/C delivery (OCSF transforms) | FAIL — triggers F-PASS10-HIGH-001; "zero in-repo plugins" contradicts Wave 1/C in-repo OCSF plugin delivery |
| SOT-19 | Wave 1/E rescope completeness sweep | CrowdStrike references after v1.3 Rule 4 rescope | FAIL — triggers F-PASS10-MED-001 (L589-590) + F-PASS10-MED-002 (L609); two stale CrowdStrike-specific examples remain |
| SOT-20 | boot.rs sibling-document sync (custom_adapter_registry) | ADR-023 Context + C5 vs live boot.rs at HEAD | FAIL — triggers F-PASS10-MED-003; boot.rs post-S-WAVE5-PREP-01 (53b87961) no longer has dead field; ADR still describes it as future removal |
| SOT-21 | Rule 1 pure plugin model internal self-consistency | Rule 1 body text consistent with itself across all body sites | PASS — Rule 1 body is self-consistent; the contradiction is between Rule body and "zero in-repo plugins" constraint sites |
| SOT-22 | Process-gap TDs cited correctly | TD-FACTORY-HOOK-BYPASS-001, TD-FIX-BURST-VERIFY-002, TD-ADR-AMEND-001/002 cited in scope | PASS — §G Process-Gap Awareness section cites relevant TDs; no orphaned references |

**PASS: 19 / FAIL: 3**

The three SOT failures directly map to the four findings (SOT-19 maps to two
findings: F-PASS10-MED-001 + F-PASS10-MED-002).

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 3 |
| LOW | 0 |
| OBS | 0 |

**Overall Assessment:** NOT_CLEAN
**Convergence:** FINDINGS_REMAIN — streak RESET 2/3 → 0/3
**Readiness:** fix-burst-8 required before pass-11

---

## Convergence Assessment

Pass-10 is NOT_CLEAN. The 3-CLEAN convergence streak resets from 2/3 to 0/3.

The 4 findings are genuine defects, not edge-case interpretations:
- F-PASS10-HIGH-001 is a direct internal contradiction between constraint language
  and Wave delivery commitments — both authored in the same document.
- F-PASS10-MED-001/002 are stale CrowdStrike-specific references that survived the
  v1.3 Wave 1/E rescope because the rescope did not sweep illustrative examples in
  the Rule 4 and Boot sections.
- F-PASS10-MED-003 is live sibling-document drift: the code state changed in PR
  #138 but the ADR was not updated.

All four are wording-only fixes. No architectural change is required. The
fundamental architecture (pure plugin model, Wave structure, VP/BC citations,
story points, dependency chains) remains sound and is confirmed PASS by 19 of 22
SOT checks.

Fix-burst-8 should:
1. Clarify "zero in-repo plugins" at L275, L732, L809-810, L841, L986 to
   distinguish third-party/extension plugins from bundled first-party platform
   plugins (F-PASS10-HIGH-001).
2. Replace CrowdStrike OAuth2 refresh plugin example at L589-590 with generic
   sensor plugin example (F-PASS10-MED-001).
3. Replace CrowdStrike boot warning example at L609 with generic plugin example
   (F-PASS10-MED-002).
4. Update Context L121-123 and C5 L616-617 to past-tense reflecting removal
   completed in commit `53b87961` (S-WAVE5-PREP-01) (F-PASS10-MED-003).

After fix-burst-8 produces ADR-023 v1.8, dispatch pass-11. Streak restarts at 0/3.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 10 |
| **New findings** | 4 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 4 / (4 + 0) = 1.0 (HIGH — all findings are novel, surfaced by cross-section reasoning not previously applied) |
| **Median severity** | MEDIUM |
| **Trajectory** | 26 → 16 → 12 → 14 → 3 → 3 → 1 → 0 → 0 → 4 |
| **Verdict** | FINDINGS_REMAIN — streak reset; fix-burst-8 required |

The pass-10 finding cluster validates the "fresh-context compounding value"
hypothesis: each new fresh-context pass exercises reasoning angles that prior
passes' anchored framings did not cover. Passes 8 and 9 verified arithmetic,
VP/BC citations, version stamps, story counts, and DI back-references — all
PASS. Pass-10 applied cross-section reasoning between Rule definitions and Wave
delivery tables, and cross-referenced the live sibling document `boot.rs`. This
angle was not covered in earlier passes and surfaced 4 novel defect classes:
internal contradiction (F-PASS10-HIGH-001), incomplete v1.3 rescope sweep
(F-PASS10-MED-001/002), and sibling-document drift from a recently merged PR
(F-PASS10-MED-003).

After fix-burst-8 closes these 4 findings as ADR-023 v1.8, pass-11 should
re-apply both the prior arithmetic/VP/BC/version-stamp checks AND the new
cross-section and sibling-document angles. The 3-CLEAN streak must hold across
all angles simultaneously to achieve protocol convergence.
