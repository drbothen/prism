# Demo Evidence Report — S-6.11

**Story:** S-6.11 — prism-dtu-slack: DTU for Slack Webhook API — L2 (stateful)
**Module:** `crates/prism-dtu-slack`
**Branch:** `feature/S-6.11-dtu-slack`
**Impl commit:** `878ee294` (FailureLayer 429 body fix — the legitimate TDD signal in this story)
**Dispatch note:** 13 of 14 tests were GREEN-BY-DESIGN at the Red Gate stub commit (stub-author
fully pre-implemented the Slack webhook handler per the established DTU pattern). 1 test was
genuinely RED: `ac_4_rate_limit_returns_429_with_retry_after_and_ratelimited_body` asserted the
body literal `"ratelimited"` but the `FailureLayer` in `prism-dtu-common` was emitting an empty
body. Fixed at `878ee294` by updating `prism-dtu-common/src/layers/failure.rs`. Cross-crate
impact: sibling DTU clones (S-6.12, S-6.13) only asserted on status code, not body, so no
regression.
**Recording date:** 2026-04-25
**Recorder:** Demo Recorder (claude-sonnet-4-6)

---

## BC Traceability Note

S-6.11 declares **no new product-level BCs** (`behavioral_contracts: []`). This is test
infrastructure — the crate exists to serve integration tests for S-4.08 (Slack action delivery)
and S-5.06 (fire_action MCP tool). Architecture anchor is `dtu-assessment.md §3.5.1`
(Slack scope matrix, Block Kit validation, rate-limit behavior, payload capture test API).

---

## Coverage Table

| AC | Description | BC Ref | Test Name(s) | Tape | GIF | Result |
|----|-------------|--------|--------------|------|-----|--------|
| AC-1 | POST /services/{token} with valid Block Kit `blocks` → HTTP 200 `ok=true` + stable `message_ts` | dtu-assessment §3.5.1 | `ac_1_valid_blocks_payload_returns_200_ok_with_stable_message_ts`, `ac_1_text_only_payload_returns_200` | [ac-1-valid-blocks-payload.tape](ac-1-valid-blocks-payload.tape) | [ac-1-valid-blocks-payload.gif](ac-1-valid-blocks-payload.gif) | PASS |
| AC-2 | POST /services/{token} missing both `blocks` and `text` → HTTP 400 `"invalid_payload"` | dtu-assessment §3.5.1 | `ac_2_missing_blocks_and_text_returns_400_invalid_payload` | [ac-2-missing-blocks-and-text.tape](ac-2-missing-blocks-and-text.tape) | [ac-2-missing-blocks-and-text.gif](ac-2-missing-blocks-and-text.gif) | PASS |
| AC-3 | POST /services/{token} with unknown top-level field → HTTP 400 `"unknown_field"` | dtu-assessment §3.5.1 | `ac_3_unknown_top_level_field_returns_400_unknown_field`, `ac_3_all_allowed_top_level_fields_are_accepted` | [ac-3-unknown-top-level-field.tape](ac-3-unknown-top-level-field.tape) | [ac-3-unknown-top-level-field.gif](ac-3-unknown-top-level-field.gif) | PASS |
| AC-4 | FailureMode::RateLimit after 3 requests → HTTP 429 + `Retry-After: 30` + body `"ratelimited"` | dtu-assessment §3.5.1 | `ac_4_rate_limit_returns_429_with_retry_after_and_ratelimited_body` | [ac-4-rate-limit-429.tape](ac-4-rate-limit-429.tape) | [ac-4-rate-limit-429.gif](ac-4-rate-limit-429.gif) | PASS (was RED, fixed at 878ee294) |
| AC-5 | 3 successful deliveries → GET /dtu/received-payloads returns all 3 in order | dtu-assessment §3.5.1 | `ac_5_three_deliveries_captured_in_order`, `ac_5_in_process_received_payloads_api_matches_http_endpoint` | [ac-5-deliveries-captured-in-order.tape](ac-5-deliveries-captured-in-order.tape) | [ac-5-deliveries-captured-in-order.gif](ac-5-deliveries-captured-in-order.gif) | PASS |
| AC-6 | reset() clears `received_payloads` and rate-limit counter to 0 | dtu-assessment §3.5.1 | `ac_6_reset_clears_received_payloads_and_request_counter`, `ac_6_post_dtu_reset_endpoint_clears_state` | [ac-6-reset-clears-state.tape](ac-6-reset-clears-state.tape) | [ac-6-reset-clears-state.gif](ac-6-reset-clears-state.gif) | PASS |

**Coverage: 6/6 ACs demonstrated. All PASS.**

---

## TDD Signal Analysis

| Test | At Red Gate | At 878ee294 | Signal |
|------|-------------|-------------|--------|
| `ac_1_valid_blocks_payload_returns_200_ok_with_stable_message_ts` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ac_1_text_only_payload_returns_200` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ac_2_missing_blocks_and_text_returns_400_invalid_payload` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ec_001_empty_json_object_returns_400_invalid_payload` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ac_3_unknown_top_level_field_returns_400_unknown_field` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ac_3_all_allowed_top_level_fields_are_accepted` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ac_4_rate_limit_returns_429_with_retry_after_and_ratelimited_body` | **RED** | **GREEN** | **Legitimate TDD signal — body `"ratelimited"` fix in FailureLayer** |
| `ec_002_fail_with_500_returns_internal_server_error` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ac_5_three_deliveries_captured_in_order` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ac_5_in_process_received_payloads_api_matches_http_endpoint` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ac_6_reset_clears_received_payloads_and_request_counter` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ac_6_post_dtu_reset_endpoint_clears_state` | GREEN | GREEN | GREEN-BY-DESIGN |
| `ec_004_message_ts_is_stable_across_deliveries` | GREEN | GREEN | GREEN-BY-DESIGN |
| `architecture_forbidden_dependencies_documented` | GREEN | GREEN | GREEN-BY-DESIGN |

**13/14 GREEN-BY-DESIGN. 1/14 legitimately RED then fixed.**

---

## Edge Case Coverage (in ac_tests suite, not separately recorded)

| EC | Description | Test Name | Result |
|----|-------------|-----------|--------|
| EC-001 | Empty JSON object `{}` → HTTP 400 `"invalid_payload"` | `ec_001_empty_json_object_returns_400_invalid_payload` | PASS |
| EC-002 | `fail_with: 500` failure mode → HTTP 500 | `ec_002_fail_with_500_returns_internal_server_error` | PASS |
| EC-004 | `message_ts` stable across two deliveries | `ec_004_message_ts_is_stable_across_deliveries` | PASS |

---

## Cross-Crate Fix Note

The AC-4 fix touched `prism-dtu-common/src/layers/failure.rs` — the shared `FailureLayer`
middleware used by ALL DTU clones (prism-dtu-slack, prism-dtu-pagerduty, prism-dtu-opsgenie).
Sibling clones' tests only assert on status codes (429), not response body, so the fix
introduces no regression in S-6.12 or S-6.13. The fix correctly makes the 429 body match
the Slack-spec literal `"ratelimited"` while PagerDuty/OpsGenie clones remain unaffected.

---

## Full Test Run Summary

**Total tests in `prism-dtu-slack` (feature=dtu):** 14
**Passed:** 14
**Failed:** 0

All tests confirmed via `cargo test -p prism-dtu-slack --features dtu --test ac_tests`.

---

## Recording Methodology

- **Tool:** VHS 0.10.0 (`/opt/homebrew/bin/vhs`)
- **Font:** FiraCode Nerd Font Mono (consistent with S-6.12/S-6.13 convention)
- **Theme:** Dracula
- **Dimensions:** 1000x600, FontSize 14, Padding 20
- **Shell:** bash
- **AC-1:** Covers both `blocks`-only and `text`-only success paths (two tests filtered together).
- **AC-2:** Covers the error path — missing both `blocks` and `text` → 400 `invalid_payload`.
  Also covers EC-001 (empty JSON object) implicitly via the same code path.
- **AC-3:** Covers unknown-field rejection (error path) and all-allowed-fields acceptance
  (success path) in one tape.
- **AC-4:** The legitimately-RED-then-fixed test. Demonstrates rate-limit threshold enforcement,
  429 status, Retry-After header, and exact body literal `"ratelimited"`.
- **AC-5:** Covers both the HTTP endpoint (`GET /dtu/received-payloads`) and in-process
  Rust API (`clone.received_payloads()`) for payload capture.
- **AC-6:** Covers both the Rust trait method (`clone.reset()`) and HTTP endpoint
  (`POST /dtu/reset`) state-clearing paths.

---

## File Inventory

```
docs/demo-evidence/S-6.11/
  ac-1-valid-blocks-payload.tape           (653 B)
  ac-1-valid-blocks-payload.gif            (149 KB)
  ac-2-missing-blocks-and-text.tape        (603 B)
  ac-2-missing-blocks-and-text.gif         (123 KB)
  ac-3-unknown-top-level-field.tape        (646 B)
  ac-3-unknown-top-level-field.gif         (144 KB)
  ac-4-rate-limit-429.tape                 (612 B)
  ac-4-rate-limit-429.gif                  (129 KB)
  ac-5-deliveries-captured-in-order.tape   (641 B)
  ac-5-deliveries-captured-in-order.gif    (148 KB)
  ac-6-reset-clears-state.tape             (620 B)
  ac-6-reset-clears-state.gif              (139 KB)
  evidence-report.md                       (this file)
```

Total: 12 demo files (6 tape + 6 gif) + 1 evidence report = **13 files**.
