---
document_type: preflight-summary
phase: 4.A
producer: state-manager
timestamp: 2026-05-02T00:00:00Z
total_findings: 116
verdicts:
  consistency_drift: FAIL
  spec_quality: APPROVED_WITH_CONDITIONS
  uncertainty_scan: 14_HIGH_research_required
  architect_adr: 5_NEW_ADRS_PROPOSED
gate_decision: REMEDIATION_REQUIRED
---

# Wave 4 Phase 4.A Pre-Flight Summary

## Executive Verdict
**REMEDIATION REQUIRED before any story dispatch.** The 8 W4 stories drafted 2026-04-16/17 have systemic drift, unverified library assumptions, and quality gaps.

## Findings Roll-Up

| Pass | File | Verdict | H | M | L | KUDO |
|------|------|---------|---|---|---|------|
| Consistency / Drift | consistency-drift-audit.md | FAIL | 11 | 12 | 5 | — |
| Spec Quality | spec-quality-review.md | APPROVED_WITH_CONDITIONS | 6 | 21 | 12 | 8 |
| Uncertainty Scan | uncertainty-scan.md | 14 HIGH research items | 14 | 18 | 9 | — |
| Architect ADR Identification | architect-adr-identification.md | DISCOVERY | — | — | — | — |
| **TOTAL** | | | **31** | **51** | **26** | **8** |

## Top Critical Blockers (HIGH severity, dispatch-blocking)

### Drift / Architecture
1. **`prism-operations` crate does not exist** — all 8 stories assume it does. S-4.01 must add a workspace-registration task.
2. **Zero stories establish OrgId scoping** on Wave 4 domain types (ScheduleEntry, DiffResult, DetectionRule, Alert, Case, ActionSpec) — violates ADR-006/008.
3. **S-3.02 dependency unmerged** (status: draft) — blocks S-4.01, S-4.03, S-4.04.
4. **S-4.06 internal contradiction** — line 365 stale state-machine labels (Open → InProgress) contradicting BC-2.14.002 canonical (New → Acknowledged → Investigating → Resolved → Closed).

### Uncertainty / Library
5. **DataFusion 53 API surface unverified** (S-4.03, S-4.04) — UDF registration changed between major versions; brand-new dep.
6. **`cron` 0.12.x outdated** (S-4.08) — current 0.15+; 4 competing crates not evaluated.
7. **`blake3`, `lettre`, `libfuzzer-sys` not in Cargo.lock** — brand-new deps with no verification.

### Spec Quality
8. **S-4.06 + S-4.08 frontmatter↔manifest point disagreements** (5pt vs 9pt).
9. **S-4.03 under-sized** (5pt; IOC subsystem alone ≈3pt).
10. **All 8 stories missing `tdd_mode: strict`** (Wave 3 baseline).

## Architecture Open Questions (architect surfaced 7)

To resolve before ADR drafting:
- ADR-018 Diff Pack Format: standalone or merge into ADR-013 Schedule?
- (others — see architect-adr-identification.md)

## Cross-Cutting Hygiene (28 stories × N findings)

All 8 stories share:
- `cycle: "v1.0.0-greenfield"` (should be `wave-4-operations`)
- `tdd_mode: strict` missing
- `traces_to: []` empty
- `input-hash: "248b3b0"` predates Wave 3 ba3b10c7
- Unqualified `architecture/` path references

## Recommended Remediation Sequence

1. **Research dispatch** (research-agent, Context7+Perplexity): resolve 13 uncertainty research tasks → fixes most HIGH library findings.
2. **Architect open-questions resolution** with human (7 questions) → unblocks ADR drafting.
3. **Architect drafts ADR-013, 015, 016, 017** (and possibly 018) in parallel where deps allow → full VSDD per D-202/D-204.
4. **Story-writer drift remediation pass**: fix prism-operations crate-add task in S-4.01; sweep TenantId→OrgId; add OrgId scoping to all 8 stories; fix S-4.06 state-machine labels; fix dependency S-3.02 status (or descope/replace); align frontmatter (cycle, tdd_mode, traces_to, input-hash); update story bodies per uncertainty findings.
5. **Spec-quality remediation**: split S-4.06/S-4.08 sizing; fix AC measurability HIGHs; resolve frontmatter↔manifest point disagreements.
6. **Re-run pre-flight** (consistency-validator + spec-reviewer) → confirm verdicts → CONDITIONAL or PASS.
7. **3-clean adversarial spec convergence** on new ADRs/BCs + remediated stories.
8. **Consistency-validator + spec-reviewer fresh-context final** + input-hash drift check.
9. **Human approval gate** → Phase 4.A complete.
10. **Dispatch S-4.01 + S-4.03** in parallel (entry stories).

## Estimated Effort

- Research: 1 burst (13 tasks parallelizable)
- ADR authoring: 4-5 ADRs × full VSDD = significant
- Story remediation: 8 stories × ~5-10 substantive edits each
- Adversarial convergence: 3+ passes minimum

This is the largest pre-flight remediation pass to date in the project. D-202's "BLOCKING" decision is correctly identifying real risk.
