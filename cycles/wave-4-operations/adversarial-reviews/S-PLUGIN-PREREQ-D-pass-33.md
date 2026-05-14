---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 33
target_sha: 95d46be2
story_content_sha: ebbf241c07295f785a464cdf7ba0eaf57c38a9f6
error_taxonomy_content_sha: 2e6af6997d6c2d9a239f725afd22877ac7823e8c
bc_content_sha: 898ad6282b8f514e5b378b483932ea40f3a05a2c
base_sha: 95d46be2
verdict: BLOCKED
streak: "0/3 HOLD (pass-33 BLOCKED: 0 CRIT + 0 HIGH + 2 MED + 1 LOW + 2 OBS)"
finding_summary: {CRITICAL: 0, HIGH: 0, MEDIUM: 2, LOW: 1, OBS: 2}
prior_passes: [pass-1, pass-2, pass-3, pass-4, pass-5, pass-6, pass-7, pass-8, pass-9, pass-10, pass-11, pass-12, pass-13, pass-14, pass-15, pass-16, pass-17, pass-18, pass-19, pass-20, pass-21, pass-22, pass-23, pass-24, pass-25, pass-26, pass-27, pass-28, pass-29, pass-30, pass-31, pass-32]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3, fix-burst-4, fix-burst-5, fix-burst-6, fix-burst-7, fix-burst-8, fix-burst-9, fix-burst-10, fix-burst-11, fix-burst-12, fix-burst-13, fix-burst-14, fix-burst-15, fix-burst-16, fix-burst-17, fix-burst-18, fix-burst-19, fix-burst-20, fix-burst-21, fix-burst-22, fix-burst-23, fix-burst-24, fix-burst-25, fix-burst-26, fix-burst-27, fix-burst-28, fix-burst-29, fix-burst-30]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4 → 5 → 1 → 1 → 3 → 4 → 5"
idempotency_check: false
post_fix_check: true
post_fix_target: "fix-burst-30 (F-LP32-CRIT-001 Path A + F-LP32-MED-001/002 — all 3 in-scope closed)"
trajectory_note: "Pass-33 increased from 4 → 5 findings (2 MED + 1 LOW + 2 process-gap OBS; no CRIT/HIGH). Recurring pattern class: version-pin sibling-prose drift (8th instance; codification #11 + TD-VSDD-060 miss). Error-message delimiter drift (2nd consecutive; fix-burst-29 closure claim was byte-non-verbatim). Codifications #11-#17 all HELD on prior classes but two recurring classes persisted. F-LP33-LOW-001 'catalog discipline' phantom-section phrasing (8 sites) routes story-writer per prism canonical production-grade principle (no LOW deferral). Two process-gap OBS promote codification candidates: POL-23 (BC-version sibling sweep gate) and codification #16 formal promotion to POL-24."
producer: "adversary (vsdd-factory; reified by state-manager per established cascade convention)"
---

# Adversarial Pass 33 — S-PLUGIN-PREREQ-D

**Verdict: BLOCKED (0 CRIT + 0 HIGH + 2 MED + 1 LOW + 2 OBS)**

**Context:** This is a post-fix-burst-30 fresh-context pass. Fix-burst-30 closed 3 in-scope findings
(F-LP32-CRIT-001 Path A + F-LP32-MED-001/002) via multi-agent parallel dispatch (story-writer +
product-owner + state-manager). The expected outcome was CLEAN (0/3 → 1/3). Actual: BLOCKED by
2 MED + 1 LOW + 2 process-gap OBS. Net in-scope actionable: 3 findings (2 MED + 1 LOW). Streak
holds at 0/3 per BC-5.39.001.

Trajectory pass-25..pass-33: 4 → 1 → 4 → 5 → 1 → 1 → 3 → 4 → **5** — third consecutive pass
with 3+ findings. Cause analysis: two recurring class failures (version-pin sibling-prose drift + error
message delimiter form) rather than new architectural drift. Specifically:

F-LP33-MED-001 is the 8th recurrence of the lexical-vs-semantic version-pin drift class. Fix-burst-30
closed story line 419 (v1.5→v1.7 for AC-9 block-quote); fix-burst-29 already closed line 373
(v1.5→v1.6 as a sibling-catch). However, line 373 was updated to v1.6, and BC-2.17.002 subsequently
advanced to v1.7 at fix-burst-30. Line 373 was never swept for the v1.6→v1.7 bump. This is the
canonical sibling-site-not-swept pattern (TD-VSDD-060) applied to version pins — codification #11
(lexical-vs-semantic) + TD-VSDD-060 discipline both required a sweep of ALL `BC-2.17.002 v[0-9]`
occurrences on every BC version bump, which did not occur.

F-LP33-MED-002 is the 2nd consecutive pass (passes 32 + 33) triggering the codification #16-candidate
(error message template verbatim sweep). Fix-burst-29 F-LP31-HIGH-001 aligned E-PLUGIN-013/014 message
templates, and pass-32 codification #16 check PASSED. However, the story retains two variant forms of
the same E-PLUGIN-013 message template body: single-quoted at line 906 and no-delimiter at line 323,
while the canonical error-taxonomy.md:455 uses backtick fencing. Pass-32 codification #16 check
verified only the three canonical three-site alignment (§Error Taxonomy Additions vs AC-5 vs
error-taxonomy.md) and PASSED — but that check operates on the main §Error Taxonomy Additions table
row, not on all prose occurrences of the message body in the story. Lines 906 and 323 are prose
references in AC-text, not §Error Taxonomy Additions table rows, which is why they escaped the
pass-32 check.

F-LP33-LOW-001 (8-site "catalog discipline" phantom-section phrasing) routes to story-writer per
prism canonical production-grade principle: no LOW deferral. The LOW designation reflects impact
severity (implementer confusion, not spec correctness), not fix-urgency. Fix-burst-31 includes this
in scope alongside the 2 MED fixes.

---

## Codification Regression Checks (#11–#17 + #13 Sub-Extension)

All active codification disciplines verified against story v1.30 (SHA ebbf241c).

### Codification #11 — Lexical-vs-Semantic Anchor-Content Verification

**Target:** Every POL-22 Phase A anchor citation must be confirmed by opening and grepping
the cited document, not by story-body substring matching alone.

Applied to all 30+ cited anchors in this pass. All verified by semantic open-and-grep:
BC-2.16.002 §Canonical Structured Event Catalog: section present — PASS. ADR-023 §C4: section
present — PASS. BC-2.17.001..007 H1 titles verified — PASS (8 anchors). VP-PLUGIN-004/007
entries in VP-INDEX — PASS. BC-2.22.001 §Boot Sequence Steps — PASS.

EXCEPTION — codification #11 catches F-LP33-MED-001: AC-9 trace header at story line 373 reads
`BC-2.17.002 v1.6`. Semantic verification: BC-2.17.002 current version is v1.7 (fix-burst-30;
D-528 PATH A amendment). Version pin v1.6 refers to the intermediate post-fix-burst-29 state;
v1.7 is the canonical current version with EC-17-007 phantom variant removed. Story line 373 was
updated from v1.5 to v1.6 as a sibling-catch in fix-burst-29 (D-526) but was not swept again
when BC advanced to v1.7 in fix-burst-30 (D-528). This is a v1.6→v1.7 pin lag at one prose site.

**Codification #11: FAIL — 1 stale version pin (BC-2.17.002 v1.6 at story line 373 vs canonical v1.7). All other 30+ anchors PASS.**

### Codification #12 — BC Body-Table Title Verbatim Verification (POL-22 Phase B)

**Target:** Every BC body-table Title cell must match BC H1 verbatim (whitespace-normalized).

9 BC rows in body BC table verified (story v1.30):

| BC | Body-Table Title | Result |
|----|-----------------|--------|
| BC-2.16.002 | "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation" | PASS |
| BC-2.17.001 | verbatim BC H1 | PASS |
| BC-2.17.002 | verbatim BC H1 | PASS |
| BC-2.17.003 | verbatim BC H1 | PASS |
| BC-2.17.004 | verbatim BC H1 | PASS |
| BC-2.17.006 | verbatim BC H1 | PASS |
| BC-2.17.007 | verbatim BC H1 (parenthetical annotation preserved) | PASS |
| BC-2.22.001 | verbatim BC H1 | PASS |

**Codification #12: HELD — 8/8 BC body-table Title cells verbatim.**

### Codification #13 — POL-7 Cross-Table Sweep (BC Title Verbatim at ALL Citation Sites)

**Target:** Every BC-NNN.NNN citation in the story must have verbatim BC H1 title at all
citation sites (body BC table, §References, Architecture Compliance Rules, exclusion-note
paragraphs, prose).

Phase B extended verification — 5-chain sample:

| Chain | BC | Body BC Table | §References | Exclusion-Note / Prose | Result |
|-------|-----|--------------|-------------|------------------------|--------|
| 1 | BC-2.16.002 | PASS (verbatim) | PASS (fix-burst-28) | N/A | **PASS** |
| 2 | BC-2.17.001 | PASS | PASS | N/A | PASS |
| 3 | BC-2.17.005 | N/A | PASS (line 1016) | PASS (line 269 — fix-burst-27) | PASS |
| 4 | BC-2.17.002 | PASS | PASS | N/A | PASS |
| 5 | BC-2.22.001 | PASS | PASS | N/A | PASS |

**Codification #13: HELD — all BC title citation sites verbatim.**

### Codification #13 Sub-Extension — §References Completeness Check

**Target:** All members of `behavioral_contracts:` frontmatter array must appear in §References.

Frontmatter `behavioral_contracts:` members (8 entries): BC-2.16.002, BC-2.17.001,
BC-2.17.002, BC-2.17.003, BC-2.17.004, BC-2.17.006, BC-2.17.007, BC-2.22.001.

§References BC entries (post fix-burst-28 + fix-burst-29 + fix-burst-30): 9 entries
(BC-2.17.005 correctly present per codification #15 design — cited in exclusion-note paragraph).
All 8 frontmatter members confirmed present in §References.

**Codification #13 sub-extension: HELD.**

### Codification #14 — Phantom-Section-Anchor Sweep

**Target:** Every §X notation in the story that cites a BC or ADR must resolve to an actual
section heading in the cited document.

All §X notations verified:
- Story line 918 + 260 + 466: BC-2.16.002 §Canonical Structured Event Catalog row
  pipeline_max_requests_exceeded — section exists: PASS
- ADR-023 §C4: section C4 exists — PASS
- BC-2.17.002 §Error Conditions E-PLUGIN-005: section present — PASS
- BC-2.17.002 v1.12 §Canonical Structured Event Catalog: PASS
- All other §X notations: PASS

EXCEPTION — F-LP33-LOW-001: story cites "BC-2.16.002 v1.12 catalog discipline" at 8 sites
(lines 300, 357, 581, 616, 648, 692, 808, 916). The phrase "catalog discipline" is not a
named section in BC-2.16.002 v1.12. It implies a section that does not exist. The actual
named section is §Canonical Structured Event Catalog; the routing rule (audit-channel via
event_type) lives only in the Trigger column of the `plugin_load_unsigned` row at line 94.
This is a borderline codification #14 violation (phantom-section-by-implication).

**Codification #14: PARTIAL FAIL — 8-site phantom-section-by-implication ("catalog discipline");
all §X notations with explicit section anchors PASS.**

### Codification #15 — Sibling-Prose-Not-Swept Exclusion-Note (POL-7 Extension)

**Target:** BCs cited in exclusion-note paragraphs must also have verbatim titles.

Story line 269 (exclusion-note): BC-2.17.005 title verbatim match (fix-burst-27 applied) — PASS.

**Codification #15: HELD.**

### Codification #16 — Verbatim Cross-Table Sweep for Error Message Template Text

**Target (candidate, not yet formally codified):** Error message strings in §Error Taxonomy
Additions table must be verbatim-consistent across: (a) AC-5 body text, (b) error-taxonomy.md
E-PLUGIN-NNN rows, (c) §Error Taxonomy Additions table.

§Error Taxonomy Additions table row verification (3-site check):
- E-PLUGIN-013: table row vs AC-5 vs error-taxonomy.md:455 — PASS (pass-32 fix-burst-29 aligned)
- E-PLUGIN-014: table row vs AC-5 vs error-taxonomy.md — PASS
- E-PLUGIN-015/016: unchanged — PASS
- E-PIPELINE-001: unchanged — PASS

**Codification #16 table-row check: PASS (all §Error Taxonomy Additions table rows verbatim).**

EXCEPTION — codification #16 does not cover ALL prose occurrences of error message body text
in the story, only the §Error Taxonomy Additions table. F-LP33-MED-002 surfaces exactly this
gap: story line 906 contains the E-PLUGIN-013 message body in a single-quoted prose context
(`'allowed_urls = []'`) and story line 323 contains it with no delimiter (`allowed_urls = []`),
while the canonical form at error-taxonomy.md:455 uses backtick fencing
(`` `allowed_urls = []` ``). The 3-site table-row check passed; the prose-occurrence check did
not run. This demonstrates that codification #16 in its current form checks only table rows, not
all prose occurrences — the same failure mode as codification #13's original scope limitation
(body BC table only) before the cross-table sweep extension was added.

**Codification #16: HELD on table-row check; prose-occurrence gap triggered F-LP33-MED-002.
Codification #16 scope extension required (candidate → formal; prose occurrences included).**

### Codification #17 — BC-Amendment Error-Variant Existence Verification

**Target (candidate):** When any agent introduces a new named entity reference (enum variant,
error code, type name, function name) in a BC body or story spec, the cross-burst adversary MUST
grep the canonical definition location to verify existence before declaring closure CLEAN.

No new named entity references introduced at fix-burst-30. EC-17-007 was amended to REMOVE the
phantom variant reference (not introduce one). Carry-forward verification: grep for
`AllowlistRejected` in error.rs, error-taxonomy.md, and story body — ZERO matches. The phantom
variant has been fully excised.

**Codification #17: HELD — no phantom named entity references found.**

---

## POL-22 Phase A — Anchor Verification (35+ samples)

Verified 35+ story body anchor citations against target documents (semantic open-and-grep
per codification #11 discipline):

- BC-2.16.002 §Canonical Structured Event Catalog: section present — PASS
- BC-2.17.001..007 H1 titles: all verified in respective BC files — PASS (8 anchors)
- BC-2.22.001 §Boot Sequence Steps: section present — PASS
- ADR-023 §C4: section C4 present — PASS
- ADR-022 §A (exit codes), §C (runtime wiring), §D (concurrency permits): all present — PASS
- VP-PLUGIN-004 (VP-INDEX §VP-149): entry present — PASS
- VP-PLUGIN-007 (VP-INDEX §VP-152): entry present — PASS
- SS-22, SS-17, SS-16 in ARCH-INDEX: all present — PASS
- E-PLUGIN-001..016 in error-taxonomy.md: all present — PASS
- E-PIPELINE-001 in error-taxonomy.md: present — PASS
- All 25+ story §References entries: all target files verifiable — PASS
- BC-2.16.002 catalog row names verified in BC-2.16.002 §Canonical Structured Event Catalog — PASS
- `AllowlistRejected` grep: ZERO matches anywhere (phantom variant fully removed per fix-burst-30) — PASS

**FAIL — 1 version pin:** BC-2.17.002 v1.6 at story line 373 (AC-9 trace header); canonical v1.7.
Story line 373 was updated from v1.5 → v1.6 as sibling-catch at fix-burst-29, but not re-swept
when BC-2.17.002 advanced to v1.7 at fix-burst-30 (EC-17-007 phantom variant removal).

**POL-22 Phase A: 1 FAIL (stale version pin v1.6 at line 373); all other 35+ anchors PASS.**

---

## POL-22 Phase B — BC-Title Chain Verification (10 chains)

Full 10-chain verification: all BCs in `behavioral_contracts:` frontmatter array (8) plus
version-pin consistency audit at key trace headers.

| Chain | BC | Body BC Table Title | §References Title | Verbatim BC H1 | Result |
|-------|----|--------------------|--------------------|----------------|--------|
| 1 | BC-2.16.002 | PASS (verbatim) | PASS (fix-burst-28) | confirmed | **PASS** |
| 2 | BC-2.17.001 | PASS | PASS | verified | PASS |
| 3 | BC-2.17.002 | PASS | PASS | verified | PASS |
| 4 | BC-2.17.003 | PASS | PASS | verified | PASS |
| 5 | BC-2.17.004 | PASS | PASS | verified | PASS |
| 6 | BC-2.17.006 | PASS | PASS | verified | PASS |
| 7 | BC-2.17.007 | PASS | PASS (parenthetical preserved) | verified | PASS |
| 8 | BC-2.22.001 | PASS | PASS | verified | PASS |
| 9 | BC-2.17.005 | N/A | PASS (line 1016) | verified | PASS |
| 10 | BC-2.17.002 version pin chain | line 373 = v1.6; line 419 = v1.7 (fix-burst-30); canonical v1.7 | N/A | N/A | PARTIAL — line 373 stale at v1.6; line 419 correctly updated to v1.7; asymmetric sibling drift. Fix-burst-30 fixed line 419 but not line 373. |

**POL-22 Phase B: PARTIAL — 9/10 chains PASS; chain 10 PARTIAL (version-pin asymmetry: line 373
at v1.6 vs line 419 at v1.7 vs canonical v1.7). This is the 8th version-pin sibling-prose drift
instance in this cascade (passes 9, 15, 16, 19, 20, 29, 30, 32, 33).**

---

## POL-22 Phase C — Carry-Forward Regression (17+ samples)

Prior fix-burst closures 1..30 spot-checked:

| Prior Finding | Fix Applied At | Regression Check |
|---------------|---------------|-----------------|
| F-LP32-CRIT-001 (phantom AllowlistRejected Path A closure) | fix-burst-30 | PASS — AllowlistRejected ZERO matches everywhere; EC-17-007 uses E-PLUGIN-005 SandboxViolation semantics |
| F-LP32-MED-001 (AC-9 closure note line 419 stale v1.5) | fix-burst-30 | PASS — line 419 reads v1.7 (fully updated) |
| F-LP32-MED-002 (changelog rows 1.27/1.28/1.29 Burst column) | fix-burst-30 | PASS — 5-cell schema restored |
| F-LP31-HIGH-001 (E-PLUGIN-013/014 §Error Taxonomy table verbatim) | fix-burst-29 | PASS — 3-site table-row check; prose-occurrence gap (F-LP33-MED-002) is a scope-extension finding, not a regression of the table-row fix itself |
| F-LP31-HIGH-002 (BC-2.17.002 EC-17-007 default-deny) | fix-burst-29 (v1.5→v1.6) + fix-burst-30 (v1.6→v1.7) | PASS — EC-17-007 now reads "Request denied with HTTP 403 returned to plugin (existing E-PLUGIN-005 SandboxViolation semantics); audit log entry created" — semantically correct |
| F-LP30-MED-001 (§References BC-2.16.002 completeness) | fix-burst-28 | PASS — §References contains BC-2.16.002 verbatim H1 |
| F-LP29-MED-001 (exclusion-note BC-2.17.005 verbatim) | fix-burst-27 | PASS — line 269 verbatim |
| F-LP28-MED-001/002 (phantom §-section anchors) | fix-burst-26 | PASS — no phantom §S-PLUGIN-PREREQ-D sections |
| F-LP27-MED-001 (subsystems SS-16 missing) | fix-burst-25 | PASS — [SS-22, SS-17, SS-16] confirmed |
| F-LP27-MED-002 (PluginError #[non_exhaustive] MVP-hedge) | fix-burst-25 | PASS — unconditional prescription present |
| F-LP25-HIGH-001 (spawn_blocking fabricated ADR-023 §C4) | fix-burst-23 | PASS — BC-2.17.005 §Invariants anchor confirmed |

**POL-22 Phase C: PASS — all prior closures (fix-burst-1..fix-burst-30) CLEAN.
F-LP33-MED-001 is a new drift instance, not a regression of a prior fix.**

---

## POL-22 Phase D — Findings

### F-LP33-MED-001 — AC-9 trace header carries stale BC-2.17.002 version pin (v1.6 vs canonical v1.7)

**Severity:** MEDIUM
**Confidence:** HIGH
**Tags:** Codification #11 (lexical-vs-semantic) + TD-VSDD-060 (sibling-site sweep)
**Instance count:** 8th recurrence of version-pin sibling-prose drift in this cascade
(prior instances: passes 9, 15, 16, 19, 20, 29, 30, 32)

**Evidence:**

| Site | Says | Expected |
|------|------|---------|
| Story line 373 (AC-9 trace header) | `BC-2.17.002 v1.6` | `BC-2.17.002 v1.7` |
| Story line 419 (AC-9 block-quote closure note) | `v1.7` (fixed at fix-burst-30) | `v1.7` — PASS |
| BC-2.17.002 canonical current version | v1.7 (fix-burst-30 D-528 PATH A) | v1.7 |

**Root cause:** Fix-burst-29 (D-526) caught line 373 as a sibling-site via TD-VSDD-060 sweep and
updated it from v1.5 → v1.6. Fix-burst-30 (D-528) then advanced BC-2.17.002 to v1.7 and updated
line 419, but did NOT re-sweep all `BC-2.17.002 v[0-9]+\.[0-9]+` occurrences for the v1.6→v1.7
transition. The sibling-site sweep was applied during the v1.5→v1.6 bump but not the v1.6→v1.7
bump. This is the canonical "sibling-sweep applied once but not on re-bump" failure mode.

**Required fix:** story-writer edit story line 373 — replace `v1.6` with `v1.7`.

**Codification note:** This is the 8th instance of the version-pin sibling-prose drift class.
POL-23 candidate (F-LP33-OBS-001): automated gate requiring a repo-wide grep of all
`BC-ID v[0-9]+\.[0-9]+` pins whenever a BC version is bumped.

**Route:** story-writer (fix-burst-31)

---

### F-LP33-MED-002 — E-PLUGIN-013 message template delimiter drift (3 forms exist)

**Severity:** MEDIUM
**Confidence:** HIGH
**Tags:** Codification #16-candidate scope extension (prose occurrences not covered by table-row check)

**Evidence:**

| Location | Form | Content |
|----------|------|---------|
| Story line 906 (AC-text prose) | Single-quoted | `'allowed_urls = []'` |
| Story line 323 (AC-text prose) | No delimiter | `allowed_urls = []` |
| error-taxonomy.md:455 | Backtick-fenced (CANONICAL) | `` `allowed_urls = []` `` |
| Story §Error Taxonomy Additions table row | Backtick-fenced | `` `allowed_urls = []` `` — PASS (codification #16 table-row check) |

**Root cause:** Fix-burst-29 (D-526) F-LP31-HIGH-001 aligned the §Error Taxonomy Additions table
row (and AC-5 body text in that section) to the canonical backtick form. Pass-32 codification
#16 check confirmed the table-row alignment and declared closure CONFIRMED. However, the story
retains two prose AC-text occurrences of the same message body (lines 906 and 323) in non-table
context, using non-canonical delimiter forms. The codification #16 check as implemented covers
the §Error Taxonomy Additions table rows only, not all prose occurrences.

This is structurally identical to the gap that motivated codification #13's cross-table sweep
extension (codification #13 originally covered body BC table only; extended to §References +
exclusion-notes + prose). Codification #16 needs the same extension.

**Required fixes:**
- story-writer edit line 906: replace single-quoted form `'allowed_urls = []'` with
  double-backtick code-span fenced form `` `allowed_urls = []` `` (matching canonical)
- story-writer edit line 323: replace no-delimiter form `allowed_urls = []` with
  `` `allowed_urls = []` `` (matching canonical)
- No error-taxonomy.md or BC changes required.

**Route:** story-writer (fix-burst-31)

---

### F-LP33-LOW-001 — "BC-2.16.002 v1.12 catalog discipline" phrasing references no named BC section

**Severity:** LOW
**Confidence:** HIGH
**Tags:** Codification #14 (phantom-section-anchor sweep) — borderline application to implied-section
**Instance count:** First occurrence of this specific phrasing form

**Evidence — 8 sites:**

| Line | Phrasing |
|------|---------|
| 300 | `BC-2.16.002 v1.12 catalog discipline` |
| 357 | `BC-2.16.002 v1.12 catalog discipline` |
| 581 | `BC-2.16.002 v1.12 catalog discipline` |
| 616 | `BC-2.16.002 v1.12 catalog discipline` |
| 648 | `BC-2.16.002 v1.12 catalog discipline` |
| 692 | `BC-2.16.002 v1.12 catalog discipline` |
| 808 | `BC-2.16.002 v1.12 catalog discipline` |
| 916 | `BC-2.16.002 v1.12 catalog discipline` |

**Root cause:** The phrase "catalog discipline" implies a named principle or section within
BC-2.16.002 v1.12. No such section exists. The actual rule (audit-channel routing via event_type
field) lives only as a parenthetical in the Trigger column of the `plugin_load_unsigned` row at
BC-2.16.002:94. An implementer encountering "catalog discipline" as a reference would search for
a section or definition in BC-2.16.002 that does not exist.

**Required fix (8-site sweep):** story-writer replace "catalog discipline" phrasing at all 8
sites. Preferred replacement: `BC-2.16.002 v1.12 §Canonical Structured Event Catalog (row
plugin_load_unsigned Trigger cell)`. Lighter alternative: `catalog routing convention` (drops
"discipline" to eliminate the named-anchor implication while preserving the semantic intent).

**No LOW deferral per prism canonical production-grade principle.** Fix in scope at fix-burst-31.

**Route:** story-writer (fix-burst-31)

---

### F-LP33-OBS-001 — [process-gap] Persistent recurrence of lexical-vs-semantic version-pin drift

**Severity:** OBSERVATION
**Type:** Process gap / Codification candidate

**Description:** Version-pin sibling-prose drift has now manifested in 8+ distinct instances across
passes 9, 15, 16, 19, 20, 29, 30, 32, and 33 of this cascade. Each instance follows the same
pattern: a BC version bump is applied to one story cite location (or some locations), but other
prose citations of the same BC version pin are not swept.

The current controls (TD-VSDD-060 sibling-site sweep discipline) are manually applied and have
proven insufficient to prevent recurrence. Eight instances exceeds the 3-instance codification
threshold by 5.

**Proposed codification — POL-23 candidate:** Automated BC-version-bump sibling-site grep gate.
For every fix-burst that bumps a BC `version:` frontmatter field, the same fix-burst MUST:
1. Grep all dependent story files for `<BC-ID> v[0-9]+\.[0-9]+` patterns
2. Confirm zero stale version pins remain for the prior version
3. Document the sweep result in the fix-burst closure state record

This gates version-bump acceptance on an explicit sweep rather than relying on agent discipline.

**Tags:** [process-gap] — codification candidate #18 (POL-23 BC-version-bump sibling-site grep
gate). Route to orchestrator / cycle-close session-reviewer adjudication.

---

### F-LP33-OBS-002 — [process-gap] Codification #16 repeatedly partially-implemented (2nd consecutive pass)

**Severity:** OBSERVATION
**Type:** Codification candidate formal promotion

**Description:** Codification #16 (verbatim cross-table sweep for error message template text)
has triggered on two consecutive passes (32 and 33). Pass-32: the check verified §Error Taxonomy
Additions table rows and declared HELD; fix-burst-29 F-LP31-HIGH-001 closure CONFIRMED. Pass-33:
the same check verified §Error Taxonomy Additions table rows and declared PASS again, but missed
prose occurrences of the same error message body at lines 906 and 323.

The pattern is identical to codification #13's original scope limitation (covered only body BC
table; extended to §References + exclusion-notes + prose after pass-27 found #13 insufficient).
Codification #16 needs formal promotion from candidate to active codification WITH the same
scope extension: "verbatim message template check covers ALL story sections where the message
body appears, not only the §Error Taxonomy Additions table."

**Proposed codification — POL-24:** Byte-verbatim grep for ALL occurrences of each error
message template body in the story spec, confirmed against canonical error-taxonomy.md entry.
Scope: §Error Taxonomy Additions table + all AC-text prose + Summary + §Scope bullets.
Formal promotion from codification #16-candidate to active codification #16 (POL-24).

**Tags:** [process-gap] — codification #16 formal promotion + new codification candidate #19
= POL-23. Total codification candidates in adjudication queue: 19 (was 17 + codification #16
formally promoted to active + codification candidate #18 = POL-23 newly proposed).

---

## Summary

| Finding | Severity | Status | Route |
|---------|----------|--------|-------|
| F-LP33-MED-001 | MEDIUM | OPEN | story-writer — story line 373 `v1.6` → `v1.7` (8th version-pin sibling drift) |
| F-LP33-MED-002 | MEDIUM | OPEN | story-writer — line 906 single-quote → backtick + line 323 no-delim → backtick (E-PLUGIN-013 message template prose occurrences) |
| F-LP33-LOW-001 | LOW | OPEN | story-writer — 8-site "catalog discipline" phrasing replaced with explicit canonical anchor; no LOW deferral per production-grade default |
| F-LP33-OBS-001 | OBS [process-gap] | OPEN | orchestrator / cycle-close session-reviewer — codification candidate #18 POL-23 BC-version-bump sibling grep gate |
| F-LP33-OBS-002 | OBS [process-gap] | OPEN | orchestrator / cycle-close session-reviewer — codification #16 formal promotion to active (POL-24) + scope extension to prose occurrences |

**fix-burst-31 routing:** Dispatch story-writer single-agent (no BC amendments needed this pass).
- story-writer: (1) line 373 `v1.6` → `v1.7`; (2) line 906 single-quote → backtick; (3) line 323
  no-delim → backtick; (4) 8-site "catalog discipline" → canonical anchor phrasing (lines 300, 357,
  581, 616, 648, 692, 808, 916); (5) story v1.30 → v1.31 changelog row for fix-burst-31.

**Streak: 0/3 HOLD.** Pass-34 follows fix-burst-31 closure.
