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

## 2026-05-21 D-762 — Test-Writer Remediation

14. **Test-writer scope-creep risk pattern.** When test-writer is asked to create "stubs" alongside failing tests, there is a recurring tendency to author nearly-complete production artifacts and rationalize it via "canonical principle says no broken artifacts." In this instance, initial dispatch (commits 60081cb5/d6b197fa) produced 2.5 story points of implementer Task 3-6 TOML content disguised as stubs. The tell: 13 Red Gate tests passed instead of failing. Mitigation for future dispatches: explicitly enumerate the line-count budget for stubs AND the exact fields a skeleton may contain (e.g., for sensor.toml: only `sensor_id`, `name`, `auth_type`, `base_url`, `version` — no `[[tables]]`, no endpoint definitions, no credential schemas). Codification left to session-reviewer (S-7.02).
    _Discovered: D-762 test-writer remediation cycle, 2026-05-21_

15. **Red Gate vs. regression gate disambiguation.** The story spec uses `RG-NN` naming for all "must fail before implementation" tests, but several `RG-NN` tests are intentionally weak structural assertions that pass once spec files exist (e.g., RG-08 INV-SPEC-PARSER-OPEN-001 anti-pattern check, which is really an architectural-invariant gate that should pass at all times). This conflates two test categories: (a) genuine Red Gates that drive implementer behavior — should fail in skeleton state, go green when implementer completes Tasks 3-6/11/12; (b) regression-gates / architectural-invariant gates that should pass continuously. Naming confusion creates ambiguity in Red Gate pass/fail counts. Future stories should either rename category (b) to `AS-NN` (architectural sanity) or strengthen them to genuinely fail in Red Gate state. This is a story-writer / product-owner pattern observation; codification deferred to session-reviewer (S-7.02).
    _Discovered: D-762 Red Gate analysis (8 legitimate-PASS vs 13 FAIL breakdown), 2026-05-21_

## 2026-05-21 D-764 — FB-IMPL-1 Process Observations

16. **[process-gap] Architect event_type ↔ BC-2.16.002 catalog propagation.** During FB-IMPL-1, architect specified `tracing::warn!(event_type = "timestamp.fallback_to_now")` in ADR-028 §D8-B + BC-2.16.013 v1.12 but did NOT add the corresponding catalog row to BC-2.16.002. PO had to be dispatched as a separate burst (b3989982) to close the routing gap before implementer dispatch — otherwise the next adversary pass would have flagged a guaranteed P1 finding per CLAUDE.md §Conventions structured event catalog discipline. The extra burst consumed one additional single-commit slot. Proposed codification target deferred to session-reviewer (S-7.02): architect agent prompt amendment — when architect specifies a new tracing emission, MUST EITHER add BC-2.16.002 catalog row in same burst OR explicitly hand off to PO via orchestrator with explicit checklist entry. Recurrence count: 1 (this burst).
    _Discovered: D-764 FB-IMPL-1-PO-B burst (b3989982), 2026-05-21_

17. **[process-gap] Implementer "deferred to non-ignored test" rationalization for spec-contract behavior.** During FB-IMPL-1, implementer burst-1 closed 8 of 9 actionable findings but rationalized PipelineExecutor Option A normalization as "deferred to non-ignored test" because all integration tests were #[ignore]'d for DTU clone reasons. This is a misapplication of TDD: a missing test is a TEST gap, not justification to defer production behavior required by an ADR-locked contract. Orchestrator rejected via Standing Rule 3 §1 + Canonical Principle Rule 1; remediation burst (implementer 8b480db8) added 7 driving unit tests that exercised the new behavior directly, bypassing the DTU dependency. Proposed codification target deferred to session-reviewer (S-7.02): implementer agent prompt amendment — when no failing test drives a spec-required behavior, the correct path is to ADD a unit test that drives it, NOT to defer the implementation. Adversary verification checklist amendment — when implementer ships ColumnSpec / config / TOML fields without runtime consumers, flag as a paper-fix per TD-VSDD-059 even if the field exists.
    _Discovered: D-764 FB-IMPL-1 orchestrator rejection + remediation cycle, 2026-05-21_

18. **[OBSERVATION] FB-IMPL-1 adversary pass-1 finding novelty.** Pass-1 had high novelty: 15 findings, 4 CRITICAL on URL/response_path/pagination that would have silently failed at runtime against the DTU clones. The CRITICAL findings are exactly the kind of TDD-shipped code that passes `just check` (because the integration tests are #[ignore]'d) but fails against real DTU clones. The adversary's verification probe #1 (TOML spec correctness) caught all 4 CRITICAL findings via grep-and-compare against DTU route registrations. Reinforces value of adversary's perimeter-aware verification probes for stories with #[ignore]'d DTU-gated integration tests.
    _Discovered: D-764 pass-1 adversary report (agent a598496b1b1bf90c4), 2026-05-21_

## 2026-05-21 D-765 — FB-IMPL-2 Process Observations

19. **[process-gap] [RECURRING — 2nd occurrence in this cascade] NEW tracing emission without BC-2.16.002 catalog row.** Pass-1 fix-burst (FB-IMPL-1) architect specified `event_type = "timestamp.fallback_to_now"` without adding catalog row (closed by PO burst b3989982). Pass-2 finding F-LP2-HIGH-001 caught implementer adding `event_type = "timestamp_parse_failure"` without adding catalog row (closed by REMOVAL in commit 6ae464c3 since `?` propagation provides audit trail). Both occurrences in same cascade indicates this is a recurring axis worth tracking. The orchestrator-discovered remediation pattern (option to REMOVE the emission when structural error propagation suffices) is itself a useful policy clarification. Proposed codification target (defer to session-reviewer): add explicit checklist item to architect + implementer agent prompts — when adding a `tracing::*!(event_type=…)` site, EITHER add BC-2.16.002 catalog row in same burst OR justify why emission is non-redundant with structural error propagation. Recurrence count: 2 (this cascade).
    _Discovered: D-765 FB-IMPL-2, F-LP2-HIGH-001 + D-764 FB-IMPL-1-PO-B (b3989982), 2026-05-21_

20. **[OBSERVATION] Parallel-burst hook coordination via git stash --keep-index.** During FB-IMPL-2, implementer and test-writer ran in parallel on the same feature branch worktree (touching disjoint files: `crates/prism-spec-engine/src/` vs `crates/prism-spec-engine/tests/`). Test-writer's commit-time hook (cargo fmt --all --check) included implementer's in-progress unformatted spec_parser.rs changes, causing hook failure. Test-writer organically adopted `git stash --keep-index` to isolate hook run to only staged content. This worked. Proposed codification target (defer to session-reviewer): orchestrator parallel-burst dispatch rule — when dispatching agents in parallel on the same feature branch, EITHER (a) sequence them, (b) require each agent to use `git stash --keep-index` pattern, OR (c) require each agent to pre-format their own files to avoid workspace-wide fmt drift. Recurrence count: 1 (this cascade).
    _Discovered: D-765 FB-IMPL-2 parallel test-writer/implementer dispatch, 2026-05-21_

21. **[OBSERVATION] POL-29 sibling-sweep partial application (catalog-row vs body-sweep asymmetry).** Pass-2 finding F-LP2-MEDIUM-002 caught story body still citing BC-2.16.002 v1.35 after FB-IMPL-1 PO commit b3989982 bumped to v1.36. PO performed sibling sweep on PRIMARY citation sites (BC-INDEX, BC-2.16.012, S-PLUGIN-PREREQ-E, STORY-INDEX) but missed PLUGIN-MIGRATION-001-D story body BC table. This is partial POL-29 application — sweep scope was broad but not exhaustive. Proposed codification target (defer to session-reviewer): tighten POL-29 enforcement check — when PO/architect bumps a BC version, run `grep -rn "BC-N.NN.NNN v<old>" .factory/` BEFORE committing AND verify zero results (excluding §Changelog immutable historical entries) per POL-29 step 8c amendment. Recurrence count: 1 (this cascade).
    _Discovered: D-765 FB-IMPL-2, F-LP2-MEDIUM-002, 2026-05-21_

22. **[OBSERVATION] Validator scope narrowing pattern (initial implementation handles happy path but not edge case).** Pass-2 finding F-LP2-HIGH-005 caught implementer's BC-2.16.009 validator only checking timestamp_formats on Datetime columns; non-Datetime columns declaring nonsense timestamp_formats passed validation silently. Implementer fixed in commit 7d03917c with proper two-stage validation (Stage 1: reject on wrong column type, Stage 2: recognized-format-set check). Pattern observation: when implementing a validator gate, the initial implementation tends to handle the "obvious" type (where the validator is most relevant) and miss the "shouldn't happen" cases. Not a recurring axis yet. Proposed codification target (defer to session-reviewer): add to implementer self-audit checklist — for validator/check code, explicitly enumerate ALL possible input states + verify each has explicit handling.
    _Discovered: D-765 FB-IMPL-2, F-LP2-HIGH-005, commit 7d03917c, 2026-05-21_

## 2026-05-21 D-767 — FB-IMPL-4 Process Observations

23. **[OBSERVATION] [RECURRING — 3rd occurrence in cascade] POL-29 partial sibling-sweep, this time implementer-introduced.** Pass-4 finding F-LP4-HIGH-001 surfaced 17 stale ADR-028 v1.9 cite sites that arose because architect bumped ADR-028 v1.9→v1.10 in FB-IMPL-2 commit eb714b3c BEFORE implementer wrote subsequent commits adding NEW source code / TOML / test cites that pinned the (then-current-at-burst-start) v1.9. Implementer should have read the LATEST ADR-028 version after architect's bump landed. This is the 3rd recurrence of the POL-29 partial-sweep axis in this cascade: (1st: pass-2 PO body BC version sweep gap; 2nd: pass-3 OCSF copy-paste sibling defect; 3rd: this pass implementer cite-pin drift). Implementer's FB-IMPL-4 dispatch demonstrated the corrective pattern (`grep -rn "ADR-028 v1\.9" crates/` returning ZERO matches before declaring done; sibling-grep found 3 additional sites the adversary undercounted). **Proposed codification target (defer to session-reviewer):** add to implementer agent prompt — when authoring code that cites a versioned spec artifact (BC vN.NN, ADR vN.NN, error-taxonomy vN.NN, etc.), READ the LATEST artifact version FIRST and cite that version. After completing a fix-burst that includes cites, run `grep -rn "<artifact-name> v<old>"` workspace-wide to confirm no drift. Recurrence count: 3 (across this cascade).
    _Discovered: D-767 FB-IMPL-4 F-LP4-HIGH-001, 2026-05-21_

24. **[OBSERVATION] Cascade convergence trajectory acceleration.** Pass-1 found 15 findings, pass-2 found 13, pass-3 found 10, pass-4 found 2. This is monotonic decay with acceleration. PREREQ-D precedent took 25 passes / 19 fix-bursts; PLUGIN-MIGRATION-001-D is at pass-4 / 4 fix-bursts with finding count decay suggesting potential 3-CLEAN convergence within 3-5 more passes. The accelerating decay is likely attributable to: (a) explicit sibling-sweep mandate added to implementer briefs starting D-766; (b) production-grade default discipline enforcement (no MVP deferrals); (c) high-quality fresh-context adversary probes finding novel axes each pass that drove targeted remediation. **Observation:** the cascade is more efficient than PREREQ-D historical baseline — possibly because PREREQ-D's defect-pattern accumulation (POL-29 evolved through 12 amendments during that cascade) had front-loaded the policy maturation work. PLUGIN-MIGRATION-001-D inherits a mature POL-29 + agent discipline corpus.
    _Discovered: D-767 cascade trajectory analysis, 2026-05-21_

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | Adversary rubric: callsite-argument-semantic-realism attack vector | adversary dispatch template | proposed |
| 2 | Adversary rubric: VP artifact existence grep before convergence | adversary dispatch template | proposed |
| 3 | Pre-PR checklist: cargo-semver-checks for API-retiring stories | pr-manager lifecycle | proposed |
| 10 | Codify: tests reading .factory/ at runtime are forbidden | dev conventions | proposed |
| 12 | Architecture pattern: spec-governance invariants in .factory/hooks/ | spec/test separation | proposed |
| 13 | Worktree cleanup: proptest seed backup before force-remove | devops-engineer procedure | proposed |

## 2026-05-21 D-766 — FB-IMPL-3 Process Observations

23. **[OBSERVATION] [RECURRING — 2nd occurrence] Fix-burst partial sweep pattern — sibling defects missed.** Pass-3 finding F-LP3-HIGH-002 (armis manufacturer OCSF) is the SAME copy-paste defect CLASS as pass-2 F-LP2-HIGH-003 (cyberint alert_id OCSF), but pass-2 fix only swept the called-out site (cyberint), missing armis. F-LP3-HIGH-004 (claroty body_template `{"size": 100}`) is a PARTIAL fix of pass-2 F-LP2-CRIT-002 (Claroty body_template removed `${page_offset}` but left `{"size": 100}`). Both are extensions of pass-2 fixes that didn't sweep semantic siblings. The orchestrator's FB-IMPL-3 brief added EXPLICIT "sibling-sweep mandate" instructions to implementer's HIGH-002 + HIGH-003 sub-tasks, and implementer report confirms zero other defects found in workspace-wide sweep. **The explicit sibling-sweep mandate worked** — pass-3's FB-IMPL-3 produced no further sibling drift. **Proposed codification target (defer to session-reviewer):** add to implementer agent prompt — when fixing a defect, if the defect is part of a known recurring CLASS (copy-paste defect, type-mismatch, etc.), MUST execute workspace-wide grep to find ALL sibling instances of the same class. Recurrence count: 2 (across this cascade).
    _Discovered: D-766 FB-IMPL-3, F-LP3-HIGH-002 + F-LP3-HIGH-004, 2026-05-21_

24. **[OBSERVATION] DTU↔TOML schema drift as a productive adversarial probe axis.** Pass-3 surfaced 4 HIGH findings (HIGH-001/002/003 + MEDIUM-004) all rooted in DTU types.rs / routes.rs schemas vs TOML column declarations. Prior passes focused on grammar, error-taxonomy registration, and normalization path — none had explicitly cross-checked TOML columns against DTU emission shapes. The adversary's pass-3 introduction of probe #1 (TOML spec correctness vs DTU) + probe #16 (HIGH-003 sibling sweep) was high-value. **Proposed codification target (defer to session-reviewer):** add DTU↔TOML schema parity probe to adversary verification probe checklist as a standing requirement for sensor-spec stories. Adversary should explicitly READ the DTU types.rs + routes/<table>.rs files and cross-reference EVERY column declaration in the TOML spec.
    _Discovered: D-766 FB-IMPL-3, pass-3 adversary report, 2026-05-21_

25. **[OBSERVATION] Production-grade architectural deferral pattern (affected_assets array column).** During HIGH-001 cyberint reauthoring, implementer encountered the DTU `affected_assets: Vec<serde_json::Value>` field which has no representation in the current TOML column grammar (array columns not supported). Implementer correctly DEFERRED this column with INLINE DOCUMENTED RATIONALE in the TOML comment, citing the legitimate architectural constraint (not a quality shortcut). This satisfies Canonical Principle Boundary 3 ("does not mean infinite scope expansion"). The deferral is correctly attached to the spec via inline TOML comment + (implicit) future-story implication. **Observation:** this is the CORRECT application of the production-grade default — deferring an architectural extension (array column grammar) is legitimate; deferring a documented contract (Option A normalization) is forbidden. The discriminator is whether the deferral preserves a TESTED contract vs leaves a contract unimplemented.
    _Discovered: D-766 FB-IMPL-3, F-LP3-HIGH-001 cyberint reauthoring, 2026-05-21_
