---
document_type: gate-step-f-holdout-evaluation
level: ops
version: "1.0"
status: complete
producer: holdout-evaluator
timestamp: 2026-05-02T19:00:00Z
phase: 3
inputs:
  - .factory/holdout-scenarios/HS-003-multi-tenant.md
  - .factory/holdout-scenarios/HOLDOUT-INDEX.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-f-holdout-evaluation-pass3.md
  - crates/prism-dtu-harness/tests/logical_isolation_test.rs
  - crates/prism-dtu-harness/tests/network_isolation_test.rs
  - crates/prism-dtu-armis/tests/cr017_tag_alert_org_id_guard.rs
  - crates/prism-dtu-armis/tests/multi_tenant.rs
  - crates/prism-dtu-armis/tests/x_org_id_auth.rs
  - crates/prism-dtu-crowdstrike/tests/cr018_detections_org_id_guard.rs
  - crates/prism-dtu-crowdstrike/tests/multi_tenant.rs
  - crates/prism-dtu-crowdstrike/tests/x_org_id_auth.rs
  - crates/prism-customer-config/tests/sec_p3_001_inline_table_redaction.rs
  - crates/prism-customer-config/tests/sec_p3_002_pipe_anchor.rs
  - crates/prism-audit/tests/sec007_org_slug_cross_check.rs
input-hash: "45aeaea"
traces_to: prd.md
pass: 4
previous_evaluation: gate-step-f-holdout-evaluation-pass3.md
verdict: PASS
mean_satisfaction: 0.886
must_pass_ratio: "27/30"
---

# Gate Step F — Holdout Evaluation: Prism Wave 3 (Pass 4)

## Summary

| Metric | Value |
|--------|-------|
| Pass | 4 |
| Date | 2026-05-02 |
| Verdict | PASS |
| Mean satisfaction | 0.886 |
| Must-pass ratio | 27/30 ABOVE_BAR |
| Holdout scenario | HS-003 (multi-tenant) |
| Bar threshold | 0.85 / 26-of-30 |
| Result | ABOVE_BAR — holdout gate PASSES |

## Pass-3 Fix Verification

| Gap | Status | Closed By |
|----|--------|-----------|
| SEC-P3-001 TOML inline-table redaction | RESOLVED | W3-FIX-SEC-004 PR #122 (4e053105) |
| SEC-P3-002 pipe-finder anchor | RESOLVED | W3-FIX-SEC-004 PR #122 |
| SEC-P3-003 constant-time admin token | RESOLVED | W3-FIX-SEC-004 PR #122 |
| CR-016 poll cadence sibling gap | RESOLVED | W3-FIX-CODE-005 PR #123 (e4be29ae) |
| CR-017 Armis sibling endpoint org-id guard | RESOLVED | W3-FIX-CODE-005 PR #123 |
| CR-018 CrowdStrike detections org-id guard | RESOLVED | W3-FIX-CODE-005 PR #123 |

## Per-scenario scoring

| Sub-scenario | Score | Evidence |
|---|---|---|
| HS-003-01 Tenant data isolation | 0.95 | DTU-harness logical_isolation 33/33 + network_isolation 14/14; Armis multi_tenant 11/11; CrowdStrike 14/14; Claroty/Cyberint multi_tenant suites all green |
| HS-003-02 Tenant ID spoofing prevention | 0.95 | x_org_id_auth across Armis/CrowdStrike/Claroty/Cyberint all pass; BC-3.1.001/3.1.002 all pass; sec007_org_slug_cross_check all pass |
| HS-003-03 Cache isolation between tenants | 0.85 | Per-org session/state isolation tests pass; reset_for selective; HTTP-level cross-org cache miss validated by proptests |
| HS-003-04 Cursor state isolation | 0.85 | Reset_for selectivity tests + HTTP reset_for selective tests pass; no path traversal; no cross-org write effect (proptests); BC-3.3.004 enforced |
| HS-003-05 Error message tenant isolation | 0.90 | sec_p3_001 inline-table redaction (6/6); sec_p3_002 pipe anchor (5/5); sec006 multiline (5/5); BC-3.2.002 credential_value_not_in_error_message passes |
| HS-003-06 Per-tenant rate limiting / admin auth | 0.80 | sec_p3_003 constant-time admin token (7/7); ac_6_rate_limit_429 + CrowdStrike rate-limit green; cross-tenant quota exhaustion still indirect |
| HS-003-07 Log field isolation / sibling endpoints | 0.90 | W3-FIX-CODE-005 — cr017 Armis tags/alerts/activity/risk org-id guards 8/8; cr018 CrowdStrike detections 6/6; sec007 audit cross-check 7/7 |

**Aggregate mean_satisfaction:** (0.95+0.95+0.85+0.85+0.90+0.80+0.90)/7 = 0.886

## Verdict

PASS — mean_satisfaction 0.886 ≥ 0.85 bar; must_pass_ratio 27/30 ≥ 18/30 bar; sustains and incrementally improves on pass-3 (0.86 / 26-of-30).

ABOVE_BAR (vs pass-3):
- HS-003-07 sibling endpoint coverage now broad (Armis tags/alerts/activity/risk + CrowdStrike detections all guarded)
- HS-003-05 TOML redaction edge cases (inline-tables + pipe-anchored finder closed via PR #122)
- HS-003-02 constant-time admin token comparison reinforces spoofing prevention surface

Residual non-blocking:
- BC-3.5.001/002 timing tests `#[ignore]` under TD-W3-TIMING-001
- HS-003-06 lacks dedicated cross-tenant API-quota exhaustion soak test
