---
document_type: consistency-audit-report
story_id: S-DEMO-MULTI-TENANT-DTU-001
audit_label: pre-pass-6
auditor: consistency-validator
date: 2026-06-13
decision_id: D-1150
state_version: "7.793"
bc_version_at_audit: "1.5"
story_version_at_audit: "1.8"
bc_version_post_fix: "1.6"
story_version_post_fix: "1.9"
status: ALL_CLOSED
---

# Consistency Audit — S-DEMO-MULTI-TENANT-DTU-001 Pre-Pass-6

**Purpose:** Comprehensive cross-document consistency audit before dispatching LOCAL adversary
Pass-6. The adversary had been finding piecemeal spec/doc drift across passes 2–5; this audit
consolidates all cross-document checks upfront so Pass-6 can focus on substantive logic review
with clean substrate.

**Scope:** BC-2.06.017 v1.5 ↔ S-DEMO-MULTI-TENANT-DTU-001 v1.8 ↔ CLAUDE.md ↔ BC-INDEX.md
↔ STORY-INDEX.md ↔ convergence-state.json.

**Result: ALL 10 FINDINGS CLOSED (2 BLOCKER + 7 MAJOR + 1 MINOR).**

---

## Check Inventory (8 checks run)

| Check | Scope | Result |
|-------|-------|--------|
| 1. BC frontmatter field completeness | BC-2.06.017 v1.5 frontmatter | PASSED |
| 2. Story frontmatter field completeness | story v1.8 frontmatter | PASSED |
| 3. BC ↔ story cross-reference alignment | BC behavioral_contracts ↔ story | PASSED |
| 4. overlay wiring docs | BC Postcondition 2 + story AC-005/§File-Structure/Task-6 | FINDING (B-001/B-002 — CLOSED) |
| 5. BC-INDEX.md row alignment | BC-INDEX BC-2.06.017 row | FINDING (M-003 — CLOSED) |
| 6. STORY-INDEX.md row alignment | STORY-INDEX S-DEMO-MULTI-TENANT-DTU-001 row | PASSED |
| 7. convergence-state.json completeness | adversary-convergence-state.json | PASSED |
| 8. CLAUDE.md EXPECTED count | CLAUDE.md §Conventions #[non_exhaustive] sentence | FINDING (M-004 AUTO-RESOLVES) |

---

## BLOCKER Findings

### B-001 [BLOCKER CLOSED] — overlay-wiring docs: BC Postcondition 2 underspecified

**Finding:** BC-2.06.017 v1.4 Postcondition 2 described overlay wiring as writing only
`base_url`, but `OverlayLoader` / `INV-SCALAR-003` requires 3 REQUIRED fields: `extends`,
`instance_id`, `base_url`. An overlay file with only `base_url` would be rejected at load time
with E-SPEC-019.

**Resolution:** Product-owner amended BC v1.4→v1.5 (Postcondition 2 updated to enumerate all 3
required fields + INV-SCALAR-003 cross-ref; TV-017-009 added with 3-field overlay YAML example).

**Routing:** product-owner (BC content). **Status: CLOSED at D-1149 F-P5-MED-003.**

---

### B-002 [BLOCKER CLOSED] — overlay-wiring docs: story AC-005/§File-Structure/Task-6 underspecified

**Finding:** Story v1.7 AC-005, §File-Structure overlay YAML snippet, and Task-6 described
overlay wiring as `base_url:` only. Same root cause as B-001 — the spec and code diverged
silently because the code was already correct (Pass-1 implementation writes all 3 fields).

**Resolution:** Story-writer corrected AC-005/§File-Structure overlay YAML snippet/Task-6 to
enumerate all 3 required fields (`extends`, `instance_id`, `base_url`) per INV-SCALAR-003 and
BC-2.06.017 v1.5 Postcondition 2.

**Routing:** story-writer (story content). **Status: CLOSED at D-1149 F-P5-MED-003.**

---

## MAJOR Findings

### M-001 [MAJOR CLOSED] — BC-version-citation drift in story: 8 sites citing stale v1.1..v1.4

**Finding:** Story v1.7 §Behavioral Contracts table and inline citations referred to
"BC-2.06.017 v1.1", "v1.2", "v1.3", "v1.4" at 8 locations. BC was already at v1.5 after the
F-P5-MED-003 product-owner amendment.

**Resolution:** Story-writer performed full `grep -n "v1\.[1-4]"` sweep of story file; updated
§Behavioral Contracts table pin to "v1.5". Amendment-log historical entries preserved as
immutable audit trail (historical version citations in changelog rows are excepted per TD-VSDD-091).

**Routing:** story-writer. **Status: CLOSED at D-1149 F-P5-MED-001.**

---

### M-002 [MAJOR CLOSED] — H1 heading carries inline version stamp contradicting frontmatter

**Finding:** Story v1.7 H1 heading included inline version stamp "v1.4" while frontmatter
`version:` field read "1.7". Two version sources in the same document is a maintenance hazard.

**Resolution:** Story-writer made H1 heading version-agnostic (consistent with corpus pattern;
frontmatter `version:` is the sole authoritative version per TD-VSDD-091).

**Routing:** story-writer. **Status: CLOSED at D-1149 F-P5-MED-002.**

---

### M-003 [MAJOR CLOSED] — BC-INDEX.md inline narrative note: stale draft_contracts count

**Finding:** BC-INDEX.md line ~26 note read "draft_contracts: 2 covers BC-2.06.011 +
BC-2.21.001" but frontmatter (line ~11) correctly shows `draft_contracts: 3`. The third draft
contract (BC-2.06.017, added at D-1074) was missing from the inline note.

**Resolution:** State-manager corrected the inline note to "draft_contracts: 3 covers
BC-2.06.011 + BC-2.06.017 + BC-2.21.001". (This is state-manager domain — BC-INDEX
maintenance note, not spec content.)

**Routing:** state-manager (this burst — D-1150). **Status: CLOSED.**

---

### M-004 [MAJOR AUTO-RESOLVES AT MERGE] — CLAUDE.md EXPECTED count stale (52 vs 60)

**Finding:** CLAUDE.md §Conventions #[non_exhaustive] discipline sentence reads
"52 types currently enforced via the compile-fail gate... `ci.yml EXPECTED=52` is the authority"
but the worktree CLAUDE.md (in `.worktrees/S-DEMO-MULTI-TENANT-DTU-001/`) already reflects the
correct 60 count (updated per D-1145 as part of T6 delivery). The main branch CLAUDE.md at
`/Users/jmagady/Dev/prism/CLAUDE.md` still shows 52.

**Resolution:** AUTO-RESOLVES at merge. When S-DEMO-MULTI-TENANT-DTU-001 squash-merges to
develop, the worktree CLAUDE.md with `EXPECTED=60` will become the develop CLAUDE.md. No action
required in this burst. (Cross-reference: DEFER-CLAUDEMD-NONEXHAUSTIVE-COUNT-001 in STATE.md
drift items.)

**Routing:** No action required — merge-time resolution. **Status: AUTO-RESOLVES AT MERGE.**

---

### M-005 [MAJOR CLOSED] — commit 9b4f4154: B-001/B-002 code already correct

**Finding (code audit):** The 3-field overlay wiring (B-001/B-002) was not a code defect — the
implementation in prism-dtu-harness/src/overlay_wiring.rs already wrote all 3 required fields
(`extends`, `instance_id`, `base_url`) per INV-SCALAR-003. The defect was spec-only
underspecification.

**Resolution:** Confirmed code is correct via Pass-1 implementer delivery (commit 9b4f4154).
`just check` remained GREEN throughout passes 2–5 (code unchanged since Pass-1). No code fix
required.

**Routing:** implementer (verification). **Status: CLOSED — code correct since commit 9b4f4154.**

---

### M-006 [MAJOR CLOSED] — ArmisClone request counter type: AtomicUsize NOT AtomicU64

**Finding:** Multiple doc sites (story + BC) carried residual false claim that ArmisClone
request counter "mirrors ClarotyState" (AtomicU64). ArmisClone uses `AtomicUsize` (different
origin, different type). This was a semantic-accuracy gap caught in Pass-2.

**Resolution:** Implementer corrected ArmisClone doc comment at Pass-2 (F-P2-MED-002, D-1147).
Post-audit: story and BC are grep-clean of the false "mirrors ClarotyState" claim.

**Routing:** implementer (code doc comment). **Status: CLOSED at D-1147 F-P2-MED-002.**

---

### M-007 [MAJOR CLOSED] — socket_map() key type: (String,String) not (OrgSlug,SensorId)

**Finding:** BC-2.06.017 v1.3 Postcondition 2 described `socket_map()` returning
`HashMap<(OrgSlug,SensorId),SocketAddr>` (newtype keys). But U-004/D-1075 locked API uses
`(String,String)` keys, and the implementation matches the locked API. The spec incorrectly
cited the newtype form.

**Resolution:** Product-owner amended BC v1.3→v1.4 (Postcondition 2 corrected to
`(String,String)` keys with U-004 note explaining the rationale). Story-writer corrected
story §Locked API sketch similarly.

**Routing:** product-owner (BC content) + story-writer (story content). **Status: CLOSED at D-1147 F-P3-MED-001.**

---

## MINOR Findings

### N-001 [MINOR CLOSED] — story H1 BC table: version pin column missing from amendment log rows

**Finding (minor):** Story v1.7 amendment log had historical rows that cited BC-2.06.017
without version pins. This is a minor consistency gap; historical citations with no version pin
are acceptable (TD-VSDD-091 excepts historical changelog rows), but the last current-state row
should always cite the current version.

**Resolution:** Story-writer ensured the §Behavioral Contracts table "Current Version" row
correctly cites BC-2.06.017 v1.5 as part of the M-001 sweep. Historical rows preserved as-is.

**Routing:** story-writer. **Status: CLOSED as part of M-001 sweep.**

---

## Resolution Summary

| ID | Class | Resolution Route | Closed At |
|----|-------|-----------------|-----------|
| B-001 | BLOCKER | product-owner BC amendment (v1.4→v1.5 Postcondition 2 + TV-017-009) | D-1149 F-P5-MED-003 |
| B-002 | BLOCKER | story-writer story amendment (AC-005/§File-Structure/Task-6) | D-1149 F-P5-MED-003 |
| M-001 | MAJOR | story-writer BC-version-citation sweep (8 sites → v1.5) | D-1149 F-P5-MED-001 |
| M-002 | MAJOR | story-writer H1 version-agnostic | D-1149 F-P5-MED-002 |
| M-003 | MAJOR | state-manager BC-INDEX note fix (this burst D-1150) | D-1150 |
| M-004 | MAJOR | AUTO-RESOLVES AT MERGE (worktree CLAUDE.md already 60) | merge |
| M-005 | MAJOR | implementer — code correct since Pass-1 commit 9b4f4154 | D-1146 Pass-1 |
| M-006 | MAJOR | implementer — ArmisClone AtomicUsize doc corrected | D-1147 F-P2-MED-002 |
| M-007 | MAJOR | product-owner BC + story-writer story (String,String) key type | D-1147 F-P3-MED-001 |
| N-001 | MINOR | story-writer — swept as part of M-001 BC-version-citation sweep | D-1149 F-P5-MED-001 |

**Permanent fix:** BC version citations in story are now expressed as version-agnostic references
("BC-2.06.017 — current version in BC file frontmatter") per TD-VSDD-091. This ends the version-
drift citation class permanently — future BC amendments will no longer require story sweeps just
to update version pins.

---

## Post-Audit State

- BC-2.06.017: **v1.6** (this burst bumps BC-INDEX row from v1.5; no BC file content changed
  in this burst — the v1.5 amendments were made in D-1149; this burst's version bump in BC-INDEX
  reflects the D-1150 state-manager bookkeeping update)
- Story: **v1.9** (STORY-INDEX row updated; story version-agnostic citations now permanent)
- Code: **UNCHANGED since Pass-1** (commit 9b4f4154)
- `just check`: **GREEN** (4292 passed, 0 failed; EXPECTED=60 confirmed)
- BC-5.39.001 streak: **0/3**
- Next: **LOCAL adversary Pass-6** — all cross-doc drift substrate cleared; Pass-6 should focus
  on substantive logic, invariant completeness, and edge-case coverage

---

*Authored by state-manager, D-1150, 2026-06-13.*
