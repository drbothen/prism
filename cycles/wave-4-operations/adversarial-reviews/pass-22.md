---
pass_id: 22
window_position: "0/3"
disposition: BLOCKED
date: 2026-05-03
tally:
  CRITICAL: 0
  HIGH: 1
  MEDIUM: 1
  LOW: 1
  INFO: 0
verdict: FINDINGS_REMAIN
remediation_status: REMEDIATED
---

# Adversarial Review — Pass 22

**Date:** 2026-05-03
**Window position:** 0/3 — BLOCKED → REMEDIATED. Window stays 0/3.
**Pass 23 + Pass 24 + Pass 25 needed for fresh 3-clean window.**

## Tally

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 1 |
| LOW | 1 |
| INFO | 0 |
| **Total** | **3** |

F-P22-H-001 SUBSTANTIVE. F-P22-M-001 SUBSTANTIVE (subsumed by H-001). F-P22-L-001 COSMETIC.

## Finding ID Convention

Finding IDs for this pass: `F-P22-{SEV}-{NNN}` (project-local short form per Wave 4 convention).

## Part A — Fix Verification

Pass 21 findings all RESOLVED:
- F-P21-H-001 (data-layer.md concurrency "16 scheduled"): RESOLVED — v1.3 correct (8/8+2 ad-hoc per D-209).
- F-P21-H-002 (data-layer.md CF count 16→17 + case_dedup_idx): RESOLVED — v1.3 correct (17 CFs, case_dedup_idx row present).
- F-P21-M-001 (data-layer.md retry key format): RESOLVED — v1.3 canonical `{org_id}:\x04:{action_id}:{idempotency_key}` per ADR-016 §2.5.

Pre-Pass-22 broad-scope sweep findings all RESOLVED:
- F-PreP22-H-001 (concurrency-architecture.md 16-permit): RESOLVED — v1.1 8/8 split correct.
- F-PreP22-H-002 (observability.md user-facing examples): RESOLVED — v1.1 split-form examples correct.
- F-PreP22-H-003 (interface-definitions.md ActionEngine): RESOLVED — v2.5 ActionDeliveryEngine throughout.
- F-PreP22-H-004 (vp-045 spec body 16-permit ActionEngine): RESOLVED — v1.2 8-permit ActionDeliveryEngine + slug-preservation banner.

## Part B — New Findings

### HIGH

#### F-P22-H-001 [SUBSTANTIVE] — actions.md §"Delivery state" 4-row CF key table stale (no `{org_id}:` prefix; pre-ADR-016 §2.5 form)

- **Severity:** HIGH
- **Category:** spec-fidelity / CF key format inconsistency
- **Location:** `.factory/specs/architecture/actions.md` §"Delivery state" (action_state CF key table)
- **Description:** The `action_state` column-family key table contained 4 stale rows without the canonical `{org_id}:` prefix. The table predated ADR-016 §2.5 canonicalization and was missing the 5-row form (pending `\x01`, in-flight `\x02`, dead-letter `\x03`, retry-state `\x04`, dedup `\x05`). Additionally, the document still reflected the pre-D-209 16-permit semaphore, the pre-ADR-013 §2.1 1-second tick interval, and the pre-rename `ActionEngine` name — all three were fixed in the Pre-Pass-21 broad-sweep to v1.1, but the CF key table was not updated at that time.
- **Evidence:** actions.md v1.1 §"Delivery state" had 4-row table with `{alert_id}` key component; no `{org_id}:` prefix; no `{idempotency_key}` sort-key on retry row; discriminator set incomplete.
- **Proposed Fix:** Rewrite action_state CF key table to canonical 5-row ADR-016 §2.5 form with `{org_id}:` prefix and correct discriminators.
- **Resolution:** Architect remediated — actions.md v1.1 → v1.2. CF key table rewritten to canonical 5-row ADR-016 §2.5 form.

### MEDIUM

#### F-P22-M-001 [SUBSTANTIVE, subsumed by H-001] — actions.md retry row used `{alert_id}` not `{idempotency_key}`

- **Severity:** MEDIUM
- **Category:** spec-fidelity / CF key sort-key inconsistency
- **Location:** `.factory/specs/architecture/actions.md` §"Delivery state" (action_state CF key table, retry row)
- **Description:** Within the same stale 4-row table as F-P22-H-001, the retry row used `{alert_id}` as the sort-key component instead of the canonical `{idempotency_key}` per ADR-016 §2.5. A consumer implementing from this row would produce an incorrect retry CF key. This is a distinct logical defect from the missing `{org_id}:` prefix.
- **Evidence:** actions.md v1.1 retry row: `action_state:{alert_id}:{...}` — `{alert_id}` instead of `{idempotency_key}` per ADR-016 §2.5 retry-state canonical form.
- **Proposed Fix:** Correct retry sort-key to `{idempotency_key}` per ADR-016 §2.5 `\x04` row.
- **Resolution:** Subsumed by F-P22-H-001. actions.md v1.2 rewrites entire table; retry row now correctly uses `{idempotency_key}` as `\x04` discriminator sort-key per ADR-016 §2.5.

### LOW

#### F-P22-L-001 [COSMETIC] — ARCH-INDEX Document Map line 39 actions.md row missing version annotation

- **Severity:** LOW
- **Category:** index-annotation gap (cosmetic)
- **Location:** `.factory/specs/architecture/ARCH-INDEX.md` Document Map, Actions row (line ~39)
- **Description:** The ARCH-INDEX Document Map actions.md row lacked a version annotation. Sibling rows for data-layer (v1.3 + decision references), concurrency-architecture (v1.1 + D-209 reference), and observability (v1.1 + D-209 reference) all carry inline version annotations. The actions.md row (bumped to v1.2 by the H-001 fix) was unannotated, leaving index inconsistent.
- **Evidence:** ARCH-INDEX v2.20 line 39: `| Actions | actions.md | ~1,500 | implementer, test-writer | Alert delivery + scheduled reports — Slack, PagerDuty, Jira, email, syslog. TOML specs + .prx plugins. |` — no version annotation.
- **Proposed Fix:** Append `— v1.2 (D-209 8/8 split + 60s tick + ActionDeliveryEngine + ADR-016 §2.5 CF table per F-P22-H-001)` to the actions.md Document Map row; bump ARCH-INDEX version 2.20 → 2.21.
- **Resolution:** State-manager — ARCH-INDEX line 39 updated; version 2.20 → 2.21.

## Cross-Cut Verification Samples (14 verified)

The adversary verified 14 cross-cut samples across the corpus prior to finding F-P22-H-001:

| # | Document | Version | Claim Verified |
|---|----------|---------|----------------|
| 1 | concurrency-architecture.md | v1.1 | 8/8 semaphore split per D-209; no 16-permit mention |
| 2 | observability.md | v1.1 | user-facing log examples show 8/8 split; no 3/16 or 16 permits |
| 3 | data-layer.md | v1.3 | CF count 17; retry key `{org_id}:\x04:{action_id}:{idempotency_key}` per ADR-016 §2.5 |
| 4 | ADR-013 | §2.3 | croner 2.1; 60s default tick; per-subsystem semaphore 8 permits |
| 5 | ADR-016 | §2.11 | at-least-once delivery; dead-letter `\x03` discriminator; `{idempotency_key}` in retry row |
| 6 | BC-2.18.001 | v1.8 | ActionDeliveryEngine; 8-permit per ADR-016 §2.3; CF keys with `{org_id}:` prefix |
| 7 | S-4.01 | Task 5 | ScheduleFireMissed{miss_reason: SemaphoreExhausted} per ADR-013 §2.4; 8 permits |
| 8 | S-4.08 | AC-13/16 | ActionDeliveryEngine references consistent; no ActionEngine occurrences |
| 9 | ARCH-INDEX | AD-004 | 17 CFs; case_dedup_idx listed |
| 10 | VP-INDEX | arithmetic | VP-137..VP-145 (9 VPs) listed; total consistent with vp_count 145 |
| 11 | verification-architecture.md | v1.28 | VP-053 crate column prism-operations (not prism-core); P13 Mermaid node updated |
| 12 | BC-2.18.003 | v1.4 | ActionDeliveryEngine (not ActionEngine) |
| 13 | BC-2.18.008 | v1.4 | ActionDeliveryEngine (not ActionEngine) |
| 14 | interface-definitions.md | v2.5 | ActionDeliveryEngine throughout; no ActionEngine occurrences |

**Pattern:** The pre-Pass-22 broad-scope sweep (F-PreP22-H-001..H-004) and Pass-21 fix (F-P21-H-001..M-001) addressed the most-recently-modified foundation docs but did not include actions.md's own action_state CF key table in the ADR-016 §2.5 canonical-form audit. The table had been surface-updated (ActionEngine→ActionDeliveryEngine, 16-permit→8-permit, 1s→60s) in the Pre-Pass-21 sweep, but the deeper CF key structure remained pre-canonicalization.

## TD-VSDD-047 Codification

**Trigger:** F-P22-H-001. actions.md §"Delivery state" had 4 stale rows even after Pre-Pass-21 broad sweep and Pass 21 fix to data-layer.md. The broad-sweep fixed surface-level claims but missed the CF key table format — a deeper structural inconsistency with ADR-016 §2.5.

**Lesson:** When fixing a CF key format in one architecture document, grep all architecture docs for the same CF name and audit all key-format tables in lockstep. Filed as TD-VSDD-047 in vsdd-plugin-tech-debt.md.

**Hook recommendation:** When state-manager bumps a CF-related doc, grep for `:{action_id}:|:{schedule_id}:|:diff:|:case:|:alert:|:retry:|:dedup:` patterns across all `architecture/*.md` and BC files; flag any non-canonical format.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 1 |
| LOW | 1 |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — iterate
**Readiness:** requires revision

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 22 |
| **New findings** | 3 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 3/3 = 1.00 |
| **Median severity** | 4 (1 HIGH + 1 MEDIUM + 1 LOW; HIGH=4 on 1-5 scale) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5→5→4→7→9→2→4→3→3→0→4→3→3 |
| **Verdict** | FINDINGS_REMAIN |

All 3 findings are genuinely new. F-P22-H-001 and F-P22-M-001 are concentrated in actions.md's CF key table — a structural gap that survived two prior broad-scope sweeps because sweeps verified surface-level claims (names, constants) but did not audit CF key table format against ADR-016 §2.5. F-P22-L-001 is a cosmetic index annotation gap. TD-VSDD-047 filed to prevent recurrence.

## Window Status

**Window:** 0/3 BLOCKED → REMEDIATED.
**Pass 23 + Pass 24 + Pass 25 needed** for fresh 3-clean window (3 consecutive CLEAN passes required).
