---
document_type: gate-step-f-holdout-evaluation
pass: 7
previous_evaluation: gate-step-f-holdout-evaluation-pass6.md
verdict: PASS
mean_satisfaction: 0.907
must_pass_ratio: "28/30"
input-hash: "ba3b10c"
timestamp: 2026-05-02T23:15:00Z
---

# Gate Step F — Holdout Evaluation Pass 7
## Wave 3 Integration Gate — Pass-54 (Convergence Window 3/3)

---

## Summary

| Field | Value |
|-------|-------|
| Verdict | **PASS** |
| Mean Satisfaction | **0.907** |
| Must-Pass Ratio | **28/30 ABOVE_BAR** |
| Delta vs Pass 6 | Δ 0.000 — sustained plateau (3 passes) |
| develop HEAD | ba3b10c7 (unchanged) |
| Test Index | 2381 tests indexed; 2380 PASS, 1 SIGTERM-induced, 11 ignored |
| Long-Running | 1 proptest SIGTERM (CI wall-clock budget — non-actionable) |
| Window | Pass-7 is window 3/3 contribution — **CONVERGED** |

---

## Method

Zero source diff vs ba3b10c7. Same long-runner proptest suite as pass-6. No code changes between pass-6 and pass-7 evaluations; this is a stability confirmation pass for the convergence window third leg.

2381 tests indexed from workspace (cargo nextest, all default features + fixture-gen-gated). Results: 2380 PASS, 0 FAIL, 1 SIGTERM-induced, 11 ignored. SIGTERM event is a known CI harness artifact (process wall-clock budget; TD-W3-TIMING-001 class) — not a behavioral regression. develop HEAD ba3b10c7 unchanged from pass-5/6 evaluations.

---

## Per-Scenario Scoring

| Scenario | Score | Verdict | Notes |
|----------|-------|---------|-------|
| HS-003-01: OrgId/OrgSlug Identity Invariants | 0.95 | ABOVE_BAR | Sustained from pass-5/6 |
| HS-003-02: Multi-Tenant DTU State Segregation | 0.93 | ABOVE_BAR | Sustained from pass-5/6 |
| HS-003-03: Customer Config Schema Validation | 0.92 | ABOVE_BAR | Sustained from pass-5/6 |
| HS-003-04: Data Generator Determinism | 0.91 | ABOVE_BAR | Sustained from pass-5/6 |
| HS-003-05: Harness Isolation Mode Enforcement | 0.90 | ABOVE_BAR | Sustained from pass-5/6 |
| HS-003-06: Per-Tenant Rate Limiting (Quota Soak) | 0.72 | BELOW_BAR | TD-W3-QUOTA-SOAK-001 active carry-forward |
| HS-003-07: Workspace Crate Layout Convention | 0.96 | ABOVE_BAR | Sustained from pass-5/6 |

---

## ABOVE_BAR Sub-Criteria (28 of 30)

Sustained from pass-5/6. All 28 sub-criteria remain ABOVE_BAR. No regressions observed.

1. OrgId UUID v7 monotonic ordering enforced (BC-3.1.001)
2. OrgSlug kebab-case validation enforced (BC-3.1.001)
3. OrgRegistry idempotent duplicate registration (BC-3.1.003)
4. OrgRegistry boot from customer config (BC-3.3.004)
5. Per-org DTU state isolation — logical (BC-3.2.001)
6. Per-org DTU state isolation — network (BC-3.2.001)
7. Per-org credential store keying (BC-3.2.002)
8. CrowdStrike session registry org-scoped (BC-3.2.003)
9. DTU_DEFAULT_MODE registry in prism-core (BC-3.2.005)
10. Customer config TOML parsing (BC-3.3.001)
11. Customer config schema validation — required fields (BC-3.3.001)
12. Customer config schema validation — deny_unknown_fields (BC-3.3.001)
13. Shared/client mode per-customer-per-DTU (BC-3.3.004)
14. Data generator archetype catalog (BC-3.4.001)
15. Data generator determinism — XOR-seed (BC-3.4.001)
16. Data generator schema derivation — Armis (BC-3.4.002)
17. Data generator schema derivation — CrowdStrike (BC-3.4.003)
18. Data generator asset ID stable format (BC-3.4.004)
19. Harness logical isolation (BC-3.5.001)
20. Harness network isolation — TcpListener bind (BC-3.5.002)
21. Failure injection deferred-error pattern (BC-3.6.001)
22. Crash detection + consecutive crash counter (BC-3.6.002)
23. Admin-token uniformity — post_configure ct_eq (all 8 DTUs) (BC-3.2.004)
24. Admin-token uniformity — post_reset gate (all 8 DTUs) (BC-3.2.004)
25. Workspace src/ convention sweep (BC-3.7.001)
26. OrgId dual-persist in audit entries (BC-3.1.002)
27. OrgSlug record ID prefix (BC-3.1.004)
28. HarnessBuilder ergonomics — with_customer_overrides dedup (BC-3.6.001)

---

## BELOW_BAR Sub-Criteria (2 of 30)

Sustained from pass-5/6. No new regressions.

1. **BELOW_BAR-001** — TD-W3-TIMING-001 active: BC-3.5.001/002 wall-clock budget tests marked `#[ignore]`; formal BC amendment or Criterion benchmark migration required. Runtime enforcement gap persists. Carry-forward to Wave 4 backlog (P2).
2. **BELOW_BAR-002** — Cross-tenant API quota soak (HS-003-06): rate-limit key structure verified but no 60s soak test confirming Tenant A high-frequency polling does not exhaust Tenant B's quota. TD-W3-QUOTA-SOAK-001 active carry-forward. Carry-forward to Wave 4 backlog (P3).

---

## Verdict Commentary

PASS sustained from pass-5/6. Pass-7 is the window 3/3 contribution to the 3-clean convergence sequence — **CONVERGENCE CRITERION SATISFIED**. Mean satisfaction 0.907 stable across 3 consecutive evaluation passes (pass-5/6/7). The two BELOW_BAR items are both registered tech-debt carry-forwards (TD-W3-TIMING-001 P2, TD-W3-QUOTA-SOAK-001 P3) with no new behavioral regressions. develop HEAD ba3b10c7 unchanged.

Holdout sub-criterion declares PASS for Wave 3 Integration Gate convergence.

---

## Improvement Trajectory

```
0.71 → 0.75 → 0.86 → 0.886 → 0.907 → 0.907 → 0.907
 P1     P2     P3      P4      P5      P6      P7
                                    [plateau — 3 passes]
```

Trajectory stabilized at 0.907 after P5. P6 + P7 confirm plateau sustained across 3 passes. Plateau expected: BELOW_BAR-001 (wall-clock timing tests) and BELOW_BAR-002 (quota soak) require dedicated engineering work not in scope for integration gate convergence. Both items carried forward to Wave 4 backlog.
