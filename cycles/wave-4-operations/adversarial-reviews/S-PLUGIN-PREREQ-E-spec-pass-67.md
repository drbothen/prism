---
review_id: S-PLUGIN-PREREQ-E-spec-pass-67
pass_number: 67
reviewer: vsdd-factory:adversary
review_type: spec
artifact_scope: S-PLUGIN-PREREQ-E spec package (post-FB54 D-676; first pass under POL-29 v1.16 with all 3 recidivist-class variant-form registries populated)
parent_sha: "a5ab742c"
streak_pre_pass: "0/3"
streak_post_pass: "0/3"
verdict: BLOCKED
findings_count: 2
severity_breakdown:
  HIGH: 1
  MEDIUM: 0
  LOW: 0
  OBSERVATION: 1
novelty: HIGH (META — POL-29 v1.16 registry first-test surfaces active recurrence the registry was authored to detect; class (b) ADR-026 D7 pin recurrence #18; FB52→FB53→FB54 3 consecutive bursts failed step 8 STRENGTHENED enforcement)
pol_29_v16_first_test: REGISTRY_SOUND_ENFORCEMENT_ABSENT
related_state_decision: D-677
related_fix_burst: FB55
date: 2026-05-17
---

# Adversarial Review — Pass 67 (11th of restart-9; first test of POL-29 v1.16 with all 3 recidivist-class variant-form registries populated)

## Verdict
BLOCKED. 1 HIGH + 1 OBS [process-gap]. POL-29 v1.16 variant-form registries for classes (a), (b), (c) are syntactically and semantically sound — the (b) ADR-026 D7 pin canonical grep CORRECTLY identifies recurrence #18 of class (b) at ~18 live-narrative spec sites. The registry exists. The enforcement does not. Streak resets to 0/3.

## HIGH — F-LP67-HIGH-001 (ADR-026 D7 v1.17→v1.18 propagation gap; ~18 live-narrative spec sites; POL-29 v1.16 step 3a (b) recurrence #18)

**Evidence:** ADR-026 frontmatter at v1.18 (FB52 bump with D7 body edit at line 312); ARCH-INDEX line 94 confirms v1.18 propagation for ARCH-INDEX. Live-narrative D7 pins NOT propagated across:

| Site | File:Line | Current pin | Expected |
|------|-----------|-------------|----------|
| Story Task 7 plugin_name source cite | story:187 | `ADR-026 D7 v1.17` | `ADR-026 D7 v1.18` |
| Story Task 7b boot.rs cite | story:194 | `v1.17` | `v1.18` |
| Story AC-9 D7 cite | story:279 | `v1.17` | `v1.18` |
| Story §ACR row | story:365 (×2) | `v1.17` | `v1.18` |
| Story §FSR invalidation.rs row | story:402 | `v1.17` | `v1.18` |
| BC-2.16.012 §Postconditions plugin_name | BC-2.16.012:84 | `v1.17` | `v1.18` |
| BC-2.16.012 §Edge Cases EC-016-012-005 | BC-2.16.012:109 | `v1.17` | `v1.18` |
| BC-2.16.012 §Architecture Anchors | BC-2.16.012:138 | `§D7 v1.17` | `§D7 v1.18` |
| BC-2.16.012 (additional) | BC-2.16.012:124 | `v1.17` | `v1.18` |
| VP-156 §Property Statement | VP-156:42 | `v1.17` | `v1.18` |
| VP-156 §Source Contract BC row | VP-156:86 | `v1.17` | `v1.18` |
| VP-156 §Source Contract ADR row | VP-156:90 | `v1.17` | `v1.18` |
| VP-156 proof harness skeleton | VP-156:124 | `v1.17` | `v1.18` |
| HS-PREREQ-E-003 step 1 | HS-003:125 | `v1.17` | `v1.18` |
| HS-PREREQ-E-003 preconditions | HS-003:149 | `v1.17` | `v1.18` |
| HS-PREREQ-E-003 VP-156 trailer | HS-003:206 | `v1.17` | `v1.18` |
| ADR-022 §B Step 8 first-statement | ADR-022:243 | `§D7 v1.17` | `§D7 v1.18` |
| error-taxonomy E-PLUGIN-012 row body | error-taxonomy:459 | `v1.17` | `v1.18` |
| error-taxonomy E-PLUGIN-020 row body | error-taxonomy:467 | `v1.17` | `v1.18` |
| BC-2.16.002 catalog row (sibling-sweep catch) | BC-2.16.002:110 | `v1.17` | `v1.18` |

Total: 20 pin occurrences across 7 spec files.

**Provenance:** FB52 D-672 bumped ADR-026 v1.17→v1.18 with D7 body edit at line 312; D7-citation propagation was NOT performed. FB53 and FB54 did not catch the gap (3 consecutive bursts). POL-29 step 8 STRENGTHENED ("State-manager pre-commit verification: confirm sweep reports are present AND empirically validate post-grep counts against actual workspace state for each value class changed in the FB. Re-execute the per-variant greps from step 3/3a") did not fire. The variant-form registry codified in FB54 v1.16 detected the recurrence on its very first pass.

**Policy violated:** POL-29 v1.16 step 3a (b) + step 5 + step 8 STRENGTHENED; POL-23 within_fb_sibling_sweep_discipline; FB50 OBS-LP62-002 Interpretation #2.

**Proposed fix:** FB55 PO + architect joint sweep of all 20 sites; bump 7 spec files; INDEX cascade per POL-9 / POL-11. (CLOSED by FB55 PO + architect + state-manager.)

## OBS — OBS-LP67-001 [process-gap] (POL-29 v1.16 step 8 STRENGTHENED is a manual instruction not a tooling gate)

POL-29 v1.16 step 8 STRENGTHENED documents the verification requirement but `lint_hook: null` (policies.yaml line ~586) means there is no automated validator that runs the canonical greps at state-manager pre-commit. F-LP67-HIGH-001 is exactly the recurrence the v1.16 registry was authored to detect, and it still occurred because no tooling gate runs the registry's greps.

**Proposed cycle-close codification (S-7.02):**
- Author `hooks/validate-pol-29-variant-form-registry.sh` script that, given a candidate `.factory/` commit's changed value-classes, runs every canonical grep in POL-29 step 3a's variant-form registries against the live workspace and rejects the commit if any registry grep returns a non-zero count in spec-domain (excluding §Changelog rows per TD-VSDD-091).
- Wire into the factory-dispatcher hook chain at pre-commit for state-manager dispatches.
- Update POL-29 metadata: `lint_hook: hooks/validate-pol-29-variant-form-registry.sh` (replace `null`).
- Codification routing: cycle-close (NOT in-burst — requires structural design + factory-dispatcher integration that's beyond the immediate sweep). Drift item to track until v1.0.0-greenfield cycle close.

## POL-29 v1.16 First-Test Effectiveness Note

The variant-form registry for value-classes (b) and (c) is syntactically valid YAML and the recommended canonical greps semantically match the variant forms they enumerate (verified by hand-applying the regex and comparing to surfaced sites). The registry is sound. However, the registry alone did not prevent F-LP67-HIGH-001 because there is no tooling gate that runs the registry's own greps at FB-commit time. Cycle-close codification of `hooks/validate-pol-29-variant-form-registry.sh` is the structural counterpart to the FB54 policy-text amendment.
