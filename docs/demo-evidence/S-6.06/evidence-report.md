# Evidence Report — S-6.06: prism-dtu-common

| Field | Value |
|-------|-------|
| Story ID | S-6.06 |
| Story title | prism-dtu-common — shared DTU behavioral clone infrastructure |
| Story version | v1.4 |
| Report date | 2026-04-22 |
| Branch | `feature/S-6.06-dtu-common` |
| Crate path | `crates/prism-dtu-common/` |
| Evidence type | Artifact-based (library crate — no CLI or UI) |
| POL-010 compliance | Confirmed — all files under `docs/demo-evidence/S-6.06/` |

---

## Commit History (implementation)

| SHA | Description |
|-----|-------------|
| `3847f1e` | feat(S-6.06): stub architecture for prism-dtu-common |
| `cc87100` | feat(S-6.06): AC-5 seeded_rng with ChaCha20 determinism |
| `3138b4c` | feat(S-6.06): AC-9 load_fixture + load_fixture_as with clear panic on missing file |
| `d572eae` | feat(S-6.06): AC-4 LatencyLayer tower middleware with tokio sleep |
| `0662f9c` | feat(S-6.06): AC-1/AC-2/AC-3 FailureLayer AuthReject+RateLimit+BehavioralClone server binding |
| `b867bd8` | feat(S-6.06): AC-7 WebhookReceiver capture POST body + path + headers |
| `df10b78` | feat(S-6.06): AC-6 SyslogReceiver RFC 5424 UDP capture |
| `e0951bc` | feat(S-6.06): AC-8 FidelityValidator HTTP checks + field presence; test_utils assertion helpers |
| `3933fe4` | chore(S-6.06): remove stub-phase allow attrs + clippy clean + cargo fmt |
| `6a3064a` | fix(S-6.06/tests): AC-6 use receiver.bound_addr() for real port + fmt cleanup |

---

## AC Coverage Matrix

| AC | Short title | Evidence file | Test function | Verdict |
|----|-------------|---------------|---------------|---------|
| AC-1 | BehavioralClone::start() binds and is reachable | `AC-1-behavioral-clone-start.md` | `ac_1_behavioral_clone_start_binds_and_bound_addr_is_reachable` | SATISFIED |
| AC-2 | FailureLayer RateLimit returns 429 + Retry-After | `AC-2-failure-layer-rate-limit.md` | `ac_2_failure_layer_rate_limit_returns_429_after_threshold` | SATISFIED |
| AC-3 | FailureLayer AuthReject returns 401 unconditionally | `AC-3-failure-layer-auth-reject.md` | `ac_3_failure_layer_auth_reject_returns_401_unconditionally` | SATISFIED |
| AC-4 | LatencyLayer delays response by >= 80ms | `AC-4-latency-layer-delay.md` | `ac_4_latency_layer_delays_response_by_configured_ms` | SATISFIED |
| AC-5 | seeded_rng — same seed produces identical sequence | `AC-5-seeded-rng-determinism.md` | `ac_5_seeded_rng_same_seed_produces_identical_sequence` + `ac_5_seeded_rng_different_seeds_produce_different_sequences` | SATISFIED |
| AC-6 | SyslogReceiver captures RFC 5424 UDP messages | `AC-6-syslog-receiver.md` | `ac_6_syslog_receiver_captures_udp_rfc5424_message` | SATISFIED |
| AC-7 | WebhookReceiver captures POST path and body | `AC-7-webhook-receiver.md` | `ac_7_webhook_receiver_captures_post_body_and_path` | SATISFIED |
| AC-8 | FidelityValidator flags missing required field | `AC-8-fidelity-validator.md` | `ac_8_fidelity_validator_flags_missing_required_field` | SATISFIED |
| AC-9 | load_fixture returns parsed JSON | `AC-9-fixture-loader.md` | `ac_9_load_fixture_returns_parsed_json_for_existing_file` | SATISFIED |

---

## Green Gate Verification

All checks run in worktree `/Users/jmagady/dev/prism/.worktrees/S-6.06-dtu-common/` at commit `6a3064a`.

### cargo test

```
$ cargo test --features prism-dtu-common/dtu 2>&1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.38s

test ac_1_behavioral_clone_start_binds_and_bound_addr_is_reachable ... ok
test ac_2_failure_layer_rate_limit_returns_429_after_threshold ... ok
test ac_3_failure_layer_auth_reject_returns_401_unconditionally ... ok
test ac_4_latency_layer_delays_response_by_configured_ms ... ok
test ac_5_seeded_rng_same_seed_produces_identical_sequence ... ok
test ac_5_seeded_rng_different_seeds_produce_different_sequences ... ok
test ac_6_syslog_receiver_captures_udp_rfc5424_message ... ok
test ac_7_webhook_receiver_captures_post_body_and_path ... ok
test ac_8_fidelity_validator_flags_missing_required_field ... ok
test ac_9_load_fixture_returns_parsed_json_for_existing_file ... ok

Result: 9 integration tests + 2 sub-tests = 10 test runs — ALL PASSED
```

Full transcript: `test-run.txt`

### cargo clippy

```
$ cargo clippy --features prism-dtu-common/dtu -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
Exit 0 — CLEAN
```

### cargo fmt --check

```
$ cargo fmt --check
Exit 0 — CLEAN
(nightly-only rustfmt option warnings are informational, not errors)
```

---

## Public API Summary

See `public-api.md` for the full tree. Key exports:

- `BehavioralClone` — trait all 13 per-surface DTU crates must implement
- `StubConfig` / `FailureMode` — configuration types
- `LatencyLayer` / `FailureLayer` — Tower middleware for chaos injection
- `SyslogReceiver` — RFC 5424 UDP capture server
- `WebhookReceiver` / `CapturedRequest` — generic HTTP POST capture
- `FidelityValidator` / `FidelityReport` / `FidelityCheck` / `FidelityFailure` — behavioral fidelity checking
- `seeded_rng(seed: u64) -> ChaCha20Rng` — deterministic RNG
- `load_fixture` / `load_fixture_as` — JSON fixture loader
- `test_utils::{assert_field_present, assert_header_present, assert_status, build_test_client}` — assertion helpers

`cargo doc` deferred to CI — nightly-only `rustfmt` options generate stable-toolchain warnings
that do not affect compilation or doc generation correctness.

---

## Known Limitations

- AC-9 "missing file" error-path (panic on non-existent fixture) is intentionally not tested in the integration suite because the pattern would be tautological against a stub. A `#[should_panic]` test will be added in S-6.07 once the consumer crate provides real fixture files.
- `cargo doc` validation deferred to CI (stable toolchain, `--features dtu` docs job).
