---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-05-19T21:00:00Z
cycle: "wave-0-plugin-prereqs"
inputs: [STATE.md]
input-hash: "[extracted-2026-05-19-compact]"
traces_to: STATE.md
---

# Lessons Learned — wave-0-plugin-prereqs

<!-- Durable lessons from the PREREQ-E implementation cascade for future VSDD factory runs.
     Organized by category: agent-level, process-level, infrastructure-level.
     Extracted from STATE.md during D-727 compact-state. -->

## Agent-Level

1. **Rule C argument-semantic-aliasing is invisible to callsite-presence audits** — The cascade's
   compounding value was demonstrated when pass-3 verified validate_cross_composition at callsite-
   presence level, but pass-4 fresh-context verified it at argument-semantic-realism level and
   found Rule C structurally unreachable (both args alias auth_type). Adversary rubric should
   include callsite-argument-semantic-realism as an explicit attack vector.
   _Discovered: impl-pass-4 (D-704), 2026-05-18_

2. **VP artifact existence gap is a blind spot for body-logic auditors** — Passes 1–7 audited
   validator LOGIC but never grep-checked declared VP artifact existence. VP-153 P0 proptest was
   declared in story frontmatter + VP-INDEX but test FILE was missing. Adversary skill should add
   VP-existence verification as a mandatory attack vector.
   _Discovered: impl-pass-8 (D-710), 2026-05-18_

3. **Implementer flake-claim verification requires TD entry evidence** — Pass-2 caught a false
   flake claim (implementer cited TD-S-WAVE5-PREP-01-FLAKY-SIGTERM which does not exist; D-318
   records the test was FIXED 2026-05-09). Pass-7 independently adjudicated a separate flake claim
   Outcome (a). Standing Rule 3 §1 (orchestrator must independently verify pre-existing-flake
   claims) is essential and was validated as effective.
   _Discovered: impl-pass-2 (D-701) + impl-pass-7 (D-709), 2026-05-18_

4. **ZERO-DRIFT discipline enables rapid convergence after asymptote** — After passes 10–12
   demonstrated the cascade had entered spec-hygiene-only territory, introducing ZERO-DRIFT
   discipline (architect + PO self-audit checklist; minimum-touch; surface-not-sweep) produced
   pass-13 HIGH→MED severity transition and then passes 14/15/16 consecutive ZERO findings.
   ZERO-DRIFT is the correct convergence strategy once substantive defects have been eliminated.
   _Discovered: passes 13–16 (D-717..D-721), 2026-05-19_

5. **PR-LEVEL cascade catches CI-only invariants LOCAL cascade cannot** — LOCAL cascade (strict
   fresh-context against unchanged HEAD) correctly converged at pass-16, but PR #151 CI revealed
   2 real defects: (a) test reads .factory/ at runtime (`.factory/` is orphan-branch worktree,
   never shipped to CI); (b) cargo-semver-checks breaking-change version bump required. PR-LEVEL
   cascade is essential, not optional.
   _Discovered: pr-pass-1 (D-725), 2026-05-19_

## Process-Level

6. **Sibling-sweep VP proptests proactively prevents same-cycle blocking** — When VP-153 proptest
   landing was required, proactively landing VP-156 P1 (same class of declared-but-missing VP
   proptest) in the same burst prevented pass-9 from blocking on the identical gap pattern.
   TD-VSDD-060 sibling-sweep applied at test-writer scope.
   _Discovered: FB-IMPL-6 (D-711), 2026-05-18_

7. **Architect escalation for scope-boundary findings is faster than cascade iteration** — F-P5-001
   (Rule C dead in production keyring path) was a 3rd-iteration paper-fix lineage. Rather than
   continuing to close the same finding class with deeper fixes, escalating to architect for
   scope adjudication (Option B: backend-scope conditional) definitively resolved the class.
   For structural scope questions, architect adjudication in-scope is always faster.
   _Discovered: F-P5-001 → D-706, 2026-05-18_

8. **Rollback loop-continuation bugs require structural fix, not per-case guards** — The step 7.6
   orphaned-tool bug class (F-P6-001) required per-plugin atomic loop semantics (`continue
   'plugin_loop`) not a per-case try/catch. Structural fix eliminates the bug class at root;
   per-case guards would have left similar gaps.
   _Discovered: FB-IMPL-5 (D-708), 2026-05-18_

9. **User Option A "strict 3-CLEAN regardless of asymptote" produces full spec-hygiene coverage**
   — The asymptote signal was flagged honestly (D-715) and user chose Option A. The additional
   passes 12–16 caught: FB-IMPL-7/8 self-induced drift (3 HIGH), spec-hygiene sibling-sweep misses
   (2 MED). ZERO-DRIFT discipline then converged cleanly at pass-16. The passes were NOT wasted —
   they produced a more thoroughly audited spec kit.
   _Discovered: D-715→D-721, 2026-05-18..19_

10. **Test portability requirement (.factory/ never ships to CI)** — Any test that reads
    `.factory/` artifacts at runtime will fail in CI (`.factory/` is an orphan-branch worktree
    mount). Tests that need spec governance validation should use `.factory/hooks/` scripts invoked
    as factory pre-commit hooks, not embedded in Rust unit test assertions.
    _Discovered: F-PR-1-001 (D-725), 2026-05-19_

11. **cargo-semver-checks must run for each breaking API change before PR** — prism-spec-engine
    0.8.0 had 3 `*_missing` breaking API removals (CustomAdapter, CustomAdapterRegistry, SensorAuth)
    that required 0.8.0→0.9.0. Pre-1.0 SemVer: any breaking removal requires minor bump.
    LOCAL cascade cannot catch this; PR-LEVEL cascade (CI) catches it. Codify as pre-PR checklist
    item for all stories that retire public API surfaces.
    _Discovered: F-PR-1-002 (D-725), 2026-05-19_

## Infrastructure-Level

12. **Two-layer enforcement for spec governance invariants** — The E-SPEC-008 retirement annotation
    test was split into (a) `.factory/hooks/` shell script (spec governance layer) + (b) Rust unit
    test sub-assertion B (code layer). This pattern is generalizable: spec governance invariants
    belong in `.factory/hooks/`; code invariants belong in Rust compile-fail or unit tests.
    _Discovered: FB-PR-1 architect adjudication (D-725), 2026-05-19_

13. **vp156 proptest regression seeds require separate restoration after worktree force-removal**
    — `.worktrees/S-PLUGIN-PREREQ-E` was force-removed post-merge, which deleted the local
    proptest regression seeds file `vp156_write_tool_registration_uniqueness.proptest-regressions`.
    Seeds preserved offsite at /tmp/prism-vp156-regression-seeds-FOLLOWUP.txt for restoration
    via small maintenance PR. Worktree cleanup procedure should include proptest seed backup step.
    _Discovered: post-D-726 worktree cleanup, 2026-05-19_

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | Adversary rubric: callsite-argument-semantic-realism attack vector | adversary dispatch template | proposed |
| 2 | Adversary rubric: VP artifact existence grep before convergence | adversary dispatch template | proposed |
| 3 | Pre-PR checklist: cargo-semver-checks for API-retiring stories | pr-manager lifecycle | proposed |
| 10 | Codify: tests reading .factory/ at runtime are forbidden | dev conventions | proposed |
| 12 | Architecture pattern: spec-governance invariants in .factory/hooks/ | spec/test separation | proposed |
| 13 | Worktree cleanup: proptest seed backup before force-remove | devops-engineer procedure | proposed |
