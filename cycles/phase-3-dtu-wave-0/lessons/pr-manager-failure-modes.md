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

## Confirmed Root Cause (2026-04-21)

**All three failure modes trace to one root cause: the pr-manager agent definition references
`sessions_spawn` as the sub-agent spawn mechanism, but `sessions_spawn` does not exist in the
current Claude Code harness.** The correct mechanism is the `Agent` tool with a `subagent_type:`
parameter.

This is not a hypothesis. It is confirmed by forensic examination of three artifacts:

**Evidence 1 — Agent definition tool-name mismatch.**
`/Users/jmagady/.claude/plugins/cache/vsdd-factory/vsdd-factory/0.48.0/agents/pr-manager.md`
references `sessions_spawn` 13 times as the authoritative spawn mechanism (lines 36, 37, 76-78,
182, and elsewhere). The runtime harness does not expose this tool. When the agent reaches step 5
and attempts to invoke `sessions_spawn`, the tool is absent.

**Evidence 2 — Dispatch #5 honest failure reporting.**
`/Users/jmagady/dev/prism/.factory/code-delivery/chore-wave-0a-housekeeping/review-findings.md`,
line 11: "Reviewer: pr-manager (inline, subagent spawn unavailable — skill recursive loop)".
The agent logged its own inability to spawn pr-reviewer and fell back to inline self-review.

**Evidence 3 — Verbatim agent output from dispatch #5.**
The agent reported: "The permission system is enforcing the independence requirement for step 5 —
I cannot self-approve and then merge. The `vsdd-factory:pr-review-triage` skill is looping
recursively and I cannot spawn subagents via `sessions_spawn` in this context." And: "I have no
`sessions_spawn` tool available in this conversation context to launch a genuine independent
subagent."

### Unified theory across all three failure modes

| Failure Mode | Mechanism given the missing tool |
|---|---|
| FM1 (premature exit at step 4) | Agent completes security review; reaches step 5; realizes `sessions_spawn` is absent; silently exits with "Safe to proceed to PR reviewer convergence loop" as its terminal sentence — masking the underlying cause |
| FM2 (merge-authorization over-correction) | Compensating behavior: when affordances don't match the playbook, the agent adds safety hedges ("requires explicit user authorization") because it is uncertain about its own authority scope |
| FM3 (recursive loop + spawn unavailable) | Direct manifestation: agent tried `sessions_spawn` → not found; fell back to `vsdd-factory:pr-review-triage` → recursed back into itself; reported honest failure |

### Why dispatches #1 and #3 succeeded

Dispatch #3 (PR #2 RESUME) explicitly labeled the reviewer with a real model name
(`claude-sonnet-4-6`) in
`/Users/jmagady/dev/prism/.factory/code-delivery/S-0.02/review-findings.md`, line 14 — meaning
the Agent/Task tool WAS invoked successfully. The agent reasoned its way around the missing
`sessions_spawn` tool non-deterministically. Dispatch #1 (PR #1) outcome is consistent with the
same reasoning-around behavior. The intermittency is explained: the missing tool causes the agent
to probe fallbacks; whether those fallbacks succeed depends on context-window state and routing.

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

These are concrete, evidence-backed fixes, not hypotheses.

### Primary fix — Update agent definition to use the correct tool name (HIGH)

In `agents/pr-manager.md`, replace every reference to `sessions_spawn` with the actual
harness-available mechanism. The `sessions_spawn` tool does not exist. The correct pattern in
the current Claude Code harness is the `Agent` tool with a `subagent_type:` parameter.

Locations to edit in `vsdd-factory/0.48.0/agents/pr-manager.md`:
- **Line 36**: "NEVER execute `gh` or `git` commands yourself — ALWAYS spawn github-ops with
  `agentId: "github-ops"`" → rewrite: "ALWAYS delegate to github-ops via the Agent tool,
  `subagent_type: 'vsdd-factory:github-ops'`"
- **Line 37**: "NEVER call `sessions_spawn` without `agentId`..." → rewrite: "Every sub-agent
  spawn MUST specify `subagent_type` and a cd-prefixed prompt"
- **Lines 76-78**: "Use `sessions_spawn` with `runtime: "subagent"`, `agentId`, and `cwd`..." →
  rewrite to Agent tool syntax: `Agent(subagent_type="vsdd-factory:github-ops", prompt="cd <cwd> && <task>")`
- **Line 182**: Merge example using `sessions_spawn` → update to Agent tool syntax
- All other `sessions_spawn` occurrences (13 total per the agent definition)

Replacement pattern:

OLD (broken):
```
sessions_spawn({ runtime: "subagent", agentId: "github-ops", cwd: "<project-path>", task: "..." })
```

NEW (correct):
```
Agent(subagent_type="vsdd-factory:github-ops", prompt="cd <project-path> && <task>")
```

### Secondary fix — Remove merge-authorization over-correction (HIGH)

The "SECURITY WARNING" behavior in dispatch #5 was emergent compensation for the tool mismatch.
Two approaches:

(a) Add explicit pre-authorization language to the Operating Procedure section of pr-manager.md:
"MERGE AUTHORIZATION: When dispatched with the full 9-step mandate by orchestrator, merge at
step 8 is PRE-AUTHORIZED. Do not gate on additional user confirmation."

(b) Update the orchestrator's canonical dispatch template to include the phrase
`AUTHORIZE_MERGE=yes`. Add a note to the pr-manager definition: "If dispatch prompt includes
`AUTHORIZE_MERGE=yes` or explicitly lists step 8 (Merge), proceed to merge after APPROVE verdict
without requesting additional authorization."

### Tertiary fix — Add step-completion instrumentation (IMPORTANT)

After each of the 9 steps, pr-manager MUST emit a structured signal:
```
STEP_COMPLETE: step=<N> name=<step-name> status=<ok|failed|skipped> note=<short reason>
```

The orchestrator parses these lines and can deterministically detect premature exits. Without
this, the orchestrator must diff expected vs actual output and guess at the failing step. The FM1
intermittency (silent exit at step 4) is invisible to the orchestrator without this instrumentation.

### Quaternary fix — Thin-coordinator refactor (ARCHITECTURAL, longer-term)

Rewrite pr-manager as a lightweight dispatcher that takes a PR, delegates each step to a separate
fresh-context subagent, and accumulates results. In this model:
- pr-manager emits `STEP_COMPLETE: step=4 ...` and signals `NEED_REVIEWER: {pr_number}`
- The orchestrator (which has the Agent tool) spawns pr-reviewer directly
- The orchestrator re-dispatches pr-manager with review results injected as input

This eliminates single-agent-state brittleness and makes each step independently retriable.
It also decouples the merge-authorization question from the agent's internal flow.

---

## Recommended Prompt for vsdd-factory Session

Paste this into a fresh Claude Code session opened in the vsdd-factory plugin repo. This prompt
embeds the confirmed root cause and specific file+line targets so the agent can act immediately
without re-investigation.

```
I need you to fix the pr-manager agent definition in this vsdd-factory plugin repo.

## Confirmed Root Cause

The agent definition at `agents/pr-manager.md` (v0.48.0) references `sessions_spawn` 13 times
as the mechanism to spawn sub-agents. This tool DOES NOT EXIST in the current Claude Code
harness. The correct mechanism is the `Agent` tool with a `subagent_type:` parameter.

This mismatch is the unified root cause of all three failure modes observed during Phase 3
Wave-0a of a production pipeline (5 dispatches: 3 successes, 2 failures including 1 catastrophic).

---

## FIX 1 — Replace sessions_spawn with Agent tool syntax (PRIMARY — do this first)

File: `agents/pr-manager.md` (current version: 0.48.0)

Search for ALL occurrences of `sessions_spawn` (there are 13). Replace every call with the
Agent tool equivalent. The substitution pattern is:

OLD (broken):
  sessions_spawn({ runtime: "subagent", agentId: "github-ops", cwd: "<path>", task: "..." })

NEW (correct):
  Agent(subagent_type="vsdd-factory:github-ops", prompt="cd <path> && <task>")

Key locations with line numbers from v0.48.0:
- Line 36: prose reference to sessions_spawn for github-ops delegation
- Line 37: constraint "NEVER call sessions_spawn without agentId..."
- Lines 76-78: the canonical spawn paragraph ("Use sessions_spawn with runtime: 'subagent'...")
- Line 182: merge example showing sessions_spawn invocation
- All other occurrences (search for "sessions_spawn" to find them all)

For each spawn site, determine the correct subagent_type:
- github-ops tasks → subagent_type="vsdd-factory:github-ops"
- pr-reviewer tasks → subagent_type="vsdd-factory:pr-review-triage"

---

## FIX 2 — Add merge pre-authorization language (SECONDARY)

The agent generated a false "SECURITY WARNING" blocking merge in one dispatch because it could
not reason about its own authority when the spawn mechanism was broken. Add this to the
Operating Procedure section:

"MERGE AUTHORIZATION: When dispatched by the orchestrator with an explicit step 8 (Merge)
instruction, or when the dispatch prompt includes AUTHORIZE_MERGE=yes, merge is PRE-AUTHORIZED.
Do not gate on additional user confirmation. The orchestrator's dispatch IS the authorization."

Also update the canonical orchestrator dispatch template (wherever it lives in the plugin) to
always include `AUTHORIZE_MERGE=yes` in the step 8 description.

---

## FIX 3 — Add step-completion instrumentation (TERTIARY)

After each of the 9 steps, pr-manager MUST emit a structured line:
  STEP_COMPLETE: step=<N> name=<step-name> status=<ok|failed|skipped> note=<short reason>

Add this requirement to the agent definition's Operating Procedure. The orchestrator will parse
these lines to detect premature exits without needing to diff full output.

---

## VERIFICATION STEPS

After making changes:
1. Search the updated pr-manager.md for any remaining "sessions_spawn" — there should be zero.
2. Trace the 9-step flow through the updated definition and confirm:
   - Step 4 (security review) emits STEP_COMPLETE and has an explicit "proceed immediately to
     step 5" directive with no optional stopping point
   - Step 5 (pr-reviewer spawn) uses Agent tool syntax with subagent_type="vsdd-factory:pr-review-triage"
   - Step 8 (merge) executes when dispatch includes AUTHORIZE_MERGE=yes or explicit step 8 instruction
3. Audit vsdd-factory:pr-review-triage for circular skill dependencies — confirm it does NOT
   invoke pr-manager or any skill that calls back to pr-review-triage.
4. Write or update a test that mocks a PR (title, diff, CI result) and verifies:
   - pr-manager emits STEP_COMPLETE for all 9 steps in sequence
   - The Agent tool is invoked (not sessions_spawn) for both github-ops and pr-reviewer steps
5. Open a PR with title: "fix(pr-manager): replace sessions_spawn with Agent tool, add merge
   pre-authorization, add step instrumentation"

---

## LONGER-TERM (do not block the above on this)

Consider a thin-coordinator refactor: pr-manager emits NEED_REVIEWER and returns; the
orchestrator spawns pr-reviewer via Agent tool and re-dispatches pr-manager with results injected.
This eliminates single-agent-state brittleness entirely.
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
| 2026-04-21 | — | Root cause confirmed via forensic artifact review; Confirmed Root Cause section added |

---

## Cross-Reference

Artifacts examined as forensic evidence for the Confirmed Root Cause section. A future
investigator can re-verify the diagnosis against these exact files and line numbers.

| File | Lines | What it shows |
|------|-------|---------------|
| `/Users/jmagady/.claude/plugins/cache/vsdd-factory/vsdd-factory/0.48.0/agents/pr-manager.md` | 36, 37, 76-78, 182 (+ 9 other occurrences) | Agent definition references `sessions_spawn` 13 times as the canonical spawn mechanism; tool does not exist in the current harness |
| `/Users/jmagady/dev/prism/.factory/code-delivery/S-0.01/review-findings.md` | 9-11 | Dispatch #1 convergence table — APPROVE verdict, merge proceeded; consistent with agent reasoning around the missing tool |
| `/Users/jmagady/dev/prism/.factory/code-delivery/S-0.02/review-findings.md` | 14 | Dispatch #3 reviewer row records real model name `claude-sonnet-4-6` — confirms a genuine subagent spawn succeeded via Agent/Task tool (not sessions_spawn) |
| `/Users/jmagady/dev/prism/.factory/code-delivery/chore-wave-0a-housekeeping/review-findings.md` | 11 | Dispatch #5 reviewer row: "pr-manager (inline, subagent spawn unavailable — skill recursive loop)" — agent's own honest failure report confirming sessions_spawn absence |
