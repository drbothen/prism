---
document_type: process-analysis
title: "State-Manager Burst Latency — Root-Cause Diagnosis (35–70 min/burst)"
producer: state-manager
status: diagnosis-only (not yet remediated)
date: 2026-08-21
cycle: wave-5-e-demo-fidelity
severity: process-gap (performance)
---

# State-Manager Burst Latency — Root-Cause Diagnosis

## Context
State-manager `.factory/` bursts during the S-ADR058-OCSF-ROUTING-001 cascade consistently took 35–70 minutes each (D-2262 ~51m, D-2263 ~51m, D-2264 ~36m, D-2265 ~69m, D-2266 ~48m). This diagnoses why and lists remediation levers. DIAGNOSIS ONLY — nothing here has been implemented.

## Evidence (from `.factory/logs/dispatcher-internal-2026-08-21.jsonl`, 19.7 MB, + hooks-registry rc.23)
- **73 PostToolUse plugins** configured; the whole chain fires on every Edit/Write/Bash call.
- Today: 15,297 PostToolUse plugin events, 8,422 tool calls, **942 timeout/deadline hits**, 40 PostToolUseFailures.
- Top timeout offenders: `validate-table-cell-count` (94), `validate-changelog-monotonicity` (94), `convergence-tracker` (50), then `validate-template-compliance` / `validate-story-bc-sync` / `validate-vp-consistency` / `validate-state-size` / `validate-subsystem-names` / `validate-wave-gate-completeness` (~26 each).
- Timeout budgets: 57 plugins at `timeout_ms=5000`, 14 at `10000`, 1 at `8000`, 1 at `30000`. A timeout burns the full budget before failing open.
- Burst tool-call counts: D-2265 = 101 calls / ~69 min; D-2266 = 45 calls / ~48 min → **~40–85 s per tool call**, far above a normal 1–10 s Read/Edit.

## Root cause (primary)
The 73-plugin PostToolUse hook chain taxes every tool call. Several heavy markdown validators repeatedly re-parse large index/changelog files and exceed their 5–10 s budgets (942 timeouts today), each burning the full budget. A state-manager burst makes dozens of tool calls, so the fixed per-call tax × call-count = 35–70 min. story-writer independently observed "PostToolUse hooks timing out (fail-closed) consistently; writes still succeeded."

## Contributing factors
1. **Bloated index/changelog files (amplifier).** STORY-INDEX/BC-INDEX rows carry long inline `was vX.Y…` changelog chains; STATE.md sits near its size cap. Bigger files → validators exceed budget more often + state-manager's own reads/edits are heavier.
2. **High tool-call count per burst (multiplier).** State-manager is dispatched LAST to verify prior legs + update ~3 indexes + run gates + commit + push — the most tool calls, each paying the hook tax.
3. **`records-lint` L10 corpus-wide scan** (~502 index cells) every commit, plus verify-sha-currency + compute-input-hash subprocesses.
4. **Orphan-branch `.factory` worktree** (3,223 commits) + push round-trip each burst (minor).

## Remediation levers (ranked; NOT YET DONE)
1. **Trim/scope the 73-plugin PostToolUse chain (biggest win).** Gate each validator to only the file types it targets and only on `.factory` Edit/Write — not every tool call. Fix or right-size the chronic timeout offenders (`validate-table-cell-count`, `validate-changelog-monotonicity`).
2. **Compact the bloated index/changelog files.** Move per-row `was vX.Y` history out of STORY-INDEX/BC-INDEX rows into each artifact's own changelog (or cycle archive); keep index rows as one-line current pins.
3. **Cut state-manager's tool-call count.** Tighter briefs (exact edits, no re-read-to-verify); let each specialist finalize its own index row so state-manager only does STATE + commit.
4. **Batch edits** (fewer larger Edits) and **make records-lint L10 incremental** (changed-artifacts only).
5. **Route mechanical commit/index work to a lower-latency model tier.**

Net: #1 and #2 are high-leverage; the hook chain is the dominant cost and file bloat makes it worse.

## Suggested follow-up
Promote levers #1 and #2 to a self-improvement story (or a human-directed tech-debt entry) at cycle close. This artifact is the evidence base.
