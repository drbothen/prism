# Demo Evidence Report — S-6.12

**Story:** S-6.12 — prism-dtu-pagerduty: DTU for PagerDuty Events API v2 — L3 (behavioral)
**Module:** `crates/prism-dtu-pagerduty`
**Branch:** `feature/S-6.12-dtu-pagerduty`
**Impl commit:** `96fb9e7e` (Red Gate / stub-author pre-implemented all routes)
**Dispatch note:** Step 4 (implementer) was skipped — stub-author delivered a fully-working
implementation. All 17 fidelity tests passed GREEN-BY-DESIGN at stub commit `96fb9e7e`.
Demo recording proceeded directly to Step 5.
**Recording date:** 2026-04-25
**Recorder:** Demo Recorder (claude-sonnet-4-6)

---

## BC Traceability Note

S-6.12 declares **no new product-level BCs** (`behavioral_contracts: []`). This is test
infrastructure — the crate exists to serve integration tests for S-4.08 (action delivery)
and S-5.06 (fire_action MCP tool). Architecture anchor is `dtu-assessment.md §3.5.2`
(PagerDuty scope matrix, stateful incident lifecycle, dedup key semantics).

---

## Coverage Table

| AC | Description | BC Ref | Test Name | Tape | GIF | Result |
|----|-------------|--------|-----------|------|-----|--------|
| AC-1 | POST /v2/enqueue trigger → HTTP 202 + status Triggered | dtu-assessment §3.5.2 | `test_full_lifecycle_trigger_ack_resolve` | [ac-1-trigger-event.tape](ac-1-trigger-event.tape) | [ac-1-trigger-event.gif](ac-1-trigger-event.gif) | PASS |
| AC-2 | POST /v2/enqueue acknowledge → HTTP 200 + status Acknowledged | dtu-assessment §3.5.2 | `test_full_lifecycle_trigger_ack_resolve` | [ac-2-acknowledge-event.tape](ac-2-acknowledge-event.tape) | [ac-2-acknowledge-event.gif](ac-2-acknowledge-event.gif) | PASS |
| AC-3 | POST /v2/enqueue resolve → HTTP 200 + status Resolved | dtu-assessment §3.5.2 | `test_full_lifecycle_trigger_ack_resolve` | [ac-3-resolve-event.tape](ac-3-resolve-event.tape) | [ac-3-resolve-event.gif](ac-3-resolve-event.gif) | PASS |
| AC-4 | acknowledge on Resolved incident → HTTP 400 | dtu-assessment §3.5.2 | `test_ac4_ack_on_resolved_returns_400` | [ac-4-ack-on-resolved-rejected.tape](ac-4-ack-on-resolved-rejected.tape) | [ac-4-ack-on-resolved-rejected.gif](ac-4-ack-on-resolved-rejected.gif) | PASS |
| AC-5 | re-trigger on active incident → HTTP 202 idempotent | dtu-assessment §3.5.2 | `test_ac5_trigger_idempotent_on_active_incident` | [ac-5-trigger-idempotent.tape](ac-5-trigger-idempotent.tape) | [ac-5-trigger-idempotent.gif](ac-5-trigger-idempotent.gif) | PASS |
| AC-6 | severity "fatal" (invalid) → HTTP 400 invalid severity | dtu-assessment §3.5.2 | `test_ac6_invalid_severity_returns_400` | [ac-6-invalid-severity-rejected.tape](ac-6-invalid-severity-rejected.tape) | [ac-6-invalid-severity-rejected.gif](ac-6-invalid-severity-rejected.gif) | PASS |
| AC-7 | missing routing_key → HTTP 400 missing routing_key | dtu-assessment §3.5.2 | `test_ac7_missing_routing_key_returns_400` | [ac-7-missing-routing-key-rejected.tape](ac-7-missing-routing-key-rejected.tape) | [ac-7-missing-routing-key-rejected.gif](ac-7-missing-routing-key-rejected.gif) | PASS |
| AC-8 | auth_mode=reject via /dtu/configure → HTTP 403 invalid key (maps to E-ACTION-AUTH-001) | dtu-assessment §3.5.2 | `test_ac8_auth_reject_mode_returns_403` | [ac-8-auth-reject-mode.tape](ac-8-auth-reject-mode.tape) | [ac-8-auth-reject-mode.gif](ac-8-auth-reject-mode.gif) | PASS |
| AC-9 | FailureMode::RateLimit → HTTP 429 + Retry-After: 60 | dtu-assessment §3.5.2 | `test_ac9_rate_limit_returns_429_with_retry_after` | [ac-9-rate-limit-429.tape](ac-9-rate-limit-429.tape) | [ac-9-rate-limit-429.gif](ac-9-rate-limit-429.gif) | PASS |

**Coverage: 9/9 ACs demonstrated. All PASS.**

---

## Edge Case Coverage (in fidelity test suite, not separately recorded)

| EC | Description | Test Name | Result |
|----|-------------|-----------|--------|
| EC-001 | trigger with no dedup_key → auto-generated UUID returned | `test_ec1_auto_generated_dedup_key` | PASS |
| EC-002 | resolve on unknown dedup_key → HTTP 400 invalid dedup_key | `test_ec2_resolve_unknown_dedup_key_returns_400` | PASS |
| EC-003 | re-trigger after resolve → fresh Triggered incident | `test_ec3_retrigger_after_resolve_creates_fresh_incident` | PASS |
| EC-004 | "CRITICAL" (wrong casing) → HTTP 400 (case-sensitive per PagerDuty spec) | `test_ec4_uppercase_severity_returns_400` | PASS |
| EC-005 | auth_mode=reject, then /dtu/reset clears it → 202 restored | `test_ec5_auth_reject_cleared_by_reset` | PASS |

---

## Full Fidelity Test Run Summary

**Total tests:** 17
**Passed:** 17
**Failed:** 0

All tests confirmed via `cargo test -p prism-dtu-pagerduty --features dtu --test fidelity`.

---

## Recording Methodology

- **Tool:** VHS 0.10.0 (`/opt/homebrew/bin/vhs`)
- **Font:** FiraCode Nerd Font Mono (matches S-2.02/S-2.03 convention)
- **Theme:** Dracula
- **Dimensions:** 1000x600, FontSize 14, Padding 20
- **Shell:** bash
- **AC-1/2/3:** Covered by a single lifecycle test (`test_full_lifecycle_trigger_ack_resolve`)
  which exercises trigger → acknowledge → resolve in sequence. Three separate tapes each
  filter the output to highlight the relevant AC's assertion lines.
- **AC-4 through AC-9:** Each covered by a dedicated per-AC test with exact name matching.
- **Error paths:** AC-4 (ack-on-resolved), AC-6 (invalid severity), AC-7 (missing routing_key),
  AC-8 (auth reject 403), AC-9 (rate limit 429) are all error-path demonstrations.
- **Success paths:** AC-1 (trigger 202), AC-2 (ack 200), AC-3 (resolve 200), AC-5 (idempotent
  re-trigger 202) are success-path demonstrations.

---

## File Inventory

```
docs/demo-evidence/S-6.12/
  ac-1-trigger-event.tape          (575 B)
  ac-1-trigger-event.gif           (120 KB)
  ac-2-acknowledge-event.tape      (573 B)
  ac-2-acknowledge-event.gif       (117 KB)
  ac-3-resolve-event.tape          (561 B)
  ac-3-resolve-event.gif           (115 KB)
  ac-4-ack-on-resolved-rejected.tape (548 B)
  ac-4-ack-on-resolved-rejected.gif  (106 KB)
  ac-5-trigger-idempotent.tape     (575 B)
  ac-5-trigger-idempotent.gif      (120 KB)
  ac-6-invalid-severity-rejected.tape (561 B)
  ac-6-invalid-severity-rejected.gif  (113 KB)
  ac-7-missing-routing-key-rejected.tape (563 B)
  ac-7-missing-routing-key-rejected.gif  (110 KB)
  ac-8-auth-reject-mode.tape       (556 B)
  ac-8-auth-reject-mode.gif        (112 KB)
  ac-9-rate-limit-429.tape         (558 B)
  ac-9-rate-limit-429.gif          (111 KB)
  evidence-report.md               (this file)
```

Total: 18 demo files (9 tape + 9 gif) + 1 evidence report = **19 files**.
