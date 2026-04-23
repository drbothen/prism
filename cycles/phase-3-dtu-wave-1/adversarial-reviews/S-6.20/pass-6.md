---
document_type: adversarial-review
level: ops
version: "1.5"
status: complete
producer: adversary
timestamp: 2026-04-23T00:00:00
phase: 3
inputs:
  - .factory/stories/S-6.20-dtu-demo-server.md
input-hash: "d0e77ae"
traces_to: prd.md
pass: 6
previous_review: .factory/cycles/phase-3-dtu-wave-1/adversarial-reviews/S-6.20/pass-5.md
story: S-6.20
cycle: phase-3-dtu-wave-1
findings_total: 2
counts:
  critical: 0
  high: 0
  medium: 2
  low: 0
  observation: 0
verdict: BLOCKED
regressions_from_pass_4: 0
regressions_from_pass_5: 1
novel_findings: 1
predecessor_verdict: "BLOCKED (7 findings)"
remediation_landed_in: "v1.5 @ b6ec97e9"
next_action: "story-writer v1.6 remediation (2 MEDIUM); then adversary Pass 7"
convergence_trajectory: "14 → 7 → 2"
---

# Adversarial Review: S-6.20 DTU Demo Server (Pass 6)

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: `P3DTU` (phase-3-dtu-wave-1)
- `<PASS>`: Two-digit pass number
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

## Part A — Fix Verification (Pass 5 → Pass 6)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| ADV-P3DTU-P05-HIGH-001 | HIGH | RESOLVED | Line 999 says "5 files, ~10 LOC"; Task 2 migration table has 5 rows. No leftover "8 files" strings. |
| ADV-P3DTU-P05-HIGH-002 | HIGH | RESOLVED | BLOCKING PREREQUISITE block (lines 219–250) with grep commands. Known Dependencies table (lines 1083–1092) references TD-WV0-05, AC-10, AC-5, R-DEMO-002. |
| ADV-P3DTU-P05-MED-001 | MEDIUM | RESOLVED | Lines 1088–1092 state TD-WV0-05 dependency. |
| ADV-P3DTU-P05-MED-002 | MEDIUM | PARTIALLY_RESOLVED | Task 14 bodies correct; File Structure summary table still shows ~+37. Regression captured as ADV-P3DTU-P06-MED-001 below. |
| ADV-P3DTU-P05-MED-003 | MEDIUM | RESOLVED | TOML schema, struct field, AC-12 all present. |
| ADV-P3DTU-P05-LOW-001 | LOW | RESOLVED | StartReport struct, last_start_report(), assert_port_released helper all specified. AC-11 asserts `cleaned_up_after_failure == vec!["crowdstrike", "claroty", "cyberint"]`. |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

_None._

### HIGH

_None._

### MEDIUM

#### ADV-P3DTU-P06-MED-001: File Structure table LOC values regress MEDIUM-2 remediation

- **Severity:** MEDIUM
- **Category:** contradictions
- **Location:** Story lines 993–994 (File Structure Requirements table)
- **Description:** The File Structure Requirements table still shows `~+37 LOC` for both crowdstrike and claroty Task 14 modifications, contradicting the corrected values written into Task 14 bodies by the v1.5 MEDIUM-2 fix.
- **Evidence:**
  - Task 14 Crate 1 "Net delta" row at line 550: crowdstrike **~+15 LOC** (delete -36, add +45, +5, +1)
  - Task 14 Crate 2 "Net delta" row at line 573: claroty **~+30 LOC** (delete -16, add +40, +5, +1)
  - v1.5 changelog (line 1114) records MEDIUM-2 rewrote Crate 1 to ~+15 and Crate 2 to ~+30
  - File Structure table lines 993/994 still show pre-remediation `~+37` from v1.4
- **Proposed Fix:** Update line 993 → `~+15 LOC`, line 994 → `~+30 LOC`, or add a note stating the File Structure table is an upper bound and the Task 14 net delta is authoritative.

#### ADV-P3DTU-P06-MED-002: Stale line-number citation for ADR-002 Amendment trait-default

- **Severity:** MEDIUM
- **Category:** ambiguous-language
- **Location:** Story line 554 (Task 14 Crate 1); line 577 (Task 14 Crate 2)
- **Description:** Both lines cite "ADR-002 Amendment, lines 759–761" for the trait-default `start()` shim, but lines 759–761 of the story contain AC-7 ("deterministic-logging") content, and ADR-002 itself is only 423 lines total. The Amendment block lives in the story at lines 817–907, with the trait-default `start()` at lines 849–853.
- **Evidence:**
  - `.factory/specs/architecture/decisions/ADR-002-l2-dtu-clone-template.md` has only **423 lines** total
  - Story lines 759–761 are part of **AC-7** ("deterministic-logging"), unrelated to the trait default
  - ADR-002 Amendment block is embedded in the story at lines **817–907**; trait-default `start()` body at line **852**
- **Proposed Fix:** Change line 554 citation to `"(ADR-002 Amendment, this story §'ADR-002 Amendment', lines 849–853)"`. Task 14 Crate 2 (line 577) has no line-number claim — no edit needed there.

### LOW

_None._

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 2 |
| LOW | 0 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision (v1.6)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 6 |
| **New findings** | 1 (ADV-P3DTU-P06-MED-002 — novel mis-anchor) |
| **Duplicate/variant findings** | 1 (ADV-P3DTU-P06-MED-001 — propagation miss, same class as Pass-5 HIGH-1) |
| **Novelty score** | 1 / (1 + 1) = 0.50 |
| **Median severity** | 2.0 (both MEDIUM) |
| **Trajectory** | 14 → 7 → 2 |
| **Verdict** | FINDINGS_REMAIN |

## Source-of-truth verifications

- Crowdstrike `start()` at `crates/prism-dtu-crowdstrike/src/clone.rs` lines 62–97 — 36 LOC ✓
- Claroty `start()` at `crates/prism-dtu-claroty/src/clone.rs` lines 96–111 — 16 LOC ✓
- NVD `/dtu/health` NOT mounted at `crates/prism-dtu-nvd/src/clone.rs:63-70` ✓
- Threatintel `/dtu/reset` and `/dtu/health` NOT mounted at `crates/prism-dtu-threatintel/src/clone.rs:46-53` ✓
- All 6 DTU crates exist per workspace Cargo.toml ✓
- Cyberint `tokio::spawn` line 88, Armis line 105, Threatintel line 71, NVD line 83 ✓

## Convergence trajectory

- Pass 4: 14 findings (2C+5H+5M+2L) → v1.4
- Pass 5: 7 findings (2H+3M+2L, 1 false positive) → v1.5
- Pass 6: 2 findings (2M, 1 regression + 1 novel) → v1.6 pending
- Trend: 14 → 7 → 2 (strong convergence trajectory)
