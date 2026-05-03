---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 11
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-04T00:30:00Z
predecessor: pass-10.md (BLOCKED 5 findings; remediated 2026-05-03)
verdict: BLOCKED
findings_count: 5
severity_breakdown: { CRITICAL: 0, HIGH: 1, MEDIUM: 2, LOW: 2, OBS: 0 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-04
remediation_commits: [<Stage 1 SHA>]
structural_prevention_adopted: true
---

# Adversarial Review Pass 11 — Wave 4 Phase 4.A

**Verdict:** BLOCKED (5 findings: 0C / 1H / 2M / 2L / 0OBS)

**Structural Prevention Adopted:** Story-body ADR/BC cross-references no longer pin `vN.M` versions. This structural change removes the entire sister-prose regression class that has produced repeated findings across passes 7–11. Starting with this remediation burst, story bodies reference ADRs and BCs by ID only (no version suffix).

**Trajectory:** 38→17→8→7→7→5→5→6→6→5→5 (flat at 5; structural fix adopted; expect Pass 12 convergence)

---

## Findings

### F-P11-H-001 — Stale ADR/BC Version Pins in Story Prose (Sister-Prose Regression Class)

**ID:** F-P11-H-001
**Severity:** HIGH
**Category:** Sister-prose regression (structural recurrence)
**Files:** S-4.08 (multiple body locations), S-4.05 (body ADR references)
**Finding:** Story body prose contains explicit `vN.M` version pins for ADR and BC cross-references. These pins become stale on every ADR/BC amendment, creating a persistent regression class. Pass 7 removed version pins from S-4.08 AC-8 but left pins in prose paragraphs. Pass 8–10 remediation targeted specific instances without addressing the root cause structurally.

In S-4.08 v1.18, the following version pins were found in story body prose:
- ADR-016 v0.7 cited in Task 4 dependency narrative
- BC-2.18.001 v1.7 cited in acceptance criterion context
- ADR-013 v0.5 cited in scheduling semaphore description
- BC-2.18.002 v1.4 cited in tick description

In S-4.05 v1.8, the following version pins were found:
- ADR-016 v0.7 cited in alert delivery prose
- ADR-015 v0.4 cited in detection rule reference
- BC-2.13.005 v0.3 cited in alert generation note

**Structural Prevention:** Remove ALL `vN.M` suffixes from ADR/BC cross-references in story bodies. Reference by ID only: `ADR-016`, `BC-2.18.001`. Version currency is the ADR/BC document's responsibility, not the story's.

**Remediation:** S-4.08 v1.18 → v1.19 (4 pins removed). S-4.05 v1.8 → v1.9 (3 pins removed).

---

### F-P11-M-001 — S-4.05 Stale ADR-016 v0.2 Reference in Pre-Remediation Body Section

**ID:** F-P11-M-001
**Severity:** MEDIUM
**Category:** Stale cross-reference
**File:** S-4.05
**Finding:** S-4.05 body contains a remnant `ADR-016 v0.2` reference in the alert dedup window description. ADR-016 is at v0.7 and the semantics changed significantly between v0.2 and v0.7 (idempotency_key, dead-letter CF key, retry-state row). The v0.2 pin misleads implementers about the dedup window resolution contract.

**Remediation:** Replace `ADR-016 v0.2` with `ADR-016` (no version pin, per F-P11-H-001 structural prevention). Covered by S-4.05 v1.9 bump.

---

### F-P11-M-002 — Dead-Letter Prose Case-Trigger Half Missing in S-4.08

**ID:** F-P11-M-002
**Severity:** MEDIUM
**Category:** Specification incompleteness
**File:** S-4.08
**Finding:** S-4.08 v1.18 extended the dead-letter prose in Task 4 to address the delivery-failure trigger (BC-2.18.001 EC-18-001). However, the case-trigger half of the dead-letter description — which specifies that a dead-lettered action MUST create a Case via the case management subsystem (per ADR-016 §2.5, BC-2.18.001 EC-18-003) — was only partially authored. The "dead-letter → case creation" linkage prose exists in one location but is absent from the AC summary list.

**Remediation:** S-4.08 v1.19 extends the dead-letter prose to explicitly cover the case-trigger half in both the body task and the relevant AC. Addressed as part of the v1.18 → v1.19 bump.

---

### F-P11-L-001 — BC-2.18.001 v1.6 Changelog Author Attribution Routing Process-Gap

**ID:** F-P11-L-001
**Severity:** LOW
**Category:** Agent routing process-gap (see TD-VSDD-038)
**File:** BC-2.18.001 (historical — pass-9 remediation)
**Finding:** BC-2.18.001 v1.6 changelog entry (line 184 of BC-2.18.001.md as of pass-9 state) attributes the BC body change to `state-manager` as author. Per STATE.md line 469 routing table, BC body/frontmatter edits route to `vsdd-factory:product-owner`. The same pattern was observed in pass-7 remediation for BC-2.12.004 v1.5 (also attributed to state-manager).

This is a systematic process-gap: when sweep bursts require line-level BC body edits as a side-effect of index updates, state-manager performs the edit directly rather than dispatching product-owner. This violates agent-routing discipline but is operationally efficient for small sweep classes.

**Remediation:** Filed as TD-VSDD-038 (agent routing edge cases for sweep bursts). No document change required for this finding — it documents a historical pattern. The TD captures the resolution options.

---

### F-P11-L-002 — AC-18 Trace Mis-Anchor in S-4.08

**ID:** F-P11-L-002
**Severity:** LOW
**Category:** Acceptance criterion traceability
**File:** S-4.08
**Finding:** S-4.08 v1.18 AC-18 references "dead-letter queue state" and traces it to VP-044 (Kani proof: action retry state machine). VP-044's scope is the retry state machine bounded by 5 attempts. The dead-letter terminal state IS within VP-044's scope, but AC-18's prose traces specifically to the queue-level observability assertion (VP-047 covers UUID v7 ordering; neither VP covers queue observability directly). The trace should reference BC-2.18.001 EC-18-003 as the normative anchor for dead-letter case creation, with VP-044 as the verification anchor for the retry-to-dead-letter transition.

**Remediation:** S-4.08 v1.19 re-anchors AC-18 to both BC-2.18.001 (normative) and VP-044 (verification). Addressed as part of the v1.18 → v1.19 bump.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 11 |
| **New findings** | 4 (F-P11-H-001, F-P11-M-001, F-P11-M-002, F-P11-L-002) |
| **Duplicate/variant findings** | 1 (F-P11-L-001 — routing process-gap, variant of Pass 7 BC-2.12.004 pattern) |
| **Novelty score** | 0.80 (4/5) |
| **Median severity** | 2.5 (between MEDIUM and HIGH) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5→5 |
| **Verdict** | FINDINGS_REMAIN |
