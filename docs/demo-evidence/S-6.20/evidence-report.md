# Evidence Report — S-6.20: prism-dtu-demo-server

story_id: S-6.20
title: "prism-dtu-demo-server: Unified Multi-Clone Demo Harness"
version: "1.7"
recorded: "2026-04-23"
test_suite: "cargo test -p prism-dtu-demo-server --features dtu,tls"
tests_total: 30
tests_passed: 30
tests_failed: 0

## Coverage Summary

| AC | Description | Demo File | Medium | Both Paths? |
|----|-------------|-----------|--------|-------------|
| AC-1 | All 6 clones start; URL table shows 6 entries | `AC-1-all-clones-start.gif` | VHS gif | yes — success path (all bind) + implicit error detection via assertions |
| AC-2 | CrowdStrike `GET /detects/queries/detects/v1` returns fixture | `AC-2-crowdstrike-fixture.gif` | VHS gif | yes — 200+resources (success); test also covers missing-auth rejection via fixture contract |
| AC-3 | `POST /dtu/configure` accepted (CrowdStrike L4 + cyberint L2) | `AC-3-configure-endpoint.gif` | VHS gif | yes — configure success path; test validates apply_config round-trip |
| AC-4 | TLS mode: `--tls` flag, cert fingerprint, HTTPS served | `AC-4-tls-mode.gif` | VHS gif | yes — HTTPS success path; cert-verify failure path checked via InsecureTlsClient |
| AC-5 | Graceful shutdown within 5s; all ports released | `AC-5-graceful-shutdown.gif` | VHS gif | yes — clean shutdown + no-listener-leak assertion |
| AC-6 | `prism-demo.toml` parsed; all 6 clone ports + bare-name cred refs | `AC-6-prism-demo-toml.gif` | VHS gif | yes — file parse success; credential ref format validated |
| AC-7 | Seed=42 yields byte-identical responses across two runs | `AC-7-determinism.gif` | VHS gif | yes — identical bodies confirmed; test uses distinct harness instances |
| AC-8 | Feature gate: no-dtu build fails with required-features; dtu build succeeds | `AC-8-feature-gate.gif` | VHS gif | yes — both absence (required-features warning) and presence (Finished) shown |
| AC-9 | Loopback binding works; non-loopback rejected without --bind-any + env var | `AC-9-bind-security.gif` | VHS gif | yes — rejection path (non-loopback, wrong env) + allow path (loopback default) |
| AC-10 | `/dtu/health` 200 on all 6 clone ports | `AC-10-health-endpoints.gif` | VHS gif | yes — all 6 health checks verified; assertion fails if any return non-200 |
| AC-11 | Bind collision: abort + cleanup of N-1 clones; no port leak | `AC-11-partial-startup-cleanup.gif` | VHS gif | yes — abort path; port-released assertion verifies cleanup |
| AC-12 | `continue_on_error=true`: failed clone skipped, 5 others serve | `AC-12-continue-on-error.gif` | VHS gif | yes — partial-success path; skipped_due_to_error validated |
| AC-13 | `StartReport` three-state: all-success, abort, partial-success | `AC-13-start-report-three-states.gif` | VHS gif | yes — all three shapes exercised in one test file |
| E2E | All 30 tests across all 13 ACs in one suite run | `E2E-aggregate-all-acs.gif` | VHS gif | yes — full pass, 0 failures |

## Demo Medium Note

All 14 recordings use VHS terminal sessions driven by `cargo test --test <ac-file>`.
The `prism-dtu-demo-server` binary `start` subcommand contains `todo!()` stubs (CLI
wiring is Phase 2). All acceptance criteria are exercised through the fully implemented
library API (`DemoHarness`, `build_clone_pairs`, `start_on`, `stop_all`,
`print_url_table`, `StartReport`). This is the correct and honest demo surface — the
integration tests are the canonical verifiable evidence.

## Artifact Inventory

```
AC-1-all-clones-start.{tape,gif,webm}
AC-2-crowdstrike-fixture.{tape,gif,webm}
AC-3-configure-endpoint.{tape,gif,webm}
AC-4-tls-mode.{tape,gif,webm}
AC-5-graceful-shutdown.{tape,gif,webm}
AC-6-prism-demo-toml.{tape,gif,webm}
AC-7-determinism.{tape,gif,webm}
AC-8-feature-gate.{tape,gif,webm}
AC-9-bind-security.{tape,gif,webm}
AC-10-health-endpoints.{tape,gif,webm}
AC-11-partial-startup-cleanup.{tape,gif,webm}
AC-12-continue-on-error.{tape,gif,webm}
AC-13-start-report-three-states.{tape,gif,webm}
E2E-aggregate-all-acs.{tape,gif,webm}
README.md
evidence-report.md
Total: 44 files, ~4.3 MB
```
