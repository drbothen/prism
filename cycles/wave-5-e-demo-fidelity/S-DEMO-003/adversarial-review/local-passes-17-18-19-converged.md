# S-DEMO-003 LOCAL Adversarial Cascade — Convergence Report
# Passes 17, 18, 19 — CONVERGED (BC-5.39.001 D-779)

**Story:** S-DEMO-003 — Demo Setup Scripts and Runbook
**Cycle:** wave-5-e-demo-fidelity
**Code HEAD at convergence:** `c61b61bd` (feature/S-DEMO-003)
**Story version at convergence:** v1.17
**BC-2.06.003 version:** v1.10 (DRAFT — promotes at merge per POL-14)
**Authority:** BC-5.39.001 D-779

---

## Convergence Verdict

CLEAN(strict): yes (all 3 passes)
CLEAN(PR-merge): yes (all 3 passes)
LOCAL streak: 3/3 CONVERGED
Novelty: ZERO

---

## Pass 17 — CLEAN(strict)

Date: 2026-06-07
Streak after: 1/3
Findings: 0 (zero of any severity)
Feature HEAD: c61b61bd

Notes: Pass-16 F-P16-MED-001 closure (cyberint auth_type api_key→cookie_roundtrip in
BC-2.06.003 v1.10) verified load-bearing. SAP-1 PASS (no unregistered event_type
emissions). SAP-2 PASS (DTU↔TOML schema parity). All 9 red-gate tests confirmed
present and load-bearing. Streak 0/3 → 1/3 per BC-5.39.001 D-779.

---

## Pass 18 — CLEAN(strict)

Date: 2026-06-07
Streak after: 2/3
Findings: 0 (zero of any severity)
Feature HEAD: c61b61bd

Notes: Full independent re-derivation of all story ACs, error-taxonomy E-CRED
namespace alignment (ADR-035 v1.2), boot-step-5 probe OrgId-keyed namespace
(BC-2.06.003 v1.10), and keyring backend singleton invariant (ADR-034 §D5).
SAP-1 PASS. SAP-2 PASS. Streak 1/3 → 2/3 per BC-5.39.001 D-779.

---

## Pass 19 — CLEAN(strict) — CONVERGENCE

Date: 2026-06-07
Streak after: 3/3 CONVERGED
Findings: 0 (zero of any severity)
Novelty: ZERO
Feature HEAD: c61b61bd

Notes: Holistic final pass — full AC coverage re-derived, spec↔impl↔index drift
confirmed zero, BC-2.06.003 v1.10 §Per-Sensor table correct (all four sensors),
boot-step-5 probe Tier-3a OrgId-keyed read confirmed, single shared KeyringBackend
ADR-034 §D5 confirmed, async CredentialRefProbe::probe confirmed. SAP-1 PASS.
SAP-2 PASS. Novelty ZERO. Streak 2/3 → 3/3 CONVERGED per BC-5.39.001 D-779.
LOCAL cascade CLOSED. NEXT: demo-recorder → push → pr-manager PR cycle →
PR-LEVEL 3-CLEAN → merge → POL-14 BC-2.06.003 draft→active.

---

## Full 19-Pass Trajectory Summary (restarted post D-1048 re-baseline)

| Pass | CLEAN(strict) | CLEAN(PR-merge) | Findings | Streak | Key Finding(s) Closed |
|------|---------------|-----------------|----------|--------|-----------------------|
| 1 | no | no | 1H + 1L | 0/3 | F-P1-HIGH-001 (DEMO-RUNBOOK §6b E-CRED mapping) + F-P1-LOW-001 (test doc-table) |
| 2 | no | no | 1H | 0/3 | F-P2-HIGH-001 (unused OrgSlug import masked by crate lint) |
| 3 | yes | yes | 0 | 1/3 | — |
| 4 | yes | yes | 0 | 2/3 | — |
| 5 | no | no | 1L | 0/3 | F-P5-LOW-001 (stale PrismCredentialResolver doc comments) |
| 6 | yes | yes | 0 | 1/3 | — |
| 7 | no | no | 1M + 1O | 0/3 | F-P7-MED-001 (stale constructor doc) + F-P7-OBS-001 (index path divergence) |
| 8 | no | no | 1M + 1L + 1O | 0/3 | F-P8 cluster (config-load shadowing — introduced by pass-7 fix) |
| 9 | no | no | 1M | 0/3 | F-P9-MED-001 (path dropped from org-resolution messages) |
| 10 | no | no | 1M + 1L | 0/3 | F-P10-MED-001 (runbook phantom claroty_assets table) + F-P10-LOW-001 (tilde-path doc) |
| 11 | yes | yes | 0 | 1/3 | — |
| 12 | no | no | 1H | 0/3 | F-P12-HIGH-001 (POL-13 status frontmatter drift — story file status: ready vs in_progress) |
| 13 | yes | yes | 0 | 1/3 | — |
| 14 | no | no | 1 CRIT | 0/3 | F-P14-CRIT-001 (CRITICAL — boot-step-5 probe OrgId-namespace mismatch; demo-unbootable; D-1050) |
| 15 | no | no | 2H | 0/3 | F-P15-HIGH-001 (duplicate KeyringBackend ADR-034 §D5 violation + fabricated deferral comment) + F-P15-HIGH-002 (async-signature spec claim drift; D-1051) |
| 16 | no | no | 1M | 0/3 | F-P16-MED-001 (cyberint auth_type api_key→cookie_roundtrip in BC-2.06.003; D-1052) |
| 17 | yes | yes | 0 | 1/3 | — CONVERGING |
| 18 | yes | yes | 0 | 2/3 | — CONVERGING |
| 19 | yes | yes | 0 | 3/3 | — CONVERGED |

Streak resets: 6 (at passes 1→reset, 2→reset, 5→reset, 7→reset, 9→reset, 12→reset, 14→reset, 15→reset, 16→reset)
Total findings closed: ~16 (1 CRIT + 5 HIGH + 4 MED + 4 LOW + 2 OBS)
Most critical finding: F-P14-CRIT-001 — demo-unbootable OrgId-namespace mismatch in boot-step-5 probe
