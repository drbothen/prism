# pr-level-pass-9 | adversary=vsdd-factory:adversary fresh-context | frozen HEAD 828449de | 2026-07-17 | CLEAN strict — streak 1/3

# Adversarial Review — PR #225 PR-LEVEL Pass 9
**Story:** DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 | **Frozen HEAD:** 828449de | **Base:** develop (merge-base 84062ced)

## Finding-count summary
**CRITICAL: 0 | HIGH: 0 | MEDIUM: 0 | LOW: 0 | OBS: 0 | PROCESS-GAP: 0** — zero findings of any severity.

## Verification statement
Re-derived independently at frozen HEAD without prior-pass conclusions: story v0.16 (full), PR body, CI/run evidence, error-taxonomy §DEMO (preamble + all 7 rows + changelog), BC-3.6.001 + BC-2.06.017 files + BC-INDEX rows, STORY-INDEX changelog, ARCH-INDEX Subsystem Registry (SS-01/SS-22), ADR-003 Amendment #5, as-built code (cmd_configure, shutdown handlers, resolve_configure_token, write_multi_admin_token_sidecar_to_path, lib.rs constants/re-exports). Every corroborable claim corroborated.

## Taxonomy §DEMO internal-consistency (mandatory axis 3)
Preamble vs rows PASS: v2.55 correctly scopes E-DEMO-001..006 construction-time (via build_clone_pairs) and carves out E-DEMO-007 as sole runtime error (resolve_configure_token → cmd_configure, anyhow::Result<String>); each 001..006 row confirms construction-time detection; 007 row confirms runtime emission; no row contradicts. INV citations PASS (both invariants scoped construction-time, explicitly excluding 007, consistent with v2.55 changelog). Rows vs emitting code PASS: E-DEMO-007 Message Format byte-identical to e_demo_007 closure (multi_org_cmd.rs:1023-1029); EC-005 ambiguity message passed as {reason} into the canonical template — no divergence; sorted {:?} renders ["org-a", "org-b"] matching story. Severity/category consistent (all broken/configuration; 007 as configuration defensible — root cause is absent sidecar config state; matches story AC-003 row).

## Version-pin lattice (mandatory axis 1)
All CONSISTENT: story v0.16 (fm ln 9) == STORY-INDEX (D-1816 changelog v2.703) == PR body (ln 310); BC-3.6.001 v0.8 + BC-2.06.017 v1.12 == files == BC-INDEX; taxonomy referenced as path not pin (known-accepted #8; canonical now v2.55 — 007 registered @v2.54, carve-out @v2.55); ARCH-INDEX v2.193 ln 154/175 citations current. Pass-7 pin lag confirmed closed.

## Anchors vs registry text (mandatory axis 2 / POL-4, POL-6)
subsystems: [SS-01] VERIFIED (ARCH-INDEX v2.193:154 lists prism-dtu-demo-server in SS-01 crate column; :175 SS-22 Process Lifecycle prism-bin-only, correctly NOT chosen). BC H1 titles verified across files, BC-INDEX, story tables; both BCs subsystem SS-01. No mis-anchoring.

## Story frontmatter-body coherence (POL-8)
bcs: [BC-3.6.001, BC-2.06.017] bidirectional with body table + AC traces (AC-001/002/003→BC-3.6.001; AC-004→BC-2.06.017). PASS.

## PR-description verification
Diff stat matches; 11 tests match; sweep arithmetic correct (111+17+15+1+1+1=146; 131+7+8=146); BC pins match; known-accepted 1–3 quarantined; convergence table consistent with frozen-HEAD rule; sidecar constants match lib.rs:43,49 verbatim.

## As-built code corroboration (POL-22 Phase C / TD-VSDD-059)
cmd_configure resolves token BEFORE POST, attaches header, token_present=true only (AD-017), no event_type. Shutdown cleanup both handlers (main.rs:387,541; T-09). Fail-loud _global ok_or_else (751-763; Test K). Atomic 0600 write (783-801). ADR-003 Amendment #5 §Decision (628-632) + §Implementation item 4 (664-666) match story block-quotes verbatim (POL-22 Phase A).

## SAP-1
PASS — zero event_type matches across demo-server src; diff adds only a fieldless tracing::debug!; no catalog row required. Other changed files test-only.

## POL-22 A/C
Phase A PASS (verbatim ADR quotes; registry-text citations verified). Phase C PASS (all symbols resolve at cited worktree paths).

## CI status
All 44 checks pass; all 5 runs success at 828449de; matches PR headRefOid.

## Novelty assessment
LOW — converged. Fresh angles (taxonomy preamble-vs-emission consistency, lattice cross-product, registry-text anchors, ADR verbatim quotes, shutdown/fail-loud corroboration) found no gaps. Load-bearing claims backed by tests; structural fix, not paper-fix.

## Dual verdict
CLEAN (strict): yes
CLEAN (PR-merge): yes

## Routing
None — no findings.
