# PR-LEVEL Pass 11 — canonical pass-11, adversary=vsdd-factory:adversary fresh-context error-path-matrix + POL-rubric-on-artifact probes, frozen HEAD 828449de, 2026-07-17, 1 LOW — streak RESET 0/3; CLEAN PR-merge yes

# Adversarial Review — PR #225 (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001) — PR-LEVEL Pass 11
**Finding-count summary:** CRITICAL 0 · HIGH 0 · MEDIUM 0 · LOW 1 · OBS 0 · PROCESS-GAP 0

## Verification statement
Fresh-context at frozen HEAD 828449de; no prior pass reports read. Re-derived from artifacts directly: story v0.16, PR body, evidence file, full worktree source of touched files (main.rs cmd_configure/write_token_sidecar/shutdown, multi_instance.rs start_instances/MultiInstanceServers, multi_org_cmd.rs resolvers/writers, harness.rs, lib.rs, td_wv1_04 KillGuard, Cargo.toml, README), BC-3.6.001 v0.8, BC-2.06.017 v1.12, BC-INDEX, STORY-INDEX, error-taxonomy v2.55 §DEMO. Known-accepted held out.

## F-ADMTOK-PR11-LOW-001 — Story §Error Taxonomy Addition table uses non-canonical column headers
Severity LOW. Story line 266 (AC-003 addition table) headed | Code | State | Domain | Message template | Retryable | Description | vs canonical DEMO section header (taxonomy:613) | Code | Severity | Category | Message Format | Retryable | Description |. `broken` sits under Severity in the taxonomy (vocabulary broken/degraded/cosmetic), `configuration` under Category — story mislabels as State/Domain. Load-bearing content byte-correct and correctly registered; only the story mirror-table labels diverge, semantically mis-describing `broken` as a lifecycle state. Routing: product-owner (align headers to Severity | Category | Message Format). Blast radius 1 file; no sibling table carries the mislabel.

## Error-path matrix (EC × resolver branch) — resolve_configure_token
All 11 branches reachable and correctly mapped (flat read-fail/parse-fail/hit/miss EC-003 Test H; nested read-fail/parse-fail/exact-hit/bare-0 EC-003 Test I/bare-1/bare->1 EC-005 Test D; neither EC-004 Test C). CLI-reachability honesty verified (URL resolved first; skewed-sidecar arms documented by EC-005 + isolation tests). _global cannot collide with per-org bare names (KNOWN_SENSORS ∩ KNOWN_ENRICHMENT_CLONES = ∅). No finding.

## Sibling-story / shared-surface consistency
Configure surface consistent; token resolver mirrors URL resolver; shutdown removes both token sidecars alongside URL sidecars (main.rs:387,541). Nothing contradicts the fix. No finding.

## Version-pin lattice
All ✓ (BC-2.06.017 v1.12 across 5 sites; BC-3.6.001 v0.8 across 3; story v0.16; taxonomy v2.55; known-accepted #8 respected).

## PR-description verification
Diff stat ✓; test table consistent with Cargo autodiscovery (known-accepted #5) ✓; sweep arithmetic reproduced ✓; README claims match code ✓; KillGuard disarm-after-wait verified (151-158, 246-249) — no PID-reuse hazard ✓.

## SAP-1
PASS — 0 event_type matches; new debug! fieldless + AD-017-safe.

## POL-22 A/C (registry-text based)
A PASS (BC-2.06.017, BC-3.6.001, CAP-036, ADR-003 Amendment #5, E-DEMO-007 resolve; BC-INDEX↔H1 titles match; both SS-01). C PASS (every §Architecture Mapping / §File Structure anchor resolves to real symbols at cited modules).

## Additional axes (no findings)
POL-8 bidirectional BC coherence ✓; POL-16 Test A documented contract-lock not inverted ✓; Cargo hygiene reqwest rustls-tls + default-features=false in deps AND dev-deps (ADR-050), 10s timeout ratified crate-local, libc cfg(unix) ✓; ownership/async-move ✓; fail-loud _global ✓; changelogs monotonic ✓.

## CI status
All 44 checks pass; all 5 runs success at 828449de.

## Novelty assessment
LOW — 21 LOCAL + 10 PR passes have driven code to production-grade; the single new finding is a header-schema comparison prior passes never performed. Refinement, not a gap; substantively converged.

## Dual verdict
CLEAN (strict): no
CLEAN (PR-merge): yes
