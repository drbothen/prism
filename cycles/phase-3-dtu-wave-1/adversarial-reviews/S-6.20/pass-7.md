---
document_type: adversarial-review
level: ops
version: "1.6"
status: complete
producer: adversary
timestamp: 2026-04-23T00:00:00
phase: 3
inputs:
  - .factory/stories/S-6.20-dtu-demo-server.md
input-hash: "621e8fe"
traces_to: prd.md
pass: 7
previous_review: .factory/cycles/phase-3-dtu-wave-1/adversarial-reviews/S-6.20/pass-6.md
story: S-6.20
cycle: phase-3-dtu-wave-1
findings_total: 2
counts:
  critical: 0
  high: 0
  medium: 0
  low: 1
  observation: 1
verdict: CONVERGED
regressions_from_pass_4: 0
regressions_from_pass_5: 0
regressions_from_pass_6: 0
novel_findings: 2
predecessor_verdict: "BLOCKED (2 MEDIUM)"
remediation_landed_in: "v1.6 @ 98f47d86"
next_action: "v1.7 closes LOW-1 (AC-12 StartReport semantics); then Passes 8-9 to fulfill 3-clean-pass window"
convergence_trajectory: "14 → 7 → 2 → 1 (LOW-only)"
---

# Adversarial Review: S-6.20 DTU Demo Server (Pass 7)

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: `P3DTU` (phase-3-dtu-wave-1)
- `<PASS>`: Two-digit pass number
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

## Part A — Fix Verification (Pass 6 → Pass 7)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| ADV-P3DTU-P06-MED-001 | MEDIUM | RESOLVED | Line 994: crowdstrike `~+15 LOC net, see Task 14 Crate 1` matches Task 14 Crate 1 Net delta at line 550. Line 995: claroty `~+30 LOC net, see Task 14 Crate 2` matches line 574. cyberint/armis/threatintel/nvd at 996-999 match Task 14 lines 599/623/647/668. |
| ADV-P3DTU-P06-MED-002 | MEDIUM | RESOLVED | Line 554 now cites `§"ADR-002 Amendment" below at story lines 849-853`. Heading at line 818. Target lines contain trait-default `start()` body. No other stale citations. |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

_None._

### HIGH

_None._

### MEDIUM

_None._

### LOW

#### ADV-P3DTU-P07-LOW-001: AC-12 does not assert StartReport semantics for continue_on_error=true partial success

- **Severity:** LOW
- **Category:** verification-gaps
- **Location:** Story lines 786-792 (AC-12) and 389-409 (StartReport struct)
- **Description:** AC-11 asserts `cleaned_up_after_failure == vec!["crowdstrike","claroty","cyberint"]` under continue_on_error=false. AC-12 (continue_on_error=true) describes WARN log + skip + URL table exclusion but does NOT specify what `last_start_report()` returns. Implementer could build either of two reasonable semantics.
- **Evidence:** Under continue_on_error=true: `successfully_started` includes survivors, `cleaned_up_after_failure` is empty (no rollback), `failed_at` is ambiguous — StartReport has only one `failed_at: Option<(String, std::io::Error)>` slot, which cannot represent multiple skipped-clone failures.
- **Proposed Fix:** Either add AC-13 asserting StartReport shape under continue_on_error=true with a concrete assertion, or extend StartReport with `skipped_due_to_error: Vec<(String, std::io::Error)>` field (preferred). Policy: POL-004 (observability-behavior pairing).

### Observations

#### OBS-1: v1.6 changelog entry is accurate and well-scoped

- **Location:** Story line 1115
- **Note:** Changelog correctly names both Pass-6 MEDIUM fixes with specific numbers. Exemplary remediation discipline. Positive observation.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 1 |

**Overall Assessment:** pass-with-findings
**Convergence:** CONVERGED per 2-finding-LOW-max threshold. User's "No pragmatic convergence" directive triggers v1.7 remediation of LOW-1 before Pass 8 to pursue zero-finding closure.
**Readiness:** requires v1.7 revision (LOW-1 remediation), then Passes 8-9 to fulfill 3-clean-pass window.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 7 |
| **New findings** | 2 (ADV-P3DTU-P07-LOW-001 + OBS-1) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 2 / (2 + 0) = 1.0 |
| **Median severity** | 0.5 (LOW + OBS) |
| **Trajectory** | 14 → 7 → 2 → 1 |
| **Verdict** | CONVERGENCE_REACHED |

## Novel scan (areas 3-12)

- **Frontmatter:** `version: "1.6"` ✓. `input-hash: ""` (canonical state-manager-fill state) ✓.
- **Changelog:** v1.6 row present at line 1115 ✓.
- **AC consistency:** 12 ACs (AC-1..AC-12). All Tasks 1-11, 14 reference correct ACs ✓.
- **BC refs:** `behavioral_contracts: []` (infra story, no product BCs per line 78) ✓.
- **VP refs:** `verification_properties: []` ✓.
- **R-/EC- traceability:** R-DEMO-001 → AC-9; R-DEMO-002 → AC-5; R-DEMO-003 → EC-011/Task 3; EC-012 → AC-11; EC-013 → ADR-002 Amendment. All traced.
- **Task↔File atomicity:** Task 14 crate 1-6 rows match File Structure rows 994-999 one-for-one ✓.
- **Trait-default sanity:** Verified cyberint/armis/threatintel/nvd `start()` hardcode `TcpListener::bind("127.0.0.1:0")` — trait-default `start_on("127.0.0.1:0", None)` is semantic-equivalent. ThreatIntel's `start()` doesn't read `self.config.bind` either, so trait-default path works. ✓

## Source verifications

- ADR-002 file: ~30 lines total (confirmed; "759-761" impossible)
- crowdstrike `start()` lines 62-97 ≈ 36 LOC ✓
- claroty `start()` lines 96-111 ≈ 16 LOC ✓
- 4 "no-server_handle" crates: all `start()` bodies hardcode `127.0.0.1:0` — trait-default compatible ✓

## Convergence trajectory

- Pass 4: 14 findings (2C+5H+5M+2L) → v1.4
- Pass 5: 7 findings (2H+3M+2L) → v1.5
- Pass 6: 2 findings (2M) → v1.6
- Pass 7: 1 LOW + 1 OBS → v1.7 pending
- Trend: 14 → 7 → 2 → 1 (sustained decay, CONVERGED)
