# Lessons Learned — S-DEMO-QUERY-PUSHDOWN-001 (wave-5-e-demo-fidelity)

## Codification Candidates

### [recurrence] ADV-P05-HIGH-001 — Hand-fed-FQL-vs-real-path test anti-pattern: when fixing one hand-fed test, sweep ALL push-down tests across ALL crates (TD-VSDD-060)

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1012
**Finding:** ADV-P05-HIGH-001 (v2.x LOCAL pass-5)
**Tags:** [recurrence] [sibling-sweep] [TD-VSDD-060] [dead-code-via-test-layer]
**Classification:** BLOCKING — recurrence of a previously closed defect class; exhaustive sweep is the required response.

**Description:**
Pass 4 caught the hand-fed-FQL + direct-`PipelineExecutor::execute`-boundary test anti-pattern for AC-CWS-002 (in `prism-bin`). The pass-4 fix correctly addressed AC-CWS-002 but did not exhaustively sweep sibling tests across other crates in `crates_touched`. Pass 5 found the same defect class at AC-EQUIV-001 in `prism-spec-engine` — the test claimed "via real materialization path" in its name but bypassed `run_materialization_pipeline`.

This is a **cross-crate sibling-sweep gap**: the pass-4 fix-burst operated within `prism-bin` and did not review the `prism-spec-engine` AC-EQUIV-001 test, which was testing the same behavioral property (result equivalence via real path) but living in a different crate.

**Root cause of recurrence:**
The pass-4 AC-CWS-002 fix-burst swept within `prism-bin` but treated the fix as crate-scoped. The AC-EQUIV-001 test in `prism-spec-engine` existed independently and was NOT reviewed as a sibling despite both tests covering the same "real materialization path" behavioral domain (BC-2.11.007 result-equivalence).

**Correct response (codified rule):**
When fixing a hand-fed-FQL / direct-`PipelineExecutor::execute`-boundary test anti-pattern, the fix-burst MUST:
1. Sweep ALL push-down tests across ALL `crates_touched` (not just the crate where the defect was found).
2. Explicitly list and disposition each test: real-path vs boundary vs unit (per the 32-test sweep pattern used in pass 5).
3. Any test whose NAME claims "via real path" / "via run_materialization_pipeline" / "via production entry point" but whose BODY does NOT call that function = P1 HIGH finding; fix in the same burst.

**Outcome:** Pass-5 fix-burst added exhaustive sibling-sweep of 32 push-down tests across 4 crates (TD-VSDD-060 compliant). All 32 dispositioned. No further misnamed real-path claims remain.

**Codification direction (for future session-reviewer / VSDD process improvement):**
- This pattern should become a standing adversary probe: after any fix involving `run_materialization_pipeline` test rewrites, adversary MUST grep for ALL tests containing `"run_materialization_pipeline"` in their name or doc-comment across the full workspace, then verify each drives the function, not just PipelineExecutor directly.
- Codification could be SAP-3 or an amendment to SAP-1/SAP-2 scope (TD-VSDD-060 sweep extension).

---

### [process-gap] ADV-P04-OBS-001 — Test-docstring file.rs:NNN line-pin discipline gap (TD-VSDD-091 adjacency)

**Date recorded:** 2026-06-05
**D-NNN anchor:** D-1011
**Finding:** ADV-P04-OBS-001 (v2.x LOCAL pass-4)
**Tags:** [process-gap] [codification-candidate] [TD-VSDD-091-adjacency]
**Classification:** NOT blocking — process improvement candidate only.

**Description:**
Several test function doc-comments in `crates/prism-dtu-crowdstrike/` and `crates/prism-query/src/pushdown.rs` cite source-file locations as `file.rs:NNN` (line-number pins). TD-VSDD-091 (anti-volatile-pin discipline) prohibits line-number pins in narrative spec content because they decay on subsequent diffs. The rule is currently enforced by adversary pass probes for `.factory/` SPEC artifacts.

However, there is no automated enforcement path for test-docstring line-number pins inside `crates/**/*.rs` files. A test's `/// See file.rs:123` style comment is technically in-code narrative — it can decay the same way a spec pin does — but it falls outside the current TD-VSDD-091 adversary probe scope and no pre-commit lint exists for it.

**Codification direction (for future session-reviewer / factory process improvement):**
- Option A: Extend adversary SAP probe to grep `crates/**/*.rs` doc-comments for `\w+\.rs:\d+` pattern and flag as LOW findings.
- Option B: Add a clippy custom lint or a pre-commit hook regex scan for `\w+\.rs:\d+` inside `///` comment lines.
- Option C: Record as CLAUDE.md standing convention and address opportunistically during fix-bursts that touch doc-comments.

**Resolution:** Option C is the minimum bar; Option A is the right permanent fix. Codification decision deferred to session-reviewer at cycle-close. Do NOT add a new story now — this is a process/lint gap, not an unmet user-facing requirement.

**Non-blocking confirmation:** The line-pin sites do not affect test behavior or correctness. Code passes `just check 4032/4032`. This record exists only as a cycle-close codification prompt.
