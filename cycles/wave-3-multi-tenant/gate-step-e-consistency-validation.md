---
document_type: gate-step-report
gate_step: e
gate_step_name: consistency-validation
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
phase: 3
wave: 3
step: e
validator: consistency-validator
scope: "37 Wave 3 MT stories + 3 W3-FIX stories + STORY-INDEX + BC-INDEX + VP-INDEX + ARCH-INDEX + verification-coverage-matrix + cycle-manifest"
reviewer: vsdd-factory:consistency-validator
date: 2026-05-01
develop_sha: a3bd5a0f
verdict: CONDITIONAL_FAIL
total_items: 15
critical: 1
high: 4
medium: 2
low: 2
clean: 6
---

# Wave 3 Integration Gate — Gate Step E: Consistency Validation

**Scope:** 37 Wave 3 MT stories (S-3.0.01/02 + S-3.1.01–07 + S-3.2.01–08 + S-3.3.01–06 + S-3.4.01–05 + S-3.5.01 + S-3.6.01/02 + S-3.7.00–05) + 3 W3-FIX stories + STORY-INDEX v1.71 + BC-INDEX v4.26 + VP-INDEX v1.19 + ARCH-INDEX v1.8 + verification-coverage-matrix v1.22 + cycle-manifest
**Reviewer:** vsdd-factory:consistency-validator
**Date:** 2026-05-01
**develop SHA:** a3bd5a0f
**Wave base:** 6696e374^ (parent of PR #73, S-3.0.01 first merge)
**Verdict:** CONDITIONAL FAIL — 1 CRITICAL + 4 HIGH blocking items

---

## Check Summary

| Check | Result | Findings |
|-------|--------|----------|
| 1. Spec traceability — story BCs exist in BC-INDEX with matching titles | PASS | 0 issues |
| 2. BC coverage — all Wave 3 BCs covered by stories; no empty behavioral_contracts | PASS (with note) | S-3.0.01 empty BC is intentional |
| 3. Story dependency integrity — depends_on chains are DAG; all predecessors exist | PASS | 0 issues |
| 4. Anchor consistency — anchor_subsystem IDs valid; anchor_bcs resolve | PASS | 0 issues |
| 5. Naming sweep — TenantId/OrgSlug consistent post-rename | PASS | 1 intentional alias |
| 6. Frontmatter freshness — status field reflects merged state | CRITICAL FAIL | 36 stories status: draft |
| 7. Index completeness — STORY-INDEX includes all stories; BC-INDEX complete; VP-INDEX consistent | HIGH FAIL (×3) | 0 MERGED annotations; 3 W3-FIX unregistered; BC-INDEX pin stale |
| 8. Demo evidence coverage | PASS | 37/37 dirs present with evidence-report + GIFs |

---

## CRITICAL Finding (Blocking)

### WGCV-W3-001 (CRITICAL): 36 of 37 Wave 3 MT Story Files Have `status: draft` Despite Being Merged

**Files affected (36 of 37):**

All Wave 3 MT story files except S-3.2.03 retain `status: draft` in frontmatter line 11. The 36 affected files are:

- `.factory/stories/S-3.0.01-lefthook-fmt-hook-fix.md` line 11: `status: draft`
- `.factory/stories/S-3.0.02-dtu-mode-metadata.md` line 11: `status: draft`
- `.factory/stories/S-3.1.01-org-id-newtype.md` line 11: `status: draft`
- `.factory/stories/S-3.1.02-tenant-id-to-org-slug-rename.md` line 11: `status: draft`
- `.factory/stories/S-3.1.03-org-registry.md` line 11: `status: draft`
- `.factory/stories/S-3.1.04-credentials-org-id-boundary.md` line 11: `status: draft`
- `.factory/stories/S-3.1.05-spec-engine-org-id-boundary.md` line 11: `status: draft`
- `.factory/stories/S-3.1.06-sensors-org-id-boundary.md` line 11: `status: draft`
- `.factory/stories/S-3.1.07-audit-org-fields.md` line 11: `status: draft`
- `.factory/stories/S-3.2.01-claroty-state-segregation.md` line 11: `status: draft`
- `.factory/stories/S-3.2.02-armis-state-segregation.md` line 11: `status: draft`
- `.factory/stories/S-3.2.04-cyberint-state-segregation.md` line 11: `status: draft`
- `.factory/stories/S-3.2.05-slack-dtu-org-tagging.md` line 11: `status: draft`
- `.factory/stories/S-3.2.06-pagerduty-dtu-org-tagging.md` line 11: `status: draft`
- `.factory/stories/S-3.2.07-jira-dtu-org-tagging.md` line 11: `status: draft`
- `.factory/stories/S-3.2.08-prism-query-crowdstrike-session-id-org-scoping.md` line 11: `status: draft`
- `.factory/stories/S-3.3.01-customer-config-schema.md` line 11: `status: draft`
- `.factory/stories/S-3.3.02-org-registry-boot.md` line 11: `status: draft`
- `.factory/stories/S-3.3.03-harness-logical-isolation.md` line 11: `status: draft`
- `.factory/stories/S-3.3.04-harness-network-isolation.md` line 11: `status: draft`
- `.factory/stories/S-3.3.05-harness-builder-ergonomics.md` line 11: `status: draft`
- `.factory/stories/S-3.3.06-reload-config-mode-change-rejection.md` line 11: `status: draft`
- `.factory/stories/S-3.4.01-migrate-claroty-tests.md` line 11: `status: draft`
- `.factory/stories/S-3.4.02-migrate-armis-tests.md` line 11: `status: draft`
- `.factory/stories/S-3.4.03-migrate-crowdstrike-tests.md` line 11: `status: draft`
- `.factory/stories/S-3.4.04-migrate-cyberint-tests.md` line 11: `status: draft`
- `.factory/stories/S-3.4.05-migrate-slack-pagerduty-jira-tests.md` line 11: `status: draft`
- `.factory/stories/S-3.5.01-src-convention-sweep.md` line 11: `status: draft`
- `.factory/stories/S-3.6.01-hs-006-refresh.md` line 11: `status: draft`
- `.factory/stories/S-3.6.02-hs-007-refresh.md` line 11: `status: draft`
- `.factory/stories/S-3.7.00-schema-derive-armis-crowdstrike.md` line 11: `status: draft`
- `.factory/stories/S-3.7.01-archetype-catalog-genopts.md` line 11: `status: draft`
- `.factory/stories/S-3.7.02-claroty-generator.md` line 11: `status: draft`
- `.factory/stories/S-3.7.03-cyberint-generator.md` line 11: `status: draft`
- `.factory/stories/S-3.7.04-armis-generator.md` line 11: `status: draft`
- `.factory/stories/S-3.7.05-crowdstrike-generator.md` line 11: `status: draft`

**Exception:** `S-3.2.03-crowdstrike-state-segregation.md` correctly shows `status: merged`, but has no corresponding MERGED annotation in STORY-INDEX (see WGCV-W3-002).

**Evidence:** All 36 stories are confirmed merged via `git log develop`. PRs #73–#112 cover S-3.0.01 through W3-FIX-CI-001 (2026-04-28 through 2026-04-30). STORY-INDEX `current_step` field (STATE.md line 25) reads "Wave 3 FULLY CLOSED — W3-FIX-CI-001 merged (PR #112, a3bd5a0f)."

**VSDD convention:** Story frontmatter `status:` is the authoritative per-story lifecycle record. `status: draft` means "not implemented." The STORY-INDEX timestamp of `2026-04-27T00:00:00` (pre-implementation) confirms no W3-FIX-G equivalent was executed post-implementation for Wave 3.

**Precedent:** This is the identical pattern as WGCV-W2-001 (Wave 2 gate). Wave 2 was resolved via W2-FIX-G (state-manager only, pure factory-artifacts update).

**Remediation:** Dispatch W3-FIX-G (state-manager only):
1. Update all 36 story files: `status: draft` → `status: merged`
2. Close WGCV-W3-001

---

## HIGH Findings (Blocking)

### WGCV-W3-002 (HIGH FAIL): STORY-INDEX Has Zero MERGED Annotations for Any Wave 3 MT Story

**Evidence:**
- `grep "\[MERGED" .factory/stories/STORY-INDEX.md | grep "S-3\."` returns 0 results
- Wave 2 comparison: S-2.01 through S-2.08 and S-6.11/12/13 all carry `[MERGED PR #NNN SHA DATE +Nt]` annotations in their title cells (e.g., STORY-INDEX line 237: `S-2.01 | RocksDB Initialization ... [MERGED PR #43 0d24ab79 2026-04-24 +24t]`)
- 37 Wave 3 MT story rows in STORY-INDEX (both the Epic-view tables lines 126–197 and the Full Story List lines 281–317) have no MERGED annotations despite all 37 PRs being merged
- STORY-INDEX version is v1.71 (timestamp 2026-04-27T00:00:00), predating Phase 3.B implementation which ran 2026-04-28 to 2026-04-30

**Affected rows (sample):** STORY-INDEX lines 126–127 (E-3.0), 133–139 (E-3.1), 145–152 (E-3.2), 158–163 (E-3.3), 169–173 (E-3.4), 179 (E-3.5), 185–186 (E-3.6), 192–197 (E-3.7), and Full Story List rows 281–317.

**Remediation:** As part of W3-FIX-G: add `[MERGED PR #NNN SHA DATE +Nt]` annotations for all 37 story rows; bump STORY-INDEX to v1.72 with changelog entry.

The PR/SHA/date/test-count facts for all 37 stories are available in STATE.md (lines 87–324) and the burst-log.md.

---

### WGCV-W3-003 (HIGH FAIL): 3 W3-FIX Stories Unregistered in STORY-INDEX; Use Invalid Subsystem SS-00

**Files affected:**
- `.factory/stories/W3-FIX-CI-001-ci-wall-clock-optimization.md` — not in STORY-INDEX; `subsystems: [SS-00]` line 8; `status: ready` line 14
- `.factory/stories/W3-FIX-LEFTHOOK-001-pre-push-gate-tuning.md` — not in STORY-INDEX; `subsystems: [SS-00]` line 8; `status: ready` line 14
- `.factory/stories/W3-FIX-WIN-001-port-release-windows-cross-platform.md` — not in STORY-INDEX; `status: ready` line 14

**STORY-INDEX impact:** `total_stories: 113` frontmatter and overview narrative cite "37 Wave 3 Multi-Tenant stories" — the W3-FIX devx stories are deliberately excluded from the VSDD story count (same policy as Wave 2 fix-PRs). However, per VSDD criterion 31 (Accumulated story count matches STORY-INDEX), ALL story files that implement behavioral scope require index registration. W3-FIX stories are merged, have demo evidence, and implement platform scope.

**SS-00 violation:** ARCH-INDEX Subsystem Registry (lines 111–134) lists SS-01 through SS-21. SS-00 does not exist. W3-FIX-CI-001 and W3-FIX-LEFTHOOK-001 reference `subsystems: [SS-00]` which is an invalid subsystem anchor with no resolvable architecture backing.

**W3-FIX-WIN-001 status:** Frontmatter `status: ready` (line 14) — the story was merged (PR #105, ea90c9ee) but frontmatter was not updated.

**Remediation options:**
- Option A: Register all 3 W3-FIX stories in STORY-INDEX under a "E-3.devx" or similar epic; correct SS-00 → SS-01 (prism-dtu-harness scope for W3-FIX-WIN-001) and remove/replace SS-00 in W3-FIX-CI-001/LEFTHOOK-001 with the closest applicable SS (or declare them as cross-cutting infra with no SS anchor per Wave 2 convention)
- Option B (lighter): Declare W3-FIX devx stories exempt from STORY-INDEX registration (matching W2-FIX-G/H/I pattern which were registered in STORY-INDEX as W2-FIX-* rows) — but SS-00 must still be corrected in all 3 files and status: ready → merged for W3-FIX-WIN-001

---

### WGCV-W3-004 (HIGH FAIL): cycle-manifest.md Shows `status: in-progress` With All-TBD Delivery Metrics

**File:** `.factory/cycles/wave-3-multi-tenant/cycle-manifest.md` (frontmatter line 7: `status: in-progress`; body lines 14–23 all read "TBD")

**Evidence:**
- `status: in-progress` despite STATE.md declaring "Wave 3 FULLY CLOSED" (STATE.md line 25) and `wave_3_closed: 2026-04-30` (STATE.md line 88)
- Delivery table shows: `Stories delivered | TBD`, `BCs created | TBD`, `VPs created | TBD`
- Actual values available: 37 stories (S-3.0.01/02 + full E-3.1 through E-3.7) + 3 W3-FIX; 22 BCs (BC-3.1.001–004 through BC-3.7.001); 74 new VPs (VP-063–VP-136)

**Remediation:** Update cycle-manifest.md: `status: in-progress` → `status: closed`; populate the Delivered metrics table from STATE.md and burst-log.md.

---

## MEDIUM Findings (Non-Blocking)

### WGCV-W3-005 (MEDIUM): STORY-INDEX BC-INDEX Version Pin Stale at v4.17; Current BC-INDEX Is v4.26

**Location:** `.factory/stories/STORY-INDEX.md` line 25 — `"222 active per BC-INDEX.md v4.17"` and STORY-INDEX Wave Summary note — `"Unique active BCs = 222 (per BC-INDEX.md v4.17, 222 active contracts)"`

**Current BC-INDEX version:** `v4.26` (BC-INDEX frontmatter line 6).

**Severity assessment:** MEDIUM rather than HIGH because the active contract count (222) has not changed since v4.17 — only metadata/changelog edits have occurred in versions v4.18 through v4.26. The pin is stale but the numeric assertions derived from it remain correct.

**Remediation:** Update STORY-INDEX v1.71 → v1.72: replace both occurrences of `v4.17` with `v4.26` in the overview and Wave Summary sections. Bundle into W3-FIX-G.

---

### WGCV-W3-006 (MEDIUM, OBSERVATION): All 22 Wave 3 BCs Retain `PROPOSED` Status Post-Implementation

**Location:** BC-INDEX lines 246–297; all 22 Wave 3 BC rows show `Status: PROPOSED`.

**Assessment:** Wave 3 BCs were authored during Phase 3.A with `v0.2 PROPOSED` or `v0.3 PROPOSED` status. Implementation is complete (all implementing stories merged) but BC lifecycle was not updated to reflect active/draft status post-implementation.

**Comparison:** Wave 1-2 BCs show `status: draft` in BC-INDEX (not `PROPOSED`). The PROPOSED→draft transition was not executed for Wave 3 BCs as part of the Wave 3 closure.

**Non-blocking rationale:** BC lifecycle promotion is a factory convention rather than a gate-blocking criterion in this project. No downstream artifact validation fails as a result. Recording as MEDIUM observation.

**Remediation:** As part of Wave 3 post-processing or W3-FIX-G, update all 22 Wave 3 BC file frontmatter from `status: PROPOSED` → `status: draft` and update BC-INDEX Status column accordingly. Bump BC-INDEX to v4.27.

---

## LOW Findings (Acceptable)

### WGCV-W3-007 (LOW, ACCEPTABLE): STORY-INDEX Timestamp Does Not Reflect Wave 3 Implementation Phase

**Location:** `.factory/stories/STORY-INDEX.md` frontmatter line 7: `timestamp: 2026-04-27T00:00:00` (pre-implementation; Phase 3.B ran 2026-04-28 to 2026-04-30).

**Assessment:** STORY-INDEX was not re-timestamped during implementation batches. The timestamp reflects the last spec-phase update (v1.71, Phase 3.A approved 2026-04-28). This is acceptable per the VSDD pattern where STORY-INDEX is updated at gate boundaries by state-manager, not incrementally per-PR. The timestamp will be corrected as part of W3-FIX-G (post-gate state-manager update).

---

### WGCV-W3-008 (LOW, ACCEPTABLE): TenantId Alias Exposed in prism-core lib.rs

**Location:** `crates/prism-core/src/lib.rs` line 98: `pub use tenant::TenantId;`

**Assessment:** S-3.1.02 (TenantId→OrgSlug rename, PR #93) intentionally retained `pub type TenantId = OrgSlug` as a Wave 3 deprecation alias per D-157. The alias is the only non-comment `TenantId` reference in the codebase after the rename. Removal is planned for Wave 4. This is consistent with ADR-006 §9 and D-157. No naming drift; Wave 3 code uses OrgSlug throughout.

---

## HIGH Items — CLEAN (Non-Blocking Validations)

### WGCV-W3-009 (HIGH but CLEAN): Spec Traceability — All 22 Wave 3 BCs Exist in BC-INDEX and Have Story Coverage

All 22 Wave 3 BCs (BC-3.1.001–004, BC-3.2.001–005, BC-3.3.001–004, BC-3.4.001–004, BC-3.5.001–002, BC-3.6.001–002, BC-3.7.001) verified:
- Present in BC-INDEX v4.26 (lines 246–297)
- Corresponding `.factory/specs/behavioral-contracts/BC-3.*.md` files exist (22 files confirmed)
- H1 titles in BC files match BC-INDEX title column (spot-checked BC-3.1.001, BC-3.1.002, BC-3.2.001, BC-3.2.005, BC-3.7.001)
- Every Wave 3 BC covered by at least one story (Python sweep: 0 uncovered BCs)

Conversely, all Wave 3 story `behavioral_contracts:` arrays reference valid BC IDs. The only story with `behavioral_contracts: []` is S-3.0.01 (lefthook devx fix) — intentional per VSDD convention for infra stories, consistent with S-2.08 precedent.

---

### WGCV-W3-010 (HIGH but CLEAN): Story Dependency Integrity — All depends_on References Resolve

Python sweep of all 37 Wave 3 MT story `depends_on:` fields confirms zero missing predecessor files. Cross-wave dependencies (S-1.06, S-6.06–S-6.13) all have story files in `.factory/stories/`. All cross-wave predecessors confirmed merged (`status: merged` in their frontmatter). No DAG cycles detected.

---

### WGCV-W3-011 (HIGH but CLEAN): Anchor Consistency — Subsystem IDs and BC References Valid

All subsystem IDs used in Wave 3 MT story frontmatter (`SS-01`, `SS-02`, `SS-03`, `SS-04`, `SS-05`, `SS-06`, `SS-11`, `SS-21`) are present in ARCH-INDEX Subsystem Registry (lines 111–134). No invalid SS-IDs in the 37 S-3.x.x files. All `anchor_bcs` references resolve to BC files.

Note: W3-FIX-CI-001 and W3-FIX-LEFTHOOK-001 use `SS-00` which is invalid — reported separately as WGCV-W3-003.

---

### WGCV-W3-012 (HIGH but CLEAN): Naming Sweep — OrgSlug/OrgId Consistent Post-Rename

`git diff 6696e374^..a3bd5a0f` analysis of Rust source additions:
- All new Wave 3 code uses `OrgId` / `OrgSlug` (not `TenantId`)
- Only non-comment additions of `TenantId` are: (1) the deprecation module comment in prism-core/src/lib.rs, (2) the `pub type TenantId = OrgSlug` alias line, and (3) test file comments migrating old test names. All intentional per D-157.
- Zero `CustomerId` or `OrgId`-as-`CustomerId` confusion in Wave 3 diff. ADR-006/ADR-008 OrgId/OrgSlug semantics consistent across all 37 implementation PRs.

---

### WGCV-W3-013 (HIGH but CLEAN): VP Arithmetic Consistent Across VP-INDEX and Coverage Matrix

VP-INDEX v1.19 totals: Kani=30, Proptest=77, Unit=4, Fuzz=6, Integration=19, Total=136.

Coverage-matrix v1.22 per-module sums: Kani=30, Proptest=77, Unit=4, Fuzz=6, Integration=19, Total=136.

Per-method totals match exactly. Coverage matrix P0/P1 split (113 P0 / 23 P1) consistent with VP-INDEX Summary table. STORY-INDEX `total_vps_assigned: 136` matches both.

---

### WGCV-W3-014 (HIGH but CLEAN): Demo Evidence — All 37 Wave 3 MT Stories Have Evidence Directories

`docs/demo-evidence/` contains directories for all 37 Wave 3 MT story IDs (S-3.0.01 through S-3.7.05). Spot-check of 5 random stories (S-3.1.01, S-3.2.08, S-3.3.03, S-3.4.05, S-3.7.00):

| Story | Files | evidence-report.md | GIF/WebM |
|-------|-------|---------------------|----------|
| S-3.1.01 | 7 | present | 4 |
| S-3.2.08 | 10 | present | 6 |
| S-3.3.03 | 13 | present | 8 |
| S-3.4.05 | 19 | present | 12 |
| S-3.7.00 | 7 | present | 4 |

W3-FIX stories also have evidence directories: W3-FIX-CI-001, W3-FIX-LEFTHOOK-001, W3-FIX-WIN-001.

POL-010 demo evidence requirement: SATISFIED for all 37 Wave 3 MT stories.

---

## Summary

| Severity | Count | Disposition |
|----------|-------|-------------|
| CRITICAL | 1 | WGCV-W3-001 — BLOCKING: 36 story files `status: draft` → must update to `merged` |
| HIGH FAIL | 3 | WGCV-W3-002/003/004 — BLOCKING: STORY-INDEX 0 MERGED annotations; 3 W3-FIX unregistered + SS-00 invalid + status: ready; cycle-manifest not updated |
| MEDIUM | 2 | WGCV-W3-005/006 — Non-blocking: BC-INDEX version pin stale v4.17→v4.26; 22 BCs still PROPOSED |
| LOW | 2 | WGCV-W3-007/008 — Acceptable: STORY-INDEX timestamp pre-dates implementation; TenantId alias intentional |
| HIGH CLEAN | 6 | WGCV-W3-009..014 — All validated: BC traceability, dep integrity, anchor consistency, naming sweep, VP arithmetic, demo evidence |

**Consistency score (blocking criteria only):** 4 blocking items outstanding out of 8 checks = **50% gate pass rate**. Spec traceability, dependency integrity, anchor consistency, naming sweep, VP arithmetic, and demo evidence all PASS. Frontmatter freshness, index completeness, and cycle artifact currency FAIL.

---

## Path to PASS

Dispatch **W3-FIX-G** (state-manager scope — pure factory-artifacts update):

1. **36 story file status updates** (closes WGCV-W3-001):
   - Update `status: draft` → `status: merged` in all 36 affected files
   - S-3.0.01 through S-3.7.05 excluding S-3.2.03 (already merged)

2. **STORY-INDEX MERGED annotations** (closes WGCV-W3-002):
   - Add `[MERGED PR #NNN SHA DATE +Nt]` to all 37 Wave 3 MT story rows in both Epic-view tables and Full Story List
   - PR facts available in STATE.md and burst-log.md
   - Bump STORY-INDEX to v1.72 with changelog entry
   - Update STORY-INDEX BC-INDEX pin from v4.17 → v4.26 (closes WGCV-W3-005)

3. **W3-FIX story registration** (closes WGCV-W3-003):
   - Register W3-FIX-CI-001, W3-FIX-LEFTHOOK-001, W3-FIX-WIN-001 in STORY-INDEX
   - Correct SS-00 → appropriate SS in W3-FIX-CI-001 and W3-FIX-LEFTHOOK-001 frontmatter (or document as cross-cutting per a new convention)
   - Update W3-FIX-WIN-001 `status: ready` → `status: merged`

4. **cycle-manifest.md update** (closes WGCV-W3-004):
   - `status: in-progress` → `status: closed`
   - Populate Delivered metrics: 37 stories, 22 BCs, 74 VPs, HS-006/007 refreshed, 2363 tests, develop HEAD a3bd5a0f

5. **BC PROPOSED → draft** (closes WGCV-W3-006, optional but recommended):
   - Update 22 Wave 3 BC file frontmatter `status: PROPOSED` → `status: draft`
   - Update BC-INDEX Status column; bump BC-INDEX to v4.27

After W3-FIX-G commit, re-run consistency-validator for PASS verdict.
