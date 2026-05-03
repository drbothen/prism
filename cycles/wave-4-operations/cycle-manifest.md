---
document_type: cycle-manifest
cycle_id: wave-4-operations
cycle_type: feature
version: wave-4-preflight-v1.5
status: in-progress
started: pending
completed: pending
producer: state-manager
predecessor_cycle: wave-3-multi-tenant (CONVERGED 2026-05-02)
---

# Cycle Manifest: Wave 4 Operations (Pre-Flight)

## Delivered

_Not yet started — pre-flight plan only._

| Metric | Value |
|--------|-------|
| Stories delivered | — (0 of 8 dispatched) |
| BCs created | — |
| VPs created | — |
| Holdout scenarios | TBD (HS-OPS-001 or equivalent) |
| Total cost | — |
| Adversarial passes | — |
| Final holdout satisfaction | — |
| Release version | wave-4-operations |

## Spec Changes

_None yet — pre-flight phase only._

| Artifact | Change | Before | After |
|----------|--------|--------|-------|
| — | — | — | — |

## Living Spec Snapshot

_To be captured at Wave 4 gate convergence._

Captured at: (pending — git tag wave-4-operations on factory-artifacts branch at convergence)
Retrieve: git show wave-4-operations:specs/prd.md

## Deprecations (if any)

_None identified at pre-flight. TenantId alias removal (Wave 3 D-157 deferral) may produce deprecations during Wave 4._

| Artifact | Deprecated By | Replacement | Sunset Date |
|----------|--------------|-------------|-------------|
| TenantId alias | Wave 4 (per D-157) | OrgId (D-041) | wave-4-operations |

## Tech Debt Created

_None yet — pre-flight. TD-W3-* carry-forwards from Wave 3 listed below for bucketing decision._

| ID | Description | Priority | Source |
|----|-------------|----------|--------|
| TD-W3-TIMING-001 | BC-3.5.001/002 wall-clock budget tests #[ignore] — Criterion bench migration needed | P2 | Wave 3 carry-forward |
| TD-W3-QUOTA-SOAK-001 | Cross-tenant API quota soak test gap (HS-003-06) | P3 | Wave 3 carry-forward |
| TD-W3-CT-EQ-COVERAGE-001 | Non-DTU non-constant comparison audit (sweep beyond DTU clone routes) | P3 | Wave 3 carry-forward |

## Governance Policies Adopted

_None yet — pre-flight._

| Policy | Adopted In | Incident Reference | Generalization |
|--------|-----------|-------------------|----------------|
| — | — | — | — |

## Notes

### Wave 4 Charter

**Theme:** Operations layer — schedule execution, detection rules, alerting, case management, action delivery.
**Crate:** `prism-operations`
**Entry baseline:** develop@ba3b10c7 (Wave 3 CONVERGED 2026-05-02) / factory-artifacts@b3a9d5bf

Wave 4 introduces the operations runtime: scheduled sensor polling, differential result packs, detection rule evaluation, alert generation, case lifecycle management, and outbound action delivery. This is the first wave that produces externally-visible operational outputs (alerts, cases, notifications).

### Wave 4 Story Inventory

8 stories — all status: draft, all P0.

| Story | Title | Pts | BCs | VPs | Layer | Depends On |
|-------|-------|-----|-----|-----|-------|------------|
| S-4.01 | Schedule CRUD and Execution Loop | 5 | 5 | VP-026, VP-030 | 3 | S-3.02, S-2.01 |
| S-4.02 | Differential Results and Packs | 5 | 2 | VP-019 | 2 | S-4.01 |
| S-4.03 | Detection Rule Loading and Compilation | 8 | 5 | VP-018 | 3 | S-3.02, S-1.08, S-2.01 |
| S-4.04 | Detection Evaluation (Single/Correlation/Sequence) | 5 | 5 | VP-027 | 3 | S-4.03 |
| S-4.05 | Alert Generation | 1 | 2 | VP-028 | 1 | S-4.04 |
| S-4.06 | Case Management | 9 | 5 | VP-052/53/54/60 | 3 | S-4.05, S-2.01 |
| S-4.07 | Case Metrics + Acknowledge Alert | 3 | 3 | — | 2 | S-4.06 |
| S-4.08 | Action Delivery Framework | 9 | 5 | VP-044/45/46/47 | 3 | S-4.05, S-4.06, S-4.01, S-1.15, S-6.11/12/13 |

**Total story points:** 45
**Total BCs covered:** ~32 unique (per STORY-INDEX wave-4 raw count = 45)

### Topology / Dispatch Order

```
ENTRY (parallel — both depend only on merged Wave 1-3):
├─ S-4.01 (Schedule CRUD)
└─ S-4.03 (Detection Rule Loading)

CHAIN A (Schedule -> Diff):
S-4.01 -> S-4.02

CHAIN B (Detection -> Alert -> Case -> Metrics):
S-4.03 -> S-4.04 -> S-4.05 -> S-4.06 -> S-4.07

TERMINAL (Action Delivery — joins both chains + DTU deps):
S-4.05 + S-4.06 + S-4.01 + S-1.15 + S-6.11/12/13 -> S-4.08
```

Parallelism opportunities:
- **Group A (entry):** {S-4.01, S-4.03} — dispatch simultaneously after pre-flight green
- **Group B (mid):** {S-4.02 after 4.01, S-4.04 after 4.03} — dispatch simultaneously when respective dependencies merge
- **Group C (mid-late):** {S-4.05 after 4.04}
- **Group D (terminal):** {S-4.06 + S-4.07 chain}, then {S-4.08 after all join conditions met}

### Pre-Flight Checklist (BLOCKING before story dispatch)

| Check | Owner | Status | Notes |
|-------|-------|--------|-------|
| Spec-drift audit on 8 W4 stories | spec-drift-analyzer | COMPLETE — FAIL (11H/12M/5L) | See preflight-findings/consistency-drift-audit.md |
| Uncertainty scanner on each story | uncertainty-scanner | COMPLETE — 14 HIGH research required | See preflight-findings/uncertainty-scan.md; 13 research tasks queued |
| Research dispatch (13 tasks) | research-agent | COMPLETE | See preflight-findings/research-findings.md |
| New ADRs needed? | architect + human | COMPLETE — 6 ADRs confirmed (D-207) | ADR-013/015/016/017/018/019; architect authoring queued; phased parallel plan per D-207 |
| Architectural decisions logged | state-manager | COMPLETE (D-207..213) | D-207..D-213 logged 2026-05-02; canonical: STATE.md |
| Spec quality review on 8 W4 stories | spec-reviewer | COMPLETE — APPROVED_WITH_CONDITIONS (6H/21M/12L/8K) | See preflight-findings/spec-quality-review.md |
| Story re-validation (status: draft -> ready) | product-owner / story-writer | NOT_STARTED | Gated on ADR acceptance + drift remediation |
| BC anchor validation against current BC-INDEX | product-owner | NOT_STARTED | Gated on findings remediation |
| TenantId -> OrgId references in story bodies | story-writer | NOT_STARTED | Gated on findings remediation; all 8 stories need OrgId scoping on domain types |
| ADR-006..012 + new ADR refs cited in stories | spec-drift-analyzer | NOT_STARTED | Gated on ADR acceptance |
| Spec-first phasing (D-045 analog) decision | human | COMPLETE — D-202 | DRIFT-REMEDIATION + FULL VSDD ON NEW SPECS; BLOCKING for implementation |
| Carry-forward debt placement | human | COMPLETE — D-203 | REMEDIATE ALL as W4-FIX-* candidates |

### Spec-First Discipline (D-045 Analog) — Decision Required

Wave 3 was spec-first BLOCKING (Phase 3.A required full spec convergence + human approval before any implementation). Wave 4 question: are W4 stories sufficiently spec'd (drafted 2026-04-16, ADRs 011/012 cover harness + workspace), or do we need new ADRs for schedule semantics, detection rule language, action delivery framework, and case state machine?

**Human input required.** Default recommendation: light spec refresh + targeted new ADRs only if existing stories surface gaps.

### Architecture Gates (if Phase 4.A adopted)

1. ADR drafts for new architectural decisions (schedule, detection, action delivery)
2. BC drafts for new behavioral contracts (BC-4.x family)
3. 3-clean adversarial spec convergence
4. Consistency-validator fresh-context pass
5. Spec-reviewer sign-off
6. Input-hash drift check
7. Human approval gate

### Convergence Targets (Wave 4 Gate)

Mirror Wave 3 gate criteria:
- 3-clean adversarial integration gate window
- All sub-reviewers CLEAN (code, security, consistency, holdout)
- Holdout HS-OPS-001 or equivalent (TBD): mean >= 0.85, must-pass >= 18/30
- 0 CRITICAL / 0 HIGH / 0 MEDIUM at convergence

### Open Questions for Human Approval

1. **Spec-first BLOCKING (Phase 4.A) yes/no?** — ANSWERED: See §9.1 below.
2. **Which W3 carry-forward debt becomes W4-FIX-* vs deferred?** — ANSWERED: See §9.1 below.
3. **New ADRs needed** (schedule semantics, detection lang, action framework, case state machine)? — ANSWERED: See §9.1 below.
4. **Wave 4 wave-cycle name:** `wave-4-operations` — confirm? — ANSWERED: See §9.1 below.

### 9.1 Human-Approved Answers (2026-05-02)

Decisions D-202..D-205 logged in STATE.md v6.19.

| Q | Question | Answer | Decision |
|---|----------|--------|----------|
| Q1 | Spec-first phasing? | DRIFT-REMEDIATION + FULL VSDD ON NEW SPECS (effectively BLOCKING) — all 8 W4 stories must be drift-audited + remediated before dispatch; all new ADRs/BCs follow full VSDD process (3-clean adversarial convergence → consistency-validator → spec-reviewer → input-hash → human approval) | D-202 |
| Q2 | Carry-forward debt bucketing? | REMEDIATE ALL — W4-FIX-* fix-wave stories for each: TD-W3-TIMING-001→W4-FIX-PERF-001, TD-W3-QUOTA-SOAK-001→W4-FIX-PERF-002, TD-W3-CT-EQ-COVERAGE-001→W4-FIX-CODE-001, SEC-P3-004/005/006/SEC-005→W4-FIX-SEC-001..004. TD-S-1.07-01 (P1 KeyringBackend) NOT closed in Wave 4 (Wave 5 prerequisite). | D-203 |
| Q3 | New ADRs needed? | YES — architect identifies and authors all needed ADRs; likely candidates: ADR-013 (Schedule semantics), ADR-014 (Detection rule language), ADR-015 (Action delivery framework), ADR-016 (Case state machine); additional ADRs per architect spec-drift audit findings; all follow full VSDD per D-202 | D-204 |
| Q4 | Cycle name? | `wave-4-operations` confirmed | D-205 |

### Wave 5 Prerequisite (DO NOT close in Wave 4)

TD-S-1.07-01 (P1): KeyringBackend production wire-up — must be resolved before Wave 5 gate closes.

---

## 10. Methodology Innovation Disclosure

This pre-flight cycle-manifest is itself a methodology innovation. Wave 3 (and prior waves) did NOT receive a pre-flight artifact before story dispatch — Wave 3 was kicked off via D-040..D-046 decisions in STATE.md plus a "Wave 3 Approved Plan" table embedded in SESSION-HANDOFF.md.

The patterns introduced here (pre-flight charter, story inventory, topology, blocking checklist, spec-first decision, open questions, resume steps) are pending vsdd-factory codification:

| TD ID | Methodology Pattern | Target |
|-------|---------------------|--------|
| TD-VSDD-035 | Pre-flight cycle-manifest as wave-kickoff artifact | `/vsdd-factory:author-wave-preflight` skill |
| TD-VSDD-036 | Per-wave spec-first phasing decision (BLOCKING / DRIFT-AUDIT / NON-BLOCKING) | wave-gate skill + policy registry |
| TD-VSDD-037 | Cross-wave carry-forward debt bucketing protocol | state-manager skill |

These TDs are filed in `.factory/vsdd-plugin-tech-debt.md` (out of Wave 4 scope) and will be addressed during a future vsdd-factory plugin maintenance cycle. Until codified, the pre-flight pattern is "stable but unofficial."

**2026-05-02 update:** §9.1 Human-Approved Answers were provided 2026-05-02 and decisions D-202..D-205 codify them in STATE.md v6.19. The methodology TD entries (TD-VSDD-035/036/037) remain pending vsdd-factory codification.

---

## 11. Architectural Decisions Logged (2026-05-02)

D-207..D-213 logged in STATE.md v6.21 following research completion and architect open-questions resolution. Canonical content in STATE.md Decisions Log.

| Decision | Synopsis |
|----------|----------|
| D-207 | 6-ADR topology for Wave 4: ADR-013/015/016/017/018/019 in 3 phased parallel rounds. ADR-019 (SIEM Output Formats) added per D-212. |
| D-208 | OrgId/ClientId dual hierarchy retained. All 8 W4 domain types gain `org_id: OrgId`; RocksDB CF keys gain `{org_id}:` prefix per ADR-008. `Client(ClientId)` references become `Client(OrgId, ClientId)`. |
| D-209 | Per-subsystem semaphores: 8 permits for S-4.01 (schedule), 8 permits for S-4.08 (action delivery). No shared semaphore; eliminates cross-subsystem starvation. |
| D-210 | `clients = []` in `.action.toml` is a config error, rejected at validation. Org-wide broadcast requires explicit sentinel (`clients = ["*"]` or `scope = "all"`; architect picks canonical form in ADR-016). |
| D-211 | Alert dedup window resolved at scheduling-time, baked into `RuleCondition`. Schedule CRUD invalidates cached resolutions; rules reload. Eliminates per-eval OrgRegistry round-trip. |
| D-212 | Build `prism-siem-formats` in-house: `cef::v0::Encoder` + `leef::v2::Encoder`. No maintained Rust crates exist (rust-cef abandoned 2021). Proptest fuzz invariants per ArcSight CEF Standard + IBM LEEF v2 Guide. |
| D-213 | ADR-017 narrative: "1898-curated, industry-informed" — cites NIST 800-61 r2 (not r3; r3 abandoned state-machine April 2025), ITIL v3, Cortex XSOAR, Splunk SOAR. prism-core::case NOT reworked (Kani proofs VP-005/006/051 lock 12-transition table). |

Story inventory will be remediated post-ADR-acceptance per drift audit categories K, I, D, M, F + research-findings library updates.

---

## Changelog

| Version | Date | Change |
|---------|------|--------|
| wave-4-preflight-v1.5 | 2026-05-02T20:00:00Z | Phase 1 ADRs committed (ADR-013 + ADR-017 PROPOSED v0.1); VP-137 + VP-138 stubs added. Phase 2 (ADR-015 + ADR-018) queued. |
| wave-4-preflight-v1.4 | 2026-05-02T15:00:00Z | Architectural decisions D-207..D-213 logged. Research complete (research-findings.md committed). 6-ADR topology decided (D-207); OrgId/ClientId hierarchy confirmed (D-208); per-subsystem semaphores 8/8 (D-209); clients=[] reject (D-210); dedup scheduling-time (D-211); prism-siem-formats in-house (D-212); ADR-017 narrative 1898-curated (D-213). §11 Architectural Decisions Logged added. Pre-Flight Checklist updated. STATE v6.20→v6.21. |
| wave-4-preflight-v1.3 | 2026-05-02T14:00:00Z | All 4 pre-flight passes complete. D-206 logged: 116 findings (31H/51M/26L/8K). Preflight-findings/ directory: architect-adr-identification.md (5 ADRs proposed), consistency-drift-audit.md (FAIL), spec-quality-review.md (APPROVED_WITH_CONDITIONS), uncertainty-scan.md (14H; 13 research tasks), preflight-summary.md. Pre-Flight Checklist updated. STATE v6.19→v6.20. |
| wave-4-preflight-v1.2 | 2026-05-02T13:00:00Z | §9.1 Human-Approved Answers (2026-05-02) added. Decisions D-202..D-205 codified. Phase 4.A entered. §10 annotation added. STATE v6.18→v6.19. |
| wave-4-preflight-v1.1 | 2026-05-02T12:00:00Z | Section 10 Methodology Innovation Disclosure added. TD-VSDD-035/036/037 filed in vsdd-plugin-tech-debt.md per user catch (2026-05-02). |
| wave-4-preflight | 2026-05-02 | Initial pre-flight plan authored by state-manager. 8 stories inventoried (all status: draft, P0). Topology, dispatch order, pre-flight checklist, and open questions documented. |
