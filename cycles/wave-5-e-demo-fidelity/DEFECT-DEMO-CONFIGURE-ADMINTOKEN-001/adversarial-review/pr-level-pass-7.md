# Adversarial Review — PR #225 PR-LEVEL Pass 7 (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001)
<!-- adversary=vsdd-factory:adversary fresh-context version-pin-lattice probe; frozen HEAD 828449de; 2026-07-17; 1 MED; streak RESET 0/3 -->

**Finding count: 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW, 0 OBS, 0 PROCESS-GAP**

## Verification Statement
Reviewed at frozen HEAD 828449de against canonical .factory/ and the worktree. Independently re-derived: cmd_configure (main.rs:597-708), resolve_configure_token (multi_org_cmd.rs:1015-1127), write_multi_admin_token_sidecar_to_path (706-814), write_token_sidecar_to_path + token_map (harness.rs:302-397), admin_token_map + pre-move extraction (multi_instance.rs:205-414), pub surface (lib.rs:37-68). No adversarial-review/ artifacts read.

## F-ADMTOK-PR7-MED-001 — STORY-INDEX version-pin lags story frontmatter (v0.16 not propagated)
- Severity MEDIUM (confidence HIGH). Policy: POL-13 + version-pin-lattice probe.
- Story frontmatter version: "0.16" (line 9) + §Changelog v0.16 row (line 441, SS-22→SS-01 re-anchor); STORY-INDEX.md line 828 pins "draft v0.15" (828449de D-1804); latest index changelog v2.701 = v0.15. No v0.16 entry anywhere in STORY-INDEX.
- The v0.16 change itself is CORRECT (ARCH-INDEX:154 SS-01 crate column lists prism-dtu-demo-server; :175 SS-22 prism-bin-only). Defect is purely the un-propagated registry pin, which survived two prior clean passes.
- Routing: state-manager — add v0.16 STORY-INDEX row; confirm STATE.md/SESSION-HANDOFF carry no same lag.

## Defect-Class Sweep Results (all PASS)
Only cmd_configure issues a client POST to an authenticated endpoint (Commands enum: Start/Stop/StartMulti/Configure); no sibling client-POST unfixed. /dtu/reset is admin-token-protected but has NO HTTP client caller in the demo server (only in-process pair.clone.reset() at harness.rs:264) — sweep correctly scoped. Cross-crate surface all additive; Semver check passed. Token/URL source-consistency: resolve_configure_token mirrors resolve_configure_url precedence exactly; fail-loud E-DEMO-007; no silent 401 swallow.

## PR-Description Verification
Diff-stat, 10-file set, 146-POST tally consistent (SWEEP-MIRROR main.rs:598-636 matches story §Root Cause). Test A-K table matches; not a paper-fix (TD-VSDD-059 PASS — header genuinely attached main.rs:688). BC pins match. PR body story-version 0.16 is what surfaced the MED.

## SAP-1 Result
Zero event_type matches; new tracing::debug! carries clone/token_present only. PASS.

## POL-22 Results (registry-text based)
Phase A: BC-2.06.017 H1 (file line 40) and BC-3.6.001 H1 (5,35) match citations; both SS-01; ARCH-INDEX:154 confirms; E-DEMO-007 at taxonomy:615. PASS. Phase C: template byte-identical across taxonomy:615 / code (1023-1029) / story AC-003 (253). POL-24 PASS.

## Version-Pin Lattice Audit
Story frontmatter v0.16 ✓ | PR body v0.16 ✓ | STORY-INDEX v0.15 ✗ (→ MED-001) | BC-2.06.017 v1.12 ✓ | BC-3.6.001 v0.8 ✓ | taxonomy v2.54 ✓ | ARCH-INDEX SS-01:154 ✓

## CI Status
All 44 checks pass; all 5 runs success at 828449de. Known-accepted not re-flagged.

## Novelty Assessment
MEDIUM — the finding is genuinely new (registry-propagation gap visible only at the version-pin-lattice axis, which code-anchored prior passes systematically miss). All code-level axes clean and converged; implementation production-grade.

## Dual Verdict
CLEAN (strict): no
CLEAN (PR-merge): no
Rationale: MED resets streak to 0/3 and fails the PR-merge gate. Route to state-manager (v0.16 index row); re-gate after. Code fix otherwise merge-ready.
