---
document_type: adversarial-review
cycle: wave-0-plugin-prereqs
story_id: S-DTU-CYBERINT-AUTH-FIDELITY-001
pass: PR-9
type: PR-LEVEL
lens: policy+scope
parallel_passes: "7, 8, 9 ran simultaneously on frozen HEAD 3e0fe7f8 (diverse lenses for coverage)"
date: 2026-05-30
feature_head: "3e0fe7f8"
pr_number: 164
base_branch: develop
base_head: "e898c3c9"
diff_artifact_supplied: true
worktree_path_discipline: true
clean_strict: false
clean_pr_merge: false
findings_count: 5
findings_by_severity:
  MED: 1
  LOW: 2
  OBS: 2
streak_after_pass: 0
target_streak: 3
status: "F-PR9-MED-001/LOW-001/LOW-002 CLOSED by FB-PR5 demo-recorder 7d05cdb7; OBS findings verified closed"
---

# PR-LEVEL Adversary Pass 9 — S-DTU-CYBERINT-AUTH-FIDELITY-001 PR #164

## Header

- **Pass:** PR-LEVEL Pass 9 (parallel re-convergence attempt — policy+scope lens)
- **Date:** 2026-05-30
- **Feature HEAD at review:** 3e0fe7f8 (FB-PR4: SEC-001/SEC-002 hardening + AC-011 evidence + PR body correction)
- **PR:** #164 (feature/S-DTU-CYBERINT-AUTH-FIDELITY-001 → develop)
- **Base develop HEAD:** e898c3c9
- **Diff artifact:** SUPPLIED (worktree-path discipline applied)
- **D-829 bundling context supplied:** YES
- **Parallel passes:** Passes 7, 8, and 9 ran simultaneously on frozen HEAD 3e0fe7f8 with diverse review lenses. Collectively they surfaced findings; re-convergence attempt FAILED. All findings closed by FB-PR5.
- **CLEAN(strict):** NO (1 MED + 2 LOW + 2 OBS findings)
- **CLEAN(PR-merge):** NO (1 MED finding: F-PR9-MED-001 contradicted evidence file)

## Findings

### F-PR9-MED-001 [MED] — evidence-report.md AC-011 Cells Contradicted Corrected AC-011 File

**Severity:** MED
**Status:** CLOSED by FB-PR5 demo-recorder commit 7d05cdb7

**Description:** The demo evidence report `docs/demo-evidence/S-DTU-CYBERINT-AUTH-FIDELITY-001/evidence-report.md` AC-011 row still contained the original text "zero results — confirmed no event_type emissions outside BC-2.16.002 catalog" phrasing that was authored against the pre-FB-PR4 state. After FB-PR4 updated `AC-011-no-uncatalogued-event-type.txt` to reflect the corrected verification methodology, the evidence-report.md summary cell was not updated to match. The evidence-report.md and the AC-011 evidence file were internally inconsistent: the evidence file (AC-011-no-uncatalogued-event-type.txt) described the new stable-ref methodology; evidence-report.md still cited the old "zero results" phrasing that corresponded to the pre-FB-PR4 evidence.

**Root cause:** Demo-recorder FB-PR4 burst updated the standalone AC-011 evidence file but did not propagate the change to the evidence-report.md summary table row.

**Resolution (FB-PR5):** Demo-recorder commit 7d05cdb7 updated the evidence-report.md AC-011 summary cell to match the corrected AC-011 evidence file. Permanent class fix: all evidence files now use stable `PR#164/story-v1.8` references instead of volatile HEAD-SHA pins, eliminating the future-staleness vector.

---

### F-PR9-LOW-001 [LOW] — Stale Volatile HEAD-SHA Pins in Evidence Headers (Recurring Class — 3 occurrences)

**Severity:** LOW
**Status:** CLOSED by FB-PR5 demo-recorder commit 7d05cdb7

**Description (TD-VSDD-091 evidence-file variant):** Policy+scope lens audit found volatile HEAD-SHA pins in evidence file headers occurring in 3 separate evidence files. Evidence headers cited specific HEAD SHAs (e.g., `d09bdfa9`, `b3aa0970`) that are now stale after FB-PR4 advanced the HEAD to `3e0fe7f8`. This is the same volatile-pin class as OBS-PR7-001 (found in pass 7, catalog+index lens) and F-PR8-LOW-001 context.

**Locations:** AC-007-build-request-cookie-injection.txt, AC-008-end-to-end-parity.txt, AC-009-negative-parity-cyberint-session.txt headers — all cited stale HEAD SHAs.

**Permanent fix applied:** FB-PR5 demo-recorder 7d05cdb7 replaced all volatile HEAD-SHA pins in all 11 evidence files with stable `PR#164/v1.8` references. This permanently closes the class — future HEAD advances will not stale the evidence headers.

---

### F-PR9-LOW-002 [LOW] — AC-010 Evidence Title Line Listed E-AUTH-004/006 Instead of E-AUTH-005/006/007

**Severity:** LOW
**Status:** CLOSED by FB-PR5 demo-recorder commit 7d05cdb7

**Description:** The `AC-010-error-taxonomy-compliance.txt` evidence file title line read "Tests: E-AUTH-004 (missing_token), E-AUTH-006 (allowlist_reject)" which was stale relative to the corrected error taxonomy. After FB-PR2 established the correct error code mapping (E-AUTH-005 for missing-cookie-token, E-AUTH-006 for allowlist-reject, E-AUTH-007 for empty-token), the AC-010 evidence file title retained the pre-correction codes (E-AUTH-004/006). The story body and BC-2.01.017 were corrected in FB-PR2 but this evidence file header was missed.

**Resolution (FB-PR5):** Demo-recorder commit 7d05cdb7 corrected the AC-010 evidence title line to "Tests: E-AUTH-005 (missing_token), E-AUTH-006 (allowlist_reject), E-AUTH-007 (empty_token)" matching BC-2.01.017 §Edge Cases authoritative mapping.

---

### OBS-PR9-001 [OBS] — PR Description "Story version" Field Showed v1.5

**Severity:** OBS
**Status:** CLOSED by FB-PR5 pr-manager (GitHub PR #164 body already updated by pr-manager during pr-reviewer-1 IMP-2 fix; .factory copy updated in this burst)

**Description:** The PR #164 body "AI Pipeline Metadata" table showed `Story version | v1.5` which was stale (story is now v1.7 at Pass 9 review time, and will be v1.8 after FB-PR5). The live GitHub PR #164 body was updated by pr-manager during the pr-reviewer-1 IMP-2 correction, which updated the story version field. The `.factory/code-delivery/S-DTU-CYBERINT-AUTH-FIDELITY-001/pr-description.md` copy is updated in this burst.

---

### OBS-PR9-002 [OBS] — Scope Clarity: boot.rs Functional Changes Claim

**Severity:** OBS
**Status:** Already CLOSED by pr-manager (GitHub PR #164 body) + .factory mirror (this burst)

**Description:** Policy+scope lens re-verified the scope note added in IMP-2 fix: "boot.rs diff contains only cargo-fmt import-regrouping (no functional change)" is correctly stated in the corrected PR description. The previous OBS was that the original PR body claimed "boot step 9A constructs StaticCookieAuthProvider" which overstated scope. The corrected PR body (from pr-manager IMP-2 fix at 3e0fe7f8) accurately states boot.rs changes are fmt-only. No residual scope overstatement. Confirmed CLOSED.

---

## Probe Results

### SAP-1 — Tracing Emission Catalog Completeness

**Result: PASS**

All `event_type` emissions across `crates/` workspace verified against BC-2.16.002 v1.60 catalog (68 entries). Policy+scope lens focus: verified that PR description's claimed catalog compliance (AC-011) matches actual catalog state. PASS.

### SAP-2 — DTU/TOML Schema Parity

**Result: PASS**

No TOML or DTU struct changes in FB-PR4. All columns match DTU struct fields.

### POL-7 Sweep (policy+scope lens)

**Result: PASS (after FB-PR5)**

Canonical naming: `StaticCookieAuthProvider` (no spaces) used consistently throughout code, BC files, story, and evidence. F-PR8-LOW-001 (stray space in BC table) closed by FB-PR5 story-writer 9e18624b — confirmed CLOSED.

### Scope Compliance Review

**Result: PASS**

Story scope boundary verified: StaticCookieAuthProvider + DTU route correction + pipeline wiring = correctly scoped per story AC-001 through AC-011. Boot-time (binary `prism start`) routing is correctly excluded from scope (deferred to S-DEMO-001 GAP-002-A-gated). No scope creep found.

## Streak Accounting

- Passes 4/5/6: CLEAN(strict)=YES, streak 3/3 (d09bdfa9). Re-opened by FB-PR4.
- Pass 7 (parallel): CLEAN(strict)=NO. Streak: 0/3.
- Pass 8 (parallel): CLEAN(strict)=NO. Streak: 0/3.
- **Pass 9 (parallel): CLEAN(strict)=NO. MED + 2 LOW + 2 OBS. Streak: 0/3.** CLEAN(PR-merge)=NO (F-PR9-MED-001 evidence contradiction).
- All findings across 7/8/9 closed by FB-PR5 (implementer 44aa7fed + story-writer 9e18624b + demo-recorder 7d05cdb7). HEAD advanced to 7d05cdb7.

## Permanent Class Fix Established

Evidence files now use stable `PR#164/v1.8` references instead of volatile HEAD-SHA pins (closed the 3x-recurring staleness class found across passes 7, 8, 9). Future HEAD advances will not stale the evidence headers. TD-VSDD-091 anti-volatile-pin discipline applied to evidence files as well as code comments.

## Next Action

All FB-PR5 specialist work complete. HEAD advanced to 7d05cdb7. Streak 0/3. Dispatch PR-LEVEL passes 10-12 on HEAD 7d05cdb7 for fresh re-convergence attempt.
