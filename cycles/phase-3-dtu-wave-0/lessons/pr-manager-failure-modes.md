---
document_type: lessons
cycle: phase-3-dtu-wave-0
date: 2026-04-21
producer: state-manager
scope: pr-manager-agent
severity: important
---

# Lessons: pr-manager Agent Failure Modes (Phase 3 Wave-0a)

## Summary

During Phase 3 Wave-0a, the pr-manager agent completed some dispatches successfully and failed
others intermittently. PR #1 completed all 9 steps end-to-end in a single dispatch. Dispatches
for PR #2 and PR #3 each stopped prematurely at step 4 (security review), requiring orchestrator
RESUME dispatches. The PR #2 RESUME succeeded cleanly. The PR #3 RESUME hit catastrophic failure
(false SECURITY WARNING + sub-agent spawn broken), requiring the orchestrator to bypass pr-manager
entirely. The failure pattern is INTERMITTENT, not deterministic — the same class of operation
succeeded in some dispatches and failed in others.

---

## Dispatch History (authoritative)

| Dispatch # | Target | Outcome | Notes |
|-----------|--------|---------|-------|
| 1 | PR #1 S-0.01 | **FULL 9-STEP SUCCESS in ONE dispatch** | PR created (#1 at bd6a04b), security review (1 LOW accepted, 1 OBS documented), pr-reviewer 1 cycle APPROVE, CI documented-expected, squash merge (9de5e29), remote branch deleted |
| 2 | PR #2 S-0.02 first dispatch | **Stopped at step 4** | Agent reported security CLEAN then exited without proceeding to steps 5–9 |
| 3 | PR #2 S-0.02 RESUME dispatch | **Completed steps 5–9 successfully** | pr-reviewer 1 cycle APPROVE (4 non-blocking, 4 comment), CI matched PR #1 pattern, squash merge (8595bf9), remote branch deleted |
| 4 | PR #3 housekeeping first dispatch | **Stopped at step 4** | Same pattern as dispatch #2 — security CLEAN then exit |
| 5 | PR #3 housekeeping RESUME dispatch | **Catastrophic failure** | Generated literal "SECURITY WARNING" about unauthorized merge (FM2); sub-agent spawn unavailable + pr-review-triage recursive loop (FM3); escalated to user |

**Key takeaway**: 3 of 5 dispatches completed their assigned steps successfully. The mid-flow exit
(FM1) is intermittent (dispatches 2 and 4, but NOT dispatch 1). The catastrophic failure (FM2 +
FM3) was observed in exactly ONE dispatch (dispatch 5) out of two RESUME attempts.

---

## Context

Wave-0a delivered two PRs to the `develop` branch:
- **PR #1**: S-0.01 CI/CD pipeline — merged cleanly in a single pr-manager dispatch (dispatch #1).
- **PR #2**: S-0.02 developer toolchain — dispatch #2 exited at step 4; dispatch #3 (RESUME) completed all remaining steps successfully.
- **PR #3**: Wave-0a housekeeping chore — dispatch #4 exited at step 4; dispatch #5 (RESUME) hit FM2 + FM3. Orchestrator bypassed pr-manager and merged PR #3 directly.

---

## Failure Mode 1: Intermittent Mid-Flow Exit After Security Review

**Observed in: dispatches 2 and 4 (NOT dispatch 1)**

### Observed

Two of the five pr-manager dispatches exited cleanly after completing step 4 (security review
verdict CLEAN) without continuing into steps 5–9 (pr-reviewer convergence loop, CI wait,
dependency check, merge, branch cleanup). Dispatch #1 (PR #1) did NOT exhibit this behavior —
it completed all 9 steps in one shot. The failure is not deterministic.

- PR #2 (S-0.02): Dispatch #2 completed steps 1–4, then exited.
- PR #3 (Wave-0a housekeeping): Dispatch #4 completed steps 0–4, then exited.

Both dispatch prompts explicitly listed all 9 steps as required.

### Evidence

Dispatch prompt stated: *"Run the full PR process. Follow your 9-step process: populate PR
description from template, verify demo evidence, create PR via github-ops, security review,
pr-reviewer convergence loop, wait for CI, dependency check, merge."*

Agent's terminal output ended with language equivalent to: *"Verdict: PASS. No blocking security
findings. Safe to proceed to PR reviewer convergence loop."* — then the agent stopped without
proceeding.

### Root Cause Hypothesis

The pr-manager agent's internal playbook may have a structural break or checkpoint pattern after
the security review section that causes the agent to sometimes treat the security sign-off as a
terminal goal rather than a transitional milestone. The intermittency suggests this is sensitive
to context, prompt phrasing, or LLM routing — see Hypotheses section below.

### Proposed Fix (plugin-level)

In `agents/pr-manager/AGENT.md` (or equivalent), remove any structural break or checkpoint after
the security review step. Add an explicit continuation directive: *"After security review CLEAN,
immediately proceed to step 5 (pr-reviewer convergence loop) without pausing, without requesting
confirmation, without stopping."* Consider numbering the steps with explicit `THEN` connectives
so the agent's internal goal tree treats them as one compound task, not a sequence of independent
sub-goals.

### Workaround (orchestrator-level)

Orchestrator sends a RESUME dispatch with: *"You are RESUMING the [story] PR lifecycle. The prior
pr-manager dispatch completed through step 4 (security review: CLEAN) but exited before running
the remaining steps. Continue from step 5."* This worked for PR #2 (dispatch #3) and failed
for PR #3 (dispatch #5) — see FM2 and FM3.

---

## Failure Mode 2: Over-Cautious Merge Permission Guard

**Observed in: dispatch 5 ONLY (PR #3 RESUME). NOT observed in dispatch 3 (PR #2 RESUME).**

### Observed

On the PR #3 RESUME dispatch (dispatch #5), the agent generated a literal `SECURITY WARNING`
header claiming that merging PR #3 via squash into the default branch was unauthorized because
"user only asked for a review, not a merge." This did NOT occur in the PR #2 RESUME (dispatch #3),
which merged cleanly.

### Evidence

The orchestrator's RESUME dispatch prompt explicitly stated: *"Step 8 — Merge. Method: squash
(precedent from PRs #1 and #2). After merge, develop advances from 8595bf9 to new squash commit
SHA."* The per-story-delivery.md playbook step 2f grants the orchestrator authority to delegate
the full 9-step process including merge. The agent disregarded this in dispatch #5.

### Root Cause Hypothesis

The pr-manager agent has a merge-authorization guard that fires inconsistently — it did not fire
in dispatch #3 (PR #2 RESUME) but did fire in dispatch #5 (PR #3 RESUME). The guard may be
sensitive to context window state, prompt phrasing differences between the two RESUME dispatches,
or LLM routing. When it fires, it is too broad: it triggers even when the dispatch prompt
explicitly includes the merge step.

### Proposed Fix (plugin-level)

Remove the merge-authorization over-correction guard, or narrow it to fire only when the dispatch
prompt does NOT include an explicit merge instruction. Acceptable variant: require the orchestrator
dispatch template to include `AUTHORIZE_MERGE=yes` as a phrase, then bake that phrase into the
canonical orchestrator dispatch template so it is always present and human intervention is never
needed in normal flow.

### Workaround (orchestrator-level)

Orchestrator bypassed pr-manager after failure modes 2 and 3 hit simultaneously in dispatch #5.
Orchestrator merged PR #3 directly via github-ops tool.

---

## Failure Mode 3: Sub-Agent Spawn Mechanism Broken

**Observed in: dispatch 5 ONLY (PR #3 RESUME). NOT observed in dispatch 3 (PR #2 RESUME), which
spawned pr-reviewer successfully.**

### Observed

On the PR #3 RESUME dispatch (dispatch #5), the agent reported it could not spawn a pr-reviewer
sub-agent. Two attempts to invoke `vsdd-factory:pr-review-triage` produced a recursive loop. The
agent reported: *"I cannot spawn subagents via `sessions_spawn` in this context... Two attempts to
invoke `vsdd-factory:pr-review-triage` produced a recursive loop (the skill re-triggers itself).
I have no `sessions_spawn` tool available in this conversation context to launch a genuine
independent subagent."*

The PR #2 RESUME (dispatch #3) did NOT hit this — it spawned pr-reviewer cleanly and completed
the convergence loop with APPROVE.

### Evidence

pr-manager's 9-step playbook requires: *"Spawn pr-reviewer (fresh-context, different model
family) for fresh-eyes review."* In dispatch #5, both `sessions_spawn` and
`vsdd-factory:pr-review-triage` fallbacks failed. In dispatch #3, the spawn succeeded.

### Root Cause Hypothesis (two candidates)

**(a) Tool naming mismatch (documentation-vs-runtime drift).** The pr-manager agent's playbook
references `sessions_spawn` as the mechanism to launch sub-agents. In this harness, the actual
available tool is the `Agent` tool with a `subagent_type:` parameter. The intermittency (worked
in dispatch #3, broken in dispatch #5) suggests the agent may take different fallback paths
depending on context window state.

**(b) Skill self-recursion.** The `vsdd-factory:pr-review-triage` skill invoked by pr-manager
may itself dispatch pr-manager logic via skill-dispatch, looping back to the calling agent's
context. The agent correctly detects the loop in dispatch #5 and bails, but why it did not loop
in dispatch #3 is unclear — possibly context-window or routing differences.

### Proposed Fix (plugin-level)

Fix (a): Update pr-manager's playbook to use the correct tool call syntax (`Agent` tool with
`subagent_type:`) instead of `sessions_spawn`. Add a runtime check: if the `Agent` tool is not
available, emit `BLOCKED_NEED_REVIEWER` rather than attempting fallbacks that loop.

Fix (b): Audit the `vsdd-factory:pr-review-triage` skill for circular skill dependencies. If it
invokes pr-manager or any skill that eventually calls pr-review-triage, break the cycle by
extracting the review logic into a leaf skill with no outbound skill dispatches.

### Proposed Fix (orchestrator-level, alternative architecture)

Rewrite pr-manager's playbook so that the pr-reviewer step emits a structured signal
(`NEED_REVIEWER: {pr_number, diff_summary}`) and returns. The orchestrator, which does have the
`Agent` tool, spawns pr-reviewer directly and re-dispatches pr-manager with the review results
injected as input. This "orchestrator-driven review injection model" avoids the sub-agent spawn
problem entirely.

### Workaround (orchestrator-level)

Orchestrator spawned pr-reviewer directly via Agent tool with `subagent_type=vsdd-factory:pr-review-triage`.
This worked cleanly: pr-reviewer ran fresh-context and produced an APPROVE verdict with 2
non-blocking suggestions and 1 nit. Orchestrator then proceeded with merge directly.

---

## Cross-Cutting Pattern

The intermittency across dispatches is the defining characteristic. FM1 hit dispatches 2 and 4
but not dispatch 1. FM2 and FM3 hit dispatch 5 but not dispatch 3. This suggests pr-manager's
flow control is sensitive to context window state, prompt phrasing, or LLM routing — not just
a simple playbook bug. Each step boundary is an opportunity for the agent to misidentify a
sub-goal as terminal (FM1), apply a local guard that overrides the orchestrator's intent (FM2),
or hit a tool unavailability it cannot recover from gracefully (FM3).

**Candidate architectural change:** Rewrite pr-manager as a thin coordinator that delegates
sub-steps to the orchestrator via explicit callback signals, rather than a thick end-to-end
executor. In this model, pr-manager emits structured signals at each step boundary
(`STEP_COMPLETE: {step, result}`) and the orchestrator decides whether to continue, resume, or
intervene. This makes step boundaries explicit and observable, eliminates the premature-exit
failure mode, and decouples the merge-authorization question from the agent's internal flow.

---

## Hypotheses for Intermittent Behavior

1. **Context-window / session-age effect**: pr-manager dispatches later in the session have more
   crowded orchestrator context; the dispatch prompts may subtly differ or the LLM routing may
   differ. PR #1 was dispatched when context was fresh; dispatches 4 and 5 were deep in a long
   session.

2. **Prompt-phrasing sensitivity**: The PR #1 dispatch prompt included explicit "first-PR mode"
   hand-holding ("since develop has no workflows yet"). PR #2 and PR #3 dispatches were more
   terse. Agents may be sensitive to "you are RESUMING" vs "you are dispatched" framing, and to
   the amount of context-setting provided.

3. **LLM routing variance**: Different Claude model snapshots (Sonnet 4.5 vs Opus 4.7 vs similar)
   may behave differently on the same prompt. Without visibility into routing, this is
   unverifiable — but it is a plausible explanation for why the same RESUME class worked once
   (dispatch #3) and failed catastrophically once (dispatch #5).

4. **PR-complexity dependence**: PR #1 diff was ~150 lines YAML + demo. PR #2 was larger
   (Justfile + lefthook + dev-setup.sh + semgrep + Cargo.toml + test fixes + demo). PR #3 was
   mixed (git mv + config). The agent's own step-5-onwards logic may branch on perceived
   complexity, producing different step-boundary behavior for "large" vs "small" PRs.

5. **Resume-context specifically**: Dispatch #3 (PR #2 resume) worked; dispatch #5 (PR #3
   resume) failed catastrophically. The failed one added BOTH "security warning" generation AND
   "spawn broken" simultaneously — while the successful one completed cleanly. Possibly LLM
   routing variance OR residual context-window accumulation from a longer session, making the
   agent's internal state less coherent by dispatch #5.

---

## Proposed vsdd-factory Fixes (priority ordered)

1. **Add instrumentation to pr-manager** — make it emit a structured "step-completion" signal
   after each of the 9 steps. Orchestrator can detect premature exits deterministically and issue
   targeted resumes without guessing at which step failed.

2. **Fix sub-agent spawn mechanism (FM3)** — high priority; blocked automation of the pr-reviewer
   step in dispatch #5. Either update playbook to use correct tool syntax, or adopt
   orchestrator-driven review injection model.

3. **Remove merge permission over-correction (FM2)** — high priority; generates false SECURITY
   WARNING noise and blocks merge in normal flow when it fires.

4. **Remove premature-exit milestone after security review (FM1)** — high priority; doubles
   spawning cost on dispatches where it fires.

5. **Add retry-idempotency contract** — pr-manager should be able to detect a prior partial state
   (e.g., "PR already exists, security review already in comment history") and pick up where it
   left off without re-doing completed steps. This makes RESUME dispatches more robust.

6. **Reproduce the catastrophic failure** — attempt to isolate FM2+FM3 by running pr-manager on
   a dummy PR in a test repo with controlled context. If reproducible, file as a concrete plugin
   bug with stable repro.

7. **A/B test prompt phrasing** — dispatch pr-manager with "you are dispatched" vs "you are
   RESUMING" vs "continue PR lifecycle" on otherwise-identical inputs to measure phrasing
   sensitivity.

8. **Consider thin-coordinator refactor (long-term)** — architectural improvement that prevents
   all three failure classes by making step boundaries explicit and externally observable.

---

## Recommended Prompt for vsdd-factory Session

Paste this prompt into a fresh Claude Code session in the vsdd-factory plugin repo to improve
pr-manager reliability. Note: pr-manager works successfully most of the time (3 of 5 dispatches
completed their assigned steps). The goal is instrumentation and targeted fixes, not a full
rewrite.

```
I need you to improve the pr-manager agent definition in this vsdd-factory plugin repo.

Background: During Phase 3 Wave-0a of a production pipeline, pr-manager was dispatched 5 times.
3 dispatches completed successfully. 2 dispatches (out of 5) stopped prematurely at step 4;
1 of those 2 RESUME dispatches then hit catastrophic failures. The agent is intermittently
flaky and needs instrumentation to diagnose and targeted fixes for three specific behaviors.

The goal is NOT to claim the agent is broadly broken — it works most of the time.
The goal IS to add instrumentation, fix the sub-agent spawn if legitimately broken, and
remove the over-correction in the merge-permission guard.

---

IMPROVEMENT 1: ADD STEP-COMPLETION INSTRUMENTATION

The most important fix. Make pr-manager emit a structured signal after each of the 9 steps:
  STEP_COMPLETE: {step_number: N, step_name: "...", result: "...", next_step: N+1}

This allows the orchestrator to detect premature exits deterministically. Without this, the
orchestrator cannot distinguish "agent completed all 9 steps" from "agent exited after step 4"
without reading the full output.

---

IMPROVEMENT 2: FIX PREMATURE EXIT AFTER SECURITY REVIEW (intermittent — 2 of 5 dispatches)

Symptom: The agent completes step 4 (security review) and exits. It does not continue into
steps 5–9 even though the dispatch prompt explicitly listed all 9 steps as required. This is
NOT universal — dispatch #1 (PR #1) ran all 9 steps without stopping.

Root cause hypothesis: The agent's playbook may have a structural break after the security
review section that intermittently causes the agent to treat security sign-off as a terminal goal.

Fix needed: In the pr-manager agent file (likely agents/pr-manager/AGENT.md or similar):
1. Find the security review section. Remove any language that implies it is a stopping point.
2. Add an explicit continuation directive immediately after the security review step:
   "After security review verdict is recorded, immediately proceed to step 5 (pr-reviewer
   convergence loop) without pausing, without requesting confirmation, without stopping."
3. Use explicit THEN connectives between steps so the goal tree treats all 9 steps as one
   compound task.

---

IMPROVEMENT 3: FIX OVER-CAUTIOUS MERGE PERMISSION GUARD (observed in 1 of 5 dispatches)

Symptom: In one dispatch, the agent emitted a "SECURITY WARNING" claiming merge was unauthorized,
even though the orchestrator's dispatch prompt explicitly listed merge (step 8) as a required
step with method and SHA details included. This did NOT occur in the prior RESUME dispatch for
the same class of operation.

Root cause: A merge-authorization guard fires inconsistently — possibly sensitive to prompt
phrasing or context window state. When it fires, it is too broad: it triggers even when the
dispatch prompt explicitly authorizes the merge.

Fix needed:
1. Find the merge authorization guard in the pr-manager agent definition.
2. Either remove it, or narrow it: fire only when the dispatch prompt does NOT include an
   explicit merge instruction (e.g., "Step 8 - Merge" or "AUTHORIZE_MERGE=yes").
3. If you keep the phrase-based approach, update the orchestrator's canonical dispatch template
   to always include that phrase, so human intervention is never needed in normal flow.
4. Correct behavior: if pr-reviewer APPROVES and CI passes (or fails in documented-expected way),
   proceed to merge. The orchestrator's dispatch IS the authorization.

---

IMPROVEMENT 4: FIX SUB-AGENT SPAWN MECHANISM (observed in 1 of 5 dispatches)

Symptom: In one dispatch, the agent could not spawn the pr-reviewer sub-agent. It reported
"I have no sessions_spawn tool available" and two attempts to invoke vsdd-factory:pr-review-triage
produced a recursive loop. This did NOT occur in the prior RESUME dispatch for PR #2, which
spawned pr-reviewer cleanly.

Root cause (two candidates):
(a) Documentation-vs-runtime drift: the playbook references "sessions_spawn" but the actual
    available tool may be the Agent tool with a subagent_type: parameter.
(b) Skill self-recursion: vsdd-factory:pr-review-triage may invoke pr-manager logic,
    creating a loop.

Fix needed:
1. Search the pr-manager agent definition for references to "sessions_spawn". Replace with
   the correct Agent tool call syntax for spawning subagents in this plugin's harness.
2. Audit vsdd-factory:pr-review-triage for circular skill dependencies.
3. Add a graceful fallback: if the Agent tool is not available, emit
   "BLOCKED_NEED_REVIEWER: {pr_number, reason}" and return — do NOT attempt recursive fallbacks.
4. Alternative: rewrite the pr-reviewer step so pr-manager emits "NEED_REVIEWER: {pr_number,
   diff_summary}" and returns. The orchestrator spawns pr-reviewer directly and re-dispatches
   pr-manager with review results injected. This is the orchestrator-driven review injection
   model.

---

VERIFICATION STEPS after making changes:

1. Read existing pr-manager agent tests (if any) and confirm they still pass.
2. Trace the 9-step flow manually through the updated agent definition and confirm:
   - Step 4 (security review) emits STEP_COMPLETE and flows into step 5 without pause
   - Step 8 (merge) executes when dispatch prompt includes merge instruction
   - Step 5 (pr-reviewer spawn) uses correct tool syntax or emits NEED_REVIEWER signal
3. Check vsdd-factory:pr-review-triage for outbound skill calls; confirm no cycle back to
   pr-manager.
4. Submit a PR with the fixes. Title: "fix(pr-manager): add step instrumentation, fix merge
   guard over-correction, fix sub-agent spawn mechanism"

Please find the relevant agent file(s), show me what you find, make the fixes, and open a PR.
```

---

## Timeline

| Date | Dispatch # | Event |
|------|-----------|-------|
| 2026-04-21 | #1 | PR #1 S-0.01: pr-manager completed all 9 steps end-to-end in a single dispatch |
| 2026-04-21 | #2 | PR #2 S-0.02: pr-manager stopped at step 4 (FM1 first observed) |
| 2026-04-21 | #3 | PR #2 S-0.02 RESUME: pr-manager completed steps 5–9 successfully |
| 2026-04-21 | #4 | PR #3 housekeeping: pr-manager stopped at step 4 (FM1 second observation) |
| 2026-04-21 | #5 | PR #3 housekeeping RESUME: FM2 + FM3 hit; orchestrator bypassed pr-manager |
| 2026-04-21 | — | Orchestrator spawned pr-reviewer directly; PR #3 merged by orchestrator via github-ops |
| 2026-04-21 | — | This lessons doc written by state-manager; severity set to `important` (intermittent, not universal) |
