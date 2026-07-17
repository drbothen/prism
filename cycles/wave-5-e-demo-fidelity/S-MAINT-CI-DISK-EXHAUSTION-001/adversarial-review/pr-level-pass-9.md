---
pass: 9
story: S-MAINT-CI-DISK-EXHAUSTION-001
lane: PR-LEVEL
frozen_head: bd65e93a
clean_strict: false
clean_pr_merge: false
streak_before: 0
streak_after: 0
date: 2026-07-16
---

# S-MAINT-CI-DISK-EXHAUSTION-001 PR-LEVEL Pass 9

**Frozen HEAD:** bd65e93a
**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO
**Streak:** 0/3 (reset per DRIFT-ORCH-PRLEVEL-PUSH-001 — fix-burst-12 code @bd65e93a pushed new HEAD)

## Findings Summary

### HIGH (2)

- **F-MAINT-P9-HIGH-001** — AC-006 sed over-match corrupts third-party sources: the `sed -i` command used to rewrite apt source-list entries matches too broadly and corrupts third-party source-list entries (e.g., `packages.microsoft.com`). The 403-class response from packages.microsoft.com cannot be cured by source-list rewriting — it requires a fundamentally different strategy. Live-demonstrated in CI run 29524703679 attempt-1 where the rewrite broke the runner's third-party package sources.

- **F-MAINT-P9-HIGH-002** — Fallback rewrites to HTTP-only endpoint: the AC-006 mirror fallback target (`https://azure.archive.ubuntu.com`) is HTTP-only; an `https://` URL to this host cannot succeed. The cure path after the `sed` rewrite structurally cannot complete — the fallback host does not serve HTTPS. This means AC-006's two-attempt resilience design is non-functional regardless of source-list state.

### MEDIUM (1)

- **F-MAINT-P9-MED-001** — Run 29524703679 disqualified from AC-005: run 29524703679 had an attempt-1 failure (F-MAINT-P9-HIGH-001/HIGH-002 triggered); green status was achieved only via re-run. AC-005 requires 3 consecutive green `pull_request` CI runs; re-run passes do not satisfy the "consecutive green" requirement. AC-005 stands at 1/3 with this slot vacated.

### LOW (2)

- **LOW-001** — PR-body diff-stat figures false: the PR description quotes diff-stat line counts (e.g., "12 ci.yml sites") that do not match the actual count in the bd65e93a diff. PR body must be refreshed against the actual frozen HEAD diff before merge.

- **LOW-002** — Phantom linux-test residuals in code comment + PR body: references to the old `linux-test` job name (corrected to `test` by fix-burst-12) still appear in a code comment and in the PR body description. Stale job-name references must be swept before merge gate.

### OBSERVATION (2)

- **OBS-001** — PR-body technical inaccuracies: several technical claims in the PR body are factually incorrect with respect to the current ci.yml state (e.g., echo count assertion values, apt strategy description). PR body must be audited for technical accuracy against frozen HEAD content.

- **OBS-002** — Fix-burst numbering mismatch: commit subject says "fix-burst-12" but the narrative in the story and PR body uses "fix-burst-14". Inconsistent numbering creates audit-trail confusion; one authoritative count must be chosen.

## Disposition

All HIGH findings require structural redesign of AC-006 fallback strategy: third-party source-list isolation (remove rather than rewrite) + host-anchored sed + correct http:// scheme for the fallback host. Fix-burst spec layer response: story v0.17→v0.18 @b54af749 (AC-006 fallback redesign: third-party source-list removal + host-anchored sed + http:// scheme; new EC for third-party-source class; RG-5/RG-7 pattern updates; EC-010 rationale corrected). Implementer dispatched to apply v0.18 to ci.yml + e2e.yml (in flight). New frozen HEAD and AC-005 restart will be recorded in next burst.
