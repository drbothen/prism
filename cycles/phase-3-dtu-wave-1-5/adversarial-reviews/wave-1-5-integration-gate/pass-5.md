---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-24T00:00:00
phase: 3
inputs:
  - .factory/STATE.md
  - .factory/wave-state.yaml
  - .factory/SESSION-HANDOFF.md
  - .factory/STATE-MANAGER-CHECKLIST.md
  - .factory/hooks/verify-sha-currency.sh
  - .factory/cycles/phase-3-dtu-wave-1-5/adversarial-reviews/wave-1-5-integration-gate/pass-1.md
  - .factory/cycles/phase-3-dtu-wave-1-5/adversarial-reviews/wave-1-5-integration-gate/pass-2.md
  - .factory/cycles/phase-3-dtu-wave-1-5/adversarial-reviews/wave-1-5-integration-gate/pass-3.md
  - .factory/cycles/phase-3-dtu-wave-1-5/adversarial-reviews/wave-1-5-integration-gate/pass-4.md
input-hash: "9bd71ef"
traces_to: .factory/specs/prd.md
pass: 5
previous_review: pass-4.md
---

# Adversarial Review: Prism Wave 1.5 Integration Gate (Pass 5)

## Finding ID Convention

Finding IDs use the format: `P3WV15E-A-<SEV>-<SEQ>`

- `P3WV15E`: Phase 3, Wave 1.5, Pass E (fifth pass)
- `A`: Adversarial (not code-reviewer)
- `<SEV>`: `H` (HIGH), `M` (MEDIUM), `L` (LOW), `OBS` (observation)
- `<SEQ>`: Three-digit sequence

---

## Part A — Fix Verification (Pass 5)

Pass 4 had 10 findings (2H + 4M + 2L + 2OBS). State-manager remediation burst at `3e2359ac` (Stage 1) + `d603c83a` (Stage 2 tense-flip + SHA backfill) + `4508234a` (hook grep fix) + `105c5b17` (SHA backfill for 4508234a) attempted to close all findings. This burst extended to 4 commits — a burst chain violation. Verification of Pass 4 closures below.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| P3WV15D-A-H-001 | HIGH | PARTIALLY_RESOLVED | Stage 1 (`3e2359ac`) and Stage 2 (`d603c83a`) correctly updated factory-artifacts HEAD citations in STATE.md and SESSION-HANDOFF.md to `d603c83a`. However, two subsequent commits (`4508234a` hook-fix + `105c5b17` backfill) extended the chain to 4 commits. STATE.md Session Resume Checkpoint and body now cite `4508234a` (8+ locations), SESSION-HANDOFF.md cites `4508234a` (2 locations), but actual factory-artifacts HEAD is `105c5b17`. SHA-drift recurs (5th recurrence). Re-escalated as H-001 this pass. |
| P3WV15D-A-H-002 | HIGH | RESOLVED | All 17+ "in progress" narrative locations were converted to past tense in Stage 2 (`d603c83a`). No "in progress", "this burst remediates", or "remediation in progress" language remains in STATE.md, SESSION-HANDOFF.md, or wave-state.yaml. |
| P3WV15D-A-M-001 | MEDIUM | RESOLVED | `gate_pass_3.remediation_sha` backfilled to `b1b145b3`; `findings_remediated: 8` recorded; all null fields filled. |
| P3WV15D-A-M-002 | MEDIUM | RESOLVED | STATE.md Pass 3 frontmatter entry now includes `remediated: 8, remediation_sha: b1b145b3, remediation_pr: null`. |
| P3WV15D-A-M-003 | MEDIUM | RESOLVED | SESSION-HANDOFF.md Key Files row updated to `(v5.4)`. |
| P3WV15D-A-M-004 | MEDIUM | RESOLVED | wave-state.yaml notes narrative extended with Pass 3 + Pass 4 paragraphs. `gate_pass_4` record added. |
| P3WV15D-A-L-001 | LOW | RESOLVED | SESSION-HANDOFF.md convergence table now includes Pass 3 remediation separator row and WV1.5-4 BLOCKED row. |
| P3WV15D-A-L-002 | LOW | RESOLVED | Session Resume Checkpoint priority #1 uses outcome-neutral "if CLEAN / if BLOCKED" framing for Pass 5. |
| P3WV15D-A-OBS-001 | OBSERVATION | PARTIALLY_RESOLVED | Hook tightened: `HEAD_IS_BACKFILL` check added (commit message must contain "backfill"). However, the hook still passes when HEAD^ also has "backfill" in its message (multi-commit chain). The 4-commit burst (`3e2359ac→d603c83a→4508234a→105c5b17`) passes the hook's exception because `105c5b17` contains "backfill" and `4508234a` == HEAD^. The hook cannot distinguish a legitimate 2-commit protocol from a 4-commit chain extension. Re-raised as M-004 this pass. |
| P3WV15D-A-OBS-002 | OBSERVATION | RESOLVED | STATE-MANAGER-CHECKLIST.md command #8 now references the canonical hook script. |

**Pass 4 resolution summary:** 7 findings RESOLVED (H-002, M-001, M-002, M-003, M-004, L-001, L-002, OBS-002), 2 PARTIALLY_RESOLVED (H-001 — SHA-drift recurs due to 4-commit chain; OBS-001 — hook tightened but multi-commit chain detection missing). The defining failure is that the burst extended to 4 commits, creating 3 different intermediate SHAs (`3e2359ac`, `d603c83a`, `4508234a`) that were each cited in documents before the final HEAD (`105c5b17`) was ever cited anywhere.

---

## Part B — New Findings (Pass 5)

### HIGH

#### P3WV15E-A-H-001: SHA Drift — 5th Recurrence; Burst Chain Extended to 4 Commits Creating Multi-SHA Citation Fragmentation

- **Severity:** HIGH
- **Category:** spec-fidelity / SHA currency drift regression
- **Location:** Multiple: `STATE.md` lines 187, 188, 189, 204, 281, 348, 350 (cite `d603c83a` or `4508234a`); `SESSION-HANDOFF.md` lines 15, 24, 32 (cite `3e2359ac` or `4508234a`); actual factory-artifacts HEAD = `105c5b17` (cited nowhere)
- **Description:** The Pass 4 remediation burst extended to 4 commits instead of the required 2: Stage 1 (`3e2359ac`) → Stage 2 (`d603c83a`) → hook-fix (`4508234a`) → SHA-backfill-of-hook-fix (`105c5b17`). Each intermediate commit was cited in documents at the time of writing, producing 3 different SHA cites for the same conceptual event. The actual factory-artifacts HEAD `105c5b17` is cited in no document. This is the 5th consecutive adversarial pass to catch the SHA-drift defect class:
  - Pass 1 (M-003): develop HEAD stale
  - Pass 2 (H-002): factory-artifacts HEAD stale at multiple locations
  - Pass 3 (H-001): factory-artifacts HEAD stale at 6 locations
  - Pass 4 (H-001): factory-artifacts HEAD stale — Stage 2 never completed
  - Pass 5 (H-001): actual HEAD `105c5b17` cited nowhere; 3 different intermediate SHAs cited across documents
- **Evidence:** `git -C .factory rev-parse HEAD` = `105c5b17`. STATE.md Session Resume Checkpoint line 350: `factory-artifacts HEAD: 4508234a`. STATE.md line 187: `factory-artifacts d603c83a`. SESSION-HANDOFF.md line 24: `factory-artifacts HEAD | 4508234a`. SESSION-HANDOFF.md line 32: `factory-artifacts 3e2359ac`. `105c5b17` appears in none of these locations.
- **Root cause:** SHA cites were applied DURING the burst (when the SHA chain was unfolding), not AFTER the burst (when only ONE final SHA exists). The discipline of writing `15fa97e6` in Stage 1 and replacing with the single canonical SHA in Stage 2 was not followed. Instead, each commit updated documents with its predecessor's SHA, creating a chain of citations.
- **Proposed Fix:** Follow the SINGLE CANONICAL SHA discipline: Stage 1 writes all fixes using `15fa97e6` placeholder everywhere; Stage 2 replaces ALL occurrences of `15fa97e6` with Stage 1's actual SHA via global substitution. NO third commit. The "burst HEAD" is Stage 1's SHA — the single canonical value for all documents in this burst.

---

### MEDIUM

#### P3WV15E-A-M-001: STATE.md Cites `d603c83a` as Canonical "Pass 4 Remediation HEAD" (8+ Locations)

- **Severity:** MEDIUM
- **Category:** spec-fidelity / stale SHA cite
- **Location:** `STATE.md` lines 187, 188, 189, 204, 281 — body narrative; line 75 frontmatter `adversary_wave_1_5_gate_pass_4_wave_integration_gate.remediation_sha`
- **Description:** After the 4-commit burst, STATE.md retained `d603c83a` as the canonical "Pass 4 remediation HEAD" at 8+ locations. `d603c83a` was Stage 2 of the burst — correct at time of writing — but superseded by `4508234a` and then `105c5b17`. All body narrative references to "factory-artifacts d603c83a" and all frontmatter `remediation_sha: d603c83a` fields are stale by 2 commits.
- **Evidence:** STATE.md line 187: `...factory-artifacts d603c83a (Stage 1: 3e2359ac + Stage 2 tense-flip: d603c83a)...`. STATE.md line 75: `remediation_sha: d603c83a`. Actual HEAD: `105c5b17`.
- **Proposed Fix:** Replace all `d603c83a` citations for "Pass 4 remediation HEAD" with the single canonical burst SHA (Stage 1 of this burst). Note: the body narrative MAY describe the chain history without SHAs ("Stage 1 wrote fixes; Stage 2 tense-flipped 17+ locations; hook fix + SHA backfill followed") but the canonical remediation_sha field must cite the burst HEAD.

---

#### P3WV15E-A-M-002: STATE.md Cites `4508234a` as "factory-artifacts HEAD" in Session Resume Checkpoint (2 Locations)

- **Severity:** MEDIUM
- **Category:** spec-fidelity / stale SHA cite
- **Location:** `STATE.md` lines 348, 350 — Session Resume Checkpoint
- **Description:** The Session Resume Checkpoint TL;DR (line 348) and the metric row (line 350) cite `factory-artifacts HEAD: 4508234a`. This is the hook-fix intermediate commit, not the actual HEAD (`105c5b17`). The Session Resume Checkpoint is the primary document read at session start — stale SHA here will mislead the next agent.
- **Evidence:** STATE.md line 350: `**develop HEAD:** e45159b9 | **factory-artifacts HEAD:** \`4508234a\``. Actual HEAD: `105c5b17`.
- **Proposed Fix:** Replace Session Resume Checkpoint `factory-artifacts HEAD` with the single canonical burst SHA.

---

#### P3WV15E-A-M-003: SESSION-HANDOFF.md Cites `3e2359ac` as "Pass 4 Remediation" in Two Locations

- **Severity:** MEDIUM
- **Category:** spec-fidelity / stale SHA cite
- **Location:** `SESSION-HANDOFF.md` lines 8 (frontmatter `successor_focus:`) and 32 (Gate status table row)
- **Description:** SESSION-HANDOFF.md frontmatter `successor_focus:` reads "Pass 4 remediated at 3e2359ac" — citing Stage 1 SHA of the Pass 4 burst. The Gate status table row also references `3e2359ac` as the remediation point. `3e2359ac` is Stage 1 of a 4-commit chain; the canonical remediation event is the full burst, not the Stage 1 intermediate commit.
- **Evidence:** SESSION-HANDOFF.md line 8: `successor_focus: "Wave 1.5 gate Pass 5 — adversary review of remediated state (Pass 4 remediated at 3e2359ac)"`. SESSION-HANDOFF.md line 32: `| Gate status | Wave 1.5 gate Pass 4 REMEDIATED — factory-artifacts 3e2359ac...`.
- **Proposed Fix:** Replace all `3e2359ac` citations with the single canonical burst SHA.

---

#### P3WV15E-A-M-004: Hook Does Not Detect Multi-Commit Chain — Fires Exception Incorrectly for 4-Commit Burst

- **Severity:** MEDIUM
- **Category:** structural prevention / hook correctness
- **Location:** `.factory/hooks/verify-sha-currency.sh` lines 92–108
- **Description:** The hook's two-commit-protocol exception checks: (a) cited SHA == HEAD^ AND (b) HEAD commit message contains "backfill". Commit `105c5b17` satisfies both conditions: its message contains "backfill" and HEAD^ is `4508234a`. The hook reports NOTE (exception granted) and exits PASS — even though HEAD^^'s message (`4508234a`: "backfill final factory-artifacts SHA") ALSO contains "backfill", indicating a chain extension beyond 2 commits. A legitimate 2-commit protocol has exactly: HEAD (backfill) + HEAD^ (fixes, no "backfill"). In the current 4-commit chain, HEAD^^ and HEAD^^^ also contain "backfill" or fix language — but the hook only looks back 1 commit.
- **Evidence:** `git -C .factory log --oneline -4`: `105c5b17 state: backfill...`, `4508234a state: backfill...`, `d603c83a state: Stage 2 — backfill...`, `3e2359ac state: Wave 1.5 gate Pass 4 BLOCKED...`. Hook exits PASS. Actual HEAD `105c5b17` not cited in any document.
- **Proposed Fix:** Add HEAD^^ check: if HEAD^'s commit message also contains "backfill", report FAIL with "MULTI_COMMIT_CHAIN_NOT_ALLOWED — only exactly 2 commits permitted (1 fix + 1 backfill)". This prevents the exception from granting false PASS for 3+ commit chains.

---

#### P3WV15E-A-M-005: wave-state.yaml `merged_prs` List Missing PR #42; Inconsistent With STATE.md

- **Severity:** MEDIUM
- **Category:** spec-fidelity / incomplete data
- **Location:** `wave-state.yaml` line 694, `wave_1_5.merged_prs:`
- **Description:** wave-state.yaml `wave_1_5.merged_prs: [33, 34, 35, 36, 37, 38, 39, 40, 41]` — PR #42 (gate Pass 2 code remediation, `e45159b9`) is missing. STATE.md and SESSION-HANDOFF.md both reference PR #42 as part of Wave 1.5 remediation. The list is also inconsistent with `PR count merged: 42` in SESSION-HANDOFF.md. The complete Wave 1.5 PR list should include all sprint PRs (#33-#40) plus gate remediation PRs (#41, #42).
- **Evidence:** wave-state.yaml line 694: `merged_prs: [33, 34, 35, 36, 37, 38, 39, 40, 41]`. STATE.md line 80: `wave_1_5_prs_merged: [33, 34, 35, 36, 37, 38, 39, 40]` (also missing #41 and #42). SESSION-HANDOFF.md line 30: "Wave 1.5 PRs | 8 merged (#33 PR-A, #34 PR-A.1, #35 PR-B, #36 PR-C, #37 PR-D, #38 PR-D.1, #39 PR-E, #40 PR-F)" — also missing gate remediation PRs.
- **Proposed Fix:** Standardize to `merged_prs: [33, 34, 35, 36, 37, 38, 39, 40, 41, 42]` in wave-state.yaml and STATE.md `wave_1_5_prs_merged`. Update SESSION-HANDOFF.md Wave 1.5 PR summary to "10 merged (#33-#40 sprint + #41 Pass 1 rem + #42 Pass 2 code rem)".

---

### LOW

#### P3WV15E-A-L-001: SESSION-HANDOFF Convergence Table Missing Pass 4 Remediation Row and WV1.5-5 Row

- **Severity:** LOW
- **Category:** spec-fidelity / incomplete audit trail
- **Location:** SESSION-HANDOFF.md "Convergence Gate Status — Wave 1.5" table
- **Description:** The Wave 1.5 convergence table ends at `WV1.5-4 | BLOCKED | 10 | ...`. It is missing two rows:
  - A Pass 4 remediation separator row documenting the 2-stage + hook-fix + SHA-backfill chain
  - A WV1.5-5 row: `WV1.5-5 | BLOCKED | 11 | 5th SHA-drift recurrence; 4-commit chain extension; actual HEAD 105c5b17 cited nowhere`
- **Evidence:** SESSION-HANDOFF.md convergence table ends at WV1.5-4 with no subsequent rows.
- **Proposed Fix:** Add both missing rows.

---

#### P3WV15E-A-L-002: STATE.md Pass 4 Frontmatter Entry Has Wrong `remediation_sha`

- **Severity:** LOW
- **Category:** spec-fidelity / schema field accuracy
- **Location:** `STATE.md` frontmatter line 75, `adversary_wave_1_5_gate_pass_4_wave_integration_gate.remediation_sha`
- **Description:** The Pass 4 frontmatter entry records `remediation_sha: d603c83a`. This was Stage 2 of the burst at the time of writing but the full burst extended to `4508234a` and then `105c5b17`. The canonical remediation SHA for the Pass 4 burst (the single value that identifies "Pass 4 remediation complete") should be the burst HEAD, not an intermediate commit.
- **Evidence:** STATE.md line 75: `remediation_sha: d603c83a`. Actual factory-artifacts HEAD: `105c5b17`.
- **Proposed Fix:** Update `remediation_sha` in the Pass 4 frontmatter entry to the single canonical burst SHA for this Pass 5 remediation (which will retroactively close Pass 4 with the correct "final" SHA). Note: this is acceptable because the canonical SHA is the burst HEAD that resolves all Pass 4 + Pass 5 findings simultaneously.

---

### OBSERVATION

#### P3WV15E-A-OBS-001: STATE-MANAGER-CHECKLIST.md Does Not Document Single Canonical SHA Rule

- **Severity:** OBSERVATION
- **Category:** structural prevention
- **Location:** `STATE-MANAGER-CHECKLIST.md` — SHA backfill protocol section
- **Description:** The checklist describes the two-commit protocol (Option A/B) but does not explicitly state the "single canonical SHA" discipline: that a burst MUST use exactly one placeholder (`15fa97e6`) throughout all documents in Stage 1, and Stage 2 replaces that placeholder globally with the single Stage 1 SHA. Without this rule in writing, state-managers apply SHAs document-by-document as they write, creating per-document SHA chains. The 5 consecutive SHA-drift recurrences all have this root cause.
- **Suggested improvement:** Add a "Single Canonical SHA Rule" subsection explicitly stating: (1) Stage 1 writes `15fa97e6` everywhere a SHA is needed; (2) Stage 2 performs a global replacement of `15fa97e6` with Stage 1's actual SHA; (3) NO third commit; (4) if a fix is needed post-commit-2, `git reset --soft HEAD~2` and redo from Stage 1.
- **Note:** Informational. The 2-commit protocol as described is correct in concept but incomplete in execution guidance.

---

#### P3WV15E-A-OBS-002: Hook `cat-file` Fabrication Check Not Present

- **Severity:** OBSERVATION
- **Category:** structural prevention / hook completeness
- **Location:** `.factory/hooks/verify-sha-currency.sh`
- **Description:** The hook verifies that cited SHAs are current (match actual HEAD) but does not verify that cited SHAs actually EXIST in the factory-artifacts git object store. A fabricated SHA (one that passes length/pattern checks but does not exist as a git object) would not be caught. Adding `git -C .factory cat-file -e <cited_sha>` before the comparison would distinguish FABRICATED from STALE SHAs and produce a more precise error message.
- **Suggested improvement:** Add a `git -C "$FACTORY_DIR" cat-file -e "$CITED_FA_STATE"^{commit} 2>/dev/null || echo "WARN: STATE.md cited factory-artifacts SHA $CITED_FA_STATE does not exist as a git object (FABRICATED?)"` check before the comparison block.
- **Note:** Informational.

---

## Summary

| Severity | Count | IDs |
|----------|-------|-----|
| HIGH | 2 | P3WV15E-A-H-001 (5th recurrence SHA-drift), P3WV15E-A-H-001 sub-finding (multi-SHA fragmentation across 3 intermediate commits) |
| MEDIUM | 5 | P3WV15E-A-M-001, P3WV15E-A-M-002, P3WV15E-A-M-003, P3WV15E-A-M-004, P3WV15E-A-M-005 |
| LOW | 2 | P3WV15E-A-L-001, P3WV15E-A-L-002 |
| OBSERVATION | 2 | P3WV15E-A-OBS-001, P3WV15E-A-OBS-002 |
| **TOTAL** | **11** | |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — 2H; clean pass requires 0H, 0C
**Readiness:** Requires remediation; Single Canonical SHA discipline + hook multi-commit-chain detection + 2-commit-only protocol MANDATORY for Pass 6

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 |
| **New findings** | 4 (M-004 hook multi-commit-chain detection gap; M-005 merged_prs list inconsistency; L-002 Pass 4 frontmatter wrong SHA; OBS-002 fabrication check) |
| **Duplicate/variant findings** | 7 (H-001 = 5th recurrence of SHA-drift class; M-001 = stale d603c83a cites in STATE.md body; M-002 = stale 4508234a in Session Resume Checkpoint; M-003 = stale 3e2359ac in SESSION-HANDOFF.md; L-001 = audit-trail gap recurrence; OBS-001 = hook improvement carried from Pass 4 OBS-001) |
| **Novelty score** | 4 / (4 + 7) = 0.36 |
| **Median severity** | 3.0 (MEDIUM) |
| **Trajectory** | 11→12→10→10→11 |
| **Verdict** | FINDINGS_REMAIN — 5th consecutive SHA-drift recurrence; root cause is SHA-during-burst discipline failure, not hook failure. Single Canonical SHA discipline is the only structural fix. |
