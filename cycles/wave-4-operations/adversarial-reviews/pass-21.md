---
pass_id: 21
window_position: "0/3"
disposition: BLOCKED
date: 2026-05-03
tally:
  CRITICAL: 0
  HIGH: 2
  MEDIUM: 1
  LOW: 0
  INFO: 0
verdict: FINDINGS_REMAIN
remediation_status: REMEDIATED
---

# Adversarial Review — Pass 21

**Date:** 2026-05-03
**Window position:** 0/3 — BLOCKED → REMEDIATED. Window stays 0/3.
**Pass 22 + Pass 23 + Pass 24 needed for fresh 3-clean window.**

## Tally

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 1 |
| LOW | 0 |
| INFO | 0 |
| **Total** | **3** |

All 3 findings SUBSTANTIVE. All in data-layer.md (laggard sister-file across 20+ passes).

## Findings

### F-P21-H-001 [SUBSTANTIVE] — HIGH

**Site:** data-layer.md line 269 — concurrency claim "16 scheduled" stale
**Description:** Line 269 stated "16 scheduled" queries concurrent. D-209 locked per-subsystem semaphores at 8 permits each (schedule: 8, action delivery: 8). The claim was stale — it predated D-209 which resolved the shared-semaphore question to per-subsystem isolation at 8 permits.
**Resolution:** Architect remediated data-layer.md v1.2 → v1.3. Updated to reflect D-209 LOCKED 8/8 + 2 ad-hoc permits per subsystem.

### F-P21-H-002 [SUBSTANTIVE] — HIGH

**Site:** data-layer.md CF count 16 vs ARCH-INDEX AD-004 17; missing `case_dedup_idx` row
**Description:** data-layer.md listed 16 column families in its CF inventory table, omitting `case_dedup_idx`. ARCH-INDEX AD-004 had been updated to 17 CFs (with case_dedup_idx added per P5-XADR-A-M-006 at ARCH-INDEX v2.5). data-layer.md was not updated at that time — a partial-fix regression. The missing row left the canonical CF inventory inconsistent with the authoritative AD-004 source.
**Resolution:** Architect remediated data-layer.md v1.2 → v1.3. Updated CF count 16→17, added `case_dedup_idx` row per P5-XADR-A-M-006.

### F-P21-M-001 [SUBSTANTIVE] — MEDIUM

**Site:** data-layer.md line 266 — retry CF key format stale
**Description:** Line 266 showed a retry CF key format that did not match the canonical form established in ADR-016 §2.5. The canonical format per ADR-016 §2.5 is `{org_id}:\x04:{action_id}:{idempotency_key}`. The stale format predated the ADR-016 §2.5 key canonicalization work from pass 8/9 iterations.
**Resolution:** Architect remediated data-layer.md v1.2 → v1.3. Updated to canonical `{org_id}:\x04:{action_id}:{idempotency_key}` per ADR-016 §2.5.

## Broad-Sweep Cross-Cut Samples (20 verified)

The adversary performed a 20-sample broad sweep across the corpus before focusing on the laggard. All 20 cross-cut samples landed correctly:

| # | File | Claim Verified |
|---|------|---------------|
| 1 | actions.md | ActionDeliveryEngine (post-rename); 8-permit semaphore per D-209 |
| 2 | actions.md | 60s default tick per ADR-013 §2.1 |
| 3 | module-decomposition.md | ActionDeliveryEngine (3 rename sites correct) |
| 4 | api-surface.md | ActionDeliveryEngine (1 rename site correct) |
| 5 | verification-architecture.md | Mermaid P13 node label corrected |
| 6 | BC-2.18.003 | ActionDeliveryEngine (post-rename v1.4) |
| 7 | BC-2.18.008 | ActionDeliveryEngine (post-rename v1.4) |
| 8 | ADR-016 | v0.12 VP-045/047 priority P1→P0 per POL-9 |
| 9 | ADR-013 | v0.7 croner 2.1; ScheduleFireMissed{miss_reason: SemaphoreExhausted} |
| 10 | S-4.01 | ScheduleFireMissed{miss_reason: SemaphoreExhausted} correct |
| 11 | S-4.02 | CF key format correct per ADR-018 |
| 12 | S-4.04 | CF-name vs key notation correct per Pre-Pass-14 sweep |
| 13 | S-4.05 | action_state CF correct (not detection_state) |
| 14 | S-4.06 | ADR-016 annotation correct (not ADR-019); case_dedup_idx present |
| 15 | S-4.08 | 8-permit semaphore per D-209; 60s tick; no version pins |
| 16 | BC-2.18.001 | CF keys with OrgId prefix + \x04/\x03 discriminators |
| 17 | VP-INDEX | VP-045 desc "Action Delivery Semaphore" correct |
| 18 | verification-architecture.md | VP-045/047 priority P0 correct |
| 19 | STORY-INDEX | S-4.06 ADR annotation correct; VPs fully-prefixed |
| 20 | ARCH-INDEX AD-004 | 17 CFs with case_dedup_idx listed |

**Pattern:** data-layer.md was the laggard sister-file across 20+ passes. The pre-Pass-21 broad-sweep (F-PreP21-H-001/002) applied surface rename (ActionEngine→ActionDeliveryEngine) but missed deeper architectural claims (concurrency permits, CF count, retry key format). Pass 21 caught all 3.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 21 |
| **New findings** | 3 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 3/3 = 1.00 |
| **Median severity** | 4 (2 HIGH + 1 MEDIUM; HIGH=4, MEDIUM=3 on 1-5 scale) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5→5→4→7→9→2→4→3→3→0→4→3 |
| **Verdict** | FINDINGS_REMAIN |

All 3 findings are genuinely new — none are variants of prior-pass findings. All 3 are concentrated in data-layer.md, the laggard sister-file that was updated at the surface level (ActionEngine→ActionDeliveryEngine) during the Pre-Pass-21 broad-sweep but whose deeper architectural claims (concurrency model, CF inventory, retry key format) were not updated. Pass 21 caught all 3 in a single targeted pass.

## Window Status

**Window:** 0/3 BLOCKED → REMEDIATED.
**Pass 22 + Pass 23 + Pass 24 needed** for fresh 3-clean window (3 consecutive CLEAN passes required).
