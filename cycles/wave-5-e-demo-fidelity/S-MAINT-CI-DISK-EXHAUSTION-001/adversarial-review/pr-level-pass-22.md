# pass-22 | adversary=vsdd-factory:adversary fresh-context | frozen HEAD d412defe | 2026-07-17 | 1 MED | streak RESET 0/3

# Adversarial Review — PR-LEVEL Pass 22 — S-MAINT-CI-DISK-EXHAUSTION-001 (PR #224)
**Finding summary:** 0 CRIT · 0 HIGH · 1 MED · 0 LOW · 0 OBS · 0 PROCESS-GAP

## Frozen-HEAD / PR-State Verification
OPEN | d412defece40... | develop matches. 5 runs success at d412defe (3 PR 29626148843/824/815 + 2 push 29626147606/595); all checks pass. Merge-base 84062ced. Diff scope: only the two workflow files.

## F-MAINT-P22-MED-001 — v0.26 changelog + EC-001 mis-attribute the job-level timeout resize to HEAD 9c315608; the resize is the d412defe delta
Severity MEDIUM, confidence HIGH. Story line 1286 (v0.26 changelog): "implementer resized job-level timeouts at HEAD 9c315608" and line 1220 (EC-001): "job-level values resized at HEAD 9c315608". Git truth: d412defe's delta IS the resize (test 45→75, tndef 25→45; parent 9c315608 still carried 45/25); 9c315608 is the OBSERVATION HEAD (run 29615360280 durations) and the SEC-001/002 HEAD (v0.24/v0.25 rows correctly anchor there). ci.yml comments themselves are CORRECT (say observed at 9c315608 — true; no false resize-commit claim). Defect isolated to the two spec citations asserting the resize ACTION occurred at 9c315608. Defeats bisection traceability (a bisector at 9c315608 finds pre-resize values). Fix: change resize-commit attribution to d412defe in EC-001 (mutable body prose, in place) and via a corrective v0.27 changelog row (v0.26 row immutable per POL-32); numeric values/durations/run-ID/2× rationale unchanged (correct). Routing: product-owner. ROOT CAUSE (orchestrator process note): PO spec-sync ran in PARALLEL with the implementer resize, so the resize SHA did not exist when the PO wrote v0.26 — sequencing lesson candidate.

## Mandatory-Axis Probe Results
(1) resize vs EC-001 vs durations: values/durations/rationales all consistent and match code; actual d412defe durations well under ceilings (worst leg windows-msvc 28m32s; tndef 10m32s); sole defect is the SHA attribution. (2) diff-vs-scope-contract CLEAN (tndef got exactly its 7 enumerated carve-outs; test matrix AC-driven + SEC-001 + EC-001/v0.26 timeout; AC-006-only jobs wrapper-only; fmt/deny/audit untouched; e2e wrapper-only). (3) version-pin lattice CLEAN (all 11 action pins byte-match story citations; fallback documented both sides; no floating refs). (4) frontmatter anchors CLEAN (empty anchors justified; 7 ACs; 10 RGTs; 22-assertion echo runtime-satisfied; STORY-INDEX row v0.26). (5) PR-description: cascade-metadata lag known-accepted; no independent substantive defect.

## verify-workflow-structure false-pass audit
22 assertions self-match-proof, runtime-computed counts, ||true guards (v0.21), section-scoped awk (v0.8), 5-min job timeout with 16-18s runtime. No false-green vector.

## AC-005 Dual-Reading (PENDING HUMAN RULING — not a finding)
Literal: 5 green runs at one commit/one trigger event. Distinct-trigger-events (F-MAINT-P10-OBS-008): 1/3 on this HEAD. Evidence incomplete under both strict readings; adjudication reserved to human.

## POL-22
Phase A: all citations resolve EXCEPT the 9c315608 resize attribution (the finding). Phase C: all job names/actions/paths resolve; no phantoms.

## SAP-1
N/A — workflow YAML only.

## Novelty Assessment
MEDIUM — genuinely new spec-vs-git-truth attribution defect specific to this HEAD's delta, surfaced by the mandatory resize-commit axis; all other axes CLEAN; near-converged cascade.

## Verdict
CLEAN (strict): no
CLEAN (PR-merge): no
1 MED resets streak to 0/3 on frozen d412defe and blocks PR-merge until the attribution is corrected (EC-001 line 1220 in place; corrective v0.27 changelog row).
