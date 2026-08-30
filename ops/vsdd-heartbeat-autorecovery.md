---
document_type: ops-runbook
title: VSDD Orchestrator Heartbeat Auto-Recovery
version: "1.0"
producer: devops-engineer
created: 2026-08-29
status: active
portability: project-agnostic — parameterized by {PLACEHOLDERS}
changelog:
  - version: "1.0"
    date: 2026-08-29
    author: devops-engineer
    summary: Initial creation. Documents the durable CronCreate heartbeat mechanism, its gap-filling rationale, schedule rationale, full prompt template, install steps for new projects, and standing procedure integration for autonomous VSDD pipeline recovery.
---

# VSDD Orchestrator Heartbeat Auto-Recovery Runbook

## 1. Purpose

The VSDD orchestrator runs autonomous pipeline phases that may take hours to complete across many background agent delegations. Background agents already emit an auto-notification to the orchestrator on both completion and failure — so per-agent failures **are** recoverable while the main orchestrator loop is alive and waiting.

The heartbeat covers the **hard-stall case** that the in-session notification loop cannot: the orchestrator's main loop is dead (crashed, timed out, session restart, Claude Code crash), or the session is idle with no pending notification because the triggering notification was lost. In those cases the pipeline just stops. No in-session mechanism can revive it — only an **external durable scheduler** operating independently of the running session can.

**The heartbeat fills that gap.** It is a durable cron job that survives restarts and crashes, fires periodically while the REPL is idle, and runs an idempotent 6-step recovery routine that re-orients and re-dispatches any stalled or failed task.

## 2. Mechanism

### 2.1 CronCreate (durable: true) — the correct tool

The Claude Code REPL exposes two scheduling mechanisms. Only one is appropriate for pipeline recovery:

| Mechanism | Persistence | Behavior | Use for recovery? |
|-----------|-------------|----------|-------------------|
| `ScheduleWakeup` | In-session only — dies when the Claude Code process exits or the session is cleared | Schedules a single wakeup at a future time; useful for next-step reminders within an ongoing session | **NO** — cannot recover a dead session; silently disappears on crash/restart |
| `CronCreate` with `durable: true` | Persisted to `.claude/scheduled_tasks.json` on disk; survives restarts, crashes, and session clears | Runs the prompt on the cron schedule whenever the REPL is idle | **YES** — this is the correct mechanism |

`CronCreate` with `durable: true` and `recurring: true` is the **only mechanism that can externally revive a dead pipeline session**. When the user relaunches Claude Code after a crash or restart, the durable cron is loaded from `.claude/scheduled_tasks.json` and fires on its next scheduled tick while the REPL is idle.

### 2.2 Behavioral constraints

- Crons fire **only while the REPL is idle** — they do not interrupt an active conversation or agent run.
- A cron prompt runs as a full orchestrator context restore, so it must be self-contained and idempotent.
- **Recurring crons auto-expire after 7 days.** The heartbeat prompt must self-re-arm before expiry — see Step 5 of the heartbeat routine.

## 3. Schedule

Recommended schedule: `8,23,38,53 * * * *` (fires at :08, :23, :38, :53 of every hour — every 15 minutes).

**Off-peak-minute rationale:** CronCreate guidance recommends avoiding :00 and :30 minute marks. Those marks attract clustered cron firing across many automated processes (CI jobs, system maintenance, third-party schedulers), and firing at those moments means the heartbeat competes with fleet activity on shared infrastructure. The :08/:23/:38/:53 pattern avoids those collision windows while maintaining even spacing.

**15-minute cadence tradeoff:**

| Cadence | Stall-recovery latency | Prompt-cache cost | Recommendation |
|---------|------------------------|-------------------|----------------|
| 5 min | Very low | High — cache-busting every 5 min for a 3–4 hour run | Too aggressive for most pipelines |
| 15 min | Moderate — worst case ~15 min of lost time | Low — most runs touch the cache once or twice | **Recommended default** |
| 30 min | High — worst case ~30 min of lost time | Very low | Acceptable for low-urgency pipelines |

For time-critical autonomous runs, shorten to `3,18,33,48 * * * *` (every 15 min, different offset) or `*/5 * * * *` (every 5 min, accepting the cache cost).

## 4. Heartbeat Prompt Template

The heartbeat prompt must be filled in at install time with three project-specific values:

- `{PROJECT_PATH}` — absolute filesystem path to the project root (e.g., `/Users/jmagady/Dev/prism`)
- `{OBJECTIVE}` — one-sentence summary of what the pipeline is trying to accomplish (e.g., "deliver Wave 3 multi-tenant stories through Phase 3 TDD implementation to develop")
- `{CONVERGENCE_BAR}` — the pipeline-complete condition to check before declaring no-op (e.g., "all Wave 3 stories merged to develop and STATE.md phase = 3/complete")

### 4.1 Template (parameterized)

```
VSDD ORCHESTRATOR HEARTBEAT — {PROJECT_PATH}

Objective: {OBJECTIVE}
Pipeline-complete condition: {CONVERGENCE_BAR}

This is a durable heartbeat that fires every 15 minutes. You are the VSDD orchestrator.
Run the 6-step idempotent recovery routine below. NEVER duplicate in-flight work.

---

STEP 1 — ORIENT (read state, verify SHA currency)

1a. Run: git -C {PROJECT_PATH} rev-parse HEAD
1b. Run: bash {PROJECT_PATH}/.factory/hooks/verify-sha-currency.sh
    - If it exits non-zero: read the error output, route a state-manager fix-burst to align
      STATE.md `develop_head` with the real HEAD before continuing.
1c. Read {PROJECT_PATH}/.factory/STATE.md (current phase, open tasks, last decision row D-NNN).
1d. Read {PROJECT_PATH}/.factory/SESSION-HANDOFF.md (resume checkpoint, current wave/story).
    Goal: establish the exact pipeline position before touching anything.

STEP 2 — AGENT STATUS CHECK (do not trust memory; verify on-disk state)

2a. Run TaskList to enumerate all background agent tasks and their statuses.
2b. For each task with status=FAILED or status=STALLED (no update in >10 min):
    - Read the on-disk artifact the task was supposed to produce.
    - If the artifact is absent or incomplete: re-dispatch the task with its original brief.
    - If the artifact is present but task is marked FAILED: verify the artifact is correct before
      treating the task as done; re-dispatch if any doubt.
2c. NEVER re-dispatch a task that is RUNNING and producing output. NEVER race a live agent.

STEP 3 — CRITICAL-PATH RESUME (if idle with work remaining)

3a. If TaskList shows zero running agents AND the pipeline is not at {CONVERGENCE_BAR}:
    - Identify the next critical-path step from SESSION-HANDOFF.md.
    - Dispatch the appropriate specialist agent for that step.
    - Log the dispatch in a state-manager burst (update STATE.md with the new decision row).
3b. If the pipeline IS at {CONVERGENCE_BAR}: proceed to Step 4 (no-op path).

STEP 4 — NO-OP IF HEALTHY OR IN-FLIGHT

If agents are running normally and the pipeline is progressing, or if {CONVERGENCE_BAR} is met:
- Do nothing. Log a single line: "HEARTBEAT: pipeline healthy at [timestamp], no action taken."
- Do NOT interrupt running agents.
- Do NOT re-dispatch anything already in-flight.

STEP 5 — SELF-PERPETUATE (re-arm before 7-day expiry)

5a. Run CronList to find this heartbeat cron by its known ID or by matching the schedule
    `8,23,38,53 * * * *` against the project path `{PROJECT_PATH}`.
5b. Check the cron's `created_at` timestamp. If it is within 6 days of expiry (i.e., created
    more than 1 day ago — crons expire after 7 days):
    - Run CronCreate with the same `durable: true`, `recurring: true`, same schedule, and this
      same prompt verbatim with {PROJECT_PATH}, {OBJECTIVE}, and {CONVERGENCE_BAR} filled in.
    - After creating the new cron, delete the old one via CronDelete to avoid duplicate fires.
5c. If the cron is still within its first day, no re-arm needed.

STEP 6 — STATE CHECKPOINT (if STATE.md is stale)

6a. Check the last decision-row timestamp in STATE.md. If more than 2 hours have elapsed
    since the last D-NNN row and the pipeline is actively in-flight:
    - Dispatch state-manager to write a checkpoint row recording: current phase, current
      wave, any agents running, and the heartbeat fire timestamp.
    - State-manager must follow TD-VSDD-053 (single-commit-per-burst) and POL-3.
6b. If STATE.md was updated within the last 2 hours, skip this step.

---

END OF HEARTBEAT ROUTINE.
```

### 4.2 Worked example — prism project (current instance)

The following is the live heartbeat installed for the prism project on 2026-08-29:

- **Cron ID:** `b98bd9dc`
- **Schedule:** `8,23,38,53 * * * *`
- **Created:** 2026-08-29
- **Project path:** `/Users/jmagady/Dev/prism`
- **Objective:** deliver Wave 3 multi-tenant stories and Wave 4 ops stories through Phase 3 TDD implementation to `develop`, converging to Phase 7 release readiness
- **Convergence bar:** all active waves merged to `develop`, STATE.md phase = 7/complete or human-approved release tag pushed

To verify the prism heartbeat is active:

```bash
# List all scheduled tasks — confirm b98bd9dc is present with schedule 8,23,38,53 * * * *
# (Use Claude Code CronList tool — not a shell command)
```

## 5. Install Steps for a New Project

Follow this procedure when bootstrapping a new project onto the VSDD factory pipeline and you need a durable heartbeat.

### 5.1 Check for an existing heartbeat

Before creating a new cron, check whether one already exists for the project.

Using the Claude Code `CronList` tool:
- List all scheduled tasks.
- Search for any entry whose prompt contains `{NEW_PROJECT_PATH}` or whose description references the project name.
- If one exists: verify it is still within its 7-day window; if expiring, delete and re-create per Step 5.2.

### 5.2 Create the durable cron

Using the Claude Code `CronCreate` tool with these parameters:

```
tool: CronCreate
durable: true
recurring: true
schedule: "8,23,38,53 * * * *"
description: "VSDD heartbeat auto-recovery — {PROJECT_NAME}"
prompt: <fill in Section 4.1 template with {PROJECT_PATH}, {OBJECTIVE}, {CONVERGENCE_BAR}>
```

The tool writes the entry to `.claude/scheduled_tasks.json`. Verify by running `CronList` immediately after and confirming the new entry appears.

### 5.3 Record the cron ID in SESSION-HANDOFF.md

After creation, the orchestrator records the cron ID in the project's `SESSION-HANDOFF.md` under a "Heartbeat" section:

```markdown
## Heartbeat

- Cron ID: <id returned by CronCreate>
- Schedule: 8,23,38,53 * * * *
- Installed: {DATE}
- Expires: {DATE + 7 days}
- Re-arm instruction: see .factory/ops/vsdd-heartbeat-autorecovery.md §5
```

### 5.4 Add a standing rule to CLAUDE.md (optional but recommended)

Add a project-level note to the project's `CLAUDE.md` or the standing-rules section of `SESSION-HANDOFF.md`:

```
Standing rule: at every session startup, verify the VSDD heartbeat cron
(see .factory/ops/vsdd-heartbeat-autorecovery.md) is present via CronList.
If absent (expired or never installed), re-arm immediately using the install
procedure in §5 of that runbook before beginning any pipeline work.
```

This ensures that even after a long hiatus (cron expired), the first action of any new session is to restore the safety net before dispatching agents.

## 6. Standing Procedure Integration and Limitations

### 6.1 How it fits into the VSDD operating model

The heartbeat is a **safety net**, not a replacement for the orchestrator's normal operation. The normal loop is:

1. Orchestrator dispatches agent.
2. Agent completes (or fails) and auto-notifies the orchestrator.
3. Orchestrator processes the notification and dispatches the next step.

This loop is fast, event-driven, and efficient. The heartbeat does not participate in the normal loop. It only fires when the REPL is idle — meaning no active conversation, no pending agent notification being processed. In normal operation the heartbeat fires and takes the "no-op if healthy" path (Step 4) on every tick.

The heartbeat recovers from:
- Orchestrator session crash (Claude Code exits unexpectedly)
- Session clear / context window restart
- Lost completion notification (notification arrives while session was not running)
- Orchestrator stall on a long-running phase with no background agent active
- Missed agent failure notification (agent failed but the orchestrator never saw it)

The heartbeat does **not** recover from:
- An agent stuck in a logical loop but technically "running" (TaskList shows it active) — this requires human intervention
- Permanent loss of `.factory/` artifacts without a `factory-artifacts` branch backup
- GitHub API outages preventing PR/merge operations

### 6.2 Key operational constraints

**7-day expiry.** Claude Code recurring crons expire after 7 days automatically. The heartbeat self-re-arms in Step 5, but only when the REPL is idle and the cron actually fires. If a project is dormant for more than 7 days with no Claude Code session at all, the cron expires silently. The standing rule in §5.4 ensures session startup checks for expiry.

**Idle-only firing.** The cron does not interrupt an active agent run or conversation. If the REPL is busy, the tick is skipped and the next one is attempted. This means recovery latency can exceed 15 minutes if the session is continuously active (a good problem — it means the pipeline is running normally).

**Idempotency requirement.** The heartbeat prompt must never produce a duplicate dispatch. Step 2 and Step 3 both guard against this by checking TaskList before dispatching. Any heartbeat prompt that does not include these guards risks creating duplicate agent runs, which can corrupt pipeline state or produce conflicting commits.

**Single-commit-per-burst (TD-VSDD-053).** If the heartbeat triggers state-manager in Step 6, the state-manager dispatch must follow the single-commit-per-burst discipline. The heartbeat does not commit directly; it delegates to state-manager.

**POL-3 (no bypass of `.factory/` hooks).** The heartbeat operates only through Claude Code tools (Agent dispatch, CronCreate, state-manager delegation) — never through direct Python/sed/echo mutations of `.factory/` files.

### 6.3 Portability — lift to vsdd-factory engine

This runbook is designed to be project-agnostic. The {PLACEHOLDER} parameterization in the prompt template is intentional: the heartbeat mechanism is identical across all VSDD projects; only the three parameters differ.

**Intended generalization:** this runbook is a candidate for promotion into the vsdd-factory engine as `HEARTBEAT.md` (alongside `FACTORY.md` and `VSDD.md`) so that every new project initialized by devops-engineer gets the heartbeat out of the box. The install steps in §5 would become part of the `vsdd-factory:repo-initialization` skill, with the cron ID recorded automatically in the generated `SESSION-HANDOFF.md`.

Until that promotion lands, the procedure lives here at `.factory/ops/vsdd-heartbeat-autorecovery.md` and is referenced from the project's `SESSION-HANDOFF.md`.
