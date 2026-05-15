---
cycle: wave-4-operations
last_updated: 2026-05-15
maintainer: orchestrator + state-manager
lessons_codified: 15
---

# Wave-4 Operations — Cycle Lessons

This file durably codifies lessons-learned that emerged during the wave-4 operations cycle (PREREQ-A and PREREQ-B per-story-delivery). Lessons here SHOULD be referenced from fix-burst dispatches and adversary reviews so they don't get lost to STATE.md compaction (per TD-VSDD-058 precedent which documents D-214..D-320 lost to fix-burst-17 compaction).

## Codified Lessons

### Lesson 1: structured-event-catalog ↔ tracing-emission discipline (PG-LP11-001)

**Codified:** 2026-05-11 (fix-burst-12 closure of F-LP12-LOW-002)
**Recurrence count at codification:** 2 (F-LP9-MED-001 closed auth events; F-LP11-MED-001 surfaced same pattern for non-auth events)
**Source decision row:** STATE.md D-419
**Subsystem scope:** SS-16 (prism-spec-engine) — pipeline.rs, auth_provider.rs, validation.rs, interpolation.rs

**Operative rule:** Any fix-burst that introduces a new `tracing::*!(event_type = "...")` site in the prism-spec-engine source files MUST amend the BC-2.16.002 Structured Event Catalog in the SAME atomic commit (TD-VSDD-053). The implementer's burst-closure checklist now includes:

1. After making the code change, run `git diff` and grep for new `event_type = "..."` literals
2. If any new event_types are introduced, identify the field-schema (which structured fields beyond event_type the macro emits)
3. Update BC-2.16.002's Structured Event Catalog (currently v1.8) to add a new row with: event_type | level | function | fields | trigger condition
4. Bump BC version in the same commit
5. Update BC-INDEX with the new BC version

**Why this matters:** The Structured Event Catalog is the contract surface SIEM/SOC operators use to build alert pipelines. A new event_type emitted without catalog update means the contract surface lags impl. The adversary surfaced this pattern twice (P9 + P11). Without this codification, the third occurrence would not be caught until pass-N+M.

**Verification at adversary pass:** the adversary review (LOCAL passes) MUST grep `event_type = "` in pipeline.rs / auth_provider.rs / validation.rs / interpolation.rs and cross-reference against BC-2.16.002 catalog rows. Discrepancy = finding.

**Enforcement layers (status as of 2026-05-11 post-fix-burst-13):**

The SOP relies on FOUR enforcement layers. Current wiring status:

1. **Implementer agent: burst-closure self-check** — STATUS: **PAPER** (not wired in engine prompt as of 2026-05-11). The implementer.md prompt does NOT reference lessons.md or the Structured Event Catalog discipline. A future engine-side TD (filed under vsdd-factory plugin work, task #54) should extend implementer.md to cite this lesson. Until that lands, Layer 1 is aspirational, not enforced.

2. **State-manager agent: pre-commit grep verification** — STATUS: **PAPER** (not wired in state-manager.md prompt as of 2026-05-11). The state-manager.md prompt does NOT include a grep step cross-referencing new event_type sites against BC catalog rows. Until tooling lands (Layer 4), Layer 2 is aspirational.

3. **Adversary agent: pass-N closure verification** — STATUS: **ACTIVE**. Each LOCAL adversary pass since pass-9 has applied this verification. F-LP9-MED-001, F-LP11-MED-001, F-LP12-MED-001, and F-LP13-MED-001 all surfaced via this layer. Layer 3 is the sole load-bearing enforcement until other layers wire.

4. **Lefthook automation: pre-commit grep hook** — STATUS: **DEFERRED** (filed as TD-VSDD-093 P3 for tooling-sprint). When TD-093 lands, a `.factory/hooks/` lefthook pre-commit hook will automatically grep new `event_type = "..."` literals and block commits that don't update BC-2.16.002 catalog. Until then, no automated check exists.

**Net enforcement reality:** 1 of 4 layers actively enforces (adversary). Recurrence count of catalog-drift findings has reached 4 (F-LP9/11/12/13) BECAUSE Layer 3 is the only layer catching it post-impl. The other layers need wiring/tooling to provide pre-impl prevention.

**Linked artifacts:**
- BC-2.16.002 Structured Event Catalog (v1.8 latest)
- F-LP9-MED-001 (auth audit-signal drift, 1st occurrence)
- F-LP11-MED-001 (non-auth events drift, 2nd occurrence)
- F-LP12-LOW-002 (codification-durability gap that surfaced this file's creation)
- STATE.md D-419 (original codification, now superseded by this file)


---

## Step 9 Worktree Cleanup Discipline — Empirical Evidence Anchor [codified]

**Pattern:** Step 9 (worktree cleanup) of the per-story-delivery workflow has now been demonstrated end-to-end across 4 consecutive stories:

| Story | Step 9 Date | Worktree Removed | Branch Deleted At | Notes |
|-------|------------|------------------|-------------------|-------|
| S-3.07 | 2026-05-08 | .worktrees/S-3.07/ | post-squash | First Step 9 execution |
| S-PLUGIN-PREREQ-B | 2026-05-12 | .worktrees/S-PLUGIN-PREREQ-B/ | post-squash | Archival state (worktree retained for reference) |
| S-PLUGIN-PREREQ-C | 2026-05-12 | .worktrees/S-PLUGIN-PREREQ-C/ | post-squash | Archival state (worktree retained for reference) |
| S-PLUGIN-PREREQ-D | 2026-05-15 | .worktrees/S-PLUGIN-PREREQ-D/ | e57d0929 (D-570) | Clean removal, no --force |

**No new codification needed** — Step 9 discipline is already captured in per-story-delivery workflow (Step 9: post-merge worktree cleanup). This entry serves as an empirical-evidence anchor: Step 9 has been executed for 4 consecutive stories without incident, confirming the workflow is operationally stable.

**Linked artifacts:** D-570 (STATE.md), SESSION-HANDOFF.md §POST-STEP-9 CLOSURE SUMMARY, CYCLE-SNAPSHOT §STEP-9-CLOSURE

---

## D-571 CYCLE-CLOSE LESSONS (S-PLUGIN-PREREQ-D — Codified 2026-05-15)

The following 14 lessons were codified during the D-571 session-reviewer cycle-close. Sources: SESSION-REVIEW-D571.md §Lessons-Codified.

### Lesson 2: Lexical-vs-semantic anchor-content verification (POL-22 Phase A extension)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** Candidate #11, pass-25 F-LP25; 6+ recurrences across passes 13/14/15/18/19/25
**Codified-as:** POL-22 Phase A amendment
**Target-anchor:** SESSION-REVIEW-D571.md §Candidate #11

Adversary must grep the CITED DOCUMENT to verify the cited rule actually exists at the stated anchor — story-body substring match is insufficient. Example failure mode: story cites "ADR-023 §C4" for a spawn_blocking rule; the rule is not in ADR-023 §C4 but in BC-2.17.005 §Invariants. For every cited document anchor (ARTIFACT §SectionName), the adversary must grep the cited document's text — not merely confirm the citation appears in the story body. 6+ recurrences confirmed this is systemic.

---

### Lesson 3: BC body-table title verbatim + POL-7 cross-table scope extension

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** Candidates #12, #13, #15; passes 26/27/29; 5+ recurrences
**Codified-as:** POL-7 verification_steps amendment (4 additions)
**Target-anchor:** SESSION-REVIEW-D571.md §Candidates #12/#13/#15

All five citation surfaces in a story require verbatim BC H1 matching: (1) body BC table Title column, (2) §References section entries, (3) Architecture Compliance Rules table rows, (4) prose exclusion-note paragraphs, (5) exclusion-note prose paragraphs citing BCs outside `behavioral_contracts:` frontmatter. Paraphrased or shortened titles are violations even when intent is clear.

---

### Lesson 4: §References completeness

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** Candidate #13-sub; pass-30 sub-extension
**Codified-as:** POL-7 verification_steps amendment (added completeness check)
**Target-anchor:** SESSION-REVIEW-D571.md §Candidate #13-sub

Every BC ID in story frontmatter `behavioral_contracts:` array must appear in the story §References section. Format correctness is necessary but not sufficient — completeness is also required. A §References section can have correct format but be incomplete.

---

### Lesson 5: Phantom-section-anchor prohibition (POL-21)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** Candidate #14, pass-28 F-LP28; 4+ recurrences (passes 28/31/34)
**Codified-as:** POL-21 (new policy — phantom_section_anchor_prohibited)
**Target-anchor:** SESSION-REVIEW-D571.md §Candidate #14

§X notation (e.g., "BC-2.16.002 §S-PLUGIN-PREREQ-D AC-16") must resolve to an actual `##` heading in the referenced document. Bold-labeled bullets are NOT section headings and MUST NOT be cited with the §-sigil. If the target is a bold-labeled bullet, the correct form is "§ParentHeading (BoldLabel bullet)" to make the ancestry explicit. 4+ recurrences confirmed HIGH priority.

---

### Lesson 6: Error message template verbatim (POL-24)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** Candidate #16, pass-31 F-LP31; promoted mid-cascade at D-530; 2+ consecutive pass triggers
**Codified-as:** POL-24 (error_message_template_verbatim — added to policies.yaml)
**Target-anchor:** SESSION-REVIEW-D571.md §Candidate #16

Story §Error Taxonomy Additions rows and all prose references to error-taxonomy.md entries must match the canonical `message_template:` field byte-for-byte — including delimiter style (backtick vs single-quote), capitalization, and punctuation. Visually similar but non-identical text trips downstream tooling and adversary verification.

---

### Lesson 7: BC-amendment named entity existence verification (POL-22 Phase C)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** Candidate #17, pass-32 F-LP32-CRIT-001; 4 recurrences (passes 7/15-16/21/23 as recursive prescription gaps; pass-32 as BC-internal phantom)
**Codified-as:** POL-22 Phase C amendment (new phase — named-entity-existence-verification)
**Target-anchor:** SESSION-REVIEW-D571.md §Candidate #17

When a BC body introduces or cites a named entity (enum variant, error code, type name), the adversary MUST grep the canonical definition location before declaring CLEAN. Root cause of F-LP32-CRIT-001: `PluginError::AllowlistRejected` introduced by fix-burst-29 in BC-2.17.002 but the variant does not exist in prism-core error.rs. 4 in-burst regressions from this class.

---

### Lesson 8: BC-version-bump sibling-grep gate (POL-23)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** POL-23 candidate, pass-33 F-LP33-OBS-001; 8 recurrences across the full cascade
**Codified-as:** POL-23 (new policy — bc_version_bump_sibling_grep_gate; added to policies.yaml)
**Target-anchor:** SESSION-REVIEW-D571.md §POL-23 candidate

When a BC version bumps, the SAME atomic commit must update ALL story body prose sites that pin the old version string — including inline trace-header lines, §References section version annotations, and Architecture Compliance Rules table version cells. BC frontmatter `modified:` and `timestamp:` fields must also be synced to the amendment date. 8 recurrences across the cascade confirmed systemic gap. Story version bumps must also sync both `version:` and `updated:` frontmatter fields (PG-IMPL-LP6-003 extension to POL-23).

---

### Lesson 9: Multi-cite propagation sweep mandatory (POL-25)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** POL-25 candidate, OBS-LP35-002; pass-35; 5 cascade instances (passes 28/32/33/35 x2) + 4 VP-INDEX propagation misses
**Codified-as:** POL-25 (new policy — multi_cite_propagation_sweep_mandatory; added to policies.yaml)
**Target-anchor:** SESSION-REVIEW-D571.md §POL-25 candidate

When a fix-burst corrects a phrasing or anchor-string pattern across multiple artifact sites, the grep MUST enumerate ALL documents that can carry the pattern — including VP-INDEX named-alias row descriptions, error-taxonomy.md prose, §References in all stories tracing to the same BC, and architecture documents. Closing 4 of 5 sites and missing the 5th constitutes an incomplete fix.

---

### Lesson 10: §Changelog schema-integrity validator (POL-26)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** POL-26 candidate, OBS-LP38-001; pass-38; 4 recurrences (F-LP32-MED-002, F-LP34-HIGH-001, F-LP38-MED-001/002)
**Codified-as:** POL-26 (new policy — changelog_schema_integrity; added to policies.yaml)
**Target-anchor:** SESSION-REVIEW-D571.md §POL-26 candidate

Before committing §Changelog row additions to index files, state-manager must count pipe-delimited cells in each new row and verify they match the header row count. VP-INDEX has 5 columns; STORY-INDEX has 3 columns. D-NNN references are NEVER standalone trailing cells — always folded into the rightmost content cell as a parenthetical prefix. Root cause: orchestrator dispatch prompt templates prescribed incorrect §Changelog row formats. 4 recurrences.

---

### Lesson 11: Story frontmatter version sync (PG-IMPL-LP6-003 / POL-23 extension)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** PG-IMPL-LP6-003; impl-passes 6 and 8 (2 consecutive impl-passes blocked by same class)
**Codified-as:** POL-23 verification_steps amendment (story frontmatter axis)
**Target-anchor:** SESSION-REVIEW-D571.md §PG-IMPL-LP6-003

Every story version bump must sync both `version:` (machine-readable frontmatter pointer) and `updated:` (ISO date) frontmatter fields in the same atomic commit. The `version:` field is the canonical machine-readable pointer; tooling reading the story file gets the stale version when only body changelog is updated. 2 consecutive impl-passes blocked by this exact class — structural enforcement threshold met.

---

### Lesson 12: Production-linker enforcement (PG-IMPL-LP4/5-001)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** PG-IMPL-LP4-001 (impl-pass-4) + PG-IMPL-LP5-001 (impl-pass-5); 5 paper-fix recurrences across impl-passes 3–5
**Codified-as:** TD-VSDD-059 paper-fix detection extension (adversary dispatch rubric)
**Target-anchor:** SESSION-REVIEW-D571.md §PG-IMPL-LP4-001, §PG-IMPL-LP5-001

Tests claiming production-callback coverage must use the production builder (`PluginRuntime::build_linker`), not a test-local linker (`Linker::<HostState>::new(&engine)`). Detection heuristic: grep test body for `Linker::new(` / `Linker::<.*>::new(` — if present without production builder call, reopen as paper-fix. Sanity-revert verification required: revert production function's core logic; the test MUST fail. 5 recurrences.

---

### Lesson 13: Component Model fixture source files required (PG-IMPL-LP6-002)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** PG-IMPL-LP6-002; impl-pass-6 F-PASS6-MED-001
**Codified-as:** TD-VSDD-059 extension (implementer self-audit checklist)
**Target-anchor:** SESSION-REVIEW-D571.md §PG-IMPL-LP6-002

New test fixtures committed to tests/fixtures/ must include accompanying WIT IDL, WAT core module source files, and a documented build recipe (Justfile recipe). Binary-only fixture commits without source are TD-VSDD-059 paper-fix vectors: when wasmtime is bumped or wasm-tools changes ABI emission, there is no source-of-truth to rebuild from.

---

### Lesson 14: `modified:` field ISO normalization (POL-27, Path A)

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** OBS-LP41-001; BC-2.22.001 format heterogeneity; Path A recommended and canonized
**Codified-as:** POL-27 (new policy — bc_modified_field_iso_date_format; added to policies.yaml)
**Target-anchor:** SESSION-REVIEW-D571.md §Bucket-C, §POL-27

BC `modified:` frontmatter field must use ISO 8601 date scalar format (YYYY-MM-DD) matching the most recent §Changelog entry date. Burst-ID-list format (e.g., `modified: [fix-burst-22, fix-burst-7]`) is prohibited for new BCs and for BCs modified after this policy takes effect. Path A selected over Path B because: POL-20 precedent for `introduced:` format discipline applies equally to `modified:`; heterogeneity across ~30 files creates maintenance burden; ISO dates are universally parseable and sort correctly. Maintenance sweep story queued for ~30 pre-existing files (grace period until sweep lands).

---

### Lesson 15: state-manager-single-commit DECISIVELY STABLE

**Codified:** 2026-05-15 (D-571 cycle-close)
**Source cascade:** TD-VSDD-053 single-commit protocol; 77th consecutive single-commit at D-571
**Codified-as:** Stable-convention confirmation (no new POL required)
**Target-anchor:** SESSION-REVIEW-D571.md §Lesson 15

77 consecutive single-commits (TD-VSDD-053) with zero MULTI_COMMIT_CHAIN_NOT_ALLOWED violations. The Single-Commit Burst Protocol is operationally stable at this project's scale. No additional codification needed. Reconfirming stable-convention status per §Candidate #4 pre-compact snapshot verdict.
