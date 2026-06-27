---
name: wrap
description: Capture all in-flight pipeline state into a durable zero-context resume snapshot so the session can be cleared and resumed with no prior context. Use when the user says "wrap", "wrap up", "we're low on context", "make state durable", or before clearing/restarting a session. Produces a SESSION-HANDOFF §RESUME SNAPSHOT + STATE.md update via a single state-manager commit.
---

# /wrap — Session Wrap & Durability Snapshot

## When to use

Low on context / about to clear & restart / end of a work session. Goal: a fresh session resumes with NO prior memory using only the durable snapshot.

Do NOT invoke this skill while new pipeline work is still starting up. The first act is to freeze — see Step 1.

---

## Procedure

### Step 1 — Freeze

Stop launching NEW work. Confirm all in-flight background agents have returned and capture their final results. List any still running and wait for them. Do NOT clear the session with unrecorded agent results — their reports are not durable and will be lost.

### Step 2 — Gather current state (read-only)

Run these commands and record every value:

```bash
# Develop HEAD
git rev-parse origin/develop

# Factory-artifacts HEAD
git -C .factory log --oneline -1

# Active worktrees
ls .worktrees/
# For each active worktree:
git -C .worktrees/<worktree> rev-parse --short HEAD
git -C .worktrees/<worktree> branch --show-current

# Open PRs (confirm none unexpectedly open)
gh pr list

# SHA currency validator
bash .factory/hooks/verify-sha-currency.sh
```

For each active workstream record:
- Branch name + worktree path + frozen HEAD (short SHA).
- Adversarial 3-CLEAN streak: X/3 and the exact frozen HEAD the streak is counted on (BC-5.39.001 + DRIFT-ORCH-PRLEVEL-PUSH-001 frozen-HEAD rule).
- Gate count (e.g., `EXPECTED=82` in ci.yml) and `just check` status (pass / pending / failing).

### Step 3 — Dispatch `vsdd-factory:state-manager` for the snapshot commit

The orchestrator dispatches state-manager; it NEVER writes `.factory/` itself.

State-manager produces ONE atomic commit (TD-VSDD-053). Never `--no-verify`. No multi-commit chain. Commit message form: `chore(factory): session wrap — RESUME SNAPSHOT D-NNN`.

The commit must author or update `SESSION-HANDOFF.md` with a fresh `§RESUME SNAPSHOT D-NNN` section that:

1. Marks the immediately prior snapshot SUPERSEDED (inline annotation in that section header).
2. Opens with a `### RESUME IN ONE BREATH` block — a 3-line human-readable summary of exactly where the project is and what happens next.
3. Records, verbatim and precise:
   - **HEADS** — develop SHA, factory-artifacts SHA, "no agents in flight" confirmation (or list if any residual).
   - **Per WORKSTREAM** — branch, worktree path, frozen HEAD (short SHA), 3-CLEAN streak (X/3) with the frozen HEAD the streak is counted on, gate count, key BCs/specs referenced, and an explicit `RESUME NEXT-ACTION` sentence (the exact next dispatch, e.g., "dispatch vsdd-factory:adversary for pass N on feature/STORY-NNN HEAD abc1234").
   - **Pending USER-approved work** — any fix lists, merge-ordering constraints, or scope approvals granted this session that haven't been started.
   - **Demo / release roadmap remaining** — next milestones in-flight.
   - **Worktree inventory** — active / stale-leave-alone / removable-post-merge classification for each `.worktrees/` entry.
   - **Decision-log delta** — new D-NNN rows added this session not yet in a prior snapshot.
   - **This snapshot's own D-NNN row** — record the wrap itself as a decision entry.

4. Bumps `STATE.md` `version:` field, updates `current_step` and `Session Resume Checkpoint` to point at the new snapshot section.

After authoring the files, state-manager runs:

```bash
bash .factory/hooks/verify-sha-currency.sh   # must exit 0, all PASS
git -C .factory add -A
git -C .factory commit -m "chore(factory): session wrap — RESUME SNAPSHOT D-NNN"
git -C .factory push origin factory-artifacts   # D-1066 standing authorization
```

Then runs the chain-check:
```bash
git -C .factory log --oneline -3   # confirm no MULTI_COMMIT_CHAIN
```

### Step 4 — Confirm to the human

Report back:
- The factory-artifacts commit SHA.
- The "RESUME IN ONE BREATH" text verbatim.
- The phrase **SAFE TO CLEAR.**

---

## Constraints

- The orchestrator NEVER writes `.factory/` itself — snapshot write is delegated to state-manager (Companion Principle §5).
- One atomic commit per TD-VSDD-053. Never `--no-verify`. No multi-commit chain (MULTI_COMMIT_CHAIN_NOT_ALLOWED).
- Do not start new pipeline work during a wrap. Freeze first.
- Force-push to `factory-artifacts` is covered by the D-1066 standing authorization (append-only branch). A force-push to `develop` still requires explicit human approval.
- The frozen-HEAD streak rule (DRIFT-ORCH-PRLEVEL-PUSH-001): do NOT carry over a pre-wrap streak count if any new commits land on the feature branch after the wrap begins. The streak resets to 0/3 on any new push.

---

## Reference

Canonical example: D-1302 in `SESSION-HANDOFF.md` (factory-artifacts commit `8d2d55e6`). A fresh session resuming from a snapshot of that shape should have everything it needs to proceed without any prior context.

See also:
- `.factory/SESSION-HANDOFF.md` — live handoff doc
- `.factory/STATE.md` — pipeline state (version, current_step, resume checkpoint)
- `CLAUDE.md §Factory Hook Diagnostics` — SHA drift and multi-commit-chain recovery procedures
- `CLAUDE.md §Operational Discipline TDs` — TD-VSDD-053 (single-commit-per-burst)
