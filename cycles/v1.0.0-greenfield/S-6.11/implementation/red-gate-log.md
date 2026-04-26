---
story: S-6.11
phase: red-gate
status: VERIFIED
timestamp: 2026-04-25
agent: test-writer
---

# Red Gate Log — S-6.11 prism-dtu-slack: DTU for Slack Webhook API (L2 stateful)

## Result

**PARTIAL RED — 1 test fails, 13 pass (GREEN-BY-DESIGN). Red Gate is valid.**

The stub author note stated: "webhook handler was fully implemented in stub (not todo!()) per
the established dtu pattern. Some ACs may be GREEN-BY-DESIGN at Red Gate." This is confirmed.

```
test result: FAILED. 13 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

The one RED test (`ac_4`) catches a real behavioral gap: the `FailureLayerShared` (tower
middleware) intercepts rate-limit requests and returns `429` with `Body::empty()`, not
the spec-required `"ratelimited"` body. The webhook handler's own rate-limit path would
return `"ratelimited"`, but the layer fires first. Implementer must ensure the 429 body
from the layer matches the spec (`"ratelimited"`).

## Stubs Created

None. Stub was pre-implemented by stub-author. Tests only.

## Red Gate Verification

### S-6.11 — AC Tests (`tests/ac_tests.rs`)

| Test Name | AC | Status | Notes |
|-----------|-----|--------|-------|
| `ac_1_valid_blocks_payload_returns_200_ok_with_stable_message_ts` | AC-1 | GREEN-BY-DESIGN | Stub fully implements the webhook handler |
| `ac_1_text_only_payload_returns_200` | AC-1 | GREEN-BY-DESIGN | Stub handles `text`-only payloads |
| `ac_2_missing_blocks_and_text_returns_400_invalid_payload` | AC-2 | GREEN-BY-DESIGN | Stub validates missing `blocks`/`text` |
| `ec_001_empty_json_object_returns_400_invalid_payload` | EC-001 | GREEN-BY-DESIGN | Stub handles empty `{}` |
| `ac_3_unknown_top_level_field_returns_400_unknown_field` | AC-3 | GREEN-BY-DESIGN | Stub rejects unknown top-level keys |
| `ac_3_all_allowed_top_level_fields_are_accepted` | AC-3 | GREEN-BY-DESIGN | Stub allow-list matches schema |
| `ac_4_rate_limit_returns_429_with_retry_after_and_ratelimited_body` | AC-4 | **RED (FAIL)** | 429 body is empty; spec requires `"ratelimited"` |
| `ec_002_fail_with_500_returns_internal_server_error` | EC-002 | GREEN-BY-DESIGN | Stub FailureLayer returns 500 |
| `ac_5_three_deliveries_captured_in_order` | AC-5 | GREEN-BY-DESIGN | Stub captures and returns payloads in order |
| `ac_5_in_process_received_payloads_api_matches_http_endpoint` | AC-5 | GREEN-BY-DESIGN | In-process API matches HTTP endpoint |
| `ac_6_reset_clears_received_payloads_and_request_counter` | AC-6 | GREEN-BY-DESIGN | Stub reset clears all state |
| `ac_6_post_dtu_reset_endpoint_clears_state` | AC-6 | GREEN-BY-DESIGN | POST /dtu/reset works via HTTP |
| `ec_004_message_ts_is_stable_across_deliveries` | EC-004 | GREEN-BY-DESIGN | Stub returns stable `"1234567890.123456"` |
| `architecture_forbidden_dependencies_documented` | arch | GREEN-BY-DESIGN | Crate name assertion always passes |

### Fidelity test (`tests/fidelity.rs`)

| Test Name | AC | Status | Notes |
|-----------|-----|--------|-------|
| `slack_dtu_fidelity` | AC-1 | GREEN-BY-DESIGN | Pre-existing fidelity test; stub fully implemented |

## RED Test Detail

### `ac_4_rate_limit_returns_429_with_retry_after_and_ratelimited_body`

**Failure location:** `tests/ac_tests.rs:309`

**Assertion failure:**
```
assertion `left == right` failed: AC-4: body must be literal '"ratelimited"'
  left: ""
 right: "\"ratelimited\""
```

**Root cause:** The `FailureLayerShared` tower middleware intercepts the request before
it reaches the handler. The layer's `apply_failure_mode()` in `prism-dtu-common` returns
`Body::empty()` for 429 responses (see `layers/failure.rs:167`). The handler code at
`webhook.rs:78-83` would return `"ratelimited"` but is never reached.

**Spec reference:** AC-4 states: body `"ratelimited"`. Story Task 5 confirms the body.

**Implementer action:** Update `FailureLayerShared`'s 429 response body to include
`"ratelimited"`, or restructure so the handler's 429 path is reached (e.g. by removing
rate-limit from the tower layer and relying solely on the handler's counter).

## GREEN-BY-DESIGN Analysis

13 of 14 tests pass at Red Gate because the stub author fully implemented all AC behavior
per the established DTU pattern. This is correct per the story instructions:

> "Stub author note: webhook handler was fully implemented in stub (not todo!()) per the
> established dtu pattern. Some ACs may be GREEN-BY-DESIGN at Red Gate. That's OK —
> write the tests anyway."

These tests remain valuable: they anchor exact spec-literal values (particularly `message_ts`
`"1234567890.123456"`) and exact status codes, ensuring a future refactor cannot silently
change observable behavior.

## Regression Check

| Category | Count | Status |
|----------|-------|--------|
| Pre-existing workspace tests | 1058 | All pass |
| New ac_tests.rs tests | 14 | 13 pass, 1 fail (RED) |
| Workspace total | 1072 | 1071 pass, 1 fail |

No pre-existing test was broken by these additions.

## Files Written

| File | Purpose |
|------|---------|
| `crates/prism-dtu-slack/tests/ac_tests.rs` | 14 AC tests (AC-1..AC-6, EC-001, EC-002, EC-004) |
| `crates/prism-dtu-slack/Cargo.toml` | Added `[[test]] ac_tests` binary + `http = "1"` dev-dep |

## Hand-Off to Implementer

**Story ready for implementation:** S-6.11

**Implementation guidance:**

1. Fix `FailureLayerShared`'s 429 response in `prism-dtu-common/src/layers/failure.rs`:
   - Change `Body::empty()` to `Body::from("\"ratelimited\"")` for the `RateLimit` arm,
     OR restructure so the webhook handler's own rate-limit path executes.
   - Note: this change is in `prism-dtu-common`, not `prism-dtu-slack`. Test runs on
     `prism-dtu-slack` — confirm the change makes `ac_4` pass without breaking
     `prism-dtu-common` tests.

2. All other ACs are already implemented. The implementer need only verify the RED test
   passes after the fix and that no GREEN-BY-DESIGN test regresses.

3. `ac_6_reset_clears_received_payloads_and_request_counter` exercises a subtle interaction:
   after reset, the first new request at rate-limit threshold=1 should return 200 (count=1
   equals threshold, not exceeds it). The `>` comparison in the handler (`count > after_n_requests`)
   is correct for this. Confirm the FailureLayerShared also uses `>` not `>=`.
