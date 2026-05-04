---
document_type: adversarial-review
pass_id: 26
window_position: "0/3"
disposition: BLOCKED
verdict: FINDINGS_REMAIN
date: 2026-05-04
wave: wave-4-operations
phase: 4A
reviewer: adversary
cross_cuts_verified: 12
---

# Adversarial Review — Pass 26

## Tally

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 0 |
| LOW | 0 |
| INFO | 0 |
| **Total** | **1** |

**Disposition:** BLOCKED (FINDINGS_REMAIN)
**Window:** 0/3 — stays at 0/3; Pass 27 next

---

## Findings

### F-P26-H-001 [HIGH — SUBSTANTIVE]

**Title:** ADR-016 orphan `action_dispatcher` token at lines 552 and 568 — sibling-file regression of F-P25-H-001 PRD fix

**File:** `.factory/specs/architecture/decisions/ADR-016-action-delivery-framework.md`
**Lines:** 552, 568
**Token:** `action_dispatcher`
**Canonical:** `action_delivery` (per concurrency-architecture.md v1.1 and module-decomposition.md v1.13)

**Description:** ADR-016 v0.12 contains two occurrences of the stale `action_dispatcher` token in its body prose. These occurrences are a sibling-file regression of F-P25-H-001 (which fixed the same orphan in prd.md PRD §2 line 382). Both sites were introduced by orchestrator-authored fix-burst prompt text in prior remediation bursts that introduced factual claims (module names, type names) without verifying against the architecture canonical glossary.

**Resolution:** Architect bumped ADR-016 v0.12 → v0.13 with both orphan tokens corrected to `action_delivery`. ARCH-INDEX ADR-016 registry row updated to v0.13.

**Substance:** SUBSTANTIVE — architectural terminology consistency; would propagate stale module name into implementation.

---

### F-PreP27-H-001 [HIGH — SUBSTANTIVE — Pre-emptive catch before Pass 27]

**Title:** vp-045 spec v1.2 contains `action_dispatcher` orphan at lines 37, 44, and 68 — same orphan token class; caught proactively

**File:** `.factory/specs/verification-properties/vp-045-schedule-semaphore-try-acquire-nonblocking.md`
**Lines:** 37, 44, 68
**Token:** `action_dispatcher`
**Canonical:** `action_delivery` (per concurrency-architecture.md v1.1 and module-decomposition.md v1.13)

**Description:** After F-P26-H-001 was identified, a proactive sibling sweep per the TD-VSDD-051 pattern revealed three additional occurrences of the same stale orphan token in the vp-045 spec file body. These 3 sites bring the total orphan count across the corpus to 5 (1 in PRD fixed at Pass 25, 2 in ADR-016 fixed at Pass 26, 3 in vp-045 caught here before Pass 27). All five were introduced by orchestrator-authored fix-burst prompt text.

**Resolution:** Product-owner bumped vp-045 spec v1.2 → v1.3 with all three orphan tokens corrected. No ARCH-INDEX Document Map row change needed (vp-045 spec annotation does not track body versions separately).

**Substance:** SUBSTANTIVE — VP spec body terminology must align with canonical module names for formal verifier and implementer consumers.

---

## Cross-Cut Verification (12 angles)

1. ADR-016 §1 Introduction — checked for `action_dispatcher` orphans
2. ADR-016 §2 Design — checked for `action_dispatcher` orphans (found at lines 552 + 568)
3. ADR-016 §3 Rationale — spot-checked canonical module name usage
4. ADR-016 §4 Consequences — spot-checked canonical module name usage
5. ADR-016 §5 Verification Plan — cross-checked VP references (VP-044/045/046/047)
6. vp-045 spec — full body scan for `action_dispatcher` orphans (found at lines 37/44/68)
7. prd.md v1.10 — verified F-P25-H-001 fix still holds (no reversion)
8. concurrency-architecture.md v1.1 — confirmed `action_delivery` as canonical module name
9. module-decomposition.md v1.13 — confirmed `ActionDeliveryEngine` canonical type name
10. actions.md v1.3 — verified Mermaid labels and prose use canonical names
11. operational-pipeline.md v1.2 — verified F-P23-H-001 fix still holds
12. interface-definitions.md v2.6 — verified F-PreP24-H-001 canonical labels still intact

---

## Meta-Insight: Orchestrator-Prompt-Introduced Orphan Pattern — CODIFIED TD-VSDD-051

All 5 orphan `action_dispatcher` sites across 3 documents (PRD §2, ADR-016, vp-045 spec) were introduced by **orchestrator-authored fix-burst prompts** in prior remediation passes. The orchestrator's prompt text contained the stale module name as an explanatory reference, and the agent writing the fix copied the orphan into the document body verbatim.

**Root cause:** Orchestrator fix-burst prompts that introduce factual claims (module names, type names, paths) are not verified against architecture canonical glossary before dispatch.

**TD-VSDD-051 codified:** Pre-dispatch verification hook — grep orchestrator's fix-burst prompt text for module names + type names + paths; cross-check against canonical glossary (`concurrency-architecture.md` + `module-decomposition.md`). Sibling-ADR prose sweep: when a drift class is closed in PRD or BC, automatically scan all sibling ADRs (§5 Verification Plan, remediation notes) for same orphan token.

---

## Verdict

**FINDINGS_REMAIN.** Window position stays **0/3**. Pass 26 BLOCKED → REMEDIATED.
- F-P26-H-001 REMEDIATED: ADR-016 v0.13
- F-PreP27-H-001 REMEDIATED: vp-045 spec v1.3
- TD-VSDD-051 codified

**Pass 27 next (window 0/3, slot 1/3 attempt).**
