<!--
  D-1827 ACCELERATED DELTA-SCOPED RE-GATE — NOT a 3-CLEAN streak advance.
  Namespace: F-ADMTOK-PR21
  Frozen HEAD reviewed: 5c9458d6 (branch: fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001)
  Gate type: one-time human-authorized D-1827 exception — delta-scoped fresh-context pass
  BC-5.39.001 3-CLEAN streak: NOT advanced by this pass (streak counter unchanged)
  Follow-up: fix-burst @dac830d1 closed F-ADMTOK-PR21-OBS-001 same-session (D-1834)
-->
# Adversarial Review — Pass F-ADMTOK-PR21 (D-1827 Accelerated Re-Gate)

**Frozen HEAD:** `5c9458d6` · branch `fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001`
**Worktree:** `/Users/jmagady/Dev/prism/.worktrees/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001`
**Canonical-repo-root:** `/Users/jmagady/Dev/prism`
**Date:** 2026-07-18 · one-time D-1827 delta-scoped fresh-context pass (not a 3-CLEAN streak advance)

**Worktree-Identity Preflight:** PASS. Identity tuple present; worktree basename `DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001` equals story-id (case-insensitive). Spec/BC/taxonomy ground-truth read from canonical-repo-root `.factory/`; feature-code read from worktree absolute paths.

## Top-Line Counts
- CRIT: 0
- HIGH: 0
- MED: 0
- LOW: 0
- OBS: 1
- PROCESS-GAP: 0

## Findings

### F-ADMTOK-PR21-OBS-001 — Story EC-008 accepts-test cites example literals absent from the test (OBS)
**Confidence:** HIGH (grounded) · **Routing:** `vsdd-factory:product-owner` (story-text reconciliation; code is correct and comprehensive)

**File/anchor:** story `…/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-*.md` §Edge Cases EC-008 (line 338) and §Changelog v0.20 (line 443) vs `…/crates/prism-dtu-demo-server/src/main.rs` `test_validate_clone_name_accepts_valid` (lines 826-835+).

**Description:** EC-008 and the v0.20 changelog both state the accepts-test inputs are "bare sensor names `crowdstrike` / `armis`, org-slug composite `acme-crowdstrike`, names with underscore `my_sensor` and hyphen `my-sensor`." The as-built test uses `crowdstrike`, `cyberint`, `armis`, `org-a-crowdstrike`, `org-b-cyberint`, `sensor_name-v2`, `Sensor42`. Two cited literals (`crowdstrike`, `armis`) are present; three (`acme-crowdstrike`, `my_sensor`, `my-sensor`) are not. All claimed coverage *categories* — bare, org-slug composite, underscore, hyphen — are still exercised (plus a bonus mixed-case case `Sensor42`), and the load-bearing charset contract is byte-identical. This is a descriptive spec-text-vs-code drift in illustrative example literals; it does not affect behavior, the charset contract, or category coverage, and does not mislead an implementer about what to build. Non-blocking for PR-merge. Cheapest correct fix: align the story's parenthetical example list to the actual test literals (or vice-versa).

## Axis Results

**Axis 1 — Version-pin lattice: PASS.** Story frontmatter v0.20 (line 9) == STORY-INDEX row `draft v0.20` (line 828) == PR body pins v0.20 (lines 5, 332). BC-3.6.001 v0.8 (BC file line 6, story table, PR body). BC-2.06.017 v1.12 (BC file line 5, story table, PR body). Error-taxonomy v2.55 (file line 6). All aligned.

**Axis 2 — Anchors vs registry (POL-4/6/7): PASS.** Story `subsystems: [SS-01]`; both BCs `subsystem: SS-01`; justification cites ARCH-INDEX v2.193 SS-01 (Sensor Adapters) owning `prism-dtu-demo-server`. BC H1 titles match exactly: BC-3.6.001 "Per-Org Failure Injection" (file line 35), BC-2.06.017 "Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing" (file line 40) — consistent with story Behavioral Contracts table and PR body traceability. F-ADMTOK-PR4-HIGH-001 (SS-22→SS-01 re-anchor) confirmed propagated to frontmatter, justification comment, and STORY-INDEX.

**Axis 3 — Error-taxonomy consistency: PASS.** E-DEMO-007 registered (line 621), template byte-verbatim per POL-24 vs story AC-003 row and EC-008. Preamble carve-out present (lines 600-611): E-DEMO-001..006 construction-time; E-DEMO-007 sole runtime error. EC-008 validation error correctly NOT an E-DEMO code and intentionally excluded from taxonomy (argument-validation gate, not runtime state-resolution) — consistent between story EC-008, changelog v0.20, and code (plain `anyhow::bail!`). Changelog monotonic-descending 2.55→2.50 (POL-32), row cell counts match header (POL-26).

**Axis 4 — Mirror-table byte-identity: PASS.** Story §Root Cause sweep footnote (447/131/6/8, 146 total), main.rs SWEEP-MIRROR block (lines 664-674, 447/131/6/8, 146), and PR body §TD-VSDD-060 (447/131/6/8, 146) are consistent. Per-class tally (111 correct-token + 17 no-token + 15 wrong-token + 1 prod-CLI + 1 harness + 1 synthetic = 146) identical across story and PR body.

**Axis 5 — Story tables vs as-built code (POL-22 Phase C): PASS.** All cited symbols resolve at cited files: `validate_clone_name`/`sanitize_clone_name` (main.rs 599-632), ordering (`validate_clone_name` line 642 BEFORE `resolve_configure_url` line 693), `resolve_configure_token` (line 715), `X-Admin-Token` attach (line 736), `TOKEN_FILE`/`TOKEN_MULTI_FILE` usage (717-718). File-structure and architecture-mapping tables match.

**Axis 6 — EC end-to-end incl. NEW EC-008: PASS (with OBS-001).** EC-001..EC-007 claimed behaviors consistent with code paths (URL-resolution-first ordering caveats for EC-003/004/005 match `resolve_configure_url` before `resolve_configure_token`). EC-008 claimed behavior — validate FIRST, reject chars outside `[a-zA-Z0-9_-]`, sanitize disallowed→`?`, error template, exit 1 — matches code exactly. Only the accepts-test example-literal enumeration drifts (OBS-001).

## Delta-Focus Results (`828449de`→`5c9458d6`, +136/−0, 1 file)

- **validate_clone_name runs FIRST:** CONFIRMED — line 642, before `resolve_configure_url` (693), all sidecar I/O, and the `tracing::debug!` at 720. CWE-117 vector closed: `clone_name` only reaches tracing after validation.
- **sanitize_clone_name → `?`:** CONFIRMED — non-`[a-zA-Z0-9_-]` chars mapped to `'?'` (599-609); error echoes sanitized form only.
- **Charset claim exact match:** CONFIRMED — `is_ascii_alphanumeric() || c == '-' || c == '_'` == `[a-zA-Z0-9_-]`. Error template byte-identical to EC-008 / v0.20 changelog (POL-24).
- **2 contract tests load-bearing (TD-VSDD-059):** CONFIRMED — both call `validate_clone_name` directly with real `expect_err`/`expect` assertions on error text (`invalid clone name`, `?`, `alphanumerics`) and accept-paths; not paper tests. Rejects test covers all 5 EC-008 categories (newline/null/ANSI/slash/space) verbatim.
- **Net-additive, no regression:** CONFIRMED — diff is +136/−0; token resolution order, header attach (736), AD-017 `token_present=true` placeholder (720-724), and 10s timeout (with SEC-002 rationale, 726-729) unchanged.

## SAP-1 (tracing catalog completeness): PASS
`event_type\s*=` across `…/crates/prism-dtu-demo-server/` → zero matches. Delta adds no tracing emissions of any kind; the pre-existing `tracing::debug!` (720) carries no `event_type` field. No BC-2.16.002 catalog row required.

## POL-22 Phase A / Phase C: PASS
Phase A (lexical-vs-semantic): BC-3.6.001 Precondition-4 anchor (configure requires admin_token auth) and BC-2.06.017 Postcondition-1 anchor (MultiInstanceServers/admin_token_map) are semantically correct for the story's scope. Phase C (named-entity existence): all referenced symbols and files exist in the worktree checkout.

## AD-017 credential redaction: PASS
No new logging in delta. Existing debug logs validated `clone_name` + `token_present=true`; no token value transits logs.

## CI Status: PASS
45/45 checks green on HEAD `5c9458d6` (runs 29630596434, 29630597622, 29630597682). Known-accepted items (bc_2_06_018_seeding Red Gate ×3; DEMO_ORG_UUID_B dead_code; DRIFT-HARNESS-ADMIN-TOKEN-CT-001 CWE-208 out-of-scope) respected — not re-flagged.

## Novelty Assessment
Novelty: LOW. The delta is a tight, well-contained security gate; the only new observation is a trivial example-literal mismatch in story prose. No structural, security, or contract gaps found. The SEC-001/SEC-002 delta is production-grade and fully contract-locked.

## Dual Verdict
```
CLEAN (strict): no      (1 OBS — F-ADMTOK-PR21-OBS-001)
CLEAN (PR-merge): yes   (zero CRIT/HIGH/MED findings)
```

The D-1827 accelerated re-gate merge gate (CLEAN PR-merge) is satisfied on frozen HEAD `5c9458d6`. The single OBS is non-blocking; recommend product-owner reconcile the EC-008/v0.20 accepts-test example literals to the as-built test inputs in a follow-up (or during the state-manager post-merge burst).
