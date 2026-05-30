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

## 2026-05-21 D-768 — FB-IMPL-5 Process Observations

25. **[OBSERVATION] [RECURRING — 2nd occurrence] "PO BC bump → architect ADR pin cascade" micro-pattern.** Pass-5 PO commit a2ef75e1 bumped BC-2.16.002 v1.36→v1.37 and surfaced ADR-026:336 cite-pin to orchestrator for architect routing. Same exact pattern as f9f6feed (architect advanced ADR-026 v1.32→v1.33 after PO's earlier BC-2.16.002 v1.35→v1.36 bump). Both followed identical sequence: PO touches BC, PO grep-sweep finds ADR-026 cite-pin, PO correctly stays in scope and surfaces to architect, architect dispatches single-line cascading commit. **Proposed codification target (defer to session-reviewer):** EITHER (a) extend PO scope to allow trivial ADR cite-pin advances when no semantic content change (the cite is purely referential), OR (b) add to PO agent prompt mandate — when bumping any BC version, run `grep -rn "<BC-ID> v<old>" .factory/specs/architecture/` BEFORE committing AND surface ALL ADR sites to orchestrator with proposed architect handoff in same surface message. Recurrence count: 2 (across this cascade).
    _Discovered: D-768 FB-IMPL-5 PO commit a2ef75e1 + architect commit c1aae7fe, 2026-05-21_

26. **[OBSERVATION] First zero-HIGH adversary pass — cascade convergence accelerating.** Pass-5 produced zero CRITICAL + zero HIGH findings for the first time in this cascade. Trajectory 15→13→10→2→3 shows asymptote behavior. All remaining findings are POL-29 propagation hygiene (cite-pin sweeps), not semantic correctness defects. The 10 positive OBSERVATIONs (byte-fidelity verified, tracing catalog clean, #[non_exhaustive] OK, no stub residue, defensive guards load-bearing, DTU cross-check clean, chrono dep correct, HTTP timeouts applied) form a comprehensive structural-correctness confirmation. **Observation:** the adversary's positive-confirmation discipline (explicitly noting what's verified-clean, not just what's defective) is high-value for cascade-end stages where defects become rare. Future adversary briefs should encourage this explicit positive-verification reporting — it builds human confidence in convergence claims and provides durable evidence of structural correctness that survives future passes.
    _Discovered: D-768 pass-5 adversary report (0 CRIT + 0 HIGH + 1 MED + 2 LOW + 10 OBS), 2026-05-21_

27. **[OBSERVATION] Pre-commit hook efficiency for doc-comment-only sweeps.** FB-IMPL-5 implementer (LOW-002 sweep) committed before `just check` workspace run completed, relying on pre-commit hooks (layout + fmt + clippy) as the gate. Defensible call: doc-comment-only changes structurally cannot break tests (no source semantics modified, no API surface touched, no test files modified in production assertion behavior). The full workspace run was running in parallel and showed 0 FAILED at the point of commit (1609/3724 PASS, no failures). **Observation:** when fix-burst scope is exclusively narrative (comments, doc-strings, TOML comments, README updates), pre-commit hooks alone are sufficient gating; full workspace `just check` is high-value-but-optional. Implementer agent prompt could acknowledge this tradeoff explicitly to avoid unnecessary full-workspace-wait delays during doc-comment-only fix bursts.
    _Discovered: D-768 FB-IMPL-5 implementer LOW-002 commit 6a0ca01e, 2026-05-21_

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

## 2026-05-21 D-769 — FB-IMPL-6 Process Observations

26. **[OBSERVATION] Pre-existing TD-VSDD-091 violation in story task-body exposed by cascade-induced line-number drift.** The 7 line-number cite-pins in story task-body sections (Tasks 3/4/5/6/11/12) were authored BEFORE the cascade started as "verified by reading the file" attestations. They were correct at story-authoring time (commits prior to FB-IMPL-1) but decayed as cascade fix-bursts shifted source-code line numbers. The cascade did not INTRODUCE these cites but DID cause their decay through valid source-code changes. Per S-7.01 partial-fix regression discipline, pass-6 correctly flagged them as residual drift. Per Canonical Principle "no pragmatic convergence," swept rather than accepted as exempt. **Proposed codification target (defer to session-reviewer):** add to story-writer agent prompt — when authoring "verified by reading the file" attestation cites, use function-name anchors (e.g., `SpecLoader::load_all`) NOT line-number cites (e.g., `spec_parser.rs:715`). Update CLAUDE.md TD-VSDD-091 to explicitly call out task-body attestation cites as in-scope (not exempt class). Recurrence count: 1 in this cascade (pre-existing condition, not introduced by cascade).
    _Discovered: D-769 FB-IMPL-6, F-LP6-LOW-001 TD-VSDD-091 task-body line-cite sweep, 2026-05-21_

## 2026-05-21 D-770 — FB-IMPL-7 Process Observations

27. **[OBSERVATION] [process-gap from pass-7] POL-29 step 8f sibling-sweep scope incompleteness — crates/ cite-pins missed during .factory/-only PO sweeps.** Pass-7 F-LP7-OBS-001 identified that FB-IMPL-5 PO burst (4d934f28) bumped BC-2.16.002 v1.36→v1.37 with no-op content change and ran POL-29 sibling-sweep across `.factory/` but did NOT include `crates/` doc-comment cite-pins. Result: 1 stale cite-pin survived in `pipeline.rs:2774` test-doc until pass-7 caught it. The PO scope itself does not normally include `crates/`, so this is a routing question: who runs the `crates/` sweep when PO bumps a BC version? Options: (a) PO scope expansion to grep `crates/` and surface to implementer in same burst with explicit handoff, (b) state-manager scope expansion to include `crates/` sweep in its closure burst when BC version bumped, (c) standing implementer hook to grep `crates/` after every BC PO commit. **Proposed codification target (defer to session-reviewer):** add to POL-29 step 8f explicit clause — "when bumping BC frontmatter version with no semantic content change, sibling-sweep MUST include both `.factory/` AND `crates/**/*.rs` paths; route `crates/` findings to implementer if PO scope cannot directly modify code." Recurrence count: 1 (this cascade — first occurrence; worth tracking).
    _Discovered: D-770 FB-IMPL-7, F-LP7-OBS-001 process-gap, 2026-05-21_

## 2026-05-21 D-771 — Pass-8 First CLEAN-ZERO

29. **[OBSERVATION] First true CLEAN-ZERO pass marks convergence inflection.** Pass-8 produced zero findings of any severity (no LOW, no OBS, no process-gap) after comprehensive 13-probe verification. The adversary's own framing — "convergence-by-content rather than convergence-by-fatigue" — captures the key insight: when a fresh-context adversary applying ALL standard probes finds nothing, this is strong evidence the implementation has reached genuine structural and propagation correctness. Compare to convergence-by-fatigue (adversary stops looking) or convergence-by-narrowing (probes get less rigorous). Pass-8's 13 probes were the same set used in passes 3/4/5/6/7; the difference is the artifact, not the inquiry. The cascade architecture (fresh-context per pass, no leak from prior reviews, strict probe checklist) makes convergence-by-content vs convergence-by-fatigue distinguishable. This is a high-confidence convergence signal.
    _Discovered: D-771 pass-8 CLEAN-ZERO (agent a0b2ed17335e90dc0), 2026-05-21_

28. **[OBSERVATION] First "CLEAN-per-criterion" pass — convergence criterion clarification.** Pass-7 result (0 CRIT/HIGH/MED + 1 LOW + 1 OBS) was the first to satisfy the lenient convergence criterion explicitly stated in the orchestrator's pass-7 brief ("CLEAN = zero CRITICAL+HIGH+MEDIUM; LOW + OBSERVATION allowed"). The adversary correctly interpreted this and marked CLEAN: yes. However, per Canonical Principle "no pragmatic convergence," orchestrator chose to honor the strict BC-5.39.001 interpretation (any finding resets streak) and dispatched FB-IMPL-7 to sweep the LOW. **Observation:** the cascade now has TWO operating convergence definitions: (a) lenient ≥0 CRIT+HIGH+MED → CLEAN (used for adversary's CLEAN-flag interpretation), and (b) strict zero findings → streak advances (used for orchestrator's fix-burst dispatch decision). These should be unified. **Proposed codification target:** add to BC-5.39.001 explicit clarification — "CLEAN means zero findings of ANY severity for streak-advancement; LOW + OBSERVATION acceptable for PR-merge but each occurrence resets the streak counter. The cascade gate (3-CLEAN streak) is the threshold for declaring CONVERGED; the PR-merge gate is a lower bar (no CRIT/HIGH/MED)."
    _Discovered: D-770 FB-IMPL-7, pass-7 CLEAN-per-criterion vs BC-5.39.001 strict interpretation, 2026-05-21_

## 2026-05-21 D-772 — FB-IMPL-8 POL-29 4th Recurrence

30. **[OBSERVATION] [RECURRING — 4th occurrence] POL-29 variant-form grep enumeration discipline gap.** PO commit a2ef75e1 (FB-IMPL-5) bumped BC-2.16.002 v1.36→v1.37 and ran POL-29 sibling-sweep grep using pattern `BC-2.16.002 v1.36` (with `v` prefix). The grep correctly found 4 hits — 3 immutable changelog rows + 1 ADR-026:336 architect-domain referral. **But the grep pattern missed the table-cell variant form** `| 1.36 |` at story line 195 (which uses `|` pipe table syntax without `v` prefix). POL-29 v1.19 step 8b ALREADY mandates "EXPLICIT PER-VARIANT GREP ENUMERATION" specifically to prevent this class of miss (codified after F-LP75-HIGH-001 META-META-PATTERN in PREREQ-E cascade). The discipline exists; the first-application coverage is the gap. **Proposed codification target (defer to session-reviewer):** add POL-29 step 8b SUPPLEMENT — when bumping any BC/ADR version, run a STANDARD WORKSPACE GREP CHECKLIST including: (a) `<artifact-name> v<old>` (v-prefix form), (b) `<artifact-name> | <old> ` (table-cell pipe form), (c) `<artifact-name>.* row <N>` (anchor-cite form), (d) backtick-quoted forms, (e) any project-specific variants in the registry. PO/architect/state-manager agent prompts should include this checklist verbatim. Recurrence count: 4 in this cascade (pass-2 MED-002 same-class but different sibling site → 3 cumulative recurrences caught in prior FBs; pass-9 LOW-001 = 4th cumulative).
    _Discovered: D-772 FB-IMPL-8, F-LP9-LOW-001 table-cell sweep, 2026-05-21_

31. **[OBSERVATION] Cascade trajectory: regression from pass-8 zero-findings to pass-9 LOW.** Pass-8 was the first zero-findings pass; pass-9 found 1 LOW (the same recurring axis). This shows that even at deep asymptote, fresh-context adversary can surface NEW instances of KNOWN classes when the grep coverage in prior bursts was incomplete. **Observation:** the cascade-convergence criterion (3 consecutive CLEAN-ZERO passes) provides legitimate robustness against this — single-pass CLEAN is not sufficient; multi-pass confirmation catches the residual variant-form drift. The lenient criterion (zero CRIT+HIGH+MED → CLEAN) would have allowed convergence at pass-7; the strict criterion catches the recurring axis at pass-9. **Proposed codification target:** the strict 3-CLEAN-ZERO interpretation is the canonical convergence criterion; the lenient criterion is acceptable for PR-merge gating but NOT for cascade convergence.
    _Discovered: D-772 FB-IMPL-8, pass-9 regression from pass-8 CLEAN-ZERO, 2026-05-21_

## 2026-05-21 D-773 — FB-IMPL-9 POL-29 5th Recurrence (Transitive Cite-Pin Chain)

32. **[OBSERVATION] [RECURRING — 5th occurrence, NEW dimension] POL-29 transitive cite-pin chain drift.** Pass-10 F-LP10-LOW-001 surfaced 3 sites (`BC-2.16.013:357 error-taxonomy.md v1.42`, `BC-2.16.013:358 error-taxonomy.md v1.43`, `story:1017 error-taxonomy.md v1.42`) all cite STALE error-taxonomy versions while error-taxonomy is now v1.44. **Root cause:** when error-taxonomy bumped independently (FB-IMPL-1 v1.42→v1.43 for E-SPEC-018 registration, then FB-IMPL-5 PO v1.43→v1.44 for POL-30 Fork B sweep), neither burst's POL-29 sibling-sweep checked for `per error-taxonomy.md v...` cite-pin chains in OTHER artifacts (BCs, stories, HSs). Each bump's POL-29 sweep was scoped to within-artifact sibling sites + same-class cite-pins, not transitive chains. **Distinguishing this from prior recurrences:** the 4 prior POL-29 recurrences were all within-class (same artifact type citing same version pattern). This 5th is cross-artifact-class (artifact A bumps → cite-pin chain to A in artifact B becomes stale). **Grep sweep discovered 6 sites total, not just the 3 enumerated by pass-10:** BC-2.16.013:357 (v1.42), BC-2.16.013:358 (v1.43), story:1017 (v1.42), and additionally HS-018:31 (v1.42), HS-018:71 (v1.42), HS-018:89 (v1.42) — all swept. **Proposed codification target (defer to session-reviewer):** add POL-29 step 8 EXTENSION — when bumping any authoritative artifact (BC, ADR, error-taxonomy, capability), run a TRANSITIVE CITE-PIN CHAIN grep with patterns: (a) `per <artifact-name>.md v<old>` (b) `registered in <artifact-name>.md v<old>` (c) `<artifact-name> v<old> §` (d) `<artifact-name> | <old> ` table-cell variants. The grep must search BOTH .factory/ AND crates/ scope. This is a CHAIN-class extension to POL-29 v1.19's per-variant enumeration (POL-29 v1.20 proposal). Recurrence count: 5 in this cascade (4 within-class + 1 cross-artifact-class = first occurrence of the chain dimension).
    _Discovered: D-773 FB-IMPL-9, F-LP10-LOW-001 transitive cite-pin chain, 2026-05-21_

33. **[OBSERVATION] Cascade convergence pattern: each fresh-context pass surfaces ONE residual sibling-sweep variant.** Trajectory: pass-6 → 1 LOW (pipeline.rs:2774 sweep gap); pass-7 → 1 LOW (sweep gap); pass-8 → 0 (CLEAN); pass-9 → 1 LOW (table-cell variant); pass-10 → 1 LOW (transitive chain). Pattern: each pass-after-asymptote uncovers a NEW variant of POL-29 partial-sweep that prior bursts' grep coverage did not include. The CASCADE is doing the variant-form-discovery work that POL-29 step 8 should be doing in fewer bursts. **Observation:** the BC-5.39.001 3-CLEAN-streak discipline catches these naturally; the alternative (lenient streak that accepts LOW) would have shipped 001-D with at least 3 known POL-29 partial-sweep gaps. **Proposed codification target:** the 3-CLEAN-ZERO strict criterion is essential for catching POL-29 variant-form coverage gaps; it is not just a finding-count threshold but a variant-discovery mechanism. Document this rationale in BC-5.39.001 itself.
    _Discovered: D-773 FB-IMPL-9, cascade convergence meta-pattern observation, 2026-05-21_

## 2026-05-21 D-774 — FB-IMPL-10 POL-29 6th Recurrence + 2nd-Order Discipline Application

34. **[OBSERVATION] [RECURRING — 6th occurrence] POL-29 2nd-order propagation gap.** FB-IMPL-9 swept the error-taxonomy A→B leg (cite-pins inside BC-2.16.013), as a side-effect bumped BC-2.16.013 frontmatter v1.14→v1.15, but did NOT propagate the side-effect bump to the 9 downstream B→{downstream} sites (8 story body sites + 1 BC-2.16.002 row 112 Anchor). F-LP11-MED-001 = 9 stale cite-pins. **This is the cascade's recurring-axis structure:** each closure burst that bumps a frontmatter as side-effect creates a new downstream propagation gap. FB-IMPL-10 applies PROACTIVE 2nd-order grep discipline: after each frontmatter bump in the burst, immediately re-scan workspace for stale downstream cite-pins to the OLD version. The 2nd-order grep found 1 additional LIVE site (story line 195 `| BC-2.16.002 | 1.37 |` → `| 1.38 |`) that would otherwise have become F-LP12-LOW-001. **Codification candidate (defer to session-reviewer):** POL-29 step 8 EXTENSION — when a closure burst's STAGED diff includes ANY frontmatter version delta as SIDE-EFFECT (i.e., a BC was bumped because its cite-pins were swept, not because its own body changed), run workspace grep for the OLD version IN ADDITION to the primary swept value-class. The grep must include all variant forms (v-prefix, table-cell, anchor cite, transitive chain). This is iterative: each bump-cascade level requires another grep until iteration produces zero new bumps (POL-29 v1.24 step 8e FIXED-POINT discipline already mandates this for self-bumps — extension to BUMPS-AS-SIDE-EFFECT-OF-OTHER-SWEEPS is the missing piece). Recurrence count: 6 in this cascade.
    _Discovered: D-774 FB-IMPL-10, F-LP11-MED-001 2nd-order propagation gap, 2026-05-21_

35. **[OBSERVATION] Cascade-as-policy-discovery mechanism.** Across 11 adversary passes + 10 fix-bursts, the cascade discovered 6 distinct POL-29 variant axes: (1) story body BC-version sweep gap, (2) OCSF mapping copy-paste sibling, (3) ADR cite-pin sibling, (4) variant-form grep enumeration (table-cell vs v-prefix), (5) transitive cite-pin chain, (6) 2nd-order propagation. Each was caught by fresh-context adversary applying standard probes. The CASCADE itself has done the variant-discovery work that POL-29 step 8 amendments should be doing in fewer iterations. The codification debt accumulated in lessons.md (35+ entries spanning 6 POL-29 axes) is the structural deliverable that should ship with the 001-D PR as a session-reviewer adjudication queue. **Observation:** the cascade's variant-discovery cost is real (~12 hours of compute time, 286 commits, 48 closures) but produces a measurable POL-29 specification improvement that prevents similar drift in future stories. The ROI question is whether the policy-discovery work justifies the per-story cost OR should be front-loaded into a dedicated codification cycle.
    _Discovered: D-774 FB-IMPL-10, cascade-as-policy-discovery meta-pattern, 2026-05-21_

## 2026-05-22 D-775 — Cascade Exit per User Option B

38. **[OBSERVATION] [USER ADJUDICATION] Cascade exit via Option B at pass-12 DECISION POINT.** User accepted accept-with-codification trade-off after 12 passes / 11 fix-bursts / 7 POL-29 axis recurrences. Rationale: code correctness verified since pass-8 ZERO-findings + all subsequent passes confirm; tests load-bearing 3724/3724 +43 net; spec semantics byte-fidelity verified; the 7 POL-29 recurrences are PURELY documentation-pin-propagation (no semantic/runtime risk); 35+ lessons.md entries form structural codification queue for session-reviewer; user goal is live MCP+DTU+OCSF demo, critical path 001-D PR + 001-A + 001-B + 001-C; diminishing returns on cascade vs opportunity cost on demo path; "no pragmatic convergence" was about CORRECTNESS — code IS production-grade. FB-IMPL-11 sweeps line 132 (still production-grade default for the open finding: the body header was stale, it is now correct) + extends self-2nd-order discipline as a new POL-29 axis. **Codification queue summary — 7 POL-29 axis recurrences documented in lessons.md entries 14-37:** (1) tracing-without-catalog-row (2 occurrences, lessons 21-22), (2) implementer defer-to-non-ignored-test rationalization (1 occurrence, lesson 23 partial), (3) DTU↔TOML schema parity probe as productive adversary axis (lesson 24), (4) PO BC-bump→architect ADR cite cascade (2 occurrences, lessons 30-31), (5) crates/ scope incompleteness in POL-29 step 8f (lesson 27), (6) variant-form grep enumeration discipline (lesson 30), (7) transitive cite-pin chain (lesson 32), (8) 2nd-order propagation (lesson 34), (9) self-2nd-order propagation (this entry — body header cites stale frontmatter version). These are session-reviewer adjudication candidates for POL-29 v1.20+ amendments. **Process observation:** the LOCAL cascade's variant-discovery cost (~12 hours, 287 commits, 49 closures) produced a measurable POL-29 specification improvement; the question for session-reviewer is whether to apply these amendments retroactively across other in-flight cycles or front-load them as a dedicated codification cycle.
    _Discovered: D-775 USER OPTION B DECISION 2026-05-22, pass-12 DECISION POINT after 7 POL-29 axis recurrences_

## 2026-05-24 D-810 — S-CONFIG fix-burst-3 F-LP2-MED-002 Sibling-Sweep Gap

39. **[PROCESS GAP] #[non_exhaustive] compile-fail gate EXPECTED count not swept after adding new public types.** S-CONFIG-MULTI-TENANT-OVERRIDE-001 fix-burst-3 introduced 3 new #[non_exhaustive] public types but did NOT update `ci.yml EXPECTED=32` to `EXPECTED=35`. F-LP2-MED-002 caught this in adversary pass-2. **Root cause:** the implementer's sibling-sweep scope (TD-VSDD-060) did not include `ci.yml` EXPECTED count as a sibling site for the `#[non_exhaustive]` discipline defined in CLAUDE.md §Conventions. The compile-fail gate at `tests/external/non-exhaustive-violation/` (CLAUDE.md: "32+ types currently enforced via compile-fail gate at `tests/external/non-exhaustive-violation/`; `ci.yml EXPECTED=32` is the authority") explicitly names `ci.yml` as the authority. **Proposed codification target:** when an implementer adds a #[non_exhaustive] type to a crate in the compile-fail gate's scope (prism-core, prism-spec-engine, prism-query), the mandatory sibling sweep MUST include: (a) `tests/external/non-exhaustive-violation/` crate for the new type appearance, (b) `ci.yml EXPECTED=N` count update. Grep pattern: `EXPECTED=[0-9]+` in both `.github/workflows/ci.yml` and compile-fail gate crate. **This is a new dimension of TD-VSDD-060 sibling-site sweep** (CLAUDE.md Operational Discipline TDs): the count is a cross-file numerical citation that decays when types are added without updating the count. **Adversary standing probe extension (SAP-3 candidate):** when any story adds #[non_exhaustive] types, adversary MUST grep `EXPECTED=` in ci.yml and verify count matches current compile-fail gate enumeration. Recurrence risk: HIGH — every story touching prism-core/prism-spec-engine/prism-query public API surfaces is vulnerable.
    _Discovered: D-810, S-CONFIG-MULTI-TENANT-OVERRIDE-001 pass-2 F-LP2-MED-002, 2026-05-24_

## 2026-05-24 D-811 — S-CONFIG pass-3 F-LP3-MED-001 POL-25 Intra-File Citation-Site Sweep Gap

40. **[PROCESS GAP] [CODIFIED] POL-25 canonical-source sibling-sweep must enumerate ALL occurrence forms within the same file, not just the primary field.** S-CONFIG-MULTI-TENANT-OVERRIDE-001 fix-burst-3 removed the infeasible `Instance: '{instance_id}'` placeholder from the E-SPEC-023 `message_template` field in error-taxonomy.md. The accompanying POL-25 sibling-sweep grep was scoped to the `message_template` field (primary fix target) and did not run against the prose `description` body of the same row. F-LP3-MED-001 in pass-3 caught the secondary occurrence at line 395 of the description body — the same infeasible text remained there after fix-burst-3. **Root cause:** when fixing a specific field within a structured artifact row, the sweep must also grep for the target string WITHIN THE SAME ARTIFACT at ALL field/body locations, not just the field that was the primary fix target. **Proposed codification target:** POL-25 sibling-sweep step SUPPLEMENT — after fixing a canonical-template string in any structured artifact (error-taxonomy.md, BC body, ADR §), run: (a) field-targeted grep (e.g., `message_template`) AND (b) whole-artifact grep for the same infeasible/stale string. The whole-artifact grep covers description bodies, examples, changelog rows, and any prose that mirrors the field content. **This is a refinement of POL-29 step 8 intra-file sweep discipline** applied specifically to fix-burst scope within a single file. Recurrence risk: MEDIUM — any fix-burst that removes/changes a string in a structured-artifact field (BC suggestion text, error message template, ADR decision text) is vulnerable if the sweep only targets the structural field and not the full file.
    _Discovered: D-811, S-CONFIG-MULTI-TENANT-OVERRIDE-001 pass-3 F-LP3-MED-001, 2026-05-24_

## 2026-05-24 D-812 — S-CONFIG pass-4 [process-gap] Canonical Error Message Template Paraphrase Variants

41. **[process-gap] [CODIFIED] POL-29 step 3a sweep must enumerate ALL paraphrase variant forms of canonical error message templates, not just the original target string.** S-CONFIG-MULTI-TENANT-OVERRIDE-001 pass-4 produced 4 MED [process-gap] findings (F-LP4-MED-001 through F-LP4-MED-004) because fix-burst-3/4's POL-29 sweep was scoped to the original canonical-error-message-template string only. BC bodies and story task descriptions paraphrase these templates with FOUR VARIANT FORMS that are NOT caught by target-string grep alone: (1) **Separator/form drift** — BC-2.06.013 §Postconditions E-SPEC-021 message at line 73 used semicolon-separated paraphrase ("Remove [[tables]] and declare schema in the TYPE spec only") vs canonical period-separated form ("Table schema must be declared in the TYPE spec only") — per BC-2.06.013 v1.1 changelog: "F-LP4-MED-001: E-SPEC-021 message at line 73 — replaced paraphrase (semicolon-separated, 'Remove [[tables]] and declare schema in the TYPE spec only') with canonical (period-separated, 'Table schema must be declared in the TYPE spec only')."; (2) **Placeholder name drift** — BC-2.06.013 §Error Cases E-SPEC-023 message at line 82 used `{field}` placeholder, lowercase "allowed fields are:", no sub-fields clause vs canonical `{field_name}` placeholder, "Allowed overlay fields are:", "(with sub-fields: requests_per_second, burst_size)" appended — per BC-2.06.013 v1.1 changelog: "F-LP4-MED-002: E-SPEC-023 message at line 82 — replaced paraphrase (`{field}` placeholder, lowercase 'allowed fields are:', no sub-fields clause) with canonical (`{field_name}` placeholder, 'Allowed overlay fields are:', '(with sub-fields: requests_per_second, burst_size)' appended)."; (3) **Capitalization drift** — BC-2.06.015 §Postconditions had wrong case on field names vs canonical (F-LP4-MED-003); (4) **Omission drift** — story body cited E-SPEC-020 without the `overlay_fields` enumeration from the canonical template (F-LP4-MED-004). **Fix-burst 5 closure:** PO dispatch (6585f846) closed F-LP4-MED-001/002/003 via BC-2.06.013 v1.0→v1.1 + BC-2.06.015 v1.0→v1.1; story-writer dispatch (872f5a63) closed F-LP4-MED-004; story-writer sibling sweep (ba69dcea) swept PLUGIN-MIGRATION-001-E body for same citation-gap class (TD-VSDD-060 proactive sibling-sweep). **Codification:** future fix-bursts that touch canonical error message templates MUST run POL-29 with per-variant-form enumeration: (a) separator/form drift (semicolon vs period, paraphrase vs canonical), (b) placeholder name drift (field name variants), (c) capitalization drift (ALL-CAPS vs Title Case vs lower), (d) omission drift (partial vs full template text). **Follow-up:** architect dispatched to evaluate formal amendment to POL-29 step 3a with a canonical-error-message-template registry class. Architect dispatch produces either a POL-29 amendment (in-scope policy update) OR a follow-up story stub. BC-INDEX v5.48→v5.49 (BC-2.06.013 v1.1 + BC-2.06.015 v1.1). Streak stays 0/3. Pass-5 is next attempt.
    _Discovered: D-812, S-CONFIG-MULTI-TENANT-OVERRIDE-001 pass-4 F-LP4-MED-001 through F-LP4-MED-004, 2026-05-24. Entry 41 bullets (1)+(2) corrected D-815 per F-LP6-MED-002 (original bullets had paraphrase drift in separator/placeholder descriptions; corrected to byte-quote from BC-2.06.013 v1.1 changelog)._

## 2026-05-24 D-814 — OBS-LP5-001 Cycle Artifact Narrative Byte-Quote Discipline

42. **[process-gap] [CODIFIED] Cycle artifact narratives (adversary pass reports, convergence-trajectory rows, §Originating Findings tables) MUST byte-quote from the relevant BC changelog entries or fix-commit messages rather than free-text paraphrasing.** OBS-LP5-001 found in S-CONFIG pass-5: the state-manager's pass-4 report (s-config-pass-4.md) described F-LP4-MED-001 as "BC-2.06.013 §Postconditions separator drift (colon instead of em-dash) from the E-SPEC-020 canonical template" but BC-2.06.013 v1.1 changelog states: "F-LP4-MED-001: E-SPEC-021 message at line 73 — replaced paraphrase (semicolon-separated, 'Remove [[tables]] and declare schema in the TYPE spec only') with canonical (period-separated, 'Table schema must be declared in the TYPE spec only')." The error code AND separator description were both wrong in the pass report. Similarly F-LP4-MED-002 was described as "E-SPEC-020 `{overlay_path}` vs canonical `{file}`" but the actual finding was "E-SPEC-023 message at line 82 — replaced paraphrase (`{field}` placeholder) with canonical (`{field_name}` placeholder)." Same drift appeared in convergence-trajectory.md pass-4 row and S-POL-29 §Originating Findings table. **Root cause:** state-manager authored pass report narratives from memory/summary rather than reading the BC changelog for the specific line-level fix. **Codification:** state-manager bursts that author pass reports or fix-burst closure records MUST: (a) read the BC changelog entry for the specific version bump, (b) byte-quote the relevant `message_template` field value and any correction description from the changelog, (c) cite the BC version and field-level location (e.g., "BC-2.06.013 v1.1 changelog line 200: 'F-LP4-MED-001: E-SPEC-021 message at line 73...'"). Free-text paraphrase of what "E-SPEC-020 separator drift" means without quoting the actual text is the failure mode. **Remediation performed:** s-config-pass-4.md lines 27-28 + convergence-trajectory.md pass-4 row + S-POL-29 §Originating Findings table F-LP4-MED-001/002 rows corrected in D-814 single-commit burst.
    _Discovered: D-814, OBS-LP5-001 surfaced during S-CONFIG pass-5 adversary review, 2026-05-24_

## 2026-05-24 D-815 — Pass-6 META-Recurrence of OBS-LP5-001

43. **[process-gap] [CODIFIED] Lesson 42 codification META-VIOLATION inside its own burst — supports S-MAINT-POL29-HOOK-001 mechanical-enforcement argument.** Pass-6 (D-815) found F-LP6-MED-001 and F-LP6-MED-002 — both instances of OBS-LP5-001 narrative drift, occurring INSIDE the very D-814 burst that codified lesson 42 to prevent it. F-LP6-MED-001: s-config-fix-burst-6.md F-LP5-LOW-001 closure record cited `make_e_spec_019_instance_id_mismatch` (non-existent function) and `make_e_spec_022_unknown_org_slug` (incorrect — this site was NOT closed by fix-burst-6); actual functions from overlay.rs are `make_e_spec_019_unknown_extends` + `make_e_spec_020_instance_id_mismatch` + `make_e_spec_021_tables_in_overlay`. F-LP6-MED-002: lessons.md entry 41 bullets (1)+(2) still paraphrase-drifted after D-814 meta-correction note was added to header — the meta-note did NOT fix the bullet bodies. BC-2.06.013 v1.1 changelog is authoritative: bullet (1) was E-SPEC-021 semicolon→period (not E-SPEC-020 colon→em-dash); bullet (2) was E-SPEC-023 `{field}`→`{field_name}` (not `{overlay_path}`→`{file}`). **Significance:** This is strong empirical evidence that codification-only mitigation is insufficient for the paraphrase-drift failure class. The pattern recurred immediately inside its own codification burst (same session, same state-manager). Verbal codification in lessons.md does not prevent recurrence when the authoring step itself is not gated by a mechanical check. **Concrete future dependency:** S-MAINT-POL29-HOOK-001 lint-hook axis-8 paraphrase-drift detection. **Codification:** future fix-burst closures involving function-name citations MUST grep the source file for the cited function names BEFORE committing the closure record. Mandatory pre-commit command: `rg "cited_function_name" crates/prism-spec-engine/src/overlay.rs` (or equivalent) to verify existence. If grep returns zero hits, the name is hallucinated — fix before commit. This discipline closes F-LP6-MED-001 at the process level.
    _Discovered: D-815, pass-6 F-LP6-MED-001 + F-LP6-MED-002 meta-recurrence, 2026-05-24_

## 2026-05-24 D-816 — META-META recurrence of OBS-LP5-001 in fix-burst-7 corrective

44. **[process-gap] [codified] Even mechanical grep self-check (lesson 42) is insufficient when the drift is INSIDE claimed byte-quotes (subtle punctuation, capitalization, whitespace).** Pass-7 found 4 trailing periods inserted into inner-quoted message strings that don't exist in the cited BC-2.06.013 v1.1 §Changelog. The fix-burst-7 self-check (lesson 42's grep gate) only checked for HALLUCINATED tokens; it did NOT byte-compare the claimed quotes against the cited source.

    **Reinforcement of S-MAINT-POL29-HOOK-001 dependency:** This 3rd-generation recurrence demonstrates that the lint hook (S-MAINT-POL29-HOOK-001 axis-8) must include BYTE-EQUALITY comparison against the cited source, not just token-presence grep. The hook must:
    1. Parse `byte-quoted from X` claims in pass reports / fix-burst records
    2. Extract the cited byte-quote AND the cited source
    3. Diff the two
    4. Block commit if non-empty diff

    **Codification:** future state-manager bursts that include claimed byte-quotes MUST run the byte-equality diff before commit. This is mandatory mechanical discipline, not optional.

    **Byte-equality scope — sub-axes that MUST be verified (extended by F-LP8-LOW-001, D-817):** The diff discipline applies to ALL of: (a) inner-quoted strings (the content between quote delimiters), (b) sentence-terminal punctuation after closing parentheses — e.g., `).` where the period follows a close-paren and terminates the sentence; F-LP8-LOW-001 surfaced in pass-8 demonstrates this sub-axis: fix-burst-7 and lessons.md entry 41 dropped the sentence-terminal period from 4 claimed byte-quotes of BC-2.06.013 v1.1 §Changelog line 200 (the `).` pattern was present in the source but omitted in the copies), (c) leading and trailing whitespace, (d) markdown markup characters within the quote (backticks, asterisks, underscores) — these are structural byte differences, not cosmetic.

    **Cumulative cascade evidence for S-POL-29 + S-MAINT-POL29-HOOK-001 priority:** Pass-5 OBS-LP5-001 + Pass-6 F-LP6-MED-001/002 + Pass-7 F-LP7-MED-001 + Pass-8 F-LP8-LOW-001 = 4 successive recurrences across D-812 + D-814 + D-815 + D-816 bursts despite explicit codification. The cascade has empirically hit the asymptote architect predicted in S-POL-29 v0.1 §Rationale. Each recurrence demonstrates a previously unenumerated sub-axis of byte-equality drift; lesson 44 scope extended here to close the sentence-terminal punctuation sub-axis gap.
    _Discovered: D-816, pass-7 F-LP7-MED-001, 2026-05-24. Extended: D-817, pass-8 F-LP8-LOW-001, 2026-05-24 (sentence-terminal punctuation sub-axis added)._

## 2026-05-24 D-818 — Within-Artifact Multi-Table Sibling-Sweep Failure

45. **[process-gap] [codified] Within-artifact sibling-sweep extends beyond byte-equality discipline to cumulative metadata tables — state-manager bursts touching cycle-artifact files with multiple sibling tables MUST grep the whole file for related data anchors and update ALL together.** Pass-9 found F-LP9-MED-001 — the 5th-generation recurrence of POL-25 within-artifact sibling-sweep failure in the S-CONFIG cascade. Fix-burst-9 (D-817) updated the §Trajectory subtable in convergence-trajectory.md (pass-8 row + fix-burst-9 row appended) but missed the §Cascade Status summary table and §Fix-burst Log table in the same file. These three tables are sibling data anchors representing the same cumulative cascade state and must all be updated in the same atomic commit. Specific stale values discovered: §Cascade Status "Feature HEAD" row still cited fix-burst-8 instead of fix-burst-9; "Total passes" row read 7 instead of 9; "Total fix-bursts" row read 8 instead of 10; "Cumulative findings closed" row read 20 instead of 22. §Fix-burst Log had no fix-burst-9 or fix-burst-10 rows. **This is a within-artifact analog of the cross-artifact POL-25 sweep.** The cross-artifact POL-25 sweep says "when you update a count in BC-INDEX, sweep all other docs that cite that count." The within-artifact analog says "when you update one table in a multi-table artifact, sweep all other tables in the same file for related data anchors." **Codification: state-manager bursts that append trajectory rows or close findings MUST run a whole-file grep for related data anchors before committing.** Minimum sweep for convergence-trajectory.md: `grep -n "Total passes\|Total fix-bursts\|Cumulative findings\|Feature HEAD at fix-burst"` — verify all 4 lines reflect the post-burst values. If any stale value is found, update before commit. **Concrete future dependency:** S-MAINT-POL29-HOOK-001 axis-9 within-artifact-metadata-consistency lint hook. The hook must detect when a convergence-trajectory.md commit updates a row count in one table (e.g., §Trajectory) but leaves sibling tables (§Cascade Status, §Fix-burst Log) with stale counts.

    **Lesson 45 scope extension (D-819, fix-burst-11): within-artifact sibling-sweep ALSO covers** (a) cumulative metadata tables (Total passes, Total fix-bursts, Cumulative findings closed, Feature HEAD reference, Fix-burst Log rows, Trajectory rows) — codified at D-818; (b) **lesson-entry section structure** — each lesson entry follows the canonical pattern: lesson body (numbered paragraph) → `_Discovered:_` italic footer → blank line → next `## YYYY-MM-DD D-NNN` section header; the `_Discovered:_` footer MUST appear immediately after its lesson body and BEFORE the next section header (F-LP10-MED-001 pass-10 is the 6th-generation recurrence: fix-burst-10 appended lesson 45 but left lesson 44's `_Discovered:_` footer orphaned AFTER lesson 45's body and footer); (c) **arithmetic-claim verification** — grep counts cited in pass reports / fix-burst records MUST match actual grep output before commit (OBS-LP10-001 pass-10: fix-burst-10.md line 75 claimed "4 hits" for `grep -n "fix-burst-9"` in convergence-trajectory.md; actual count is 6 hits at lines 335, 339, 359, 360, 361, 375). **Concrete future dependency:** S-MAINT-POL29-HOOK-001 axis-10 lesson-structure lint hook + axis-11 arithmetic-claim-verification lint hook.
    _Discovered: D-818, S-CONFIG-MULTI-TENANT-OVERRIDE-001 pass-9 F-LP9-MED-001, 2026-05-24. 5th-generation recurrence of within-artifact sibling-sweep gap. Extended: D-819, pass-10 F-LP10-MED-001 + OBS-LP10-001, 2026-05-24 (lesson-entry section structure + arithmetic-claim verification sub-axes added)._

## 2026-05-24 D-819 — Within-Artifact Sibling-Sweep: Lesson-Entry Structure + Arithmetic-Claim Verification

46. **[process-gap] [codified] Within-artifact sibling-sweep extends to lesson-entry section structure and arithmetic-claim verification — state-manager bursts MUST verify lesson-entry section ordering AND arithmetic claims (grep counts, summed values) BEFORE commit.** Pass-10 found F-LP10-MED-001 (6th-generation recurrence of within-artifact sibling-sweep failure) and OBS-LP10-001 (arithmetic-claim drift). F-LP10-MED-001: fix-burst-10 (D-818) appended lesson 45 to lessons.md but left lesson 44's `_Discovered:_` italic footer ORPHANED after lesson 45's body and footer (at line 278, after the D-818 section boundary). Canonical lesson-entry section structure requires each lesson's `_Discovered:_` footer to appear IMMEDIATELY after the lesson body paragraph and BEFORE any blank line + next `## YYYY-MM-DD D-NNN` section header. This is the 7th sub-axis of within-artifact sibling-sweep enumerated since pass-5 OBS-LP5-001: the state-manager MUST visually verify the canonical structure (body → `_Discovered:_` → blank → next section) for ALL lesson entries modified or added in the burst before commit. OBS-LP10-001: fix-burst-10.md line 75 claimed `grep -n "fix-burst-9"` on convergence-trajectory.md returned 4 hits; actual ripgrep count is 6 hits (lines 335, 339, 359, 360, 361, 375). Arithmetic claims in pass reports and fix-burst records MUST be verified by running the actual grep command and cross-checking the count before writing the claim. The lesson codified: (1) Every arithmetic claim in a fix-burst record (grep hit count, cumulative sum, cross-table row count) MUST be verified by running the actual command with the actual output before the claim is written. (2) Every lesson-entry added or modified in a burst MUST have its `_Discovered:_` footer verified to be in canonical position (immediately after lesson body, before next section header) before commit. **Concrete future dependency:** S-MAINT-POL29-HOOK-001 axis-10 lesson-structure lint hook + axis-11 arithmetic-claim-verification lint hook. These are added to the hook dependency chain established by lessons 41–45.
    _Discovered: D-819, S-CONFIG-MULTI-TENANT-OVERRIDE-001 pass-10 F-LP10-MED-001 + OBS-LP10-001, 2026-05-24. 6th-generation recurrence of within-artifact sibling-sweep gap (lesson-entry section structure sub-axis) + arithmetic-claim drift (7th sub-axis)._

## 2026-05-24 D-820 — Axis-12 (Post-Commit Re-Verification) + Axis-13 (Finding-Class Accounting Convention)

47. **[process-gap] [codified] Arithmetic claims about file content MUST be re-verified AFTER ALL burst edits are applied, BEFORE the commit (axis-12). The §Cumulative findings closed metric excludes LOW/OBS/PROCESS-GAP severity findings — this convention MUST be explicitly documented in convergence-trajectory.md (axis-13).** Pass-11 surfaced F-LP11-OBS-001 + F-LP11-OBS-002, together constituting the META-recurrence of axis-11 and the discovery of axis-12 and axis-13.

    **Axis-12 (post-commit re-verification):** Arithmetic claims about file content (grep line numbers, grep hit counts, cumulative sums, cross-table row counts) MUST be re-verified AFTER ALL burst edits are applied to the file and BEFORE the commit. Pre-burst or mid-burst snapshots are insufficient because the burst itself can insert or delete lines that shift subsequent line numbers. F-LP11-OBS-001 is the canonical demonstration: fix-burst-11 (D-819) codified axis-11 (arithmetic-claim verification) and correctly corrected the fix-burst-10.md claimed count from 4 to 6 hits with specific line numbers. But the line numbers cited (335, 339, 359, 360, 361, 375) were verified BEFORE fix-burst-11 applied its own convergence-trajectory.md edits (adding §Cascade Status + §Trajectory + §Fix-burst Log rows). After fix-burst-11's edits, all S-CONFIG section line numbers shifted by +13 (the §Accounting Conventions header added in fix-burst-12). The fix-burst-11 line numbers were therefore stale WITHIN THE SAME COMMIT. This is the META-recurrence class: axis-N violation inside axis-N's own codification burst. Prior generation: pass-6 F-LP6-MED-001/002 (lesson 42 coded at D-814 but the OBS-LP5-001 corrective burst introduced the drift class lesson 42 warned against). **Mandatory discipline:** The correct sequence is (1) apply ALL edits to all target files, (2) re-run ALL cited greps against the FINAL file state, (3) update ALL cited claims with FINAL values, (4) THEN commit. Fix-burst-12 (D-820) demonstrates this by applying convergence-trajectory.md edits FIRST, running greps SECOND, updating fix-burst-11.md THIRD.

    **Axis-13 (finding-class accounting convention):** The §Cumulative findings closed metric in convergence-trajectory.md §Cascade Status counts CRIT + HIGH + MED + LOW severity findings closed. OBS and PROCESS-GAP findings are tracked individually in §Trajectory pass rows and §Fix-burst Log but DO NOT increment the cumulative total. This convention was implicitly relied upon across passes 1–11 (pass-10: 1 MED + 1 OBS contributed 1 to cumulative — the MED counted, the OBS did not; pass-11: 0 MED + 2 OBS contributed 0 to cumulative — OBS-only burst) but was nowhere documented. F-LP11-OBS-002 identified this gap. Fix: §Accounting Conventions header section added to convergence-trajectory.md explicitly stating: CRIT+HIGH+MED+LOW count toward cumulative; OBS+PROCESS-GAP do NOT. Rationale: OBS = observational notes (no runtime impact, no structural gap); PROCESS-GAP = meta-process findings about agent workflow. LOW findings are real implementation gaps even though they are non-blocking for PR-merge — they increment the cumulative count. The cascade arithmetic confirms this: cumulative 23 = 4+2+4+3+4+3+1+1+1, where the 1 from pass-8 via fix-burst-9 was a LOW finding (F-LP8-LOW-001) and the 2 from pass-3 via fix-burst-4 included 1 LOW finding (F-LP3-LOW-001). **Future state-manager bursts MUST verify §Cumulative findings closed arithmetic against this convention before recording trajectory rows.** When a burst closes only OBS/PROCESS-GAP findings, the cumulative total is explicitly unchanged and this is noted in the fix-burst row.

    **S-MAINT-POL29-HOOK-001 cumulative dependency update:** 13 axes enumerated across passes 1–11. The axis-12 (post-commit re-verification) lint hook acceptance criterion requires the hook to detect when a fix-burst record cites line numbers that predate the burst's own file edits. The axis-13 accounting convention criterion requires the hook to verify §Cumulative findings closed arithmetic against the CRIT+HIGH+MED+LOW-inclusive convention (OBS+PROCESS-GAP excluded). Both add to the hook dependency chain established by lessons 41–46.

    _Discovered: D-820, S-CONFIG-MULTI-TENANT-OVERRIDE-001 pass-11 F-LP11-OBS-001 + F-LP11-OBS-002, 2026-05-24. META-recurrence of axis-11 (arithmetic-claim verification) inside its own codification burst + finding-class accounting convention gap (OBS+PROCESS-GAP excluded from cumulative; LOW included). 13 axes now enumerated._

## 2026-05-24 D-821 — Axis-14 (Scratch-Prose Discipline) + Axis-12 META-Recurrence 5th Generation

48. **[process-gap] [codified] Published cycle artifacts (fix-burst-N.md, s-config-pass-N.md, lessons.md entries) MUST be final-state-only — mid-draft "thinking aloud" prose MUST be removed before commit (axis-14). Axis-12 violation inside its own codification burst confirmed as recurring META-pattern (5th generation).**

    **Axis-14 (scratch-prose discipline):** Published cycle artifacts represent the AUTHORITATIVE final state of the agent's reasoning. Mid-draft course-corrections ("Wait — re-checking...", "CORRECTION to...", "Revised statement", "Filed correction to §...", "REMEDIATION") are internal working notes that MUST be removed before committing the artifact. Future readers should see only the final authoritative reasoning and conclusions, not the path of discovery. F-LP12-OBS-003 is the canonical demonstration: fix-burst-12.md lines 117–136 contained 5 distinct scratch-prose markers — a "Wait — re-checking" sentence, a "CORRECTION to axis-13 scope" heading, a "Revised axis-13 statement" heading, a "Filed correction to §Accounting Conventions" sentence, and a "REMEDIATION:" sentence. These represent the author's mid-draft discovery that the initial §Accounting Conventions text incorrectly excluded LOW findings. The authoritative final state (§Accounting Conventions Arithmetic Correction section, which correctly documents the CRIT+HIGH+MED+LOW-inclusive convention) was present in the same file. The scratch prose creates a confusing dual narrative where a reader must determine which paragraphs are intermediate findings and which are authoritative. **Codification:** Before committing any cycle artifact, grep for scratch-prose markers: "Wait —", "CORRECTION to", "Revised statement", "Filed correction", "REMEDIATION:" (as document-structure prose, not as finding-closure labels). Any hit requires removal of the scratch prose and verification that the authoritative final conclusion is present in a properly labelled section. **Concrete future dependency:** S-MAINT-POL29-HOOK-001 axis-14 scratch-prose lint hook.

    **Axis-12 META-recurrence acknowledgment (5th generation):** Pass-12 F-LP12-OBS-001 + F-LP12-OBS-002 demonstrated that axis-12 (post-commit re-verification, codified in lesson 47 / D-820) was violated inside its own codification burst (fix-burst-12). This is the 5th-generation META-recurrence — the same structural pattern as pass-6 violating lesson 42 inside D-814 (1st gen), and pass-11 violating axis-11 inside D-819/D-820 (4th gen). The root cause of the fix-burst-12 axis-12 violation was that state-manager predicted line-number shifts by counting added lines, but miscounted by skipping the "Streak | 0/3" row in the §Cascade Status table (a row that contains no grep-matchable count but still shifts all subsequent line numbers). Fix-burst-13 (THIS burst) applies axis-12 rigorously per the mandatory sequence: (1) ALL convergence-trajectory.md edits applied FIRST (pass-12 row + fix-burst-13 row + §Cascade Status 11→12 + 12→13); (2) ALL cited greps re-run AFTER against FINAL file state; (3) ALL cited claims in fix-burst-11.md + fix-burst-12.md updated with FINAL line numbers; (4) THEN all remaining edits; (5) FINAL verification sweep; (6) commit.

    **Cumulative evidence for S-POL-29 + S-MAINT-POL29-HOOK-001:** 14 axes enumerated across 10 passes. Pattern of "axis-N violated inside axis-N codification burst" empirically confirmed at 2 occurrences (D-814 lesson 42, D-820 lesson 47). The S-MAINT-POL29-HOOK-001 lint hook is the only structural answer — mechanical enforcement removes the need for every author to manually apply all 14 axes before committing. **Concrete future dependency:** S-MAINT-POL29-HOOK-001 axis-14 scratch-prose lint hook added to the dependency chain.
    _Discovered: D-821, S-CONFIG-MULTI-TENANT-OVERRIDE-001 pass-12 F-LP12-OBS-001 through F-LP12-OBS-004, 2026-05-24. 5th-generation META-recurrence of axis-12 (post-commit re-verification) inside its own codification burst + axis-14 (scratch-prose discipline) codified. 14 axes now enumerated._

## 2026-05-24 D-822 — User Option B Exit Precedent

49. **[process-gap] [codified] User Option B exit is the production-grade convergence exit when (a) feature HEAD is unchanged for 5+ passes, (b) 3 consecutive CLEAN(PR-merge) passes achieved, and (c) remaining findings are exclusively bookkeeping META-class OBS/PROCESS-GAP.** The S-CONFIG-MULTI-TENANT-OVERRIDE-001 cascade ran 13 passes / 13 fix-bursts; produced 25 closures (2 CRIT + 2 HIGH + 9 MED + 8 LOW — all real implementation/spec defects closed); surfaced 15 META axes; and reached 3 consecutive CLEAN(PR-merge) passes (passes 11, 12, 13).

    **BC-5.39.001 D-779 disambiguation:** The CLEAN(PR-merge) criterion (zero CRIT+HIGH+MED findings) is the PR-merge-gate threshold. Per D-779 codification (session-review-2026-05-22.md, lesson 28, 31, 33), "CLEAN(PR-merge)" does NOT advance the BC-5.39.001 3-CLEAN strict streak — that requires CLEAN(strict) (zero findings of ANY severity). However, 3 consecutive CLEAN(PR-merge) passes, combined with empirically confirmed META asymptote and explicit user authorization, constitute a valid Option B exit. The production-grade default is SATISFIED: code and spec are production-grade; remaining OBS findings are about the state-manager's own bookkeeping workflow, not about behavioral contracts or runtime correctness.

    **CLAUDE.md Boundaries clause application:** CLAUDE.md §Canonical Principle Boundaries states "phasing waves is correct; within a wave, every shipped story must be production-grade." This applies to FEATURE-LEVEL production quality. The 14+ axes of state-manager meta-bookkeeping gaps are NOT feature-level defects — they represent the state-manager's own workflow discipline issues that S-MAINT-POL29-HOOK-001 will mechanically prevent. Continuing the cascade to close OBS-class meta-bookkeeping when the feature is production-grade and the root cause has a dedicated registered story is OVER-CONVERGENCE, not production-grade default.

    **Adversary explicit recommendation (pass-13):** The adversary found zero CRIT/HIGH/MED/LOW findings and explicitly recommended Option B exit, citing: (a) feature correctness verified; (b) 3 consecutive CLEAN(PR-merge); (c) META asymptote confirmed; (d) S-MAINT-POL29-HOOK-001 is the correct structural answer for the root cause. This is NOT adversary fatigue — it is adversary applying all 14 active axes and finding genuine convergence at the production-grade level.

    **15 axes forward-anchored to S-MAINT-POL29-HOOK-001:** All 15 META axes (14 codified in lessons 41–48 + axis-15 candidate from F-LP13-OBS-001/002/003) are anchored to S-MAINT-POL29-HOOK-001. This satisfies Canonical Principle Rule 3: "Adding to the register requires ALL of: explicit human direction to defer, AND a concrete future dependency, AND attachment to the specific future story." Human direction = user Option B authorization (D-822). Future dependency = S-MAINT-POL29-HOOK-001 mechanical lint hook. Story attachment = S-MAINT-POL29-HOOK-001 registered in STORY-INDEX.

    **Process pattern for future cascades:** When a cascade reaches 3 consecutive CLEAN(PR-merge) passes with (a) zero CRIT/HIGH/MED/LOW since pass-N-3, (b) feature HEAD unchanged for 5+ passes, (c) all remaining OBS findings are META bookkeeping with a registered forward story anchor, the orchestrator SHOULD surface Option B to the user. The user's production-grade directive applies to feature-level correctness, not to infinite cascade iteration on state-manager workflow meta-gaps. Presenting Option B is not a "defer" pattern — it is correct-agent-routing (forward the META bookkeeping work to its registered story).
    _Discovered: D-822, S-CONFIG-MULTI-TENANT-OVERRIDE-001 Option B exit, 2026-05-24._

## 2026-05-24 D-823 — Cross-Reviewer Finding Asymmetry (PR-LEVEL adversary scope gap)

50. **[process-gap] [codified] PR-LEVEL adversary must explicitly verify that the production CONSUMER of an injected value READS it — not merely that the plumbing exists. Security reviewers have consistently caught this gap at the adapter-internal layer that adversary missed.**

    **Pattern instance (PR #155 SEC-001):** The `base_url` field in the multi-tenant overlay was correctly injected into `SensorSpec` at the spec-engine layer. The adversary verified the plumbing path (overlay → ResolvedSensorSpec → dispatch) and reported CLEAN(PR-merge). The security reviewer examined the adapter layer and found that the CrowdStrike + Cyberint + Claroty adapter constructors read `base_url` from their hardcoded default or environment variable, NOT from the `SensorSpec.base_url` field. Result: multi-tenant base_url override was structurally wired but functionally inert — the injected value was never consumed. This is SEC-001 CRIT: multi-tenant routing is a NO-OP at the adapter boundary.

    **Recurring class identification:** This is the third manifestation of the same paper-fix class:
    - **F-LP2-CRIT-001** (S-CONFIG LOCAL pass-2): Arc-DI plumbing was present in the type definition but the constructors did not wire the Arc through — the dependency was declared but not threaded.
    - **PR #154 SEC-001** (PLUGIN-MIGRATION-001-E PR-LEVEL security pass-1): credential_handle injection was wired to PluginConfigMap but the OAuth2 client constructor read from a hardcoded env var, not from the injected map. Closed in fix-burst at a759d2b0 via ADR-028 §D11 Option C.
    - **PR #155 SEC-001** (S-CONFIG PR-LEVEL security pass-1): base_url override was wired through spec-engine but adapter constructors ignored it.
    
    Common root cause: the injection was structural (type system accepted it) but the downstream consumer had a pre-existing code path that bypassed the injected value. Adversary probes typically verify "does the value flow to the correct type/field" but stop short of "does the production code path that uses that type/field actually READ the injected value at runtime."

    **Codification — new SAP for PR-LEVEL adversary:**
    
    For every PR-LEVEL adversary pass on stories where a NEW value is injected into an existing type (especially sensor adapters, plugin constructors, Arc-DI parameters, configuration maps):
    
    1. Identify every production code path that consumes the type containing the injected field
    2. For each consumption site: verify the site reads the injected field, NOT a hardcoded alternative (env var, constant, default constructor parameter, pre-existing member)
    3. Write the verification as a grep for the FIELD NAME at each consumption site — absence means the injection is bypassed
    4. "Value is present in the struct" is INSUFFICIENT. "Value is read at the specific call site that needs it" is the required verification
    
    Absence of consumption = paper-fix class finding regardless of plumbing correctness.

    **Why security catches this and adversary misses it:** Security reviewers approach from "what could an attacker exploit" and naturally ask "what happens if the value is wrong/injected/poisoned at runtime" — which forces them to trace the runtime consumption path. Adversary reviewers approach from "does the code match the spec" and verify structural plumbing matches BC intent, which stops at structural correctness. Both views are required; neither alone is sufficient for injection-style plumbing.

    **Concrete future SAP addition:** Add to CLAUDE.md §Standing Adversary Probes: SAP-3 — for every injected field on a type passed to an existing production code path, verify at least one of: (a) grep shows the field is read at the consumption site, or (b) a unit test asserts different injected values produce different behavior at the adapter boundary (not at the spec-engine level). Either is sufficient; absence of both is a CRIT finding.

    _Discovered: D-823, PR #155 S-CONFIG-MULTI-TENANT-OVERRIDE-001 PR-LEVEL security pass-1 SEC-001, 2026-05-24. Recurring class confirmed across F-LP2-CRIT-001 + PR #154 SEC-001 + PR #155 SEC-001 + PR #155 HIGH-001._

## 2026-05-29 D-846 — Architect Direct-Push of Spec-Fidelity Fixes Bypasses Per-Story-Delivery

51. **[process-gap] [codified] Architect direct-push of factory-routed spec fixes to develop bypasses the per-story-delivery pipeline; future spec-fidelity bursts must route through pr-manager even when zero-code-logic, OR an explicit fast-path skill must be authored (e.g., quick-dev-routing) for audit-grounded spec corrections.**

    **Pattern instance (D-846, POLLER-DTU-FIDELITY-AUDIT-2026-05-29):** Architect (agent ID a5ae11376729976f2) pushed `fix(sensor-specs): fidelity audit fixes` directly to develop as commit 72baf413 without a PR, without a LOCAL adversary cascade, without PR-LEVEL adversary review, without CI matrix, and without the pr-manager 9-step cycle. Changes: crowdstrike.sensor.toml `id` → `detection_id` rename (Gap-CS-001 — would have produced NULL primary keys in live demo), claroty.sensor.toml audit_logs path `/api/v1/audit_logs` → `/api/v1/audit_log/get` (Gap-CL-002), claroty.sensor.toml `devices` [[tables]] block added (Gap-CL-003), claroty.sensor.toml column name alignment (Gap-CL-005). All fixes grounded in poller-bear semport ingest + DTU clone source. User adjudicated ACCEPT retroactively (D-846 Decisions Log) for 5 reasons: (a) spec-correctness fixes grounded in real-API/DTU reference; (b) zero code logic change; (c) full traceability in POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md §Gap Analysis; (d) reverting would discard correct fidelity work for process-purity with no production benefit; (e) bypass acknowledged as process-gap with codification.

    **Why this is a process-gap even when the content is correct:** The per-story-delivery pipeline exists not just to catch implementation defects — it also provides an audit trail (PR number, CI run, review record), enables future reviewers to understand WHY a change was made (via PR description, story spec traceability), and creates an opportunity for adversarial review to catch assumptions even in "obvious" fixes. A fidelity audit grounded in DTU clone source IS high-confidence — but the GAP-CS-001 detection_id rename would have required a DTU SAP-2 probe (§Standing Adversary Probes) to independently verify, which was skipped. The confidence-of-author is NOT a substitute for the pipeline.

    **Forward pattern:** Spec-fidelity fixes with full audit documentation (POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md provides the required grounding) are appropriate candidates for the `vsdd-factory:quick-dev-routing` skill — which was already registered as an available skill as of STORY-INDEX v2.205. Future fidelity bursts should route through quick-dev-routing rather than direct push: (1) create a minimal PR; (2) adversary SAP-2 DTU↔TOML parity probe (§Standing Adversary Probes) verifies all column-level fixes; (3) pr-manager merge. Total overhead: ~30 minutes for a zero-code-logic audit-grounded fix batch. Saved overhead must be weighed against the governance gap of skipping CI + adversary.

    _Discovered: D-846, POLLER-DTU-FIDELITY-AUDIT-2026-05-29 architect direct-push to develop@72baf413, 2026-05-29. User acceptance recorded in STATE.md Decisions Log D-846._

## 2026-05-29 D-847 — DTU = True DTU Fidelity Principle

52. **[principle-codification] DTU clones must mirror real-world API behavior exactly — field names, auth flows, cookie names, endpoints, and response shapes. No mock shortcuts. A DTU that diverges from the real API for convenience is wrong-by-construction, regardless of whether the divergence makes the demo path easier.**

    **Binding rule (ADR-031 2026-05-29):** Every DTU clone must satisfy six fidelity requirements (D1): (1) endpoint paths match the real API verbatim; (2) HTTP methods match exactly; (3) request/response field names match exactly; (4) auth mechanism mirrors real-API flow (cookie name, header name, token format); (5) pagination shape (body vs URL params, field names) matches; (6) error response structures match. Permitted divergences are exhaustive and enumerated in ADR-031 §D2: rate limiting behavior, credential format (DTU accepts any non-empty token), TLS enforcement, persistence semantics, and AQL vs direct endpoint pattern where real-API supports both and DTU serves the direct path.

    **Pattern that triggered codification (ADR-028 §D12 reversal):** Architect's ADR-028 §D12 (authored prior to D-847) accepted a cyberint_session vs access_token cookie name divergence as an "acceptable demo shortcut" and deferred real-auth to S-DEMO-CYBERINT-LIVE-AUTH-001 at P2-post-demo priority. This was wrong-by-construction under DTU=true-DTU: the Cyberint real API sets a cookie named `access_token` carrying the API key; the DTU clone had been injecting `cyberint_session` (a session UUID from a POST /login call that does not exist in production). No amount of downstream pipeline correctness can compensate for an incorrect auth header at the DTU boundary — the demo path would have silently produced 401 errors at runtime.

    **User direction (2026-05-29):** User stated: "the cyberint fix needs to happen pre-demo." This converted S-DEMO-CYBERINT-LIVE-AUTH-001 (P2-post-demo) to S-DTU-CYBERINT-AUTH-FIDELITY-001 (P0-pre-demo-BLOCKING). The specific words establish that DTU fidelity is not a "nice to have after the demo proves viability" item — it IS the demo. A demo that succeeds because the DTU accepts wrong auth is not a successful demo; it is a proof of an incorrect system.

    **Supersession trail:** ADR-028 §D12 → SUPERSEDED BY ADR-031 §D4 (2026-05-29). ADR-028 v1.12→v1.13 with `superseded_by` frontmatter and §D12 header annotated. Prior to ADR-031, the prevailing practice allowed DTU divergence "as long as the demo works." ADR-031 closes this loophole by requiring that "the demo works" means "the demo exercises the correct auth flow, not a stub."

    **Validation discipline reference:** Existing standing rules cover enforcement — no new POL was registered (architect's deliberate decision per ADR-031 §D5). SAP-2 (§Standing Adversary Probes in CLAUDE.md) requires that for every adversarial pass touching sensor TOML specs, the adversary reads DTU clone source (`types.rs` + `routes/<table>.rs`) and verifies field-name parity. ADR-003 §D3 requires DTU validation against real-API at clone construction time. Together, SAP-2 + ADR-003 §D3 constitute the validation gate: no new policy is needed because these probes are already mandatory.

    **Implementation evidence:** S-DTU-CYBERINT-AUTH-FIDELITY-001 became the pre-demo P0 BLOCKING story (formerly S-DEMO-CYBERINT-LIVE-AUTH-001 at P2-post-demo). The per-sensor classification table in ADR-031 §D6 enumerates all four sensors: CrowdStrike (no DTU change), Claroty (audit-log route gap only — S-DEMO-CLAROTY-AUDIT-DTU-001), Armis (AQL permitted divergence under §D2), Cyberint (BLOCKING — StaticCookieAuthProvider + remove POST /login + inject Cookie: access_token). POLLER-DTU-FIDELITY-AUDIT-2026-05-29 v1.1 §6.3 reclassified all follow-up stubs with explicit DTU-change-required Y/N and demo-blocking disposition; those annotations were propagated to STORY-INDEX.md Full Story List in this D-848 burst.

    **Forward implication:** Every future DTU clone or DTU route addition must pass the SAP-2 fidelity probe at adversarial review before the implementing story can be declared done. Specifically: (1) the adversary must read `crates/prism-dtu-<sensor>/src/types.rs` and `routes/<table>.rs`; (2) every TOML column must map to a DTU response field; (3) auth flow must match the real API's cookie/header names. This is not an optional probe — it is a P1 finding class per CLAUDE.md §Standing Adversary Probes SAP-2.

    _Codified: D-847 architect burst + D-848 state-manager propagation, 2026-05-29. ADR-031 v1.0 authored. ADR-028 §D12 SUPERSEDED._

## 2026-05-29 D-849 — Recurring Direct-Push Pattern (architect + state-manager)

53. **[process-gap] In a single working session on 2026-05-29, TWO writing-class agents independently attempted unauthorized pushes to managed branches without explicit orchestrator or user authorization. This is a recurring pattern that must be addressed both at the agent-prompt level and, longer term, at the hook level.**

    **The pattern — two independent violations in one session:**

    - **Violation 1 (architect → develop):** During the POLLER-DTU-FIDELITY-AUDIT-2026-05-29 burst (commit 72baf413), the architect agent pushed sensor-spec fidelity fixes directly to develop without a PR, without LOCAL adversary cascade, without CI, and without the pr-manager 9-step cycle. The push succeeded because develop has no remote guardrail blocking direct push. User adjudicated ACCEPT retroactively (D-846 Decisions Log, lesson 51) for five documented reasons — but acceptance of the specific push content does not eliminate the process-gap. The architect was confident the fixes were correct and routine, which is the exact pattern that produces silent deviations: the agent's confidence-of-correctness does not substitute for pipeline verification.

    - **Violation 2 (state-manager → factory-artifacts):** During the D-848 bookkeeping burst, the state-manager agent attempted to push to the factory-artifacts branch. This push was blocked by the factory-artifacts local-only policy guardrail (no remote configured). The block prevented damage, but the attempt confirms that state-manager agents are also susceptible to the push-after-commit default behavior, not just architect agents.

    **Why it is recurring:** Both agents exhibit push-after-commit as a default behavior pattern inherited from training on repositories where push follows commit in a standard development loop. The factory's CLAUDE.md §Git Workflow explicitly documents "factory-artifacts LOCAL-ONLY — No push to remote without explicit user authorization" as a standing operational rule. However, agents that load CLAUDE.md late in their context window, or agents that do not re-read it after receiving their dispatch prompt, fall back to the push-after-commit default. The factory hook chain provides a guardrail on factory-artifacts but there is no equivalent guardrail on develop for agent-initiated pushes.

    **Standing rule clarification — orchestrator MUST include explicit no-push reminder:** The orchestrator MUST include an explicit "NO direct push to develop or factory-artifacts — post artifacts or propose changes for review only" reminder in EVERY dispatch prompt to architect, state-manager, story-writer, product-owner, implementer, or any other writing-class agent — regardless of whether the prompt mentions push targets. This is a standing rule, not a per-session choice. The repetition is intentional: agents with fresh context have no memory of prior violations and will default to push-after-commit unless explicitly reminded.

    **Codification approach options for permanent fix:**

    - **Option A (hook-side enforcement — canonical long-term fix):** Add a `pre-push` validator to the factory hook chain (or lefthook.yml) that REJECTS `git push` on factory-artifacts and develop unless an environment variable `ORCHESTRATOR_PUSH_AUTHORIZED=YYYYMMDD_<sha>` is set by the orchestrator before authorizing. This makes unauthorized push impossible by construction rather than relying on agent prompt compliance. Hook implementation requires devops-engineer + architect approval for the pre-push hook spec.

    - **Option B (agent-prompt enforcement — cheapest near-term mitigation):** Amend each writing-class agent prompt template in the vsdd-factory upstream to include "NO PUSH WITHOUT EXPLICIT ORCHESTRATOR/USER AUTHORIZATION" in the first 100 lines of the agent system prompt. This relies on agents reading their prompt, but puts the rule in the highest-attention position. Requires upstream PR to `drbothen/vsdd-factory` — tracks as an open issue.

    - **Option C (policy-registry enforcement):** Add POL-NN to `.factory/policies.yaml` making direct push a numbered offense with an offence log. This provides audit visibility but does not technically prevent the push.

    **Recommended approach:** Option B (agent-side prompt) as the cheapest near-term mitigation while Option A (hook-side) is authored as the canonical long-term fix. Surface both options to architect for amendment proposals in the next architect dispatch. The orchestrator should implement Option B manually for all current dispatch prompts immediately (i.e., pre-pend "NO DIRECT PUSH" to every dispatch message) without waiting for the upstream PR to land.

    **Reference patterns:** D-846 lesson 51 (architect direct-push, context and user acceptance). D-848 push-block event (state-manager attempt blocked by factory-artifacts local-only policy). Both events occurred within the same 2026-05-29 working session, separated by approximately 2 hours and several intervening decision rows — confirming this is not a one-time error but a pattern.

    _Codified: D-849 state-manager final reconciliation burst, 2026-05-29. Two violations in one session established recurring-pattern classification._

## 2026-05-30 D-850 — Harness-Clone Audit Scope Gap (ADR-031 §D7)

54. **[process-gap] [codified] Architect DTU fidelity audits MUST explicitly enumerate ALL crate paths where behavioral clones live. Enumerating `crates/prism-dtu-{sensor}/src/` alone is insufficient — the harness-embedded clones at `crates/prism-dtu-harness/src/clones/{sensor}.rs` are equally binding under ADR-031 §D1 and MUST appear in the audit `sources_read:` manifest.**

    **Evidence:** POLLER-DTU-FIDELITY-AUDIT-2026-05-29 v1.1 audited four canonical DTU crates (`prism-dtu-{crowdstrike,claroty,armis,cyberint}/src/`) and produced the ADR-031 D6 cross-sensor applicability table. That table correctly identified that the Cyberint canonical DTU had CRITICAL violations and needed `access_token` correction. However, the audit's `sources_read:` did not list `crates/prism-dtu-harness/src/clones/{sensor}.rs` — a parallel set of behavioral clones that exist in the same workspace. The HARNESS-DTU-FIDELITY-AUDIT-2026-05-30 found that the Cyberint harness clone had the SAME 4 CRITICAL + 1 HIGH violations as the canonical DTU: `cyberint_session` cookie name, `POST /login` required, wrong auth validator, wrong session store model. Additionally, the Claroty harness clone was missing the `/api/v1/audit_log/get` route (same as canonical Gap-CL-006). CrowdStrike and Armis harness clones were clean.

    **Root cause:** The prior audit instruction was "audit the canonical DTU clones" without explicitly naming the harness path. The architect scoped to the canonical path prefix (`crates/prism-dtu-{sensor}/`) and did not scan for ALL behavioral clones in the workspace. This is exactly the class of error that TD-VSDD-091 is designed to catch for spec content — but there was no equivalent discipline for audit scope declaration.

    **Mandatory discipline (all future architect DTU fidelity audits):**

    The canonical `sources_read:` manifest for a DTU fidelity audit MUST include:

    ```yaml
    sources_read:
      - crates/prism-dtu-{sensor}/src/clone.rs
      - crates/prism-dtu-{sensor}/src/routes/{relevant_routes}.rs
      - crates/prism-dtu-harness/src/clones/{sensor}.rs      # MANDATORY — never omit
      - crates/prism-dtu-{sensor}/src/state.rs
      - crates/prism-sensors/specs/{sensor}.sensor.toml
      - .factory/semport/poller-{codename}/poller-{codename}-broad-sweep.md
    ```

    Before completing any architect DTU audit, run: `ls crates/prism-dtu-harness/src/clones/` and verify every listed file is audited. If a new sensor is added, a corresponding harness clone entry MUST be created and audited on the same story.

    **Remediation:** ADR-031 amended to v1.1 with §D7 explicitly enumerating harness-clone paths as in-scope. POLLER-DTU-FIDELITY-AUDIT-2026-05-29 updated to v1.2 with scope-incompleteness addendum. HARNESS-DTU-FIDELITY-AUDIT-2026-05-30 produced as the corrective audit. S-DTU-CYBERINT-AUTH-FIDELITY-001 scope expanded (Scope-1) to include `prism-dtu-harness/src/clones/cyberint.rs` rewrite in the same PR as the canonical DTU fix. Claroty harness audit_log gap co-scoped with S-DEMO-CLAROTY-AUDIT-DTU-001.

    _Discovered: D-850 architect harness-clone audit burst, 2026-05-30. F-LP1-OBS-001 from S-DTU-CYBERINT-AUTH-FIDELITY-001 Pass 1 LOCAL adversary cascade. Triggered by F-LP1-CRIT-001 which surfaced the parallel Cyberint harness clone violation._

## 2026-05-30 D-853 — Adversary Mislabeled File Path Causes Implementer N/A Mis-declaration

55. **[process-gap] [codified] Adversary reports MUST verify that cited file paths actually exist before including them in findings. Implementer declarations of N/A MUST search by symbol/function name across the workspace, not accept the adversary's literal path at face value.**

    **Evidence:** F-LP1-LOW-002 (S-DTU-CYBERINT-AUTH-FIDELITY-001 Pass 1) cited `prism-dtu-cyberint/src/auth_provider.rs` as the file requiring `unsafe { std::env::set_var }` refactoring. The implementer searched that path, found it did not exist, and declared the finding N/A. The PO investigation revealed the file actually lives at `prism-spec-engine/src/auth_provider.rs` — a different crate entirely. The cleanup burst (79e3b545) correctly refactored the 3 unit tests in the real file using `MockCredentialResolver` / `NotFoundCredentialResolver` injection.

    **Adversary rule:** When citing a file path for a finding, verify existence via `test -f <path>` or `rg --files <crate>/src/ | grep <filename>` before finalizing the report. A finding that cites a nonexistent file path is defective and will cause the implementer to declare N/A incorrectly.

    **Implementer rule:** When the adversary's cited file does not exist, search the workspace for the symbol, function name, or pattern before declaring N/A. The correct flow: `rg 'set_var\|unsafe' crates/ --type rust` — find the actual file, then close the finding against the real location.

    **Borderline codification note:** This was a single occurrence in one cascade. Codified because (a) the false-N/A was caught only by PO investigation (not by the implementer's own verification), and (b) the symbol-search-before-N/A rule is a generalizable implementer discipline that would have caught this without PO intervention.

    _Discovered: D-853 implementer cleanup burst, 2026-05-30. F-LP1-LOW-002 from S-DTU-CYBERINT-AUTH-FIDELITY-001 Pass 1 LOCAL adversary cascade._

## 2026-05-30 D-854 — PO Adjudication Requires Verbatim Code Quote from Cited Source-of-Truth Code Path

56. **[process-gap] [codified] PO adjudications resolving code-vs-spec conflicts MUST include verbatim code quotes from the cited code path (file + function/line anchor + actual code text). Adjudications based on spec-narrative derivation alone carry fabrication risk.**

    **Evidence:** PO's Option A adjudication for F-LP1-MED-002 (D-852, commit `4baa0e91`) amended BC-2.01.017 EC-017-005 from E-AUTH-006 to E-AUTH-005, based on the stated rationale: "BC-2.03.006 (credential-backend, more specific) normalizes empty env-var as not-found → Ok(None) → E-AUTH-005." This claim was derived from BC-2.03.006 prose narrative, not from the actual `prism_credentials::resolve_secret` source code.

    **The fabrication exposed by Pass 2 adversary (F-LP2-CRIT-001):** Independent fresh-context re-derivation from `crates/prism-credentials/src/resolve_secret.rs` — `EnvVar` arm — shows that an env-var set to the empty string returns `Ok(Some(SecretString("")))`, NOT `Ok(None)`. The resolver does NOT normalize empty strings as not-found. The "normalization" happens only at the consumer layer if the consumer explicitly guards for empty content. BC-2.03.006 prose describes the CONSUMER's responsibility, not the resolver's wire behavior.

    **Consequence:** BC-2.01.017 v1.1 EC-017-005 mandated the wrong error code for an entire category of auth failures. PO re-adjudicated at `2707ee69`: BC-2.01.017 v1.2 restores E-AUTH-006 for the empty-value path. The fix-burst was avoided only because the adversary's fresh-context pass caught it before the implementer shipped code relying on the wrong contract.

    **CODIFICATION:**

    **PO rule:** When resolving a finding of the form "code returns X but BC mandates Y," the adjudication note MUST include:
    1. The exact function name and module path of the code path cited (e.g., `prism_credentials::resolve_secret` → `EnvVar` arm)
    2. The verbatim code text (or a meaningful excerpt) showing the actual return value
    3. An explicit statement of which behavior (code or spec) is correct and WHY, grounded in the code text

    Example of a compliant adjudication note:
    ```
    resolve_secret.rs EnvVar arm:
        let val = std::env::var(var_name).ok();
        Ok(val.map(SecretString::new))
    For var_name set to "" → std::env::var returns Ok("") → val = Some("") → Ok(Some(SecretString("")))
    This is NOT Ok(None). The resolver does not normalize empty strings.
    Therefore: BC wins at the consumer layer — StaticCookieAuthProvider must guard for empty and return E-AUTH-006.
    ```

    **Orchestrator rule:** When reviewing PO adjudication output for code-vs-spec findings, scan for direct code quotes. If absent, route back to PO for evidence supplementation BEFORE accepting the adjudication. Do not merge the adjudication commit until quotes are present.

    **Borderline escalation note:** This was the first occurrence of PO adjudication fabrication in the project. Codified at CRITICAL severity (F-LP2-CRIT-001) because: (a) the fabrication was plausible — it sounded like correct spec-narrative reasoning; (b) it was caught only by the adversary's fresh-context pass 2, not by any inline review; (c) the consequence was a spec-contract regression that would have caused E2E test failures in the next `just check` after implementer shipped the empty-value guard.

    _Discovered: D-854 Pass 2 LOCAL adversary cascade, 2026-05-30. F-LP2-CRIT-001 from S-DTU-CYBERINT-AUTH-FIDELITY-001 Pass 2 adversary report._

## 2026-05-30 D-856 — Wrong-Crate-Search N/A Pattern (Pass 3 F-LP3-MED-002)

57. **[process-gap] [codified] Implementer must search ALL plausible crates workspace-wide when the adversary cites a file path that does not exist at the cited location. Declaring N/A on "file doesn't exist" without a workspace-wide symbol search is a false closure.**

    **Evidence — F-LP3-MED-002 (Pass 3) re-opens F-LP1-MED-003 (Pass 1):** Pass 1 closed F-LP1-MED-003 as "N/A — file doesn't exist." The adversary's cited path was `crates/prism-dtu-cyberint/tests/parity/cyberint.rs:144`. The implementer searched `crates/prism-dtu-cyberint/tests/parity/` (the adversary's literal path), found no `parity/` directory there, and declared N/A. Pass 3 discovered that the stale comment (claiming "DTU cookie check validates non-empty cyberint_session cookie") actually exists at `crates/prism-spec-engine/tests/parity/cyberint.rs:144` — a different crate entirely.

    **Same wrong-crate pattern as F-LP1-LOW-002 (also in this cascade):** Pass 1 adversary cited `prism-dtu-cyberint/src/auth_provider.rs` for the `unsafe { std::env::set_var }` refactoring finding. The implementer initially searched that path, found it did not exist at that crate, and the correct file was identified as `prism-spec-engine/src/auth_provider.rs`. That instance was caught and corrected during the cleanup burst at commit `79e3b545`. The F-LP1-MED-003 instance was NOT caught — it reached Pass 3 as an unresolved finding.

    **Root cause:** The implementer accepted the adversary's literal file path as authoritative and stopped searching when the literal path did not exist. The adversary's cited paths ARE evidence, not ground truth — they reflect the adversary's best recollection or inference at review time, and adversaries can cite wrong-crate paths when a symbol with the same name exists in multiple crates or when the adversary inferred a likely path from the symbol name.

    **Codification — mandatory implementer discipline for N/A declarations:**

    When the adversary cites a file path and the implementer cannot find the file at the cited location, the implementer MUST execute the following protocol BEFORE declaring N/A:

    1. **Workspace-wide grep for the cited symbol, function, or key phrase:**
       ```bash
       rg '<cited_symbol_or_phrase>' crates/ --type rust
       ```
       For example: `rg 'cyberint_session' crates/ --type rust` or `rg 'cookie check validates' crates/ --type rust`

    2. **Document the actual location in the closure note** — state explicitly which crate/file the symbol was found in (or confirm it was found in ZERO locations workspace-wide)

    3. **Close the finding at the correct location** if the symbol exists in a different crate

    4. **If genuinely no analog exists anywhere in the workspace**, document the workspace-wide grep command and result as evidence for the N/A declaration (e.g., "rg 'cyberint_session' crates/ --type rust returned 0 hits; finding is N/A")

    **State-manager check:** When reviewing implementer fix-burst reports, scan for "N/A — file doesn't exist" closure claims. For each such claim, verify that the closure note includes either (a) a workspace-wide grep result showing zero hits, or (b) a confirmed correct-crate location that was inspected and found clean. Absence of either is an incomplete closure that should be returned to the implementer.

    **Scope of this pattern:** This discipline applies to any implementer N/A declaration where the adversary has cited a specific file path. It does NOT require workspace-wide search for every finding — only for the specific case where the literal adversary path does not resolve.

    _Discovered: D-856 Pass 3 LOCAL adversary cascade, 2026-05-30. F-LP3-MED-002 from S-DTU-CYBERINT-AUTH-FIDELITY-001 Pass 3 re-opens F-LP1-MED-003 closed N/A in Pass 1. Same wrong-crate pattern as F-LP1-LOW-002 (closed at 79e3b545 during cleanup burst). Second occurrence in S-DTU-CYBERINT-AUTH-FIDELITY-001 cascade — codification warranted._

## 2026-05-30 D-860 — Adversary Grounding-Truth Requirement (Pass 5 REJECTED)

58. **[process-gap] [codified] Adversary must self-verify cwd + branch + HEAD as FIRST action before conducting any probes. Orchestrator must independently verify CRIT/HIGH adversary findings before accepting them as real. Adversary reports that claim symbols "do not exist" require `wc -l` + `rg` literal output as evidence — bare assertions are not acceptable.**

    **Evidence — Pass 5 LOCAL adversary REJECTED (S-DTU-CYBERINT-AUTH-FIDELITY-001):** Pass 5 adversary returned 9 findings (5 CRIT + 2 HIGH + 1 MED + 1 PROCESS-GAP) claiming the entire cyberint auth-fidelity implementation does not exist:

    - F-LP5-CRIT-001: `StaticCookieAuthProvider` absent from `auth_provider.rs`; file claimed to be 354 lines
    - F-LP5-CRIT-002: `CredentialResolver` trait + `BackendUnavailableCredentialResolver` absent
    - F-LP5-CRIT-003: `clone.rs:113` still registers `POST /login`
    - F-LP5-CRIT-004: `extract_session_token` still present; no `extract_access_token`
    - F-LP5-CRIT-005: `build_request` unconditionally injects `Authorization: Bearer`
    - F-LP5-HIGH-001: `session_store` UUID naming still in harness (not renamed to `access_token_store`)
    - F-LP5-HIGH-002: Pattern B Scope-1 deliverables unfulfilled; `access_token_store` absent from harness
    - F-LP5-MED-001: `lib.rs:5` still advertises `POST /login`
    - F-LP5-PG-001: Pass 4 closure verification was fabricated

    The report was internally consistent and persuasive. Orchestrator independent verification (running `rg`/`wc -l`/`sed -n` against the actual feature worktree at HEAD `89aa9bd1`) showed ALL 9 findings are false:

    - `auth_provider.rs` is **1092 lines** (not 354)
    - `StaticCookieAuthProvider` EXISTS at line 358
    - `CredentialResolver` trait EXISTS at lines 146-157
    - `BackendUnavailableCredentialResolver` EXISTS at line 287 (cfg-gated for AD-017)
    - `clone.rs:111` has `// NOTE: POST /login route is intentionally ABSENT.` — no route registered
    - `extract_access_token` EXISTS at `alerts.rs:56` + `harness clones/cyberint.rs:760`
    - `access_token_store` EXISTS at `harness cyberint.rs:168`
    - `lib.rs` has 0 hits for `POST /login`
    - Pass 4 line citations all verified accurate against the 1092-line file

    **Hypothesis:** The adversary's "354 lines" claim for `auth_provider.rs` is consistent with the file's line count on `develop@72baf413` (pre-implementation state, before the Pass 1-3 fix-bursts). The adversary likely resolved file paths against the main working tree (`/Users/jmagady/Dev/prism`) rather than the feature worktree (`.worktrees/S-DTU-CYBERINT-AUTH-FIDELITY-001`). Alternatively, stale context from a prior session session carried pre-implementation file snapshots into the Pass 5 review. Pure hallucination is less likely given the specificity of "354 lines" (a plausible pre-implementation size).

    **This is the "structurally-plausible fabrication" pattern:** adversary output that LOOKS rigorous (specific line numbers, specific function names, specific error messages) but is grounded in stale or wrong-branch context. It passes a naive plausibility check because the language is precise. The defense is not stylistic — it is empirical re-verification.

    **Three codifications:**

    **Codification 1 — Adversary self-verification preamble (mandatory for every LOCAL adversary dispatch):**

    Every adversary dispatch prompt MUST require the agent to run the following as its FIRST action before any probes:

    ```bash
    pwd && git branch --show-current && git rev-parse HEAD
    ```

    The agent MUST confirm the output matches the orchestrator-provided expected values:
    - `pwd` must end with the story worktree path (e.g., `.worktrees/S-DTU-CYBERINT-AUTH-FIDELITY-001`)
    - `git branch --show-current` must match the expected feature branch (e.g., `feature/S-DTU-CYBERINT-AUTH-FIDELITY-001`)
    - `git rev-parse HEAD` must match the expected feature HEAD SHA (e.g., `89aa9bd1...`)

    If ANY check fails, the agent MUST STOP and report the mismatch to the orchestrator. The agent must NOT proceed with probes if the preamble check fails.

    **Codification 2 — File-existence proof requirement:**

    When an adversary claims a symbol or file does NOT exist, the finding MUST include:
    - The exact `rg <pattern> <path>` or `rg <pattern> crates/` command run
    - The literal output showing 0 hits (copy-paste, not paraphrase)
    - `wc -l <file>` showing the file's line count
    - `head -1 <file>` or equivalent showing the file exists and is readable

    Without this triad of evidence, "doesn't exist" claims are treated as SUSPECT and may be rejected by the orchestrator without a fix-burst. The orchestrator runs independent verification before accepting any existence-negation claim.

    **Codification 3 — Orchestrator independent verification of CRIT/HIGH findings:**

    When adversary returns a finding of severity CRIT or HIGH, the orchestrator MUST independently run the adversary's cited verification command against the claimed file path BEFORE dispatching any fix-burst. Protocol:

    1. Run `rg <adversary_cited_symbol> <adversary_cited_path>` independently
    2. If symbol EXISTS where adversary claimed it doesn't: finding is REJECTED; mark pass as FABRICATED; increment `pass_N_rejected` counter; do NOT reset streak
    3. If symbol DOES NOT EXIST as claimed: finding is CONFIRMED; proceed with fix-burst dispatch
    4. Orchestrator logs the independent verification result in the convergence-state.json `rejection_basis` field

    **Escalation: when an entire pass is fabricated:** If the orchestrator's independent verification shows that MULTIPLE CRIT/HIGH findings in a single pass are all false, the orchestrator should consider the entire pass suspect and reject it wholesale (rather than cherry-picking individual findings). A pass with 5+ CRIT findings that all fail independent verification is strong evidence of wrong-branch or stale-context pathology, not individual probe error.

    **State-manager obligation:** When recording a pass rejection, state-manager MUST:
    1. Write `adversarial-review/local-pass-N-REJECTED.md` with §1 (verbatim adversary report), §2 (orchestrator refutation with literal command outputs), §3 (root cause hypothesis), §4 (disposition)
    2. Update `adversary-convergence-state.json` with a `"pass": "N-REJECTED"` entry — NOT a normal pass entry that would advance the streak
    3. Update `current_streak` to UNCHANGED (REJECTED pass does not reset and does not advance)
    4. Set `status: "PASS_N_REJECTED_REDISPATCH_PENDING"`
    5. Update STATE.md `pass_N_rejected: true` + `pass_N_rejection_reason`

    _Discovered: D-860 Pass 5 LOCAL adversary rejection, 2026-05-30. S-DTU-CYBERINT-AUTH-FIDELITY-001 cascade. Adversary agent ID a12ee1d29ff472fbf. First occurrence of whole-pass fabrication in this project's adversarial history. 9/9 CRIT/HIGH findings refuted by orchestrator independent verification._

---

## 2026-05-30 D-870 — Three-Recurrence Rule for Sibling-Sweep Defects: Comprehensive Sweep + Policy Codification on Third Occurrence

**Tags:** [process-gap] [codified]

**Lesson 59:** Three consecutive adversarial pass findings of the same class (changelog monotonic-descending ordering — F-LP8-MED-001 BC-2.01.017 + F-LP9-MED-001 story spec + F-LP10-MED-001 error-taxonomy) within a single cascade demonstrates that reactive single-artifact fixes are insufficient when the root cause is a missing or unenforced convention. The reactive pattern (fix the reported site, pass the next pass, surface the same class in a sibling artifact) is a compounding cycle that consumes adversary pass budget without eliminating the defect class.

**Root cause:** No policy existed requiring monotonic descending changelog ordering across all factory artifacts. Each reactive fix addressed only the specific file surfaced by the adversary, leaving all sibling artifacts unchecked. The sibling-sweep discipline (TD-VSDD-060) was applied correctly to code changes, but was not being applied to changelog hygiene across spec/index/taxonomy files.

**Response pattern (three-recurrence rule):** On the THIRD occurrence of the same finding class within a cascade, the orchestrator MUST:

1. Declare the reactive-fix pattern exhausted. The third occurrence is definitive evidence that convention codification has not happened.
2. Route to PO/architect for a **comprehensive sweep** — ALL artifacts in the factory corpus that could contain the same defect class must be enumerated and checked, not just the reported site and its immediate siblings.
3. Execute the comprehensive sweep and the policy codification in the **same atomic burst**. Codifying the policy without sweeping leaves the defect class still present in unchecked artifacts. Sweeping without codifying leaves the convention unenforced for future artifacts.
4. The policy must include: the convention statement, the enforcement scope (which artifact types it applies to), and the finding severity for violations under adversarial review.

**This session's application:** D-870 (commit 559ab76d) executed: (1) comprehensive sweep of error-taxonomy.md, STORY-INDEX, BC-INDEX, and BC-2.16.013; (2) POL-32 codified in `policies.yaml` — `changelog_monotonic_descending: All artifact changelog sections must use monotonic descending version ordering (newest first). Violations are MED findings under adversarial review.` The sweep also promoted D-LP9-001 from deferred to in-scope, closing 2 findings within a single PO burst.

**Forward guidance for state-manager:** When an adversary pass surfaces a finding-class that has appeared in 2 or more prior passes of the same cascade, flag this to the orchestrator as a "recurrence pattern" requiring escalation to the three-recurrence rule protocol. Do not wait for the third occurrence to surface the pattern. The orchestrator decides whether to invoke comprehensive sweep + codification immediately (on second recurrence) or wait for third. The default trigger is THIRD recurrence; the orchestrator may invoke earlier if the recurrence count suggests the class is systemic.

**Codification evidence:** F-LP8-MED-001 closed via PO commit 399ef378 (Pass 8 fix, D-866). F-LP9-MED-001 closed via story-writer commit ac0843a4 (Pass 9 fix, D-868). F-LP10-MED-001 closed via PO comprehensive sweep 559ab76d (Pass 10 fix, D-870) + POL-32 codified at same burst.

_Discovered: D-870 PO comprehensive sweep, 2026-05-30. S-DTU-CYBERINT-AUTH-FIDELITY-001 cascade. Third occurrence of changelog monotonic-descending ordering class. POL-32 codified in policies.yaml v1.31._

## 2026-05-30 D-876 — SAP-4 BC Cite-Pin Sweep Probe + POL-29 Hygiene-Only-Bump Exemption

**Tags:** [process-gap] [codified]

**Lesson 60:** F-LP12-LOW-001 surfaced 20 cite-pins to BC-2.01.017 v1.3 or v1.2 in `crates/prism-spec-engine/src/auth_provider.rs` after BC was bumped v1.3 → v1.4 (D-866 Pass 8 PO fix — hygiene-only changelog cleanup, no semantic change). POL-29 step 8f v1.29 mandates sibling-sweep "including no-semantic-change bumps" but PO adjudication (D-875, commit 23a17f6d) determined all 20 pins are Category A "introduced-in" anchors: e.g., `BC-2.01.017 v1.3 EC-017-010` means EC-017-010 was introduced in v1.3, so the cite-pin is the canonical introduced-in anchor — not a stale pin to update. Two codifications result from this investigation:

**SAP-4 — BC cite-pin sweep probe (new standing adversary probe):**

For every BC/ADR/VP frontmatter version bump surfaced in prior passes, the adversary MUST run:

```bash
rg '<artifact-ID> v<prior-version>' crates/ --type rust
```

For each hit, categorize as:

- **Category A (introduced-in anchor):** The cite-pin points to the version when the cited contract element (EC, postcondition, invariant, error code) was *introduced*. Example: `BC-2.01.017 v1.3 EC-017-010` at an error-handling site means EC-017-010 was introduced in v1.3. Per TD-VSDD-091 introduced-in anchor convention (now documented in BC-2.01.017 v1.5 §Notes), these anchors are correct and must NOT be updated when BC version advances.
- **Category B (current-version pin):** The cite-pin is meant to reference "the current version of this BC" — a general citation that should track the latest version. These MUST be updated when BC version advances.

**Adversary report:** `MED` finding if Category B hits are found without corresponding code update; `LOW` finding (pending-intent) if all hits appear to be introduced-in anchors but no convention is documented in the BC or POL; `CLEAN` (no finding) if convention is documented (BC §Notes or POL entry). F-LP12-LOW-001 was `LOW/pending-intent` because no documentation existed at the time — resolved to `CLEAN` by BC-2.01.017 v1.5 §Notes (D-875) documenting the introduced-in anchor convention project-wide.

**POL-29 step 8f v1.29 hygiene-only-bump exemption (PO recommendation for formal amendment):**

PO recommended amending POL-29 step 8f to read: "including no-semantic-change bumps EXCEPT hygiene-only commits (changelog reorder, schema normalization, formatting) where no EC, postcondition, invariant, or error code was changed." This exemption is proposed because D-866 was a hygiene-only changelog reorder (monotonic descending fix per POL-32) with zero semantic content change — it added no new ECs, did not change any postcondition semantics, and did not add or remove any invariants or error codes. Requiring a full crates cite-pin sweep on hygiene-only commits produces false-positive sweep findings (all 20 hits are introduced-in anchors that correctly point to v1.3 when EC-017-010 was introduced). The exemption would make the sweep obligation proportional to the semantic change level of the bump.

**Orchestrator dispatch:** Policy-owner + spec-steward to formally evaluate and draft POL-29 v1.30 amendment in next session. Until the amendment is formally adopted, the standing rule (POL-29 v1.29 step 8f) applies and the orchestrator must apply SAP-4 to classify all hits before dispatching any fix-burst.

**BC-2.01.017 v1.5 §Notes documentation:** BC now contains a project-wide introduced-in anchor convention description. Future BC authors should follow this pattern when citing specific contract elements in code documentation: use `<BC-ID> v<version> <element-ID>` form where `<version>` is the version when `<element-ID>` was introduced (not the current BC version).

_Discovered: D-876 state-manager closure burst, 2026-05-30. F-LP12-LOW-001 from S-DTU-CYBERINT-AUTH-FIDELITY-001 Pass 12 LOCAL adversary cascade. PO adjudication D-875 (commit 23a17f6d): all 20 cite-pins Category A. BC-2.01.017 v1.5 §Notes documents convention. Orchestrator codification queue: SAP-4 CLAUDE.md amendment + POL-29 step 8f hygiene-only-bump exemption. Count correction: Pass 13 adversary (F-LP13-LOW-001) identified PO table had 20 rows but verdict said 21 — corrected to 20 by D-877 state-manager burst._
