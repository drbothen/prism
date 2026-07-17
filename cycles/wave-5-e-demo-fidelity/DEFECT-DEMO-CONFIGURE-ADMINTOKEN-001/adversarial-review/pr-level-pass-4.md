<!-- canonical pass-4, adversary=vsdd-factory:adversary fresh-context heightened-scrutiny, frozen HEAD 828449de, 2026-07-17, 1 HIGH — streak RESET 0/3 -->

# Adversarial Review — PR #225 PR-LEVEL Pass 4
## DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 (frozen HEAD 828449de)

**Top line: 1 HIGH, 0 CRIT, 0 MED, 0 LOW, 0 OBS, 0 PROCESS-GAP**

## Verification statement
Independently re-derived from frozen HEAD 828449de: all 8 changed source files, 1611-line test suite, canonical story spec, BC-3.6.001, E-DEMO-007 row, .gitignore, ARCH-INDEX. Probed: TOCTOU/symlink on .tmp path, token leakage via logs/Debug/errors, HashMap nondeterminism, malformed/truncated sidecar JSON, concurrent binary-test interference, _global collisions, EC-005 parity, feature-flag matrix gaps — all clean. One HIGH from the Semantic Anchoring Audit.

## F-ADMTOK-PR4-HIGH-001 — Story subsystems: [SS-22] anchor uses fabricated subsystem name and misattributes crate ownership vs ARCH-INDEX
- Severity HIGH (confidence HIGH). Policies: POL-6 + POL-22 + POL-4.
- Anchors: ARCH-INDEX.md:175 (SS-22 | Process Lifecycle | prism-bin | scope: boot orchestration contract only — ADR-022 §B 11-step boot, exit-code map, traffic-gate signal handlers); story frontmatter lines 32-38 (subsystems: [SS-22] justified as "SS-22 (Binary Entrypoint) owns the prism-dtu-demo-server CLI subcommand surface ... Per ARCH-INDEX Subsystem Registry SS-22").
- Defects: (1) fabricated name — registry says "Process Lifecycle", never aliased "Binary Entrypoint"; citation falsified (POL-22). (2) ownership misattribution — ARCH-INDEX scopes SS-22 to prism-bin boot orchestration only, delegating per-subsystem init to SS-06/SS-21/SS-03/SS-05; prism-dtu-demo-server is separate DTU/demo infra not implementing ADR-022 boot (POL-6/POL-4).
- Why prior passes missed it: 21 LOCAL + prior PR passes concentrated on code/test load-bearing; none re-audited the subsystems: citation against the registry.
- Fix: correct name to "Process Lifecycle" AND reconcile scope — (a) re-anchor to the subsystem that actually owns prism-dtu-demo-server demo/DTU infrastructure, or (b) if SS-22 is deliberately least-wrong, replace the false justification with an accurate rationale not contradicting the registry's prism-bin-only scope.
- Routing: story-writer via orchestrator. Spec-side fix on factory-artifacts; not a code change. Blocks convergence per Semantic Anchoring Audit.

## Observations
None as findings. (.tmp create-truncate predictable-filename shape adjudicated within KNOWN-ACCEPTED #4's pre-existing edge: demo/test infra, ephemeral UUID-v4 non-credentials, byte-identical to long-standing URL-sidecar pattern.)

## PR-description verification
Diff stat matches (10 files, +2224/−22). All 11 test names exist verbatim. 10/10 vs 11/11 fixture-gen correct (Test G cfg-gated @1195). Sibling-sweep arithmetic internally consistent (131+7+8=146; 103+8=111). AD-017 clean (token_present=true only; fail-loud key lists are instance names). Atomic tmp+rename + 0600 both writers (rename preserves 0600). E-DEMO-007 verbatim (taxonomy:615 == multi_org_cmd.rs:1023-1029, POL-24). Malformed/truncated JSON → structured E-DEMO-007, never panic (1040-1041, 1062-1065). Determinism: all org lists sorted before {:?} (916, 1093; key sorts 725/753; locked by Tests J/F/K). Binary E2E isolation via tempdir + :0 ports (Test E 738/749, Test G 1202/1210).

## SAP-1 result
PASS — no event_type matches in touched crate; new tracing::debug! (main.rs:674) carries no event_type; no BC-2.16.002 row required.

## POL-22 Phase A / Phase C
Phase A: E-DEMO-007 + BC-3.6.001 Precondition 4 quotes accurate; EXCEPTION — ARCH-INDEX SS-22 citation falsified (F-ADMTOK-PR4-HIGH-001). Phase C: all code symbols resolve (lib.rs:43/49, re-exports 56/64-68).

## CI status
All 42 check rows pass; run-list all success at 828449de. Note only.

## Novelty assessment
MEDIUM-HIGH for the single finding. Code/test surface genuinely converged (deep probes clean, production-grade). The HIGH is NEW, non-retreaded: spec-artifact subsystem mis-anchor that survived 21 LOCAL + prior PR passes — exactly the fresh-context compounding value the audit expects.

## Dual Verdict
CLEAN (strict): no
CLEAN (PR-merge): no
