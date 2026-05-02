---
document_type: gate-step-report
gate_step: e
gate_step_name: consistency-validation
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
phase: 3
wave: 3
step: e
pass: 3
validator: consistency-validator
scope: "Wave 3 + Wave 3.1 + Wave 3.2 (4 merged fix PRs: #118 CODE-004, #119 SEC-002, #120 CODE-002, #121 CREDS-001) — post-W3.2 final consistency validation"
reviewer: consistency-validator
date: 2026-05-02
develop_sha: a7f0d374
verdict: CONDITIONAL_PASS
total_checks: 10
checks_pass: 6
checks_conditional: 2
checks_fail: 2
---

# Wave 3 Integration Gate — Gate Step E: Consistency Validation Pass 3
# Post-Wave-3.2 Fix-Wave Final Cross-Story Re-Validation

**Scope:** Wave 3 + Wave 3.1 + Wave 3.2 — all 46 stories (37 Wave 3 MT + 3 devx + 6 W3.1 + 1 ImplPhase + 4 W3.2: W3-FIX-CODE-004 #118, W3-FIX-SEC-002 #119, W3-FIX-CODE-002 #120, W3-FIX-CREDS-001 #121)
**Validator:** consistency-validator
**Date:** 2026-05-02
**develop SHA evaluated:** a7f0d374 (W3-FIX-CODE-002, PR #120 — develop HEAD per task spec)
**Wave base:** 6696e374^ (parent of PR #73, S-3.0.01 first merge)
**Verdict:** CONDITIONAL_PASS — 2 blocking drift items (E-CFG-018/019 absent from error-taxonomy.md; Full Story List MERGED annotations missing for 4 W3.2 stories); 1 arithmetic inconsistency in STORY-INDEX total_stories counter; all spec-traceability checks pass; TD entries documented correctly.

---

## Summary Table

| Check | Item | Verdict | Severity |
|-------|------|---------|----------|
| 1 | All 4 W3.2 stories registered in STORY-INDEX with status: merged + MERGED annotations | CONDITIONAL | MEDIUM |
| 2 | Spec traceability — W3.2 story behavioral_contracts reference valid BCs | PASS | — |
| 3 | TD-W3-TIMING-001 documented in tech-debt-register + cycle-manifest | PASS | — |
| 4 | TD-W3-CREDS-001 false-positive confirmed; register updated | PASS | — |
| 5 | E-CFG-019 InvalidOrgSlugPattern in error-taxonomy.md | FAIL | MEDIUM |
| 6 | CR-014 deviation documented (pub via #[doc(hidden)]) | PASS | — |
| 7 | Demo evidence per POL-010 for all 4 W3.2 stories | PASS | — |
| 8 | STATE.md v6.09 current (develop_head, pr_count, awaiting) | PASS | — |
| 9 | cycle-manifest Wave 3 + 3.1 + 3.2 closure status | PASS | — |
| 10 | Pass-49 / pass-50 trajectory in STATE.md wave_3_integration_gate_* | PASS | — |

**Additionally detected (not in task scope but blocking):**
| D-001 | E-CFG-018 SpecPathTraversal absent from error-taxonomy.md | FAIL | MEDIUM |
| D-002 | STORY-INDEX total_stories: 122 arithmetic error (actual: 125) | DRIFT | LOW |

---

## Check 1: All 4 W3.2 Stories Registered — CONDITIONAL_PASS

### Epic-view table (lines 190-196, STORY-INDEX.md)

All 4 W3.2 stories appear in the E-3.5 epic-view table with correct MERGED annotations:

| Story | PR | SHA | MERGED annotation present |
|-------|----|-----|--------------------------|
| W3-FIX-CODE-004 | #118 | 618ad644 | YES (line 196) |
| W3-FIX-SEC-002 | #119 | f89e7044 | YES (line 190) |
| W3-FIX-CODE-002 | #120 | a7f0d374 | YES (line 193) |
| W3-FIX-CREDS-001 | #121 | 9d04235d | YES (line 195) |

Status field in story files: all 4 read `status: merged`. PASS on epic-view and file status.

### Full Story List (lines 340-346, STORY-INDEX.md)

The Full Story List rows for the 4 W3.2 stories are present but LACK the `[MERGED PR #NNN SHA DATE]` inline annotation that the W3-FIX-G burst applied to Wave 3 MT stories and the W3.1 state hygiene burst applied to W3.1 stories:

| Story | Full Story List row (line) | MERGED annotation |
|-------|---------------------------|------------------|
| W3-FIX-SEC-002 | 340 | ABSENT |
| W3-FIX-CODE-002 | 343 | ABSENT |
| W3-FIX-CREDS-001 | 345 | ABSENT |
| W3-FIX-CODE-004 | 346 | ABSENT |

The W3.2 state hygiene burst changelog (STORY-INDEX.md line 82) states "MERGED annotations added to 4 W3.2 stories" — however the actual Full Story List rows do not reflect this. The annotations appear only in the epic-view table at the top of the E-3.5 section, not propagated to the Full Story List at lines 340-346.

**Remediation required:** Add `[MERGED PR #118 618ad644 2026-05-02]`, `[MERGED PR #119 f89e7044 2026-05-02]`, `[MERGED PR #120 a7f0d374 2026-05-02]`, `[MERGED PR #121 9d04235d 2026-05-02]` to the corresponding Full Story List title cells. This is a state-hygiene item.

**Verdict:** CONDITIONAL_PASS. Story files have `status: merged`. Epic-view table has MERGED annotations. Only Full Story List cells are missing annotations. Not a traceability gap — all BCs, counts, and story content are correct.

---

## Check 2: Spec Traceability — PASS

All 4 W3.2 stories have `behavioral_contracts:` frontmatter referencing BCs that exist as active entries in BC-INDEX.md v4.27.

### W3-FIX-CODE-004 (PR #118)
- `behavioral_contracts: [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.3.004, BC-3.2.001]`
- BC-INDEX.md lines 283-284 (BC-3.5.001/002), 290 (BC-3.6.001), 268 (BC-3.3.004), 255 (BC-3.2.001): all ACTIVE (status: draft, lifecycle active)
- PASS

### W3-FIX-SEC-002 (PR #119)
- `behavioral_contracts: [BC-3.5.001, BC-3.5.002, BC-3.2.001]`
- Note: story wave field reads `3.1` (not `3.2`); this is a pre-existing classification — SEC-002 was filed in the W3.1 wave but merged in the W3.2 batch. Not a blocking inconsistency.
- All 3 BCs exist in BC-INDEX: PASS

### W3-FIX-CODE-002 (PR #120)
- `behavioral_contracts: [BC-3.3.001, BC-3.3.004, BC-3.5.001, BC-3.5.002, BC-3.1.002]`
- Story frontmatter has 5 BCs. STORY-INDEX epic-view shows `BC-3.3.001,BC-3.3.004,BC-3.2.005` (3 BCs). BC Traceability Matrix row for BC-3.2.005 (line 566) does NOT include W3-FIX-CODE-002. Story frontmatter does NOT include BC-3.2.005.
- STORY-INDEX BC column for W3-FIX-CODE-002 (line 193) lists `BC-3.3.001,BC-3.3.004,BC-3.2.005`. Story frontmatter has `[BC-3.3.001, BC-3.3.004, BC-3.5.001, BC-3.5.002, BC-3.1.002]`. These are partially divergent: STORY-INDEX epic-view column omits BC-3.5.001/002/BC-3.1.002 and adds BC-3.2.005 which is absent from story frontmatter.
- All BCs in frontmatter are valid. BC-3.2.005 in STORY-INDEX column is not in frontmatter — this is a state-hygiene divergence (story frontmatter is authoritative per VSDD protocol).
- Not a blocking traceability failure; BC coverage is intact.
- PASS with note: STORY-INDEX epic-view BC column for W3-FIX-CODE-002 is partially divergent from story frontmatter (STORY-INDEX lists BC-3.2.005; frontmatter does not). Recommend aligning STORY-INDEX column to match story frontmatter in next hygiene pass.

### W3-FIX-CREDS-001 (PR #121)
- `behavioral_contracts: [BC-3.2.002]`
- BC-INDEX.md line 256: BC-3.2.002 ACTIVE (status: draft). PASS

---

## Check 3: TD-W3-TIMING-001 Documented — PASS

TD-W3-TIMING-001 appears in two locations with consistent content:

1. **cycle-manifest.md line 124**: `| TD-W3-TIMING-001 | HIGH-medium | BC-3.5.001/002 wall-clock budget tests fragile under workspace nextest parallelism. Test marked #[ignore] in #113. Follow-up: optimize harness build OR formally amend BC-3.5.001/ADR-011 D-058 OR migrate to Criterion benchmark. |`

2. **cycle-manifest.md line 150 (Tech Debt Status table)**: `| TD-W3-TIMING-001 | medium | ACTIVE FOLLOW-UP | BC-3.5.001/002 wall-clock tests #[ignore]; formal BC amendment or Criterion benchmark migration required before convergence |`

The entry is correctly classified as medium severity, status ACTIVE FOLLOW-UP (not blocking immediate gate), with explicit remediation path. PASS.

---

## Check 4: TD-W3-CREDS-001 Status Updated — PASS

TD-W3-CREDS-001 is correctly documented as resolved/closed:

1. **cycle-manifest.md line 125** (Wave 3.1 section): records original HIGH gap — `CredentialStoreOrgId` trait `todo!()` stubs; filed as W3-FIX-CREDS-001.

2. **cycle-manifest.md line 140**: `- TD-W3-CREDS-001: BC-3.2.002 trait impl FALSE POSITIVE confirmed; regression coverage added (PR #121)`

3. **cycle-manifest.md line 151 (Tech Debt Status table)**: `| TD-W3-CREDS-001 | resolved | CLOSED | BC-3.2.002 false-positive confirmed; regression coverage added in PR #121 (W3-FIX-CREDS-001) |`

The false-positive classification is recorded, the fix story (W3-FIX-CREDS-001 / PR #121) is referenced, and the status is CLOSED. PASS.

---

## Check 5: E-CFG-019 in error-taxonomy.md — FAIL

**Finding: E-CFG-019 (`InvalidOrgSlugPattern`) is ABSENT from error-taxonomy.md.**

W3-FIX-CODE-002 story body (line 93-95, 221, 300) defines and requires `E-CFG-019: InvalidOrgSlugPattern` for org_slug regex validation. The error code appears in:
- W3-FIX-CODE-002 body: `E-CFG-019: InvalidOrgSlugPattern` (multiple references)
- W3-FIX-CODE-002 Task 1 (line 300): `Add InvalidOrgSlugPattern variant E-CFG-019`

The error-taxonomy.md (v1.12) currently contains E-CFG codes 000-017, 020, 030, 031, 100-103. E-CFG-018 and E-CFG-019 are both absent (see Finding D-001 below for E-CFG-018).

**Severity:** MEDIUM — error taxonomy is the authoritative source for all error codes. An error code defined in a merged story's body but absent from the taxonomy means the spec corpus is inconsistent. Any downstream implementation, documentation, or holdout that references the taxonomy will not find E-CFG-019.

**Remediation required before pass-50 gate:**
Add to error-taxonomy.md after the E-CFG-017 row:

```
| E-CFG-018 | broken | security | "customers/{file}: E-CFG-018: spec path '{path}' traverses outside the customers directory" | No | `[[dtu]]` block `spec` field path contains `..` components or is absolute, causing the canonicalized path to escape the customers/ directory root. R-CUST-015 extension per W3-FIX-SEC-003 / BC-3.3.004. |
| E-CFG-019 | broken | configuration | "customers/{file}: E-CFG-019: org_slug '{value}' contains invalid characters; must match [a-zA-Z0-9_-]+" | No | `org_slug` field value contains characters outside the allowed pattern `[a-zA-Z0-9_-]+`. Added by W3-FIX-CODE-002 CR-003. BC-3.3.004 R-CUST-002 extension. |
```

Error taxonomy version must advance to v1.13.

---

## Check 6: CR-014 Deviation Documented — PASS

The `pub`-via-`#[doc(hidden)]` choice for `validate_spec_path` is documented in two authoritative locations:

1. **cycle-manifest.md line 138**: `- LOW: CR-014 (deviation accepted — kept pub via #[doc(hidden)] due to integration test usage), CR-015, SEC-P2-006`

2. **cycle-manifest.md line 144**: `- CR-014 deviation: validate_spec_path kept pub via #[doc(hidden)] (integration test usage)`

These appear under the Wave 3.2 Fix Wave Amendment closure notes. The justification (integration test usage) is recorded and the deviation is classified as LOW (accepted). PASS.

Note: The W3-FIX-CODE-004 story body and frontmatter include `parent_finding: "CR-010, CR-011, CR-012/SEC-P2-001, CR-013, CR-014, CR-015, ..."` which cross-references the finding. The deviation rationale is in the cycle-manifest, which is the appropriate home for cross-story decisions. PASS.

---

## Check 7: Demo Evidence per POL-010 — PASS

Verified via `docs/demo-evidence/` directory listing. All 4 W3.2 story directories are present:

```
docs/demo-evidence/W3-FIX-CODE-002/
docs/demo-evidence/W3-FIX-CODE-004/
docs/demo-evidence/W3-FIX-CREDS-001/
docs/demo-evidence/W3-FIX-SEC-002/
```

POL-010 compliance: all 4 W3.2 stories have demo-evidence subdirectories on the develop branch at SHA a7f0d374. PASS.

---

## Check 8: STATE.md v6.09 Current — PASS

STATE.md frontmatter confirms:

- `version: "6.09"` (line 4) — correct
- `develop_head: "a7f0d374"` (line 60) — matches task-specified develop HEAD
- `pr_count_merged: 121` (line 86) — reflects PRs #118-#121 all merged (4 W3.2 PRs on top of prior 117)
- `awaiting: "Wave integration gate pass-50 dispatch (adversary fresh-context + code-reviewer + security-reviewer + consistency-validator + holdout-evaluator). Goal: 3 consecutive CLEAN passes for convergence."` (line 26) — correctly describes next action

`current_step` (line 25) accurately describes the W3.2 closure state. The state snapshot is current. PASS.

---

## Check 9: cycle-manifest Closure Status — PASS

**Wave 3 (Multi-Tenant):** cycle-manifest.md line 105 — `Wave 3 (Multi-Tenant) CLOSED 2026-04-30`. All 37 stories referenced. PASS.

**Wave 3.1 Fix Wave Amendment:** cycle-manifest.md lines 109-127 — `Status: CLOSED`. PRs #113-#117 listed. develop HEAD on closure: `cda17ed4`. PASS.

**Wave 3.2 Fix Wave Amendment:** cycle-manifest.md lines 129-151 — `Status: CLOSED`. PRs #118 CODE-004, #119 SEC-002, #120 CODE-002, #121 CREDS-001 listed. develop HEAD on closure: `a7f0d374`. Tech Debt Status Update table present. PASS.

All three sub-waves are correctly recorded as CLOSED with PR numbers, SHAs, and closure dates. PASS.

---

## Check 10: Pass-49 / Pass-50 Trajectory in STATE.md — PASS

STATE.md `wave_3_integration_gate_*` block (lines 87-95):

- `wave_3_integration_gate_step_e` (line 90): records Pass 1 (CONDITIONAL_FAIL → PASS_POST_W3_FIX_G)
- `wave_3_integration_gate_pass_49` (line 95): records Pass 2 — `{ date: 2026-05-02, verdict: FINDINGS_OPEN_NEW_GAPS, h: 1, m: 7, l: 2, c_pass2_verdict: APPROVE_WITH_CONCERNS, d_pass2_verdict: APPROVED_WITH_CONDITIONS, e_pass2_verdict: CONDITIONAL_PASS, f_pass2_verdict: CONDITIONAL_PASS, mean_satisfaction: 0.75, must_pass_ratio: "18/30" ... }`
- `wave_3_integration_gate_status` (line 92): `"FINDINGS_OPEN — Wave 3.2 fix wave CLOSED 2026-05-02 (4 PRs #118-#121). develop@a7f0d374. Pass-50 dispatch queued. Goal: 3-clean-pass convergence."`

Pass-49 is recorded with full verdict breakdown. The current check (Pass 3 of gate step E) corresponds to the pass-50 dispatch. STATE.md correctly positions the project at pass-50 entry state. PASS.

Note: `wave_3_integration_gate_step_e` at line 90 was not updated to include pass-49's step-E verdict — that appears only in `wave_3_integration_gate_pass_49` at line 95. This is acceptable; the pass_49 block is the authoritative pass-level record.

---

## Additional Drift Findings (Outside Task Scope)

### D-001: E-CFG-018 SpecPathTraversal Absent from error-taxonomy.md — FAIL (MEDIUM)

W3-FIX-SEC-003 (PR #114, merged 2026-05-01) defines `E-CFG-018: SpecPathTraversal` for path traversal rejection. The story title itself is "path canonicalization + E-CFG-018 SpecPathTraversal rejection". The error code is referenced throughout the story body.

error-taxonomy.md v1.12 has E-CFG-001 through E-CFG-017, E-CFG-020, E-CFG-030/031, E-CFG-100-103. E-CFG-018 is absent.

This was already present as of W3-FIX-SEC-003 delivery (PR #114) and was a pre-existing gap not caught by Pass 2. It must be remediated together with E-CFG-019 (same pass, same file).

**Combined remediation:** Add both E-CFG-018 and E-CFG-019 to error-taxonomy.md §CFG-001..017 table; bump version to v1.13; update STATE.md `error_taxonomy_version` field.

### D-002: STORY-INDEX total_stories Arithmetic Inconsistency — DRIFT (LOW)

STORY-INDEX.md frontmatter declares `total_stories: 122`. The overview text at line 23 enumerates: 76 + 37 + 3 + 6 + 1 + 2 = 125. The file count on disk (excluding STORY-INDEX.md itself) is 125.

The `total_stories: 122` was last updated by the W3.1 state hygiene burst (which moved 119→120 per changelog line 80 for S-3.1.06-ImplPhase, and 120→122 by the W3.2 story-writer burst per line 81). The W3.2 state hygiene burst did not update `total_stories` to 125 because the W3.2 amendment counting is 122 + 0 net (W3-FIX-SEC-002 and W3-FIX-CODE-002 were already registered in the W3.1 group of 6 in the overview text but the W3.2 story-writer burst filed W3-FIX-CREDS-001 and W3-FIX-CODE-004 bringing the count from 120→122 — yet the arithmetic in the overview text sums to 125).

Root cause: the overview text breakdown includes "6 Wave 3.1 fix stories: W3-FIX-SEC-001/002/003 + W3-FIX-CODE-001/002/003" (6 stories) + "1 Wave 3.1 impl-phase story: S-3.1.06-ImplPhase" (1 story) + "2 Wave 3.2 fix stories: W3-FIX-CREDS-001 + W3-FIX-CODE-004" (2 stories) = 9 new stories beyond the original 76+37+3=116. 116 + 9 = 125. But the claimed total is 122.

The 122 was set when only W3-FIX-CREDS-001 + W3-FIX-CODE-004 were registered (burst D-185 set 120→122) but S-3.1.06-ImplPhase (burst D-184 set 119→120 per the W3.1 burst) and the original W3.1 group raised 113→116+6=122 via D-183 burst. The confusion is that S-3.1.06-ImplPhase + W3-FIX-CREDS-001 + W3-FIX-CODE-004 are all listed in the total breakdown but total_stories was not incremented to 125.

**Correct total_stories: 125.** Remediation: update frontmatter `total_stories: 122 → 125` and fix overview arithmetic parenthetical.

---

## BC Traceability Matrix Consistency Check

Spot-check of BC Traceability Matrix (lines 564-579, STORY-INDEX.md) for W3.2 BC references:

| BC | Expected coverage includes W3.2 story | Matrix entry (line) | Status |
|----|--------------------------------------|---------------------|--------|
| BC-3.2.001 | W3-FIX-CODE-004 | line 564 — includes W3-FIX-CODE-004 | PASS |
| BC-3.2.002 | W3-FIX-CREDS-001 | line 565 — includes W3-FIX-CREDS-001 | PASS |
| BC-3.3.001 | W3-FIX-CODE-002 | line 569 — does NOT include W3-FIX-CODE-002 | NOTE |
| BC-3.3.004 | W3-FIX-CODE-004 | line 572 — includes W3-FIX-CODE-004 | PASS |
| BC-3.5.001 | W3-FIX-CODE-004, W3-FIX-SEC-002 | line 577 — includes CODE-004; SEC-002 absent | NOTE |
| BC-3.5.002 | W3-FIX-CODE-004, W3-FIX-SEC-002 | line 578 — includes CODE-004; SEC-002 absent | NOTE |
| BC-3.6.001 | W3-FIX-CODE-004 | line 579 — includes W3-FIX-CODE-004 | PASS |

**Notes:** BC-3.3.001 matrix row (line 569) does not include W3-FIX-CODE-002 (which has BC-3.3.001 in frontmatter). BC-3.5.001/002 matrix rows do not include W3-FIX-SEC-002 (which has BC-3.5.001/BC-3.5.002 in frontmatter). These are state-hygiene gaps in the BC Traceability Matrix, not traceability failures (the story frontmatter is correct). Recommend adding in next hygiene burst alongside the Full Story List MERGED annotation fix.

---

## Gate Verdict Summary

**Verdict: CONDITIONAL_PASS**

**Blocking items (must be resolved before pass-50 clean-pass verdict):**

| ID | Item | Location | Action Required |
|----|------|----------|----------------|
| WGCV3-P3-001 | E-CFG-018 absent from error-taxonomy.md | `prd-supplements/error-taxonomy.md` | Add E-CFG-018 SpecPathTraversal row; bump to v1.13 |
| WGCV3-P3-002 | E-CFG-019 absent from error-taxonomy.md | `prd-supplements/error-taxonomy.md` | Add E-CFG-019 InvalidOrgSlugPattern row (same pass as WGCV3-P3-001) |

**Non-blocking state-hygiene items (should be resolved; will be flagged by pass-50 adversarial if outstanding):**

| ID | Item | Location | Action |
|----|------|----------|--------|
| WGCV3-P3-003 | Full Story List MERGED annotations missing for 4 W3.2 stories | `STORY-INDEX.md` lines 340-346 | Add `[MERGED PR #NNN SHA DATE]` to each title cell |
| WGCV3-P3-004 | STORY-INDEX total_stories: 122 should be 125 | `STORY-INDEX.md` frontmatter + overview line 23 | Update frontmatter + arithmetic in overview text |
| WGCV3-P3-005 | BC Traceability Matrix missing W3-FIX-CODE-002 in BC-3.3.001 row | `STORY-INDEX.md` line 569 | Add W3-FIX-CODE-002 to BC-3.3.001 row |
| WGCV3-P3-006 | BC Traceability Matrix missing W3-FIX-SEC-002 in BC-3.5.001/002 rows | `STORY-INDEX.md` lines 577-578 | Add W3-FIX-SEC-002 to both rows |
| WGCV3-P3-007 | STORY-INDEX epic-view BC column for W3-FIX-CODE-002 lists BC-3.2.005; story frontmatter does not | `STORY-INDEX.md` line 193 | Align STORY-INDEX column to match story frontmatter (remove BC-3.2.005; add BC-3.5.001/002/BC-3.1.002) OR update story frontmatter to include BC-3.2.005 with justification |

**Items confirmed clean:**

- All 4 W3.2 story files have `status: merged`
- All 4 W3.2 stories have correct `behavioral_contracts` referencing active BCs
- TD-W3-TIMING-001: formally documented as ACTIVE FOLLOW-UP in cycle-manifest
- TD-W3-CREDS-001: formally documented as CLOSED / false positive in cycle-manifest
- CR-014 deviation: formally documented in cycle-manifest
- Demo evidence: all 4 W3.2 story directories present in `docs/demo-evidence/`
- STATE.md v6.09: develop_head, pr_count, awaiting all reflect post-W3.2 state
- cycle-manifest: Wave 3, 3.1, and 3.2 all recorded as CLOSED
- STATE.md pass-49 block: fully populated with per-step verdicts; pass-50 trajectory correctly expressed

---

## Recommendation for Pass-50 Dispatch

Pass-50 can proceed immediately with the following precondition: **the product-owner must add E-CFG-018 and E-CFG-019 to error-taxonomy.md before or alongside the pass-50 adversarial dispatch.** The error taxonomy gap (WGCV3-P3-001/002) is a spec-corpus inconsistency that will surface as a finding in any adversarial or holdout pass that cross-references the taxonomy.

The state-hygiene items (WGCV3-P3-003 through WGCV3-P3-007) can be addressed in a single state-hygiene burst that accompanies or follows the error-taxonomy fix, prior to the pass-50 adversarial verdict being filed.

If both the error-taxonomy fix and state-hygiene burst are applied before pass-50 adversarial completes, pass-50 step-E should return PASS (no blocking findings).
