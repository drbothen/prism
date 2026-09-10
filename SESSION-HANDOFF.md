---
document_type: session-handoff
level: ops
version: "9.014"
status: current
timestamp: 2026-09-10T12:00:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **D-2516 (2026-09-10): ORG RENAME BURST (TD-VSDD-053) — BOHICA-LABS/prism rename COMPLETE. PR #283 squash-merged to develop@85f30ea7f 2026-09-10; .factory swept 83 occurrences/47 files. develop_head 09e9b28d2→85f30ea7f. records-lint L1/L7/L9/L10 PASS. STATE v10.021→v10.022; SESSION-HANDOFF v9.013→v9.014. §RESUME SNAPSHOT D-2515 SUPERSEDED by D-2516.**

_D-2321..D-2491 delta entries and superseded snapshots archived to cycles/wave-5-e-demo-fidelity/session-handoff-archive.md (D-2494 compaction 2026-09-08). D-2505 snapshot superseded by D-2509. D-2509 snapshot superseded by D-2510. D-2510 snapshot superseded by D-2511. D-2511 snapshot superseded by D-2512. D-2512 snapshot superseded by D-2513. D-2513 snapshot superseded by D-2514. D-2514 snapshot superseded by D-2515. D-2515 snapshot superseded by D-2516._

---

## §RESUME SNAPSHOT — D-2516 (2026-09-10 — ORG RENAME COMPLETE; STATE v10.022) [supersedes D-2515]

### RESUME IN ONE BREATH
Prism at develop@85f30ea7f — org renamed drbothen/prism → BOHICA-LABS/prism COMPLETE (PR #283 squash-merged 2026-09-10). .factory swept exhaustively: 83 occurrences/47 files. Nothing in flight. NEXT: Dependabot #265–#274 triage; RELEASE_PROMOTE_TOKEN check for BOHICA-LABS (non-blocking); branch-protection fix (D-2503, cycle-close/human); remaining v1 feature scope. FRAMING: NOT stable — substantial feature scope remains before v1.0.0 stable.

**RESUME STEP 0:** `CronList` → re-arm heartbeat cron b98bd9dc (8,23,38,53 * * * *) if absent/expired (.factory/ops/vsdd-heartbeat-autorecovery.md).

**RESUME NEXT-ACTION:** Triage Dependabot #265–#274 + verify/close PR #255. Check RELEASE_PROMOTE_TOKEN scope for BOHICA-LABS (non-blocking until next release). Continue v1 feature scope backlog. Branch-protection + pr-manager-completion-guard process-gap (D-2503) is human-gated — carry to cycle-close.

**HEADS (backup boundary):**
- develop HEAD `85f30ea7f` (origin/develop; PR #283 org-rename merged 2026-09-10; nightly LIVE). `main`: `bdf24cec8` (stub, untouched).
- `factory-artifacts`: run `git -C .factory log -1 --format='%h'` for current HEAD (TD-VSDD-053).
- Open PRs: PR #255 (OBSOLETE — verify/close). Dependabot: #265–#274 (UNTRIAGED).
- WORKTREES: REMOVABLE-POST-MERGE: S-REL-NIGHTLY-NOTES-001, E-REL-NOTES (#264), S-CLAROTY-VULNS-001 (#245), S-ENGINE-LIMIT-EARLY-STOP-001 (#243), S-REL-NIGHTLY-001 (#276), S-REL-SPECS-TARBALL-001 (#279). PARKED: S-3.09, W3-FIX-S307-001 (DIRTY do-NOT-touch), S-ENGINE-H2-LARGE-RESPONSE-001.
- Releases: v1.0.0-beta.1 + v1.0.0-beta.2 + v1.0.0-nightly.* (multiple) + edge-nightly.

**PENDING / OPEN ITEMS:**
- **(a) Backlog:** Dependabot #265–#274 triage; verify/close PR #255; Node-24 action-version bumps; rotate test-soc/.mcp.json keys (AD-017); Demo Phase-2 live-monroe Q&A.
- **(b) PROCESS-GAP D-2503 (4th recurrence; D-2515 JUSTIFIED DEFERRAL):** combined candidates (a) adjust develop branch-protection (BOHICA-LABS org may enable this); (b) pr-manager-completion-guard honor orchestrator-scoped partial dispatch — target cycle-close/human.
- **(c) FLAG: RELEASE_PROMOTE_TOKEN** PAT may need re-scope for BOHICA-LABS owner before next release-tag dispatch (non-blocking until next release).
- **(d) v1 feature scope:** remaining stories before stable.

**HEARTBEAT:** durable cron b98bd9dc (8,23,38,53 * * * *) in .claude/scheduled_tasks.json. CLAUDE.md §Orchestrator Auto-Recovery Heartbeat is authoritative standing rule.

**DECISION-LOG DELTA THIS BURST:** D-2516 (ORG RENAME COMPLETE — BOHICA-LABS/prism; PR #283 squash-merged develop@85f30ea7f; .factory swept 83 occurrences/47 files; develop_head 09e9b28d2→85f30ea7f; STATE v10.021→v10.022; SESSION-HANDOFF v9.013→v9.014). All recorded.

**STANDING DECISIONS (carry forward):** (a) No pragmatic convergence / fix all issues (production-grade default). (b) D-989 + D-2445 autonomy grant in force (merge+tag on green gates; force-push any branch still needs human). (c) D-2410 DO NOT SAVE LIVE-TEST OUTPUT INTO REPO. (d) Live xDome validation: canonical runbook .factory/ops/live-tenant-validation-runbook.md. (e) AUTONOMOUS MERGE + TAG (D-2445): all PRs autonomous on green gates; force-push STILL requires explicit human approval. (f) DEFECT-1 RESOLVED by PR #237. (g) POST-v1: TD-SENSOR-SORTBY-PUSHDOWN-001, TD-DI019-RECORDS-CAP-001, TD-CONFIG-SURFACE-EPIC-001. (h) Sensor scope v1: Claroty xDome ONLY (D-2440/D-2443). (i) RELEASING.md at repo root; release-config quality_gates vsdd-partial. (j) No registry publish in v1. (k) Demo bundle + DTU parity DEFERRED post-beta.1. (l) BETA channel; v1.0.0-rc.1 = immutable never-published ghost (D-2452). (m) ADR-063 v1.14 git-cliff hybrid model + §D7 per-channel scoping SHIPPED PR #281; cargo-release 1.1.5 (ADR-064 §D3); PRISM_VERSION COMPLETE; docs VERSION-AGNOSTIC. (n) DEMO-SCOPE.md v2.1; live-monroe-capstone-runbook.md v1.0. (o) S-REL-DOCS-CI-WIRE-001 v0.1. (p) S-REL-HOLDOUT-HARNESS-001 v0.1. (q) Nightly lane LIVE; S-REL-NIGHTLY-NOTES-001 COMPLETE. (r) S-REL-SPECS-TARBALL-001 COMPLETE: PR #279 @54523dccd; all 9 ACs. (s) PROCESS-GAP D-2503 (4th recurrence D-2515): JUSTIFIED DEFERRAL; cycle-close target. (t) S-REL-CHANGELOG-CHANNEL-SCOPE-001 MERGED develop@09e9b28d2 (PR #281 2026-09-10). (u) v1.0.0-beta.2 PRE-RELEASE PUBLISHED (RELEASING.md §6 all 8 PASS; beta.2 workstream COMPLETE). (v) ORG RENAME COMPLETE: BOHICA-LABS/prism (D-2516 2026-09-10; PR #283 develop@85f30ea7f); .factory swept exhaustively; RELEASE_PROMOTE_TOKEN check pending.

---

## Standing Rules (Permanent — Do Not Archive)

### Production-Grade Default
Default behavior is enterprise/production-grade correctness. Speed lives in feature ordering, not feature completeness. See CLAUDE.md §CANONICAL PRINCIPLE for full rule set.

### Pipeline Authority
Orchestrator coordinates all phases; specialist agents do the writing. Orchestrator does NOT write files itself. See CLAUDE.md §Pipeline Authority + Agent Routing Table.

### BC-5.39.001 3-CLEAN Convergence
Adversarial cascades require three consecutive CLEAN(strict) passes for convergence. Any finding resets streak to 0/3. Frozen-HEAD streak rule (DRIFT-ORCH-PRLEVEL-PUSH-001): 3-CLEAN only counts consecutive passes on UNCHANGED HEAD. CLEAN(strict) = ZERO findings of ANY severity. CLEAN(PR-merge) = zero CRIT+HIGH+MED (LOW/OBS non-blocking). Streak requires CLEAN(strict).

### TD-VSDD-053 Single-Commit-Per-Burst
Each logical burst → ONE commit in .factory/. MULTI_COMMIT_CHAIN_NOT_ALLOWED detector blocks consecutive commits containing "backfill"/"Stage 1"/"Stage 2". STATE.md no longer cites the current HEAD SHA — run `git -C .factory log -1 --format='%h %s'` for live HEAD.

### TD-VSDD-091/POL-39 Anti-Volatile-Pin
Narrative spec content must cite function names + behavioral anchors, NOT filename.ext:NNN line numbers. All record-tier text (adversary pass reports, changelog rows, STATE.md entries) MUST use section/symbol/anchor cites ONLY.

### TD-VSDD-097 Three-Dimension Sweep
Every fix-burst MUST explicitly discharge all THREE dimensions: (1) Sibling pair sweep; (2) Downstream copy target sweep; (3) Mandate anchor (every MUST names story + AC + Red Gate test, or cites a real story ID deferral).

### Heartbeat SOP
First action every session: `CronList` → re-arm if absent/expired. Durable recurring cron b98bd9dc (8,23,38,53 * * * *). CLAUDE.md §Orchestrator Auto-Recovery Heartbeat is the authoritative standing rule.
