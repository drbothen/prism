# S-6.07 Demo Evidence Report

**Story:** S-6.07 — prism-dtu-crowdstrike: DTU for CrowdStrike Falcon API (L4 adversarial)
**Branch:** feature/S-6.07-dtu-crowdstrike
**Format:** VHS terminal recordings (`.tape` + `.gif` + `.webm`) per POL-010
**Font:** FiraCode Nerd Font Mono (detected via `fc-list`)

---

## Coverage Manifest

| AC | Description | Demo File | Type | Status |
|----|-------------|-----------|------|--------|
| AC-1 | start + bound_addr + GET detections 200 | AC-1.tape / AC-1.gif / AC-1.webm | VHS | Recorded |
| AC-2 | Two-step pagination (Step 1 + Step 2) | AC-2.tape / AC-2.gif / AC-2.webm | VHS | Recorded |
| AC-3 | Contain write persists to containment_store | AC-3.tape / AC-3.gif / AC-3.webm | VHS | Recorded |
| AC-4 | RateLimit → 429 + Retry-After header | AC-4.tape / AC-4.gif / AC-4.webm | VHS | Recorded |
| AC-5 | OAuth2 token endpoint | AC-5.tape / AC-5.gif / AC-5.webm | VHS | Recorded |
| AC-6 | Determinism (seed=42 → same response twice) | AC-6.tape / AC-6.gif / AC-6.webm | VHS | Recorded |
| AC-7 | 401 without Authorization header | AC-7.tape / AC-7.gif / AC-7.webm | VHS | Recorded |
| AC-8a | reset() clears session registry (pre-reset session = empty resources) | AC-8a.tape / AC-8a.gif / AC-8a.webm | VHS | Recorded |
| AC-8b | Post-reset fresh Step 1+2 returns fixture baseline (containment_status: normal) | AC-8b.tape / AC-8b.gif / AC-8b.webm | VHS | Recorded |
| AC-9 | VP-033 integration test (write-before-delivery ordering) | AC-9.txt | Placeholder | Ignored (needs-prism-audit) |
| AC-10 | VP-036 integration test (SessionContext drop before E-SENSOR-002) | AC-10.txt | Placeholder | Ignored (needs-prism-audit) |

---

## What Each Demo Shows

**AC-1** (`AC-1.tape`): Runs `cargo test --features dtu --test ac_1_happy_path`.
Demonstrates that `CrowdstrikeClone::start()` binds an ephemeral loopback port, `bound_addr()` returns
a valid socket address, and `GET /detects/queries/detects/v1` with a Bearer token returns HTTP 200
with a `resources` array and pagination metadata.

**AC-2** (`AC-2.tape`): Runs `cargo test --features dtu --test ac_2_two_step_pagination`.
Demonstrates the two-step pipeline: Step 1 (`GET /devices/queries/devices/v1`) returns host IDs and
registers them in the session registry under `X-DTU-Session-Id`; Step 2 (`GET /devices/entities/devices/v2`)
returns matching host detail records for those IDs. Verifies the same pipeline for detections.

**AC-3** (`AC-3.tape`): Runs `cargo test --features dtu --test ac_3_contain_write`.
Demonstrates that `POST /devices/entities/devices-actions/v2?action_name=contain` with `{"ids": ["h-001"]}`
returns HTTP 202 with `containment_status: "contained"`, and that a subsequent GET to the host detail
endpoint reflects `containment_status: "contained"` (state persisted in `containment_store`). Also
demonstrates `lift_containment` transitions the device back to `"normal"`.

**AC-4** (`AC-4.tape`): Runs `cargo test --features dtu --test ac_4_rate_limit`.
Demonstrates that `FailureMode::RateLimit { after_n_requests: 3, retry_after_secs: 60 }` allows
3 requests to succeed (HTTP 200) and causes the 4th to receive HTTP 429 with `Retry-After: 60`.
Also verifies the rate-limit counter is shared across all endpoints.

**AC-5** (`AC-5.tape`): Runs `cargo test --features dtu --test ac_5_oauth`.
Demonstrates that `POST /oauth2/token` with a `client_credentials` body returns HTTP 200 with
`access_token: "dtu-fake-cs-token"`. Also demonstrates that `auth_mode: "reject"` returns HTTP 401,
and that the token from the OAuth endpoint works on downstream authenticated endpoints.

**AC-6** (`AC-6.tape`): Runs `cargo test --features dtu --test ac_6_determinism`.
Demonstrates that a clone started with `seed: 42` returns bit-for-bit identical responses on
repeated calls to `GET /detects/queries/detects/v1` with the same query params. Also verifies that
different seeds produce different responses (determinism is seed-specific, not global).

**AC-7** (`AC-7.tape`): Runs `cargo test --features dtu --test ac_7_auth`.
Demonstrates that requests to auth-required endpoints without an `Authorization` header receive
HTTP 401 with `{"errors": [{"code": 401, "message": "..."}]}`. Covers detection query, host query,
host detail, and contain-write endpoints.

**AC-8a** (`AC-8a.tape`): Runs `cargo test --features dtu --test ac_8_reset ac_8_reset_clears_session_registry`.
Demonstrates AC-8a: after `reset()`, a Step 2 request carrying the pre-reset `X-DTU-Session-Id` returns
HTTP 200 with an empty `resources` array (EC-003: session is a registry miss after the store is cleared).

**AC-8b** (`AC-8b.tape`): Runs `cargo test --features dtu --test ac_8_reset ac_8_reset_clears_containment_store`.
Demonstrates AC-8b: after `reset()`, a fresh Step 1 with a new `X-DTU-Session-Id` followed by Step 2
returns host records with `containment_status: "normal"` — the fixture baseline, confirming the
containment store was cleared.

**AC-9** (`AC-9.txt`): Placeholder. The VP-033 integration test
(`crowdstrike_vp033_write_intent_before_dtu_arrival`) is marked `#[ignore = "needs-prism-audit"]`
because it requires `prism-audit::InMemoryBackend` and `prism-sensors::SensorAdapter`, both scheduled
for story S-3.07. A smoke sub-test (`crowdstrike_vp033_contain_endpoint_returns_202_smoke`) IS active
and passes — confirmed by AC-3 demo. Recording will be added when S-3.07 lands.

**AC-10** (`AC-10.txt`): Placeholder. The VP-036 integration test
(`crowdstrike_vp036_session_context_drops_before_error`) is marked `#[ignore = "needs-prism-audit"]`
because it requires `prism-sensors::SessionContext` and the `Arc::weak_count` instrumentation,
both scheduled for story S-3.06. A smoke sub-test (`crowdstrike_vp036_step2_returns_500_on_internal_error_injection`)
IS active and passes. Recording will be added when S-3.06 lands.

---

## Test Coverage Summary

All 28 non-ignored tests in the `prism-dtu-crowdstrike` crate pass. The 2 ignored tests
(`needs-prism-audit`) have placeholder evidence files documenting the ignore reason and
the future story trigger.

```
cargo test --features dtu --package prism-dtu-crowdstrike 2>&1 | grep -E "test result|ignored"
```

Expected: `0 failed`, 2 ignored (the VP-033 and VP-036 main assertions).

---

## Re-Run Instructions

Prerequisites: Rust toolchain (see `rust-toolchain.toml`), VHS 0.10.0+, `FiraCode Nerd Font Mono`.

```bash
# From the worktree root:
cd /path/to/worktree

# Re-record a single AC:
vhs docs/demo-evidence/S-6.07/AC-1.tape

# Re-record all ACs (takes ~3 min total):
for ac in AC-1 AC-2 AC-3 AC-4 AC-5 AC-6 AC-7 AC-8a AC-8b; do
  vhs docs/demo-evidence/S-6.07/${ac}.tape
done

# Verify all tests pass independently:
cargo test --features dtu --package prism-dtu-crowdstrike
```

Each tape is self-contained: it `cd`s to the worktree and runs `cargo test --features dtu --test <binary>`.
The Rust binary is already compiled (from prior `cargo build`) so each tape takes ~15s.

---

## Ignored Tests Documentation

| Test | File | Ignore Reason | Unblock Story |
|------|------|---------------|---------------|
| `crowdstrike_vp033_write_intent_before_dtu_arrival` | `tests/integration_vp033.rs` | requires prism-audit InMemoryBackend | S-3.07 |
| `crowdstrike_vp036_session_context_drops_before_error` | `tests/integration_vp036.rs` | requires prism-sensors SessionContext | S-3.06 |
