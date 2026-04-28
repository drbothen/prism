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
  - .factory/cycles/phase-3-dtu-wave-1-5/adversarial-reviews/wave-1-5-integration-gate/pass-1.md
input-hash: "9bd71ef"
traces_to: .factory/specs/prd.md
pass: 2
previous_review: pass-1.md
---

# Adversarial Review: Prism Wave 1.5 Integration Gate (Pass 2)

## Finding ID Convention

Finding IDs use the format: `P3WV15B-A-<SEV>-<SEQ>`

- `P3WV15B`: Phase 3, Wave 1.5, Pass B (second pass)
- `A`: Adversarial (not code-reviewer)
- `<SEV>`: `H` (HIGH), `M` (MEDIUM), `L` (LOW), `OBS` (observation)
- `<SEQ>`: Three-digit sequence

Examples: `P3WV15B-A-H-001`, `P3WV15B-A-M-002`, `P3WV15B-A-L-003`

## Part A — Fix Verification (Pass 2)

Pass 1 had 11 findings (1H + 4M + 5L + 2OBS). Implementer PR #41 (28a085c9) addressed code findings. State-manager burst addressed state findings.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| P3WV15A-A-H-001 | HIGH | PARTIALLY_RESOLVED | PR #41 fixed CrowdStrike `Cargo.toml` lint delegation — workspace = true added. However blanket `#![allow(clippy::expect_used)]` in 9 source files remains (see H-001 below — regression). |
| P3WV15A-A-M-001 | MEDIUM | PARTIALLY_RESOLVED | PR #41 addressed only 1 of the originally flagged files. 9 files still carry blanket suppressions. Escalated to H-001 this pass. |
| P3WV15A-A-M-002 | MEDIUM | RESOLVED | `waves.wave_1_5` block added to wave-state.yaml; `wave_2.notes` updated. |
| P3WV15A-A-M-003 | MEDIUM | UNRESOLVED | STATE.md `develop_head` still shows `5a2d1c8c`; SESSION-HANDOFF.md and Session Resume Checkpoint cite stale SHAs. Escalated to H-002 this pass. |
| P3WV15A-A-M-004 | MEDIUM | UNRESOLVED | Claroty `/dtu/configure` returning `"configured"` vs `"ok"` — not addressed in PR #41. Carried as M-004 this pass. |
| P3WV15A-A-L-001 | LOW | RESOLVED | `current_step` and `awaiting` updated in STATE.md frontmatter. |
| P3WV15A-A-L-002 | LOW | RESOLVED | `convergence_window_progress` reset to "0 of 3 clean passes". |
| P3WV15A-A-L-003 | LOW | RESOLVED | `convergence_status` updated to `PHASE_3_WAVE_1_5_GATE_PASS_1_BLOCKED_REMEDIATION_IN_PROGRESS`. |
| P3WV15A-A-L-004 | LOW | RESOLVED | Wave 1.5 Pass 1 row added to SESSION-HANDOFF.md convergence table. |
| P3WV15A-A-L-005 | LOW | RESOLVED | STATE.md version bumped 5.0 → 5.1. |
| P3WV15A-A-OBS-001 | OBSERVATION | RESOLVED | `next_gate_required` updated to `pass_2_pending`. |
| P3WV15A-A-OBS-002 | OBSERVATION | RESOLVED | SESSION-HANDOFF.md Key Files updated with Wave 1.5 path. |

**Pass 1 resolution summary:** 7 findings RESOLVED, 2 PARTIALLY_RESOLVED (H-001 and M-001 partial — escalated), 2 UNRESOLVED (M-003 escalated to H-002; M-004 carried forward). Net: 2H regressions remain + 2 new defect classes carried.

---

## Part B — New Findings (Pass 2)

### HIGH

#### P3WV15B-A-H-001: Pass 1 M-001 Regression — 9 Files Still Have Blanket `#![allow(clippy::expect_used)]` Suppressions

- **Severity:** HIGH
- **Category:** code-quality / lint policy regression
- **Location:** 9 production source files — `dtu-crowdstrike/src/lib.rs`, `dtu-cyberint/src/lib.rs`, `dtu-claroty/src/lib.rs`, `dtu-armis/src/lib.rs`, `dtu-nvd/src/lib.rs`, `dtu-threatintel/src/lib.rs`, `dtu-demo-harness/src/main.rs`, `prism-config/src/lib.rs`, `prism-core/src/lib.rs`
- **Description:** PR #41 remediated Pass 1 H-001 (CrowdStrike `Cargo.toml` lint delegation) and partially addressed M-001 (blanket suppressions) — fixing only 1 of the originally flagged files. Nine files still carry top-level `#![allow(clippy::expect_used)]` inner attributes. These blanket suppressions bypass the workspace-level `deny(clippy::expect_used)` policy across the entire file, not just at specific call sites. The workspace policy is therefore still unenforced in 9 crates, making this a direct regression of Pass 1 M-001.
- **Evidence:** `grep -r '#!\[allow(clippy::expect_used)\]' --include='*.rs'` returns 9 files post-PR #41. All 9 are production crates in the DTU clone fleet or core infrastructure. Pass 1 M-001 cited 6 files; the full scope is now confirmed at 9.
- **Proposed Fix:** For each of the 9 remaining files, remove the file-level `#![allow(clippy::expect_used)]` inner attribute and replace with targeted `#[allow(clippy::expect_used)] // rationale: <reason>` annotations at each specific `.expect(...)` call site, or convert the call to proper error propagation. Implementer PR required before Pass 3.

---

#### P3WV15B-A-H-002: Pass 1 M-003 Regression — `STATE.md` `develop_head` + `factory-artifacts HEAD` Stale Post PR #41

- **Severity:** HIGH
- **Category:** spec-fidelity / SHA currency drift regression
- **Location:** `.factory/STATE.md` frontmatter `develop_head`; `.factory/STATE.md` Session Resume Checkpoint; `.factory/SESSION-HANDOFF.md` Current State table
- **Description:** STATE.md frontmatter `develop_head: "5a2d1c8c"` has not been updated to reflect PR #41 merge SHA `28a085c9`. The Session Resume Checkpoint still reads `develop HEAD: 5a2d1c8c`. SESSION-HANDOFF.md Current State table still cites `develop HEAD | 5a2d1c8c` and `factory-artifacts HEAD | fb157080` — both stale. Pass 1 M-003 required SHA currency. The state-manager remediation burst after Pass 1 updated narratives but did not advance the develop SHA fields. This is a direct regression of Pass 1 M-003.
- **Evidence:** `git rev-parse develop` returns `28a085c9...`; STATE.md frontmatter `develop_head: "5a2d1c8c"`. SESSION-HANDOFF.md line 23 reads `develop HEAD | 5a2d1c8c`. The actual factory-artifacts HEAD post Pass 1 remediation burst is also stale at `fb157080` vs. actual current HEAD.
- **Proposed Fix (state-manager):** Update `develop_head: "28a085c9"` in STATE.md frontmatter. Update Session Resume Checkpoint SHA fields. Update SESSION-HANDOFF.md Current State table `develop HEAD` and `factory-artifacts HEAD` rows. Use two-commit protocol for factory-artifacts HEAD backfill.

---

### MEDIUM

#### P3WV15B-A-M-001: STATE Body Narrative Claims Remediation In-Progress; PR #41 Has Merged

- **Severity:** MEDIUM
- **Category:** spec-fidelity / stale narrative
- **Location:** `.factory/STATE.md` — Project Metadata table (Last Updated, Current Phase, Current Step rows); Phase Progress table Wave 1.5 row; Wave 1.5 Gate paragraph; Session Resume Checkpoint TL;DR
- **Description:** Multiple STATE.md body locations still describe the Wave 1.5 gate as "Pass 1 BLOCKED — remediation in progress" with implementer action items pending. PR #41 (28a085c9) has merged. Pass 2 has now run and returned BLOCKED (2H + 4M + 4L + 2OBS). The narrative should reflect: Pass 1 partially remediated (PR #41 merged; 1/10 files fixed, 9 remain per Pass 2 H-001); Pass 2 BLOCKED; Pass 3 pending after implementer closes 9 remaining files.
- **Evidence:** STATE.md line 183 reads "Pass 1 BLOCKED — remediation in progress; Pass 2 pending after implementer closes H-001/M-001/M-004". STATE.md line 269 reads "Remediation in progress. Pass 2 pending after implementer closes H-001/M-001/M-004." Both are stale relative to current state.
- **Proposed Fix (state-manager):** Update all narrative locations to reflect: PR #41 MERGED (28a085c9); Pass 2 ran BLOCKED (12 findings: 2H + 4M + 4L + 2OBS); Pass 3 pending after implementer closes 9 remaining blanket suppressions + crowdstrike Cargo.toml M-004.

---

#### P3WV15B-A-M-002: Current Phase Steps Wave 1.5 Table Missing PR #41 Remediation Row

- **Severity:** MEDIUM
- **Category:** spec-fidelity / audit trail gap
- **Location:** `.factory/STATE.md` — Current Phase Steps — Wave 1.5 table (lines ~204–216)
- **Description:** The Current Phase Steps table jumps from "Wave 1.5 adversarial gate Pass 1" (BLOCKED — REMEDIATION IN PROGRESS) directly to "Wave 1.5 adversarial gate Pass 2" (PENDING). There is no row for the Pass 1 remediation step — the implementer + pr-manager work that produced PR #41 (28a085c9). This is a gap in the audit trail: a substantive remediation step occurred between Pass 1 and Pass 2 and is not recorded in the step-by-step table.
- **Evidence:** The table contains a Pass 1 row (BLOCKED) and a Pass 2 row (PENDING) with no intervening remediation row. PR #41 merged 28a085c9, partially remediating H-001/M-001/M-004 (1 of 10 files fixed; Claroty not addressed).
- **Proposed Fix (state-manager):** Insert a remediation row between Pass 1 and Pass 2: `| Wave 1.5 gate Pass 1 remediation | implementer + pr-manager | COMPLETE | PR #41 (28a085c9); closed H-001 (partial — 1 of 10 files; 9 remain per Pass 2 H-001), M-001 (partial), M-004 (deferred — not addressed), L-001, L-003, L-004, L-005 |`. Update Pass 2 row status from PENDING to BLOCKED.

---

#### P3WV15B-A-M-003: SESSION-HANDOFF.md Agent Routing Lists "Pass 1 (NEXT)"

- **Severity:** MEDIUM
- **Category:** spec-fidelity / stale agent routing
- **Location:** `.factory/SESSION-HANDOFF.md` — Agent Routing table, line ~138
- **Description:** SESSION-HANDOFF.md Agent Routing table row reads `Wave 1.5 adversarial gate Pass 1 (NEXT) | vsdd-factory:adversary`. Pass 1 is complete (BLOCKED). Pass 2 is complete (BLOCKED). The next action is Pass 3, conditioned on implementer closing 9 remaining blanket suppressions + M-004 and state-manager applying SHA/narrative fixes. A session starting from this handoff would incorrectly conclude that Pass 1 is still the next step.
- **Evidence:** `grep "Pass 1 (NEXT)" .factory/SESSION-HANDOFF.md` returns the stale row.
- **Proposed Fix (state-manager):** Update Agent Routing row to `Wave 1.5 adversarial gate Pass 3 (NEXT — after implementer closes H-001 9-file remainder + M-004 + state-manager closes H-002/M-001/M-002/M-003/L-001..L-004) | vsdd-factory:adversary`.

---

#### P3WV15B-A-M-004: `dtu-crowdstrike` `Cargo.toml` `unwrap_used = "allow"` Silently Relaxes Workspace Deny

- **Severity:** MEDIUM
- **Category:** code-quality / lint policy bypass
- **Location:** `prism-dtu-crowdstrike/Cargo.toml` — `[lints.clippy]` table
- **Description:** Even after PR #41 added `workspace = true` to fix the `[lints]` delegation, the `dtu-crowdstrike` crate `Cargo.toml` still carries `unwrap_used = "allow"` in its `[lints.clippy]` table. Under Cargo semantics, a crate-local lint table override takes precedence over the workspace lint value for that specific lint. This means `clippy::unwrap_used` is still silently allowed in the CrowdStrike DTU crate despite the workspace-level deny policy. This is a distinct bypass from the `expect_used` blanket-suppression issue in H-001.
- **Evidence:** `grep -A10 '\[lints.clippy\]' prism-dtu-crowdstrike/Cargo.toml` shows `unwrap_used = "allow"`. The workspace `Cargo.toml` declares `[workspace.lints.clippy] unwrap_used = "deny"`. Cargo lint resolution: crate-level override wins for the specific lint.
- **Proposed Fix (implementer):** Remove `unwrap_used = "allow"` from `prism-dtu-crowdstrike/Cargo.toml`. Replace with site-scoped `#[allow(clippy::unwrap_used)]` annotations at each specific `.unwrap()` call site, or eliminate the calls. Include in same PR as H-001 fix.

---

### LOW

#### P3WV15B-A-L-001: Session Resume Checkpoint Header Still Labeled "pass-1-blocked"

- **Severity:** LOW
- **Category:** spec-fidelity / documentation polish
- **Location:** `.factory/STATE.md` — Session Resume Checkpoint section header (line ~332)
- **Description:** The Session Resume Checkpoint section is labeled `2026-04-24-wave-1-5-gate-pass-1-blocked`. This label was accurate when Pass 1 was the current state. Pass 2 has now run and is BLOCKED. The checkpoint should be replaced with a new checkpoint labeled `2026-04-24-wave-1-5-gate-pass-2-blocked-in-remediation`, and the old `pass-1-blocked` checkpoint archived to `cycles/phase-3-dtu-wave-1-5/session-checkpoints.md`.
- **Proposed Fix (state-manager):** Archive current checkpoint to session-checkpoints.md; write new checkpoint with label `2026-04-24-wave-1-5-gate-pass-2-blocked-in-remediation` reflecting Pass 2 BLOCKED state and Pass 3 preconditions.

---

#### P3WV15B-A-L-002: `wave_1_5_completed` in STATE.md Frontmatter Ambiguous with `wave_1_complete`

- **Severity:** LOW
- **Category:** spec-fidelity / naming ambiguity
- **Location:** `.factory/STATE.md` frontmatter line 74
- **Description:** STATE.md frontmatter contains `wave_1_5_completed: 2026-04-24`. The field name `wave_1_5_completed` is semantically ambiguous: it could mean (a) the sprint body is complete, or (b) the full Wave 1.5 lifecycle including gate convergence is complete. The latter interpretation is false — the gate is BLOCKED. The companion field `wave_1_complete: 2026-04-23` (line 92) uses `_complete` (no `d`) and refers to full lifecycle closure including gate convergence. The inconsistency risks a reader concluding that Wave 1.5 is fully closed. wave-state.yaml already uses the unambiguous `wave_1_5_sprint_completed: 2026-04-24`.
- **Proposed Fix (state-manager):** Rename `wave_1_5_completed` → `wave_1_5_sprint_completed` in STATE.md frontmatter to match wave-state.yaml convention and eliminate semantic collision with `wave_1_complete`.

---

#### P3WV15B-A-L-003: `waves.wave_1_5.gate_pass_1` Missing `remediation_pr` / `remediation_sha` Fields

- **Severity:** LOW
- **Category:** spec-fidelity / incomplete record
- **Location:** `.factory/wave-state.yaml` — `waves.wave_1_5.gate_pass_1` (line ~697)
- **Description:** `waves.wave_1_5.gate_pass_1` record is `{ verdict: BLOCKED, findings: 11, findings_high: 1, findings_medium: 4, findings_low: 5, findings_observation: 2, timestamp: 2026-04-24 }`. All Wave 1 gate pass records include `remediation_pr` and `remediation_sha` fields (e.g., `integration_gate_pass_1: { ..., remediation_pr: 30, remediation_sha: f290f450, ... }`). PR #41 (28a085c9) is the Pass 1 remediation PR and should be recorded in this schema-consistent way.
- **Proposed Fix (state-manager):** Extend the `gate_pass_1` record with `remediation_pr: 41, remediation_sha: 28a085c9, findings_remediated: 7, findings_deferred_in_remediation: 0` to match Wave 1 schema.

---

#### P3WV15B-A-L-004: `wave_1_5_gate_status` Stale at Two Locations

- **Severity:** LOW
- **Category:** spec-fidelity / stale status string
- **Location:** `.factory/wave-state.yaml` — top-level line 9 (`wave_1_5_gate_status`) and `waves.wave_1_5.gate_status` (line ~690)
- **Description:** Two locations in wave-state.yaml reflect the Pass 1 blocked/in-remediation state but not the Pass 2 result:
  1. Top-level: `wave_1_5_gate_status: wave_1_5_integration_gate_pass_1_blocked_remediation_in_progress`
  2. Nested: `waves.wave_1_5.gate_status: sprint_complete_gate_pass_1_blocked_in_remediation`
  Both should reflect that Pass 1 was remediated (PR #41), Pass 2 ran and is BLOCKED, and Pass 3 is pending after implementer closes remaining findings.
- **Proposed Fix (state-manager):** Update both to `wave_1_5_integration_gate_pass_1_remediated_pass_2_blocked_awaiting_pass_3`.

---

### OBSERVATION

#### P3WV15B-A-OBS-001: CHECKLIST Command #8 Does Not Check `develop HEAD` Currency

- **Severity:** OBSERVATION (informational)
- **Location:** `.factory/STATE-MANAGER-CHECKLIST.md` — Pre-Commit Verification command #8
- **Description:** CHECKLIST command #8 checks factory-artifacts HEAD currency in STATE.md and SESSION-HANDOFF.md but does not check `develop_head` currency. Both H-002 in this pass and M-003 in Pass 1 involved stale `develop_head` in STATE.md frontmatter. If command #8 also verified `develop_head` against `git rev-parse develop`, the drift would have been caught pre-commit. This coverage gap has now produced two consecutive regressions.
- **Recommendation:** Extend command #8 to also verify `develop_head` in STATE.md frontmatter and `develop HEAD` in SESSION-HANDOFF.md Current State table against `git rev-parse develop`. See OBS-002 for proposed script.

---

#### P3WV15B-A-OBS-002: Proposed CHECKLIST Command #8 Extension with develop HEAD Check

- **Severity:** OBSERVATION (informational)
- **Location:** `.factory/STATE-MANAGER-CHECKLIST.md` — Pre-Commit Verification command #8
- **Description:** Proposed replacement for CHECKLIST command #8 that covers both factory-artifacts HEAD and develop HEAD currency across both STATE.md and SESSION-HANDOFF.md:

```bash
# Command #8 extended: factory-artifacts HEAD AND develop HEAD currency check
ACTUAL_FA=$(git -C .factory rev-parse HEAD)
ACTUAL_DEV=$(git rev-parse develop)
CITED_FA_STATE=$(grep -oE 'factory-artifacts HEAD:? ?`?[0-9a-f]{8}' .factory/STATE.md | head -1 | grep -oE '[0-9a-f]{8}$')
CITED_DEV_STATE=$(grep -oE 'develop_head: "?[0-9a-f]{8}' .factory/STATE.md | head -1 | grep -oE '[0-9a-f]{8}$')
CITED_FA_HANDOFF=$(grep -oE 'factory-artifacts HEAD:? ?\|? ?`?[0-9a-f]{8}' .factory/SESSION-HANDOFF.md | head -1 | grep -oE '[0-9a-f]{8}$')
CITED_DEV_HANDOFF=$(grep -oE 'develop HEAD:? ?\|? ?`?[0-9a-f]{8}' .factory/SESSION-HANDOFF.md | head -1 | grep -oE '[0-9a-f]{8}$')
[ "${ACTUAL_FA:0:8}" = "$CITED_FA_STATE" ] && [ "${ACTUAL_FA:0:8}" = "$CITED_FA_HANDOFF" ] \
  && [ "${ACTUAL_DEV:0:8}" = "$CITED_DEV_STATE" ] && [ "${ACTUAL_DEV:0:8}" = "$CITED_DEV_HANDOFF" ] \
  || echo "STALE SHA drift detected"
```

**Note on two-commit protocol:** When using the two-commit SHA backfill protocol, commit 2's SHA will always be one ahead of the SHA commit 2 cites (commit 2 was written during commit 1's preparation context). This is expected — document as a known exception in the checklist rather than treating it as a false positive.

- **Recommendation:** State-manager adopts this command as the replacement for command #8.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 4 |
| LOW | 4 |
| OBSERVATION | 2 |
| **Total** | **12** |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — 2 HIGH regressions (H-001 blanket suppressions in 9 files; H-002 SHA drift) plus 4M + 4L + 2OBS. Iterate after remediation.
**Readiness:** Requires remediation before Pass 3. Implementer closes H-001 (9 remaining files) + M-004 (crowdstrike Cargo.toml). State-manager closes H-002 + M-001 + M-002 + M-003 + L-001 + L-002 + L-003 + L-004 + OBS-001/002.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 |
| **New findings** | 2 (M-004 carried from P1 as new P2 instance; OBS-001/002 truly new) |
| **Duplicate/variant findings** | 10 (H-001 = P1-M-001 regression; H-002 = P1-M-003 regression; M-001/M-002/M-003 = state drift variants of P1 class; L-001/L-002/L-003/L-004 = bookkeeping variants) |
| **Novelty score** | 0.17 (2 / (2 + 10)) |
| **Median severity** | 2.5 (between MEDIUM and LOW) |
| **Trajectory** | 11→12 (Pass 1 → Pass 2; regression: partial remediation opened 2 HIGH regressions) |
| **Verdict** | FINDINGS_REMAIN — regressions detected; partial remediation insufficient; Pass 3 required after full remediation |
