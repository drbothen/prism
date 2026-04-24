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
  - .factory/cycles/phase-3-dtu-wave-1-5/adversarial-reviews/wave-1-5-integration-gate/pass-1.md
input-hash: "[live-state]"
traces_to: .factory/specs/prd.md
pass: 1
previous_review: null
---

# Adversarial Review: Prism Wave 1.5 Integration Gate (Pass 1)

## Finding ID Convention

Finding IDs use the format: `P3WV15A-A-<SEV>-<SEQ>`

- `P3WV15A`: Phase 3, Wave 1.5, Pass A (first pass)
- `A`: Adversarial (not code-reviewer)
- `<SEV>`: `H` (HIGH), `M` (MEDIUM), `L` (LOW), `OBS` (observation)
- `<SEQ>`: Three-digit sequence

Examples: `P3WV15A-A-H-001`, `P3WV15A-A-M-002`, `P3WV15A-A-L-003`

## Part B — New Findings (Pass 1 — all findings)

### HIGH

#### P3WV15A-A-H-001: `prism-dtu-crowdstrike` silently opts out of workspace `expect_used = "deny"`

- **Severity:** HIGH
- **Category:** code-quality / lint configuration bypass
- **Location:** `prism-dtu-crowdstrike/Cargo.toml` — `[lints.clippy]` block
- **Description:** `prism-dtu-crowdstrike` declares a crate-local `[lints.clippy]` section without `workspace = true`, silently opting out of the workspace-level `expect_used = "deny"` lint. All other crates in the workspace inherit workspace lints via `[lints] workspace = true`; only the CrowdStrike DTU clone bypasses this. `.expect()` calls in this crate are not caught by CI, creating a silent enforcement hole in the security-critical clone fleet.
- **Evidence:** Workspace `Cargo.toml` declares `[workspace.lints.clippy] expect_used = "deny"`. Other crates use `[lints] workspace = true`. `prism-dtu-crowdstrike/Cargo.toml` uses a bare `[lints.clippy]` block without delegating to the workspace, which under Cargo semantics takes precedence and drops all workspace lint inheritance.
- **Proposed Fix:** Replace `prism-dtu-crowdstrike/Cargo.toml`'s `[lints.clippy]` block with `[lints]\nworkspace = true`, or at minimum add `expect_used = "deny"` explicitly inside the crate-local block so the policy is not silently voided.

---

### MEDIUM

#### P3WV15A-A-M-001: 6 production files use file-wide `#![allow(clippy::expect_used)]` blanket suppressions

- **Severity:** MEDIUM
- **Category:** code-quality / blanket lint suppression
- **Location:** 6 production source files (enumeration required by implementer via `grep -r 'allow(clippy::expect_used)' --include='*.rs'`)
- **Description:** Six production source files contain crate-root or file-wide `#![allow(clippy::expect_used)]` inner attributes. These suppress the workspace lint across every line of those files rather than at the specific call site. Blanket suppressions defeat the policy's intent by making all `.expect()` calls in those files invisible to lint enforcement, including any future additions.
- **Evidence:** File-wide inner attribute `#![allow(clippy::expect_used)]` found in 6 files. Site-scoped `#[allow]` on the specific call is acceptable; file-wide suppression is not.
- **Proposed Fix:** Replace each file-wide `#![allow(clippy::expect_used)]` with either: (a) targeted `#[allow(clippy::expect_used)]` at each specific call site with a comment justifying the `.expect()`, or (b) convert the `.expect()` calls to proper error propagation (`?`, `unwrap_or_else`, etc.). File-wide suppressions in production code are not acceptable under the workspace lint policy.

---

#### P3WV15A-A-M-002: `wave-state.yaml` missing `wave_1_5` block; `wave_2.notes` stale

- **Severity:** MEDIUM
- **Category:** spec-fidelity / state tracking drift
- **Location:** `.factory/wave-state.yaml` — `waves:` map; `wave_2.notes` field
- **Description:** The `waves:` map contains entries for `wave_0a`, `wave_0b`, `wave_0c`, `wave_0_retrospective`, and `wave_1`, but no `wave_1_5` block. Wave 1.5 is a documented debt-reduction sprint with 8 merged PRs (#33–#40), 24 resolved TDs, an active gate, and a gate Pass 1 record. Its absence from the `waves:` map is a bookkeeping gap inconsistent with all other waves. Additionally, `wave_2.notes` reads "Awaiting Wave 1 integration gate" — stale since Wave 1 integration gate converged and re-converged 2026-04-23.
- **Evidence:** `grep "wave_1_5" .factory/wave-state.yaml` finds only top-level fields (e.g. `wave_1_5_opened`), never a `waves.wave_1_5:` key. `grep "wave_2:" -A5 .factory/wave-state.yaml` shows stale notes referencing Wave 1 gate only.
- **Proposed Fix (state-manager):** Add `wave_1_5:` block to `waves:` map with gate_status, merged PRs list, tds_resolved count, sprint_completed date, and gate_pass_1 record. Update `wave_2.notes` to reference Wave 1.5 gate dependency and note Wave 1 convergence date.

---

#### P3WV15A-A-M-003: `SESSION-HANDOFF.md` + `STATE.md` factory-artifacts HEAD `0a594cec` stale vs actual `ffe84907`

- **Severity:** MEDIUM
- **Category:** spec-fidelity / SHA currency drift
- **Location:** `.factory/SESSION-HANDOFF.md` (line ~24); `.factory/STATE.md` (Session Resume Checkpoint)
- **Description:** Both `SESSION-HANDOFF.md` and `STATE.md` cite `factory-artifacts HEAD: 0a594cec`. The actual factory-artifacts HEAD at the time of this adversarial pass is `ffe84907`. The previous state-manager burst committed and pushed but did not backfill the SHA in either document, producing stale citations that mislead future sessions about the artifact branch state.
- **Evidence:** `git -C .factory rev-parse HEAD` returns `ffe84907...`; both documents cite `0a594cec`.
- **Proposed Fix (state-manager):** Update factory-artifacts HEAD SHA in both documents using the two-commit protocol: commit 1 with `TBD_backfill`, commit 2 backfills the actual SHA from commit 1.

---

#### P3WV15A-A-M-004: Claroty `/dtu/configure` returns `{"status":"configured"}` vs other 5 clones `{"status":"ok"}`

- **Severity:** MEDIUM
- **Category:** spec-fidelity / DTU clone behavioral inconsistency
- **Location:** `prism-dtu-claroty/src/` — `/dtu/configure` route handler
- **Description:** The Claroty DTU clone's `/dtu/configure` endpoint returns `{"status":"configured"}` while all other 5 clones (CrowdStrike, Cyberint, Armis, ThreatIntel, NVD) return `{"status":"ok"}`. This behavioral inconsistency across the DTU clone fleet is not sanctioned by any ADR or spec. The unified demo harness (S-6.20) and integration tests that pattern-match on the configure response may silently mis-categorize Claroty's response, creating hidden test gaps.
- **Evidence:** Source comparison of `/dtu/configure` handlers across the 6 clone crates shows Claroty uniquely returning `"configured"` rather than `"ok"`. No ADR-003 entry nor S-6.08 spec rationale justifies divergence.
- **Proposed Fix (implementer):** Align Claroty's `/dtu/configure` response body to `{"status":"ok"}` matching the other 5 clones, or document a deliberate specification rationale in ADR-003. Absent a spec rationale, the inconsistency is a defect.

---

### LOW

#### P3WV15A-A-L-001: `STATE.md` `current_step` and `awaiting` pre-date Pass 1 dispatch

- **Severity:** LOW
- **Category:** spec-fidelity / documentation polish
- **Location:** `.factory/STATE.md` — frontmatter `current_step` and `awaiting` fields
- **Description:** `current_step` reads "Wave 1.5 sprint complete. 8 PRs merged. 24 TDs resolved. 1000 tests passing. Adversarial convergence gate next (3-clean-pass minimum per policy)." and `awaiting` reads "Wave 1.5 integration gate adversary — first pass; structural prevention active". These descriptions pre-date Pass 1 and do not reflect that Pass 1 has now run and returned BLOCKED. The fields should be updated to reflect the current state: gate pass 1 complete, BLOCKED, remediation in progress.
- **Proposed Fix (state-manager):** Update `current_step` and `awaiting` to reflect Pass 1 BLOCKED verdict and remediation status.

---

#### P3WV15A-A-L-002: `convergence_window_progress` reads "3 of 3" (Wave 1 re-convergence value) — should reset for Wave 1.5

- **Severity:** LOW
- **Category:** spec-fidelity / documentation polish
- **Location:** `.factory/STATE.md` — frontmatter `convergence_window_progress`
- **Description:** `convergence_window_progress: "3 of 3 clean passes (re-convergence complete)"` is the Wave 1 re-convergence value. For Wave 1.5 gate, the window resets to 0/3. The field should read "0 of 3 clean passes" since Pass 1 is BLOCKED.
- **Proposed Fix (state-manager):** Update `convergence_window_progress` to "0 of 3 clean passes".

---

#### P3WV15A-A-L-003: `convergence_status` does not reflect Pass 1 outcome

- **Severity:** LOW
- **Category:** spec-fidelity / documentation polish
- **Location:** `.factory/STATE.md` — frontmatter `convergence_status`
- **Description:** `convergence_status: "PHASE_3_WAVE_1_5_SPRINT_COMPLETE_AWAITING_GATE_CONVERGENCE"` was accurate before Pass 1. After Pass 1 BLOCKED, it should reflect the remediation-in-progress state.
- **Proposed Fix (state-manager):** Update `convergence_status` to `PHASE_3_WAVE_1_5_GATE_PASS_1_BLOCKED_REMEDIATION_IN_PROGRESS`.

---

#### P3WV15A-A-L-004: `SESSION-HANDOFF.md` Convergence Gate Status table missing Wave 1.5 Pass 1 row

- **Severity:** LOW
- **Category:** spec-fidelity / documentation polish
- **Location:** `.factory/SESSION-HANDOFF.md` — Convergence Gate Status table
- **Description:** The convergence gate table covers Wave 1 passes 1–18 but has no row for Wave 1.5 Pass 1. The table heading reads "Goal: 3 consecutive clean passes — ACHIEVED" (Wave 1 language). Wave 1.5 gate rows should be appended.
- **Proposed Fix (state-manager):** Add Wave 1.5 Pass 1 row to the convergence table in SESSION-HANDOFF.md; update gate goal line for Wave 1.5.

---

#### P3WV15A-A-L-005: `STATE.md` version 5.0 should be bumped after this burst

- **Severity:** LOW
- **Category:** spec-fidelity / documentation versioning
- **Location:** `.factory/STATE.md` — frontmatter `version`
- **Description:** STATE.md is at version "5.0". The state-manager burst that persists Pass 1 findings and remediates M-002/M-003 constitutes a minor update and should bump the version to "5.1".
- **Proposed Fix (state-manager):** Bump `version: "5.0"` to `version: "5.1"`.

---

### OBSERVATION

#### P3WV15A-A-OBS-001: `next_gate_required` top-level field is `wave_1_5_integration_gate_pass_1_pending` — accurate pre-pass, should advance to `pass_2_pending` post-remediation

- **Severity:** OBSERVATION (informational)
- **Location:** `.factory/wave-state.yaml` — top-level `next_gate_required`
- **Description:** Once Pass 1 remediation is committed and Pass 2 is dispatched, `next_gate_required` should be updated to `wave_1_5_integration_gate_pass_2_pending`. No action needed now; state-manager should handle as part of the remediation burst.

---

#### P3WV15A-A-OBS-002: `SESSION-HANDOFF.md` Key Files table references Wave 1 adversarial-reviews path only

- **Severity:** OBSERVATION (informational)
- **Location:** `.factory/SESSION-HANDOFF.md` — Key Files table
- **Description:** The Key Files table cites `.factory/cycles/phase-3-dtu-wave-1/adversarial-reviews/wave-1-integration-gate/` (Pass 1–18 reports). It does not yet reference the new Wave 1.5 cycle directory. Informational; the path will naturally be correct once Pass 2 is dispatched and the directory is populated.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 4 |
| LOW | 5 |
| OBSERVATION | 2 |
| **Total** | **11** |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — iterate after remediation
**Readiness:** Requires remediation before Pass 2. State-manager closes M-002/M-003/L-001/L-002/L-003/L-004/L-005; implementer closes H-001/M-001/M-004.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 11 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (11 / (11 + 0)) |
| **Median severity** | 3.0 (MEDIUM) |
| **Trajectory** | 11 (Pass 1 baseline) |
| **Verdict** | FINDINGS_REMAIN — Pass 1 baseline established; remediation required before Pass 2 |
