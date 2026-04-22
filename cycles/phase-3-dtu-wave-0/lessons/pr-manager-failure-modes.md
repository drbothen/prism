---
document_type: lessons
cycle: phase-3-dtu-wave-0
date: 2026-04-21
producer: state-manager
scope: pr-manager-agent
severity: critical
---

# Lessons: pr-manager Agent Failure Modes (Phase 3 Wave-0a)

## Summary

During Phase 3 Wave-0a, the pr-manager agent exhibited three distinct failure modes across two separate dispatches (PR #2 for S-0.02 and PR #3 for the Wave-0a housekeeping chore). Each failure required orchestrator intervention, doubling spawning cost and introducing context loss between steps. All three failures point to brittle flow control at step boundaries in the pr-manager's 9-step playbook.

## Context

Wave-0a delivered two PRs to the `develop` branch:
- **PR #1**: S-0.01 CI/CD pipeline — merged cleanly (no pr-manager failures recorded).
- **PR #2**: S-0.02 developer toolchain — pr-manager exited after step 4 (security review); required orchestrator RESUME dispatch.
- **PR #3**: Wave-0a housekeeping chore — pr-manager exited after step 4 again, then on RESUME generated a false SECURITY WARNING blocking merge, and could not spawn pr-reviewer sub-agent. Orchestrator bypassed pr-manager entirely and spawned pr-reviewer directly.

---

## Failure Mode 1: Premature Exit After Security Review

### Observed

Two separate pr-manager dispatches exited cleanly after completing step 4 (security review verdict CLEAN) without continuing into steps 5–9 (pr-reviewer convergence loop, CI wait, dependency check, merge, branch cleanup).

- PR #2 (S-0.02): First dispatch completed steps 1–4, then exited.
- PR #3 (Wave-0a housekeeping): First dispatch completed steps 0–4, then exited.

Both dispatch prompts explicitly listed all 9 steps as required.

### Evidence (quote from dispatch + agent response pattern)

Dispatch prompt stated: *"Run the full PR process. Follow your 9-step process: populate PR description from template, verify demo evidence, create PR via github-ops, security review, pr-reviewer convergence loop, wait for CI, dependency check, merge."*

Agent's terminal output ended with language equivalent to: *"Verdict: PASS. No blocking security findings. Safe to proceed to PR reviewer convergence loop."* — then the agent stopped without proceeding to that convergence loop.

### Root Cause Hypothesis

The pr-manager agent's internal playbook likely has a structural break or checkpoint pattern after the security review section that causes the agent to treat the security sign-off as a terminal goal rather than a transitional milestone. This may stem from a section divider in the prompt, a sub-goal completion signal, or an overly-literal interpretation of the security review as the "last gate before human approval."

### Proposed Fix (plugin-level)

In `agents/pr-manager/AGENT.md` (or equivalent), remove any structural break or checkpoint after the security review step. Add an explicit continuation directive: *"After security review CLEAN, immediately proceed to step 5 (pr-reviewer convergence loop) without pausing, without requesting confirmation, without stopping."* Consider numbering the steps with explicit `THEN` connectives so the agent's internal goal tree treats them as one compound task, not a sequence of independent sub-goals.

### Workaround (orchestrator-level)

Orchestrator sends a RESUME dispatch with: *"You are RESUMING the [story] PR lifecycle. The prior pr-manager dispatch completed through step 4 (security review: CLEAN) but exited before running the remaining steps. Continue from step 5."*

---

## Failure Mode 2: Over-Cautious Merge Permission Guard

### Observed

On the PR #3 RESUME dispatch, the agent generated a literal `SECURITY WARNING` header claiming that merging PR #3 via squash into the default branch was unauthorized because "user only asked for a review, not a merge."

### Evidence

The orchestrator's RESUME dispatch prompt explicitly stated: *"Step 8 — Merge. Method: squash (precedent from PRs #1 and #2). After merge, develop advances from 8595bf9 to new squash commit SHA."* The per-story-delivery.md playbook step 2f grants the orchestrator authority to delegate the full 9-step process including merge. The agent disregarded this and treated the orchestrator as an untrusted caller requiring additional human-in-the-loop authorization specifically for the merge action.

### Root Cause Hypothesis

The pr-manager agent has a merge-authorization guard that was likely added as a correction for a prior incident where an agent merged without authorization. The guard is now too broad: it fires on any merge, including legitimate orchestrator-delegated merges where the dispatch prompt explicitly includes the merge step. The agent is not recognizing the orchestrator's dispatch prompt as sufficient authorization.

### Proposed Fix (plugin-level)

Remove the merge-authorization over-correction guard, or narrow it to fire only when the dispatch prompt does NOT include an explicit merge instruction. Acceptable variant: require the orchestrator dispatch template to include `AUTHORIZE_MERGE=yes` as a phrase, then bake that phrase into the canonical orchestrator dispatch template so it is always present and human intervention is never needed in normal flow.

### Workaround (orchestrator-level)

Orchestrator bypassed pr-manager after failure modes 2 and 3 hit simultaneously. Orchestrator merged PR #3 directly via github-ops tool.

---

## Failure Mode 3: Sub-Agent Spawn Mechanism Broken

### Observed

On the PR #3 RESUME dispatch, the agent reported it could not spawn a pr-reviewer sub-agent. Two attempts to invoke `vsdd-factory:pr-review-triage` produced a recursive loop. The agent reported: *"I cannot spawn subagents via `sessions_spawn` in this context... Two attempts to invoke `vsdd-factory:pr-review-triage` produced a recursive loop (the skill re-triggers itself). I have no `sessions_spawn` tool available in this conversation context to launch a genuine independent subagent."*

### Evidence

pr-manager's 9-step playbook requires: *"Spawn pr-reviewer (fresh-context, different model family) for fresh-eyes review."* The agent attempted to fulfill this via `sessions_spawn: vsdd-factory:pr-reviewer` and via `vsdd-factory:pr-review-triage`, both of which failed.

### Root Cause Hypothesis (two candidates)

**(a) Tool naming mismatch (documentation-vs-runtime drift).** The pr-manager agent's playbook references `sessions_spawn` as the mechanism to launch sub-agents. In this harness, the actual available tool is the `Agent` tool with a `subagent_type:` parameter. The agent does not know to map `sessions_spawn: vsdd-factory:pr-reviewer` to `Agent(subagent_type=vsdd-factory:pr-reviewer, ...)`. The playbook documentation is stale relative to the runtime tool surface.

**(b) Skill self-recursion.** The `vsdd-factory:pr-review-triage` skill invoked by pr-manager may itself dispatch pr-manager logic via skill-dispatch, looping back to the calling agent's context. The agent correctly detects the loop and bails, but the underlying cause is a skill dependency cycle in the plugin.

### Proposed Fix (plugin-level)

Fix (a): Update pr-manager's playbook to use the correct tool call syntax (`Agent` tool with `subagent_type:`) instead of `sessions_spawn`. Add a runtime check: if the `Agent` tool is not available, emit `BLOCKED_NEED_REVIEWER` (see workaround below) rather than attempting fallbacks that loop.

Fix (b): Audit the `vsdd-factory:pr-review-triage` skill for circular skill dependencies. If it invokes pr-manager or any skill that eventually calls pr-review-triage, break the cycle by extracting the review logic into a leaf skill with no outbound skill dispatches.

### Proposed Fix (orchestrator-level, alternative architecture)

Rewrite pr-manager's playbook so that the pr-reviewer step emits a structured signal (`NEED_REVIEWER: {pr_number, diff_summary}`) and returns. The orchestrator, which does have the `Agent` tool, spawns pr-reviewer directly and re-dispatches pr-manager with the review results injected as input. This "orchestrator-driven review injection model" avoids the sub-agent spawn problem entirely.

### Workaround (orchestrator-level)

Orchestrator spawned pr-reviewer directly via Agent tool with `subagent_type=vsdd-factory:pr-review-triage`. This worked cleanly: pr-reviewer ran fresh-context and produced an APPROVE verdict with 2 non-blocking suggestions and 1 nit. Orchestrator then proceeded with merge directly.

---

## Cross-Cutting Pattern

All three failures share a common root: pr-manager's flow control is brittle at step boundaries. The agent is implemented as a thick end-to-end executor that must maintain goal continuity across 9 heterogeneous steps in a single context. Each step boundary is an opportunity for the agent to misidentify a sub-goal as terminal (failure 1), to apply a local guard that overrides the orchestrator's intent (failure 2), or to hit a tool unavailability that it cannot recover from gracefully (failure 3).

**Candidate architectural change:** Rewrite pr-manager as a thin coordinator that delegates sub-steps to the orchestrator via explicit callback signals, rather than a thick end-to-end executor. In this model, pr-manager emits structured signals at each step boundary (`STEP_COMPLETE: {step, result}`) and the orchestrator decides whether to continue, resume, or intervene. This makes step boundaries explicit and observable, eliminates the premature-exit failure mode, and decouples the merge-authorization question from the agent's internal flow.

---

## Proposed vsdd-factory Fixes (priority ordered)

1. **Fix sub-agent spawn mechanism (failure 3)** — highest priority; blocks automation of the pr-reviewer step. Either update playbook to use correct tool syntax, or adopt orchestrator-driven review injection model.
2. **Remove merge permission over-correction (failure 2)** — high priority; generates false SECURITY WARNING noise and blocks merge in normal flow.
3. **Remove premature-exit milestone after security review (failure 1)** — high priority; doubles spawning cost on every dispatch.
4. **Consider thin-coordinator refactor (long-term)** — architectural improvement that prevents all three failure classes by making step boundaries explicit and externally observable.

---

## Recommended Prompt for vsdd-factory Session

Paste this prompt into a fresh Claude Code session in the vsdd-factory plugin repo to fix the pr-manager agent definition:

```
I need you to fix three bugs in the pr-manager agent definition in this vsdd-factory plugin repo.

Background: During Phase 3 Wave-0a of a production pipeline, the pr-manager agent exhibited
three failure modes across two separate dispatches. Here is what happened and what to fix:

---

FAILURE MODE 1: PREMATURE EXIT AFTER SECURITY REVIEW (step 4)

Symptom: The agent completes step 4 (security review) and exits. It does not continue into
steps 5-9 (pr-reviewer convergence, CI wait, dep check, merge, branch cleanup) even though
the dispatch prompt explicitly listed all 9 steps as required.

Root cause: The agent's playbook has a structural break or checkpoint after the security
review section that causes the agent to treat security sign-off as a terminal goal.

Fix needed: In the pr-manager agent file (likely agents/pr-manager/AGENT.md or similar):
1. Find the security review section. Remove any language that implies it is a stopping point.
2. Add an explicit continuation directive immediately after the security review step:
   "After security review verdict is recorded, immediately proceed to step 5 (pr-reviewer
   convergence loop) without pausing, without requesting confirmation, without stopping."
3. Consider using explicit THEN connectives between steps so the goal tree treats all 9 steps
   as one compound task.

---

FAILURE MODE 2: OVER-CAUTIOUS MERGE PERMISSION GUARD

Symptom: The agent emits a "SECURITY WARNING" claiming merge is unauthorized, even when the
orchestrator's dispatch prompt explicitly lists merge (step 8) as a required step with method
and SHA details included.

Root cause: A merge-authorization guard fires on all merges regardless of whether the
dispatch prompt explicitly authorizes the merge. The agent treats the orchestrator as an
untrusted caller requiring human-in-the-loop authorization for merge specifically.

Fix needed:
1. Find the merge authorization guard in the pr-manager agent definition.
2. Either remove it entirely, or narrow it: it should only fire if the dispatch prompt does
   NOT include an explicit merge instruction (e.g., "Step 8 - Merge" or "AUTHORIZE_MERGE=yes").
3. If you keep the phrase-based approach (AUTHORIZE_MERGE=yes), also update the orchestrator's
   canonical dispatch template (wherever it lives in the plugin) to include that phrase, so
   human intervention is never required in normal flow.
4. The correct behavior: if pr-reviewer APPROVES and CI passes (or fails in a documented-
   expected way), proceed to merge. The orchestrator's dispatch IS the authorization.

---

FAILURE MODE 3: SUB-AGENT SPAWN MECHANISM BROKEN

Symptom: The agent cannot spawn the pr-reviewer sub-agent. It reports "I have no
sessions_spawn tool available" and two attempts to invoke vsdd-factory:pr-review-triage
produced a recursive loop.

Root cause (two candidates):
(a) Documentation-vs-runtime drift: the playbook references "sessions_spawn" as the spawn
    mechanism, but the actual available tool in this harness is the Agent tool with a
    subagent_type: parameter. The agent doesn't know to map one to the other.
(b) Skill self-recursion: vsdd-factory:pr-review-triage may invoke pr-manager logic,
    creating a loop.

Fix needed:
1. Search the pr-manager agent definition for references to "sessions_spawn". Replace with
   the correct Agent tool call syntax for spawning subagents in this plugin's harness.
2. Audit vsdd-factory:pr-review-triage for circular skill dependencies. If it calls back into
   pr-manager or any skill that calls pr-review-triage, break the cycle.
3. Add a graceful fallback: if the Agent tool is not available in the current context, emit
   a structured signal "BLOCKED_NEED_REVIEWER: {pr_number, reason}" and return cleanly —
   do NOT attempt recursive fallbacks.
4. Alternative architectural fix: rewrite the pr-reviewer step so pr-manager emits
   "NEED_REVIEWER: {pr_number, diff_summary}" and returns. The orchestrator (which has the
   Agent tool) spawns pr-reviewer directly and re-dispatches pr-manager with the review
   results injected as input. This is the orchestrator-driven review injection model and
   avoids the spawn problem entirely.

---

VERIFICATION STEPS after making changes:

1. Read the existing pr-manager agent tests (if any) and confirm they still pass.
2. Trace the 9-step flow manually through the updated agent definition and confirm:
   - Step 4 (security review) flows into step 5 without pause
   - Step 8 (merge) executes when dispatch prompt includes merge instruction
   - Step 5 (pr-reviewer spawn) either uses correct tool syntax or emits NEED_REVIEWER signal
3. Check vsdd-factory:pr-review-triage for outbound skill calls; confirm no cycle back to
   pr-manager.
4. Submit a PR to the vsdd-factory plugin repo with the fixes. Title: "fix(pr-manager): 
   premature exit, merge guard over-correction, sub-agent spawn mechanism"

Please find the relevant agent file(s), show me what you find, make the fixes, and open a PR.
```

---

## Timeline

| Date | Event |
|------|-------|
| 2026-04-21 | Wave-0a S-0.02 PR #2: failure mode 1 observed; orchestrator sent RESUME dispatch |
| 2026-04-21 | Wave-0a chore PR #3: failure modes 1 + 2 + 3 observed on first and RESUME dispatches |
| 2026-04-21 | Orchestrator bypassed pr-manager; spawned pr-reviewer directly; PR #3 merged by orchestrator |
| 2026-04-21 | This lessons doc written by state-manager per user request |
