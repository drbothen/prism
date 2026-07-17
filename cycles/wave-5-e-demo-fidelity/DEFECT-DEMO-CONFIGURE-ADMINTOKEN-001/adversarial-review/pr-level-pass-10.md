<!-- canonical pass-10 | adversary=vsdd-factory:adversary fresh-context negative-space probes | frozen HEAD 828449de | 2026-07-17 | CLEAN strict — streak 2/3 -->

# Adversarial Review — PR #225 PR-LEVEL Pass 10
## DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 (frozen HEAD 828449de)
**Finding-count summary:** CRITICAL 0 · HIGH 0 · MEDIUM 0 · LOW 0 · OBS 0 · PROCESS-GAP 0

## Verification Statement
Independently re-derived without prior-pass reports. Read at frozen HEAD: main.rs (full), multi_org_cmd.rs (full), multi_instance.rs (full), harness.rs (sidecar+accessor region), lib.rs (full), defect test suite (E2E bodies + isolation grep + Tests E/F/G/K), README (sidecar/configure sections). Canonical specs: story v0.16, BC-3.6.001, BC-2.06.017, BC-INDEX, ARCH-INDEX, error-taxonomy §DEMO. PR evidence from staged files.

## Per-Finding Entries
None at any severity.

## Attack Angles Exercised (negative-space + interplay)
Sidecar-lifecycle interplay: resolve_configure_url vs resolve_configure_token identical precedence + no-fallthrough semantics (844-980 vs 1015-1127); all four sidecar-presence permutations select same source; stale/mixed state fails cleanly, never silent auth mismatch. Key-map consistency (start mode): url_map()/token_map() derive keys identically over bound pairs (harness.rs 283-313) — flat-mode fail-loud structural; multi variant cross-checks cfg.orgs/enrichment against admin_token_map and errors loudly (706-814). Ownership: token extraction before tokio::spawn move (multi_instance.rs 388-406); zero-instance/error paths correct. Test suite: all 11 tests tempdir-isolated; binaries use current_dir(tmp); no set_current_dir global mutation; KillGuard RAII with mem::forget disarm (recycled-pid safe); Test G cfg-gated. Demo-path unblock: Tests E (flat) + G (multi) assert configure exit 0 + "200", load-bearing vs T-06/T-08/T-09 reversion; demo-run.sh reads only URL_MULTI_FILE._global — unperturbed.

## Version-Pin Lattice
All ✓: story v0.16 == STORY-INDEX; BC-3.6.001 v0.8 (file + BC-INDEX:330); BC-2.06.017 v1.12 (file + BC-INDEX:119); taxonomy v2.55; ARCH-INDEX v2.193 content confirmed (154 SS-01 lists prism-dtu-demo-server; 175 SS-22 prism-bin only). BC H1s sync across files, BC-INDEX, story tables; both BCs SS-01.

## Taxonomy §DEMO Consistency (v2.55)
Preamble (600-611) correctly scopes 001..006 construction-time + 007 sole runtime (resolve_configure_token → cmd_configure, anyhow::Result<String>); INV citations scoped. Row (621) present, template verbatim. Code emits byte-verbatim (1023-1029); cmd_configure propagates via ? (669-673). Preamble↔rows↔code fully consistent; POL-24 PASS; EC-005 sub-message matches.

## PR-Description Verification
BC pins + story version match canonical; test evidence matches orchestrator facts; diff stat matches; sweep totals internally consistent (131+7+8=146; 1+1+111+17+15+1=146); Known-Accepted matches dispatch; CI checklist reflects all-pass. PASS.

## SAP-1
PASS — only added tracing is fieldless-event debug! (main.rs 674-678, no event_type); SIGTERM/SIGINT info lines carry no event_type; PR-body "0 new event_type= emissions" corroborated.

## POL-22 (registry-text based)
Phase A PASS (ADR-003 Amendment #5, BC Precondition/Postcondition quotes, ARCH-INDEX registry text all resolve; BC titles verified against BC-INDEX + H1s). Phase C PASS (every §Architecture Mapping deliverable resolves to a real symbol at the cited module).

## CI Status
All 44 checks pass; all 5 runs success at 828449de.

## Partial-Fix Regression Discipline (S-7.01)
F-ADMTOK-PR4-HIGH-001: SS-01 consistent across frontmatter + justification + PR metadata; zero residual SS-22/"Binary Entrypoint". F-ADMTOK-PR8-MED-001: preamble/row/code aligned; sibling E-CFG-008 retired-row range updated. 0600 assertions present both writers + Tests B/F/K. No propagation gaps.

## Novelty Assessment
LOW — spec/code/taxonomy/README/tests mutually coherent; deliberately under-probed angles all resolved correctly. Production-grade; convergence confirmed.

## Dual Verdict
CLEAN (strict): yes
CLEAN (PR-merge): yes
