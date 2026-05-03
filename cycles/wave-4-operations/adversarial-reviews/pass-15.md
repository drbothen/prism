---
document_type: adversary-review
pass_id: 15
wave: wave-4
phase: 4.A
window_position: "0/3 (BLOCKED)"
producer: vsdd-factory:adversary
date: 2026-05-03
disposition: BLOCKED
---

# Adversary Pass 15 — Wave 4 Phase 4.A

## Part A — Fix Verification (Pass 14 findings)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-P14-H-001 | HIGH | RESOLVED | ScheduleFireSkipped→ScheduleFireMissed{miss_reason:SemaphoreExhausted} confirmed in S-4.01 v1.12 |
| F-P14-H-002 | HIGH | RESOLVED | BC-2.12.004 future-date 2026-05-04→2026-05-03 confirmed in v1.8 |
| F-P14-M-001 | MEDIUM | RESOLVED | 13-site enum tuple cascade confirmed clean across ADR-013 v0.7, ADR-015 v0.5, ADR-018 v0.5, S-4.01 v1.12, S-4.02 v1.11 |
| F-P14-M-002 | MEDIUM | RESOLVED | ADR-013 v0.7 producer attribution paragraph confirmed |
| F-P14-M-003 | MEDIUM | RESOLVED | S-4.02 v1.11 pack_id org_id clarification confirmed |
| F-P14-M-004 | MEDIUM | RESOLVED | S-4.08 v1.21 OCSF→CEF canonical table per ADR-019 §3 confirmed |
| F-P14-L-001 | LOW | RESOLVED | S-4.05 v1.12 EC-007 detection_state→action_state confirmed |
| F-P14-L-002 | LOW | RESOLVED | ADR-013 v0.7 Status H2 sync confirmed |

## Part B — New Findings

### HIGH

#### F-P15-H-001: S-4.08 Architecture Compliance Rules cron tick interval stale (Pass-8 sister-text gap)

- **Severity:** HIGH
- **Category:** spec-fidelity / sister-text propagation gap
- **Location:** S-4.08 Architecture Compliance Rules section, line ~473
- **Description:** The Architecture Compliance Rules section still reads "1-second interval" for the scheduler tick. ADR-013 §2.1 specifies a 60-second default tick configurable via `PRISM_SCHEDULER_TICK_SECS` [10-3600s]. This is a partial-fix regression of P8-S-4.08-A-H-003: Pass 8 corrected the primary §4 prose and AC-6 but the Architecture Compliance Rules section is a separate orthogonal prose block that was outside the Pass 8 sweep scope.
- **Evidence:** S-4.08 line ~473 contains "1-second interval"; ADR-013 §2.1 canonical value is 60s default with [10-3600s] range via env var.
- **Proposed Fix:** Replace "1-second interval" with "60-second default tick, configurable via PRISM_SCHEDULER_TICK_SECS [10-3600s] per ADR-013 §2.1"
- **Resolution:** story-writer: S-4.08 v1.21 → v1.22. REMEDIATED.

#### F-P15-H-002: STORY-INDEX total_vps_assigned cascade gap (POLICY 3+9 violation)

- **Severity:** HIGH
- **Category:** spec-fidelity / POLICY 9 cascade propagation gap (new class)
- **Location:** STORY-INDEX.md line ~12 frontmatter `total_vps_assigned: 136`; line ~26 prose `**VPs assigned:** 136 (... 77 proptests ...)`
- **Description:** Wave 4 ADR-burst (Phases 1-3, 2026-05-02) added VP-137 through VP-145 (9 new VPs). Each was correctly propagated to VP-INDEX v1.25 (total=145), verification-architecture.md v1.26 ("145 Verified Properties"), and verification-coverage-matrix.md v1.30 (Total 145). However, STORY-INDEX frontmatter `total_vps_assigned` and the prose overview VP breakdown were never updated. At Pass 15: authoritative VP-INDEX v1.25 shows 145 total with 86 proptests; STORY-INDEX still reads 136 and "77 proptests".
- **Evidence:** VP-INDEX v1.25 line 177 total=145; verification-architecture.md v1.26 "145 Verified Properties"; verification-coverage-matrix.md v1.30 Total=145. STORY-INDEX frontmatter total_vps_assigned=136 and prose "136 (30 Kani proofs, 77 proptests, ...)".
- **Proposed Fix:** STORY-INDEX frontmatter total_vps_assigned: 136 → 145; prose "136 (30 Kani proofs, 77 proptests, ...)" → "145 (30 Kani proofs, 86 proptests, ...)".
- **Resolution:** state-manager: STORY-INDEX v1.96 → v1.97. REMEDIATED.

### MEDIUM

_None._

### LOW

_None._

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 0 |
| LOW | 0 |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — iterate (Pass 16 required; window 1/3 attempt)
**Readiness:** requires revision — both HIGH findings remediated; ready for Pass 16

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 15 |
| **New findings** | 2 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 2 / (2 + 0) = 1.0 |
| **Median severity** | HIGH (4.0) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5→5→4→7→9→2 |
| **Verdict** | FINDINGS_REMAIN |

**Novelty notes:** Both findings are genuinely new. F-P15-H-001 is a previously-undetected Pass-8 sister-text gap in an orthogonal prose block (Architecture Compliance Rules) not in the original Pass-8 sweep scope — confirmed new site, same defect class. F-P15-H-002 is a NEW process-gap class: STORY-INDEX top-level aggregates (`total_vps_assigned`, prose tally) are not in the standard POLICY 9 cascade checklist. This class has never previously appeared. TD-VSDD-042 filed to codify the checklist extension.

## Observations (Clean Checks)

1. Pass 14 cascades all clean — audit-event terminology, enum tuple, BC frontmatter date confirmed stable.
2. CF-key notation correct throughout — `CF=<name>`, `key={org_id}:...` form per ADR-016 §2.5.
3. VP-053 module attribution clean — prism-operations confirmed in verification-architecture.md v1.26.
4. VP-145 dual-anchor clean — INV-CASE-006 anchor to S-4.06 consistent across VP-INDEX and verification-architecture.
5. AD-004 17 CFs verified — ARCH-INDEX v2.12 consistent with all 8 W4 story bodies.
6. ADR-019 §1 boundary discipline upheld — SIEM output formats scope clean, no bleed into ADR-016.
7. D-209/VP-137/VP-143 dual-anchors consistent — per-subsystem semaphore invariants cross-referenced correctly.
8. OCSF→CEF mapping table aligned — S-4.08 v1.21 canonical table matches ADR-019 §3.
9. idempotency_key model consistent — dead-letter CF value field placement uniform across ADR-016, S-4.08, BC-2.18.001.
10. ADR subsystems_affected map to canonical SS-NNN IDs throughout all 6 Wave 4 ADRs.
11. Story BC frontmatter-body table consistency clean across spot-checked S-4.01, S-4.02, S-4.05, S-4.08.
