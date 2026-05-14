---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 32
target_sha: b9c4edea
story_content_sha: 4d4551fb
error_taxonomy_content_sha: 2e6af699
bc_content_sha: TBD-from-BC-2.17.002-v1.6
base_sha: 95d46be2
verdict: BLOCKED
streak: "0/3 HOLD (pass-32 BLOCKED: 1 CRIT/HIGH phantom variant + 2 MED + 2 OBS)"
finding_summary: {CRITICAL: 1, HIGH: 0, MEDIUM: 2, LOW: 0, OBS: 2}
finding_summary_note: "F-LP32-CRIT-001 classified at boundary CRITICAL/HIGH per security-relevant cross-document drift; treated as HIGH for streak purposes"
prior_passes: [pass-1, pass-2, pass-3, pass-4, pass-5, pass-6, pass-7, pass-8, pass-9, pass-10, pass-11, pass-12, pass-13, pass-14, pass-15, pass-16, pass-17, pass-18, pass-19, pass-20, pass-21, pass-22, pass-23, pass-24, pass-25, pass-26, pass-27, pass-28, pass-29, pass-30, pass-31]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3, fix-burst-4, fix-burst-5, fix-burst-6, fix-burst-7, fix-burst-8, fix-burst-9, fix-burst-10, fix-burst-11, fix-burst-12, fix-burst-13, fix-burst-14, fix-burst-15, fix-burst-16, fix-burst-17, fix-burst-18, fix-burst-19, fix-burst-20, fix-burst-21, fix-burst-22, fix-burst-23, fix-burst-24, fix-burst-25, fix-burst-26, fix-burst-27, fix-burst-28, fix-burst-29]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4 → 5 → 1 → 1 → 3 → 4"
idempotency_check: false
post_fix_check: true
post_fix_target: "fix-burst-29 (F-LP31-HIGH-001/002 + F-LP31-MED-001 — closure introduced new CRITICAL drift)"
trajectory_note: "Pass-32 increased from 3 → 4 findings — second consecutive break of decreasing trend; the CRITICAL is a fix-burst-29 introduced regression (phantom variant). Demonstrates 'incomplete fix introduces new drift' anti-pattern: fix-burst-29 closed F-LP31-HIGH-002 by amending BC-2.17.002 EC-17-007, but introduced PluginError::AllowlistRejected citation that does not exist in error.rs, error-taxonomy.md, story §Error Taxonomy Additions, or AC-7. Cross-burst adversary did not verify variant existence at commit time."
producer: "adversary (vsdd-factory; reified by state-manager due to read-only tool profile)"
---

# Adversarial Pass 32 — S-PLUGIN-PREREQ-D

**Verdict: BLOCKED (1 CRITICAL/HIGH + 2 MEDIUM + 2 OBS)**

**Context:** This is a post-fix-burst-29 fresh-context pass. Fix-burst-29 closed 3 in-scope
findings (F-LP31-HIGH-001/002 + F-LP31-MED-001) plus 1 sibling-catch, via multi-agent parallel
dispatch (story-writer + product-owner + state-manager). The expected outcome was CLEAN
(0/3 → 1/3). Actual: BLOCKED by 1 CRITICAL/HIGH phantom variant + 2 MED + 2 OBS. Net actionable:
3 findings (1 CRIT/HIGH + 2 MED). Streak holds at 0/3 per BC-5.39.001.

Trajectory pass-25..pass-32: 4 → 1 → 4 → 5 → 1 → 1 → 3 → **4** — second consecutive break of
decreasing trend. Both breaks caused by fix-burst-closure-introduced drift: fix-burst-29 amended
BC-2.17.002 EC-17-007 to close F-LP31-HIGH-002 (default-deny security semantic) but the
amendment introduced `PluginError::AllowlistRejected`, a variant that does not exist anywhere in
the codebase, error taxonomy, or story spec.

F-LP32-CRIT-001 is a security-relevant cross-document drift (4-site inconsistency) introduced
by the story-writer §BC Amendments directive at story line 1008 which invented a variant name
without verification. The product-owner executed the amendment as written. The cross-burst
adversary did not verify variant existence at closure time. This demonstrates a critical gap:
when a fix introduces a NEW named entity reference (enum variant, error code, function name),
the cross-burst adversary must verify that named entity exists before declaring closure.
This gap is tagged as codification candidate #17.

**Adjudication — Path A (selected):** Amend BC-2.17.002 v1.6 → v1.7 EC-17-007 to remove the
phantom variant reference. Use existing semantics: HTTP 403 returned + E-PLUGIN-005
SandboxViolation (already covers this case at error.rs:1020-1023). Aligns BC with existing
AC-7 prescription + existing code + existing taxonomy. Zero new scope.

Path B (introduce new PluginError::AllowlistRejected variant) rejected: larger blast-radius
for security-equivalent outcome. Production-grade default per CLAUDE.md Canonical Principle
Rule 2 (feature order is the only acceptable speed lever; Path B expands scope unnecessarily).

---

## Codification Regression Checks (#11–#16 + #13 Sub-Extension)

All seven active codification disciplines verified against story v1.29 (SHA 4d4551fb).

### Codification #11 — Lexical-vs-Semantic Anchor-Content Verification

**Target:** Every POL-22 Phase A anchor citation must be confirmed by opening and grepping
the cited document, not by story-body substring matching alone.

Applied to all 30+ cited anchors in this pass. All verified by semantic open-and-grep:
BC-2.16.002 §Canonical Structured Event Catalog: section present — PASS. ADR-023 §C4: section
present — PASS. BC-2.17.001..007 H1 titles verified — PASS (8 anchors). VP-PLUGIN-004/007
entries in VP-INDEX — PASS. BC-2.22.001 §Boot Sequence Steps — PASS.

EXCEPTION — codification #11 catches the CRITICAL finding: BC-2.17.002 v1.6 EC-17-007 cites
`PluginError::AllowlistRejected`. Semantic verification: open error.rs and grep for
`AllowlistRejected` — ZERO matches. The named entity does not exist. This is the canonical
failure mode codification #11 was designed to catch.

**Codification #11: FAIL — 1 phantom named entity reference (PluginError::AllowlistRejected
in BC-2.17.002 v1.6 EC-17-007). All other anchors PASS.**

### Codification #12 — BC Body-Table Title Verbatim Verification (POL-22 Phase B)

**Target:** Every BC body-table Title cell must match BC H1 verbatim (whitespace-normalized).

9 BC rows in body BC table verified (unchanged from pass-31 — no body-table edits in
fix-burst-29):

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

§References BC entries (post fix-burst-28 + fix-burst-29): 9 entries (BC-2.17.005 correctly
present per codification #15 design — cited in exclusion-note paragraph). All 8 frontmatter
members confirmed present in §References.

**Codification #13 sub-extension: HELD.**

### Codification #14 — Phantom-Section-Anchor Sweep

**Target:** Every §X notation in the story that cites a BC or ADR must resolve to an actual
section heading in the cited document.

All §X notations verified:
- Story line 918: BC-2.16.002 §Canonical Structured Event Catalog row pipeline_max_requests_exceeded — section exists: PASS
- Story line 260: same anchor — PASS
- Story line 466: same anchor — PASS
- ADR-023 §C4: section C4 exists — PASS
- BC-2.17.002 §Error Conditions E-PLUGIN-005: section present — PASS

EXCEPTION — new §BC Amendments In-Scope directive added by fix-burst-29 at story line 1008:
references "PluginError::AllowlistRejected" in prose context (not a §X notation but a named
entity citation); this is the same phantom entity caught by codification #11 above.

**Codification #14: HELD — zero phantom §X section anchors found. (Named entity phantom
caught under codification #11.)**

### Codification #15 — Sibling-Prose-Not-Swept Exclusion-Note (POL-7 Extension)

**Target:** BCs cited in exclusion-note paragraphs must also have verbatim titles.

Story line 269 (exclusion-note): BC-2.17.005 title verbatim match (fix-burst-27 applied) — PASS.

**Codification #15: HELD.**

### Codification #16 — Verbatim Cross-Table Sweep for Error Message Template Text

**Target (pre-adjudication — candidate, not yet formally codified):** Error message strings
in §Error Taxonomy Additions table must be verbatim-consistent across: (a) AC-5 body text,
(b) error-taxonomy.md E-PLUGIN-NNN rows, (c) §Error Taxonomy Additions table.

E-PLUGIN-013 (fix-burst-29 aligned): story §Error Taxonomy Additions message vs AC-5 body
vs error-taxonomy.md — all three PASS (fix-burst-29 F-LP31-HIGH-001 applied).

E-PLUGIN-014 (fix-burst-29 aligned): same three-site verification — all PASS.

E-PLUGIN-015 (fix-burst-28 prior — unchanged): three-site verification — PASS.

E-PLUGIN-016 (fix-burst-28 prior — unchanged): three-site verification — PASS.

E-PIPELINE-001 (fix-burst-28 prior — unchanged): three-site verification — PASS.

**Codification #16: HELD (pre-adjudication) — all 5 error codes pass three-site verbatim
message check. Fix-burst-29 F-LP31-HIGH-001 closure CONFIRMED.**

---

## POL-22 Phase A — Anchor Verification (40+ samples)

Verified 40+ story body anchor citations against target documents (semantic open-and-grep
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
- All 25 story §References entries: all target files verifiable — PASS
- BC-2.16.002 catalog row names verified in BC-2.16.002 §Canonical Structured Event Catalog — PASS

**FAIL — 1 anchor:** BC-2.17.002 v1.6 EC-17-007 cites `PluginError::AllowlistRejected`:
- Grep `AllowlistRejected` in crates/prism-core/src/error.rs → ZERO matches
- PluginError enum (lines 984-1034) variants: Trapped, Timeout, MemoryExceeded, NotLoaded,
  InvalidInterface, SandboxViolation, CompilationFailed, EmptyPluginId — 8 variants, no AllowlistRejected
- Grep error-taxonomy.md for `AllowlistRejected` → ZERO matches
- Grep story §Error Taxonomy Additions for `AllowlistRejected` → ZERO matches (story
  §Error Taxonomy Additions introduces only E-PLUGIN-013/014/015/016 + E-PIPELINE-001)
- Story AC-7 prescribes HTTP 403 returned to plugin + tracing::warn! — no variant prescribed
- host_functions.rs:64-68: returns HttpResponse { status: 403, ... } — no PluginError return

**POL-22 Phase A: 1 FAIL (phantom named entity); all other 40+ anchors PASS.**

---

## POL-22 Phase B — BC-Title Chain Verification (10 chains)

Full 10-chain verification: all BCs in `behavioral_contracts:` frontmatter array (8) plus
BC-2.17.002 v1.6 version consistency.

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
| 10 | BC-2.17.002 version pin | story line 373 updated to v1.6 (fix-burst-29 sibling-catch) | v1.6 pin confirmed | N/A | PARTIAL — BC body EC-17-007 cites phantom AllowlistRejected; title chain PASS but body content FAIL |

**POL-22 Phase B: PARTIAL — 9/10 chains PASS; chain 10 PARTIAL (BC body content FAIL at EC-17-007; title chain itself PASS). Fix-burst-29 F-LP31-HIGH-002 EC-17-007 version amendment CONFIRMED landed; new phantom variant introduced in same amendment.**

---

## POL-22 Phase C — Carry-Forward Regression (17+ samples)

Prior fix-burst closures 1..29 spot-checked:

| Prior Finding | Fix Applied At | Regression Check |
|---------------|---------------|-----------------|
| F-LP30-MED-001 (§References BC-2.16.002 completeness) | fix-burst-28 | PASS — §References contains BC-2.16.002 verbatim H1 |
| F-LP29-MED-001 (exclusion-note BC-2.17.005 verbatim) | fix-burst-27 | PASS — line 269 verbatim |
| F-LP28-MED-001/002 (phantom §-section anchors) | fix-burst-26 | PASS — no phantom §S-PLUGIN-PREREQ-D sections |
| F-LP27-MED-001 (subsystems SS-16 missing) | fix-burst-25 | PASS — [SS-22, SS-17, SS-16] confirmed |
| F-LP27-MED-002 (PluginError #[non_exhaustive] MVP-hedge) | fix-burst-25 | PASS — unconditional prescription present |
| F-LP27-MED-003 (§References 7/8 BC titles verbatim) | fix-burst-25 | PASS — all 8 verbatim (fix-burst-28 completed) |
| F-LP26-MED-001 (BC-2.16.002 body-table title verbatim) | fix-burst-24 | PASS — verbatim BC H1 confirmed |
| F-LP25-HIGH-001 (spawn_blocking fabricated ADR-023 §C4) | fix-burst-23 | PASS — BC-2.17.005 §Invariants anchor confirmed |
| F-LP31-HIGH-001 (E-PLUGIN-013/014 message templates) | fix-burst-29 | PASS — three-site verbatim confirmed (codification #16 check) |
| F-LP31-MED-001 (AC-15 AuthToken Debug example) | fix-burst-29 | PASS — AuthToken(<redacted>) aligned |
| F-LP31-HIGH-002 (EC-17-007 default-deny security semantic) | fix-burst-29 | PARTIAL — default-deny amendment landed; phantom variant introduced (F-LP32-CRIT-001) |

**POL-22 Phase C: PARTIAL — fix-burst-29 F-LP31-HIGH-002 closure introduced new CRITICAL
drift (phantom AllowlistRejected). All prior closures (fix-burst-1..fix-burst-28) CLEAN.
Demonstrates 'incomplete fix introduces new drift' anti-pattern.**

---

## POL-22 Phase D — Findings

### F-LP32-CRIT-001 — Phantom `PluginError::AllowlistRejected` variant introduced by fix-burst-29

**Severity:** CRITICAL/HIGH (security-relevant + cross-document drift)
**Confidence:** HIGH (independently verified via repo-wide grep)
**Tags:** [process-gap] — codification candidate #17
**Origin:** fix-burst-29 closure; story-writer §BC Amendments directive at story line 1008
invented the variant name without verification; product-owner executed the amendment as written;
cross-burst adversary did not verify variant existence at closure time.

**Evidence (4-site cross-document inconsistency):**

| Site | Says |
|------|------|
| BC-2.17.002 v1.6 EC-17-007 (line ~85) | `PluginError::AllowlistRejected returned` |
| Story §BC Amendments In-Scope (line 1008) | Same phantom variant prescription |
| Story AC-7 (line 356) | HTTP 403 returned to plugin (no variant prescribed) |
| host_functions.rs:64-68 | `HttpResponse { status: 403, ... }` returned synchronously |
| PluginError enum (error.rs:984-1034) | 8 variants: Trapped, Timeout, MemoryExceeded, NotLoaded, InvalidInterface, SandboxViolation, CompilationFailed, EmptyPluginId — NO AllowlistRejected |
| error-taxonomy.md | No E-PLUGIN-NNN entry for AllowlistRejected |
| Story §Error Taxonomy Additions | No AllowlistRejected row (introduces only E-PLUGIN-013/014/015/016 + E-PIPELINE-001) |

**Adjudication — Path A (selected for fix-burst-30):**

Amend BC-2.17.002 v1.6 → v1.7 EC-17-007 to remove phantom variant reference. Replace with
existing semantics: "Request denied with HTTP 403 returned to plugin (existing E-PLUGIN-005
SandboxViolation semantics); audit log entry created." This aligns with:
- AC-7 prescription (HTTP 403 + tracing::warn!)
- host_functions.rs:64-68 existing code (HttpResponse { status: 403 })
- error-taxonomy.md E-PLUGIN-005 SandboxViolation (existing; error.rs:1020-1023)
- Story §Error Taxonomy Additions (no new variant needed)

Zero new scope. BC-INDEX v4.72 → v4.73.

Path B (introduce new PluginError::AllowlistRejected variant with error.rs + story +
error-taxonomy all updated) rejected: larger blast-radius for security-equivalent outcome;
per CLAUDE.md Canonical Principle Rule 2, feature scope expansion to introduce an enum variant
requires a story-level decision, not a spec-amendment fix-burst.

**Route:** product-owner (BC-2.17.002 v1.6→v1.7 EC-17-007 amendment; BC-INDEX v4.72→v4.73)

---

### F-LP32-MED-001 — Stale BC-2.17.002 v1.5 version pin at AC-9 closure note (line 419)

**Severity:** MEDIUM
**Confidence:** HIGH

Story line 419 blockquote reads:
> "current pinned version v1.5 (fix-burst-7 lifecycle_status-only sweep)"

BC-2.17.002 is now at v1.6 (amended by fix-burst-29 F-LP31-HIGH-002). The closure note
cites v1.5, which is the pre-amendment version. This creates implementer confusion: AC-9
appears to be pegged to a version that no longer reflects the default-deny contract change.

**Required fix:** Update blockquote to "current pinned version v1.6 (fix-burst-29
EC-17-007 default-deny amendment)" OR strip the version pin entirely (leaving just the
behavioral anchor).

**Route:** story-writer (story line 419)

---

### F-LP32-MED-002 — Changelog schema regression: rows 1.27/1.28/1.29 missing Burst column

**Severity:** MEDIUM
**Confidence:** HIGH

Story changelog table schema (line ~1054) is: `| Version | Burst | Date | Author | Change |` (5 columns).

Rows 1.26 (fix-burst-26 stage-1) through current are correctly formatted with 5 cell values.
However rows 1.27, 1.28, and 1.29 are missing the Burst column value:
- Row 1.27: 4 cell values (missing `fix-burst-27 stage-1`)
- Row 1.28: 4 cell values (missing `fix-burst-28 stage-1`)
- Row 1.29: 4 cell values (missing `fix-burst-29 stage-1`)

This is a rendering corruption — markdown tables with mismatched column counts produce
unpredictable display behavior; the Burst column content is lost for 3 versions.

**Required fixes:**
- Row 1.27: insert `fix-burst-27 stage-1` as Burst column value
- Row 1.28: insert `fix-burst-28 stage-1` as Burst column value
- Row 1.29: insert `fix-burst-29 stage-1` as Burst column value

**Route:** story-writer (changelog rows 1.27/1.28/1.29)

---

### F-LP32-OBS-001 [process-gap] — §BC Amendments In-Scope section uses forward-looking framing post-amendment

**Severity:** OBSERVATION (low priority)
**Type:** Process gap

Story §BC Amendments In-Scope at line ~996 uses "MUST land in the same fix-burst" framing
describing the required BC-2.17.002 amendment. That amendment HAS landed (BC-2.17.002 v1.6,
fix-burst-29). After fix-burst-30 lands the v1.7 amendment (F-LP32-CRIT-001 closure), this
section should either be:
(a) retroactively reframed to past-tense ("Amendment landed at fix-burst-29 stage-1; see
    BC-2.17.002 v1.6; v1.7 follows at fix-burst-30 for phantom variant correction"), OR
(b) removed as historical record (the amendment directive is fulfilled)

**Tags:** [process-gap] — candidate for cycle-close session-reviewer retrospective. Not a
blocker; story-writer fix-burst-30 scope if easy to resolve inline.

---

### F-LP32-OBS-002 [process-gap] — BC-amendment error-variant existence verification gap (codification candidate #17)

**Severity:** OBSERVATION
**Type:** Codification candidate #17

**Description:** When a story-writer or product-owner introduces a NEW named entity reference
in BC body text (enum variant, error code, function name), there is no current codification
requiring verification that the named entity exists in its canonical definition location. The
verification gap manifested in pass-32: fix-burst-29 introduced `PluginError::AllowlistRejected`
in BC-2.17.002 EC-17-007 without verifying the variant exists in error.rs PluginError enum,
error-taxonomy.md, or story §Error Taxonomy Additions.

Proposed codification #17: **Named Entity Existence Verification** — When any agent introduces
a new named entity reference (enum variant, error code, type name, function name) in a BC body
or story spec, the cross-burst adversary at the NEXT pass MUST:
1. Extract the named entity
2. Grep for it in its canonical definition location (error.rs for PluginError, error-taxonomy.md
   for E-NNN codes, crate source for type/function names)
3. Verify existence before declaring the prior fix-burst's closure as CLEAN

This is the 6th recurrence of the "introduced-name-not-verified" class of defect in this cycle.

**Tags:** [process-gap] — codification candidate #17. Route to cycle-close session-reviewer for
adjudication.

---

## Summary

| Finding | Severity | Status | Route |
|---------|----------|--------|-------|
| F-LP32-CRIT-001 | CRITICAL/HIGH | OPEN | product-owner — BC-2.17.002 v1.6→v1.7 EC-17-007 phantom AllowlistRejected removal; BC-INDEX v4.72→v4.73 |
| F-LP32-MED-001 | MEDIUM | OPEN | story-writer — AC-9 closure note line 419 stale v1.5 pin → v1.6 |
| F-LP32-MED-002 | MEDIUM | OPEN | story-writer — changelog rows 1.27/1.28/1.29 add missing Burst column values |
| F-LP32-OBS-001 | OBS [process-gap] | OPEN | story-writer (if easy) — §BC Amendments forward-looking framing; cycle-close tag |
| F-LP32-OBS-002 | OBS [process-gap] | OPEN | cycle-close session-reviewer — codification candidate #17 |

**fix-burst-30 routing:** Dispatch story-writer + product-owner IN PARALLEL.
- story-writer: (1) line 419 stale pin v1.5→v1.6; (2) changelog rows 1.27/1.28/1.29 add
  Burst column values (fix-burst-27 stage-1 / fix-burst-28 stage-1 / fix-burst-29 stage-1);
  (3) §BC Amendments In-Scope past-tense reframe (optional — if in-scope for fix-burst-30).
- product-owner: BC-2.17.002 v1.6→v1.7 EC-17-007 amend to remove phantom variant; use
  "Request denied with HTTP 403 returned to plugin (existing E-PLUGIN-005 SandboxViolation
  semantics); audit log entry created"; BC-INDEX v4.72→v4.73.

**Streak: 0/3 HOLD.** Pass-33 follows fix-burst-30 closure.
