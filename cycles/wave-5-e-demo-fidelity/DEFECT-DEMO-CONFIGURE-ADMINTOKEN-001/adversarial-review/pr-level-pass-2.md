# PR-LEVEL Pass 2 — canonical pass-2, adversary=vsdd-factory:adversary fresh-context, evidence staged by github-ops, persisted by state-manager on behalf of the adversary, frozen HEAD 828449de, 2026-07-17

# Adversarial Review — PR #225 PR-LEVEL Pass 2
## DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 — cmd_configure missing X-Admin-Token header

**Finding-count summary:** CRIT 0 · HIGH 0 · MED 0 · LOW 0 · OBS 0 · PROCESS-GAP 0

## Verification Statement
Reviewed against frozen HEAD 828449de (PR OPEN, base develop, merge-base 84062ced), using the worktree, evidence file, PR body, story v0.15, BC-3.6.001 v0.8, BC-2.06.017 v1.12, error-taxonomy v2.54. Independently read all 8 changed files, reproduced AC-004 grep counts, verified all 11 cited test names exist, verified E-DEMO-007 byte-verbatim, ran SAP-1 across the touched crate. No prior-pass material read.

## Diff-Surface Findings
None. Implementation correct across all four ACs: AC-001/002 token resolve-before-POST with header attach (main.rs:669-692); token_map extraction before tokio::spawn move (multi_instance.rs:391/397); 0600 + atomic writes in both sidecar writers (harness.rs:361, multi_org_cmd.rs:706) contract-locked by umask-robust Tests B/F/K; _global fail-loud ok_or_else (multi_org_cmd.rs:751, Test K); AC-003 E-DEMO-007 on EC-003/004/005, no panic/silent-401; AD-017 token_present=true only; POL-24 template byte-verbatim (resolve_configure_token:1025 == error-taxonomy.md:615); no unwrap/expect in new production paths. TD-VSDD-059: Tests E/B/F/K genuinely load-bearing (binary E2E asserts exit 0 + "200"; revert of T-08 fails it).

## PR-Description Claim Verification
All 11 test names exist; AC-004 counts verified (131 same-line; 8 raw dynamic − 2 SWEEP-MIRROR = 6; 10 raw FidelityCheck − 2 mirror = 8; both 146 partitions reconcile); E-DEMO-007 registered (taxonomy line 615, changelog 623); zero event_type= in touched crate; version pins consistent; badges consistent. No stale/false/fabricated claims.

## SAP-1 Result
Zero event_type matches in crates/prism-dtu-demo-server (only touched crate); the added tracing::debug! carries clone + token_present, no event_type. No BC-2.16.002 row required. CLEAN.

## POL-22 Phase A/C Results
Phase A: BC-3.6.001 Precondition 4 verbatim (BC lines 73-74); BC-2.06.017 v1.12 Postcondition 1 governs MultiInstanceServers, admin_token_map() parallels socket_map(); E-DEMO-007 semantics correct. Phase C: resolve_configure_token, write_multi_admin_token_sidecar_to_path, write_token_sidecar_to_path, DemoHarness::token_map, MultiInstanceServers::admin_token_map, TOKEN_FILE, TOKEN_MULTI_FILE all exist (re-exported lib.rs:56-68); all 11 test fns exist; all cited files exist. Both phases CLEAN.

## CI Status
All checks pass (two workflow runs at HEAD 828449de, both success; no PENDING/FAILED) — all 5 test targets, clippy, format, non-exhaustive, perimeter gates, shellcheck, E2E smoke.

## Known-Accepted Items (not re-flagged)
3 bc_2_06_018 red-gate failures; DEMO_ORG_UUID_B clippy warning; DRIFT-HARNESS-ADMIN-TOKEN-CT-001; pre-existing .tmp sidecar perms edge; tolerated missing [[test]] Cargo.toml entry.

## Novelty Assessment
NONE. Zero findings after independent re-derivation. Production-grade: correct ordering, fail-loud propagation, credential opacity, atomic 0600 writes, load-bearing tests.

## Verdict
CLEAN (strict): yes
CLEAN (PR-merge): yes
