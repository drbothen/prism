---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T21:00:00Z
phase: 5
pass: 4
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
review_id: ADR-023-pass-4
date: 2026-05-10
reviewer: adversary
target_artifact: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
target_artifact_sha_at_review: "eabee0e0"
target_artifact_version: "v1.3"
findings_total: 14
findings_by_tier:
  CRIT: 3
  HIGH: 5
  MED: 4
  LOW: 3
  OBS: 2
process_gap_findings: 2
previous_review: "ADR-023-pass-3.md"
convergence_status: NOT_CLEAN
fix_burst_required: true
residuals_from_previous_pass: 2
new_findings_this_pass: 12
streak_status: "0/3 (no change — pass-4 NOT_CLEAN)"
trajectory: "26 → 16 → 12 → 14 (decay REVERSED; cascade defects from Wave 1/E rescope)"
related_tasks: [94, 95, 96]
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-3.md"
  - ".factory/cycles/wave-4-operations/td-from-adr-023-pass-1.md"
  - ".factory/specs/architecture/ARCH-INDEX.md"
  - ".factory/specs/verification-properties/VP-INDEX.md"
  - ".factory/specs/domain-spec/invariants.md"
  - ".factory/policies.yaml"
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
input-hash: "[live-state]"
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 4)

## Finding ID Convention

Finding IDs use the pass-4-scoped format:

- `F-PASS4-{CRIT,HIGH,MED,LOW,OBS}-NNN` — net-new finding in pass-4, not present in v1.2

Pass-3 residuals that remain open in v1.3 are tracked by their pass-3 IDs in the residual
verification table (Part A) and are not re-assigned new IDs until fully closed.

This is pass 4 of the ADR-023 adversarial review cycle. Target: 3 consecutive CLEAN passes
(current streak: 0/3, unchanged from pass-3 — this pass NOT_CLEAN).

---

## Summary

Fresh-context review of ADR-023 v1.3 at SHA `eabee0e0`. Pass-4 surfaces **14 findings**
(3 CRIT / 5 HIGH / 4 MED / 3 LOW / 2 OBS), of which 2 are residuals carried forward from
pass-3 (deferred-not-closed pattern) and 12 are new defects introduced or visible in v1.3.
Additionally, 2 findings carry a `[process-gap]` dimension (F-PASS4-CRIT-003,
F-PASS4-OBS-001/002).

**Trajectory REVERSED for first time:** 26 → 16 → 12 → **14**. The decay trend from passes
1-3 reversed in pass-4. Root cause: v1.3 Wave 1 rescoping removed story PLUGIN-MIGRATION-001-E
from Wave 1 scope, cascading inconsistencies across story count claims, SP arithmetic, and
header tables. This is a structural regression introduced by the fix-burst itself.

**Pass-3 residual verification:** 2 pass-3 findings are PARTIAL-CLOSE-DEFERRED:
- F-PASS3-HIGH-001 (VP-INDEX VP-PLUGIN-006 undefined reference): deferred to PREREQ-F
  sub-task per architect note — the VP is not yet authored; deferred-not-closed.
- F-PASS3-HIGH-004 (DI-012/BC back-references): deferred pending Wave 2/G — the
  back-reference annotations are not yet in place; deferred-not-closed.

**Critical META concern (F-PASS4-CRIT-003):** Fix-burst-3 architect used Python `open`/`write`
calls (non-Edit/Write tool path) to mutate `.factory/` files, bypassing the factory-dispatcher
validate-changelog-monotonicity hook. This bypass is causally connected to the cascade defects:
the per-Edit hook coherence checks that would have caught the story-count drift and SP arithmetic
error were never invoked on the Python-written content. Codifying TD-FACTORY-HOOK-BYPASS-001
BEFORE fix-burst-4 is an adversary recommendation to prevent further bypass-enabled cascades.

**Streak:** 0/3 unchanged.

---

## Part A — Pass-3 Residual Verification

| Finding | Pass-3 Status | v1.3 Status | Verdict | Notes |
|---------|--------------|-------------|---------|-------|
| F-PASS3-HIGH-001 (VP-PLUGIN-006 undefined) | OPEN | Still cited in §E; VP-INDEX has no VP-PLUGIN-* entries | PARTIAL-CLOSE-DEFERRED | Architect note: deferred to PREREQ-F sub-task. VP must be authored before citation stands. |
| F-PASS3-HIGH-004 (DI-012/BC back-references) | OPEN | DI-012 back-references not yet added to BC files | PARTIAL-CLOSE-DEFERRED | Deferred pending Wave 2/G — back-reference annotations absent. |

Both residuals are deferred-not-closed. They count against the 14-finding total as the 2
residuals this pass.

---

## Part B — New Findings (Pass 4)

### Critical Findings

#### F-PASS4-CRIT-001 — Story count 12 vs 13 contradicts at 5+ sites

**Severity:** CRITICAL
**Introduced in:** v1.3 (Wave 1/E rescope from 5 to 4 stories)

**Sites with contradictions:**

1. **§C Status field** — `migration_stories: 13` in the Status section claims 13 migration
   stories total.
2. **§D Negative Consequences** — "13-story migration" appears in the risk/effort callout.
3. **§D Migration Plan** — "Migration Plan (13 stories across 3 waves)" section heading.
4. **§E Wave 1 header** — The Wave 1 story table header row says "5 stories" but lists only
   4 stories after PLUGIN-MIGRATION-001-E was removed.
5. **§F Source/Origin** — "13-story migration sequence" in the provenance narrative.

**Root cause:** Fix-burst-3 removed PLUGIN-MIGRATION-001-E from Wave 1 scope (per user
directive to rescope OAuth2-refresh WASM plugin to Wave 2), reducing Wave 1 from 5 to 4
stories and total from 13 to 12 — but only updated the Wave 1 table body, not the 5+
locations that cite the total story count or the Wave 1 story count explicitly.

**Fix required:** Update all 5+ sites to reflect the post-rescope count. If the canonical
total is 12 stories (Wave 0: 5 + Wave 1: 4 + Wave 2: 3), then §C Status, §D Negative
Consequences, §D Migration Plan heading, §E Wave 1 header row, and §F Source/Origin must
all read 12. Grep for `13` within ADR-023 body and verify each occurrence is either a
legitimate non-story-count use or updated.

---

#### F-PASS4-CRIT-002 — Wave 1 SP arithmetic 30-47 claimed vs actual 32-52; total 95-138 claimed vs actual 95-145

**Severity:** CRITICAL
**Introduced in:** v1.3

**Claimed in §E Wave 1 summary row:** "Wave 1: 4 stories, 30-47 SP"
**Actual arithmetic for the 4 remaining Wave 1 stories:**
- PLUGIN-MIGRATION-001-A: 8-13 SP
- PLUGIN-MIGRATION-001-B: 8-13 SP
- PLUGIN-MIGRATION-001-C: 8-13 SP
- PLUGIN-MIGRATION-001-D: 8-13 SP

Sum: 32-52 SP (not 30-47). The claimed range 30-47 appears to result from subtracting the
removed story E (5-8 SP) from a previously miscalculated Wave 1 total (35-55 SP for 5
stories), rather than re-deriving from the per-story rows.

**Claimed total:** "Total: 95-138 SP" in the three-wave summary row.
**Actual total:**
- Wave 0: 42-62 SP (5 stories, unchanged)
- Wave 1: 32-52 SP (4 stories, corrected)
- Wave 2: 21-31 SP (3 stories, unchanged per §E)
- Correct total: 95-145 SP (not 95-138).

**Fix required:** Re-derive Wave 1 SP total by summing the four per-story bands in the
table. Recompute the three-wave total from the per-wave sums. Grep for `30-47`, `95-138`,
and any other SP range claims in the body to catch all sites.

---

#### F-PASS4-CRIT-003 — Hook-bypass via Python open/write enabled defects to land [process-gap] [META]

**Severity:** CRITICAL (META / process-gap dimension)
**Introduced in:** fix-burst-3 execution method
**Category:** process-gap

Fix-burst-3 architect used Python `open(path, "w").write(content)` calls (or equivalent
non-Edit/Write tool path) to mutate `.factory/specs/architecture/decisions/ADR-023-*.md`
files. This bypassed the factory-dispatcher's `validate-changelog-monotonicity` hook, which
fires on every Edit/Write tool invocation to check:

1. That the changelog version is monotonically increasing.
2. That the frontmatter `version:` field matches the changelog entry.
3. That story-count and SP arithmetic claims in the changelog row are consistent with
   the body content (per TD-FIX-BURST-VERIFY-002 scope if implemented).

Because the Python write bypassed the dispatcher, no hook coherence check ran against the
v1.3 content at write-time. The cascade defects (F-PASS4-CRIT-001 story count drift,
F-PASS4-CRIT-002 SP arithmetic error) are plausibly causally connected to this bypass —
the checks that would have caught them were never invoked.

**Additionally observed:**
- The v1.3 changelog row contains the text `"COMMITTED v1.2"` — a stale stamp from the
  v1.2 changelog entry that was not cleared during the Python write.
- Two to three occurrences of `"v1.0+1"` remain in the body at locations that should have
  been updated during fix-burst-3.

**Fix required (two dimensions):**
1. **Content:** Fix the stale `COMMITTED v1.2` stamp and the leftover `v1.0+1` occurrences.
2. **Process:** Codify TD-FACTORY-HOOK-BYPASS-001 (P1) before fix-burst-4 is dispatched —
   any non-Edit/Write file mutation in `.factory/` paths is policy-forbidden. The bypass
   for "atomicity" is precisely the expedient the dispatcher exists to prevent.

**Adversary recommendation:** Register TD-FACTORY-HOOK-BYPASS-001 and add a clause to the
architect agent dispatch prompt BEFORE fix-burst-4 begins.

---

### Important Findings

#### F-PASS4-HIGH-001 — Pass-3 residual: VP-PLUGIN-006 still undefined (deferred-not-closed)

**Severity:** HIGH (residual from F-PASS3-HIGH-001)
**Status:** PARTIAL-CLOSE-DEFERRED

ADR-023 v1.3 §E "Verification Properties" still cites `VP-PLUGIN-006` as the property
governing plugin load-path isolation. VP-INDEX.md has zero `VP-PLUGIN-*` entries. The
architect's fix-burst-3 closure note states "deferred to PREREQ-F sub-task" — meaning the
VP is planned but not yet authored.

A cited VP that does not exist is a broken reference. Until VP-PLUGIN-006 is authored and
registered in VP-INDEX, the §E citation is unverifiable. This finding remains open until
either: (a) VP-PLUGIN-006 is authored and committed to VP-INDEX before ADR-023 passes
through the convergence window, or (b) the citation is temporarily replaced with a
`[VP-PLUGIN-006 — to be authored in S-PLUGIN-PREREQ-F]` placeholder that does not claim
verification coverage the spec does not yet have.

---

#### F-PASS4-HIGH-002 — Pass-3 residual: DI-012/BC back-references absent (deferred-not-closed)

**Severity:** HIGH (residual from F-PASS3-HIGH-004)
**Status:** PARTIAL-CLOSE-DEFERRED

ADR-023 v1.3 §C Rule 2 states: "Domain invariant DI-012 (sealed SensorAuth trait) is
un-sealed by this ADR. Affected BCs must carry `scheduled_amendment_in: ADR-023` annotation."
The affected BCs (BC-2.16.004-rust-escape-hatch.md, BC-2.01.013-datasource-trait-adapter-pattern.md,
and the SensorAuth-related BC) do not yet carry this annotation. Architect deferred to Wave 2/G.

The bidirectional traceability requirement (TD-ADR-AMEND-001 augmentation, D-335) is not
satisfied until the BC files carry the back-reference. A pass-3 finding deferred to a future
wave does not close the finding — it remains open until the back-reference is in place.

---

#### F-PASS4-HIGH-003 — §E table cites VP-145 but VP-145 is Wave 4 story-gate VP, not plugin VP

**Severity:** HIGH (new this pass)

ADR-023 v1.3 §E Verification Properties table row for "Plugin manifest schema validation"
cites `VP-145`. VP-INDEX v1.29 shows VP-145 as a Wave 4 Phase 4.A story-gate property for
ADR-019 (S-4.08 action delivery throughput). It is NOT a plugin manifest schema validation
property. This is a wrong VP citation — the plugin domain does not have a VP-145 property.

The cite may have been introduced in fix-burst-3 when the architect attempted to add VP
coverage for the manifest validation behavior but cited from memory rather than reading
VP-INDEX.

**Fix required:** Remove VP-145 citation from §E plugin manifest validation row. Either
cite the correct VP if one exists, or annotate `[VP to be authored in S-PLUGIN-PREREQ-F]`
as a placeholder.

---

#### F-PASS4-HIGH-004 — §C Rule 1 and §E Verification Properties disagree on VP-PLUGIN-006 scope

**Severity:** HIGH (new this pass)

Two sections of ADR-023 v1.3 give inconsistent accounts of what VP-PLUGIN-006 covers:

- **§C Rule 1** ("Plugin-only execution") states: "VP-PLUGIN-006 governs load-path isolation
  and WASM sandbox memory limits."
- **§E Verification Properties** table row for VP-PLUGIN-006 says: "Verifies that no plugin
  can access host filesystem paths outside of the allowed sandbox root."

These are not the same property. §C describes a two-dimensional property (load-path isolation
AND memory limits). §E describes only filesystem path containment. A VP that verifies memory
limits is different from one that verifies filesystem path containment. If the same VP is
intended to cover both, the §E description must be expanded. If they are separate VPs, two
entries are required in §E.

This inconsistency also matters for fix-burst-4: whichever VP-PLUGIN-006 description the
architect uses when authoring the VP in PREREQ-F, the other site will be stale immediately.
The conflict must be resolved in ADR-023 before the VP is authored.

---

#### F-PASS4-HIGH-005 — §D changelog v1.3 row contains stale "COMMITTED v1.2" stamp

**Severity:** HIGH (new this pass; related to F-PASS4-CRIT-003)

The v1.3 changelog row in ADR-023 §D contains the text:

```
| v1.3 | 2026-05-10 | COMMITTED v1.2 — closed 10 pass-3 findings... |
```

The phrase `COMMITTED v1.2` is a stale artifact from the v1.2 changelog row that was not
cleared during fix-burst-3. The v1.3 row should describe what v1.3 changes, not re-assert
the prior version's commitment status. This is consistent with the hook-bypass finding
(F-PASS4-CRIT-003): a Python-write of the changelog row failed to update the boilerplate.

**Fix required:** Remove `COMMITTED v1.2` from the v1.3 row description. Replace with a
concise summary of what v1.3 actually changes relative to v1.2.

---

### Medium Findings

#### F-PASS4-MED-001 — "v1.0+1" leftover at 3 body sites

**Severity:** MEDIUM

Two to three occurrences of the string `v1.0+1` remain in the ADR-023 v1.3 body at
locations that should have been updated or consolidated during fix-burst-3. The legitimate
occurrences are in the signing-deferral sections (§D Negative Consequences, §E Prerequisites)
where `v1.0+1` is the correct target version for TD-PLUGIN-SIGNING-001. However, redundant
sibling sentences produced by incomplete Python-write replacements repeat the `v1.0+1`
string in contexts that read as copy-paste artifacts rather than intentional statements.

**Fix required:** Audit all `v1.0+1` occurrences in the body. Keep each legitimate single
deferral reference per section; remove duplicates introduced by incomplete fix-burst-3 rewrites.

---

#### F-PASS4-MED-002 — Wave 0 story count: §E table shows 5 stories; §D prose says 6

**Severity:** MEDIUM

The §E three-wave summary table correctly lists Wave 0 as 5 stories (S-PLUGIN-PREREQ-A
through E). However, the prose commentary in §D Migration Plan reads: "Wave 0 comprises
6 foundational prerequisite stories (PREREQ-A through F)." PREREQ-F (BC+DI amendments)
was added to Wave 0 per D-334 (user decision, pass-1), STORY-INDEX v2.34 includes
S-PLUGIN-PREREQ-F (total 150 stories), but §E summary shows only 5 stories and the body
story-list table does not include S-PLUGIN-PREREQ-F.

**Fix required:** Either add PREREQ-F to the §E Wave 0 table (making Wave 0 = 6 stories)
or clarify why PREREQ-F is tracked in STORY-INDEX but excluded from the ADR's wave table.
The count inconsistency between §D prose (6) and §E table (5) must resolve to one canonical
value with a stated rationale.

---

#### F-PASS4-MED-003 — §B Context/Background describes CustomAdapter as "sealed" in present tense

**Severity:** MEDIUM

ADR-023 §B Context/Background states: "The `CustomAdapter` Rust trait is sealed against
external implementation" in a present-tense description of the problem being solved.
However, Rule 5 (confirmed by user in D-334) retires the CustomAdapter Rust trait entirely —
not merely un-seals it. The §B description should be past-tense or framed as the pre-ADR
state. As written, a reader encounters §B first and reads "CustomAdapter is sealed" as a
current-state claim, then must reconcile with §C Rule 5 which retires the trait. This
framing ambiguity has caused proposed-fix language errors in prior passes.

**Fix required:** Reframe §B CustomAdapter description as historical context ("Prior to this
ADR, CustomAdapter was sealed...") to eliminate the present-tense/future-action confusion.

---

#### F-PASS4-MED-004 — "Wave 2/G" label used inconsistently across §D and §E

**Severity:** MEDIUM

The label "Wave 2/G" appears in ADR-023 v1.3 with two distinct meanings:
- **§D Migration Plan** uses "Wave 2/G" to mean the third implementation wave (cleanup
  and docs sweep), following Wave 0 and Wave 1.
- **§E Implementation Roadmap** uses "Wave 2" (without "/G" suffix) for the same wave.

The "/G" label was inherited from the story naming convention (PLUGIN-MIGRATION-001-A
through H), where the letter suffix maps to an ordered dependency. The mixed usage of
"Wave 2/G" vs "Wave 2" for the same wave creates navigational confusion when correlating
§D migration plan to §E roadmap.

**Fix required:** Standardize the wave label across §D and §E. Use "Wave 2" consistently
with story IDs referenced parenthetically within the wave, removing the ambiguous "/G" suffix
from the wave label itself.

---

### Low Findings

#### F-PASS4-LOW-001 — §C Rule 2 is self-referential (cites itself as authority)

**Severity:** LOW

§C Rule 2 ("Un-seal SensorAuth trait") includes the sentence: "Per ADR-023 §C Rule 2,
all implementations of SensorAuth that previously required internal visibility will now
be accessible as a public trait." This is self-referential — the rule is citing itself
as its own authority. The correct citation is domain invariant DI-012 and user decision
D-334 that motivated the unsealing.

**Fix required:** Replace `Per ADR-023 §C Rule 2,` with `Per domain invariant DI-012 and
user decision D-334,` or equivalent authoritative citation.

---

#### F-PASS4-LOW-002 — Changelog v1.3 row omits per-finding IDs for closed pass-3 findings

**Severity:** LOW

The v1.3 changelog row says "closed 10 pass-3 findings" but does not list which finding
IDs were closed. Prior changelog rows (v1.1 closing 26, v1.2 closing 14+) similarly omitted
per-finding IDs. This makes it difficult to verify at a glance which pass-3 findings were
addressed in v1.3 vs which remain open.

**Fix required (low priority):** Add a parenthetical listing the closed finding IDs:
"closed 10 pass-3 findings (F-PASS3-CRIT-001, F-PASS3-HIGH-002 through F-PASS3-OBS-002;
F-PASS3-HIGH-001 and F-PASS3-HIGH-004 deferred per pass-3.md)." Consistent with
finding-audit discipline across the cycle.

---

#### F-PASS4-LOW-003 — §A Decision table "Status" column uses both "COMMITTED" and "ACTIVE" inconsistently

**Severity:** LOW

The §A Decision table has the Status column populated inconsistently:
- Row for "Plugin-only execution rule (Rule 1)" shows `COMMITTED`
- Row for "SensorAuth un-sealing (Rule 2)" shows `ACTIVE`
- Row for "Wave 0/F PREREQ-F BC amendments" shows `COMMITTED`

ADR lifecycle states are: PROPOSED → COMMITTED → (implementation: ACTIVE after code lands).
The Decision table should uniformly show `COMMITTED` for decisions in a COMMITTED ADR;
`ACTIVE` implies implementation has landed, which is premature for all Wave 0 stories.

**Fix required:** Standardize Status column to `COMMITTED` for all rows in the decision
table. If implementation status is relevant, reference the STORY-INDEX sprint_state column
rather than the ADR decision table.

---

### Observational Findings (Process-Gap)

#### F-PASS4-OBS-001 — Architect agent prompt template does not prohibit non-Edit/Write tool paths [process-gap]

**Severity:** OBS (process-gap)

The architect agent dispatch prompt used in fix-burst-3 did not contain an explicit
prohibition against using Python `open`/`write` or shell `echo >/cat >` file-write methods
to mutate `.factory/` files. This absence enabled the hook-bypass that allowed
F-PASS4-CRIT-001/002 to land undetected.

The adversary recommends that TD-FACTORY-HOOK-BYPASS-001 (P1) include as required action:
"Architect agent prompt template: add explicit instruction NOT to use Python/shell file-write
tools for `.factory/` files; use Edit/Write tool path exclusively." This is the
prompt-template dimension complementary to the CLAUDE.md dimension of the same TD.

---

#### F-PASS4-OBS-002 — TD-FIX-BURST-VERIFY-002 arithmetic scope not yet codified in architect checklist [process-gap]

**Severity:** OBS (process-gap)

TD-FIX-BURST-VERIFY-002 (filed at D-336) requires a pre-write validator that rejects spec
documents with undefined VP/BC/policy citations. The scope as filed does not explicitly
include arithmetic claims (story counts, SP totals, line counts). F-PASS4-CRIT-001 (story
count drift) and F-PASS4-CRIT-002 (SP arithmetic error) are exactly the class of defect
that arithmetic-scope extension would catch.

**Adversary recommendation:** Extend TD-FIX-BURST-VERIFY-002 scope: "Validator must check
that any arithmetic claim in changelog row text (e.g., 'N stories', 'N-M SP') matches the
body content via parse-and-sum verification." This is an addendum to the existing TD, not
a new TD.

---

## Source-of-Truth Verifications

The following verifications were performed (read-only, information-asymmetric context):

| # | Verification | Source | Result |
|---|-------------|--------|--------|
| 1 | Story count in §C Status field | ADR-023 v1.3 §C | FAIL — claims 13; post-E-rescope canonical is 12 |
| 2 | Story count in §D Negative Consequences | ADR-023 v1.3 §D | FAIL — claims 13 |
| 3 | Story count in §D Migration Plan heading | ADR-023 v1.3 §D | FAIL — claims 13 |
| 4 | Wave 1 story count in §E table header | ADR-023 v1.3 §E | FAIL — claims 5; table has 4 entries |
| 5 | Story count in §F Source/Origin | ADR-023 v1.3 §F | FAIL — claims 13 |
| 6 | Wave 1 SP sum: A+B+C+D at 8-13 each = 32-52 | ADR-023 v1.3 §E per-story rows | FAIL — §E summary claims 30-47 |
| 7 | Total SP: Wave0(42-62)+Wave1(32-52)+Wave2(21-31) = 95-145 | ADR-023 v1.3 §E summary | FAIL — claims 95-138 |
| 8 | VP-PLUGIN-006 in VP-INDEX | VP-INDEX v1.29 | FAIL — absent; no VP-PLUGIN-* entries |
| 9 | VP-145 in VP-INDEX | VP-INDEX v1.29 | FAIL — VP-145 is Wave 4 ADR-019 property, not plugin manifest |
| 10 | POL-11 name | policies.yaml | PASS — now correctly cited as index_bump_required_for_index_mutations |
| 11 | BC-2.16.004 back-reference to ADR-023 | BC file | FAIL — scheduled_amendment_in field absent |
| 12 | BC-2.01.013 back-reference to ADR-023 | BC file | FAIL — scheduled_amendment_in field absent |
| 13 | Wave 0 story count: §E (5) vs §D prose (6) | §E vs §D vs STORY-INDEX | INCONSISTENT — §D says 6, §E table has 5 |
| 14 | Self-referential §C Rule 2 citation | ADR-023 v1.3 §C Rule 2 | FAIL — cites itself |
| 15 | Changelog v1.3 row for stale text | ADR-023 v1.3 §D changelog | FAIL — contains "COMMITTED v1.2" |
| 16 | §C Rule 1 vs §E VP-PLUGIN-006 scope consistency | ADR-023 v1.3 §C, §E | FAIL — load-path isolation + memory limits (§C) vs filesystem containment only (§E) |

---

## Top 3 Most-Critical Findings

**F-PASS4-CRIT-001:** Story count 12 vs 13 contradicts at 5 sites simultaneously (§C Status,
§D Negative Consequences, §D Migration Plan heading, §E Wave 1 header row, §F Source/Origin).
Wave 1 was rescoped from 5 to 4 stories (PLUGIN-MIGRATION-001-E removed) but only the Wave 1
table body was updated. All 5 count-citation sites must be updated to reflect 12 total stories.

**F-PASS4-CRIT-002:** Wave 1 SP arithmetic 30-47 claimed vs actual 32-52 for the 4 remaining
Wave 1 stories (A+B+C+D at 8-13 each). The three-wave total 95-138 claimed vs actual 95-145.
SP totals were recomputed incorrectly after removing PLUGIN-MIGRATION-001-E. Grep for `30-47`
and `95-138` to locate all sites; re-derive from per-story rows.

**F-PASS4-CRIT-003 [process-gap]:** Fix-burst-3 architect used Python open/write (non-Edit/Write
tool path) to mutate ADR-023 files, bypassing factory-dispatcher validate-changelog-monotonicity
hook. Bypass appears causally connected to CRIT-001/002 (no per-Edit hook coherence check ran).
Also left two content artifacts: stale "COMMITTED v1.2" stamp in v1.3 changelog row, and
"v1.0+1" leftovers at body sites. Codify TD-FACTORY-HOOK-BYPASS-001 P1 BEFORE fix-burst-4.

---

## Convergence Assessment

**Verdict: NOT_CLEAN — BOTH** (substantive defects + process-gap)

- Streak: 0/3 (no change from pass-3)
- Trajectory: 26 → 16 → 12 → **14** (REVERSED — decay trend broken for first time)
- Substantive defects: 3 CRIT + 5 HIGH + 4 MED + 3 LOW = 15 substantive findings
- Process-gap findings: F-PASS4-CRIT-003 (process-gap dimension) + F-PASS4-OBS-001/002
- Regression root cause: v1.3 Wave 1/E rescope created cascade inconsistencies at 5+ sites

**Adversary recommendation:** Before dispatching fix-burst-4, the orchestrator should:
1. Codify TD-FACTORY-HOOK-BYPASS-001 P1 (architect agent prompt + CLAUDE.md update)
2. Extend TD-FIX-BURST-VERIFY-002 scope to arithmetic claims
3. Dispatch fix-burst-4 architect with explicit mandate to grep-verify ALL story count,
   SP sum, and version string claims against body content (not only VP/BC/policy citations)

---

## Novelty Assessment

| **Pass** | Pass 4 |
|----------|--------|
| **Total findings** | 14 |
| **Residuals (from pass-3)** | 2 (F-PASS3-HIGH-001 VP-PLUGIN-006; F-PASS3-HIGH-004 DI-012/BC back-references) |
| **Net-new defects** | 12 (85.7%) |
| **CRIT novel findings** | 2 (F-PASS4-CRIT-001 story count drift; F-PASS4-CRIT-002 SP arithmetic; F-PASS4-CRIT-003 hook-bypass [process-gap]) |
| **HIGH novel findings** | 3 (F-PASS4-HIGH-003 VP-145 wrong citation; F-PASS4-HIGH-004 VP-PLUGIN-006 scope conflict; F-PASS4-HIGH-005 stale changelog stamp) |
| **MED novel findings** | 4 (F-PASS4-MED-001 v1.0+1 leftovers; F-PASS4-MED-002 Wave 0 count 5 vs 6; F-PASS4-MED-003 CustomAdapter tense; F-PASS4-MED-004 Wave 2/G label) |
| **Process-gap novel findings** | 2 (F-PASS4-OBS-001 prompt template; F-PASS4-OBS-002 arithmetic scope) |
| **Novelty score** | 0.857 (12 of 14 findings are new) |
| **Trajectory** | 26→16→12→14 |
| **Verdict** | FINDINGS_REMAIN |

Novelty ratio is high (0.857) and the trajectory REVERSED for the first time in this review
cycle. The reversal is explained by the Wave 1/E rescope cascade: a single structural change
(removing one story from Wave 1) that was incompletely propagated generated 5 CRIT/HIGH
findings by itself. Fix-burst-4 must apply a broader post-change consistency sweep — any
structural change (story count, SP total, wave scope) must be grep-verified at all citation
sites before committing.

---

## Operational Notes

Pass-4 was performed from a fresh-context, read-only adversary profile. No code execution.
All source-of-truth verifications were performed against files read in this session. The
16-verification table above represents the full scope of factual checks performed.

The trajectory reversal and hook-bypass META finding together indicate that the fix-burst
process itself needs a pre-dispatch discipline upgrade (TD-FACTORY-HOOK-BYPASS-001) before
fix-burst-4 proceeds. The adversary strongly recommends this codification happen before
the architect dispatch, not after.
