---
document_type: adversarial-review
review_id: wave-a-spec-pass-63
pass_number: 63
reviewer: vsdd-factory:adversary
review_type: records-compliance
artifact_scope:
  reviewed:
    - .factory/specs/behavioral-contracts/BC-INDEX.md (v8.72 — post-FB46)
    - .factory/specs/architecture/ARCH-INDEX.md (v2.275 — post-FB46)
    - .factory/stories/STORY-INDEX.md (v2.725 — post-FB46)
    - .factory/stories/S-WAVE-A-ENGINE-001-header-scheme-field-rule9-validation-auth-dispatch.md (v2.2)
    - .factory/STATE.md (v8.564)
    - .factory/policies.yaml (v1.36)
    - .factory/cycles/wave-5-e-demo-fidelity/lessons.md (through Lesson 99)
  untracked_found:
    - .factory/specs/architecture/decisions/ADR-055-validate-sensor-spec-production-wiring.md (v1.0)
    - .factory/stories/S-WAVE-A-CYBERINT-PATCH-001-cyberint-header-scheme-patch.md (v1.0)
    - .factory/stories/S-WAVE-A-CYBERINT-SPEC-001-cyberint-dual-surface-spec-migration.md (v1.0)
    - .factory/stories/S-ADR055-WAVE-A-001-validate-sensor-spec-production-wiring.md (v1.0)
    - .factory/stories/S-WAVE-A-MCP-001-add-sensor-spec-structured-error-response.md (v1.0)
    - .factory/stories/S-ADR054-WAVE-A-001-declarative-http-auth-acquisition.md (v1.0)
    - .factory/stories/S-WAVE-A-ARMIS-REMEDIATION-001-armis-token-exchange-spec-and-dtu-reclone.md (v1.0)
frozen_head: factory-artifacts@e95700456
streak_pre_pass: "0/3"
streak_post_pass: "0/3"
verdict: BLOCKED
findings_count: 33
severity_breakdown:
  critical: 1
  high: 8
  medium: 10
  low: 7
  observation: 5
  process_gap: 2
novelty: HIGH
related_state_decision: D-2017
related_fix_burst: FB47a
date: 2026-07-25
---

# Wave-A Spec-Evolution Adversarial Review — Local Pass 63

**Scope:** Post-FB46 records-compliance audit — index/ledger currency verification against on-disk artifact frontmatter, untracked-file registration gap detection, and cascade-divergence root-cause analysis. Full cross-index consistency check: BC-INDEX v8.72, ARCH-INDEX v2.275, STORY-INDEX v2.725.

---

## F-WASE-P63-CRIT-001 — BC-INDEX v8.72 NOTE contains falsified ledger attributions for BC-2.01.016 v1.15 and BC-2.16.014 v1.19

**Severity:** CRITICAL
**Category:** Ledger falsification — POL-1 violation + provenance integrity

BC-INDEX v8.72 NOTE states: "MED-001 close: BC-2.01.016 v1.14→v1.15 (§Behavioral Contracts attribution corrected in S-WAVE-A-ENGINE-001 — MED-008: get_token() default method addition belongs to BC-2.16.014 §P9, not BC-2.01.016 scope); BC-2.16.014 v1.18→v1.19 (F-WASE-P62-HIGH-001+HIGH-005 companion — §Entry points four-function model corrected; BC-2.16.008 §Error Conditions expansion reciprocal reference added)."

Verification of on-disk frontmatter:

BC-2.01.016 top changelog row: "v1.15 | wave-a-spec-evolution-fix-burst-45 (F-WASE-P61-MED-004) | added `| Stories | S-WAVE-A-ENGINE-001 |` row to §Traceability"

BC-2.16.014 top changelog row: "v1.19 | wave-a-spec-evolution-fix-burst-45 (F-WASE-P61-MED-004) | added `| Stories | S-WAVE-A-ENGINE-001 |` row to §Traceability"

Both v1.15 and v1.19 were authored in FB45 (F-WASE-P61-MED-004), NOT FB46. The BC-INDEX v8.72 NOTE attributes them to FB46 (F-WASE-P62-MED-008 and F-WASE-P62-HIGH-001+HIGH-005) — these descriptions are phantom and do not correspond to any actual change in those files during FB46.

Further, the actual FB46 changes to BC-2.16.009 (v1.25→v1.26) and BC-2.16.008 (v1.5→v1.6) are NOT recorded in the v8.72 NOTE. The ledger omits the real changes and fabricates non-existent ones.

**Root cause:** POL-37 (ledger-from-frontmatter discipline) absent; state-manager generated the v8.72 NOTE from dispatch brief values rather than on-disk frontmatter verification. This is a process-gap codification target.

**Resolution:** BC-INDEX v8.73 (FB47a): forward-correction rows appended to BC-2.01.016 and BC-2.16.014 table cells retracting phantom attributions; v8.73 NOTE records actual FB46 changes; BC-2.16.009 v1.26 + BC-2.16.008 v1.6 inline pins added.

---

## F-WASE-P63-HIGH-001 — BC-INDEX v8.72 BC-2.16.009 inline version pin missing v1.26 entry

**Severity:** HIGH
**Category:** Index staleness — L10 cross-document consistency gate violation

BC-INDEX v8.72 BC-2.16.009 table cell status ends at v1.25. On-disk artifact has version "1.26". A v1.26 FB46 bump is unrecorded in the index.

**Resolution:** BC-INDEX v8.73 (FB47a): v1.26 annotation appended to BC-2.16.009 cell.

---

## F-WASE-P63-HIGH-002 — BC-INDEX v8.72 BC-2.16.008 status cell shows bare "draft" with no version pin

**Severity:** HIGH
**Category:** Index staleness — L10 cross-document consistency gate violation

BC-INDEX v8.72 BC-2.16.008 table cell status is simply "draft" with no inline version annotation. On-disk artifact has version "1.6". The entire lifecycle from v1.0 to v1.6 is unrecorded in the index cell.

**Resolution:** BC-INDEX v8.73 (FB47a): v1.6 annotation added to BC-2.16.008 cell.

---

## F-WASE-P63-HIGH-003 — ARCH-INDEX v2.275 doc-map Verification Architecture cell stale at v1.42 (artifact is v1.46)

**Severity:** HIGH
**Category:** Index staleness — 4 version gaps

ARCH-INDEX v2.275 doc-map shows `v1.42` for verification-architecture.md. On-disk artifact frontmatter: `version: "1.46"`. Four intermediate versions (v1.43 VP-157, v1.44 VP-158, v1.45 VP-159, v1.46 VP-160) are unrecorded.

**Resolution:** ARCH-INDEX v2.276 (FB47a): doc-map cell updated to v1.46 with intermediate version summary.

---

## F-WASE-P63-HIGH-004 — ARCH-INDEX v2.275 doc-map Verification Coverage Matrix cell stale at v1.43 (artifact is v1.48)

**Severity:** HIGH
**Category:** Index staleness — 5 version gaps

ARCH-INDEX v2.275 doc-map shows `v1.43` for verification-coverage-matrix.md. On-disk artifact frontmatter: `version: "1.48"`. Five intermediate versions unrecorded.

**Resolution:** ARCH-INDEX v2.276 (FB47a): doc-map cell updated to v1.48 with summary.

---

## F-WASE-P63-HIGH-005 — ARCH-INDEX v2.275 ADR-053 registry cell missing v0.35 annotation

**Severity:** HIGH
**Category:** Index staleness — most-recent ADR version unrecorded

ARCH-INDEX v2.275 ADR-053 registry cell shows "ACCEPTED v0.34" as the most recent. On-disk ADR-053 frontmatter: `version: "0.35"`. FB46 added §D6 (F-WASE-P62-CRIT-001 adjudication Option B) which is the architecturally significant decision; the absence means the ARCH-INDEX ADR table does not reflect the current architectural state.

**Resolution:** ARCH-INDEX v2.276 (FB47a): v0.35 annotation appended to ADR-053 cell.

---

## F-WASE-P63-HIGH-006 — ADR-055 file exists on disk (untracked) with no ARCH-INDEX registration

**Severity:** HIGH
**Category:** Missing registration

`git status` shows `.factory/specs/architecture/decisions/ADR-055-validate-sensor-spec-production-wiring.md` as untracked. ARCH-INDEX ADR table has no ADR-055 row. An untracked ADR with no index entry is invisible to all downstream consumers.

**Resolution:** ARCH-INDEX v2.276 (FB47a): ADR-055 row added (PROPOSED v1.0, SS-06, related_adrs: [ADR-030]).

---

## F-WASE-P63-HIGH-007 — Six Wave-A story files exist on disk (untracked) with no STORY-INDEX registration

**Severity:** HIGH
**Category:** Missing registration — 6 story files

`git status` confirms these untracked story files with no STORY-INDEX rows:
- S-WAVE-A-CYBERINT-PATCH-001-cyberint-header-scheme-patch.md (v1.0, P0, 1pt, 3 ACs)
- S-WAVE-A-CYBERINT-SPEC-001-cyberint-dual-surface-spec-migration.md (v1.0, P0, 8pts, 8 ACs)
- S-ADR055-WAVE-A-001-validate-sensor-spec-production-wiring.md (v1.0, P0, 8pts, 10 ACs)
- S-WAVE-A-MCP-001-add-sensor-spec-structured-error-response.md (v1.0, P1, 5pts, 7 ACs)
- S-ADR054-WAVE-A-001-declarative-http-auth-acquisition.md (v1.0, P0, 13pts, 9 ACs)
- S-WAVE-A-ARMIS-REMEDIATION-001-armis-token-exchange-spec-and-dtu-reclone.md (v1.0, P1, 8pts, 6 ACs)

Total: 6 stories, 43 story-points unregistered.

**Resolution:** STORY-INDEX v2.726 (FB47a): all 6 rows added; total_stories 257→263.

---

## F-WASE-P63-HIGH-008 — S-WAVE-A-ENGINE-001 story file blocks missing S-WAVE-A-CYBERINT-PATCH-001

**Severity:** HIGH
**Category:** Story file drift — co-land merge dependency undocumented in story spec

S-WAVE-A-ENGINE-001 frontmatter `blocks:` lists S-ADR054-WAVE-A-001, S-WAVE-A-CYBERINT-SPEC-001, S-WAVE-A-ARMIS-REMEDIATION-001. S-WAVE-A-CYBERINT-PATCH-001 has MERGE-GATE-ENGINE-001 documented in its own file specifying it MUST co-land with ENGINE-001. The reciprocal blocks entry is absent from ENGINE-001.

**Resolution:** S-WAVE-A-ENGINE-001 v2.3 (FB47a): S-WAVE-A-CYBERINT-PATCH-001 added to blocks list.

---

## F-WASE-P63-MED-001 — ARCH-INDEX v2.275 VP-INDEX description factually wrong: "version stays 2.13"

**Severity:** MEDIUM
**Category:** Ledger inaccuracy — VP-INDEX progression misrepresented

ARCH-INDEX v2.275 changelog row states "VP-INDEX: v2.12 volatile cite fix (version stays 2.13)." This is wrong in two ways: (1) VP-INDEX DID advance from v2.12 to v2.13 — it was not a "volatile cite fix" that left the version unchanged; (2) VP-160 was registered in VP-INDEX v2.13, an architecturally significant addition.

On-disk VP-INDEX top changelog row (verified): VP-INDEX v2.12→v2.13 records VP-160 registration (Rule 9 cookie-name charset totality Kani proof, kani P0, BC-2.16.009).

**Resolution:** ARCH-INDEX v2.276 (FB47a): v2.276 row forward-corrects v2.275.

---

## F-WASE-P63-MED-002 — ARCH-INDEX v2.275 changelog row contains phantom BC-2.01.016 and BC-2.16.014 attributions

**Severity:** MEDIUM
**Category:** Ledger falsification cascade — CRIT-001 propagated to ARCH-INDEX

ARCH-INDEX v2.275 changelog row states: "BC-INDEX v8.71→v8.72 (BC-2.01.016 v1.14→v1.15 §Behavioral Contracts attribution corrected; BC-2.16.014 v1.18→v1.19 §Entry points four-function model corrected + BC-2.16.008 §Error Conditions reciprocal reference added)." These attributions are phantom — see CRIT-001. The cascade from BC-INDEX v8.72 NOTE into ARCH-INDEX v2.275 row propagated the falsification.

**Resolution:** ARCH-INDEX v2.276 (FB47a): forward-correction in v2.276 row.

---

## F-WASE-P63-MED-003 — STORY-INDEX S-WAVE-A-ENGINE-001 v2.0 shows wrong AC/RGT counts

**Severity:** MEDIUM
**Category:** POL-1 violation + ledger inaccuracy

STORY-INDEX v2.725 S-WAVE-A-ENGINE-001 registration paragraph states "21 ACs; 24 Red Gate tests." The v2.0 registration (D-2013, 2026-07-24) was 19 ACs / 23 Red Gate tests. The v2.724 burst (FB45) retroactively modified the v2.0 text in-place to say 21/24, violating POL-1.

Evidence: STORY-INDEX v2.723 changelog row states "S-WAVE-A-ENGINE-001 registered (19 ACs; 23 RGTs)." STORY-INDEX v2.724 changelog row states "19→21 ACs / 23→24 RGTs (HIGH-002 AC-019/AC-020/RG-024...)" — this is a forward-correction for new ACs added in FB45, but the correction was applied as an in-place edit to v2.0 text rather than as a v2.1 segment.

**Resolution:** STORY-INDEX v2.726 (FB47a): v2.0 text restored to 19/23; v2.1 segment added documenting the correction and recording the blocks list addition.

---

## F-WASE-P63-MED-004 — STORY-INDEX S-MAINT-VOLATILE-CITE-001 metadata wrong: 3 ACs / 2 pts / P3 / tdd_mode:none

**Severity:** MEDIUM
**Category:** Index staleness — story metadata drift

STORY-INDEX v2.725 S-MAINT-VOLATILE-CITE-001 row description states: "3 ACs; 2 pts; P3; tdd_mode: none."

On-disk story frontmatter (verified): priority: P2, points: 5, tdd_mode: strict, 4 ACs (counted via `grep -c "^### AC-"`).

All four fields are wrong. The crates_touched field in the description also says `.factory/specs` but the actual value is `[]`.

**Resolution:** STORY-INDEX v2.726 (FB47a): description text and column values corrected.

---

## F-WASE-P63-MED-005 — STORY-INDEX S-MAINT-VOLATILE-CITE-001 Crate column shows `.factory/specs` (should be `--`)

**Severity:** MEDIUM
**Category:** Index staleness — Crate column drift

STORY-INDEX v2.725 S-MAINT-VOLATILE-CITE-001 Crate column shows `.factory/specs`. On-disk frontmatter `crates_touched: []`. The crate column should be `--` (no crate touched).

**Resolution:** STORY-INDEX v2.726 (FB47a): Crate column corrected.

---

## F-WASE-P63-MED-006 — STORY-INDEX S-MAINT-VOLATILE-CITE-002 metadata wrong: 3 ACs / 2 pts / P3 / tdd_mode:none

**Severity:** MEDIUM
**Category:** Index staleness — story metadata drift

STORY-INDEX v2.725 S-MAINT-VOLATILE-CITE-002 row description states: "3 ACs; 2 pts; P3; tdd_mode: none."

On-disk story frontmatter (verified): priority: P2, points: 8, tdd_mode: strict, 4 ACs.

**Resolution:** STORY-INDEX v2.726 (FB47a): description text and column values corrected.

---

## F-WASE-P63-MED-007 — STORY-INDEX S-MAINT-VOLATILE-CITE-002 Crate column shows `.factory/specs`; Depends On column shows `--`

**Severity:** MEDIUM
**Category:** Index staleness — two column drifts

(1) Crate column shows `.factory/specs`; actual crates_touched: []. (2) Depends On column shows `--`; actual depends_on: [S-MAINT-VOLATILE-CITE-001].

**Resolution:** STORY-INDEX v2.726 (FB47a): Crate column → `--`; Depends On column → `S-MAINT-VOLATILE-CITE-001`.

---

## F-WASE-P63-MED-008 — STORY-INDEX S-WAVE-A-ENGINE-001 description blocks list missing S-WAVE-A-CYBERINT-PATCH-001

**Severity:** MEDIUM
**Category:** Story description drift

STORY-INDEX v2.725 S-WAVE-A-ENGINE-001 description text shows "blocks S-ADR054-WAVE-A-001 + S-WAVE-A-CYBERINT-SPEC-001 + S-WAVE-A-ARMIS-REMEDIATION-001." S-WAVE-A-CYBERINT-PATCH-001 is absent.

**Resolution:** STORY-INDEX v2.726 (FB47a): S-WAVE-A-CYBERINT-PATCH-001 prepended to blocks list in v2.1 segment; v2.1 also records this change.

---

## F-WASE-P63-MED-009 — STORY-INDEX total_stories frontmatter shows 257; should be 263 after registrations

**Severity:** MEDIUM
**Category:** Index staleness — frontmatter count drift

STORY-INDEX v2.725 `total_stories: 257`. Six new stories exist on disk unregistered. After registration: 257 + 6 = 263.

**Resolution:** STORY-INDEX v2.726 (FB47a): total_stories: 263.

---

## F-WASE-P63-MED-010 — ARCH-INDEX v2.275 changelog row mirrors v2.725 STORY-INDEX wrong total_stories count

**Severity:** MEDIUM
**Category:** Cascade ledger error

ARCH-INDEX v2.275 changelog row states "total_stories 255→257." On-disk STORY-INDEX v2.725 frontmatter shows total_stories: 257. However, six stories are unregistered. The 257 count in both ARCH-INDEX and STORY-INDEX is pre-registration and does not reflect the correct 263 total.

**Resolution:** ARCH-INDEX v2.276 (FB47a): v2.276 row states total_stories 257→263; STORY-INDEX v2.726 corrected.

---

## F-WASE-P63-LOW-001 — STORY-INDEX v2.725 blank line within table body

**Severity:** LOW
**Category:** Table formatting — blank lines within table body disrupt parsers

A blank line appears within the story table body between the DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 row and the S-MAINT-PRMGR-HOOK-SCOPE-001 row. Markdown table parsers treat a blank line as table termination; any row after the blank line may be rendered as outside the table.

**Resolution:** STORY-INDEX v2.726 (FB47a): blank line removed.

---

## F-WASE-P63-LOW-002 — STORY-INDEX §Changelog mixed version prefix (v2.725/v2.724 bare; v2.723+ use v-prefix)

**Severity:** LOW
**Category:** Format inconsistency — version prefix normalization

§Changelog rows v2.725 and v2.724 use bare numeric versions ("2.725", "2.724"). All rows v2.723 and earlier use the v-prefix convention ("v2.723", "v2.722", ...). This creates a mixed convention. The v2.724 entry's LOW-005 note says "v-prefix stripped from version field" but that was a burst-specific decision that created divergence with the broader corpus.

**Resolution:** STORY-INDEX v2.726 (FB47a): new v2.726 entry uses v-prefix; normalizing back to the corpus convention.

---

## F-WASE-P63-LOW-003 — STATE.md v8.564 bc_index_version: "8.72" stale + phantom NOTE

**Severity:** LOW
**Category:** State staleness — frontmatter field drift

STATE.md v8.564 frontmatter `bc_index_version: "8.72"`. After FB47a BC-INDEX advances to v8.73, this field becomes stale. Additionally, the NOTE lines in STATE.md frontmatter reflecting the v8.72 phantom attributions will perpetuate the CRIT-001 falsification.

**Resolution:** STATE.md v8.565 (FB47a): bc_index_version updated; D-2017 decision row added.

---

## F-WASE-P63-LOW-004 — STATE.md v8.564 arch_index_version and story_index_version stale

**Severity:** LOW
**Category:** State staleness — frontmatter field drift

STATE.md v8.564 frontmatter: arch_index_version: "2.275" (will be 2.276); story_index_version: "2.725" (will be 2.726).

**Resolution:** STATE.md v8.565 (FB47a): both fields updated.

---

## F-WASE-P63-LOW-005 — STATE.md v8.564 total_stories stale

**Severity:** LOW
**Category:** State staleness

STATE.md v8.564 `total_stories: 257`. After 6 registrations: 263.

**Resolution:** STATE.md v8.565 (FB47a): total_stories: 263.

---

## F-WASE-P63-LOW-006 — STATE.md v8.564 session resume checkpoint stale

**Severity:** LOW
**Category:** State staleness — session resume checkpoint

STATE.md v8.564 Session Resume Checkpoint still references the post-FB46 state. After FB47a, the checkpoint must be advanced.

**Resolution:** STATE.md v8.565 (FB47a): session resume checkpoint updated.

---

## F-WASE-P63-LOW-007 — ARCH-INDEX v2.275 ADR-054 cell references §D10(c) for ExpiryMode-on-oauth2 rejection (corrected in v2.272 to §D10(h) but the v2.271 correction note in v2.272 references a specific AC count from the phantom v2.724 state)

**Severity:** LOW
**Category:** Cascade staleness

The ARCH-INDEX v2.275 ADR-054 registry cell history references "v0.52 (D-2007 2026-07-24: §D11 three-constructor manifest ratified)." This is accurate. However, the v2.272 in-place correction note at the end of that cell's history references an AC count ("21 ACs") derived from the phantom v2.724 S-WAVE-A-ENGINE-001 state rather than the correct 19 AC base. This is a residual staleness from the CRIT-001 cascade.

**Resolution:** ARCH-INDEX v2.276 (FB47a): v2.276 row notes the CRIT-001 correction; no in-place edit needed (POL-1).

---

## F-WASE-P63-OBS-001 — policies.yaml v1.36 missing POL-37: ledger-from-frontmatter discipline

**Severity:** OBSERVATION
**Category:** Governance gap — process codification

POL-37 (state-manager must generate burst ledger rows by reading each touched file's frontmatter version: after specialist edits land, never from dispatch brief values) does not exist in policies.yaml. Its absence is the root process cause of CRIT-001.

**Resolution:** policies.yaml v1.38 (FB47a): POL-37 added (HIGH severity).

---

## F-WASE-P63-OBS-002 — policies.yaml v1.36 missing POL-38: BC new EC ⇒ story AC/RGT obligation

**Severity:** OBSERVATION
**Category:** Governance gap — story-spec coupling

POL-38 (when a BC amendment adds an EC whose anchor story is draft + tdd_mode:strict, same burst must add corresponding AC and Red Gate test or record explicit deferral) does not exist. This gap enabled S-MAINT-VOLATILE-CITE-001/002 to be registered with incorrect AC counts because the story was authored without triggering this check.

**Resolution:** policies.yaml v1.38 (FB47a): POL-38 added (HIGH severity).

---

## F-WASE-P63-OBS-003 — lessons.md missing Lessons 100-104

**Severity:** OBSERVATION
**Category:** Lessons capture gap

lessons.md ends at Lesson 99 (pass-62 observations). Five cascade-divergence lessons (100-104) covering the CRIT-001 pattern, L10 gate count exposure, vague corrective instruction risk, orchestrator verification discipline, and L10 capability boundary explicitness are unrecorded.

**Resolution:** lessons.md (FB47a): Lessons 100-104 appended.

---

## F-WASE-P63-OBS-004 — STATE.md missing D-2017 decision row for FB47a

**Severity:** OBSERVATION
**Category:** Decision log gap

STATE.md v8.564 decision log ends at D-2016. D-2017 (FB47a records-correction burst) is unrecorded.

**Resolution:** STATE.md v8.565 (FB47a): D-2017 row added.

---

## F-WASE-P63-OBS-005 — DRIFT-CASCADE-DIVERGENCE-001 not registered in STATE.md

**Severity:** OBSERVATION
**Category:** Drift record gap

The pattern where CRIT-001 originated (cascade diverged because fix-bursts were larger than review could absorb; state-manager generated ledger rows from brief values rather than on-disk verification; six story files authored but not registered) has not been codified as a DRIFT record in STATE.md.

**Resolution:** STATE.md v8.565 (FB47a): DRIFT-CASCADE-DIVERGENCE-001 registered.

---

## F-WASE-P63-PROCESS-GAP-001 — POL-37 absence enabled CRIT-001: state-manager generated BC-INDEX v8.72 NOTE from dispatch brief rather than on-disk verification

**Severity:** PROCESS-GAP
**Category:** Root cause — ledger falsification origin

The state-manager authoring BC-INDEX v8.72 NOTE sourced BC-2.01.016 and BC-2.16.014 version descriptions from the dispatch brief rather than reading each file's frontmatter `version:` and top changelog row after specialist edits landed. This is the direct root cause of CRIT-001.

Evidence: BC-2.01.016 and BC-2.16.014 top changelog rows both say "fix-burst-45 (F-WASE-P61-MED-004)" — not FB46. A from-disk read would have caught this immediately. The BC-INDEX v8.72 NOTE was never verified against on-disk artifact state.

**Codification target:** POL-37 (ledger-from-frontmatter). Impact: every state-manager burst where BC-INDEX, ARCH-INDEX, or STORY-INDEX notes are authored from brief values rather than disk verification is at risk of this class of falsification.

---

## F-WASE-P63-PROCESS-GAP-002 — Six story files authored by story-writer but not registered — registration gate absent between authoring and cascade phases

**Severity:** PROCESS-GAP
**Category:** Pipeline gate gap — story registration not enforced

Six story files (S-WAVE-A-CYBERINT-PATCH-001, S-WAVE-A-CYBERINT-SPEC-001, S-ADR055-WAVE-A-001, S-WAVE-A-MCP-001, S-ADR054-WAVE-A-001, S-WAVE-A-ARMIS-REMEDIATION-001) were authored by story-writer and committed to factory-artifacts but never registered in STORY-INDEX. The cascade proceeded through FB46 (21 findings, 21 fixes) without noticing that 43 story-points of Wave-A delivery were invisible to the index.

Root cause: the orchestration pipeline has no mandatory "untracked story files = STORY-INDEX registration required" gate. Story-writer and state-manager operate in separate dispatches; the state-manager registration burst was apparently deferred and then not dispatched before the next adversary pass.

**Codification target:** Story registration gate — untracked `S-*.md` files in `.factory/stories/` must be registered before the next adversary pass begins. Applicable to ADR files too (ADR-055 was also untracked).

---

## Summary

**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO

**BC-5.39.001 streak:** 0/3 (reset; findings present)

**Fix-burst:** FB47a — records-correction and registration burst

**Blocking findings:** 1 CRIT + 8 HIGH = 9 blocking findings. All require records-only corrections; no spec body changes required.

**Closures by FB47a:**
- CRIT-001: BC-INDEX v8.73 forward-correction rows + v8.73 NOTE recording actual FB46 changes
- HIGH-001, HIGH-002: BC-INDEX v8.73 inline version pins for BC-2.16.009 v1.26 + BC-2.16.008 v1.6
- HIGH-003, HIGH-004: ARCH-INDEX v2.276 doc-map cells updated to v1.46 + v1.48
- HIGH-005: ARCH-INDEX v2.276 ADR-053 cell updated to v0.35
- HIGH-006: ARCH-INDEX v2.276 ADR-055 row added
- HIGH-007: STORY-INDEX v2.726 six story rows added (total_stories 257→263)
- HIGH-008: S-WAVE-A-ENGINE-001 v2.3 blocks += S-WAVE-A-CYBERINT-PATCH-001
- MED-001 through MED-010: STORY-INDEX v2.726 + ARCH-INDEX v2.276 corrections
- LOW-001 through LOW-007: STORY-INDEX v2.726 + STATE.md v8.565 corrections
- OBS-001, OBS-002: policies.yaml v1.38 (POL-37 + POL-38)
- OBS-003: lessons.md Lessons 100-104
- OBS-004, OBS-005: STATE.md v8.565 D-2017 + DRIFT-CASCADE-DIVERGENCE-001
- PROCESS-GAP-001: closed by POL-37 (policies.yaml v1.38)
- PROCESS-GAP-002: documented; story-registration-gate story TBD
