---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-07-17T00:00:00Z
phase: 5
inputs:
  - .factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md
  - .factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md
  - crates/prism-dtu-demo-server/src/multi_org_cmd.rs
input-hash: "53908fb"
traces_to: .factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md
pass: 12
previous_review: local-pass-11.md
scope: LOCAL
fix_pr: DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001
feature_head_at_review: 0feaf281
fix_burst_head: 4feac52b
date: 2026-07-17
clean_strict: false
clean_pr_merge: false
finding_counts:
  MED: 1
  LOW: 3
  OBS: 1
  total: 5
streak_after: 0/3
convergence: IN_PROGRESS
adversary_routing_note: "TD-VSDD-005 RESOLVED rc.22 — live-evidence routing uses general-purpose-as-adversary (shell access required for grep/nextest verification)"
---

# Adversarial Review: DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 (Pass 12)

## Finding ID Convention

Finding IDs use the format: `F-ADMTOK-P<PASS>-<SEV>-<SEQ>`

- `F-ADMTOK`: Fixed prefix for DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 findings
- `P<PASS>`: Two-digit pass number (e.g., `P12`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Examples: `F-ADMTOK-P12-MED-001`, `F-ADMTOK-P12-LOW-001`

## Positive Verifications

All four sweep commands reproduce expected counts: 447 prism-mcp tests / 131 prism-dtu-harness tests / 6 DEFECT-ADMINTOKEN story RGTs (story §Red Gate Test Plan) / 8 Test A–J test stubs; grand total 146 = 111 (cmd_configure path) + 17 (EC-005 resolver) + 15 (start_multi path) + 1 (Test J new) + 1 (Test I) + 1 (Test H). FidelityCheck 8-site paper-fix probe PASS (Test A payload aligned to AC literal; Test C+D format-lock; Test E+F binary-path E2E; Test G start-multi; Test H+I shutdown; Test J new resolver-ordering). `.gitignore` 10 sidecar names all return `check-ignore` positive. nextest 58 pass + 3 known bc_2_06_018_seeding Red Gate (pre-existing on develop; not story regressions). SAP-1: zero new `event_type =` emissions confirmed by rg sweep. POL-22/24/29/32/34 + POL-13 all PASS on v0.12 story content. AD-017: no credential values in story or BC artifacts.

## Part A — Fix Verification (pass >= 2 only)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-ADMTOK-P11-MED-001 | MED | RESOLVED | FidelityValidator wiring confirmed present @0feaf281; 8-site paper-fix probe PASS |
| F-ADMTOK-P11-LOW-001 | LOW | RESOLVED | Command forms table present in story v0.12; individual form accuracy examined in pass-12 (see F-ADMTOK-P12-LOW-001 below) |
| F-ADMTOK-P11-LOW-002 | LOW | RESOLVED | Phantom mirror reference removed from story v0.12 |
| F-ADMTOK-P11-OBS-001 | OBS | RESOLVED | pid.tmp gitignore pattern added and confirmed check-ignore positive |

## Part B — New Findings (or all findings for pass 1)

### MEDIUM

#### F-ADMTOK-P12-MED-001: Phantom §Sidecar-availability anchor in story BC-table cell + BC-2.06.017 v1.11 Postcondition 1; GAP-3 semantically mis-cited

- **Severity:** MED
- **Category:** spec-fidelity / contradictions
- **Policy:** POL-21 (cite-anchors must resolve to real headings); POL-4 (spec accuracy)
- **Location:** `.factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md` §Red Gate Test Plan BC-table cell; `.factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md` v1.11 Postcondition 1
- **Description:** Fix-burst-1 (fb-1) introduced a phantom section anchor `§Sidecar-availability` into two places. (1) The story's §Red Gate Test Plan BC-table cell for BC-2.06.017 cites `§Sidecar-availability` as the relevant section anchor. No such section exists in BC-2.06.017 v1.11 — the real relevant section is `§Postconditions` (specifically Postcondition 7, token_sidecar_path). (2) BC-2.06.017 v1.11 Postcondition 1 contains an inline `§Sidecar-availability` cross-reference citation that is equally phantom. Additionally, GAP-3 is semantically mis-cited: the launcher story (S-DEMO-LAUNCHER-CONSOLIDATION-001) defines GAP-3 as the DEMO_RUN_DIR cwd-threading note, not an atomic-write guarantee. Story v0.12 uses GAP-3 in a context implying it guarantees atomic-write semantics for the token sidecar file — that is incorrect.
- **Evidence:** `grep -n "Sidecar-availability" .factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md` returns hits; `grep -n "Sidecar-availability" .factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md` returns a hit in Postcondition 1. No `## Sidecar-availability` or `### Sidecar-availability` heading exists in either file. `grep -n "GAP-3" .factory/stories/S-DEMO-LAUNCHER-CONSOLIDATION-001*.md` confirms GAP-3 = DEMO_RUN_DIR cwd-threading note.
- **Proposed Fix:** Strip phantom `§Sidecar-availability` anchor from story BC-table cell; replace with `§Postconditions/Postcondition-7` citation. Correct GAP-3 citation in same context to reflect cwd-threading semantics. Update BC-2.06.017 Postcondition 1 to remove the phantom anchor citation; substance unchanged.

### HIGH

_(No HIGH findings this pass.)_

### LOW

#### F-ADMTOK-P12-LOW-001: Story "command forms are IDENTICAL" claim false; 3 of 4 forms differ from code mirrors in -n / wc -l

- **Severity:** LOW
- **Category:** spec-fidelity
- **Policy:** POL-4 (spec accuracy); POL-22 (cite-anchors must reproduce live behavior)
- **Location:** `.factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md` §Acceptance Criteria command-forms table
- **Description:** Story v0.12 §Acceptance Criteria contains a table asserting the four canonical command invocation forms for `prism configure` are "IDENTICAL" between the story spec and the live code mirrors in `multi_org_cmd.rs`. Verification of the 4 forms against the code shows 3 of 4 differ: Form 1 uses `-n org-a` (story) vs `--name org-a` (code); Form 2 uses `wc -l` (story) vs `| wc -w` (code comment); Form 3 shows a 2-org invocation but the code mirror reflects the 3-org variant introduced in fb-7. The blanket "IDENTICAL" claim is false for 3 of 4 entries.
- **Evidence:** `grep -n "\-n org\|wc -l\|wc -w" crates/prism-dtu-demo-server/src/multi_org_cmd.rs` shows the discrepancies. The fb-7 commit message confirms the 3-org variant was introduced after story v0.8 command-forms table was authored.
- **Proposed Fix:** Replace "IDENTICAL" blanket claim with per-form verified annotations; correct Forms 1–3 to match code mirrors byte-for-byte; annotate Form 4 as verified-correct unchanged.

#### F-ADMTOK-P12-LOW-002: POL-22 Phase C TD-VSDD-060 sweep table names nonexistent `deny_unknown_fields` helper; real helper = `assert_configure_strict`

- **Severity:** LOW
- **Category:** spec-fidelity / coverage-gap
- **Policy:** POL-22 (cited test function names must exist); TD-VSDD-060 (sibling-sweep)
- **Location:** `.factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md` §Verification Phase C TD-VSDD-060 sweep table
- **Description:** Story v0.12 §Verification Phase C lists a TD-VSDD-060 sibling sweep table including a row citing `deny_unknown_fields` as the helper function to be verified across all call sites. No function named `deny_unknown_fields` exists in `prism-dtu-demo-server` (confirmed by rg). The real helper enforcing strict deserialization is `assert_configure_strict`, introduced in fb-3 and renamed from an earlier form. The Phase C table was not updated after the rename.
- **Evidence:** `rg "deny_unknown_fields" crates/prism-dtu-demo-server/src/` returns zero hits. `rg "assert_configure_strict" crates/prism-dtu-demo-server/src/` returns the real function definition and 3 call sites.
- **Proposed Fix:** Story v0.13 §Verification Phase C: `deny_unknown_fields` → `assert_configure_strict` in TD-VSDD-060 sweep table; update rg verification line; correct the 3 affected cells.

#### F-ADMTOK-P12-LOW-003: EC-005 E-DEMO-007 ambiguity error unreachable via CLI in canonical URL-ambiguity scenario; resolve_configure_url bails with plain anyhow error; token-resolver arm is defense-in-depth locked by Test D

- **Severity:** LOW
- **Category:** spec-fidelity / missing-edge-cases
- **Policy:** POL-4 (spec accuracy)
- **Location:** `.factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md` §Edge Cases EC-005
- **Description:** Story v0.12 §Edge Cases EC-005 describes that when multiple org-slug candidates match the configured URL, `E-DEMO-007` (AmbiguousOrgSlug) fires. The prose implies this error fires from `resolve_configure_url`. However, `resolve_configure_url` returns a plain `anyhow::Error` (not `E-DEMO-007`) when multiple candidates match — it bails early with a formatted message before reaching the `E-DEMO-007` emission arm. `E-DEMO-007` is only reachable via `resolve_configure_token` (the fallback when `resolve_configure_url` returns `Ok(None)`). In the canonical CLI scenario where a URL IS found but is ambiguous, `resolve_configure_url` bails before `resolve_configure_token` is invoked. EC-005 conflates the two resolver paths.
- **Evidence:** Code inspection of `resolve_configure_url` in `crates/prism-dtu-demo-server/src/multi_org_cmd.rs` shows `anyhow::bail!` on multi-match, not the `DemoError::AmbiguousOrgSlug` variant that produces E-DEMO-007. Test D exercises `resolve_configure_token` only (confirmed by function name in test body).
- **Proposed Fix:** Story v0.13 §Edge Cases EC-005: rewrite resolution-order prose to accurately distinguish URL-resolver path (returns plain anyhow error on multi-match) from token-resolver path (emits E-DEMO-007 on multi-match); update Test D note to cite the correct code path.

### OBSERVATIONS

#### F-ADMTOK-P12-OBS-001: Sibling determinism gap — resolve_configure_url lacked bare_matches sort present in resolve_configure_token

- **Severity:** OBS
- **Category:** code-quality / missing-edge-cases
- **Policy:** TD-VSDD-060 (sibling-site sweep discipline)
- **Location:** `crates/prism-dtu-demo-server/src/multi_org_cmd.rs` `resolve_configure_url` function (frozen 0feaf281)
- **Description:** `resolve_configure_token` sorts its `bare_matches` vector before returning an ambiguity error, ensuring deterministic error messages regardless of HashMap iteration order. `resolve_configure_url` (the sibling function) assembles `bare_matches` from HashMap iteration without sorting. Error messages from the URL-path ambiguity case are therefore non-deterministic across runs. While no existing test asserts URL-path error message content, the sibling-determinism gap is an OBS-level consistency concern.
- **Evidence:** `grep -n "bare_matches" crates/prism-dtu-demo-server/src/multi_org_cmd.rs` shows `.sort()` call present in `resolve_configure_token` block and absent in `resolve_configure_url` block.
- **Proposed Fix:** Add `bare_matches.sort()` before the ambiguity error construction in `resolve_configure_url`, matching the `resolve_configure_token` sibling pattern. Add a load-bearing test verifying deterministic sort in the URL-resolver path. Apply TD-VSDD-060 sweep to any other diagnostic-list construction sites lacking sorts.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 1 |
| LOW | 3 |
| OBS | 1 |

**Overall Assessment:** block
**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO
**Convergence:** FINDINGS_REMAIN — iterate (streak reset 0/3; fb-11 dispatched)
**Readiness:** requires revision — all 5 findings closed by FIX-BURST-11 (story v0.13 + BC-2.06.017 v1.12 + code @4feac52b); NEXT = LOCAL pass-13 on frozen 4feac52b

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 12 |
| **New findings** | 5 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 5 / (5 + 0) = 1.00 |
| **Median severity** | LOW (3 LOW + 1 MED + 1 OBS) |
| **Trajectory** | →4→3→3→4→5 (passes 8→9→10→11→12) |
| **Verdict** | FINDINGS_REMAIN |
