---
pass_id: 20
window_position: "0/3 (BLOCKED — window reset from 2/3)"
disposition: BLOCKED
date: 2026-05-03
tally:
  CRITICAL: 0
  HIGH: 2
  MEDIUM: 0
  LOW: 2
  INFO: 0
  total: 4
---

# Adversary Pass 20 — BLOCKED → REMEDIATED

## Summary

Pass 20 was dispatched as the window 3/3 closure attempt (2/3 OPEN from passes 18+19). Pass 20
found 2 SUBSTANTIVE HIGH findings and 2 COSMETIC LOW findings that had survived all 19 prior
passes. Both HIGH findings are cross-document propagation failures — the category most resistant
to pass-by-pass incremental convergence.

**Window reset: 2/3 → 0/3.** Pass 21 + Pass 22 + Pass 23 needed for a fresh 3-clean window.

## Findings

### F-P20-H-001 — SUBSTANTIVE HIGH: VP-045 description "Schedule semaphore" stale across VP catalog ecosystem

| Field | Value |
|-------|-------|
| Severity | HIGH |
| Substance | SUBSTANTIVE |
| Class | Pass-6 BC-H1 rename failed to propagate to VP catalog ecosystem |
| Files affected | VP-INDEX.md, verification-architecture.md, verification-coverage-matrix.md |
| Resolution agent | state-manager |

**Finding:** VP-045 description reads "Schedule semaphore: try_acquire used (non-blocking), never
acquire" across VP-INDEX.md (line 66), verification-architecture.md (line 174), and
verification-coverage-matrix.md BC-2.18.004 row parenthetical (line 123).

Pass 6 renamed BC-2.18.004 H1 from "Schedule Semaphore" to "Action Delivery Semaphore"
(BC-2.18.004 v1.4 wave-4-phase-4a-fix-burst; 16-permit→8-permit + rename). That rename was
correctly propagated to BC-2.18.004 H1, BC-INDEX, and STORY-INDEX, but failed to cascade to the
VP catalog ecosystem — VP-INDEX, verification-architecture, and verification-coverage-matrix all
still carried the stale "Schedule semaphore" language.

**Resolution:** state-manager updated VP-045 description in all three VP catalog documents:
- VP-INDEX.md v1.25→v1.26: "Schedule semaphore" → "Action delivery semaphore: try_acquire used (non-blocking), never acquire"
- verification-architecture.md v1.26→v1.27: same VP-045 row description updated
- verification-coverage-matrix.md v1.30→v1.31: BC-2.18.004 row parenthetical "(Schedule semaphore non-blocking)" → "(Action delivery semaphore non-blocking)"

### F-P20-H-002 — SUBSTANTIVE HIGH: VP-045 + VP-047 priority drift (POL-9 violation)

| Field | Value |
|-------|-------|
| Severity | HIGH |
| Substance | SUBSTANTIVE |
| Class | Priority field drift — ADR-016 §5.2/§5.4 said P1; VP-INDEX SoT says P0 |
| Files affected | ADR-016 (decisions/ADR-016-action-delivery-framework.md) |
| Resolution agent | architect |
| POL violated | POL-9 (VP-INDEX is source-of-truth for VP priority; ADR must follow) |

**Finding:** ADR-016 §5.2 (VP-045) and §5.4 (VP-047) listed both VPs with priority P1. VP-INDEX
SoT has both VP-045 and VP-047 at P0. POL-9 requires ADR body to follow VP-INDEX SoT; ADR-016
had drifted.

**Resolution:** architect updated ADR-016 v0.11→v0.12: §5.2 VP-045 P1→P0 and §5.4 VP-047
P1→P0 to match VP-INDEX SoT. ARCH-INDEX ADR Registry updated v0.11→v0.12 (ARCH-INDEX v2.16→v2.17).

### F-P20-L-001 — COSMETIC LOW: S-4.08 token budget version pin stale

| Field | Value |
|-------|-------|
| Severity | LOW |
| Substance | COSMETIC |
| Class | Story body version pin (structural prevention category) |
| Files affected | S-4.08 (stories/wave-4/S-4.08-action-delivery-framework.md) |
| Resolution agent | story-writer |

**Finding:** S-4.08 body contained a stale version pin for the token budget library reference.
Structural prevention discipline (drop version pins from story body cross-refs) was applied
in Pass 11, but this pin was missed in that sweep.

**Resolution:** story-writer updated S-4.08 v1.22→v1.23: token budget version pin dropped.
STORY-INDEX row updated [v1.22] → [v1.23]. STORY-INDEX v2.01→v2.02.

### F-P20-L-002 — COSMETIC LOW: BC-2.18.001/002/004 ActionEngine shorthand vs ActionDeliveryEngine canonical

| Field | Value |
|-------|-------|
| Severity | LOW |
| Substance | COSMETIC |
| Class | Type-name shorthand drift in BC body prose |
| Files affected | BC-2.18.001, BC-2.18.002, BC-2.18.004 |
| Resolution agent | product-owner |

**Finding:** BC body prose in BC-2.18.001 (1 site), BC-2.18.002 (2 sites), BC-2.18.004 (10 sites)
used the shorthand "ActionEngine" instead of the canonical "ActionDeliveryEngine". The canonical
struct name is ActionDeliveryEngine throughout the prism-operations crate; the shorthand was
an informal abbreviation inconsistent with the spec's own H1 declarations.

**Resolution:** product-owner updated:
- BC-2.18.001 v1.7→v1.8: 1 site corrected ActionEngine→ActionDeliveryEngine
- BC-2.18.002 v1.4→v1.5: 2 sites corrected ActionEngine→ActionDeliveryEngine
- BC-2.18.004 v1.4→v1.5: 10 sites corrected ActionEngine→ActionDeliveryEngine
BC-INDEX v4.30→v4.31.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 20 |
| **New findings** | 4 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 4/4 = 1.00 |
| **Median severity** | 3.5 (2 HIGH + 2 LOW; HIGH=4, LOW=2 on 1-5 scale) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5→5→4→7→9→2→4→3→3→0→4 |
| **Verdict** | FINDINGS_REMAIN |

All 4 findings are genuinely new — none are variants of prior-pass findings. F-P20-H-001 is a
fresh propagation gap class (VP catalog ecosystem); F-P20-H-002 is a fresh POL-9 ADR priority
drift; both survived 19 prior passes of heightened scrutiny. Fresh-context value confirmed:
the 3-clean window discipline catches what looser convergence criteria miss.

## Cross-cut Sample Log

20 cross-cut chains sampled under heightened scrutiny:

| Chain | Result |
|-------|--------|
| ADR Status H2 sync (6 ADRs) | CLEAN |
| VP-INDEX self-arithmetic (145 VPs) | CLEAN |
| VP-INDEX→verification-architecture VP-045 propagation | FAIL (F-P20-H-001) |
| VP-INDEX→verification-coverage-matrix VP-045 propagation | FAIL (F-P20-H-001) |
| ADR-016 §5.x priority vs VP-INDEX SoT (VP-045, VP-047) | FAIL (F-P20-H-002) |
| BC H1↔BC-INDEX alignment BC-2.18.001/002/004 | CLEAN |
| CF key prefix uniformity (4 ADRs) | CLEAN |
| VP-137/VP-143 closed-loop chain | CLEAN |
| ScheduleChangeNotification tuple sites | CLEAN |
| Frontmatter date uniformity (6 ADRs) | CLEAN |
| Story body ActionDeliveryEngine canonical name | FAIL (F-P20-L-002) |
| S-4.08 token budget reference | FAIL (F-P20-L-001) |
| VP-047 priority VP-INDEX vs ADR-016 | FAIL (F-P20-H-002, counted once) |
| BC-2.18.002/004 version in BC-INDEX | CLEAN |
| Remaining 6 chains | CLEAN |

## Verdict

`FINDINGS_REMAIN` — BLOCKED.

Window status: 2/3 OPEN → BLOCKED → **0/3 RESET** after full remediation.

All 4 findings remediated:
- F-P20-H-001: state-manager (VP-INDEX v1.26, verification-architecture v1.27, coverage-matrix v1.31)
- F-P20-H-002: architect (ADR-016 v0.12, ARCH-INDEX v2.17)
- F-P20-L-001: story-writer (S-4.08 v1.23, STORY-INDEX v2.02)
- F-P20-L-002: product-owner (BC-2.18.001 v1.8, BC-2.18.002 v1.5, BC-2.18.004 v1.5, BC-INDEX v4.31)

Pass 21 + Pass 22 + Pass 23 needed for a fresh 3-clean window.
