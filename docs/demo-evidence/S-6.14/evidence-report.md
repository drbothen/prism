# Evidence Report — S-6.14

| Field | Value |
|---|---|
| Story ID | S-6.14 |
| Title | prism-dtu-threatintel: DTU for Threat Intel Aggregator — L2 (stateful) |
| Date | 2026-04-21 |
| Impl commit | a84a253 |
| Branch | feature/S-6.14-dtu-threatintel |
| Evidence commit | (this commit) |
| POL-010 path | `docs/demo-evidence/S-6.14/` |
| Product type | Library crate (no UI) — artifact-based evidence |
| Recording tool | N/A — library crate; evidence via cargo test output + per-AC markdown |

## AC Coverage Matrix

| AC | Title | Test File | Test Function | Result |
|----|-------|-----------|---------------|--------|
| AC-1 | Malicious IP lookup | `tests/ac_1_malicious_ip_lookup.rs` | `ac_1_malicious_ip_returns_threat_score_85_and_greynoise_source` | PASS |
| AC-2 | Benign IP lookup | `tests/ac_2_benign_ip_lookup.rs` | `ac_2_benign_ip_returns_not_malicious_with_score_below_20` | PASS |
| AC-3 | Unknown IP benign defaults | `tests/ac_3_unknown_ip_benign_defaults.rs` | `ac_3_unknown_ip_returns_benign_defaults` | PASS |
| AC-4 | Malicious hash lookup | `tests/ac_4_malicious_hash_lookup.rs` | `ac_4_pre_registered_malicious_hash_returns_virustotal_source_and_score_above_80` | PASS |
| AC-5 | Auth reject missing key | `tests/ac_5_auth_reject_missing_key.rs` | `ac_5_missing_api_key_returns_401_with_error_body` | PASS |
| AC-6 | Rate limit 429 | `tests/ac_6_rate_limit_429.rs` | `ac_6_rate_limit_after_3_returns_429_on_4th_request_with_retry_after_30` | PASS |
| AC-7 | Dynamic registry configure | `tests/ac_7_dynamic_registry_configure.rs` | `ac_7_dynamic_registry_addition_serves_malicious_fixture` | PASS |

Coverage: **7/7 PASS** — 100%

## Green Gate Summary

Command: `cargo test --features prism-dtu-threatintel/dtu`
Exit code: 0
S-6.14 tests: 7 passed, 0 failed
Full workspace tests (including prism-dtu-common): all green

Full output: [test-run.txt](test-run.txt)

## Evidence Files

| File | Purpose |
|---|---|
| [AC-1-malicious-ip-lookup.md](AC-1-malicious-ip-lookup.md) | AC-1 statement, impl excerpt, test output, mapping |
| [AC-2-benign-ip-lookup.md](AC-2-benign-ip-lookup.md) | AC-2 statement, impl excerpt, test output, mapping |
| [AC-3-unknown-ip-benign-defaults.md](AC-3-unknown-ip-benign-defaults.md) | AC-3 statement, impl excerpt, test output, mapping |
| [AC-4-malicious-hash-lookup.md](AC-4-malicious-hash-lookup.md) | AC-4 statement, impl excerpt, test output, mapping |
| [AC-5-auth-reject-missing-key.md](AC-5-auth-reject-missing-key.md) | AC-5 statement, impl excerpt, test output, mapping |
| [AC-6-rate-limit-429.md](AC-6-rate-limit-429.md) | AC-6 statement, impl excerpt, test output, mapping |
| [AC-7-dynamic-registry-configure.md](AC-7-dynamic-registry-configure.md) | AC-7 statement, impl excerpt, test output, mapping |
| [public-api.md](public-api.md) | Exported structs, traits, HTTP surface, default registry |
| [usage-example.md](usage-example.md) | 20-line integration test consumer snippet |
| [test-run.txt](test-run.txt) | Full `cargo test --features prism-dtu-threatintel/dtu` output |

## Architecture Notes

- Crate is gated behind `#[cfg(any(test, feature = "dtu"))]` — never compiled into production binaries.
- `ThreatIntelClone` implements `BehavioralClone` from `prism-dtu-common` (S-6.06).
- L2 fidelity: stateful fixture registry + rate-limit counter; no live network required.
- All lookups are deterministic for a given fixture registry state.
- Forbidden deps (`prism-sensors`, `prism-query`, etc.) enforced via `deny.toml`.
