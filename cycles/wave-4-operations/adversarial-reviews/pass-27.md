---
pass_id: 27
window_position: "0/3"
disposition: BLOCKED
date: 2026-05-04
verdict: FINDINGS_REMAIN
findings_summary: { CRITICAL: 0, HIGH: 1, MEDIUM: 0, LOW: 0, INFO: 0 }
remediation_status: REMEDIATED
remediation_artifact: ADR-016 v0.13→v0.14 (§5.4 footer + v0.12 changelog VP-047 rationale corrected)
next_pass: 28
---

# Adversary Pass 27 — Wave 4 Phase 4.A

**Window:** 0/3 (BLOCKED — window stays 0/3; Pass 28 next)
**Disposition:** BLOCKED → REMEDIATED
**Date:** 2026-05-04

## Tally

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 0 |
| LOW | 0 |
| INFO | 0 |
| **Total** | **1** |

## Findings

### F-P27-H-001 [HIGH — SUBSTANTIVE] VP-047 Rationale Semantic Mis-Anchor in ADR-016

**Location:** ADR-016 §5.4 footer (line 533) + v0.12 changelog entry (line 579)

**Finding:** Both the §5.4 verification-plan footer and the v0.12 changelog entry describe VP-047 with the rationale "action delivery dedup correctness". This is wrong. Per VP-INDEX line 68 (authoritative `Property` column) and BC-2.18.009, VP-047 anchors **template variable UUID v7 validation** — not action delivery dedup correctness. Action delivery dedup is covered by VP-044/VP-046, not VP-047. The mis-anchor is a semantic error: it mis-attributes VP-047's scope to a different verification property, potentially misleading implementers and test-writers about what VP-047 actually verifies.

**Root cause:** Orchestrator-authored fix-burst prompt for the Pass 20 F-P20-H-002 fix-burst contained the phrase "VP-047 verifies action delivery dedup correctness" without cross-checking VP-INDEX's authoritative `Property` field for VP-047. The erroneous rationale was transcribed verbatim into ADR-016 §5.4 and the v0.12 changelog entry.

**Resolution:** architect burst — ADR-016 v0.13→v0.14:
- §5.4 footer line 533: VP-047 rationale corrected to "template variable UUID v7 validation" per VP-INDEX line 68
- v0.12 changelog entry line 579: same correction applied

**Comprehensive cross-check:** grep across all 6 W4 ADRs (ADR-013/015/016/017/018/019) for the erroneous rationale string confirmed F-P27-H-001 is the sole VP-INDEX mis-anchor site. No sibling files affected.

**Classification:** SUBSTANTIVE — wrong VP scope claim in authoritative spec document.

## Cross-Cut Verification (14 chains)

1. ADR-016 §5.4 VP-047 rationale text vs VP-INDEX `Property` column — **MISMATCH FOUND** (F-P27-H-001)
2. ADR-016 §5.4 VP-044 rationale vs VP-INDEX — correct
3. ADR-016 §5.4 VP-045 rationale vs VP-INDEX — correct
4. ADR-016 §5.4 VP-046 rationale vs VP-INDEX — correct
5. ADR-013 VP cross-references vs VP-INDEX — clean (0 findings)
6. ADR-015 VP cross-references vs VP-INDEX — clean (0 findings)
7. ADR-017 VP cross-references vs VP-INDEX — clean (0 findings)
8. ADR-018 VP cross-references vs VP-INDEX — clean (0 findings)
9. ADR-019 VP cross-references vs VP-INDEX — clean (0 findings)
10. BC-2.18.009 scope vs VP-047 Property column — consistent (UUID v7 validation)
11. VP-INDEX line 68 Property text vs fix in ADR-016 v0.14 — verified consistent post-fix
12. ADR-016 v0.12 changelog rationale string (line 579) vs VP-INDEX — **MISMATCH FOUND** (same F-P27-H-001; both sites resolved in v0.14)
13. No orphan "action delivery dedup correctness" string in any sibling ADR (grep confirmed)
14. ADR-016 §2.5 retry key + §5.5 retry scanner tick — unchanged, still canonical

## META-INSIGHT

**6th orchestrator-prompt-introduced defect this session.** This is a NEW class beyond stale module names (TD-VSDD-050/051 covered module-name and orphan-token classes). This class: **semantic mis-anchor in VP rationale text** — the orchestrator's fix-burst prompt attributed an incorrect `Property` scope to VP-047, which was transcribed without verification into spec content.

The pattern: when an orchestrator fix-burst prompt mentions a VP (e.g., "VP-047 verifies X"), that claim must be verified against VP-INDEX `Property` column before the prompt is dispatched. Currently there is no mechanical gate for this.

**TD-VSDD-052 codified** (see vsdd-plugin-tech-debt.md).

## TD Filed

**TD-VSDD-052:** Pre-dispatch VP scope verification — when orchestrator's fix-burst prompt mentions a VP-NNN with explanatory rationale, automatically grep VP-INDEX for VP-NNN's authoritative `Property` field and require the rationale wording to match canonical Property terms. Sister of TD-VSDD-051 (orchestrator-prompt module-name verification). Discovered Pass 27: Pass 20 F-P20-H-002 fix-burst prompt authored "VP-047 verifies action delivery dedup correctness" but VP-047 actually anchors UUID v7 validation per VP-INDEX line 68. Hook recommendation: abort dispatch if prompt VP rationale does not contain canonical Property terms.

## Remediation Summary

| File | Version Change | Finding | Type |
|------|---------------|---------|------|
| ADR-016-action-delivery-framework.md | v0.13 → v0.14 | F-P27-H-001 (2 sites: §5.4 footer + v0.12 changelog) | SUBSTANTIVE |

**ARCH-INDEX:** v2.26 → v2.27 (ADR-016 registry row v0.14; pass-27 changelog row)

## Next Action

Pass 28 (window 1/3 attempt). Window stays 0/3.
